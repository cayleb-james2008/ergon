//! Outcome Scoring & Recursive Self-Improvement Loop.
//!
//! Tracks live artifact outcomes (`outcomes.jsonl`), versioned artifact
//! evolution (`versions.json`), deterministic replay cases (`replay_cases.json`),
//! and an immutable evolution audit log (`evolution_log.jsonl`) under
//! `<root_data_dir>/evolution/` with `0600` POSIX permissions.

use crate::library::Library;
use crate::records::{load_work, now_millis};
use crate::skills::{check_script, run_library_skill};
use crate::tools::{atomic_write_bytes_with_mode, sandbox_path};
use crate::tripwire::sha256_hex;
use crate::{call_model_with_usage, truncate, Config, NO_KEY_ERROR};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const DEFAULT_IMPROVER_ID: &str = "default-improver";
pub const DEFAULT_IMPROVER_INSTRUCTIONS: &str =
    "Improve the artifact concisely to fix failed checks and user edits without breaking passing tests.";

static EVOLUTION_LOCK: Mutex<()> = Mutex::new(());
static EVOLUTION_SEQ: AtomicU64 = AtomicU64::new(0);
static SANDBOX_SEQ: AtomicU64 = AtomicU64::new(0);

fn with_evolution_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = EVOLUTION_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

fn next_seq() -> u64 {
    EVOLUTION_SEQ.fetch_add(1, Ordering::SeqCst)
}

fn evolution_dir(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("evolution")
}

fn outcomes_path(root_data_dir: &Path) -> PathBuf {
    evolution_dir(root_data_dir).join("outcomes.jsonl")
}

fn versions_path(root_data_dir: &Path) -> PathBuf {
    evolution_dir(root_data_dir).join("versions.json")
}

fn replay_cases_path(root_data_dir: &Path) -> PathBuf {
    evolution_dir(root_data_dir).join("replay_cases.json")
}

fn evolution_log_path(root_data_dir: &Path) -> PathBuf {
    evolution_dir(root_data_dir).join("evolution_log.jsonl")
}

fn valid_kind(kind: &str) -> bool {
    matches!(kind, "skill" | "subagent" | "tool" | "improver")
}

fn sanitize_no_jargon(text: &str) -> String {
    text.replace("rune skill failed to build", "script failed to build")
        .replace("rune sources", "script sources")
        .replace("rune source", "script source")
        .replace("Rune", "Script")
}

