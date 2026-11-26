//! Rune scripting layer.
//!
//! Agent skills are Rune scripts (embedded `.rn` sources in `src/skills/`).
//! They do the formatting/shaping/analysis work themselves and perform side
//! effects only through host functions registered into the Rune context here:
//! workspace fs read/write/list, memory read/write, HTTP fetch, plus pure
//! deterministic CSV/JSON table parsers.

use crate::{fetch_url_text, sandbox_path, Config};
use rune::ast::Spanned;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub const WRITE_DOCUMENT_SCRIPT: &str = include_str!("skills/write_document.rn");
pub const UPDATE_MEMORY_SCRIPT: &str = include_str!("skills/update_memory.rn");
pub const MAKE_SPREADSHEET_SCRIPT: &str = include_str!("skills/make_spreadsheet.rn");
pub const RESEARCH_SCRIPT: &str = include_str!("skills/research.rn");
pub const RECONCILE_LEDGER_SCRIPT: &str = include_str!("skills/reconcile_ledger.rn");
pub const SUMMARIZE_MEETING_SCRIPT: &str = include_str!("skills/summarize_meeting.rn");
pub const AUDIT_CONTRACT_SCRIPT: &str = include_str!("skills/audit_contract.rn");
pub const TRIAGE_INBOX_SCRIPT: &str = include_str!("skills/triage_inbox.rn");
pub const CLEAN_DATASET_SCRIPT: &str = include_str!("skills/clean_dataset.rn");
pub const COMPARE_DOCUMENTS_SCRIPT: &str = include_str!("skills/compare_documents.rn");
pub const DRAFT_BRIEF_SCRIPT: &str = include_str!("skills/draft_brief.rn");
pub const EXTRACT_TABLES_SCRIPT: &str = include_str!("skills/extract_tables.rn");

pub const ALL_BUILTIN_SKILL_IDS: [&str; 12] = [
    "write_document",
    "make_spreadsheet",
    "research",
    "update_memory",
    "reconcile_ledger",
    "summarize_meeting",
    "audit_contract",
    "triage_inbox",
    "clean_dataset",
    "compare_documents",
    "draft_brief",
    "extract_tables",
];

/// Strict Rune VM instruction cap per skill invocation.
pub const RUNE_INSTRUCTION_BUDGET: usize = 5_000_000;
/// Strict Rune VM heap allocation cap (bytes) per skill invocation.
pub const RUNE_MEMORY_LIMIT_BYTES: usize = 128 * 1024 * 1024;

/// Parse RFC-4180 CSV text into rows of cells. Handles UTF-8 BOM, CRLF/LF,
/// quoted fields with embedded commas/newlines, and doubled quotes (`""`).
/// Pure and deterministic — performs zero I/O.
pub fn parse_csv_rows(raw: &str) -> Vec<Vec<String>> {
    let text = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cell.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                cell.push(ch);
            }
        } else {
            match ch {
                '"' if cell.is_empty() => {
                    in_quotes = true;
                }
                ',' => {
                    row.push(std::mem::take(&mut cell));
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut cell));
                    if !(row.len() == 1 && row[0].trim().is_empty()) {
                        rows.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                '\n' => {
                    row.push(std::mem::take(&mut cell));
                    if !(row.len() == 1 && row[0].trim().is_empty()) {
                        rows.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                other => cell.push(other),
            }
        }
    }

    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        if !(row.len() == 1 && row[0].trim().is_empty()) {
            rows.push(row);
        }
    }

    rows
}

