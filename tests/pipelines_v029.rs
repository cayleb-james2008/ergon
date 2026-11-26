//! End-to-end tests for Ergon v0.29.0 Gap #1:
//! - Deterministic zero-LLM pipelines (skill -> export, without an API key)
//! - Human checkpoint pause -> simulated server restart / store reload ->
//!   `POST /api/pipelines/runs/:id/resume` injecting `{{steps.<id>.human_answer}}`
//! - Step failure + `POST /api/pipelines/runs/:id/retry` resuming from failed step
//! - Boot recovery turning stale `"running"` runs into `"interrupted"` / `"paused_checkpoint"`
//! - Starter playbooks running real `steps_v2` deterministic skills against workspace files
//! - `request_checkpoint` tool pausing an ad-hoc chat run into a resumable `PipelineRun`

use ergon_rs::{pipelines, records, serve, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
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
    fn new(name: &str, model_base_url: &str, api_key: Option<&str>) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-pipelines-v029-{}-{}-{name}",
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
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(30));
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        addr.port(),
        body.len()
    );
    stream.write_all(req.as_bytes()).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP header/body separator");
    let head = String::from_utf8_lossy(&raw[..sep]);
    let status = head
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
    let deadline = Instant::now() + Duration::from_secs(10);
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

#[test]
fn deterministic_zero_llm_pipeline_runs_skill_to_export_and_emits_sse() {
    let env = TestDir::new("zero-llm", "http://127.0.0.1:9/v1", None);
    env.write_ws(
        "raw-orders.csv",
        "Order ID,Customer,Amount\nORD-1,Alice,100.00\nORD-2,Bob,N/A\nORD-1,Alice,100.00\nORD-3,Cara,250.00\n",
    );
    let addr = spawn_server(env.cfg.clone());

    // 1. /api/status advertises pipelines capability
    let (st, status_json) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(status_json["capabilities"]["pipelines"], json!(true));

    // Subscribe to SSE /api/runs/stream in a background thread before launching pipeline
    let sse_events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sse_clone = sse_events.clone();
    std::thread::spawn(move || {
        let mut s = TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
        let req = format!(
            "GET /api/runs/stream HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: keep-alive\r\n\r\n",
            addr.port()
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut reader = BufReader::new(s);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }
            if let Some(ev) = line.strip_prefix("event: ") {
                let ev_name = ev.trim().to_string();
                let is_done = ev_name == "pipeline_finished";
                sse_clone.lock().unwrap().push(ev_name);
                if is_done {
                    break;
                }
            }
            line.clear();
        }
    });
    std::thread::sleep(Duration::from_millis(40));

    // 2. Start zero-LLM pipeline (skill -> export)
    let body = json!({
        "name": "Zero-LLM Order Pipeline",
        "inputs": { "file": "raw-orders.csv" },
        "steps_v2": [
            {
                "id": "clean",
                "label": "Clean raw orders CSV",
                "kind": "skill",
                "skill_id": "clean_dataset",
                "args": {
                    "input_file": "{{inputs.file}}",
                    "key_column": "order_id",
                    "output_title": "Cleaned Orders"
                },
                "when": "always"
            },
            {
                "id": "export_xlsx",
                "label": "Export cleaned workbook",
                "kind": "export",
                "export": {
                    "path": "{{steps.clean.files[0]}}",
                    "format": "xlsx"
                },
                "when": "on_previous_ok"
            }
        ]
    });
    let (st, res) = http_req(addr, "POST", "/api/pipelines/run", &body.to_string());
    assert_eq!(st, 202, "POST /api/pipelines/run: {res}");
    let run_id = res["run"]["run_id"].as_str().unwrap().to_string();

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "run failed: {done}");
    assert_eq!(done["step_states"][0]["status"], json!("ok"));
    assert_eq!(done["step_states"][1]["status"], json!("ok"));
    assert!(env.cfg.workspace_dir.join("Cleaned Orders.csv").exists());
    assert!(env.cfg.workspace_dir.join("Cleaned Orders.xlsx").exists());
    let xlsx_bytes = std::fs::read(env.cfg.workspace_dir.join("Cleaned Orders.xlsx")).unwrap();
    assert!(xlsx_bytes.starts_with(b"PK\x03\x04"), "valid xlsx zip header");

    // 3. List runs endpoint returns the completed run
    let (st, list) = http_req(addr, "GET", "/api/pipelines/runs?status=completed", "");
    assert_eq!(st, 200);
    assert!(list["runs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["run_id"] == run_id));

    // 4. Verify SSE events arrived
    std::thread::sleep(Duration::from_millis(80));
    let evs = sse_events.lock().unwrap().clone();
    assert!(
        evs.iter().any(|e| e == "pipeline_step_started"),
        "expected pipeline_step_started SSE event, got {evs:?}"
    );
    assert!(
        evs.iter().any(|e| e == "pipeline_step_done"),
        "expected pipeline_step_done SSE event, got {evs:?}"
    );
    assert!(
        evs.iter().any(|e| e == "pipeline_finished"),
        "expected pipeline_finished SSE event, got {evs:?}"
    );
}

#[test]
fn checkpoint_pause_survives_restart_and_resumes_with_human_answer() {
    let env = TestDir::new("checkpoint-resume", "http://127.0.0.1:9/v1", None);
    env.write_ws(
        "invoices.csv",
        "date,ref,vendor,category,amount,receipt\n2026-11-01,INV-1,Acme,Ops,400.00,verified\n2026-11-02,INV-2,Boreal,Ops,950.00,missing\n2026-11-03,INV-1,Acme,Ops,400.00,verified\n",
    );
    let addr1 = spawn_server(env.cfg.clone());

    // 1. Save routine with typed steps_v2
    let routine_req = json!({
        "name": "Month-End Close Pipeline",
        "prompt": "Reconcile invoices, pause on anomalies, and write resolution memo",
        "schedule": "manual",
        "steps_v2": [
            {
                "id": "reconcile",
                "label": "Reconcile ledger CSV",
                "kind": "skill",
                "skill_id": "reconcile_ledger",
                "args": {
                    "source_files": ["{{inputs.file}}"],
                    "output_title": "Q4 Audit"
                },
                "when": "always"
            },
            {
                "id": "human_gate",
                "label": "Review flagged anomalies",
                "kind": "checkpoint",
                "checkpoint": {
                    "prompt": "Confirm remediation choice before writing final memo",
                    "input_mode": "choice",
                    "choices": ["Exclude missing receipts", "Proceed with audit note"],
                    "inspect_files": ["{{steps.reconcile.files[0]}}"]
                },
                "when": "on_anomaly"
            },
            {
                "id": "memo",
                "label": "Write final resolution memo",
                "kind": "skill",
                "skill_id": "write_document",
                "args": {
                    "title": "Final Decision Memo",
                    "sections": [
                        "## Human Decision\nChoice: {{steps.human_gate.human_answer}}\nAction: {{steps.human_gate.human_action}}\nAudit Source: {{steps.reconcile.files[0]}}"
                    ]
                },
                "when": "on_previous_ok"
            }
        ]
    });
    let (st, rt_res) = http_req(addr1, "POST", "/api/routines", &routine_req.to_string());
    assert_eq!(st, 201, "POST /api/routines: {rt_res}");
    let routine_id = rt_res["routine"]["id"].as_str().unwrap().to_string();
    assert!(rt_res["routine"]["steps_v2"].is_array());
    assert!(rt_res["routine"]["steps"].is_array());

    // 2. Run the saved pipeline
    let (st, run_res) = http_req(
        addr1,
        "POST",
        "/api/pipelines/run",
        &json!({ "routine_id": routine_id, "inputs": { "file": "invoices.csv" } }).to_string(),
    );
    assert_eq!(st, 202, "POST /api/pipelines/run: {run_res}");
    let run_id = run_res["run"]["run_id"].as_str().unwrap().to_string();

    let paused = poll_run_until(addr1, &run_id, |r| {
        matches!(r["status"].as_str(), Some("paused_checkpoint" | "completed" | "failed"))
    });
    assert_eq!(paused["status"], json!("paused_checkpoint"), "expected pause: {paused}");
    assert_eq!(paused["current_step_index"], json!(1));
    assert_eq!(paused["step_states"][0]["status"], json!("ok"));
    assert_eq!(paused["step_states"][1]["status"], json!("waiting_human"));
    assert_eq!(
        paused["pending_checkpoint"]["inspect_files"],
        json!(["Q4 Audit.md"])
    );
    let cp_decision_id = paused["pending_checkpoint"]["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify linked decision exists in /api/decisions
    let (st, decs) = http_req(addr1, "GET", "/api/decisions", "");
    assert_eq!(st, 200);
    assert!(decs["decisions"].as_array().unwrap().iter().any(|d| {
        d["id"] == cp_decision_id
            && d["status"] == "pending"
            && d["kind"] == "checkpoint"
            && d["pipeline_run_id"] == run_id
    }));

    // 3. Simulate server restart: spawn a brand new server instance pointing at the same data_dir
    let addr2 = spawn_server(env.cfg.clone());
    let (st, reloaded) = http_req(addr2, "GET", &format!("/api/pipelines/runs/{run_id}"), "");
    assert_eq!(st, 200);
    assert_eq!(reloaded["run"]["status"], json!("paused_checkpoint"));

    // 4. Resume on server #2 with human choice
    let (st, resumed) = http_req(
        addr2,
        "POST",
        &format!("/api/pipelines/runs/{run_id}/resume"),
        &json!({
            "action": "answer",
            "choice": "Exclude missing receipts"
        })
        .to_string(),
    );
    assert_eq!(st, 202, "POST /resume: {resumed}");

    let completed = poll_run_until(addr2, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(completed["status"], json!("completed"), "resume failed: {completed}");
    assert_eq!(completed["step_states"][1]["status"], json!("ok"));
    assert_eq!(
        completed["step_states"][1]["human_answer"],
        json!("Exclude missing receipts")
    );
    assert_eq!(completed["step_states"][2]["status"], json!("ok"));

    let memo_text = std::fs::read_to_string(env.cfg.workspace_dir.join("Final Decision Memo.md")).unwrap();
    assert!(
        memo_text.contains("Choice: Exclude missing receipts"),
        "expected injected human_answer in memo: {memo_text}"
    );
    assert!(
        memo_text.contains("Audit Source: Q4 Audit.md"),
        "expected injected upstream file in memo: {memo_text}"
    );

    // Linked decision was settled atomically
    let (st, decs_after) = http_req(addr2, "GET", "/api/decisions", "");
    assert_eq!(st, 200);
    assert!(decs_after["decisions"].as_array().unwrap().iter().any(|d| {
        d["id"] == cp_decision_id && d["status"] == "approved"
    }));
}

#[test]
fn step_failure_and_retry_resumes_from_failed_step() {
    let env = TestDir::new("retry-step", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(env.cfg.clone());

    let body = json!({
        "name": "Retryable Pipeline",
        "steps_v2": [
            {
                "id": "prep",
                "label": "Write prep doc",
                "kind": "skill",
                "skill_id": "write_document",
                "args": { "title": "Prep", "sections": ["Initial prep content."] },
                "when": "always"
            },
            {
                "id": "clean",
                "label": "Clean missing CSV",
                "kind": "skill",
                "skill_id": "clean_dataset",
                "args": { "input_file": "missing-later.csv", "output_title": "Cleaned Later" },
                "when": "on_previous_ok"
            },
            {
                "id": "export_prep",
                "label": "Export prep to Word",
                "kind": "export",
                "export": { "path": "{{steps.prep.files[0]}}", "format": "docx" },
                "when": "on_previous_ok"
            }
        ]
    });
    let (st, res) = http_req(addr, "POST", "/api/pipelines/run", &body.to_string());
    assert_eq!(st, 202);
    let run_id = res["run"]["run_id"].as_str().unwrap().to_string();

    let failed = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("failed" | "completed"))
    });
    assert_eq!(failed["status"], json!("failed"), "expected failure: {failed}");
    assert_eq!(failed["step_states"][0]["status"], json!("ok"));
    assert_eq!(failed["step_states"][1]["status"], json!("failed"));
    assert_eq!(failed["step_states"][2]["status"], json!("pending"));

    // Fix the missing input file and retry from failed step 1
    env.write_ws("missing-later.csv", "id,val\n1,10\n2,20\n");
    let (st, retry_res) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{run_id}/retry"),
        "{}",
    );
    assert_eq!(st, 202, "POST /retry: {retry_res}");

    let completed = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(completed["status"], json!("completed"), "retry failed: {completed}");
    assert_eq!(completed["step_states"][0]["status"], json!("ok"));
    assert_eq!(completed["step_states"][1]["status"], json!("ok"));
    assert_eq!(completed["step_states"][2]["status"], json!("ok"));
    assert!(env.cfg.workspace_dir.join("Cleaned Later.csv").exists());
    assert!(env.cfg.workspace_dir.join("Prep.docx").exists());
}

