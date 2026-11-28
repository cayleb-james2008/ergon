//! Auto-updating Plugin Catalog & Live Official MCP Registry Adapter.
//!
//! Combines:
//! 1. The embedded baseline catalog (`catalog/plugins.json`, 32 workplace apps)
//!    so Ergon starts offline immediately with zero network dependency.
//! 2. Remote curated catalog updates (`ergon_manifest` feeds, `0600` cached in
//!    `<root_data_dir>/plugin_registry_cache.json`).
//! 3. Live Official MCP Registry (`https://registry.modelcontextprotocol.io/v0.1/servers`)
//!    sync, search, safety filtering, and plain-English 1-click app install backed by
//!    `secrets.enc.json` (`0600`) and `/phone` Approve/Deny gating.

use crate::auth::url_encode_component;
use crate::tools::atomic_write_bytes_with_mode;
use crate::{mcp, parse_url_host_port, records, tripwire};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

const EMBEDDED_CATALOG_JSON: &str = include_str!("../catalog/plugins.json");
const SOURCES_FILE: &str = "plugin_registry_sources.json";
const CACHE_FILE: &str = "plugin_registry_cache.json";

pub const DEFAULT_CURATED_FEED_URL: &str =
    "https://raw.githubusercontent.com/cayleb-james2008/ergon/main/catalog/plugins.json";
pub const DEFAULT_MCP_REGISTRY_URL: &str =
    "https://registry.modelcontextprotocol.io/v0.1/servers?version=latest&limit=60";

const DEFAULT_SYNC_INTERVAL_SECS: u64 = 6 * 3600;
const MIN_SYNC_INTERVAL_SECS: u64 = 300;
const MAX_SYNC_INTERVAL_SECS: u64 = 7 * 86_400;
const MAX_FETCH_BYTES: usize = 2 * 1024 * 1024;
const MAX_COMMUNITY_CACHED: usize = 250;

static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

fn with_registry_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

#[derive(Clone, Debug)]
pub struct PluginActionSpec {
    pub name: String,
    pub label: String,
    pub description: String,
    pub access: String, // "read" | "write"
    pub requires_approval: bool,
}

