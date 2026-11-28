//! Integration tests for Proactive App Recognition & Auto-Recommended Plugin Setups.

use ergon_rs::{plugins, serve, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
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
            "ergon-plugin-recs-{}-{}-{label}",
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

    fn person_dir(&self) -> PathBuf {
        // When server runs with a fresh data_dir, Active Person defaults to `p0` or root depending on fixture;
        // Let's seed on root AND resolve the server's active person dir via /api/status if needed,
        // or inspect both.
        self.cfg.data_dir.clone()
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
        .expect("HTTP header separator");
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

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[test]
fn detects_apps_safely_persists_dismissals_and_syncs_with_offers() {
    let _g_disable = EnvVarGuard::set("ERGON_DISABLE_HOST_APP_SCAN", "1");
    let _g_host = EnvVarGuard::unset("ERGON_TEST_HOST_APPS");

    let td = TestDir::new("e2e");
    let person_dir = td.person_dir();
    let ws = person_dir.join("workspace");

    // 1) Workspace files & URLs + sensitive file exclusion check (.env must be ignored)
    write_file(
        &ws.join("ops-runbook.md"),
        "Track roadmap in https://linear.app/acme/ENG-9 and review https://docs.google.com/spreadsheets/d/xyz",
    );
    write_file(&ws.join("schema.sql"), "SELECT * FROM invoices;\n");
    write_file(&ws.join("wireframe.fig"), "FIGMA-BINARY-PLACEHOLDER");
    write_file(
        &ws.join(".env"),
        "HUBSPOT_PORTAL=https://app.hubspot.com/contacts\nHUBSPOT_SECRET=supersecret\n",
    );
    write_file(
        &ws.join("api-credentials.txt"),
        "https://app.hubspot.com/contacts should NOT be scanned from credential files",
    );

    // 2) Conversations + ordinary prose false-positive guard + memory.md
    write_file(
        &person_dir.join("threads.json"),
        &json!({
            "threads": [{
                "id": "t-ops-1",
                "title": "Weekly ops sync",
                "messages": [
                    { "role": "user", "content": "Post summaries in our Slack channel #ops once done." },
                    { "role": "assistant", "content": "Also pick up the slack with a linear trend and zoom in." }
                ]
            }]
        })
        .to_string(),
    );
    write_file(
        &person_dir.join("memory.md"),
        "# Notes\n- Escalate billing tickets in Zendesk on Fridays.\n",
    );

    let addr = spawn_server(td.cfg.clone());
    let (st, recs_body, raw) = http_req(addr, "GET", "/api/plugins/recommendations", "");
    assert_eq!(st, 200, "{raw}");
    let recs = recs_body["recommendations"].as_array().expect("recommendations array");
    let ids: Vec<&str> = recs.iter().filter_map(|r| r["plugin_id"].as_str()).collect();

    for expected in ["linear", "google-sheets", "postgres-sqlite", "figma", "slack", "zendesk"] {
        assert!(
            ids.contains(&expected),
            "expected {expected} in recommendations, got {ids:?}"
        );
    }
    // Secret/credential files must never be read, and ordinary prose must not trigger Zoom.
    assert!(!ids.contains(&"hubspot"), "sensitive files leaked into detection: {ids:?}");
    assert!(!ids.contains(&"zoom"), "false positive zoom in prose: {ids:?}");

    // Verify metadata quality on Linear recommendation
    let lin = recs.iter().find(|r| r["plugin_id"] == "linear").unwrap();
    assert_eq!(lin["confidence"], "high");
    assert_eq!(lin["setup_kind"], "one_click_connect");
    assert!(!lin["reason_summary"].as_str().unwrap_or("").is_empty());

    // 3) Enabling a local-ready plugin excludes it from recommendations immediately
    let (st, _, raw) = http_req(
        addr,
        "POST",
        "/api/plugins/toggle",
        r#"{"plugin_id":"google-sheets","enabled":true}"#,
    );
    assert_eq!(st, 200, "{raw}");

    let (st, after_enable, _) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st, 200);
    let after_enable_ids: Vec<&str> = after_enable["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["plugin_id"].as_str())
        .collect();
    assert!(!after_enable_ids.contains(&"google-sheets"));

    // 4) Dismiss persistence (0600) + reset
    let (st, dismissed_body, raw) = http_req(
        addr,
        "POST",
        "/api/plugins/recommendations/dismiss",
        r#"{"plugin_id":"figma"}"#,
    );
    assert_eq!(st, 200, "{raw}");
    let after_dismiss_ids: Vec<&str> = dismissed_body["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["plugin_id"].as_str())
        .collect();
    assert!(!after_dismiss_ids.contains(&"figma"));
    assert_eq!(dismissed_body["dismissed_count"], json!(1));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(plugins::plugins_path(&person_dir))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "plugins.json must remain 0600");
    }

    // 5) Proactive offers analyzer emits deduplicated plugin_setup offers and syncs dismissal
    let (st, analyze_body, raw) = http_req(addr, "POST", "/api/offers/analyze", "{}");
    assert_eq!(st, 200, "{raw}");
    let plugin_offers: Vec<&Value> = analyze_body["all_offers"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["kind"] == "plugin_setup" && o["status"] == "pending")
        .collect();
    assert!(
        !plugin_offers.is_empty(),
        "expected plugin_setup offer in analyze response: {analyze_body}"
    );
    let first_offer_id = plugin_offers[0]["id"].as_str().unwrap().to_string();
    let offered_pid = plugin_offers[0]["plugin_id"].as_str().unwrap().to_string();

    let (st, _, raw) = http_req(
        addr,
        "POST",
        &format!("/api/offers/{first_offer_id}/dismiss"),
        "{}",
    );
    assert_eq!(st, 200, "{raw}");

    let (st, after_offer_dismiss, _) = http_req(addr, "GET", "/api/plugins/recommendations", "");
    assert_eq!(st, 200);
    let remaining: Vec<&str> = after_offer_dismiss["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["plugin_id"].as_str())
        .collect();
    assert!(
        !remaining.contains(&offered_pid.as_str()),
        "dismissing plugin_setup offer must also dismiss recommendation for {offered_pid}"
    );

    // Resetting dismissed recommendations brings figma back cleanly
    let (st, reset_body, raw) = http_req(
        addr,
        "POST",
        "/api/plugins/recommendations/dismiss",
        r#"{"reset":true}"#,
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(reset_body["dismissed_count"], json!(0));
}

#[test]
fn detects_simulated_desktop_apps_and_respects_disable_flag() {
    let td = TestDir::new("host-apps");
    let addr = spawn_server(td.cfg.clone());

    let _g_disable_off = EnvVarGuard::unset("ERGON_DISABLE_HOST_APP_SCAN");
    let _g_host_apps = EnvVarGuard::set("ERGON_TEST_HOST_APPS", "notion,zoom");
    let (st, scan_body, raw) = http_req(addr, "POST", "/api/plugins/recommendations/scan", "{}");
    assert_eq!(st, 200, "{raw}");
    let scan_ids: Vec<&str> = scan_body["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["plugin_id"].as_str())
        .collect();
    assert!(
        scan_ids.contains(&"notion") && scan_ids.contains(&"zoom"),
        "expected notion and zoom in {scan_ids:?}"
    );

    let _g_disable = EnvVarGuard::set("ERGON_DISABLE_HOST_APP_SCAN", "1");
    let (st, disabled_body, raw) = http_req(addr, "POST", "/api/plugins/recommendations/scan", "{}");
    assert_eq!(st, 200, "{raw}");
    assert_eq!(
        disabled_body["recommendations"].as_array().unwrap().len(),
        0,
        "ERGON_DISABLE_HOST_APP_SCAN=1 should suppress host app recommendations"
    );
}