#[test]
fn boot_recovery_converts_stale_running_runs_to_interrupted_and_paused() {
    let env = TestDir::new("boot-recover", "http://127.0.0.1:9/v1", None);
    let now_ms = records::now_millis();
    let stale_running = pipelines::PipelineRun {
        run_id: "prun-stale-running".to_string(),
        routine_id: None,
        routine_name: "Interrupted Job".to_string(),
        thread_id: "t-stale-1".to_string(),
        status: "running".to_string(),
        current_step_index: 0,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs: Default::default(),
        steps: vec![pipelines::PipelineStepSpec {
            id: "doc".to_string(),
            label: "Write doc".to_string(),
            kind: "skill".to_string(),
            skill_id: Some("write_document".to_string()),
            args: json!({ "title": "Recovered", "sections": ["Hello"] }),
            prompt: None,
            checkpoint: None,
            export: None,
            when: "always".to_string(),
        }],
        step_states: vec![pipelines::PipelineStepState {
            step_id: "doc".to_string(),
            label: "Write doc".to_string(),
            kind: "skill".to_string(),
            status: "running".to_string(),
            started_ms: now_ms,
            finished_ms: 0,
            output: String::new(),
            files: Vec::new(),
            metrics: json!({}),
            checkpoint_id: None,
            human_answer: None,
            human_action: None,
            error: None,
        }],
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
    };
    pipelines::save_run(&env.cfg.data_dir, &stale_running).unwrap();

    // Boot server -> recovers stale "running" into "interrupted"
    let addr = spawn_server(env.cfg.clone());
    let (st, got) = http_req(addr, "GET", "/api/pipelines/runs/prun-stale-running", "");
    assert_eq!(st, 200);
    assert_eq!(got["run"]["status"], json!("interrupted"));
    assert_eq!(got["run"]["step_states"][0]["status"], json!("failed"));

    // Retry the interrupted run
    let (st, ret) = http_req(addr, "POST", "/api/pipelines/runs/prun-stale-running/retry", "{}");
    assert_eq!(st, 202, "retry interrupted failed: {ret}");
    let done = poll_run_until(addr, "prun-stale-running", |r| r["status"] == "completed");
    assert_eq!(done["status"], json!("completed"));
    assert!(env.cfg.workspace_dir.join("Recovered.md").exists());
}

