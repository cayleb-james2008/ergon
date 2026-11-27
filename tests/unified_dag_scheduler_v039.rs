//! Integration tests for Ergon v0.39.0 Gap #3:
//! - Concurrent `run_id`-keyed `LiveRuns` isolation, SSE `run_id` deltas, and per-run cancellation
//! - Parallel DAG branch checkpoints (`pending_checkpoints`) resuming individually & `step_snapshots`
//! - Offline zero-LLM `steps_v2` scheduled routine execution + unified routine DAG routing
//! - Per-step `retry_max` / `retry_backoff_ms`, `timeout_secs`, `on_error` (`continue` / `goto:<id>`), and `subpipeline`
//! - 5-field Cron + timezone offset parsing, next-run calculation, and routine CRUD

use ergon_rs::{pipelines, records, scheduler, serve, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
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
            "ergon-v039-{}-{}-{name}",
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

fn spawn_server_with_scheduler(cfg: Config, enable_scheduler: bool) -> SocketAddr {
    if enable_scheduler {
        std::env::set_var("ERGON_DISABLE_SCHEDULER", "0");
        std::env::set_var("ERGON_SCHEDULER_TICK_MS", "50");
    } else {
        std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(35));
    addr
}

fn spawn_server(cfg: Config) -> SocketAddr {
    spawn_server_with_scheduler(cfg, false)
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n", addr.port());
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
        assert!(
            Instant::now() < deadline,
            "timed out waiting on run {run_id}; last state: {run}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn concurrent_live_runs_isolation_sse_run_id_and_per_run_cancellation() {
    // Mock LLM server:
    // - On call 1 for each task ("TASK_ALPHA" / "TASK_BETA"): executes write_file tool.
    // - On call 2 for each task: waits on a gate channel until released so both runs are
    //   guaranteed to be simultaneously active in LiveRuns!
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();

    let (alpha_gate_tx, alpha_gate_rx) = mpsc::channel::<()>();
    let (beta_gate_tx, beta_gate_rx) = mpsc::channel::<()>();
    let alpha_rx = Arc::new(Mutex::new(alpha_gate_rx));
    let beta_rx = Arc::new(Mutex::new(beta_gate_rx));
    let alpha_calls = Arc::new(AtomicUsize::new(0));
    let beta_calls = Arc::new(AtomicUsize::new(0));

    let a_rx_c = alpha_rx.clone();
    let b_rx_c = beta_rx.clone();
    let a_cnt = alpha_calls.clone();
    let b_cnt = beta_calls.clone();

    std::thread::spawn(move || {
        for stream in mock_listener.incoming().flatten() {
            let a_rx_w = a_rx_c.clone();
            let b_rx_w = b_rx_c.clone();
            let a_cnt_w = a_cnt.clone();
            let b_cnt_w = b_cnt.clone();
            std::thread::spawn(move || {
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

                let reply = if body_str.contains("TASK_ALPHA") {
                    let n = a_cnt_w.fetch_add(1, Ordering::SeqCst);
                    if n == 0 {
                        json!({
                            "tool": "write_file",
                            "args": { "path": "alpha.md", "content": "Alpha step 1" }
                        })
                        .to_string()
                    } else {
                        let _ = a_rx_w.lock().unwrap().recv_timeout(Duration::from_secs(5));
                        json!({ "done": true, "summary": "Alpha done." }).to_string()
                    }
                } else {
                    let n = b_cnt_w.fetch_add(1, Ordering::SeqCst);
                    if n == 0 {
                        json!({
                            "tool": "write_file",
                            "args": { "path": "beta.md", "content": "Beta step 1" }
                        })
                        .to_string()
                    } else {
                        let _ = b_rx_w.lock().unwrap().recv_timeout(Duration::from_secs(5));
                        json!({ "done": true, "summary": "Beta done." }).to_string()
                    }
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
            });
        }
    });

    let env = TestDir::new(
        "concurrent-live",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let addr = spawn_server(env.cfg.clone());

    // Subscribe to SSE stream to confirm run_id is included in deltas.
    let sse_events = Arc::new(Mutex::new(String::new()));
    let sse_events_c = sse_events.clone();
    let sse_stop = Arc::new(AtomicBool::new(false));
    let sse_stop_c = sse_stop.clone();
    let sse_thread = std::thread::spawn(move || {
        let mut s = TcpStream::connect(addr).unwrap();
        let _ = s.set_read_timeout(Some(Duration::from_millis(150)));
        let req = format!(
            "GET /api/runs/stream HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAccept: text/event-stream\r\n\r\n",
            addr.port()
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut buf = [0u8; 4096];
        let deadline = Instant::now() + Duration::from_secs(8);
        while !sse_stop_c.load(Ordering::SeqCst) && Instant::now() < deadline {
            match s.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => sse_events_c
                    .lock()
                    .unwrap()
                    .push_str(&String::from_utf8_lossy(&buf[..n])),
                Err(_) => continue,
            }
        }
    });
    std::thread::sleep(Duration::from_millis(60));

    let h_alpha = std::thread::spawn(move || {
        http_req(
            addr,
            "POST",
            "/api/chat",
            &json!({ "thread_id": "", "message": "TASK_ALPHA" }).to_string(),
        )
    });
    let h_beta = std::thread::spawn(move || {
        http_req(
            addr,
            "POST",
            "/api/chat",
            &json!({ "thread_id": "", "message": "TASK_BETA" }).to_string(),
        )
    });

    // Wait until GET /api/runs/status reports BOTH concurrent runs active.
    let deadline = Instant::now() + Duration::from_secs(5);
    let status_snapshot = loop {
        let (st, v) = http_req(addr, "GET", "/api/runs/status", "");
        assert_eq!(st, 200);
        if v["runs"].as_array().is_some_and(|a| a.len() == 2) {
            break v;
        }
        assert!(
            Instant::now() < deadline,
            "expected 2 concurrent active runs, last status: {v}"
        );
        std::thread::sleep(Duration::from_millis(25));
    };

    assert_eq!(status_snapshot["active"], json!(true));
    let runs = status_snapshot["runs"].as_array().unwrap();
    let alpha_run = runs
        .iter()
        .find(|r| r["task"] == "TASK_ALPHA")
        .expect("alpha run present");
    let beta_run = runs
        .iter()
        .find(|r| r["task"] == "TASK_BETA")
        .expect("beta run present");
    let alpha_run_id = alpha_run["run_id"].as_str().unwrap().to_string();
    let beta_run_id = beta_run["run_id"].as_str().unwrap().to_string();
    assert_ne!(alpha_run_id, beta_run_id);
    assert_ne!(alpha_run["thread_id"], beta_run["thread_id"]);
    assert_eq!(alpha_run["person_id"], json!("default"));

    // Cancel ONLY alpha by its run_id.
    let (st_c, cancel_res) = http_req(
        addr,
        "POST",
        &format!("/api/runs/{alpha_run_id}/cancel"),
        "{}",
    );
    assert_eq!(st_c, 200, "cancel alpha: {cancel_res}");
    assert_eq!(cancel_res["cancelled"], json!(true));
    assert_eq!(cancel_res["run_id"], json!(alpha_run_id));

    // Unblock alpha so it observes cancel and exits; beta remains in LiveRuns!
    let _ = alpha_gate_tx.send(());
    let (st_a, res_a) = h_alpha.join().unwrap();
    assert_eq!(st_a, 200, "alpha response: {res_a}");
    assert_eq!(res_a["cancelled"], json!(true), "alpha should cancel: {res_a}");

    // Verify RunGuard::drop for alpha removed ONLY alpha, leaving beta still active!
    let (st_mid, mid_status) = http_req(addr, "GET", "/api/runs/status", "");
    assert_eq!(st_mid, 200);
    assert_eq!(mid_status["active"], json!(true));
    let mid_runs = mid_status["runs"].as_array().unwrap();
    assert_eq!(mid_runs.len(), 1, "beta must remain in LiveRuns: {mid_status}");
    assert_eq!(mid_runs[0]["run_id"], json!(beta_run_id));

    // Unblock beta and verify it completes cleanly.
    let _ = beta_gate_tx.send(());
    let (st_b, res_b) = h_beta.join().unwrap();
    assert_eq!(st_b, 200, "beta response: {res_b}");
    assert!(res_b["cancelled"].is_null(), "beta must not be cancelled: {res_b}");
    assert!(env.cfg.workspace_dir.join("beta.md").exists());

    // After both finish, LiveRuns is idle.
    let (st_end, end_status) = http_req(addr, "GET", "/api/runs/status", "");
    assert_eq!(st_end, 200);
    assert_eq!(end_status["active"], json!(false));
    assert_eq!(end_status["runs"], json!([]));

    sse_stop.store(true, Ordering::SeqCst);
    let _ = sse_thread.join();
    let raw_sse = sse_events.lock().unwrap().clone();
    assert!(
        raw_sse.contains(&alpha_run_id) && raw_sse.contains(&beta_run_id),
        "SSE frames must include run_id for both concurrent runs: {raw_sse}"
    );
}

#[test]
fn parallel_dag_branch_checkpoints_resume_individually_and_preserve_step_snapshots() {
    let env = TestDir::new("dag-multi-cp", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(env.cfg.clone());

    let steps = json!([
        {
            "id": "prep_a",
            "label": "Prep A",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "PrepA", "sections": ["Branch A ready"] }
        },
        {
            "id": "prep_b",
            "label": "Prep B",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "PrepB", "sections": ["Branch B ready"] }
        },
        {
            "id": "cp_a",
            "label": "Approve A",
            "kind": "checkpoint",
            "depends_on": ["prep_a"],
            "checkpoint": {
                "prompt": "Provide note for branch A",
                "input_mode": "text"
            }
        },
        {
            "id": "cp_b",
            "label": "Approve B",
            "kind": "checkpoint",
            "depends_on": ["prep_b"],
            "checkpoint": {
                "prompt": "Confirm branch B",
                "input_mode": "approve_reject"
            }
        },
        {
            "id": "after_a",
            "label": "After A",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["cp_a"],
            "args": { "title": "OutA", "sections": ["AnswerA={{steps.cp_a.human_answer}}"] }
        },
        {
            "id": "after_b",
            "label": "After B",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["cp_b"],
            "args": { "title": "OutB", "sections": ["AnswerB={{steps.cp_b.human_answer}}"] }
        },
        {
            "id": "join",
            "label": "Join",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["after_a", "after_b"],
            "args": { "title": "FinalJoin", "sections": ["A={{steps.after_a.output}}", "B={{steps.after_b.output}}"] }
        }
    ]);

    let (st_run, run_res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Parallel Checkpoints", "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_run, 202, "start parallel cp pipeline: {run_res}");
    let run_id = run_res["run"]["run_id"].as_str().unwrap().to_string();

    let paused = poll_run_until(addr, &run_id, |r| r["status"] == "paused_checkpoint");
    let cps = paused["pending_checkpoints"].as_array().unwrap();
    assert_eq!(cps.len(), 2, "expected 2 simultaneous pending_checkpoints: {paused}");

    let cp_a = cps.iter().find(|c| c["step_id"] == "cp_a").unwrap();
    let cp_b = cps.iter().find(|c| c["step_id"] == "cp_b").unwrap();
    let cp_a_id = cp_a["checkpoint_id"].as_str().unwrap().to_string();
    let cp_b_id = cp_b["checkpoint_id"].as_str().unwrap().to_string();
    assert_ne!(cp_a_id, cp_b_id);

    // Resume ONLY cp_a using { checkpoint_id, decision, text } contract.
    let (st_ra, ra) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{run_id}/resume"),
        &json!({
            "checkpoint_id": cp_a_id,
            "decision": "answer",
            "text": "Alpha Approved Text"
        })
        .to_string(),
    );
    assert_eq!(st_ra, 202, "resume cp_a: {ra}");

    // Wait until after_a completes and run settles back to paused_checkpoint for cp_b!
    let paused_b = poll_run_until(addr, &run_id, |r| {
        r["status"] == "paused_checkpoint"
            && r["step_states"]
                .as_array()
                .is_some_and(|arr| arr.iter().any(|s| s["step_id"] == "after_a" && s["status"] == "ok"))
    });
    let rem_cps = paused_b["pending_checkpoints"].as_array().unwrap();
    assert_eq!(rem_cps.len(), 1, "only cp_b should remain: {paused_b}");
    assert_eq!(rem_cps[0]["checkpoint_id"], json!(cp_b_id));
    let out_a = std::fs::read_to_string(env.cfg.workspace_dir.join("OutA.md")).unwrap();
    assert!(out_a.contains("AnswerA=Alpha Approved Text"), "{out_a}");

    // Now resume cp_b.
    let (st_rb, rb) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{run_id}/resume"),
        &json!({
            "checkpoint_id": cp_b_id,
            "decision": "approve",
            "text": "Beta Confirmed"
        })
        .to_string(),
    );
    assert_eq!(st_rb, 202, "resume cp_b: {rb}");

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "parallel DAG failed: {done}");
    assert_eq!(done["pending_checkpoints"], json!([]));
    let out_b = std::fs::read_to_string(env.cfg.workspace_dir.join("OutB.md")).unwrap();
    assert!(out_b.contains("AnswerB=Beta Confirmed"), "{out_b}");
    assert!(env.cfg.workspace_dir.join("FinalJoin.md").exists());

    // Verify per-step step_snapshots serialization round-trip preserves independent branch snapshots.
    let mut prun = pipelines::get_run(&env.cfg.data_dir, &run_id).unwrap();
    prun.set_step_snapshot(
        "branch_a",
        Some(ergon_rs::AgentSnapshot {
            messages: vec![json!({"role": "user", "content": "Branch A ctx"})],
            written: vec!["a.md".to_string()],
            budget_used: 1,
        }),
    );
    prun.set_step_snapshot(
        "branch_b",
        Some(ergon_rs::AgentSnapshot {
            messages: vec![json!({"role": "user", "content": "Branch B ctx"})],
            written: vec!["b.md".to_string()],
            budget_used: 2,
        }),
    );
    let round_trip = pipelines::PipelineRun::from_json(&prun.to_json()).unwrap();
    assert_eq!(
        round_trip.step_snapshot("branch_a").unwrap().written,
        vec!["a.md".to_string()]
    );
    assert_eq!(
        round_trip.step_snapshot("branch_b").unwrap().written,
        vec!["b.md".to_string()]
    );
}

