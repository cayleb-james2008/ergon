//! Integration tests for Ergon v0.31.0 Gap #3:
//! - Persistent connectors.json & connector_secrets.json with 0600 POSIX permissions
//! - Masked secrets on GET /api/connectors & secret rotation
//! - Signed inbound webhook (HMAC-SHA256 & X-Ergon-Hook-Token) -> 202 + inbox file + routine/pipeline trigger
//! - Webhook Origin/CSRF exemption & 401 on invalid/missing auth
//! - Local folder-watch ingest (2-poll stability, glob filter, SHA-256 dedup, sensitive path rejection, loopback-only)
//! - Outbound delivery + {{secret:key}} host substitution + tripwire EGRESS_TOOLS & active-run redaction
//! - Strict Client forced approval hold + decision settlement execution + Air-Gapped hard-block

use ergon_rs::{
    connectors, execute_tool, records, serve, serve_with_hosts, tripwire, Config,
};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-connectors-v031-{}-{}-{name}",
            std::process::id(),
            seq
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

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
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

fn http_req_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut has_host = false;
    let mut has_ct = false;
    let mut head = format!("{method} {path} HTTP/1.1\r\n");
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("host") {
            has_host = true;
        }
        if k.eq_ignore_ascii_case("content-type") {
            has_ct = true;
        }
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if !has_host {
        head.push_str(&format!("Host: 127.0.0.1:{}\r\n", addr.port()));
    }
    if !has_ct && !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body).unwrap();
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
    let body_bytes = &raw[sep + 4..];
    let body_text = String::from_utf8_lossy(body_bytes).to_string();
    let val = serde_json::from_slice(body_bytes).unwrap_or(Value::Null);
    (status, val, body_text)
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let (status, val, _) = http_req_headers(addr, method, path, &[], body.as_bytes());
    (status, val)
}

#[derive(Clone, Debug)]
struct CapturedReq {
    method: String,
    headers: Vec<(String, String)>,
    body: String,
}

