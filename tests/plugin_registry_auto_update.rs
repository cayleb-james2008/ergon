//! Integration tests for Ergon's auto-updating plugin catalog & Official MCP Registry integration:
//! - Configurable `ergon_manifest` + `mcp_registry_v01` sources with `0600` cache + SHA-256 digest
//! - Strict filtering of deprecated servers and unsafe package/command inputs
//! - Live `?q=` search across merged catalog and MCP Registry feed
//! - 1-click install (encrypting secrets in `secrets.enc.json` `0600`, `require_approval_by_default: true`) & uninstall
//! - Air-Gapped policy (`403` on sync, outbound fetch skipped on search)

use ergon_rs::{plugin_registry, plugins, save_workspace_policy, secret_store, serve, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

static SEQ: AtomicUsize = AtomicUsize::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(label: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-plugreg-{}-{}-{label}",
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
    std::thread::sleep(Duration::from_millis(40));
    addr
}

fn spawn_mock_feeds(live_search_hits: Arc<AtomicUsize>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut s = stream;
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap_or(0);
            let req_line = String::from_utf8_lossy(&buf[..n])
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let path = req_line.split_whitespace().nth(1).unwrap_or("/");
            let body = if path.starts_with("/manifest.json") {
                json!({
                    "schema_version": 1,
                    "plugins": [{
                        "id": "acme-erp",
                        "suite": "business",
                        "suite_label": "Business Apps",
                        "name": "Acme ERP",
                        "tagline": "Curated ERP feed app",
                        "description": "Synced from curated Ergon feed.",
                        "mode": "cloud",
                        "transport": "mcp",
                        "local_ready": false,
                        "default_enabled": false,
                        "actions": [{ "name": "run_tool", "label": "Query ERP", "description": "Read orders", "access": "read", "requires_approval": false }],
                        "install_spec": {
                            "transport": "http_sse",
                            "command_or_url": "https://erp.example.com/mcp",
                            "args": [],
                            "credential_fields": [{ "key": "ACME_ERP_KEY", "label": "Acme ERP Key", "description": "API token", "is_secret": true, "is_required": true, "kind": "bearer" }]
                        }
                    }]
                })
                .to_string()
            } else if path.starts_with("/v0.1/servers") {
                if path.contains("search=") {
                    live_search_hits.fetch_add(1, Ordering::SeqCst);
                }
                json!({
                    "servers": [
                        {
                            "server": {
                                "name": "com.example/remote-notes",
                                "title": "Remote Notes Cloud",
                                "description": "Safe remote streamable-http server.",
                                "version": "1.0.0",
                                "websiteUrl": "https://notes.example.com",
                                "remotes": [{ "type": "streamable-http", "url": "https://mcp.example.com/notes" }]
                            },
                            "_meta": { "io.modelcontextprotocol.registry/official": { "status": "active", "isLatest": true } }
                        },
                        {
                            "server": {
                                "name": "com.acme/acme-widget",
                                "title": "Acme Widget Tool",
                                "description": "Safe npm npx package with API token.",
                                "version": "2.1.0",
                                "packages": [{
                                    "registryType": "npm",
                                    "identifier": "@acme/mcp-widget",
                                    "transport": { "type": "stdio" },
                                    "environmentVariables": [{ "name": "ACME_TOKEN", "description": "Acme secret token", "isSecret": true, "isRequired": true }]
                                }]
                            },
                            "_meta": { "io.modelcontextprotocol.registry/official": { "status": "active", "isLatest": true } }
                        },
                        {
                            "server": {
                                "name": "com.old/deprecated-srv",
                                "title": "Deprecated Server",
                                "description": "Must be filtered out.",
                                "version": "0.1.0",
                                "remotes": [{ "type": "streamable-http", "url": "https://old.example.com/mcp" }]
                            },
                            "_meta": { "io.modelcontextprotocol.registry/official": { "status": "deprecated", "isLatest": true } }
                        },
                        {
                            "server": {
                                "name": "com.evil/unsafe-pkg",
                                "title": "Unsafe Shell Metachar Package",
                                "description": "Must be rejected by sanitizer.",
                                "version": "1.0.0",
                                "packages": [{
                                    "registryType": "npm",
                                    "identifier": "bad;rm -rf /",
                                    "transport": { "type": "stdio" }
                                }]
                            },
                            "_meta": { "io.modelcontextprotocol.registry/official": { "status": "active", "isLatest": true } }
                        }
                    ]
                })
                .to_string()
            } else {
                "{}".to_string()
            };
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });
    std::thread::sleep(Duration::from_millis(30));
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
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

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(ref v) = self.prev {
            std::env::set_var(self.key, v);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn plugin_registry_sync_search_install_and_air_gapped_enforcement() {
    let _local_guard = EnvVarGuard::set("ERGON_ALLOW_LOCAL_CONNECTORS", "1");
    let td = TestDir::new("sync-install");
    let _vault_guard = secret_store::ScopedVaultOverride::new(
        &td.cfg.data_dir,
        secret_store::VaultOverride {
            key: None,
            backend: Some(secret_store::BackendPreference::AeadFile),
            disable_os_keychain_probe: true,
        },
    );
    let live_search_hits = Arc::new(AtomicUsize::new(0));
    let feed_addr = spawn_mock_feeds(live_search_hits.clone());
    let server_addr = spawn_server(td.cfg.clone());

    let feed_base = format!("http://127.0.0.1:{}", feed_addr.port());
    let (st_cfg, cfg_val, raw_cfg) = http_req(
        server_addr,
        "PUT",
        "/api/plugins/registry/config",
        &json!({
            "auto_update": "manual",
            "sync_interval_secs": 3600,
            "sources": [
                { "id": "curated-test", "name": "Curated Test", "kind": "ergon_manifest", "url": format!("{feed_base}/manifest.json"), "enabled": true },
                { "id": "mcp-test", "name": "MCP Registry Test", "kind": "mcp_registry_v01", "url": format!("{feed_base}/v0.1/servers?version=latest&limit=20"), "enabled": true }
            ]
        })
        .to_string(),
    );
    assert_eq!(st_cfg, 200, "{raw_cfg}");
    assert_eq!(cfg_val["auto_update"], json!("manual"));

    // Sync now -> 0600 cache + SHA-256 + curated & safe MCP servers merged, deprecated/unsafe filtered
    let (st_sync, sync_val, raw_sync) = http_req(server_addr, "POST", "/api/plugins/registry/sync", "{}");
    assert_eq!(st_sync, 200, "{raw_sync}");
    assert!(sync_val["catalog_sha256"].as_str().unwrap().len() >= 32, "{sync_val}");

    let cache_path = td.cfg.data_dir.join("plugin_registry_cache.json");
    assert!(cache_path.exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&cache_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "cache file must be 0600");
    }

    let (st_list, list_val, raw_list) = http_req(server_addr, "GET", "/api/plugins", "");
    assert_eq!(st_list, 200, "{raw_list}");
    let listed = list_val["plugins"].as_array().unwrap();
    assert!(listed.iter().any(|p| p["id"] == "acme-erp"), "missing curated feed app");
    assert!(listed.iter().any(|p| p["name"] == "Remote Notes Cloud"), "missing remote http MCP app");
    assert!(listed.iter().any(|p| p["name"] == "Acme Widget Tool"), "missing npm MCP app");
    assert!(!listed.iter().any(|p| p["name"] == "Deprecated Server"), "deprecated server must be excluded");
    assert!(!listed.iter().any(|p| p["name"] == "Unsafe Shell Metachar Package"), "unsafe pkg must be excluded");

    // Search endpoint hits merged catalog + live feed
    let (st_search, search_val, raw_search) =
        http_req(server_addr, "GET", "/api/plugins/registry/search?q=acme", "");
    assert_eq!(st_search, 200, "{raw_search}");
    let results = search_val["results"].as_array().unwrap();
    assert!(results.iter().any(|p| p["id"] == "acme-erp"), "{search_val}");
    assert!(results.iter().any(|p| p["name"] == "Acme Widget Tool"), "{search_val}");
    assert!(live_search_hits.load(Ordering::SeqCst) >= 1);

    // 1-click install with encrypted secret storage & default approval gate
    let widget = results
        .iter()
        .find(|p| p["name"] == "Acme Widget Tool")
        .unwrap();
    let widget_id = widget["id"].as_str().unwrap();
    let sentinel = "sk-ergon-test-secret-0123456789";
    let (st_inst, inst_val, raw_inst) = http_req(
        server_addr,
        "POST",
        "/api/plugins/registry/install",
        &json!({
            "plugin_id": widget_id,
            "credentials": { "ACME_TOKEN": sentinel }
        })
        .to_string(),
    );
    assert_eq!(st_inst, 200, "{raw_inst}");
    assert_eq!(inst_val["plugin"]["installed"], json!(true));
    assert_eq!(inst_val["plugin"]["enabled"], json!(true));

    // Secret must NOT exist in plaintext inside plugins.json or mcp_servers.json
    let plugins_raw = std::fs::read_to_string(plugins::plugins_path(&td.cfg.data_dir)).unwrap();
    assert!(!plugins_raw.contains(sentinel), "plaintext secret leaked into plugins.json");
    let mcp_raw = std::fs::read_to_string(td.cfg.data_dir.join("mcp_servers.json")).unwrap();
    assert!(!mcp_raw.contains(sentinel), "plaintext secret leaked into mcp_servers.json");
    let mcp_val: Value = serde_json::from_str(&mcp_raw).unwrap();
    let srv = mcp_val["servers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"].as_str() == inst_val["mcp_server_id"].as_str())
        .expect("installed MCP server entry");
    assert_eq!(srv["require_approval_by_default"], json!(true));

    // Uninstall removes plugin + linked MCP server
    let (st_un, un_val, raw_un) = http_req(
        server_addr,
        "POST",
        "/api/plugins/registry/uninstall",
        &json!({ "plugin_id": widget_id }).to_string(),
    );
    assert_eq!(st_un, 200, "{raw_un}");
    assert_eq!(un_val["ok"], json!(true));

    // Air-Gapped policy blocks sync (403) and skips live search fetch
    let prev_hits = live_search_hits.load(Ordering::SeqCst);
    save_workspace_policy(&td.cfg.data_dir, "air_gapped").unwrap();
    let (st_air, _, raw_air) = http_req(server_addr, "POST", "/api/plugins/registry/sync", "{}");
    assert_eq!(st_air, 403, "{raw_air}");
    let (st_air_s, _, _) = http_req(server_addr, "GET", "/api/plugins/registry/search?q=acme", "");
    assert_eq!(st_air_s, 200);
    assert_eq!(
        live_search_hits.load(Ordering::SeqCst),
        prev_hits,
        "air-gapped policy must skip live registry search"
    );

    // Silence unused import warning cleanly if plugin_registry helper isn't called directly
    let _ = plugin_registry::embedded_baseline().len();
}
