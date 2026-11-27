//! End-to-end integration tests for Ergon v0.35.0 Gap #3:
//! - Parallel `depends_on` DAG fan-out/join + HTTP 400 cycle rejection + `POST /api/pipelines/validate`
//! - `for_each` batch execution (`child_items`, `step_id[i]`, `{{steps.<id>.items}}`) + `>50` cap rejection
//! - `deliver` connector step (immediate in `fast_internal` & checkpoint-held in `strict_client` until approved via `/resume` or `/api/decisions/:id/approve`)
//! - Ad-hoc `agent_snapshot` context preservation across `request_checkpoint` pause/resume
//! - `run_loop` automatic compaction under tight `context_budget_chars` & safe continuation past 24 steps
//! - OpenAI SSE `token_delta` streaming + clean JSON fallback

use ergon_rs::{
    library, records, run_agent_with_options, serve, sse, AgentLoopOptions, Config, LiveRuns,
    Store, ToolCtx,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(name: &str, model_base_url: &str, api_key: Option<&str>) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-dag-v035-{}-{}-{name}",
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
            model_base_url: model_base_url.to_string(),
            model_id: "test-model".to_string(),
            api_key: api_key.map(str::to_string),
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }

    fn write_ws(&self, rel: &str, content: &str) {
        let full = self.cfg.workspace_dir.join(rel);
        if let Some(p) = full.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn spawn_server(cfg: Config) -> SocketAddr {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(35));
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    http_req_with_headers(addr, method, path, &[], body)
}

fn http_req_with_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut has_host = false;
    let mut head = format!("{method} {path} HTTP/1.1\r\n");
    for (k, v) in extra_headers {
        if k.eq_ignore_ascii_case("host") {
            has_host = true;
        }
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if !has_host {
        head.push_str(&format!("Host: 127.0.0.1:{}\r\n", addr.port()));
    }
    if !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP header/body separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_bytes = &raw[sep + 4..];
    let json_val = serde_json::from_slice(body_bytes).unwrap_or(Value::Null);
    (status, json_val)
}

fn poll_run_until(
    addr: SocketAddr,
    run_id: &str,
    predicate: impl Fn(&Value) -> bool,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let (st, v) = http_req(addr, "GET", &format!("/api/pipelines/runs/{run_id}"), "");
        assert_eq!(st, 200, "GET run failed: {v}");
        let run = v["run"].clone();
        if predicate(&run) {
            return run;
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for pipeline run {run_id}; last state: {run}");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn spawn_webhook_receiver() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let cap_clone = captured.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut content_len = 0usize;
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                let tr = line.trim();
                if tr.is_empty() {
                    break;
                }
                if let Some(v) = tr
                    .strip_prefix("Content-Length:")
                    .or_else(|| tr.strip_prefix("content-length:"))
                {
                    content_len = v.trim().parse().unwrap_or(0);
                }
                line.clear();
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);
            cap_clone
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).to_string());
            let resp_body = r#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                resp_body.len()
            );
            let mut s = reader.into_inner();
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (addr, captured)
}