fn spawn_mock_target(echo_text: &'static str) -> (SocketAddr, Arc<Mutex<Vec<CapturedReq>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let cap_clone = captured.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { break };
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match s.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if let Some(sep) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            let head_s = String::from_utf8_lossy(&buf[..sep]);
                            let cl = head_s
                                .lines()
                                .find_map(|l| {
                                    let (k, v) = l.split_once(':')?;
                                    if k.trim().eq_ignore_ascii_case("content-length") {
                                        v.trim().parse::<usize>().ok()
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0);
                            if buf.len() >= sep + 4 + cl {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            if let Some(sep) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head_s = String::from_utf8_lossy(&buf[..sep]).to_string();
                let mut lines = head_s.lines();
                let req_line = lines.next().unwrap_or("");
                let method = req_line.split_whitespace().next().unwrap_or("").to_string();
                let mut headers = Vec::new();
                for l in lines {
                    if let Some((k, v)) = l.split_once(':') {
                        headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
                    }
                }
                let body = String::from_utf8_lossy(&buf[sep + 4..]).to_string();
                cap_clone.lock().unwrap().push(CapturedReq {
                    method,
                    headers,
                    body,
                });
            }
            let resp_body = json!({ "received": true, "echo": echo_text }).to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                resp_body.len(),
                resp_body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (addr, captured)
}

#[test]
fn signed_webhook_202_pipeline_trigger_and_invalid_hmac_401() {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let td = TestDir::new("webhook");
    let addr = spawn_server(td.cfg.clone());

    // Advertise connectors capability in /api/status.
    let (st, status_json) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(status_json["capabilities"]["connectors"], json!(true));

    // Create a deterministic zero-LLM pipeline routine using {{inputs.trigger_file}}.
    let (st_r, rtn_json) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "Webhook Intake Pipeline",
            "prompt": "Process incoming webhook payload",
            "schedule": "manual",
            "steps_v2": [
                {
                    "id": "doc",
                    "kind": "skill",
                    "label": "Write intake note",
                    "skill_id": "write_document",
                    "args": {
                        "title": "Webhook Note",
                        "sections": ["Ingested trigger file: {{inputs.trigger_file}}"]
                    }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_r, 201, "{rtn_json}");
    let routine_id = rtn_json["routine"]["id"].as_str().unwrap().to_string();

    // Create webhook trigger linked to the routine.
    let (st_t, trg_json) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({
            "name": "Stripe Events",
            "kind": "webhook",
            "routine_id": routine_id,
            "target_kind": "pipeline"
        })
        .to_string(),
    );
    assert_eq!(st_t, 201, "{trg_json}");
    let trigger_id = trg_json["trigger"]["id"].as_str().unwrap().to_string();
    let hook_path = trg_json["hook_path"].as_str().unwrap().to_string();
    let signing_secret = trg_json["signing_secret"].as_str().unwrap().to_string();
    let hook_token = trg_json["hook_token"].as_str().unwrap().to_string();
    assert!(!signing_secret.is_empty());
    assert!(!hook_token.is_empty());

    // Verify 0600 POSIX permissions on connectors.json & connector_secrets.json.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let c_mode = std::fs::metadata(connectors::connectors_path(&td.cfg.data_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        let s_mode = std::fs::metadata(connectors::connector_secrets_path(&td.cfg.data_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(c_mode, 0o600);
        assert_eq!(s_mode, 0o600);
    }

    // GET /api/connectors must mask secrets as has_secret: true.
    let (st_g, list_json, list_raw) =
        http_req_headers(addr, "GET", "/api/connectors", &[], b"");
    assert_eq!(st_g, 200);
    assert_eq!(list_json["triggers"][0]["has_secret"], json!(true));
    assert!(!list_raw.contains(&signing_secret));
    assert!(!list_raw.contains(&hook_token));

    let payload = br#"{"event":"invoice.paid","amount":1250}"#;

    // 1. Missing auth -> 401.
    let (st_noauth, _, _) = http_req_headers(addr, "POST", &hook_path, &[], payload);
    assert_eq!(st_noauth, 401);

    // 2. Bad HMAC (even with cross-origin headers) -> 401.
    let bad_sig = format!("sha256={}", "0".repeat(64));
    let (st_bad, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("Origin", "https://webhooks.stripe.com"),
            ("Sec-Fetch-Site", "cross-site"),
            ("X-Ergon-Signature", &bad_sig),
        ],
        payload,
    );
    assert_eq!(st_bad, 401);

    // 3. Valid HMAC-SHA256 + foreign Origin/Sec-Fetch-Site (CSRF exempt) -> 202 + routine triggered!
    let mac = tripwire::hmac_sha256(signing_secret.as_bytes(), payload);
    let sig_hdr = format!("sha256={}", tripwire::hex(&mac));
    let (st_ok, ok_json, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("Origin", "https://webhooks.stripe.com"),
            ("Sec-Fetch-Site", "cross-site"),
            ("X-Ergon-Signature", &sig_hdr),
        ],
        payload,
    );
    assert_eq!(st_ok, 202, "{ok_json}");
    assert_eq!(ok_json["accepted"], json!(true));
    let trigger_file = ok_json["trigger_file"].as_str().unwrap().to_string();
    assert!(trigger_file.starts_with("inbox/hook-"), "{trigger_file}");
    let inbox_full = td.cfg.workspace_dir.join(&trigger_file);
    assert!(inbox_full.exists());
    let inbox_doc: Value =
        serde_json::from_str(&std::fs::read_to_string(&inbox_full).unwrap()).unwrap();
    assert_eq!(inbox_doc["payload"]["event"], json!("invoice.paid"));

    // Wait for linked pipeline run to complete and verify trigger_file input flowed through.
    let pipe_run_id = ok_json["run"]["pipeline_run_id"]
        .as_str()
        .unwrap()
        .to_string();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut final_status = String::new();
    while Instant::now() < deadline {
        let (st_p, p_json) =
            http_req(addr, "GET", &format!("/api/pipelines/runs/{pipe_run_id}"), "");
        assert_eq!(st_p, 200);
        final_status = p_json["run"]["status"].as_str().unwrap_or("").to_string();
        if final_status == "completed" {
            assert_eq!(p_json["run"]["inputs"]["trigger_file"], json!(trigger_file));
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(final_status, "completed");
    let note_md = std::fs::read_to_string(td.cfg.workspace_dir.join("Webhook Note.md")).unwrap();
    assert!(note_md.contains(&trigger_file), "{note_md}");

    // 4. Rotate webhook secret -> old token fails (401), new token succeeds (202).
    let (st_rot, rot_json) = http_req(
        addr,
        "POST",
        &format!("/api/connectors/triggers/{trigger_id}/rotate"),
        "",
    );
    assert_eq!(st_rot, 200, "{rot_json}");
    let new_token = rot_json["hook_token"].as_str().unwrap().to_string();
    assert_ne!(new_token, hook_token);

    let (st_old, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[("X-Ergon-Hook-Token", &hook_token)],
        payload,
    );
    assert_eq!(st_old, 401);

    let (st_new, new_json, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[("X-Ergon-Hook-Token", &new_token)],
        payload,
    );
    assert_eq!(st_new, 202, "{new_json}");

    // Delete trigger.
    let (st_del, del_json) = http_req(
        addr,
        "DELETE",
        &format!("/api/connectors/triggers/{trigger_id}"),
        "",
    );
    assert_eq!(st_del, 200, "{del_json}");
}

#[test]
fn folder_watch_ingest_stability_dedup_and_security() {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let td = TestDir::new("folder-watch");
    let watch_dir = td.root.join("dropzone");
    std::fs::create_dir_all(&watch_dir).unwrap();

    // Serve with an extra allowed non-loopback host to verify loopback-only rejection.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cfg_srv = td.cfg.clone();
    std::thread::spawn(move || {
        let _ = serve_with_hosts(listener, cfg_srv, None, vec!["desk.tailnet.ts.net".to_string()]);
    });
    std::thread::sleep(Duration::from_millis(40));

    // 1. Reject non-loopback Host configuring folder_watch -> 403.
    let (st_non_loop, _, _) = http_req_headers(
        addr,
        "POST",
        "/api/connectors/triggers",
        &[("Host", "desk.tailnet.ts.net:8787")],
        &serde_json::to_vec(&json!({
            "name": "Remote Folder Watch",
            "kind": "folder_watch",
            "watch_dir": watch_dir.to_string_lossy(),
            "glob": "*.csv"
        }))
        .unwrap(),
    );
    assert_eq!(st_non_loop, 403);

    // 2. Reject sensitive paths (SSH / Ergon data_dir) -> 400.
    let (st_sens, _, _) = http_req_headers(
        addr,
        "POST",
        "/api/connectors/triggers",
        &[],
        &serde_json::to_vec(&json!({
            "name": "Bad Watch",
            "kind": "folder_watch",
            "watch_dir": td.cfg.data_dir.to_string_lossy(),
            "glob": "*.json"
        }))
        .unwrap(),
    );
    assert_eq!(st_sens, 400);
    assert!(tripwire::sensitive_target("/home/alice/.ssh/id_rsa").is_some());

    // 3. Configure valid loopback folder_watch trigger.
    let (st_fw, fw_json) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({
            "name": "CSV Dropzone",
            "kind": "folder_watch",
            "watch_dir": watch_dir.to_string_lossy(),
            "glob": "*.csv,*.txt",
            "routine_id": "rtn-linked-csv"
        })
        .to_string(),
    );
    assert_eq!(st_fw, 201, "{fw_json}");

    // Drop matching file + non-matching file.
    std::fs::write(watch_dir.join("orders.csv"), "id,amount\n1,42.00\n").unwrap();
    std::fs::write(watch_dir.join("ignore.png"), "PNGDATA").unwrap();

    let mut poll_state = connectors::FolderWatchPollState::default();

    // Poll 1: file seen once -> NOT ingested yet (needs 2 stable polls).
    let ev1 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll_state);
    assert!(ev1.is_empty(), "must wait for 2 stable polls");
    assert!(!td.cfg.workspace_dir.join("intake/orders.csv").exists());

    // Poll 2: stable across 2 polls -> ingested into workspace/intake/orders.csv!
    let ev2 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll_state);
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].trigger_file, "intake/orders.csv");
    assert_eq!(ev2[0].routine_id.as_deref(), Some("rtn-linked-csv"));
    assert!(td.cfg.workspace_dir.join("intake/orders.csv").exists());
    assert!(!td.cfg.workspace_dir.join("intake/ignore.png").exists());

    // Poll 3 + duplicate file with identical SHA-256 -> deduped by manifest!
    std::fs::write(watch_dir.join("orders_dup.csv"), "id,amount\n1,42.00\n").unwrap();
    let ev3 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll_state);
    assert!(ev3.is_empty());
    let ev4 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll_state);
    assert!(ev4.is_empty(), "identical SHA-256 content must be deduped");
    assert!(!td.cfg.workspace_dir.join("intake/orders_dup.csv").exists());
}

