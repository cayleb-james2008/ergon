//! Cycle 4 Wave E1 Integration Suite (v0.36.0 -> v0.37.0)
//!
//! Covers:
//! 1. Extended Client CRM (`contacts`, `notes_md`, `tax_rate_pct`, `payment_terms`) & Engagement SOW
//!    (`workspace_prefix`, `sow_rules`, `rollover_mode`, `period_start_ms`, `period_end_ms`, `archived_ms`)
//!    with `PATCH /api/clients/:id`, `PATCH /api/engagements/:id`, and `GET /api/engagements/:id/context`.
//! 2. `time_entries.jsonl` (`0o600`) CRUD (`GET/POST/DELETE /api/time-entries`) and `invoices.json` (`0o600`)
//!    generation (`POST /api/invoices/generate`) producing real PDF/XLSX/JSON files + lifecycle
//!    (`PATCH /api/invoices/:id`: `draft -> issued -> paid -> void`).
//! 3. `VaultChunk` & `IndexedFile` scoping via `workspace_prefix` & `.ergon-scope.json` across
//!    `GET /api/search`, `search_vault`, `GET /api/files`, and `POST /api/vault/aggregate`.
//! 4. Engagement-scoped `POST /api/export/handoff`, `GET /api/files/pack`, and
//!    `audit_bundle::build_handoff_bundle_zip_with_chain` returning 409 on cross-client path references
//!    under `Strict Client`.
//! 5. `system_prompt` active client/engagement context injection and `WorkspacePolicy.isolate_active_engagement`
//!    prefix containment in `tools.rs` (preserving `shared/` access and blocking symlink escapes).

use ergon_rs::{audit_bundle, docs, library, records, search, serve, system_prompt, tools, Config, Person};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
            "client-vault-v040-{}-{}-{name}",
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

fn http_req_raw(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String, Vec<u8>) {
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
    let head_str = String::from_utf8_lossy(&raw[..sep]).to_string();
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_bytes = raw[sep + 4..].to_vec();
    (status, head_str, body_bytes)
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let (status, _, bytes) = http_req_raw(addr, method, path, body);
    let val = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, val)
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
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 0 {
        let mut buf = vec![0u8; content_length.min(4 * 1024 * 1024)];
        let _ = reader.read_exact(&mut buf);
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}

#[cfg(unix)]
fn assert_mode_600(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0o600 on {}", path.display());
}

