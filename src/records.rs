//! The day-to-day records: the work log, the decisions inbox and the
//! offers inbox, plus the brief / today / graph assemblies built from
//! them. All per-person, all plain JSON files in the person's data
//! dir, all written by the app itself (never by the UI layer faking
//! state the agent did not produce).

use crate::library::{Library, Suite};
use crate::tools::atomic_write_bytes_with_mode;
use crate::{sandbox_path_resolving, slugify, truncate, Config};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Write as _;
use std::path::{Path, PathBuf};
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
    append_entry(cfg, task, summary, files, thread_id, finished, false, None, None);
}

#[allow(clippy::too_many_arguments)]
pub fn append_work_with_usage(
    cfg: &Config,
    task: &str,
    summary: &str,
    files: &[String],
    thread_id: &str,
    finished: bool,
    engagement_id: Option<&str>,
    usage: Option<&RunUsage>,
) {
    append_entry(
        cfg,
        task,
        summary,
        files,
        thread_id,
        finished,
        false,
        engagement_id,
        usage,
    );
}

/// A run with no tool step and no written file: an answer, not work.
/// It is kept in the log, marked `answered`, and left out of the brief,
/// the graph's work nodes and the offers scan (EB2).
pub fn append_answer(cfg: &Config, task: &str, summary: &str, thread_id: &str) {
    append_entry(cfg, task, summary, &[], thread_id, true, true, None, None);
}

