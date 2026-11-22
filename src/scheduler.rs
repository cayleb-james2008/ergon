//! Cadence parser, next-run scheduler calculator, and atomic routine run history.
//!
//! Supports human-readable recurring schedules (`every 15m`, `every 30s`,
//! `hourly`, `daily 09:00`, `weekly mon`, or explicit `interval_secs`),
//! computes deterministic UTC `next_run_ms` timestamps with zero external
//! chrono dependencies, tracks pause/resume state (`enabled: bool`), and
//! persists `RoutineRunRecord` history atomically under `<person_dir>/routine_history.json`.

use crate::{backup_corrupt, records};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const DEFAULT_SCHEDULER_TICK_SECS: u64 = 15;
pub const MIN_INTERVAL_SECS: u64 = 1;
pub const MAX_INTERVAL_SECS: u64 = 365 * 86_400;
pub const MAX_RUN_HISTORY: usize = 500;

static RUN_SEQ: AtomicU64 = AtomicU64::new(0);
static HISTORY_LOCK: Mutex<()> = Mutex::new(());

fn with_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = HISTORY_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cadence {
    Manual,
    IntervalSecs(u64),
    Hourly { minute: u8 },
    Daily { hour: u8, minute: u8 },
    /// `weekday`: 0 = Mon, 1 = Tue, 2 = Wed, 3 = Thu, 4 = Fri, 5 = Sat, 6 = Sun.
    Weekly { weekday: u8, hour: u8, minute: u8 },
}

impl Cadence {
    pub fn canonical_label(&self) -> String {
        match *self {
            Cadence::Manual => "manual".to_string(),
            Cadence::IntervalSecs(s) => {
                if s % 86_400 == 0 {
                    format!("every {}d", s / 86_400)
                } else if s % 3_600 == 0 {
                    format!("every {}h", s / 3_600)
                } else if s % 60 == 0 {
                    format!("every {}m", s / 60)
                } else {
                    format!("every {s}s")
                }
            }
            Cadence::Hourly { minute: 0 } => "hourly".to_string(),
            Cadence::Hourly { minute } => format!("hourly :{minute:02}"),
            Cadence::Daily { hour, minute } => format!("daily {hour:02}:{minute:02}"),
            Cadence::Weekly {
                weekday,
                hour,
                minute,
            } => {
                let day = weekday_short(weekday);
                if hour == 9 && minute == 0 {
                    format!("weekly {day}")
                } else {
                    format!("weekly {day} {hour:02}:{minute:02}")
                }
            }
        }
    }

    pub fn interval_secs(&self) -> Option<u64> {
        match *self {
            Cadence::Manual => None,
            Cadence::IntervalSecs(s) => Some(s),
            Cadence::Hourly { .. } => Some(3_600),
            Cadence::Daily { .. } => Some(86_400),
            Cadence::Weekly { .. } => Some(7 * 86_400),
        }
    }

    pub fn is_scheduled(&self) -> bool {
        !matches!(self, Cadence::Manual)
    }
}

fn weekday_short(w: u8) -> &'static str {
    match w {
        0 => "mon",
        1 => "tue",
        2 => "wed",
        3 => "thu",
        4 => "fri",
        5 => "sat",
        _ => "sun",
    }
}

fn parse_weekday(tok: &str) -> Option<u8> {
    match tok.trim().to_ascii_lowercase().as_str() {
        "mon" | "monday" => Some(0),
        "tue" | "tues" | "tuesday" => Some(1),
        "wed" | "weds" | "wednesday" => Some(2),
        "thu" | "thur" | "thurs" | "thursday" => Some(3),
        "fri" | "friday" => Some(4),
        "sat" | "saturday" => Some(5),
        "sun" | "sunday" => Some(6),
        _ => None,
    }
}

fn parse_hhmm(tok: &str) -> Result<(u8, u8), String> {
    let Some((h_str, m_str)) = tok.trim().split_once(':') else {
        return Err(format!("invalid time '{tok}', expected HH:MM"));
    };
    let hour: u8 = h_str
        .parse()
        .map_err(|_| format!("invalid hour in '{tok}'"))?;
    let minute: u8 = m_str
        .parse()
        .map_err(|_| format!("invalid minute in '{tok}'"))?;
    if hour > 23 || minute > 59 {
        return Err(format!("time out of range in '{tok}' (expected 00:00..23:59)"));
    }
    Ok((hour, minute))
}

