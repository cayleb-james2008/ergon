//! Ergon binary: bind, serve, and optionally open a chromeless desktop app
//! window (Chromium `--app=` on Linux, Edge/Chrome `--app=` on Windows).
//!
//! Configuration via flags or env:
//!   --version / -V          Print version and exit
//!   --help / -h             Print usage and exit
//!   --app / --open          Open chromeless desktop app window
//!   --no-open               Headless/server mode (never open a window)
//!   --host <HOST>           Bind host (default ERGON_HOST or 127.0.0.1)
//!   --port <PORT>           Bind port (default ERGON_PORT or 8787)
//!   ERGON_DATA_DIR, ERGON_WORKSPACE, ERGON_MODEL_BASE_URL,
//!   ERGON_MODEL_ID, ERGON_MODEL_API_KEY, ERGON_NO_OPEN, ERGON_OPEN,
//!   ERGON_DISABLE_SCHEDULER, ERGON_SCHEDULER_TICK_MS.

use std::net::TcpListener;
use std::process::{Command, Stdio};

#[derive(Debug, Default, PartialEq, Eq)]
struct CliOpts {
    show_version: bool,
    show_help: bool,
    force_open: bool,
    no_open: bool,
    tls: bool,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    tls_init_mode: bool,
    tls_sans: Option<String>,
    user_mode: bool,
    user_action: Option<String>,
    user_target: Option<String>,
    user_password: Option<String>,
    user_role: Option<String>,
    user_name: Option<String>,
    user_email: Option<String>,
    verify_mode: bool,
    verify_target: Option<String>,
    verify_workspace: Option<String>,
    export_handoff_mode: bool,
    export_out: Option<String>,
    export_title: Option<String>,
    export_force: bool,
    json_output: bool,
    host: Option<String>,
    port: Option<u16>,
}

