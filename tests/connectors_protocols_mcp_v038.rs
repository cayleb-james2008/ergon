//! Integration tests for Ergon Cycle 4 Wave E2 (`v0.38.0` Gap #2):
//! - Real TCP loopback SMTP + MIME multipart/mixed base64 binary attachment wire protocol
//! - Real TCP loopback IMAP4rev1 poll -> `workspace/intake/` (.md + attachments) + subject filter + \Seen
//! - OAuth2 refresh_token HTTP flow -> 0600 `connector_secrets.json` + `{{secret:oauth_access_token}}`
//! - Stdio & HTTP/SSE MCP `discover` + dynamic prompt/tool registration + `tools/call` + `Strict Client` `mcp_tool_call` gate
//! - Gated read-only inbound MCP JSON-RPC server (`POST /api/mcp/inbound`)
//! - Tripwire SSRF blocking across SMTP, IMAP, OAuth2, and HTTP MCP

use ergon_rs::{
    connectors, execute_tool, records, serve, system_prompt, tripwire, Config,
};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
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
        let root = std::env::temp_dir().join(format!(
            "ergon-protocols-v038-{}-{}-{name}",
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
            model_base_url: "http://127.0.0.1:9".to_string(),
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

fn read_full_http_request(stream: &mut TcpStream) -> (String, String) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut raw = Vec::new();
    let mut buf = [0u8; 2048];
    let mut header_end = None;
    let mut content_len = 0usize;
    loop {
        let n = stream.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buf[..n]);
        if header_end.is_none() {
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let hdr_str = String::from_utf8_lossy(&raw[..pos]);
                for line in hdr_str.lines() {
                    if let Some((k, v)) = line.split_once(':') {
                        if k.trim().eq_ignore_ascii_case("content-length") {
                            content_len = v.trim().parse().unwrap_or(0);
                        }
                    }
                }
            }
        }
        if let Some(end) = header_end {
            if raw.len() >= end + content_len {
                break;
            }
        }
    }
    let end = header_end.unwrap_or(raw.len());
    let head = String::from_utf8_lossy(&raw[..end]).to_string();
    let body = String::from_utf8_lossy(&raw[end..raw.len().min(end + content_len)]).to_string();
    (head, body)
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

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let body_bytes = body.as_bytes();
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n", addr.port());
    if !body_bytes.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).unwrap();
    }
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_text = String::from_utf8_lossy(&raw[sep + 4..]).to_string();
    let val = serde_json::from_str::<Value>(&body_text).unwrap_or(Value::Null);
    (status, val, body_text)
}

#[derive(Default, Debug)]
struct CapturedSmtpMessage {
    mail_from: String,
    rcpt_to: Vec<String>,
    auth_plain: String,
    data: String,
}

fn spawn_mock_smtp_server() -> (SocketAddr, Arc<Mutex<Vec<CapturedSmtpMessage>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let cap_bg = captured.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let cap_conn = cap_bg.clone();
            std::thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut writer = stream.try_clone().unwrap();
                let mut reader = BufReader::new(stream);
                let _ = writer.write_all(b"220 mock-smtp.local ESMTP ready\r\n");
                let _ = writer.flush();

                let mut msg = CapturedSmtpMessage::default();
                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }
                    let trimmed = line.trim_end_matches(['\r', '\n']);
                    let upper = trimmed.to_ascii_uppercase();
                    if upper.starts_with("EHLO ") || upper.starts_with("HELO ") {
                        let _ = writer.write_all(b"250-mock-smtp.local\r\n250 AUTH PLAIN LOGIN\r\n");
                    } else if let Some(rest) = trimmed.strip_prefix("AUTH PLAIN ") {
                        msg.auth_plain = rest.trim().to_string();
                        let _ = writer.write_all(b"235 2.7.0 Authentication successful\r\n");
                    } else if upper.starts_with("MAIL FROM:") {
                        msg.mail_from = trimmed[10..].trim().to_string();
                        let _ = writer.write_all(b"250 2.1.0 Ok\r\n");
                    } else if upper.starts_with("RCPT TO:") {
                        msg.rcpt_to.push(trimmed[8..].trim().to_string());
                        let _ = writer.write_all(b"250 2.1.5 Ok\r\n");
                    } else if upper == "DATA" {
                        let _ = writer.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n");
                        let _ = writer.flush();
                        let mut data_buf = String::new();
                        loop {
                            let mut dline = String::new();
                            if reader.read_line(&mut dline).unwrap_or(0) == 0 {
                                break;
                            }
                            if dline == ".\r\n" || dline == ".\n" {
                                break;
                            }
                            if let Some(unstuffed) = dline.strip_prefix("..") {
                                data_buf.push('.');
                                data_buf.push_str(unstuffed);
                            } else {
                                data_buf.push_str(&dline);
                            }
                        }
                        msg.data = data_buf;
                        cap_conn.lock().unwrap().push(std::mem::take(&mut msg));
                        let _ = writer.write_all(b"250 2.0.0 Queued\r\n");
                    } else if upper == "QUIT" {
                        let _ = writer.write_all(b"221 2.0.0 Bye\r\n");
                        let _ = writer.flush();
                        break;
                    } else {
                        let _ = writer.write_all(b"500 unrecognized\r\n");
                    }
                    let _ = writer.flush();
                }
            });
        }
    });
    (addr, captured)
}

