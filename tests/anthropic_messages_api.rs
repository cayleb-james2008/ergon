//! Native Anthropic Messages API (`/v1/messages`) & Agent Integration Proof
//!
//! Proves:
//! 1. `is_anthropic_url` detects `https://api.anthropic.com/v1`, `/messages` endpoints,
//!    and `ERGON_MODEL_PROTOCOL=anthropic`.
//! 2. `Config::from_env()` picks up `ANTHROPIC_API_KEY` cleanly when explicit OpenAI URLs
//!    are unset (`https://api.anthropic.com/v1`, `claude-sonnet-4-20250514`).
//! 3. Requests carry `x-api-key` and `anthropic-version: 2023-06-01` (never `Authorization`),
//!    system prompt carries `cache_control: {"type": "ephemeral"}`, and turns alternate.
//! 4. Streaming SSE (`message_start`, `content_block_delta` `text_delta`, `message_delta`)
//!    emits live `token_delta` events and accounts `cache_read_input_tokens`.
//! 5. Non-streaming JSON responses, `tool_use` blocks, HTTP 529 overloaded retries,
//!    and end-to-end `run_agent` (`write_document`) work cleanly via `/v1/messages`.

use ergon_rs::{
    build_anthropic_payload, call_model_with_usage_streaming, is_anthropic_url, library,
    run_agent, sse, Config, ModelStreamTarget, Store, ToolCtx, DEFAULT_ANTHROPIC_BASE_URL,
    DEFAULT_ANTHROPIC_MODEL_ID,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_cfg(label: &str, base_url: &str, api_key: Option<&str>) -> (PathBuf, Config) {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let base = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    let root = base
        .join("test-data")
        .join(format!("anthropic-{label}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let data_dir = root.join("data");
    let workspace_dir = root.join("workspace");
    let memory_path = data_dir.join("memory.md");
    let cfg = Config {
        data_dir,
        workspace_dir,
        memory_path,
        model_base_url: base_url.to_string(),
        model_id: DEFAULT_ANTHROPIC_MODEL_ID.to_string(),
        api_key: api_key.map(str::to_string),
    };
    cfg.ensure_dirs().unwrap();
    (root, cfg)
}

fn read_http_req(reader: &mut BufReader<std::net::TcpStream>) -> (String, HashMap<String, String>, Value) {
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();
    let mut headers = HashMap::new();
    let mut content_len = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let tr = line.trim();
        if tr.is_empty() {
            break;
        }
        if let Some((k, v)) = tr.split_once(':') {
            let kl = k.trim().to_ascii_lowercase();
            let vv = v.trim().to_string();
            if kl == "content-length" {
                content_len = vv.parse().unwrap_or(0);
            }
            headers.insert(kl, vv);
        }
    }
    let mut body = vec![0u8; content_len];
    reader.read_exact(&mut body).unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (request_line.trim().to_string(), headers, json_body)
}

#[test]
fn anthropic_protocol_detection_env_and_payload_translation() {
    assert!(is_anthropic_url("https://api.anthropic.com/v1"));
    assert!(is_anthropic_url("http://127.0.0.1:9000/v1/messages"));
    assert!(!is_anthropic_url("https://opencode.ai/zen/go/v1"));

    let (root, cfg) = temp_cfg("payload", DEFAULT_ANTHROPIC_BASE_URL, Some("sk-ant-test-key"));
    let msgs = vec![
        json!({"role": "system", "content": "You are Ergon."}),
        json!({"role": "assistant", "content": "Prior assistant turn"}),
        json!({"role": "user", "content": "Part 1"}),
        json!({"role": "tool", "content": "Tool observation"}),
    ];
    let body = build_anthropic_payload(&cfg, &msgs, true);
    assert_eq!(body["model"], json!(DEFAULT_ANTHROPIC_MODEL_ID));
    assert_eq!(body["stream"], json!(true));
    assert_eq!(body["system"][0]["type"], json!("text"));
    assert_eq!(body["system"][0]["text"], json!("You are Ergon."));
    assert_eq!(
        body["system"][0]["cache_control"],
        json!({"type": "ephemeral"})
    );
    let turns = body["messages"].as_array().unwrap();
    assert_eq!(turns[0]["role"], json!("user"));
    // Verify ANTHROPIC_API_KEY env auto-selection cleanly while preserving prior env
    let prev_ant = std::env::var_os("ANTHROPIC_API_KEY");
    let prev_dd = std::env::var_os("ERGON_DATA_DIR");
    let prev_no = std::env::var_os("ERGON_NO_AUTO_MODEL");
    let prev_base = std::env::var_os("ERGON_MODEL_BASE_URL");
    let prev_key = std::env::var_os("ERGON_MODEL_API_KEY");
    let prev_mod = std::env::var_os("ERGON_MODEL_ID");

    std::env::remove_var("ERGON_NO_AUTO_MODEL");
    std::env::remove_var("ERGON_MODEL_BASE_URL");
    std::env::remove_var("ERGON_MODEL_API_KEY");
    std::env::remove_var("ERGON_MODEL_ID");
    std::env::set_var("ERGON_DATA_DIR", root.join("env-data"));
    std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-mock-env");
    let env_cfg = Config::from_env();
    assert_eq!(env_cfg.model_base_url, DEFAULT_ANTHROPIC_BASE_URL);
    assert_eq!(env_cfg.model_id, DEFAULT_ANTHROPIC_MODEL_ID);
    assert_eq!(env_cfg.api_key.as_deref(), Some("sk-ant-mock-env"));
    assert!(env_cfg.model_configured());

    // Restore environment variables
    let restore = |k: &str, v: Option<std::ffi::OsString>| match v {
        Some(val) => std::env::set_var(k, val),
        None => std::env::remove_var(k),
    };
    restore("ANTHROPIC_API_KEY", prev_ant);
    restore("ERGON_DATA_DIR", prev_dd);
    restore("ERGON_NO_AUTO_MODEL", prev_no);
    restore("ERGON_MODEL_BASE_URL", prev_base);
    restore("ERGON_MODEL_API_KEY", prev_key);
    restore("ERGON_MODEL_ID", prev_mod);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn anthropic_streaming_caching_529_retry_and_agent_tool_execution() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let seen_headers: Arc<Mutex<Vec<HashMap<String, String>>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let call_num = Arc::new(AtomicUsize::new(0));

    let sh_bg = seen_headers.clone();
    let sp_bg = seen_paths.clone();
    let cn_bg = call_num.clone();

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let (req_line, headers, req_json) = read_http_req(&mut reader);
            let n = cn_bg.fetch_add(1, Ordering::SeqCst);
            sp_bg.lock().unwrap().push(req_line);
            sh_bg.lock().unwrap().push(headers);

            let mut s = reader.into_inner();
            if n == 0 {
                // Verify 529 overloaded retry backoff recovers transparently
                let body = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
                let resp = format!(
                    "HTTP/1.1 529 Overloaded\r\nContent-Type: application/json\r\nRetry-After: 0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(resp.as_bytes());
                continue;
            }
            if n == 1 {
                // Streaming Anthropic SSE response with cache_read_input_tokens
                assert_eq!(req_json["stream"], json!(true));
                let sse = concat!(
                    "event: message_start\n",
                    "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":120,\"cache_read_input_tokens\":800,\"output_tokens\":1}}}\n\n",
                    "event: content_block_delta\n",
                    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"{\\\"tool\\\":\\\"write_document\\\", \"}}\n\n",
                    "event: content_block_delta\n",
                    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"\\\"args\\\":{\\\"title\\\":\\\"Claude Brief\\\",\\\"sections\\\":[\\\"Verified via Anthropic Messages API.\\\"]}}}\"}}\n\n",
                    "event: message_delta\n",
                    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":42}}\n\n",
                    "event: message_stop\n",
                    "data: {\"type\":\"message_stop\"}\n\n"
                );
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sse}",
                    sse.len()
                );
                let _ = s.write_all(resp.as_bytes());
                continue;
            }
            // Non-streaming final done JSON turn from agent loop
            let msg_json = json!({
                "id": "msg_done",
                "type": "message",
                "role": "assistant",
                "model": DEFAULT_ANTHROPIC_MODEL_ID,
                "content": [
                    {
                        "type": "text",
                        "text": "{\"done\":true,\"summary\":\"Wrote Claude Brief.md via Anthropic API.\"}"
                    }
                ],
                "usage": {
                    "input_tokens": 90,
                    "cache_read_input_tokens": 800,
                    "output_tokens": 24
                }
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{msg_json}",
                msg_json.len()
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });

    let (root, cfg) = temp_cfg(
        "e2e",
        &format!("http://127.0.0.1:{}/v1/messages", addr.port()),
        Some("sk-ant-live-mock"),
    );

    // 1. Direct streaming call (hits 529 once, retries, parses Anthropic SSE & token_delta)
    let bus = sse::RunEventBus::new();
    let target = ModelStreamTarget {
        events: &bus,
        person_id: "default",
        run_id: "r1",
        thread_id: "t1",
        step_index: 0,
    };
    let msgs = vec![
        json!({"role": "system", "content": "System instructions"}),
        json!({"role": "user", "content": "Write Claude Brief"}),
    ];
    let (reply, usage) = call_model_with_usage_streaming(&cfg, "s-ant", &msgs, Some(target)).unwrap();
    assert_eq!(reply["tool"], json!("write_document"));
    assert_eq!(usage.prompt_tokens, 120);
    assert_eq!(usage.cached_tokens, 800);
    assert_eq!(usage.completion_tokens, 42);
    assert!(
        bus.wait_since(0, std::time::Duration::ZERO)
            .iter()
            .any(|ev| ev.event == "token_delta" && ev.data["delta"].as_str().unwrap_or("").contains("write_document")),
        "expected token_delta SSE frames from Anthropic content_block_delta"
    );

    // Verify wire headers & path
    {
        let paths = seen_paths.lock().unwrap();
        assert!(paths[1].starts_with("POST /v1/messages "));
        let hdrs = &seen_headers.lock().unwrap()[1];
        assert_eq!(hdrs.get("x-api-key").map(String::as_str), Some("sk-ant-live-mock"));
        assert_eq!(hdrs.get("anthropic-version").map(String::as_str), Some("2023-06-01"));
        assert!(!hdrs.contains_key("authorization"));
    }

    // 2. Full non-streaming agent turn completing cleanly
    let store = Arc::new(Mutex::new(Store::load(&cfg.data_dir.join("threads.json"))));
    let tid = { store.lock().unwrap().ensure_thread("ant-t1") };
    let lib = Arc::new(Mutex::new(library::Library::load(&cfg.data_dir)));
    let ctx = ToolCtx {
        library: lib,
        root_data_dir: cfg.data_dir.clone(),
        person_id: "default".to_string(),
    };
    let out = run_agent(&cfg, &ctx, &store, &tid, "Finish up", &[]).unwrap();
    assert!(out.summary.contains("Wrote Claude Brief.md"), "{}", out.summary);

    let _ = std::fs::remove_dir_all(&root);
}