fn parse_duration_secs(tokens: &[&str]) -> Result<u64, String> {
    if tokens.is_empty() {
        return Err("missing duration after 'every'".to_string());
    }
    let (num_str, unit_str) = if tokens.len() == 1 {
        let s = tokens[0];
        let split_at = s
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(s.len());
        (&s[..split_at], &s[split_at..])
    } else if tokens.len() == 2 {
        (tokens[0], tokens[1])
    } else {
        return Err(format!("unexpected extra tokens in duration: {}", tokens.join(" ")));
    };

    let amount: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid interval number '{num_str}'"))?;
    if amount == 0 {
        return Err("interval must be greater than zero".to_string());
    }
    let mult: u64 = match unit_str.trim().to_ascii_lowercase().as_str() {
        "" | "s" | "sec" | "secs" | "second" | "seconds" => 1,
        "m" | "min" | "mins" | "minute" | "minutes" => 60,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3_600,
        "d" | "day" | "days" => 86_400,
        other => {
            return Err(format!(
                "unsupported interval unit '{other}' (use s, m, h, or d)"
            ))
        }
    };
    let secs = amount
        .checked_mul(mult)
        .ok_or_else(|| "interval overflows maximum duration".to_string())?;
    if !(MIN_INTERVAL_SECS..=MAX_INTERVAL_SECS).contains(&secs) {
        return Err(format!(
            "interval must be between {MIN_INTERVAL_SECS}s and {MAX_INTERVAL_SECS}s"
        ));
    }
    Ok(secs)
}

/// Parse a schedule string (`manual`, `every 15m`, `every 30s`, `hourly`,
/// `daily 09:00`, `weekly mon`, `weekly fri 17:30`, or raw seconds).
pub fn parse_cadence(spec: &str) -> Result<Cadence, String> {
    let lower = spec.trim().to_ascii_lowercase();
    if lower.is_empty() || matches!(lower.as_str(), "manual" | "off" | "none" | "on-demand") {
        return Ok(Cadence::Manual);
    }

    let tokens: Vec<&str> = lower.split_whitespace().collect();
    match tokens[0] {
        "every" => {
            let secs = parse_duration_secs(&tokens[1..])?;
            Ok(Cadence::IntervalSecs(secs))
        }
        "hourly" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Hourly { minute: 0 });
            }
            if tokens.len() == 2 {
                let raw_m = tokens[1].trim_start_matches(':');
                let minute: u8 = raw_m
                    .parse()
                    .map_err(|_| format!("invalid hourly minute '{}'", tokens[1]))?;
                if minute > 59 {
                    return Err(format!("minute out of range: {minute}"));
                }
                return Ok(Cadence::Hourly { minute });
            }
            Err(format!("invalid hourly cadence: '{spec}'"))
        }
        "daily" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Daily { hour: 9, minute: 0 });
            }
            if tokens.len() == 2 {
                let (hour, minute) = parse_hhmm(tokens[1])?;
                return Ok(Cadence::Daily { hour, minute });
            }
            Err(format!("invalid daily cadence: '{spec}'"))
        }
        "weekly" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Weekly {
                    weekday: 0,
                    hour: 9,
                    minute: 0,
                });
            }
            let weekday = parse_weekday(tokens[1])
                .ok_or_else(|| format!("invalid weekday '{}' in '{spec}'", tokens[1]))?;
            if tokens.len() == 2 {
                return Ok(Cadence::Weekly {
                    weekday,
                    hour: 9,
                    minute: 0,
                });
            }
            if tokens.len() == 3 {
                let (hour, minute) = parse_hhmm(tokens[2])?;
                return Ok(Cadence::Weekly {
                    weekday,
                    hour,
                    minute,
                });
            }
            Err(format!("invalid weekly cadence: '{spec}'"))
        }
        _ => {
            // Support compact forms like "15m", "300s", "900".
            if tokens.len() == 1 {
                let secs = parse_duration_secs(&tokens)?;
                return Ok(Cadence::IntervalSecs(secs));
            }
            Err(format!(
                "unsupported cadence '{spec}' (use 'manual', 'every 15m', 'hourly', 'daily 09:00', or 'weekly mon')"
            ))
        }
    }
}