#[test]
fn smtp_loopback_mime_binary_attachment_and_strict_client_approval() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1")),
        ("ERGON_FETCH_ALLOW_PRIVATE", Some("1")),
    ]);
    let td = TestDir::new("smtp-mime");
    let addr = spawn_server(td.cfg.clone());
    let (smtp_addr, captured) = spawn_mock_smtp_server();

    // Write a binary attachment in workspace.
    let binary_payload: Vec<u8> = vec![
        0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x34, 0x0a, 0x00, 0xff, 0xfe, 0x42, 0x99,
        0x10, 0x20,
    ];
    std::fs::create_dir_all(td.cfg.workspace_dir.join("reports")).unwrap();
    std::fs::write(
        td.cfg.workspace_dir.join("reports/q4-brief.pdf"),
        &binary_payload,
    )
    .unwrap();

    let (st_c, c_json, c_raw) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Client SMTP Relay",
            "kind": "smtp_relay",
            "require_approval": false,
            "smtp_config": {
                "host": "127.0.0.1",
                "port": smtp_addr.port(),
                "security": "plain_loopback_test",
                "username_secret_key": "smtp_user",
                "password_secret_key": "smtp_pass",
                "from_addr": "ops@ergon.local",
                "default_to": ["client@acme.example"]
            },
            "secrets": {
                "smtp_user": "relay-user",
                "smtp_pass": "super-secret-smtp-pass-987"
            }
        })
        .to_string(),
    );
    assert_eq!(st_c, 201, "{c_raw}");
    assert!(!c_raw.contains("super-secret-smtp-pass-987"));
    let conn_id = c_json["connector"]["id"].as_str().unwrap().to_string();
    assert_eq!(c_json["connector"]["has_secret"], json!(true));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(connectors::connector_secrets_path(&td.cfg.data_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    // 1. Direct SMTP delivery with binary attachment + dot-stuffing check.
    let res = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({
            "connector_id": conn_id,
            "file": "reports/q4-brief.pdf",
            "subject": "Q4 Client Brief",
            "body": "Please review.\n.leading dot line"
        }),
    )
    .unwrap();
    assert!(res.contains("delivered via connector"), "{res}");

    {
        let msgs = captured.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        let m = &msgs[0];
        assert_eq!(m.mail_from, "<ops@ergon.local>");
        assert_eq!(m.rcpt_to, vec!["<client@acme.example>"]);
        let expected_auth = tripwire::base64_encode(b"\0relay-user\0super-secret-smtp-pass-987");
        assert_eq!(m.auth_plain, expected_auth);
        assert!(m.data.contains("Content-Type: multipart/mixed;"));
        assert!(m
            .data
            .contains("Content-Disposition: attachment; filename=\"q4-brief.pdf\""));
        assert!(m.data.contains("Content-Transfer-Encoding: base64"));
        assert!(m.data.contains(".leading dot line"));

        let expected_b64 = tripwire::base64_encode(&binary_payload);
        assert!(
            m.data.contains(&expected_b64),
            "MIME body must carry exact base64 binary bytes: {}",
            m.data
        );
    }

    // 2. Strict Client mode forces approval hold and executes SMTP with attachment on approve.
    let (st_pol, _, _) = http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    assert_eq!(st_pol, 200);

    let held = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({
            "connector_id": conn_id,
            "file": "reports/q4-brief.pdf",
            "subject": "Held SMTP Brief",
            "body": "Awaiting sign-off"
        }),
    )
    .unwrap();
    assert!(held.contains("parked for your person's approval"), "{held}");
    assert_eq!(captured.lock().unwrap().len(), 1);

    let decs = records::load_decisions(&td.cfg.data_dir);
    let dec = decs
        .iter()
        .find(|d| d["kind"] == "connector_delivery" && d["status"] == "pending")
        .unwrap();
    let dec_id = dec["id"].as_str().unwrap();

    let (st_app, app_json, app_raw) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{dec_id}/approve"),
        "",
    );
    assert_eq!(st_app, 200, "{app_raw}");
    assert!(
        app_json["outcome"]
            .as_str()
            .unwrap_or("")
            .contains("Approved and delivered"),
        "{app_json}"
    );
    assert_eq!(captured.lock().unwrap().len(), 2);
}

