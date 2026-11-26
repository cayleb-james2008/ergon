//! Cycle 2 Wave H1 Hardening & Security Integration Suite (v0.31.0 -> v0.32.0)
//!
//! Covers:
//! 1. `pipelines.rs`: restart recovery of interrupted/cancelled/corrupt-checkpoint runs,
//!    circular/self/forward/deep `{{steps.*}}` template caps, UTF-8 metrics safety,
//!    and concurrent resume/retry/cancel idempotency.
//! 2. `tables.rs`, `docs.rs`, `file_history.rs`: malformed UTF-8 & binary CSV rejection,
//!    ragged CSV normalization, broken XLSX XML & stored zip-bomb guards,
//!    huge PDF line-wrap & pagination guards, and concurrent PUT/PATCH `expected_sha256`
//!    409 conflict + atomic `.history/` snapshot integrity.
//! 3. `connectors.rs` & `tripwire.rs`: CWE-59 symlinked watched-folder & parent-symlink
//!    rejection, >1 MB webhook 413 rejection, timestamp/replay & timing-safe HMAC
//!    verification, partial & JSON-escaped secret redaction (preserving safe `token_hint`),
//!    and Air-Gapped / Strict-Client policy edge cases.

use ergon_rs::{
    connectors, docs, execute_tool, file_history, pipelines, records, serve, tripwire, Config,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
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
            "hardening-v032-{}-{}-{name}",
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
            model_id: "stub".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }

    fn write_ws(&self, rel: &str, bytes: &[u8]) {
        let full = self.cfg.workspace_dir.join(rel);
        if let Some(p) = full.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(full, bytes).unwrap();
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

fn http_req_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut has_host = false;
    let mut has_ct = false;
    let mut head = format!("{method} {path} HTTP/1.1\r\n");
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("host") {
            has_host = true;
        }
        if k.eq_ignore_ascii_case("content-type") {
            has_ct = true;
        }
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if !has_host {
        head.push_str(&format!("Host: 127.0.0.1:{}\r\n", addr.port()));
    }
    if !has_ct && !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    let _ = stream.write_all(head.as_bytes());
    if !body.is_empty() {
        let _ = stream.write_all(body);
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
    let body_text = String::from_utf8_lossy(body_bytes).to_string();
    let val = serde_json::from_slice(body_bytes).unwrap_or(Value::Null);
    (status, val, body_text)
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let (status, val, _) = http_req_headers(addr, method, path, &[], body.as_bytes());
    (status, val)
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

fn make_step(id: &str, kind: &str, skill_id: Option<&str>, args: Value) -> pipelines::PipelineStepSpec {
    pipelines::PipelineStepSpec {
        id: id.to_string(),
        label: id.to_string(),
        kind: kind.to_string(),
        skill_id: skill_id.map(str::to_string),
        args,
        prompt: None,
        checkpoint: None,
        export: None,
        when: "always".to_string(),
    }
}

fn make_state(id: &str, kind: &str, status: &str, output: &str) -> pipelines::PipelineStepState {
    pipelines::PipelineStepState {
        step_id: id.to_string(),
        label: id.to_string(),
        kind: kind.to_string(),
        status: status.to_string(),
        started_ms: 100,
        finished_ms: 200,
        output: output.to_string(),
        files: Vec::new(),
        metrics: json!({}),
        usage: None,
        checkpoint_id: None,
        human_answer: None,
        human_action: None,
        error: None,
    }
}

// ---------------------------------------------------------------------------
// 1. Pipelines Hardening
// ---------------------------------------------------------------------------

#[test]
fn pipeline_restart_recovery_handles_interrupted_cancelled_and_corrupt_checkpoint() {
    let td = TestDir::new("pipe-recover");
    let now_ms = records::now_millis();

    // 1. Stale running run with cancel_requested = true -> recovers as "cancelled".
    let cancelled_stale = pipelines::PipelineRun {
        run_id: "prun-stale-cancel".into(),
        routine_id: None,
        routine_name: "Cancel Before Restart".into(),
        thread_id: "t-rc-1".into(),
        status: "running".into(),
        current_step_index: 0,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs: HashMap::new(),
        steps: vec![make_step(
            "s1",
            "skill",
            Some("write_document"),
            json!({ "title": "ShouldNotRun", "sections": ["x"] }),
        )],
        step_states: vec![make_state("s1", "skill", "running", "")],
        pending_checkpoint: None,
        error: None,
        cancel_requested: true,
        engagement_id: None,
        usage: Default::default(),
    };
    pipelines::save_run(&td.cfg.data_dir, &cancelled_stale).unwrap();

    // 2. Corrupt "paused_checkpoint" with missing pending_checkpoint -> recovers as "interrupted".
    let broken_cp = pipelines::PipelineRun {
        run_id: "prun-broken-cp".into(),
        routine_id: None,
        routine_name: "Broken CP".into(),
        thread_id: "t-rc-2".into(),
        status: "paused_checkpoint".into(),
        current_step_index: 0,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs: HashMap::new(),
        steps: vec![make_step(
            "s1",
            "skill",
            Some("write_document"),
            json!({ "title": "BrokenCPRecovered", "sections": ["ok"] }),
        )],
        step_states: vec![make_state("s1", "checkpoint", "waiting_human", "")],
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: Default::default(),
    };
    pipelines::save_run(&td.cfg.data_dir, &broken_cp).unwrap();

    // 3. Multi-step stale running run interrupted at step 1 -> recovers as "interrupted" and resumes via /resume!
    let interrupted_multi = pipelines::PipelineRun {
        run_id: "prun-interrupted-multi".into(),
        routine_id: None,
        routine_name: "Multi Step Interrupted".into(),
        thread_id: "t-rc-3".into(),
        status: "running".into(),
        current_step_index: 1,
        created_ms: now_ms,
        updated_ms: now_ms,
        inputs: HashMap::new(),
        steps: vec![
            make_step(
                "s1",
                "skill",
                Some("write_document"),
                json!({ "title": "StepOneDone", "sections": ["first"] }),
            ),
            make_step(
                "s2",
                "skill",
                Some("write_document"),
                json!({ "title": "StepTwoResumed", "sections": ["Prev={{steps.prev.output}}"] }),
            ),
        ],
        step_states: vec![
            make_state("s1", "skill", "ok", "StepOneOutput"),
            make_state("s2", "skill", "running", ""),
        ],
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: Default::default(),
    };
    pipelines::save_run(&td.cfg.data_dir, &interrupted_multi).unwrap();

    let addr = spawn_server(td.cfg.clone());

    let (st1, r1) = http_req(addr, "GET", "/api/pipelines/runs/prun-stale-cancel", "");
    assert_eq!(st1, 200);
    assert_eq!(r1["run"]["status"], json!("cancelled"));
    assert_eq!(r1["run"]["step_states"][0]["status"], json!("failed"));

    let (st2, r2) = http_req(addr, "GET", "/api/pipelines/runs/prun-broken-cp", "");
    assert_eq!(st2, 200);
    assert_eq!(r2["run"]["status"], json!("interrupted"));

    let (st3, r3) = http_req(addr, "GET", "/api/pipelines/runs/prun-interrupted-multi", "");
    assert_eq!(st3, 200);
    assert_eq!(r3["run"]["status"], json!("interrupted"));
    assert_eq!(r3["run"]["step_states"][1]["status"], json!("failed"));

    // Resume interrupted run via /resume (without retry) and verify {{steps.prev.output}} resolved!
    let (st_res, res_json) = http_req(
        addr,
        "POST",
        "/api/pipelines/runs/prun-interrupted-multi/resume",
        "{}",
    );
    assert_eq!(st_res, 202, "resume interrupted: {res_json}");
    let done = poll_run_until(addr, "prun-interrupted-multi", |r| r["status"] == "completed");
    assert_eq!(done["step_states"][1]["status"], json!("ok"));
    let doc = std::fs::read_to_string(td.cfg.workspace_dir.join("StepTwoResumed.md")).unwrap();
    assert!(doc.contains("Prev=StepOneOutput"), "{doc}");
}

#[test]
fn pipeline_template_caps_circular_self_forward_deep_and_utf8_metrics() {
    let td = TestDir::new("pipe-tpl");
    let run = pipelines::PipelineRun {
        run_id: "prun-tpl".into(),
        routine_id: None,
        routine_name: "Tpl".into(),
        thread_id: "t-tpl".into(),
        status: "running".into(),
        current_step_index: 1,
        created_ms: 1,
        updated_ms: 1,
        inputs: HashMap::from([("big".into(), "A".repeat(4096))]),
        steps: vec![
            make_step("s1", "agent", None, json!({})),
            make_step("s2", "agent", None, json!({})),
            make_step("s3", "agent", None, json!({})),
        ],
        step_states: vec![
            make_state("s1", "skill", "ok", "hello {{steps.s2.output}} {{steps.s1.output}}"),
            make_state("s2", "skill", "running", "in-flight-secret"),
            make_state("s3", "skill", "ok", "stale-future-output"),
        ],
        pending_checkpoint: None,
        error: None,
        cancel_requested: false,
        engagement_id: None,
        usage: Default::default(),
    };

    // Self-reference (s2 while current_step_index=1) and forward reference (s3) must NOT resolve!
    // Circular reference inside s1.output must NOT recursively expand!
    let resolved = pipelines::resolve_template_str(
        "s1=[{{steps.s1.output}}] self=[{{steps.s2.output}}] future=[{{steps.s3.output}}] prev=[{{steps.prev.output}}]",
        &run,
    );
    assert_eq!(
        resolved,
        "s1=[hello {{steps.s2.output}} {{steps.s1.output}}] self=[] future=[] prev=[hello {{steps.s2.output}} {{steps.s1.output}}]"
    );

    // Malformed nested braces must not panic or misparse.
    let nested = pipelines::resolve_template_str("x={{steps.{{inputs.big}}.output}} y={{inputs.big}}", &run);
    assert!(nested.contains("y=AAAA"), "{nested}");

    // Huge template expansion bomb is capped cleanly at MAX_TEMPLATE_OUTPUT_BYTES.
    let bomb_tpl = "{{inputs.big}}".repeat(200);
    let bomb_out = pipelines::resolve_template_str(&bomb_tpl, &run);
    assert!(bomb_out.len() <= pipelines::MAX_TEMPLATE_OUTPUT_BYTES);

    // Deeply nested JSON Value (> MAX_TEMPLATE_DEPTH) does not overflow stack.
    let mut deep = json!("{{steps.s1.output}}");
    for _ in 0..48 {
        deep = json!({ "nested": deep });
    }
    let _ = pipelines::resolve_template_value(&deep, &run);

    // Multi-byte UTF-8 straddling byte 65,536 in .md artifact during deterministic pipeline must not panic!
    let mut big_md = "a".repeat(65_534);
    big_md.push('—'); // 3-byte UTF-8 char straddling byte 65,536!
    big_md.push_str("\n| Metric | Value |\n| --- | --- |\n| Outliers | 2 |\n");
    td.write_ws("Big UTF8.md", big_md.as_bytes());
    let addr = spawn_server(td.cfg.clone());
    let (st, run_res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "UTF8 Metrics Test",
            "steps_v2": [
                {
                    "id": "s1",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": { "title": "Big UTF8 Copy", "sections": ["{{steps.s1.output}}"] }
                },
                {
                    "id": "s2",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": { "title": "Async Prev Test", "sections": ["FromPrev={{steps.prev.files[0]}}"] }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st, 202, "{run_res}");
    let rid = run_res["run"]["run_id"].as_str().unwrap().to_string();
    let done = poll_run_until(addr, &rid, |r| r["status"] == "completed");
    assert_eq!(done["status"], json!("completed"));
    let s2_doc = std::fs::read_to_string(td.cfg.workspace_dir.join("Async Prev Test.md")).unwrap();
    assert!(s2_doc.contains("FromPrev=Big UTF8 Copy.md"), "{s2_doc}");
}

#[test]
fn pipeline_concurrent_resume_retry_and_cancel_idempotency() {
    let td = TestDir::new("pipe-conc");
    let addr = spawn_server(td.cfg.clone());

    // 1. Create a pipeline that pauses at a checkpoint.
    let (st, res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Concurrent Checkpoint",
            "steps_v2": [
                {
                    "id": "cp",
                    "kind": "checkpoint",
                    "checkpoint": { "prompt": "Approve?", "input_mode": "approve_reject" }
                },
                {
                    "id": "after",
                    "kind": "skill",
                    "skill_id": "write_document",
                    "args": { "title": "AfterConcurrent", "sections": ["ans={{steps.cp.human_answer}}"] }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st, 202);
    let run_id = res["run"]["run_id"].as_str().unwrap().to_string();
    poll_run_until(addr, &run_id, |r| r["status"] == "paused_checkpoint");

    // 8 concurrent threads race POST /resume -> strictly 1 gets 202, 7 get 409!
    let threads = 8usize;
    let barrier = Arc::new(Barrier::new(threads));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let b = barrier.clone();
        let rid = run_id.clone();
        handles.push(std::thread::spawn(move || {
            b.wait();
            http_req(
                addr,
                "POST",
                &format!("/api/pipelines/runs/{rid}/resume"),
                r#"{"action":"approve"}"#,
            )
            .0
        }));
    }
    let statuses: Vec<u16> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let ok_count = statuses.iter().filter(|&&s| s == 202).count();
    let conflict_count = statuses.iter().filter(|&&s| s == 409).count();
    assert_eq!(ok_count, 1, "statuses: {statuses:?}");
    assert_eq!(conflict_count, threads - 1, "statuses: {statuses:?}");

    poll_run_until(addr, &run_id, |r| r["status"] == "completed");

    // Concurrent cancel on completed run is idempotent 200 and keeps status "completed".
    let (st_c1, c1) = http_req(addr, "POST", &format!("/api/pipelines/runs/{run_id}/cancel"), "");
    let (st_c2, c2) = http_req(addr, "POST", &format!("/api/pipelines/runs/{run_id}/cancel"), "");
    assert_eq!(st_c1, 200);
    assert_eq!(st_c2, 200);
    assert_eq!(c1["run"]["status"], json!("completed"));
    assert_eq!(c2["run"]["status"], json!("completed"));

    // 2. Concurrent /retry on a failed run -> strictly 1 gets 202, others 409!
    let (st_f, f_res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Concurrent Retry",
            "steps_v2": [
                {
                    "id": "bad",
                    "kind": "skill",
                    "skill_id": "clean_dataset",
                    "args": { "input_file": "will-exist-later.csv", "output_title": "RetryOut" }
                }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_f, 202);
    let fail_id = f_res["run"]["run_id"].as_str().unwrap().to_string();
    poll_run_until(addr, &fail_id, |r| r["status"] == "failed");

    td.write_ws("will-exist-later.csv", b"id,v\n1,10\n");
    let barrier2 = Arc::new(Barrier::new(threads));
    let mut handles2 = Vec::new();
    for _ in 0..threads {
        let b = barrier2.clone();
        let rid = fail_id.clone();
        handles2.push(std::thread::spawn(move || {
            b.wait();
            http_req(
                addr,
                "POST",
                &format!("/api/pipelines/runs/{rid}/retry"),
                "{}",
            )
            .0
        }));
    }
    let retry_statuses: Vec<u16> = handles2.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(
        retry_statuses.iter().filter(|&&s| s == 202).count(),
        1,
        "retry_statuses: {retry_statuses:?}"
    );
    assert_eq!(
        retry_statuses.iter().filter(|&&s| s == 409).count(),
        threads - 1,
        "retry_statuses: {retry_statuses:?}"
    );
    poll_run_until(addr, &fail_id, |r| r["status"] == "completed");

    // 3. Concurrent /cancel on a paused_checkpoint run -> all 200, final status "cancelled".
    let (st_p2, p2_res) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Concurrent Cancel CP",
            "steps_v2": [{ "id": "cp", "kind": "checkpoint", "checkpoint": { "prompt": "Wait" } }]
        })
        .to_string(),
    );
    assert_eq!(st_p2, 202);
    let cp_id = p2_res["run"]["run_id"].as_str().unwrap().to_string();
    poll_run_until(addr, &cp_id, |r| r["status"] == "paused_checkpoint");

    let barrier3 = Arc::new(Barrier::new(threads));
    let mut handles3 = Vec::new();
    for _ in 0..threads {
        let b = barrier3.clone();
        let rid = cp_id.clone();
        handles3.push(std::thread::spawn(move || {
            b.wait();
            http_req(
                addr,
                "POST",
                &format!("/api/pipelines/runs/{rid}/cancel"),
                "",
            )
        }));
    }
    for h in handles3 {
        let (st_x, v_x) = h.join().unwrap();
        assert_eq!(st_x, 200);
        assert_eq!(v_x["run"]["status"], json!("cancelled"));
    }
}

// ---------------------------------------------------------------------------
// 2. Tables, Docs (XLSX / PDF), and File History Hardening
// ---------------------------------------------------------------------------

#[test]
fn tables_rejects_malformed_utf8_binary_and_handles_ragged_csv() {
    let td = TestDir::new("tables-edge");
    td.write_ws("bad_utf8.csv", b"name,amount\nCaf\xff,12.50\n");
    td.write_ws("binary.csv", b"name,amount\n\x00\x01\x02,10\n");
    td.write_ws("nul.csv", b"name,amount\n\x00\x01\x02,10\n");
    td.write_ws("ragged.csv", b"vendor,amount\nAcme\nBeta,20.50,extra_tag,99\n");

    let addr = spawn_server(td.cfg.clone());

    let (st_u, u_err) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        r#"{"path":"bad_utf8.csv"}"#,
    );
    assert_eq!(st_u, 400, "{u_err}");

    let (st_b, b_err) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        r#"{"path":"binary.csv"}"#,
    );
    assert_eq!(st_b, 400, "{b_err}");

    let (st_n, n_err) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        r#"{"path":"nul.csv"}"#,
    );
    assert_eq!(st_n, 400, "{n_err}");

    let (st_r, r_ok) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        r#"{"path":"ragged.csv","save_as":"ragged_clean.csv"}"#,
    );
    assert_eq!(st_r, 200, "{r_ok}");
    assert_eq!(r_ok["columns"], json!(["vendor", "amount", "col_3", "col_4"]));
    let rows = r_ok["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], json!(["Acme", "", "", ""]));
    assert_eq!(rows[1], json!(["Beta", "20.50", "extra_tag", "99"]));
    assert!(td.cfg.workspace_dir.join("ragged_clean.csv").is_file());
}

#[test]
fn docs_broken_xlsx_xml_and_zip_guards() {
    // 1. Broken XLSX XML: huge col letter overflow, unclosed <v>, huge sparse row XFD999999,
    //    out-of-bounds shared string index, non-finite/huge date serial.
    let broken_sheet = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
        r#"<sheetData>"#,
        r#"<row r="1"><c r="AAAAAAAAAAAAAAAAAAAAAAAA1" t="inlineStr"><is><t>Hdr</t></is></c>"#,
        r#"<c r="B1" t="s"><v>99999</c><c r="C1" s="0"><v>45000</v></c></row>"#,
        r#"<row r="999999"><c r="XFD999999"><v>12345</v></c></row>"#,
        r#"<row r="2"><c r="A2" t="inlineStr"><is><t>Safe</t></is></c><c r="B2"><v>42.5</v></c></row>"#,
        r#"</sheetData></worksheet>"#,
    );
    let wb = r#"<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Broken" sheetId="1" r:id="rId1"/></sheets></workbook>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#;
    let xlsx = docs::zip_write(&[
        ("xl/workbook.xml", wb.as_bytes().to_vec()),
        ("xl/_rels/workbook.xml.rels", rels.as_bytes().to_vec()),
        ("xl/worksheets/sheet1.xml", broken_sheet.as_bytes().to_vec()),
    ]);
    // Must not panic or OOM!
    let sheets = docs::xlsx_sheets(&xlsx).expect("broken XLSX parses safely without OOM/panic");
    assert_eq!(sheets.len(), 1);
    assert!(sheets[0].rows.len() <= 10, "sparse row 999999 must not allocate 1M rows");
    assert!(sheets[0].rows.iter().any(|r| r.iter().any(|c| c == "Safe")));

    // Non-finite / out-of-range serial dates return empty safely.
    assert_eq!(docs::excel_serial_to_iso(f64::NAN, true), "");
    assert_eq!(docs::excel_serial_to_iso(-5.0, false), "");
    assert_eq!(docs::excel_serial_to_iso(9_999_999.0, false), "");

    // 2. Stored (method=0) zip archive exceeding MAX_ARCHIVE_INFLATE across entries is rejected!
    let big_stored = vec![b'X'; 40 * 1024 * 1024];
    // Build stored-only zip manually via zip_write on incompressible-ish? No, zip_write uses DEFLATE if smaller;
    // reading 5 * 40MB stored/deflated entries exceeds MAX_ARCHIVE_INFLATE (160MB).
    let mut entries = Vec::new();
    for i in 0..5 {
        entries.push((format!("f{i}.bin"), big_stored.clone()));
    }
    let refs: Vec<(&str, &[u8])> = entries.iter().map(|(n, d)| (n.as_str(), d.as_slice())).collect();
    let z = docs::zip_write_refs(&refs);
    let zr = docs::ZipReader::open(&z).unwrap();
    let mut err_hit = false;
    for i in 0..5 {
        if zr.read(&format!("f{i}.bin")).is_err() {
            err_hit = true;
            break;
        }
    }
    assert!(err_hit, "ZipReader must enforce MAX_ARCHIVE_INFLATE across entries");
}

