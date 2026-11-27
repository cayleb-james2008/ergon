//! Model Context Protocol (MCP) Client & Gated Inbound JSON-RPC Server.
//!
//! Persists server configurations and discovered tool caches in
//! `<person_data_dir>/mcp_servers.json` (0600 POSIX). Outbound `stdio` and
//! `http_sse` tool calls are checked by Tripwire egress/secret/sandbox guards
//! and parked as `mcp_tool_call` approval decisions when required by server
//! config or `Strict Client` policy.

use crate::tools::{atomic_write_bytes_with_mode, sandbox_path, sandbox_path_resolving};
use crate::{connectors, docs, records, search, secret_store, tables, tools, tripwire, Config, WorkspacePolicy};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Condvar, Mutex, OnceLock};
use std::time::Duration;

static MCP_SEQ: AtomicU64 = AtomicU64::new(0);
static MCP_LOCK: Mutex<()> = Mutex::new(());
static IN_FLIGHT_MCP_SETTLE: OnceLock<(Mutex<HashSet<String>>, Condvar)> = OnceLock::new();
static SETTLED_MCP_OUTCOMES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

const MAX_MCP_TOOLS_PER_SERVER: usize = 256;
const MAX_MCP_RESPONSE_CHARS: usize = 16_000;
const MCP_IO_TIMEOUT: Duration = Duration::from_secs(12);

fn with_mcp_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = MCP_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

fn next_seq() -> u64 {
    MCP_SEQ.fetch_add(1, Ordering::Relaxed)
}

pub fn mcp_servers_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("mcp_servers.json")
}

fn is_valid_mcp_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

fn is_valid_mcp_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

#[derive(Clone, Debug)]
pub struct McpToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl McpToolSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.input_schema,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let name = v["name"].as_str()?.trim().to_string();
        if !is_valid_mcp_tool_name(&name) {
            return None;
        }
        let description = v["description"].as_str().unwrap_or("").trim().to_string();
        let input_schema = if !v["input_schema"].is_null() {
            v["input_schema"].clone()
        } else if !v["inputSchema"].is_null() {
            v["inputSchema"].clone()
        } else {
            json!({ "type": "object", "properties": {} })
        };
        Some(Self {
            name,
            description,
            input_schema,
        })
    }
}

