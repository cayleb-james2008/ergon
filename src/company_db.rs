//! Shared Company Database & Company Memory (Step 3a).
//!
//! Provides:
//! - Role- and client-scoped company memory persisted under
//!   `<root_data_dir>/company/company_memory.json` with `0600` POSIX permissions
//!   and atomic temp+rename writes guarded by `COMPANY_LOCK`.
//! - Read-only RBAC-enforced queries over company-wide structured tables
//!   (`people`, `roles`, `clients`, `work`, `receipts`, `tools`, `outcomes`,
//!   `memory`, `summary`).
//!
//! Never reads, modifies, or exposes any person's private `memory.md`,
//! passwords, or session tokens.

use crate::auth::Role;
use crate::library::Library;
use crate::receipts::load_chain;
use crate::records::{load_clients_store, load_work, now_millis, valid_entity_id, ClientsStore};
use crate::tools::atomic_write_bytes_with_mode;
use crate::{backup_corrupt, truncate, valid_person_id, Config, PeopleState};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

const MAX_MEMORY_CONTENT_CHARS: usize = 2000;
const MAX_PROMPT_MEMORY_ENTRIES: usize = 25;
const DEFAULT_QUERY_LIMIT: usize = 50;
const MAX_QUERY_LIMIT: usize = 200;

static COMPANY_LOCK: Mutex<()> = Mutex::new(());
static MEMORY_SEQ: AtomicU64 = AtomicU64::new(0);

fn with_company_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = COMPANY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

/// Directory storing company-wide shared database state (`<root_data_dir>/company`).
pub fn company_dir(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("company")
}

fn company_memory_path(root_data_dir: &Path) -> PathBuf {
    company_dir(root_data_dir).join("company_memory.json")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanyMemoryEntry {
    pub id: String,
    pub category: String,
    pub scope: String,
    pub content: String,
    pub author_person_id: String,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl CompanyMemoryEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "category": self.category,
            "scope": self.scope,
            "content": self.content,
            "author_person_id": self.author_person_id,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        if !valid_entity_id(&id) {
            return None;
        }
        let category = normalize_category(v["category"].as_str()?).ok()?;
        let scope = normalize_scope(v["scope"].as_str()?).ok()?;
        let raw_content = v["content"].as_str()?;
        if raw_content.contains('\0') {
            return None;
        }
        let content = raw_content.trim().to_string();
        if content.is_empty() || content.chars().count() > MAX_MEMORY_CONTENT_CHARS {
            return None;
        }
        let author_person_id = v["author_person_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| valid_person_id(s))
            .unwrap_or("default")
            .to_string();
        let created_ms = v["created_ms"].as_u64().unwrap_or(0);
        let updated_ms = v["updated_ms"].as_u64().unwrap_or(created_ms);
        Some(Self {
            id,
            category,
            scope,
            content,
            author_person_id,
            created_ms,
            updated_ms,
        })
    }
}

fn normalize_category(raw: &str) -> Result<String, (u16, String)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "fact" => Ok("fact".to_string()),
        "procedure" => Ok("procedure".to_string()),
        "preference" => Ok("preference".to_string()),
        "policy" => Ok("policy".to_string()),
        _ => Err((
            400,
            "invalid category: expected 'fact', 'procedure', 'preference', or 'policy'".to_string(),
        )),
    }
}

fn normalize_scope(raw: &str) -> Result<String, (u16, String)> {
    let s = raw.trim().to_ascii_lowercase();
    if s == "company" {
        return Ok("company".to_string());
    }
    if let Some(role_name) = s.strip_prefix("role:") {
        let Some(r) = Role::parse_strict(role_name) else {
            return Err((400, format!("invalid role in scope: '{role_name}'")));
        };
        return Ok(format!("role:{}", r.as_str()));
    }
    if let Some(client_id) = s.strip_prefix("client:") {
        if !valid_entity_id(client_id) {
            return Err((
                400,
                "invalid client id in scope: must be 1..=64 lowercase alphanumeric, '-', or '_'"
                    .to_string(),
            ));
        }
        return Ok(format!("client:{client_id}"));
    }
    Err((
        400,
        "invalid scope: expected 'company', 'role:<role>', or 'client:<client_id>'".to_string(),
    ))
}