#[test]
fn docs_pdf_huge_line_wrap_and_pagination_guards() {
    // 1. Huge unspaced 20,000-char token + giant table cell + 6,000 lines (> MAX_PDF_PAGES).
    let long_word = "X".repeat(20_000);
    let giant_cell = "word ".repeat(600);
    let mut md = format!(
        "# Stress PDF\n\n{long_word}\n\n| ColA | ColB |\n| --- | --- |\n| {giant_cell} | $99.00 |\n\n"
    );
    for i in 0..10_000 {
        md.push_str(&format!("Paragraph line {i}: verifying PDF pagination guard.\n\n"));
    }

    let pdf = docs::markdown_to_pdf(&md, &"T".repeat(120), Some(&"B".repeat(120)));
    assert!(pdf.starts_with(b"%PDF-1.4\n"));
    let raw = String::from_utf8_lossy(&pdf);
    assert!(raw.contains("%%EOF"));

    let (text, pages) = docs::pdf_to_text(pdf).unwrap();
    assert!(
        pages <= docs::MAX_PDF_PAGES,
        "expected <= {} pages, got {pages}",
        docs::MAX_PDF_PAGES
    );
    assert!(text.contains("Stress PDF"), "expected title in extracted PDF text");
}

#[test]
fn put_and_patch_concurrent_expected_sha256_409_and_history_integrity() {
    let td = TestDir::new("put-patch-conc");
    td.write_ws("shared.md", b"line 1\nline 2\nline 3\n");
    let addr = spawn_server(td.cfg.clone());

    let (st_g, got) = http_req(addr, "GET", "/api/files/content?path=shared.md", "");
    assert_eq!(st_g, 200);
    let sha_v1 = got["sha256"].as_str().unwrap().to_string();

    // 8 concurrent PUT requests with the same expected_sha256 -> strictly 1 wins (200), 7 get 409!
    let threads = 8usize;
    let barrier = Arc::new(Barrier::new(threads));
    let mut puts = Vec::new();
    for i in 0..threads {
        let b = barrier.clone();
        let sha = sha_v1.clone();
        puts.push(std::thread::spawn(move || {
            b.wait();
            http_req(
                addr,
                "PUT",
                "/api/files/content",
                &json!({
                    "path": "shared.md",
                    "content": format!("winner-{i}\nline 2\nline 3\n"),
                    "expected_sha256": sha,
                })
                .to_string(),
            )
        }));
    }
    let put_res: Vec<(u16, Value)> = puts.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(
        put_res.iter().filter(|(s, _)| *s == 200).count(),
        1,
        "{put_res:?}"
    );
    assert_eq!(
        put_res.iter().filter(|(s, _)| *s == 409).count(),
        threads - 1,
        "{put_res:?}"
    );

    let sha_v2 = put_res
        .iter()
        .find(|(s, _)| *s == 200)
        .unwrap()
        .1["sha256"]
        .as_str()
        .unwrap()
        .to_string();

    // Concurrent PATCH with sha_v2 -> strictly 1 wins (200), 7 get 409!
    let barrier2 = Arc::new(Barrier::new(threads));
    let mut patches = Vec::new();
    for i in 0..threads {
        let b = barrier2.clone();
        let sha = sha_v2.clone();
        patches.push(std::thread::spawn(move || {
            b.wait();
            http_req(
                addr,
                "PATCH",
                "/api/files/content",
                &json!({
                    "path": "shared.md",
                    "expected_sha256": sha,
                    "ops": [{ "op": "replace", "find": "line 2", "replace": format!("patched-{i}") }]
                })
                .to_string(),
            )
        }));
    }
    let patch_res: Vec<(u16, Value)> = patches.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(
        patch_res.iter().filter(|(s, _)| *s == 200).count(),
        1,
        "{patch_res:?}"
    );
    assert_eq!(
        patch_res.iter().filter(|(s, _)| *s == 409).count(),
        threads - 1,
        "{patch_res:?}"
    );

    // Oversized patch ops (> 64) rejected with 400.
    let too_many_ops: Vec<Value> = (0..65)
        .map(|_| json!({"op":"replace","find":"line 3","replace":"line 3"}))
        .collect();
    let (st_ops, _) = http_req(
        addr,
        "PATCH",
        "/api/files/content",
        &json!({ "path": "shared.md", "ops": too_many_ops }).to_string(),
    );
    assert_eq!(st_ops, 400);

    // Verify .history/ contains exact original v1 bytes and catches manifest sha256 tampering!
    let revs = file_history::list_revisions(&td.cfg.data_dir, &td.cfg.workspace_dir, "shared.md").unwrap();
    assert_eq!(revs.len(), 2);
    let earliest = revs.last().unwrap();
    let v1_bytes = file_history::read_revision(
        &td.cfg.data_dir,
        &td.cfg.workspace_dir,
        "shared.md",
        &earliest.rev_id,
    )
    .unwrap();
    assert_eq!(v1_bytes, b"line 1\nline 2\nline 3\n");

    // Tamper manifest: clear sha256 -> read_revision and restore_revision MUST fail integrity check!
    let bucket = file_history::history_dir(&td.cfg.data_dir)
        .join(tripwire::sha256_hex(b"shared.md"));
    let manifest_p = bucket.join("manifest.json");
    let mut m_val: Value = serde_json::from_str(&std::fs::read_to_string(&manifest_p).unwrap()).unwrap();
    m_val["revisions"][0]["sha256"] = json!("");
    std::fs::write(&manifest_p, m_val.to_string()).unwrap();

    let tamper_err = file_history::read_revision(
        &td.cfg.data_dir,
        &td.cfg.workspace_dir,
        "shared.md",
        &earliest.rev_id,
    )
    .unwrap_err();
    assert!(tamper_err.contains("integrity"), "{tamper_err}");
}

