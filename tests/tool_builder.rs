//! Wave 2 Step 3b Integration Tests: Real Tool Builder (sandbox test gate,
//! approval packaging, shared library publish, cross-employee install & execution).

use ergon_rs::{execute_tool_full, library, serve_with_gate, system_prompt, Config, Person, ToolCtx};
use serde_json::{json, Value};
use std::io::{Read, Write};
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
            "tool-builder-{}-{}-{label}",
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

fn line_counter_source() -> &'static str {
    r#"pub fn main(path) {
    let text = host_workspace_read(path);
    if text.starts_with("error:") {
        return text;
    }
    let n = 0;
    for _line in text.split("\n") {
        n += 1;
    }
    let res = host_workspace_write("line_count.txt", `lines=${n}`);
    if res != "ok" {
        return res;
    }
    `lines=${n}`
}"#
}

fn line_counter_tests() -> Value {
    json!([
        {
            "name": "counts lines and writes report",
            "setup_files": { "sample.txt": "alpha\nbeta\ngamma" },
            "args": ["sample.txt"],
            "expect_ok": true,
            "expect_output_contains": "lines=3",
            "expect_files": { "line_count.txt": "lines=3" }
        },
        {
            "name": "missing input returns error",
            "setup_files": {},
            "args": ["missing.txt"],
            "expect_ok": false,
            "expect_output_contains": "missing.txt",
            "expect_files": {}
        }
    ])
}

