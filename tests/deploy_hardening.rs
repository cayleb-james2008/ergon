use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

fn test_root(tag: &str) -> PathBuf {
    let n = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "ergon-deploy-hardening-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn http_raw(port: u16, method: &str, path: &str, body: Option<&str>) -> std::io::Result<(u16, String, String)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes())?;
    stream.flush()?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    let status: u16 = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let split = raw.find("\r\n\r\n").unwrap_or(raw.len());
    let head = raw[..split].to_string();
    let resp_body = if split + 4 <= raw.len() {
        raw[split + 4..].to_string()
    } else {
        String::new()
    };
    Ok((status, head, resp_body))
}

#[test]
fn server_session_cookies_are_high_entropy_and_unique() {
    let root = test_root("session-entropy");
    let cfg = ergon_rs::Config {
        data_dir: root.join("data"),
        workspace_dir: root.join("data/workspace"),
        memory_path: root.join("data/memory.md"),
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "test-model".to_string(),
        api_key: None,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let _ = ergon_rs::serve(listener, cfg);
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok((200, _, _)) = http_raw(port, "GET", "/api/status", None) {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    let mut payloads = HashSet::new();
    for _ in 0..16 {
        let (status, head, _) = http_raw(port, "GET", "/", None).unwrap();
        assert_eq!(status, 200);
        let cookie_val = head
            .lines()
            .find_map(|line| {
                let low = line.to_ascii_lowercase();
                if low.starts_with("set-cookie:") {
                    let rest = line.split_once(':')?.1.trim();
                    let first = rest.split(';').next()?.trim();
                    first.strip_prefix("ergon_session=").map(str::to_string)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("missing ergon_session Set-Cookie header: {head}"));
        assert!(cookie_val.starts_with('s'), "unexpected cookie format: {cookie_val}");
        let (hex, _) = cookie_val[1..]
            .split_once('-')
            .unwrap_or_else(|| panic!("missing counter suffix: {cookie_val}"));
        assert_eq!(
            hex.len(),
            32,
            "session cookie must carry 128-bit CSPRNG hex payload: {cookie_val}"
        );
        assert!(
            hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
            "session cookie hex must be lowercase hex: {cookie_val}"
        );
        assert!(
            payloads.insert(hex.to_string()),
            "duplicate session cookie entropy payload: {cookie_val}"
        );
    }
}

#[cfg(unix)]
mod unix_deploy_tests {
    use super::*;
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader};
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::sync::mpsc;

    fn free_loopback_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    struct ChildGuard(Option<Child>);

    impl ChildGuard {
        fn new(child: Child) -> Self {
            Self(Some(child))
        }

        fn id(&self) -> u32 {
            self.0.as_ref().expect("child already taken").id()
        }

        fn wait_timeout(&mut self, timeout: Duration) -> Option<ExitStatus> {
            let child = self.0.as_mut()?;
            let deadline = Instant::now() + timeout;
            while Instant::now() < deadline {
                if let Ok(Some(status)) = child.try_wait() {
                    self.0 = None;
                    return Some(status);
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            None
        }
    }

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn send_sigterm(pid: u32) {
        let status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .expect("failed to invoke kill -TERM");
        assert!(status.success(), "kill -TERM {pid} failed: {status}");
    }

    fn wait_for_ready(port: u16, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Ok((200, _, _)) = http_raw(port, "GET", "/api/status", None) {
                return;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        panic!("ergon binary failed to become ready on port {port} within {timeout:?}");
    }

    fn read_stub_request(stream: &TcpStream) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut content_length = 0usize;
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).unwrap_or(0);
            if n == 0 || line == "\r\n" || line == "\n" {
                break;
            }
            let lower = line.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
        if content_length > 0 {
            let mut body = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body);
        }
    }

    #[test]
    fn sigterm_exits_cleanly_with_status_zero_when_idle() {
        let root = test_root("sigterm-idle");
        let port = free_loopback_port();
        let child = Command::new(env!("CARGO_BIN_EXE_ergon"))
            .args(["--no-open", "--host", "127.0.0.1", "--port", &port.to_string()])
            .env("ERGON_DATA_DIR", root.join("data"))
            .env("ERGON_NO_AUTO_MODEL", "1")
            .env("ERGON_DISABLE_SCHEDULER", "1")
            .env("ERGON_UPDATE_MODE", "off")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ergon binary");
        let mut guard = ChildGuard::new(child);

        wait_for_ready(port, Duration::from_secs(10));

        let start = Instant::now();
        send_sigterm(guard.id());
        let status = guard
            .wait_timeout(Duration::from_secs(5))
            .expect("ergon did not exit within 5 seconds of SIGTERM");
        let elapsed = start.elapsed();

        assert_eq!(
            status.code(),
            Some(0),
            "ergon must exit cleanly with status 0 on SIGTERM, got {status:?} after {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "idle SIGTERM shutdown took too long: {elapsed:?}"
        );
    }

    #[test]
    fn sigterm_drains_inflight_chat_request_and_exits_zero() {
        let root = test_root("sigterm-drain");
        let data_dir = root.join("data");
        let port = free_loopback_port();

        // Start a stub OpenAI server that signals when the request enters
        // flight and holds it open for 600ms so SIGTERM arrives mid-request.
        let stub_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stub_port = stub_listener.local_addr().unwrap().port();
        let (entered_tx, entered_rx) = mpsc::channel::<()>();
        let summary_text = "Drained cleanly during graceful shutdown.";
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = stub_listener.accept() {
                read_stub_request(&stream);
                let _ = entered_tx.send(());
                std::thread::sleep(Duration::from_millis(600));
                let inner = json!({"done": true, "summary": summary_text}).to_string();
                let payload = json!({
                    "id": "chatcmpl-drain",
                    "object": "chat.completion",
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": inner},
                        "finish_reason": "stop"
                    }]
                })
                .to_string();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });

        let child = Command::new(env!("CARGO_BIN_EXE_ergon"))
            .args(["--no-open", "--host", "127.0.0.1", "--port", &port.to_string()])
            .env("ERGON_DATA_DIR", &data_dir)
            .env("ERGON_MODEL_BASE_URL", format!("http://127.0.0.1:{stub_port}/v1"))
            .env("ERGON_MODEL_API_KEY", "test-drain-key")
            .env("ERGON_SHUTDOWN_GRACE_SECS", "10")
            .env("ERGON_DISABLE_SCHEDULER", "1")
            .env("ERGON_UPDATE_MODE", "off")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ergon binary");
        let mut guard = ChildGuard::new(child);

        wait_for_ready(port, Duration::from_secs(10));

        // Issue a chat request in flight on a client thread.
        let chat_handle = std::thread::spawn(move || {
            http_raw(
                port,
                "POST",
                "/api/chat",
                Some(r#"{"message":"Confirm graceful shutdown drain"}"#),
            )
        });

        // Wait until the stub confirms ergon is mid-run (RUNS_IN_FLIGHT >= 1).
        entered_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("stub model never received in-flight chat request");

        let term_sent = Instant::now();
        send_sigterm(guard.id());

        // Accept loop stops promptly once SIGTERM arrives, refusing new connections.
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            TcpStream::connect(("127.0.0.1", port)).is_err(),
            "listener must stop accepting new connections once SIGTERM arrives"
        );

        // The already-accepted in-flight chat request must finish cleanly with 200 OK.
        let (status, head, body) = chat_handle
            .join()
            .expect("chat client thread panicked")
            .expect("in-flight HTTP request was dropped instead of drained");
        assert_eq!(status, 200, "in-flight chat must complete 200 OK: {body}");
        assert!(
            head.to_ascii_lowercase().contains("connection: close"),
            "draining response should instruct Connection: close: {head}"
        );
        assert!(
            body.contains(summary_text),
            "drained chat response must include completed summary: {body}"
        );

        // Process must exit cleanly with status 0 within 5s of SIGTERM.
        let exit_status = guard
            .wait_timeout(Duration::from_secs(5))
            .expect("ergon did not exit within 5 seconds after draining");
        let elapsed = term_sent.elapsed();
        assert_eq!(
            exit_status.code(),
            Some(0),
            "ergon must exit 0 after draining in-flight work, got {exit_status:?} after {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "drained SIGTERM shutdown exceeded 5s budget: {elapsed:?}"
        );

        // Verify persisted thread JSON on disk was written cleanly without truncation.
        let threads_raw = std::fs::read_to_string(data_dir.join("threads.json"))
            .expect("threads.json must exist after drained chat");
        let threads_json: Value =
            serde_json::from_str(&threads_raw).expect("threads.json must be valid JSON");
        assert!(
            threads_json.to_string().contains(summary_text),
            "persisted threads.json must preserve drained chat summary"
        );
    }

    #[test]
    fn install_sh_creates_0600_env_0700_launcher_and_preserves_secrets() {
        let root = test_root("install-hardening");
        let pkg = root.join("pkg");
        let home = root.join("home");
        let prefix = root.join("prefix");
        let data = root.join("data");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::create_dir_all(&home).unwrap();

        let install_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install.sh");
        let syntax = Command::new("bash")
            .args(["-n", install_src.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            syntax.status.success(),
            "bash -n install.sh failed: {}",
            String::from_utf8_lossy(&syntax.stderr)
        );

        std::fs::copy(env!("CARGO_BIN_EXE_ergon"), pkg.join("ergon")).unwrap();
        std::fs::copy(&install_src, pkg.join("install.sh")).unwrap();

        let out = Command::new("bash")
            .arg(pkg.join("install.sh"))
            .args([
                "--prefix",
                prefix.to_str().unwrap(),
                "--data",
                data.to_str().unwrap(),
                "--no-service",
                // This test probes the launcher over plain HTTP; TLS defaults are
                // covered by tests/auth_tls_rbac.rs.
                "--no-tls",
            ])
            .env("HOME", &home)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "install.sh failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);

        let env_file = data.join("ergon.env");
        let launcher = prefix.join("bin/ergon-serve");
        assert!(env_file.exists(), "ergon.env must be created in DATA_DIR");
        assert!(launcher.exists(), "ergon-serve launcher must be created");

        let env_mode = std::fs::metadata(&env_file).unwrap().permissions().mode() & 0o777;
        let launcher_mode = std::fs::metadata(&launcher).unwrap().permissions().mode() & 0o777;
        assert_eq!(env_mode, 0o600, "ergon.env must be mode 0600, got {env_mode:o}");
        assert_eq!(
            launcher_mode, 0o700,
            "ergon-serve must be mode 0700, got {launcher_mode:o}"
        );

        let launcher_src = std::fs::read_to_string(&launcher).unwrap();
        assert!(
            !launcher_src.contains("export ERGON_MODEL_API_KEY"),
            "ergon-serve must not embed ERGON_MODEL_API_KEY instructions: {launcher_src}"
        );
        assert!(
            launcher_src.contains("ergon.env"),
            "ergon-serve must source ergon.env: {launcher_src}"
        );
        assert!(
            stdout.contains(&env_file.display().to_string()),
            "Next steps must guide users to edit ergon.env: {stdout}"
        );
        assert!(
            !stdout.contains("edit ERGON_MODEL_API_KEY into"),
            "Next steps must not instruct placing secrets inside ergon-serve: {stdout}"
        );

        // Write a test API key to ergon.env and re-run install.sh to verify
        // existing secrets are preserved and permissions stay 0600.
        std::fs::write(&env_file, "ERGON_MODEL_API_KEY=test-secret-from-env-file\n").unwrap();
        let out2 = Command::new("bash")
            .arg(pkg.join("install.sh"))
            .args([
                "--prefix",
                prefix.to_str().unwrap(),
                "--data",
                data.to_str().unwrap(),
                "--no-service",
                // This test probes the launcher over plain HTTP; TLS defaults are
                // covered by tests/auth_tls_rbac.rs.
                "--no-tls",
            ])
            .env("HOME", &home)
            .output()
            .unwrap();
        assert!(out2.status.success(), "re-running install.sh failed");
        let env_after = std::fs::read_to_string(&env_file).unwrap();
        assert!(
            env_after.contains("ERGON_MODEL_API_KEY=test-secret-from-env-file"),
            "re-running install.sh must preserve existing ergon.env secrets"
        );
        let env_mode2 = std::fs::metadata(&env_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(env_mode2, 0o600, "ergon.env must stay 0600 after reinstall");

        // Launch via ergon-serve and confirm it sources ergon.env and exits 0 on SIGTERM.
        let port = free_loopback_port();
        let child = Command::new(&launcher)
            .env("ERGON_PORT", port.to_string())
            .env("ERGON_NO_OPEN", "1")
            .env("ERGON_DISABLE_SCHEDULER", "1")
            .env("ERGON_UPDATE_MODE", "off")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ergon-serve");
        let mut guard = ChildGuard::new(child);

        wait_for_ready(port, Duration::from_secs(10));
        let (st, _, body) = http_raw(port, "GET", "/api/status", None).unwrap();
        assert_eq!(st, 200);
        let status_json: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            status_json["model_configured"],
            json!(true),
            "ergon-serve must source ERGON_MODEL_API_KEY from ergon.env: {body}"
        );

        send_sigterm(guard.id());
        let exit = guard
            .wait_timeout(Duration::from_secs(5))
            .expect("ergon-serve did not exit cleanly on SIGTERM");
        assert_eq!(exit.code(), Some(0));
    }

    #[test]
    fn install_sh_systemd_unit_uses_environment_file_and_checks_linger() {
        // Only run when systemctl exists on the host (e.g. Linux with systemd).
        if Command::new("systemctl").arg("--version").output().is_err() {
            return;
        }
        let root = test_root("install-systemd");
        let pkg = root.join("pkg");
        let home = root.join("home");
        let prefix = root.join("prefix");
        let data = root.join("data dir");
        let fake_xdg = root.join("no-bus");
        let fake_bin = root.join("fake-bin");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&fake_xdg).unwrap();
        std::fs::create_dir_all(&fake_bin).unwrap();

        let install_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install.sh");
        std::fs::copy(env!("CARGO_BIN_EXE_ergon"), pkg.join("ergon")).unwrap();
        std::fs::copy(&install_src, pkg.join("install.sh")).unwrap();

        // Provide a stub loginctl returning Linger=no so the linger warning path
        // is verified deterministically regardless of the host user's setting.
        let stub_loginctl = fake_bin.join("loginctl");
        std::fs::write(&stub_loginctl, "#!/usr/bin/env bash\necho 'Linger=no'\n").unwrap();
        std::fs::set_permissions(&stub_loginctl, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path_env = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );

        let out = Command::new("bash")
            .arg(pkg.join("install.sh"))
            .args([
                "--prefix",
                prefix.to_str().unwrap(),
                "--data",
                data.to_str().unwrap(),
            ])
            .env("HOME", &home)
            .env("XDG_RUNTIME_DIR", &fake_xdg)
            .env("DBUS_SESSION_BUS_ADDRESS", "")
            .env("PATH", &path_env)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "install.sh systemd run failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        let unit_path = home.join(".config/systemd/user/ergon.service");
        assert!(unit_path.exists(), "systemd user unit must be written");
        let unit = std::fs::read_to_string(&unit_path).unwrap();
        assert!(
            unit.contains(&format!("EnvironmentFile=-{}/ergon.env", data.display())),
            "unit must reference EnvironmentFile=-$DATA_DIR/ergon.env: {unit}"
        );
        assert!(
            unit.contains(&format!("ExecStart=\"{}/bin/ergon-serve\"", prefix.display())),
            "unit must quote ExecStart: {unit}"
        );
        assert!(
            !unit.contains("Environment=ERGON_MODEL_API_KEY"),
            "unit must not instruct placing raw keys inline: {unit}"
        );
        assert!(
            stdout.contains("loginctl enable-linger"),
            "installer must warn with loginctl enable-linger guidance when Linger!=yes: {stdout}"
        );
    }
}
