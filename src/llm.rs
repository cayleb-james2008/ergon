//! Model client supporting both OpenAI-compatible (`/v1/chat/completions`)
//! and native Anthropic Messages (`/v1/messages`) endpoints, including SSE
//! streaming (`token_delta` frames), prompt caching, usage accounting, and
//! bounded transient retry backoff.

use crate::{parse_url_host_port, truncate, Config, BROWSER_UA, NO_KEY_ERROR};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::BufRead as _;

/// HTTP statuses worth another try: timeouts, rate limits, overloads and
/// gateway hiccups (including Anthropic 529 overloaded). Anything else (400s, auth) fails at once.
pub fn retryable_status(code: u16) -> bool {
    matches!(code, 408 | 425 | 429 | 500 | 502 | 503 | 504 | 520..=529)
}

/// How long to wait before attempt `attempt + 1` (1-based attempts):
/// the server's `Retry-After` seconds when given (capped), else 1.5 s, 4 s, 9 s.
pub fn retry_delay(attempt: u32, retry_after: Option<&str>) -> std::time::Duration {
    if let Some(secs) = retry_after
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s >= 0.0)
    {
        return std::time::Duration::from_millis((secs.min(20.0) * 1000.0) as u64);
    }
    std::time::Duration::from_millis(match attempt {
        1 => 1_500,
        2 => 4_000,
        _ => 9_000,
    })
}

const MODEL_ATTEMPTS: u32 = 4;
const MODEL_RETRY_BUDGET: std::time::Duration = std::time::Duration::from_secs(150);
const MAX_SSE_STREAM_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelCallUsage {
    pub model_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cached_tokens: u64,
    pub latency_ms: u64,
}

impl ModelCallUsage {
    pub fn to_json(&self, cost_usd: f64) -> Value {
        json!({
            "model_id": self.model_id,
            "prompt_tokens": self.prompt_tokens,
            "completion_tokens": self.completion_tokens,
            "total_tokens": self.total_tokens,
            "cached_tokens": self.cached_tokens,
            "latency_ms": self.latency_ms,
            "cost_usd": crate::records::round_usd_8(cost_usd),
        })
    }
}

#[derive(Clone, Copy)]
pub struct ModelStreamTarget<'a> {
    pub events: &'a crate::sse::RunEventBus,
    pub person_id: &'a str,
    pub run_id: &'a str,
    pub thread_id: &'a str,
    pub step_index: usize,
}

#[derive(Default)]
struct SseToolCallAccum {
    name: String,
    arguments: String,
}

/// Detect whether `url` points to an Anthropic Messages API endpoint.
pub fn is_anthropic_url(url: &str) -> bool {
    if let Ok(proto) = std::env::var("ERGON_MODEL_PROTOCOL") {
        let p = proto.trim().to_ascii_lowercase();
        if p == "anthropic" {
            return true;
        }
        if p == "openai" {
            return false;
        }
    }
    let trimmed = url.trim().trim_end_matches('/').to_ascii_lowercase();
    if trimmed.ends_with("/messages") || trimmed.ends_with("/anthropic/v1") {
        return true;
    }
    parse_url_host_port(url)
        .map(|(h, _)| h == "api.anthropic.com" || h.ends_with(".anthropic.com"))
        .unwrap_or(false)
}

fn model_disallows_temperature(model_id: &str) -> bool {
    let m = model_id.trim().to_ascii_lowercase();
    let tail = m.rsplit('/').next().unwrap_or(&m);
    tail.starts_with("claude-haiku-5")
        || tail.starts_with("claude-sonnet-5")
        || tail.starts_with("claude-opus-5")
        || tail.starts_with("claude-5")
        || tail.starts_with("o1")
        || tail.starts_with("o3")
        || tail.starts_with("o4")
        || tail.starts_with("gpt-5")
}

pub fn anthropic_messages_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with("/messages") {
        trimmed.to_string()
    } else if lower.ends_with("/v1") {
        format!("{trimmed}/messages")
    } else {
        format!("{trimmed}/v1/messages")
    }
}