fn json_cell_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Parse a JSON dataset (array of objects, object wrapping a record array,
/// single object, or array of arrays) into a rectangular string table whose
/// first row is the header list. Returns an empty Vec on invalid/unsupported JSON.
pub fn parse_json_table_rows(raw: &str) -> Vec<Vec<String>> {
    let trimmed = raw.strip_prefix('\u{feff}').unwrap_or(raw).trim();
    let Ok(parsed) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };

    let records: Vec<&Value> = match &parsed {
        Value::Array(arr) => arr.iter().collect(),
        Value::Object(map) => {
            let mut found: Option<&Vec<Value>> = None;
            for preferred in ["rows", "data", "items", "records", "messages", "tickets", "entries"] {
                if let Some(Value::Array(arr)) = map.get(preferred) {
                    found = Some(arr);
                    break;
                }
            }
            if found.is_none() {
                for val in map.values() {
                    if let Value::Array(arr) = val {
                        if arr.iter().any(|item| item.is_object()) {
                            found = Some(arr);
                            break;
                        }
                    }
                }
            }
            match found {
                Some(arr) => arr.iter().collect(),
                None => vec![&parsed],
            }
        }
        _ => return Vec::new(),
    };

    if records.is_empty() {
        return Vec::new();
    }

    // Handle array of arrays directly.
    if records.iter().all(|r| r.is_array()) {
        return records
            .into_iter()
            .filter_map(|r| r.as_array())
            .map(|arr| arr.iter().map(json_cell_to_string).collect())
            .collect();
    }

    let mut headers: Vec<String> = Vec::new();
    for rec in &records {
        if let Some(obj) = rec.as_object() {
            for key in obj.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }
    if headers.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<Vec<String>> = Vec::with_capacity(records.len() + 1);
    out.push(headers.clone());
    for rec in records {
        if let Some(obj) = rec.as_object() {
            let row = headers
                .iter()
                .map(|h| obj.get(h).map(json_cell_to_string).unwrap_or_default())
                .collect();
            out.push(row);
        }
    }
    out
}

/// `trusted` is true for Ergon's own built-in workflows. Library skills
/// (written by any person's agent and shared company-wide) run untrusted:
/// they may not rewrite the person's memory — memory is injected into
/// every future prompt, so a write there is a persistent instruction
/// channel into someone else's agent.
fn build_context(cfg: &Config, trusted: bool) -> Result<rune::Context, String> {
    let mut context =
        rune::Context::with_default_modules().map_err(|e| format!("rune context: {e}"))?;
    let mut module = rune::Module::new();

    // --- workspace fs host functions (sandboxed inside the workspace) ---
    let ws = cfg.workspace_dir.clone();
    let hist_data_dir = cfg.data_dir.clone();
    module
        .function(["host_workspace_write"], move |path: String, content: String| -> String {
            match sandbox_path(&ws, &path) {
                Ok(full) => {
                    if let Some(parent) = full.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return format!("error: create dirs: {e}");
                        }
                    }
                    let _ = crate::file_history::record_revision(
                        &hist_data_dir,
                        &ws,
                        &path,
                        "workflow_write",
                    );
                    match crate::atomic_write_bytes(&full, content.as_bytes()) {
                        Ok(()) => "ok".to_string(),
                        Err(e) => format!("error: write {path}: {e}"),
                    }
                }
                Err(e) => format!("error: {e}"),
            }
        })
        .build()
        .map_err(|e| format!("register host_workspace_write: {e}"))?;

    let ws = cfg.workspace_dir.clone();
    module
        .function(["host_workspace_read"], move |path: String| -> String {
            // Workflows read PDF, Word, Excel (`book.xlsx#Sheet`) and
            // PowerPoint as text, exactly like .md and .csv files.
            let (base, sheet) = crate::docs::split_sheet(&path);
            match sandbox_path(&ws, base) {
                Ok(full) => match crate::docs::doc_kind(base) {
                    Some(kind) => crate::docs::extract(&full, kind, sheet)
                        .map(|e| e.text)
                        .unwrap_or_else(|e| format!("error: read {path}: {e}")),
                    None => std::fs::read_to_string(&full)
                        .unwrap_or_else(|e| format!("error: read {path}: {e}")),
                },
                Err(e) => format!("error: {e}"),
            }
        })
        .build()
        .map_err(|e| format!("register host_workspace_read: {e}"))?;

    let ws = cfg.workspace_dir.clone();
    module
        .function(["host_workspace_list"], move |path: String| -> String {
            match sandbox_path(&ws, &path) {
                Ok(full) => match std::fs::read_dir(&full) {
                    Ok(rd) => {
                        let mut names: Vec<String> = rd
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect();
                        names.sort();
                        names.join("\n")
                    }
                    Err(e) => format!("error: list {path}: {e}"),
                },
                Err(e) => format!("error: {e}"),
            }
        })
        .build()
        .map_err(|e| format!("register host_workspace_list: {e}"))?;

    // --- memory host functions (the per-person memory.md) ---
    let mem = cfg.memory_path.clone();
    module
        .function(["host_memory_read"], move || -> String {
            let content = std::fs::read_to_string(&mem).unwrap_or_else(|_| "# Memory\n\n".to_string());
            if content.is_empty() {
                "# Memory\n\n".to_string()
            } else if content.ends_with('\n') {
                content
            } else {
                format!("{content}\n")
            }
        })
        .build()
        .map_err(|e| format!("register host_memory_read: {e}"))?;

    let mem = cfg.memory_path.clone();
    module
        .function(["host_memory_write"], move |content: String| -> String {
            if !trusted {
                crate::tripwire::library_memory_write_refused();
                return "error: library skills cannot rewrite memory — the agent records what it learns with update_memory".to_string();
            }
            if let Some(parent) = mem.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return format!("error: create dirs: {e}");
                }
            }
            match std::fs::write(&mem, &content) {
                Ok(()) => "ok".to_string(),
                Err(e) => format!("error: write memory: {e}"),
            }
        })
        .build()
        .map_err(|e| format!("register host_memory_write: {e}"))?;

    // --- http host function ---
    module
        .function(["host_fetch_url"], |url: String| -> String {
            fetch_url_text(&url).unwrap_or_else(|e| format!("error: {e}"))
        })
        .build()
        .map_err(|e| format!("register host_fetch_url: {e}"))?;

    // --- pure deterministic data parsers (zero I/O) ---
    module
        .function(["host_parse_csv"], |raw: String| -> Vec<Vec<String>> {
            parse_csv_rows(&raw)
        })
        .build()
        .map_err(|e| format!("register host_parse_csv: {e}"))?;

    module
        .function(["host_parse_json_table"], |raw: String| -> Vec<Vec<String>> {
            parse_json_table_rows(&raw)
        })
        .build()
        .map_err(|e| format!("register host_parse_json_table: {e}"))?;

    context
        .install(module)
        .map_err(|e| format!("install ergon host module: {e}"))?;
    Ok(context)
}

