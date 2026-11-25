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

pub const PLAYBOOK_PACK_FORMAT: &str = "ergon-playbook-v1";
pub const MAX_PLAYBOOK_PACK_BYTES: usize = 256 * 1024;

fn clean_playbook_list(arr: Option<&Vec<Value>>, max_chars: usize) -> Vec<String> {
    arr.map(|items| {
        items
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().chars().take(max_chars).collect::<String>())
            .filter(|s| !s.is_empty())
            .take(8)
            .collect()
    })
    .unwrap_or_default()
}

pub fn playbook_canonical_sha256(
    name: &str,
    prompt: &str,
    schedule: &str,
    interval_secs: Option<u64>,
    skill_id: Option<&str>,
    playbook: &PlaybookOptions,
) -> String {
    let canonical = json!([
        PLAYBOOK_PACK_FORMAT,
        name,
        prompt,
        schedule,
        interval_secs,
        skill_id,
        playbook.steps,
        playbook.expected_inputs,
        playbook.pack_deliverables,
    ])
    .to_string();
    crate::receipts::sha256_hex(canonical.as_bytes())
}

fn safe_playbook_filename(name: &str) -> String {
    let slug: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let stem = if trimmed.is_empty() {
        "playbook"
    } else {
        &trimmed[..trimmed.len().min(48)]
    };
    format!("{stem}.ergon-playbook.json")
}

/// Export a saved routine/playbook as a signed `.ergon-playbook.json` pack with SHA-256 digest.
pub fn export_playbook_pack(person_data_dir: &Path, id: &str) -> Option<(Value, String, String)> {
    let rt = get_routine(person_data_dir, id)?;
    let name: String = rt["name"]
        .as_str()
        .unwrap_or("")
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    let prompt: String = rt["prompt"]
        .as_str()
        .unwrap_or("")
        .trim()
        .chars()
        .take(4000)
        .collect();
    let schedule: String = rt["schedule"]
        .as_str()
        .unwrap_or("manual")
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(64)
        .collect();
    let schedule = if schedule.is_empty() {
        "manual".to_string()
    } else {
        schedule
    };
    let interval_secs = rt["interval_secs"].as_u64();
    let skill_id = rt["skill_id"]
        .as_str()
        .map(|s| s.trim().chars().take(64).collect::<String>())
        .filter(|s| !s.is_empty());
    let opts = PlaybookOptions {
        steps: clean_playbook_list(rt["steps"].as_array(), 2000),
        expected_inputs: clean_playbook_list(rt["expected_inputs"].as_array(), 64),
        pack_deliverables: rt["pack_deliverables"].as_bool().unwrap_or(false),
    };
    let digest = playbook_canonical_sha256(
        &name,
        &prompt,
        &schedule,
        interval_secs,
        skill_id.as_deref(),
        &opts,
    );
    let filename = safe_playbook_filename(&name);
    let pack = json!({
        "format": PLAYBOOK_PACK_FORMAT,
        "version": env!("CARGO_PKG_VERSION"),
        "exported_ms": records::now_millis(),
        "playbook": {
            "id": id,
            "name": name,
            "prompt": prompt,
            "schedule": schedule,
            "interval_secs": interval_secs,
            "skill_id": skill_id,
            "steps": opts.steps,
            "expected_inputs": opts.expected_inputs,
            "pack_deliverables": opts.pack_deliverables,
        },
        "sha256": digest,
    });
    Some((pack, digest, filename))
}

fn validate_string_array(
    obj: &serde_json::Map<String, Value>,
    key: &str,
    max_items: usize,
    max_chars: usize,
) -> Result<Vec<String>, (u16, String)> {
    let Some(val) = obj.get(key) else {
        return Ok(Vec::new());
    };
    if val.is_null() {
        return Ok(Vec::new());
    }
    let Some(arr) = val.as_array() else {
        return Err((400, format!("playbook.{key} must be an array of strings")));
    };
    if arr.len() > max_items {
        return Err((
            400,
            format!("playbook.{key} exceeds maximum of {max_items} entries"),
        ));
    }
    let mut out = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        let Some(s) = item.as_str() else {
            return Err((
                400,
                format!("playbook.{key}[{idx}] must be a non-empty string"),
            ));
        };
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.chars().count() > max_chars {
            return Err((
                400,
                format!("playbook.{key}[{idx}] must be 1..={max_chars} characters"),
            ));
        }
        out.push(s.to_string());
    }
    Ok(out)
}