fn parse_usage_from_value(
    usage_obj: Option<&Value>,
    model_id: String,
    latency_ms: u64,
) -> ModelCallUsage {
    let prompt_tokens = usage_obj
        .and_then(|u| {
            u.get("prompt_tokens")
                .or_else(|| u.get("input_tokens"))
                .and_then(|x| x.as_u64())
        })
        .unwrap_or(0);
    let completion_tokens = usage_obj
        .and_then(|u| {
            u.get("completion_tokens")
                .or_else(|| u.get("output_tokens"))
                .and_then(|x| x.as_u64())
        })
        .unwrap_or(0);
    let total_tokens = usage_obj
        .and_then(|u| u.get("total_tokens").and_then(|x| x.as_u64()))
        .unwrap_or_else(|| prompt_tokens.saturating_add(completion_tokens));
    let cached_tokens = usage_obj
        .and_then(|u| {
            u.get("cached_tokens")
                .or_else(|| u.get("prompt_tokens_details").and_then(|d| d.get("cached_tokens")))
                .or_else(|| u.get("input_tokens_details").and_then(|d| d.get("cached_tokens")))
                .or_else(|| u.get("cache_read_input_tokens"))
                .and_then(|x| x.as_u64())
        })
        .unwrap_or(0);
    ModelCallUsage {
        model_id,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        cached_tokens,
        latency_ms,
    }
}

fn message_text_content(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.get("text")
                    .and_then(|t| t.as_str())
                    .or_else(|| item.as_str())
                    .map(str::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Convert OpenAI-shaped `[ {role, content}, ... ]` messages into an Anthropic
/// `/v1/messages` request body with `cache_control: ephemeral` on the system prompt.
pub fn build_anthropic_payload(cfg: &Config, messages: &[Value], stream: bool) -> Value {
    let mut system_parts: Vec<String> = Vec::new();
    let mut turns: Vec<(String, String)> = Vec::new();

    for msg in messages {
        let raw_role = msg
            .get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("user")
            .trim()
            .to_ascii_lowercase();
        let text = message_text_content(msg.get("content").unwrap_or(&Value::Null));
        if text.trim().is_empty() {
            continue;
        }
        if raw_role == "system" {
            system_parts.push(text);
            continue;
        }
        let role = if raw_role == "assistant" {
            "assistant".to_string()
        } else {
            "user".to_string()
        };
        if let Some(last) = turns.last_mut() {
            if last.0 == role {
                last.1.push_str("\n\n");
                last.1.push_str(&text);
                continue;
            }
        }
        turns.push((role, text));
    }

    if turns.first().is_some_and(|(r, _)| r != "user") || turns.is_empty() {
        turns.insert(0, ("user".to_string(), "Proceed with the task.".to_string()));
    }

    let api_messages: Vec<Value> = turns
        .into_iter()
        .map(|(role, content)| json!({ "role": role, "content": content }))
        .collect();

    let max_tokens = std::env::var("ERGON_ANTHROPIC_MAX_TOKENS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|n| n.clamp(256, 32768))
        .unwrap_or(4096);

    let mut payload = json!({
        "model": cfg.model_id,
        "max_tokens": max_tokens,
        "messages": api_messages,
    });
    if !model_disallows_temperature(&cfg.model_id) {
        if let Ok(raw_t) = std::env::var("ERGON_MODEL_TEMPERATURE") {
            if let Ok(t) = raw_t.trim().parse::<f64>() {
                if t.is_finite() {
                    payload["temperature"] = json!(t);
                }
            }
        }
    }
    if !system_parts.is_empty() {
        payload["system"] = json!([
            {
                "type": "text",
                "text": system_parts.join("\n\n"),
                "cache_control": { "type": "ephemeral" }
            }
        ]);
    }
    if stream {
        payload["stream"] = json!(true);
    }
    payload
}

fn consume_openai_sse_data_line(
    line: &str,
    content_buf: &mut String,
    tool_calls: &mut BTreeMap<usize, SseToolCallAccum>,
    model_id: &mut Option<String>,
    usage_val: &mut Option<Value>,
    sse_error: &mut Option<String>,
    stream_target: Option<ModelStreamTarget<'_>>,
) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return false;
    }
    let Some(data_raw) = trimmed.strip_prefix("data:") else {
        return false;
    };
    let data = data_raw.trim();
    if data.is_empty() {
        return false;
    }
    if data == "[DONE]" {
        return true;
    }
    let Ok(chunk) = serde_json::from_str::<Value>(data) else {
        return false;
    };
    if let Some(err_obj) = chunk.get("error").filter(|e| !e.is_null()) {
        let msg = err_obj
            .get("message")
            .and_then(|m| m.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| err_obj.to_string());
        *sse_error = Some(truncate(&msg, 400));
    }
    if let Some(m) = chunk["model"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
        *model_id = Some(m.to_string());
    }
    if let Some(u) = chunk.get("usage").filter(|u| u.is_object()) {
        *usage_val = Some(u.clone());
    }
    let Some(delta) = chunk
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("delta"))
    else {
        return false;
    };
    if let Some(text) = delta.get("content").and_then(|c| c.as_str()) {
        if !text.is_empty() && content_buf.len() < MAX_SSE_STREAM_BYTES {
            content_buf.push_str(text);
            if let Some(target) = stream_target {
                target.events.publish_token_delta(
                    target.person_id,
                    target.run_id,
                    target.thread_id,
                    text,
                    target.step_index,
                );
            }
        }
    }
    if let Some(tc_arr) = delta.get("tool_calls").and_then(|t| t.as_array()) {
        for tc in tc_arr {
            let idx = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
            let entry = tool_calls.entry(idx).or_default();
            if let Some(func) = tc.get("function") {
                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                    if entry.name.len() < 256 {
                        entry.name.push_str(name);
                    }
                }
                if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                    if entry.arguments.len() < MAX_SSE_STREAM_BYTES {
                        entry.arguments.push_str(args);
                    }
                    if !args.is_empty() {
                        if let Some(target) = stream_target {
                            target.events.publish_token_delta(
                                target.person_id,
                                target.run_id,
                                target.thread_id,
                                args,
                                target.step_index,
                            );
                        }
                    }
                }
            }
        }
    }
    false
}