#[derive(Clone, Debug)]
pub struct McpServerSpec {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command_or_url: String,
    pub args: Vec<String>,
    pub env_secret_keys: Vec<String>,
    pub auth_secret_key: Option<String>,
    pub require_approval_by_default: bool,
    pub enabled: bool,
    pub cached_tools: Vec<McpToolSpec>,
    pub last_discovered_ms: Option<u64>,
    pub last_error: Option<String>,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl McpServerSpec {
    fn referenced_secret_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for entry in &self.env_secret_keys {
            let sec_key = match entry.split_once('=') {
                Some((_, k)) => k.trim(),
                None => entry.trim(),
            };
            if !sec_key.is_empty() && !keys.iter().any(|k| k == sec_key) {
                keys.push(sec_key.to_string());
            }
        }
        if let Some(auth_k) = self.auth_secret_key.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if !keys.iter().any(|k| k == auth_k) {
                keys.push(auth_k.to_string());
            }
        }
        keys
    }

    pub fn to_public_json_with_secrets(
        &self,
        policy: &WorkspacePolicy,
        stored_secret_keys: &HashSet<String>,
    ) -> Value {
        let effective_require_approval =
            self.require_approval_by_default || policy.profile == "strict_client";
        let blocked_by_policy = self.transport == "http_sse"
            && (policy.block_outbound_fetch || policy.profile == "air_gapped");
        let refs = self.referenced_secret_keys();
        let has_secret = !refs.is_empty() && refs.iter().all(|k| stored_secret_keys.contains(k));
        json!({
            "id": self.id,
            "name": self.name,
            "transport": self.transport,
            "command_or_url": self.command_or_url,
            "args": self.args,
            "env_secret_keys": self.env_secret_keys,
            "auth_secret_key": self.auth_secret_key,
            "has_secret": has_secret,
            "require_approval_by_default": self.require_approval_by_default,
            "effective_require_approval": effective_require_approval,
            "blocked_by_policy": blocked_by_policy,
            "enabled": self.enabled,
            "cached_tools": self.cached_tools.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            "last_discovered_ms": self.last_discovered_ms,
            "last_error": self.last_error,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn to_public_json(&self, policy: &WorkspacePolicy) -> Value {
        self.to_public_json_with_secrets(policy, &HashSet::new())
    }

    fn to_stored_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "transport": self.transport,
            "command_or_url": self.command_or_url,
            "args": self.args,
            "env_secret_keys": self.env_secret_keys,
            "auth_secret_key": self.auth_secret_key,
            "require_approval_by_default": self.require_approval_by_default,
            "enabled": self.enabled,
            "cached_tools": self.cached_tools.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            "last_discovered_ms": self.last_discovered_ms,
            "last_error": self.last_error,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        if !is_valid_mcp_id(&id) {
            return None;
        }
        let name = v["name"].as_str().unwrap_or("MCP Server").to_string();
        let transport = v["transport"].as_str()?.trim().to_ascii_lowercase();
        if transport != "stdio" && transport != "http_sse" {
            return None;
        }
        let command_or_url = v["command_or_url"].as_str()?.trim().to_string();
        if command_or_url.is_empty() {
            return None;
        }
        let args = v["args"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let env_secret_keys = parse_env_secret_keys(&v["env_secret_keys"]).unwrap_or_default();
        let auth_secret_key = v["auth_secret_key"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let cached_tools = v["cached_tools"]
            .as_array()
            .map(|a| a.iter().filter_map(McpToolSpec::from_json).collect())
            .unwrap_or_default();
        Some(Self {
            id,
            name,
            transport,
            command_or_url,
            args,
            env_secret_keys,
            auth_secret_key,
            require_approval_by_default: v["require_approval_by_default"].as_bool().unwrap_or(true),
            enabled: v["enabled"].as_bool().unwrap_or(true),
            cached_tools,
            last_discovered_ms: v["last_discovered_ms"].as_u64(),
            last_error: v["last_error"].as_str().map(str::to_string),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug, Default)]
struct McpStore {
    servers: Vec<McpServerSpec>,
    inbound_enabled: bool,
}

fn migrate_legacy_mcp_plaintext_secrets(person_data_dir: &Path, doc: &mut Value) -> bool {
    let mut new_secrets: BTreeMap<String, String> = BTreeMap::new();
    let mut changed = false;
    if let Some(servers) = doc.get_mut("servers").and_then(|v| v.as_array_mut()) {
        for (idx, srv) in servers.iter_mut().enumerate() {
            let Some(obj) = srv.as_object_mut() else {
                continue;
            };
            let sid: String = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("mcp")
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') { c } else { '_' })
                .take(24)
                .collect();
            let sid = if sid.is_empty() { format!("mcp_{idx}") } else { sid };

            for bearer_field in ["bearer_token", "api_key", "secret_value"] {
                if let Some(tok) = obj
                    .get(bearer_field)
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && !secret_store::is_vault_ref(s))
                    .map(str::to_string)
                {
                    changed = true;
                    obj.remove(bearer_field);
                    let key = obj
                        .get("auth_secret_key")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("mcp_auth_{sid}"));
                    obj.insert("auth_secret_key".to_string(), json!(key));
                    new_secrets.insert(key, tok);
                }
            }

            if let Some(env_map) = obj.get("env").and_then(|v| v.as_object()).cloned() {
                let mut migrated_env_keys =
                    parse_env_secret_keys(obj.get("env_secret_keys").unwrap_or(&Value::Null))
                        .unwrap_or_default();
                let mut any_env = false;
                for (env_name, env_val) in env_map {
                    if let Some(raw_val) = env_val
                        .as_str()
                        .map(str::trim)
                        .filter(|s| !s.is_empty() && !secret_store::is_vault_ref(s))
                    {
                        any_env = true;
                        let env_frag: String = env_name
                            .chars()
                            .map(|c| {
                                if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
                                    c.to_ascii_lowercase()
                                } else {
                                    '_'
                                }
                            })
                            .take(24)
                            .collect();
                        let sec_key = format!("mcp_env_{sid}_{env_frag}");
                        let mapping = format!("{env_name}={sec_key}");
                        if !migrated_env_keys.contains(&mapping) {
                            migrated_env_keys.push(mapping);
                        }
                        new_secrets.insert(sec_key, raw_val.to_string());
                    }
                }
                if any_env {
                    changed = true;
                    obj.remove("env");
                    obj.insert("env_secret_keys".to_string(), json!(migrated_env_keys));
                }
            }
        }
    }

    if !changed {
        return false;
    }
    connectors::store_connector_secrets(person_data_dir, &new_secrets).is_ok()
}

impl McpStore {
    fn load_unlocked(person_data_dir: &Path) -> Self {
        let path = mcp_servers_path(person_data_dir);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
            return Self::default();
        };
        let migrated = migrate_legacy_mcp_plaintext_secrets(person_data_dir, &mut v);
        let store = Self {
            servers: v["servers"]
                .as_array()
                .map(|a| a.iter().filter_map(McpServerSpec::from_json).collect())
                .unwrap_or_default(),
            inbound_enabled: v["inbound_enabled"].as_bool().unwrap_or(false),
        };
        if migrated {
            let _ = store.save_unlocked(person_data_dir);
        }
        store
    }

    fn save_unlocked(&self, person_data_dir: &Path) -> Result<(), String> {
        let servers_json: Vec<Value> = self.servers.iter().map(|s| s.to_stored_json()).collect();
        let doc = json!({
            "version": 1,
            "inbound_enabled": self.inbound_enabled,
            "servers": servers_json,
        });
        let bytes =
            serde_json::to_vec_pretty(&doc).map_err(|e| format!("serialize mcp servers: {e}"))?;
        atomic_write_bytes_with_mode(&mcp_servers_path(person_data_dir), &bytes, Some(0o600))
            .map_err(|e| format!("save mcp_servers.json: {e}"))
    }
}

fn parse_env_secret_keys(val: &Value) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    match val {
        Value::Null => {}
        Value::Array(items) => {
            for item in items {
                let s = item
                    .as_str()
                    .map(str::trim)
                    .ok_or_else(|| "env_secret_keys entries must be strings".to_string())?;
                if !s.is_empty() {
                    if s.contains(['\r', '\n', '\0']) {
                        return Err("env_secret_keys contains invalid characters".to_string());
                    }
                    out.push(s.to_string());
                }
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                let env_name = k.trim();
                let sec_key = v
                    .as_str()
                    .map(str::trim)
                    .ok_or_else(|| format!("env_secret_keys['{env_name}'] must be a string"))?;
                if env_name.is_empty()
                    || sec_key.is_empty()
                    || env_name.contains(['\r', '\n', '\0', '='])
                    || sec_key.contains(['\r', '\n', '\0'])
                {
                    return Err(format!("invalid env_secret_keys mapping for '{env_name}'"));
                }
                out.push(format!("{env_name}={sec_key}"));
            }
        }
        _ => return Err("env_secret_keys must be an array or object".to_string()),
    }
    Ok(out)
}

pub fn list_mcp_servers_json(person_data_dir: &Path, policy: &WorkspacePolicy) -> Value {
    let stored_keys: HashSet<String> = connectors::get_connector_secret_map(person_data_dir)
        .into_keys()
        .collect();
    with_mcp_lock(|| {
        let store = McpStore::load_unlocked(person_data_dir);
        let servers: Vec<Value> = store
            .servers
            .iter()
            .map(|s| s.to_public_json_with_secrets(policy, &stored_keys))
            .collect();
        json!({
            "servers": servers,
            "inbound_enabled": is_inbound_enabled_unlocked(&store),
            "policy_profile": policy.profile,
        })
    })
}