fn spawn_mock_imap_server() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let seen_uids = Arc::new(Mutex::new(Vec::new()));
    let seen_bg = seen_uids.clone();

    let csv_bytes = b"id,amount\n42,199.00\n";
    let csv_b64 = tripwire::base64_encode(csv_bytes);
    let email_101 = format!(
        "From: billing@vendor.example\r\n\
         To: inbox@ergon.local\r\n\
         Subject: Urgent Invoice #42\r\n\
         Date: Thu, 26 Nov 2026 12:00:00 +0000\r\n\
         Message-ID: <inv42@vendor.example>\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/mixed; boundary=\"b101\"\r\n\r\n\
         --b101\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n\
         Please process the attached invoice.\r\n\
         --b101\r\n\
         Content-Type: text/csv; name=\"invoice-42.csv\"\r\n\
         Content-Disposition: attachment; filename=\"invoice-42.csv\"\r\n\
         Content-Transfer-Encoding: base64\r\n\r\n\
         {csv_b64}\r\n\
         --b101--\r\n"
    );
    let email_102 = "From: news@digest.example\r\n\
         To: inbox@ergon.local\r\n\
         Subject: Weekly Newsletter\r\n\
         Date: Thu, 26 Nov 2026 12:05:00 +0000\r\n\
         Message-ID: <news@digest.example>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n\
         Top stories this week.\r\n"
        .to_string();

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let seen_conn = seen_bg.clone();
            let msg_101 = email_101.clone();
            let msg_102 = email_102.clone();
            std::thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut writer = stream.try_clone().unwrap();
                let mut reader = BufReader::new(stream);
                let _ = writer.write_all(b"* OK Mock IMAP4rev1 ready\r\n");
                let _ = writer.flush();

                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }
                    let trimmed = line.trim_end_matches(['\r', '\n']);
                    let mut parts = trimmed.splitn(2, ' ');
                    let tag = parts.next().unwrap_or("A0");
                    let cmd = parts.next().unwrap_or("");
                    let cmd_up = cmd.to_ascii_uppercase();

                    if cmd_up.starts_with("LOGIN ") {
                        let _ = writer.write_all(format!("{tag} OK LOGIN completed\r\n").as_bytes());
                    } else if cmd_up.starts_with("SELECT ") {
                        let _ = writer.write_all(
                            format!("* 2 EXISTS\r\n{tag} OK [READ-WRITE] SELECT completed\r\n")
                                .as_bytes(),
                        );
                    } else if cmd_up == "UID SEARCH UNSEEN" {
                        let seen = seen_conn.lock().unwrap();
                        let mut unseen = Vec::new();
                        if !seen.contains(&"101".to_string()) {
                            unseen.push("101");
                        }
                        if !seen.contains(&"102".to_string()) {
                            unseen.push("102");
                        }
                        let resp = format!(
                            "* SEARCH {}\r\n{tag} OK SEARCH completed\r\n",
                            unseen.join(" ")
                        );
                        let _ = writer.write_all(resp.as_bytes());
                    } else if cmd_up.starts_with("UID FETCH 101 ") {
                        let hdr = format!("* 1 FETCH (UID 101 BODY[] {{{}}}\r\n", msg_101.len());
                        let _ = writer.write_all(hdr.as_bytes());
                        let _ = writer.write_all(msg_101.as_bytes());
                        let _ = writer.write_all(format!(")\r\n{tag} OK FETCH completed\r\n").as_bytes());
                    } else if cmd_up.starts_with("UID FETCH 102 ") {
                        let hdr = format!("* 2 FETCH (UID 102 BODY[] {{{}}}\r\n", msg_102.len());
                        let _ = writer.write_all(hdr.as_bytes());
                        let _ = writer.write_all(msg_102.as_bytes());
                        let _ = writer.write_all(format!(")\r\n{tag} OK FETCH completed\r\n").as_bytes());
                    } else if cmd_up.starts_with("UID STORE ") {
                        if let Some(uid) = cmd.split_whitespace().nth(2) {
                            seen_conn.lock().unwrap().push(uid.to_string());
                        }
                        let _ = writer.write_all(format!("{tag} OK STORE completed\r\n").as_bytes());
                    } else if cmd_up == "LOGOUT" {
                        let _ = writer.write_all(
                            format!("* BYE\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                        );
                        let _ = writer.flush();
                        break;
                    } else {
                        let _ = writer.write_all(format!("{tag} BAD unknown\r\n").as_bytes());
                    }
                    let _ = writer.flush();
                }
            });
        }
    });
    (addr, seen_uids)
}