impl PluginActionSpec {
    pub fn from_json(v: &Value) -> Option<Self> {
        let name = v["name"].as_str()?.trim().to_string();
        if name.is_empty() {
            return None;
        }
        let label = v["label"].as_str().unwrap_or(&name).trim().to_string();
        let description = v["description"].as_str().unwrap_or("").trim().to_string();
        let access = if v["access"].as_str() == Some("write") {
            "write".to_string()
        } else {
            "read".to_string()
        };
        let requires_approval = v["requires_approval"]
            .as_bool()
            .unwrap_or(access == "write");
        Some(Self {
            name,
            label,
            description,
            access,
            requires_approval,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "label": self.label,
            "description": self.description,
            "access": self.access,
            "requires_approval": self.requires_approval,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PluginConnectPreset {
    pub api_base_url: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_scopes: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_security: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub imap_security: Option<String>,
    pub match_hosts: Vec<String>,
    pub connector_tab: String,
}

impl PluginConnectPreset {
    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let opt_s = |k: &str| {
            v[k].as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        let match_hosts = v["match_hosts"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            api_base_url: opt_s("api_base_url"),
            oauth_token_url: opt_s("oauth_token_url"),
            oauth_scopes: opt_s("oauth_scopes"),
            smtp_host: opt_s("smtp_host"),
            smtp_port: v["smtp_port"].as_u64().and_then(|n| u16::try_from(n).ok()),
            smtp_security: opt_s("smtp_security"),
            imap_host: opt_s("imap_host"),
            imap_port: v["imap_port"].as_u64().and_then(|n| u16::try_from(n).ok()),
            imap_security: opt_s("imap_security"),
            match_hosts,
            connector_tab: opt_s("connector_tab").unwrap_or_else(|| "outbound".to_string()),
        })
    }

    pub fn to_public_json(&self) -> Value {
        json!({
            "api_base_url": self.api_base_url,
            "oauth_token_url": self.oauth_token_url,
            "oauth_scopes": self.oauth_scopes,
            "smtp_host": self.smtp_host,
            "smtp_port": self.smtp_port,
            "smtp_security": self.smtp_security,
            "imap_host": self.imap_host,
            "imap_port": self.imap_port,
            "imap_security": self.imap_security,
            "connector_tab": self.connector_tab,
        })
    }

    pub fn to_storage_json(&self) -> Value {
        json!({
            "api_base_url": self.api_base_url,
            "oauth_token_url": self.oauth_token_url,
            "oauth_scopes": self.oauth_scopes,
            "smtp_host": self.smtp_host,
            "smtp_port": self.smtp_port,
            "smtp_security": self.smtp_security,
            "imap_host": self.imap_host,
            "imap_port": self.imap_port,
            "imap_security": self.imap_security,
            "match_hosts": self.match_hosts,
            "connector_tab": self.connector_tab,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CredentialField {
    pub key: String,
    pub label: String,
    pub description: String,
    pub is_secret: bool,
    pub is_required: bool,
    pub kind: String, // "env" | "bearer"
}

impl CredentialField {
    pub fn from_json(v: &Value) -> Option<Self> {
        let key = sanitize_env_key(v["key"].as_str().unwrap_or(""))?;
        let label = v["label"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&key)
            .to_string();
        let description = v["description"].as_str().unwrap_or("").trim().to_string();
        let is_secret = v["is_secret"].as_bool().unwrap_or(true);
        let is_required = v["is_required"].as_bool().unwrap_or(true);
        let kind = if v["kind"].as_str() == Some("bearer") {
            "bearer".to_string()
        } else {
            "env".to_string()
        };
        Some(Self {
            key,
            label,
            description,
            is_secret,
            is_required,
            kind,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "key": self.key,
            "label": self.label,
            "description": self.description,
            "is_secret": self.is_secret,
            "is_required": self.is_required,
            "kind": self.kind,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PluginInstallSpec {
    pub transport: String, // "http_sse" | "stdio"
    pub command_or_url: String,
    pub args: Vec<String>,
    pub credential_fields: Vec<CredentialField>,
}

impl PluginInstallSpec {
    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let transport = match v["transport"].as_str()?.trim() {
            "http_sse" => "http_sse".to_string(),
            "stdio" => "stdio".to_string(),
            _ => return None,
        };
        let cmd = v["command_or_url"].as_str()?.trim().to_string();
        if cmd.is_empty() || cmd.chars().any(|c| c.is_control() || c == '\0') {
            return None;
        }
        if transport == "http_sse" {
            let low = cmd.to_ascii_lowercase();
            if !(low.starts_with("https://") || low.starts_with("http://")) {
                return None;
            }
            if cmd.contains('{') || cmd.contains('}') {
                return None;
            }
        } else if cmd.starts_with('-') || tripwire::sensitive_target(&cmd).is_some() {
            return None;
        }
        let args = v["args"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if transport == "stdio" {
            for a in &args {
                if a.chars().any(|c| c.is_control() || c == '\0')
                    || tripwire::sensitive_target(a).is_some()
                {
                    return None;
                }
            }
        }
        let credential_fields = v["credential_fields"]
            .as_array()
            .map(|arr| arr.iter().filter_map(CredentialField::from_json).collect())
            .unwrap_or_default();
        Some(Self {
            transport,
            command_or_url: cmd,
            args,
            credential_fields,
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "transport": self.transport,
            "command_or_url": self.command_or_url,
            "args": self.args,
            "credential_fields": self.credential_fields.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DynamicPluginSpec {
    pub id: String,
    pub suite: String,
    pub suite_label: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub mode: String,      // "local" | "cloud" | "hybrid"
    pub transport: String, // "local_builtin" | "smtp_imap_xoauth2" | "oauth2_rest" | "mcp"
    pub local_ready: bool,
    pub default_enabled: bool,
    pub version: String,
    pub source: String, // "builtin" | "curated_feed" | "community_registry"
    pub actions: Vec<PluginActionSpec>,
    pub connect_preset: Option<PluginConnectPreset>,
    pub install_spec: Option<PluginInstallSpec>,
    pub website_url: Option<String>,
    pub repository_url: Option<String>,
}

impl DynamicPluginSpec {
    pub fn from_json(v: &Value, default_source: &str) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        if id.is_empty() || id.len() > 96 {
            return None;
        }
        let name = v["name"]
            .as_str()?
            .replace(['\r', '\n'], " ")
            .trim()
            .chars()
            .take(80)
            .collect::<String>();
        if name.is_empty() {
            return None;
        }
        let suite = match v["suite"].as_str().unwrap_or("business").trim() {
            "google" => "google",
            "microsoft" => "microsoft",
            "local" => "local",
            "community" => "community",
            _ => "business",
        }
        .to_string();
        let suite_label = v["suite_label"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(match suite.as_str() {
                "google" => "Google Workspace",
                "microsoft" => "Microsoft 365 & Outlook",
                "local" => "On-Device Local",
                "community" => "App Directory",
                _ => "Business Apps",
            })
            .to_string();
        let tagline = v["tagline"]
            .as_str()
            .unwrap_or("")
            .replace(['\r', '\n'], " ")
            .trim()
            .chars()
            .take(120)
            .collect::<String>();
        let description = v["description"]
            .as_str()
            .unwrap_or("")
            .replace(['\r', '\n'], " ")
            .trim()
            .chars()
            .take(280)
            .collect::<String>();
        let mode = match v["mode"].as_str().unwrap_or("cloud").trim() {
            "local" => "local",
            "hybrid" => "hybrid",
            _ => "cloud",
        }
        .to_string();
        let transport = match v["transport"].as_str().unwrap_or("mcp").trim() {
            "local_builtin" => "local_builtin",
            "smtp_imap_xoauth2" => "smtp_imap_xoauth2",
            "oauth2_rest" => "oauth2_rest",
            _ => "mcp",
        }
        .to_string();
        let actions: Vec<PluginActionSpec> = v["actions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(PluginActionSpec::from_json).collect())
            .filter(|a: &Vec<_>| !a.is_empty())
            .unwrap_or_else(|| {
                vec![
                    PluginActionSpec {
                        name: "run_tool".to_string(),
                        label: "Inspect / read".to_string(),
                        description: format!("Run read-only actions via {name}."),
                        access: "read".to_string(),
                        requires_approval: false,
                    },
                    PluginActionSpec {
                        name: "write_action".to_string(),
                        label: "External write".to_string(),
                        description: format!("Execute external write in {name} (parks approval)."),
                        access: "write".to_string(),
                        requires_approval: true,
                    },
                ]
            });

        Some(Self {
            id,
            suite,
            suite_label,
            name,
            tagline: if tagline.is_empty() {
                description.chars().take(72).collect()
            } else {
                tagline
            },
            description,
            mode,
            transport,
            local_ready: v["local_ready"].as_bool().unwrap_or(false),
            default_enabled: v["default_enabled"].as_bool().unwrap_or(false),
            version: v["version"].as_str().unwrap_or("1.0.0").trim().to_string(),
            source: v["source"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(default_source)
                .to_string(),
            actions,
            connect_preset: PluginConnectPreset::from_json(&v["connect_preset"]),
            install_spec: PluginInstallSpec::from_json(&v["install_spec"]),
            website_url: safe_http_link(v["website_url"].as_str()),
            repository_url: safe_http_link(v["repository_url"].as_str()),
        })
    }

    pub fn to_storage_json(&self) -> Value {
        json!({
            "id": self.id,
            "suite": self.suite,
            "suite_label": self.suite_label,
            "name": self.name,
            "tagline": self.tagline,
            "description": self.description,
            "mode": self.mode,
            "transport": self.transport,
            "local_ready": self.local_ready,
            "default_enabled": self.default_enabled,
            "version": self.version,
            "source": self.source,
            "actions": self.actions.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
            "connect_preset": self.connect_preset.as_ref().map(|p| p.to_storage_json()),
            "install_spec": self.install_spec.as_ref().map(|i| i.to_json()),
            "website_url": self.website_url,
            "repository_url": self.repository_url,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RegistrySource {
    pub id: String,
    pub name: String,
    pub kind: String, // "ergon_manifest" | "mcp_registry_v01"
    pub url: String,
    pub enabled: bool,
    pub builtin: bool,
}

impl RegistrySource {
    fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let url = v["url"].as_str()?.trim().to_string();
        if id.is_empty() || url.is_empty() {
            return None;
        }
        let kind = match v["kind"].as_str().unwrap_or("mcp_registry_v01").trim() {
            "ergon_manifest" => "ergon_manifest".to_string(),
            "mcp_registry_v01" => "mcp_registry_v01".to_string(),
            _ => return None,
        };
        Some(Self {
            id,
            name: v["name"].as_str().unwrap_or("Plugin Feed").trim().to_string(),
            kind,
            url,
            enabled: v["enabled"].as_bool().unwrap_or(true),
            builtin: v["builtin"].as_bool().unwrap_or(false),
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "url": self.url,
            "enabled": self.enabled,
            "builtin": self.builtin,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RegistrySourceStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub enabled: bool,
    pub last_sync_ms: u64,
    pub item_count: usize,
    pub status: String, // "ok" | "error" | "idle"
    pub error: Option<String>,
}

impl RegistrySourceStatus {
    fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            id: v["id"].as_str()?.to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            kind: v["kind"].as_str().unwrap_or("").to_string(),
            url: v["url"].as_str().unwrap_or("").to_string(),
            enabled: v["enabled"].as_bool().unwrap_or(true),
            last_sync_ms: v["last_sync_ms"].as_u64().unwrap_or(0),
            item_count: v["item_count"].as_u64().unwrap_or(0) as usize,
            status: v["status"].as_str().unwrap_or("idle").to_string(),
            error: v["error"].as_str().map(str::to_string),
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "url": self.url,
            "enabled": self.enabled,
            "last_sync_ms": self.last_sync_ms,
            "item_count": self.item_count,
            "status": self.status,
            "error": self.error,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RegistryConfig {
    pub auto_update: String, // "auto" | "manual" | "off"
    pub sync_interval_secs: u64,
    pub sources: Vec<RegistrySource>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            auto_update: "auto".to_string(),
            sync_interval_secs: DEFAULT_SYNC_INTERVAL_SECS,
            sources: vec![
                RegistrySource {
                    id: "ergon-curated".to_string(),
                    name: "Ergon Curated Catalog".to_string(),
                    kind: "ergon_manifest".to_string(),
                    url: DEFAULT_CURATED_FEED_URL.to_string(),
                    enabled: true,
                    builtin: true,
                },
                RegistrySource {
                    id: "official-mcp-registry".to_string(),
                    name: "Official MCP Registry".to_string(),
                    kind: "mcp_registry_v01".to_string(),
                    url: DEFAULT_MCP_REGISTRY_URL.to_string(),
                    enabled: true,
                    builtin: true,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RegistryCache {
    pub synced_at_ms: u64,
    pub catalog_sha256: String,
    pub curated_overrides: Vec<DynamicPluginSpec>,
    pub community_plugins: Vec<DynamicPluginSpec>,
    pub source_status: Vec<RegistrySourceStatus>,
}

fn sources_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join(SOURCES_FILE)
}

fn cache_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join(CACHE_FILE)
}

fn safe_http_link(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.starts_with("https://") || s.starts_with("http://") {
        Some(s.to_string())
    } else {
        None
    }
}

fn sanitize_env_key(raw: &str) -> Option<String> {
    let key = raw.trim();
    if key.is_empty() || key.len() > 64 {
        return None;
    }
    if key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        Some(key.to_string())
    } else {
        None
    }
}

fn is_safe_package_identifier(raw: &str) -> bool {
    let s = raw.trim();
    !s.is_empty()
        && s.len() <= 140
        && !s.starts_with('-')
        && !s.contains("..")
        && !s.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || matches!(c, ';' | '&' | '|' | '`' | '$' | '<' | '>' | '\'' | '"' | '\\')
        })
}

fn humanize_registry_name(server_name: &str) -> String {
    let raw = server_name.rsplit('/').next().unwrap_or(server_name).trim();
    let trimmed = raw
        .strip_prefix("mcp-server-")
        .or_else(|| raw.strip_prefix("mcp-"))
        .or_else(|| raw.strip_suffix("-mcp"))
        .unwrap_or(raw);
    let words: Vec<String> = trimmed
        .split(['-', '_', '.'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut ch = w.chars();
            match ch.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + ch.as_str(),
            }
        })
        .collect();
    if words.is_empty() {
        server_name.to_string()
    } else {
        words.join(" ")
    }
}

pub fn embedded_baseline() -> Vec<DynamicPluginSpec> {
    let Ok(val) = serde_json::from_str::<Value>(EMBEDDED_CATALOG_JSON) else {
        return Vec::new();
    };
    val["plugins"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| DynamicPluginSpec::from_json(v, "builtin"))
                .collect()
        })
        .unwrap_or_default()
}

pub fn load_registry_config(root_data_dir: &Path) -> RegistryConfig {
    let mut cfg = with_registry_lock(|| load_registry_config_unlocked(root_data_dir));
    if let Ok(mode) = std::env::var("ERGON_PLUGIN_AUTO_UPDATE") {
        let m = mode.trim().to_ascii_lowercase();
        if matches!(m.as_str(), "auto" | "manual" | "off") {
            cfg.auto_update = m;
        }
    }
    if let Ok(secs) = std::env::var("ERGON_PLUGIN_SYNC_INTERVAL_SECS") {
        if let Ok(n) = secs.trim().parse::<u64>() {
            cfg.sync_interval_secs = n.clamp(MIN_SYNC_INTERVAL_SECS, MAX_SYNC_INTERVAL_SECS);
        }
    }
    if let Ok(u) = std::env::var("ERGON_PLUGIN_CATALOG_URL") {
        let trimmed = u.trim();
        if !trimmed.is_empty() {
            if let Some(s) = cfg.sources.iter_mut().find(|s| s.id == "ergon-curated") {
                s.url = trimmed.to_string();
            }
        }
    }
    if let Ok(u) = std::env::var("ERGON_MCP_REGISTRY_URL") {
        let trimmed = u.trim();
        if !trimmed.is_empty() {
            if let Some(s) = cfg.sources.iter_mut().find(|s| s.id == "official-mcp-registry") {
                s.url = trimmed.to_string();
            }
        }
    }
    cfg
}

fn load_registry_config_unlocked(root_data_dir: &Path) -> RegistryConfig {
    let path = sources_path(root_data_dir);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return RegistryConfig::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return RegistryConfig::default();
    };
    let auto_update = match v["auto_update"].as_str().unwrap_or("auto") {
        "manual" => "manual".to_string(),
        "off" => "off".to_string(),
        _ => "auto".to_string(),
    };
    let sync_interval_secs = v["sync_interval_secs"]
        .as_u64()
        .unwrap_or(DEFAULT_SYNC_INTERVAL_SECS)
        .clamp(MIN_SYNC_INTERVAL_SECS, MAX_SYNC_INTERVAL_SECS);
    let mut sources: Vec<RegistrySource> = v["sources"]
        .as_array()
        .map(|arr| arr.iter().filter_map(RegistrySource::from_json).collect())
        .unwrap_or_default();
    if sources.is_empty() {
        sources = RegistryConfig::default().sources;
    }
    RegistryConfig {
        auto_update,
        sync_interval_secs,
        sources,
    }
}

fn save_registry_config_unlocked(root_data_dir: &Path, cfg: &RegistryConfig) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&json!({
        "auto_update": cfg.auto_update,
        "sync_interval_secs": cfg.sync_interval_secs,
        "sources": cfg.sources.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
    }))
    .map_err(|e| format!("serialize registry config: {e}"))?;
    atomic_write_bytes_with_mode(&sources_path(root_data_dir), body.as_bytes(), Some(0o600))
        .map_err(|e| format!("write plugin_registry_sources.json: {e}"))
}

pub fn load_registry_cache(root_data_dir: &Path) -> RegistryCache {
    with_registry_lock(|| load_registry_cache_unlocked(root_data_dir))
}

fn load_registry_cache_unlocked(root_data_dir: &Path) -> RegistryCache {
    let path = cache_path(root_data_dir);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return RegistryCache::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return RegistryCache::default();
    };
    RegistryCache {
        synced_at_ms: v["synced_at_ms"].as_u64().unwrap_or(0),
        catalog_sha256: v["catalog_sha256"].as_str().unwrap_or("").to_string(),
        curated_overrides: v["curated_overrides"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| DynamicPluginSpec::from_json(x, "curated_feed"))
                    .collect()
            })
            .unwrap_or_default(),
        community_plugins: v["community_plugins"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| DynamicPluginSpec::from_json(x, "community_registry"))
                    .collect()
            })
            .unwrap_or_default(),
        source_status: v["source_status"]
            .as_array()
            .map(|arr| arr.iter().filter_map(RegistrySourceStatus::from_json).collect())
            .unwrap_or_default(),
    }
}

fn save_registry_cache_unlocked(root_data_dir: &Path, cache: &RegistryCache) -> Result<(), String> {
    let body = serde_json::to_string_pretty(&json!({
        "synced_at_ms": cache.synced_at_ms,
        "catalog_sha256": cache.catalog_sha256,
        "curated_overrides": cache.curated_overrides.iter().map(|p| p.to_storage_json()).collect::<Vec<_>>(),
        "community_plugins": cache.community_plugins.iter().map(|p| p.to_storage_json()).collect::<Vec<_>>(),
        "source_status": cache.source_status.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
    }))
    .map_err(|e| format!("serialize registry cache: {e}"))?;
    atomic_write_bytes_with_mode(&cache_path(root_data_dir), body.as_bytes(), Some(0o600))
        .map_err(|e| format!("write plugin_registry_cache.json: {e}"))
}

fn validate_feed_url(data_dir: &Path, url: &str, channel: &str) -> Result<(), String> {
    let trimmed = url.trim();
    let ok_scheme = trimmed.starts_with("https://")
        || (trimmed.starts_with("http://") && crate::is_loopback_url(trimmed));
    if !ok_scheme {
        return Err("Plugin registry URL must use https:// (or loopback http:// for local testing)".to_string());
    }
    tripwire::check_egress_for(data_dir, trimmed, channel)?;
    if let Some((host, port)) = parse_url_host_port(trimmed) {
        tripwire::check_egress_host_for(data_dir, &host, port, channel)?;
    }
    Ok(())
}

pub fn merged_catalog(root_data_dir: &Path) -> Vec<DynamicPluginSpec> {
    let baseline = embedded_baseline();
    let cache = load_registry_cache(root_data_dir);
    let mut by_id: BTreeMap<String, usize> = BTreeMap::new();
    let mut list: Vec<DynamicPluginSpec> = Vec::with_capacity(
        baseline.len() + cache.curated_overrides.len() + cache.community_plugins.len(),
    );

    for item in baseline {
        if !by_id.contains_key(&item.id) {
            by_id.insert(item.id.clone(), list.len());
            list.push(item);
        }
    }
    for item in cache.curated_overrides {
        if let Some(&idx) = by_id.get(&item.id) {
            list[idx] = item;
        } else {
            by_id.insert(item.id.clone(), list.len());
            list.push(item);
        }
    }
    for item in cache.community_plugins {
        if !by_id.contains_key(&item.id) {
            by_id.insert(item.id.clone(), list.len());
            list.push(item);
        }
    }
    list
}

pub fn find_in_catalog(catalog: &[DynamicPluginSpec], id: &str) -> Option<DynamicPluginSpec> {
    let key = id.trim();
    catalog.iter().find(|p| p.id == key).cloned()
}

pub fn registry_status_json(root_data_dir: &Path) -> Value {
    let cfg = load_registry_config(root_data_dir);
    let cache = load_registry_cache(root_data_dir);
    let catalog = merged_catalog(root_data_dir);
    let curated_count = catalog.iter().filter(|p| p.source != "community_registry").count();
    let community_count = catalog.iter().filter(|p| p.source == "community_registry").count();
    let sources_json: Vec<Value> = cfg
        .sources
        .iter()
        .map(|s| {
            let st = cache.source_status.iter().find(|st| st.id == s.id);
            json!({
                "id": s.id,
                "name": s.name,
                "kind": s.kind,
                "url": s.url,
                "enabled": s.enabled,
                "builtin": s.builtin,
                "last_sync_ms": st.map(|x| x.last_sync_ms).unwrap_or(0),
                "item_count": st.map(|x| x.item_count).unwrap_or(0),
                "status": st.map(|x| x.status.as_str()).unwrap_or("idle"),
                "error": st.and_then(|x| x.error.clone()),
            })
        })
        .collect();
    json!({
        "auto_update": cfg.auto_update,
        "sync_interval_secs": cfg.sync_interval_secs,
        "last_sync_ms": cache.synced_at_ms,
        "catalog_sha256": if cache.catalog_sha256.is_empty() { Value::Null } else { json!(cache.catalog_sha256) },
        "curated_count": curated_count,
        "community_count": community_count,
        "sources_count": cfg.sources.len(),
        "sources": sources_json,
    })
}

pub fn update_registry_config(
    root_data_dir: &Path,
    person_data_dir: &Path,
    body: &Value,
) -> Result<Value, (u16, String)> {
    let validation_dir = if person_data_dir.exists() {
        person_data_dir
    } else {
        root_data_dir
    };
    with_registry_lock(|| {
        let mut cfg = load_registry_config_unlocked(root_data_dir);
        if let Some(m) = body["auto_update"].as_str().map(str::trim) {
            if !matches!(m, "auto" | "manual" | "off") {
                return Err((400, "auto_update must be 'auto', 'manual', or 'off'".to_string()));
            }
            cfg.auto_update = m.to_string();
        }
        if let Some(secs) = body["sync_interval_secs"].as_u64() {
            cfg.sync_interval_secs = secs.clamp(MIN_SYNC_INTERVAL_SECS, MAX_SYNC_INTERVAL_SECS);
        }
        if let Some(arr) = body["sources"].as_array() {
            let mut next = Vec::new();
            for item in arr {
                let Some(src) = RegistrySource::from_json(item) else {
                    return Err((400, "each source requires id, kind, and url".to_string()));
                };
                validate_feed_url(validation_dir, &src.url, "connector_config")
                    .map_err(|e| (400, e))?;
                next.push(src);
            }
            if next.is_empty() {
                return Err((400, "at least one registry source is required".to_string()));
            }
            cfg.sources = next;
        }
        if let Some(add) = body["add_source"].as_object() {
            let url = add
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let name = add
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Custom Registry Feed")
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let kind = match add.get("kind").and_then(|v| v.as_str()).unwrap_or("ergon_manifest") {
                "ergon_manifest" => "ergon_manifest".to_string(),
                "mcp_registry_v01" => "mcp_registry_v01".to_string(),
                _ => {
                    return Err((
                        400,
                        "source kind must be 'ergon_manifest' or 'mcp_registry_v01'".to_string(),
                    ))
                }
            };
            validate_feed_url(validation_dir, &url, "connector_config").map_err(|e| (400, e))?;
            let id = format!("custom-{}", crate::slugify(&name));
            if let Some(ex) = cfg.sources.iter_mut().find(|s| s.id == id) {
                ex.url = url;
                ex.name = name;
                ex.kind = kind;
                ex.enabled = true;
            } else {
                cfg.sources.push(RegistrySource {
                    id,
                    name,
                    kind,
                    url,
                    enabled: true,
                    builtin: false,
                });
            }
        }
        if let Some(remove_id) = body["remove_source_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            cfg.sources.retain(|s| s.builtin || s.id != remove_id);
        }
        if let (Some(tid), Some(ten)) = (
            body["toggle_source_id"].as_str().map(str::trim),
            body["enabled"].as_bool(),
        ) {
            if let Some(s) = cfg.sources.iter_mut().find(|s| s.id == tid) {
                s.enabled = ten;
            }
        }
        save_registry_config_unlocked(root_data_dir, &cfg).map_err(|e| (500, e))?;
        Ok::<(), (u16, String)>(())
    })?;
    Ok(registry_status_json(root_data_dir))
}

fn fetch_capped_json(data_dir: &Path, url: &str) -> Result<(Value, String), String> {
    validate_feed_url(data_dir, url, "connector")?;
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(15))
        .build();
    let resp = agent
        .get(url)
        .set("Accept", "application/json")
        .set("User-Agent", concat!("ergon/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| format!("fetch {url}: {e}"))?;
    let status = resp.status();
    if status >= 400 {
        return Err(format!("HTTP {status} from {url}"));
    }
    let mut buf = Vec::new();
    let mut reader = resp.into_reader().take((MAX_FETCH_BYTES + 1) as u64);
    reader
        .read_to_end(&mut buf)
        .map_err(|e| format!("read response from {url}: {e}"))?;
    if buf.len() > MAX_FETCH_BYTES {
        return Err(format!("registry response exceeded {MAX_FETCH_BYTES} bytes cap"));
    }
    let sha = tripwire::sha256_hex(&buf);
    let val: Value = serde_json::from_slice(&buf).map_err(|e| format!("invalid JSON from {url}: {e}"))?;
    Ok((val, sha))
}

fn parse_ergon_manifest(val: &Value) -> Vec<DynamicPluginSpec> {
    val["plugins"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| DynamicPluginSpec::from_json(v, "curated_feed"))
                .collect()
        })
        .unwrap_or_default()
}

pub fn normalize_mcp_registry_server(entry: &Value, policy_dir: &Path) -> Option<DynamicPluginSpec> {
    let server = if entry["server"].is_object() {
        &entry["server"]
    } else {
        entry
    };
    let meta = &entry["_meta"]["io.modelcontextprotocol.registry/official"];
    if let Some(status) = meta["status"].as_str() {
        if status != "active" {
            return None;
        }
    }
    if meta["isLatest"].as_bool() == Some(false) {
        return None;
    }

    let raw_name = server["name"].as_str()?.trim();
    if raw_name.is_empty() {
        return None;
    }
    let slug = crate::slugify(raw_name);
    if slug.is_empty() {
        return None;
    }
    let id = format!("reg-{slug}");
    let title = server["title"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| humanize_registry_name(raw_name));
    let desc = server["description"]
        .as_str()
        .unwrap_or("")
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(260)
        .collect::<String>();
    let version = server["version"].as_str().unwrap_or("1.0.0").trim().to_string();

    // Build safe install_spec from either remotes[] or packages[]
    let mut install_spec: Option<PluginInstallSpec> = None;
    let mut connect_host: Option<String> = None;
    let mut mode = "cloud".to_string();

    if let Some(remotes) = server["remotes"].as_array() {
        for r in remotes {
            let rtype = r["type"].as_str().unwrap_or("");
            if !matches!(rtype, "streamable-http" | "sse") {
                continue;
            }
            let Some(u) = r["url"].as_str().map(str::trim) else {
                continue;
            };
            if u.contains('{') || u.contains('}') {
                continue;
            }
            if validate_feed_url(policy_dir, u, "connector_config").is_err() {
                continue;
            }
            let mut creds = Vec::new();
            if let Some(headers) = r["headers"].as_array() {
                for h in headers {
                    let hname = h["name"].as_str().unwrap_or("").trim();
                    let is_req = h["isRequired"].as_bool().unwrap_or(false);
                    if hname.eq_ignore_ascii_case("Authorization") || h["isSecret"].as_bool() == Some(true) {
                        let key = sanitize_env_key(&format!("{}_TOKEN", slug.replace('-', "_").to_ascii_uppercase()))
                            .unwrap_or_else(|| "API_TOKEN".to_string());
                        creds.push(CredentialField {
                            key,
                            label: format!("{title} Access Token"),
                            description: h["description"]
                                .as_str()
                                .unwrap_or("API key or bearer token stored encrypted in Ergon's vault.")
                                .trim()
                                .to_string(),
                            is_secret: true,
                            is_required: is_req,
                            kind: "bearer".to_string(),
                        });
                        break;
                    }
                }
            }
            connect_host = parse_url_host_port(u).map(|(h, _)| h);
            install_spec = Some(PluginInstallSpec {
                transport: "http_sse".to_string(),
                command_or_url: u.to_string(),
                args: Vec::new(),
                credential_fields: creds,
            });
            break;
        }
    }

    if install_spec.is_none() {
        if let Some(pkgs) = server["packages"].as_array() {
            for p in pkgs {
                let rtype = p["registryType"].as_str().unwrap_or("");
                let Some(ident) = p["identifier"].as_str().map(str::trim) else {
                    continue;
                };
                if !is_safe_package_identifier(ident) {
                    continue;
                }
                let (cmd, args) = match rtype {
                    "npm" => ("npx".to_string(), vec!["-y".to_string(), ident.to_string()]),
                    "pypi" => ("uvx".to_string(), vec![ident.to_string()]),
                    _ => continue,
                };
                let mut creds = Vec::new();
                if let Some(envs) = p["environmentVariables"].as_array() {
                    for ev in envs {
                        let Some(k) = sanitize_env_key(ev["name"].as_str().unwrap_or("")) else {
                            continue;
                        };
                        let is_req = ev["isRequired"].as_bool().unwrap_or(false);
                        let is_sec = ev["isSecret"].as_bool().unwrap_or(true);
                        let edesc = ev["description"].as_str().unwrap_or("").trim().to_string();
                        creds.push(CredentialField {
                            label: humanize_registry_name(&k),
                            key: k,
                            description: edesc,
                            is_secret: is_sec,
                            is_required: is_req,
                            kind: "env".to_string(),
                        });
                    }
                }
                mode = "hybrid".to_string();
                install_spec = Some(PluginInstallSpec {
                    transport: "stdio".to_string(),
                    command_or_url: cmd,
                    args,
                    credential_fields: creds,
                });
                break;
            }
        }
    }

    let install_spec = install_spec?;
    let tagline = if desc.is_empty() {
        format!("Community plugin · {raw_name}")
    } else {
        desc.chars().take(80).collect()
    };

    Some(DynamicPluginSpec {
        id,
        suite: "community".to_string(),
        suite_label: "App Directory".to_string(),
        name: title.clone(),
        tagline,
        description: if desc.is_empty() {
            format!("Auto-synced from Official MCP Registry ({raw_name}).")
        } else {
            desc
        },
        mode,
        transport: "mcp".to_string(),
        local_ready: false,
        default_enabled: false,
        version,
        source: "community_registry".to_string(),
        actions: vec![
            PluginActionSpec {
                name: "run_tool".to_string(),
                label: "Query / read".to_string(),
                description: format!("Inspect connected {title} tools and read context."),
                access: "read".to_string(),
                requires_approval: false,
            },
            PluginActionSpec {
                name: "write_action".to_string(),
                label: "Write / sync action".to_string(),
                description: format!("Run consequential writes in {title} (parks /phone approval)."),
                access: "write".to_string(),
                requires_approval: true,
            },
        ],
        connect_preset: Some(PluginConnectPreset {
            api_base_url: if install_spec.transport == "http_sse" {
                Some(install_spec.command_or_url.clone())
            } else {
                None
            },
            match_hosts: connect_host.into_iter().collect(),
            connector_tab: "mcp".to_string(),
            ..Default::default()
        }),
        install_spec: Some(install_spec),
        website_url: safe_http_link(server["websiteUrl"].as_str()),
        repository_url: safe_http_link(server["repository"]["url"].as_str()),
    })
}

fn parse_mcp_registry_feed(val: &Value, policy_dir: &Path) -> Vec<DynamicPluginSpec> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    if let Some(arr) = val["servers"].as_array() {
        for entry in arr {
            if let Some(spec) = normalize_mcp_registry_server(entry, policy_dir) {
                if seen.insert(spec.id.clone()) {
                    out.push(spec);
                    if out.len() >= MAX_COMMUNITY_CACHED {
                        break;
                    }
                }
            }
        }
    }
    out
}

pub fn sync_registry_now(
    root_data_dir: &Path,
    person_data_dir: &Path,
) -> Result<Value, (u16, String)> {
    let policy = crate::load_workspace_policy(person_data_dir);
    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err((
            403,
            "blocked by Air-Gapped workspace policy: plugin catalog auto-sync is disabled".to_string(),
        ));
    }
    let cfg = load_registry_config(root_data_dir);
    let enabled_sources: Vec<RegistrySource> =
        cfg.sources.into_iter().filter(|s| s.enabled).collect();
    if enabled_sources.is_empty() {
        return Err((400, "no plugin registry sources are enabled".to_string()));
    }

    let now = records::now_millis();
    let mut next_curated: Vec<DynamicPluginSpec> = Vec::new();
    let mut next_community: Vec<DynamicPluginSpec> = Vec::new();
    let mut statuses: Vec<RegistrySourceStatus> = Vec::new();
    let mut digest_inputs: Vec<String> = Vec::new();
    let mut any_ok = false;
    let mut last_err = String::new();

    for src in &enabled_sources {
        match fetch_capped_json(person_data_dir, &src.url) {
            Ok((val, sha)) => {
                any_ok = true;
                digest_inputs.push(sha);
                let count = if src.kind == "ergon_manifest" {
                    let parsed = parse_ergon_manifest(&val);
                    let n = parsed.len();
                    next_curated.extend(parsed);
                    n
                } else {
                    let parsed = parse_mcp_registry_feed(&val, person_data_dir);
                    let n = parsed.len();
                    next_community.extend(parsed);
                    n
                };
                statuses.push(RegistrySourceStatus {
                    id: src.id.clone(),
                    name: src.name.clone(),
                    kind: src.kind.clone(),
                    url: src.url.clone(),
                    enabled: true,
                    last_sync_ms: now,
                    item_count: count,
                    status: "ok".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                last_err = e.clone();
                statuses.push(RegistrySourceStatus {
                    id: src.id.clone(),
                    name: src.name.clone(),
                    kind: src.kind.clone(),
                    url: src.url.clone(),
                    enabled: true,
                    last_sync_ms: now,
                    item_count: 0,
                    status: "error".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    if !any_ok {
        return Err((502, format!("plugin registry sync failed: {last_err}")));
    }

    // Dedupe community list against curated/baseline IDs
    let baseline_ids: HashSet<String> = embedded_baseline().into_iter().map(|p| p.id).collect();
    let curated_ids: HashSet<String> = next_curated.iter().map(|p| p.id.clone()).collect();
    let mut seen_comm = HashSet::new();
    next_community.retain(|p| {
        !baseline_ids.contains(&p.id)
            && !curated_ids.contains(&p.id)
            && seen_comm.insert(p.id.clone())
    });
    next_community.truncate(MAX_COMMUNITY_CACHED);

    let combined_sha = tripwire::sha256_hex(digest_inputs.join(":").as_bytes());
    with_registry_lock(|| {
        let prev = load_registry_cache_unlocked(root_data_dir);
        let cache = RegistryCache {
            synced_at_ms: now,
            catalog_sha256: combined_sha,
            curated_overrides: if next_curated.is_empty() {
                prev.curated_overrides
            } else {
                next_curated
            },
            community_plugins: if next_community.is_empty() {
                prev.community_plugins
            } else {
                next_community
            },
            source_status: statuses,
        };
        save_registry_cache_unlocked(root_data_dir, &cache)
    })
    .map_err(|e| (500, e))?;

    Ok(registry_status_json(root_data_dir))
}

pub fn search_registry(
    root_data_dir: &Path,
    person_data_dir: &Path,
    query: &str,
) -> Value {
    let q = query.trim();
    let q_low = q.to_ascii_lowercase();
    let local = merged_catalog(root_data_dir);
    let mut results: Vec<DynamicPluginSpec> = local
        .into_iter()
        .filter(|p| {
            q_low.is_empty()
                || p.id.to_ascii_lowercase().contains(&q_low)
                || p.name.to_ascii_lowercase().contains(&q_low)
                || p.tagline.to_ascii_lowercase().contains(&q_low)
                || p.description.to_ascii_lowercase().contains(&q_low)
                || p.suite_label.to_ascii_lowercase().contains(&q_low)
        })
        .collect();

    let mut live_error: Option<String> = None;
    let policy = crate::load_workspace_policy(person_data_dir);
    if q.len() >= 2 && !(policy.block_outbound_fetch || policy.profile == "air_gapped") {
        let cfg = load_registry_config(root_data_dir);
        if let Some(mcp_src) = cfg
            .sources
            .iter()
            .find(|s| s.enabled && s.kind == "mcp_registry_v01")
        {
            let base = mcp_src.url.split('?').next().unwrap_or(&mcp_src.url);
            let search_url = format!(
                "{base}?search={}&version=latest&limit=20",
                url_encode_component(q)
            );
            match fetch_capped_json(person_data_dir, &search_url) {
                Ok((val, _)) => {
                    let mut seen: HashSet<String> = results.iter().map(|r| r.id.clone()).collect();
                    for item in parse_mcp_registry_feed(&val, person_data_dir) {
                        if seen.insert(item.id.clone()) {
                            results.push(item);
                        }
                    }
                }
                Err(e) => {
                    live_error = Some(e);
                }
            }
        }
    }

    let items: Vec<Value> = results.iter().map(|r| r.to_storage_json()).collect();
    json!({
        "query": q,
        "count": items.len(),
        "live_error": live_error,
        "results": items,
    })
}

pub fn build_mcp_server_from_plugin(
    person_data_dir: &Path,
    spec: &DynamicPluginSpec,
    credentials: &Value,
) -> Result<(String, Value), (u16, String)> {
    let Some(install) = spec.install_spec.as_ref() else {
        return Err((
            400,
            format!("{} does not define an auto-installable connector spec.", spec.name),
        ));
    };

    let mut env_secret_keys: Vec<String> = Vec::new();
    let mut auth_secret_key: Option<String> = None;
    let mut secrets: serde_json::Map<String, Value> = serde_json::Map::new();

    for field in &install.credential_fields {
        let raw_val = credentials
            .get(&field.key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if field.is_required && raw_val.is_empty() {
            return Err((
                400,
                format!("{} is required to connect {}", field.label, spec.name),
            ));
        }
        if !raw_val.is_empty() {
            secrets.insert(field.key.clone(), json!(raw_val));
        }
        if field.kind == "bearer" && auth_secret_key.is_none() {
            auth_secret_key = Some(field.key.clone());
        } else {
            env_secret_keys.push(field.key.clone());
        }
    }

    let mcp_id = format!("plugin-{}", crate::slugify(&spec.id));
    let payload = json!({
        "id": mcp_id,
        "name": spec.name,
        "transport": install.transport,
        "command_or_url": install.command_or_url,
        "args": install.args,
        "env_secret_keys": env_secret_keys,
        "auth_secret_key": auth_secret_key,
        "secrets": Value::Object(secrets),
        "require_approval_by_default": true,
        "enabled": true,
    });
    let policy = crate::load_workspace_policy(person_data_dir);
    let (server_json, _) = mcp::upsert_mcp_server(person_data_dir, &payload, &policy)?;
    let final_id = server_json["id"].as_str().unwrap_or(&mcp_id).to_string();
    Ok((final_id, server_json))
}

/// Start the daemon background auto-sync loop (only invoked when `background_updates == true`).
pub fn start_background_sync(root_data_dir: PathBuf) {
    let _ = std::thread::Builder::new()
        .name("ergon-plugin-registry-sync".to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(45));
            loop {
                let cfg = load_registry_config(&root_data_dir);
                let interval = cfg.sync_interval_secs.clamp(MIN_SYNC_INTERVAL_SECS, MAX_SYNC_INTERVAL_SECS);
                if cfg.auto_update == "auto" {
                    let policy = crate::load_workspace_policy(&root_data_dir);
                    if !(policy.block_outbound_fetch || policy.profile == "air_gapped") {
                        let _ = sync_registry_now(&root_data_dir, &root_data_dir);
                    }
                }
                std::thread::sleep(Duration::from_secs(interval));
            }
        });
}
