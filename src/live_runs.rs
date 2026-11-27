//! Crash-resilient persistence and boot recovery for ad-hoc live chat runs.
//!
//! Active and interrupted ad-hoc agent runs are persisted atomically to
//! `$ERGON_DATA_DIR/live_runs.json` (mode `0600`, temp file + rename) on every
//! lifecycle transition (start, tool step checkpoint, completion, cancellation,
//! failure, boot recovery, and resume claim).

use crate::records;
use crate::tools::{atomic_write_bytes_with_mode, truncate};
use crate::{backup_corrupt, AgentSnapshot};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub const MAX_LIVE_RUNS_HISTORY: usize = 50;
pub const MAX_PERSISTED_STEPS: usize = 128;
pub const MAX_AUTO_RESUMES: u32 = 3;

static LIVE_RUNS_IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn with_io_lock<T>(f: impl FnOnce() -> T) -> T {
    let lock = LIVE_RUNS_IO_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn live_runs_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("live_runs.json")
}

/// Whether boot recovery should automatically resume interrupted runs when a
/// model is configured. Enabled by default; set `ERGON_AUTO_RESUME_RUNS=0` (or
/// `false`/`no`/`off`) to surface interrupted runs for manual resume only.
pub fn auto_resume_enabled() -> bool {
    match std::env::var("ERGON_AUTO_RESUME_RUNS") {
        Ok(v) => {
            let low = v.trim().to_ascii_lowercase();
            !matches!(low.as_str(), "0" | "false" | "no" | "off")
        }
        Err(_) => true,
    }
}

#[derive(Clone, Debug)]
pub struct PersistedLiveRun {
    pub run_id: String,
    pub thread_id: String,
    pub person_id: String,
    pub task: String,
    pub started_ms: u64,
    pub updated_ms: u64,
    pub finished_ms: Option<u64>,
    pub steps: Vec<String>,
    pub written_files: Vec<String>,
    /// One of: `running`, `interrupted`, `resuming`, `completed`, `cancelled`, `failed`.
    pub status: String,
    pub error: Option<String>,
    pub summary: Option<String>,
    pub resume_count: u32,
    pub snapshot: Option<AgentSnapshot>,
}

impl PersistedLiveRun {
    pub fn is_active(&self) -> bool {
        matches!(self.status.as_str(), "running" | "resuming")
    }

    pub fn is_recoverable(&self) -> bool {
        self.status == "interrupted"
    }

    pub fn to_storage_json(&self) -> Value {
        let mut obj = json!({
            "run_id": self.run_id,
            "thread_id": self.thread_id,
            "person_id": self.person_id,
            "task": self.task,
            "started_ms": self.started_ms,
            "updated_ms": self.updated_ms,
            "finished_ms": self.finished_ms,
            "steps": self.steps,
            "written_files": self.written_files,
            "status": self.status,
            "error": self.error,
            "summary": self.summary,
            "resume_count": self.resume_count,
        });
        if let Some(ref snap) = self.snapshot {
            obj["snapshot"] = snap.to_json();
        }
        obj
    }

