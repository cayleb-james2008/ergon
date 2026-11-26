//! Cycle 3 Wave E1 Integration Suite (v0.32.0 -> v0.33.0)
//!
//! Covers:
//! 1. Client/Engagement CRUD + active_engagement_by_person switch + 0600 file permissions.
//! 2. Real LLM token/cost/latency metering from mock OpenAI responses + model_pricing overrides.
//! 3. ChainedReceipt legacy + v2 canonical usage_digest verification + tamper detection + audit bundle.
//! 4. GET /api/metering/summary rollup (totals, by_engagement, by_model, daily_series, filters).
//! 5. Retainer hard-cap budget_gate decision hold (pre-run & mid-run) + decision approval lift.
//! 6. Pipeline usage accumulation + SSE usage deltas on step_done/run_finished/pipeline_step_done/pipeline_finished.

use ergon_rs::{audit_bundle, receipts, records, serve, Config};
use serde_json::{json, Value};
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
    fn new(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "clients-metering-v033-{}-{}-{name}",
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
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "mock-pro".to_string(),
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
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(35));
    addr
}

fn drain_http_request(stream: &TcpStream) {
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
    }
}

fn spawn_mock_openai(responses: Vec<Value>) -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_bg = calls.clone();
    let queue = Arc::new(Mutex::new(responses));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            drain_http_request(&stream);
            let idx = calls_bg.fetch_add(1, Ordering::SeqCst);
            let payload = {
                let q = queue.lock().unwrap();
                if q.is_empty() {
                    json!({
                        "model": "mock-pro",
                        "choices": [{"message": {"content": "{\"done\": true, \"summary\": \"Fallback done.\"}"}}],
                        "usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}
                    })
                } else {
                    q[idx.min(q.len() - 1)].clone()
                }
            };
            let body = payload.to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (addr, calls)
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
    let body_bytes = &raw[sep + 4..];
    let val = serde_json::from_slice(body_bytes).unwrap_or(Value::Null);
    (status, val)
}