fn parse_cli_args(args: &[String]) -> CliOpts {
    let mut opts = CliOpts::default();
    let mut i = 0;
    while i < args.len() {
        let in_subcmd =
            opts.verify_mode || opts.export_handoff_mode || opts.user_mode || opts.tls_init_mode;
        match args[i].as_str() {
            "--version" | "-V" => opts.show_version = true,
            "--help" | "-h" => opts.show_help = true,
            "--app" | "--open" => opts.force_open = true,
            "--no-open" => opts.no_open = true,
            "--tls" => opts.tls = true,
            "--json" => opts.json_output = true,
            "--force" => opts.export_force = true,
            "verify" if !in_subcmd => opts.verify_mode = true,
            "export-handoff" if !in_subcmd => opts.export_handoff_mode = true,
            "user" if !in_subcmd => opts.user_mode = true,
            "tls" if !in_subcmd => {
                if args.get(i + 1).map(String::as_str) == Some("init") {
                    opts.tls_init_mode = true;
                    i += 1;
                }
            }
            "--tls-cert" => {
                if let Some(v) = args.get(i + 1) {
                    opts.tls_cert = Some(v.clone());
                    opts.tls = true;
                    i += 1;
                }
            }
            "--tls-key" => {
                if let Some(v) = args.get(i + 1) {
                    opts.tls_key = Some(v.clone());
                    opts.tls = true;
                    i += 1;
                }
            }
            "--sans" => {
                if let Some(v) = args.get(i + 1) {
                    opts.tls_sans = Some(v.clone());
                    i += 1;
                }
            }
            "--password" => {
                if let Some(v) = args.get(i + 1) {
                    opts.user_password = Some(v.clone());
                    i += 1;
                }
            }
            "--role" => {
                if let Some(v) = args.get(i + 1) {
                    opts.user_role = Some(v.clone());
                    i += 1;
                }
            }
            "--name" => {
                if let Some(v) = args.get(i + 1) {
                    opts.user_name = Some(v.clone());
                    i += 1;
                }
            }
            "--email" => {
                if let Some(v) = args.get(i + 1) {
                    opts.user_email = Some(v.clone());
                    i += 1;
                }
            }
            "--workspace" => {
                if let Some(v) = args.get(i + 1) {
                    opts.verify_workspace = Some(v.clone());
                    i += 1;
                }
            }
            "--out" => {
                if let Some(v) = args.get(i + 1) {
                    opts.export_out = Some(v.clone());
                    i += 1;
                }
            }
            "--title" => {
                if let Some(v) = args.get(i + 1) {
                    opts.export_title = Some(v.clone());
                    i += 1;
                }
            }
            "--host" => {
                if let Some(v) = args.get(i + 1) {
                    opts.host = Some(v.clone());
                    i += 1;
                }
            }
            "--port" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u16>().ok()) {
                    opts.port = Some(v);
                    i += 1;
                }
            }
            other => {
                if let Some(v) = other.strip_prefix("--host=") {
                    if !v.is_empty() {
                        opts.host = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--port=").and_then(|s| s.parse::<u16>().ok()) {
                    opts.port = Some(v);
                } else if let Some(v) = other.strip_prefix("--tls-cert=") {
                    if !v.is_empty() {
                        opts.tls_cert = Some(v.to_string());
                        opts.tls = true;
                    }
                } else if let Some(v) = other.strip_prefix("--tls-key=") {
                    if !v.is_empty() {
                        opts.tls_key = Some(v.to_string());
                        opts.tls = true;
                    }
                } else if let Some(v) = other.strip_prefix("--sans=") {
                    if !v.is_empty() {
                        opts.tls_sans = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--password=") {
                    opts.user_password = Some(v.to_string());
                } else if let Some(v) = other.strip_prefix("--role=") {
                    if !v.is_empty() {
                        opts.user_role = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--name=") {
                    if !v.is_empty() {
                        opts.user_name = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--email=") {
                    if !v.is_empty() {
                        opts.user_email = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--workspace=") {
                    if !v.is_empty() {
                        opts.verify_workspace = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--out=") {
                    if !v.is_empty() {
                        opts.export_out = Some(v.to_string());
                    }
                } else if let Some(v) = other.strip_prefix("--title=") {
                    if !v.is_empty() {
                        opts.export_title = Some(v.to_string());
                    }
                } else if opts.user_mode && !other.starts_with('-') {
                    if opts.user_action.is_none() {
                        opts.user_action = Some(other.to_string());
                    } else if opts.user_target.is_none() {
                        opts.user_target = Some(other.to_string());
                    } else if opts.user_password.is_none() {
                        opts.user_password = Some(other.to_string());
                    }
                } else if opts.verify_mode && !other.starts_with('-') && opts.verify_target.is_none() {
                    opts.verify_target = Some(other.to_string());
                } else if opts.export_handoff_mode
                    && !other.starts_with('-')
                    && opts.export_out.is_none()
                {
                    opts.export_out = Some(other.to_string());
                }
            }
        }
        i += 1;
    }
    opts
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            let low = v.trim().to_ascii_lowercase();
            matches!(low.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

fn should_open_desktop_window(
    opts: &CliOpts,
    env_no_open: bool,
    env_force_open: bool,
    env_ci: bool,
    env_invocation_id: bool,
    has_display: bool,
) -> bool {
    if opts.no_open || env_no_open || env_ci || !has_display {
        return false;
    }
    if opts.force_open || env_force_open {
        return true;
    }
    !env_invocation_id
}

fn desktop_url_for_bind(host: &str, port: u16, tls: bool) -> String {
    let scheme = if tls { "https" } else { "http" };
    let trimmed = host.trim();
    let connect_host = match trimmed {
        "" | "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        h if h.contains(':') && !h.starts_with('[') => return format!("{scheme}://[{h}]:{port}/"),
        h => h,
    };
    format!("{scheme}://{connect_host}:{port}/")
}

#[cfg(target_os = "windows")]
fn has_desktop_display() -> bool {
    true
}

#[cfg(target_os = "macos")]
fn has_desktop_display() -> bool {
    true
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn has_desktop_display() -> bool {
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn spawn_detached(program: &str, args: &[String]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

#[cfg(target_os = "windows")]
fn launch_chromeless_window(url: &str) {
    let app_arg = format!("--app={url}");
    let app_args = vec![app_arg, "--new-window".to_string()];

    let mut candidates: Vec<String> = vec![
        "msedge.exe".to_string(),
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(),
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".to_string(),
        "chrome.exe".to_string(),
        r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string(),
        "brave.exe".to_string(),
    ];
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        candidates.push(format!(r"{local_app}\Google\Chrome\Application\chrome.exe"));
        candidates.push(format!(r"{local_app}\Microsoft\Edge\Application\msedge.exe"));
    }
    for bin in &candidates {
        if spawn_detached(bin, &app_args) {
            return;
        }
    }
    let _ = spawn_detached(
        "cmd",
        &[
            "/C".to_string(),
            "start".to_string(),
            String::new(),
            url.to_string(),
        ],
    );
}

#[cfg(target_os = "macos")]
fn launch_chromeless_window(url: &str) {
    let app_args = vec![format!("--app={url}"), "--new-window".to_string()];
    for bin in [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    ] {
        if spawn_detached(bin, &app_args) {
            return;
        }
    }
    let _ = spawn_detached("open", &[url.to_string()]);
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn launch_chromeless_window(url: &str) {
    let app_args = vec![
        format!("--app={url}"),
        "--class=ergon".to_string(),
        "--new-window".to_string(),
    ];
    for bin in [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "brave",
        "brave-browser",
        "microsoft-edge",
        "microsoft-edge-stable",
    ] {
        if spawn_detached(bin, &app_args) {
            return;
        }
    }
    let _ = spawn_detached("xdg-open", &[url.to_string()]);
}

fn maybe_open_desktop_window(opts: &CliOpts, host: &str, port: u16, tls: bool) {
    let env_no_open = std::env::var_os("ERGON_NO_OPEN").is_some_and(|v| {
        let s = v.to_string_lossy().trim().to_ascii_lowercase();
        !s.is_empty() && s != "0" && s != "false"
    });
    let env_force_open = env_truthy("ERGON_OPEN");
    let env_ci = std::env::var_os("CI").is_some();
    let env_invocation_id = std::env::var_os("INVOCATION_ID").is_some();
    let has_display = has_desktop_display();
    if !should_open_desktop_window(
        opts,
        env_no_open,
        env_force_open,
        env_ci,
        env_invocation_id,
        has_display,
    ) {
        return;
    }
    let url = desktop_url_for_bind(host, port, tls);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        launch_chromeless_window(&url);
    });
}

#[cold]
fn run_export_handoff_cli(opts: &CliOpts) -> i32 {
    let cfg = ergon_rs::Config::from_env();
    let people = ergon_rs::PeopleState::load(&cfg);
    let person = people.active_person();
    let mut p_cfg = cfg.person_config(&person);
    if let Some(ws) = &opts.verify_workspace {
        p_cfg.workspace_dir = std::path::PathBuf::from(ws);
    }
    let policy = ergon_rs::load_workspace_policy(&p_cfg.data_dir);
    let security = ergon_rs::tripwire::audit_section(&cfg.data_dir, &person.id);
    let title = opts.export_title.as_deref().unwrap_or("Client Handoff");
    let summary_md = ergon_rs::server::build_handoff_summary_markdown(&p_cfg, &person, title);

    match ergon_rs::audit_bundle::build_handoff_bundle_zip(
        &p_cfg,
        &person,
        title,
        &summary_md,
        &security,
        &policy,
        opts.export_force,
    ) {
        Err(blocked) => {
            if opts.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&blocked.to_json()).unwrap_or_default()
                );
            } else {
                eprintln!(
                    "ergon export-handoff: BLOCKED by {} policy ({} finding(s))",
                    blocked.policy.label, blocked.scan.leak_count
                );
                for f in &blocked.scan.findings {
                    eprintln!("  {}:L{} [{}] {}", f.file, f.line, f.kind, f.fix);
                }
                eprintln!("Fix the findings above or pass --force to export anyway.");
            }
            1
        }
        Ok(res) => {
            let out_arg = opts
                .export_out
                .as_deref()
                .unwrap_or("ergon-client-handoff.zip");
            let out_path = std::path::Path::new(out_arg);
            let is_zip = out_arg.to_ascii_lowercase().ends_with(".zip");
            if is_zip {
                if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("ergon export-handoff: cannot create {}: {e}", parent.display());
                        return 2;
                    }
                }
                if let Err(e) = std::fs::write(out_path, &res.zip_bytes) {
                    eprintln!("ergon export-handoff: cannot write {}: {e}", out_path.display());
                    return 2;
                }
            } else {
                if let Err(e) = std::fs::create_dir_all(out_path) {
                    eprintln!("ergon export-handoff: cannot create {}: {e}", out_path.display());
                    return 2;
                }
                for (name, bytes) in [
                    ("CLIENT_SUMMARY.md", res.summary_md.as_bytes()),
                    ("audit_bundle.json", res.audit_bundle_bytes.as_slice()),
                    ("VERIFY.txt", res.verify_txt.as_bytes()),
                    ("SHA256SUMS", res.sha256sums.as_bytes()),
                ] {
                    if let Err(e) = std::fs::write(out_path.join(name), bytes) {
                        eprintln!("ergon export-handoff: cannot write {name}: {e}");
                        return 2;
                    }
                }
            }
            if opts.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "mode": if is_zip { "zip" } else { "dir" },
                        "output": out_arg,
                        "sha256": res.sha256,
                        "zip_bytes": res.zip_bytes.len(),
                        "policy": policy.profile,
                        "scan": res.scan.to_json(),
                    }))
                    .unwrap_or_default()
                );
            } else {
                println!(
                    "ergon export-handoff: PASS -> {} ({} bytes, sha256:{}, leaks:{})",
                    out_arg,
                    res.zip_bytes.len(),
                    &res.sha256[..12.min(res.sha256.len())],
                    res.scan.leak_count
                );
            }
            0
        }
    }
}

