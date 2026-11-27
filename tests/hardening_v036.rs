//! Cycle 3 Hardening Suite (v0.36.0)
//!
//! Covers:
//! - E1: Invalid/negative/NaN pricing rates, zero-token/negative retainer budgets,
//!   orphan engagement creation (404/400), saturating RunUsage token/cost math,
//!   and corrupt/truncated/non-UTF8 clients.json recovery without 500s.
//! - E2: Corrupt/truncated vault_index.json recovery, bounded reindex memory/chunk caps,
//!   and execute_vault_aggregate edge cases (zero matches, malformed globs, missing columns,
//!   "$1,200.50"/"N/A" coercion, CWE-22/CWE-59 traversal/symlink checks on glob & save_as).
//! - E3: DAG validation (self-loop, nonexistent deps, diamond joins, disconnected components,
//!   empty & >50 for_each arrays), idempotent deliver checkpoint settlement under repeated/racing
//!   approve/resume POSTs, SSE parser resilience on malformed/partial data: frames & early EOF,
//!   and run_loop compaction on empty tool list & oversized single tool result.

use ergon_rs::search::SearchCache;
use ergon_rs::{
    call_model_with_usage_streaming, library, records, run_agent_with_options, serve, sse,
    tables, AgentLoopOptions, Config, LiveRuns, Store, ToolCtx,
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
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base
            .join("test-data")
            .join(format!("hardening-v036-{name}-{}-{seq}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: model_base_url.to_string(),
            model_id: "mock-pro".to_string(),
            api_key: api_key.map(str::to_string),
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

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let req = if body.is_empty() && matches!(method, "GET" | "HEAD") {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            addr.port()
        )
    } else if body.is_empty() {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            addr.port()
        )
    } else {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            addr.port(),
            body.len()
        )
    };
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

