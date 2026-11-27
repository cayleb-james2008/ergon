//! Wave 3 Step 3f Integration Tests: Scoped API Keys, Outbound HMAC Webhooks, & Authenticated Full MCP Server.

use ergon_rs::{serve_with_gate, tripwire, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
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
            "backend-api-mcp-wh-{}-{}-{label}",
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

fn http_req(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
    cookie: Option<&str>,
    extra_headers: &[(&str, &str)],
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
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !bytes.is_empty() {
        stream.write_all(bytes).unwrap();
    }
    stream.flush().unwrap();
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let (head, body_str) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    HttpResp {
        status,
        headers,
        body: body_str.to_string(),
    }
}

#[derive(Clone, Debug)]
struct CapturedWebhook {
    event: String,
    delivery: String,
    signature: String,
    raw_body: Vec<u8>,
}

fn spawn_webhook_receiver() -> (SocketAddr, Arc<Mutex<Vec<CapturedWebhook>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let cap_clone = captured.clone();

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let cap = cap_clone.clone();
            std::thread::spawn(move || {
                let mut stream = stream;
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut content_len = 0usize;
                let mut event = String::new();
                let mut delivery = String::new();
                let mut signature = String::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" || line == "\n" {
                        break;
                    }
                    if let Some((k, v)) = line.split_once(':') {
                        let kl = k.trim().to_ascii_lowercase();
                        let val = v.trim().to_string();
                        match kl.as_str() {
                            "content-length" => content_len = val.parse().unwrap_or(0),
                            "x-ergon-event" => event = val,
                            "x-ergon-delivery" => delivery = val,
                            "x-ergon-signature" => signature = val,
                            _ => {}
                        }
                    }
                }
                let mut body = vec![0u8; content_len];
                let _ = reader.read_exact(&mut body);
                cap.lock().unwrap().push(CapturedWebhook {
                    event,
                    delivery,
                    signature,
                    raw_body: body,
                });
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
                let _ = stream.flush();
            });
        }
    });

    (addr, captured)
}