fn spawn_sse_collector(addr: SocketAddr, stop_event: &'static str) -> std::thread::JoinHandle<Vec<(String, Value)>> {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let req = format!(
        "GET /api/runs/stream HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: keep-alive\r\n\r\n",
        addr.port()
    );
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut reader = BufReader::new(stream);
    // Drain HTTP response headers + initial status frame before returning handle
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }
    std::thread::spawn(move || {
        let mut events = Vec::new();
        let mut cur_event = String::new();
        let deadline = Instant::now() + Duration::from_secs(10);
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

fn poll_run_until(addr: SocketAddr, run_id: &str, pred: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let (st, v) = http_req(addr, "GET", &format!("/api/pipelines/runs/{run_id}"), "");
        assert_eq!(st, 200, "GET run failed: {v}");
        let run = v["run"].clone();
        if pred(&run) {
            return run;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {run_id}: {run}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn clients_engagements_crud_switch_and_permissions() {
    let td = TestDir::new("crud");
    let addr = spawn_server(td.cfg.clone());

    let (st, init) = http_req(addr, "GET", "/api/clients", "");
    assert_eq!(st, 200);
    assert_eq!(init["clients"].as_array().unwrap().len(), 0);
    assert_eq!(init["engagements"].as_array().unwrap().len(), 0);
    assert!(init["active_engagement_id"].is_null());

    // Validation errors
    let (st_bad, _) = http_req(addr, "POST", "/api/clients", r#"{"name": "   "}"#);
    assert_eq!(st_bad, 400);
    let (st_missing_client, _) = http_req(
        addr,
        "POST",
        "/api/engagements",
        r#"{"client_id": "nonexistent", "name": "Q1"}"#,
    );
    assert_eq!(st_missing_client, 404);

    // Create client
    let (st_c1, c1_resp) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({
            "name": "Northwind Advisory",
            "brand_header": "NORTHWIND ADVISORY — CONFIDENTIAL",
            "billing_currency": "usd",
            "default_rate_usd": 240.0
        })
        .to_string(),
    );
    assert_eq!(st_c1, 201, "{c1_resp}");
    let cid = c1_resp["client"]["id"].as_str().unwrap().to_string();
    assert_eq!(cid, "northwind-advisory");
    assert_eq!(c1_resp["client"]["billing_currency"], json!("USD"));
    assert_eq!(c1_resp["client"]["default_rate_usd"], json!(240.0));

    // clients.json mode must be 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(records::clients_path(&td.cfg.data_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "clients.json mode was {mode:o}");
    }

    // Create two engagements (e1 inherits default_rate_usd, e2 overrides)
    let (st_e1, e1_resp) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Q4 Ops Retainer",
            "retainer_hours": 20.0,
            "retainer_budget_usd": 100.0,
            "period": "monthly"
        })
        .to_string(),
    );
    assert_eq!(st_e1, 201, "{e1_resp}");
    let e1_id = e1_resp["engagement"]["id"].as_str().unwrap().to_string();
    assert_eq!(e1_resp["engagement"]["hourly_rate_usd"], json!(240.0));

    let (st_e2, e2_resp) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Audit Sprint",
            "retainer_hours": 10.0,
            "retainer_budget_usd": 50.0,
            "hourly_rate_usd": 300.0,
            "period": "fixed"
        })
        .to_string(),
    );
    assert_eq!(st_e2, 201, "{e2_resp}");
    let e2_id = e2_resp["engagement"]["id"].as_str().unwrap().to_string();
    assert_eq!(e2_resp["engagement"]["hourly_rate_usd"], json!(300.0));

    // Switch unknown -> 404
    let (st_sw_404, _) = http_req(
        addr,
        "POST",
        "/api/engagements/switch",
        r#"{"engagement_id": "missing-eng"}"#,
    );
    assert_eq!(st_sw_404, 404);

    // Switch to e1
    let (st_sw1, sw1) = http_req(
        addr,
        "POST",
        "/api/engagements/switch",
        &json!({ "engagement_id": e1_id }).to_string(),
    );
    assert_eq!(st_sw1, 200, "{sw1}");
    assert_eq!(sw1["active_engagement_id"], json!(e1_id));
    assert_eq!(sw1["engagement"]["name"], json!("Q4 Ops Retainer"));

    let (st_get2, list2) = http_req(addr, "GET", "/api/clients", "");
    assert_eq!(st_get2, 200);
    assert_eq!(list2["active_engagement_id"], json!(e1_id));
    assert_eq!(list2["clients"].as_array().unwrap().len(), 1);
    assert_eq!(list2["engagements"].as_array().unwrap().len(), 2);

    // Switch to e2, then clear with null
    let (st_sw2, sw2) = http_req(
        addr,
        "POST",
        "/api/engagements/switch",
        &json!({ "engagement_id": e2_id }).to_string(),
    );
    assert_eq!(st_sw2, 200);
    assert_eq!(sw2["active_engagement_id"], json!(e2_id));

    let (st_clr, clr) = http_req(
        addr,
        "POST",
        "/api/engagements/switch",
        r#"{"engagement_id": null}"#,
    );
    assert_eq!(st_clr, 200, "{clr}");
    assert!(clr["active_engagement_id"].is_null());
}

