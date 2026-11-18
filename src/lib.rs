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
use std::collections::HashMap;
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

/// Lexically normalise an absolute path: drop `.` components and
/// resolve `..` by popping (never above the filesystem root).
fn normalize_absolute(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::RootDir => out.push("/"),
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Prefix(_) => {}
        }
    }
    out
}

/// Resolve `rel` inside `root`, rejecting absolute paths and any escape
/// — including through symlinks. Every file tool goes through here.
///
/// The walk is a queue of components over the canonicalised root.
/// Whenever the component just added is a symlink (checked with
/// `symlink_metadata`, which sees DANGLING links too — `exists()` does
/// not, and that blindness was a proven escape), the link's target
/// components are pushed back onto the FRONT of the queue and walked
/// like any other components — so a target that itself contains links
/// or `..` steps is fully re-resolved, never trusted lexically. (A
/// previous version substituted the target lexically and only checked
/// the final path; a link to `y/secret` with `y` itself a link outside
/// walked straight through it.) At every step the current path must be
/// inside the root or an ancestor of it (the walk may pass through `/`
/// while resolving an absolute target); the finished path must be
/// inside the root. Chains are capped at 16 resolutions.
pub fn sandbox_path(root: &Path, rel: &str) -> Result<PathBuf, String> {
    sandbox_path_resolving(root, rel, true)
}

/// Like `sandbox_path`, but the FINAL component is not resolved when
/// `resolve_final` is false — used by move_file's source, where the
/// thing being moved is the link itself, not its target.
pub fn sandbox_path_resolving(
    root: &Path,
    rel: &str,
    resolve_final: bool,
) -> Result<PathBuf, String> {
    let rel = rel.trim();
    if rel.is_empty() {
        return Ok(root.to_path_buf());
    }
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err(format!("absolute paths are not allowed: {rel}"));
    }
    enum Step {
        Push(std::ffi::OsString),
        Pop,
    }
    let mut queue: std::collections::VecDeque<Step> = std::collections::VecDeque::new();
    for comp in p.components() {
        match comp {
            Component::Normal(part) => queue.push_back(Step::Push(part.to_os_string())),
            Component::CurDir => {}
            _ => return Err(format!("path escapes the workspace: {rel}")),
        }
    }
    let canon_root = root
        .canonicalize()
        .unwrap_or_else(|_| normalize_absolute(&root.to_path_buf()));
    let mut cur = canon_root.clone();
    let mut hops = 0usize;
    while let Some(step) = queue.pop_front() {
        match step {
            Step::Pop => {
                cur.pop();
            }
            Step::Push(part) => {
                cur.push(&part);
                let is_final = queue.is_empty();
                if !(is_final && !resolve_final) {
                    if let Ok(md) = cur.symlink_metadata() {
                        if md.file_type().is_symlink() {
                            hops += 1;
                            if hops > 16 {
                                return Err(format!("path escapes the workspace: {rel}"));
                            }
                            let target = std::fs::read_link(&cur).map_err(|e| {
                                format!("cannot resolve a link in path {rel}: {e}")
                            })?;
                            cur.pop(); // back to the link's parent directory
                            if target.is_absolute() {
                                cur = PathBuf::from("/");
                            }
                            // The target's components go back on the
                            // front of the queue, in order, and are
                            // walked (and re-resolved) like any others.
                            let mut target_steps: Vec<Step> = Vec::new();
                            for comp in target.components() {
                                match comp {
                                    Component::Normal(part) => {
                                        target_steps
                                            .push(Step::Push(part.to_os_string()))
                                    }
                                    Component::ParentDir => target_steps.push(Step::Pop),
                                    Component::CurDir | Component::RootDir => {}
                                    Component::Prefix(_) => {
                                        return Err(format!(
                                            "path escapes the workspace: {rel}"
                                        ))
                                    }
                                }
                            }
                            for s in target_steps.into_iter().rev() {
                                queue.push_front(s);
                            }
                        }
                    }
                }
            }
        }
        // Containment at every step: inside the root, or an ancestor
        // of it (legitimate only while an absolute target is being
        // walked from `/` toward the root).
        if !cur.starts_with(&canon_root) && !canon_root.starts_with(&cur) {
            return Err(format!("path escapes the workspace: {rel}"));
        }
    }
    if !cur.starts_with(&canon_root) {
        return Err(format!("path escapes the workspace: {rel}"));
    }
    Ok(cur)
}

