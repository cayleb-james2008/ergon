use super::*;

/// Build the shared execution environment for async pipeline workers.
pub(super) fn pipeline_exec_env(
    state: &AppState,
    _person: &Person,
    runtime: &Arc<PersonRuntime>,
    ctx: &ToolCtx,
) -> pipelines::PipelineExecEnv {
    pipelines::PipelineExecEnv {
        cfg: effective_cfg(state, runtime),
        ctx: ctx.clone(),
        store: runtime.store.clone(),
        live_runs: state.live_runs.clone(),
        events: state.events.clone(),
        search_cache: state.search_cache.clone(),
    }
}

/// Build the JSON snapshot returned by `GET /api/runs/status` and the initial
/// `event: status` frame of `GET /api/runs/stream`, combining v0.39.0 multi-run
/// isolation (`runs[]` across concurrent chat + pipeline runs) with persisted
/// crash-recovery state (`recoverable_runs`, `recent_runs`, `model_configured`).
pub(super) fn runs_status_json(state: &AppState, person: &Person, runtime: &Arc<PersonRuntime>) -> Value {
    let model_configured = {
        let live = state.model_cfg.lock().unwrap_or_else(|p| p.into_inner());
        crate::model_configured_for(&live.model_base_url, live.api_key.is_some())
    };
    let now_ms = records::now_millis();
    let mut runs: Vec<LiveRun> = {
        let map = state.live_runs.lock().unwrap_or_else(|p| p.into_inner());
        map.values()
            .filter(|r| r.person_id == person.id)
            .cloned()
            .collect()
    };
    runs.sort_by(|a, b| a.started_ms.cmp(&b.started_ms).then_with(|| a.run_id.cmp(&b.run_id)));
    let running_pipes = pipelines::list_runs(&runtime.cfg.data_dir, Some("running"), 50);
    let pipe_by_id: std::collections::HashMap<&str, &pipelines::PipelineRun> = running_pipes
        .iter()
        .map(|pr| (pr.run_id.as_str(), pr))
        .collect();
    let mut seen_pipe_ids = std::collections::HashSet::new();
    let mut runs_json: Vec<Value> = runs
        .iter()
        .map(|run| {
            let linked_pipe = run
                .pipeline_run_id
                .as_deref()
                .and_then(|pid| {
                    seen_pipe_ids.insert(pid.to_string());
                    pipe_by_id.get(pid).copied()
                });
            let mut obj = json!({
                "active": true,
                "status": run.status,
                "run_id": run.run_id,
                "person_id": run.person_id,
                "thread_id": run.thread_id,
                "pipeline_run_id": run.pipeline_run_id,
                "step_id": run.step_id,
                "task": run.task,
                "started_ms": run.started_ms,
                "elapsed_ms": now_ms.saturating_sub(run.started_ms),
                "steps": run.steps,
            });
            if let Some(pr) = linked_pipe {
                obj["kind"] = json!(if pr.routine_id.is_some() { "routine" } else { "pipeline" });
                obj["routine_id"] = json!(pr.routine_id);
                obj["routine_name"] = json!(pr.routine_name);
            }
            obj
        })
        .collect();
    for pr in &running_pipes {
        if seen_pipe_ids.contains(&pr.run_id) {
            continue;
        }
        let steps_json: Vec<Value> = pr
            .step_states
            .iter()
            .enumerate()
            .map(|(idx, ss)| {
                let st = match ss.status.as_str() {
                    "failed" => "error",
                    "ok" | "skipped" => "done",
                    "waiting_human" => "waiting",
                    _ => "running",
                };
                json!({
                    "id": if ss.step_id.is_empty() { format!("step-{idx}") } else { ss.step_id.clone() },
                    "step_id": ss.step_id,
                    "tool": ss.kind,
                    "label": ss.label,
                    "status": st,
                    "raw": ss.label,
                    "child_items": ss.child_items().into_iter().map(|c| c.to_json()).collect::<Vec<_>>(),
                })
            })
            .collect();
        runs_json.push(json!({
            "active": true,
            "status": "running",
            "run_id": pr.run_id,
            "person_id": person.id,
            "thread_id": pr.thread_id,
            "pipeline_run_id": pr.run_id,
            "step_id": pr.steps.get(pr.current_step_index).map(|s| s.id.as_str()),
            "task": pr.routine_name,
            "started_ms": pr.created_ms,
            "elapsed_ms": now_ms.saturating_sub(pr.created_ms),
            "kind": if pr.routine_id.is_some() { "routine" } else { "pipeline" },
            "routine_id": pr.routine_id,
            "routine_name": pr.routine_name,
            "steps": steps_json,
        }));
    }
    let persisted = live_runs::list_for_person(&state.cfg.data_dir, &person.id);
    let recoverable_runs: Vec<Value> = persisted
        .iter()
        .filter(|r| r.is_recoverable())
        .map(|r| r.to_api_json(model_configured))
        .collect();
    let recent_runs: Vec<Value> = persisted
        .iter()
        .take(10)
        .map(|r| r.to_api_json(model_configured))
        .collect();
    if let Some(latest) = runs.last() {
        json!({
            "active": true,
            "status": latest.status,
            "runs": runs_json,
            "run_id": latest.run_id,
            "thread_id": latest.thread_id,
            "pipeline_run_id": latest.pipeline_run_id,
            "step_id": latest.step_id,
            "task": latest.task,
            "started_ms": latest.started_ms,
            "elapsed_ms": now_ms.saturating_sub(latest.started_ms),
            "steps": latest.steps,
            "model_configured": model_configured,
            "recoverable_runs": recoverable_runs,
            "recent_runs": recent_runs,
        })
    } else if let Some(first_pipe) = runs_json.last() {
        json!({
            "active": true,
            "status": "running",
            "runs": runs_json,
            "run_id": first_pipe["run_id"],
            "thread_id": first_pipe["thread_id"],
            "pipeline_run_id": first_pipe["pipeline_run_id"],
            "step_id": first_pipe["step_id"],
            "task": first_pipe["task"],
            "started_ms": first_pipe["started_ms"],
            "elapsed_ms": first_pipe["elapsed_ms"],
            "steps": first_pipe["steps"],
            "model_configured": model_configured,
            "recoverable_runs": recoverable_runs,
            "recent_runs": recent_runs,
        })
    } else {
        json!({
            "active": false,
            "status": if recoverable_runs.is_empty() { "idle" } else { "interrupted" },
            "runs": [],
            "run_id": "",
            "thread_id": "",
            "pipeline_run_id": Value::Null,
            "step_id": Value::Null,
            "task": "",
            "started_ms": 0,
            "elapsed_ms": 0,
            "steps": [],
            "model_configured": model_configured,
            "recoverable_runs": recoverable_runs,
            "recent_runs": recent_runs,
        })
    }
}

