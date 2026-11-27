//! Wave 2 Step 3c Integration Tests: Proactive Offers, Per-Person Notifications,
//! Push Subscriptions (with egress guard), Offer Approve/Deny Aliases, and SSE `push_notification`.

use ergon_rs::{library, records, serve_with_gate, Config, Person};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    root: PathBuf,
    cfg: Config,
}

impl TestEnv {
    fn new(label: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "proactive-offers-push-{}-{}-{label}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "stub".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct HttpResp {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpResp {
    fn session_cookie(&self) -> Option<String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case("set-cookie") {
                if let Some(first) = v.split(';').next() {
                    if first.trim().starts_with("ergon_session=") {
                        return Some(first.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|e| panic!("invalid JSON ({}): {e}\n{}", self.status, self.body))
    }
}

fn spawn_server(cfg: Config) -> SocketAddr {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve_with_gate(listener, cfg, None);
    });
    for _ in 0..50 {
        if TcpStream::connect(addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    addr
}

fn http_req(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
    cookie: Option<&str>,
) -> HttpResp {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let bytes = body.unwrap_or("").as_bytes();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        addr.port()
    );
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
    }
    if let Some(c) = cookie {
        req.push_str(&format!("Cookie: {c}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !bytes.is_empty() {
        stream.write_all(bytes).unwrap();
    }
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw).to_string();
    let (head, resp_body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .collect();
    HttpResp {
        status,
        headers,
        body: resp_body.to_string(),
    }
}

#[test]
fn notifications_push_egress_cross_team_offers_and_sse_delivery() {
    let env = TestEnv::new("push");

    // Pre-seed a shared tool from Employee A ("alice") in library.json before spawning server
    // so State loads it immediately.
    let mut lib = library::Library::load(&env.cfg.data_dir);
    let tool_src = r#"pub fn main(note) {
    let res = host_workspace_write("invoice_check.txt", note);
    if res != "ok" { return res; }
    "ok"
}"#
    .to_string();
    let published = lib
        .publish_tool(
            "Invoice Matcher",
            "Matches vendor invoice totals and writes invoice_check.txt",
            vec!["note".to_string()],
            tool_src.clone(),
            vec![json!({
                "name": "basic",
                "args": ["paid"],
                "expect_ok": true,
                "expect_output_contains": "ok"
            })],
            json!({ "passed": 1, "total": 1, "all_passed": true }),
            "alice",
        )
        .unwrap();
    let second = lib
        .publish_skill(
            "Quarterly Summary Helper",
            "Compiles quarterly finance metrics into summary notes",
            vec!["entry".to_string()],
            "pub fn main(entry) { entry }".to_string(),
            "alice",
        )
        .unwrap();
    lib.save(&env.cfg.data_dir);

    let addr = spawn_server(env.cfg.clone());

    // Setup Admin (Alice) and Member (Bob).
    let setup = http_req(
        addr,
        "POST",
        "/api/auth/setup",
        Some(
            &json!({
                "username": "alice_a",
                "password": "AdminPassword!123",
                "display_name": "Alice"
            })
            .to_string(),
        ),
        None,
    );
    assert_eq!(setup.status, 201);
    let cookie_a = setup.session_cookie().unwrap();

    let create_b = http_req(
        addr,
        "POST",
        "/api/auth/users",
        Some(
            &json!({
                "username": "bob_b",
                "password": "UserPassword!123",
                "role": "member",
                "display_name": "Bob"
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(create_b.status, 201);
    let bob_pid = create_b.json()["user"]["person_id"]
        .as_str()
        .unwrap()
        .to_string();

    let cookie_b = http_req(
        addr,
        "POST",
        "/api/auth/login",
        Some(&json!({ "username": "bob_b", "password": "UserPassword!123" }).to_string()),
        None,
    )
    .session_cookie()
    .unwrap();

    // 1. Push subscriptions + egress guard.
    let sub_ok = http_req(
        addr,
        "POST",
        "/api/push/subscribe",
        Some(
            &json!({
                "channel": "web_push",
                "endpoint": "browser://local",
                "label": "Bob iPhone"
            })
            .to_string(),
        ),
        Some(&cookie_b),
    );
    assert_eq!(sub_ok.status, 201, "subscribe local: {}", sub_ok.body);
    let sub_id = sub_ok.json()["subscription"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let sub_ssrf = http_req(
        addr,
        "POST",
        "/api/push/subscribe",
        Some(
            &json!({
                "channel": "webhook",
                "endpoint": "http://169.254.169.254/latest/meta-data",
                "label": "Blocked SSRF"
            })
            .to_string(),
        ),
        Some(&cookie_b),
    );
    assert_eq!(sub_ssrf.status, 400, "SSRF webhook must be blocked");

    let sub_http = http_req(
        addr,
        "POST",
        "/api/push/subscribe",
        Some(
            &json!({
                "channel": "web_push",
                "endpoint": "http://insecure.example.com/push",
                "label": "Insecure"
            })
            .to_string(),
        ),
        Some(&cookie_b),
    );
    assert_eq!(sub_http.status, 400, "non-https web_push must be rejected");

    let list_subs = http_req(addr, "GET", "/api/push/subscriptions", None, Some(&cookie_b));
    assert_eq!(list_subs.status, 200);
    assert_eq!(list_subs.json()["subscriptions"].as_array().unwrap().len(), 1);

    let del_sub = http_req(
        addr,
        "DELETE",
        &format!("/api/push/subscribe/{sub_id}"),
        None,
        Some(&cookie_b),
    );
    assert_eq!(del_sub.status, 200);
    assert_eq!(
        http_req(addr, "GET", "/api/push/subscriptions", None, Some(&cookie_b))
            .json()["subscriptions"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    // 2. Seed finished work for Bob matching both shared library items + SSE listener.
    let bob_cfg = env.cfg.person_config(&Person {
        id: bob_pid.clone(),
        name: "Bob".to_string(),
        role: "member".to_string(),
    });
    bob_cfg.ensure_dirs().unwrap();
    for i in 1..=2 {
        records::append_work_with_usage(
            &bob_cfg,
            &format!("Reconcile vendor invoice matcher batch {i}"),
            "Matched vendor invoice totals",
            &[],
            &format!("t-inv-{i}"),
            true,
            None,
            None,
        );
        records::append_work_with_usage(
            &bob_cfg,
            &format!("Compile quarterly summary finance metrics {i}"),
            "Compiled quarterly finance metrics",
            &[],
            &format!("t-qtr-{i}"),
            true,
            None,
            None,
        );
    }

    // Open SSE stream for Bob BEFORE triggering analyze + tool build.
    let mut sse_stream = TcpStream::connect(addr).unwrap();
    sse_stream
        .set_read_timeout(Some(Duration::from_secs(6)))
        .unwrap();
    let sse_req = format!(
        "GET /api/runs/stream HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nCookie: {cookie_b}\r\n\r\n",
        addr.port()
    );
    sse_stream.write_all(sse_req.as_bytes()).unwrap();
    let mut sse_reader = BufReader::new(sse_stream);
    // Read HTTP headers and initial status frame.
    let mut initial_seen = false;
    let mut line = String::new();
    while sse_reader.read_line(&mut line).unwrap_or(0) > 0 {
        if line.contains("event: status") {
            initial_seen = true;
        }
        if initial_seen && line == "\n" {
            break;
        }
        line.clear();
    }
    assert!(initial_seen, "expected initial SSE status frame");

    // 3. Call POST /api/offers/analyze as Bob -> creates cross-team offers + notifications + SSE push_notification!
    let analyze = http_req(addr, "POST", "/api/offers/analyze", Some("{}"), Some(&cookie_b));
    assert_eq!(analyze.status, 200, "analyze: {}", analyze.body);
    let generated = analyze.json()["generated"].as_array().unwrap().clone();
    assert_eq!(generated.len(), 2, "expected 2 cross-team offers: {generated:?}");
    let inv_offer_id = generated
        .iter()
        .find(|o| o["item_id"] == published.id)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let qtr_offer_id = generated
        .iter()
        .find(|o| o["item_id"] == second.id)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Read SSE stream to verify `event: push_notification` arrived live!
    let mut found_push_event = false;
    let mut sse_buf = String::new();
    for _ in 0..40 {
        let mut l = String::new();
        match sse_reader.read_line(&mut l) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                sse_buf.push_str(&l);
                if sse_buf.contains("event: push_notification") && l == "\n" {
                    found_push_event = true;
                    break;
                }
            }
        }
    }
    assert!(
        found_push_event,
        "expected live SSE push_notification event, got:\n{sse_buf}"
    );

    // 4. Approve offer via `/approve` alias -> installs tool into Bob's suite!
    let approve_offer = http_req(
        addr,
        "POST",
        &format!("/api/offers/{inv_offer_id}/approve"),
        Some("{}"),
        Some(&cookie_b),
    );
    assert_eq!(approve_offer.status, 200, "approve offer: {}", approve_offer.body);
    assert_eq!(approve_offer.json()["offer"]["status"], "accepted");
    let bob_suite = library::load_suite(&bob_cfg.data_dir);
    assert!(bob_suite.installed.contains(&published.id));

    // Dismiss offer via `/deny` alias -> never re-offered!
    let deny_offer = http_req(
        addr,
        "POST",
        &format!("/api/offers/{qtr_offer_id}/deny"),
        Some("{}"),
        Some(&cookie_b),
    );
    assert_eq!(deny_offer.status, 200);
    assert_eq!(deny_offer.json()["offer"]["status"], "dismissed");

    let re_analyze = http_req(addr, "POST", "/api/offers/analyze", Some("{}"), Some(&cookie_b));
    assert_eq!(re_analyze.status, 200);
    assert_eq!(
        re_analyze.json()["generated"].as_array().unwrap().len(),
        0,
        "dismissed/accepted offers must not be regenerated"
    );

    // 5. Notifications list & mark-read!
    let notifs = http_req(addr, "GET", "/api/notifications", None, Some(&cookie_b));
    assert_eq!(notifs.status, 200);
    let notif_json = notifs.json();
    let n_arr = notif_json["notifications"].as_array().unwrap();
    assert_eq!(n_arr.len(), 2);
    assert_eq!(notif_json["unread_count"], 2);

    let first_nid = n_arr[0]["id"].as_str().unwrap();
    let mark_read = http_req(
        addr,
        "POST",
        &format!("/api/notifications/{first_nid}/read"),
        Some("{}"),
        Some(&cookie_b),
    );
    assert_eq!(mark_read.status, 200);
    assert_eq!(mark_read.json()["notification"]["read"], true);

    let after_read = http_req(addr, "GET", "/api/notifications", None, Some(&cookie_b)).json();
    assert_eq!(after_read["unread_count"], 1);
}
