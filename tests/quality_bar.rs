//! Wave 3 Step 3e Integration Tests: Per-Task Quality Bar ("Better every time, until good enough").

use ergon_rs::{serve_with_gate, Config};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    root: PathBuf,
    cfg: Config,
}

impl TestEnv {
    fn new(label: &str, model_url: String) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
        let root = base.join("test-data").join(format!(
            "quality-bar-{}-{}-{label}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: model_url,
            model_id: "mock-model".to_string(),
            api_key: Some("sk-mock".to_string()),
        };
        cfg.ensure_dirs().unwrap();
        Self { root, cfg }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn spawn_mock_llm() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let seen_systems = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen_systems.clone();
    let turn = Arc::new(AtomicUsize::new(0));

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let seen = seen_clone.clone();
            let turn = turn.clone();
            std::thread::spawn(move || {
                let mut stream = stream;
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut content_len = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" || line == "\n" {
                        break;
                    }
                    if let Some((k, v)) = line.split_once(':') {
                        if k.trim().eq_ignore_ascii_case("content-length") {
                            content_len = v.trim().parse().unwrap_or(0);
                        }
                    }
                }
                let mut body = vec![0u8; content_len];
                let _ = reader.read_exact(&mut body);
                let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                let messages = parsed["messages"].as_array().cloned().unwrap_or_default();
                let sys = messages
                    .first()
                    .and_then(|m| m["content"].as_str())
                    .unwrap_or("")
                    .to_string();

                let idx = turn.fetch_add(1, Ordering::SeqCst);
                let reply_json = match idx {
                    // Run 1: write initial brief, then finish
                    0 => {
                        seen.lock().unwrap().push(sys);
                        json!({
                            "tool": "write_file",
                            "args": {
                                "path": "brief1.md",
                                "content": "# Weekly Client Brief\nStatus: on track.\n"
                            }
                        })
                    }
                    1 => json!({
                        "done": true,
                        "summary": "Wrote brief1.md with weekly client brief."
                    }),
                    // Run 2: sees BEAT LAST RUN, includes learned criteria + [file:L##] citation
                    2 => {
                        seen.lock().unwrap().push(sys);
                        json!({
                            "tool": "write_file",
                            "args": {
                                "path": "brief2.md",
                                "content": "# Weekly Client Brief\n## Risk Register\n- Supply chain [notes.md:L3]\n## Owner Actions\n- Alice confirms dates.\n"
                            }
                        })
                    }
                    3 => json!({
                        "done": true,
                        "summary": "Wrote brief2.md with Risk Register and Owner Actions."
                    }),
                    // Run 3: sees ACCEPTED — GOOD ENOUGH, DO NOT DRIFT
                    4 => {
                        seen.lock().unwrap().push(sys);
                        json!({
                            "tool": "write_file",
                            "args": {
                                "path": "brief3.md",
                                "content": "# Weekly Client Brief\n## Risk Register\n- Budget [notes.md:L5]\n## Owner Actions\n- Bob confirms rollout.\n"
                            }
                        })
                    }
                    _ => json!({
                        "done": true,
                        "summary": "Wrote brief3.md following accepted recipe."
                    }),
                };

                let resp_body = json!({
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": reply_json.to_string()
                        }
                    }]
                })
                .to_string();
                let http = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    resp_body.len(),
                    resp_body
                );
                let _ = stream.write_all(http.as_bytes());
                let _ = stream.flush();
            });
        }
    });

    (addr, seen_systems)
}

fn spawn_server(cfg: Config) -> SocketAddr {
    std::env::set_var("ERGON_DISABLE_SCHEDULER", "1");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = serve_with_gate(listener, cfg, None);
    });
    for _ in 0..50 {
        if TcpStream::connect(addr).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    addr
}

fn http_req(addr: SocketAddr, method: &str, path: &str, body: Option<&str>) -> (u16, Value, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    let bytes = body.unwrap_or("").as_bytes();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        addr.port()
    );
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).unwrap();
    if !bytes.is_empty() {
        stream.write_all(bytes).unwrap();
    }
    stream.flush().unwrap();
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let (head, body_str) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let json = serde_json::from_str(body_str).unwrap_or(Value::Null);
    (status, json, body_str.to_string())
}

