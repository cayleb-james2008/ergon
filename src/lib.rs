//! Ergon — an AI agent for every employee, personal to each person, that
//! learns how they work and finishes their digital work autonomously.
//!
//! Architecture: a single Rust binary serving an embedded web UI over a
//! hand-rolled HTTP server (std TcpListener). The agent loop talks to an
//! OpenAI-compatible chat-completions endpoint (OpenCode Go by default)
//! using a strict JSON tool protocol, executes tools for real against a
//! sandboxed workspace folder, and uses Rune scripts (see `skills`) for
//! the document-writing and memory-update skills.

pub mod audit_bundle;
pub mod docs;
pub mod file_history;
pub mod library;
pub mod receipts;
pub mod records;
pub mod routines;
pub mod scheduler;
pub mod search;
pub mod server;
pub use server::{
    allowed_hosts_from, parked_note, serve, serve_with_gate, serve_with_hosts, serve_with_updates,
    AppState, LiveModelConfig, PersonRuntime,
};
pub mod skill_catalog;
pub mod skills;
pub mod sse;
pub mod tools;
pub mod tripwire;
pub use tools::{
    check_mentions, execute_tool, execute_tool_full, export_workspace_file, extract_written,
    fetch_url_text, html_to_text, mention_candidates, page_text, sandbox_path,
    sandbox_path_resolving, truncate, verify_text_citations, verify_text_citations_against,
    CitationCheck, CitationReport, ToolCtx,
};
pub mod ui;
pub mod update;

use crate::tools::{atomic_write_bytes, atomic_write_bytes_with_mode};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const BROWSER_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
pub const NO_KEY_ERROR: &str =
    "no model key configured: set ERGON_MODEL_API_KEY so the agent can reach its model. Ergon will not fabricate an answer without one.";
pub const STEP_CAP: usize = 24;

// ---------------------------------------------------------------------------
// Workspace Audit Policy Profiles (v0.25.0)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspacePolicy {
    pub profile: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub strict_citation_gate: bool,
    pub approval_gate_mode: &'static str,
    pub redact_host_paths: bool,
    pub block_outbound_fetch: bool,
    pub updated_ms: u64,
}

impl WorkspacePolicy {
    pub fn for_profile(raw: &str, updated_ms: u64) -> Result<Self, &'static str> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "fast_internal" | "fast" | "internal" => Ok(Self {
                profile: "fast_internal",
                label: "Fast Internal",
                summary: "Advisory citation checks, local-speed approvals, and outbound web research enabled.",
                strict_citation_gate: false,
                approval_gate_mode: "deletes_and_risky",
                redact_host_paths: false,
                block_outbound_fetch: false,
                updated_ms,
            }),
            "strict_client" | "strict" | "client" => Ok(Self {
                profile: "strict_client",
                label: "Strict Client",
                summary: "Blocks pack export on stale [file:L##] citations, redacts host paths in handoffs, and enforces approval gates.",
                strict_citation_gate: true,
                approval_gate_mode: "all_external_and_deletes",
                redact_host_paths: true,
                block_outbound_fetch: false,
                updated_ms,
            }),
            "air_gapped" | "airgapped" | "offline" => Ok(Self {
                profile: "air_gapped",
                label: "Air-Gapped",
                summary: "Zero outbound network fetches, strict citation gate before export, and full host-path redaction.",
                strict_citation_gate: true,
                approval_gate_mode: "lockdown",
                redact_host_paths: true,
                block_outbound_fetch: true,
                updated_ms,
            }),
            _ => Err("workspace policy profile must be fast_internal, strict_client, or air_gapped"),
        }
    }

    pub fn to_json(&self, person_id: &str) -> Value {
        json!({
            "person_id": person_id,
            "profile": self.profile,
            "label": self.label,
            "summary": self.summary,
            "strict_citation_gate": self.strict_citation_gate,
            "approval_gate_mode": self.approval_gate_mode,
            "redact_host_paths": self.redact_host_paths,
            "block_outbound_fetch": self.block_outbound_fetch,
            "updated_ms": self.updated_ms,
        })
    }
}

pub fn load_workspace_policy(data_dir: &Path) -> WorkspacePolicy {
    let path = data_dir.join("workspace_policy.json");
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(p) = v["profile"].as_str() {
                let updated = v["updated_ms"].as_u64().unwrap_or(0);
                if let Ok(policy) = WorkspacePolicy::for_profile(p, updated) {
                    return policy;
                }
            }
        }
    }
    WorkspacePolicy::for_profile("fast_internal", 0).unwrap()
}

pub fn save_workspace_policy(data_dir: &Path, profile: &str) -> Result<WorkspacePolicy, String> {
    let ts = records::now_millis();
    let policy = WorkspacePolicy::for_profile(profile, ts).map_err(|e| e.to_string())?;
    let path = data_dir.join("workspace_policy.json");
    let body = serde_json::to_vec(&json!({
        "profile": policy.profile,
        "updated_ms": policy.updated_ms,
    }))
    .map_err(|e| format!("serialize policy: {e}"))?;
    atomic_write_bytes(&path, &body).map_err(|e| format!("save workspace policy: {e}"))?;
    Ok(policy)
}