#[cold]
fn run_user_cli(opts: &CliOpts) -> i32 {
    let cfg = ergon_rs::Config::from_env();
    if let Err(e) = cfg.ensure_dirs() {
        eprintln!("ergon user: cannot initialize data directory: {e}");
        return 1;
    }
    let people = std::sync::RwLock::new(ergon_rs::PeopleState::load(&cfg));
    let auth = ergon_rs::auth::AuthStore::load(&cfg.data_dir);
    let action = opts.user_action.as_deref().unwrap_or("list");
    match action {
        "list" => {
            let users = auth.list_users_public();
            if opts.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "users": users }))
                        .unwrap_or_default()
                );
            } else if users.is_empty() {
                println!("No local user accounts configured yet.");
            } else {
                for u in &users {
                    println!(
                        "{}\t{}\t{}\tworkspace={}",
                        u["username"].as_str().unwrap_or(""),
                        u["role"].as_str().unwrap_or(""),
                        u["display_name"].as_str().unwrap_or(""),
                        u["person_id"].as_str().unwrap_or(""),
                    );
                }
            }
            0
        }
        "add" => {
            let Some(username) = opts.user_target.as_deref() else {
                eprintln!("usage: ergon user add <USERNAME> --password <PASS> [--role admin|pm|ae|office|installer|member] [--name NAME] [--email EMAIL]");
                return 2;
            };
            let password = opts
                .user_password
                .clone()
                .or_else(|| std::env::var("ERGON_USER_PASSWORD").ok())
                .unwrap_or_default();
            if password.is_empty() {
                eprintln!("ergon user add: pass --password <PASS> or set ERGON_USER_PASSWORD");
                return 2;
            }
            let default_role = if auth.has_users() { "member" } else { "admin" };
            let role = opts.user_role.as_deref().unwrap_or(default_role);
            match auth.create_user(
                username,
                &password,
                role,
                opts.user_name.as_deref(),
                opts.user_email.as_deref(),
                &people,
                &cfg,
            ) {
                Ok(user) => {
                    if opts.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "ok": true,
                                "user": user.to_public_json()
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!(
                            "Created user '{}' (role={}, workspace={})",
                            user.username,
                            user.role.as_str(),
                            user.person_id
                        );
                    }
                    0
                }
                Err((_, msg)) => {
                    eprintln!("ergon user add: {msg}");
                    1
                }
            }
        }
        "passwd" => {
            let Some(username) = opts.user_target.as_deref() else {
                eprintln!("usage: ergon user passwd <USERNAME> --password <NEW_PASS>");
                return 2;
            };
            let password = opts
                .user_password
                .clone()
                .or_else(|| std::env::var("ERGON_USER_PASSWORD").ok())
                .unwrap_or_default();
            if password.is_empty() {
                eprintln!("ergon user passwd: pass --password <NEW_PASS> or set ERGON_USER_PASSWORD");
                return 2;
            }
            match auth.set_password_cli(username, &password) {
                Ok(user) => {
                    if opts.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "ok": true,
                                "user": user.to_public_json()
                            }))
                            .unwrap_or_default()
                        );
                    } else {
                        println!("Updated password for '{}' (existing sessions revoked).", user.username);
                    }
                    0
                }
                Err(msg) => {
                    eprintln!("ergon user passwd: {msg}");
                    1
                }
            }
        }
        other => {
            eprintln!("ergon user: unknown subcommand '{other}' (expected add, passwd, or list)");
            2
        }
    }
}