#[test]
fn dag_parallel_fanout_join_validate_and_cycle_rejection() {
    let env = TestDir::new("dag-fanout", "http://127.0.0.1:9/v1", None);
    env.write_ws(
        "meeting-a.md",
        "Attendees: Alice\nDecision: Launch Alpha\nAction: Alice prepares brief\n",
    );
    env.write_ws(
        "meeting-b.md",
        "Attendees: Bob\nDecision: Launch Beta\nAction: Bob prepares sheet\n",
    );
    let addr = spawn_server(env.cfg.clone());

    // 1. Validate cycle -> HTTP 400
    let cycle_steps = json!([
        {
            "id": "a",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "A", "sections": ["x"] },
            "depends_on": ["b"]
        },
        {
            "id": "b",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "B", "sections": ["y"] },
            "depends_on": ["a"]
        }
    ]);
    let (st_cyc_val, res_cyc_val) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({ "steps_v2": cycle_steps }).to_string(),
    );
    assert_eq!(st_cyc_val, 400, "expected 400 on cycle validate: {res_cyc_val}");
    assert_eq!(res_cyc_val["valid"], json!(false));

    // Origin guard check on /api/pipelines/validate
    let (st_orig, _) = http_req_with_headers(
        addr,
        "POST",
        "/api/pipelines/validate",
        &[("Origin", "https://evil.example")],
        &json!({ "steps_v2": cycle_steps }).to_string(),
    );
    assert_eq!(st_orig, 403, "cross-origin validate must return 403");

    // 2. Launch cycle -> HTTP 400
    let (st_cyc_run, res_cyc_run) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Cycle", "steps_v2": cycle_steps }).to_string(),
    );
    assert_eq!(st_cyc_run, 400, "expected 400 on cycle run: {res_cyc_run}");

    // 3. Valid diamond DAG + expr: & on_all_ok conditions
    let dag_steps = json!([
        {
            "id": "branch_a",
            "label": "Draft A",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "OutA", "sections": ["Alpha"] },
            "depends_on": []
        },
        {
            "id": "branch_b",
            "label": "Draft B",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "OutB", "sections": ["Beta"] },
            "depends_on": []
        },
        {
            "id": "skipped_branch",
            "label": "Only if failed",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "ShouldSkip", "sections": ["nope"] },
            "depends_on": ["branch_a"],
            "when": "expr:steps.branch_a.status == failed"
        },
        {
            "id": "join",
            "label": "Join Report",
            "kind": "skill",
            "skill_id": "write_document",
            "args": {
                "title": "Combined DAG",
                "sections": [
                    "A={{steps.branch_a.files[0]}}",
                    "B={{steps.branch_b.files[0]}}"
                ]
            },
            "depends_on": ["branch_a", "branch_b"],
            "when": "on_all_ok"
        },
        {
            "id": "after_expr_ok",
            "label": "Expr OK Step",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "ExprPassed", "sections": ["yes"] },
            "depends_on": ["join"],
            "when": "expr:steps.join.status == ok"
        }
    ]);

    let (st_val, res_val) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({ "steps_v2": dag_steps }).to_string(),
    );
    assert_eq!(st_val, 200, "valid DAG failed validate: {res_val}");
    assert_eq!(res_val["ok"], json!(true));
    assert_eq!(res_val["valid"], json!(true));
    let topo = res_val["topological_order"].as_array().unwrap();
    assert_eq!(topo.len(), 5);

    let (st_run, res_run) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Diamond DAG", "steps_v2": dag_steps }).to_string(),
    );
    assert_eq!(st_run, 202, "run failed: {res_run}");
    let run_id = res_run["run"]["run_id"].as_str().unwrap().to_string();

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "DAG failed: {done}");
    assert_eq!(done["step_states"][0]["status"], json!("ok"));
    assert_eq!(done["step_states"][1]["status"], json!("ok"));
    assert_eq!(done["step_states"][2]["status"], json!("skipped"));
    assert_eq!(done["step_states"][3]["status"], json!("ok"));
    assert_eq!(done["step_states"][4]["status"], json!("ok"));

    let combined = std::fs::read_to_string(env.cfg.workspace_dir.join("Combined DAG.md")).unwrap();
    assert!(combined.contains("A=OutA.md"), "missing A file: {combined}");
    assert!(combined.contains("B=OutB.md"), "missing B file: {combined}");
    assert!(!env.cfg.workspace_dir.join("ShouldSkip.md").exists());
    assert!(env.cfg.workspace_dir.join("ExprPassed.md").exists());
}

