//! Integration tests for one-click OAuth2 (PKCE S256) + native Google Workspace
//! and Microsoft Graph REST adapters inside Ergon's 18-plugin workplace library.

use ergon_rs::{auth, serve, tripwire, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static SEQ: AtomicU64 = AtomicU64::new(0);

struct CloudTestDir {
    root: PathBuf,
    cfg: Config,
}

impl CloudTestDir {
    fn new(label: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-plugin-cloud-{}-{}-{label}",
            std::process::id(),
            id
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9".to_string(),
            model_id: "test-model".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }
}

impl Drop for CloudTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Debug)]
struct MockHit {
    method: String,
    path: String,
    auth: String,
    body: String,
}

struct MockCloudServer {
    addr: SocketAddr,
    hits: Arc<Mutex<Vec<MockHit>>>,
    stop: Arc<AtomicBool>,
}

impl Drop for MockCloudServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

fn read_http_message(stream: &mut TcpStream) -> Option<(String, String, String, String)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let sep = loop {
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break pos;
                }
            }
            Err(_) => return None,
        }
    };
    let head = String::from_utf8_lossy(&buf[..sep]).to_string();
    let mut clen = 0usize;
    let mut auth = String::new();
    for line in head.lines().skip(1) {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("Content-Length") {
                clen = v.trim().parse().unwrap_or(0);
            } else if k.eq_ignore_ascii_case("Authorization") {
                auth = v.trim().to_string();
            }
        }
    }
    while buf.len() < sep + 4 + clen {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let body_end = (sep + 4 + clen).min(buf.len());
    let body = String::from_utf8_lossy(&buf[sep + 4..body_end]).to_string();
    let mut parts = head.lines().next().unwrap_or("").split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    Some((method, path, auth, body))
}