fn format_rune_diagnostics(
    err: impl std::fmt::Display,
    diagnostics: &rune::Diagnostics,
    sources: &rune::Sources,
) -> String {
    let mut details = Vec::new();
    for d in diagnostics.diagnostics() {
        let (prefix, id, span_opt, msg) = match d {
            rune::diagnostics::Diagnostic::Fatal(f) => {
                let sp = match f.kind() {
                    rune::diagnostics::FatalDiagnosticKind::CompileError(e) => Some(e.span()),
                    _ => None,
                };
                ("", f.source_id(), sp, f.to_string())
            }
            rune::diagnostics::Diagnostic::Warning(w) => {
                ("warning: ", w.source_id(), Some(w.span()), w.to_string())
            }
            rune::diagnostics::Diagnostic::RuntimeWarning(r) => {
                details.push(format!("warning: {r}"));
                continue;
            }
            _ => continue,
        };
        let loc = span_opt
            .and_then(|sp| sources.get(id).map(|s| s.pos_to_utf8_linecol(sp.start.into_usize())))
            .map(|(line, col)| format!("line {} col {}: ", line + 1, col + 1))
            .unwrap_or_default();
        details.push(format!("{prefix}{loc}{msg}"));
    }
    if details.is_empty() {
        format!("rune skill failed to build: {err}")
    } else {
        format!("rune skill failed to build: {err}\n{}", details.join("\n"))
    }
}

/// Compile a script against the host context without running it.
/// Publishing a skill into the company library goes through this
/// first: a script that does not build is refused, never stored.
pub fn check_script(cfg: &Config, script: &str) -> Result<(), String> {
    let context = build_context(cfg, false)?;
    let mut sources = rune::Sources::new();
    sources
        .insert(rune::Source::memory(script).map_err(|e| format!("rune source: {e}"))?)
        .map_err(|e| format!("rune sources: {e}"))?;
    let mut diagnostics = rune::Diagnostics::new();
    rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build()
        .map_err(|e| format_rune_diagnostics(e, &diagnostics, &sources))?;
    Ok(())
}

fn run_script<A>(cfg: &Config, script: &str, args: A) -> Result<String, String>
where
    A: rune::runtime::Args + rune::runtime::GuardedArgs,
{
    run_script_with(cfg, script, args, true)
}

fn run_script_with<A>(cfg: &Config, script: &str, args: A, trusted: bool) -> Result<String, String>
where
    A: rune::runtime::Args + rune::runtime::GuardedArgs,
{
    let context = build_context(cfg, trusted)?;
    let runtime = Arc::new(context.runtime().map_err(|e| format!("rune runtime: {e}"))?);
    let mut sources = rune::Sources::new();
    sources
        .insert(rune::Source::memory(script).map_err(|e| format!("rune source: {e}"))?)
        .map_err(|e| format!("rune sources: {e}"))?;
    let mut diagnostics = rune::Diagnostics::new();
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build()
        .map_err(|e| format_rune_diagnostics(e, &diagnostics, &sources))?;
    let mut vm = rune::Vm::new(runtime, Arc::new(unit));
    // Every skill runs under an instruction budget AND a memory
    // limit: a looping or allocating script — including one an agent
    // published to the library — must fail, never freeze the server
    // or eat the machine. (The budget cannot preempt a host function
    // mid-call; those are individually bounded, e.g. fetch's 20 s
    // timeout.)
    let value = rune::runtime::budget::with(
        RUNE_INSTRUCTION_BUDGET,
        rune::alloc::limit::with(RUNE_MEMORY_LIMIT_BYTES, move || vm.call(["main"], args)),
    )
    .call()
    .map_err(|e| format!("rune skill call failed: {e}"))?;
    rune::from_value::<String>(value).map_err(|e| format!("rune skill returned non-string: {e}"))
}

/// Interpret a file-writing skill's return value. Success is prefixed
/// "file:" by the skills themselves, so a file whose name legitimately
/// starts with "error:" (a document titled "error: Weekly Report") is
/// never mistaken for a host error — the old protocol sniffed the bare
/// "error:" prefix and reported success as failure.
fn skill_filename(returned: &str) -> Result<String, String> {
    if let Some(name) = returned.strip_prefix("file:") {
        return Ok(name.to_string());
    }
    if let Some(err) = returned.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
    Ok(returned.to_string())
}

/// Interpret either single-file (`file:<name>`) or multi-file (`files:<a|b>`)
/// return values from Rune skills.
fn skill_filenames(returned: &str) -> Result<Vec<String>, String> {
    if let Some(list) = returned.strip_prefix("files:") {
        let names: Vec<String> = list
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if names.is_empty() {
            return Err("skill returned empty file list".to_string());
        }
        return Ok(names);
    }
    Ok(vec![skill_filename(returned)?])
}

fn format_written_files(cfg: &Config, verb: &str, files: &[String]) -> Result<String, String> {
    let mut parts: Vec<String> = Vec::with_capacity(files.len());
    for f in files {
        let full = sandbox_path(&cfg.workspace_dir, f)?;
        let len = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
        parts.push(format!("{f} ({len} bytes)"));
    }
    Ok(format!("wrote {verb} {}", parts.join(" and ")))
}

// ---------------------------------------------------------------------------
// Built-in Rune Skills (12 total — original 4 + 8 v0.5.0 employee-grade skills)
// ---------------------------------------------------------------------------

