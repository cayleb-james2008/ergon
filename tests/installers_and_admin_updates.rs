//! Integration tests for Gap 8 (MVP Row 7):
//! 1. Admin update flow behind `ERGON_GATE_TOKEN` / `ERGON_ADMIN_TOKEN` (no blanket 403 for admins,
//!    strict 401 for unauthenticated callers, strict 403 for non-admin/wrong-token callers).
//! 2. Cross-platform `ergon service` manifest generation, validation, and CLI execution.
//! 3. Windows MSI/PowerShell, `.github/workflows/release.yml`, and `docs/SIGNING.md` integrity.

use ergon_rs::service::{
    render_launchd_plist, render_systemd_unit, render_windows_sc_commands,
    render_windows_service_manifest_json, validate_service_config, ServiceConfig,
};
use ergon_rs::update::{asset_name, sha256_hex, Config as UpdateConfig, Mode, Updater};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn test_root(tag: &str) -> PathBuf {
    let n = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!(
        "ergon-gap8-{tag}-{}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn test_app_config(root: &Path) -> ergon_rs::Config {
    let data = root.join("data");
    let ws = data.join("workspace");
    std::fs::create_dir_all(&ws).unwrap();
    ergon_rs::Config {
        data_dir: data.clone(),
        workspace_dir: ws,
        memory_path: data.join("MEMORY.md"),
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "test-model".to_string(),
        api_key: None,
    }
}

fn raw_http(port: u16, head: &str, body: &str) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let req = format!(
        "{head}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    let _ = s.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf).to_string();
    let (hdr, bdy) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = hdr
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    (status, bdy.to_string())
}

fn req_http(
    port: u16,
    method: &str,
    path: &str,
    host: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, Value) {
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\n");
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    let payload = body.unwrap_or("");
    if body.is_some() {
        head.push_str("Content-Type: application/json\r\n");
    }
    let (status, raw) = raw_http(port, &head, payload);
    let val = serde_json::from_str(&raw).unwrap_or_else(|_| json!({"raw": raw}));
    (status, val)
}

fn start_mock_release_server(version: &str, payload: Vec<u8>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let want_asset = asset_name("ergon");
    let digest = sha256_hex(&payload);
    let sums = format!("{digest}  {want_asset}\n");
    let version = version.to_string();
    std::thread::spawn(move || {
        for s in listener.incoming() {
            let Ok(mut s) = s else { continue };
            let mut buf = [0u8; 2048];
            let n = s.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req.split_whitespace().nth(1).unwrap_or("");
            let (ctype, body): (&str, Vec<u8>) = if path.ends_with("/releases/latest") {
                let rel = json!({
                    "tag_name": format!("v{version}"),
                    "body": "Verified release notes for admin update test",
                    "html_url": format!("https://github.com/cayleb-james2008/ergon/releases/tag/v{version}"),
                    "assets": [
                        {
                            "name": want_asset,
                            "browser_download_url": format!("http://127.0.0.1:{port}/dl/bin"),
                            "size": payload.len(),
                        },
                        {
                            "name": "SHA256SUMS",
                            "browser_download_url": format!("http://127.0.0.1:{port}/dl/sums"),
                        }
                    ]
                });
                ("application/json", rel.to_string().into_bytes())
            } else if path == "/dl/bin" {
                ("application/octet-stream", payload.clone())
            } else if path == "/dl/sums" {
                ("text/plain", sums.as_bytes().to_vec())
            } else {
                let _ = s.write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
                continue;
            };
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = s.write_all(head.as_bytes());
            let _ = s.write_all(&body);
        }
    });
    port
}

fn fake_executable(dir: &Path, version: &str) -> PathBuf {
    let p = dir.join("ergon-fake");
    std::fs::write(&p, format!("#!/bin/sh\necho 'ergon {version}'\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    p
}

fn start_server_with_mock_updater(
    root: &Path,
    gate_token: Option<&str>,
    admin_token: Option<&str>,
    allowed_hosts: Vec<String>,
    mock_port: u16,
    exe: &Path,
) -> u16 {
    let cfg = test_app_config(root);
    let mode_path = cfg.data_dir.join("update_config.json");
    let updater = Arc::new(
        Updater::new(UpdateConfig {
            bin: "ergon",
            current: env!("CARGO_PKG_VERSION"),
            repo: "cayleb-james2008/ergon".to_string(),
            api: format!("http://127.0.0.1:{mock_port}"),
            mode: Mode::Notify,
            exe: exe.to_path_buf(),
            allow_dev: true,
            may_apply: gate_token.is_none(),
        })
        .with_mode_path(mode_path),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let gt = gate_token.map(String::from);
    let at = admin_token.map(String::from);
    std::thread::spawn(move || {
        let _ = ergon_rs::serve_with_updater(listener, cfg, gt, at, allowed_hosts, updater);
    });
    std::thread::sleep(Duration::from_millis(40));
    port
}

#[cfg(unix)]
#[test]
fn gated_admin_can_check_configure_mode_and_install_while_non_admins_get_403_and_unauth_gets_401() {
    let root = test_root("gated_admin");
    let exe = fake_executable(&root, env!("CARGO_PKG_VERSION"));
    let new_bin = b"#!/bin/sh\necho 'ergon 99.0.0'\n".to_vec();
    let mock_port = start_mock_release_server("99.0.0", new_bin.clone());
    let port = start_server_with_mock_updater(
        &root,
        Some("GateSecret-99"),
        None,
        vec![],
        mock_port,
        &exe,
    );
    let remote_host = format!("ergon.company.internal:{port}");

    // 1. Unauthenticated callers get 401 across all update routes.
    for (method, path, body) in [
        ("GET", "/api/update", None),
        ("POST", "/api/update/check", Some("{}")),
        ("POST", "/api/update/mode", Some(r#"{"mode":"auto"}"#)),
        ("POST", "/api/update/install", Some("{}")),
        ("POST", "/api/update/apply", Some("{}")),
        ("POST", "/api/update/restart", Some("{}")),
    ] {
        let (st, _) = req_http(port, method, path, &remote_host, &[], body);
        assert_eq!(st, 401, "unauthenticated {method} {path} must return 401");
    }

    // Wrong gate/bearer token alone also returns 401.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/install",
        &remote_host,
        &[("Authorization", "Bearer wrong-token")],
        Some("{}"),
    );
    assert_eq!(st, 401, "wrong bearer token without valid gate must return 401");

    // 2. Regular gated user (X-Ergon-Gate only, no admin role/token) gets read-only GET & 403 on mutations.
    let (st, snap) = req_http(
        port,
        "GET",
        "/api/update",
        &remote_host,
        &[("X-Ergon-Gate", "GateSecret-99")],
        None,
    );
    assert_eq!(st, 200);
    assert_eq!(snap["is_admin"], json!(false));
    assert_eq!(snap["can_manage"], json!(false));
    assert_eq!(snap["can_apply"], json!(false));
    assert_eq!(snap["gated"], json!(true));

    for (path, body) in [
        ("/api/update/check", "{}"),
        ("/api/update/mode", r#"{"mode":"auto"}"#),
        ("/api/update/install", "{}"),
        ("/api/update/apply", "{}"),
        ("/api/update/restart", "{}"),
    ] {
        let (st, _) = req_http(
            port,
            "POST",
            path,
            &remote_host,
            &[("X-Ergon-Gate", "GateSecret-99")],
            Some(body),
        );
        assert_eq!(st, 403, "plain gated non-admin POST {path} must return 403");
    }

    // Explicit non-admin role MUST be rejected with 403 even if Bearer token matches.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/install",
        &remote_host,
        &[
            ("Authorization", "Bearer GateSecret-99"),
            ("X-Ergon-Role", "member"),
        ],
        Some("{}"),
    );
    assert_eq!(st, 403, "explicit non-admin role must be refused with 403");

    // Valid gate + wrong admin token MUST be rejected with 403.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/mode",
        &remote_host,
        &[
            ("X-Ergon-Gate", "GateSecret-99"),
            ("X-Ergon-Admin-Token", "wrong-admin"),
        ],
        Some(r#"{"mode":"auto"}"#),
    );
    assert_eq!(st, 403, "valid gate + wrong admin token must return 403");

    // Client-supplied X-Ergon-Admin: 1 behind X-Ergon-Gate alone MUST be rejected with 403.
    let (st, snap_optin) = req_http(
        port,
        "GET",
        "/api/update",
        &remote_host,
        &[
            ("X-Ergon-Gate", "GateSecret-99"),
            ("X-Ergon-Admin", "1"),
        ],
        None,
    );
    assert_eq!(st, 200);
    assert_eq!(snap_optin["is_admin"], json!(false), "standalone X-Ergon-Admin must not grant admin");
    for (path, body) in [
        ("/api/update/mode", r#"{"mode":"auto"}"#),
        ("/api/update/install", "{}"),
    ] {
        let (st, _) = req_http(
            port,
            "POST",
            path,
            &remote_host,
            &[
                ("X-Ergon-Gate", "GateSecret-99"),
                ("X-Ergon-Admin", "1"),
            ],
            Some(body),
        );
        assert_eq!(st, 403, "X-Ergon-Gate + X-Ergon-Admin: 1 alone must return 403 on {path}");
    }

    // Self-escalation via user-editable person.role="admin" MUST be rejected with 403.
    let (st, created) = req_http(
        port,
        "POST",
        "/api/people",
        &remote_host,
        &[("X-Ergon-Gate", "GateSecret-99")],
        Some(r#"{"name":"Self Escalator","role":"admin"}"#),
    );
    assert_eq!(st, 201, "create person failed: {created}");
    let created_id = created["person"]["id"]
        .as_str()
        .or_else(|| created["id"].as_str())
        .expect("created person id");
    let (st, _) = req_http(
        port,
        "POST",
        "/api/people/switch",
        &remote_host,
        &[("X-Ergon-Gate", "GateSecret-99")],
        Some(&json!({"id": created_id}).to_string()),
    );
    assert_eq!(st, 200, "switch person failed");
    let (st, snap_person) = req_http(
        port,
        "GET",
        "/api/update",
        &remote_host,
        &[("X-Ergon-Gate", "GateSecret-99")],
        None,
    );
    assert_eq!(st, 200);
    assert_eq!(
        snap_person["is_admin"],
        json!(false),
        "editable person.role='admin' must NEVER grant update admin behind gate"
    );
    for (path, body) in [
        ("/api/update/mode", r#"{"mode":"auto"}"#),
        ("/api/update/install", "{}"),
    ] {
        let (st, _) = req_http(
            port,
            "POST",
            path,
            &remote_host,
            &[("X-Ergon-Gate", "GateSecret-99")],
            Some(body),
        );
        assert_eq!(
            st, 403,
            "plain X-Ergon-Gate caller switched to role='admin' profile must still get 403 on {path}"
        );
    }

    // 3. Authenticated admin via Authorization: Bearer <gate_token> gets full access (even over non-loopback Host!).
    let bearer = &[("Authorization", "Bearer GateSecret-99")];
    let (st, snap) = req_http(port, "GET", "/api/update", &remote_host, bearer, None);
    assert_eq!(st, 200);
    assert_eq!(snap["is_admin"], json!(true));
    assert_eq!(snap["can_manage"], json!(true));
    assert_eq!(snap["gated"], json!(true));

    // Invalid mode -> 400.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/mode",
        &remote_host,
        bearer,
        Some(r#"{"mode":"turbo"}"#),
    );
    assert_eq!(st, 400);

    // Valid mode -> 200 and persisted to update_config.json.
    let (st, mode_snap) = req_http(
        port,
        "POST",
        "/api/update/mode",
        &remote_host,
        bearer,
        Some(r#"{"mode":"auto"}"#),
    );
    assert_eq!(st, 200, "admin POST /api/update/mode failed: {mode_snap}");
    assert_eq!(mode_snap["mode"], json!("auto"));
    let persisted = std::fs::read_to_string(root.join("data/update_config.json")).unwrap();
    assert!(persisted.contains("\"auto\""), "mode must persist: {persisted}");

    // Admin via X-Ergon-Token can trigger check.
    let (st, checked) = req_http(
        port,
        "POST",
        "/api/update/check",
        &remote_host,
        &[("X-Ergon-Token", "GateSecret-99")],
        Some("{}"),
    );
    assert_eq!(st, 200, "admin POST /api/update/check failed: {checked}");
    assert_eq!(checked["available"], json!(true));
    assert_eq!(checked["latest"], json!("99.0.0"));
    assert_eq!(checked["can_apply"], json!(true));

    // Admin via X-Ergon-Gate + X-Ergon-Role: admin can install the update for real (NO blanket 403!).
    let (st, installed) = req_http(
        port,
        "POST",
        "/api/update/install",
        &remote_host,
        &[
            ("X-Ergon-Gate", "GateSecret-99"),
            ("X-Ergon-Role", "admin"),
        ],
        Some("{}"),
    );
    assert_eq!(st, 200, "admin POST /api/update/install failed: {installed}");
    assert_eq!(installed["installed"], json!("99.0.0"));
    assert_eq!(installed["staged"], json!("99.0.0"));
    assert_eq!(
        std::fs::read(&exe).unwrap(),
        new_bin,
        "authenticated admin install behind gate must atomically swap executable"
    );
}

#[cfg(unix)]
#[test]
fn dedicated_admin_token_and_local_loopback_admin_rules_are_enforced() {
    let root = test_root("dedicated_admin");
    let exe = fake_executable(&root, env!("CARGO_PKG_VERSION"));
    let new_bin = b"#!/bin/sh\necho 'ergon 98.0.0'\n".to_vec();
    let mock_port = start_mock_release_server("98.0.0", new_bin.clone());
    let port = start_server_with_mock_updater(
        &root,
        Some("Gate-Only"),
        Some("Admin-Only-77"),
        vec![],
        mock_port,
        &exe,
    );
    let host = format!("ergon.internal:{port}");

    // Gate + role header alone is NOT sufficient when dedicated ERGON_ADMIN_TOKEN is set.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/check",
        &host,
        &[("X-Ergon-Gate", "Gate-Only"), ("X-Ergon-Role", "admin")],
        Some("{}"),
    );
    assert_eq!(st, 403, "dedicated admin token must be required when configured");

    // Wrong admin token -> 403.
    let (st, _) = req_http(
        port,
        "POST",
        "/api/update/mode",
        &host,
        &[
            ("X-Ergon-Gate", "Gate-Only"),
            ("Authorization", "Bearer Wrong-Admin"),
        ],
        Some(r#"{"mode":"off"}"#),
    );
    assert_eq!(st, 403);

    // Correct dedicated admin token -> 200 on check + apply.
    let (st, checked) = req_http(
        port,
        "POST",
        "/api/update/check",
        &host,
        &[("Authorization", "Bearer Admin-Only-77")],
        Some("{}"),
    );
    assert_eq!(st, 200, "{checked}");
    assert_eq!(checked["latest"], json!("98.0.0"));

    let (st, applied) = req_http(
        port,
        "POST",
        "/api/update/apply",
        &host,
        &[("X-Ergon-Token", "Admin-Only-77")],
        Some("{}"),
    );
    assert_eq!(st, 200, "{applied}");
    assert_eq!(applied["applied"], json!("98.0.0"));
    assert_eq!(std::fs::read(&exe).unwrap(), new_bin);

    // Ungated server: loopback is local admin, LAN host is still 403.
    let root2 = test_root("loopback_admin");
    let exe2 = fake_executable(&root2, env!("CARGO_PKG_VERSION"));
    let port2 = start_server_with_mock_updater(
        &root2,
        None,
        None,
        vec!["lan-desk.local".to_string()],
        mock_port,
        &exe2,
    );
    let (st, _) = req_http(
        port2,
        "POST",
        "/api/update/mode",
        &format!("lan-desk.local:{port2}"),
        &[],
        Some(r#"{"mode":"off"}"#),
    );
    assert_eq!(st, 403, "LAN host without admin token must remain 403");

    let (st, snap) = req_http(
        port2,
        "POST",
        "/api/update/mode",
        &format!("127.0.0.1:{port2}"),
        &[],
        Some(r#"{"mode":"off"}"#),
    );
    assert_eq!(st, 200, "loopback local admin must configure update mode: {snap}");
    assert_eq!(snap["mode"], json!("off"));
    // When mode is off, install returns 409 ("updates are turned off"), NOT 403.
    let (st, err_v) = req_http(
        port2,
        "POST",
        "/api/update/install",
        &format!("127.0.0.1:{port2}"),
        &[],
        Some("{}"),
    );
    assert_eq!(st, 409, "mode=off install returns 409: {err_v}");
}

#[test]
fn service_manifests_generate_and_validate_for_linux_macos_and_windows() {
    let linux_cfg = ServiceConfig {
        service_name: "ergon".to_string(),
        display_name: "Ergon Personal Agent Server".to_string(),
        description: "Ergon — an AI agent for every employee".to_string(),
        exe_path: PathBuf::from("/usr/local/bin/ergon"),
        data_dir: PathBuf::from("/var/lib/ergon"),
        host: "127.0.0.1".to_string(),
        port: 8787,
        system_wide: true,
    };
    assert!(validate_service_config(&linux_cfg).is_ok());
    let unit = render_systemd_unit(&linux_cfg).unwrap();
    assert!(unit.contains("ExecStart=\"/usr/local/bin/ergon\" service run"));
    assert!(unit.contains("WantedBy=multi-user.target"));

    let mac_cfg = ServiceConfig {
        service_name: "com.ergon.agent".to_string(),
        display_name: "Ergon Personal Agent Server".to_string(),
        description: "Ergon agent".to_string(),
        exe_path: PathBuf::from("/Applications/Ergon.app/Contents/MacOS/ergon"),
        data_dir: PathBuf::from("/Library/Application Support/Ergon"),
        host: "127.0.0.1".to_string(),
        port: 8787,
        system_wide: false,
    };
    let plist = render_launchd_plist(&mac_cfg).unwrap();
    assert!(plist.contains("<plist version=\"1.0\">"));
    assert!(plist.contains("<string>com.ergon.agent</string>"));
    assert!(plist.contains("<string>/Applications/Ergon.app/Contents/MacOS/ergon</string>"));

    let win_cfg = ServiceConfig {
        service_name: "ErgonService".to_string(),
        display_name: "Ergon Personal Agent Server".to_string(),
        description: "Ergon Windows Service".to_string(),
        exe_path: PathBuf::from(r"C:\Program Files\Ergon\ergon.exe"),
        data_dir: PathBuf::from(r"C:\ProgramData\Ergon\data"),
        host: "127.0.0.1".to_string(),
        port: 8787,
        system_wide: true,
    };
    let sc = render_windows_sc_commands(&win_cfg).unwrap();
    assert_eq!(sc.len(), 4);
    assert!(sc[0][3].contains("service run --name ErgonService"));
    let mut trailing_slash_cfg = win_cfg.clone();
    trailing_slash_cfg.data_dir = PathBuf::from(r"C:\ProgramData\Ergon\data\");
    let sc_slash = render_windows_sc_commands(&trailing_slash_cfg).unwrap();
    assert!(
        sc_slash[0][3].contains(r#"--data "C:\ProgramData\Ergon\data\.""#),
        "trailing backslash must not escape closing quote: {}",
        sc_slash[0][3]
    );
    let win_json: Value =
        serde_json::from_str(&render_windows_service_manifest_json(&win_cfg).unwrap()).unwrap();
    assert_eq!(win_json["service_name"], json!("ErgonService"));
    assert_eq!(win_json["start_type"], json!("auto"));

    // Exercise the compiled `ergon` CLI binary for `ergon service print-manifest`.
    let bin = env!("CARGO_BIN_EXE_ergon");
    for fmt in ["systemd", "launchd", "windows"] {
        let out = std::process::Command::new(bin)
            .args(["service", "print-manifest", "--format", fmt, "--json"])
            .output()
            .expect("invoke ergon service print-manifest");
        assert!(
            out.status.success(),
            "print-manifest --format {fmt} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let parsed: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(parsed["ok"], json!(true));
        assert_eq!(parsed["format"], json!(fmt));
        assert!(!parsed["manifest"].as_str().unwrap_or("").is_empty());
    }
}

#[test]
fn packaging_scripts_xml_manifests_release_workflow_and_signing_docs_are_valid() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Windows WiX v4 MSI + PowerShell scripts
    let wxs = std::fs::read_to_string(repo_root.join("packaging/windows/ergon.wxs")).unwrap();
    for needle in [
        "http://wixtoolset.org/schemas/v4/wxs",
        "http://wixtoolset.org/schemas/v4/wxs/util",
        "ProgramFiles64Folder",
        "CommonAppDataFolder",
        "ServiceInstall",
        "ServiceControl",
        "util:ServiceConfig",
        "ErgonService",
        "MajorUpgrade",
        "&quot;[DATAFOLDER].&quot;",
    ] {
        assert!(wxs.contains(needle), "ergon.wxs missing required element: {needle}");
    }

    let install_ps1 =
        std::fs::read_to_string(repo_root.join("packaging/windows/install-windows.ps1")).unwrap();
    assert!(install_ps1.contains("ErgonService"));
    assert!(install_ps1.contains("service install"));
    assert!(install_ps1.contains("New-NetFirewallRule"));

    let uninstall_ps1 =
        std::fs::read_to_string(repo_root.join("packaging/windows/uninstall-windows.ps1")).unwrap();
    assert!(uninstall_ps1.contains("ErgonService"));
    assert!(uninstall_ps1.contains("PurgeData"));

    let build_msi =
        std::fs::read_to_string(repo_root.join("packaging/windows/build-msi.ps1")).unwrap();
    assert!(build_msi.contains("wix build"));
    assert!(build_msi.contains("signtool"));
    assert!(build_msi.contains("WINDOWS_SIGN_PFX_BASE64"));

    // Release workflow & signing docs
    let workflow =
        std::fs::read_to_string(repo_root.join(".github/workflows/release.yml")).unwrap();
    for needle in [
        "cargo test --locked",
        "packaging/windows/build-msi.ps1",
        "WINDOWS_SIGN_PFX_BASE64",
        "WINDOWS_SIGN_PFX_PASSWORD",
        "SHA256SUMS",
    ] {
        assert!(workflow.contains(needle), "release.yml missing: {needle}");
    }
    assert!(!workflow.contains("apple-darwin"), "release.yml must not build macOS (Windows/Linux only)");

    let signing_doc = std::fs::read_to_string(repo_root.join("docs/SIGNING.md")).unwrap();
    for secret in [
        "WINDOWS_SIGN_PFX_BASE64",
        "WINDOWS_SIGN_PFX_PASSWORD",
        "WINDOWS_SIGN_TIMESTAMP_URL",
        "SHA256SUMS",
    ] {
        assert!(signing_doc.contains(secret), "docs/SIGNING.md missing: {secret}");
    }
}