    pub fn to_api_json(&self, model_configured: bool) -> Value {
        json!({
            "run_id": self.run_id,
            "thread_id": self.thread_id,
            "person_id": self.person_id,
            "task": self.task,
            "started_ms": self.started_ms,
            "updated_ms": self.updated_ms,
            "finished_ms": self.finished_ms,
            "steps": self.steps,
            "written_files": self.written_files,
            "status": self.status,
            "error": self.error,
            "summary": self.summary,
            "resume_count": self.resume_count,
            "can_resume": model_configured && self.is_recoverable(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let run_id = obj.get("run_id")?.as_str()?.trim().to_string();
        if run_id.is_empty() {
            return None;
        }
        let thread_id = obj
            .get("thread_id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let person_id = obj
            .get("person_id")
            .and_then(|x| x.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("default")
            .to_string();
        let task = obj
            .get("task")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let started_ms = obj.get("started_ms").and_then(|x| x.as_u64()).unwrap_or(0);
        let updated_ms = obj
            .get("updated_ms")
            .and_then(|x| x.as_u64())
            .unwrap_or(started_ms);
        let finished_ms = obj.get("finished_ms").and_then(|x| x.as_u64());
        let steps = obj
            .get("steps")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let written_files = obj
            .get("written_files")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let raw_status = obj
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("interrupted");
        let status = match raw_status {
            "running" | "interrupted" | "resuming" | "completed" | "cancelled" | "failed" => {
                raw_status.to_string()
            }
            _ => "interrupted".to_string(),
        };
        let error = obj
            .get("error")
            .and_then(|x| x.as_str())
            .map(str::to_string);
        let summary = obj
            .get("summary")
            .and_then(|x| x.as_str())
            .map(str::to_string);
        let resume_count = obj
            .get("resume_count")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as u32;
        let snapshot = obj.get("snapshot").and_then(AgentSnapshot::from_json);
        Some(Self {
            run_id,
            thread_id,
            person_id,
            task,
            started_ms,
            updated_ms,
            finished_ms,
            steps,
            written_files,
            status,
            error,
            summary,
            resume_count,
            snapshot,
        })
    }
}

fn load_unlocked(root_data_dir: &Path) -> (Vec<PersistedLiveRun>, Option<String>) {
    let path = live_runs_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (Vec::new(), None);
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(v) => match v.get("runs").and_then(|r| r.as_array()) {
            Some(arr) => {
                let runs = arr
                    .iter()
                    .filter_map(PersistedLiveRun::from_json)
                    .collect();
                (runs, None)
            }
            None => {
                let bak = backup_corrupt(&path)
                    .and_then(|_| path.file_name().map(|n| n.to_string_lossy().into_owned()));
                (Vec::new(), bak)
            }
        },
        Err(_) => {
            let bak = backup_corrupt(&path)
                .and_then(|_| path.file_name().map(|n| n.to_string_lossy().into_owned()));
            (Vec::new(), bak)
        }
    }
}

fn prune_runs(runs: &mut Vec<PersistedLiveRun>) {
    // Keep all non-terminal runs plus at most MAX_LIVE_RUNS_HISTORY newest terminal runs.
    let mut keep_active: Vec<PersistedLiveRun> = Vec::new();
    let mut terminal: Vec<PersistedLiveRun> = Vec::new();
    for r in runs.drain(..) {
        if matches!(r.status.as_str(), "running" | "interrupted" | "resuming") {
            keep_active.push(r);
        } else {
            terminal.push(r);
        }
    }
    terminal.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
    terminal.truncate(MAX_LIVE_RUNS_HISTORY);
    keep_active.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
    keep_active.extend(terminal);
    *runs = keep_active;
}

fn save_unlocked(root_data_dir: &Path, runs: &mut Vec<PersistedLiveRun>) -> Result<(), String> {
    prune_runs(runs);
    let payload = json!({
        "version": 1,
        "runs": runs.iter().map(PersistedLiveRun::to_storage_json).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize live_runs.json: {e}"))?;
    atomic_write_bytes_with_mode(&live_runs_path(root_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write live_runs.json: {e}"))
}

pub fn load_all(root_data_dir: &Path) -> Vec<PersistedLiveRun> {
    with_io_lock(|| load_unlocked(root_data_dir).0)
}

pub fn list_for_person(root_data_dir: &Path, person_id: &str) -> Vec<PersistedLiveRun> {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        runs.retain(|r| r.person_id == person_id);
        runs.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
        runs
    })
}

pub fn recoverable_for_person(root_data_dir: &Path, person_id: &str) -> Vec<PersistedLiveRun> {
    list_for_person(root_data_dir, person_id)
        .into_iter()
        .filter(|r| r.is_recoverable())
        .collect()
}

pub fn get_run(root_data_dir: &Path, run_id: &str) -> Option<PersistedLiveRun> {
    with_io_lock(|| {
        let (runs, _) = load_unlocked(root_data_dir);
        runs.into_iter().find(|r| r.run_id == run_id)
    })
}

/// Insert or update a persisted live run when starting or resuming.
pub fn record_started(root_data_dir: &Path, run: PersistedLiveRun) -> Result<(), String> {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        if let Some(existing) = runs.iter_mut().find(|r| r.run_id == run.run_id) {
            existing.thread_id = run.thread_id;
            existing.person_id = run.person_id;
            existing.task = run.task;
            existing.updated_ms = run.updated_ms;
            existing.finished_ms = None;
            existing.status = run.status;
            existing.error = None;
            if !run.steps.is_empty() && existing.steps.is_empty() {
                existing.steps = run.steps;
            }
            if !run.written_files.is_empty() && existing.written_files.is_empty() {
                existing.written_files = run.written_files;
            }
            if run.snapshot.is_some() {
                existing.snapshot = run.snapshot;
            }
        } else {
            runs.push(run);
        }
        save_unlocked(root_data_dir, &mut runs)
    })
}

/// Append a completed tool step and optional conversation snapshot atomically.
pub fn record_step(
    root_data_dir: &Path,
    run_id: &str,
    step: &str,
    written_files: &[String],
    snapshot: Option<AgentSnapshot>,
) -> Result<(), String> {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        let Some(run) = runs.iter_mut().find(|r| r.run_id == run_id) else {
            return Ok(());
        };
        if run.steps.len() >= MAX_PERSISTED_STEPS {
            run.steps.remove(0);
        }
        run.steps.push(truncate(step, 2048));
        run.written_files = written_files.to_vec();
        if let Some(snap) = snapshot {
            run.snapshot = Some(snap);
        }
        run.updated_ms = records::now_millis();
        save_unlocked(root_data_dir, &mut runs)
    })
}

/// Transition a persisted live run to a terminal state (`completed`, `cancelled`, `failed`).
pub fn record_terminal(
    root_data_dir: &Path,
    run_id: &str,
    status: &str,
    written_files: &[String],
    summary: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        let Some(run) = runs.iter_mut().find(|r| r.run_id == run_id) else {
            return Ok(());
        };
        let now = records::now_millis();
        run.status = status.to_string();
        run.updated_ms = now;
        run.finished_ms = Some(now);
        if !written_files.is_empty() {
            run.written_files = written_files.to_vec();
        }
        if let Some(s) = summary {
            run.summary = Some(truncate(&s, 2000));
        }
        run.error = error.map(|e| truncate(&e, 1000));
        // Clear bulky conversation snapshot once terminal to keep live_runs.json compact.
        run.snapshot = None;
        save_unlocked(root_data_dir, &mut runs)
    })
}

