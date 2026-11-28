//! Integration tests for Ergon's Auto-Updating Default App Plugin Library:
//! - 32 starter workplace plugins across Google Workspace (6), Microsoft 365 & Outlook (6),
//!   Business Apps (18), and On-Device Local (2)
//! - Registry status block included in `GET /api/plugins`
//! - Atomic 0600 `plugins.json` persistence for enable + preferred_mode (`auto` | `local` | `cloud`)
//! - On-device local & hybrid execution (`list_workspace`, `office-excel.build_sheet`)
//! - `/phone` & Today `Approve/Deny` gating for consequential external writes (`plugin_action`)
//! - Automatic detection of connected Microsoft 365 / Outlook & Gmail connectors per person

use ergon_rs::{plugins, serve, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static SEQ: AtomicU64 = AtomicU64::new(0);

struct PluginTestDir {
    root: PathBuf,
    cfg: Config,
}

impl PluginTestDir {
    fn new(label: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-plugins-{}-{}-{label}",
            std::process::id(),
            id
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

impl Drop for PluginTestDir {
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

#[test]
fn plugins_catalog_routes_and_0600_persistence() {
    let td = PluginTestDir::new("catalog");
    let addr = spawn_server(td.cfg.clone());

    let (st, body, raw) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st, 200, "{raw}");
    let items = body["plugins"].as_array().expect("plugins array");
    assert_eq!(items.len(), 32, "expected 32 default plugins: {items:?}");
    assert_eq!(body["summary"]["total"], json!(32));
    assert_eq!(body["summary"]["enabled_count"], json!(5));
    assert!(body["registry"]["auto_update"].is_string(), "missing registry block: {body}");

    let count_suite = |suite: &str| {
        items
            .iter()
            .filter(|p| p["suite"].as_str() == Some(suite))
            .count()
    };
    assert_eq!(count_suite("google"), 6);
    assert_eq!(count_suite("microsoft"), 6);
    assert_eq!(count_suite("business"), 18);
    assert_eq!(count_suite("local"), 2);

    for id in [
        "google-gmail",
        "google-calendar",
        "google-drive",
        "google-docs",
        "google-sheets",
        "google-slides",
        "outlook-mail",
        "outlook-calendar",
        "onedrive-sharepoint",
        "microsoft-teams",
        "office-excel",
        "office-word-pdf",
        "slack",
        "notion",
        "github",
        "crm-billing",
        "linear",
        "jira",
        "asana",
        "confluence",
        "figma",
        "zoom",
        "hubspot",
        "salesforce",
        "stripe",
        "quickbooks",
        "zendesk",
        "airtable",
        "shopify",
        "postgres-sqlite",
        "local-workspace-files",
        "local-mcp-bridge",
    ] {
        assert!(
            items.iter().any(|p| p["id"].as_str() == Some(id)),
            "missing starter plugin {id}"
        );
    }

    let (st2, toggled, raw2) = http_req(
        addr,
        "POST",
        "/api/plugins/toggle",
        &json!({
            "plugin_id": "google-drive",
            "enabled": true,
            "preferred_mode": "local"
        })
        .to_string(),
    );
    assert_eq!(st2, 200, "{raw2}");
    assert_eq!(toggled["plugin"]["enabled"], json!(true));
    assert_eq!(toggled["plugin"]["preferred_mode"], json!("local"));
    assert_eq!(toggled["plugin"]["effective_mode"], json!("local"));

    let ppath = plugins::plugins_path(&td.cfg.data_dir);
    assert!(ppath.exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&ppath).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "plugins.json must be 0600, got {mode:o}");
    }

    let (st3, after, _) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st3, 200);
    assert_eq!(after["summary"]["enabled_count"], json!(6));
}

