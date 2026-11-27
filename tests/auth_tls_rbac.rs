//! Integration tests for Gap 1 (P0): Multi-user auth, PBKDF2-HMAC-SHA256,
//! persisted opaque sessions, RBAC, strict cross-user isolation, break-glass
//! admin gate token, real HTTPS/TLS (`rustls`), `/phone` gating, CLI user/TLS
//! commands, and full OIDC Discovery + PKCE (`S256`) + Userinfo flow.

use ergon_rs::Config;
use rustls_pki_types::pem::PemObject;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

fn test_root(tag: &str) -> PathBuf {
    let n = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "ergon-auth-tls-{tag}-{}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn test_config(root: &Path) -> Config {
    let data = root.join("data");
    let ws = data.join("workspace");
    let mem = data.join("memory.md");
    Config {
        data_dir: data,
        workspace_dir: ws,
        memory_path: mem,
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "stub".to_string(),
        api_key: None,
    }
}

fn start_plain(cfg: Config, gate: Option<&str>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let g = gate.map(String::from);
    std::thread::spawn(move || {
        let _ = ergon_rs::serve_with_gate(listener, cfg, g);
    });
    wait_ready_plain(port);
    port
}

fn wait_ready_plain(port: u16) {
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

struct HttpResp {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpResp {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn set_cookies(&self) -> Vec<&str> {
        self.headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
            .map(|(_, v)| v.as_str())
            .collect()
    }

    fn session_cookie_pair(&self) -> Option<String> {
        for c in self.set_cookies() {
            if let Some(first) = c.split(';').next() {
                if first.trim().starts_with("ergon_session=") {
                    return Some(first.trim().to_string());
                }
            }
        }
        None
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|e| panic!("invalid JSON ({}): {e}\n{}", self.status, self.body))
    }
}

fn parse_http_response(raw: &str) -> HttpResp {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    HttpResp {
        status,
        headers,
        body: body.to_string(),
    }
}

fn http_req(port: u16, method: &str, path: &str, body: Option<&str>, extra_headers: &[(&str, &str)]) -> HttpResp {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let body_bytes = body.unwrap_or("").as_bytes();
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", body_bytes.len()));
    }
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).unwrap();
    }
    stream.flush().unwrap();
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    parse_http_response(&raw)
}

fn tls_client_config(cert_pem_path: &Path) -> Arc<rustls::ClientConfig> {
    let cert_bytes = std::fs::read(cert_pem_path).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    for item in rustls_pki_types::CertificateDer::pem_slice_iter(&cert_bytes) {
        roots.add(item.unwrap()).unwrap();
    }
    Arc::new(
        rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth(),
    )
}

fn https_req(
    port: u16,
    client_cfg: &Arc<rustls::ClientConfig>,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> HttpResp {
    let tcp = TcpStream::connect(("127.0.0.1", port)).unwrap();
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let server_name = rustls_pki_types::ServerName::try_from("localhost").unwrap();
    let conn = rustls::ClientConnection::new(client_cfg.clone(), server_name).unwrap();
    let mut stream = rustls::StreamOwned::new(conn, tcp);
    let body_bytes = body.unwrap_or("").as_bytes();
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n");
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", body_bytes.len()));
    }
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).unwrap();
    }
    stream.flush().unwrap();
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    parse_http_response(&raw)
}