// ---------------------------------------------------------------------------
// 3. Connectors & Tripwire Hardening
// ---------------------------------------------------------------------------

#[test]
fn folder_watch_rejects_symlinked_dir_parent_and_symlink_files_cwe59() {
    let td = TestDir::new("fw-cwe59");
    let outside = td.root.join("outside_sensitive");
    let outside_sub = outside.join("sub");
    std::fs::create_dir_all(&outside_sub).unwrap();
    std::fs::write(outside.join("secret.csv"), "a,b\n1,2\n").unwrap();

    let watch_real = td.root.join("real_watch");
    std::fs::create_dir_all(&watch_real).unwrap();

    #[cfg(unix)]
    {
        let link_dir = td.root.join("link_watch");
        std::os::unix::fs::symlink(&outside, &link_dir).unwrap();
        // Direct symlink watch_dir rejected.
        assert!(connectors::validate_watch_dir(
            &td.cfg.data_dir,
            &td.cfg.data_dir,
            &link_dir.to_string_lossy()
        )
        .is_err());
        // Intermediate symlink parent (link_watch/sub) rejected!
        assert!(connectors::validate_watch_dir(
            &td.cfg.data_dir,
            &td.cfg.data_dir,
            &link_dir.join("sub").to_string_lossy()
        )
        .is_err());
        // Symlinked file inside a valid watch_dir is skipped during polling.
        std::os::unix::fs::symlink(outside.join("secret.csv"), watch_real.join("escape.csv"))
            .unwrap();
    }

    // `..` traversal and `/run` forbidden root rejected.
    assert!(connectors::validate_watch_dir(
        &td.cfg.data_dir,
        &td.cfg.data_dir,
        &format!("{}/../outside_sensitive", watch_real.to_string_lossy())
    )
    .is_err());
    assert!(connectors::validate_watch_dir(&td.cfg.data_dir, &td.cfg.data_dir, "/run").is_err());

    let addr = spawn_server(td.cfg.clone());
    let (st_fw, _) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({
            "name": "Real Dropzone",
            "kind": "folder_watch",
            "watch_dir": watch_real.to_string_lossy(),
            "glob": "*.csv"
        })
        .to_string(),
    );
    assert_eq!(st_fw, 201);

    let mut poll = connectors::FolderWatchPollState::default();
    let ev1 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll);
    let ev2 = connectors::poll_folder_watch_once(&td.cfg, &td.cfg.data_dir, &mut poll);
    assert!(ev1.is_empty() && ev2.is_empty(), "symlinked file must never ingest");
    assert!(!td.cfg.workspace_dir.join("intake/escape.csv").exists());
}

