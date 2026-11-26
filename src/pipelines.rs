//! Resumable Hybrid Workflow & Pipeline Engine (`pipeline_runs.json`).
//!
//! Executes typed multi-step pipelines (`skill`, `agent`, `checkpoint`, `export`)
//! asynchronously on worker threads, persists step-by-step state atomically,
//! pauses cleanly at human checkpoints (`paused_checkpoint`), survives server
//! restarts, and resumes downstream steps with `{{steps.<id>.human_answer}}`
//! and `{{steps.<id>.files[0]}}` template resolution.

use crate::records;
use crate::routines;
use crate::search::SearchCache;
use crate::skills;
use crate::sse::RunEventBus;
use crate::tools::{self, sandbox_path, ToolCtx};
use crate::{atomic_write_bytes, run_agent_for_pipeline, CheckpointPause, Config, LiveRuns, SharedStore};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const MAX_PIPELINE_STEPS: usize = 24;
const MAX_STORED_RUNS: usize = 200;
pub const MAX_TEMPLATE_DEPTH: usize = 16;
pub const MAX_TEMPLATE_EXPANSIONS: usize = 128;
pub const MAX_TEMPLATE_OUTPUT_BYTES: usize = 256 * 1024;

static PIPELINE_SEQ: AtomicU64 = AtomicU64::new(0);
static PIPELINES_LOCK: Mutex<()> = Mutex::new(());
static ACTIVE_CANCELS: std::sync::OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    std::sync::OnceLock::new();
static RECOVERED_DIRS: std::sync::OnceLock<Mutex<HashSet<PathBuf>>> =
    std::sync::OnceLock::new();

fn with_pipelines_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = PIPELINES_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

fn cancel_registry() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    ACTIVE_CANCELS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn recovered_dirs() -> &'static Mutex<HashSet<PathBuf>> {
    RECOVERED_DIRS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn pipeline_runs_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("pipeline_runs.json")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineCheckpointSpec {
    pub prompt: String,
    pub input_mode: String, // "approve_reject" | "choice" | "text"
    pub choices: Vec<String>,
    pub inspect_files: Vec<String>,
}

impl PipelineCheckpointSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "prompt": self.prompt,
            "input_mode": self.input_mode,
            "choices": self.choices,
            "inspect_files": self.inspect_files,
        })
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let prompt = v["prompt"]
            .as_str()
            .unwrap_or("")
            .trim()
            .chars()
            .take(2000)
            .collect::<String>();
        if prompt.is_empty() {
            return Err("checkpoint.prompt is required".to_string());
        }
        let raw_mode = v["input_mode"]
            .as_str()
            .unwrap_or("approve_reject")
            .trim()
            .to_ascii_lowercase();
        let input_mode = match raw_mode.as_str() {
            "" | "approve_reject" => "approve_reject".to_string(),
            "choice" | "text" => raw_mode,
            other => {
                return Err(format!(
                    "checkpoint.input_mode '{other}' must be approve_reject, choice, or text"
                ))
            }
        };
        let choices: Vec<String> = v["choices"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str())
                    .map(|s| s.trim().chars().take(200).collect::<String>())
                    .filter(|s| !s.is_empty())
                    .take(20)
                    .collect()
            })
            .unwrap_or_default();
        if input_mode == "choice" && choices.is_empty() {
            return Err("checkpoint with input_mode='choice' requires choices".to_string());
        }
        let inspect_files: Vec<String> = v["inspect_files"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str())
                    .map(|s| s.trim().chars().take(256).collect::<String>())
                    .filter(|s| !s.is_empty())
                    .take(16)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            prompt,
            input_mode,
            choices,
            inspect_files,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineExportSpec {
    pub path: String,
    pub format: String, // "docx" | "xlsx"
    pub output: Option<String>,
}

impl PipelineExportSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "format": self.format,
            "output": self.output,
        })
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let path = v["path"]
            .as_str()
            .unwrap_or("")
            .trim()
            .chars()
            .take(512)
            .collect::<String>();
        if path.is_empty() {
            return Err("export.path is required".to_string());
        }
        let format = v["format"]
            .as_str()
            .unwrap_or("")
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let output = v["output"]
            .as_str()
            .map(|s| s.trim().chars().take(512).collect::<String>())
            .filter(|s| !s.is_empty());
        Ok(Self {
            path,
            format,
            output,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineStepSpec {
    pub id: String,
    pub label: String,
    pub kind: String, // "skill" | "agent" | "checkpoint" | "export"
    pub skill_id: Option<String>,
    pub args: Value,
    pub prompt: Option<String>,
    pub checkpoint: Option<PipelineCheckpointSpec>,
    pub export: Option<PipelineExportSpec>,
    pub when: String, // "always" | "on_anomaly" | "on_previous_ok"
}

impl PipelineStepSpec {
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), json!(self.id));
        obj.insert("label".to_string(), json!(self.label));
        obj.insert("kind".to_string(), json!(self.kind));
        if let Some(sk) = &self.skill_id {
            obj.insert("skill_id".to_string(), json!(sk));
        }
        if self.args.is_object() {
            obj.insert("args".to_string(), self.args.clone());
        }
        if let Some(p) = &self.prompt {
            obj.insert("prompt".to_string(), json!(p));
        }
        if let Some(cp) = &self.checkpoint {
            obj.insert("checkpoint".to_string(), cp.to_json());
        }
        if let Some(ex) = &self.export {
            obj.insert("export".to_string(), ex.to_json());
        }
        obj.insert("when".to_string(), json!(self.when));
        Value::Object(obj)
    }

    pub fn from_json(v: &Value, index: usize) -> Result<Self, String> {
        let Some(obj) = v.as_object() else {
            return Err(format!("steps_v2[{index}] must be a JSON object"));
        };
        let raw_kind = obj
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("agent")
            .trim()
            .to_ascii_lowercase();
        if !matches!(raw_kind.as_str(), "skill" | "agent" | "checkpoint" | "export") {
            return Err(format!(
                "steps_v2[{index}].kind '{raw_kind}' must be skill, agent, checkpoint, or export"
            ));
        }
        let raw_id = obj
            .get("id")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim();
        let id = if raw_id.is_empty() {
            format!("step_{}", index + 1)
        } else {
            raw_id
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                .take(64)
                .collect::<String>()
        };
        if id.is_empty() {
            return Err(format!("steps_v2[{index}].id contains no valid characters"));
        }
        let skill_id = obj
            .get("skill_id")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().chars().take(64).collect::<String>())
            .filter(|s| !s.is_empty());
        let args = match obj.get("args") {
            None | Some(Value::Null) => json!({}),
            Some(a) if a.is_object() => a.clone(),
            Some(_) => return Err(format!("steps_v2[{index}].args must be an object")),
        };
        let prompt = obj
            .get("prompt")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().chars().take(4000).collect::<String>())
            .filter(|s| !s.is_empty());
        let checkpoint = match obj.get("checkpoint") {
            None | Some(Value::Null) => None,
            Some(cp_val) => Some(PipelineCheckpointSpec::from_json(cp_val)?),
        };
        let export = match obj.get("export") {
            None | Some(Value::Null) => None,
            Some(ex_val) => Some(PipelineExportSpec::from_json(ex_val)?),
        };

        // Per-kind structural validation
        match raw_kind.as_str() {
            "skill" if skill_id.is_none() => {
                return Err(format!("steps_v2[{index}] of kind 'skill' requires skill_id"));
            }
            "checkpoint" if checkpoint.is_none() && prompt.is_none() => {
                return Err(format!(
                    "steps_v2[{index}] of kind 'checkpoint' requires checkpoint.prompt"
                ));
            }
            "export" if export.is_none() => {
                return Err(format!(
                    "steps_v2[{index}] of kind 'export' requires export.path"
                ));
            }
            "agent" if prompt.is_none() => {
                // Allow label as fallback prompt only if non-empty
                let lbl = obj
                    .get("label")
                    .and_then(|s| s.as_str())
                    .map(str::trim)
                    .unwrap_or("");
                if lbl.is_empty() {
                    return Err(format!(
                        "steps_v2[{index}] of kind 'agent' requires prompt or label"
                    ));
                }
            }
            _ => {}
        }

        let label = obj
            .get("label")
            .and_then(|s| s.as_str())
            .map(|s| s.replace(['\r', '\n'], " ").trim().chars().take(160).collect::<String>())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| match raw_kind.as_str() {
                "skill" => skill_id.clone().unwrap_or_else(|| id.clone()),
                "checkpoint" => checkpoint
                    .as_ref()
                    .map(|c| c.prompt.chars().take(80).collect())
                    .unwrap_or_else(|| "Human Checkpoint".to_string()),
                "export" => export
                    .as_ref()
                    .map(|e| format!("Export {}", e.path))
                    .unwrap_or_else(|| "Export".to_string()),
                _ => prompt
                    .as_ref()
                    .map(|p| p.chars().take(80).collect())
                    .unwrap_or_else(|| id.clone()),
            });

        let raw_when = obj
            .get("when")
            .and_then(|s| s.as_str())
            .unwrap_or("always")
            .trim()
            .to_ascii_lowercase();
        let when = match raw_when.as_str() {
            "" | "always" => "always".to_string(),
            "on_anomaly" | "on_previous_ok" => raw_when,
            other => {
                return Err(format!(
                    "steps_v2[{index}].when '{other}' must be always, on_anomaly, or on_previous_ok"
                ))
            }
        };

        Ok(Self {
            id,
            label,
            kind: raw_kind,
            skill_id,
            args,
            prompt,
            checkpoint,
            export,
            when,
        })
    }
}

pub fn parse_steps_v2(v: &Value) -> Result<Vec<PipelineStepSpec>, String> {
    let Some(arr) = v.as_array() else {
        return Err("steps_v2 must be an array".to_string());
    };
    if arr.is_empty() {
        return Err("steps_v2 must contain at least one step".to_string());
    }
    if arr.len() > MAX_PIPELINE_STEPS {
        return Err(format!(
            "steps_v2 exceeds maximum of {MAX_PIPELINE_STEPS} steps"
        ));
    }
    let mut seen_ids = HashSet::new();
    let mut out = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        let mut step = PipelineStepSpec::from_json(item, idx)?;
        if !seen_ids.insert(step.id.clone()) {
            step.id = format!("{}_{}", step.id, idx + 1);
            seen_ids.insert(step.id.clone());
        }
        out.push(step);
    }
    Ok(out)
}

