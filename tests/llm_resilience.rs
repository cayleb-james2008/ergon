//! Integration tests for live LLM run resilience:
//! 1. True mid-run crash (`SIGKILL` of real `ergon` binary) + restart auto-recovery
//!    and automatic resume from persisted `$ERGON_DATA_DIR/live_runs.json` (mode `0600`).
//! 2. Unconfigured-model boot recovery: honest `interrupted` state in `/api/runs/status`,
//!    honest HTTP `503` on `/api/runs/resume` while unconfigured, then clean manual
//!    one-click `POST /api/runs/resume` after configuring a model.
//! 3. Persisted `cancelled`, `failed`, and corrupt `live_runs.json` quarantine recovery.
//! 4. SSE replay (`?since=` and `Last-Event-ID`) + zero-gap concurrent event delivery.

use ergon_rs::{live_runs, records, serve, AgentSnapshot, Config, Store};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "llm-resilience-{}-{}-{name}",
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
            model_id: "mock-resilience".to_string(),
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

struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }

    fn kill_hard(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_hard();
    }
}

fn pick_free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn spawn_inproc_server(cfg: Config) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    wait_for_server_ready(addr);
    addr
}

fn wait_for_server_ready(addr: SocketAddr) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(200)) {
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let req = format!(
                "GET /api/version HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                addr.port()
            );
            if s.write_all(req.as_bytes()).is_ok() {
                let mut raw = Vec::new();
                if s.read_to_end(&mut raw).is_ok() && raw.starts_with(b"HTTP/1.1 200") {
                    return;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("server at {addr} did not become ready in time");
}

fn read_http_request_body(stream: &TcpStream) -> String {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let Ok(n) = reader.read_line(&mut line) else {
            break;
        };
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 0 {
        let mut body = vec![0u8; content_length.min(4 * 1024 * 1024)];
        let _ = reader.read_exact(&mut body);
        return String::from_utf8_lossy(&body).into_owned();
    }
    String::new()
}

fn http_req_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    extra_headers: &[(&str, &str)],
) -> (u16, String, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n",
        addr.port()
    );
    if !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    for (k, v) in extra_headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body.as_bytes()).unwrap();
    }
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP header separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_str = String::from_utf8_lossy(&raw[sep + 4..]).into_owned();
    let val = serde_json::from_str(&body_str).unwrap_or(Value::Null);
    (status, body_str, val)
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let (st, _, v) = http_req_headers(addr, method, path, body, &[]);
    (st, v)
}

fn spawn_sse_collector(
    addr: SocketAddr,
    path: &str,
    last_event_id: Option<&str>,
    stop_event: &'static str,
) -> std::thread::JoinHandle<Vec<(String, Value)>> {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(12)))
        .unwrap();
    let mut req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: keep-alive\r\n",
        addr.port()
    );
    if let Some(id) = last_event_id {
        req.push_str(&format!("Last-Event-ID: {id}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut reader = BufReader::new(stream);
    // Drain HTTP response headers
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }
    // Synchronize on the initial `event: status` frame before returning handle
    let mut saw_status_data = false;
    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 {
            break;
        }
        if line.starts_with("data: ") {
            saw_status_data = true;
        } else if saw_status_data && (line == "\r\n" || line == "\n") {
            break;
        }
    }
    std::thread::spawn(move || {
        let mut events = Vec::new();
        let mut cur_event = String::new();
        let deadline = Instant::now() + Duration::from_secs(12);
        while Instant::now() < deadline {
            let mut l = String::new();
            match reader.read_line(&mut l) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let t = l.trim_end_matches(['\r', '\n']);
                    if let Some(ev) = t.strip_prefix("event: ") {
                        cur_event = ev.to_string();
                    } else if let Some(data) = t.strip_prefix("data: ") {
                        if let Ok(v) = serde_json::from_str::<Value>(data) {
                            let ev_name = cur_event.clone();
                            events.push((ev_name.clone(), v));
                            if ev_name == stop_event {
                                break;
                            }
                        }
                    }
                }
            }
        }
        events
    })
}

