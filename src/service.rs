//! Cross-platform service management (`ergon service install|uninstall|status|run|print-manifest`).
//!
//! Generates validated service manifests for:
//! - Linux (`systemd` user/system units, matching `install.sh` conventions)
//! - macOS (`launchd` LaunchAgent / LaunchDaemon `.plist` XML)
//! - Windows (`sc.exe` service registration with automatic restart recovery + native Win32 SCM dispatcher)

use serde_json::json;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestFormat {
    Systemd,
    Launchd,
    Windows,
}

impl ManifestFormat {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "systemd" | "linux" => Some(Self::Systemd),
            "launchd" | "macos" | "darwin" | "plist" => Some(Self::Launchd),
            "windows" | "win" | "sc" => Some(Self::Windows),
            _ => None,
        }
    }

    pub fn host_default() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Launchd
        } else {
            Self::Systemd
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Systemd => "systemd",
            Self::Launchd => "launchd",
            Self::Windows => "windows",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceConfig {
    pub service_name: String,
    pub display_name: String,
    pub description: String,
    pub exe_path: PathBuf,
    pub data_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub system_wide: bool,
}

impl ServiceConfig {
    pub fn default_service_name_for(format: &ManifestFormat) -> &'static str {
        match format {
            ManifestFormat::Windows => "ErgonService",
            ManifestFormat::Launchd => "com.ergon.agent",
            ManifestFormat::Systemd => "ergon",
        }
    }

    pub fn default_data_dir_for(format: &ManifestFormat, system_wide: bool) -> PathBuf {
        match format {
            ManifestFormat::Windows => {
                if let Ok(pd) = std::env::var("ProgramData") {
                    if !pd.trim().is_empty() {
                        return PathBuf::from(pd).join("Ergon").join("data");
                    }
                }
                PathBuf::from(r"C:\ProgramData\Ergon\data")
            }
            ManifestFormat::Launchd => {
                if system_wide {
                    PathBuf::from("/Library/Application Support/Ergon")
                } else if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join("Ergon")
                } else {
                    PathBuf::from("/Library/Application Support/Ergon")
                }
            }
            ManifestFormat::Systemd => {
                if system_wide {
                    PathBuf::from("/var/lib/ergon")
                } else if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home).join(".local").join("share").join("ergon")
                } else {
                    PathBuf::from("/var/lib/ergon")
                }
            }
        }
    }

    pub fn from_host_env(
        service_name: Option<String>,
        data_dir: Option<PathBuf>,
        host: Option<String>,
        port: Option<u16>,
        system_wide: bool,
        format: &ManifestFormat,
    ) -> Self {
        let exe_path = std::env::current_exe().unwrap_or_else(|_| {
            if matches!(format, ManifestFormat::Windows) {
                PathBuf::from(r"C:\Program Files\Ergon\ergon.exe")
            } else {
                PathBuf::from("/usr/local/bin/ergon")
            }
        });
        let resolved_data = data_dir
            .or_else(|| {
                std::env::var("ERGON_DATA_DIR")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| Self::default_data_dir_for(format, system_wide));
        let resolved_host = host
            .or_else(|| {
                std::env::var("ERGON_HOST")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let resolved_port = port
            .or_else(|| {
                std::env::var("ERGON_PORT")
                    .ok()
                    .and_then(|s| s.trim().parse::<u16>().ok())
            })
            .unwrap_or(8787);
        Self {
            service_name: service_name
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| Self::default_service_name_for(format).to_string()),
            display_name: "Ergon Personal Agent Server".to_string(),
            description: "Ergon — an AI agent for every employee".to_string(),
            exe_path,
            data_dir: resolved_data,
            host: resolved_host,
            port: resolved_port,
            system_wide,
        }
    }
}

fn has_control_chars(s: &str) -> bool {
    s.chars().any(|c| c.is_control())
}

/// Validates service name, paths, bind host, and port before writing manifests or invoking OS tools.
pub fn validate_service_config(cfg: &ServiceConfig) -> Result<(), String> {
    if cfg.service_name.is_empty() || cfg.service_name.len() > 128 {
        return Err("service name must be 1..=128 characters".to_string());
    }
    if !cfg
        .service_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(
            "service name may contain only ASCII letters, digits, '.', '-', and '_'".to_string(),
        );
    }
    if cfg.port == 0 {
        return Err("service port must be between 1 and 65535".to_string());
    }
    let host = cfg.host.trim();
    if host.is_empty() || has_control_chars(host) || host.chars().any(char::is_whitespace) {
        return Err("service host must be a non-empty hostname or IP without whitespace".to_string());
    }
    let exe_str = cfg.exe_path.to_string_lossy();
    if exe_str.trim().is_empty() || has_control_chars(&exe_str) {
        return Err("executable path must be non-empty without control characters".to_string());
    }
    let data_str = cfg.data_dir.to_string_lossy();
    if data_str.trim().is_empty() || has_control_chars(&data_str) {
        return Err("data directory must be non-empty without control characters".to_string());
    }
    Ok(())
}

