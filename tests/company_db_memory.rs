//! Wave 2 Step 3a Integration Tests: Shared Company DB & Role-Scoped Company Memory.

use ergon_rs::{
    execute_tool_full, library, records, serve_with_gate, system_prompt, Config, Person, ToolCtx,
};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    root: PathBuf,
    cfg: Config,
}

impl TestEnv {
    fn new(label: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "company-db-memory-{}-{}-{label}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "stub".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct HttpResp {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpResp {
    fn session_cookie(&self) -> Option<String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case("set-cookie") {
                if let Some(first) = v.split(';').next() {
                    if first.trim().starts_with("ergon_session=") {
                        return Some(first.trim().to_string());
                    }
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

fn spawn_server(cfg: Config) -> SocketAddr {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve_with_gate(listener, cfg, None);
    });
    for _ in 0..50 {
        if TcpStream::connect(addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    addr
}

fn http_req(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
    cookie: Option<&str>,
) -> HttpResp {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let bytes = body.unwrap_or("").as_bytes();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        addr.port()
    );
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
    }
    if let Some(c) = cookie {
        req.push_str(&format!("Cookie: {c}\r\n"));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !bytes.is_empty() {
        stream.write_all(bytes).unwrap();
    }
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw).to_string();
    let (head, resp_body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .collect();
    HttpResp {
        status,
        headers,
        body: resp_body.to_string(),
    }
}

fn login(addr: SocketAddr, username: &str, password: &str) -> String {
    let r = http_req(
        addr,
        "POST",
        "/api/auth/login",
        Some(&json!({ "username": username, "password": password }).to_string()),
        None,
    );
    assert_eq!(r.status, 200, "login failed for {username}: {}", r.body);
    r.session_cookie().expect("session cookie missing")
}

fn setup_multi_user(addr: SocketAddr) -> (String, String, String) {
    let setup = http_req(
        addr,
        "POST",
        "/api/auth/setup",
        Some(
            &json!({
                "username": "alice_admin",
                "password": "AdminPassword!123",
                "display_name": "Alice Admin"
            })
            .to_string(),
        ),
        None,
    );
    assert_eq!(setup.status, 201, "setup failed: {}", setup.body);
    let admin_cookie = setup.session_cookie().unwrap();

    for (u, role, name) in [
        ("carol_pm", "pm", "Carol PM"),
        ("bob_inst", "installer", "Bob Installer"),
    ] {
        let cr = http_req(
            addr,
            "POST",
            "/api/auth/users",
            Some(
                &json!({
                    "username": u,
                    "password": "UserPassword!123",
                    "role": role,
                    "display_name": name
                })
                .to_string(),
            ),
            Some(&admin_cookie),
        );
        assert_eq!(cr.status, 201, "create user {u}: {}", cr.body);
    }

    let pm_cookie = login(addr, "carol_pm", "UserPassword!123");
    let inst_cookie = login(addr, "bob_inst", "UserPassword!123");
    (admin_cookie, pm_cookie, inst_cookie)
}

fn person_id_for(addr: SocketAddr, cookie: &str) -> String {
    let me = http_req(addr, "GET", "/api/auth/me", None, Some(cookie));
    assert_eq!(me.status, 200);
    me.json()["user"]["person_id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn find_person_cfg(root_cfg: &Config, person_id: &str) -> Config {
    if person_id == "default" {
        root_cfg.clone()
    } else {
        root_cfg.person_config(&Person {
            id: person_id.to_string(),
            name: person_id.to_string(),
            role: String::new(),
        })
    }
}

#[test]
fn company_memory_isolation_rbac_client_scopes_and_restart_persistence() {
    let env = TestEnv::new("full");
    let addr = spawn_server(env.cfg.clone());
    let (admin_cookie, pm_cookie, inst_cookie) = setup_multi_user(addr);

    let alice_pid = person_id_for(addr, &admin_cookie);
    let carol_pid = person_id_for(addr, &pm_cookie);
    let bob_pid = person_id_for(addr, &inst_cookie);

    // Seed private memory.md entries that must NEVER leak into company memory/DB.
    let alice_cfg = find_person_cfg(&env.cfg, &alice_pid);
    let bob_cfg = find_person_cfg(&env.cfg, &bob_pid);
    let carol_cfg = find_person_cfg(&env.cfg, &carol_pid);
    alice_cfg.ensure_dirs().unwrap();
    bob_cfg.ensure_dirs().unwrap();
    carol_cfg.ensure_dirs().unwrap();
    std::fs::write(&alice_cfg.memory_path, "# Memory\n- PRIVATE_ALICE_TOKEN_991\n").unwrap();
    std::fs::write(&bob_cfg.memory_path, "# Memory\n- PRIVATE_BOB_NOTE_772\n").unwrap();

    // PM creates Acme client (owned by Carol) and Admin creates Beta client (owned by Alice).
    let acme_res = http_req(
        addr,
        "POST",
        "/api/clients",
        Some(&json!({ "id": "acme", "name": "Acme Corp" }).to_string()),
        Some(&pm_cookie),
    );
    assert_eq!(acme_res.status, 201, "create acme: {}", acme_res.body);

    let beta_res = http_req(
        addr,
        "POST",
        "/api/clients",
        Some(&json!({ "id": "beta", "name": "Beta LLC" }).to_string()),
        Some(&admin_cookie),
    );
    assert_eq!(beta_res.status, 201, "create beta: {}", beta_res.body);

    // Installer (Bob) gets 403 when attempting to write or delete company memory, or read clients table.
    let deny_write = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(
            &json!({
                "category": "fact",
                "scope": "company",
                "content": "Unauthorized installer entry"
            })
            .to_string(),
        ),
        Some(&inst_cookie),
    );
    assert_eq!(deny_write.status, 403, "installer write must be 403: {}", deny_write.body);

    let deny_clients_db = http_req(
        addr,
        "GET",
        "/api/company/db?table=clients",
        None,
        Some(&inst_cookie),
    );
    assert_eq!(
        deny_clients_db.status, 403,
        "installer clients query must be 403: {}",
        deny_clients_db.body
    );

    // PM (Carol) writes company-wide, role:installer, client:acme, and client:beta entries.
    let r_comp = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(
            &json!({
                "category": "policy",
                "scope": "company",
                "content": "Standard payment terms are Net-30."
            })
            .to_string(),
        ),
        Some(&pm_cookie),
    );
    assert_eq!(r_comp.status, 201, "pm company write: {}", r_comp.body);
    let comp_id = r_comp.json()["entry"]["id"].as_str().unwrap().to_string();

    let r_role = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(
            &json!({
                "category": "procedure",
                "scope": "role:installer",
                "content": "Verify breaker panel photo before handoff."
            })
            .to_string(),
        ),
        Some(&pm_cookie),
    );
    assert_eq!(r_role.status, 201, "pm role write: {}", r_role.body);

    let r_acme = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(
            &json!({
                "category": "preference",
                "scope": "client:acme",
                "content": "Acme requires PDF reports every Friday."
            })
            .to_string(),
        ),
        Some(&pm_cookie),
    );
    assert_eq!(r_acme.status, 201, "pm client:acme write: {}", r_acme.body);