pub fn from_legacy_steps(steps: &[String], prompt: &str) -> Vec<PipelineStepSpec> {
    let items: Vec<String> = steps
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let effective = if items.is_empty() && !prompt.trim().is_empty() {
        vec![prompt.trim().to_string()]
    } else {
        items
    };
    effective
        .into_iter()
        .take(MAX_PIPELINE_STEPS)
        .enumerate()
        .map(|(idx, p)| PipelineStepSpec {
            id: format!("step_{}", idx + 1),
            label: p.chars().take(80).collect(),
            kind: "agent".to_string(),
            skill_id: None,
            args: json!({}),
            prompt: Some(p),
            checkpoint: None,
            export: None,
            when: "always".to_string(),
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineStepState {
    pub step_id: String,
    pub label: String,
    pub kind: String,
    pub status: String, // "pending" | "running" | "ok" | "waiting_human" | "failed" | "skipped"
    pub started_ms: u64,
    pub finished_ms: u64,
    pub output: String,
    pub files: Vec<String>,
    pub metrics: Value,
    pub usage: Option<records::RunUsage>,
    pub checkpoint_id: Option<String>,
    pub human_answer: Option<String>,
    pub human_action: Option<String>,
    pub error: Option<String>,
}

impl PipelineStepState {
    pub fn pending_for(step: &PipelineStepSpec) -> Self {
        Self {
            step_id: step.id.clone(),
            label: step.label.clone(),
            kind: step.kind.clone(),
            status: "pending".to_string(),
            started_ms: 0,
            finished_ms: 0,
            output: String::new(),
            files: Vec::new(),
            metrics: json!({}),
            usage: None,
            checkpoint_id: None,
            human_answer: None,
            human_action: None,
            error: None,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "step_id": self.step_id,
            "label": self.label,
            "kind": self.kind,
            "status": self.status,
            "started_ms": self.started_ms,
            "finished_ms": self.finished_ms,
            "output": self.output,
            "files": self.files,
            "metrics": self.metrics,
            "usage": self.usage.as_ref().map(|u| u.to_json()),
            "checkpoint_id": self.checkpoint_id,
            "human_answer": self.human_answer,
            "human_action": self.human_action,
            "error": self.error,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            step_id: v["step_id"].as_str()?.to_string(),
            label: v["label"].as_str().unwrap_or("").to_string(),
            kind: v["kind"].as_str().unwrap_or("agent").to_string(),
            status: v["status"].as_str().unwrap_or("pending").to_string(),
            started_ms: v["started_ms"].as_u64().unwrap_or(0),
            finished_ms: v["finished_ms"].as_u64().unwrap_or(0),
            output: v["output"].as_str().unwrap_or("").to_string(),
            files: v["files"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            metrics: if v["metrics"].is_object() {
                v["metrics"].clone()
            } else {
                json!({})
            },
            usage: v.get("usage").and_then(records::RunUsage::from_json),
            checkpoint_id: v["checkpoint_id"].as_str().map(str::to_string),
            human_answer: v["human_answer"].as_str().map(str::to_string),
            human_action: v["human_action"].as_str().map(str::to_string),
            error: v["error"].as_str().map(str::to_string),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingCheckpoint {
    pub checkpoint_id: String,
    pub step_index: usize,
    pub step_id: String,
    pub prompt: String,
    pub input_mode: String,
    pub choices: Vec<String>,
    pub inspect_files: Vec<String>,
    pub created_ms: u64,
}

impl PendingCheckpoint {
    pub fn to_json(&self) -> Value {
        json!({
            "checkpoint_id": self.checkpoint_id,
            "step_index": self.step_index,
            "step_id": self.step_id,
            "prompt": self.prompt,
            "input_mode": self.input_mode,
            "choices": self.choices,
            "inspect_files": self.inspect_files,
            "created_ms": self.created_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            checkpoint_id: v["checkpoint_id"].as_str()?.to_string(),
            step_index: v["step_index"].as_u64()? as usize,
            step_id: v["step_id"].as_str().unwrap_or("").to_string(),
            prompt: v["prompt"].as_str().unwrap_or("").to_string(),
            input_mode: v["input_mode"].as_str().unwrap_or("approve_reject").to_string(),
            choices: v["choices"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            inspect_files: v["inspect_files"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineRun {
    pub run_id: String,
    pub routine_id: Option<String>,
    pub routine_name: String,
    pub thread_id: String,
    pub status: String, // "running" | "paused_checkpoint" | "interrupted" | "completed" | "failed" | "cancelled"
    pub current_step_index: usize,
    pub created_ms: u64,
    pub updated_ms: u64,
    pub inputs: HashMap<String, String>,
    pub steps: Vec<PipelineStepSpec>,
    pub step_states: Vec<PipelineStepState>,
    pub pending_checkpoint: Option<PendingCheckpoint>,
    pub error: Option<String>,
    pub cancel_requested: bool,
    pub engagement_id: Option<String>,
    pub usage: records::RunUsage,
}

impl PipelineRun {
    pub fn to_json(&self) -> Value {
        json!({
            "run_id": self.run_id,
            "routine_id": self.routine_id,
            "routine_name": self.routine_name,
            "thread_id": self.thread_id,
            "status": self.status,
            "current_step_index": self.current_step_index,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
            "inputs": self.inputs,
            "steps": self.steps.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "step_states": self.step_states.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "pending_checkpoint": self.pending_checkpoint.as_ref().map(|c| c.to_json()),
            "error": self.error,
            "engagement_id": self.engagement_id,
            "usage": self.usage.to_json(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let run_id = v["run_id"].as_str()?.to_string();
        let mut inputs = HashMap::new();
        if let Some(obj) = v["inputs"].as_object() {
            for (k, val) in obj {
                let s = match val {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                inputs.insert(k.clone(), s);
            }
        }
        let steps: Vec<PipelineStepSpec> = v["steps"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .enumerate()
                    .filter_map(|(i, s)| PipelineStepSpec::from_json(s, i).ok())
                    .collect()
            })
            .unwrap_or_default();
        let step_states: Vec<PipelineStepState> = v["step_states"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(PipelineStepState::from_json)
                    .collect()
            })
            .unwrap_or_default();
        let pending_checkpoint = v
            .get("pending_checkpoint")
            .and_then(|p| if p.is_null() { None } else { PendingCheckpoint::from_json(p) });

        Some(Self {
            run_id,
            routine_id: v["routine_id"].as_str().map(str::to_string),
            routine_name: v["routine_name"].as_str().unwrap_or("Pipeline").to_string(),
            thread_id: v["thread_id"].as_str().unwrap_or("").to_string(),
            status: v["status"].as_str().unwrap_or("failed").to_string(),
            current_step_index: v["current_step_index"].as_u64().unwrap_or(0) as usize,
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
            inputs,
            steps,
            step_states,
            pending_checkpoint,
            error: v["error"].as_str().map(str::to_string),
            cancel_requested: v["cancel_requested"].as_bool().unwrap_or(false),
            engagement_id: v["engagement_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            usage: v
                .get("usage")
                .and_then(records::RunUsage::from_json)
                .unwrap_or_default(),
        })
    }
}

fn load_runs_unlocked(person_data_dir: &Path) -> Vec<PipelineRun> {
    let path = pipeline_runs_path(person_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v["runs"]
        .as_array()
        .map(|arr| arr.iter().filter_map(PipelineRun::from_json).collect())
        .unwrap_or_default()
}

fn save_runs_unlocked(person_data_dir: &Path, runs: &[PipelineRun]) -> Result<(), String> {
    let path = pipeline_runs_path(person_data_dir);
    let start = runs.len().saturating_sub(MAX_STORED_RUNS);
    let slice: Vec<Value> = runs[start..].iter().map(|r| {
        let mut obj = r.to_json();
        if r.cancel_requested {
            obj["cancel_requested"] = json!(true);
        }
        obj
    }).collect();
    let body = json!({ "runs": slice }).to_string();
    atomic_write_bytes(&path, body.as_bytes()).map_err(|e| format!("write pipeline_runs: {e}"))
}

pub fn load_runs(person_data_dir: &Path) -> Vec<PipelineRun> {
    with_pipelines_lock(|| load_runs_unlocked(person_data_dir))
}

pub fn get_run(person_data_dir: &Path, run_id: &str) -> Option<PipelineRun> {
    with_pipelines_lock(|| {
        load_runs_unlocked(person_data_dir)
            .into_iter()
            .find(|r| r.run_id == run_id)
    })
}

pub fn list_runs(
    person_data_dir: &Path,
    status_filter: Option<&str>,
    limit: usize,
) -> Vec<PipelineRun> {
    let mut runs = load_runs(person_data_dir);
    if let Some(st) = status_filter.map(str::trim).filter(|s| !s.is_empty()) {
        runs.retain(|r| r.status == st);
    }
    runs.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms).then_with(|| b.created_ms.cmp(&a.created_ms)));
    runs.truncate(limit.clamp(1, 200));
    runs
}

pub fn save_run(person_data_dir: &Path, run: &PipelineRun) -> Result<(), String> {
    with_pipelines_lock(|| {
        let mut runs = load_runs_unlocked(person_data_dir);
        if let Some(pos) = runs.iter().position(|r| r.run_id == run.run_id) {
            runs[pos] = run.clone();
        } else {
            runs.push(run.clone());
        }
        save_runs_unlocked(person_data_dir, &runs)
    })
}

fn mutate_run<F, R>(person_data_dir: &Path, run_id: &str, f: F) -> Result<R, String>
where
    F: FnOnce(&mut PipelineRun) -> Result<R, String>,
{
    with_pipelines_lock(|| {
        let mut runs = load_runs_unlocked(person_data_dir);
        let Some(pos) = runs.iter().position(|r| r.run_id == run_id) else {
            return Err("pipeline run not found".to_string());
        };
        let out = f(&mut runs[pos])?;
        save_runs_unlocked(person_data_dir, &runs)?;
        Ok(out)
    })
}

fn mutate_run_http<F, R>(person_data_dir: &Path, run_id: &str, f: F) -> Result<R, (u16, String)>
where
    F: FnOnce(&mut PipelineRun) -> Result<R, (u16, String)>,
{
    with_pipelines_lock(|| {
        let mut runs = load_runs_unlocked(person_data_dir);
        let Some(pos) = runs.iter().position(|r| r.run_id == run_id) else {
            return Err((404, "pipeline run not found".to_string()));
        };
        let out = f(&mut runs[pos])?;
        save_runs_unlocked(person_data_dir, &runs).map_err(|e| (500, e))?;
        Ok(out)
    })
}

/// Convert any stale `"running"` runs persisted from a prior server process
/// into resumable `"paused_checkpoint"` (when a checkpoint was already active),
/// `"cancelled"` (when cancellation was already requested), or `"interrupted"`
/// state so the user can resume or retry after restart.
pub fn recover_stale_runs_on_boot(person_data_dir: &Path) -> usize {
    let key = person_data_dir.to_path_buf();
    {
        let mut set = recovered_dirs().lock().unwrap_or_else(|p| p.into_inner());
        if !set.insert(key) {
            return 0;
        }
    }
    force_recover_stale_runs(person_data_dir)
}

/// Force recovery scan (used at boot and in simulated-restart tests).
pub fn force_recover_stale_runs(person_data_dir: &Path) -> usize {
    with_pipelines_lock(|| {
        let mut runs = load_runs_unlocked(person_data_dir);
        let now_ms = records::now_millis();
        let mut changed = 0usize;
        for run in &mut runs {
            if run.status == "paused_checkpoint" && run.pending_checkpoint.is_none() {
                changed += 1;
                run.updated_ms = now_ms;
                run.status = "interrupted".to_string();
                run.error = Some(
                    "Interrupted by server restart (missing checkpoint state) — retry or resume to continue.".to_string(),
                );
                for st in &mut run.step_states {
                    if st.status == "running" || st.status == "waiting_human" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Interrupted by server restart.".to_string());
                    }
                }
                continue;
            }
            if run.status != "running" {
                continue;
            }
            changed += 1;
            run.updated_ms = now_ms;
            if run.cancel_requested {
                run.status = "cancelled".to_string();
                run.pending_checkpoint = None;
                run.error = Some("Cancelled before server restart.".to_string());
                for st in &mut run.step_states {
                    if st.status == "running" || st.status == "waiting_human" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Cancelled before server restart.".to_string());
                    }
                }
            } else if let Some(cp) = run.pending_checkpoint.clone() {
                run.status = "paused_checkpoint".to_string();
                if let Some(st) = run.step_states.get_mut(cp.step_index) {
                    if st.status != "waiting_human" {
                        st.status = "waiting_human".to_string();
                        st.checkpoint_id = Some(cp.checkpoint_id);
                    }
                }
            } else {
                run.status = "interrupted".to_string();
                run.error = Some(
                    "Interrupted by server restart — retry or resume to continue.".to_string(),
                );
                for st in &mut run.step_states {
                    if st.status == "running" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Interrupted by server restart.".to_string());
                    }
                }
            }
        }
        if changed > 0 {
            let _ = save_runs_unlocked(person_data_dir, &runs);
        }
        changed
    })
}

// ---------------------------------------------------------------------------
// Template resolution (`{{inputs.*}}` & `{{steps.<id>.*}}`)
// ---------------------------------------------------------------------------

fn find_step_state<'a>(run: &'a PipelineRun, id: &str) -> Option<&'a PipelineStepState> {
    let limit = run.current_step_index.min(run.step_states.len());
    if id == "prev" || id == "previous" {
        return run
            .step_states
            .iter()
            .take(limit)
            .rev()
            .find(|s| matches!(s.status.as_str(), "ok" | "waiting_human"));
    }
    // Only allow referencing steps strictly prior to `current_step_index` (and not
    // currently running) so self-references and forward/circular step references
    // never resolve to unexecuted or in-flight state.
    run.step_states
        .iter()
        .take(limit)
        .find(|s| s.step_id == id && s.status != "running")
}

fn resolve_expr(expr: &str, run: &PipelineRun) -> Option<Value> {
    let trimmed = expr.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || trimmed.contains("{{") || trimmed.contains("}}") {
        return None;
    }
    if let Some(key) = trimmed.strip_prefix("inputs.") {
        return Some(json!(run.inputs.get(key.trim()).cloned().unwrap_or_default()));
    }
    if let Some(rest) = trimmed.strip_prefix("steps.") {
        let (step_id, field) = rest.split_once('.')?;
        let state = find_step_state(run, step_id.trim())?;
        let field = field.trim();
        if field == "files" {
            return Some(json!(state.files));
        }
        if let Some(idx_part) = field.strip_prefix("files[").and_then(|s| s.strip_suffix(']')) {
            if let Ok(idx) = idx_part.trim().parse::<usize>() {
                return Some(json!(state.files.get(idx).cloned().unwrap_or_default()));
            }
        }
        if field == "output" {
            return Some(json!(state.output));
        }
        if field == "human_answer" {
            return Some(json!(state.human_answer.clone().unwrap_or_default()));
        }
        if field == "human_action" {
            return Some(json!(state.human_action.clone().unwrap_or_default()));
        }
        if let Some(metric_key) = field.strip_prefix("metrics.") {
            let mut cur = &state.metrics;
            let parts: Vec<&str> = metric_key.split('.').map(str::trim).take(9).collect();
            if parts.len() > 8 {
                return Some(json!(""));
            }
            for seg in parts {
                if seg.is_empty() {
                    return Some(json!(""));
                }
                cur = match cur.get(seg) {
                    Some(v) => v,
                    None => return Some(json!("")),
                };
            }
            return Some(cur.clone());
        }
    }
    None
}

fn push_capped(out: &mut String, s: &str, cap: usize) {
    if out.len() >= cap {
        return;
    }
    let avail = cap - out.len();
    if s.len() <= avail {
        out.push_str(s);
    } else {
        let mut end = avail;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        out.push_str(&s[..end]);
    }
}

fn value_to_inline_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .map(value_to_inline_str)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn resolve_template_str(tpl: &str, run: &PipelineRun) -> String {
    let mut out = String::with_capacity(tpl.len().min(MAX_TEMPLATE_OUTPUT_BYTES));
    let mut rest = tpl;
    let mut expansions = 0usize;
    while let Some(start) = rest.find("{{") {
        push_capped(&mut out, &rest[..start], MAX_TEMPLATE_OUTPUT_BYTES);
        if out.len() >= MAX_TEMPLATE_OUTPUT_BYTES || expansions >= MAX_TEMPLATE_EXPANSIONS {
            return out;
        }
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            expansions += 1;
            let expr = &after_open[..end];
            // If a malformed nested `{{` appears before `}}`, advance past the
            // outer `{{` without misparsing across braces.
            if expr.contains("{{") {
                push_capped(&mut out, "{{", MAX_TEMPLATE_OUTPUT_BYTES);
                rest = after_open;
                continue;
            }
            if let Some(val) = resolve_expr(expr, run) {
                let inline = value_to_inline_str(&val);
                push_capped(&mut out, &inline, MAX_TEMPLATE_OUTPUT_BYTES);
            }
            rest = &after_open[end + 2..];
        } else {
            push_capped(&mut out, &rest[start..], MAX_TEMPLATE_OUTPUT_BYTES);
            rest = "";
            break;
        }
    }
    push_capped(&mut out, rest, MAX_TEMPLATE_OUTPUT_BYTES);
    out
}

fn resolve_template_value_depth(val: &Value, run: &PipelineRun, depth: usize) -> Value {
    if depth >= MAX_TEMPLATE_DEPTH {
        return val.clone();
    }
    match val {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") && trimmed.matches("{{").count() == 1 {
                let expr = &trimmed[2..trimmed.len() - 2];
                if let Some(resolved) = resolve_expr(expr, run) {
                    if resolved.is_array() || resolved.is_number() || resolved.is_boolean() {
                        return resolved;
                    }
                }
            }
            Value::String(resolve_template_str(s, run))
        }
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                let r = resolve_template_value_depth(item, run, depth + 1);
                // If an array element resolved from `{{steps.x.files}}` into an array, flatten cleanly
                if item.as_str().is_some_and(|s| s.trim().ends_with(".files}}")) {
                    if let Value::Array(inner) = r {
                        out.extend(inner);
                        continue;
                    }
                }
                out.push(r);
            }
            Value::Array(out)
        }
        Value::Object(map) => {
            let mut out = Map::with_capacity(map.len());
            for (k, v) in map {
                out.insert(k.clone(), resolve_template_value_depth(v, run, depth + 1));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

pub fn resolve_template_value(val: &Value, run: &PipelineRun) -> Value {
    resolve_template_value_depth(val, run, 0)
}

fn should_run_step(step: &PipelineStepSpec, prior_states: &[PipelineStepState]) -> (bool, &'static str) {
    match step.when.as_str() {
        "on_previous_ok" => {
            let prev = prior_states
                .iter()
                .rev()
                .find(|s| s.status != "skipped");
            match prev {
                None => (true, ""),
                Some(s) if s.status == "ok" => (true, ""),
                Some(_) => (false, "Skipped because previous step did not succeed."),
            }
        }
        "on_anomaly" => {
            let any_anomaly = prior_states.iter().any(|s| {
                s.metrics["has_anomaly"].as_bool() == Some(true)
                    || s.metrics["anomalies"].as_u64().unwrap_or(0) > 0
                    || s.metrics["outliers"].as_u64().unwrap_or(0) > 0
                    || s.metrics["duplicates"].as_u64().unwrap_or(0) > 0
                    || s.metrics["missing_receipts"].as_u64().unwrap_or(0) > 0
                    || s.metrics["high_risks"].as_u64().unwrap_or(0) > 0
            });
            if any_anomaly {
                (true, "")
            } else {
                (false, "Skipped: no anomalies flagged in prior steps.")
            }
        }
        _ => (true, ""),
    }
}

// ---------------------------------------------------------------------------
// Step Execution & Metrics Extraction
// ---------------------------------------------------------------------------

fn extract_table_pipe_u64(text: &str, needle: &str) -> u64 {
    for line in text.lines() {
        if crate::tables::ascii_contains_ignore_case(line, needle) {
            if let Some(after_first) = line.split('|').map(str::trim).filter(|p| !p.is_empty()).nth(1) {
                if let Ok(n) = after_first.split_whitespace().next().unwrap_or("").parse::<u64>() {
                    return n;
                }
            }
        }
    }
    0
}

fn extract_step_metrics(cfg: &Config, files: &[String], output: &str) -> Value {
    use std::io::Read as _;
    let mut anomalies = 0u64;
    let mut duplicates = 0u64;
    let mut missing_receipts = 0u64;
    let mut outliers = 0u64;
    let mut high_risks = 0u64;
    let mut urgent = 0u64;
    let mut total_bytes = 0u64;

    for rel in files {
        if let Ok(full) = sandbox_path(&cfg.workspace_dir, rel) {
            if let Ok(md) = std::fs::metadata(&full) {
                total_bytes += md.len();
            }
            if rel.to_ascii_lowercase().ends_with(".md") {
                if let Ok(f) = std::fs::File::open(&full) {
                    let mut buf = Vec::with_capacity(8192);
                    if f.take(65_536).read_to_end(&mut buf).is_ok() {
                        let clip = String::from_utf8_lossy(&buf);
                        duplicates += extract_table_pipe_u64(&clip, "Duplicate references flagged");
                        duplicates += extract_table_pipe_u64(&clip, "Duplicate rows removed");
                        missing_receipts += extract_table_pipe_u64(&clip, "Missing receipts flagged");
                        anomalies += extract_table_pipe_u64(&clip, "Amount anomalies flagged");
                        outliers += extract_table_pipe_u64(&clip, "Outlier");
                        high_risks += extract_table_pipe_u64(&clip, "Risk findings");
                        urgent += extract_table_pipe_u64(&clip, "**Urgent**");
                    }
                }
            }
        }
    }

    let total_flags = anomalies + duplicates + missing_receipts + outliers + high_risks + urgent;
    let has_anomaly = total_flags > 0
        || crate::tables::ascii_contains_ignore_case(output, "anomal");
    json!({
        "files_count": files.len(),
        "bytes_total": total_bytes,
        "anomalies": total_flags,
        "duplicates": duplicates,
        "missing_receipts": missing_receipts,
        "outliers": outliers,
        "high_risks": high_risks,
        "urgent": urgent,
        "has_anomaly": has_anomaly,
    })
}

#[derive(Clone)]
pub struct PipelineExecEnv {
    pub cfg: Config,
    pub ctx: ToolCtx,
    pub store: SharedStore,
    pub live_runs: LiveRuns,
    pub events: RunEventBus,
    pub search_cache: SearchCache,
}

fn execute_skill_step(
    cfg: &Config,
    library: Option<&Arc<Mutex<crate::library::Library>>>,
    skill_id: &str,
    resolved_args: &Value,
) -> Result<(String, Vec<String>), String> {
    if let Some(res) = skills::dispatch_builtin_skill(cfg, skill_id, resolved_args) {
        let text = res?;
        let files = crate::extract_written(&text, &cfg.workspace_dir);
        return Ok((text, files));
    }
    // Check installed/library skills in company library
    if let Some(lib_arc) = library {
        let runnable = {
            let lib = lib_arc.lock().unwrap();
            lib.get(skill_id).and_then(|item| {
                if item.kind == "skill" && !item.builtin {
                    let values: Vec<String> = item
                        .params
                        .iter()
                        .map(|p| {
                            resolved_args[p.as_str()]
                                .as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        })
                        .collect();
                    Some((item.source.clone(), values))
                } else {
                    None
                }
            })
        };
        if let Some((source, values)) = runnable {
            let text = skills::run_library_skill(cfg, &source, &values)?;
            let files = crate::extract_written(&text, &cfg.workspace_dir);
            return Ok((text, files));
        }
    }
    Err(format!("unknown deterministic skill '{skill_id}'"))
}

/// Synchronously run a deterministic (`skill` / `export`) pipeline without
/// requiring an HTTP `AppState` or LLM key (used by `launch_starter_playbook`
/// and offline tests). Persists the `PipelineRun` into `pipeline_runs.json`.
pub fn execute_deterministic_steps_sync(
    cfg: &Config,
    routine_id: Option<&str>,
    routine_name: &str,
    steps: &[PipelineStepSpec],
    inputs: HashMap<String, String>,
) -> Result<PipelineRun, String> {
    let now_ms = records::now_millis();
    let seq = PIPELINE_SEQ.fetch_add(1, Ordering::SeqCst);
    let run_id = format!("prun-{now_ms}-{seq}");
    let thread_id = format!("starter-{}", routine_id.unwrap_or(&run_id));
    let mut run = PipelineRun {
        run_id: run_id.clone(),
        routine_id: routine_id.map(str::to_string),
        routine_name: routine_name.to_string(),
        thread_id: thread_id.clone(),
        status: "running".to_string(),
        current_step_index: 0,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs,
        steps: steps.to_vec(),
        step_states: steps.iter().map(PipelineStepState::pending_for).collect(),
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: records::RunUsage::default(),
    };
    save_run(&cfg.data_dir, &run)?;

    for idx in 0..run.steps.len() {
        let step = run.steps[idx].clone();
        let (should_run, skip_reason) = should_run_step(&step, &run.step_states[..idx]);
        if !should_run {
            let ts = records::now_millis();
            run.current_step_index = idx;
            run.updated_ms = ts;
            run.step_states[idx].status = "skipped".to_string();
            run.step_states[idx].started_ms = ts;
            run.step_states[idx].finished_ms = ts;
            run.step_states[idx].output = skip_reason.to_string();
            save_run(&cfg.data_dir, &run)?;
            continue;
        }
        let started = records::now_millis();
        run.current_step_index = idx;
        run.step_states[idx].status = "running".to_string();
        run.step_states[idx].started_ms = started;

        let res: Result<(String, Vec<String>), String> = match step.kind.as_str() {
            "skill" => {
                let skill_id = step.skill_id.as_deref().unwrap_or("");
                let resolved_args = resolve_template_value(&step.args, &run);
                execute_skill_step(cfg, None, skill_id, &resolved_args)
            }
            "export" => {
                let ex = step.export.as_ref().ok_or("export step missing spec")?;
                let src_path = resolve_template_str(&ex.path, &run);
                let out_opt = ex.output.as_deref().map(|o| resolve_template_str(o, &run));
                tools::export_workspace_file(
                    cfg,
                    &src_path,
                    if ex.format.is_empty() { None } else { Some(ex.format.as_str()) },
                    out_opt.as_deref(),
                )
                .map(|(out_path, bytes, what)| {
                    (format!("exported {out_path} ({bytes} bytes, {what})"), vec![out_path])
                })
            }
            other => Err(format!("sync deterministic runner does not support step kind '{other}'")),
        };

        let finished = records::now_millis();
        run.updated_ms = finished;
        match res {
            Ok((output, files)) => {
                let metrics = extract_step_metrics(cfg, &files, &output);
                run.step_states[idx].status = "ok".to_string();
                run.step_states[idx].finished_ms = finished;
                run.step_states[idx].output = output.clone();
                run.step_states[idx].files = files.clone();
                run.step_states[idx].metrics = metrics;
                records::append_work(
                    cfg,
                    &format!("{routine_name}: {}", step.label),
                    &output,
                    &files,
                    &thread_id,
                    true,
                );
            }
            Err(err) => {
                run.status = "failed".to_string();
                run.error = Some(err.clone());
                run.step_states[idx].status = "failed".to_string();
                run.step_states[idx].finished_ms = finished;
                run.step_states[idx].error = Some(err.clone());
                save_run(&cfg.data_dir, &run)?;
                return Err(err);
            }
        }
        save_run(&cfg.data_dir, &run)?;
    }

    run.status = "completed".to_string();
    run.current_step_index = run.steps.len();
    run.updated_ms = records::now_millis();
    save_run(&cfg.data_dir, &run)?;
    if let Some(rid) = routine_id {
        let _ = routines::record_run(&cfg.data_dir, rid, &thread_id, "finished");
    }
    Ok(run)
}

fn build_checkpoint_from_step(step: &PipelineStepSpec, run: &PipelineRun) -> PipelineCheckpointSpec {
    if let Some(cp) = &step.checkpoint {
        let inspect_files: Vec<String> = cp
            .inspect_files
            .iter()
            .map(|f| resolve_template_str(f, run))
            .filter(|f| !f.trim().is_empty())
            .collect();
        PipelineCheckpointSpec {
            prompt: resolve_template_str(&cp.prompt, run),
            input_mode: cp.input_mode.clone(),
            choices: cp.choices.clone(),
            inspect_files,
        }
    } else {
        PipelineCheckpointSpec {
            prompt: resolve_template_str(step.prompt.as_deref().unwrap_or(&step.label), run),
            input_mode: "approve_reject".to_string(),
            choices: Vec::new(),
            inspect_files: Vec::new(),
        }
    }
}

fn park_pipeline_at_checkpoint(
    env: &PipelineExecEnv,
    run_id: &str,
    step_idx: usize,
    cp_spec: PipelineCheckpointSpec,
    prior_files: Vec<String>,
    prior_output: Option<String>,
) -> Result<PipelineRun, String> {
    let safe_inspect: Vec<String> = cp_spec
        .inspect_files
        .iter()
        .filter(|p| sandbox_path(&env.cfg.workspace_dir, p).is_ok())
        .cloned()
        .collect();
    let run_snapshot = get_run(&env.cfg.data_dir, run_id)
        .ok_or_else(|| "pipeline run not found".to_string())?;
    let step_id = run_snapshot
        .steps
        .get(step_idx)
        .map(|s| s.id.clone())
        .unwrap_or_else(|| format!("step_{}", step_idx + 1));

    let decision_id = records::add_decision_with(
        &env.cfg.data_dir,
        "checkpoint",
        &cp_spec.prompt,
        safe_inspect.first().map(String::as_str),
        &run_snapshot.thread_id,
        json!({
            "pipeline_run_id": run_id,
            "step_index": step_idx,
            "step_id": step_id,
            "input_mode": cp_spec.input_mode,
            "choices": cp_spec.choices,
            "inspect_files": safe_inspect,
        }),
    );

    let now_ms = records::now_millis();
    let pending = PendingCheckpoint {
        checkpoint_id: decision_id.clone(),
        step_index: step_idx,
        step_id: step_id.clone(),
        prompt: cp_spec.prompt.clone(),
        input_mode: cp_spec.input_mode,
        choices: cp_spec.choices,
        inspect_files: safe_inspect,
        created_ms: now_ms,
    };

    let updated = mutate_run(&env.cfg.data_dir, run_id, |r| {
        r.status = "paused_checkpoint".to_string();
        r.current_step_index = step_idx;
        r.updated_ms = now_ms;
        r.pending_checkpoint = Some(pending.clone());
        if let Some(st) = r.step_states.get_mut(step_idx) {
            st.status = "waiting_human".to_string();
            st.checkpoint_id = Some(decision_id.clone());
            if let Some(ref out) = prior_output {
                st.output = out.clone();
            } else {
                st.output = format!("Waiting on human checkpoint: {}", pending.prompt);
            }
            if !prior_files.is_empty() {
                st.files = prior_files;
            }
        }
        Ok(r.clone())
    })?;

    env.store.lock().unwrap().push(
        &updated.thread_id,
        "step",
        &format!("Paused at checkpoint ({decision_id}): {}", pending.prompt),
    );
    env.search_cache.invalidate(&env.ctx.person_id);
    env.events.publish(
        "pipeline_paused",
        &env.ctx.person_id,
        &updated.run_id,
        &updated.thread_id,
        json!({
            "routine_name": updated.routine_name,
            "step_index": step_idx,
            "step_id": step_id,
            "checkpoint": pending.to_json(),
            "run": updated.to_json(),
            "ts_ms": now_ms,
        }),
    );
    Ok(updated)
}

fn register_cancel_flag(run_id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let mut map = cancel_registry().lock().unwrap_or_else(|p| p.into_inner());
    map.insert(run_id.to_string(), flag.clone());
    flag
}

fn unregister_cancel_flag(run_id: &str) {
    let mut map = cancel_registry().lock().unwrap_or_else(|p| p.into_inner());
    map.remove(run_id);
}

fn is_run_cancelled(person_data_dir: &Path, run_id: &str, flag: &AtomicBool) -> bool {
    if flag.load(Ordering::SeqCst) {
        return true;
    }
    get_run(person_data_dir, run_id)
        .map(|r| r.cancel_requested || r.status == "cancelled")
        .unwrap_or(false)
}

fn deterministic_step_usage(
    root_data_dir: &Path,
    engagement_id: Option<&str>,
    files_written: usize,
) -> (records::RunUsage, Option<String>) {
    let store = records::load_clients_store(root_data_dir);
    let eng = engagement_id.and_then(|eid| store.engagements.iter().find(|e| e.id == eid));
    let client_id = eng.map(|e| e.client_id.clone());
    let hourly_rate = eng
        .map(|e| {
            if e.hourly_rate_usd > 0.0 {
                e.hourly_rate_usd
            } else {
                store
                    .clients
                    .iter()
                    .find(|c| c.id == e.client_id)
                    .map(|c| c.default_rate_usd)
                    .unwrap_or(0.0)
            }
        })
        .unwrap_or(0.0);
    let mut usage = records::RunUsage::default();
    usage.finalize_attribution(true, false, 1, files_written, hourly_rate);
    (usage, client_id)
}

fn execute_pipeline_loop(env: &PipelineExecEnv, run_id: &str, start_index: usize) -> Result<PipelineRun, String> {
    let cancel_flag = register_cancel_flag(run_id);
    struct CancelGuard<'a>(&'a str);
    impl Drop for CancelGuard<'_> {
        fn drop(&mut self) {
            unregister_cancel_flag(self.0);
        }
    }
    let _cg = CancelGuard(run_id);

    let initial = get_run(&env.cfg.data_dir, run_id)
        .ok_or_else(|| "pipeline run not found".to_string())?;
    let total_steps = initial.steps.len();

    for idx in start_index..total_steps {
        if is_run_cancelled(&env.cfg.data_dir, run_id, &cancel_flag) {
            let now_ms = records::now_millis();
            let cancelled = mutate_run(&env.cfg.data_dir, run_id, |r| {
                r.status = "cancelled".to_string();
                r.updated_ms = now_ms;
                r.pending_checkpoint = None;
                Ok(r.clone())
            })?;
            env.events.publish(
                "pipeline_finished",
                &env.ctx.person_id,
                &cancelled.run_id,
                &cancelled.thread_id,
                json!({
                    "status": "cancelled",
                    "error": Value::Null,
                    "engagement_id": cancelled.engagement_id,
                    "usage": cancelled.usage.to_json(),
                    "run": cancelled.to_json(),
                    "ts_ms": now_ms,
                }),
            );
            return Ok(cancelled);
        }

        let run_snap = get_run(&env.cfg.data_dir, run_id)
            .ok_or_else(|| "pipeline run not found".to_string())?;
        let step = run_snap.steps[idx].clone();

        let (should_run, skip_reason) = should_run_step(&step, &run_snap.step_states[..idx]);
        if !should_run {
            let now_ms = records::now_millis();
            mutate_run(&env.cfg.data_dir, run_id, |r| {
                r.current_step_index = idx;
                r.updated_ms = now_ms;
                if let Some(st) = r.step_states.get_mut(idx) {
                    st.status = "skipped".to_string();
                    st.started_ms = now_ms;
                    st.finished_ms = now_ms;
                    st.output = skip_reason.to_string();
                }
                Ok(())
            })?;
            env.events.publish(
                "pipeline_step_done",
                &env.ctx.person_id,
                &run_snap.run_id,
                &run_snap.thread_id,
                json!({
                    "step_index": idx,
                    "step_id": step.id,
                    "label": step.label,
                    "kind": step.kind,
                    "status": "skipped",
                    "output": skip_reason,
                    "files": [],
                    "metrics": {},
                    "error": Value::Null,
                    "elapsed_ms": 0,
                    "ts_ms": now_ms,
                }),
            );
            continue;
        }

        let started_ms = records::now_millis();
        let run_snap = mutate_run(&env.cfg.data_dir, run_id, |r| {
            r.status = "running".to_string();
            r.current_step_index = idx;
            r.updated_ms = started_ms;
            if let Some(st) = r.step_states.get_mut(idx) {
                st.status = "running".to_string();
                st.started_ms = started_ms;
                st.error = None;
            }
            Ok(r.clone())
        })?;

        env.events.publish(
            "pipeline_step_started",
            &env.ctx.person_id,
            &run_snap.run_id,
            &run_snap.thread_id,
            json!({
                "routine_id": run_snap.routine_id,
                "routine_name": run_snap.routine_name,
                "step_index": idx,
                "step_id": step.id,
                "label": step.label,
                "kind": step.kind,
                "total_steps": total_steps,
                "ts_ms": started_ms,
            }),
        );

        match step.kind.as_str() {
            "checkpoint" => {
                let cp_spec = build_checkpoint_from_step(&step, &run_snap);
                return park_pipeline_at_checkpoint(
                    env,
                    run_id,
                    idx,
                    cp_spec,
                    Vec::new(),
                    None,
                );
            }
            "skill" => {
                let skill_id = step.skill_id.as_deref().unwrap_or("");
                let resolved_args = resolve_template_value(&step.args, &run_snap);
                let step_t0 = std::time::Instant::now();
                match execute_skill_step(&env.cfg, Some(&env.ctx.library), skill_id, &resolved_args) {
                    Ok((output, files)) => {
                        let finished_ms = records::now_millis();
                        let metrics = extract_step_metrics(&env.cfg, &files, &output);
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let (step_usage, step_client_id) = deterministic_step_usage(
                            &env.ctx.root_data_dir,
                            run_snap.engagement_id.as_deref(),
                            files.len(),
                        );
                        let updated_run = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.updated_ms = finished_ms;
                            r.usage.accumulate(&step_usage);
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "ok".to_string();
                                st.finished_ms = finished_ms;
                                st.output = output.clone();
                                st.files = files.clone();
                                st.metrics = metrics.clone();
                                st.usage = Some(step_usage.clone());
                                st.error = None;
                            }
                            Ok(r.clone())
                        })?;
                        records::append_work_with_usage(
                            &env.cfg,
                            &format!("Pipeline {}: {}", run_snap.routine_name, step.label),
                            &output,
                            &files,
                            &run_snap.thread_id,
                            true,
                            run_snap.engagement_id.as_deref(),
                            Some(&step_usage),
                        );
                        records::append_usage_ledger(
                            &env.cfg.data_dir,
                            &records::UsageLedgerEntry {
                                ts_ms: finished_ms,
                                run_id: run_snap.run_id.clone(),
                                thread_id: run_snap.thread_id.clone(),
                                person_id: env.ctx.person_id.clone(),
                                engagement_id: run_snap.engagement_id.clone(),
                                client_id: step_client_id,
                                source: "pipeline_skill".to_string(),
                                finished: true,
                                answered: false,
                                usage: step_usage.clone(),
                            },
                        );
                        env.store.lock().unwrap().push(
                            &run_snap.thread_id,
                            "step",
                            &format!("Step {} ({}): {}", idx + 1, step.label, output),
                        );
                        env.search_cache.invalidate(&env.ctx.person_id);
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "ok",
                                "output": output,
                                "files": files,
                                "metrics": metrics,
                                "usage": step_usage.to_json(),
                                "run_usage": updated_run.usage.to_json(),
                                "engagement_id": updated_run.engagement_id,
                                "error": Value::Null,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                    }
                    Err(err) => {
                        let finished_ms = records::now_millis();
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.status = "failed".to_string();
                            r.updated_ms = finished_ms;
                            r.error = Some(err.clone());
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "failed".to_string();
                                st.finished_ms = finished_ms;
                                st.error = Some(err.clone());
                            }
                            Ok(r.clone())
                        })?;
                        env.store.lock().unwrap().push(
                            &run_snap.thread_id,
                            "error",
                            &format!("Step {} ({}) failed: {}", idx + 1, step.label, err),
                        );
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "failed",
                                "output": "",
                                "files": [],
                                "metrics": {},
                                "error": err,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                        env.events.publish(
                            "pipeline_finished",
                            &env.ctx.person_id,
                            &failed.run_id,
                            &failed.thread_id,
                            json!({
                                "status": "failed",
                                "error": failed.error,
                                "engagement_id": failed.engagement_id,
                                "usage": failed.usage.to_json(),
                                "run": failed.to_json(),
                                "ts_ms": finished_ms,
                            }),
                        );
                        return Ok(failed);
                    }
                }
            }
            "export" => {
                let ex = step.export.as_ref().ok_or("export step missing spec")?;
                let src_path = resolve_template_str(&ex.path, &run_snap);
                let out_path_opt = ex.output.as_deref().map(|o| resolve_template_str(o, &run_snap));
                let step_t0 = std::time::Instant::now();
                match tools::export_workspace_file(
                    &env.cfg,
                    &src_path,
                    if ex.format.is_empty() { None } else { Some(ex.format.as_str()) },
                    out_path_opt.as_deref(),
                ) {
                    Ok((out_path, bytes, what)) => {
                        let finished_ms = records::now_millis();
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let output = format!("exported {out_path} ({bytes} bytes, {what})");
                        let files = vec![out_path];
                        let metrics = json!({ "files_count": 1, "bytes_total": bytes, "anomalies": 0, "has_anomaly": false });
                        let (step_usage, step_client_id) = deterministic_step_usage(
                            &env.ctx.root_data_dir,
                            run_snap.engagement_id.as_deref(),
                            files.len(),
                        );
                        let updated_run = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.updated_ms = finished_ms;
                            r.usage.accumulate(&step_usage);
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "ok".to_string();
                                st.finished_ms = finished_ms;
                                st.output = output.clone();
                                st.files = files.clone();
                                st.metrics = metrics.clone();
                                st.usage = Some(step_usage.clone());
                                st.error = None;
                            }
                            Ok(r.clone())
                        })?;
                        records::append_work_with_usage(
                            &env.cfg,
                            &format!("Pipeline {}: {}", run_snap.routine_name, step.label),
                            &output,
                            &files,
                            &run_snap.thread_id,
                            true,
                            run_snap.engagement_id.as_deref(),
                            Some(&step_usage),
                        );
                        records::append_usage_ledger(
                            &env.cfg.data_dir,
                            &records::UsageLedgerEntry {
                                ts_ms: finished_ms,
                                run_id: run_snap.run_id.clone(),
                                thread_id: run_snap.thread_id.clone(),
                                person_id: env.ctx.person_id.clone(),
                                engagement_id: run_snap.engagement_id.clone(),
                                client_id: step_client_id,
                                source: "pipeline_export".to_string(),
                                finished: true,
                                answered: false,
                                usage: step_usage.clone(),
                            },
                        );
                        env.store.lock().unwrap().push(
                            &run_snap.thread_id,
                            "step",
                            &format!("Step {} ({}): {}", idx + 1, step.label, output),
                        );
                        env.search_cache.invalidate(&env.ctx.person_id);
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "ok",
                                "output": output,
                                "files": files,
                                "metrics": metrics,
                                "usage": step_usage.to_json(),
                                "run_usage": updated_run.usage.to_json(),
                                "engagement_id": updated_run.engagement_id,
                                "error": Value::Null,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                    }
                    Err(err) => {
                        let finished_ms = records::now_millis();
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.status = "failed".to_string();
                            r.updated_ms = finished_ms;
                            r.error = Some(err.clone());
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "failed".to_string();
                                st.finished_ms = finished_ms;
                                st.error = Some(err.clone());
                            }
                            Ok(r.clone())
                        })?;
                        env.store.lock().unwrap().push(
                            &run_snap.thread_id,
                            "error",
                            &format!("Step {} ({}) failed: {}", idx + 1, step.label, err),
                        );
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "failed",
                                "output": "",
                                "files": [],
                                "metrics": {},
                                "error": err,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                        env.events.publish(
                            "pipeline_finished",
                            &env.ctx.person_id,
                            &failed.run_id,
                            &failed.thread_id,
                            json!({
                                "status": "failed",
                                "error": failed.error,
                                "engagement_id": failed.engagement_id,
                                "usage": failed.usage.to_json(),
                                "run": failed.to_json(),
                                "ts_ms": finished_ms,
                            }),
                        );
                        return Ok(failed);
                    }
                }
            }
            _ => {
                // "agent" step
                let raw_prompt = step.prompt.as_deref().unwrap_or(&step.label);
                let resolved_prompt = resolve_template_str(raw_prompt, &run_snap);
                let step_t0 = std::time::Instant::now();
                let history = {
                    let st = env.store.lock().unwrap();
                    st.history_json(&run_snap.thread_id)
                };
                match run_agent_for_pipeline(
                    &env.cfg,
                    &env.ctx,
                    &env.store,
                    &run_snap.thread_id,
                    &resolved_prompt,
                    &history,
                    Some((&env.live_runs, &env.events, &env.ctx.person_id)),
                ) {
                    Ok(out) => {
                        if out.budget_gated {
                            let finished_ms = records::now_millis();
                            let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                            let step_usage = out.usage.clone();
                            let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                                r.status = "failed".to_string();
                                r.updated_ms = finished_ms;
                                r.usage.accumulate(&step_usage);
                                r.error = Some(out.summary.clone());
                                if let Some(st) = r.step_states.get_mut(idx) {
                                    st.status = "failed".to_string();
                                    st.finished_ms = finished_ms;
                                    st.usage = Some(step_usage.clone());
                                    st.error = Some(out.summary.clone());
                                }
                                Ok(r.clone())
                            })?;
                            env.events.publish(
                                "pipeline_step_done",
                                &env.ctx.person_id,
                                &run_snap.run_id,
                                &run_snap.thread_id,
                                json!({
                                    "step_index": idx,
                                    "step_id": step.id,
                                    "label": step.label,
                                    "kind": step.kind,
                                    "status": "failed",
                                    "output": out.summary,
                                    "files": out.written_files,
                                    "metrics": {},
                                    "usage": step_usage.to_json(),
                                    "run_usage": failed.usage.to_json(),
                                    "engagement_id": failed.engagement_id,
                                    "error": failed.error,
                                    "elapsed_ms": elapsed_ms,
                                    "ts_ms": finished_ms,
                                }),
                            );
                            env.events.publish(
                                "pipeline_finished",
                                &env.ctx.person_id,
                                &failed.run_id,
                                &failed.thread_id,
                                json!({
                                    "status": "failed",
                                    "error": failed.error,
                                    "engagement_id": failed.engagement_id,
                                    "usage": failed.usage.to_json(),
                                    "run": failed.to_json(),
                                    "ts_ms": finished_ms,
                                }),
                            );
                            return Ok(failed);
                        }
                        if let Some(cp_pause) = out.paused_checkpoint {
                            let step_usage = out.usage.clone();
                            let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                                r.usage.accumulate(&step_usage);
                                if let Some(st) = r.step_states.get_mut(idx) {
                                    st.usage = Some(step_usage);
                                }
                                Ok(())
                            });
                            let cp_spec = PipelineCheckpointSpec {
                                prompt: cp_pause.prompt,
                                input_mode: cp_pause.input_mode,
                                choices: cp_pause.choices,
                                inspect_files: cp_pause.inspect_files,
                            };
                            return park_pipeline_at_checkpoint(
                                env,
                                run_id,
                                idx,
                                cp_spec,
                                out.written_files,
                                Some(out.summary),
                            );
                        }
                        let finished_ms = records::now_millis();
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let metrics = extract_step_metrics(&env.cfg, &out.written_files, &out.summary);
                        let step_usage = out.usage.clone();
                        let updated_run = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.updated_ms = finished_ms;
                            r.usage.accumulate(&step_usage);
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "ok".to_string();
                                st.finished_ms = finished_ms;
                                st.output = out.summary.clone();
                                st.files = out.written_files.clone();
                                st.metrics = metrics.clone();
                                st.usage = Some(step_usage.clone());
                                st.error = None;
                            }
                            Ok(r.clone())
                        })?;
                        env.search_cache.invalidate(&env.ctx.person_id);
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "ok",
                                "output": out.summary,
                                "files": out.written_files,
                                "metrics": metrics,
                                "usage": step_usage.to_json(),
                                "run_usage": updated_run.usage.to_json(),
                                "engagement_id": updated_run.engagement_id,
                                "error": Value::Null,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                    }
                    Err(err) => {
                        let finished_ms = records::now_millis();
                        let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                        let is_cancel = err == "run cancelled by user";
                        let next_status = if is_cancel { "cancelled" } else { "failed" };
                        let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                            r.status = next_status.to_string();
                            r.updated_ms = finished_ms;
                            r.error = Some(err.clone());
                            if let Some(st) = r.step_states.get_mut(idx) {
                                st.status = "failed".to_string();
                                st.finished_ms = finished_ms;
                                st.error = Some(err.clone());
                            }
                            Ok(r.clone())
                        })?;
                        env.store.lock().unwrap().push(
                            &run_snap.thread_id,
                            "error",
                            &format!("Step {} ({}) {}: {}", idx + 1, step.label, next_status, err),
                        );
                        env.events.publish(
                            "pipeline_step_done",
                            &env.ctx.person_id,
                            &run_snap.run_id,
                            &run_snap.thread_id,
                            json!({
                                "step_index": idx,
                                "step_id": step.id,
                                "label": step.label,
                                "kind": step.kind,
                                "status": "failed",
                                "output": "",
                                "files": [],
                                "metrics": {},
                                "error": err,
                                "elapsed_ms": elapsed_ms,
                                "ts_ms": finished_ms,
                            }),
                        );
                        env.events.publish(
                            "pipeline_finished",
                            &env.ctx.person_id,
                            &failed.run_id,
                            &failed.thread_id,
                            json!({
                                "status": next_status,
                                "error": failed.error,
                                "engagement_id": failed.engagement_id,
                                "usage": failed.usage.to_json(),
                                "run": failed.to_json(),
                                "ts_ms": finished_ms,
                            }),
                        );
                        return Ok(failed);
                    }
                }
            }
        }
    }

    let finished_ms = records::now_millis();
    let final_cancelled = is_run_cancelled(&env.cfg.data_dir, run_id, &cancel_flag);
    let completed = mutate_run(&env.cfg.data_dir, run_id, |r| {
        if final_cancelled || r.cancel_requested || r.status == "cancelled" {
            r.status = "cancelled".to_string();
            r.pending_checkpoint = None;
        } else {
            r.status = "completed".to_string();
            r.current_step_index = total_steps;
            r.pending_checkpoint = None;
            r.error = None;
        }
        r.updated_ms = finished_ms;
        Ok(r.clone())
    })?;
    if completed.status == "cancelled" {
        env.events.publish(
            "pipeline_finished",
            &env.ctx.person_id,
            &completed.run_id,
            &completed.thread_id,
            json!({
                "status": "cancelled",
                "error": Value::Null,
                "engagement_id": completed.engagement_id,
                "usage": completed.usage.to_json(),
                "run": completed.to_json(),
                "ts_ms": finished_ms,
            }),
        );
        return Ok(completed);
    }
    if let Some(rid) = completed.routine_id.as_deref() {
        let _ = routines::record_run(&env.cfg.data_dir, rid, &completed.thread_id, "finished");
    }
    env.events.publish(
        "pipeline_finished",
        &env.ctx.person_id,
        &completed.run_id,
        &completed.thread_id,
        json!({
            "status": "completed",
            "error": Value::Null,
            "engagement_id": completed.engagement_id,
            "usage": completed.usage.to_json(),
            "run": completed.to_json(),
            "ts_ms": finished_ms,
        }),
    );
    Ok(completed)
}

