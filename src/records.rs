//! The day-to-day records: the work log, the decisions inbox and the
//! offers inbox, plus the brief / today / graph assemblies built from
//! them. All per-person, all plain JSON files in the person's data
//! dir, all written by the app itself (never by the UI layer faking
//! state the agent did not produce).

use crate::library::{Library, Suite};
use crate::{sandbox_path_resolving, truncate, Config};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Every records store is a JSON file maintained by load → mutate →
/// save. The server is threaded and the same person can be reached
/// from desktop, phone and API at once, so those sequences are
/// serialised through this lock — without it, concurrent writers
/// silently lose each other's records (and mint colliding ids from
/// the same loaded length). The lock is never held across a model
/// call or a Rune run; it guards file sequences only.
static RECORDS_LOCK: Mutex<()> = Mutex::new(());

fn with_records_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = RECORDS_LOCK.lock().unwrap();
    f()
}

/// Process-unique sequence for record ids (timestamps alone collide
/// when two records are created in the same millisecond).
static RECORD_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_seq() -> u64 {
    RECORD_SEQ.fetch_add(1, Ordering::SeqCst)
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// (year, month, day) for a unix-millis timestamp, UTC. The civil
/// conversion is the standard days-from-epoch algorithm; no date
/// crate needed for day-granularity records.
pub fn civil_from_millis(ms: u64) -> (i64, u32, u32) {
    let days = (ms / 86_400_000) as i64;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn date_string(ms: u64) -> String {
    let (y, m, d) = civil_from_millis(ms);
    format!("{y:04}-{m:02}-{d:02}")
}

/// The UTC offset (minutes) in force at `unix_secs` in the zone file
/// at `path` — a minimal TZif reader (versions 1–4; only transition
/// times and ttinfo offsets are needed). Returns None on any parse
/// problem; callers fall back to UTC rather than guessing.
pub fn tz_offset_minutes_at(path: &Path, unix_secs: i64) -> Option<i64> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 44 || &data[0..4] != b"TZif" {
        return None;
    }
    let version = data[4];
    let be32 = |b: &[u8]| -> i64 {
        i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64
    };
    let counts = |hdr: &[u8]| -> (usize, usize, usize, usize) {
        // (timecnt, typecnt, charcnt, leapcnt) at fixed offsets in
        // the 44-byte header.
        let timecnt = be32(&hdr[32..36]) as usize;
        let typecnt = be32(&hdr[36..40]) as usize;
        let charcnt = be32(&hdr[40..44]) as usize;
        let leapcnt = be32(&hdr[28..32]) as usize;
        (timecnt, typecnt, charcnt, leapcnt)
    };
    let indicator_counts = |hdr: &[u8]| -> (usize, usize) {
        // (isutcnt, isstdcnt) — the two trailing indicator arrays of
        // a TZif block, needed to step over a v1 block exactly.
        (
            be32(&hdr[20..24]) as usize,
            be32(&hdr[24..28]) as usize,
        )
    };
    let parse_block = |start: usize, wide: bool, hdr: &[u8]| -> Option<(Vec<i64>, Vec<u8>, Vec<i64>)> {
        let (timecnt, typecnt, _charcnt, _leapcnt) = counts(hdr);
        let mut p = start;
        let mut transitions: Vec<i64> = Vec::new();
        if wide {
            for _ in 0..timecnt {
                if p + 8 > data.len() {
                    return None;
                }
                transitions.push(i64::from_be_bytes([
                    data[p], data[p + 1], data[p + 2], data[p + 3], data[p + 4], data[p + 5],
                    data[p + 6], data[p + 7],
                ]));
                p += 8;
            }
        } else {
            for _ in 0..timecnt {
                if p + 4 > data.len() {
                    return None;
                }
                transitions.push(be32(&data[p..p + 4]));
                p += 4;
            }
        }
        let mut indices: Vec<u8> = Vec::new();
        for _ in 0..timecnt {
            if p >= data.len() {
                return None;
            }
            indices.push(data[p]);
            p += 1;
        }
        let mut offsets: Vec<i64> = Vec::new();
        for _ in 0..typecnt {
            if p + 6 > data.len() {
                return None;
            }
            offsets.push(be32(&data[p..p + 4]));
            p += 6;
        }
        Some((transitions, indices, offsets))
    };

    let (transitions, indices, offsets) = if version == 0 {
        parse_block(44, false, &data[0..44])?
    } else {
        // Skip the v1 block to reach the v2 header + wide block.
        let (tc, ty, ch, lp) = counts(&data[0..44]);
        let (isut, isstd) = indicator_counts(&data[0..44]);
        let v1_len = tc * 4 + tc + ty * 6 + ch + lp * 8 + isstd + isut;
        let hdr2_start = 44 + v1_len;
        if hdr2_start + 44 > data.len() || &data[hdr2_start..hdr2_start + 4] != b"TZif" {
            return None;
        }
        parse_block(hdr2_start + 44, true, &data[hdr2_start..hdr2_start + 44])?
    };
    if offsets.is_empty() {
        return None;
    }
    // The type in force: the last transition at or before the time,
    // else the first type (pre-first-transition LMT entries are
    // close enough for day boundaries).
    let mut idx = 0usize;
    for (i, t) in transitions.iter().enumerate() {
        if *t <= unix_secs {
            idx = indices.get(i).copied().unwrap_or(0) as usize;
        } else {
            break;
        }
    }
    offsets.get(idx).map(|secs| secs / 60)
}

/// The host's local UTC offset at a moment: $TZ's zone file when it
/// names one, else /etc/localtime, else UTC. The records' "today" is
/// the person's today, not UTC's — a desk in New York flips its
/// Today River at 19:00 if UTC is used.
pub fn local_offset_minutes(ms: u64) -> i64 {
    let secs = (ms / 1000) as i64;
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(tz) = std::env::var("TZ") {
        let tz = tz.trim_start_matches(':');
        if !tz.is_empty() {
            candidates.push(std::path::PathBuf::from(tz));
            candidates.push(std::path::PathBuf::from("/usr/share/zoneinfo").join(tz));
        }
    }
    candidates.push(std::path::PathBuf::from("/etc/localtime"));
    for path in candidates {
        if let Some(off) = tz_offset_minutes_at(&path, secs) {
            return off;
        }
    }
    0
}

/// The local calendar date of a timestamp on the host's clock.
pub fn date_string_local(ms: u64) -> String {
    let offset_ms = local_offset_minutes(ms) * 60_000;
    let shifted = (ms as i64).saturating_add(offset_ms);
    date_string(shifted.max(0) as u64)
}

fn load_array(path: &Path, key: &str) -> Vec<Value> {
    if let Ok(text) = std::fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(items) = v[key].as_array() {
                return items.clone();
            }
        }
    }
    Vec::new()
}

