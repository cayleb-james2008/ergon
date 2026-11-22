//! Standing Recurring Routines store (`routines.json`).
//!
//! Employees can save recurring autonomous workflows (e.g. weekly expense
//! reconciliation, daily inbox triage) per person and run them on demand via
//! `POST /api/routines/:id/run`. Persisted atomically via `.tmp` + rename.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::records;

static ROUTINE_SEQ: AtomicU64 = AtomicU64::new(0);
static ROUTINES_LOCK: Mutex<()> = Mutex::new(());

fn with_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = ROUTINES_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

pub fn routines_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("routines.json")
}

fn load_raw(path: &Path) -> Vec<Value> {
    if let Ok(text) = std::fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(arr) = v["routines"].as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

fn save_raw(path: &Path, items: &[Value]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir routines dir: {e}"))?;
    }
    let tmp = path.with_extension("json.tmp");
    let body = json!({ "routines": items }).to_string();
    std::fs::write(&tmp, body).map_err(|e| format!("write routines tmp: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename routines: {e}"))?;
    Ok(())
}

pub fn list_routines(person_data_dir: &Path) -> Vec<Value> {
    with_lock(|| load_raw(&routines_path(person_data_dir)))
}

pub fn get_routine(person_data_dir: &Path, id: &str) -> Option<Value> {
    with_lock(|| {
        load_raw(&routines_path(person_data_dir))
            .into_iter()
            .find(|r| r["id"].as_str() == Some(id))
    })
}

pub fn create_routine(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    skill_id: Option<&str>,
) -> Result<Value, String> {
    let clean_name: String = name
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    let clean_prompt: String = prompt.trim().chars().take(4000).collect();
    if clean_name.is_empty() {
        return Err("routine name is required".to_string());
    }
    if clean_prompt.is_empty() {
        return Err("routine prompt is required".to_string());
    }
    let clean_sched: String = schedule
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(64)
        .collect();
    let sched = if clean_sched.is_empty() {
        "manual".to_string()
    } else {
        clean_sched
    };
    let skill = skill_id
        .map(|s| s.trim().chars().take(64).collect::<String>())
        .filter(|s| !s.is_empty());

    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let ts = records::now_millis();
        let seq = ROUTINE_SEQ.fetch_add(1, Ordering::SeqCst);
        let id = format!("rtn-{ts}-{seq}");
        let item = json!({
            "id": id,
            "name": clean_name,
            "prompt": clean_prompt,
            "schedule": sched,
            "skill_id": skill,
            "enabled": true,
            "created_ms": ts,
            "last_run_ms": Value::Null,
            "last_thread_id": Value::Null,
            "last_status": Value::Null,
            "run_count": 0,
        });
        items.push(item.clone());
        save_raw(&path, &items)?;
        Ok(item)
    })
}

pub fn delete_routine(person_data_dir: &Path, id: &str) -> Result<bool, String> {
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let before = items.len();
        items.retain(|r| r["id"].as_str() != Some(id));
        if items.len() == before {
            return Ok(false);
        }
        save_raw(&path, &items)?;
        Ok(true)
    })
}

/// Update mutable fields of an existing routine (`enabled`, `schedule`, `name`,
/// `prompt`, `skill_id`) atomically. Returns `Ok(Some(updated))` when found or
/// `Ok(None)` when no routine has `id`.
pub fn update_routine(
    person_data_dir: &Path,
    id: &str,
    enabled: Option<bool>,
    schedule: Option<&str>,
    name: Option<&str>,
    prompt: Option<&str>,
    skill_id: Option<Option<&str>>,
) -> Result<Option<Value>, String> {
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let Some(idx) = items.iter().position(|r| r["id"].as_str() == Some(id)) else {
            return Ok(None);
        };
        if let Some(en) = enabled {
            items[idx]["enabled"] = json!(en);
        }
        if let Some(raw_sched) = schedule {
            let clean: String = raw_sched
                .replace(['\r', '\n'], " ")
                .trim()
                .chars()
                .take(64)
                .collect();
            items[idx]["schedule"] = json!(if clean.is_empty() { "manual" } else { &clean });
        }
        if let Some(raw_name) = name {
            let clean: String = raw_name
                .replace(['\r', '\n'], " ")
                .trim()
                .chars()
                .take(120)
                .collect();
            if clean.is_empty() {
                return Err("routine name cannot be empty".to_string());
            }
            items[idx]["name"] = json!(clean);
        }
        if let Some(raw_prompt) = prompt {
            let clean: String = raw_prompt.trim().chars().take(4000).collect();
            if clean.is_empty() {
                return Err("routine prompt cannot be empty".to_string());
            }
            items[idx]["prompt"] = json!(clean);
        }
        if let Some(sk_opt) = skill_id {
            let sk = sk_opt
                .map(|s| s.trim().chars().take(64).collect::<String>())
                .filter(|s| !s.is_empty());
            items[idx]["skill_id"] = json!(sk);
        }
        items[idx]["updated_ms"] = json!(records::now_millis());
        let updated = items[idx].clone();
        save_raw(&path, &items)?;
        Ok(Some(updated))
    })
}

pub fn record_run(
    person_data_dir: &Path,
    id: &str,
    thread_id: &str,
    status: &str,
) -> Option<Value> {
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let idx = items.iter().position(|r| r["id"].as_str() == Some(id))?;
        let count = items[idx]["run_count"].as_u64().unwrap_or(0) + 1;
        items[idx]["last_run_ms"] = json!(records::now_millis());
        items[idx]["last_thread_id"] = json!(thread_id);
        items[idx]["last_status"] = json!(status);
        items[idx]["run_count"] = json!(count);
        let updated = items[idx].clone();
        let _ = save_raw(&path, &items);
        Some(updated)
    })
}