fn contains_secret_or_host_path(content: &str) -> bool {
    let low = content.to_ascii_lowercase();
    let bytes = low.as_bytes();

    // Check `sk-` at a non-alphanumeric boundary so normal words like `task-1`
    // or `desk-check` do not trigger false positives.
    for (idx, _) in low.match_indices("sk-") {
        let prev_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
        if prev_ok {
            return true;
        }
    }

    const PATTERNS: [&str; 10] = [
        "ergk_",
        "ergs_",
        "password=",
        "api_key=",
        "secret=",
        "bearer ",
        "-----begin ",
        "/home/",
        "c:\\users\\",
        "c:/users/",
    ];
    PATTERNS.iter().any(|p| low.contains(p))
}

fn validate_memory_content(raw: &str) -> Result<String, (u16, String)> {
    if raw.contains('\0') {
        return Err((
            400,
            "company memory content cannot contain null bytes".to_string(),
        ));
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err((400, "company memory content cannot be empty".to_string()));
    }
    if trimmed.chars().count() > MAX_MEMORY_CONTENT_CHARS {
        return Err((
            400,
            format!("company memory content exceeds {MAX_MEMORY_CONTENT_CHARS} characters"),
        ));
    }
    if contains_secret_or_host_path(trimmed) {
        return Err((
            400,
            "company memory cannot contain secrets or host paths".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn load_company_memory_unlocked(
    root_data_dir: &Path,
    backup_if_corrupt: bool,
) -> Vec<CompanyMemoryEntry> {
    let path = company_memory_path(root_data_dir);
    let Ok(bytes) = std::fs::read(&path) else {
        return Vec::new();
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        if backup_if_corrupt {
            let _ = backup_corrupt(&path);
        }
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        if backup_if_corrupt {
            let _ = backup_corrupt(&path);
        }
        return Vec::new();
    };
    let Some(arr) = v["entries"].as_array() else {
        if backup_if_corrupt {
            let _ = backup_corrupt(&path);
        }
        return Vec::new();
    };
    arr.iter()
        .filter_map(CompanyMemoryEntry::from_json)
        .collect()
}

fn save_company_memory_unlocked(
    root_data_dir: &Path,
    entries: &[CompanyMemoryEntry],
) -> Result<(), (u16, String)> {
    let path = company_memory_path(root_data_dir);
    let payload = json!({
        "entries": entries.iter().map(CompanyMemoryEntry::to_json).collect::<Vec<_>>(),
    });
    let serialized = serde_json::to_string_pretty(&payload)
        .map_err(|e| (500, format!("serialize company memory: {e}")))?;
    atomic_write_bytes_with_mode(&path, serialized.as_bytes(), Some(0o600))
        .map_err(|e| (500, format!("save company memory: {e}")))
}

/// Load all persisted company memory entries without role filtering.
pub fn load_company_memory(root_data_dir: &Path) -> Vec<CompanyMemoryEntry> {
    with_company_lock(|| load_company_memory_unlocked(root_data_dir, false))
}

/// Determine whether a principal with `role` and `allowed_client_ids` may read `entry`.
pub fn can_read_entry(
    entry: &CompanyMemoryEntry,
    role: Role,
    allowed_client_ids: &[String],
) -> bool {
    if entry.scope == "company" {
        return true;
    }
    if let Some(r) = entry.scope.strip_prefix("role:") {
        return role.is_admin() || role.as_str() == r;
    }
    if let Some(cid) = entry.scope.strip_prefix("client:") {
        return role.can_manage_clients()
            && (role.is_admin()
                || allowed_client_ids.is_empty()
                || allowed_client_ids.iter().any(|c| c == cid));
    }
    false
}

/// List all company memory entries visible to `role` and `allowed_client_ids`.
pub fn list_company_memory_for(
    root_data_dir: &Path,
    role: Role,
    allowed_client_ids: &[String],
) -> Vec<CompanyMemoryEntry> {
    load_company_memory(root_data_dir)
        .into_iter()
        .filter(|e| can_read_entry(e, role, allowed_client_ids))
        .collect()
}

/// Append a new entry to shared company memory (`admin | pm | ae | office` only).
pub fn add_company_memory(
    root_data_dir: &Path,
    role: Role,
    author_person_id: &str,
    category: &str,
    scope: &str,
    content: &str,
) -> Result<CompanyMemoryEntry, (u16, String)> {
    if !role.can_manage_clients() {
        return Err((
            403,
            format!(
                "forbidden: role '{}' cannot add company memory",
                role.as_str()
            ),
        ));
    }
    let author = author_person_id.trim();
    if !valid_person_id(author) {
        return Err((400, "invalid author_person_id".to_string()));
    }
    let category = normalize_category(category)?;
    let scope = normalize_scope(scope)?;
    let content = validate_memory_content(content)?;

    with_company_lock(|| {
        let mut entries = load_company_memory_unlocked(root_data_dir, true);
        let now = now_millis();
        let seq = MEMORY_SEQ.fetch_add(1, Ordering::SeqCst);
        let mut id = format!("cmem-{now}-{seq}");
        while entries.iter().any(|e| e.id == id) {
            let next = MEMORY_SEQ.fetch_add(1, Ordering::SeqCst);
            id = format!("cmem-{now}-{next}");
        }
        let entry = CompanyMemoryEntry {
            id,
            category,
            scope,
            content,
            author_person_id: author.to_string(),
            created_ms: now,
            updated_ms: now,
        };
        entries.push(entry.clone());
        save_company_memory_unlocked(root_data_dir, &entries)?;
        Ok(entry)
    })
}

/// Delete a company memory entry by `id` (`admin | pm | ae | office` only).
pub fn delete_company_memory(
    root_data_dir: &Path,
    role: Role,
    id: &str,
) -> Result<bool, (u16, String)> {
    if !role.can_manage_clients() {
        return Err((
            403,
            format!(
                "forbidden: role '{}' cannot delete company memory",
                role.as_str()
            ),
        ));
    }
    let id = id.trim();
    if !valid_entity_id(id) {
        return Ok(false);
    }
    with_company_lock(|| {
        let mut entries = load_company_memory_unlocked(root_data_dir, true);
        let before = entries.len();
        entries.retain(|e| e.id != id);
        if entries.len() == before {
            return Ok(false);
        }
        save_company_memory_unlocked(root_data_dir, &entries)?;
        Ok(true)
    })
}

/// Render authorized company memory entries as a read-only prompt section,
/// capped at 25 entries and kept strictly separate from private memory.
pub fn render_company_memory_for_prompt(
    root_data_dir: &Path,
    role: Role,
    allowed_client_ids: &[String],
) -> String {
    let entries = list_company_memory_for(root_data_dir, role, allowed_client_ids);
    if entries.is_empty() {
        return String::new();
    }
    let start = entries.len().saturating_sub(MAX_PROMPT_MEMORY_ENTRIES);
    let mut out = String::from(
        "## Shared Company Memory (Company-Wide Reference — Separate from Private Memory)\n\
         Authorized shared facts, procedures, preferences, and policies for your role:\n",
    );
    for entry in &entries[start..] {
        let flat: String = entry
            .content
            .replace(['\r', '\n'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let body = truncate(&flat, MAX_MEMORY_CONTENT_CHARS);
        out.push_str(&format!(
            "- [{} | {}] {}\n",
            entry.category, entry.scope, body
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Company DB Query Engine
// ---------------------------------------------------------------------------

fn value_matches_query(v: &Value, q_lower: &str) -> bool {
    match v {
        Value::String(s) => s.to_ascii_lowercase().contains(q_lower),
        Value::Number(n) => n.to_string().to_ascii_lowercase().contains(q_lower),
        Value::Bool(b) => b.to_string().contains(q_lower),
        Value::Array(arr) => arr.iter().any(|x| value_matches_query(x, q_lower)),
        Value::Object(map) => map.values().any(|x| value_matches_query(x, q_lower)),
        Value::Null => false,
    }
}

fn filter_and_cap(rows: Vec<Value>, q: Option<&str>, limit: usize) -> (usize, Vec<Value>) {
    let filtered: Vec<Value> = match q {
        Some(q_lower) => rows
            .into_iter()
            .filter(|r| value_matches_query(r, q_lower))
            .collect(),
        None => rows,
    };
    let total = filtered.len();
    let capped = filtered.into_iter().take(limit).collect();
    (total, capped)
}

fn person_data_dir(root_cfg: &Config, person_id: &str) -> Option<PathBuf> {
    if !valid_person_id(person_id) {
        return None;
    }
    if person_id == "default" {
        Some(root_cfg.data_dir.clone())
    } else {
        Some(root_cfg.data_dir.join("people").join(person_id))
    }
}

fn target_people_dirs(
    root_cfg: &Config,
    people: &PeopleState,
    principal_person_id: &str,
    role: Role,
) -> Vec<(String, PathBuf)> {
    if role.is_admin() {
        let mut out = Vec::new();
        for p in &people.people {
            if let Some(dir) = person_data_dir(root_cfg, &p.id) {
                out.push((p.id.clone(), dir));
            }
        }
        return out;
    }
    match person_data_dir(root_cfg, principal_person_id) {
        Some(dir) => vec![(principal_person_id.to_string(), dir)],
        None => Vec::new(),
    }
}

fn visible_clients_and_engagements(
    store: &ClientsStore,
    principal_person_id: &str,
    role: Role,
) -> (Vec<Value>, Vec<Value>, Vec<String>) {
    let mut client_rows = Vec::new();
    let mut visible_client_ids = Vec::new();
    let mut engagement_rows = Vec::new();

    for client in &store.clients {
        let is_visible = role.is_admin()
            || client.owner_person_id.is_none()
            || client.owner_person_id.as_deref() == Some(principal_person_id)
            || store
                .engagements
                .iter()
                .any(|e| e.client_id == client.id && e.person_id == principal_person_id);
        if !is_visible {
            continue;
        }
        visible_client_ids.push(client.id.clone());

        let client_owned_or_unowned = role.is_admin()
            || client.owner_person_id.is_none()
            || client.owner_person_id.as_deref() == Some(principal_person_id);

        let engs: Vec<Value> = store
            .engagements
            .iter()
            .filter(|e| {
                e.client_id == client.id
                    && (client_owned_or_unowned || e.person_id == principal_person_id)
            })
            .map(|e| e.to_json())
            .collect();

        engagement_rows.extend(engs.iter().cloned());
        let mut obj = client.to_json();
        obj["engagements"] = Value::Array(engs);
        client_rows.push(obj);
    }

    (client_rows, engagement_rows, visible_client_ids)
}

pub fn allowed_client_ids_for_memory(
    root_data_dir: &Path,
    principal_person_id: &str,
    role: Role,
) -> Vec<String> {
    if !role.can_manage_clients() {
        return Vec::new();
    }
    if role.is_admin() {
        return Vec::new();
    }
    let store = load_clients_store(root_data_dir);
    if store.clients.is_empty() {
        return Vec::new();
    }
    let (_, _, visible_ids) = visible_clients_and_engagements(&store, principal_person_id, role);
    if visible_ids.is_empty() {
        // Non-empty clients store where the caller owns/touches none: ensure
        // `can_read_entry` does not treat empty `allowed_client_ids` as wildcard.
        vec!["__none__".to_string()]
    } else {
        visible_ids
    }
}

fn role_summary_rows() -> Vec<Value> {
    Role::ALL
        .iter()
        .filter_map(|name| Role::parse_strict(name))
        .map(|r| {
            let desc = match r {
                Role::Admin => {
                    "Full workspace administration, user management, and cross-team visibility."
                }
                Role::Pm => {
                    "Manage clients, engagements, workspace policies, and shared company memory."
                }
                Role::Ae => "Manage clients, engagements, invoices, and shared company memory.",
                Role::Office => {
                    "Operations, finance, invoices, client management, and shared company memory."
                }
                Role::Installer => {
                    "Field delivery role scoped to own workspace and company/role memory."
                }
                Role::Member => "Standard member scoped to own workspace and company/role memory.",
            };
            json!({
                "role": r.as_str(),
                "is_admin": r.is_admin(),
                "can_manage_clients": r.can_manage_clients(),
                "can_view_invoices": r.can_view_invoices(),
                "can_manage_invoices": r.can_manage_invoices(),
                "can_manage_workspace_policy": r.can_manage_workspace_policy(),
                "description": desc,
            })
        })
        .collect()
}

fn collect_work_rows(
    root_cfg: &Config,
    people: &PeopleState,
    principal_person_id: &str,
    role: Role,
) -> Vec<Value> {
    let mut rows = Vec::new();
    for (pid, dir) in target_people_dirs(root_cfg, people, principal_person_id, role) {
        for entry in load_work(&dir) {
            let mut obj = json!({
                "person_id": pid,
                "ts": entry["ts"].as_u64().unwrap_or(0),
                "task": entry["task"].as_str().unwrap_or(""),
                "finished": entry["finished"].as_bool().unwrap_or(false),
                "answered": entry["answered"].as_bool().unwrap_or(false),
                "files": entry["files"].as_array().cloned().unwrap_or_default(),
            });
            if let Some(eid) = entry["engagement_id"].as_str().filter(|s| !s.is_empty()) {
                obj["engagement_id"] = json!(eid);
            }
            if let Some(tid) = entry["thread_id"].as_str().filter(|s| !s.is_empty()) {
                obj["thread_id"] = json!(tid);
            }
            rows.push(obj);
        }
    }
    rows.sort_by(|a, b| {
        b["ts"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["ts"].as_u64().unwrap_or(0))
    });
    rows
}

fn collect_receipt_rows(
    root_cfg: &Config,
    people: &PeopleState,
    principal_person_id: &str,
    role: Role,
) -> Vec<Value> {
    let mut rows = Vec::new();
    for (pid, dir) in target_people_dirs(root_cfg, people, principal_person_id, role) {
        for r in load_chain(&dir) {
            let mut obj = json!({
                "person_id": pid,
                "id": r.id,
                "index": r.index,
                "ts": r.ts,
                "task": r.task,
                "finished": r.finished,
                "answered": r.answered,
                "files": r.files,
                "prev_hash": r.prev_hash,
                "sha256": r.sha256,
            });
            if let Some(eid) = r.engagement_id.as_deref().filter(|s| !s.is_empty()) {
                obj["engagement_id"] = json!(eid);
            }
            if let Some(digest) = r.usage_digest() {
                obj["usage_digest"] = json!(digest);
            }
            rows.push(obj);
        }
    }
    rows.sort_by(|a, b| {
        b["ts"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["ts"].as_u64().unwrap_or(0))
            .then_with(|| {
                b["index"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["index"].as_u64().unwrap_or(0))
            })
    });
    rows
}

fn collect_tool_rows(library: &Library) -> Vec<Value> {
    library
        .items
        .iter()
        .filter(|i| matches!(i.kind.as_str(), "skill" | "tool" | "subagent" | "app"))
        .map(|i| {
            json!({
                "id": i.id,
                "kind": i.kind,
                "name": i.name,
                "description": i.description,
                "author": i.author,
                "version": i.version,
                "builtin": i.builtin,
            })
        })
        .collect()
}

fn collect_outcome_rows(root_data_dir: &Path) -> Vec<Value> {
    let path = root_data_dir.join("evolution").join("outcomes.jsonl");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut agg: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        let Some(artifact_id) = v["artifact_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let entry = agg.entry(artifact_id.to_string()).or_insert((0, 0, 0.0));
        entry.0 += 1;
        if let Some(score) = v["quality_score"].as_f64().filter(|s| s.is_finite()) {
            entry.1 += 1;
            entry.2 += score;
        }
    }
    let mut rows: Vec<Value> = agg
        .into_iter()
        .map(|(artifact_id, (count, scored_count, sum))| {
            let avg = if scored_count > 0 {
                let raw = sum / scored_count as f64;
                json!((raw * 10_000.0).round() / 10_000.0)
            } else {
                Value::Null
            };
            json!({
                "artifact_id": artifact_id,
                "count": count,
                "scored_count": scored_count,
                "avg_quality_score": avg,
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                a["artifact_id"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["artifact_id"].as_str().unwrap_or(""))
            })
    });
    rows
}

/// Query a company database table under RBAC and per-principal isolation rules.
#[allow(clippy::too_many_arguments)]
pub fn query_company_db(
    root_data_dir: &Path,
    root_cfg: &Config,
    people: &PeopleState,
    library: &Library,
    principal_person_id: &str,
    role: Role,
    table: &str,
    query: Option<&str>,
    limit: usize,
) -> Result<Value, (u16, String)> {
    let table_norm = table.trim().to_ascii_lowercase();
    let limit = if limit == 0 {
        DEFAULT_QUERY_LIMIT
    } else {
        limit.clamp(1, MAX_QUERY_LIMIT)
    };
    let q_owned = query
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty());
    let q = q_owned.as_deref();

    match table_norm.as_str() {
        "people" => {
            let rows: Vec<Value> = people
                .people
                .iter()
                .map(|p| json!({ "id": p.id, "name": p.name, "role": p.role }))
                .collect();
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "people", "total": total, "limit": limit, "rows": rows }))
        }
        "roles" => {
            let (total, rows) = filter_and_cap(role_summary_rows(), q, limit);
            Ok(json!({ "table": "roles", "total": total, "limit": limit, "rows": rows }))
        }
        "clients" => {
            if !role.can_manage_clients() {
                return Err((
                    403,
                    "forbidden: role cannot access client records".to_string(),
                ));
            }
            let store = load_clients_store(root_data_dir);
            let (client_rows, engagement_rows, _) =
                visible_clients_and_engagements(&store, principal_person_id, role);
            let (total, rows) = filter_and_cap(client_rows, q, limit);
            let (_, capped_engagements) = filter_and_cap(engagement_rows, q, limit);
            Ok(json!({
                "table": "clients",
                "total": total,
                "limit": limit,
                "rows": rows,
                "engagements": capped_engagements,
            }))
        }
        "work" => {
            let rows = collect_work_rows(root_cfg, people, principal_person_id, role);
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "work", "total": total, "limit": limit, "rows": rows }))
        }
        "receipts" => {
            let rows = collect_receipt_rows(root_cfg, people, principal_person_id, role);
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "receipts", "total": total, "limit": limit, "rows": rows }))
        }
        "tools" => {
            let rows = collect_tool_rows(library);
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "tools", "total": total, "limit": limit, "rows": rows }))
        }
        "outcomes" => {
            let rows = collect_outcome_rows(root_data_dir);
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "outcomes", "total": total, "limit": limit, "rows": rows }))
        }
        "memory" => {
            let allowed_clients = allowed_client_ids_for_memory(root_data_dir, principal_person_id, role);
            let entries = list_company_memory_for(root_data_dir, role, &allowed_clients);
            let rows: Vec<Value> = entries.iter().map(CompanyMemoryEntry::to_json).collect();
            let (total, rows) = filter_and_cap(rows, q, limit);
            Ok(json!({ "table": "memory", "total": total, "limit": limit, "rows": rows }))
        }
        "summary" => {
            let allowed_clients = allowed_client_ids_for_memory(root_data_dir, principal_person_id, role);
            let memory_count = list_company_memory_for(root_data_dir, role, &allowed_clients).len();
            let work_count = collect_work_rows(root_cfg, people, principal_person_id, role).len();
            let receipts_count =
                collect_receipt_rows(root_cfg, people, principal_person_id, role).len();
            let tools_count = collect_tool_rows(library).len();
            let outcomes_count = collect_outcome_rows(root_data_dir).len();

            let (clients_count, engagements_count) = if role.can_manage_clients() {
                let store = load_clients_store(root_data_dir);
                let (c, e, _) = visible_clients_and_engagements(&store, principal_person_id, role);
                (json!(c.len()), json!(e.len()))
            } else {
                (Value::Null, Value::Null)
            };

            Ok(json!({
                "table": "summary",
                "role": role.as_str(),
                "person_id": principal_person_id,
                "counts": {
                    "people": people.people.len(),
                    "roles": Role::ALL.len(),
                    "clients": clients_count,
                    "engagements": engagements_count,
                    "work": work_count,
                    "receipts": receipts_count,
                    "tools": tools_count,
                    "outcomes": outcomes_count,
                    "memory": memory_count,
                }
            }))
        }
        _ => Err((
            400,
            format!(
                "unknown table '{table}': expected people, roles, clients, work, receipts, tools, outcomes, memory, or summary"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::{create_client_for_person, create_engagement};
    use crate::Person;

    fn temp_root(label: &str) -> PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "ergon-company-db-{label}-{}-{id}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_config(root: &Path) -> Config {
        Config {
            data_dir: root.to_path_buf(),
            workspace_dir: root.join("workspace"),
            memory_path: root.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test-model".to_string(),
            api_key: None,
        }
    }

    #[test]
    fn memory_rbac_write_delete_and_scope_visibility() {
        let root = temp_root("rbac");
        // Installer and member cannot write or delete.
        let err = add_company_memory(
            &root,
            Role::Installer,
            "bob",
            "fact",
            "company",
            "Steel beam",
        )
        .unwrap_err();
        assert_eq!(err.0, 403);
        let err = add_company_memory(
            &root,
            Role::Member,
            "carol",
            "fact",
            "company",
            "Office hours",
        )
        .unwrap_err();
        assert_eq!(err.0, 403);

        // Authorized roles can write.
        let e_company = add_company_memory(
            &root,
            Role::Pm,
            "alice",
            "policy",
            "company",
            "All invoices are Net-30 unless otherwise stated.",
        )
        .unwrap();
        let e_role = add_company_memory(
            &root,
            Role::Office,
            "dana",
            "procedure",
            "role:installer",
            "Capture two site photos before sign-off.",
        )
        .unwrap();
        let e_client = add_company_memory(
            &root,
            Role::Ae,
            "erin",
            "preference",
            "client:acme",
            "Acme prefers PDF reports on Fridays.",
        )
        .unwrap();

        // File permissions on Unix must be 0600.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(company_memory_path(&root)).unwrap();
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }

        // Installer sees company + role:installer, NEVER client:acme.
        let inst_entries = list_company_memory_for(&root, Role::Installer, &[]);
        assert_eq!(inst_entries.len(), 2);
        assert!(inst_entries.iter().any(|e| e.id == e_company.id));
        assert!(inst_entries.iter().any(|e| e.id == e_role.id));
        assert!(!can_read_entry(&e_client, Role::Installer, &[]));
        assert!(!can_read_entry(&e_client, Role::Member, &[]));

        // Member sees ONLY company.
        let member_entries = list_company_memory_for(&root, Role::Member, &[]);
        assert_eq!(member_entries.len(), 1);
        assert_eq!(member_entries[0].id, e_company.id);

        // PM with allowed_client_ids=["other"] cannot see client:acme; with ["acme"] can.
        let pm_other = list_company_memory_for(&root, Role::Pm, &["other".to_string()]);
        assert!(!pm_other.iter().any(|e| e.id == e_client.id));
        let pm_acme = list_company_memory_for(&root, Role::Pm, &["acme".to_string()]);
        assert!(pm_acme.iter().any(|e| e.id == e_client.id));

        // Admin sees all entries regardless of role or client filter.
        let admin_entries = list_company_memory_for(&root, Role::Admin, &["other".to_string()]);
        assert_eq!(admin_entries.len(), 3);

        // Delete enforcement.
        assert_eq!(
            delete_company_memory(&root, Role::Member, &e_company.id)
                .unwrap_err()
                .0,
            403
        );
        assert!(delete_company_memory(&root, Role::Pm, &e_role.id).unwrap());
        assert!(!delete_company_memory(&root, Role::Pm, &e_role.id).unwrap());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn memory_rejects_secrets_host_paths_and_invalid_inputs() {
        let root = temp_root("validation");
        let bad_contents = [
            "OpenAI key sk-1234567890abcdef",
            "key=ergk_secret_token",
            "session=ergs_secret_token",
            "password=hunter2",
            "api_key=abc123",
            "secret=xyz",
            "Authorization: Bearer abc.def.ghi",
            "-----BEGIN RSA PRIVATE KEY-----",
            "Located at /home/alice/secrets.txt",
            "C:\\Users\\Alice\\Desktop\\notes.txt",
        ];
        for bad in bad_contents {
            let err = add_company_memory(&root, Role::Admin, "alice", "fact", "company", bad)
                .unwrap_err();
            assert_eq!(err.0, 400, "expected 400 for: {bad}");
            assert_eq!(err.1, "company memory cannot contain secrets or host paths");
        }

        // Benign English hyphenated words containing `sk-` must still succeed.
        let ok = add_company_memory(
            &root,
            Role::Admin,
            "alice",
            "fact",
            "company",
            "Follow task-level desk-check signoff before shipping.",
        )
        .unwrap();
        assert_eq!(ok.category, "fact");

        // Invalid category / scope / length / null bytes.
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "rumor", "company", "hi")
                .unwrap_err()
                .0,
            400
        );
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "fact", "role:wizard", "hi")
                .unwrap_err()
                .0,
            400
        );
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "fact", "client:../etc", "hi")
                .unwrap_err()
                .0,
            400
        );
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "fact", "company", "   ")
                .unwrap_err()
                .0,
            400
        );
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "fact", "company", "bad\0byte")
                .unwrap_err()
                .0,
            400
        );
        let too_long = "x".repeat(2001);
        assert_eq!(
            add_company_memory(&root, Role::Admin, "alice", "fact", "company", &too_long)
                .unwrap_err()
                .0,
            400
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prompt_rendering_caps_at_25_and_isolates_private_memory() {
        let root = temp_root("prompt");
        std::fs::write(root.join("memory.md"), "PRIVATE_SECRET_NOTE\n").unwrap();

        for i in 0..30 {
            add_company_memory(
                &root,
                Role::Admin,
                "alice",
                "fact",
                "company",
                &format!("Company rule #{i}\nsecond line"),
            )
            .unwrap();
        }
        add_company_memory(
            &root,
            Role::Admin,
            "alice",
            "preference",
            "client:acme",
            "Client confidential preference",
        )
        .unwrap();

        let rendered = render_company_memory_for_prompt(&root, Role::Member, &[]);
        assert!(rendered.contains("Shared Company Memory"));
        assert!(!rendered.contains("PRIVATE_SECRET_NOTE"));
        assert!(!rendered.contains("Client confidential preference"));
        assert!(!rendered.contains("Company rule #0"));
        assert!(rendered.contains("Company rule #29"));
        assert_eq!(
            rendered.lines().filter(|l| l.starts_with("- [")).count(),
            25
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn query_company_db_enforces_isolation_and_aggregates_outcomes() {
        let root = temp_root("query");
        let cfg = test_config(&root);
        std::fs::write(root.join("memory.md"), "TOP_SECRET_DEFAULT_MEMORY").unwrap();
        let alice_dir = root.join("people").join("alice");
        std::fs::create_dir_all(&alice_dir).unwrap();
        std::fs::write(alice_dir.join("memory.md"), "TOP_SECRET_ALICE_MEMORY").unwrap();

        let people = PeopleState {
            people: vec![
                Person {
                    id: "default".to_string(),
                    name: "Default Admin".to_string(),
                    role: "admin".to_string(),
                },
                Person {
                    id: "alice".to_string(),
                    name: "Alice PM".to_string(),
                    role: "pm".to_string(),
                },
                Person {
                    id: "bob".to_string(),
                    name: "Bob Installer".to_string(),
                    role: "installer".to_string(),
                },
            ],
            active: "default".to_string(),
            recovered: Vec::new(),
        };
        let library = Library::load(&root);

        // Work logs for default and alice.
        let mut alice_cfg = cfg.person_config(&people.people[1]);
        alice_cfg.data_dir = alice_dir.clone();
        crate::records::append_work(
            &cfg,
            "Admin audit task",
            "private summary",
            &["audit.md".to_string()],
            "t1",
            true,
        );
        crate::records::append_work(
            &alice_cfg,
            "Alice project plan",
            "private summary",
            &["plan.md".to_string()],
            "t2",
            true,
        );

        // Clients owned by alice vs bob.
        create_client_for_person(
            &root,
            Some("alice"),
            &json!({ "id": "acme", "name": "Acme Corp" }),
        )
        .unwrap();
        create_client_for_person(
            &root,
            Some("bob"),
            &json!({ "id": "globex", "name": "Globex Corp" }),
        )
        .unwrap();
        create_engagement(
            &root,
            "alice",
            &json!({ "id": "eng-1", "client_id": "acme", "person_id": "alice", "name": "Q4 Retainer" }),
        )
        .unwrap();

        // Outcomes JSONL.
        std::fs::create_dir_all(root.join("evolution")).unwrap();
        std::fs::write(
            root.join("evolution").join("outcomes.jsonl"),
            "{\"artifact_id\":\"daily-log\",\"quality_score\":0.8,\"prompt\":\"PRIVATE\"}\n\
             {\"artifact_id\":\"daily-log\",\"quality_score\":1.0}\n\
             not-json\n",
        )
        .unwrap();

        // Installer cannot query clients.
        let err = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "bob",
            Role::Installer,
            "clients",
            None,
            50,
        )
        .unwrap_err();
        assert_eq!(err.0, 403);

        // Alice (PM) sees only her owned client (acme), not bob's client (globex).
        let alice_clients = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "alice",
            Role::Pm,
            "clients",
            None,
            50,
        )
        .unwrap();
        assert_eq!(alice_clients["total"], 1);
        assert_eq!(alice_clients["rows"][0]["id"], "acme");

        // Admin sees both clients.
        let admin_clients = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "default",
            Role::Admin,
            "clients",
            None,
            50,
        )
        .unwrap();
        assert_eq!(admin_clients["total"], 2);

        // Non-admin work & receipts are isolated to principal_person_id.
        let alice_work = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "alice",
            Role::Pm,
            "work",
            None,
            50,
        )
        .unwrap();
        assert_eq!(alice_work["total"], 1);
        assert_eq!(alice_work["rows"][0]["person_id"], "alice");
        assert!(!alice_work.to_string().contains("TOP_SECRET"));

        let admin_work = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "default",
            Role::Admin,
            "work",
            None,
            50,
        )
        .unwrap();
        assert_eq!(admin_work["total"], 2);

        let alice_receipts = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "alice",
            Role::Pm,
            "receipts",
            None,
            50,
        )
        .unwrap();
        assert_eq!(alice_receipts["total"], 1);

        // Outcomes aggregation never exposes private prompts.
        let outcomes = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "bob",
            Role::Installer,
            "outcomes",
            None,
            50,
        )
        .unwrap();
        assert_eq!(outcomes["total"], 1);
        assert_eq!(outcomes["rows"][0]["artifact_id"], "daily-log");
        assert_eq!(outcomes["rows"][0]["count"], 2);
        assert_eq!(outcomes["rows"][0]["avg_quality_score"], 0.9);
        assert!(!outcomes.to_string().contains("PRIVATE"));

        // Tools + people + roles + summary.
        let tools = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "bob",
            Role::Installer,
            "tools",
            Some("daily"),
            50,
        )
        .unwrap();
        assert!(tools["total"].as_u64().unwrap() >= 1);

        let summary_member = query_company_db(
            &root,
            &cfg,
            &people,
            &library,
            "bob",
            Role::Member,
            "summary",
            None,
            50,
        )
        .unwrap();
        assert!(summary_member["counts"]["clients"].is_null());

        let _ = std::fs::remove_dir_all(&root);
    }
}