fn spawn_mock_cloud(provider: &'static str) -> MockCloudServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let hits: Arc<Mutex<Vec<MockHit>>> = Arc::new(Mutex::new(Vec::new()));
    let hits_bg = Arc::clone(&hits);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_bg = Arc::clone(&stop);

    std::thread::spawn(move || {
        while !stop_bg.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut conn, _)) => {
                    let _ = conn.set_nonblocking(false);
                    let _ = conn.set_read_timeout(Some(Duration::from_secs(5)));
                    let Some((method, path, auth, body)) = read_http_message(&mut conn) else {
                        continue;
                    };
                    hits_bg.lock().unwrap().push(MockHit {
                        method: method.clone(),
                        path: path.clone(),
                        auth: auth.clone(),
                        body: body.clone(),
                    });

                    let (status, resp_json): (u16, Value) = match (method.as_str(), path.as_str()) {
                        ("POST", "/oauth/token") => {
                            if body.contains("grant_type=refresh_token") {
                                (
                                    200,
                                    json!({
                                        "access_token": "tok-refreshed-2",
                                        "refresh_token": "rt-rotated-2",
                                        "expires_in": 3600,
                                        "token_type": "Bearer"
                                    }),
                                )
                            } else {
                                (
                                    200,
                                    json!({
                                        "access_token": "tok-initial-1",
                                        "refresh_token": "rt-initial-1",
                                        "expires_in": 3600,
                                        "token_type": "Bearer"
                                    }),
                                )
                            }
                        }
                        ("GET", "/userinfo") => {
                            if provider == "google" {
                                (200, json!({ "email": "alice@acme.test", "name": "Alice Acme" }))
                            } else {
                                (200, json!({ "mail": "alice@contoso.test", "displayName": "Alice Contoso" }))
                            }
                        }
                        // Google Workspace endpoints
                        ("GET", p) if p.starts_with("/gmail/v1/users/me/messages?") => (
                            200,
                            json!({ "messages": [{ "id": "gm-1" }] }),
                        ),
                        ("GET", p) if p.starts_with("/gmail/v1/users/me/messages/gm-1") => (
                            200,
                            json!({
                                "id": "gm-1",
                                "snippet": "Updated PO attached",
                                "payload": { "headers": [
                                    { "name": "Subject", "value": "Q4 Vendor Quote" },
                                    { "name": "From", "value": "vendor@example.test" }
                                ] }
                            }),
                        ),
                        ("POST", "/gmail/v1/users/me/messages/send") => (200, json!({ "id": "gm-sent-9" })),
                        ("GET", p) if p.starts_with("/calendar/v3/calendars/primary/events") => (
                            200,
                            json!({ "items": [{ "id": "ev-1", "summary": "Quarterly Review" }] }),
                        ),
                        ("POST", "/calendar/v3/calendars/primary/events") => (200, json!({ "id": "ev-2" })),
                        ("GET", p) if p.starts_with("/drive/v3/files") => (
                            200,
                            json!({ "files": [{ "id": "df-1", "name": "Budget.xlsx" }] }),
                        ),
                        ("POST", p) if p.starts_with("/upload/drive/v3/files") => (200, json!({ "id": "df-up-1" })),
                        ("POST", "/v1/documents") => (200, json!({ "documentId": "doc-77" })),
                        ("POST", p) if p.starts_with("/v1/documents/doc-77:batchUpdate") => (200, json!({ "documentId": "doc-77" })),
                        ("POST", "/v4/spreadsheets") => (200, json!({ "spreadsheetId": "sh-88" })),
                        ("PUT", p) if p.starts_with("/v4/spreadsheets/") => (200, json!({ "updatedCells": 2 })),
                        // Microsoft Graph endpoints
                        ("GET", p) if p.starts_with("/me/mailFolders/inbox/messages") => (
                            200,
                            json!({ "value": [{
                                "subject": "Renewals Review",
                                "from": { "emailAddress": { "address": "cfo@contoso.test" } },
                                "bodyPreview": "Please review Q4 totals"
                            }] }),
                        ),
                        ("POST", "/me/sendMail") => (202, json!({})),
                        ("GET", p) if p.starts_with("/me/events") => (
                            200,
                            json!({ "value": [{ "id": "ms-ev-1", "subject": "Ops Sync" }] }),
                        ),
                        ("POST", "/me/events") => (201, json!({ "id": "ms-ev-2" })),
                        ("GET", p) if p.starts_with("/me/drive/root/children") => (
                            200,
                            json!({ "value": [{ "id": "od-1", "name": "Forecast.xlsx" }] }),
                        ),
                        ("PUT", p) if p.starts_with("/me/drive/root:/") => (201, json!({ "id": "od-up-1" })),
                        ("GET", p) if p.starts_with("/teams/") && p.contains("/messages") => (
                            200,
                            json!({ "value": [{ "id": "tm-1", "body": { "content": "Ready" } }] }),
                        ),
                        ("POST", p) if p.starts_with("/teams/") && p.contains("/messages") => (201, json!({ "id": "tm-2" })),
                        _ => (404, json!({ "error": format!("unhandled mock {method} {path}") })),
                    };

                    let payload = resp_json.to_string();
                    let wire = format!(
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        payload.len(),
                        payload
                    );
                    let _ = conn.write_all(wire.as_bytes());
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    MockCloudServer { addr, hits, stop }
}

fn spawn_server(cfg: Config) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(40));
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let body_bytes = body.as_bytes();
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n", addr.port());
    if !body_bytes.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).unwrap();
    }
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_text = String::from_utf8_lossy(&raw[sep + 4..]).to_string();
    let val = serde_json::from_str::<Value>(&body_text).unwrap_or(Value::Null);
    (status, val, body_text)
}

fn extract_query_param(url: &str, key: &str) -> String {
    let qs = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return v.to_string();
            }
        }
    }
    String::new()
}

fn form_param(body: &str, key: &str) -> String {
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return v.to_string();
            }
        }
    }
    String::new()
}