fn systemd_quote(val: &str) -> String {
    let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn xml_escape(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Renders a Linux systemd unit file matching `install.sh` conventions.
pub fn render_systemd_unit(cfg: &ServiceConfig) -> Result<String, String> {
    validate_service_config(cfg)?;
    let exe = systemd_quote(&cfg.exe_path.to_string_lossy());
    let data_env = systemd_quote(&format!("ERGON_DATA_DIR={}", cfg.data_dir.to_string_lossy()));
    let host_env = systemd_quote(&format!("ERGON_HOST={}", cfg.host.trim()));
    let port_env = systemd_quote(&format!("ERGON_PORT={}", cfg.port));
    let wanted_by = if cfg.system_wide {
        "multi-user.target"
    } else {
        "default.target"
    };
    Ok(format!(
        "[Unit]\n\
         Description={desc}\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         Environment={data_env}\n\
         Environment={host_env}\n\
         Environment={port_env}\n\
         Environment=\"ERGON_NO_OPEN=1\"\n\
         # Environment=\"ERGON_MODEL_API_KEY=\"\n\
         # Environment=\"ERGON_GATE_TOKEN=\"\n\
         # Environment=\"ERGON_ADMIN_TOKEN=\"\n\
         ExecStart={exe} service run --data {data_arg} --host {host} --port {port}\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy={wanted_by}\n",
        desc = cfg.description,
        data_arg = systemd_quote(&cfg.data_dir.to_string_lossy()),
        host = cfg.host.trim(),
        port = cfg.port,
    ))
}

/// Renders a valid macOS `launchd` LaunchAgent / LaunchDaemon `.plist` XML manifest.
pub fn render_launchd_plist(cfg: &ServiceConfig) -> Result<String, String> {
    validate_service_config(cfg)?;
    let log_dir = cfg.data_dir.join("logs");
    let stdout_log = log_dir.join("ergon-service.out.log");
    let stderr_log = log_dir.join("ergon-service.err.log");
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
           <key>Label</key>\n\
           <string>{label}</string>\n\
           <key>ProgramArguments</key>\n\
           <array>\n\
             <string>{exe}</string>\n\
             <string>service</string>\n\
             <string>run</string>\n\
             <string>--data</string>\n\
             <string>{data}</string>\n\
             <string>--host</string>\n\
             <string>{host}</string>\n\
             <string>--port</string>\n\
             <string>{port}</string>\n\
           </array>\n\
           <key>EnvironmentVariables</key>\n\
           <dict>\n\
             <key>ERGON_DATA_DIR</key>\n\
             <string>{data}</string>\n\
             <key>ERGON_HOST</key>\n\
             <string>{host}</string>\n\
             <key>ERGON_PORT</key>\n\
             <string>{port}</string>\n\
             <key>ERGON_NO_OPEN</key>\n\
             <string>1</string>\n\
           </dict>\n\
           <key>RunAtLoad</key>\n\
           <true/>\n\
           <key>KeepAlive</key>\n\
           <dict>\n\
             <key>SuccessfulExit</key>\n\
             <false/>\n\
           </dict>\n\
           <key>StandardOutPath</key>\n\
           <string>{out_log}</string>\n\
           <key>StandardErrorPath</key>\n\
           <string>{err_log}</string>\n\
         </dict>\n\
         </plist>\n",
        label = xml_escape(&cfg.service_name),
        exe = xml_escape(&cfg.exe_path.to_string_lossy()),
        data = xml_escape(&cfg.data_dir.to_string_lossy()),
        host = xml_escape(cfg.host.trim()),
        port = cfg.port,
        out_log = xml_escape(&stdout_log.to_string_lossy()),
        err_log = xml_escape(&stderr_log.to_string_lossy()),
    ))
}