pub fn set_inbound_enabled(person_data_dir: &Path, enabled: bool) -> Result<Value, String> {
    with_mcp_lock(|| {
        let mut store = McpStore::load_unlocked(person_data_dir);
        store.inbound_enabled = enabled;
        store.save_unlocked(person_data_dir)?;
        Ok(json!({
            "ok": true,
            "inbound_enabled": store.inbound_enabled,
        }))
    })
}

fn validate_server_command_or_url(
    person_data_dir: &Path,
    transport: &str,
    command_or_url: &str,
) -> Result<(), String> {
    if command_or_url.is_empty() || command_or_url.contains(['\r', '\n', '\0']) {
        return Err("command_or_url must be non-empty and cannot contain control characters".to_string());
    }
    if transport == "stdio" {
        if tripwire::sensitive_target(command_or_url).is_some() {
            return Err("stdio MCP command cannot target a sensitive path".to_string());
        }
        Ok(())
    } else if transport == "http_sse" {
        let lower = command_or_url.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err("http_sse MCP server must use http:// or https:// URL".to_string());
        }
        if command_or_url.contains("{{secret:") {
            return Err("command_or_url cannot contain secret placeholders".to_string());
        }
        tripwire::check_egress(command_or_url, "connector_config")?;
        let default_port = if lower.starts_with("https://") { 443 } else { 80 };
        let (host, port) = connectors::parse_url_host_port_pub(command_or_url, default_port);
        tripwire::check_egress_host_for(person_data_dir, &host, port, "connector_config")?;
        Ok(())
    } else {
        Err("transport must be 'stdio' or 'http_sse'".to_string())
    }
}

