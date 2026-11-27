use super::*;

/// Routes workspace file listing/content/upload/export, file history/restore, and audit bundles.
pub(super) fn try_route_files_and_audit(
    stream: &mut ConnStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/files/history") => {
            let rel = query_param(&req.query, "path").unwrap_or_default();
            let rel = rel.trim().to_string();
            if rel.is_empty() {
                let tracked = file_history::list_tracked_files(&runtime.cfg.data_dir);
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "person_id": person.id,
                        "count": tracked.len(),
                        "tracked_files": tracked,
                    }),
                ));
            }
            let mut payload = match file_history::history_payload_json(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &rel,
            ) {
                Ok(v) => v,
                Err(e) => {
                    let code = if is_history_security_error(&e) { 403 } else { 400 };
                    return Some(respond_err(stream, code, &e));
                }
            };
            let diff_pair = query_param(&req.query, "diff")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let (rev_a, rev_b) = if let Some(pair) = diff_pair {
                let (a, b) = pair
                    .split_once(',')
                    .map(|(x, y)| (x.trim().to_string(), y.trim().to_string()))
                    .unwrap_or((pair, "current".to_string()));
                (Some(a), if b.is_empty() { "current".to_string() } else { b })
            } else {
                let a = query_param(&req.query, "rev_a")
                    .or_else(|| query_param(&req.query, "from"))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let b = query_param(&req.query, "rev_b")
                    .or_else(|| query_param(&req.query, "to"))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "current".to_string());
                (a, b)
            };
            if let Some(a) = rev_a {
                match file_history::diff_revisions(
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &rel,
                    &a,
                    &rev_b,
                ) {
                    Ok(diff) => {
                        payload["diff"] = diff.to_json();
                    }
                    Err(e) => {
                        let code = if is_history_security_error(&e) {
                            403
                        } else if e.contains("not found") {
                            404
                        } else {
                            400
                        };
                        return Some(respond_err(stream, code, &e));
                    }
                }
            }
            respond_json(stream, 200, &payload)
        }
        ("POST", "/api/files/restore") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let rel = parsed["path"].as_str().unwrap_or("").trim().to_string();
            let rev_id = parsed["rev_id"]
                .as_str()
                .or_else(|| parsed["revision_id"].as_str())
                .or_else(|| parsed["id"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if rel.is_empty() || rev_id.is_empty() {
                return Some(respond_err(stream, 400, "path and rev_id are required"));
            }
            match file_history::restore_revision(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &rel,
                &rev_id,
            ) {
                Ok(res) => {
                    state.search_cache.invalidate(&person.id);
                    let mut out = res.to_json();
                    out["restored"] = json!(true);
                    respond_json(stream, 200, &out)
                }
                Err(e) => {
                    let code = if is_history_security_error(&e) {
                        403
                    } else if e.contains("not found") {
                        404
                    } else {
                        400
                    };
                    respond_err(stream, code, &e)
                }
            }
        }
        ("GET", "/api/audit/bundle") => {
            let format = query_param(&req.query, "format")
                .unwrap_or_else(|| "json".to_string())
                .to_ascii_lowercase();
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let security = tripwire::audit_section(&root_cfg.data_dir, &person.id);
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            if format == "md" || format == "markdown" {
                let mut bundle = audit_bundle::build_audit_bundle(&runtime.cfg, person);
                bundle["integrity"]["security_events_sha256"] = security["events_sha256"].clone();
                bundle["security"] = security;
                let mut md = audit_bundle::render_audit_bundle_markdown(&bundle);
                md.push_str(&render_security_markdown(&bundle["security"]));
                if policy.redact_host_paths {
                    md = crate::redact_host_paths(&md);
                }
                let mut headers = vec![("X-Content-Type-Options", "nosniff".to_string())];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-audit-bundle.md\"".to_string(),
                    ));
                }
                respond_bytes(
                    stream,
                    200,
                    "text/markdown; charset=utf-8",
                    md.as_bytes(),
                    &headers,
                )
            } else {
                let cached = state.search_cache.get_if_fresh(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                );
                let mut bytes = audit_bundle::build_audit_bundle_json_bytes_with_chain(
                    &runtime.cfg,
                    person,
                    &security,
                    cached
                        .as_ref()
                        .map(|idx| (idx.chain.as_slice(), idx.chain_valid, idx.head_hash.as_str())),
                );
                if policy.redact_host_paths {
                    let s = String::from_utf8_lossy(&bytes);
                    if s.contains("/home/")
                        || s.contains("/Users/")
                        || s.contains("/tmp/")
                        || s.contains("/var/folders/")
                        || s.contains("C:\\Users\\")
                        || s.contains("C:/Users/")
                    {
                        bytes = crate::redact_host_paths(&s).into_bytes();
                    }
                }
                let mut headers = vec![("X-Content-Type-Options", "nosniff".to_string())];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-audit-bundle.json\"".to_string(),
                    ));
                }
                respond_bytes(
                    stream,
                    200,
                    "application/json; charset=utf-8",
                    &bytes,
                    &headers,
                )
            }
        }
        ("GET", "/api/files/pack") => {
            let raw_files = query_param(&req.query, "files").unwrap_or_default();
            let title = query_param(&req.query, "title")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Client Deliverable Pack".to_string());
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let format = query_param(&req.query, "format")
                .unwrap_or_else(|| "md".to_string())
                .to_ascii_lowercase();
            let preflight_only = query_param(&req.query, "preflight")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let include_summary = query_param(&req.query, "summary")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let receipt_id = query_param(&req.query, "receipt_id")
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let req_client_id = query_param(&req.query, "client_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let req_engagement_id = query_param(&req.query, "engagement_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            let scope = audit_bundle::HandoffScope::resolve(
                &state.cfg.data_dir,
                &person.id,
                req_client_id.as_deref(),
                req_engagement_id.as_deref(),
                &policy,
            );
            let resolver = scope
                .as_ref()
                .map(|_| search::ScopeResolver::load(&state.cfg.data_dir, &runtime.cfg.workspace_dir));

            let mut requested: Vec<String> = raw_files
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .take(25)
                .collect();
            let explicit = !requested.is_empty();
            let idx_opt = if requested.is_empty() || !receipt_id.is_empty() || include_summary {
                Some(person_idx(state, person, runtime))
            } else {
                state.search_cache.get_if_fresh(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                )
            };
            if requested.is_empty() {
                if let Some(idx) = &idx_opt {
                    let (_, items) = idx.list_files_scoped(
                        0,
                        Some(15),
                        "",
                        scope.as_ref().and_then(|s| s.client_id.as_deref()),
                        scope.as_ref().and_then(|s| s.engagement_id.as_deref()),
                    );
                    for item in items {
                        if let Some(p) = item["path"].as_str() {
                            requested.push(p.to_string());
                        }
                    }
                }
            }
            let mut extra_receipts: Vec<(&str, u64)> = Vec::new();
            let mut summary_runs: Vec<&receipts::ChainedReceipt> = Vec::new();
            if let Some(idx) = &idx_opt {
                if !receipt_id.is_empty() {
                    if let Some(rcpt) = receipts::find_receipt(&idx.chain, &receipt_id) {
                        extra_receipts.push((rcpt.summary.as_str(), rcpt.ts));
                    }
                }
                let scan_limit = if include_summary { 15 } else { 8 };
                for rcpt in idx.chain.iter().rev() {
                    if let (Some(sc), Some(res)) = (scope.as_ref(), resolver.as_ref()) {
                        if !receipt_matches_handoff_scope(rcpt, sc, res) {
                            continue;
                        }
                    }
                    if include_summary && rcpt.finished && !rcpt.answered && summary_runs.len() < 15 {
                        summary_runs.push(rcpt);
                        if !explicit {
                            for f in &rcpt.files {
                                if let (Some(sc), Some(res)) = (scope.as_ref(), resolver.as_ref()) {
                                    if !res.matches_path(
                                        f,
                                        sc.client_id.as_deref(),
                                        sc.engagement_id.as_deref(),
                                    ) {
                                        continue;
                                    }
                                }
                                if requested.len() < 25 && !requested.iter().any(|x| x == f) {
                                    requested.push(f.clone());
                                }
                            }
                        }
                    }
                    if extra_receipts.len() < scan_limit
                        && !rcpt.summary.is_empty()
                        && rcpt.id != receipt_id
                        && rcpt.thread_id != receipt_id
                    {
                        extra_receipts.push((rcpt.summary.as_str(), rcpt.ts));
                    }
                    if summary_runs.len() >= 15 && extra_receipts.len() >= scan_limit {
                        break;
                    }
                }
            }
            let mut cross_client_findings: Vec<Value> = Vec::new();
            if let (Some(sc), Some(res)) = (scope.as_ref(), resolver.as_ref()) {
                for req_p in &requested {
                    if let Some(other_cid) = res.is_cross_client_path(
                        req_p,
                        sc.client_id.as_deref(),
                        sc.engagement_id.as_deref(),
                    ) {
                        cross_client_findings.push(json!({
                            "kind": "cross_client_path",
                            "file": req_p,
                            "line": 1,
                            "other_client_id": other_cid,
                            "excerpt": format!("Requested path '{req_p}' belongs to client '{other_cid}'"),
                        }));
                    }
                }
            }
            let preflight = match inspect_workspace_pack(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &requested,
                explicit,
                &extra_receipts,
                None,
            ) {
                Ok(res) => res,
                Err((status, err)) => return Some(respond_err(stream, status, &err)),
            };
            if let (Some(sc), Some(res)) = (scope.as_ref(), resolver.as_ref()) {
                let mut cc_scan = audit_bundle::HandoffLeakScan {
                    clean: true,
                    leak_count: 0,
                    host_path_count: 0,
                    secret_count: 0,
                    env_snippet_count: 0,
                    pii_count: 0,
                    cross_client_count: 0,
                    findings: Vec::new(),
                };
                for item in &preflight.items {
                    audit_bundle::scan_text_cross_client_into(
                        res,
                        &item.path,
                        &item.body,
                        sc.client_id.as_deref(),
                        sc.engagement_id.as_deref(),
                        &mut cc_scan,
                    );
                }
                for (rtext, _) in &extra_receipts {
                    audit_bundle::scan_text_cross_client_into(
                        res,
                        "receipt_summary",
                        rtext,
                        sc.client_id.as_deref(),
                        sc.engagement_id.as_deref(),
                        &mut cc_scan,
                    );
                }
                for f in cc_scan.findings {
                    cross_client_findings.push(f.to_json());
                }
            }
            let summary_gates = build_summary_run_gates(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &summary_runs,
            );
            if preflight_only {
                let mut out = preflight.to_json();
                if include_summary {
                    out["summary_runs_count"] = json!(summary_gates.len());
                    out["summary_runs"] = json!(summary_gates.iter().map(SummaryRunGate::to_json).collect::<Vec<_>>());
                }
                out["cross_client_count"] = json!(cross_client_findings.len());
                out["cross_client_findings"] = json!(cross_client_findings);
                out["scope"] = json!({
                    "client_id": scope.as_ref().and_then(|s| s.client_id.clone()),
                    "engagement_id": scope.as_ref().and_then(|s| s.engagement_id.clone()),
                });
                return Some(respond_json(stream, 200, &out));
            }
            let force_pack = query_param(&req.query, "force")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let strict_cross_block = policy.profile == "strict_client"
                || policy.strict_citation_gate
                || policy.isolate_active_engagement;
            if strict_cross_block && !force_pack && !cross_client_findings.is_empty() {
                return Some(respond_json(
                    stream,
                    409,
                    &json!({
                        "error": format!(
                            "Blocked by {} policy: {} cross-client path reference(s) detected in deliverable pack.",
                            policy.label,
                            cross_client_findings.len()
                        ),
                        "policy": policy.profile,
                        "cross_client_count": cross_client_findings.len(),
                        "findings": cross_client_findings,
                    }),
                ));
            }
            let preflight_ok = preflight.ok();
            let ready_count = preflight.ready_count;
            let gate_label = preflight.citation_gate();
            if policy.strict_citation_gate
                && !force_pack
                && (preflight.citations_stale > 0 || preflight.citations_broken > 0)
            {
                return Some(respond_json(
                    stream,
                    409,
                    &json!({
                        "error": format!(
                            "Blocked by {} policy: {} stale and {} broken [file:L##] citation(s) must be resolved before export.",
                            policy.label,
                            preflight.citations_stale,
                            preflight.citations_broken
                        ),
                        "policy": policy.profile,
                        "citation_gate": gate_label,
                        "citations_stale": preflight.citations_stale,
                        "citations_broken": preflight.citations_broken,
                    }),
                ));
            }
            let delta_a = query_param(&req.query, "delta_a").unwrap_or_default();
            let delta_b = query_param(&req.query, "delta_b").unwrap_or_default();
            let delta_file = query_param(&req.query, "delta_file").unwrap_or_default();
            let mut delta_md_opt: Option<String> = None;
            if !delta_a.trim().is_empty() && !delta_b.trim().is_empty() {
                let owned_idx;
                let idx = match &idx_opt {
                    Some(i) => i,
                    None => {
                        owned_idx = person_idx(state, person, runtime);
                        &owned_idx
                    }
                };
                let Some(ra) = receipts::find_receipt(&idx.chain, delta_a.trim()) else {
                    return Some(respond_err(stream, 404, "delta receipt a not found"));
                };
                let Some(rb) = receipts::find_receipt(&idx.chain, delta_b.trim()) else {
                    return Some(respond_err(stream, 404, "delta receipt b not found"));
                };
                let (_, delta_md) = build_receipt_compare(ra, rb);
                delta_md_opt = Some(delta_md);
            } else if !delta_file.trim().is_empty() {
                let rev_a = query_param(&req.query, "rev_a").unwrap_or_default();
                let rev_b = query_param(&req.query, "rev_b").unwrap_or_else(|| "current".to_string());
                match file_history::diff_revisions(
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    delta_file.trim(),
                    rev_a.trim(),
                    rev_b.trim(),
                ) {
                    Ok(diff) => {
                        let mut dmd = format!(
                            "## Verified Snapshot Delta (`{}`: `{}` -> `{}`)\n\n- **Changes:** +{} / -{} lines ({}% similarity)\n- **SHA-256:** `{}` -> `{}`\n",
                            diff.path,
                            diff.rev_a,
                            diff.rev_b,
                            diff.added,
                            diff.removed,
                            diff.similarity_pct,
                            &diff.sha256_a[..12.min(diff.sha256_a.len())],
                            &diff.sha256_b[..12.min(diff.sha256_b.len())]
                        );
                        if !diff.unified.trim().is_empty() {
                            dmd.push_str(&format!("\n```diff\n{}\n```\n", diff.unified.trim_end()));
                        }
                        delta_md_opt = Some(dmd);
                    }
                    Err(e) => {
                        let code = if is_history_security_error(&e) { 403 } else { 404 };
                        return Some(respond_err(stream, code, &e));
                    }
                }
            }
            let has_delta = delta_md_opt.is_some();
            let has_summary = !summary_gates.is_empty();
            let total_cits = preflight.citations.len();
            let summary_cits_hdr = format!("{}/{}", preflight.citations_verified, total_cits);
            let summary_stale_hdr = (preflight.citations_stale + preflight.citations_broken).to_string();
            if format == "html" {
                let (mut html, mut pack_sha) = build_workspace_handoff_html(
                    &preflight,
                    &person.name,
                    &title,
                    delta_md_opt.as_deref(),
                    &summary_gates,
                );
                if policy.redact_host_paths {
                    html = crate::redact_host_paths(&html);
                    pack_sha = receipts::sha256_hex(html.as_bytes());
                }
                let mut headers = vec![
                    ("X-Content-Type-Options", "nosniff".to_string()),
                    (
                        "Content-Security-Policy",
                        "default-src 'none'; style-src 'unsafe-inline'".to_string(),
                    ),
                    ("X-Ergon-Pack-SHA256", pack_sha),
                    (
                        "X-Ergon-Pack-Preflight",
                        if preflight_ok { "pass" } else { "warn" }.to_string(),
                    ),
                    ("X-Ergon-Pack-Citation-Gate", gate_label.to_string()),
                    ("X-Ergon-Pack-Ready-Count", ready_count.to_string()),
                    (
                        "X-Ergon-Pack-Delta",
                        if has_delta { "yes" } else { "none" }.to_string(),
                    ),
                    (
                        "X-Ergon-Pack-Summary",
                        if has_summary { "yes" } else { "none" }.to_string(),
                    ),
                    ("X-Ergon-Pack-Summary-Runs", summary_gates.len().to_string()),
                    ("X-Ergon-Pack-Summary-Citations", summary_cits_hdr),
                    ("X-Ergon-Pack-Summary-Stale", summary_stale_hdr),
                ];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-client-handoff.html\"".to_string(),
                    ));
                }
                return Some(respond_bytes(
                    stream,
                    200,
                    "text/html; charset=utf-8",
                    html.as_bytes(),
                    &headers,
                ));
            }
            let mut md = format_workspace_pack_markdown(
                &preflight,
                &person.name,
                &title,
                &summary_gates,
            );
            if let Some(delta_md) = delta_md_opt {
                md.push_str("\n---\n\n");
                md.push_str(&delta_md);
            }
            if policy.redact_host_paths {
                md = crate::redact_host_paths(&md);
            }
            let pack_sha = receipts::sha256_hex(md.as_bytes());
            let mut headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                ("X-Ergon-Pack-SHA256", pack_sha),
                (
                    "X-Ergon-Pack-Preflight",
                    if preflight_ok { "pass" } else { "warn" }.to_string(),
                ),
                ("X-Ergon-Pack-Citation-Gate", gate_label.to_string()),
                ("X-Ergon-Pack-Ready-Count", ready_count.to_string()),
                (
                    "X-Ergon-Pack-Delta",
                    if has_delta { "yes" } else { "none" }.to_string(),
                ),
                (
                    "X-Ergon-Pack-Summary",
                    if has_summary { "yes" } else { "none" }.to_string(),
                ),
                ("X-Ergon-Pack-Summary-Runs", summary_gates.len().to_string()),
                ("X-Ergon-Pack-Summary-Citations", summary_cits_hdr),
                ("X-Ergon-Pack-Summary-Stale", summary_stale_hdr),
            ];
            if download {
                headers.push((
                    "Content-Disposition",
                    "attachment; filename=\"ergon-client-pack.md\"".to_string(),
                ));
            }
            respond_bytes(
                stream,
                200,
                "text/markdown; charset=utf-8",
                md.as_bytes(),
                &headers,
            )
        }
        ("GET", "/api/workspace/policy") => {
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            respond_json(stream, 200, &policy.to_json(&person.id))
        }
        ("PUT", "/api/workspace/policy") | ("POST", "/api/workspace/policy") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let raw_profile = parsed["profile"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let isolate_opt = parsed["isolate_active_engagement"]
                .as_bool()
                .or_else(|| parsed["strict_engagement"].as_bool());
            if raw_profile.is_none() && isolate_opt.is_none() {
                return Some(respond_err(
                    stream,
                    400,
                    "profile or isolate_active_engagement is required",
                ));
            }
            match crate::save_workspace_policy_with_options(
                &runtime.cfg.data_dir,
                raw_profile,
                isolate_opt,
            ) {
                Ok(policy) => respond_json(stream, 200, &policy.to_json(&person.id)),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/verify/evidence") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let content = parsed["content"].as_str().unwrap_or("");
            if content.trim().is_empty() {
                return Some(respond_err(stream, 400, "evidence content is empty"));
            }
            let filename = parsed["filename"].as_str();
            let check_ws = parsed["check_workspace"].as_bool().unwrap_or(false);
            let ws_opt = if check_ws {
                Some(runtime.cfg.workspace_dir.as_path())
            } else {
                None
            };
            let res = audit_bundle::verify_evidence(content, filename, ws_opt);
            respond_json(stream, 200, &res.to_json())
        }
        ("POST", "/api/export/handoff") => {
            handle_export_handoff(stream, state, req, person, runtime)
        }
        ("POST", "/api/citations/verify") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let mut text = parsed["text"].as_str().unwrap_or("").to_string();
            let receipt_id = parsed["receipt_id"].as_str().unwrap_or("").trim();
            let mut receipt_ts = 0u64;
            if !receipt_id.is_empty() {
                let idx = person_idx(state, person, runtime);
                if let Some(rcpt) = receipts::find_receipt(&idx.chain, receipt_id) {
                    receipt_ts = rcpt.ts;
                    if text.is_empty() {
                        text = format!("{}\n{}", rcpt.task, rcpt.summary);
                    }
                }
            }
            let expected_map: std::collections::HashMap<String, String> = parsed["expected_shas"]
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            let exp_opt = (!expected_map.is_empty()).then_some(&expected_map);
            let mut citations = Vec::new();
            let mut seen = std::collections::HashMap::new();
            let mut verified = 0usize;
            let mut stale = 0usize;
            let mut broken = 0usize;
            collect_citations_into(
                Some(&runtime.cfg.data_dir),
                &runtime.cfg.workspace_dir,
                &text,
                receipt_ts,
                exp_opt,
                &mut citations,
                &mut seen,
                &mut verified,
                &mut stale,
                &mut broken,
            );
            let report = CitationReport {
                total: citations.len(),
                verified,
                stale,
                broken,
                citations,
            };
            respond_json(stream, 200, &report.to_json())
        }
        ("GET", "/api/files") => {
            let (offset, limit, q, mut client_id, mut engagement_id) = if req.query.is_empty() {
                (0usize, None, String::new(), None, None)
            } else {
                (
                    query_param(&req.query, "offset")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0),
                    query_param(&req.query, "limit").and_then(|s| s.parse::<usize>().ok()),
                    query_param(&req.query, "q").unwrap_or_default(),
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
                idx.write_files_json_scoped(
                    buf,
                    &person.id,
                    offset,
                    limit,
                    &q,
                    client_id.as_deref(),
                    engagement_id.as_deref(),
                );
            })
        }
        ("GET", "/api/files/content") => {
            let rel = query_param(&req.query, "path").unwrap_or_default();
            let rel = rel.trim().to_string();
            if rel.is_empty() {
                return Some(respond_err(stream, 400, "path is required"));
            }
            if rel.len() > 240 || rel.split('/').any(|s| s.len() > 200) {
                return Some(respond_err(stream, 400, "path is too long"));
            }
            if rel.contains('\0')
                || rel.contains('\\')
                || rel.contains(':')
                || rel.chars().any(|c| c.is_control())
                || rel.starts_with('/')
                || rel.to_ascii_lowercase().contains("%2e%2e")
                || rel.split('/').any(|seg| seg == ".." || seg == "." || seg.is_empty() || seg.starts_with('.'))
            {
                return Some(respond_err(stream, 403, "path escapes the workspace"));
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &rel, true) {
                Ok(p) => p,
                Err(e) => return Some(respond_err(stream, 403, &e),)};
            if full == runtime.cfg.workspace_dir || full.is_dir() {
                return Some(respond_err(stream, 400, "path must name a file inside the workspace"));
            }
            let md = match std::fs::metadata(&full) {
                Ok(m) => m,
                Err(_) => return Some(respond_err(stream, 404, "file not found"),)};
            const MAX_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;
            let download_requested = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            // Documents preview as their extracted text (a workbook one
            // sheet at a time); downloading still sends the original bytes.
            if let (false, Some(kind)) = (download_requested, docs::doc_kind(&rel)) {
                let sheet = query_param(&req.query, "sheet");
                let name = full.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string();
                let ext = full.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
                let mtime_ms = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let mut v = json!({
                    "path": rel, "name": name, "size": md.len(), "mtime_ms": mtime_ms, "ext": ext,
                });
                match docs::extract(&full, kind, sheet.as_deref()) {
                    Ok(ex) => {
                        v["is_binary"] = json!(false);
                        v["content"] = json!(ex.text);
                        v["extracted"] = json!({
                            "kind": kind.label(),
                            "format": ex.format,
                            "sheets": ex.sheets,
                            "sheet": ex.sheet,
                            "pages": ex.pages,
                        });
                    }
                    Err(e) => {
                        v["is_binary"] = json!(true);
                        v["content"] = json!("");
                        v["extract_error"] = json!(e);
                    }
                }
                return Some(respond_json(stream, 200, &v));
            }
            if md.len() > MAX_PREVIEW_BYTES && !download_requested {
                return Some(respond_err(stream, 413, "file exceeds the 2 MiB preview limit"));
            }
            let bytes = match std::fs::read(&full) {
                Ok(b) => b,
                Err(_) => return Some(respond_err(stream, 403, "file could not be read"),)};
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if download {
                let safe_name: String = name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' })
                    .collect();
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"{safe_name}\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                return Some(respond_bytes(stream, 200, mime_for_ext(&ext), &bytes, &headers));
            }
            let mtime_ms = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            match String::from_utf8(bytes) {
                Ok(content) => {
                    let sha256 = receipts::sha256_hex(content.as_bytes());
                    respond_json(
                        stream,
                        200,
                        &json!({
                            "path": rel,
                            "name": name,
                            "size": md.len(),
                            "mtime_ms": mtime_ms,
                            "ext": ext,
                            "sha256": sha256,
                            "is_binary": false,
                            "content": content,
                        }),
                    )
                }
                Err(_) => respond_json(
                    stream,
                    200,
                    &json!({
                        "path": rel,
                        "name": name,
                        "size": md.len(),
                        "mtime_ms": mtime_ms,
                        "ext": ext,
                        "is_binary": true,
                        "content": "",
                    }),
                ),
            }
        }
        ("PUT", "/api/files/content") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let raw_rel = parsed["path"].as_str().unwrap_or("");
            let content = match parsed["content"].as_str() {
                Some(s) => s,
                None => return Some(respond_err(stream, 400, "content is required")),
            };
            if content.len() as u64 > docs::MAX_DOC_BYTES {
                return Some(respond_err(stream, 413, "file exceeds the 25 MB limit"));
            }
            let expected = parsed["expected_sha256"].as_str();
            let reason = parsed["reason"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("studio-edit");
            let outcome = match file_history::atomic_checked_write_workspace(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                raw_rel,
                content.as_bytes(),
                expected,
                reason,
            ) {
                Ok(o) => o,
                Err(e) => return Some(respond_checked_write_err(stream, e)),
            };
            state.search_cache.invalidate(&person.id);
            let edit_summaries = {
                let mut lib = state.library.lock().unwrap();
                self_improve::increment_edits_for_run(
                    &state.cfg.data_dir,
                    Some(&mut lib),
                    &person.id,
                    &outcome.rel,
                )
            };
            for s in edit_summaries {
                if let Some(prev_v) = s.rolled_back_to {
                    connectors::dispatch_event_webhooks(
                        &state.cfg.data_dir,
                        "evolution.rolled_back",
                        json!({
                            "artifact_id": s.record.artifact_id,
                            "kind": s.record.artifact_kind,
                            "rolled_back_to": prev_v,
                            "reason": s.rollback_reason,
                            "person_id": person.id,
                        }),
                    );
                }
            }
            let mtime_ms = sandbox_path_resolving(&runtime.cfg.workspace_dir, &outcome.rel, true)
                .ok()
                .and_then(|p| std::fs::metadata(p).ok())
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            respond_json(
                stream,
                200,
                &json!({
                    "path": outcome.rel,
                    "size": outcome.size_bytes,
                    "mtime_ms": mtime_ms,
                    "sha256": outcome.new_sha256,
                    "rev_id": outcome.prev_rev_id,
                }),
            )
        }
        ("PATCH", "/api/files/content") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let raw_rel = parsed["path"].as_str().unwrap_or("");
            let Some(ops) = parsed["ops"].as_array().filter(|a| !a.is_empty()) else {
                return Some(respond_err(stream, 400, "ops array is required and must be non-empty"));
            };
            let expected = parsed["expected_sha256"].as_str();
            let outcome = match file_history::atomic_checked_patch_workspace(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                raw_rel,
                expected,
                "studio-patch",
                |orig_text| apply_patch_ops(orig_text, ops),
            ) {
                Ok(o) => o,
                Err(e) => return Some(respond_checked_write_err(stream, e)),
            };
            state.search_cache.invalidate(&person.id);
            let edit_summaries = {
                let mut lib = state.library.lock().unwrap();
                self_improve::increment_edits_for_run(
                    &state.cfg.data_dir,
                    Some(&mut lib),
                    &person.id,
                    &outcome.rel,
                )
            };
            for s in edit_summaries {
                if let Some(prev_v) = s.rolled_back_to {
                    connectors::dispatch_event_webhooks(
                        &state.cfg.data_dir,
                        "evolution.rolled_back",
                        json!({
                            "artifact_id": s.record.artifact_id,
                            "kind": s.record.artifact_kind,
                            "rolled_back_to": prev_v,
                            "reason": s.rollback_reason,
                            "person_id": person.id,
                        }),
                    );
                }
            }
            let stats = file_history::diff_texts(
                &format!("{} (before)", outcome.rel),
                &format!("{} (after)", outcome.rel),
                &outcome.orig_text,
                &outcome.new_text,
            );
            let rev_diff = file_history::RevisionDiff {
                path: outcome.rel.clone(),
                rev_a: outcome
                    .prev_rev_id
                    .clone()
                    .unwrap_or_else(|| "previous".to_string()),
                rev_b: "current".to_string(),
                sha256_a: outcome.orig_sha256,
                sha256_b: outcome.new_sha256.clone(),
                is_binary: false,
                added: stats.added,
                removed: stats.removed,
                unchanged: stats.unchanged,
                similarity_pct: stats.similarity_pct,
                hunks: stats.hunks,
                unified: stats.unified,
            };
            respond_json(
                stream,
                200,
                &json!({
                    "path": outcome.rel,
                    "applied": outcome.applied,
                    "size": outcome.new_text.len(),
                    "sha256": outcome.new_sha256,
                    "rev_id": outcome.prev_rev_id,
                    "diff": rev_diff.to_json(),
                }),
            )
        }
        ("POST", "/api/tables/query") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            match tables::execute_query(&runtime.cfg, &parsed) {
                Ok(res) => {
                    if res.saved_path.is_some() {
                        state.search_cache.invalidate(&person.id);
                    }
                    respond_json(stream, 200, &res.to_json())
                }
                Err(e) => {
                    let status = if e.contains("appears to be binary")
                        || e.contains("is not valid UTF-8")
                        || e.contains("cannot be parsed")
                    {
                        400
                    } else if is_history_security_error(&e) || e.contains("symlink") {
                        403
                    } else if e.contains("not found") {
                        404
                    } else {
                        400
                    };
                    respond_err(stream, status, &e)
                }
            }
        }
        ("POST", "/api/vault/aggregate") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let req_client_id = parsed["client_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    query_param(&req.query, "client_id")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                });
            let req_engagement_id = parsed["engagement_id"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    query_param(&req.query, "engagement_id")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                });
            let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
            let mut eff_client_id = req_client_id;
            let mut eff_engagement_id = req_engagement_id;
            let mut eff_prefix: Option<String> = None;
            if let Some(ref eid) = eff_engagement_id {
                let store = records::load_clients_store(&state.cfg.data_dir);
                if let Some(eng) = store.engagements.iter().find(|e| &e.id == eid) {
                    if !eng.workspace_prefix.is_empty() {
                        eff_prefix = Some(eng.workspace_prefix.clone());
                    }
                    if eff_client_id.is_none() {
                        eff_client_id = Some(eng.client_id.clone());
                    }
                }
            } else if eff_client_id.is_none() && policy.isolate_active_engagement {
                if let Some((eng, _)) = records::active_engagement_for(&state.cfg.data_dir, &person.id) {
                    eff_engagement_id = Some(eng.id.clone());
                    eff_client_id = Some(eng.client_id.clone());
                    if !eng.workspace_prefix.is_empty() {
                        eff_prefix = Some(eng.workspace_prefix.clone());
                    }
                }
            }
            let effective_args = match tools::scope_aggregate_args(
                &runtime.cfg,
                &parsed,
                eff_prefix.as_deref(),
                eff_client_id.as_deref(),
                eff_engagement_id.as_deref(),
            ) {
                Ok(a) => a,
                Err(e) => {
                    let status = if e.contains("blocked by engagement isolation") {
                        403
                    } else {
                        400
                    };
                    return Some(respond_err(stream, status, &e));
                }
            };
            match tables::execute_vault_aggregate(&runtime.cfg, &effective_args) {
                Ok(res) => {
                    if res.saved_path.is_some() {
                        state.search_cache.invalidate(&person.id);
                        search::SearchCache::global()
                            .invalidate(&runtime.cfg.data_dir.to_string_lossy());
                    }
                    respond_json(stream, 200, &res.to_json())
                }
                Err(e) => {
                    let status = if e.contains("appears to be binary")
                        || e.contains("is not valid UTF-8")
                        || e.contains("cannot be parsed")
                    {
                        400
                    } else if is_history_security_error(&e) || e.contains("symlink") {
                        403
                    } else if e.contains("not found") {
                        404
                    } else {
                        400
                    };
                    respond_err(stream, status, &e)
                }
            }
        }
        ("POST", "/api/files/export") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let has_sheets = parsed["sheets"].as_array().is_some_and(|a| !a.is_empty());
            let rel = parsed["path"]
                .as_str()
                .or_else(|| if has_sheets { parsed["output"].as_str() } else { None })
                .unwrap_or("")
                .trim()
                .to_string();
            if rel.is_empty() {
                return Some(respond_err(stream, 400, "path is required"));
            }
            let fallback_hdr = if parsed["brand_header"]
                .as_str()
                .map(str::trim)
                .is_none_or(|s| s.is_empty())
            {
                records::active_brand_header_for(&root_cfg.data_dir, &person.id)
            } else {
                None
            };
            let brand_hdr = parsed["brand_header"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .or(fallback_hdr.as_deref());
            match export_workspace_file_with_options(
                &runtime.cfg,
                &rel,
                parsed["format"].as_str(),
                parsed["output"].as_str(),
                brand_hdr,
                parsed["sheets"].as_array().map(|a| a.as_slice()),
                parsed["totals_row"].as_bool().unwrap_or(false),
            ) {
                Ok((out, bytes, label)) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(stream, 201, &json!({"path": out, "size": bytes, "kind": label, "source": rel}))
                }
                Err(e) => {
                    let status = if e.contains("escapes")
                        || e.contains("absolute")
                        || e.contains("symlink")
                        || e.contains("hidden")
                    {
                        403
                    } else {
                        400
                    };
                    respond_err(stream, status, &e)
                }
            }
        }
        ("POST", "/api/files/upload") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let raw_rel = parsed["path"]
                .as_str()
                .or_else(|| parsed["name"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw_rel.is_empty() {
                return Some(respond_err(stream, 400, "filename is required"));
            }
            if raw_rel.len() > 240 || raw_rel.split('/').any(|s| s.len() > 200) {
                return Some(respond_err(stream, 400, "filename is too long"));
            }
            if raw_rel.contains('\0')
                || raw_rel.contains('\\')
                || raw_rel.contains(':')
                || raw_rel.chars().any(|c| c.is_control())
                || raw_rel.starts_with('/')
                || raw_rel.to_ascii_lowercase().contains("%2e%2e")
                || raw_rel.split('/').any(|seg| seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('.'))
            {
                return Some(respond_err(stream, 403, "invalid workspace filename"));
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, false) {
                Ok(p) => p,
                Err(e) => return Some(respond_err(stream, 403, &e),)};
            if sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, true).is_err() {
                return Some(respond_err(stream, 403, "path escapes the workspace"));
            }
            if full == runtime.cfg.workspace_dir {
                return Some(respond_err(stream, 400, "filename must name a file"));
            }
            let encoding = parsed["encoding"].as_str().unwrap_or("utf8");
            let content_str = match parsed["content"].as_str() {
                Some(s) => s,
                None => {
                    return Some(respond_err(stream, 400, "content is required"))}
            };
            let bytes = if encoding.eq_ignore_ascii_case("base64") {
                match decode_base64(content_str) {
                    Ok(b) => b,
                    Err(e) => return Some(respond_err(stream, 400, &e),)}
            } else {
                content_str.as_bytes().to_vec()
            };
            if bytes.len() as u64 > docs::MAX_DOC_BYTES {
                return Some(respond_err(stream, 413, "files over 25 MB cannot be uploaded"));
            }
            if full.symlink_metadata().is_ok_and(|md| md.file_type().is_symlink()) {
                return Some(respond_err(stream, 403, "cannot overwrite symlink"));
            }
            let _ = file_history::record_revision(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &raw_rel,
                "upload",
            );
            if let Err(e) = atomic_write_bytes(&full, &bytes) {
                return Some(respond_json(
                    stream,
                    500,
                    &json!({"error": format!("save file failed: {e}")}),
                ));
            }
            state.search_cache.invalidate(&person.id);
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&raw_rel)
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let mtime_ms = std::fs::metadata(&full)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            respond_json(
                stream,
                201,
                &json!({
                    "path": raw_rel,
                    "name": name,
                    "size": bytes.len(),
                    "mtime_ms": mtime_ms,
                    "ext": ext,
                }),
            )
        }
        _ => return None,
    })
}

