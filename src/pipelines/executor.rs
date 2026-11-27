use super::*;

// ---------------------------------------------------------------------------
// Step Execution & Metrics Extraction
// ---------------------------------------------------------------------------

pub(super) fn extract_table_pipe_u64(text: &str, needle: &str) -> u64 {
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

pub(super) fn extract_step_metrics(cfg: &Config, files: &[String], output: &str) -> Value {
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

pub(super) fn execute_skill_step(
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

/// Returns parsed `steps_v2` when a routine consists strictly of deterministic
/// zero-LLM steps (`skill` / `export`) that can execute without a model API key.
pub fn parse_deterministic_steps_v2(routine: &Value) -> Option<Vec<PipelineStepSpec>> {
    let raw = routine
        .get("steps_v2")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))?;
    let steps = parse_steps_v2(raw).ok()?;
    if steps
        .iter()
        .all(|s| matches!(s.kind.as_str(), "skill" | "export"))
    {
        Some(steps)
    } else {
        None
    }
}

/// Whether a routine consists solely of deterministic zero-LLM `steps_v2`
/// (`skill` / `export`) and can therefore fire on schedule without a model key.
pub fn routine_is_deterministic_zero_llm(routine: &Value) -> bool {
    parse_deterministic_steps_v2(routine).is_some()
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
    execute_deterministic_steps_sync_with_options(
        cfg,
        routine_id,
        routine_name,
        steps,
        inputs,
        None,
        true,
    )
}

/// Variant of `execute_deterministic_steps_sync` that lets callers (such as
/// `server::execute_routine_run_with_inputs`) supply the authoritative
/// `thread_id` for receipt linkage and opt out of an internal
/// `routines::record_run` call when the caller records the run itself.
pub fn execute_deterministic_steps_sync_with_options(
    cfg: &Config,
    routine_id: Option<&str>,
    routine_name: &str,
    steps: &[PipelineStepSpec],
    inputs: HashMap<String, String>,
    thread_id_override: Option<&str>,
    record_routine_run: bool,
) -> Result<PipelineRun, String> {
    let now_ms = records::now_millis();
    let seq = PIPELINE_SEQ.fetch_add(1, Ordering::SeqCst);
    let run_id = format!("prun-{now_ms}-{seq}");
    let thread_id = thread_id_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("starter-{}", routine_id.unwrap_or(&run_id)));
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
    if record_routine_run {
        if let Some(rid) = routine_id {
            let _ = routines::record_run(&cfg.data_dir, rid, &thread_id, "finished");
        }
    }
    Ok(run)
}

