//! Integration tests for Ergon Gap 7 (P1):
//! - AES-256-GCM authenticated encrypted-at-rest secret vault (`secrets.enc.json`)
//!   with unique random 96-bit nonces, key-bound AAD, and `0600` key file / env key / OS keychain fallback
//! - Zero plaintext secret substrings anywhere in `ERGON_DATA_DIR` (recursive byte scan)
//! - Clean authentication failure on ciphertext, nonce, AAD key-swap, wrong key, or JSON tampering
//! - Automatic transparent migration of legacy plaintext `connector_secrets.json`, `connectors.json`,
//!   `oauth.json`, `mcp_servers.json`, and `config.json`

use ergon_rs::{
    connectors, execute_tool, load_saved_model_config, mcp, secret_store, serve, tripwire, Config,
};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static SEQ: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    cfg: Config,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-vault-test-{}-{}-{name}",
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

fn spawn_server(cfg: Config) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg);
    });
    std::thread::sleep(Duration::from_millis(40));
    addr
}

fn http_req_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n", addr.port());
    let mut has_ct = false;
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("content-type") {
            has_ct = true;
        }
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if !has_ct && !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body).unwrap();
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

fn http_req(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value, String) {
    http_req_headers(addr, method, path, &[], body.as_bytes())
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

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_all_files(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

fn assert_zero_plaintext_in_dir(dir: &Path, forbidden_secrets: &[&str]) {
    let mut files = Vec::new();
    collect_all_files(dir, &mut files);
    assert!(
        !files.is_empty(),
        "expected persisted files under {}",
        dir.display()
    );
    for file in files {
        let bytes = std::fs::read(&file).unwrap();
        for secret in forbidden_secrets {
            assert!(
                !secret.is_empty(),
                "test secret string must be non-empty"
            );
            let needle = secret.as_bytes();
            let found = bytes.windows(needle.len()).any(|w| w == needle);
            assert!(
                !found,
                "LEAK DETECTED: plaintext secret '{secret}' appeared on disk in {}",
                file.display()
            );
        }
    }
}

#[test]
fn all_connector_mcp_and_model_secrets_encrypted_at_rest_zero_plaintext_on_disk() {
    let td = TestDir::new("zero-plaintext");
    let _policy = tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));
    let addr = spawn_server(td.cfg.clone());

    let smtp_user = "smtp-vault-user-alpha";
    let smtp_pass = "smtp-vault-pass-99887766";
    let imap_user = "imap-vault-user-beta";
    let imap_pass = "imap-vault-pass-55443322";
    let oa2_csec = "oa2-vault-client-secret-771199";
    let oa2_rt_init = "oa2-vault-refresh-init-332211";
    let oa2_at_fresh = "oa2-vault-access-fresh-884422";
    let oa2_rt_rot = "oa2-vault-refresh-rotated-667788";
    let partner_tok = "partner-vault-api-token-abc987";
    let mcp_env_tok = "mcp-vault-stdio-secret-135791";
    let mcp_http_tok = "mcp-vault-http-bearer-246802";
    let model_api_key = "sk-vault-model-secret-424242";

    // Mock OAuth2 token endpoint
    let oauth_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let oauth_addr = oauth_listener.local_addr().unwrap();
    let oa2_at_srv = oa2_at_fresh.to_string();
    let oa2_rt_srv = oa2_rt_rot.to_string();
    std::thread::spawn(move || {
        for mut stream in oauth_listener.incoming().flatten() {
            let (_h, body_req) = read_full_http_request(&mut stream);
            assert!(body_req.contains("grant_type=refresh_token"), "{body_req}");
            let body = json!({
                "access_token": oa2_at_srv,
                "expires_in": 3600,
                "refresh_token": oa2_rt_srv
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    // Mock partner REST endpoint
    let rest_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let rest_addr = rest_listener.local_addr().unwrap();
    let rest_auths = Arc::new(Mutex::new(Vec::<String>::new()));
    let rest_auths_bg = rest_auths.clone();
    std::thread::spawn(move || {
        for mut stream in rest_listener.incoming().flatten() {
            let (head, _body) = read_full_http_request(&mut stream);
            let auth = head
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("authorization:"))
                .unwrap_or("")
                .to_string();
            rest_auths_bg.lock().unwrap().push(auth);
            let body = r#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    // Mock HTTP/SSE MCP server verifying Authorization: Bearer <mcp_http_tok>
    let mcp_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mcp_addr = mcp_listener.local_addr().unwrap();
    let mcp_auths = Arc::new(Mutex::new(Vec::<String>::new()));
    let mcp_auths_bg = mcp_auths.clone();
    std::thread::spawn(move || {
        for mut stream in mcp_listener.incoming().flatten() {
            let (head, body) = read_full_http_request(&mut stream);
            let auth = head
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("authorization:"))
                .unwrap_or("")
                .to_string();
            mcp_auths_bg.lock().unwrap().push(auth);
            let rpc: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let id = rpc["id"].clone();
            let method = rpc["method"].as_str().unwrap_or("");
            let resp_body = if method == "initialize" {
                json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}}}).to_string()
            } else if method == "tools/list" {
                json!({"jsonrpc":"2.0","id":id,"result":{"tools":[{"name":"ping","description":"Ping","inputSchema":{"type":"object","properties":{}}}]}}).to_string()
            } else if method == "tools/call" {
                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"pong-authed"}],"isError":false}}).to_string()
            } else {
                "{}".to_string()
            };
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                resp_body.len()
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    // 1. Save SMTP connector secrets
    let (st, _, raw) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Vault SMTP",
            "kind": "smtp_relay",
            "smtp_config": {
                "host": "127.0.0.1",
                "port": 2525,
                "security": "plain_loopback_test",
                "username_secret_key": "smtp_user",
                "password_secret_key": "smtp_pass",
                "from_addr": "ops@ergon.local",
                "default_to": ["client@example.com"]
            },
            "secrets": {
                "smtp_user": smtp_user,
                "smtp_pass": smtp_pass
            }
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    assert!(!raw.contains(smtp_pass));

    // 2. Save IMAP trigger secrets
    let (st, _, raw) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({
            "name": "Vault IMAP",
            "kind": "imap_poll",
            "imap_config": {
                "host": "127.0.0.1",
                "port": 1143,
                "security": "plain_loopback_test",
                "folder": "INBOX",
                "username_secret_key": "imap_user",
                "password_secret_key": "imap_pass"
            },
            "secrets": {
                "imap_user": imap_user,
                "imap_pass": imap_pass
            }
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    assert!(!raw.contains(imap_pass));

    // 3. Save OAuth2 + Partner REST connector & trigger live OAuth2 refresh
    let (st, oa_json, raw) = http_req(
        addr,
        "POST",
        "/api/connectors/outbound",
        &json!({
            "name": "Vault OAuth2 API",
            "kind": "rest_api",
            "url": format!("http://127.0.0.1:{}/events", rest_addr.port()),
            "method": "POST",
            "headers_template": {
                "Authorization": "Bearer {{secret:oauth_access_token}}",
                "X-Partner": "{{secret:partner_token}}"
            },
            "oauth_config": {
                "token_url": format!("http://127.0.0.1:{}/token", oauth_addr.port()),
                "client_id": "ergon-vault-app",
                "client_secret_key": "oa2_csec",
                "refresh_token_secret_key": "oa2_rt"
            },
            "require_approval": false,
            "secrets": {
                "oa2_csec": oa2_csec,
                "oa2_rt": oa2_rt_init,
                "partner_token": partner_tok
            }
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    let oa_conn_id = oa_json["connector"]["id"].as_str().unwrap().to_string();

    let delivered = execute_tool(
        &td.cfg,
        "invoke_connector",
        &json!({ "connector_id": oa_conn_id, "payload": { "hello": "vault" } }),
    )
    .unwrap();
    assert!(delivered.contains("delivered via connector"), "{delivered}");
    assert!(rest_auths
        .lock()
        .unwrap()
        .iter()
        .any(|h| h.contains(&format!("Bearer {oa2_at_fresh}"))));

    // 4. Create inbound signed webhook trigger
    let (st, wh_json, raw) = http_req(
        addr,
        "POST",
        "/api/connectors/triggers",
        &json!({ "name": "Vault Webhook", "kind": "webhook" }).to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    let wh_sig = wh_json["signing_secret"].as_str().unwrap().to_string();
    let wh_tok = wh_json["hook_token"].as_str().unwrap().to_string();
    let wh_path = wh_json["hook_path"].as_str().unwrap().to_string();

    // Verify webhook HMAC still validates cleanly using decrypted vault secret
    let hook_payload = br#"{"event":"vault.test"}"#;
    let mac = tripwire::hmac_sha256(wh_sig.as_bytes(), hook_payload);
    let sig_hdr = format!("sha256={}", tripwire::hex(&mac));
    let (st_wh, _, raw_wh) = http_req_headers(
        addr,
        "POST",
        &wh_path,
        &[("X-Ergon-Signature", &sig_hdr)],
        hook_payload,
    );
    assert_eq!(st_wh, 202, "{raw_wh}");

    // 5. Save MCP stdio env secret + HTTP MCP bearer secret via POST /api/mcp/servers
    let (st, mcp_stdio_json, raw) = http_req(
        addr,
        "POST",
        "/api/mcp/servers",
        &json!({
            "id": "vaultstdio",
            "name": "Vault Stdio MCP",
            "transport": "stdio",
            "command_or_url": "python3",
            "args": ["-c", "pass"],
            "env_secret_keys": { "MCP_TOKEN": "mcp_env_tok" },
            "secrets": { "mcp_env_tok": mcp_env_tok },
            "require_approval_by_default": false
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    assert!(!raw.contains(mcp_env_tok));
    assert_eq!(mcp_stdio_json["server"]["has_secret"], json!(true));

    let (st, mcp_http_json, raw) = http_req(
        addr,
        "POST",
        "/api/mcp/servers",
        &json!({
            "id": "vaulthttp",
            "name": "Vault HTTP MCP",
            "transport": "http_sse",
            "command_or_url": format!("http://127.0.0.1:{}/mcp", mcp_addr.port()),
            "auth_secret_key": "mcp_http_bearer",
            "secrets": { "mcp_http_bearer": mcp_http_tok },
            "require_approval_by_default": false
        })
        .to_string(),
    );
    assert_eq!(st, 201, "{raw}");
    assert!(!raw.contains(mcp_http_tok));
    assert_eq!(mcp_http_json["server"]["has_secret"], json!(true));

    // Discover & invoke HTTP MCP server — verify decrypted bearer token reached mock server
    let (st_d, _, raw_d) = http_req(addr, "POST", "/api/mcp/servers/vaulthttp/discover", "");
    assert_eq!(st_d, 200, "{raw_d}");
    let mcp_call = execute_tool(&td.cfg, "mcp.vaulthttp.ping", &json!({})).unwrap();
    assert!(mcp_call.contains("pong-authed"), "{mcp_call}");
    assert!(mcp_auths
        .lock()
        .unwrap()
        .iter()
        .any(|h| h.contains(&format!("Bearer {mcp_http_tok}"))));

    // 6. Save model API key via POST /api/config
    let (st, _, raw) = http_req(
        addr,
        "POST",
        "/api/config",
        &json!({
            "model_base_url": "https://api.example.com/v1",
            "model_id": "test-model",
            "api_key": model_api_key
        })
        .to_string(),
    );
    assert_eq!(st, 200, "{raw}");
    assert!(!raw.contains(model_api_key));
    let loaded_model = load_saved_model_config(&td.cfg.data_dir).unwrap();
    assert_eq!(loaded_model.api_key.as_deref(), Some(model_api_key));

    // 7. Recursive byte scan across EVERY file in ERGON_DATA_DIR: ZERO plaintext secrets!
    let all_secrets = [
        smtp_user,
        smtp_pass,
        imap_user,
        imap_pass,
        oa2_csec,
        oa2_rt_init,
        oa2_at_fresh,
        oa2_rt_rot,
        partner_tok,
        mcp_env_tok,
        mcp_http_tok,
        model_api_key,
        wh_sig.as_str(),
        wh_tok.as_str(),
    ];
    assert_zero_plaintext_in_dir(&td.cfg.data_dir, &all_secrets);

    // 8. GET API responses NEVER leak any raw secret string
    let (st_c, list_c, raw_c) = http_req(addr, "GET", "/api/connectors", "");
    assert_eq!(st_c, 200);
    assert_eq!(list_c["secret_storage"]["cipher"], json!("aes-256-gcm"));
    let (st_m, _, raw_m) = http_req(addr, "GET", "/api/mcp/servers", "");
    assert_eq!(st_m, 200);
    let (st_g, cfg_g, raw_g) = http_req(addr, "GET", "/api/config", "");
    assert_eq!(st_g, 200);
    assert_eq!(cfg_g["has_api_key"], json!(true));
    for sec in all_secrets {
        assert!(!raw_c.contains(sec), "GET /api/connectors leaked {sec}");
        assert!(!raw_m.contains(sec), "GET /api/mcp/servers leaked {sec}");
        assert!(!raw_g.contains(sec), "GET /api/config leaked {sec}");
    }

    // 9. Verify 0600 POSIX permissions on vault, key file, and secret metadata files
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for p in [
            secret_store::vault_path(&td.cfg.data_dir),
            secret_store::vault_key_path(&td.cfg.data_dir),
            connectors::connector_secrets_path(&td.cfg.data_dir),
            connectors::connectors_path(&td.cfg.data_dir),
            mcp::mcp_servers_path(&td.cfg.data_dir),
            ergon_rs::config_file_path(&td.cfg.data_dir),
        ] {
            let mode = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "expected 0600 on {}", p.display());
        }
    }

    // 10. Verify unique random 96-bit nonces across repeated encryptions of identical plaintext
    secret_store::set_secret(&td.cfg.data_dir, "nonce:test", "same-secret-value").unwrap();
    let v1: Value =
        serde_json::from_str(&std::fs::read_to_string(secret_store::vault_path(&td.cfg.data_dir)).unwrap())
            .unwrap();
    let n1 = v1["entries"]["nonce:test"]["nonce"].as_str().unwrap().to_string();
    let c1 = v1["entries"]["nonce:test"]["ciphertext"].as_str().unwrap().to_string();

    secret_store::set_secret(&td.cfg.data_dir, "nonce:test", "same-secret-value").unwrap();
    let v2: Value =
        serde_json::from_str(&std::fs::read_to_string(secret_store::vault_path(&td.cfg.data_dir)).unwrap())
            .unwrap();
    let n2 = v2["entries"]["nonce:test"]["nonce"].as_str().unwrap().to_string();
    let c2 = v2["entries"]["nonce:test"]["ciphertext"].as_str().unwrap().to_string();
    assert_ne!(n1, n2, "every encryption must generate a fresh random 96-bit nonce");
    assert_ne!(c1, c2, "ciphertexts must differ under distinct nonces");

    // 11. Verify tripwire sensitive_target covers new vault files
    assert!(tripwire::sensitive_target("/tmp/any/secrets.enc.json").is_some());
    assert!(tripwire::sensitive_target("/tmp/any/.vault.key").is_some());
    assert!(tripwire::sensitive_target("/tmp/any/oauth.json").is_some());
}

#[test]
fn tampering_ciphertext_nonce_aad_or_json_fails_authentication_cleanly() {
    let td = TestDir::new("tamper-aead");
    let _policy = tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));

    let mut initial = std::collections::BTreeMap::new();
    initial.insert("smtp_pass".to_string(), "smtp-secret-alpha-111".to_string());
    initial.insert("imap_pass".to_string(), "imap-secret-beta-222".to_string());
    connectors::store_connector_secrets(&td.cfg.data_dir, &initial).unwrap();
    let policy = ergon_rs::load_workspace_policy(&td.cfg.data_dir);
    let (conn_val, _) = connectors::upsert_outbound_connector(
        &td.cfg.data_dir,
        &json!({
            "name": "Tamper Check API",
            "kind": "rest_api",
            "url": "http://127.0.0.1:9/hook",
            "headers_template": { "Authorization": "Bearer {{secret:smtp_pass}}" },
            "require_approval": false
        }),
        &policy,
    )
    .unwrap();
    let conn_id = conn_val["id"].as_str().unwrap().to_string();
    let wh_val = connectors::create_trigger(
        &td.cfg.data_dir,
        &td.cfg.data_dir,
        &json!({ "name": "Tamper Check Hook", "kind": "webhook" }),
    )
    .unwrap();
    let wh_hook_id = wh_val["trigger"]["hook_id"].as_str().unwrap().to_string();
    let wh_sig = wh_val["signing_secret"].as_str().unwrap().to_string();
    assert_eq!(secret_store::verify_vault_integrity(&td.cfg.data_dir).unwrap(), 4);

    let vpath = secret_store::vault_path(&td.cfg.data_dir);
    let clean_bytes = std::fs::read(&vpath).unwrap();

    // 1. Tamper with ciphertext -> authentication failure across vault, connector delivery, webhook auth, and writes
    {
        let mut doc: Value = serde_json::from_slice(&clean_bytes).unwrap();
        let orig = doc["entries"]["connector:secret:smtp_pass"]["ciphertext"]
            .as_str()
            .unwrap()
            .to_string();
        let mut chars: Vec<char> = orig.chars().collect();
        chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
        let flipped: String = chars.into_iter().collect();
        doc["entries"]["connector:secret:smtp_pass"]["ciphertext"] = json!(flipped);
        std::fs::write(&vpath, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();

        let err = secret_store::get_secret(&td.cfg.data_dir, "connector:secret:smtp_pass")
            .unwrap_err();
        assert!(err.contains("authentication failed"), "{err}");

        let store_err = connectors::load_connector_secrets(&td.cfg.data_dir).unwrap_err();
        assert!(store_err.contains("authentication failed"), "{store_err}");

        let invoke_err = execute_tool(
            &td.cfg,
            "invoke_connector",
            &json!({ "connector_id": conn_id, "payload": { "x": 1 } }),
        )
        .unwrap_err();
        assert!(invoke_err.contains("authentication failed"), "{invoke_err}");

        let hook_body = br#"{"tamper":true}"#;
        let mac = tripwire::hmac_sha256(wh_sig.as_bytes(), hook_body);
        let sig_hdr = format!("sha256={}", tripwire::hex(&mac));
        let (wh_status, wh_err) = connectors::verify_and_ingest_webhook_with_meta(
            &td.cfg,
            &wh_hook_id,
            Some(&sig_hdr),
            None,
            None,
            None,
            "application/json",
            hook_body,
        )
        .unwrap_err();
        assert_eq!(wh_status, 500, "{wh_err}");
        assert!(wh_err.contains("authentication failed"), "{wh_err}");

        // Must refuse to silently overwrite a tampered vault
        let write_err = secret_store::set_secret(&td.cfg.data_dir, "other:key", "val").unwrap_err();
        assert!(write_err.contains("authentication failed"), "{write_err}");
    }

    // 2. Tamper with nonce -> authentication failure
    {
        let mut doc: Value = serde_json::from_slice(&clean_bytes).unwrap();
        let orig = doc["entries"]["connector:secret:smtp_pass"]["nonce"]
            .as_str()
            .unwrap()
            .to_string();
        let mut chars: Vec<char> = orig.chars().collect();
        chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
        let flipped: String = chars.into_iter().collect();
        doc["entries"]["connector:secret:smtp_pass"]["nonce"] = json!(flipped);
        std::fs::write(&vpath, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();

        let err = secret_store::get_secret(&td.cfg.data_dir, "connector:secret:smtp_pass")
            .unwrap_err();
        assert!(err.contains("authentication failed"), "{err}");
    }

    // 3. AAD key-substitution attack: splice smtp_pass ciphertext into imap_pass slot -> FAILS!
    {
        let mut doc: Value = serde_json::from_slice(&clean_bytes).unwrap();
        let smtp_entry = doc["entries"]["connector:secret:smtp_pass"].clone();
        doc["entries"]["connector:secret:imap_pass"] = smtp_entry;
        std::fs::write(&vpath, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();

        let err = secret_store::get_secret(&td.cfg.data_dir, "connector:secret:imap_pass")
            .unwrap_err();
        assert!(
            err.contains("authentication failed"),
            "AAD must bind secret key name to defeat key-swap attacks: {err}"
        );
    }

    // 4. Corrupt JSON in secrets.enc.json -> fails cleanly & refuses to overwrite
    {
        std::fs::write(&vpath, b"{corrupt json").unwrap();
        let err = connectors::load_connector_secrets(&td.cfg.data_dir).unwrap_err();
        assert!(err.contains("corrupt encrypted secret vault"), "{err}");
        let werr = secret_store::set_secret(&td.cfg.data_dir, "k", "v").unwrap_err();
        assert!(werr.contains("corrupt encrypted secret vault"), "{werr}");
    }
}

#[test]
fn legacy_plaintext_migration_scrubs_all_files_and_preserves_functionality() {
    let td = TestDir::new("legacy-migrate");
    let _policy = tripwire::ScopedLocalPolicy::new(&td.cfg.data_dir, Some(true), Some(true));

    let leg_v1_tok = "legacy-v1-partner-secret-111";
    let leg_v1_sig = "whsec_legacy_v1_signing_222";
    let leg_v1_hook = "whtok_legacy_v1_token_333";
    let leg_smtp_pass = "legacy-inline-smtp-pass-444";
    let leg_imap_pass = "legacy-inline-imap-pass-555";
    let leg_oa2_csec = "legacy-inline-oa2-csec-666";
    let leg_oa2_rt = "legacy-inline-oa2-rt-777";
    let leg_oauth_file_at = "legacy-oauth-json-access-888";
    let leg_oauth_file_cs = "legacy-oauth-json-csec-999";
    let leg_mcp_bearer = "legacy-mcp-bearer-token-aaa";
    let leg_mcp_env = "legacy-mcp-env-secret-bbb";
    let leg_model_key = "sk-legacy-config-model-key-ccc";

    // Seed legacy v1 plaintext connector_secrets.json
    std::fs::write(
        connectors::connector_secrets_path(&td.cfg.data_dir),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "secrets": {
                "legacy_partner": leg_v1_tok
            },
            "secret_updated_ms": {
                "legacy_partner": 1700000000000u64
            },
            "webhook_secrets": {
                "trg-legacy-wh": {
                    "signing_secret": leg_v1_sig,
                    "hook_token": leg_v1_hook,
                    "updated_ms": 1700000000000u64
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    // Seed legacy connectors.json with inline plaintext credentials
    std::fs::write(
        connectors::connectors_path(&td.cfg.data_dir),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "triggers": [
                {
                    "id": "trg-legacy-wh",
                    "name": "Legacy Webhook",
                    "kind": "webhook",
                    "enabled": true,
                    "target_kind": "routine",
                    "hook_id": "hk_legacy_1234"
                },
                {
                    "id": "trg-legacy-imap",
                    "name": "Legacy IMAP",
                    "kind": "imap_poll",
                    "enabled": true,
                    "target_kind": "routine",
                    "imap_config": {
                        "host": "imap.example.com",
                        "port": 993,
                        "security": "imaps",
                        "folder": "INBOX",
                        "password": leg_imap_pass
                    }
                }
            ],
            "outbound": [
                {
                    "id": "conn-legacy-smtp",
                    "name": "Legacy SMTP",
                    "kind": "smtp_relay",
                    "url": "smtps://smtp.example.com:465",
                    "method": "POST",
                    "require_approval": true,
                    "enabled": true,
                    "smtp_config": {
                        "host": "smtp.example.com",
                        "port": 465,
                        "security": "smtps",
                        "from_addr": "ops@example.com",
                        "default_to": ["dest@example.com"],
                        "password": leg_smtp_pass
                    },
                    "oauth_config": {
                        "token_url": "https://oauth.example.com/token",
                        "client_id": "legacy-client",
                        "client_secret": leg_oa2_csec,
                        "refresh_token": leg_oa2_rt
                    }
                }
            ],
            "delivery_log": [],
            "intake_manifest": []
        }))
        .unwrap(),
    )
    .unwrap();

    // Seed legacy plaintext oauth.json
    std::fs::write(
        connectors::legacy_oauth_path(&td.cfg.data_dir),
        serde_json::to_vec_pretty(&json!({
            "client_id": "legacy-oauth-app",
            "client_secret": leg_oauth_file_cs,
            "access_token": leg_oauth_file_at
        }))
        .unwrap(),
    )
    .unwrap();

    // Seed legacy plaintext mcp_servers.json
    std::fs::write(
        mcp::mcp_servers_path(&td.cfg.data_dir),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "inbound_enabled": false,
            "servers": [
                {
                    "id": "legmcp",
                    "name": "Legacy MCP",
                    "transport": "http_sse",
                    "command_or_url": "https://mcp.example.com/rpc",
                    "bearer_token": leg_mcp_bearer,
                    "env": { "API_KEY": leg_mcp_env },
                    "enabled": true
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    // Seed legacy plaintext config.json
    std::fs::write(
        ergon_rs::config_file_path(&td.cfg.data_dir),
        serde_json::to_vec_pretty(&json!({
            "model_base_url": "https://api.example.com/v1",
            "model_id": "legacy-model",
            "api_key": leg_model_key
        }))
        .unwrap(),
    )
    .unwrap();

    // Trigger transparent migrations via standard load APIs
    let policy = ergon_rs::load_workspace_policy(&td.cfg.data_dir);
    let _ = connectors::list_connectors_json(&td.cfg.data_dir, &policy);
    let _ = mcp::list_mcp_servers_json(&td.cfg.data_dir, &policy);
    let migrated_model = load_saved_model_config(&td.cfg.data_dir).unwrap();
    assert_eq!(migrated_model.api_key.as_deref(), Some(leg_model_key));

    // Recursive scan across EVERY file in ERGON_DATA_DIR must find ZERO legacy plaintext strings!
    assert_zero_plaintext_in_dir(
        &td.cfg.data_dir,
        &[
            leg_v1_tok,
            leg_v1_sig,
            leg_v1_hook,
            leg_smtp_pass,
            leg_imap_pass,
            leg_oa2_csec,
            leg_oa2_rt,
            leg_oauth_file_at,
            leg_oauth_file_cs,
            leg_mcp_bearer,
            leg_mcp_env,
            leg_model_key,
        ],
    );

    // Verify metadata files now carry opaque vault: references
    let cs_doc: Value = serde_json::from_str(
        &std::fs::read_to_string(connectors::connector_secrets_path(&td.cfg.data_dir)).unwrap(),
    )
    .unwrap();
    assert_eq!(cs_doc["version"], json!(2));
    assert_eq!(cs_doc["storage"], json!("vault"));
    assert!(cs_doc["secret_refs"]["legacy_partner"]
        .as_str()
        .unwrap()
        .starts_with("vault:"));

    let oauth_doc: Value = serde_json::from_str(
        &std::fs::read_to_string(connectors::legacy_oauth_path(&td.cfg.data_dir)).unwrap(),
    )
    .unwrap();
    assert!(oauth_doc["access_token"].as_str().unwrap().starts_with("vault:"));
    assert_eq!(oauth_doc["has_access_token"], json!(true));
    assert_eq!(oauth_doc["has_client_secret"], json!(true));

    let cfg_doc: Value = serde_json::from_str(
        &std::fs::read_to_string(ergon_rs::config_file_path(&td.cfg.data_dir)).unwrap(),
    )
    .unwrap();
    assert!(cfg_doc.get("api_key").is_none());
    assert_eq!(cfg_doc["api_key_ref"], json!("vault:model:api_key"));
    assert_eq!(cfg_doc["has_api_key"], json!(true));

    // Verify every migrated secret decrypts to its exact original value
    let store = connectors::load_connector_secrets(&td.cfg.data_dir).unwrap();
    assert_eq!(store.secrets.get("legacy_partner").map(String::as_str), Some(leg_v1_tok));
    assert_eq!(
        store.secrets.get("smtp_pass_conn-legacy-smtp").map(String::as_str),
        Some(leg_smtp_pass)
    );
    assert_eq!(
        store.secrets.get("imap_pass_trg-legacy-imap").map(String::as_str),
        Some(leg_imap_pass)
    );
    assert_eq!(
        store.secrets.get("oauth_client_secret_conn-legacy-smtp").map(String::as_str),
        Some(leg_oa2_csec)
    );
    assert_eq!(
        store.secrets.get("oauth_refresh_token_conn-legacy-smtp").map(String::as_str),
        Some(leg_oa2_rt)
    );
    assert_eq!(
        store.secrets.get("oauth_access_token").map(String::as_str),
        Some(leg_oauth_file_at)
    );
    assert_eq!(
        store.secrets.get("oauth_client_secret").map(String::as_str),
        Some(leg_oauth_file_cs)
    );
    assert_eq!(
        store.secrets.get("mcp_auth_legmcp").map(String::as_str),
        Some(leg_mcp_bearer)
    );
    assert_eq!(
        store.secrets.get("mcp_env_legmcp_api_key").map(String::as_str),
        Some(leg_mcp_env)
    );

    // Verify migrated webhook trigger still verifies live HMAC-SHA256 requests!
    let body = br#"{"migrated":true}"#;
    let mac = tripwire::hmac_sha256(leg_v1_sig.as_bytes(), body);
    let sig_hdr = format!("sha256={}", tripwire::hex(&mac));
    let res = connectors::verify_and_ingest_webhook_with_meta(
        &td.cfg,
        "hk_legacy_1234",
        Some(&sig_hdr),
        None,
        None,
        None,
        "application/json",
        body,
    );
    assert!(res.is_ok(), "migrated webhook must authenticate: {res:?}");
}

#[test]
fn env_secret_key_mode_and_headless_keychain_fallback_work_cleanly() {
    // 1. Explicit master key (equivalent to ERGON_SECRET_KEY) keeps .vault.key off disk
    let td_env = TestDir::new("env-key");
    {
        let _ov = secret_store::ScopedVaultOverride::new(
            &td_env.cfg.data_dir,
            secret_store::VaultOverride {
                key: Some("ergon-kms-master-key-prod-01".to_string()),
                backend: Some(secret_store::BackendPreference::AeadFile),
                disable_os_keychain_probe: true,
            },
        );
        secret_store::set_secret(&td_env.cfg.data_dir, "connector:secret:kms_tok", "kms-secret-xyz")
            .unwrap();
        assert!(
            !secret_store::vault_key_path(&td_env.cfg.data_dir).exists(),
            ".vault.key must NOT be created when key is supplied externally"
        );
        assert_zero_plaintext_in_dir(&td_env.cfg.data_dir, &["kms-secret-xyz", "ergon-kms-master-key-prod-01"]);
        assert_eq!(
            secret_store::get_secret(&td_env.cfg.data_dir, "connector:secret:kms_tok").unwrap(),
            Some("kms-secret-xyz".to_string())
        );
    }

    // Opening the same vault with a WRONG master key fails authentication cleanly
    {
        let _wrong = secret_store::ScopedVaultOverride::new(
            &td_env.cfg.data_dir,
            secret_store::VaultOverride {
                key: Some("wrong-master-key-99".to_string()),
                backend: Some(secret_store::BackendPreference::AeadFile),
                disable_os_keychain_probe: true,
            },
        );
        let err = secret_store::get_secret(&td_env.cfg.data_dir, "connector:secret:kms_tok")
            .unwrap_err();
        assert!(err.contains("authentication failed"), "{err}");
    }

    // 2. Requesting OS keychain on a headless server (probe unavailable) cleanly falls back to AEAD file
    let td_headless = TestDir::new("headless-fallback");
    {
        let _ov = secret_store::ScopedVaultOverride::new(
            &td_headless.cfg.data_dir,
            secret_store::VaultOverride {
                key: None,
                backend: Some(secret_store::BackendPreference::OsKeychain),
                disable_os_keychain_probe: true,
            },
        );
        secret_store::set_secret(
            &td_headless.cfg.data_dir,
            "connector:secret:headless_tok",
            "headless-secret-777",
        )
        .unwrap();
        assert!(secret_store::vault_path(&td_headless.cfg.data_dir).exists());
        assert!(secret_store::vault_key_path(&td_headless.cfg.data_dir).exists());
        assert_zero_plaintext_in_dir(&td_headless.cfg.data_dir, &["headless-secret-777"]);
        assert_eq!(
            secret_store::get_secret(&td_headless.cfg.data_dir, "connector:secret:headless_tok")
                .unwrap(),
            Some("headless-secret-777".to_string())
        );
        let info = secret_store::active_backend_info(&td_headless.cfg.data_dir);
        assert_eq!(info["backend"], json!("aead_file"));
        assert_eq!(info["cipher"], json!("aes-256-gcm"));
    }
}