/// Strictly validate, verify SHA-256 checksum, and import a `.ergon-playbook.json` pack (<= 256 KB).
pub fn import_playbook_pack(
    person_data_dir: &Path,
    raw_bytes: &[u8],
) -> Result<(Value, String), (u16, String)> {
    if raw_bytes.len() > MAX_PLAYBOOK_PACK_BYTES {
        return Err((
            413,
            format!("playbook pack exceeds maximum size of {MAX_PLAYBOOK_PACK_BYTES} bytes"),
        ));
    }
    let parsed: Value = serde_json::from_slice(raw_bytes)
        .map_err(|e| (400, format!("invalid playbook JSON: {e}")))?;
    let Some(top) = parsed.as_object() else {
        return Err((400, "playbook pack must be a JSON object".to_string()));
    };
    for key in top.keys() {
        if !matches!(
            key.as_str(),
            "format" | "version" | "exported_ms" | "playbook" | "sha256"
        ) {
            return Err((400, format!("unexpected top-level field '{key}'")));
        }
    }
    if top.get("format").and_then(|v| v.as_str()) != Some(PLAYBOOK_PACK_FORMAT) {
        return Err((
            400,
            format!("playbook.format must be '{PLAYBOOK_PACK_FORMAT}'"),
        ));
    }
    if let Some(ver) = top.get("version").filter(|v| !v.is_null()) {
        let Some(s) = ver.as_str() else {
            return Err((400, "version must be a string".to_string()));
        };
        if s.len() > 32 {
            return Err((400, "version exceeds 32 characters".to_string()));
        }
    }
    if let Some(exp) = top.get("exported_ms").filter(|v| !v.is_null()) {
        if exp.as_u64().is_none() {
            return Err((400, "exported_ms must be an unsigned integer".to_string()));
        }
    }
    let claimed_sha = top
        .get("sha256")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if claimed_sha.len() != 64 || !claimed_sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err((
            400,
            "sha256 checksum must be a 64-character hex digest".to_string(),
        ));
    }
    let Some(pb) = top.get("playbook").and_then(|v| v.as_object()) else {
        return Err((400, "playbook object is required".to_string()));
    };
    for key in pb.keys() {
        if !matches!(
            key.as_str(),
            "id"
                | "name"
                | "prompt"
                | "schedule"
                | "interval_secs"
                | "skill_id"
                | "steps"
                | "expected_inputs"
                | "pack_deliverables"
        ) {
            return Err((400, format!("unexpected playbook field '{key}'")));
        }
    }
    if let Some(id_val) = pb.get("id").filter(|v| !v.is_null()) {
        let Some(s) = id_val.as_str() else {
            return Err((400, "playbook.id must be a string".to_string()));
        };
        if s.len() > 128 {
            return Err((400, "playbook.id exceeds 128 characters".to_string()));
        }
    }
    let Some(name) = pb.get("name").and_then(|v| v.as_str()) else {
        return Err((400, "playbook.name is required".to_string()));
    };
    if name.trim().is_empty()
        || name.contains(['\r', '\n'])
        || name.chars().count() > 120
    {
        return Err((
            400,
            "playbook.name must be 1..=120 non-control characters".to_string(),
        ));
    }
    let Some(prompt) = pb.get("prompt").and_then(|v| v.as_str()) else {
        return Err((400, "playbook.prompt is required".to_string()));
    };
    if prompt.trim().is_empty() || prompt.chars().count() > 4000 {
        return Err((
            400,
            "playbook.prompt must be 1..=4000 characters".to_string(),
        ));
    }
    let Some(schedule) = pb.get("schedule").and_then(|v| v.as_str()) else {
        return Err((400, "playbook.schedule is required".to_string()));
    };
    if schedule.trim().is_empty()
        || schedule.contains(['\r', '\n'])
        || schedule.chars().count() > 64
    {
        return Err((
            400,
            "playbook.schedule must be 1..=64 single-line characters".to_string(),
        ));
    }
    let interval_secs = match pb.get("interval_secs") {
        None | Some(Value::Null) => None,
        Some(v) => match v.as_u64() {
            Some(iv) => Some(iv),
            None => {
                return Err((
                    400,
                    "playbook.interval_secs must be a positive integer or null".to_string(),
                ))
            }
        },
    };
    let cadence = crate::scheduler::parse_cadence_or_interval(Some(schedule), interval_secs)
        .map_err(|e| (400, format!("invalid playbook schedule: {e}")))?;
    let skill_id = match pb.get("skill_id") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let Some(s) = v.as_str() else {
                return Err((400, "playbook.skill_id must be a string or null".to_string()));
            };
            if s.trim().is_empty() || s.chars().any(|c| c.is_control()) || s.chars().count() > 64 {
                return Err((
                    400,
                    "playbook.skill_id must be 1..=64 non-control characters".to_string(),
                ));
            }
            Some(s)
        }
    };
    let steps = validate_string_array(pb, "steps", 8, 2000)?;
    let expected_inputs = validate_string_array(pb, "expected_inputs", 8, 64)?;
    let pack_deliverables = match pb.get("pack_deliverables") {
        None | Some(Value::Null) => false,
        Some(v) => v.as_bool().ok_or_else(|| {
            (
                400,
                "playbook.pack_deliverables must be a boolean".to_string(),
            )
        })?,
    };
    let opts = PlaybookOptions {
        steps,
        expected_inputs,
        pack_deliverables,
    };
    let expected_sha = playbook_canonical_sha256(
        name,
        prompt,
        schedule,
        interval_secs,
        skill_id,
        &opts,
    );
    if !expected_sha.eq_ignore_ascii_case(claimed_sha) {
        return Err((
            400,
            "sha256 checksum mismatch: playbook pack failed integrity verification".to_string(),
        ));
    }
    let routine = create_playbook_routine(
        person_data_dir,
        name,
        prompt,
        &cadence.canonical_label(),
        cadence.explicit_interval_secs(),
        skill_id,
        true,
        &opts,
    )
    .map_err(|e| (400, e))?;
    Ok((routine, expected_sha))
}
