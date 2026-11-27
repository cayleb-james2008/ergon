//! Per-task quality bar and "better than last time, until good enough" loop.
//!
//! Stores per-person quality bars under `<person_data_dir>/quality_bars.json`
//! with `0600` POSIX permissions, serialized through `QUALITY_LOCK`.

use crate::records::now_millis;
use crate::slugify;
use crate::tools::{atomic_write_bytes_with_mode, sandbox_path};
use crate::tripwire::sha256_hex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static QUALITY_LOCK: Mutex<()> = Mutex::new(());

const FAMILY_STOPWORDS: [&str; 20] = [
    "the", "a", "an", "my", "please", "into", "and", "of", "to", "for", "me", "in", "on", "with",
    "from", "this", "write", "create", "make", "draft",
];

const CRITERIA_STOPWORDS: [&str; 19] = [
    "the", "a", "an", "and", "or", "of", "to", "for", "in", "on", "with", "from", "include",
    "including", "add", "must", "should", "need", "needs",
];

pub const STATUS_IMPROVING: &str = "improving";
pub const STATUS_ACCEPTED: &str = "accepted_good_enough";

fn with_quality_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = QUALITY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn quality_bars_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("quality_bars.json")
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualityRunRecord {
    pub run_id: String,
    pub ts_ms: u64,
    pub task: String,
    pub summary: String,
    pub files: Vec<String>,
    pub score: f64,
    pub delta_vs_prev: f64,
    pub criteria_met: usize,
    pub criteria_total: usize,
    pub feedback_action: Option<String>,
    pub feedback_note: Option<String>,
}

impl QualityRunRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "run_id": self.run_id,
            "ts_ms": self.ts_ms,
            "task": self.task,
            "summary": self.summary,
            "files": self.files,
            "score": self.score,
            "delta_vs_prev": self.delta_vs_prev,
            "criteria_met": self.criteria_met,
            "criteria_total": self.criteria_total,
            "feedback_action": self.feedback_action,
            "feedback_note": self.feedback_note,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            run_id: v["run_id"].as_str()?.to_string(),
            ts_ms: v["ts_ms"].as_u64().unwrap_or(0),
            task: v["task"].as_str().unwrap_or("").to_string(),
            summary: v["summary"].as_str().unwrap_or("").to_string(),
            files: string_array(&v["files"]),
            score: v["score"].as_f64().unwrap_or(0.0),
            delta_vs_prev: v["delta_vs_prev"].as_f64().unwrap_or(0.0),
            criteria_met: v["criteria_met"].as_u64().unwrap_or(0) as usize,
            criteria_total: v["criteria_total"].as_u64().unwrap_or(0) as usize,
            feedback_action: v["feedback_action"].as_str().map(str::to_string),
            feedback_note: v["feedback_note"].as_str().map(str::to_string),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualityBar {
    pub task_family: String,
    pub label: String,
    pub status: String,
    pub criteria: Vec<String>,
    pub runs: Vec<QualityRunRecord>,
    pub best_run_id: Option<String>,
    pub best_score: f64,
    pub accepted_run_id: Option<String>,
    pub accepted_digest: Option<String>,
    pub accepted_recipe: Option<String>,
    pub accepted_ts_ms: Option<u64>,
    pub updated_ms: u64,
}