/// Rune skill: format title + sections into a finished markdown document
/// in the workspace. Returns a human-readable tool result.
pub fn write_document(cfg: &Config, title: &str, sections: Vec<String>) -> Result<String, String> {
    let returned = run_script(cfg, WRITE_DOCUMENT_SCRIPT, (title.to_string(), sections))?;
    let filename = skill_filename(&returned)?;
    let full = sandbox_path(&cfg.workspace_dir, &filename)?;
    let len = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
    Ok(format!("wrote document {filename} ({len} bytes)"))
}

/// Rune skill: format columns + rows into a CSV spreadsheet file in the
/// workspace. Returns a human-readable tool result.
pub fn make_spreadsheet(
    cfg: &Config,
    title: &str,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
) -> Result<String, String> {
    let n_rows = rows.len();
    let returned = run_script(cfg, MAKE_SPREADSHEET_SCRIPT, (title.to_string(), columns, rows))?;
    let filename = skill_filename(&returned)?;
    let full = sandbox_path(&cfg.workspace_dir, &filename)?;
    let len = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
    Ok(format!(
        "wrote spreadsheet {filename} ({n_rows} data rows, {len} bytes)"
    ))
}

/// Rune skill: fetch several pages and compile them into one research
/// notes document in the workspace. Returns a human-readable tool result.
pub fn research(cfg: &Config, topic: &str, urls: Vec<String>) -> Result<String, String> {
    let filename = format!("Research - {topic}.md");
    let returned = run_script(
        cfg,
        RESEARCH_SCRIPT,
        (topic.to_string(), filename.clone(), urls),
    )?;
    let written = skill_filename(&returned)?;
    let full = sandbox_path(&cfg.workspace_dir, &written)?;
    let len = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
    // No source count here: the notes document itself states how many
    // sources actually loaded (counted inside the skill, where the
    // fetch results are known). A count of attempted URLs would read
    // as a count of fetched pages.
    Ok(format!("wrote research notes {written} ({len} bytes)"))
}

/// Rune skill: append a formatted learned note to memory.md.
/// Returns a human-readable tool result.
pub fn update_memory(cfg: &Config, note: &str) -> Result<String, String> {
    // A learned note becomes ONE bullet in memory.md. Newlines in the
    // note would let it forge section headings or extra bullets in the
    // file (structure injection into every future system prompt), so
    // the note is flattened to a single line before the skill sees it.
    let note = note.replace(['\n', '\r'], " ");
    let note = note.trim();
    if note.is_empty() {
        return Err("update_memory needs a non-empty note".to_string());
    }
    // The same lesson learned twice is one bullet, not two: every
    // duplicate would ride along in every future prompt.
    let norm = |s: &str| -> String {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim_end_matches(['.', '!', ';'])
            .to_lowercase()
    };
    let wanted = norm(note);
    let memory = cfg.read_memory();
    if memory
        .lines()
        .filter_map(|l| l.trim().strip_prefix("- "))
        .any(|b| norm(b) == wanted)
    {
        return Ok(format!("already in memory (not added again): - {note}"));
    }
    let entry = run_script(cfg, UPDATE_MEMORY_SCRIPT, (note.to_string(),))?;
    if let Some(err) = entry.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
    Ok(format!("memory updated: {entry}"))
}

fn normalize_memory_rule(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(['.', '!', ';'])
        .to_lowercase()
}

fn is_learned_bullet(raw_line: &str) -> Option<&str> {
    let t = raw_line.trim();
    let rest = t.strip_prefix("- ")?.trim();
    if rest.is_empty() || t.starts_with("- Name:") || t.starts_with("- Role:") {
        return None;
    }
    Some(rest)
}

fn memory_rule_polarity_and_topics(norm: &str) -> (i8, Option<(&'static str, String)>, Vec<String>) {
    let neg_markers = [
        "never ",
        "avoid ",
        "do not ",
        "don't ",
        "dont ",
        "exclude ",
        "omit ",
        "skip ",
        "disable ",
        "no ",
    ];
    let pos_markers = [
        "always ",
        "use ",
        "include ",
        "enable ",
        "keep ",
        "require ",
        "prefer ",
    ];
    let has_neg = neg_markers.iter().any(|m| norm.starts_with(m) || norm.contains(&format!(" {m}")));
    let has_pos = pos_markers.iter().any(|m| norm.starts_with(m) || norm.contains(&format!(" {m}")));
    let pol = match (has_pos, has_neg) {
        (true, false) => 1,
        (false, true) => -1,
        (true, true) => -1,
        _ => 0,
    };
    let pref_prefixes = [
        "prefer ",
        "default format ",
        "export format ",
        "tone: ",
        "currency: ",
        "rounding: ",
        "date format: ",
        "language: ",
    ];
    let pref = pref_prefixes.iter().find_map(|p| {
        norm.strip_prefix(p)
            .map(|val| (*p, val.trim().to_string()))
            .filter(|(_, val)| !val.is_empty())
    });
    const STOP: &[&str] = &[
        "the", "and", "for", "with", "from", "that", "this", "use", "always", "never", "avoid",
        "not", "dont", "don", "prefer", "include", "exclude", "omit", "skip", "keep", "only",
        "must", "should", "when", "into", "all", "are", "was", "were", "have", "has", "any",
        "enable", "disable", "require",
    ];
    let mut topics: Vec<String> = norm
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOP.contains(w))
        .map(|w| w.to_string())
        .collect();
    topics.sort();
    topics.dedup();
    (pol, pref, topics)
}