#[test]
fn mock_openai_usage_capture_pricing_sse_and_work_persistence() {
    let (mock_addr, mock_calls) = spawn_mock_openai(vec![
        json!({
            "model": "mock-pro",
            "choices": [{
                "message": {
                    "content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"memo.md\", \"content\": \"Client Memo\\n\"}}"
                }
            }],
            "usage": {
                "prompt_tokens": 1000,
                "completion_tokens": 500,
                "total_tokens": 1500,
                "prompt_tokens_details": { "cached_tokens": 200 }
            }
        }),
        json!({
            "model": "mock-pro",
            "choices": [{
                "message": {
                    "content": "{\"done\": true, \"summary\": \"Wrote memo.md for the client.\"}"
                }
            }],
            "usage": {
                "prompt_tokens": 1000,
                "completion_tokens": 500,
                "total_tokens": 1500,
                "cached_tokens": 200
            }
        }),
    ]);

    let mut td = TestDir::new("usage-capture");
    td.cfg.model_base_url = format!("http://127.0.0.1:{}/v1", mock_addr.port());
    td.cfg.model_id = "mock-pro".to_string();
    let addr = spawn_server(td.cfg.clone());

    // Configure model_pricing overrides via PUT /api/config
    let (st_cfg, cfg_resp) = http_req(
        addr,
        "PUT",
        "/api/config",
        &json!({
            "model_pricing": {
                "mock-pro": {
                    "input_per_1m_usd": 10.0,
                    "output_per_1m_usd": 30.0,
                    "cached_input_per_1m_usd": 2.0
                }
            }
        })
        .to_string(),
    );
    assert_eq!(st_cfg, 200, "{cfg_resp}");
    assert_eq!(
        cfg_resp["model_pricing"]["mock-pro"]["input_per_1m_usd"],
        json!(10.0)
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(records::model_pricing_path(&td.cfg.data_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    // Create client + engagement + activate
    let (_, c_resp) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({"name": "Acme Corp", "default_rate_usd": 120.0}).to_string(),
    );
    let cid = c_resp["client"]["id"].as_str().unwrap();
    let (_, e_resp) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Acme Advisory",
            "retainer_hours": 10.0,
            "retainer_budget_usd": 5.0,
            "activate": true
        })
        .to_string(),
    );
    let eid = e_resp["engagement"]["id"].as_str().unwrap().to_string();

    // Collect SSE events during POST /api/chat
    let sse_handle = spawn_sse_collector(addr, "run_finished");
    let (st_chat, chat_resp) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({"message": "Write memo.md"}).to_string(),
    );
    assert_eq!(st_chat, 200, "{chat_resp}");
    assert_eq!(mock_calls.load(Ordering::SeqCst), 2);

    // Per call: (800*10 + 200*2 + 500*30)/1e6 = (8000+400+15000)/1e6 = 0.0234 -> 2 calls = 0.0468
    let usage = &chat_resp["usage"];
    assert_eq!(chat_resp["engagement_id"], json!(eid));
    assert_eq!(usage["model_id"], json!("mock-pro"));
    assert_eq!(usage["calls"], json!(2));
    assert_eq!(usage["prompt_tokens"], json!(2000));
    assert_eq!(usage["completion_tokens"], json!(1000));
    assert_eq!(usage["total_tokens"], json!(3000));
    assert_eq!(usage["cached_tokens"], json!(400));
    assert!((usage["cost_usd"].as_f64().unwrap() - 0.0468).abs() < 1e-7, "{usage}");
    assert!(usage["estimated_minutes_saved"].as_f64().unwrap() > 0.0, "{usage}");
    assert!(usage["billable_value_usd"].as_f64().unwrap() > 0.0, "{usage}");

    // Verify work.json persisted engagement_id + usage
    let work = records::load_work(&td.cfg.data_dir);
    assert_eq!(work.len(), 1);
    assert_eq!(work[0]["engagement_id"], json!(eid));
    assert_eq!(work[0]["usage"]["total_tokens"], json!(3000));

    // Verify usage_ledger.jsonl persisted entry
    let ledger = records::load_usage_ledger(&td.cfg.data_dir);
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].engagement_id.as_deref(), Some(eid.as_str()));
    assert!((ledger[0].usage.cost_usd - 0.0468).abs() < 1e-7);

    // Verify SSE step_done + run_finished emitted usage deltas
    let events = sse_handle.join().unwrap();
    let step_done = events
        .iter()
        .find(|(ev, _)| ev == "step_done")
        .expect("step_done SSE event");
    assert_eq!(step_done.1["usage"]["total_tokens"], json!(1500));
    assert!((step_done.1["usage"]["cost_usd"].as_f64().unwrap() - 0.0234).abs() < 1e-7);
    let run_finished = events
        .iter()
        .find(|(ev, _)| ev == "run_finished")
        .expect("run_finished SSE event");
    assert_eq!(run_finished.1["engagement_id"], json!(eid));
    assert_eq!(run_finished.1["usage"]["total_tokens"], json!(3000));
    assert!((run_finished.1["usage"]["cost_usd"].as_f64().unwrap() - 0.0468).abs() < 1e-7);
}

