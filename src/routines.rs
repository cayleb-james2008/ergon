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
    create_pipeline_routine(
        person_data_dir,
        name,
        prompt,
        schedule,
        interval_secs,
        skill_id,
        enabled,
        playbook,
        &[],
    )
}

/// Create a routine/playbook with optional typed `steps_v2` alongside legacy
/// `steps` for full backward compatibility.
#[allow(clippy::too_many_arguments)]
pub fn create_pipeline_routine(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    interval_secs: Option<u64>,
    skill_id: Option<&str>,
    enabled: bool,
    playbook: &PlaybookOptions,
    steps_v2: &[crate::pipelines::PipelineStepSpec],
) -> Result<Value, String> {
    create_pipeline_routine_with_tz(
        person_data_dir,
        name,
        prompt,
        schedule,
        interval_secs,
        None,
        None,
        skill_id,
        enabled,
        playbook,
        steps_v2,
    )
}

fn looks_like_cron_schedule(spec: &str) -> bool {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return false;
    }
    let low = trimmed.to_ascii_lowercase();
    if low == "cron" || low.starts_with("cron ") || low.starts_with("cron:") {
        return true;
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() >= 5 {
        return true;
    }
    if trimmed.contains('*') || trimmed.contains('/') || trimmed.contains(',') {
        return true;
    }
    tokens.len() >= 2 && tokens[0].bytes().next().is_some_and(|b| b.is_ascii_digit())
}

/// Create a routine/playbook with optional typed `steps_v2` and optional cron
/// timezone offset (`tz_offset_mins` or `tz`).
#[allow(clippy::too_many_arguments)]
pub fn create_pipeline_routine_with_tz(
    person_data_dir: &Path,
    name: &str,
    prompt: &str,
    schedule: &str,
    interval_secs: Option<u64>,
    tz_offset_mins: Option<i32>,
    tz: Option<&str>,
    skill_id: Option<&str>,
    enabled: bool,
    playbook: &PlaybookOptions,
    steps_v2: &[crate::pipelines::PipelineStepSpec],
) -> Result<Value, String> {
    let clean_name: String = name
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect();
    let mut clean_steps: Vec<String> = playbook
        .steps
        .iter()
        .map(|s| s.trim().chars().take(2000).collect::<String>())
        .filter(|s| !s.is_empty())
        .take(8)
        .collect();
    if clean_steps.is_empty() && !steps_v2.is_empty() {
        clean_steps = steps_v2
            .iter()
            .take(8)
            .map(|s| s.label.clone())
            .collect();
    }
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
        .take(128)
        .collect();
    let explicit_tz = if let Some(m) = tz_offset_mins {
        if !(-720..=840).contains(&m) {
            return Err("tz_offset_mins must be between -720 and 840".to_string());
        }
        Some(m)
    } else if let Some(t) = tz.map(str::trim).filter(|s| !s.is_empty()) {
        Some(crate::scheduler::parse_tz_offset_mins(t)?)
    } else {
        None
    };
    let (sched, resolved_interval, resolved_tz) = if let Some(iv) = interval_secs {
        let cadence = crate::scheduler::parse_cadence_or_interval_with_tz(
            if clean_sched.is_empty() { None } else { Some(clean_sched.as_str()) },
            Some(iv),
            explicit_tz,
        )?;
        (cadence.canonical_label(), cadence.explicit_interval_secs(), None)
    } else if clean_sched.is_empty() {
        ("manual".to_string(), None, None)
    } else {
        match crate::scheduler::parse_cadence_with_tz(&clean_sched, explicit_tz) {
            Ok(crate::scheduler::Cadence::Cron {
                expr,
                tz_offset_mins,
            }) => (expr, None, Some(tz_offset_mins)),
            Ok(other) => (other.canonical_label(), other.explicit_interval_secs(), None),
            Err(err) => {
                if explicit_tz.is_some() || looks_like_cron_schedule(&clean_sched) {
                    return Err(err);
                }
                (clean_sched, None, None)
            }
        }
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
        if let Some(tz_mins) = resolved_tz {
            item["tz_offset_mins"] = json!(tz_mins);
            let tz_label = tz
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| crate::scheduler::format_tz_offset(tz_mins));
            item["tz"] = json!(tz_label);
        }
        if !clean_steps.is_empty() {
            item["steps"] = json!(clean_steps);
        }
        if !steps_v2.is_empty() {
            item["steps_v2"] = json!(steps_v2.iter().map(|s| s.to_json()).collect::<Vec<_>>());
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

/// Update `steps_v2` on an existing routine atomically.
pub fn set_routine_steps_v2(
    person_data_dir: &Path,
    id: &str,
    steps_v2: &[crate::pipelines::PipelineStepSpec],
) -> Result<Option<Value>, String> {
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let Some(idx) = items.iter().position(|r| r["id"].as_str() == Some(id)) else {
            return Ok(None);
        };
        if steps_v2.is_empty() {
            if let Some(obj) = items[idx].as_object_mut() {
                obj.remove("steps_v2");
            }
        } else {
            items[idx]["steps_v2"] = json!(steps_v2.iter().map(|s| s.to_json()).collect::<Vec<_>>());
        }
        items[idx]["updated_ms"] = json!(records::now_millis());
        let updated = items[idx].clone();
        save_raw(&path, &items)?;
        Ok(Some(updated))
    })
}

