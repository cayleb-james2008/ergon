use super::*;

/// Routes decisions, proactive offers, morning brief, company DB/memory, tools, notifications, keys, webhooks, self-improvement, and quality bars.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_route_company_work(
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
        ("GET", "/api/decisions") => {
            let decisions = records::load_decisions(&runtime.cfg.data_dir);
            respond_json(stream, 200, &json!({"decisions": decisions}))
        }
        ("GET", "/api/offers") => {
            // Refresh first, so the inbox shows what the work log has
            // newly earned since the last look. The library is cloned
            // and the lock dropped before the scan does file I/O —
            // the mutex guards the in-memory items, nothing slower.
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let offers = refresh_offers_and_notify(
                &runtime.cfg,
                &suite,
                &lib,
                Some(&state.events),
                &person.id,
            );
            respond_json(stream, 200, &json!({"offers": offers}))
        }
        ("GET", "/api/brief") => {
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            refresh_offers_and_notify(
                &runtime.cfg,
                &suite,
                &lib,
                Some(&state.events),
                &person.id,
            );
            let brief = records::build_brief(&runtime.cfg, &person.name, &suite, &lib);
            respond_json(stream, 200, &brief)
        }
        ("GET", "/api/today") => {
            let mut today = records::build_today(&runtime.cfg);
            // Tripwire events from today join the stream — warnings and
            // up only (individual decoy probes stay in Settings → Security),
            // and never twice: an event already shown as a decision title
            // is not repeated.
            let date = today["date"].as_str().unwrap_or("").to_string();
            let mut decision_titles: Vec<String> = records::load_decisions(&runtime.cfg.data_dir)
                .iter()
                .filter_map(|d| d["summary"].as_str().map(|s| s.to_string()))
                .collect();
            if person.id != "default" {
                // Machine-level notices live in the default desk's inbox.
                decision_titles.extend(
                    records::load_decisions(&root_cfg.data_dir)
                        .iter()
                        .filter_map(|d| d["summary"].as_str().map(|s| s.to_string())),
                );
            }
            if let Some(events) = today["events"].as_array_mut() {
                for e in tripwire::events_for(&root_cfg.data_dir, &person.id, 200) {
                    let ts = e["ts"].as_u64().unwrap_or(0);
                    let sev = tripwire::Severity::from_label(e["severity"].as_str().unwrap_or(""));
                    let title = e["title"].as_str().unwrap_or("");
                    if sev >= tripwire::Severity::Warning
                        && records::date_string_local(ts) == date
                        && !decision_titles.iter().any(|t| t == title)
                    {
                        events.push(json!({
                            "ts": ts,
                            "kind": "security",
                            "title": e["title"],
                            "detail": e["detail"],
                            "severity": e["severity"],
                            "action": e["action"],
                        }));
                    }
                }
                events.sort_by_key(|e| std::cmp::Reverse(e["ts"].as_u64().unwrap_or(0)));
            }
            respond_json(stream, 200, &today)
        }
        ("GET", "/api/graph") => {
            // Cloned snapshots, locks dropped: build_graph reads
            // workspace files from disk and must not hold the
            // people/library locks across that I/O.
            // In auth-enforced mode, non-admins see ONLY their own
            // isolated node/work/files — never other employees' data.
            let people_snapshot = if auth_enforced && !principal.is_some_and(|p| p.role.is_admin()) {
                vec![person.clone()]
            } else {
                state.people.read().unwrap().people.clone()
            };
            let lib = state.library.lock().unwrap().clone();
            respond_json(
                stream,
                200,
                &records::build_graph(&people_snapshot, root_cfg, &lib),
            )
        }
        ("POST", path) if path.starts_with("/api/decisions/") => {
            let rest = path.trim_start_matches("/api/decisions/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return Some(respond_err(stream, 404, "not found"))}
            };
            if action != "approve" && action != "deny" {
                return Some(respond_err(stream, 404, "not found"));
            }
            match records::resolve_decision(&runtime.cfg, &id, action == "approve") {
                Ok((decision, outcome)) => {
                    // Tripwire decisions act after the settle (blocking a
                    // site, quarantining a file, publishing a held skill
                    // need app state the records lock must not span).
                    let (decision, outcome) = settle_tripwire_decision(state, runtime, person, &decision, action == "approve")
                        .unwrap_or((decision, outcome));
                    let (decision, outcome) = if decision["kind"].as_str() == Some("mcp_tool_call") {
                        let final_outcome = if action == "approve" {
                            mcp::settle_mcp_decision(&runtime.cfg, &decision)
                        } else {
                            "You said no - MCP tool call was cancelled.".to_string()
                        };
                        let updated = records::set_decision_outcome(
                            &runtime.cfg.data_dir,
                            &id,
                            &final_outcome,
                        )
                        .unwrap_or(decision);
                        (updated, final_outcome)
                    } else if decision["kind"].as_str() == Some("tool_offer") {
                        let final_outcome = {
                            let mut lib = state.library.lock().unwrap();
                            tool_builder::settle_tool_offer_decision(
                                &root_cfg.data_dir,
                                &runtime.cfg.data_dir,
                                &mut lib,
                                &decision,
                                action == "approve",
                            )
                        };
                        let updated = records::set_decision_outcome(
                            &runtime.cfg.data_dir,
                            &id,
                            &final_outcome,
                        )
                        .unwrap_or(decision);
                        (updated, final_outcome)
                    } else {
                        (decision, outcome)
                    };
                    if decision["kind"].as_str() == Some("checkpoint") {
                        if let Some(prun_id) = decision["pipeline_run_id"].as_str().filter(|s| !s.is_empty()) {
                            let env = pipeline_exec_env(
                                state,
                                person,
                                runtime,
                                &ToolCtx {
                                    library: state.library.clone(),
                                    root_data_dir: root_cfg.data_dir.clone(),
                                    person_id: person.id.clone(),
                                },
                            );
                            let resume_body = json!({
                                "action": if action == "approve" { "approve" } else { "reject" }
                            });
                            let _ = pipelines::resume_pipeline(env, prun_id, &resume_body, true);
                        }
                    } else if decision["kind"].as_str() == Some("connector_delivery") {
                        if let Some(linked_run) = pipelines::find_run_by_checkpoint_id(&runtime.cfg.data_dir, &id) {
                            let env = pipeline_exec_env(
                                state,
                                person,
                                runtime,
                                &ToolCtx {
                                    library: state.library.clone(),
                                    root_data_dir: root_cfg.data_dir.clone(),
                                    person_id: person.id.clone(),
                                },
                            );
                            let resume_body = json!({
                                "action": if action == "approve" { "approve" } else { "reject" }
                            });
                            let _ = pipelines::resume_pipeline(env, &linked_run.run_id, &resume_body, true);
                        }
                    }
                    // The outcome reports back into the thread the
                    // decision came from, so the record is complete
                    // where the work happened.
                    let tid = decision["thread_id"].as_str().unwrap_or("");
                    if !tid.is_empty() {
                        let mut store = runtime.store.lock().unwrap();
                        store.push(
                            tid,
                            "step",
                            &format!(
                                "Decision settled — {}: {}",
                                decision["summary"].as_str().unwrap_or(""),
                                outcome
                            ),
                        );
                        // The parked-delete note says the file is still
                        // there; once settled it must say what happened (ER1).
                        if let Some(path) = decision["path"].as_str().filter(|_| decision["kind"] == "delete") {
                            let settled = if action != "approve" {
                                format!("Note: {path} was kept because you declined the deletion.")
                            } else if outcome.starts_with("Approved and done") {
                                format!("Note: {path} was deleted after you approved it.")
                            } else if outcome.starts_with("Approved: ") {
                                format!("Note: {path} was already gone when you approved it.")
                            } else {
                                format!("Note: {path} was not deleted; see the decision settled above.")
                            };
                            store.settle_note(tid, &parked_note(path), &settled);
                        }
                    }
                    state.search_cache.invalidate(&person.id);
                    let approved = action == "approve";
                    let dec_artifact_id = decision["artifact_id"]
                        .as_str()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .or_else(|| {
                            if decision["kind"].as_str() == Some("tool_offer") {
                                decision["tool_name"]
                                    .as_str()
                                    .map(crate::slugify)
                                    .filter(|s| !s.is_empty())
                            } else {
                                None
                            }
                        });
                    if let Some(ref art_id) = dec_artifact_id {
                        let art_kind = decision["artifact_kind"]
                            .as_str()
                            .unwrap_or("tool")
                            .to_string();
                        let sum = {
                            let mut lib = state.library.lock().unwrap();
                            let art_ver = lib.get(art_id).map(|i| i.version).unwrap_or(1);
                            self_improve::record_outcome(
                                &root_cfg.data_dir,
                                Some(&mut lib),
                                self_improve::OutcomeRecord {
                                    id: String::new(),
                                    ts_ms: records::now_millis(),
                                    person_id: person.id.clone(),
                                    artifact_kind: art_kind,
                                    artifact_id: art_id.clone(),
                                    artifact_version: art_ver,
                                    run_id: tid.to_string(),
                                    task: decision["summary"].as_str().unwrap_or("").to_string(),
                                    approved: Some(approved),
                                    edits_needed: 0,
                                    reruns: 0,
                                    time_saved_mins: 0.0,
                                    explicit_score: None,
                                    quality_score: 0.0,
                                    feedback: Some(outcome.clone()),
                                },
                            )
                        };
                        if let Some(prev_v) = sum.rolled_back_to {
                            connectors::dispatch_event_webhooks(
                                &root_cfg.data_dir,
                                "evolution.rolled_back",
                                json!({
                                    "artifact_id": sum.record.artifact_id,
                                    "kind": sum.record.artifact_kind,
                                    "rolled_back_to": prev_v,
                                    "reason": sum.rollback_reason,
                                    "person_id": person.id,
                                }),
                            );
                        }
                    }
                    connectors::dispatch_event_webhooks(
                        &root_cfg.data_dir,
                        "decision.settled",
                        json!({
                            "decision_id": decision["id"],
                            "kind": decision["kind"],
                            "status": decision["status"],
                            "approved": approved,
                            "outcome": outcome,
                            "person_id": person.id,
                            "thread_id": tid,
                        }),
                    );
                    respond_json(
                        stream,
                        200,
                        &json!({"decision": decision, "outcome": outcome}),
                    )
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/offers/analyze") => {
            let eff = effective_cfg(state, runtime);
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let analysis = proactive_offers::analyze_proactive_offers(
                &eff,
                &person.id,
                &suite,
                &lib,
                Some(&state.events),
            );
            let mut resp = analysis.to_json();
            if let Some(ref spec) = analysis.proposed_tool {
                let name = spec["tool_name"].as_str().unwrap_or("");
                let description = spec["tool_description"].as_str().unwrap_or("");
                let params = string_list(&spec["params"]);
                let source = spec["source"].as_str().unwrap_or("").to_string();
                match parse_tool_tests(&spec["tests"]).and_then(|tests| {
                    tool_builder::stage_tool_for_approval(
                        &root_cfg.data_dir,
                        &runtime.cfg,
                        &person.id,
                        "",
                        name,
                        description,
                        params,
                        source,
                        tests,
                    )
                }) {
                    Ok(pkg) => {
                        proactive_offers::emit_notification(
                            &runtime.cfg.data_dir,
                            Some(&state.events),
                            &person.id,
                            "approval_parked",
                            &format!("Would this help? New tool: {}", pkg.name),
                            &format!(
                                "{}/{} sandbox tests passed. Approve to install and share.",
                                pkg.test_report.passed, pkg.test_report.total
                            ),
                            "decision",
                            &pkg.decision_id,
                        );
                        resp["staged_tool"] = pkg.to_json();
                        resp["staged_decision_id"] = json!(pkg.decision_id);
                    }
                    Err(e) => {
                        resp["staged_error"] = json!(e);
                    }
                }
            }
            respond_json(stream, 200, &resp)
        }
        ("POST", path) if path.starts_with("/api/offers/") => {
            let rest = path.trim_start_matches("/api/offers/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return Some(respond_err(stream, 404, "not found"))}
            };
            match action.as_str() {
                "dismiss" | "deny" => match records::set_offer_status(&runtime.cfg.data_dir, &id, "dismissed") {
                    Ok(offer) => respond_json(stream, 200, &json!({"offer": offer})),
                    Err(e) => respond_err(stream, 400, &e),
                },
                "accept" | "approve" => {
                    // Look the offer up first; acceptance DOES the
                    // thing offered (install the app / library tool /
                    // stage or install built tool / create one-tap app),
                    // then marks the offer settled.
                    let offers = records::load_offers(&runtime.cfg.data_dir);
                    let Some(offer) = offers.iter().find(|o| o["id"].as_str() == Some(id.as_str())).cloned() else {
                        return Some(respond_err(stream, 404, "offer not found"));
                    };
                    if offer["status"].as_str() != Some("pending") {
                        return Some(respond_err(stream, 400, "that offer was already settled"));
                    }
                    let kind = offer["kind"].as_str().unwrap_or("");
                    let mut made: Option<Value> = None;
                    if kind == "install_app" || kind == "install_library_tool" {
                        let item_id = offer["item_id"].as_str().unwrap_or("");
                        let exists = {
                            let lib = state.library.lock().unwrap();
                            lib.get(item_id).is_some()
                        };
                        if !exists {
                            return Some(respond_err(stream, 404, "library item not found"));
                        }
                        library::with_suite_lock(|| {
                            let mut suite = library::load_suite(&runtime.cfg.data_dir);
                            if suite.install(item_id) {
                                library::save_suite(&runtime.cfg.data_dir, &suite);
                            }
                        });
                        made = Some(json!({"installed": item_id}));
                    } else if kind == "build_tool" {
                        if let Some(pkg_id) = offer["tool_package_id"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                        {
                            let Some(pkg) = tool_builder::get_tool_package(&root_cfg.data_dir, pkg_id) else {
                                return Some(respond_err(stream, 404, "tool package not found"));
                            };
                            if !pkg.test_report.all_passed {
                                return Some(respond_err(stream, 400, "tool package failed tests"));
                            }
                            let outcome = if !pkg.decision_id.is_empty() {
                                if let Ok((dec, _)) =
                                    records::resolve_decision(&runtime.cfg, &pkg.decision_id, true)
                                {
                                    let mut lib = state.library.lock().unwrap();
                                    let out = tool_builder::settle_tool_offer_decision(
                                        &root_cfg.data_dir,
                                        &runtime.cfg.data_dir,
                                        &mut lib,
                                        &dec,
                                        true,
                                    );
                                    let _ = records::set_decision_outcome(
                                        &runtime.cfg.data_dir,
                                        &pkg.decision_id,
                                        &out,
                                    );
                                    out
                                } else {
                                    let mut lib = state.library.lock().unwrap();
                                    tool_builder::settle_tool_offer_decision(
                                        &root_cfg.data_dir,
                                        &runtime.cfg.data_dir,
                                        &mut lib,
                                        &json!({ "tool_package_id": pkg.id }),
                                        true,
                                    )
                                }
                            } else {
                                let mut lib = state.library.lock().unwrap();
                                tool_builder::settle_tool_offer_decision(
                                    &root_cfg.data_dir,
                                    &runtime.cfg.data_dir,
                                    &mut lib,
                                    &json!({ "tool_package_id": pkg.id }),
                                    true,
                                )
                            };
                            made = Some(json!({"tool_package_id": pkg.id, "outcome": outcome}));
                        } else {
                            let name = offer["tool_name"]
                                .as_str()
                                .or_else(|| offer["name"].as_str())
                                .unwrap_or("");
                            let description = offer["tool_description"]
                                .as_str()
                                .or_else(|| offer["detail"].as_str())
                                .unwrap_or("");
                            let params = string_list(&offer["params"]);
                            let source = offer["source"].as_str().unwrap_or("").to_string();
                            let tests = match parse_tool_tests(&offer["tests"]) {
                                Ok(t) => t,
                                Err(e) => return Some(respond_err(stream, 400, &e)),
                            };
                            match tool_builder::stage_tool_for_approval(
                                &root_cfg.data_dir,
                                &runtime.cfg,
                                &person.id,
                                "",
                                name,
                                description,
                                params,
                                source,
                                tests,
                            ) {
                                Ok(pkg) => {
                                    proactive_offers::emit_notification(
                                        &runtime.cfg.data_dir,
                                        Some(&state.events),
                                        &person.id,
                                        "approval_parked",
                                        &format!("Would this help? New tool: {}", pkg.name),
                                        &format!(
                                            "{}/{} sandbox tests passed. Approve to install and share.",
                                            pkg.test_report.passed, pkg.test_report.total
                                        ),
                                        "decision",
                                        &pkg.decision_id,
                                    );
                                    made = Some(json!({
                                        "staged_tool": pkg.to_json(),
                                        "decision_id": pkg.decision_id,
                                    }));
                                }
                                Err(e) => return Some(respond_err(stream, 400, &e)),
                            }
                        }
                    } else if kind == "repeat_task" {
                        let template = offer["template"].as_str().unwrap_or("");
                        let short: String = template.chars().take(48).collect();
                        let name = format!("One-tap: {short}");
                        let mut lib = state.library.lock().unwrap();
                        match lib.publish_app(
                            &name,
                            "Created from an offer: work this person kept asking for, as a one-tap app.",
                            template.to_string(),
                            Vec::new(),
                            &person.id,
                        ) {
                            Ok(item) => {
                                lib.save(&root_cfg.data_dir);
                                library::with_suite_lock(|| {
                                    let mut suite =
                                        library::load_suite(&runtime.cfg.data_dir);
                                    suite.install(&item.id);
                                    library::save_suite(&runtime.cfg.data_dir, &suite);
                                });
                                made = Some(item.to_json());
                            }
                            Err(e) => {
                                return Some(respond_err(stream, 400, &e))}
                        }
                    }
                    match records::set_offer_status(&runtime.cfg.data_dir, &id, "accepted") {
                        Ok(offer) => respond_json(
                            stream,
                            200,
                            &json!({"offer": offer, "made": made}),
                        ),
                        Err(e) => respond_err(stream, 400, &e),
                    }
                }
                _ => respond_err(stream, 404, "not found"),
            }
        }
        ("GET", "/api/company/memory") => {
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                auth::Role::Admin
            };
            let allowed = company_db::allowed_client_ids_for_memory(
                &root_cfg.data_dir,
                &person.id,
                caller_role,
            );
            let entries: Vec<Value> =
                company_db::list_company_memory_for(&root_cfg.data_dir, caller_role, &allowed)
                    .into_iter()
                    .map(|e| e.to_json())
                    .collect();
            respond_json(
                stream,
                200,
                &json!({
                    "entries": entries,
                    "role": caller_role.as_str(),
                    "can_write": caller_role.can_manage_clients(),
                }),
            )
        }
        ("POST", "/api/company/memory") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                auth::Role::Admin
            };
            let category = parsed["category"].as_str().unwrap_or("fact");
            let scope = parsed["scope"].as_str().unwrap_or("company");
            let content = parsed["content"].as_str().unwrap_or("");
            match company_db::add_company_memory(
                &root_cfg.data_dir,
                caller_role,
                &person.id,
                category,
                scope,
                content,
            ) {
                Ok(entry) => respond_json(stream, 201, &json!({ "entry": entry.to_json() })),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path) if path.starts_with("/api/company/memory/") => {
            let id = percent_decode(path.trim_start_matches("/api/company/memory/"))
                .trim()
                .to_string();
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                auth::Role::Admin
            };
            match company_db::delete_company_memory(&root_cfg.data_dir, caller_role, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "company memory entry not found"),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/company/db") => {
            let table = query_param(&req.query, "table").unwrap_or_else(|| "summary".to_string());
            let q = query_param(&req.query, "q")
                .or_else(|| query_param(&req.query, "query"))
                .filter(|s| !s.trim().is_empty());
            let limit = query_param(&req.query, "limit")
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(50);
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                auth::Role::Admin
            };
            let people = state.people.read().unwrap().clone();
            let lib = state.library.lock().unwrap().clone();
            match company_db::query_company_db(
                &root_cfg.data_dir,
                root_cfg,
                &people,
                &lib,
                &person.id,
                caller_role,
                &table,
                q.as_deref(),
                limit,
            ) {
                Ok(res) => respond_json(stream, 200, &res),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", "/api/company/db/query") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let table = parsed["table"].as_str().unwrap_or("summary");
            let q = parsed["query"]
                .as_str()
                .or_else(|| parsed["q"].as_str())
                .filter(|s| !s.trim().is_empty());
            let limit = parsed["limit"].as_u64().unwrap_or(50) as usize;
            let caller_role = if auth_enforced {
                principal.map(|p| p.role).unwrap_or(auth::Role::Member)
            } else {
                auth::Role::Admin
            };
            let people = state.people.read().unwrap().clone();
            let lib = state.library.lock().unwrap().clone();
            match company_db::query_company_db(
                &root_cfg.data_dir,
                root_cfg,
                &people,
                &lib,
                &person.id,
                caller_role,
                table,
                q,
                limit,
            ) {
                Ok(res) => respond_json(stream, 200, &res),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/tools") => {
            let packages: Vec<Value> = tool_builder::list_tool_packages(&root_cfg.data_dir)
                .into_iter()
                .map(|p| p.to_json())
                .collect();
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let library_tools: Vec<Value> = lib
                .items
                .iter()
                .filter(|i| i.kind == "tool")
                .map(|i| i.to_json())
                .collect();
            let installed_tool_ids: Vec<String> = suite
                .resolved(&lib)
                .into_iter()
                .filter(|i| i.kind == "tool")
                .map(|i| i.id)
                .collect();
            respond_json(
                stream,
                200,
                &json!({
                    "packages": packages,
                    "library_tools": library_tools,
                    "installed_tool_ids": installed_tool_ids,
                }),
            )
        }
        ("POST", "/api/tools/test") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let params = string_list(&parsed["params"]);
            let source = parsed["source"].as_str().unwrap_or("");
            let tests = match parse_tool_tests(&parsed["tests"]) {
                Ok(t) => t,
                Err(e) => return Some(respond_err(stream, 400, &e)),
            };
            match tool_builder::verify_and_test_tool(&root_cfg.data_dir, &params, source, &tests) {
                Ok(report) => respond_json(stream, 200, &json!({ "report": report.to_json() })),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/tools/build") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let name = parsed["name"].as_str().unwrap_or("");
            let description = parsed["description"].as_str().unwrap_or("");
            let params = string_list(&parsed["params"]);
            let source = parsed["source"].as_str().unwrap_or("").to_string();
            let thread_id = parsed["thread_id"].as_str().unwrap_or("");
            let tests = match parse_tool_tests(&parsed["tests"]) {
                Ok(t) => t,
                Err(e) => return Some(respond_err(stream, 400, &e)),
            };
            match tool_builder::stage_tool_for_approval(
                &root_cfg.data_dir,
                &runtime.cfg,
                &person.id,
                thread_id,
                name,
                description,
                params,
                source,
                tests,
            ) {
                Ok(pkg) => {
                    proactive_offers::emit_notification(
                        &runtime.cfg.data_dir,
                        Some(&state.events),
                        &person.id,
                        "approval_parked",
                        &format!("Would this help? New tool: {}", pkg.name),
                        &format!(
                            "{}/{} sandbox tests passed. Approve to install and share.",
                            pkg.test_report.passed, pkg.test_report.total
                        ),
                        "decision",
                        &pkg.decision_id,
                    );
                    respond_json(
                        stream,
                        201,
                        &json!({ "package": pkg.to_json(), "decision_id": pkg.decision_id }),
                    )
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", path) if path.starts_with("/api/tools/") && path.ends_with("/run") => {
            let raw_id = path
                .strip_prefix("/api/tools/")
                .and_then(|rest| rest.strip_suffix("/run"))
                .unwrap_or("");
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() {
                return Some(respond_err(stream, 404, "tool not found"));
            }
            let item = {
                let lib = state.library.lock().unwrap();
                lib.get(&id)
            };
            let Some(item) = item else {
                return Some(respond_err(stream, 404, "tool not found"));
            };
            if !matches!(item.kind.as_str(), "tool" | "skill") || item.builtin {
                return Some(respond_err(stream, 400, "that library item is not a runnable tool"));
            }
            let suite = library::load_suite(&runtime.cfg.data_dir);
            if !suite.installed.iter().any(|x| x == &item.id) {
                return Some(respond_err(stream, 403, "that tool is not installed in your suite"));
            }
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
            let scalar_str = |v: &Value| -> String {
                match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => String::new(),
                }
            };
            let values: Vec<String> = if let Some(arr) = parsed["args"].as_array() {
                arr.iter().map(scalar_str).collect()
            } else {
                let arg_obj = if parsed["args"].is_object() {
                    &parsed["args"]
                } else {
                    &parsed
                };
                item.params
                    .iter()
                    .map(|p| scalar_str(&arg_obj[p.as_str()]))
                    .collect()
            };
            match skills::run_library_skill(&runtime.cfg, &item.source, &values) {
                Ok(output) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(
                        stream,
                        200,
                        &json!({ "ok": true, "id": item.id, "output": output }),
                    )
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", "/api/notifications") => {
            let mut items = proactive_offers::load_notifications(&runtime.cfg.data_dir);
            items.reverse();
            let unread_count = items.iter().filter(|n| !n.read).count();
            let list: Vec<Value> = items.into_iter().map(|n| n.to_json()).collect();
            respond_json(
                stream,
                200,
                &json!({ "notifications": list, "unread_count": unread_count }),
            )
        }
        ("POST", path) if path.starts_with("/api/notifications/") && path.ends_with("/read") => {
            let raw_id = path
                .strip_prefix("/api/notifications/")
                .and_then(|rest| rest.strip_suffix("/read"))
                .unwrap_or("");
            let id = percent_decode(raw_id).trim().to_string();
            match proactive_offers::mark_notification_read(&runtime.cfg.data_dir, &id) {
                Ok(item) => respond_json(stream, 200, &json!({ "notification": item.to_json() })),
                Err(e) if e == "notification not found" => respond_err(stream, 404, &e),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", "/api/push/subscriptions") => {
            let subs: Vec<Value> = proactive_offers::list_push_subscriptions(&runtime.cfg.data_dir)
                .into_iter()
                .map(|s| s.to_json())
                .collect();
            respond_json(stream, 200, &json!({ "subscriptions": subs }))
        }
        ("POST", "/api/push/subscribe") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let channel = parsed["channel"].as_str().unwrap_or("web_push");
            let endpoint = parsed["endpoint"].as_str().unwrap_or("");
            let label = parsed["label"].as_str().unwrap_or("");
            match proactive_offers::add_push_subscription(
                &runtime.cfg.data_dir,
                channel,
                endpoint,
                label,
            ) {
                Ok(sub) => respond_json(stream, 201, &json!({ "subscription": sub.to_json() })),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("DELETE", "/api/push/subscribe") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| query_param(&req.query, "id"))
                .unwrap_or_default();
            match proactive_offers::remove_push_subscription(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "push subscription not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("DELETE", path) if path.starts_with("/api/push/subscribe/") => {
            let id = percent_decode(path.trim_start_matches("/api/push/subscribe/"))
                .trim()
                .to_string();
            match proactive_offers::remove_push_subscription(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "push subscription not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        // Scoped API Keys (/api/keys*)
        ("GET", "/api/keys") => {
            let keys: Vec<Value> = auth::list_api_keys(&root_cfg.data_dir)
                .into_iter()
                .map(|k| k.to_public_json())
                .collect();
            respond_json(stream, 200, &json!({ "keys": keys }))
        }
        ("POST", "/api/keys") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let name = parsed["name"].as_str().unwrap_or("");
            let scopes = string_list(&parsed["scopes"]);
            let target_person_id = parsed["person_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(&person.id);
            let role = if let Some(r_str) = parsed["role"].as_str().filter(|s| !s.trim().is_empty()) {
                match auth::Role::parse_strict(r_str) {
                    Some(r) => r,
                    None => return Some(respond_err(stream, 400, "invalid role")),
                }
            } else {
                let users = state.auth.list_users();
                if let Some(u) = users.iter().find(|u| u.person_id == target_person_id) {
                    u.role
                } else {
                    let people = state.people.read().unwrap();
                    people
                        .people
                        .iter()
                        .find(|p| p.id == target_person_id)
                        .map(|p| auth::Role::parse_lenient(&p.role))
                        .unwrap_or(auth::Role::Admin)
                }
            };
            let expires_at_ms = parsed["expires_at_ms"].as_u64();
            match auth::create_api_key(
                &root_cfg.data_dir,
                name,
                target_person_id,
                role,
                &scopes,
                expires_at_ms,
            ) {
                Ok((rec, raw_key)) => {
                    let mut out = rec.to_public_json();
                    out["key"] = json!(raw_key);
                    out["api_key"] = rec.to_public_json();
                    respond_json(stream, 201, &out)
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path) if path.starts_with("/api/keys/") => {
            let id = percent_decode(path.trim_start_matches("/api/keys/"))
                .trim()
                .to_string();
            match auth::revoke_api_key(&root_cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "API key not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        // Outbound Webhook Subscriptions (/api/webhooks/subscriptions*)
        ("GET", "/api/webhooks/subscriptions") => {
            let subs: Vec<Value> = connectors::list_webhook_subscriptions(&root_cfg.data_dir)
                .into_iter()
                .map(|s| s.to_public_json())
                .collect();
            respond_json(stream, 200, &json!({ "subscriptions": subs }))
        }
        ("POST", "/api/webhooks/subscriptions") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match connectors::create_webhook_subscription(&root_cfg.data_dir, &parsed) {
                Ok((sub, signing_secret)) => respond_json(
                    stream,
                    201,
                    &json!({
                        "subscription": sub.to_public_json(),
                        "signing_secret": signing_secret,
                    }),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", "/api/webhooks/subscriptions") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| query_param(&req.query, "id"))
                .unwrap_or_default();
            match connectors::delete_webhook_subscription(&root_cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "webhook subscription not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("DELETE", path) if path.starts_with("/api/webhooks/subscriptions/") => {
            let id = percent_decode(path.trim_start_matches("/api/webhooks/subscriptions/"))
                .trim()
                .to_string();
            match connectors::delete_webhook_subscription(&root_cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({ "deleted": true, "id": id })),
                Ok(false) => respond_err(stream, 404, "webhook subscription not found"),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        // Self-Improvement / Evolution (/api/evolution/*)
        ("GET", "/api/evolution/scores") => {
            respond_json(stream, 200, &self_improve::score_summary(&root_cfg.data_dir))
        }
        ("GET", path) if path.starts_with("/api/evolution/history/") => {
            let id = percent_decode(path.trim_start_matches("/api/evolution/history/"))
                .trim()
                .to_string();
            if id.is_empty() {
                return Some(respond_err(stream, 400, "artifact_id is required"));
            }
            respond_json(
                stream,
                200,
                &self_improve::artifact_history(&root_cfg.data_dir, &id),
            )
        }
        ("POST", "/api/evolution/outcomes") => {
            let mut parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            if (auth_enforced && !principal.is_some_and(|p| p.role.is_admin()))
                || parsed
                    .get("person_id")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .is_none_or(|s| s.is_empty())
            {
                parsed["person_id"] = json!(person.id);
            }
            let Some(rec) = self_improve::OutcomeRecord::from_json(&parsed) else {
                return Some(respond_err(
                    stream,
                    400,
                    "invalid outcome payload: artifact_id is required",
                ));
            };
            let sum = {
                let mut lib = state.library.lock().unwrap();
                self_improve::record_outcome(&root_cfg.data_dir, Some(&mut lib), rec)
            };
            if let Some(prev_v) = sum.rolled_back_to {
                connectors::dispatch_event_webhooks(
                    &root_cfg.data_dir,
                    "evolution.rolled_back",
                    json!({
                        "artifact_id": sum.record.artifact_id,
                        "kind": sum.record.artifact_kind,
                        "rolled_back_to": prev_v,
                        "reason": sum.rollback_reason,
                        "person_id": sum.record.person_id,
                    }),
                );
            }
            respond_json(stream, 200, &sum.to_json())
        }
        ("POST", "/api/evolution/cases") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let Some(case) = self_improve::ReplayCase::from_json(&parsed) else {
                return Some(respond_err(
                    stream,
                    400,
                    "invalid replay case: artifact_id is required",
                ));
            };
            match self_improve::add_replay_case(&root_cfg.data_dir, case) {
                Ok(saved) => respond_json(stream, 201, &json!({ "case": saved.to_json() })),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/evolution/propose") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let artifact_id = parsed["artifact_id"].as_str().unwrap_or("").trim();
            if artifact_id.is_empty() {
                return Some(respond_err(stream, 400, "artifact_id is required"));
            }
            let eff = effective_cfg(state, runtime);
            let candidate_content = parsed["candidate_content"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let res = if let Some(cand) = candidate_content {
                let kind = parsed["kind"]
                    .as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        state
                            .library
                            .lock()
                            .unwrap()
                            .get(artifact_id)
                            .map(|i| i.kind)
                            .unwrap_or_else(|| {
                                if artifact_id == self_improve::DEFAULT_IMPROVER_ID {
                                    "improver".to_string()
                                } else {
                                    "tool".to_string()
                                }
                            })
                    });
                let params = {
                    let p = string_list(&parsed["candidate_params"]);
                    if p.is_empty() {
                        string_list(&parsed["params"])
                    } else {
                        p
                    }
                };
                let note = parsed["note"]
                    .as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("API proposed improvement");
                let mut lib = state.library.lock().unwrap();
                self_improve::evaluate_and_maybe_promote(
                    &root_cfg.data_dir,
                    &eff,
                    Some(&mut lib),
                    artifact_id,
                    &kind,
                    cand,
                    params,
                    &person.id,
                    note,
                )
            } else {
                let mut lib = state.library.lock().unwrap();
                self_improve::propose_improvement_with_model(
                    &eff,
                    &root_cfg.data_dir,
                    Some(&mut lib),
                    artifact_id,
                )
            };
            match res {
                Ok(ev) => {
                    if ev.promoted {
                        connectors::dispatch_event_webhooks(
                            &root_cfg.data_dir,
                            "evolution.promoted",
                            ev.to_json(),
                        );
                    }
                    respond_json(stream, 200, &ev.to_json())
                }
                Err(e) if e == NO_KEY_ERROR => respond_err(stream, 503, &e),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/evolution/rollback") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let artifact_id = parsed["artifact_id"].as_str().unwrap_or("").trim();
            if artifact_id.is_empty() {
                return Some(respond_err(stream, 400, "artifact_id is required"));
            }
            let res = {
                let mut lib = state.library.lock().unwrap();
                self_improve::manual_rollback(&root_cfg.data_dir, Some(&mut lib), artifact_id)
            };
            match res {
                Ok(val) => {
                    connectors::dispatch_event_webhooks(
                        &root_cfg.data_dir,
                        "evolution.rolled_back",
                        json!({
                            "artifact_id": artifact_id,
                            "from_version": val["from_version"],
                            "to_version": val["to_version"],
                            "person_id": person.id,
                        }),
                    );
                    respond_json(stream, 200, &val)
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        // Per-Task Quality Bars (/api/quality-bars*)
        ("GET", "/api/quality-bars") => {
            let bars: Vec<Value> = quality_bar::load_quality_bars(&runtime.cfg.data_dir)
                .into_iter()
                .map(|b| b.to_json())
                .collect();
            respond_json(stream, 200, &json!({ "quality_bars": bars }))
        }
        ("GET", path) if path.starts_with("/api/quality-bars/") && !path.ends_with("/feedback") => {
            let family = percent_decode(path.trim_start_matches("/api/quality-bars/"))
                .trim()
                .to_string();
            match quality_bar::get_quality_bar(&runtime.cfg.data_dir, &family) {
                Some(bar) => respond_json(stream, 200, &json!({ "quality_bar": bar.to_json() })),
                None => respond_err(stream, 404, "quality bar not found"),
            }
        }
        ("POST", path)
            if path.starts_with("/api/quality-bars/") && path.ends_with("/feedback") =>
        {
            let raw_fam = path
                .strip_prefix("/api/quality-bars/")
                .and_then(|rest| rest.strip_suffix("/feedback"))
                .unwrap_or("");
            let family = percent_decode(raw_fam).trim().to_string();
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let action = parsed["action"].as_str().unwrap_or("");
            let note = parsed["note"].as_str();
            let add_criteria = string_list(&parsed["add_criteria"]);
            let recipe = parsed["recipe"].as_str();
            match quality_bar::apply_feedback(
                &runtime.cfg.data_dir,
                &family,
                action,
                note,
                add_criteria,
                recipe,
            ) {
                Ok(bar) => {
                    if bar.status == quality_bar::STATUS_ACCEPTED {
                        connectors::dispatch_event_webhooks(
                            &root_cfg.data_dir,
                            "quality_bar.accepted",
                            json!({
                                "task_family": bar.task_family,
                                "label": bar.label,
                                "person_id": person.id,
                                "accepted_run_id": bar.accepted_run_id,
                                "best_score": bar.best_score,
                                "accepted_recipe": bar.accepted_recipe,
                            }),
                        );
                    }
                    respond_json(stream, 200, &json!({ "quality_bar": bar.to_json() }))
                }
                Err(e) if e.starts_with("quality bar not found") => respond_err(stream, 404, &e),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        _ => return None,
    })
}

