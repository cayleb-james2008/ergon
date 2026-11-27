use super::*;

pub(super) fn scheduler_tick_ms() -> u64 {
    std::env::var("ERGON_SCHEDULER_TICK_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ms| ms.clamp(250, 300_000))
        .unwrap_or(15_000)
}


pub(super) fn latest_receipt_id_for_thread(person_data_dir: &Path, thread_id: &str) -> Option<String> {
    let chain = receipts::load_chain(person_data_dir);
    chain
        .iter()
        .rev()
        .find(|r| r.thread_id == thread_id)
        .map(|r| r.id.clone())
}

pub(super) struct RoutineExecResult {
    pub(super) http_status: u16,
    pub(super) routine: Value,
    pub(super) thread_id: String,
    pub(super) thread: Value,
    pub(super) run_record: scheduler::RoutineRunRecord,
    pub(super) error: Option<String>,
}

pub(super) fn execute_routine_run(
    state: &AppState,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    id: &str,
    trigger: &str,
) -> Option<RoutineExecResult> {
    execute_routine_run_with_inputs(state, person, runtime, id, trigger, None)
}

pub(super) fn substitute_trigger_input(text: &str, trigger_file: Option<&str>) -> String {
    let Some(tf) = trigger_file else {
        return text.to_string();
    };
    if text.contains("{{inputs.trigger_file}}") {
        text.replace("{{inputs.trigger_file}}", tf)
    } else {
        text.to_string()
    }
}

pub(super) fn build_routine_deliverable_pack(
    state: &AppState,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    name: &str,
    thread_id: &str,
) {
    let idx = person_idx(state, person, runtime);
    let (_, items) = idx.list_files(0, Some(15), "");
    let pack_files: Vec<String> = items
        .iter()
        .filter_map(|item| item["path"].as_str().map(|s| s.to_string()))
        .filter(|p| p != "Playbook Deliverable Pack.md")
        .collect();
    if let Ok((md, pack_sha, _, _)) = build_workspace_pack(
        &runtime.cfg.workspace_dir,
        &person.name,
        &format!("{name} — Deliverable Pack"),
        &pack_files,
        false,
    ) {
        if let Ok(full) = sandbox_path(&runtime.cfg.workspace_dir, "Playbook Deliverable Pack.md") {
            if !full
                .symlink_metadata()
                .is_ok_and(|m| m.file_type().is_symlink())
            {
                let _ = file_history::record_revision(
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    "Playbook Deliverable Pack.md",
                    "playbook",
                );
                if atomic_write_bytes(&full, md.as_bytes()).is_ok() {
                    state.search_cache.invalidate(&person.id);
                    runtime.store.lock().unwrap().push(
                        thread_id,
                        "step",
                        &format!(
                            "wrote Playbook Deliverable Pack.md ({} bytes, sha256:{pack_sha})",
                            md.len()
                        ),
                    );
                }
            }
        }
    }
}

pub(super) fn execute_routine_run_with_inputs(
    state: &AppState,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    id: &str,
    trigger: &str,
    trigger_file: Option<&str>,
) -> Option<RoutineExecResult> {
    let routine = routines::get_routine(&runtime.cfg.data_dir, id)?;
    let lock_key = format!("{}:{}", person.id, id);
    {
        let mut active = state
            .running_routines
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if !active.insert(lock_key.clone()) {
            let now_ms = records::now_millis();
            let rec = scheduler::new_run_record(
                id,
                routine["name"].as_str().unwrap_or("Routine"),
                now_ms,
                now_ms,
                "error",
                None,
                None,
                "Routine is already running.",
                trigger,
                Some("routine is already running".to_string()),
            );
            return Some(RoutineExecResult {
                http_status: 409,
                routine,
                thread_id: String::new(),
                thread: Value::Null,
                run_record: rec,
                error: Some("routine is already running".to_string()),
            });
        }
    }
    struct RunSlotGuard<'a>(&'a Mutex<std::collections::HashSet<String>>, String);
    impl Drop for RunSlotGuard<'_> {
        fn drop(&mut self) {
            let mut active = self.0.lock().unwrap_or_else(|p| p.into_inner());
            active.remove(&self.1);
        }
    }
    let _slot_guard = RunSlotGuard(&state.running_routines, lock_key);
    // Re-read after acquiring the per-routine slot so rapid scheduler ticks cannot
    // execute the same cadence window twice.
    let routine = routines::get_routine(&runtime.cfg.data_dir, id)?;
    if trigger == "scheduled" && !scheduler::is_routine_due(&routine, records::now_millis()) {
        return None;
    }

    if pipelines::has_running_pipeline_for_routine(&runtime.cfg.data_dir, id) {
        let now_ms = records::now_millis();
        let rec = scheduler::new_run_record(
            id,
            routine["name"].as_str().unwrap_or("Routine"),
            now_ms,
            now_ms,
            "error",
            None,
            None,
            "Routine is already running.",
            trigger,
            Some("routine is already running".to_string()),
        );
        return Some(RoutineExecResult {
            http_status: 409,
            routine,
            thread_id: String::new(),
            thread: Value::Null,
            run_record: rec,
            error: Some("routine is already running".to_string()),
        });
    }

    let name = routine["name"].as_str().unwrap_or("Routine").to_string();
    let raw_prompt = routine["prompt"].as_str().unwrap_or("").to_string();
    let prompt = {
        let sub = substitute_trigger_input(&raw_prompt, trigger_file);
        if let Some(tf) = trigger_file {
            if raw_prompt.contains("{{inputs.trigger_file}}") {
                sub
            } else if sub.trim().is_empty() {
                format!("Process trigger file: {tf}")
            } else {
                format!("{sub}\n\n[Trigger input trigger_file: {tf}]")
            }
        } else {
            sub
        }
    };
    let pack_deliverables = routine["pack_deliverables"].as_bool().unwrap_or(false);
    let mut steps: Vec<String> = routine["steps"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| substitute_trigger_input(s.trim(), trigger_file)))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if steps.is_empty() && !prompt.trim().is_empty() {
        steps.push(prompt.clone());
    }
    let started_ms = records::now_millis();
    let cfg = effective_cfg(state, runtime);
    let ctx = ToolCtx {
        library: state.library.clone(),
        root_data_dir: state.cfg.data_dir.clone(),
        person_id: person.id.clone(),
    };

    if let Some(v2_raw) = routine
        .get("steps_v2")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
    {
        if let Ok(steps_v2) = pipelines::parse_steps_v2(v2_raw) {
            let needs_llm = pipelines::steps_require_llm(&runtime.cfg.data_dir, &steps_v2);
            if needs_llm && !cfg.model_configured() {
                let thread_id = {
                    let mut store = runtime.store.lock().unwrap();
                    let tid = store.create_thread();
                    store.push(&tid, "user", &format!("{name} — {prompt}"));
                    store.push(&tid, "error", NO_KEY_ERROR);
                    tid
                };
                let finished_ms = records::now_millis();
                let updated =
                    routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "no_model")
                        .unwrap_or(routine);
                let rec = scheduler::new_run_record(
                    id,
                    &name,
                    started_ms,
                    finished_ms,
                    "error",
                    None,
                    Some(thread_id.clone()),
                    NO_KEY_ERROR,
                    trigger,
                    Some(NO_KEY_ERROR.to_string()),
                );
                let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
                let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                let idx = person_idx(state, person, runtime);
                let mut enriched = scheduler::enrich_routine_with_verification(
                    &updated,
                    finished_ms,
                    &hist,
                    &idx.chain,
                    idx.chain_valid,
                );
                if let Some(obj) = enriched.as_object_mut() {
                    obj.insert(
                        "offline_deterministic".to_string(),
                        json!(!pipelines::routine_value_requires_llm(
                            &runtime.cfg.data_dir,
                            &updated
                        )),
                    );
                }
                return Some(RoutineExecResult {
                    http_status: 503,
                    routine: enriched,
                    thread_id: thread_id.clone(),
                    thread: json!({"id": thread_id}),
                    run_record: rec,
                    error: Some(NO_KEY_ERROR.to_string()),
                });
            }

            let sync_exec = matches!(trigger, "manual" | "catchup");
            let async_exec = !sync_exec;
            let mut pipe_inputs = serde_json::Map::new();
            if let Some(tf) = trigger_file {
                pipe_inputs.insert("trigger_file".to_string(), json!(tf));
            }
            let mut pipe_body = json!({
                "routine_id": id,
                "inputs": Value::Object(pipe_inputs),
            });
            if async_exec {
                pipe_body["trigger"] = json!(trigger);
            }
            let env = pipelines::PipelineExecEnv {
                cfg: cfg.clone(),
                ctx: ctx.clone(),
                store: runtime.store.clone(),
                live_runs: state.live_runs.clone(),
                events: state.events.clone(),
                search_cache: state.search_cache.clone(),
            };
            let outcome = pipelines::create_and_start_pipeline(env, &pipe_body, async_exec);
            state.search_cache.invalidate(&person.id);
            let finished_ms = records::now_millis();
            match outcome {
                Ok(prun) if async_exec => {
                    let rec = scheduler::new_run_record(
                        id,
                        &name,
                        started_ms,
                        finished_ms,
                        "running",
                        None,
                        Some(prun.thread_id.clone()),
                        &format!("Started async pipeline {}", prun.run_id),
                        trigger,
                        None,
                    );
                    let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                    let enriched = scheduler::enrich_routine_json(&routine, finished_ms, &hist);
                    return Some(RoutineExecResult {
                        http_status: 202,
                        routine: enriched,
                        thread_id: prun.thread_id.clone(),
                        thread: json!({"id": prun.thread_id, "pipeline_run_id": prun.run_id}),
                        run_record: rec,
                        error: None,
                    });
                }
                Ok(prun) => {
                    if prun.status == "completed" {
                        if pack_deliverables {
                            build_routine_deliverable_pack(
                                state,
                                person,
                                runtime,
                                &name,
                                &prun.thread_id,
                            );
                        }
                        let summary: String = prun
                            .step_states
                            .iter()
                            .filter(|s| s.status == "ok" && !s.output.is_empty())
                            .map(|s| s.output.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !summary.is_empty() {
                            runtime
                                .store
                                .lock()
                                .unwrap()
                                .push(&prun.thread_id, "assistant", &summary);
                        }
                        let updated = routines::get_routine(&runtime.cfg.data_dir, id)
                            .unwrap_or(routine);
                        let mut receipt_id = latest_receipt_id_for_thread(
                            &runtime.cfg.data_dir,
                            &prun.thread_id,
                        );
                        if receipt_id.is_none() {
                            records::append_work(
                                &runtime.cfg,
                                &name,
                                if summary.is_empty() {
                                    "Completed deterministic routine."
                                } else {
                                    &summary
                                },
                                &[],
                                &prun.thread_id,
                                true,
                            );
                            receipt_id = latest_receipt_id_for_thread(
                                &runtime.cfg.data_dir,
                                &prun.thread_id,
                            );
                        }
                        state.search_cache.invalidate(&person.id);
                        let rec = scheduler::new_run_record(
                            id,
                            &name,
                            started_ms,
                            finished_ms,
                            "ok",
                            receipt_id.clone(),
                            Some(prun.thread_id.clone()),
                            &summary,
                            trigger,
                            None,
                        );
                        let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
                        let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                        let idx = person_idx(state, person, runtime);
                        let mut enriched = scheduler::enrich_routine_with_verification(
                            &updated,
                            finished_ms,
                            &hist,
                            &idx.chain,
                            idx.chain_valid,
                        );
                        if let Some(obj) = enriched.as_object_mut() {
                            obj.insert(
                                "offline_deterministic".to_string(),
                                json!(!pipelines::routine_value_requires_llm(
                                    &runtime.cfg.data_dir,
                                    &updated
                                )),
                            );
                        }
                        state.events.publish(
                            "routine_fired",
                            &person.id,
                            &rec.run_id,
                            &prun.thread_id,
                            json!({
                                "routine_id": id,
                                "routine_name": name,
                                "trigger": trigger,
                                "status": "ok",
                                "thread_id": prun.thread_id,
                                "pipeline_run_id": prun.run_id,
                                "receipt_id": receipt_id,
                                "duration_ms": rec.duration_ms,
                                "summary": truncate(&summary, 240),
                                "next_run_ms": enriched["next_run_ms"],
                                "ts_ms": finished_ms,
                            }),
                        );
                        let store = runtime.store.lock().unwrap();
                        let thread_v = store
                            .thread_json(&prun.thread_id)
                            .unwrap_or_else(|| json!({"id": prun.thread_id}));
                        return Some(RoutineExecResult {
                            http_status: 200,
                            routine: enriched,
                            thread_id: prun.thread_id,
                            thread: thread_v,
                            run_record: rec,
                            error: None,
                        });
                    } else if prun.status == "paused_checkpoint" {
                        let updated = routines::record_run(
                            &runtime.cfg.data_dir,
                            id,
                            &prun.thread_id,
                            "paused_checkpoint",
                        )
                        .unwrap_or(routine);
                        let summary = "Paused at human checkpoint.".to_string();
                        let rec = scheduler::new_run_record(
                            id,
                            &name,
                            started_ms,
                            finished_ms,
                            "paused_checkpoint",
                            None,
                            Some(prun.thread_id.clone()),
                            &summary,
                            trigger,
                            None,
                        );
                        let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
                        let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                        let enriched = scheduler::enrich_routine_json(&updated, finished_ms, &hist);
                        let store = runtime.store.lock().unwrap();
                        let thread_v = store
                            .thread_json(&prun.thread_id)
                            .unwrap_or_else(|| json!({"id": prun.thread_id}));
                        return Some(RoutineExecResult {
                            http_status: 200,
                            routine: enriched,
                            thread_id: prun.thread_id,
                            thread: thread_v,
                            run_record: rec,
                            error: None,
                        });
                    } else {
                        let err = prun
                            .error
                            .unwrap_or_else(|| format!("pipeline ended with status '{}'", prun.status));
                        let st_label = if prun.status == "cancelled" {
                            "cancelled"
                        } else {
                            "error"
                        };
                        let updated = routines::record_run(
                            &runtime.cfg.data_dir,
                            id,
                            &prun.thread_id,
                            st_label,
                        )
                        .unwrap_or(routine);
                        let rec = scheduler::new_run_record(
                            id,
                            &name,
                            started_ms,
                            finished_ms,
                            st_label,
                            None,
                            Some(prun.thread_id.clone()),
                            &err,
                            trigger,
                            Some(err.clone()),
                        );
                        let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
                        let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                        let enriched = scheduler::enrich_routine_json(&updated, finished_ms, &hist);
                        return Some(RoutineExecResult {
                            http_status: 500,
                            routine: enriched,
                            thread_id: prun.thread_id.clone(),
                            thread: json!({"id": prun.thread_id}),
                            run_record: rec,
                            error: Some(err),
                        });
                    }
                }
                Err((_, e)) => {
                    let thread_id = {
                        let mut store = runtime.store.lock().unwrap();
                        let tid = store.create_thread();
                        store.push(&tid, "user", &format!("{name} — {prompt}"));
                        store.push(&tid, "error", &e);
                        tid
                    };
                    let updated =
                        routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "error")
                            .unwrap_or(routine);
                    let rec = scheduler::new_run_record(
                        id,
                        &name,
                        started_ms,
                        finished_ms,
                        "error",
                        None,
                        Some(thread_id.clone()),
                        &e,
                        trigger,
                        Some(e.clone()),
                    );
                    let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
                    let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
                    let enriched = scheduler::enrich_routine_json(&updated, finished_ms, &hist);
                    return Some(RoutineExecResult {
                        http_status: 500,
                        routine: enriched,
                        thread_id: thread_id.clone(),
                        thread: json!({"id": thread_id}),
                        run_record: rec,
                        error: Some(e),
                    });
                }
            }
        }
    }

    let thread_id = {
        let mut store = runtime.store.lock().unwrap();
        let tid = store.create_thread();
        store.push(&tid, "user", &format!("{name} — {prompt}"));
        tid
    };

    if !cfg.model_configured() {
        runtime
            .store
            .lock()
            .unwrap()
            .push(&thread_id, "error", NO_KEY_ERROR);
        let finished_ms = records::now_millis();
        let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "no_model")
            .unwrap_or(routine);
        let rec = scheduler::new_run_record(
            id,
            &name,
            started_ms,
            finished_ms,
            "error",
            None,
            Some(thread_id.clone()),
            NO_KEY_ERROR,
            trigger,
            Some(NO_KEY_ERROR.to_string()),
        );
        let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
        let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
        let enriched = scheduler::enrich_routine_json(&updated, finished_ms, &hist);
        return Some(RoutineExecResult {
            http_status: 503,
            routine: enriched,
            thread_id: thread_id.clone(),
            thread: json!({"id": thread_id}),
            run_record: rec,
            error: Some(NO_KEY_ERROR.to_string()),
        });
    }

    let total_steps = steps.len();
    let mut history: Vec<Value> = Vec::new();
    let mut outcome: Result<AgentOutcome, String> = Err("routine has no steps".to_string());
    let mut combined_summaries: Vec<String> = Vec::new();
    for (idx, step_prompt) in steps.iter().enumerate() {
        let task_msg = if total_steps > 1 {
            format!(
                "[Playbook step {}/{total_steps} for \"{name}\"] {step_prompt}",
                idx + 1
            )
        } else {
            step_prompt.clone()
        };
        if idx > 0 {
            runtime.store.lock().unwrap().push(&thread_id, "user", &task_msg);
        }
        let step_res = run_agent_live(
            &cfg,
            &ctx,
            &runtime.store,
            &thread_id,
            &task_msg,
            &history,
            Some((&state.live_runs, &state.events, &person.id)),
        );
        match step_res {
            Ok(out) => {
                history.push(json!({"role": "user", "content": task_msg}));
                history.push(json!({"role": "assistant", "content": out.summary}));
                combined_summaries.push(out.summary.clone());
                outcome = Ok(out);
            }
            Err(e) => {
                outcome = Err(e);
                break;
            }
        }
    }
    if total_steps > 1 {
        if let Ok(ref mut out) = outcome {
            out.summary = combined_summaries.join("\n\n");
        }
    }
    state.search_cache.invalidate(&person.id);
    if outcome.is_ok() && pack_deliverables {
        let idx = person_idx(state, person, runtime);
        let (_, items) = idx.list_files(0, Some(15), "");
        let pack_files: Vec<String> = items
            .iter()
            .filter_map(|item| item["path"].as_str().map(|s| s.to_string()))
            .filter(|p| p != "Playbook Deliverable Pack.md")
            .collect();
        if let Ok((md, pack_sha, _, _)) = build_workspace_pack(
            &runtime.cfg.workspace_dir,
            &person.name,
            &format!("{name} — Deliverable Pack"),
            &pack_files,
            false,
        ) {
            if let Ok(full) = sandbox_path(&runtime.cfg.workspace_dir, "Playbook Deliverable Pack.md") {
                if !full.symlink_metadata().is_ok_and(|m| m.file_type().is_symlink()) {
                    let _ = file_history::record_revision(
                        &runtime.cfg.data_dir,
                        &runtime.cfg.workspace_dir,
                        "Playbook Deliverable Pack.md",
                        "playbook",
                    );
                    if atomic_write_bytes(&full, md.as_bytes()).is_ok() {
                        state.search_cache.invalidate(&person.id);
                        runtime.store.lock().unwrap().push(
                            &thread_id,
                            "step",
                            &format!(
                                "wrote Playbook Deliverable Pack.md ({} bytes, sha256:{pack_sha})",
                                md.len()
                            ),
                        );
                    }
                }
            }
        }
    }
    let finished_ms = records::now_millis();
    let receipt_id = latest_receipt_id_for_thread(&runtime.cfg.data_dir, &thread_id);

    match outcome {
        Ok(agent_out) => {
            let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "finished")
                .unwrap_or(routine);
            let rec = scheduler::new_run_record(
                id,
                &name,
                started_ms,
                finished_ms,
                "ok",
                receipt_id.clone(),
                Some(thread_id.clone()),
                &agent_out.summary,
                trigger,
                None,
            );
            let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
            let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
            let idx = person_idx(state, person, runtime);
            let enriched = scheduler::enrich_routine_with_verification(
                &updated,
                finished_ms,
                &hist,
                &idx.chain,
                idx.chain_valid,
            );
            state.events.publish(
                "routine_fired",
                &person.id,
                &rec.run_id,
                &thread_id,
                json!({
                    "routine_id": id,
                    "routine_name": name,
                    "trigger": trigger,
                    "status": "ok",
                    "thread_id": thread_id,
                    "receipt_id": receipt_id,
                    "duration_ms": rec.duration_ms,
                    "summary": truncate(&agent_out.summary, 240),
                    "next_run_ms": enriched["next_run_ms"],
                    "ts_ms": finished_ms,
                }),
            );
            let store = runtime.store.lock().unwrap();
            let thread_v = store
                .thread_json(&thread_id)
                .unwrap_or_else(|| json!({"id": thread_id}));
            Some(RoutineExecResult {
                http_status: 200,
                routine: enriched,
                thread_id,
                thread: thread_v,
                run_record: rec,
                error: None,
            })
        }
        Err(e) => {
            let is_cancel = e == "run cancelled by user";
            let st = if is_cancel { "cancelled" } else { "error" };
            runtime.store.lock().unwrap().push(&thread_id, "error", &e);
            let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, st)
                .unwrap_or(routine);
            let rec = scheduler::new_run_record(
                id,
                &name,
                started_ms,
                finished_ms,
                st,
                receipt_id.clone(),
                Some(thread_id.clone()),
                &e,
                trigger,
                Some(e.clone()),
            );
            let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
            let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
            let idx = person_idx(state, person, runtime);
            let enriched = scheduler::enrich_routine_with_verification(
                &updated,
                finished_ms,
                &hist,
                &idx.chain,
                idx.chain_valid,
            );
            state.events.publish(
                "routine_fired",
                &person.id,
                &rec.run_id,
                &thread_id,
                json!({
                    "routine_id": id,
                    "routine_name": name,
                    "trigger": trigger,
                    "status": st,
                    "thread_id": thread_id,
                    "receipt_id": receipt_id,
                    "duration_ms": rec.duration_ms,
                    "error": truncate(&e, 240),
                    "next_run_ms": enriched["next_run_ms"],
                    "ts_ms": finished_ms,
                }),
            );
            Some(RoutineExecResult {
                http_status: 500,
                routine: enriched,
                thread_id: thread_id.clone(),
                thread: json!({"id": thread_id}),
                run_record: rec,
                error: Some(e),
            })
        }
    }
}