fn merge_usage_object(dst: &mut Option<Value>, incoming: &Value) {
    let Some(inc_obj) = incoming.as_object() else {
        return;
    };
    if dst.as_ref().is_none_or(|v| !v.is_object()) {
        *dst = Some(Value::Object(inc_obj.clone()));
        return;
    }
    if let Some(Value::Object(map)) = dst.as_mut() {
        for (k, v) in inc_obj {
            map.insert(k.clone(), v.clone());
        }
    }
}

fn consume_anthropic_sse_data_line(
    line: &str,
    content_buf: &mut String,
    tool_calls: &mut BTreeMap<usize, SseToolCallAccum>,
    model_id: &mut Option<String>,
    usage_val: &mut Option<Value>,
    sse_error: &mut Option<String>,
    stream_target: Option<ModelStreamTarget<'_>>,
) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') || trimmed.starts_with("event:") {
        return false;
    }
    let Some(data_raw) = trimmed.strip_prefix("data:") else {
        return false;
    };
    let data = data_raw.trim();
    if data.is_empty() {
        return false;
    }
    if data == "[DONE]" {
        return true;
    }
    let Ok(chunk) = serde_json::from_str::<Value>(data) else {
        return false;
    };

    if let Some(err_obj) = chunk.get("error").filter(|e| !e.is_null()) {
        let msg = err_obj
            .get("message")
            .and_then(|m| m.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| err_obj.to_string());
        *sse_error = Some(truncate(&msg, 400));
        return false;
    }

    match chunk.get("type").and_then(|t| t.as_str()).unwrap_or("") {
        "message_start" => {
            if let Some(msg) = chunk.get("message") {
                if let Some(m) = msg["model"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    *model_id = Some(m.to_string());
                }
                if let Some(u) = msg.get("usage") {
                    merge_usage_object(usage_val, u);
                }
            }
        }
        "content_block_start" => {
            let idx = chunk.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
            if let Some(cb) = chunk.get("content_block") {
                if cb.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                    let entry = tool_calls.entry(idx).or_default();
                    if let Some(name) = cb.get("name").and_then(|n| n.as_str()) {
                        entry.name = name.to_string();
                    }
                    if let Some(inp) = cb.get("input").filter(|v| v.is_object() && !v.as_object().unwrap().is_empty()) {
                        entry.arguments = inp.to_string();
                    }
                }
            }
        }
        "content_block_delta" => {
            let idx = chunk.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
            let Some(delta) = chunk.get("delta") else {
                return false;
            };
            let dtype = delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if dtype == "text_delta" {
                if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                    if !text.is_empty() && content_buf.len() < MAX_SSE_STREAM_BYTES {
                        content_buf.push_str(text);
                        if let Some(target) = stream_target {
                            target.events.publish_token_delta(
                                target.person_id,
                                target.run_id,
                                target.thread_id,
                                text,
                                target.step_index,
                            );
                        }
                    }
                }
            } else if dtype == "input_json_delta" {
                if let Some(part) = delta.get("partial_json").and_then(|t| t.as_str()) {
                    let entry = tool_calls.entry(idx).or_default();
                    if entry.arguments.len() < MAX_SSE_STREAM_BYTES {
                        entry.arguments.push_str(part);
                    }
                    if !part.is_empty() {
                        if let Some(target) = stream_target {
                            target.events.publish_token_delta(
                                target.person_id,
                                target.run_id,
                                target.thread_id,
                                part,
                                target.step_index,
                            );
                        }
                    }
                }
            }
        }
        "message_delta" => {
            if let Some(u) = chunk.get("usage") {
                merge_usage_object(usage_val, u);
            }
        }
        "message_stop" => return true,
        _ => {}
    }
    false
}

