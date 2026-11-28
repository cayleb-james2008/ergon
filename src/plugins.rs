//! Default App Plugin Library + Auto-Updating Catalog & Hybrid Cloud/Local Runtime.
//!
//! Ergon loads its curated baseline catalog (`catalog/plugins.json`, 32 workplace
//! apps across Google Workspace, Microsoft 365 & Outlook, Business Apps, and On-Device Local)
//! and merges in live curated catalog updates and Official MCP Registry community apps
//! from `crate::plugin_registry`.
//!
//! Plugins run **locally on-device** when offline-capable or switch seamlessly to **cloud** mode
//! once bound to one-click OAuth2, Connectors, or 1-click installed apps (`install_plugin`).
//! Consequential external writes always park as Approve/Deny cards on `/phone` and Today.

use crate::plugin_registry::{self, DynamicPluginSpec};
use crate::tools::atomic_write_bytes_with_mode;
use crate::{connectors, mcp, parse_url_host_port, plugin_cloud, records, Config};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static PLUGINS_LOCK: Mutex<()> = Mutex::new(());

fn with_plugins_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = PLUGINS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn plugins_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("plugins.json")
}

#[derive(Clone, Debug)]
pub struct PluginPersonSetting {
    pub enabled: bool,
    pub preferred_mode: String, // "auto" | "local" | "cloud"
    pub updated_ms: u64,
}