/// Scan `$ERGON_DATA_DIR/live_runs.json` on server boot.
///
/// Any run left in `running` or `resuming` state from a prior crash/restart is
/// atomically transitioned to `interrupted` so it can either be auto-resumed
/// (when a model is configured and auto-resume is enabled) or surfaced honestly
/// in `/api/runs/status` for one-click `/api/runs/resume`.
///
/// Returns `(newly_interrupted_runs, corrupt_backup_filename)`.
pub fn recover_stale_runs_on_boot(
    root_data_dir: &Path,
    model_configured: bool,
) -> (Vec<PersistedLiveRun>, Option<String>) {
    with_io_lock(|| {
        let (mut runs, corrupt_bak) = load_unlocked(root_data_dir);
        let now = records::now_millis();
        let mut recovered = Vec::new();
        let mut changed = false;
        for run in &mut runs {
            if run.status == "running" || run.status == "resuming" {
                changed = true;
                run.status = "interrupted".to_string();
                run.updated_ms = now;
                run.finished_ms = None;
                run.error = Some(if !model_configured {
                    "Interrupted by server restart — configure a model connection to resume."
                        .to_string()
                } else if run.resume_count >= MAX_AUTO_RESUMES {
                    "Interrupted repeatedly — automatic resume paused; resume manually to continue."
                        .to_string()
                } else {
                    "Interrupted by server restart — ready to resume.".to_string()
                });
                recovered.push(run.clone());
            }
        }
        if changed {
            let _ = save_unlocked(root_data_dir, &mut runs);
        }
        (recovered, corrupt_bak)
    })
}

/// Atomically claim an interrupted (or optionally failed) run for resuming,
/// transitioning it to `resuming` and incrementing `resume_count`.
///
/// Guarantees that concurrent boot auto-resume and manual `POST /api/runs/resume`
/// can never execute the same run twice.
pub fn claim_for_resume(
    root_data_dir: &Path,
    person_id: &str,
    run_id: Option<&str>,
    thread_id: Option<&str>,
    allow_failed: bool,
) -> Result<PersistedLiveRun, (u16, String)> {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        let target_idx = if let Some(rid) = run_id.map(str::trim).filter(|s| !s.is_empty()) {
            let Some(pos) = runs
                .iter()
                .position(|r| r.run_id == rid && r.person_id == person_id)
            else {
                return Err((404, format!("no persisted run '{rid}' found")));
            };
            let st = runs[pos].status.as_str();
            if st == "running" || st == "resuming" {
                return Err((409, format!("run '{rid}' is already {st}")));
            }
            if st == "completed" || st == "cancelled" {
                return Err((409, format!("run '{rid}' is already {st}")));
            }
            if st == "failed" && !allow_failed {
                return Err((409, format!("run '{rid}' already failed")));
            }
            pos
        } else {
            let tf = thread_id.map(str::trim).filter(|s| !s.is_empty());
            let mut candidates: Vec<(usize, u64)> = runs
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r.person_id == person_id
                        && r.status == "interrupted"
                        && tf.map(|t| r.thread_id == t).unwrap_or(true)
                })
                .map(|(idx, r)| (idx, r.updated_ms))
                .collect();
            candidates.sort_by(|a, b| b.1.cmp(&a.1));
            match candidates.first() {
                Some((idx, _)) => *idx,
                None => return Err((404, "no interrupted run available to resume".to_string())),
            }
        };

        let now = records::now_millis();
        runs[target_idx].status = "resuming".to_string();
        runs[target_idx].updated_ms = now;
        runs[target_idx].finished_ms = None;
        runs[target_idx].error = None;
        runs[target_idx].resume_count = runs[target_idx].resume_count.saturating_add(1);
        let claimed = runs[target_idx].clone();
        save_unlocked(root_data_dir, &mut runs).map_err(|e| (500, e))?;
        Ok(claimed)
    })
}

/// Revert a `resuming` claim back to `interrupted` if execution could not start.
pub fn revert_resume_claim(root_data_dir: &Path, run_id: &str, reason: &str) {
    with_io_lock(|| {
        let (mut runs, _) = load_unlocked(root_data_dir);
        if let Some(run) = runs.iter_mut().find(|r| r.run_id == run_id) {
            if run.status == "resuming" {
                run.status = "interrupted".to_string();
                run.updated_ms = records::now_millis();
                run.error = Some(truncate(reason, 500));
                let _ = save_unlocked(root_data_dir, &mut runs);
            }
        }
    });
}
