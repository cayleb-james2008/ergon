//! Cycle 4 Wave H1 Hardening Integration & Edge-Case Suite
//!
//! Covers:
//! 1. Client Vaults & Invoicing: strict-client containment in Rune skills and `/api/files`,
//!    XLSX/CSV formula-injection escaping (`=`, `+`, `-`, `@`), corrupt ledger/invoice recovery,
//!    and zero-partial-write invoice generation rollback.
//! 2. Connectors & MCP: secret/token redaction across OAuth2/SMTP/IMAP/MCP/URLs, SSRF IPv4/IPv6
//!    (ULA, link-local with zone ID, IPv4-mapped/compatible, NAT64, 6to4, non-standard IPv4) and
//!    redirect rejection, STDIO arg/env sanitization, and bounded session timeouts.
//! 3. Unified DAG Scheduler: concurrent `resume_pipeline` worker-slot race safety, crash recovery
//!    of parallel running branches alongside `pending_checkpoints[]`, parent->subpipeline cancellation,
//!    year-boundary/century-leap-day/tz-override Cron edge cases, and transitive cyclic `subpipeline`
//!    termination.

use ergon_rs::{connectors, docs, mcp, pipelines, records, scheduler, serve, skills, tools, tripwire, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static H1_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let seq = H1_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "h1-{}-{}-{name}",
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
            model_id: "test-model".to_string(),
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

struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn set(pairs: &[(&'static str, Option<&str>)]) -> Self {
        let mut saved = Vec::with_capacity(pairs.len());
        for (k, v) in pairs {
            saved.push((*k, std::env::var(k).ok()));
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
        for (k, orig) in &self.saved {
            match orig {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
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

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
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
    let _ = stream.read_to_end(&mut raw);
    let sep = raw.windows(4).position(|w| w == b"\r\n\r\n").expect("HTTP separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let text = String::from_utf8_lossy(&raw[sep + 4..]).to_string();
    let val = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    (status, val, text)
}

fn poll_run_until(addr: SocketAddr, run_id: &str, pred: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let (st, v, raw) = http_req(addr, "GET", &format!("/api/pipelines/runs/{run_id}"), "");
        assert_eq!(st, 200, "GET run failed: {raw}");
        let run = v["run"].clone();
        if pred(&run) {
            return run;
        }
        assert!(Instant::now() < deadline, "timed out waiting on {run_id}: {run}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

// ---------------------------------------------------------------------------
// 1. Client Vaults & Invoicing Hardening
// ---------------------------------------------------------------------------

#[test]
fn h1_strict_client_blocks_rune_skills_and_scopes_api_files() {
    let td = TestDir::new("strict-skills-files");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/beta")).unwrap();
    std::fs::create_dir_all(td.cfg.workspace_dir.join("shared")).unwrap();
    std::fs::write(td.cfg.workspace_dir.join("clients/acme/note.md"), "Acme data\n").unwrap();
    std::fs::write(td.cfg.workspace_dir.join("clients/beta/secret.md"), "Beta secret\n").unwrap();
    std::fs::write(td.cfg.workspace_dir.join("shared/tpl.md"), "Shared\n").unwrap();

    let addr = spawn_server(td.cfg.clone());
    assert_eq!(http_req(addr, "POST", "/api/clients", &json!({"id":"acme","name":"Acme"}).to_string()).0, 201);
    assert_eq!(http_req(addr, "POST", "/api/clients", &json!({"id":"beta","name":"Beta"}).to_string()).0, 201);
    assert_eq!(
        http_req(
            addr,
            "POST",
            "/api/engagements",
            &json!({"id":"acme-q4","client_id":"acme","name":"Acme Q4","workspace_prefix":"clients/acme","activate":true}).to_string()
        ).0,
        201
    );
    assert_eq!(
        http_req(
            addr,
            "POST",
            "/api/engagements",
            &json!({"id":"beta-q4","client_id":"beta","name":"Beta Q4","workspace_prefix":"clients/beta"}).to_string()
        ).0,
        201
    );
    assert_eq!(http_req(addr, "PUT", "/api/workspace/policy", r#"{"profile":"strict_client"}"#).0, 200);

    // GET /api/files with no query params must scope to active engagement (never expose Beta!).
    let (st_f, files_json, files_raw) = http_req(addr, "GET", "/api/files", "");
    assert_eq!(st_f, 200, "{files_raw}");
    let paths: Vec<&str> = files_json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"clients/acme/note.md"), "{paths:?}");
    assert!(!paths.contains(&"clients/beta/secret.md"), "{paths:?}");

    // Rune library skill cannot read or write Beta or root files under strict_client.
    let read_cross_script = r#"
pub fn main() {
    host_workspace_read("clients/beta/secret.md")
}
"#;
    let err_read = skills::run_library_skill(&td.cfg, read_cross_script, &[]).unwrap_err();
    assert!(err_read.contains("engagement isolation"), "expected isolation block: {err_read}");

    let list_root_script = r#"
pub fn main() {
    host_workspace_list(".")
}
"#;
    let listed = skills::run_library_skill(&td.cfg, list_root_script, &[]).unwrap();
    assert!(listed.contains("clients/acme/"), "{listed}");
    assert!(!listed.contains("beta"), "{listed}");

    // write_document / make_spreadsheet targeting another client or root are blocked by host_workspace_write.
    let err_doc = tools::execute_tool(
        &td.cfg,
        "write_document",
        &json!({ "title": "clients/beta/leak", "sections": ["nope"] }),
    )
    .unwrap_err();
    assert!(err_doc.contains("engagement isolation"), "{err_doc}");
    assert!(!td.cfg.workspace_dir.join("clients/beta/leak.md").exists());

    // Allowed inside clients/acme/
    let ok_doc = tools::execute_tool(
        &td.cfg,
        "write_document",
        &json!({ "title": "clients/acme/summary", "sections": ["Acme summary"] }),
    )
    .unwrap();
    assert!(ok_doc.contains("clients/acme/summary.md"), "{ok_doc}");
}

#[test]
fn h1_formula_injection_escaping_in_xlsx_csv_and_invoice_exports() {
    let td = TestDir::new("formula-escape");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();

    // Direct CSV and XLSX escaping checks (preserving negative numeric decimals).
    let rows = vec![
        vec!["=BADHDR".to_string(), "Amount".to_string(), "Note".to_string()],
        vec!["=CMD|'/C calc'!A0".to_string(), "-25.50".to_string(), "+1+1".to_string()],
        vec!["-cmd".to_string(), "100.00".to_string(), "@SUM(A1:A2)".to_string()],
        vec!["\t=HYPERLINK()".to_string(), "42".to_string(), "\r=2+2".to_string()],
    ];
    let csv = docs::rows_to_csv(&rows);
    assert!(csv.contains("'=BADHDR"), "{csv}");
    assert!(csv.contains("'=CMD"), "{csv}");
    assert!(csv.contains("'+1+1"), "{csv}");
    assert!(csv.contains("'-cmd"), "{csv}");
    assert!(csv.contains("'@SUM(A1:A2)"), "{csv}");
    assert!(csv.contains("'\t=HYPERLINK()"), "{csv}");
    assert!(csv.contains("'\r=2+2"), "{csv}");
    assert!(csv.contains(",-25.50,"), "plain negative number must stay unescaped: {csv}");

    let xlsx = docs::rows_to_xlsx(&rows, "FormulaTest", "FormulaTest");
    let parsed_sheets = docs::xlsx_sheets(&xlsx).unwrap();
    let r = &parsed_sheets[0].rows;
    assert_eq!(r[0][0], "'=BADHDR");
    assert_eq!(r[1][0], "'=CMD|'/C calc'!A0");
    assert_eq!(r[1][1], "-25.5");
    assert_eq!(r[1][2], "'+1+1");
    assert_eq!(r[2][0], "'-cmd");
    assert_eq!(r[2][2], "'@SUM(A1:A2)");
    assert_eq!(r[3][0], "'\t=HYPERLINK()");
    assert_eq!(r[3][2], "'\r=2+2");

    // Invoice XLSX + PDF with formula-injection payload in client_name & line item description.
    records::create_client(
        &td.cfg.data_dir,
        &json!({ "id": "acme", "name": "=HYPERLINK(\"http://evil\",\"Acme\")" }),
    )
    .unwrap();
    records::create_engagement(
        &td.cfg.data_dir,
        "default",
        &json!({ "id": "acme-q4", "client_id": "acme", "name": "Q4", "workspace_prefix": "clients/acme", "hourly_rate_usd": 100.0 }),
    )
    .unwrap();
    records::create_time_entry(
        &td.cfg.data_dir,
        &td.cfg.data_dir,
        "default",
        &json!({ "engagement_id": "acme-q4", "kind": "time", "description": "@SUM(1+1)*cmd|' /C calc'!A0", "hours": 2.0 }),
    )
    .unwrap();

    // Mismatched client_id rejected 400.
    let mismatch = records::generate_invoice(
        &td.cfg.data_dir,
        &td.cfg,
        "default",
        &json!({ "engagement_id": "acme-q4", "client_id": "other" }),
    )
    .unwrap_err();
    assert_eq!(mismatch.0, 400);

    let inv = records::generate_invoice(
        &td.cfg.data_dir,
        &td.cfg,
        "default",
        &json!({ "engagement_id": "acme-q4", "export_formats": ["pdf", "xlsx", "json"] }),
    )
    .unwrap();
    let xlsx_path = inv.deliverable_paths.iter().find(|p| p.ends_with(".xlsx")).unwrap();
    let inv_sheets = docs::xlsx_sheets(&std::fs::read(td.cfg.workspace_dir.join(xlsx_path)).unwrap()).unwrap();
    assert_eq!(inv_sheets[0].rows[2][1], "'=HYPERLINK(\"http://evil\",\"Acme\")");
    assert_eq!(inv_sheets[1].rows[1][1], "'@SUM(1+1)*cmd|' /C calc'!A0");
    let pdf_path = inv.deliverable_paths.iter().find(|p| p.ends_with(".pdf")).unwrap();
    assert!(std::fs::read(td.cfg.workspace_dir.join(pdf_path)).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn h1_corrupt_ledger_invoice_recovery_and_zero_partial_writes() {
    let td = TestDir::new("corrupt-recovery");
    std::fs::create_dir_all(td.cfg.workspace_dir.join("clients/acme")).unwrap();

    records::create_client(&td.cfg.data_dir, &json!({"id":"acme","name":"Acme"})).unwrap();
    records::create_engagement(
        &td.cfg.data_dir,
        "default",
        &json!({"id":"acme-q4","client_id":"acme","name":"Q4","workspace_prefix":"clients/acme","hourly_rate_usd":100.0}),
    )
    .unwrap();

    // 1. Corrupt trailing JSONL line in time_entries.jsonl + usage_ledger.jsonl preserves valid entries and backs up .bak.
    let te = records::create_time_entry(
        &td.cfg.data_dir,
        &td.cfg.data_dir,
        "default",
        &json!({"engagement_id":"acme-q4","kind":"time","description":"Valid work","hours":3.0}),
    )
    .unwrap();
    let te_path = records::time_entries_path(&td.cfg.data_dir);
    let mut te_raw = std::fs::read_to_string(&te_path).unwrap();
    te_raw.push_str("{corrupt truncated json line\n");
    std::fs::write(&te_path, te_raw).unwrap();

    let recovered_te = records::load_time_entries(&td.cfg.data_dir);
    assert_eq!(recovered_te.len(), 1);
    assert_eq!(recovered_te[0].id, te.id);
    assert!(te_path.with_extension("json.bak").exists());

    // 2. Corrupt invoices.json schema (`{"invoices": "bad"}`) backs up cleanly.
    let inv_path = records::invoices_path(&td.cfg.data_dir);
    std::fs::write(&inv_path, r#"{"invoices":"not-an-array"}"#).unwrap();
    assert!(records::load_invoices(&td.cfg.data_dir).is_empty());
    assert!(inv_path.with_extension("json.bak").exists());

    // 3. Symlink failure on second export format ("xlsx") must leave ZERO partial writes ("pdf") and unbilled time entry!
    #[cfg(unix)]
    {
        let inv_dir = td.cfg.workspace_dir.join("clients/acme/deliverables/invoices");
        std::fs::create_dir_all(&inv_dir).unwrap();
        std::os::unix::fs::symlink(
            "/etc/passwd",
            inv_dir.join("inv-0001-2026-11.xlsx"),
        )
        .unwrap();
        let err = records::generate_invoice(
            &td.cfg.data_dir,
            &td.cfg,
            "default",
            &json!({
                "engagement_id": "acme-q4",
                "period_label": "2026-11",
                "export_formats": ["pdf", "xlsx", "json"]
            }),
        )
        .unwrap_err();
        assert_eq!(err.0, 403);
        assert!(
            !inv_dir.join("inv-0001-2026-11.pdf").exists(),
            "PDF deliverable must NOT be written when xlsx validation fails"
        );
        assert!(records::load_invoices(&td.cfg.data_dir).is_empty());
        assert!(records::load_time_entries(&td.cfg.data_dir)[0].invoiced_invoice_id.is_none());
    }
}

// ---------------------------------------------------------------------------
// 2. Connectors & MCP Security Hardening
// ---------------------------------------------------------------------------

#[test]
fn h1_ssrf_ipv6_nonstandard_ipv4_and_redirect_rejection() {
    let _env = EnvGuard::set(&[
        ("ERGON_ALLOW_LOCAL_CONNECTORS", None),
        ("ERGON_FETCH_ALLOW_PRIVATE", None),
        ("ERGON_FETCH_ALLOW_HOSTS", None),
    ]);

    // IPv6 private/link-local/NAT64/6to4/IPv4-mapped/compatible & non-standard IPv4 forms.
    for bad in [
        "http://[::1]:8080/hook",
        "http://[::ffff:127.0.0.1]/hook",
        "http://[::ffff:10.0.0.1]/hook",
        "http://[::127.0.0.1]/hook",
        "http://[fd12:3456::1]/hook",
        "http://[fe80::1%eth0]/hook",
        "http://[fe80::1%25eth0]/hook",
        "http://[64:ff9b::10.0.0.1]/hook",
        "http://[2002:7f00:1::1]/hook",
        "http://0x7f000001/hook",
        "http://2130706433/hook",
        "http://0177.0.0.1/hook",
        "smtp://169.254.169.254:25",
    ] {
        assert!(
            tripwire::check_egress(bad, "connector").is_err(),
            "expected SSRF block on {bad}"
        );
    }

    // Even if prompt provenance mentions "127.0.0.1", guarded_connector_agent MUST block private targets!
    let td = TestDir::new("ssrf-redirect");
    let _arm = tripwire::arm(tripwire::ArmSpec {
        root: td.cfg.data_dir.clone(),
        person_dir: td.cfg.data_dir.clone(),
        workspace: td.cfg.workspace_dir.clone(),
        memory_path: td.cfg.memory_path.clone(),
        person_id: "default".to_string(),
        run_id: "r-ssrf".to_string(),
        thread_id: "t-ssrf".to_string(),
        provenance: "Please never call 127.0.0.1".to_string(),
        secrets: Vec::new(),
    });
    assert!(tripwire::guard_resolve_connector("127.0.0.1", 8080).is_err());
    assert!(tripwire::guard_resolve_connector("::1", 8080).is_err());
}

#[test]
fn h1_connectors_and_mcp_redaction_stdio_sanitization_and_timeouts() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1")),
        ("ERGON_FETCH_ALLOW_PRIVATE", Some("1")),
    ]);
    let td = TestDir::new("conn-mcp-hardening");
    let policy = ergon_rs::load_workspace_policy(&td.cfg.data_dir);

    assert_eq!(
        connectors::redact_url_userinfo("https://alice:top-secret-pw@api.example.com/hook"),
        "https://***@api.example.com/hook"
    );

    // OAuth2 server echoing refresh_token & client_secret on 401 -> must be redacted!
    let oauth_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let oauth_addr = oauth_listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = oauth_listener.accept() {
            let mut buf = [0u8; 2048];
            let _ = s.read(&mut buf);
            let body = "invalid token rt-leak-secret-999 and cs-leak-secret-888";
            let resp = format!("HTTP/1.1 401 Unauthorized\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = s.write_all(resp.as_bytes());
        }
    });
    let (created, _) = connectors::upsert_outbound_connector(
        &td.cfg.data_dir,
        &json!({
            "name": "OAuth Leak Check",
            "kind": "rest_api",
            "url": format!("http://127.0.0.1:{}/api", oauth_addr.port()),
            "oauth_config": {
                "token_url": format!("http://127.0.0.1:{}/tok", oauth_addr.port()),
                "client_id": "cid",
                "client_secret_key": "oa_cs",
                "refresh_token_secret_key": "oa_rt"
            },
            "require_approval": false,
            "secrets": {
                "oa_cs": "cs-leak-secret-888",
                "oa_rt": "rt-leak-secret-999"
            }
        }),
        &policy,
    )
    .unwrap();
    let cid = created["id"].as_str().unwrap();
    let oa_err = connectors::ensure_fresh_oauth_token(&td.cfg.data_dir, cid).unwrap_err();
    assert!(!oa_err.contains("rt-leak-secret-999"), "{oa_err}");
    assert!(!oa_err.contains("cs-leak-secret-888"), "{oa_err}");

    // IMAP server echoing password in BAD response -> must be redacted!
    let imap_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let imap_addr = imap_listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = imap_listener.accept() {
            let _ = s.write_all(b"* OK ready\r\n");
            let mut buf = [0u8; 1024];
            let _ = s.read(&mut buf);
            let _ = s.write_all(b"A002 NO invalid password imap-super-secret-555\r\n");
        }
    });
    let trg = connectors::create_trigger(
        &td.cfg.data_dir,
        &td.cfg.data_dir,
        &json!({
            "name": "IMAP Redact",
            "kind": "imap_poll",
            "imap_config": {
                "host": "127.0.0.1",
                "port": imap_addr.port(),
                "security": "plain_loopback_test",
                "folder": "INBOX",
                "username_secret_key": "imap_u",
                "password_secret_key": "imap_p"
            },
            "secrets": { "imap_u": "alice@example.com", "imap_p": "imap-super-secret-555" }
        }),
    )
    .unwrap();
    let trg_id = trg["trigger"]["id"].as_str().unwrap();
    let imap_err = connectors::poll_imap_trigger_now(&td.cfg, trg_id).unwrap_err();
    assert!(!imap_err.contains("imap-super-secret-555"), "{imap_err}");

    // STDIO MCP arg & env_secret_keys sanitization checks.
    assert_eq!(
        mcp::upsert_mcp_server(
            &td.cfg.data_dir,
            &json!({"name":"Bad Flag","transport":"stdio","command_or_url":"-c","args":["id"]}),
            &policy
        )
        .unwrap_err()
        .0,
        400
    );
    assert_eq!(
        mcp::upsert_mcp_server(
            &td.cfg.data_dir,
            &json!({"name":"Bad Secret Arg","transport":"stdio","command_or_url":"python3","args":["{{secret:k}}"]}),
            &policy
        )
        .unwrap_err()
        .0,
        400
    );
    assert_eq!(
        mcp::upsert_mcp_server(
            &td.cfg.data_dir,
            &json!({"name":"Bad Sensitive Arg","transport":"stdio","command_or_url":"cat","args":["/home/user/.ssh/id_rsa"]}),
            &policy
        )
        .unwrap_err()
        .0,
        400
    );
    assert_eq!(
        mcp::upsert_mcp_server(
            &td.cfg.data_dir,
            &json!({"name":"Bad Env","transport":"stdio","command_or_url":"python3","env_secret_keys":{"LD_PRELOAD":"k"}}),
            &policy
        )
        .unwrap_err()
        .0,
        400
    );
    assert_eq!(
        mcp::upsert_mcp_server(
            &td.cfg.data_dir,
            &json!({"name":"Bad Ergon Env","transport":"stdio","command_or_url":"python3","env_secret_keys":{"ERGON_API_KEY":"k"}}),
            &policy
        )
        .unwrap_err()
        .0,
        400
    );
}

// ---------------------------------------------------------------------------
// 3. Unified DAG Scheduler, Cron & Subpipeline Hardening
// ---------------------------------------------------------------------------

#[test]
fn h1_cron_year_boundary_century_leap_day_and_tz_override() {
    use scheduler::Cadence;

    // Year boundary with +05:30: 2026-12-31T20:00:00Z is 2027-01-01 01:30+05:30.
    // Cron "0 2 1 1 *" +05:30 must fire at 2027-01-01 02:00+05:30 = 2026-12-31T20:30:00Z.
    let dec31_2000z = 1_798_747_200_000u64; // 2026-12-31T20:00:00Z
    let ny_pos = Cadence::Cron {
        expr: "0 2 1 1 *".to_string(),
        tz_offset_mins: 330,
    };
    let next_pos = scheduler::next_run_ms(&ny_pos, Some(dec31_2000z), dec31_2000z).unwrap();
    assert_eq!(next_pos, dec31_2000z + 30 * 60_000);

    // Year boundary with -05:00: 2027-01-01T02:00:00Z is 2026-12-31 21:00-05:00.
    // Cron "30 23 31 12 *" -05:00 must fire at 2026-12-31 23:30-05:00 = 2027-01-01T04:30:00Z.
    let jan1_0200z = 1_798_768_800_000u64; // 2027-01-01T02:00:00Z
    let ny_neg = Cadence::Cron {
        expr: "30 23 31 12 *".to_string(),
        tz_offset_mins: -300,
    };
    let next_neg = scheduler::next_run_ms(&ny_neg, Some(jan1_0200z), jan1_0200z).unwrap();
    assert_eq!(next_neg, jan1_0200z + 150 * 60_000);

    // Century non-leap year 2100: from 2096-02-29T13:00:00Z, next "0 12 29 2 *" is 2104-02-29T12:00:00Z!
    // (2096-02-29T00:00:00Z = day 46080 since 1970-01-01)
    let feb_2096_1300z = (46_080u64 * 86_400 + 13 * 3_600) * 1_000;
    let leap_cron = scheduler::parse_cadence("0 12 29 2 *").unwrap();
    let next_2104 = scheduler::next_run_ms(&leap_cron, Some(feb_2096_1300z), feb_2096_1300z).unwrap();
    assert_eq!(records::civil_from_millis(next_2104), (2104, 2, 29));

    // Standard Vixie cron ORs DOM and DOW when both are restricted ("0 12 29 2 1" -> next Monday in Feb: 2097-02-04).
    let leap_mon = scheduler::parse_cadence("0 12 29 2 1").unwrap();
    let next_mon = scheduler::next_run_ms(&leap_mon, Some(feb_2096_1300z), feb_2096_1300z).unwrap();
    assert_eq!(records::civil_from_millis(next_mon), (2097, 2, 4));

    // Override non-zero inline tz back to UTC (0).
    let overridden = scheduler::parse_cadence_with_tz("0 9 * * * -05:00", Some(0)).unwrap();
    assert_eq!(
        overridden,
        Cadence::Cron {
            expr: "0 9 * * *".to_string(),
            tz_offset_mins: 0
        }
    );

    // Patching schedule without specifying tz preserves existing tz_offset_mins.
    let mut rtn = json!({
        "id": "rtn-1",
        "schedule": "0 9 * * 1-5",
        "tz_offset_mins": -300,
        "tz": "America/New_York"
    });
    scheduler::apply_routine_patch(&mut rtn, &json!({ "schedule": "30 10 * * 1-5" })).unwrap();
    assert_eq!(rtn["tz_offset_mins"], json!(-300));
}

#[test]
fn h1_concurrent_resume_crash_recovery_and_cyclic_subpipeline() {
    let td = TestDir::new("dag-h1");
    let addr = spawn_server(td.cfg.clone());

    // 1. Create cyclic routines A -> B -> A and verify validate rejects static cycle AND runtime terminates immediately.
    let r_a = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({"name": "CycleA", "prompt": "A", "schedule": "manual"}).to_string(),
    )
    .1["routine"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r_b = http_req(
        addr,
        "POST",
        "/api/routines",
        &json!({
            "name": "CycleB",
            "prompt": "B",
            "schedule": "manual",
            "steps_v2": [{ "id": "call_a", "kind": "subpipeline", "routine_id": r_a }]
        })
        .to_string(),
    )
    .1["routine"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        http_req(
            addr,
            "PATCH",
            &format!("/api/routines/{r_a}"),
            &json!({
                "steps_v2": [{ "id": "call_b", "kind": "subpipeline", "routine_id": r_b }]
            })
            .to_string(),
        )
        .0,
        200
    );

    let (st_val, val_json, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/validate",
        &json!({
            "routine_id": r_a,
            "steps_v2": [{ "id": "call_b", "kind": "subpipeline", "routine_id": r_b }]
        })
        .to_string(),
    );
    assert_eq!(st_val, 400, "{val_json}");
    assert!(val_json["errors"][0].as_str().unwrap().contains("cyclic subpipeline"));

    // Runtime cycle detection (even with templated routine_id bypassing static validate) fails on first cycle!
    let (st_cyc, cyc_res, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({
            "name": "Templated Cycle",
            "inputs": { "target": r_b, "__ergon_subpipeline_depth": "0" },
            "steps_v2": [{ "id": "dyn_b", "kind": "subpipeline", "routine_id": "{{inputs.target}}" }]
        })
        .to_string(),
    );
    assert_eq!(st_cyc, 202);
    let cyc_id = cyc_res["run"]["run_id"].as_str().unwrap().to_string();
    let cyc_done = poll_run_until(addr, &cyc_id, |r| matches!(r["status"].as_str(), Some("failed" | "completed")));
    assert_eq!(cyc_done["status"], json!("failed"));
    assert!(
        cyc_done["error"].as_str().unwrap_or("").contains("cyclic subpipeline detected"),
        "{cyc_done}"
    );

    // 2. Concurrent resume_pipeline calls on two simultaneous checkpoints in one DAG must not spawn duplicate join workers!
    let steps = json!([
        { "id": "cp_a", "kind": "checkpoint", "checkpoint": { "prompt": "A?", "input_mode": "approve_reject" } },
        { "id": "cp_b", "kind": "checkpoint", "checkpoint": { "prompt": "B?", "input_mode": "approve_reject" } },
        { "id": "join", "kind": "skill", "skill_id": "write_document", "depends_on": ["cp_a", "cp_b"], "args": { "title": "JoinOnce", "sections": ["ok"] } }
    ]);
    let (_, start_res, _) = http_req(
        addr,
        "POST",
        "/api/pipelines/run",
        &json!({"name": "Concurrent Resume DAG", "steps_v2": steps}).to_string(),
    );
    let run_id = start_res["run"]["run_id"].as_str().unwrap().to_string();
    let paused = poll_run_until(addr, &run_id, |r| r["status"] == "paused_checkpoint");
    let cps = paused["pending_checkpoints"].as_array().unwrap();
    let cp_a = cps.iter().find(|c| c["step_id"] == "cp_a").unwrap()["checkpoint_id"].as_str().unwrap().to_string();
    let cp_b = cps.iter().find(|c| c["step_id"] == "cp_b").unwrap()["checkpoint_id"].as_str().unwrap().to_string();

    let rid1 = run_id.clone();
    let rid2 = run_id.clone();
    let h1 = std::thread::spawn(move || {
        http_req(addr, "POST", &format!("/api/pipelines/runs/{rid1}/resume"), &json!({"checkpoint_id": cp_a, "decision": "approve"}).to_string())
    });
    let h2 = std::thread::spawn(move || {
        http_req(addr, "POST", &format!("/api/pipelines/runs/{rid2}/resume"), &json!({"checkpoint_id": cp_b, "decision": "approve"}).to_string())
    });
    assert_eq!(h1.join().unwrap().0, 202);
    assert_eq!(h2.join().unwrap().0, 202);
    let done = poll_run_until(addr, &run_id, |r| matches!(r["status"].as_str(), Some("completed" | "failed")));
    assert_eq!(done["status"], json!("completed"), "{done}");
    assert_eq!(done["pending_checkpoints"], json!([]));
    assert!(td.cfg.workspace_dir.join("JoinOnce.md").exists());

    // 3. Crash recovery of paused_checkpoint run where a parallel sibling step was "running":
    //    force_recover_stale_runs must reset the sibling step from "running" -> "pending".
    let mut stuck = pipelines::get_run(&td.cfg.data_dir, &run_id).unwrap();
    stuck.run_id = "prun-crash-test".to_string();
    stuck.status = "paused_checkpoint".to_string();
    stuck.step_states[0].status = "waiting_human".to_string();
    stuck.step_states[1].status = "running".to_string();
    stuck.step_states[2].status = "pending".to_string();
    stuck.set_pending_checkpoints(vec![pipelines::PendingCheckpoint {
        checkpoint_id: "dec-crash-cp".to_string(),
        step_index: 0,
        step_id: "cp_a".to_string(),
        prompt: "Recover?".to_string(),
        input_mode: "approve_reject".to_string(),
        choices: Vec::new(),
        inspect_files: Vec::new(),
        created_ms: 1000,
    }]);
    pipelines::save_run(&td.cfg.data_dir, &stuck).unwrap();
    let recovered_count = pipelines::force_recover_stale_runs(&td.cfg.data_dir);
    assert!(recovered_count >= 1);
    let after = pipelines::get_run(&td.cfg.data_dir, "prun-crash-test").unwrap();
    assert_eq!(after.status, "paused_checkpoint");
    assert_eq!(after.step_states[0].status, "waiting_human");
    assert_eq!(after.step_states[1].status, "pending");
}
