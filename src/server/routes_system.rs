use super::*;

/// Routes UI pages, auth/OIDC/RBAC, server status/config, updates, people, library/suite, and security.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_route_system(
    stream: &mut ConnStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    principal: Option<&auth::AuthPrincipal>,
    auth_enforced: bool,
    client_ip: Option<std::net::IpAddr>,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let tls_enabled = state.tls.is_some();
    let max_age_secs = std::env::var("ERGON_SESSION_TTL_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(12 * 3600)
        .clamp(300, 30 * 86_400);
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET" | "HEAD", "/") => {
            serve_page(stream, state, req, person, ui::index_html(), auth_enforced)
        }
        ("GET" | "HEAD", "/phone") => {
            serve_page(stream, state, req, person, ui::index_html(), auth_enforced)
        }
        ("GET" | "HEAD", path) if path.starts_with("/chunks/") => serve_chunk(stream, req, path),
        ("GET" | "HEAD", "/api/health") => {
            let body = json!({
                "ok": true,
                "name": "ergon",
                "version": env!("CARGO_PKG_VERSION"),
                "auth_required": auth_enforced,
                "needs_setup": !state.auth.has_users(),
                "oidc_enabled": state.auth.oidc_enabled(),
                "tls": tls_enabled,
            })
            .to_string();
            if req.method == "HEAD" {
                respond_head(stream, 200, "application/json", body.len(), &[])
            } else {
                respond_bytes(stream, 200, "application/json", body.as_bytes(), &[])
            }
        }
        ("GET", "/api/auth/me") => respond_json(
            stream,
            200,
            &json!({
                "authenticated": principal.is_some(),
                "auth_required": auth_enforced,
                "needs_setup": !state.auth.has_users(),
                "oidc_enabled": state.auth.oidc_enabled(),
                "tls": tls_enabled,
                "user": principal.map(auth::AuthPrincipal::to_json),
                "roles": auth::Role::ALL,
            }),
        ),
        ("POST", "/api/auth/setup") => {
            if state.auth.has_users() {
                return Some(respond_err(
                    stream,
                    403,
                    "initial admin setup already completed",
                ));
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let username = parsed["username"].as_str().unwrap_or("").trim();
            let password = parsed["password"].as_str().unwrap_or("");
            let display_name = parsed["display_name"]
                .as_str()
                .or_else(|| parsed["name"].as_str());
            let email = parsed["email"].as_str();
            let user = match state.auth.setup_initial_admin(
                username,
                password,
                display_name,
                email,
                &state.people,
                root_cfg,
            ) {
                Ok(u) => u,
                Err(msg) => {
                    let st = if msg.contains("already completed") { 403 } else { 400 };
                    return Some(respond_err(stream, st, &msg));
                }
            };
            let linked_person = person_by_id(state, &user.person_id);
            let _ = runtime_for(state, &linked_person);
            let (raw_token, expires_at_ms) = match state.auth.create_session(&user, "password") {
                Ok(s) => s,
                Err(e) => return Some(respond_err(stream, 500, &e)),
            };
            let cookie = auth::session_cookie_header(&raw_token, tls_enabled, max_age_secs);
            let body = json!({
                "ok": true,
                "user": user.to_public_json(),
                "token": raw_token,
                "expires_at_ms": expires_at_ms,
            })
            .to_string();
            respond_h(
                stream,
                201,
                "application/json",
                &body,
                &[("Set-Cookie", cookie)],
            )
        }
        ("POST", "/api/auth/login") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let username = parsed["username"]
                .as_str()
                .or_else(|| parsed["email"].as_str())
                .or_else(|| parsed["identifier"].as_str())
                .unwrap_or("");
            let password = parsed["password"].as_str().unwrap_or("");
            let client_key = client_ip
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "local".to_string());
            match state.auth.login_local(username, password, &client_key) {
                Ok((user, raw_token, expires_at_ms)) => {
                    let linked_person = person_by_id(state, &user.person_id);
                    let _ = runtime_for(state, &linked_person);
                    let cookie = auth::session_cookie_header(&raw_token, tls_enabled, max_age_secs);
                    let body = json!({
                        "ok": true,
                        "user": user.to_public_json(),
                        "token": raw_token,
                        "expires_at_ms": expires_at_ms,
                    })
                    .to_string();
                    respond_h(
                        stream,
                        200,
                        "application/json",
                        &body,
                        &[("Set-Cookie", cookie)],
                    )
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("POST", "/api/auth/logout") => {
            if let Some(tok) = auth::extract_bearer_token(req.authorization.as_deref())
                .or(req.session.as_deref())
            {
                state.auth.revoke_session(tok);
            }
            let cookie = auth::clear_session_cookie_header(tls_enabled);
            respond_h(
                stream,
                200,
                "application/json",
                r#"{"ok":true}"#,
                &[
                    ("Set-Cookie", cookie),
                    (
                        "Clear-Site-Data",
                        "\"cookies\", \"storage\", \"cache\"".to_string(),
                    ),
                ],
            )
        }
        ("POST", "/api/auth/password") => {
            let Some(p) = principal else {
                return Some(respond_err(stream, 401, "authentication required"));
            };
            if p.is_break_glass {
                return Some(respond_err(
                    stream,
                    400,
                    "break-glass sessions cannot change a user password via /api/auth/password; use PATCH /api/auth/users/:id",
                ));
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let cur = parsed["current_password"].as_str().unwrap_or("");
            let next = parsed["new_password"].as_str().unwrap_or("");
            match state.auth.change_own_password(
                &p.user_id,
                cur,
                next,
                p.session_token_hash.as_deref(),
            ) {
                Ok(()) => respond_json(stream, 200, &json!({"ok": true})),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/auth/users") => respond_json(
            stream,
            200,
            &json!({
                "users": state.auth.list_users_public(),
                "roles": auth::Role::ALL,
            }),
        ),
        ("POST", "/api/auth/users") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            let username = parsed["username"].as_str().unwrap_or("");
            let password = parsed["password"].as_str().unwrap_or("");
            let role = parsed["role"].as_str().unwrap_or("member");
            let display_name = parsed["display_name"]
                .as_str()
                .or_else(|| parsed["name"].as_str());
            let email = parsed["email"].as_str();
            match state.auth.create_user(
                username,
                password,
                role,
                display_name,
                email,
                &state.people,
                root_cfg,
            ) {
                Ok(user) => {
                    let linked_person = person_by_id(state, &user.person_id);
                    let _ = runtime_for(state, &linked_person);
                    respond_json(stream, 201, &json!({"user": user.to_public_json()}))
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("PATCH", path) if path.starts_with("/api/auth/users/") => {
            let raw_id = path.strip_prefix("/api/auth/users/").unwrap_or("").trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "user not found"));
            }
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match state.auth.update_user(&id, &parsed, &state.people, root_cfg) {
                Ok(user) => respond_json(stream, 200, &json!({"user": user.to_public_json()})),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("DELETE", path) if path.starts_with("/api/auth/users/") => {
            let raw_id = path.strip_prefix("/api/auth/users/").unwrap_or("").trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_err(stream, 404, "user not found"));
            }
            match state.auth.delete_user(&id) {
                Ok(user) => respond_json(
                    stream,
                    200,
                    &json!({"ok": true, "deleted": user.to_public_json()}),
                ),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/auth/oidc/config") => {
            respond_json(stream, 200, &state.auth.oidc_config_public())
        }
        ("PUT", "/api/auth/oidc/config") | ("POST", "/api/auth/oidc/config") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match state.auth.update_oidc_config(&parsed) {
                Ok(cfg_json) => respond_json(stream, 200, &cfg_json),
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/auth/oidc/login") => {
            let next = query_param(&req.query, "next").unwrap_or_else(|| "/".to_string());
            let scheme = if tls_enabled { "https" } else { "http" };
            let host = req.host.as_deref().unwrap_or("127.0.0.1");
            let origin = format!("{scheme}://{host}");
            match state.auth.start_oidc_login(&next, &origin) {
                Ok(auth_url) => {
                    if query_param(&req.query, "json").as_deref() == Some("1") {
                        respond_json(stream, 200, &json!({"authorization_url": auth_url}))
                    } else {
                        respond_h(
                            stream,
                            302,
                            "text/plain; charset=utf-8",
                            "",
                            &[("Location", auth_url)],
                        )
                    }
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/auth/oidc/callback") => {
            if let Some(err) = query_param(&req.query, "error").filter(|s| !s.is_empty()) {
                let desc = query_param(&req.query, "error_description").unwrap_or_default();
                let msg = if desc.is_empty() {
                    format!("OIDC provider rejected sign-in: {err}")
                } else {
                    format!("OIDC provider rejected sign-in: {err} ({desc})")
                };
                return Some(respond_err(stream, 400, &msg));
            }
            let code = query_param(&req.query, "code").unwrap_or_default();
            let state_param = query_param(&req.query, "state").unwrap_or_default();
            match state
                .auth
                .complete_oidc_callback(&code, &state_param, &state.people, root_cfg)
            {
                Ok((user, raw_token, expires_at_ms, redirect_after)) => {
                    let linked_person = person_by_id(state, &user.person_id);
                    let _ = runtime_for(state, &linked_person);
                    let cookie = auth::session_cookie_header(&raw_token, tls_enabled, max_age_secs);
                    if query_param(&req.query, "json").as_deref() == Some("1") {
                        let body = json!({
                            "ok": true,
                            "user": user.to_public_json(),
                            "token": raw_token,
                            "expires_at_ms": expires_at_ms,
                            "redirect": redirect_after,
                        })
                        .to_string();
                        respond_h(
                            stream,
                            200,
                            "application/json",
                            &body,
                            &[("Set-Cookie", cookie)],
                        )
                    } else {
                        respond_h(
                            stream,
                            302,
                            "text/plain; charset=utf-8",
                            "",
                            &[("Location", redirect_after), ("Set-Cookie", cookie)],
                        )
                    }
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/version") => {
            respond_bytes(stream, 200, "application/json", VERSION_JSON_BYTES, &[])
        }
        ("GET", "/api/update") => {
            let is_admin = update_admin_ok(state, req, principal, auth_enforced);
            respond_json(
                stream,
                200,
                &state
                    .updater
                    .snapshot_for_caller(is_admin, state.gate_token.is_some()),
            )
        }
        ("POST", "/api/update/check") => {
            let is_admin = update_admin_ok(state, req, principal, auth_enforced);
            if !is_admin {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "admin privileges are required to check for updates on a shared host"}),
                ));
            }
            let _ = state.updater.check_now();
            respond_json(
                stream,
                200,
                &state
                    .updater
                    .snapshot_for_caller(true, state.gate_token.is_some()),
            )
        }
        ("POST", "/api/update/mode") => {
            let is_admin = update_admin_ok(state, req, principal, auth_enforced);
            if !is_admin {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "admin privileges are required to change update mode"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let Some(raw_mode) = parsed["mode"].as_str() else {
                return Some(respond_err(stream, 400, "mode is required (off, notify, or auto)"));
            };
            let Some(mode) = update::Mode::parse_strict(raw_mode) else {
                return Some(respond_err(stream, 400, "mode must be off, notify, or auto"));
            };
            match state.updater.set_mode(mode) {
                Ok(_) => respond_json(
                    stream,
                    200,
                    &state
                        .updater
                        .snapshot_for_caller(true, state.gate_token.is_some()),
                ),
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("POST", "/api/update/apply")
        | ("POST", "/api/update/install")
        | ("POST", "/api/update/restart") => {
            // Local loopback admins (when ungated) and authenticated admins behind a
            // hosting gate may install updates and restart; non-admins and unauthenticated
            // LAN callers are strictly refused with 403.
            let is_admin = update_admin_ok(state, req, principal, auth_enforced);
            if !is_admin {
                let msg = if state.gate_token.is_some() {
                    "admin privileges are required to install updates or restart a gated Ergon host"
                } else {
                    "updating is only allowed from this machine, not through a shared address"
                };
                return Some(respond_json(stream, 403, &json!({"error": msg})));
            }
            if req.path == "/api/update/apply" || req.path == "/api/update/install" {
                match state.updater.apply_authorized() {
                    Ok(v) => {
                        let mut snap = state
                            .updater
                            .snapshot_for_caller(true, state.gate_token.is_some());
                        snap["applied"] = json!(v);
                        snap["installed"] = json!(v);
                        respond_json(stream, 200, &snap)
                    }
                    Err(e) => respond_err(stream, 409, &e),
                }
            } else if state.updater.snapshot()["staged"].as_str().is_none() {
                respond_err(stream, 409, "no update is waiting for a restart")
            } else if RUNS_IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                respond_err(stream, 409, "a task is still running; restart when it has finished")
            } else {
                let _ = respond_json(stream, 200, &json!({"restarting": true}));
                let up = state.updater.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    if let Err(e) = up.restart_authorized() {
                        crate::logging::error("ergon::server", &format!("ergon: restart failed: {e}"));
                    }
                });
                Ok(())
            }
        }
        ("GET", "/api/status") => {
            let mut recovered: Vec<String> = Vec::new();
            if auth_enforced && !principal.is_some_and(|p| p.role.is_admin()) {
                let st = runtime.store.lock().unwrap();
                if let Some(s) = &st.recovered {
                    recovered.push(s.clone());
                }
            } else {
                {
                    let people = state.people.read().unwrap();
                    if !people.recovered.is_empty() {
                        recovered.extend(people.recovered.iter().cloned());
                    }
                }
                {
                    let lib = state.library.lock().unwrap();
                    if let Some(s) = &lib.recovered {
                        recovered.push(s.clone());
                    }
                }
                {
                    let runtimes = state.runtimes.read().unwrap();
                    for rt in runtimes.values() {
                        let st = rt.store.lock().unwrap();
                        if let Some(s) = &st.recovered {
                            recovered.push(s.clone());
                        }
                    }
                }
            }
            if let Some(s) = &state.live_runs_recovered {
                recovered.push(s.clone());
            }
            if recovered.len() > 1 {
                recovered.sort();
                recovered.dedup();
            }
            let (model_configured, model_id, model_base_url, auto_detected) = {
                let live = state.model_cfg.lock().unwrap();
                (
                    crate::model_configured_for(&live.model_base_url, live.api_key.is_some()),
                    live.model_id.clone(),
                    live.model_base_url.clone(),
                    live.auto_detected,
                )
            };
            let is_loopback = is_loopback_url(&model_base_url);
            let oidc_enabled = state.auth.oidc_enabled();
            respond_json_with(stream, 200, |buf| {
                buf.extend_from_slice(br#"{"version":"#);
                search::write_json_str(buf, env!("CARGO_PKG_VERSION"));
                buf.extend_from_slice(br#","model_configured":"#);
                search::write_json_bool(buf, model_configured);
                buf.extend_from_slice(br#","model":"#);
                search::write_json_str(buf, &model_id);
                buf.extend_from_slice(br#","model_base_url":"#);
                search::write_json_str(buf, &model_base_url);
                buf.extend_from_slice(br#","is_loopback_model":"#);
                search::write_json_bool(buf, is_loopback);
                buf.extend_from_slice(br#","auto_detected":"#);
                search::write_json_bool(buf, auto_detected);
                buf.extend_from_slice(br#","auth_required":"#);
                search::write_json_bool(buf, auth_enforced);
                buf.extend_from_slice(br#","tls":"#);
                search::write_json_bool(buf, tls_enabled);
                buf.extend_from_slice(
                    br#","capabilities":{"sse":true,"skills":true,"routines":true,"playbooks":true,"pipelines":true,"search":true,"receipts_export":true,"cancel":true,"scheduler":"#,
                );
                search::write_json_bool(buf, state.scheduler_enabled);
                buf.extend_from_slice(
                    br#","file_history":true,"audit_bundle":true,"backups":true,"tripwire":true,"file_edit":true,"table_query":true,"pdf_export":true,"connectors":true,"mcp":true,"clients":true,"metering":true,"vault_search":true,"client_crm":true,"time_entries":true,"invoicing":true,"run_resume":true,"diagnostics":true,"help":true,"auth":true,"oidc":"#,
                );
                search::write_json_bool(buf, oidc_enabled);
                buf.extend_from_slice(br#","tls":"#);
                search::write_json_bool(buf, tls_enabled);
                buf.extend_from_slice(br#"},"scheduler_enabled":"#);
                search::write_json_bool(buf, state.scheduler_enabled);
                buf.extend_from_slice(br#","person":{"id":"#);
                search::write_json_str(buf, &person.id);
                buf.extend_from_slice(br#","name":"#);
                search::write_json_str(buf, &person.name);
                buf.extend_from_slice(br#","role":"#);
                search::write_json_str(buf, &person.role);
                buf.extend_from_slice(br#"},"store_recovered":["#);
                for (i, item) in recovered.iter().enumerate() {
                    if i > 0 {
                        buf.push(b',');
                    }
                    search::write_json_str(buf, item);
                }
                buf.extend_from_slice(b"]}");
            })
        }
        ("GET", "/api/config") => {
            if !auth_enforced
                && (state.gate_token.is_some()
                    || !host_allowed(req.host.as_deref().unwrap_or(""), &[]))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "model settings are only available from this machine, not through a shared address"}),
                ));
            }
            let eff = effective_cfg(state, runtime);
            let live_meta = state.model_cfg.lock().unwrap().clone();
            let gateways: Vec<Value> = detect_local_gateways()
                .into_iter()
                .map(|g| g.to_json())
                .collect();
            let model_pricing = records::effective_model_pricing_json(&root_cfg.data_dir);
            respond_json(
                stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": live_meta.auto_detected,
                    "detected_gateways": gateways,
                    "model_pricing": model_pricing,
                }),
            )
        }
        ("PUT", "/api/config") | ("POST", "/api/config") => {
            if !auth_enforced
                && (state.gate_token.is_some()
                    || !host_allowed(req.host.as_deref().unwrap_or(""), &[]))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "changing model settings is only allowed from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let has_pricing = parsed.get("model_pricing").is_some_and(|v| !v.is_null());
            let has_model_fields = parsed.get("model_base_url").is_some_and(|v| !v.is_null())
                || parsed.get("model_id").is_some_and(|v| !v.is_null())
                || parsed.get("api_key").is_some_and(|v| !v.is_null())
                || parsed["clear_api_key"].as_bool().unwrap_or(false);

            if !has_pricing || has_model_fields {
                let base = parsed["model_base_url"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches('/')
                    .to_string();
                let model_id = parsed["model_id"].as_str().unwrap_or("").trim().to_string();
                if !(base.starts_with("http://") || base.starts_with("https://"))
                    || parse_url_host_port(&base).is_none()
                {
                    return Some(respond_err(
                        stream,
                        400,
                        "model_base_url must be a valid http:// or https:// URL",
                    ));
                }
                if model_id.is_empty() {
                    return Some(respond_err(stream, 400, "model_id is required"));
                }
                let clear_key = parsed["clear_api_key"].as_bool().unwrap_or(false);
                let next_key = if clear_key {
                    None
                } else if let Some(raw_k) = parsed["api_key"].as_str() {
                    let k = raw_k.trim().to_string();
                    if k.is_empty() {
                        state.model_cfg.lock().unwrap().api_key.clone()
                    } else {
                        Some(k)
                    }
                } else {
                    state.model_cfg.lock().unwrap().api_key.clone()
                };
                let saved = SavedModelConfig {
                    model_base_url: base.clone(),
                    model_id: model_id.clone(),
                    api_key: next_key.clone(),
                };
                if let Err(e) = save_model_config(&root_cfg.data_dir, &saved) {
                    return Some(respond_err(stream, 500, &e));
                }
                {
                    let mut live = state.model_cfg.lock().unwrap();
                    live.model_base_url = base;
                    live.model_id = model_id;
                    live.api_key = next_key;
                    live.auto_detected = false;
                }
            }
            if has_pricing {
                if let Err(e) = records::save_model_pricing_overrides(
                    &root_cfg.data_dir,
                    &parsed["model_pricing"],
                ) {
                    return Some(respond_err(stream, 400, &e));
                }
            }
            let eff = effective_cfg(state, runtime);
            let live_meta = state.model_cfg.lock().unwrap().clone();
            let model_pricing = records::effective_model_pricing_json(&root_cfg.data_dir);
            respond_json(
                stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": live_meta.auto_detected,
                    "model_pricing": model_pricing,
                }),
            )
        }
        ("GET", "/api/people") => {
            if auth_enforced && !principal.is_some_and(|p| p.role.is_admin()) {
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "people": [person.to_json()],
                        "active": person.id,
                    }),
                ));
            }
            let people = state.people.read().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(person.id);
            respond_json(stream, 200, &v)
        }
        ("POST", "/api/people") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("").trim().to_string();
            if name.is_empty() {
                return Some(respond_err(stream, 400, "name is required"));
            }
            let role = parsed["role"].as_str().unwrap_or("").trim().to_string();
            let person = {
                let mut people = state.people.write().unwrap();
                let p = people.add(&name, &role);
                people.save(root_cfg);
                p
            };
            // Create their dirs + seeded memory now, so the person is
            // real on disk from this moment.
            let _ = runtime_for(state, &person);
            respond_json(stream, 201, &person.to_json())
        }
        ("POST", "/api/active-person") | ("POST", "/api/people/switch") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"]
                .as_str()
                .or_else(|| parsed["person_id"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if auth_enforced {
                let Some(p) = principal else {
                    return Some(respond_err(stream, 401, "authentication required"));
                };
                if !p.role.is_admin() && id != p.home_person_id {
                    return Some(respond_err(
                        stream,
                        403,
                        "forbidden: non-admin users cannot switch into another person's workspace",
                    ));
                }
                {
                    let people = state.people.read().unwrap();
                    if !people.people.iter().any(|item| item.id == id) {
                        return Some(respond_err(stream, 404, "person not found"));
                    }
                }
                if p.role.is_admin() {
                    if let Some(ref tok_hash) = p.session_token_hash {
                        if let Err(e) = state.auth.switch_admin_session_person(tok_hash, &id) {
                            return Some(respond_err(stream, 500, &e));
                        }
                    }
                }
                if p.role.is_admin() {
                    let people = state.people.read().unwrap();
                    let mut v = people.to_json();
                    v["active"] = json!(id);
                    return Some(respond_json(stream, 200, &v));
                }
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "people": [person.to_json()],
                        "active": person.id,
                    }),
                ));
            }
            {
                let people = state.people.read().unwrap();
                if !people.people.iter().any(|p| p.id == id) {
                    return Some(respond_err(stream, 404, "person not found"));
                }
            }
            // Legacy single-user loopback mode (zero accounts, no TLS):
            // move only this browser session, or the global pointer for
            // cookie-less API clients.
            let mut set_cookie: Option<String> = None;
            if session_known(state, &req.session) {
                let sid = req.session.clone().unwrap();
                insert_session(state, sid, id.clone());
            } else {
                {
                    let mut people = state.people.write().unwrap();
                    people.active = id.clone();
                    people.save(root_cfg);
                }
                let sid = new_session_id();
                insert_session(state, sid.clone(), id.clone());
                set_cookie = Some(format!(
                    "ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"
                ));
            }
            let people = state.people.read().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(id);
            match set_cookie {
                Some(cookie) => respond_h(
                    stream,
                    200,
                    "application/json",
                    &v.to_string(),
                    &[("Set-Cookie", cookie)],
                ),
                None => respond_json(stream, 200, &v),
            }
        }
        ("GET", "/api/memory") => {
            let content = runtime.cfg.read_memory();
            let health = skills::audit_memory_health(&runtime.cfg);
            respond_json(
                stream,
                200,
                &json!({"person_id": person.id, "content": content, "health": health}),
            )
        }
        ("GET", "/api/memory/health") => {
            let mut health = skills::audit_memory_health(&runtime.cfg);
            if let Some(obj) = health.as_object_mut() {
                obj.insert("person_id".to_string(), json!(person.id));
            }
            respond_json(stream, 200, &health)
        }
        ("POST", "/api/memory/heal") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let mode = parsed["mode"].as_str().unwrap_or("all");
            match skills::heal_memory(&runtime.cfg, mode) {
                Ok(mut out) => {
                    state.search_cache.invalidate(&person.id);
                    if let Some(obj) = out.as_object_mut() {
                        obj.insert("person_id".to_string(), json!(person.id));
                    }
                    respond_json(stream, 200, &out)
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", "/api/library") => {
            let lib = state.library.lock().unwrap();
            respond_json(
                stream,
                200,
                &json!({
                    "items": lib.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
                }),
            )
        }
        ("GET", "/api/plugins") => {
            respond_json(stream, 200, &crate::plugins::list_plugins_json(&runtime.cfg.data_dir))
        }
        ("POST", "/api/plugins/toggle") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["plugin_id"].as_str().unwrap_or("");
            let enabled = parsed["enabled"].as_bool();
            let preferred_mode = parsed["preferred_mode"].as_str();
            match crate::plugins::toggle_plugin(&runtime.cfg.data_dir, id, enabled, preferred_mode) {
                Ok(plugin) => respond_json(stream, 200, &json!({ "plugin": plugin })),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/plugins/action") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let plugin_id = parsed["plugin_id"].as_str().unwrap_or("");
            let action = parsed["action"].as_str().unwrap_or("");
            let mode = parsed["mode"].as_str().filter(|s| !s.is_empty() && *s != "auto");
            let thread_id = parsed["thread_id"].as_str().unwrap_or("");
            let args = if parsed["args"].is_null() {
                json!({})
            } else {
                parsed["args"].clone()
            };
            let plugin_name = crate::plugins::find_plugin(plugin_id)
                .map(|p| p.name)
                .unwrap_or(plugin_id);
            let run_id = format!("plugin-{}", records::now_millis());
            state.events.publish(
                "tool_start",
                &person.id,
                &run_id,
                thread_id,
                json!({
                    "tool": "run_plugin_action",
                    "plugin_id": plugin_id,
                    "plugin_name": plugin_name,
                    "action": action,
                    "mode": mode.unwrap_or("local"),
                    "label": format!("{plugin_name} · {action}"),
                    "ts_ms": records::now_millis(),
                }),
            );
            match crate::plugins::execute_plugin_action(
                &runtime.cfg,
                plugin_id,
                action,
                mode,
                &args,
                thread_id,
            ) {
                Ok((output, resolved_mode)) => {
                    state.events.publish(
                        "tool_done",
                        &person.id,
                        &run_id,
                        thread_id,
                        json!({
                            "tool": "run_plugin_action",
                            "plugin_id": plugin_id,
                            "plugin_name": plugin_name,
                            "action": action,
                            "mode": resolved_mode,
                            "ok": true,
                            "label": format!("{plugin_name} · {action}"),
                            "summary": crate::truncate(&output, 240),
                            "ts_ms": records::now_millis(),
                        }),
                    );
                    respond_json(
                        stream,
                        200,
                        &json!({
                            "ok": true,
                            "mode": resolved_mode,
                            "output": output,
                        }),
                    )
                }
                Err(e) => {
                    state.events.publish(
                        "error",
                        &person.id,
                        &run_id,
                        thread_id,
                        json!({
                            "tool": "run_plugin_action",
                            "plugin_id": plugin_id,
                            "plugin_name": plugin_name,
                            "action": action,
                            "label": format!("{plugin_name} · {action}"),
                            "summary": crate::truncate(&e, 240),
                            "ts_ms": records::now_millis(),
                        }),
                    );
                    respond_err(stream, 400, &e)
                }
            }
        }
        ("GET", "/api/plugins/oauth/apps") => {
            let scheme = if tls_enabled { "https" } else { "http" };
            let host = req.host.as_deref().unwrap_or("127.0.0.1");
            let origin = format!("{scheme}://{host}");
            respond_json(
                stream,
                200,
                &crate::plugin_cloud::list_oauth_suites_json(
                    &root_cfg.data_dir,
                    &runtime.cfg.data_dir,
                    &origin,
                ),
            )
        }
        ("PUT" | "POST", "/api/plugins/oauth/apps") => {
            let parsed: Value = match serde_json::from_slice(&req.body) {
                Ok(v) => v,
                Err(_) => return Some(respond_err(stream, 400, "request body is not valid JSON")),
            };
            match crate::plugin_cloud::save_oauth_app_config(&root_cfg.data_dir, &parsed) {
                Ok(()) => {
                    let scheme = if tls_enabled { "https" } else { "http" };
                    let host = req.host.as_deref().unwrap_or("127.0.0.1");
                    let origin = format!("{scheme}://{host}");
                    respond_json(
                        stream,
                        200,
                        &crate::plugin_cloud::list_oauth_suites_json(
                            &root_cfg.data_dir,
                            &runtime.cfg.data_dir,
                            &origin,
                        ),
                    )
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET" | "POST", "/api/plugins/oauth/start") => {
            let parsed = if req.method == "POST" {
                serde_json::from_slice::<Value>(&req.body).unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            let provider = parsed["provider"]
                .as_str()
                .map(str::to_string)
                .or_else(|| query_param(&req.query, "provider"))
                .unwrap_or_default();
            let return_to = parsed["return_to"]
                .as_str()
                .map(str::to_string)
                .or_else(|| query_param(&req.query, "return_to"))
                .unwrap_or_else(|| "/".to_string());
            let scheme = if tls_enabled { "https" } else { "http" };
            let host = req.host.as_deref().unwrap_or("127.0.0.1");
            let origin = format!("{scheme}://{host}");
            match crate::plugin_cloud::start_oauth_flow(
                &root_cfg.data_dir,
                &person.id,
                &runtime.cfg.data_dir,
                &provider,
                &origin,
                &return_to,
            ) {
                Ok(auth_url) => {
                    if req.method == "POST"
                        || query_param(&req.query, "json").as_deref() == Some("1")
                    {
                        respond_json(
                            stream,
                            200,
                            &json!({ "ok": true, "authorization_url": auth_url }),
                        )
                    } else {
                        respond_h(
                            stream,
                            302,
                            "text/plain; charset=utf-8",
                            "",
                            &[("Location", auth_url)],
                        )
                    }
                }
                Err((status, msg)) => respond_err(stream, status, &msg),
            }
        }
        ("GET", "/api/plugins/oauth/callback") => {
            if let Some(err) = query_param(&req.query, "error").filter(|s| !s.is_empty()) {
                let desc = query_param(&req.query, "error_description").unwrap_or_default();
                let msg = if desc.is_empty() {
                    format!("OAuth sign-in rejected: {err}")
                } else {
                    format!("OAuth sign-in rejected: {err} ({desc})")
                };
                if query_param(&req.query, "json").as_deref() == Some("1") {
                    return Some(respond_err(stream, 400, &msg));
                }
                let loc = format!(
                    "/?plugin_oauth_error={}",
                    crate::auth::url_encode_component(&msg)
                );
                return Some(respond_h(
                    stream,
                    302,
                    "text/plain; charset=utf-8",
                    "",
                    &[("Location", loc)],
                ));
            }
            let code = query_param(&req.query, "code").unwrap_or_default();
            let state_param = query_param(&req.query, "state").unwrap_or_default();
            let session_pid = principal.map(|p| p.active_person_id.as_str());
            match crate::plugin_cloud::complete_oauth_callback(
                &root_cfg.data_dir,
                &code,
                &state_param,
                session_pid,
            ) {
                Ok((provider, account_email, return_to)) => {
                    if query_param(&req.query, "json").as_deref() == Some("1") {
                        respond_json(
                            stream,
                            200,
                            &json!({
                                "ok": true,
                                "provider": provider,
                                "account_email": account_email,
                                "return_to": return_to,
                            }),
                        )
                    } else {
                        let sep = if return_to.contains('?') { '&' } else { '?' };
                        let loc = format!(
                            "{return_to}{sep}plugin_connected={}",
                            crate::auth::url_encode_component(&provider)
                        );
                        respond_h(
                            stream,
                            302,
                            "text/plain; charset=utf-8",
                            "",
                            &[("Location", loc)],
                        )
                    }
                }
                Err((status, msg)) => {
                    if query_param(&req.query, "json").as_deref() == Some("1") {
                        respond_err(stream, status, &msg)
                    } else {
                        let loc = format!(
                            "/?plugin_oauth_error={}",
                            crate::auth::url_encode_component(&msg)
                        );
                        respond_h(
                            stream,
                            302,
                            "text/plain; charset=utf-8",
                            "",
                            &[("Location", loc)],
                        )
                    }
                }
            }
        }
        ("POST", "/api/plugins/oauth/disconnect") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let provider = parsed["provider"].as_str().unwrap_or("");
            match crate::plugin_cloud::disconnect_provider(&runtime.cfg.data_dir, provider) {
                Ok(()) => {
                    let scheme = if tls_enabled { "https" } else { "http" };
                    let host = req.host.as_deref().unwrap_or("127.0.0.1");
                    let origin = format!("{scheme}://{host}");
                    respond_json(
                        stream,
                        200,
                        &crate::plugin_cloud::list_oauth_suites_json(
                            &root_cfg.data_dir,
                            &runtime.cfg.data_dir,
                            &origin,
                        ),
                    )
                }
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", "/api/suite") => {
            let lib = state.library.lock().unwrap();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let installed: Vec<Value> = suite
                .resolved(&lib)
                .iter()
                .map(|i| i.to_json())
                .collect();
            respond_json(
                stream,
                200,
                &json!({"person_id": person.id, "installed": installed}),
            )
        }
        ("POST", "/api/suite/install") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            let lib = state.library.lock().unwrap();
            match lib.get(id) {
                None => respond_err(stream, 404, "library item not found"),
                Some(item) if item.builtin => respond_err(stream, 400, "that item is built into every agent already"),
                Some(item) => {
                    library::with_suite_lock(|| {
                        let mut suite = library::load_suite(&runtime.cfg.data_dir);
                        suite.install(&item.id);
                        library::save_suite(&runtime.cfg.data_dir, &suite);
                    });
                    respond_json(stream, 200, &json!({"installed": item.id}))
                }
            }
        }
        ("POST", "/api/suite/remove") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            library::with_suite_lock(|| {
                let mut suite = library::load_suite(&runtime.cfg.data_dir);
                suite.installed.retain(|i| i != id);
                library::save_suite(&runtime.cfg.data_dir, &suite);
            });
            respond_json(stream, 200, &json!({"removed": id}))
        }
        ("GET", "/api/security") => {
            let (ok, len, head) = tripwire::check_log_integrity(&root_cfg.data_dir);
            let events = tripwire::events_for(&root_cfg.data_dir, &person.id, 60);
            let now = records::now_millis();
            let day: Vec<&Value> = events
                .iter()
                .filter(|e| now.saturating_sub(e["ts"].as_u64().unwrap_or(0)) < 86_400_000)
                .collect();
            let policy = tripwire::load_policy(&root_cfg.data_dir);
            let local = state.gate_token.is_none() && host_allowed(req.host.as_deref().unwrap_or(""), &[]);
            respond_json(
                stream,
                200,
                &json!({
                    "armed": tripwire::armed_traps(),
                    "events": events,
                    "last_24h": {
                        "events": day.len(),
                        "critical": day.iter().filter(|e| e["severity"] == "critical").count(),
                        "blocked": day.iter().filter(|e| matches!(e["action"].as_str(), Some("blocked" | "stopped run" | "banned" | "refused" | "held"))).count(),
                    },
                    "log": {"verified": ok, "length": len, "head_hash": head},
                    "bans": state.tripwire.bans_json(),
                    "policy": {
                        "allow_hosts": policy.allow_hosts,
                        "block_hosts": policy.block_hosts,
                        "allow_private_env": std::env::var("ERGON_FETCH_ALLOW_PRIVATE").map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true")).unwrap_or(false),
                    },
                    "can_edit": local,
                }),
            )
        }
        ("POST", "/api/security/policy") => {
            // Same rule as model settings: only from this machine.
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "security settings can only be changed from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            if let Some(ip) = parsed["unban"].as_str() {
                return Some(match ip.trim().parse::<std::net::IpAddr>() {
                    Ok(ip) => respond_json(stream, 200, &json!({"unbanned": state.tripwire.unban(ip), "bans": state.tripwire.bans_json()})),
                    Err(_) => respond_err(stream, 400, "unban needs an IP address"),
                });
            }
            let list = parsed["list"].as_str().unwrap_or("");
            let host = parsed["host"].as_str().unwrap_or("");
            let add = parsed["add"].as_bool().unwrap_or(true);
            match tripwire::update_policy(&root_cfg.data_dir, list, host, add) {
                Ok(p) => respond_json(stream, 200, &json!({"policy": {"allow_hosts": p.allow_hosts, "block_hosts": p.block_hosts}})),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("GET", "/api/backups") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "backups can only be managed directly from this machine, not through a shared address"}),
                ));
            }
            let bcfg = backup::load_backup_config(&root_cfg.data_dir);
            let bst = backup::load_backup_state(&root_cfg.data_dir);
            let local_dir = bcfg.effective_local_dir(&root_cfg.data_dir);
            let backups: Vec<Value> = backup::list_backups(&local_dir)
                .iter()
                .map(backup::BackupListEntry::to_json)
                .collect();
            respond_json(
                stream,
                200,
                &json!({
                    "config": bcfg.to_json(&root_cfg.data_dir),
                    "state": bst.to_json(),
                    "backups": backups,
                    "can_edit": true,
                }),
            )
        }
        ("POST", "/api/backups/config") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "backup settings can only be changed directly from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            if !parsed.is_object() {
                return Some(respond_err(stream, 400, "backup config body must be a JSON object"));
            }
            match backup::update_backup_config_from_json(&root_cfg.data_dir, &parsed) {
                Ok(bcfg) => respond_json(
                    stream,
                    200,
                    &json!({
                        "ok": true,
                        "config": bcfg.to_json(&root_cfg.data_dir),
                    }),
                ),
                Err(e) => respond_err(stream, 400, &e),
            }
        }
        ("POST", "/api/backups/run") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "running backups is only allowed directly from this machine, not through a shared address"}),
                ));
            }
            let _guard = state.backup_lock.lock().unwrap_or_else(|p| p.into_inner());
            let bcfg = backup::load_backup_config(&root_cfg.data_dir);
            match backup::run_backup_with_config(&root_cfg.data_dir, &bcfg, None) {
                Ok(report) => {
                    let local_dir = bcfg.effective_local_dir(&root_cfg.data_dir);
                    let backups: Vec<Value> = backup::list_backups(&local_dir)
                        .iter()
                        .map(backup::BackupListEntry::to_json)
                        .collect();
                    let bst = backup::load_backup_state(&root_cfg.data_dir);
                    respond_json(
                        stream,
                        200,
                        &json!({
                            "ok": true,
                            "backup": report.to_json(),
                            "state": bst.to_json(),
                            "backups": backups,
                        }),
                    )
                }
                Err(e) => respond_err(stream, 500, &e),
            }
        }
        ("GET", "/api/backups/download") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "downloading backups is only allowed directly from this machine, not through a shared address"}),
                ));
            }
            let Some(name) = query_param(&req.query, "name").filter(|s| !s.is_empty()) else {
                return Some(respond_err(stream, 400, "missing required query parameter 'name'"));
            };
            let bcfg = backup::load_backup_config(&root_cfg.data_dir);
            let local_dir = bcfg.effective_local_dir(&root_cfg.data_dir);
            let resolved = match backup::resolve_confined_backup_path(&local_dir, &name) {
                Ok(p) => p,
                Err(e) => {
                    let code = if e.contains("not found") || e.contains("does not exist") {
                        404
                    } else {
                        403
                    };
                    return Some(respond_err(stream, code, &e));
                }
            };
            let bytes = match std::fs::read(&resolved) {
                Ok(b) => b,
                Err(_) => return Some(respond_err(stream, 404, "backup archive not found")),
            };
            let sha = receipts::sha256_hex(&bytes);
            let headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                (
                    "Content-Disposition",
                    format!("attachment; filename=\"{name}\""),
                ),
                ("X-Ergon-Backup-SHA256", sha),
            ];
            respond_bytes(stream, 200, "application/zip", &bytes, &headers)
        }
        ("POST", "/api/backups/restore") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "restoring backups is only allowed directly from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("").trim();
            if name.is_empty() {
                return Some(respond_err(stream, 400, "restore requires backup 'name'"));
            }
            if !parsed["confirm"].as_bool().unwrap_or(false) {
                return Some(respond_err(
                    stream,
                    400,
                    "restore requires explicit confirmation (\"confirm\": true)",
                ));
            }
            if RUNS_IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                return Some(respond_err(
                    stream,
                    409,
                    "a task is still running; wait for it to finish before restoring a backup",
                ));
            }
            let bcfg = backup::load_backup_config(&root_cfg.data_dir);
            let local_dir = bcfg.effective_local_dir(&root_cfg.data_dir);
            let resolved = match backup::resolve_confined_backup_path(&local_dir, name) {
                Ok(p) => p,
                Err(e) => {
                    let code = if e.contains("not found") || e.contains("does not exist") {
                        404
                    } else {
                        403
                    };
                    return Some(respond_err(stream, code, &e));
                }
            };
            let _guard = state.backup_lock.lock().unwrap_or_else(|p| p.into_inner());
            match backup::restore_backup(&resolved, &root_cfg.data_dir, true) {
                Ok(summary) => {
                    reload_state_after_restore(state);
                    let next_cfg = backup::load_backup_config(&root_cfg.data_dir);
                    let next_dir = next_cfg.effective_local_dir(&root_cfg.data_dir);
                    let backups: Vec<Value> = backup::list_backups(&next_dir)
                        .iter()
                        .map(backup::BackupListEntry::to_json)
                        .collect();
                    respond_json(
                        stream,
                        200,
                        &json!({
                            "ok": true,
                            "restore": summary.to_json(),
                            "backups": backups,
                        }),
                    )
                }
                Err(e) => {
                    let status = if e.contains("failed verification") {
                        400
                    } else {
                        500
                    };
                    respond_err(stream, status, &e)
                }
            }
        }
        ("GET", "/api/diagnostics/summary") => {
            let model_configured = {
                let live = state.model_cfg.lock().unwrap();
                crate::model_configured_for(&live.model_base_url, live.api_key.is_some())
            };
            let uptime = state.started_at.elapsed().as_secs();
            respond_json(
                stream,
                200,
                &logging::diagnostic_summary_json(root_cfg, uptime, model_configured),
            )
        }
        ("GET", "/api/diagnostics/bundle") => {
            let live_meta = {
                let live = state.model_cfg.lock().unwrap();
                (
                    crate::model_configured_for(&live.model_base_url, live.api_key.is_some()),
                    live.model_id.clone(),
                    is_loopback_url(&live.model_base_url),
                )
            };
            let uptime = state.started_at.elapsed().as_secs();
            logging::info(
                "ergon::diagnostics",
                &format!("generating IT diagnostic bundle for workspace '{}'", person.id),
            );
            match logging::build_diagnostic_bundle_zip(
                root_cfg,
                uptime,
                state.scheduler_enabled,
                Some(live_meta),
            ) {
                Ok(res) => {
                    let headers = vec![
                        ("X-Content-Type-Options", "nosniff".to_string()),
                        ("Cache-Control", "no-store".to_string()),
                        (
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", res.filename),
                        ),
                        ("X-Ergon-Diagnostics-SHA256", res.sha256),
                        ("X-Ergon-Diagnostics-Logs", res.log_files.to_string()),
                        ("X-Ergon-Diagnostics-Crashes", res.crash_reports.to_string()),
                    ];
                    respond_bytes(stream, 200, "application/zip", &res.zip_bytes, &headers)
                }
                Err(e) => {
                    logging::error("ergon::diagnostics", &format!("failed to build diagnostic bundle: {e}"));
                    respond_err(stream, 500, &format!("could not build diagnostic bundle: {e}"))
                }
            }
        }
        ("GET", "/api/help/config") => {
            let local = state.gate_token.is_none()
                && host_allowed(req.host.as_deref().unwrap_or(""), &[]);
            respond_json(
                stream,
                200,
                &logging::help_config_json(&root_cfg.data_dir, &runtime.cfg.data_dir, local),
            )
        }
        ("PUT", "/api/help/config") | ("POST", "/api/help/config") => {
            let local = state.gate_token.is_none()
                && host_allowed(req.host.as_deref().unwrap_or(""), &[]);
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            if !parsed.is_object() {
                return Some(respond_err(stream, 400, "JSON object body is required"));
            }
            if let Some(done) = parsed["onboarding_completed"].as_bool() {
                if let Err(e) = logging::save_help_prefs(&runtime.cfg.data_dir, done) {
                    return Some(respond_err(stream, 500, &e));
                }
            }
            if let Some(contact_obj) = parsed.get("admin_contact").and_then(|v| v.as_object()) {
                if !local {
                    return Some(respond_json(
                        stream,
                        403,
                        &json!({"error": "admin contact settings can only be changed from this machine, not through a shared address"}),
                    ));
                }
                let current = logging::load_admin_contact(&root_cfg.data_dir);
                let name = contact_obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current.name);
                let email = contact_obj
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current.email);
                let phone = contact_obj
                    .get("phone")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current.phone);
                let channel_url = contact_obj
                    .get("channel_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current.channel_url);
                let notes = contact_obj
                    .get("notes")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current.notes);
                if let Err(e) = logging::save_admin_contact(
                    &root_cfg.data_dir,
                    name,
                    email,
                    phone,
                    channel_url,
                    notes,
                ) {
                    return Some(respond_err(stream, 400, &e));
                }
            }
            respond_json(
                stream,
                200,
                &logging::help_config_json(&root_cfg.data_dir, &runtime.cfg.data_dir, local),
            )
        }
        _ => return None,
    })
}