#[test]
fn setup_login_sessions_persistence_logout_and_0600_storage() {
    let root = test_root("setup_login");
    let cfg = test_config(&root);
    let port = start_plain(cfg.clone(), None);

    // 1. Before any accounts exist (loopback dev mode): auth_required is false, /api/status works.
    let me0 = http_req(port, "GET", "/api/auth/me", None, &[]);
    assert_eq!(me0.status, 200);
    let me0_j = me0.json();
    assert_eq!(me0_j["auth_required"], json!(false));
    assert_eq!(me0_j["needs_setup"], json!(true));
    assert_eq!(http_req(port, "GET", "/api/status", None, &[]).status, 200);

    // 2. First admin setup creates account + session cookie + bearer token.
    let admin_pw = "Admin-Passphrase-2026!";
    let setup = http_req(
        port,
        "POST",
        "/api/auth/setup",
        Some(
            &json!({
                "username": "admin",
                "password": admin_pw,
                "display_name": "Alex Admin",
                "email": "admin@example.com"
            })
            .to_string(),
        ),
        &[],
    );
    assert_eq!(setup.status, 201, "setup failed: {}", setup.body);
    let setup_j = setup.json();
    let admin_token = setup_j["token"].as_str().unwrap().to_string();
    assert!(admin_token.starts_with("ergs_"));
    let admin_cookie = setup.session_cookie_pair().expect("Set-Cookie missing");
    let raw_set_cookie = setup.set_cookies().join("; ");
    assert!(raw_set_cookie.contains("HttpOnly"), "{raw_set_cookie}");
    assert!(raw_set_cookie.contains("SameSite=Lax"), "{raw_set_cookie}");
    assert!(raw_set_cookie.contains("Path=/"), "{raw_set_cookie}");

    // 3. Verify 0600 permissions and ZERO plaintext passwords on disk.
    let users_file = cfg.data_dir.join("auth_users.json");
    let sessions_file = cfg.data_dir.join("auth_sessions.json");
    assert!(users_file.is_file());
    assert!(sessions_file.is_file());
    let users_raw = std::fs::read_to_string(&users_file).unwrap();
    let sessions_raw = std::fs::read_to_string(&sessions_file).unwrap();
    assert!(!users_raw.contains(admin_pw), "plaintext password leaked!");
    assert!(users_raw.contains("pbkdf2_sha256$120000$"));
    assert!(
        !sessions_raw.contains(&admin_token),
        "raw session token must be hashed on disk!"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&users_file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&sessions_file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // 4. Second setup attempt is strictly forbidden (403).
    let setup2 = http_req(
        port,
        "POST",
        "/api/auth/setup",
        Some(&json!({"username": "hijack", "password": "HijackPassword99"}).to_string()),
        &[],
    );
    assert_eq!(setup2.status, 403, "second setup must be refused: {}", setup2.body);

    // 5. The instant an account exists, ALL /api/* (except /api/auth/* and /api/health) require auth (401).
    for (m, p, b) in [
        ("GET", "/api/status", None),
        ("GET", "/api/version", None),
        ("GET", "/api/people", None),
        ("GET", "/api/files", None),
        ("GET", "/api/clients", None),
        ("GET", "/api/invoices", None),
        ("GET", "/api/auth/users", None),
        ("POST", "/api/people/switch", Some(r#"{"id":"default"}"#)),
    ] {
        let r = http_req(port, m, p, b, &[]);
        assert_eq!(r.status, 401, "expected 401 on unauth {m} {p}, got {}: {}", r.status, r.body);
    }
    assert_eq!(http_req(port, "GET", "/api/health", None, &[]).status, 200);
    let me_unauth = http_req(port, "GET", "/api/auth/me", None, &[]);
    assert_eq!(me_unauth.status, 200);
    assert_eq!(me_unauth.json()["authenticated"], json!(false));
    assert_eq!(me_unauth.json()["auth_required"], json!(true));
    assert_eq!(me_unauth.json()["needs_setup"], json!(false));

    // 6. Both Cookie and Authorization: Bearer authenticate cleanly.
    let with_cookie = http_req(port, "GET", "/api/status", None, &[("Cookie", &admin_cookie)]);
    assert_eq!(with_cookie.status, 200, "{}", with_cookie.body);
    let bearer_hdr = format!("Bearer {admin_token}");
    let with_bearer = http_req(port, "GET", "/api/auth/me", None, &[("Authorization", &bearer_hdr)]);
    assert_eq!(with_bearer.status, 200);
    assert_eq!(with_bearer.json()["authenticated"], json!(true));
    assert_eq!(with_bearer.json()["user"]["username"], json!("admin"));
    assert_eq!(with_bearer.json()["user"]["role"], json!("admin"));

    // 7. Wrong password returns 401; correct login mints a new session.
    let bad_login = http_req(
        port,
        "POST",
        "/api/auth/login",
        Some(&json!({"username": "admin", "password": "wrong-password"}).to_string()),
        &[],
    );
    assert_eq!(bad_login.status, 401);

    let good_login = http_req(
        port,
        "POST",
        "/api/auth/login",
        Some(&json!({"username": "admin@example.com", "password": admin_pw}).to_string()),
        &[],
    );
    assert_eq!(good_login.status, 200, "{}", good_login.body);
    let login_cookie = good_login.session_cookie_pair().unwrap();

    // 8. Persisted session survives server restart against the same DATA_DIR.
    let port2 = start_plain(cfg, None);
    let after_restart = http_req(port2, "GET", "/api/status", None, &[("Cookie", &login_cookie)]);
    assert_eq!(after_restart.status, 200, "session must persist across restart");

    // 9. Logout revokes session immediately.
    let logout = http_req(port2, "POST", "/api/auth/logout", Some("{}"), &[("Cookie", &login_cookie)]);
    assert_eq!(logout.status, 200);
    let after_logout = http_req(port2, "GET", "/api/status", None, &[("Cookie", &login_cookie)]);
    assert_eq!(after_logout.status, 401, "revoked session must be rejected");
}

#[test]
fn rbac_and_cross_user_isolation_across_workspaces_crm_invoices_and_admin_routes() {
    let root = test_root("rbac_iso");
    let cfg = test_config(&root);
    let port = start_plain(cfg, None);

    // Setup admin
    let admin_res = http_req(
        port,
        "POST",
        "/api/auth/setup",
        Some(&json!({"username": "admin", "password": "AdminPassword123", "display_name": "Boss Admin"}).to_string()),
        &[],
    );
    assert_eq!(admin_res.status, 201);
    let admin_cookie = admin_res.session_cookie_pair().unwrap();
    let admin_person_id = admin_res.json()["user"]["person_id"].as_str().unwrap().to_string();

    // Admin creates 4 role-distinct users: pm (alice), ae (bob), installer (carl), member (dana)
    let create_user = |u: &str, pw: &str, role: &str, name: &str| -> (String, String) {
        let cr = http_req(
            port,
            "POST",
            "/api/auth/users",
            Some(&json!({"username": u, "password": pw, "role": role, "display_name": name}).to_string()),
            &[("Cookie", &admin_cookie)],
        );
        assert_eq!(cr.status, 201, "create {u}: {}", cr.body);
        let pid = cr.json()["user"]["person_id"].as_str().unwrap().to_string();
        let lg = http_req(
            port,
            "POST",
            "/api/auth/login",
            Some(&json!({"username": u, "password": pw}).to_string()),
            &[],
        );
        assert_eq!(lg.status, 200, "login {u}: {}", lg.body);
        (pid, lg.session_cookie_pair().unwrap())
    };

    let (alice_pid, alice_cookie) = create_user("alice", "AlicePass123", "pm", "Alice PM");
    let (bob_pid, bob_cookie) = create_user("bob", "BobPass12345", "ae", "Bob AE");
    let (_carl_pid, carl_cookie) = create_user("carl", "CarlPass1234", "installer", "Carl Field");
    let (_dana_pid, dana_cookie) = create_user("dana", "DanaPass1234", "member", "Dana Member");

    // 1. Non-admin hitting EVERY admin route -> 403 Forbidden
    for (method, path, body) in [
        ("GET", "/api/auth/users", None),
        ("POST", "/api/auth/users", Some(r#"{"username":"x","password":"Password123","role":"member"}"#)),
        ("GET", "/api/auth/oidc/config", None),
        ("PUT", "/api/auth/oidc/config", Some(r#"{"enabled":false}"#)),
        ("GET", "/api/config", None),
        ("POST", "/api/config", Some(r#"{"model_base_url":"http://127.0.0.1:1/v1","model_id":"x"}"#)),
        ("POST", "/api/update/check", Some("{}")),
        ("POST", "/api/update/apply", Some("{}")),
        ("GET", "/api/security", None),
        ("POST", "/api/security/policy", Some(r#"{"action":"block","host":"evil.test"}"#)),
        ("GET", "/api/connectors", None),
        ("GET", "/api/mcp/servers", None),
        ("POST", "/api/people", Some(r#"{"name":"Ghost","role":"pm"}"#)),
    ] {
        for (who, ck) in [
            ("alice-pm", &alice_cookie),
            ("bob-ae", &bob_cookie),
            ("carl-installer", &carl_cookie),
            ("dana-member", &dana_cookie),
        ] {
            let r = http_req(port, method, path, body, &[("Cookie", ck)]);
            assert_eq!(
                r.status, 403,
                "{who} must get 403 on admin route {method} {path}, got {}: {}",
                r.status, r.body
            );
        }
    }

    // 2. Non-admin POST /api/people/switch & /api/active-person to another person -> 403 Forbidden
    for path in ["/api/people/switch", "/api/active-person"] {
        let sw1 = http_req(
            port,
            "POST",
            path,
            Some(&json!({"id": bob_pid}).to_string()),
            &[("Cookie", &alice_cookie)],
        );
        assert_eq!(sw1.status, 403, "alice switching to bob via {path}: {}", sw1.body);

        let sw2 = http_req(
            port,
            "POST",
            path,
            Some(&json!({"person_id": admin_person_id}).to_string()),
            &[("Cookie", &carl_cookie)],
        );
        assert_eq!(sw2.status, 403, "carl switching to admin via {path}: {}", sw2.body);
    }

    // GET /api/people for non-admin leaks ONLY self
    let alice_people = http_req(port, "GET", "/api/people", None, &[("Cookie", &alice_cookie)]).json();
    let arr = alice_people["people"].as_array().unwrap();
    assert_eq!(arr.len(), 1, "non-admin /api/people must return only self: {alice_people}");
    assert_eq!(arr[0]["id"], json!(alice_pid));

    // 3. Cross-user workspace file, thread & work graph isolation
    let put_f = http_req(
        port,
        "PUT",
        "/api/files/content",
        Some(&json!({"path": "alice-private.md", "content": "TOP SECRET ALICE DOC"}).to_string()),
        &[("Cookie", &alice_cookie)],
    );
    assert_eq!(put_f.status, 200, "alice write file: {}", put_f.body);
    let new_th = http_req(port, "POST", "/api/threads", Some("{}"), &[("Cookie", &alice_cookie)]);
    assert_eq!(new_th.status, 201);
    // Record a real work receipt in Alice's isolated data_dir so /api/graph has Alice's work node + file node.
    {
        let alice_data_dir = root.join("data").join("people").join(&alice_pid);
        let alice_cfg = Config {
            data_dir: alice_data_dir.clone(),
            workspace_dir: alice_data_dir.join("workspace"),
            memory_path: alice_data_dir.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "stub".to_string(),
            api_key: None,
        };
        ergon_rs::records::append_work(
            &alice_cfg,
            "Alice Confidential Q4 Bid Audit",
            "Reconciled private bid sheet",
            &["alice-private.md".to_string()],
            "t-alice-1",
            true,
        );
    }

    // Bob and Carl calling GET /api/graph MUST NOT see Alice's person node, task title, or filename!
    for (who, ck) in [("bob-ae", &bob_cookie), ("carl-installer", &carl_cookie)] {
        let g = http_req(port, "GET", "/api/graph", None, &[("Cookie", ck)]);
        assert_eq!(g.status, 200, "{who} /api/graph: {}", g.body);
        assert!(
            !g.body.contains(&format!("person:{alice_pid}")),
            "{who} /api/graph leaked Alice person node: {}",
            g.body
        );
        assert!(
            !g.body.contains("Alice Confidential Q4 Bid Audit"),
            "{who} /api/graph leaked Alice task title: {}",
            g.body
        );
        assert!(
            !g.body.contains("alice-private.md"),
            "{who} /api/graph leaked Alice filename: {}",
            g.body
        );
    }
    // Admin DOES see the full org work graph including Alice.
    let admin_graph = http_req(port, "GET", "/api/graph", None, &[("Cookie", &admin_cookie)]);
    assert_eq!(admin_graph.status, 200);
    assert!(admin_graph.body.contains(&format!("person:{alice_pid}")));
    assert!(admin_graph.body.contains("alice-private.md"));

    // Bob cannot see Alice's file or thread
    let bob_files = http_req(port, "GET", "/api/files", None, &[("Cookie", &bob_cookie)]).json();
    assert!(
        !bob_files.to_string().contains("alice-private.md"),
        "bob must not see alice's files: {bob_files}"
    );
    let bob_read = http_req(
        port,
        "GET",
        "/api/files/content?path=alice-private.md",
        None,
        &[("Cookie", &bob_cookie)],
    );
    assert_eq!(bob_read.status, 404, "bob must not read alice's file");
    let bob_threads = http_req(port, "GET", "/api/threads", None, &[("Cookie", &bob_cookie)]).json();
    assert_eq!(bob_threads["threads"].as_array().unwrap().len(), 0);

    // 4. Cross-user CRM / Client / Engagement / Time Entry / Invoice isolation + Role RBAC
    // Alice (pm) creates Client A + Engagement A
    let c_a = http_req(
        port,
        "POST",
        "/api/clients",
        Some(&json!({"name": "Northwind Field Co", "default_rate_usd": 175.0}).to_string()),
        &[("Cookie", &alice_cookie)],
    );
    assert_eq!(c_a.status, 201, "alice create client: {}", c_a.body);
    let client_a_id = c_a.json()["client"]["id"].as_str().unwrap().to_string();

    let e_a = http_req(
        port,
        "POST",
        "/api/engagements",
        Some(&json!({"client_id": client_a_id, "name": "Q4 Site Rollout"}).to_string()),
        &[("Cookie", &alice_cookie)],
    );
    assert_eq!(e_a.status, 201, "alice create engagement: {}", e_a.body);
    let eng_a_id = e_a.json()["engagement"]["id"].as_str().unwrap().to_string();

    // Alice (pm) can view invoices (200) but CANNOT generate/modify invoices (403 — AE/Office/Admin only)
    assert_eq!(http_req(port, "GET", "/api/invoices", None, &[("Cookie", &alice_cookie)]).status, 200);
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/invoices/generate",
            Some(&json!({"engagement_id": eng_a_id, "period_label": "Nov 2026"}).to_string()),
            &[("Cookie", &alice_cookie)],
        )
        .status,
        403
    );

    // Bob (ae) CANNOT see or mutate Alice's client, engagement, time entries, metering, or invoices
    let bob_cli = http_req(port, "GET", "/api/clients", None, &[("Cookie", &bob_cookie)]).json();
    assert_eq!(bob_cli["clients"].as_array().unwrap().len(), 0, "bob sees alice's client: {bob_cli}");
    assert_eq!(bob_cli["engagements"].as_array().unwrap().len(), 0);

    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/clients?person_id={alice_pid}"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "PATCH",
            &format!("/api/clients/{client_a_id}"),
            Some(&json!({"name": "Hijacked"}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/engagements/{eng_a_id}/context"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "PATCH",
            &format!("/api/engagements/{eng_a_id}"),
            Some(&json!({"name": "Hijacked"}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/engagements/switch",
            Some(&json!({"engagement_id": eng_a_id}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/time-entries?engagement_id={eng_a_id}"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/time-entries?client_id={client_a_id}"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/time-entries",
            Some(&json!({"engagement_id": eng_a_id, "description": "Inject", "hours": 1.0}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/invoices/generate",
            Some(&json!({"engagement_id": eng_a_id, "period_label": "Nov 2026"}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/invoices?engagement_id={eng_a_id}"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );
    assert_eq!(
        http_req(
            port,
            "GET",
            &format!("/api/metering/summary?engagement_id={eng_a_id}"),
            None,
            &[("Cookie", &bob_cookie)],
        )
        .status,
        403
    );

    // Bob (ae) CAN create his own client, engagement, time entry, and invoice!
    let c_b = http_req(
        port,
        "POST",
        "/api/clients",
        Some(&json!({"name": "Summit Electric", "default_rate_usd": 200.0}).to_string()),
        &[("Cookie", &bob_cookie)],
    );
    assert_eq!(c_b.status, 201);
    let client_b_id = c_b.json()["client"]["id"].as_str().unwrap().to_string();
    let e_b = http_req(
        port,
        "POST",
        "/api/engagements",
        Some(&json!({"client_id": client_b_id, "name": "Annual Service"}).to_string()),
        &[("Cookie", &bob_cookie)],
    );
    assert_eq!(e_b.status, 201);
    let eng_b_id = e_b.json()["engagement"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/time-entries",
            Some(&json!({"engagement_id": eng_b_id, "description": "Audit", "hours": 2.0, "rate_usd": 200.0}).to_string()),
            &[("Cookie", &bob_cookie)],
        )
        .status,
        201
    );
    let inv_b = http_req(
        port,
        "POST",
        "/api/invoices/generate",
        Some(&json!({"engagement_id": eng_b_id, "period_label": "Nov 2026"}).to_string()),
        &[("Cookie", &bob_cookie)],
    );
    assert_eq!(inv_b.status, 201, "bob AE generates own invoice: {}", inv_b.body);

    // Carl (installer) cannot create clients, view/generate invoices, or change workspace policy
    assert_eq!(
        http_req(
            port,
            "POST",
            "/api/clients",
            Some(&json!({"name": "Blocked Client"}).to_string()),
            &[("Cookie", &carl_cookie)],
        )
        .status,
        403
    );
    assert_eq!(http_req(port, "GET", "/api/invoices", None, &[("Cookie", &carl_cookie)]).status, 403);
    assert_eq!(
        http_req(
            port,
            "PUT",
            "/api/workspace/policy",
            Some(&json!({"profile": "strict_client"}).to_string()),
            &[("Cookie", &carl_cookie)],
        )
        .status,
        403
    );
    // Carl CAN use his own desk endpoints & /phone
    assert_eq!(http_req(port, "GET", "/api/today", None, &[("Cookie", &carl_cookie)]).status, 200);
    assert_eq!(http_req(port, "GET", "/phone", None, &[("Cookie", &carl_cookie)]).status, 200);
}

#[test]
fn break_glass_gate_token_is_admin_audited_and_never_elevates_non_admin_session() {
    let root = test_root("break_glass");
    let cfg = test_config(&root);
    let port = start_plain(cfg, Some("BreakGlass-Secret-99"));

    // Create initial admin & non-admin installer (using break-glass or setup)
    let setup = http_req(
        port,
        "POST",
        "/api/auth/setup",
        Some(&json!({"username": "admin", "password": "AdminPassword123"}).to_string()),
        &[("X-Ergon-Gate", "BreakGlass-Secret-99")],
    );
    assert_eq!(setup.status, 201, "{}", setup.body);
    let admin_cookie = setup.session_cookie_pair().unwrap();
    let admin_pid = setup.json()["user"]["person_id"].as_str().unwrap().to_string();

    let mk = http_req(
        port,
        "POST",
        "/api/auth/users",
        Some(&json!({"username": "tech", "password": "TechPassword123", "role": "installer"}).to_string()),
        &[("Cookie", &admin_cookie)],
    );
    assert_eq!(mk.status, 201);
    let tech_login = http_req(
        port,
        "POST",
        "/api/auth/login",
        Some(&json!({"username": "tech", "password": "TechPassword123"}).to_string()),
        &[],
    );
    assert_eq!(tech_login.status, 200);
    let tech_cookie = tech_login.session_cookie_pair().unwrap();

    // 1. Wrong gate token -> 401
    let bad_gate = http_req(
        port,
        "GET",
        "/api/auth/users",
        None,
        &[("X-Ergon-Gate", "wrong-token")],
    );
    assert_eq!(bad_gate.status, 401);

    // 2. Valid break-glass gate token (no user session) -> admin access + audited!
    let bg_users = http_req(
        port,
        "GET",
        "/api/auth/users",
        None,
        &[("X-Ergon-Gate", "BreakGlass-Secret-99")],
    );
    assert_eq!(bg_users.status, 200, "{}", bg_users.body);
    let sec = http_req(
        port,
        "GET",
        "/api/security",
        None,
        &[("Cookie", &admin_cookie)],
    );
    assert_eq!(sec.status, 200);
    assert!(
        sec.body.contains("AU2-BREAKGLASS"),
        "break-glass usage must be audited in security events: {}",
        sec.body
    );

    // 3. Non-admin user combining their session cookie WITH the break-glass header
    // MUST NOT elevate privileges or bypass cross-user isolation!
    let escalate = http_req(
        port,
        "GET",
        "/api/auth/users",
        None,
        &[("Cookie", &tech_cookie), ("X-Ergon-Gate", "BreakGlass-Secret-99")],
    );
    assert_eq!(
        escalate.status, 403,
        "non-admin session + break-glass must NOT elevate to admin: {}",
        escalate.body
    );
    let cross_sw = http_req(
        port,
        "POST",
        "/api/people/switch",
        Some(&json!({"id": admin_pid}).to_string()),
        &[("Cookie", &tech_cookie), ("X-Ergon-Gate", "BreakGlass-Secret-99")],
    );
    assert_eq!(
        cross_sw.status, 403,
        "non-admin session + break-glass must NOT bypass workspace isolation: {}",
        cross_sw.body
    );
}

#[test]
fn real_tls_https_handshake_and_phone_behind_tls_login_gate() {
    let root = test_root("real_tls");
    let cfg = test_config(&root);
    cfg.ensure_dirs().unwrap();
    let tls_dir = cfg.data_dir.join("tls");
    let (cert_path, key_path) = ergon_rs::tls::ensure_self_signed_cert(&tls_dir, &[]).unwrap();
    let tls_cfg = ergon_rs::tls::load_tls_config(&cert_path, &key_path, true).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let cfg_srv = cfg.clone();
    std::thread::spawn(move || {
        let _ = ergon_rs::serve_with_tls(listener, cfg_srv, Some(tls_cfg), false);
    });
    wait_ready_plain(port);

    // 1. Plaintext HTTP against TLS port is strictly rejected (400).
    let plain = http_req(port, "GET", "/api/health", None, &[]);
    assert_eq!(plain.status, 400, "plaintext HTTP must be rejected on TLS port: {}", plain.body);

    // 2. Real TLS handshake using rustls client pinned to generated self-signed cert.
    let client_cfg = tls_client_config(&cert_path);
    let me = https_req(port, &client_cfg, "GET", "/api/auth/me", None, &[]);
    assert_eq!(me.status, 200, "TLS /api/auth/me failed: {}", me.body);
    assert!(
        me.header("strict-transport-security").is_some(),
        "HSTS header must be set on TLS responses"
    );
    let me_j = me.json();
    assert_eq!(me_j["tls"], json!(true));
    assert_eq!(me_j["auth_required"], json!(true));
    assert_eq!(me_j["authenticated"], json!(false));
    assert_eq!(me_j["needs_setup"], json!(true));

    // 3. Unauthenticated /api/* over TLS -> 401; /phone returns static login shell with NO session cookie.
    assert_eq!(https_req(port, &client_cfg, "GET", "/api/status", None, &[]).status, 401);
    let phone_unauth = https_req(port, &client_cfg, "GET", "/phone", None, &[]);
    assert_eq!(phone_unauth.status, 200);
    assert!(
        phone_unauth.session_cookie_pair().is_none(),
        "unauthenticated /phone must NOT issue a session cookie"
    );
    let phone_head = https_req(port, &client_cfg, "HEAD", "/phone", None, &[]);
    assert_eq!(phone_head.status, 200);
    assert!(phone_head.body.is_empty());

    // 4. Setup admin over TLS -> cookie carries Secure + HttpOnly + SameSite=Lax!
    let setup = https_req(
        port,
        &client_cfg,
        "POST",
        "/api/auth/setup",
        Some(&json!({"username": "tlsadmin", "password": "TlsAdminPass2026!"}).to_string()),
        &[],
    );
    assert_eq!(setup.status, 201, "{}", setup.body);
    let raw_cookie = setup.set_cookies().join("; ");
    assert!(raw_cookie.contains("HttpOnly"), "{raw_cookie}");
    assert!(raw_cookie.contains("SameSite=Lax"), "{raw_cookie}");
    assert!(raw_cookie.contains("Secure"), "TLS cookie MUST carry Secure flag: {raw_cookie}");
    let cookie = setup.session_cookie_pair().unwrap();

    // 5. Authenticated /api/auth/me, /api/status, and /phone over TLS succeed.
    let authed_me = https_req(port, &client_cfg, "GET", "/api/auth/me", None, &[("Cookie", &cookie)]);
    assert_eq!(authed_me.status, 200);
    assert_eq!(authed_me.json()["authenticated"], json!(true));
    assert_eq!(authed_me.json()["user"]["username"], json!("tlsadmin"));
    assert_eq!(
        https_req(port, &client_cfg, "GET", "/api/status", None, &[("Cookie", &cookie)]).status,
        200
    );
    assert_eq!(
        https_req(port, &client_cfg, "GET", "/phone", None, &[("Cookie", &cookie)]).status,
        200
    );
}

#[test]
fn oidc_discovery_pkce_callback_end_to_end() {
    // Spin up a real local loopback OIDC provider implementing:
    // - GET /.well-known/openid-configuration
    // - POST /token (verifies code + PKCE S256 code_verifier + client_secret)
    // - GET /userinfo (verifies Bearer access_token)
    #[derive(Clone)]
    struct IssuedCode {
        code_challenge: String,
        nonce: String,
        sub: String,
        email: String,
        name: String,
        role: String,
        tamper_nonce: bool,
    }

    let idp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let idp_port = idp_listener.local_addr().unwrap().port();
    let issuer = format!("http://127.0.0.1:{idp_port}");
    let codes: Arc<Mutex<HashMap<String, IssuedCode>>> = Arc::new(Mutex::new(HashMap::new()));
    let tokens: Arc<Mutex<HashMap<String, IssuedCode>>> = Arc::new(Mutex::new(HashMap::new()));

    let codes_srv = codes.clone();
    let tokens_srv = tokens.clone();
    let issuer_srv = issuer.clone();
    std::thread::spawn(move || {
        for incoming in idp_listener.incoming() {
            let Ok(mut s) = incoming else { continue };
            let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let mut raw_bytes = Vec::new();
            let mut chunk = [0u8; 4096];
            let mut header_end: Option<usize> = None;
            let mut content_len: usize = 0;
            loop {
                match s.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        raw_bytes.extend_from_slice(&chunk[..n]);
                        if header_end.is_none() {
                            if let Some(pos) = raw_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                                header_end = Some(pos + 4);
                                let head_s = String::from_utf8_lossy(&raw_bytes[..pos]);
                                for line in head_s.lines() {
                                    if let Some((k, v)) = line.split_once(':') {
                                        if k.trim().eq_ignore_ascii_case("content-length") {
                                            content_len = v.trim().parse::<usize>().unwrap_or(0);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(he) = header_end {
                            if raw_bytes.len() >= he + content_len {
                                break;
                            }
                        }
                    }
                }
            }
            if raw_bytes.is_empty() {
                continue;
            }
            let raw = String::from_utf8_lossy(&raw_bytes);
            let first = raw.lines().next().unwrap_or("");
            let mut parts = first.split_whitespace();
            let method = parts.next().unwrap_or("");
            let path = parts.next().unwrap_or("");
            let body = raw.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");

            let write_json = |s: &mut TcpStream, status: u16, v: &Value| {
                let b = v.to_string();
                let resp = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}",
                    b.len()
                );
                let _ = s.write_all(resp.as_bytes());
                let _ = s.flush();
            };

            if method == "GET" && path == "/.well-known/openid-configuration" {
                write_json(
                    &mut s,
                    200,
                    &json!({
                        "issuer": issuer_srv,
                        "authorization_endpoint": format!("{issuer_srv}/authorize"),
                        "token_endpoint": format!("{issuer_srv}/token"),
                        "userinfo_endpoint": format!("{issuer_srv}/userinfo"),
                        "response_types_supported": ["code"],
                        "code_challenge_methods_supported": ["S256"],
                    }),
                );
            } else if method == "POST" && path == "/token" {
                let mut form: HashMap<String, String> = HashMap::new();
                for pair in body.split('&') {
                    if let Some((k, v)) = pair.split_once('=') {
                        form.insert(k.to_string(), v.to_string());
                    }
                }
                let code = form.get("code").cloned().unwrap_or_default();
                let verifier = form.get("code_verifier").cloned().unwrap_or_default();
                let secret = form.get("client_secret").cloned().unwrap_or_default();
                let issued = codes_srv.lock().unwrap().remove(&code);
                let Some(ic) = issued else {
                    write_json(&mut s, 400, &json!({"error": "invalid_grant"}));
                    continue;
                };
                if secret != "test-oidc-secret" {
                    write_json(&mut s, 401, &json!({"error": "invalid_client"}));
                    continue;
                }
                // Verify PKCE S256: base64url(sha256(code_verifier)) == code_challenge
                let expected_challenge = ergon_rs::auth::base64url_encode_nopad(
                    &ergon_rs::tripwire::sha256(verifier.as_bytes()),
                );
                if expected_challenge != ic.code_challenge {
                    write_json(&mut s, 400, &json!({"error": "invalid_pkce_verifier"}));
                    continue;
                }
                let jwt_header = ergon_rs::auth::base64url_encode_nopad(br#"{"alg":"none","typ":"JWT"}"#);
                let exp = (ergon_rs::records::now_millis() / 1000) + 3600;
                let nonce_out = if ic.tamper_nonce {
                    "wrong-nonce".to_string()
                } else {
                    ic.nonce.clone()
                };
                let jwt_payload = ergon_rs::auth::base64url_encode_nopad(
                    &json!({
                        "iss": issuer_srv,
                        "sub": ic.sub,
                        "aud": "ergon-test-client",
                        "exp": exp,
                        "nonce": nonce_out,
                        "email": ic.email,
                        "name": ic.name,
                        "role": ic.role,
                    })
                    .to_string()
                    .into_bytes(),
                );
                let id_token = format!("{jwt_header}.{jwt_payload}.sig");
                let access_token = format!("at_{code}");
                tokens_srv.lock().unwrap().insert(access_token.clone(), ic);
                write_json(
                    &mut s,
                    200,
                    &json!({
                        "access_token": access_token,
                        "token_type": "Bearer",
                        "id_token": id_token,
                    }),
                );
            } else if method == "GET" && path == "/userinfo" {
                let auth_hdr = raw
                    .lines()
                    .find(|l| l.to_ascii_lowercase().starts_with("authorization:"))
                    .and_then(|l| l.split_once(':'))
                    .map(|(_, v)| v.trim().to_string())
                    .unwrap_or_default();
                let tok = auth_hdr.strip_prefix("Bearer ").unwrap_or("").trim();
                let info = tokens_srv.lock().unwrap().get(tok).cloned();
                if let Some(ic) = info {
                    write_json(
                        &mut s,
                        200,
                        &json!({
                            "sub": ic.sub,
                            "email": ic.email,
                            "name": ic.name,
                            "role": ic.role,
                        }),
                    );
                } else {
                    write_json(&mut s, 401, &json!({"error": "invalid_token"}));
                }
            }
        }
    });

    // Boot Ergon, setup initial admin, and configure OIDC via admin API
    let root = test_root("oidc_e2e");
    let cfg = test_config(&root);
    let port = start_plain(cfg, None);
    let setup = http_req(
        port,
        "POST",
        "/api/auth/setup",
        Some(&json!({"username": "admin", "password": "AdminPassword123"}).to_string()),
        &[],
    );
    assert_eq!(setup.status, 201);
    let admin_cookie = setup.session_cookie_pair().unwrap();

    let oidc_save = http_req(
        port,
        "PUT",
        "/api/auth/oidc/config",
        Some(
            &json!({
                "enabled": true,
                "issuer": issuer,
                "client_id": "ergon-test-client",
                "client_secret": "test-oidc-secret",
                "redirect_uri": format!("http://127.0.0.1:{port}/api/auth/oidc/callback"),
                "default_role": "member",
                "role_claim": "role",
                "auto_provision": true
            })
            .to_string(),
        ),
        &[("Cookie", &admin_cookie)],
    );
    assert_eq!(oidc_save.status, 200, "save oidc: {}", oidc_save.body);
    assert_eq!(oidc_save.json()["configured"], json!(true));

    // Helper to parse query params from Location URL
    let parse_query = |url: &str| -> HashMap<String, String> {
        let q = url.split_once('?').map(|(_, r)| r).unwrap_or("");
        q.split('&')
            .filter_map(|p| p.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    };

    // 1. Start OIDC login -> 302 to IdP with state, nonce, PKCE S256 code_challenge
    let login_redir = http_req(port, "GET", "/api/auth/oidc/login?next=/phone", None, &[]);
    assert_eq!(login_redir.status, 302, "{}", login_redir.body);
    let loc = login_redir.header("location").expect("Location header missing");
    assert!(loc.starts_with(&format!("{issuer}/authorize?")), "{loc}");
    let q = parse_query(loc);
    assert_eq!(q.get("code_challenge_method").map(String::as_str), Some("S256"));
    let state_1 = q.get("state").unwrap().clone();
    let nonce_1 = q.get("nonce").unwrap().clone();
    let challenge_1 = q.get("code_challenge").unwrap().clone();
    assert!(!state_1.is_empty() && !nonce_1.is_empty() && !challenge_1.is_empty());

    // 2. Tampered state must be refused (400)
    let bad_state = http_req(
        port,
        "GET",
        "/api/auth/oidc/callback?code=any&state=forged-state",
        None,
        &[],
    );
    assert_eq!(bad_state.status, 400);

    // 3. Tampered nonce in ID token must be refused (401)
    codes.lock().unwrap().insert(
        "code_bad_nonce".to_string(),
        IssuedCode {
            code_challenge: challenge_1.clone(),
            nonce: nonce_1.clone(),
            sub: "oidc-sub-bad".to_string(),
            email: "bad@example.com".to_string(),
            name: "Bad Nonce".to_string(),
            role: "office".to_string(),
            tamper_nonce: true,
        },
    );
    let bad_nonce_cb = http_req(
        port,
        "GET",
        &format!("/api/auth/oidc/callback?code=code_bad_nonce&state={state_1}"),
        None,
        &[],
    );
    assert_eq!(bad_nonce_cb.status, 401, "tampered nonce must be rejected: {}", bad_nonce_cb.body);

    // 4. Full happy path: PKCE S256 + state + nonce + userinfo -> auto-provisions "office" user + cookie + redirect /phone
    let login_redir2 = http_req(port, "GET", "/api/auth/oidc/login?next=/phone", None, &[]);
    assert_eq!(login_redir2.status, 302);
    let q2 = parse_query(login_redir2.header("location").unwrap());
    let state_2 = q2.get("state").unwrap().clone();
    let nonce_2 = q2.get("nonce").unwrap().clone();
    let challenge_2 = q2.get("code_challenge").unwrap().clone();
    codes.lock().unwrap().insert(
        "code_good_2".to_string(),
        IssuedCode {
            code_challenge: challenge_2,
            nonce: nonce_2,
            sub: "oidc-sub-777".to_string(),
            email: "olivia.office@example.com".to_string(),
            name: "Olivia Office".to_string(),
            role: "office".to_string(),
            tamper_nonce: false,
        },
    );
    let ok_cb = http_req(
        port,
        "GET",
        &format!("/api/auth/oidc/callback?code=code_good_2&state={state_2}"),
        None,
        &[],
    );
    assert_eq!(ok_cb.status, 302, "OIDC callback failed: {}", ok_cb.body);
    assert_eq!(ok_cb.header("location"), Some("/phone"));
    let oidc_cookie = ok_cb.session_cookie_pair().expect("OIDC session cookie missing");

    // 5. Replaying the same state must fail (single-use)
    let replay = http_req(
        port,
        "GET",
        &format!("/api/auth/oidc/callback?code=code_good_2&state={state_2}"),
        None,
        &[],
    );
    assert_eq!(replay.status, 400, "replayed OIDC state must fail");

    // 6. Verify OIDC session authenticates as Olivia (role: office) and enforces RBAC!
    let me = http_req(port, "GET", "/api/auth/me", None, &[("Cookie", &oidc_cookie)]).json();
    assert_eq!(me["authenticated"], json!(true));
    assert_eq!(me["user"]["role"], json!("office"));
    assert_eq!(me["user"]["auth_method"], json!("oidc"));
    assert_eq!(me["user"]["email"], json!("olivia.office@example.com"));
    // Office role can view invoices (200) but cannot hit admin endpoints (403)
    assert_eq!(http_req(port, "GET", "/api/invoices", None, &[("Cookie", &oidc_cookie)]).status, 200);
    assert_eq!(http_req(port, "GET", "/api/auth/users", None, &[("Cookie", &oidc_cookie)]).status, 403);
}

#[test]
fn cli_user_and_tls_init_commands() {
    let root = test_root("cli_auth_tls");
    let data = root.join("data");
    let bin = env!("CARGO_BIN_EXE_ergon");

    // 1. ergon user add admin
    let out1 = std::process::Command::new(bin)
        .args(["user", "add", "opsadmin", "--role", "admin", "--password", "CliAdminPass123", "--json"])
        .env("ERGON_DATA_DIR", &data)
        .output()
        .unwrap();
    assert!(out1.status.success(), "{}", String::from_utf8_lossy(&out1.stderr));
    let j1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    assert_eq!(j1["user"]["username"], json!("opsadmin"));
    assert_eq!(j1["user"]["role"], json!("admin"));

    // 2. ergon user list --json
    let out2 = std::process::Command::new(bin)
        .args(["user", "list", "--json"])
        .env("ERGON_DATA_DIR", &data)
        .output()
        .unwrap();
    assert!(out2.status.success());
    let j2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(j2["users"].as_array().unwrap().len(), 1);

    // 3. ergon user passwd
    let out3 = std::process::Command::new(bin)
        .args(["user", "passwd", "opsadmin", "--password", "NewCliAdminPass456", "--json"])
        .env("ERGON_DATA_DIR", &data)
        .output()
        .unwrap();
    assert!(out3.status.success(), "{}", String::from_utf8_lossy(&out3.stderr));
    let store = ergon_rs::auth::AuthStore::load(&data);
    assert!(store.login_local("opsadmin", "CliAdminPass123", "test").is_err());
    assert!(store.login_local("opsadmin", "NewCliAdminPass456", "test").is_ok());

    // 4. ergon tls init --json
    let out4 = std::process::Command::new(bin)
        .args(["tls", "init", "--json"])
        .env("ERGON_DATA_DIR", &data)
        .output()
        .unwrap();
    assert!(out4.status.success(), "{}", String::from_utf8_lossy(&out4.stderr));
    let j4: Value = serde_json::from_slice(&out4.stdout).unwrap();
    assert_eq!(j4["ok"], json!(true));
    assert!(data.join("tls/cert.pem").is_file());
    assert!(data.join("tls/key.pem").is_file());
}