fn append_jsonl_0600(path: &Path, v: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create directory {}: {e}", parent.display()))?;
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut line = v.to_string();
    line.push('\n');
    f.write_all(line.as_bytes())
        .map_err(|e| format!("append {}: {e}", path.display()))?;
    f.flush()
        .map_err(|e| format!("flush {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn parse_string_vec(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_string_map(v: &Value) -> BTreeMap<String, String> {
    v.as_object()
        .map(|map| {
            map.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct OutcomeRecord {
    pub id: String,
    pub ts_ms: u64,
    pub person_id: String,
    pub artifact_kind: String,
    pub artifact_id: String,
    pub artifact_version: u32,
    pub run_id: String,
    pub task: String,
    pub approved: Option<bool>,
    pub edits_needed: u32,
    pub reruns: u32,
    pub time_saved_mins: f64,
    pub explicit_score: Option<f64>,
    pub quality_score: f64,
    pub feedback: Option<String>,
}

impl OutcomeRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "ts_ms": self.ts_ms,
            "person_id": self.person_id,
            "artifact_kind": self.artifact_kind,
            "artifact_id": self.artifact_id,
            "artifact_version": self.artifact_version,
            "run_id": self.run_id,
            "task": self.task,
            "approved": self.approved,
            "edits_needed": self.edits_needed,
            "reruns": self.reruns,
            "time_saved_mins": self.time_saved_mins,
            "explicit_score": self.explicit_score,
            "quality_score": self.quality_score,
            "feedback": self.feedback,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let artifact_id = v["artifact_id"].as_str()?.trim().to_string();
        if artifact_id.is_empty() {
            return None;
        }
        let approved = v.get("approved").and_then(|a| a.as_bool());
        let edits_needed = v["edits_needed"].as_u64().unwrap_or(0) as u32;
        let reruns = v["reruns"].as_u64().unwrap_or(0) as u32;
        let time_saved_mins = v["time_saved_mins"].as_f64().unwrap_or(0.0);
        let explicit_score = v.get("explicit_score").and_then(|e| e.as_f64());
        let quality_score = v
            .get("quality_score")
            .and_then(|q| q.as_f64())
            .filter(|q| q.is_finite())
            .unwrap_or_else(|| {
                compute_quality_score(
                    approved,
                    edits_needed,
                    reruns,
                    time_saved_mins,
                    explicit_score,
                )
            });
        Some(Self {
            id: v["id"].as_str().unwrap_or("").to_string(),
            ts_ms: v["ts_ms"].as_u64().unwrap_or(0),
            person_id: v["person_id"].as_str().unwrap_or("default").to_string(),
            artifact_kind: v["artifact_kind"].as_str().unwrap_or("skill").to_string(),
            artifact_id,
            artifact_version: v["artifact_version"].as_u64().unwrap_or(1) as u32,
            run_id: v["run_id"].as_str().unwrap_or("").to_string(),
            task: v["task"].as_str().unwrap_or("").to_string(),
            approved,
            edits_needed,
            reruns,
            time_saved_mins,
            explicit_score,
            quality_score: quality_score.clamp(0.0, 1.0),
            feedback: v
                .get("feedback")
                .and_then(|f| f.as_str())
                .map(str::to_string),
        })
    }
}

#[derive(Clone, Debug)]
pub struct OutcomeSummary {
    pub record: OutcomeRecord,
    pub rolled_back_to: Option<u32>,
    pub rollback_reason: Option<String>,
}

impl OutcomeSummary {
    pub fn to_json(&self) -> Value {
        json!({
            "record": self.record.to_json(),
            "rolled_back_to": self.rolled_back_to,
            "rollback_reason": self.rollback_reason,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactVersionEntry {
    pub version: u32,
    pub content: String,
    pub params: Vec<String>,
    pub created_ms: u64,
    pub author: String,
    pub note: String,
    pub replay_score: f64,
    pub status: String, // "active" | "superseded" | "rejected" | "rolled_back"
}

impl ArtifactVersionEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "content": self.content,
            "params": self.params,
            "created_ms": self.created_ms,
            "author": self.author,
            "note": self.note,
            "replay_score": self.replay_score,
            "status": self.status,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let version = v["version"].as_u64()? as u32;
        Some(Self {
            version,
            content: v["content"].as_str().unwrap_or("").to_string(),
            params: parse_string_vec(&v["params"]),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            author: v["author"].as_str().unwrap_or("ergon").to_string(),
            note: v["note"].as_str().unwrap_or("").to_string(),
            replay_score: v["replay_score"].as_f64().unwrap_or(0.0),
            status: v["status"].as_str().unwrap_or("active").to_string(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactEvolution {
    pub id: String,
    pub kind: String,
    pub active_version: u32,
    pub previous_stable_version: Option<u32>,
    pub baseline_live_score: f64,
    pub versions: Vec<ArtifactVersionEntry>,
}

impl ArtifactEvolution {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind,
            "active_version": self.active_version,
            "previous_stable_version": self.previous_stable_version,
            "baseline_live_score": self.baseline_live_score,
            "versions": self.versions.iter().map(ArtifactVersionEntry::to_json).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let id = v["id"].as_str()?.trim().to_string();
        let kind = v["kind"].as_str()?.trim().to_string();
        if id.is_empty() || !valid_kind(&kind) {
            return None;
        }
        let versions = v["versions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(ArtifactVersionEntry::from_json)
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            id,
            kind,
            active_version: v["active_version"].as_u64().unwrap_or(1) as u32,
            previous_stable_version: v
                .get("previous_stable_version")
                .and_then(|p| p.as_u64())
                .map(|p| p as u32),
            baseline_live_score: v["baseline_live_score"].as_f64().unwrap_or(0.0),
            versions,
        })
    }

    fn active_entry(&self) -> Option<&ArtifactVersionEntry> {
        self.versions
            .iter()
            .find(|v| v.version == self.active_version)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayCheck {
    ExpectOk,
    OutputContains(String),
    OutputNotContains(String),
    FileContains { path: String, needle: String },
    MinLines(usize),
}

impl ReplayCheck {
    pub fn to_json(&self) -> Value {
        match self {
            Self::ExpectOk => json!({ "type": "expect_ok" }),
            Self::OutputContains(s) => json!({ "type": "output_contains", "value": s }),
            Self::OutputNotContains(s) => json!({ "type": "output_not_contains", "value": s }),
            Self::FileContains { path, needle } => json!({
                "type": "file_contains",
                "path": path,
                "needle": needle,
            }),
            Self::MinLines(n) => json!({ "type": "min_lines", "min": n }),
        }
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if let Some(s) = v.as_str() {
            return matches!(s, "ExpectOk" | "expect_ok").then_some(Self::ExpectOk);
        }
        let obj = v.as_object()?;
        if let Some(kind) = obj
            .get("type")
            .or_else(|| obj.get("kind"))
            .and_then(|k| k.as_str())
        {
            return match kind {
                "expect_ok" | "ExpectOk" => Some(Self::ExpectOk),
                "output_contains" | "OutputContains" => obj
                    .get("value")
                    .or_else(|| obj.get("needle"))
                    .and_then(|x| x.as_str())
                    .map(|s| Self::OutputContains(s.to_string())),
                "output_not_contains" | "OutputNotContains" => obj
                    .get("value")
                    .or_else(|| obj.get("needle"))
                    .and_then(|x| x.as_str())
                    .map(|s| Self::OutputNotContains(s.to_string())),
                "file_contains" | "FileContains" => {
                    let path = obj.get("path")?.as_str()?.to_string();
                    let needle = obj
                        .get("needle")
                        .or_else(|| obj.get("value"))?
                        .as_str()?
                        .to_string();
                    Some(Self::FileContains { path, needle })
                }
                "min_lines" | "MinLines" => obj
                    .get("min")
                    .or_else(|| obj.get("value"))
                    .and_then(|n| n.as_u64())
                    .map(|n| Self::MinLines(n as usize)),
                _ => None,
            };
        }
        if obj.contains_key("ExpectOk") {
            return Some(Self::ExpectOk);
        }
        if let Some(s) = obj.get("OutputContains").and_then(|x| x.as_str()) {
            return Some(Self::OutputContains(s.to_string()));
        }
        if let Some(s) = obj.get("OutputNotContains").and_then(|x| x.as_str()) {
            return Some(Self::OutputNotContains(s.to_string()));
        }
        if let Some(fc) = obj.get("FileContains").and_then(|x| x.as_object()) {
            let path = fc.get("path")?.as_str()?.to_string();
            let needle = fc.get("needle")?.as_str()?.to_string();
            return Some(Self::FileContains { path, needle });
        }
        if let Some(n) = obj.get("MinLines").and_then(|x| x.as_u64()) {
            return Some(Self::MinLines(n as usize));
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct ReplayCase {
    pub case_id: String,
    pub artifact_id: String,
    pub artifact_kind: String,
    pub setup_files: BTreeMap<String, String>,
    pub args: Vec<String>,
    pub prompt_or_input: String,
    pub checks: Vec<ReplayCheck>,
}

impl ReplayCase {
    pub fn to_json(&self) -> Value {
        json!({
            "case_id": self.case_id,
            "artifact_id": self.artifact_id,
            "artifact_kind": self.artifact_kind,
            "setup_files": self.setup_files,
            "args": self.args,
            "prompt_or_input": self.prompt_or_input,
            "checks": self.checks.iter().map(ReplayCheck::to_json).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let artifact_id = v["artifact_id"].as_str()?.trim().to_string();
        if artifact_id.is_empty() {
            return None;
        }
        let case_id = v["case_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("case-{}-{}", now_millis(), next_seq()));
        let checks = v["checks"]
            .as_array()
            .map(|arr| arr.iter().filter_map(ReplayCheck::from_json).collect())
            .unwrap_or_default();
        Some(Self {
            case_id,
            artifact_id,
            artifact_kind: v["artifact_kind"].as_str().unwrap_or("").to_string(),
            setup_files: parse_string_map(&v["setup_files"]),
            args: parse_string_vec(&v["args"]),
            prompt_or_input: v["prompt_or_input"].as_str().unwrap_or("").to_string(),
            checks,
        })
    }
}

#[derive(Clone, Debug)]
pub struct EvolutionLogEntry {
    pub ts_ms: u64,
    pub event: String,
    pub artifact_id: String,
    pub kind: String,
    pub from_version: u32,
    pub to_version: u32,
    pub champion_score: f64,
    pub candidate_score: f64,
    pub detail: String,
}

impl EvolutionLogEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "ts_ms": self.ts_ms,
            "event": self.event,
            "artifact_id": self.artifact_id,
            "kind": self.kind,
            "from_version": self.from_version,
            "to_version": self.to_version,
            "champion_score": self.champion_score,
            "candidate_score": self.candidate_score,
            "detail": self.detail,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        Some(Self {
            ts_ms: v["ts_ms"].as_u64().unwrap_or(0),
            event: v["event"].as_str()?.to_string(),
            artifact_id: v["artifact_id"].as_str()?.to_string(),
            kind: v["kind"].as_str().unwrap_or("").to_string(),
            from_version: v["from_version"].as_u64().unwrap_or(0) as u32,
            to_version: v["to_version"].as_u64().unwrap_or(0) as u32,
            champion_score: v["champion_score"].as_f64().unwrap_or(0.0),
            candidate_score: v["candidate_score"].as_f64().unwrap_or(0.0),
            detail: v["detail"].as_str().unwrap_or("").to_string(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct EvaluationOutcome {
    pub artifact_id: String,
    pub kind: String,
    pub promoted: bool,
    pub from_version: u32,
    pub to_version: u32,
    pub candidate_compiled: bool,
    pub champion_score: f64,
    pub candidate_score: f64,
    pub regressions: usize,
    pub cases_run: usize,
    pub detail: String,
}

impl EvaluationOutcome {
    pub fn to_json(&self) -> Value {
        json!({
            "artifact_id": self.artifact_id,
            "kind": self.kind,
            "promoted": self.promoted,
            "from_version": self.from_version,
            "to_version": self.to_version,
            "candidate_compiled": self.candidate_compiled,
            "champion_score": self.champion_score,
            "candidate_score": self.candidate_score,
            "regressions": self.regressions,
            "cases_run": self.cases_run,
            "detail": self.detail,
        })
    }
}

// ---------------------------------------------------------------------------
// Deterministic Quality Scoring
// ---------------------------------------------------------------------------

pub fn compute_quality_score(
    approved: Option<bool>,
    edits_needed: u32,
    reruns: u32,
    time_saved_mins: f64,
    explicit_score: Option<f64>,
) -> f64 {
    let base = match approved {
        Some(true) => 0.85,
        None => 0.70,
        Some(false) => 0.15,
    };
    let edit_penalty = 0.15 * (edits_needed as f64).min(4.0);
    let rerun_penalty = 0.10 * (reruns as f64).min(4.0);
    let time_bonus = if time_saved_mins.is_finite() {
        (time_saved_mins / 30.0).clamp(0.0, 0.15)
    } else {
        0.0
    };
    let computed = base - edit_penalty - rerun_penalty + time_bonus;
    let combined = match explicit_score {
        Some(e) if e.is_finite() => 0.6 * computed + 0.4 * e.clamp(0.0, 1.0),
        _ => computed,
    };
    combined.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Store Load / Save Helpers (always called under EVOLUTION_LOCK)
// ---------------------------------------------------------------------------

fn default_improver_evolution() -> ArtifactEvolution {
    ArtifactEvolution {
        id: DEFAULT_IMPROVER_ID.to_string(),
        kind: "improver".to_string(),
        active_version: 1,
        previous_stable_version: None,
        baseline_live_score: 0.0,
        versions: vec![ArtifactVersionEntry {
            version: 1,
            content: DEFAULT_IMPROVER_INSTRUCTIONS.to_string(),
            params: Vec::new(),
            created_ms: now_millis(),
            author: "ergon".to_string(),
            note: "Seeded default improver".to_string(),
            replay_score: 0.0,
            status: "active".to_string(),
        }],
    }
}

fn load_versions_unlocked(root_data_dir: &Path) -> Vec<ArtifactEvolution> {
    let path = versions_path(root_data_dir);
    let mut items: Vec<ArtifactEvolution> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|v| {
            v["artifacts"]
                .as_array()
                .or_else(|| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(ArtifactEvolution::from_json)
                        .collect()
                })
        })
        .unwrap_or_default();

    if !items.iter().any(|a| a.id == DEFAULT_IMPROVER_ID) {
        items.insert(0, default_improver_evolution());
        let _ = save_versions_unlocked(root_data_dir, &items);
    }
    items
}

fn save_versions_unlocked(
    root_data_dir: &Path,
    artifacts: &[ArtifactEvolution],
) -> Result<(), String> {
    let payload = json!({
        "version": 1,
        "artifacts": artifacts.iter().map(ArtifactEvolution::to_json).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize versions.json: {e}"))?;
    atomic_write_bytes_with_mode(&versions_path(root_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write versions.json: {e}"))
}

fn load_replay_cases_unlocked(root_data_dir: &Path) -> Vec<ReplayCase> {
    let path = replay_cases_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v["cases"]
        .as_array()
        .or_else(|| v.as_array())
        .map(|arr| arr.iter().filter_map(ReplayCase::from_json).collect())
        .unwrap_or_default()
}

fn save_replay_cases_unlocked(root_data_dir: &Path, cases: &[ReplayCase]) -> Result<(), String> {
    let payload = json!({
        "version": 1,
        "cases": cases.iter().map(ReplayCase::to_json).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize replay_cases.json: {e}"))?;
    atomic_write_bytes_with_mode(&replay_cases_path(root_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write replay_cases.json: {e}"))
}

fn load_raw_outcomes_unlocked(root_data_dir: &Path) -> Vec<OutcomeRecord> {
    let path = outcomes_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|v| OutcomeRecord::from_json(&v))
        .collect()
}

/// Coalesces append-only `outcomes.jsonl` rows so edit updates appended for an
/// existing `(person_id, artifact_id, run_id)` supersede the earlier snapshot.
fn coalesce_outcomes(raw: Vec<OutcomeRecord>) -> Vec<OutcomeRecord> {
    let mut out: Vec<OutcomeRecord> = Vec::new();
    let mut idx_by_key: HashMap<String, usize> = HashMap::new();
    for rec in raw {
        let key = if !rec.run_id.trim().is_empty() {
            format!("run:{}:{}:{}", rec.person_id, rec.artifact_id, rec.run_id)
        } else if !rec.id.trim().is_empty() {
            format!("id:{}", rec.id)
        } else {
            format!("seq:{}", out.len())
        };
        if let Some(&idx) = idx_by_key.get(&key) {
            out[idx] = rec;
        } else {
            idx_by_key.insert(key, out.len());
            out.push(rec);
        }
    }
    out
}

fn load_outcomes_unlocked(root_data_dir: &Path) -> Vec<OutcomeRecord> {
    coalesce_outcomes(load_raw_outcomes_unlocked(root_data_dir))
}

fn load_log_unlocked(root_data_dir: &Path) -> Vec<EvolutionLogEntry> {
    let path = evolution_log_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|v| EvolutionLogEntry::from_json(&v))
        .collect()
}

fn append_log_unlocked(root_data_dir: &Path, entry: &EvolutionLogEntry) -> Result<(), String> {
    append_jsonl_0600(&evolution_log_path(root_data_dir), &entry.to_json())
}

// ---------------------------------------------------------------------------
// Public API: Seeding, Replay Cases, Outcomes, Auto-Rollback, History
// ---------------------------------------------------------------------------

pub fn ensure_artifact_seeded(
    root_data_dir: &Path,
    artifact_id: &str,
    kind: &str,
    content: &str,
    params: Vec<String>,
    author: &str,
) -> ArtifactEvolution {
    let kind_norm = if valid_kind(kind) { kind } else { "skill" };
    with_evolution_lock(|| {
        let mut all = load_versions_unlocked(root_data_dir);
        if let Some(existing) = all.iter().find(|a| a.id == artifact_id) {
            return existing.clone();
        }
        let ev = ArtifactEvolution {
            id: artifact_id.to_string(),
            kind: kind_norm.to_string(),
            active_version: 1,
            previous_stable_version: None,
            baseline_live_score: 0.0,
            versions: vec![ArtifactVersionEntry {
                version: 1,
                content: content.to_string(),
                params,
                created_ms: now_millis(),
                author: if author.trim().is_empty() {
                    "ergon".to_string()
                } else {
                    author.to_string()
                },
                note: "Seeded baseline version".to_string(),
                replay_score: 0.0,
                status: "active".to_string(),
            }],
        };
        all.push(ev.clone());
        let _ = save_versions_unlocked(root_data_dir, &all);
        ev
    })
}

pub fn add_replay_case(root_data_dir: &Path, mut case: ReplayCase) -> Result<ReplayCase, String> {
    let aid = case.artifact_id.trim();
    if aid.is_empty() {
        return Err("replay case artifact_id cannot be empty".to_string());
    }
    case.artifact_id = aid.to_string();
    if !case.artifact_kind.is_empty() && !valid_kind(&case.artifact_kind) {
        return Err(format!("invalid artifact_kind '{}'", case.artifact_kind));
    }
    if case.case_id.trim().is_empty() {
        case.case_id = format!("case-{}-{}", now_millis(), next_seq());
    }
    for rel in case.setup_files.keys() {
        if rel.trim().is_empty() || rel.contains("..") || Path::new(rel).is_absolute() {
            return Err(format!("invalid setup_files path '{rel}'"));
        }
    }
    for chk in &case.checks {
        if let ReplayCheck::FileContains { path, .. } = chk {
            if path.trim().is_empty() || path.contains("..") || Path::new(path).is_absolute() {
                return Err(format!("invalid FileContains path '{path}'"));
            }
        }
    }

    with_evolution_lock(|| {
        let _ = load_versions_unlocked(root_data_dir);
        let mut cases = load_replay_cases_unlocked(root_data_dir);
        if let Some(existing) = cases.iter_mut().find(|c| c.case_id == case.case_id) {
            *existing = case.clone();
        } else {
            cases.push(case.clone());
        }
        save_replay_cases_unlocked(root_data_dir, &cases)?;
        Ok(case)
    })
}

fn check_auto_rollback_unlocked(
    root_data_dir: &Path,
    library: Option<&mut Library>,
    artifact_id: &str,
) -> (Option<u32>, Option<String>) {
    let mut versions = load_versions_unlocked(root_data_dir);
    let Some(pos) = versions.iter().position(|a| a.id == artifact_id) else {
        return (None, None);
    };
    let Some(prev_v) = versions[pos].previous_stable_version else {
        return (None, None);
    };

    let active_v = versions[pos].active_version;
    let baseline = versions[pos].baseline_live_score;
    let outcomes = load_outcomes_unlocked(root_data_dir);
    let matching: Vec<&OutcomeRecord> = outcomes
        .iter()
        .filter(|o| o.artifact_id == artifact_id && o.artifact_version == active_v)
        .collect();

    if matching.len() < 2 {
        return (None, None);
    }

    let count = matching.len() as f64;
    let mean_score = matching.iter().map(|o| o.quality_score).sum::<f64>() / count;
    let denials = matching
        .iter()
        .filter(|o| o.approved == Some(false))
        .count();
    let denial_rate = denials as f64 / count;

    let score_regressed = mean_score < baseline - 0.10;
    let high_denial = denial_rate >= 0.50;
    if !score_regressed && !high_denial {
        return (None, None);
    }

    let Some(prev_entry) = versions[pos]
        .versions
        .iter()
        .find(|v| v.version == prev_v)
        .cloned()
    else {
        return (None, None);
    };

    for entry in &mut versions[pos].versions {
        if entry.version == active_v {
            entry.status = "rolled_back".to_string();
        } else if entry.version == prev_v {
            entry.status = "active".to_string();
        }
    }
    let kind = versions[pos].kind.clone();
    versions[pos].active_version = prev_v;
    versions[pos].previous_stable_version = None;
    versions[pos].baseline_live_score = prev_entry.replay_score.min(0.85);

    if save_versions_unlocked(root_data_dir, &versions).is_err() {
        return (None, None);
    }

    if let Some(lib) = library {
        if matches!(kind.as_str(), "skill" | "tool" | "subagent")
            && lib
                .update_item_for_evolution(artifact_id, &prev_entry.content, prev_v, None)
                .is_ok()
        {
            lib.save(root_data_dir);
        }
    }

    let reason = format!(
        "auto-rolled back v{active_v} -> v{prev_v}: live mean={mean_score:.3} (baseline={baseline:.3}), denial_rate={denial_rate:.2} across {} runs",
        matching.len()
    );
    let _ = append_log_unlocked(
        root_data_dir,
        &EvolutionLogEntry {
            ts_ms: now_millis(),
            event: "auto_rolled_back".to_string(),
            artifact_id: artifact_id.to_string(),
            kind,
            from_version: active_v,
            to_version: prev_v,
            champion_score: prev_entry.replay_score,
            candidate_score: mean_score,
            detail: reason.clone(),
        },
    );

    (Some(prev_v), Some(reason))
}

pub fn check_auto_rollback(
    root_data_dir: &Path,
    library: Option<&mut Library>,
    artifact_id: &str,
) -> (Option<u32>, Option<String>) {
    with_evolution_lock(|| check_auto_rollback_unlocked(root_data_dir, library, artifact_id))
}

pub fn record_outcome(
    root_data_dir: &Path,
    library: Option<&mut Library>,
    mut rec: OutcomeRecord,
) -> OutcomeSummary {
    with_evolution_lock(|| {
        let versions = load_versions_unlocked(root_data_dir);
        if rec.ts_ms == 0 {
            rec.ts_ms = now_millis();
        }
        if rec.id.trim().is_empty() {
            rec.id = format!("out-{}-{}", rec.ts_ms, next_seq());
        }
        if rec.person_id.trim().is_empty() {
            rec.person_id = "default".to_string();
        }
        if rec.artifact_version == 0 {
            rec.artifact_version = versions
                .iter()
                .find(|a| a.id == rec.artifact_id)
                .map(|a| a.active_version)
                .unwrap_or(1);
        }
        rec.quality_score = compute_quality_score(
            rec.approved,
            rec.edits_needed,
            rec.reruns,
            rec.time_saved_mins,
            rec.explicit_score,
        );

        let _ = append_jsonl_0600(&outcomes_path(root_data_dir), &rec.to_json());
        let (rolled_back_to, rollback_reason) =
            check_auto_rollback_unlocked(root_data_dir, library, &rec.artifact_id);
        OutcomeSummary {
            record: rec,
            rolled_back_to,
            rollback_reason,
        }
    })
}

fn person_data_dir_for(root_data_dir: &Path, person_id: &str) -> PathBuf {
    if person_id.trim().is_empty() || person_id == "default" {
        root_data_dir.to_path_buf()
    } else {
        root_data_dir.join("people").join(person_id)
    }
}

fn file_matches_work_entry(entry: &Value, file_needle: &str) -> bool {
    let needle = file_needle.trim();
    if needle.is_empty() {
        return false;
    }
    entry["files"]
        .as_array()
        .map(|arr| {
            arr.iter().filter_map(|v| v.as_str()).any(|f| {
                f == needle || f.ends_with(&format!("/{needle}")) || needle.ends_with(&format!("/{f}"))
            })
        })
        .unwrap_or(false)
}

pub fn increment_edits_for_run(
    root_data_dir: &Path,
    mut library: Option<&mut Library>,
    person_id: &str,
    run_id_or_file: &str,
) -> Vec<OutcomeSummary> {
    let key = run_id_or_file.trim();
    if key.is_empty() {
        return Vec::new();
    }

    with_evolution_lock(|| {
        let _ = load_versions_unlocked(root_data_dir);
        let outcomes = load_outcomes_unlocked(root_data_dir);

        // First try direct run_id matches (for the person, or any person if person_id is empty).
        let mut matched: Vec<OutcomeRecord> = outcomes
            .iter()
            .filter(|o| {
                (person_id.is_empty() || o.person_id == person_id) && o.run_id == key
            })
            .cloned()
            .collect();

        // Fallback: match deliverables produced by a recent run in work_log.json,
        // or match task/feedback mentions of the file path.
        if matched.is_empty() {
            let p_dir = person_data_dir_for(root_data_dir, person_id);
            let work = load_work(&p_dir);
            let recent_work_ids: Vec<(String, String)> = work
                .iter()
                .rev()
                .filter(|e| file_matches_work_entry(e, key))
                .take(3)
                .map(|e| {
                    let tid = e["thread_id"].as_str().unwrap_or("").to_string();
                    let task = e["task"].as_str().unwrap_or("").to_string();
                    (tid, task)
                })
                .collect();

            for o in outcomes.iter().rev() {
                if !person_id.is_empty() && o.person_id != person_id {
                    continue;
                }
                let task_or_feedback =
                    o.task.contains(key) || o.feedback.as_deref().is_some_and(|f| f.contains(key));
                let work_match = recent_work_ids.iter().any(|(tid, task)| {
                    (!tid.is_empty() && o.run_id == *tid)
                        || (!task.is_empty() && o.task == *task)
                });
                if task_or_feedback || work_match {
                    matched.push(o.clone());
                    break;
                }
            }
        }

        let mut summaries = Vec::with_capacity(matched.len());
        for mut rec in matched {
            rec.edits_needed = rec.edits_needed.saturating_add(1);
            rec.ts_ms = now_millis();
            if !rec.run_id.trim().is_empty() || rec.id.trim().is_empty() {
                rec.id = format!("out-{}-{}", rec.ts_ms, next_seq());
            }
            rec.quality_score = compute_quality_score(
                rec.approved,
                rec.edits_needed,
                rec.reruns,
                rec.time_saved_mins,
                rec.explicit_score,
            );
            let _ = append_jsonl_0600(&outcomes_path(root_data_dir), &rec.to_json());
            let (rolled_back_to, rollback_reason) = check_auto_rollback_unlocked(
                root_data_dir,
                library.as_deref_mut(),
                &rec.artifact_id,
            );
            summaries.push(OutcomeSummary {
                record: rec,
                rolled_back_to,
                rollback_reason,
            });
        }
        summaries
    })
}

pub fn score_summary(root_data_dir: &Path) -> Value {
    with_evolution_lock(|| {
        let versions = load_versions_unlocked(root_data_dir);
        let outcomes = load_outcomes_unlocked(root_data_dir);
        let log = load_log_unlocked(root_data_dir);

        let artifacts: Vec<Value> = versions
            .iter()
            .map(|ev| {
                let active_outcomes: Vec<&OutcomeRecord> = outcomes
                    .iter()
                    .filter(|o| o.artifact_id == ev.id && o.artifact_version == ev.active_version)
                    .collect();
                let all_outcomes: Vec<&OutcomeRecord> = outcomes
                    .iter()
                    .filter(|o| o.artifact_id == ev.id)
                    .collect();
                let live_mean = if active_outcomes.is_empty() {
                    None
                } else {
                    Some(
                        active_outcomes.iter().map(|o| o.quality_score).sum::<f64>()
                            / active_outcomes.len() as f64,
                    )
                };
                let approval_count = active_outcomes
                    .iter()
                    .filter(|o| o.approved == Some(true))
                    .count();
                let approval_rate = if active_outcomes.is_empty() {
                    None
                } else {
                    Some(approval_count as f64 / active_outcomes.len() as f64)
                };
                let active_replay_score = ev.active_entry().map(|e| e.replay_score).unwrap_or(0.0);
                json!({
                    "id": ev.id,
                    "kind": ev.kind,
                    "active_version": ev.active_version,
                    "previous_stable_version": ev.previous_stable_version,
                    "baseline_live_score": ev.baseline_live_score,
                    "active_replay_score": active_replay_score,
                    "live_mean_score": live_mean,
                    "live_outcome_count": active_outcomes.len(),
                    "total_outcomes": all_outcomes.len(),
                    "approval_rate": approval_rate,
                    "versions_count": ev.versions.len(),
                    "versions": ev.versions.iter().map(|v| json!({
                        "version": v.version,
                        "status": v.status,
                        "replay_score": v.replay_score,
                        "author": v.author,
                        "note": v.note,
                        "created_ms": v.created_ms,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();

        let recent_start = log.len().saturating_sub(25);
        let recent_log: Vec<Value> = log[recent_start..]
            .iter()
            .rev()
            .map(EvolutionLogEntry::to_json)
            .collect();

        json!({
            "artifacts": artifacts,
            "total_outcomes": outcomes.len(),
            "recent_log": recent_log,
        })
    })
}

pub fn artifact_history(root_data_dir: &Path, artifact_id: &str) -> Value {
    with_evolution_lock(|| {
        let versions = load_versions_unlocked(root_data_dir);
        let outcomes = load_outcomes_unlocked(root_data_dir);
        let cases = load_replay_cases_unlocked(root_data_dir);
        let log = load_log_unlocked(root_data_dir);

        let ev = versions.into_iter().find(|a| a.id == artifact_id);
        let versions_json = ev
            .as_ref()
            .map(|e| {
                e.versions
                    .iter()
                    .map(ArtifactVersionEntry::to_json)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let outcomes_json: Vec<Value> = outcomes
            .into_iter()
            .filter(|o| o.artifact_id == artifact_id)
            .map(|o| o.to_json())
            .collect();
        let cases_json: Vec<Value> = cases
            .into_iter()
            .filter(|c| c.artifact_id == artifact_id)
            .map(|c| c.to_json())
            .collect();
        let log_json: Vec<Value> = log
            .into_iter()
            .filter(|l| l.artifact_id == artifact_id)
            .map(|l| l.to_json())
            .collect();

        json!({
            "artifact_id": artifact_id,
            "evolution": ev.map(|e| e.to_json()),
            "versions": versions_json,
            "outcomes": outcomes_json,
            "replay_cases": cases_json,
            "evolution_log": log_json,
        })
    })
}

pub fn manual_rollback(
    root_data_dir: &Path,
    library: Option<&mut Library>,
    artifact_id: &str,
) -> Result<Value, String> {
    with_evolution_lock(|| {
        let mut versions = load_versions_unlocked(root_data_dir);
        let Some(pos) = versions.iter().position(|a| a.id == artifact_id) else {
            return Err(format!("artifact '{artifact_id}' not found in evolution store"));
        };
        let active_v = versions[pos].active_version;
        let Some(prev_v) = versions[pos].previous_stable_version else {
            return Err(format!(
                "no previous stable version to roll back to for '{artifact_id}'"
            ));
        };
        let Some(prev_entry) = versions[pos]
            .versions
            .iter()
            .find(|v| v.version == prev_v)
            .cloned()
        else {
            return Err(format!(
                "previous version v{prev_v} missing for '{artifact_id}'"
            ));
        };

        for entry in &mut versions[pos].versions {
            if entry.version == active_v {
                entry.status = "rolled_back".to_string();
            } else if entry.version == prev_v {
                entry.status = "active".to_string();
            }
        }
        let kind = versions[pos].kind.clone();
        versions[pos].active_version = prev_v;
        versions[pos].previous_stable_version = None;
        versions[pos].baseline_live_score = prev_entry.replay_score.min(0.85);
        save_versions_unlocked(root_data_dir, &versions)?;

        if let Some(lib) = library {
            if matches!(kind.as_str(), "skill" | "tool" | "subagent")
                && lib
                    .update_item_for_evolution(artifact_id, &prev_entry.content, prev_v, None)
                    .is_ok()
            {
                lib.save(root_data_dir);
            }
        }

        append_log_unlocked(
            root_data_dir,
            &EvolutionLogEntry {
                ts_ms: now_millis(),
                event: "rolled_back".to_string(),
                artifact_id: artifact_id.to_string(),
                kind,
                from_version: active_v,
                to_version: prev_v,
                champion_score: prev_entry.replay_score,
                candidate_score: 0.0,
                detail: format!("manual rollback from v{active_v} to v{prev_v}"),
            },
        )?;

        Ok(json!({
            "ok": true,
            "artifact_id": artifact_id,
            "from_version": active_v,
            "to_version": prev_v,
            "evolution": versions[pos].to_json(),
        }))
    })
}

// ---------------------------------------------------------------------------
// Isolated Sandbox & Replay Grading
// ---------------------------------------------------------------------------

struct SandboxDir {
    root: PathBuf,
    cfg: Config,
}

impl SandboxDir {
    fn new(root_data_dir: &Path) -> Result<Self, String> {
        let seq = SANDBOX_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = root_data_dir
            .join(".evolution_sandbox")
            .join(format!("run-{seq}"));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data_dir");
        let workspace_dir = root.join("workspace");
        std::fs::create_dir_all(&data_dir).map_err(|e| format!("create sandbox data_dir: {e}"))?;
        std::fs::create_dir_all(&workspace_dir)
            .map_err(|e| format!("create sandbox workspace: {e}"))?;
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "evolution-sandbox".to_string(),
            api_key: None,
        };
        Ok(Self { root, cfg })
    }
}

impl Drop for SandboxDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn populate_setup_files(
    workspace_dir: &Path,
    setup_files: &BTreeMap<String, String>,
) -> Result<(), String> {
    for (rel, content) in setup_files {
        if rel.trim().is_empty() {
            return Err("setup_files path cannot be empty".to_string());
        }
        let full = sandbox_path(workspace_dir, rel)
            .map_err(|e| format!("invalid setup_files path '{rel}': {e}"))?;
        if full == workspace_dir {
            return Err(format!("invalid setup_files path '{rel}'"));
        }
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create setup_files dir for '{rel}': {e}"))?;
        }
        std::fs::write(&full, content.as_bytes())
            .map_err(|e| format!("write setup_files '{rel}': {e}"))?;
    }
    Ok(())
}

fn grade_checks(
    checks: &[ReplayCheck],
    ok: bool,
    output: &str,
    workspace_dir: &Path,
) -> Vec<bool> {
    let effective: Vec<ReplayCheck> = if checks.is_empty() {
        vec![ReplayCheck::ExpectOk]
    } else {
        checks.to_vec()
    };
    effective
        .iter()
        .map(|chk| match chk {
            ReplayCheck::ExpectOk => ok,
            ReplayCheck::OutputContains(needle) => output.contains(needle),
            ReplayCheck::OutputNotContains(needle) => !output.contains(needle),
            ReplayCheck::FileContains { path, needle } => sandbox_path(workspace_dir, path)
                .ok()
                .and_then(|full| std::fs::read_to_string(full).ok())
                .is_some_and(|text| text.contains(needle)),
            ReplayCheck::MinLines(min) => output.lines().count() >= *min,
        })
        .collect()
}

fn run_script_case(
    root_data_dir: &Path,
    source: &str,
    params: &[String],
    case: &ReplayCase,
) -> Vec<bool> {
    let Ok(sandbox) = SandboxDir::new(root_data_dir) else {
        return vec![false; case.checks.len().max(1)];
    };
    if populate_setup_files(&sandbox.cfg.workspace_dir, &case.setup_files).is_err() {
        return vec![false; case.checks.len().max(1)];
    }
    let args = if !case.args.is_empty() {
        case.args.clone()
    } else if !params.is_empty() && !case.prompt_or_input.is_empty() {
        vec![case.prompt_or_input.clone()]
    } else {
        Vec::new()
    };
    let (ok, output) = match run_library_skill(&sandbox.cfg, source, &args) {
        Ok(out) => (true, out),
        Err(err) => (false, err),
    };
    grade_checks(&case.checks, ok, &output, &sandbox.cfg.workspace_dir)
}

fn extract_model_output_text(reply: &Value) -> String {
    for key in ["output", "candidate_content", "content", "summary", "answer"] {
        if let Some(s) = reply.get(key).and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                return s.to_string();
            }
        }
    }
    reply.to_string()
}

fn run_subagent_case(
    root_data_dir: &Path,
    cfg: &Config,
    instructions: &str,
    case: &ReplayCase,
) -> Vec<bool> {
    let Ok(sandbox) = SandboxDir::new(root_data_dir) else {
        return vec![false; case.checks.len().max(1)];
    };
    if populate_setup_files(&sandbox.cfg.workspace_dir, &case.setup_files).is_err() {
        return vec![false; case.checks.len().max(1)];
    }
    if instructions.trim().is_empty() {
        return vec![false; case.checks.len().max(1)];
    }

    let (ok, output) = if cfg.model_configured() {
        let system = format!(
            "{instructions}\n\nRespond with a JSON object: {{\"output\": \"<response text>\"}}"
        );
        let user = if !case.prompt_or_input.is_empty() {
            case.prompt_or_input.clone()
        } else {
            case.args.join(" ")
        };
        let messages = vec![
            json!({ "role": "system", "content": system }),
            json!({ "role": "user", "content": user }),
        ];
        match call_model_with_usage(cfg, "ergon-eval-subagent", &messages) {
            Ok((reply, _)) => (true, extract_model_output_text(&reply)),
            Err(err) => (false, err),
        }
    } else {
        (true, instructions.to_string())
    };
    grade_checks(&case.checks, ok, &output, &sandbox.cfg.workspace_dir)
}

fn run_improver_case(
    root_data_dir: &Path,
    cfg: &Config,
    improver_instructions: &str,
    case: &ReplayCase,
) -> Vec<bool> {
    let Ok(sandbox) = SandboxDir::new(root_data_dir) else {
        return vec![false; case.checks.len().max(1)];
    };
    if populate_setup_files(&sandbox.cfg.workspace_dir, &case.setup_files).is_err() {
        return vec![false; case.checks.len().max(1)];
    }
    if improver_instructions.trim().is_empty() {
        return vec![false; case.checks.len().max(1)];
    }

    if cfg.model_configured() {
        let system = format!(
            "{improver_instructions}\n\nNever mention internal scripting engine names. Reply with JSON: {{\"candidate_content\": \"<improved script or instructions>\", \"output\": \"<summary>\"}}"
        );
        let user = json!({
            "prompt_or_input": case.prompt_or_input,
            "args": case.args,
            "setup_files": case.setup_files,
        })
        .to_string();
        let messages = vec![
            json!({ "role": "system", "content": system }),
            json!({ "role": "user", "content": user }),
        ];
        match call_model_with_usage(cfg, "ergon-eval-improver", &messages) {
            Ok((reply, _)) => {
                let candidate = reply
                    .get("candidate_content")
                    .or_else(|| reply.get("source"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !candidate.trim().is_empty() && candidate.contains("fn main") {
                    if check_script(&sandbox.cfg, candidate).is_err() {
                        return vec![false; case.checks.len().max(1)];
                    }
                    let (ok, output) = match run_library_skill(&sandbox.cfg, candidate, &case.args)
                    {
                        Ok(out) => (true, out),
                        Err(err) => (false, err),
                    };
                    return grade_checks(&case.checks, ok, &output, &sandbox.cfg.workspace_dir);
                }
                let text = extract_model_output_text(&reply);
                grade_checks(&case.checks, true, &text, &sandbox.cfg.workspace_dir)
            }
            Err(err) => grade_checks(&case.checks, false, &err, &sandbox.cfg.workspace_dir),
        }
    } else {
        grade_checks(
            &case.checks,
            true,
            improver_instructions,
            &sandbox.cfg.workspace_dir,
        )
    }
}

// ---------------------------------------------------------------------------
// Evaluation & Promotion
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn evaluate_and_maybe_promote(
    root_data_dir: &Path,
    sandbox_root_cfg: &Config,
    library: Option<&mut Library>,
    artifact_id: &str,
    kind: &str,
    candidate_content: &str,
    candidate_params: Vec<String>,
    author: &str,
    note: &str,
) -> Result<EvaluationOutcome, String> {
    let artifact_id = artifact_id.trim();
    if artifact_id.is_empty() {
        return Err("artifact_id cannot be empty".to_string());
    }
    if !valid_kind(kind) {
        return Err(format!("invalid artifact kind '{kind}'"));
    }

    // 1. Seed v1 if not yet tracked, and snapshot champion + replay cases under lock.
    let lib_seed = library.as_deref().and_then(|lib| lib.get(artifact_id));
    let (champion_version, champion_content, champion_params, cases) = with_evolution_lock(|| {
        let mut versions = load_versions_unlocked(root_data_dir);
        if !versions.iter().any(|a| a.id == artifact_id) {
            let (seed_content, seed_params, seed_author, seed_ver, seed_note) =
                if let Some(ref item) = lib_seed {
                    let c = match item.kind.as_str() {
                        "skill" | "tool" => item.source.clone(),
                        "subagent" => item.instructions.clone(),
                        _ => item.template.clone(),
                    };
                    (
                        c,
                        item.params.clone(),
                        item.author.clone(),
                        item.version.max(1),
                        "Seeded from company library".to_string(),
                    )
                } else {
                    (
                        String::new(),
                        candidate_params.clone(),
                        "ergon".to_string(),
                        1,
                        "Seeded empty baseline".to_string(),
                    )
                };
            versions.push(ArtifactEvolution {
                id: artifact_id.to_string(),
                kind: kind.to_string(),
                active_version: seed_ver,
                previous_stable_version: None,
                baseline_live_score: 0.0,
                versions: vec![ArtifactVersionEntry {
                    version: seed_ver,
                    content: seed_content,
                    params: seed_params,
                    created_ms: now_millis(),
                    author: seed_author,
                    note: seed_note,
                    replay_score: 0.0,
                    status: "active".to_string(),
                }],
            });
            save_versions_unlocked(root_data_dir, &versions)?;
        }

        let ev = versions
            .iter()
            .find(|a| a.id == artifact_id)
            .ok_or_else(|| format!("artifact '{artifact_id}' missing after seeding"))?;
        let active_entry = ev
            .active_entry()
            .cloned()
            .ok_or_else(|| format!("active version missing for '{artifact_id}'"))?;
        let all_cases = load_replay_cases_unlocked(root_data_dir);
        let matching_cases: Vec<ReplayCase> = all_cases
            .into_iter()
            .filter(|c| {
                c.artifact_id == artifact_id
                    && (c.artifact_kind.is_empty() || c.artifact_kind == kind)
            })
            .collect();
        Ok::<_, String>((
            ev.active_version,
            active_entry.content,
            active_entry.params,
            matching_cases,
        ))
    })?;

    // 2. Compile / validity check outside the lock.
    let compile_res: Result<(), String> = if matches!(kind, "tool" | "skill") {
        if candidate_content.trim().is_empty() {
            Err("candidate script cannot be empty".to_string())
        } else if candidate_params.len() > 5 {
            Err("scripts take at most 5 parameters".to_string())
        } else {
            let compile_sandbox = SandboxDir::new(root_data_dir)?;
            check_script(&compile_sandbox.cfg, candidate_content).map_err(|e| sanitize_no_jargon(&e))
        }
    } else if candidate_content.trim().is_empty() {
        Err(format!("{kind} candidate content cannot be empty"))
    } else {
        Ok(())
    };

    if let Err(err_msg) = compile_res {
        let rejected_ver = with_evolution_lock(|| {
            let mut versions = load_versions_unlocked(root_data_dir);
            let pos = versions
                .iter()
                .position(|a| a.id == artifact_id)
                .ok_or_else(|| format!("artifact '{artifact_id}' missing"))?;
            let next_v = versions[pos]
                .versions
                .iter()
                .map(|v| v.version)
                .max()
                .unwrap_or(champion_version)
                + 1;
            versions[pos].versions.push(ArtifactVersionEntry {
                version: next_v,
                content: candidate_content.to_string(),
                params: candidate_params.clone(),
                created_ms: now_millis(),
                author: author.to_string(),
                note: format!("{note} (compile rejected: {})", truncate(&err_msg, 180)),
                replay_score: 0.0,
                status: "rejected".to_string(),
            });
            save_versions_unlocked(root_data_dir, &versions)?;
            append_log_unlocked(
                root_data_dir,
                &EvolutionLogEntry {
                    ts_ms: now_millis(),
                    event: "rejected".to_string(),
                    artifact_id: artifact_id.to_string(),
                    kind: kind.to_string(),
                    from_version: champion_version,
                    to_version: next_v,
                    champion_score: 0.0,
                    candidate_score: 0.0,
                    detail: format!("compile check failed: {}", truncate(&err_msg, 300)),
                },
            )?;
            Ok::<_, String>(next_v)
        })?;

        return Ok(EvaluationOutcome {
            artifact_id: artifact_id.to_string(),
            kind: kind.to_string(),
            promoted: false,
            from_version: champion_version,
            to_version: rejected_ver,
            candidate_compiled: false,
            champion_score: 0.0,
            candidate_score: 0.0,
            regressions: 0,
            cases_run: 0,
            detail: format!("rejected: {err_msg}"),
        });
    }

    if cases.is_empty() {
        let rejected_ver = with_evolution_lock(|| {
            let mut versions = load_versions_unlocked(root_data_dir);
            let pos = versions
                .iter()
                .position(|a| a.id == artifact_id)
                .ok_or_else(|| format!("artifact '{artifact_id}' missing"))?;
            let next_v = versions[pos]
                .versions
                .iter()
                .map(|v| v.version)
                .max()
                .unwrap_or(champion_version)
                + 1;
            versions[pos].versions.push(ArtifactVersionEntry {
                version: next_v,
                content: candidate_content.to_string(),
                params: candidate_params.clone(),
                created_ms: now_millis(),
                author: author.to_string(),
                note: format!("{note} (no replay cases)"),
                replay_score: 0.0,
                status: "rejected".to_string(),
            });
            save_versions_unlocked(root_data_dir, &versions)?;
            append_log_unlocked(
                root_data_dir,
                &EvolutionLogEntry {
                    ts_ms: now_millis(),
                    event: "rejected".to_string(),
                    artifact_id: artifact_id.to_string(),
                    kind: kind.to_string(),
                    from_version: champion_version,
                    to_version: next_v,
                    champion_score: 0.0,
                    candidate_score: 0.0,
                    detail: "no replay cases recorded; at least 1 replay case required".to_string(),
                },
            )?;
            Ok::<_, String>(next_v)
        })?;
        return Ok(EvaluationOutcome {
            artifact_id: artifact_id.to_string(),
            kind: kind.to_string(),
            promoted: false,
            from_version: champion_version,
            to_version: rejected_ver,
            candidate_compiled: true,
            champion_score: 0.0,
            candidate_score: 0.0,
            regressions: 0,
            cases_run: 0,
            detail: "rejected: no replay cases ran".to_string(),
        });
    }

    // 3. Evaluate champion vs candidate on all replay cases outside the lock.
    let mut champ_case_scores = Vec::with_capacity(cases.len());
    let mut cand_case_scores = Vec::with_capacity(cases.len());
    let mut regressions: usize = 0;

    for case in &cases {
        let (champ_checks, cand_checks) = match kind {
            "tool" | "skill" => (
                run_script_case(root_data_dir, &champion_content, &champion_params, case),
                run_script_case(root_data_dir, candidate_content, &candidate_params, case),
            ),
            "subagent" => (
                run_subagent_case(root_data_dir, sandbox_root_cfg, &champion_content, case),
                run_subagent_case(root_data_dir, sandbox_root_cfg, candidate_content, case),
            ),
            "improver" => (
                run_improver_case(root_data_dir, sandbox_root_cfg, &champion_content, case),
                run_improver_case(root_data_dir, sandbox_root_cfg, candidate_content, case),
            ),
            _ => unreachable!(),
        };

        let check_len = champ_checks.len().max(cand_checks.len()).max(1);
        for i in 0..check_len {
            let c_ok = champ_checks.get(i).copied().unwrap_or(false);
            let k_ok = cand_checks.get(i).copied().unwrap_or(false);
            if c_ok && !k_ok {
                regressions += 1;
            }
        }
        let c_pass = champ_checks.iter().filter(|&&b| b).count() as f64;
        let k_pass = cand_checks.iter().filter(|&&b| b).count() as f64;
        champ_case_scores.push(c_pass / (champ_checks.len().max(1) as f64));
        cand_case_scores.push(k_pass / (cand_checks.len().max(1) as f64));
    }

    let cases_run = cases.len();
    let champion_score = champ_case_scores.iter().sum::<f64>() / (cases_run as f64);
    let candidate_score = cand_case_scores.iter().sum::<f64>() / (cases_run as f64);
    let should_promote =
        cases_run > 0 && regressions == 0 && candidate_score > champion_score + 0.01;

    // 4. Persist promotion or rejection under lock.
    let (to_version, detail) = with_evolution_lock(|| {
        let mut versions = load_versions_unlocked(root_data_dir);
        let pos = versions
            .iter()
            .position(|a| a.id == artifact_id)
            .ok_or_else(|| format!("artifact '{artifact_id}' missing"))?;
        let next_v = versions[pos]
            .versions
            .iter()
            .map(|v| v.version)
            .max()
            .unwrap_or(champion_version)
            + 1;

        if should_promote {
            for entry in &mut versions[pos].versions {
                if entry.version == champion_version {
                    entry.status = "superseded".to_string();
                    entry.replay_score = champion_score;
                }
            }
            versions[pos].versions.push(ArtifactVersionEntry {
                version: next_v,
                content: candidate_content.to_string(),
                params: candidate_params.clone(),
                created_ms: now_millis(),
                author: author.to_string(),
                note: note.to_string(),
                replay_score: candidate_score,
                status: "active".to_string(),
            });
            versions[pos].previous_stable_version = Some(champion_version);
            versions[pos].active_version = next_v;
            versions[pos].baseline_live_score = candidate_score.min(0.85);
            save_versions_unlocked(root_data_dir, &versions)?;

            if let Some(lib) = library {
                if matches!(kind, "skill" | "tool" | "subagent") {
                    let report = json!({
                        "source_sha256": sha256_hex(candidate_content.as_bytes()),
                        "champion_score": champion_score,
                        "candidate_score": candidate_score,
                        "cases_run": cases_run,
                        "regressions": regressions,
                        "promoted_at_ms": now_millis(),
                    });
                    if lib
                        .update_item_for_evolution(
                            artifact_id,
                            candidate_content,
                            next_v,
                            Some(report),
                        )
                        .is_ok()
                    {
                        lib.save(root_data_dir);
                    }
                }
            }

            let detail = format!(
                "promoted v{champion_version} -> v{next_v} (champion={champion_score:.3}, candidate={candidate_score:.3}, cases={cases_run}, regressions=0)"
            );
            append_log_unlocked(
                root_data_dir,
                &EvolutionLogEntry {
                    ts_ms: now_millis(),
                    event: "promoted".to_string(),
                    artifact_id: artifact_id.to_string(),
                    kind: kind.to_string(),
                    from_version: champion_version,
                    to_version: next_v,
                    champion_score,
                    candidate_score,
                    detail: detail.clone(),
                },
            )?;
            Ok::<_, String>((next_v, detail))
        } else {
            versions[pos].versions.push(ArtifactVersionEntry {
                version: next_v,
                content: candidate_content.to_string(),
                params: candidate_params,
                created_ms: now_millis(),
                author: author.to_string(),
                note: note.to_string(),
                replay_score: candidate_score,
                status: "rejected".to_string(),
            });
            save_versions_unlocked(root_data_dir, &versions)?;
            let detail = format!(
                "rejected v{next_v} vs champion v{champion_version} (champion={champion_score:.3}, candidate={candidate_score:.3}, regressions={regressions}, cases={cases_run})"
            );
            append_log_unlocked(
                root_data_dir,
                &EvolutionLogEntry {
                    ts_ms: now_millis(),
                    event: "rejected".to_string(),
                    artifact_id: artifact_id.to_string(),
                    kind: kind.to_string(),
                    from_version: champion_version,
                    to_version: next_v,
                    champion_score,
                    candidate_score,
                    detail: detail.clone(),
                },
            )?;
            Ok::<_, String>((next_v, detail))
        }
    })?;

    Ok(EvaluationOutcome {
        artifact_id: artifact_id.to_string(),
        kind: kind.to_string(),
        promoted: should_promote,
        from_version: champion_version,
        to_version,
        candidate_compiled: true,
        champion_score,
        candidate_score,
        regressions,
        cases_run,
        detail,
    })
}

pub fn propose_improvement_with_model(
    cfg: &Config,
    root_data_dir: &Path,
    library: Option<&mut Library>,
    artifact_id: &str,
) -> Result<EvaluationOutcome, String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let artifact_id = artifact_id.trim();
    if artifact_id.is_empty() {
        return Err("artifact_id cannot be empty".to_string());
    }

    let lib_item = library.as_deref().and_then(|l| l.get(artifact_id));
    let (improver_instructions, kind, active_version, current_content, params, recent_outcomes, cases) =
        with_evolution_lock(|| {
            let mut versions = load_versions_unlocked(root_data_dir);
            if !versions.iter().any(|a| a.id == artifact_id) {
                if let Some(ref item) = lib_item {
                    let c = match item.kind.as_str() {
                        "skill" | "tool" => item.source.clone(),
                        "subagent" => item.instructions.clone(),
                        _ => item.template.clone(),
                    };
                    versions.push(ArtifactEvolution {
                        id: artifact_id.to_string(),
                        kind: item.kind.clone(),
                        active_version: item.version.max(1),
                        previous_stable_version: None,
                        baseline_live_score: 0.0,
                        versions: vec![ArtifactVersionEntry {
                            version: item.version.max(1),
                            content: c,
                            params: item.params.clone(),
                            created_ms: now_millis(),
                            author: item.author.clone(),
                            note: "Seeded from company library".to_string(),
                            replay_score: 0.0,
                            status: "active".to_string(),
                        }],
                    });
                    save_versions_unlocked(root_data_dir, &versions)?;
                } else {
                    return Err(format!(
                        "artifact '{artifact_id}' not found in evolution store or library"
                    ));
                }
            }

            let improver_instructions = versions
                .iter()
                .find(|a| a.id == DEFAULT_IMPROVER_ID)
                .and_then(|a| a.active_entry())
                .map(|e| e.content.clone())
                .unwrap_or_else(|| DEFAULT_IMPROVER_INSTRUCTIONS.to_string());

            let ev = versions
                .iter()
                .find(|a| a.id == artifact_id)
                .ok_or_else(|| format!("artifact '{artifact_id}' not found"))?;
            let active = ev
                .active_entry()
                .ok_or_else(|| format!("active version missing for '{artifact_id}'"))?;

            let outcomes = load_outcomes_unlocked(root_data_dir);
            let mut relevant: Vec<Value> = outcomes
                .into_iter()
                .filter(|o| o.artifact_id == artifact_id)
                .map(|o| {
                    json!({
                        "version": o.artifact_version,
                        "task": truncate(&o.task, 200),
                        "approved": o.approved,
                        "edits_needed": o.edits_needed,
                        "reruns": o.reruns,
                        "quality_score": o.quality_score,
                        "feedback": o.feedback,
                    })
                })
                .collect();
            if relevant.len() > 10 {
                let start = relevant.len() - 10;
                relevant = relevant.split_off(start);
            }

            let all_cases = load_replay_cases_unlocked(root_data_dir);
            let cases: Vec<Value> = all_cases
                .into_iter()
                .filter(|c| c.artifact_id == artifact_id)
                .map(|c| c.to_json())
                .collect();

            Ok((
                improver_instructions,
                ev.kind.clone(),
                ev.active_version,
                active.content.clone(),
                active.params.clone(),
                relevant,
                cases,
            ))
        })?;

    let system = format!(
        "{improver_instructions}\n\
         Rules:\n\
         - Never mention internal scripting engine names.\n\
         - Preserve existing passing behavior while fixing reported failures, edits, and replay checks.\n\
         - If artifact kind is 'tool' or 'skill', keep `pub fn main(...)` taking the exact declared params in order and using sandboxed host helpers (`host_workspace_read`, `host_workspace_write`, `host_workspace_list`).\n\
         - Reply with strictly valid JSON: {{\"candidate_content\": \"...\", \"note\": \"concise summary of the improvement\"}}"
    );
    let user = json!({
        "artifact_id": artifact_id,
        "kind": kind,
        "active_version": active_version,
        "params": params,
        "current_content": current_content,
        "recent_outcomes": recent_outcomes,
        "replay_cases": cases,
    })
    .to_string();

    let messages = vec![
        json!({ "role": "system", "content": system }),
        json!({ "role": "user", "content": user }),
    ];

    let (reply, _) = call_model_with_usage(cfg, "ergon-self-improve", &messages)?;
    let candidate_content = reply
        .get("candidate_content")
        .or_else(|| reply.get("content"))
        .or_else(|| reply.get("source"))
        .or_else(|| reply.get("instructions"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "model reply did not include candidate_content".to_string())?
        .to_string();
    let note = reply
        .get("note")
        .or_else(|| reply.get("summary"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Model-proposed improvement")
        .to_string();

    with_evolution_lock(|| {
        append_log_unlocked(
            root_data_dir,
            &EvolutionLogEntry {
                ts_ms: now_millis(),
                event: "proposed".to_string(),
                artifact_id: artifact_id.to_string(),
                kind: kind.clone(),
                from_version: active_version,
                to_version: active_version + 1,
                champion_score: 0.0,
                candidate_score: 0.0,
                detail: note.clone(),
            },
        )
    })?;

    evaluate_and_maybe_promote(
        root_data_dir,
        cfg,
        library,
        artifact_id,
        &kind,
        &candidate_content,
        params,
        "ergon-self-improve",
        &note,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::seed_items;
    use std::sync::atomic::AtomicU64;

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_env(label: &str) -> (PathBuf, PathBuf, Config) {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ergon-self-improve-{}-{}-{label}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let root_data = root.join("root_data");
        let person_dir = root_data.join("people").join("alice");
        let cfg = Config {
            data_dir: person_dir.clone(),
            workspace_dir: person_dir.join("workspace"),
            memory_path: person_dir.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test-offline".to_string(),
            api_key: None,
        };
        std::fs::create_dir_all(&root_data).unwrap();
        cfg.ensure_dirs().unwrap();
        (root, root_data, cfg)
    }

    fn make_outcome(
        artifact_id: &str,
        kind: &str,
        version: u32,
        run_id: &str,
        approved: Option<bool>,
        edits: u32,
    ) -> OutcomeRecord {
        OutcomeRecord {
            id: String::new(),
            ts_ms: 0,
            person_id: "alice".to_string(),
            artifact_kind: kind.to_string(),
            artifact_id: artifact_id.to_string(),
            artifact_version: version,
            run_id: run_id.to_string(),
            task: "Format summary".to_string(),
            approved,
            edits_needed: edits,
            reruns: 0,
            time_saved_mins: 0.0,
            explicit_score: None,
            quality_score: 0.0,
            feedback: None,
        }
    }

    #[test]
    fn quality_score_formula_is_deterministic_and_bounded() {
        let perfect = compute_quality_score(Some(true), 0, 0, 30.0, None);
        assert!((perfect - 1.0).abs() < 1e-9, "got {perfect}");

        let neutral = compute_quality_score(None, 0, 0, 0.0, None);
        assert!((neutral - 0.70).abs() < 1e-9, "got {neutral}");

        let edited = compute_quality_score(Some(true), 2, 1, 0.0, None);
        // 0.85 - 0.30 - 0.10 = 0.45
        assert!((edited - 0.45).abs() < 1e-9, "got {edited}");

        let blended = compute_quality_score(Some(false), 4, 4, 0.0, Some(1.0));
        // computed = 0.15 - 0.60 - 0.40 = -0.85 -> 0.6*(-0.85) + 0.4*(1.0) = -0.11 -> clamped 0.0
        assert!((blended - 0.0).abs() < 1e-9, "got {blended}");
    }

    #[test]
    fn tool_v2_beats_v1_promoted_and_worse_v3_rejected_and_live_regression_rolls_back() {
        let (root, root_data, cfg) = temp_env("promote-reject-rollback");
        let mut lib = Library {
            items: seed_items(),
            recovered: None,
        };

        let v1_src = r#"pub fn main(name) {
    `hello ${name}`
}"#;
        let v2_src = r#"pub fn main(name) {
    let out = `hello ${name} - verified`;
    let res = host_workspace_write("greeting.txt", out.clone());
    if res != "ok" {
        return res;
    }
    out
}"#;
        let v3_worse_src = r#"pub fn main(name) {
    if name == "" {
        return "broken";
    }
    "broken output"
}"#;

        lib.publish_tool(
            "Greeter Tool",
            "Writes greeting",
            vec!["name".to_string()],
            v1_src.to_string(),
            vec![],
            json!({"passed": 1, "total": 1}),
            "alice",
        )
        .unwrap();

        ensure_artifact_seeded(
            &root_data,
            "greeter-tool",
            "tool",
            v1_src,
            vec!["name".to_string()],
            "alice",
        );

        add_replay_case(
            &root_data,
            ReplayCase {
                case_id: "greet-1".to_string(),
                artifact_id: "greeter-tool".to_string(),
                artifact_kind: "tool".to_string(),
                setup_files: BTreeMap::new(),
                args: vec!["Alice".to_string()],
                prompt_or_input: String::new(),
                checks: vec![
                    ReplayCheck::ExpectOk,
                    ReplayCheck::OutputContains("hello Alice".to_string()),
                    ReplayCheck::OutputContains("verified".to_string()),
                    ReplayCheck::FileContains {
                        path: "greeting.txt".to_string(),
                        needle: "verified".to_string(),
                    },
                ],
            },
        )
        .unwrap();

        // v2 beats v1 -> promoted
        let eval_v2 = evaluate_and_maybe_promote(
            &root_data,
            &cfg,
            Some(&mut lib),
            "greeter-tool",
            "tool",
            v2_src,
            vec!["name".to_string()],
            "alice",
            "Add verified tag and output file",
        )
        .unwrap();
        assert!(eval_v2.promoted, "{eval_v2:?}");
        assert_eq!(eval_v2.from_version, 1);
        assert_eq!(eval_v2.to_version, 2);
        assert_eq!(eval_v2.regressions, 0);
        assert!(eval_v2.candidate_score > eval_v2.champion_score + 0.01);
        assert_eq!(lib.get("greeter-tool").unwrap().version, 2);

        // Worse v3 -> rejected (champion remains v2)
        let eval_v3 = evaluate_and_maybe_promote(
            &root_data,
            &cfg,
            Some(&mut lib),
            "greeter-tool",
            "tool",
            v3_worse_src,
            vec!["name".to_string()],
            "alice",
            "Broken regression",
        )
        .unwrap();
        assert!(!eval_v3.promoted, "{eval_v3:?}");
        assert!(eval_v3.regressions > 0);
        assert_eq!(lib.get("greeter-tool").unwrap().version, 2);

        // Live regression on v2 -> auto-rollback to v1
        let o1 = record_outcome(
            &root_data,
            Some(&mut lib),
            make_outcome("greeter-tool", "tool", 2, "run-1", Some(false), 1),
        );
        assert!(o1.rolled_back_to.is_none());

        let o2 = record_outcome(
            &root_data,
            Some(&mut lib),
            make_outcome("greeter-tool", "tool", 2, "run-2", Some(false), 1),
        );
        assert_eq!(o2.rolled_back_to, Some(1));
        assert_eq!(lib.get("greeter-tool").unwrap().version, 1);
        assert!(lib.get("greeter-tool").unwrap().source.contains("hello ${name}"));

        // Verify increment_edits_for_run updates quality score on matching run
        let edit_updates = increment_edits_for_run(&root_data, Some(&mut lib), "alice", "run-1");
        assert_eq!(edit_updates.len(), 1);
        assert_eq!(edit_updates[0].record.edits_needed, 2);

        let summary = score_summary(&root_data);
        assert!(summary["artifacts"].as_array().unwrap().len() >= 2);
        let history = artifact_history(&root_data, "greeter-tool");
        assert_eq!(history["evolution"]["active_version"], json!(1));
        assert!(history["evolution_log"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event"] == "auto_rolled_back"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn improver_v2_beats_v1_and_is_promoted() {
        let (root, root_data, cfg) = temp_env("improver-evo");

        // Ensure default-improver v1 is seeded and add a replay case for it.
        add_replay_case(
            &root_data,
            ReplayCase {
                case_id: "imp-case-1".to_string(),
                artifact_id: DEFAULT_IMPROVER_ID.to_string(),
                artifact_kind: "improver".to_string(),
                setup_files: BTreeMap::new(),
                args: Vec::new(),
                prompt_or_input: "Synthesize safe improvement guidance".to_string(),
                checks: vec![
                    ReplayCheck::ExpectOk,
                    ReplayCheck::OutputContains("edge cases".to_string()),
                    ReplayCheck::OutputContains("passing tests".to_string()),
                    ReplayCheck::MinLines(2),
                ],
            },
        )
        .unwrap();

        let v2_instructions = concat!(
            "Analyze failed checks and user edits, handle edge cases explicitly, ",
            "and preserve all existing passing tests.\n",
            "Keep changes minimal and verify every workspace path before returning."
        );

        let res = evaluate_and_maybe_promote(
            &root_data,
            &cfg,
            None,
            DEFAULT_IMPROVER_ID,
            "improver",
            v2_instructions,
            Vec::new(),
            "alice",
            "Add explicit edge-case and multi-line guidance",
        )
        .unwrap();

        assert!(res.promoted, "{res:?}");
        assert_eq!(res.from_version, 1);
        assert_eq!(res.to_version, 2);
        assert_eq!(res.regressions, 0);
        assert!(res.candidate_score > res.champion_score + 0.01);

        let history = artifact_history(&root_data, DEFAULT_IMPROVER_ID);
        assert_eq!(history["evolution"]["active_version"], json!(2));

        let _ = std::fs::remove_dir_all(&root);
    }
}