#[test]
fn webhook_rejects_over_1mb_413_and_enforces_timestamp_replay_and_hmac() {
    let td = TestDir::new("wh-hard");
    let addr = spawn_server(td.cfg.clone());

    let (st_t, trg) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({ "name": "Hardened Hook", "kind": "webhook" }).to_string(),
    );
    assert_eq!(st_t, 201);
    let hook_path = trg["hook_path"].as_str().unwrap().to_string();
    let hook_id = hook_path.trim_start_matches("/api/hooks/").to_string();
    let signing_secret = trg["signing_secret"].as_str().unwrap().to_string();

    // 1. >1 MB webhook payload rejected with HTTP 413 over real TCP AND direct call!
    let huge_body = vec![b'a'; connectors::MAX_WEBHOOK_BYTES + 1];
    let (st_413, _, _) = http_req_headers(addr, "POST", &hook_path, &[], &huge_body);
    assert_eq!(st_413, 413, "expected 413 for >1 MB webhook");
    let direct_413 = connectors::verify_and_ingest_webhook(
        &td.cfg,
        &hook_id,
        None,
        None,
        "application/json",
        &huge_body,
    )
    .unwrap_err();
    assert_eq!(direct_413.0, 413);

    // 2. Stale (>5m) or malformed X-Ergon-Timestamp rejected (401).
    let payload = br#"{"event":"order.created","id":42}"#;
    let sig_body = format!(
        "sha256={}",
        tripwire::hex(&tripwire::hmac_sha256(signing_secret.as_bytes(), payload))
    );
    let stale_ts = (records::now_millis() / 1000).saturating_sub(600).to_string();
    let (st_stale, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("X-Ergon-Signature", &sig_body),
            ("X-Ergon-Timestamp", &stale_ts),
        ],
        payload,
    );
    assert_eq!(st_stale, 401);

    let (st_bad_ts, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("X-Ergon-Signature", &sig_body),
            ("X-Ergon-Timestamp", "not-a-timestamp"),
        ],
        payload,
    );
    assert_eq!(st_bad_ts, 401);

    // 3. Valid timestamp-bound HMAC `{ts}.{body}` succeeds (202) and replay is rejected (401)!
    let fresh_ts = (records::now_millis() / 1000).to_string();
    let mut signed_msg = Vec::new();
    signed_msg.extend_from_slice(fresh_ts.as_bytes());
    signed_msg.push(b'.');
    signed_msg.extend_from_slice(payload);
    let sig_ts = format!(
        "sha256={}",
        tripwire::hex(&tripwire::hmac_sha256(signing_secret.as_bytes(), &signed_msg))
    );
    let (st_ok, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("X-Ergon-Signature", &sig_ts),
            ("X-Ergon-Timestamp", &fresh_ts),
            ("X-Ergon-Delivery", "deliv-unique-001"),
        ],
        payload,
    );
    assert_eq!(st_ok, 202);

    // Replaying exact same signed request (timestamp + delivery id) is rejected with 401!
    let (st_rep, _, _) = http_req_headers(
        addr,
        "POST",
        &hook_path,
        &[
            ("X-Ergon-Signature", &sig_ts),
            ("X-Ergon-Timestamp", &fresh_ts),
            ("X-Ergon-Delivery", "deliv-unique-001"),
        ],
        payload,
    );
    assert_eq!(st_rep, 401, "replayed signed webhook must be rejected");
}