#[cold]
fn run_tls_init_cli(opts: &CliOpts) -> i32 {
    let cfg = ergon_rs::Config::from_env();
    if let Err(e) = cfg.ensure_dirs() {
        eprintln!("ergon tls init: cannot initialize data directory: {e}");
        return 1;
    }
    let mut sans = ergon_rs::allowed_hosts_from(&std::env::var("ERGON_ALLOWED_HOSTS").unwrap_or_default());
    if let Some(ref extra) = opts.tls_sans {
        sans.extend(ergon_rs::allowed_hosts_from(extra));
    }
    let tls_dir = cfg.data_dir.join("tls");
    match ergon_rs::tls::ensure_self_signed_cert(&tls_dir, &sans) {
        Ok((cert, key)) => {
            let loaded = ergon_rs::tls::load_tls_config(&cert, &key, true);
            let fp = loaded
                .as_ref()
                .map(|c| c.fingerprint_sha256.as_str())
                .unwrap_or("");
            if opts.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "cert": cert.display().to_string(),
                        "key": key.display().to_string(),
                        "sha256": fp,
                    }))
                    .unwrap_or_default()
                );
            } else {
                println!("TLS cert: {}", cert.display());
                println!("TLS key:  {} (mode 0600)", key.display());
                if !fp.is_empty() {
                    println!("SHA-256:  {fp}");
                }
            }
            0
        }
        Err(e) => {
            eprintln!("ergon tls init: {e}");
            1
        }
    }
}