fn poll_run_until<F>(addr: SocketAddr, run_id: &str, pred: F) -> Value
where
    F: Fn(&Value) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let (st, res) = http_req(addr, "GET", &format!("/api/pipelines/runs/{run_id}"), "");
        assert_eq!(st, 200, "GET run failed: {res}");
        let run = res["run"].clone();
        if pred(&run) {
            return run;
        }
        if Instant::now() >= deadline {
            panic!("timed out polling run {run_id}: last={run}");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

// ---------------------------------------------------------------------------
// E1: Clients, Engagements, Pricing, Saturating RunUsage & Corrupt Recovery
// ---------------------------------------------------------------------------

#[test]
fn e1_pricing_retainer_orphan_and_saturating_math_hardening() {
    let td = TestDir::new("e1-metering", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(td.cfg.clone());

    // 1. Reject invalid / negative / NaN-like / excessive pricing rates with 400
    for bad_pricing in [
        json!({ "model_pricing": { "m1": { "input_per_1m_usd": -1.0 } } }),
        json!({ "model_pricing": { "m1": { "output_per_1m_usd": "NaN" } } }),
        json!({ "model_pricing": { "m1": { "cached_input_per_1m_usd": -0.01 } } }),
        json!({ "model_pricing": { "m1": { "input_per_1m_usd": 1e12 } } }),
        json!({ "model_pricing": { "": { "input_per_1m_usd": 1.0 } } }),
        json!({ "model_pricing": "not-an-object" }),
    ] {
        let (st, _) = http_req(addr, "PUT", "/api/config", &bad_pricing.to_string());
        assert_eq!(st, 400, "expected 400 for bad pricing {bad_pricing}");
    }

    // Direct Rust ModelPrice edge checks (NaN, Inf, negative rates in struct)
    let corrupt_price = records::ModelPrice {
        input_per_1m_usd: f64::NAN,
        output_per_1m_usd: f64::INFINITY,
        cached_input_per_1m_usd: -50.0,
    };
    assert_eq!(records::compute_call_cost_usd(&corrupt_price, 0, 0, 0), 0.0);
    assert_eq!(records::compute_call_cost_usd(&corrupt_price, 1000, 500, 200), 0.0);

    // 2. Orphan engagement creation for nonexistent client_id -> 404/400
    let (st_orphan, _) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": "nonexistent-client",
            "name": "Orphan Retainer",
            "retainer_budget_usd": 100.0
        })
        .to_string(),
    );
    assert!(
        st_orphan == 404 || st_orphan == 400,
        "expected 404/400 for orphan engagement, got {st_orphan}"
    );

    // Create real client
    let (st_c, res_c) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({ "name": "Hardened Client", "default_rate_usd": 150.0 }).to_string(),
    );
    assert_eq!(st_c, 201, "{res_c}");
    let cid = res_c["client"]["id"].as_str().unwrap().to_string();

    // Reject negative retainer budget / negative hours / NaN rate / negative client rate
    for bad_eng in [
        json!({ "client_id": cid, "name": "Neg Budget", "retainer_budget_usd": -5.0 }),
        json!({ "client_id": cid, "name": "Neg Hours", "retainer_hours": -1.0 }),
        json!({ "client_id": cid, "name": "Neg Rate", "hourly_rate_usd": -25.0 }),
        json!({ "client_id": cid, "name": "Bad Budget", "retainer_budget_usd": "NaN" }),
    ] {
        let (st, _) = http_req(addr, "POST", "/api/engagements", &bad_eng.to_string());
        assert_eq!(st, 400, "expected 400 for invalid engagement {bad_eng}");
    }

    // Zero retainer budget (0.0) is accepted cleanly and must not cause div-by-zero in metering
    let (st_zero, res_zero) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "client_id": cid,
            "name": "Zero Cap Retainer",
            "retainer_budget_usd": 0.0,
            "retainer_hours": 0.0,
            "activate": true
        })
        .to_string(),
    );
    assert_eq!(st_zero, 201, "{res_zero}");
    let eid = res_zero["engagement"]["id"].as_str().unwrap().to_string();

    // 3. Saturating RunUsage math + zero-token ledger entry + huge u64 tokens
    let mut u = records::RunUsage::default();
    u.add_call("mock-pro", 0, 0, 0, 0, 0, 0.0);
    assert_eq!(u.calls, 1);
    assert_eq!(u.total_tokens, 0);
    assert_eq!(u.cost_usd, 0.0);

    u.add_call("mock-pro", u64::MAX - 5, 10, 0, 0, u64::MAX - 1, 1.25);
    assert_eq!(u.prompt_tokens, u64::MAX - 5);
    assert_eq!(u.total_tokens, u64::MAX);
    assert_eq!(u.latency_ms, u64::MAX - 1);
    assert_eq!(u.cost_usd, 1.25);

    // NaN / Inf / negative cost passed to add_call or accumulate must NOT corrupt cost_usd!
    u.add_call("mock-pro", 100, 100, 200, 0, 50, f64::NAN);
    assert_eq!(u.cost_usd, 1.25);
    u.add_call("mock-pro", 100, 100, 200, 0, 50, f64::INFINITY);
    assert_eq!(u.cost_usd, 1.25);
    u.accumulate(&records::RunUsage {
        model_id: "mock-pro".to_string(),
        calls: 2,
        prompt_tokens: 100,
        completion_tokens: 100,
        total_tokens: 200,
        cached_tokens: 0,
        latency_ms: 100,
        cost_usd: -999.0,
        estimated_minutes_saved: f64::NAN,
        billable_value_usd: f64::INFINITY,
    });
    assert_eq!(u.prompt_tokens, u64::MAX);
    assert_eq!(u.latency_ms, u64::MAX);
    assert_eq!(u.cost_usd, 1.25);

    // Persist huge & zero-token entries to ledger and verify GET /api/metering/summary never overflows or NaNs
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: records::now_millis(),
            run_id: "r-sat-1".to_string(),
            thread_id: "t-sat-1".to_string(),
            person_id: "default".to_string(),
            engagement_id: Some(eid.clone()),
            client_id: Some(cid.clone()),
            source: "agent".to_string(),
            finished: true,
            answered: true,
            usage: u.clone(),
        },
    );
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: records::now_millis(),
            run_id: "r-sat-2".to_string(),
            thread_id: "t-sat-2".to_string(),
            person_id: "default".to_string(),
            engagement_id: Some(eid.clone()),
            client_id: Some(cid.clone()),
            source: "agent".to_string(),
            finished: true,
            answered: true,
            usage: u,
        },
    );
    let (st_m, res_m) = http_req(addr, "GET", "/api/metering/summary", "");
    assert_eq!(st_m, 200, "{res_m}");
    assert_eq!(res_m["totals"]["retainer_burn_pct"], json!(0.0));
    assert_eq!(res_m["totals"]["cost_usd"], json!(2.5));
}