pub fn upsert_mcp_server(
    person_data_dir: &Path,
    body: &Value,
    policy: &WorkspacePolicy,
) -> Result<(Value, bool), (u16, String)> {
    if !body.is_object() {
        return Err((400, "request body must be a JSON object".to_string()));
    }
    let name: String = body["name"]
        .as_str()
        .unwrap_or("")
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    if name.is_empty() {
        return Err((400, "MCP server name is required".to_string()));
    }
    let transport = body["transport"]
        .as_str()
        .unwrap_or("stdio")
        .trim()
        .to_ascii_lowercase();
    if transport != "stdio" && transport != "http_sse" {
        return Err((400, "transport must be 'stdio' or 'http_sse'".to_string()));
    }
    let command_or_url = body["command_or_url"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    validate_server_command_or_url(person_data_dir, &transport, &command_or_url)
        .map_err(|e| (400, e))?;

    let args = body["args"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for a in &args {
        if a.contains('\0') {
            return Err((400, "args cannot contain null bytes".to_string()));
        }
    }
    let env_secret_keys = parse_env_secret_keys(&body["env_secret_keys"]).map_err(|e| (400, e))?;
    let auth_secret_key = body["auth_secret_key"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Some(ref ak) = auth_secret_key {
        if ak.len() > 64 || !ak.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
            return Err((400, format!("invalid auth_secret_key '{ak}'")));
        }
    }
    let mut new_secrets: BTreeMap<String, String> = BTreeMap::new();
    if let Some(sec_obj) = body["secrets"].as_object() {
        for (k, v) in sec_obj {
            let sk = k.trim();
            let sv = v
                .as_str()
                .ok_or_else(|| (400, format!("secret '{sk}' must be a string")))?;
            if !sv.is_empty() {
                new_secrets.insert(sk.to_string(), sv.to_string());
            }
        }
    }
    if let (Some(sk), Some(sv)) = (
        body["secret_key"].as_str().map(str::trim).filter(|s| !s.is_empty()),
        body["secret_value"].as_str(),
    ) {
        if !sv.is_empty() {
            new_secrets.insert(sk.to_string(), sv.to_string());
        }
    }
    if !new_secrets.is_empty() {
        connectors::store_connector_secrets(person_data_dir, &new_secrets).map_err(|e| {
            if e.contains("invalid secret key") || e.contains("newlines") {
                (400, e)
            } else {
                (500, e)
            }
        })?;
    }

    let require_approval_by_default = body["require_approval_by_default"].as_bool().unwrap_or(true);
    let enabled = body["enabled"].as_bool().unwrap_or(true);
    let requested_id = body["id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(rid) = requested_id {
        if !is_valid_mcp_id(rid) {
            return Err((400, "invalid MCP server id".to_string()));
        }
    }
    let now_ms = records::now_millis();
    let stored_keys: HashSet<String> = connectors::get_connector_secret_map(person_data_dir)
        .into_keys()
        .collect();

    with_mcp_lock(|| {
        let mut store = McpStore::load_unlocked(person_data_dir);
        let existing_idx = requested_id.and_then(|rid| store.servers.iter().position(|s| s.id == rid));
        let (srv, created) = if let Some(idx) = existing_idx {
            let mut updated = store.servers[idx].clone();
            updated.name = name;
            updated.transport = transport;
            updated.command_or_url = command_or_url;
            updated.args = args;
            updated.env_secret_keys = env_secret_keys;
            if auth_secret_key.is_some() {
                updated.auth_secret_key = auth_secret_key;
            }
            updated.require_approval_by_default = require_approval_by_default;
            updated.enabled = enabled;
            updated.updated_ms = now_ms;
            store.servers[idx] = updated.clone();
            (updated, false)
        } else {
            let id = requested_id
                .map(str::to_string)
                .unwrap_or_else(|| format!("mcp-{now_ms}-{}", next_seq()));
            let srv = McpServerSpec {
                id,
                name,
                transport,
                command_or_url,
                args,
                env_secret_keys,
                auth_secret_key,
                require_approval_by_default,
                enabled,
                cached_tools: Vec::new(),
                last_discovered_ms: None,
                last_error: None,
                created_ms: now_ms,
                updated_ms: now_ms,
            };
            store.servers.push(srv.clone());
            (srv, true)
        };
        store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
        Ok((srv.to_public_json_with_secrets(policy, &stored_keys), created))
    })
}

pub fn delete_mcp_server(person_data_dir: &Path, id: &str) -> Result<bool, String> {
    with_mcp_lock(|| {
        let mut store = McpStore::load_unlocked(person_data_dir);
        let before = store.servers.len();
        store.servers.retain(|s| s.id != id);
        if store.servers.len() == before {
            return Ok(false);
        }
        store.save_unlocked(person_data_dir)?;
        Ok(true)
    })
}

fn resolve_server_env(
    person_data_dir: &Path,
    server: &McpServerSpec,
) -> Result<Vec<(String, String)>, String> {
    if server.env_secret_keys.is_empty() {
        return Ok(Vec::new());
    }
    let secrets = connectors::get_connector_secret_map_result(person_data_dir)?;
    let mut env_pairs = Vec::new();
    for entry in &server.env_secret_keys {
        let (env_var, secret_key) = match entry.split_once('=') {
            Some((e, k)) => (e.trim(), k.trim()),
            None => (entry.trim(), entry.trim()),
        };
        if let Some(val) = secrets.get(secret_key) {
            env_pairs.push((env_var.to_string(), val.clone()));
        }
    }
    Ok(env_pairs)
}

fn read_stdio_jsonrpc_response(
    rx: &mpsc::Receiver<String>,
    expected_id: u64,
) -> Result<Value, String> {
    let deadline = std::time::Instant::now() + MCP_IO_TIMEOUT;
    loop {
        let rem = deadline.saturating_duration_since(std::time::Instant::now());
        if rem.is_zero() {
            return Err("stdio MCP server timed out waiting for JSON-RPC response".to_string());
        }
        let line = rx
            .recv_timeout(rem)
            .map_err(|_| "stdio MCP server closed stdout or timed out".to_string())?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if msg["id"].as_u64() == Some(expected_id) {
            return Ok(msg);
        }
    }
}

fn run_stdio_mcp_session(
    cfg: &Config,
    server: &McpServerSpec,
    followup_method: &str,
    followup_params: Value,
) -> Result<Value, String> {
    let env_pairs = resolve_server_env(&cfg.data_dir, server)?;
    let mut cmd = Command::new(&server.command_or_url);
    cmd.args(&server.args)
        .current_dir(&cfg.workspace_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env_remove("ERGON_MODEL_API_KEY")
        .env_remove("ERGON_API_KEY")
        .env_remove("ERGON_GATE_TOKEN")
        .env_remove("ERGON_MCP_INBOUND_TOKEN")
        .env_remove("ERGON_SECRET_KEY");
    for (k, v) in env_pairs {
        cmd.env(k, v);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn stdio MCP server '{}': {e}", server.command_or_url))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "stdio MCP server missing stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdio MCP server missing stdout".to_string())?;

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let session_res = (|| -> Result<Value, String> {
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "ergon",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }
        });
        writeln!(stdin, "{init_req}").map_err(|e| format!("write MCP initialize: {e}"))?;
        stdin.flush().map_err(|e| format!("flush MCP initialize: {e}"))?;
        let init_resp = read_stdio_jsonrpc_response(&rx, 1)?;
        if let Some(err) = init_resp.get("error").filter(|v| !v.is_null()) {
            return Err(format!("MCP initialize error: {err}"));
        }

        let notif = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        let _ = writeln!(stdin, "{notif}");
        let _ = stdin.flush();

        let call_req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": followup_method,
            "params": followup_params,
        });
        writeln!(stdin, "{call_req}").map_err(|e| format!("write MCP {followup_method}: {e}"))?;
        stdin.flush().map_err(|e| format!("flush MCP {followup_method}: {e}"))?;
        let call_resp = read_stdio_jsonrpc_response(&rx, 2)?;
        if let Some(err) = call_resp.get("error").filter(|v| !v.is_null()) {
            return Err(format!("MCP {followup_method} error: {err}"));
        }
        Ok(call_resp["result"].clone())
    })();

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    session_res
}

fn parse_http_or_sse_jsonrpc(ct: &str, body: &[u8], expected_id: u64) -> Result<Value, String> {
    let text = String::from_utf8_lossy(body);
    if ct.to_ascii_lowercase().contains("text/event-stream") || text.contains("\ndata:") || text.starts_with("data:") {
        let mut last_match = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(payload) = trimmed.strip_prefix("data:") {
                let data_str = payload.trim();
                if let Ok(v) = serde_json::from_str::<Value>(data_str) {
                    if v["id"].as_u64() == Some(expected_id) {
                        return Ok(v);
                    }
                    last_match = Some(v);
                }
            }
        }
        if let Some(v) = last_match {
            return Ok(v);
        }
        return Err("MCP SSE stream contained no JSON-RPC payload".to_string());
    }
    serde_json::from_slice::<Value>(body).map_err(|e| format!("parse MCP JSON-RPC response: {e}"))
}

