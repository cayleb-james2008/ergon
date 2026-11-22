//! Ergon — an AI agent for every employee, personal to each person, that
//! learns how they work and finishes their digital work autonomously.
//!
//! Architecture: a single Rust binary serving an embedded web UI over a
//! hand-rolled HTTP server (std TcpListener). The agent loop talks to an
//! OpenAI-compatible chat-completions endpoint (OpenCode Go by default)
//! using a strict JSON tool protocol, executes tools for real against a
//! sandboxed workspace folder, and uses Rune scripts (see `skills`) for
//! the document-writing and memory-update skills.

pub mod library;
pub mod records;
pub mod skill_catalog;
pub mod skills;
pub mod ui;
pub mod update;

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

pub const DEFAULT_MODEL_BASE_URL: &str = "https://opencode.ai/zen/go/v1";
pub const DEFAULT_MODEL_ID: &str = "deepseek-v4.1-flash";

#[derive(Clone, Debug)]
pub struct DetectedGateway {
    pub base_url: String,
    pub model_id: String,
    pub label: String,
    pub models: Vec<String>,
}

impl DetectedGateway {
    pub fn to_json(&self) -> Value {
        json!({
            "base_url": self.base_url,
            "model_id": self.model_id,
            "label": self.label,
            "models": self.models,
        })
    }
}

/// Extract (host, port) from an http(s) URL authority.
fn parse_url_host_port(url: &str) -> Option<(String, u16)> {
    let (scheme, rest) = url.trim().split_once("://")?;
    let default_port = match scheme.to_ascii_lowercase().as_str() {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let authority = rest.split('/').next().unwrap_or("").trim();
    if authority.is_empty() {
        return None;
    }
    if let Some(after_open) = authority.strip_prefix('[') {
        let (host, tail) = after_open.split_once(']')?;
        let port = if let Some(p) = tail.strip_prefix(':') {
            p.parse::<u16>().ok()?
        } else {
            default_port
        };
        return Some((format!("[{}]", host.to_ascii_lowercase()), port));
    }
    if let Some((host, p)) = authority.rsplit_once(':') {
        let port = p.parse::<u16>().ok()?;
        Some((host.to_ascii_lowercase(), port))
    } else {
        Some((authority.to_ascii_lowercase(), default_port))
    }
}

/// Whether `url` targets loopback (`127.0.0.1`, `localhost`, or `[::1]`).
pub fn is_loopback_url(url: &str) -> bool {
    parse_url_host_port(url)
        .map(|(h, _)| matches!(h.as_str(), "127.0.0.1" | "localhost" | "[::1]"))
        .unwrap_or(false)
}

/// Fast non-blocking TCP reachability probe for loopback model URLs (250ms).
/// Ensures dead discard ports (e.g. `127.0.0.1:9` in unit/no-key tests) fail
/// closed while running local OpenAI-compatible gateways work without an API key.
pub fn loopback_tcp_reachable(url: &str) -> bool {
    let Some((host, port)) = parse_url_host_port(url) else {
        return false;
    };
    if !matches!(host.as_str(), "127.0.0.1" | "localhost" | "[::1]") {
        return false;
    }
    let ip: std::net::IpAddr = match host.as_str() {
        "[::1]" => std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        _ => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
    };
    let addr = std::net::SocketAddr::new(ip, port);
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(250)).is_ok()
}

/// Probe known running local OpenAI-compatible gateways in priority order.
/// Ollama (`11434`) is checked first because it is a pure stateless OpenAI JSON
/// endpoint that executes Ergon's sandboxed tool protocol directly without
/// running host-level CLI tools outside the workspace.
pub fn detect_local_gateways() -> Vec<DetectedGateway> {
    const CANDIDATES: [(&str, &str, &str); 3] = [
        (
            "http://127.0.0.1:11434/v1",
            "qwen2.5:3b",
            "Local Ollama (:11434)",
        ),
        (
            "http://127.0.0.1:8790/v1",
            "deepseek-v4.1-flash-free",
            "Local OpenCode Zen (:8790)",
        ),
        (
            "http://127.0.0.1:8789/v1",
            "gemini-3.8-flash-high",
            "Local Antigravity Bridge (:8789)",
        ),
    ];
    let mut found = Vec::new();
    for (base_url, preferred_model, label) in CANDIDATES {
        if !loopback_tcp_reachable(base_url) {
            continue;
        }
        let models_url = format!("{base_url}/models");
        let resp = ureq::get(&models_url)
            .timeout(std::time::Duration::from_millis(1200))
            .call();
        let Ok(r) = resp else { continue };
        let Ok(v) = r.into_json::<Value>() else { continue };
        let mut models: Vec<String> = v["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        if models.is_empty() {
            models.push(preferred_model.to_string());
        }
        let chosen = if models.iter().any(|m| m == preferred_model) {
            preferred_model.to_string()
        } else {
            models[0].clone()
        };
        found.push(DetectedGateway {
            base_url: base_url.to_string(),
            model_id: chosen,
            label: label.to_string(),
            models,
        });
    }
    found
}

#[derive(Clone, Debug)]
pub struct SavedModelConfig {
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
}

pub fn config_file_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

pub fn load_saved_model_config(data_dir: &Path) -> Option<SavedModelConfig> {
    let raw = std::fs::read_to_string(config_file_path(data_dir)).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let base = v["model_base_url"]
        .as_str()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())?;
    let model = v["model_id"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string());
    let key = v["api_key"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(SavedModelConfig {
        model_base_url: base,
        model_id: model,
        api_key: key,
    })
}