fn strip_bracketed_citations(rule: &str) -> String {
    let mut out = String::with_capacity(rule.len());
    let mut rest = rule;
    while let Some(start) = rest.find('[') {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find(']') {
            let inner = &rest[start + 1..start + end];
            if inner.contains(":L") {
                out.push(' ');
                rest = &rest[start + end + 1..];
                continue;
            }
        }
        out.push('[');
        rest = &rest[start + 1..];
    }
    out.push_str(rest);
    out
}

fn extract_workspace_file_refs(rule: &str) -> Vec<String> {
    const EXTS: &[&str] = &[
        ".csv", ".md", ".json", ".txt", ".docx", ".xlsx", ".pptx", ".pdf", ".rn", ".tsv", ".yaml",
        ".yml",
    ];
    let stripped = strip_bracketed_citations(rule);
    let mut refs = Vec::new();
    // 1. Check backtick-delimited paths first (supports spaces in filenames like `Q1 budget.csv`).
    let mut rem = stripped.as_str();
    while let Some(a) = rem.find('`') {
        let after = &rem[a + 1..];
        let Some(b) = after.find('`') else {
            break;
        };
        let cand = after[..b].trim();
        let low = cand.to_ascii_lowercase();
        if cand.len() >= 3
            && !low.starts_with("http://")
            && !low.starts_with("https://")
            && EXTS.iter().any(|ext| low.ends_with(ext))
            && !refs.iter().any(|r| r == cand)
        {
            refs.push(cand.to_string());
        }
        rem = &after[b + 1..];
    }
    // 2. Scan unbracketed, un-backticked whitespace tokens.
    let without_backticks: String = stripped
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, s)| s)
        .collect::<Vec<_>>()
        .join(" ");
    for tok in without_backticks.split_whitespace() {
        let cleaned = tok.trim_matches(|c: char| {
            matches!(c, '\'' | '"' | '(' | ')' | '[' | ']' | ',' | ';' | ':' | '.' | '!')
        });
        let base = match cleaned.rfind(":L") {
            Some(pos)
                if cleaned[pos + 2..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit()) =>
            {
                &cleaned[..pos]
            }
            _ => cleaned,
        };
        let low = base.to_ascii_lowercase();
        if base.len() >= 3
            && !low.starts_with("http://")
            && !low.starts_with("https://")
            && EXTS.iter().any(|ext| low.ends_with(ext))
            && !refs.iter().any(|r| r == base)
        {
            refs.push(base.to_string());
        }
    }
    refs
}

fn check_rule_stale_ref(workspace_dir: &Path, rule: &str) -> Option<String> {
    let report = crate::verify_text_citations(workspace_dir, rule);
    if report.broken > 0 {
        if let Some(bad) = report
            .citations
            .iter()
            .find(|c| c.status == "missing" || c.status == "out_of_range")
        {
            if bad.status == "missing" {
                return Some(format!("Missing cited file `[{}]`", bad.raw));
            }
            return Some(format!(
                "Out-of-range citation `[{}]` (file has {} lines)",
                bad.raw, bad.total_lines
            ));
        }
    }
    for rel in extract_workspace_file_refs(rule) {
        if rel == "memory.md" {
            continue;
        }
        let ok = sandbox_path(workspace_dir, &rel).is_ok_and(|p| {
            p.exists() && !p.symlink_metadata().is_ok_and(|m| m.file_type().is_symlink())
        });
        if !ok {
            return Some(format!("Missing workspace file `{rel}`"));
        }
    }
    None
}

#[derive(Clone)]
struct MemoryIssue {
    line: usize,
    kind: &'static str,
    text: String,
    detail: String,
    related_line: Option<usize>,
}

