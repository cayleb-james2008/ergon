//! Ergon — an AI agent for every employee, personal to each person, that
//! learns how they work and finishes their digital work autonomously.
//!
//! Architecture: a single Rust binary serving an embedded web UI over a
//! hand-rolled HTTP server (std TcpListener). The agent loop talks to an
//! OpenAI-compatible chat-completions endpoint (OpenCode Go by default)
//! using a strict JSON tool protocol, executes tools for real against a
//! sandboxed workspace folder, and uses Rune scripts (see `skills`) for
//! the document-writing and memory-update skills.

pub mod skills;
pub mod ui;

use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const BROWSER_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
pub const NO_KEY_ERROR: &str =
    "no model key configured: set ERGON_MODEL_API_KEY so the agent can reach its model. Ergon will not fabricate an answer without one.";
pub const STEP_CAP: usize = 24;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub memory_path: PathBuf,
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Config {
        let data_dir = std::env::var("ERGON_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data"));
        let workspace_dir = std::env::var("ERGON_WORKSPACE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("workspace"));
        let memory_path = data_dir.join("memory.md");
        let model_base_url = std::env::var("ERGON_MODEL_BASE_URL")
            .unwrap_or_else(|_| "https://opencode.ai/zen/go/v1".to_string())
            .trim_end_matches('/')
            .to_string();
        let model_id =
            std::env::var("ERGON_MODEL_ID").unwrap_or_else(|_| "deepseek-v4.1-flash".to_string());
        let api_key = std::env::var("ERGON_MODEL_API_KEY")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url,
            model_id,
            api_key,
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.workspace_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        if !self.memory_path.exists() {
            std::fs::write(&self.memory_path, "# Memory\n\n")?;
        }
        Ok(())
    }

    pub fn read_memory(&self) -> String {
        std::fs::read_to_string(&self.memory_path).unwrap_or_else(|_| "# Memory\n\n".to_string())
    }
}

// ---------------------------------------------------------------------------
// Sandboxed workspace paths + small helpers
// ---------------------------------------------------------------------------

/// Resolve `rel` inside `root`, rejecting absolute paths and any `..`
/// escape. Every file tool goes through here.
pub fn sandbox_path(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = rel.trim();
    if rel.is_empty() {
        return Ok(root.to_path_buf());
    }
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err(format!("absolute paths are not allowed: {rel}"));
    }
    let mut out = root.to_path_buf();
    for comp in p.components() {
        match comp {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(format!("path escapes the workspace: {rel}")),
        }
    }
    Ok(out)
}

pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push_str("\n…[truncated]");
    out
}