/// Patch an existing routine using `scheduler::apply_routine_patch` (validates
/// cadence/interval_secs, supports pause/resume via `enabled`, atomic write).
pub fn patch_routine(
    person_data_dir: &Path,
    id: &str,
    patch: &Value,
) -> Result<Option<Value>, String> {
    let parsed_steps_v2 = match patch.get("steps_v2") {
        None => None,
        Some(Value::Null) => Some(Vec::new()),
        Some(v) if v.as_array().is_some_and(|a| a.is_empty()) => Some(Vec::new()),
        Some(v) => Some(crate::pipelines::parse_steps_v2(v)?),
    };
    with_lock(|| {
        let path = routines_path(person_data_dir);
        let mut items = load_raw(&path);
        let Some(idx) = items.iter().position(|r| r["id"].as_str() == Some(id)) else {
            return Ok(None);
        };
        crate::scheduler::apply_routine_patch(&mut items[idx], patch)?;
        if let Some(ref steps_v2) = parsed_steps_v2 {
            if steps_v2.is_empty() {
                if let Some(obj) = items[idx].as_object_mut() {
                    obj.remove("steps_v2");
                }
            } else {
                items[idx]["steps_v2"] = json!(steps_v2.iter().map(|s| s.to_json()).collect::<Vec<_>>());
            }
        }
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

fn infer_step_skill_and_mode(
    step_text: &str,
    fallback_skill: Option<&str>,
    is_first: bool,
) -> (Option<String>, &'static str, Option<&'static str>) {
    let lower = step_text.to_ascii_lowercase();
    let explicit_skills = [
        "reconcile_ledger",
        "clean_dataset",
        "audit_contract",
        "summarize_meeting",
        "triage_inbox",
        "compare_documents",
        "draft_brief",
        "extract_tables",
        "make_spreadsheet",
        "write_document",
        "update_memory",
        "research",
    ];
    let matched = explicit_skills
        .iter()
        .find(|&&id| lower.contains(id))
        .map(|s| (*s).to_string())
        .or_else(|| {
            if lower.contains("ledger") || lower.contains("reconcil") || lower.contains("expense") {
                Some("reconcile_ledger".to_string())
            } else if lower.contains("clean") || lower.contains("dataset") || lower.contains("normaliz") {
                Some("clean_dataset".to_string())
            } else if lower.contains("contract") || lower.contains("clause") {
                Some("audit_contract".to_string())
            } else if lower.contains("meeting") || lower.contains("action item") {
                Some("summarize_meeting".to_string())
            } else if lower.contains("inbox") || lower.contains("triage") || lower.contains("email") {
                Some("triage_inbox".to_string())
            } else if lower.contains("compare") || lower.contains("diff") {
                Some("compare_documents".to_string())
            } else if lower.contains("spreadsheet") || lower.contains(".xlsx") {
                Some("make_spreadsheet".to_string())
            } else if lower.contains("docx") || lower.contains("word") || lower.contains("memo") || lower.contains("write") {
                Some("write_document".to_string())
            } else if lower.contains("table") || lower.contains("extract") {
                Some("extract_tables".to_string())
            } else if lower.contains("brief") || lower.contains("summary") {
                Some("draft_brief".to_string())
            } else if lower.contains("memory") {
                Some("update_memory".to_string())
            } else if lower.contains("research") {
                Some("research".to_string())
            } else if is_first {
                fallback_skill.map(str::to_string)
            } else {
                None
            }
        });

    let writes = lower.contains("export")
        || lower.contains("write")
        || lower.contains("save")
        || lower.contains("create")
        || lower.contains("clean")
        || lower.contains(".docx")
        || lower.contains(".xlsx")
        || matches!(
            matched.as_deref(),
            Some(
                "clean_dataset"
                    | "reconcile_ledger"
                    | "make_spreadsheet"
                    | "write_document"
                    | "draft_brief"
                    | "update_memory"
            )
        );

    let output_hint = match matched.as_deref() {
        Some("clean_dataset") => Some("Cleaned dataset (.csv)"),
        Some("reconcile_ledger") => Some("Expense reconciliation report (.md)"),
        Some("make_spreadsheet") => Some("Typed workbook (.xlsx)"),
        Some("write_document") => Some("Formatted deliverable (.docx / .md)"),
        Some("draft_brief") => Some("Executive brief (.md)"),
        Some("audit_contract") => {
            if writes {
                Some("Contract audit findings (.md)")
            } else {
                None
            }
        }
        Some("summarize_meeting") => Some("Meeting digest & follow-up (.md)"),
        Some("update_memory") => Some("Standing memory rules (memory.md)"),
        _ if writes => Some("Workspace deliverable artifact"),
        _ => None,
    };

    (
        matched,
        if writes { "read_and_write" } else { "read_only" },
        output_hint,
    )
}

fn extract_step_paths(
    step_text: &str,
    workspace_paths: &[String],
    expected_inputs: &[String],
    include_expected: bool,
) -> Vec<String> {
    const EXTS: [&str; 12] = [
        ".csv", ".tsv", ".md", ".markdown", ".txt", ".json", ".xlsx", ".docx", ".pdf", ".pptx",
        ".yaml", ".log",
    ];
    let mut found: Vec<String> = Vec::new();
    let mut push_unique = |cand: &str| {
        let clean = cand
            .trim_matches(|c: char| {
                matches!(
                    c,
                    '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
                )
            })
            .trim_end_matches('.');
        let clean = clean
            .split_once(":L")
            .map(|(p, _)| p)
            .unwrap_or(clean);
        if clean.is_empty() || clean.starts_with('/') || clean.contains("..") {
            return;
        }
        if !found.iter().any(|x| x == clean) {
            found.push(clean.to_string());
        }
    };

    if include_expected {
        for inp in expected_inputs {
            push_unique(inp);
        }
    }
    for tok in step_text.split_whitespace() {
        let low = tok.to_ascii_lowercase();
        if EXTS.iter().any(|ext| {
            low.ends_with(ext)
                || low.contains(&format!("{ext}:l"))
                || low.contains(&format!("{ext},"))
                || low.contains(&format!("{ext})"))
        }) {
            push_unique(tok);
        }
    }
    let lower_step = step_text.to_ascii_lowercase();
    for ws_path in workspace_paths {
        let base = ws_path.rsplit('/').next().unwrap_or(ws_path);
        if (lower_step.contains(&ws_path.to_ascii_lowercase())
            || lower_step.contains(&base.to_ascii_lowercase()))
            && !found.iter().any(|x| x == ws_path)
        {
            found.push(ws_path.clone());
        }
    }
    found
}

/// Deterministic, zero-side-effect dry-run preview of a saved routine/playbook:
/// expands ordered steps, matches workspace inputs & predicted output paths,
/// and reports `.history/` rollback readiness before execution.
pub fn preview_playbook_routine(
    person_data_dir: &Path,
    workspace_dir: &Path,
    workspace_paths: &[String],
    id: &str,
) -> Option<Value> {
    let rt = get_routine(person_data_dir, id)?;
    let name = rt["name"].as_str().unwrap_or("Playbook").trim();
    let prompt = rt["prompt"].as_str().unwrap_or("").trim();
    let schedule = rt["schedule"].as_str().unwrap_or("manual").trim();
    let fallback_skill = rt["skill_id"].as_str().map(str::trim).filter(|s| !s.is_empty());
    let pack_deliverables = rt["pack_deliverables"].as_bool().unwrap_or(false);
    let expected_inputs = clean_playbook_list(rt["expected_inputs"].as_array(), 64);
    let mut raw_steps = clean_playbook_list(rt["steps"].as_array(), 2000);
    if raw_steps.is_empty() && !prompt.is_empty() {
        raw_steps.push(prompt.to_string());
    }

    let typed_steps = rt
        .get("steps_v2")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
        .and_then(|v| crate::pipelines::parse_steps_v2(v).ok());

    let mut steps_out = Vec::new();
    let mut impact_order: Vec<(String, &'static str, Option<String>)> = Vec::new();
    let mut push_impact = |path: String, role: &'static str, step_label: Option<String>| {
        if let Some(existing) = impact_order.iter_mut().find(|x| x.0 == path) {
            if role == "read_and_update" || role == "deliverable_output" {
                existing.1 = role;
            }
            return;
        }
        impact_order.push((path, role, step_label));
    };

    if let Some(ref v2) = typed_steps {
        let total = v2.len();
        for (idx, st) in v2.iter().enumerate() {
            let mode = match st.kind.as_str() {
                "checkpoint" => "human_checkpoint",
                "export" | "skill" => "read_and_write",
                "subpipeline" => "subpipeline",
                _ => "agent",
            };
            let step_text = st
                .prompt
                .clone()
                .or_else(|| st.checkpoint.as_ref().map(|c| c.prompt.clone()))
                .unwrap_or_else(|| st.label.clone());
            let mut refs = extract_step_paths(&step_text, workspace_paths, &expected_inputs, idx == 0);
            if let Some(obj) = st.args.as_object() {
                for v in obj.values() {
                    if let Some(s) = v.as_str() {
                        if !s.contains("{{") && !s.trim().is_empty() && s.contains('.') && !refs.iter().any(|x| x == s) {
                            refs.push(s.to_string());
                        }
                    }
                }
            }
            let mut predicted_outputs: Vec<String> = Vec::new();
            if st.kind == "export" {
                if let Some(ex) = &st.export {
                    predicted_outputs.push(format!("Exported {} (.{})", ex.path, ex.format));
                }
            } else if let Some(sk) = &st.skill_id {
                predicted_outputs.push(format!("Deterministic skill: {sk}"));
            }
            if pack_deliverables && idx + 1 == total {
                predicted_outputs.push("Playbook Deliverable Pack.md".to_string());
            }
            for r in &refs {
                push_impact(r.clone(), "input", Some(format!("Step {}", idx + 1)));
            }
            steps_out.push(json!({
                "step": idx + 1,
                "id": st.id,
                "kind": st.kind,
                "label": st.label,
                "prompt": step_text,
                "skill_id": st.skill_id,
                "mode": mode,
                "referenced_paths": refs,
                "predicted_outputs": predicted_outputs,
            }));
        }
    } else {
        for (idx, step_text) in raw_steps.iter().enumerate() {
            let (matched_skill, mode, output_hint) =
                infer_step_skill_and_mode(step_text, fallback_skill, idx == 0);
            let refs = extract_step_paths(step_text, workspace_paths, &expected_inputs, idx == 0);
            let mut predicted_outputs: Vec<String> = Vec::new();
            if let Some(hint) = output_hint {
                predicted_outputs.push(hint.to_string());
            }
            if pack_deliverables && idx + 1 == raw_steps.len() {
                predicted_outputs.push("Playbook Deliverable Pack.md".to_string());
            }
            for r in &refs {
                let role = if mode == "read_and_write" && r.ends_with(".csv") && matched_skill.as_deref() == Some("clean_dataset") {
                    "read_and_update"
                } else {
                    "input"
                };
                push_impact(r.clone(), role, Some(format!("Step {}", idx + 1)));
            }
            steps_out.push(json!({
                "step": idx + 1,
                "prompt": step_text,
                "skill_id": matched_skill,
                "mode": mode,
                "referenced_paths": refs,
                "predicted_outputs": predicted_outputs,
            }));
        }
    }

    if pack_deliverables {
        push_impact(
            "Playbook Deliverable Pack.md".to_string(),
            "deliverable_pack",
            Some("Final step".to_string()),
        );
    }

    // If no concrete file paths were referenced, surface up to 3 relevant workspace files as candidate context.
    if impact_order.is_empty() {
        for ws in workspace_paths.iter().take(3) {
            impact_order.push((ws.clone(), "workspace_context", Some("Context".to_string())));
        }
    }

    let mut existing_snapshots = crate::file_history::list_tracked_files(person_data_dir).len();
    let mut file_impacts = Vec::with_capacity(impact_order.len());
    for (path, role, step_label) in &impact_order {
        let exists = workspace_paths.iter().any(|p| p == path)
            || crate::tools::sandbox_path(workspace_dir, path)
                .map(|p| p.is_file())
                .unwrap_or(false);
        let revs = crate::file_history::revision_count(person_data_dir, workspace_dir, path);
        if revs > 0 {
            existing_snapshots = existing_snapshots.max(revs);
        }
        file_impacts.push(json!({
            "path": path,
            "role": role,
            "step": step_label,
            "exists_in_workspace": exists,
            "history_revisions": revs,
            "reversible": true,
        }));
    }

    let matched_ws: Vec<&String> = workspace_paths
        .iter()
        .filter(|p| impact_order.iter().any(|(imp, _, _)| imp == *p))
        .collect();

    Some(json!({
        "routine_id": id,
        "name": name,
        "schedule": schedule,
        "dry_run": true,
        "zero_side_effects": true,
        "step_count": steps_out.len(),
        "steps": steps_out,
        "expected_inputs": expected_inputs,
        "matched_workspace_files": matched_ws,
        "file_impacts": file_impacts,
        "pack_deliverables": pack_deliverables,
        "rollback": {
            "safe": true,
            "badge": "Reversible via .history/",
            "history_snapshots_enabled": true,
            "existing_snapshots": existing_snapshots,
            "reversible_paths": file_impacts.len(),
            "note": "Dry-run preview inspects your workspace read-only. Every existing file modified during execution is snapshotted into .history/ first for 1-click restore.",
        },
        "generated_ms": records::now_millis(),
    }))
}