fn spawn_mock_openai_queue(responses: Vec<(u16, Value)>) -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_bg = calls.clone();
    let queue = Arc::new(Mutex::new(responses));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let _ = read_http_request_body(&stream);
            let idx = calls_bg.fetch_add(1, Ordering::SeqCst);
            let (status, payload) = {
                let q = queue.lock().unwrap();
                q[idx.min(q.len() - 1)].clone()
            };
            let body = payload.to_string();
            let reason = if status == 200 { "OK" } else { "Bad Request" };
            let resp = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (addr, calls)
}

#[test]
fn mid_run_sigkill_crash_auto_resumes_on_restart_and_completes() {
    let td = TestDir::new("sigkill-auto-resume");

    // Mock OpenAI server that:
    // - Call 1: executes `write_file("phase1.md", ...)`
    // - Call 2: signals the test that step 1 finished & persisted, then blocks until crash kills ergon
    // - Call 3 (after restart auto-resume!): verifies step 1 context survived and writes `phase2.md`
    // - Call 4: finishes with done=true
    let mock_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    let (entered_call2_tx, entered_call2_rx) = mpsc::channel::<()>();
    let (release_call2_tx, release_call2_rx) = mpsc::channel::<()>();
    let resumed_bodies: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let resumed_bodies_bg = resumed_bodies.clone();
    let mock_calls = Arc::new(AtomicUsize::new(0));
    let mock_calls_bg = mock_calls.clone();

    std::thread::spawn(move || {
        let release_rx = Mutex::new(Some(release_call2_rx));
        for stream in mock_listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let req_body = read_http_request_body(&stream);
            let idx = mock_calls_bg.fetch_add(1, Ordering::SeqCst);
            if idx == 1 {
                let _ = entered_call2_tx.send(());
                if let Some(rx) = release_rx.lock().unwrap().take() {
                    let _ = rx.recv_timeout(Duration::from_secs(15));
                }
                // Original client was SIGKILLed; close cleanly and accept next connections.
                continue;
            }
            if idx >= 2 {
                resumed_bodies_bg.lock().unwrap().push(req_body);
            }
            let payload = match idx {
                0 => json!({
                    "model": "mock-resilience",
                    "choices": [{
                        "message": {
                            "content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"phase1.md\", \"content\": \"# Phase 1\\nWritten before crash.\\n\"}}"
                        }
                    }],
                    "usage": {"prompt_tokens": 120, "completion_tokens": 40, "total_tokens": 160}
                }),
                2 => json!({
                    "model": "mock-resilience",
                    "choices": [{
                        "message": {
                            "content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"phase2.md\", \"content\": \"# Phase 2\\nWritten after restart auto-resume.\\n\"}}"
                        }
                    }],
                    "usage": {"prompt_tokens": 150, "completion_tokens": 50, "total_tokens": 200}
                }),
                _ => json!({
                    "model": "mock-resilience",
                    "choices": [{
                        "message": {
                            "content": "{\"done\": true, \"summary\": \"Completed phase1.md and phase2.md cleanly across server restart.\"}"
                        }
                    }],
                    "usage": {"prompt_tokens": 160, "completion_tokens": 45, "total_tokens": 205}
                }),
            };
            let body = payload.to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });

    // 1. Boot a real `ergon` subprocess against `td.cfg.data_dir` and our mock OpenAI server.
    let bin = env!("CARGO_BIN_EXE_ergon");
    let first_port = pick_free_port();
    let first_addr: SocketAddr = format!("127.0.0.1:{first_port}").parse().unwrap();
    let child = Command::new(bin)
        .args(["--no-open", "--host", "127.0.0.1", "--port", &first_port.to_string()])
        .env("ERGON_DATA_DIR", &td.cfg.data_dir)
        .env("ERGON_WORKSPACE", &td.cfg.workspace_dir)
        .env(
            "ERGON_MODEL_BASE_URL",
            format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        )
        .env("ERGON_MODEL_ID", "mock-resilience")
        .env("ERGON_DISABLE_SCHEDULER", "1")
        .env("ERGON_NO_AUTO_MODEL", "1")
        .env_remove("ERGON_AUTO_RESUME_RUNS")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ergon binary");
    let mut child_guard = ChildGuard::new(child);
    wait_for_server_ready(first_addr);

    // 2. Kick off an ad-hoc chat task in a background thread.
    let chat_handle = std::thread::spawn(move || {
        let mut s = TcpStream::connect(first_addr).unwrap();
        let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
        let payload = json!({
            "message": "Write phase1.md then phase2.md and summarize."
        })
        .to_string();
        let req = format!(
            "POST /api/chat HTTP/1.1\r\nHost: 127.0.0.1:{first_port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
            payload.len()
        );
        let _ = s.write_all(req.as_bytes());
        let mut sink = Vec::new();
        let _ = s.read_to_end(&mut sink);
    });

    // Wait deterministically until step 1 completed and LLM call 2 is in flight.
    entered_call2_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("mock OpenAI call 2 reached");

    // Verify `live_runs.json` persisted atomically with mode 0600 and step 1 snapshot BEFORE crash!
    let live_path = live_runs::live_runs_path(&td.cfg.data_dir);
    assert!(live_path.exists(), "live_runs.json must exist mid-run");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&live_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "live_runs.json mode must be 0600, got {mode:o}");
    }
    let mid_runs = live_runs::load_all(&td.cfg.data_dir);
    assert_eq!(mid_runs.len(), 1, "expected 1 persisted run mid-flight: {mid_runs:?}");
    let crashed_run_id = mid_runs[0].run_id.clone();
    let crashed_thread_id = mid_runs[0].thread_id.clone();
    assert_eq!(mid_runs[0].status, "running");
    assert_eq!(mid_runs[0].person_id, "default");
    assert_eq!(mid_runs[0].steps.len(), 1);
    assert_eq!(mid_runs[0].written_files, vec!["phase1.md".to_string()]);
    assert!(
        mid_runs[0].snapshot.is_some(),
        "step checkpoint snapshot must be persisted mid-run"
    );

    // 3. Hard-kill the server process (SIGKILL — zero Drop handlers run, true crash).
    child_guard.kill_hard();
    let _ = release_call2_tx.send(());
    let _ = chat_handle.join();

    // Confirm on-disk file is still honestly in "running" state right after hard crash.
    let post_crash = live_runs::get_run(&td.cfg.data_dir, &crashed_run_id).unwrap();
    assert_eq!(post_crash.status, "running");
    assert!(td.cfg.workspace_dir.join("phase1.md").exists());
    assert!(!td.cfg.workspace_dir.join("phase2.md").exists());

    // 4. Restart a fresh server on the SAME data_dir with the configured mock OpenAI endpoint!
    let mut restart_cfg = td.cfg.clone();
    restart_cfg.model_base_url = format!("http://127.0.0.1:{}/v1", mock_addr.port());
    let restarted_addr = spawn_inproc_server(restart_cfg);

    // 5. Wait for automatic startup recovery + resume to complete cleanly.
    let deadline = Instant::now() + Duration::from_secs(10);
    let final_run = loop {
        if let Some(r) = live_runs::get_run(&td.cfg.data_dir, &crashed_run_id) {
            if r.status == "completed" {
                break r;
            }
            assert_ne!(r.status, "failed", "auto-resume failed: {r:?}");
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for auto-resume completion: {:?}",
            live_runs::get_run(&td.cfg.data_dir, &crashed_run_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    };

    assert_eq!(final_run.status, "completed");
    assert_eq!(final_run.resume_count, 1);
    assert!(final_run.finished_ms.is_some());
    assert!(final_run.snapshot.is_none(), "snapshot should be pruned on completion");
    assert!(
        final_run.written_files.contains(&"phase1.md".to_string())
            && final_run.written_files.contains(&"phase2.md".to_string()),
        "both pre-crash and post-resume files must be credited: {:?}",
        final_run.written_files
    );

    // Verify both workspace files exist with real content.
    let p1 = std::fs::read_to_string(td.cfg.workspace_dir.join("phase1.md")).unwrap();
    let p2 = std::fs::read_to_string(td.cfg.workspace_dir.join("phase2.md")).unwrap();
    assert!(p1.contains("Written before crash."));
    assert!(p2.contains("Written after restart auto-resume."));

    // Verify resumed prompt carried preserved step-1 context to the model.
    let captured = resumed_bodies.lock().unwrap();
    assert!(
        captured.iter().any(|b| b.contains("phase1.md")),
        "resumed model call must receive preserved step-1 snapshot context: {captured:?}"
    );

    // Verify `/api/runs/status` is clean idle with zero remaining recoverable runs.
    let (st_status, status_v) = http_req(restarted_addr, "GET", "/api/runs/status", "");
    assert_eq!(st_status, 200, "{status_v}");
    assert_eq!(status_v["active"], json!(false));
    assert_eq!(status_v["status"], json!("idle"));
    assert_eq!(status_v["recoverable_runs"], json!([]));

    // Verify SSE replay (`?once=1&since=0`) surfaces `run_recovered`, `run_resumed`, `step_done`, `run_finished`.
    let (st_sse, sse_body, _) =
        http_req_headers(restarted_addr, "GET", "/api/runs/stream?once=1&since=0", "", &[]);
    assert_eq!(st_sse, 200);
    assert!(sse_body.contains("event: status"), "{sse_body}");
    assert!(sse_body.contains("event: run_recovered"), "{sse_body}");
    assert!(sse_body.contains("event: run_resumed"), "{sse_body}");
    assert!(sse_body.contains("event: step_done"), "{sse_body}");
    assert!(sse_body.contains("event: run_finished"), "{sse_body}");

    // Verify receipt chain is valid and contains the completed task.
    let (st_rec, rec_v) = http_req(restarted_addr, "GET", "/api/receipts/verify", "");
    assert_eq!(st_rec, 200, "{rec_v}");
    assert_eq!(rec_v["valid"], json!(true));
    assert!(rec_v["total"].as_u64().unwrap_or(0) >= 1, "{rec_v}");

    // Verify thread preserved original prompt once + resume step + final assistant summary.
    let (st_th, th_v) = http_req(
        restarted_addr,
        "GET",
        &format!("/api/threads/{crashed_thread_id}"),
        "",
    );
    assert_eq!(st_th, 200, "{th_v}");
    let msgs = th_v["messages"].as_array().unwrap();
    let user_msgs: Vec<_> = msgs.iter().filter(|m| m["role"] == "user").collect();
    assert_eq!(user_msgs.len(), 1, "must not duplicate user prompt on resume: {msgs:?}");
    assert!(
        msgs.iter().any(|m| m["role"] == "assistant"
            && m["content"]
                .as_str()
                .unwrap_or("")
                .contains("phase2.md")),
        "expected final assistant summary in resumed thread: {msgs:?}"
    );
}

#[test]
fn unconfigured_boot_recovery_is_honest_and_manual_resume_api_completes() {
    let td = TestDir::new("unconfigured-then-manual-resume");

    // Seed thread + step-1 file + a crashed `running` record in `live_runs.json`.
    std::fs::write(
        td.cfg.workspace_dir.join("draft1.md"),
        "# Draft 1\nSaved before crash.\n",
    )
    .unwrap();
    let threads_path = td.cfg.data_dir.join("threads.json");
    let mut store = Store::load(&threads_path);
    let tid = store.create_thread();
    store.push(&tid, "user", "Write draft1.md then draft2.md.");
    store.push(&tid, "step", "Tool: write_file\nwrote draft1.md");

    let run_id = "run-crashed-unconfigured-1".to_string();
    let now = records::now_millis();
    live_runs::record_started(
        &td.cfg.data_dir,
        live_runs::PersistedLiveRun {
            run_id: run_id.clone(),
            thread_id: tid.clone(),
            person_id: "default".to_string(),
            task: "Write draft1.md then draft2.md.".to_string(),
            started_ms: now.saturating_sub(4000),
            updated_ms: now.saturating_sub(2000),
            finished_ms: None,
            steps: vec!["Tool: write_file\nwrote draft1.md".to_string()],
            written_files: vec!["draft1.md".to_string()],
            status: "running".to_string(),
            error: None,
            summary: None,
            resume_count: 0,
            snapshot: Some(AgentSnapshot {
                messages: vec![
                    json!({"role": "user", "content": "Write draft1.md then draft2.md."}),
                    json!({"role": "assistant", "content": "{\"tool\":\"write_file\",\"args\":{\"path\":\"draft1.md\",\"content\":\"# Draft 1\\nSaved before crash.\\n\"}}"}),
                    json!({"role": "user", "content": "wrote draft1.md"}),
                ],
                written: vec!["draft1.md".to_string()],
                budget_used: 1,
            }),
        },
    )
    .unwrap();

    // 1. Boot server with NO model configured (127.0.0.1:9 discard port, api_key: None).
    let addr = spawn_inproc_server(td.cfg.clone());

    // 2. Verify boot recovery transitioned `running` -> `interrupted` honestly (no fabrication).
    let recovered = live_runs::get_run(&td.cfg.data_dir, &run_id).unwrap();
    assert_eq!(recovered.status, "interrupted");
    assert_eq!(recovered.resume_count, 0);
    assert!(
        recovered
            .error
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase()
            .contains("model"),
        "expected honest unconfigured-model note: {:?}",
        recovered.error
    );

    let (st_status, status_v) = http_req(addr, "GET", "/api/runs/status", "");
    assert_eq!(st_status, 200, "{status_v}");
    assert_eq!(status_v["active"], json!(false));
    assert_eq!(status_v["status"], json!("interrupted"));
    assert_eq!(status_v["model_configured"], json!(false));
    let recs = status_v["recoverable_runs"].as_array().unwrap();
    assert_eq!(recs.len(), 1, "{status_v}");
    assert_eq!(recs[0]["run_id"], json!(run_id));
    assert_eq!(recs[0]["status"], json!("interrupted"));
    assert_eq!(recs[0]["can_resume"], json!(false));

    // 3. Attempting `POST /api/runs/resume` while no model is configured returns honest 503
    //    and leaves the run safely in `interrupted` state.
    let (st_503, resp_503) = http_req(
        addr,
        "POST",
        "/api/runs/resume",
        &json!({"run_id": run_id}).to_string(),
    );
    assert_eq!(st_503, 503, "{resp_503}");
    assert_eq!(resp_503["status"], json!("interrupted"));
    assert_eq!(
        live_runs::get_run(&td.cfg.data_dir, &run_id).unwrap().status,
        "interrupted"
    );

    // 4. Verify SSE status + replay reports the recovered run.
    let (st_sse, sse_body, _) =
        http_req_headers(addr, "GET", "/api/runs/stream?once=1&since=0", "", &[]);
    assert_eq!(st_sse, 200);
    assert!(sse_body.contains("event: status"), "{sse_body}");
    assert!(sse_body.contains(&run_id), "{sse_body}");
    assert!(sse_body.contains("event: run_recovered"), "{sse_body}");

    // 5. Now configure a loopback mock OpenAI server via `PUT /api/config`.
    let (mock_addr, mock_calls) = spawn_mock_openai_queue(vec![
        (
            200,
            json!({
                "model": "mock-resilience",
                "choices": [{
                    "message": {
                        "content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"draft2.md\", \"content\": \"# Draft 2\\nCompleted on manual resume.\\n\"}}"
                    }
                }],
                "usage": {"prompt_tokens": 90, "completion_tokens": 35, "total_tokens": 125}
            }),
        ),
        (
            200,
            json!({
                "model": "mock-resilience",
                "choices": [{
                    "message": {
                        "content": "{\"done\": true, \"summary\": \"Resumed cleanly and wrote draft2.md alongside draft1.md.\"}"
                    }
                }],
                "usage": {"prompt_tokens": 110, "completion_tokens": 30, "total_tokens": 140}
            }),
        ),
    ]);

    let (st_cfg, cfg_v) = http_req(
        addr,
        "PUT",
        "/api/config",
        &json!({
            "model_base_url": format!("http://127.0.0.1:{}/v1", mock_addr.port()),
            "model_id": "mock-resilience"
        })
        .to_string(),
    );
    assert_eq!(st_cfg, 200, "{cfg_v}");

    let (st_ready, ready_v) = http_req(addr, "GET", "/api/runs/status", "");
    assert_eq!(st_ready, 200);
    assert_eq!(ready_v["model_configured"], json!(true));
    assert_eq!(ready_v["recoverable_runs"][0]["can_resume"], json!(true));

    // 6. Subscribe to SSE and trigger one-click `POST /api/runs/resume`.
    let sse_handle = spawn_sse_collector(addr, "/api/runs/stream", None, "run_finished");
    let (st_res, res_v) = http_req(
        addr,
        "POST",
        "/api/runs/resume",
        &json!({"run_id": run_id}).to_string(),
    );
    assert_eq!(st_res, 200, "{res_v}");
    assert_eq!(res_v["resumed"], json!(true));
    assert_eq!(res_v["run_id"], json!(run_id));
    assert_eq!(res_v["status"], json!("completed"));
    assert_eq!(mock_calls.load(Ordering::SeqCst), 2);
    assert!(td.cfg.workspace_dir.join("draft2.md").exists());

    // 7. Verify SSE observed `run_resumed` -> `step_done` -> `run_finished`.
    let events = sse_handle.join().unwrap();
    assert!(
        events.iter().any(|(ev, d)| ev == "run_resumed" && d["run_id"] == json!(run_id)),
        "expected run_resumed SSE event: {events:?}"
    );
    assert!(
        events.iter().any(|(ev, d)| ev == "step_done" && d["run_id"] == json!(run_id)),
        "expected step_done SSE event: {events:?}"
    );
    assert!(
        events.iter().any(|(ev, d)| ev == "run_finished" && d["run_id"] == json!(run_id)),
        "expected run_finished SSE event: {events:?}"
    );

    // 8. Verify `live_runs.json` transitioned to `completed` and duplicate resume returns 409.
    let done = live_runs::get_run(&td.cfg.data_dir, &run_id).unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(done.resume_count, 1);

    let (st_dup, _) = http_req(
        addr,
        "POST",
        "/api/runs/resume",
        &json!({"run_id": run_id}).to_string(),
    );
    assert_eq!(st_dup, 409, "re-resuming completed run must return 409 Conflict");
}

#[test]
fn persisted_live_runs_tracks_cancelled_failed_and_corrupt_quarantine() {
    // 1. Corrupt `live_runs.json` is quarantined to `.bak` and reported by `/api/status`.
    let td_corrupt = TestDir::new("corrupt-quarantine");
    let path = live_runs::live_runs_path(&td_corrupt.cfg.data_dir);
    std::fs::write(&path, b"{not valid json!!!").unwrap();
    let addr_corrupt = spawn_inproc_server(td_corrupt.cfg.clone());
    assert!(
        td_corrupt.cfg.data_dir.join("live_runs.json.bak").exists(),
        "corrupt live_runs.json must be quarantined to live_runs.json.bak"
    );
    let (st_s, s_v) = http_req(addr_corrupt, "GET", "/api/status", "");
    assert_eq!(st_s, 200, "{s_v}");
    assert!(
        s_v["store_recovered"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "live_runs.json"),
        "store_recovered must report quarantined live_runs.json: {s_v}"
    );
    assert_eq!(s_v["capabilities"]["run_resume"], json!(true));

    // 2. Failed run persists `status: "failed"` with error message and 0600 permissions.
    let (mock_err_addr, _) = spawn_mock_openai_queue(vec![(
        400,
        json!({"error": {"message": "invalid prompt payload"}}),
    )]);
    let mut td_err = TestDir::new("failed-status");
    td_err.cfg.model_base_url = format!("http://127.0.0.1:{}/v1", mock_err_addr.port());
    let addr_err = spawn_inproc_server(td_err.cfg.clone());
    let (st_fail, fail_v) = http_req(
        addr_err,
        "POST",
        "/api/chat",
        &json!({"message": "Trigger a 400 error"}).to_string(),
    );
    assert_eq!(st_fail, 500, "{fail_v}");
    let err_runs = live_runs::load_all(&td_err.cfg.data_dir);
    assert_eq!(err_runs.len(), 1);
    assert_eq!(err_runs[0].status, "failed");
    assert!(err_runs[0].error.is_some());

    // 3. Cancelled run persists `status: "cancelled"`.
    let cancel_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let cancel_mock_addr = cancel_listener.local_addr().unwrap();
    let (step1_tx, step1_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        let mut call_n = 0usize;
        for stream in cancel_listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let _ = read_http_request_body(&stream);
            call_n += 1;
            let payload = if call_n == 1 {
                json!({
                    "model": "mock-resilience",
                    "choices": [{"message": {"content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"c1.md\", \"content\": \"ok\"}}"}}]
                })
            } else {
                let _ = step1_tx.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(10));
                json!({
                    "model": "mock-resilience",
                    "choices": [{"message": {"content": "{\"done\": true, \"summary\": \"late\"}"}}]
                })
            };
            let body = payload.to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });

    let mut td_cancel = TestDir::new("cancelled-status");
    td_cancel.cfg.model_base_url = format!("http://127.0.0.1:{}/v1", cancel_mock_addr.port());
    let addr_cancel = spawn_inproc_server(td_cancel.cfg.clone());
    let chat_t = std::thread::spawn(move || {
        http_req(
            addr_cancel,
            "POST",
            "/api/chat",
            &json!({"message": "Write c1.md then wait"}).to_string(),
        )
    });
    step1_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("call 2 reached");
    let (st_c, c_v) = http_req(addr_cancel, "POST", "/api/runs/active/cancel", "{}");
    assert_eq!(st_c, 200, "{c_v}");
    let _ = release_tx.send(());
    let (st_chat, chat_v) = chat_t.join().unwrap();
    assert_eq!(st_chat, 200, "{chat_v}");
    assert_eq!(chat_v["cancelled"], json!(true));
    let cancel_runs = live_runs::load_all(&td_cancel.cfg.data_dir);
    assert_eq!(cancel_runs.len(), 1);
    assert_eq!(cancel_runs[0].status, "cancelled");
}