#[test]
fn e1_corrupt_clients_json_recovers_without_500() {
    let td = TestDir::new("e1-corrupt-clients", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(td.cfg.clone());
    let clients_file = records::clients_path(&td.cfg.data_dir);

    // 1. Truncated JSON in clients.json
    std::fs::write(&clients_file, "{\"clients\":[{\"id\":\"broken\"").unwrap();
    let (st_get, res_get) = http_req(addr, "GET", "/api/clients", "");
    assert_eq!(st_get, 200, "GET /api/clients must recover from truncated JSON: {res_get}");
    assert_eq!(res_get["clients"], json!([]));

    let (st_sum, _) = http_req(addr, "GET", "/api/metering/summary", "");
    assert_eq!(st_sum, 200);

    // 2. Non-UTF8 corrupt binary bytes in clients.json
    std::fs::write(&clients_file, [0xff, 0xfe, 0x00, 0x80, 0xbd]).unwrap();
    let (st_post, res_post) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({ "name": "Recovered Co" }).to_string(),
    );
    assert_eq!(st_post, 201, "POST /api/clients must recover from binary corrupt file: {res_post}");
    assert_eq!(res_post["client"]["name"], json!("Recovered Co"));
}

// ---------------------------------------------------------------------------
// E2: Corrupt vault_index.json Recovery, Bounded Reindex & Vault Aggregate
// ---------------------------------------------------------------------------

#[test]
fn e2_corrupt_vault_index_recovers_and_reindex_stays_bounded() {
    let td = TestDir::new("e2-vault-corrupt", "http://127.0.0.1:9/v1", None);
    td.write_ws("notes/alpha.md", b"# Alpha\n\nTarget sapphire_hardening_marker_101.\n");
    // Write a large file exceeding MAX_PLAIN_TEXT_BYTES cap or large chunk count to verify safe capping
    let mut big_lines = String::new();
    for i in 0..1200 {
        big_lines.push_str(&format!("Paragraph {i}: {}\n\n", "word ".repeat(90)));
    }
    td.write_ws("notes/big.txt", big_lines.as_bytes());

    let vault_path = td.cfg.data_dir.join("vault_index.json");

    // 1. Truncated JSON in vault_index.json -> SearchCache::get rebuilds cleanly
    std::fs::write(&vault_path, "{\"version\":1,\"files\":[{\"path\":\"notes/alpha.md\"").unwrap();
    let cache = SearchCache::new();
    let idx = cache.get("default", &td.cfg.data_dir, &td.cfg.workspace_dir, &td.cfg.memory_path);
    assert_eq!(idx.files.len(), 2);
    let hits = idx.search_vault_chunks("sapphire_hardening_marker_101", "", "exact", 5);
    assert_eq!(hits.len(), 1);
    let big_file = idx.files.iter().find(|f| f.path == "notes/big.txt").unwrap();
    assert!(
        big_file.chunk_range.len() <= 1000,
        "chunk count per file must be capped <= 1000, got {}",
        big_file.chunk_range.len()
    );

    // 2. Corrupt chunk embedding inside otherwise valid JSON -> rebuilds via HTTP search & reindex
    std::fs::write(
        &vault_path,
        json!({
            "version": 1,
            "embedding_sig": "det-256",
            "files": [{
                "path": "notes/alpha.md",
                "sha256": "deadbeef",
                "chunks": [{ "chunk_id": "notes/alpha.md:0", "text": "broken", "embedding": [] }]
            }]
        })
        .to_string(),
    )
    .unwrap();

    let addr = spawn_server(td.cfg.clone());
    let (st_s, res_s) = http_req(
        addr,
        "GET",
        "/api/search?q=sapphire_hardening_marker_101&mode=exact",
        "",
    );
    assert_eq!(st_s, 200, "{res_s}");
    assert_eq!(res_s["counts"]["files"], json!(1));

    // Corrupt vault_index.json with non-UTF8 bytes before POST /api/vault/reindex
    std::fs::write(&vault_path, [0xff, 0x00, 0xfe, 0xab]).unwrap();
    let (st_re, res_re) = http_req(addr, "POST", "/api/vault/reindex", "");
    assert_eq!(st_re, 200, "{res_re}");
    assert_eq!(res_re["ok"], json!(true));
    assert_eq!(res_re["indexed_files"], json!(2));

    // Verify persisted vault_index.json is now valid JSON again
    let repaired: Value = serde_json::from_str(&std::fs::read_to_string(&vault_path).unwrap())
        .expect("vault_index.json should be valid JSON after reindex");
    assert_eq!(repaired["version"], json!(1));
}