#[test]
fn google_workspace_oauth2_pkce_refresh_native_rest_and_approval_gate() {
    let td = CloudTestDir::new("google");
    let _policy = tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));
    let mock = spawn_mock_cloud("google");
    let addr = spawn_server(td.cfg.clone());
    let base = format!("http://127.0.0.1:{}", mock.addr.port());

    // 1. Register Google OAuth app (secret encrypted in secrets.enc.json)
    let (st_reg, reg_json, raw_reg) = http_req(
        addr,
        "PUT",
        "/api/plugins/oauth/apps",
        &json!({
            "provider": "google",
            "client_id": "test-google-client-id",
            "client_secret": "super-secret-google-client-key",
            "auth_url": format!("{base}/oauth/auth"),
            "token_url": format!("{base}/oauth/token"),
            "userinfo_url": format!("{base}/userinfo"),
            "api_base_url": base
        })
        .to_string(),
    );
    assert_eq!(st_reg, 200, "{raw_reg}");
    let g_suite = reg_json["suites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["provider"] == "google")
        .unwrap();
    assert_eq!(g_suite["configured"], json!(true));
    assert_eq!(g_suite["has_client_secret"], json!(true));

    let apps_disk = std::fs::read_to_string(td.cfg.data_dir.join("plugin_oauth_apps.json")).unwrap();
    assert!(
        !apps_disk.contains("super-secret-google-client-key"),
        "client_secret must NEVER persist in plaintext: {apps_disk}"
    );
    assert!(apps_disk.contains("vault:plugin_oauth:google:client_secret"));

    // 2. Start OAuth2 PKCE S256 flow
    let (st_start, start_json, raw_start) = http_req(
        addr,
        "POST",
        "/api/plugins/oauth/start",
        &json!({ "provider": "google", "return_to": "/?tab=plugins" }).to_string(),
    );
    assert_eq!(st_start, 200, "{raw_start}");
    let auth_url = start_json["authorization_url"].as_str().unwrap();
    assert!(auth_url.contains("code_challenge_method=S256"), "{auth_url}");
    assert!(auth_url.contains("access_type=offline"), "{auth_url}");
    let state = extract_query_param(auth_url, "state");
    let code_challenge = extract_query_param(auth_url, "code_challenge");
    assert!(!state.is_empty() && !code_challenge.is_empty());

    // 3. Complete OAuth callback and verify PKCE verifier + encrypted tokens + single-use state
    let (st_cb, cb_json, raw_cb) = http_req(
        addr,
        "GET",
        &format!("/api/plugins/oauth/callback?code=auth-code-g1&state={state}&json=1"),
        "",
    );
    assert_eq!(st_cb, 200, "{raw_cb}");
    assert_eq!(cb_json["ok"], json!(true));
    assert_eq!(cb_json["account_email"], json!("alice@acme.test"));

    let (st_replay, _, _) = http_req(
        addr,
        "GET",
        &format!("/api/plugins/oauth/callback?code=auth-code-g1&state={state}&json=1"),
        "",
    );
    assert_eq!(st_replay, 400, "replayed state must fail");

    let token_hit = mock
        .hits
        .lock()
        .unwrap()
        .iter()
        .find(|h| h.path == "/oauth/token")
        .cloned()
        .expect("token exchange hit");
    let verifier = form_param(&token_hit.body, "code_verifier");
    assert_eq!(
        auth::base64url_encode_nopad(&tripwire::sha256(verifier.as_bytes())),
        code_challenge
    );

    let conns_disk = std::fs::read_to_string(td.cfg.data_dir.join("plugin_connections.json")).unwrap();
    assert!(!conns_disk.contains("tok-initial-1"));
    assert!(!conns_disk.contains("rt-initial-1"));
    assert!(conns_disk.contains("vault:plugin_oauth:google:access_token"));

    // 4. All 6 Google Workspace plugins flip to cloud_connected
    let (st_p, p_json, _) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st_p, 200);
    for pid in [
        "google-gmail",
        "google-calendar",
        "google-drive",
        "google-docs",
        "google-sheets",
        "google-slides",
    ] {
        let item = p_json["plugins"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == pid)
            .unwrap();
        assert_eq!(item["cloud_connected"], json!(true), "{pid} not connected: {item}");
        assert_eq!(item["oauth_connected"], json!(true));
    }

    // 5. Execute live read actions across Gmail, Calendar, and Drive
    let (st_mail, res_mail, raw_mail) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "google-gmail", "action": "read_inbox" }).to_string(),
    );
    assert_eq!(st_mail, 200, "{raw_mail}");
    assert!(res_mail["output"].as_str().unwrap().contains("workspace/intake/gmail-inbox-"));

    let intake_dir = td.cfg.workspace_dir.join("intake");
    let intake_files: Vec<_> = std::fs::read_dir(&intake_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("gmail-inbox-"))
        .collect();
    assert_eq!(intake_files.len(), 1);
    let md = std::fs::read_to_string(intake_files[0].path()).unwrap();
    assert!(md.contains("Q4 Vendor Quote"), "{md}");

    let (st_cal, res_cal, raw_cal) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "google-calendar", "action": "list_events" }).to_string(),
    );
    assert_eq!(st_cal, 200, "{raw_cal}");
    assert!(res_cal["output"].as_str().unwrap().contains("Quarterly Review"));

    let (st_drv, res_drv, raw_drv) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "google-drive", "action": "list_files" }).to_string(),
    );
    assert_eq!(st_drv, 200, "{raw_drv}");
    assert!(res_drv["output"].as_str().unwrap().contains("Budget.xlsx"));

    // 6. Force token expiry -> verify automatic refresh_token rotation
    let mut conn_v: Value = serde_json::from_str(&conns_disk).unwrap();
    conn_v["connections"]["google"]["expires_at_ms"] = json!(1);
    std::fs::write(
        td.cfg.data_dir.join("plugin_connections.json"),
        serde_json::to_string_pretty(&conn_v).unwrap(),
    )
    .unwrap();

    let (st_ref, _, raw_ref) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "google-calendar", "action": "list_events" }).to_string(),
    );
    assert_eq!(st_ref, 200, "{raw_ref}");
    let refreshed_hit = mock
        .hits
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find(|h| h.path.starts_with("/calendar/v3/calendars/primary/events"))
        .cloned()
        .unwrap();
    assert_eq!(refreshed_hit.auth, "Bearer tok-refreshed-2");

    // 7. Consequential writes park on /phone; Approve executes REST, Deny skips external call
    let (st_send, _, raw_send) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "google-gmail",
            "action": "send_mail",
            "args": { "to": "vendor@example.test", "subject": "Approved", "body": "Proceed." }
        })
        .to_string(),
    );
    assert_eq!(st_send, 200, "{raw_send}");
    let (st_doc, _, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "google-docs",
            "action": "publish_doc",
            "args": { "title": "Board Memo", "content": "Approved summary" }
        })
        .to_string(),
    );
    assert_eq!(st_doc, 200);
    let (st_sh, _, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "google-sheets",
            "action": "update_range",
            "args": { "title": "Ledger", "values": [["SKU", "Total"], ["A1", "100.00"]] }
        })
        .to_string(),
    );
    assert_eq!(st_sh, 200);

    // None of the external writes should have executed prior to approval
    assert!(
        !mock
            .hits
            .lock()
            .unwrap()
            .iter()
            .any(|h| h.path == "/gmail/v1/users/me/messages/send" || h.path == "/v1/documents"),
        "consequential writes must wait for /phone approval"
    );

    let (_, decs, _) = http_req(addr, "GET", "/api/decisions", "");
    let pending: Vec<String> = decs["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["status"] == "pending")
        .map(|d| d["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(pending.len(), 3);

    // Approve first two (Gmail send + Docs publish), Deny third (Sheets update)
    let (st_a1, a1, raw_a1) =
        http_req(addr, "POST", &format!("/api/decisions/{}/approve", pending[0]), "{}");
    assert_eq!(st_a1, 200, "{raw_a1}");
    assert!(a1["decision"]["outcome"].as_str().unwrap().contains("gm-sent-9"), "{a1}");

    let (st_a2, a2, raw_a2) =
        http_req(addr, "POST", &format!("/api/decisions/{}/approve", pending[1]), "{}");
    assert_eq!(st_a2, 200, "{raw_a2}");
    assert!(a2["decision"]["outcome"].as_str().unwrap().contains("doc-77"), "{a2}");

    let (st_d3, d3, _) =
        http_req(addr, "POST", &format!("/api/decisions/{}/deny", pending[2]), "{}");
    assert_eq!(st_d3, 200);
    assert_eq!(d3["decision"]["status"], json!("denied"));

    let all_hits = mock.hits.lock().unwrap().clone();
    assert!(all_hits.iter().any(|h| h.method == "POST" && h.path == "/gmail/v1/users/me/messages/send"));
    assert!(all_hits.iter().any(|h| h.method == "POST" && h.path == "/v1/documents"));
    assert!(
        !all_hits.iter().any(|h| h.path == "/v4/spreadsheets"),
        "denied Sheets update must never fire"
    );

    // 8. Disconnect clears vault tokens & status
    let (st_disc, disc_json, raw_disc) = http_req(
        addr,
        "POST",
        "/api/plugins/oauth/disconnect",
        &json!({ "provider": "google" }).to_string(),
    );
    assert_eq!(st_disc, 200, "{raw_disc}");
    let g_after = disc_json["suites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["provider"] == "google")
        .unwrap();
    assert_eq!(g_after["connected"], json!(false));
}

#[test]
fn microsoft_365_oauth2_pkce_graph_rest_and_air_gapped_block() {
    let td = CloudTestDir::new("microsoft");
    let _policy = tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));
    let mock = spawn_mock_cloud("microsoft");
    let addr = spawn_server(td.cfg.clone());
    let base = format!("http://127.0.0.1:{}", mock.addr.port());

    std::fs::write(td.cfg.workspace_dir.join("Quote.csv"), "SKU,Price\nX1,42.00\n").unwrap();

    // Register Microsoft Entra app + complete PKCE OAuth2
    let (st_reg, _, raw_reg) = http_req(
        addr,
        "PUT",
        "/api/plugins/oauth/apps",
        &json!({
            "provider": "microsoft",
            "client_id": "ms-app-id-1",
            "tenant": "contoso.onmicrosoft.com",
            "auth_url": format!("{base}/oauth/auth"),
            "token_url": format!("{base}/oauth/token"),
            "userinfo_url": format!("{base}/userinfo"),
            "api_base_url": base
        })
        .to_string(),
    );
    assert_eq!(st_reg, 200, "{raw_reg}");

    let (st_s, s_json, _) = http_req(
        addr,
        "POST",
        "/api/plugins/oauth/start",
        &json!({ "provider": "microsoft" }).to_string(),
    );
    assert_eq!(st_s, 200);
    let state = extract_query_param(s_json["authorization_url"].as_str().unwrap(), "state");
    let (st_cb, cb_json, raw_cb) = http_req(
        addr,
        "GET",
        &format!("/api/plugins/oauth/callback?code=ms-code-1&state={state}&json=1"),
        "",
    );
    assert_eq!(st_cb, 200, "{raw_cb}");
    assert_eq!(cb_json["account_email"], json!("alice@contoso.test"));

    // Read actions across Outlook Mail, Outlook Calendar, OneDrive, and Teams
    let (st_om, res_om, raw_om) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "outlook-mail", "action": "read_inbox" }).to_string(),
    );
    assert_eq!(st_om, 200, "{raw_om}");
    assert!(res_om["output"].as_str().unwrap().contains("workspace/intake/outlook-inbox-"));

    let (st_oc, res_oc, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "outlook-calendar", "action": "list_events" }).to_string(),
    );
    assert_eq!(st_oc, 200);
    assert!(res_oc["output"].as_str().unwrap().contains("Ops Sync"));

    let (st_od, res_od, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "onedrive-sharepoint", "action": "list_files" }).to_string(),
    );
    assert_eq!(st_od, 200);
    assert!(res_od["output"].as_str().unwrap().contains("Forecast.xlsx"));

    let (st_tm, res_tm, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "microsoft-teams",
            "action": "read_channel",
            "args": { "team_id": "team-1", "channel_id": "chan-1" }
        })
        .to_string(),
    );
    assert_eq!(st_tm, 200);
    assert!(res_tm["output"].as_str().unwrap().contains("1 channel message"));

    // Park + approve Outlook sendMail and OneDrive workspace file publish
    let _ = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "outlook-mail",
            "action": "send_mail",
            "args": { "to": "cfo@contoso.test", "subject": "Report", "body": "Ready." }
        })
        .to_string(),
    );
    let _ = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "onedrive-sharepoint",
            "action": "publish_file",
            "args": { "path": "Quote.csv" }
        })
        .to_string(),
    );

    let (_, decs, _) = http_req(addr, "GET", "/api/decisions", "");
    for dec in decs["decisions"].as_array().unwrap() {
        let did = dec["id"].as_str().unwrap();
        let (st_app, _, raw_app) =
            http_req(addr, "POST", &format!("/api/decisions/{did}/approve"), "{}");
        assert_eq!(st_app, 200, "{raw_app}");
    }
    let hits = mock.hits.lock().unwrap().clone();
    assert!(hits.iter().any(|h| h.method == "POST" && h.path == "/me/sendMail"));
    let upload_hit = hits
        .iter()
        .find(|h| h.method == "PUT" && h.path.starts_with("/me/drive/root:/Quote.csv"))
        .expect("OneDrive upload hit");
    assert!(upload_hit.body.contains("SKU,Price"), "{}", upload_hit.body);

    // Switch workspace to Air-Gapped -> OAuth & REST calls must be blocked
    let (st_pol, _, raw_pol) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        &json!({ "profile": "air_gapped" }).to_string(),
    );
    assert_eq!(st_pol, 200, "{raw_pol}");

    let (st_blk, blk_json, _) = http_req(
        addr,
        "POST",
        "/api/plugins/oauth/start",
        &json!({ "provider": "microsoft" }).to_string(),
    );
    assert_eq!(st_blk, 403, "{blk_json}");

    let (st_api_blk, api_blk_json, _) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({ "plugin_id": "outlook-mail", "action": "read_inbox" }).to_string(),
    );
    assert_eq!(st_api_blk, 400, "{api_blk_json}");
    assert!(api_blk_json["error"].as_str().unwrap().contains("Air-Gapped"));
}
