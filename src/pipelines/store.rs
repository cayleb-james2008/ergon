use super::*;

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
    pub fn step_snapshots(&self) -> HashMap<String, AgentSnapshot> {
        let mut out: HashMap<String, AgentSnapshot> = self
            .inputs
            .get(SNAPSHOTS_INPUT_KEY)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| {
                v.as_object().map(|obj| {
                    obj.iter()
                        .filter_map(|(k, val)| AgentSnapshot::from_json(val).map(|s| (k.clone(), s)))
                        .collect()
                })
            })
            .unwrap_or_default();
        if out.is_empty() {
            if let Some(legacy) = self
                .inputs
                .get(SNAPSHOT_INPUT_KEY)
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .and_then(|v| AgentSnapshot::from_json(&v))
            {
                let key = self
                    .steps
                    .get(self.current_step_index)
                    .map(|s| s.id.clone())
                    .unwrap_or_else(|| "default".to_string());
                out.insert(key, legacy);
            }
        }
        out
    }

    pub fn step_snapshot(&self, step_id: &str) -> Option<AgentSnapshot> {
        self.step_snapshots().get(step_id).cloned()
    }

    pub fn set_step_snapshot(&mut self, step_id: &str, snap: Option<AgentSnapshot>) {
        let mut map = self
            .inputs
            .get(SNAPSHOTS_INPUT_KEY)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        match snap {
            Some(s) => {
                map.insert(step_id.to_string(), s.to_json());
                self.inputs.insert(
                    SNAPSHOTS_INPUT_KEY.to_string(),
                    Value::Object(map).to_string(),
                );
                self.inputs
                    .insert(SNAPSHOT_INPUT_KEY.to_string(), s.to_json().to_string());
            }
            None => {
                map.remove(step_id);
                if map.is_empty() {
                    self.inputs.remove(SNAPSHOTS_INPUT_KEY);
                    self.inputs.remove(SNAPSHOT_INPUT_KEY);
                } else {
                    self.inputs.insert(
                        SNAPSHOTS_INPUT_KEY.to_string(),
                        Value::Object(map.clone()).to_string(),
                    );
                    if let Some(first_val) = map.values().next() {
                        self.inputs
                            .insert(SNAPSHOT_INPUT_KEY.to_string(), first_val.to_string());
                    }
                }
            }
        }
    }

    pub fn agent_snapshot(&self) -> Option<AgentSnapshot> {
        self.inputs
            .get(SNAPSHOT_INPUT_KEY)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| AgentSnapshot::from_json(&v))
            .or_else(|| self.step_snapshots().into_values().next())
    }

    pub fn set_agent_snapshot(&mut self, snap: Option<AgentSnapshot>) {
        match snap {
            Some(s) => {
                self.inputs
                    .insert(SNAPSHOT_INPUT_KEY.to_string(), s.to_json().to_string());
                let key = self
                    .steps
                    .get(self.current_step_index)
                    .map(|st| st.id.clone())
                    .unwrap_or_else(|| "default".to_string());
                let mut map = self
                    .inputs
                    .get(SNAPSHOTS_INPUT_KEY)
                    .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                map.insert(key, s.to_json());
                self.inputs.insert(
                    SNAPSHOTS_INPUT_KEY.to_string(),
                    Value::Object(map).to_string(),
                );
            }
            None => {
                self.inputs.remove(SNAPSHOT_INPUT_KEY);
                self.inputs.remove(SNAPSHOTS_INPUT_KEY);
            }
        }
    }

    pub fn pending_checkpoints(&self) -> Vec<PendingCheckpoint> {
        let mut cps: Vec<PendingCheckpoint> = self
            .inputs
            .get(CHECKPOINTS_INPUT_KEY)
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(PendingCheckpoint::from_json)
                        .collect()
                })
            })
            .unwrap_or_default();
        if cps.is_empty() {
            if let Some(cp) = &self.pending_checkpoint {
                cps.push(cp.clone());
            }
        }
        cps
    }

    pub fn set_pending_checkpoints(&mut self, cps: Vec<PendingCheckpoint>) {
        self.pending_checkpoint = cps.first().cloned();
        if cps.is_empty() {
            self.inputs.remove(CHECKPOINTS_INPUT_KEY);
        } else {
            let arr: Vec<Value> = cps.iter().map(|c| c.to_json()).collect();
            self.inputs
                .insert(CHECKPOINTS_INPUT_KEY.to_string(), Value::Array(arr).to_string());
        }
    }

    pub fn add_pending_checkpoint(&mut self, cp: PendingCheckpoint) {
        let mut cps = self.pending_checkpoints();
        if let Some(pos) = cps
            .iter()
            .position(|existing| existing.checkpoint_id == cp.checkpoint_id || existing.step_id == cp.step_id)
        {
            cps[pos] = cp;
        } else {
            cps.push(cp);
        }
        self.set_pending_checkpoints(cps);
    }

    pub fn remove_pending_checkpoint(&mut self, checkpoint_id: &str) -> Option<PendingCheckpoint> {
        let mut cps = self.pending_checkpoints();
        let pos = cps.iter().position(|c| c.checkpoint_id == checkpoint_id)?;
        let removed = cps.remove(pos);
        self.set_pending_checkpoints(cps);
        Some(removed)
    }

    pub fn take_pending_checkpoints(&mut self) -> Vec<PendingCheckpoint> {
        let cps = self.pending_checkpoints();
        self.set_pending_checkpoints(Vec::new());
        cps
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
                    SNAPSHOT_INPUT_KEY
                        | SNAPSHOTS_INPUT_KEY
                        | CHECKPOINTS_INPUT_KEY
                        | SUBPIPELINE_DEPTH_KEY
                        | SUBPIPELINE_STACK_KEY
                        | ROUTINE_TRIGGER_KEY
                        | ITEM_INPUT_KEY
                        | ITEM_INDEX_INPUT_KEY
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
        let step_snaps = self.step_snapshots();
        let mut step_snaps_json = Map::new();
        for (sid, snap) in &step_snaps {
            step_snaps_json.insert(sid.clone(), snap.to_json());
        }
        let snapshot_val = self
            .agent_snapshot()
            .map(|s| s.to_json())
            .unwrap_or(Value::Null);
        let cps = self.pending_checkpoints();
        let primary_cp = self
            .pending_checkpoint
            .as_ref()
            .or_else(|| cps.first())
            .map(|c| c.to_json());
        let mut out = json!({
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
            "pending_checkpoints": cps.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "pending_checkpoint": primary_cp,
            "step_snapshots": Value::Object(step_snaps_json),
            "agent_snapshot": snapshot_val,
            "error": self.error,
            "engagement_id": self.engagement_id,
            "usage": self.usage.to_json(),
        });
        if let Some(trig) = self.inputs.get(ROUTINE_TRIGGER_KEY) {
            out["routine_trigger"] = json!(trig);
        }
        if let Some(depth) = self.inputs.get(SUBPIPELINE_DEPTH_KEY) {
            out["subpipeline_depth"] = json!(depth);
        }
        if let Some(stack) = self.inputs.get(SUBPIPELINE_STACK_KEY) {
            out["subpipeline_stack"] = json!(stack);
        }
        out
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let run_id = v["run_id"].as_str()?.to_string();
        let mut inputs = HashMap::new();
        if let Some(obj) = v["inputs"].as_object() {
            for (k, val) in obj {
                if matches!(
                    k.as_str(),
                    SNAPSHOT_INPUT_KEY
                        | SNAPSHOTS_INPUT_KEY
                        | CHECKPOINTS_INPUT_KEY
                        | SUBPIPELINE_DEPTH_KEY
                        | SUBPIPELINE_STACK_KEY
                        | ROUTINE_TRIGGER_KEY
                        | ITEM_INPUT_KEY
                        | ITEM_INDEX_INPUT_KEY
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
        if let Some(snaps_obj) = v.get("step_snapshots").and_then(|s| s.as_object()) {
            let mut valid_snaps = Map::new();
            for (k, val) in snaps_obj {
                if let Some(snap) = AgentSnapshot::from_json(val) {
                    valid_snaps.insert(k.clone(), snap.to_json());
                }
            }
            if !valid_snaps.is_empty() {
                inputs.insert(
                    SNAPSHOTS_INPUT_KEY.to_string(),
                    Value::Object(valid_snaps).to_string(),
                );
            }
        }
        if let Some(snap_val) = v.get("agent_snapshot").filter(|s| s.is_object()) {
            if let Some(snap) = AgentSnapshot::from_json(snap_val) {
                inputs.insert(SNAPSHOT_INPUT_KEY.to_string(), snap.to_json().to_string());
            }
        }
        if let Some(trig) = v
            .get("routine_trigger")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        {
            inputs.insert(ROUTINE_TRIGGER_KEY.to_string(), trig.to_string());
        }
        if let Some(depth) = v
            .get("subpipeline_depth")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        {
            inputs.insert(SUBPIPELINE_DEPTH_KEY.to_string(), depth.to_string());
        }
        if let Some(stack) = v
            .get("subpipeline_stack")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        {
            inputs.insert(SUBPIPELINE_STACK_KEY.to_string(), stack.to_string());
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
        let mut cps: Vec<PendingCheckpoint> = v
            .get("pending_checkpoints")
            .and_then(|p| p.as_array())
            .map(|arr| arr.iter().filter_map(PendingCheckpoint::from_json).collect())
            .unwrap_or_default();
        let legacy_cp = v
            .get("pending_checkpoint")
            .and_then(|p| if p.is_null() { None } else { PendingCheckpoint::from_json(p) });
        if cps.is_empty() {
            if let Some(ref cp) = legacy_cp {
                cps.push(cp.clone());
            }
        }
        if !cps.is_empty() {
            let arr: Vec<Value> = cps.iter().map(|c| c.to_json()).collect();
            inputs.insert(CHECKPOINTS_INPUT_KEY.to_string(), Value::Array(arr).to_string());
        }
        let pending_checkpoint = cps.first().cloned().or(legacy_cp);

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
                (r.status == "paused_checkpoint" || r.status == "running")
                    && (r
                        .pending_checkpoints()
                        .iter()
                        .any(|cp| cp.checkpoint_id == checkpoint_id)
                        || r.pending_checkpoint
                            .as_ref()
                            .is_some_and(|cp| cp.checkpoint_id == checkpoint_id))
            })
    })
}