pub(super) enum ResumedRunResult {
    Completed(Value),
    Cancelled(Value),
    SafetyStop { thread_id: String, err: String },
    Failed { status: u16, thread_id: String, err: String },
}

/// Execute a previously claimed persisted live run (`status == "resuming"`) from
/// its saved thread and optional step snapshot. Used by both boot auto-resume
/// and `POST /api/runs/resume`.
pub(super) fn execute_resumed_run(
    state: &Arc<AppState>,
    person: &Person,
    claimed: &live_runs::PersistedLiveRun,
) -> ResumedRunResult {
    let runtime = runtime_for(state, person);
    let cfg = effective_cfg(state, &runtime);
    if !cfg.model_configured() {
        live_runs::revert_resume_claim(&state.cfg.data_dir, &claimed.run_id, NO_KEY_ERROR);
        return ResumedRunResult::Failed {
            status: 503,
            thread_id: claimed.thread_id.clone(),
            err: NO_KEY_ERROR.to_string(),
        };
    }

    let (thread_id, history) = {
        let mut store = runtime.store.lock().unwrap();
        let tid = store.ensure_thread_preserving_id(&claimed.thread_id, &claimed.task);
        let mut hist = if claimed.snapshot.is_some() {
            Vec::new()
        } else {
            store.history_json(&tid)
        };
        if claimed.snapshot.is_none() {
            if let Some(last) = hist.last() {
                if last["role"].as_str() == Some("user")
                    && last["content"].as_str().map(str::trim) == Some(claimed.task.trim())
                {
                    hist.pop();
                }
            }
        }
        store.push(
            &tid,
            "step",
            &format!(
                "Resuming interrupted task (attempt {}, {} prior step(s) preserved).",
                claimed.resume_count,
                claimed.steps.len()
            ),
        );
        (tid, hist)
    };

    let resume_prompt = if claimed.snapshot.is_some() {
        Some(
            "The server restarted while you were working on this task. Continue from your completed steps above and finish the task without repeating already-completed file writes."
                .to_string(),
        )
    } else {
        None
    };

    let opts = AgentLoopOptions {
        snapshot: claimed.snapshot.clone(),
        ..AgentLoopOptions::default()
    };
    let resume_opts = ResumeRunOptions {
        run_id: claimed.run_id.clone(),
        prior_steps: claimed.steps.clone(),
        prior_started_ms: claimed.started_ms,
        resume_prompt,
    };

    let ctx = ToolCtx {
        library: state.library.clone(),
        root_data_dir: state.cfg.data_dir.clone(),
        person_id: person.id.clone(),
    };

    let outcome = run_agent_resumed(
        &cfg,
        &ctx,
        &runtime.store,
        &thread_id,
        &claimed.task,
        &history,
        Some((&state.live_runs, &state.events, &person.id)),
        opts,
        resume_opts,
    );
    state.search_cache.invalidate(&person.id);

    match outcome {
        Ok(outcome) => {
            let store = runtime.store.lock().unwrap();
            let mut v = store
                .thread_json(&thread_id)
                .unwrap_or_else(|| json!({"thread_id": thread_id}));
            if let Some(prun_id) = outcome.paused_run_id {
                v["paused_pipeline_run_id"] = json!(prun_id);
            }
            if let Some(cp) = outcome.paused_checkpoint {
                v["pending_checkpoint"] = json!({
                    "prompt": cp.prompt,
                    "input_mode": cp.input_mode,
                    "choices": cp.choices,
                    "inspect_files": cp.inspect_files,
                });
            }
            v["resumed"] = json!(true);
            v["run_id"] = json!(claimed.run_id);
            v["status"] = json!("completed");
            v["written_files"] = json!(outcome.written_files);
            v["usage"] = outcome.usage.to_json();
            v["engagement_id"] = json!(outcome.engagement_id);
            if outcome.budget_gated {
                v["budget_gated"] = json!(true);
                v["decision_id"] = json!(outcome.decision_id);
            }
            ResumedRunResult::Completed(v)
        }
        Err(e) if e == "run cancelled by user" => {
            let mut store = runtime.store.lock().unwrap();
            store.push(&thread_id, "step", "Run cancelled by user.");
            let mut v = store
                .thread_json(&thread_id)
                .unwrap_or_else(|| json!({"thread_id": thread_id}));
            v["resumed"] = json!(true);
            v["run_id"] = json!(claimed.run_id);
            v["cancelled"] = json!(true);
            v["status"] = json!("cancelled");
            ResumedRunResult::Cancelled(v)
        }
        Err(e) if e.starts_with(tripwire::HALT_PREFIX) => ResumedRunResult::SafetyStop {
            thread_id,
            err: e,
        },
        Err(e) => {
            runtime.store.lock().unwrap().push(&thread_id, "error", &e);
            ResumedRunResult::Failed {
                status: 500,
                thread_id,
                err: e,
            }
        }
    }
}

