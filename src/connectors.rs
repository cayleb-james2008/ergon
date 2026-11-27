//! Unattended Intake Triggers & Policy-Gated Outbound Action Connectors.
//!
//! Persists non-secret connector metadata and delivery logs in
//! `<person_data_dir>/connectors.json` (0600 POSIX) and isolates all webhook
//! signing secrets, hook tokens, and outbound API secrets in a separate
//! `<person_data_dir>/connector_secrets.json` (0600 POSIX) that is never
//! readable from the workspace or injected into LLM prompts.

use crate::tools::{atomic_write_bytes, atomic_write_bytes_with_mode, sandbox_path, sandbox_path_resolving};
use crate::{records, tripwire, Config, WorkspacePolicy};
use rustls::pki_types::ServerName;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, UNIX_EPOCH};

const MAX_SMTP_LINE_BYTES: usize = 16 * 1024;
const MAX_SMTP_CONT_LINES: usize = 128;
const SMTP_SESSION_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_IMAP_LINE_BYTES: usize = 64 * 1024;
const MAX_IMAP_UNTAGGED_LINES: usize = 512;
const IMAP_SESSION_TIMEOUT: Duration = Duration::from_secs(25);

pub const MAX_WEBHOOK_BYTES: usize = 1_048_576;
pub const MAX_FOLDER_WATCH_BYTES: u64 = 25 * 1024 * 1024;
pub const MAX_SMTP_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_IMAP_MESSAGE_BYTES: usize = 25 * 1024 * 1024;
const MAX_IMAP_MESSAGES_PER_POLL: usize = 25;
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
pub struct ImapTarget {
    pub host: String,
    pub port: u16,
    pub security: String,
    pub folder: String,
    pub poll_interval_secs: u64,
    pub mark_seen: bool,
    pub username_secret_key: String,
    pub password_secret_key: String,
    pub subject_filter: Option<String>,
}