#[test]
fn imap_poll_writes_markdown_and_attachments_into_intake_with_subject_filter() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1")),
        ("ERGON_FETCH_ALLOW_PRIVATE", Some("1")),
    ]);
    let td = TestDir::new("imap-poll");
    let addr = spawn_server(td.cfg.clone());
    let (imap_addr, seen_uids) = spawn_mock_imap_server();

    let (st_t, t_json, t_raw) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({
            "name": "Vendor Invoice Inbox",
            "kind": "imap_poll",
            "imap_config": {
                "host": "127.0.0.1",
                "port": imap_addr.port(),
                "security": "plain_loopback_test",
                "folder": "INBOX",
                "poll_interval_secs": 30,
                "mark_seen": true,
                "username_secret_key": "imap_user",
                "password_secret_key": "imap_pass",
                "subject_filter": "Invoice"
            },
            "secrets": {
                "imap_user": "bot@example.com",
                "imap_pass": "imap-secret-99"
            }
        })
        .to_string(),
    );
    assert_eq!(st_t, 201, "{t_raw}");
    assert!(!t_raw.contains("imap-secret-99"));
    assert_eq!(t_json["trigger"]["has_secret"], json!(true));
    let trg_id = t_json["trigger"]["id"].as_str().unwrap().to_string();

    let events = connectors::poll_imap_trigger_now(&td.cfg, &trg_id).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].trigger_file.starts_with("intake/imap-"));
    assert_eq!(events[0].attachments, vec!["intake/invoice-42.csv"]);

    let md_text =
        std::fs::read_to_string(td.cfg.workspace_dir.join(&events[0].trigger_file)).unwrap();
    assert!(md_text.contains("Urgent Invoice #42"));
    assert!(md_text.contains("billing@vendor.example"));
    assert!(md_text.contains("Please process the attached invoice."));

    let csv_text =
        std::fs::read_to_string(td.cfg.workspace_dir.join("intake/invoice-42.csv")).unwrap();
    assert_eq!(csv_text, "id,amount\n42,199.00\n");

    // UID 101 marked \Seen, UID 102 filtered out and untouched.
    assert_eq!(*seen_uids.lock().unwrap(), vec!["101".to_string()]);

    // Re-poll produces 0 duplicates.
    let events2 = connectors::poll_imap_trigger_now(&td.cfg, &trg_id).unwrap();
    assert!(events2.is_empty());
}

