//! Rune scripting layer.
//!
//! Agent skills are Rune scripts (embedded `.rn` sources in `src/skills/`).
//! They do the formatting/shaping/analysis work themselves and perform side
//! effects only through host functions registered into the Rune context here:
//! workspace fs read/write/list, memory read/write, HTTP fetch, plus pure
//! deterministic CSV/JSON table parsers.

use crate::{fetch_url_text, sandbox_path, Config};
use serde_json::Value;
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
                    match std::fs::write(&full, &content) {
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
    let mut buf = rune::termcolor::Buffer::no_color();
    if diagnostics.emit(&mut buf, sources).is_ok() {
        let rendered = String::from_utf8_lossy(buf.as_slice()).trim().to_string();
        if !rendered.is_empty() {
            return format!("rune skill failed to build: {err}\n{rendered}");
        }
    }
    format!("rune skill failed to build: {err}")
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