#[test]
fn e2_vault_aggregate_edge_cases_and_cwe22_cwe59_checks() {
    let td = TestDir::new("e2-agg", "http://127.0.0.1:9/v1", None);
    td.write_ws(
        "data/good.csv",
        b"Vendor,Amount,Note\nAcme,\"$1,200.50\",ok\nBeta,(200.50),credit\n",
    );
    td.write_ws(
        "data/extra_no_amt.csv",
        b"Vendor,Note\nGamma,no-amount-col\n",
    );
    td.write_ws("data/with_na.csv", b"Vendor,Amount\nDelta,N/A\n");
    td.write_ws("connector_secrets.json", b"{\"api_key\":\"secret\"}");

    let outside = td.root.join("outside_table.csv");
    std::fs::write(&outside, "Vendor,Amount\nEvil,9999\n").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, td.cfg.workspace_dir.join("data/sym.csv")).unwrap();
        let sym_dir = td.root.join("outside_dir");
        std::fs::create_dir_all(&sym_dir).unwrap();
        std::fs::write(sym_dir.join("leak.csv"), "Vendor,Amount\nLeak,10\n").unwrap();
        std::os::unix::fs::symlink(&sym_dir, td.cfg.workspace_dir.join("sym_dir")).unwrap();
        std::os::unix::fs::symlink(
            td.root.join("dangling_out.csv"),
            td.cfg.workspace_dir.join("dangling_save.csv"),
        )
        .unwrap();
    }

    let addr = spawn_server(td.cfg.clone());

    // 1. "$1,200.50" + "(200.50)" currency coercion + missing Amount column in extra_no_amt.csv
    let (st_ok, res_ok) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({
            "glob": "data/{good,extra_no_amt}.csv",
            "aggregates": [
                { "col": "Amount", "fn": "sum", "as": "net" },
                { "col": "*", "fn": "count", "as": "rows" }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_ok, 200, "{res_ok}");
    assert_eq!(res_ok["rows"][0], json!(["1000", "3"]));

    // 2. Non-numeric "N/A" in sum returns clean 400 (no panic), while count succeeds
    let (st_na, res_na) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({
            "glob": "data/with_na.csv",
            "aggregates": [{ "col": "Amount", "fn": "sum", "as": "total" }]
        })
        .to_string(),
    );
    assert_eq!(st_na, 400, "expected 400 on N/A sum: {res_na}");

    let (st_na_cnt, res_na_cnt) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({
            "glob": "data/with_na.csv",
            "aggregates": [{ "col": "Amount", "fn": "count", "as": "cnt" }]
        })
        .to_string(),
    );
    assert_eq!(st_na_cnt, 200, "{res_na_cnt}");
    assert_eq!(res_na_cnt["rows"][0], json!(["1"]));

    // 3. Zero matched files -> 400
    let (st_zero, _) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({ "glob": "data/does_not_exist_*.csv" }).to_string(),
    );
    assert_eq!(st_zero, 400);

    // 4. Malformed globs -> 400
    for bad_glob in [
        "data/{csv,xlsx",
        "data/csv}",
        "data/{a,{b,c}}.csv",
        "data/{a,}.csv",
        "data/{}.csv",
    ] {
        let (st, res) = http_req(
            addr,
            "POST",
            "/api/vault/aggregate",
            &json!({ "glob": bad_glob }).to_string(),
        );
        assert_eq!(st, 400, "expected 400 for malformed glob {bad_glob}: {res}");
    }

    // 5. Missing columns in where / group_by / aggregates / sort -> 400
    for bad_col_body in [
        json!({ "glob": "data/good.csv", "where": [{ "col": "MissingCol", "op": "eq", "value": "1" }] }),
        json!({ "glob": "data/good.csv", "group_by": ["MissingCol"] }),
        json!({ "glob": "data/good.csv", "aggregates": [{ "col": "MissingCol", "fn": "sum" }] }),
        json!({ "glob": "data/good.csv", "sort": { "col": "MissingCol", "dir": "asc" } }),
    ] {
        let (st, _) = http_req(addr, "POST", "/api/vault/aggregate", &bad_col_body.to_string());
        assert_eq!(st, 400, "expected 400 for missing column {bad_col_body}");
    }

    // 6. CWE-22 / CWE-59 traversal & symlink checks on glob and save_as
    for trav_glob in [
        "../outside_table.csv",
        "data/%2e%2e/outside_table.csv",
        "data/{../outside,good}.csv",
        "data\\good.csv",
        "/etc/*.csv",
        ".git/*.csv",
        "connector_secrets.json",
    ] {
        let (st, res) = http_req(
            addr,
            "POST",
            "/api/vault/aggregate",
            &json!({ "glob": trav_glob }).to_string(),
        );
        assert_eq!(st, 403, "expected 403 for traversal glob {trav_glob}: {res}");
    }

    // Symlinked CSV file / dir must never be read by glob
    let direct_sym = tables::execute_vault_aggregate(
        &td.cfg,
        &json!({ "glob": "sym_dir/*.csv" }),
    );
    assert!(direct_sym.is_err(), "symlinked dir must yield 0 matches");

    for bad_save in [
        "../escape.csv",
        "%2e%2e/escape.csv",
        "connector_secrets.json",
        ".hidden.csv",
        #[cfg(unix)]
        "data/sym.csv",
        #[cfg(unix)]
        "dangling_save.csv",
    ] {
        let (st, res) = http_req(
            addr,
            "POST",
            "/api/vault/aggregate",
            &json!({
                "glob": "data/good.csv",
                "save_as": bad_save
            })
            .to_string(),
        );
        assert_eq!(st, 403, "expected 403 for unsafe save_as {bad_save}: {res}");
    }
    assert_eq!(
        std::fs::read_to_string(&outside).unwrap(),
        "Vendor,Amount\nEvil,9999\n"
    );
}