fn scan_memory_issues(cfg: &Config, memory: &str) -> (usize, Vec<MemoryIssue>) {
    struct Entry {
        line: usize,
        text: String,
        norm: String,
        pol: i8,
        pref: Option<(&'static str, String)>,
        topics: Vec<String>,
        is_dup: bool,
    }

    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut issues: Vec<MemoryIssue> = Vec::new();

    for (idx, raw) in memory.lines().enumerate() {
        let line_no = idx + 1;
        let Some(rule) = is_learned_bullet(raw) else {
            continue;
        };
        let norm = normalize_memory_rule(rule);
        let is_dup = if let Some(&first_line) = seen.get(&norm) {
            issues.push(MemoryIssue {
                line: line_no,
                kind: "duplicate",
                text: rule.to_string(),
                detail: format!("Duplicate of L{first_line}"),
                related_line: Some(first_line),
            });
            true
        } else {
            seen.insert(norm.clone(), line_no);
            false
        };

        if let Some(detail) = check_rule_stale_ref(&cfg.workspace_dir, rule) {
            issues.push(MemoryIssue {
                line: line_no,
                kind: "stale_ref",
                text: rule.to_string(),
                detail,
                related_line: None,
            });
        }

        let (pol, pref, topics) = memory_rule_polarity_and_topics(&norm);
        entries.push(Entry {
            line: line_no,
            text: rule.to_string(),
            norm,
            pol,
            pref,
            topics,
            is_dup,
        });
    }

    // Conflict pass: flag older superseded rule when a newer distinct rule contradicts it.
    let mut conflicted_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for i in 0..entries.len() {
        if entries[i].is_dup {
            continue;
        }
        for j in (i + 1)..entries.len() {
            if entries[j].is_dup || entries[i].norm == entries[j].norm {
                continue;
            }
            let same_pref_conflict = match (&entries[i].pref, &entries[j].pref) {
                (Some((p1, v1)), Some((p2, v2))) => p1 == p2 && v1 != v2,
                _ => false,
            };
            let opposite_pol = entries[i].pol * entries[j].pol == -1;
            let shared_topics = if opposite_pol {
                entries[i]
                    .topics
                    .iter()
                    .filter(|t| entries[j].topics.contains(t))
                    .count()
            } else {
                0
            };
            let min_topics = entries[i].topics.len().min(entries[j].topics.len());
            let topic_conflict = opposite_pol
                && ((shared_topics >= 2) || (min_topics > 0 && min_topics <= 2 && shared_topics >= 1));

            if (same_pref_conflict || topic_conflict) && conflicted_lines.insert(entries[i].line) {
                issues.push(MemoryIssue {
                    line: entries[i].line,
                    kind: "conflict",
                    text: entries[i].text.clone(),
                    detail: format!(
                        "{} L{}",
                        if same_pref_conflict {
                            "Superseded by preference at"
                        } else {
                            "Conflicts with newer rule at"
                        },
                        entries[j].line
                    ),
                    related_line: Some(entries[j].line),
                });
                break;
            }
        }
    }

    issues.sort_by_key(|x| (x.line, x.kind));
    (entries.len(), issues)
}

fn memory_health_json(learned_count: usize, issues: &[MemoryIssue]) -> Value {
    let mut conflicts = 0usize;
    let mut duplicates = 0usize;
    let mut stale_refs = 0usize;
    let mut issue_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let issue_vals: Vec<Value> = issues
        .iter()
        .map(|i| {
            issue_lines.insert(i.line);
            match i.kind {
                "conflict" => conflicts += 1,
                "duplicate" => duplicates += 1,
                "stale_ref" => stale_refs += 1,
                _ => {}
            }
            serde_json::json!({
                "line": i.line,
                "kind": i.kind,
                "text": i.text,
                "detail": i.detail,
                "related_line": i.related_line,
                "removable": true,
            })
        })
        .collect();
    let issue_count = issue_lines.len();
    serde_json::json!({
        "status": if issue_count == 0 { "healthy" } else { "issues" },
        "learned_count": learned_count,
        "issue_count": issue_count,
        "conflicts": conflicts,
        "duplicates": duplicates,
        "stale_refs": stale_refs,
        "issues": issue_vals,
    })
}

/// Audit standing `memory.md` rules for duplicates, contradictions, and broken workspace paths.
pub fn audit_memory_health(cfg: &Config) -> Value {
    let memory = cfg.read_memory();
    let (learned_count, issues) = scan_memory_issues(cfg, &memory);
    memory_health_json(learned_count, &issues)
}

/// Prune stale references, duplicate entries, and/or superseded conflicting rules from `memory.md`.
pub fn heal_memory(cfg: &Config, mode: &str) -> Result<Value, String> {
    let mode = mode.trim().to_ascii_lowercase();
    let mode = if mode.is_empty() { "all" } else { mode.as_str() };
    if !matches!(mode, "all" | "stale" | "conflicts" | "duplicates") {
        return Err("mode must be all, stale, conflicts, or duplicates".to_string());
    }
    let memory = cfg.read_memory();
    let (_, issues) = scan_memory_issues(cfg, &memory);
    let mut remove_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for iss in &issues {
        let take = match mode {
            "all" => true,
            "stale" => iss.kind == "stale_ref",
            "conflicts" => iss.kind == "conflict",
            "duplicates" => iss.kind == "duplicate",
            _ => false,
        };
        if take {
            remove_lines.insert(iss.line);
        }
    }
    if remove_lines.is_empty() {
        let (learned_count, fresh_issues) = scan_memory_issues(cfg, &memory);
        return Ok(serde_json::json!({
            "content": memory,
            "removed_count": 0,
            "removed": [],
            "health": memory_health_json(learned_count, &fresh_issues),
        }));
    }

    let mut kept = Vec::new();
    let mut removed = Vec::new();
    for (idx, line) in memory.lines().enumerate() {
        let line_no = idx + 1;
        if remove_lines.contains(&line_no) {
            removed.push(serde_json::json!({
                "line": line_no,
                "text": line.trim(),
            }));
        } else {
            kept.push(line);
        }
    }
    let mut next_content = kept.join("\n");
    if !next_content.ends_with('\n') {
        next_content.push('\n');
    }
    crate::tools::atomic_write_bytes(&cfg.memory_path, next_content.as_bytes())
        .map_err(|e| format!("failed to write memory.md: {e}"))?;
    let (learned_count, fresh_issues) = scan_memory_issues(cfg, &next_content);
    Ok(serde_json::json!({
        "content": next_content,
        "removed_count": removed.len(),
        "removed": removed,
        "health": memory_health_json(learned_count, &fresh_issues),
    }))
}

/// Rune skill: reconcile one or more workspace CSV invoice/expense ledgers,
/// flagging duplicate references, missing receipts, and amount anomalies,
/// and emitting vendor/category rollups as both Markdown and CSV.
pub fn reconcile_ledger(
    cfg: &Config,
    source_files: Vec<String>,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        RECONCILE_LEDGER_SCRIPT,
        (source_files, output_title.to_string()),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "ledger reconciliation", &files)
}