#[test]
fn for_each_batch_execution_and_cap_rejection() {
    let env = TestDir::new("for-each", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(env.cfg.clone());

    // 1. >50 cap rejection
    let over_cap: Vec<String> = (0..51).map(|i| format!("Doc_{i}")).collect();
    let over_steps = json!([
        {
            "id": "batch_over",
            "kind": "skill",
            "skill_id": "write_document",
            "for_each": "{{inputs.docs}}",
            "args": { "title": "{{item}}", "sections": ["body"] }
        }
    ]);
    let (st_over, res_over) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Over Cap",
            "inputs": { "docs": json!(over_cap) },
            "steps_v2": over_steps
        })
        .to_string(),
    );
    assert_eq!(st_over, 400, "expected 400 for >50 for_each: {res_over}");

    // 2. Valid 4-item batch execution + downstream {{steps.batch.items}} & {{steps.batch.files}}
    let batch_steps = json!([
        {
            "id": "batch",
            "label": "Generate Batch Docs",
            "kind": "skill",
            "skill_id": "write_document",
            "for_each": "{{inputs.docs}}",
            "args": { "title": "Batch_{{item}}", "sections": ["Index={{item.index}}"] }
        },
        {
            "id": "summary",
            "label": "Rollup Batch",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["batch"],
            "args": {
                "title": "Batch_Summary",
                "sections": [
                    "First={{steps.batch.items[0].step_id}}",
                    "Files={{steps.batch.files}}"
                ]
            }
        }
    ]);
    let (st_run, res_run) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Batch Run",
            "inputs": { "docs": json!(["One", "Two", "Three", "Four"]) },
            "steps_v2": batch_steps
        })
        .to_string(),
    );
    assert_eq!(st_run, 202, "batch run failed: {res_run}");
    let run_id = res_run["run"]["run_id"].as_str().unwrap().to_string();

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "batch failed: {done}");
    let children = done["step_states"][0]["child_items"].as_array().unwrap();
    assert_eq!(children.len(), 4);
    assert_eq!(children[0]["step_id"], json!("batch[0]"));
    assert_eq!(children[3]["step_id"], json!("batch[3]"));
    assert_eq!(done["child_items"]["batch"].as_array().unwrap().len(), 4);

    let summary = std::fs::read_to_string(env.cfg.workspace_dir.join("Batch_Summary.md")).unwrap();
    assert!(summary.contains("First=batch[0]"), "summary: {summary}");
    assert!(summary.contains("Batch_Four.md"), "summary: {summary}");
}

#[test]
fn deliver_step_immediate_fast_and_strict_client_checkpoint_hold() {
    std::env::set_var("ERGON_FETCH_ALLOW_PRIVATE", "1");
    let env = TestDir::new("deliver", "http://127.0.0.1:9/v1", None);
    env.write_ws("brief.md", "# Client Brief\nApproved metrics.\n");
    let addr = spawn_server(env.cfg.clone());
    let (hook_addr, captured) = spawn_webhook_receiver();

    // Create outbound connector (require_approval: false)
    let (st_c, conn_res) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Client Webhook",
            "kind": "http_webhook",
            "url": format!("http://127.0.0.1:{}/hook", hook_addr.port()),
            "method": "POST",
            "require_approval": false
        })
        .to_string(),
    );
    assert_eq!(st_c, 201, "create connector: {conn_res}");
    let conn_id = conn_res["connector"]["id"].as_str().unwrap().to_string();

    // Validate deliver step passes
    let steps = json!([
        {
            "id": "send",
            "label": "Send Brief",
            "kind": "deliver",
            "deliver": {
                "connector_id": conn_id,
                "file": "brief.md"
            }
        },
        {
            "id": "after",
            "label": "Post-Delivery Note",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["send"],
            "when": "on_all_ok",
            "args": { "title": "Delivery_Confirmed", "sections": ["{{steps.send.output}}"] }
        }
    ]);
    let (st_v, res_v) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({ "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_v, 200, "validate deliver: {res_v}");

    // 1. Fast internal mode -> immediate delivery
    let (st_r1, res_r1) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Fast Deliver", "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_r1, 202, "run fast deliver: {res_r1}");
    let run_id_1 = res_r1["run"]["run_id"].as_str().unwrap().to_string();
    let done1 = poll_run_until(addr, &run_id_1, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done1["status"], json!("completed"), "fast deliver failed: {done1}");
    assert_eq!(captured.lock().unwrap().len(), 1);
    assert!(captured.lock().unwrap()[0].contains("Approved metrics."));

    // 2. Switch to strict_client -> deliver parks in paused_checkpoint until approved!
    let (st_pol, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    assert_eq!(st_pol, 200);

    let (st_r2, res_r2) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Strict Deliver", "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_r2, 202);
    let run_id_2 = res_r2["run"]["run_id"].as_str().unwrap().to_string();
    let paused = poll_run_until(addr, &run_id_2, |r| {
        matches!(
            r["status"].as_str(),
            Some("paused_checkpoint" | "completed" | "failed")
        )
    });
    assert_eq!(
        paused["status"],
        json!("paused_checkpoint"),
        "strict_client deliver must park in paused_checkpoint: {paused}"
    );
    assert_eq!(
        captured.lock().unwrap().len(),
        1,
        "must NOT leak unapproved delivery while parked"
    );

    // Approve via POST /api/pipelines/runs/:id/resume
    let (st_res, _) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{run_id_2}/resume"),
        r#"{"action":"approve"}"#,
    );
    assert_eq!(st_res, 202);
    let done2 = poll_run_until(addr, &run_id_2, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done2["status"], json!("completed"), "strict resume failed: {done2}");
    assert_eq!(
        captured.lock().unwrap().len(),
        2,
        "approved checkpoint must execute queued delivery exactly once"
    );
    assert!(env.cfg.workspace_dir.join("Delivery_Confirmed.md").exists());

    // 3. Also verify settling via POST /api/decisions/:id/approve resumes and delivers once!
    let (st_r3, res_r3) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Strict Deliver Via Decision", "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_r3, 202);
    let run_id_3 = res_r3["run"]["run_id"].as_str().unwrap().to_string();
    let paused3 = poll_run_until(addr, &run_id_3, |r| {
        matches!(
            r["status"].as_str(),
            Some("paused_checkpoint" | "completed" | "failed")
        )
    });
    assert_eq!(paused3["status"], json!("paused_checkpoint"));
    let dec_id = paused3["pending_checkpoint"]["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(captured.lock().unwrap().len(), 2);

    let (st_dec, res_dec) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec_id}/approve"),
        "",
    );
    assert_eq!(st_dec, 200, "approve decision failed: {res_dec}");
    let done3 = poll_run_until(addr, &run_id_3, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done3["status"], json!("completed"), "decision settle resume failed: {done3}");
    assert_eq!(
        captured.lock().unwrap().len(),
        3,
        "decision approval must deliver exactly once and resume downstream step"
    );
    let _ = records::load_decisions(&env.cfg.data_dir);
}