#[test]
fn quality_bar_lifecycle_run1_improve_run2_accept_run3_no_drift() {
    let (llm_addr, seen_systems) = spawn_mock_llm();
    let env = TestEnv::new("lifecycle", format!("http://{llm_addr}/v1"));
    let addr = spawn_server(env.cfg.clone());

    // Run 1
    let (st1, r1, raw1) = http_req(
        addr,
        "POST",
        "/api/chat",
        Some(&json!({ "message": "Draft a weekly client brief for Acme" }).to_string()),
    );
    assert_eq!(st1, 200, "{raw1}");
    assert_eq!(r1["quality_bar"]["task_family"], "weekly-client-brief");
    assert_eq!(r1["quality_bar"]["status"], "improving");
    let score1 = r1["quality_bar"]["score"].as_f64().unwrap();

    // Feedback: improve with criteria
    let (st_fb, fb_json, raw_fb) = http_req(
        addr,
        "POST",
        "/api/quality-bars/weekly-client-brief/feedback",
        Some(
            &json!({
                "action": "improve",
                "note": "Include Risk Register and Owner Actions",
                "add_criteria": ["Risk Register", "Owner Actions"]
            })
            .to_string(),
        ),
    );
    assert_eq!(st_fb, 200, "{raw_fb}");
    assert_eq!(fb_json["quality_bar"]["status"], "improving");

    // Run 2 (different client, same family) -> beats Run 1
    let (st2, r2, raw2) = http_req(
        addr,
        "POST",
        "/api/chat",
        Some(&json!({ "message": "Draft a weekly client brief for Beta" }).to_string()),
    );
    assert_eq!(st2, 200, "{raw2}");
    let score2 = r2["quality_bar"]["score"].as_f64().unwrap();
    assert!(score2 > score1, "Run 2 ({score2}) must beat Run 1 ({score1}): {raw2}");
    assert!(r2["quality_bar"]["delta_vs_prev"].as_f64().unwrap() > 0.0);

    // Accept as good enough
    let (st_acc, acc_json, raw_acc) = http_req(
        addr,
        "POST",
        "/api/quality-bars/weekly-client-brief/feedback",
        Some(
            &json!({
                "action": "accept_good_enough",
                "recipe": "Keep Risk Register + Owner Actions with [file:L##] citations"
            })
            .to_string(),
        ),
    );
    assert_eq!(st_acc, 200, "{raw_acc}");
    assert_eq!(acc_json["quality_bar"]["status"], "accepted_good_enough");

    // Run 3 -> holds at bar, DO NOT DRIFT guidance injected
    let (st3, r3, raw3) = http_req(
        addr,
        "POST",
        "/api/chat",
        Some(&json!({ "message": "Draft a weekly client brief for Gamma" }).to_string()),
    );
    assert_eq!(st3, 200, "{raw3}");
    assert_eq!(r3["quality_bar"]["status"], "accepted_good_enough");
    assert_eq!(r3["quality_bar"]["stable_at_bar"], true);
    assert_eq!(r3["quality_bar"]["changes_stopped"], true);

    // Verify system prompts seen across Run 1 -> Run 2 -> Run 3
    let prompts = seen_systems.lock().unwrap();
    assert_eq!(prompts.len(), 3);
    assert!(!prompts[0].contains("QUALITY BAR"));
    assert!(prompts[1].contains("QUALITY BAR (BEAT LAST RUN)"));
    assert!(prompts[1].contains("Risk Register"));
    assert!(prompts[2].contains("DO NOT DRIFT"));
    assert!(prompts[2].contains("Keep Risk Register + Owner Actions"));

    // Verify Brief & Quality Bars endpoints reflect the bar
    let (st_brief, brief, _) = http_req(addr, "GET", "/api/brief", None);
    assert_eq!(st_brief, 200);
    assert_eq!(brief["quality_bars"][0]["status"], "accepted_good_enough");
    assert_eq!(
        brief["finished"][0]["quality_bar"]["task_family"],
        "weekly-client-brief"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(env.cfg.data_dir.join("quality_bars.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