impl QualityBar {
    pub fn to_json(&self) -> Value {
        json!({
            "task_family": self.task_family,
            "label": self.label,
            "status": self.status,
            "criteria": self.criteria,
            "runs": self.runs.iter().map(QualityRunRecord::to_json).collect::<Vec<_>>(),
            "best_run_id": self.best_run_id,
            "best_score": self.best_score,
            "accepted_run_id": self.accepted_run_id,
            "accepted_digest": self.accepted_digest,
            "accepted_recipe": self.accepted_recipe,
            "accepted_ts_ms": self.accepted_ts_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let task_family = v["task_family"].as_str()?.trim().to_string();
        if task_family.is_empty() {
            return None;
        }
        let status = match v["status"].as_str().unwrap_or(STATUS_IMPROVING) {
            STATUS_ACCEPTED => STATUS_ACCEPTED.to_string(),
            _ => STATUS_IMPROVING.to_string(),
        };
        let runs = v["runs"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(QualityRunRecord::from_json)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Some(Self {
            label: v["label"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(&task_family)
                .to_string(),
            task_family,
            status,
            criteria: string_array(&v["criteria"]),
            runs,
            best_run_id: v["best_run_id"].as_str().map(str::to_string),
            best_score: v["best_score"].as_f64().unwrap_or(0.0),
            accepted_run_id: v["accepted_run_id"].as_str().map(str::to_string),
            accepted_digest: v["accepted_digest"].as_str().map(str::to_string),
            accepted_recipe: v["accepted_recipe"].as_str().map(str::to_string),
            accepted_ts_ms: v["accepted_ts_ms"].as_u64(),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

struct StoredBar {
    bar: QualityBar,
    run_digests: HashMap<String, String>,
}

impl StoredBar {
    fn to_json(&self) -> Value {
        let runs: Vec<Value> = self
            .bar
            .runs
            .iter()
            .map(|r| {
                let mut obj = r.to_json();
                if let Some(d) = self.run_digests.get(&r.run_id) {
                    obj["content_digest"] = Value::String(d.clone());
                }
                obj
            })
            .collect();
        let mut obj = self.bar.to_json();
        obj["runs"] = Value::Array(runs);
        obj
    }

    fn from_json(v: &Value) -> Option<Self> {
        let bar = QualityBar::from_json(v)?;
        let mut run_digests = HashMap::new();
        if let Some(arr) = v["runs"].as_array() {
            for item in arr {
                if let (Some(id), Some(digest)) =
                    (item["run_id"].as_str(), item["content_digest"].as_str())
                {
                    if !digest.is_empty() {
                        run_digests.insert(id.to_string(), digest.to_string());
                    }
                }
            }
        }
        Some(Self { bar, run_digests })
    }
}

fn string_array(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Normalize a free-form task description into a stable task-family key.
///
/// Strips common leading/connecting stopwords and keeps up to 3 significant
/// words (requiring at least 2 significant words, otherwise falling back to
/// `slugify`).
pub fn task_family_key(task: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for token in task.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if token.is_empty() || FAMILY_STOPWORDS.contains(&token) {
            continue;
        }
        words.push(token.to_string());
        if words.len() == 3 {
            break;
        }
    }
    if words.len() >= 2 {
        return words.join("-");
    }
    if !task.chars().any(|c| c.is_ascii_alphanumeric()) {
        return "general".to_string();
    }
    slugify(task)
}

fn normalize_explicit_family(raw: &str) -> String {
    if !raw.chars().any(|c| c.is_ascii_alphanumeric()) {
        return "general".to_string();
    }
    slugify(raw)
}

fn resolve_task_family(task: &str, explicit_family: Option<&str>) -> String {
    match explicit_family.map(str::trim).filter(|s| !s.is_empty()) {
        Some(fam) => normalize_explicit_family(fam),
        None => task_family_key(task),
    }
}

fn matches_family(stored: &str, requested: &str) -> bool {
    let trimmed = requested.trim();
    if trimmed.is_empty() {
        return false;
    }
    stored == trimmed || stored == normalize_explicit_family(trimmed)
}

fn load_stored_unlocked(person_data_dir: &Path) -> Vec<StoredBar> {
    let path = quality_bars_path(person_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let items = v["bars"]
        .as_array()
        .or_else(|| v.as_array())
        .cloned()
        .unwrap_or_default();
    items.iter().filter_map(StoredBar::from_json).collect()
}

fn save_stored_unlocked(person_data_dir: &Path, bars: &[StoredBar]) -> Result<(), String> {
    let payload = json!({
        "bars": bars.iter().map(StoredBar::to_json).collect::<Vec<_>>()
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize quality_bars.json: {e}"))?;
    atomic_write_bytes_with_mode(&quality_bars_path(person_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write quality_bars.json: {e}"))
}

pub fn load_quality_bars(person_data_dir: &Path) -> Vec<QualityBar> {
    with_quality_lock(|| {
        load_stored_unlocked(person_data_dir)
            .into_iter()
            .map(|s| s.bar)
            .collect()
    })
}

pub fn get_quality_bar(person_data_dir: &Path, task_family: &str) -> Option<QualityBar> {
    with_quality_lock(|| {
        load_stored_unlocked(person_data_dir)
            .into_iter()
            .map(|s| s.bar)
            .find(|b| matches_family(&b.task_family, task_family))
    })
}

pub fn prompt_guidance_for_task(
    person_data_dir: &Path,
    task: &str,
    explicit_family: Option<&str>,
) -> Option<String> {
    let family = resolve_task_family(task, explicit_family);
    let bar = get_quality_bar(person_data_dir, &family)?;

    if bar.status == STATUS_ACCEPTED {
        let recipe = bar
            .accepted_recipe
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Follow the accepted structure, tone, and deliverable format.");
        let criteria = if bar.criteria.is_empty() {
            "Preserve the accepted output format without drift.".to_string()
        } else {
            bar.criteria.join("; ")
        };
        return Some(format!(
            "QUALITY BAR (ACCEPTED — GOOD ENOUGH, DO NOT DRIFT): Prior output for '{}' was accepted as good enough (score {:.2}). Follow the accepted recipe/format and criteria below faithfully without unnecessary redesign or drift:\n- Accepted recipe: {}\n- Criteria: {}",
            bar.label, bar.best_score, recipe, criteria
        ));
    }

    if bar.status == STATUS_IMPROVING && !bar.runs.is_empty() {
        let last_score = bar.runs.last().map(|r| r.score).unwrap_or(0.0);
        let criteria = if bar.criteria.is_empty() {
            "None yet — produce clear structure, non-empty deliverables, and verifiable citations."
                .to_string()
        } else {
            bar.criteria.join("; ")
        };
        let feedback = bar
            .runs
            .iter()
            .rev()
            .find_map(|r| r.feedback_note.as_deref().map(str::trim).filter(|s| !s.is_empty()))
            .unwrap_or("None yet.");
        return Some(format!(
            "QUALITY BAR (BEAT LAST RUN): Previous run for '{}' scored {:.2} (best {:.2}). Aim to beat the last run by addressing learned criteria and feedback:\n- Criteria: {}\n- Recent feedback: {}",
            bar.label, last_score, bar.best_score, criteria, feedback
        ));
    }

    None
}

fn has_line_citation(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(close_rel) = text[i + 1..].find(']') {
                let inner = &text[i + 1..i + 1 + close_rel];
                if !inner.contains('\n') {
                    if let Some(pos) = inner.find(":L") {
                        let after = &inner[pos + 2..];
                        if after.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                            return true;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    false
}

fn has_structured_heading(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("# ")
            || trimmed.starts_with("## ")
            || trimmed.starts_with("### ")
            || trimmed.starts_with("#### ")
    })
}

fn criterion_matches(criterion: &str, corpus_lower: &str) -> bool {
    let c_lower = criterion.trim().to_lowercase();
    if c_lower.is_empty() {
        return true;
    }
    if corpus_lower.contains(&c_lower) {
        return true;
    }
    let all_tokens: Vec<&str> = c_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    if all_tokens.is_empty() {
        return false;
    }
    let mut keywords: Vec<&str> = all_tokens
        .iter()
        .copied()
        .filter(|t| !CRITERIA_STOPWORDS.contains(t))
        .collect();
    if keywords.is_empty() {
        keywords = all_tokens;
    }
    let matched = keywords.iter().filter(|k| corpus_lower.contains(**k)).count();
    let needed = if keywords.len() <= 2 {
        keywords.len()
    } else {
        (keywords.len() * 2).div_ceil(3)
    };
    matched >= needed
}

fn round_score(score: f64) -> f64 {
    let clamped = score.clamp(0.0, 1.0);
    (clamped * 10_000.0).round() / 10_000.0
}

fn read_written_contents(workspace_dir: &Path, written_files: &[String]) -> (usize, String) {
    let mut non_empty = 0usize;
    let mut combined = String::new();
    for rel in written_files {
        let Ok(path) = sandbox_path(workspace_dir, rel) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if bytes.iter().any(|b| !b.is_ascii_whitespace()) {
            non_empty += 1;
        }
        combined.push('\n');
        combined.push_str(rel);
        combined.push('\n');
        combined.push_str(&String::from_utf8_lossy(&bytes));
    }
    (non_empty, combined)
}

pub fn record_task_run(
    person_data_dir: &Path,
    workspace_dir: &Path,
    task: &str,
    explicit_family: Option<&str>,
    run_id: &str,
    summary: &str,
    written_files: &[String],
) -> Value {
    let family = resolve_task_family(task, explicit_family);
    let ts_ms = now_millis();
    let (non_empty_files, files_corpus) = read_written_contents(workspace_dir, written_files);

    let mut digest_input = String::with_capacity(summary.len() + files_corpus.len() + 16);
    digest_input.push_str(summary);
    digest_input.push_str(&files_corpus);
    let content_digest = sha256_hex(digest_input.as_bytes());
    let corpus_lower = digest_input.to_lowercase();

    let summary_score = if !summary.trim().is_empty() { 0.25 } else { 0.0 };
    let files_score = if written_files.is_empty() {
        0.0
    } else {
        0.25 * (non_empty_files as f64 / written_files.len() as f64)
    };
    let structure_score = if has_line_citation(&digest_input) || has_structured_heading(&digest_input)
    {
        0.15
    } else {
        0.0
    };

    with_quality_lock(|| {
        let mut bars = load_stored_unlocked(person_data_dir);
        let idx = match bars
            .iter()
            .position(|s| matches_family(&s.bar.task_family, &family))
        {
            Some(i) => i,
            None => {
                let label = if !task.trim().is_empty() {
                    task.trim().chars().take(120).collect::<String>()
                } else {
                    family.clone()
                };
                bars.push(StoredBar {
                    bar: QualityBar {
                        task_family: family.clone(),
                        label,
                        status: STATUS_IMPROVING.to_string(),
                        criteria: Vec::new(),
                        runs: Vec::new(),
                        best_run_id: None,
                        best_score: 0.0,
                        accepted_run_id: None,
                        accepted_digest: None,
                        accepted_recipe: None,
                        accepted_ts_ms: None,
                        updated_ms: ts_ms,
                    },
                    run_digests: HashMap::new(),
                });
                bars.len() - 1
            }
        };

        let stored = &mut bars[idx];
        let criteria_total = stored.bar.criteria.len();
        let (criteria_met, criteria_score) = if criteria_total == 0 {
            // Baseline 0.15 out of 0.35 so a clean first run scores ~0.65-0.80,
            // leaving deterministic headroom for criteria-driven Run 2 to beat Run 1.
            (0usize, 0.15)
        } else {
            let met = stored
                .bar
                .criteria
                .iter()
                .filter(|c| criterion_matches(c, &corpus_lower))
                .count();
            (met, 0.35 * (met as f64 / criteria_total as f64))
        };

        let score = round_score(summary_score + files_score + structure_score + criteria_score);
        let prev_score = stored.bar.runs.last().map(|r| r.score).unwrap_or(0.0);
        let delta_vs_prev = round_score_signed(score - prev_score);

        let is_accepted = stored.bar.status == STATUS_ACCEPTED;

        stored.run_digests.insert(run_id.to_string(), content_digest);
        stored.bar.runs.push(QualityRunRecord {
            run_id: run_id.to_string(),
            ts_ms,
            task: task.to_string(),
            summary: summary.to_string(),
            files: written_files.to_vec(),
            score,
            delta_vs_prev,
            criteria_met,
            criteria_total,
            feedback_action: None,
            feedback_note: None,
        });

        if !is_accepted {
            if stored.bar.best_run_id.is_none() || score >= stored.bar.best_score {
                stored.bar.best_score = score;
                stored.bar.best_run_id = Some(run_id.to_string());
            }
            stored.bar.updated_ms = ts_ms;
        }

        let status = stored.bar.status.clone();
        let best_score = stored.bar.best_score;
        let criteria = stored.bar.criteria.clone();
        let task_family = stored.bar.task_family.clone();

        let _ = save_stored_unlocked(person_data_dir, &bars);

        json!({
            "task_family": task_family,
            "status": status,
            "score": score,
            "prev_score": prev_score,
            "delta_vs_prev": delta_vs_prev,
            "best_score": best_score,
            "criteria": criteria,
            "stable_at_bar": is_accepted,
            "changes_stopped": is_accepted,
        })
    })
}

fn round_score_signed(delta: f64) -> f64 {
    (delta * 10_000.0).round() / 10_000.0
}

fn append_unique_criterion(criteria: &mut Vec<String>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    if !criteria.iter().any(|c| c.eq_ignore_ascii_case(trimmed)) {
        criteria.push(trimmed.to_string());
    }
}

pub fn apply_feedback(
    person_data_dir: &Path,
    task_family: &str,
    action: &str,
    note: Option<&str>,
    add_criteria: Vec<String>,
    recipe: Option<&str>,
) -> Result<QualityBar, String> {
    let normalized_action = action.trim().to_ascii_lowercase();
    let is_accept = normalized_action == "accept_good_enough" || normalized_action == STATUS_ACCEPTED;
    let is_improve = normalized_action == "improve";
    if !is_accept && !is_improve {
        return Err(format!(
            "unsupported feedback action '{action}': expected 'accept_good_enough' or 'improve'"
        ));
    }

    with_quality_lock(|| {
        let mut bars = load_stored_unlocked(person_data_dir);
        let idx = bars
            .iter()
            .position(|s| matches_family(&s.bar.task_family, task_family))
            .ok_or_else(|| format!("quality bar not found for task_family '{task_family}'"))?;

        let ts_ms = now_millis();
        let stored = &mut bars[idx];

        for c in &add_criteria {
            append_unique_criterion(&mut stored.bar.criteria, c);
        }

        let clean_note = note.map(str::trim).filter(|s| !s.is_empty());

        if is_accept {
            stored.bar.status = STATUS_ACCEPTED.to_string();
            let chosen_run_id = stored
                .bar
                .runs
                .last()
                .map(|r| r.run_id.clone())
                .or_else(|| stored.bar.best_run_id.clone());
            if let Some(ref rid) = chosen_run_id {
                if let Some(digest) = stored.run_digests.get(rid) {
                    stored.bar.accepted_digest = Some(digest.clone());
                }
            }
            stored.bar.accepted_run_id = chosen_run_id;
            stored.bar.accepted_ts_ms = Some(ts_ms);
            if let Some(r) = recipe.map(str::trim).filter(|s| !s.is_empty()) {
                stored.bar.accepted_recipe = Some(r.to_string());
            }
            if let Some(last) = stored.bar.runs.last_mut() {
                last.feedback_action = Some("accept_good_enough".to_string());
                if let Some(n) = clean_note {
                    last.feedback_note = Some(n.to_string());
                }
            }
        } else {
            stored.bar.status = STATUS_IMPROVING.to_string();
            if let Some(n) = clean_note {
                append_unique_criterion(&mut stored.bar.criteria, n);
            }
            if let Some(last) = stored.bar.runs.last_mut() {
                last.feedback_action = Some("improve".to_string());
                if let Some(n) = clean_note {
                    last.feedback_note = match last.feedback_note.take() {
                        Some(existing) if !existing.trim().is_empty() && !existing.eq_ignore_ascii_case(n) => {
                            Some(format!("{existing}; {n}"))
                        }
                        _ => Some(n.to_string()),
                    };
                }
            }
        }

        stored.bar.updated_ms = ts_ms;
        let updated = stored.bar.clone();
        save_stored_unlocked(person_data_dir, &bars)?;
        Ok(updated)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn make_temp_dir(label: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "ergon-quality-bar-test-{}-{}-{}",
            label,
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn task_family_key_groups_similar_tasks_and_strips_stopwords() {
        assert_eq!(
            task_family_key("weekly client brief for acme"),
            "weekly-client-brief"
        );
        assert_eq!(
            task_family_key("Please draft a weekly client brief for beta"),
            "weekly-client-brief"
        );
        assert_eq!(task_family_key("Summarize"), "summarize");
        assert_eq!(task_family_key("   "), "general");
    }

    #[test]
    fn quality_bar_improve_beat_accept_and_stop_drift() {
        let root = make_temp_dir("lifecycle");
        let data_dir = root.join("data");
        let ws_dir = root.join("ws");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::create_dir_all(&ws_dir).unwrap();

        let task = "weekly client brief for acme";
        assert!(prompt_guidance_for_task(&data_dir, task, None).is_none());

        // Run 1: clean initial output without learned criteria yet.
        std::fs::write(
            ws_dir.join("brief.md"),
            "# Weekly Client Brief\nProgress update for Acme.\n",
        )
        .unwrap();
        let r1 = record_task_run(
            &data_dir,
            &ws_dir,
            task,
            None,
            "run-1",
            "Drafted weekly client brief for Acme.",
            &["brief.md".to_string()],
        );
        assert_eq!(r1["task_family"], "weekly-client-brief");
        assert_eq!(r1["status"], "improving");
        assert_eq!(r1["stable_at_bar"], false);
        assert_eq!(r1["changes_stopped"], false);
        let s1 = r1["score"].as_f64().unwrap();
        assert!((0.65..=0.80).contains(&s1), "expected ~0.65-0.80, got {s1}");

        // Feedback: ask to improve and teach explicit criteria.
        let bar_after_fb = apply_feedback(
            &data_dir,
            "weekly-client-brief",
            "improve",
            Some("Include Risk Register and Owner Actions"),
            vec![
                "Risk Register".to_string(),
                "Owner Actions".to_string(),
            ],
            None,
        )
        .unwrap();
        assert_eq!(bar_after_fb.status, "improving");
        assert!(bar_after_fb.criteria.contains(&"Risk Register".to_string()));

        let guidance_improving = prompt_guidance_for_task(
            &data_dir,
            "weekly client brief for beta",
            None,
        )
        .expect("improving guidance should be present");
        assert!(guidance_improving.contains("QUALITY BAR (BEAT LAST RUN)"));
        assert!(guidance_improving.contains("Risk Register"));

        // Run 2: satisfies all learned criteria and includes a line citation.
        std::fs::write(
            ws_dir.join("brief.md"),
            "# Weekly Client Brief\n## Risk Register\n- Vendor delay [notes.md:L4]\n## Owner Actions\n- Alice to confirm schedule.\n",
        )
        .unwrap();
        let r2 = record_task_run(
            &data_dir,
            &ws_dir,
            "weekly client brief for beta",
            None,
            "run-2",
            "Updated brief with Risk Register and Owner Actions.",
            &["brief.md".to_string()],
        );
        assert!(
            r2["delta_vs_prev"].as_f64().unwrap() > 0.0,
            "Run 2 must beat Run 1: {r2}"
        );
        assert!(r2["score"].as_f64().unwrap() > s1);

        // Feedback: accept as good enough with recipe lock.
        let accepted = apply_feedback(
            &data_dir,
            "weekly-client-brief",
            "accept_good_enough",
            Some("Great layout"),
            vec![],
            Some("Keep Risk Register + Owner Actions with [file:L##] citations"),
        )
        .unwrap();
        assert_eq!(accepted.status, "accepted_good_enough");
        assert_eq!(accepted.accepted_run_id.as_deref(), Some("run-2"));
        assert!(accepted.accepted_digest.is_some());
        assert_eq!(
            accepted.accepted_recipe.as_deref(),
            Some("Keep Risk Register + Owner Actions with [file:L##] citations")
        );

        let guidance_locked = prompt_guidance_for_task(&data_dir, task, None)
            .expect("accepted guidance should be present");
        assert!(guidance_locked.contains("DO NOT DRIFT"));
        assert!(guidance_locked.contains("Keep Risk Register + Owner Actions"));

        // Run 3: stable at bar, changes stopped, accepted bar preserved.
        let r3 = record_task_run(
            &data_dir,
            &ws_dir,
            task,
            None,
            "run-3",
            "Followed accepted recipe with Risk Register and Owner Actions.",
            &["brief.md".to_string()],
        );
        assert_eq!(r3["status"], "accepted_good_enough");
        assert_eq!(r3["stable_at_bar"], true);
        assert_eq!(r3["changes_stopped"], true);

        let bar_final = get_quality_bar(&data_dir, "weekly-client-brief").unwrap();
        assert_eq!(bar_final.accepted_run_id.as_deref(), Some("run-2"));
        assert_eq!(bar_final.runs.len(), 3);

        // Re-opening from accepted state via "improve" works cleanly.
        let reopened = apply_feedback(
            &data_dir,
            "weekly-client-brief",
            "improve",
            Some("Add Budget Variance"),
            vec!["Budget Variance".to_string()],
            None,
        )
        .unwrap();
        assert_eq!(reopened.status, "improving");
        assert!(reopened.criteria.contains(&"Budget Variance".to_string()));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(quality_bars_path(&data_dir))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn explicit_family_and_error_paths() {
        let root = make_temp_dir("explicit");
        let data_dir = root.join("data");
        let ws_dir = root.join("ws");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::create_dir_all(&ws_dir).unwrap();

        assert!(apply_feedback(&data_dir, "missing", "improve", None, vec![], None).is_err());

        std::fs::write(ws_dir.join("out.md"), "# Report\nDetails.\n").unwrap();
        let res = record_task_run(
            &data_dir,
            &ws_dir,
            "totally different wording",
            Some("board-pack"),
            "run-a",
            "Completed board pack.",
            &["out.md".to_string()],
        );
        assert_eq!(res["task_family"], "board-pack");
        assert!(apply_feedback(&data_dir, "board-pack", "bogus", None, vec![], None).is_err());

        let bars = load_quality_bars(&data_dir);
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].task_family, "board-pack");

        let _ = std::fs::remove_dir_all(&root);
    }
}
