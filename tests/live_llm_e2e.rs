//! Env-gated real Live LLM End-to-End test.
//!
//! Reads `ERGON_LIVE_LLM_API_KEY` (and optional `ERGON_LIVE_LLM_BASE_URL` /
//! `ERGON_LIVE_LLM_MODEL`), or falls back to `ERGON_MODEL_API_KEY` when
//! `ERGON_RUN_LIVE_LLM_TEST=1` is explicitly set.
//!
//! - When no live key is configured in the environment: prints a clear
//!   diagnostic skip message to stderr and returns cleanly (never failing CI
//!   and never printing any key).
//! - When a live key IS configured: boots a real `ergon` server against a fresh
//!   isolated `DATA_DIR`, calls `POST /api/chat` with a real task prompt that
//!   writes a workspace file, verifies the file exists with real content,
//!   verifies the SHA-256 receipt chain and persisted `live_runs.json`, and
//!   NEVER logs or prints any API key.

use ergon_rs::{live_runs, serve, Config, DEFAULT_MODEL_BASE_URL, DEFAULT_MODEL_ID};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct TempRunDir {
    root: PathBuf,
    data_dir: PathBuf,
    workspace_dir: PathBuf,
    memory_path: PathBuf,
}

impl TempRunDir {
    fn new() -> Self {
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let root = base
            .join("test-data")
            .join(format!("live-llm-e2e-{}-{ts}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        Self {
            root,
            data_dir,
            workspace_dir,
            memory_path,
        }
    }
}

impl Drop for TempRunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn env_opted_in(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn resolve_live_llm_env() -> Option<(String, String, String)> {
    let api_key = non_empty_env("ERGON_LIVE_LLM_API_KEY").or_else(|| {
        if env_opted_in("ERGON_RUN_LIVE_LLM_TEST") {
            non_empty_env("ERGON_MODEL_API_KEY")
        } else {
            None
        }
    })?;
    let base_url = non_empty_env("ERGON_LIVE_LLM_BASE_URL")
        .or_else(|| non_empty_env("ERGON_MODEL_BASE_URL"))
        .unwrap_or_else(|| DEFAULT_MODEL_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string();
    let model_id = non_empty_env("ERGON_LIVE_LLM_MODEL")
        .or_else(|| non_empty_env("ERGON_MODEL_ID"))
        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string());
    Some((api_key, base_url, model_id))
}

fn redact_secret(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        return text.to_string();
    }
    text.replace(secret, "[REDACTED]")
}

fn http_json(addr: SocketAddr, method: &str, path: &str, body: &str, timeout: Duration) -> (u16, String, Value) {
    let mut stream = TcpStream::connect(addr).expect("connect to local ergon server");
    stream.set_read_timeout(Some(timeout)).unwrap();
    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n",
        addr.port()
    );
    if !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(head.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body.as_bytes()).unwrap();
    }
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP response separator");
    let head_str = String::from_utf8_lossy(&raw[..sep]);
    let status = head_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_str = String::from_utf8_lossy(&raw[sep + 4..]).into_owned();
    let val = serde_json::from_str(&body_str).unwrap_or(Value::Null);
    (status, body_str, val)
}

#[test]
fn live_llm_chat_writes_workspace_file_and_verifies_receipt() {
    let Some((api_key, base_url, model_id)) = resolve_live_llm_env() else {
        eprintln!(
            "SKIP live_llm_e2e: set ERGON_LIVE_LLM_API_KEY (and optional ERGON_LIVE_LLM_BASE_URL / ERGON_LIVE_LLM_MODEL), or ERGON_RUN_LIVE_LLM_TEST=1 with ERGON_MODEL_API_KEY, to run live model E2E"
        );
        return;
    };

    let tmp = TempRunDir::new();
    let cfg = Config {
        data_dir: tmp.data_dir.clone(),
        workspace_dir: tmp.workspace_dir.clone(),
        memory_path: tmp.memory_path.clone(),
        model_base_url: base_url.clone(),
        model_id: model_id.clone(),
        api_key: Some(api_key.clone()),
    };
    cfg.ensure_dirs().unwrap();

    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let cfg_srv = cfg.clone();
    std::thread::spawn(move || {
        let _ = serve(listener, cfg_srv);
    });

    // Wait for server readiness
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    let target_file = "live-llm-e2e-note.md";
    let prompt = format!(
        "Use the write_file tool to create '{target_file}' with heading '# Live Model Verification' and three concise bullet points about atomic crash recovery. After writing the file, reply done with a one-sentence summary."
    );

    let (status, raw_body, resp) = http_json(
        addr,
        "POST",
        "/api/chat",
        &json!({ "message": prompt }).to_string(),
        Duration::from_secs(180),
    );
    assert_eq!(
        status,
        200,
        "POST /api/chat against {base_url} ({model_id}) failed with HTTP {status}: {}",
        redact_secret(&raw_body, &api_key)
    );
    assert!(
        !raw_body.contains(&api_key),
        "chat response must never leak API key"
    );

    // 1. Verify workspace file exists with real non-empty content (and no leaked key).
    let out_path = tmp.workspace_dir.join(target_file);
    assert!(
        out_path.exists(),
        "expected live LLM run to create {target_file}; response summary: {}",
        redact_secret(&resp.to_string(), &api_key)
    );
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.trim().len() >= 30,
        "expected substantial real content in {target_file}, got {} bytes",
        content.len()
    );
    assert!(
        !content.contains(&api_key),
        "workspace output must never contain API key"
    );

    // 2. Verify SHA-256 receipt chain recorded and validates cleanly.
    let (st_verify, verify_raw, verify_v) =
        http_json(addr, "GET", "/api/receipts/verify", "", Duration::from_secs(15));
    assert_eq!(
        st_verify,
        200,
        "receipt verify failed: {}",
        redact_secret(&verify_raw, &api_key)
    );
    assert_eq!(verify_v["valid"], json!(true));
    assert!(
        verify_v["total"].as_u64().unwrap_or(0) >= 1,
        "expected >= 1 receipt after live LLM task"
    );

    // 3. Verify persisted `live_runs.json` recorded clean completion.
    let runs = live_runs::load_all(&tmp.data_dir);
    assert!(
        runs.iter().any(|r| r.status == "completed"),
        "expected completed run in live_runs.json"
    );
}