#[test]
fn offline_zero_llm_steps_v2_scheduled_routine_executes_without_model() {
    // No LLM key or endpoint configured!
    let env = TestDir::new("offline-sched", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server_with_scheduler(env.cfg.clone(), true);

    // 1. Offline deterministic steps_v2 routine scheduled every 1s.
    let (st_r1, r1) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "Offline Digest",
            "prompt": "Build deterministic digest",
            "schedule": "every 1s",
            "enabled": true,
            "steps_v2": [
                {
                    "id": "write_doc",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": {
                        "title": "OfflineScheduled",
                        "sections": ["Generated offline without LLM"]
                    }
                },
                {
                    "id": "export_doc",
                    "kind": "export",
                    "depends_on": ["write_doc"],
                    "export": { "path": "OfflineScheduled.md", "format": "docx" }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_r1, 201, "create offline routine: {r1}");
    assert_eq!(r1["routine"]["offline_deterministic"], json!(true));
    let rid_offline = r1["routine"]["id"].as_str().unwrap().to_string();

    // 2. Hybrid LLM routine (must be skipped cleanly by scheduler when !model_configured).
    let (st_r2, r2) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "LLM Routine",
            "prompt": "Needs model",
            "schedule": "every 1s",
            "enabled": true,
            "steps_v2": [
                {
                    "id": "think",
                    "kind": "agent",
                    "prompt": "Summarize workspace"
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_r2, 201, "create llm routine: {r2}");
    assert_eq!(r2["routine"]["offline_deterministic"], json!(false));

    // Wait up to 6s for scheduler thread to fire the offline routine automatically.
    let deadline = Instant::now() + Duration::from_secs(6);
    let mut fired_ok = false;
    while Instant::now() < deadline {
        let (st_h, hist) = http_req(
            addr,
            "GET",
            &format!("/api/routines/{rid_offline}/history"),
            "",
        );
        if st_h == 200
            && hist["runs"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x["status"] == "ok" && x["trigger"] == "scheduled"))
        {
            fired_ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    assert!(fired_ok, "offline steps_v2 routine did not fire on scheduler");
    assert!(
        env.cfg.workspace_dir.join("OfflineScheduled.md").exists(),
        "scheduled offline skill output missing"
    );
    assert!(
        env.cfg.workspace_dir.join("OfflineScheduled.docx").exists(),
        "scheduled offline export output missing"
    );

    // 3. Also verify sync POST /api/routines/:id/run routes steps_v2 through create_and_start_pipeline.
    let (st_sync, sync_res) = http_req(
        addr,
        "POST",
        &format!("/api/routines/{rid_offline}/run"),
        "{}",
    );
    assert_eq!(st_sync, 200, "sync run offline steps_v2: {sync_res}");
    assert_eq!(sync_res["run"]["status"], json!("ok"));
}

#[test]
fn retry_timeout_on_error_and_subpipeline_execution() {
    std::env::set_var("ERGON_FETCH_ALLOW_PRIVATE", "1");

    // Flaky HTTP endpoint: 1st request -> 500, 2nd request -> 200.
    // Slow HTTP endpoint: sleeps 2s before replying (triggers timeout_secs: 1).
    let flaky_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let flaky_addr = flaky_listener.local_addr().unwrap();
    let flaky_calls = Arc::new(AtomicUsize::new(0));
    let flaky_calls_c = flaky_calls.clone();
    std::thread::spawn(move || {
        for mut stream in flaky_listener.incoming().flatten() {
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            let n = flaky_calls_c.fetch_add(1, Ordering::SeqCst);
            let resp = if n == 0 {
                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 4\r\nConnection: close\r\n\r\nfail"
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
            };
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let slow_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let slow_addr = slow_listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for mut stream in slow_listener.incoming().flatten() {
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            std::thread::sleep(Duration::from_secs(2));
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
        }
    });

    let env = TestDir::new("retry-sub", "http://127.0.0.1:9/v1", None);
    env.write_ws("payload.md", "Retry payload");
    let addr = spawn_server(env.cfg.clone());

    let (st_c1, c1) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Flaky Hook",
            "kind": "http_webhook",
            "url": format!("http://127.0.0.1:{}/flaky", flaky_addr.port()),
            "method": "POST",
            "require_approval": false
        })
        .to_string(),
    );
    assert_eq!(st_c1, 201, "{c1}");
    let flaky_conn_id = c1["connector"]["id"].as_str().unwrap().to_string();

    let (st_c2, c2) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Slow Hook",
            "kind": "http_webhook",
            "url": format!("http://127.0.0.1:{}/slow", slow_addr.port()),
            "method": "POST",
            "require_approval": false
        })
        .to_string(),
    );
    assert_eq!(st_c2, 201, "{c2}");
    let slow_conn_id = c2["connector"]["id"].as_str().unwrap().to_string();

    let dead_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let dead_port = dead_listener.local_addr().unwrap().port();
    drop(dead_listener);
    let (st_c3, c3) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Dead Hook",
            "kind": "http_webhook",
            "url": format!("http://127.0.0.1:{dead_port}/dead"),
            "method": "POST",
            "require_approval": false
        })
        .to_string(),
    );
    assert_eq!(st_c3, 201, "{c3}");
    let dead_conn_id = c3["connector"]["id"].as_str().unwrap().to_string();

    // Child routine for Subpipeline step.
    let (st_child, child_res) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "Child Writer",
            "prompt": "Write child doc",
            "schedule": "manual",
            "steps_v2": [
                {
                    "id": "child_doc",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": {
                        "title": "ChildResult",
                        "sections": ["ChildTag={{inputs.tag}}"]
                    }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_child, 201, "{child_res}");
    let child_rid = child_res["routine"]["id"].as_str().unwrap().to_string();

    // Parent sequential pipeline exercising:
    // 1) retry_max + retry_backoff_ms (succeeds on 2nd attempt)
    // 2) timeout_secs: 1 + on_error: "goto:fallback" (skips normal_after, jumps to fallback)
    // 3) fallback executes subpipeline step
    // 4) on_error: "continue" on bad deliver -> continues to final step
    let parent_steps = json!([
        {
            "id": "flaky_deliver",
            "kind": "deliver",
            "deliver": { "connector_id": flaky_conn_id, "file": "payload.md" },
            "retry_max": 2,
            "retry_backoff_ms": 20
        },
        {
            "id": "slow_step",
            "kind": "deliver",
            "deliver": { "connector_id": slow_conn_id, "file": "payload.md" },
            "timeout_secs": 1,
            "on_error": "goto:fallback"
        },
        {
            "id": "normal_after",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "ShouldBeSkipped", "sections": ["nope"] }
        },
        {
            "id": "fallback",
            "kind": "subpipeline",
            "routine_id": child_rid,
            "inputs": { "tag": "RecoveredFromTimeout" }
        },
        {
            "id": "soft_fail",
            "kind": "deliver",
            "deliver": { "connector_id": dead_conn_id, "file": "payload.md" },
            "on_error": "continue"
        },
        {
            "id": "finish",
            "kind": "skill",
            "skill_id": "write_document",
            "args": { "title": "ParentFinal", "sections": ["Sub={{steps.fallback.files[0]}}"] }
        }
    ]);

    let (st_v, res_v) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({ "steps_v2": parent_steps }).to_string(),
    );
    assert_eq!(st_v, 200, "validate parent: {res_v}");

    let (st_run, run_res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Policy + Subpipeline", "steps_v2": parent_steps }).to_string(),
    );
    assert_eq!(st_run, 202, "{run_res}");
    let run_id = run_res["run"]["run_id"].as_str().unwrap().to_string();

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "parent run failed: {done}");
    assert_eq!(flaky_calls.load(Ordering::SeqCst), 2);

    let states = done["step_states"].as_array().unwrap();
    assert_eq!(states[0]["status"], json!("ok"));
    assert_eq!(states[0]["metrics"]["attempts"], json!(2));
    assert_eq!(states[1]["status"], json!("failed"));
    assert!(
        states[1]["error"].as_str().unwrap().contains("timed out"),
        "{}",
        states[1]
    );
    assert_eq!(states[2]["status"], json!("skipped"));
    assert_eq!(states[3]["status"], json!("ok"));
    assert_eq!(states[4]["status"], json!("failed"));
    assert_eq!(states[5]["status"], json!("ok"));

    let child_doc = std::fs::read_to_string(env.cfg.workspace_dir.join("ChildResult.md")).unwrap();
    assert!(child_doc.contains("ChildTag=RecoveredFromTimeout"), "{child_doc}");
    let parent_doc = std::fs::read_to_string(env.cfg.workspace_dir.join("ParentFinal.md")).unwrap();
    assert!(parent_doc.contains("Sub=ChildResult.md"), "{parent_doc}");
    assert!(!env.cfg.workspace_dir.join("ShouldBeSkipped.md").exists());
}