#[test]
fn oauth2_refresh_persists_access_token_and_caches_until_expiry() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1")),
        ("ERGON_FETCH_ALLOW_PRIVATE", Some("1")),
    ]);
    let td = TestDir::new("oauth2");
    let addr = spawn_server(td.cfg.clone());

    let refresh_calls = Arc::new(AtomicUsize::new(0));
    let refresh_bg = refresh_calls.clone();
    let oauth_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let oauth_addr = oauth_listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for mut stream in oauth_listener.incoming().flatten() {
            let (_head, body_req) = read_full_http_request(&mut stream);
            assert!(body_req.contains("grant_type=refresh_token"), "{body_req}");
            assert!(body_req.contains("refresh_token=rt-initial-001"), "{body_req}");
            assert!(body_req.contains("client_id=ergon-app-id"), "{body_req}");
            assert!(body_req.contains("client_secret=oa2-client-sec-777"), "{body_req}");
            refresh_bg.fetch_add(1, Ordering::SeqCst);
            let body = r#"{"access_token":"oa2-fresh-token-abc123","expires_in":3600,"refresh_token":"rt-rotated-002"}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let api_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let api_addr = api_listener.local_addr().unwrap();
    let api_auth_headers = Arc::new(Mutex::new(Vec::new()));
    let api_auth_bg = api_auth_headers.clone();
    std::thread::spawn(move || {
        for mut stream in api_listener.incoming().flatten() {
            let (head, _body) = read_full_http_request(&mut stream);
            let auth_line = head
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("authorization:"))
                .unwrap_or("")
                .to_string();
            api_auth_bg.lock().unwrap().push(auth_line);
            let body = r#"{"delivered":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let (st_c, c_json, c_raw) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "OAuth2 Partner API",
            "kind": "rest_api",
            "url": format!("http://127.0.0.1:{}/events", api_addr.port()),
            "method": "POST",
            "headers_template": {
                "Authorization": "Bearer {{secret:oauth_access_token}}"
            },
            "oauth_config": {
                "token_url": format!("http://127.0.0.1:{}/token", oauth_addr.port()),
                "client_id": "ergon-app-id",
                "client_secret_key": "oa2_client_secret",
                "refresh_token_secret_key": "oa2_refresh_token",
                "scopes": ["crm.read", "crm.write"]
            },
            "require_approval": false,
            "secrets": {
                "oa2_client_secret": "oa2-client-sec-777",
                "oa2_refresh_token": "rt-initial-001"
            }
        })
        .to_string(),
    );
    assert_eq!(st_c, 201, "{c_raw}");
    let conn_id = c_json["connector"]["id"].as_str().unwrap().to_string();

    // First delivery triggers OAuth2 refresh.
    let res1 = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({ "connector_id": conn_id, "payload": { "step": 1 } }),
    )
    .unwrap();
    assert!(res1.contains("delivered via connector"), "{res1}");
    assert_eq!(refresh_calls.load(Ordering::SeqCst), 1);

    // Second immediate delivery reuses cached token (still 1 refresh call).
    let res2 = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({ "connector_id": conn_id, "payload": { "step": 2 } }),
    )
    .unwrap();
    assert!(res2.contains("delivered via connector"), "{res2}");
    assert_eq!(refresh_calls.load(Ordering::SeqCst), 1);

    let auths = api_auth_headers.lock().unwrap();
    assert_eq!(auths.len(), 2);
    assert!(auths[0].contains("Bearer oa2-fresh-token-abc123"));
    assert!(auths[1].contains("Bearer oa2-fresh-token-abc123"));

    // Verify secrets file updated on disk with 0600 permissions and rotated refresh token.
    let sec_text =
        std::fs::read_to_string(connectors::connector_secrets_path(&td.cfg.data_dir)).unwrap();
    assert!(sec_text.contains("oa2-fresh-token-abc123"));
    assert!(sec_text.contains("rt-rotated-002"));
}

#[test]
fn mcp_stdio_http_discover_call_strict_client_and_inbound_server() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1")),
        ("ERGON_FETCH_ALLOW_PRIVATE", Some("1")),
    ]);
    let td = TestDir::new("mcp-e2e");
    let addr = spawn_server(td.cfg.clone());

    // Store a connector secret to inject into stdio MCP server env.
    http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Secret Seed",
            "kind": "http_webhook",
            "url": "https://example.com/hook",
            "secret_key": "mcp_api_key",
            "secret_value": "mcp-secret-token-444"
        })
        .to_string(),
    );

    // 1. Register stdio MCP server backed by inline python3 JSON-RPC handler.
    let py_script = r#"
