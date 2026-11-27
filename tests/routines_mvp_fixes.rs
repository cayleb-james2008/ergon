//! Regression tests for Ergon MVP Gap 4 (Routines scheduler & Catch-Up fixes):
//! 1. Deterministic zero-LLM `steps_v2` routines execute automatically on the
//!    background scheduler tick even when NO model API key is configured
//!    (`!cfg.model_configured()`), writing real workspace output, emitting
//!    `routine_fired` SSE, recording `run_count == 1`, and verifying SHA-256
//!    receipt linkage (`last_run_verified == "verified"`).
//! 2. A single manual `Catch up` (and `Run now`) on a deterministic routine
//!    records EXACTLY 1 run (no double `routines::record_run`), clears missed
//!    windows, and marks `last_run_verified == "verified"` (no false `Drift`).
//! 3. LLM-requiring routines (prompt-only or `steps_v2` containing `agent` /
//!    `checkpoint` / `deliver`) with no model configured do NOT crash or hang
//!    the scheduler, do not spam background error runs, and report HTTP 503
//!    cleanly when explicitly triggered.

use ergon_rs::{pipelines, records, routines, serve, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn set(vars: &[(&'static str, Option<&str>)]) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for &(k, v) in vars {
            saved.push((k, std::env::var(k).ok()));
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
        Self { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, prev) in self.saved.drain(..) {
            match prev {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }
}

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new_unconfigured(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-routines-mvp-{}-{}-{name}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        // Discard port 9 + api_key None guarantees `!cfg.model_configured()`.
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "unconfigured-test".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        assert!(
            !cfg.model_configured(),
            "test config must have model_configured() == false"
        );
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
    std::thread::sleep(Duration::from_millis(40));
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
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

fn sample_invoices_csv() -> &'static str {
    "date,ref,vendor,category,amount,receipt\n\
     2026-11-01,INV-1001,Northwind Supply,Materials,420.50,verified\n\
     2026-11-02,INV-1002,Boreal Logistics,Freight,985.25,missing\n\
     2026-11-03,INV-1001,Northwind Supply,Materials,420.50,verified\n"
}

fn deterministic_ledger_routine(id: &str, name: &str, created_ms: u64, output_title: &str) -> Value {
    json!({
        "id": id,
        "name": name,
        "prompt": "Reconcile daily-invoices.csv deterministically",
        "schedule": "every 15m",
        "interval_secs": 900,
        "skill_id": "reconcile_ledger",
        "enabled": true,
        "created_ms": created_ms,
        "last_run_ms": Value::Null,
        "last_thread_id": Value::Null,
        "last_status": Value::Null,
        "run_count": 0,
        "steps_v2": [
            {
                "id": "reconcile",
                "label": "Reconcile daily invoices",
                "kind": "skill",
                "skill_id": "reconcile_ledger",
                "args": {
                    "source_files": ["daily-invoices.csv"],
                    "output_title": output_title
                },
                "when": "always"
            }
        ]
    })
}

#[test]
fn deterministic_zero_llm_detector_classifies_routines_accurately() {
    let now = records::now_millis();
    let skill_only = deterministic_ledger_routine("rtn-skill", "Skill", now, "Out");
    assert!(pipelines::routine_is_deterministic_zero_llm(&skill_only));

    let skill_and_export = json!({
        "id": "rtn-exp",
        "steps_v2": [
            {
                "id": "doc",
                "label": "Write doc",
                "kind": "skill",
                "skill_id": "write_document",
                "args": { "title": "Brief", "sections": ["Body"] },
                "when": "always"
            },
            {
                "id": "exp",
                "label": "Export docx",
                "kind": "export",
                "export": { "path": "Brief.md", "format": "docx" },
                "when": "on_previous_ok"
            }
        ]
    });
    assert!(pipelines::routine_is_deterministic_zero_llm(&skill_and_export));

    let prompt_only = json!({
        "id": "rtn-llm",
        "prompt": "Summarize emails",
        "schedule": "every 15m"
    });
    assert!(!pipelines::routine_is_deterministic_zero_llm(&prompt_only));

    let mixed_agent = json!({
        "id": "rtn-agent",
        "steps_v2": [
            { "id": "a", "label": "Ask AI", "kind": "agent", "prompt": "Analyze workspace" }
        ]
    });
    assert!(!pipelines::routine_is_deterministic_zero_llm(&mixed_agent));

    let with_checkpoint = json!({
        "id": "rtn-cp",
        "steps_v2": [
            {
                "id": "s",
                "label": "Write doc",
                "kind": "skill",
                "skill_id": "write_document",
                "args": { "title": "Draft", "sections": ["Body"] }
            },
            {
                "id": "cp",
                "label": "Approve",
                "kind": "checkpoint",
                "checkpoint": { "prompt": "Approve?" }
            }
        ]
    });
    assert!(!pipelines::routine_is_deterministic_zero_llm(&with_checkpoint));

    let empty_v2 = json!({ "id": "rtn-empty", "steps_v2": [] });
    assert!(!pipelines::routine_is_deterministic_zero_llm(&empty_v2));
}

#[test]
fn background_scheduler_runs_zero_llm_steps_v2_without_model_and_records_single_verified_run() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let _env_guard = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", None),
        ("ERGON_SCHEDULER_TICK_MS", Some("250")),
    ]);

    let env = TestDir::new_unconfigured("bg-zero-llm");
    env.write_ws("daily-invoices.csv", sample_invoices_csv());

    // Seed one due zero-LLM deterministic routine AND one due LLM-requiring routine.
    let old_ms = records::now_millis().saturating_sub(20 * 60 * 1000);
    let det_rt = deterministic_ledger_routine(
        "rtn-bg-det",
        "Auto Ledger Audit",
        old_ms,
        "Auto Scheduled Ledger Audit",
    );
    let llm_rt = json!({
        "id": "rtn-bg-llm",
        "name": "LLM Summary Routine",
        "prompt": "Summarize workspace changes with a model",
        "schedule": "every 15m",
        "interval_secs": 900,
        "enabled": true,
        "created_ms": old_ms,
        "last_run_ms": Value::Null,
        "run_count": 0
    });
    std::fs::write(
        routines::routines_path(&env.cfg.data_dir),
        json!({ "routines": [det_rt, llm_rt] }).to_string(),
    )
    .unwrap();

    let addr = spawn_server(env.cfg.clone());

    // Capture live `routine_fired` SSE notifications from the background scheduler.
    let sse_fired: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sse_clone = sse_fired.clone();
    std::thread::spawn(move || {
        if let Ok(mut s) = TcpStream::connect(addr) {
            let _ = s.set_read_timeout(Some(Duration::from_secs(4)));
            let req = format!(
                "GET /api/runs/stream HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: keep-alive\r\n\r\n",
                addr.port()
            );
            let _ = s.write_all(req.as_bytes());
            let mut reader = BufReader::new(s);
            let mut line = String::new();
            let mut is_routine_fired = false;
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                if let Some(ev) = line.strip_prefix("event: ") {
                    is_routine_fired = ev.trim() == "routine_fired";
                } else if is_routine_fired {
                    if let Some(data) = line.strip_prefix("data: ") {
                        sse_clone.lock().unwrap().push(data.trim().to_string());
                        break;
                    }
                }
                line.clear();
            }
        }
    });

    // Wait for the background scheduler tick to execute `rtn-bg-det` automatically.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let (st, body) = http_req(addr, "GET", "/api/routines", "");
        assert_eq!(st, 200, "GET /api/routines failed: {body}");
        let list = body["routines"].as_array().unwrap();
        let det_now = list
            .iter()
            .find(|r| r["id"] == "rtn-bg-det")
            .cloned()
            .unwrap_or(Value::Null);
        if det_now["run_count"].as_u64().unwrap_or(0) >= 1 {
            break;
        }
        if Instant::now() >= deadline {
            panic!("background scheduler did not auto-fire zero-LLM routine; last state: {body}");
        }
        std::thread::sleep(Duration::from_millis(80));
    }

    // Wait across two extra scheduler ticks to verify no duplicate firing occurs.
    std::thread::sleep(Duration::from_millis(650));
    let (st, recheck) = http_req(addr, "GET", "/api/routines", "");
    assert_eq!(st, 200);
    let list = recheck["routines"].as_array().unwrap();
    let det_after = list
        .iter()
        .find(|r| r["id"] == "rtn-bg-det")
        .cloned()
        .unwrap();
    let llm_after = list
        .iter()
        .find(|r| r["id"] == "rtn-bg-llm")
        .cloned()
        .unwrap();

    // 1. Deterministic zero-LLM routine ran ONCE automatically, wrote real file, and verified cleanly.
    assert_eq!(det_after["run_count"], json!(1), "expected run_count == 1: {det_after}");
    assert_eq!(det_after["missed_windows"], json!(0), "{det_after}");
    assert_eq!(det_after["last_status"], json!("finished"), "{det_after}");
    assert_eq!(
        det_after["last_run_verified"],
        json!("verified"),
        "expected Verified receipt badge, not Drift: {det_after}"
    );
    assert_eq!(
        det_after["last_run_receipt_sha256"]
            .as_str()
            .unwrap_or("")
            .len(),
        64,
        "{det_after}"
    );

    let out_file = env.cfg.workspace_dir.join("Auto Scheduled Ledger Audit.md");
    assert!(out_file.exists(), "expected real output file at {out_file:?}");
    let out_text = std::fs::read_to_string(&out_file).unwrap();
    assert!(
        out_text.contains("Northwind Supply") && out_text.contains("Boreal Logistics"),
        "expected real reconciled invoice data in output: {out_text}"
    );

    let (st, hist) = http_req(addr, "GET", "/api/routines/rtn-bg-det/history", "");
    assert_eq!(st, 200, "{hist}");
    assert_eq!(hist["count"], json!(1), "{hist}");
    assert_eq!(hist["runs"][0]["trigger"], json!("scheduled"), "{hist}");
    assert_eq!(hist["runs"][0]["status"], json!("ok"), "{hist}");
    assert!(
        hist["runs"][0]["receipt_id"].as_str().is_some_and(|s| !s.is_empty()),
        "expected linked receipt_id on run history: {hist}"
    );

    // SSE notification arrived for the deterministic scheduled run.
    let fired = sse_fired.lock().unwrap().clone();
    assert!(
        fired.iter().any(|d| d.contains("rtn-bg-det")),
        "expected routine_fired SSE event for rtn-bg-det, got: {fired:?}"
    );

    // 2. Co-located LLM-requiring routine did NOT crash the scheduler and stayed cleanly skipped.
    assert_eq!(llm_after["run_count"], json!(0), "unconfigured LLM routine should stay skipped on background ticks: {llm_after}");
    let (st, status_body) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(status_body["model_configured"], json!(false));
}