#[test]
fn starter_playbook_launches_real_skill_and_respects_existing_workspace_inputs() {
    let env = TestDir::new("starter-real", "http://127.0.0.1:9/v1", None);
    env.write_ws(
        "standup-notes.md",
        "Attendees: Dana, Sam\nDana: We decided to ship pipeline v2 on Friday.\nAction: Sam will archive old scripts.\n",
    );
    let addr = spawn_server(env.cfg.clone());

    let (st, res) = http_req(
        addr,
        "POST",
        "/api/playbooks/starters/launch",
        &json!({ "id": "starter-weekly-client-digest", "run_now": true }).to_string(),
    );
    assert_eq!(st, 201, "launch starter: {res}");
    assert_eq!(res["ok"], json!(true));
    assert!(
        res["seeded_inputs"].as_array().unwrap().is_empty(),
        "should NOT seed fixture when workspace already has standup-notes.md: {res}"
    );
    assert!(
        !env.cfg.workspace_dir.join("meeting-notes.md").exists(),
        "must not write seed fixture over existing workspace notes"
    );
    let digest = std::fs::read_to_string(env.cfg.workspace_dir.join("Weekly Client Digest.md")).unwrap();
    assert!(
        digest.contains("Dana") && digest.contains("Sam"),
        "expected real summarize_meeting output from standup-notes.md, got: {digest}"
    );
    assert!(env.cfg.workspace_dir.join("Weekly Client Digest.docx").exists());
    assert!(
        !res["summary"].as_str().unwrap_or("").contains("Rune"),
        "UI-facing summary must never mention Rune"
    );
}