fn finalize_sse_reply(
    content_buf: &str,
    tool_calls: &BTreeMap<usize, SseToolCallAccum>,
    sse_error: Option<&str>,
) -> Result<Value, String> {
    if !content_buf.trim().is_empty() {
        return parse_model_reply(content_buf);
    }
    if let Some((_, tc)) = tool_calls.iter().next() {
        if !tc.name.trim().is_empty() {
            let raw_args = tc.arguments.trim();
            let args_val = if raw_args.is_empty() {
                json!({})
            } else {
                match serde_json::from_str::<Value>(raw_args) {
                    Ok(v) if v.is_object() => v,
                    Ok(_) | Err(_) => {
                        return Err(format!(
                            "model SSE tool_call arguments were incomplete or malformed JSON: {}",
                            truncate(raw_args, 200)
                        ));
                    }
                }
            };
            return Ok(normalize_model_reply(json!({
                "tool": tc.name.trim(),
                "args": args_val,
            })));
        }
    }
    if let Some(err) = sse_error {
        return Err(format!("model SSE stream error: {err}"));
    }
    Err("model SSE stream ended with no assistant content or tool_calls".to_string())
}

fn parse_single_json_model_response(
    cfg: &Config,
    v: &Value,
    latency_ms: u64,
    anthropic: bool,
) -> Result<(Value, ModelCallUsage), String> {
    let model_id = v["model"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&cfg.model_id)
        .to_string();
    let usage = parse_usage_from_value(v.get("usage"), model_id, latency_ms);

    if anthropic || v.get("choices").is_none() && v.get("content").is_some() {
        if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
            let mut text_parts = Vec::new();
            for block in arr {
                let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if btype == "text" {
                    if let Some(t) = block.get("text").and_then(|x| x.as_str()) {
                        text_parts.push(t);
                    }
                } else if btype == "tool_use" && text_parts.is_empty() {
                    if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                        let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                        return Ok((
                            normalize_model_reply(json!({ "tool": name, "args": input })),
                            usage,
                        ));
                    }
                }
            }
            let joined = text_parts.join("\n");
            if !joined.trim().is_empty() {
                return Ok((parse_model_reply(&joined)?, usage));
            }
        }
        return Err(format!(
            "Anthropic response had no text or tool_use block: {}",
            truncate(&v.to_string(), 400)
        ));
    }

    let msg = &v["choices"][0]["message"];
    let content = &msg["content"];
    let parsed = match content {
        Value::String(s) if !s.trim().is_empty() => parse_model_reply(s)?,
        Value::Object(_) => normalize_model_reply(content.clone()),
        _ => {
            if let Some(tc0) = msg.get("tool_calls").and_then(|t| t.get(0)) {
                let name = tc0["function"]["name"].as_str().unwrap_or("").trim();
                if !name.is_empty() {
                    let args_raw = &tc0["function"]["arguments"];
                    let args_val = match args_raw {
                        Value::String(s) => serde_json::from_str::<Value>(s)
                            .unwrap_or_else(|_| Value::String(s.clone())),
                        other => other.clone(),
                    };
                    normalize_model_reply(json!({ "tool": name, "args": args_val }))
                } else {
                    return Err(format!(
                        "model response had no message content: {}",
                        truncate(&v.to_string(), 400)
                    ));
                }
            } else {
                return Err(format!(
                    "model response had no message content: {}",
                    truncate(&v.to_string(), 400)
                ));
            }
        }
    };
    Ok((parsed, usage))
}