fn spawn_pipeline_worker(env: PipelineExecEnv, run_id: String, start_index: usize) {
    let _ = std::thread::Builder::new()
        .name(format!("ergon-pipe-{run_id}"))
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            if let Err(err) = execute_pipeline_loop(&env, &run_id, start_index) {
                let now_ms = records::now_millis();
                if let Ok(failed) = mutate_run(&env.cfg.data_dir, &run_id, |r| {
                    if r.status == "running" {
                        r.status = "failed".to_string();
                        r.error = Some(err.clone());
                        r.updated_ms = now_ms;
                    }
                    Ok(r.clone())
                }) {
                    if failed.status == "failed" {
                        env.events.publish(
                            "pipeline_finished",
                            &env.ctx.person_id,
                            &failed.run_id,
                            &failed.thread_id,
                            json!({
                                "status": "failed",
                                "error": failed.error,
                                "engagement_id": failed.engagement_id,
                                "usage": failed.usage.to_json(),
                                "run": failed.to_json(),
                                "ts_ms": now_ms,
                            }),
                        );
                    }
                }
            }
        });
}

fn parse_inputs(val: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            let s = match v {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                other => other.to_string(),
            };
            map.insert(k.clone(), s);
        }
    }
    map
}

pub fn create_and_start_pipeline(
    env: PipelineExecEnv,
    body: &Value,
    async_exec: bool,
) -> Result<PipelineRun, (u16, String)> {
    let routine_id = body["routine_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let mut routine_name = body["name"]
        .as_str()
        .unwrap_or("")
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(120)
        .collect::<String>();

    let steps = if body.get("steps_v2").is_some() && !body["steps_v2"].is_null() {
        if let Some(rid) = routine_id.as_deref() {
            if routine_name.is_empty() {
                if let Some(rt) = routines::get_routine(&env.cfg.data_dir, rid) {
                    routine_name = rt["name"].as_str().unwrap_or("Pipeline").to_string();
                }
            }
        }
        parse_steps_v2(&body["steps_v2"]).map_err(|e| (400, e))?
    } else if let Some(rid) = routine_id.as_deref() {
        let Some(rt) = routines::get_routine(&env.cfg.data_dir, rid) else {
            return Err((404, "routine not found".to_string()));
        };
        if routine_name.is_empty() {
            routine_name = rt["name"].as_str().unwrap_or("Pipeline").to_string();
        }
        if rt.get("steps_v2").is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty())) {
            parse_steps_v2(&rt["steps_v2"]).map_err(|e| (400, e))?
        } else {
            let legacy: Vec<String> = rt["steps"]
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let prompt = rt["prompt"].as_str().unwrap_or("");
            let converted = from_legacy_steps(&legacy, prompt);
            if converted.is_empty() {
                return Err((400, "routine has no executable steps".to_string()));
            }
            converted
        }
    } else {
        return Err((400, "provide either steps_v2 or routine_id".to_string()));
    };

    if routine_name.is_empty() {
        routine_name = "Pipeline".to_string();
    }
    let inputs = parse_inputs(&body["inputs"]);
    let now_ms = records::now_millis();
    let seq = PIPELINE_SEQ.fetch_add(1, Ordering::SeqCst);
    let run_id = format!("prun-{now_ms}-{seq}");
    let thread_id = {
        let mut st = env.store.lock().unwrap();
        let tid = st.create_thread();
        st.push(&tid, "user", &format!("Pipeline: {routine_name}"));
        tid
    };
    let step_states: Vec<PipelineStepState> = steps
        .iter()
        .map(PipelineStepState::pending_for)
        .collect();
    let engagement_id = body["engagement_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            records::active_engagement_for(&env.ctx.root_data_dir, &env.ctx.person_id)
                .map(|(e, _)| e.id)
        });
    let run = PipelineRun {
        run_id: run_id.clone(),
        routine_id,
        routine_name,
        thread_id,
        status: "running".to_string(),
        current_step_index: 0,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs,
        steps,
        step_states,
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id,
        usage: records::RunUsage::default(),
    };
    save_run(&env.cfg.data_dir, &run).map_err(|e| (500, e))?;

    if async_exec {
        spawn_pipeline_worker(env, run_id, 0);
        Ok(run)
    } else {
        execute_pipeline_loop(&env, &run_id, 0).map_err(|e| (500, e))
    }
}