fn save_array(path: &Path, key: &str, items: &[Value]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let v = json!({ key: items });
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, v.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

// ---------------------------------------------------------------------------
// Work log — every finished run, appended by the agent loop itself.
// ---------------------------------------------------------------------------

pub fn work_path(person_data_dir: &Path) -> std::path::PathBuf {
    person_data_dir.join("work.json")
}

pub fn append_work(
    cfg: &Config,
    task: &str,
    summary: &str,
    files: &[String],
    thread_id: &str,
    finished: bool,
) {
    append_entry(cfg, task, summary, files, thread_id, finished, false);
}

/// A run with no tool step and no written file: an answer, not work.
/// It is kept in the log, marked `answered`, and left out of the brief,
/// the graph's work nodes and the offers scan (EB2).
pub fn append_answer(cfg: &Config, task: &str, summary: &str, thread_id: &str) {
    append_entry(cfg, task, summary, &[], thread_id, true, true);
}

fn append_entry(
    cfg: &Config,
    task: &str,
    summary: &str,
    files: &[String],
    thread_id: &str,
    finished: bool,
    answered: bool,
) {
    with_records_lock(|| {
        let path = work_path(&cfg.data_dir);
        let mut items = load_array(&path, "entries");
        items.push(json!({
            "ts": now_millis(),
            "task": task,
            "summary": summary,
            "files": files,
            "thread_id": thread_id,
            "finished": finished,
            "answered": answered,
        }));
        save_array(&path, "entries", &items);
    });
}

pub fn load_work(person_data_dir: &Path) -> Vec<Value> {
    load_array(&work_path(person_data_dir), "entries")
}

// ---------------------------------------------------------------------------
// Decisions — work that arrives as a decision. Consequential actions
// (deleting) park here instead of executing; the agent can also raise
// a question for its person. Approval of a delete executes it for
// real; every resolution is recorded.
// ---------------------------------------------------------------------------

pub fn decisions_path(person_data_dir: &Path) -> std::path::PathBuf {
    person_data_dir.join("decisions.json")
}

pub fn load_decisions(person_data_dir: &Path) -> Vec<Value> {
    load_array(&decisions_path(person_data_dir), "decisions")
}

/// If a pending delete decision for `rel` already exists, return its id so a
/// retrying model cannot spam duplicate pending approvals for the same file.
pub fn find_pending_delete(person_data_dir: &Path, rel: &str) -> Option<String> {
    load_decisions(person_data_dir).into_iter().find_map(|d| {
        if d["status"].as_str() == Some("pending")
            && d["kind"].as_str() == Some("delete")
            && d["path"].as_str() == Some(rel)
        {
            d["id"].as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

pub fn add_decision(
    person_data_dir: &Path,
    kind: &str,
    summary: &str,
    path: Option<&str>,
    thread_id: &str,
) -> String {
    with_records_lock(|| {
        let mut items = load_decisions(person_data_dir);
        let ts = now_millis();
        let id = format!("d{ts}-{}", next_seq());
        items.push(json!({
            "id": id,
            "ts": ts,
            "kind": kind,
            "summary": summary,
            "path": path,
            "thread_id": thread_id,
            "status": "pending",
            "resolved_ts": Value::Null,
            "outcome": Value::Null,
        }));
        save_array(&decisions_path(person_data_dir), "decisions", &items);
        id
    })
}

/// Resolve a pending decision. For an approved delete, the file is
/// deleted for real through the sandbox; the outcome string says
/// exactly what happened (including when the file was already gone).
/// Returns the outcome, or an error when the decision is unknown or
/// already resolved.
pub fn resolve_decision(
    cfg: &Config,
    id: &str,
    approve: bool,
) -> Result<(Value, String), String> {
    // The whole settle sequence — read, check pending, execute,
    // record — runs under the records lock: two approvals racing
    // must not both execute.
    with_records_lock(|| {
        let path = decisions_path(&cfg.data_dir);
        let mut items = load_decisions(&cfg.data_dir);
        let idx = items
            .iter()
            .position(|d| d["id"].as_str() == Some(id))
            .ok_or_else(|| "decision not found".to_string())?;
        if items[idx]["status"].as_str() != Some("pending") {
            return Err("that decision was already settled".to_string());
        }
        let kind = items[idx]["kind"].as_str().unwrap_or("").to_string();
        let target = items[idx]["path"].as_str().map(|s| s.to_string());
        let outcome = if !approve {
            "You said no — nothing was done.".to_string()
        } else if kind == "delete" {
            match target.as_deref() {
                // The FINAL component is not resolved: approving
                // "delete link.txt" deletes the link the person
                // named — never the file it points at. (Resolving it
                // deleted the target and reported the link's name:
                // the one consequential gate doing the wrong
                // irreversible thing. move_file already had this
                // right; delete now matches it.)
                Some(rel) => match sandbox_path_resolving(&cfg.workspace_dir, rel, false) {
                    Ok(full) => {
                        if let Ok(md) = full.symlink_metadata() {
                            if !md.file_type().is_symlink() && md.is_file() {
                                let _ = crate::file_history::record_revision(
                                    &cfg.data_dir,
                                    &cfg.workspace_dir,
                                    rel,
                                    "delete",
                                );
                            }
                            match std::fs::remove_file(&full) {
                                Ok(()) => format!("Approved and done: {rel} was deleted."),
                                Err(e) => format!("Approved, but deleting {rel} failed: {e}"),
                            }
                        } else {
                            format!(
                                "Approved: {rel} was already gone, so nothing needed deleting."
                            )
                        }
                    }
                    Err(e) => format!("Approved, but the path is not deletable: {e}"),
                },
                None => "Approved.".to_string(),
            }
        } else {
            "Approved.".to_string()
        };
        items[idx]["status"] = json!(if approve { "approved" } else { "denied" });
        items[idx]["resolved_ts"] = json!(now_millis());
        items[idx]["outcome"] = json!(outcome);
        let decision = items[idx].clone();
        save_array(&path, "decisions", &items);
        Ok((decision, outcome))
    })
}

// ---------------------------------------------------------------------------
// Offers — the agent noticing repeated work and offering to take it
// over. Generation is deterministic over the work log: no model is
// involved, so offers are provable offline and never invented.
// ---------------------------------------------------------------------------

pub fn offers_path(person_data_dir: &Path) -> std::path::PathBuf {
    person_data_dir.join("offers.json")
}

pub fn load_offers(person_data_dir: &Path) -> Vec<Value> {
    load_array(&offers_path(person_data_dir), "offers")
}

const STOPWORDS: [&str; 16] = [
    "the", "a", "an", "my", "please", "into", "and", "of", "to", "for", "me", "in", "on", "with",
    "from", "this",
];

/// A stable pattern key for a task: its first two significant words.
/// "Draft email to landlord", "Draft email to bank" and "Draft email
/// to boss" share the key "draft email" — the same kind of work with
/// a varying object. Grouping on the object too (a three-word key)
/// meant that family could never earn an offer. A task with fewer
/// than two significant words gets no key: a lone verb would group
/// unrelated work.
fn pattern_key(task: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for w in task.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if w.is_empty() || STOPWORDS.contains(&w) {
            continue;
        }
        words.push(w.to_string());
        if words.len() == 2 {
            break;
        }
    }
    if words.len() < 2 {
        return String::new();
    }
    words.join(" ")
}

/// Scan the work log and create any offers that are newly earned.
/// An offer key that already exists in ANY state (pending, accepted,
/// dismissed) is never offered again — a no is a no. Only FINISHED
/// work earns offers, and re-running an existing app does not count:
/// an app's own template in the log is the app being used, not a new
/// repeated request (counting it offered an app of an app run).
pub fn refresh_offers(cfg: &Config, suite: &Suite, library: &Library) -> Vec<Value> {
    with_records_lock(|| {
        let work_all = load_work(&cfg.data_dir);
        // The scan is over recent work only — the log grows forever,
        // and a pattern from a year ago is not a live habit.
        let work: Vec<&Value> = work_all
            .iter()
            .rev()
            .take(500)
            .filter(|e| e["finished"].as_bool() != Some(false))
            .filter(|e| e["answered"].as_bool() != Some(true))
            .collect();
        let app_templates: Vec<&str> = library
            .items
            .iter()
            .filter(|i| i.kind == "app" && !i.template.is_empty())
            .map(|i| i.template.as_str())
            .collect();
        let mut offers = load_offers(&cfg.data_dir);
        let known: Vec<String> = offers
            .iter()
            .filter_map(|o| o["key"].as_str().map(|s| s.to_string()))
            .collect();
        let mut changed = false;
        let ts = now_millis();

        // Rule 1: the same kind of task finished three or more times.
        let mut groups: Vec<(String, Vec<&Value>)> = Vec::new();
        for entry in &work {
            let task = entry["task"].as_str().unwrap_or("");
            if app_templates.contains(&task) {
                continue;
            }
            let key = pattern_key(task);
            if key.is_empty() {
                continue;
            }
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, entries)) => entries.push(entry),
                None => groups.push((key, vec![entry])),
            }
        }
        for (key, entries) in &groups {
            if entries.len() >= 3 && !known.contains(key) {
                let sample = entries
                    .first()
                    .and_then(|e| e["task"].as_str())
                    .unwrap_or("")
                    .to_string();
                offers.push(json!({
                    "id": format!("o{ts}-{}", next_seq()),
                    "ts": ts,
                    "kind": "repeat_task",
                    "key": key,
                    "title": format!("You often ask for this: \"{}\"", truncate(&sample, 80)),
                    "detail": format!(
                        "It has come up {} times. Turn it into a one-tap app in your suite, so next time it is one tap instead of a typed request?",
                        entries.len()
                    ),
                    "template": sample,
                    "status": "pending",
                }));
                changed = true;
            }
        }

        // Rule 2: spreadsheets keep getting produced by hand.
        let csv_count = work
            .iter()
            .filter(|e| {
                e["files"]
                    .as_array()
                    .map(|files| files.iter().any(|f| f.as_str().map(|s| s.ends_with(".csv")).unwrap_or(false)))
                    .unwrap_or(false)
            })
            .count();
        let artefact_key = "artefact:csv".to_string();
        if csv_count >= 3
            && !known.contains(&artefact_key)
            && !suite.installed.iter().any(|i| i == "expense-sheet")
            && library.get("expense-sheet").is_some()
        {
            offers.push(json!({
                "id": format!("o{ts}-{}", next_seq()),
                "ts": ts,
                "kind": "install_app",
                "key": artefact_key,
                "title": "You keep ending up with spreadsheets".to_string(),
                "detail": format!(
                    "{csv_count} finished jobs produced a spreadsheet. Add the Expense sheet app to your suite so the next one starts from a form, not a blank request?"
                ),
                "item_id": "expense-sheet",
                "status": "pending",
            }));
            changed = true;
        }

        if changed {
            save_array(&offers_path(&cfg.data_dir), "offers", &offers);
        }
        offers
    })
}

pub fn set_offer_status(person_data_dir: &Path, id: &str, status: &str) -> Result<Value, String> {
    with_records_lock(|| {
        let mut offers = load_offers(person_data_dir);
        let idx = offers
            .iter()
            .position(|o| o["id"].as_str() == Some(id))
            .ok_or_else(|| "offer not found".to_string())?;
        if offers[idx]["status"].as_str() != Some("pending") {
            return Err("that offer was already settled".to_string());
        }
        offers[idx]["status"] = json!(status);
        let offer = offers[idx].clone();
        save_array(&offers_path(person_data_dir), "offers", &offers);
        Ok(offer)
    })
}

// ---------------------------------------------------------------------------
// Brief, today, graph — assemblies over the real stores.
// ---------------------------------------------------------------------------

/// The morning brief: what the agent finished recently, what is
/// waiting on the person, and what it learned. Assembled from the
/// stores; empty sections say so plainly rather than being padded.
pub fn build_brief(cfg: &Config, person_name: &str, suite: &Suite, library: &Library) -> Value {
    let now = now_millis();
    let today = date_string_local(now);
    let work = load_work(&cfg.data_dir);
    let decisions = load_decisions(&cfg.data_dir);
    // A run that parked a decision ended, but its job waits on the
    // person: not finished until the decision is settled. Same rule as
    // the paper: any pending decision, matched by its thread.
    let waiting: Vec<&str> = decisions
        .iter()
        .filter(|d| d["status"].as_str() == Some("pending"))
        .filter_map(|d| d["thread_id"].as_str().filter(|t| !t.is_empty()))
        .collect();
    // Only finished work counts as finished in the brief — a run
    // that stopped early is visible in Today, not celebrated here.
    let recent: Vec<&Value> = work
        .iter()
        .filter(|e| e["finished"].as_bool() != Some(false))
        .filter(|e| e["answered"].as_bool() != Some(true))
        .filter(|e| !e["thread_id"].as_str().is_some_and(|t| waiting.contains(&t)))
        .filter(|e| now.saturating_sub(e["ts"].as_u64().unwrap_or(0)) <= 48 * 3_600_000)
        .collect();
    let finished: Vec<Value> = recent
        .iter()
        .rev()
        .take(8)
        .map(|e| {
            json!({
                "date": date_string_local(e["ts"].as_u64().unwrap_or(0)),
                "task": e["task"],
                "summary": e["summary"],
                "files": e["files"],
                "thread_id": e["thread_id"],
                "ts": e["ts"],
            })
        })
        .collect();
    // Runs that stopped early in the same window: not celebrated as finished, but listed so the person can pick them up.
    let stopped: Vec<Value> = work
        .iter()
        .filter(|e| e["finished"].as_bool() == Some(false))
        .filter(|e| now.saturating_sub(e["ts"].as_u64().unwrap_or(0)) <= 48 * 3_600_000)
        .rev()
        .take(5)
        .map(|e| json!({ "task": e["task"], "summary": e["summary"], "thread_id": e["thread_id"], "ts": e["ts"] }))
        .collect();
    let pending_decisions: Vec<Value> = decisions
        .iter()
        .filter(|d| d["status"].as_str() == Some("pending"))
        .map(|d| json!({"id": d["id"], "kind": d["kind"], "summary": d["summary"]}))
        .collect();
    let offers = load_offers(&cfg.data_dir);
    let pending_offers: Vec<Value> = offers
        .iter()
        .filter(|o| o["status"].as_str() == Some("pending"))
        .map(|o| json!({"id": o["id"], "title": o["title"], "detail": o["detail"]}))
        .collect();
    let memory = cfg.read_memory();
    let learned: Vec<String> = crate::standing_preferences(&memory)
        .into_iter()
        .rev()
        .take(5)
        .collect();
    json!({
        "date": today,
        "person": person_name,
        "headline": if recent.is_empty() {
            "Nothing finished in the last two days — the desk is clear.".to_string()
        } else {
            format!("{} job{} finished in the last two days.", recent.len(), if recent.len() == 1 { "" } else { "s" })
        },
        "finished": finished,
        "stopped": stopped,
        "pending_decisions": pending_decisions,
        "pending_offers": pending_offers,
        "learned_recently": learned,
        // Resolved count: a suite entry whose library item is gone
        // is a ghost and must not inflate the brief (the suite API
        // already hides it).
        "suite_size": suite.resolved(library).len(),
        "library_size": library.items.len(),
    })
}

/// Today's stream for the Today River face: work finished, decisions
/// raised and settled, offers made — one chronological list.
pub fn build_today(cfg: &Config) -> Value {
    let now = now_millis();
    let today = date_string_local(now);
    let mut events: Vec<Value> = Vec::new();
    for e in load_work(&cfg.data_dir) {
        let ts = e["ts"].as_u64().unwrap_or(0);
        if date_string_local(ts) == today {
            events.push(json!({
                "ts": ts, "kind": "work",
                "title": e["task"], "detail": e["summary"], "files": e["files"],
            }));
        }
    }
    for d in load_decisions(&cfg.data_dir) {
        let ts = d["ts"].as_u64().unwrap_or(0);
        if date_string_local(ts) == today {
            events.push(json!({
                "ts": ts, "kind": "decision",
                "title": d["summary"], "detail": format!("Waiting on you ({})", d["kind"]),
            }));
        }
        if let Some(rts) = d["resolved_ts"].as_u64() {
            if date_string_local(rts) == today {
                events.push(json!({
                    "ts": rts, "kind": "decision_settled",
                    "title": d["summary"], "detail": d["outcome"],
                }));
            }
        }
    }
    for o in load_offers(&cfg.data_dir) {
        let ts = o["ts"].as_u64().unwrap_or(0);
        if date_string_local(ts) == today {
            events.push(json!({
                "ts": ts, "kind": "offer",
                "title": o["title"], "detail": o["detail"],
            }));
        }
    }
    events.sort_by_key(|e| e["ts"].as_u64().unwrap_or(0));
    events.reverse();
    json!({ "date": today, "events": events })
}

/// The work graph: people, their recent finished work, the files it
/// produced, and the suite items they carry. Real stores only.
pub fn build_graph(people: &[crate::Person], root: &Config, library: &Library) -> Value {
    let mut nodes: Vec<Value> = Vec::new();
    let mut edges: Vec<Value> = Vec::new();
    for person in people {
        let pcfg = root.person_config(person);
        let pid = format!("person:{}", person.id);
        nodes.push(json!({
            "id": pid, "kind": "person",
            "label": if person.role.is_empty() { person.name.clone() } else { format!("{} — {}", person.name, person.role) },
        }));
        let suite = crate::library::load_suite(&pcfg.data_dir);
        for item_id in &suite.installed {
            let iid = format!("item:{item_id}");
            // No node, no edge: a suite entry whose library item is
            // gone must not leave an edge pointing at nothing.
            if library.get(item_id).is_none() {
                continue;
            }
            if !nodes.iter().any(|n| n["id"] == json!(iid)) {
                let item = library.get(item_id).expect("checked above");
                nodes.push(json!({
                    "id": iid, "kind": "item",
                    "label": item.name, "detail": item.kind,
                }));
            }
            edges.push(json!({"from": pid, "to": iid, "kind": "carries"}));
        }
        let work: Vec<Value> = load_work(&pcfg.data_dir)
            .into_iter()
            .filter(|e| e["answered"].as_bool() != Some(true))
            .collect();
        for (n, e) in work.iter().rev().take(12).enumerate() {
            let wid = format!("work:{}:{}", person.id, n);
            nodes.push(json!({
                "id": wid, "kind": "work",
                "label": truncate(e["task"].as_str().unwrap_or(""), 60),
                "detail": date_string_local(e["ts"].as_u64().unwrap_or(0)),
            }));
            edges.push(json!({"from": pid, "to": wid, "kind": "finished"}));
            if let Some(files) = e["files"].as_array() {
                for f in files {
                    if let Some(fname) = f.as_str() {
                        let fid = format!("file:{}:{fname}", person.id);
                        if !nodes.iter().any(|nd| nd["id"] == json!(fid)) {
                            nodes.push(json!({
                                "id": fid, "kind": "file", "label": fname,
                            }));
                        }
                        edges.push(json!({"from": wid, "to": fid, "kind": "produced"}));
                    }
                }
            }
        }
    }
    json!({ "nodes": nodes, "edges": edges })
}