#[test]
fn single_catchup_on_missed_deterministic_routine_records_exactly_one_verified_run() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let _env_guard = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_SCHEDULER_TICK_MS", Some("60000")),
    ]);

    let env = TestDir::new_unconfigured("single-catchup");
    env.write_ws("daily-invoices.csv", sample_invoices_csv());

    // Seed an overdue deterministic routine (35 minutes old on every 15m -> 2 missed windows).
    let old_ms = records::now_millis().saturating_sub(35 * 60 * 1000);
    let det_rt = deterministic_ledger_routine(
        "rtn-catchup-det",
        "Missed Ledger Check",
        old_ms,
        "Catchup Ledger Audit",
    );
    std::fs::write(
        routines::routines_path(&env.cfg.data_dir),
        json!({ "routines": [det_rt] }).to_string(),
    )
    .unwrap();

    let addr = spawn_server(env.cfg.clone());

    // Before catch-up: >= 2 missed windows, 0 runs, no verification badge yet.
    let (st, before) = http_req(addr, "GET", "/api/routines", "");
    assert_eq!(st, 200);
    assert!(before["routines"][0]["missed_windows"].as_u64().unwrap() >= 2);
    assert_eq!(before["routines"][0]["run_count"], json!(0));
    assert_eq!(before["routines"][0]["last_run_verified"], Value::Null);

    // Trigger a single manual Catch Up.
    let (st, cu) = http_req(addr, "POST", "/api/routines/rtn-catchup-det/catchup", "{}");
    assert_eq!(st, 200, "POST /catchup failed: {cu}");
    assert_eq!(cu["caught_up"], json!(true), "{cu}");
    assert!(cu["missed_windows_cleared"].as_u64().unwrap() >= 2, "{cu}");
    assert_eq!(
        cu["routine"]["run_count"],
        json!(1),
        "single Catch Up MUST record exactly 1 run (not 2): {cu}"
    );
    assert_eq!(cu["routine"]["missed_windows"], json!(0), "{cu}");
    assert_eq!(
        cu["routine"]["last_run_verified"],
        json!("verified"),
        "single Catch Up MUST NOT falsely flag Drift: {cu}"
    );
    assert_eq!(
        cu["routine"]["last_run_receipt_sha256"]
            .as_str()
            .unwrap_or("")
            .len(),
        64,
        "{cu}"
    );

    // Persisted GET /api/routines + /history agree on exactly 1 verified run.
    let (st, after) = http_req(addr, "GET", "/api/routines", "");
    assert_eq!(st, 200);
    assert_eq!(after["routines"][0]["run_count"], json!(1), "{after}");
    assert_eq!(after["routines"][0]["history_count"], json!(1), "{after}");
    assert_eq!(
        after["routines"][0]["last_run_verified"],
        json!("verified"),
        "{after}"
    );

    let (st, hist) = http_req(addr, "GET", "/api/routines/rtn-catchup-det/history", "");
    assert_eq!(st, 200);
    assert_eq!(hist["count"], json!(1), "{hist}");
    assert_eq!(hist["runs"][0]["trigger"], json!("catchup"), "{hist}");
    assert_eq!(hist["runs"][0]["status"], json!("ok"), "{hist}");
    assert!(
        hist["runs"][0]["receipt_id"].as_str().is_some_and(|s| !s.is_empty()),
        "{hist}"
    );

    // Output artifact exists on disk with real reconciliation results.
    let out_file = env.cfg.workspace_dir.join("Catchup Ledger Audit.md");
    assert!(out_file.exists(), "expected {out_file:?}");
    let out_text = std::fs::read_to_string(&out_file).unwrap();
    assert!(out_text.contains("Northwind Supply"), "{out_text}");

    // Pause via PATCH preserves `Verified` badge and `run_count == 1`.
    let (st, patched) = http_req(
        addr,
        "PATCH",
        "/api/routines/rtn-catchup-det",
        &json!({ "enabled": false }).to_string(),
    );
    assert_eq!(st, 200, "{patched}");
    assert_eq!(patched["routine"]["run_count"], json!(1), "{patched}");
    assert_eq!(
        patched["routine"]["last_run_verified"],
        json!("verified"),
        "{patched}"
    );

    // Re-enable and confirm second Catch Up is a no-op (`caught_up: false`, `run_count` stays 1).
    let (st, _) = http_req(
        addr,
        "PATCH",
        "/api/routines/rtn-catchup-det",
        &json!({ "enabled": true }).to_string(),
    );
    assert_eq!(st, 200);
    let (st, cu2) = http_req(addr, "POST", "/api/routines/rtn-catchup-det/catchup", "{}");
    assert_eq!(st, 200);
    assert_eq!(cu2["caught_up"], json!(false), "{cu2}");
    assert_eq!(cu2["routine"]["run_count"], json!(1), "{cu2}");

    // Manual `Run now` via POST /api/routines/:id/run increments run_count by EXACTLY 1 (1 -> 2).
    let (st, run_now) = http_req(addr, "POST", "/api/routines/rtn-catchup-det/run", "{}");
    assert_eq!(st, 200, "{run_now}");
    assert_eq!(
        run_now["routine"]["run_count"],
        json!(2),
        "manual Run Now must increment run_count by 1 (1 -> 2): {run_now}"
    );
    assert_eq!(
        run_now["routine"]["last_run_verified"],
        json!("verified"),
        "{run_now}"
    );
}