impl ImapTarget {
    pub fn to_json(&self) -> Value {
        json!({
            "host": self.host,
            "port": self.port,
            "security": self.security,
            "folder": self.folder,
            "poll_interval_secs": self.poll_interval_secs,
            "mark_seen": self.mark_seen,
            "username_secret_key": self.username_secret_key,
            "password_secret_key": self.password_secret_key,
            "subject_filter": self.subject_filter,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let host = obj.get("host")?.as_str()?.trim().to_string();
        if host.is_empty() {
            return None;
        }
        let security = obj
            .get("security")
            .and_then(|s| s.as_str())
            .unwrap_or("imaps")
            .trim()
            .to_ascii_lowercase();
        if !matches!(
            security.as_str(),
            "imaps" | "starttls" | "plain_loopback_test"
        ) {
            return None;
        }
        let default_port = if security == "imaps" { 993 } else { 143 };
        let port = obj
            .get("port")
            .and_then(|p| p.as_u64())
            .and_then(|p| u16::try_from(p).ok())
            .filter(|&p| p > 0)
            .unwrap_or(default_port);
        let folder = obj
            .get("folder")
            .and_then(|s| s.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("INBOX")
            .to_string();
        let poll_interval_secs = obj
            .get("poll_interval_secs")
            .and_then(|n| n.as_u64())
            .unwrap_or(60)
            .clamp(1, 86_400);
        let mark_seen = obj
            .get("mark_seen")
            .and_then(|b| b.as_bool())
            .unwrap_or(true);
        let username_secret_key = obj
            .get("username_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let password_secret_key = obj
            .get("password_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let subject_filter = obj
            .get("subject_filter")
            .and_then(|s| s.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        Some(Self {
            host,
            port,
            security,
            folder,
            poll_interval_secs,
            mark_seen,
            username_secret_key,
            password_secret_key,
            subject_filter,
        })
    }
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
    pub imap_config: Option<ImapTarget>,
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
            "imap_config": self.imap_config.as_ref().map(|i| i.to_json()),
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
        if id.is_empty() || !matches!(kind.as_str(), "webhook" | "folder_watch" | "imap_poll") {
            return None;
        }
        let imap_config = if kind == "imap_poll" {
            Some(ImapTarget::from_json(&v["imap_config"])?)
        } else {
            ImapTarget::from_json(&v["imap_config"])
        };
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
            imap_config,
            trigger_count: v["trigger_count"].as_u64().unwrap_or(0),
            last_triggered_ms: v["last_triggered_ms"].as_u64(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SmtpTarget {
    pub host: String,
    pub port: u16,
    pub security: String,
    pub username_secret_key: String,
    pub password_secret_key: String,
    pub from_addr: String,
    pub default_to: Vec<String>,
}

impl SmtpTarget {
    pub fn to_json(&self) -> Value {
        json!({
            "host": self.host,
            "port": self.port,
            "security": self.security,
            "username_secret_key": self.username_secret_key,
            "password_secret_key": self.password_secret_key,
            "from_addr": self.from_addr,
            "default_to": self.default_to,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let host = obj.get("host")?.as_str()?.trim().to_string();
        if host.is_empty() {
            return None;
        }
        let security = obj
            .get("security")
            .and_then(|s| s.as_str())
            .unwrap_or("smtps")
            .trim()
            .to_ascii_lowercase();
        if !matches!(
            security.as_str(),
            "smtps" | "starttls" | "plain_loopback_test"
        ) {
            return None;
        }
        let default_port = match security.as_str() {
            "smtps" => 465,
            "starttls" => 587,
            _ => 2525,
        };
        let port = obj
            .get("port")
            .and_then(|p| p.as_u64())
            .and_then(|p| u16::try_from(p).ok())
            .filter(|&p| p > 0)
            .unwrap_or(default_port);
        let username_secret_key = obj
            .get("username_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let password_secret_key = obj
            .get("password_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let from_addr = obj
            .get("from_addr")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let default_to = obj
            .get("default_to")
            .and_then(|a| a.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|x| x.as_str().map(str::trim).filter(|s| !s.is_empty()))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            host,
            port,
            security,
            username_secret_key,
            password_secret_key,
            from_addr,
            default_to,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OAuth2Config {
    pub token_url: String,
    pub client_id: String,
    pub client_secret_key: String,
    pub refresh_token_secret_key: String,
    pub scopes: Vec<String>,
    pub access_token_expires_ms: Option<u64>,
}

impl OAuth2Config {
    pub fn to_json(&self) -> Value {
        json!({
            "token_url": self.token_url,
            "client_id": self.client_id,
            "client_secret_key": self.client_secret_key,
            "refresh_token_secret_key": self.refresh_token_secret_key,
            "scopes": self.scopes,
            "access_token_expires_ms": self.access_token_expires_ms,
        })
    }

    fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let token_url = obj.get("token_url")?.as_str()?.trim().to_string();
        let client_id = obj.get("client_id")?.as_str()?.trim().to_string();
        if token_url.is_empty() || client_id.is_empty() {
            return None;
        }
        let client_secret_key = obj
            .get("client_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let refresh_token_secret_key = obj
            .get("refresh_token_secret_key")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let scopes = obj
            .get("scopes")
            .and_then(|a| a.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|x| x.as_str().map(str::trim).filter(|s| !s.is_empty()))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let access_token_expires_ms = obj.get("access_token_expires_ms").and_then(|n| n.as_u64());
        Some(Self {
            token_url,
            client_id,
            client_secret_key,
            refresh_token_secret_key,
            scopes,
            access_token_expires_ms,
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
    pub smtp_config: Option<SmtpTarget>,
    pub oauth_config: Option<OAuth2Config>,
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
        if let Some(smtp) = &self.smtp_config {
            for k in [&smtp.username_secret_key, &smtp.password_secret_key] {
                if !k.is_empty() && !keys.contains(k) {
                    keys.push(k.clone());
                }
            }
        }
        if let Some(oauth) = &self.oauth_config {
            for k in [&oauth.client_secret_key, &oauth.refresh_token_secret_key] {
                if !k.is_empty() && !keys.contains(k) {
                    keys.push(k.clone());
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
        let has_secret = !keys.is_empty()
            && keys
                .iter()
                .filter(|k| {
                    // oauth_access_token may be populated lazily on first refresh
                    k.as_str() != "oauth_access_token" || self.oauth_config.is_none()
                })
                .all(|k| secrets.secrets.contains_key(k));
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
            "smtp_config": self.smtp_config.as_ref().map(|s| s.to_json()),
            "oauth_config": self.oauth_config.as_ref().map(|o| o.to_json()),
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
            "smtp_config": self.smtp_config.as_ref().map(|s| s.to_json()),
            "oauth_config": self.oauth_config.as_ref().map(|o| o.to_json()),
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
        let url = v["url"].as_str().unwrap_or("").trim().to_string();
        let smtp_config = SmtpTarget::from_json(&v["smtp_config"]);
        let oauth_config = OAuth2Config::from_json(&v["oauth_config"]);
        if id.is_empty() || (url.is_empty() && smtp_config.is_none()) {
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
            smtp_config,
            oauth_config,
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

pub fn get_connector_secret_map(person_data_dir: &Path) -> BTreeMap<String, String> {
    with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(person_data_dir).secrets)
}

pub fn parse_url_host_port_pub(raw_url: &str, default_port: u16) -> (String, u16) {
    parse_url_host_port(raw_url, default_port)
}

/// Scrub inline `user:pass@` authority credentials from a URL before logging
/// or returning it in error payloads.
pub fn redact_url_userinfo(raw_url: &str) -> String {
    let Some((scheme, rest)) = raw_url.split_once("://") else {
        return raw_url.to_string();
    };
    let auth_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..auth_end];
    if let Some((_, host_part)) = authority.rsplit_once('@') {
        format!("{scheme}://***@{host_part}{}", &rest[auth_end..])
    } else {
        raw_url.to_string()
    }
}

/// Redact any known connector secret values (including partial and JSON-escaped
/// echoes) from `text`, plus any extra transient tokens (e.g. AUTH blobs).
pub fn redact_with_store_extra(
    text: &str,
    secrets: &ConnectorSecretsStore,
    extras: &[String],
) -> String {
    let mut out = tripwire::redact(text);
    for val in secrets.all_values() {
        tripwire::redact_secret_value(&mut out, &val, false);
    }
    for extra in extras {
        if extra.trim().len() >= 4 {
            tripwire::redact_secret_value(&mut out, extra, false);
        }
    }
    out
}

/// Redact any known connector secret values (including partial and JSON-escaped
/// echoes) from `text`.
pub fn redact_with_store(text: &str, secrets: &ConnectorSecretsStore) -> String {
    redact_with_store_extra(text, secrets, &[])
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
                } else if t.kind == "imap_poll" {
                    t.imap_config.as_ref().is_some_and(|imap| {
                        let keys: Vec<&str> = [
                            imap.username_secret_key.as_str(),
                            imap.password_secret_key.as_str(),
                        ]
                        .into_iter()
                        .filter(|k| !k.is_empty())
                        .collect();
                        !keys.is_empty()
                            && keys.iter().all(|k| secrets.secrets.contains_key(*k))
                    })
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

fn is_loopback_host(host: &str) -> bool {
    let h = host.trim().trim_start_matches('[').trim_end_matches(']');
    h.eq_ignore_ascii_case("localhost")
        || h.parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

fn local_connectors_env_allowed() -> bool {
    let truthy = |k: &str| {
        std::env::var(k)
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false)
    };
    truthy("ERGON_ALLOW_LOCAL_CONNECTORS") || truthy("ERGON_FETCH_ALLOW_PRIVATE")
}

fn validate_plain_loopback_security(host: &str, security: &str, proto: &str) -> Result<(), String> {
    if security == "plain_loopback_test" {
        if !is_loopback_host(host) {
            return Err(format!(
                "{proto} security 'plain_loopback_test' is only allowed on loopback hosts (127.0.0.1, ::1, localhost)"
            ));
        }
        if !local_connectors_env_allowed() {
            return Err(format!(
                "{proto} security 'plain_loopback_test' requires ERGON_ALLOW_LOCAL_CONNECTORS=1"
            ));
        }
    }
    Ok(())
}

fn parse_url_host_port(raw_url: &str, default_port: u16) -> (String, u16) {
    let after_scheme = raw_url
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(raw_url);
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    if let Some(rest) = host_port.strip_prefix('[') {
        let host = rest.split(']').next().unwrap_or("").to_string();
        let port = rest
            .split("]:")
            .nth(1)
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(default_port);
        return (host, port);
    }
    if let Some((h, p)) = host_port.rsplit_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port);
        }
    }
    (host_port.to_string(), default_port)
}

fn validate_email_addr(addr: &str, field: &str) -> Result<String, String> {
    let trimmed = addr.trim();
    if trimmed.is_empty()
        || trimmed.len() > 254
        || trimmed.contains(['\r', '\n', '\0', '<', '>'])
        || !trimmed.contains('@')
    {
        return Err(format!("{field} must be a valid email address"));
    }
    let Some((local, domain)) = trimmed.split_once('@') else {
        return Err(format!("{field} must be a valid email address"));
    };
    if local.is_empty() || domain.is_empty() || domain.contains(char::is_whitespace) {
        return Err(format!("{field} must be a valid email address"));
    }
    Ok(trimmed.to_string())
}

fn parse_secrets_map_from_body(body: &Value) -> Result<BTreeMap<String, String>, (u16, String)> {
    let mut new_secrets = BTreeMap::new();
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
    Ok(new_secrets)
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
    if !matches!(kind.as_str(), "webhook" | "folder_watch" | "imap_poll") {
        return Err((
            400,
            "trigger kind must be 'webhook', 'folder_watch', or 'imap_poll'".to_string(),
        ));
    }
    let name: String = body["name"]
        .as_str()
        .unwrap_or(match kind.as_str() {
            "webhook" => "Webhook Trigger",
            "imap_poll" => "IMAP Inbox Trigger",
            _ => "Folder Watch Trigger",
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
            imap_config: None,
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
    } else if kind == "imap_poll" {
        let imap_val = if body["imap_config"].is_object() {
            &body["imap_config"]
        } else {
            body
        };
        let imap = ImapTarget::from_json(imap_val).ok_or_else(|| {
            (
                400,
                "imap_poll trigger requires imap_config with valid host and security ('imaps', 'starttls', or 'plain_loopback_test')".to_string(),
            )
        })?;
        if imap.folder.contains(['\r', '\n', '\0']) {
            return Err((400, "imap folder contains invalid characters".to_string()));
        }
        if !imap.username_secret_key.is_empty() && !is_valid_secret_key(&imap.username_secret_key) {
            return Err((400, "invalid imap username_secret_key".to_string()));
        }
        if !imap.password_secret_key.is_empty() && !is_valid_secret_key(&imap.password_secret_key) {
            return Err((400, "invalid imap password_secret_key".to_string()));
        }
        validate_plain_loopback_security(&imap.host, &imap.security, "IMAP")
            .map_err(|e| (400, e))?;
        tripwire::check_egress_host(&imap.host, imap.port, "connector_config")
            .map_err(|e| (400, e))?;

        let new_secrets = parse_secrets_map_from_body(body)?;
        let rule = TriggerRule {
            id,
            name,
            kind: "imap_poll".to_string(),
            enabled,
            routine_id,
            target_kind,
            hook_id: None,
            watch_dir: None,
            glob: None,
            max_bytes: MAX_IMAP_MESSAGE_BYTES as u64,
            imap_config: Some(imap.clone()),
            trigger_count: 0,
            last_triggered_ms: None,
            created_ms: now_ms,
            updated_ms: now_ms,
        };
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
            let has_sec = [&imap.username_secret_key, &imap.password_secret_key]
                .into_iter()
                .filter(|k| !k.is_empty())
                .all(|k| secrets.secrets.contains_key(k))
                && (!imap.username_secret_key.is_empty() || !imap.password_secret_key.is_empty());
            store.triggers.push(rule.clone());
            store.save_unlocked(person_data_dir).map_err(|e| (500, e))?;
            Ok(json!({
                "trigger": rule.to_json(has_sec),
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
            imap_config: None,
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
    let mut smtp_config: Option<SmtpTarget> = None;
    if body.get("smtp_config").is_some_and(|v| !v.is_null()) {
        let mut parsed_smtp = SmtpTarget::from_json(&body["smtp_config"]).ok_or_else(|| {
            (
                400,
                "smtp_config must include valid host and security ('smtps', 'starttls', or 'plain_loopback_test')".to_string(),
            )
        })?;
        validate_plain_loopback_security(&parsed_smtp.host, &parsed_smtp.security, "SMTP")
            .map_err(|e| (400, e))?;
        tripwire::check_egress_host(&parsed_smtp.host, parsed_smtp.port, "connector_config")
            .map_err(|e| (400, e))?;
        if !parsed_smtp.username_secret_key.is_empty()
            && !is_valid_secret_key(&parsed_smtp.username_secret_key)
        {
            return Err((400, "invalid smtp username_secret_key".to_string()));
        }
        if !parsed_smtp.password_secret_key.is_empty()
            && !is_valid_secret_key(&parsed_smtp.password_secret_key)
        {
            return Err((400, "invalid smtp password_secret_key".to_string()));
        }
        if !parsed_smtp.from_addr.is_empty() {
            parsed_smtp.from_addr = validate_email_addr(&parsed_smtp.from_addr, "smtp_config.from_addr")
                .map_err(|e| (400, e))?;
        }
        let mut clean_to = Vec::with_capacity(parsed_smtp.default_to.len());
        for rcpt in &parsed_smtp.default_to {
            clean_to.push(validate_email_addr(rcpt, "smtp_config.default_to").map_err(|e| (400, e))?);
        }
        parsed_smtp.default_to = clean_to;
        smtp_config = Some(parsed_smtp);
    } else if kind == "smtp_relay" {
        return Err((400, "smtp_relay connector requires smtp_config".to_string()));
    }

    let mut oauth_config: Option<OAuth2Config> = None;
    if body.get("oauth_config").is_some_and(|v| !v.is_null()) {
        let parsed_oauth = OAuth2Config::from_json(&body["oauth_config"]).ok_or_else(|| {
            (
                400,
                "oauth_config must include valid token_url and client_id".to_string(),
            )
        })?;
        if parsed_oauth.token_url.contains("{{secret:") {
            return Err((400, "oauth_config.token_url cannot contain secret placeholders".to_string()));
        }
        tripwire::check_egress(&parsed_oauth.token_url, "connector_config")
            .map_err(|e| (400, e))?;
        let (thost, tport) = parse_url_host_port(&parsed_oauth.token_url, 443);
        tripwire::check_egress_host(&thost, tport, "connector_config")
            .map_err(|e| (400, e))?;
        if !parsed_oauth.client_secret_key.is_empty()
            && !is_valid_secret_key(&parsed_oauth.client_secret_key)
        {
            return Err((400, "invalid oauth_config.client_secret_key".to_string()));
        }
        if !parsed_oauth.refresh_token_secret_key.is_empty()
            && !is_valid_secret_key(&parsed_oauth.refresh_token_secret_key)
        {
            return Err((400, "invalid oauth_config.refresh_token_secret_key".to_string()));
        }
        oauth_config = Some(parsed_oauth);
    }

    let mut url = body["url"].as_str().unwrap_or("").trim().to_string();
    if url.is_empty() {
        if let Some(smtp) = &smtp_config {
            let scheme = if smtp.security == "smtps" {
                "smtps"
            } else {
                "smtp"
            };
            url = format!("{scheme}://{}:{}", smtp.host, smtp.port);
        }
    }
    if url.is_empty() || url.contains("{{secret:") {
        return Err((
            400,
            "connector url must be a valid http://, https://, smtp://, or smtps:// URL without secret placeholders"
                .to_string(),
        ));
    }
    tripwire::check_egress(&url, "connector_config").map_err(|e| (400, e))?;
    let default_port = if url.to_ascii_lowercase().starts_with("https://") {
        443
    } else if url.to_ascii_lowercase().starts_with("smtps://") {
        465
    } else if url.to_ascii_lowercase().starts_with("smtp://") {
        587
    } else {
        80
    };
    let (uhost, uport) = parse_url_host_port(&url, default_port);
    tripwire::check_egress_host(&uhost, uport, "connector_config").map_err(|e| (400, e))?;

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

    let new_secrets = parse_secrets_map_from_body(body)?;

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
        if let Some(smtp) = &smtp_config {
            for k in [&smtp.username_secret_key, &smtp.password_secret_key] {
                if !k.is_empty() && !secret_keys.contains(k) {
                    secret_keys.push(k.clone());
                }
            }
        }
        if let Some(oauth) = &oauth_config {
            for k in [&oauth.client_secret_key, &oauth.refresh_token_secret_key] {
                if !k.is_empty() && !secret_keys.contains(k) {
                    secret_keys.push(k.clone());
                }
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
            if smtp_config.is_some() {
                updated.smtp_config = smtp_config;
            }
            if oauth_config.is_some() {
                updated.oauth_config = oauth_config;
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
                smtp_config,
                oauth_config,
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

fn form_urlencode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn format_rfc2822_utc(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    // Howard Hinnant's civil-from-days algorithm (epoch 1970-01-01 = Thursday)
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    const DOW: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MON: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let dow = DOW[days.rem_euclid(7) as usize];
    let mon = MON[(m - 1).clamp(0, 11) as usize];
    format!("{dow}, {d:02} {mon} {year:04} {hour:02}:{min:02}:{sec:02} +0000")
}

/// Refresh an OAuth2 access token into `0o600` `connector_secrets.json` under
/// key `"oauth_access_token"` if `connector_id` defines `oauth_config` and the
/// cached token is missing or within 30 seconds of expiry.
pub fn ensure_fresh_oauth_token(
    person_data_dir: &Path,
    connector_id: &str,
) -> Result<Option<String>, String> {
    let now_ms = records::now_millis();
    let (oauth, refresh_token, client_secret, existing_token) = with_connectors_lock(|| {
        let store = ConnectorsStore::load_unlocked(person_data_dir);
        let secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        let Some(conn) = store.outbound.iter().find(|c| c.id == connector_id) else {
            return Err(format!("unknown outbound connector '{connector_id}'"));
        };
        let Some(oauth) = conn.oauth_config.clone() else {
            return Ok((None, String::new(), None, None));
        };
        let existing = secrets.secrets.get("oauth_access_token").cloned();
        let refresh = secrets
            .secrets
            .get(&oauth.refresh_token_secret_key)
            .cloned()
            .unwrap_or_default();
        let csec = if oauth.client_secret_key.is_empty() {
            None
        } else {
            secrets.secrets.get(&oauth.client_secret_key).cloned()
        };
        Ok((Some(oauth), refresh, csec, existing))
    })?;

    let Some(oauth) = oauth else {
        return Ok(None);
    };
    if let (Some(exp), Some(tok)) = (oauth.access_token_expires_ms, existing_token.as_ref()) {
        if !tok.is_empty() && now_ms.saturating_add(30_000) < exp {
            return Ok(Some(tok.clone()));
        }
    }
    if refresh_token.trim().is_empty() {
        return Err(format!(
            "missing OAuth2 refresh token secret '{}'",
            oauth.refresh_token_secret_key
        ));
    }

    tripwire::check_egress(&oauth.token_url, "connector")?;
    let (thost, tport) = parse_url_host_port(&oauth.token_url, 443);
    tripwire::check_egress_host(&thost, tport, "connector")?;

    let mut form_parts = vec![
        format!("grant_type=refresh_token"),
        format!("refresh_token={}", form_urlencode_component(&refresh_token)),
        format!("client_id={}", form_urlencode_component(&oauth.client_id)),
    ];
    if let Some(cs) = client_secret.as_deref().filter(|s| !s.is_empty()) {
        form_parts.push(format!("client_secret={}", form_urlencode_component(cs)));
    }
    if !oauth.scopes.is_empty() {
        let joined = oauth.scopes.join(" ");
        form_parts.push(format!("scope={}", form_urlencode_component(&joined)));
    }
    let form_body = form_parts.join("&");

    let mut oauth_extras = vec![refresh_token.clone()];
    if let Some(ref cs) = client_secret {
        oauth_extras.push(cs.clone());
    }
    if let Some(ref ex) = existing_token {
        oauth_extras.push(ex.clone());
    }
    let redact_oauth_err = |msg: String| -> String {
        let store_snap = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(person_data_dir));
        redact_with_store_extra(&msg, &store_snap, &oauth_extras)
    };

    let agent = tripwire::guarded_connector_agent(Duration::from_secs(15));
    let resp = agent
        .post(&oauth.token_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept", "application/json")
        .send_string(&form_body)
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => {
                let mut buf = Vec::new();
                let _ = r.into_reader().take(1024).read_to_end(&mut buf);
                let msg = String::from_utf8_lossy(&buf);
                redact_oauth_err(format!("OAuth2 token refresh failed with HTTP {code}: {msg}"))
            }
            other => redact_oauth_err(tripwire::explain_fetch_error(&oauth.token_url, &other)),
        })?;

    let mut raw = Vec::new();
    resp.into_reader()
        .take(65_536)
        .read_to_end(&mut raw)
        .map_err(|e| redact_oauth_err(format!("read OAuth2 token response: {e}")))?;
    let parsed: Value = serde_json::from_slice(&raw)
        .map_err(|e| redact_oauth_err(format!("parse OAuth2 token JSON: {e}")))?;
    let access_token = parsed["access_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "OAuth2 token response missing access_token".to_string())?
        .to_string();
    if access_token.contains('\r') || access_token.contains('\n') {
        return Err("OAuth2 access_token contains invalid newline".to_string());
    }
    let expires_in_secs = parsed["expires_in"].as_u64().unwrap_or(3600).clamp(60, 86_400 * 30);
    let updated_ms = records::now_millis();
    let expires_ms = updated_ms.saturating_add(expires_in_secs.saturating_mul(1000));
    let rotated_refresh = parsed["refresh_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.contains('\r') && !s.contains('\n'))
        .map(str::to_string);

    let mut reg_secrets = vec![access_token.clone()];
    if let Some(ref rt) = rotated_refresh {
        reg_secrets.push(rt.clone());
    }

    with_connectors_lock(|| {
        let mut secrets = ConnectorSecretsStore::load_unlocked(person_data_dir);
        secrets
            .secrets
            .insert("oauth_access_token".to_string(), access_token.clone());
        secrets
            .secret_updated_ms
            .insert("oauth_access_token".to_string(), updated_ms);
        if let Some(new_rt) = rotated_refresh {
            secrets
                .secrets
                .insert(oauth.refresh_token_secret_key.clone(), new_rt);
            secrets
                .secret_updated_ms
                .insert(oauth.refresh_token_secret_key.clone(), updated_ms);
        }
        secrets.save_unlocked(person_data_dir)?;

        let mut store = ConnectorsStore::load_unlocked(person_data_dir);
        if let Some(c) = store.outbound.iter_mut().find(|c| c.id == connector_id) {
            if let Some(oc) = c.oauth_config.as_mut() {
                oc.access_token_expires_ms = Some(expires_ms);
            }
            if !c.secret_keys.iter().any(|k| k == "oauth_access_token") {
                c.secret_keys.push("oauth_access_token".to_string());
            }
            c.updated_ms = updated_ms;
            store.save_unlocked(person_data_dir)?;
        }
        Ok::<(), String>(())
    })?;

    tripwire::register_redaction_secrets(&reg_secrets);
    Ok(Some(access_token))
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
    let mut secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    let safe_url = redact_with_store(&redact_url_userinfo(&connector.url), &secrets);

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

    let (uhost, uport) = parse_url_host_port(&connector.url, 443);
    if let Err(e) = tripwire::check_egress(&connector.url, "connector")
        .and_then(|_| tripwire::check_egress_host(&uhost, uport, "connector"))
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

    if connector.oauth_config.is_some() {
        if let Err(e) = ensure_fresh_oauth_token(&cfg.data_dir, &connector.id) {
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
        secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
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

    let agent = tripwire::guarded_connector_agent(Duration::from_secs(15));
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

fn build_rustls_config() -> Result<Arc<rustls::ClientConfig>, String> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| format!("configure TLS versions: {e}"))?
    .with_root_certificates(root_store)
    .with_no_client_auth();
    Ok(Arc::new(cfg))
}

enum ProtoStream {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl ProtoStream {
    fn tcp_ref(&self) -> &TcpStream {
        match self {
            Self::Plain(s) => s,
            Self::Tls(s) => s.get_ref(),
        }
    }

    fn apply_deadline(&self, deadline: Instant, proto: &str) -> Result<(), String> {
        let rem = deadline.saturating_duration_since(Instant::now());
        if rem.is_zero() {
            return Err(format!("{proto} session timed out"));
        }
        let bounded = rem.max(Duration::from_millis(1));
        let tcp = self.tcp_ref();
        let _ = tcp.set_read_timeout(Some(bounded));
        let _ = tcp.set_write_timeout(Some(bounded));
        Ok(())
    }

    fn connect(host: &str, port: u16, use_tls: bool) -> Result<Self, String> {
        let addrs = tripwire::guard_resolve_connector(host, port)
            .map_err(|e| format!("resolve {host}:{port}: {e}"))?;
        let mut last_err = None;
        let mut connected = None;
        for addr in addrs {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(15)) {
                Ok(s) => {
                    connected = Some(s);
                    break;
                }
                Err(e) => last_err = Some(e),
            }
        }
        let stream = connected.ok_or_else(|| {
            format!(
                "connect to {host}:{port} failed: {}",
                last_err
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "no reachable address".to_string())
            )
        })?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
        if use_tls {
            Self::upgrade_tcp(stream, host)
        } else {
            Ok(Self::Plain(stream))
        }
    }

    fn upgrade_tcp(tcp: TcpStream, host: &str) -> Result<Self, String> {
        let tls_cfg = build_rustls_config()?;
        let server_name = ServerName::try_from(host.trim().to_string())
            .map_err(|_| format!("invalid TLS server name '{host}'"))?;
        let conn = rustls::ClientConnection::new(tls_cfg, server_name)
            .map_err(|e| format!("TLS handshake init for {host}: {e}"))?;
        let mut tls_stream = rustls::StreamOwned::new(conn, tcp);
        tls_stream
            .flush()
            .map_err(|e| format!("TLS handshake flush for {host}: {e}"))?;
        Ok(Self::Tls(Box::new(tls_stream)))
    }

    fn upgrade_starttls(self, host: &str) -> Result<Self, String> {
        match self {
            Self::Plain(tcp) => Self::upgrade_tcp(tcp, host),
            already_tls => Ok(already_tls),
        }
    }
}

impl Read for ProtoStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(s) => s.read(buf),
            Self::Tls(s) => s.read(buf),
        }
    }
}

impl Write for ProtoStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(s) => s.write(buf),
            Self::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(s) => s.flush(),
            Self::Tls(s) => s.flush(),
        }
    }
}

fn read_line_bounded(
    reader: &mut BufReader<ProtoStream>,
    max_bytes: usize,
    deadline: Instant,
    proto: &str,
) -> Result<Vec<u8>, String> {
    reader.get_ref().apply_deadline(deadline, proto)?;
    let mut out = Vec::new();
    loop {
        let avail = reader
            .fill_buf()
            .map_err(|e| format!("{proto} read error: {e}"))?;
        if avail.is_empty() {
            if out.is_empty() {
                return Err(format!("{proto} server closed connection unexpectedly"));
            }
            return Ok(out);
        }
        if let Some(pos) = avail.iter().position(|&b| b == b'\n') {
            let take = pos + 1;
            if out.len() + take > max_bytes {
                return Err(format!("{proto} response line exceeds {max_bytes} byte limit"));
            }
            out.extend_from_slice(&avail[..take]);
            reader.consume(take);
            return Ok(out);
        }
        if out.len() + avail.len() > max_bytes {
            return Err(format!("{proto} response line exceeds {max_bytes} byte limit"));
        }
        out.extend_from_slice(avail);
        let len = avail.len();
        reader.consume(len);
        reader.get_ref().apply_deadline(deadline, proto)?;
    }
}

fn read_smtp_response(
    reader: &mut BufReader<ProtoStream>,
    deadline: Instant,
) -> Result<(u16, String), String> {
    let mut full = String::new();
    let mut final_code = 0u16;
    for _ in 0..MAX_SMTP_CONT_LINES {
        let raw = read_line_bounded(reader, MAX_SMTP_LINE_BYTES, deadline, "SMTP")?;
        let line = String::from_utf8_lossy(&raw);
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.len() >= 3 {
            if let Ok(code) = trimmed[..3].parse::<u16>() {
                final_code = code;
                full.push_str(trimmed);
                full.push('\n');
                if trimmed.as_bytes().get(3) == Some(&b'-') {
                    continue;
                }
                return Ok((final_code, full));
            }
        }
        full.push_str(trimmed);
        full.push('\n');
        return Ok((final_code, full));
    }
    Err("SMTP response exceeded maximum continuation lines".to_string())
}

fn smtp_cmd(
    reader: &mut BufReader<ProtoStream>,
    cmd: &str,
    expected_codes: &[u16],
    deadline: Instant,
) -> Result<(u16, String), String> {
    reader.get_ref().apply_deadline(deadline, "SMTP")?;
    reader
        .get_mut()
        .write_all(cmd.as_bytes())
        .and_then(|_| reader.get_mut().flush())
        .map_err(|e| format!("SMTP write error: {e}"))?;
    let (code, resp) = read_smtp_response(reader, deadline)?;
    if !expected_codes.contains(&code) {
        return Err(format!("SMTP unexpected status {code}: {}", resp.trim()));
    }
    Ok((code, resp))
}

fn mime_type_for_filename(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".pdf") {
        "application/pdf"
    } else if lower.ends_with(".xlsx") {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    } else if lower.ends_with(".docx") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    } else if lower.ends_with(".pptx") {
        "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    } else if lower.ends_with(".csv") {
        "text/csv"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".md") || lower.ends_with(".txt") || lower.ends_with(".log") {
        "text/plain"
    } else {
        "application/octet-stream"
    }
}

fn wrap_base64_76(b64: &str) -> String {
    let mut out = String::with_capacity(b64.len() + (b64.len() / 76) * 2 + 2);
    for chunk in b64.as_bytes().chunks(76) {
        if let Ok(s) = std::str::from_utf8(chunk) {
            out.push_str(s);
            out.push_str("\r\n");
        }
    }
    out
}

fn dot_stuff_smtp_data(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len() + 64);
    let mut at_line_start = true;
    let mut i = 0;
    while i < raw.len() {
        let b = raw[i];
        if at_line_start && b == b'.' {
            out.push(b'.');
        }
        if b == b'\n' {
            if out.last().copied() != Some(b'\r') {
                out.push(b'\r');
            }
            out.push(b'\n');
            at_line_start = true;
        } else {
            out.push(b);
            at_line_start = false;
        }
        i += 1;
    }
    if !out.ends_with(b"\r\n") {
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b".\r\n");
    out
}

fn read_smtp_workspace_attachment(
    workspace_dir: &Path,
    rel: &str,
) -> Result<(String, Vec<u8>), String> {
    let clean = crate::tables::validate_workspace_rel_path(rel)?;
    let unresolved = sandbox_path_resolving(workspace_dir, &clean, false)?;
    if unresolved
        .symlink_metadata()
        .is_ok_and(|md| md.file_type().is_symlink())
    {
        return Err(format!("SMTP attachment cannot be a symlink: '{rel}'"));
    }
    let full = sandbox_path(workspace_dir, &clean)?;
    let md = std::fs::metadata(&full)
        .map_err(|e| format!("cannot read SMTP attachment '{rel}': {e}"))?;
    if !md.is_file() {
        return Err(format!("SMTP attachment '{rel}' is not a regular file"));
    }
    if md.len() > MAX_SMTP_ATTACHMENT_BYTES as u64 {
        return Err(format!(
            "SMTP attachment '{rel}' exceeds 10 MB limit ({} bytes)",
            md.len()
        ));
    }
    let bytes = std::fs::read(&full)
        .map_err(|e| format!("cannot read SMTP attachment '{rel}': {e}"))?;
    if bytes.len() > MAX_SMTP_ATTACHMENT_BYTES {
        return Err(format!(
            "SMTP attachment '{rel}' exceeds 10 MB limit ({} bytes)",
            bytes.len()
        ));
    }
    let fname = full
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "attachment.bin".to_string())
        .replace(['"', '\r', '\n', '/', '\\'], "_");
    Ok((fname, bytes))
}

fn build_rfc5322_mime_message(
    from_addr: &str,
    to_addrs: &[String],
    subject: &str,
    text_body: &str,
    attachments: &[(String, Vec<u8>)],
    now_ms: u64,
    seq: u64,
) -> Vec<u8> {
    let safe_subject: String = subject
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(240)
        .collect();
    let boundary = format!("----=_ErgonBoundary_{now_ms}_{seq}");
    let date_hdr = format_rfc2822_utc(now_ms);
    let to_hdr = to_addrs.join(", ");

    let mut msg = Vec::new();
    msg.extend_from_slice(format!("From: <{from_addr}>\r\n").as_bytes());
    msg.extend_from_slice(format!("To: {to_hdr}\r\n").as_bytes());
    msg.extend_from_slice(format!("Subject: {safe_subject}\r\n").as_bytes());
    msg.extend_from_slice(format!("Date: {date_hdr}\r\n").as_bytes());
    msg.extend_from_slice(format!("Message-ID: <{now_ms}.{seq}@ergon.local>\r\n").as_bytes());
    msg.extend_from_slice(b"MIME-Version: 1.0\r\n");
    msg.extend_from_slice(
        format!("Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n").as_bytes(),
    );

    // Text body part
    msg.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    msg.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
    msg.extend_from_slice(b"Content-Transfer-Encoding: 8bit\r\n\r\n");
    msg.extend_from_slice(text_body.replace("\r\n", "\n").replace('\n', "\r\n").as_bytes());
    msg.extend_from_slice(b"\r\n");

    for (fname, bytes) in attachments {
        let mime = mime_type_for_filename(fname);
        let b64 = wrap_base64_76(&tripwire::base64_encode(bytes));
        msg.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        msg.extend_from_slice(format!("Content-Type: {mime}; name=\"{fname}\"\r\n").as_bytes());
        msg.extend_from_slice(
            format!("Content-Disposition: attachment; filename=\"{fname}\"\r\n").as_bytes(),
        );
        msg.extend_from_slice(b"Content-Transfer-Encoding: base64\r\n\r\n");
        msg.extend_from_slice(b64.as_bytes());
    }
    msg.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    msg
}

fn resolve_smtp_secret(val_or_key: &str, secrets: &ConnectorSecretsStore) -> Result<String, String> {
    let trimmed = val_or_key.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if let Some(start) = trimmed.find("{{secret:") {
        let after = &trimmed[start + 9..];
        if let Some(end) = after.find("}}") {
            let key = after[..end].trim();
            return secrets
                .secrets
                .get(key)
                .cloned()
                .ok_or_else(|| format!("missing connector secret '{key}'"));
        }
    }
    secrets
        .secrets
        .get(trimmed)
        .cloned()
        .ok_or_else(|| format!("missing connector secret '{trimmed}'"))
}

pub fn execute_smtp_delivery(
    cfg: &Config,
    connector: &OutboundConnector,
    payload: &Value,
    attachment_rel: Option<&str>,
    thread_id: &str,
    decision_id: Option<&str>,
) -> Result<DeliveryLog, String> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    let now_ms = records::now_millis();
    let seq = next_seq();
    let del_id = format!("del-{now_ms}-{seq}");
    let mut secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    let safe_url = redact_with_store(&redact_url_userinfo(&connector.url), &secrets);

    let Some(smtp) = &connector.smtp_config else {
        return Err("smtp_relay connector is missing smtp_config".to_string());
    };

    let mut attachment_paths: Vec<String> = Vec::new();
    if let Some(rel) = attachment_rel.map(str::trim).filter(|s| !s.is_empty()) {
        attachment_paths.push(rel.to_string());
    }
    if let Some(f) = payload["file"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
        if !attachment_paths.iter().any(|p| p == f) {
            attachment_paths.push(f.to_string());
        }
    }
    if let Some(arr) = payload["attachments"].as_array() {
        for item in arr {
            if let Some(p) = item.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                if !attachment_paths.iter().any(|existing| existing == p) {
                    attachment_paths.push(p.to_string());
                }
            }
        }
    }

    let mut attachments = Vec::with_capacity(attachment_paths.len());
    for rel in &attachment_paths {
        attachments.push(read_smtp_workspace_attachment(&cfg.workspace_dir, rel)?);
    }

    let from_addr = payload["from"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(smtp.from_addr.as_str());
    let from_addr = validate_email_addr(from_addr, "smtp from_addr")?;

    let mut to_addrs: Vec<String> = Vec::new();
    if let Some(s) = payload["to"].as_str() {
        for part in s.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                to_addrs.push(validate_email_addr(t, "to")?);
            }
        }
    } else if let Some(arr) = payload["to"].as_array() {
        for item in arr {
            if let Some(s) = item.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                to_addrs.push(validate_email_addr(s, "to")?);
            }
        }
    }
    if to_addrs.is_empty() {
        for rcpt in &smtp.default_to {
            to_addrs.push(validate_email_addr(rcpt, "default_to")?);
        }
    }
    if to_addrs.is_empty() {
        return Err("SMTP delivery requires at least one recipient ('to' or smtp_config.default_to)".to_string());
    }

    let subject = payload["subject"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(connector.name.as_str())
        .to_string();
    let text_body = payload["body"]
        .as_str()
        .or_else(|| payload["text"].as_str())
        .or_else(|| payload["message"].as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if payload.is_null() {
                String::new()
            } else {
                serde_json::to_string_pretty(payload).unwrap_or_default()
            }
        });

    let mime_bytes = build_rfc5322_mime_message(
        &from_addr,
        &to_addrs,
        &subject,
        &text_body,
        &attachments,
        now_ms,
        seq,
    );
    let payload_sha256 = tripwire::sha256_hex(&mime_bytes);

    let record_fail = |status: &str, err_msg: String, secrets_ref: &ConnectorSecretsStore| {
        let safe_err = redact_with_store(&err_msg, secrets_ref);
        let log = DeliveryLog {
            id: del_id.clone(),
            connector_id: connector.id.clone(),
            connector_name: connector.name.clone(),
            kind: connector.kind.clone(),
            method: "SMTP".to_string(),
            url: safe_url.clone(),
            status: status.to_string(),
            http_status: None,
            decision_id: decision_id.map(str::to_string),
            thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
            payload_sha256: payload_sha256.clone(),
            payload_bytes: mime_bytes.len(),
            response_preview: String::new(),
            error: Some(safe_err.clone()),
            ts_ms: now_ms,
        };
        record_delivery_log(&cfg.data_dir, &connector.id, log);
        safe_err
    };

    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err(record_fail(
            "blocked",
            "blocked by Air-Gapped workspace policy: outbound connector delivery is disabled for this workspace".to_string(),
            &secrets,
        ));
    }

    if let Err(e) = validate_plain_loopback_security(&smtp.host, &smtp.security, "SMTP")
        .and_then(|_| tripwire::check_egress_host(&smtp.host, smtp.port, "connector"))
        .and_then(|_| {
            let p_str = String::from_utf8_lossy(&mime_bytes);
            tripwire::check_egress_payload(&p_str, "connector")
        })
    {
        return Err(record_fail("blocked", e, &secrets));
    }

    if connector.oauth_config.is_some() {
        if let Err(e) = ensure_fresh_oauth_token(&cfg.data_dir, &connector.id) {
            return Err(record_fail("error", e, &secrets));
        }
        secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    }

    let username = match resolve_smtp_secret(&smtp.username_secret_key, &secrets) {
        Ok(u) => u,
        Err(e) => return Err(record_fail("error", e, &secrets)),
    };
    let password = match resolve_smtp_secret(&smtp.password_secret_key, &secrets) {
        Ok(p) => p,
        Err(e) => return Err(record_fail("error", e, &secrets)),
    };

    let mut smtp_extras: Vec<String> = Vec::new();
    if !username.is_empty() {
        smtp_extras.push(username.clone());
    }
    if !password.is_empty() {
        smtp_extras.push(password.clone());
    }
    if !username.is_empty() || !password.is_empty() {
        let mut raw_plain = Vec::with_capacity(username.len() + password.len() + 2);
        raw_plain.push(0);
        raw_plain.extend_from_slice(username.as_bytes());
        raw_plain.push(0);
        raw_plain.extend_from_slice(password.as_bytes());
        smtp_extras.push(tripwire::base64_encode(&raw_plain));
        smtp_extras.push(tripwire::base64_encode(
            format!("user={username}\x01auth=Bearer {password}\x01\x01").as_bytes(),
        ));
        smtp_extras.push(tripwire::base64_encode(password.as_bytes()));
    }

    let run_smtp = || -> Result<String, String> {
        let deadline = Instant::now() + SMTP_SESSION_TIMEOUT;
        let initial_tls = smtp.security == "smtps";
        let stream = ProtoStream::connect(&smtp.host, smtp.port, initial_tls)?;
        stream.apply_deadline(deadline, "SMTP")?;
        let mut reader = BufReader::new(stream);
        let (greet_code, _) = read_smtp_response(&mut reader, deadline)?;
        if greet_code != 220 {
            return Err(format!("SMTP server rejected greeting ({greet_code})"));
        }
        let (_, mut ehlo_caps) = smtp_cmd(&mut reader, "EHLO ergon.local\r\n", &[250], deadline)?;
        if smtp.security == "starttls" {
            smtp_cmd(&mut reader, "STARTTLS\r\n", &[220], deadline)?;
            let inner = reader.into_inner().upgrade_starttls(&smtp.host)?;
            inner.apply_deadline(deadline, "SMTP")?;
            reader = BufReader::new(inner);
            let (_, caps_after) = smtp_cmd(&mut reader, "EHLO ergon.local\r\n", &[250], deadline)?;
            ehlo_caps = caps_after;
        }
        if !username.is_empty() || !password.is_empty() {
            let caps_upper = ehlo_caps.to_ascii_uppercase();
            if smtp.password_secret_key == "oauth_access_token" && caps_upper.contains("XOAUTH2") {
                let auth_str = format!("user={username}\x01auth=Bearer {password}\x01\x01");
                let b64 = tripwire::base64_encode(auth_str.as_bytes());
                smtp_cmd(&mut reader, &format!("AUTH XOAUTH2 {b64}\r\n"), &[235], deadline)?;
            } else if caps_upper.contains("LOGIN") && !caps_upper.contains("PLAIN") {
                smtp_cmd(&mut reader, "AUTH LOGIN\r\n", &[334], deadline)?;
                let u64 = tripwire::base64_encode(username.as_bytes());
                smtp_cmd(&mut reader, &format!("{u64}\r\n"), &[334], deadline)?;
                let p64 = tripwire::base64_encode(password.as_bytes());
                smtp_cmd(&mut reader, &format!("{p64}\r\n"), &[235], deadline)?;
            } else {
                let mut raw_auth = Vec::with_capacity(username.len() + password.len() + 2);
                raw_auth.push(0);
                raw_auth.extend_from_slice(username.as_bytes());
                raw_auth.push(0);
                raw_auth.extend_from_slice(password.as_bytes());
                let b64 = tripwire::base64_encode(&raw_auth);
                smtp_cmd(&mut reader, &format!("AUTH PLAIN {b64}\r\n"), &[235], deadline)?;
            }
        }
        smtp_cmd(&mut reader, &format!("MAIL FROM:<{from_addr}>\r\n"), &[250], deadline)?;
        for rcpt in &to_addrs {
            smtp_cmd(&mut reader, &format!("RCPT TO:<{rcpt}>\r\n"), &[250, 251], deadline)?;
        }
        smtp_cmd(&mut reader, "DATA\r\n", &[354], deadline)?;
        let stuffed = dot_stuff_smtp_data(&mime_bytes);
        reader.get_ref().apply_deadline(deadline, "SMTP")?;
        reader
            .get_mut()
            .write_all(&stuffed)
            .and_then(|_| reader.get_mut().flush())
            .map_err(|e| format!("SMTP write DATA error: {e}"))?;
        let (data_code, data_resp) = read_smtp_response(&mut reader, deadline)?;
        if data_code != 250 {
            return Err(format!("SMTP DATA rejected ({data_code}): {}", data_resp.trim()));
        }
        let _ = reader.get_mut().write_all(b"QUIT\r\n");
        let _ = reader.get_mut().flush();
        Ok(data_resp.trim().to_string())
    };

    match run_smtp() {
        Ok(resp_text) => {
            let preview = crate::truncate(
                &redact_with_store_extra(&resp_text, &secrets, &smtp_extras),
                MAX_RESPONSE_PREVIEW,
            );
            let log = DeliveryLog {
                id: del_id,
                connector_id: connector.id.clone(),
                connector_name: connector.name.clone(),
                kind: connector.kind.clone(),
                method: "SMTP".to_string(),
                url: safe_url,
                status: "ok".to_string(),
                http_status: Some(250),
                decision_id: decision_id.map(str::to_string),
                thread_id: (!thread_id.is_empty()).then(|| thread_id.to_string()),
                payload_sha256,
                payload_bytes: mime_bytes.len(),
                response_preview: preview,
                error: None,
                ts_ms: now_ms,
            };
            record_delivery_log(&cfg.data_dir, &connector.id, log.clone());
            Ok(log)
        }
        Err(e) => {
            let safe_err = redact_with_store_extra(&e, &secrets, &smtp_extras);
            Err(record_fail("error", safe_err, &secrets))
        }
    }
}

#[derive(Default)]
pub struct ImapPollState {
    last_poll_ms: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct ImapIngestEvent {
    pub trigger_id: String,
    pub trigger_name: String,
    pub routine_id: Option<String>,
    pub target_kind: String,
    pub trigger_file: String,
    pub attachments: Vec<String>,
    pub subject: String,
    pub from: String,
}

fn imap_quote(s: &str) -> Result<String, String> {
    if s.contains(['\r', '\n', '\0']) {
        return Err("IMAP argument contains invalid control characters".to_string());
    }
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("\"{escaped}\""))
}

fn read_line_bytes(
    reader: &mut BufReader<ProtoStream>,
    deadline: Instant,
) -> Result<Vec<u8>, String> {
    read_line_bounded(reader, MAX_IMAP_LINE_BYTES, deadline, "IMAP")
}

fn parse_imap_literal_len(line: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(line).ok()?.trim_end_matches(['\r', '\n']);
    if !s.ends_with('}') {
        return None;
    }
    let open = s.rfind('{')?;
    let inner = s[open + 1..s.len() - 1].trim().trim_end_matches('+');
    inner.parse::<usize>().ok()
}

fn read_imap_tagged(
    reader: &mut BufReader<ProtoStream>,
    tag: &str,
    deadline: Instant,
) -> Result<(Vec<String>, Vec<Vec<u8>>), String> {
    let mut lines = Vec::new();
    let mut literals = Vec::new();
    for _ in 0..MAX_IMAP_UNTAGGED_LINES {
        let line = read_line_bytes(reader, deadline)?;
        let lit_len = parse_imap_literal_len(&line);
        let text = String::from_utf8_lossy(&line)
            .trim_end_matches(['\r', '\n'])
            .to_string();
        if let Some(len) = lit_len {
            if len > MAX_IMAP_MESSAGE_BYTES {
                return Err(format!("IMAP literal exceeds {MAX_IMAP_MESSAGE_BYTES} byte limit"));
            }
            reader.get_ref().apply_deadline(deadline, "IMAP")?;
            let mut buf = vec![0u8; len];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("IMAP literal read error: {e}"))?;
            literals.push(buf);
            // Consume trailing response line after literal if present
            let trailing = read_line_bytes(reader, deadline)?;
            let tr_text = String::from_utf8_lossy(&trailing)
                .trim_end_matches(['\r', '\n'])
                .to_string();
            lines.push(text);
            if !tr_text.is_empty() {
                if let Some(rest) = tr_text.strip_prefix(tag) {
                    let rest_trim = rest.trim_start();
                    if rest_trim.starts_with("OK") {
                        return Ok((lines, literals));
                    }
                    if rest_trim.starts_with("NO") || rest_trim.starts_with("BAD") {
                        return Err(format!("IMAP {tag} failed: {rest_trim}"));
                    }
                }
                lines.push(tr_text);
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix(tag) {
            let rest_trim = rest.trim_start();
            if rest_trim.starts_with("OK") {
                return Ok((lines, literals));
            }
            if rest_trim.starts_with("NO") || rest_trim.starts_with("BAD") {
                return Err(format!("IMAP {tag} failed: {rest_trim}"));
            }
        }
        lines.push(text);
    }
    Err(format!("IMAP {tag} exceeded {MAX_IMAP_UNTAGGED_LINES} response lines"))
}

fn unfold_rfc5322_headers(header_block: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for raw_line in header_block.replace("\r\n", "\n").lines() {
        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            if let Some((_, val)) = out.last_mut() {
                val.push(' ');
                val.push_str(raw_line.trim());
            }
        } else if let Some((k, v)) = raw_line.split_once(':') {
            out.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    out
}

fn header_val<'a>(headers: &'a [(String, String)], key: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn parse_header_param(header: &str, param_name: &str) -> Option<String> {
    let target = format!("{param_name}=");
    for part in header.split(';').skip(1) {
        let trimmed = part.trim();
        let low = trimmed.to_ascii_lowercase();
        if low.starts_with(&target) {
            let val = trimmed[target.len()..].trim();
            return Some(val.trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

fn decode_quoted_printable(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'=' {
            if i + 2 <= input.len() && input[i + 1] == b'\r' && input.get(i + 2) == Some(&b'\n') {
                i += 3;
                continue;
            }
            if i + 1 < input.len() && input[i + 1] == b'\n' {
                i += 2;
                continue;
            }
            if i + 2 < input.len() {
                if let (Some(hi), Some(lo)) = (hex_nibble(input[i + 1]), hex_nibble(input[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(input[i]);
        i += 1;
    }
    out
}

fn decode_transfer_body(raw: &[u8], encoding: &str) -> Vec<u8> {
    match encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let s = String::from_utf8_lossy(raw);
            tripwire::base64_decode(&s).unwrap_or_else(|| raw.to_vec())
        }
        "quoted-printable" => decode_quoted_printable(raw),
        _ => raw.to_vec(),
    }
}

struct ParsedEmail {
    from: String,
    subject: String,
    date: String,
    message_id: String,
    body_text: String,
    attachments: Vec<(String, Vec<u8>)>,
}

fn split_headers_and_body(raw: &[u8]) -> (String, &[u8]) {
    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
        let hdr = String::from_utf8_lossy(&raw[..pos]).to_string();
        return (hdr, &raw[pos + 4..]);
    }
    if let Some(pos) = raw.windows(2).position(|w| w == b"\n\n") {
        let hdr = String::from_utf8_lossy(&raw[..pos]).to_string();
        return (hdr, &raw[pos + 2..]);
    }
    (String::new(), raw)
}

fn collect_mime_parts(
    raw: &[u8],
    text_parts: &mut Vec<String>,
    html_parts: &mut Vec<String>,
    attachments: &mut Vec<(String, Vec<u8>)>,
    depth: u8,
) {
    if depth > 6 {
        return;
    }
    let (hdr_str, body) = split_headers_and_body(raw);
    let headers = unfold_rfc5322_headers(&hdr_str);
    let ct = header_val(&headers, "content-type").unwrap_or("text/plain; charset=utf-8");
    let ct_low = ct.to_ascii_lowercase();
    let cte = header_val(&headers, "content-transfer-encoding").unwrap_or("7bit");
    let disp = header_val(&headers, "content-disposition").unwrap_or("");

    if ct_low.contains("multipart/") {
        if let Some(boundary) = parse_header_param(ct, "boundary") {
            let marker = format!("--{boundary}");
            let body_str = String::from_utf8_lossy(body);
            for section in body_str.split(&marker).skip(1) {
                if section.starts_with("--") {
                    break;
                }
                let trimmed = section
                    .strip_prefix("\r\n")
                    .or_else(|| section.strip_prefix('\n'))
                    .unwrap_or(section);
                let trimmed = trimmed
                    .strip_suffix("\r\n")
                    .or_else(|| trimmed.strip_suffix('\n'))
                    .unwrap_or(trimmed);
                if !trimmed.is_empty() {
                    collect_mime_parts(
                        trimmed.as_bytes(),
                        text_parts,
                        html_parts,
                        attachments,
                        depth + 1,
                    );
                }
            }
            return;
        }
    }

    let decoded = decode_transfer_body(body, cte);
    let filename = parse_header_param(disp, "filename")
        .or_else(|| parse_header_param(ct, "name"));
    let is_attachment = disp.to_ascii_lowercase().starts_with("attachment")
        || (filename.is_some()
            && !ct_low.starts_with("text/plain")
            && !ct_low.starts_with("text/html"));

    if is_attachment {
        if decoded.len() <= MAX_SMTP_ATTACHMENT_BYTES {
            let fname = filename.unwrap_or_else(|| format!("attachment-{}.bin", next_seq()));
            attachments.push((fname, decoded));
        }
    } else if ct_low.starts_with("text/html") {
        html_parts.push(String::from_utf8_lossy(&decoded).to_string());
    } else {
        text_parts.push(String::from_utf8_lossy(&decoded).to_string());
    }
}

fn parse_rfc5322_email(raw: &[u8]) -> ParsedEmail {
    let (hdr_str, _) = split_headers_and_body(raw);
    let headers = unfold_rfc5322_headers(&hdr_str);
    let from = header_val(&headers, "from")
        .unwrap_or("(unknown sender)")
        .to_string();
    let subject = header_val(&headers, "subject")
        .unwrap_or("(no subject)")
        .to_string();
    let date = header_val(&headers, "date").unwrap_or("").to_string();
    let message_id = header_val(&headers, "message-id")
        .unwrap_or("")
        .to_string();

    let mut text_parts = Vec::new();
    let mut html_parts = Vec::new();
    let mut attachments = Vec::new();
    collect_mime_parts(raw, &mut text_parts, &mut html_parts, &mut attachments, 0);

    let body_text = if !text_parts.is_empty() {
        text_parts.join("\n\n")
    } else if !html_parts.is_empty() {
        crate::tools::html_to_text(&html_parts.join("\n"))
    } else {
        String::new()
    };

    ParsedEmail {
        from,
        subject,
        date,
        message_id,
        body_text,
        attachments,
    }
}

fn write_intake_file_unique(
    workspace_dir: &Path,
    raw_name: &str,
    bytes: &[u8],
) -> Result<(String, String), String> {
    let sha = tripwire::sha256_hex(bytes);
    let safe_name = sanitize_intake_filename(raw_name);
    let mut rel = format!("intake/{safe_name}");
    let mut dest = sandbox_path_resolving(workspace_dir, &rel, false)?;
    if dest
        .symlink_metadata()
        .is_ok_and(|md| md.file_type().is_symlink())
        || dest.exists()
    {
        rel = format!("intake/{}", disambiguate_filename(&safe_name, &sha));
        dest = sandbox_path_resolving(workspace_dir, &rel, false)?;
        if dest
            .symlink_metadata()
            .is_ok_and(|md| md.file_type().is_symlink())
        {
            return Err("cannot overwrite symlink in intake".to_string());
        }
    }
    sandbox_path(workspace_dir, &rel)?;
    atomic_write_bytes(&dest, bytes).map_err(|e| format!("write intake file: {e}"))?;
    Ok((rel, sha))
}

fn poll_one_imap_trigger(cfg: &Config, trg: &TriggerRule) -> Result<Vec<ImapIngestEvent>, String> {
    let Some(imap) = &trg.imap_config else {
        return Ok(Vec::new());
    };
    validate_plain_loopback_security(&imap.host, &imap.security, "IMAP")?;
    tripwire::check_egress_host(&imap.host, imap.port, "connector")?;

    let secrets = with_connectors_lock(|| ConnectorSecretsStore::load_unlocked(&cfg.data_dir));
    let username = resolve_smtp_secret(&imap.username_secret_key, &secrets)?;
    let password = resolve_smtp_secret(&imap.password_secret_key, &secrets)?;
    let mut imap_extras: Vec<String> = Vec::new();
    if !username.is_empty() {
        imap_extras.push(username.clone());
    }
    if !password.is_empty() {
        imap_extras.push(password.clone());
    }

    let run_imap = || -> Result<Vec<ImapIngestEvent>, String> {
        let deadline = Instant::now() + IMAP_SESSION_TIMEOUT;
        let initial_tls = imap.security == "imaps";
        let stream = ProtoStream::connect(&imap.host, imap.port, initial_tls)?;
        stream.apply_deadline(deadline, "IMAP")?;
        let mut reader = BufReader::new(stream);

        // Untagged greeting (* OK ...)
        let greeting =
            String::from_utf8_lossy(&read_line_bytes(&mut reader, deadline)?).to_string();
        if !greeting.trim_start().starts_with('*') {
            return Err(format!("unexpected IMAP greeting: {}", greeting.trim()));
        }

        if imap.security == "starttls" {
            reader.get_ref().apply_deadline(deadline, "IMAP")?;
            reader
                .get_mut()
                .write_all(b"A001 STARTTLS\r\n")
                .and_then(|_| reader.get_mut().flush())
                .map_err(|e| format!("IMAP STARTTLS write: {e}"))?;
            let _ = read_imap_tagged(&mut reader, "A001", deadline)?;
            let inner = reader.into_inner().upgrade_starttls(&imap.host)?;
            inner.apply_deadline(deadline, "IMAP")?;
            reader = BufReader::new(inner);
        }

        let login_cmd = format!(
            "A002 LOGIN {} {}\r\n",
            imap_quote(&username)?,
            imap_quote(&password)?
        );
        reader.get_ref().apply_deadline(deadline, "IMAP")?;
        reader
            .get_mut()
            .write_all(login_cmd.as_bytes())
            .and_then(|_| reader.get_mut().flush())
            .map_err(|e| format!("IMAP LOGIN write: {e}"))?;
        let _ = read_imap_tagged(&mut reader, "A002", deadline)?;

        let select_cmd = format!("A003 SELECT {}\r\n", imap_quote(&imap.folder)?);
        reader.get_ref().apply_deadline(deadline, "IMAP")?;
        reader
            .get_mut()
            .write_all(select_cmd.as_bytes())
            .and_then(|_| reader.get_mut().flush())
            .map_err(|e| format!("IMAP SELECT write: {e}"))?;
        let _ = read_imap_tagged(&mut reader, "A003", deadline)?;

        reader.get_ref().apply_deadline(deadline, "IMAP")?;
        reader
            .get_mut()
            .write_all(b"A004 UID SEARCH UNSEEN\r\n")
            .and_then(|_| reader.get_mut().flush())
            .map_err(|e| format!("IMAP SEARCH write: {e}"))?;
        let (search_lines, _) = read_imap_tagged(&mut reader, "A004", deadline)?;
        let mut uids: Vec<String> = Vec::new();
        for line in search_lines {
            if let Some(rest) = line.strip_prefix("* SEARCH") {
                for tok in rest.split_whitespace() {
                    if tok.bytes().all(|b| b.is_ascii_digit()) {
                        uids.push(tok.to_string());
                    }
                }
            }
        }

        let mut events = Vec::new();
        let mut seq_num = 5u32;
        for uid in uids.into_iter().take(MAX_IMAP_MESSAGES_PER_POLL) {
            let tag = format!("A{seq_num:03}");
            seq_num += 1;
            let fetch_cmd = format!("{tag} UID FETCH {uid} BODY.PEEK[]\r\n");
            reader.get_ref().apply_deadline(deadline, "IMAP")?;
            reader
                .get_mut()
                .write_all(fetch_cmd.as_bytes())
                .and_then(|_| reader.get_mut().flush())
                .map_err(|e| format!("IMAP FETCH write: {e}"))?;
            let (_, literals) = read_imap_tagged(&mut reader, &tag, deadline)?;
            let Some(raw_msg) = literals.into_iter().next() else {
                continue;
            };
            let raw_sha = tripwire::sha256_hex(&raw_msg);
            let parsed = parse_rfc5322_email(&raw_msg);

            if let Some(filter) = imap.subject_filter.as_deref() {
                if !parsed
                    .subject
                    .to_ascii_lowercase()
                    .contains(&filter.to_ascii_lowercase())
                {
                    continue;
                }
            }

            let already_ingested = with_connectors_lock(|| {
                ConnectorsStore::load_unlocked(&cfg.data_dir)
                    .intake_manifest
                    .iter()
                    .any(|m| m.sha256 == raw_sha)
            });
            if already_ingested {
                continue;
            }

            let now_ms = records::now_millis();
            let slug: String = parsed
                .subject
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                        c
                    } else {
                        '_'
                    }
                })
                .take(48)
                .collect();
            let slug = slug.trim_matches('_');
            let md_filename = if slug.is_empty() {
                format!("imap-{now_ms}-uid{uid}.md")
            } else {
                format!("imap-{now_ms}-{slug}.md")
            };
            let md_content = format!(
                "# {}\n\nFrom: {}\nDate: {}\nMessage-ID: {}\nFolder: {}\n\n---\n\n{}\n",
                parsed.subject.trim(),
                parsed.from.trim(),
                parsed.date.trim(),
                parsed.message_id.trim(),
                imap.folder,
                parsed.body_text.trim()
            );

            let (trigger_file, _) =
                write_intake_file_unique(&cfg.workspace_dir, &md_filename, md_content.as_bytes())?;
            let mut written_attachments = Vec::new();
            let mut att_manifest = Vec::new();
            for (att_name, att_bytes) in &parsed.attachments {
                if let Ok((att_rel, att_sha)) =
                    write_intake_file_unique(&cfg.workspace_dir, att_name, att_bytes)
                {
                    written_attachments.push(att_rel.clone());
                    att_manifest.push(IntakeManifestEntry {
                        sha256: att_sha,
                        filename: att_rel,
                        source_path: format!(
                            "imap://{}/{}/uid/{uid}/{}",
                            imap.host, imap.folder, att_name
                        ),
                        trigger_id: trg.id.clone(),
                        ingested_ms: now_ms,
                    });
                }
            }

            with_connectors_lock(|| {
                let mut store = ConnectorsStore::load_unlocked(&cfg.data_dir);
                store.push_manifest(IntakeManifestEntry {
                    sha256: raw_sha.clone(),
                    filename: trigger_file.clone(),
                    source_path: format!("imap://{}/{}/uid/{uid}", imap.host, imap.folder),
                    trigger_id: trg.id.clone(),
                    ingested_ms: now_ms,
                });
                for entry in att_manifest {
                    store.push_manifest(entry);
                }
                if let Some(t) = store.triggers.iter_mut().find(|t| t.id == trg.id) {
                    t.trigger_count = t.trigger_count.saturating_add(1);
                    t.last_triggered_ms = Some(now_ms);
                    t.updated_ms = now_ms;
                }
                store.save_unlocked(&cfg.data_dir)
            })?;

            if imap.mark_seen {
                let stag = format!("A{seq_num:03}");
                seq_num += 1;
                let store_cmd = format!("{stag} UID STORE {uid} +FLAGS.SILENT (\\Seen)\r\n");
                let _ = reader
                    .get_mut()
                    .write_all(store_cmd.as_bytes())
                    .and_then(|_| reader.get_mut().flush());
                let _ = read_imap_tagged(&mut reader, &stag, deadline);
            }

            events.push(ImapIngestEvent {
                trigger_id: trg.id.clone(),
                trigger_name: trg.name.clone(),
                routine_id: trg.routine_id.clone(),
                target_kind: trg.target_kind.clone(),
                trigger_file,
                attachments: written_attachments,
                subject: parsed.subject,
                from: parsed.from,
            });
        }

        let _ = reader.get_mut().write_all(b"A999 LOGOUT\r\n");
        let _ = reader.get_mut().flush();
        Ok(events)
    };

    run_imap().map_err(|e| redact_with_store_extra(&e, &secrets, &imap_extras))
}

/// Poll all enabled `imap_poll` triggers whose `poll_interval_secs` has elapsed.
pub fn poll_imap_once(cfg: &Config, poll_state: &mut ImapPollState) -> Vec<ImapIngestEvent> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Vec::new();
    }
    let triggers: Vec<TriggerRule> = with_connectors_lock(|| {
        ConnectorsStore::load_unlocked(&cfg.data_dir)
            .triggers
            .into_iter()
            .filter(|t| t.kind == "imap_poll" && t.enabled)
            .collect()
    });
    let mut events = Vec::new();
    let now_ms = records::now_millis();
    for trg in triggers {
        let interval_ms = trg
            .imap_config
            .as_ref()
            .map(|i| i.poll_interval_secs.saturating_mul(1000))
            .unwrap_or(60_000);
        let due = poll_state
            .last_poll_ms
            .get(&trg.id)
            .is_none_or(|last| now_ms.saturating_sub(*last) >= interval_ms);
        if !due {
            continue;
        }
        poll_state.last_poll_ms.insert(trg.id.clone(), now_ms);
        if let Ok(mut evs) = poll_one_imap_trigger(cfg, &trg) {
            events.append(&mut evs);
        }
    }
    events
}

/// Immediately poll one `imap_poll` trigger by id (used by tests and manual triggers).
pub fn poll_imap_trigger_now(
    cfg: &Config,
    trigger_id: &str,
) -> Result<Vec<ImapIngestEvent>, String> {
    let policy = crate::load_workspace_policy(&cfg.data_dir);
    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err("blocked by Air-Gapped workspace policy: IMAP polling is disabled".to_string());
    }
    let trg = with_connectors_lock(|| {
        ConnectorsStore::load_unlocked(&cfg.data_dir)
            .triggers
            .into_iter()
            .find(|t| t.id == trigger_id && t.kind == "imap_poll")
    })
    .ok_or_else(|| format!("unknown imap_poll trigger '{trigger_id}'"))?;
    poll_one_imap_trigger(cfg, &trg)
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

pub fn load_connector_kind(person_data_dir: &Path, connector_id: &str) -> Option<String> {
    load_connector(person_data_dir, connector_id)
        .ok()
        .map(|c| c.kind)
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

    // Validate target URL/host and payload before parking or delivering.
    let payload_text = String::from_utf8_lossy(&payload_bytes);
    tripwire::check_egress(&connector.url, "connector")
        .and_then(|_| {
            if let Some(smtp) = &connector.smtp_config {
                tripwire::check_egress_host(&smtp.host, smtp.port, "connector")
            } else {
                let (uhost, uport) = parse_url_host_port(&connector.url, 443);
                tripwire::check_egress_host(&uhost, uport, "connector")
            }
        })
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
            if connector.kind == "smtp_relay" { "SMTP" } else { connector.method.as_str() },
            connector.name,
            safe_url
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
                "method": if connector.kind == "smtp_relay" { "SMTP" } else { connector.method.as_str() },
                "url": safe_url,
                "attachment_rel": attachment_rel,
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
            method: if connector.kind == "smtp_relay" { "SMTP".to_string() } else { connector.method.clone() },
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

    let log = if connector.kind == "smtp_relay" {
        execute_smtp_delivery(cfg, &connector, payload, attachment_rel, thread_id, None)?
    } else {
        execute_http_delivery(cfg, &connector, payload, thread_id, None)?
    };
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

static IN_FLIGHT_DELIVERY_SETTLE: OnceLock<(Mutex<HashSet<String>>, Condvar)> = OnceLock::new();

fn settling_bus() -> &'static (Mutex<HashSet<String>>, Condvar) {
    IN_FLIGHT_DELIVERY_SETTLE.get_or_init(|| (Mutex::new(HashSet::new()), Condvar::new()))
}

struct DeliverySettleGuard<'a> {
    key: String,
    bus: &'a (Mutex<HashSet<String>>, Condvar),
}

impl<'a> Drop for DeliverySettleGuard<'a> {
    fn drop(&mut self) {
        let (mtx, cv) = self.bus;
        let mut set = mtx.lock().unwrap_or_else(|p| p.into_inner());
        set.remove(&self.key);
        cv.notify_all();
    }
}

fn acquire_delivery_settle_guard(data_dir: &Path, decision_id: &str) -> DeliverySettleGuard<'static> {
    let bus = settling_bus();
    let key = format!("{}::{}", data_dir.display(), decision_id);
    let (mtx, cv) = bus;
    let mut set = mtx.lock().unwrap_or_else(|p| p.into_inner());
    while set.contains(&key) {
        set = cv.wait(set).unwrap_or_else(|p| p.into_inner());
    }
    set.insert(key.clone());
    DeliverySettleGuard { key, bus }
}

fn existing_settled_delivery_outcome(data_dir: &Path, decision_id: &str) -> Option<String> {
    if decision_id.is_empty() {
        return None;
    }
    with_connectors_lock(|| {
        let store = ConnectorsStore::load_unlocked(data_dir);
        store
            .delivery_log
            .iter()
            .rev()
            .find(|d| {
                d.decision_id.as_deref() == Some(decision_id)
                    && matches!(d.status.as_str(), "ok" | "delivered" | "denied" | "error" | "blocked")
            })
            .map(|d| match d.status.as_str() {
                "denied" => "You said no — outbound delivery was cancelled.".to_string(),
                "error" | "blocked" => format!(
                    "Approved, but outbound delivery failed: {}",
                    d.error.as_deref().unwrap_or("delivery failed")
                ),
                _ => format!(
                    "Approved and delivered via \"{}\" (HTTP {}, {} bytes).",
                    d.connector_name,
                    d.http_status.unwrap_or(200),
                    d.payload_bytes
                ),
            })
    })
}

/// Wait for any in-flight `settle_delivery_decision` on `decision_id` to finish
/// and return the persisted decision outcome (or delivery log summary).
pub fn wait_for_delivery_settlement(data_dir: &Path, decision_id: &str) -> Option<String> {
    if decision_id.is_empty() {
        return None;
    }
    let _guard = acquire_delivery_settle_guard(data_dir, decision_id);
    if let Some(existing) = existing_settled_delivery_outcome(data_dir, decision_id) {
        return Some(existing);
    }
    records::load_decisions(data_dir)
        .into_iter()
        .find(|d| d["id"].as_str() == Some(decision_id))
        .and_then(|d| d["outcome"].as_str().map(str::to_string))
}

/// Execute an approved `connector_delivery` decision (called from `records::resolve_decision`
/// outside `RECORDS_LOCK`). Idempotent per `decision_id` so concurrent/repeated settlement
/// cannot double-deliver. Returns the final human-readable outcome string.
pub fn settle_delivery_decision(cfg: &Config, decision: &Value) -> String {
    let decision_id = decision["id"].as_str().unwrap_or("");
    let _guard = (!decision_id.is_empty())
        .then(|| acquire_delivery_settle_guard(&cfg.data_dir, decision_id));
    if let Some(existing) = existing_settled_delivery_outcome(&cfg.data_dir, decision_id) {
        return existing;
    }
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
    let attachment_rel = decision["attachment_rel"]
        .as_str()
        .or_else(|| decision["path"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let res = if connector.kind == "smtp_relay" {
        execute_smtp_delivery(cfg, &connector, payload, attachment_rel, thread_id, Some(decision_id))
    } else {
        execute_http_delivery(cfg, &connector, payload, thread_id, Some(decision_id))
    };
    match res {
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