pub fn save_model_config(data_dir: &Path, saved: &SavedModelConfig) -> Result<(), String> {
    std::fs::create_dir_all(data_dir).map_err(|e| format!("create data dir: {e}"))?;
    let path = config_file_path(data_dir);
    let tmp = data_dir.join("config.json.tmp");
    let body = json!({
        "model_base_url": saved.model_base_url,
        "model_id": saved.model_id,
        "api_key": saved.api_key,
    })
    .to_string();
    std::fs::write(&tmp, body).map_err(|e| format!("write config.json: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, &path).map_err(|e| format!("save config.json: {e}"))?;
    Ok(())
}

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

        let env_base = std::env::var("ERGON_MODEL_BASE_URL")
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty());
        let env_model = std::env::var("ERGON_MODEL_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let env_key = std::env::var("ERGON_MODEL_API_KEY")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        let no_auto = std::env::var("ERGON_NO_AUTO_MODEL")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let saved = load_saved_model_config(&data_dir);

        let (model_base_url, model_id, api_key) =
            if env_base.is_some() || env_key.is_some() || saved.is_some() {
                let base = env_base
                    .or_else(|| saved.as_ref().map(|s| s.model_base_url.clone()))
                    .unwrap_or_else(|| DEFAULT_MODEL_BASE_URL.to_string());
                let model = env_model
                    .or_else(|| saved.as_ref().map(|s| s.model_id.clone()))
                    .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string());
                let key = env_key.or_else(|| saved.and_then(|s| s.api_key));
                (base, model, key)
            } else if !no_auto {
                let gateways = detect_local_gateways();
                if let Some(gw) = gateways.first() {
                    let model = env_model.unwrap_or_else(|| gw.model_id.clone());
                    (gw.base_url.clone(), model, None)
                } else {
                    (
                        DEFAULT_MODEL_BASE_URL.to_string(),
                        env_model.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
                        None,
                    )
                }
            } else {
                (
                    DEFAULT_MODEL_BASE_URL.to_string(),
                    env_model.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
                    None,
                )
            };

        Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url,
            model_id,
            api_key,
        }
    }

    /// True when the agent can reach a configured model: either an API key is
    /// present, or the base URL targets a real loopback OpenAI-compatible port
    /// (excluding RFC 863 discard port 9 used by offline/no-key tests).
    pub fn model_configured(&self) -> bool {
        self.api_key.is_some()
            || parse_url_host_port(&self.model_base_url)
                .map(|(h, port)| matches!(h.as_str(), "127.0.0.1" | "localhost" | "[::1]") && port != 9)
                .unwrap_or(false)
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
        .unwrap_or_else(|_| normalize_absolute(root));
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
                if !is_final || resolve_final {
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

/// Candidate file names around every document-extension occurrence
/// in a piece of text, longest first. Filenames in this product
/// routinely contain SPACES ("Weekly Report.md", every research file
/// is "Research - <topic>.md"), so a whitespace tokenizer cannot
/// represent them — instead each extension occurrence expands
/// leftwards over filename characters (letters, digits, spaces, and
/// `. _ - /`), and every word-boundary suffix is a candidate, longest
/// first: "wrote Weekly Report.md" yields ["wrote Weekly Report.md",
/// "Weekly Report.md", "Report.md"]. Callers resolve candidates
/// against the files that actually exist / were written; the longest
/// match wins. URLs are excluded.
pub fn mention_candidates(text: &str) -> Vec<Vec<String>> {
    const EXTS: [&str; 8] = ["md", "csv", "txt", "json", "html", "pdf", "docx", "xlsx"];
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '.' {
            // Does an extension start here, ending on a word boundary?
            let rest: String = chars[i + 1..].iter().take(4).collect();
            let ext = EXTS.iter().find(|e| {
                rest.to_lowercase().starts_with(**e)
                    && chars
                        .get(i + 1 + e.len())
                        .map(|c| !c.is_alphanumeric())
                        .unwrap_or(true)
            });
            if let Some(ext) = ext {
                let end = i + 1 + ext.len(); // exclusive char index just past the extension
                // Expand left over filename characters.
                let mut start = i;
                while start > 0 {
                    let c = chars[start - 1];
                    if c.is_alphanumeric() || matches!(c, ' ' | '.' | '_' | '-' | '/') {
                        start -= 1;
                    } else {
                        break;
                    }
                }
                let raw_full: String = chars[start..end].iter().collect();
                // A URL is not a workspace file: the expansion abuts
                // a scheme colon ("https://…") or begins with "//".
                let is_url = raw_full.starts_with("//")
                    || raw_full.contains("://")
                    || (start > 0 && chars[start - 1] == ':');
                let raw = raw_full.trim().trim_start_matches(['.', '/']).to_string();
                if !raw.is_empty() && !is_url {
                    let mut cands = vec![raw.clone()];
                    for (j, ch) in raw.char_indices() {
                        if ch == ' ' {
                            let suffix = raw[j + 1..].trim_start().to_string();
                            if !suffix.is_empty() {
                                cands.push(suffix);
                            }
                        }
                    }
                    out.push(cands);
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// The files a skill result says it produced: for each extension
/// occurrence, the longest candidate that actually exists in the
/// workspace (falling back to the longest candidate — the skill's own
/// claim, checked again by the gate if wrong).
pub fn extract_written(result_text: &str, workspace: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for cands in mention_candidates(result_text) {
        let found = cands
            .iter()
            .find(|c| sandbox_path(workspace, c).map(|p| p.exists()).unwrap_or(false))
            .cloned()
            .or_else(|| cands.first().cloned());
        if let Some(f) = found {
            if !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// Check a "done" summary's file mentions against reality. Returns
/// (missing, present_but_unwritten): a mention resolves as written
/// when it (or its basename) matches a file this run wrote; as
/// present when it exists in the workspace; as explained when it
/// names a file this run moved away (the honest "renamed a.txt to
/// b.txt" phrasing); otherwise it is missing.
pub fn check_mentions(
    summary: &str,
    workspace: &Path,
    written: &[String],
    consumed: &[String],
) -> (Vec<String>, Vec<String>) {
    let basename = |p: &str| {
        p.rsplit('/')
            .next()
            .unwrap_or(p)
            .to_string()
    };
    let written_bases: Vec<String> = written.iter().map(|w| basename(w)).collect();
    let mut missing: Vec<String> = Vec::new();
    let mut unwritten: Vec<String> = Vec::new();
    for cands in mention_candidates(summary) {
        let shortest = cands.last().cloned().unwrap_or_default();
        let is_written = cands
            .iter()
            .any(|c| written.contains(c) || written_bases.contains(&basename(c)));
        if is_written {
            continue;
        }
        let present = cands.iter().find(|c| {
            sandbox_path(workspace, c)
                .map(|p| p.exists())
                .unwrap_or(false)
        });
        if let Some(found) = present {
            if !unwritten.contains(found) {
                unwritten.push(found.clone());
            }
            continue;
        }
        // Basename presence anywhere in the workspace also counts as
        // present (a summary naming "Report.md" for docs/Report.md).
        let base_present = workspace_has_basename(workspace, &basename(&shortest), 0);
        if base_present {
            if !unwritten.contains(&shortest) {
                unwritten.push(shortest);
            }
            continue;
        }
        let explained = cands
            .iter()
            .any(|c| consumed.contains(c))
            || consumed.iter().any(|c| basename(c) == basename(&shortest));
        if explained {
            continue;
        }
        if !missing.contains(&shortest) {
            missing.push(shortest);
        }
    }
    (missing, unwritten)
}

fn workspace_has_basename(dir: &Path, base: &str, depth: usize) -> bool {
    if depth > 4 {
        return false;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in rd.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == base {
            return true;
        }
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
            && !name.starts_with('.')
            && workspace_has_basename(&entry.path(), base, depth + 1)
        {
            return true;
        }
    }
    false
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
    let mut in_tag = false;
    let mut skip_until: Option<&str> = None;
    for (i, c) in html.char_indices() {
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
    /// Store files backed up at load (named in GET /api/status).
    pub recovered: Vec<String>,
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
            recovered: Vec::new(),
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

/// Backs up a corrupt store and returns its file name, for the
/// "earlier conversations were moved" notice (GET /api/status).
pub fn backed_up_name(path: &Path) -> Option<String> {
    backup_corrupt(path)?;
    path.file_name().map(|n| n.to_string_lossy().into_owned())
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
                return PeopleState {
                    recovered: backed_up_name(&path).into_iter().collect(),
                    ..PeopleState::default()
                };
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
                        return PeopleState {
                            recovered: backed_up_name(&path).into_iter().collect(),
                            ..PeopleState::default()
                        };
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
    /// The backed-up file name when this store started fresh (E5).
    pub recovered: Option<String>,
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
            recovered: None,
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
                        store.recovered = backed_up_name(path);
                    }
                },
                Err(_) => {
                    store.recovered = backed_up_name(path);
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

    /// Rewrites the first step message in a thread whose text is exactly
    /// `old` (a parked-delete note) to `new`, and persists it (ER1).
    pub fn settle_note(&mut self, thread_id: &str, old: &str, new: &str) -> bool {
        let Some(t) = self.threads.iter_mut().find(|t| t.id == thread_id) else {
            return false;
        };
        let Some(m) = t
            .messages
            .iter_mut()
            .find(|m| m.role == "step" && m.content == old)
        else {
            return false;
        };
        m.content = new.to_string();
        self.save();
        true
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

/// One model call. Carries the proven wire requirements: bearer key (when set
/// or required for non-loopback endpoints), `x-opencode-session` header, and a
/// browser-like User-Agent. Loopback gateways work without a bearer key.
pub fn call_model(cfg: &Config, session: &str, messages: &[Value]) -> Result<Value, String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    let url = format!("{}/chat/completions", cfg.model_base_url);
    let mut payload = json!({
        "model": cfg.model_id,
        "messages": messages,
        "temperature": 0.2,
    });
    if cfg.model_base_url.contains(":11434") {
        payload["response_format"] = json!({"type": "json_object"});
    }
    let mut req = ureq::post(&url)
        .set("x-opencode-session", session)
        .set("User-Agent", BROWSER_UA)
        .timeout(std::time::Duration::from_secs(120));
    if let Some(key) = cfg.api_key.as_deref().filter(|k| !k.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req.send_json(payload);
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
        Value::Object(_) => Ok(normalize_model_reply(content.clone())),
        _ => Err(format!(
            "model response had no message content: {}",
            truncate(&v.to_string(), 400)
        )),
    }
}

fn normalize_model_reply(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        // Common local-model / OpenAI-function-call aliases:
        // {"name": "write_file", "arguments": {...}} -> {"tool": "write_file", "args": {...}}
        if !obj.contains_key("tool") {
            if let Some(name) = obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()) {
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
            && obj.get("summary").and_then(|s| s.as_str()).is_some_and(|s| !s.trim().is_empty())
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

/// The wider context the completion-era tools need: the company
/// library (shared, mutable — agents publish into it), the root data
/// dir (where the library persists) and the person whose agent is
/// acting (the author of anything it publishes).
#[derive(Clone)]
pub struct ToolCtx {
    pub library: Arc<Mutex<library::Library>>,
    pub root_data_dir: PathBuf,
    pub person_id: String,
}

/// The full tool dispatcher the agent loop uses: the library/suite/
/// decision tools first, then the base tools (`execute_tool`, whose
/// signature the test suite pins), then the person's installed
/// library skills by item id.
pub fn execute_tool_full(
    cfg: &Config,
    ctx: &ToolCtx,
    name: &str,
    args: &Value,
) -> Result<String, String> {
    match name {
        "browse_library" => {
            let query = args["query"].as_str().unwrap_or("");
            let lib = ctx.library.lock().unwrap();
            let suite = library::load_suite(&cfg.data_dir);
            let items = lib.search(query);
            if items.is_empty() {
                return Ok("(no library items match)".to_string());
            }
            let lines: Vec<String> = items
                .iter()
                .map(|i| {
                    let state = if i.builtin {
                        "built-in".to_string()
                    } else if suite.installed.contains(&i.id) {
                        "in your suite".to_string()
                    } else {
                        "available".to_string()
                    };
                    format!(
                        "- {} [{}] {} — {} (by {}, v{}, {})",
                        i.id, i.kind, i.name, i.description, i.author, i.version, state
                    )
                })
                .collect();
            Ok(format!(
                "The company library ({} items):\n{}",
                items.len(),
                lines.join("\n")
            ))
        }
        "install_item" => {
            let id = args["item_id"]
                .as_str()
                .ok_or("install_item needs args.item_id")?;
            let lib = ctx.library.lock().unwrap();
            let item = lib
                .get(id)
                .ok_or_else(|| format!("no library item with id '{id}'"))?;
            if item.builtin {
                return Ok(format!(
                    "'{}' is built into every agent already — nothing to install.",
                    item.name
                ));
            }
            let installed_now = library::with_suite_lock(|| {
                let mut suite = library::load_suite(&cfg.data_dir);
                let changed = suite.install(id);
                if changed {
                    library::save_suite(&cfg.data_dir, &suite);
                }
                changed
            });
            if installed_now {
                Ok(format!(
                    "installed '{}' ({}) into your suite — you can call it in this task too, and it stays in your suite for future tasks",
                    item.name, item.kind
                ))
            } else {
                Ok(format!("'{}' is already in your suite.", item.name))
            }
        }
        "publish_skill" => {
            let name = args["name"]
                .as_str()
                .ok_or("publish_skill needs args.name")?;
            let description = args["description"].as_str().unwrap_or("");
            let source = args["source"]
                .as_str()
                .ok_or("publish_skill needs args.source (the Rune script)")?;
            let params = string_list(&args["params"]);
            if params.len() > 5 {
                return Err("library skills take at most 5 parameters".to_string());
            }
            // The compile gate: a script that does not build against
            // the real host context is refused, never stored — the
            // library must not fill up with skills that cannot run.
            skills::check_script(cfg, source)?;
            let mut lib = ctx.library.lock().unwrap();
            let item = lib.publish_skill(
                name,
                description,
                params,
                source.to_string(),
                &ctx.person_id,
            )?;
            lib.save(&ctx.root_data_dir);
            Ok(format!(
                "published skill '{}' (id {}, v{}) to the company library — every person's agent can now install it",
                item.name, item.id, item.version
            ))
        }
        "delete_file" => {
            let rel = args["path"]
                .as_str()
                .ok_or("delete_file needs args.path")?;
            // Consequential: the file is NOT deleted here. The action
            // parks as a decision for the person; approval executes
            // it for real. The agent is told plainly that it is
            // parked, so its summary cannot claim a deletion that has
            // not happened.
            // The final component is not resolved: a symlink to a
            // file is itself a file that exists and can be parked
            // for deletion (of the link, never its target).
            let full = sandbox_path_resolving(&cfg.workspace_dir, rel, false)?;
            if full.symlink_metadata().is_err() {
                return Err(format!("delete_file: no such file: {rel}"));
            }
            let thread_id = args["_thread"].as_str().unwrap_or("");
            if let Some(existing_id) = records::find_pending_delete(&cfg.data_dir, rel) {
                return Ok(format!(
                    "already parked for your person's approval (decision {existing_id}): {rel} is already waiting in their decisions inbox. Do NOT call delete_file again for {rel}; finish now with {{\"done\": true, \"summary\": \"Parked deletion of {rel} for your approval.\"}}."
                ));
            }
            let id = records::add_decision(
                &cfg.data_dir,
                "delete",
                &format!("Delete {rel}"),
                Some(rel),
                thread_id,
            );
            Ok(format!(
                "parked for your person's approval (decision {id}): deleting is a consequential action, so {rel} has NOT been deleted yet. It will be deleted only if they approve it in their decisions inbox. Do NOT call delete_file again for {rel}. Continue with any remaining work or reply now with {{\"done\": true, \"summary\": \"...\"}} noting that the deletion is awaiting their decision."
            ))
        }
        "flag_decision" => {
            let summary = args["summary"]
                .as_str()
                .ok_or("flag_decision needs args.summary")?;
            let thread_id = args["_thread"].as_str().unwrap_or("");
            let id = records::add_decision(&cfg.data_dir, "question", summary, None, thread_id);
            Ok(format!(
                "raised as decision {id} in your person's decisions inbox — they will see it and settle it there. Continue with the rest of the task."
            ))
        }
        _ => {
            // An installed library skill, called by its item id.
            let suite = library::load_suite(&cfg.data_dir);
            if suite.installed.iter().any(|i| i == name) {
                // Clone what the run needs and DROP the library lock
                // before executing: a skill can run for seconds (and
                // fetch over the network), and holding the
                // company-wide mutex across it froze every
                // library-dependent surface for its whole duration.
                let runnable = {
                    let lib = ctx.library.lock().unwrap();
                    lib.get(name).and_then(|item| {
                        if item.kind == "skill" && !item.builtin {
                            let values: Vec<String> = item
                                .params
                                .iter()
                                .map(|p| {
                                    args[p.as_str()]
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_default()
                                })
                                .collect();
                            Some((item.source.clone(), values))
                        } else {
                            None
                        }
                    })
                };
                if let Some((source, values)) = runnable {
                    return skills::run_library_skill(cfg, &source, &values);
                }
            }
            execute_tool(cfg, name, args)
        }
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

fn system_prompt(cfg: &Config, suite_items: &[library::LibraryItem]) -> String {
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
    // The person's suite: installed library skills are callable tools
    // (by item id, with their declared parameters); installed
    // subagents are delegable by id. Apps are run by the person from
    // their suite panel, not called as tools.
    let mut suite_block = String::new();
    let skills: Vec<&library::LibraryItem> = suite_items
        .iter()
        .filter(|i| i.kind == "skill" && !i.builtin)
        .collect();
    if !skills.is_empty() {
        suite_block.push_str("\n         LIBRARY SKILLS IN THIS PERSON'S SUITE (call like any tool, by id):\n");
        for s in skills {
            let params = s
                .params
                .iter()
                .map(|p| format!("\"{p}\": \"...\""))
                .collect::<Vec<_>>()
                .join(", ");
            suite_block.push_str(&format!(
                "         - {} {{{}}} — {}\n",
                s.id, params, s.description
            ));
        }
    }
    let subagents: Vec<&library::LibraryItem> = suite_items
        .iter()
        .filter(|i| i.kind == "subagent")
        .collect();
    if !subagents.is_empty() {
        suite_block.push_str("\n         SUBAGENTS YOU CAN DELEGATE TO (via the delegate tool):\n");
        for s in subagents {
            suite_block.push_str(&format!("         - {} — {}\n", s.id, s.description));
        }
    }
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
         - update_memory {{\"note\": \"...\"}} — record something you learned about this person (a preference or fact) into their long-term memory (a Rune skill).\n\
         - browse_library {{\"query\": \"spreadsheet\"}} — search the company-wide library of skills, subagents and apps shared by every person's agent.\n\
         - install_item {{\"item_id\": \"daily-log\"}} — add a library item to this person's suite, so it is yours to use from the next task on.\n\
         - publish_skill {{\"name\": \"...\", \"description\": \"...\", \"params\": [\"entry\"], \"source\": \"<Rune script>\"}} — contribute a reusable skill you wrote to the company library, so every person's agent can use it. Library skills are Rune scripts whose main takes the declared params as strings in order and returns a string (or \"file:<name>\" after writing a file through host_workspace_write). A script that does not compile is refused.\n\
         - delegate {{\"subagent\": \"researcher\", \"task\": \"...\"}} — hand a focused sub-task to a specialist subagent from the suite; it works with the same tools and returns its result to you.\n\
         - delete_file {{\"path\": \"old.txt\"}} — CONSEQUENTIAL: deleting is permanent, so this does not delete immediately — it parks the deletion as a decision for your person to approve. Never claim a file was deleted when it was only parked.\n\
         - flag_decision {{\"summary\": \"...\"}} — raise a question as a decision in your person's inbox when a choice is genuinely theirs to make.{suite_block}\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. When you learn a durable preference or fact about this person, record it with update_memory before finishing. When you build something reusable, publish it to the library so the whole company benefits. Finish with done only when the task is actually complete.\n\n\
         PREFERENCES AND FACTS THIS PERSON HAS TAUGHT YOU — apply them in every task:\n{prefs_block}\n\n\
         WHAT YOU KNOW ABOUT THIS PERSON (their memory file — treat it as true and use it):\n{memory}"
    )
}

pub struct AgentOutcome {
    pub summary: String,
    pub steps: Vec<String>,
}

/// The files one run actually wrote — the evidence a "done" summary's
/// named files are checked against. Not a count: WHICH files, so a
/// summary claiming a file this run never touched is visible as
/// exactly that. `consumed` are the move sources: files this run
/// legitimately made disappear. Shared with subagent runs, whose
/// writes are the parent run's work product too.
#[derive(Default)]
pub struct WriteLedger {
    pub written: Vec<String>,
    pub consumed: Vec<String>,
    /// Tool steps that succeeded (a failed step is not work, EB2b).
    pub ok_steps: usize,
}

fn credit_writes(
    ledger: &mut WriteLedger,
    cfg: &Config,
    tool: &str,
    args: &Value,
    result_text: &str,
    dynamic_skills: &std::collections::HashSet<String>,
) {
    match tool {
        "write_file" => {
            if let Some(p) = args["path"].as_str() {
                ledger.written.push(p.to_string());
            }
        }
        "move_file" => {
            if let Some(p) = args["to"].as_str() {
                ledger.written.push(p.to_string());
            }
            if let Some(p) = args["from"].as_str() {
                ledger.consumed.push(p.to_string());
            }
        }
        // A memory update is real work product too: the summary may
        // honestly name memory.md (it lives in the data dir, not the
        // workspace).
        "update_memory" => {
            ledger.written.push("memory.md".to_string());
        }
        // The Rune skills name the file they produced in their
        // result text.
        "write_document" | "make_spreadsheet" | "research" => {
            ledger
                .written
                .extend(extract_written(result_text, &cfg.workspace_dir));
        }
        // A library skill from the person's suite: credit only files
        // its result names that actually exist (its result text is a
        // claim, checked like any other).
        other if dynamic_skills.contains(other) => {
            for cands in mention_candidates(result_text) {
                if let Some(found) = cands.iter().find(|c| {
                    sandbox_path(&cfg.workspace_dir, c)
                        .map(|p| p.exists())
                        .unwrap_or(false)
                }) {
                    if !ledger.written.contains(found) {
                        ledger.written.push(found.clone());
                    }
                }
            }
        }
        _ => {}
    }
}

/// The system prompt for a subagent run: the profile's own
/// instructions, the same JSON tool protocol, the person's standing
/// preferences, and the working toolset — minus the tools that belong
/// to the main agent alone (delegating further, publishing, and the
/// consequential pair).
fn subagent_system_prompt(
    cfg: &Config,
    profile: &library::LibraryItem,
    suite_items: &[library::LibraryItem],
) -> String {
    let memory = cfg.read_memory();
    let prefs = standing_preferences(&memory);
    let prefs_block = if prefs.is_empty() {
        "(none yet)".to_string()
    } else {
        prefs
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut skills_block = String::new();
    for s in suite_items.iter().filter(|i| i.kind == "skill" && !i.builtin) {
        let params = s
            .params
            .iter()
            .map(|p| format!("\"{p}\": \"...\""))
            .collect::<Vec<_>>()
            .join(", ");
        skills_block.push_str(&format!(
            "\n         - {} {{{}}} — {}",
            s.id, params, s.description
        ));
    }
    format!(
        "You are {}, a specialist subagent working for a person's main Ergon agent. {}\n\n\
         PROTOCOL: Reply with exactly ONE JSON object per message, nothing else.\n\
         - To use a tool: {{\"tool\": \"<name>\", \"args\": {{...}}}}\n\
         - When your sub-task is complete: {{\"done\": true, \"summary\": \"<what you did and found>\"}}\n\n\
         TOOLS:\n\
         - list_dir {{\"path\": \".\"}} — list files in the workspace.\n\
         - read_file {{\"path\": \"notes.txt\"}} — read a workspace file.\n\
         - write_file {{\"path\": \"out.txt\", \"content\": \"...\"}} — write a workspace file.\n\
         - fetch_url {{\"url\": \"https://...\"}} — fetch a URL and read its text.\n\
         - write_document {{\"title\": \"Summary\", \"sections\": [\"...\"]}} — produce a finished markdown document.\n\
         - make_spreadsheet {{\"title\": \"Budget\", \"columns\": [\"Item\", \"Cost\"], \"rows\": [[\"Coffee\", \"4.50\"]]}} — produce a finished CSV spreadsheet.\n\
         - research {{\"topic\": \"...\", \"urls\": [\"https://...\"]}} — compile research notes from several pages.\n\
         - search_files {{\"query\": \"...\", \"path\": \".\"}} — search workspace text files.\n\
         - move_file {{\"from\": \"a.txt\", \"to\": \"archive/a.txt\"}} — move or rename a workspace file.\n\
         - update_memory {{\"note\": \"...\"}} — record something learned about this person.{skills}\n\n\
         RULES: Use tools to do the real work; never claim work you did not do through a tool. Your summary goes back to the main agent, so state plainly what you produced and where.\n\n\
         PREFERENCES AND FACTS THIS PERSON HAS TAUGHT THEIR AGENT:\n{prefs}",
        profile.name, profile.instructions, prefs = prefs_block, skills = skills_block
    )
}

/// Live execution state for an in-flight agent run so the UI can stream tool
/// steps in real time before the final HTTP response returns.
#[derive(Clone, Debug)]
pub struct LiveRun {
    pub thread_id: String,
    pub task: String,
    pub started_ms: u64,
    pub steps: Vec<String>,
}

pub type LiveRuns = Arc<Mutex<HashMap<String, LiveRun>>>;

fn push_live_step(live: Option<(&LiveRuns, &str)>, step: String) {
    if let Some((runs, person_id)) = live {
        if let Ok(mut map) = runs.lock() {
            if let Some(run) = map.get_mut(person_id) {
                run.steps.push(step);
            }
        }
    }
}

/// One agent loop: model → tool → model → … until done or the step
/// cap. Steps are pushed into the thread as they happen (subagent
/// steps are labelled with the subagent's name). Returns the done
/// summary. The honesty gate and the work log belong to the top-level
/// run (`run_agent`), not to subagent runs.
#[allow(clippy::too_many_arguments)]
fn run_loop(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    system: &str,
    task: &str,
    history: &[Value],
    allow_delegate: bool,
    step_cap: usize,
    label: Option<&str>,
    ledger: &mut WriteLedger,
    dynamic_skills: &mut std::collections::HashSet<String>,
    suite_items: &[library::LibraryItem],
    budget: &mut usize,
    session_tag: &str,
    live: Option<(&LiveRuns, &str)>,
) -> Result<(String, Vec<String>), String> {
    // Subagent runs get their own provider session: if the provider
    // keys conversation state on the session header, the two
    // personas must not interleave in one server-side conversation.
    let session = if session_tag.is_empty() {
        format!("ergon-{thread_id}")
    } else {
        format!("ergon-{thread_id}-{session_tag}")
    };
    let mut messages: Vec<Value> = vec![json!({"role": "system", "content": system})];
    messages.extend(history.iter().cloned());
    messages.push(json!({"role": "user", "content": task}));
    let mut steps: Vec<String> = Vec::new();

    // Two caps, both enforced: this loop's own step cap, and the
    // whole run's shared model-call budget. A delegate call used to
    // cost the parent 1 step and buy a fresh 12 — delegation
    // multiplied the cap (24 delegations ≈ 312 model calls in one
    // task). Now every call, parent or subagent, draws from the
    // same budget of STEP_CAP.
    let mut local_steps = 0usize;
    let mut json_retries = 0usize;
    loop {
        if local_steps >= step_cap || *budget >= STEP_CAP {
            break;
        }
        local_steps += 1;
        *budget += 1;
        let reply = match call_model(cfg, &session, &messages) {
            Ok(v) => v,
            Err(e) if e.starts_with("model reply was not a JSON object:") && json_retries < 2 => {
                json_retries += 1;
                messages.push(json!({
                    "role": "user",
                    "content": "Protocol reminder: reply with ONLY one valid JSON object — either {\"tool\": \"<name>\", \"args\": {...}} (e.g. {\"tool\": \"write_file\", \"args\": {\"path\": \"...\", \"content\": \"...\"}}) or {\"done\": true, \"summary\": \"...\"}."
                }));
                continue;
            }
            Err(e) => return Err(e),
        };
        if reply["done"].as_bool() == Some(true) {
            let summary = reply["summary"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("Task complete.")
                .to_string();
            return Ok((summary, steps));
        }
        let tool = match reply["tool"].as_str() {
            Some(t) if !t.trim().is_empty() => t.to_string(),
            _ if json_retries < 2 => {
                json_retries += 1;
                messages.push(json!({ "role": "assistant", "content": reply.to_string() }));
                messages.push(json!({
                    "role": "user",
                    "content": "Protocol reminder: wrap tool calls with a \"tool\" key and an \"args\" object, like {\"tool\": \"write_file\", \"args\": {\"path\": \"...\", \"content\": \"...\"}}, or finish with {\"done\": true, \"summary\": \"...\"}."
                }));
                continue;
            }
            _ => {
                return Err(format!(
                    "model reply was neither a tool call nor done: {}",
                    truncate(&reply.to_string(), 300)
                ));
            }
        };
        let mut args = reply["args"].clone();

        let (result_text, ok) = if tool == "delegate" {
            if !allow_delegate {
                (
                    "error: subagents cannot delegate further — do this part yourself".to_string(),
                    false,
                )
            } else {
                run_delegation(
                    cfg, ctx, store, thread_id, &args, ledger, dynamic_skills, suite_items,
                    budget, live,
                )
            }
        } else if !allow_delegate
            && matches!(
                tool.as_str(),
                "publish_skill" | "install_item" | "delete_file" | "flag_decision"
                    | "browse_library"
            )
        {
            // The subagent toolset restriction is enforced HERE, not
            // just written in its prompt: a subagent must not publish
            // to the company library, change the suite, or park
            // decisions in the person's inbox under the main agent's
            // authorship. (Deletion itself was never reachable from
            // any loop — it only executes on human approval.)
            (
                format!(
                    "error: subagents cannot use {tool} — that belongs to the main agent. Do this part with your own tools, or say in your summary that the main agent should do it."
                ),
                false,
            )
        } else {
            // Consequential tools carry the originating thread, so a
            // settled decision can report back into it.
            if matches!(tool.as_str(), "delete_file" | "flag_decision") {
                args["_thread"] = json!(thread_id);
            }
            match execute_tool_full(cfg, ctx, &tool, &args) {
                Ok(r) => (r, true),
                Err(e) => (format!("error: {e}"), false),
            }
        };
        if ok {
            ledger.ok_steps += 1;
            credit_writes(ledger, cfg, &tool, &args, &result_text, dynamic_skills);
            // A skill installed mid-run is callable from the very
            // next step (the dispatcher re-reads the suite) — so the
            // honesty gate must credit it too, or an honest run gets
            // a false "this run did not write" note.
            if tool == "install_item" {
                if let Some(id) = args["item_id"].as_str() {
                    dynamic_skills.insert(id.to_string());
                }
            }
        }
        let step_line = match label {
            Some(l) => format!("Tool: {tool} (via {l})"),
            None => format!("Tool: {tool}"),
        };
        steps.push(step_line.clone());
        let rendered_step = format!("{step_line}\n{}", truncate(&result_text, 500));
        push_live_step(live, rendered_step.clone());
        store.lock().unwrap().push(thread_id, "step", &rendered_step);
        // Guard against small local models looping on delete_file after a file
        // has already been parked for approval in this run.
        if tool == "delete_file" && ok && result_text.starts_with("already parked for your person's approval") {
            let rel = args["path"].as_str().unwrap_or("the file");
            return Ok((
                format!("Parked deletion of {rel} for your approval in Today's desk. The file has not been deleted yet."),
                steps,
            ));
        }
        messages.push(json!({"role": "assistant", "content": reply.to_string()}));
        messages.push(json!({
            "role": "user",
            "content": format!("Tool result for {tool}: {}", truncate(&result_text, 6000)),
        }));
    }
    Err(if *budget >= STEP_CAP {
        format!(
            "agent stopped after {STEP_CAP} steps (the run's total budget, shared with its subagents) without finishing — the task may be blocked or too large for one run."
        )
    } else {
        format!(
            "agent stopped after {step_cap} steps without finishing — the task may be blocked or too large for one run."
        )
    })
}

/// The delegate tool: run one suite subagent on a focused sub-task.
/// Its steps land in the same thread (labelled), its writes credit
/// the parent run's ledger, and its summary returns as the tool
/// result the main agent continues from.
#[allow(clippy::too_many_arguments)]
fn run_delegation(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    args: &Value,
    ledger: &mut WriteLedger,
    dynamic_skills: &mut std::collections::HashSet<String>,
    suite_items: &[library::LibraryItem],
    budget: &mut usize,
    live: Option<(&LiveRuns, &str)>,
) -> (String, bool) {
    let sub_id = args["subagent"].as_str().unwrap_or("");
    let task = args["task"].as_str().unwrap_or("");
    if sub_id.is_empty() || task.is_empty() {
        return (
            "error: delegate needs args.subagent and args.task".to_string(),
            false,
        );
    }
    let Some(profile) = suite_items
        .iter()
        .find(|i| i.id == sub_id && i.kind == "subagent")
        .cloned()
    else {
        return (
            format!(
                "error: no subagent '{sub_id}' in this person's suite — delegate only to a subagent the suite carries"
            ),
            false,
        );
    };
    let delegated_line = format!("Delegated to {}: {}", profile.name, truncate(task, 300));
    push_live_step(live, delegated_line.clone());
    store.lock().unwrap().push(thread_id, "step", &delegated_line);
    let system = subagent_system_prompt(cfg, &profile, suite_items);
    match run_loop(
        cfg,
        ctx,
        store,
        thread_id,
        &system,
        task,
        &[],
        false,
        12,
        Some(&profile.name),
        ledger,
        dynamic_skills,
        suite_items,
        budget,
        &profile.id,
        live,
    ) {
        Ok((summary, _)) => (format!("{} reports: {}", profile.name, summary), true),
        Err(e) => (format!("error: subagent {} failed: {e}", profile.name), false),
    }
}

/// Run one task to completion for the effective person: the loop,
/// then the honesty gate on its summary, the work log entry, and an
/// offers scan — the records the brief, the graph and the offers
/// engine are built from.
/// Agent runs in progress, so an update never restarts the server under one.
static RUNS_IN_FLIGHT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct RunGuard {
    live: Option<(LiveRuns, String)>,
}

impl RunGuard {
    fn new(live: Option<(&LiveRuns, &str)>, thread_id: &str, task: &str, started_ms: u64) -> RunGuard {
        RUNS_IN_FLIGHT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let stored = live.map(|(runs, pid)| {
            if let Ok(mut map) = runs.lock() {
                map.insert(
                    pid.to_string(),
                    LiveRun {
                        thread_id: thread_id.to_string(),
                        task: task.to_string(),
                        started_ms,
                        steps: Vec::new(),
                    },
                );
            }
            (runs.clone(), pid.to_string())
        });
        RunGuard { live: stored }
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        RUNS_IN_FLIGHT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if let Some((runs, pid)) = &self.live {
            if let Ok(mut map) = runs.lock() {
                map.remove(pid);
            }
        }
    }
}

pub fn run_agent(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    task: &str,
    history: &[Value],
) -> Result<AgentOutcome, String> {
    run_agent_live(cfg, ctx, store, thread_id, task, history, None)
}

pub fn run_agent_live(
    cfg: &Config,
    ctx: &ToolCtx,
    store: &SharedStore,
    thread_id: &str,
    task: &str,
    history: &[Value],
    live: Option<(&LiveRuns, &str)>,
) -> Result<AgentOutcome, String> {
    if !cfg.model_configured() {
        return Err(NO_KEY_ERROR.to_string());
    }
    // Deletes parked after this instant belong to this run (EB1).
    let run_started = records::now_millis();
    let _in_flight = RunGuard::new(live, thread_id, task, run_started);
    let suite = library::load_suite(&cfg.data_dir);
    let suite_items = {
        let lib = ctx.library.lock().unwrap();
        suite.resolved(&lib)
    };
    let mut dynamic_skills: std::collections::HashSet<String> = suite_items
        .iter()
        .filter(|i| i.kind == "skill" && !i.builtin)
        .map(|i| i.id.clone())
        .collect();
    let system = system_prompt(cfg, &suite_items);
    let mut ledger = WriteLedger::default();
    let mut budget = 0usize;
    let (summary, steps) = match run_loop(
        cfg,
        ctx,
        store,
        thread_id,
        &system,
        task,
        history,
        true,
        STEP_CAP,
        None,
        &mut ledger,
        &mut dynamic_skills,
        &suite_items,
        &mut budget,
        "",
        live,
    ) {
        Ok(v) => v,
        Err(e) => {
            // A run that stopped early still did visible things
            // (files written before the cap). Record it as
            // unfinished work — Today shows it; the brief and the
            // offers engine count only finished work.
            records::append_work(
                cfg,
                task,
                &format!("Did not finish — {e}"),
                &ledger.written,
                thread_id,
                false,
            );
            return Err(e);
        }
    };
    store
        .lock()
        .unwrap()
        .push(thread_id, "assistant", &summary);
    // Honesty gate, checked against the run's own write ledger —
    // WHICH files this run wrote, never the summary's verbs or nouns
    // (verb/noun lists flagged honest inline answers and waved
    // through claims about files the run never touched). Mentions are
    // matched against known files with spaced-name-aware candidates:
    //  1. a named file that does not exist at all is a hard
    //     discrepancy — caution, in the open;
    //  2. a named file that exists but this run did not write
    //     gets a factual ledger line (what the run DID write),
    //     so a pre-existing file claimed as the deliverable is
    //     visible without accusing an honest input reference.
    // Both lines are emitted when both apply.
    let (missing_named, unwritten_named) = check_mentions(
        &summary,
        &cfg.workspace_dir,
        &ledger.written,
        &ledger.consumed,
    );
    if !missing_named.is_empty() {
        store.lock().unwrap().push(
            thread_id,
            "step",
            &format!(
                "Note: the summary names {}, which is not in the workspace — treat the summary above with caution.",
                missing_named.join(", ")
            ),
        );
    }
    if !unwritten_named.is_empty() {
        let wrote = if ledger.written.is_empty() {
            "no files".to_string()
        } else {
            ledger.written.join(", ")
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
    // A deletion is only ever parked for approval. Every delete parked in
    // THIS run gets one factual line after the summary, whatever the
    // summary says: it has not been deleted and waits in Decisions.
    for d in records::load_decisions(&cfg.data_dir) {
        let parked_now = d["status"] == "pending"
            && d["kind"] == "delete"
            && d["thread_id"].as_str() == Some(thread_id)
            && d["ts"].as_u64().unwrap_or(0) >= run_started;
        if let Some(path) = d["path"].as_str().filter(|_| parked_now) {
            store.lock().unwrap().push(thread_id, "step", &parked_note(path));
        }
    }
    // The work log + offers scan: the records the rest of the
    // product (brief, graph, today, offers) is built from.
    // Work means a tool step that succeeded or a written file; a run whose
    // steps all failed is an answer, not finished work (EB2, EB2b).
    if ledger.ok_steps == 0 && ledger.written.is_empty() {
        records::append_answer(cfg, task, &summary, thread_id);
    } else {
        records::append_work(cfg, task, &summary, &ledger.written, thread_id, true);
    }
    {
        let lib = ctx.library.lock().unwrap();
        let suite_now = library::load_suite(&cfg.data_dir);
        records::refresh_offers(cfg, &suite_now, &lib);
    }
    Ok(AgentOutcome { summary, steps })
}

// ---------------------------------------------------------------------------
// HTTP server (hand-rolled, std TcpListener)
// ---------------------------------------------------------------------------

/// One person's live state: their scoped config + their thread store.
pub struct PersonRuntime {
    pub cfg: Config,
    pub store: SharedStore,
}

fn load_runtime(root: &Config, person: &Person, library: &library::Library) -> PersonRuntime {
    let cfg = root.person_config(person);
    let fresh_memory = !cfg.memory_path.exists();
    let _ = cfg.ensure_dirs();
    // Their suite exists from the first moment too: drawn from the
    // company library, shaped by their role.
    library::ensure_suite(&cfg.data_dir, person, library);
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

#[derive(Clone, Debug)]
pub struct LiveModelConfig {
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub auto_detected: bool,
}

pub struct AppState {
    /// Root config (data dir + startup model route).
    pub cfg: Config,
    /// Live mutable model configuration updated by POST /api/config without restarting.
    pub model_cfg: Arc<Mutex<LiveModelConfig>>,
    /// The company library — one shared store of skills, subagents
    /// and apps that the agents themselves maintain.
    pub library: Arc<Mutex<library::Library>>,
    /// The hosting-layer gate token (ERGON_GATE_TOKEN), when set:
    /// every request must carry it as the `X-Ergon-Gate` header. The
    /// hosting layer authenticates people; Ergon itself has no
    /// accounts and no passwords.
    pub gate_token: Option<String>,
    /// Extra Host names accepted without a gate token (ERGON_ALLOWED_HOSTS).
    pub allowed_hosts: Vec<String>,
    pub people: Arc<Mutex<PeopleState>>,
    pub runtimes: Arc<Mutex<HashMap<String, Arc<PersonRuntime>>>>,
    /// In-flight runs per person for live step progress streaming (`GET /api/runs/status`).
    pub live_runs: LiveRuns,
    /// Browser sessions (cookie id → person id). "Active person" used
    /// to be one global pointer: any client could switch it for every
    /// other client mid-task. A session pins the person per browser;
    /// cookie-less API clients keep the global-pointer behaviour.
    pub sessions: Arc<Mutex<HashMap<String, String>>>,
    /// Checks GitHub for newer releases and installs them on request.
    pub updater: Arc<update::Updater>,
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
    if let Some(rt) = map.get(&person.id) {
        return rt.clone();
    }
    let lib = state.library.lock().unwrap().clone();
    let rt = Arc::new(load_runtime(&state.cfg, person, &lib));
    map.insert(person.id.clone(), rt.clone());
    rt
}

pub fn serve(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    serve_env(listener, cfg, false)
}

/// What the `ergon` binary runs: `serve` plus the periodic check for a newer
/// GitHub release (see `update`). Tests use `serve`, which never calls out.
pub fn serve_with_updates(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    serve_env(listener, cfg, true)
}

fn serve_env(listener: TcpListener, cfg: Config, background_updates: bool) -> std::io::Result<()> {
    let gate = std::env::var("ERGON_GATE_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let hosts = allowed_hosts_from(&std::env::var("ERGON_ALLOWED_HOSTS").unwrap_or_default());
    serve_inner(listener, cfg, gate, hosts, background_updates)
}

/// Parses ERGON_ALLOWED_HOSTS: comma-separated host names, with or without
/// ports, case-insensitive. Each entry is kept as its bare host name.
pub fn allowed_hosts_from(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|h| host_name(&h.trim().to_ascii_lowercase()))
        .filter(|h| !h.is_empty())
        .collect()
}

pub fn serve_with_gate(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
) -> std::io::Result<()> {
    serve_with_hosts(listener, cfg, gate_token, Vec::new())
}

/// Serves with explicit gate token and extra allowed Host names.
pub fn serve_with_hosts(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    allowed_hosts: Vec<String>,
) -> std::io::Result<()> {
    serve_inner(listener, cfg, gate_token, allowed_hosts, false)
}

/// `background_updates` starts the periodic release check; only the real
/// binary does (tests must not call out to GitHub).
fn serve_inner(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    allowed_hosts: Vec<String>,
    background_updates: bool,
) -> std::io::Result<()> {
    cfg.ensure_dirs()?;
    let gate_token_is_none = gate_token.is_none();
    let people = PeopleState::load(&cfg);
    let library = library::Library::load(&cfg.data_dir);
    let auto_detected = cfg.api_key.is_none() && cfg.model_configured();
    let model_cfg = Arc::new(Mutex::new(LiveModelConfig {
        model_base_url: cfg.model_base_url.clone(),
        model_id: cfg.model_id.clone(),
        api_key: cfg.api_key.clone(),
        auto_detected,
    }));
    let state = Arc::new(AppState {
        cfg,
        model_cfg,
        library: Arc::new(Mutex::new(library)),
        gate_token,
        allowed_hosts,
        people: Arc::new(Mutex::new(people)),
        runtimes: Arc::new(Mutex::new(HashMap::new())),
        live_runs: Arc::new(Mutex::new(HashMap::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
        updater: update::Updater::from_env("ERGON", "ergon", env!("CARGO_PKG_VERSION"), "cayleb-james2008/ergon", gate_token_is_none),
    });
    if background_updates {
        state.updater.start_background();
    }
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
    query: String,
    body: Vec<u8>,
    /// The caller's `ergon_session` cookie, if they presented one.
    session: Option<String>,
    /// The hosting layer's gate header, if presented.
    gate: Option<String>,
    /// Browser-set headers the cross-site checks read (lowercased).
    content_type: String,
    origin: Option<String>,
    host: Option<String>,
    sec_fetch_site: Option<String>,
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
    let request_line = match head_line(&mut reader, MAX_HEAD_BYTES) {
        Ok(Some((0, _))) => return None,
        Ok(Some((_, line))) => line,
        Ok(None) => return refuse_head(stream, &mut reader),
        Err(_) => return None,
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let raw_path = parts.next()?.to_string();
    let (path_part, query_part) = match raw_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_path.as_str(), ""),
    };
    let path = path_part.to_string();
    let query = query_part.to_string();
    let mut content_length: usize = 0;
    let mut session: Option<String> = None;
    let mut gate: Option<String> = None;
    let mut content_type = String::new();
    let mut origin: Option<String> = None;
    let mut host: Option<String> = None;
    let mut sec_fetch_site: Option<String> = None;
    let mut head_bytes = request_line.len();
    loop {
        arm(stream)?;
        let room = MAX_HEAD_BYTES - head_bytes;
        let (n, line) = match head_line(&mut reader, room) {
            Ok(Some(read)) => read,
            Ok(None) => return refuse_head(stream, &mut reader),
            Err(_) => return None,
        };
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        head_bytes += n;
        let lower = line.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
        if let Some(v) = lower.strip_prefix("content-type:") {
            content_type = v.trim().to_string();
        }
        if let Some(v) = lower.strip_prefix("origin:") {
            origin = Some(v.trim().to_string());
        }
        if let Some(v) = lower.strip_prefix("host:") {
            host = Some(v.trim().to_string());
        }
        if let Some(v) = lower.strip_prefix("sec-fetch-site:") {
            sec_fetch_site = Some(v.trim().to_string());
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
        if lower.starts_with("x-ergon-gate:") {
            // The value is a secret: take it from the ORIGINAL line,
            // not the lowercased copy used for header-name matching.
            if let Some((_, v)) = line.split_once(':') {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    gate = Some(v);
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
        query,
        body,
        session,
        gate,
        content_type,
        origin,
        host,
        sec_fetch_site,
    })
}

/// Reads one request-head line of at most `room` bytes, returning its
/// length and text. `Ok(None)` means the line is longer than `room`:
/// it is never buffered past the cap (E2).
fn head_line(reader: &mut BufReader<std::net::TcpStream>, room: usize) -> std::io::Result<Option<(usize, String)>> {
    let mut buf = Vec::new();
    let n = (&mut *reader).take(room as u64 + 1).read_until(b'\n', &mut buf)?;
    if n > room {
        return Ok(None);
    }
    Ok(Some((n, String::from_utf8_lossy(&buf).into_owned())))
}

/// Answers an over-long request head with 431 at once, then reads and
/// discards what the client still sends until it closes (or the read
/// deadline passes) so the answer is not reset by the socket (E2).
fn refuse_head(stream: &mut std::net::TcpStream, reader: &mut BufReader<std::net::TcpStream>) -> Option<Request> {
    let body = format!(
        "{{\"error\": \"request head too large: the limit is {MAX_HEAD_BYTES} bytes\"}}"
    );
    let response = format!(
        "HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut sink = [0u8; 8192];
    while matches!(reader.read(&mut sink), Ok(n) if n > 0) {}
    None
}

/// The note pushed into a thread when a delete is parked (EB1, ER1).
pub fn parked_note(path: &str) -> String {
    format!("Note: {path} has NOT been deleted yet; the deletion is waiting for your approval in Decisions.")
}

/// Whether a Host header names this machine (DNS-rebinding guard).
fn host_name(host: &str) -> String {
    match host.strip_prefix('[') {
        Some(rest) => format!("[{}]", rest.split(']').next().unwrap_or("")),
        None => host.split(':').next().unwrap_or("").to_string(),
    }
}

/// Whether a Host names this machine or one the operator allowed (EH1).
fn host_allowed(host: &str, allowed: &[String]) -> bool {
    let name = host_name(host);
    matches!(name.as_str(), "localhost" | "127.0.0.1" | "[::1]") || allowed.contains(&name)
}

/// Refuses a request a page on another site could have sent (E1), with
/// the same observable rules as the Fleet app: a foreign Host (unless
/// the hosting gate is set), a state-changing request with a body that
/// is not JSON (a text/plain POST has no preflight), cross-site fetch
/// metadata, and an Origin whose authority is not the request's Host.
fn refusal(req: &Request, gated: bool, allowed: &[String]) -> Option<(u16, &'static str)> {
    if !gated && !host_allowed(req.host.as_deref().unwrap_or(""), allowed) {
        return Some((403, "requests must address this machine as localhost, 127.0.0.1 or [::1], or a host listed in ERGON_ALLOWED_HOSTS"));
    }
    if !matches!(req.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        return None;
    }
    if !req.body.is_empty() && !req.content_type.starts_with("application/json") {
        return Some((415, "POST requires Content-Type: application/json"));
    }
    if req.sec_fetch_site.as_deref() == Some("cross-site") {
        return Some((403, "cross-site requests are refused"));
    }
    if let Some(origin) = &req.origin {
        let authority = origin.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        if req.host.as_deref() != Some(authority) {
            return Some((403, "cross-origin requests are refused"));
        }
    }
    None
}

fn respond(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    respond_h(stream, status, content_type, body, &[])
}

fn respond_bytes(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        431 => "Request Header Fields Too Large",
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
    stream.write_all(body)?;
    stream.flush()
}

fn respond_h(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    respond_bytes(stream, status, content_type, body.as_bytes(), extra_headers)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(k) == key {
            return Some(percent_decode(v));
        }
    }
    None
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u8;
    for b in input.bytes() {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'=' {
            break;
        }
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("invalid base64 data".to_string()),
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "csv" => "text/csv; charset=utf-8",
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" | "log" | "rs" | "ts" | "tsx" | "js" | "py" | "toml" | "yaml" | "yml" => {
            "text/plain; charset=utf-8"
        }
        _ => "application/octet-stream",
    }
}

fn list_workspace_files(root: &Path) -> Vec<Value> {
    fn walk(root: &Path, dir: &Path, depth: usize, out: &mut Vec<(u64, String, Value)>) {
        if depth > 6 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for entry in rd.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let p = entry.path();
            if ft.is_dir() {
                walk(root, &p, depth + 1, out);
                continue;
            }
            let Ok(md) = entry.metadata() else { continue };
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let mtime_ms = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            out.push((
                mtime_ms,
                rel.clone(),
                json!({
                    "path": rel,
                    "name": name,
                    "size": md.len(),
                    "mtime_ms": mtime_ms,
                    "ext": ext,
                }),
            ));
        }
    }
    let mut items = Vec::new();
    walk(root, root, 0, &mut items);
    items.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    items.into_iter().map(|(_, _, v)| v).collect()
}

fn effective_cfg(state: &AppState, runtime: &PersonRuntime) -> Config {
    let mut cfg = runtime.cfg.clone();
    let live = state.model_cfg.lock().unwrap();
    cfg.model_base_url = live.model_base_url.clone();
    cfg.model_id = live.model_id.clone();
    cfg.api_key = live.api_key.clone();
    cfg
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
    // Hosting-layer gate: when a gate token is configured, the
    // hosting layer (which authenticates people its own way) must
    // present it on every request. Ergon itself keeps no accounts
    // and no passwords — this is a deployment gate, not a login.
    if let Some(expected) = &state.gate_token {
        if req.gate.as_deref() != Some(expected.as_str()) {
            return respond_json(
                &mut stream,
                401,
                &json!({"error": "this Ergon is gated by its hosting layer — the gate header is missing or wrong"}),
            );
        }
    }
    if let Some((status, message)) = refusal(&req, state.gate_token.is_some(), &state.allowed_hosts) {
        return respond_json(&mut stream, status, &json!({"error": message}));
    }
    // The person this request acts for (session-pinned when the caller
    // has a session cookie, else the global active person).
    let person = effective_person(state, &req.session);
    let runtime = runtime_for(state, &person);
    // The tool context for agent runs on behalf of this person.
    let ctx = ToolCtx {
        library: state.library.clone(),
        root_data_dir: root_cfg.data_dir.clone(),
        person_id: person.id.clone(),
    };
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
        ("GET", "/api/version") => respond_json(
            &mut stream,
            200,
            &json!({"name": "ergon", "version": env!("CARGO_PKG_VERSION")}),
        ),
        ("GET", "/api/update") => respond_json(&mut stream, 200, &state.updater.snapshot()),
        ("POST", "/api/update/check") => respond_json(&mut stream, 200, &state.updater.check_now()),
        ("POST", "/api/update/apply") | ("POST", "/api/update/restart") => {
            // Replacing the program is for whoever sits at this machine: not
            // through a LAN host name, not through the shared gate.
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "updating is only allowed from this machine, not through a shared address"}),
                );
            }
            if req.path == "/api/update/apply" {
                match state.updater.apply() {
                    Ok(v) => {
                        let mut snap = state.updater.snapshot();
                        snap["applied"] = json!(v);
                        respond_json(&mut stream, 200, &snap)
                    }
                    Err(e) => respond_json(&mut stream, 409, &json!({"error": e})),
                }
            } else if let Err(e) = state.updater.snapshot()["staged"].as_str().map(|_| ()).ok_or("no update is waiting for a restart") {
                respond_json(&mut stream, 409, &json!({"error": e}))
            } else if RUNS_IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                respond_json(&mut stream, 409, &json!({"error": "a task is still running; restart when it has finished"}))
            } else {
                let _ = respond_json(&mut stream, 200, &json!({"restarting": true}));
                let up = state.updater.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    if let Err(e) = up.restart() {
                        eprintln!("ergon: restart failed: {e}");
                    }
                });
                Ok(())
            }
        }
        ("GET", "/api/status") => {
            let mut recovered: Vec<String> = state.people.lock().unwrap().recovered.clone();
            recovered.extend(state.library.lock().unwrap().recovered.clone());
            for rt in state.runtimes.lock().unwrap().values() {
                recovered.extend(rt.store.lock().unwrap().recovered.clone());
            }
            recovered.sort();
            recovered.dedup();
            let eff = effective_cfg(state, &runtime);
            let live_meta = state.model_cfg.lock().unwrap().clone();
            respond_json(
                &mut stream,
                200,
                &json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "model_configured": eff.model_configured(),
                    "model": eff.model_id,
                    "model_base_url": eff.model_base_url,
                    "is_loopback_model": is_loopback_url(&eff.model_base_url),
                    "auto_detected": live_meta.auto_detected,
                    "person": person.to_json(),
                    "store_recovered": recovered,
                }),
            )
        }
        ("GET", "/api/config") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "model settings are only available from this machine, not through a shared address"}),
                );
            }
            let eff = effective_cfg(state, &runtime);
            let live_meta = state.model_cfg.lock().unwrap().clone();
            let gateways: Vec<Value> = detect_local_gateways()
                .into_iter()
                .map(|g| g.to_json())
                .collect();
            respond_json(
                &mut stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": live_meta.auto_detected,
                    "detected_gateways": gateways,
                }),
            )
        }
        ("POST", "/api/config") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "changing model settings is only allowed from this machine, not through a shared address"}),
                );
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let base = parsed["model_base_url"]
                .as_str()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/')
                .to_string();
            let model_id = parsed["model_id"].as_str().unwrap_or("").trim().to_string();
            if !(base.starts_with("http://") || base.starts_with("https://")) || parse_url_host_port(&base).is_none() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "model_base_url must be a valid http:// or https:// URL"}),
                );
            }
            if model_id.is_empty() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "model_id is required"}),
                );
            }
            let clear_key = parsed["clear_api_key"].as_bool().unwrap_or(false);
            let next_key = if clear_key {
                None
            } else if let Some(raw_k) = parsed["api_key"].as_str() {
                let k = raw_k.trim().to_string();
                if k.is_empty() {
                    state.model_cfg.lock().unwrap().api_key.clone()
                } else {
                    Some(k)
                }
            } else {
                state.model_cfg.lock().unwrap().api_key.clone()
            };
            let saved = SavedModelConfig {
                model_base_url: base.clone(),
                model_id: model_id.clone(),
                api_key: next_key.clone(),
            };
            if let Err(e) = save_model_config(&root_cfg.data_dir, &saved) {
                return respond_json(&mut stream, 500, &json!({"error": e}));
            }
            {
                let mut live = state.model_cfg.lock().unwrap();
                live.model_base_url = base;
                live.model_id = model_id;
                live.api_key = next_key;
                live.auto_detected = false;
            }
            let eff = effective_cfg(state, &runtime);
            respond_json(
                &mut stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": false,
                }),
            )
        }
        ("GET", "/api/runs/status") => {
            let active = state.live_runs.lock().unwrap().get(&person.id).cloned();
            match active {
                Some(run) => respond_json(
                    &mut stream,
                    200,
                    &json!({
                        "active": true,
                        "thread_id": run.thread_id,
                        "task": run.task,
                        "started_ms": run.started_ms,
                        "steps": run.steps,
                    }),
                ),
                None => respond_json(
                    &mut stream,
                    200,
                    &json!({
                        "active": false,
                        "thread_id": "",
                        "task": "",
                        "started_ms": 0,
                        "steps": [],
                    }),
                ),
            }
        }
        ("GET", "/api/files") => {
            let files = list_workspace_files(&runtime.cfg.workspace_dir);
            respond_json(
                &mut stream,
                200,
                &json!({
                    "person_id": person.id,
                    "files": files,
                }),
            )
        }
        ("GET", "/api/files/content") => {
            let rel = query_param(&req.query, "path").unwrap_or_default();
            let rel = rel.trim().to_string();
            if rel.is_empty() {
                return respond_json(&mut stream, 400, &json!({"error": "path is required"}));
            }
            if rel.contains('\0') || rel.starts_with('/') || rel.starts_with('\\') {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "path escapes the workspace"}),
                );
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &rel, true) {
                Ok(p) => p,
                Err(e) => return respond_json(&mut stream, 403, &json!({"error": e})),
            };
            if full == runtime.cfg.workspace_dir || full.is_dir() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "path must name a file inside the workspace"}),
                );
            }
            let md = match std::fs::metadata(&full) {
                Ok(m) => m,
                Err(_) => return respond_json(&mut stream, 404, &json!({"error": "file not found"})),
            };
            const MAX_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;
            if md.len() > MAX_PREVIEW_BYTES {
                return respond_json(
                    &mut stream,
                    413,
                    &json!({"error": "file exceeds the 2 MiB preview limit"}),
                );
            }
            let bytes = match std::fs::read(&full) {
                Ok(b) => b,
                Err(e) => return respond_json(&mut stream, 500, &json!({"error": format!("read failed: {e}")})),
            };
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if download {
                let safe_name: String = name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' })
                    .collect();
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"{safe_name}\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                return respond_bytes(&mut stream, 200, mime_for_ext(&ext), &bytes, &headers);
            }
            let mtime_ms = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            match String::from_utf8(bytes) {
                Ok(content) => respond_json(
                    &mut stream,
                    200,
                    &json!({
                        "path": rel,
                        "name": name,
                        "size": md.len(),
                        "mtime_ms": mtime_ms,
                        "ext": ext,
                        "is_binary": false,
                        "content": content,
                    }),
                ),
                Err(_) => respond_json(
                    &mut stream,
                    200,
                    &json!({
                        "path": rel,
                        "name": name,
                        "size": md.len(),
                        "mtime_ms": mtime_ms,
                        "ext": ext,
                        "is_binary": true,
                        "content": "",
                    }),
                ),
            }
        }
        ("POST", "/api/files/upload") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let raw_rel = parsed["path"]
                .as_str()
                .or_else(|| parsed["name"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw_rel.is_empty() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "filename is required"}),
                );
            }
            if raw_rel.contains('\0')
                || raw_rel.starts_with('/')
                || raw_rel.starts_with('\\')
                || raw_rel.split('/').any(|seg| seg.is_empty() || seg.starts_with('.'))
            {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "invalid workspace filename"}),
                );
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, false) {
                Ok(p) => p,
                Err(e) => return respond_json(&mut stream, 403, &json!({"error": e})),
            };
            if sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, true).is_err() {
                return respond_json(
                    &mut stream,
                    403,
                    &json!({"error": "path escapes the workspace"}),
                );
            }
            if full == runtime.cfg.workspace_dir {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "filename must name a file"}),
                );
            }
            let encoding = parsed["encoding"].as_str().unwrap_or("utf8");
            let content_str = match parsed["content"].as_str() {
                Some(s) => s,
                None => {
                    return respond_json(
                        &mut stream,
                        400,
                        &json!({"error": "content is required"}),
                    )
                }
            };
            let bytes = if encoding.eq_ignore_ascii_case("base64") {
                match decode_base64(content_str) {
                    Ok(b) => b,
                    Err(e) => return respond_json(&mut stream, 400, &json!({"error": e})),
                }
            } else {
                content_str.as_bytes().to_vec()
            };
            if let Some(parent) = full.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return respond_json(
                        &mut stream,
                        500,
                        &json!({"error": format!("create parent dir: {e}")}),
                    );
                }
            }
            let tmp = full.with_extension("upload.tmp");
            if let Err(e) = std::fs::write(&tmp, &bytes).and_then(|_| std::fs::rename(&tmp, &full)) {
                let _ = std::fs::remove_file(&tmp);
                return respond_json(
                    &mut stream,
                    500,
                    &json!({"error": format!("save file failed: {e}")}),
                );
            }
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&raw_rel)
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let mtime_ms = std::fs::metadata(&full)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            respond_json(
                &mut stream,
                201,
                &json!({
                    "path": raw_rel,
                    "name": name,
                    "size": bytes.len(),
                    "mtime_ms": mtime_ms,
                    "ext": ext,
                }),
            )
        }
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
        ("GET", "/phone") => {
            // The phone face: same core, same session discipline.
            if !session_known(state, &req.session) {
                let sid = new_session_id();
                insert_session(state, sid.clone(), person.id.clone());
                return respond_h(
                    &mut stream,
                    200,
                    "text/html; charset=utf-8",
                    ui::PHONE_HTML,
                    &[(
                        "Set-Cookie",
                        format!("ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"),
                    )],
                );
            }
            respond(&mut stream, 200, "text/html; charset=utf-8", ui::PHONE_HTML)
        }
        ("GET", "/api/library") => {
            let lib = state.library.lock().unwrap();
            respond_json(
                &mut stream,
                200,
                &json!({
                    "items": lib.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
                }),
            )
        }
        ("GET", "/api/suite") => {
            let lib = state.library.lock().unwrap();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let installed: Vec<Value> = suite
                .resolved(&lib)
                .iter()
                .map(|i| i.to_json())
                .collect();
            respond_json(
                &mut stream,
                200,
                &json!({"person_id": person.id, "installed": installed}),
            )
        }
        ("POST", "/api/suite/install") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            let lib = state.library.lock().unwrap();
            match lib.get(id) {
                None => respond_json(
                    &mut stream,
                    404,
                    &json!({"error": "library item not found"}),
                ),
                Some(item) if item.builtin => respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "that item is built into every agent already"}),
                ),
                Some(item) => {
                    library::with_suite_lock(|| {
                        let mut suite = library::load_suite(&runtime.cfg.data_dir);
                        suite.install(&item.id);
                        library::save_suite(&runtime.cfg.data_dir, &suite);
                    });
                    respond_json(&mut stream, 200, &json!({"installed": item.id}))
                }
            }
        }
        ("POST", "/api/suite/remove") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            library::with_suite_lock(|| {
                let mut suite = library::load_suite(&runtime.cfg.data_dir);
                suite.installed.retain(|i| i != id);
                library::save_suite(&runtime.cfg.data_dir, &suite);
            });
            respond_json(&mut stream, 200, &json!({"removed": id}))
        }
        ("GET", "/api/decisions") => {
            let decisions = records::load_decisions(&runtime.cfg.data_dir);
            respond_json(&mut stream, 200, &json!({"decisions": decisions}))
        }
        ("GET", "/api/offers") => {
            // Refresh first, so the inbox shows what the work log has
            // newly earned since the last look. The library is cloned
            // and the lock dropped before the scan does file I/O —
            // the mutex guards the in-memory items, nothing slower.
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let offers = records::refresh_offers(&runtime.cfg, &suite, &lib);
            respond_json(&mut stream, 200, &json!({"offers": offers}))
        }
        ("GET", "/api/brief") => {
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            records::refresh_offers(&runtime.cfg, &suite, &lib);
            let brief = records::build_brief(&runtime.cfg, &person.name, &suite, &lib);
            respond_json(&mut stream, 200, &brief)
        }
        ("GET", "/api/today") => respond_json(
            &mut stream,
            200,
            &records::build_today(&runtime.cfg),
        ),
        ("GET", "/api/graph") => {
            // Cloned snapshots, locks dropped: build_graph reads
            // every person's files from disk and must not hold the
            // people/library locks across that I/O.
            let people_snapshot = state.people.lock().unwrap().people.clone();
            let lib = state.library.lock().unwrap().clone();
            respond_json(
                &mut stream,
                200,
                &records::build_graph(&people_snapshot, root_cfg, &lib),
            )
        }
        ("POST", path) if path.starts_with("/api/decisions/") => {
            let rest = path.trim_start_matches("/api/decisions/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return respond_json(
                        &mut stream,
                        404,
                        &json!({"error": "not found"}),
                    )
                }
            };
            if action != "approve" && action != "deny" {
                return respond_json(&mut stream, 404, &json!({"error": "not found"}));
            }
            match records::resolve_decision(&runtime.cfg, &id, action == "approve") {
                Ok((decision, outcome)) => {
                    // The outcome reports back into the thread the
                    // decision came from, so the record is complete
                    // where the work happened.
                    let tid = decision["thread_id"].as_str().unwrap_or("");
                    if !tid.is_empty() {
                        let mut store = runtime.store.lock().unwrap();
                        store.push(
                            tid,
                            "step",
                            &format!(
                                "Decision settled — {}: {}",
                                decision["summary"].as_str().unwrap_or(""),
                                outcome
                            ),
                        );
                        // The parked-delete note says the file is still
                        // there; once settled it must say what happened (ER1).
                        if let Some(path) = decision["path"].as_str().filter(|_| decision["kind"] == "delete") {
                            let settled = if action != "approve" {
                                format!("Note: {path} was kept because you declined the deletion.")
                            } else if outcome.starts_with("Approved and done") {
                                format!("Note: {path} was deleted after you approved it.")
                            } else if outcome.starts_with("Approved: ") {
                                format!("Note: {path} was already gone when you approved it.")
                            } else {
                                format!("Note: {path} was not deleted; see the decision settled above.")
                            };
                            store.settle_note(tid, &parked_note(path), &settled);
                        }
                    }
                    respond_json(
                        &mut stream,
                        200,
                        &json!({"decision": decision, "outcome": outcome}),
                    )
                }
                Err(e) => respond_json(&mut stream, 400, &json!({"error": e})),
            }
        }
        ("POST", path) if path.starts_with("/api/offers/") => {
            let rest = path.trim_start_matches("/api/offers/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return respond_json(
                        &mut stream,
                        404,
                        &json!({"error": "not found"}),
                    )
                }
            };
            match action.as_str() {
                "dismiss" => match records::set_offer_status(&runtime.cfg.data_dir, &id, "dismissed") {
                    Ok(offer) => respond_json(&mut stream, 200, &json!({"offer": offer})),
                    Err(e) => respond_json(&mut stream, 400, &json!({"error": e})),
                },
                "accept" => {
                    // Look the offer up first; acceptance DOES the
                    // thing offered (install the app / create the
                    // one-tap app from the repeated task), then marks
                    // the offer settled.
                    let offers = records::load_offers(&runtime.cfg.data_dir);
                    let Some(offer) = offers.iter().find(|o| o["id"].as_str() == Some(id.as_str())).cloned() else {
                        return respond_json(&mut stream, 404, &json!({"error": "offer not found"}));
                    };
                    if offer["status"].as_str() != Some("pending") {
                        return respond_json(
                            &mut stream,
                            400,
                            &json!({"error": "that offer was already settled"}),
                        );
                    }
                    let kind = offer["kind"].as_str().unwrap_or("");
                    let mut made: Option<Value> = None;
                    if kind == "install_app" {
                        let item_id = offer["item_id"].as_str().unwrap_or("");
                        library::with_suite_lock(|| {
                            let mut suite = library::load_suite(&runtime.cfg.data_dir);
                            if suite.install(item_id) {
                                library::save_suite(&runtime.cfg.data_dir, &suite);
                            }
                        });
                        made = Some(json!({"installed": item_id}));
                    } else if kind == "repeat_task" {
                        let template = offer["template"].as_str().unwrap_or("");
                        let short: String = template.chars().take(48).collect();
                        let name = format!("One-tap: {short}");
                        let mut lib = state.library.lock().unwrap();
                        match lib.publish_app(
                            &name,
                            "Created from an offer: work this person kept asking for, as a one-tap app.",
                            template.to_string(),
                            Vec::new(),
                            &person.id,
                        ) {
                            Ok(item) => {
                                lib.save(&root_cfg.data_dir);
                                library::with_suite_lock(|| {
                                    let mut suite =
                                        library::load_suite(&runtime.cfg.data_dir);
                                    suite.install(&item.id);
                                    library::save_suite(&runtime.cfg.data_dir, &suite);
                                });
                                made = Some(item.to_json());
                            }
                            Err(e) => {
                                return respond_json(&mut stream, 400, &json!({"error": e}))
                            }
                        }
                    }
                    match records::set_offer_status(&runtime.cfg.data_dir, &id, "accepted") {
                        Ok(offer) => respond_json(
                            &mut stream,
                            200,
                            &json!({"offer": offer, "made": made}),
                        ),
                        Err(e) => respond_json(&mut stream, 400, &json!({"error": e})),
                    }
                }
                _ => respond_json(&mut stream, 404, &json!({"error": "not found"})),
            }
        }
        ("POST", path) if path.starts_with("/api/apps/") && path.ends_with("/run") => {
            // Single strip, not trim_*_matches (which strips
            // repeatedly and would resolve "/api/apps/foo/run/run"
            // to the app "foo").
            let id = path
                .strip_prefix("/api/apps/")
                .and_then(|rest| rest.strip_suffix("/run"))
                .unwrap_or("")
                .to_string();
            let item = {
                let lib = state.library.lock().unwrap();
                lib.get(&id)
            };
            let Some(item) = item else {
                return respond_json(&mut stream, 404, &json!({"error": "app not found"}));
            };
            if item.kind != "app" {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "that library item is not an app"}),
                );
            }
            let suite = library::load_suite(&runtime.cfg.data_dir);
            if !suite.installed.contains(&item.id) {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "that app is not in this person's suite"}),
                );
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            // Every field the template actually uses must be filled
            // BEFORE substitution — checking afterwards cannot work:
            // substitution erases the evidence of an empty field and
            // the run would start with a hole in its instructions.
            for field in &item.fields {
                if item.template.contains(&format!("{{{field}}}")) {
                    let value = parsed["fields"][field.as_str()]
                        .as_str()
                        .unwrap_or("")
                        .trim();
                    if value.is_empty() {
                        return respond_json(
                            &mut stream,
                            400,
                            &json!({"error": "fill in every field before running the app"}),
                        );
                    }
                }
            }
            // Fill the template's {field} slots from the run's inputs.
            let mut task = item.template.clone();
            for field in &item.fields {
                let value = parsed["fields"][field.as_str()]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                task = task.replace(&format!("{{{field}}}"), &value);
            }
            let cfg = effective_cfg(state, &runtime);
            let thread_id = {
                let mut store = runtime.store.lock().unwrap();
                let tid = store.create_thread();
                store.push(&tid, "user", &format!("{} — {}", item.name, task));
                tid
            };
            if !cfg.model_configured() {
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
            match run_agent_live(
                &cfg,
                &ctx,
                &runtime.store,
                &thread_id,
                &task,
                &[],
                Some((&state.live_runs, &person.id)),
            ) {
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
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                Value::Null
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => {
                        return respond_json(
                            &mut stream,
                            400,
                            &json!({"error": "request body is not valid JSON"}),
                        )
                    }
                }
            };
            let message = parsed["message"].as_str().unwrap_or("").trim().to_string();
            if message.is_empty() {
                return respond_json(
                    &mut stream,
                    400,
                    &json!({"error": "message is required"}),
                );
            }
            let cfg = effective_cfg(state, &runtime);
            // The user's words are recorded FIRST, before any failure
            // path: a refused or failed run must never silently lose
            // the task the person typed (the no-key path used to drop
            // it, so the message vanished on reload).
            let (thread_id, history) = {
                let mut store = runtime.store.lock().unwrap();
                let requested = parsed["thread_id"].as_str().unwrap_or("");
                // A named thread must exist: opening a new one silently
                // would put the message somewhere the person never asked (EB3).
                if !requested.is_empty() && store.thread_json(requested).is_none() {
                    return respond_json(&mut stream, 404, &json!({"error": "no such thread"}));
                }
                let tid = store.ensure_thread(requested);
                let history = store.history_json(&tid);
                store.push(&tid, "user", &message);
                (tid, history)
            };
            if !cfg.model_configured() {
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
            match run_agent_live(
                &cfg,
                &ctx,
                &runtime.store,
                &thread_id,
                &message,
                &history,
                Some((&state.live_runs, &person.id)),
            ) {
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