pub fn call_model(cfg: &Config, session: &str, messages: &[Value]) -> Result<Value, String> {
    call_model_with_usage(cfg, session, messages).map(|(v, _)| v)
}

pub fn call_model_with_usage(
    cfg: &Config,
    session: &str,
    messages: &[Value],
) -> Result<(Value, ModelCallUsage), String> {
    call_model_with_usage_streaming(cfg, session, messages, None)
}

pub fn call_model_with_usage_streaming(
    cfg: &Config,
    session: &str,
    messages: &[Value],
    stream_target: Option<ModelStreamTarget<'_>>,
) -> Result<(Value, ModelCallUsage), String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let anthropic = is_anthropic_url(&cfg.model_base_url);
    let url = if anthropic {
        anthropic_messages_url(&cfg.model_base_url)
    } else {
        format!("{}/chat/completions", cfg.model_base_url)
    };
    let payload = if anthropic {
        build_anthropic_payload(cfg, messages, stream_target.is_some())
    } else {
        let mut p = json!({
            "model": cfg.model_id,
            "messages": messages,
        });
        if !model_disallows_temperature(&cfg.model_id) {
            p["temperature"] = json!(0.2);
        }
        if stream_target.is_some() {
            p["stream"] = json!(true);
        }
        if cfg.model_base_url.contains(":11434") {
            p["response_format"] = json!({"type": "json_object"});
        }
        p
    };

    // Transient failures retry with bounded backoff honoring Retry-After.
    // Dropped connections retry for remote endpoints only so a stopped local
    // gateway reports its failure immediately.
    let started = std::time::Instant::now();
    let remote = !crate::is_loopback_url(&cfg.model_base_url);
    let read_timeout_secs = std::env::var("ERGON_MODEL_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|s| s.clamp(10, 900))
        .unwrap_or(300);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(read_timeout_secs))
        .timeout_write(std::time::Duration::from_secs(60))
        .build();
    let mut attempt: u32 = 0;
    let resp = loop {
        attempt += 1;
        let req = if anthropic {
            let mut r = agent
                .post(&url)
                .set("anthropic-version", "2023-06-01")
                .set("User-Agent", BROWSER_UA);
            if let Some(key) = cfg.api_key.as_deref().filter(|k| !k.is_empty()) {
                r = r.set("x-api-key", key);
            }
            r
        } else {
            let mut r = agent
                .post(&url)
                .set("x-opencode-session", session)
                .set("User-Agent", BROWSER_UA);
            if let Some(key) = cfg.api_key.as_deref().filter(|k| !k.is_empty()) {
                r = r.set("Authorization", &format!("Bearer {key}"));
            }
            r
        };
        let can_retry = attempt < MODEL_ATTEMPTS && started.elapsed() < MODEL_RETRY_BUDGET;
        match req.send_json(payload.clone()) {
            Ok(r) => break r,
            Err(ureq::Error::Status(code, r)) if can_retry && retryable_status(code) => {
                let wait = retry_delay(attempt, r.header("retry-after"));
                crate::logging::warn(
                    "ergon::model",
                    &format!(
                        "ergon: model endpoint returned HTTP {code}; retrying in {} ms (attempt {}/{MODEL_ATTEMPTS})",
                        wait.as_millis(),
                        attempt + 1
                    ),
                );
                std::thread::sleep(wait);
            }
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                let tries = if attempt > 1 {
                    format!(" (after {attempt} attempts)")
                } else {
                    String::new()
                };
                return Err(format!(
                    "model endpoint returned HTTP {code}{tries}: {}",
                    truncate(&body, 400)
                ));
            }
            Err(ureq::Error::Transport(t))
                if can_retry
                    && remote
                    && matches!(
                        t.kind(),
                        ureq::ErrorKind::Io | ureq::ErrorKind::ConnectionFailed
                    ) =>
            {
                let wait = retry_delay(attempt, None);
                crate::logging::warn(
                    "ergon::model",
                    &format!(
                        "ergon: model call failed ({t}); retrying in {} ms (attempt {}/{MODEL_ATTEMPTS})",
                        wait.as_millis(),
                        attempt + 1
                    ),
                );
                std::thread::sleep(wait);
            }
            Err(e) => return Err(format!("model call failed: {e}")),
        }
    };

    let content_type = resp
        .header("content-type")
        .unwrap_or("")
        .to_ascii_lowercase();
    if content_type.contains("text/event-stream") {
        let mut reader = std::io::BufReader::new(resp.into_reader());
        let mut content_buf = String::new();
        let mut tool_calls: BTreeMap<usize, SseToolCallAccum> = BTreeMap::new();
        let mut model_id: Option<String> = None;
        let mut usage_val: Option<Value> = None;
        let mut sse_error: Option<String> = None;
        let mut line = String::new();
        let mut bytes_seen = 0usize;
        loop {
            line.clear();
            let n = reader
                .read_line(&mut line)
                .map_err(|e| format!("reading model SSE stream failed: {e}"))?;
            if n == 0 {
                break;
            }
            bytes_seen = bytes_seen.saturating_add(n);
            if bytes_seen > MAX_SSE_STREAM_BYTES * 2 {
                return Err("model SSE stream exceeded maximum size".to_string());
            }
            let done = if anthropic {
                consume_anthropic_sse_data_line(
                    &line,
                    &mut content_buf,
                    &mut tool_calls,
                    &mut model_id,
                    &mut usage_val,
                    &mut sse_error,
                    stream_target,
                )
            } else {
                consume_openai_sse_data_line(
                    &line,
                    &mut content_buf,
                    &mut tool_calls,
                    &mut model_id,
                    &mut usage_val,
                    &mut sse_error,
                    stream_target,
                )
            };
            if done {
                break;
            }
        }
        let latency_ms = started.elapsed().as_millis() as u64;
        let parsed = finalize_sse_reply(&content_buf, &tool_calls, sse_error.as_deref())?;
        let usage = parse_usage_from_value(
            usage_val.as_ref(),
            model_id.unwrap_or_else(|| cfg.model_id.clone()),
            latency_ms,
        );
        return Ok((parsed, usage));
    }

    let raw_body = resp
        .into_string()
        .map_err(|e| format!("reading model response failed: {e}"))?;
    let latency_ms = started.elapsed().as_millis() as u64;
    let looks_like_sse = raw_body.trim_start().starts_with("data:")
        || raw_body.trim_start().starts_with("event:")
        || (!raw_body.trim_start().starts_with('{')
            && raw_body
                .lines()
                .any(|l| l.trim_start().starts_with("data:")));
    if looks_like_sse {
        let mut content_buf = String::new();
        let mut tool_calls: BTreeMap<usize, SseToolCallAccum> = BTreeMap::new();
        let mut model_id: Option<String> = None;
        let mut usage_val: Option<Value> = None;
        let mut sse_error: Option<String> = None;
        for line in raw_body.lines() {
            let done = if anthropic {
                consume_anthropic_sse_data_line(
                    line,
                    &mut content_buf,
                    &mut tool_calls,
                    &mut model_id,
                    &mut usage_val,
                    &mut sse_error,
                    stream_target,
                )
            } else {
                consume_openai_sse_data_line(
                    line,
                    &mut content_buf,
                    &mut tool_calls,
                    &mut model_id,
                    &mut usage_val,
                    &mut sse_error,
                    stream_target,
                )
            };
            if done {
                break;
            }
        }
        let parsed = finalize_sse_reply(&content_buf, &tool_calls, sse_error.as_deref())?;
        let usage = parse_usage_from_value(
            usage_val.as_ref(),
            model_id.unwrap_or_else(|| cfg.model_id.clone()),
            latency_ms,
        );
        return Ok((parsed, usage));
    }

    let v: Value =
        serde_json::from_str(&raw_body).map_err(|e| format!("model response was not JSON: {e}"))?;
    parse_single_json_model_response(cfg, &v, latency_ms, anthropic)
}