#[test]
fn scoped_api_keys_outbound_hmac_webhooks_and_full_mcp_server() {
    let env = TestEnv::new("all");
    let addr = spawn_server(env.cfg.clone());

    // 1. Setup admin & pm user accounts
    let r_setup = http_req(
        addr,
        "POST",
        "/api/auth/setup",
        Some(r#"{"username":"alice","password":"password-1234","display_name":"Alice Admin"}"#),
        None,
        &[],
    );
    assert_eq!(r_setup.status, 201, "{}", r_setup.body);
    let admin_cookie = r_setup.session_cookie().unwrap();
    let admin_pid = r_setup.json()["user"]["person_id"].as_str().unwrap().to_string();

    let r_pm = http_req(
        addr,
        "POST",
        "/api/auth/users",
        Some(r#"{"username":"carol","password":"password-1234","role":"pm","display_name":"Carol PM"}"#),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_pm.status, 201, "{}", r_pm.body);
    let pm_pid = r_pm.json()["user"]["person_id"].as_str().unwrap().to_string();

    let r_mem = http_req(
        addr,
        "POST",
        "/api/auth/users",
        Some(r#"{"username":"bob","password":"password-1234","role":"member","display_name":"Bob Member"}"#),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_mem.status, 201, "{}", r_mem.body);
    let mem_pid = r_mem.json()["user"]["person_id"].as_str().unwrap().to_string();

    // 2. Create scoped API keys (read-only, mcp-read-only, mcp-read-write, webhooks)
    let r_ro = http_req(
        addr,
        "POST",
        "/api/keys",
        Some(&json!({"name": "ro-key", "scopes": ["read"], "person_id": pm_pid}).to_string()),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_ro.status, 201, "{}", r_ro.body);
    let ro_key = r_ro.json()["key"].as_str().unwrap().to_string();
    assert!(ro_key.starts_with("ergk_"));

    let r_mcp_ro = http_req(
        addr,
        "POST",
        "/api/keys",
        Some(&json!({"name": "mcp-ro", "scopes": ["mcp", "read"], "person_id": pm_pid}).to_string()),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_mcp_ro.status, 201);
    let mcp_ro_key = r_mcp_ro.json()["key"].as_str().unwrap().to_string();

    let r_mcp_rw = http_req(
        addr,
        "POST",
        "/api/keys",
        Some(&json!({"name": "mcp-rw", "scopes": ["mcp", "read", "write"], "person_id": pm_pid}).to_string()),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_mcp_rw.status, 201);
    let mcp_rw_key = r_mcp_rw.json()["key"].as_str().unwrap().to_string();

    let r_mem_rw = http_req(
        addr,
        "POST",
        "/api/keys",
        Some(&json!({"name": "mem-rw", "scopes": ["mcp", "read", "write"], "person_id": mem_pid}).to_string()),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_mem_rw.status, 201);
    let mem_rw_key = r_mem_rw.json()["key"].as_str().unwrap().to_string();

    let r_wh_key = http_req(
        addr,
        "POST",
        "/api/keys",
        Some(&json!({"name": "wh-key", "scopes": ["webhooks"], "person_id": admin_pid}).to_string()),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_wh_key.status, 201);
    let wh_key = r_wh_key.json()["key"].as_str().unwrap().to_string();

    // Verify GET /api/keys redacts plaintext secrets and api_keys.json is 0600 without plaintext
    let r_list_keys = http_req(addr, "GET", "/api/keys", None, Some(&admin_cookie), &[]);
    assert_eq!(r_list_keys.status, 200);
    assert!(!r_list_keys.body.contains(&ro_key));
    let keys_disk = std::fs::read_to_string(env.cfg.data_dir.join("api_keys.json")).unwrap();
    assert!(!keys_disk.contains(&ro_key));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(env.cfg.data_dir.join("api_keys.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    // 3. Enforce 401 & 403 on missing/insufficient scopes
    let r_bad = http_req(
        addr,
        "GET",
        "/api/brief",
        None,
        None,
        &[("Authorization", "Bearer ergk_invalid_token_000")],
    );
    assert_eq!(r_bad.status, 401);

    let r_ok_read = http_req(
        addr,
        "GET",
        "/api/brief",
        None,
        None,
        &[("X-Ergon-Api-Key", &ro_key)],
    );
    assert_eq!(r_ok_read.status, 200);

    let r_403_write = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(r#"{"content":"nope"}"#),
        None,
        &[("Authorization", &format!("Bearer {ro_key}"))],
    );
    assert_eq!(r_403_write.status, 403);

    let r_403_keys = http_req(
        addr,
        "GET",
        "/api/keys",
        None,
        None,
        &[("X-Ergon-Api-Key", &ro_key)],
    );
    assert_eq!(r_403_keys.status, 403);

    // 4. Outbound webhook subscription SSRF guard + HMAC-SHA256 delivery
    let (wh_addr, captured_wh) = spawn_webhook_receiver();
    let wh_url = format!("http://127.0.0.1:{}/hook", wh_addr.port());

    // Blocked by default (loopback SSRF)
    let r_ssrf = http_req(
        addr,
        "POST",
        "/api/webhooks/subscriptions",
        Some(&json!({"url": wh_url, "events": ["tool.published", "evolution.promoted"]}).to_string()),
        None,
        &[("X-Ergon-Api-Key", &wh_key)],
    );
    assert_eq!(r_ssrf.status, 400, "{}", r_ssrf.body);

    // Enable ScopedLocalPolicy for local test webhook receiver
    let _local_policy = tripwire::ScopedLocalPolicy::new(&env.cfg.data_dir, Some(true), Some(false));
    let wh_secret = "super-secret-signing-key-123";
    let r_sub = http_req(
        addr,
        "POST",
        "/api/webhooks/subscriptions",
        Some(
            &json!({
                "url": wh_url,
                "events": ["decision.created", "decision.settled", "evolution.promoted"],
                "secret": wh_secret,
                "label": "Test Hook"
            })
            .to_string(),
        ),
        None,
        &[("X-Ergon-Api-Key", &wh_key)],
    );
    assert_eq!(r_sub.status, 201, "{}", r_sub.body);
    let sub_id = r_sub.json()["subscription"]["id"].as_str().unwrap().to_string();

    // Secret must NOT be stored plaintext in webhook_subscriptions.json or returned in GET
    let wh_disk =
        std::fs::read_to_string(env.cfg.data_dir.join("webhook_subscriptions.json")).unwrap();
    assert!(!wh_disk.contains(wh_secret));
    assert!(wh_disk.contains("vault:"));
    let r_wh_list = http_req(
        addr,
        "GET",
        "/api/webhooks/subscriptions",
        None,
        None,
        &[("X-Ergon-Api-Key", &wh_key)],
    );
    assert_eq!(r_wh_list.status, 200);
    assert!(!r_wh_list.body.contains(wh_secret));

    // Trigger evolution.promoted webhook via /api/evolution/cases + /api/evolution/propose
    let _ = http_req(
        addr,
        "POST",
        "/api/evolution/cases",
        Some(
            &json!({
                "case_id": "wh-imp-1",
                "artifact_id": "default-improver",
                "artifact_kind": "improver",
                "prompt_or_input": "Check promotion webhook",
                "checks": [
                    { "type": "expect_ok" },
                    { "type": "output_contains", "value": "rigorous" }
                ]
            })
            .to_string(),
        ),
        Some(&admin_cookie),
        &[],
    );
    let r_prom = http_req(
        addr,
        "POST",
        "/api/evolution/propose",
        Some(
            &json!({
                "artifact_id": "default-improver",
                "kind": "improver",
                "candidate_content": "Apply rigorous verification before promoting.",
                "note": "Rigorous v2"
            })
            .to_string(),
        ),
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_prom.status, 200, "{}", r_prom.body);
    assert_eq!(r_prom.json()["promoted"], true);

    // Verify HMAC-SHA256 signature on captured webhook
    {
        let hooks = captured_wh.lock().unwrap();
        let ev = hooks
            .iter()
            .find(|h| h.event == "evolution.promoted")
            .expect("expected evolution.promoted webhook");
        assert!(!ev.delivery.is_empty());
        let expected_sig = format!(
            "sha256={}",
            tripwire::hex(&tripwire::hmac_sha256(wh_secret.as_bytes(), &ev.raw_body))
        );
        assert_eq!(ev.signature, expected_sig);
    }

    // 5. Full Authenticated MCP Server (/api/mcp/inbound)
    // Auto-allowed with valid "mcp" scoped API key even without toggling /api/mcp/inbound/config!
    let r_mcp_list_ro = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}).to_string()),
        None,
        &[("Authorization", &format!("Bearer {mcp_ro_key}"))],
    );
    assert_eq!(r_mcp_ro_status(&r_mcp_list_ro), 200, "{}", r_mcp_list_ro.body);
    let ro_tools = r_mcp_list_ro.json()["result"]["tools"].as_array().unwrap().clone();
    assert_eq!(ro_tools.len(), 6);

    // Read-only key attempting write tool -> 403 JSON-RPC -32003
    let r_mcp_write_denied = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "write_workspace_file",
                    "arguments": { "path": "mcp-note.txt", "content": "blocked" }
                }
            })
            .to_string(),
        ),
        None,
        &[("X-Ergon-Api-Key", &mcp_ro_key)],
    );
    assert_eq!(r_mcp_write_denied.status, 403, "{}", r_mcp_write_denied.body);
    assert_eq!(r_mcp_write_denied.json()["error"]["code"], -32003);

    // Read-write key sees all 10 tools and executes write + read tools + resources
    let r_mcp_list_rw = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(&json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list", "params": {}}).to_string()),
        None,
        &[("Authorization", &format!("Bearer {mcp_rw_key}"))],
    );
    assert_eq!(r_mcp_list_rw.status, 200);
    assert_eq!(
        r_mcp_list_rw.json()["result"]["tools"].as_array().unwrap().len(),
        10
    );

    let r_mcp_wfile = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "write_workspace_file",
                    "arguments": { "path": "notes/mcp.md", "content": "Hello from MCP!" }
                }
            })
            .to_string(),
        ),
        None,
        &[("Authorization", &format!("Bearer {mcp_rw_key}"))],
    );
    assert_eq!(r_mcp_wfile.status, 200, "{}", r_mcp_wfile.body);
    assert_eq!(r_mcp_wfile.json()["result"]["isError"], false);

    let r_mcp_wmem = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "write_company_memory",
                    "arguments": {
                        "category": "policy",
                        "scope": "company",
                        "content": "MCP writes obey RBAC."
                    }
                }
            })
            .to_string(),
        ),
        None,
        &[("Authorization", &format!("Bearer {mcp_rw_key}"))],
    );
    assert_eq!(r_mcp_wmem.status, 200, "{}", r_mcp_wmem.body);
    assert_eq!(r_mcp_wmem.json()["result"]["isError"], false);

    // Member role with mcp+write scope STILL gets -32003 on write_company_memory (RBAC enforced!)
    let r_mcp_mem_denied = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "write_company_memory",
                    "arguments": { "category": "fact", "scope": "company", "content": "forbidden" }
                }
            })
            .to_string(),
        ),
        None,
        &[("Authorization", &format!("Bearer {mem_rw_key}"))],
    );
    assert_eq!(r_mcp_mem_denied.status, 403, "{}", r_mcp_mem_denied.body);
    assert_eq!(r_mcp_mem_denied.json()["error"]["code"], -32003);

    // Build, approve, and install a tool in Carol's suite and run it via MCP run_tool
    let r_pm_login = http_req(
        addr,
        "POST",
        "/api/auth/login",
        Some(r#"{"username":"carol","password":"password-1234"}"#),
        None,
        &[],
    );
    assert_eq!(r_pm_login.status, 200);
    let pm_cookie = r_pm_login.session_cookie().unwrap();
    let r_tbuild = http_req(
        addr,
        "POST",
        "/api/tools/build",
        Some(
            &json!({
                "name": "MCP Adder",
                "description": "Adds suffix",
                "params": ["word"],
                "source": "pub fn main(word) { `${word}-ok` }",
                "tests": [{
                    "name": "adds ok",
                    "setup_files": {},
                    "args": ["ping"],
                    "expect_ok": true,
                    "expect_output_contains": "ping-ok"
                }]
            })
            .to_string(),
        ),
        Some(&pm_cookie),
        &[],
    );
    assert_eq!(r_tbuild.status, 201, "{}", r_tbuild.body);
    let dec_id = r_tbuild.json()["decision_id"].as_str().unwrap().to_string();
    let r_tapp = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec_id}/approve"),
        Some("{}"),
        Some(&pm_cookie),
        &[],
    );
    assert_eq!(r_tapp.status, 200, "{}", r_tapp.body);

    let r_mcp_run = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {
                    "name": "run_tool",
                    "arguments": { "id": "mcp-adder", "args": ["ping"] }
                }
            })
            .to_string(),
        ),
        None,
        &[("Authorization", &format!("Bearer {mcp_rw_key}"))],
    );
    assert_eq!(r_mcp_run.status, 200, "{}", r_mcp_run.body);
    assert!(r_mcp_run.json()["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("ping-ok"));

    // create_task + resources/list + resources/read
    let r_mcp_task = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(
            &json!({
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "create_task",
                    "arguments": { "message": "Summarize notes/mcp.md" }
                }
            })
            .to_string(),
        ),
        None,
        &[("Authorization", &format!("Bearer {mcp_rw_key}"))],
    );
    assert_eq!(r_mcp_task.status, 200, "{}", r_mcp_task.body);
    assert_eq!(r_mcp_task.json()["result"]["isError"], false);

    let r_res_list = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        Some(&json!({"jsonrpc": "2.0", "id": 9, "method": "resources/list", "params": {}}).to_string()),
        None,
        &[("Authorization", &format!("Bearer {mcp_ro_key}"))],
    );
    assert_eq!(r_res_list.status, 200);
    assert_eq!(
        r_res_list.json()["result"]["resources"].as_array().unwrap().len(),
        3
    );

    for uri in [
        "ergon://company/memory",
        "ergon://company/tools",
        "ergon://workspace/files",
    ] {
        let r_res_read = http_req(
            addr,
            "POST",
            "/api/mcp/inbound",
            Some(
                &json!({
                    "jsonrpc": "2.0",
                    "id": 10,
                    "method": "resources/read",
                    "params": { "uri": uri }
                })
                .to_string(),
            ),
            None,
            &[("Authorization", &format!("Bearer {mcp_ro_key}"))],
        );
        assert_eq!(r_res_read.status, 200, "{uri}: {}", r_res_read.body);
        assert!(r_res_read.json()["result"]["contents"][0]["text"]
            .as_str()
            .is_some());
    }

    // Delete webhook subscription & revoke API key
    let r_wh_del = http_req(
        addr,
        "DELETE",
        &format!("/api/webhooks/subscriptions/{sub_id}"),
        None,
        None,
        &[("X-Ergon-Api-Key", &wh_key)],
    );
    assert_eq!(r_wh_del.status, 200);

    let r_key_id = r_ro.json()["id"].as_str().unwrap().to_string();
    let r_key_del = http_req(
        addr,
        "DELETE",
        &format!("/api/keys/{r_key_id}"),
        None,
        Some(&admin_cookie),
        &[],
    );
    assert_eq!(r_key_del.status, 200);
    let r_revoked = http_req(
        addr,
        "GET",
        "/api/brief",
        None,
        None,
        &[("X-Ergon-Api-Key", &ro_key)],
    );
    assert_eq!(r_revoked.status, 401);
}

fn r_mcp_ro_status(r: &HttpResp) -> u16 {
    r.status
}