#[test]
fn llm_requiring_routine_without_model_does_not_crash_scheduler_and_reports_cleanly() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let _env_guard = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", None),
        ("ERGON_SCHEDULER_TICK_MS", Some("250")),
    ]);

    let env = TestDir::new_unconfigured("llm-no-model");
    env.write_ws("daily-invoices.csv", sample_invoices_csv());

    let old_ms = records::now_millis().saturating_sub(35 * 60 * 1000);
    let llm_prompt_rt = json!({
        "id": "rtn-llm-only",
        "name": "Executive Brief (LLM)",
        "prompt": "Draft an executive summary with an LLM",
        "schedule": "every 15m",
        "interval_secs": 900,
        "enabled": true,
        "created_ms": old_ms,
        "last_run_ms": Value::Null,
        "run_count": 0
    });
    let llm_agent_step_rt = json!({
        "id": "rtn-agent-step",
        "name": "Mixed Pipeline (Agent)",
        "prompt": "Run agent step",
        "schedule": "every 15m",
        "interval_secs": 900,
        "enabled": true,
        "created_ms": old_ms,
        "last_run_ms": Value::Null,
        "run_count": 0,
        "steps_v2": [
            {
                "id": "analyze",
                "label": "AI analysis",
                "kind": "agent",
                "prompt": "Analyze workspace invoices",
                "when": "always"
            }
        ]
    });
    std::fs::write(
        routines::routines_path(&env.cfg.data_dir),
        json!({ "routines": [llm_prompt_rt, llm_agent_step_rt] }).to_string(),
    )
    .unwrap();

    let addr = spawn_server(env.cfg.clone());

    // Let multiple background scheduler ticks run -> no crash, no hang, no spam runs.
    std::thread::sleep(Duration::from_millis(700));
    let (st, status) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(status["model_configured"], json!(false));

    let (st, list_before) = http_req(addr, "GET", "/api/routines", "");
    assert_eq!(st, 200);
    for r in list_before["routines"].as_array().unwrap() {
        assert_eq!(
            r["run_count"],
            json!(0),
            "unconfigured LLM routine should not spam background runs: {r}"
        );
    }

    // Explicit Run Now on unconfigured LLM routine reports HTTP 503 cleanly (does not hang).
    let (st, run_res) = http_req(addr, "POST", "/api/routines/rtn-llm-only/run", "{}");
    assert_eq!(st, 503, "expected HTTP 503 for unconfigured LLM routine: {run_res}");
    assert!(
        run_res["error"]
            .as_str()
            .is_some_and(|e| !e.trim().is_empty()),
        "expected clear error message: {run_res}"
    );

    // Explicit Catch Up on unconfigured `agent` steps_v2 routine also returns HTTP 503 cleanly.
    let (st, cu_res) = http_req(addr, "POST", "/api/routines/rtn-agent-step/catchup", "{}");
    assert_eq!(st, 503, "expected HTTP 503 for unconfigured agent routine: {cu_res}");
    assert_eq!(cu_res["caught_up"], json!(false), "{cu_res}");
    assert!(
        cu_res["error"]
            .as_str()
            .is_some_and(|e| !e.trim().is_empty()),
        "{cu_res}"
    );

    // Server remains healthy after explicit 503 reports.
    let (st, ok_after) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(ok_after["model_configured"], json!(false));

    // Bulk POST /api/scheduler/catchup with a mixed set (1 overdue deterministic + 1 overdue LLM):
    // deterministic succeeds with run_count == 1 + Verified, LLM fails cleanly without crashing.
    let env_bulk = TestDir::new_unconfigured("bulk-catchup-mixed");
    env_bulk.write_ws("daily-invoices.csv", sample_invoices_csv());
    let det_bulk = deterministic_ledger_routine(
        "rtn-bulk-det",
        "Bulk Deterministic",
        old_ms,
        "Bulk Deterministic Audit",
    );
    let llm_bulk = json!({
        "id": "rtn-bulk-llm",
        "name": "Bulk LLM",
        "prompt": "Summarize with LLM",
        "schedule": "every 15m",
        "interval_secs": 900,
        "enabled": true,
        "created_ms": old_ms,
        "last_run_ms": Value::Null,
        "run_count": 0
    });
    std::fs::write(
        routines::routines_path(&env_bulk.cfg.data_dir),
        json!({ "routines": [det_bulk, llm_bulk] }).to_string(),
    )
    .unwrap();
    let _bulk_guard = EnvGuard::set(&[("ERGON_DISABLE_SCHEDULER", Some("1"))]);
    let addr_bulk = spawn_server(env_bulk.cfg.clone());
    let (st, bulk_res) = http_req(addr_bulk, "POST", "/api/scheduler/catchup", "{}");
    assert_eq!(st, 200, "POST /api/scheduler/catchup failed: {bulk_res}");
    assert_eq!(bulk_res["caught_up_count"], json!(1), "{bulk_res}");
    assert_eq!(bulk_res["failed_count"], json!(1), "{bulk_res}");
    let det_item = bulk_res["routines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "rtn-bulk-det")
        .unwrap();
    assert_eq!(det_item["run_count"], json!(1), "{det_item}");
    assert_eq!(det_item["last_run_verified"], json!("verified"), "{det_item}");
}
