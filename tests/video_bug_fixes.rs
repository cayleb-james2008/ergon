//! Integration tests for Gap 5 (small bugs caught uncut on video):
//! 1. `.csv` / `.tsv` tabular PDF 1.4 export (`export_file` & `POST /api/files/export`)
//!    and rejection of non-exportable binary extensions (`.zip`, `.pdf`, `.xlsx`, `.docx`, `.png`).
//! 2. `make_spreadsheet` native 2D JSON `rows`, stringified JSON `rows`, CSV-newline `rows`,
//!    and `write_document` `sections` preserving commas without naive `.split(',')` corruption.
//! 3. `HEAD /phone` (and `HEAD /`) returning `200 OK` with matching headers and empty body.

use ergon_rs::{docs, execute_tool, serve, skills, Config};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
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
        let root = base
            .join("test-data")
            .join(format!("video-bugs-{name}-{}-{seq}", std::process::id()));
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

fn http_raw(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> (u16, HashMap<String, String>, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        addr.port()
    );
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if !body.is_empty() || !matches!(method, "GET" | "HEAD") {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("\r\n");
    req.push_str(body);
    stream.write_all(req.as_bytes()).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP header/body separator");
    let head = String::from_utf8_lossy(&raw[..sep]);
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    (status, headers, raw[sep + 4..].to_vec())
}

fn http_json(addr: SocketAddr, method: &str, path: &str, body: &Value) -> (u16, Value) {
    let payload = if matches!(method, "GET" | "HEAD") {
        String::new()
    } else {
        body.to_string()
    };
    let (status, _, bytes) = http_raw(addr, method, path, &[], &payload);
    let parsed = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, parsed)
}

#[test]
fn csv_and_tsv_export_to_valid_pdf_14_and_reject_unsupported_extensions() {
    let td = TestDir::new("csv-pdf");
    td.write_ws(
        "field-expenses.csv",
        b"Department,Vendor,Code,Qty,Amount\n\
          Field Ops,\"Acme, Inc.\",SKU_001,2,$1250.50\n\
          Admin,Northstar Supply,SKU_002,5,$499.75\n\
          Project Mgmt,Summit Tools,2*3_KIT,1,$540.00\n",
    );
    td.write_ws("summary.tsv", b"Region\tJobs\tTotal\nNorth\t12\t$8,400.00\nSouth\t9\t$6,150.00\n");
    td.write_ws("archive.zip", b"PK\x03\x04fakezip");
    td.write_ws("photo.png", b"\x89PNG\r\n\x1a\nfakepng");
    td.write_ws("existing.pdf", b"%PDF-1.4\n%%EOF\n");

    let addr = spawn_server(td.cfg.clone());

    // 1. POST /api/files/export format=pdf on .csv succeeds (201) with real PDF 1.4 table
    let (st_csv, res_csv) = http_json(
        addr,
        "POST",
        "/api/files/export",
        &json!({
            "path": "field-expenses.csv",
            "format": "pdf",
            "brand_header": "Operations Ledger Export"
        }),
    );
    assert_eq!(st_csv, 201, "CSV -> PDF export must succeed: {res_csv}");
    assert_eq!(res_csv["path"], "field-expenses.pdf");

    let pdf_bytes = std::fs::read(td.cfg.workspace_dir.join("field-expenses.pdf")).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-1.4\n"), "missing PDF 1.4 header");
    let pdf_ascii = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_ascii.contains("\nxref\n"));
    assert!(pdf_ascii.contains("%%EOF"));

    let (extracted, pages) = docs::pdf_to_text(pdf_bytes).unwrap();
    assert_eq!(pages, 1);
    for expected in [
        "Operations Ledger Export",
        "Department",
        "Vendor",
        "Amount",
        "Acme, Inc.",
        "SKU_001",
        "$1250.50",
        "Northstar Supply",
        "2*3_KIT",
        "$540.00",
    ] {
        assert!(
            extracted.contains(expected),
            "CSV PDF missing '{expected}' in extracted text:\n{extracted}"
        );
    }

    // 2. Direct tool call export_file on .csv -> pdf also works cleanly
    let tool_out = execute_tool(
        &td.cfg,
        "export_file",
        &json!({ "path": "field-expenses.csv", "format": "pdf", "output": "tool-export.pdf" }),
    )
    .expect("export_file tool should export .csv to .pdf");
    assert!(tool_out.contains("tool-export.pdf"), "{tool_out}");
    let tool_pdf = std::fs::read(td.cfg.workspace_dir.join("tool-export.pdf")).unwrap();
    assert!(tool_pdf.starts_with(b"%PDF-1.4\n"));

    // 3. TSV -> PDF also exports cleanly
    let (st_tsv, res_tsv) = http_json(
        addr,
        "POST",
        "/api/files/export",
        &json!({ "path": "summary.tsv", "format": "pdf" }),
    );
    assert_eq!(st_tsv, 201, "TSV -> PDF export must succeed: {res_tsv}");
    let tsv_pdf = std::fs::read(td.cfg.workspace_dir.join("summary.pdf")).unwrap();
    let (tsv_text, _) = docs::pdf_to_text(tsv_pdf).unwrap();
    assert!(tsv_text.contains("North") && tsv_text.contains("$8,400.00"), "{tsv_text}");

    // 4. Non-exportable binary/existing-PDF extensions still reject with HTTP 400
    for bad in ["archive.zip", "photo.png", "existing.pdf"] {
        let (st_bad, res_bad) = http_json(
            addr,
            "POST",
            "/api/files/export",
            &json!({ "path": bad, "format": "pdf" }),
        );
        assert_eq!(st_bad, 400, "expected 400 for {bad}: {res_bad}");
    }
}