/// Very small HTML-to-text extraction: drop tags and script/style bodies,
/// collapse whitespace. Good enough for the fetch_url tool.
pub fn html_to_text(html: &str) -> String {
    let lower = html.to_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut chars = html.char_indices().peekable();
    let mut in_tag = false;
    let mut skip_until: Option<&str> = None;
    while let Some((i, c)) = chars.next() {
        if let Some(close) = skip_until {
            if lower[i..].starts_with(close) {
                skip_until = None;
                in_tag = true;
            }
            continue;
        }
        match c {
            '<' => {
                in_tag = true;
                let rest = &lower[i..];
                if rest.starts_with("<script") {
                    skip_until = Some("</script");
                } else if rest.starts_with("<style") {
                    skip_until = Some("</style");
                }
            }
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ => {
                if !in_tag {
                    out.push(c);
                }
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn fetch_url_text(url: &str) -> Result<String, String> {
    let resp = ureq::get(url)
        .set("User-Agent", BROWSER_UA)
        .timeout(std::time::Duration::from_secs(20))
        .call()
        .map_err(|e| format!("fetch {url} failed: {e}"))?;
    let body = resp
        .into_string()
        .map_err(|e| format!("read {url} failed: {e}"))?;
    let text = if body.trim_start().starts_with('<') {
        html_to_text(&body)
    } else {
        body
    };
    Ok(truncate(&text, 12_000))
}

// ---------------------------------------------------------------------------
// Thread store
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant" | "step" | "error"
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Default)]
pub struct Store {
    pub threads: Vec<Thread>,
}

pub type SharedStore = Arc<Mutex<Store>>;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn new_thread_id() -> String {
    let n = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("t{millis}-{n}")
}

impl Store {
    pub fn create_thread(&mut self) -> String {
        let id = new_thread_id();
        self.threads.push(Thread {
            id: id.clone(),
            title: "New chat".to_string(),
            messages: Vec::new(),
        });
        id
    }

    pub fn ensure_thread(&mut self, id: &str) -> String {
        if id.is_empty() || !self.threads.iter().any(|t| t.id == id) {
            return self.create_thread();
        }
        id.to_string()
    }

    pub fn push(&mut self, thread_id: &str, role: &str, content: &str) {
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) {
            if role == "user" && t.messages.is_empty() {
                t.title = truncate(&content.chars().take(60).collect::<String>(), 60);
            }
            t.messages.push(ChatMessage {
                role: role.to_string(),
                content: content.to_string(),
            });
        }
    }

    pub fn thread_json(&self, id: &str) -> Option<Value> {
        self.threads.iter().find(|t| t.id == id).map(|t| {
            json!({
                "id": t.id,
                "title": t.title,
                "messages": t.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
            })
        })
    }

    pub fn list_json(&self) -> Value {
        json!({
            "threads": self.threads.iter().rev().map(|t| json!({"id": t.id, "title": t.title})).collect::<Vec<_>>(),
        })
    }
}

// ---------------------------------------------------------------------------
// Model client (OpenCode Go / OpenAI-compatible chat completions)
// ---------------------------------------------------------------------------

/// One model call. Carries the proven wire requirements: bearer key,
/// `x-opencode-session` header, and a browser-like User-Agent.
pub fn call_model(cfg: &Config, session: &str, messages: &[Value]) -> Result<Value, String> {
    let key = cfg.api_key.as_deref().ok_or_else(|| NO_KEY_ERROR.to_string())?;
    let url = format!("{}/chat/completions", cfg.model_base_url);
    let payload = json!({
        "model": cfg.model_id,
        "messages": messages,
        "temperature": 0.2,
    });
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {key}"))
        .set("x-opencode-session", session)
        .set("User-Agent", BROWSER_UA)
        .timeout(std::time::Duration::from_secs(120))
        .send_json(payload);
    let resp = match resp {
        Ok(r) => r,
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(format!("model endpoint returned HTTP {code}: {}", truncate(&body, 400)));
        }
        Err(e) => return Err(format!("model call failed: {e}")),
    };
    let v: Value = resp
        .into_json()
        .map_err(|e| format!("model response was not JSON: {e}"))?;
    let content = &v["choices"][0]["message"]["content"];
    match content {
        Value::String(s) => parse_model_reply(s),
        Value::Object(_) => Ok(content.clone()),
        _ => Err(format!(
            "model response had no message content: {}",
            truncate(&v.to_string(), 400)
        )),
    }
}

/// Parse one model reply: exactly one JSON object, either a tool call or
/// done. Tolerates ```json fences around the object.
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
            return Ok(v);
        }
    }
    // Last resort: first { .. last } slice.
    if let (Some(a), Some(b)) = (unfenced.find('{'), unfenced.rfind('}')) {
        if a < b {
            if let Ok(v) = serde_json::from_str::<Value>(&unfenced[a..=b]) {
                if v.is_object() {
                    return Ok(v);
                }
            }
        }
    }
    Err(format!("model reply was not a JSON object: {}", truncate(raw, 300)))
}

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