fn normalize_model_reply(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        // Common local-model / OpenAI-function-call aliases:
        // {"name": "write_file", "arguments": {...}} -> {"tool": "write_file", "args": {...}}
        if !obj.contains_key("tool") {
            if let Some(name) = obj
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
            {
                obj.remove("name");
                obj.insert("tool".to_string(), Value::String(name));
            }
        }
        if obj.contains_key("tool") && !obj.contains_key("args") {
            if let Some(arguments) = obj.remove("arguments") {
                obj.insert("args".to_string(), arguments);
            }
        }
        if let Some(Value::String(s)) = obj.get("args").cloned() {
            if let Ok(parsed) = serde_json::from_str::<Value>(&s) {
                if parsed.is_object() {
                    obj.insert("args".to_string(), parsed);
                }
            }
        }
        if obj.contains_key("tool") && !obj.get("args").is_some_and(|a| a.is_object()) {
            let mut extra = serde_json::Map::new();
            let keys: Vec<String> = obj
                .keys()
                .filter(|k| k.as_str() != "tool" && k.as_str() != "args")
                .cloned()
                .collect();
            for k in keys {
                if let Some(val) = obj.remove(&k) {
                    extra.insert(k, val);
                }
            }
            obj.insert("args".to_string(), Value::Object(extra));
        }
        if obj.get("done").and_then(|d| d.as_str()) == Some("true") {
            obj.insert("done".to_string(), Value::Bool(true));
        }
        if !obj.contains_key("tool")
            && !obj.contains_key("done")
            && !obj.contains_key("path")
            && obj
                .get("summary")
                .and_then(|s| s.as_str())
                .is_some_and(|s| !s.trim().is_empty())
        {
            obj.insert("done".to_string(), Value::Bool(true));
        }
    }
    v
}