/// Resolve cadence from an optional `schedule` string and/or explicit
/// `interval_secs` integer (explicit `interval_secs` takes precedence when > 0).
pub fn parse_cadence_or_interval(
    schedule: Option<&str>,
    interval_secs: Option<u64>,
) -> Result<Cadence, String> {
    if let Some(secs) = interval_secs {
        if !(MIN_INTERVAL_SECS..=MAX_INTERVAL_SECS).contains(&secs) {
            return Err(format!(
                "interval_secs must be between {MIN_INTERVAL_SECS} and {MAX_INTERVAL_SECS}"
            ));
        }
        return Ok(Cadence::IntervalSecs(secs));
    }
    parse_cadence(schedule.unwrap_or("manual"))
}

/// Howard Hinnant's exact civil date -> days since 1970-01-01 (UTC).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let d = day as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + (doe as i64) - 719_468
}

/// Day of week from days since Unix epoch: 0 = Monday .. 6 = Sunday.
fn weekday_from_days(days_since_epoch: i64) -> u8 {
    (days_since_epoch + 3).rem_euclid(7) as u8
}

fn utc_day_ms(days: i64, hour: u8, minute: u8) -> u64 {
    let secs = days
        .saturating_mul(86_400)
        .saturating_add((hour as i64) * 3_600)
        .saturating_add((minute as i64) * 60);
    if secs <= 0 {
        0
    } else {
        (secs as u64).saturating_mul(1_000)
    }
}

/// Compute the next scheduled execution timestamp (Unix millis UTC) strictly
/// after `anchor = last_run_ms.unwrap_or(now_ms)`. Returns `None` for `Manual`.
pub fn next_run_ms(cadence: &Cadence, last_run_ms: Option<u64>, now_ms: u64) -> Option<u64> {
    let anchor = last_run_ms.filter(|&t| t > 0).unwrap_or(now_ms);
    match *cadence {
        Cadence::Manual => None,
        Cadence::IntervalSecs(secs) => Some(anchor.saturating_add(secs.saturating_mul(1_000))),
        Cadence::Hourly { minute } => {
            let hour_ms: u64 = 3_600_000;
            let hour_floor = (anchor / hour_ms) * hour_ms;
            let candidate = hour_floor.saturating_add((minute as u64) * 60_000);
            if candidate > anchor {
                Some(candidate)
            } else {
                Some(candidate.saturating_add(hour_ms))
            }
        }
        Cadence::Daily { hour, minute } => {
            let (y, m, d) = records::civil_from_millis(anchor);
            let day = days_from_civil(y, m, d);
            let today_slot = utc_day_ms(day, hour, minute);
            if today_slot > anchor {
                Some(today_slot)
            } else {
                Some(utc_day_ms(day + 1, hour, minute))
            }
        }
        Cadence::Weekly {
            weekday,
            hour,
            minute,
        } => {
            let (y, m, d) = records::civil_from_millis(anchor);
            let day = days_from_civil(y, m, d);
            let cur_w = weekday_from_days(day);
            let delta = ((weekday as i64) - (cur_w as i64)).rem_euclid(7);
            let candidate = utc_day_ms(day + delta, hour, minute);
            if candidate > anchor {
                Some(candidate)
            } else {
                Some(utc_day_ms(day + delta + 7, hour, minute))
            }
        }
    }
}

/// Compute `next_run_ms` for a persisted routine record (accounting for
/// `enabled`, `created_ms`, and `last_run_ms`). Returns `None` when paused or
/// manual.
pub fn compute_routine_next_run_ms(routine: &Value, now_ms: u64) -> Option<u64> {
    if !routine["enabled"].as_bool().unwrap_or(true) {
        return None;
    }
    let cadence = parse_cadence_or_interval(
        routine["schedule"].as_str(),
        routine["interval_secs"].as_u64(),
    )
    .ok()?;
    if !cadence.is_scheduled() {
        return None;
    }
    let last_run = routine["last_run_ms"].as_u64().filter(|&t| t > 0);
    let created = routine["created_ms"].as_u64().filter(|&t| t > 0);
    let anchor = last_run.or(created).unwrap_or(now_ms);
    next_run_ms(&cadence, Some(anchor), now_ms)
}