import sys, json, os
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    req = json.loads(line)
    mid = req.get("id")
    method = req.get("method")
    if method == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":mid,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"py-mcp","version":"1.0"}}}), flush=True)
    elif method == "tools/list":
        print(json.dumps({"jsonrpc":"2.0","id":mid,"result":{"tools":[{"name":"echo_note","description":"Echo a note","inputSchema":{"type":"object","properties":{"text":{"type":"string"}}}}]}}), flush=True)
    elif method == "tools/call":
        args = req.get("params", {}).get("arguments", {})
        txt = args.get("text", "")
        key = os.environ.get("MCP_API_KEY", "missing")
        print(json.dumps({"jsonrpc":"2.0","id":mid,"result":{"content":[{"type":"text","text":f"echo:{txt}:env={key}"}],"isError":False}}), flush=True)
"#;

    let (st_stdio, stdio_json, stdio_raw) = http_req(
        addr,
        "POST",
        "/api/mcp/servers",
        &json!({
            "id": "pytools",
            "name": "Python Stdio MCP",
            "transport": "stdio",
            "command_or_url": "python3",
            "args": ["-u", "-c", py_script],
            "env_secret_keys": { "MCP_API_KEY": "mcp_api_key" },
            "require_approval_by_default": false,
            "enabled": true
        })
        .to_string(),
    );
    assert_eq!(st_stdio, 201, "{stdio_raw}");
    assert_eq!(stdio_json["server"]["id"], json!("pytools"));

    // Discover stdio tools
    let (st_disc1, disc1_json, disc1_raw) =
        http_req(addr, "POST", "/api/mcp/servers/pytools/discover", "");
    assert_eq!(st_disc1, 200, "{disc1_raw}");
    assert_eq!(disc1_json["tools"][0]["name"], json!("echo_note"));

    // 2. Register HTTP/SSE MCP server
    let sse_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let sse_addr = sse_listener.local_addr().unwrap();
    let http_calls = Arc::new(AtomicUsize::new(0));
    let http_calls_bg = http_calls.clone();
    std::thread::spawn(move || {
        for mut stream in sse_listener.incoming().flatten() {
            let (_head, body) = read_full_http_request(&mut stream);
            let rpc: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let id = rpc["id"].clone();
            let method = rpc["method"].as_str().unwrap_or("");
            if method == "initialize" {
                let b = json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}}}).to_string();
                let r = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nMcp-Session-Id: sess-1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len());
                let _ = stream.write_all(r.as_bytes());
            } else if method == "notifications/initialized" {
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}");
            } else if method == "tools/list" {
                let b = json!({"jsonrpc":"2.0","id":id,"result":{"tools":[{"name":"crm_lookup","description":"Lookup client","inputSchema":{"type":"object","properties":{"client":{"type":"string"}}}}]}}).to_string();
                let r = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len());
                let _ = stream.write_all(r.as_bytes());
            } else if method == "tools/call" {
                http_calls_bg.fetch_add(1, Ordering::SeqCst);
                let client = rpc["params"]["arguments"]["client"].as_str().unwrap_or("");
                let ev = json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":format!("crm_match:{client}")}],"isError":false}}).to_string();
                let sse_body = format!("data: {ev}\n\n");
                let r = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sse_body}", sse_body.len());
                let _ = stream.write_all(r.as_bytes());
            } else {
                let b = json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"unknown"}}).to_string();
                let r = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len());
                let _ = stream.write_all(r.as_bytes());
            }
            let _ = stream.flush();
        }
    });

    let (st_http, _, http_raw) = http_req(
        addr,
        "POST",
        "/api/mcp/servers",
        &json!({
            "id": "crmhttp",
            "name": "CRM HTTP MCP",
            "transport": "http_sse",
            "command_or_url": format!("http://127.0.0.1:{}/mcp", sse_addr.port()),
            "args": [],
            "env_secret_keys": [],
            "require_approval_by_default": false,
            "enabled": true
        })
        .to_string(),
    );
    assert_eq!(st_http, 201, "{http_raw}");

    let (st_disc2, disc2_json, disc2_raw) =
        http_req(addr, "POST", "/api/mcp/servers/crmhttp/discover", "");
    assert_eq!(st_disc2, 200, "{disc2_raw}");
    assert_eq!(disc2_json["tools"][0]["name"], json!("crm_lookup"));

    // System prompt dynamically lists enabled MCP tools!
    let prompt = system_prompt(&td.cfg, &[]);
    assert!(prompt.contains("mcp.pytools.echo_note"), "{prompt}");
    assert!(prompt.contains("mcp.crmhttp.crm_lookup"), "{prompt}");

    // Call stdio MCP tool -> spotlighted & secret redacted!
    let stdio_res = execute_tool(
        &td.cfg,
        "mcp.pytools.echo_note",
        &json!({ "text": "hello_world" }),
    )
    .unwrap();
    assert!(stdio_res.contains("echo:hello_world"), "{stdio_res}");
    assert!(!stdio_res.contains("mcp-secret-token-444"), "{stdio_res}");

    // Call HTTP/SSE MCP tool directly in fast_internal
    let http_res = execute_tool(
        &td.cfg,
        "mcp.crmhttp.crm_lookup",
        &json!({ "client": "Acme" }),
    )
    .unwrap();
    assert!(http_res.contains("crm_match:Acme"), "{http_res}");
    assert_eq!(http_calls.load(Ordering::SeqCst), 1);

    // Switch to Strict Client -> mcp_tool_call parks for approval, executes on approve!
    http_req(
        addr,
        "PUT",
        "/api/workspace/policy",
        r#"{"profile":"strict_client"}"#,
    );
    let held_mcp = execute_tool(
        &td.cfg,
        "mcp.crmhttp.crm_lookup",
        &json!({ "client": "BetaCorp" }),
    )
    .unwrap();
    assert!(
        held_mcp.contains("parked for your person's approval"),
        "{held_mcp}"
    );
    assert_eq!(http_calls.load(Ordering::SeqCst), 1);

    let decs = records::load_decisions(&td.cfg.data_dir);
    let mcp_dec = decs
        .iter()
        .find(|d| d["kind"] == "mcp_tool_call" && d["status"] == "pending")
        .unwrap();
    let mcp_dec_id = mcp_dec["id"].as_str().unwrap();

    let (st_app, app_json, app_raw) = http_req(
        addr,
        "POST",
        &format!("/api/decisions/{mcp_dec_id}/approve"),
        "",
    );
    assert_eq!(st_app, 200, "{app_raw}");
    assert!(
        app_json["outcome"]
            .as_str()
            .unwrap_or("")
            .contains("crm_match:BetaCorp"),
        "{app_json}"
    );
    assert_eq!(http_calls.load(Ordering::SeqCst), 2);

    // Delete server
    let (st_del, _, _) = http_req(addr, "DELETE", "/api/mcp/servers/pytools", "");
    assert_eq!(st_del, 200);
    let (st_list, list_json, _) = http_req(addr, "GET", "/api/mcp/servers", "");
    assert_eq!(st_list, 200);
    assert_eq!(list_json["servers"].as_array().unwrap().len(), 1);

    // 3. Inbound MCP server: disabled (403) -> enabled -> read-only search_vault & query_table!
    std::fs::write(
        td.cfg.workspace_dir.join("ledger.csv"),
        "vendor,amount\nAcme,250.00\n",
    )
    .unwrap();

    let (st_in_dis, _, _) = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string(),
    );
    assert_eq!(st_in_dis, 403);

    let (st_en, _, _) = http_req(addr, "POST", "/api/mcp/inbound/config", r#"{"enabled":true}"#);
    assert_eq!(st_en, 200);

    let (st_in_list, in_list_json, _) = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string(),
    );
    assert_eq!(st_in_list, 200);
    let in_tools = in_list_json["result"]["tools"].as_array().unwrap();
    assert_eq!(in_tools.len(), 2);

    let (st_in_tbl, in_tbl_json, in_tbl_raw) = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "query_table",
                "arguments": { "path": "ledger.csv", "save_as": "should_not_write.csv" }
            }
        })
        .to_string(),
    );
    assert_eq!(st_in_tbl, 200, "{in_tbl_raw}");
    assert!(in_tbl_json["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .contains("Acme"));
    assert!(!td.cfg.workspace_dir.join("should_not_write.csv").exists());

    // Disallow write/unknown tools on inbound server
    let (st_in_bad, in_bad_json, _) = http_req(
        addr,
        "POST",
        "/api/mcp/inbound",
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "write_file", "arguments": { "path": "x.txt", "content": "y" } }
        })
        .to_string(),
    );
    assert_eq!(st_in_bad, 200);
    assert!(in_bad_json["error"]["code"].as_i64() == Some(-32601));
}