#[test]
fn client_crm_engagement_sow_patch_and_context_e2e() {
    let td = TestDir::new("crm-patch-ctx");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/brief.md"),
        "# Acme Q4 Brief\nDeliverables ready.\n",
    )
    .unwrap();
    let addr = spawn_server(td.cfg.clone());

    let (st, created_c) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({
            "id": "acme",
            "name": "Acme Corp",
            "brand_header": "Acme Confidential",
            "billing_currency": "USD",
            "default_rate_usd": 200.0
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{created_c}");

    let (st, patched_c) = http_req(
        addr,
        "PATCH",
        "/api/clients/acme",
        &json!({
            "contacts": [
                { "name": "Dana Vance", "email": "dana@acme.example", "role": "VP Ops", "phone": "555-0101", "primary": true },
                { "name": "Lee Chen", "email": "lee@acme.example", "role": "Finance", "primary": false }
            ],
            "notes_md": "Prefer weekly PDF executive packets.\nCC Dana on invoices.",
            "tax_rate_pct": 8.25,
            "payment_terms": "Net 15"
        })
        .to_string(),
    );
    assert_eq!(st, 200, "{patched_c}");
    assert_eq!(patched_c["client"]["tax_rate_pct"], json!(8.25));
    assert_eq!(patched_c["client"]["payment_terms"], json!("Net 15"));
    assert_eq!(patched_c["client"]["contacts"].as_array().unwrap().len(), 2);
    assert_eq!(patched_c["client"]["contacts"][0]["name"], json!("Dana Vance"));

    let (st, created_e) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "id": "acme-q4",
            "client_id": "acme",
            "name": "Q4 Automation Retainer",
            "retainer_hours": 40.0,
            "retainer_budget_usd": 8000.0,
            "hourly_rate_usd": 200.0,
            "workspace_prefix": "clients/acme",
            "sow_rules": ["Include [file:L##] citations in every report"],
            "rollover_mode": "carry_hours",
            "activate": true
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{created_e}");
    assert_eq!(created_e["engagement"]["workspace_prefix"], json!("clients/acme"));
    assert_eq!(created_e["engagement"]["rollover_mode"], json!("carry_hours"));

    // Reject path traversal in workspace_prefix
    let (bad_st, _) = http_req(
        addr,
        "PATCH",
        "/api/engagements/acme-q4",
        &json!({ "workspace_prefix": "../secrets" }).to_string(),
    );
    assert_eq!(bad_st, 400);

    // Patch SOW rules and period window
    let (st, patched_e) = http_req(
        addr,
        "PATCH",
        "/api/engagements/acme-q4",
        &json!({
            "sow_rules": [
                "Include [file:L##] citations in every report",
                "Never share raw host paths in deliverables"
            ],
            "period_start_ms": 1_700_000_000_000u64,
            "period_end_ms": 1_900_000_000_000u64
        })
        .to_string(),
    );
    assert_eq!(st, 200, "{patched_e}");
    assert_eq!(patched_e["engagement"]["sow_rules"].as_array().unwrap().len(), 2);

    let (st, ctx_val) = http_req(addr, "GET", "/api/engagements/acme-q4/context", "");
    assert_eq!(st, 200, "{ctx_val}");
    assert_eq!(ctx_val["is_active"], json!(true));
    assert_eq!(ctx_val["client"]["name"], json!("Acme Corp"));
    assert_eq!(ctx_val["primary_contact"]["name"], json!("Dana Vance"));
    assert_eq!(ctx_val["workspace_prefix"], json!("clients/acme"));
    assert_eq!(ctx_val["scoped_files_count"], json!(1));
    assert!(ctx_val["prompt_block"]
        .as_str()
        .unwrap()
        .contains("Dana Vance"));

    // Archive and reactivate
    let (st, arch_e) = http_req(
        addr,
        "PATCH",
        "/api/engagements/acme-q4",
        &json!({ "status": "archived" }).to_string(),
    );
    assert_eq!(st, 200);
    assert_eq!(arch_e["engagement"]["status"], json!("archived"));
    assert!(arch_e["engagement"]["archived_ms"].as_u64().is_some());

    let (st, act_e) = http_req(
        addr,
        "PATCH",
        "/api/engagements/acme-q4",
        &json!({ "status": "active" }).to_string(),
    );
    assert_eq!(st, 200);
    assert_eq!(act_e["engagement"]["status"], json!("active"));
    assert!(act_e["engagement"]["archived_ms"].is_null());

    #[cfg(unix)]
    assert_mode_600(&records::clients_path(&td.cfg.data_dir));
}

#[test]
fn time_entries_and_invoicing_lifecycle_with_pdf_xlsx_json() {
    let td = TestDir::new("time-invoicing");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    let addr = spawn_server(td.cfg.clone());

    let (st, _) = http_req(
        addr,
        "POST",
        "/api/clients",
        &json!({
            "id": "acme",
            "name": "Acme Corp",
            "brand_header": "Acme Advisory",
            "billing_currency": "USD",
            "default_rate_usd": 150.0,
            "tax_rate_pct": 10.0,
            "payment_terms": "Net 30",
            "contacts": [{ "name": "Dana Vance", "email": "dana@acme.example", "primary": true }]
        })
        .to_string(),
    );
    assert_eq!(st, 201);

    let (st, _) = http_req(
        addr,
        "POST",
        "/api/engagements",
        &json!({
            "id": "acme-ret",
            "client_id": "acme",
            "name": "Monthly Ops",
            "retainer_hours": 20.0,
            "retainer_budget_usd": 5000.0,
            "hourly_rate_usd": 150.0,
            "workspace_prefix": "clients/acme",
            "rollover_mode": "carry_hours",
            "period_start_ms": 1_700_000_000_000u64,
            "activate": true
        })
        .to_string(),
    );
    assert_eq!(st, 201);

    // Seed prior-period time entry (< period_start_ms) to verify rollover carry_hours
    let (st, _) = http_req(
        addr,
        "POST",
        "/api/time-entries",
        &json!({
            "engagement_id": "acme-ret",
            "kind": "time",
            "description": "Prior cycle onboarding",
            "hours": 12.0,
            "rate_usd": 150.0,
            "billable": true,
            "ts_ms": 1_690_000_000_000u64
        })
        .to_string(),
    );
    assert_eq!(st, 201);

    // Log current-period time, expense, adjustment, and an extra entry to delete
    let (st, te1) = http_req(
        addr,
        "POST",
        "/api/time-entries",
        &json!({
            "engagement_id": "acme-ret",
            "kind": "time",
            "description": "Pipeline review and audit",
            "hours": 4.0,
            "billable": true
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{te1}");
    assert_eq!(te1["entry"]["amount_usd"], json!(600.0));

    let (st, te2) = http_req(
        addr,
        "POST",
        "/api/time-entries",
        &json!({
            "engagement_id": "acme-ret",
            "kind": "expense",
            "description": "Vendor API pass-through",
            "amount_usd": 75.0,
            "billable": true
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{te2}");

    let (st, te3) = http_req(
        addr,
        "POST",
        "/api/time-entries",
        &json!({
            "engagement_id": "acme-ret",
            "kind": "adjustment",
            "description": "Retainer loyalty credit",
            "amount_usd": -25.0,
            "billable": true
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{te3}");

    let (st, te_del) = http_req(
        addr,
        "POST",
        "/api/time-entries",
        &json!({
            "engagement_id": "acme-ret",
            "kind": "time",
            "description": "Duplicate entry to delete",
            "hours": 1.0,
            "billable": true
        })
        .to_string(),
    );
    assert_eq!(st, 201);
    let del_id = te_del["entry"]["id"].as_str().unwrap().to_string();
    let (st, del_res) = http_req(addr, "DELETE", &format!("/api/time-entries/{del_id}"), "");
    assert_eq!(st, 200, "{del_res}");
    assert_eq!(del_res["ok"], json!(true));

    // Seed LLM cost in usage_ledger.jsonl
    let mut llm_usage = records::RunUsage::default();
    llm_usage.add_call("gpt-4.1-mini", 10_000, 2_000, 12_000, 0, 120, 10.0);
    records::append_usage_ledger(
        &td.cfg.data_dir,
        &records::UsageLedgerEntry {
            ts_ms: records::now_millis(),
            run_id: "run-inv-1".to_string(),
            thread_id: "t-inv-1".to_string(),
            person_id: "default".to_string(),
            engagement_id: Some("acme-ret".to_string()),
            client_id: Some("acme".to_string()),
            source: "agent".to_string(),
            finished: true,
            answered: true,
            usage: llm_usage,
        },
    );

    #[cfg(unix)]
    assert_mode_600(&records::time_entries_path(&td.cfg.data_dir));

    // Verify rollover hours in context: 20h + (20 - 12 = 8h carried) = 28h effective; 4h used -> 24h left
    let (st, ctx_val) = http_req(addr, "GET", "/api/engagements/acme-ret/context", "");
    assert_eq!(st, 200);
    assert_eq!(ctx_val["retainer"]["rollover_hours_carried"], json!(8.0));
    assert_eq!(ctx_val["retainer"]["effective_retainer_hours"], json!(28.0));
    assert_eq!(ctx_val["retainer"]["remaining_hours"], json!(24.0));

    // Generate invoice with PDF, XLSX, JSON
    let (st, inv_res) = http_req(
        addr,
        "POST",
        "/api/invoices/generate",
        &json!({
            "engagement_id": "acme-ret",
            "period_label": "2026-11",
            "include_llm_cost": true,
            "include_time_entries": true,
            "export_formats": ["pdf", "xlsx", "json"],
            "notes": "Thank you for your business."
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{inv_res}");
    let inv = &inv_res["invoice"];
    let inv_id = inv["id"].as_str().unwrap().to_string();
    assert_eq!(inv["status"], json!("draft"));
    // Subtotal: 600 + 75 - 25 + 10 = 660.0; Tax 10% = 66.0; Total = 726.0
    assert_eq!(inv["subtotal_usd"], json!(660.0));
    assert_eq!(inv["tax_amount_usd"], json!(66.0));
    assert_eq!(inv["total_usd"], json!(726.0));
    let paths: Vec<String> = inv["deliverable_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    assert_eq!(paths.len(), 3);
    for p in &paths {
        assert!(p.starts_with("clients/acme/deliverables/invoices/"), "{p}");
    }
    let pdf_bytes = std::fs::read(td.cfg.workspace_dir.join(&paths[0])).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
    let xlsx_bytes = std::fs::read(td.cfg.workspace_dir.join(&paths[1])).unwrap();
    assert!(xlsx_bytes.starts_with(b"PK\x03\x04"));
    let sheets = docs::xlsx_sheets(&xlsx_bytes).unwrap();
    assert_eq!(sheets.len(), 2);
    let json_bytes = std::fs::read(td.cfg.workspace_dir.join(&paths[2])).unwrap();
    let json_doc: Value = serde_json::from_slice(&json_bytes).unwrap();
    assert_eq!(json_doc["total_usd"], json!(726.0));

    #[cfg(unix)]
    assert_mode_600(&records::invoices_path(&td.cfg.data_dir));

    // Invalid transition draft -> paid must return 409
    let (bad_st, _) = http_req(
        addr,
        "PATCH",
        &format!("/api/invoices/{inv_id}"),
        &json!({ "status": "paid" }).to_string(),
    );
    assert_eq!(bad_st, 409);

    // Valid lifecycle: draft -> issued -> paid -> void
    let (st, issued) = http_req(
        addr,
        "PATCH",
        &format!("/api/invoices/{inv_id}"),
        &json!({ "status": "issued" }).to_string(),
    );
    assert_eq!(st, 200);
    assert_eq!(issued["invoice"]["status"], json!("issued"));
    assert!(issued["invoice"]["issued_ms"].as_u64().is_some());

    let (st, paid) = http_req(
        addr,
        "PATCH",
        &format!("/api/invoices/{inv_id}"),
        &json!({ "status": "paid" }).to_string(),
    );
    assert_eq!(st, 200);
    assert_eq!(paid["invoice"]["status"], json!("paid"));
    assert!(paid["invoice"]["paid_ms"].as_u64().is_some());

    let (st, voided) = http_req(
        addr,
        "PATCH",
        &format!("/api/invoices/{inv_id}"),
        &json!({ "status": "void" }).to_string(),
    );
    assert_eq!(st, 200);
    assert_eq!(voided["invoice"]["status"], json!("void"));

    // Voided invoice cannot transition back to issued
    let (bad_st, _) = http_req(
        addr,
        "PATCH",
        &format!("/api/invoices/{inv_id}"),
        &json!({ "status": "issued" }).to_string(),
    );
    assert_eq!(bad_st, 409);

    // GET /api/invoices filter
    let (st, list_val) = http_req(addr, "GET", "/api/invoices?engagement_id=acme-ret&status=void", "");
    assert_eq!(st, 200);
    assert_eq!(list_val["invoices"].as_array().unwrap().len(), 1);

    // Time entries unbilled filter should now include te1..te3 again since invoice was voided
    let (st, unbilled) = http_req(addr, "GET", "/api/time-entries?engagement_id=acme-ret&unbilled=true", "");
    assert_eq!(st, 200);
    assert_eq!(unbilled["time_entries"].as_array().unwrap().len(), 4);
}

#[test]
fn vault_search_and_aggregate_scoping_e2e() {
    let td = TestDir::new("search-agg-scope");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/beta")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("extras")).unwrap();

    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/contract.md"),
        "# Acme MSA\nConfidential renewal clause omega_token_991.\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/beta/contract.md"),
        "# Beta MSA\nConfidential renewal clause omega_token_991.\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("extras/acme_notes.md"),
        "# Extra Notes\nTagged via .ergon-scope.json omega_token_991.\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join(".ergon-scope.json"),
        json!({
            "files": {
                "extras/acme_notes.md": { "client_id": "acme", "engagement_id": "acme-q4" }
            }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/spend.csv"),
        "vendor,amount\nCloud,100.00\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/beta/spend.csv"),
        "vendor,amount\nCloud,999.00\n",
    )
    .unwrap();

    let addr = spawn_server(td.cfg.clone());
    for (cid, cname, eid, pfx, activate) in [
        ("acme", "Acme Corp", "acme-q4", "clients/acme", true),
        ("beta", "Beta LLC", "beta-q4", "clients/beta", false),
    ] {
        assert_eq!(
            http_req(addr, "POST", "/api/clients", &json!({ "id": cid, "name": cname }).to_string()).0,
            201
        );
        assert_eq!(
            http_req(
                addr,
                "POST",
                "/api/engagements",
                &json!({ "id": eid, "client_id": cid, "name": eid, "workspace_prefix": pfx, "activate": activate }).to_string(),
            )
            .0,
            201
        );
    }

    // Verify IndexedFile & VaultChunk carry client_id & engagement_id
    let idx = search::SearchCache::global().reindex(
        "default",
        &td.cfg.data_dir,
        &td.cfg.workspace_dir,
        &td.cfg.memory_path,
    );
    let extra_file = idx.files.iter().find(|f| f.path == "extras/acme_notes.md").unwrap();
    assert_eq!(extra_file.client_id.as_deref(), Some("acme"));
    assert_eq!(extra_file.engagement_id.as_deref(), Some("acme-q4"));
    let beta_chunk = idx.chunks.iter().find(|c| c.path == "clients/beta/contract.md").unwrap();
    assert_eq!(beta_chunk.client_id.as_deref(), Some("beta"));
    assert_eq!(beta_chunk.engagement_id.as_deref(), Some("beta-q4"));

    // GET /api/search filtered by engagement_id returns only Acme + .ergon-scope.json Acme files
    let (st, res_a) = http_req(
        addr,
        "GET",
        "/api/search?q=omega_token_991&scope=files&engagement_id=acme-q4",
        "",
    );
    assert_eq!(st, 200);
    let paths_a: Vec<&str> = res_a["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths_a.contains(&"clients/acme/contract.md"), "{paths_a:?}");
    assert!(paths_a.contains(&"extras/acme_notes.md"), "{paths_a:?}");
    assert!(!paths_a.contains(&"clients/beta/contract.md"), "{paths_a:?}");

    // POST /api/vault/aggregate scoped by engagement_id only sums Acme's 100.00 (never Beta's 999.00)
    let (st, agg_a) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({
            "glob": "**/*.csv",
            "engagement_id": "acme-q4",
            "group_by": ["vendor"],
            "aggregates": [{ "col": "amount", "fn": "sum", "as": "total" }]
        })
        .to_string(),
    );
    assert_eq!(st, 200, "{agg_a}");
    assert_eq!(agg_a["rows"][0][1], json!("100"));

    // Cross-client glob under explicit engagement scope is rejected (403)
    let (st, _) = http_req(
        addr,
        "POST",
        "/api/vault/aggregate",
        &json!({
            "glob": "clients/beta/*.csv",
            "engagement_id": "acme-q4",
            "aggregates": [{ "col": "amount", "fn": "sum", "as": "total" }]
        })
        .to_string(),
    );
    assert_eq!(st, 403);

    // Enable strict_client -> active engagement isolation automatically scopes GET /api/search and search_vault!
    let (st, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        &json!({ "profile": "strict_client" }).to_string(),
    );
    assert_eq!(st, 200);

    let (st, res_iso) = http_req(addr, "GET", "/api/search?q=omega_token_991&scope=files", "");
    assert_eq!(st, 200);
    let paths_iso: Vec<&str> = res_iso["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(!paths_iso.contains(&"clients/beta/contract.md"), "{paths_iso:?}");
    assert!(paths_iso.contains(&"clients/acme/contract.md"), "{paths_iso:?}");

    let sv_out = tools::execute_tool(
        &td.cfg,
        "search_vault",
        &json!({ "query": "omega_token_991", "limit": 10 }),
    )
    .unwrap();
    assert!(sv_out.contains("clients/acme/contract.md"), "{sv_out}");
    assert!(!sv_out.contains("clients/beta/contract.md"), "{sv_out}");
}

#[test]
fn handoff_pack_and_zip_cross_client_409_under_strict_client() {
    let td = TestDir::new("handoff-cross-client");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/beta")).unwrap();

    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/report.md"),
        "# Acme Report\nAll metrics verified.\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/beta/secret.md"),
        "# Beta Secret\nPrivate Beta data.\n",
    )
    .unwrap();

    let addr = spawn_server(td.cfg.clone());
    assert_eq!(
        http_req(addr, "POST", "/api/clients", &json!({ "id": "acme", "name": "Acme" }).to_string()).0,
        201
    );
    assert_eq!(
        http_req(addr, "POST", "/api/clients", &json!({ "id": "beta", "name": "Beta" }).to_string()).0,
        201
    );
    assert_eq!(
        http_req(
            addr,
            "POST",
            "/api/engagements",
            &json!({ "id": "acme-q4", "client_id": "acme", "name": "Acme Q4", "workspace_prefix": "clients/acme", "activate": true }).to_string(),
        )
        .0,
        201
    );
    assert_eq!(
        http_req(
            addr,
            "POST",
            "/api/engagements",
            &json!({ "id": "beta-q4", "client_id": "beta", "name": "Beta Q4", "workspace_prefix": "clients/beta" }).to_string(),
        )
        .0,
        201
    );
    assert_eq!(
        http_req(
            addr,
            "PUT",
            "/api/workspace/policy",
            &json!({ "profile": "strict_client" }).to_string(),
        )
        .0,
        200
    );

    // 1. Clean scoped handoff export succeeds and omits clients/beta/secret.md
    let (st, _, zip_bytes) = http_req_raw(
        addr,
        "POST",
        "/api/export/handoff",
        &json!({ "engagement_id": "acme-q4", "title": "Acme Q4 Handoff" }).to_string(),
    );
    assert_eq!(st, 200);
    assert!(zip_bytes.starts_with(b"PK\x03\x04"));

    // 2. Direct call to build_handoff_bundle_zip_with_chain blocks cross-client path reference in summary
    let person = Person {
        id: "default".into(),
        name: "You".into(),
        role: "Operations".into(),
    };
    let strict = ergon_rs::load_workspace_policy(&td.cfg.data_dir);
    let sec = json!({"events_sha256": "0".repeat(64), "events": []});
    let blocked = audit_bundle::build_handoff_bundle_zip_with_chain(
        &td.cfg,
        &person,
        "Acme Handoff",
        "See [clients/beta/secret.md:L1] for details.",
        &sec,
        &strict,
        false,
        None,
        true,
    )
    .err()
    .expect("strict_client must block cross-client citation");
    assert!(blocked.scan.cross_client_count >= 1);

    // 3. Cross-client reference inside Acme's file returns HTTP 409 on both /api/export/handoff and /api/files/pack
    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/report.md"),
        "# Acme Report\nLeaked ref [clients/beta/secret.md:L1]\n",
    )
    .unwrap();
    let (st_exp, exp_err) = http_req(
        addr,
        "POST",
        "/api/export/handoff",
        &json!({ "engagement_id": "acme-q4" }).to_string(),
    );
    assert_eq!(st_exp, 409, "{exp_err}");
    assert!(exp_err["scan"]["cross_client_count"].as_u64().unwrap() >= 1);

    let (st_pack, pack_err) = http_req(
        addr,
        "GET",
        "/api/files/pack?engagement_id=acme-q4&files=clients/acme/report.md",
        "",
    );
    assert_eq!(st_pack, 409, "{pack_err}");
    assert!(pack_err["cross_client_count"].as_u64().unwrap() >= 1);

    let (st_pack2, _) = http_req(
        addr,
        "GET",
        "/api/files/pack?engagement_id=acme-q4&files=clients/beta/secret.md",
        "",
    );
    assert_eq!(st_pack2, 409);
}

#[test]
fn system_prompt_injection_and_tool_prefix_containment() {
    let td = TestDir::new("prompt-and-containment");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/beta")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("shared")).unwrap();

    std::fs::write(
        td.cfg.workspace_dir.join("clients/acme/plan.md"),
        "Acme plan line 1\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("clients/beta/secret.md"),
        "Beta secret line 1\n",
    )
    .unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("shared/templates.md"),
        "Shared SOP template\n",
    )
    .unwrap();
    std::fs::write(td.cfg.workspace_dir.join("root_notes.md"), "Root note\n").unwrap();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            "../beta/secret.md",
            td.cfg.workspace_dir.join("clients/acme/escape.md"),
        )
        .unwrap();
    }

    records::create_client(
        &td.cfg.data_dir,
        &json!({
            "id": "acme",
            "name": "Acme Global",
            "contacts": [{ "name": "Dana Vance", "email": "dana@acme.example", "role": "CTO", "primary": true }]
        }),
    )
    .unwrap();
    records::create_engagement(
        &td.cfg.data_dir,
        "default",
        &json!({
            "id": "acme-q4",
            "client_id": "acme",
            "name": "Q4 Security Audit",
            "retainer_hours": 30.0,
            "retainer_budget_usd": 6000.0,
            "hourly_rate_usd": 200.0,
            "workspace_prefix": "clients/acme",
            "sow_rules": ["Cite every control finding with [file:L##]"],
            "activate": true
        }),
    )
    .unwrap();

    // 1. system_prompt includes active client/engagement block
    let prompt = system_prompt(&td.cfg, &[]);
    assert!(prompt.contains("Acme Global"), "{prompt}");
    assert!(prompt.contains("Dana Vance <dana@acme.example>"), "{prompt}");
    assert!(prompt.contains("Cite every control finding with [file:L##]"), "{prompt}");
    assert!(prompt.contains("clients/acme"), "{prompt}");
    assert!(prompt.contains("6000.00 USD remaining"), "{prompt}");

    // 2. Also verify live agent run sends active engagement block in system message to OpenAI gateway
    let captured_system = Arc::new(Mutex::new(String::new()));
    let cap_clone = captured_system.clone();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mock_addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let body = read_http_request_body(&stream);
            if let Ok(v) = serde_json::from_str::<Value>(&body) {
                if let Some(sys) = v["messages"][0]["content"].as_str() {
                    *cap_clone.lock().unwrap() = sys.to_string();
                }
            }
            let resp_body = json!({
                "model": "mock-pro",
                "choices": [{"message": {"content": "{\"done\": true, \"summary\": \"Completed check.\"}"}}],
                "usage": {"prompt_tokens": 50, "completion_tokens": 20, "total_tokens": 70}
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                resp_body.len(),
                resp_body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    let mut live_cfg = td.cfg.clone();
    live_cfg.model_base_url = format!("http://{mock_addr}/v1");
    let lib = Arc::new(Mutex::new(library::Library::default()));
    let ctx = tools::ToolCtx {
        library: lib,
        root_data_dir: td.cfg.data_dir.clone(),
        person_id: "default".to_string(),
    };
    let store: ergon_rs::SharedStore = Arc::new(Mutex::new(ergon_rs::Store::default()));
    let tid = store.lock().unwrap().create_thread();
    ergon_rs::run_agent(&live_cfg, &ctx, &store, &tid, "Check status", &[]).unwrap();
    let sent_sys = captured_system.lock().unwrap().clone();
    assert!(sent_sys.contains("Acme Global"), "{sent_sys}");
    assert!(sent_sys.contains("Dana Vance"), "{sent_sys}");

    // 3. Enable isolate_active_engagement and verify tools containment
    ergon_rs::save_workspace_policy_with_options(&td.cfg.data_dir, Some("fast_internal"), Some(true))
        .unwrap();

    // Allowed inside clients/acme/ and shared/
    let ok_acme = tools::execute_tool(&td.cfg, "read_file", &json!({ "path": "clients/acme/plan.md" }))
        .unwrap();
    assert!(ok_acme.contains("Acme plan line 1"));
    let ok_shared = tools::execute_tool(&td.cfg, "read_file", &json!({ "path": "shared/templates.md" }))
        .unwrap();
    assert!(ok_shared.contains("Shared SOP template"));
    let ok_write = tools::execute_tool(
        &td.cfg,
        "write_file",
        &json!({ "path": "clients/acme/out.md", "content": "ok\n" }),
    )
    .unwrap();
    assert!(ok_write.contains("wrote clients/acme/out.md"));
    let ok_ls_root = tools::execute_tool(&td.cfg, "list_dir", &json!({ "path": "." })).unwrap();
    assert!(ok_ls_root.contains("clients/acme/"), "{ok_ls_root}");
    assert!(ok_ls_root.contains("shared/"), "{ok_ls_root}");
    assert!(!ok_ls_root.contains("beta"), "{ok_ls_root}");

    // Blocked outside clients/acme/ and shared/
    let err_beta = tools::execute_tool(&td.cfg, "read_file", &json!({ "path": "clients/beta/secret.md" }))
        .unwrap_err();
    assert!(err_beta.contains("engagement isolation"), "{err_beta}");
    let err_root = tools::execute_tool(&td.cfg, "read_file", &json!({ "path": "root_notes.md" }))
        .unwrap_err();
    assert!(err_root.contains("engagement isolation"), "{err_root}");
    let err_write = tools::execute_tool(
        &td.cfg,
        "write_file",
        &json!({ "path": "clients/beta/pwn.md", "content": "x" }),
    )
    .unwrap_err();
    assert!(err_write.contains("engagement isolation"), "{err_write}");
    let err_mv = tools::execute_tool(
        &td.cfg,
        "move_file",
        &json!({ "from": "clients/acme/out.md", "to": "clients/beta/out.md" }),
    )
    .unwrap_err();
    assert!(err_mv.contains("engagement isolation"), "{err_mv}");
    let err_ls = tools::execute_tool(&td.cfg, "list_dir", &json!({ "path": "clients/beta" }))
        .unwrap_err();
    assert!(err_ls.contains("engagement isolation"), "{err_ls}");
    let err_del = tools::execute_tool_full(
        &td.cfg,
        &ctx,
        "delete_file",
        &json!({ "path": "clients/beta/secret.md" }),
    )
    .unwrap_err();
    assert!(err_del.contains("engagement isolation"), "{err_del}");

    #[cfg(unix)]
    {
        let err_symlink = tools::execute_tool(
            &td.cfg,
            "read_file",
            &json!({ "path": "clients/acme/escape.md" }),
        )
        .unwrap_err();
        assert!(err_symlink.contains("engagement isolation"), "{err_symlink}");
    }
}