#[test]
fn tool_builder_rejects_bad_code_tests_and_escapes_and_enforces_approval_lifecycle() {
    let env = TestEnv::new("lifecycle");
    let addr = spawn_server(env.cfg.clone());

    // Setup Admin (Employee A) and Member (Employee B).
    let setup = http_req(
        addr,
        "POST",
        "/api/auth/setup",
        Some(
            &json!({
                "username": "alice_a",
                "password": "AdminPassword!123",
                "display_name": "Alice A"
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
                "display_name": "Bob B"
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

    let login_b = http_req(
        addr,
        "POST",
        "/api/auth/login",
        Some(&json!({ "username": "bob_b", "password": "UserPassword!123" }).to_string()),
        None,
    );
    assert_eq!(login_b.status, 200);
    let cookie_b = login_b.session_cookie().unwrap();

    // 1. Compile error rejection.
    let bad_compile = http_req(
        addr,
        "POST",
        "/api/tools/build",
        Some(
            &json!({
                "name": "Broken Tool",
                "description": "Does not compile",
                "params": ["x"],
                "source": "pub fn main(x) { let = ; }",
                "tests": [{ "name": "t1", "args": ["a"], "expect_ok": true }]
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(bad_compile.status, 400);

    // 2. Failing test rejection.
    let bad_test = http_req(
        addr,
        "POST",
        "/api/tools/build",
        Some(
            &json!({
                "name": "Wrong Output",
                "description": "Fails test assertion",
                "params": ["path"],
                "source": line_counter_source(),
                "tests": [{
                    "name": "expects wrong count",
                    "setup_files": { "a.txt": "one\ntwo" },
                    "args": ["a.txt"],
                    "expect_ok": true,
                    "expect_output_contains": "lines=999"
                }]
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(bad_test.status, 400);
    assert!(bad_test.body.contains("verification failed"));

    // 3. Sandbox escape rejection (`host_memory_write` and `../escape`).
    let bad_mem = http_req(
        addr,
        "POST",
        "/api/tools/test",
        Some(
            &json!({
                "params": ["x"],
                "source": "pub fn main(x) { host_memory_write(x); \"ok\" }",
                "tests": [{ "name": "t1", "args": ["hi"], "expect_ok": true }]
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(bad_mem.status, 400);

    let bad_escape = http_req(
        addr,
        "POST",
        "/api/tools/test",
        Some(
            &json!({
                "params": ["path"],
                "source": line_counter_source(),
                "tests": [{
                    "name": "traversal",
                    "setup_files": { "../escape.txt": "nope" },
                    "args": ["../escape.txt"],
                    "expect_ok": true
                }]
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(bad_escape.status, 400);

    // 4. Stage valid tool via /api/tools/build -> pending_approval, NOT yet callable.
    let build_ok = http_req(
        addr,
        "POST",
        "/api/tools/build",
        Some(
            &json!({
                "name": "Line Counter",
                "description": "Counts lines in a workspace file and writes line_count.txt.",
                "params": ["path"],
                "source": line_counter_source(),
                "tests": line_counter_tests()
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(build_ok.status, 201, "build_ok: {}", build_ok.body);
    let build_json = build_ok.json();
    let decision_id = build_json["decision_id"].as_str().unwrap().to_string();
    assert_eq!(build_json["package"]["status"], "pending_approval");
    assert_eq!(build_json["package"]["test_report"]["passed"], 2);

    // Before approval: uncallable via HTTP /api/tools/line-counter/run and absent from library & suite.
    let pre_run = http_req(
        addr,
        "POST",
        "/api/tools/line-counter/run",
        Some(&json!({ "args": { "path": "doc.txt" } }).to_string()),
        Some(&cookie_a),
    );
    assert_eq!(pre_run.status, 404, "unapproved tool must not be in library");
    let lib_pre = http_req(addr, "GET", "/api/library", None, Some(&cookie_a));
    assert!(!lib_pre.body.contains("\"line-counter\""));

    // 5. Approve decision -> installs for Employee A + publishes to shared library!
    let approve = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{decision_id}/approve"),
        Some("{}"),
        Some(&cookie_a),
    );
    assert_eq!(approve.status, 200, "approve: {}", approve.body);
    assert!(approve.json()["outcome"]
        .as_str()
        .unwrap()
        .contains("Approved and installed"));

    let tools_after = http_req(addr, "GET", "/api/tools", None, Some(&cookie_a));
    assert_eq!(tools_after.status, 200);
    assert!(tools_after.body.contains("\"line-counter\""));

    // Write a workspace file for Employee A and run the new tool via HTTP!
    let put_a = http_req(
        addr,
        "PUT",
        "/api/files/content",
        Some(&json!({ "path": "notes.txt", "content": "one\ntwo\nthree\nfour" }).to_string()),
        Some(&cookie_a),
    );
    assert_eq!(put_a.status, 200, "put_a: {}", put_a.body);

    let run_a = http_req(
        addr,
        "POST",
        "/api/tools/line-counter/run",
        Some(&json!({ "args": { "path": "notes.txt" } }).to_string()),
        Some(&cookie_a),
    );
    assert_eq!(run_a.status, 200, "run_a: {}", run_a.body);
    assert_eq!(run_a.json()["output"], "lines=4");

    // Verify Employee A's system_prompt includes `TOOLS IN THIS PERSON'S SUITE` with `line-counter`.
    let alice_pid = http_req(addr, "GET", "/api/auth/me", None, Some(&cookie_a)).json()["user"]
        ["person_id"]
        .as_str()
        .unwrap()
        .to_string();
    let alice_cfg = if alice_pid == "default" {
        env.cfg.clone()
    } else {
        env.cfg.person_config(&Person {
            id: alice_pid.clone(),
            name: alice_pid.clone(),
            role: String::new(),
        })
    };
    let lib_loaded = library::Library::load(&env.cfg.data_dir);
    let suite_a = library::load_suite(&alice_cfg.data_dir).resolved(&lib_loaded);
    let prompt_a = system_prompt(&alice_cfg, &suite_a);
    assert!(prompt_a.contains("TOOLS IN THIS PERSON'S SUITE"));
    assert!(prompt_a.contains("line-counter"));

    // 6. Employee B cannot run it before installing, installs from shared library, then runs in own workspace!
    let pre_run_b = http_req(
        addr,
        "POST",
        "/api/tools/line-counter/run",
        Some(&json!({ "args": { "path": "b.txt" } }).to_string()),
        Some(&cookie_b),
    );
    assert_eq!(pre_run_b.status, 403);

    let install_b = http_req(
        addr,
        "POST",
        "/api/suite/install",
        Some(&json!({ "item_id": "line-counter" }).to_string()),
        Some(&cookie_b),
    );
    assert_eq!(install_b.status, 200);

    let put_b = http_req(
        addr,
        "PUT",
        "/api/files/content",
        Some(&json!({ "path": "b.txt", "content": "x\ny" }).to_string()),
        Some(&cookie_b),
    );
    assert_eq!(put_b.status, 200);

    let run_b = http_req(
        addr,
        "POST",
        "/api/tools/line-counter/run",
        Some(&json!({ "args": { "path": "b.txt" } }).to_string()),
        Some(&cookie_b),
    );
    assert_eq!(run_b.status, 200, "run_b: {}", run_b.body);
    assert_eq!(run_b.json()["output"], "lines=2");

    // Also callable via agent execute_tool_full for Employee B!
    let bob_cfg = env.cfg.person_config(&Person {
        id: bob_pid.clone(),
        name: "Bob B".to_string(),
        role: "member".to_string(),
    });
    let bob_ctx = ToolCtx {
        library: Arc::new(Mutex::new(library::Library::load(&env.cfg.data_dir))),
        root_data_dir: env.cfg.data_dir.clone(),
        person_id: bob_pid,
    };
    let tool_out = execute_tool_full(
        &bob_cfg,
        &bob_ctx,
        "line-counter",
        &json!({ "path": "b.txt" }),
    )
    .unwrap();
    assert_eq!(tool_out, "lines=2");

    // 7. Deny flow: stage another tool and deny -> never published or installed.
    let build_2 = http_req(
        addr,
        "POST",
        "/api/tools/build",
        Some(
            &json!({
                "name": "Declined Counter",
                "description": "Will be denied",
                "params": ["path"],
                "source": line_counter_source(),
                "tests": line_counter_tests()
            })
            .to_string(),
        ),
        Some(&cookie_a),
    );
    assert_eq!(build_2.status, 201);
    let d2_id = build_2.json()["decision_id"].as_str().unwrap().to_string();

    let deny = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{d2_id}/deny"),
        Some("{}"),
        Some(&cookie_a),
    );
    assert_eq!(deny.status, 200);
    let lib_final = http_req(addr, "GET", "/api/library", None, Some(&cookie_a));
    assert!(!lib_final.body.contains("declined-counter"));
    let tools_final = http_req(addr, "GET", "/api/tools", None, Some(&cookie_a));
    let declined_pkg = tools_final.json()["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "Declined Counter")
        .unwrap()
        .clone();
    assert_eq!(declined_pkg["status"], "denied");
}
