//! v0.9.0 proofs: Ergon reads the files employees actually have (PDF,
//! Word, Excel, PowerPoint — fixtures made by real producers: python-docx,
//! pandoc, openpyxl re-saved by LibreOffice, python-pptx, reportlab and
//! Chromium; see tests/fixtures/docs/make_fixtures.py), writes share-ready
//! Word/Excel deliverables, uploads files byte-exact up to 25 MB, retries
//! transient model failures, and keeps memory free of duplicates.

use ergon_rs::{docs, execute_tool, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docs").join(name)
}

fn test_root(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    let root = target.join("test-data").join(format!("docs-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn cfg_at(root: &Path, base: &str, key: Option<&str>) -> Config {
    let data_dir = root.join("data");
    let ws = data_dir.join("workspace");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(data_dir.join("memory.md"), "# Memory\n\n").unwrap();
    Config {
        data_dir: data_dir.clone(),
        workspace_dir: ws,
        memory_path: data_dir.join("memory.md"),
        model_base_url: base.to_string(),
        model_id: "stub".into(),
        api_key: key.map(|k| k.to_string()),
    }
}

fn with_fixtures(cfg: &Config, names: &[&str]) {
    for n in names {
        std::fs::copy(fixture(n), cfg.workspace_dir.join(n)).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Reading real-world documents
// ---------------------------------------------------------------------------

#[test]
fn word_documents_read_as_structured_markdown() {
    let t = docs::docx_to_markdown(&std::fs::read(fixture("contract.docx")).unwrap()).unwrap();
    assert!(t.contains("# Master Services Agreement"), "{t}");
    assert!(t.contains("# 1. Term and Renewal"), "{t}");
    assert!(t.contains("1. Monthly fee of $4,500 payable net 30."), "{t}");
    assert!(t.contains("2. Late payments accrue 1.5% interest per month."), "{t}");
    assert!(t.contains("- Excludes gross negligence"), "{t}");
    assert!(t.contains("| Milestone | Date | Amount |\n| --- | --- | --- |\n| Kickoff | 2026-01-15 | $9,000 |"), "{t}");
    // A different producer (pandoc): real Heading styles and list numbering.
    let m = docs::docx_to_markdown(&std::fs::read(fixture("memo-pandoc.docx")).unwrap()).unwrap();
    assert!(m.contains("# Q3 Planning Memo") && m.contains("## Decisions"), "{m}");
    assert!(m.contains("- Hire two support engineers"), "{m}");
    assert!(m.contains("1. Who owns the migration?") && m.contains("2. What is the budget cap?"), "{m}");
}

#[test]
fn excel_workbooks_read_as_csv_with_real_dates_and_cached_formulas() {
    let sheets = docs::xlsx_sheets(&std::fs::read(fixture("expenses.xlsx")).unwrap()).unwrap();
    assert_eq!(sheets.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["Expenses", "Notes"]);
    let csv = docs::rows_to_csv(&sheets[0].rows);
    assert!(csv.starts_with("Date,Vendor,Category,Amount,Receipt\n2026-10-03,Acme Office Supply,Office,182.4,TRUE\n"), "{csv}");
    assert!(csv.contains(",Total,,2719.15,"), "the SUM formula's cached value is read: {csv}");
    assert!(docs::rows_to_csv(&sheets[1].rows).contains("Duplicate Skyline charge under dispute"));
}

#[test]
fn powerpoint_decks_read_slide_by_slide_with_notes() {
    let t = docs::pptx_to_text(&std::fs::read(fixture("deck.pptx")).unwrap()).unwrap();
    assert!(t.contains("## Slide 1\n\nQ4 Launch Plan\nOps review, November 2026"), "{t}");
    assert!(t.contains("Speaker notes: Open with the revenue chart."), "{t}");
    assert!(t.contains("## Slide 2\n\nMilestones\nBeta on Dec 1\nGA on Jan 15"), "{t}");
}

#[test]
fn pdfs_read_page_by_page_including_cid_fonts() {
    let (t, pages) = docs::pdf_to_text(std::fs::read(fixture("invoice-reportlab.pdf")).unwrap()).unwrap();
    assert_eq!(pages, 2);
    assert!(t.contains("## Page 1") && t.contains("INVOICE INV-2026-0042") && t.contains("$4,500.00"), "{t}");
    assert!(t.contains("## Page 2") && t.contains("Late fee 1.5% per month"), "{t}");
    // Chromium prints with CID fonts + ToUnicode maps, like most modern apps.
    let (c, _) = docs::pdf_to_text(std::fs::read(fixture("report-chromium.pdf")).unwrap()).unwrap();
    assert!(c.contains("Weekly Ops Report"), "{c}");
    assert!(c.contains("Café expansion approved — naïve estimates revised."), "accents and dashes survive: {c}");
}

#[test]
fn broken_and_hostile_documents_fail_cleanly() {
    // Garbage with a .pdf name: an error, never a panic or a hang.
    let junk = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 9 0 R >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF".to_vec();
    assert!(docs::pdf_to_text(junk).is_err());
    assert!(docs::docx_to_markdown(b"PK\x03\x04 not really a zip").is_err());
    // Zip bomb: a tiny entry that inflates past the per-entry cap.
    let bomb = vec![0u8; 70 * 1024 * 1024];
    let z = docs::zip_write(&[("word/document.xml", bomb)]);
    assert!(z.len() < 200_000, "compresses to almost nothing");
    let r = docs::ZipReader::open(&z).unwrap();
    let err = r.read("word/document.xml").unwrap_err();
    assert!(err.contains("too large"), "{err}");
}

// ---------------------------------------------------------------------------
// Everywhere the agent and the workflows read
// ---------------------------------------------------------------------------

#[test]
fn read_file_reads_documents_sheets_and_pages_long_files() {
    let root = test_root("read");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    with_fixtures(&cfg, &["contract.docx", "expenses.xlsx", "invoice-reportlab.pdf"]);
    let pdf = execute_tool(&cfg, "read_file", &json!({"path": "invoice-reportlab.pdf"})).unwrap();
    assert!(pdf.starts_with("(text extracted from a 2-page PDF)") && pdf.contains("INV-2026-0042"), "{pdf}");
    let xl = execute_tool(&cfg, "read_file", &json!({"path": "expenses.xlsx"})).unwrap();
    assert!(xl.contains("other sheets: \"Notes\"") && xl.contains("2026-10-09,Skyline Travel"), "{xl}");
    let notes = execute_tool(&cfg, "read_file", &json!({"path": "expenses.xlsx#Notes"})).unwrap();
    assert!(notes.contains("Policy: receipts required over $25"), "{notes}");
    let bad = execute_tool(&cfg, "read_file", &json!({"path": "expenses.xlsx#Nope"})).unwrap_err();
    assert!(bad.contains("no sheet named \"Nope\""), "{bad}");

    // A long file is read window by window, never silently cut.
    let long: String = (0..2000).map(|i| format!("line {i:04} of the long report\n")).collect();
    std::fs::write(cfg.workspace_dir.join("long.txt"), &long).unwrap();
    let first = execute_tool(&cfg, "read_file", &json!({"path": "long.txt"})).unwrap();
    assert!(first.contains("line 0000") && first.contains("\"offset\": 5800"), "{}", &first[first.len() - 200..]);
    let mut offset = 5800;
    let mut seen_last = false;
    for _ in 0..20 {
        let w = execute_tool(&cfg, "read_file", &json!({"path": "long.txt", "offset": offset})).unwrap();
        if w.contains("line 1999") {
            seen_last = true;
            assert!(w.contains("[end of long.txt"), "{}", &w[w.len() - 120..]);
            break;
        }
        offset += 5800;
    }
    assert!(seen_last, "every line is reachable");
    // Unreadable binaries say so plainly.
    std::fs::write(cfg.workspace_dir.join("photo.bin"), [0xffu8, 0xfe, 0x00, 0x81]).unwrap();
    let e = execute_tool(&cfg, "read_file", &json!({"path": "photo.bin"})).unwrap_err();
    assert!(e.contains("binary file"), "{e}");
}

#[test]
fn search_files_looks_inside_documents() {
    let root = test_root("search");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    with_fixtures(&cfg, &["invoice-reportlab.pdf", "expenses.xlsx", "deck.pptx"]);
    let r = execute_tool(&cfg, "search_files", &json!({"query": "late fee"})).unwrap();
    assert!(r.contains("invoice-reportlab.pdf:"), "{r}");
    let r = execute_tool(&cfg, "search_files", &json!({"query": "receipts required"})).unwrap();
    assert!(r.contains("expenses.xlsx:"), "every sheet is searched: {r}");
    let r = execute_tool(&cfg, "search_files", &json!({"query": "revenue chart"})).unwrap();
    assert!(r.contains("deck.pptx:"), "speaker notes are searched: {r}");
}

#[test]
fn built_in_workflows_run_on_word_and_excel_files() {
    let root = test_root("workflows");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    with_fixtures(&cfg, &["contract.docx", "expenses.xlsx"]);
    let r = execute_tool(
        &cfg,
        "audit_contract",
        &json!({"contract_file": "contract.docx", "required_clauses": ["termination", "liability", "renewal"], "output_title": "Contract Audit"}),
    )
    .unwrap();
    assert!(r.contains("Contract Audit.md"), "{r}");
    let audit = std::fs::read_to_string(cfg.workspace_dir.join("Contract Audit.md")).unwrap();
    assert!(audit.contains("| termination | **MISSING** |"), "{audit}");
    assert!(audit.contains("| liability | **PRESENT** |"), "{audit}");

    execute_tool(&cfg, "reconcile_ledger", &json!({"source_files": ["expenses.xlsx"], "output_title": "Oct Reconciliation"})).unwrap();
    let rec = std::fs::read_to_string(cfg.workspace_dir.join("Oct Reconciliation.md")).unwrap();
    assert!(rec.contains("| Total line items processed | 4 |"), "the Total row is not an expense: {rec}");
    assert!(rec.contains("| Combined ledger total | $2,719.15 |"), "matches the workbook's own SUM: {rec}");
    assert!(rec.contains("| Possible duplicate charges flagged | 1 |"), "{rec}");
    assert!(rec.contains("same date, vendor and amount as expenses.xlsx:L3"), "{rec}");
    assert!(rec.contains("| Summary rows skipped (not counted) | 1 |"), "{rec}");
}

#[test]
fn different_references_are_not_duplicate_charges() {
    let root = test_root("refs");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    std::fs::write(
        cfg.workspace_dir.join("ap.csv"),
        "Date,Invoice,Vendor,Amount\n2026-10-01,INV-1,Acme,100.00\n2026-10-01,INV-2,Acme,100.00\n2026-10-02,,Beta,50.00\n2026-10-02,,Beta,50.00\n",
    )
    .unwrap();
    execute_tool(&cfg, "reconcile_ledger", &json!({"source_files": ["ap.csv"], "output_title": "AP"})).unwrap();
    let rec = std::fs::read_to_string(cfg.workspace_dir.join("AP.md")).unwrap();
    assert!(rec.contains("| Possible duplicate charges flagged | 1 |"), "only the unreferenced Beta pair: {rec}");
    assert!(rec.contains("ap.csv:L5"), "{rec}");
}

// ---------------------------------------------------------------------------
// Share-ready deliverables
// ---------------------------------------------------------------------------

#[test]
fn export_file_makes_word_and_excel_copies() {
    let root = test_root("export");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    std::fs::write(cfg.workspace_dir.join("Brief.md"), "# Board Brief\n\n- Revenue **up 12%**\n\n| Region | Sales |\n| --- | ---: |\n| EMEA | 1,200 |\n").unwrap();
    std::fs::write(cfg.workspace_dir.join("spend.csv"), "Date,Vendor,Amount\n2026-10-03,Acme,$182.40\n").unwrap();
    let r = execute_tool(&cfg, "export_file", &json!({"path": "Brief.md"})).unwrap();
    assert!(r.starts_with("wrote Brief.docx (") && r.contains("a Word document"), "{r}");
    let back = docs::docx_to_markdown(&std::fs::read(cfg.workspace_dir.join("Brief.docx")).unwrap()).unwrap();
    assert!(back.contains("# Board Brief") && back.contains("- Revenue up 12%") && back.contains("| EMEA | 1,200 |"), "{back}");
    let r = execute_tool(&cfg, "export_file", &json!({"path": "spend.csv", "output": "reports/Spend"})).unwrap();
    assert!(r.starts_with("wrote reports/Spend.xlsx ("), "{r}");
    let sheets = docs::xlsx_sheets(&std::fs::read(cfg.workspace_dir.join("reports/Spend.xlsx")).unwrap()).unwrap();
    assert_eq!(sheets[0].rows[1], vec!["2026-10-03".to_string(), "Acme".into(), "182.4".into()]);
    let e = execute_tool(&cfg, "export_file", &json!({"path": "spend.csv", "format": "docx"})).unwrap_err();
    assert!(e.contains("Word export takes a .md or .txt"), "{e}");
    assert!(execute_tool(&cfg, "export_file", &json!({"path": "../escape.md"})).is_err());
}

// ---------------------------------------------------------------------------
// HTTP: uploads, previews, export, retries
// ---------------------------------------------------------------------------

fn start_app(cfg: Config) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let _ = ergon_rs::serve(listener, cfg);
    });
    port
}

fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, Vec<u8>) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    s.write_all(body.as_bytes()).unwrap();
    let mut raw = Vec::new();
    let _ = s.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw);
    let status = text.split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0);
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4).unwrap_or(raw.len());
    (status, raw[split..].to_vec())
}

