//! Sandboxed workspace filesystem/URL tools, export helpers, and honesty checks.

use crate::*;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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

static FILE_TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Atomically write `bytes` to `full` using a unique temporary file in the same
/// parent directory, cleaning up on failure so concurrent writes never collide.
pub(crate) fn atomic_write_bytes(full: &Path, bytes: &[u8]) -> std::io::Result<()> {
    atomic_write_bytes_with_mode(full, bytes, None)
}

/// Atomically write `bytes` to `full`, applying optional POSIX file mode bits
/// on the temporary file before renaming into place.
pub(crate) fn atomic_write_bytes_with_mode(
    full: &Path,
    bytes: &[u8],
    mode: Option<u32>,
) -> std::io::Result<()> {
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let seq = FILE_TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let file_name = full
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let tmp = full.with_file_name(format!(".{file_name}.ergon-{seq}.tmp"));
    let write_res = std::fs::write(&tmp, bytes).and_then(|_| {
        #[cfg(unix)]
        if let Some(m) = mode {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(m))?;
        }
        #[cfg(not(unix))]
        let _ = mode;
        std::fs::rename(&tmp, full)
    });
    if let Err(e) = write_res {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
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
    const EXTS: [&str; 9] = ["md", "csv", "txt", "json", "html", "pdf", "docx", "xlsx", "pptx"];
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

/// Characters of a file the agent sees per `read_file` call (the loop
/// passes tool results to the model at up to 6,000 characters).
pub const READ_WINDOW: usize = 5_800;

/// One window of a long text, with an explicit way to read on: a 40-page
/// contract is read in full, window by window, instead of silently cut.
pub fn page_text(content: &str, rel: &str, offset: usize) -> String {
    let total = content.chars().count();
    if offset == 0 && total <= READ_WINDOW {
        return content.to_string();
    }
    if offset >= total {
        return format!("(end of {rel}: it has {total} characters)");
    }
    let end = (offset + READ_WINDOW).min(total);
    let window: String = content.chars().skip(offset).take(end - offset).collect();
    if end < total {
        format!(
            "{window}\n…[showing characters {offset}–{end} of {total}; call read_file with {{\"path\": \"{rel}\", \"offset\": {end}}} to read on]"
        )
    } else {
        format!("{window}\n…[end of {rel}: characters {offset}–{end} of {total}]")
    }
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

/// Every outbound page fetch — the `fetch_url` tool, the research
/// workflow and library skills' `host_fetch_url` — goes through the
/// tripwire egress guard: URL-level checks (secrets, bait keys, blocked
/// and metadata hosts) and then an SSRF-safe resolver that vets the
/// RESOLVED address of every hop, redirects included.
pub fn fetch_url_text(url: &str) -> Result<String, String> {
    tripwire::check_egress(url, "fetch")?;
    let agent = tripwire::guarded_agent(std::time::Duration::from_secs(20));
    let resp = agent
        .get(url)
        .set("User-Agent", BROWSER_UA)
        .call()
        .map_err(|e| tripwire::explain_fetch_error(url, &e))?;
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
// Tools
// ---------------------------------------------------------------------------

pub fn execute_tool(cfg: &Config, name: &str, args: &Value) -> Result<String, String> {
    match name {
        "list_dir" => {
            let rel = args["path"].as_str().unwrap_or(".");
            let full = sandbox_path(&cfg.workspace_dir, rel)?;
            let mut entries: Vec<String> = Vec::new();
            let rd = std::fs::read_dir(&full)
                .map_err(|e| format!("list_dir {rel}: {e}"))?;
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
            // `book.xlsx#Q2` names one sheet of a workbook.
            let (base, sheet) = docs::split_sheet(rel);
            let full = sandbox_path(&cfg.workspace_dir, base)?;
            let content = match docs::doc_kind(base) {
                Some(kind) => docs::agent_view(&full, base, kind, sheet)?,
                None => std::fs::read_to_string(&full).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::InvalidData {
                        format!("read_file {rel}: this is a binary file (not text, PDF, Word, Excel or PowerPoint), so it cannot be read as text")
                    } else {
                        format!("read_file {rel}: {e}")
                    }
                })?,
            };
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            Ok(page_text(&content, rel, offset))
        }
        "write_file" => {
            let rel = args["path"].as_str().ok_or("write_file needs args.path")?;
            let content = args["content"].as_str().ok_or("write_file needs args.content")?;
            let full = sandbox_path(&cfg.workspace_dir, rel)?;
            let _ = file_history::record_revision(&cfg.data_dir, &cfg.workspace_dir, rel, "write_file");
            atomic_write_bytes(&full, content.as_bytes()).map_err(|e| format!("write_file {rel}: {e}"))?;
            Ok(format!("wrote {rel} ({} bytes)", content.len()))
        }
        "patch_file" => {
            let raw_rel = args["path"].as_str().ok_or("patch_file needs args.path")?;
            let find = args["find"].as_str().ok_or("patch_file needs args.find")?;
            if find.is_empty() {
                return Err("patch_file needs a non-empty args.find".to_string());
            }
            let replace = args["replace"]
                .as_str()
                .ok_or("patch_file needs args.replace")?;
            let replace_all = args["replace_all"].as_bool().unwrap_or(false);
            let rel = crate::tables::validate_workspace_rel_path(raw_rel)?;
            let unresolved = sandbox_path_resolving(&cfg.workspace_dir, &rel, false)?;
            if unresolved
                .symlink_metadata()
                .is_ok_and(|md| md.file_type().is_symlink())
            {
                return Err(format!("patch_file: cannot patch symlink {rel}"));
            }
            let full = sandbox_path(&cfg.workspace_dir, &rel)?;
            let text = std::fs::read_to_string(&full).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    format!("patch_file: no such file: {rel}")
                } else if e.kind() == std::io::ErrorKind::InvalidData {
                    format!("patch_file {rel}: binary files cannot be patched as UTF-8 text")
                } else {
                    format!("patch_file {rel}: {e}")
                }
            })?;
            let count = text.match_indices(find).count();
            if count == 0 {
                return Err(format!("patch_file: target text not found in {rel}"));
            }
            if count > 1 && !replace_all {
                return Err(format!(
                    "patch_file: found {count} matches in {rel} (ambiguous); pass a more specific find string or set replace_all: true"
                ));
            }
            let updated = if replace_all {
                text.replace(find, replace)
            } else {
                text.replacen(find, replace, 1)
            };
            let _ = file_history::record_revision(
                &cfg.data_dir,
                &cfg.workspace_dir,
                &rel,
                "patch_file",
            );
            atomic_write_bytes(&full, updated.as_bytes())
                .map_err(|e| format!("patch_file {rel}: {e}"))?;
            Ok(format!(
                "patched {rel} ({count} replacement{}, {} bytes)",
                if count == 1 { "" } else { "s" },
                updated.len()
            ))
        }
        "query_table" => {
            let res = crate::tables::execute_query(cfg, args)?;
            let mut out = format!(
                "query_table: {} of {} rows matched ({} output row{})",
                res.matched_rows,
                res.total_rows,
                res.rows.len(),
                if res.rows.len() == 1 { "" } else { "s" }
            );
            if let Some(sp) = &res.saved_path {
                out.push_str(&format!(" — wrote {sp}"));
            }
            let mut preview_rows = Vec::with_capacity(res.rows.len().min(40) + 1);
            if !res.columns.is_empty() {
                preview_rows.push(res.columns.clone());
            }
            preview_rows.extend(res.rows.iter().take(40).cloned());
            if !preview_rows.is_empty() {
                out.push('\n');
                out.push_str(&docs::rows_to_csv(&preview_rows));
                if res.rows.len() > 40 {
                    out.push_str(&format!("... ({} more rows omitted)", res.rows.len() - 40));
                }
            }
            Ok(out.trim_end().to_string())
        }
        "fetch_url" => {
            if crate::load_workspace_policy(&cfg.data_dir).block_outbound_fetch {
                return Err("blocked by Air-Gapped workspace policy: outbound network fetches are disabled for this workspace".to_string());
            }
            let url = args["url"].as_str().ok_or("fetch_url needs args.url")?;
            fetch_url_text(url)
        }
        "invoke_connector" => {
            let connector_id = args["connector_id"]
                .as_str()
                .or_else(|| args["id"].as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or("invoke_connector needs args.connector_id")?;
            let attachment_rel = args["file"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let payload = if let Some(p) = args.get("payload").filter(|v| !v.is_null()) {
                p.clone()
            } else if let Some(rel) = attachment_rel {
                let full = sandbox_path(&cfg.workspace_dir, rel)?;
                let bytes = std::fs::read(&full)
                    .map_err(|e| format!("invoke_connector cannot read file '{rel}': {e}"))?;
                if bytes.len() > 512 * 1024 {
                    return Err(format!("invoke_connector file '{rel}' exceeds 512 KB limit"));
                }
                if rel.to_ascii_lowercase().ends_with(".json") {
                    serde_json::from_slice::<Value>(&bytes).unwrap_or_else(|_| {
                        serde_json::json!({
                            "file": rel,
                            "content": String::from_utf8_lossy(&bytes).to_string(),
                        })
                    })
                } else {
                    serde_json::json!({
                        "file": rel,
                        "content": String::from_utf8_lossy(&bytes).to_string(),
                    })
                }
            } else {
                serde_json::json!({})
            };
            if let Some(rel) = attachment_rel {
                sandbox_path(&cfg.workspace_dir, rel)?;
            }
            let thread_id = args["_thread"].as_str().unwrap_or("");
            match crate::connectors::invoke_connector(
                cfg,
                connector_id,
                &payload,
                attachment_rel,
                thread_id,
            )? {
                crate::connectors::InvokeOutcome::Delivered(log) => Ok(format!(
                    "delivered via connector '{}' (HTTP {}, {} bytes, sha256:{})",
                    log.connector_name,
                    log.http_status.unwrap_or(200),
                    log.payload_bytes,
                    log.payload_sha256
                )),
                crate::connectors::InvokeOutcome::Held {
                    decision_id,
                    delivery,
                } => Ok(format!(
                    "parked for your person's approval (decision {decision_id}): outbound delivery to {} ({}) requires approval before sending. Do NOT call invoke_connector again for this delivery; report that it is awaiting approval.",
                    delivery.connector_name, delivery.url
                )),
            }
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
            if crate::load_workspace_policy(&cfg.data_dir).block_outbound_fetch {
                return Err("blocked by Air-Gapped workspace policy: outbound network fetches are disabled for this workspace".to_string());
            }
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
                match searchable_text(&base) {
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
        "export_file" => {
            let has_sheets = args["sheets"].as_array().is_some_and(|a| !a.is_empty());
            let rel = args["path"]
                .as_str()
                .or_else(|| if has_sheets { args["output"].as_str() } else { None })
                .ok_or("export_file needs args.path (a .md, .txt, .csv or .tsv workspace file)")?;
            let (out, bytes, label) = export_workspace_file_with_options(
                cfg,
                rel,
                args["format"].as_str(),
                args["output"].as_str(),
                args["brand_header"].as_str(),
                args["sheets"].as_array().map(|a| a.as_slice()),
                args["totals_row"].as_bool().unwrap_or(false),
            )?;
            Ok(format!("wrote {out} ({bytes} bytes) — {label} made from {rel}, ready to share"))
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
        other => {
            if let Some(res) = skills::dispatch_builtin_skill(cfg, other, args) {
                res
            } else {
                Err(format!("unknown tool: {other}"))
            }
        }
    }
}

/// Turn a workspace Markdown/text file into Word or PDF, or a CSV/TSV into Excel,
/// next to the source (or at `output`). Returns (path, bytes, what).
pub fn export_workspace_file(
    cfg: &Config,
    rel: &str,
    format: Option<&str>,
    output: Option<&str>,
) -> Result<(String, u64, &'static str), String> {
    export_workspace_file_with_options(cfg, rel, format, output, None, None, false)
}

/// Extended deliverable exporter supporting native `"pdf"` output (with
/// optional `brand_header`), `"docx"`, and single- or multi-sheet `"xlsx"`
/// (with optional `sheets` array and `totals_row` `SUM(...)` formulas).
pub fn export_workspace_file_with_options(
    cfg: &Config,
    rel: &str,
    format: Option<&str>,
    output: Option<&str>,
    brand_header: Option<&str>,
    sheets: Option<&[Value]>,
    totals_row: bool,
) -> Result<(String, u64, &'static str), String> {
    let clean_rel = crate::tables::validate_workspace_rel_path(rel)?;
    let lower = clean_rel.to_ascii_lowercase();
    let src_ext = lower.rsplit('.').next().unwrap_or("");
    let has_sheets = sheets.is_some_and(|s| !s.is_empty());
    let fmt = match format.map(|f| f.trim().trim_start_matches('.').to_ascii_lowercase()) {
        Some(f) if !f.is_empty() => f,
        _ if has_sheets => "xlsx".to_string(),
        _ => match src_ext {
            "csv" | "tsv" => "xlsx".to_string(),
            _ => "docx".to_string(),
        },
    };

    enum ExportTarget {
        Docx,
        Pdf,
        XlsxSingle,
        XlsxMulti,
    }

    let target = match (fmt.as_str(), src_ext, has_sheets) {
        ("docx" | "word", "md" | "markdown" | "txt", _) => ExportTarget::Docx,
        ("pdf", "md" | "markdown" | "txt", _) => ExportTarget::Pdf,
        ("xlsx" | "excel", _, true) => ExportTarget::XlsxMulti,
        ("xlsx" | "excel", "csv" | "tsv", false) => ExportTarget::XlsxSingle,
        ("docx" | "word", _, _) => {
            return Err(format!("export_file: Word export takes a .md or .txt file, not {clean_rel}"))
        }
        ("pdf", _, _) => {
            return Err(format!("export_file: PDF export takes a .md or .txt file, not {clean_rel}"))
        }
        ("xlsx" | "excel", _, false) => {
            return Err(format!("export_file: Excel export takes a .csv or .tsv file (or a sheets array), not {clean_rel}"))
        }
        (other, _, _) => {
            return Err(format!("export_file: unknown format \"{other}\" (use pdf, docx, or xlsx)"))
        }
    };

    let stem_rel = clean_rel.rsplit_once('.').map(|(a, _)| a).unwrap_or(&clean_rel);
    let stem_name = stem_rel.rsplit('/').next().unwrap_or(stem_rel).to_string();
    let want_ext = match target {
        ExportTarget::Docx => "docx",
        ExportTarget::Pdf => "pdf",
        ExportTarget::XlsxSingle | ExportTarget::XlsxMulti => "xlsx",
    };
    let out = match output.map(|o| o.trim()).filter(|o| !o.is_empty()) {
        Some(o) if o.to_ascii_lowercase().ends_with(&format!(".{want_ext}")) => o.to_string(),
        Some(o) => format!("{o}.{want_ext}"),
        None => format!("{stem_rel}.{want_ext}"),
    };
    let out = crate::tables::validate_workspace_rel_path(&out)?;
    let out_full = sandbox_path_resolving(&cfg.workspace_dir, &out, false)?;
    if sandbox_path_resolving(&cfg.workspace_dir, &out, true).is_err()
        || out_full.symlink_metadata().is_ok_and(|md| md.file_type().is_symlink())
    {
        return Err("export_file: cannot overwrite symlink".to_string());
    }

    let (bytes, label) = match target {
        ExportTarget::Docx | ExportTarget::Pdf => {
            let full = sandbox_path(&cfg.workspace_dir, &clean_rel)?;
            let text = std::fs::read_to_string(&full)
                .map_err(|e| format!("export_file {clean_rel}: {e}"))?;
            let title = text
                .lines()
                .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
                .unwrap_or_else(|| stem_name.clone());
            let md = if src_ext == "txt" {
                text.lines().map(|l| format!("{l}  ")).collect::<Vec<_>>().join("\n")
            } else {
                text
            };
            if matches!(target, ExportTarget::Pdf) {
                (docs::markdown_to_pdf(&md, &title, brand_header), "a PDF document")
            } else {
                (docs::markdown_to_docx(&md, &title), "a Word document")
            }
        }
        ExportTarget::XlsxSingle => {
            let full = sandbox_path(&cfg.workspace_dir, &clean_rel)?;
            let text = std::fs::read_to_string(&full)
                .map_err(|e| format!("export_file {clean_rel}: {e}"))?;
            let rows: Vec<Vec<String>> = if src_ext == "tsv" {
                text.lines().map(|l| l.split('\t').map(|c| c.to_string()).collect()).collect()
            } else {
                skills::parse_csv_rows(&text)
            };
            if rows.is_empty() {
                return Err(format!("export_file: {clean_rel} has no rows"));
            }
            let bytes = docs::sheets_to_xlsx(
                &[docs::ExportSheet { name: stem_name.clone(), rows, totals_row }],
                &stem_name,
            );
            (bytes, "an Excel workbook")
        }
        ExportTarget::XlsxMulti => {
            let specs = sheets.unwrap_or(&[]);
            let mut export_sheets: Vec<docs::ExportSheet> = Vec::with_capacity(specs.len());
            for (idx, item) in specs.iter().enumerate() {
                let sname = item["name"]
                    .as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("Sheet{}", idx + 1));
                let s_totals = item["totals_row"].as_bool().unwrap_or(false);
                let rows = if item["rows"].is_array() {
                    string_rows(&item["rows"])
                } else if let Some(sp) = item["path"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    let loaded = crate::tables::load_table(cfg, sp, item["sheet"].as_str())?;
                    let mut r = Vec::with_capacity(loaded.rows.len() + 1);
                    if !loaded.columns.is_empty() {
                        r.push(loaded.columns);
                    }
                    r.extend(loaded.rows);
                    r
                } else {
                    return Err(format!("export_file: sheet '{sname}' requires either rows or path"));
                };
                export_sheets.push(docs::ExportSheet { name: sname, rows, totals_row: s_totals });
            }
            (docs::sheets_to_xlsx(&export_sheets, &stem_name), "an Excel workbook")
        }
    };

    let _ = file_history::record_revision(&cfg.data_dir, &cfg.workspace_dir, &out, "export");
    atomic_write_bytes(&out_full, &bytes).map_err(|e| format!("export_file {out}: {e}"))?;
    Ok((out, bytes.len() as u64, label))
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
        "request_checkpoint" => {
            let cp = parse_checkpoint_tool_args(cfg, args)?;
            if args["_defer_decision"].as_bool() == Some(true) {
                return Ok(format!(
                    "paused at human checkpoint ({}): {}",
                    cp.input_mode, cp.prompt
                ));
            }
            let thread_id = args["_thread"].as_str().unwrap_or("");
            let id = records::add_decision_with(
                &cfg.data_dir,
                "checkpoint",
                &cp.prompt,
                cp.inspect_files.first().map(String::as_str),
                thread_id,
                serde_json::json!({
                    "input_mode": cp.input_mode,
                    "choices": cp.choices,
                    "inspect_files": cp.inspect_files,
                }),
            );
            Ok(format!(
                "paused at human checkpoint (decision {id}): {}. The run will resume automatically once your person answers.",
                cp.prompt
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

pub(crate) fn string_list(v: &Value) -> Vec<String> {
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

pub(crate) fn parse_checkpoint_tool_args(
    cfg: &Config,
    args: &Value,
) -> Result<crate::CheckpointPause, String> {
    let prompt = args["prompt"]
        .as_str()
        .or_else(|| args["summary"].as_str())
        .unwrap_or("")
        .trim()
        .chars()
        .take(2000)
        .collect::<String>();
    if prompt.is_empty() {
        return Err("request_checkpoint needs args.prompt".to_string());
    }
    let raw_mode = args["input_mode"]
        .as_str()
        .unwrap_or("approve_reject")
        .trim()
        .to_ascii_lowercase();
    let input_mode = match raw_mode.as_str() {
        "" | "approve_reject" => "approve_reject".to_string(),
        "choice" | "text" => raw_mode,
        other => {
            return Err(format!(
                "request_checkpoint: unknown input_mode '{other}' (use approve_reject, choice, or text)"
            ))
        }
    };
    let choices: Vec<String> = string_list(&args["choices"])
        .into_iter()
        .map(|s| s.trim().chars().take(200).collect::<String>())
        .filter(|s| !s.is_empty())
        .take(20)
        .collect();
    if input_mode == "choice" && choices.is_empty() {
        return Err("request_checkpoint with input_mode='choice' requires args.choices".to_string());
    }
    let mut inspect_files: Vec<String> = Vec::new();
    for raw_path in string_list(&args["inspect_files"]) {
        let p = raw_path.trim();
        if p.is_empty() {
            continue;
        }
        sandbox_path(&cfg.workspace_dir, p)?;
        if !inspect_files.iter().any(|x| x == p) {
            inspect_files.push(p.to_string());
        }
    }
    Ok(crate::CheckpointPause {
        prompt,
        input_mode,
        choices,
        inspect_files,
    })
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

/// Text to search in a file: plain text as-is; PDF, Word, Excel (every
/// sheet) and PowerPoint through the document extractor.
fn searchable_text(path: &Path) -> Result<String, String> {
    let name = path.to_string_lossy();
    match docs::doc_kind(&name) {
        Some(docs::DocKind::Excel) => {
            let first = docs::extract(path, docs::DocKind::Excel, None)?;
            let mut all = String::new();
            for sheet in &first.sheets {
                if let Ok(ex) = docs::extract(path, docs::DocKind::Excel, Some(sheet)) {
                    all.push_str(&ex.text);
                }
            }
            Ok(all)
        }
        Some(kind) => docs::extract(path, kind, None).map(|e| e.text),
        None => std::fs::read_to_string(path).map_err(|e| e.to_string()),
    }
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
        let is_doc = docs::doc_kind(&name).is_some();
        let limit = if is_doc { docs::MAX_DOC_BYTES } else { 512_000 };
        if entry.metadata().map(|m| m.len()).unwrap_or(0) > limit {
            stats.skipped += 1;
            continue;
        }
        let Ok(content) = searchable_text(&path) else {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationCheck {
    pub raw: String,
    pub path: String,
    pub line: usize,
    pub end_line: Option<usize>,
    pub status: &'static str,
    pub total_lines: usize,
    pub file_sha256: String,
    pub line_excerpt: String,
    pub baseline_sha256: Option<String>,
}

impl CitationCheck {
    pub fn to_json(&self) -> Value {
        json!({
            "raw": self.raw,
            "path": self.path,
            "line": self.line,
            "end_line": self.end_line,
            "status": self.status,
            "total_lines": self.total_lines,
            "file_sha256": self.file_sha256,
            "line_excerpt": self.line_excerpt,
            "baseline_sha256": self.baseline_sha256,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct CitationReport {
    pub total: usize,
    pub verified: usize,
    pub stale: usize,
    pub broken: usize,
    pub citations: Vec<CitationCheck>,
}

impl CitationReport {
    pub fn freshness_pct(&self) -> u8 {
        if self.total == 0 {
            100
        } else {
            ((self.verified * 100) / self.total) as u8
        }
    }

    pub fn gate_status(&self) -> &'static str {
        if self.broken == 0 && self.stale == 0 {
            "100% Verified"
        } else if self.stale > 0 {
            "Stale Line Drift"
        } else {
            "Broken Citations"
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "total": self.total,
            "verified": self.verified,
            "stale": self.stale,
            "broken": self.broken,
            "freshness_pct": self.freshness_pct(),
            "gate_status": self.gate_status(),
            "ok": self.broken == 0 && self.stale == 0,
            "citations": self.citations.iter().map(CitationCheck::to_json).collect::<Vec<_>>(),
        })
    }
}

const CITATION_EXTS: &[&str] = &[
    "md", "markdown", "csv", "tsv", "txt", "json", "log", "py", "ts", "tsx", "js", "rs", "rn",
    "toml", "yaml", "yml", "html",
];

fn is_path_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/' | b' ')
}

/// Extract deduped `[file.ext:L##]` / `file.ext:L##` / `[file.ext:L##-L##]` citations from text.
pub fn extract_file_citations(text: &str) -> Vec<(String, String, usize, Option<usize>)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(String, String, usize, Option<usize>)> = Vec::new();
    let mut seen: std::collections::HashSet<(String, usize, Option<usize>)> =
        std::collections::HashSet::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b':' {
            let after = &bytes[i + 1..];
            let mut pos = 0usize;
            let has_l = after.first().is_some_and(|c| *c == b'L' || *c == b'l');
            if has_l {
                pos += 1;
            }
            let num_start = pos;
            while pos < after.len() && after[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos > num_start {
                if let Ok(line_no) = std::str::from_utf8(&after[num_start..pos])
                    .unwrap_or("")
                    .parse::<usize>()
                {
                    // Optional range `-L##` or `-##`
                    let mut end_line: Option<usize> = None;
                    let mut total_after = pos;
                    if pos < after.len() && after[pos] == b'-' {
                        let mut rpos = pos + 1;
                        if after.get(rpos).is_some_and(|c| *c == b'L' || *c == b'l') {
                            rpos += 1;
                        }
                        let rstart = rpos;
                        while rpos < after.len() && after[rpos].is_ascii_digit() {
                            rpos += 1;
                        }
                        if rpos > rstart {
                            if let Ok(end_no) = std::str::from_utf8(&after[rstart..rpos])
                                .unwrap_or("")
                                .parse::<usize>()
                            {
                                end_line = Some(end_no);
                                total_after = rpos;
                            }
                        }
                    }
                    // Look left for `<path>.<ext>` (stopping at `[`, `` ` ``, newline, quote, or boundary).
                    let mut left = i;
                    while left > 0 && is_path_char(bytes[left - 1]) {
                        left -= 1;
                    }
                    let cand_raw = text[left..i].trim();
                    // Strip leading words before the rightmost space unless preceded by `[` or `` ` ``.
                    let bracketed = left > 0 && bytes[left - 1] == b'[';
                    let candidate = if bracketed {
                        cand_raw
                    } else {
                        cand_raw.rsplit_once(' ').map(|(_, r)| r).unwrap_or(cand_raw)
                    };
                    if let Some((_, ext)) = candidate.rsplit_once('.') {
                        let ext_low = ext.to_ascii_lowercase();
                        if !candidate.contains("://")
                            && CITATION_EXTS.contains(&ext_low.as_str())
                            && line_no > 0
                        {
                            let raw_span = format!(
                                "{}{}",
                                candidate,
                                &text[i..i + 1 + total_after]
                            );
                            if seen.insert((candidate.to_string(), line_no, end_line)) {
                                out.push((raw_span, candidate.to_string(), line_no, end_line));
                                if out.len() >= 32 {
                                    break;
                                }
                            }
                            i += 1 + total_after;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// Verify `[file:L##]` cross-file citations against live workspace files, exact line counts,
/// SHA-256 digests, and optional baseline SHA-256 digests (`expected_shas` -> `"stale"` on drift).
pub fn verify_text_citations_against(
    workspace_dir: &Path,
    text: &str,
    expected_shas: Option<&std::collections::HashMap<String, String>>,
) -> CitationReport {
    let extracted = extract_file_citations(text);
    // Cache per path: Option<(file_sha256, lines)>
    let mut file_cache: std::collections::HashMap<String, Option<(String, Vec<String>)>> =
        std::collections::HashMap::new();
    let mut report = CitationReport::default();

    for (raw, path, line, end_line) in extracted {
        if !file_cache.contains_key(&path) {
            let loaded = sandbox_path(workspace_dir, &path).ok().and_then(|full| {
                if full
                    .symlink_metadata()
                    .is_ok_and(|md| md.file_type().is_symlink())
                {
                    return None;
                }
                let raw_bytes = std::fs::read(&full).ok()?;
                let sha = crate::receipts::sha256_hex(&raw_bytes);
                let text_body = searchable_text(&full).ok()?;
                let lines: Vec<String> = text_body.lines().map(|l| l.to_string()).collect();
                Some((sha, lines))
            });
            file_cache.insert(path.clone(), loaded);
        }
        let cached = file_cache.get(&path).and_then(|v| v.as_ref());
        let expected_sha = expected_shas.and_then(|m| m.get(&path)).cloned();

        let (status, total_lines, file_sha256, line_excerpt) = match cached {
            None => ("missing", 0, String::new(), String::new()),
            Some((sha, lines)) => {
                let total = lines.len();
                let end_ok = end_line.is_none_or(|e| e >= line && e <= total);
                if line >= 1 && line <= total && end_ok {
                    let excerpt = truncate(lines[line - 1].trim(), 140);
                    let is_stale = expected_sha
                        .as_deref()
                        .is_some_and(|exp| !exp.is_empty() && !exp.eq_ignore_ascii_case(sha));
                    let st = if is_stale { "stale" } else { "verified" };
                    (st, total, sha.clone(), excerpt)
                } else {
                    ("out_of_range", total, sha.clone(), String::new())
                }
            }
        };
        match status {
            "verified" => report.verified += 1,
            "stale" => report.stale += 1,
            _ => report.broken += 1,
        }
        report.total += 1;
        report.citations.push(CitationCheck {
            raw,
            path,
            line,
            end_line,
            status,
            total_lines,
            file_sha256,
            line_excerpt,
            baseline_sha256: expected_sha,
        });
    }
    report
}

/// Verify `[file:L##]` cross-file citations against sandboxed files and exact line counts in `workspace_dir`.
pub fn verify_text_citations(workspace_dir: &Path, text: &str) -> CitationReport {
    verify_text_citations_against(workspace_dir, text, None)
}