/// Renders the exact `sc.exe` registration commands for a Windows Service with automatic restart recovery.
pub fn render_windows_sc_commands(cfg: &ServiceConfig) -> Result<Vec<Vec<String>>, String> {
    validate_service_config(cfg)?;
    let exe = cfg.exe_path.to_string_lossy().replace('"', "");
    let data = cfg.data_dir.to_string_lossy().replace('"', "");
    let bin_path = format!(
        "\"{exe}\" service run --name {} --data \"{data}\" --host {} --port {}",
        cfg.service_name,
        cfg.host.trim(),
        cfg.port
    );
    Ok(vec![
        vec![
            "create".to_string(),
            cfg.service_name.clone(),
            "binPath=".to_string(),
            bin_path,
            "DisplayName=".to_string(),
            cfg.display_name.clone(),
            "start=".to_string(),
            "auto".to_string(),
        ],
        vec![
            "description".to_string(),
            cfg.service_name.clone(),
            cfg.description.clone(),
        ],
        vec![
            "failure".to_string(),
            cfg.service_name.clone(),
            "reset=".to_string(),
            "86400".to_string(),
            "actions=".to_string(),
            "restart/5000/restart/15000/restart/30000".to_string(),
        ],
        vec![
            "failureflag".to_string(),
            cfg.service_name.clone(),
            "1".to_string(),
        ],
    ])
}

/// Renders a Windows service JSON manifest (describing `sc.exe` registration & service environment).
pub fn render_windows_service_manifest_json(cfg: &ServiceConfig) -> Result<String, String> {
    let commands = render_windows_sc_commands(cfg)?;
    serde_json::to_string_pretty(&json!({
        "platform": "windows",
        "service_name": cfg.service_name,
        "display_name": cfg.display_name,
        "description": cfg.description,
        "exe_path": cfg.exe_path.to_string_lossy(),
        "data_dir": cfg.data_dir.to_string_lossy(),
        "host": cfg.host,
        "port": cfg.port,
        "start_type": "auto",
        "recovery": {
            "reset_secs": 86400,
            "actions": ["restart/5000", "restart/15000", "restart/30000"],
        },
        "sc_commands": commands,
    }))
    .map_err(|e| format!("failed to serialize Windows service manifest: {e}"))
}

pub fn render_manifest(cfg: &ServiceConfig, format: &ManifestFormat) -> Result<String, String> {
    match format {
        ManifestFormat::Systemd => render_systemd_unit(cfg),
        ManifestFormat::Launchd => render_launchd_plist(cfg),
        ManifestFormat::Windows => render_windows_service_manifest_json(cfg),
    }
}

#[derive(Clone, Debug)]
pub struct ServiceStatusReport {
    pub service_name: String,
    pub platform: &'static str,
    pub installed: bool,
    pub running: bool,
    pub state: String,
    pub unit_path: Option<String>,
    pub detail: String,
}

impl ServiceStatusReport {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "service_name": self.service_name,
            "platform": self.platform,
            "installed": self.installed,
            "running": self.running,
            "state": self.state,
            "unit_path": self.unit_path,
            "detail": self.detail,
        })
    }
}

/// Runs the Ergon server headlessly with the supplied service configuration.
fn run_headless_server(cfg: &ServiceConfig) -> Result<(), String> {
    validate_service_config(cfg)?;
    std::fs::create_dir_all(&cfg.data_dir)
        .map_err(|e| format!("cannot create data dir {}: {e}", cfg.data_dir.display()))?;
    // Configure process environment for the server without opening any browser window.
    std::env::set_var("ERGON_DATA_DIR", &cfg.data_dir);
    std::env::set_var("ERGON_HOST", cfg.host.trim());
    std::env::set_var("ERGON_PORT", cfg.port.to_string());
    std::env::set_var("ERGON_NO_OPEN", "1");

    let app_cfg = crate::Config::from_env();
    let listener = TcpListener::bind((cfg.host.trim(), cfg.port))
        .map_err(|e| format!("cannot bind {}:{}: {e}", cfg.host.trim(), cfg.port))?;
    println!(
        "ergon service ({}) listening on http://{}:{}",
        cfg.service_name,
        cfg.host.trim(),
        cfg.port
    );
    crate::serve_with_updates(listener, app_cfg).map_err(|e| format!("server error: {e}"))
}