#[test]
fn tripwire_ssrf_blocks_private_and_metadata_for_smtp_imap_oauth_and_mcp() {
    let _env = EnvGuard::set(&[
        ("ERGON_DISABLE_SCHEDULER", Some("1")),
        ("ERGON_ALLOW_LOCAL_CONNECTORS", None),
        ("ERGON_FETCH_ALLOW_PRIVATE", None),
        ("ERGON_FETCH_ALLOW_HOSTS", None),
    ]);
    let td = TestDir::new("ssrf-block");
    let addr = spawn_server(td.cfg.clone());

    // Private IPs & metadata IPs blocked on SMTP config in production mode.
    for bad_host in ["127.0.0.1", "localhost", "10.0.0.1", "192.168.1.50", "169.254.169.254", "metadata.google.internal"] {
        let (st, _, raw) = http_req(
            addr,
            "POST",
            "/api/connectors/outbound",
            &json!({
                "name": "Blocked SMTP",
                "kind": "smtp_relay",
                "smtp_config": {
                    "host": bad_host,
                    "port": 465,
                    "security": "smtps",
                    "from_addr": "a@b.com",
                    "default_to": ["c@d.com"]
                }
            })
            .to_string(),
        );
        assert_eq!(st, 400, "expected 400 for SMTP host {bad_host}, got {raw}");

        let (st_imap, _, raw_imap) = http_req(
            addr,
            "POST",
            "/api/connectors/triggers",
            &json!({
                "name": "Blocked IMAP",
                "kind": "imap_poll",
                "imap_config": {
                    "host": bad_host,
                    "port": 993,
                    "security": "imaps",
                    "folder": "INBOX"
                }
            })
            .to_string(),
        );
        assert_eq!(
            st_imap, 400,
            "expected 400 for IMAP host {bad_host}, got {raw_imap}"
        );

        let (st_mcp, _, raw_mcp) = http_req(
            addr,
            "POST",
            "/api/mcp/servers",
            &json!({
                "name": "Blocked MCP",
                "transport": "http_sse",
                "command_or_url": format!("http://{bad_host}:8080/mcp")
            })
            .to_string(),
        );
        assert_eq!(
            st_mcp, 400,
            "expected 400 for HTTP MCP host {bad_host}, got {raw_mcp}"
        );
    }

    // plain_loopback_test rejected on non-loopback host even if local connectors enabled.
    {
        let _local = EnvGuard::set(&[("ERGON_ALLOW_LOCAL_CONNECTORS", Some("1"))]);
        let (st_non_lb, _, raw_non_lb) = http_req(
            addr,
            "POST",
            "/api/connectors/outbound",
            &json!({
                "name": "Bad Plain SMTP",
                "kind": "smtp_relay",
                "smtp_config": {
                    "host": "smtp.example.com",
                    "port": 2525,
                    "security": "plain_loopback_test",
                    "from_addr": "a@b.com",
                    "default_to": ["c@d.com"]
                }
            })
            .to_string(),
        );
        assert_eq!(st_non_lb, 400, "{raw_non_lb}");
    }

    // Tripwire protects mcp_servers.json and catches secrets passed to mcp.* tools.
    assert!(tripwire::sensitive_target("/tmp/any/mcp_servers.json").is_some());
    let _arm = tripwire::arm(tripwire::ArmSpec {
        root: td.cfg.data_dir.clone(),
        person_dir: td.cfg.data_dir.clone(),
        workspace: td.cfg.workspace_dir.clone(),
        memory_path: td.cfg.memory_path.clone(),
        person_id: "default".to_string(),
        run_id: "run-mcp-trip".to_string(),
        thread_id: "t-mcp".to_string(),
        provenance: "summarize notes".to_string(),
        secrets: vec!["top-secret-api-key-123".to_string()],
    });
    let pre = tripwire::pre_tool(
        "mcp.crmhttp.crm_lookup",
        &json!({ "client": "top-secret-api-key-123" }),
        false,
    );
    assert!(matches!(pre, tripwire::PreTool::Refuse(_)));
}