/// Whether `routine` is enabled, has a valid recurring cadence, and is due at `now_ms`.
pub fn is_routine_due(routine: &Value, now_ms: u64) -> bool {
    compute_routine_next_run_ms(routine, now_ms)
        .map(|due_at| due_at <= now_ms)
        .unwrap_or(false)
}

/// Filter and sort all due routines (earliest `next_run_ms` first).
pub fn find_due_routines(routines: &[Value], now_ms: u64) -> Vec<Value> {
    let mut due: Vec<(u64, Value)> = routines
        .iter()
        .filter_map(|r| {
            let next = compute_routine_next_run_ms(r, now_ms)?;
            if next <= now_ms {
                Some((next, r.clone()))
            } else {
                None
            }
        })
        .collect();
    due.sort_by_key(|(next, _)| *next);
    due.into_iter().map(|(_, r)| r).collect()
}

/// Whether background ticking is disabled via `ERGON_DISABLE_SCHEDULER=1|true|yes`.
pub fn is_scheduler_disabled_by_env() -> bool {
    std::env::var("ERGON_DISABLE_SCHEDULER")
        .map(|v| {
            let s = v.trim();
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

/// Apply a `PATCH /api/routines/:id` JSON payload to a routine object in-place.
/// Supports `enabled`, `schedule`, `interval_secs`, `name`, `prompt`, `skill_id`.
pub fn apply_routine_patch(routine: &mut Value, patch: &Value) -> Result<(), String> {
    let Some(obj) = routine.as_object_mut() else {
        return Err("routine record must be a JSON object".to_string());
    };
    if let Some(enabled) = patch.get("enabled").and_then(|v| v.as_bool()) {
        obj.insert("enabled".to_string(), json!(enabled));
    }
    if let Some(name) = patch.get("name").and_then(|v| v.as_str()) {
        let clean: String = name
            .replace(['\r', '\n'], " ")
            .trim()
            .chars()
            .take(120)
            .collect();
        if clean.is_empty() {
            return Err("routine name cannot be empty".to_string());
        }
        obj.insert("name".to_string(), json!(clean));
    }
    if let Some(prompt) = patch.get("prompt").and_then(|v| v.as_str()) {
        let clean: String = prompt.trim().chars().take(4000).collect();
        if clean.is_empty() {
            return Err("routine prompt cannot be empty".to_string());
        }
        obj.insert("prompt".to_string(), json!(clean));
    }
    if patch.get("schedule").is_some() || patch.get("interval_secs").is_some() {
        let interval = patch.get("interval_secs").and_then(|v| v.as_u64());
        let sched = patch.get("schedule").and_then(|v| v.as_str());
        if patch.get("interval_secs").is_some() && interval.is_none() && !patch["interval_secs"].is_null() {
            return Err("interval_secs must be a positive integer".to_string());
        }
        let cadence = parse_cadence_or_interval(sched, interval)?;
        obj.insert("schedule".to_string(), json!(cadence.canonical_label()));
        match cadence.interval_secs() {
            Some(secs) => {
                obj.insert("interval_secs".to_string(), json!(secs));
            }
            None => {
                obj.remove("interval_secs");
            }
        }
    }
    if let Some(skill) = patch.get("skill_id") {
        if skill.is_null() {
            obj.insert("skill_id".to_string(), Value::Null);
        } else if let Some(s) = skill.as_str() {
            let clean: String = s.trim().chars().take(64).collect();
            obj.insert(
                "skill_id".to_string(),
                if clean.is_empty() {
                    Value::Null
                } else {
                    json!(clean)
                },
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Atomic Routine Run History Store (`routine_history.json`)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutineRunRecord {
    pub run_id: String,
    pub routine_id: String,
    pub routine_name: String,
    pub started_ms: u64,
    pub finished_ms: u64,
    pub duration_ms: u64,
    pub status: String, // "ok" | "error" | "cancelled"
    pub receipt_id: Option<String>,
    pub thread_id: Option<String>,
    pub summary: String,
    pub trigger: String, // "scheduled" | "manual"
    pub error: Option<String>,
}

impl RoutineRunRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "run_id": self.run_id,
            "routine_id": self.routine_id,
            "routine_name": self.routine_name,
            "started_ms": self.started_ms,
            "finished_ms": self.finished_ms,
            "duration_ms": self.duration_ms,
            "status": self.status,
            "receipt_id": self.receipt_id,
            "thread_id": self.thread_id,
            "summary": self.summary,
            "trigger": self.trigger,
            "error": self.error,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let started_ms = v["started_ms"].as_u64().unwrap_or(0);
        let finished_ms = v["finished_ms"].as_u64().unwrap_or(started_ms);
        let duration_ms = v["duration_ms"]
            .as_u64()
            .unwrap_or_else(|| finished_ms.saturating_sub(started_ms));
        Some(RoutineRunRecord {
            run_id: v["run_id"].as_str()?.to_string(),
            routine_id: v["routine_id"].as_str()?.to_string(),
            routine_name: v["routine_name"].as_str().unwrap_or("").to_string(),
            started_ms,
            finished_ms,
            duration_ms,
            status: v["status"].as_str().unwrap_or("ok").to_string(),
            receipt_id: v["receipt_id"].as_str().map(|s| s.to_string()),
            thread_id: v["thread_id"].as_str().map(|s| s.to_string()),
            summary: v["summary"].as_str().unwrap_or("").to_string(),
            trigger: v["trigger"].as_str().unwrap_or("scheduled").to_string(),
            error: v["error"].as_str().map(|s| s.to_string()),
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub fn new_run_record(
    routine_id: &str,
    routine_name: &str,
    started_ms: u64,
    finished_ms: u64,
    status: &str,
    receipt_id: Option<String>,
    thread_id: Option<String>,
    summary: &str,
    trigger: &str,
    error: Option<String>,
) -> RoutineRunRecord {
    let seq = RUN_SEQ.fetch_add(1, Ordering::SeqCst);
    let end = finished_ms.max(started_ms);
    RoutineRunRecord {
        run_id: format!("rrun-{started_ms}-{seq}"),
        routine_id: routine_id.to_string(),
        routine_name: routine_name.to_string(),
        started_ms,
        finished_ms: end,
        duration_ms: end.saturating_sub(started_ms),
        status: status.to_string(),
        receipt_id,
        thread_id,
        summary: summary.chars().take(1000).collect(),
        trigger: trigger.to_string(),
        error: error.map(|e| e.chars().take(500).collect()),
    }
}

pub fn history_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("routine_history.json")
}

fn load_history_unlocked(path: &Path) -> Vec<RoutineRunRecord> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let parsed = serde_json::from_str::<Value>(&text);
    let arr = parsed
        .as_ref()
        .ok()
        .and_then(|v| v["runs"].as_array());
    match arr {
        Some(items) => items
            .iter()
            .filter_map(RoutineRunRecord::from_json)
            .collect(),
        None => {
            let _ = backup_corrupt(path);
            Vec::new()
        }
    }
}

fn save_history_unlocked(path: &Path, runs: &[RoutineRunRecord]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir routine history dir: {e}"))?;
    }
    let tmp = path.with_extension("json.tmp");
    let body = json!({
        "runs": runs.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    })
    .to_string();
    std::fs::write(&tmp, body).map_err(|e| format!("write routine history tmp: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename routine history: {e}"))?;
    Ok(())
}

/// Append one `RoutineRunRecord` atomically (capping stored history at `MAX_RUN_HISTORY`).
pub fn record_run_history(
    person_data_dir: &Path,
    record: RoutineRunRecord,
) -> Result<RoutineRunRecord, String> {
    with_lock(|| {
        let path = history_path(person_data_dir);
        let mut runs = load_history_unlocked(&path);
        runs.push(record.clone());
        if runs.len() > MAX_RUN_HISTORY {
            let excess = runs.len() - MAX_RUN_HISTORY;
            runs.drain(0..excess);
        }
        save_history_unlocked(&path, &runs)?;
        Ok(record)
    })
}

/// List routine run history newest-first, optionally filtered to one `routine_id`.
pub fn list_run_history(
    person_data_dir: &Path,
    routine_id: Option<&str>,
    limit: usize,
) -> Vec<RoutineRunRecord> {
    with_lock(|| {
        let mut runs = load_history_unlocked(&history_path(person_data_dir));
        runs.reverse();
        if let Some(rid) = routine_id {
            runs.retain(|r| r.routine_id == rid);
        }
        if limit > 0 && runs.len() > limit {
            runs.truncate(limit);
        }
        runs
    })
}

/// Enrich a routine JSON record with live scheduler fields (`next_run_ms`,
/// `cadence_label`, `interval_secs`, `due_now`, `in_seconds`, `history_count`).
pub fn enrich_routine_json(routine: &Value, now_ms: u64, history: &[RoutineRunRecord]) -> Value {
    let mut out = routine.clone();
    let enabled = routine["enabled"].as_bool().unwrap_or(true);
    let parsed = parse_cadence_or_interval(
        routine["schedule"].as_str(),
        routine["interval_secs"].as_u64(),
    );
    let (cadence_valid, cadence_label, interval_secs) = match parsed {
        Ok(c) => (true, c.canonical_label(), c.interval_secs()),
        Err(_) => (
            false,
            routine["schedule"].as_str().unwrap_or("manual").to_string(),
            None,
        ),
    };
    let next = compute_routine_next_run_ms(routine, now_ms);
    let due_now = enabled && next.map(|n| n <= now_ms).unwrap_or(false);
    let in_seconds = next.map(|n| n.saturating_sub(now_ms) / 1_000);

    let rid = routine["id"].as_str().unwrap_or("");
    let matching: Vec<&RoutineRunRecord> = history.iter().filter(|h| h.routine_id == rid).collect();
    let last_receipt_id = matching.first().and_then(|h| h.receipt_id.clone());

    if let Some(obj) = out.as_object_mut() {
        obj.insert("enabled".to_string(), json!(enabled));
        obj.insert("cadence_valid".to_string(), json!(cadence_valid));
        obj.insert("cadence_label".to_string(), json!(cadence_label));
        obj.insert("interval_secs".to_string(), json!(interval_secs));
        obj.insert("next_run_ms".to_string(), json!(next));
        obj.insert("due_now".to_string(), json!(due_now));
        obj.insert("in_seconds".to_string(), json!(in_seconds));
        obj.insert("history_count".to_string(), json!(matching.len()));
        if obj.get("last_receipt_id").is_none() {
            obj.insert("last_receipt_id".to_string(), json!(last_receipt_id));
        }
    }
    out
}

/// Summary status snapshot of the background routine scheduler for UI & APIs.
pub fn scheduler_status_json(
    scheduler_enabled: bool,
    tick_secs: u64,
    routines: &[Value],
    history: &[RoutineRunRecord],
    now_ms: u64,
) -> Value {
    let mut active_routines = 0usize;
    let mut paused_routines = 0usize;
    let mut manual_routines = 0usize;
    let mut due_count = 0usize;
    let mut next_due_ms: Option<u64> = None;

    for r in routines {
        let enabled = r["enabled"].as_bool().unwrap_or(true);
        let scheduled = parse_cadence_or_interval(
            r["schedule"].as_str(),
            r["interval_secs"].as_u64(),
        )
        .map(|c| c.is_scheduled())
        .unwrap_or(false);

        if !scheduled {
            manual_routines += 1;
        } else if !enabled {
            paused_routines += 1;
        } else {
            active_routines += 1;
            if let Some(next) = compute_routine_next_run_ms(r, now_ms) {
                if next <= now_ms {
                    due_count += 1;
                }
                next_due_ms = Some(next_due_ms.map(|cur| cur.min(next)).unwrap_or(next));
            }
        }
    }

    let last_run_ms = history.first().map(|h| h.finished_ms);
    json!({
        "enabled": scheduler_enabled,
        "tick_secs": tick_secs,
        "active_routines": active_routines,
        "paused_routines": paused_routines,
        "manual_routines": manual_routines,
        "due_count": due_count,
        "next_due_ms": next_due_ms,
        "total_runs": history.len(),
        "last_run_ms": last_run_ms,
    })
}