pub fn execute_tool(cfg: &Config, name: &str, args: &Value) -> Result<String, String> {
    match name {
        "list_dir" => {
            let rel = args["path"].as_str().unwrap_or(".");
            let full = sandbox_path(&cfg.workspace_dir, rel)?;
            let mut entries: Vec<String> = Vec::new();
            let rd = std::fs::read_dir(&full)
                .map_err(|e| format!("list_dir {}: {e}", full.display()))?;
            for e in rd.filter_map(|e| e.ok()) {
                let fname = e.file_name().to_string_lossy().into_owned();
                let suffix = if e.path().is_dir() { "/" } else { "" };
                entries.push(format!("{fname}{suffix}"));
            }
            entries.sort();
            if entries.is_empty() {
                Ok("(empty directory)".to_string())
            } else {
                Ok(entries.join("\n"))
            }
        }
        "read_file" => {
            let rel = args["path"].as_str().ok_or("read_file needs args.path")?;
            let full = sandbox_path(&cfg.workspace_dir, rel)?;
            let content = std::fs::read_to_string(&full)
                .map_err(|e| format!("read_file {rel}: {e}"))?;
            Ok(truncate(&content, 12_000))
        }
        "write_file" => {
            let rel = args["path"].as_str().ok_or("write_file needs args.path")?;
            let content = args["content"].as_str().ok_or("write_file needs args.content")?;
            let full = sandbox_path(&cfg.workspace_dir, rel)?;
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("write_file mkdir: {e}"))?;
            }
            std::fs::write(&full, content).map_err(|e| format!("write_file {rel}: {e}"))?;
            Ok(format!("wrote {rel} ({} bytes)", content.len()))
        }
        "fetch_url" => {
            let url = args["url"].as_str().ok_or("fetch_url needs args.url")?;
            fetch_url_text(url)
        }
        "write_document" => {
            let title = args["title"].as_str().ok_or("write_document needs args.title")?;
            let sections = normalize_sections(&args["sections"]);
            if sections.is_empty() {
                return Err("write_document needs args.sections".to_string());
            }
            skills::write_document(cfg, title, sections)
        }
        "update_memory" => {
            let note = args["note"]
                .as_str()
                .or_else(|| args["content"].as_str())
                .or_else(|| args["text"].as_str())
                .ok_or("update_memory needs args.note")?;
            skills::update_memory(cfg, note)
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn normalize_sections(v: &Value) -> Vec<String> {
    match v {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => s.clone(),
                Value::Object(o) => {
                    let heading = o["heading"].as_str().unwrap_or("");
                    let body = o["body"].as_str().unwrap_or("");
                    if heading.is_empty() {
                        body.to_string()
                    } else {
                        format!("## {heading}\n\n{body}")
                    }
                }
                other => other.to_string(),
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Agent loop
// ---------------------------------------------------------------------------

fn system_prompt(cfg: &Config) -> String {
    format!(
        "You are Ergon, a personal AI agent for one employee. Your job is to FINISH their digital work autonomously, end to end — not to prepare drafts for approval, not to ask for confirmation. You act through tools and you keep going until the work is done.\n\n\
         PROTOCOL: Reply with exactly ONE JSON object per message, nothing else.\n\
         - To use a tool: {{\"tool\": \"<name>\", \"args\": {{...}}}}\n\
         - When the whole task is complete: {{\"done\": true, \"summary\": \"<what was accomplished>\"}}\n\n\
         TOOLS:\n\
         - list_dir {{\"path\": \".\"}} — list files in the workspace (paths are relative to the workspace root).\n\
         - read_file {{\"path\": \"notes.txt\"}} — read a workspace file.\n\
         - write_file {{\"path\": \"out.txt\", \"content\": \"...\"}} — write a workspace file.\n\
         - fetch_url {{\"url\": \"https://...\"}} — fetch a URL and read its text.\n\
         - write_document {{\"title\": \"Summary\", \"sections\": [\"section text\", ...]}} — produce a finished markdown document in the workspace (a Rune skill formats and writes it).\n\
         - update_memory {{\"note\": \"...\"}} — record something you learned about this person (a preference or fact) into their long-term memory (a Rune skill).\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. When you learn a durable preference or fact about this person, record it with update_memory before finishing. Finish with done only when the task is actually complete.\n\n\
         WHAT YOU KNOW ABOUT THIS PERSON (their memory file — treat it as true and use it):\n{memory}",
        memory = cfg.read_memory()
    )
}

pub struct AgentOutcome {
    pub summary: String,
    pub steps: Vec<String>,
}

/// Run one task to completion: model → tool → model → … until done,
/// the step cap, or a genuine block. Steps are pushed into the thread as
/// they happen; the finished summary lands as the final message.
pub fn run_agent(
    cfg: &Config,
    store: &SharedStore,
    thread_id: &str,
    task: &str,
) -> Result<AgentOutcome, String> {
    if cfg.api_key.is_none() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let session = format!("ergon-{thread_id}");
    let mut messages: Vec<Value> = vec![
        json!({"role": "system", "content": system_prompt(cfg)}),
        json!({"role": "user", "content": task}),
    ];
    let mut steps: Vec<String> = Vec::new();
    let mut seen_tools: HashSet<String> = HashSet::new();

    for _ in 0..STEP_CAP {
        let reply = call_model(cfg, &session, &messages)?;
        if reply["done"].as_bool() == Some(true) {
            let summary = reply["summary"]
                .as_str()
                .unwrap_or("Task complete.")
                .to_string();
            store
                .lock()
                .unwrap()
                .push(thread_id, "assistant", &summary);
            return Ok(AgentOutcome { summary, steps });
        }
        let tool = reply["tool"]
            .as_str()
            .ok_or_else(|| format!("model reply was neither a tool call nor done: {}", truncate(&reply.to_string(), 300)))?
            .to_string();
        let args = reply["args"].clone();
        seen_tools.insert(tool.clone());

        let (result_text, ok) = match execute_tool(cfg, &tool, &args) {
            Ok(r) => (r, true),
            Err(e) => (format!("error: {e}"), false),
        };
        let step_line = format!("Tool: {tool}");
        steps.push(step_line.clone());
        store.lock().unwrap().push(
            thread_id,
            "step",
            &format!("{step_line}\n{}", truncate(&result_text, 500)),
        );
        let _ = ok;
        messages.push(json!({"role": "assistant", "content": reply.to_string()}));
        messages.push(json!({
            "role": "user",
            "content": format!("Tool result for {tool}: {}", truncate(&result_text, 6000)),
        }));
    }
    let _ = seen_tools;
    Err(format!(
        "agent stopped after {STEP_CAP} steps without finishing — the task may be blocked or too large for one run."
    ))
}

// ---------------------------------------------------------------------------
// HTTP server (hand-rolled, std TcpListener)
// ---------------------------------------------------------------------------

pub struct AppState {
    pub cfg: Config,
    pub store: SharedStore,
}

pub fn serve(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    cfg.ensure_dirs()?;
    let state = Arc::new(AppState {
        cfg,
        store: Arc::new(Mutex::new(Store::default())),
    });
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || {
                    let _ = handle_connection(stream, &state);
                });
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut std::net::TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let raw_path = parts.next()?.to_string();
    let path = raw_path.split('?').next().unwrap_or("/").to_string();
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).ok()?;
    }
    Some(Request { method, path, body })
}