pub(super) fn folder_watch_tick_ms() -> u64 {
    std::env::var("ERGON_FOLDER_WATCH_TICK_MS")
        .or_else(|_| std::env::var("ERGON_SCHEDULER_TICK_MS"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ms| ms.clamp(50, 300_000))
        .unwrap_or(2_000)
}

pub(super) fn fire_linked_trigger_target(
    state: &Arc<AppState>,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    routine_id: Option<&str>,
    target_kind: &str,
    trigger_file: &str,
    trigger_source: &str,
) -> Option<Value> {
    let rid = routine_id.map(str::trim).filter(|s| !s.is_empty())?;
    let routine = routines::get_routine(&runtime.cfg.data_dir, rid)?;
    let has_v2 = routine
        .get("steps_v2")
        .and_then(|v| v.as_array())
        .is_some_and(|a| !a.is_empty());
    if target_kind == "pipeline" || has_v2 {
        let ctx = ToolCtx {
            library: state.library.clone(),
            root_data_dir: state.cfg.data_dir.clone(),
            person_id: person.id.clone(),
        };
        let env = pipeline_exec_env(state, person, runtime, &ctx);
        let body = json!({
            "routine_id": rid,
            "inputs": {
                "trigger_file": trigger_file,
            }
        });
        match pipelines::create_and_start_pipeline(env, &body, true) {
            Ok(run) => Some(json!({
                "kind": "pipeline",
                "routine_id": rid,
                "pipeline_run_id": run.run_id,
                "thread_id": run.thread_id,
                "trigger": trigger_source,
            })),
            Err((_, e)) => Some(json!({
                "kind": "pipeline",
                "routine_id": rid,
                "error": e,
            })),
        }
    } else {
        let st_run = state.clone();
        let person_run = person.clone();
        let rt_run = runtime.clone();
        let rid_owned = rid.to_string();
        let tf_owned = trigger_file.to_string();
        let src_owned = trigger_source.to_string();
        std::thread::spawn(move || {
            let _ = execute_routine_run_with_inputs(
                &st_run,
                &person_run,
                &rt_run,
                &rid_owned,
                &src_owned,
                Some(&tf_owned),
            );
        });
        Some(json!({
            "kind": "routine",
            "routine_id": rid,
            "trigger": trigger_source,
        }))
    }
}

pub(super) fn start_folder_watch_thread(state: &Arc<AppState>) {
    let weak = Arc::downgrade(state);
    let tick_ms = folder_watch_tick_ms();
    std::thread::spawn(move || {
        let mut poll_states: HashMap<String, connectors::FolderWatchPollState> = HashMap::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(tick_ms));
            if shutdown_requested() {
                break;
            }
            let Some(st) = weak.upgrade() else {
                break;
            };
            let people: Vec<Person> = st
                .people
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .people
                .clone();
            for person in people {
                let p_cfg = st.cfg.person_config(&person);
                if !connectors::connectors_path(&p_cfg.data_dir).exists() {
                    continue;
                }
                let rt = runtime_for(&st, &person);
                let poll_state = poll_states.entry(person.id.clone()).or_default();
                let ingested =
                    connectors::poll_folder_watch_once(&rt.cfg, &st.cfg.data_dir, poll_state);
                for ev in ingested {
                    st.search_cache.invalidate(&person.id);
                    let _ = fire_linked_trigger_target(
                        &st,
                        &person,
                        &rt,
                        ev.routine_id.as_deref(),
                        &ev.target_kind,
                        &ev.trigger_file,
                        "folder_watch",
                    );
                }
            }
        }
    });
}