#[test]
fn make_spreadsheet_accepts_native_2d_stringified_json_and_csv_rows() {
    let td = TestDir::new("spreadsheet-rows");

    // (a) Native 2D JSON array rows via dispatch_builtin_skill
    let native_res = skills::dispatch_builtin_skill(
        &td.cfg,
        "make_spreadsheet",
        &json!({
            "title": "Native-2D-Report",
            "columns": ["Item", "Vendor", "Qty", "Amount"],
            "rows": [
                ["Valve Kit", "Acme, Inc.", 2, "150.00"],
                ["Pipe Clamp", "Summit Tools", 5, "75.50"]
            ]
        }),
    )
    .expect("make_spreadsheet is built-in")
    .expect("native 2D rows must succeed");
    assert!(native_res.contains("2 data rows"), "{native_res}");
    let native_csv = std::fs::read_to_string(td.cfg.workspace_dir.join("Native-2D-Report.csv")).unwrap();
    assert!(native_csv.contains("Item,Vendor,Qty,Amount\n"), "{native_csv}");
    assert!(native_csv.contains("Valve Kit,\"Acme, Inc.\",2,150.00\n"), "{native_csv}");
    assert!(native_csv.contains("Pipe Clamp,Summit Tools,5,75.50\n"), "{native_csv}");

    // (b) Stringified 2D JSON array rows (and stringified JSON columns with embedded comma)
    let stringified_res = skills::dispatch_builtin_skill(
        &td.cfg,
        "make_spreadsheet",
        &json!({
            "title": "Stringified-Rows",
            "columns": r#"["Vendor, Legal Name", "Qty"]"#,
            "rows": r#"[["Alpha, LLC", "3"], ["Beta Corp", "7"]]"#
        }),
    )
    .expect("make_spreadsheet is built-in")
    .expect("stringified 2D JSON rows must succeed");
    assert!(stringified_res.contains("2 data rows"), "{stringified_res}");
    let str_csv = std::fs::read_to_string(td.cfg.workspace_dir.join("Stringified-Rows.csv")).unwrap();
    assert!(str_csv.contains("\"Vendor, Legal Name\",Qty\n"), "{str_csv}");
    assert!(str_csv.contains("\"Alpha, LLC\",3\n"), "{str_csv}");
    assert!(str_csv.contains("Beta Corp,7\n"), "{str_csv}");

    // (c) Stringified 2D JSON array with headers inside rows and no separate columns
    let rows_only_res = execute_tool(
        &td.cfg,
        "make_spreadsheet",
        &json!({
            "title": "Rows-Only-Tool",
            "rows": r#"[["Item","Qty"],["A","2"],["B","4"]]"#
        }),
    )
    .expect("execute_tool make_spreadsheet must accept stringified 2D rows");
    assert!(rows_only_res.contains("3 data rows"), "{rows_only_res}");
    let rows_only_csv = std::fs::read_to_string(td.cfg.workspace_dir.join("Rows-Only-Tool.csv")).unwrap();
    assert_eq!(rows_only_csv, "Item,Qty\nA,2\nB,4\n");

    // (d) CSV-like newline string rows preserving RFC-4180 quoted commas
    let csv_str_res = skills::dispatch_builtin_skill(
        &td.cfg,
        "make_spreadsheet",
        &json!({
            "title": "CSV-String-Rows",
            "columns": ["Client", "Hours", "Total"],
            "rows": "\"Cascade, Industrial\",3.5,647.50\nNorthstar,2.0,370.00"
        }),
    )
    .expect("make_spreadsheet is built-in")
    .expect("CSV newline rows string must succeed");
    assert!(csv_str_res.contains("2 data rows"), "{csv_str_res}");
    let csv_str_out = std::fs::read_to_string(td.cfg.workspace_dir.join("CSV-String-Rows.csv")).unwrap();
    assert!(csv_str_out.contains("\"Cascade, Industrial\",3.5,647.50\n"), "{csv_str_out}");
    assert!(csv_str_out.contains("Northstar,2.0,370.00\n"), "{csv_str_out}");

    // (e) End-to-end pipeline API: make_spreadsheet with 2D rows -> export PDF
    let addr = spawn_server(td.cfg.clone());
    let (st_run, run_res) = http_json(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "routine_name": "Q4 Parts Rollup",
            "steps_v2": [
                {
                    "id": "sheet",
                    "label": "Build Q4 CSV",
                    "kind": "skill",
                    "skill_id": "make_spreadsheet",
                    "args": {
                        "title": "Q4-Parts-Rollup",
                        "columns": ["Item", "Supplier", "Qty", "Total"],
                        "rows": [
                            ["Conduit 10ft", "Acme, Inc.", "12", "$144.00"],
                            ["Junction Box", "Northstar", "8", "$96.00"]
                        ]
                    },
                    "when": "always"
                },
                {
                    "id": "pdf",
                    "label": "Export PDF",
                    "kind": "export",
                    "depends_on": ["sheet"],
                    "export": {
                        "path": "{{steps.sheet.files[0]}}",
                        "format": "pdf"
                    },
                    "when": "on_previous_ok"
                }
            ]
        }),
    );
    assert!(
        st_run == 200 || st_run == 201 || st_run == 202,
        "pipeline run launch failed ({st_run}): {run_res}"
    );
    let run_id = run_res["run_id"]
        .as_str()
        .or_else(|| run_res["run"]["run_id"].as_str())
        .expect("run_id in pipeline response")
        .to_string();

    let mut final_status = String::new();
    for _ in 0..80 {
        let (st_poll, poll) = http_json(addr, "GET", &format!("/api/pipelines/runs/{run_id}"), &Value::Null);
        if st_poll == 200 {
            let status = poll["run"]["status"]
                .as_str()
                .or_else(|| poll["status"].as_str())
                .unwrap_or("")
                .to_string();
            if matches!(status.as_str(), "done" | "completed" | "failed" | "error") {
                final_status = status;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        final_status == "done" || final_status == "completed",
        "expected pipeline completion, got '{final_status}'"
    );

    let generated_csv =
        std::fs::read_to_string(td.cfg.workspace_dir.join("Q4-Parts-Rollup.csv")).unwrap();
    assert!(generated_csv.contains("Conduit 10ft,\"Acme, Inc.\",12,$144.00"), "{generated_csv}");
    let generated_pdf = std::fs::read(td.cfg.workspace_dir.join("Q4-Parts-Rollup.pdf")).unwrap();
    assert!(generated_pdf.starts_with(b"%PDF-1.4\n"));
    let (pdf_text, _) = docs::pdf_to_text(generated_pdf).unwrap();
    assert!(pdf_text.contains("Conduit 10ft") && pdf_text.contains("Acme, Inc.") && pdf_text.contains("$144.00"), "{pdf_text}");
}

#[test]
fn write_document_preserves_commas_in_sections_native_stringified_and_plain() {
    let td = TestDir::new("write-doc-sections");

    // (a) Native JSON array with commas inside sentences
    let res_native = skills::dispatch_builtin_skill(
        &td.cfg,
        "write_document",
        &json!({
            "title": "Comma-Safe-Native",
            "sections": [
                "## Summary\nReviewed Q1, Q2, and Q3 schedules.",
                "## Owners\nAssigned to Maya, Devon, and Chris."
            ]
        }),
    )
    .unwrap()
    .expect("write_document native array must succeed");
    assert!(res_native.contains("Comma-Safe-Native.md"), "{res_native}");
    let md_native = std::fs::read_to_string(td.cfg.workspace_dir.join("Comma-Safe-Native.md")).unwrap();
    assert!(md_native.contains("Reviewed Q1, Q2, and Q3 schedules."), "{md_native}");
    assert!(md_native.contains("Assigned to Maya, Devon, and Chris."), "{md_native}");

    // (b) Stringified JSON array (as sent when pasting/serializing JSON)
    let res_json_str = skills::dispatch_builtin_skill(
        &td.cfg,
        "write_document",
        &json!({
            "title": "Comma-Safe-Stringified",
            "sections": r###"["## Findings\nAlpha, Beta, and Gamma verified.", "## Next Steps\nConfirm pricing, delivery, and warranty."]"###
        }),
    )
    .unwrap()
    .expect("write_document stringified JSON array must succeed");
    assert!(res_json_str.contains("Comma-Safe-Stringified.md"), "{res_json_str}");
    let md_str = std::fs::read_to_string(td.cfg.workspace_dir.join("Comma-Safe-Stringified.md")).unwrap();
    assert!(md_str.contains("Alpha, Beta, and Gamma verified."), "{md_str}");
    assert!(md_str.contains("Confirm pricing, delivery, and warranty."), "{md_str}");

    // (c) Plain Markdown prose string containing commas (must NOT split on commas!)
    let res_plain = skills::dispatch_builtin_skill(
        &td.cfg,
        "write_document",
        &json!({
            "title": "Comma-Safe-Plain",
            "sections": "## Notes\nOne, two, and three items checked in one sentence."
        }),
    )
    .unwrap()
    .expect("write_document plain prose string must succeed");
    assert!(res_plain.contains("Comma-Safe-Plain.md"), "{res_plain}");
    let md_plain = std::fs::read_to_string(td.cfg.workspace_dir.join("Comma-Safe-Plain.md")).unwrap();
    assert!(
        md_plain.contains("One, two, and three items checked in one sentence."),
        "plain prose section must not be split on commas:\n{md_plain}"
    );
}

#[test]
fn head_phone_and_root_return_200_with_matching_headers_and_empty_body() {
    let td = TestDir::new("head-phone");
    let addr = spawn_server(td.cfg.clone());

    for route in ["/phone", "/"] {
        let (st_get, hdrs_get, body_get) = http_raw(addr, "GET", route, &[], "");
        assert_eq!(st_get, 200, "GET {route} must return 200");
        assert!(!body_get.is_empty(), "GET {route} must return HTML body");

        let (st_head, hdrs_head, body_head) = http_raw(addr, "HEAD", route, &[], "");
        assert_eq!(st_head, 200, "HEAD {route} must return 200 OK");
        assert!(
            body_head.is_empty(),
            "HEAD {route} must return empty body, got {} bytes",
            body_head.len()
        );
        assert_eq!(
            hdrs_head.get("content-type").map(String::as_str),
            Some("text/html; charset=utf-8"),
            "HEAD {route} content-type mismatch"
        );
        assert_eq!(
            hdrs_head.get("content-length"),
            hdrs_get.get("content-length"),
            "HEAD {route} content-length must match GET"
        );
        assert_eq!(
            hdrs_head.get("content-length").and_then(|v| v.parse::<usize>().ok()),
            Some(body_get.len()),
            "HEAD {route} content-length must equal actual GET body length"
        );
        assert_eq!(hdrs_head.get("etag"), hdrs_get.get("etag"), "HEAD {route} ETag must match GET");
        assert_eq!(
            hdrs_head.get("cache-control").map(String::as_str),
            Some("no-cache"),
            "HEAD {route} Cache-Control mismatch"
        );
        assert_eq!(
            hdrs_head.get("vary").map(String::as_str),
            Some("Accept-Encoding"),
            "HEAD {route} Vary mismatch"
        );

        // Conditional HEAD with matching ETag returns 304 Not Modified with empty body
        let etag = hdrs_head.get("etag").unwrap().clone();
        let (st_304, _, body_304) =
            http_raw(addr, "HEAD", route, &[("If-None-Match", etag.as_str())], "");
        assert_eq!(st_304, 304, "HEAD {route} with matching ETag must return 304");
        assert!(body_304.is_empty(), "304 response must have empty body");

        // Gzip HEAD returns 200 with Content-Encoding: gzip, matching gzip Content-Length, and empty body
        let (st_get_gz, hdrs_get_gz, body_get_gz) =
            http_raw(addr, "GET", route, &[("Accept-Encoding", "gzip")], "");
        let (st_head_gz, hdrs_head_gz, body_head_gz) =
            http_raw(addr, "HEAD", route, &[("Accept-Encoding", "gzip")], "");
        assert_eq!(st_get_gz, 200);
        assert_eq!(st_head_gz, 200);
        assert!(body_head_gz.is_empty());
        assert_eq!(hdrs_head_gz.get("content-encoding").map(String::as_str), Some("gzip"));
        assert_eq!(hdrs_head_gz.get("content-length"), hdrs_get_gz.get("content-length"));
        assert_eq!(
            hdrs_head_gz.get("content-length").and_then(|v| v.parse::<usize>().ok()),
            Some(body_get_gz.len())
        );
    }
}