#[test]
fn sse_last_event_id_replays_buffered_events_without_gaps() {
    let (mock_addr, _) = spawn_mock_openai_queue(vec![
        (
            200,
            json!({
                "model": "mock-resilience",
                "choices": [{"message": {"content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"replay.md\", \"content\": \"hello\"}}"}}]
            }),
        ),
        (
            200,
            json!({
                "model": "mock-resilience",
                "choices": [{"message": {"content": "{\"done\": true, \"summary\": \"Wrote replay.md.\"}"}}]
            }),
        ),
    ]);
    let mut td = TestDir::new("sse-last-event-id");
    td.cfg.model_base_url = format!("http://127.0.0.1:{}/v1", mock_addr.port());
    let addr = spawn_inproc_server(td.cfg.clone());

    let (st, chat_v) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({"message": "Write replay.md"}).to_string(),
    );
    assert_eq!(st, 200, "{chat_v}");

    // Connect AFTER the run finished with `Last-Event-ID: 0` — must replay all events!
    let sse_handle = spawn_sse_collector(addr, "/api/runs/stream", Some("0"), "run_finished");
    let events = sse_handle.join().unwrap();
    assert!(events.iter().any(|(e, _)| e == "run_started"), "{events:?}");
    assert!(events.iter().any(|(e, _)| e == "step_done"), "{events:?}");
    assert!(events.iter().any(|(e, _)| e == "file_written"), "{events:?}");
    assert!(events.iter().any(|(e, _)| e == "run_finished"), "{events:?}");
}