#[test]
fn adhoc_agent_snapshot_preserves_context_across_pause_resume() {
    let mock_replies = Arc::new(Mutex::new(vec![
        json!({"tool": "write_file", "args": {"path": "draft.md", "content": "Draft v1 body"}}).to_string(),
        json!({
            "tool": "request_checkpoint",
            "args": {
                "prompt": "Which footer should I add?",
                "input_mode": "text",
                "inspect_files": ["draft.md"]
            }
        })
        .to_string(),
        json!({"done": true, "summary": "Finalized draft.md with your footer."}).to_string(),
    ]));
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    let seen_bodies: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen_bodies.clone();
    let replies_clone = mock_replies.clone();
    std::thread::spawn(move || {
        for stream in mock_listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut content_len = 0usize;
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                let tr = line.trim();
                if tr.is_empty() {
                    break;
                }
                if let Some(v) = tr
                    .strip_prefix("Content-Length:")
                    .or_else(|| tr.strip_prefix("content-length:"))
                {
                    content_len = v.trim().parse().unwrap_or(0);
                }
                line.clear();
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);
            seen_clone
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).to_string());
            let next_reply = {
                let mut q = replies_clone.lock().unwrap();
                if q.is_empty() {
                    json!({"done": true, "summary": "Done."}).to_string()
                } else {
                    q.remove(0)
                }
            };
            let payload = json!({
                "choices": [{
                    "message": { "role": "assistant", "content": next_reply }
                }]
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let mut s = reader.into_inner();
            let _ = s.write_all(resp.as_bytes());
        }
    });

    let env = TestDir::new(
        "adhoc-snapshot",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let addr = spawn_server(env.cfg.clone());

    let (st_chat, chat_res) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({ "message": "Draft draft.md and ask before finalizing." }).to_string(),
    );
    assert_eq!(st_chat, 200, "POST /api/chat: {chat_res}");
    let prun_id = chat_res["paused_pipeline_run_id"]
        .as_str()
        .expect("paused_pipeline_run_id")
        .to_string();

    let (st_get, run_get) = http_req(addr, "GET", &format!("/api/pipelines/runs/{prun_id}"), "");
    assert_eq!(st_get, 200);
    let snap = &run_get["run"]["agent_snapshot"];
    assert!(snap.is_object(), "agent_snapshot missing: {run_get}");
    assert!(
        snap["written"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "draft.md"),
        "snapshot written must preserve draft.md: {snap}"
    );
    assert!(snap["budget_used"].as_u64().unwrap_or(0) >= 2);
    assert!(
        !snap["messages"].as_array().unwrap().is_empty(),
        "snapshot messages must be non-empty"
    );

    let (st_res, _) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{prun_id}/resume"),
        &json!({ "action": "answer", "answer": "Add Q4 Confidential Footer" }).to_string(),
    );
    assert_eq!(st_res, 202);
    let done = poll_run_until(addr, &prun_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "resume failed: {done}");

    let bodies = seen_bodies.lock().unwrap();
    let resumed_req = bodies.last().expect("resumed model call");
    assert!(
        resumed_req.contains("write_file") && resumed_req.contains("draft.md"),
        "resumed model request must carry full pre-checkpoint tool conversation, got: {resumed_req}"
    );
    assert!(
        resumed_req.contains("Add Q4 Confidential Footer"),
        "resumed model request must carry human answer"
    );
}

