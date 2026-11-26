//! Unattended Intake Triggers & Policy-Gated Outbound Action Connectors.
//!
//! Persists non-secret connector metadata and delivery logs in
//! `<person_data_dir>/connectors.json` (0600 POSIX) and isolates all webhook
//! signing secrets, hook tokens, and outbound API secrets in a separate
//! `<person_data_dir>/connector_secrets.json` (0600 POSIX) that is never
//! readable from the workspace or injected into LLM prompts.

use crate::tools::{atomic_write_bytes, atomic_write_bytes_with_mode, sandbox_path, sandbox_path_resolving};
use crate::{records, tripwire, Config, WorkspacePolicy};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, UNIX_EPOCH};

pub const MAX_WEBHOOK_BYTES: usize = 1_048_576;
pub const MAX_FOLDER_WATCH_BYTES: u64 = 25 * 1024 * 1024;
pub const MAX_WEBHOOK_SKEW_MS: u64 = 5 * 60 * 1000;
const MAX_WEBHOOK_REPLAY_ENTRIES: usize = 4096;
const MAX_DELIVERY_LOG: usize = 500;
const MAX_INTAKE_MANIFEST: usize = 2000;
const MAX_RESPONSE_PREVIEW: usize = 512;

static CONN_SEQ: AtomicU64 = AtomicU64::new(0);
static CONNECTORS_LOCK: Mutex<()> = Mutex::new(());
static WEBHOOK_REPLAY_CACHE: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

fn with_connectors_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = CONNECTORS_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

fn next_seq() -> u64 {
    CONN_SEQ.fetch_add(1, Ordering::Relaxed)
}

pub fn connectors_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("connectors.json")
}

pub fn connector_secrets_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("connector_secrets.json")
}