/// Rune skill: extract attendees, decisions, owners, due dates, open
/// questions, and ready-to-send follow-up drafts from a meeting transcript.
pub fn summarize_meeting(
    cfg: &Config,
    notes_file: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        SUMMARIZE_MEETING_SCRIPT,
        (notes_file.to_string(), output_title.to_string()),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "meeting summary", &files)
}

/// Rune skill: audit a contract or agreement for required clauses, renewal
/// and notice windows, missing-clause flags, and a line-cited risk table.
pub fn audit_contract(
    cfg: &Config,
    contract_file: &str,
    required_clauses: Vec<String>,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        AUDIT_CONTRACT_SCRIPT,
        (
            contract_file.to_string(),
            required_clauses,
            output_title.to_string(),
        ),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "contract audit", &files)
}

/// Rune skill: triage messages or tickets into Urgent / Action / FYI buckets
/// with concrete next steps and source citations.
pub fn triage_inbox(
    cfg: &Config,
    inbox_file: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        TRIAGE_INBOX_SCRIPT,
        (inbox_file.to_string(), output_title.to_string()),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "inbox triage", &files)
}

/// Rune skill: normalize a CSV or JSON dataset schema, deduplicate rows,
/// flag nulls and statistical outliers, and emit a clean CSV + diff report.
pub fn clean_dataset(
    cfg: &Config,
    input_file: &str,
    key_column: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        CLEAN_DATASET_SCRIPT,
        (
            input_file.to_string(),
            key_column.to_string(),
            output_title.to_string(),
        ),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "cleaned dataset", &files)
}

/// Rune skill: compute a line-by-line diff and executive change summary
/// between two workspace files.
pub fn compare_documents(
    cfg: &Config,
    left_file: &str,
    right_file: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        COMPARE_DOCUMENTS_SCRIPT,
        (
            left_file.to_string(),
            right_file.to_string(),
            output_title.to_string(),
        ),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "document comparison", &files)
}

/// Rune skill: draft an executive memo or SOP grounded strictly in named
/// workspace files and standing memory.
pub fn draft_brief(
    cfg: &Config,
    topic: &str,
    source_files: Vec<String>,
    audience: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        DRAFT_BRIEF_SCRIPT,
        (
            topic.to_string(),
            source_files,
            audience.to_string(),
            output_title.to_string(),
        ),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "executive brief", &files)
}

/// Rune skill: parse unstructured Markdown tables, key=value logs, TSV, or
/// record blocks into RFC-4180 CSV spreadsheet(s).
pub fn extract_tables(
    cfg: &Config,
    source_file: &str,
    output_title: &str,
) -> Result<String, String> {
    let returned = run_script(
        cfg,
        EXTRACT_TABLES_SCRIPT,
        (source_file.to_string(), output_title.to_string()),
    )?;
    let files = skill_filenames(&returned)?;
    format_written_files(cfg, "extracted table", &files)
}

/// Canonicalize either snake_case or kebab-case skill name into its
/// built-in tool id (`reconcile-ledger` -> `reconcile_ledger`).
pub fn canonical_builtin_skill_id(name: &str) -> Option<&'static str> {
    let norm = name.trim().to_ascii_lowercase().replace('-', "_");
    ALL_BUILTIN_SKILL_IDS
        .iter()
        .copied()
        .find(|id| *id == norm)
}

/// Whether `name` (in snake_case or kebab-case) identifies one of the 12
/// built-in Rune skills.
pub fn is_builtin_skill(name: &str) -> bool {
    canonical_builtin_skill_id(name).is_some()
}

fn arg_str<'a>(args: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some(s) = args.get(*k).and_then(|v| v.as_str()) {
            return Some(s);
        }
    }
    None
}