fn post_http_mcp_jsonrpc(
    agent: &ureq::Agent,
    url: &str,
    session_id: Option<&str>,
    bearer_token: Option<&str>,
    body: &Value,
    expected_id: Option<u64>,
) -> Result<(Value, Option<String>), String> {
    let mut req = agent
        .post(url)
        .set("Content-Type", "application/json")
        .set("Accept", "application/json, text/event-stream");
    if let Some(sid) = session_id {
        req = req.set("Mcp-Session-Id", sid);
    }
    if let Some(tok) = bearer_token.map(str::trim).filter(|s| !s.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {tok}"));
    }
    let resp = req
        .send_json(body.clone())
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => {
                let mut buf = Vec::new();
                let _ = r.into_reader().take(2048).read_to_end(&mut buf);
                format!(
                    "HTTP MCP server returned HTTP {code}: {}",
                    String::from_utf8_lossy(&buf)
                )
            }
            other => tripwire::explain_fetch_error(url, &other),
        })?;
    let next_sid = resp.header("Mcp-Session-Id").map(str::to_string);
    let ct = resp.content_type().to_string();
    let mut raw = Vec::new();
    resp.into_reader()
        .take(512 * 1024)
        .read_to_end(&mut raw)
        .map_err(|e| format!("read HTTP MCP response: {e}"))?;
    if raw.iter().all(u8::is_ascii_whitespace) {
        return Ok((json!({}), next_sid));
    }
    let parsed = parse_http_or_sse_jsonrpc(&ct, &raw, expected_id.unwrap_or(1))?;
    Ok((parsed, next_sid))
}

fn run_http_mcp_session(
    cfg: &Config,
    server: &McpServerSpec,
    followup_method: &str,
    followup_params: Value,
) -> Result<Value, String> {
    tripwire::check_egress(&server.command_or_url, "connector")?;
    let default_port = if server.command_or_url.to_ascii_lowercase().starts_with("https://") {
        443
    } else {
        80
    };
    let (host, port) = connectors::parse_url_host_port_pub(&server.command_or_url, default_port);
    tripwire::check_egress_host_for(&cfg.data_dir, &host, port, "connector")?;

    let bearer_token = if let Some(ref auth_key) = server.auth_secret_key {
        let secrets = connectors::get_connector_secret_map_result(&cfg.data_dir)?;
        let tok = secrets
            .get(auth_key)
            .cloned()
            .ok_or_else(|| format!("missing MCP auth secret '{auth_key}'"))?;
        Some(tok)
    } else {
        None
    };

    let agent = tripwire::guarded_connector_agent_for(&cfg.data_dir, MCP_IO_TIMEOUT);
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "ergon",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }
    });
    let (init_resp, sid) = post_http_mcp_jsonrpc(
        &agent,
        &server.command_or_url,
        None,
        bearer_token.as_deref(),
        &init_req,
        Some(1),
    )?;
    if let Some(err) = init_resp.get("error").filter(|v| !v.is_null()) {
        return Err(format!("MCP initialize error: {err}"));
    }

    let notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    let _ = post_http_mcp_jsonrpc(
        &agent,
        &server.command_or_url,
        sid.as_deref(),
        bearer_token.as_deref(),
        &notif,
        None,
    );

    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": followup_method,
        "params": followup_params,
    });
    let (call_resp, _) = post_http_mcp_jsonrpc(
        &agent,
        &server.command_or_url,
        sid.as_deref(),
        bearer_token.as_deref(),
        &call_req,
        Some(2),
    )?;
    if let Some(err) = call_resp.get("error").filter(|v| !v.is_null()) {
        return Err(format!("MCP {followup_method} error: {err}"));
    }
    Ok(call_resp["result"].clone())
}

fn run_mcp_session(
    cfg: &Config,
    server: &McpServerSpec,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    if server.transport == "stdio" {
        run_stdio_mcp_session(cfg, server, method, params)
    } else if server.transport == "http_sse" {
        run_http_mcp_session(cfg, server, method, params)
    } else {
        Err(format!("unsupported MCP transport '{}'", server.transport))
    }
}

pub fn discover_mcp_server(
    cfg: &Config,
    server_id: &str,
) -> Result<Value, (u16, String)> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    let server = with_mcp_lock(|| {
        McpStore::load_unlocked(&cfg.data_dir)
            .servers
            .into_iter()
            .find(|s| s.id == server_id)
    })
    .ok_or_else(|| (404, format!("MCP server '{server_id}' not found")))?;

    if server.transport == "http_sse"
        && (policy.block_outbound_fetch || policy.profile == "air_gapped")
    {
        return Err((
            403,
            "blocked by Air-Gapped workspace policy: HTTP MCP discovery is disabled".to_string(),
        ));
    }

    match run_mcp_session(cfg, &server, "tools/list", json!({})) {
        Ok(result) => {
            let tools_arr = result["tools"].as_array().cloned().unwrap_or_default();
            let parsed_tools: Vec<McpToolSpec> = tools_arr
                .iter()
                .filter_map(McpToolSpec::from_json)
                .take(MAX_MCP_TOOLS_PER_SERVER)
                .collect();
            let now_ms = records::now_millis();
            let stored_keys: HashSet<String> = connectors::get_connector_secret_map(&cfg.data_dir)
                .into_keys()
                .collect();
            let updated = with_mcp_lock(|| {
                let mut store = McpStore::load_unlocked(&cfg.data_dir);
                let Some(s) = store.servers.iter_mut().find(|s| s.id == server_id) else {
                    return Err((404, format!("MCP server '{server_id}' not found")));
                };
                s.cached_tools = parsed_tools.clone();
                s.last_discovered_ms = Some(now_ms);
                s.last_error = None;
                s.updated_ms = now_ms;
                let out = s.to_public_json_with_secrets(&policy, &stored_keys);
                store.save_unlocked(&cfg.data_dir).map_err(|e| (500, e))?;
                Ok(out)
            })?;
            Ok(json!({
                "ok": true,
                "server": updated,
                "tools": parsed_tools.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            }))
        }
        Err(e) => {
            let safe_err = connectors::redact_for_person(&cfg.data_dir, &e);
            let now_ms = records::now_millis();
            with_mcp_lock(|| {
                let mut store = McpStore::load_unlocked(&cfg.data_dir);
                if let Some(s) = store.servers.iter_mut().find(|s| s.id == server_id) {
                    s.last_error = Some(safe_err.clone());
                    s.updated_ms = now_ms;
                    let _ = store.save_unlocked(&cfg.data_dir);
                }
            });
            let status = if safe_err.contains("egress guard") { 403 } else { 502 };
            Err((status, safe_err))
        }
    }
}