#[test]
fn chained_receipt_legacy_and_v2_hash_verification_and_tamper_detection() {
    let td = TestDir::new("receipts-v2");
    let addr = spawn_server(td.cfg.clone());

    // 1. Legacy entry (no engagement_id, no usage)
    records::append_work(
        &td.cfg,
        "Legacy task",
        "Legacy summary",
        &["old.txt".to_string()],
        "t-legacy",
        true,
    );
    // 2. v2 entry (with engagement_id + usage)
    let mut v2_usage = records::RunUsage {
        model_id: "gpt-4o".to_string(),
        calls: 1,
        prompt_tokens: 400,
        completion_tokens: 100,
        total_tokens: 500,
        cached_tokens: 50,
        latency_ms: 42,
        cost_usd: 0.002,
        estimated_minutes_saved: 15.4,
        billable_value_usd: 38.5,
    };
    v2_usage.finalize_attribution(true, false, 1, 1, 150.0);
    records::append_work_with_usage(
        &td.cfg,
        "Client deliverable",
        "Produced report.md",
        &["report.md".to_string()],
        "t-v2",
        true,
        Some("eng-northwind"),
        Some(&v2_usage),
    );

    let chain = receipts::load_chain(&td.cfg.data_dir);
    assert_eq!(chain.len(), 2);
    assert!(chain[0].engagement_id.is_none());
    assert!(chain[0].usage.is_none());
    assert!(chain[0].usage_digest().is_none());
    assert_eq!(chain[1].engagement_id.as_deref(), Some("eng-northwind"));
    assert!(chain[1].usage.is_some());
    assert!(chain[1].usage_digest().is_some());

    // Verify legacy hash matches exact legacy formula: sha256("000..|0|ts|t-legacy|1|0|Legacy task|Legacy summary|old.txt")
    let legacy_preimage = format!(
        "{}|0|{}|t-legacy|1|0|Legacy task|Legacy summary|old.txt",
        receipts::GENESIS_HASH,
        chain[0].ts
    );
    assert_eq!(
        chain[0].sha256,
        receipts::sha256_hex(legacy_preimage.as_bytes()),
        "legacy receipt hash must stay 100% backward-compatible"
    );

    let (valid, head) = receipts::verify_chain(&chain);
    assert!(valid);
    assert_eq!(head, chain[1].sha256);

    // Tampering with v2 usage must fail verify_chain!
    let mut tampered = chain.clone();
    if let Some(Value::Object(ref mut map)) = tampered[1].usage {
        map.insert("cost_usd".to_string(), json!(999.0));
    }
    let (tamper_ok, _) = receipts::verify_chain(&tampered);
    assert!(!tamper_ok, "tampered v2 usage must fail verify_chain");

    // Tampering with v2 engagement_id must fail verify_chain!
    let mut tampered_eid = chain.clone();
    tampered_eid[1].engagement_id = Some("eng-evil".to_string());
    let (tamper_eid_ok, _) = receipts::verify_chain(&tampered_eid);
    assert!(!tamper_eid_ok, "tampered v2 engagement_id must fail verify_chain");

    // Verify HTTP /api/receipts/verify & /api/audit/bundle pass on mixed legacy+v2 chain
    let (st_v, v_resp) = http_req(addr, "GET", "/api/receipts/verify", "");
    assert_eq!(st_v, 200);
    assert_eq!(v_resp["valid"], json!(true));
    assert_eq!(v_resp["total"], json!(2));

    let (st_b, bundle_json) = http_req(addr, "GET", "/api/audit/bundle", "");
    assert_eq!(st_b, 200);
    let audit_res = audit_bundle::verify_audit_bundle(&bundle_json);
    assert!(
        audit_res.valid,
        "mixed legacy+v2 audit bundle verification failed: {:?}",
        audit_res.errors
    );
}