fn arg_string_list(args: &Value, keys: &[&str]) -> Vec<String> {
    for k in keys {
        if let Some(v) = args.get(*k) {
            match v {
                Value::Array(items) => {
                    return items
                        .iter()
                        .map(|item| match item {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect();
                }
                Value::String(s) if !s.trim().is_empty() => {
                    return s
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }
    Vec::new()
}

fn arg_string_rows(args: &Value, key: &str) -> Vec<Vec<String>> {
    match args.get(key) {
        Some(Value::Array(rows)) => rows
            .iter()
            .map(|r| match r {
                Value::Array(cells) => cells
                    .iter()
                    .map(|c| match c {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect(),
                Value::String(s) => vec![s.clone()],
                _ => Vec::new(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Unified dispatcher for all 12 built-in Rune skills from a JSON `args`
/// payload. Returns `None` if `name` is not a built-in skill.
pub fn dispatch_builtin_skill(
    cfg: &Config,
    name: &str,
    args: &Value,
) -> Option<Result<String, String>> {
    let canonical = canonical_builtin_skill_id(name)?;
    let res = match canonical {
        "write_document" => {
            let title = match arg_str(args, &["title", "output_title"]) {
                Some(t) if !t.trim().is_empty() => t,
                _ => return Some(Err("write_document needs args.title".to_string())),
            };
            let sections = arg_string_list(args, &["sections", "body", "content"]);
            if sections.is_empty() {
                return Some(Err("write_document needs args.sections".to_string()));
            }
            write_document(cfg, title, sections)
        }
        "make_spreadsheet" => {
            let title = match arg_str(args, &["title", "output_title"]) {
                Some(t) if !t.trim().is_empty() => t,
                _ => return Some(Err("make_spreadsheet needs args.title".to_string())),
            };
            let columns = arg_string_list(args, &["columns", "headers"]);
            let rows = arg_string_rows(args, "rows");
            if columns.is_empty() && rows.is_empty() {
                return Some(Err(
                    "make_spreadsheet needs args.columns or args.rows".to_string(),
                ));
            }
            make_spreadsheet(cfg, title, columns, rows)
        }
        "research" => {
            let topic = match arg_str(args, &["topic", "title"]) {
                Some(t) if !t.trim().is_empty() => t,
                _ => return Some(Err("research needs args.topic".to_string())),
            };
            let urls = arg_string_list(args, &["urls", "url"]);
            if urls.is_empty() {
                return Some(Err(
                    "research needs args.urls (a list of page URLs)".to_string(),
                ));
            }
            research(cfg, topic, urls)
        }
        "update_memory" => {
            let note = match arg_str(args, &["note", "content", "text"]) {
                Some(n) => n,
                None => return Some(Err("update_memory needs args.note".to_string())),
            };
            update_memory(cfg, note)
        }
        "reconcile_ledger" => {
            let source_files = arg_string_list(args, &["source_files", "files", "file", "path"]);
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Ledger Reconciliation");
            reconcile_ledger(cfg, source_files, output_title)
        }
        "summarize_meeting" => {
            let notes_file = arg_str(args, &["notes_file", "file", "path", "transcript_file"]).unwrap_or("");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Meeting Summary");
            summarize_meeting(cfg, notes_file, output_title)
        }
        "audit_contract" => {
            let contract_file = arg_str(args, &["contract_file", "file", "path"]).unwrap_or("");
            let required_clauses = arg_string_list(args, &["required_clauses", "clauses"]);
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Contract Audit");
            audit_contract(cfg, contract_file, required_clauses, output_title)
        }
        "triage_inbox" => {
            let inbox_file = arg_str(args, &["inbox_file", "file", "path"]).unwrap_or("");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Inbox Triage");
            triage_inbox(cfg, inbox_file, output_title)
        }
        "clean_dataset" => {
            let input_file = arg_str(args, &["input_file", "file", "path"]).unwrap_or("");
            let key_column = arg_str(args, &["key_column", "dedup_key", "key"]).unwrap_or("");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Cleaned Dataset");
            clean_dataset(cfg, input_file, key_column, output_title)
        }
        "compare_documents" => {
            let left_file = arg_str(args, &["left_file", "before", "old_file", "file_a"]).unwrap_or("");
            let right_file = arg_str(args, &["right_file", "after", "new_file", "file_b"]).unwrap_or("");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Document Comparison");
            compare_documents(cfg, left_file, right_file, output_title)
        }
        "draft_brief" => {
            let topic = arg_str(args, &["topic", "subject"]).unwrap_or("");
            let source_files = arg_string_list(args, &["source_files", "files", "sources", "file"]);
            let audience = arg_str(args, &["audience", "recipient"]).unwrap_or("Team stakeholders");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("");
            draft_brief(cfg, topic, source_files, audience, output_title)
        }
        "extract_tables" => {
            let source_file = arg_str(args, &["source_file", "file", "path"]).unwrap_or("");
            let output_title = arg_str(args, &["output_title", "title"]).unwrap_or("Extracted Table");
            extract_tables(cfg, source_file, output_title)
        }
        _ => return None,
    };
    Some(res)
}

/// Run a LIBRARY skill: a Rune script from the company library,
/// following the library convention — `main` takes the item's
/// declared parameters as strings, in order (at most 5), and returns
/// a string: plain result text, `file:<name>` when it wrote a
/// workspace file, or `error:...` on failure. The return is shaped
/// into the same kind of tool result the built-in skills produce.
pub fn run_library_skill(cfg: &Config, source: &str, values: &[String]) -> Result<String, String> {
    let v = |i: usize| values.get(i).cloned().unwrap_or_default();
    let returned = match values.len() {
        0 => run_script_with(cfg, source, (), false)?,
        1 => run_script_with(cfg, source, (v(0),), false)?,
        2 => run_script_with(cfg, source, (v(0), v(1)), false)?,
        3 => run_script_with(cfg, source, (v(0), v(1), v(2)), false)?,
        4 => run_script_with(cfg, source, (v(0), v(1), v(2), v(3)), false)?,
        5 => run_script_with(cfg, source, (v(0), v(1), v(2), v(3), v(4)), false)?,
        _ => return Err("library skills take at most 5 parameters".to_string()),
    };
    if let Some(err) = returned.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
    if returned.starts_with("files:") {
        let files = skill_filenames(&returned)?;
        return format_written_files(cfg, "output", &files);
    }
    if let Some(name) = returned.strip_prefix("file:") {
        let full = sandbox_path(&cfg.workspace_dir, name)?;
        let len = std::fs::metadata(&full).map(|m| m.len()).unwrap_or(0);
        return Ok(format!("wrote {name} ({len} bytes)"));
    }
    Ok(returned)
}
