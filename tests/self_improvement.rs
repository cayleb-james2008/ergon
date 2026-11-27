//! Wave 3 Step 3d Integration Tests: Outcome Scoring & Recursive Self-Improvement Loop.

use ergon_rs::{
    library, serve_with_gate, Config,
};
use serde_json::{json, Value};
use std::io::{Read, Write};
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
            "self-improve-{}-{}-{label}",
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

fn http_req(addr: SocketAddr, method: &str, path: &str, body: Option<&str>) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
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
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !bytes.is_empty() {
        stream.write_all(bytes).unwrap();
    }
    stream.flush().unwrap();
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let (head, body_str) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let json = serde_json::from_str(body_str).unwrap_or(Value::Null);
    (status, json, body_str.to_string())
}

#[test]
fn self_improvement_http_promote_reject_rollback_and_meta_improver() {
    let env = TestEnv::new("e2e");

    // Seed an initial library tool (v1) before starting the server.
    let v1_src = r#"pub fn main(name) {
    `hello ${name}`
}"#;
    let mut lib = library::Library::load(&env.cfg.data_dir);
    lib.publish_tool(
        "Greeter Evo",
        "Greets a person",
        vec!["name".to_string()],
        v1_src.to_string(),
        vec![],
        json!({"passed": 1, "total": 1}),
        "default",
    )
    .unwrap();
    lib.save(&env.cfg.data_dir);

    let addr = spawn_server(env.cfg.clone());

    // 1. Add deterministic replay case for greeter-evo
    let (st_case, case_json, raw_case) = http_req(
        addr,
        "POST",
        "/api/evolution/cases",
        Some(
            &json!({
                "case_id": "greeter-case-1",
                "artifact_id": "greeter-evo",
                "artifact_kind": "tool",
                "args": ["Alice"],
                "checks": [
                    { "type": "expect_ok" },
                    { "type": "output_contains", "value": "hello Alice" },
                    { "type": "output_contains", "value": "verified" },
                    { "type": "file_contains", "path": "greeting.txt", "needle": "verified" }
                ]
            })
            .to_string(),
        ),
    );
    assert_eq!(st_case, 201, "{raw_case}");
    assert_eq!(case_json["case"]["case_id"], "greeter-case-1");

    // 2. Propose v2 that beats v1 -> promoted
    let v2_src = r#"pub fn main(name) {
    let out = `hello ${name} - verified`;
    let res = host_workspace_write("greeting.txt", out.clone());
    if res != "ok" {
        return res;
    }
    out
}"#;
    let (st_v2, v2_json, raw_v2) = http_req(
        addr,
        "POST",
        "/api/evolution/propose",
        Some(
            &json!({
                "artifact_id": "greeter-evo",
                "kind": "tool",
                "candidate_content": v2_src,
                "candidate_params": ["name"],
                "note": "Write greeting.txt and tag verified"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_v2, 200, "{raw_v2}");
    assert_eq!(v2_json["promoted"], true, "{raw_v2}");
    assert_eq!(v2_json["from_version"], 1);
    assert_eq!(v2_json["to_version"], 2);
    assert_eq!(v2_json["regressions"], 0);

    // 3. Propose worse v3 -> rejected (champion stays v2)
    let v3_bad = r#"pub fn main(name) {
    "broken"
}"#;
    let (st_v3, v3_json, raw_v3) = http_req(
        addr,
        "POST",
        "/api/evolution/propose",
        Some(
            &json!({
                "artifact_id": "greeter-evo",
                "kind": "tool",
                "candidate_content": v3_bad,
                "candidate_params": ["name"],
                "note": "Broken regression"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_v3, 200, "{raw_v3}");
    assert_eq!(v3_json["promoted"], false, "{raw_v3}");
    assert!(v3_json["regressions"].as_u64().unwrap() > 0);
    assert!(!raw_v3.contains("Rune"), "must not expose internal engine name");

    // 4. Record live regression outcomes on v2 -> auto-rollback to v1
    let (st_o1, o1_json, raw_o1) = http_req(
        addr,
        "POST",
        "/api/evolution/outcomes",
        Some(
            &json!({
                "artifact_kind": "tool",
                "artifact_id": "greeter-evo",
                "artifact_version": 2,
                "run_id": "run-live-1",
                "task": "Greet Alice",
                "approved": false,
                "edits_needed": 1,
                "reruns": 1,
                "feedback": "Missing details"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_o1, 200, "{raw_o1}");
    assert!(o1_json["rolled_back_to"].is_null());

    let (st_o2, o2_json, raw_o2) = http_req(
        addr,
        "POST",
        "/api/evolution/outcomes",
        Some(
            &json!({
                "artifact_kind": "tool",
                "artifact_id": "greeter-evo",
                "artifact_version": 2,
                "run_id": "run-live-2",
                "task": "Greet Bob",
                "approved": false,
                "edits_needed": 2,
                "reruns": 1,
                "feedback": "Wrong format"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_o2, 200, "{raw_o2}");
    assert_eq!(o2_json["rolled_back_to"], 1, "{raw_o2}");

    // 5. Meta-improver: default-improver v2 beats v1 -> promoted
    let (st_icase, _, raw_icase) = http_req(
        addr,
        "POST",
        "/api/evolution/cases",
        Some(
            &json!({
                "case_id": "imp-1",
                "artifact_id": "default-improver",
                "artifact_kind": "improver",
                "prompt_or_input": "Improve artifact safely",
                "checks": [
                    { "type": "expect_ok" },
                    { "type": "output_contains", "value": "edge cases" },
                    { "type": "output_contains", "value": "passing tests" },
                    { "type": "min_lines", "min": 2 }
                ]
            })
            .to_string(),
        ),
    );
    assert_eq!(st_icase, 201, "{raw_icase}");

    let (st_imp, imp_json, raw_imp) = http_req(
        addr,
        "POST",
        "/api/evolution/propose",
        Some(
            &json!({
                "artifact_id": "default-improver",
                "kind": "improver",
                "candidate_content": "Fix failed checks, handle edge cases explicitly, and preserve passing tests.\nKeep diffs concise and verify workspace paths.",
                "note": "Meta-improver v2"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_imp, 200, "{raw_imp}");
    assert_eq!(imp_json["promoted"], true, "{raw_imp}");
    assert_eq!(imp_json["to_version"], 2);

    // 6. Verify scores & history endpoints and 0600 permissions
    let (st_sc, sc_json, _) = http_req(addr, "GET", "/api/evolution/scores", None);
    assert_eq!(st_sc, 200);
    assert!(sc_json["total_outcomes"].as_u64().unwrap() >= 2);
    assert!(sc_json["recent_log"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["event"] == "auto_rolled_back"));

    let (st_hist, hist_json, _) = http_req(addr, "GET", "/api/evolution/history/greeter-evo", None);
    assert_eq!(st_hist, 200);
    assert_eq!(hist_json["evolution"]["active_version"], 1);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in ["versions.json", "outcomes.jsonl", "replay_cases.json", "evolution_log.jsonl"] {
            let mode = std::fs::metadata(env.cfg.data_dir.join("evolution").join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600, "{name} should be 0600");
        }
    }
}