#[test]
fn metering_summary_rollup_and_filters() {
    let td = TestDir::new("metering-summary");
    let addr = spawn_server(td.cfg.clone());

    let (_, c1) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({"name": "Alpha", "default_rate_usd": 200.0}).to_string(),
    );
    let c1_id = c1["client"]["id"].as_str().unwrap().to_string();
    let (_, c2) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({"name": "Beta", "default_rate_usd": 100.0}).to_string(),
    );
    let c2_id = c2["client"]["id"].as_str().unwrap().to_string();

    let (_, e1) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({"client_id": c1_id, "name": "Alpha Monthly", "retainer_budget_usd": 10.0, "activate": true}).to_string(),
    );
    let e1_id = e1["engagement"]["id"].as_str().unwrap().to_string();
    let (_, e2) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({"client_id": c2_id, "name": "Beta Sprint", "retainer_budget_usd": 5.0}).to_string(),
    );
    let e2_id = e2["engagement"]["id"].as_str().unwrap().to_string();

    let now_ms = records::now_millis();
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: now_ms,
            run_id: "r1".into(),
            thread_id: "t1".into(),
            person_id: "default".into(),
            engagement_id: Some(e1_id.clone()),
            client_id: Some(c1_id.clone()),
            source: "agent".into(),
            finished: true,
            answered: false,
            usage: records::RunUsage {
                model_id: "gpt-4o".into(),
                calls: 2,
                prompt_tokens: 1000,
                completion_tokens: 500,
                total_tokens: 1500,
                cached_tokens: 100,
                latency_ms: 80,
                cost_usd: 2.5,
                estimated_minutes_saved: 30.0,
                billable_value_usd: 100.0,
            },
        },
    );
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: now_ms,
            run_id: "r2".into(),
            thread_id: "t2".into(),
            person_id: "default".into(),
            engagement_id: Some(e2_id.clone()),
            client_id: Some(c2_id.clone()),
            source: "agent".into(),
            finished: true,
            answered: false,
            usage: records::RunUsage {
                model_id: "deepseek-v4.1-flash".into(),
                calls: 1,
                prompt_tokens: 400,
                completion_tokens: 200,
                total_tokens: 600,
                cached_tokens: 0,
                latency_ms: 30,
                cost_usd: 0.5,
                estimated_minutes_saved: 15.0,
                billable_value_usd: 25.0,
            },
        },
    );
    // Out-of-window entry (60 days ago) should be excluded for window_days=30
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: now_ms.saturating_sub(60 * 86_400_000),
            run_id: "r-old".into(),
            thread_id: "t-old".into(),
            person_id: "default".into(),
            engagement_id: Some(e1_id.clone()),
            client_id: Some(c1_id.clone()),
            source: "agent".into(),
            finished: true,
            answered: false,
            usage: records::RunUsage {
                model_id: "gpt-4o".into(),
                calls: 1,
                prompt_tokens: 9999,
                completion_tokens: 1,
                total_tokens: 10000,
                cached_tokens: 0,
                latency_ms: 10,
                cost_usd: 9.0,
                estimated_minutes_saved: 10.0,
                billable_value_usd: 30.0,
            },
        },
    );

    let (st_all, all_sum) = http_req(addr, "GET", "/api/metering/summary?window_days=30", "");
    assert_eq!(st_all, 200, "{all_sum}");
    assert_eq!(all_sum["totals"]["runs"], json!(2));
    assert_eq!(all_sum["totals"]["prompt_tokens"], json!(1400));
    assert_eq!(all_sum["totals"]["completion_tokens"], json!(700));
    assert_eq!(all_sum["totals"]["total_tokens"], json!(2100));
    assert!((all_sum["totals"]["cost_usd"].as_f64().unwrap() - 3.0).abs() < 1e-6);
    assert!((all_sum["totals"]["minutes_saved"].as_f64().unwrap() - 45.0).abs() < 1e-6);
    assert!((all_sum["totals"]["billable_value_usd"].as_f64().unwrap() - 125.0).abs() < 1e-6);
    // Active engagement is e1 ($2.5 / $10.0 = 25.0%)
    assert!((all_sum["totals"]["retainer_burn_pct"].as_f64().unwrap() - 25.0).abs() < 1e-6);
    assert_eq!(all_sum["by_engagement"].as_array().unwrap().len(), 2);
    assert_eq!(all_sum["by_model"].as_array().unwrap().len(), 2);
    assert_eq!(all_sum["daily_series"].as_array().unwrap().len(), 1);

    // Filter by engagement_id=e2_id
    let (st_e2, e2_sum) = http_req(
        addr,
        "GET",
        &format!("/api/metering/summary?engagement_id={e2_id}&window_days=30"),
        "",
    );
    assert_eq!(st_e2, 200);
    assert_eq!(e2_sum["totals"]["runs"], json!(1));
    assert_eq!(e2_sum["totals"]["total_tokens"], json!(600));
    assert!((e2_sum["totals"]["retainer_burn_pct"].as_f64().unwrap() - 10.0).abs() < 1e-6);

    // Filter by client_id=c1_id
    let (st_c1, c1_sum) = http_req(
        addr,
        "GET",
        &format!("/api/metering/summary?client_id={c1_id}&window_days=30"),
        "",
    );
    assert_eq!(st_c1, 200);
    assert_eq!(c1_sum["totals"]["runs"], json!(1));
    assert_eq!(c1_sum["totals"]["total_tokens"], json!(1500));
}