#[test]
fn request_checkpoint_tool_parks_adhoc_chat_and_resumes_with_human_answer() {
    // Mock OpenAI-compatible server returning:
    // 1) write_file draft.md
    // 2) request_checkpoint
    // 3) (after resume) done
    let mock_replies = Arc::new(Mutex::new(vec![
        json!({"tool": "write_file", "args": {"path": "draft.md", "content": "Initial draft"}}).to_string(),
        json!({
            "tool": "request_checkpoint",
            "args": {
                "prompt": "What footer should be added to draft.md?",
                "input_mode": "text",
                "inspect_files": ["draft.md"]
            }
        }).to_string(),
        json!({"done": true, "summary": "Applied your checkpoint instruction."}).to_string(),
    ]));
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    let seen_prompts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen_prompts.clone();
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
                if let Some(v) = tr.strip_prefix("Content-Length:").or_else(|| tr.strip_prefix("content-length:")) {
                    content_len = v.trim().parse().unwrap_or(0);
                }
                line.clear();
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);
            let body_str = String::from_utf8_lossy(&body).to_string();
            seen_clone.lock().unwrap().push(body_str);
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
        "adhoc-cp",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let addr = spawn_server(env.cfg.clone());

    let (st, chat_res) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({ "message": "Draft draft.md and ask me before finalizing." }).to_string(),
    );
    assert_eq!(st, 200, "POST /api/chat: {chat_res}");
    let prun_id = chat_res["paused_pipeline_run_id"]
        .as_str()
        .expect("paused_pipeline_run_id on chat response")
        .to_string();
    assert_eq!(
        chat_res["pending_checkpoint"]["prompt"],
        json!("What footer should be added to draft.md?")
    );

    // Resume the ad-hoc checkpoint with a text answer
    let (st, resume_res) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{prun_id}/resume"),
        &json!({ "action": "answer", "answer": "Include Confidential Q4 footer" }).to_string(),
    );
    assert_eq!(st, 202, "POST /resume: {resume_res}");

    let done = poll_run_until(addr, &prun_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "resume chat failed: {done}");

    let prompts = seen_prompts.lock().unwrap();
    assert!(
        prompts.last().is_some_and(|p| p.contains("Include Confidential Q4 footer")),
        "expected downstream resumed agent call to include human answer; prompts: {prompts:?}"
    );
}