#[test]
fn plugins_hybrid_local_execution_and_approval_gated_writes() {
    let td = PluginTestDir::new("hybrid");
    std::fs::write(td.cfg.workspace_dir.join("notes.txt"), "hello workspace\n").unwrap();
    let addr = spawn_server(td.cfg.clone());

    // 1. Offline local workspace read
    let (st, res, raw) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "local-workspace-files",
            "action": "list_workspace",
            "args": { "path": "." }
        })
        .to_string(),
    );
    assert_eq!(st, 200, "{raw}");
    assert_eq!(res["mode"], json!("local"));
    assert!(res["output"].as_str().unwrap().contains("notes.txt"), "{res}");

    // 2. Offline Excel / Spreadsheet build on-device
    let (st_x, res_x, raw_x) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "office-excel",
            "action": "build_sheet",
            "args": {
                "title": "Q4 Budget Sheet",
                "columns": ["Item", "Cost"],
                "rows": [["Laptop", "1200.00"], ["Desk", "450.00"]]
            }
        })
        .to_string(),
    );
    assert_eq!(st_x, 200, "{raw_x}");
    assert_eq!(res_x["mode"], json!("local"));
    assert!(td.cfg.workspace_dir.join("Q4 Budget Sheet.csv").exists());

    // 3. Consequential cloud write must park a decision card on /phone & Today
    let (st_w, res_w, raw_w) = http_req(
        addr,
        "POST",
        "/api/plugins/action",
        &json!({
            "plugin_id": "outlook-mail",
            "action": "send_mail",
            "args": {
                "to": "client@acme.com",
                "subject": "Q4 Summary",
                "body": "Attached report."
            }
        })
        .to_string(),
    );
    assert_eq!(st_w, 200, "{raw_w}");
    assert!(
        res_w["output"]
            .as_str()
            .unwrap()
            .contains("parked for your person's approval"),
        "{res_w}"
    );

    let (st_d, decs, _) = http_req(addr, "GET", "/api/decisions", "");
    assert_eq!(st_d, 200);
    let pending = decs["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["kind"].as_str() == Some("plugin_action") && d["status"].as_str() == Some("pending"))
        .expect("pending plugin_action decision");
    let did = pending["id"].as_str().unwrap();

    let (st_deny, denied, raw_deny) =
        http_req(addr, "POST", &format!("/api/decisions/{did}/deny"), "{}");
    assert_eq!(st_deny, 200, "{raw_deny}");
    assert_eq!(denied["decision"]["status"], json!("denied"));
    assert!(
        denied["decision"]["outcome"]
            .as_str()
            .unwrap()
            .contains("cancelled"),
        "{denied}"
    );
}

#[test]
fn plugins_detect_outlook_connector_and_isolate_people() {
    let td = PluginTestDir::new("outlook-isolate");
    let _policy = ergon_rs::tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));
    let addr = spawn_server(td.cfg.clone());

    // Configure an Outlook M365 SMTP connector
    let (st_c, _, raw_c) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Company Outlook Relay",
            "kind": "smtp_relay",
            "enabled": true,
            "smtp_config": {
                "host": "smtp.office365.com",
                "port": 587,
                "security": "starttls",
                "from_addr": "ops@contoso.com",
                "default_to": ["team@contoso.com"],
                "username_secret_key": "smtp_user",
                "password_secret_key": "oauth_access_token"
            }
        })
        .to_string(),
    );
    assert_eq!(st_c, 201, "{raw_c}");

    // Default person now sees outlook-mail automatically connected in cloud mode
    let (st_p, body_p, _) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st_p, 200);
    let outlook = body_p["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_str() == Some("outlook-mail"))
        .unwrap();
    assert_eq!(outlook["cloud_connected"], json!(true));
    assert_eq!(outlook["status"], json!("connected_cloud"));
    assert_eq!(outlook["connected_via"], json!("Company Outlook Relay"));

    // Create and switch to Person B: isolated plugins state
    let (st_new, person_b, raw_b) = http_req(
        addr,
        "POST",
        "/api/people",
        &json!({ "name": "Priya", "role": "Operations" }).to_string(),
    );
    assert_eq!(st_new, 201, "{raw_b}");
    let pid_b = person_b["id"].as_str().unwrap();
    let (st_sw, _, _) = http_req(
        addr,
        "POST",
        "/api/active-person",
        &json!({ "id": pid_b }).to_string(),
    );
    assert_eq!(st_sw, 200);

    let (st_pb, body_pb, _) = http_req(addr, "GET", "/api/plugins", "");
    assert_eq!(st_pb, 200);
    let outlook_b = body_pb["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_str() == Some("outlook-mail"))
        .unwrap();
    assert_eq!(outlook_b["cloud_connected"], json!(false));
    assert_eq!(outlook_b["status"], json!("available"));
}