/// File names mentioned in a piece of text (tokens ending in a known
/// document extension) — used to check a "done" summary's named
/// artefacts against the workspace that supposedly holds them.
pub fn named_files(text: &str) -> Vec<String> {
    const EXTS: [&str; 8] = ["md", "csv", "txt", "json", "html", "pdf", "docx", "xlsx"];
    // Tokens are runs of filename characters — every other character
    // (whitespace, quotes, ':', '!', '?', brackets…) separates, so a
    // trailing colon or exclamation mark cannot hide a named file.
    text.split(|c: char| !(c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')))
        .filter_map(|tok| {
            let tok = tok.trim_end_matches('.');
            if tok.contains("://") || tok.starts_with("//") || tok.is_empty() {
                return None; // a URL is not a workspace file
            }
            let (_, ext) = tok.rsplit_once('.')?;
            if !tok.starts_with('.') && EXTS.contains(&ext.to_lowercase().as_str()) {
                Some(tok.to_string())
            } else {
                None
            }
        })
        .collect()
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
// People — one agent per employee. Each person has their own memory file,
// workspace and thread store. The "default" person uses the root config's
// own workspace/memory (the original single-person layout); every other
// person lives under data_dir/people/<id>/.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub role: String,
}

impl Person {
    pub fn to_json(&self) -> Value {
        json!({"id": self.id, "name": self.name, "role": self.role})
    }
    pub fn from_json(v: &Value) -> Option<Person> {
        Some(Person {
            id: v["id"].as_str()?.to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            role: v["role"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// A person id is safe to use as a directory name: lowercase slug
/// characters only, bounded length. Ids minted by `slugify` always
/// qualify; ids loaded from disk must prove it.
pub fn valid_person_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true; // trims leading dashes
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "person".to_string()
    } else {
        out
    }
}

#[derive(Clone, Debug)]
pub struct PeopleState {
    pub people: Vec<Person>,
    pub active: String,
}

impl Default for PeopleState {
    fn default() -> Self {
        PeopleState {
            people: vec![Person {
                id: "default".to_string(),
                name: "You".to_string(),
                role: String::new(),
            }],
            active: "default".to_string(),
        }
    }
}

/// Move a corrupt state file aside to the first free backup name
/// (`<file>.bak`, then `.bak2`, `.bak3`, …) so its bytes survive for
/// inspection instead of being overwritten by the next save — or by
/// the previous backup.
pub fn backup_corrupt(path: &Path) -> Option<PathBuf> {
    let mut candidate = path.with_extension("json.bak");
    let mut n = 2;
    while candidate.exists() {
        candidate = path.with_extension(format!("json.bak{n}"));
        n += 1;
    }
    std::fs::rename(path, &candidate).ok()?;
    eprintln!(
        "ergon: {} was corrupt; backed up to {} and starting fresh",
        path.display(),
        candidate.display()
    );
    Some(candidate)
}

impl PeopleState {
    pub fn load(root: &Config) -> PeopleState {
        let path = root.data_dir.join("people.json");
        if let Ok(text) = std::fs::read_to_string(&path) {
            let parsed = serde_json::from_str::<Value>(&text);
            // Valid means exactly the shape this app writes: a JSON
            // object whose "people" is an array. Anything else —
            // unparseable, a bare array/number, a missing or mistyped
            // key — is corruption: keep the bytes as a backup rather
            // than letting the next save destroy them silently. (An
            // earlier check treated a MISSING key as fine, which let
            // `[]`, `{}` and `42` be silently overwritten.)
            let shape_ok = parsed
                .as_ref()
                .map(|v| {
                    v.is_object()
                        && v.get("people").map(|p| p.is_array()).unwrap_or(false)
                })
                .unwrap_or(false);
            if !shape_ok {
                let _ = backup_corrupt(&path);
                return PeopleState::default();
            }
            if let Ok(v) = parsed {
                let mut state = PeopleState::default();
                if let Some(items) = v["people"].as_array() {
                    // Ids from disk are validated, not trusted: an id is
                    // a directory name under data_dir, so a crafted
                    // people.json must not be able to aim a person's
                    // workspace outside it (e.g. id "../../elsewhere").
                    let parsed: Vec<Person> = items
                        .iter()
                        .filter_map(Person::from_json)
                        .filter(|p| valid_person_id(&p.id))
                        .collect();
                    if !items.is_empty() && parsed.is_empty() {
                        // Every entry was unusable — that is corruption
                        // wearing a valid shape; back it up like any
                        // other corrupt file instead of overwriting it.
                        let _ = backup_corrupt(&path);
                        return PeopleState::default();
                    }
                    if !parsed.is_empty() {
                        state.people = parsed;
                    }
                }
                if !state.people.iter().any(|p| p.id == "default") {
                    state.people.insert(
                        0,
                        Person {
                            id: "default".to_string(),
                            name: "You".to_string(),
                            role: String::new(),
                        },
                    );
                }
                if let Some(active) = v["active"].as_str() {
                    if state.people.iter().any(|p| p.id == active) {
                        state.active = active.to_string();
                    }
                }
                return state;
            }
        }
        PeopleState::default()
    }

    pub fn save(&self, root: &Config) {
        let _ = std::fs::create_dir_all(&root.data_dir);
        let v = json!({
            "people": self.people.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "active": self.active,
        });
        let path = root.data_dir.join("people.json");
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, v.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    pub fn active_person(&self) -> Person {
        self.people
            .iter()
            .find(|p| p.id == self.active)
            .cloned()
            .unwrap_or_else(|| self.people[0].clone())
    }

    pub fn add(&mut self, name: &str, role: &str) -> Person {
        // Names and roles are interpolated into the seeded memory file,
        // so they are flattened to single lines and length-capped here:
        // a name carrying newlines could otherwise forge sections and
        // bullets in memory.md (structure injection).
        let clean = |s: &str, cap: usize| -> String {
            let flat: String = s
                .replace(['\n', '\r'], " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            flat.chars().take(cap).collect()
        };
        let name = clean(name, 80);
        let role = clean(role, 120);
        let base = slugify(&name);
        let mut id = base.clone();
        let mut n = 2;
        while self.people.iter().any(|p| p.id == id) {
            id = format!("{base}-{n}");
            n += 1;
        }
        let person = Person {
            id,
            name,
            role,
        };
        self.people.push(person.clone());
        person
    }

    pub fn to_json(&self) -> Value {
        json!({
            "people": self.people.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "active": self.active,
        })
    }
}

impl Config {
    /// The config scoped to one person: their own data dir (threads.json
    /// lives there), workspace and memory file. The default person keeps
    /// the root layout.
    pub fn person_config(&self, person: &Person) -> Config {
        if person.id == "default" {
            return self.clone();
        }
        let dir = self.data_dir.join("people").join(&person.id);
        Config {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            memory_path: dir.join("memory.md"),
            model_base_url: self.model_base_url.clone(),
            model_id: self.model_id.clone(),
            api_key: self.api_key.clone(),
        }
    }
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

pub fn thread_to_json(t: &Thread) -> Value {
    json!({
        "id": t.id,
        "title": t.title,
        "messages": t.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
    })
}

pub fn thread_from_json(v: &Value) -> Option<Thread> {
    let id = v["id"].as_str()?.to_string();
    let title = v["title"].as_str().unwrap_or("New chat").to_string();
    let mut messages = Vec::new();
    if let Some(items) = v["messages"].as_array() {
        for m in items {
            if let (Some(role), Some(content)) = (m["role"].as_str(), m["content"].as_str()) {
                messages.push(ChatMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                });
            }
        }
    }
    Some(Thread { id, title, messages })
}

#[derive(Default)]
pub struct Store {
    pub threads: Vec<Thread>,
    /// Where this store persists (threads.json); set by `Store::load`.
    pub save_path: Option<PathBuf>,
}

impl Store {
    /// Load a store from its threads.json (missing/corrupt file = empty
    /// store). The store saves itself back to the same path on every
    /// mutation, atomically (tmp file + rename).
    pub fn load(path: &Path) -> Store {
        let mut store = Store {
            threads: Vec::new(),
            save_path: Some(path.to_path_buf()),
        };
        if let Ok(text) = std::fs::read_to_string(path) {
            match serde_json::from_str::<Value>(&text) {
                Ok(v) => match v["threads"].as_array() {
                    Some(items) => {
                        for item in items {
                            if let Some(t) = thread_from_json(item) {
                                store.threads.push(t);
                            }
                        }
                    }
                    // Valid JSON, wrong shape (e.g. {"threads": 42}):
                    // just as corrupt as a syntax error — the next
                    // save must not destroy these bytes unrecorded.
                    None => {
                        let _ = backup_corrupt(path);
                    }
                },
                Err(_) => {
                    let _ = backup_corrupt(path);
                }
            }
        }
        store
    }

    pub fn save(&self) {
        let Some(path) = &self.save_path else { return };
        let v = json!({
            "threads": self.threads.iter().map(thread_to_json).collect::<Vec<_>>(),
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, v.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
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
        self.save();
        id
    }

    pub fn ensure_thread(&mut self, id: &str) -> String {
        if id.is_empty() || !self.threads.iter().any(|t| t.id == id) {
            return self.create_thread();
        }
        id.to_string()
    }

    pub fn push(&mut self, thread_id: &str, role: &str, content: &str) {
        let mut changed = false;
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) {
            if role == "user" && t.messages.is_empty() {
                t.title = truncate(&content.chars().take(60).collect::<String>(), 60);
            }
            t.messages.push(ChatMessage {
                role: role.to_string(),
                content: content.to_string(),
            });
            changed = true;
        }
        if changed {
            self.save();
        }
    }

    pub fn thread_json(&self, id: &str) -> Option<Value> {
        self.threads.iter().find(|t| t.id == id).map(thread_to_json)
    }

    /// The conversation so far, as model messages: the last 12
    /// user/assistant turns (step and error lines are the ledger, not
    /// conversation), each truncated. Without this the agent saw only
    /// the current task — a thread was a log, not a conversation, and
    /// "now change it" had nothing to refer to.
    pub fn history_json(&self, thread_id: &str) -> Vec<Value> {
        let Some(t) = self.threads.iter().find(|t| t.id == thread_id) else {
            return Vec::new();
        };
        let turns: Vec<&ChatMessage> = t
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .collect();
        turns
            .iter()
            .skip(turns.len().saturating_sub(12))
            .map(|m| {
                json!({
                    "role": if m.role == "user" { "user" } else { "assistant" },
                    "content": truncate(&m.content, 1500),
                })
            })
            .collect()
    }

    pub fn list_json(&self) -> Value {
        json!({
            "threads": self.threads.iter().rev().map(|t| json!({"id": t.id, "title": t.title, "messages": t.messages.len()})).collect::<Vec<_>>(),
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
        "make_spreadsheet" => {
            let title = args["title"]
                .as_str()
                .ok_or("make_spreadsheet needs args.title")?;
            let columns = string_list(&args["columns"]);
            let rows = string_rows(&args["rows"]);
            if columns.is_empty() && rows.is_empty() {
                return Err("make_spreadsheet needs args.columns or args.rows".to_string());
            }
            skills::make_spreadsheet(cfg, title, columns, rows)
        }
        "research" => {
            let topic = args["topic"].as_str().ok_or("research needs args.topic")?;
            let urls = string_list(&args["urls"]);
            if urls.is_empty() {
                return Err("research needs args.urls (a list of page URLs)".to_string());
            }
            skills::research(cfg, topic, urls)
        }
        "search_files" => {
            let query = args["query"]
                .as_str()
                .ok_or("search_files needs args.query")?
                .to_lowercase();
            if query.is_empty() {
                return Err("search_files needs a non-empty args.query".to_string());
            }
            let rel = args["path"].as_str().unwrap_or(".");
            let base = sandbox_path(&cfg.workspace_dir, rel)?;
            let mut matches: Vec<String> = Vec::new();
            let mut stats = SearchStats::default();
            if base.is_file() {
                // A path naming one file searches that file (walking
                // it as a directory used to silently find nothing). An
                // unreadable file is disclosed as skipped, like the
                // walk path — never a plain "(no matches)".
                match std::fs::read_to_string(&base) {
                    Ok(content) => {
                        for (i, line) in content.lines().enumerate() {
                            if line.to_lowercase().contains(&query) {
                                if matches.len() >= 50 {
                                    stats.capped = true;
                                    break;
                                }
                                matches.push(format!(
                                    "{}:{}: {}",
                                    rel,
                                    i + 1,
                                    truncate(line.trim(), 120)
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        stats.skipped += 1;
                    }
                }
            } else {
                search_walk(&cfg.workspace_dir, &base, &query, &mut matches, 0, &mut stats);
            }
            // The result must say what it did NOT search: hidden files,
            // files over 512 KB and folders deeper than 6 levels are
            // skipped, and results stop at 50 matches. Presenting a
            // partial search as complete would mislead the agent.
            let skip_note = format!(
                "({} entries (files or folders) were not searched: hidden, over 512 KB, unreadable, or deeper than 6 levels)",
                stats.skipped
            );
            if matches.is_empty() {
                if stats.skipped > 0 {
                    Ok(format!("(no matches in the files searched) {skip_note}"))
                } else {
                    Ok("(no matches)".to_string())
                }
            } else {
                let mut out = matches.join("\n");
                if stats.capped {
                    out.push_str("\n(stopped at 50 matches — there may be more)");
                }
                if stats.skipped > 0 {
                    out.push_str(&format!("\n{skip_note}"));
                }
                Ok(out)
            }
        }
        "move_file" => {
            let from = args["from"].as_str().ok_or("move_file needs args.from")?;
            let to = args["to"].as_str().ok_or("move_file needs args.to")?;
            // NEITHER side's final component is resolved: moving a
            // symlink moves the link itself, and moving onto a link's
            // name replaces the link, not its target. (Resolving the
            // destination used to land the file at the link's target
            // while reporting the link's name.) Existence is checked
            // with symlink_metadata for the same reason — exists()
            // follows links and calls a dangling link "not there".
            let src = sandbox_path_resolving(&cfg.workspace_dir, from, false)?;
            let dst = sandbox_path_resolving(&cfg.workspace_dir, to, false)?;
            if src.symlink_metadata().is_err() {
                return Err(format!("move_file: no such file: {from}"));
            }
            // Never silently overwrite: rename would destroy the
            // destination's content without a word. (symlink_metadata
            // so a dangling link at the destination also counts.)
            if dst.symlink_metadata().is_ok() {
                return Err(format!(
                    "move_file: destination already exists: {to} — pick another name, or move/delete the existing file first"
                ));
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("move_file mkdir: {e}"))?;
            }
            std::fs::rename(&src, &dst)
                .map_err(|e| format!("move_file {from} -> {to}: {e}"))?;
            Ok(format!("moved {from} to {to}"))
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

fn string_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn string_rows(v: &Value) -> Vec<Vec<String>> {
    match v {
        Value::Array(rows) => rows.iter().map(string_list).collect(),
        _ => Vec::new(),
    }
}

/// Recursive text search under `base` (already sandbox-checked). Skips
/// dotfiles, symlinks (never follows them out of the workspace), files
/// over 512 KB, and caps depth and match count so a big tree stays fast.
/// What a search did not cover — reported back to the agent with the
/// results, so partial coverage is never presented as complete.
#[derive(Default)]
struct SearchStats {
    skipped: usize,
    capped: bool,
}

fn search_walk(
    root: &Path,
    base: &Path,
    query: &str,
    out: &mut Vec<String>,
    depth: usize,
    stats: &mut SearchStats,
) {
    if depth > 6 {
        stats.skipped += 1;
        return;
    }
    if stats.capped {
        return;
    }
    let Ok(rd) = std::fs::read_dir(base) else { return };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if stats.capped {
            return;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            stats.skipped += 1;
            continue;
        }
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_symlink() {
            stats.skipped += 1;
            continue;
        }
        let path = entry.path();
        if ft.is_dir() {
            search_walk(root, &path, query, out, depth + 1, stats);
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(0) > 512_000 {
            stats.skipped += 1;
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            stats.skipped += 1;
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(query) {
                // The cap flag is set only when a 51st match actually
                // exists — "there may be more" must be true when said.
                if out.len() >= 50 {
                    stats.capped = true;
                    return;
                }
                out.push(format!("{rel}:{}: {}", i + 1, truncate(line.trim(), 120)));
            }
        }
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

/// The learned facts and preferences in a memory file: every bullet
/// that is not an identity line. Identity is exactly the two seeded
/// lines (`- Name: …`, `- Role: …`); everything else in the file is
/// something the agent learned by working with this person.
///
/// (An earlier version excluded the whole "## About" section — but
/// learned notes are appended at the end of the file, which for a
/// seeded person is still inside About, so learning silently never
/// reached the prompt for anyone created through the people feature.)
pub fn standing_preferences(memory: &str) -> Vec<String> {
    let mut prefs = Vec::new();
    for line in memory.lines() {
        let t = line.trim();
        if let Some(bullet) = t.strip_prefix("- ") {
            let bullet = bullet.trim();
            if bullet.is_empty() || is_identity_bullet(bullet) {
                continue;
            }
            prefs.push(bullet.to_string());
        }
    }
    prefs
}

/// An identity bullet is `Name: …` or `Role: …` in any casing, with
/// optional space before the colon. The check is spelling-tolerant
/// because the boundary it guards (identity vs learned facts) must not
/// depend on exact capitalisation.
fn is_identity_bullet(bullet: &str) -> bool {
    // Normalise the fullwidth colon too — the boundary must not depend
    // on which colon character a note happened to use.
    let lower = bullet.to_lowercase().replace('\u{ff1a}', ":");
    for key in ["name", "role"] {
        if let Some(rest) = lower.strip_prefix(key) {
            if rest.trim_start().starts_with(':') {
                return true;
            }
        }
    }
    false
}

fn system_prompt(cfg: &Config) -> String {
    let memory = cfg.read_memory();
    let prefs = standing_preferences(&memory);
    let prefs_block = if prefs.is_empty() {
        "(none yet — learn how this person works as you go, and record it with update_memory)".to_string()
    } else {
        prefs
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
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
         - make_spreadsheet {{\"title\": \"Budget\", \"columns\": [\"Item\", \"Cost\"], \"rows\": [[\"Coffee\", \"4.50\"]]}} — produce a finished CSV spreadsheet in the workspace (a Rune skill formats and writes it).\n\
         - research {{\"topic\": \"Local Prices\", \"urls\": [\"https://...\", \"https://...\"]}} — fetch several pages and compile them into one research notes document in the workspace (a Rune skill).\n\
         - search_files {{\"query\": \"quarterly\", \"path\": \".\"}} — search the workspace's text files for a phrase; returns matching lines with file and line number.\n\
         - move_file {{\"from\": \"a.txt\", \"to\": \"archive/a.txt\"}} — move or rename a workspace file.\n\
         - update_memory {{\"note\": \"...\"}} — record something you learned about this person (a preference or fact) into their long-term memory (a Rune skill).\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. When you learn a durable preference or fact about this person, record it with update_memory before finishing. Finish with done only when the task is actually complete.\n\n\
         PREFERENCES AND FACTS THIS PERSON HAS TAUGHT YOU — apply them in every task:\n{prefs}\n\n\
         WHAT YOU KNOW ABOUT THIS PERSON (their memory file — treat it as true and use it):\n{memory}",
        prefs = prefs_block,
        memory = memory
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
    history: &[Value],
) -> Result<AgentOutcome, String> {
    if cfg.api_key.is_none() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let session = format!("ergon-{thread_id}");
    let mut messages: Vec<Value> = vec![json!({"role": "system", "content": system_prompt(cfg)})];
    messages.extend(history.iter().cloned());
    messages.push(json!({"role": "user", "content": task}));
    let mut steps: Vec<String> = Vec::new();
    // The files this run actually wrote — the evidence a "done"
    // summary's named files are checked against. Not a count: WHICH
    // files, so a summary claiming a file this run never touched is
    // visible as exactly that.
    let mut written_files: Vec<String> = Vec::new();

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
            // Honesty gate, checked against the run's own write
            // ledger — WHICH files this run wrote, never the summary's
            // verbs or nouns (verb/noun lists flagged honest inline
            // answers and waved through claims about files the run
            // never touched):
            //  1. a named file that does not exist at all is a hard
            //     discrepancy — caution, in the open;
            //  2. a named file that exists but this run did not write
            //     gets a factual ledger line (what the run DID write),
            //     so a pre-existing file claimed as the deliverable is
            //     visible without accusing an honest input reference.
            let named = named_files(&summary);
            let missing_named: Vec<String> = named
                .iter()
                .filter(|f| {
                    sandbox_path(&cfg.workspace_dir, f)
                        .map(|p| !p.exists())
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
            let unwritten_named: Vec<String> = named
                .iter()
                .filter(|f| !missing_named.contains(f) && !written_files.contains(f))
                .cloned()
                .collect();
            if !missing_named.is_empty() {
                store.lock().unwrap().push(
                    thread_id,
                    "step",
                    &format!(
                        "Note: the summary names {}, which is not in the workspace — treat the summary above with caution.",
                        missing_named.join(", ")
                    ),
                );
            } else if !unwritten_named.is_empty() {
                let wrote = if written_files.is_empty() {
                    "no files".to_string()
                } else {
                    written_files.join(", ")
                };
                store.lock().unwrap().push(
                    thread_id,
                    "step",
                    &format!(
                        "Note: this run wrote {wrote}; the summary also names {}, which this run did not write.",
                        unwritten_named.join(", ")
                    ),
                );
            }
            return Ok(AgentOutcome { summary, steps });
        }
        let tool = reply["tool"]
            .as_str()
            .ok_or_else(|| format!("model reply was neither a tool call nor done: {}", truncate(&reply.to_string(), 300)))?
            .to_string();
        let args = reply["args"].clone();

        let (result_text, ok) = match execute_tool(cfg, &tool, &args) {
            Ok(r) => (r, true),
            Err(e) => (format!("error: {e}"), false),
        };
        if ok {
            match tool.as_str() {
                "write_file" => {
                    if let Some(p) = args["path"].as_str() {
                        written_files.push(p.to_string());
                    }
                }
                "move_file" => {
                    if let Some(p) = args["to"].as_str() {
                        written_files.push(p.to_string());
                    }
                }
                // The Rune skills name the file they produced in
                // their result text.
                "write_document" | "make_spreadsheet" | "research" => {
                    written_files.extend(named_files(&result_text));
                }
                _ => {}
            }
        }
        let step_line = format!("Tool: {tool}");
        steps.push(step_line.clone());
        store.lock().unwrap().push(
            thread_id,
            "step",
            &format!("{step_line}\n{}", truncate(&result_text, 500)),
        );
        messages.push(json!({"role": "assistant", "content": reply.to_string()}));
        messages.push(json!({
            "role": "user",
            "content": format!("Tool result for {tool}: {}", truncate(&result_text, 6000)),
        }));
    }
    Err(format!(
        "agent stopped after {STEP_CAP} steps without finishing — the task may be blocked or too large for one run."
    ))
}

// ---------------------------------------------------------------------------
// HTTP server (hand-rolled, std TcpListener)
// ---------------------------------------------------------------------------

/// One person's live state: their scoped config + their thread store.
pub struct PersonRuntime {
    pub cfg: Config,
    pub store: SharedStore,
}

fn load_runtime(root: &Config, person: &Person) -> PersonRuntime {
    let cfg = root.person_config(person);
    let fresh_memory = !cfg.memory_path.exists();
    let _ = cfg.ensure_dirs();
    if fresh_memory && person.id != "default" {
        // Seed a new person's memory with who they are, so the agent
        // starts out knowing its person.
        let mut seed = format!("# Memory\n\n## About\n- Name: {}", person.name);
        if !person.role.is_empty() {
            seed.push_str(&format!("\n- Role: {}", person.role));
        }
        seed.push_str("\n\n## Learned\n");
        seed.push('\n');
        let _ = std::fs::write(&cfg.memory_path, seed);
    }
    let threads_path = cfg.data_dir.join("threads.json");
    PersonRuntime {
        cfg,
        store: Arc::new(Mutex::new(Store::load(&threads_path))),
    }
}

pub struct AppState {
    /// Root config (model route + root data dir).
    pub cfg: Config,
    pub people: Arc<Mutex<PeopleState>>,
    pub runtimes: Arc<Mutex<HashMap<String, Arc<PersonRuntime>>>>,
    /// Browser sessions (cookie id → person id). "Active person" used
    /// to be one global pointer: any client could switch it for every
    /// other client mid-task. A session pins the person per browser;
    /// cookie-less API clients keep the global-pointer behaviour.
    pub sessions: Arc<Mutex<HashMap<String, String>>>,
}

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Session ids are the only credential a browser presents, so they
/// must be unguessable: random bytes from the OS, not a timestamp and
/// a counter (an early version's ids were predictable to within a few
/// guesses of a victim's visit time).
fn new_session_id() -> String {
    let n = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut bytes = [0u8; 16];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut bytes))
        .is_ok();
    if filled {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        format!("s{hex}-{n}")
    } else {
        // No OS randomness available: fall back to a wide mix of
        // time, pid and counter — still far harder to bracket than a
        // bare millisecond timestamp.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("s{:x}-{:x}-{n}", nanos, std::process::id())
    }
}

/// Whether this session id is one the server actually issued. A
/// client-chosen id is never honoured (session fixation): unknown ids
/// are treated as "no session" and replaced with a fresh server id.
/// Insert a session, bounding the map: sessions are memory-only and
/// re-issued transparently, so past a generous cap the map is simply
/// cleared rather than growing for the life of the process.
fn insert_session(state: &AppState, sid: String, person_id: String) {
    let mut sessions = state.sessions.lock().unwrap();
    if sessions.len() > 10_000 {
        sessions.clear();
    }
    sessions.insert(sid, person_id);
}

fn session_known(state: &AppState, session: &Option<String>) -> bool {
    match session {
        Some(sid) => state.sessions.lock().unwrap().contains_key(sid),
        None => false,
    }
}

/// The person this request acts for: the session's person when the
/// caller presented a known session cookie, else the global active.
fn effective_person(state: &AppState, session: &Option<String>) -> Person {
    if let Some(sid) = session {
        let mapped = state.sessions.lock().unwrap().get(sid).cloned();
        if let Some(pid) = mapped {
            let people = state.people.lock().unwrap();
            if let Some(p) = people.people.iter().find(|p| p.id == pid) {
                return p.clone();
            }
        }
    }
    state.people.lock().unwrap().active_person()
}

fn runtime_for(state: &AppState, person: &Person) -> Arc<PersonRuntime> {
    let mut map = state.runtimes.lock().unwrap();
    map.entry(person.id.clone())
        .or_insert_with(|| Arc::new(load_runtime(&state.cfg, person)))
        .clone()
}

pub fn serve(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    cfg.ensure_dirs()?;
    let people = PeopleState::load(&cfg);
    let state = Arc::new(AppState {
        cfg,
        people: Arc::new(Mutex::new(people)),
        runtimes: Arc::new(Mutex::new(HashMap::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
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
    /// The caller's `ergon_session` cookie, if they presented one.
    session: Option<String>,
}

/// Bodies are small JSON. A request claiming a larger one is dropped
/// before any allocation: an unbounded `vec![0u8; content_length]`
/// here once let a single header (Content-Length: 2^40, no body)
/// abort the whole server process.
const MAX_BODY_BYTES: usize = 1_048_576;
const MAX_HEAD_BYTES: usize = 32_768;

fn read_request(stream: &mut std::net::TcpStream) -> Option<Request> {
    // A TOTAL deadline for the whole request, not a per-read timeout:
    // a per-read clock lets a slow-loris drip headers forever, one
    // gap at a time. Before each read the socket timeout is re-armed
    // with whatever of the 30 seconds remains.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let arm = |stream: &std::net::TcpStream| -> Option<()> {
        let remain = deadline.checked_duration_since(std::time::Instant::now())?;
        stream.set_read_timeout(Some(remain)).ok()?;
        Some(())
    };
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    arm(stream)?;
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let raw_path = parts.next()?.to_string();
    let path = raw_path.split('?').next().unwrap_or("/").to_string();
    let mut content_length: usize = 0;
    let mut session: Option<String> = None;
    let mut head_bytes = request_line.len();
    loop {
        arm(stream)?;
        let mut line = String::new();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        head_bytes += n;
        if head_bytes > MAX_HEAD_BYTES {
            return None;
        }
        let lower = line.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
        if let Some(v) = lower.strip_prefix("cookie:") {
            for pair in v.split(';') {
                let pair = pair.trim();
                if let Some(id) = pair.strip_prefix("ergon_session=") {
                    let id = id.trim().to_string();
                    if !id.is_empty() {
                        session = Some(id);
                    }
                }
            }
        }
    }
    if content_length > MAX_BODY_BYTES {
        // A real 413, not a silent drop: the caller learns the limit
        // instead of staring at a dead connection. The in-flight body
        // is drained first (bounded — a few MB, within the deadline):
        // answering while the client's body is still queued gets the
        // response reset away before the client can read it.
        let mut to_drain = content_length.min(4 * 1024 * 1024);
        let mut sink = [0u8; 8192];
        while to_drain > 0 {
            if arm(stream).is_none() {
                break;
            }
            let want = to_drain.min(sink.len());
            match reader.read(&mut sink[..want]) {
                Ok(0) => break,
                Ok(n) => to_drain -= n,
                Err(_) => break,
            }
        }
        let body = format!(
            "{{\"error\": \"request body too large: the limit is {MAX_BODY_BYTES} bytes\"}}"
        );
        let response = format!(
            "HTTP/1.1 413 Content Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        return None;
    }
    arm(stream)?;
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).ok()?;
    }
    Some(Request {
        method,
        path,
        body,
        session,
    })
}

fn respond(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    respond_h(stream, status, content_type, body, &[])
}

fn respond_h(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in extra_headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

fn respond_json(stream: &mut std::net::TcpStream, status: u16, v: &Value) -> std::io::Result<()> {
    respond(stream, status, "application/json", &v.to_string())
}

fn handle_connection(mut stream: std::net::TcpStream, state: &AppState) -> std::io::Result<()> {
    // A stalled client must not hold its connection thread forever.
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let req = match read_request(&mut stream) {
        Some(r) => r,
        None => return Ok(()),
    };
    let root_cfg = &state.cfg;
    // The person this request acts for (session-pinned when the caller
    // has a session cookie, else the global active person).
    let person = effective_person(state, &req.session);
    let runtime = runtime_for(state, &person);
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            // First visit (or an unknown/forged/stale cookie): issue a
            // fresh server-generated session pinned to the current
            // active person, so this browser keeps its own person even
            // if another client switches the global one.
            if !session_known(state, &req.session) {
                let sid = new_session_id();
                insert_session(state, sid.clone(), person.id.clone());
                return respond_h(
                    &mut stream,
                    200,
                    "text/html; charset=utf-8",
                    ui::INDEX_HTML,
                    &[(
                        "Set-Cookie",
                        format!("ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"),
                    )],
                );
            }
            respond(&mut stream, 200, "text/html; charset=utf-8", ui::INDEX_HTML)
        }
        ("GET", "/api/status") => respond_json(
            &mut stream,
            200,
            &json!({
                "model_configured": root_cfg.api_key.is_some(),
                "model": root_cfg.model_id,
                "model_base_url": root_cfg.model_base_url,
                "person": person.to_json(),
            }),
        ),
        ("GET", "/api/people") => {
            let people = state.people.lock().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(person.id);
            respond_json(&mut stream, 200, &v)
        }
        ("POST", "/api/people") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("").trim().to_string();
            if name.is_empty() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "name is required"}),
                );
            }
            let role = parsed["role"].as_str().unwrap_or("").trim().to_string();
            let person = {
                let mut people = state.people.lock().unwrap();
                let p = people.add(&name, &role);
                people.save(root_cfg);
                p
            };
            // Create their dirs + seeded memory now, so the person is
            // real on disk from this moment.
            let _ = runtime_for(state, &person);
            respond_json(&mut stream, 201, &person.to_json())
        }
        ("POST", "/api/active-person") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"].as_str().unwrap_or("").to_string();
            {
                let people = state.people.lock().unwrap();
                if !people.people.iter().any(|p| p.id == id) {
                    return respond_json(
                        &mut stream,
                        404,
                        &json!({"error": "person not found"}),
                    );
                }
            }
            // Who does this switch move? A browser with a known session
            // moves ONLY its own session — never the global pointer
            // every other client inherits. A cookie-less API client
            // moves the global default (and is issued a session pinned
            // to it). A forged/unknown cookie is treated as no cookie:
            // the server issues its own id, never the client's.
            let mut set_cookie: Option<String> = None;
            if session_known(state, &req.session) {
                let sid = req.session.clone().unwrap();
                insert_session(state, sid, id.clone());
            } else {
                {
                    let mut people = state.people.lock().unwrap();
                    people.active = id.clone();
                    people.save(root_cfg);
                }
                let sid = new_session_id();
                insert_session(state, sid.clone(), id.clone());
                set_cookie = Some(format!(
                    "ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"
                ));
            }
            let people = state.people.lock().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(id);
            match set_cookie {
                Some(cookie) => respond_h(
                    &mut stream,
                    200,
                    "application/json",
                    &v.to_string(),
                    &[("Set-Cookie", cookie)],
                ),
                None => respond_json(&mut stream, 200, &v),
            }
        }
        ("GET", "/api/memory") => respond_json(
            &mut stream,
            200,
            &json!({"person_id": person.id, "content": runtime.cfg.read_memory()}),
        ),
        ("GET", "/api/threads") => {
            let store = runtime.store.lock().unwrap();
            respond_json(&mut stream, 200, &store.list_json())
        }
        ("POST", "/api/threads") => {
            let mut store = runtime.store.lock().unwrap();
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
            let cfg = &runtime.cfg;
            // The user's words are recorded FIRST, before any failure
            // path: a refused or failed run must never silently lose
            // the task the person typed (the no-key path used to drop
            // it, so the message vanished on reload).
            let (thread_id, history) = {
                let mut store = runtime.store.lock().unwrap();
                let tid = store.ensure_thread(parsed["thread_id"].as_str().unwrap_or(""));
                let history = store.history_json(&tid);
                store.push(&tid, "user", &message);
                (tid, history)
            };
            if cfg.api_key.is_none() {
                runtime
                    .store
                    .lock()
                    .unwrap()
                    .push(&thread_id, "error", NO_KEY_ERROR);
                return respond_json(
                    &mut stream,
                    503,
                    &json!({"error": NO_KEY_ERROR, "thread_id": thread_id}),
                );
            }
            match run_agent(cfg, &runtime.store, &thread_id, &message, &history) {
                Ok(_outcome) => {
                    let store = runtime.store.lock().unwrap();
                    let v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    respond_json(&mut stream, 200, &v)
                }
                Err(e) => {
                    runtime.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(&mut stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", path) if path.starts_with("/api/threads/") => {
            let id = path.trim_start_matches("/api/threads/");
            let store = runtime.store.lock().unwrap();
            match store.thread_json(id) {
                Some(v) => respond_json(&mut stream, 200, &v),
                None => respond_json(&mut stream, 404, &json!({"error": "thread not found"})),
            }
        }
        _ => respond_json(&mut stream, 404, &json!({"error": "not found"})),
    }
}