/// Return `(full_tool_name, description, param_keys)` for all enabled MCP servers.
pub fn load_enabled_mcp_tools(person_data_dir: &Path) -> Vec<(String, String, Vec<String>)> {
    with_mcp_lock(|| {
        let store = McpStore::load_unlocked(person_data_dir);
        let mut out = Vec::new();
        for srv in store.servers.iter().filter(|s| s.enabled) {
            for t in &srv.cached_tools {
                let full_name = format!("mcp.{}.{}", srv.id, t.name);
                let param_keys = t.input_schema["properties"]
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default();
                out.push((full_name, t.description.clone(), param_keys));
            }
        }
        out
    })
}

fn format_mcp_call_result(result: &Value) -> Result<String, String> {
    let is_err = result["isError"].as_bool().unwrap_or(false);
    let text = if let Some(arr) = result["content"].as_array() {
        let mut chunks = Vec::new();
        for item in arr {
            if let Some(txt) = item["text"].as_str() {
                chunks.push(txt.to_string());
            } else {
                chunks.push(item.to_string());
            }
        }
        chunks.join("\n")
    } else if !result.is_null() {
        serde_json::to_string_pretty(result).unwrap_or_else(|_| result.to_string())
    } else {
        "(empty MCP result)".to_string()
    };
    if is_err {
        Err(format!("MCP tool returned error: {text}"))
    } else {
        Ok(text)
    }
}

fn clean_mcp_args(args: &Value) -> Value {
    match args {
        Value::Object(map) => {
            let mut clean = serde_json::Map::new();
            for (k, v) in map {
                if k != "_thread" && k != "_defer_decision" {
                    clean.insert(k.clone(), v.clone());
                }
            }
            Value::Object(clean)
        }
        Value::Null => json!({}),
        other => other.clone(),
    }
}

fn execute_mcp_tool_call(
    cfg: &Config,
    server: &McpServerSpec,
    tool_name: &str,
    full_tool_name: &str,
    args: &Value,
) -> Result<String, String> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    if server.transport == "http_sse"
        && (policy.block_outbound_fetch || policy.profile == "air_gapped")
    {
        return Err(
            "blocked by Air-Gapped workspace policy: HTTP MCP calls are disabled for this workspace"
                .to_string(),
        );
    }
    let clean_args = clean_mcp_args(args);
    let raw_res = run_mcp_session(
        cfg,
        server,
        "tools/call",
        json!({
            "name": tool_name,
            "arguments": clean_args,
        }),
    )?;
    let formatted = format_mcp_call_result(&raw_res)?;
    let mut redacted = connectors::redact_for_person(&cfg.data_dir, &formatted);
    if policy.redact_host_paths {
        redacted = crate::redact_host_paths(&redacted);
    }
    let truncated = crate::truncate(&redacted, MAX_MCP_RESPONSE_CHARS);
    Ok(tripwire::spotlight(full_tool_name, &truncated))
}

pub fn invoke_mcp_tool(
    cfg: &Config,
    full_tool_name: &str,
    args: &Value,
    thread_id: &str,
) -> Result<String, String> {
    let rest = full_tool_name
        .strip_prefix("mcp.")
        .ok_or_else(|| format!("unknown tool: {full_tool_name}"))?;
    let (server_id, tool_name) = rest
        .split_once('.')
        .ok_or_else(|| format!("invalid MCP tool name '{full_tool_name}'"))?;
    let server = with_mcp_lock(|| {
        McpStore::load_unlocked(&cfg.data_dir)
            .servers
            .into_iter()
            .find(|s| s.id == server_id)
    })
    .ok_or_else(|| format!("unknown MCP server '{server_id}'"))?;

    if !server.enabled {
        return Err(format!("MCP server '{}' is disabled", server.name));
    }
    if !server.cached_tools.iter().any(|t| t.name == tool_name) {
        return Err(format!(
            "MCP server '{}' has no discovered tool named '{tool_name}'",
            server.name
        ));
    }

    let policy = crate::load_workspace_policy(&cfg.data_dir);
    if server.transport == "http_sse" {
        if policy.block_outbound_fetch || policy.profile == "air_gapped" {
            return Err("blocked by Air-Gapped workspace policy: HTTP MCP calls are disabled".to_string());
        }
        tripwire::check_egress(&server.command_or_url, "connector")?;
        let default_port = if server.command_or_url.to_ascii_lowercase().starts_with("https://") {
            443
        } else {
            80
        };
        let (host, port) = connectors::parse_url_host_port_pub(&server.command_or_url, default_port);
        tripwire::check_egress_host_for(&cfg.data_dir, &host, port, "connector")?;
    }

    let clean_args = clean_mcp_args(args);
    tripwire::check_egress_payload(&clean_args.to_string(), "mcp")?;

    let effective_require_approval =
        server.require_approval_by_default || policy.profile == "strict_client";

    if effective_require_approval {
        let raw_preview = serde_json::to_string(&clean_args).unwrap_or_default();
        let mut redacted = connectors::redact_for_person(&cfg.data_dir, &raw_preview);
        if policy.redact_host_paths {
            redacted = crate::redact_host_paths(&redacted);
        }
        let args_preview = crate::truncate(&redacted, 400);
        let safe_args: Value =
            serde_json::from_str(&redacted).unwrap_or_else(|_| json!(args_preview));
        let summary = format!("Run MCP tool \"{full_tool_name}\" via \"{}\"", server.name);
        let mut detail = if policy.profile == "strict_client" {
            format!(
                "Strict Client policy requires human approval before calling MCP tool '{tool_name}' on \"{}\".",
                server.name
            )
        } else {
            format!(
                "MCP server \"{}\" requires approval before calling tool '{tool_name}'.",
                server.name
            )
        };
        if policy.redact_host_paths {
            detail = crate::redact_host_paths(&detail);
        }
        let decision_id = records::add_decision_with(
            &cfg.data_dir,
            "mcp_tool_call",
            &summary,
            None,
            thread_id,
            json!({
                "detail": detail,
                "severity": "warning",
                "server_id": server.id,
                "server_name": server.name,
                "tool_name": tool_name,
                "mcp_tool": full_tool_name,
                "transport": server.transport,
                "arguments": safe_args,
                "args_preview": args_preview,
                "arguments_preview": args_preview,
            }),
        );
        return Ok(format!(
            "parked for your person's approval (decision {decision_id}): MCP tool '{full_tool_name}' requires approval before running. Do NOT call {full_tool_name} again for this request; report that it is awaiting approval."
        ));
    }

    execute_mcp_tool_call(cfg, &server, tool_name, full_tool_name, &clean_args)
}

