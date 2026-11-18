//! The day-to-day records: the work log, the decisions inbox and the
//! offers inbox, plus the brief / today / graph assemblies built from
//! them. All per-person, all plain JSON files in the person's data
//! dir, all written by the app itself (never by the UI layer faking
//! state the agent did not produce).

use crate::library::{Library, Suite};
use crate::{sandbox_path, truncate, Config};
use serde_json::{json, Value};
use std::path::Path;

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

pub fn append_work(cfg: &Config, task: &str, summary: &str, files: &[String], thread_id: &str) {
    let path = work_path(&cfg.data_dir);
    let mut items = load_array(&path, "entries");
    items.push(json!({
        "ts": now_millis(),
        "task": task,
        "summary": summary,
        "files": files,
        "thread_id": thread_id,
    }));
    save_array(&path, "entries", &items);
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

pub fn add_decision(
    person_data_dir: &Path,
    kind: &str,
    summary: &str,
    path: Option<&str>,
    thread_id: &str,
) -> String {
    let mut items = load_decisions(person_data_dir);
    let ts = now_millis();
    let id = format!("d{ts}-{}", items.len());
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
            Some(rel) => match sandbox_path(&cfg.workspace_dir, rel) {
                Ok(full) => {
                    if full.symlink_metadata().is_ok() {
                        match std::fs::remove_file(&full) {
                            Ok(()) => format!("Approved and done: {rel} was deleted."),
                            Err(e) => format!("Approved, but deleting {rel} failed: {e}"),
                        }
                    } else {
                        format!("Approved: {rel} was already gone, so nothing needed deleting.")
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

/// A stable pattern key for a task: its first three significant words.
fn pattern_key(task: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for w in task.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if w.is_empty() || STOPWORDS.contains(&w) {
            continue;
        }
        words.push(w.to_string());
        if words.len() == 3 {
            break;
        }
    }
    words.join(" ")
}

/// Scan the work log and create any offers that are newly earned.
/// An offer key that already exists in ANY state (pending, accepted,
/// dismissed) is never offered again — a no is a no.
pub fn refresh_offers(cfg: &Config, suite: &Suite, library: &Library) -> Vec<Value> {
    let work = load_work(&cfg.data_dir);
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
        let key = pattern_key(entry["task"].as_str().unwrap_or(""));
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
                .last()
                .and_then(|e| e["task"].as_str())
                .unwrap_or("")
                .to_string();
            offers.push(json!({
                "id": format!("o{ts}-{}", offers.len()),
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
            "id": format!("o{ts}-{}", offers.len()),
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
}

pub fn set_offer_status(person_data_dir: &Path, id: &str, status: &str) -> Result<Value, String> {
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
}

// ---------------------------------------------------------------------------
// Brief, today, graph — assemblies over the real stores.
// ---------------------------------------------------------------------------

/// The morning brief: what the agent finished recently, what is
/// waiting on the person, and what it learned. Assembled from the
/// stores; empty sections say so plainly rather than being padded.
pub fn build_brief(cfg: &Config, person_name: &str, suite: &Suite, library: &Library) -> Value {
    let now = now_millis();
    let today = date_string(now);
    let work = load_work(&cfg.data_dir);
    let recent: Vec<&Value> = work
        .iter()
        .filter(|e| now.saturating_sub(e["ts"].as_u64().unwrap_or(0)) <= 48 * 3_600_000)
        .collect();
    let finished: Vec<Value> = recent
        .iter()
        .rev()
        .take(8)
        .map(|e| {
            json!({
                "date": date_string(e["ts"].as_u64().unwrap_or(0)),
                "task": e["task"],
                "summary": e["summary"],
                "files": e["files"],
            })
        })
        .collect();
    let decisions = load_decisions(&cfg.data_dir);
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
        "pending_decisions": pending_decisions,
        "pending_offers": pending_offers,
        "learned_recently": learned,
        "suite_size": suite.installed.len(),
        "library_size": library.items.len(),
    })
}

/// Today's stream for the Today River face: work finished, decisions
/// raised and settled, offers made — one chronological list.
pub fn build_today(cfg: &Config) -> Value {
    let now = now_millis();
    let today = date_string(now);
    let mut events: Vec<Value> = Vec::new();
    for e in load_work(&cfg.data_dir) {
        let ts = e["ts"].as_u64().unwrap_or(0);
        if date_string(ts) == today {
            events.push(json!({
                "ts": ts, "kind": "work",
                "title": e["task"], "detail": e["summary"], "files": e["files"],
            }));
        }
    }
    for d in load_decisions(&cfg.data_dir) {
        let ts = d["ts"].as_u64().unwrap_or(0);
        if date_string(ts) == today {
            events.push(json!({
                "ts": ts, "kind": "decision",
                "title": d["summary"], "detail": format!("Waiting on you ({})", d["kind"]),
            }));
        }
        if let Some(rts) = d["resolved_ts"].as_u64() {
            if date_string(rts) == today {
                events.push(json!({
                    "ts": rts, "kind": "decision_settled",
                    "title": d["summary"], "detail": d["outcome"],
                }));
            }
        }
    }
    for o in load_offers(&cfg.data_dir) {
        let ts = o["ts"].as_u64().unwrap_or(0);
        if date_string(ts) == today {
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
            if !nodes.iter().any(|n| n["id"] == json!(iid)) {
                if let Some(item) = library.get(item_id) {
                    nodes.push(json!({
                        "id": iid, "kind": "item",
                        "label": item.name, "detail": item.kind,
                    }));
                }
            }
            edges.push(json!({"from": pid, "to": iid, "kind": "carries"}));
        }
        let work = load_work(&pcfg.data_dir);
        for (n, e) in work.iter().rev().take(12).enumerate() {
            let wid = format!("work:{}:{}", person.id, n);
            nodes.push(json!({
                "id": wid, "kind": "work",
                "label": truncate(e["task"].as_str().unwrap_or(""), 60),
                "detail": date_string(e["ts"].as_u64().unwrap_or(0)),
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