#[test]
fn five_field_cron_and_timezone_scheduling_unit_and_http() {
    use scheduler::Cadence;

    // Preserve legacy rejection of "cron * * * * *".
    assert!(scheduler::parse_cadence("cron * * * * *").is_err());

    let c1 = scheduler::parse_cadence("0 9 * * 1-5").unwrap();
    assert_eq!(
        c1,
        Cadence::Cron {
            expr: "0 9 * * 1-5".to_string(),
            tz_offset_mins: 0
        }
    );
    let c2 = scheduler::parse_cadence("30 14 * * mon-fri -05:00").unwrap();
    assert_eq!(
        c2,
        Cadence::Cron {
            expr: "30 14 * * mon-fri".to_string(),
            tz_offset_mins: -300
        }
    );
    assert_eq!(scheduler::parse_tz_offset_mins("America/New_York").unwrap(), -300);
    assert_eq!(scheduler::parse_tz_offset_mins("UTC+05:30").unwrap(), 330);

    // 2028-02-25 is Friday (1_835_049_600s = 2028-02-25T00:00:00Z).
    // At 2028-02-25T15:00:00Z (10:00 in -05:00), next 09:00 -05:00 Mon-Fri MUST skip Sat 26 & Sun 27
    // and land on Mon 2028-02-28T09:00-05:00 = 2028-02-28T14:00:00Z!
    let fri_1500_utc_ms: u64 = (1_835_049_600 + 15 * 3_600) * 1_000;
    let cron_ny = Cadence::Cron {
        expr: "0 9 * * 1-5".to_string(),
        tz_offset_mins: -300,
    };
    let next_ny = scheduler::next_run_ms(&cron_ny, Some(fri_1500_utc_ms), fri_1500_utc_ms).unwrap();
    assert_eq!(records::civil_from_millis(next_ny), (2028, 2, 28));
    assert_eq!((next_ny / 1_000) % 86_400, 14 * 3_600);

    // Leap day cron: "0 12 29 2 *" from 2027-01-01T00:00:00Z -> 2028-02-29T12:00:00Z.
    let jan_2027_ms: u64 = 1_798_761_600 * 1_000;
    let leap_cron = scheduler::parse_cadence("0 12 29 2 *").unwrap();
    let next_leap = scheduler::next_run_ms(&leap_cron, Some(jan_2027_ms), jan_2027_ms).unwrap();
    assert_eq!(records::civil_from_millis(next_leap), (2028, 2, 29));
    assert_eq!((next_leap / 1_000) % 86_400, 12 * 3_600);

    // HTTP POST /api/routines + PATCH /api/routines/:id with 5-field cron + tz_offset_mins.
    let env = TestDir::new("cron-http", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(env.cfg.clone());
    let (st_p, post_res) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "NY Weekday Morning",
            "prompt": "Run morning check",
            "schedule": "0 9 * * 1-5",
            "tz_offset_mins": -300,
            "tz": "America/New_York",
            "enabled": true
        })
        .to_string(),
    );
    assert_eq!(st_p, 201, "POST cron routine: {post_res}");
    let rtn = &post_res["routine"];
    assert_eq!(rtn["schedule"], json!("0 9 * * 1-5"));
    assert_eq!(rtn["tz_offset_mins"], json!(-300));
    assert_eq!(rtn["cadence_label"], json!("0 9 * * 1-5 -05:00"));
    assert!(rtn["interval_secs"].is_null());
    // 09:00 -05:00 = 14:00:00Z
    assert_eq!(rtn["next_run_ms"].as_u64().unwrap() % 86_400_000, 14 * 3_600_000);

    let rid = rtn["id"].as_str().unwrap();
    let (st_patch, patch_res) = http_req(
        addr,
        "PATCH",
        &format!("/api/routines/{rid}"),
        &json!({ "schedule": "30 8 * * *", "tz": "+05:30" }).to_string(),
    );
    assert_eq!(st_patch, 200, "PATCH cron routine: {patch_res}");
    let pr = &patch_res["routine"];
    assert_eq!(pr["schedule"], json!("30 8 * * *"));
    assert_eq!(pr["tz_offset_mins"], json!(330));
    // 08:30 +05:30 = 03:00:00Z
    assert_eq!(pr["next_run_ms"].as_u64().unwrap() % 86_400_000, 3 * 3_600_000);

    // POST /api/routines rejects invalid cron expressions and invalid tz with HTTP 400,
    // while keeping legacy descriptive schedules ("weekly") and Asia/Singapore cron working.
    for bad_sched in ["99 9 * * *", "cron * * * * *", "* * * *"] {
        let (st_bad, res_bad) = http_req(
            addr,
            "POST",
            "/api/routines",
            &json!({
                "name": "Bad Cron",
                "prompt": "Should fail",
                "schedule": bad_sched,
            })
            .to_string(),
        );
        assert_eq!(st_bad, 400, "expected 400 for bad schedule '{bad_sched}': {res_bad}");
    }
    let (st_bad_tz, res_bad_tz) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "Bad TZ",
            "prompt": "Should fail",
            "schedule": "0 9 * * 1-5",
            "tz": "Mars/Olympus",
        })
        .to_string(),
    );
    assert_eq!(st_bad_tz, 400, "expected 400 for bad tz: {res_bad_tz}");

    let (st_sg, res_sg) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "SG Morning",
            "prompt": "Run check",
            "schedule": "15 9 * * 1-5",
            "tz": "Asia/Singapore",
        })
        .to_string(),
    );
    assert_eq!(st_sg, 201, "expected 201 for Asia/Singapore: {res_sg}");
    assert_eq!(res_sg["routine"]["tz_offset_mins"], json!(480));

    let (st_wk, res_wk) = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "Legacy Weekly",
            "prompt": "Run weekly check",
            "schedule": "weekly",
        })
        .to_string(),
    );
    assert_eq!(st_wk, 201, "expected 201 for legacy weekly: {res_wk}");
}
