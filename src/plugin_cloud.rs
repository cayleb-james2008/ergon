//! Native Browser OAuth2 (PKCE S256) & Cloud REST Adapters for Google Workspace
//! and Microsoft 365 & Outlook inside Ergon's 18-plugin workplace library.
//!
//! Security & confinement invariants:
//! - Company OAuth app `client_secret` and per-employee `access_token` /
//!   `refresh_token` are ALWAYS stored encrypted in `secrets.enc.json` via
//!   `crate::secret_store` (`0600` POSIX + AES-256-GCM). Only `vault:...`
//!   references appear in JSON metadata files.
//! - Authorization uses CSPRNG 192-bit `state` + 256-bit PKCE `code_verifier`
//!   (`S256`) bound server-side to `(person_id, provider, redirect_uri)` with
//!   a 10-minute TTL and single-use consumption.
//! - Every outbound HTTP call enforces `Air-Gapped` workspace policy checks,
//!   `tripwire::check_egress_for(..., "connector")` + `check_egress_host_for`,
//!   `ureq` `.redirects(0)`, and `tripwire::check_egress_payload` on writes.
//! - Local workspace reads/writes go strictly through `crate::tools::sandbox_path`
//!   and `crate::tools::execute_tool`.

use crate::auth::{base64url_encode_nopad, secure_random_bytes, url_encode_component};
use crate::tools::{atomic_write_bytes_with_mode, execute_tool, sandbox_path};
use crate::{parse_url_host_port, records, secret_store, tripwire, Config};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const APPS_FILE: &str = "plugin_oauth_apps.json";
const CONNECTIONS_FILE: &str = "plugin_connections.json";
const PENDING_TTL_MS: u64 = 10 * 60 * 1000;
const MAX_PENDING_FLOWS: usize = 64;
const TOKEN_REFRESH_LEEWAY_MS: u64 = 90 * 1000;

const GOOGLE_DEFAULT_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_DEFAULT_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DEFAULT_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const GOOGLE_DEFAULT_SCOPES: &str = "openid email profile https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/calendar https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/documents https://www.googleapis.com/auth/spreadsheets https://www.googleapis.com/auth/presentations";

const MS_DEFAULT_TENANT: &str = "common";
const MS_DEFAULT_USERINFO_URL: &str = "https://graph.microsoft.com/v1.0/me";
const MS_DEFAULT_API_BASE: &str = "https://graph.microsoft.com/v1.0";
const MS_DEFAULT_SCOPES: &str = "openid email profile offline_access User.Read Mail.ReadWrite Mail.Send Calendars.ReadWrite Files.ReadWrite ChannelMessage.Read.All ChannelMessage.Send Team.ReadBasic.All";

const GOOGLE_PLUGIN_IDS: &[&str] = &[
    "google-gmail",
    "google-calendar",
    "google-drive",
    "google-docs",
    "google-sheets",
    "google-slides",
];

const MICROSOFT_PLUGIN_IDS: &[&str] = &[
    "outlook-mail",
    "outlook-calendar",
    "onedrive-sharepoint",
    "microsoft-teams",
    "office-excel",
];

static CLOUD_LOCK: Mutex<()> = Mutex::new(());
static PENDING_FLOWS: OnceLock<Mutex<HashMap<String, PendingPluginOAuth>>> = OnceLock::new();

fn with_cloud_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = CLOUD_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

fn pending_map() -> &'static Mutex<HashMap<String, PendingPluginOAuth>> {
    PENDING_FLOWS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Debug)]
struct PendingPluginOAuth {
    provider: String,
    person_id: String,
    person_data_dir: PathBuf,
    code_verifier: String,
    redirect_uri: String,
    return_to: String,
    created_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub struct OAuthAppConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret_ref: Option<String>,
    pub tenant: String,
    pub auth_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub api_base_url: Option<String>,
    pub scopes: Option<String>,
    pub updated_ms: u64,
    pub from_env: bool,
}

#[derive(Clone, Debug)]
pub struct PluginConnectionRecord {
    pub provider: String,
    pub account_email: Option<String>,
    pub account_name: Option<String>,
    pub scopes: String,
    pub expires_at_ms: u64,
    pub connected_at_ms: u64,
    pub access_token_ref: String,
    pub refresh_token_ref: Option<String>,
}

pub fn normalize_provider(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "google" | "google_workspace" | "workspace" => Some("google"),
        "microsoft" | "m365" | "microsoft365" | "outlook" => Some("microsoft"),
        _ => None,
    }
}

pub fn provider_for_plugin(plugin_id: &str) -> Option<&'static str> {
    let key = plugin_id.trim();
    if GOOGLE_PLUGIN_IDS.contains(&key) {
        Some("google")
    } else if MICROSOFT_PLUGIN_IDS.contains(&key) {
        Some("microsoft")
    } else {
        None
    }
}

pub fn suite_label(provider: &str) -> &'static str {
    match provider {
        "google" => "Google Workspace",
        "microsoft" => "Microsoft 365 & Outlook",
        _ => "Cloud Suite",
    }
}

fn plugin_ids_for_provider(provider: &str) -> &'static [&'static str] {
    match provider {
        "google" => GOOGLE_PLUGIN_IDS,
        "microsoft" => MICROSOFT_PLUGIN_IDS,
        _ => &[],
    }
}

fn oauth_apps_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join(APPS_FILE)
}

fn connections_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join(CONNECTIONS_FILE)
}

fn app_secret_key(provider: &str) -> String {
    format!("plugin_oauth:{provider}:client_secret")
}

fn access_secret_key(provider: &str) -> String {
    format!("plugin_oauth:{provider}:access_token")
}

fn refresh_secret_key(provider: &str) -> String {
    format!("plugin_oauth:{provider}:refresh_token")
}

fn default_tenant(raw: Option<&str>) -> String {
    raw.map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.'))
        .unwrap_or(MS_DEFAULT_TENANT)
        .to_string()
}

fn resolved_auth_url(app: &OAuthAppConfig) -> String {
    if let Some(u) = app.auth_url.as_deref().filter(|s| !s.is_empty()) {
        return u.to_string();
    }
    match app.provider.as_str() {
        "google" => GOOGLE_DEFAULT_AUTH_URL.to_string(),
        _ => format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            default_tenant(Some(&app.tenant))
        ),
    }
}

fn resolved_token_url(app: &OAuthAppConfig) -> String {
    if let Some(u) = app.token_url.as_deref().filter(|s| !s.is_empty()) {
        return u.to_string();
    }
    match app.provider.as_str() {
        "google" => GOOGLE_DEFAULT_TOKEN_URL.to_string(),
        _ => format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            default_tenant(Some(&app.tenant))
        ),
    }
}