/// Extract the first balanced JSON object from a text blob that may include prose
/// or code fences around it.
fn extract_first_json_object(s: &str) -> Option<Value> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0usize;
            let mut in_str = false;
            let mut esc = false;
            let mut j = i;
            while j < bytes.len() {
                let b = bytes[j];
                if in_str {
                    if esc {
                        esc = false;
                    } else if b == b'\\' {
                        esc = true;
                    } else if b == b'"' {
                        in_str = false;
                    }
                } else {
                    match b {
                        b'"' => in_str = true,
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                if let Ok(v) = serde_json::from_str::<Value>(&s[i..=j]) {
                                    if v.is_object() {
                                        return Some(v);
                                    }
                                }
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }
    None
}

/// Parse one model reply: exactly one JSON object, either a tool call or
/// done. Tolerates ```json fences or brief wrapper prose around the object.
pub fn parse_model_reply(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.strip_suffix("```").unwrap_or(s))
        .unwrap_or(trimmed)
        .trim();
    if let Ok(v) = serde_json::from_str::<Value>(unfenced) {
        if v.is_object() {
            return Ok(normalize_model_reply(v));
        }
    }
    if let Some(v) = extract_first_json_object(unfenced) {
        return Ok(normalize_model_reply(v));
    }
    // Last resort: first { .. last } slice.
    if let (Some(a), Some(b)) = (unfenced.find('{'), unfenced.rfind('}')) {
        if a < b {
            if let Ok(v) = serde_json::from_str::<Value>(&unfenced[a..=b]) {
                if v.is_object() {
                    return Ok(normalize_model_reply(v));
                }
            }
        }
    }
    Err(format!("model reply was not a JSON object: {}", truncate(raw, 300)))
}