#[test]
fn outbound_delivery_secret_isolation_strict_client_hold_and_air_gapped_block() {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    std::env::set_var("ERGON_FETCH_ALLOW_PRIVATE", "1");
    let td = TestDir::new("outbound");
    let addr = spawn_server(td.cfg.clone());
    let (mock_addr, captured) = spawn_mock_target("delivery-ok-secret-test");

    let secret_val = "ergon-live-connector-secret-xyz987";
    let (st_c, conn_json, conn_raw) = http_req_headers(
        addr,
        "POST",
        "/api/connectors/outbound",
        &[],
        &serde_json::to_vec(&json!({
            "name": "Partner REST API",
            "kind": "rest_api",
            "url": format!("http://127.0.0.1:{}/deliver", mock_addr.port()),
            "method": "POST",
            "headers_template": {
                "Authorization": "Bearer {{secret:partner_api_key}}",
                "X-Client": "ergon-ops"
            },
            "require_approval": false,
            "secrets": {
                "partner_api_key": secret_val
            }
        }))
        .unwrap(),
    );
    assert_eq!(st_c, 201, "{conn_raw}");
    assert!(!conn_raw.contains(secret_val));
    let conn_id = conn_json["connector"]["id"].as_str().unwrap().to_string();
    assert_eq!(conn_json["connector"]["has_secret"], json!(true));

    // Verify tripwire sensitive_target protects connector_secrets.json and connectors.json.
    assert!(tripwire::sensitive_target("/tmp/any/connector_secrets.json").is_some());
    assert!(tripwire::sensitive_target("/tmp/any/connectors.json").is_some());

    // 1. Direct delivery under fast_internal (require_approval: false) -> delivers with host-substituted secret.
    let tool_res = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({
            "connector_id": conn_id,
            "payload": { "invoice": "INV-100", "total": 420.50 }
        }),
    )
    .unwrap();
    assert!(tool_res.contains("delivered via connector"), "{tool_res}");
    assert!(!tool_res.contains(secret_val));
    {
        let reqs = captured.lock().unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].method, "POST");
        assert!(reqs[0]
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == &format!("Bearer {secret_val}")));
        assert!(reqs[0].body.contains("INV-100"));
    }

    // 2. Active-run tripwire: redacts connector secrets AND blocks egress tools carrying secret values.
    {
        let _arm = tripwire::arm(tripwire::ArmSpec {
            root: td.cfg.data_dir.clone(),
            person_dir: td.cfg.data_dir.clone(),
            workspace: td.cfg.workspace_dir.clone(),
            memory_path: td.cfg.memory_path.clone(),
            person_id: "default".to_string(),
            run_id: "run-sec-test".to_string(),
            thread_id: "t-1".to_string(),
            provenance: "deliver the invoice summary".to_string(),
            secrets: Vec::new(),
        });
        let echoed = tripwire::redact(&format!("summary accidentally includes {secret_val}!"));
        assert!(!echoed.contains(secret_val));
        assert!(echoed.contains("[redacted]"));

        let pre = tripwire::pre_tool(
            "invoke_connector",
            &json!({ "connector_id": conn_id, "payload": { "leak": secret_val } }),
            false,
        );
        assert!(matches!(pre, tripwire::PreTool::Refuse(_)));
        assert!(tripwire::halted().is_some());
    }

    // 3. Switch to Strict Client -> require_approval is forced true!
    let (st_pol1, pol1) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    assert_eq!(st_pol1, 200);
    assert_eq!(pol1["profile"], json!("strict_client"));

    let (st_get1, get1) = http_req(addr, "GET", "/api/connectors", "");
    assert_eq!(st_get1, 200);
    assert_eq!(get1["outbound"][0]["effective_require_approval"], json!(true));

    // invoke_connector parks connector_delivery decision instead of sending.
    let held_res = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({
            "connector_id": conn_id,
            "payload": { "invoice": "INV-200", "total": 990.00 }
        }),
    )
    .unwrap();
    assert!(held_res.contains("parked for your person's approval"), "{held_res}");
    assert_eq!(captured.lock().unwrap().len(), 1, "must not send while held");

    let decisions = records::load_decisions(&td.cfg.data_dir);
    let dec = decisions
        .iter()
        .find(|d| d["kind"] == "connector_delivery" && d["status"] == "pending")
        .expect("pending connector_delivery decision");
    let dec_id = dec["id"].as_str().unwrap().to_string();
    assert_eq!(dec["connector_id"], json!(conn_id));

    // Approve decision -> executes real outbound HTTP delivery!
    let (st_app, app_json) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec_id}/approve"),
        "",
    );
    assert_eq!(st_app, 200, "{app_json}");
    assert_eq!(app_json["decision"]["status"], json!("approved"));
    assert!(app_json["outcome"]
        .as_str()
        .unwrap()
        .contains("Approved and delivered"));
    assert_eq!(captured.lock().unwrap().len(), 2);

    // 4. Switch to Air-Gapped -> hard-blocked for both tool and /test endpoint!
    let (st_pol2, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"air_gapped"}"#,
    );
    assert_eq!(st_pol2, 200);

    let err = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({ "connector_id": conn_id, "payload": { "invoice": "INV-300" } }),
    )
    .unwrap_err();
    assert!(err.contains("Air-Gapped"), "{err}");

    let (st_test, test_json) = http_req(
        addr,
        "POST",
        &format!("/api/connectors/outbound/{conn_id}/test"),
        "{}",
    );
    assert_eq!(st_test, 403, "{test_json}");
    assert_eq!(captured.lock().unwrap().len(), 2, "must not send under air_gapped");

    // Verify DeliveryLog captured ok, held, ok (approved), and blocked entries without exposing secret_val.
    let (_, final_list, final_raw) =
        http_req_headers(addr, "GET", "/api/connectors", &[], b"");
    assert!(!final_raw.contains(secret_val));
    let logs = final_list["delivery_log"].as_array().unwrap();
    assert!(logs.iter().any(|l| l["status"] == "ok"));
    assert!(logs.iter().any(|l| l["status"] == "held"));
    assert!(logs.iter().any(|l| l["status"] == "blocked"));
}