fn settle_bus() -> &'static (Mutex<HashSet<String>>, Condvar) {
    IN_FLIGHT_MCP_SETTLE.get_or_init(|| (Mutex::new(HashSet::new()), Condvar::new()))
}

struct McpSettleGuard<'a> {
    key: String,
    bus: &'a (Mutex<HashSet<String>>, Condvar),
}

impl<'a> Drop for McpSettleGuard<'a> {
    fn drop(&mut self) {
        let (mtx, cv) = self.bus;
        let mut set = mtx.lock().unwrap_or_else(|p| p.into_inner());
        set.remove(&self.key);
        cv.notify_all();
    }
}

fn acquire_mcp_settle_guard(data_dir: &Path, decision_id: &str) -> McpSettleGuard<'static> {
    let bus = settle_bus();
    let key = format!("{}::{}", data_dir.display(), decision_id);
    let (mtx, cv) = bus;
    let mut set = mtx.lock().unwrap_or_else(|p| p.into_inner());
    while set.contains(&key) {
        set = cv.wait(set).unwrap_or_else(|p| p.into_inner());
    }
    set.insert(key.clone());
    McpSettleGuard { key, bus }
}

pub fn settle_mcp_decision(cfg: &Config, decision: &Value) -> String {
    let decision_id = decision["id"].as_str().unwrap_or("");
    let _guard = (!decision_id.is_empty())
        .then(|| acquire_mcp_settle_guard(&cfg.data_dir, decision_id));
    let cache_key = format!("{}::{}", cfg.data_dir.display(), decision_id);
    if !decision_id.is_empty() {
        let cache = SETTLED_MCP_OUTCOMES.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(existing) = cache.lock().unwrap_or_else(|p| p.into_inner()).get(&cache_key) {
            return existing.clone();
        }
    }

    let server_id = decision["server_id"].as_str().unwrap_or("");
    let tool_name = decision["tool_name"].as_str().unwrap_or("");
    let full_tool_name = decision["mcp_tool"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| format!("mcp.{server_id}.{tool_name}"));
    let args = &decision["arguments"];

    let server = with_mcp_lock(|| {
        McpStore::load_unlocked(&cfg.data_dir)
            .servers
            .into_iter()
            .find(|s| s.id == server_id)
    });
    let outcome = match server {
        None => format!("Approved, but MCP server '{server_id}' is no longer configured."),
        Some(srv) if !srv.enabled => {
            format!("Approved, but MCP server \"{}\" is disabled.", srv.name)
        }
        Some(srv) => match execute_mcp_tool_call(cfg, &srv, tool_name, &full_tool_name, args) {
            Ok(out) => format!(
                "Approved and executed MCP tool '{full_tool_name}': {}",
                crate::truncate(&out, 400)
            ),
            Err(e) => format!("Approved, but MCP tool call failed: {e}"),
        },
    };

    if !decision_id.is_empty() {
        let cache = SETTLED_MCP_OUTCOMES.get_or_init(|| Mutex::new(HashMap::new()));
        cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(cache_key, outcome.clone());
    }
    outcome
}

fn is_inbound_enabled_unlocked(store: &McpStore) -> bool {
    let env_enabled = std::env::var("ERGON_MCP_INBOUND_ENABLED")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    env_enabled || store.inbound_enabled
}