pub(super) fn imap_poll_tick_ms() -> u64 {
    std::env::var("ERGON_IMAP_POLL_TICK_MS")
        .or_else(|_| std::env::var("ERGON_SCHEDULER_TICK_MS"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ms| ms.clamp(50, 300_000))
        .unwrap_or(5_000)
}

pub(super) fn start_imap_poll_thread(state: &Arc<AppState>) {
    let weak = Arc::downgrade(state);
    let tick_ms = imap_poll_tick_ms();
    std::thread::spawn(move || {
        let mut poll_states: HashMap<String, connectors::ImapPollState> = HashMap::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(tick_ms));
            if shutdown_requested() {
                break;
            }
            let Some(st) = weak.upgrade() else {
                break;
            };
            let people: Vec<Person> = st
                .people
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .people
                .clone();
            for person in people {
                let p_cfg = st.cfg.person_config(&person);
                if !connectors::connectors_path(&p_cfg.data_dir).exists() {
                    continue;
                }
                let rt = runtime_for(&st, &person);
                let poll_state = poll_states.entry(person.id.clone()).or_default();
                let ingested = connectors::poll_imap_once(&rt.cfg, poll_state);
                for ev in ingested {
                    st.search_cache.invalidate(&person.id);
                    let _ = fire_linked_trigger_target(
                        &st,
                        &person,
                        &rt,
                        ev.routine_id.as_deref(),
                        &ev.target_kind,
                        &ev.trigger_file,
                        "imap_poll",
                    );
                }
            }
        }
    });
}

