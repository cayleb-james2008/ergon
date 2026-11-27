use super::*;

/// Routes search, receipts, skills, routines, decisions, offers, brief, today, and graph.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_route_knowledge_and_routines(
    stream: &mut ConnStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    principal: Option<&auth::AuthPrincipal>,
    auth_enforced: bool,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/search") => {
            let (q, scope, mode, limit, path_prefix, mut client_id, mut engagement_id) = if req.query.is_empty() {
                (
                    String::new(),
                    String::from("all"),
                    String::from("hybrid"),
                    30usize,
                    String::new(),
                    None,
                    None,
                )
            } else {
                (
                    query_param(&req.query, "q").unwrap_or_default(),
                    query_param(&req.query, "scope")
                        .map(|s| s.to_ascii_lowercase())
                        .filter(|s| matches!(s.as_str(), "all" | "receipts" | "files" | "memory"))
                        .unwrap_or_else(|| "all".to_string()),
                    query_param(&req.query, "mode")
                        .map(|s| s.to_ascii_lowercase())
                        .filter(|s| matches!(s.as_str(), "hybrid" | "bm25" | "exact"))
                        .unwrap_or_else(|| "hybrid".to_string()),
                    query_param(&req.query, "limit")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(30),
                    query_param(&req.query, "path_prefix").unwrap_or_default(),
                    query_param(&req.query, "client_id")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                    query_param(&req.query, "engagement_id")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                )
            };
            if client_id.is_none() && engagement_id.is_none() {
                let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
                if policy.isolate_active_engagement {
                    if let Some((eng, _)) =
                        records::active_engagement_for(&state.cfg.data_dir, &person.id)
                    {
                        engagement_id = Some(eng.id);
                        client_id = Some(eng.client_id);
                    }
                }
            }
            let idx = person_idx(state, person, runtime);
            respond_json_with(stream, 200, |buf| {
                idx.write_search_json_scoped(
                    buf,
                    &q,
                    &scope,
                    &mode,
                    limit,
                    &path_prefix,
                    client_id.as_deref(),
                    engagement_id.as_deref(),
                );
            })
        }
        ("POST", "/api/vault/reindex") => {
            let t0 = std::time::Instant::now();
            let idx = state.search_cache.reindex(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            search::SearchCache::global().invalidate(&runtime.cfg.data_dir.to_string_lossy());
            let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
            respond_json(
                stream,
                200,
                &json!({
                    "ok": true,
                    "indexed_files": idx.files.len(),
                    "total_chunks": idx.total_chunks(),
                    "indexed_bytes": idx.indexed_bytes,
                    "semantic_enabled": idx.semantic_enabled,
                    "elapsed_ms": elapsed_ms,
                }),
            )
        }
        ("GET", "/api/receipts") => {
            let (offset, limit, q, status) = if req.query.is_empty() {
                (0usize, 50usize, String::new(), String::from("all"))
            } else {
                (
                    query_param(&req.query, "offset")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0),
                    query_param(&req.query, "limit")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(50),
                    query_param(&req.query, "q").unwrap_or_default(),
                    query_param(&req.query, "status")
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_else(|| "all".to_string()),
                )
            };
            let idx = person_idx(state, person, runtime);
            respond_json_with(stream, 200, |buf| {
                idx.write_receipts_json(buf, offset, limit, &q, &status);
            })
        }
        ("GET", "/api/receipts/compare") => {
            let a_id = query_param(&req.query, "a").unwrap_or_default();
            let b_id = query_param(&req.query, "b").unwrap_or_default();
            if a_id.trim().is_empty() || b_id.trim().is_empty() {
                return Some(respond_err(stream, 400, "query parameters a and b are required"));
            }
            let idx = person_idx(state, person, runtime);
            let Some(ra) = receipts::find_receipt(&idx.chain, a_id.trim()) else {
                return Some(respond_err(stream, 404, "receipt a not found"));
            };
            let Some(rb) = receipts::find_receipt(&idx.chain, b_id.trim()) else {
                return Some(respond_err(stream, 404, "receipt b not found"));
            };
            let (payload, _) = build_receipt_compare(ra, rb);
            respond_json(stream, 200, &payload)
        }
        ("GET", "/api/receipts/verify") => {
            let idx = person_idx(state, person, runtime);
            respond_json(
                stream,
                200,
                &json!({
                    "total": idx.chain.len(),
                    "valid": idx.chain_valid,
                    "head_hash": idx.head_hash,
                }),
            )
        }
        ("GET", path) if path.starts_with("/api/receipts/") && path.ends_with("/export") => {
            let raw_id = path
                .strip_prefix("/api/receipts/")
                .and_then(|r| r.strip_suffix("/export"))
                .unwrap_or("");
            let id = percent_decode(raw_id);
            let fmt = query_param(&req.query, "format")
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "json".to_string());
            if fmt != "json" && fmt != "md" {
                return Some(respond_err(stream, 400, "format must be md or json"));
            }
            let idx = person_idx(state, person, runtime);
            let Some(rcpt) = receipts::find_receipt(&idx.chain, &id) else {
                return Some(respond_err(stream, 404, "receipt not found"));
            };
            let safe_id: String = rcpt
                .id
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') { c } else { '_' })
                .collect();
            if fmt == "md" {
                let md = receipts::export_receipt_markdown(
                    rcpt,
                    &runtime.cfg.workspace_dir,
                    idx.chain.len(),
                    idx.chain_valid,
                );
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"receipt-{safe_id}.md\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                respond_bytes(stream, 200, "text/markdown; charset=utf-8", md.as_bytes(), &headers)
            } else {
                let body = receipts::export_receipt_json(
                    rcpt,
                    &runtime.cfg.workspace_dir,
                    idx.chain.len(),
                    idx.chain_valid,
                );
                let bytes = serde_json::to_vec_pretty(&body).unwrap_or_else(|_| body.to_string().into_bytes());
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"receipt-{safe_id}.json\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                respond_bytes(stream, 200, "application/json; charset=utf-8", &bytes, &headers)
            }
        }
        ("GET", "/api/skills") => {
            respond_json(stream, 200, &skill_catalog::catalog_json())
        }
        ("POST", "/api/skills/test") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let res = skill_catalog::handle_skill_test_request(&parsed);
            respond_json(stream, 200, &res)
        }
        ("GET", path) if path.starts_with("/api/skills/") => {
            let id = path.trim_start_matches("/api/skills/").trim();
            match skill_catalog::get_skill(id) {
                Some(skill) => respond_json(stream, 200, &skill.to_json()),
                None => respond_err(stream, 404, "skill not found"),
            }
        }
        ("GET", "/api/routines") => {
            let now_ms = records::now_millis();
            let raw = routines::list_routines(&runtime.cfg.data_dir);
            let history = scheduler::list_run_history(&runtime.cfg.data_dir, None, 500);
            let idx = person_idx(state, person, runtime);
            let enriched: Vec<Value> = raw
                .iter()
                .map(|r| {
                    let mut v = scheduler::enrich_routine_with_verification(
                        r,
                        now_ms,
                        &history,
                        &idx.chain,
                        idx.chain_valid,
                    );
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert(
                            "offline_deterministic".to_string(),
                            json!(!pipelines::routine_value_requires_llm(&runtime.cfg.data_dir, r)),
                        );
                    }
                    v
                })
                .collect();
            let tick_secs = (scheduler_tick_ms() / 1000).max(1);
            let sched_status = scheduler::scheduler_status_json(
                state.scheduler_enabled,
                tick_secs,
                &raw,
                &history,
                now_ms,
            );
            respond_json(
                stream,
                200,
                &json!({
                    "routines": enriched,
                    "scheduler": sched_status,
                }),
            )
        }
        ("POST", "/api/routines") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("");
            let prompt = parsed["prompt"].as_str().unwrap_or("");
            let schedule = parsed["schedule"].as_str().unwrap_or("manual");
            let skill_id = parsed["skill_id"].as_str();
            let enabled = parsed["enabled"].as_bool().unwrap_or(true);
            let interval_secs = if parsed.get("interval_secs").is_some() && !parsed["interval_secs"].is_null() {
                match parsed["interval_secs"].as_u64() {
                    Some(v) => Some(v),
                    None => {
                        return Some(respond_err(stream, 400, "interval_secs must be a positive integer"))}
                }
            } else {
                None
            };
            let playbook = routines::PlaybookOptions {
                steps: parsed["steps"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                expected_inputs: parsed["expected_inputs"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                pack_deliverables: parsed["pack_deliverables"].as_bool().unwrap_or(false),
            };
            let steps_v2 = if parsed.get("steps_v2").is_some()
                && !parsed["steps_v2"].is_null()
                && !parsed["steps_v2"].as_array().is_some_and(|a| a.is_empty())
            {
                match pipelines::parse_steps_v2(&parsed["steps_v2"]) {
                    Ok(v) => v,
                    Err(e) => return Some(respond_err(stream, 400, &e)),
                }
            } else {
                Vec::new()
            };
            let tz_offset_mins = if parsed.get("tz_offset_mins").is_some()
                && !parsed["tz_offset_mins"].is_null()
            {
                match parsed["tz_offset_mins"].as_i64() {
                    Some(v) if (-720..=840).contains(&v) => Some(v as i32),
                    _ => {
                        return Some(respond_err(
                            stream,
                            400,
                            "tz_offset_mins must be an integer between -720 and 840",
                        ))
                    }
                }
            } else {
                None
            };
            let tz_val = parsed.get("tz").or_else(|| parsed.get("timezone"));
            let tz = match tz_val {
                None | Some(Value::Null) => None,
                Some(Value::String(s)) => Some(s.as_str()),
                Some(_) => return Some(respond_err(stream, 400, "tz must be a string")),
            };
            match routines::create_pipeline_routine_with_tz(
                &runtime.cfg.data_dir,
                name,
                prompt,
                schedule,
                interval_secs,
                tz_offset_mins,
                tz,
                skill_id,
                enabled,
                &playbook,
                &steps_v2,
            ) {
                Ok(routine) => {
                    let now_ms = records::now_millis();
                    let mut enriched = scheduler::enrich_routine_json(&routine, now_ms, &[]);
                    if let Some(obj) = enriched.as_object_mut() {
                        obj.insert(
                            "offline_deterministic".to_string(),
                            json!(!pipelines::routine_value_requires_llm(
                                &runtime.cfg.data_dir,
                                &routine
                            )),
                        );
                    }
                    respond_json(stream, 201, &json!({"routine": enriched}))
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", path) if path.starts_with("/api/routines/") && path.ends_with("/history") => {
            let id = path
                .strip_prefix("/api/routines/")
                .and_then(|r| r.strip_suffix("/history"))
                .unwrap_or("")
                .trim()
                .to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_err(stream, 400, "invalid routine id"));
            }
            let limit = query_param(&req.query, "limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50)
                .clamp(1, 500);
            let runs = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), limit);
            if runs.is_empty() && routines::get_routine(&runtime.cfg.data_dir, &id).is_none() {
                return Some(respond_err(stream, 404, "routine not found"));
            }
            let items: Vec<Value> = runs.iter().map(|r| r.to_json()).collect();
            respond_json(
                stream,
                200,
                &json!({
                    "routine_id": id,
                    "count": items.len(),
                    "runs": items,
                    "history": items,
                }),
            )
        }
        ("PATCH", path) if path.starts_with("/api/routines/") => {
            let id = path.trim_start_matches("/api/routines/").trim().to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_err(stream, 400, "invalid routine id"));
            }
            let parsed: Value = match serde_json::from_slice::<Value>(&req.body) {
                Ok(v) if v.is_object() => v,
                _ => {
                    return Some(respond_err(stream, 400, "PATCH body must be a JSON object"))}
            };
            match routines::patch_routine(&runtime.cfg.data_dir, &id, &parsed) {
                Ok(Some(routine)) => {
                    let now_ms = records::now_millis();
                    let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), 20);
                    let idx = person_idx(state, person, runtime);
                    let mut enriched = scheduler::enrich_routine_with_verification(
                        &routine,
                        now_ms,
                        &hist,
                        &idx.chain,
                        idx.chain_valid,
                    );
                    if let Some(obj) = enriched.as_object_mut() {
                        obj.insert(
                            "offline_deterministic".to_string(),
                            json!(!pipelines::routine_value_requires_llm(
                                &runtime.cfg.data_dir,
                                &routine
                            )),
                        );
                    }
                    respond_json(stream, 200, &json!({"routine": enriched}))
                }
                Ok(None) => respond_err(stream, 404, "routine not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("DELETE", path) if path.starts_with("/api/routines/") => {
            let id = path.trim_start_matches("/api/routines/").trim().to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_err(stream, 400, "invalid routine id"));
            }
            match routines::delete_routine(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({"deleted": id})),
                Ok(false) => respond_err(stream, 404, "routine not found"),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", "/api/scheduler/catchup") => {
            let now_ms = records::now_millis();
            let raw = routines::list_routines(&runtime.cfg.data_dir);
            let due = scheduler::find_due_routines(&raw, now_ms);
            let mut results: Vec<Value> = Vec::new();
            let mut caught_up_count = 0usize;
            let mut failed_count = 0usize;
            for rt in due.iter().take(10) {
                let rid = rt["id"].as_str().unwrap_or("");
                if rid.is_empty() {
                    continue;
                }
                let missed = scheduler::count_missed_windows(rt, now_ms);
                if let Some(exec) = execute_routine_run(state, person, runtime, rid, "catchup") {
                    let ok = exec.error.is_none();
                    if ok {
                        caught_up_count += 1;
                    } else {
                        failed_count += 1;
                    }
                    results.push(json!({
                        "routine_id": rid,
                        "ok": ok,
                        "missed_windows_cleared": if ok { missed } else { 0 },
                        "thread_id": exec.thread_id,
                        "error": exec.error,
                    }));
                }
            }
            let updated_now = records::now_millis();
            let updated_raw = routines::list_routines(&runtime.cfg.data_dir);
            let history = scheduler::list_run_history(&runtime.cfg.data_dir, None, 500);
            let idx = person_idx(state, person, runtime);
            let enriched: Vec<Value> = updated_raw
                .iter()
                .map(|r| {
                    scheduler::enrich_routine_with_verification(
                        r,
                        updated_now,
                        &history,
                        &idx.chain,
                        idx.chain_valid,
                    )
                })
                .collect();
            let tick_secs = (scheduler_tick_ms() / 1000).max(1);
            let sched_status = scheduler::scheduler_status_json(
                state.scheduler_enabled,
                tick_secs,
                &updated_raw,
                &history,
                updated_now,
            );
            respond_json(
                stream,
                200,
                &json!({
                    "total_due": due.len(),
                    "caught_up_count": caught_up_count,
                    "failed_count": failed_count,
                    "results": results,
                    "routines": enriched,
                    "scheduler": sched_status,
                }),
            )
        }
        ("POST", path)
            if (path.starts_with("/api/routines/") || path.starts_with("/api/playbooks/"))
                && path.ends_with("/catchup") =>
        {
            let prefix = if path.starts_with("/api/playbooks/") {
                "/api/playbooks/"
            } else {
                "/api/routines/"
            };
            let raw_id = path
                .strip_prefix(prefix)
                .and_then(|r| r.strip_suffix("/catchup"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid routine id"));
            }
            let Some(rt) = routines::get_routine(&runtime.cfg.data_dir, &id) else {
                return Some(respond_err(stream, 404, "routine not found"));
            };
            let now_ms = records::now_millis();
            let missed = scheduler::count_missed_windows(&rt, now_ms);
            let force = query_param(&req.query, "force")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if missed == 0 && !force {
                let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), 20);
                let idx = person_idx(state, person, runtime);
                let enriched = scheduler::enrich_routine_with_verification(
                    &rt,
                    now_ms,
                    &hist,
                    &idx.chain,
                    idx.chain_valid,
                );
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "routine": enriched,
                        "caught_up": false,
                        "missed_windows": 0,
                    }),
                ));
            }
            let Some(exec) = execute_routine_run(state, person, runtime, &id, "catchup") else {
                return Some(respond_err(stream, 404, "routine not found"));
            };
            if let Some(err) = exec.error {
                respond_json(
                    stream,
                    exec.http_status,
                    &json!({
                        "error": err,
                        "caught_up": false,
                        "missed_windows": missed,
                        "thread_id": exec.thread_id,
                        "routine": exec.routine,
                        "run": exec.run_record.to_json(),
                    }),
                )
            } else {
                respond_json(
                    stream,
                    200,
                    &json!({
                        "caught_up": true,
                        "missed_windows_cleared": missed,
                        "routine": exec.routine,
                        "thread": exec.thread,
                        "thread_id": exec.thread_id,
                        "run": exec.run_record.to_json(),
                    }),
                )
            }
        }
        ("POST", path) if path.starts_with("/api/routines/") && path.ends_with("/run") => {
            let id = path
                .strip_prefix("/api/routines/")
                .and_then(|r| r.strip_suffix("/run"))
                .unwrap_or("")
                .trim()
                .to_string();
            let Some(exec) = execute_routine_run(state, person, runtime, &id, "manual") else {
                return Some(respond_err(stream, 404, "routine not found"));
            };
            if let Some(err) = exec.error {
                respond_json(
                    stream,
                    exec.http_status,
                    &json!({
                        "error": err,
                        "thread_id": exec.thread_id,
                        "routine": exec.routine,
                        "run": exec.run_record.to_json(),
                    }),
                )
            } else {
                respond_json(
                    stream,
                    200,
                    &json!({
                        "routine": exec.routine,
                        "thread": exec.thread,
                        "thread_id": exec.thread_id,
                        "run": exec.run_record.to_json(),
                    }),
                )
            }
        }
        ("GET", path) if path.starts_with("/api/playbooks/") && path.ends_with("/export") => {
            let raw_id = path
                .strip_prefix("/api/playbooks/")
                .and_then(|r| r.strip_suffix("/export"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid playbook id"));
            }
            let Some((pack, digest, filename)) =
                routines::export_playbook_pack(&runtime.cfg.data_dir, &id)
            else {
                return Some(respond_err(stream, 404, "playbook not found"));
            };
            let body = serde_json::to_vec_pretty(&pack).unwrap_or_default();
            let headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                ("X-Ergon-Playbook-SHA256", digest),
                (
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                ),
            ];
            respond_bytes(
                stream,
                200,
                "application/json; charset=utf-8",
                &body,
                &headers,
            )
        }
        ("POST", "/api/playbooks/import") => {
            match routines::import_playbook_pack(&runtime.cfg.data_dir, &req.body) {
                Ok((routine, digest)) => {
                    let now_ms = records::now_millis();
                    let enriched = scheduler::enrich_routine_json(&routine, now_ms, &[]);
                    respond_json(
                        stream,
                        201,
                        &json!({
                            "routine": enriched,
                            "sha256": digest,
                            "verified": true,
                        }),
                    )
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/playbooks/starters") => {
            respond_json(
                stream,
                200,
                &json!({"starters": skill_catalog::starter_playbooks_json()}),
            )
        }
        ("POST", "/api/playbooks/starters/launch") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"]
                .as_str()
                .or_else(|| parsed["starter_id"].as_str())
                .unwrap_or("");
            let run_now = parsed["run_now"].as_bool().unwrap_or(true);
            match skill_catalog::launch_starter_playbook(&runtime.cfg, id, run_now) {
                Ok(res) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(stream, 201, &res)
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", path)
            if (path.starts_with("/api/playbooks/") || path.starts_with("/api/routines/"))
                && path.ends_with("/preview") =>
        {
            let prefix = if path.starts_with("/api/playbooks/") {
                "/api/playbooks/"
            } else {
                "/api/routines/"
            };
            let raw_id = path
                .strip_prefix(prefix)
                .and_then(|r| r.strip_suffix("/preview"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid routine id"));
            }
            let idx = person_idx(state, person, runtime);
            let (_, ws_files) = idx.list_files(0, Some(500), "");
            let ws_paths: Vec<String> = ws_files
                .into_iter()
                .filter_map(|v| v["path"].as_str().map(str::to_string))
                .collect();
            match routines::preview_playbook_routine(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &ws_paths,
                &id,
            ) {
                Some(preview) => respond_json(stream, 200, &preview),
                None => respond_err(stream, 404, "routine not found"),
            }
        }
        ("GET", "/api/connectors") => {
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            respond_json(
                stream,
                200,
                &connectors::list_connectors_json(&runtime.cfg.data_dir, &policy),
            )
        }
        ("POST", "/api/connectors/triggers") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_folder_watch = parsed["kind"]
                .as_str()
                .map(|s| s.trim().eq_ignore_ascii_case("folder_watch"))
                .unwrap_or(false)
                && parsed["rotate"].as_bool() != Some(true);
            if is_folder_watch
                && (state.gate_token.is_some()
                    || !host_allowed(req.host.as_deref().unwrap_or(""), &[]))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "folder_watch triggers can only be configured from this machine (loopback-only)"}),
                ));
            }
            let is_rotate = parsed["rotate"].as_bool() == Some(true);
            match connectors::create_trigger(&runtime.cfg.data_dir, &root_cfg.data_dir, &parsed) {
                Ok(v) => respond_json(stream, if is_rotate { 200 } else { 201 }, &v),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", path)
            if path.starts_with("/api/connectors/triggers/") && path.ends_with("/rotate") =>
        {
            let raw_id = path
                .strip_prefix("/api/connectors/triggers/")
                .and_then(|r| r.strip_suffix("/rotate"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid trigger id"));
            }
            match connectors::rotate_webhook_secret(&runtime.cfg.data_dir, &id) {
                Ok(v) => respond_json(stream, 200, &v),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path)
            if path == "/api/connectors/triggers"
                || path.starts_with("/api/connectors/triggers/") =>
        {
            let path_id = path
                .strip_prefix("/api/connectors/triggers/")
                .map(|s| percent_decode(s).trim().to_string())
                .filter(|s| !s.is_empty());
            let body_id = if req.body.is_empty() {
                None
            } else {
                serde_json::from_slice::<Value>(&req.body)
                    .ok()
                    .and_then(|v| v["id"].as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
            };
            let id = path_id
                .or_else(|| query_param(&req.query, "id").filter(|s| !s.is_empty()))
                .or(body_id)
                .unwrap_or_default();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "trigger id is required"));
            }
            match connectors::delete_trigger(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({"deleted": id})),
                Ok(false) => respond_err(stream, 404, "trigger not found"),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", "/api/connectors/outbound") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            match connectors::upsert_outbound_connector(&runtime.cfg.data_dir, &parsed, &policy) {
                Ok((connector, created)) => respond_json(
                    stream,
                    if created { 201 } else { 200 },
                    &json!({"connector": connector}),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path)
            if path == "/api/connectors/outbound"
                || path.starts_with("/api/connectors/outbound/") =>
        {
            let path_id = path
                .strip_prefix("/api/connectors/outbound/")
                .map(|s| percent_decode(s).trim().to_string())
                .filter(|s| !s.is_empty());
            let body_id = if req.body.is_empty() {
                None
            } else {
                serde_json::from_slice::<Value>(&req.body)
                    .ok()
                    .and_then(|v| v["id"].as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
            };
            let id = path_id
                .or_else(|| query_param(&req.query, "id").filter(|s| !s.is_empty()))
                .or(body_id)
                .unwrap_or_default();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "connector id is required"));
            }
            match connectors::delete_outbound_connector(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({"deleted": id})),
                Ok(false) => respond_err(stream, 404, "connector not found"),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", path)
            if path.starts_with("/api/connectors/outbound/") && path.ends_with("/test") =>
        {
            let raw_id = path
                .strip_prefix("/api/connectors/outbound/")
                .and_then(|r| r.strip_suffix("/test"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid connector id"));
            }
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                Value::Null
            } else {
                match serde_json::from_slice::<Value>(&req.body) {
                    Ok(v) => v.get("payload").cloned().unwrap_or(v),
                    Err(_) => {
                        return Some(respond_err(stream, 400, "request body is not valid JSON"))
                    }
                }
            };
            let (status, payload) =
                connectors::test_outbound_connector(&runtime.cfg, &id, &parsed);
            respond_json(stream, status, &payload)
        }
        ("GET", "/api/mcp/servers") => {
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            respond_json(
                stream,
                200,
                &mcp::list_mcp_servers_json(&runtime.cfg.data_dir, &policy),
            )
        }
        ("POST", "/api/mcp/servers") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_stdio = parsed["transport"]
                .as_str()
                .unwrap_or("stdio")
                .trim()
                .eq_ignore_ascii_case("stdio");
            if is_stdio
                && (state.gate_token.is_some()
                    || !host_allowed(req.host.as_deref().unwrap_or(""), &[]))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "stdio MCP servers can only be configured from this machine (loopback-only)"}),
                ));
            }
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            match mcp::upsert_mcp_server(&runtime.cfg.data_dir, &parsed, &policy) {
                Ok((server, created)) => respond_json(
                    stream,
                    if created { 201 } else { 200 },
                    &json!({"server": server}),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", path)
            if path.starts_with("/api/mcp/servers/") && path.ends_with("/discover") =>
        {
            let raw_id = path
                .strip_prefix("/api/mcp/servers/")
                .and_then(|r| r.strip_suffix("/discover"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "invalid MCP server id"));
            }
            match mcp::discover_mcp_server(&runtime.cfg, &id) {
                Ok(v) => respond_json(stream, 200, &v),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path)
            if path == "/api/mcp/servers" || path.starts_with("/api/mcp/servers/") =>
        {
            let path_id = path
                .strip_prefix("/api/mcp/servers/")
                .map(|s| percent_decode(s).trim().to_string())
                .filter(|s| !s.is_empty());
            let body_id = if req.body.is_empty() {
                None
            } else {
                serde_json::from_slice::<Value>(&req.body)
                    .ok()
                    .and_then(|v| v["id"].as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
            };
            let id = path_id
                .or_else(|| query_param(&req.query, "id").filter(|s| !s.is_empty()))
                .or(body_id)
                .unwrap_or_default();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "MCP server id is required"));
            }
            match mcp::delete_mcp_server(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({"deleted": id})),
                Ok(false) => respond_err(stream, 404, "MCP server not found"),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", "/api/mcp/inbound/config") => {
            if state.gate_token.is_some()
                || !host_allowed(req.host.as_deref().unwrap_or(""), &[])
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "MCP inbound settings can only be changed from this machine (loopback-only)"}),
                ));
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let enabled = parsed["enabled"].as_bool().unwrap_or(false);
            match mcp::set_inbound_enabled(&runtime.cfg.data_dir, enabled) {
                Ok(v) => respond_json(stream, 200, &v),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", "/api/mcp/inbound") => {
            let token_valid = mcp_inbound_bearer_valid(req.authorization.as_deref());
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                principal.map(|p| p.role).unwrap_or(auth::Role::Admin)
            };
            let authenticated = auth_enforced || principal.is_some() || token_valid;
            let bypass_enabled_gate = principal.is_some_and(|p| {
                p.role.is_admin() || (p.api_key_scopes.is_some() && p.has_scope("mcp"))
            });
            let can_read = principal.is_none_or(|p| p.has_scope("read") || p.has_scope("mcp"));
            let can_write = if let Some(p) = principal {
                if p.api_key_scopes.is_some() {
                    p.has_scope("write") && p.has_scope("mcp")
                } else {
                    p.role.is_admin()
                }
            } else {
                token_valid
            };
            let people_snap = state.people.read().unwrap().clone();
            let mcp_ctx = mcp::InboundMcpContext {
                cfg: &runtime.cfg,
                root_cfg,
                person_id: &person.id,
                role: caller_role,
                authenticated,
                can_read,
                can_write,
                bypass_enabled_gate,
                people: Some(&people_snap),
                library: Some(&state.library),
                store: Some(&runtime.store),
            };
            let (status, payload) = mcp::handle_inbound_mcp_with_auth(
                &mcp_ctx,
                req.authorization.as_deref(),
                &req.body,
            );
            state.search_cache.invalidate(&person.id);
            respond_json(stream, status, &payload)
        }
        ("GET", "/api/clients") => {
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            let store = records::load_clients_store(&root_cfg.data_dir);
            let client_filter = query_param(&req.query, "client_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let mut person_filter = query_param(&req.query, "person_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            if is_restricted_user {
                if let Some(ref pf) = person_filter {
                    if pf != &person.id {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot list clients or engagements belonging to another user",
                        ));
                    }
                }
                person_filter = Some(person.id.clone());
            }
            let engagements: Vec<Value> = store
                .engagements
                .iter()
                .filter(|e| client_filter.as_deref().is_none_or(|c| e.client_id == c))
                .filter(|e| person_filter.as_deref().is_none_or(|p| e.person_id == p))
                .map(|e| e.to_json())
                .collect();
            let clients: Vec<Value> = store
                .clients
                .iter()
                .filter(|c| !is_restricted_user || store.person_can_access_client(&c.id, &person.id))
                .filter(|c| client_filter.as_deref().is_none_or(|cf| c.id == cf))
                .map(|c| c.to_json())
                .collect();
            let active_engagement_id = store
                .active_engagement_by_person
                .get(&person.id)
                .cloned();
            respond_json(
                stream,
                200,
                &json!({
                    "clients": clients,
                    "engagements": engagements,
                    "active_engagement_id": active_engagement_id,
                }),
            )
        }
        ("POST", "/api/clients") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match records::create_client_for_person(&root_cfg.data_dir, Some(&person.id), &parsed) {
                Ok(client) => respond_json(stream, 201, &json!({ "client": client.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", "/api/engagements") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                if let Some(req_pid) = parsed["person_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    if req_pid != person.id {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot create an engagement for another user",
                        ));
                    }
                }
                if let Some(cid) = parsed["client_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    let store = records::load_clients_store(&root_cfg.data_dir);
                    if store.clients.iter().any(|c| c.id == cid)
                        && !store.person_can_access_client(cid, &person.id)
                    {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: you do not have access to this client",
                        ));
                    }
                }
            }
            match records::create_engagement(&root_cfg.data_dir, &person.id, &parsed) {
                Ok(engagement) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(stream, 201, &json!({ "engagement": engagement.to_json() }))
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", "/api/engagements/switch") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                if let Some(eid) = parsed["engagement_id"]
                    .as_str()
                    .or_else(|| parsed["id"].as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let store = records::load_clients_store(&root_cfg.data_dir);
                    if store.person_owns_engagement(eid, &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot switch to another user's engagement",
                        ));
                    }
                }
            }
            match records::switch_engagement(&root_cfg.data_dir, &person.id, &parsed) {
                Ok((active_engagement_id, engagement)) => respond_json(
                    stream,
                    200,
                    &json!({
                        "active_engagement_id": active_engagement_id,
                        "engagement": engagement.map(|e| e.to_json()),
                    }),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("PATCH", path) if path.starts_with("/api/clients/") => {
            let raw_id = path.strip_prefix("/api/clients/").unwrap_or("").trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "client not found"));
            }
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if store.clients.iter().any(|c| c.id == id)
                    && !store.person_can_access_client(&id, &person.id)
                {
                    return Some(respond_err(
                        stream,
                        403,
                        "forbidden: cannot modify another user's client",
                    ));
                }
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match records::update_client(&root_cfg.data_dir, &id, &parsed) {
                Ok(client) => respond_json(stream, 200, &json!({ "client": client.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", path)
            if path.starts_with("/api/engagements/") && path.ends_with("/context") =>
        {
            let raw_id = path
                .strip_prefix("/api/engagements/")
                .and_then(|r| r.strip_suffix("/context"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "engagement not found"));
            }
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if store.person_owns_engagement(&id, &person.id) == Some(false) {
                    return Some(respond_err(
                        stream,
                        403,
                        "forbidden: cannot inspect another user's engagement",
                    ));
                }
            }
            match records::compute_engagement_context(
                &root_cfg.data_dir,
                &runtime.cfg.data_dir,
                &person.id,
                &id,
            ) {
                Ok(mut ctx_val) => {
                    let idx = person_idx(state, person, runtime);
                    let (scoped_count, scoped_items) =
                        idx.list_files_scoped(0, Some(50), "", None, Some(&id));
                    ctx_val["scoped_files_count"] = json!(scoped_count);
                    ctx_val["scoped_files"] = json!(scoped_items);
                    let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
                    ctx_val["isolate_active_engagement"] = json!(policy.isolate_active_engagement);
                    ctx_val["policy_profile"] = json!(policy.profile);
                    if let Some(prompt_block) = crate::active_engagement_prompt_block(
                        &root_cfg.data_dir,
                        &runtime.cfg.data_dir,
                        &person.id,
                    ) {
                        if ctx_val["is_active"].as_bool() == Some(true) {
                            ctx_val["prompt_block"] = json!(prompt_block);
                        }
                    }
                    respond_json(stream, 200, &ctx_val)
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("PATCH", path) if path.starts_with("/api/engagements/") => {
            let raw_id = path.strip_prefix("/api/engagements/").unwrap_or("").trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "engagement not found"));
            }
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if store.person_owns_engagement(&id, &person.id) == Some(false) {
                    return Some(respond_err(
                        stream,
                        403,
                        "forbidden: cannot modify another user's engagement",
                    ));
                }
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match records::update_engagement(&root_cfg.data_dir, &id, &parsed) {
                Ok(engagement) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(stream, 200, &json!({ "engagement": engagement.to_json() }))
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/time-entries") => {
            let eng_filter = query_param(&req.query, "engagement_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let cli_filter = query_param(&req.query, "client_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if let Some(ref eid) = eng_filter {
                    if store.person_owns_engagement(eid, &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view time entries for another user's engagement",
                        ));
                    }
                }
                if let Some(ref cid) = cli_filter {
                    if store.clients.iter().any(|c| c.id == *cid)
                        && !store.person_can_access_client(cid, &person.id)
                    {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view time entries for another user's client",
                        ));
                    }
                }
            }
            let kind_filter = query_param(&req.query, "kind")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty());
            let billable_filter = query_param(&req.query, "billable")
                .and_then(|s| match s.trim().to_ascii_lowercase().as_str() {
                    "1" | "true" => Some(true),
                    "0" | "false" => Some(false),
                    _ => None,
                });
            let unbilled_only = query_param(&req.query, "unbilled")
                .or_else(|| query_param(&req.query, "unbilled_only"))
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let invoices = records::load_invoices(&runtime.cfg.data_dir);
            let active_inv_ids: std::collections::HashSet<String> = invoices
                .into_iter()
                .filter(|i| i.status != "void")
                .map(|i| i.id)
                .collect();

            let mut entries = records::load_time_entries(&runtime.cfg.data_dir);
            entries.retain(|e| {
                eng_filter.as_deref().is_none_or(|id| e.engagement_id == id)
                    && cli_filter.as_deref().is_none_or(|id| e.client_id == id)
                    && kind_filter.as_deref().is_none_or(|k| e.kind == k)
                    && billable_filter.is_none_or(|b| e.billable == b)
                    && (!unbilled_only
                        || e.invoiced_invoice_id
                            .as_deref()
                            .is_none_or(|iid| !active_inv_ids.contains(iid)))
            });
            entries.reverse();
            let total_hours: f64 = entries.iter().map(|e| e.hours).sum();
            let total_amount: f64 = entries.iter().map(|e| e.amount_usd).sum();
            let billable_amount: f64 = entries
                .iter()
                .filter(|e| e.billable)
                .map(|e| e.amount_usd)
                .sum();
            respond_json(
                stream,
                200,
                &json!({
                    "time_entries": entries.iter().map(records::TimeExpenseEntry::to_json).collect::<Vec<_>>(),
                    "totals": {
                        "count": entries.len(),
                        "hours": records::round_4(total_hours),
                        "amount_usd": records::round_4(total_amount),
                        "billable_amount_usd": records::round_4(billable_amount),
                    },
                }),
            )
        }
        ("POST", "/api/time-entries") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                if let Some(eid) = parsed["engagement_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    let store = records::load_clients_store(&root_cfg.data_dir);
                    if store.person_owns_engagement(eid, &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot log time against another user's engagement",
                        ));
                    }
                }
            }
            match records::create_time_entry(
                &root_cfg.data_dir,
                &runtime.cfg.data_dir,
                &person.id,
                &parsed,
            ) {
                Ok(entry) => respond_json(stream, 201, &json!({ "entry": entry.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path) if path == "/api/time-entries" || path.starts_with("/api/time-entries/") => {
            let path_id = path
                .strip_prefix("/api/time-entries/")
                .map(|s| percent_decode(s).trim().to_string())
                .filter(|s| !s.is_empty());
            let body_id = if req.body.is_empty() {
                None
            } else {
                serde_json::from_slice::<Value>(&req.body)
                    .ok()
                    .and_then(|v| v["id"].as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
            };
            let id = path_id
                .or_else(|| query_param(&req.query, "id").filter(|s| !s.is_empty()))
                .or(body_id)
                .unwrap_or_default();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 400, "time entry id is required"));
            }
            match records::delete_time_entry(&runtime.cfg.data_dir, &id) {
                Ok(removed) => respond_json(
                    stream,
                    200,
                    &json!({ "ok": true, "deleted_id": removed.id, "entry": removed.to_json() }),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/invoices") => {
            let eng_filter = query_param(&req.query, "engagement_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let cli_filter = query_param(&req.query, "client_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if let Some(ref eid) = eng_filter {
                    if store.person_owns_engagement(eid, &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view invoices for another user's engagement",
                        ));
                    }
                }
                if let Some(ref cid) = cli_filter {
                    if store.clients.iter().any(|c| c.id == *cid)
                        && !store.person_can_access_client(cid, &person.id)
                    {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view invoices for another user's client",
                        ));
                    }
                }
            }
            let status_filter = query_param(&req.query, "status")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty());
            let mut invoices = records::load_invoices(&runtime.cfg.data_dir);
            invoices.retain(|inv| {
                eng_filter.as_deref().is_none_or(|id| inv.engagement_id == id)
                    && cli_filter.as_deref().is_none_or(|id| inv.client_id == id)
                    && status_filter.as_deref().is_none_or(|st| inv.status == st)
            });
            invoices.reverse();
            respond_json(
                stream,
                200,
                &json!({
                    "invoices": invoices.iter().map(records::InvoiceRecord::to_json).collect::<Vec<_>>(),
                }),
            )
        }
        ("POST", "/api/invoices/generate") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                if let Some(eid) = parsed["engagement_id"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    let store = records::load_clients_store(&root_cfg.data_dir);
                    if store.person_owns_engagement(eid, &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot generate an invoice for another user's engagement",
                        ));
                    }
                }
            }
            match records::generate_invoice(&root_cfg.data_dir, &runtime.cfg, &person.id, &parsed) {
                Ok(invoice) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(
                        stream,
                        201,
                        &json!({
                            "invoice": invoice.to_json(),
                            "deliverables": invoice.deliverable_paths,
                        }),
                    )
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("PATCH", path) if path.starts_with("/api/invoices/") => {
            let raw_id = path.strip_prefix("/api/invoices/").unwrap_or("").trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "invoice not found"));
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match records::update_invoice_status(&runtime.cfg.data_dir, &id, &parsed) {
                Ok(invoice) => respond_json(stream, 200, &json!({ "invoice": invoice.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/metering/summary") => {
            let engagement_id = query_param(&req.query, "engagement_id");
            let client_id = query_param(&req.query, "client_id");
            let is_restricted_user = auth_enforced && !principal.is_some_and(|p| p.role.is_admin());
            if is_restricted_user {
                let store = records::load_clients_store(&root_cfg.data_dir);
                if let Some(ref eid) = engagement_id {
                    if store.person_owns_engagement(eid.trim(), &person.id) == Some(false) {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view metering for another user's engagement",
                        ));
                    }
                }
                if let Some(ref cid) = client_id {
                    let cid_t = cid.trim();
                    if store.clients.iter().any(|c| c.id == cid_t)
                        && !store.person_can_access_client(cid_t, &person.id)
                    {
                        return Some(respond_err(
                            stream,
                            403,
                            "forbidden: cannot view metering for another user's client",
                        ));
                    }
                }
            }
            let window_days = query_param(&req.query, "window_days")
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(30);
            let summary = records::compute_metering_summary(
                &root_cfg.data_dir,
                &runtime.cfg.data_dir,
                &person.id,
                engagement_id.as_deref(),
                client_id.as_deref(),
                window_days,
            );
            respond_json(stream, 200, &summary)
        }
        _ => return None,
    })
}