fn print_help() {
    println!(
        "ergon {} - Personal Employee Agent Desk\n\n\
         USAGE:\n  ergon [--app | --no-open] [--host HOST] [--port PORT] [--tls] [--tls-cert FILE --tls-key FILE] [--version] [--help]\n  \
         ergon user <list | add USERNAME | passwd USERNAME> [--password PASS] [--role ROLE] [--name NAME] [--email EMAIL] [--json]\n  \
         ergon tls init [--sans HOST1,HOST2] [--json]\n  \
         ergon verify <BUNDLE_OR_PACK> [--workspace DIR] [--json]\n  \
         ergon export-handoff [--out PATH] [--title TITLE] [--workspace DIR] [--force] [--json]\n\n\
         FLAGS:\n  --app, --open       Open chromeless desktop app window (Chromium/Edge --app)\n  \
         --no-open           Run headless without opening a desktop window\n  \
         --host HOST         Bind address (default: ERGON_HOST or 127.0.0.1)\n  \
         --port PORT         Bind port (default: ERGON_PORT or 8787)\n  \
         --tls               Enable HTTPS/TLS (auto-generates self-signed cert in $ERGON_DATA_DIR/tls)\n  \
         --tls-cert FILE     PEM certificate chain path (or ERGON_TLS_CERT)\n  \
         --tls-key FILE      PEM private key path (or ERGON_TLS_KEY)\n  \
         --workspace DIR     Cross-check bundle/pack or override workspace directory\n  \
         --out PATH          Handoff output .zip file or directory (default: ergon-client-handoff.zip)\n  \
         --title TITLE       Handoff bundle title (default: Client Handoff)\n  \
         --force             Override Strict Client leak pre-check block\n  \
         --json              Emit machine-readable JSON\n  \
         -V, --version       Print version and exit\n  \
         -h, --help          Print this help and exit",
        env!("CARGO_PKG_VERSION")
    );
}

