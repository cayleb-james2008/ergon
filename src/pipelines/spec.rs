use super::*;

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
    Subpipeline {
        routine_id: String,
        inputs: HashMap<String, String>,
    },
}

impl PipelineStepKind {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "skill" => Some(Self::Skill),
            "agent" => Some(Self::Agent),
            "checkpoint" => Some(Self::Checkpoint),
            "export" => Some(Self::Export),
            "deliver" => Some(Self::Deliver),
            "subpipeline" => Some(Self::Subpipeline {
                routine_id: String::new(),
                inputs: HashMap::new(),
            }),
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
            Self::Subpipeline { .. } => "subpipeline",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineSubpipelineSpec {
    pub routine_id: String,
    pub inputs: HashMap<String, String>,
}

impl PipelineSubpipelineSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "routine_id": self.routine_id,
            "inputs": self.inputs,
        })
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let Some(obj) = v.as_object() else {
            return Err("subpipeline must be a JSON object".to_string());
        };
        let routine_id = obj
            .get("routine_id")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .chars()
            .take(128)
            .collect::<String>();
        if routine_id.is_empty() {
            return Err("subpipeline.routine_id is required".to_string());
        }
        let inputs = parse_inputs(obj.get("inputs").unwrap_or(&Value::Null), true);
        Ok(Self { routine_id, inputs })
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

    pub fn timeout_secs(&self) -> Option<u64> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("timeout_secs"))
            .and_then(|v| v.as_u64())
    }

    pub fn retry_max(&self) -> Option<u32> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("retry_max"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
    }

    pub fn retry_backoff_ms(&self) -> Option<u64> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("retry_backoff_ms"))
            .and_then(|v| v.as_u64())
    }

    pub fn on_error(&self) -> Option<String> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("on_error"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.trim().is_empty())
    }

    pub fn subpipeline(&self) -> Option<PipelineSubpipelineSpec> {
        self.args
            .get(DAG_META_KEY)
            .and_then(|m| m.get("subpipeline"))
            .and_then(|s| if s.is_null() { None } else { PipelineSubpipelineSpec::from_json(s).ok() })
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
        self.set_step_meta(
            depends_on,
            for_each,
            deliver,
            self.timeout_secs(),
            self.retry_max(),
            self.retry_backoff_ms(),
            self.on_error(),
            self.subpipeline(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_step_meta(
        &mut self,
        depends_on: Vec<String>,
        for_each: Option<String>,
        deliver: Option<PipelineDeliverSpec>,
        timeout_secs: Option<u64>,
        retry_max: Option<u32>,
        retry_backoff_ms: Option<u64>,
        on_error: Option<String>,
        subpipeline: Option<PipelineSubpipelineSpec>,
    ) {
        if !self.args.is_object() {
            self.args = json!({});
        }
        let obj = self.args.as_object_mut().unwrap();
        if depends_on.is_empty()
            && for_each.is_none()
            && deliver.is_none()
            && timeout_secs.is_none()
            && retry_max.is_none()
            && retry_backoff_ms.is_none()
            && on_error.is_none()
            && subpipeline.is_none()
        {
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
        if let Some(ts) = timeout_secs {
            meta.insert("timeout_secs".to_string(), json!(ts));
        }
        if let Some(rm) = retry_max {
            meta.insert("retry_max".to_string(), json!(rm));
        }
        if let Some(rb) = retry_backoff_ms {
            meta.insert("retry_backoff_ms".to_string(), json!(rb));
        }
        if let Some(oe) = on_error {
            meta.insert("on_error".to_string(), json!(oe));
        }
        if let Some(sub) = subpipeline {
            meta.insert("subpipeline".to_string(), sub.to_json());
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
        if let Some(ts) = self.timeout_secs() {
            obj.insert("timeout_secs".to_string(), json!(ts));
        }
        if let Some(rm) = self.retry_max() {
            obj.insert("retry_max".to_string(), json!(rm));
        }
        if let Some(rb) = self.retry_backoff_ms() {
            obj.insert("retry_backoff_ms".to_string(), json!(rb));
        }
        if let Some(oe) = self.on_error() {
            obj.insert("on_error".to_string(), json!(oe));
        }
        if let Some(sub) = self.subpipeline() {
            obj.insert("subpipeline".to_string(), sub.to_json());
            obj.insert("routine_id".to_string(), json!(sub.routine_id));
            obj.insert("inputs".to_string(), json!(sub.inputs));
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
                "steps_v2[{index}].kind '{raw_kind}' must be skill, agent, checkpoint, export, deliver, or subpipeline"
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
        let subpipeline = if raw_kind == "subpipeline" {
            if let Some(sub_val) = obj.get("subpipeline").filter(|v| !v.is_null()) {
                Some(PipelineSubpipelineSpec::from_json(sub_val)?)
            } else {
                let mut tmp = Map::new();
                if let Some(rid) = obj.get("routine_id") {
                    tmp.insert("routine_id".to_string(), rid.clone());
                }
                if let Some(inp) = obj.get("inputs") {
                    tmp.insert("inputs".to_string(), inp.clone());
                }
                Some(PipelineSubpipelineSpec::from_json(&Value::Object(tmp))?)
            }
        } else {
            None
        };

        let timeout_secs: Option<u64> = match obj.get("timeout_secs") {
            None | Some(Value::Null) => None,
            Some(v) => {
                let Some(n) = v.as_u64() else {
                    return Err(format!("steps_v2[{index}].timeout_secs must be a positive integer"));
                };
                if n == 0 || n > 86_400 {
                    return Err(format!("steps_v2[{index}].timeout_secs must be between 1 and 86400"));
                }
                Some(n)
            }
        };
        let retry_max: Option<u32> = match obj.get("retry_max") {
            None | Some(Value::Null) => None,
            Some(v) => {
                let Some(n) = v.as_u64() else {
                    return Err(format!("steps_v2[{index}].retry_max must be a non-negative integer"));
                };
                if n > 20 {
                    return Err(format!("steps_v2[{index}].retry_max cannot exceed 20"));
                }
                Some(n as u32)
            }
        };
        let retry_backoff_ms: Option<u64> = match obj.get("retry_backoff_ms") {
            None | Some(Value::Null) => None,
            Some(v) => {
                let Some(n) = v.as_u64() else {
                    return Err(format!("steps_v2[{index}].retry_backoff_ms must be a non-negative integer"));
                };
                if n > 300_000 {
                    return Err(format!("steps_v2[{index}].retry_backoff_ms cannot exceed 300000"));
                }
                Some(n)
            }
        };
        let on_error: Option<String> = match obj.get("on_error") {
            None | Some(Value::Null) => None,
            Some(v) => {
                let Some(raw) = v.as_str() else {
                    return Err(format!("steps_v2[{index}].on_error must be a string"));
                };
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    let low = trimmed.to_ascii_lowercase();
                    if low == "fail" || low == "continue" {
                        Some(low)
                    } else if let Some(target) = trimmed.strip_prefix("goto:") {
                        let t = target.trim();
                        if t.is_empty() {
                            return Err(format!("steps_v2[{index}].on_error 'goto:' requires target step_id"));
                        }
                        Some(format!("goto:{t}"))
                    } else {
                        return Err(format!(
                            "steps_v2[{index}].on_error '{trimmed}' must be fail, continue, or goto:<step_id>"
                        ));
                    }
                }
            }
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
            "subpipeline" if subpipeline.is_none() => {
                return Err(format!(
                    "steps_v2[{index}] of kind 'subpipeline' requires routine_id"
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
                "subpipeline" => subpipeline
                    .as_ref()
                    .map(|s| format!("Subpipeline {}", s.routine_id))
                    .unwrap_or_else(|| "Subpipeline".to_string()),
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
        spec.set_step_meta(
            depends_on,
            for_each,
            deliver,
            timeout_secs,
            retry_max,
            retry_backoff_ms,
            on_error,
            subpipeline,
        );
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


// ---------------------------------------------------------------------------
// Template resolution (`{{inputs.*}}` & `{{steps.<id>.*}}`)
// ---------------------------------------------------------------------------

pub(super) fn find_step_state<'a>(run: &'a PipelineRun, id: &str) -> Option<&'a PipelineStepState> {
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

pub(super) fn child_items_value(state: &PipelineStepState) -> Value {
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

pub(super) fn resolve_nested_json_field(mut cur: &Value, rest: &str) -> Value {
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

pub(super) fn resolve_expr(expr: &str, run: &PipelineRun) -> Option<Value> {
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

pub(super) fn push_capped(out: &mut String, s: &str, cap: usize) {
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

pub(super) fn value_to_inline_str(v: &Value) -> String {
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

pub(super) fn resolve_template_value_depth(val: &Value, run: &PipelineRun, depth: usize) -> Value {
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

pub(super) fn eval_when_expr(expr_raw: &str, run: &PipelineRun) -> bool {
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

pub(super) fn collect_relevant_prior_states<'a>(
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

pub(super) fn should_run_step_for_run(
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

pub(super) fn resolve_for_each_items(tpl: &str, run: &PipelineRun) -> Result<Vec<Value>, String> {
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

pub(super) fn scan_templates_in_str(
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

pub(super) fn scan_templates_in_value(
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

pub(super) fn compute_ancestors_map(steps: &[PipelineStepSpec]) -> HashMap<String, HashSet<String>> {
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
    let mut inputs = parse_inputs(&body["inputs"], true);
    if let Some(trig) = body.get("trigger").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()) {
        inputs.insert(ROUTINE_TRIGGER_KEY.to_string(), trig.to_string());
    }
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
        if step.kind == "subpipeline" {
            if let Some(sub) = step.subpipeline() {
                scan_templates_in_str(&sub.routine_id, &step.id, &step_ids, anc_ref, &mut errors);
                for val in sub.inputs.values() {
                    scan_templates_in_str(val, &step.id, &step_ids, anc_ref, &mut errors);
                }
                if !sub.routine_id.contains("{{") {
                    let root_rid = body["routine_id"].as_str().map(str::trim);
                    if root_rid == Some(sub.routine_id.as_str()) {
                        errors.push(format!(
                            "step '{}': subpipeline cannot reference itself ('{}')",
                            step.id, sub.routine_id
                        ));
                    } else if routines::get_routine(&cfg.data_dir, &sub.routine_id).is_none() {
                        errors.push(format!(
                            "step '{}': unknown subpipeline routine '{}'",
                            step.id, sub.routine_id
                        ));
                    } else {
                        let mut stack: Vec<String> = Vec::new();
                        if let Some(rrid) = root_rid.filter(|s| !s.is_empty()) {
                            stack.push(rrid.to_string());
                        }
                        if let Some(cycle) = find_subpipeline_cycle(&cfg.data_dir, &sub.routine_id, &mut stack, 0) {
                            errors.push(format!(
                                "step '{}': cyclic subpipeline reference detected ({})",
                                step.id, cycle.join(" -> ")
                            ));
                        }
                    }
                }
            }
        }
        if let Some(oe) = step.on_error() {
            if let Some(target) = oe.strip_prefix("goto:") {
                if !step_ids.contains(target) {
                    errors.push(format!(
                        "step '{}': on_error goto targets unknown step '{target}'",
                        step.id
                    ));
                } else if target == step.id {
                    errors.push(format!(
                        "step '{}': on_error goto cannot target itself",
                        step.id
                    ));
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

pub(super) fn find_subpipeline_cycle(
    person_data_dir: &Path,
    routine_id: &str,
    stack: &mut Vec<String>,
    depth: usize,
) -> Option<Vec<String>> {
    if depth >= MAX_SUBPIPELINE_DEPTH {
        return None;
    }
    if stack.iter().any(|id| id == routine_id) {
        let mut cycle = stack.clone();
        cycle.push(routine_id.to_string());
        return Some(cycle);
    }
    let rt = routines::get_routine(person_data_dir, routine_id)?;
    let v2_raw = rt.get("steps_v2").filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))?;
    let steps = parse_steps_v2(v2_raw).ok()?;
    stack.push(routine_id.to_string());
    for step in &steps {
        if step.kind == "subpipeline" {
            if let Some(sub) = step.subpipeline() {
                if !sub.routine_id.contains("{{") {
                    if let Some(cycle) =
                        find_subpipeline_cycle(person_data_dir, &sub.routine_id, stack, depth + 1)
                    {
                        return Some(cycle);
                    }
                }
            }
        }
    }
    stack.pop();
    None
}

/// Whether executing `steps_v2` (and any reachable `subpipeline` routines) can
/// reach an `"agent"` step requiring an LLM.
pub fn steps_require_llm(person_data_dir: &Path, steps_v2: &[PipelineStepSpec]) -> bool {
    let mut visited = HashSet::new();
    steps_require_llm_inner(person_data_dir, steps_v2, &mut visited)
}

pub(super) fn steps_require_llm_inner(
    person_data_dir: &Path,
    steps_v2: &[PipelineStepSpec],
    visited: &mut HashSet<String>,
) -> bool {
    for step in steps_v2 {
        match step.kind.as_str() {
            "agent" => return true,
            "subpipeline" => {
                if let Some(sub) = step.subpipeline() {
                    if sub.routine_id.contains("{{") {
                        return true;
                    }
                    if !visited.insert(sub.routine_id.clone()) {
                        continue;
                    }
                    let Some(rt) = routines::get_routine(person_data_dir, &sub.routine_id) else {
                        return true;
                    };
                    if routine_value_requires_llm_inner(person_data_dir, &rt, visited) {
                        return true;
                    }
                } else {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether a persisted routine JSON record requires an LLM to run.
pub fn routine_value_requires_llm(person_data_dir: &Path, routine: &Value) -> bool {
    let mut visited = HashSet::new();
    if let Some(id) = routine.get("id").and_then(|v| v.as_str()) {
        visited.insert(id.to_string());
    }
    routine_value_requires_llm_inner(person_data_dir, routine, &mut visited)
}

pub(super) fn routine_value_requires_llm_inner(
    person_data_dir: &Path,
    routine: &Value,
    visited: &mut HashSet<String>,
) -> bool {
    if let Some(v2_raw) = routine
        .get("steps_v2")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
    {
        if let Ok(steps_v2) = parse_steps_v2(v2_raw) {
            return steps_require_llm_inner(person_data_dir, &steps_v2, visited);
        }
    }
    true
}

/// Whether `routine_id` already has an active `"running"` pipeline run persisted.
pub fn has_running_pipeline_for_routine(person_data_dir: &Path, routine_id: &str) -> bool {
    load_runs(person_data_dir)
        .into_iter()
        .any(|r| r.status == "running" && r.routine_id.as_deref() == Some(routine_id))
}