pub(super) fn start_scheduler_thread(state: &Arc<AppState>) {
    let weak = Arc::downgrade(state);
    let tick_ms = scheduler_tick_ms();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(tick_ms));
            if shutdown_requested() {
                break;
            }
            let Some(st) = weak.upgrade() else {
                break;
            };
            let people: Vec<Person> = st
                .people
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .people
                .clone();
            let now_ms = records::now_millis();
            for person in people {
                let p_cfg = st.cfg.person_config(&person);
                if !routines::routines_path(&p_cfg.data_dir).exists() {
                    continue;
                }
                let rt = runtime_for(&st, &person);
                let eff = effective_cfg(&st, &rt);
                let model_ok = eff.model_configured();
                let list = routines::list_routines(&rt.cfg.data_dir);
                let due = scheduler::find_due_routines(&list, now_ms);
                for r in due {
                    let Some(rid) = r["id"].as_str().map(|s| s.to_string()) else {
                        continue;
                    };
                    // Deterministic zero-LLM `steps_v2` routines run offline without a model key;
                    // LLM-requiring routines wait until a model is configured instead of spamming errors.
                    if !model_ok && pipelines::routine_value_requires_llm(&rt.cfg.data_dir, &r) {
                        continue;
                    }
                    if pipelines::has_running_pipeline_for_routine(&rt.cfg.data_dir, &rid) {
                        continue;
                    }
                    let lock_key = format!("{}:{}", person.id, rid);
                    let already = st
                        .running_routines
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .contains(&lock_key);
                    if already {
                        continue;
                    }
                    let st_run = st.clone();
                    let person_run = person.clone();
                    let rt_run = rt.clone();
                    std::thread::spawn(move || {
                        let _ = execute_routine_run(&st_run, &person_run, &rt_run, &rid, "scheduled");
                    });
                }
            }
        }
    });
}