fn respond(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

fn respond_json(stream: &mut std::net::TcpStream, status: u16, v: &Value) -> std::io::Result<()> {
    respond(stream, status, "application/json", &v.to_string())
}

fn handle_connection(mut stream: std::net::TcpStream, state: &AppState) -> std::io::Result<()> {
    let req = match read_request(&mut stream) {
        Some(r) => r,
        None => return Ok(()),
    };
    let cfg = &state.cfg;
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => respond(&mut stream, 200, "text/html; charset=utf-8", ui::INDEX_HTML),
        ("GET", "/api/status") => respond_json(
            &mut stream,
            200,
            &json!({
                "model_configured": cfg.api_key.is_some(),
                "model": cfg.model_id,
                "model_base_url": cfg.model_base_url,
            }),
        ),
        ("GET", "/api/threads") => {
            let store = state.store.lock().unwrap();
            respond_json(&mut stream, 200, &store.list_json())
        }
        ("POST", "/api/threads") => {
            let mut store = state.store.lock().unwrap();
            let id = store.create_thread();
            let v = store.thread_json(&id).unwrap_or_else(|| json!({"id": id}));
            respond_json(&mut stream, 201, &v)
        }
        ("POST", "/api/chat") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let message = parsed["message"].as_str().unwrap_or("").trim().to_string();
            if message.is_empty() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "message is required"}),
                );
            }
            if cfg.api_key.is_none() {
                return respond_json(
                    &mut stream,
                    503,
                    &json!({"error": NO_KEY_ERROR}),
                );
            }
            let thread_id = {
                let mut store = state.store.lock().unwrap();
                let tid = store.ensure_thread(parsed["thread_id"].as_str().unwrap_or(""));
                store.push(&tid, "user", &message);
                tid
            };
            match run_agent(cfg, &state.store, &thread_id, &message) {
                Ok(_outcome) => {
                    let store = state.store.lock().unwrap();
                    let v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    respond_json(&mut stream, 200, &v)
                }
                Err(e) => {
                    state.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(&mut stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", path) if path.starts_with("/api/threads/") => {
            let id = path.trim_start_matches("/api/threads/");
            let store = state.store.lock().unwrap();
            match store.thread_json(id) {
                Some(v) => respond_json(&mut stream, 200, &v),
                None => respond_json(&mut stream, 404, &json!({"error": "thread not found"})),
            }
        }
        _ => respond_json(&mut stream, 404, &json!({"error": "not found"})),
    }
}