pub fn append_answer_with_usage(
    cfg: &Config,
    task: &str,
    summary: &str,
    thread_id: &str,
    engagement_id: Option<&str>,
    usage: Option<&RunUsage>,
) {
    append_entry(
        cfg,
        task,
        summary,
        &[],
        thread_id,
        true,
        true,
        engagement_id,
        usage,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_entry(
    cfg: &Config,
    task: &str,
    summary: &str,
    files: &[String],
    thread_id: &str,
    finished: bool,
    answered: bool,
    engagement_id: Option<&str>,
    usage: Option<&RunUsage>,
) {
    with_records_lock(|| {
        let path = work_path(&cfg.data_dir);
        let mut items = load_array(&path, "entries");
        let mut entry = json!({
            "ts": now_millis(),
            "task": task,
            "summary": summary,
            "files": files,
            "thread_id": thread_id,
            "finished": finished,
            "answered": answered,
        });
        if let Some(eid) = engagement_id.map(str::trim).filter(|s| !s.is_empty()) {
            entry["engagement_id"] = json!(eid);
        }
        if let Some(u) = usage {
            entry["usage"] = u.to_json();
        }
        items.push(entry);
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
    add_decision_with(person_data_dir, kind, summary, path, thread_id, Value::Null)
}

/// Like `add_decision`, with extra fields merged in (a tripwire alert's
/// `detail`, `severity`, `source`, or the `payload` an approval acts on).
/// The core fields always win over `extra`.
pub fn add_decision_with(
    person_data_dir: &Path,
    kind: &str,
    summary: &str,
    path: Option<&str>,
    thread_id: &str,
    extra: Value,
) -> String {
    with_records_lock(|| {
        let mut items = load_decisions(person_data_dir);
        let ts = now_millis();
        let id = format!("d{ts}-{}", next_seq());
        let mut item = json!({});
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                item[k] = v.clone();
            }
        }
        for (k, v) in [
            ("id", json!(id)),
            ("ts", json!(ts)),
            ("kind", json!(kind)),
            ("summary", json!(summary)),
            ("path", json!(path)),
            ("thread_id", json!(thread_id)),
            ("status", json!("pending")),
            ("resolved_ts", Value::Null),
            ("outcome", Value::Null),
        ] {
            item[k] = v;
        }
        items.push(item);
        save_array(&decisions_path(person_data_dir), "decisions", &items);
        id
    })
}

/// Replace a settled decision's outcome text — for approvals whose
/// action runs after the settle (it needs app state the records lock
/// must not be held across, like the company library).
pub fn set_decision_outcome(person_data_dir: &Path, id: &str, outcome: &str) -> Option<Value> {
    with_records_lock(|| {
        let mut items = load_decisions(person_data_dir);
        let idx = items.iter().position(|d| d["id"].as_str() == Some(id))?;
        items[idx]["outcome"] = json!(outcome);
        let out = items[idx].clone();
        save_array(&decisions_path(person_data_dir), "decisions", &items);
        Some(out)
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
    // must not both execute. Network-bound connector deliveries
    // transition state atomically under the lock and execute right
    // after releasing it.
    let (decision, outcome, kind) = with_records_lock(|| {
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
            if kind == "connector_delivery" {
                "You said no — outbound delivery was cancelled.".to_string()
            } else if kind == "budget_gate" {
                "You said no — retainer budget gate remains active.".to_string()
            } else {
                "You said no — nothing was done.".to_string()
            }
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
        } else if kind == "connector_delivery" {
            "Approved — executing outbound delivery...".to_string()
        } else if kind == "budget_gate" {
            "Approved — retainer budget gate lifted for the next run on this engagement."
                .to_string()
        } else {
            "Approved.".to_string()
        };
        items[idx]["status"] = json!(if approve { "approved" } else { "denied" });
        items[idx]["resolved_ts"] = json!(now_millis());
        items[idx]["outcome"] = json!(outcome);
        if kind == "budget_gate" && approve {
            items[idx]["budget_gate_consumed"] = json!(false);
        }
        let decision = items[idx].clone();
        save_array(&path, "decisions", &items);
        Ok((decision, outcome, kind))
    })?;

    if kind == "connector_delivery" {
        if approve {
            let final_outcome = crate::connectors::settle_delivery_decision(cfg, &decision);
            let updated =
                set_decision_outcome(&cfg.data_dir, id, &final_outcome).unwrap_or(decision);
            return Ok((updated, final_outcome));
        } else {
            crate::connectors::record_denied_delivery(cfg, &decision);
        }
    }
    Ok((decision, outcome))
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
            let what = match d["kind"].as_str().unwrap_or("") {
                "delete" => "a deletion",
                "question" => "a question",
                "security" | "security_notice" => "a security alert",
                "allow_host" => "an address to allow",
                "memory_note" => "a memory note",
                "publish_skill" => "a library skill",
                "budget_gate" => "a retainer budget gate",
                _ => "a decision",
            };
            events.push(json!({
                "ts": ts, "kind": "decision",
                "title": d["summary"], "detail": format!("Waiting on you ({what})"),
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

// ---------------------------------------------------------------------------
// Clients, Engagements, Model Pricing, Usage Ledger & Metering Summary (v0.33)
// ---------------------------------------------------------------------------

static CLIENTS_LOCK: Mutex<()> = Mutex::new(());

fn with_clients_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = CLIENTS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn round_usd_8(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * 100_000_000.0).round() / 100_000_000.0
}

pub fn round_4(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * 10_000.0).round() / 10_000.0
}

pub fn round_2(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * 100.0).round() / 100.0
}

fn clean_single_line(s: &str, cap: usize) -> String {
    let flat: String = s
        .replace(['\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    flat.chars().take(cap).collect()
}

pub fn valid_entity_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

fn non_neg_f64(v: &Value, default: f64, field: &str) -> Result<f64, (u16, String)> {
    if v.is_null() {
        return Ok(default);
    }
    let Some(n) = v.as_f64() else {
        return Err((400, format!("{field} must be a finite non-negative number")));
    };
    if !n.is_finite() || n < 0.0 {
        return Err((400, format!("{field} must be a finite non-negative number")));
    }
    Ok(n)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientRecord {
    pub id: String,
    pub name: String,
    pub brand_header: String,
    pub billing_currency: String,
    pub default_rate_usd: f64,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl ClientRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "brand_header": self.brand_header,
            "billing_currency": self.billing_currency,
            "default_rate_usd": round_4(self.default_rate_usd),
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        if !valid_entity_id(&id) {
            return None;
        }
        let name = clean_single_line(v["name"].as_str().unwrap_or(""), 120);
        if name.is_empty() {
            return None;
        }
        Some(Self {
            id,
            name,
            brand_header: clean_single_line(v["brand_header"].as_str().unwrap_or(""), 240),
            billing_currency: clean_single_line(
                v["billing_currency"].as_str().unwrap_or("USD"),
                12,
            )
            .to_ascii_uppercase(),
            default_rate_usd: v["default_rate_usd"]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngagementRecord {
    pub id: String,
    pub client_id: String,
    pub person_id: String,
    pub name: String,
    pub retainer_hours: f64,
    pub retainer_budget_usd: f64,
    pub hourly_rate_usd: f64,
    pub period: String,
    pub status: String,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl EngagementRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "client_id": self.client_id,
            "person_id": self.person_id,
            "name": self.name,
            "retainer_hours": round_4(self.retainer_hours),
            "retainer_budget_usd": round_4(self.retainer_budget_usd),
            "hourly_rate_usd": round_4(self.hourly_rate_usd),
            "period": self.period,
            "status": self.status,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let client_id = v["client_id"].as_str()?.trim().to_string();
        if !valid_entity_id(&id) || !valid_entity_id(&client_id) {
            return None;
        }
        let name = clean_single_line(v["name"].as_str().unwrap_or(""), 120);
        if name.is_empty() {
            return None;
        }
        let person_id = v["person_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| crate::valid_person_id(s))
            .unwrap_or("default")
            .to_string();
        Some(Self {
            id,
            client_id,
            person_id,
            name,
            retainer_hours: v["retainer_hours"]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            retainer_budget_usd: v["retainer_budget_usd"]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            hourly_rate_usd: v["hourly_rate_usd"]
                .as_f64()
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            period: clean_single_line(v["period"].as_str().unwrap_or("monthly"), 32),
            status: clean_single_line(v["status"].as_str().unwrap_or("active"), 32),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClientsStore {
    pub clients: Vec<ClientRecord>,
    pub engagements: Vec<EngagementRecord>,
    pub active_engagement_by_person: HashMap<String, String>,
}

impl ClientsStore {
    pub fn to_json(&self) -> Value {
        json!({
            "clients": self.clients.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "engagements": self.engagements.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
            "active_engagement_by_person": self.active_engagement_by_person,
        })
    }
}

pub fn clients_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("clients.json")
}

fn load_clients_store_unlocked(root_data_dir: &Path) -> ClientsStore {
    let path = clients_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return ClientsStore::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        let _ = crate::backup_corrupt(&path);
        return ClientsStore::default();
    };
    if !v.is_object() {
        let _ = crate::backup_corrupt(&path);
        return ClientsStore::default();
    }
    let clients: Vec<ClientRecord> = v["clients"]
        .as_array()
        .map(|arr| arr.iter().filter_map(ClientRecord::from_json).collect())
        .unwrap_or_default();
    let engagements: Vec<EngagementRecord> = v["engagements"]
        .as_array()
        .map(|arr| arr.iter().filter_map(EngagementRecord::from_json).collect())
        .unwrap_or_default();
    let mut active_engagement_by_person = HashMap::new();
    if let Some(obj) = v["active_engagement_by_person"].as_object() {
        for (pid, eid_val) in obj {
            if let Some(eid) = eid_val.as_str() {
                if crate::valid_person_id(pid) && engagements.iter().any(|e| e.id == eid) {
                    active_engagement_by_person.insert(pid.clone(), eid.to_string());
                }
            }
        }
    }
    ClientsStore {
        clients,
        engagements,
        active_engagement_by_person,
    }
}

fn save_clients_store_unlocked(root_data_dir: &Path, store: &ClientsStore) -> Result<(), String> {
    let path = clients_path(root_data_dir);
    let body = store.to_json().to_string();
    atomic_write_bytes_with_mode(&path, body.as_bytes(), Some(0o600))
        .map_err(|e| format!("save clients.json: {e}"))
}

pub fn load_clients_store(root_data_dir: &Path) -> ClientsStore {
    with_clients_lock(|| load_clients_store_unlocked(root_data_dir))
}

fn mint_unique_id(desired: Option<&str>, fallback_name: &str, existing: &[&str]) -> Result<String, (u16, String)> {
    if let Some(raw) = desired.map(str::trim).filter(|s| !s.is_empty()) {
        let low = raw.to_ascii_lowercase();
        if !valid_entity_id(&low) {
            return Err((
                400,
                "id must be 1..=64 lowercase alphanumeric, '-', or '_' characters".to_string(),
            ));
        }
        if existing.contains(&low.as_str()) {
            return Err((409, format!("id '{low}' already exists")));
        }
        return Ok(low);
    }
    let base = slugify(fallback_name);
    let mut candidate = base.clone();
    let mut n = 2usize;
    while existing.contains(&candidate.as_str()) {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    Ok(candidate)
}

pub fn create_client(root_data_dir: &Path, body: &Value) -> Result<ClientRecord, (u16, String)> {
    if !body.is_object() {
        return Err((400, "request body must be a JSON object".to_string()));
    }
    let name = clean_single_line(body["name"].as_str().unwrap_or(""), 120);
    if name.is_empty() {
        return Err((400, "name is required".to_string()));
    }
    let brand_header = clean_single_line(body["brand_header"].as_str().unwrap_or(""), 240);
    let raw_currency = clean_single_line(body["billing_currency"].as_str().unwrap_or("USD"), 12);
    let billing_currency = if raw_currency.is_empty() {
        "USD".to_string()
    } else {
        raw_currency.to_ascii_uppercase()
    };
    let default_rate_usd = non_neg_f64(&body["default_rate_usd"], 0.0, "default_rate_usd")?;

    with_clients_lock(|| {
        let mut store = load_clients_store_unlocked(root_data_dir);
        let existing: Vec<&str> = store.clients.iter().map(|c| c.id.as_str()).collect();
        let id = mint_unique_id(body["id"].as_str(), &name, &existing)?;
        let ts = now_millis();
        let client = ClientRecord {
            id,
            name,
            brand_header,
            billing_currency,
            default_rate_usd,
            created_ms: ts,
            updated_ms: ts,
        };
        store.clients.push(client.clone());
        save_clients_store_unlocked(root_data_dir, &store).map_err(|e| (500, e))?;
        Ok(client)
    })
}

pub fn create_engagement(
    root_data_dir: &Path,
    default_person_id: &str,
    body: &Value,
) -> Result<EngagementRecord, (u16, String)> {
    if !body.is_object() {
        return Err((400, "request body must be a JSON object".to_string()));
    }
    let client_id = body["client_id"].as_str().unwrap_or("").trim().to_string();
    if client_id.is_empty() {
        return Err((400, "client_id is required".to_string()));
    }
    let name = clean_single_line(body["name"].as_str().unwrap_or(""), 120);
    if name.is_empty() {
        return Err((400, "name is required".to_string()));
    }
    let person_id = body["person_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_person_id)
        .to_string();
    if !crate::valid_person_id(&person_id) {
        return Err((400, "invalid person_id".to_string()));
    }
    let retainer_hours = non_neg_f64(&body["retainer_hours"], 0.0, "retainer_hours")?;
    let retainer_budget_usd =
        non_neg_f64(&body["retainer_budget_usd"], 0.0, "retainer_budget_usd")?;
    let period = {
        let p = clean_single_line(body["period"].as_str().unwrap_or("monthly"), 32);
        if p.is_empty() {
            "monthly".to_string()
        } else {
            p.to_ascii_lowercase()
        }
    };
    let status = {
        let s = clean_single_line(body["status"].as_str().unwrap_or("active"), 32);
        if s.is_empty() {
            "active".to_string()
        } else {
            s.to_ascii_lowercase()
        }
    };
    let activate = body["activate"].as_bool().unwrap_or(false);

    with_clients_lock(|| {
        let mut store = load_clients_store_unlocked(root_data_dir);
        let Some(client) = store.clients.iter().find(|c| c.id == client_id).cloned() else {
            return Err((404, "client not found".to_string()));
        };
        let hourly_rate_usd = non_neg_f64(
            &body["hourly_rate_usd"],
            client.default_rate_usd,
            "hourly_rate_usd",
        )?;
        let existing: Vec<&str> = store.engagements.iter().map(|e| e.id.as_str()).collect();
        let id = mint_unique_id(body["id"].as_str(), &name, &existing)?;
        let ts = now_millis();
        let engagement = EngagementRecord {
            id: id.clone(),
            client_id,
            person_id: person_id.clone(),
            name,
            retainer_hours,
            retainer_budget_usd,
            hourly_rate_usd,
            period,
            status,
            created_ms: ts,
            updated_ms: ts,
        };
        store.engagements.push(engagement.clone());
        if activate {
            store.active_engagement_by_person.insert(person_id, id);
        }
        save_clients_store_unlocked(root_data_dir, &store).map_err(|e| (500, e))?;
        Ok(engagement)
    })
}

pub fn switch_engagement(
    root_data_dir: &Path,
    person_id: &str,
    body: &Value,
) -> Result<(Option<String>, Option<EngagementRecord>), (u16, String)> {
    if !body.is_object() {
        return Err((400, "request body must be a JSON object".to_string()));
    }
    let clear_requested = body["clear"].as_bool().unwrap_or(false)
        || body.get("engagement_id").is_some_and(Value::is_null)
        || body.get("id").is_some_and(Value::is_null);
    let requested = body["engagement_id"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .map(str::trim)
        .unwrap_or("");
    if !clear_requested && requested.is_empty() {
        return Err((400, "engagement_id is required".to_string()));
    }
    with_clients_lock(|| {
        let mut store = load_clients_store_unlocked(root_data_dir);
        if clear_requested && requested.is_empty() {
            store.active_engagement_by_person.remove(person_id);
            save_clients_store_unlocked(root_data_dir, &store).map_err(|e| (500, e))?;
            return Ok((None, None));
        }
        let Some(eng) = store
            .engagements
            .iter()
            .find(|e| e.id == requested)
            .cloned()
        else {
            return Err((404, "engagement not found".to_string()));
        };
        store
            .active_engagement_by_person
            .insert(person_id.to_string(), eng.id.clone());
        save_clients_store_unlocked(root_data_dir, &store).map_err(|e| (500, e))?;
        Ok((Some(eng.id.clone()), Some(eng)))
    })
}

pub fn active_engagement_for(
    root_data_dir: &Path,
    person_id: &str,
) -> Option<(EngagementRecord, Option<ClientRecord>)> {
    let store = load_clients_store(root_data_dir);
    let eid = store.active_engagement_by_person.get(person_id)?;
    let eng = store.engagements.iter().find(|e| &e.id == eid)?.clone();
    let client = store
        .clients
        .iter()
        .find(|c| c.id == eng.client_id)
        .cloned();
    Some((eng, client))
}

pub fn active_brand_header_for(root_data_dir: &Path, person_id: &str) -> Option<String> {
    let (_, client) = active_engagement_for(root_data_dir, person_id)?;
    let hdr = client?.brand_header.trim().to_string();
    if hdr.is_empty() {
        None
    } else {
        Some(hdr)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelPrice {
    pub input_per_1m_usd: f64,
    pub output_per_1m_usd: f64,
    pub cached_input_per_1m_usd: f64,
}

impl ModelPrice {
    pub fn to_json(&self) -> Value {
        json!({
            "input_per_1m_usd": self.input_per_1m_usd,
            "output_per_1m_usd": self.output_per_1m_usd,
            "cached_input_per_1m_usd": self.cached_input_per_1m_usd,
        })
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let obj = v
            .as_object()
            .ok_or_else(|| "model pricing entry must be an object".to_string())?;
        let read_rate = |keys: &[&str], default: f64| -> Result<f64, String> {
            for k in keys {
                if let Some(val) = obj.get(*k) {
                    if val.is_null() {
                        continue;
                    }
                    let n = val
                        .as_f64()
                        .filter(|n| n.is_finite() && *n >= 0.0)
                        .ok_or_else(|| format!("pricing field '{k}' must be a finite non-negative number"))?;
                    return Ok(n);
                }
            }
            Ok(default)
        };
        let input = read_rate(&["input_per_1m_usd", "prompt_per_1m_usd"], 0.0)?;
        let output = read_rate(&["output_per_1m_usd", "completion_per_1m_usd"], 0.0)?;
        let cached = read_rate(
            &["cached_input_per_1m_usd", "cached_per_1m_usd"],
            input * 0.25,
        )?;
        Ok(Self {
            input_per_1m_usd: input,
            output_per_1m_usd: output,
            cached_input_per_1m_usd: cached,
        })
    }
}

pub fn builtin_model_pricing() -> BTreeMap<String, ModelPrice> {
    let entries: &[(&str, f64, f64, f64)] = &[
        ("deepseek-v4.1-flash", 0.14, 0.28, 0.035),
        ("deepseek-v4.1-flash-free", 0.14, 0.28, 0.035),
        ("deepseek-chat", 0.27, 1.10, 0.07),
        ("deepseek-reasoner", 0.55, 2.19, 0.14),
        ("gpt-4o", 2.50, 10.00, 1.25),
        ("gpt-4o-mini", 0.15, 0.60, 0.075),
        ("gpt-4.1", 2.00, 8.00, 0.50),
        ("gpt-4.1-mini", 0.40, 1.60, 0.10),
        ("gpt-4.1-nano", 0.10, 0.40, 0.025),
        ("o3-mini", 1.10, 4.40, 0.55),
        ("o4-mini", 1.10, 4.40, 0.275),
        ("claude-3-5-sonnet", 3.00, 15.00, 0.30),
        ("claude-3-7-sonnet", 3.00, 15.00, 0.30),
        ("claude-sonnet-4", 3.00, 15.00, 0.30),
        ("claude-opus-4", 15.00, 75.00, 1.50),
        ("claude-haiku-3-5", 0.80, 4.00, 0.08),
        ("gemini-2.5-pro", 1.25, 10.00, 0.3125),
        ("gemini-2.5-flash", 0.15, 0.60, 0.0375),
        ("gemini-3.8-flash-high", 0.30, 1.20, 0.075),
        ("default", 0.50, 1.50, 0.125),
    ];
    let mut map = BTreeMap::new();
    for (id, i, o, c) in entries {
        map.insert(
            (*id).to_string(),
            ModelPrice {
                input_per_1m_usd: *i,
                output_per_1m_usd: *o,
                cached_input_per_1m_usd: *c,
            },
        );
    }
    map
}

pub fn model_pricing_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("model_pricing.json")
}

pub fn load_model_pricing_overrides(root_data_dir: &Path) -> BTreeMap<String, ModelPrice> {
    let path = model_pricing_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return BTreeMap::new();
    };
    let obj = v["models"]
        .as_object()
        .or_else(|| v.as_object());
    let Some(map_obj) = obj else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (k, val) in map_obj {
        let model = k.trim();
        if model.is_empty() {
            continue;
        }
        if let Ok(price) = ModelPrice::from_json(val) {
            out.insert(model.to_string(), price);
        }
    }
    out
}

pub fn load_effective_model_pricing(root_data_dir: &Path) -> BTreeMap<String, ModelPrice> {
    let mut map = builtin_model_pricing();
    for (k, v) in load_model_pricing_overrides(root_data_dir) {
        map.insert(k, v);
    }
    map
}

pub fn effective_model_pricing_json(root_data_dir: &Path) -> Value {
    let map = load_effective_model_pricing(root_data_dir);
    let mut obj = serde_json::Map::new();
    for (k, v) in map {
        obj.insert(k, v.to_json());
    }
    Value::Object(obj)
}

pub fn save_model_pricing_overrides(
    root_data_dir: &Path,
    overrides_val: &Value,
) -> Result<BTreeMap<String, ModelPrice>, String> {
    let raw_obj = overrides_val
        .get("models")
        .and_then(|v| v.as_object())
        .or_else(|| overrides_val.as_object())
        .ok_or_else(|| "model_pricing must be a JSON object mapping model_id -> rates".to_string())?;
    let mut overrides = load_model_pricing_overrides(root_data_dir);
    for (k, val) in raw_obj {
        let key = clean_single_line(k, 128);
        if key.is_empty() {
            return Err("model_pricing key cannot be empty".to_string());
        }
        if val.is_null() {
            overrides.remove(&key);
            continue;
        }
        let price = ModelPrice::from_json(val)?;
        overrides.insert(key, price);
    }
    let mut models_json = serde_json::Map::new();
    for (k, v) in &overrides {
        models_json.insert(k.clone(), v.to_json());
    }
    let body = json!({ "models": models_json }).to_string();
    atomic_write_bytes_with_mode(&model_pricing_path(root_data_dir), body.as_bytes(), Some(0o600))
        .map_err(|e| format!("save model_pricing.json: {e}"))?;
    Ok(load_effective_model_pricing(root_data_dir))
}

pub fn resolve_model_price(pricing: &BTreeMap<String, ModelPrice>, model_id: &str) -> ModelPrice {
    let trimmed = model_id.trim();
    if let Some(p) = pricing.get(trimmed) {
        return *p;
    }
    let low = trimmed.to_ascii_lowercase();
    if let Some((_, p)) = pricing
        .iter()
        .find(|(k, _)| k.to_ascii_lowercase() == low)
    {
        return *p;
    }
    if let Some((_, p)) = pricing
        .iter()
        .filter(|(k, _)| *k != "default" && low.starts_with(&k.to_ascii_lowercase()))
        .max_by_key(|(k, _)| k.len())
    {
        return *p;
    }
    pricing.get("default").copied().unwrap_or(ModelPrice {
        input_per_1m_usd: 0.50,
        output_per_1m_usd: 1.50,
        cached_input_per_1m_usd: 0.125,
    })
}

pub fn compute_call_cost_usd(
    price: &ModelPrice,
    prompt_tokens: u64,
    completion_tokens: u64,
    cached_tokens: u64,
) -> f64 {
    let cached = cached_tokens.min(prompt_tokens);
    let uncached = prompt_tokens.saturating_sub(cached);
    let cost = (uncached as f64 * price.input_per_1m_usd
        + cached as f64 * price.cached_input_per_1m_usd
        + completion_tokens as f64 * price.output_per_1m_usd)
        / 1_000_000.0;
    round_usd_8(cost.max(0.0))
}

pub fn compute_minutes_saved(
    finished: bool,
    answered: bool,
    ok_steps: usize,
    files_written: usize,
    completion_tokens: u64,
) -> f64 {
    let raw = if !finished {
        if ok_steps == 0 && files_written == 0 {
            0.0
        } else {
            (ok_steps as f64) * 2.0 + (files_written as f64) * 4.0
        }
    } else if answered && ok_steps == 0 && files_written == 0 {
        2.0 + (completion_tokens as f64 / 250.0).min(10.0)
    } else {
        3.0 + (ok_steps as f64) * 4.0
            + (files_written as f64) * 8.0
            + (completion_tokens as f64 / 250.0).min(15.0)
    };
    round_2(raw.max(0.0))
}

pub fn compute_billable_value_usd(minutes_saved: f64, hourly_rate_usd: f64) -> f64 {
    if minutes_saved <= 0.0 || hourly_rate_usd <= 0.0 || !hourly_rate_usd.is_finite() {
        return 0.0;
    }
    round_4((minutes_saved / 60.0) * hourly_rate_usd)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunUsage {
    pub model_id: String,
    pub calls: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cached_tokens: u64,
    pub latency_ms: u64,
    pub cost_usd: f64,
    pub estimated_minutes_saved: f64,
    pub billable_value_usd: f64,
}

impl RunUsage {
    #[allow(clippy::too_many_arguments)]
    pub fn add_call(
        &mut self,
        model_id: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
        cached_tokens: u64,
        latency_ms: u64,
        cost_usd: f64,
    ) {
        if !model_id.trim().is_empty() {
            self.model_id = model_id.trim().to_string();
        }
        self.calls = self.calls.saturating_add(1);
        self.prompt_tokens = self.prompt_tokens.saturating_add(prompt_tokens);
        self.completion_tokens = self.completion_tokens.saturating_add(completion_tokens);
        let eff_total = if total_tokens > 0 {
            total_tokens
        } else {
            prompt_tokens.saturating_add(completion_tokens)
        };
        self.total_tokens = self.total_tokens.saturating_add(eff_total);
        self.cached_tokens = self.cached_tokens.saturating_add(cached_tokens);
        self.latency_ms = self.latency_ms.saturating_add(latency_ms);
        self.cost_usd = round_usd_8(self.cost_usd + cost_usd.max(0.0));
    }

    pub fn accumulate(&mut self, other: &RunUsage) {
        if !other.model_id.trim().is_empty() {
            self.model_id = other.model_id.clone();
        }
        self.calls = self.calls.saturating_add(other.calls);
        self.prompt_tokens = self.prompt_tokens.saturating_add(other.prompt_tokens);
        self.completion_tokens = self.completion_tokens.saturating_add(other.completion_tokens);
        self.total_tokens = self.total_tokens.saturating_add(other.total_tokens);
        self.cached_tokens = self.cached_tokens.saturating_add(other.cached_tokens);
        self.latency_ms = self.latency_ms.saturating_add(other.latency_ms);
        self.cost_usd = round_usd_8(self.cost_usd + other.cost_usd);
        self.estimated_minutes_saved =
            round_2(self.estimated_minutes_saved + other.estimated_minutes_saved);
        self.billable_value_usd = round_4(self.billable_value_usd + other.billable_value_usd);
    }

    pub fn finalize_attribution(
        &mut self,
        finished: bool,
        answered: bool,
        ok_steps: usize,
        files_written: usize,
        hourly_rate_usd: f64,
    ) {
        self.estimated_minutes_saved = compute_minutes_saved(
            finished,
            answered,
            ok_steps,
            files_written,
            self.completion_tokens,
        );
        self.billable_value_usd =
            compute_billable_value_usd(self.estimated_minutes_saved, hourly_rate_usd);
    }

    pub fn to_json(&self) -> Value {
        json!({
            "model_id": self.model_id,
            "calls": self.calls,
            "prompt_tokens": self.prompt_tokens,
            "completion_tokens": self.completion_tokens,
            "total_tokens": self.total_tokens,
            "cached_tokens": self.cached_tokens,
            "latency_ms": self.latency_ms,
            "cost_usd": round_usd_8(self.cost_usd),
            "estimated_minutes_saved": round_2(self.estimated_minutes_saved),
            "billable_value_usd": round_4(self.billable_value_usd),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let prompt_tokens = obj.get("prompt_tokens").and_then(|x| x.as_u64()).unwrap_or(0);
        let completion_tokens = obj
            .get("completion_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let total_tokens = obj
            .get("total_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or_else(|| prompt_tokens.saturating_add(completion_tokens));
        Some(Self {
            model_id: obj
                .get("model_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            calls: obj.get("calls").and_then(|x| x.as_u64()).unwrap_or(0),
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cached_tokens: obj
                .get("cached_tokens")
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
            latency_ms: obj.get("latency_ms").and_then(|x| x.as_u64()).unwrap_or(0),
            cost_usd: obj
                .get("cost_usd")
                .and_then(|x| x.as_f64())
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            estimated_minutes_saved: obj
                .get("estimated_minutes_saved")
                .and_then(|x| x.as_f64())
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
            billable_value_usd: obj
                .get("billable_value_usd")
                .and_then(|x| x.as_f64())
                .filter(|n| n.is_finite() && *n >= 0.0)
                .unwrap_or(0.0),
        })
    }
}

#[derive(Clone, Debug)]
pub struct UsageLedgerEntry {
    pub ts_ms: u64,
    pub run_id: String,
    pub thread_id: String,
    pub person_id: String,
    pub engagement_id: Option<String>,
    pub client_id: Option<String>,
    pub source: String,
    pub finished: bool,
    pub answered: bool,
    pub usage: RunUsage,
}

impl UsageLedgerEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "ts_ms": self.ts_ms,
            "run_id": self.run_id,
            "thread_id": self.thread_id,
            "person_id": self.person_id,
            "engagement_id": self.engagement_id,
            "client_id": self.client_id,
            "source": self.source,
            "finished": self.finished,
            "answered": self.answered,
            "usage": self.usage.to_json(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let usage = obj
            .get("usage")
            .and_then(RunUsage::from_json)
            .or_else(|| RunUsage::from_json(v))?;
        Some(Self {
            ts_ms: obj
                .get("ts_ms")
                .or_else(|| obj.get("ts"))
                .and_then(|x| x.as_u64())
                .unwrap_or(0),
            run_id: obj
                .get("run_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            thread_id: obj
                .get("thread_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            person_id: obj
                .get("person_id")
                .and_then(|x| x.as_str())
                .unwrap_or("default")
                .to_string(),
            engagement_id: obj
                .get("engagement_id")
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            client_id: obj
                .get("client_id")
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            source: obj
                .get("source")
                .and_then(|x| x.as_str())
                .unwrap_or("agent")
                .to_string(),
            finished: obj
                .get("finished")
                .and_then(|x| x.as_bool())
                .unwrap_or(true),
            answered: obj
                .get("answered")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            usage,
        })
    }
}

pub fn usage_ledger_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("usage_ledger.jsonl")
}

pub fn append_usage_ledger(person_data_dir: &Path, entry: &UsageLedgerEntry) {
    with_records_lock(|| {
        let path = usage_ledger_path(person_data_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let line = format!("{}\n", entry.to_json());
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
            }
            let _ = f.write_all(line.as_bytes());
        }
    });
}

pub fn load_usage_ledger(person_data_dir: &Path) -> Vec<UsageLedgerEntry> {
    let path = usage_ledger_path(person_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| UsageLedgerEntry::from_json(&v))
        .collect()
}

pub fn engagement_spent_usd(
    root_data_dir: &Path,
    person_data_dir: &Path,
    engagement_id: &str,
) -> f64 {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut push_unique = |p: PathBuf| {
        if !dirs.iter().any(|existing| existing == &p) {
            dirs.push(p);
        }
    };
    push_unique(person_data_dir.to_path_buf());
    push_unique(root_data_dir.to_path_buf());
    let people_dir = root_data_dir.join("people");
    if let Ok(rd) = std::fs::read_dir(&people_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                push_unique(p);
            }
        }
    }
    let mut total = 0.0_f64;
    for dir in dirs {
        for e in load_usage_ledger(&dir) {
            if e.engagement_id.as_deref() == Some(engagement_id) {
                total += e.usage.cost_usd;
            }
        }
    }
    round_usd_8(total)
}

pub fn find_pending_budget_gate(
    person_data_dir: &Path,
    engagement_id: &str,
    thread_id: &str,
) -> Option<String> {
    load_decisions(person_data_dir).into_iter().find_map(|d| {
        if d["status"].as_str() == Some("pending")
            && d["kind"].as_str() == Some("budget_gate")
            && d["engagement_id"].as_str() == Some(engagement_id)
            && d["thread_id"].as_str() == Some(thread_id)
        {
            d["id"].as_str().map(str::to_string)
        } else {
            None
        }
    })
}

pub fn consume_approved_budget_gate(
    person_data_dir: &Path,
    engagement_id: &str,
) -> Option<String> {
    with_records_lock(|| {
        let path = decisions_path(person_data_dir);
        let mut items = load_decisions(person_data_dir);
        let idx = items.iter().position(|d| {
            d["status"].as_str() == Some("approved")
                && d["kind"].as_str() == Some("budget_gate")
                && d["engagement_id"].as_str() == Some(engagement_id)
                && d["budget_gate_consumed"].as_bool() != Some(true)
        })?;
        items[idx]["budget_gate_consumed"] = json!(true);
        items[idx]["budget_gate_consumed_ts"] = json!(now_millis());
        let id = items[idx]["id"].as_str().map(str::to_string)?;
        save_array(&path, "decisions", &items);
        Some(id)
    })
}

pub fn compute_metering_summary(
    root_data_dir: &Path,
    person_data_dir: &Path,
    person_id: &str,
    engagement_filter: Option<&str>,
    client_filter: Option<&str>,
    window_days: u64,
) -> Value {
    let store = load_clients_store(root_data_dir);
    let eng_map: HashMap<&str, &EngagementRecord> = store
        .engagements
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect();
    let client_map: HashMap<&str, &ClientRecord> =
        store.clients.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut entries = load_usage_ledger(person_data_dir);
    if entries.is_empty() && !usage_ledger_path(person_data_dir).exists() {
        for w in load_work(person_data_dir) {
            if let Some(usage) = w.get("usage").and_then(RunUsage::from_json) {
                let eid = w["engagement_id"]
                    .as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let cid = eid
                    .as_deref()
                    .and_then(|id| eng_map.get(id).map(|e| e.client_id.clone()));
                entries.push(UsageLedgerEntry {
                    ts_ms: w["ts"].as_u64().unwrap_or(0),
                    run_id: String::new(),
                    thread_id: w["thread_id"].as_str().unwrap_or("").to_string(),
                    person_id: person_id.to_string(),
                    engagement_id: eid,
                    client_id: cid,
                    source: "work".to_string(),
                    finished: w["finished"].as_bool().unwrap_or(true),
                    answered: w["answered"].as_bool().unwrap_or(false),
                    usage,
                });
            }
        }
    }

    let now_ms = now_millis();
    let clamped_days = window_days.clamp(1, 3650);
    let cutoff_ms = now_ms.saturating_sub(clamped_days.saturating_mul(86_400_000));
    let eng_f = engagement_filter.map(str::trim).filter(|s| !s.is_empty());
    let cli_f = client_filter.map(str::trim).filter(|s| !s.is_empty());

    let mut runs: u64 = 0;
    let mut prompt_tokens: u64 = 0;
    let mut completion_tokens: u64 = 0;
    let mut total_tokens: u64 = 0;
    let mut cached_tokens: u64 = 0;
    let mut cost_usd: f64 = 0.0;
    let mut minutes_saved: f64 = 0.0;
    let mut billable_value_usd: f64 = 0.0;

    #[derive(Default)]
    struct EngAgg {
        runs: u64,
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
        cost_usd: f64,
        minutes_saved: f64,
        billable_value_usd: f64,
    }
    #[derive(Default)]
    struct ModelAgg {
        runs: u64,
        calls: u64,
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
        cached_tokens: u64,
        cost_usd: f64,
    }
    #[derive(Default)]
    struct DayAgg {
        runs: u64,
        prompt_tokens: u64,
        completion_tokens: u64,
        total_tokens: u64,
        cost_usd: f64,
        minutes_saved: f64,
        billable_value_usd: f64,
    }

    let mut by_eng: BTreeMap<String, EngAgg> = BTreeMap::new();
    let mut by_mod: BTreeMap<String, ModelAgg> = BTreeMap::new();
    let mut by_day: BTreeMap<String, DayAgg> = BTreeMap::new();

    for e in entries {
        if e.ts_ms < cutoff_ms {
            continue;
        }
        let eid = e.engagement_id.as_deref().unwrap_or("");
        let resolved_cid = e
            .client_id
            .as_deref()
            .or_else(|| eng_map.get(eid).map(|eng| eng.client_id.as_str()))
            .unwrap_or("");
        if let Some(f) = eng_f {
            if eid != f {
                continue;
            }
        }
        if let Some(f) = cli_f {
            if resolved_cid != f {
                continue;
            }
        }

        runs = runs.saturating_add(1);
        prompt_tokens = prompt_tokens.saturating_add(e.usage.prompt_tokens);
        completion_tokens = completion_tokens.saturating_add(e.usage.completion_tokens);
        total_tokens = total_tokens.saturating_add(e.usage.total_tokens);
        cached_tokens = cached_tokens.saturating_add(e.usage.cached_tokens);
        cost_usd += e.usage.cost_usd;
        minutes_saved += e.usage.estimated_minutes_saved;
        billable_value_usd += e.usage.billable_value_usd;

        let ea = by_eng.entry(eid.to_string()).or_default();
        ea.runs += 1;
        ea.prompt_tokens += e.usage.prompt_tokens;
        ea.completion_tokens += e.usage.completion_tokens;
        ea.total_tokens += e.usage.total_tokens;
        ea.cost_usd += e.usage.cost_usd;
        ea.minutes_saved += e.usage.estimated_minutes_saved;
        ea.billable_value_usd += e.usage.billable_value_usd;

        let model_key = if e.usage.model_id.trim().is_empty() {
            "none".to_string()
        } else {
            e.usage.model_id.clone()
        };
        let ma = by_mod.entry(model_key).or_default();
        ma.runs += 1;
        ma.calls += e.usage.calls;
        ma.prompt_tokens += e.usage.prompt_tokens;
        ma.completion_tokens += e.usage.completion_tokens;
        ma.total_tokens += e.usage.total_tokens;
        ma.cached_tokens += e.usage.cached_tokens;
        ma.cost_usd += e.usage.cost_usd;

        let day_key = date_string_local(e.ts_ms);
        let da = by_day.entry(day_key).or_default();
        da.runs += 1;
        da.prompt_tokens += e.usage.prompt_tokens;
        da.completion_tokens += e.usage.completion_tokens;
        da.total_tokens += e.usage.total_tokens;
        da.cost_usd += e.usage.cost_usd;
        da.minutes_saved += e.usage.estimated_minutes_saved;
        da.billable_value_usd += e.usage.billable_value_usd;
    }

    let retainer_burn_pct = if let Some(eid) = eng_f {
        eng_map
            .get(eid)
            .filter(|e| e.retainer_budget_usd > 0.0)
            .map(|e| round_2((cost_usd / e.retainer_budget_usd) * 100.0))
            .unwrap_or(0.0)
    } else if let Some(cid) = cli_f {
        let budget: f64 = store
            .engagements
            .iter()
            .filter(|e| e.client_id == cid && e.retainer_budget_usd > 0.0)
            .map(|e| e.retainer_budget_usd)
            .sum();
        if budget > 0.0 {
            round_2((cost_usd / budget) * 100.0)
        } else {
            0.0
        }
    } else if let Some(active_eid) = store.active_engagement_by_person.get(person_id) {
        if let Some(eng) = eng_map.get(active_eid.as_str()) {
            if eng.retainer_budget_usd > 0.0 {
                let spent = by_eng
                    .get(active_eid.as_str())
                    .map(|a| a.cost_usd)
                    .unwrap_or(0.0);
                round_2((spent / eng.retainer_budget_usd) * 100.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    let mut by_engagement: Vec<Value> = by_eng
        .into_iter()
        .map(|(eid, agg)| {
            let eng = eng_map.get(eid.as_str()).copied();
            let cid = eng.map(|e| e.client_id.as_str()).unwrap_or("");
            let cname = client_map.get(cid).map(|c| c.name.as_str()).unwrap_or("");
            let budget = eng.map(|e| e.retainer_budget_usd).unwrap_or(0.0);
            let burn = if budget > 0.0 {
                round_2((agg.cost_usd / budget) * 100.0)
            } else {
                0.0
            };
            json!({
                "engagement_id": eid,
                "engagement_name": eng.map(|e| e.name.as_str()).unwrap_or("Unassigned"),
                "client_id": cid,
                "client_name": cname,
                "runs": agg.runs,
                "prompt_tokens": agg.prompt_tokens,
                "completion_tokens": agg.completion_tokens,
                "total_tokens": agg.total_tokens,
                "cost_usd": round_usd_8(agg.cost_usd),
                "minutes_saved": round_2(agg.minutes_saved),
                "billable_value_usd": round_4(agg.billable_value_usd),
                "retainer_budget_usd": round_4(budget),
                "retainer_hours": round_4(eng.map(|e| e.retainer_hours).unwrap_or(0.0)),
                "retainer_burn_pct": burn,
            })
        })
        .collect();
    by_engagement.sort_by(|a, b| {
        b["cost_usd"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["cost_usd"].as_f64().unwrap_or(0.0))
            .then_with(|| {
                a["engagement_id"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["engagement_id"].as_str().unwrap_or(""))
            })
    });

    let mut by_model: Vec<Value> = by_mod
        .into_iter()
        .map(|(mid, agg)| {
            json!({
                "model_id": mid,
                "runs": agg.runs,
                "calls": agg.calls,
                "prompt_tokens": agg.prompt_tokens,
                "completion_tokens": agg.completion_tokens,
                "total_tokens": agg.total_tokens,
                "cached_tokens": agg.cached_tokens,
                "cost_usd": round_usd_8(agg.cost_usd),
            })
        })
        .collect();
    by_model.sort_by(|a, b| {
        b["cost_usd"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["cost_usd"].as_f64().unwrap_or(0.0))
            .then_with(|| {
                a["model_id"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["model_id"].as_str().unwrap_or(""))
            })
    });

    let daily_series: Vec<Value> = by_day
        .into_iter()
        .map(|(date, agg)| {
            json!({
                "date": date,
                "runs": agg.runs,
                "prompt_tokens": agg.prompt_tokens,
                "completion_tokens": agg.completion_tokens,
                "total_tokens": agg.total_tokens,
                "cost_usd": round_usd_8(agg.cost_usd),
                "minutes_saved": round_2(agg.minutes_saved),
                "billable_value_usd": round_4(agg.billable_value_usd),
            })
        })
        .collect();

    json!({
        "window_days": clamped_days,
        "engagement_id": eng_f,
        "client_id": cli_f,
        "totals": {
            "runs": runs,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": total_tokens,
            "cached_tokens": cached_tokens,
            "cost_usd": round_usd_8(cost_usd),
            "minutes_saved": round_2(minutes_saved),
            "billable_value_usd": round_4(billable_value_usd),
            "retainer_burn_pct": retainer_burn_pct,
        },
        "by_engagement": by_engagement,
        "by_model": by_model,
        "daily_series": daily_series,
    })
}