pub(super) fn load_runs_unlocked(person_data_dir: &Path) -> Vec<PipelineRun> {
    let path = pipeline_runs_path(person_data_dir);
    let Ok(bytes) = std::fs::read(&path) else {
        return Vec::new();
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        let _ = crate::backup_corrupt(&path);
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        let _ = crate::backup_corrupt(&path);
        return Vec::new();
    };
    let Some(arr) = v.as_object().and_then(|m| m.get("runs")).and_then(|x| x.as_array()) else {
        let _ = crate::backup_corrupt(&path);
        return Vec::new();
    };
    let mut out = Vec::with_capacity(arr.len());
    let mut had_corrupt = false;
    for item in arr {
        match PipelineRun::from_json(item) {
            Some(r) => out.push(r),
            None => had_corrupt = true,
        }
    }
    if had_corrupt {
        if out.is_empty() {
            let _ = crate::backup_corrupt(&path);
        } else {
            let _ = crate::backup_corrupt_copy(&path);
            let _ = save_runs_unlocked(person_data_dir, &out);
        }
    }
    out
}

pub(super) fn save_runs_unlocked(person_data_dir: &Path, runs: &[PipelineRun]) -> Result<(), String> {
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
    atomic_write_bytes_with_mode(&path, body.as_bytes(), Some(0o600))
        .map_err(|e| format!("write pipeline_runs: {e}"))
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

pub(super) fn mutate_run<F, R>(person_data_dir: &Path, run_id: &str, f: F) -> Result<R, String>
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

pub(super) fn mutate_run_http<F, R>(person_data_dir: &Path, run_id: &str, f: F) -> Result<R, (u16, String)>
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
            let cps = run.pending_checkpoints();
            if run.status == "paused_checkpoint" && cps.is_empty() {
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
            if run.status == "paused_checkpoint" && !cps.is_empty() {
                let cp_indices: HashSet<usize> = cps.iter().map(|c| c.step_index).collect();
                let mut reset_any = false;
                for i in 0..run.step_states.len() {
                    if run.step_states[i].status == "running" && !cp_indices.contains(&i) {
                        if let Some(spec) = run.steps.get(i) {
                            run.step_states[i] = PipelineStepState::pending_for(spec);
                            reset_any = true;
                        }
                    }
                }
                if reset_any {
                    run.updated_ms = now_ms;
                    changed += 1;
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
                run.set_pending_checkpoints(Vec::new());
                run.error = Some("Cancelled before server restart.".to_string());
                for st in &mut run.step_states {
                    if st.status == "running" || st.status == "waiting_human" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Cancelled before server restart.".to_string());
                    }
                }
            } else if !cps.is_empty() {
                run.status = "paused_checkpoint".to_string();
                let cp_indices: HashSet<usize> = cps.iter().map(|c| c.step_index).collect();
                for cp in cps {
                    if let Some(st) = run.step_states.get_mut(cp.step_index) {
                        if st.status != "waiting_human" {
                            st.status = "waiting_human".to_string();
                            st.checkpoint_id = Some(cp.checkpoint_id);
                        }
                    }
                }
                for i in 0..run.step_states.len() {
                    if run.step_states[i].status == "running" && !cp_indices.contains(&i) {
                        if let Some(spec) = run.steps.get(i) {
                            run.step_states[i] = PipelineStepState::pending_for(spec);
                        }
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