// ---------------------------------------------------------------------------
// E3: DAG Validation, Idempotent Deliver, SSE Parser & Compaction Edge Cases
// ---------------------------------------------------------------------------

#[test]
fn e3_dag_validation_edge_cases_and_disconnected_execution() {
    let td = TestDir::new("e3-dag-val", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(td.cfg.clone());

    // 1. Self-loop -> 400
    let (st_self, res_self) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [{
                "id": "step_a",
                "kind": "skill",
                "skill_id": "write_document",
                "depends_on": ["step_a"],
                "args": { "title": "Self", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_self, 400, "{res_self}");
    assert_eq!(res_self["valid"], json!(false));

    // 2. Nonexistent dependency -> 400
    let (st_ghost, res_ghost) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [{
                "id": "step_a",
                "kind": "skill",
                "skill_id": "write_document",
                "depends_on": ["does_not_exist"],
                "args": { "title": "Ghost", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_ghost, 400, "{res_ghost}");

    // 3. Empty and >50 for_each arrays -> 400
    let (st_empty_arr, res_empty_arr) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [{
                "id": "fe_empty",
                "kind": "skill",
                "skill_id": "write_document",
                "for_each": [],
                "args": { "title": "T", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_empty_arr, 400, "{res_empty_arr}");

    let (st_empty_str_arr, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [{
                "id": "fe_empty_s",
                "kind": "skill",
                "skill_id": "write_document",
                "for_each": "[]",
                "args": { "title": "T", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_empty_str_arr, 400);

    let (st_empty_input_arr, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "inputs": { "docs": [] },
            "steps_v2": [{
                "id": "fe_empty_in",
                "kind": "skill",
                "skill_id": "write_document",
                "for_each": "{{inputs.docs}}",
                "args": { "title": "{{item}}", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_empty_input_arr, 400);

    let fifty_one: Vec<String> = (0..51).map(|i| format!("item_{i}")).collect();
    let (st_over50, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [{
                "id": "fe_51",
                "kind": "skill",
                "skill_id": "write_document",
                "for_each": fifty_one,
                "args": { "title": "{{item}}", "sections": ["x"] }
            }]
        })
        .to_string(),
    );
    assert_eq!(st_over50, 400);

    // 4. Diamond join -> 200
    let (st_diamond, res_diamond) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [
                { "id": "a", "kind": "skill", "skill_id": "write_document", "args": { "title": "A", "sections": ["1"] } },
                { "id": "b", "kind": "skill", "skill_id": "write_document", "depends_on": ["a"], "args": { "title": "B", "sections": ["{{steps.a.output}}"] } },
                { "id": "c", "kind": "skill", "skill_id": "write_document", "depends_on": ["a"], "args": { "title": "C", "sections": ["{{steps.a.output}}"] } },
                { "id": "d", "kind": "skill", "skill_id": "write_document", "depends_on": ["b", "c"], "args": { "title": "D", "sections": ["{{steps.b.output}}", "{{steps.c.output}}"] } }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_diamond, 200, "{res_diamond}");
    assert_eq!(res_diamond["topological_order"].as_array().unwrap().len(), 4);

    // 5. Disconnected components -> 200 and execute cleanly; cross-component template -> 400
    let (st_cross, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "steps_v2": [
                { "id": "c1_a", "kind": "skill", "skill_id": "write_document", "args": { "title": "C1A", "sections": ["1"] } },
                { "id": "c2_a", "kind": "skill", "skill_id": "write_document", "args": { "title": "C2A", "sections": ["2"] } },
                { "id": "c2_b", "kind": "skill", "skill_id": "write_document", "depends_on": ["c2_a"], "args": { "title": "C2B", "sections": ["{{steps.c1_a.output}}"] } }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_cross, 400, "cross-component reference without depends_on must be 400");

    let disconn_steps = json!([
        { "id": "c1_a", "kind": "skill", "skill_id": "write_document", "args": { "title": "Disc_C1A", "sections": ["1"] } },
        { "id": "c1_b", "kind": "skill", "skill_id": "write_document", "depends_on": ["c1_a"], "args": { "title": "Disc_C1B", "sections": ["2"] } },
        { "id": "c2_a", "kind": "skill", "skill_id": "write_document", "args": { "title": "Disc_C2A", "sections": ["3"] } },
        { "id": "c2_b", "kind": "skill", "skill_id": "write_document", "depends_on": ["c2_a"], "args": { "title": "Disc_C2B", "sections": ["4"] } }
    ]);
    let (st_dv, res_dv) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({ "steps_v2": disconn_steps }).to_string(),
    );
    assert_eq!(st_dv, 200, "{res_dv}");
    assert_eq!(res_dv["topological_order"].as_array().unwrap().len(), 4);

    let (st_dr, res_dr) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Disconnected", "steps_v2": disconn_steps }).to_string(),
    );
    assert_eq!(st_dr, 202, "{res_dr}");
    let run_id = res_dr["run"]["run_id"].as_str().unwrap().to_string();
    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "{done}");
    assert!(td.cfg.workspace_dir.join("Disc_C1B.md").exists());
    assert!(td.cfg.workspace_dir.join("Disc_C2B.md").exists());
}

#[test]
fn e3_deliver_checkpoint_settlement_is_idempotent_under_repeated_and_racing_posts() {
    std::env::set_var("ERGON_FETCH_ALLOW_PRIVATE", "1");
    let hook_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let hook_addr = hook_listener.local_addr().unwrap();
    let deliveries = Arc::new(AtomicUsize::new(0));
    let del_bg = deliveries.clone();
    std::thread::spawn(move || {
        for mut stream in hook_listener.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            del_bg.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(40));
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}",
            );
        }
    });

    let td = TestDir::new("e3-idemp-deliver", "http://127.0.0.1:9/v1", None);
    let addr = spawn_server(td.cfg.clone());

    let (st_conn, res_conn) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "id": "idemp-hook",
            "name": "Idemp Hook",
            "kind": "http_webhook",
            "url": format!("http://127.0.0.1:{}/hook", hook_addr.port()),
            "method": "POST",
            "enabled": true
        })
        .to_string(),
    );
    assert_eq!(st_conn, 201, "{res_conn}");
    let (st_pol, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    assert_eq!(st_pol, 200);

    let steps = json!([
        {
            "id": "send",
            "kind": "deliver",
            "deliver": {
                "connector_id": "idemp-hook",
                "payload": { "event": "idemp_test" }
            }
        },
        {
            "id": "after",
            "kind": "skill",
            "skill_id": "write_document",
            "depends_on": ["send"],
            "args": { "title": "After_Idemp", "sections": ["{{steps.send.output}}"] }
        }
    ]);

    let (st_run, res_run) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({ "name": "Idemp Deliver", "steps_v2": steps }).to_string(),
    );
    assert_eq!(st_run, 202, "{res_run}");
    let run_id = res_run["run"]["run_id"].as_str().unwrap().to_string();
    let paused = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("paused_checkpoint" | "completed" | "failed"))
    });
    assert_eq!(paused["status"], json!("paused_checkpoint"), "{paused}");
    let dec_id = paused["pending_checkpoint"]["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Race simultaneous /resume and /decisions/:id/approve + repeat POSTs
    let mut handles = Vec::new();
    for i in 0..4 {
        let r_id = run_id.clone();
        let d_id = dec_id.clone();
        handles.push(std::thread::spawn(move || {
            if i % 2 == 0 {
                http_req(
                    addr,
                    "POST",
                    &format!("/api/pipelines/runs/{r_id}/resume"),
                    r#"{"action":"approve"}"#,
                )
            } else {
                http_req(addr, "POST", &format!("/api/decisions/{d_id}/approve"), "")
            }
        }));
    }
    for h in handles {
        let _ = h.join().unwrap();
    }

    let done = poll_run_until(addr, &run_id, |r| {
        matches!(r["status"].as_str(), Some("completed" | "failed"))
    });
    assert_eq!(done["status"], json!("completed"), "{done}");

    // Additional repeated approve & resume after completion
    let (st_rep1, _) = http_req(
        addr,
        "POST",
        &format!("/api/pipelines/runs/{run_id}/resume"),
        r#"{"action":"approve"}"#,
    );
    assert_eq!(st_rep1, 409);
    let (st_rep2, _) = http_req(addr, "POST", &format!("/api/decisions/{dec_id}/approve"), "");
    assert_eq!(st_rep2, 400);

    assert_eq!(
        deliveries.load(Ordering::SeqCst),
        1,
        "deliver checkpoint must execute webhook EXACTLY ONCE across concurrent/repeated POSTs"
    );
}

