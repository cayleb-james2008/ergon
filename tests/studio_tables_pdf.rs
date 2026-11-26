//! End-to-end tests for Ergon E2 Gap #2:
//! - Native pure-Rust PDF 1.4 export (`markdown_to_pdf` + `pdf_to_text` round-trip)
//! - Multi-sheet `.xlsx` export with real `<f>SUM(...)</f>` formulas & cached `<v>` totals
//! - `PUT /api/files/content` & `PATCH /api/files/content` optimistic lock `409` + `.history/` restore
//! - `patch_file` tool unique/ambiguous match & `replace_all`
//! - `query_table` & `POST /api/tables/query` 10,000-row rollup latency, exact decimal totals,
//!   `.csv` / `.tsv` / `.xlsx#SheetName` / `.json` loading, and `save_as`
//! - CWE-22 / CWE-59 traversal & symlink refusal across `/api/files/content`, `/api/tables/query`, `/api/files/export`

use ergon_rs::{docs, execute_tool, serve, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
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
        let root = base
            .join("test-data")
            .join(format!("studio-e2-{name}-{}-{seq}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:1".to_string(),
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
    let req = if body.is_empty() && matches!(method, "GET" | "HEAD") {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
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

#[test]
fn pdf_export_round_trips_through_pdf_to_text_and_api() {
    let td = TestDir::new("pdf-rt");
    let mut md = String::from(
        "# Q4 Client Advisory\n\n\
         Prepared for **Northstar Holdings** — reconciled figures and variance commentary.\n\n\
         | Category | Budget | Actual |\n\
         | --- | ---: | ---: |\n\
         | Cloud | $4,000.00 | $3,850.25 |\n\
         | Ops | $1,500.00 | $1,420.75 |\n\n",
    );
    for i in 1..=50 {
        md.push_str(&format!(
            "{i}. Verified vendor invoice #{i:03} against master statement.\n"
        ));
    }
    td.write_ws("reports/q4-brief.md", md.as_bytes());

    let addr = spawn_server(td.cfg.clone());
    let (st, exported) = http_req(
        addr,
        "POST",
        "/api/files/export",
        &json!({
            "path": "reports/q4-brief.md",
            "format": "pdf",
            "brand_header": "Ergon Advisory Desk"
        })
        .to_string(),
    );
    assert_eq!(st, 201, "export pdf response: {exported}");
    assert_eq!(exported["path"], "reports/q4-brief.pdf");

    let pdf_bytes = std::fs::read(td.cfg.workspace_dir.join("reports/q4-brief.pdf")).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-1.4\n"));
    let pdf_ascii = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_ascii.contains("/BaseFont /Helvetica\n") || pdf_ascii.contains("/BaseFont /Helvetica "));
    assert!(pdf_ascii.contains("\nxref\n"));
    assert!(pdf_ascii.contains("%%EOF"));

    let (extracted, pages) = docs::pdf_to_text(pdf_bytes).unwrap();
    assert!(pages >= 2, "expected multi-page PDF, got {pages}");
    assert!(extracted.contains("Q4 Client Advisory"), "{extracted}");
    assert!(extracted.contains("Ergon Advisory Desk"), "{extracted}");
    assert!(extracted.contains("$3,850.25"), "{extracted}");
    assert!(extracted.contains("Page 1 of"), "{extracted}");
}

#[test]
fn multisheet_xlsx_export_emits_sum_formulas_and_cached_totals() {
    let td = TestDir::new("xlsx-multi");
    td.write_ws(
        "q4-spend.csv",
        b"Date,Vendor,Amount\n2026-10-01,Acme,$1250.50\n2026-10-02,Beta,249.50\n2026-10-03,Gamma,(100.00)\n",
    );
    td.write_ws(
        "notes.csv",
        b"Item,Count\nInvoices,3\nDisputes,1\n",
    );

    let addr = spawn_server(td.cfg.clone());
    let (st, res) = http_req(
        addr,
        "POST",
        "/api/files/export",
        &json!({
            "path": "Q4-Board-Pack.xlsx",
            "format": "xlsx",
            "sheets": [
                { "name": "Spend", "path": "q4-spend.csv", "totals_row": true },
                { "name": "Summary", "path": "notes.csv", "totals_row": true }
            ]
        })
        .to_string(),
    );
    assert_eq!(st, 201, "export xlsx: {res}");
    assert_eq!(res["path"], "Q4-Board-Pack.xlsx");

    let xlsx_bytes = std::fs::read(td.cfg.workspace_dir.join("Q4-Board-Pack.xlsx")).unwrap();
    let zip = docs::ZipReader::open(&xlsx_bytes).unwrap();
    let sheet1_xml = zip.read_string("xl/worksheets/sheet1.xml").unwrap();
    assert!(sheet1_xml.contains("<f>SUM(C2:C4)</f><v>1400</v>"), "{sheet1_xml}");
    let sheet2_xml = zip.read_string("xl/worksheets/sheet2.xml").unwrap();
    assert!(sheet2_xml.contains("<f>SUM(B2:B3)</f><v>4</v>"), "{sheet2_xml}");

    let sheets = docs::xlsx_sheets(&xlsx_bytes).unwrap();
    assert_eq!(sheets.len(), 2);
    assert_eq!(sheets[0].name, "Spend");
    assert_eq!(sheets[0].rows[4], vec!["Total", "", "1400"]);
    assert_eq!(sheets[1].name, "Summary");
    assert_eq!(sheets[1].rows[3], vec!["Total", "4"]);
}

#[test]
fn put_and_patch_content_enforce_optimistic_lock_and_history_restore() {
    let td = TestDir::new("studio-edit");
    td.write_ws("draft.md", b"# Draft\n\nLine two.\nLine three.\n");

    let addr = spawn_server(td.cfg.clone());

    let (st, status_json) = http_req(addr, "GET", "/api/status", "");
    assert_eq!(st, 200);
    assert_eq!(status_json["capabilities"]["file_edit"], json!(true));
    assert_eq!(status_json["capabilities"]["table_query"], json!(true));
    assert_eq!(status_json["capabilities"]["pdf_export"], json!(true));

    let (st, get_res) = http_req(addr, "GET", "/api/files/content?path=draft.md", "");
    assert_eq!(st, 200);
    let sha_v1 = get_res["sha256"].as_str().unwrap().to_string();

    // Valid PUT with matching expected_sha256 succeeds and snapshots prior version.
    let (st, put_res) = http_req(
        addr,
        "PUT",
        "/api/files/content",
        &json!({
            "path": "draft.md",
            "content": "# Draft\n\nUpdated two.\nLine three.\n",
            "expected_sha256": sha_v1,
            "reason": "inline-edit"
        })
        .to_string(),
    );
    assert_eq!(st, 200, "PUT response: {put_res}");
    let rev_v1 = put_res["rev_id"].as_str().unwrap().to_string();
    let sha_v2 = put_res["sha256"].as_str().unwrap().to_string();
    assert_ne!(sha_v1, sha_v2);

    // Stale expected_sha256 triggers 409 Conflict.
    let (st, conflict) = http_req(
        addr,
        "PUT",
        "/api/files/content",
        &json!({
            "path": "draft.md",
            "content": "stale overwrite",
            "expected_sha256": sha_v1,
        })
        .to_string(),
    );
    assert_eq!(st, 409, "expected 409 conflict: {conflict}");

    // Stale PATCH also triggers 409 Conflict.
    let (st, patch_conflict) = http_req(
        addr,
        "PATCH",
        "/api/files/content",
        &json!({
            "path": "draft.md",
            "expected_sha256": sha_v1,
            "ops": [{ "op": "replace", "find": "Updated two.", "replace": "Line 2" }]
        })
        .to_string(),
    );
    assert_eq!(st, 409, "expected 409 on PATCH: {patch_conflict}");

    // Valid PATCH with replace + replace_lines returns diff and updates sha256.
    let (st, patch_ok) = http_req(
        addr,
        "PATCH",
        "/api/files/content",
        &json!({
            "path": "draft.md",
            "expected_sha256": sha_v2,
            "ops": [
                { "op": "replace", "find": "Updated two.", "replace": "Second line." },
                { "op": "replace_lines", "start_line": 4, "end_line": 4, "replace": "Final line." }
            ]
        })
        .to_string(),
    );
    assert_eq!(st, 200, "PATCH response: {patch_ok}");
    assert_eq!(patch_ok["applied"], 2);
    assert!(patch_ok["diff"]["unified"].as_str().unwrap().contains("+Final line."));
    let live = std::fs::read_to_string(td.cfg.workspace_dir.join("draft.md")).unwrap();
    assert_eq!(live, "# Draft\n\nSecond line.\nFinal line.\n");

    // Restore back to rev_v1 via /api/files/restore.
    let (st, restored) = http_req(
        addr,
        "POST",
        "/api/files/restore",
        &json!({ "path": "draft.md", "rev_id": rev_v1 }).to_string(),
    );
    assert_eq!(st, 200, "restore response: {restored}");
    let restored_text = std::fs::read_to_string(td.cfg.workspace_dir.join("draft.md")).unwrap();
    assert_eq!(restored_text, "# Draft\n\nLine two.\nLine three.\n");
}

#[test]
fn patch_file_tool_handles_unique_ambiguous_and_replace_all() {
    let td = TestDir::new("patch-tool");
    td.write_ws("memo.md", b"Alpha\nBeta\nAlpha\nGamma\n");

    // Ambiguous match refuses without replace_all.
    let err = execute_tool(
        &td.cfg,
        "patch_file",
        &json!({ "path": "memo.md", "find": "Alpha", "replace": "Omega" }),
    )
    .unwrap_err();
    assert!(err.contains("2 matches") || err.contains("ambiguous"), "{err}");

    // Unique match succeeds and records history.
    let ok = execute_tool(
        &td.cfg,
        "patch_file",
        &json!({ "path": "memo.md", "find": "Beta", "replace": "Beta-v2" }),
    )
    .unwrap();
    assert!(ok.contains("patched memo.md (1 replacement"), "{ok}");

    // replace_all replaces all matches.
    let ok_all = execute_tool(
        &td.cfg,
        "patch_file",
        &json!({ "path": "memo.md", "find": "Alpha", "replace": "Omega", "replace_all": true }),
    )
    .unwrap();
    assert!(ok_all.contains("2 replacements"), "{ok_all}");

    let final_txt = std::fs::read_to_string(td.cfg.workspace_dir.join("memo.md")).unwrap();
    assert_eq!(final_txt, "Omega\nBeta-v2\nOmega\nGamma\n");

    let revs = ergon_rs::file_history::list_revisions(&td.cfg.data_dir, &td.cfg.workspace_dir, "memo.md").unwrap();
    assert_eq!(revs.len(), 2);
}

#[test]
fn query_table_10k_row_rollup_and_multisource_formats() {
    let td = TestDir::new("table-query");

    // Generate 10,000 rows with exact cent amounts (0.01 / 0.10 / 0.20) to verify zero float drift.
    let mut csv = String::from("id,vendor,region,amount\n");
    for i in 0..10_000 {
        let vendor = match i % 4 {
            0 => "Acme Corp",
            1 => "Beta Labs",
            2 => "Gamma Inc",
            _ => "Delta LLC",
        };
        let region = if i % 2 == 0 { "US" } else { "EU" };
        let amt = match i % 3 {
            0 => "$10.10",
            1 => "$20.20",
            _ => "$30.30",
        };
        csv.push_str(&format!("{i},{vendor},{region},{amt}\n"));
    }
    td.write_ws("ledger_10k.csv", csv.as_bytes());

    let addr = spawn_server(td.cfg.clone());
    let t0 = Instant::now();
    let (st, res) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        &json!({
            "path": "ledger_10k.csv",
            "where": [{ "col": "region", "op": "eq", "value": "US" }],
            "group_by": ["vendor"],
            "aggregates": [
                { "col": "amount", "fn": "sum", "as": "total_usd" },
                { "col": "amount", "fn": "avg", "as": "avg_usd" },
                { "col": "*", "fn": "count", "as": "rows" }
            ],
            "sort": { "col": "vendor", "dir": "asc" },
            "save_as": "us_vendor_rollup.csv"
        })
        .to_string(),
    );
    let elapsed = t0.elapsed();
    assert_eq!(st, 200, "query response: {res}");
    assert!(elapsed < Duration::from_secs(2), "10k rollup too slow: {elapsed:?}");
    assert_eq!(res["total_rows"], 10_000);
    assert_eq!(res["matched_rows"], 5_000);
    assert_eq!(res["saved_path"], "us_vendor_rollup.csv");

    // region=US has even indices i=0,2,4,...,9998 (5000 rows), split into Acme (i%4==0, 2500 rows) and Gamma (i%4==2, 2500 rows).
    // For i%4==0: i%3 cycles uniformly across 0,1,2 over 2500 rows:
    // 2500 / 3 = 833 full cycles (each 10.10 + 20.20 + 30.30 = 60.60) + 1 extra (i=0 -> 10.10) = 50489.90!
    // Wait: 833 * 60.60 = 50479.80; + 10.10 = 50489.90.
    // For i%4==2 (i=2,6,10,...): i=2 -> 30.30, 833 cycles * 60.60 + 30.30 = 50510.10.
    let rows = res["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], "Acme Corp");
    assert_eq!(rows[0][1], "50489.9");
    assert_eq!(rows[0][3], "2500");
    assert_eq!(rows[1][0], "Gamma Inc");
    assert_eq!(rows[1][1], "50510.1");
    assert_eq!(rows[1][3], "2500");
    assert!(td.cfg.workspace_dir.join("us_vendor_rollup.csv").is_file());

    // Verify XLSX path#SheetName + JSON + TSV loading through `query_table` tool.
    let wb = docs::sheets_to_xlsx(
        &[
            docs::ExportSheet {
                name: "Q1".into(),
                rows: vec![vec!["item".into(), "cost".into()], vec!["Pen".into(), "2.50".into()]],
                totals_row: false,
            },
            docs::ExportSheet {
                name: "Q4".into(),
                rows: vec![
                    vec!["item".into(), "cost".into()],
                    vec!["Server".into(), "1200.00".into()],
                    vec!["Desk".into(), "450.25".into()],
                ],
                totals_row: false,
            },
        ],
        "Expenses",
    );
    td.write_ws("expenses.xlsx", &wb);
    let q_xlsx = execute_tool(
        &td.cfg,
        "query_table",
        &json!({
            "path": "expenses.xlsx#Q4",
            "where": [{ "col": "cost", "op": "gt", "value": "500" }]
        }),
    )
    .unwrap();
    assert!(q_xlsx.contains("1 of 2 rows matched") && q_xlsx.contains("Server,1200"), "{q_xlsx}");

    td.write_ws(
        "items.json",
        br#"[{"sku":"A1","qty":3,"price":12.50},{"sku":"B2","qty":2,"price":10.00}]"#,
    );
    let q_json = execute_tool(
        &td.cfg,
        "query_table",
        &json!({
            "path": "items.json",
            "aggregates": [{ "col": "qty", "fn": "sum", "as": "total_qty" }]
        }),
    )
    .unwrap();
    assert!(q_json.contains("total_qty\n5"), "{q_json}");
}

#[test]
fn cwe22_and_cwe59_traversal_refused_across_new_endpoints() {
    let td = TestDir::new("cwe22-59");
    td.write_ws("legit.md", b"# Legit\n\nBody.\n");
    td.write_ws("legit.csv", b"a,b\n1,2\n");

    let outside = td.root.join("outside_secret.txt");
    std::fs::write(&outside, b"TOP_SECRET_DATA").unwrap();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, td.cfg.workspace_dir.join("escape_link.md")).unwrap();
        std::os::unix::fs::symlink(&outside, td.cfg.workspace_dir.join("escape_link.csv")).unwrap();
    }

    let addr = spawn_server(td.cfg.clone());

    for bad_path in [
        "../outside_secret.txt",
        "%2e%2e/outside_secret.txt",
        "/etc/passwd",
        ".hidden.md",
        "sub\\evil.md",
        #[cfg(unix)]
        "escape_link.md",
    ] {
        let (st_put, _) = http_req(
            addr,
            "PUT",
            "/api/files/content",
            &json!({ "path": bad_path, "content": "hacked" }).to_string(),
        );
        assert_eq!(st_put, 403, "PUT should reject {bad_path}");

        let (st_patch, _) = http_req(
            addr,
            "PATCH",
            "/api/files/content",
            &json!({ "path": bad_path, "ops": [{ "op": "replace", "find": "a", "replace": "b" }] }).to_string(),
        );
        assert_eq!(st_patch, 403, "PATCH should reject {bad_path}");

        let (st_export, _) = http_req(
            addr,
            "POST",
            "/api/files/export",
            &json!({ "path": bad_path, "format": "pdf" }).to_string(),
        );
        assert_eq!(st_export, 403, "export should reject source {bad_path}");
    }

    // Export output traversal & multi-sheet path traversal refused with 403.
    let (st_out, _) = http_req(
        addr,
        "POST",
        "/api/files/export",
        &json!({ "path": "legit.md", "format": "pdf", "output": "../evil.pdf" }).to_string(),
    );
    assert_eq!(st_out, 403);

    let (st_sheet_trav, _) = http_req(
        addr,
        "POST",
        "/api/files/export",
        &json!({
            "path": "ok.xlsx",
            "format": "xlsx",
            "sheets": [{ "name": "Leak", "path": "../outside_secret.csv" }]
        })
        .to_string(),
    );
    assert_eq!(st_sheet_trav, 403);

    // Table query source traversal, symlink, and save_as traversal refused with 403.
    for bad_q in ["../outside_secret.csv", "%2e%2e/secret.csv", "escape_link.csv"] {
        #[cfg(not(unix))]
        if bad_q == "escape_link.csv" {
            continue;
        }
        let (st_q, _) = http_req(
            addr,
            "POST",
            "/api/tables/query",
            &json!({ "path": bad_q }).to_string(),
        );
        assert_eq!(st_q, 403, "tables/query should reject {bad_q}");
    }

    let (st_save_trav, _) = http_req(
        addr,
        "POST",
        "/api/tables/query",
        &json!({ "path": "legit.csv", "save_as": "../pwn.csv" }).to_string(),
    );
    assert_eq!(st_save_trav, 403);

    #[cfg(unix)]
    {
        let (st_save_sym, _) = http_req(
            addr,
            "POST",
            "/api/tables/query",
            &json!({ "path": "legit.csv", "save_as": "escape_link.csv" }).to_string(),
        );
        assert_eq!(st_save_sym, 403);
    }

    // Verify outside_secret.txt was never modified.
    assert_eq!(std::fs::read(&outside).unwrap(), b"TOP_SECRET_DATA");
}