#[derive(Clone, Debug)]
pub struct TriggerRule {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub enabled: bool,
    pub routine_id: Option<String>,
    pub target_kind: String,
    pub hook_id: Option<String>,
    pub watch_dir: Option<String>,
    pub glob: Option<String>,
    pub max_bytes: u64,
    pub trigger_count: u64,
    pub last_triggered_ms: Option<u64>,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl TriggerRule {
    pub fn to_json(&self, has_secret: bool) -> Value {
        let hook_path = self
            .hook_id
            .as_deref()
            .map(|h| format!("/api/hooks/{h}"));
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "enabled": self.enabled,
            "routine_id": self.routine_id,
            "target_kind": self.target_kind,
            "hook_id": self.hook_id,
            "hook_path": hook_path,
            "watch_dir": self.watch_dir,
            "glob": self.glob,
            "max_bytes": self.max_bytes,
            "has_secret": has_secret,
            "trigger_count": self.trigger_count,
            "last_triggered_ms": self.last_triggered_ms,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let name = v["name"].as_str().unwrap_or("Trigger").to_string();
        let kind = v["kind"].as_str()?.trim().to_string();
        if id.is_empty() || (kind != "webhook" && kind != "folder_watch") {
            return None;
        }
        Some(Self {
            id,
            name,
            kind,
            enabled: v["enabled"].as_bool().unwrap_or(true),
            routine_id: v["routine_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            target_kind: v["target_kind"]
                .as_str()
                .map(str::trim)
                .filter(|s| *s == "pipeline" || *s == "routine")
                .unwrap_or("routine")
                .to_string(),
            hook_id: v["hook_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            watch_dir: v["watch_dir"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            glob: v["glob"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            max_bytes: v["max_bytes"]
                .as_u64()
                .unwrap_or(MAX_FOLDER_WATCH_BYTES)
                .clamp(1, MAX_FOLDER_WATCH_BYTES),
            trigger_count: v["trigger_count"].as_u64().unwrap_or(0),
            last_triggered_ms: v["last_triggered_ms"].as_u64(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct OutboundConnector {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub url: String,
    pub method: String,
    pub headers_template: BTreeMap<String, String>,
    pub require_approval: bool,
    pub secret_keys: Vec<String>,
    pub enabled: bool,
    pub last_delivery_ms: Option<u64>,
    pub last_status: Option<String>,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl OutboundConnector {
    pub fn referenced_secret_keys(&self) -> Vec<String> {
        let mut keys = self.secret_keys.clone();
        for val in self.headers_template.values() {
            for k in extract_secret_placeholders(val) {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        keys
    }

    pub fn to_public_json(
        &self,
        secrets: &ConnectorSecretsStore,
        policy: &WorkspacePolicy,
    ) -> Value {
        let keys = self.referenced_secret_keys();
        let has_secret = !keys.is_empty() && keys.iter().all(|k| secrets.secrets.contains_key(k));
        let effective_require_approval =
            self.require_approval || policy.profile == "strict_client";
        let blocked_by_policy = policy.block_outbound_fetch || policy.profile == "air_gapped";
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "url": self.url,
            "method": self.method,
            "headers_template": self.headers_template,
            "require_approval": self.require_approval,
            "effective_require_approval": effective_require_approval,
            "blocked_by_policy": blocked_by_policy,
            "has_secret": has_secret,
            "secret_keys": keys,
            "enabled": self.enabled,
            "last_delivery_ms": self.last_delivery_ms,
            "last_status": self.last_status,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    fn to_stored_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "url": self.url,
            "method": self.method,
            "headers_template": self.headers_template,
            "require_approval": self.require_approval,
            "secret_keys": self.secret_keys,
            "enabled": self.enabled,
            "last_delivery_ms": self.last_delivery_ms,
            "last_status": self.last_status,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let name = v["name"].as_str().unwrap_or("Connector").to_string();
        let kind = v["kind"].as_str()?.trim().to_string();
        let url = v["url"].as_str()?.trim().to_string();
        if id.is_empty() || url.is_empty() {
            return None;
        }
        let mut headers_template = BTreeMap::new();
        if let Some(obj) = v["headers_template"].as_object() {
            for (k, val) in obj {
                if let Some(s) = val.as_str() {
                    headers_template.insert(k.clone(), s.to_string());
                }
            }
        }
        let secret_keys: Vec<String> = v["secret_keys"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            id,
            name,
            kind,
            url,
            method: v["method"]
                .as_str()
                .unwrap_or("POST")
                .trim()
                .to_ascii_uppercase(),
            headers_template,
            require_approval: v["require_approval"].as_bool().unwrap_or(true),
            secret_keys,
            enabled: v["enabled"].as_bool().unwrap_or(true),
            last_delivery_ms: v["last_delivery_ms"].as_u64(),
            last_status: v["last_status"].as_str().map(str::to_string),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DeliveryLog {
    pub id: String,
    pub connector_id: String,
    pub connector_name: String,
    pub kind: String,
    pub method: String,
    pub url: String,
    pub status: String,
    pub http_status: Option<u16>,
    pub decision_id: Option<String>,
    pub thread_id: Option<String>,
    pub payload_sha256: String,
    pub payload_bytes: usize,
    pub response_preview: String,
    pub error: Option<String>,
    pub ts_ms: u64,
}

impl DeliveryLog {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "connector_id": self.connector_id,
            "connector_name": self.connector_name,
            "kind": self.kind,
            "method": self.method,
            "url": self.url,
            "status": self.status,
            "http_status": self.http_status,
            "decision_id": self.decision_id,
            "thread_id": self.thread_id,
            "payload_sha256": self.payload_sha256,
            "payload_bytes": self.payload_bytes,
            "response_preview": self.response_preview,
            "error": self.error,
            "ts_ms": self.ts_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.to_string();
        let connector_id = v["connector_id"].as_str()?.to_string();
        Some(Self {
            id,
            connector_id,
            connector_name: v["connector_name"].as_str().unwrap_or("").to_string(),
            kind: v["kind"].as_str().unwrap_or("").to_string(),
            method: v["method"].as_str().unwrap_or("POST").to_string(),
            url: v["url"].as_str().unwrap_or("").to_string(),
            status: v["status"].as_str().unwrap_or("ok").to_string(),
            http_status: v["http_status"].as_u64().map(|n| n as u16),
            decision_id: v["decision_id"].as_str().map(str::to_string),
            thread_id: v["thread_id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            payload_sha256: v["payload_sha256"].as_str().unwrap_or("").to_string(),
            payload_bytes: v["payload_bytes"].as_u64().unwrap_or(0) as usize,
            response_preview: v["response_preview"].as_str().unwrap_or("").to_string(),
            error: v["error"].as_str().map(str::to_string),
            ts_ms: v["ts_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct IntakeManifestEntry {
    pub sha256: String,
    pub filename: String,
    pub source_path: String,
    pub trigger_id: String,
    pub ingested_ms: u64,
}

impl IntakeManifestEntry {
    fn to_json(&self) -> Value {
        json!({
            "sha256": self.sha256,
            "filename": self.filename,
            "source_path": self.source_path,
            "trigger_id": self.trigger_id,
            "ingested_ms": self.ingested_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            sha256: v["sha256"].as_str()?.to_string(),
            filename: v["filename"].as_str().unwrap_or("").to_string(),
            source_path: v["source_path"].as_str().unwrap_or("").to_string(),
            trigger_id: v["trigger_id"].as_str().unwrap_or("").to_string(),
            ingested_ms: v["ingested_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConnectorsStore {
    pub triggers: Vec<TriggerRule>,
    pub outbound: Vec<OutboundConnector>,
    pub delivery_log: Vec<DeliveryLog>,
    pub intake_manifest: Vec<IntakeManifestEntry>,
}

impl ConnectorsStore {
    fn load_unlocked(person_data_dir: &Path) -> Self {
        let path = connectors_path(person_data_dir);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            return Self::default();
        };
        Self {
            triggers: v["triggers"]
                .as_array()
                .map(|a| a.iter().filter_map(TriggerRule::from_json).collect())
                .unwrap_or_default(),
            outbound: v["outbound"]
                .as_array()
                .map(|a| a.iter().filter_map(OutboundConnector::from_json).collect())
                .unwrap_or_default(),
            delivery_log: v["delivery_log"]
                .as_array()
                .map(|a| a.iter().filter_map(DeliveryLog::from_json).collect())
                .unwrap_or_default(),
            intake_manifest: v["intake_manifest"]
                .as_array()
                .map(|a| a.iter().filter_map(IntakeManifestEntry::from_json).collect())
                .unwrap_or_default(),
        }
    }

    fn save_unlocked(&self, person_data_dir: &Path) -> Result<(), String> {
        let triggers_json: Vec<Value> = self
            .triggers
            .iter()
            .map(|t| {
                let mut v = t.to_json(t.kind == "webhook");
                if let Some(obj) = v.as_object_mut() {
                    obj.remove("has_secret");
                    obj.remove("hook_path");
                }
                v
            })
            .collect();
        let outbound_json: Vec<Value> = self.outbound.iter().map(|o| o.to_stored_json()).collect();
        let log_json: Vec<Value> = self.delivery_log.iter().map(|d| d.to_json()).collect();
        let manifest_json: Vec<Value> = self.intake_manifest.iter().map(|m| m.to_json()).collect();
        let doc = json!({
            "version": 1,
            "triggers": triggers_json,
            "outbound": outbound_json,
            "delivery_log": log_json,
            "intake_manifest": manifest_json,
        });
        let bytes = serde_json::to_vec_pretty(&doc).map_err(|e| format!("serialize connectors: {e}"))?;
        atomic_write_bytes_with_mode(&connectors_path(person_data_dir), &bytes, Some(0o600))
            .map_err(|e| format!("save connectors.json: {e}"))
    }

    fn push_delivery(&mut self, entry: DeliveryLog) {
        self.delivery_log.push(entry);
        if self.delivery_log.len() > MAX_DELIVERY_LOG {
            let drain = self.delivery_log.len() - MAX_DELIVERY_LOG;
            self.delivery_log.drain(0..drain);
        }
    }

    fn push_manifest(&mut self, entry: IntakeManifestEntry) {
        self.intake_manifest.push(entry);
        if self.intake_manifest.len() > MAX_INTAKE_MANIFEST {
            let drain = self.intake_manifest.len() - MAX_INTAKE_MANIFEST;
            self.intake_manifest.drain(0..drain);
        }
    }
}

#[derive(Clone, Debug)]
pub struct WebhookSecretEntry {
    pub signing_secret: String,
    pub hook_token: String,
    pub updated_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ConnectorSecretsStore {
    pub secrets: BTreeMap<String, String>,
    pub secret_updated_ms: BTreeMap<String, u64>,
    pub webhook_secrets: BTreeMap<String, WebhookSecretEntry>,
}

impl ConnectorSecretsStore {
    fn load_unlocked(person_data_dir: &Path) -> Self {
        let path = connector_secrets_path(person_data_dir);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            return Self::default();
        };
        let mut secrets = BTreeMap::new();
        if let Some(obj) = v["secrets"].as_object() {
            for (k, val) in obj {
                if let Some(s) = val.as_str() {
                    secrets.insert(k.clone(), s.to_string());
                }
            }
        }
        let mut secret_updated_ms = BTreeMap::new();
        if let Some(obj) = v["secret_updated_ms"].as_object() {
            for (k, val) in obj {
                if let Some(ts) = val.as_u64() {
                    secret_updated_ms.insert(k.clone(), ts);
                }
            }
        }
        let mut webhook_secrets = BTreeMap::new();
        if let Some(obj) = v["webhook_secrets"].as_object() {
            for (k, val) in obj {
                if let (Some(sig), Some(tok)) = (
                    val["signing_secret"].as_str(),
                    val["hook_token"].as_str(),
                ) {
                    webhook_secrets.insert(
                        k.clone(),
                        WebhookSecretEntry {
                            signing_secret: sig.to_string(),
                            hook_token: tok.to_string(),
                            updated_ms: val["updated_ms"].as_u64().unwrap_or(0),
                        },
                    );
                }
            }
        }
        Self {
            secrets,
            secret_updated_ms,
            webhook_secrets,
        }
    }

    fn save_unlocked(&self, person_data_dir: &Path) -> Result<(), String> {
        let wh: BTreeMap<String, Value> = self
            .webhook_secrets
            .iter()
            .map(|(k, e)| {
                (
                    k.clone(),
                    json!({
                        "signing_secret": e.signing_secret,
                        "hook_token": e.hook_token,
                        "updated_ms": e.updated_ms,
                    }),
                )
            })
            .collect();
        let doc = json!({
            "version": 1,
            "secrets": self.secrets,
            "secret_updated_ms": self.secret_updated_ms,
            "webhook_secrets": wh,
        });
        let bytes = serde_json::to_vec_pretty(&doc)
            .map_err(|e| format!("serialize connector secrets: {e}"))?;
        atomic_write_bytes_with_mode(&connector_secrets_path(person_data_dir), &bytes, Some(0o600))
            .map_err(|e| format!("save connector_secrets.json: {e}"))
    }

    fn all_values(&self) -> Vec<String> {
        let mut out = Vec::new();
        for v in self.secrets.values() {
            if v.trim().len() >= 6 && !out.contains(v) {
                out.push(v.clone());
            }
        }
        for w in self.webhook_secrets.values() {
            if w.signing_secret.trim().len() >= 6 && !out.contains(&w.signing_secret) {
                out.push(w.signing_secret.clone());
            }
            if w.hook_token.trim().len() >= 6 && !out.contains(&w.hook_token) {
                out.push(w.hook_token.clone());
            }
        }
        out
    }
}

/// Collect all raw connector secret values for arming active-run tripwire checks
/// and redaction lists. Never returned to LLM prompts or HTTP responses.
pub fn collect_all_secret_values(person_data_dir: &Path) -> Vec<String> {
    with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(person_data_dir).all_values())
}

/// Redact any known connector secret values (including partial and JSON-escaped
/// echoes) from `text`.
pub fn redact_with_store(text: &str, secrets: &ConnectorSecretsStore) -> String {
    let mut out = tripwire::redact(text);
    for val in secrets.all_values() {
        tripwire::redact_secret_value(&mut out, &val, false);
    }
    out
}

pub fn redact_for_person(person_data_dir: &Path, text: &str) -> String {
    let secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(person_data_dir));
    redact_with_store(text, &secrets)
}

fn extract_secret_placeholders(tmpl: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = tmpl;
    while let Some(start) = rest.find("{{secret:") {
        let after = &rest[start + 9..];
        if let Some(end) = after.find("}}") {
            let key = after[..end].trim();
            if is_valid_secret_key(key) && !keys.iter().any(|k| k == key) {
                keys.push(key.to_string());
            }
            rest = &after[end + 2..];
        } else {
            break;
        }
    }
    keys
}

fn is_valid_secret_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

fn mint_high_entropy_hex(bytes_len: usize) -> String {
    tripwire::hex(&tripwire::random_bytes(bytes_len))
}

fn mint_webhook_secrets(now_ms: u64) -> WebhookSecretEntry {
    WebhookSecretEntry {
        signing_secret: format!("whsec_{}", mint_high_entropy_hex(24)),
        hook_token: format!("whtok_{}", mint_high_entropy_hex(24)),
        updated_ms: now_ms,
    }
}

pub fn list_connectors_json(person_data_dir: &Path, policy: &WorkspacePolicy) -> Value {
    with_connectors_lock(|| {
        let store = ConnectorsStore::load_unlocked(person_data_dir);
        let secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        let triggers: Vec<Value> = store
            .triggers
            .iter()
            .map(|t| {
                let has_sec = if t.kind == "webhook" {
                    secrets.webhook_secrets.contains_key(&t.id)
                } else {
                    false
                };
                t.to_json(has_sec)
            })
            .collect();
        let outbound: Vec<Value> = store
            .outbound
            .iter()
            .map(|o| o.to_public_json(&secrets, policy))
            .collect();
        let masked_secrets: Vec<Value> = secrets
            .secrets
            .keys()
            .map(|k| {
                json!({
                    "key": k,
                    "has_secret": true,
                    "updated_ms": secrets.secret_updated_ms.get(k).copied().unwrap_or(0),
                })
            })
            .collect();
        let delivery_log: Vec<Value> = store
            .delivery_log
            .iter()
            .rev()
            .map(|d| d.to_json())
            .collect();
        json!({
            "triggers": triggers,
            "outbound": outbound,
            "secrets": masked_secrets,
            "delivery_log": delivery_log,
            "policy_profile": policy.profile,
        })
    })
}

/// Validate a host directory path for `folder_watch`. Rejects sensitive paths
/// via `tripwire::sensitive_target`, system directories, and Ergon's own data dirs.
pub fn validate_watch_dir(
    person_data_dir: &Path,
    root_data_dir: &Path,
    raw_dir: &str,
) -> Result<PathBuf, String> {
    let trimmed = raw_dir.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err("watch_dir is required and cannot contain null bytes".to_string());
    }
    if let Some(label) = tripwire::sensitive_target(trimmed) {
        return Err(format!(
            "watch_dir rejected because it targets sensitive path ({label})"
        ));
    }
    let p = PathBuf::from(trimmed);
    if !p.is_absolute() {
        return Err("watch_dir must be an absolute directory path".to_string());
    }
    // Walk every component from root to reject `..` traversal and any
    // intermediate or final symlink (CWE-59).
    let mut walked = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::RootDir => walked.push("/"),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) => {
                return Err("watch_dir cannot contain '..' or drive prefixes".to_string());
            }
            Component::Normal(part) => {
                walked.push(part);
                let md = std::fs::symlink_metadata(&walked)
                    .map_err(|e| format!("watch_dir does not exist or cannot be read: {e}"))?;
                if md.file_type().is_symlink() {
                    return Err("watch_dir cannot be or traverse a symlink".to_string());
                }
            }
        }
    }
    let meta = std::fs::symlink_metadata(&p)
        .map_err(|e| format!("watch_dir does not exist or cannot be read: {e}"))?;
    if meta.file_type().is_symlink() {
        return Err("watch_dir cannot be a symlink".to_string());
    }
    if !meta.is_dir() {
        return Err("watch_dir must be a directory".to_string());
    }
    let canon = p
        .canonicalize()
        .map_err(|e| format!("cannot canonicalize watch_dir: {e}"))?;
    let canon_str = canon.to_string_lossy();
    if let Some(label) = tripwire::sensitive_target(&canon_str) {
        return Err(format!(
            "watch_dir rejected because it resolves to sensitive path ({label})"
        ));
    }
    const FORBIDDEN_ROOTS: &[&str] = &[
        "/",
        "/etc",
        "/proc",
        "/sys",
        "/dev",
        "/bin",
        "/sbin",
        "/lib",
        "/lib64",
        "/usr",
        "/boot",
        "/root",
        "/run",
        "/var/run",
        "/home",
        "/Users",
    ];
    for &f in FORBIDDEN_ROOTS {
        let fp = Path::new(f);
        if canon == fp || (f != "/" && f != "/home" && f != "/Users" && canon.starts_with(fp)) {
            return Err(format!("watch_dir cannot be inside protected system path {f}"));
        }
    }
    for own_dir in [person_data_dir, root_data_dir] {
        if let Ok(canon_own) = own_dir.canonicalize() {
            if canon.starts_with(&canon_own) || canon_own.starts_with(&canon) {
                return Err(
                    "watch_dir cannot overlap Ergon's own internal data directory".to_string(),
                );
            }
        } else if canon.starts_with(own_dir) || own_dir.starts_with(&canon) {
            return Err("watch_dir cannot overlap Ergon's own internal data directory".to_string());
        }
    }
    Ok(canon)
}

pub fn create_trigger(
    person_data_dir: &Path,
    root_data_dir: &Path,
    body: &Value,
) -> Result<Value, (u16, String)> {
    if !body.is_object() {
        return Err((400, "request body must be a JSON object".to_string()));
    }
    if body["rotate"].as_bool() == Some(true) {
        if let Some(id) = body["id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            return rotate_webhook_secret(person_data_dir, id);
        }
    }
    let kind = body["kind"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if kind != "webhook" && kind != "folder_watch" {
        return Err((
            400,
            "trigger kind must be 'webhook' or 'folder_watch'".to_string(),
        ));
    }
    let name: String = body["name"]
        .as_str()
        .unwrap_or(if kind == "webhook" {
            "Webhook Trigger"
        } else {
            "Folder Watch Trigger"
        })
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    if name.is_empty() {
        return Err((400, "trigger name cannot be empty".to_string()));
    }
    let enabled = body["enabled"].as_bool().unwrap_or(true);
    let routine_id = body["routine_id"]
        .as_str()
        .or_else(|| body["pipeline_id"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Some(rid) = &routine_id {
        if !is_valid_id(rid) {
            return Err((400, "invalid routine_id".to_string()));
        }
    }
    let target_kind = match body["target_kind"].as_str().map(str::trim) {
        Some("pipeline") => "pipeline".to_string(),
        Some("routine") | None | Some("") => {
            if body.get("pipeline_id").is_some() && body.get("routine_id").is_none() {
                "pipeline".to_string()
            } else {
                "routine".to_string()
            }
        }
        Some(other) => {
            return Err((
                400,
                format!("target_kind must be 'routine' or 'pipeline', got '{other}'"),
            ))
        }
    };

    let now_ms = records::now_millis();
    let id = format!("trg-{now_ms}-{}", next_seq());

    if kind == "webhook" {
        let hook_id = format!("hk_{}", mint_high_entropy_hex(16));
        let wh_secret = mint_webhook_secrets(now_ms);
        let rule = TriggerRule {
            id: id.clone(),
            name,
            kind: "webhook".to_string(),
            enabled,
            routine_id,
            target_kind,
            hook_id: Some(hook_id.clone()),
            watch_dir: None,
            glob: None,
            max_bytes: MAX_WEBHOOK_BYTES as u64,
            trigger_count: 0,
            last_triggered_ms: None,
            created_ms: now_ms,
            updated_ms: now_ms,
        };
        with_connectors_lock(|| {
            let mut store = ConnectorsStore::load_unlocked(person_data_dir);
            let mut secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
            secrets
                .webhook_secrets
                .insert(id.clone(), wh_secret.clone());
            secrets
                .save_unlocked(person_data_dir)
                .map_err(|e| (500, e))?;
            store.triggers.push(rule.clone());
            store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
            Ok(json!({
                "trigger": rule.to_json(true),
                "signing_secret": wh_secret.signing_secret,
                "hook_token": wh_secret.hook_token,
                "hook_path": format!("/api/hooks/{hook_id}"),
            }))
        })
    } else {
        let raw_dir = body["watch_dir"]
            .as_str()
            .ok_or_else(|| (400, "folder_watch trigger requires watch_dir".to_string()))?;
        let canon_dir = validate_watch_dir(person_data_dir, root_data_dir, raw_dir)
            .map_err(|e| (400, e))?;
        let glob = body["glob"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("*")
            .chars()
            .take(256)
            .collect::<String>();
        let max_bytes = body["max_bytes"]
            .as_u64()
            .unwrap_or(MAX_FOLDER_WATCH_BYTES)
            .clamp(1, MAX_FOLDER_WATCH_BYTES);
        let rule = TriggerRule {
            id,
            name,
            kind: "folder_watch".to_string(),
            enabled,
            routine_id,
            target_kind,
            hook_id: None,
            watch_dir: Some(canon_dir.to_string_lossy().to_string()),
            glob: Some(glob),
            max_bytes,
            trigger_count: 0,
            last_triggered_ms: None,
            created_ms: now_ms,
            updated_ms: now_ms,
        };
        with_connectors_lock(|| {
            let mut store = ConnectorsStore::load_unlocked(person_data_dir);
            store.triggers.push(rule.clone());
            store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
            Ok(json!({
                "trigger": rule.to_json(false),
            }))
        })
    }
}

pub fn rotate_webhook_secret(
    person_data_dir: &Path,
    trigger_id: &str,
) -> Result<Value, (u16, String)> {
    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        let Some(trg) = store.triggers.iter_mut().find(|t| t.id == trigger_id) else {
            return Err((404, "trigger not found".to_string()));
        };
        if trg.kind != "webhook" {
            return Err((
                400,
                "only webhook triggers have signing secrets to rotate".to_string(),
            ));
        }
        let now_ms = records::now_millis();
        let new_wh = mint_webhook_secrets(now_ms);
        trg.updated_ms = now_ms;
        let trg_json = trg.to_json(true);
        let hook_path = trg
            .hook_id
            .as_deref()
            .map(|h| format!("/api/hooks/{h}"))
            .unwrap_or_default();
        let mut secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        secrets
            .webhook_secrets
            .insert(trigger_id.to_string(), new_wh.clone());
        secrets
            .save_unlocked(person_data_dir)
            .map_err(|e| (500, e))?;
        store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
        Ok(json!({
            "trigger": trg_json,
            "signing_secret": new_wh.signing_secret,
            "hook_token": new_wh.hook_token,
            "hook_path": hook_path,
        }))
    })
}

pub fn delete_trigger(person_data_dir: &Path, trigger_id: &str) -> Result<bool, String> {
    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        let before = store.triggers.len();
        store.triggers.retain(|t| t.id != trigger_id);
        if store.triggers.len() == before {
            return Ok(false);
        }
        let mut secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        if secrets.webhook_secrets.remove(trigger_id).is_some() {
            secrets.save_unlocked(person_data_dir)?;
        }
        store.save_unlocked(person_data_dir)?;
        Ok(true)
    })
}

pub fn upsert_outbound_connector(
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
        return Err((400, "connector name is required".to_string()));
    }
    let kind = body["kind"]
        .as_str()
        .unwrap_or("http_webhook")
        .trim()
        .to_ascii_lowercase();
    if !matches!(kind.as_str(), "http_webhook" | "rest_api" | "smtp_relay") {
        return Err((
            400,
            "connector kind must be 'http_webhook', 'rest_api', or 'smtp_relay'".to_string(),
        ));
    }
    let url = body["url"].as_str().unwrap_or("").trim().to_string();
    if url.is_empty() || url.contains("{{secret:") {
        return Err((
            400,
            "connector url must be a valid http:// or https:// URL without secret placeholders"
                .to_string(),
        ));
    }
    tripwire::check_egress(&url, "connector_config").map_err(|e| (400, e))?;

    let method = body["method"]
        .as_str()
        .unwrap_or("POST")
        .trim()
        .to_ascii_uppercase();
    if method != "POST" && method != "PUT" {
        return Err((400, "connector method must be POST or PUT".to_string()));
    }
    let mut headers_template = BTreeMap::new();
    if let Some(obj) = body["headers_template"].as_object() {
        for (k, v) in obj {
            let hk = k.trim();
            let hv = v.as_str().unwrap_or("");
            if hk.is_empty()
                || hk.contains('\r')
                || hk.contains('\n')
                || hv.contains('\r')
                || hv.contains('\n')
            {
                return Err((400, format!("invalid header template '{hk}'")));
            }
            headers_template.insert(hk.to_string(), hv.to_string());
        }
    }

    let now_ms = records::now_millis();
    let requested_id = body["id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(rid) = requested_id {
        if !is_valid_id(rid) {
            return Err((400, "invalid connector id".to_string()));
        }
    }

    let mut new_secrets: BTreeMap<String, String> = BTreeMap::new();
    if let Some(sec_obj) = body["secrets"].as_object() {
        for (k, v) in sec_obj {
            let sk = k.trim();
            if !is_valid_secret_key(sk) {
                return Err((400, format!("invalid secret key '{sk}'")));
            }
            let sv = v
                .as_str()
                .ok_or_else(|| (400, format!("secret '{sk}' must be a string")))?;
            if sv.contains('\r') || sv.contains('\n') {
                return Err((400, format!("secret '{sk}' cannot contain newlines")));
            }
            if !sv.is_empty() {
                new_secrets.insert(sk.to_string(), sv.to_string());
            }
        }
    }
    if let (Some(sk), Some(sv)) = (
        body["secret_key"].as_str().map(str::trim).filter(|s| !s.is_empty()),
        body["secret_value"].as_str(),
    ) {
        if !is_valid_secret_key(sk) {
            return Err((400, format!("invalid secret_key '{sk}'")));
        }
        if sv.contains('\r') || sv.contains('\n') {
            return Err((400, "secret_value cannot contain newlines".to_string()));
        }
        if !sv.is_empty() {
            new_secrets.insert(sk.to_string(), sv.to_string());
        }
    }

    let require_approval = body["require_approval"].as_bool().unwrap_or(true);
    let enabled = body["enabled"].as_bool().unwrap_or(true);

    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        let mut secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        if !new_secrets.is_empty() {
            for (k, v) in &new_secrets {
                secrets.secrets.insert(k.clone(), v.clone());
                secrets.secret_updated_ms.insert(k.clone(), now_ms);
            }
            secrets
                .save_unlocked(person_data_dir)
                .map_err(|e| (500, e))?;
        }
        let mut secret_keys: Vec<String> = Vec::new();
        for val in headers_template.values() {
            for k in extract_secret_placeholders(val) {
                if !secret_keys.contains(&k) {
                    secret_keys.push(k);
                }
            }
        }
        for k in new_secrets.keys() {
            if !secret_keys.contains(k) {
                secret_keys.push(k.clone());
            }
        }

        let existing_idx = requested_id
            .and_then(|rid| store.outbound.iter().position(|c| c.id == rid));
        let (conn, created) = if let Some(idx) = existing_idx {
            let mut updated = store.outbound[idx].clone();
            updated.name = name;
            updated.kind = kind;
            updated.url = url;
            updated.method = method;
            updated.headers_template = headers_template;
            updated.require_approval = require_approval;
            for k in secret_keys {
                if !updated.secret_keys.contains(&k) {
                    updated.secret_keys.push(k);
                }
            }
            updated.enabled = enabled;
            updated.updated_ms = now_ms;
            store.outbound[idx] = updated.clone();
            (updated, false)
        } else {
            let id = requested_id
                .map(str::to_string)
                .unwrap_or_else(|| format!("conn-{now_ms}-{}", next_seq()));
            let conn = OutboundConnector {
                id,
                name,
                kind,
                url,
                method,
                headers_template,
                require_approval,
                secret_keys,
                enabled,
                last_delivery_ms: None,
                last_status: None,
                created_ms: now_ms,
                updated_ms: now_ms,
            };
            store.outbound.push(conn.clone());
            (conn, true)
        };
        store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
        Ok((conn.to_public_json(&secrets, policy), created))
    })
}

pub fn delete_outbound_connector(person_data_dir: &Path, id: &str) -> Result<bool, String> {
    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        let before = store.outbound.len();
        store.outbound.retain(|c| c.id != id);
        if store.outbound.len() == before {
            return Ok(false);
        }
        store.save_unlocked(person_data_dir)?;
        Ok(true)
    })
}

#[derive(Clone, Debug)]
pub struct WebhookIngestResult {
    pub trigger_id: String,
    pub trigger_name: String,
    pub routine_id: Option<String>,
    pub target_kind: String,
    pub trigger_file: String,
    pub received_ms: u64,
}

fn decode_hex_32(s: &str) -> Option<[u8; 32]> {
    let trimmed = s.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = trimmed.as_bytes();
    for i in 0..32 {
        let hi = hex_nibble(bytes[i * 2])?;
        let lo = hex_nibble(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn parse_webhook_timestamp_ms(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let val = trimmed.parse::<u64>().ok()?;
    if val < 10_000_000_000 {
        val.checked_mul(1000)
    } else {
        Some(val)
    }
}

fn check_and_record_webhook_replay(replay_key: String, now_ms: u64) -> bool {
    let cache = WEBHOOK_REPLAY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock().unwrap_or_else(|p| p.into_inner());
    map.retain(|_, ts| now_ms.saturating_sub(*ts) <= MAX_WEBHOOK_SKEW_MS);
    if map.contains_key(&replay_key) {
        return false;
    }
    if map.len() >= MAX_WEBHOOK_REPLAY_ENTRIES {
        map.clear();
    }
    map.insert(replay_key, now_ms);
    true
}

/// Authenticate an incoming `POST /api/hooks/:hook_id` via HMAC-SHA256
/// (`X-Ergon-Signature: sha256=<hex>`) or `X-Ergon-Hook-Token`, write payload to
/// `workspace/inbox/hook-<ts>.json`, and return trigger metadata so the server
/// can fire the linked routine/pipeline with `{ trigger_file }`.
pub fn verify_and_ingest_webhook(
    cfg: &Config,
    hook_id: &str,
    signature_hdr: Option<&str>,
    token_hdr: Option<&str>,
    content_type: &str,
    body: &[u8],
) -> Result<WebhookIngestResult, (u16, String)> {
    verify_and_ingest_webhook_with_meta(
        cfg,
        hook_id,
        signature_hdr,
        token_hdr,
        None,
        None,
        content_type,
        body,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn verify_and_ingest_webhook_with_meta(
    cfg: &Config,
    hook_id: &str,
    signature_hdr: Option<&str>,
    token_hdr: Option<&str>,
    timestamp_hdr: Option<&str>,
    delivery_hdr: Option<&str>,
    content_type: &str,
    body: &[u8],
) -> Result<WebhookIngestResult, (u16, String)> {
    if body.len() > MAX_WEBHOOK_BYTES {
        return Err((
            413,
            format!("webhook payload exceeds {MAX_WEBHOOK_BYTES} byte limit"),
        ));
    }
    let hook_clean = hook_id.trim();
    if hook_clean.is_empty() {
        return Err((401, "invalid or missing webhook authentication".to_string()));
    }

    let now_ms = records::now_millis();
    let ts_raw = timestamp_hdr.map(str::trim).filter(|s| !s.is_empty());
    if let Some(ts_str) = ts_raw {
        let Some(ts_ms) = parse_webhook_timestamp_ms(ts_str) else {
            return Err((401, "invalid webhook timestamp header".to_string()));
        };
        if now_ms.abs_diff(ts_ms) > MAX_WEBHOOK_SKEW_MS {
            return Err((
                401,
                "webhook timestamp outside allowed replay window".to_string(),
            ));
        }
    }

    let (trigger_id, trigger_name, routine_id, target_kind, wh_secret) =
        with_connectors_lock(|| {
            let store = ConnectorsStore::load_unlocked(&cfg.data_dir);
            let secrets = ConnectorSecretsStore::load_unlocked(&cfg.data_dir);
            let Some(trg) = store
                .triggers
                .iter()
                .find(|t| t.kind == "webhook" && t.enabled && t.hook_id.as_deref() == Some(hook_clean))
            else {
                return Err((401, "invalid or missing webhook authentication".to_string()));
            };
            let Some(sec) = secrets.webhook_secrets.get(&trg.id).cloned() else {
                return Err((401, "invalid or missing webhook authentication".to_string()));
            };
            Ok((
                trg.id.clone(),
                trg.name.clone(),
                trg.routine_id.clone(),
                trg.target_kind.clone(),
                sec,
            ))
        })?;

    let mut authed = false;
    if let Some(sig) = signature_hdr.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(hex_part) = sig
            .strip_prefix("sha256=")
            .or_else(|| sig.strip_prefix("SHA256="))
        {
            if let Some(presented) = decode_hex_32(hex_part) {
                let expected_body =
                    tripwire::hmac_sha256(wh_secret.signing_secret.as_bytes(), body);
                let mut sig_ok = tripwire::ct_eq(&presented, &expected_body);
                if !sig_ok {
                    if let Some(ts_str) = ts_raw {
                        let mut signed_msg = Vec::with_capacity(ts_str.len() + 1 + body.len());
                        signed_msg.extend_from_slice(ts_str.as_bytes());
                        signed_msg.push(b'.');
                        signed_msg.extend_from_slice(body);
                        let expected_ts = tripwire::hmac_sha256(
                            wh_secret.signing_secret.as_bytes(),
                            &signed_msg,
                        );
                        sig_ok = tripwire::ct_eq(&presented, &expected_ts);
                    }
                }
                if sig_ok {
                    authed = true;
                }
            }
        }
    }
    if !authed {
        if let Some(tok) = token_hdr.map(str::trim).filter(|s| !s.is_empty()) {
            let h_tok = tripwire::sha256(tok.as_bytes());
            let h_exp = tripwire::sha256(wh_secret.hook_token.as_bytes());
            if tripwire::ct_eq(&h_tok, &h_exp) {
                authed = true;
            }
        }
    }
    if !authed {
        return Err((401, "invalid or missing webhook authentication".to_string()));
    }

    let delivery_clean = delivery_hdr.map(str::trim).filter(|s| !s.is_empty());
    if let Some(did) = delivery_clean {
        let key = format!("del:{trigger_id}:{did}");
        if !check_and_record_webhook_replay(key, now_ms) {
            return Err((401, "replayed webhook delivery rejected".to_string()));
        }
    } else if let Some(ts_str) = ts_raw {
        let body_hash = tripwire::sha256_hex(body);
        let key = format!("ts:{trigger_id}:{ts_str}:{body_hash}");
        if !check_and_record_webhook_replay(key, now_ms) {
            return Err((401, "replayed webhook delivery rejected".to_string()));
        }
    }
    let seq = next_seq();
    let rel_path = format!("inbox/hook-{now_ms}-{seq}.json");
    let full = sandbox_path(&cfg.workspace_dir, &rel_path).map_err(|e| (500, e))?;
    let payload_val: Value = serde_json::from_slice::<Value>(body)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body).to_string()));
    let envelope = json!({
        "trigger_id": trigger_id,
        "trigger_name": trigger_name,
        "hook_id": hook_clean,
        "received_ms": now_ms,
        "content_type": content_type,
        "payload": payload_val,
    });
    let envelope_bytes =
        serde_json::to_vec_pretty(&envelope).map_err(|e| (500, format!("serialize hook: {e}")))?;
    atomic_write_bytes(&full, &envelope_bytes)
        .map_err(|e| (500, format!("write webhook inbox file: {e}")))?;

    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(&cfg.data_dir);
        if let Some(trg) = store.triggers.iter_mut().find(|t| t.id == trigger_id) {
            trg.trigger_count = trg.trigger_count.saturating_add(1);
            trg.last_triggered_ms = Some(now_ms);
            trg.updated_ms = now_ms;
        }
        let _ = store.save_unlocked(&cfg.data_dir);
    });

    Ok(WebhookIngestResult {
        trigger_id,
        trigger_name,
        routine_id,
        target_kind,
        trigger_file: rel_path,
        received_ms: now_ms,
    })
}

#[derive(Clone, Debug)]
struct FileObservation {
    size: u64,
    mtime_nanos: u128,
    seen_count: u8,
}

#[derive(Default)]
pub struct FolderWatchPollState {
    observations: HashMap<(String, PathBuf), FileObservation>,
}

#[derive(Clone, Debug)]
pub struct FolderIngestEvent {
    pub trigger_id: String,
    pub trigger_name: String,
    pub routine_id: Option<String>,
    pub target_kind: String,
    pub trigger_file: String,
    pub sha256: String,
    pub bytes: usize,
}

fn glob_match_one(pat: &str, name: &str) -> bool {
    let p: Vec<char> = pat.to_ascii_lowercase().chars().collect();
    let s: Vec<char> = name.to_ascii_lowercase().chars().collect();
    let mut prev = vec![false; s.len() + 1];
    let mut cur = vec![false; s.len() + 1];
    prev[0] = true;
    for pc in p {
        cur.fill(false);
        if pc == '*' {
            cur[0] = prev[0];
            for j in 1..=s.len() {
                cur[j] = prev[j] || cur[j - 1];
            }
        } else {
            for j in 1..=s.len() {
                if pc == '?' || pc == s[j - 1] {
                    cur[j] = prev[j - 1];
                }
            }
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[s.len()]
}

fn read_watched_file_no_symlink(path: &Path, max_bytes: u64) -> Option<Vec<u8>> {
    let lmeta = std::fs::symlink_metadata(path).ok()?;
    if lmeta.file_type().is_symlink() || !lmeta.is_file() || lmeta.len() > max_bytes {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let fmeta = file.metadata().ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        if lmeta.dev() != fmeta.dev() || lmeta.ino() != fmeta.ino() {
            return None;
        }
    }
    if !fmeta.is_file() || fmeta.len() > max_bytes {
        return None;
    }
    let mut bytes = Vec::with_capacity(fmeta.len().min(65_536) as usize);
    if file.take(max_bytes + 1).read_to_end(&mut bytes).is_err() {
        return None;
    }
    if bytes.len() as u64 > max_bytes {
        return None;
    }
    Some(bytes)
}

pub fn glob_matches(pattern_list: &str, filename: &str) -> bool {
    let patterns: Vec<&str> = pattern_list
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|p| glob_match_one(p, filename))
}

fn sanitize_intake_filename(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_start_matches('.');
    if trimmed.is_empty() {
        format!("intake-{}.bin", next_seq())
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn disambiguate_filename(filename: &str, sha_hex: &str) -> String {
    let short = &sha_hex[..sha_hex.len().min(8)];
    if let Some((stem, ext)) = filename.rsplit_once('.') {
        if !stem.is_empty() && !ext.is_empty() {
            return format!("{stem}-{short}.{ext}");
        }
    }
    format!("{filename}-{short}")
}

/// Poll all enabled `folder_watch` triggers once for one person.
/// Files are ingested only when `(size, mtime)` is stable across 2 consecutive
/// polls, <= `max_bytes` (capped at 25 MB), matches the glob filter, and is not
/// already present in the SHA-256 `intake_manifest`.
pub fn poll_folder_watch_once(
    cfg: &Config,
    root_data_dir: &Path,
    poll_state: &mut FolderWatchPollState,
) -> Vec<FolderIngestEvent> {
    let triggers: Vec<TriggerRule> = with_connectors_lock(|| {
        ConnectorsStore::load_unlocked(&cfg.data_dir)
            .triggers
            .into_iter()
            .filter(|t| t.kind == "folder_watch" && t.enabled)
            .collect()
    });
    let mut events = Vec::new();
    let mut active_keys: HashSet<(String, PathBuf)> = HashSet::new();

    for trg in triggers {
        let Some(raw_dir) = trg.watch_dir.as_deref() else {
            continue;
        };
        let Ok(canon_dir) = validate_watch_dir(&cfg.data_dir, root_data_dir, raw_dir) else {
            continue;
        };
        let Ok(entries) = std::fs::read_dir(&canon_dir) else {
            continue;
        };
        let glob = trg.glob.as_deref().unwrap_or("*");
        let max_bytes = trg.max_bytes.min(MAX_FOLDER_WATCH_BYTES);

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(fname_os) = path.file_name() else {
                continue;
            };
            let fname = fname_os.to_string_lossy().to_string();
            if fname.starts_with('.') || !glob_matches(glob, &fname) {
                continue;
            }
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() || !meta.is_file() {
                continue;
            }
            let size = meta.len();
            if size > max_bytes {
                continue;
            }
            let mtime_nanos = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);

            let key = (trg.id.clone(), path.clone());
            active_keys.insert(key.clone());

            let stable = match poll_state.observations.get_mut(&key) {
                Some(obs) if obs.size == size && obs.mtime_nanos == mtime_nanos => {
                    obs.seen_count = obs.seen_count.saturating_add(1);
                    obs.seen_count == 2
                }
                _ => {
                    poll_state.observations.insert(
                        key,
                        FileObservation {
                            size,
                            mtime_nanos,
                            seen_count: 1,
                        },
                    );
                    false
                }
            };
            if !stable {
                continue;
            }

            let Some(bytes) = read_watched_file_no_symlink(&path, max_bytes) else {
                continue;
            };
            let sha = tripwire::sha256_hex(&bytes);
            let now_ms = records::now_millis();
            let safe_name = sanitize_intake_filename(&fname);

            let maybe_ingested: Result<Option<String>, String> = with_connectors_lock(|| {
                let mut store = ConnectorsStore::load_unlocked(&cfg.data_dir);
                if store.intake_manifest.iter().any(|m| m.sha256 == sha) {
                    return Ok(None);
                }
                let mut rel = format!("intake/{safe_name}");
                let mut dest = sandbox_path_resolving(&cfg.workspace_dir, &rel, false)?;
                if dest
                    .symlink_metadata()
                    .is_ok_and(|md| md.file_type().is_symlink())
                    || dest.exists()
                {
                    rel = format!("intake/{}", disambiguate_filename(&safe_name, &sha));
                    dest = sandbox_path_resolving(&cfg.workspace_dir, &rel, false)?;
                    if dest
                        .symlink_metadata()
                        .is_ok_and(|md| md.file_type().is_symlink())
                    {
                        return Err("cannot overwrite symlink in intake".to_string());
                    }
                }
                sandbox_path(&cfg.workspace_dir, &rel)?;
                atomic_write_bytes(&dest, &bytes)
                    .map_err(|e| format!("copy intake file: {e}"))?;
                store.push_manifest(IntakeManifestEntry {
                    sha256: sha.clone(),
                    filename: safe_name.clone(),
                    source_path: path.to_string_lossy().to_string(),
                    trigger_id: trg.id.clone(),
                    ingested_ms: now_ms,
                });
                if let Some(t) = store.triggers.iter_mut().find(|t| t.id == trg.id) {
                    t.trigger_count = t.trigger_count.saturating_add(1);
                    t.last_triggered_ms = Some(now_ms);
                    t.updated_ms = now_ms;
                }
                store.save_unlocked(&cfg.data_dir)?;
                Ok(Some(rel))
            });

            if let Ok(Some(trigger_file)) = maybe_ingested {
                events.push(FolderIngestEvent {
                    trigger_id: trg.id.clone(),
                    trigger_name: trg.name.clone(),
                    routine_id: trg.routine_id.clone(),
                    target_kind: trg.target_kind.clone(),
                    trigger_file,
                    sha256: sha,
                    bytes: bytes.len(),
                });
            }
        }
    }

    poll_state
        .observations
        .retain(|k, _| active_keys.contains(k));
    events
}

#[derive(Clone, Debug)]
pub enum InvokeOutcome {
    Delivered(DeliveryLog),
    Held {
        decision_id: String,
        delivery: DeliveryLog,
    },
}

fn substitute_header_secrets(
    headers_template: &BTreeMap<String, String>,
    secrets: &ConnectorSecretsStore,
) -> Result<Vec<(String, String)>, String> {
    let mut resolved = Vec::with_capacity(headers_template.len());
    for (k, tmpl) in headers_template {
        let mut out = String::with_capacity(tmpl.len());
        let mut rest = tmpl.as_str();
        while let Some(start) = rest.find("{{secret:") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 9..];
            let Some(end) = after.find("}}") else {
                return Err(format!("unclosed secret placeholder in header '{k}'"));
            };
            let key = after[..end].trim();
            let Some(val) = secrets.secrets.get(key) else {
                return Err(format!("missing connector secret '{key}'"));
            };
            out.push_str(val);
            rest = &after[end + 2..];
        }
        out.push_str(rest);
        if out.contains('\r') || out.contains('\n') {
            return Err(format!("resolved header '{k}' contains invalid newline"));
        }
        resolved.push((k.clone(), out));
    }
    Ok(resolved)
}

fn build_delivery_payload_bytes(payload: &Value) -> Vec<u8> {
    if let Some(s) = payload.as_str() {
        if let Ok(parsed) = serde_json::from_str::<Value>(s) {
            if parsed.is_object() || parsed.is_array() {
                return serde_json::to_vec(&parsed).unwrap_or_else(|_| s.as_bytes().to_vec());
            }
        }
        serde_json::to_vec(&json!({ "message": s })).unwrap_or_else(|_| s.as_bytes().to_vec())
    } else if payload.is_null() {
        b"{}".to_vec()
    } else {
        serde_json::to_vec(payload).unwrap_or_else(|_| b"{}".to_vec())
    }
}

fn execute_http_delivery(
    cfg: &Config,
    connector: &OutboundConnector,
    payload: &Value,
    thread_id: &str,
    decision_id: Option<&str>,
) -> Result<DeliveryLog, String> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    let payload_bytes = build_delivery_payload_bytes(payload);
    let payload_sha256 = tripwire::sha256_hex(&payload_bytes);
    let now_ms = records::now_millis();
    let del_id = format!("del-{now_ms}-{}", next_seq());
    let secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    let safe_url = redact_with_store(&connector.url, &secrets);

    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        let err_msg = "blocked by Air-Gapped workspace policy: outbound connector delivery is disabled for this workspace".to_string();
        let log = DeliveryLog {
            id: del_id,
            connector_id: connector.id.clone(),
            connector_name: connector.name.clone(),
            kind: connector.kind.clone(),
            method: connector.method.clone(),
            url: safe_url,
            status: "blocked".to_string(),
            http_status: None,
            decision_id: decision_id.map(str::to_string),
            thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
            payload_sha256,
            payload_bytes: payload_bytes.len(),
            response_preview: String::new(),
            error: Some(err_msg.clone()),
            ts_ms: now_ms,
        };
        record_delivery_log(&cfg.data_dir, &connector.id, log);
        return Err(err_msg);
    }

    if let Err(e) = tripwire::check_egress(&connector.url, "connector")
        .and_then(|_| {
            let p_str = String::from_utf8_lossy(&payload_bytes);
            tripwire::check_egress_payload(&p_str, "connector")
        })
    {
        let safe_err = redact_with_store(&e, &secrets);
        let log = DeliveryLog {
            id: del_id,
            connector_id: connector.id.clone(),
            connector_name: connector.name.clone(),
            kind: connector.kind.clone(),
            method: connector.method.clone(),
            url: safe_url,
            status: "blocked".to_string(),
            http_status: None,
            decision_id: decision_id.map(str::to_string),
            thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
            payload_sha256,
            payload_bytes: payload_bytes.len(),
            response_preview: String::new(),
            error: Some(safe_err.clone()),
            ts_ms: now_ms,
        };
        record_delivery_log(&cfg.data_dir, &connector.id, log);
        return Err(safe_err);
    }

    // Substitute {{secret:key}} strictly inside Rust host code AFTER tripwire checks.
    let headers = match substitute_header_secrets(&connector.headers_template, &secrets) {
        Ok(h) => h,
        Err(e) => {
            let safe_err = redact_with_store(&e, &secrets);
            let log = DeliveryLog {
                id: del_id,
                connector_id: connector.id.clone(),
                connector_name: connector.name.clone(),
                kind: connector.kind.clone(),
                method: connector.method.clone(),
                url: safe_url,
                status: "error".to_string(),
                http_status: None,
                decision_id: decision_id.map(str::to_string),
                thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
                payload_sha256,
                payload_bytes: payload_bytes.len(),
                response_preview: String::new(),
                error: Some(safe_err.clone()),
                ts_ms: now_ms,
            };
            record_delivery_log(&cfg.data_dir, &connector.id, log);
            return Err(safe_err);
        }
    };

    let agent = tripwire::guarded_agent(Duration::from_secs(15));
    let mut req = if connector.method == "PUT" {
        agent.put(&connector.url)
    } else {
        agent.post(&connector.url)
    };
    let has_ct = headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
    if !has_ct {
        req = req.set("Content-Type", "application/json");
    }
    for (k, v) in &headers {
        req = req.set(k, v);
    }

    let send_res = req.send_bytes(&payload_bytes);
    match send_res {
        Ok(resp) => {
            let status_code = resp.status();
            let mut body_buf = Vec::new();
            let _ = resp.into_reader().take(4096).read_to_end(&mut body_buf);
            let raw_preview = String::from_utf8_lossy(&body_buf).to_string();
            let preview = crate::truncate(
                &redact_with_store(&raw_preview, &secrets),
                MAX_RESPONSE_PREVIEW,
            );
            let log = DeliveryLog {
                id: del_id,
                connector_id: connector.id.clone(),
                connector_name: connector.name.clone(),
                kind: connector.kind.clone(),
                method: connector.method.clone(),
                url: safe_url,
                status: "ok".to_string(),
                http_status: Some(status_code),
                decision_id: decision_id.map(str::to_string),
                thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
                payload_sha256,
                payload_bytes: payload_bytes.len(),
                response_preview: preview,
                error: None,
                ts_ms: now_ms,
            };
            record_delivery_log(&cfg.data_dir, &connector.id, log.clone());
            Ok(log)
        }
        Err(ureq::Error::Status(code, resp)) => {
            let mut body_buf = Vec::new();
            let _ = resp.into_reader().take(2048).read_to_end(&mut body_buf);
            let raw_preview = String::from_utf8_lossy(&body_buf).to_string();
            let preview = crate::truncate(
                &redact_with_store(&raw_preview, &secrets),
                MAX_RESPONSE_PREVIEW,
            );
            let err_msg = format!("connector returned HTTP {code}: {preview}");
            let log = DeliveryLog {
                id: del_id,
                connector_id: connector.id.clone(),
                connector_name: connector.name.clone(),
                kind: connector.kind.clone(),
                method: connector.method.clone(),
                url: safe_url,
                status: "error".to_string(),
                http_status: Some(code),
                decision_id: decision_id.map(str::to_string),
                thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
                payload_sha256,
                payload_bytes: payload_bytes.len(),
                response_preview: preview,
                error: Some(err_msg.clone()),
                ts_ms: now_ms,
            };
            record_delivery_log(&cfg.data_dir, &connector.id, log);
            Err(err_msg)
        }
        Err(e) => {
            let msg = tripwire::explain_fetch_error(&connector.url, &e);
            let safe_err = redact_with_store(&msg, &secrets);
            let log = DeliveryLog {
                id: del_id,
                connector_id: connector.id.clone(),
                connector_name: connector.name.clone(),
                kind: connector.kind.clone(),
                method: connector.method.clone(),
                url: safe_url,
                status: "error".to_string(),
                http_status: None,
                decision_id: decision_id.map(str::to_string),
                thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
                payload_sha256,
                payload_bytes: payload_bytes.len(),
                response_preview: String::new(),
                error: Some(safe_err.clone()),
                ts_ms: now_ms,
            };
            record_delivery_log(&cfg.data_dir, &connector.id, log);
            Err(safe_err)
        }
    }
}

fn record_delivery_log(person_data_dir: &Path, connector_id: &str, log: DeliveryLog) {
    with_connectors_lock(|| {
        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        if let Some(c) = store.outbound.iter_mut().find(|c| c.id == connector_id) {
            c.last_delivery_ms = Some(log.ts_ms);
            c.last_status = Some(log.status.clone());
            c.updated_ms = log.ts_ms;
        }
        store.push_delivery(log);
        let _ = store.save_unlocked(person_data_dir);
    });
}

fn load_connector(person_data_dir: &Path, connector_id: &str) -> Result<OutboundConnector, String> {
    with_connectors_lock(|| {
        ConnectorsStore::load_unlocked(person_data_dir)
            .outbound
            .into_iter()
            .find(|c| c.id == connector_id)
            .ok_or_else(|| format!("unknown outbound connector '{connector_id}'"))
    })
}

/// Execute or park an outbound connector delivery according to workspace policy.
pub fn invoke_connector(
    cfg: &Config,
    connector_id: &str,
    payload: &Value,
    attachment_rel: Option<&str>,
    thread_id: &str,
) -> Result<InvokeOutcome, String> {
    let connector = load_connector(&cfg.data_dir, connector_id)?;
    if !connector.enabled {
        return Err(format!("connector '{}' is disabled", connector.name));
    }
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    let secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    let safe_url = redact_with_store(&connector.url, &secrets);
    let payload_bytes = build_delivery_payload_bytes(payload);
    let payload_sha256 = tripwire::sha256_hex(&payload_bytes);
    let now_ms = records::now_millis();

    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        let err_msg = "blocked by Air-Gapped workspace policy: outbound connector delivery is disabled for this workspace".to_string();
        let log = DeliveryLog {
            id: format!("del-{now_ms}-{}", next_seq()),
            connector_id: connector.id.clone(),
            connector_name: connector.name.clone(),
            kind: connector.kind.clone(),
            method: connector.method.clone(),
            url: safe_url,
            status: "blocked".to_string(),
            http_status: None,
            decision_id: None,
            thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
            payload_sha256,
            payload_bytes: payload_bytes.len(),
            response_preview: String::new(),
            error: Some(err_msg.clone()),
            ts_ms: now_ms,
        };
        record_delivery_log(&cfg.data_dir, &connector.id, log);
        return Err(err_msg);
    }

    // Validate target URL and payload before parking or delivering.
    let payload_text = String::from_utf8_lossy(&payload_bytes);
    tripwire::check_egress(&connector.url, "connector")
        .and_then(|_| tripwire::check_egress_payload(&payload_text, "connector"))
        .map_err(|e| redact_with_store(&e, &secrets))?;

    let effective_require_approval =
        connector.require_approval || policy.profile == "strict_client";

    if effective_require_approval {
        let raw_preview = serde_json::to_string(payload).unwrap_or_default();
        let mut redacted = redact_with_store(&raw_preview, &secrets);
        if policy.redact_host_paths {
            redacted = crate::redact_host_paths(&redacted);
        }
        let payload_preview = crate::truncate(&redacted, 400);
        let safe_payload: Value = serde_json::from_str(&redacted)
            .unwrap_or_else(|_| json!(payload_preview));
        let summary = format!(
            "Send outbound {} via \"{}\" ({})",
            connector.method, connector.name, safe_url
        );
        let mut detail = if policy.profile == "strict_client" {
            format!(
                "Strict Client policy requires human approval before sending data to {safe_url}. Payload SHA-256: {payload_sha256}."
            )
        } else {
            format!(
                "Connector \"{}\" requires approval before sending data to {}. Payload SHA-256: {}.",
                connector.name, safe_url, payload_sha256
            )
        };
        if policy.redact_host_paths {
            detail = crate::redact_host_paths(&detail);
        }
        let decision_id = records::add_decision_with(
            &cfg.data_dir,
            "connector_delivery",
            &summary,
            attachment_rel,
            thread_id,
            json!({
                "detail": detail,
                "severity": "warning",
                "connector_id": connector.id,
                "connector_name": connector.name,
                "connector_kind": connector.kind,
                "method": connector.method,
                "url": safe_url,
                "payload": safe_payload,
                "payload_preview": payload_preview,
                "payload_sha256": payload_sha256,
            }),
        );
        let log = DeliveryLog {
            id: format!("del-{now_ms}-{}", next_seq()),
            connector_id: connector.id.clone(),
            connector_name: connector.name.clone(),
            kind: connector.kind.clone(),
            method: connector.method.clone(),
            url: safe_url,
            status: "held".to_string(),
            http_status: None,
            decision_id: Some(decision_id.clone()),
            thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
            payload_sha256,
            payload_bytes: payload_bytes.len(),
            response_preview: String::new(),
            error: None,
            ts_ms: now_ms,
        };
        record_delivery_log(&cfg.data_dir, &connector.id, log.clone());
        return Ok(InvokeOutcome::Held {
            decision_id,
            delivery: log,
        });
    }

    let log = execute_http_delivery(cfg, &connector, payload, thread_id, None)?;
    Ok(InvokeOutcome::Delivered(log))
}

/// HTTP `POST /api/connectors/outbound/:id/test` helper returning HTTP status + JSON.
pub fn test_outbound_connector(
    cfg: &Config,
    connector_id: &str,
    payload: &Value,
) -> (u16, Value) {
    let test_payload = if payload.is_null() {
        json!({
            "event": "ergon_connector_test",
            "connector_id": connector_id,
            "ts_ms": records::now_millis(),
        })
    } else {
        payload.clone()
    };
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    match invoke_connector(cfg, connector_id, &test_payload, None, "") {
        Ok(InvokeOutcome::Delivered(log)) => (
            200,
            json!({
                "ok": true,
                "status": log.status,
                "delivery": log.to_json(),
            }),
        ),
        Ok(InvokeOutcome::Held {
            decision_id,
            delivery,
        }) => (
            202,
            json!({
                "ok": false,
                "held": true,
                "status": "held",
                "decision_id": decision_id,
                "delivery": delivery.to_json(),
            }),
        ),
        Err(e) => {
            let status = if e.contains("unknown outbound connector") {
                404
            } else if policy.block_outbound_fetch || e.contains("Air-Gapped") {
                403
            } else {
                502
            };
            let latest = with_connectors_lock(|| {
                ConnectorsStore::load_unlocked(&cfg.data_dir)
                    .delivery_log
                    .into_iter()
                    .rev()
                    .find(|d| d.connector_id == connector_id)
                    .map(|d| d.to_json())
            });
            (
                status,
                json!({
                    "ok": false,
                    "error": e,
                    "delivery": latest,
                }),
            )
        }
    }
}

/// Execute an approved `connector_delivery` decision (called from `records::resolve_decision`
/// outside `RECORDS_LOCK`). Returns the final human-readable outcome string.
pub fn settle_delivery_decision(cfg: &Config, decision: &Value) -> String {
    let decision_id = decision["id"].as_str().unwrap_or("");
    let connector_id = decision["connector_id"].as_str().unwrap_or("");
    let thread_id = decision["thread_id"].as_str().unwrap_or("");
    let payload = &decision["payload"];
    let connector = match load_connector(&cfg.data_dir, connector_id) {
        Ok(c) => c,
        Err(e) => return format!("Approved, but connector is unavailable: {e}"),
    };
    if !connector.enabled {
        return format!("Approved, but connector \"{}\" is disabled.", connector.name);
    }
    match execute_http_delivery(cfg, &connector, payload, thread_id, Some(decision_id)) {
        Ok(log) => format!(
            "Approved and delivered via \"{}\" (HTTP {}, {} bytes).",
            connector.name,
            log.http_status.unwrap_or(200),
            log.payload_bytes
        ),
        Err(e) => format!("Approved, but outbound delivery failed: {e}"),
    }
}

/// Record a denied `connector_delivery` in `DeliveryLog`.
pub fn record_denied_delivery(cfg: &Config, decision: &Value) {
    let connector_id = decision["connector_id"].as_str().unwrap_or("");
    if connector_id.is_empty() {
        return;
    }
    let now_ms = records::now_millis();
    let payload_bytes = build_delivery_payload_bytes(&decision["payload"]);
    let log = DeliveryLog {
        id: format!("del-{now_ms}-{}", next_seq()),
        connector_id: connector_id.to_string(),
        connector_name: decision["connector_name"]
            .as_str()
            .unwrap_or(connector_id)
            .to_string(),
        kind: decision["connector_kind"]
            .as_str()
            .unwrap_or("http_webhook")
            .to_string(),
        method: decision["method"].as_str().unwrap_or("POST").to_string(),
        url: decision["url"].as_str().unwrap_or("").to_string(),
        status: "denied".to_string(),
        http_status: None,
        decision_id: decision["id"].as_str().map(str::to_string),
        thread_id: decision["thread_id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        payload_sha256: decision["payload_sha256"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        payload_bytes: payload_bytes.len(),
        response_preview: String::new(),
        error: Some("denied by user".to_string()),
        ts_ms: now_ms,
    };
    record_delivery_log(&cfg.data_dir, connector_id, log);
}
