//! Integration tests for Coolify reverse-proxy HTTPS (`ERGON_PUBLIC_URL`,
//! `ERGON_TRUST_PROXY`), per-employee encrypted model keys (`/api/config/personal`),
//! and the `ergon healthcheck` CLI command.

use ergon_rs::{serve, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static SEQ: AtomicU64 = AtomicU64::new(0);

struct EnvVarGuard {
    key: &'static str,
    prev: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let prev = std::env::var_os(key);
        unsafe { std::env::set_var(key, val) };
        Self { key, prev }
    }

    fn unset(key: &'static str) -> Self {
        let prev = std::env::var_os(key);
        unsafe { std::env::remove_var(key) };
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(label: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-coolify-{}-{}-{label}",
            std::process::id(),
            id
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
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

fn spawn_server(cfg: Config) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(50));
    addr
}

fn http_req_with_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> (u16, String, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let body_bytes = body.as_bytes();
    let mut head = format!("{method} {path} HTTP/1.1\r\n");
    let has_host = headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host"));
    if !has_host {
        head.push_str(&format!("Host: 127.0.0.1:{}\r\n", addr.port()));
    }
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
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
        .expect("HTTP header separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]).to_string();
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_text = String::from_utf8_lossy(&raw[sep + 4..]).to_string();
    let val = serde_json::from_str::<Value>(&body_text).unwrap_or(Value::Null);
    (status, head_str, val, body_text)
}

fn extract_set_cookie_token(head: &str) -> String {
    for line in head.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("set-cookie") {
                if let Some(rest) = v.trim().strip_prefix("ergon_session=") {
                    return rest.split(';').next().unwrap_or("").to_string();
                }
            }
        }
    }
    String::new()
}

