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
    let body = json!({ "routines": items }).to_string();
    crate::atomic_write_bytes(path, body.as_bytes()).map_err(|e| format!("write routines: {e}"))
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

#[derive(Clone, Debug, Default)]
pub struct PlaybookOptions {
    pub steps: Vec<String>,
    pub expected_inputs: Vec<String>,
    pub pack_deliverables: bool,
}

pub fn create_routine(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    skill_id: Option<&str>,
) -> Result<Value, String> {
    create_routine_with_options(person_data_dir, name, prompt, schedule, None, skill_id, true)
}

pub fn create_routine_with_options(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    interval_secs: Option<u64>,
    skill_id: Option<&str>,
    enabled: bool,
) -> Result<Value, String> {
    create_playbook_routine(
        person_data_dir,
        name,
        prompt,
        schedule,
        interval_secs,
        skill_id,
        enabled,
        &PlaybookOptions::default(),
    )
}

/// Create a recurring routine or multi-step chained Workspace Playbook (`steps`,
/// optional `expected_inputs`, and optional `pack_deliverables`).
#[allow(clippy::too_many_arguments)]
pub fn create_playbook_routine(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    interval_secs: Option<u64>,
    skill_id: Option<&str>,
    enabled: bool,
    playbook: &PlaybookOptions,
) -> Result<Value, String> {
    let clean_name: String = name
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    let clean_steps: Vec<String> = playbook
        .steps
        .iter()
        .map(|s| s.trim().chars().take(2000).collect::<String>())
        .filter(|s| !s.is_empty())
        .take(8)
        .collect();
    let clean_inputs: Vec<String> = playbook
        .expected_inputs
        .iter()
        .map(|s| s.replace(['\r', '\n'], " ").trim().chars().take(64).collect::<String>())
        .filter(|s| !s.is_empty())
        .take(8)
        .collect();
    let clean_prompt: String = {
        let raw = prompt.trim();
        if raw.is_empty() && !clean_steps.is_empty() {
            clean_steps.join(" -> ")
        } else {
            raw.to_string()
        }
    }
    .chars()
    .take(4000)
    .collect();
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
    let (sched, resolved_interval) = if let Some(iv) = interval_secs {
        let cadence = crate::scheduler::parse_cadence_or_interval(
            if clean_sched.is_empty() { None } else { Some(clean_sched.as_str()) },
            Some(iv),
        )?;
        (cadence.canonical_label(), cadence.explicit_interval_secs())
    } else if clean_sched.is_empty() {
        ("manual".to_string(), None)
    } else if let Ok(cadence) = crate::scheduler::parse_cadence(&clean_sched) {
        (cadence.canonical_label(), cadence.explicit_interval_secs())
    } else {
        (clean_sched, None)
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
        let mut item = json!({
            "id": id,
            "name": clean_name,
            "prompt": clean_prompt,
            "schedule": sched,
            "skill_id": skill,
            "enabled": enabled,
            "created_ms": ts,
            "last_run_ms": Value::Null,
            "last_thread_id": Value::Null,
            "last_status": Value::Null,
            "run_count": 0,
        });
        if let Some(iv) = resolved_interval {
            item["interval_secs"] = json!(iv);
        }
        if !clean_steps.is_empty() {
            item["steps"] = json!(clean_steps);
        }
        if !clean_inputs.is_empty() {
            item["expected_inputs"] = json!(clean_inputs);
        }
        if playbook.pack_deliverables {
            item["pack_deliverables"] = json!(true);
        }
        items.push(item.clone());
        save_raw(&path, &items)?;
        Ok(item)
    })
}

/// Patch an existing routine using `scheduler::apply_routine_patch` (validates
/// cadence/interval_secs, supports pause/resume via `enabled`, atomic write).
pub fn patch_routine(
    person_data_dir: &Path,
    id: &str,
    patch: &Value,
) -> Result<Option<Value>, String> {
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let Some(idx) = items.iter().position(|r| r["id"].as_str() == Some(id)) else {
            return Ok(None);
        };
        crate::scheduler::apply_routine_patch(&mut items[idx], patch)?;
        items[idx]["updated_ms"] = json!(records::now_millis());
        let updated = items[idx].clone();
        save_raw(&path, &items)?;
        Ok(Some(updated))
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
