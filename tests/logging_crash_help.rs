//! Integration tests for Gap 3 (MVP.md Row 9):
//! 1. Structured rotating log files (`ergon.log` -> `ergon.log.1`..`.N` bounded) + secret redaction
//! 2. Real panic crash capture (`$ERGON_DATA_DIR/crashes/crash-*.json`) with secret redaction
//! 3. One-click IT diagnostic bundle (`GET /api/diagnostics/bundle` & `ergon diagnostics`) with 0 secret leaks
//! 4. First-run onboarding & admin contact config persistence (`GET/POST /api/help/config`)

use ergon_rs::logging::{self, LogLevel, RotatingLogger};
use ergon_rs::{serve, serve_with_gate, tripwire, Config};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let n = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!(
        "ergon-logging-help-{name}-{}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

fn test_config(root: &Path) -> Config {
    Config {
        data_dir: root.to_path_buf(),
        workspace_dir: root.join("workspace"),
        memory_path: root.join("memory.md"),
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "test-model".to_string(),
        api_key: None,
    }
}

fn start_server(cfg: Config, gate: Option<String>) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let _ = match gate {
            Some(tok) => serve_with_gate(listener, cfg, Some(tok)),
            None => serve(listener, cfg),
        };
    });
    for _ in 0..40 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    port
}

struct HttpResp {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn http_raw(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> HttpResp {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let payload = body.unwrap_or("");
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n"
    );
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if body.is_some() {
        req.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            payload.len()
        ));
    }
    req.push_str("\r\n");
    req.push_str(payload);
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP response header terminator");
    let head = String::from_utf8_lossy(&raw[..split]);
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    HttpResp {
        status,
        headers,
        body: raw[split + 4..].to_vec(),
    }
}

/// Minimal pure-Rust ZIP reader using `miniz_oxide` (zero extra test dependencies).
fn unzip_all(zip_bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    assert!(
        zip_bytes.starts_with(b"PK\x03\x04"),
        "expected ZIP local file header magic"
    );
    let mut out = BTreeMap::new();
    let mut pos = 0usize;
    while pos + 30 <= zip_bytes.len() && &zip_bytes[pos..pos + 4] == b"PK\x03\x04" {
        let method = u16::from_le_bytes([zip_bytes[pos + 8], zip_bytes[pos + 9]]);
        let comp_len = u32::from_le_bytes([
            zip_bytes[pos + 18],
            zip_bytes[pos + 19],
            zip_bytes[pos + 20],
            zip_bytes[pos + 21],
        ]) as usize;
        let name_len = u16::from_le_bytes([zip_bytes[pos + 26], zip_bytes[pos + 27]]) as usize;
        let extra_len = u16::from_le_bytes([zip_bytes[pos + 28], zip_bytes[pos + 29]]) as usize;
        let name_start = pos + 30;
        let name_end = name_start + name_len;
        let data_start = name_end + extra_len;
        let data_end = data_start + comp_len;
        let name = String::from_utf8_lossy(&zip_bytes[name_start..name_end]).into_owned();
        let compressed = &zip_bytes[data_start..data_end];
        let content = match method {
            0 => compressed.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec(compressed)
                .expect("valid deflate stream in diagnostic ZIP"),
            other => panic!("unexpected ZIP compression method {other}"),
        };
        out.insert(name, content);
        pos = data_end;
    }
    out
}