    let r_beta = http_req(
        addr,
        "POST",
        "/api/company/memory",
        Some(
            &json!({
                "category": "fact",
                "scope": "client:beta",
                "content": "Beta NDA addendum signed."
            })
            .to_string(),
        ),
        Some(&admin_cookie),
    );
    assert_eq!(r_beta.status, 201, "admin client:beta write: {}", r_beta.body);

    // Secret & host path rejection.
    for bad in [
        "Use key sk-1234567890secret for auth",
        "Token ergk_secret_token_abc",
        "Check /home/cayleb/keys/id_rsa",
        "Login with password=hunter2",
    ] {
        let rej = http_req(
            addr,
            "POST",
            "/api/company/memory",
            Some(&json!({ "category": "fact", "scope": "company", "content": bad }).to_string()),
            Some(&admin_cookie),
        );
        assert_eq!(rej.status, 400, "secret/path must be rejected for {bad}: {}", rej.body);
    }

    // Visibility matrix:
    // - Installer Bob sees company + role:installer ONLY (never client:acme, client:beta, or private memory.md).
    let bob_mem = http_req(addr, "GET", "/api/company/memory", None, Some(&inst_cookie));
    assert_eq!(bob_mem.status, 200);
    let bob_contents: Vec<String> = bob_mem.json()["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["content"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(bob_contents.len(), 2, "bob entries: {bob_contents:?}");
    assert!(bob_contents.iter().any(|c| c.contains("Net-30")));
    assert!(bob_contents.iter().any(|c| c.contains("breaker panel")));
    assert!(!bob_mem.body.contains("Acme requires PDF"));
    assert!(!bob_mem.body.contains("Beta NDA"));
    assert!(!bob_mem.body.contains("PRIVATE_ALICE_TOKEN_991"));
    assert!(!bob_mem.body.contains("PRIVATE_BOB_NOTE_772"));

    // - PM Carol sees company + client:acme (she owns acme, NOT beta, NOT role:installer).
    let carol_mem = http_req(addr, "GET", "/api/company/memory", None, Some(&pm_cookie));
    assert_eq!(carol_mem.status, 200);
    assert!(carol_mem.body.contains("Net-30"));
    assert!(carol_mem.body.contains("Acme requires PDF"));
    assert!(!carol_mem.body.contains("Beta NDA"));
    assert!(!carol_mem.body.contains("breaker panel"));

    // - Admin Alice sees all 4 entries, still ZERO private memory.md leaks.
    let alice_mem = http_req(addr, "GET", "/api/company/memory", None, Some(&admin_cookie));
    assert_eq!(alice_mem.status, 200);
    assert_eq!(alice_mem.json()["entries"].as_array().unwrap().len(), 4);
    assert!(!alice_mem.body.contains("PRIVATE_ALICE_TOKEN_991"));
    assert!(!alice_mem.body.contains("PRIVATE_BOB_NOTE_772"));

    // Installer delete must return 403.
    let deny_del = http_req(
        addr,
        "DELETE",
        &format!("/api/company/memory/{comp_id}"),
        None,
        Some(&inst_cookie),
    );
    assert_eq!(deny_del.status, 403);

    // Work log cross-person isolation in company DB.
    records::append_work_with_usage(
        &alice_cfg,
        "Alice confidential payroll run",
        "Completed payroll",
        &[],
        "t-alice",
        true,
        None,
        None,
    );
    records::append_work_with_usage(
        &bob_cfg,
        "Bob site survey",
        "Survey done",
        &[],
        "t-bob",
        true,
        None,
        None,
    );

    let bob_work = http_req(addr, "GET", "/api/company/db?table=work", None, Some(&inst_cookie));
    assert_eq!(bob_work.status, 200);
    assert!(bob_work.body.contains("Bob site survey"));
    assert!(
        !bob_work.body.contains("Alice confidential payroll"),
        "bob must not see alice's work rows: {}",
        bob_work.body
    );

    let alice_work = http_req(
        addr,
        "POST",
        "/api/company/db/query",
        Some(&json!({ "table": "work", "limit": 50 }).to_string()),
        Some(&admin_cookie),
    );
    assert_eq!(alice_work.status, 200);
    assert!(alice_work.body.contains("Alice confidential payroll"));
    assert!(alice_work.body.contains("Bob site survey"));

    // System prompt separation: authorized company memory vs private memory.md.
    let bob_prompt = system_prompt(&bob_cfg, &[]);
    assert!(bob_prompt.contains("COMPANY-WIDE SHARED MEMORY"));
    assert!(bob_prompt.contains("WHAT YOU KNOW ABOUT THIS PERSON"));
    assert!(bob_prompt.contains("PRIVATE_BOB_NOTE_772"));
    assert!(bob_prompt.contains("breaker panel"));
    assert!(!bob_prompt.contains("Acme requires PDF"));
    assert!(!bob_prompt.contains("PRIVATE_ALICE_TOKEN_991"));

    // Agent tool execution: spoofed role in args must be ignored; AuthStore real role enforced!
    let lib = Arc::new(Mutex::new(library::Library::load(&env.cfg.data_dir)));
    let bob_ctx = ToolCtx {
        library: lib.clone(),
        root_data_dir: env.cfg.data_dir.clone(),
        person_id: bob_pid.clone(),
    };
    let spoof_err = execute_tool_full(
        &bob_cfg,
        &bob_ctx,
        "write_company_memory",
        &json!({
            "role": "admin",
            "category": "fact",
            "scope": "company",
            "content": "Spoofed admin write"
        }),
    )
    .unwrap_err();
    assert!(
        spoof_err.contains("forbidden"),
        "spoofed role must fail with forbidden: {spoof_err}"
    );

    let bob_read_tool = execute_tool_full(&bob_cfg, &bob_ctx, "read_company_memory", &json!({})).unwrap();
    let bob_read_v: Value = serde_json::from_str(&bob_read_tool).unwrap();
    assert_eq!(bob_read_v["role"], "installer");
    assert_eq!(bob_read_v["count"], 2);

    let bob_db_tool = execute_tool_full(
        &bob_cfg,
        &bob_ctx,
        "query_company_db",
        &json!({ "table": "clients" }),
    )
    .unwrap_err();
    assert!(bob_db_tool.contains("forbidden"));

    // Restart persistence on a fresh server instance against the same data_dir.
    let addr2 = spawn_server(env.cfg.clone());
    let admin_cookie2 = login(addr2, "alice_admin", "AdminPassword!123");
    let persisted = http_req(addr2, "GET", "/api/company/memory", None, Some(&admin_cookie2));
    assert_eq!(persisted.status, 200);
    assert_eq!(persisted.json()["entries"].as_array().unwrap().len(), 4);
    assert!(persisted.body.contains("Net-30"));

    // POSIX 0600 permissions on company_memory.json.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(Path::new(&env.cfg.data_dir).join("company/company_memory.json")).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }
}