/// Routes live run status/SSE/cancel/resume, pipelines, apps execution, threads, and chat.
pub(super) fn try_route_runs_and_chat(
    stream: &mut ConnStream,
    state: &Arc<AppState>,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    ctx: &ToolCtx,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/api/pipelines/validate") => {
            if state.gate_token.is_none()
                && !host_allowed(req.host.as_deref().unwrap_or(""), &state.allowed_hosts)
            {
                return Some(respond_err(stream, 403, "loopback host required"));
            }
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                json!({})
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
                }
            };
            let (status, payload) =
                pipelines::validate_pipeline_payload(&runtime.cfg, &state.library, &parsed);
            respond_json(stream, status, &payload)
        }
        ("POST", "/api/pipelines/run") => {
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                json!({})
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
                }
            };
            let env = pipeline_exec_env(state, person, runtime, ctx);
            match pipelines::create_and_start_pipeline(env, &parsed, true) {
                Ok(run) => respond_json(stream, 202, &json!({ "run": run.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/pipelines/runs") => {
            let status_filter = query_param(&req.query, "status");
            let limit = query_param(&req.query, "limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50)
                .clamp(1, 200);
            let runs = pipelines::list_runs(&runtime.cfg.data_dir, status_filter.as_deref(), limit);
            let items: Vec<Value> = runs.into_iter().map(|r| r.to_json()).collect();
            respond_json(stream, 200, &json!({ "runs": items }))
        }
        ("GET", path) if path.starts_with("/api/pipelines/runs/") => {
            let raw_id = path.trim_start_matches("/api/pipelines/runs/").trim();
            let run_id = percent_decode(raw_id).trim().to_string();
            if run_id.is_empty() || run_id.contains('/') || run_id.contains("..") {
                return Some(respond_err(stream, 400, "invalid pipeline run id"));
            }
            match pipelines::get_run(&runtime.cfg.data_dir, &run_id) {
                Some(run) => respond_json(stream, 200, &json!({ "run": run.to_json() })),
                None => respond_err(stream, 404, "pipeline run not found"),
            }
        }
        ("POST", path) if path.starts_with("/api/pipelines/runs/") && path.ends_with("/resume") => {
            let raw_id = path
                .strip_prefix("/api/pipelines/runs/")
                .and_then(|r| r.strip_suffix("/resume"))
                .unwrap_or("")
                .trim();
            let run_id = percent_decode(raw_id).trim().to_string();
            if run_id.is_empty() || run_id.contains('/') || run_id.contains("..") {
                return Some(respond_err(stream, 400, "invalid pipeline run id"));
            }
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                json!({})
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
                }
            };
            let env = pipeline_exec_env(state, person, runtime, ctx);
            match pipelines::resume_pipeline(env, &run_id, &parsed, true) {
                Ok(run) => respond_json(stream, 202, &json!({ "run": run.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", path) if path.starts_with("/api/pipelines/runs/") && path.ends_with("/retry") => {
            let raw_id = path
                .strip_prefix("/api/pipelines/runs/")
                .and_then(|r| r.strip_suffix("/retry"))
                .unwrap_or("")
                .trim();
            let run_id = percent_decode(raw_id).trim().to_string();
            if run_id.is_empty() || run_id.contains('/') || run_id.contains("..") {
                return Some(respond_err(stream, 400, "invalid pipeline run id"));
            }
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                json!({})
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
                }
            };
            let from_step_index = parsed.get("from_step_index").and_then(|v| v.as_u64()).map(|n| n as usize);
            let env = pipeline_exec_env(state, person, runtime, ctx);
            match pipelines::retry_pipeline(env, &run_id, from_step_index, true) {
                Ok(run) => respond_json(stream, 202, &json!({ "run": run.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", path) if path.starts_with("/api/pipelines/runs/") && path.ends_with("/cancel") => {
            let raw_id = path
                .strip_prefix("/api/pipelines/runs/")
                .and_then(|r| r.strip_suffix("/cancel"))
                .unwrap_or("")
                .trim();
            let run_id = percent_decode(raw_id).trim().to_string();
            if run_id.is_empty() || run_id.contains('/') || run_id.contains("..") {
                return Some(respond_err(stream, 400, "invalid pipeline run id"));
            }
            let env = pipeline_exec_env(state, person, runtime, ctx);
            match pipelines::cancel_pipeline(&env, &run_id) {
                Ok(run) => respond_json(stream, 200, &json!({ "run": run.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/runs/status") => {
            respond_json(stream, 200, &runs_status_json(state, person, runtime))
        }
        ("GET", "/api/runs/stream") => {
            let thread_filter = query_param(&req.query, "thread_id");
            let once = query_param(&req.query, "once")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let requested_since = query_param(&req.query, "since")
                .and_then(|s| s.trim().parse::<u64>().ok())
                .or_else(|| {
                    req.last_event_id
                        .as_deref()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                });
            // Capture cursor BEFORE snapshotting run state and BEFORE writing any
            // bytes to the socket so no event published concurrently is ever skipped.
            let cursor = requested_since.unwrap_or_else(|| state.events.head_seq());
            let initial = runs_status_json(state, person, runtime);
            sse::stream_sse_from(
                stream,
                &state.events,
                &person.id,
                thread_filter.as_deref(),
                &initial,
                cursor,
                once,
            )
        }
        ("POST", "/api/runs/resume")
        | ("POST", _) if req.path == "/api/runs/resume" || (req.path.starts_with("/api/runs/") && req.path.ends_with("/resume")) => {
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                json!({})
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => {
                        return Some(respond_err(stream, 400, "request body is not valid JSON"));
                    }
                }
            };
            let path_run_id = if req.path != "/api/runs/resume" {
                req.path
                    .strip_prefix("/api/runs/")
                    .and_then(|r| r.strip_suffix("/resume"))
                    .map(|s| percent_decode(s).trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            };
            let body_run_id = parsed["run_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let target_run_id = path_run_id.or(body_run_id);
            let target_thread_id = parsed["thread_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let allow_failed = parsed["allow_failed"].as_bool().unwrap_or(false);
            let async_mode = parsed["async"].as_bool().unwrap_or(false);

            let eff = effective_cfg(state, runtime);
            if !eff.model_configured() {
                let candidate = {
                    let all = live_runs::list_for_person(&state.cfg.data_dir, &person.id);
                    if let Some(ref rid) = target_run_id {
                        all.into_iter().find(|r| r.run_id == *rid)
                    } else {
                        all.into_iter().find(|r| {
                            r.is_recoverable()
                                && target_thread_id
                                    .as_deref()
                                    .map(|t| r.thread_id == t)
                                    .unwrap_or(true)
                        })
                    }
                };
                return Some(match candidate {
                    Some(run) => respond_json(
                        stream,
                        503,
                        &json!({
                            "error": NO_KEY_ERROR,
                            "run_id": run.run_id,
                            "thread_id": run.thread_id,
                            "status": run.status,
                        }),
                    ),
                    None => respond_err(stream, 404, "no interrupted run available to resume"),
                });
            }

            let busy_now = state
                .live_runs
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .values()
                .any(|r| r.person_id == person.id && r.pipeline_run_id.is_none());
            if busy_now {
                return Some(respond_err(
                    stream,
                    409,
                    "another run is already active for this person",
                ));
            }

            let claimed = match live_runs::claim_for_resume(
                &state.cfg.data_dir,
                &person.id,
                target_run_id.as_deref(),
                target_thread_id.as_deref(),
                allow_failed,
            ) {
                Ok(c) => c,
                Err((status, msg)) => return Some(respond_err(stream, status, &msg)),
            };

            if async_mode {
                let st_bg = state.clone();
                let person_bg = person.clone();
                let claimed_bg = claimed.clone();
                let _ = std::thread::Builder::new()
                    .name("ergon-resume".to_string())
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || {
                        let _ = execute_resumed_run(&st_bg, &person_bg, &claimed_bg);
                    });
                return Some(respond_json(
                    stream,
                    202,
                    &json!({
                        "resuming": true,
                        "run_id": claimed.run_id,
                        "thread_id": claimed.thread_id,
                        "status": "resuming",
                        "prior_steps": claimed.steps.len(),
                    }),
                ));
            }

            match execute_resumed_run(state, person, &claimed) {
                ResumedRunResult::Completed(v) | ResumedRunResult::Cancelled(v) => {
                    respond_json(stream, 200, &v)
                }
                ResumedRunResult::SafetyStop { thread_id, err } => {
                    respond_safety_stop(stream, runtime, &thread_id, &err)
                }
                ResumedRunResult::Failed {
                    status,
                    thread_id,
                    err,
                } => respond_json(
                    stream,
                    status,
                    &json!({
                        "error": err,
                        "run_id": claimed.run_id,
                        "thread_id": thread_id,
                        "status": if status == 503 { "interrupted" } else { "failed" },
                    }),
                ),
            }
        }
        ("POST", path) if path.starts_with("/api/runs/") && path.ends_with("/cancel") => {
            let raw_id = path
                .strip_prefix("/api/runs/")
                .and_then(|r| r.strip_suffix("/cancel"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            let matched: Vec<(String, String)> = {
                let map = state.live_runs.lock().unwrap();
                let mut list: Vec<(u64, String, String)> = map
                    .values()
                    .filter(|run| {
                        run.person_id == person.id
                            && (id == "active"
                                || id.is_empty()
                                || run.run_id == id
                                || run.thread_id == id
                                || run.pipeline_run_id.as_deref() == Some(id.as_str()))
                    })
                    .map(|run| {
                        run.cancel.store(true, Ordering::SeqCst);
                        (run.started_ms, run.run_id.clone(), run.thread_id.clone())
                    })
                    .collect();
                list.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                list.into_iter().map(|(_, rid, tid)| (rid, tid)).collect()
            };
            let mut candidate_pipes = pipelines::list_runs(&runtime.cfg.data_dir, Some("running"), 50);
            if id != "active" && !id.is_empty() {
                candidate_pipes.extend(pipelines::list_runs(&runtime.cfg.data_dir, Some("paused_checkpoint"), 50));
                candidate_pipes.extend(pipelines::list_runs(&runtime.cfg.data_dir, Some("interrupted"), 50));
            }
            let matching_pipes: Vec<(String, String)> = candidate_pipes
                .into_iter()
                .filter(|pr| {
                    id == "active"
                        || id.is_empty()
                        || pr.run_id == id
                        || (!pr.thread_id.is_empty() && pr.thread_id == id)
                })
                .map(|pr| (pr.run_id, pr.thread_id))
                .collect();
            if !matching_pipes.is_empty() {
                let env = pipeline_exec_env(state, person, runtime, ctx);
                for (pr_id, _) in &matching_pipes {
                    let _ = pipelines::cancel_pipeline(&env, pr_id);
                }
            }
            if matched.is_empty() && matching_pipes.is_empty() {
                respond_err(stream, 404, "no matching active run")
            } else {
                let ts_ms = records::now_millis();
                let mut cancelled_run_ids = Vec::with_capacity(matched.len() + matching_pipes.len());
                for (run_id, thread_id) in &matched {
                    cancelled_run_ids.push(run_id.clone());
                    state.events.publish(
                        "run_cancelling",
                        &person.id,
                        run_id,
                        thread_id,
                        json!({"ts_ms": ts_ms}),
                    );
                }
                for (pr_id, _) in &matching_pipes {
                    if !cancelled_run_ids.contains(pr_id) {
                        cancelled_run_ids.push(pr_id.clone());
                    }
                }
                let (first_run_id, first_thread_id) = matched
                    .first()
                    .or_else(|| matching_pipes.first())
                    .expect("non-empty matched or matching_pipes");
                respond_json(
                    stream,
                    200,
                    &json!({
                        "cancelled": true,
                        "run_id": first_run_id,
                        "thread_id": first_thread_id,
                        "cancelled_run_ids": cancelled_run_ids,
                    }),
                )
            }
        }
        ("POST", path) if path.starts_with("/api/apps/") && path.ends_with("/run") => {
            // Single strip, not trim_*_matches (which strips
            // repeatedly and would resolve "/api/apps/foo/run/run"
            // to the app "foo").
            let id = path
                .strip_prefix("/api/apps/")
                .and_then(|rest| rest.strip_suffix("/run"))
                .unwrap_or("")
                .to_string();
            let item = {
                let lib = state.library.lock().unwrap();
                lib.get(&id)
            };
            let Some(item) = item else {
                return Some(respond_err(stream, 404, "app not found"));
            };
            if item.kind != "app" {
                return Some(respond_err(stream, 400, "that library item is not an app"));
            }
            let suite = library::load_suite(&runtime.cfg.data_dir);
            if !suite.installed.contains(&item.id) {
                return Some(respond_err(stream, 400, "that app is not in this person's suite"));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            // Every field the template actually uses must be filled
            // BEFORE substitution — checking afterwards cannot work:
            // substitution erases the evidence of an empty field and
            // the run would start with a hole in its instructions.
            for field in &item.fields {
                if item.template.contains(&format!("{{{field}}}")) {
                    let value = parsed["fields"][field.as_str()]
                        .as_str()
                        .unwrap_or("")
                        .trim();
                    if value.is_empty() {
                        return Some(respond_err(stream, 400, "fill in every field before running the app"));
                    }
                }
            }
            // Fill the template's {field} slots from the run's inputs.
            let mut task = item.template.clone();
            for field in &item.fields {
                let value = parsed["fields"][field.as_str()]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                task = task.replace(&format!("{{{field}}}"), &value);
            }
            let cfg = effective_cfg(state, runtime);
            let thread_id = {
                let mut store = runtime.store.lock().unwrap();
                let tid = store.create_thread();
                store.push(&tid, "user", &format!("{} — {}", item.name, task));
                tid
            };
            if !cfg.model_configured() {
                runtime
                    .store
                    .lock()
                    .unwrap()
                    .push(&thread_id, "error", NO_KEY_ERROR);
                return Some(respond_json(
                    stream,
                    503,
                    &json!({"error": NO_KEY_ERROR, "thread_id": thread_id}),
                ));
            }
            let outcome = run_agent_live(
                &cfg,
                ctx,
                &runtime.store,
                &thread_id,
                &task,
                &[],
                Some((&state.live_runs, &state.events, &person.id)),
            );
            state.search_cache.invalidate(&person.id);
            match outcome {
                Ok(outcome) => {
                    let store = runtime.store.lock().unwrap();
                    let mut v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    if let Some(prun_id) = outcome.paused_run_id {
                        v["paused_pipeline_run_id"] = json!(prun_id);
                    }
                    if let Some(cp) = outcome.paused_checkpoint {
                        v["pending_checkpoint"] = json!({
                            "prompt": cp.prompt,
                            "input_mode": cp.input_mode,
                            "choices": cp.choices,
                            "inspect_files": cp.inspect_files,
                        });
                    }
                    v["usage"] = outcome.usage.to_json();
                    v["engagement_id"] = json!(outcome.engagement_id);
                    if outcome.budget_gated {
                        v["budget_gated"] = json!(true);
                        v["decision_id"] = json!(outcome.decision_id);
                    }
                    if let Some(qb) = outcome.quality_bar {
                        v["quality_bar"] = qb;
                    }
                    respond_json(stream, 200, &v)
                }
                Err(e) if e.starts_with(tripwire::HALT_PREFIX) => {
                    respond_safety_stop(stream, runtime, &thread_id, &e)
                }
                Err(e) => {
                    runtime.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", "/api/threads") => {
            let store = runtime.store.lock().unwrap();
            respond_json(stream, 200, &store.list_json())
        }
        ("POST", "/api/threads") => {
            let mut store = runtime.store.lock().unwrap();
            let id = store.create_thread();
            let v = store.thread_json(&id).unwrap_or_else(|| json!({"id": id}));
            respond_json(stream, 201, &v)
        }
        ("POST", "/api/chat") => {
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                Value::Null
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => {
                        return Some(respond_err(stream, 400, "request body is not valid JSON"))}
                }
            };
            let message = parsed["message"].as_str().unwrap_or("").trim().to_string();
            if message.is_empty() {
                return Some(respond_err(stream, 400, "message is required"));
            }
            let cfg = effective_cfg(state, runtime);
            // The user's words are recorded FIRST, before any failure
            // path: a refused or failed run must never silently lose
            // the task the person typed (the no-key path used to drop
            // it, so the message vanished on reload).
            let (thread_id, history) = {
                let mut store = runtime.store.lock().unwrap();
                let requested = parsed["thread_id"].as_str().unwrap_or("");
                // A named thread must exist: opening a new one silently
                // would put the message somewhere the person never asked (EB3).
                if !requested.is_empty() && store.thread_json(requested).is_none() {
                    return Some(respond_err(stream, 404, "no such thread"));
                }
                let tid = store.ensure_thread(requested);
                let history = store.history_json(&tid);
                store.push(&tid, "user", &message);
                (tid, history)
            };
            if !cfg.model_configured() {
                runtime
                    .store
                    .lock()
                    .unwrap()
                    .push(&thread_id, "error", NO_KEY_ERROR);
                return Some(respond_json(
                    stream,
                    503,
                    &json!({"error": NO_KEY_ERROR, "thread_id": thread_id}),
                ));
            }
            let outcome = run_agent_live(
                &cfg,
                ctx,
                &runtime.store,
                &thread_id,
                &message,
                &history,
                Some((&state.live_runs, &state.events, &person.id)),
            );
            state.search_cache.invalidate(&person.id);
            match outcome {
                Ok(outcome) => {
                    let store = runtime.store.lock().unwrap();
                    let mut v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    if let Some(prun_id) = outcome.paused_run_id {
                        v["paused_pipeline_run_id"] = json!(prun_id);
                    }
                    if let Some(cp) = outcome.paused_checkpoint {
                        v["pending_checkpoint"] = json!({
                            "prompt": cp.prompt,
                            "input_mode": cp.input_mode,
                            "choices": cp.choices,
                            "inspect_files": cp.inspect_files,
                        });
                    }
                    v["usage"] = outcome.usage.to_json();
                    v["engagement_id"] = json!(outcome.engagement_id);
                    if outcome.budget_gated {
                        v["budget_gated"] = json!(true);
                        v["decision_id"] = json!(outcome.decision_id);
                    }
                    if let Some(qb) = outcome.quality_bar {
                        v["quality_bar"] = qb;
                    }
                    respond_json(stream, 200, &v)
                }
                Err(e) if e == "run cancelled by user" => {
                    let mut store = runtime.store.lock().unwrap();
                    store.push(&thread_id, "step", "Run cancelled by user.");
                    let mut v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    v["cancelled"] = json!(true);
                    respond_json(stream, 200, &v)
                }
                Err(e) if e.starts_with(tripwire::HALT_PREFIX) => {
                    respond_safety_stop(stream, runtime, &thread_id, &e)
                }
                Err(e) => {
                    runtime.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", path) if path.starts_with("/api/threads/") => {
            let id = path.trim_start_matches("/api/threads/");
            let store = runtime.store.lock().unwrap();
            match store.thread_json(id) {
                Some(v) => respond_json(stream, 200, &v),
                None => respond_err(stream, 404, "thread not found"),
            }
        }
        _ => return None,
    })
}