#[test]
fn run_loop_compacts_under_tight_budget_and_continues_past_24_steps() {
    let step_counter = Arc::new(AtomicUsize::new(0));
    let observed_max_chars = Arc::new(AtomicUsize::new(0));
    let saw_digest = Arc::new(Mutex::new(false));
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();

    let sc = step_counter.clone();
    let omc = observed_max_chars.clone();
    let sd = saw_digest.clone();
    std::thread::spawn(move || {
        for stream in mock_listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut content_len = 0usize;
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                let tr = line.trim();
                if tr.is_empty() {
                    break;
                }
                if let Some(v) = tr
                    .strip_prefix("Content-Length:")
                    .or_else(|| tr.strip_prefix("content-length:"))
                {
                    content_len = v.trim().parse().unwrap_or(0);
                }
                line.clear();
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);
            let body_str = String::from_utf8_lossy(&body).to_string();
            let v: Value = serde_json::from_str(&body_str).unwrap_or(Value::Null);
            let msgs = v["messages"].as_array().cloned().unwrap_or_default();
            // Check non-system message chars stayed bounded and system message kept at index 0
            assert_eq!(msgs[0]["role"], json!("system"));
            let non_sys_chars: usize = msgs.iter().skip(1).map(|m| m.to_string().len()).sum();
            omc.fetch_max(non_sys_chars, Ordering::SeqCst);
            if msgs.iter().any(|m| {
                m["content"]
                    .as_str()
                    .is_some_and(|c| c.contains("[CONTEXT_DIGEST") && c.contains("note.md"))
            }) {
                *sd.lock().unwrap() = true;
            }

            let step = sc.fetch_add(1, Ordering::SeqCst) + 1;
            let reply = if step == 1 {
                json!({
                    "tool": "write_file",
                    "args": { "path": "note.md", "content": "X".repeat(300) }
                })
                .to_string()
            } else if step <= 26 {
                json!({
                    "tool": "read_file",
                    "args": { "path": "note.md" }
                })
                .to_string()
            } else {
                json!({
                    "done": true,
                    "summary": "Finished 26 steps after compaction."
                })
                .to_string()
            };
            let payload = json!({
                "choices": [{
                    "message": { "role": "assistant", "content": reply }
                }]
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let mut s = reader.into_inner();
            let _ = s.write_all(resp.as_bytes());
        }
    });

    let env = TestDir::new(
        "compact-26",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let lib = Arc::new(Mutex::new(library::Library::load(&env.cfg.data_dir)));
    let ctx = ToolCtx {
        library: lib,
        root_data_dir: env.cfg.data_dir.clone(),
        person_id: "default".to_string(),
    };
    let store = Arc::new(Mutex::new(Store::load(&env.cfg.data_dir)));
    let tid = store.lock().unwrap().create_thread();
    let live_runs: LiveRuns = Arc::new(Mutex::new(HashMap::new()));
    let bus = sse::RunEventBus::new();

    let out = run_agent_with_options(
        &env.cfg,
        &ctx,
        &store,
        &tid,
        "Run 26 compacted steps using note.md",
        &[],
        Some((&live_runs, &bus, "default")),
        AgentLoopOptions {
            context_budget_chars: 1400,
            max_steps: 30,
            snapshot: None,
            ..AgentLoopOptions::default()
        },
    )
    .expect("compacted 26-step run should succeed");

    assert_eq!(out.steps.len(), 26);
    assert!(out.written_files.contains(&"note.md".to_string()));
    assert!(*saw_digest.lock().unwrap(), "expected [CONTEXT_DIGEST] preserving WriteLedger note.md");
}

#[test]
fn sse_token_delta_streaming_and_json_fallback() {
    let call_num = Arc::new(AtomicUsize::new(0));
    let cn = call_num.clone();
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();

    std::thread::spawn(move || {
        for stream in mock_listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut content_len = 0usize;
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                let tr = line.trim();
                if tr.is_empty() {
                    break;
                }
                if let Some(v) = tr
                    .strip_prefix("Content-Length:")
                    .or_else(|| tr.strip_prefix("content-length:"))
                {
                    content_len = v.trim().parse().unwrap_or(0);
                }
                line.clear();
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);
            let n = cn.fetch_add(1, Ordering::SeqCst);
            let mut s = reader.into_inner();
            if n == 0 {
                // First call returns true OpenAI SSE stream chunks!
                let sse_body = concat!(
                    "data: {\"id\":\"1\",\"model\":\"stream-m\",\"choices\":[{\"delta\":{\"content\":\"{\\\"done\\\":true,\"}}]}\n\n",
                    "data: {\"id\":\"1\",\"model\":\"stream-m\",\"choices\":[{\"delta\":{\"content\":\"\\\"summary\\\":\\\"Streamed \"}}]}\n\n",
                    "data: {\"id\":\"1\",\"model\":\"stream-m\",\"choices\":[{\"delta\":{\"content\":\"ok\\\"}\"}}]}\n\n",
                    "data: [DONE]\n\n"
                );
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sse_body}",
                    sse_body.len()
                );
                let _ = s.write_all(resp.as_bytes());
            } else {
                // Second call falls back cleanly to standard application/json!
                let payload = json!({
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": json!({"done": true, "summary": "JSON fallback ok"}).to_string()
                        }
                    }]
                })
                .to_string();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = s.write_all(resp.as_bytes());
            }
        }
    });

    let env = TestDir::new(
        "sse-stream",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let lib = Arc::new(Mutex::new(library::Library::load(&env.cfg.data_dir)));
    let ctx = ToolCtx {
        library: lib,
        root_data_dir: env.cfg.data_dir.clone(),
        person_id: "default".to_string(),
    };
    let store = Arc::new(Mutex::new(Store::load(&env.cfg.data_dir)));
    let tid = store.lock().unwrap().create_thread();
    let live_runs: LiveRuns = Arc::new(Mutex::new(HashMap::new()));
    let bus = sse::RunEventBus::new();

    // 1. Streaming SSE response emits token_delta events and reconstructs summary
    let out1 = run_agent_with_options(
        &env.cfg,
        &ctx,
        &store,
        &tid,
        "Say hello via SSE",
        &[],
        Some((&live_runs, &bus, "default")),
        AgentLoopOptions::default(),
    )
    .expect("SSE streaming run should succeed");
    assert_eq!(out1.summary, "Streamed ok");

    let events = bus.wait_since(0, Duration::from_millis(50));
    let deltas: Vec<String> = events
        .iter()
        .filter(|e| e.event == "token_delta")
        .filter_map(|e| e.data["delta"].as_str().map(str::to_string))
        .collect();
    assert_eq!(deltas.len(), 3, "expected 3 token_delta events: {events:?}");
    assert_eq!(deltas.concat(), "{\"done\":true,\"summary\":\"Streamed ok\"}");

    // 2. JSON fallback still works with SSE bus attached
    let out2 = run_agent_with_options(
        &env.cfg,
        &ctx,
        &store,
        &tid,
        "Say hello via JSON fallback",
        &[],
        Some((&live_runs, &bus, "default")),
        AgentLoopOptions::default(),
    )
    .expect("JSON fallback run should succeed");
    assert_eq!(out2.summary, "JSON fallback ok");
}