// ---------------------------------------------------------------------------
// Linux (systemd)
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn linux_unit_path(cfg: &ServiceConfig) -> Result<PathBuf, String> {
    if cfg.system_wide {
        Ok(PathBuf::from("/etc/systemd/system").join(format!("{}.service", cfg.service_name)))
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user")
            .join(format!("{}.service", cfg.service_name)))
    }
}

#[cfg(target_os = "linux")]
pub fn install_service(cfg: &ServiceConfig) -> Result<String, String> {
    let unit = render_systemd_unit(cfg)?;
    std::fs::create_dir_all(&cfg.data_dir)
        .map_err(|e| format!("cannot create data dir {}: {e}", cfg.data_dir.display()))?;
    let unit_path = linux_unit_path(cfg)?;
    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(&unit_path, unit)
        .map_err(|e| format!("cannot write {}: {e}", unit_path.display()))?;

    let mut cmd = Command::new("systemctl");
    if !cfg.system_wide {
        cmd.arg("--user");
    }
    let reloaded = cmd.arg("daemon-reload").status().is_ok_and(|s| s.success());
    if reloaded {
        let mut enable = Command::new("systemctl");
        if !cfg.system_wide {
            enable.arg("--user");
        }
        let enabled = enable
            .args(["enable", "--now", &cfg.service_name])
            .status()
            .is_ok_and(|s| s.success());
        if enabled {
            Ok(format!(
                "installed and started systemd service '{}' at {}",
                cfg.service_name,
                unit_path.display()
            ))
        } else {
            Ok(format!(
                "wrote systemd unit at {} (daemon reloaded; enable manually with systemctl {} enable --now {})",
                unit_path.display(),
                if cfg.system_wide { "" } else { "--user" },
                cfg.service_name
            ))
        }
    } else {
        Ok(format!(
            "wrote systemd unit at {} (no active systemd bus answered; start directly or enable from a logged-in session)",
            unit_path.display()
        ))
    }
}

#[cfg(target_os = "linux")]
pub fn uninstall_service(cfg: &ServiceConfig) -> Result<String, String> {
    validate_service_config(cfg)?;
    let unit_path = linux_unit_path(cfg)?;
    let mut disable = Command::new("systemctl");
    if !cfg.system_wide {
        disable.arg("--user");
    }
    let _ = disable.args(["disable", "--now", &cfg.service_name]).status();
    if unit_path.exists() {
        std::fs::remove_file(&unit_path)
            .map_err(|e| format!("cannot remove {}: {e}", unit_path.display()))?;
    }
    let mut reload = Command::new("systemctl");
    if !cfg.system_wide {
        reload.arg("--user");
    }
    let _ = reload.arg("daemon-reload").status();
    Ok(format!(
        "removed systemd service '{}' ({})",
        cfg.service_name,
        unit_path.display()
    ))
}

#[cfg(target_os = "linux")]
pub fn service_status(cfg: &ServiceConfig) -> Result<ServiceStatusReport, String> {
    validate_service_config(cfg)?;
    let unit_path = linux_unit_path(cfg)?;
    let installed = unit_path.exists();
    let mut cmd = Command::new("systemctl");
    if !cfg.system_wide {
        cmd.arg("--user");
    }
    let out = cmd.args(["is-active", &cfg.service_name]).output();
    let (running, state, detail) = match out {
        Ok(o) => {
            let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let active = o.status.success() && txt == "active";
            let st = if !txt.is_empty() {
                txt
            } else if installed {
                "installed (bus unavailable)".to_string()
            } else {
                "not-installed".to_string()
            };
            let det = if !err.is_empty() { err } else { st.clone() };
            (active, st, det)
        }
        Err(e) => (
            false,
            if installed { "installed" } else { "not-installed" }.to_string(),
            format!("systemctl unavailable: {e}"),
        ),
    };
    Ok(ServiceStatusReport {
        service_name: cfg.service_name.clone(),
        platform: "linux",
        installed,
        running,
        state,
        unit_path: Some(unit_path.to_string_lossy().to_string()),
        detail,
    })
}