fn jsonrpc_err(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

/// Handle gated read-only inbound MCP JSON-RPC requests (`POST /api/mcp/inbound`).
pub fn handle_inbound_mcp(
    cfg: &Config,
    root_data_dir: &Path,
    person_id: &str,
    auth_header: Option<&str>,
    body: &[u8],
) -> (u16, Value) {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return (400, jsonrpc_err(Value::Null, -32700, "invalid JSON payload")),
    };
    let req_id = parsed.get("id").cloned().unwrap_or(Value::Null);

    let enabled = with_mcp_lock(|| {
        let store = McpStore::load_unlocked(&cfg.data_dir);
        is_inbound_enabled_unlocked(&store)
    });
    if !enabled {
        return (
            403,
            jsonrpc_err(req_id, -32001, "MCP inbound server is disabled"),
        );
    }

    if let Ok(expected) = std::env::var("ERGON_MCP_INBOUND_TOKEN") {
        let expected = expected.trim();
        if !expected.is_empty() {
            let presented = auth_header
                .and_then(|h| h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")))
                .map(str::trim)
                .unwrap_or("");
            let h1 = tripwire::sha256(presented.as_bytes());
            let h2 = tripwire::sha256(expected.as_bytes());
            if !tripwire::ct_eq(&h1, &h2) {
                return (
                    401,
                    jsonrpc_err(req_id, -32002, "invalid or missing MCP inbound bearer token"),
                );
            }
        }
    }

    let method = parsed["method"].as_str().unwrap_or("");
    let params = &parsed["params"];

    match method {
        "initialize" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "ergon-inbound-mcp",
                        "version": env!("CARGO_PKG_VERSION"),
                    }
                }
            }),
        ),
        "notifications/initialized" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {}
            }),
        ),
        "tools/list" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "tools": [
                        {
                            "name": "search_vault",
                            "description": "Read-only hybrid search across workspace files.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string" },
                                    "path_prefix": { "type": "string" },
                                    "mode": { "type": "string" },
                                    "limit": { "type": "integer" }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "query_table",
                            "description": "Read-only deterministic filtering, grouping, and aggregation over workspace tables.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string" },
                                    "where": { "type": "array" },
                                    "group_by": { "type": "array" },
                                    "aggregates": { "type": "array" },
                                    "sort": { "type": "object" },
                                    "limit": { "type": "integer" }
                                },
                                "required": ["path"]
                            }
                        }
                    ]
                }
            }),
        ),
        "tools/call" => {
            let tool_name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];
            let iso_scope = tools::active_isolation_scope(root_data_dir, &cfg.data_dir, person_id);
            let iso_eng_id = iso_scope.as_ref().map(|(eng, _)| eng.id.as_str());
            let iso_prefix = iso_scope.as_ref().map(|(_, p)| p.as_str());
            let policy = crate::load_workspace_policy(&cfg.data_dir);

            let res: Result<String, String> = match tool_name {
                "search_vault" => {
                    let query = args["query"]
                        .as_str()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| "search_vault requires non-empty arguments.query".to_string());
                    query.and_then(|q| {
                        let req_prefix = args["path_prefix"]
                            .as_str()
                            .map(str::trim)
                            .filter(|s| !s.is_empty());
                        let clean_prefix = if let Some(pfx) = iso_prefix {
                            if let Some(rp) = req_prefix {
                                let clean = tables::validate_workspace_rel_path(rp.trim_end_matches('/'))?;
                                tools::check_engagement_path_allowed(&cfg.workspace_dir, pfx, &clean, false)?;
                                clean
                            } else {
                                pfx.to_string()
                            }
                        } else if let Some(rp) = req_prefix {
                            tables::validate_workspace_rel_path(rp.trim_end_matches('/'))?
                        } else {
                            String::new()
                        };
                        let mode = args["mode"].as_str().unwrap_or("hybrid");
                        let limit = args["limit"].as_u64().unwrap_or(10).clamp(1, 50) as usize;
                        let key = cfg.data_dir.to_string_lossy().into_owned();
                        let idx = search::SearchCache::global().get(
                            &key,
                            &cfg.data_dir,
                            &cfg.workspace_dir,
                            &cfg.memory_path,
                        );
                        let hits = idx.search_vault_chunks_scoped(
                            q,
                            &clean_prefix,
                            mode,
                            limit,
                            None,
                            iso_eng_id,
                        );
                        let hits_json: Vec<Value> = hits
                            .iter()
                            .map(|h| {
                                json!({
                                    "path": h.path,
                                    "line_start": h.line_start,
                                    "line_end": h.line_end,
                                    "page": h.page,
                                    "sheet": h.sheet,
                                    "heading": h.heading,
                                    "score": h.score,
                                    "excerpt": h.excerpt,
                                })
                            })
                            .collect();
                        serde_json::to_string_pretty(&json!({
                            "query": q,
                            "count": hits_json.len(),
                            "hits": hits_json,
                        }))
                        .map_err(|e| format!("serialize search: {e}"))
                    })
                }
                "query_table" => {
                    let raw_path = args["path"]
                        .as_str()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| "query_table requires arguments.path".to_string());
                    raw_path.and_then(|raw| {
                        let (file_part, _) = docs::split_sheet(raw);
                        let clean_file = tables::validate_workspace_rel_path(file_part)?;
                        if let Some(pfx) = iso_prefix {
                            tools::check_engagement_path_allowed(&cfg.workspace_dir, pfx, &clean_file, false)?;
                        }
                        let unresolved = sandbox_path_resolving(&cfg.workspace_dir, &clean_file, false)?;
                        if unresolved
                            .symlink_metadata()
                            .is_ok_and(|md| md.file_type().is_symlink())
                        {
                            return Err(format!("query_table: cannot read symlink '{clean_file}'"));
                        }
                        let _ = sandbox_path(&cfg.workspace_dir, &clean_file)?;
                        let mut ro_args = args.clone();
                        if let Some(obj) = ro_args.as_object_mut() {
                            obj.remove("save_as");
                        }
                        let res = tables::execute_query(cfg, &ro_args)?;
                        serde_json::to_string_pretty(&json!({
                            "matched_rows": res.matched_rows,
                            "total_rows": res.total_rows,
                            "columns": res.columns,
                            "rows": res.rows,
                        }))
                        .map_err(|e| format!("serialize table query: {e}"))
                    })
                }
                _ => {
                    return (
                        200,
                        jsonrpc_err(
                            req_id,
                            -32601,
                            &format!("tool '{tool_name}' is not permitted on read-only inbound MCP server"),
                        ),
                    );
                }
            };

            match res {
                Ok(text) => {
                    let mut safe_text = connectors::redact_for_person(&cfg.data_dir, &text);
                    if policy.redact_host_paths {
                        safe_text = crate::redact_host_paths(&safe_text);
                    }
                    (
                        200,
                        json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "content": [{ "type": "text", "text": safe_text }],
                                "isError": false,
                            }
                        }),
                    )
                }
                Err(e) => (
                    200,
                    json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "result": {
                            "content": [{ "type": "text", "text": e }],
                            "isError": true,
                        }
                    }),
                ),
            }
        }
        _ => (
            200,
            jsonrpc_err(req_id, -32601, &format!("unsupported MCP method '{method}'")),
        ),
    }
}