pub(super) fn backup_scheduler_tick_ms() -> u64 {
    std::env::var("ERGON_BACKUP_TICK_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ms| ms.clamp(250, 3_600_000))
        .unwrap_or(60_000)
}

pub(super) fn is_backup_scheduler_disabled() -> bool {
    std::env::var("ERGON_DISABLE_BACKUP_SCHEDULER")
        .map(|v| {
            let low = v.trim().to_ascii_lowercase();
            matches!(low.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

pub(super) fn start_backup_scheduler_thread(state: &Arc<AppState>) {
    if is_backup_scheduler_disabled() {
        return;
    }
    let weak = Arc::downgrade(state);
    let tick_ms = backup_scheduler_tick_ms();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(tick_ms));
            let Some(st) = weak.upgrade() else {
                break;
            };
            let cfg = backup::load_backup_config(&st.cfg.data_dir);
            if !cfg.enabled {
                continue;
            }
            let rt_state = backup::load_backup_state(&st.cfg.data_dir);
            let now_ms = records::now_millis();
            if !backup::is_scheduled_backup_due(&cfg, &rt_state, now_ms) {
                continue;
            }
            let Ok(_guard) = st.backup_lock.try_lock() else {
                continue;
            };
            match backup::run_backup_with_config(&st.cfg.data_dir, &cfg, None) {
                Ok(report) => {
                    eprintln!(
                        "ergon: scheduled backup complete: {} ({} files, {} bytes, sha256:{})",
                        report.filename,
                        report.file_count,
                        report.zip_bytes,
                        &report.sha256[..12.min(report.sha256.len())]
                    );
                }
                Err(err) => {
                    eprintln!("ergon: scheduled backup failed: {err}");
                }
            }
        }
    });
}

pub(super) fn reload_state_after_restore(state: &AppState) {
    let people = PeopleState::load(&state.cfg);
    {
        let mut p_lock = state.people.write().unwrap_or_else(|p| p.into_inner());
        *p_lock = people;
    }
    {
        let mut rt_lock = state.runtimes.write().unwrap_or_else(|p| p.into_inner());
        rt_lock.clear();
    }
    {
        let mut lib_lock = state.library.lock().unwrap_or_else(|p| p.into_inner());
        *lib_lock = library::Library::load(&state.cfg.data_dir);
    }
    state.search_cache.invalidate_all();
    if let Some(saved) = crate::load_saved_model_config(&state.cfg.data_dir) {
        let mut mc = state.model_cfg.lock().unwrap_or_else(|p| p.into_inner());
        mc.model_base_url = saved.model_base_url;
        mc.model_id = saved.model_id;
        mc.api_key = saved.api_key;
        mc.auto_detected = false;
    }
}