#[cfg(target_os = "linux")]
pub fn run_service(cfg: &ServiceConfig) -> Result<(), String> {
    run_headless_server(cfg)
}

// ---------------------------------------------------------------------------
// macOS (launchd)
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn macos_plist_path(cfg: &ServiceConfig) -> Result<PathBuf, String> {
    if cfg.system_wide {
        Ok(PathBuf::from("/Library/LaunchDaemons").join(format!("{}.plist", cfg.service_name)))
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", cfg.service_name)))
    }
}

#[cfg(target_os = "macos")]
pub fn install_service(cfg: &ServiceConfig) -> Result<String, String> {
    let plist = render_launchd_plist(cfg)?;
    std::fs::create_dir_all(cfg.data_dir.join("logs"))
        .map_err(|e| format!("cannot create data/log dir {}: {e}", cfg.data_dir.display()))?;
    let plist_path = macos_plist_path(cfg)?;
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    std::fs::write(&plist_path, plist)
        .map_err(|e| format!("cannot write {}: {e}", plist_path.display()))?;
    let _ = Command::new("launchctl")
        .arg("unload")
        .arg("-w")
        .arg(&plist_path)
        .status();
    let loaded = Command::new("launchctl")
        .arg("load")
        .arg("-w")
        .arg(&plist_path)
        .status()
        .is_ok_and(|s| s.success());
    if loaded {
        Ok(format!(
            "installed and loaded launchd service '{}' at {}",
            cfg.service_name,
            plist_path.display()
        ))
    } else {
        Ok(format!(
            "wrote launchd plist at {} (load from a logged-in macOS session with: launchctl load -w {})",
            plist_path.display(),
            plist_path.display()
        ))
    }
}

#[cfg(target_os = "macos")]
pub fn uninstall_service(cfg: &ServiceConfig) -> Result<String, String> {
    validate_service_config(cfg)?;
    let plist_path = macos_plist_path(cfg)?;
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .arg("unload")
            .arg("-w")
            .arg(&plist_path)
            .status();
        std::fs::remove_file(&plist_path)
            .map_err(|e| format!("cannot remove {}: {e}", plist_path.display()))?;
    }
    Ok(format!(
        "removed launchd service '{}' ({})",
        cfg.service_name,
        plist_path.display()
    ))
}

#[cfg(target_os = "macos")]
pub fn service_status(cfg: &ServiceConfig) -> Result<ServiceStatusReport, String> {
    validate_service_config(cfg)?;
    let plist_path = macos_plist_path(cfg)?;
    let installed = plist_path.exists();
    let out = Command::new("launchctl")
        .args(["list", &cfg.service_name])
        .output();
    let (running, state, detail) = match out {
        Ok(o) if o.status.success() => {
            let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let has_pid = txt.contains("\"PID\"");
            (
                has_pid,
                if has_pid { "running" } else { "loaded" }.to_string(),
                txt,
            )
        }
        Ok(o) => (
            false,
            if installed { "stopped" } else { "not-installed" }.to_string(),
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => (
            false,
            if installed { "installed" } else { "not-installed" }.to_string(),
            format!("launchctl unavailable: {e}"),
        ),
    };
    Ok(ServiceStatusReport {
        service_name: cfg.service_name.clone(),
        platform: "macos",
        installed,
        running,
        state,
        unit_path: Some(plist_path.to_string_lossy().to_string()),
        detail,
    })
}

#[cfg(target_os = "macos")]
pub fn run_service(cfg: &ServiceConfig) -> Result<(), String> {
    run_headless_server(cfg)
}

// ---------------------------------------------------------------------------
// Windows (sc.exe + native Win32 Service Control Manager dispatcher)
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn run_sc(args: &[String]) -> Result<String, String> {
    let out = Command::new("sc.exe")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run sc.exe: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "sc.exe {} failed (exit {:?}): {} {}",
            args.join(" "),
            out.status.code(),
            stdout,
            stderr
        ))
    }
}

