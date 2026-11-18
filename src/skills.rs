//! Rune scripting layer.
//!
//! Agent skills are Rune scripts (embedded `.rn` sources in `src/skills/`).
//! They do the formatting/shaping work themselves and perform side effects
//! only through host functions registered into the Rune context here:
//! workspace fs read/write/list, memory read/write, and HTTP fetch.
//! The agent loop invokes these scripts for the `write_document` and
//! `update_memory` tools — Rune genuinely drives that behaviour.

use crate::{fetch_url_text, sandbox_path, Config};
use std::sync::Arc;

pub const WRITE_DOCUMENT_SCRIPT: &str = include_str!("skills/write_document.rn");
pub const UPDATE_MEMORY_SCRIPT: &str = include_str!("skills/update_memory.rn");
pub const MAKE_SPREADSHEET_SCRIPT: &str = include_str!("skills/make_spreadsheet.rn");
pub const RESEARCH_SCRIPT: &str = include_str!("skills/research.rn");

fn build_context(cfg: &Config) -> Result<rune::Context, String> {
    let mut context =
        rune::Context::with_default_modules().map_err(|e| format!("rune context: {e}"))?;
    let mut module = rune::Module::new();

    // --- workspace fs host functions (sandboxed inside the workspace) ---
    let ws = cfg.workspace_dir.clone();
    module
        .function(["host_workspace_write"], move |path: String, content: String| -> String {
            match sandbox_path(&ws, &path) {
                Ok(full) => {
                    if let Some(parent) = full.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return format!("error: create dirs: {e}");
                        }
                    }
                    match std::fs::write(&full, &content) {
                        Ok(()) => "ok".to_string(),
                        Err(e) => format!("error: write {}: {e}", full.display()),
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
            match sandbox_path(&ws, &path) {
                Ok(full) => std::fs::read_to_string(&full)
                    .unwrap_or_else(|e| format!("error: read {}: {e}", full.display())),
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
                    Err(e) => format!("error: list {}: {e}", full.display()),
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

    context
        .install(module)
        .map_err(|e| format!("install ergon host module: {e}"))?;
    Ok(context)
}

fn run_script<A>(cfg: &Config, script: &str, args: A) -> Result<String, String>
where
    A: rune::runtime::Args + rune::runtime::GuardedArgs,
{
    let context = build_context(cfg)?;
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
        .map_err(|e| format!("rune skill failed to build: {e}"))?;
    let mut vm = rune::Vm::new(runtime, Arc::new(unit));
    let value = vm
        .call(["main"], args)
        .map_err(|e| format!("rune skill call failed: {e}"))?;
    rune::from_value::<String>(value).map_err(|e| format!("rune skill returned non-string: {e}"))
}

/// Rune skill: format title + sections into a finished markdown document
/// in the workspace. Returns a human-readable tool result.
pub fn write_document(cfg: &Config, title: &str, sections: Vec<String>) -> Result<String, String> {
    let filename = run_script(cfg, WRITE_DOCUMENT_SCRIPT, (title.to_string(), sections))?;
    if let Some(err) = filename.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
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
    let filename = run_script(cfg, MAKE_SPREADSHEET_SCRIPT, (title.to_string(), columns, rows))?;
    if let Some(err) = filename.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
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
    let written = run_script(
        cfg,
        RESEARCH_SCRIPT,
        (topic.to_string(), filename.clone(), urls),
    )?;
    if let Some(err) = written.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
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
    let entry = run_script(cfg, UPDATE_MEMORY_SCRIPT, (note.to_string(),))?;
    if let Some(err) = entry.strip_prefix("error:") {
        return Err(err.trim().to_string());
    }
    Ok(format!("memory updated: {entry}"))
}