#[test]
fn retainer_budget_gate_parks_decision_and_stops_spend() {
    let (mock_addr, mock_calls) = spawn_mock_openai(vec![
        // Call 1 costs 0.0234 (under 0.03? Wait: with cap 0.02, call 1 exceeds cap and stops BEFORE call 2!)
        json!({
            "model": "mock-pro",
            "choices": [{
                "message": {
                    "content": "{\"tool\": \"write_file\", \"args\": {\"path\": \"step1.txt\", \"content\": \"ok\"}}"
                }
            }],
            "usage": {
                "prompt_tokens": 1000,
                "completion_tokens": 500,
                "total_tokens": 1500
            }
        }),
        json!({
            "model": "mock-pro",
            "choices": [{
                "message": {
                    "content": "{\"done\": true, \"summary\": \"Should not run before approval!\"}"
                }
            }],
            "usage": {
                "prompt_tokens": 1000,
                "completion_tokens": 500,
                "total_tokens": 1500
            }
        }),
    ]);

    let mut td = TestDir::new("budget-gate");
    td.cfg.model_base_url = format!("http://127.0.0.1:{}/v1", mock_addr.port());
    td.cfg.model_id = "mock-pro".to_string();
    let addr = spawn_server(td.cfg.clone());

    // Set pricing: $10/1M in, $30/1M out -> 1000 in + 500 out = $0.025 per call
    let (st_cfg, _) = http_req(
        addr,
        "PUT",
        "/api/config",
        &json!({
            "model_pricing": {
                "mock-pro": {
                    "input_per_1m_usd": 10.0,
                    "output_per_1m_usd": 30.0,
                    "cached_input_per_1m_usd": 2.0
                }
            }
        })
        .to_string(),
    );
    assert_eq!(st_cfg, 200);

    let (_, c_resp) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({"name": "Capped Client", "default_rate_usd": 100.0}).to_string(),
    );
    let cid = c_resp["client"]["id"].as_str().unwrap();
    // Hard cap of $0.02: Call 1 ($0.025) breaches it, so Call 2 MUST be blocked by budget_gate!
    let (_, e_resp) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Tight Retainer",
            "retainer_budget_usd": 0.02,
            "activate": true
        })
        .to_string(),
    );
    assert_eq!(e_resp["engagement"]["retainer_budget_usd"], json!(0.02));

    let (st1, r1) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({"message": "Write step1.txt and finish"}).to_string(),
    );
    assert_eq!(st1, 200, "{r1}");
    assert_eq!(r1["budget_gated"], json!(true));
    assert_eq!(mock_calls.load(Ordering::SeqCst), 1, "Call 2 must NOT have fired");
    let did = r1["decision_id"].as_str().unwrap().to_string();

    // Verify decisions inbox has pending kind="budget_gate"
    let (st_d, d_list) = http_req(addr, "GET", "/api/decisions", "");
    assert_eq!(st_d, 200);
    let dec = d_list["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == json!(did))
        .expect("budget_gate decision in inbox");
    assert_eq!(dec["kind"], json!("budget_gate"));
    assert_eq!(dec["status"], json!("pending"));

    // Second run while gate is pending must burn ZERO model calls!
    let (st2, r2) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({"message": "Try again without approval"}).to_string(),
    );
    assert_eq!(st2, 200, "{r2}");
    assert_eq!(r2["budget_gated"], json!(true));
    assert_eq!(
        mock_calls.load(Ordering::SeqCst),
        1,
        "Over-budget run must burn ZERO model calls"
    );

    // Approve the budget_gate decision -> next run proceeds cleanly!
    let (st_app, app_resp) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{did}/approve"),
        "{}",
    );
    assert_eq!(st_app, 200, "{app_resp}");
    assert_eq!(app_resp["decision"]["status"], json!("approved"));

    let (st3, r3) = http_req(
        addr,
        "POST",
        "/api/chat",
        &json!({"message": "Now proceed"}).to_string(),
    );
    assert_eq!(st3, 200, "{r3}");
    assert!(r3["budget_gated"].is_null());
    assert_eq!(mock_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn pipeline_accumulates_step_and_run_usage_and_emits_sse_deltas() {
    let td = TestDir::new("pipeline-usage");
    let addr = spawn_server(td.cfg.clone());

    let (_, c_resp) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({
            "name": "Pipeline Client",
            "brand_header": "PIPELINE CLIENT HEADER",
            "default_rate_usd": 180.0
        })
        .to_string(),
    );
    let cid = c_resp["client"]["id"].as_str().unwrap();
    let (_, e_resp) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Pipeline Retainer",
            "retainer_hours": 15.0,
            "retainer_budget_usd": 50.0,
            "activate": true
        })
        .to_string(),
    );
    let eid = e_resp["engagement"]["id"].as_str().unwrap().to_string();

    let sse_handle = spawn_sse_collector(addr, "pipeline_finished");
    let (st_run, start_resp) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Client Brief Pipeline",
            "steps_v2": [
                {
                    "id": "doc",
                    "label": "Write Summary Doc",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": { "title": "ClientUpdate", "sections": ["Delivered Q4 metrics."] }
                },
                {
                    "id": "pdf",
                    "label": "Export PDF",
                    "kind": "export",
                    "export": { "path": "{{steps.doc.files[0]}}", "format": "pdf" }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_run, 202, "{start_resp}");
    let run_id = start_resp["run"]["run_id"].as_str().unwrap().to_string();
    assert_eq!(start_resp["run"]["engagement_id"], json!(eid));

    let finished = poll_run_until(addr, &run_id, |r| r["status"] == "completed");
    assert_eq!(finished["engagement_id"], json!(eid));
    assert!(finished["usage"]["estimated_minutes_saved"].as_f64().unwrap() > 0.0);
    assert!(finished["usage"]["billable_value_usd"].as_f64().unwrap() > 0.0);
    assert!(finished["step_states"][0]["usage"]["estimated_minutes_saved"].as_f64().unwrap() > 0.0);
    assert!(finished["step_states"][1]["usage"]["estimated_minutes_saved"].as_f64().unwrap() > 0.0);

    let events = sse_handle.join().unwrap();
    let pipe_step_events: Vec<_> = events
        .iter()
        .filter(|(ev, _)| ev == "pipeline_step_done")
        .collect();
    assert_eq!(pipe_step_events.len(), 2, "{events:?}");
    for (_, ev) in &pipe_step_events {
        assert_eq!(ev["engagement_id"], json!(eid));
        assert!(ev["usage"]["estimated_minutes_saved"].as_f64().unwrap() > 0.0);
        assert!(ev["run_usage"]["estimated_minutes_saved"].as_f64().unwrap() > 0.0);
    }
    let pipe_done = events
        .iter()
        .find(|(ev, _)| ev == "pipeline_finished")
        .expect("pipeline_finished SSE event");
    assert_eq!(pipe_done.1["engagement_id"], json!(eid));
    assert!(pipe_done.1["usage"]["billable_value_usd"].as_f64().unwrap() > 0.0);

    // Verify /api/metering/summary picked up the pipeline steps
    let (st_sum, sum_resp) = http_req(
        addr,
        "GET",
        &format!("/api/metering/summary?engagement_id={eid}"),
        "",
    );
    assert_eq!(st_sum, 200);
    assert_eq!(sum_resp["totals"]["runs"], json!(2));
    assert!(sum_resp["totals"]["minutes_saved"].as_f64().unwrap() > 0.0);
    assert!(sum_resp["totals"]["billable_value_usd"].as_f64().unwrap() > 0.0);
}