#[cfg(windows)]
pub fn install_service(cfg: &ServiceConfig) -> Result<String, String> {
    let cmds = render_windows_sc_commands(cfg)?;
    std::fs::create_dir_all(&cfg.data_dir)
        .map_err(|e| format!("cannot create data dir {}: {e}", cfg.data_dir.display()))?;
    // Stop and delete any prior service registration with the same name so reinstalls are idempotent.
    let _ = Command::new("sc.exe")
        .args(["stop", &cfg.service_name])
        .output();
    let _ = Command::new("sc.exe")
        .args(["delete", &cfg.service_name])
        .output();
    for args in &cmds {
        run_sc(args)?;
    }
    let _ = run_sc(&["start".to_string(), cfg.service_name.clone()]);
    Ok(format!(
        "installed Windows service '{}' (data dir: {})",
        cfg.service_name,
        cfg.data_dir.display()
    ))
}

#[cfg(windows)]
pub fn uninstall_service(cfg: &ServiceConfig) -> Result<String, String> {
    validate_service_config(cfg)?;
    let _ = Command::new("sc.exe")
        .args(["stop", &cfg.service_name])
        .output();
    run_sc(&["delete".to_string(), cfg.service_name.clone()])?;
    Ok(format!("removed Windows service '{}'", cfg.service_name))
}