fn json_of(b: &[u8]) -> Value {
    serde_json::from_slice(b).unwrap_or(Value::Null)
}

fn b64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for c in bytes.chunks(3) {
        let n = ((c[0] as u32) << 16) | ((*c.get(1).unwrap_or(&0) as u32) << 8) | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

#[test]
fn uploads_are_byte_exact_preview_as_text_and_export() {
    let root = test_root("http");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    let ws = cfg.workspace_dir.clone();
    let port = start_app(cfg);

    // A real Word file, uploaded the way the desk now sends every file.
    let docx = std::fs::read(fixture("contract.docx")).unwrap();
    let body = json!({"path": "contract.docx", "content": b64(&docx), "encoding": "base64"}).to_string();
    let (st, b) = http(port, "POST", "/api/files/upload", Some(&body));
    assert_eq!(st, 201, "{}", String::from_utf8_lossy(&b));
    assert_eq!(std::fs::read(ws.join("contract.docx")).unwrap(), docx, "stored byte for byte");

    // Bigger than the old 1 MB request cap: now fine on the upload route…
    let big: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
    let body = json!({"path": "scan.bin", "content": b64(&big), "encoding": "base64"}).to_string();
    let (st, _) = http(port, "POST", "/api/files/upload", Some(&body));
    assert_eq!(st, 201);
    assert_eq!(std::fs::read(ws.join("scan.bin")).unwrap(), big);
    // …and downloadable again (the 2 MiB limit is for previews only).
    let (st, got) = http(port, "GET", "/api/files/content?path=scan.bin&download=1", None);
    assert_eq!(st, 200);
    assert_eq!(got, big);
    // …while every other route keeps its 1 MB cap.
    let (st, _) = http(port, "POST", "/api/chat", Some(&format!("{{\"message\":\"{}\"}}", "x".repeat(1_200_000))));
    assert_eq!(st, 413);

    // Preview: extracted Markdown; download: the original bytes.
    let (st, b) = http(port, "GET", "/api/files/content?path=contract.docx", None);
    assert_eq!(st, 200);
    let v = json_of(&b);
    assert_eq!(v["extracted"]["format"], "markdown");
    assert!(v["content"].as_str().unwrap().contains("# 3. Liability"), "{v}");
    let (_, raw) = http(port, "GET", "/api/files/content?path=contract.docx&download=1", None);
    assert_eq!(raw, docx);

    // Workbook preview, one sheet at a time.
    std::fs::copy(fixture("expenses.xlsx"), ws.join("expenses.xlsx")).unwrap();
    let v = json_of(&http(port, "GET", "/api/files/content?path=expenses.xlsx&sheet=Notes", None).1);
    assert_eq!(v["extracted"]["format"], "csv");
    assert_eq!(v["extracted"]["sheets"], json!(["Expenses", "Notes"]));
    assert_eq!(v["extracted"]["sheet"], "Notes");

    // Save as Word from the viewer.
    std::fs::write(ws.join("Weekly.md"), "# Weekly\n\nAll good.\n").unwrap();
    let (st, b) = http(port, "POST", "/api/files/export", Some(r#"{"path":"Weekly.md"}"#));
    assert_eq!(st, 201, "{}", String::from_utf8_lossy(&b));
    assert_eq!(json_of(&b)["path"], "Weekly.docx");
    assert!(ws.join("Weekly.docx").exists());
    let (st, _) = http(port, "POST", "/api/files/export", Some(r#"{"path":"../../x.md"}"#));
    assert_eq!(st, 403);
}

struct Stub {
    port: u16,
    hits: Arc<Mutex<usize>>,
}

/// A model endpoint that answers the first `fail` calls with `status`
/// (plus Retry-After: 0), then replies normally.
fn flaky_model(fail: usize, status: u16, replies: Vec<String>) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(Mutex::new(0usize));
    let h = hits.clone();
    std::thread::spawn(move || {
        let mut ok_idx = 0;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut len = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0u8; len];
            let _ = reader.read_exact(&mut body);
            let n = {
                let mut g = h.lock().unwrap();
                *g += 1;
                *g
            };
            let resp = if n <= fail {
                let msg = "{\"error\":\"overloaded\"}";
                format!("HTTP/1.1 {status} Busy\r\nRetry-After: 0\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{msg}", msg.len())
            } else {
                let reply = replies.get(ok_idx).cloned().unwrap_or_else(|| "{\"done\": true, \"summary\": \"ok\"}".into());
                ok_idx += 1;
                let p = json!({"choices": [{"message": {"role": "assistant", "content": reply}}]}).to_string();
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{p}", p.len())
            };
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    Stub { port, hits }
}

#[test]
fn a_busy_model_costs_a_retry_not_the_task() {
    let stub = flaky_model(2, 503, vec![json!({"done": true, "summary": "Answered after the provider recovered."}).to_string()]);
    let root = test_root("retry");
    let port = start_app(cfg_at(&root, &format!("http://127.0.0.1:{}/v1", stub.port), Some("k")));
    let (st, b) = http(port, "POST", "/api/chat", Some(r#"{"thread_id":"","message":"what's on today?"}"#));
    assert_eq!(st, 200, "{}", String::from_utf8_lossy(&b));
    assert!(String::from_utf8_lossy(&b).contains("Answered after the provider recovered."));
    assert_eq!(*stub.hits.lock().unwrap(), 3, "two 503s, then success");
}

#[test]
fn a_bad_request_is_not_retried() {
    let stub = flaky_model(5, 400, vec![]);
    let root = test_root("noretry");
    let port = start_app(cfg_at(&root, &format!("http://127.0.0.1:{}/v1", stub.port), Some("k")));
    let (st, b) = http(port, "POST", "/api/chat", Some(r#"{"thread_id":"","message":"hello"}"#));
    assert_eq!(st, 500);
    assert!(String::from_utf8_lossy(&b).contains("HTTP 400"));
    assert_eq!(*stub.hits.lock().unwrap(), 1);
}

#[test]
fn retry_policy() {
    assert!(ergon_rs::retryable_status(429) && ergon_rs::retryable_status(503) && ergon_rs::retryable_status(529));
    assert!(!ergon_rs::retryable_status(400) && !ergon_rs::retryable_status(401) && !ergon_rs::retryable_status(404));
    assert_eq!(ergon_rs::retry_delay(1, Some("2")).as_millis(), 2000);
    assert_eq!(ergon_rs::retry_delay(1, Some("600")).as_millis(), 20_000, "capped");
    assert_eq!(ergon_rs::retry_delay(2, None).as_millis(), 4000);
}

// ---------------------------------------------------------------------------
// Memory hygiene
// ---------------------------------------------------------------------------

#[test]
fn the_same_lesson_is_remembered_once() {
    let root = test_root("dedupe");
    let cfg = cfg_at(&root, "http://127.0.0.1:9/v1", None);
    ergon_rs::skills::update_memory(&cfg, "Prefers metric units").unwrap();
    let again = ergon_rs::skills::update_memory(&cfg, "  prefers   metric units. ").unwrap();
    assert!(again.starts_with("already in memory"), "{again}");
    let mem = std::fs::read_to_string(&cfg.memory_path).unwrap();
    assert_eq!(mem.matches("metric units").count(), 1, "{mem}");
}

#[test]
fn long_memory_is_trimmed_for_prompts_keeping_identity_and_recent_learning() {
    let mut mem = String::from("# Memory\n\n## About\n- Name: Maya Chen\n- Role: Ops lead\n\n## Learned\n");
    for i in 0..1000 {
        mem.push_str(&format!("- Learned note number {i:04} about how Maya works\n"));
    }
    let p = ergon_rs::memory_for_prompt(&mem);
    assert!(p.chars().count() <= 10_200, "{}", p.chars().count());
    assert!(p.contains("- Name: Maya Chen") && p.contains("- Role: Ops lead"), "identity kept");
    assert!(p.contains("note number 0999"), "most recent learning kept");
    assert!(!p.contains("note number 0001"), "oldest dropped from the prompt");
    assert!(p.contains("older lines of memory are not shown here"), "and it says so");
    assert_eq!(ergon_rs::memory_for_prompt("# Memory\n\n- short\n"), "# Memory\n\n- short\n");
}
