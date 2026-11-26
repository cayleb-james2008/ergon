//! Resumable Hybrid Workflow & Pipeline Engine (`pipeline_runs.json`).
//!
//! Executes typed multi-step pipelines (`skill`, `agent`, `checkpoint`, `export`)
//! asynchronously on worker threads, persists step-by-step state atomically,
//! pauses cleanly at human checkpoints (`paused_checkpoint`), survives server
//! restarts, and resumes downstream steps with `{{steps.<id>.human_answer}}`
//! and `{{steps.<id>.files[0]}}` template resolution.

use crate::connectors;
use crate::records;
use crate::routines;
use crate::search::SearchCache;
use crate::skills;
use crate::sse::RunEventBus;
use crate::tools::{self, sandbox_path, sandbox_path_resolving, ToolCtx};
use crate::{
    atomic_write_bytes, run_agent_for_pipeline_with_options, AgentLoopOptions, AgentSnapshot,
    CheckpointPause, Config, LiveRuns, SharedStore,
};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

pub const MAX_PIPELINE_STEPS: usize = 24;
pub const MAX_FANOUT_ITEMS: usize = 50;
pub const MAX_DAG_CONCURRENCY: usize = 4;
const MAX_STORED_RUNS: usize = 200;
pub const MAX_TEMPLATE_DEPTH: usize = 16;
pub const MAX_TEMPLATE_EXPANSIONS: usize = 128;
pub const MAX_TEMPLATE_OUTPUT_BYTES: usize = 256 * 1024;

const DAG_META_KEY: &str = "__ergon_dag";
const CHILD_ITEMS_KEY: &str = "__child_items";
const SNAPSHOT_INPUT_KEY: &str = "__ergon_agent_snapshot";
const ITEM_INPUT_KEY: &str = "__ergon_item";
const ITEM_INDEX_INPUT_KEY: &str = "__ergon_item_index";

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineStepKind {
    Skill,
    Agent,
    Checkpoint,
    Export,
    Deliver,
}

impl PipelineStepKind {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "skill" => Some(Self::Skill),
            "agent" => Some(Self::Agent),
            "checkpoint" => Some(Self::Checkpoint),
            "export" => Some(Self::Export),
            "deliver" => Some(Self::Deliver),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Agent => "agent",
            Self::Checkpoint => "checkpoint",
            Self::Export => "export",
            Self::Deliver => "deliver",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineDeliverSpec {
    pub connector_id: String,
    pub payload: Option<Value>,
    pub file: Option<String>,
}

impl PipelineDeliverSpec {
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("connector_id".to_string(), json!(self.connector_id));
        if let Some(p) = &self.payload {
            obj.insert("payload".to_string(), p.clone());
        }
        if let Some(f) = &self.file {
            obj.insert("file".to_string(), json!(f));
        }
        Value::Object(obj)
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let Some(obj) = v.as_object() else {
            return Err("deliver must be a JSON object".to_string());
        };
        let connector_id = obj
            .get("connector_id")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .chars()
            .take(128)
            .collect::<String>();
        if connector_id.is_empty() {
            return Err("deliver.connector_id is required".to_string());
        }
        let payload = obj
            .get("payload")
            .filter(|p| !p.is_null())
            .cloned();
        let file = obj
            .get("file")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().chars().take(512).collect::<String>())
            .filter(|s| !s.is_empty());
        Ok(Self {
            connector_id,
            payload,
            file,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineStepSpec {
    pub id: String,
    pub label: String,
    pub kind: String, // "skill" | "agent" | "checkpoint" | "export" | "deliver"
    pub skill_id: Option<String>,
    pub args: Value,
    pub prompt: Option<String>,
    pub checkpoint: Option<PipelineCheckpointSpec>,
    pub export: Option<PipelineExportSpec>,
    pub when: String, // "always" | "on_anomaly" | "on_previous_ok" | "on_all_ok" | "expr:<path> == <val>"
}

impl PipelineStepSpec {
    pub fn kind_enum(&self) -> Option<PipelineStepKind> {
        PipelineStepKind::parse(&self.kind)
    }

    pub fn depends_on(&self) -> Vec<String> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("depends_on"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn for_each(&self) -> Option<String> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("for_each"))
            .and_then(|f| f.as_str())
            .map(str::to_string)
            .filter(|s| !s.trim().is_empty())
    }

    pub fn deliver(&self) -> Option<PipelineDeliverSpec> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("deliver"))
            .and_then(|d| if d.is_null() { None } else { PipelineDeliverSpec::from_json(d).ok() })
    }

    pub fn clean_args(&self) -> Value {
        let Some(obj) = self.args.as_object() else {
            return self.args.clone();
        };
        if !obj.contains_key(DAG_META_KEY) {
            return self.args.clone();
        }
        let mut out = obj.clone();
        out.remove(DAG_META_KEY);
        Value::Object(out)
    }

    pub fn set_dag_meta(
        &mut self,
        depends_on: Vec<String>,
        for_each: Option<String>,
        deliver: Option<PipelineDeliverSpec>,
    ) {
        if !self.args.is_object() {
            self.args = json!({});
        }
        let obj = self.args.as_object_mut().unwrap();
        if depends_on.is_empty() && for_each.is_none() && deliver.is_none() {
            obj.remove(DAG_META_KEY);
            return;
        }
        let mut meta = Map::new();
        if !depends_on.is_empty() {
            meta.insert("depends_on".to_string(), json!(depends_on));
        }
        if let Some(fe) = for_each {
            meta.insert("for_each".to_string(), json!(fe));
        }
        if let Some(del) = deliver {
            meta.insert("deliver".to_string(), del.to_json());
        }
        obj.insert(DAG_META_KEY.to_string(), Value::Object(meta));
    }

    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), json!(self.id));
        obj.insert("label".to_string(), json!(self.label));
        obj.insert("kind".to_string(), json!(self.kind));
        if let Some(sk) = &self.skill_id {
            obj.insert("skill_id".to_string(), json!(sk));
        }
        let clean = self.clean_args();
        if clean.is_object() {
            obj.insert("args".to_string(), clean);
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
        let deps = self.depends_on();
        if !deps.is_empty() {
            obj.insert("depends_on".to_string(), json!(deps));
        }
        if let Some(fe) = self.for_each() {
            obj.insert("for_each".to_string(), json!(fe));
        }
        if let Some(del) = self.deliver() {
            obj.insert("deliver".to_string(), del.to_json());
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
        if PipelineStepKind::parse(&raw_kind).is_none() {
            return Err(format!(
                "steps_v2[{index}].kind '{raw_kind}' must be skill, agent, checkpoint, export, or deliver"
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
            Some(a) if a.is_object() => {
                let mut m = a.as_object().unwrap().clone();
                m.remove(DAG_META_KEY);
                Value::Object(m)
            }
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
        let deliver = match obj.get("deliver") {
            None | Some(Value::Null) => None,
            Some(del_val) => Some(PipelineDeliverSpec::from_json(del_val)?),
        };

        let depends_on: Vec<String> = match obj.get("depends_on") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(arr)) => {
                let mut deps = Vec::with_capacity(arr.len());
                for (di, d) in arr.iter().enumerate() {
                    let Some(raw) = d.as_str() else {
                        return Err(format!(
                            "steps_v2[{index}].depends_on[{di}] must be a string"
                        ));
                    };
                    let clean: String = raw
                        .trim()
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                        .take(64)
                        .collect();
                    if clean.is_empty() {
                        return Err(format!(
                            "steps_v2[{index}].depends_on[{di}] cannot be empty"
                        ));
                    }
                    if !deps.contains(&clean) {
                        deps.push(clean);
                    }
                }
                deps
            }
            Some(_) => return Err(format!("steps_v2[{index}].depends_on must be an array")),
        };

        let for_each: Option<String> = match obj.get("for_each") {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.chars().take(2048).collect())
                }
            }
            Some(Value::Array(arr)) => {
                if arr.is_empty() {
                    return Err(format!(
                        "steps_v2[{index}].for_each array cannot be empty"
                    ));
                }
                if arr.len() > MAX_FANOUT_ITEMS {
                    return Err(format!(
                        "steps_v2[{index}].for_each exceeds MAX_FANOUT_ITEMS ({MAX_FANOUT_ITEMS})"
                    ));
                }
                Some(Value::Array(arr.clone()).to_string())
            }
            Some(_) => return Err(format!("steps_v2[{index}].for_each must be a string or array")),
        };

        if raw_kind == "checkpoint" && for_each.is_some() {
            return Err(format!(
                "steps_v2[{index}] of kind 'checkpoint' does not support for_each"
            ));
        }

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
            "deliver" if deliver.is_none() => {
                return Err(format!(
                    "steps_v2[{index}] of kind 'deliver' requires deliver.connector_id"
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
                "deliver" => deliver
                    .as_ref()
                    .map(|d| format!("Deliver via {}", d.connector_id))
                    .unwrap_or_else(|| "Deliver".to_string()),
                _ => prompt
                    .as_ref()
                    .map(|p| p.chars().take(80).collect())
                    .unwrap_or_else(|| id.clone()),
            });

        let raw_when_trimmed = obj
            .get("when")
            .and_then(|s| s.as_str())
            .unwrap_or("always")
            .trim();
        let raw_when_low = raw_when_trimmed.to_ascii_lowercase();
        let when = if raw_when_low.is_empty() || raw_when_low == "always" {
            "always".to_string()
        } else if matches!(
            raw_when_low.as_str(),
            "on_anomaly" | "on_previous_ok" | "on_all_ok"
        ) {
            raw_when_low
        } else if raw_when_low.starts_with("expr:") {
            let expr_body = raw_when_trimmed[5..].trim();
            if expr_body.is_empty() || (!expr_body.contains("==") && !expr_body.contains("!=")) {
                return Err(format!(
                    "steps_v2[{index}].when '{raw_when_trimmed}' must be expr:<path> == <val> or != <val>"
                ));
            }
            format!("expr:{expr_body}")
        } else {
            return Err(format!(
                "steps_v2[{index}].when '{raw_when_low}' must be always, on_anomaly, on_previous_ok, on_all_ok, or expr:<path> == <val>"
            ));
        };

        let mut spec = Self {
            id,
            label,
            kind: raw_kind,
            skill_id,
            args,
            prompt,
            checkpoint,
            export,
            when,
        };
        spec.set_dag_meta(depends_on, for_each, deliver);
        Ok(spec)
    }
}