#[cfg(windows)]
pub fn service_status(cfg: &ServiceConfig) -> Result<ServiceStatusReport, String> {
    validate_service_config(cfg)?;
    let out = Command::new("sc.exe")
        .args(["query", &cfg.service_name])
        .output()
        .map_err(|e| format!("failed to run sc.exe query: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let installed = out.status.success();
    let running = installed && text.contains("RUNNING");
    let state = if !installed {
        "not-installed".to_string()
    } else if running {
        "running".to_string()
    } else if text.contains("STOPPED") {
        "stopped".to_string()
    } else {
        "installed".to_string()
    };
    Ok(ServiceStatusReport {
        service_name: cfg.service_name.clone(),
        platform: "windows",
        installed,
        running,
        state,
        unit_path: None,
        detail: text.trim().to_string(),
    })
}

#[cfg(windows)]
mod win_scm {
    use super::{run_headless_server, ServiceConfig};
    use std::ffi::c_void;
    use std::sync::{Arc, Condvar, Mutex, OnceLock};

    const SERVICE_WIN32_OWN_PROCESS: u32 = 0x0000_0010;
    const SERVICE_STOPPED: u32 = 0x0000_0001;
    const SERVICE_START_PENDING: u32 = 0x0000_0002;
    const SERVICE_STOP_PENDING: u32 = 0x0000_0003;
    const SERVICE_RUNNING: u32 = 0x0000_0004;
    const SERVICE_ACCEPT_STOP: u32 = 0x0000_0001;
    const SERVICE_ACCEPT_SHUTDOWN: u32 = 0x0000_0004;
    const SERVICE_CONTROL_STOP: u32 = 0x0000_0001;
    const SERVICE_CONTROL_INTERROGATE: u32 = 0x0000_0004;
    const SERVICE_CONTROL_SHUTDOWN: u32 = 0x0000_0005;
    const ERROR_FAILED_SERVICE_CONTROLLER_CONNECT: u32 = 1063;
    const NO_ERROR: u32 = 0;
    const ERROR_CALL_NOT_IMPLEMENTED: u32 = 120;

    type ServiceStatusHandle = isize;
    type ServiceMainFn = unsafe extern "system" fn(u32, *mut *mut u16);
    type HandlerExFn = unsafe extern "system" fn(u32, u32, *mut c_void, *mut c_void) -> u32;

    #[repr(C)]
    struct ServiceTableEntryW {
        lp_service_name: *const u16,
        lp_service_proc: Option<ServiceMainFn>,
    }

    #[repr(C)]
    struct ServiceStatus {
        dw_service_type: u32,
        dw_current_state: u32,
        dw_controls_accepted: u32,
        dw_win32_exit_code: u32,
        dw_service_specific_exit_code: u32,
        dw_check_point: u32,
        dw_wait_hint: u32,
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn StartServiceCtrlDispatcherW(lp_service_start_table: *const ServiceTableEntryW) -> i32;
        fn RegisterServiceCtrlHandlerExW(
            lp_service_name: *const u16,
            lp_handler_proc: HandlerExFn,
            lp_context: *mut c_void,
        ) -> ServiceStatusHandle;
        fn SetServiceStatus(
            h_service_status: ServiceStatusHandle,
            lp_service_status: *const ServiceStatus,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLastError() -> u32;
    }

    struct ScmContext {
        cfg: ServiceConfig,
        stop_signal: Arc<(Mutex<bool>, Condvar)>,
        status_handle: Mutex<ServiceStatusHandle>,
    }

    static SCM_CTX: OnceLock<ScmContext> = OnceLock::new();

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn report_status(
        handle: ServiceStatusHandle,
        state: u32,
        accepted: u32,
        exit_code: u32,
        wait_hint: u32,
    ) {
        if handle == 0 {
            return;
        }
        let status = ServiceStatus {
            dw_service_type: SERVICE_WIN32_OWN_PROCESS,
            dw_current_state: state,
            dw_controls_accepted: accepted,
            dw_win32_exit_code: exit_code,
            dw_service_specific_exit_code: 0,
            dw_check_point: 0,
            dw_wait_hint: wait_hint,
        };
        unsafe {
            let _ = SetServiceStatus(handle, &status);
        }
    }

    unsafe extern "system" fn ctrl_handler(
        control: u32,
        _event_type: u32,
        _event_data: *mut c_void,
        _context: *mut c_void,
    ) -> u32 {
        match control {
            SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
                if let Some(ctx) = SCM_CTX.get() {
                    let handle = *ctx.status_handle.lock().unwrap();
                    report_status(handle, SERVICE_STOP_PENDING, 0, NO_ERROR, 5_000);
                    let (lock, cvar) = &*ctx.stop_signal;
                    let mut stopped = lock.lock().unwrap();
                    *stopped = true;
                    cvar.notify_all();
                }
                NO_ERROR
            }
            SERVICE_CONTROL_INTERROGATE => NO_ERROR,
            _ => ERROR_CALL_NOT_IMPLEMENTED,
        }
    }

    unsafe extern "system" fn service_main(_argc: u32, _argv: *mut *mut u16) {
        let Some(ctx) = SCM_CTX.get() else {
            return;
        };
        let wide_name = to_wide_null(&ctx.cfg.service_name);
        let handle = unsafe {
            RegisterServiceCtrlHandlerExW(wide_name.as_ptr(), ctrl_handler, std::ptr::null_mut())
        };
        if handle == 0 {
            return;
        }
        *ctx.status_handle.lock().unwrap() = handle;
        report_status(handle, SERVICE_START_PENDING, 0, NO_ERROR, 10_000);

        let cfg_clone = ctx.cfg.clone();
        let stop_clone = ctx.stop_signal.clone();
        std::thread::spawn(move || {
            if let Err(e) = run_headless_server(&cfg_clone) {
                eprintln!("ergon service error: {e}");
                let (lock, cvar) = &*stop_clone;
                let mut stopped = lock.lock().unwrap();
                *stopped = true;
                cvar.notify_all();
            }
        });

        report_status(
            handle,
            SERVICE_RUNNING,
            SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN,
            NO_ERROR,
            0,
        );

        let (lock, cvar) = &*ctx.stop_signal;
        let mut stopped = lock.lock().unwrap();
        while !*stopped {
            stopped = cvar.wait(stopped).unwrap();
        }
        report_status(handle, SERVICE_STOPPED, 0, NO_ERROR, 0);
        std::process::exit(0);
    }

    pub fn dispatch_or_run(cfg: &ServiceConfig) -> Result<(), String> {
        let _ = SCM_CTX.set(ScmContext {
            cfg: cfg.clone(),
            stop_signal: Arc::new((Mutex::new(false), Condvar::new())),
            status_handle: Mutex::new(0),
        });
        let wide_name = to_wide_null(&cfg.service_name);
        let table = [
            ServiceTableEntryW {
                lp_service_name: wide_name.as_ptr(),
                lp_service_proc: Some(service_main),
            },
            ServiceTableEntryW {
                lp_service_name: std::ptr::null(),
                lp_service_proc: None,
            },
        ];
        let ok = unsafe { StartServiceCtrlDispatcherW(table.as_ptr()) };
        if ok != 0 {
            return Ok(());
        }
        let err = unsafe { GetLastError() };
        if err == ERROR_FAILED_SERVICE_CONTROLLER_CONNECT {
            // Invoked from an interactive terminal rather than Windows SCM: run headless directly.
            return run_headless_server(cfg);
        }
        Err(format!("StartServiceCtrlDispatcherW failed with Win32 error {err}"))
    }
}

#[cfg(windows)]
pub fn run_service(cfg: &ServiceConfig) -> Result<(), String> {
    win_scm::dispatch_or_run(cfg)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn install_service(_cfg: &ServiceConfig) -> Result<String, String> {
    Err("service installation is supported on Linux, macOS, and Windows".to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn uninstall_service(_cfg: &ServiceConfig) -> Result<String, String> {
    Err("service uninstallation is supported on Linux, macOS, and Windows".to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn service_status(cfg: &ServiceConfig) -> Result<ServiceStatusReport, String> {
    Ok(ServiceStatusReport {
        service_name: cfg.service_name.clone(),
        platform: "other",
        installed: false,
        running: false,
        state: "unsupported".to_string(),
        unit_path: None,
        detail: "unsupported OS".to_string(),
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub fn run_service(cfg: &ServiceConfig) -> Result<(), String> {
    run_headless_server(cfg)
}

/// Checks if a path looks clean for service registration (used by tests & CLI).
pub fn is_valid_service_path(p: &Path) -> bool {
    let s = p.to_string_lossy();
    !s.trim().is_empty() && !has_control_chars(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cfg(name: &str, exe: &str, data: &str, system_wide: bool) -> ServiceConfig {
        ServiceConfig {
            service_name: name.to_string(),
            display_name: "Ergon Personal Agent Server".to_string(),
            description: "Ergon — an AI agent for every employee".to_string(),
            exe_path: PathBuf::from(exe),
            data_dir: PathBuf::from(data),
            host: "127.0.0.1".to_string(),
            port: 8787,
            system_wide,
        }
    }

    #[test]
    fn systemd_unit_renders_valid_user_and_system_units() {
        let user_cfg = sample_cfg("ergon", "/usr/local/bin/ergon", "/var/tmp/ergon-data", false);
        let unit = render_systemd_unit(&user_cfg).unwrap();
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("ExecStart=\"/usr/local/bin/ergon\" service run"));
        assert!(unit.contains("ERGON_DATA_DIR=/var/tmp/ergon-data"));
        assert!(unit.contains("ERGON_NO_OPEN=1"));
        assert!(unit.contains("Restart=on-failure"));
        assert!(unit.contains("WantedBy=default.target"));

        let sys_cfg = sample_cfg("ergon", "/usr/local/bin/ergon", "/var/lib/ergon", true);
        let sys_unit = render_systemd_unit(&sys_cfg).unwrap();
        assert!(sys_unit.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn launchd_plist_renders_and_escapes_xml_safely() {
        let cfg = sample_cfg(
            "com.ergon.agent",
            "/Applications/Ergon.app/Contents/MacOS/ergon",
            "/Library/Application Support/Ergon & Team",
            true,
        );
        let plist = render_launchd_plist(&cfg).unwrap();
        assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(plist.contains("<string>com.ergon.agent</string>"));
        assert!(plist.contains("Ergon &amp; Team"), "XML ampersand must be escaped");
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<key>ERGON_NO_OPEN</key>"));
    }

    #[test]
    fn windows_sc_commands_include_auto_start_and_failure_recovery() {
        let cfg = sample_cfg(
            "ErgonService",
            r"C:\Program Files\Ergon\ergon.exe",
            r"C:\ProgramData\Ergon\data",
            true,
        );
        let cmds = render_windows_sc_commands(&cfg).unwrap();
        assert_eq!(cmds.len(), 4);
        assert_eq!(cmds[0][0], "create");
        assert_eq!(cmds[0][1], "ErgonService");
        assert!(cmds[0][3].contains("service run --name ErgonService"));
        assert!(cmds[0].contains(&"auto".to_string()));
        assert_eq!(cmds[2][0], "failure");
        assert!(cmds[2].iter().any(|a| a.contains("restart/5000")));
    }

    #[test]
    fn service_validation_rejects_bad_names_ports_and_control_chars() {
        let mut bad = sample_cfg("bad name; rm -rf /", "/bin/ergon", "/var/lib/ergon", false);
        assert!(validate_service_config(&bad).is_err());

        bad.service_name = "ergon".to_string();
        bad.port = 0;
        assert!(validate_service_config(&bad).is_err());

        bad.port = 8787;
        bad.host = "127.0.0.1\nExecStart=/evil".to_string();
        assert!(validate_service_config(&bad).is_err());
    }
}