#[cold]
fn run_verify_cli(opts: &CliOpts) -> i32 {
    let Some(target) = opts.verify_target.as_deref() else {
        eprintln!("ergon verify requires a path to an audit bundle (.json), client pack (.md), or playbook pack (.json).");
        return 2;
    };
    let content = match std::fs::read_to_string(target) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ergon verify: cannot read {target}: {e}");
            return 2;
        }
    };
    let ws = opts.verify_workspace.as_deref().map(std::path::Path::new);
    let res = ergon_rs::audit_bundle::verify_evidence(&content, Some(target), ws);
    if opts.json_output {
        println!("{}", res.to_json());
    } else {
        println!(
            "{} [{}] {}",
            if res.valid { "PASS" } else { "FAIL" },
            res.kind,
            res.summary
        );
        if !res.actual_sha256.is_empty() {
            println!("sha256: {}", res.actual_sha256);
        }
        for err in &res.errors {
            println!("error: {err}");
        }
    }
    if res.valid { 0 } else { 1 }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn tune_allocator_for_low_rss() {
    // Cap glibc per-thread arenas and mmap larger allocations so 16-thread
    // HTTP bursts do not fragment into extra multi-MB resident arenas.
    unsafe extern "C" {
        fn mallopt(param: std::ffi::c_int, value: std::ffi::c_int) -> std::ffi::c_int;
    }
    const M_MMAP_THRESHOLD: std::ffi::c_int = -3;
    const M_ARENA_MAX: std::ffi::c_int = -8;
    unsafe {
        mallopt(M_ARENA_MAX, 1);
        mallopt(M_MMAP_THRESHOLD, 32 * 1024);
    }
}