/// Redact host filesystem prefixes (`/home/<user>`, `/Users/<user>`, `C:\Users\<user>`, `/tmp/`)
/// for client-safe handoff exports while preserving relative workspace file paths.
pub fn redact_host_paths(input: &str) -> String {
    if !input.contains("/home/")
        && !input.contains("/Users/")
        && !input.contains("C:\\Users\\")
        && !input.contains("C:/Users/")
        && !input.contains("/tmp/")
    {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        let rest = &input[i..];
        let matched_prefix = if rest.starts_with("/home/") {
            Some(("/home/", "~/"))
        } else if rest.starts_with("/Users/") {
            Some(("/Users/", "~/"))
        } else if rest.starts_with("C:/Users/") {
            Some(("C:/Users/", "~/"))
        } else if rest.starts_with("C:\\\\Users\\\\") {
            Some(("C:\\\\Users\\\\", "~/"))
        } else if rest.starts_with("C:\\Users\\") {
            Some(("C:\\Users\\", "~/"))
        } else if rest.starts_with("/tmp/") {
            out.push_str("[redacted-tmp]/");
            i += 5;
            continue;
        } else {
            None
        };
        if let Some((pfx, repl)) = matched_prefix {
            out.push_str(repl);
            i += pfx.len();
            // Skip the username segment up to separator or whitespace/quote.
            while i < bytes.len()
                && !matches!(bytes[i], b'/' | b'\\' | b' ' | b'\t' | b'\r' | b'\n' | b'"' | b'\'' | b'`' | b')' | b']' | b'>')
            {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'/' || bytes[i] == b'\\') {
                if input[i..].starts_with("\\\\") {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

pub const DEFAULT_MODEL_BASE_URL: &str = "https://opencode.ai/zen/go/v1";
pub const DEFAULT_MODEL_ID: &str = "deepseek-v4.1-flash";

#[derive(Clone, Debug)]
pub struct DetectedGateway {
    pub base_url: String,
    pub model_id: String,
    pub label: String,
    pub models: Vec<String>,
}

impl DetectedGateway {
    pub fn to_json(&self) -> Value {
        json!({
            "base_url": self.base_url,
            "model_id": self.model_id,
            "label": self.label,
            "models": self.models,
        })
    }
}

#[inline]
pub fn model_configured_for(model_base_url: &str, has_api_key: bool) -> bool {
    has_api_key
        || parse_url_host_port(model_base_url)
            .map(|(h, port)| matches!(h.as_str(), "127.0.0.1" | "localhost" | "[::1]") && port != 9)
            .unwrap_or(false)
}

/// Extract (host, port) from an http(s) URL authority.
fn parse_url_host_port(url: &str) -> Option<(String, u16)> {
    let (scheme, rest) = url.trim().split_once("://")?;
    let default_port = match scheme.to_ascii_lowercase().as_str() {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let authority = rest.split('/').next().unwrap_or("").trim();
    if authority.is_empty() {
        return None;
    }
    if let Some(after_open) = authority.strip_prefix('[') {
        let (host, tail) = after_open.split_once(']')?;
        let port = if let Some(p) = tail.strip_prefix(':') {
            p.parse::<u16>().ok()?
        } else {
            default_port
        };
        return Some((format!("[{}]", host.to_ascii_lowercase()), port));
    }
    if let Some((host, p)) = authority.rsplit_once(':') {
        let port = p.parse::<u16>().ok()?;
        Some((host.to_ascii_lowercase(), port))
    } else {
        Some((authority.to_ascii_lowercase(), default_port))
    }
}

/// Whether `url` targets loopback (`127.0.0.1`, `localhost`, or `[::1]`).
pub fn is_loopback_url(url: &str) -> bool {
    parse_url_host_port(url)
        .map(|(h, _)| matches!(h.as_str(), "127.0.0.1" | "localhost" | "[::1]"))
        .unwrap_or(false)
}

/// Fast non-blocking TCP reachability probe for loopback model URLs (250ms).
/// Ensures dead discard ports (e.g. `127.0.0.1:9` in unit/no-key tests) fail
/// closed while running local OpenAI-compatible gateways work without an API key.
pub fn loopback_tcp_reachable(url: &str) -> bool {
    let Some((host, port)) = parse_url_host_port(url) else {
        return false;
    };
    if !matches!(host.as_str(), "127.0.0.1" | "localhost" | "[::1]") {
        return false;
    }
    let ip: std::net::IpAddr = match host.as_str() {
        "[::1]" => std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        _ => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
    };
    let addr = std::net::SocketAddr::new(ip, port);
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(250)).is_ok()
}

/// Probe known running local OpenAI-compatible gateways in priority order.
/// Ollama (`11434`) is checked first because it is a pure stateless OpenAI JSON
/// endpoint that executes Ergon's sandboxed tool protocol directly without
/// running host-level CLI tools outside the workspace.
pub fn detect_local_gateways() -> Vec<DetectedGateway> {
    const CANDIDATES: [(&str, &str, &str); 3] = [
        (
            "http://127.0.0.1:11434/v1",
            "qwen2.5:3b",
            "Local Ollama (:11434)",
        ),
        (
            "http://127.0.0.1:8790/v1",
            "deepseek-v4.1-flash-free",
            "Local OpenCode Zen (:8790)",
        ),
        (
            "http://127.0.0.1:8789/v1",
            "gemini-3.8-flash-high",
            "Local Antigravity Bridge (:8789)",
        ),
    ];
    let mut found = Vec::new();
    for (base_url, preferred_model, label) in CANDIDATES {
        if !loopback_tcp_reachable(base_url) {
            continue;
        }
        let models_url = format!("{base_url}/models");
        let resp = ureq::get(&models_url)
            .timeout(std::time::Duration::from_millis(1200))
            .call();
        let Ok(r) = resp else { continue };
        let Ok(v) = r.into_json::<Value>() else { continue };
        let mut models: Vec<String> = v["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        if models.is_empty() {
            models.push(preferred_model.to_string());
        }
        let chosen = if models.iter().any(|m| m == preferred_model) {
            preferred_model.to_string()
        } else {
            models[0].clone()
        };
        found.push(DetectedGateway {
            base_url: base_url.to_string(),
            model_id: chosen,
            label: label.to_string(),
            models,
        });
    }
    found
}

#[derive(Clone, Debug)]
pub struct SavedModelConfig {
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
}

pub fn config_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

pub fn load_saved_model_config(data_dir: &Path) -> Option<SavedModelConfig> {
    let raw = std::fs::read_to_string(config_file_path(data_dir)).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let base = v["model_base_url"]
        .as_str()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())?;
    let model = v["model_id"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string());
    let key = v["api_key"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(SavedModelConfig {
        model_base_url: base,
        model_id: model,
        api_key: key,
    })
}

pub fn save_model_config(data_dir: &Path, saved: &SavedModelConfig) -> Result<(), String> {
    let path = config_file_path(data_dir);
    let body = json!({
        "model_base_url": saved.model_base_url,
        "model_id": saved.model_id,
        "api_key": saved.api_key,
    })
    .to_string();
    atomic_write_bytes_with_mode(&path, body.as_bytes(), Some(0o600))
        .map_err(|e| format!("save config.json: {e}"))
}

#[derive(Clone, Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub memory_path: PathBuf,
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Config {
        let data_dir = std::env::var("ERGON_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data"));
        let workspace_dir = std::env::var("ERGON_WORKSPACE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("workspace"));
        let memory_path = data_dir.join("memory.md");

        let env_base = std::env::var("ERGON_MODEL_BASE_URL")
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty());
        let env_model = std::env::var("ERGON_MODEL_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let env_key = std::env::var("ERGON_MODEL_API_KEY")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        let no_auto = std::env::var("ERGON_NO_AUTO_MODEL")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let saved = load_saved_model_config(&data_dir);

        let (model_base_url, model_id, api_key) =
            if env_base.is_some() || env_key.is_some() || saved.is_some() {
                let base = env_base
                    .or_else(|| saved.as_ref().map(|s| s.model_base_url.clone()))
                    .unwrap_or_else(|| DEFAULT_MODEL_BASE_URL.to_string());
                let model = env_model
                    .or_else(|| saved.as_ref().map(|s| s.model_id.clone()))
                    .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string());
                let key = env_key.or_else(|| saved.and_then(|s| s.api_key));
                (base, model, key)
            } else if !no_auto {
                let gateways = detect_local_gateways();
                if let Some(gw) = gateways.first() {
                    let model = env_model.unwrap_or_else(|| gw.model_id.clone());
                    (gw.base_url.clone(), model, None)
                } else {
                    (
                        DEFAULT_MODEL_BASE_URL.to_string(),
                        env_model.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
                        None,
                    )
                }
            } else {
                (
                    DEFAULT_MODEL_BASE_URL.to_string(),
                    env_model.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
                    None,
                )
            };

        Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url,
            model_id,
            api_key,
        }
    }

    /// True when the agent can reach a configured model: either an API key is
    /// present, or the base URL targets a real loopback OpenAI-compatible port
    /// (excluding RFC 863 discard port 9 used by offline/no-key tests).
    #[inline]
    pub fn model_configured(&self) -> bool {
        model_configured_for(&self.model_base_url, self.api_key.is_some())
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.workspace_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        if !self.memory_path.exists() {
            std::fs::write(&self.memory_path, "# Memory\n\n")?;
        }
        Ok(())
    }

    pub fn read_memory(&self) -> String {
        std::fs::read_to_string(&self.memory_path).unwrap_or_else(|_| "# Memory\n\n".to_string())
    }
}

// ---------------------------------------------------------------------------
// People — one agent per employee. Each person has their own memory file,
// workspace and thread store. The "default" person uses the root config's
// own workspace/memory (the original single-person layout); every other
// person lives under data_dir/people/<id>/.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub role: String,
}

impl Person {
    pub fn to_json(&self) -> Value {
        json!({"id": self.id, "name": self.name, "role": self.role})
    }
    pub fn from_json(v: &Value) -> Option<Person> {
        Some(Person {
            id: v["id"].as_str()?.to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            role: v["role"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// A person id is safe to use as a directory name: lowercase slug
/// characters only, bounded length. Ids minted by `slugify` always
/// qualify; ids loaded from disk must prove it.
pub fn valid_person_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true; // trims leading dashes
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "person".to_string()
    } else {
        out
    }
}

#[derive(Clone, Debug)]
pub struct PeopleState {
    pub people: Vec<Person>,
    pub active: String,
    /// Store files backed up at load (named in GET /api/status).
    pub recovered: Vec<String>,
}

impl Default for PeopleState {
    fn default() -> Self {
        PeopleState {
            people: vec![Person {
                id: "default".to_string(),
                name: "You".to_string(),
                role: String::new(),
            }],
            active: "default".to_string(),
            recovered: Vec::new(),
        }
    }
}

/// Move a corrupt state file aside to the first free backup name
/// (`<file>.bak`, then `.bak2`, `.bak3`, …) so its bytes survive for
/// inspection instead of being overwritten by the next save — or by
/// the previous backup.
pub fn backup_corrupt(path: &Path) -> Option<PathBuf> {
    let mut candidate = path.with_extension("json.bak");
    let mut n = 2;
    while candidate.exists() {
        candidate = path.with_extension(format!("json.bak{n}"));
        n += 1;
    }
    std::fs::rename(path, &candidate).ok()?;
    eprintln!(
        "ergon: {} was corrupt; backed up to {} and starting fresh",
        path.display(),
        candidate.display()
    );
    Some(candidate)
}

/// Backs up a corrupt store and returns its file name, for the
/// "earlier conversations were moved" notice (GET /api/status).
pub fn backed_up_name(path: &Path) -> Option<String> {
    backup_corrupt(path)?;
    path.file_name().map(|n| n.to_string_lossy().into_owned())
}

impl PeopleState {
    pub fn load(root: &Config) -> PeopleState {
        let path = root.data_dir.join("people.json");
        if let Ok(text) = std::fs::read_to_string(&path) {
            let parsed = serde_json::from_str::<Value>(&text);
            // Valid means exactly the shape this app writes: a JSON
            // object whose "people" is an array. Anything else —
            // unparseable, a bare array/number, a missing or mistyped
            // key — is corruption: keep the bytes as a backup rather
            // than letting the next save destroy them silently. (An
            // earlier check treated a MISSING key as fine, which let
            // `[]`, `{}` and `42` be silently overwritten.)
            let shape_ok = parsed
                .as_ref()
                .map(|v| {
                    v.is_object()
                        && v.get("people").map(|p| p.is_array()).unwrap_or(false)
                })
                .unwrap_or(false);
            if !shape_ok {
                return PeopleState {
                    recovered: backed_up_name(&path).into_iter().collect(),
                    ..PeopleState::default()
                };
            }
            if let Ok(v) = parsed {
                let mut state = PeopleState::default();
                if let Some(items) = v["people"].as_array() {
                    // Ids from disk are validated, not trusted: an id is
                    // a directory name under data_dir, so a crafted
                    // people.json must not be able to aim a person's
                    // workspace outside it (e.g. id "../../elsewhere").
                    let parsed: Vec<Person> = items
                        .iter()
                        .filter_map(Person::from_json)
                        .filter(|p| valid_person_id(&p.id))
                        .collect();
                    if !items.is_empty() && parsed.is_empty() {
                        // Every entry was unusable — that is corruption
                        // wearing a valid shape; back it up like any
                        // other corrupt file instead of overwriting it.
                        return PeopleState {
                            recovered: backed_up_name(&path).into_iter().collect(),
                            ..PeopleState::default()
                        };
                    }
                    if !parsed.is_empty() {
                        state.people = parsed;
                    }
                }
                if !state.people.iter().any(|p| p.id == "default") {
                    state.people.insert(
                        0,
                        Person {
                            id: "default".to_string(),
                            name: "You".to_string(),
                            role: String::new(),
                        },
                    );
                }
                if let Some(active) = v["active"].as_str() {
                    if state.people.iter().any(|p| p.id == active) {
                        state.active = active.to_string();
                    }
                }
                return state;
            }
        }
        PeopleState::default()
    }

    pub fn save(&self, root: &Config) {
        let _ = std::fs::create_dir_all(&root.data_dir);
        let v = json!({
            "people": self.people.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "active": self.active,
        });
        let path = root.data_dir.join("people.json");
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, v.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    pub fn active_person(&self) -> Person {
        self.people
            .iter()
            .find(|p| p.id == self.active)
            .cloned()
            .unwrap_or_else(|| self.people[0].clone())
    }

    pub fn add(&mut self, name: &str, role: &str) -> Person {
        // Names and roles are interpolated into the seeded memory file,
        // so they are flattened to single lines and length-capped here:
        // a name carrying newlines could otherwise forge sections and
        // bullets in memory.md (structure injection).
        let clean = |s: &str, cap: usize| -> String {
            let flat: String = s
                .replace(['\n', '\r'], " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            flat.chars().take(cap).collect()
        };
        let name = clean(name, 80);
        let role = clean(role, 120);
        let base = slugify(&name);
        let mut id = base.clone();
        let mut n = 2;
        while self.people.iter().any(|p| p.id == id) {
            id = format!("{base}-{n}");
            n += 1;
        }
        let person = Person {
            id,
            name,
            role,
        };
        self.people.push(person.clone());
        person
    }

    pub fn to_json(&self) -> Value {
        json!({
            "people": self.people.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "active": self.active,
        })
    }
}

impl Config {
    /// The config scoped to one person: their own data dir (threads.json
    /// lives there), workspace and memory file. The default person keeps
    /// the root layout.
    pub fn person_config(&self, person: &Person) -> Config {
        if person.id == "default" {
            return self.clone();
        }
        let dir = self.data_dir.join("people").join(&person.id);
        Config {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            memory_path: dir.join("memory.md"),
            model_base_url: self.model_base_url.clone(),
            model_id: self.model_id.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Thread store
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant" | "step" | "error"
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

pub fn thread_to_json(t: &Thread) -> Value {
    json!({
        "id": t.id,
        "title": t.title,
        "messages": t.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
    })
}

pub fn thread_from_json(v: &Value) -> Option<Thread> {
    let id = v["id"].as_str()?.to_string();
    let title = v["title"].as_str().unwrap_or("New chat").to_string();
    let mut messages = Vec::new();
    if let Some(items) = v["messages"].as_array() {
        for m in items {
            if let (Some(role), Some(content)) = (m["role"].as_str(), m["content"].as_str()) {
                messages.push(ChatMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                });
            }
        }
    }
    Some(Thread { id, title, messages })
}

#[derive(Default)]
pub struct Store {
    pub threads: Vec<Thread>,
    /// The backed-up file name when this store started fresh (E5).
    pub recovered: Option<String>,
    /// Where this store persists (threads.json); set by `Store::load`.
    pub save_path: Option<PathBuf>,
}

impl Store {
    /// Load a store from its threads.json (missing/corrupt file = empty
    /// store). The store saves itself back to the same path on every
    /// mutation, atomically (tmp file + rename).
    pub fn load(path: &Path) -> Store {
        let mut store = Store {
            threads: Vec::new(),
            save_path: Some(path.to_path_buf()),
            recovered: None,
        };
        if let Ok(text) = std::fs::read_to_string(path) {
            match serde_json::from_str::<Value>(&text) {
                Ok(v) => match v["threads"].as_array() {
                    Some(items) => {
                        for item in items {
                            if let Some(t) = thread_from_json(item) {
                                store.threads.push(t);
                            }
                        }
                    }
                    // Valid JSON, wrong shape (e.g. {"threads": 42}):
                    // just as corrupt as a syntax error — the next
                    // save must not destroy these bytes unrecorded.
                    None => {
                        store.recovered = backed_up_name(path);
                    }
                },
                Err(_) => {
                    store.recovered = backed_up_name(path);
                }
            }
        }
        store
    }

    pub fn save(&self) {
        let Some(path) = &self.save_path else { return };
        let v = json!({
            "threads": self.threads.iter().map(thread_to_json).collect::<Vec<_>>(),
        });
        let _ = atomic_write_bytes(path, v.to_string().as_bytes());
    }
}

pub type SharedStore = Arc<Mutex<Store>>;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn new_thread_id() -> String {
    let n = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("t{millis}-{n}")
}

impl Store {
    pub fn create_thread(&mut self) -> String {
        let id = new_thread_id();
        self.threads.push(Thread {
            id: id.clone(),
            title: "New chat".to_string(),
            messages: Vec::new(),
        });
        self.save();
        id
    }

    pub fn ensure_thread(&mut self, id: &str) -> String {
        if id.is_empty() || !self.threads.iter().any(|t| t.id == id) {
            return self.create_thread();
        }
        id.to_string()
    }

    /// Rewrites the first step message in a thread whose text is exactly
    /// `old` (a parked-delete note) to `new`, and persists it (ER1).
    pub fn settle_note(&mut self, thread_id: &str, old: &str, new: &str) -> bool {
        let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) else {
            return false;
        };
        let Some(m) = t
            .messages
            .iter_mut()
            .find(|m| m.role == "step" && m.content == old)
        else {
            return false;
        };
        m.content = new.to_string();
        self.save();
        true
    }

    pub fn push(&mut self, thread_id: &str, role: &str, content: &str) {
        let mut changed = false;
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) {
            if role == "user" && t.messages.is_empty() {
                t.title = truncate(&content.chars().take(60).collect::<String>(), 60);
            }
            t.messages.push(ChatMessage {
                role: role.to_string(),
                content: content.to_string(),
            });
            changed = true;
        }
        if changed {
            self.save();
        }
    }

    pub fn thread_json(&self, id: &str) -> Option<Value> {
        self.threads.iter().find(|t| t.id == id).map(thread_to_json)
    }

    /// The conversation so far, as model messages: the last 12
    /// user/assistant turns (step and error lines are the ledger, not
    /// conversation), each truncated. Without this the agent saw only
    /// the current task — a thread was a log, not a conversation, and
    /// "now change it" had nothing to refer to.
    pub fn history_json(&self, thread_id: &str) -> Vec<Value> {
        let Some(t) = self.threads.iter().find(|t| t.id == thread_id) else {
            return Vec::new();
        };
        let turns: Vec<&ChatMessage> = t
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .collect();
        turns
            .iter()
            .skip(turns.len().saturating_sub(12))
            .map(|m| {
                json!({
                    "role": if m.role == "user" { "user" } else { "assistant" },
                    "content": truncate(&m.content, 1500),
                })
            })
            .collect()
    }

    pub fn list_json(&self) -> Value {
        json!({
            "threads": self.threads.iter().rev().map(|t| {
                let has_parked = t.messages.iter().any(|m| {
                    m.role == "step" && (m.content.contains("parked for your person") || m.content.contains("has NOT been deleted yet"))
                });
                let has_resolved = t.messages.iter().any(|m| {
                    m.role == "step" && (m.content.contains("was deleted after you approved it") || m.content.contains("was kept because you declined"))
                });
                let state = if has_parked && !has_resolved {
                    "waiting"
                } else if t.messages.iter().any(|m| m.role == "assistant") {
                    "finished"
                } else {
                    "none"
                };
                json!({"id": t.id, "title": t.title, "messages": t.messages.len(), "state": state})
            }).collect::<Vec<_>>(),
        })
    }
}

// ---------------------------------------------------------------------------
// Model client (OpenCode Go / OpenAI-compatible chat completions)
// ---------------------------------------------------------------------------

/// One model call. Carries the proven wire requirements: bearer key (when set
/// or required for non-loopback endpoints), `x-opencode-session` header, and a
/// browser-like User-Agent. Loopback gateways work without a bearer key.
/// HTTP statuses worth another try: timeouts, rate limits, overloads and
/// gateway hiccups. Anything else (400s, auth) fails at once.
pub fn retryable_status(code: u16) -> bool {
    matches!(code, 408 | 425 | 429 | 500 | 502 | 503 | 504 | 520..=529)
}

/// How long to wait before attempt `attempt + 1` (1-based attempts):
/// the server's `Retry-After` seconds when given (capped), else 1.5 s, 4 s, 9 s.
pub fn retry_delay(attempt: u32, retry_after: Option<&str>) -> std::time::Duration {
    if let Some(secs) = retry_after.and_then(|v| v.trim().parse::<f64>().ok()).filter(|s| s.is_finite() && *s >= 0.0) {
        return std::time::Duration::from_millis((secs.min(20.0) * 1000.0) as u64);
    }
    std::time::Duration::from_millis(match attempt {
        1 => 1_500,
        2 => 4_000,
        _ => 9_000,
    })
}

const MODEL_ATTEMPTS: u32 = 4;
const MODEL_RETRY_BUDGET: std::time::Duration = std::time::Duration::from_secs(150);

pub fn call_model(cfg: &Config, session: &str, messages: &[Value]) -> Result<Value, String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let url = format!("{}/chat/completions", cfg.model_base_url);
    let mut payload = json!({
        "model": cfg.model_id,
        "messages": messages,
        "temperature": 0.2,
    });
    if cfg.model_base_url.contains(":11434") {
        payload["response_format"] = json!({"type": "json_object"});
    }
    // A rate limit, an overloaded provider or a dropped connection costs a
    // short wait, not the whole task: transient failures are retried with
    // backoff (honouring Retry-After) inside a fixed time budget. Dropped
    // connections are retried for remote endpoints only — a local gateway
    // that is not running should say so at once.
    let started = std::time::Instant::now();
    let remote = !is_loopback_url(&cfg.model_base_url);
    let mut attempt: u32 = 0;
    let resp = loop {
        attempt += 1;
        let mut req = ureq::post(&url)
            .set("x-opencode-session", session)
            .set("User-Agent", BROWSER_UA)
            .timeout(std::time::Duration::from_secs(120));
        if let Some(key) = cfg.api_key.as_deref().filter(|k| !k.is_empty()) {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
        let can_retry = attempt < MODEL_ATTEMPTS && started.elapsed() < MODEL_RETRY_BUDGET;
        match req.send_json(payload.clone()) {
            Ok(r) => break r,
            Err(ureq::Error::Status(code, r)) if can_retry && retryable_status(code) => {
                let wait = retry_delay(attempt, r.header("retry-after"));
                eprintln!("ergon: model endpoint returned HTTP {code}; retrying in {} ms (attempt {}/{MODEL_ATTEMPTS})", wait.as_millis(), attempt + 1);
                std::thread::sleep(wait);
            }
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                let tries = if attempt > 1 { format!(" (after {attempt} attempts)") } else { String::new() };
                return Err(format!("model endpoint returned HTTP {code}{tries}: {}", truncate(&body, 400)));
            }
            Err(ureq::Error::Transport(t))
                if can_retry && remote && matches!(t.kind(), ureq::ErrorKind::Io | ureq::ErrorKind::ConnectionFailed) =>
            {
                let wait = retry_delay(attempt, None);
                eprintln!("ergon: model call failed ({t}); retrying in {} ms (attempt {}/{MODEL_ATTEMPTS})", wait.as_millis(), attempt + 1);
                std::thread::sleep(wait);
            }
            Err(e) => return Err(format!("model call failed: {e}")),
        }
    };
    let v: Value = resp
        .into_json()
        .map_err(|e| format!("model response was not JSON: {e}"))?;
    let content = &v["choices"][0]["message"]["content"];
    match content {
        Value::String(s) => parse_model_reply(s),
        Value::Object(_) => Ok(normalize_model_reply(content.clone())),
        _ => Err(format!(
            "model response had no message content: {}",
            truncate(&v.to_string(), 400)
        )),
    }
}

fn normalize_model_reply(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        // Common local-model / OpenAI-function-call aliases:
        // {"name": "write_file", "arguments": {...}} -> {"tool": "write_file", "args": {...}}
        if !obj.contains_key("tool") {
            if let Some(name) = obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()) {
                obj.remove("name");
                obj.insert("tool".to_string(), Value::String(name));
            }
        }
        if obj.contains_key("tool") && !obj.contains_key("args") {
            if let Some(arguments) = obj.remove("arguments") {
                obj.insert("args".to_string(), arguments);
            }
        }
        if let Some(Value::String(s)) = obj.get("args").cloned() {
            if let Ok(parsed) = serde_json::from_str::<Value>(&s) {
                if parsed.is_object() {
                    obj.insert("args".to_string(), parsed);
                }
            }
        }
        if obj.contains_key("tool") && !obj.get("args").is_some_and(|a| a.is_object()) {
            let mut extra = serde_json::Map::new();
            let keys: Vec<String> = obj
                .keys()
                .filter(|k| k.as_str() != "tool" && k.as_str() != "args")
                .cloned()
                .collect();
            for k in keys {
                if let Some(val) = obj.remove(&k) {
                    extra.insert(k, val);
                }
            }
            obj.insert("args".to_string(), Value::Object(extra));
        }
        if obj.get("done").and_then(|d| d.as_str()) == Some("true") {
            obj.insert("done".to_string(), Value::Bool(true));
        }
        if !obj.contains_key("tool")
            && !obj.contains_key("done")
            && !obj.contains_key("path")
            && obj.get("summary").and_then(|s| s.as_str()).is_some_and(|s| !s.trim().is_empty())
        {
            obj.insert("done".to_string(), Value::Bool(true));
        }
    }
    v
}

/// Extract the first balanced JSON object from a text blob that may include prose
/// or code fences around it.
fn extract_first_json_object(s: &str) -> Option<Value> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0usize;
            let mut in_str = false;
            let mut esc = false;
            let mut j = i;
            while j < bytes.len() {
                let b = bytes[j];
                if in_str {
                    if esc {
                        esc = false;
                    } else if b == b'\\' {
                        esc = true;
                    } else if b == b'"' {
                        in_str = false;
                    }
                } else {
                    match b {
                        b'"' => in_str = true,
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                if let Ok(v) = serde_json::from_str::<Value>(&s[i..=j]) {
                                    if v.is_object() {
                                        return Some(v);
                                    }
                                }
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }
    None
}

/// Parse one model reply: exactly one JSON object, either a tool call or
/// done. Tolerates ```json fences or brief wrapper prose around the object.
pub fn parse_model_reply(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.strip_suffix("```").unwrap_or(s))
        .unwrap_or(trimmed)
        .trim();
    if let Ok(v) = serde_json::from_str::<Value>(unfenced) {
        if v.is_object() {
            return Ok(normalize_model_reply(v));
        }
    }
    if let Some(v) = extract_first_json_object(unfenced) {
        return Ok(normalize_model_reply(v));
    }
    // Last resort: first { .. last } slice.
    if let (Some(a), Some(b)) = (unfenced.find('{'), unfenced.rfind('}')) {
        if a < b {
            if let Ok(v) = serde_json::from_str::<Value>(&unfenced[a..=b]) {
                if v.is_object() {
                    return Ok(normalize_model_reply(v));
                }
            }
        }
    }
    Err(format!("model reply was not a JSON object: {}", truncate(raw, 300)))
}

// ---------------------------------------------------------------------------
// Agent loop
// ---------------------------------------------------------------------------

/// The learned facts and preferences in a memory file: every bullet
/// that is not an identity line. Identity is exactly the two seeded
/// lines (`- Name: …`, `- Role: …`); everything else in the file is
/// something the agent learned by working with this person.
///
/// (An earlier version excluded the whole "## About" section — but
/// learned notes are appended at the end of the file, which for a
/// seeded person is still inside About, so learning silently never
/// reached the prompt for anyone created through the people feature.)
pub fn standing_preferences(memory: &str) -> Vec<String> {
    let mut prefs = Vec::new();
    for line in memory.lines() {
        let t = line.trim();
        if let Some(bullet) = t.strip_prefix("- ") {
            let bullet = bullet.trim();
            if bullet.is_empty() || is_identity_bullet(bullet) {
                continue;
            }
            prefs.push(bullet.to_string());
        }
    }
    prefs
}

/// An identity bullet is `Name: …` or `Role: …` in any casing, with
/// optional space before the colon. The check is spelling-tolerant
/// because the boundary it guards (identity vs learned facts) must not
/// depend on exact capitalisation.
fn is_identity_bullet(bullet: &str) -> bool {
    // Normalise the fullwidth colon too — the boundary must not depend
    // on which colon character a note happened to use.
    let lower = bullet.to_lowercase().replace('\u{ff1a}', ":");
    for key in ["name", "role"] {
        if let Some(rest) = lower.strip_prefix(key) {
            if rest.trim_start().starts_with(':') {
                return true;
            }
        }
    }
    false
}

/// The armed run's SECURITY section (empty outside a run).
fn security_block() -> String {
    tripwire::prompt_section()
        .map(|s| format!("\n\n{s}"))
        .unwrap_or_default()
}

/// Characters of memory.md a prompt carries; preferences listed apart.
const MEMORY_PROMPT_CHARS: usize = 10_000;
const PREFS_IN_PROMPT: usize = 60;

/// Memory as the prompt carries it: all of it while it is small; once it
/// grows, the opening (who the person is) plus the most recent learning,
/// with a plain note of what was left out. The file itself keeps everything.
pub fn memory_for_prompt(memory: &str) -> String {
    let total = memory.chars().count();
    if total <= MEMORY_PROMPT_CHARS {
        return memory.to_string();
    }
    let lines: Vec<&str> = memory.lines().collect();
    // The head: up to the end of the About section (or 1,500 characters).
    let mut head = Vec::new();
    let mut used = 0usize;
    for (i, l) in lines.iter().enumerate() {
        if i > 0 && l.starts_with("## ") && !l.to_lowercase().contains("about") {
            break;
        }
        used += l.chars().count() + 1;
        if used > 1_500 {
            break;
        }
        head.push(*l);
    }
    let budget = MEMORY_PROMPT_CHARS.saturating_sub(used + 120);
    let mut tail: Vec<&str> = Vec::new();
    let mut tail_used = 0usize;
    for l in lines[head.len()..].iter().rev() {
        let n = l.chars().count() + 1;
        if tail_used + n > budget {
            break;
        }
        tail_used += n;
        tail.push(*l);
    }
    tail.reverse();
    let omitted = lines.len() - head.len() - tail.len();
    format!(
        "{}\n(… {omitted} older lines of memory are not shown here; they are kept in memory.md …)\n{}",
        head.join("\n"),
        tail.join("\n")
    )
}

fn recent_prefs(memory: &str) -> Vec<String> {
    let mut prefs = standing_preferences(memory);
    if prefs.len() > PREFS_IN_PROMPT {
        prefs.drain(..prefs.len() - PREFS_IN_PROMPT);
    }
    prefs
}

fn system_prompt(cfg: &Config, suite_items: &[library::LibraryItem]) -> String {
    let full_memory = cfg.read_memory();
    let prefs = recent_prefs(&full_memory);
    let memory = memory_for_prompt(&full_memory);
    let prefs_block = if prefs.is_empty() {
        "(none yet — learn how this person works as you go, and record it with update_memory)".to_string()
    } else {
        prefs
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    // The person's suite: installed library skills are callable tools
    // (by item id, with their declared parameters); installed
    // subagents are delegable by id. Apps are run by the person from
    // their suite panel, not called as tools.
    let mut suite_block = String::new();
    let skills: Vec<&library::LibraryItem> = suite_items
        .iter()
        .filter(|i| i.kind == "skill" && !i.builtin)
        .collect();
    if !skills.is_empty() {
        suite_block.push_str("\n         LIBRARY SKILLS IN THIS PERSON'S SUITE (call like any tool, by id):\n");
        for s in skills {
            let params = s
                .params
                .iter()
                .map(|p| format!("\"{p}\": \"...\""))
                .collect::<Vec<_>>()
                .join(", ");
            suite_block.push_str(&format!(
                "         - {} {{{}}} — {}\n",
                s.id, params, s.description
            ));
        }
    }
    let subagents: Vec<&library::LibraryItem> = suite_items
        .iter()
        .filter(|i| i.kind == "subagent")
        .collect();
    if !subagents.is_empty() {
        suite_block.push_str("\n         SUBAGENTS YOU CAN DELEGATE TO (via the delegate tool):\n");
        for s in subagents {
            suite_block.push_str(&format!("         - {} — {}\n", s.id, s.description));
        }
    }
    format!(
        "You are Ergon, a personal AI agent for one employee. Your job is to FINISH their digital work autonomously, end to end — not to prepare drafts for approval, not to ask for confirmation. You act through tools and you keep going until the work is done.\n\n\
         PROTOCOL: Reply with exactly ONE JSON object per message, nothing else.\n\
         - To use a tool: {{\"tool\": \"<name>\", \"args\": {{...}}}}\n\
         - When the whole task is complete: {{\"done\": true, \"summary\": \"<what was accomplished>\"}}\n\n\
         TOOLS:\n\
         - list_dir {{\"path\": \".\"}} — list files in the workspace (paths are relative to the workspace root).\n\
         - read_file {{\"path\": \"notes.txt\"}} — read a workspace file. It also reads PDF, Word (.docx), PowerPoint (.pptx) and Excel (.xlsx: one sheet as CSV; \"book.xlsx#Sheet2\" for another). Long files arrive in windows: pass \"offset\" as the result says to read on.\n\
         - write_file {{\"path\": \"out.txt\", \"content\": \"...\"}} — write a workspace file.\n\
         - fetch_url {{\"url\": \"https://...\"}} — fetch a URL and read its text.\n\
         - export_file {{\"path\": \"Weekly Report.md\"}} — turn a finished Markdown/text file into a Word document (.docx), or a CSV into an Excel workbook (.xlsx), ready to share. Use it whenever the person wants a Word or Excel file, or a document to send to someone.\n\
         - write_document {{\"title\": \"Summary\", \"sections\": [\"section text\", ...]}} — produce a finished markdown document in the workspace (a Rune skill formats and writes it).\n\
         - make_spreadsheet {{\"title\": \"Budget\", \"columns\": [\"Item\", \"Cost\"], \"rows\": [[\"Coffee\", \"4.50\"]]}} — produce a finished CSV spreadsheet in the workspace (a Rune skill formats and writes it).\n\
         - research {{\"topic\": \"Local Prices\", \"urls\": [\"https://...\", \"https://...\"]}} — fetch several pages and compile them into one research notes document in the workspace (a Rune skill).\n\
         - search_files {{\"query\": \"quarterly\", \"path\": \".\"}} — search the workspace's text files for a phrase; returns matching lines with file and line number.\n\
         - move_file {{\"from\": \"a.txt\", \"to\": \"archive/a.txt\"}} — move or rename a workspace file.\n\
         - update_memory {{\"note\": \"...\"}} — record something you learned about this person (a preference or fact) into their long-term memory (a Rune skill).\n\
         - reconcile_ledger {{\"source_files\": [\"expenses.csv\"], \"output_title\": \"Q4 Reconciliation\"}} — reconcile one or more expense/invoice ledgers (CSV or Excel; \"book.xlsx#Sheet\" for a sheet) with penny-exact totals, duplicate-reference, possible-double-charge and missing-receipt flags, and vendor rollups (Rune skill).\n\
         - summarize_meeting {{\"notes_file\": \"notes.md\", \"output_title\": \"Meeting Summary\"}} — extract decisions, owners, due dates, blockers, and follow-up drafts from meeting notes (.md, .txt, Word, PDF or a PowerPoint deck) (Rune skill).\n\
         - audit_contract {{\"contract_file\": \"msa.md\", \"required_clauses\": [\"termination\", \"liability\"], \"output_title\": \"Contract Audit\"}} — audit a contract (.md, Word or PDF) for renewal dates, missing clauses, and line-cited risk redlines (Rune skill).\n\
         - triage_inbox {{\"inbox_file\": \"inbox.md\", \"output_title\": \"Inbox Triage\"}} — classify tickets/messages into Urgent / Action / FYI queues with next steps (Rune skill).\n\
         - clean_dataset {{\"input_file\": \"raw.csv\", \"key_column\": \"id\", \"output_title\": \"Cleaned Data\"}} — normalize CSV/JSON columns, deduplicate rows, flag outliers, and write clean CSV + diff report (Rune skill).\n\
         - compare_documents {{\"left_file\": \"v1.md\", \"right_file\": \"v2.md\", \"output_title\": \"Diff Report\"}} — produce a line-by-line comparison and change summary between two workspace files (Rune skill).\n\
         - draft_brief {{\"topic\": \"...\", \"source_files\": [\"a.md\"], \"audience\": \"Leadership\", \"output_title\": \"Brief\"}} — draft a source-cited executive brief grounded strictly in workspace files and memory (Rune skill).\n\
         - extract_tables {{\"source_file\": \"notes.md\", \"output_title\": \"Extracted Table\"}} — extract Markdown tables, TSV, or structured logs into RFC-4180 CSV spreadsheet(s) (Rune skill).\n\
         - browse_library {{\"query\": \"spreadsheet\"}} — search the company-wide library of skills, subagents and apps shared by every person's agent.\n\
         - install_item {{\"item_id\": \"daily-log\"}} — add a library item to this person's suite, so it is yours to use from the next task on.\n\
         - publish_skill {{\"name\": \"...\", \"description\": \"...\", \"params\": [\"entry\"], \"source\": \"<Rune script>\"}} — contribute a reusable skill you wrote to the company library, so every person's agent can use it. Library skills are Rune scripts whose main takes the declared params as strings in order and returns a string (or \"file:<name>\" after writing a file through host_workspace_write). A script that does not compile is refused.\n\
         - delegate {{\"subagent\": \"researcher\", \"task\": \"...\"}} — hand a focused sub-task to a specialist subagent from the suite; it works with the same tools and returns its result to you.\n\
         - delete_file {{\"path\": \"old.txt\"}} — CONSEQUENTIAL: deleting is permanent, so this does not delete immediately — it parks the deletion as a decision for your person to approve. Never claim a file was deleted when it was only parked.\n\
         - flag_decision {{\"summary\": \"...\"}} — raise a question as a decision in your person's inbox when a choice is genuinely theirs to make.{suite_block}\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. When you learn a durable preference or fact about this person, record it with update_memory before finishing. When you build something reusable, publish it to the library so the whole company benefits. Finish with done only when the task is actually complete.{security_block}\n\n\
         PREFERENCES AND FACTS THIS PERSON HAS TAUGHT YOU — apply them in every task:\n{prefs_block}\n\n\
         WHAT YOU KNOW ABOUT THIS PERSON (their memory file — treat it as true and use it):\n{memory}",
        security_block = security_block()
    )
}

pub struct AgentOutcome {
    pub summary: String,
    pub steps: Vec<String>,
}

/// The files one run actually wrote — the evidence a "done" summary's
/// named files are checked against. Not a count: WHICH files, so a
/// summary claiming a file this run never touched is visible as
/// exactly that. `consumed` are the move sources: files this run
/// legitimately made disappear. Shared with subagent runs, whose
/// writes are the parent run's work product too.
#[derive(Default)]
pub struct WriteLedger {
    pub written: Vec<String>,
    pub consumed: Vec<String>,
    /// Tool steps that succeeded (a failed step is not work, EB2b).
    pub ok_steps: usize,
}

fn credit_writes(
    ledger: &mut WriteLedger,
    cfg: &Config,
    tool: &str,
    args: &Value,
    result_text: &str,
    dynamic_skills: &std::collections::HashSet<String>,
) {
    match tool {
        "write_file" => {
            if let Some(p) = args["path"].as_str() {
                ledger.written.push(p.to_string());
            }
        }
        "move_file" => {
            if let Some(p) = args["to"].as_str() {
                ledger.written.push(p.to_string());
            }
            if let Some(p) = args["from"].as_str() {
                ledger.consumed.push(p.to_string());
            }
        }
        // A memory update is real work product too: the summary may
        // honestly name memory.md (it lives in the data dir, not the
        // workspace).
        "update_memory" => {
            ledger.written.push("memory.md".to_string());
        }
        // The Rune skills name the file they produced in their
        // result text.
        "write_document" | "make_spreadsheet" | "research" | "export_file" => {
            ledger
                .written
                .extend(extract_written(result_text, &cfg.workspace_dir));
        }
        // Built-in Rune skills and library skills from the person's suite:
        // credit every file named in the tool result that actually exists in
        // the workspace (deduplicated).
        other if skills::is_builtin_skill(other) || dynamic_skills.contains(other) => {
            for f in extract_written(result_text, &cfg.workspace_dir) {
                if !ledger.written.contains(&f) {
                    ledger.written.push(f);
                }
            }
            for cands in mention_candidates(result_text) {
                if let Some(found) = cands.iter().find(|c| {
                    sandbox_path(&cfg.workspace_dir, c)
                        .map(|p| p.exists())
                        .unwrap_or(false)
                }) {
                    if !ledger.written.contains(found) {
                        ledger.written.push(found.clone());
                    }
                }
            }
        }
        _ => {}
    }
}

/// The system prompt for a subagent run: the profile's own
/// instructions, the same JSON tool protocol, the person's standing
/// preferences, and the working toolset — minus the tools that belong
/// to the main agent alone (delegating further, publishing, and the
/// consequential pair).
fn subagent_system_prompt(
    cfg: &Config,
    profile: &library::LibraryItem,
    suite_items: &[library::LibraryItem],
) -> String {
    let memory = cfg.read_memory();
    let prefs = recent_prefs(&memory);
    let prefs_block = if prefs.is_empty() {
        "(none yet)".to_string()
    } else {
        prefs
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut skills_block = String::new();
    for s in suite_items.iter().filter(|i| i.kind == "skill" && !i.builtin) {
        let params = s
            .params
            .iter()
            .map(|p| format!("\"{p}\": \"...\""))
            .collect::<Vec<_>>()
            .join(", ");
        skills_block.push_str(&format!(
            "\n         - {} {{{}}} — {}",
            s.id, params, s.description
        ));
    }
    format!(
        "You are {}, a specialist subagent working for a person's main Ergon agent. {}\n\n\
         PROTOCOL: Reply with exactly ONE JSON object per message, nothing else.\n\
         - To use a tool: {{\"tool\": \"<name>\", \"args\": {{...}}}}\n\
         - When your sub-task is complete: {{\"done\": true, \"summary\": \"<what you did and found>\"}}\n\n\
         TOOLS:\n\
         - list_dir {{\"path\": \".\"}} — list files in the workspace.\n\
         - read_file {{\"path\": \"notes.txt\"}} — read a workspace file (also PDF, .docx, .pptx, and .xlsx as CSV; \"offset\" reads on in long files).\n\
         - write_file {{\"path\": \"out.txt\", \"content\": \"...\"}} — write a workspace file.\n\
         - fetch_url {{\"url\": \"https://...\"}} — fetch a URL and read its text.\n\
         - export_file {{\"path\": \"Summary.md\"}} — make a Word (.docx) copy of a Markdown file, or an Excel (.xlsx) copy of a CSV.\n\
         - write_document {{\"title\": \"Summary\", \"sections\": [\"...\"]}} — produce a finished markdown document.\n\
         - make_spreadsheet {{\"title\": \"Budget\", \"columns\": [\"Item\", \"Cost\"], \"rows\": [[\"Coffee\", \"4.50\"]]}} — produce a finished CSV spreadsheet.\n\
         - research {{\"topic\": \"...\", \"urls\": [\"https://...\"]}} — compile research notes from several pages.\n\
         - search_files {{\"query\": \"...\", \"path\": \".\"}} — search workspace text files.\n\
         - move_file {{\"from\": \"a.txt\", \"to\": \"archive/a.txt\"}} — move or rename a workspace file.\n\
         - update_memory {{\"note\": \"...\"}} — record something learned about this person.{skills}\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. Your summary goes back to the main agent, so state plainly what you produced and where.\n\n\
         PREFERENCES AND FACTS THIS PERSON HAS TAUGHT THEIR AGENT:\n{prefs}{security}",
        profile.name, profile.instructions, prefs = prefs_block, skills = skills_block, security = security_block()
    )
}

/// Live execution state for an in-flight agent run so the UI can stream tool
/// steps in real time (via SSE `/api/runs/stream` or polling `/api/runs/status`)
/// and cancel an active run (`POST /api/runs/:id/cancel`).
#[derive(Clone, Debug)]
pub struct LiveRun {
    pub run_id: String,
    pub thread_id: String,
    pub task: String,
    pub started_ms: u64,
    pub steps: Vec<String>,
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
}

pub type LiveRuns = Arc<Mutex<HashMap<String, LiveRun>>>;

static RUN_ID_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub struct LiveStreamCtx<'a> {
    pub runs: &'a LiveRuns,
    pub events: &'a sse::RunEventBus,
    pub person_id: &'a str,
    pub run_id: &'a str,
    pub cancel: &'a Arc<std::sync::atomic::AtomicBool>,
}

const MAX_LIVE_STEPS: usize = 128;

fn push_live_step(live: Option<LiveStreamCtx<'_>>, step: String) {
    if let Some(ctx) = live {
        if let Ok(mut map) = ctx.runs.lock() {
            if let Some(run) = map.get_mut(ctx.person_id) {
                if run.steps.len() >= MAX_LIVE_STEPS {
                    run.steps.remove(0);
                }
                run.steps.push(truncate(&step, 2048));
            }
        }
    }
}

fn is_cancelled(live: Option<LiveStreamCtx<'_>>) -> bool {
    live.map(|c| c.cancel.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Returns true when the user prompt explicitly names a built-in workspace tool
/// or workflow (e.g. `draft_brief`, `audit_contract`, `read_file`) or a
/// workspace filename (`.md`, `.csv`, `.txt`, `.json`, `.log`, `.tsv`, excluding
/// URLs). Used to prevent a model from short-circuiting with `{"done": true}`
/// before inspecting or running any tool against the real workspace.
pub fn task_requires_workspace_tool(task: &str) -> bool {
    const TOOLS: &[&str] = &[
        "draft_brief",
        "audit_contract",
        "reconcile_ledger",
        "summarize_meeting",
        "triage_inbox",
        "clean_dataset",
        "compare_documents",
        "extract_tables",
        "write_file",
        "read_file",
        "delete_file",
        "make_spreadsheet",
        "write_document",
        "update_memory",
        "search_files",
        "list_dir",
        "move_file",
        "fetch_url",
        "flag_decision",
        "export_file",
    ];
    const EXTS: &[&str] = &[".md", ".csv", ".txt", ".json", ".log", ".tsv", ".pdf", ".docx", ".xlsx", ".pptx"];

    for tok in task.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if !tok.is_empty() {
            let low = tok.to_ascii_lowercase();
            if TOOLS.contains(&low.as_str()) {
                return true;
            }
        }
    }

    for raw in task.split_whitespace() {
        let cleaned = raw.trim_matches(|c: char| {
            matches!(
                c,
                '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '.' | '!' | '?'
            )
        });
        let low = cleaned.to_ascii_lowercase();
        if low.starts_with("http://") || low.starts_with("https://") || low.starts_with("mailto:") {
            continue;
        }
        if EXTS
            .iter()
            .any(|ext| low.len() > ext.len() && low.ends_with(ext))
        {
            return true;
        }
    }

    false
}

/// One agent loop: model → tool → model → … until done or the step
/// cap. Steps are pushed into the thread as they happen (subagent
/// steps are labelled with the subagent's name). Returns the done
/// summary. The honesty gate and the work log belong to the top-level
/// run (`run_agent`), not to subagent runs.
#[allow(clippy::too_many_arguments)]
fn run_loop(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    system: &str,
    task: &str,
    history: &[Value],
    allow_delegate: bool,
    step_cap: usize,
    label: Option<&str>,
    ledger: &mut WriteLedger,
    dynamic_skills: &mut std::collections::HashSet<String>,
    suite_items: &[library::LibraryItem],
    budget: &mut usize,
    session_tag: &str,
    live: Option<LiveStreamCtx<'_>>,
) -> Result<(String, Vec<String>), String> {
    // Subagent runs get their own provider session: if the provider
    // keys conversation state on the session header, the two
    // personas must not interleave in one server-side conversation.
    let session = if session_tag.is_empty() {
        format!("ergon-{thread_id}")
    } else {
        format!("ergon-{thread_id}-{session_tag}")
    };
    let mut messages: Vec<Value> = vec![json!({"role": "system", "content": system})];
    messages.extend(history.iter().cloned());
    messages.push(json!({"role": "user", "content": task}));
    let mut steps: Vec<String> = Vec::new();

    // Two caps, both enforced: this loop's own step cap, and the
    // whole run's shared model-call budget. A delegate call used to
    // cost the parent 1 step and buy a fresh 12 — delegation
    // multiplied the cap (24 delegations ≈ 312 model calls in one
    // task). Now every call, parent or subagent, draws from the
    // same budget of STEP_CAP.
    let mut local_steps = 0usize;
    let mut json_retries = 0usize;
    loop {
        if is_cancelled(live) {
            return Err("run cancelled by user".to_string());
        }
        if let Some(reason) = tripwire::halted() {
            return Err(format!("{}{reason}", tripwire::HALT_PREFIX));
        }
        if local_steps >= step_cap || *budget >= STEP_CAP {
            break;
        }
        local_steps += 1;
        *budget += 1;
        let reply = match call_model(cfg, &session, &messages) {
            Ok(v) => v,
            Err(e) if e.starts_with("model reply was not a JSON object:") && json_retries < 2 => {
                json_retries += 1;
                messages.push(json!({
                    "role": "user",
                    "content": "Protocol reminder: reply with ONLY one valid JSON object — either {\"tool\": \"<name>\", \"args\": {...}} (e.g. {\"tool\": \"write_file\", \"args\": {\"path\": \"...\", \"content\": \"...\"}}) or {\"done\": true, \"summary\": \"...\"}."
                }));
                continue;
            }
            Err(e) => return Err(e),
        };
        if reply["done"].as_bool() == Some(true) {
            if steps.is_empty() && json_retries < 2 && task_requires_workspace_tool(task) {
                json_retries += 1;
                messages.push(json!({ "role": "assistant", "content": reply.to_string() }));
                messages.push(json!({
                    "role": "user",
                    "content": "Protocol reminder: you have not called any tool yet. Do NOT reply {\"done\": true} before running the requested workspace tool. Call {\"tool\": \"<name>\", \"args\": {...}} against the real workspace now so missing source files or real tool results are checked honestly."
                }));
                continue;
            }
            let summary = reply["summary"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("Task complete.")
                .to_string();
            return Ok((summary, steps));
        }
        let tool = match reply["tool"].as_str() {
            Some(t) if !t.trim().is_empty() => t.to_string(),
            _ if json_retries < 2 => {
                json_retries += 1;
                messages.push(json!({ "role": "assistant", "content": reply.to_string() }));
                messages.push(json!({
                    "role": "user",
                    "content": "Protocol reminder: wrap tool calls with a \"tool\" key and an \"args\" object, like {\"tool\": \"write_file\", \"args\": {\"path\": \"...\", \"content\": \"...\"}}, or finish with {\"done\": true, \"summary\": \"...\"}."
                }));
                continue;
            }
            _ => {
                return Err(format!(
                    "model reply was neither a tool call nor done: {}",
                    truncate(&reply.to_string(), 300)
                ));
            }
        };
        let mut args = reply["args"].clone();
        if is_cancelled(live) {
            return Err("run cancelled by user".to_string());
        }
        let step_t0 = std::time::Instant::now();
        if let Some(lctx) = live {
            lctx.events.publish(
                "step_start",
                lctx.person_id,
                lctx.run_id,
                thread_id,
                json!({
                    "step_index": local_steps,
                    "tool": tool,
                    "label": label,
                    "ts_ms": records::now_millis(),
                }),
            );
        }
        let prev_written_len = ledger.written.len();
        // A tripwire hold (memory note, library publish) is a real,
        // successful step — but it wrote nothing, so it earns no credit.
        let mut held = false;

        let (result_text, ok) = if tool == "delegate" {
            if !allow_delegate {
                (
                    "error: subagents cannot delegate further — do this part yourself".to_string(),
                    false,
                )
            } else {
                run_delegation(
                    cfg, ctx, store, thread_id, &args, ledger, dynamic_skills, suite_items,
                    budget, live,
                )
            }
        } else if !allow_delegate
            && matches!(
                tool.as_str(),
                "publish_skill" | "install_item" | "delete_file" | "flag_decision"
                    | "browse_library"
            )
        {
            // The subagent toolset restriction is enforced HERE, not
            // just written in its prompt: a subagent must not publish
            // to the company library, change the suite, or park
            // decisions in the person's inbox under the main agent's
            // authorship. (Deletion itself was never reachable from
            // any loop — it only executes on human approval.)
            (
                format!(
                    "error: subagents cannot use {tool} — that belongs to the main agent. Do this part with your own tools, or say in your summary that the main agent should do it."
                ),
                false,
            )
        } else {
            // Consequential tools carry the originating thread, so a
            // settled decision can report back into it.
            if matches!(tool.as_str(), "delete_file" | "flag_decision") {
                args["_thread"] = json!(thread_id);
            }
            // Tripwire: refuse calls carrying the bait key or a secret,
            // calls reaching outside the workspace for credentials, and
            // hold suspicious memory notes / library publishes.
            let library_skill = dynamic_skills.contains(&tool);
            match tripwire::pre_tool(&tool, &args, library_skill) {
                tripwire::PreTool::Refuse(msg) => (format!("error: {msg}"), false),
                tripwire::PreTool::Hold(msg) => {
                    held = true;
                    (msg, true)
                }
                tripwire::PreTool::Proceed => {
                    let res = execute_tool_full(cfg, ctx, &tool, &args);
                    tripwire::post_tool(&tool, &args, &res);
                    match res {
                        Ok(r) => (r, true),
                        Err(e) => (format!("error: {e}"), false),
                    }
                }
            }
        };
        if ok && held {
            ledger.ok_steps += 1;
        } else if ok {
            ledger.ok_steps += 1;
            credit_writes(ledger, cfg, &tool, &args, &result_text, dynamic_skills);
            // A skill installed mid-run is callable from the very
            // next step (the dispatcher re-reads the suite) — so the
            // honesty gate must credit it too, or an honest run gets
            // a false "this run did not write" note.
            if tool == "install_item" {
                if let Some(id) = args["item_id"].as_str() {
                    dynamic_skills.insert(id.to_string());
                }
            }
        }
        let step_elapsed_ms = step_t0.elapsed().as_millis() as u64;
        let step_line = match label {
            Some(l) => format!("Tool: {tool} (via {l})"),
            None => format!("Tool: {tool}"),
        };
        steps.push(step_line.clone());
        let rendered_step = format!("{step_line}\n{}", truncate(&result_text, 500));
        push_live_step(live, rendered_step.clone());
        if let Some(lctx) = live {
            let newly_written: Vec<String> = ledger.written[prev_written_len..].to_vec();
            for p in &newly_written {
                let bytes = sandbox_path_resolving(&cfg.workspace_dir, p, true)
                    .ok()
                    .and_then(|full| std::fs::metadata(full).ok())
                    .map(|m| m.len())
                    .unwrap_or(0);
                lctx.events.publish(
                    "file_written",
                    lctx.person_id,
                    lctx.run_id,
                    thread_id,
                    json!({
                        "path": p,
                        "bytes": bytes,
                        "ts_ms": records::now_millis(),
                    }),
                );
            }
            if ok && matches!(tool.as_str(), "delete_file" | "flag_decision") {
                lctx.events.publish(
                    "approval_parked",
                    lctx.person_id,
                    lctx.run_id,
                    thread_id,
                    json!({
                        "tool": tool,
                        "path": args["path"].as_str(),
                        "summary": truncate(&result_text, 240),
                        "ts_ms": records::now_millis(),
                    }),
                );
            }
            lctx.events.publish(
                "step_done",
                lctx.person_id,
                lctx.run_id,
                thread_id,
                json!({
                    "step_index": local_steps,
                    "tool": tool,
                    "ok": ok,
                    "elapsed_ms": step_elapsed_ms,
                    "summary": truncate(&result_text, 240),
                    "files_written": newly_written,
                    "ts_ms": records::now_millis(),
                }),
            );
        }
        store.lock().unwrap().push(thread_id, "step", &rendered_step);
        // Guard against small local models looping on delete_file after a file
        // has already been parked for approval in this run.
        if tool == "delete_file" && ok && result_text.starts_with("already parked for your person's approval") {
            let rel = args["path"].as_str().unwrap_or("the file");
            return Ok((
                format!("Parked deletion of {rel} for your approval in Today's desk. The file has not been deleted yet."),
                steps,
            ));
        }
        // A tripped critical trap (or a score over the line) stops the
        // run here: nothing more reaches the model or the tools.
        if let Some(reason) = tripwire::halted() {
            return Err(format!("{}{reason}", tripwire::HALT_PREFIX));
        }
        messages.push(json!({"role": "assistant", "content": reply.to_string()}));
        messages.push(json!({
            "role": "user",
            "content": tripwire::spotlight(&tool, &truncate(&result_text, 6000)),
        }));
    }
    Err(if *budget >= STEP_CAP {
        format!(
            "agent stopped after {STEP_CAP} steps (the run's total budget, shared with its subagents) without finishing — the task may be blocked or too large for one run."
        )
    } else {
        format!(
            "agent stopped after {step_cap} steps without finishing — the task may be blocked or too large for one run."
        )
    })
}

/// The delegate tool: run one suite subagent on a focused sub-task.
/// Its steps land in the same thread (labelled), its writes credit
/// the parent run's ledger, and its summary returns as the tool
/// result the main agent continues from.
#[allow(clippy::too_many_arguments)]
fn run_delegation(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    args: &Value,
    ledger: &mut WriteLedger,
    dynamic_skills: &mut std::collections::HashSet<String>,
    suite_items: &[library::LibraryItem],
    budget: &mut usize,
    live: Option<LiveStreamCtx<'_>>,
) -> (String, bool) {
    let sub_id = args["subagent"].as_str().unwrap_or("");
    let task = args["task"].as_str().unwrap_or("");
    if sub_id.is_empty() || task.is_empty() {
        return (
            "error: delegate needs args.subagent and args.task".to_string(),
            false,
        );
    }
    let Some(profile) = suite_items
        .iter()
        .find(|i| i.id == sub_id && i.kind == "subagent")
        .cloned()
    else {
        return (
            format!(
                "error: no subagent '{sub_id}' in this person's suite — delegate only to a subagent the suite carries"
            ),
            false,
        );
    };
    let delegated_line = format!("Delegated to {}: {}", profile.name, truncate(task, 300));
    push_live_step(live, delegated_line.clone());
    store.lock().unwrap().push(thread_id, "step", &delegated_line);
    let system = subagent_system_prompt(cfg, &profile, suite_items);
    match run_loop(
        cfg,
        ctx,
        store,
        thread_id,
        &system,
        task,
        &[],
        false,
        12,
        Some(&profile.name),
        ledger,
        dynamic_skills,
        suite_items,
        budget,
        &profile.id,
        live,
    ) {
        Ok((summary, _)) => (format!("{} reports: {}", profile.name, summary), true),
        Err(e) => (format!("error: subagent {} failed: {e}", profile.name), false),
    }
}

/// Run one task to completion for the effective person: the loop,
/// then the honesty gate on its summary, the work log entry, and an
/// offers scan — the records the brief, the graph and the offers
/// engine are built from.
/// Agent runs in progress, so an update never restarts the server under one.
pub(crate) static RUNS_IN_FLIGHT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct RunGuard {
    live: Option<(LiveRuns, String)>,
}

impl RunGuard {
    fn new(
        live: Option<(&LiveRuns, &sse::RunEventBus, &str)>,
        run_id: &str,
        thread_id: &str,
        task: &str,
        started_ms: u64,
        cancel: Arc<std::sync::atomic::AtomicBool>,
    ) -> RunGuard {
        RUNS_IN_FLIGHT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let stored = live.map(|(runs, events, pid)| {
            if let Ok(mut map) = runs.lock() {
                map.insert(
                    pid.to_string(),
                    LiveRun {
                        run_id: run_id.to_string(),
                        thread_id: thread_id.to_string(),
                        task: task.to_string(),
                        started_ms,
                        steps: Vec::new(),
                        cancel,
                    },
                );
            }
            events.publish(
                "run_started",
                pid,
                run_id,
                thread_id,
                json!({
                    "task": truncate(task, 240),
                    "started_ms": started_ms,
                    "ts_ms": started_ms,
                }),
            );
            (runs.clone(), pid.to_string())
        });
        RunGuard { live: stored }
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        RUNS_IN_FLIGHT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if let Some((runs, pid)) = &self.live {
            if let Ok(mut map) = runs.lock() {
                map.remove(pid);
            }
        }
    }
}

pub fn run_agent(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    task: &str,
    history: &[Value],
) -> Result<AgentOutcome, String> {
    run_agent_live(cfg, ctx, store, thread_id, task, history, None)
}

pub fn run_agent_live(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    task: &str,
    history: &[Value],
    live: Option<(&LiveRuns, &sse::RunEventBus, &str)>,
) -> Result<AgentOutcome, String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    // Deletes parked after this instant belong to this run (EB1).
    let run_started = records::now_millis();
    let run_t0 = std::time::Instant::now();
    let run_id = format!("run-{run_started}-{}", RUN_ID_SEQ.fetch_add(1, Ordering::SeqCst));
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let _in_flight = RunGuard::new(live, &run_id, thread_id, task, run_started, cancel.clone());
    // Arm this run's honeytraps BEFORE the system prompt is built (it
    // carries the run's bait key and content boundary). Provenance is the
    // person's own words: this task plus their earlier messages here.
    let provenance = {
        let mut p = task.to_string();
        for m in history {
            if m["role"] == "user" {
                p.push('\n');
                p.push_str(m["content"].as_str().unwrap_or(""));
            }
        }
        p
    };
    let mut secrets: Vec<String> = cfg.api_key.iter().cloned().collect();
    if let Ok(g) = std::env::var("ERGON_GATE_TOKEN") {
        secrets.push(g.trim().to_string());
    }
    let _traps = tripwire::arm(tripwire::ArmSpec {
        root: ctx.root_data_dir.clone(),
        person_dir: cfg.data_dir.clone(),
        workspace: cfg.workspace_dir.clone(),
        memory_path: cfg.memory_path.clone(),
        person_id: ctx.person_id.clone(),
        run_id: run_id.clone(),
        thread_id: thread_id.to_string(),
        provenance,
        secrets,
    });
    let live_ctx = live.map(|(runs, events, pid)| LiveStreamCtx {
        runs,
        events,
        person_id: pid,
        run_id: &run_id,
        cancel: &cancel,
    });
    let suite = library::load_suite(&cfg.data_dir);
    let suite_items = {
        let lib = ctx.library.lock().unwrap();
        suite.resolved(&lib)
    };
    let mut dynamic_skills: std::collections::HashSet<String> = suite_items
        .iter()
        .filter(|i| i.kind == "skill" && !i.builtin)
        .map(|i| i.id.clone())
        .collect();
    let system = system_prompt(cfg, &suite_items);
    let mut ledger = WriteLedger::default();
    let mut budget = 0usize;
    let (summary, steps) = match run_loop(
        cfg,
        ctx,
        store,
        thread_id,
        &system,
        task,
        history,
        true,
        STEP_CAP,
        None,
        &mut ledger,
        &mut dynamic_skills,
        &suite_items,
        &mut budget,
        "",
        live_ctx,
    ) {
        Ok(v) => v,
        Err(e) => {
            // A run that stopped early still did visible things
            // (files written before the cap). Record it as
            // unfinished work — Today shows it; the brief and the
            // offers engine count only finished work.
            let status_label = if cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else if e.starts_with(tripwire::HALT_PREFIX) {
                "stopped"
            } else {
                "error"
            };
            if let Some(report) = tripwire::take_report() {
                tripwire::park_run_alert(&cfg.data_dir, thread_id, task, &report, e.starts_with(tripwire::HALT_PREFIX));
            }
            records::append_work(
                cfg,
                task,
                &format!("Did not finish — {e}"),
                &ledger.written,
                thread_id,
                false,
            );
            if let Some(lctx) = live_ctx {
                lctx.events.publish(
                    "run_finished",
                    lctx.person_id,
                    lctx.run_id,
                    thread_id,
                    json!({
                        "status": status_label,
                        "error": e,
                        "elapsed_ms": run_t0.elapsed().as_millis() as u64,
                        "files": ledger.written,
                        "ts_ms": records::now_millis(),
                    }),
                );
            }
            return Err(e);
        }
    };
    // The bait key never reaches the person's thread or the work log.
    let summary = tripwire::redact(&summary);
    store
        .lock()
        .unwrap()
        .push(thread_id, "assistant", &summary);
    // Honesty gate, checked against the run's own write ledger —
    // WHICH files this run wrote, never the summary's verbs or nouns
    // (verb/noun lists flagged honest inline answers and waved
    // through claims about files the run never touched). Mentions are
    // matched against known files with spaced-name-aware candidates:
    //  1. a named file that does not exist at all is a hard
    //     discrepancy — caution, in the open;
    //  2. a named file that exists but this run did not write
    //     gets a factual ledger line (what the run DID write),
    //     so a pre-existing file claimed as the deliverable is
    //     visible without accusing an honest input reference.
    // Both lines are emitted when both apply.
    let unverified_zero_tool = steps.is_empty() && task_requires_workspace_tool(task);
    if unverified_zero_tool {
        store.lock().unwrap().push(
            thread_id,
            "step",
            "Note: this task named a workspace tool or file, but no tool ran — nothing was read, verified, or written. Treat the summary above with caution.",
        );
    }
    let (missing_named, unwritten_named) = check_mentions(
        &summary,
        &cfg.workspace_dir,
        &ledger.written,
        &ledger.consumed,
    );
    if !missing_named.is_empty() {
        store.lock().unwrap().push(
            thread_id,
            "step",
            &format!(
                "Note: the summary names {}, which is not in the workspace — treat the summary above with caution.",
                missing_named.join(", ")
            ),
        );
    }
    if !unwritten_named.is_empty() {
        let wrote = if ledger.written.is_empty() {
            "no files".to_string()
        } else {
            ledger.written.join(", ")
        };
        store.lock().unwrap().push(
            thread_id,
            "step",
            &format!(
                "Note: this run wrote {wrote}; the summary also names {}, which this run did not write.",
                unwritten_named.join(", ")
            ),
        );
    }
    // A deletion is only ever parked for approval. Every delete parked in
    // THIS run gets one factual line after the summary, whatever the
    // summary says: it has not been deleted and waits in Decisions.
    for d in records::load_decisions(&cfg.data_dir) {
        let parked_now = d["status"] == "pending"
            && d["kind"] == "delete"
            && d["thread_id"].as_str() == Some(thread_id)
            && d["ts"].as_u64().unwrap_or(0) >= run_started;
        if let Some(path) = d["path"].as_str().filter(|_| parked_now) {
            store.lock().unwrap().push(thread_id, "step", &parked_note(path));
        }
    }
    // The work log + offers scan: the records the rest of the
    // product (brief, graph, today, offers) is built from.
    // Work means a tool step that succeeded or a written file; a run whose
    // steps all failed is an answer, not finished work (EB2, EB2b).
    if unverified_zero_tool {
        records::append_work(
            cfg,
            task,
            &format!("Did not run requested tool — {summary}"),
            &[],
            thread_id,
            false,
        );
    } else if ledger.ok_steps == 0 && ledger.written.is_empty() {
        records::append_answer(cfg, task, &summary, thread_id);
    } else {
        records::append_work(cfg, task, &summary, &ledger.written, thread_id, true);
    }
    {
        let lib = ctx.library.lock().unwrap();
        let suite_now = library::load_suite(&cfg.data_dir);
        records::refresh_offers(cfg, &suite_now, &lib);
    }
    // Anything the traps blocked or held during an otherwise finished
    // run becomes one plain notice in Today.
    if let Some(report) = tripwire::take_report() {
        if let Some(id) = tripwire::park_run_alert(&cfg.data_dir, thread_id, task, &report, false) {
            store.lock().unwrap().push(
                thread_id,
                "step",
                &format!("Note: Ergon blocked or held something suspicious during this task (decision {id} in Today has the details)."),
            );
        }
    }
    if let Some(lctx) = live_ctx {
        lctx.events.publish(
            "run_finished",
            lctx.person_id,
            lctx.run_id,
            thread_id,
            json!({
                "status": "finished",
                "summary": truncate(&summary, 300),
                "elapsed_ms": run_t0.elapsed().as_millis() as u64,
                "files": ledger.written,
                "ts_ms": records::now_millis(),
            }),
        );
    }
    Ok(AgentOutcome { summary, steps })
}

#[cfg(test)]
mod v010_hardening_tests {
    use super::*;
    use crate::server::{host_allowed, read_request, refusal, Request};
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn atomic_write_bytes_writes_and_replaces_cleanly() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-atomic-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("nested").join("sample.txt");
        atomic_write_bytes(&path, b"first").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        atomic_write_bytes(&path, b"second").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp file leaked: {leftovers:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn host_and_origin_checks_are_case_insensitive() {
        let allowed = vec!["Desk.TailNet.ts.net".to_string()];
        assert!(host_allowed("LOCALHOST:8787", &allowed));
        assert!(host_allowed("desk.tailnet.ts.net:8787", &allowed));
        assert!(!host_allowed("evil.example:8787", &allowed));
        let req = Request {
            method: "POST".into(),
            path: "/api/threads".into(),
            query: String::new(),
            body: b"{}".to_vec(),
            session: None,
            gate: None,
            content_type: "application/json".into(),
            origin: Some("http://LocalHost:8787".into()),
            host: Some("localhost:8787".into()),
            sec_fetch_site: Some("same-origin".into()),
            authorization: None,
            user_agent: None,
            cookie_raw: None,
            forwarded_for: None,
            accept_encoding: None,
            if_none_match: None,
            keep_alive: false,
        };
        assert!(refusal(&req, false, &[]).is_none());
    }

    #[test]
    fn read_request_rejects_conflicting_content_length_and_chunked() {
        for raw in [
            "POST /api/threads HTTP/1.1\r\nHost: localhost:8787\r\nContent-Length: 2\r\nContent-Length: 5\r\n\r\n{}",
            "POST /api/threads HTTP/1.1\r\nHost: localhost:8787\r\nContent-Length: nope\r\n\r\n",
            "POST /api/threads HTTP/1.1\r\nHost: localhost:8787\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\n\r\n",
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let raw_owned = raw.to_string();
            let handle = std::thread::spawn(move || {
                let mut client = std::net::TcpStream::connect(addr).unwrap();
                client.write_all(raw_owned.as_bytes()).unwrap();
                let mut resp = String::new();
                let _ = client.read_to_string(&mut resp);
                resp
            });
            let (mut server, _) = listener.accept().unwrap();
            let parsed = read_request(&mut server);
            drop(server);
            let resp = handle.join().unwrap();
            assert!(parsed.is_none());
            assert!(resp.starts_with("HTTP/1.1 400 Bad Request"), "got: {resp}");
        }
    }

    #[test]
    fn read_request_returns_413_promptly_on_overflow_content_length_with_open_write_half() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let mut client = std::net::TcpStream::connect(addr).unwrap();
            client
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            client
                .write_all(
                    b"POST /api/config HTTP/1.1\r\nHost: localhost:8787\r\nContent-Type: application/json\r\nContent-Length: 99999999999999\r\n\r\n{}",
                )
                .unwrap();
            let mut resp = String::new();
            let _ = client.read_to_string(&mut resp);
            resp
        });
        let (mut server, _) = listener.accept().unwrap();
        let start = std::time::Instant::now();
        let parsed = read_request(&mut server);
        let elapsed = start.elapsed();
        drop(server);
        let resp = handle.join().unwrap();
        assert!(parsed.is_none());
        assert!(resp.starts_with("HTTP/1.1 413"), "got: {resp}");
        assert!(elapsed.as_secs() < 2, "took too long: {elapsed:?}");
    }

    #[test]
    fn read_request_rejects_duplicate_host_header() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let mut client = std::net::TcpStream::connect(addr).unwrap();
            client
                .write_all(b"GET /api/version HTTP/1.1\r\nHost: evil.example\r\nHost: localhost:8787\r\n\r\n")
                .unwrap();
            let mut resp = String::new();
            let _ = client.read_to_string(&mut resp);
            resp
        });
        let (mut server, _) = listener.accept().unwrap();
        let parsed = read_request(&mut server);
        drop(server);
        let resp = handle.join().unwrap();
        assert!(parsed.is_none());
        assert!(resp.starts_with("HTTP/1.1 400 Bad Request"), "got: {resp}");
    }

    #[test]
    fn gzip_compress_and_serve_page_etag_304_work_end_to_end() {
        let sample = b"<!doctype html><html><head><title>Ergon</title></head><body>ok</body></html>";
        let gz = tripwire::gzip_compress(sample);
        assert_eq!(&gz[..3], &[0x1f, 0x8b, 0x08]);
        let payload = &gz[10..gz.len() - 8];
        let decompressed = miniz_oxide::inflate::decompress_to_vec(payload).unwrap();
        assert_eq!(decompressed, sample);

        let root = std::env::temp_dir().join(format!(
            "ergon-v011-gzip-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::from_env();
        cfg.data_dir = root.join("data");
        cfg.workspace_dir = cfg.data_dir.join("workspace");
        cfg.memory_path = cfg.data_dir.join("memory.md");
        cfg.api_key = None;
        cfg.model_base_url = "http://127.0.0.1:9/v1".to_string();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(40));

        // 1. Gzip request returns 200 with Content-Encoding: gzip, ETag, Vary, and security headers
        let mut s1 = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        s1.write_all(
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept-Encoding: gzip, deflate\r\n\r\n").as_bytes(),
        )
        .unwrap();
        let mut raw1 = Vec::new();
        s1.read_to_end(&mut raw1).unwrap();
        let split1 = raw1.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head1 = String::from_utf8_lossy(&raw1[..split1]).to_string();
        let body1 = &raw1[split1 + 4..];
        assert!(head1.starts_with("HTTP/1.1 200 OK"), "{head1}");
        assert!(head1.to_ascii_lowercase().contains("content-encoding: gzip"), "{head1}");
        assert!(head1.to_ascii_lowercase().contains("vary: accept-encoding"), "{head1}");
        assert!(head1.to_ascii_lowercase().contains("x-content-type-options: nosniff"), "{head1}");
        assert!(head1.to_ascii_lowercase().contains("x-frame-options: deny"), "{head1}");
        assert!(head1.to_ascii_lowercase().contains("referrer-policy: no-referrer"), "{head1}");
        let etag = head1
            .lines()
            .find_map(|l| {
                let (k, v) = l.split_once(':')?;
                k.trim().eq_ignore_ascii_case("etag").then(|| v.trim().to_string())
            })
            .expect("ETag header missing");
        assert!(etag.ends_with("-gzip\""), "expected gzip ETag, got {etag}");
        assert_eq!(&body1[..3], &[0x1f, 0x8b, 0x08]);
        let html = String::from_utf8(
            miniz_oxide::inflate::decompress_to_vec(&body1[10..body1.len() - 8]).unwrap(),
        )
        .unwrap();
        assert!(html.contains("<html") && html.contains("Ergon"), "unexpected html");

        // 2. Conditional GET with If-None-Match returns 304 Not Modified with empty body
        let mut s2 = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        s2.write_all(
            format!(
                "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept-Encoding: gzip\r\nIf-None-Match: {etag}\r\n\r\n"
            )
            .as_bytes(),
        )
        .unwrap();
        let mut raw2 = Vec::new();
        s2.read_to_end(&mut raw2).unwrap();
        let split2 = raw2.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head2 = String::from_utf8_lossy(&raw2[..split2]).to_string();
        let body2 = &raw2[split2 + 4..];
        assert!(head2.starts_with("HTTP/1.1 304 Not Modified"), "{head2}");
        assert!(head2.to_ascii_lowercase().contains("vary: accept-encoding"), "{head2}");
        assert!(head2.to_ascii_lowercase().contains("x-frame-options: deny"), "{head2}");
        assert!(body2.is_empty(), "304 must have empty body");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn store_list_json_computes_accurate_thread_states() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-list-state-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let mut store = Store::load(&dir.join("threads.json"));
        let t_none = store.create_thread();
        store.push(&t_none, "user", "hello");
        let t_fin = store.create_thread();
        store.push(&t_fin, "user", "summarize");
        store.push(&t_fin, "assistant", "done");
        let t_wait = store.create_thread();
        store.push(&t_wait, "user", "delete old.txt");
        store.push(&t_wait, "step", &parked_note("old.txt"));
        store.push(&t_wait, "assistant", "waiting on you");

        let listed = store.list_json();
        let arr = listed["threads"].as_array().unwrap();
        let state_of = |id: &str| {
            arr.iter()
                .find(|x| x["id"].as_str() == Some(id))
                .and_then(|x| x["state"].as_str())
                .unwrap()
                .to_string()
        };
        assert_eq!(state_of(&t_none), "none");
        assert_eq!(state_of(&t_fin), "finished");
        assert_eq!(state_of(&t_wait), "waiting");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn history_and_export_reject_hidden_dotfiles_and_symlinks() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-sec-guard-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        assert!(file_history::validate_history_path(&ws, ".env").is_err());
        assert!(file_history::validate_history_path(&ws, "sub/.git/config").is_err());

        #[cfg(unix)]
        {
            std::fs::write(ws.join("note.md"), "# Note\nHello\n").unwrap();
            let outside = dir.join("outside.docx");
            std::fs::write(&outside, b"untouched").unwrap();
            std::os::unix::fs::symlink(&outside, ws.join("note.docx")).unwrap();
            let mut cfg = Config::from_env();
            cfg.data_dir = dir.clone();
            cfg.workspace_dir = ws.clone();
            cfg.memory_path = dir.join("memory.md");
            let err = export_workspace_file(&cfg, "note.md", Some("docx"), None).unwrap_err();
            assert!(err.contains("symlink"), "got: {err}");
            assert_eq!(std::fs::read(&outside).unwrap(), b"untouched");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn files_pack_endpoint_builds_sha256_pack_and_enforces_sandbox() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-pack-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("brief.md"), "# Executive Brief\n\nAll green.\n").unwrap();
        std::fs::write(ws.join("ledger.csv"), "item,amount\nCoffee,4.50\n").unwrap();
        #[cfg(unix)]
        {
            let outside = dir.join("secret.txt");
            std::fs::write(&outside, b"TOP_SECRET").unwrap();
            std::os::unix::fs::symlink(&outside, ws.join("leak.txt")).unwrap();
        }
        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_get = |target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!("GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", addr.port());
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        let ok_resp = send_get("/api/files/pack?files=brief.md,ledger.csv&download=1");
        assert!(ok_resp.starts_with("HTTP/1.1 200 OK"), "{ok_resp}");
        assert!(ok_resp.contains("X-Ergon-Pack-SHA256:"), "{ok_resp}");
        assert!(ok_resp.contains("X-Ergon-Pack-Preflight: pass"), "{ok_resp}");
        assert!(ok_resp.contains("X-Ergon-Pack-Ready-Count: 2"), "{ok_resp}");
        assert!(ok_resp.contains("attachment; filename=\"ergon-client-pack.md\""), "{ok_resp}");
        assert!(ok_resp.contains("# Client Deliverable Pack"), "{ok_resp}");
        assert!(ok_resp.contains("All green."), "{ok_resp}");
        assert!(ok_resp.contains("```csv\nitem,amount\nCoffee,4.50\n```"), "{ok_resp}");

        let pf_resp = send_get("/api/files/pack?preflight=1&files=brief.md,ledger.csv");
        assert!(pf_resp.starts_with("HTTP/1.1 200 OK"), "{pf_resp}");
        let pf_json: Value = serde_json::from_str(pf_resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(pf_json["ok"], json!(true));
        assert_eq!(pf_json["ready_count"], json!(2));
        assert_eq!(pf_json["empty_count"], json!(0));
        assert_eq!(pf_json["files"][0]["unit"], json!("lines"));
        assert_eq!(pf_json["files"][1]["unit"], json!("rows"));
        assert_eq!(pf_json["files"][1]["count"], json!(1));

        std::fs::write(ws.join("blank.md"), "   \n").unwrap();
        let pf_warn = send_get("/api/files/pack?preflight=1&files=brief.md,blank.md");
        let pf_warn_json: Value = serde_json::from_str(pf_warn.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(pf_warn_json["ok"], json!(false));
        assert_eq!(pf_warn_json["empty_count"], json!(1));

        let trav = send_get("/api/files/pack?files=../secret.txt");
        assert!(trav.starts_with("HTTP/1.1 403"), "{trav}");

        #[cfg(unix)]
        {
            let sym = send_get("/api/files/pack?files=leak.txt");
            assert!(sym.starts_with("HTTP/1.1 403"), "{sym}");
            assert!(!sym.contains("TOP_SECRET"), "{sym}");
        }

        let missing = send_get("/api/files/pack?files=does-not-exist.md");
        assert!(missing.starts_with("HTTP/1.1 404"), "{missing}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_bytes_with_mode_sets_private_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!(
            "ergon-mode-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("config.json");
        atomic_write_bytes_with_mode(&path, b"{\"api_key\":\"secret\"}", Some(0o600)).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn playbook_routine_chains_steps_and_packs_deliverables() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-playbook-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("notes.md"), "# Q4 Notes\nRevenue grew 22%.\n").unwrap();

        // Mock OpenAI-compatible server reading full request and returning done JSON.
        let model_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let model_addr = model_listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for (idx, mut s) in model_listener.incoming().take(8).flatten().enumerate() {
                let mut reader = std::io::BufReader::new(&mut s);
                let mut content_len = 0usize;
                loop {
                    let mut line = String::new();
                    if std::io::BufRead::read_line(&mut reader, &mut line).unwrap_or(0) == 0 || line == "\r\n" || line == "\n" {
                        break;
                    }
                    if let Some((k, v)) = line.split_once(':') {
                        if k.trim().eq_ignore_ascii_case("content-length") {
                            content_len = v.trim().parse().unwrap_or(0);
                        }
                    }
                }
                if content_len > 0 {
                    let mut body_buf = vec![0u8; content_len.min(262_144)];
                    let _ = std::io::Read::read_exact(&mut reader, &mut body_buf);
                }
                let reply_json = json!({
                    "choices": [{
                        "message": {
                            "content": json!({"done": true, "summary": format!("Step {} completed cleanly.", idx + 1)}).to_string()
                        }
                    }]
                })
                .to_string();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply_json.len(),
                    reply_json
                );
                let _ = s.write_all(resp.as_bytes());
                let _ = s.flush();
            }
        });

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        cfg.model_base_url = format!("http://127.0.0.1:{}/v1", model_addr.port());
        cfg.model_id = "mock-model".to_string();
        cfg.api_key = Some("sk-mock-test".to_string());

        let srv_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let srv_addr = srv_listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(srv_listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let post = |path: &str, body: Value| -> String {
            let raw = body.to_string();
            let mut s = std::net::TcpStream::connect(srv_addr).unwrap();
            let req = format!(
                "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                srv_addr.port(),
                raw.len(),
                raw
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        let created_resp = post(
            "/api/routines",
            json!({
                "name": "Close & Pack",
                "schedule": "manual",
                "steps": [
                    "Step one: outline quarterly highlights",
                    "Step two: finalize executive takeaways"
                ],
                "pack_deliverables": true
            }),
        );
        assert!(created_resp.starts_with("HTTP/1.1 201"), "{created_resp}");
        let body_json: Value = serde_json::from_str(created_resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        let rid = body_json["routine"]["id"].as_str().unwrap().to_string();
        assert_eq!(body_json["routine"]["steps"].as_array().unwrap().len(), 2);
        assert_eq!(body_json["routine"]["pack_deliverables"], json!(true));

        let run_resp = post(&format!("/api/routines/{rid}/run"), json!({}));
        assert!(run_resp.starts_with("HTTP/1.1 200"), "{run_resp}");
        assert!(run_resp.contains("Step 1 completed cleanly."), "{run_resp}");
        assert!(run_resp.contains("Step 2 completed cleanly."), "{run_resp}");
        let pack_text = std::fs::read_to_string(ws.join("Playbook Deliverable Pack.md")).unwrap();
        assert!(pack_text.contains("Close & Pack"), "{pack_text}");
        assert!(pack_text.contains("notes.md"), "{pack_text}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn embedded_chunks_serve_with_gzip_etag_304_and_immutable_headers() {
        let Some(asset) = ui::CHUNKS.iter().find(|c| c.pre_gzipped) else {
            return;
        };
        let route = asset.route;
        let expected_mime = asset.mime;
        let dir = std::env::temp_dir().join(format!(
            "ergon-chunks-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("workspace")).unwrap();
        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = dir.join("workspace");
        cfg.memory_path = dir.join("memory.md");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send = |req_hdrs: &str, target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n{req_hdrs}Connection: close\r\n\r\n",
                addr.port()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut buf = Vec::new();
            s.read_to_end(&mut buf).unwrap();
            String::from_utf8_lossy(&buf).into_owned()
        };

        let resp = send("Accept-Encoding: gzip\r\n", route);
        assert!(resp.starts_with("HTTP/1.1 200 OK"), "{resp}");
        assert!(resp.contains(expected_mime), "{resp}");
        assert!(resp.contains("Cache-Control: public, max-age=31536000, immutable"), "{resp}");
        assert!(resp.contains("X-Content-Type-Options: nosniff"), "{resp}");
        let etag = resp
            .lines()
            .find_map(|l| l.strip_prefix("ETag: ").map(|v| v.trim().to_string()))
            .unwrap();
        let resp304 = send(&format!("Accept-Encoding: gzip\r\nIf-None-Match: {etag}\r\n"), route);
        assert!(resp304.starts_with("HTTP/1.1 304 Not Modified"), "{resp304}");

        let missing = send("", "/chunks/nonexistent-abc.js");
        assert!(missing.starts_with("HTTP/1.1 404"), "{missing}");

        // Non-gzip fallback inflates pre-compressed .rodata transparently
        let raw_resp = send("", route);
        assert!(raw_resp.starts_with("HTTP/1.1 200 OK"), "{raw_resp}");
        assert!(!raw_resp.contains("Content-Encoding: gzip"), "{raw_resp}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn receipts_load_chain_streaming_matches_build_chain() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-stream-rcpt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("work.json"),
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "entries": [
                    {
                        "id": "rcpt-1",
                        "ts": 1760000001000u64,
                        "task": "Audit \"Vendor\" {braces} & [brackets]",
                        "summary": "Checked 3 clauses\nand flagged \"net-60\".",
                        "files": ["contract.md", "audit.md"],
                        "thread_id": "t-1",
                        "finished": true,
                        "answered": false
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("receipts.json"),
            serde_json::to_string(&json!({
                "receipts": [
                    {
                        "ts": 1760000002000u64,
                        "title": "Reconcile Q1 CSV",
                        "summary": "Matched 42 rows.",
                        "files": ["ledger.csv"],
                        "thread_id": "t-2"
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let streamed = receipts::load_chain(&dir);
        let dom = receipts::build_chain(&receipts::load_raw_entries(&dir));
        assert_eq!(streamed.len(), 2);
        assert_eq!(streamed.len(), dom.len());
        for (a, b) in streamed.iter().zip(dom.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.index, b.index);
            assert_eq!(a.task, b.task);
            assert_eq!(a.summary, b.summary);
            assert_eq!(a.files, b.files);
            assert_eq!(a.prev_hash, b.prev_hash);
            assert_eq!(a.sha256, b.sha256);
        }
        assert!(receipts::verify_chain(&streamed).0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn citations_verify_and_streaming_audit_bundle_json_bytes_work_end_to_end() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-cit-v016-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("contract.md"), "# MSA\nTerm 1\nTerm 2\nNet-60 risk\n").unwrap();
        std::fs::write(
            ws.join("report.md"),
            "See [contract.md:L4] for net-60 and [contract.md:L99] and [missing.md:L1].\n",
        )
        .unwrap();

        let rep = verify_text_citations(
            &ws,
            "Checked [contract.md:L4], [contract.md:L99], and [missing.md:L1].",
        );
        assert_eq!(rep.total, 3);
        assert_eq!(rep.verified, 1);
        assert_eq!(rep.broken, 2);
        assert_eq!(rep.citations[0].status, "verified");
        assert_eq!(rep.citations[1].status, "out_of_range");
        assert_eq!(rep.citations[2].status, "missing");

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        std::fs::write(&cfg.memory_path, "- Role: Legal\n").unwrap();
        records::append_work(
            &cfg,
            "Audit contract [contract.md:L4]",
            "Flagged [contract.md:L4]",
            &["report.md".to_string()],
            "t-cit",
            true,
        );

        let person = Person {
            id: "default".to_string(),
            name: "Default".to_string(),
            role: "Legal".to_string(),
        };
        let sec = json!({"events_sha256": receipts::GENESIS_HASH});
        let raw_bytes = audit_bundle::build_audit_bundle_json_bytes(&cfg, &person, &sec);
        let parsed: Value = serde_json::from_slice(&raw_bytes).unwrap();
        let ver = audit_bundle::verify_audit_bundle(&parsed);
        assert!(ver.valid, "streaming audit bundle failed: {:?}", ver.errors);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut s = std::net::TcpStream::connect(addr).unwrap();
        let body = json!({"text": "Check [contract.md:L4] and [contract.md:L99]."}).to_string();
        let req = format!(
            "POST /api/citations/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            addr.port(),
            body.len(),
            body
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut resp = String::new();
        s.read_to_string(&mut resp).unwrap();
        assert!(resp.starts_with("HTTP/1.1 200 OK"), "{resp}");
        let res_json: Value = serde_json::from_str(resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(res_json["verified"], json!(1));
        assert_eq!(res_json["broken"], json!(1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn receipts_compare_and_delta_pack_work_end_to_end() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-delta-v017-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("brief.md"), "# Q1 Brief\nRevenue $90k\n").unwrap();
        let rev1 = file_history::record_revision(&dir, &ws, "brief.md", "initial")
            .unwrap()
            .unwrap();
        std::fs::write(ws.join("brief.md"), "# Q1 Brief\nRevenue $100k\n").unwrap();

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        std::fs::write(&cfg.memory_path, "- Role: Finance\n").unwrap();
        records::append_work(
            &cfg,
            "Run Q1 Close v1",
            "Revenue: $90k\nOpen items: 3",
            &["brief.md".to_string()],
            "t-run-1",
            true,
        );
        records::append_work(
            &cfg,
            "Run Q1 Close v2",
            "Revenue: $100k\nOpen items: 0\nAll reconciled.",
            &["brief.md".to_string(), "summary.md".to_string()],
            "t-run-2",
            true,
        );
        std::fs::write(ws.join("summary.md"), "# Summary\nClosed.\n").unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_get = |target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                addr.port()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        let cmp_resp = send_get("/api/receipts/compare?a=t-run-1&b=t-run-2");
        assert!(cmp_resp.starts_with("HTTP/1.1 200 OK"), "{cmp_resp}");
        let cmp_json: Value = serde_json::from_str(cmp_resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(cmp_json["added"], json!(3));
        assert_eq!(cmp_json["removed"], json!(2));
        assert_eq!(cmp_json["files_added"], json!(["summary.md"]));

        let pack_run = send_get("/api/files/pack?files=brief.md,summary.md&delta_a=t-run-1&delta_b=t-run-2");
        assert!(pack_run.starts_with("HTTP/1.1 200 OK"), "{pack_run}");
        assert!(pack_run.contains("X-Ergon-Pack-Delta: yes"), "{pack_run}");
        assert!(pack_run.contains("## Verified Run Delta"), "{pack_run}");
        assert!(pack_run.contains("+All reconciled."), "{pack_run}");

        let pack_snap = send_get(&format!(
            "/api/files/pack?files=brief.md&delta_file=brief.md&rev_a={}&rev_b=current",
            rev1.rev_id
        ));
        assert!(pack_snap.starts_with("HTTP/1.1 200 OK"), "{pack_snap}");
        assert!(pack_snap.contains("X-Ergon-Pack-Delta: yes"), "{pack_snap}");
        assert!(pack_snap.contains("## Verified Snapshot Delta"), "{pack_snap}");
        assert!(pack_snap.contains("+Revenue $100k"), "{pack_snap}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn citation_freshness_stale_drift_and_html_handoff_work_end_to_end() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-freshness-v018-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        let contract_v1 = "Master Agreement\nNotice: 30 days\n";
        std::fs::write(ws.join("contract.txt"), contract_v1).unwrap();

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        std::fs::write(&cfg.memory_path, "- Role: Legal\n").unwrap();
        records::append_work(
            &cfg,
            "Audit notice period",
            "Confirmed clause [contract.txt:L2] in contract.",
            &["contract.txt".to_string()],
            "t-fresh-1",
            true,
        );
        records::append_work(
            &cfg,
            "Verify updated notice",
            "Updated clause [contract.txt:L2] in contract.",
            &["contract.txt".to_string()],
            "t-fresh-2",
            true,
        );

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_get = |target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "GET {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                addr.port()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        // 1. Live citation matches before file modification -> 100% Verified + clean HTML handoff.
        let pf_ok = send_get("/api/files/pack?preflight=1&files=contract.txt&receipt_id=t-fresh-1");
        let pf_ok_json: Value = serde_json::from_str(pf_ok.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(pf_ok_json["citation_gate"], json!("100% Verified"));
        assert_eq!(pf_ok_json["citations_verified"], json!(1));
        assert_eq!(pf_ok_json["citations_stale"], json!(0));
        assert_eq!(pf_ok_json["citations"][0]["line_excerpt"], json!("Notice: 30 days"));

        let html_resp = send_get(
            "/api/files/pack?format=html&download=1&files=contract.txt&receipt_id=t-fresh-1&delta_a=t-fresh-1&delta_b=t-fresh-2",
        );
        assert!(html_resp.starts_with("HTTP/1.1 200 OK"), "{html_resp}");
        assert!(html_resp.contains("X-Ergon-Pack-Citation-Gate: 100% Verified"), "{html_resp}");
        assert!(html_resp.contains("ergon-client-handoff.html"), "{html_resp}");
        assert!(html_resp.contains("Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'"), "{html_resp}");
        assert!(html_resp.contains("Verified Delta"), "{html_resp}");

        // 2. Record .history/ snapshot > receipt_ts and mutate contract.txt -> Stale Line Drift.
        std::thread::sleep(std::time::Duration::from_millis(10));
        file_history::record_revision(&dir, &ws, "contract.txt", "amended").unwrap();
        std::fs::write(ws.join("contract.txt"), "Master Agreement\nNotice: 60 days\n").unwrap();

        let pf_stale = send_get("/api/files/pack?preflight=1&files=contract.txt&receipt_id=t-fresh-1");
        let pf_stale_json: Value = serde_json::from_str(pf_stale.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(pf_stale_json["citation_gate"], json!("Stale Line Drift"));
        assert_eq!(pf_stale_json["citations_stale"], json!(1));
        assert_eq!(pf_stale_json["ok"], json!(false));
        assert_eq!(pf_stale_json["citations"][0]["status"], json!("stale"));
        assert_eq!(pf_stale_json["citations"][0]["line_excerpt"], json!("Notice: 60 days"));

        // 3. Batch Citation Freshness Gate across multi-run Client Summary Pack (preflight=1&summary=1, .md, .html) & Keep-Alive.
        let batch_pf = send_get("/api/files/pack?preflight=1&summary=1");
        assert!(batch_pf.starts_with("HTTP/1.1 200 OK"), "{batch_pf}");
        let batch_json: Value = serde_json::from_str(batch_pf.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(batch_json["summary_runs_count"], json!(2));
        assert_eq!(batch_json["citations_checked"], json!(1));
        assert_eq!(batch_json["citations_stale"], json!(1));
        assert_eq!(batch_json["citation_gate"], json!("Stale Line Drift"));

        let mut ka = std::net::TcpStream::connect(addr).unwrap();
        ka.write_all(
            format!(
                "GET /api/files/pack?summary=1&files=contract.txt HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n\
                 GET /api/files/pack?summary=1&format=html&files=contract.txt HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                addr.port(),
                addr.port()
            )
            .as_bytes(),
        )
        .unwrap();
        let mut combined = String::new();
        ka.read_to_string(&mut combined).unwrap();
        assert!(combined.contains("Connection: keep-alive"), "{combined}");
        assert!(combined.contains("X-Ergon-Pack-Summary: yes"), "{combined}");
        assert!(combined.contains("X-Ergon-Pack-Summary-Runs: 2"), "{combined}");
        assert!(combined.contains("X-Ergon-Pack-Summary-Stale: 1"), "{combined}");
        assert!(combined.contains("Batch Citation Gate:"), "{combined}");
        assert!(combined.contains("CITATION ALERT:"), "{combined}");
        assert!(combined.contains("## Verified Work Summary"), "{combined}");
        assert!(combined.contains("Audit notice period"), "{combined}");
        assert!(combined.contains("Verified Work Summary</h2>"), "{combined}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn playbook_pack_export_import_verifies_sha256_schema_and_size_cap() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-playbook-pack-v020-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        std::fs::write(&cfg.memory_path, "- Role: Ops\n").unwrap();

        let created = routines::create_playbook_routine(
            &dir,
            "Quarter-End Close",
            "Step 1 clean ledger",
            "weekly mon",
            None,
            Some("reconcile_ledger"),
            true,
            &routines::PlaybookOptions {
                steps: vec![
                    "Step 1 clean ledger".to_string(),
                    "Step 2 generate summary".to_string(),
                ],
                expected_inputs: vec!["ledger.csv".to_string()],
                pack_deliverables: true,
            },
        )
        .unwrap();
        let rt_id = created["id"].as_str().unwrap().to_string();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_req = |method: &str, target: &str, body: Option<&[u8]>| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let b = body.unwrap_or(&[]);
            let hdr = format!(
                "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                addr.port(),
                b.len()
            );
            s.write_all(hdr.as_bytes()).unwrap();
            if !b.is_empty() {
                s.write_all(b).unwrap();
            }
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        // 1. Export signed pack with X-Ergon-Playbook-SHA256 & .ergon-playbook.json disposition.
        let exp = send_req("GET", &format!("/api/playbooks/{rt_id}/export"), None);
        assert!(exp.starts_with("HTTP/1.1 200 OK"), "{exp}");
        assert!(exp.contains("X-Ergon-Playbook-SHA256:"), "{exp}");
        assert!(exp.contains("quarter-end-close.ergon-playbook.json"), "{exp}");
        let exp_body = exp.split("\r\n\r\n").nth(1).unwrap();
        let exp_json: Value = serde_json::from_str(exp_body).unwrap();
        assert_eq!(exp_json["format"], json!("ergon-playbook-v1"));
        assert_eq!(exp_json["playbook"]["steps"].as_array().unwrap().len(), 2);
        assert_eq!(exp_json["playbook"]["pack_deliverables"], json!(true));
        let digest = exp_json["sha256"].as_str().unwrap().to_string();
        assert_eq!(digest.len(), 64);

        // 2. Import valid pack creates verified new routine.
        let imp = send_req("POST", "/api/playbooks/import", Some(exp_body.as_bytes()));
        assert!(imp.starts_with("HTTP/1.1 201 Created"), "{imp}");
        let imp_json: Value = serde_json::from_str(imp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(imp_json["verified"], json!(true));
        assert_eq!(imp_json["sha256"], json!(digest));
        assert_eq!(imp_json["routine"]["name"], json!("Quarter-End Close"));
        assert_ne!(imp_json["routine"]["id"], json!(rt_id));
        assert_eq!(routines::list_routines(&dir).len(), 2);

        // 3. Tampered prompt fails SHA-256 verification (400).
        let mut tampered = exp_json.clone();
        tampered["playbook"]["prompt"] = json!("Tampered instructions");
        let tampered_bytes = serde_json::to_vec(&tampered).unwrap();
        let bad_sha = send_req("POST", "/api/playbooks/import", Some(&tampered_bytes));
        assert!(bad_sha.starts_with("HTTP/1.1 400 Bad Request"), "{bad_sha}");
        assert!(bad_sha.contains("sha256 checksum mismatch"), "{bad_sha}");

        // 4. Unexpected schema key fails strict validation (400).
        let mut bad_schema = exp_json.clone();
        bad_schema["extra"] = json!(true);
        let bad_schema_bytes = serde_json::to_vec(&bad_schema).unwrap();
        let bad_field = send_req("POST", "/api/playbooks/import", Some(&bad_schema_bytes));
        assert!(bad_field.starts_with("HTTP/1.1 400 Bad Request"), "{bad_field}");

        // 5. Payload > 256 KB fails with 413 Content Too Large.
        let oversize = vec![b' '; routines::MAX_PLAYBOOK_PACK_BYTES + 1];
        let big = send_req("POST", "/api/playbooks/import", Some(&oversize));
        assert!(big.starts_with("HTTP/1.1 413 Content Too Large"), "{big}");

        // 6. Export missing playbook returns 404.
        let missing = send_req("GET", "/api/playbooks/rtn-missing/export", None);
        assert!(missing.starts_with("HTTP/1.1 404 Not Found"), "{missing}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn playbook_dry_run_preview_reports_steps_impacts_and_rollback_without_writes() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-preview-v021-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        let orig_csv = "vendor,amount\nAcme,120.50\n";
        std::fs::write(ws.join("ledger.csv"), orig_csv).unwrap();
        file_history::record_revision(&dir, &ws, "ledger.csv", "seed").unwrap();

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        std::fs::write(&cfg.memory_path, "- Role: Finance\n").unwrap();

        let created = routines::create_playbook_routine(
            &dir,
            "Month-End Close",
            "Clean ledger.csv with clean_dataset",
            "weekly mon",
            None,
            Some("reconcile_ledger"),
            true,
            &routines::PlaybookOptions {
                steps: vec![
                    "Clean ledger.csv with clean_dataset".to_string(),
                    "Draft executive close memo and export Word (.docx)".to_string(),
                ],
                expected_inputs: vec!["ledger.csv".to_string()],
                pack_deliverables: true,
            },
        )
        .unwrap();
        let rt_id = created["id"].as_str().unwrap().to_string();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_post = |target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "POST {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}",
                addr.port()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        for path in [
            format!("/api/playbooks/{rt_id}/preview"),
            format!("/api/routines/{rt_id}/preview"),
        ] {
            let resp = send_post(&path);
            assert!(resp.starts_with("HTTP/1.1 200 OK"), "{resp}");
            let body: Value = serde_json::from_str(resp.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            assert_eq!(body["dry_run"], json!(true));
            assert_eq!(body["zero_side_effects"], json!(true));
            assert_eq!(body["step_count"], json!(2));
            assert_eq!(body["steps"][0]["skill_id"], json!("clean_dataset"));
            assert_eq!(body["steps"][1]["skill_id"], json!("write_document"));
            assert_eq!(body["rollback"]["safe"], json!(true));
            assert_eq!(body["rollback"]["badge"], json!("Reversible via .history/"));
            let impacts = body["file_impacts"].as_array().unwrap();
            assert!(impacts.iter().any(|x| x["path"] == "ledger.csv" && x["exists_in_workspace"] == true && x["history_revisions"] == 1));
            assert!(impacts.iter().any(|x| x["path"] == "Playbook Deliverable Pack.md" && x["role"] == "deliverable_pack"));
        }

        // Confirm zero workspace mutations occurred.
        assert_eq!(std::fs::read_to_string(ws.join("ledger.csv")).unwrap(), orig_csv);
        assert!(!ws.join("Playbook Deliverable Pack.md").exists());
        assert!(send_post("/api/playbooks/rtn-missing/preview").starts_with("HTTP/1.1 404 Not Found"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn routine_catchup_and_sha256_verification_badge_work_end_to_end() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-catchup-v023-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();

        // Stub OpenAI-compatible loopback model returning a clean final answer.
        let model_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let model_addr = model_listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for mut s in model_listener.incoming().flatten() {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let payload = r#"{"id":"m1","choices":[{"message":{"role":"assistant","content":"{\"action\":\"done\",\"summary\":\"Caught up ledger check cleanly.\"}"}}]}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = s.write_all(resp.as_bytes());
            }
        });

        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = ws.clone();
        cfg.memory_path = dir.join("memory.md");
        cfg.model_base_url = format!("http://{model_addr}/v1");
        cfg.model_id = "stub-model".to_string();
        std::fs::write(&cfg.memory_path, "- Role: Finance\n").unwrap();

        // Seed an active routine whose created_ms was 35 minutes ago on an "every 15m" schedule -> 2 missed windows.
        let now_ms = records::now_millis();
        let old_ms = now_ms.saturating_sub(35 * 60 * 1000);
        let seeded = json!({
            "routines": [{
                "id": "rtn-missed-1",
                "name": "15m Ledger Watch",
                "prompt": "Check workspace status",
                "schedule": "every 15m",
                "interval_secs": 900,
                "enabled": true,
                "created_ms": old_ms,
                "last_run_ms": Value::Null,
                "run_count": 0
            }]
        });
        std::fs::write(routines::routines_path(&dir), seeded.to_string()).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_http = |method: &str, target: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let body = if method == "POST" { "{}" } else { "" };
            let req = format!(
                "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                addr.port(),
                body.len()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        // 1. Before catch-up: missed_windows >= 2, last_run_verified == null.
        let before = send_http("GET", "/api/routines");
        let before_json: Value = serde_json::from_str(before.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert!(before_json["routines"][0]["missed_windows"].as_u64().unwrap() >= 2, "{before_json}");
        assert!(before_json["scheduler"]["missed_windows_total"].as_u64().unwrap() >= 2);
        assert_eq!(before_json["routines"][0]["last_run_verified"], Value::Null);

        // 2. Run 1-click Catch Up -> clears missed windows and stamps verified SHA-256 receipt!
        let cu = send_http("POST", "/api/routines/rtn-missed-1/catchup");
        assert!(cu.starts_with("HTTP/1.1 200 OK"), "{cu}");
        let cu_json: Value = serde_json::from_str(cu.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(cu_json["caught_up"], json!(true));
        assert!(cu_json["missed_windows_cleared"].as_u64().unwrap() >= 2);
        assert_eq!(cu_json["routine"]["missed_windows"], json!(0));
        assert_eq!(cu_json["routine"]["last_run_verified"], json!("verified"));
        assert_eq!(
            cu_json["routine"]["last_run_receipt_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );

        // 3. Repeated catch-up reports caught_up: false, missed_windows: 0.
        let cu2 = send_http("POST", "/api/routines/rtn-missed-1/catchup");
        let cu2_json: Value = serde_json::from_str(cu2.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(cu2_json["caught_up"], json!(false));
        assert_eq!(cu2_json["missed_windows"], json!(0));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fast_json_writers_match_value_builders() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-fast-json-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = dir.join("workspace");
        cfg.memory_path = dir.join("memory.md");
        cfg.ensure_dirs().unwrap();
        std::fs::write(cfg.workspace_dir.join("notes\"q\\.md"), "alpha\nbeta\n\"quoted\"\n").unwrap();
        std::fs::write(&cfg.memory_path, "# Memory\n- remembered alpha item\n").unwrap();
        records::append_work(
            &cfg,
            "Analyze \"alpha\"\titem",
            "Line1\nLine2",
            &["notes\"q\\.md".to_string()],
            "t-fast-1",
            true,
        );

        let idx = search::SearchCache::new().get(
            "default",
            &cfg.data_dir,
            &cfg.workspace_dir,
            &cfg.memory_path,
        );

        // 1. write_files_json parity
        let (total_f, files_v) = idx.list_files(0, Some(10), "");
        let mut buf = Vec::new();
        idx.write_files_json(&mut buf, "default", 0, Some(10), "");
        let fast_files: Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(
            fast_files,
            json!({"person_id": "default", "total": total_f, "files": files_v})
        );

        // 2. write_receipts_json parity (unfiltered + filtered)
        for (q, st) in [("", "all"), ("alpha", "finished"), ("missing", "all")] {
            buf.clear();
            idx.write_receipts_json(&mut buf, 0, 50, q, st);
            let fast_r: Value = serde_json::from_slice(&buf).unwrap();
            let slow_r = idx.list_receipts(0, 50, q, st);
            assert_eq!(fast_r, slow_r, "receipt mismatch for q={q} status={st}");
        }

        // 3. write_search_json parity (compare everything except timing field elapsed_ms)
        for (q, sc) in [("", "all"), ("alpha", "all"), ("quoted", "files"), ("remembered", "memory")] {
            buf.clear();
            idx.write_search_json(&mut buf, q, sc, 30);
            let mut fast_s: Value = serde_json::from_slice(&buf).unwrap();
            let mut slow_s = idx.search(q, sc, 30);
            fast_s["elapsed_ms"] = json!(0);
            slow_s["elapsed_ms"] = json!(0);
            assert_eq!(fast_s, slow_s, "search mismatch for q={q} scope={sc}");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn memory_health_and_heal_work_end_to_end() {
        use std::io::{Read, Write};
        let dir = std::env::temp_dir().join(format!(
            "ergon-mem-health-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = dir.join("workspace");
        cfg.memory_path = dir.join("memory.md");
        cfg.ensure_dirs().unwrap();
        std::fs::write(cfg.workspace_dir.join("active.csv"), "a,b\n1,2\n").unwrap();
        std::fs::write(cfg.workspace_dir.join("Q1 budget.csv"), "item,amt\ncloud,100\n").unwrap();
        std::fs::write(
            &cfg.memory_path,
            "# Memory\n\n## About\n- Name: Cayleb\n- Role: Finance Lead\n\n## Learned\n- Always include executive bullet points.\n- Flag variances against [active.csv:L2] and [Q1 budget.csv:L1].\n- Flag missing rows in [missing.csv:L5].\n- Always include executive bullet points.\n- Never include executive bullet points.\n- Prefer CSV exports.\n- Prefer Excel exports.\n",
        )
        .unwrap();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg_srv = cfg.clone();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg_srv);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_http = |method: &str, target: &str, body: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                addr.port(),
                body.len()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        // 1. Audit should flag stale ref ([missing.csv:L5]), duplicate ("Always include..."), and conflicts (Always vs Never, Prefer CSV vs Excel).
        let before = send_http("GET", "/api/memory", "");
        assert!(before.starts_with("HTTP/1.1 200 OK"), "{before}");
        let bj: Value = serde_json::from_str(before.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        let h = &bj["health"];
        assert_eq!(h["status"], json!("issues"));
        assert!(h["conflicts"].as_u64().unwrap() >= 2, "{h}");
        assert!(h["duplicates"].as_u64().unwrap() >= 1, "{h}");
        assert!(h["stale_refs"].as_u64().unwrap() >= 1, "{h}");

        // 2. Heal all issues in 1 click -> removes stale/dup/superseded rules, keeps valid & newer rules.
        let heal = send_http("POST", "/api/memory/heal", r#"{"mode":"all"}"#);
        assert!(heal.starts_with("HTTP/1.1 200 OK"), "{heal}");
        let hj: Value = serde_json::from_str(heal.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert!(hj["removed_count"].as_u64().unwrap() >= 4, "{hj}");
        assert_eq!(hj["health"]["status"], json!("healthy"));
        assert_eq!(hj["health"]["issue_count"], json!(0));
        let content = hj["content"].as_str().unwrap();
        assert!(content.contains("- Name: Cayleb"));
        assert!(content.contains("- Flag variances against [active.csv:L2] and [Q1 budget.csv:L1]."));
        assert!(content.contains("- Never include executive bullet points."));
        assert!(content.contains("- Prefer Excel exports."));
        assert!(!content.contains("missing.csv"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn workspace_policy_profiles_and_enforcement_work_end_to_end() {
        use std::io::{Read, Write};
        let dir = std::env::temp_dir().join(format!(
            "ergon-policy-e2e-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut cfg = Config::from_env();
        cfg.data_dir = dir.clone();
        cfg.workspace_dir = dir.join("workspace");
        cfg.memory_path = dir.join("memory.md");
        cfg.ensure_dirs().unwrap();
        std::fs::write(cfg.workspace_dir.join("brief.md"), "# Client Brief\nLine 2 ok\n").unwrap();
        records::append_work(
            &cfg,
            "Prepare client brief from /home/alice/secrets/brief.md",
            "Verified [missing-source.md:L4] against /home/alice/internal/notes.txt",
            &["brief.md".to_string()],
            "t-pol-1",
            true,
        );

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg_srv = cfg.clone();
        std::thread::spawn(move || {
            let _ = serve(listener, cfg_srv);
        });
        std::thread::sleep(std::time::Duration::from_millis(50));

        let send_http = |method: &str, target: &str, body: &str| -> String {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            let req = format!(
                "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                addr.port(),
                body.len()
            );
            s.write_all(req.as_bytes()).unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).unwrap();
            out
        };

        // 1. Default policy is fast_internal.
        let p0 = send_http("GET", "/api/workspace/policy", "");
        assert!(p0.starts_with("HTTP/1.1 200 OK"), "{p0}");
        let j0: Value = serde_json::from_str(p0.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(j0["profile"], json!("fast_internal"));
        assert_eq!(j0["strict_citation_gate"], json!(false));

        // 2. Switch to strict_client via PUT -> blocks pack export (409) when stale/broken citations exist, and redacts host paths in audit bundle!
        let p1 = send_http("PUT", "/api/workspace/policy", r#"{"profile":"strict_client"}"#);
        assert!(p1.starts_with("HTTP/1.1 200 OK"), "{p1}");
        let j1: Value = serde_json::from_str(p1.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(j1["profile"], json!("strict_client"));
        assert_eq!(j1["strict_citation_gate"], json!(true));
        assert_eq!(j1["redact_host_paths"], json!(true));

        let pack_blocked = send_http("GET", "/api/files/pack?summary=1&format=md", "");
        assert!(pack_blocked.starts_with("HTTP/1.1 409"), "{pack_blocked}");
        let audit_md = send_http("GET", "/api/audit/bundle?format=md", "");
        assert!(audit_md.starts_with("HTTP/1.1 200 OK"), "{audit_md}");
        assert!(!audit_md.contains("/home/alice/"), "{audit_md}");
        assert!(audit_md.contains("~/secrets/brief.md"), "{audit_md}");

        // 3. Switch to air_gapped -> blocks outbound fetch_url tool!
        let p2 = send_http("PUT", "/api/workspace/policy", r#"{"profile":"air_gapped"}"#);
        assert!(p2.starts_with("HTTP/1.1 200 OK"), "{p2}");
        let fetch_err = execute_tool(&cfg, "fetch_url", &json!({"url": "https://example.com"})).unwrap_err();
        assert!(fetch_err.contains("Air-Gapped"), "{fetch_err}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