#[test]
fn rotating_logger_rotates_bounded_files_and_redacts_secrets() {
    let root = test_root("rotate");
    let logs_dir = root.join("logs");
    // 384-byte limit, keeping at most 3 rotated backups (.1 .. .3)
    let logger = RotatingLogger::new(&logs_dir, 384, 3, false);

    for i in 0..32 {
        logger.log(
            LogLevel::Info,
            "ergon::test",
            &format!(
                "entry #{i} auth Bearer tok-secret-8888 key sk-live-secret-9999 password=supersecret slack xoxb-123456-secret ERGON_MODEL_API_KEY=sk-env-9999 ERGON_GATE_TOKEN=gate-secret-777"
            ),
        );
    }

    assert!(logs_dir.join("ergon.log").is_file(), "active ergon.log must exist");
    assert!(logs_dir.join("ergon.log.1").is_file(), "ergon.log.1 must exist after rotation");
    assert!(logs_dir.join("ergon.log.2").is_file(), "ergon.log.2 must exist after rotation");
    assert!(logs_dir.join("ergon.log.3").is_file(), "ergon.log.3 must exist after rotation");
    assert!(
        !logs_dir.join("ergon.log.4").exists(),
        "rotation must stay bounded to max_backups=3"
    );

    let forbidden = [
        "tok-secret-8888",
        "sk-live-secret-9999",
        "supersecret",
        "xoxb-123456-secret",
        "sk-env-9999",
        "gate-secret-777",
        "ERGON_MODEL_API_KEY",
        "ERGON_GATE_TOKEN",
    ];
    let mut parsed_lines = 0usize;
    for name in ["ergon.log", "ergon.log.1", "ergon.log.2", "ergon.log.3"] {
        let content = std::fs::read_to_string(logs_dir.join(name)).unwrap();
        assert!(!content.is_empty(), "{name} should not be empty");
        for secret in &forbidden {
            assert!(
                !content.contains(secret),
                "secret '{secret}' leaked into {name}: {content}"
            );
        }
        for line in content.lines() {
            let v: Value = serde_json::from_str(line).expect("each log line must be valid JSON");
            assert!(v["ts_utc"].as_str().unwrap_or("").ends_with('Z'));
            assert_eq!(v["level"].as_str(), Some("INFO"));
            assert_eq!(v["target"].as_str(), Some("ergon::test"));
            assert!(v["msg"].as_str().unwrap_or("").contains("Bearer [REDACTED]"));
            parsed_lines += 1;
        }
    }
    assert!(parsed_lines >= 4, "expected structured log lines across rotated files");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn real_panic_writes_redacted_crash_report_and_error_log() {
    let bin = env!("CARGO_BIN_EXE_ergon");
    let root = test_root("panic");

    // 1. Without ERGON_ALLOW_TEST_CRASH=1, --test-crash must refuse cleanly with code 2.
    let refused = Command::new(bin)
        .args(["--data", root.to_str().unwrap(), "--test-crash"])
        .env_remove("ERGON_ALLOW_TEST_CRASH")
        .output()
        .expect("spawn ergon --test-crash without env gate");
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        !root.join("crashes").exists(),
        "gated --test-crash must not write crash reports when ERGON_ALLOW_TEST_CRASH is unset"
    );

    // 2. With ERGON_ALLOW_TEST_CRASH=1, real panic triggers the installed panic hook.
    let out = Command::new(bin)
        .args(["--data", root.to_str().unwrap(), "--test-crash"])
        .env("ERGON_ALLOW_TEST_CRASH", "1")
        .env("ERGON_LOG_STDERR", "0")
        .env(
            "ERGON_TEST_CRASH_MSG",
            "simulated failure Bearer tok-secret-8888 key=sk-live-secret-9999 password=supersecret ERGON_MODEL_API_KEY=sk-env-secret",
        )
        .output()
        .expect("spawn ergon --test-crash with ERGON_ALLOW_TEST_CRASH=1");
    assert!(
        !out.status.success(),
        "real panic must exit with non-zero status"
    );

    let crashes_dir = root.join("crashes");
    assert!(crashes_dir.is_dir(), "crashes directory must be created on panic");
    let crash_files: Vec<PathBuf> = std::fs::read_dir(&crashes_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("crash-") && n.ends_with(".json"))
        })
        .collect();
    assert_eq!(crash_files.len(), 1, "expected exactly 1 crash-*.json file");

    let crash_raw = std::fs::read_to_string(&crash_files[0]).unwrap();
    for secret in [
        "tok-secret-8888",
        "sk-live-secret-9999",
        "supersecret",
        "sk-env-secret",
        "ERGON_MODEL_API_KEY",
    ] {
        assert!(
            !crash_raw.contains(secret),
            "secret '{secret}' leaked into crash report: {crash_raw}"
        );
    }
    let crash_json: Value = serde_json::from_str(&crash_raw).expect("valid JSON crash report");
    assert_eq!(crash_json["schema"].as_str(), Some("ergon.crash.v1"));
    assert_eq!(crash_json["app_version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(crash_json["os"].as_str(), Some(std::env::consts::OS));
    assert_eq!(crash_json["arch"].as_str(), Some(std::env::consts::ARCH));
    assert_eq!(crash_json["redacted"].as_bool(), Some(true));
    assert!(crash_json["message"].as_str().unwrap_or("").contains("simulated failure"));
    assert!(crash_json["location"].as_str().unwrap_or("").contains("main.rs"));

    // Verify ERROR entry was also logged into $ERGON_DATA_DIR/logs/ergon.log with secrets redacted.
    let log_raw = std::fs::read_to_string(root.join("logs").join("ergon.log")).unwrap();
    assert!(log_raw.contains("\"level\":\"ERROR\""));
    assert!(log_raw.contains("ergon::panic"));
    assert!(!log_raw.contains("tok-secret-8888"));
    assert!(!log_raw.contains("sk-live-secret-9999"));
    assert!(!log_raw.contains("supersecret"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn diagnostic_bundle_api_and_cli_zip_contents_and_zero_secret_leaks() {
    let root = test_root("diag-bundle");
    let cfg = test_config(&root);
    cfg.ensure_dirs().unwrap();

    // Seed config.json with fake API key + credentialed URL.
    std::fs::write(
        root.join("config.json"),
        json!({
            "model_base_url": "https://service:urlpass99@api.example.com/v1",
            "model_id": "test-model",
            "api_key": "sk-live-secret-9999"
        })
        .to_string(),
    )
    .unwrap();

    // Seed logs/ergon.log and logs/ergon.log.1 containing raw fake secrets (simulating pre-existing unredacted lines too).
    let logs_dir = root.join("logs");
    std::fs::create_dir_all(&logs_dir).unwrap();
    std::fs::write(
        logs_dir.join("ergon.log"),
        "{\"ts_utc\":\"2026-11-26T10:00:00.000Z\",\"level\":\"INFO\",\"target\":\"ergon::test\",\"msg\":\"started with Bearer tok-secret-8888 and sk-live-secret-9999 password=supersecret ERGON_MODEL_API_KEY=sk-live-secret-9999 ERGON_GATE_TOKEN=gate-secret-5555\"}\n",
    )
    .unwrap();
    std::fs::write(
        logs_dir.join("ergon.log.1"),
        "{\"ts_utc\":\"2026-11-26T09:00:00.000Z\",\"level\":\"WARN\",\"target\":\"ergon::test\",\"msg\":\"slack xoxb-slack-secret-4444 conn conn-ultra-secret-7777\"}\n",
    )
    .unwrap();

    // Seed a crash report containing secrets.
    let crashes_dir = root.join("crashes");
    std::fs::create_dir_all(&crashes_dir).unwrap();
    std::fs::write(
        crashes_dir.join("crash-20261126T095500000Z-4242.json"),
        json!({
            "schema": "ergon.crash.v1",
            "ts_utc": "2026-11-26T09:55:00.000Z",
            "app_version": env!("CARGO_PKG_VERSION"),
            "message": "panic with Bearer tok-secret-8888 and password=supersecret",
            "backtrace": "frame0: ERGON_MODEL_API_KEY"
        })
        .to_string(),
    )
    .unwrap();

    // Seed a security event in security_events.json.
    let mut trip = tripwire::Trip::new(
        "TEST_DIAG",
        tripwire::Severity::Warning,
        "Test security event with Bearer tok-secret-8888",
        "Attempted probe with sk-live-secret-9999",
    )
    .action("blocked")
    .source(json!({"ip": "127.0.0.1"}));
    trip.person_id = Some("default".into());
    let _ = tripwire::record(&root, &trip);

    // Seed connectors + connector_secrets.json in default person's data dir.
    let people = ergon_rs::PeopleState::load(&cfg);
    let p_cfg = cfg.person_config(&people.active_person());
    p_cfg.ensure_dirs().unwrap();
    std::fs::write(
        ergon_rs::connectors::connectors_path(&p_cfg.data_dir),
        json!({
            "triggers": [{
                "id": "trig_webhook_1",
                "name": "Invoice Webhook",
                "kind": "webhook",
                "enabled": true,
                "target_kind": "routine",
                "routine_id": "r1",
                "hook_token": "whtok_should_not_leak_1111"
            }],
            "outbound": [{
                "id": "out_webhook_1",
                "name": "Company Mail Relay",
                "kind": "webhook_post",
                "url": "https://relay.example.com/notify",
                "method": "POST",
                "enabled": true,
                "require_approval": true,
                "secret_keys": ["smtp_pass"]
            }],
            "delivery_log": [],
            "intake_manifest": []
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        ergon_rs::connectors::connector_secrets_path(&p_cfg.data_dir),
        json!({
            "secrets": {
                "smtp_pass": "conn-ultra-secret-7777"
            },
            "webhook_secrets": {
                "trig_webhook_1": "whsec-ultra-secret-6666"
            }
        })
        .to_string(),
    )
    .unwrap();

    // A. Exercise live HTTP GET /api/diagnostics/bundle and GET /api/diagnostics/summary
    let mut live_cfg = cfg.clone();
    live_cfg.api_key = Some("sk-live-secret-9999".to_string());
    let port = start_server(live_cfg, None);

    let sum_resp = http_raw(port, "GET", "/api/diagnostics/summary", None, &[]);
    assert_eq!(sum_resp.status, 200);
    let sum_json: Value = serde_json::from_slice(&sum_resp.body).unwrap();
    assert_eq!(sum_json["app_version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(sum_json["model_configured"].as_bool(), Some(true));
    assert!(sum_json["log_files"].as_u64().unwrap_or(0) >= 2);
    assert_eq!(sum_json["crash_reports"].as_u64(), Some(1));

    let api_resp = http_raw(port, "GET", "/api/diagnostics/bundle", None, &[]);
    assert_eq!(api_resp.status, 200);
    assert_eq!(
        api_resp.headers.get("content-type").map(String::as_str),
        Some("application/zip")
    );
    let cd = api_resp
        .headers
        .get("content-disposition")
        .cloned()
        .unwrap_or_default();
    assert!(
        cd.starts_with("attachment; filename=\"ergon-diagnostics-") && cd.ends_with(".zip\""),
        "unexpected Content-Disposition: {cd}"
    );
    assert!(
        api_resp
            .headers
            .get("x-ergon-diagnostics-sha256")
            .is_some_and(|h| h.len() == 64),
        "expected 64-char SHA-256 header"
    );

    // B. Exercise CLI `ergon diagnostics --data <dir> --out <zip> --json`
    let cli_zip_path = root.join("cli-diagnostics.zip");
    let bin = env!("CARGO_BIN_EXE_ergon");
    let cli_out = Command::new(bin)
        .args([
            "diagnostics",
            "--data",
            root.to_str().unwrap(),
            "--out",
            cli_zip_path.to_str().unwrap(),
            "--json",
        ])
        .env("ERGON_LOG_STDERR", "0")
        .output()
        .expect("run ergon diagnostics CLI");
    assert!(
        cli_out.status.success(),
        "ergon diagnostics CLI failed: {}",
        String::from_utf8_lossy(&cli_out.stderr)
    );
    let cli_json: Value = serde_json::from_slice(&cli_out.stdout).expect("valid JSON CLI output");
    assert_eq!(cli_json["ok"].as_bool(), Some(true));
    let cli_zip_bytes = std::fs::read(&cli_zip_path).expect("CLI diagnostic zip readable");

    let forbidden = [
        "sk-live-secret-9999",
        "tok-secret-8888",
        "supersecret",
        "ERGON_MODEL_API_KEY",
        "ERGON_GATE_TOKEN",
        "urlpass99",
        "xoxb-slack-secret-4444",
        "conn-ultra-secret-7777",
        "whsec-ultra-secret-6666",
        "whtok_should_not_leak_1111",
    ];

    for (label, bytes) in [("HTTP API", &api_resp.body), ("CLI", &cli_zip_bytes)] {
        let entries = unzip_all(bytes);
        for required in [
            "README.txt",
            "system.json",
            "security_events.json",
            "connectors.json",
            "logs/ergon.log",
            "logs/ergon.log.1",
            "crashes/crash-20261126T095500000Z-4242.json",
        ] {
            assert!(
                entries.contains_key(required),
                "[{label}] diagnostic ZIP missing required entry '{required}' (keys: {:?})",
                entries.keys().collect::<Vec<_>>()
            );
        }

        // Verify key JSON artifacts parse cleanly after redaction.
        let sys: Value =
            serde_json::from_slice(&entries["system.json"]).expect("valid system.json");
        assert_eq!(sys["app_version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
        assert_eq!(sys["os"].as_str(), Some(std::env::consts::OS));
        assert_eq!(sys["arch"].as_str(), Some(std::env::consts::ARCH));
        assert_eq!(sys["model_configured"].as_bool(), Some(true));
        assert_eq!(sys["redaction_applied"].as_bool(), Some(true));
        assert!(sys["workspace_count"].as_u64().unwrap_or(0) >= 1);

        let sec: Value = serde_json::from_slice(&entries["security_events.json"])
            .expect("valid security_events.json");
        assert!(sec["events"].as_array().is_some_and(|a| !a.is_empty()));

        let conn: Value =
            serde_json::from_slice(&entries["connectors.json"]).expect("valid connectors.json");
        assert_eq!(conn["secrets_excluded"].as_bool(), Some(true));
        assert_eq!(conn["outbound"][0]["name"].as_str(), Some("Company Mail Relay"));
        assert_eq!(conn["outbound"][0]["enabled"].as_bool(), Some(true));

        let crash: Value =
            serde_json::from_slice(&entries["crashes/crash-20261126T095500000Z-4242.json"])
                .expect("valid redacted crash JSON");
        assert_eq!(crash["schema"].as_str(), Some("ergon.crash.v1"));

        // Zero occurrences of ANY seeded secret anywhere in the entire unzipped archive!
        for (name, data) in &entries {
            let text = String::from_utf8_lossy(data);
            for secret in &forbidden {
                assert!(
                    !text.contains(secret),
                    "[{label}] secret '{secret}' leaked inside ZIP entry '{name}': {text}"
                );
            }
        }
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn help_config_onboarding_and_admin_contact_roundtrip_and_gate_protection() {
    let root = test_root("help-config");
    let cfg = test_config(&root);
    let port = start_server(cfg.clone(), None);

    // 1. Fresh workspace starts with onboarding_completed=false & unconfigured admin contact.
    let r0 = http_raw(port, "GET", "/api/help/config", None, &[]);
    assert_eq!(r0.status, 200);
    let v0: Value = serde_json::from_slice(&r0.body).unwrap();
    assert_eq!(v0["onboarding_completed"].as_bool(), Some(false));
    assert_eq!(v0["admin_contact"]["configured"].as_bool(), Some(false));
    assert_eq!(v0["can_edit_admin_contact"].as_bool(), Some(true));

    // 2. Reject unsafe javascript: channel_url.
    let bad = http_raw(
        port,
        "POST",
        "/api/help/config",
        Some(
            &json!({
                "admin_contact": {
                    "name": "IT Support",
                    "channel_url": "javascript:alert(1)"
                }
            })
            .to_string(),
        ),
        &[],
    );
    assert_eq!(bad.status, 400);

    // 3. Persist onboarding completion + IT admin contact details.
    let save_body = json!({
        "onboarding_completed": true,
        "admin_contact": {
            "name": "Ops Helpdesk",
            "email": "it-support@example.org",
            "phone": "+1-555-010-2000",
            "channel_url": "https://support.example.org/tickets",
            "notes": "Mon-Fri 8am-6pm ET; attach the diagnostic ZIP."
        }
    })
    .to_string();
    let r1 = http_raw(port, "POST", "/api/help/config", Some(&save_body), &[]);
    assert_eq!(r1.status, 200);
    let v1: Value = serde_json::from_slice(&r1.body).unwrap();
    assert_eq!(v1["onboarding_completed"].as_bool(), Some(true));
    assert!(v1["onboarding_completed_at_ms"].as_u64().is_some());
    assert_eq!(v1["admin_contact"]["configured"].as_bool(), Some(true));
    assert_eq!(v1["admin_contact"]["name"].as_str(), Some("Ops Helpdesk"));
    assert_eq!(v1["admin_contact"]["email"].as_str(), Some("it-support@example.org"));
    assert_eq!(v1["admin_contact"]["phone"].as_str(), Some("+1-555-010-2000"));
    assert_eq!(
        v1["admin_contact"]["channel_url"].as_str(),
        Some("https://support.example.org/tickets")
    );

    // 4. Verify persistence round-trip via GET /api/help/config AND direct disk reload.
    let r2 = http_raw(port, "GET", "/api/help/config", None, &[]);
    assert_eq!(r2.status, 200);
    let v2: Value = serde_json::from_slice(&r2.body).unwrap();
    assert_eq!(v2["onboarding_completed"].as_bool(), Some(true));
    assert_eq!(v2["admin_contact"]["name"].as_str(), Some("Ops Helpdesk"));
    assert_eq!(v2["admin_contact"]["email"].as_str(), Some("it-support@example.org"));

    let reloaded = logging::load_admin_contact(&root);
    assert_eq!(reloaded.name, "Ops Helpdesk");
    assert_eq!(reloaded.email, "it-support@example.org");
    let prefs = logging::load_help_prefs(&root);
    assert!(prefs.onboarding_completed);

    // 5. Gated server: employees can update onboarding_completed, but cannot overwrite admin_contact (403).
    let gated_root = test_root("help-gated");
    let gated_port = start_server(test_config(&gated_root), Some("secret-gate-123".into()));
    let gate_hdr = [("X-Ergon-Gate", "secret-gate-123")];

    let g0 = http_raw(gated_port, "GET", "/api/help/config", None, &gate_hdr);
    assert_eq!(g0.status, 200);
    let gv0: Value = serde_json::from_slice(&g0.body).unwrap();
    assert_eq!(gv0["can_edit_admin_contact"].as_bool(), Some(false));

    let g_onboard = http_raw(
        gated_port,
        "POST",
        "/api/help/config",
        Some(&json!({"onboarding_completed": true}).to_string()),
        &gate_hdr,
    );
    assert_eq!(g_onboard.status, 200);
    let gv1: Value = serde_json::from_slice(&g_onboard.body).unwrap();
    assert_eq!(gv1["onboarding_completed"].as_bool(), Some(true));

    let g_contact = http_raw(
        gated_port,
        "POST",
        "/api/help/config",
        Some(&json!({"admin_contact": {"name": "Attacker"}}).to_string()),
        &gate_hdr,
    );
    assert_eq!(g_contact.status, 403);

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&gated_root);
}

#[test]
fn binary_boot_creates_log_file_and_reads_admin_contact_env() {
    let bin = env!("CARGO_BIN_EXE_ergon");
    let root = test_root("boot-env");

    // Pick an unused ephemeral loopback port.
    let port = {
        let probe = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        probe.local_addr().unwrap().port()
    };

    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let child = Command::new(bin)
        .args(["--no-open", "--host", "127.0.0.1", "--port", &port.to_string()])
        .env("ERGON_DATA_DIR", &root)
        .env("ERGON_NO_OPEN", "1")
        .env("ERGON_NO_AUTO_MODEL", "1")
        .env("ERGON_DISABLE_SCHEDULER", "1")
        .env("ERGON_UPDATE_MODE", "off")
        .env("ERGON_LOG_STDERR", "0")
        .env("ERGON_ADMIN_CONTACT_NAME", "Field Ops IT Desk")
        .env("ERGON_ADMIN_CONTACT_EMAIL", "it-ops@example.org")
        .env("ERGON_ADMIN_CONTACT_PHONE", "+1-555-0199")
        .env("ERGON_ADMIN_CONTACT_URL", "https://helpdesk.example.org")
        .env("ERGON_ADMIN_CONTACT_NOTES", "Include your diagnostic ZIP.")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn ergon binary");
    let _guard = ChildGuard(child);

    // Wait for server readiness.
    let mut ready = false;
    for _ in 0..80 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            ready = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(ready, "ergon server did not start listening on port {port}");

    let r = http_raw(port, "GET", "/api/help/config", None, &[]);
    assert_eq!(r.status, 200);
    let v: Value = serde_json::from_slice(&r.body).unwrap();
    assert_eq!(v["onboarding_completed"].as_bool(), Some(false));
    assert_eq!(v["admin_contact"]["configured"].as_bool(), Some(true));
    assert_eq!(v["admin_contact"]["source"].as_str(), Some("env"));
    assert_eq!(v["admin_contact"]["name"].as_str(), Some("Field Ops IT Desk"));
    assert_eq!(v["admin_contact"]["email"].as_str(), Some("it-ops@example.org"));
    assert_eq!(v["admin_contact"]["phone"].as_str(), Some("+1-555-0199"));
    assert_eq!(
        v["admin_contact"]["channel_url"].as_str(),
        Some("https://helpdesk.example.org")
    );

    let log_path = root.join("logs").join("ergon.log");
    assert!(log_path.is_file(), "ergon boot must create $ERGON_DATA_DIR/logs/ergon.log");
    let log_text = std::fs::read_to_string(&log_path).unwrap();
    let first: Value = serde_json::from_str(log_text.lines().next().unwrap_or(""))
        .expect("startup log line must be valid JSONL");
    assert_eq!(first["level"].as_str(), Some("INFO"));
    assert_eq!(first["target"].as_str(), Some("ergon::server"));
    assert!(first["msg"].as_str().unwrap_or("").contains("listening"));

    drop(_guard);
    let _ = std::fs::remove_dir_all(&root);
}