pub(super) fn build_checkpoint_from_step(step: &PipelineStepSpec, run: &PipelineRun) -> PipelineCheckpointSpec {
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
pub(super) fn park_pipeline_at_checkpoint_with_options(
    env: &PipelineExecEnv,
    run_id: &str,
    step_idx: usize,
    cp_spec: PipelineCheckpointSpec,
    prior_files: Vec<String>,
    prior_output: Option<String>,
    existing_decision_id: Option<String>,
    updated_snapshot: Option<AgentSnapshot>,
) -> Result<PipelineRun, String> {
    park_pipeline_checkpoint_inner(
        env,
        run_id,
        step_idx,
        cp_spec,
        prior_files,
        prior_output,
        existing_decision_id,
        updated_snapshot,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn park_pipeline_checkpoint_inner(
    env: &PipelineExecEnv,
    run_id: &str,
    step_idx: usize,
    cp_spec: PipelineCheckpointSpec,
    prior_files: Vec<String>,
    prior_output: Option<String>,
    existing_decision_id: Option<String>,
    updated_snapshot: Option<AgentSnapshot>,
    set_paused_status: bool,
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
        if set_paused_status {
            r.status = "paused_checkpoint".to_string();
        }
        r.current_step_index = step_idx;
        r.updated_ms = now_ms;
        r.add_pending_checkpoint(pending.clone());
        if let Some(ref snap) = updated_snapshot {
            r.set_step_snapshot(&step_id, Some(snap.clone()));
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
    let event_name = if set_paused_status {
        "pipeline_paused"
    } else {
        "pipeline_step_parked"
    };
    env.events.publish(
        event_name,
        &env.ctx.person_id,
        &updated.run_id,
        &updated.thread_id,
        json!({
            "routine_name": updated.routine_name,
            "step_index": step_idx,
            "step_id": step_id,
            "checkpoint": pending.to_json(),
            "pending_checkpoints": updated.pending_checkpoints().iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "run": updated.to_json(),
            "ts_ms": now_ms,
        }),
    );
    crate::proactive_offers::emit_notification(
        &env.cfg.data_dir,
        Some(&env.events),
        &env.ctx.person_id,
        "approval_parked",
        "Checkpoint awaiting your input",
        &crate::truncate(&pending.prompt, 200),
        "decision",
        &decision_id,
    );
    Ok(updated)
}

pub(super) fn claim_worker_slot(run_id: &str, initial_cancelled: bool) -> Option<(u64, Arc<AtomicBool>)> {
    let mut map = cancel_registry().lock().unwrap_or_else(|p| p.into_inner());
    if map.contains_key(run_id) {
        return None;
    }
    let gen_id = CANCEL_GEN_SEQ.fetch_add(1, Ordering::SeqCst);
    let flag = Arc::new(AtomicBool::new(initial_cancelled));
    map.insert(run_id.to_string(), (gen_id, flag.clone()));
    Some((gen_id, flag))
}

pub(super) fn register_or_replace_cancel_flag(run_id: &str, initial_cancelled: bool) -> (u64, Arc<AtomicBool>) {
    let mut map = cancel_registry().lock().unwrap_or_else(|p| p.into_inner());
    let gen_id = CANCEL_GEN_SEQ.fetch_add(1, Ordering::SeqCst);
    let was_cancelled = initial_cancelled
        || map
            .get(run_id)
            .is_some_and(|(_, f)| f.load(Ordering::SeqCst));
    let flag = Arc::new(AtomicBool::new(was_cancelled));
    map.insert(run_id.to_string(), (gen_id, flag.clone()));
    (gen_id, flag)
}

pub(super) fn unregister_cancel_flag(run_id: &str, gen_id: u64) {
    let mut map = cancel_registry().lock().unwrap_or_else(|p| p.into_inner());
    if map.get(run_id).is_some_and(|(g, _)| *g == gen_id) {
        map.remove(run_id);
    }
}

pub(super) fn register_subpipeline_child(parent_run_id: &str, child_run_id: &str) {
    if parent_run_id.is_empty() || child_run_id.is_empty() {
        return;
    }
    let mut map = subpipeline_children_registry()
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    map.entry(parent_run_id.to_string())
        .or_default()
        .push(child_run_id.to_string());
}

pub(super) fn unregister_subpipeline_child(parent_run_id: &str, child_run_id: &str) {
    if parent_run_id.is_empty() || child_run_id.is_empty() {
        return;
    }
    let mut map = subpipeline_children_registry()
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if let Some(children) = map.get_mut(parent_run_id) {
        children.retain(|id| id != child_run_id);
        if children.is_empty() {
            map.remove(parent_run_id);
        }
    }
}

pub(super) fn is_run_cancelled(person_data_dir: &Path, run_id: &str, flag: &AtomicBool) -> bool {
    if flag.load(Ordering::SeqCst) {
        return true;
    }
    get_run(person_data_dir, run_id)
        .map(|r| r.cancel_requested || r.status == "cancelled")
        .unwrap_or(false)
}

pub(super) fn deterministic_step_usage(
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

pub(super) enum DeliverStepOutcome {
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

pub(super) fn execute_deliver_step(
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

pub(super) enum StepCoreOutcome {
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

pub(super) fn execute_single_step_core(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
    cancel_flag: &Arc<AtomicBool>,
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
        "subpipeline" => {
            let Some(sub) = step.subpipeline() else {
                return StepCoreOutcome::Failed {
                    error: "subpipeline step missing subpipeline spec".to_string(),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            };
            let cur_depth: usize = run_snap
                .inputs
                .get(SUBPIPELINE_DEPTH_KEY)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if cur_depth >= MAX_SUBPIPELINE_DEPTH {
                return StepCoreOutcome::Failed {
                    error: format!(
                        "subpipeline recursion depth limit ({MAX_SUBPIPELINE_DEPTH}) exceeded"
                    ),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            }
            let resolved_rid = resolve_template_str(&sub.routine_id, run_snap)
                .trim()
                .to_string();
            if resolved_rid.is_empty() {
                return StepCoreOutcome::Failed {
                    error: "subpipeline.routine_id resolved to empty string".to_string(),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            }
            let mut stack: Vec<String> = run_snap
                .inputs
                .get(SUBPIPELINE_STACK_KEY)
                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                .unwrap_or_default();
            if let Some(cur_rid) = run_snap.routine_id.as_deref().filter(|s| !s.is_empty()) {
                if !stack.iter().any(|x| x == cur_rid) {
                    stack.push(cur_rid.to_string());
                }
            }
            if stack.iter().any(|x| x == &resolved_rid) {
                return StepCoreOutcome::Failed {
                    error: format!(
                        "cyclic subpipeline detected: {} -> {resolved_rid}",
                        stack.join(" -> ")
                    ),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            }
            stack.push(resolved_rid.clone());
            let mut child_inputs = Map::new();
            for (k, v) in &sub.inputs {
                if !k.starts_with("__ergon_") {
                    child_inputs.insert(k.clone(), json!(resolve_template_str(v, run_snap)));
                }
            }
            child_inputs.insert(
                SUBPIPELINE_DEPTH_KEY.to_string(),
                json!((cur_depth + 1).to_string()),
            );
            child_inputs.insert(
                SUBPIPELINE_STACK_KEY.to_string(),
                json!(serde_json::to_string(&stack).unwrap_or_else(|_| "[]".to_string())),
            );
            let child_body = json!({
                "routine_id": resolved_rid,
                "inputs": Value::Object(child_inputs),
                "engagement_id": run_snap.engagement_id,
            });
            match create_and_start_pipeline_internal(
                env.clone(),
                &child_body,
                false,
                true,
                Some((&run_snap.run_id, cancel_flag.clone())),
            ) {
                Ok(child_run) => {
                    if child_run.status == "completed" {
                        let output = child_run
                            .step_states
                            .iter()
                            .filter(|s| s.status == "ok" && !s.output.is_empty())
                            .map(|s| s.output.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        let mut files = Vec::new();
                        for st in &child_run.step_states {
                            for f in &st.files {
                                if !files.contains(f) {
                                    files.push(f.clone());
                                }
                            }
                        }
                        let metrics = json!({
                            "subpipeline_run_id": child_run.run_id,
                            "subpipeline_routine_id": resolved_rid,
                            "steps_count": child_run.step_states.len(),
                            "files_count": files.len(),
                            "anomalies": 0,
                            "has_anomaly": false,
                        });
                        StepCoreOutcome::Ok {
                            output,
                            files,
                            metrics,
                            usage: child_run.usage,
                            usage_source: "pipeline_subpipeline",
                            child_items: child_run.step_states,
                            consume_snapshot: false,
                        }
                    } else if child_run.status == "cancelled" {
                        StepCoreOutcome::Failed {
                            error: child_run
                                .error
                                .unwrap_or_else(|| "subpipeline cancelled".to_string()),
                            usage: Some(child_run.usage),
                            is_cancel: true,
                            child_items: child_run.step_states,
                        }
                    } else {
                        StepCoreOutcome::Failed {
                            error: child_run.error.unwrap_or_else(|| {
                                format!(
                                    "subpipeline '{}' ended with status '{}'",
                                    resolved_rid, child_run.status
                                )
                            }),
                            usage: Some(child_run.usage),
                            is_cancel: false,
                            child_items: child_run.step_states,
                        }
                    }
                }
                Err((_, e)) => StepCoreOutcome::Failed {
                    error: format!("subpipeline '{resolved_rid}' failed: {e}"),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                },
            }
        }
        _ => {
            // "agent" step
            let raw_prompt = step.prompt.as_deref().unwrap_or(&step.label);
            let mut resolved_prompt = resolve_template_str(raw_prompt, run_snap);
            let snapshot = run_snap
                .step_snapshot(&step.id)
                .or_else(|| run_snap.agent_snapshot());
            if snapshot.is_some() {
                if let Some(ans) = run_snap
                    .step_states
                    .iter()
                    .find(|s| s.step_id == step.id)
                    .and_then(|s| s.human_answer.as_deref())
                {
                    resolved_prompt = format!(
                        "Human responded to checkpoint: {ans}. Continue and complete: {resolved_prompt}"
                    );
                }
            }
            let history = if snapshot.is_some() {
                Vec::new()
            } else {
                let st = env.store.lock().unwrap();
                st.history_json(&run_snap.thread_id)
            };
            let opts = AgentLoopOptions {
                snapshot,
                external_cancel: Some(cancel_flag.clone()),
                ..AgentLoopOptions::default()
            };
            match run_agent_for_pipeline_step(
                &env.cfg,
                &env.ctx,
                &env.store,
                &run_snap.thread_id,
                &run_snap.run_id,
                &step.id,
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

pub(super) fn execute_step_with_for_each(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
    cancel_flag: &Arc<AtomicBool>,
) -> StepCoreOutcome {
    let Some(fe_tpl) = step.for_each() else {
        return execute_single_step_core(env, step, run_snap, cancel_flag);
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
                      mut item_run: PipelineRun,
                      cancel_c: Arc<AtomicBool>| {
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
                let res = execute_single_step_core(&env_c, &step_c, &item_run, &cancel_c);
                let f_ms = records::now_millis();
                let _ = tx_item.send((idx, s_ms, f_ms, res));
            });
    };

    let mut abort_rest = false;
    while next_to_spawn < n || in_flight > 0 {
        if is_run_cancelled(&env.cfg.data_dir, &run_snap.run_id, cancel_flag) {
            abort_rest = true;
        }
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
                cancel_flag.clone(),
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

pub(super) fn execute_step_attempt_with_timeout(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
    cancel_flag: &Arc<AtomicBool>,
) -> StepCoreOutcome {
    let Some(secs) = step.timeout_secs() else {
        return execute_step_with_for_each(env, step, run_snap, cancel_flag);
    };
    let (tx, rx) = mpsc::channel::<StepCoreOutcome>();
    let env_c = env.clone();
    let step_c = step.clone();
    let snap_c = run_snap.clone();
    let cancel_c = cancel_flag.clone();
    let _ = std::thread::Builder::new()
        .name(format!("ergon-step-to-{}", step.id))
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            let res = execute_step_with_for_each(&env_c, &step_c, &snap_c, &cancel_c);
            let _ = tx.send(res);
        });
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        if is_run_cancelled(&env.cfg.data_dir, &run_snap.run_id, cancel_flag) {
            return StepCoreOutcome::Failed {
                error: "run cancelled by user".to_string(),
                usage: None,
                is_cancel: true,
                child_items: Vec::new(),
            };
        }
        let rem = deadline.saturating_duration_since(Instant::now());
        if rem.is_zero() {
            return StepCoreOutcome::Failed {
                error: format!("step '{}' timed out after {}s", step.id, secs),
                usage: None,
                is_cancel: false,
                child_items: Vec::new(),
            };
        }
        match rx.recv_timeout(rem.min(Duration::from_millis(50))) {
            Ok(outcome) => return outcome,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return StepCoreOutcome::Failed {
                    error: format!("step '{}' worker exited unexpectedly", step.id),
                    usage: None,
                    is_cancel: false,
                    child_items: Vec::new(),
                };
            }
        }
    }
}

pub(super) fn execute_step_with_policy(
    env: &PipelineExecEnv,
    step: &PipelineStepSpec,
    run_snap: &PipelineRun,
    cancel_flag: &Arc<AtomicBool>,
) -> StepCoreOutcome {
    let max_retries = step.retry_max().unwrap_or(0).min(20);
    let backoff_ms = step.retry_backoff_ms().unwrap_or(0).min(300_000);
    let mut attempt: u32 = 0;
    loop {
        if is_run_cancelled(&env.cfg.data_dir, &run_snap.run_id, cancel_flag) {
            return StepCoreOutcome::Failed {
                error: "run cancelled by user".to_string(),
                usage: None,
                is_cancel: true,
                child_items: Vec::new(),
            };
        }
        let outcome = execute_step_attempt_with_timeout(env, step, run_snap, cancel_flag);
        match outcome {
            StepCoreOutcome::Ok {
                output,
                files,
                mut metrics,
                usage,
                usage_source,
                child_items,
                consume_snapshot,
            } => {
                if attempt > 0 {
                    if let Some(obj) = metrics.as_object_mut() {
                        obj.insert("attempts".to_string(), json!(attempt + 1));
                    }
                }
                return StepCoreOutcome::Ok {
                    output,
                    files,
                    metrics,
                    usage,
                    usage_source,
                    child_items,
                    consume_snapshot,
                };
            }
            held @ StepCoreOutcome::HeldCheckpoint { .. } => return held,
            StepCoreOutcome::Failed {
                error,
                usage,
                is_cancel,
                child_items,
            } => {
                if is_cancel || attempt >= max_retries {
                    return StepCoreOutcome::Failed {
                        error,
                        usage,
                        is_cancel,
                        child_items,
                    };
                }
                attempt += 1;
                if backoff_ms > 0 {
                    let mut remaining = backoff_ms;
                    while remaining > 0 {
                        if is_run_cancelled(&env.cfg.data_dir, &run_snap.run_id, cancel_flag) {
                            return StepCoreOutcome::Failed {
                                error: "run cancelled by user".to_string(),
                                usage,
                                is_cancel: true,
                                child_items,
                            };
                        }
                        let chunk = remaining.min(25);
                        std::thread::sleep(Duration::from_millis(chunk));
                        remaining -= chunk;
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_step_success(
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
            r.set_step_snapshot(&step.id, None);
            if !r.is_dag() {
                r.set_agent_snapshot(None);
            }
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
pub(super) fn record_step_failure(
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
        record_async_routine_completion(env, &failed, next_status, finished_ms);
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

pub(super) fn record_async_routine_completion(
    env: &PipelineExecEnv,
    run: &PipelineRun,
    status: &str,
    finished_ms: u64,
) {
    let Some(rid) = run.routine_id.as_deref() else {
        return;
    };
    let Some(trigger) = run.inputs.get(ROUTINE_TRIGGER_KEY).map(String::as_str) else {
        return;
    };
    let rt_status = match status {
        "completed" | "ok" => "finished",
        "cancelled" => "cancelled",
        _ => "error",
    };
    let summary: String = if status == "completed" || status == "ok" {
        run.step_states
            .iter()
            .filter(|s| s.status == "ok" && !s.output.is_empty())
            .map(|s| s.output.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        run.error.clone().unwrap_or_else(|| status.to_string())
    };
    let mut receipt_id = receipts::load_chain(&env.cfg.data_dir)
        .iter()
        .rev()
        .find(|r| r.thread_id == run.thread_id)
        .map(|r| r.id.clone());
    if (status == "completed" || status == "ok") && receipt_id.is_none() {
        records::append_work(
            &env.cfg,
            &run.routine_name,
            if summary.is_empty() {
                "Completed deterministic routine."
            } else {
                &summary
            },
            &[],
            &run.thread_id,
            true,
        );
        receipt_id = receipts::load_chain(&env.cfg.data_dir)
            .iter()
            .rev()
            .find(|r| r.thread_id == run.thread_id)
            .map(|r| r.id.clone());
    }
    env.search_cache.invalidate(&env.ctx.person_id);
    let hist_status = if status == "completed" { "ok" } else { status };
    let rec = scheduler::new_run_record(
        rid,
        &run.routine_name,
        run.created_ms,
        finished_ms,
        hist_status,
        receipt_id.clone(),
        Some(run.thread_id.clone()),
        &summary,
        trigger,
        run.error.clone(),
    );
    let _ = scheduler::record_run_history(&env.cfg.data_dir, rec.clone());
    let updated = routines::record_run(&env.cfg.data_dir, rid, &run.thread_id, rt_status);
    let next_ms = updated
        .as_ref()
        .and_then(|u| scheduler::compute_routine_next_run_ms(u, finished_ms));
    env.events.publish(
        "routine_fired",
        &env.ctx.person_id,
        &rec.run_id,
        &run.thread_id,
        json!({
            "routine_id": rid,
            "routine_name": run.routine_name,
            "trigger": trigger,
            "status": hist_status,
            "thread_id": run.thread_id,
            "pipeline_run_id": run.run_id,
            "receipt_id": receipt_id,
            "duration_ms": rec.duration_ms,
            "summary": summary.chars().take(240).collect::<String>(),
            "error": run.error,
            "next_run_ms": next_ms,
            "ts_ms": finished_ms,
        }),
    );
}

pub(super) fn emit_step_skipped(
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

pub(super) fn finalize_cancelled_run(env: &PipelineExecEnv, run_id: &str) -> Result<PipelineRun, String> {
    let now_ms = records::now_millis();
    let cancelled = mutate_run(&env.cfg.data_dir, run_id, |r| {
        r.status = "cancelled".to_string();
        r.updated_ms = now_ms;
        r.set_pending_checkpoints(Vec::new());
        Ok(r.clone())
    })?;
    record_async_routine_completion(env, &cancelled, "cancelled", now_ms);
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

pub(super) fn execute_pipeline_loop(
    env: &PipelineExecEnv,
    run_id: &str,
    start_index: usize,
) -> Result<PipelineRun, String> {
    let initial_cancelled = get_run(&env.cfg.data_dir, run_id)
        .is_some_and(|r| r.cancel_requested || r.status == "cancelled");
    let slot = register_or_replace_cancel_flag(run_id, initial_cancelled);
    execute_pipeline_loop_with_slot(env, run_id, start_index, slot)
}

pub(super) fn execute_pipeline_loop_with_slot(
    env: &PipelineExecEnv,
    run_id: &str,
    start_index: usize,
    slot: (u64, Arc<AtomicBool>),
) -> Result<PipelineRun, String> {
    let (gen_id, cancel_flag) = slot;
    struct CancelGuard<'a>(&'a str, u64);
    impl Drop for CancelGuard<'_> {
        fn drop(&mut self) {
            unregister_cancel_flag(self.0, self.1);
        }
    }
    let _cg = CancelGuard(run_id, gen_id);

    let initial = get_run(&env.cfg.data_dir, run_id)
        .ok_or_else(|| "pipeline run not found".to_string())?;
    let total_steps = initial.steps.len();

    if initial.is_dag() {
        return execute_dag_pipeline_loop(env, run_id, &cancel_flag);
    }

    let mut idx = start_index;
    let mut goto_hops = 0usize;
    let mut force_next = false;
    while idx < total_steps {
        if is_run_cancelled(&env.cfg.data_dir, run_id, &cancel_flag) {
            return finalize_cancelled_run(env, run_id);
        }

        let run_snap = get_run(&env.cfg.data_dir, run_id)
            .ok_or_else(|| "pipeline run not found".to_string())?;
        let step = run_snap.steps[idx].clone();

        if !force_next {
            let (should_run, skip_reason) = should_run_step_for_run(&step, idx, &run_snap);
            if !should_run {
                emit_step_skipped(env, run_id, idx, &step, skip_reason)?;
                idx += 1;
                continue;
            }
        }
        force_next = false;

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
        match execute_step_with_policy(env, &step, &run_snap, &cancel_flag) {
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
                idx += 1;
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
                let on_error = step.on_error();
                if !is_cancel {
                    if matches!(on_error.as_deref(), Some("continue")) {
                        record_step_failure(
                            env,
                            run_id,
                            idx,
                            &step,
                            elapsed_ms,
                            error,
                            usage,
                            false,
                            child_items,
                            false,
                        )?;
                        idx += 1;
                        continue;
                    }
                    if let Some(target_id) = on_error.as_deref().and_then(|s| s.strip_prefix("goto:")) {
                        let target_id = target_id.trim();
                        if let Some(target_idx) = run_snap.steps.iter().position(|s| s.id == target_id) {
                            goto_hops += 1;
                            if goto_hops > MAX_PIPELINE_STEPS * 2 {
                                return record_step_failure(
                                    env,
                                    run_id,
                                    idx,
                                    &step,
                                    elapsed_ms,
                                    format!("on_error goto exceeded max hops: {error}"),
                                    usage,
                                    false,
                                    child_items,
                                    true,
                                );
                            }
                            record_step_failure(
                                env,
                                run_id,
                                idx,
                                &step,
                                elapsed_ms,
                                error,
                                usage,
                                false,
                                child_items,
                                false,
                            )?;
                            if target_idx > idx {
                                for mid in (idx + 1)..target_idx {
                                    let mid_step = run_snap.steps[mid].clone();
                                    emit_step_skipped(
                                        env,
                                        run_id,
                                        mid,
                                        &mid_step,
                                        "Skipped by on_error goto.",
                                    )?;
                                }
                            }
                            let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                                if let Some(st) = r.step_states.get_mut(target_idx) {
                                    *st = PipelineStepState::pending_for(&r.steps[target_idx]);
                                }
                                Ok(())
                            });
                            idx = target_idx;
                            force_next = true;
                            continue;
                        }
                    }
                }
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

pub(super) struct DagStepDoneMsg {
    idx: usize,
    elapsed_ms: u64,
    outcome: StepCoreOutcome,
}

pub(super) fn dag_deps_ready(
    deps: &[String],
    state_map: &HashMap<String, String>,
    settled_error_steps: &HashSet<String>,
) -> bool {
    deps.iter().all(|d| {
        matches!(state_map.get(d.as_str()).map(String::as_str), Some("ok" | "skipped"))
            || (state_map.get(d.as_str()).map(String::as_str) == Some("failed")
                && settled_error_steps.contains(d))
    })
}

pub(super) fn execute_dag_pipeline_loop(
    env: &PipelineExecEnv,
    run_id: &str,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<PipelineRun, String> {
    let (tx, rx) = mpsc::channel::<DagStepDoneMsg>();
    let mut in_flight: HashSet<usize> = HashSet::new();
    let mut draining_failure: Option<(String, bool)> = None;
    let mut settled_error_steps: HashSet<String> = HashSet::new();
    let mut goto_redirected_steps: HashMap<String, String> = HashMap::new();
    let mut goto_forced: HashSet<String> = HashSet::new();
    let mut goto_hops: usize = 0;

    if let Some(init_snap) = get_run(&env.cfg.data_dir, run_id) {
        for (i, st) in init_snap.step_states.iter().enumerate() {
            if st.status == "failed" {
                if let Some(oe) = init_snap.steps[i].on_error() {
                    if oe == "continue" {
                        settled_error_steps.insert(st.step_id.clone());
                    } else if let Some(target) = oe.strip_prefix("goto:") {
                        settled_error_steps.insert(st.step_id.clone());
                        goto_redirected_steps.insert(st.step_id.clone(), target.trim().to_string());
                    }
                }
            }
        }
    }

    loop {
        if is_run_cancelled(&env.cfg.data_dir, run_id, cancel_flag) && draining_failure.is_none() {
            draining_failure = Some(("run cancelled by user".to_string(), true));
        }

        let mut run_snap = get_run(&env.cfg.data_dir, run_id)
            .ok_or_else(|| "pipeline run not found".to_string())?;
        let total_steps = run_snap.steps.len();

        if draining_failure.is_none() {
            // Find ready pending steps and evaluate skips (`when` / goto redirects).
            loop {
                let mut skipped_any = false;
                let state_map: HashMap<String, String> = run_snap
                    .step_states
                    .iter()
                    .map(|s| (s.step_id.clone(), s.status.clone()))
                    .collect();
                for idx in 0..total_steps {
                    if in_flight.contains(&idx) || run_snap.step_states[idx].status != "pending" {
                        continue;
                    }
                    let step = run_snap.steps[idx].clone();
                    let deps = step.depends_on();
                    if !dag_deps_ready(&deps, &state_map, &settled_error_steps) {
                        continue;
                    }
                    // If an upstream dep failed with goto:<target> and this step is NOT the goto target
                    // (nor forced), skip it.
                    let redirected_away = !goto_forced.contains(&step.id)
                        && deps.iter().any(|d| {
                            state_map.get(d).map(String::as_str) == Some("failed")
                                && goto_redirected_steps
                                    .get(d)
                                    .is_some_and(|t| t != &step.id)
                        });
                    if redirected_away {
                        run_snap = emit_step_skipped(
                            env,
                            run_id,
                            idx,
                            &step,
                            "Skipped because upstream dependency redirected via on_error goto.",
                        )?;
                        skipped_any = true;
                        break;
                    }
                    if !goto_forced.contains(&step.id) {
                        let (should_run, skip_reason) =
                            should_run_step_for_run(&step, idx, &run_snap);
                        if !should_run {
                            run_snap = emit_step_skipped(env, run_id, idx, &step, skip_reason)?;
                            skipped_any = true;
                            break;
                        }
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
                let step = run_snap.steps[idx].clone();
                let deps = step.depends_on();
                if !goto_forced.contains(&step.id)
                    && !dag_deps_ready(&deps, &state_map, &settled_error_steps)
                {
                    continue;
                }
                goto_forced.remove(&step.id);

                if step.kind == "checkpoint" {
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
                    run_snap = park_pipeline_checkpoint_inner(
                        env,
                        run_id,
                        idx,
                        cp_spec,
                        Vec::new(),
                        None,
                        None,
                        None,
                        false,
                    )?;
                    continue;
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
                let cancel_w = cancel_flag.clone();
                let _ = std::thread::Builder::new()
                    .name(format!("ergon-dag-{}-{}", run_id, step.id))
                    .stack_size(4 * 1024 * 1024)
                    .spawn(move || {
                        let t0 = std::time::Instant::now();
                        let outcome = execute_step_with_policy(&env_w, &step, &snap_w, &cancel_w);
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
                record_async_routine_completion(env, &failed, "failed", finished_ms);
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

            enum BarrierDecision {
                ContinueLoop,
                Paused(Box<PipelineRun>),
                Complete,
                Stalled,
            }
            let settled_clone = settled_error_steps.clone();
            let forced_clone = goto_forced.clone();
            let now_ms = records::now_millis();
            let decision = mutate_run(&env.cfg.data_dir, run_id, |r| {
                let s_map: HashMap<String, String> = r
                    .step_states
                    .iter()
                    .map(|s| (s.step_id.clone(), s.status.clone()))
                    .collect();
                let has_ready = (0..r.steps.len()).any(|i| {
                    r.step_states[i].status == "pending"
                        && (forced_clone.contains(&r.steps[i].id)
                            || dag_deps_ready(
                                &r.steps[i].depends_on(),
                                &s_map,
                                &settled_clone,
                            ))
                });
                if has_ready {
                    r.status = "running".to_string();
                    return Ok(BarrierDecision::ContinueLoop);
                }
                let cps = r.pending_checkpoints();
                if !cps.is_empty() {
                    r.status = "paused_checkpoint".to_string();
                    r.pending_checkpoint = cps.first().cloned();
                    r.updated_ms = now_ms;
                    return Ok(BarrierDecision::Paused(Box::new(r.clone())));
                }
                let all_settled = r.step_states.iter().all(|s| {
                    matches!(s.status.as_str(), "ok" | "skipped")
                        || (s.status == "failed" && settled_clone.contains(&s.step_id))
                });
                if all_settled {
                    return Ok(BarrierDecision::Complete);
                }
                Ok(BarrierDecision::Stalled)
            })?;

            match decision {
                BarrierDecision::ContinueLoop => continue,
                BarrierDecision::Paused(paused_run) => {
                    let cps = paused_run.pending_checkpoints();
                    let first_cp = cps.first().cloned();
                    env.events.publish(
                        "pipeline_paused",
                        &env.ctx.person_id,
                        &paused_run.run_id,
                        &paused_run.thread_id,
                        json!({
                            "routine_name": paused_run.routine_name,
                            "step_index": first_cp.as_ref().map(|c| c.step_index).unwrap_or(0),
                            "step_id": first_cp.as_ref().map(|c| c.step_id.clone()).unwrap_or_default(),
                            "checkpoint": first_cp.as_ref().map(|c| c.to_json()),
                            "pending_checkpoints": cps.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
                            "run": paused_run.to_json(),
                            "ts_ms": now_ms,
                        }),
                    );
                    return Ok(*paused_run);
                }
                BarrierDecision::Complete => {
                    return finalize_completed_or_cancelled(env, run_id, total_steps, cancel_flag);
                }
                BarrierDecision::Stalled => {
                    return Err("DAG scheduler stalled: unresolvable pending dependencies".to_string());
                }
            }
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
                if let Some(u) = usage {
                    let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                        r.usage.accumulate(&u);
                        if let Some(st) = r.step_states.get_mut(msg.idx) {
                            st.usage = Some(u);
                        }
                        Ok(())
                    });
                }
                park_pipeline_checkpoint_inner(
                    env,
                    run_id,
                    msg.idx,
                    cp_spec,
                    prior_files,
                    prior_output,
                    existing_decision_id,
                    updated_snapshot,
                    false,
                )?;
            }
            StepCoreOutcome::Failed {
                error,
                usage,
                is_cancel,
                child_items,
            } => {
                let on_error = step.on_error();
                let mut handled = false;
                if !is_cancel {
                    if matches!(on_error.as_deref(), Some("continue")) {
                        settled_error_steps.insert(step.id.clone());
                        handled = true;
                    } else if let Some(target_id) =
                        on_error.as_deref().and_then(|s| s.strip_prefix("goto:"))
                    {
                        let target_id = target_id.trim();
                        if let Some(target_idx) =
                            run_snap.steps.iter().position(|s| s.id == target_id)
                        {
                            goto_hops += 1;
                            if goto_hops <= MAX_PIPELINE_STEPS * 2 {
                                settled_error_steps.insert(step.id.clone());
                                goto_redirected_steps
                                    .insert(step.id.clone(), target_id.to_string());
                                goto_forced.insert(target_id.to_string());
                                let _ = mutate_run(&env.cfg.data_dir, run_id, |r| {
                                    if let Some(st) = r.step_states.get_mut(target_idx) {
                                        if st.status != "running" {
                                            *st = PipelineStepState::pending_for(&r.steps[target_idx]);
                                        }
                                    }
                                    Ok(())
                                });
                                handled = true;
                            }
                        }
                    }
                }
                if !handled && draining_failure.is_none() {
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

pub(super) fn finalize_completed_or_cancelled(
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
            r.set_pending_checkpoints(Vec::new());
        } else {
            r.status = "completed".to_string();
            r.current_step_index = total_steps;
            r.set_pending_checkpoints(Vec::new());
            r.error = None;
        }
        r.updated_ms = finished_ms;
        Ok(r.clone())
    })?;
    if completed.status == "cancelled" {
        record_async_routine_completion(env, &completed, "cancelled", finished_ms);
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
    if completed.inputs.contains_key(ROUTINE_TRIGGER_KEY) {
        record_async_routine_completion(env, &completed, "completed", finished_ms);
    } else if let Some(rid) = completed.routine_id.as_deref() {
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

pub(super) fn spawn_pipeline_worker_with_slot(
    env: PipelineExecEnv,
    run_id: String,
    start_index: usize,
    slot: (u64, Arc<AtomicBool>),
) {
    let _ = std::thread::Builder::new()
        .name(format!("ergon-pipe-{run_id}"))
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            if let Err(err) = execute_pipeline_loop_with_slot(&env, &run_id, start_index, slot) {
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
                        record_async_routine_completion(&env, &failed, "failed", now_ms);
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

pub(super) fn parse_inputs(val: &Value, strip_internal: bool) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            if strip_internal && k.starts_with("__ergon_") {
                continue;
            }
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
    create_and_start_pipeline_internal(env, body, async_exec, false, None)
}

pub(super) fn create_and_start_pipeline_internal(
    env: PipelineExecEnv,
    body: &Value,
    async_exec: bool,
    allow_internal_inputs: bool,
    parent_sub: Option<(&str, Arc<AtomicBool>)>,
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
    let mut inputs = parse_inputs(&body["inputs"], !allow_internal_inputs);
    if let Some(trig) = body.get("trigger").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()) {
        inputs.insert(ROUTINE_TRIGGER_KEY.to_string(), trig.to_string());
    }
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

    if let Some((parent_run_id, parent_cancel)) = &parent_sub {
        if parent_cancel.load(Ordering::SeqCst) {
            let _ = cancel_pipeline(&env, &run_id);
            return get_run(&env.cfg.data_dir, &run_id)
                .ok_or_else(|| (500, "pipeline run not found".to_string()));
        }
        register_subpipeline_child(parent_run_id, &run_id);
    }

    struct SubChildGuard<'a>(Option<&'a str>, &'a str);
    impl Drop for SubChildGuard<'_> {
        fn drop(&mut self) {
            if let Some(pid) = self.0 {
                unregister_subpipeline_child(pid, self.1);
            }
        }
    }
    let _child_guard = SubChildGuard(parent_sub.as_ref().map(|(pid, _)| *pid), &run_id);

    if async_exec {
        let slot = register_or_replace_cancel_flag(&run_id, false);
        spawn_pipeline_worker_with_slot(env, run_id.clone(), 0, slot);
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
    let checkpoint_id_filter = body["checkpoint_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let action = body["action"]
        .as_str()
        .or_else(|| body["decision"].as_str())
        .unwrap_or("approve")
        .trim()
        .to_ascii_lowercase();
    let choice = body["choice"].as_str().map(str::trim).filter(|s| !s.is_empty());
    let answer = body["answer"]
        .as_str()
        .or_else(|| body["text"].as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let override_args = body.get("override_args").and_then(|v| v.as_object()).cloned();
    let now_ms = records::now_millis();

    enum ResumeTransition {
        Interrupted {
            run: PipelineRun,
            start_idx: usize,
            completed_now: bool,
            worker_slot: Option<(u64, Arc<AtomicBool>)>,
        },
        Rejected {
            run: PipelineRun,
            checkpoint_id: String,
            extra_checkpoint_ids: Vec<String>,
            resolved_answer: String,
        },
        CheckpointApproved {
            run: PipelineRun,
            checkpoint_id: String,
            cp_step_index: usize,
            next_index: usize,
            worker_slot: Option<(u64, Arc<AtomicBool>)>,
            completed_now: bool,
            resolved_answer: String,
        },
    }

    let transition = mutate_run_http(&env.cfg.data_dir, run_id, |r| {
        let cps = r.pending_checkpoints();
        if r.status == "interrupted" && cps.is_empty() {
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
            let worker_slot = if async_exec && !completed_now {
                claim_worker_slot(run_id, false)
            } else {
                None
            };
            return Ok(ResumeTransition::Interrupted {
                run: r.clone(),
                start_idx,
                completed_now,
                worker_slot,
            });
        }

        if r.status != "paused_checkpoint" && !(r.is_dag() && r.status == "running" && !cps.is_empty()) {
            return Err((
                409,
                format!("pipeline run is '{}', not paused at a checkpoint", r.status),
            ));
        }
        let cp = match checkpoint_id_filter {
            Some(target_id) => cps
                .iter()
                .find(|c| c.checkpoint_id == target_id)
                .cloned()
                .ok_or_else(|| {
                    (
                        404,
                        format!("checkpoint '{target_id}' is not pending on this pipeline run"),
                    )
                })?,
            None => cps
                .first()
                .cloned()
                .ok_or_else(|| (409, "pipeline run has no pending checkpoint".to_string()))?,
        };

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
            let all_cps = r.take_pending_checkpoints();
            let extra_checkpoint_ids: Vec<String> = all_cps
                .into_iter()
                .map(|c| c.checkpoint_id)
                .filter(|cid| cid != &cp.checkpoint_id)
                .collect();
            r.cancel_requested = true;
            r.error = Some("Aborted at human checkpoint.".to_string());
            for st in &mut r.step_states {
                if st.status == "waiting_human" || st.status == "running" {
                    st.status = "failed".to_string();
                    st.finished_ms = now_ms;
                    st.error = Some("Aborted by user at checkpoint.".to_string());
                }
            }
            if let Some(st) = r.step_states.get_mut(cp.step_index) {
                st.human_action = Some(action.clone());
                st.human_answer = Some(resolved_answer.clone());
            }
            return Ok(ResumeTransition::Rejected {
                run: r.clone(),
                checkpoint_id: cp.checkpoint_id,
                extra_checkpoint_ids,
                resolved_answer,
            });
        }

        let is_dag = r.is_dag();
        let next_index = cp.step_index + 1;
        let is_agent_cp = r
            .steps
            .get(cp.step_index)
            .is_some_and(|s| s.kind == "agent")
            && r.step_snapshot(&cp.step_id).is_some();

        if let Some(st) = r.step_states.get_mut(cp.step_index) {
            if is_agent_cp {
                st.status = "pending".to_string();
            } else {
                st.status = "ok".to_string();
                st.finished_ms = now_ms;
                st.output = resolved_answer.clone();
            }
            st.human_action = Some(action.clone());
            st.human_answer = Some(resolved_answer.clone());
            st.error = None;
        }
        r.remove_pending_checkpoint(&cp.checkpoint_id);
        let remaining_cps = r.pending_checkpoints();

        let (completed_now, should_run_worker) = if is_dag {
            let s_map: HashMap<String, String> = r
                .step_states
                .iter()
                .map(|s| (s.step_id.clone(), s.status.clone()))
                .collect();
            let settled_err: HashSet<String> = r
                .steps
                .iter()
                .enumerate()
                .filter(|&(i, s)| {
                    r.step_states.get(i).is_some_and(|st| st.status == "failed")
                        && s.on_error().is_some()
                })
                .map(|(_, s)| s.id.clone())
                .collect();
            let any_pending = r.step_states.iter().any(|s| s.status == "pending");
            let any_running = r.step_states.iter().any(|s| s.status == "running");
            let any_ready = (0..r.steps.len()).any(|i| {
                r.step_states[i].status == "pending"
                    && dag_deps_ready(&r.steps[i].depends_on(), &s_map, &settled_err)
            });
            let worker_already_running = cancel_registry()
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .contains_key(run_id);
            if !any_pending && !any_running && remaining_cps.is_empty() {
                r.status = "completed".to_string();
                (true, false)
            } else if any_ready || any_running || worker_already_running {
                r.status = "running".to_string();
                (false, !worker_already_running)
            } else {
                r.status = "paused_checkpoint".to_string();
                (false, false)
            }
        } else {
            let has_more = is_agent_cp || next_index < r.steps.len();
            r.status = if has_more {
                "running".to_string()
            } else {
                "completed".to_string()
            };
            (!has_more, has_more)
        };

        r.current_step_index = if is_agent_cp { cp.step_index } else { next_index };
        r.updated_ms = now_ms;
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
        let worker_slot = if async_exec && should_run_worker {
            claim_worker_slot(run_id, false)
        } else {
            None
        };
        Ok(ResumeTransition::CheckpointApproved {
            run: r.clone(),
            checkpoint_id: cp.checkpoint_id,
            cp_step_index: cp.step_index,
            next_index: if is_agent_cp { cp.step_index } else { next_index },
            worker_slot,
            completed_now,
            resolved_answer,
        })
    })?;

    match transition {
        ResumeTransition::Interrupted {
            run,
            start_idx,
            completed_now,
            worker_slot,
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
                if let Some(slot) = worker_slot {
                    spawn_pipeline_worker_with_slot(env, run_id.to_string(), start_idx, slot);
                }
                Ok(run)
            } else {
                execute_pipeline_loop(&env, run_id, start_idx).map_err(|e| (500, e))
            }
        }
        ResumeTransition::Rejected {
            run,
            checkpoint_id,
            extra_checkpoint_ids,
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
            for extra_id in extra_checkpoint_ids {
                if !extra_id.is_empty() {
                    let _ = records::resolve_decision(&env.cfg, &extra_id, false);
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
            worker_slot,
            completed_now,
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
                    "checkpoint_id": checkpoint_id,
                    "action": action,
                    "human_answer": resolved_answer,
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
                if let Some(slot) = worker_slot {
                    spawn_pipeline_worker_with_slot(env, run_id.to_string(), next_index, slot);
                }
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
    let (updated, idx, worker_slot) = mutate_run_http(&env.cfg.data_dir, run_id, |r| {
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
        r.set_pending_checkpoints(Vec::new());
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
        let worker_slot = if async_exec {
            claim_worker_slot(run_id, false)
        } else {
            None
        };
        Ok((r.clone(), idx, worker_slot))
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
        if let Some(slot) = worker_slot {
            spawn_pipeline_worker_with_slot(env, run_id.to_string(), idx, slot);
        }
        Ok(updated)
    } else {
        execute_pipeline_loop(&env, run_id, idx).map_err(|e| (500, e))
    }
}

pub fn cancel_pipeline(env: &PipelineExecEnv, run_id: &str) -> Result<PipelineRun, (u16, String)> {
    if let Some((_, flag)) = cancel_registry()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(run_id)
    {
        flag.store(true, Ordering::SeqCst);
    }

    let child_ids: Vec<String> = subpipeline_children_registry()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(run_id)
        .cloned()
        .unwrap_or_default();
    for cid in child_ids {
        let _ = cancel_pipeline(env, &cid);
    }

    let now_ms = records::now_millis();
    let (updated, was_paused_or_interrupted, linked_cp_ids) =
        mutate_run_http(&env.cfg.data_dir, run_id, |r| {
            if matches!(r.status.as_str(), "completed" | "failed" | "cancelled") {
                return Ok((r.clone(), false, Vec::new()));
            }
            r.cancel_requested = true;
            r.updated_ms = now_ms;
            let mut transitioned = false;
            let mut cp_ids = Vec::new();
            for cp in r.take_pending_checkpoints() {
                cp_ids.push(cp.checkpoint_id);
            }
            if matches!(r.status.as_str(), "paused_checkpoint" | "interrupted") {
                transitioned = true;
                r.status = "cancelled".to_string();
                for st in &mut r.step_states {
                    if st.status == "waiting_human" || st.status == "running" {
                        st.status = "failed".to_string();
                        st.finished_ms = now_ms;
                        st.error = Some("Cancelled by user.".to_string());
                    }
                }
            }
            Ok((r.clone(), transitioned, cp_ids))
        })?;

    for did in linked_cp_ids {
        let _ = records::resolve_decision(&env.cfg, &did, false);
    }
    if let Ok(runs) = env.live_runs.lock() {
        for live in runs.values() {
            if live.person_id == env.ctx.person_id
                && (live.pipeline_run_id.as_deref() == Some(run_id)
                    || (live.pipeline_run_id.is_none()
                        && !updated.thread_id.is_empty()
                        && live.thread_id == updated.thread_id))
            {
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
    run.set_pending_checkpoints(vec![pending.clone()]);
    if let Some(snap) = agent_snapshot {
        run.set_agent_snapshot(Some(snap.clone()));
        run.set_step_snapshot("continue", Some(snap));
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
    let notif_person = live
        .map(|(_, pid)| pid.to_string())
        .unwrap_or_else(|| records::infer_person_id(&cfg.data_dir));
    crate::proactive_offers::emit_notification(
        &cfg.data_dir,
        live.map(|(bus, _)| bus),
        &notif_person,
        "approval_parked",
        "Checkpoint awaiting your input",
        &crate::truncate(&pending.prompt, 200),
        "decision",
        &decision_id,
    );

    Ok((run_id, pending))
}
