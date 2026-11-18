//! Integration proofs for Ergon.
//!
//! 1. A stub model server (test fixture only — std TcpListener in this
//!    file, clearly separated from product code) speaks the
//!    OpenAI-compatible chat-completions shape with scripted JSON
//!    replies. The agent talks to it through the REAL ureq HTTP code
//!    path, with ERGON_MODEL_BASE_URL-equivalent config pointed at it.
//! 2. End-to-end: messy notes -> organised summary document, via
//!    list_dir -> read_file x2 -> write_document (Rune) ->
//!    update_memory (Rune) -> done. No human step in between.
//! 3. No-key honesty: /api/chat refuses, nothing is fabricated.
//! 4. Shell: GET / carries the sidebar / thread / composer markers.

use ergon_rs::Config;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-data")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn test_config(root: &PathBuf, api_key: Option<&str>, model_base: String) -> Config {
    let data_dir = root.join("data");
    let workspace_dir = data_dir.join("workspace");
    std::fs::create_dir_all(&workspace_dir).unwrap();
    std::fs::write(data_dir.join("memory.md"), "# Memory\n\n").unwrap();
    Config {
        data_dir: data_dir.clone(),
        workspace_dir,
        memory_path: data_dir.join("memory.md"),
        model_base_url: model_base,
        model_id: "deepseek-v4.1-flash".to_string(),
        api_key: api_key.map(|k| k.to_string()),
    }
}

fn start_app(cfg: Config) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let _ = ergon_rs::serve(listener, cfg);
    });
    port
}

/// Minimal raw HTTP client so tests see exact status codes (ureq would
/// turn 4xx/5xx into errors and hide the honesty-test response).
fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let status: u16 = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body_start = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(raw.len());
    (status, raw[body_start..].to_string())
}

// ---------------------------------------------------------------------------
// Stub model server (TEST FIXTURE ONLY — not product code)
// ---------------------------------------------------------------------------

struct StubModel {
    port: u16,
    raw_requests: Arc<Mutex<Vec<String>>>,
}

fn read_http_request(stream: &TcpStream) -> (String, String) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut head = String::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
        head.push_str(&line);
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).unwrap();
    }
    (head, String::from_utf8_lossy(&body).into_owned())
}