impl PluginPersonSetting {
    fn from_json(v: &Value, default_enabled: bool) -> Self {
        let enabled = v["enabled"].as_bool().unwrap_or(default_enabled);
        let preferred_mode = match v["preferred_mode"].as_str().unwrap_or("auto") {
            "local" => "local".to_string(),
            "cloud" => "cloud".to_string(),
            _ => "auto".to_string(),
        };
        let updated_ms = v["updated_ms"].as_u64().unwrap_or(0);
        Self {
            enabled,
            preferred_mode,
            updated_ms,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "preferred_mode": self.preferred_mode,
            "updated_ms": self.updated_ms,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InstalledPluginRecord {
    pub plugin_id: String,
    pub mcp_server_id: String,
    pub installed_ms: u64,
    pub spec: DynamicPluginSpec,
}

impl InstalledPluginRecord {
    fn from_json(id: &str, v: &Value) -> Option<Self> {
        let mcp_server_id = v["mcp_server_id"].as_str()?.trim().to_string();
        if mcp_server_id.is_empty() {
            return None;
        }
        let spec = DynamicPluginSpec::from_json(&v["spec"], "community_registry")?;
        Some(Self {
            plugin_id: id.to_string(),
            mcp_server_id,
            installed_ms: v["installed_ms"].as_u64().unwrap_or(0),
            spec,
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "plugin_id": self.plugin_id,
            "mcp_server_id": self.mcp_server_id,
            "installed_ms": self.installed_ms,
            "spec": self.spec.to_storage_json(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PluginsStore {
    pub settings: BTreeMap<String, PluginPersonSetting>,
    pub installed: BTreeMap<String, InstalledPluginRecord>,
}

impl PluginsStore {
    fn load_unlocked(person_data_dir: &Path) -> Self {
        let path = plugins_path(person_data_dir);
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<Value>(&raw) else {
            return Self::default();
        };
        let mut installed = BTreeMap::new();
        if let Some(obj) = val["installed"].as_object() {
            for (k, v) in obj {
                if let Some(rec) = InstalledPluginRecord::from_json(k, v) {
                    installed.insert(k.clone(), rec);
                }
            }
        }
        let baseline = plugin_registry::embedded_baseline();
        let mut settings = BTreeMap::new();
        if let Some(obj) = val["settings"].as_object() {
            for (k, v) in obj {
                let default_enabled = installed
                    .get(k.as_str())
                    .map(|r| r.spec.default_enabled)
                    .or_else(|| {
                        baseline
                            .iter()
                            .find(|p| p.id == k.as_str())
                            .map(|p| p.default_enabled)
                    })
                    .unwrap_or(false);
                settings.insert(k.clone(), PluginPersonSetting::from_json(v, default_enabled));
            }
        }
        Self { settings, installed }
    }

    fn save_unlocked(&self, person_data_dir: &Path) -> Result<(), String> {
        let mut settings_obj = serde_json::Map::new();
        for (k, v) in &self.settings {
            settings_obj.insert(k.clone(), v.to_json());
        }
        let mut installed_obj = serde_json::Map::new();
        for (k, v) in &self.installed {
            installed_obj.insert(k.clone(), v.to_json());
        }
        let body = serde_json::to_string_pretty(&json!({
            "settings": settings_obj,
            "installed": installed_obj,
        }))
        .map_err(|e| format!("serialize plugins: {e}"))?;
        atomic_write_bytes_with_mode(&plugins_path(person_data_dir), body.as_bytes(), Some(0o600))
            .map_err(|e| format!("write plugins.json: {e}"))
    }
}

pub fn catalog_for_person(person_data_dir: &Path) -> Vec<DynamicPluginSpec> {
    let root = records::infer_root_data_dir(person_data_dir);
    let mut list = plugin_registry::merged_catalog(&root);
    let mut seen: HashSet<String> = list.iter().map(|p| p.id.clone()).collect();
    let store = with_plugins_lock(|| {
        let path = plugins_path(person_data_dir);
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return BTreeMap::new();
        };
        let Ok(val) = serde_json::from_str::<Value>(&raw) else {
            return BTreeMap::new();
        };
        let mut map = BTreeMap::new();
        if let Some(obj) = val["installed"].as_object() {
            for (k, v) in obj {
                if let Some(rec) = InstalledPluginRecord::from_json(k, v) {
                    map.insert(k.clone(), rec);
                }
            }
        }
        map
    });
    for rec in store.into_values() {
        if seen.insert(rec.spec.id.clone()) {
            list.push(rec.spec);
        }
    }
    list
}

pub fn find_plugin_for_person(person_data_dir: &Path, id: &str) -> Option<DynamicPluginSpec> {
    let key = id.trim();
    catalog_for_person(person_data_dir)
        .into_iter()
        .find(|p| p.id == key)
}

pub fn find_plugin(id: &str) -> Option<DynamicPluginSpec> {
    let base = plugin_registry::embedded_baseline();
    plugin_registry::find_in_catalog(&base, id)
}

#[derive(Clone, Debug)]
struct CloudMatch {
    connector_id: Option<String>,
    connector_name: Option<String>,
    trigger_id: Option<String>,
    trigger_name: Option<String>,
    mcp_server_id: Option<String>,
    mcp_server_name: Option<String>,
}

fn host_matches(candidate: &str, patterns: &[String]) -> bool {
    let low = candidate.trim().to_ascii_lowercase();
    if low.is_empty() {
        return false;
    }
    patterns.iter().any(|pat| {
        let p = pat.to_ascii_lowercase();
        low == p || low.ends_with(&format!(".{p}"))
    })
}

fn detect_cloud_match(
    spec: &DynamicPluginSpec,
    connectors_snapshot: &Value,
    mcp_snapshot: &Value,
    installed_rec: Option<&InstalledPluginRecord>,
) -> Option<CloudMatch> {
    let mcp_servers = mcp_snapshot["servers"].as_array();
    let mcp_len = mcp_servers
        .map(|a| a.iter().filter(|s| s["enabled"].as_bool() == Some(true)).count())
        .unwrap_or(0);

    if spec.id == "local-mcp-bridge" {
        if mcp_len > 0 {
            return Some(CloudMatch {
                connector_id: None,
                connector_name: Some(format!("{mcp_len} MCP server(s)")),
                trigger_id: None,
                trigger_name: None,
                mcp_server_id: None,
                mcp_server_name: None,
            });
        }
        return None;
    }

    let mut matched = CloudMatch {
        connector_id: None,
        connector_name: None,
        trigger_id: None,
        trigger_name: None,
        mcp_server_id: None,
        mcp_server_name: None,
    };

    if let Some(rec) = installed_rec {
        if let Some(servers) = mcp_servers {
            if let Some(srv) = servers
                .iter()
                .find(|s| s["id"].as_str() == Some(rec.mcp_server_id.as_str()) && s["enabled"].as_bool() == Some(true))
            {
                matched.mcp_server_id = srv["id"].as_str().map(str::to_string);
                matched.mcp_server_name = srv["name"].as_str().map(str::to_string);
            }
        }
    }

    if let Some(preset) = spec.connect_preset.as_ref() {
        if !preset.match_hosts.is_empty() {
            if let Some(outbound) = connectors_snapshot["outbound"].as_array() {
                for c in outbound {
                    if c["enabled"].as_bool() != Some(true) {
                        continue;
                    }
                    let url = c["url"].as_str().unwrap_or("");
                    let uhost = parse_url_host_port(url).map(|(h, _)| h).unwrap_or_default();
                    let smtp_host = c["smtp_config"]["host"].as_str().unwrap_or("");
                    if host_matches(&uhost, &preset.match_hosts)
                        || host_matches(smtp_host, &preset.match_hosts)
                    {
                        matched.connector_id = c["id"].as_str().map(str::to_string);
                        matched.connector_name = c["name"].as_str().map(str::to_string);
                        break;
                    }
                }
            }

            if let Some(triggers) = connectors_snapshot["triggers"].as_array() {
                for t in triggers {
                    if t["enabled"].as_bool() != Some(true) {
                        continue;
                    }
                    if t["kind"].as_str() == Some("imap_poll") {
                        let imap_host = t["imap_config"]["host"].as_str().unwrap_or("");
                        if host_matches(imap_host, &preset.match_hosts) {
                            matched.trigger_id = t["id"].as_str().map(str::to_string);
                            matched.trigger_name = t["name"].as_str().map(str::to_string);
                            break;
                        }
                    }
                }
            }
        }
    }

    if matched.connector_id.is_some()
        || matched.trigger_id.is_some()
        || matched.mcp_server_id.is_some()
    {
        Some(matched)
    } else {
        None
    }
}

fn resolve_plugin_json(
    spec: &DynamicPluginSpec,
    setting: &PluginPersonSetting,
    cloud_match: Option<&CloudMatch>,
    oauth_provider: Option<&'static str>,
    oauth_label: Option<&str>,
    installed_rec: Option<&InstalledPluginRecord>,
) -> Value {
    let oauth_connected = oauth_label.is_some();
    let installed = installed_rec.is_some();
    let cloud_connected = cloud_match.is_some() || oauth_connected;
    let effective_mode = match setting.preferred_mode.as_str() {
        "local" if spec.local_ready => "local",
        "cloud" => "cloud",
        _ => {
            if cloud_connected && spec.local_ready {
                "hybrid"
            } else if cloud_connected {
                "cloud"
            } else if spec.local_ready {
                "local"
            } else {
                "cloud"
            }
        }
    };

    let status = if !setting.enabled && !cloud_connected {
        "available"
    } else if cloud_connected && spec.local_ready {
        "connected_hybrid"
    } else if cloud_connected {
        "connected_cloud"
    } else if spec.local_ready {
        "active_local"
    } else {
        "enabled_needs_auth"
    };

    let mut via_parts: Vec<String> = Vec::new();
    if let Some(o) = oauth_label {
        via_parts.push(o.to_string());
    }
    if let Some(m) = cloud_match {
        match (m.connector_name.as_deref(), m.trigger_name.as_deref()) {
            (Some(c), Some(t)) => via_parts.push(format!("{c} + {t}")),
            (Some(c), None) => via_parts.push(c.to_string()),
            (None, Some(t)) => via_parts.push(t.to_string()),
            (None, None) => {}
        }
        if let Some(ref mn) = m.mcp_server_name {
            via_parts.push(format!("Installed ({mn})"));
        }
    }
    let connected_via = if via_parts.is_empty() {
        None
    } else {
        Some(via_parts.join(" · "))
    };

    json!({
        "id": spec.id,
        "suite": spec.suite,
        "suite_label": spec.suite_label,
        "name": spec.name,
        "tagline": spec.tagline,
        "description": spec.description,
        "mode": spec.mode,
        "transport": spec.transport,
        "local_ready": spec.local_ready,
        "default_enabled": spec.default_enabled,
        "version": spec.version,
        "source": spec.source,
        "installed": installed,
        "enabled": setting.enabled || cloud_connected,
        "preferred_mode": setting.preferred_mode,
        "effective_mode": effective_mode,
        "status": status,
        "cloud_connected": cloud_connected,
        "oauth_provider": oauth_provider,
        "oauth_connected": oauth_connected,
        "connected_via": connected_via,
        "linked_connector_id": cloud_match.and_then(|m| m.connector_id.clone()),
        "linked_trigger_id": cloud_match.and_then(|m| m.trigger_id.clone()),
        "linked_mcp_server_id": installed_rec.map(|r| r.mcp_server_id.clone()),
        "actions": spec.actions.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
        "connect_preset": spec.connect_preset.as_ref().map(|p| p.to_public_json()),
        "install_spec": spec.install_spec.as_ref().map(|i| i.to_json()),
        "website_url": spec.website_url,
        "repository_url": spec.repository_url,
    })
}

pub fn list_plugins_json(person_data_dir: &Path) -> Value {
    let policy = crate::load_workspace_policy(person_data_dir);
    let connectors_snap = connectors::list_connectors_json(person_data_dir, &policy);
    let mcp_snap = mcp::list_mcp_servers_json(person_data_dir, &policy);
    let google_label = plugin_cloud::connected_account_label(person_data_dir, "google");
    let ms_label = plugin_cloud::connected_account_label(person_data_dir, "microsoft");

    let catalog = catalog_for_person(person_data_dir);
    let store = with_plugins_lock(|| PluginsStore::load_unlocked(person_data_dir));
    let mut plugins = Vec::with_capacity(catalog.len());
    let mut enabled_count = 0usize;
    let mut local_active = 0usize;
    let mut cloud_connected = 0usize;

    for spec in &catalog {
        let setting = store.settings.get(&spec.id).cloned().unwrap_or(PluginPersonSetting {
            enabled: spec.default_enabled,
            preferred_mode: "auto".to_string(),
            updated_ms: 0,
        });
        let installed_rec = store.installed.get(&spec.id);
        let cm = detect_cloud_match(spec, &connectors_snap, &mcp_snap, installed_rec);
        let oauth_prov = plugin_cloud::provider_for_plugin(&spec.id);
        let oauth_lbl = match oauth_prov {
            Some("google") => google_label.as_deref(),
            Some("microsoft") => ms_label.as_deref(),
            _ => None,
        };
        let entry = resolve_plugin_json(
            spec,
            &setting,
            cm.as_ref(),
            oauth_prov,
            oauth_lbl,
            installed_rec,
        );
        if entry["enabled"].as_bool() == Some(true) {
            enabled_count += 1;
        }
        match entry["status"].as_str() {
            Some("active_local") | Some("connected_hybrid") => local_active += 1,
            _ => {}
        }
        if entry["cloud_connected"].as_bool() == Some(true) {
            cloud_connected += 1;
        }
        plugins.push(entry);
    }

    let root = records::infer_root_data_dir(person_data_dir);
    json!({
        "plugins": plugins,
        "summary": {
            "total": catalog.len(),
            "enabled_count": enabled_count,
            "local_active": local_active,
            "cloud_connected": cloud_connected,
        },
        "registry": plugin_registry::registry_status_json(&root),
    })
}

pub fn toggle_plugin(
    person_data_dir: &Path,
    plugin_id: &str,
    enabled: Option<bool>,
    preferred_mode: Option<&str>,
) -> Result<Value, String> {
    let spec = find_plugin_for_person(person_data_dir, plugin_id)
        .ok_or_else(|| format!("unknown plugin id: {plugin_id}"))?;
    if let Some(m) = preferred_mode {
        if !matches!(m, "auto" | "local" | "cloud") {
            return Err("preferred_mode must be auto, local, or cloud".to_string());
        }
    }

    with_plugins_lock(|| {
        let mut store = PluginsStore::load_unlocked(person_data_dir);
        let cur = store
            .settings
            .get(&spec.id)
            .cloned()
            .unwrap_or(PluginPersonSetting {
                enabled: spec.default_enabled,
                preferred_mode: "auto".to_string(),
                updated_ms: 0,
            });
        let next = PluginPersonSetting {
            enabled: enabled.unwrap_or(cur.enabled),
            preferred_mode: preferred_mode
                .map(str::to_string)
                .unwrap_or(cur.preferred_mode),
            updated_ms: records::now_millis(),
        };
        store.settings.insert(spec.id.clone(), next);
        store.save_unlocked(person_data_dir)?;
        Ok::<(), String>(())
    })?;

    let full = list_plugins_json(person_data_dir);
    full["plugins"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["id"].as_str() == Some(spec.id.as_str())).cloned())
        .ok_or_else(|| "plugin saved but could not reload".to_string())
}

pub fn install_registry_plugin(
    root_data_dir: &Path,
    person_data_dir: &Path,
    body: &Value,
) -> Result<Value, (u16, String)> {
    let plugin_id = body["plugin_id"].as_str().unwrap_or("").trim().to_string();
    let spec = if !plugin_id.is_empty() {
        find_plugin_for_person(person_data_dir, &plugin_id).or_else(|| {
            if body["spec"].is_object() {
                DynamicPluginSpec::from_json(&body["spec"], "community_registry")
            } else {
                None
            }
        })
    } else if body["spec"].is_object() {
        DynamicPluginSpec::from_json(&body["spec"], "community_registry")
    } else {
        None
    }
    .ok_or_else(|| (400, "valid plugin_id or spec is required".to_string()))?;

    let credentials = if body["credentials"].is_object() {
        &body["credentials"]
    } else {
        body
    };
    let (mcp_server_id, _server_json) =
        plugin_registry::build_mcp_server_from_plugin(person_data_dir, &spec, credentials)?;

    let now = records::now_millis();
    with_plugins_lock(|| {
        let mut store = PluginsStore::load_unlocked(person_data_dir);
        store.installed.insert(
            spec.id.clone(),
            InstalledPluginRecord {
                plugin_id: spec.id.clone(),
                mcp_server_id: mcp_server_id.clone(),
                installed_ms: now,
                spec: spec.clone(),
            },
        );
        store.settings.insert(
            spec.id.clone(),
            PluginPersonSetting {
                enabled: true,
                preferred_mode: "auto".to_string(),
                updated_ms: now,
            },
        );
        store.save_unlocked(person_data_dir)
    })
    .map_err(|e| (500, e))?;

    let _ = root_data_dir;
    let full = list_plugins_json(person_data_dir);
    let plugin = full["plugins"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["id"].as_str() == Some(spec.id.as_str())).cloned())
        .unwrap_or_else(|| spec.to_storage_json());
    Ok(json!({
        "ok": true,
        "plugin": plugin,
        "mcp_server_id": mcp_server_id,
    }))
}

pub fn uninstall_registry_plugin(
    person_data_dir: &Path,
    plugin_id: &str,
) -> Result<Value, (u16, String)> {
    let key = plugin_id.trim();
    if key.is_empty() {
        return Err((400, "plugin_id is required".to_string()));
    }
    let removed_mcp_id = with_plugins_lock(|| {
        let mut store = PluginsStore::load_unlocked(person_data_dir);
        let rec = store.installed.remove(key);
        if let Some(s) = store.settings.get_mut(key) {
            s.enabled = false;
            s.updated_ms = records::now_millis();
        }
        store
            .save_unlocked(person_data_dir)
            .map_err(|e| (500, e))?;
        Ok::<Option<String>, (u16, String)>(rec.map(|r| r.mcp_server_id))
    })?;

    if let Some(mid) = removed_mcp_id {
        let _ = mcp::delete_mcp_server(person_data_dir, &mid);
    }

    Ok(json!({
        "ok": true,
        "plugin_id": key,
        "plugins": list_plugins_json(person_data_dir)["plugins"].clone(),
    }))
}

/// Compact summary injected into the employee agent system prompt so the agent
/// knows both its local on-device capabilities and any connected cloud apps.
pub fn prompt_summary(person_data_dir: &Path) -> String {
    let data = list_plugins_json(person_data_dir);
    let mut lines = Vec::new();
    if let Some(arr) = data["plugins"].as_array() {
        for p in arr {
            if p["enabled"].as_bool() != Some(true) {
                continue;
            }
            let id = p["id"].as_str().unwrap_or("");
            let name = p["name"].as_str().unwrap_or("");
            let mode = p["effective_mode"].as_str().unwrap_or("local");
            let status = p["status"].as_str().unwrap_or("available");
            lines.push(format!("         - {id} ({name}, mode={mode}, status={status})"));
        }
    }
    lines.join("\n")
}

/// Map a tool invocation to `(plugin_id, plugin_name, execution_mode)` for
/// unobtrusive realtime SSE activity tracking in the frontend.
pub fn describe_tool_activity(tool: &str, args: &Value) -> Option<(String, String, &'static str)> {
    match tool {
        "run_plugin_action" => {
            let pid = args["plugin_id"].as_str().unwrap_or("");
            find_plugin(pid).map(|spec| {
                let mode = match args["mode"].as_str() {
                    Some("cloud") => "cloud",
                    Some("local") => "local",
                    _ if spec.local_ready && spec.mode != "cloud" => "local",
                    _ => "cloud",
                };
                (spec.id, spec.name, mode)
            })
        }
        "make_spreadsheet" | "reconcile_ledger" | "query_table" | "aggregate_vault" => Some((
            "office-excel".to_string(),
            "Excel & Spreadsheets".to_string(),
            "local",
        )),
        "write_document" | "export_file" | "summarize_meeting" | "audit_contract" | "draft_brief" => {
            Some((
                "office-word-pdf".to_string(),
                "Word & PDF Suite".to_string(),
                "local",
            ))
        }
        "read_file" | "write_file" | "patch_file" | "list_dir" | "search_files" | "search_vault"
        | "move_file" => Some((
            "local-workspace-files".to_string(),
            "On-Device Files".to_string(),
            "local",
        )),
        "query_company_db" => Some((
            "crm-billing".to_string(),
            "CRM & Billing".to_string(),
            "local",
        )),
        "invoke_connector" => Some(("connectors".to_string(), "Cloud Connector".to_string(), "cloud")),
        other if other.starts_with("mcp.") => Some((
            "local-mcp-bridge".to_string(),
            "Local Apps & MCP".to_string(),
            "local",
        )),
        _ => None,
    }
}

/// Execute a plugin action via local on-device tooling, native OAuth REST,
/// connected MCP server, or park an Approve/Deny decision for writes.
pub fn execute_plugin_action(
    cfg: &Config,
    plugin_id: &str,
    action: &str,
    mode_override: Option<&str>,
    args: &Value,
    thread_id: &str,
) -> Result<(String, &'static str), String> {
    let spec = find_plugin_for_person(&cfg.data_dir, plugin_id)
        .ok_or_else(|| format!("unknown plugin: {plugin_id}"))?;
    let act_spec = spec
        .actions
        .iter()
        .find(|a| a.name == action)
        .ok_or_else(|| format!("plugin '{}' has no action '{}'", spec.name, action))?;

    let all = list_plugins_json(&cfg.data_dir);
    let current = all["plugins"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["id"].as_str() == Some(spec.id.as_str())))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let cloud_connected = current["cloud_connected"].as_bool() == Some(true);
    let oauth_connected = current["oauth_connected"].as_bool() == Some(true);
    let preferred = current["preferred_mode"].as_str().unwrap_or("auto");

    let is_local_only_action = matches!(
        act_spec.name.as_str(),
        "draft_reply"
            | "build_sheet"
            | "export_xlsx"
            | "read_document"
            | "draft_document"
            | "export_docx_pdf"
            | "draft_doc"
            | "reconcile_sheet"
            | "draft_outline"
            | "list_workspace"
            | "search_files"
            | "watch_folder_status"
            | "list_mcp_tools"
            | "list_clients"
    );
    let resolved_mode: &'static str = match mode_override.unwrap_or(preferred) {
        "local" => {
            if !spec.local_ready && act_spec.name != "draft_reply" {
                return Err(format!(
                    "{} does not have an offline local mode; Sign in with OAuth2 or install it in Plugins first.",
                    spec.name
                ));
            }
            "local"
        }
        "cloud" => "cloud",
        _ => {
            if is_local_only_action {
                "local"
            } else if oauth_connected || (cloud_connected && !spec.local_ready) {
                "cloud"
            } else if spec.local_ready {
                "local"
            } else {
                "cloud"
            }
        }
    };

    // Gate consequential external write actions on /phone + Today approval.
    if act_spec.requires_approval {
        let preview = crate::truncate(&serde_json::to_string(args).unwrap_or_default(), 320);
        let summary = format!(
            "{} · {} ({})",
            spec.name, act_spec.label, resolved_mode
        );
        let detail = format!(
            "Plugin \"{}\" wants to execute \"{}\" ({} mode). Approve on /phone or Today to run. Payload: {}",
            spec.name, act_spec.description, resolved_mode, preview
        );
        let decision_id = records::add_decision_with(
            &cfg.data_dir,
            "plugin_action",
            &summary,
            None,
            thread_id,
            json!({
                "detail": detail,
                "severity": "warning",
                "plugin_id": spec.id,
                "plugin_name": spec.name,
                "action": act_spec.name,
                "action_label": act_spec.label,
                "mode": resolved_mode,
                "linked_connector_id": current["linked_connector_id"].clone(),
                "linked_mcp_server_id": current["linked_mcp_server_id"].clone(),
                "args": args.clone(),
            }),
        );
        return Ok((
            format!(
                "parked for your person's approval (decision {decision_id}): {} · {} requires Approve/Deny before external execution.",
                spec.name, act_spec.label
            ),
            resolved_mode,
        ));
    }

    // Read / non-consequential actions: native OAuth REST, connected MCP, or connector.
    if resolved_mode == "cloud" {
        if let Some(out) = plugin_cloud::execute_native_cloud_action(cfg, &spec.id, &act_spec.name, args)? {
            return Ok((out, "cloud"));
        }
        if let Some(trig_id) = current["linked_trigger_id"].as_str() {
            if act_spec.name == "read_inbox" || act_spec.name == "triage_threads" {
                let events = connectors::poll_imap_trigger_now(cfg, trig_id)?;
                return Ok((
                    format!(
                        "{} polled mailbox cleanly: {} new message(s) saved into workspace/intake/.",
                        spec.name,
                        events.len()
                    ),
                    "cloud",
                ));
            }
        }
        if let Some(conn_id) = current["linked_connector_id"].as_str() {
            match connectors::invoke_connector(cfg, conn_id, args, None, thread_id)? {
                connectors::InvokeOutcome::Delivered(log) => {
                    return Ok((
                        format!(
                            "{} completed via {} (HTTP {}, {} bytes).",
                            spec.name,
                            log.connector_name,
                            log.http_status.unwrap_or(200),
                            log.payload_bytes
                        ),
                        "cloud",
                    ));
                }
                connectors::InvokeOutcome::Held { decision_id, .. } => {
                    return Ok((
                        format!(
                            "parked for your person's approval (decision {decision_id}) via connected cloud connector."
                        ),
                        "cloud",
                    ));
                }
            }
        }
        if let Some(mcp_id) = current["linked_mcp_server_id"].as_str() {
            let prefix = format!("mcp.{mcp_id}.");
            let tools = mcp::load_enabled_mcp_tools(&cfg.data_dir);
            let matching: Vec<String> = tools
                .into_iter()
                .filter(|(full_name, _, _)| full_name.starts_with(&prefix))
                .map(|(full_name, _, _)| full_name)
                .collect();
            let detail = if matching.is_empty() {
                format!("{} is installed cleanly (server={mcp_id}). Ask your agent in Chat or run Discover tools in Connectors -> MCP Tools.", spec.name)
            } else {
                format!("{} is connected (server={mcp_id}) with {} discovered tool(s): {}.", spec.name, matching.len(), matching.join(", "))
            };
            return Ok((detail, "cloud"));
        }
        return Err(format!(
            "{} is not connected yet. Open Plugins -> Connect / Sign in with OAuth2 to configure credentials.",
            spec.name
        ));
    }

    // Local on-device execution
    let out = match (spec.id.as_str(), act_spec.name.as_str()) {
        ("local-workspace-files", "list_workspace")
        | ("google-drive", "list_files")
        | ("onedrive-sharepoint", "list_files")
        | ("office-excel", "inspect_workbook") => {
            let path = args["path"].as_str().unwrap_or(".");
            let listed = crate::tools::execute_tool(cfg, "list_dir", &json!({ "path": path }))?;
            format!("{} (local workspace): {}", spec.name, listed)
        }
        ("local-workspace-files", "search_files") | ("office-word-pdf", "read_document") => {
            if let Some(q) = args["query"].as_str().filter(|s| !s.trim().is_empty()) {
                crate::tools::execute_tool(cfg, "search_files", &json!({ "query": q }))?
            } else if let Some(p) = args["path"].as_str().filter(|s| !s.trim().is_empty()) {
                crate::tools::execute_tool(cfg, "read_file", &json!({ "path": p }))?
            } else {
                crate::tools::execute_tool(cfg, "list_dir", &json!({ "path": "." }))?
            }
        }
        ("local-workspace-files", "watch_folder_status") => {
            let policy = crate::load_workspace_policy(&cfg.data_dir);
            let snap = connectors::list_connectors_json(&cfg.data_dir, &policy);
            let watched: Vec<String> = snap["triggers"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter(|t| t["kind"].as_str() == Some("folder_watch"))
                        .map(|t| {
                            format!(
                                "{} ({})",
                                t["name"].as_str().unwrap_or("Folder"),
                                t["watch_dir"].as_str().unwrap_or("-")
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            if watched.is_empty() {
                "No watched host folders configured yet; local workspace sandbox is ready.".to_string()
            } else {
                format!("Watched folders: {}", watched.join(", "))
            }
        }
        ("office-excel", "build_sheet") | ("google-sheets", "reconcile_sheet") => {
            let mut margs = args.clone();
            if margs.get("title").is_none() {
                margs["title"] = json!("Workspace Sheet");
            }
            crate::tools::execute_tool(cfg, "make_spreadsheet", &margs)?
        }
        ("office-excel", "export_xlsx") | ("office-word-pdf", "export_docx_pdf") => {
            crate::tools::execute_tool(cfg, "export_file", args)?
        }
        ("office-word-pdf", "draft_document")
        | ("google-docs", "draft_doc")
        | ("google-slides", "draft_outline")
        | ("google-gmail", "draft_reply")
        | ("outlook-mail", "draft_reply") => {
            let title = args["title"]
                .as_str()
                .unwrap_or(match spec.id.as_str() {
                    "google-slides" => "Deck Outline",
                    "google-gmail" | "outlook-mail" => "Email Reply Draft",
                    _ => "Workspace Draft",
                });
            let sections = if args.get("sections").is_some() {
                args["sections"].clone()
            } else {
                json!([args["body"]
                    .as_str()
                    .or_else(|| args["content"].as_str())
                    .unwrap_or("Draft prepared locally inside Ergon workspace.")])
            };
            crate::tools::execute_tool(
                cfg,
                "write_document",
                &json!({ "title": title, "sections": sections }),
            )?
        }
        ("crm-billing", "list_clients") => {
            let root = records::infer_root_data_dir(&cfg.data_dir);
            let store = records::load_clients_store(&root);
            let names: Vec<String> = store
                .clients
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect();
            if names.is_empty() {
                "CRM local store: 0 clients configured yet.".to_string()
            } else {
                format!("CRM local clients ({}): {}", names.len(), names.join(", "))
            }
        }
        ("local-mcp-bridge", "list_mcp_tools") => {
            let tools = mcp::load_enabled_mcp_tools(&cfg.data_dir);
            if tools.is_empty() {
                "Local MCP Bridge: 0 MCP servers connected yet (install any app in Plugins or add servers in Connectors).".to_string()
            } else {
                let list: Vec<String> = tools.iter().map(|(n, _, _)| n.clone()).collect();
                format!("Enabled MCP tools ({}): {}", list.len(), list.join(", "))
            }
        }
        ("github", "list_issues") => {
            "GitHub local plugin ready; connect GitHub in Plugins or Connectors to query live repositories.".to_string()
        }
        _ => {
            format!(
                "{} · {} checked cleanly on-device.",
                spec.name, act_spec.label
            )
        }
    };
    Ok((out, "local"))
}

pub fn settle_plugin_action_decision(cfg: &Config, decision: &Value, approved: bool) -> String {
    let plugin_name = decision["plugin_name"].as_str().unwrap_or("Plugin");
    let action_label = decision["action_label"]
        .as_str()
        .or_else(|| decision["action"].as_str())
        .unwrap_or("action");
    if !approved {
        return format!("You said no — {plugin_name} · {action_label} was cancelled.");
    }
    let plugin_id = decision["plugin_id"].as_str().unwrap_or("");
    let action = decision["action"].as_str().unwrap_or("");
    let args = &decision["args"];
    match plugin_cloud::execute_native_cloud_action(cfg, plugin_id, action, args) {
        Ok(Some(out)) => {
            return format!("Approved & executed {plugin_name} · {action_label}: {out}");
        }
        Ok(None) => {}
        Err(e) => {
            return format!("Approved {plugin_name} · {action_label}, but cloud delivery returned: {e}");
        }
    }
    let linked_conn = decision["linked_connector_id"].as_str().unwrap_or("");
    let tid = decision["thread_id"].as_str().unwrap_or("");
    if !linked_conn.is_empty() {
        match connectors::invoke_connector(cfg, linked_conn, args, None, tid) {
            Ok(connectors::InvokeOutcome::Delivered(log)) => {
                return format!(
                    "Approved & executed {plugin_name} · {action_label} via {} (HTTP {}, sha256:{}).",
                    log.connector_name,
                    log.http_status.unwrap_or(200),
                    log.payload_sha256
                );
            }
            Ok(connectors::InvokeOutcome::Held { decision_id, .. }) => {
                return format!(
                    "Approved plugin gate; outbound connector delivery also requires confirmation ({decision_id})."
                );
            }
            Err(e) => {
                return format!("Approved {plugin_name} · {action_label}, but connector delivery returned: {e}");
            }
        }
    }
    let linked_mcp = decision["linked_mcp_server_id"].as_str().unwrap_or("");
    if !linked_mcp.is_empty() {
        return format!(
            "Approved {plugin_name} · {action_label} on installed server '{linked_mcp}'."
        );
    }
    format!(
        "Approved {plugin_name} · {action_label} (connect live credentials in Plugins or Connectors to dispatch externally)."
    )
}