pub fn topological_sort_steps(steps: &[PipelineStepSpec]) -> Result<Vec<String>, String> {
    let mut id_to_idx: HashMap<&str, usize> = HashMap::with_capacity(steps.len());
    for (i, s) in steps.iter().enumerate() {
        id_to_idx.insert(s.id.as_str(), i);
    }
    let has_explicit_dag = steps.iter().any(|s| !s.depends_on().is_empty());
    if !has_explicit_dag {
        return Ok(steps.iter().map(|s| s.id.clone()).collect());
    }

    let n = steps.len();
    let mut in_degree = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, step) in steps.iter().enumerate() {
        for dep in step.depends_on() {
            if dep == step.id {
                return Err(format!("step '{}' cannot depend on itself (cycle)", step.id));
            }
            let Some(&dep_idx) = id_to_idx.get(dep.as_str()) else {
                return Err(format!(
                    "step '{}' depends_on unknown step '{dep}'",
                    step.id
                ));
            };
            in_degree[i] += 1;
            dependents[dep_idx].push(i);
        }
    }

    let mut ready = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            ready.push_back(i);
        }
    }
    let mut order = Vec::with_capacity(n);
    while let Some(u) = ready.pop_front() {
        order.push(steps[u].id.clone());
        for &v in &dependents[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                ready.push_back(v);
            }
        }
    }
    if order.len() != n {
        return Err("pipeline steps contain a circular depends_on dependency cycle".to_string());
    }
    Ok(order)
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
    let mut raw_ids = HashSet::new();
    let mut dup_raw_id: Option<String> = None;
    let mut out = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        let mut step = PipelineStepSpec::from_json(item, idx)?;
        if !raw_ids.insert(step.id.clone()) && dup_raw_id.is_none() {
            dup_raw_id = Some(step.id.clone());
        }
        if !seen_ids.insert(step.id.clone()) {
            step.id = format!("{}_{}", step.id, idx + 1);
            seen_ids.insert(step.id.clone());
        }
        out.push(step);
    }
    if out.iter().any(|s| !s.depends_on().is_empty()) {
        if let Some(dup) = dup_raw_id {
            return Err(format!("duplicate step id '{dup}' in DAG pipeline"));
        }
        topological_sort_steps(&out)?;
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

    pub fn child_items(&self) -> Vec<PipelineStepState> {
        self.metrics
            .get(CHILD_ITEMS_KEY)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(PipelineStepState::from_json)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn set_child_items(&mut self, items: Vec<PipelineStepState>) {
        if !self.metrics.is_object() {
            self.metrics = json!({});
        }
        let obj = self.metrics.as_object_mut().unwrap();
        if items.is_empty() {
            obj.remove(CHILD_ITEMS_KEY);
        } else {
            let vals: Vec<Value> = items.iter().map(|c| c.to_json()).collect();
            obj.insert(CHILD_ITEMS_KEY.to_string(), Value::Array(vals));
        }
    }

    pub fn clean_metrics(&self) -> Value {
        let Some(obj) = self.metrics.as_object() else {
            return self.metrics.clone();
        };
        if !obj.contains_key(CHILD_ITEMS_KEY) {
            return self.metrics.clone();
        }
        let mut out = obj.clone();
        out.remove(CHILD_ITEMS_KEY);
        Value::Object(out)
    }

    pub fn to_json(&self) -> Value {
        let children: Vec<Value> = self.child_items().iter().map(|c| c.to_json()).collect();
        json!({
            "step_id": self.step_id,
            "label": self.label,
            "kind": self.kind,
            "status": self.status,
            "started_ms": self.started_ms,
            "finished_ms": self.finished_ms,
            "output": self.output,
            "files": self.files,
            "metrics": self.clean_metrics(),
            "child_items": children,
            "usage": self.usage.as_ref().map(|u| u.to_json()),
            "checkpoint_id": self.checkpoint_id,
            "human_answer": self.human_answer,
            "human_action": self.human_action,
            "error": self.error,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let mut metrics = if v["metrics"].is_object() {
            v["metrics"].clone()
        } else {
            json!({})
        };
        if let Some(arr) = v.get("child_items").and_then(|c| c.as_array()) {
            if !arr.is_empty() {
                if let Some(obj) = metrics.as_object_mut() {
                    obj.insert(CHILD_ITEMS_KEY.to_string(), Value::Array(arr.clone()));
                }
            }
        }
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
            metrics,
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
    pub fn agent_snapshot(&self) -> Option<AgentSnapshot> {
        self.inputs
            .get(SNAPSHOT_INPUT_KEY)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| AgentSnapshot::from_json(&v))
    }

    pub fn set_agent_snapshot(&mut self, snap: Option<AgentSnapshot>) {
        match snap {
            Some(s) => {
                self.inputs
                    .insert(SNAPSHOT_INPUT_KEY.to_string(), s.to_json().to_string());
            }
            None => {
                self.inputs.remove(SNAPSHOT_INPUT_KEY);
            }
        }
    }

    pub fn is_dag(&self) -> bool {
        self.steps.iter().any(|s| !s.depends_on().is_empty())
    }

    pub fn to_json(&self) -> Value {
        let clean_inputs: HashMap<String, String> = self
            .inputs
            .iter()
            .filter(|(k, _)| {
                !matches!(
                    k.as_str(),
                    SNAPSHOT_INPUT_KEY | ITEM_INPUT_KEY | ITEM_INDEX_INPUT_KEY
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut child_items_map = Map::new();
        for st in &self.step_states {
            let children = st.child_items();
            if !children.is_empty() {
                child_items_map.insert(
                    st.step_id.clone(),
                    Value::Array(children.iter().map(|c| c.to_json()).collect()),
                );
            }
        }
        let snapshot_val = self
            .agent_snapshot()
            .map(|s| s.to_json())
            .unwrap_or(Value::Null);
        json!({
            "run_id": self.run_id,
            "routine_id": self.routine_id,
            "routine_name": self.routine_name,
            "thread_id": self.thread_id,
            "status": self.status,
            "current_step_index": self.current_step_index,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
            "inputs": clean_inputs,
            "steps": self.steps.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "step_states": self.step_states.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
            "child_items": Value::Object(child_items_map),
            "pending_checkpoint": self.pending_checkpoint.as_ref().map(|c| c.to_json()),
            "agent_snapshot": snapshot_val,
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
                if matches!(
                    k.as_str(),
                    SNAPSHOT_INPUT_KEY | ITEM_INPUT_KEY | ITEM_INDEX_INPUT_KEY
                ) {
                    continue;
                }
                let s = match val {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                inputs.insert(k.clone(), s);
            }
        }
        if let Some(snap_val) = v.get("agent_snapshot").filter(|s| s.is_object()) {
            if let Some(snap) = AgentSnapshot::from_json(snap_val) {
                inputs.insert(SNAPSHOT_INPUT_KEY.to_string(), snap.to_json().to_string());
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

pub fn find_run_by_checkpoint_id(person_data_dir: &Path, checkpoint_id: &str) -> Option<PipelineRun> {
    with_pipelines_lock(|| {
        load_runs_unlocked(person_data_dir)
            .into_iter()
            .rev()
            .find(|r| {
                r.status == "paused_checkpoint"
                    && r.pending_checkpoint
                        .as_ref()
                        .is_some_and(|cp| cp.checkpoint_id == checkpoint_id)
            })
    })
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
    if run.is_dag() {
        if id == "prev" || id == "previous" {
            return run
                .step_states
                .iter()
                .filter(|s| matches!(s.status.as_str(), "ok" | "waiting_human"))
                .max_by_key(|s| s.finished_ms);
        }
        return run.step_states.iter().find(|s| {
            s.step_id == id && matches!(s.status.as_str(), "ok" | "waiting_human" | "skipped")
        });
    }
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

fn child_items_value(state: &PipelineStepState) -> Value {
    let arr: Vec<Value> = state
        .child_items()
        .into_iter()
        .map(|c| {
            json!({
                "step_id": c.step_id,
                "status": c.status,
                "output": c.output,
                "files": c.files,
                "metrics": c.clean_metrics(),
            })
        })
        .collect();
    Value::Array(arr)
}

fn resolve_nested_json_field(mut cur: &Value, rest: &str) -> Value {
    let parts: Vec<&str> = rest.split('.').map(str::trim).take(9).collect();
    if parts.len() > 8 {
        return json!("");
    }
    for seg in parts {
        if seg.is_empty() {
            return json!("");
        }
        if let Some((name, idx_str)) = seg.split_once('[').and_then(|(n, r)| r.strip_suffix(']').map(|i| (n, i))) {
            if !name.is_empty() {
                cur = match cur.get(name) {
                    Some(v) => v,
                    None => return json!(""),
                };
            }
            let Ok(idx) = idx_str.trim().parse::<usize>() else {
                return json!("");
            };
            cur = match cur.get(idx) {
                Some(v) => v,
                None => return json!(""),
            };
        } else {
            cur = match cur.get(seg) {
                Some(v) => v,
                None => return json!(""),
            };
        }
    }
    cur.clone()
}

fn resolve_expr(expr: &str, run: &PipelineRun) -> Option<Value> {
    let trimmed = expr.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || trimmed.contains("{{") || trimmed.contains("}}") {
        return None;
    }
    if trimmed == "item" {
        if let Some(raw) = run.inputs.get(ITEM_INPUT_KEY) {
            return Some(serde_json::from_str::<Value>(raw).unwrap_or_else(|_| json!(raw)));
        }
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("item.") {
        if rest.trim() == "index" {
            if let Some(idx_str) = run.inputs.get(ITEM_INDEX_INPUT_KEY) {
                if let Ok(idx) = idx_str.parse::<u64>() {
                    return Some(json!(idx));
                }
                return Some(json!(idx_str));
            }
        }
        if let Some(raw) = run.inputs.get(ITEM_INPUT_KEY) {
            let item_val = serde_json::from_str::<Value>(raw).unwrap_or_else(|_| json!(raw));
            return Some(resolve_nested_json_field(&item_val, rest.trim()));
        }
        return None;
    }
    if let Some(key) = trimmed.strip_prefix("inputs.") {
        let k = key.trim();
        if matches!(
            k,
            SNAPSHOT_INPUT_KEY | ITEM_INPUT_KEY | ITEM_INDEX_INPUT_KEY
        ) {
            return Some(json!(""));
        }
        return Some(json!(run.inputs.get(k).cloned().unwrap_or_default()));
    }
    if let Some(rest) = trimmed.strip_prefix("steps.") {
        let (step_id, field) = rest.split_once('.')?;
        let state = find_step_state(run, step_id.trim())?;
        let field = field.trim();
        if field == "status" {
            return Some(json!(state.status));
        }
        if field == "files" {
            return Some(json!(state.files));
        }
        if let Some(idx_part) = field.strip_prefix("files[").and_then(|s| s.strip_suffix(']')) {
            if let Ok(idx) = idx_part.trim().parse::<usize>() {
                return Some(json!(state.files.get(idx).cloned().unwrap_or_default()));
            }
        }
        if field == "items" {
            return Some(child_items_value(state));
        }
        if let Some(rest_items) = field.strip_prefix("items") {
            if rest_items.starts_with('[') {
                let items_val = child_items_value(state);
                return Some(resolve_nested_json_field(&items_val, rest_items));
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
            let clean = state.clean_metrics();
            return Some(resolve_nested_json_field(&clean, metric_key));
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

fn eval_when_expr(expr_raw: &str, run: &PipelineRun) -> bool {
    let body = expr_raw.strip_prefix("expr:").unwrap_or(expr_raw).trim();
    let (lhs_raw, rhs_raw, negate) = if let Some((l, r)) = body.split_once("==") {
        (l.trim(), r.trim(), false)
    } else if let Some((l, r)) = body.split_once("!=") {
        (l.trim(), r.trim(), true)
    } else {
        return false;
    };
    let lhs_clean = lhs_raw
        .strip_prefix("{{")
        .and_then(|s| s.strip_suffix("}}"))
        .map(str::trim)
        .unwrap_or(lhs_raw);
    let rhs_clean = rhs_raw
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| rhs_raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(rhs_raw)
        .trim();
    let Some(resolved) = resolve_expr(lhs_clean, run) else {
        return negate;
    };
    let eq = match &resolved {
        Value::String(s) => s.trim() == rhs_clean,
        Value::Number(n) => {
            if n.to_string() == rhs_clean {
                true
            } else if let (Some(a), Ok(b)) = (n.as_f64(), rhs_clean.parse::<f64>()) {
                (a - b).abs() < 1e-9
            } else {
                false
            }
        }
        Value::Bool(b) => (if *b { "true" } else { "false" }) == rhs_clean.to_ascii_lowercase(),
        Value::Null => rhs_clean.eq_ignore_ascii_case("null") || rhs_clean.is_empty(),
        other => {
            let s = other.to_string();
            s.as_str() == rhs_clean
        }
    };
    if negate { !eq } else { eq }
}

fn collect_relevant_prior_states<'a>(
    step: &PipelineStepSpec,
    idx: usize,
    run: &'a PipelineRun,
) -> Vec<&'a PipelineStepState> {
    let deps = step.depends_on();
    if !deps.is_empty() {
        run.step_states
            .iter()
            .filter(|s| deps.iter().any(|d| d == &s.step_id))
            .collect()
    } else if run.is_dag() {
        run.step_states
            .iter()
            .filter(|s| matches!(s.status.as_str(), "ok" | "skipped" | "failed" | "waiting_human"))
            .collect()
    } else {
        run.step_states[..idx.min(run.step_states.len())].iter().collect()
    }
}

fn should_run_step_for_run(
    step: &PipelineStepSpec,
    idx: usize,
    run: &PipelineRun,
) -> (bool, &'static str) {
    let prior = collect_relevant_prior_states(step, idx, run);
    match step.when.as_str() {
        "on_previous_ok" => {
            let deps = step.depends_on();
            if !deps.is_empty() {
                if prior.iter().any(|s| s.status == "failed") {
                    (false, "Skipped because upstream dependency failed.")
                } else if prior.iter().any(|s| s.status == "ok") {
                    (true, "")
                } else {
                    (false, "Skipped because upstream dependency did not succeed.")
                }
            } else {
                let prev = prior.into_iter().rev().find(|s| s.status != "skipped");
                match prev {
                    None => (true, ""),
                    Some(s) if s.status == "ok" => (true, ""),
                    Some(_) => (false, "Skipped because previous step did not succeed."),
                }
            }
        }
        "on_all_ok" => {
            if prior.is_empty() || prior.iter().all(|s| s.status == "ok") {
                (true, "")
            } else {
                (false, "Skipped: not all upstream steps succeeded.")
            }
        }
        "on_anomaly" => {
            let any_anomaly = prior.iter().any(|s| {
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
        w if w.starts_with("expr:") => {
            if eval_when_expr(w, run) {
                (true, "")
            } else {
                (false, "Skipped: condition expression evaluated to false.")
            }
        }
        _ => (true, ""),
    }
}

fn resolve_for_each_items(tpl: &str, run: &PipelineRun) -> Result<Vec<Value>, String> {
    let trimmed = tpl.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let resolved = resolve_template_value(&Value::String(trimmed.to_string()), run);
    let items: Vec<Value> = match resolved {
        Value::Array(arr) => arr,
        Value::String(s) => {
            let st = s.trim();
            if st.is_empty() {
                Vec::new()
            } else if st.starts_with('[') {
                serde_json::from_str::<Vec<Value>>(st)
                    .map_err(|e| format!("for_each JSON array parse failed: {e}"))?
            } else {
                st.lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(|l| Value::String(l.to_string()))
                    .collect()
            }
        }
        other => vec![other],
    };
    if items.len() > MAX_FANOUT_ITEMS {
        return Err(format!(
            "for_each resolved to {} items, exceeding MAX_FANOUT_ITEMS ({MAX_FANOUT_ITEMS})",
            items.len()
        ));
    }
    Ok(items)
}

fn scan_templates_in_str(
    tpl: &str,
    step_id: &str,
    step_ids: &HashSet<&str>,
    ancestors: Option<&HashSet<String>>,
    errors: &mut Vec<String>,
) {
    let mut rest = tpl;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            errors.push(format!("step '{step_id}': unclosed template '{{{{'"));
            return;
        };
        let expr = after[..end].trim();
        if expr.contains("{{") {
            errors.push(format!("step '{step_id}': nested '{{{{' inside template"));
            return;
        }
        if let Some(s_rest) = expr.strip_prefix("steps.") {
            let target = s_rest
                .split('.')
                .next()
                .map(str::trim)
                .unwrap_or("");
            if target.is_empty() {
                errors.push(format!("step '{step_id}': empty step reference in '{{{{{expr}}}}}'"));
            } else if target != "prev" && target != "previous" {
                if !step_ids.contains(target) {
                    errors.push(format!(
                        "step '{step_id}': template references unknown step '{target}'"
                    ));
                } else if target == step_id {
                    errors.push(format!(
                        "step '{step_id}': template cannot reference itself"
                    ));
                } else if let Some(anc) = ancestors {
                    if !anc.contains(target) {
                        errors.push(format!(
                            "step '{step_id}': template references '{target}' which is not an upstream dependency"
                        ));
                    }
                }
            }
        }
        rest = &after[end + 2..];
    }
}

fn scan_templates_in_value(
    v: &Value,
    step_id: &str,
    step_ids: &HashSet<&str>,
    ancestors: Option<&HashSet<String>>,
    errors: &mut Vec<String>,
) {
    match v {
        Value::String(s) => scan_templates_in_str(s, step_id, step_ids, ancestors, errors),
        Value::Array(arr) => {
            for item in arr {
                scan_templates_in_value(item, step_id, step_ids, ancestors, errors);
            }
        }
        Value::Object(map) => {
            for val in map.values() {
                scan_templates_in_value(val, step_id, step_ids, ancestors, errors);
            }
        }
        _ => {}
    }
}

fn compute_ancestors_map(steps: &[PipelineStepSpec]) -> HashMap<String, HashSet<String>> {
    let mut deps_map: HashMap<&str, Vec<String>> = HashMap::new();
    for s in steps {
        deps_map.insert(s.id.as_str(), s.depends_on());
    }
    let mut out = HashMap::new();
    for s in steps {
        let mut seen = HashSet::new();
        let mut stack = s.depends_on();
        while let Some(d) = stack.pop() {
            if seen.insert(d.clone()) {
                if let Some(parent_deps) = deps_map.get(d.as_str()) {
                    stack.extend(parent_deps.iter().cloned());
                }
            }
        }
        out.insert(s.id.clone(), seen);
    }
    out
}

/// Validate a pipeline payload (`steps_v2` or `routine_id`, optional `inputs`)
/// before saving or launching: checks DAG cycles, `depends_on` targets,
/// `for_each`/`deliver` specs, templates, and connector/skill existence.
pub fn validate_pipeline_payload(
    cfg: &Config,
    library: &Arc<Mutex<crate::library::Library>>,
    body: &Value,
) -> (u16, Value) {
    let mut errors: Vec<String> = Vec::new();
    let steps_res = if body.get("steps_v2").is_some() && !body["steps_v2"].is_null() {
        parse_steps_v2(&body["steps_v2"])
    } else if let Some(rid) = body["routine_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(rt) = routines::get_routine(&cfg.data_dir, rid) {
            if rt.get("steps_v2").is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty())) {
                parse_steps_v2(&rt["steps_v2"])
            } else {
                let legacy: Vec<String> = rt["steps"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                let prompt = rt["prompt"].as_str().unwrap_or("");
                let converted = from_legacy_steps(&legacy, prompt);
                if converted.is_empty() {
                    Err("routine has no executable steps".to_string())
                } else {
                    Ok(converted)
                }
            }
        } else {
            Err("routine not found".to_string())
        }
    } else {
        Err("provide either steps_v2 or routine_id".to_string())
    };

    let steps = match steps_res {
        Ok(s) => s,
        Err(e) => {
            errors.push(e);
            return (
                400,
                json!({
                    "ok": false,
                    "valid": false,
                    "errors": errors,
                    "topological_order": [],
                }),
            );
        }
    };

    // Check explicit duplicate IDs in raw steps_v2 if provided
    if let Some(raw_arr) = body.get("steps_v2").and_then(|v| v.as_array()) {
        let mut raw_ids = HashSet::new();
        for (idx, item) in raw_arr.iter().enumerate() {
            if let Some(id) = item.get("id").and_then(|s| s.as_str()).map(str::trim).filter(|s| !s.is_empty()) {
                if !raw_ids.insert(id.to_string()) {
                    errors.push(format!("steps_v2[{idx}]: duplicate step id '{id}'"));
                }
            }
        }
    }

    let topo_order = match topological_sort_steps(&steps) {
        Ok(ord) => ord,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    let is_dag = steps.iter().any(|s| !s.depends_on().is_empty());
    let step_ids: HashSet<&str> = steps.iter().map(|s| s.id.as_str()).collect();
    let ancestors_map = compute_ancestors_map(&steps);
    let inputs = parse_inputs(&body["inputs"]);
    let dummy_run = PipelineRun {
        run_id: String::new(),
        routine_id: None,
        routine_name: String::new(),
        thread_id: String::new(),
        status: "pending".to_string(),
        current_step_index: 0,
        created_ms: 0,
        updated_ms: 0,
        inputs,
        steps: steps.clone(),
        step_states: Vec::new(),
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: records::RunUsage::default(),
    };

    let policy = crate::load_workspace_policy(&cfg.data_dir);
    let connectors_json = connectors::list_connectors_json(&cfg.data_dir, &policy);
    let outbound = connectors_json["outbound"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    for (idx, step) in steps.iter().enumerate() {
        let anc = if is_dag {
            ancestors_map.get(&step.id)
        } else {
            None
        };
        let mut seq_anc = HashSet::new();
        let anc_ref = if is_dag {
            anc
        } else {
            for prior in &steps[..idx] {
                seq_anc.insert(prior.id.clone());
            }
            Some(&seq_anc)
        };

        if let Some(p) = &step.prompt {
            scan_templates_in_str(p, &step.id, &step_ids, anc_ref, &mut errors);
        }
        scan_templates_in_value(&step.clean_args(), &step.id, &step_ids, anc_ref, &mut errors);
        if let Some(ex) = &step.export {
            scan_templates_in_str(&ex.path, &step.id, &step_ids, anc_ref, &mut errors);
            if let Some(o) = &ex.output {
                scan_templates_in_str(o, &step.id, &step_ids, anc_ref, &mut errors);
            }
        }
        if let Some(del) = step.deliver() {
            scan_templates_in_str(&del.connector_id, &step.id, &step_ids, anc_ref, &mut errors);
            if let Some(p) = &del.payload {
                scan_templates_in_value(p, &step.id, &step_ids, anc_ref, &mut errors);
            }
            if let Some(f) = &del.file {
                scan_templates_in_str(f, &step.id, &step_ids, anc_ref, &mut errors);
            }
            if !del.connector_id.contains("{{") {
                match outbound.iter().find(|c| c["id"].as_str() == Some(del.connector_id.as_str())) {
                    None => errors.push(format!(
                        "step '{}': unknown outbound connector '{}'",
                        step.id, del.connector_id
                    )),
                    Some(c) if c["enabled"].as_bool() == Some(false) => errors.push(format!(
                        "step '{}': outbound connector '{}' is disabled",
                        step.id, del.connector_id
                    )),
                    Some(_) => {}
                }
            }
        }
        if step.kind == "skill" {
            if let Some(sk) = &step.skill_id {
                if !sk.contains("{{") && !skills::is_builtin_skill(sk) {
                    let exists_in_lib = library
                        .lock()
                        .unwrap()
                        .get(sk)
                        .is_some_and(|i| i.kind == "skill" && !i.builtin);
                    if !exists_in_lib {
                        errors.push(format!("step '{}': unknown skill '{sk}'", step.id));
                    }
                }
            }
        }
        if let Some(fe) = step.for_each() {
            scan_templates_in_str(&fe, &step.id, &step_ids, anc_ref, &mut errors);
            if !fe.contains("{{steps.") && !fe.contains("{{item") {
                let is_literal = !fe.contains("{{");
                let all_inputs_present = !is_literal && {
                    let mut all_ok = true;
                    let mut rest = fe.as_str();
                    while let Some(s) = rest.find("{{") {
                        let after = &rest[s + 2..];
                        if let Some(e) = after.find("}}") {
                            let expr = after[..e].trim();
                            if let Some(k) = expr.strip_prefix("inputs.") {
                                if !dummy_run.inputs.contains_key(k.trim()) {
                                    all_ok = false;
                                    break;
                                }
                            } else {
                                all_ok = false;
                                break;
                            }
                            rest = &after[e + 2..];
                        } else {
                            all_ok = false;
                            break;
                        }
                    }
                    all_ok
                };
                match resolve_for_each_items(&fe, &dummy_run) {
                    Ok(items) => {
                        if items.is_empty() && (is_literal || all_inputs_present) {
                            errors.push(format!(
                                "step '{}': for_each array cannot be empty",
                                step.id
                            ));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("step '{}': {e}", step.id));
                    }
                }
            }
        }
        if step.when.starts_with("expr:") {
            let body = step.when[5..].trim();
            let lhs = body
                .split_once("==")
                .or_else(|| body.split_once("!="))
                .map(|(l, _)| l.trim())
                .unwrap_or("");
            if let Some(s_rest) = lhs.strip_prefix("steps.") {
                let target = s_rest.split('.').next().map(str::trim).unwrap_or("");
                if !target.is_empty() && target != "prev" && target != "previous" {
                    if !step_ids.contains(target) {
                        errors.push(format!(
                            "step '{}': when expression references unknown step '{target}'",
                            step.id
                        ));
                    } else if let Some(a) = anc_ref {
                        if !a.contains(target) {
                            errors.push(format!(
                                "step '{}': when expression references '{target}' which is not an upstream dependency",
                                step.id
                            ));
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        (
            200,
            json!({
                "ok": true,
                "valid": true,
                "errors": [],
                "topological_order": topo_order,
            }),
        )
    } else {
        (
            400,
            json!({
                "ok": false,
                "valid": false,
                "errors": errors,
                "topological_order": [],
            }),
        )
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
        let (should_run, skip_reason) = should_run_step_for_run(&step, idx, &run);
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
                let resolved_args = resolve_template_value(&step.clean_args(), &run);
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

#[allow(clippy::too_many_arguments)]
fn park_pipeline_at_checkpoint_with_options(
    env: &PipelineExecEnv,
    run_id: &str,
    step_idx: usize,
    cp_spec: PipelineCheckpointSpec,
    prior_files: Vec<String>,
    prior_output: Option<String>,
    existing_decision_id: Option<String>,
    updated_snapshot: Option<AgentSnapshot>,
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

    let decision_id = match existing_decision_id {
        Some(did) if !did.is_empty() => did,
        _ => records::add_decision_with(
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
        ),
    };

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
        if let Some(ref snap) = updated_snapshot {
            r.set_agent_snapshot(Some(snap.clone()));
        }
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

enum DeliverStepOutcome {
    Delivered {
        output: String,
        files: Vec<String>,
        metrics: Value,
    },
    Held {
        decision_id: String,
        prompt: String,
        inspect_files: Vec<String>,
        output: String,
    },
}

fn execute_deliver_step(
    cfg: &Config,
    del: &PipelineDeliverSpec,
    run_snap: &PipelineRun,
) -> Result<DeliverStepOutcome, String> {
    let connector_id = resolve_template_str(&del.connector_id, run_snap)
        .trim()
        .to_string();
    if connector_id.is_empty() {
        return Err("deliver.connector_id resolved to empty string".to_string());
    }
    let attachment_rel = del
        .file
        .as_deref()
        .map(|f| resolve_template_str(f, run_snap))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref rel) = attachment_rel {
        let clean = crate::tables::validate_workspace_rel_path(rel)?;
        let unresolved = sandbox_path_resolving(&cfg.workspace_dir, &clean, false)?;
        if unresolved
            .symlink_metadata()
            .is_ok_and(|md| md.file_type().is_symlink())
        {
            return Err(format!("deliver: cannot attach symlink '{rel}'"));
        }
        sandbox_path(&cfg.workspace_dir, &clean)?;
    }

    let payload = if let Some(ref p) = del.payload {
        resolve_template_value(p, run_snap)
    } else if let Some(ref rel) = attachment_rel {
        let full = sandbox_path(&cfg.workspace_dir, rel)?;
        let md = std::fs::metadata(&full)
            .map_err(|e| format!("deliver cannot read file '{rel}': {e}"))?;
        if md.len() > 512 * 1024 {
            return Err(format!("deliver file '{rel}' exceeds 512 KB limit"));
        }
        let bytes = std::fs::read(&full)
            .map_err(|e| format!("deliver cannot read file '{rel}': {e}"))?;
        if bytes.len() > 512 * 1024 {
            return Err(format!("deliver file '{rel}' exceeds 512 KB limit"));
        }
        if rel.to_ascii_lowercase().ends_with(".json") {
            serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_| {
                json!({
                    "file": rel,
                    "content": String::from_utf8_lossy(&bytes).to_string(),
                })
            })
        } else {
            json!({
                "file": rel,
                "content": String::from_utf8_lossy(&bytes).to_string(),
            })
        }
    } else {
        json!({})
    };

    match connectors::invoke_connector(
        cfg,
        &connector_id,
        &payload,
        attachment_rel.as_deref(),
        &run_snap.thread_id,
    )? {
        connectors::InvokeOutcome::Delivered(log) => {
            let output = format!(
                "delivered via '{}' (HTTP {}, {} bytes, sha256:{})",
                log.connector_name,
                log.http_status.unwrap_or(200),
                log.payload_bytes,
                log.payload_sha256
            );
            let files = attachment_rel.into_iter().collect();
            let metrics = json!({
                "connector_id": log.connector_id,
                "delivery_id": log.id,
                "http_status": log.http_status,
                "payload_bytes": log.payload_bytes,
                "payload_sha256": log.payload_sha256,
                "anomalies": 0,
                "has_anomaly": false,
            });
            Ok(DeliverStepOutcome::Delivered {
                output,
                files,
                metrics,
            })
        }
        connectors::InvokeOutcome::Held {
            decision_id,
            delivery,
        } => {
            let prompt = format!(
                "Approve outbound delivery via \"{}\" ({})",
                delivery.connector_name, delivery.url
            );
            let output = format!(
                "Held for human approval (decision {decision_id}): outbound delivery to {} ({}) requires approval.",
                delivery.connector_name, delivery.url
            );
            let inspect_files = attachment_rel.into_iter().collect();
            Ok(DeliverStepOutcome::Held {
                decision_id,
                prompt,
                inspect_files,
                output,
            })
        }
    }
}

enum StepCoreOutcome {
    Ok {
        output: String,
        files: Vec<String>,
        metrics: Value,
        usage: records::RunUsage,
        usage_source: &'static str,
        child_items: Vec<PipelineStepState>,
        consume_snapshot: bool,
    },
    HeldCheckpoint {
        cp_spec: PipelineCheckpointSpec,
        prior_files: Vec<String>,
        prior_output: Option<String>,
        existing_decision_id: Option<String>,
        usage: Option<records::RunUsage>,
        updated_snapshot: Option<AgentSnapshot>,
    },
    Failed {
        error: String,
        usage: Option<records::RunUsage>,
        is_cancel: bool,
        child_items: Vec<PipelineStepState>,
    },
}

fn execute_single_step_core(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
) -> StepCoreOutcome {
    match step.kind.as_str() {
        "checkpoint" => {
            let cp_spec = build_checkpoint_from_step(step, run_snap);
            StepCoreOutcome::HeldCheckpoint {
                cp_spec,
                prior_files: Vec::new(),
                prior_output: None,
                existing_decision_id: None,
                usage: None,
                updated_snapshot: None,
            }
        }
        "skill" => {
            let skill_id = step.skill_id.as_deref().unwrap_or("");
            let resolved_args = resolve_template_value(&step.clean_args(), run_snap);
            match execute_skill_step(&env.cfg, Some(&env.ctx.library), skill_id, &resolved_args) {
                Ok((output, files)) => {
                    let metrics = extract_step_metrics(&env.cfg, &files, &output);
                    let (step_usage, _) = deterministic_step_usage(
                        &env.ctx.root_data_dir,
                        run_snap.engagement_id.as_deref(),
                        files.len(),
                    );
                    StepCoreOutcome::Ok {
                        output,
                        files,
                        metrics,
                        usage: step_usage,
                        usage_source: "pipeline_skill",
                        child_items: Vec::new(),
                        consume_snapshot: false,
                    }
                }
                Err(error) => StepCoreOutcome::Failed {
                    error,
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                },
            }
        }
        "export" => {
            let Some(ex) = step.export.as_ref() else {
                return StepCoreOutcome::Failed {
                    error: "export step missing spec".to_string(),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            };
            let src_path = resolve_template_str(&ex.path, run_snap);
            let out_path_opt = ex.output.as_deref().map(|o| resolve_template_str(o, run_snap));
            match tools::export_workspace_file(
                &env.cfg,
                &src_path,
                if ex.format.is_empty() {
                    None
                } else {
                    Some(ex.format.as_str())
                },
                out_path_opt.as_deref(),
            ) {
                Ok((out_path, bytes, what)) => {
                    let output = format!("exported {out_path} ({bytes} bytes, {what})");
                    let files = vec![out_path];
                    let metrics = json!({
                        "files_count": 1,
                        "bytes_total": bytes,
                        "anomalies": 0,
                        "has_anomaly": false,
                    });
                    let (step_usage, _) = deterministic_step_usage(
                        &env.ctx.root_data_dir,
                        run_snap.engagement_id.as_deref(),
                        files.len(),
                    );
                    StepCoreOutcome::Ok {
                        output,
                        files,
                        metrics,
                        usage: step_usage,
                        usage_source: "pipeline_export",
                        child_items: Vec::new(),
                        consume_snapshot: false,
                    }
                }
                Err(error) => StepCoreOutcome::Failed {
                    error,
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                },
            }
        }
        "deliver" => {
            let Some(del) = step.deliver() else {
                return StepCoreOutcome::Failed {
                    error: "deliver step missing deliver spec".to_string(),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            };
            match execute_deliver_step(&env.cfg, &del, run_snap) {
                Ok(DeliverStepOutcome::Delivered {
                    output,
                    files,
                    metrics,
                }) => {
                    let (step_usage, _) = deterministic_step_usage(
                        &env.ctx.root_data_dir,
                        run_snap.engagement_id.as_deref(),
                        files.len(),
                    );
                    StepCoreOutcome::Ok {
                        output,
                        files,
                        metrics,
                        usage: step_usage,
                        usage_source: "pipeline_deliver",
                        child_items: Vec::new(),
                        consume_snapshot: false,
                    }
                }
                Ok(DeliverStepOutcome::Held {
                    decision_id,
                    prompt,
                    inspect_files,
                    output,
                }) => StepCoreOutcome::HeldCheckpoint {
                    cp_spec: PipelineCheckpointSpec {
                        prompt,
                        input_mode: "approve_reject".to_string(),
                        choices: Vec::new(),
                        inspect_files,
                    },
                    prior_files: Vec::new(),
                    prior_output: Some(output),
                    existing_decision_id: Some(decision_id),
                    usage: None,
                    updated_snapshot: None,
                },
                Err(error) => StepCoreOutcome::Failed {
                    error,
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                },
            }
        }
        _ => {
            // "agent" step
            let raw_prompt = step.prompt.as_deref().unwrap_or(&step.label);
            let resolved_prompt = resolve_template_str(raw_prompt, run_snap);
            let snapshot = run_snap.agent_snapshot();
            let history = if snapshot.is_some() {
                Vec::new()
            } else {
                let st = env.store.lock().unwrap();
                st.history_json(&run_snap.thread_id)
            };
            let opts = AgentLoopOptions {
                snapshot,
                ..AgentLoopOptions::default()
            };
            match run_agent_for_pipeline_with_options(
                &env.cfg,
                &env.ctx,
                &env.store,
                &run_snap.thread_id,
                &resolved_prompt,
                &history,
                Some((&env.live_runs, &env.events, &env.ctx.person_id)),
                opts,
            ) {
                Ok(out) => {
                    if out.budget_gated {
                        return StepCoreOutcome::Failed {
                            error: out.summary,
                            usage: Some(out.usage),
                            is_cancel: false,
                            child_items: Vec::new(),
                        };
                    }
                    if let Some(cp_pause) = out.paused_checkpoint {
                        let cp_spec = PipelineCheckpointSpec {
                            prompt: cp_pause.prompt,
                            input_mode: cp_pause.input_mode,
                            choices: cp_pause.choices,
                            inspect_files: cp_pause.inspect_files,
                        };
                        return StepCoreOutcome::HeldCheckpoint {
                            cp_spec,
                            prior_files: out.written_files,
                            prior_output: Some(out.summary),
                            existing_decision_id: None,
                            usage: Some(out.usage),
                            updated_snapshot: out.agent_snapshot,
                        };
                    }
                    let metrics = extract_step_metrics(&env.cfg, &out.written_files, &out.summary);
                    StepCoreOutcome::Ok {
                        output: out.summary,
                        files: out.written_files,
                        metrics,
                        usage: out.usage,
                        usage_source: "pipeline_agent",
                        child_items: Vec::new(),
                        consume_snapshot: true,
                    }
                }
                Err(err) => {
                    let is_cancel = err == "run cancelled by user";
                    StepCoreOutcome::Failed {
                        error: err,
                        usage: None,
                        is_cancel,
                        child_items: Vec::new(),
                    }
                }
            }
        }
    }
}

fn execute_step_with_for_each(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
) -> StepCoreOutcome {
    let Some(fe_tpl) = step.for_each() else {
        return execute_single_step_core(env, step, run_snap);
    };
    let items = match resolve_for_each_items(&fe_tpl, run_snap) {
        Ok(it) => it,
        Err(error) => {
            return StepCoreOutcome::Failed {
                error,
                usage: None,
                is_cancel: false,
                child_items: Vec::new(),
            };
        }
    };
    if items.is_empty() {
        return StepCoreOutcome::Ok {
            output: "0 items processed".to_string(),
            files: Vec::new(),
            metrics: json!({ "items_count": 0, "files_count": 0, "anomalies": 0, "has_anomaly": false }),
            usage: records::RunUsage::default(),
            usage_source: "pipeline_skill",
            child_items: Vec::new(),
            consume_snapshot: false,
        };
    }

    let n = items.len();
    let max_workers = if step.kind == "deliver" {
        1
    } else {
        MAX_DAG_CONCURRENCY.min(n)
    };

    let (tx, rx) = mpsc::channel::<(usize, u64, u64, StepCoreOutcome)>();
    let mut next_to_spawn = 0usize;
    let mut in_flight = 0usize;
    let mut results: Vec<Option<(u64, u64, StepCoreOutcome)>> = (0..n).map(|_| None).collect();

    let spawn_item = |idx: usize,
                      item_val: Value,
                      tx_item: mpsc::Sender<(usize, u64, u64, StepCoreOutcome)>,
                      env_c: PipelineExecEnv,
                      step_c: PipelineStepSpec,
                      mut item_run: PipelineRun| {
        item_run
            .inputs
            .insert(ITEM_INPUT_KEY.to_string(), item_val.to_string());
        item_run
            .inputs
            .insert(ITEM_INDEX_INPUT_KEY.to_string(), idx.to_string());
        let _ = std::thread::Builder::new()
            .name(format!("ergon-fe-{}-{}", step_c.id, idx))
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                let s_ms = records::now_millis();
                let res = execute_single_step_core(&env_c, &step_c, &item_run);
                let f_ms = records::now_millis();
                let _ = tx_item.send((idx, s_ms, f_ms, res));
            });
    };

    let mut abort_rest = false;
    while next_to_spawn < n || in_flight > 0 {
        while !abort_rest && in_flight < max_workers && next_to_spawn < n {
            let idx = next_to_spawn;
            next_to_spawn += 1;
            in_flight += 1;
            spawn_item(
                idx,
                items[idx].clone(),
                tx.clone(),
                env.clone(),
                step.clone(),
                run_snap.clone(),
            );
        }
        if in_flight > 0 {
            match rx.recv() {
                Ok((idx, s_ms, f_ms, outcome)) => {
                    in_flight -= 1;
                    if matches!(
                        outcome,
                        StepCoreOutcome::Failed { .. } | StepCoreOutcome::HeldCheckpoint { .. }
                    ) {
                        abort_rest = true;
                    }
                    results[idx] = Some((s_ms, f_ms, outcome));
                }
                Err(_) => break,
            }
        }
    }

    let mut child_items = Vec::with_capacity(n);
    let mut agg_files: Vec<String> = Vec::new();
    let mut agg_outputs: Vec<String> = Vec::new();
    let mut agg_usage = records::RunUsage::default();
    let mut first_error: Option<(String, bool)> = None;
    let mut first_held: Option<StepCoreOutcome> = None;
    let mut total_anomalies = 0u64;

    for (i, slot) in results.into_iter().enumerate() {
        let child_id = format!("{}[{i}]", step.id);
        let child_label = format!("{} [{i}]", step.label);
        match slot {
            None => {
                child_items.push(PipelineStepState {
                    step_id: child_id,
                    label: child_label,
                    kind: step.kind.clone(),
                    status: "skipped".to_string(),
                    started_ms: 0,
                    finished_ms: 0,
                    output: "Skipped after earlier item failure/pause".to_string(),
                    files: Vec::new(),
                    metrics: json!({}),
                    usage: None,
                    checkpoint_id: None,
                    human_answer: None,
                    human_action: None,
                    error: None,
                });
            }
            Some((s_ms, f_ms, StepCoreOutcome::Ok {
                output,
                files,
                metrics,
                usage,
                ..
            })) => {
                total_anomalies += metrics["anomalies"].as_u64().unwrap_or(0);
                for f in &files {
                    if !agg_files.contains(f) {
                        agg_files.push(f.clone());
                    }
                }
                agg_outputs.push(output.clone());
                agg_usage.accumulate(&usage);
                child_items.push(PipelineStepState {
                    step_id: child_id,
                    label: child_label,
                    kind: step.kind.clone(),
                    status: "ok".to_string(),
                    started_ms: s_ms,
                    finished_ms: f_ms,
                    output,
                    files,
                    metrics,
                    usage: Some(usage),
                    checkpoint_id: None,
                    human_answer: None,
                    human_action: None,
                    error: None,
                });
            }
            Some((s_ms, f_ms, StepCoreOutcome::Failed { error, usage, is_cancel, .. })) => {
                if let Some(ref u) = usage {
                    agg_usage.accumulate(u);
                }
                if first_error.is_none() {
                    first_error = Some((error.clone(), is_cancel));
                }
                child_items.push(PipelineStepState {
                    step_id: child_id,
                    label: child_label,
                    kind: step.kind.clone(),
                    status: "failed".to_string(),
                    started_ms: s_ms,
                    finished_ms: f_ms,
                    output: String::new(),
                    files: Vec::new(),
                    metrics: json!({}),
                    usage,
                    checkpoint_id: None,
                    human_answer: None,
                    human_action: None,
                    error: Some(error),
                });
            }
            Some((s_ms, f_ms, held @ StepCoreOutcome::HeldCheckpoint { .. })) => {
                child_items.push(PipelineStepState {
                    step_id: child_id,
                    label: child_label,
                    kind: step.kind.clone(),
                    status: "waiting_human".to_string(),
                    started_ms: s_ms,
                    finished_ms: f_ms,
                    output: "Held for human approval".to_string(),
                    files: Vec::new(),
                    metrics: json!({}),
                    usage: None,
                    checkpoint_id: None,
                    human_answer: None,
                    human_action: None,
                    error: None,
                });
                if first_held.is_none() {
                    first_held = Some(held);
                }
            }
        }
    }

    if let Some(held) = first_held {
        return held;
    }
    if let Some((error, is_cancel)) = first_error {
        return StepCoreOutcome::Failed {
            error,
            usage: if agg_usage.calls > 0 { Some(agg_usage) } else { None },
            is_cancel,
            child_items,
        };
    }

    let metrics = json!({
        "items_count": child_items.len(),
        "files_count": agg_files.len(),
        "anomalies": total_anomalies,
        "has_anomaly": total_anomalies > 0,
    });
    let usage_source = match step.kind.as_str() {
        "export" => "pipeline_export",
        "deliver" => "pipeline_deliver",
        "agent" => "pipeline_agent",
        _ => "pipeline_skill",
    };
    StepCoreOutcome::Ok {
        output: agg_outputs.join("\n"),
        files: agg_files,
        metrics,
        usage: agg_usage,
        usage_source,
        child_items,
        consume_snapshot: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn record_step_success(
    env: &PipelineExecEnv,
    run_id: &str,
    idx: usize,
    step: &PipelineStepSpec,
    elapsed_ms: u64,
    output: String,
    files: Vec<String>,
    metrics: Value,
    step_usage: records::RunUsage,
    usage_source: &'static str,
    child_items: Vec<PipelineStepState>,
    consume_snapshot: bool,
) -> Result<PipelineRun, String> {
    let finished_ms = records::now_millis();
    let updated_run = mutate_run(&env.cfg.data_dir, run_id, |r| {
        r.updated_ms = finished_ms;
        r.usage.accumulate(&step_usage);
        if consume_snapshot {
            r.set_agent_snapshot(None);
        }
        if let Some(st) = r.step_states.get_mut(idx) {
            st.status = "ok".to_string();
            st.finished_ms = finished_ms;
            st.output = output.clone();
            st.files = files.clone();
            st.metrics = metrics.clone();
            st.set_child_items(child_items.clone());
            st.usage = Some(step_usage.clone());
            st.error = None;
        }
        Ok(r.clone())
    })?;

    if usage_source != "pipeline_agent" {
        let store_clients = records::load_clients_store(&env.ctx.root_data_dir);
        let step_client_id = updated_run
            .engagement_id
            .as_deref()
            .and_then(|eid| store_clients.engagements.iter().find(|e| e.id == eid))
            .map(|e| e.client_id.clone());
        records::append_work_with_usage(
            &env.cfg,
            &format!("Pipeline {}: {}", updated_run.routine_name, step.label),
            &output,
            &files,
            &updated_run.thread_id,
            true,
            updated_run.engagement_id.as_deref(),
            Some(&step_usage),
        );
        records::append_usage_ledger(
            &env.cfg.data_dir,
            &records::UsageLedgerEntry {
                ts_ms: finished_ms,
                run_id: updated_run.run_id.clone(),
                thread_id: updated_run.thread_id.clone(),
                person_id: env.ctx.person_id.clone(),
                engagement_id: updated_run.engagement_id.clone(),
                client_id: step_client_id,
                source: usage_source.to_string(),
                finished: true,
                answered: false,
                usage: step_usage.clone(),
            },
        );
        env.store.lock().unwrap().push(
            &updated_run.thread_id,
            "step",
            &format!("Step {} ({}): {}", idx + 1, step.label, output),
        );
    }
    env.search_cache.invalidate(&env.ctx.person_id);
    let child_json: Vec<Value> = child_items.iter().map(|c| c.to_json()).collect();
    env.events.publish(
        "pipeline_step_done",
        &env.ctx.person_id,
        &updated_run.run_id,
        &updated_run.thread_id,
        json!({
            "step_index": idx,
            "step_id": step.id,
            "label": step.label,
            "kind": step.kind,
            "status": "ok",
            "output": output,
            "files": files,
            "metrics": metrics,
            "child_items": child_json,
            "usage": step_usage.to_json(),
            "run_usage": updated_run.usage.to_json(),
            "engagement_id": updated_run.engagement_id,
            "error": Value::Null,
            "elapsed_ms": elapsed_ms,
            "ts_ms": finished_ms,
        }),
    );
    Ok(updated_run)
}

#[allow(clippy::too_many_arguments)]
fn record_step_failure(
    env: &PipelineExecEnv,
    run_id: &str,
    idx: usize,
    step: &PipelineStepSpec,
    elapsed_ms: u64,
    err: String,
    step_usage: Option<records::RunUsage>,
    is_cancel: bool,
    child_items: Vec<PipelineStepState>,
    finalize_run: bool,
) -> Result<PipelineRun, String> {
    let finished_ms = records::now_millis();
    let next_status = if is_cancel { "cancelled" } else { "failed" };
    let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
        if finalize_run {
            r.status = next_status.to_string();
            r.error = Some(err.clone());
        }
        r.updated_ms = finished_ms;
        if let Some(ref u) = step_usage {
            r.usage.accumulate(u);
        }
        if let Some(st) = r.step_states.get_mut(idx) {
            st.status = "failed".to_string();
            st.finished_ms = finished_ms;
            st.set_child_items(child_items.clone());
            if let Some(ref u) = step_usage {
                st.usage = Some(u.clone());
            }
            st.error = Some(err.clone());
        }
        Ok(r.clone())
    })?;
    env.store.lock().unwrap().push(
        &failed.thread_id,
        "error",
        &format!("Step {} ({}) {}: {}", idx + 1, step.label, next_status, err),
    );
    env.events.publish(
        "pipeline_step_done",
        &env.ctx.person_id,
        &failed.run_id,
        &failed.thread_id,
        json!({
            "step_index": idx,
            "step_id": step.id,
            "label": step.label,
            "kind": step.kind,
            "status": "failed",
            "output": "",
            "files": [],
            "metrics": {},
            "usage": step_usage.as_ref().map(|u| u.to_json()),
            "run_usage": failed.usage.to_json(),
            "engagement_id": failed.engagement_id,
            "error": err,
            "elapsed_ms": elapsed_ms,
            "ts_ms": finished_ms,
        }),
    );
    if finalize_run {
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
    }
    Ok(failed)
}

fn emit_step_skipped(
    env: &PipelineExecEnv,
    run_id: &str,
    idx: usize,
    step: &PipelineStepSpec,
    skip_reason: &str,
) -> Result<PipelineRun, String> {
    let now_ms = records::now_millis();
    let updated = mutate_run(&env.cfg.data_dir, run_id, |r| {
        r.current_step_index = idx;
        r.updated_ms = now_ms;
        if let Some(st) = r.step_states.get_mut(idx) {
            st.status = "skipped".to_string();
            st.started_ms = now_ms;
            st.finished_ms = now_ms;
            st.output = skip_reason.to_string();
        }
        Ok(r.clone())
    })?;
    env.events.publish(
        "pipeline_step_done",
        &env.ctx.person_id,
        &updated.run_id,
        &updated.thread_id,
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
    Ok(updated)
}

fn finalize_cancelled_run(env: &PipelineExecEnv, run_id: &str) -> Result<PipelineRun, String> {
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
    Ok(cancelled)
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

    if initial.is_dag() {
        return execute_dag_pipeline_loop(env, run_id, &cancel_flag);
    }

    for idx in start_index..total_steps {
        if is_run_cancelled(&env.cfg.data_dir, run_id, &cancel_flag) {
            return finalize_cancelled_run(env, run_id);
        }

        let run_snap = get_run(&env.cfg.data_dir, run_id)
            .ok_or_else(|| "pipeline run not found".to_string())?;
        let step = run_snap.steps[idx].clone();

        let (should_run, skip_reason) = should_run_step_for_run(&step, idx, &run_snap);
        if !should_run {
            emit_step_skipped(env, run_id, idx, &step, skip_reason)?;
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

        let step_t0 = std::time::Instant::now();
        match execute_step_with_for_each(env, &step, &run_snap) {
            StepCoreOutcome::Ok {
                output,
                files,
                metrics,
                usage,
                usage_source,
                child_items,
                consume_snapshot,
            } => {
                let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                record_step_success(
                    env,
                    run_id,
                    idx,
                    &step,
                    elapsed_ms,
                    output,
                    files,
                    metrics,
                    usage,
                    usage_source,
                    child_items,
                    consume_snapshot,
                )?;
            }
            StepCoreOutcome::HeldCheckpoint {
                cp_spec,
                prior_files,
                prior_output,
                existing_decision_id,
                usage,
                updated_snapshot,
            } => {
                if let Some(u) = usage {
                    let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                        r.usage.accumulate(&u);
                        if let Some(st) = r.step_states.get_mut(idx) {
                            st.usage = Some(u);
                        }
                        Ok(())
                    });
                }
                return park_pipeline_at_checkpoint_with_options(
                    env,
                    run_id,
                    idx,
                    cp_spec,
                    prior_files,
                    prior_output,
                    existing_decision_id,
                    updated_snapshot,
                );
            }
            StepCoreOutcome::Failed {
                error,
                usage,
                is_cancel,
                child_items,
            } => {
                let elapsed_ms = step_t0.elapsed().as_millis() as u64;
                return record_step_failure(
                    env,
                    run_id,
                    idx,
                    &step,
                    elapsed_ms,
                    error,
                    usage,
                    is_cancel,
                    child_items,
                    true,
                );
            }
        }
    }

    finalize_completed_or_cancelled(env, run_id, total_steps, &cancel_flag)
}

struct DagStepDoneMsg {
    idx: usize,
    elapsed_ms: u64,
    outcome: StepCoreOutcome,
}

struct PendingDagPause {
    idx: usize,
    cp_spec: PipelineCheckpointSpec,
    prior_files: Vec<String>,
    prior_output: Option<String>,
    existing_decision_id: Option<String>,
    usage: Option<records::RunUsage>,
    updated_snapshot: Option<AgentSnapshot>,
}

fn execute_dag_pipeline_loop(
    env: &PipelineExecEnv,
    run_id: &str,
    cancel_flag: &AtomicBool,
) -> Result<PipelineRun, String> {
    let (tx, rx) = mpsc::channel::<DagStepDoneMsg>();
    let mut in_flight: HashSet<usize> = HashSet::new();
    let mut draining_failure: Option<(String, bool)> = None;
    let mut pending_pause: Option<PendingDagPause> = None;

    loop {
        if is_run_cancelled(&env.cfg.data_dir, run_id, cancel_flag) && draining_failure.is_none() {
            draining_failure = Some(("run cancelled by user".to_string(), true));
        }

        let mut run_snap = get_run(&env.cfg.data_dir, run_id)
            .ok_or_else(|| "pipeline run not found".to_string())?;
        let total_steps = run_snap.steps.len();

        if draining_failure.is_none() && pending_pause.is_none() {
            // Find ready pending steps whose depends_on are all "ok" or "skipped".
            loop {
                let mut skipped_any = false;
                let state_map: HashMap<&str, &str> = run_snap
                    .step_states
                    .iter()
                    .map(|s| (s.step_id.as_str(), s.status.as_str()))
                    .collect();
                for idx in 0..total_steps {
                    if in_flight.contains(&idx) || run_snap.step_states[idx].status != "pending" {
                        continue;
                    }
                    let deps = run_snap.steps[idx].depends_on();
                    let ready = deps.iter().all(|d| {
                        matches!(state_map.get(d.as_str()).copied(), Some("ok" | "skipped"))
                    });
                    if !ready {
                        continue;
                    }
                    let step = run_snap.steps[idx].clone();
                    let (should_run, skip_reason) = should_run_step_for_run(&step, idx, &run_snap);
                    if !should_run {
                        run_snap = emit_step_skipped(env, run_id, idx, &step, skip_reason)?;
                        skipped_any = true;
                        break;
                    }
                }
                if !skipped_any {
                    break;
                }
            }

            let state_map: HashMap<String, String> = run_snap
                .step_states
                .iter()
                .map(|s| (s.step_id.clone(), s.status.clone()))
                .collect();

            for idx in 0..total_steps {
                if in_flight.len() >= MAX_DAG_CONCURRENCY {
                    break;
                }
                if in_flight.contains(&idx) || run_snap.step_states[idx].status != "pending" {
                    continue;
                }
                let deps = run_snap.steps[idx].depends_on();
                let ready = deps.iter().all(|d| {
                    matches!(state_map.get(d.as_str()).map(String::as_str), Some("ok" | "skipped"))
                });
                if !ready {
                    continue;
                }
                let step = run_snap.steps[idx].clone();
                if step.kind == "checkpoint" {
                    if !in_flight.is_empty() {
                        // Drain current wave before parking checkpoint
                        break;
                    }
                    let started_ms = records::now_millis();
                    let snap_now = mutate_run(&env.cfg.data_dir, run_id, |r| {
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
                    let cp_spec = build_checkpoint_from_step(&step, &snap_now);
                    return park_pipeline_at_checkpoint_with_options(
                        env,
                        run_id,
                        idx,
                        cp_spec,
                        Vec::new(),
                        None,
                        None,
                        None,
                    );
                }

                let started_ms = records::now_millis();
                run_snap = mutate_run(&env.cfg.data_dir, run_id, |r| {
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

                in_flight.insert(idx);
                let env_w = env.clone();
                let snap_w = run_snap.clone();
                let tx_w = tx.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("ergon-dag-{}-{}", run_id, step.id))
                    .stack_size(4 * 1024 * 1024)
                    .spawn(move || {
                        let t0 = std::time::Instant::now();
                        let outcome = execute_step_with_for_each(&env_w, &step, &snap_w);
                        let elapsed_ms = t0.elapsed().as_millis() as u64;
                        let _ = tx_w.send(DagStepDoneMsg {
                            idx,
                            elapsed_ms,
                            outcome,
                        });
                    });
            }
        }

        if in_flight.is_empty() {
            if let Some(pause) = pending_pause.take() {
                if let Some(u) = pause.usage {
                    let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                        r.usage.accumulate(&u);
                        if let Some(st) = r.step_states.get_mut(pause.idx) {
                            st.usage = Some(u);
                        }
                        Ok(())
                    });
                }
                return park_pipeline_at_checkpoint_with_options(
                    env,
                    run_id,
                    pause.idx,
                    pause.cp_spec,
                    pause.prior_files,
                    pause.prior_output,
                    pause.existing_decision_id,
                    pause.updated_snapshot,
                );
            }
            if let Some((err, is_cancel)) = draining_failure {
                if is_cancel {
                    return finalize_cancelled_run(env, run_id);
                }
                let finished_ms = records::now_millis();
                let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                    r.status = "failed".to_string();
                    r.error = Some(err.clone());
                    r.updated_ms = finished_ms;
                    Ok(r.clone())
                })?;
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

            let latest = get_run(&env.cfg.data_dir, run_id)
                .ok_or_else(|| "pipeline run not found".to_string())?;
            if latest
                .step_states
                .iter()
                .all(|s| matches!(s.status.as_str(), "ok" | "skipped"))
            {
                return finalize_completed_or_cancelled(env, run_id, total_steps, cancel_flag);
            }
            return Err("DAG scheduler stalled: unresolvable pending dependencies".to_string());
        }

        let msg = rx
            .recv()
            .map_err(|_| "DAG worker channel closed unexpectedly".to_string())?;
        in_flight.remove(&msg.idx);
        let step = run_snap.steps[msg.idx].clone();
        match msg.outcome {
            StepCoreOutcome::Ok {
                output,
                files,
                metrics,
                usage,
                usage_source,
                child_items,
                consume_snapshot,
            } => {
                record_step_success(
                    env,
                    run_id,
                    msg.idx,
                    &step,
                    msg.elapsed_ms,
                    output,
                    files,
                    metrics,
                    usage,
                    usage_source,
                    child_items,
                    consume_snapshot,
                )?;
            }
            StepCoreOutcome::HeldCheckpoint {
                cp_spec,
                prior_files,
                prior_output,
                existing_decision_id,
                usage,
                updated_snapshot,
            } => {
                if pending_pause.is_none() {
                    pending_pause = Some(PendingDagPause {
                        idx: msg.idx,
                        cp_spec,
                        prior_files,
                        prior_output,
                        existing_decision_id,
                        usage,
                        updated_snapshot,
                    });
                }
            }
            StepCoreOutcome::Failed {
                error,
                usage,
                is_cancel,
                child_items,
            } => {
                if draining_failure.is_none() {
                    draining_failure = Some((error.clone(), is_cancel));
                }
                record_step_failure(
                    env,
                    run_id,
                    msg.idx,
                    &step,
                    msg.elapsed_ms,
                    error,
                    usage,
                    is_cancel,
                    child_items,
                    false,
                )?;
            }
        }
    }
}

fn finalize_completed_or_cancelled(
    env: &PipelineExecEnv,
    run_id: &str,
    total_steps: usize,
    cancel_flag: &AtomicBool,
) -> Result<PipelineRun, String> {
    let finished_ms = records::now_millis();
    let final_cancelled = is_run_cancelled(&env.cfg.data_dir, run_id, cancel_flag);
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
    let tmp_check_run = PipelineRun {
        run_id: String::new(),
        routine_id: None,
        routine_name: String::new(),
        thread_id: String::new(),
        status: "pending".to_string(),
        current_step_index: 0,
        created_ms: 0,
        updated_ms: 0,
        inputs: inputs.clone(),
        steps: steps.clone(),
        step_states: Vec::new(),
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: records::RunUsage::default(),
    };
    for step in &steps {
        if let Some(fe) = step.for_each() {
            if !fe.contains("{{steps.") && !fe.contains("{{item") {
                resolve_for_each_items(&fe, &tmp_check_run).map_err(|e| (400, e))?;
            }
        }
    }

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
    let mut run = PipelineRun {
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
    if let Some(snap) = body
        .get("agent_snapshot")
        .filter(|v| v.is_object())
        .and_then(AgentSnapshot::from_json)
    {
        run.set_agent_snapshot(Some(snap));
    }
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
            cp_step_index: usize,
            next_index: usize,
            has_more_steps: bool,
            resolved_answer: String,
        },
    }

    let transition = mutate_run_http(&env.cfg.data_dir, run_id, |r| {
        if r.status == "interrupted" && r.pending_checkpoint.is_none() {
            let start_idx = r.current_step_index.min(r.steps.len());
            let is_dag = r.is_dag();
            let completed_now = if is_dag {
                !r.step_states.iter().any(|s| matches!(s.status.as_str(), "pending" | "failed"))
            } else {
                start_idx >= r.steps.len()
            };
            r.status = if completed_now {
                "completed".to_string()
            } else {
                "running".to_string()
            };
            r.current_step_index = start_idx;
            r.error = None;
            r.cancel_requested = false;
            r.updated_ms = now_ms;
            if is_dag {
                for (i, st) in r.step_states.iter_mut().enumerate() {
                    if st.status == "failed" {
                        *st = PipelineStepState::pending_for(&r.steps[i]);
                    }
                }
            } else if let Some(st) = r.step_states.get_mut(start_idx) {
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

        let is_dag = r.is_dag();
        let next_index = cp.step_index + 1;
        if let Some(st) = r.step_states.get_mut(cp.step_index) {
            st.status = "ok".to_string();
            st.finished_ms = now_ms;
            st.human_action = Some(action.clone());
            st.human_answer = Some(resolved_answer.clone());
            st.output = resolved_answer.clone();
            st.error = None;
        }
        let has_more_steps = if is_dag {
            r.step_states.iter().any(|s| s.status == "pending")
        } else {
            next_index < r.steps.len()
        };
        r.status = if has_more_steps {
            "running".to_string()
        } else {
            "completed".to_string()
        };
        r.current_step_index = next_index;
        r.updated_ms = now_ms;
        r.pending_checkpoint = None;
        r.error = None;
        r.cancel_requested = false;
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
            cp_step_index: cp.step_index,
            next_index,
            has_more_steps,
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
                if let Ok((dec, _)) = records::resolve_decision(&env.cfg, &checkpoint_id, false) {
                    if dec["kind"].as_str() != Some("connector_delivery") {
                        let outcome_text = format!("Aborted at checkpoint: {resolved_answer}");
                        let _ = records::set_decision_outcome(
                            &env.cfg.data_dir,
                            &checkpoint_id,
                            &outcome_text,
                        );
                    }
                }
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
            mut run,
            checkpoint_id,
            cp_step_index,
            next_index,
            has_more_steps,
            resolved_answer,
        } => {
            let mut delivery_error: Option<String> = None;
            let mut delivery_outcome_msg: Option<String> = None;
            if !checkpoint_id.is_empty() {
                match records::resolve_decision(&env.cfg, &checkpoint_id, true) {
                    Ok((dec, final_outcome)) => {
                        if dec["kind"].as_str() == Some("connector_delivery") {
                            delivery_outcome_msg = Some(final_outcome.clone());
                            if final_outcome.starts_with("Approved, but") {
                                delivery_error = Some(final_outcome);
                            }
                        } else {
                            let outcome_text = format!("Resumed ({action}): {resolved_answer}");
                            let _ = records::set_decision_outcome(
                                &env.cfg.data_dir,
                                &checkpoint_id,
                                &outcome_text,
                            );
                        }
                    }
                    Err(e) if e.contains("already settled") => {
                        if let Some(dec) = records::load_decisions(&env.cfg.data_dir)
                            .into_iter()
                            .find(|d| d["id"].as_str() == Some(checkpoint_id.as_str()))
                        {
                            if dec["kind"].as_str() == Some("connector_delivery") {
                                let status_str = dec["status"].as_str().unwrap_or("");
                                let mut out_str = dec["outcome"]
                                    .as_str()
                                    .unwrap_or("Approved.")
                                    .to_string();
                                if status_str == "approved"
                                    && out_str.starts_with("Approved — executing")
                                {
                                    if let Some(waited) = connectors::wait_for_delivery_settlement(
                                        &env.cfg.data_dir,
                                        &checkpoint_id,
                                    ) {
                                        out_str = waited;
                                    }
                                }
                                delivery_outcome_msg = Some(out_str.clone());
                                if status_str == "denied"
                                    || out_str.starts_with("Approved, but")
                                    || out_str.starts_with("You said no")
                                {
                                    delivery_error = Some(out_str);
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }

            if let Some(ref out_msg) = delivery_outcome_msg {
                if let Some(err) = delivery_error {
                    let failed = mutate_run(&env.cfg.data_dir, run_id, |r| {
                        r.status = "failed".to_string();
                        r.error = Some(err.clone());
                        r.updated_ms = records::now_millis();
                        if let Some(st) = r.step_states.get_mut(cp_step_index) {
                            st.status = "failed".to_string();
                            st.output = err.clone();
                            st.error = Some(err.clone());
                        }
                        Ok(r.clone())
                    })
                    .unwrap_or(run);
                    env.events.publish(
                        "pipeline_finished",
                        &env.ctx.person_id,
                        &failed.run_id,
                        &failed.thread_id,
                        json!({
                            "status": "failed",
                            "error": failed.error,
                            "run": failed.to_json(),
                            "ts_ms": records::now_millis(),
                        }),
                    );
                    return Ok(failed);
                }
                if let Ok(updated) = mutate_run(&env.cfg.data_dir, run_id, |r| {
                    if let Some(st) = r.step_states.get_mut(cp_step_index) {
                        st.output = out_msg.clone();
                    }
                    Ok(r.clone())
                }) {
                    run = updated;
                }
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

            if !has_more_steps {
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
        let is_dag = r.is_dag();
        r.status = "running".to_string();
        r.current_step_index = idx;
        r.updated_ms = now_ms;
        r.pending_checkpoint = None;
        r.error = None;
        r.cancel_requested = false;
        if is_dag {
            let mut reset_ids: HashSet<String> = HashSet::new();
            reset_ids.insert(r.steps[idx].id.clone());
            loop {
                let mut added = false;
                for s in &r.steps {
                    if !reset_ids.contains(&s.id)
                        && s.depends_on().iter().any(|d| reset_ids.contains(d))
                    {
                        reset_ids.insert(s.id.clone());
                        added = true;
                    }
                }
                if !added {
                    break;
                }
            }
            for i in 0..r.step_states.len() {
                if reset_ids.contains(&r.steps[i].id) || r.step_states[i].status == "failed" {
                    r.step_states[i] = PipelineStepState::pending_for(&r.steps[i]);
                }
            }
        } else {
            for i in idx..r.step_states.len() {
                r.step_states[i] = PipelineStepState::pending_for(&r.steps[i]);
            }
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
    park_adhoc_checkpoint_with_snapshot(
        cfg,
        thread_id,
        task,
        written_files,
        steps_taken,
        cp,
        live,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn park_adhoc_checkpoint_with_snapshot(
    cfg: &Config,
    thread_id: &str,
    task: &str,
    written_files: &[String],
    steps_taken: &[String],
    cp: &CheckpointPause,
    live: Option<(&RunEventBus, &str)>,
    agent_snapshot: Option<AgentSnapshot>,
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

    let mut run = PipelineRun {
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
    if let Some(snap) = agent_snapshot {
        run.set_agent_snapshot(Some(snap));
    }
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