fn start_stub_model(replies: Vec<String>) -> StubModel {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let raw_requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = raw_requests.clone();
    std::thread::spawn(move || {
        let mut idx = 0usize;
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let (head, body) = read_http_request(&stream);
            recorded
                .lock()
                .unwrap()
                .push(format!("{head}\r\n{body}"));
            let reply = replies
                .get(idx)
                .cloned()
                .unwrap_or_else(|| "{\"done\": true, \"summary\": \"stub exhausted\"}".to_string());
            idx += 1;
            let payload = json!({
                "id": "chatcmpl-stub",
                "object": "chat.completion",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": reply},
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
    StubModel { port, raw_requests }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn shell_has_sidebar_thread_and_composer() {
    let root = test_root("shell");
    let cfg = test_config(&root, None, "http://127.0.0.1:9/v1".to_string());
    let port = start_app(cfg);
    let (status, body) = http(port, "GET", "/", None);
    assert_eq!(status, 200);
    assert!(body.contains("id=\"sidebar\""), "sidebar marker missing");
    assert!(body.contains("id=\"thread-list\""), "thread list missing");
    assert!(body.contains("New chat"), "new chat button missing");
    assert!(body.contains("id=\"thread\""), "thread marker missing");
    assert!(body.contains("id=\"composer\""), "composer marker missing");
    assert!(body.contains("id=\"composer-input\""), "composer input missing");
    assert!(body.contains("id=\"no-key-banner\""), "no-key banner missing");
    assert!(body.contains("No model key configured"), "no-key banner text missing");
}

#[test]
fn no_key_chat_refuses_honestly() {
    let root = test_root("nokey");
    let cfg = test_config(&root, None, "http://127.0.0.1:9/v1".to_string());
    let port = start_app(cfg);

    let (status, body) = http(port, "GET", "/api/status", None);
    assert_eq!(status, 200);
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["model_configured"], json!(false));

    let (status, body) = http(
        port,
        "POST",
        "/api/chat",
        Some(&json!({"thread_id": "", "message": "write me a poem"}).to_string()),
    );
    assert_eq!(status, 503, "no-key chat must refuse, got {status}: {body}");
    assert!(
        body.contains("no model key configured"),
        "error must be honest about the missing key: {body}"
    );
    assert!(
        !body.contains("poem") || body.contains("no model key"),
        "no fabricated answer may be returned: {body}"
    );
}

#[test]
fn end_to_end_messy_notes_become_summary_document() {
    let root = test_root("e2e");
    let replies = vec![
        json!({"tool": "list_dir", "args": {"path": "."}}).to_string(),
        json!({"tool": "read_file", "args": {"path": "notes1.txt"}}).to_string(),
        json!({"tool": "read_file", "args": {"path": "notes2.txt"}}).to_string(),
        json!({
            "tool": "write_document",
            "args": {
                "title": "Summary",
                "sections": [
                    "## Key points\n\n- Budget approved for Q1 launch\n- Launch date is March 3",
                    "## Owners\n\n- Alice owns the design mockups\n- Bob is writing the user guide"
                ]
            }
        })
        .to_string(),
        json!({
            "tool": "update_memory",
            "args": {"note": "Prefers project notes organised into a Summary document with Key points and Owners sections."}
        })
        .to_string(),
        json!({
            "done": true,
            "summary": "Organised your messy notes into Summary.md and noted your preferred format in memory."
        })
        .to_string(),
    ];
    let stub = start_stub_model(replies);
    let cfg = test_config(
        &root,
        Some("test-key"),
        format!("http://127.0.0.1:{}/v1", stub.port),
    );
    // Fixture workspace: messy, scattered notes (with a duplicated point).
    std::fs::write(
        cfg.workspace_dir.join("notes1.txt"),
        "Project kickoff notes\n- Budget approved for Q1 launch\n- Alice owns the design mockups\nRandom thought: order more whiteboard markers\n",
    )
    .unwrap();
    std::fs::write(
        cfg.workspace_dir.join("notes2.txt"),
        "More scattered points\n- Launch date is March 3\n- Bob is writing the user guide\n- Budget approved for Q1 launch (mentioned again)\n",
    )
    .unwrap();

    let port = start_app(cfg.clone());
    let (status, body) = http(port, "POST", "/api/threads", Some("{}"));
    assert_eq!(status, 201, "create thread: {body}");
    let thread: Value = serde_json::from_str(&body).unwrap();
    let thread_id = thread["id"].as_str().unwrap().to_string();

    let (status, body) = http(
        port,
        "POST",
        "/api/chat",
        Some(
            &json!({
                "thread_id": thread_id,
                "message": "organise these messy notes into a clean summary document"
            })
            .to_string(),
        ),
    );
    assert_eq!(status, 200, "chat must succeed end to end: {body}");

    // Transcript shows the steps and lands the final summary.
    for marker in [
        "list_dir",
        "read_file",
        "write_document",
        "update_memory",
        "Organised your messy notes into Summary.md",
    ] {
        assert!(body.contains(marker), "transcript missing step/final {marker}: {body}");
    }

    // The Rune write_document skill produced the finished document with
    // the merged content from both messy files.
    let doc_path = cfg.workspace_dir.join("Summary.md");
    assert!(doc_path.exists(), "Summary.md must exist in the workspace");
    let doc = std::fs::read_to_string(&doc_path).unwrap();
    assert!(doc.contains("# Summary"), "doc title missing: {doc}");
    for fact in [
        "Budget approved for Q1 launch",
        "Launch date is March 3",
        "Alice owns the design mockups",
        "Bob is writing the user guide",
    ] {
        assert!(doc.contains(fact), "merged fact {fact:?} missing from doc: {doc}");
    }

    // The Rune update_memory skill appended the learned note.
    let memory = std::fs::read_to_string(&cfg.memory_path).unwrap();
    assert!(
        memory.contains("Prefers project notes organised into a Summary document"),
        "memory.md must gain the learned note: {memory}"
    );

    // Wire requirements on the real HTTP path to the model: every request
    // carried the session header, a browser-like UA, and the bearer key.
    let recorded = stub.raw_requests.lock().unwrap();
    assert_eq!(recorded.len(), 6, "expected 6 model calls, got {}", recorded.len());
    for req in recorded.iter() {
        let lower = req.to_lowercase();
        assert!(lower.contains("x-opencode-session:"), "session header missing: {req}");
        assert!(lower.contains("user-agent: mozilla"), "browser UA missing: {req}");
        assert!(lower.contains("authorization: bearer test-key"), "bearer key missing");
    }
    // The person's memory was in the model's context from the start.
    assert!(
        recorded[0].contains("# Memory"),
        "first model call must include the memory file: {}",
        recorded[0]
    );
}

#[test]
fn fetch_url_extracts_text_and_workspace_is_sandboxed() {
    // Tiny fixture page server.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let page_port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let _ = read_http_request(&stream);
            let html = "<html><head><style>.x{color:red}</style><script>var secret=1;</script></head><body><h1>Hello</h1><p>Fetch works here</p></body></html>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
                html.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });

    let root = test_root("fetch");
    let cfg = test_config(&root, Some("k"), "http://127.0.0.1:9/v1".to_string());
    let out = ergon_rs::execute_tool(
        &cfg,
        "fetch_url",
        &json!({"url": format!("http://127.0.0.1:{page_port}/page")}),
    )
    .unwrap();
    assert!(out.contains("Fetch works here"), "extracted text wrong: {out}");
    assert!(!out.contains("var secret"), "script body must be stripped: {out}");
    assert!(!out.contains("color:red"), "style body must be stripped: {out}");

    // Sandbox: escaping the workspace is refused.
    let err = ergon_rs::execute_tool(&cfg, "read_file", &json!({"path": "../escape.txt"}));
    assert!(err.is_err(), "path escape must be refused, got: {err:?}");
    let err = ergon_rs::execute_tool(&cfg, "read_file", &json!({"path": "/etc/passwd"}));
    assert!(err.is_err(), "absolute path must be refused, got: {err:?}");
}

#[test]
fn sandbox_refuses_symlink_escape() {
    let root = test_root("symlink");
    let cfg = test_config(&root, Some("k"), "http://127.0.0.1:9/v1".to_string());
    // A secret OUTSIDE the workspace, and a symlink inside pointing to it.
    let outside = root.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "outside secret").unwrap();
    std::os::unix::fs::symlink(&outside, cfg.workspace_dir.join("escape")).unwrap();

    let err = ergon_rs::execute_tool(&cfg, "read_file", &json!({"path": "escape/secret.txt"}));
    assert!(err.is_err(), "read through a symlink out of the workspace must be refused, got: {err:?}");

    let err = ergon_rs::execute_tool(
        &cfg,
        "write_file",
        &json!({"path": "escape/pwned.txt", "content": "x"}),
    );
    assert!(err.is_err(), "write through a symlink out of the workspace must be refused, got: {err:?}");
    assert!(!outside.join("pwned.txt").exists(), "nothing may be written outside");
}