#[test]
fn e3_sse_malformed_frames_early_eof_and_compaction_edge_cases() {
    // 1. SSE parser hardening: malformed data: frames, truncated tool_calls at EOF, and empty EOF
    let mode = Arc::new(AtomicUsize::new(0));
    let mode_bg = mode.clone();
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
            let m = mode_bg.load(Ordering::SeqCst);
            let mut s = reader.into_inner();
            let sse_body = match m {
                0 => concat!(
                    ": keepalive\n\n",
                    "data: {not-valid-json\n\n",
                    "data: \n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"{\\\"done\\\":true,\"}}]}\n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"\\\"summary\\\":\\\"recovered\\\"}\"}}]}\n\n",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"\n" // truncated partial frame at EOF
                ),
                1 => "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"write_file\",\"arguments\":\"{\\\"path\\\":\\\"a.txt\\\"\"}}]}}]}\n\n",
                _ => "data: {\"error\":{\"message\":\"upstream aborted\"}}\n\n",
            };
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sse_body}",
                sse_body.len()
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });

    let td = TestDir::new(
        "e3-sse-compact",
        &format!("http://127.0.0.1:{}/v1", mock_addr.port()),
        Some("test-key"),
    );
    let msgs = vec![
        json!({"role": "system", "content": "sys"}),
        json!({"role": "user", "content": "hi"}),
    ];

    // Mode 0: malformed + partial EOF frame ignored cleanly while valid chunks succeed
    mode.store(0, Ordering::SeqCst);
    let (v0, _) = call_model_with_usage_streaming(&td.cfg, "s0", &msgs, None).unwrap();
    assert_eq!(v0["done"], json!(true));
    assert_eq!(v0["summary"], json!("recovered"));

    // Mode 1: truncated tool_calls arguments at early EOF MUST return Err (never silent args: {})
    mode.store(1, Ordering::SeqCst);
    let err1 = call_model_with_usage_streaming(&td.cfg, "s1", &msgs, None).unwrap_err();
    assert!(err1.contains("incomplete") || err1.contains("malformed"), "{err1}");

    // Mode 2: SSE error chunk with no content returns clean error
    mode.store(2, Ordering::SeqCst);
    let err2 = call_model_with_usage_streaming(&td.cfg, "s2", &msgs, None).unwrap_err();
    assert!(err2.contains("upstream aborted"), "{err2}");

    // 2. run_loop compaction edge cases: empty tool list & oversized single tool result
    td.write_ws("huge.txt", "Z".repeat(5000).as_bytes());
    let loop_mode = Arc::new(AtomicUsize::new(0));
    let loop_step = Arc::new(AtomicUsize::new(0));
    let max_seen_msg_chars = Arc::new(AtomicUsize::new(0));
    let lm_bg = loop_mode.clone();
    let ls_bg = loop_step.clone();
    let msmc_bg = max_seen_msg_chars.clone();

    let loop_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let loop_addr = loop_listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in loop_listener.incoming().flatten() {
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
            let v: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            for m in v["messages"].as_array().into_iter().flatten().skip(1) {
                if let Some(c) = m["content"].as_str() {
                    msmc_bg.fetch_max(c.len(), Ordering::SeqCst);
                }
            }
            let reply = if lm_bg.load(Ordering::SeqCst) == 0 {
                json!({ "done": true, "summary": "Zero tools ok." }).to_string()
            } else {
                let st = ls_bg.fetch_add(1, Ordering::SeqCst);
                if st == 0 {
                    json!({ "tool": "read_file", "args": { "path": "huge.txt" } }).to_string()
                } else {
                    json!({ "done": true, "summary": "Oversized tool handled." }).to_string()
                }
            };
            let payload = json!({
                "choices": [{ "message": { "role": "assistant", "content": reply } }]
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

    let mut cfg_loop = td.cfg.clone();
    cfg_loop.model_base_url = format!("http://127.0.0.1:{}/v1", loop_addr.port());
    let lib = Arc::new(Mutex::new(library::Library::load(&cfg_loop.data_dir)));
    let ctx = ToolCtx {
        library: lib,
        root_data_dir: cfg_loop.data_dir.clone(),
        person_id: "default".to_string(),
    };
    let store = Arc::new(Mutex::new(Store::load(&cfg_loop.data_dir)));
    let tid = store.lock().unwrap().create_thread();
    let live_runs: LiveRuns = Arc::new(Mutex::new(HashMap::new()));
    let bus = sse::RunEventBus::new();

    // Case A: empty tool list + tight budget
    loop_mode.store(0, Ordering::SeqCst);
    let out_empty = run_agent_with_options(
        &cfg_loop,
        &ctx,
        &store,
        &tid,
        "Just answer directly",
        &[],
        Some((&live_runs, &bus, "default")),
        AgentLoopOptions {
            context_budget_chars: 1400,
            max_steps: 5,
            snapshot: None,
            ..AgentLoopOptions::default()
        },
    )
    .unwrap();
    assert!(out_empty.steps.is_empty());
    assert_eq!(out_empty.summary, "Zero tools ok.");

    // Case B: oversized single tool result (5000 chars) under 1400-char context budget
    loop_mode.store(1, Ordering::SeqCst);
    max_seen_msg_chars.store(0, Ordering::SeqCst);
    let tid2 = store.lock().unwrap().create_thread();
    let out_huge = run_agent_with_options(
        &cfg_loop,
        &ctx,
        &store,
        &tid2,
        "Read huge.txt and summarize",
        &[],
        Some((&live_runs, &bus, "default")),
        AgentLoopOptions {
            context_budget_chars: 1400,
            max_steps: 5,
            snapshot: None,
            ..AgentLoopOptions::default()
        },
    )
    .unwrap();
    assert_eq!(out_huge.summary, "Oversized tool handled.");
    assert!(
        max_seen_msg_chars.load(Ordering::SeqCst) < 1200,
        "oversized tool result must be bounded under tight context budget, got {} chars",
        max_seen_msg_chars.load(Ordering::SeqCst)
    );
}