fn main() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    tune_allocator_for_low_rss();
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    if raw_args.first().map(String::as_str) == Some("--internal-pdf-text") {
        use std::io::Read as _;
        let mut buf = Vec::new();
        if std::io::stdin().take(32 * 1024 * 1024).read_to_end(&mut buf).is_err() {
            std::process::exit(1);
        }
        let out = match ergon_rs::docs::pdf_extract_pages_raw(&buf) {
            Ok(pages) => serde_json::json!({"ok": true, "pages": pages}),
            Err(error) => serde_json::json!({"ok": false, "error": error}),
        };
        println!("{out}");
        return;
    }
    let opts = parse_cli_args(&raw_args);
    if opts.show_version {
        println!("ergon {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if opts.show_help {
        print_help();
        return;
    }
    if opts.verify_mode {
        std::process::exit(run_verify_cli(&opts));
    }
    if opts.export_handoff_mode {
        std::process::exit(run_export_handoff_cli(&opts));
    }
    if opts.user_mode {
        std::process::exit(run_user_cli(&opts));
    }
    if opts.tls_init_mode {
        std::process::exit(run_tls_init_cli(&opts));
    }
    let cfg = ergon_rs::Config::from_env();
    let host = opts
        .host
        .clone()
        .or_else(|| std::env::var("ERGON_HOST").ok())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port: u16 = opts
        .port
        .or_else(|| std::env::var("ERGON_PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(8787);
    let allowed_hosts =
        ergon_rs::allowed_hosts_from(&std::env::var("ERGON_ALLOWED_HOSTS").unwrap_or_default());
    let tls_cfg = match ergon_rs::tls::resolve_tls(
        &cfg.data_dir,
        opts.tls,
        opts.tls_cert.as_deref().map(std::path::Path::new),
        opts.tls_key.as_deref().map(std::path::Path::new),
        &allowed_hosts,
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ergon: TLS configuration error: {e}");
            std::process::exit(1);
        }
    };
    let tls_active = tls_cfg.is_some();
    let listener = match TcpListener::bind((host.as_str(), port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ergon: cannot bind {host}:{port}: {e}");
            std::process::exit(1);
        }
    };
    let scheme = if tls_active { "https" } else { "http" };
    println!("ergon listening on {scheme}://{host}:{port}");
    if let Some(ref tls) = tls_cfg {
        println!(
            "tls: {} ({}, sha256:{})",
            tls.cert_path.display(),
            if tls.self_signed { "self-signed" } else { "custom cert" },
            &tls.fingerprint_sha256[..16.min(tls.fingerprint_sha256.len())]
        );
    }
    println!(
        "model: {} @ {} ({})",
        cfg.model_id,
        cfg.model_base_url,
        if cfg.api_key.is_some() {
            "key configured"
        } else if cfg.model_configured() {
            "local loopback ready"
        } else {
            "NOT configured"
        }
    );
    maybe_open_desktop_window(&opts, &host, port, tls_active);
    if let Err(e) = ergon_rs::serve_with_tls(listener, cfg, tls_cfg, true) {
        eprintln!("ergon: server error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_desktop_flags_and_overrides() {
        let args: Vec<String> = ["--app", "--port", "8473", "--host=0.0.0.0"]
            .into_iter()
            .map(String::from)
            .collect();
        let opts = parse_cli_args(&args);
        assert!(opts.force_open);
        assert!(!opts.no_open);
        assert_eq!(opts.port, Some(8473));
        assert_eq!(opts.host.as_deref(), Some("0.0.0.0"));

        let no_open = parse_cli_args(&["--no-open".to_string(), "--version".to_string()]);
        assert!(no_open.no_open);
        assert!(no_open.show_version);
    }

    #[test]
    fn desktop_window_gating_respects_no_open_headless_ci_and_systemd() {
        let base = CliOpts::default();
        assert!(should_open_desktop_window(&base, false, false, false, false, true));
        assert!(!should_open_desktop_window(&base, false, false, false, true, true));
        assert!(!should_open_desktop_window(&base, false, false, false, false, false));
        assert!(!should_open_desktop_window(&base, true, true, false, false, true));
        assert!(!should_open_desktop_window(&base, false, true, true, false, true));

        let explicit_app = CliOpts {
            force_open: true,
            ..CliOpts::default()
        };
        assert!(should_open_desktop_window(
            &explicit_app,
            false,
            false,
            false,
            true,
            true
        ));

        let conflict = CliOpts {
            force_open: true,
            no_open: true,
            ..CliOpts::default()
        };
        assert!(!should_open_desktop_window(
            &conflict, false, true, false, false, true
        ));
    }

    #[test]
    fn desktop_url_normalizes_wildcard_binds() {
        assert_eq!(desktop_url_for_bind("0.0.0.0", 8473, false), "http://127.0.0.1:8473/");
        assert_eq!(desktop_url_for_bind("::", 8473, false), "http://127.0.0.1:8473/");
        assert_eq!(desktop_url_for_bind("127.0.0.1", 8787, false), "http://127.0.0.1:8787/");
        assert_eq!(desktop_url_for_bind("::1", 8787, false), "http://[::1]:8787/");
        assert_eq!(desktop_url_for_bind("127.0.0.1", 8787, true), "https://127.0.0.1:8787/");
    }

    #[test]
    fn cli_parses_tls_and_user_subcommands() {
        let tls_args: Vec<String> = ["--tls", "--port", "8443"].into_iter().map(String::from).collect();
        let tls_opts = parse_cli_args(&tls_args);
        assert!(tls_opts.tls);
        assert_eq!(tls_opts.port, Some(8443));

        let user_args: Vec<String> = [
            "user",
            "add",
            "maya",
            "--role",
            "pm",
            "--password",
            "SecretPass123",
            "--json",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let uo = parse_cli_args(&user_args);
        assert!(uo.user_mode);
        assert_eq!(uo.user_action.as_deref(), Some("add"));
        assert_eq!(uo.user_target.as_deref(), Some("maya"));
        assert_eq!(uo.user_role.as_deref(), Some("pm"));
        assert_eq!(uo.user_password.as_deref(), Some("SecretPass123"));
        assert!(uo.json_output);
    }

    #[test]
    fn cli_parses_verify_subcommand_and_flags() {
        let args: Vec<String> = [
            "verify",
            "ergon-audit-bundle.json",
            "--workspace",
            "./ws",
            "--json",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let opts = parse_cli_args(&args);
        assert!(opts.verify_mode);
        assert_eq!(opts.verify_target.as_deref(), Some("ergon-audit-bundle.json"));
        assert_eq!(opts.verify_workspace.as_deref(), Some("./ws"));
        assert!(opts.json_output);
    }

    #[test]
    fn cli_parses_export_handoff_flags() {
        let args: Vec<String> = [
            "export-handoff",
            "--out",
            "/tmp/out.zip",
            "--title=Q4 Review",
            "--workspace",
            "./ws",
            "--force",
            "--json",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let opts = parse_cli_args(&args);
        assert!(opts.export_handoff_mode);
        assert_eq!(opts.export_out.as_deref(), Some("/tmp/out.zip"));
        assert_eq!(opts.export_title.as_deref(), Some("Q4 Review"));
        assert_eq!(opts.verify_workspace.as_deref(), Some("./ws"));
        assert!(opts.export_force);
        assert!(opts.json_output);
    }
}