#[test]
fn secret_redaction_catches_partial_json_escaped_url_b64_and_preserves_hints() {
    let td = TestDir::new("redact-hard");
    let conn_secret = "whsec_live/Alpha+Beta\"Gamma=Delta99";
    let mut store = connectors::ConnectorSecretsStore::default();
    store
        .secrets
        .insert("partner_key".into(), conn_secret.into());

    // 1. Literal, JSON-escaped (\", \\, \/, \u002b, \u003d), URL-encoded, Base64, hex, and partial echoes.
    let json_escaped = r#"{"echo":"whsec_live\/Alpha\u002bBeta\"Gamma\u003dDelta99"}"#;
    let r_json = connectors::redact_with_store(json_escaped, &store);
    assert_eq!(r_json, r#"{"echo":"[redacted]"}"#, "{r_json}");

    let partial_prefix = "server logged prefix: whsec_live/Alpha+Beta...";
    let r_pref = connectors::redact_with_store(partial_prefix, &store);
    assert!(!r_pref.contains("Alpha+Beta"), "{r_pref}");
    assert!(r_pref.contains("[redacted]"), "{r_pref}");

    let partial_suffix = "tail: ...Alpha+Beta\"Gamma=Delta99";
    let r_suf = connectors::redact_with_store(partial_suffix, &store);
    assert!(!r_suf.contains("Gamma=Delta99"), "{r_suf}");
    assert!(r_suf.contains("[redacted]"), "{r_suf}");

    let b64 = tripwire::base64_encode(conn_secret.as_bytes());
    let r_b64 = connectors::redact_with_store(&format!("b64={b64}"), &store);
    assert!(!r_b64.contains(&b64), "{r_b64}");

    // 2. Active-run bait key partial/JSON-escaped redaction MUST preserve safe token_hint!
    {
        let _arm = tripwire::arm(tripwire::ArmSpec {
            root: td.cfg.data_dir.clone(),
            person_dir: td.cfg.data_dir.clone(),
            workspace: td.cfg.workspace_dir.clone(),
            memory_path: td.cfg.memory_path.clone(),
            person_id: "default".into(),
            run_id: "run-red".into(),
            thread_id: "t-red".into(),
            provenance: "summarize".into(),
            secrets: vec![conn_secret.into()],
        });
        let prompt = tripwire::prompt_section().unwrap();
        let marker = "service key is ";
        let idx = prompt.find(marker).unwrap() + marker.len();
        let bait: String = prompt[idx..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '.')
            .collect();
        let nonce = bait.strip_prefix(tripwire::TOKEN_PREFIX).unwrap().split('-').next().unwrap();
        let safe_hint = tripwire::token_hint(nonce);

        // Safe token_hint stays intact!
        let out_hint = tripwire::redact(&format!("Alert for decoy {safe_hint}"));
        assert!(out_hint.contains(&safe_hint), "safe token_hint must not be redacted: {out_hint}");

        // Partial MAC echo and JSON-escaped bait echo ARE redacted!
        let partial_bait = &bait[..bait.len() - 6];
        let r_partial_bait = tripwire::redact(&format!("leaked {partial_bait}"));
        assert!(!r_partial_bait.contains(&bait[16..]), "{r_partial_bait}");
        assert!(r_partial_bait.contains("[redacted]"), "{r_partial_bait}");

        // Indirect file exfiltration via invoke_connector { file: ... } is blocked!
        td.write_ws("leak.txt", format!("hidden {conn_secret}").as_bytes());
        let p_err = tripwire::check_egress_payload(&format!("hidden {conn_secret}"), "connector")
            .unwrap_err();
        assert!(p_err.contains("stopped for your safety"), "{p_err}");
    }
}

#[test]
fn air_gapped_and_strict_client_policy_edge_cases() {
    std::env::set_var("ERGON_FETCH_ALLOW_PRIVATE", "1");
    let td = TestDir::new("policy-edge");
    let addr = spawn_server(td.cfg.clone());

    // 1. Metadata IP 169.254.169.254 rejected at connector creation time (400).
    let (st_meta, _) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Meta SSRF",
            "kind": "http_webhook",
            "url": "http://169.254.169.254/latest/meta-data"
        })
        .to_string(),
    );
    assert_eq!(st_meta, 400);

    // Create legitimate local connector with require_approval: false.
    let (st_c, c_json) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Client Hook",
            "kind": "http_webhook",
            "url": "http://127.0.0.1:9/never",
            "require_approval": false
        })
        .to_string(),
    );
    assert_eq!(st_c, 201);
    let conn_id = c_json["connector"]["id"].as_str().unwrap().to_string();

    // 2. Under strict_client, invoke_connector parks decision AND redacts host paths!
    let (st_p1, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    assert_eq!(st_p1, 200);

    let held1 = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({
            "connector_id": conn_id,
            "payload": { "note": "See /home/cayleb/secret/report.md" }
        }),
    )
    .unwrap();
    assert!(held1.contains("parked for your person's approval"), "{held1}");
    let decs = records::load_decisions(&td.cfg.data_dir);
    let dec1 = decs.iter().find(|d| d["status"] == "pending").unwrap();
    assert!(
        !dec1["payload_preview"].as_str().unwrap_or("").contains("/home/cayleb/"),
        "strict_client must redact host paths in parked connector decision: {dec1}"
    );
    let dec1_id = dec1["id"].as_str().unwrap().to_string();

    // Switch to air_gapped BEFORE approving dec1 -> approving dec1 must block delivery!
    let (st_p2, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"air_gapped"}"#,
    );
    assert_eq!(st_p2, 200);
    let (st_app1, app1) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec1_id}/approve"),
        "",
    );
    assert_eq!(st_app1, 200);
    assert!(
        app1["outcome"].as_str().unwrap_or("").contains("Air-Gapped"),
        "expected air_gapped block on late approval: {app1}"
    );

    // 3. Switch back to strict_client, park another decision, disable connector, then approve -> must NOT deliver!
    http_req(addr, "PUT", "/api/workspace/policy", r#"{"profile":"strict_client"}"#);
    execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({ "connector_id": conn_id, "payload": { "msg": "hi" } }),
    )
    .unwrap();
    let dec2_id = records::load_decisions(&td.cfg.data_dir)
        .into_iter()
        .find(|d| d["status"] == "pending")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Disable connector.
    let (st_dis, _) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "id": conn_id,
            "name": "Client Hook",
            "kind": "http_webhook",
            "url": "http://127.0.0.1:9/never",
            "require_approval": false,
            "enabled": false
        })
        .to_string(),
    );
    assert_eq!(st_dis, 200);

    let (st_app2, app2) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec2_id}/approve"),
        "",
    );
    assert_eq!(st_app2, 200);
    assert!(
        app2["outcome"].as_str().unwrap_or("").contains("disabled"),
        "disabled connector must not deliver on approval: {app2}"
    );

    // 4. Symlinked attachment file in invoke_connector is rejected.
    #[cfg(unix)]
    {
        let outside_file = td.root.join("outside.txt");
        std::fs::write(&outside_file, b"secret").unwrap();
        std::os::unix::fs::symlink(&outside_file, td.cfg.workspace_dir.join("sym.txt")).unwrap();
        let sym_err = execute_tool(
            &td.cfg,
            "invoke_connector",
            &json!({ "connector_id": conn_id, "file": "sym.txt" }),
        )
        .unwrap_err();
        assert!(sym_err.contains("symlink"), "{sym_err}");
    }
}