fn resolved_userinfo_url(app: &OAuthAppConfig) -> String {
    if let Some(u) = app.userinfo_url.as_deref().filter(|s| !s.is_empty()) {
        return u.to_string();
    }
    match app.provider.as_str() {
        "google" => GOOGLE_DEFAULT_USERINFO_URL.to_string(),
        _ => MS_DEFAULT_USERINFO_URL.to_string(),
    }
}

fn resolved_scopes(app: &OAuthAppConfig) -> String {
    if let Some(s) = app.scopes.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return s.to_string();
    }
    match app.provider.as_str() {
        "google" => GOOGLE_DEFAULT_SCOPES.to_string(),
        _ => MS_DEFAULT_SCOPES.to_string(),
    }
}

fn default_callback_uri(request_origin: &str) -> String {
    format!(
        "{}/api/plugins/oauth/callback",
        request_origin.trim_end_matches('/')
    )
}

fn load_apps_unlocked(root_data_dir: &Path) -> BTreeMap<String, OAuthAppConfig> {
    let mut map = BTreeMap::new();
    let path = oauth_apps_path(root_data_dir);
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(val) = serde_json::from_str::<Value>(&raw) {
            if let Some(obj) = val["apps"].as_object() {
                for (k, v) in obj {
                    let Some(prov) = normalize_provider(k) else {
                        continue;
                    };
                    let app = OAuthAppConfig {
                        provider: prov.to_string(),
                        client_id: v["client_id"].as_str().unwrap_or("").trim().to_string(),
                        client_secret_ref: v["client_secret_ref"]
                            .as_str()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string),
                        tenant: default_tenant(v["tenant"].as_str()),
                        auth_url: v["auth_url"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                        token_url: v["token_url"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                        userinfo_url: v["userinfo_url"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                        api_base_url: v["api_base_url"].as_str().map(|s| s.trim().trim_end_matches('/').to_string()).filter(|s| !s.is_empty()),
                        scopes: v["scopes"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                        updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
                        from_env: false,
                    };
                    map.insert(prov.to_string(), app);
                }
            }
        }
    }
    map
}

fn save_apps_unlocked(root_data_dir: &Path, map: &BTreeMap<String, OAuthAppConfig>) -> Result<(), String> {
    let mut obj = serde_json::Map::new();
    for (prov, app) in map {
        obj.insert(
            prov.clone(),
            json!({
                "provider": app.provider,
                "client_id": app.client_id,
                "client_secret_ref": app.client_secret_ref,
                "tenant": app.tenant,
                "auth_url": app.auth_url,
                "token_url": app.token_url,
                "userinfo_url": app.userinfo_url,
                "api_base_url": app.api_base_url,
                "scopes": app.scopes,
                "updated_ms": app.updated_ms,
            }),
        );
    }
    let body = serde_json::to_string_pretty(&json!({ "apps": obj }))
        .map_err(|e| format!("serialize plugin_oauth_apps: {e}"))?;
    atomic_write_bytes_with_mode(&oauth_apps_path(root_data_dir), body.as_bytes(), Some(0o600))
        .map_err(|e| format!("write plugin_oauth_apps.json: {e}"))
}

fn resolve_app_config(root_data_dir: &Path, provider: &str) -> OAuthAppConfig {
    let mut cfg = with_cloud_lock(|| load_apps_unlocked(root_data_dir))
        .remove(provider)
        .unwrap_or_else(|| OAuthAppConfig {
            provider: provider.to_string(),
            tenant: MS_DEFAULT_TENANT.to_string(),
            ..Default::default()
        });
    if cfg.client_id.is_empty() {
        let (id_var, tenant_var) = match provider {
            "google" => ("ERGON_GOOGLE_OAUTH_CLIENT_ID", ""),
            _ => ("ERGON_MICROSOFT_OAUTH_CLIENT_ID", "ERGON_MICROSOFT_OAUTH_TENANT"),
        };
        if let Ok(cid) = std::env::var(id_var) {
            let trimmed = cid.trim().to_string();
            if !trimmed.is_empty() {
                cfg.client_id = trimmed;
                cfg.from_env = true;
                if !tenant_var.is_empty() {
                    if let Ok(ten) = std::env::var(tenant_var) {
                        cfg.tenant = default_tenant(Some(&ten));
                    }
                }
            }
        }
    }
    cfg
}

fn resolve_client_secret(root_data_dir: &Path, app: &OAuthAppConfig) -> Option<String> {
    if let Some(ref vref) = app.client_secret_ref {
        if let Some(key) = secret_store::parse_vault_ref(vref) {
            if let Ok(Some(sec)) = secret_store::get_secret(root_data_dir, key) {
                if !sec.trim().is_empty() {
                    return Some(sec);
                }
            }
        }
    }
    if app.from_env {
        let var = match app.provider.as_str() {
            "google" => "ERGON_GOOGLE_OAUTH_CLIENT_SECRET",
            _ => "ERGON_MICROSOFT_OAUTH_CLIENT_SECRET",
        };
        if let Ok(val) = std::env::var(var) {
            let t = val.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

fn load_connections_unlocked(person_data_dir: &Path) -> BTreeMap<String, PluginConnectionRecord> {
    let mut map = BTreeMap::new();
    let path = connections_path(person_data_dir);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return map;
    };
    let Ok(val) = serde_json::from_str::<Value>(&raw) else {
        return map;
    };
    if let Some(obj) = val["connections"].as_object() {
        for (k, v) in obj {
            let Some(prov) = normalize_provider(k) else {
                continue;
            };
            let Some(at_ref) = v["access_token_ref"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
                continue;
            };
            map.insert(
                prov.to_string(),
                PluginConnectionRecord {
                    provider: prov.to_string(),
                    account_email: v["account_email"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                    account_name: v["account_name"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                    scopes: v["scopes"].as_str().unwrap_or("").to_string(),
                    expires_at_ms: v["expires_at_ms"].as_u64().unwrap_or(0),
                    connected_at_ms: v["connected_at_ms"].as_u64().unwrap_or(0),
                    access_token_ref: at_ref.to_string(),
                    refresh_token_ref: v["refresh_token_ref"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                },
            );
        }
    }
    map
}

fn save_connections_unlocked(
    person_data_dir: &Path,
    map: &BTreeMap<String, PluginConnectionRecord>,
) -> Result<(), String> {
    let mut obj = serde_json::Map::new();
    for (prov, rec) in map {
        obj.insert(
            prov.clone(),
            json!({
                "provider": rec.provider,
                "account_email": rec.account_email,
                "account_name": rec.account_name,
                "scopes": rec.scopes,
                "expires_at_ms": rec.expires_at_ms,
                "connected_at_ms": rec.connected_at_ms,
                "access_token_ref": rec.access_token_ref,
                "refresh_token_ref": rec.refresh_token_ref,
            }),
        );
    }
    let body = serde_json::to_string_pretty(&json!({ "connections": obj }))
        .map_err(|e| format!("serialize plugin_connections: {e}"))?;
    atomic_write_bytes_with_mode(&connections_path(person_data_dir), body.as_bytes(), Some(0o600))
        .map_err(|e| format!("write plugin_connections.json: {e}"))
}

fn validate_endpoint_url(data_dir: &Path, url: &str, channel: &str) -> Result<(), String> {
    let trimmed = url.trim();
    let allowed = trimmed.starts_with("https://")
        || (trimmed.starts_with("http://") && crate::is_loopback_url(trimmed));
    if !allowed {
        return Err("OAuth endpoint URL must use https:// (or loopback http:// for local testing)".to_string());
    }
    tripwire::check_egress_for(data_dir, trimmed, channel)?;
    if let Some((host, port)) = parse_url_host_port(trimmed) {
        tripwire::check_egress_host_for(data_dir, &host, port, channel)?;
    }
    Ok(())
}

fn build_http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(10))
        .build()
}

fn read_ureq_result(res: Result<ureq::Response, ureq::Error>, ctx: &str) -> Result<Value, String> {
    match res {
        Ok(resp) => {
            let status = resp.status();
            let raw = resp
                .into_string()
                .map_err(|e| format!("{ctx}: failed reading response body: {e}"))?;
            if status >= 400 {
                return Err(format!("{ctx}: HTTP {status}: {}", crate::truncate(&raw, 240)));
            }
            if raw.trim().is_empty() {
                return Ok(json!({ "ok": true, "status": status }));
            }
            serde_json::from_str::<Value>(&raw).or_else(|_| Ok(json!({ "raw": crate::truncate(&raw, 400) })))
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(format!("{ctx}: HTTP {code}: {}", crate::truncate(&body, 240)))
        }
        Err(ureq::Error::Transport(t)) => Err(format!("{ctx}: network error: {t}")),
    }
}

fn http_get_json(data_dir: &Path, url: &str, bearer: &str, ctx: &str) -> Result<Value, String> {
    validate_endpoint_url(data_dir, url, "connector")?;
    let res = build_http_agent()
        .get(url)
        .set("Authorization", &format!("Bearer {bearer}"))
        .set("Accept", "application/json")
        .set("User-Agent", concat!("ergon/", env!("CARGO_PKG_VERSION")))
        .call();
    read_ureq_result(res, ctx)
}

fn http_json_write(
    data_dir: &Path,
    method: &str,
    url: &str,
    bearer: &str,
    payload: &Value,
    ctx: &str,
) -> Result<Value, String> {
    validate_endpoint_url(data_dir, url, "connector")?;
    let body_str = serde_json::to_string(payload).map_err(|e| format!("{ctx}: serialize JSON: {e}"))?;
    tripwire::check_egress_payload(&body_str, "connector")?;
    let agent = build_http_agent();
    let req = match method {
        "PUT" => agent.put(url),
        _ => agent.post(url),
    };
    let res = req
        .set("Authorization", &format!("Bearer {bearer}"))
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .set("User-Agent", concat!("ergon/", env!("CARGO_PKG_VERSION")))
        .send_string(&body_str);
    read_ureq_result(res, ctx)
}

fn http_raw_bytes_write(
    data_dir: &Path,
    method: &str,
    url: &str,
    bearer: &str,
    content_type: &str,
    bytes: &[u8],
    ctx: &str,
) -> Result<Value, String> {
    validate_endpoint_url(data_dir, url, "connector")?;
    if let Ok(txt) = std::str::from_utf8(bytes) {
        tripwire::check_egress_payload(txt, "connector")?;
    }
    let agent = build_http_agent();
    let req = match method {
        "PUT" => agent.put(url),
        _ => agent.post(url),
    };
    let res = req
        .set("Authorization", &format!("Bearer {bearer}"))
        .set("Content-Type", content_type)
        .set("Accept", "application/json")
        .set("User-Agent", concat!("ergon/", env!("CARGO_PKG_VERSION")))
        .send_bytes(bytes);
    read_ureq_result(res, ctx)
}

/// Lightweight check used by proactive app recommendations to know whether
/// single-click Google Workspace / Microsoft 365 OAuth sign-in is configured.
pub fn is_provider_configured(root_data_dir: &Path, provider: &str) -> bool {
    let Some(norm) = normalize_provider(provider) else {
        return false;
    };
    !resolve_app_config(root_data_dir, norm).client_id.is_empty()
}

/// Public label shown on Google/Microsoft plugin cards when connected via OAuth2.
pub fn connected_account_label(person_data_dir: &Path, provider: &str) -> Option<String> {
    let conns = with_cloud_lock(|| load_connections_unlocked(person_data_dir));
    let rec = conns.get(provider)?;
    secret_store::parse_vault_ref(&rec.access_token_ref)
        .and_then(|k| secret_store::get_secret(person_data_dir, k).ok().flatten())?;
    let base = suite_label(provider);
    Some(match rec.account_email.as_deref() {
        Some(email) => format!("{base} ({email})"),
        None => base.to_string(),
    })
}

pub fn list_oauth_suites_json(
    root_data_dir: &Path,
    person_data_dir: &Path,
    request_origin: &str,
) -> Value {
    let conns = with_cloud_lock(|| load_connections_unlocked(person_data_dir));
    let redirect_uri = default_callback_uri(request_origin);
    let mut suites = Vec::with_capacity(2);
    for provider in ["google", "microsoft"] {
        let app = resolve_app_config(root_data_dir, provider);
        let has_secret = resolve_client_secret(root_data_dir, &app).is_some();
        let conn = conns.get(provider);
        let token_present = conn
            .and_then(|c| secret_store::parse_vault_ref(&c.access_token_ref))
            .and_then(|k| secret_store::get_secret(person_data_dir, k).ok().flatten())
            .is_some();
        suites.push(json!({
            "provider": provider,
            "label": suite_label(provider),
            "configured": !app.client_id.is_empty(),
            "source": if app.from_env { "env" } else if !app.client_id.is_empty() { "saved" } else { "unconfigured" },
            "client_id": app.client_id,
            "has_client_secret": has_secret,
            "tenant": if provider == "microsoft" { json!(default_tenant(Some(&app.tenant))) } else { Value::Null },
            "scopes": resolved_scopes(&app),
            "redirect_uri": redirect_uri,
            "connected": token_present,
            "account_email": conn.and_then(|c| c.account_email.clone()),
            "account_name": conn.and_then(|c| c.account_name.clone()),
            "expires_at_ms": conn.map(|c| c.expires_at_ms),
            "connected_at_ms": conn.map(|c| c.connected_at_ms),
            "plugin_ids": plugin_ids_for_provider(provider),
        }));
    }
    json!({ "suites": suites })
}

pub fn save_oauth_app_config(root_data_dir: &Path, body: &Value) -> Result<(), (u16, String)> {
    let prov = body["provider"]
        .as_str()
        .and_then(normalize_provider)
        .ok_or_else(|| (400, "provider must be 'google' or 'microsoft'".to_string()))?;
    let client_id = body["client_id"].as_str().unwrap_or("").trim().to_string();
    if client_id.is_empty() {
        return Err((400, "client_id is required".to_string()));
    }

    for field in ["auth_url", "token_url", "userinfo_url", "api_base_url"] {
        if let Some(u) = body[field].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            validate_endpoint_url(root_data_dir, u, "connector_config").map_err(|e| (400, e))?;
        }
    }

    let secret_key = app_secret_key(prov);
    let clear_secret = body["clear_client_secret"].as_bool() == Some(true);
    let new_secret = body["client_secret"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !secret_store::is_vault_ref(s))
        .map(str::to_string);

    if clear_secret {
        let _ = secret_store::delete_secret(root_data_dir, &secret_key);
    } else if let Some(ref sec) = new_secret {
        secret_store::set_secret(root_data_dir, &secret_key, sec)
            .map_err(|e| (500, format!("encrypt OAuth client_secret: {e}")))?;
    }

    with_cloud_lock(|| {
        let mut map = load_apps_unlocked(root_data_dir);
        let existing = map.get(prov).cloned().unwrap_or_else(|| OAuthAppConfig {
            provider: prov.to_string(),
            tenant: MS_DEFAULT_TENANT.to_string(),
            ..Default::default()
        });
        let secret_ref = if clear_secret {
            None
        } else if new_secret.is_some() {
            Some(secret_store::vault_ref(&secret_key))
        } else {
            existing.client_secret_ref
        };
        let opt_str = |key: &str, prev: Option<String>| -> Option<String> {
            match body.get(key) {
                Some(Value::Null) => None,
                Some(Value::String(s)) => {
                    let t = s.trim();
                    if t.is_empty() { None } else { Some(t.to_string()) }
                }
                _ => prev,
            }
        };
        let next = OAuthAppConfig {
            provider: prov.to_string(),
            client_id,
            client_secret_ref: secret_ref,
            tenant: if prov == "microsoft" {
                default_tenant(body["tenant"].as_str().or(Some(&existing.tenant)))
            } else {
                MS_DEFAULT_TENANT.to_string()
            },
            auth_url: opt_str("auth_url", existing.auth_url),
            token_url: opt_str("token_url", existing.token_url),
            userinfo_url: opt_str("userinfo_url", existing.userinfo_url),
            api_base_url: opt_str("api_base_url", existing.api_base_url)
                .map(|s| s.trim_end_matches('/').to_string()),
            scopes: opt_str("scopes", existing.scopes),
            updated_ms: records::now_millis(),
            from_env: false,
        };
        map.insert(prov.to_string(), next);
        save_apps_unlocked(root_data_dir, &map).map_err(|e| (500, e))
    })
}

pub fn start_oauth_flow(
    root_data_dir: &Path,
    person_id: &str,
    person_data_dir: &Path,
    provider_raw: &str,
    request_origin: &str,
    return_to: &str,
) -> Result<String, (u16, String)> {
    let prov = normalize_provider(provider_raw)
        .ok_or_else(|| (400, "provider must be 'google' or 'microsoft'".to_string()))?;
    let policy = crate::load_workspace_policy(person_data_dir);
    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err((
            403,
            "blocked by Air-Gapped workspace policy: cloud OAuth sign-in is disabled".to_string(),
        ));
    }
    let app = resolve_app_config(root_data_dir, prov);
    if app.client_id.is_empty() {
        return Err((
            400,
            format!(
                "{} OAuth app is not registered yet. Ask an admin to save its Client ID in Plugins.",
                suite_label(prov)
            ),
        ));
    }

    let auth_url = resolved_auth_url(&app);
    validate_endpoint_url(person_data_dir, &auth_url, "connector_config").map_err(|e| (400, e))?;

    let state = base64url_encode_nopad(&secure_random_bytes(24).map_err(|e| (500, e))?);
    let code_verifier = base64url_encode_nopad(&secure_random_bytes(32).map_err(|e| (500, e))?);
    let challenge_digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64url_encode_nopad(&challenge_digest);
    let redirect_uri = default_callback_uri(request_origin);
    let scopes = resolved_scopes(&app);
    let clean_return = crate::auth::sanitize_redirect_after(return_to);
    let now = records::now_millis();

    {
        let mut guard = pending_map().lock().unwrap_or_else(|p| p.into_inner());
        guard.retain(|_, f| now.saturating_sub(f.created_ms) < PENDING_TTL_MS);
        if guard.len() >= MAX_PENDING_FLOWS {
            guard.clear();
        }
        guard.insert(
            state.clone(),
            PendingPluginOAuth {
                provider: prov.to_string(),
                person_id: person_id.to_string(),
                person_data_dir: person_data_dir.to_path_buf(),
                code_verifier,
                redirect_uri: redirect_uri.clone(),
                return_to: clean_return,
                created_ms: now,
            },
        );
    }

    let sep = if auth_url.contains('?') { '&' } else { '?' };
    let mut full = format!(
        "{auth_url}{sep}response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        url_encode_component(&app.client_id),
        url_encode_component(&redirect_uri),
        url_encode_component(&scopes),
        url_encode_component(&state),
        url_encode_component(&code_challenge),
    );
    if prov == "google" {
        full.push_str("&access_type=offline&prompt=consent");
    } else {
        full.push_str("&response_mode=query");
    }
    Ok(full)
}

pub fn complete_oauth_callback(
    root_data_dir: &Path,
    code: &str,
    state_param: &str,
    session_person_id: Option<&str>,
) -> Result<(String, Option<String>, String), (u16, String)> {
    let code = code.trim();
    let state_param = state_param.trim();
    if code.is_empty() || state_param.is_empty() {
        return Err((400, "missing OAuth code or state parameter".to_string()));
    }

    let now = records::now_millis();
    let pending = {
        let mut guard = pending_map().lock().unwrap_or_else(|p| p.into_inner());
        guard.retain(|_, f| now.saturating_sub(f.created_ms) < PENDING_TTL_MS);
        guard.remove(state_param)
    }
    .ok_or_else(|| (400, "invalid or expired OAuth state parameter".to_string()))?;

    if let Some(sid) = session_person_id {
        if !sid.is_empty() && sid != pending.person_id {
            return Err((
                403,
                "OAuth callback session does not match the signed-in user".to_string(),
            ));
        }
    }

    let policy = crate::load_workspace_policy(&pending.person_data_dir);
    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err((
            403,
            "blocked by Air-Gapped workspace policy: cloud OAuth callback denied".to_string(),
        ));
    }

    let app = resolve_app_config(root_data_dir, &pending.provider);
    if app.client_id.is_empty() {
        return Err((400, "OAuth app configuration is missing".to_string()));
    }
    let token_url = resolved_token_url(&app);
    validate_endpoint_url(&pending.person_data_dir, &token_url, "connector")
        .map_err(|e| (400, e))?;

    let client_secret = resolve_client_secret(root_data_dir, &app);
    let mut form_pairs: Vec<(&str, &str)> = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &pending.redirect_uri),
        ("client_id", &app.client_id),
        ("code_verifier", &pending.code_verifier),
    ];
    if let Some(ref sec) = client_secret {
        form_pairs.push(("client_secret", sec.as_str()));
    }

    let token_resp = build_http_agent()
        .post(&token_url)
        .set("Accept", "application/json")
        .send_form(&form_pairs);
    let token_json =
        read_ureq_result(token_resp, "OAuth token exchange").map_err(|e| (502, e))?;

    let access_token = token_json["access_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| (502, "OAuth provider response missing access_token".to_string()))?
        .to_string();
    let refresh_token = token_json["refresh_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let expires_in_s = token_json["expires_in"].as_u64().unwrap_or(3600);
    let expires_at_ms = now.saturating_add(expires_in_s.saturating_mul(1000));
    let scopes = token_json["scope"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| resolved_scopes(&app));

    let (account_email, account_name) =
        fetch_account_profile(&pending.person_data_dir, &app, &access_token);

    let at_key = access_secret_key(&pending.provider);
    secret_store::set_secret(&pending.person_data_dir, &at_key, &access_token)
        .map_err(|e| (500, format!("encrypt OAuth access_token: {e}")))?;

    let rt_key = refresh_secret_key(&pending.provider);
    let rt_ref = if let Some(ref rt) = refresh_token {
        secret_store::set_secret(&pending.person_data_dir, &rt_key, rt)
            .map_err(|e| (500, format!("encrypt OAuth refresh_token: {e}")))?;
        Some(secret_store::vault_ref(&rt_key))
    } else {
        let _ = secret_store::delete_secret(&pending.person_data_dir, &rt_key);
        None
    };

    with_cloud_lock(|| {
        let mut conns = load_connections_unlocked(&pending.person_data_dir);
        conns.insert(
            pending.provider.clone(),
            PluginConnectionRecord {
                provider: pending.provider.clone(),
                account_email: account_email.clone(),
                account_name: account_name.clone(),
                scopes,
                expires_at_ms,
                connected_at_ms: now,
                access_token_ref: secret_store::vault_ref(&at_key),
                refresh_token_ref: rt_ref,
            },
        );
        save_connections_unlocked(&pending.person_data_dir, &conns)
    })
    .map_err(|e| (500, e))?;

    Ok((pending.provider, account_email, pending.return_to))
}

fn fetch_account_profile(
    person_data_dir: &Path,
    app: &OAuthAppConfig,
    access_token: &str,
) -> (Option<String>, Option<String>) {
    let url = resolved_userinfo_url(app);
    let Ok(v) = http_get_json(person_data_dir, &url, access_token, "OAuth profile") else {
        return (None, None);
    };
    let email = v["email"]
        .as_str()
        .or_else(|| v["mail"].as_str())
        .or_else(|| v["userPrincipalName"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let name = v["name"]
        .as_str()
        .or_else(|| v["displayName"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    (email, name)
}

pub fn disconnect_provider(person_data_dir: &Path, provider_raw: &str) -> Result<(), String> {
    let prov = normalize_provider(provider_raw)
        .ok_or_else(|| format!("unknown OAuth provider: {provider_raw}"))?;
    let _ = secret_store::delete_secret(person_data_dir, &access_secret_key(prov));
    let _ = secret_store::delete_secret(person_data_dir, &refresh_secret_key(prov));
    with_cloud_lock(|| {
        let mut conns = load_connections_unlocked(person_data_dir);
        conns.remove(prov);
        save_connections_unlocked(person_data_dir, &conns)
    })
}

fn ensure_valid_access_token(
    root_data_dir: &Path,
    person_data_dir: &Path,
    provider: &str,
) -> Result<Option<String>, String> {
    let rec = with_cloud_lock(|| load_connections_unlocked(person_data_dir))
        .get(provider)
        .cloned();
    let Some(rec) = rec else {
        return Ok(None);
    };

    let at_opt = secret_store::parse_vault_ref(&rec.access_token_ref)
        .and_then(|k| secret_store::get_secret(person_data_dir, k).ok().flatten());
    let now = records::now_millis();
    let needs_refresh = rec.expires_at_ms > 0
        && now.saturating_add(TOKEN_REFRESH_LEEWAY_MS) >= rec.expires_at_ms;

    if !needs_refresh {
        if let Some(at) = at_opt {
            return Ok(Some(at));
        }
    }

    let rt = rec
        .refresh_token_ref
        .as_deref()
        .and_then(secret_store::parse_vault_ref)
        .and_then(|k| secret_store::get_secret(person_data_dir, k).ok().flatten());
    let Some(rt_val) = rt else {
        return Ok(at_opt);
    };

    let app = resolve_app_config(root_data_dir, provider);
    if app.client_id.is_empty() {
        return Ok(at_opt);
    }
    let token_url = resolved_token_url(&app);
    validate_endpoint_url(person_data_dir, &token_url, "connector")?;
    let client_secret = resolve_client_secret(root_data_dir, &app);

    let mut form_pairs: Vec<(&str, &str)> = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", &rt_val),
        ("client_id", &app.client_id),
    ];
    if let Some(ref sec) = client_secret {
        form_pairs.push(("client_secret", sec.as_str()));
    }
    let res = build_http_agent()
        .post(&token_url)
        .set("Accept", "application/json")
        .send_form(&form_pairs);
    let token_json = read_ureq_result(res, "OAuth token refresh")?;
    let new_at = token_json["access_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "OAuth refresh response missing access_token".to_string())?
        .to_string();
    let expires_in_s = token_json["expires_in"].as_u64().unwrap_or(3600);
    let new_exp = now.saturating_add(expires_in_s.saturating_mul(1000));

    let at_key = access_secret_key(provider);
    secret_store::set_secret(person_data_dir, &at_key, &new_at)
        .map_err(|e| format!("persist refreshed access_token: {e}"))?;

    let mut new_rt_ref = rec.refresh_token_ref.clone();
    if let Some(new_rt) = token_json["refresh_token"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let rt_key = refresh_secret_key(provider);
        secret_store::set_secret(person_data_dir, &rt_key, new_rt)
            .map_err(|e| format!("persist refreshed refresh_token: {e}"))?;
        new_rt_ref = Some(secret_store::vault_ref(&rt_key));
    }

    with_cloud_lock(|| {
        let mut conns = load_connections_unlocked(person_data_dir);
        if let Some(cur) = conns.get_mut(provider) {
            cur.expires_at_ms = new_exp;
            cur.refresh_token_ref = new_rt_ref;
        }
        save_connections_unlocked(person_data_dir, &conns)
    })?;

    Ok(Some(new_at))
}

fn google_endpoint(app: &OAuthAppConfig, default_origin: &str, path: &str) -> String {
    let base = app
        .api_base_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(default_origin)
        .trim_end_matches('/');
    format!("{base}{path}")
}

fn ms_endpoint(app: &OAuthAppConfig, path: &str) -> String {
    let base = app
        .api_base_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(MS_DEFAULT_API_BASE)
        .trim_end_matches('/');
    format!("{base}{path}")
}

fn read_workspace_upload_bytes(
    cfg: &Config,
    args: &Value,
    default_name: &str,
) -> Result<(String, Vec<u8>), String> {
    if let Some(rel) = args["path"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
        let resolved = sandbox_path(&cfg.workspace_dir, rel)?;
        let bytes = std::fs::read(&resolved).map_err(|e| format!("read workspace '{rel}': {e}"))?;
        let fname = resolved
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(default_name)
            .to_string();
        return Ok((fname, bytes));
    }
    let name = args["name"]
        .as_str()
        .or_else(|| args["title"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_name)
        .replace(['/', '\\'], "_");
    let content = args["content"]
        .as_str()
        .or_else(|| args["body"].as_str())
        .unwrap_or("Generated by Ergon.");
    Ok((name, content.as_bytes().to_vec()))
}

fn build_gmail_raw_mime(to: &str, subject: &str, body: &str) -> Result<String, String> {
    let clean_to = to.trim();
    if clean_to.is_empty() || clean_to.contains('\r') || clean_to.contains('\n') {
        return Err("send_mail requires a valid single-line args.to recipient".to_string());
    }
    let clean_subj = subject.replace(['\r', '\n'], " ");
    let mime = format!(
        "To: {clean_to}\r\nSubject: {clean_subj}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
    );
    Ok(base64url_encode_nopad(mime.as_bytes()))
}

fn parse_csv_rows(raw: &str) -> Vec<Vec<Value>> {
    raw.lines()
        .map(|line| {
            line.split(',')
                .map(|c| json!(c.trim().trim_matches('"')))
                .collect()
        })
        .collect()
}

/// Execute a Google Workspace or Microsoft Graph REST action if the employee has
/// connected that cloud suite via native browser OAuth2.
/// Returns `Ok(None)` when no native OAuth connection exists (so caller falls back
/// to legacy connectors/local mode).
pub fn execute_native_cloud_action(
    cfg: &Config,
    plugin_id: &str,
    action: &str,
    args: &Value,
) -> Result<Option<String>, String> {
    let Some(provider) = provider_for_plugin(plugin_id) else {
        return Ok(None);
    };
    let root_data_dir = records::infer_root_data_dir(&cfg.data_dir);
    let policy = crate::load_workspace_policy(&cfg.data_dir);

    let has_conn = with_cloud_lock(|| load_connections_unlocked(&cfg.data_dir)).contains_key(provider);
    if !has_conn {
        return Ok(None);
    }

    if policy.block_outbound_fetch || policy.profile == "air_gapped" {
        return Err(
            "blocked by Air-Gapped workspace policy: cloud suite REST access is disabled".to_string(),
        );
    }

    let Some(token) = ensure_valid_access_token(&root_data_dir, &cfg.data_dir, provider)? else {
        return Ok(None);
    };
    let app = resolve_app_config(&root_data_dir, provider);

    let out = match (plugin_id, action) {
        // ---- Google Workspace ----
        ("google-gmail", "read_inbox") => {
            let max_results = args["limit"].as_u64().unwrap_or(5).clamp(1, 20);
            let list_url = google_endpoint(
                &app,
                "https://gmail.googleapis.com",
                &format!("/gmail/v1/users/me/messages?maxResults={max_results}"),
            );
            let list_json = http_get_json(&cfg.data_dir, &list_url, &token, "Gmail list messages")?;
            let msgs = list_json["messages"].as_array().cloned().unwrap_or_default();
            let mut lines = vec!["# Gmail Inbox Intake".to_string()];
            for m in msgs.iter().take(5) {
                let id = m["id"].as_str().unwrap_or("");
                if id.is_empty() {
                    continue;
                }
                let msg_url = google_endpoint(
                    &app,
                    "https://gmail.googleapis.com",
                    &format!("/gmail/v1/users/me/messages/{}?format=metadata", url_encode_component(id)),
                );
                if let Ok(detail) = http_get_json(&cfg.data_dir, &msg_url, &token, "Gmail message detail") {
                    let snippet = detail["snippet"].as_str().unwrap_or("");
                    let headers = detail["payload"]["headers"].as_array();
                    let header_val = |name: &str| -> String {
                        headers
                            .and_then(|arr| {
                                arr.iter().find(|h| h["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(name)))
                            })
                            .and_then(|h| h["value"].as_str())
                            .unwrap_or("")
                            .to_string()
                    };
                    let subj = header_val("Subject");
                    let from = header_val("From");
                    lines.push(format!("- **{subj}** ({from}): {snippet}"));
                }
            }
            let rel = format!("intake/gmail-inbox-{}.md", records::now_millis());
            let _ = execute_tool(
                cfg,
                "write_file",
                &json!({ "path": rel, "content": lines.join("\n") }),
            )?;
            format!("Gmail API synced {} message(s) into workspace/{rel}.", msgs.len())
        }
        ("google-gmail", "send_mail") => {
            let to = args["to"].as_str().unwrap_or("");
            let subject = args["subject"].as_str().unwrap_or("Message from Ergon");
            let body = args["body"].as_str().or_else(|| args["content"].as_str()).unwrap_or("");
            let raw = build_gmail_raw_mime(to, subject, body)?;
            let url = google_endpoint(&app, "https://gmail.googleapis.com", "/gmail/v1/users/me/messages/send");
            let resp = http_json_write(&cfg.data_dir, "POST", &url, &token, &json!({ "raw": raw }), "Gmail send")?;
            format!("Gmail API sent message (id={}).", resp["id"].as_str().unwrap_or("sent"))
        }
        ("google-calendar", "list_events") => {
            let limit = args["limit"].as_u64().unwrap_or(10).clamp(1, 25);
            let url = google_endpoint(
                &app,
                "https://www.googleapis.com",
                &format!("/calendar/v3/calendars/primary/events?maxResults={limit}&singleEvents=true&orderBy=startTime"),
            );
            let v = http_get_json(&cfg.data_dir, &url, &token, "Google Calendar list")?;
            let items = v["items"].as_array().cloned().unwrap_or_default();
            let preview: Vec<String> = items
                .iter()
                .take(5)
                .map(|e| e["summary"].as_str().unwrap_or("Event").to_string())
                .collect();
            format!(
                "Google Calendar returned {} upcoming event(s){}.",
                items.len(),
                if preview.is_empty() { String::new() } else { format!(": {}", preview.join(", ")) }
            )
        }
        ("google-calendar", "create_event") => {
            let summary = args["summary"].as_str().or_else(|| args["title"].as_str()).unwrap_or("Meeting");
            let start = args["start"].as_str().ok_or("create_event needs args.start ISO-8601 timestamp")?;
            let end = args["end"].as_str().ok_or("create_event needs args.end ISO-8601 timestamp")?;
            let mut payload = json!({
                "summary": summary,
                "start": { "dateTime": start },
                "end": { "dateTime": end },
            });
            if let Some(desc) = args["description"].as_str() {
                payload["description"] = json!(desc);
            }
            if let Some(att) = args["attendees"].as_array() {
                payload["attendees"] = json!(att
                    .iter()
                    .filter_map(|v| v.as_str().map(|e| json!({ "email": e })))
                    .collect::<Vec<_>>());
            }
            let url = google_endpoint(&app, "https://www.googleapis.com", "/calendar/v3/calendars/primary/events");
            let v = http_json_write(&cfg.data_dir, "POST", &url, &token, &payload, "Google Calendar create")?;
            format!("Google Calendar created event '{}' (id={}).", summary, v["id"].as_str().unwrap_or("ok"))
        }
        ("google-drive", "list_files") => {
            let limit = args["limit"].as_u64().unwrap_or(10).clamp(1, 50);
            let url = google_endpoint(
                &app,
                "https://www.googleapis.com",
                &format!("/drive/v3/files?pageSize={limit}&fields=files(id,name,mimeType,modifiedTime)"),
            );
            let v = http_get_json(&cfg.data_dir, &url, &token, "Google Drive list")?;
            let files = v["files"].as_array().cloned().unwrap_or_default();
            let names: Vec<String> = files
                .iter()
                .take(6)
                .map(|f| f["name"].as_str().unwrap_or("file").to_string())
                .collect();
            format!(
                "Google Drive listed {} item(s){}.",
                files.len(),
                if names.is_empty() { String::new() } else { format!(": {}", names.join(", ")) }
            )
        }
        ("google-drive", "upload_deliverable") => {
            let (fname, bytes) = read_workspace_upload_bytes(cfg, args, "deliverable.txt")?;
            let boundary = format!("ergon-drive-{}", records::now_millis());
            let meta = json!({ "name": fname });
            let mut body = Vec::new();
            body.extend_from_slice(format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{boundary}\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes());
            body.extend_from_slice(&bytes);
            body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
            let url = google_endpoint(&app, "https://www.googleapis.com", "/upload/drive/v3/files?uploadType=multipart");
            let ct = format!("multipart/related; boundary={boundary}");
            let v = http_raw_bytes_write(&cfg.data_dir, "POST", &url, &token, &ct, &body, "Google Drive upload")?;
            format!(
                "Google Drive uploaded '{}' ({} bytes, id={}).",
                fname,
                bytes.len(),
                v["id"].as_str().unwrap_or("ok")
            )
        }
        ("google-docs", "publish_doc") => {
            let title = args["title"].as_str().unwrap_or("Ergon Document");
            let content = args["content"].as_str().or_else(|| args["body"].as_str()).unwrap_or("");
            let create_url = google_endpoint(&app, "https://docs.googleapis.com", "/v1/documents");
            let doc = http_json_write(&cfg.data_dir, "POST", &create_url, &token, &json!({ "title": title }), "Google Docs create")?;
            let doc_id = doc["documentId"].as_str().unwrap_or("doc-created");
            if !content.is_empty() {
                let batch_url = google_endpoint(
                    &app,
                    "https://docs.googleapis.com",
                    &format!("/v1/documents/{}:batchUpdate", url_encode_component(doc_id)),
                );
                let batch = json!({ "requests": [{ "insertText": { "location": { "index": 1 }, "text": content } }] });
                let _ = http_json_write(&cfg.data_dir, "POST", &batch_url, &token, &batch, "Google Docs insert")?;
            }
            format!("Google Docs published '{title}' (documentId={doc_id}).")
        }
        ("google-sheets", "update_range") => {
            let range = args["range"].as_str().unwrap_or("Sheet1!A1");
            let values = if args["values"].is_array() {
                args["values"].clone()
            } else if let Some(rel) = args["path"].as_str().filter(|s| !s.trim().is_empty()) {
                let full = sandbox_path(&cfg.workspace_dir, rel)?;
                let raw = std::fs::read_to_string(&full).map_err(|e| format!("read csv '{rel}': {e}"))?;
                Value::Array(parse_csv_rows(&raw).into_iter().map(Value::Array).collect())
            } else {
                json!([[args["value"].as_str().unwrap_or("Updated by Ergon")]])
            };
            let sheet_id = if let Some(id) = args["spreadsheet_id"].as_str().filter(|s| !s.trim().is_empty()) {
                id.trim().to_string()
            } else {
                let title = args["title"].as_str().unwrap_or("Ergon Sheet");
                let create_url = google_endpoint(&app, "https://sheets.googleapis.com", "/v4/spreadsheets");
                let created = http_json_write(
                    &cfg.data_dir,
                    "POST",
                    &create_url,
                    &token,
                    &json!({ "properties": { "title": title } }),
                    "Google Sheets create",
                )?;
                created["spreadsheetId"].as_str().unwrap_or("sheet-created").to_string()
            };
            let upd_url = google_endpoint(
                &app,
                "https://sheets.googleapis.com",
                &format!(
                    "/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
                    url_encode_component(&sheet_id),
                    url_encode_component(range)
                ),
            );
            let _ = http_json_write(&cfg.data_dir, "PUT", &upd_url, &token, &json!({ "values": values }), "Google Sheets update")?;
            format!("Google Sheets updated {range} on spreadsheetId={sheet_id}.")
        }
        ("google-slides", "update_deck") => {
            let title = args["title"].as_str().unwrap_or("Ergon Presentation");
            if let Some(pres_id) = args["presentation_id"].as_str().filter(|s| !s.trim().is_empty()) {
                let url = google_endpoint(
                    &app,
                    "https://slides.googleapis.com",
                    &format!("/v1/presentations/{}:batchUpdate", url_encode_component(pres_id)),
                );
                let reqs = if args["requests"].is_array() {
                    args["requests"].clone()
                } else {
                    json!([{ "createSlide": { "slideLayoutReference": { "predefinedLayout": "TITLE_AND_BODY" } } }])
                };
                let _ = http_json_write(&cfg.data_dir, "POST", &url, &token, &json!({ "requests": reqs }), "Google Slides update")?;
                format!("Google Slides updated presentationId={pres_id}.")
            } else {
                let url = google_endpoint(&app, "https://slides.googleapis.com", "/v1/presentations");
                let v = http_json_write(&cfg.data_dir, "POST", &url, &token, &json!({ "title": title }), "Google Slides create")?;
                format!(
                    "Google Slides created '{}' (presentationId={}).",
                    title,
                    v["presentationId"].as_str().unwrap_or("ok")
                )
            }
        }

        // ---- Microsoft 365 & Outlook (Microsoft Graph) ----
        ("outlook-mail", "read_inbox") => {
            let top = args["limit"].as_u64().unwrap_or(5).clamp(1, 20);
            let url = ms_endpoint(&app, &format!("/me/mailFolders/inbox/messages?$top={top}&$select=subject,from,receivedDateTime,bodyPreview"));
            let v = http_get_json(&cfg.data_dir, &url, &token, "Outlook inbox")?;
            let items = v["value"].as_array().cloned().unwrap_or_default();
            let mut lines = vec!["# Outlook Inbox Intake".to_string()];
            for m in &items {
                let subj = m["subject"].as_str().unwrap_or("(no subject)");
                let sender = m["from"]["emailAddress"]["address"].as_str().unwrap_or("");
                let prev = m["bodyPreview"].as_str().unwrap_or("");
                lines.push(format!("- **{subj}** ({sender}): {prev}"));
            }
            let rel = format!("intake/outlook-inbox-{}.md", records::now_millis());
            let _ = execute_tool(
                cfg,
                "write_file",
                &json!({ "path": rel, "content": lines.join("\n") }),
            )?;
            format!("Microsoft Graph synced {} Outlook message(s) into workspace/{rel}.", items.len())
        }
        ("outlook-mail", "send_mail") => {
            let to = args["to"].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("send_mail needs args.to")?;
            let subject = args["subject"].as_str().unwrap_or("Message from Ergon");
            let body = args["body"].as_str().or_else(|| args["content"].as_str()).unwrap_or("");
            let payload = json!({
                "message": {
                    "subject": subject,
                    "body": { "contentType": "Text", "content": body },
                    "toRecipients": [{ "emailAddress": { "address": to } }]
                },
                "saveToSentItems": true
            });
            let url = ms_endpoint(&app, "/me/sendMail");
            let _ = http_json_write(&cfg.data_dir, "POST", &url, &token, &payload, "Outlook sendMail")?;
            format!("Outlook Mail sent to {to}.")
        }
        ("outlook-calendar", "list_events") => {
            let top = args["limit"].as_u64().unwrap_or(10).clamp(1, 25);
            let url = ms_endpoint(&app, &format!("/me/events?$top={top}&$select=subject,start,end"));
            let v = http_get_json(&cfg.data_dir, &url, &token, "Outlook calendar list")?;
            let items = v["value"].as_array().cloned().unwrap_or_default();
            let titles: Vec<String> = items
                .iter()
                .take(5)
                .map(|e| e["subject"].as_str().unwrap_or("Event").to_string())
                .collect();
            format!(
                "Outlook Calendar returned {} event(s){}.",
                items.len(),
                if titles.is_empty() { String::new() } else { format!(": {}", titles.join(", ")) }
            )
        }
        ("outlook-calendar", "schedule_meeting") => {
            let subject = args["subject"].as_str().or_else(|| args["title"].as_str()).unwrap_or("Meeting");
            let start = args["start"].as_str().ok_or("schedule_meeting needs args.start")?;
            let end = args["end"].as_str().ok_or("schedule_meeting needs args.end")?;
            let tz = args["time_zone"].as_str().unwrap_or("UTC");
            let payload = json!({
                "subject": subject,
                "start": { "dateTime": start, "timeZone": tz },
                "end": { "dateTime": end, "timeZone": tz },
            });
            let url = ms_endpoint(&app, "/me/events");
            let v = http_json_write(&cfg.data_dir, "POST", &url, &token, &payload, "Outlook schedule meeting")?;
            format!("Outlook Calendar scheduled '{}' (id={}).", subject, v["id"].as_str().unwrap_or("ok"))
        }
        ("onedrive-sharepoint", "list_files") | ("office-excel", "inspect_workbook") => {
            let top = args["limit"].as_u64().unwrap_or(10).clamp(1, 50);
            let url = ms_endpoint(&app, &format!("/me/drive/root/children?$top={top}&$select=id,name,size,lastModifiedDateTime"));
            let v = http_get_json(&cfg.data_dir, &url, &token, "OneDrive list")?;
            let items = v["value"].as_array().cloned().unwrap_or_default();
            let names: Vec<String> = items
                .iter()
                .take(6)
                .map(|i| i["name"].as_str().unwrap_or("item").to_string())
                .collect();
            format!(
                "OneDrive listed {} item(s){}.",
                items.len(),
                if names.is_empty() { String::new() } else { format!(": {}", names.join(", ")) }
            )
        }
        ("onedrive-sharepoint", "publish_file") => {
            let (fname, bytes) = read_workspace_upload_bytes(cfg, args, "deliverable.txt")?;
            let url = ms_endpoint(
                &app,
                &format!("/me/drive/root:/{}:/content", url_encode_component(&fname)),
            );
            let v = http_raw_bytes_write(
                &cfg.data_dir,
                "PUT",
                &url,
                &token,
                "application/octet-stream",
                &bytes,
                "OneDrive upload",
            )?;
            format!(
                "OneDrive uploaded '{}' ({} bytes, id={}).",
                fname,
                bytes.len(),
                v["id"].as_str().unwrap_or("ok")
            )
        }
        ("microsoft-teams", "read_channel") => {
            if let (Some(tid), Some(cid)) = (
                args["team_id"].as_str().filter(|s| !s.is_empty()),
                args["channel_id"].as_str().filter(|s| !s.is_empty()),
            ) {
                let top = args["limit"].as_u64().unwrap_or(10).clamp(1, 20);
                let url = ms_endpoint(
                    &app,
                    &format!("/teams/{}/channels/{}/messages?$top={top}", url_encode_component(tid), url_encode_component(cid)),
                );
                let v = http_get_json(&cfg.data_dir, &url, &token, "Teams read channel")?;
                let items = v["value"].as_array().cloned().unwrap_or_default();
                format!("Microsoft Teams read {} channel message(s).", items.len())
            } else {
                let url = ms_endpoint(&app, "/me/joinedTeams");
                let v = http_get_json(&cfg.data_dir, &url, &token, "Teams list joinedTeams")?;
                let items = v["value"].as_array().cloned().unwrap_or_default();
                format!("Microsoft Teams found {} joined team(s).", items.len())
            }
        }
        ("microsoft-teams", "post_update") => {
            let tid = args["team_id"].as_str().filter(|s| !s.is_empty()).ok_or("post_update needs args.team_id")?;
            let cid = args["channel_id"].as_str().filter(|s| !s.is_empty()).ok_or("post_update needs args.channel_id")?;
            let content = args["content"].as_str().or_else(|| args["body"].as_str()).unwrap_or("Update from Ergon");
            let url = ms_endpoint(
                &app,
                &format!("/teams/{}/channels/{}/messages", url_encode_component(tid), url_encode_component(cid)),
            );
            let v = http_json_write(
                &cfg.data_dir,
                "POST",
                &url,
                &token,
                &json!({ "body": { "content": content } }),
                "Teams post message",
            )?;
            format!("Microsoft Teams posted message (id={}).", v["id"].as_str().unwrap_or("ok"))
        }
        _ => return Ok(None),
    };

    Ok(Some(out))
}