#[test]
fn proxy_https_public_url_cookies_and_oauth_origin() {
    let _g_pub = EnvVarGuard::set("ERGON_PUBLIC_URL", "https://ergon.coolify.test/");
    let _g_tp = EnvVarGuard::set("ERGON_TRUST_PROXY", "1");

    let td = TestDir::new("puburl");
    let addr = spawn_server(td.cfg.clone());

    // Public host is auto-allowed by ERGON_PUBLIC_URL without manual ERGON_ALLOWED_HOSTS
    let (st, _, health, raw) = http_req_with_headers(
        addr,
        "GET",
        "/api/health",
        "",
        &[("Host", "ergon.coolify.test"), ("X-Forwarded-Proto", "https")],
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(health["tls"], json!(true));
    assert_eq!(health["public_url"], json!("https://ergon.coolify.test"));

    // OAuth redirect URI uses https://ergon.coolify.test instead of plain http://
    let (st, _, suites, raw) = http_req_with_headers(
        addr,
        "GET",
        "/api/plugins/oauth/apps",
        "",
        &[("Host", "ergon.coolify.test")],
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(
        suites["suites"][0]["redirect_uri"],
        json!("https://ergon.coolify.test/api/plugins/oauth/callback")
    );

    // Initial admin setup emits a Secure; session cookie behind the reverse proxy
    let (st, head, _, raw) = http_req_with_headers(
        addr,
        "POST",
        "/api/auth/setup",
        &json!({
            "username": "admin",
            "password": "StrongPassword!123",
            "display_name": "Admin"
        })
        .to_string(),
        &[("Host", "ergon.coolify.test"), ("X-Forwarded-Proto", "https")],
    );
    assert_eq!(st, 201, "{raw}");
    assert!(
        head.lines()
            .any(|l| l.to_ascii_lowercase().contains("set-cookie:") && l.contains("Secure")),
        "expected Secure attribute on session cookie:\n{head}"
    );
}

#[test]
fn ignores_forwarded_proto_when_trust_proxy_disabled() {
    let _g_pub = EnvVarGuard::unset("ERGON_PUBLIC_URL");
    let _g_srv = EnvVarGuard::unset("SERVICE_URL_ERGON");
    let _g_tp = EnvVarGuard::set("ERGON_TRUST_PROXY", "0");

    let td = TestDir::new("noproxy");
    let addr = spawn_server(td.cfg.clone());

    let (st, _, health, raw) = http_req_with_headers(
        addr,
        "GET",
        "/api/health",
        "",
        &[("X-Forwarded-Proto", "https")],
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(
        health["tls"],
        json!(false),
        "must not trust spoofed X-Forwarded-Proto when ERGON_TRUST_PROXY is disabled"
    );
}

#[test]
fn per_employee_encrypted_model_key_isolation_and_rbac() {
    let _g_pub = EnvVarGuard::unset("ERGON_PUBLIC_URL");
    let _g_srv = EnvVarGuard::unset("SERVICE_URL_ERGON");
    let _g_tp = EnvVarGuard::unset("ERGON_TRUST_PROXY");

    let td = TestDir::new("personal");
    let addr = spawn_server(td.cfg.clone());

    // Setup admin
    let (st, head_admin, _, raw) = http_req_with_headers(
        addr,
        "POST",
        "/api/auth/setup",
        &json!({
            "username": "boss",
            "password": "AdminPassword!123",
            "display_name": "Boss"
        })
        .to_string(),
        &[],
    );
    assert_eq!(st, 201, "{raw}");
    let admin_tok = extract_set_cookie_token(&head_admin);
    let admin_auth = format!("Bearer {admin_tok}");

    // Create two regular employees: alice and bob
    for (uname, dname) in [("alice", "Alice"), ("bob", "Bob")] {
        let (st, _, _, raw) = http_req_with_headers(
            addr,
            "POST",
            "/api/auth/users",
            &json!({
                "username": uname,
                "password": "UserPassword!123",
                "role": "member",
                "display_name": dname
            })
            .to_string(),
            &[("Authorization", &admin_auth)],
        );
        assert_eq!(st, 201, "{raw}");
    }

    // Login as Alice
    let (st, head_alice, _, raw) = http_req_with_headers(
        addr,
        "POST",
        "/api/auth/login",
        &json!({ "username": "alice", "password": "UserPassword!123" }).to_string(),
        &[],
    );
    assert_eq!(st, 200, "{raw}");
    let alice_tok = extract_set_cookie_token(&head_alice);
    let alice_auth = format!("Bearer {alice_tok}");

    // Login as Bob
    let (st, head_bob, _, raw) = http_req_with_headers(
        addr,
        "POST",
        "/api/auth/login",
        &json!({ "username": "bob", "password": "UserPassword!123" }).to_string(),
        &[],
    );
    assert_eq!(st, 200, "{raw}");
    let bob_tok = extract_set_cookie_token(&head_bob);
    let bob_auth = format!("Bearer {bob_tok}");

    // Alice cannot touch company-wide /api/config (403)
    let (st, _, _, _) = http_req_with_headers(
        addr,
        "POST",
        "/api/config",
        r#"{"model_base_url":"https://opencode.ai/zen/go/v1","model_id":"deepseek-v4.1-flash"}"#,
        &[("Authorization", &alice_auth)],
    );
    assert_eq!(st, 403);

    // Alice sets her own $20 OpenCode Go seat key on /api/config/personal
    let secret_seat_key = "sk-opencode-alice-personal-seat-999";
    let (st, _, pcfg, raw) = http_req_with_headers(
        addr,
        "POST",
        "/api/config/personal",
        &json!({
            "model_base_url": "https://opencode.ai/zen/go/v1",
            "model_id": "deepseek-v4.1-flash",
            "api_key": secret_seat_key
        })
        .to_string(),
        &[("Authorization", &alice_auth)],
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(pcfg["has_personal_key"], json!(true));
    assert_eq!(pcfg["using_personal"], json!(true));
    assert_eq!(pcfg["effective_configured"], json!(true));
    assert_eq!(pcfg["effective_model_id"], json!("deepseek-v4.1-flash"));

    // Alice's /api/status now reports model_configured=true, while Bob remains false
    let (st, _, alice_status, _) =
        http_req_with_headers(addr, "GET", "/api/status", "", &[("Authorization", &alice_auth)]);
    assert_eq!(st, 200);
    assert_eq!(alice_status["model_configured"], json!(true));

    let (st, _, bob_status, _) =
        http_req_with_headers(addr, "GET", "/api/status", "", &[("Authorization", &bob_auth)]);
    assert_eq!(st, 200);
    assert_eq!(bob_status["model_configured"], json!(false));

    // Zero plaintext API keys anywhere in Alice's data dir
    let mut found_override = false;
    for entry in std::fs::read_dir(td.cfg.data_dir.join("people")).unwrap().flatten() {
        let ov_path = entry.path().join("model_override.json");
        if ov_path.exists() {
            found_override = true;
            let contents = std::fs::read_to_string(&ov_path).unwrap();
            assert!(!contents.contains(secret_seat_key), "plaintext key leaked to model_override.json");
            assert!(contents.contains("vault:person_model:api_key"));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(&ov_path).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o600);
            }
        }
    }
    assert!(found_override, "expected model_override.json on disk for Alice");

    // DELETE /api/config/personal resets Alice back to company default
    let (st, _, cleared, raw) = http_req_with_headers(
        addr,
        "DELETE",
        "/api/config/personal",
        "",
        &[("Authorization", &alice_auth)],
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(cleared["has_personal_key"], json!(false));
    assert_eq!(cleared["using_personal"], json!(false));
    assert_eq!(cleared["effective_configured"], json!(false));
}

#[test]
fn healthcheck_cli_exits_zero_when_healthy_and_nonzero_when_down() {
    let td = TestDir::new("hc");
    let addr = spawn_server(td.cfg.clone());
    let bin = env!("CARGO_BIN_EXE_ergon");

    let ok_out = Command::new(bin)
        .args(["healthcheck", "--port", &addr.port().to_string(), "--json"])
        .output()
        .unwrap();
    assert!(
        ok_out.status.success(),
        "expected exit 0 from healthcheck, stderr: {}",
        String::from_utf8_lossy(&ok_out.stderr)
    );

    // Unused closed port must fail cleanly
    let closed_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let closed_port = closed_listener.local_addr().unwrap().port();
    drop(closed_listener);
    let fail_out = Command::new(bin)
        .args(["healthcheck", "--port", &closed_port.to_string()])
        .output()
        .unwrap();
    assert!(!fail_out.status.success(), "expected non-zero exit when port is down");
}