pub fn resume_pipeline(
    env: PipelineExecEnv,
    run_id: &str,
    body: &Value,
    async_exec: bool,
) -> Result<PipelineRun, (u16, String)> {
    let action = body["action"]
        .as_str()
        .unwrap_or("approve")
        .trim()
        .to_ascii_lowercase();
    let choice = body["choice"].as_str().map(str::trim).filter(|s| !s.is_empty());
    let answer = body["answer"].as_str().map(str::trim).filter(|s| !s.is_empty());
    let override_args = body.get("override_args").and_then(|v| v.as_object()).cloned();
    let now_ms = records::now_millis();

    enum ResumeTransition {
        Interrupted {
            run: PipelineRun,
            start_idx: usize,
            completed_now: bool,
        },
        Rejected {
            run: PipelineRun,
            checkpoint_id: String,
            resolved_answer: String,
        },
        CheckpointApproved {
            run: PipelineRun,
            checkpoint_id: String,
            next_index: usize,
            resolved_answer: String,
        },
    }

    let transition = mutate_run_http(&env.cfg.data_dir, run_id, |r| {
        if r.status == "interrupted" && r.pending_checkpoint.is_none() {
            let start_idx = r.current_step_index.min(r.steps.len());
            let completed_now = start_idx >= r.steps.len();
            r.status = if completed_now {
                "completed".to_string()
            } else {
                "running".to_string()
            };
            r.current_step_index = start_idx;
            r.error = None;
            r.cancel_requested = false;
            r.updated_ms = now_ms;
            if let Some(st) = r.step_states.get_mut(start_idx) {
                if st.status == "failed" {
                    *st = PipelineStepState::pending_for(&r.steps[start_idx]);
                }
            }
            return Ok(ResumeTransition::Interrupted {
                run: r.clone(),
                start_idx,
                completed_now,
            });
        }

        if r.status != "paused_checkpoint" {
            return Err((
                409,
                format!("pipeline run is '{}', not paused at a checkpoint", r.status),
            ));
        }
        let cp = r
            .pending_checkpoint
            .clone()
            .ok_or_else(|| (409, "pipeline run has no pending checkpoint".to_string()))?;

        if !matches!(action.as_str(), "approve" | "reject" | "answer") {
            return Err((400, "action must be approve, reject, or answer".to_string()));
        }
        let resolved_answer = choice
            .or(answer)
            .map(str::to_string)
            .unwrap_or_else(|| {
                if action == "reject" {
                    "rejected".to_string()
                } else {
                    "approved".to_string()
                }
            });

        if cp.input_mode == "choice"
            && action != "reject"
            && !cp.choices.is_empty()
            && choice.is_none()
            && answer.is_none()
        {
            return Err((400, "choice or answer is required for this checkpoint".to_string()));
        }
        if cp.input_mode == "text" && action != "reject" && answer.is_none() && choice.is_none() {
            return Err((400, "answer is required for this text checkpoint".to_string()));
        }

        if action == "reject" {
            r.status = "cancelled".to_string();
            r.updated_ms = now_ms;
            r.pending_checkpoint = None;
            r.cancel_requested = true;
            r.error = Some("Aborted at human checkpoint.".to_string());
            if let Some(st) = r.step_states.get_mut(cp.step_index) {
                st.status = "failed".to_string();
                st.finished_ms = now_ms;
                st.human_action = Some(action.clone());
                st.human_answer = Some(resolved_answer.clone());
                st.error = Some("Aborted by user at checkpoint.".to_string());
            }
            return Ok(ResumeTransition::Rejected {
                run: r.clone(),
                checkpoint_id: cp.checkpoint_id,
                resolved_answer,
            });
        }

        let next_index = cp.step_index + 1;
        r.status = if next_index >= r.steps.len() {
            "completed".to_string()
        } else {
            "running".to_string()
        };
        r.current_step_index = next_index;
        r.updated_ms = now_ms;
        r.pending_checkpoint = None;
        r.error = None;
        r.cancel_requested = false;
        if let Some(st) = r.step_states.get_mut(cp.step_index) {
            st.status = "ok".to_string();
            st.finished_ms = now_ms;
            st.human_action = Some(action.clone());
            st.human_answer = Some(resolved_answer.clone());
            st.output = resolved_answer.clone();
            st.error = None;
        }
        if let Some(overrides) = &override_args {
            if let Some(next_step) = r.steps.get_mut(next_index) {
                if let Some(args_obj) = next_step.args.as_object_mut() {
                    for (k, v) in overrides {
                        args_obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        Ok(ResumeTransition::CheckpointApproved {
            run: r.clone(),
            checkpoint_id: cp.checkpoint_id,
            next_index,
            resolved_answer,
        })
    })?;

    match transition {
        ResumeTransition::Interrupted {
            run,
            start_idx,
            completed_now,
        } => {
            env.events.publish(
                "pipeline_resumed",
                &env.ctx.person_id,
                &run.run_id,
                &run.thread_id,
                json!({
                    "step_index": start_idx,
                    "action": "resume",
                    "human_answer": Value::Null,
                    "run": run.to_json(),
                    "ts_ms": now_ms,
                }),
            );
            if completed_now {
                env.events.publish(
                    "pipeline_finished",
                    &env.ctx.person_id,
                    &run.run_id,
                    &run.thread_id,
                    json!({
                        "status": "completed",
                        "error": Value::Null,
                        "run": run.to_json(),
                        "ts_ms": now_ms,
                    }),
                );
                return Ok(run);
            }
            if async_exec {
                spawn_pipeline_worker(env, run_id.to_string(), start_idx);
                Ok(run)
            } else {
                execute_pipeline_loop(&env, run_id, start_idx).map_err(|e| (500, e))
            }
        }
        ResumeTransition::Rejected {
            run,
            checkpoint_id,
            resolved_answer,
        } => {
            if !checkpoint_id.is_empty() {
                let _ = records::resolve_decision(&env.cfg, &checkpoint_id, false);
                let outcome_text = format!("Aborted at checkpoint: {resolved_answer}");
                let _ = records::set_decision_outcome(&env.cfg.data_dir, &checkpoint_id, &outcome_text);
            }
            env.store.lock().unwrap().push(
                &run.thread_id,
                "step",
                &format!("Checkpoint rejected ({resolved_answer}) — pipeline aborted."),
            );
            env.events.publish(
                "pipeline_finished",
                &env.ctx.person_id,
                &run.run_id,
                &run.thread_id,
                json!({
                    "status": "cancelled",
                    "error": run.error,
                    "run": run.to_json(),
                    "ts_ms": now_ms,
                }),
            );
            Ok(run)
        }
        ResumeTransition::CheckpointApproved {
            run,
            checkpoint_id,
            next_index,
            resolved_answer,
        } => {
            if !checkpoint_id.is_empty() {
                let _ = records::resolve_decision(&env.cfg, &checkpoint_id, true);
                let outcome_text = format!("Resumed ({action}): {resolved_answer}");
                let _ = records::set_decision_outcome(&env.cfg.data_dir, &checkpoint_id, &outcome_text);
            }
            env.store.lock().unwrap().push(
                &run.thread_id,
                "step",
                &format!("Checkpoint settled ({action}: {resolved_answer}) — resuming pipeline."),
            );
            env.events.publish(
                "pipeline_resumed",
                &env.ctx.person_id,
                &run.run_id,
                &run.thread_id,
                json!({
                    "step_index": next_index,
                    "action": action,
                    "human_answer": resolved_answer,
                    "run": run.to_json(),
                    "ts_ms": now_ms,
                }),
            );

            if next_index >= run.steps.len() {
                env.events.publish(
                    "pipeline_finished",
                    &env.ctx.person_id,
                    &run.run_id,
                    &run.thread_id,
                    json!({
                        "status": "completed",
                        "error": Value::Null,
                        "run": run.to_json(),
                        "ts_ms": now_ms,
                    }),
                );
                return Ok(run);
            }

            if async_exec {
                spawn_pipeline_worker(env, run_id.to_string(), next_index);
                Ok(run)
            } else {
                execute_pipeline_loop(&env, run_id, next_index).map_err(|e| (500, e))
            }
        }
    }
}

pub fn retry_pipeline(
    env: PipelineExecEnv,
    run_id: &str,
    from_step_index: Option<usize>,
    async_exec: bool,
) -> Result<PipelineRun, (u16, String)> {
    let now_ms = records::now_millis();
    let (updated, idx) = mutate_run_http(&env.cfg.data_dir, run_id, |r| {
        if !matches!(r.status.as_str(), "failed" | "interrupted" | "cancelled") {
            return Err((
                409,
                format!("cannot retry pipeline run in '{}' state", r.status),
            ));
        }
        let default_idx = r
            .step_states
            .iter()
            .position(|s| s.status == "failed")
            .unwrap_or(r.current_step_index);
        let idx = from_step_index.unwrap_or(default_idx);
        if idx >= r.steps.len() {
            return Err((400, "from_step_index out of range".to_string()));
        }
        r.status = "running".to_string();
        r.current_step_index = idx;
        r.updated_ms = now_ms;
        r.pending_checkpoint = None;
        r.error = None;
        r.cancel_requested = false;
        for i in idx..r.step_states.len() {
            r.step_states[i] = PipelineStepState::pending_for(&r.steps[i]);
        }
        Ok((r.clone(), idx))
    })?;

    env.store.lock().unwrap().push(
        &updated.thread_id,
        "step",
        &format!("Retrying pipeline from step {}.", idx + 1),
    );
    env.events.publish(
        "pipeline_resumed",
        &env.ctx.person_id,
        &updated.run_id,
        &updated.thread_id,
        json!({
            "step_index": idx,
            "action": "retry",
            "human_answer": Value::Null,
            "run": updated.to_json(),
            "ts_ms": now_ms,
        }),
    );

    if async_exec {
        spawn_pipeline_worker(env, run_id.to_string(), idx);
        Ok(updated)
    } else {
        execute_pipeline_loop(&env, run_id, idx).map_err(|e| (500, e))
    }
}

pub fn cancel_pipeline(env: &PipelineExecEnv, run_id: &str) -> Result<PipelineRun, (u16, String)> {
    if let Some(flag) = cancel_registry()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(run_id)
    {
        flag.store(true, Ordering::SeqCst);
    }

    let now_ms = records::now_millis();
    let (updated, was_paused_or_interrupted, linked_cp_id) =
        mutate_run_http(&env.cfg.data_dir, run_id, |r| {
            if matches!(r.status.as_str(), "completed" | "failed" | "cancelled") {
                return Ok((r.clone(), false, None));
            }
            r.cancel_requested = true;
            r.updated_ms = now_ms;
            let mut transitioned = false;
            let mut cp_id = None;
            if matches!(r.status.as_str(), "paused_checkpoint" | "interrupted") {
                transitioned = true;
                r.status = "cancelled".to_string();
                if let Some(cp) = r.pending_checkpoint.take() {
                    cp_id = Some(cp.checkpoint_id);
                }
                if let Some(st) = r.step_states.get_mut(r.current_step_index) {
                    if st.status == "waiting_human" || st.status == "running" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Cancelled by user.".to_string());
                    }
                }
            }
            Ok((r.clone(), transitioned, cp_id))
        })?;

    if let Some(did) = linked_cp_id {
        let _ = records::resolve_decision(&env.cfg, &did, false);
    }
    if let Ok(runs) = env.live_runs.lock() {
        if let Some(live) = runs.get(&env.ctx.person_id) {
            if live.thread_id == updated.thread_id {
                live.cancel.store(true, Ordering::SeqCst);
            }
        }
    }
    if was_paused_or_interrupted {
        env.events.publish(
            "pipeline_finished",
            &env.ctx.person_id,
            &updated.run_id,
            &updated.thread_id,
            json!({
                "status": "cancelled",
                "error": Value::Null,
                "run": updated.to_json(),
                "ts_ms": now_ms,
            }),
        );
    }
    Ok(updated)
}

/// Park an ad-hoc chat/agent run that called `request_checkpoint` as a
/// resumable `PipelineRun` + linked decision.
pub fn park_adhoc_checkpoint(
    cfg: &Config,
    thread_id: &str,
    task: &str,
    written_files: &[String],
    steps_taken: &[String],
    cp: &CheckpointPause,
    live: Option<(&RunEventBus, &str)>,
) -> Result<(String, PendingCheckpoint), String> {
    let now_ms = records::now_millis();
    let seq = PIPELINE_SEQ.fetch_add(1, Ordering::SeqCst);
    let run_id = format!("prun-{now_ms}-{seq}");
    let safe_inspect: Vec<String> = cp
        .inspect_files
        .iter()
        .filter(|p| sandbox_path(&cfg.workspace_dir, p).is_ok())
        .cloned()
        .collect();
    let decision_id = records::add_decision_with(
        &cfg.data_dir,
        "checkpoint",
        &cp.prompt,
        safe_inspect.first().map(String::as_str),
        thread_id,
        json!({
            "pipeline_run_id": run_id,
            "step_index": 1,
            "step_id": "checkpoint",
            "input_mode": cp.input_mode,
            "choices": cp.choices,
            "inspect_files": safe_inspect,
        }),
    );

    let step0 = PipelineStepSpec {
        id: "initial".to_string(),
        label: task.chars().take(80).collect(),
        kind: "agent".to_string(),
        skill_id: None,
        args: json!({}),
        prompt: Some(task.to_string()),
        checkpoint: None,
        export: None,
        when: "always".to_string(),
    };
    let step1 = PipelineStepSpec {
        id: "checkpoint".to_string(),
        label: cp.prompt.chars().take(80).collect(),
        kind: "checkpoint".to_string(),
        skill_id: None,
        args: json!({}),
        prompt: Some(cp.prompt.clone()),
        checkpoint: Some(PipelineCheckpointSpec {
            prompt: cp.prompt.clone(),
            input_mode: cp.input_mode.clone(),
            choices: cp.choices.clone(),
            inspect_files: safe_inspect.clone(),
        }),
        export: None,
        when: "always".to_string(),
    };
    let step2 = PipelineStepSpec {
        id: "continue".to_string(),
        label: "Continue after checkpoint".to_string(),
        kind: "agent".to_string(),
        skill_id: None,
        args: json!({}),
        prompt: Some(format!(
            "The user answered the checkpoint (\"{}\") with: {{{{steps.checkpoint.human_answer}}}} (action: {{{{steps.checkpoint.human_action}}}}). Continue and complete the original request: {task}",
            cp.prompt
        )),
        checkpoint: None,
        export: None,
        when: "on_previous_ok".to_string(),
    };

    let state0 = PipelineStepState {
        step_id: "initial".to_string(),
        label: step0.label.clone(),
        kind: "agent".to_string(),
        status: "ok".to_string(),
        started_ms: now_ms,
        finished_ms: now_ms,
        output: steps_taken.join(" | "),
        files: written_files.to_vec(),
        metrics: json!({ "files_count": written_files.len() }),
        usage: None,
        checkpoint_id: None,
        human_answer: None,
        human_action: None,
        error: None,
    };
    let state1 = PipelineStepState {
        step_id: "checkpoint".to_string(),
        label: step1.label.clone(),
        kind: "checkpoint".to_string(),
        status: "waiting_human".to_string(),
        started_ms: now_ms,
        finished_ms: 0,
        output: format!("Waiting on human checkpoint: {}", cp.prompt),
        files: safe_inspect.clone(),
        metrics: json!({}),
        usage: None,
        checkpoint_id: Some(decision_id.clone()),
        human_answer: None,
        human_action: None,
        error: None,
    };
    let state2 = PipelineStepState::pending_for(&step2);

    let pending = PendingCheckpoint {
        checkpoint_id: decision_id.clone(),
        step_index: 1,
        step_id: "checkpoint".to_string(),
        prompt: cp.prompt.clone(),
        input_mode: cp.input_mode.clone(),
        choices: cp.choices.clone(),
        inspect_files: safe_inspect,
        created_ms: now_ms,
    };

    let run = PipelineRun {
        run_id: run_id.clone(),
        routine_id: None,
        routine_name: task.chars().take(80).collect(),
        thread_id: thread_id.to_string(),
        status: "paused_checkpoint".to_string(),
        current_step_index: 1,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs: HashMap::new(),
        steps: vec![step0, step1, step2],
        step_states: vec![state0, state1, state2],
        pending_checkpoint: Some(pending.clone()),
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: records::RunUsage::default(),
    };
    save_run(&cfg.data_dir, &run)?;

    if let Some((bus, person_id)) = live {
        bus.publish(
            "pipeline_paused",
            person_id,
            &run_id,
            thread_id,
            json!({
                "routine_name": run.routine_name,
                "step_index": 1,
                "step_id": "checkpoint",
                "checkpoint": pending.to_json(),
                "run": run.to_json(),
                "ts_ms": now_ms,
            }),
        );
    }

    Ok((run_id, pending))
}
