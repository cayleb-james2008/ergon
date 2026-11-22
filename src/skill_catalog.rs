//! Skill catalog metadata and sandboxed Rune VM validation / dry-run harness.
//!
//! Exposes rich, serializable metadata for all 12 built-in Rune skills
//! (`GET /api/skills`), compile-time validation across the full library
//! (`validate_all_skills`), and isolated ephemeral sandbox dry-runs
//! (`dry_run_skill` / `handle_skill_test_request` for `POST /api/skills/test`).

use crate::{sandbox_path, skills, truncate, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static DRY_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct SkillInputField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
    pub example: Value,
}

impl SkillInputField {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "type": self.field_type,
            "required": self.required,
            "description": self.description,
            "example": self.example,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillMetadata {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub deterministic: bool,
    pub outputs: Vec<String>,
    pub input_schema: Vec<SkillInputField>,
    pub sample_prompts: Vec<String>,
    pub source: String,
    pub source_lines: usize,
}

impl SkillMetadata {
    pub fn param_names(&self) -> Vec<String> {
        self.input_schema.iter().map(|f| f.name.clone()).collect()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "slug": self.slug,
            "title": self.title,
            "name": self.title,
            "category": self.category,
            "description": self.description,
            "deterministic": self.deterministic,
            "builtin": true,
            "outputs": self.outputs,
            "params": self.param_names(),
            "input_schema": self.input_schema.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "sample_prompts": self.sample_prompts,
            "source": self.source,
            "source_lines": self.source_lines,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillValidationEntry {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub compiled: bool,
    pub source_lines: usize,
    pub error: Option<String>,
}

impl SkillValidationEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "slug": self.slug,
            "title": self.title,
            "compiled": self.compiled,
            "source_lines": self.source_lines,
            "error": self.error,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillValidationReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub all_ok: bool,
    pub duration_ms: u64,
    pub results: Vec<SkillValidationEntry>,
}

impl SkillValidationReport {
    pub fn to_json(&self) -> Value {
        json!({
            "total": self.total,
            "passed": self.passed,
            "failed": self.failed,
            "all_ok": self.all_ok,
            "duration_ms": self.duration_ms,
            "results": self.results.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DryRunFile {
    pub path: String,
    pub bytes: u64,
    pub preview: String,
}

impl DryRunFile {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "bytes": self.bytes,
            "preview": self.preview,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillDryRunResult {
    pub skill_id: String,
    pub ok: bool,
    pub compiled: bool,
    pub output: String,
    pub error: Option<String>,
    pub files_written: Vec<DryRunFile>,
    pub memory_after: Option<String>,
    pub duration_ms: u64,
}

impl SkillDryRunResult {
    pub fn to_json(&self) -> Value {
        json!({
            "skill_id": self.skill_id,
            "ok": self.ok,
            "compiled": self.compiled,
            "output": self.output,
            "error": self.error,
            "files_written": self.files_written.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "memory_after": self.memory_after,
            "duration_ms": self.duration_ms,
        })
    }
}

fn field(
    name: &str,
    field_type: &str,
    required: bool,
    description: &str,
    example: Value,
) -> SkillInputField {
    SkillInputField {
        name: name.to_string(),
        field_type: field_type.to_string(),
        required,
        description: description.to_string(),
        example,
    }
}

#[allow(clippy::too_many_arguments)]
fn meta(
    id: &str,
    slug: &str,
    title: &str,
    category: &str,
    description: &str,
    deterministic: bool,
    outputs: &[&str],
    input_schema: Vec<SkillInputField>,
    sample_prompts: &[&str],
    source: &str,
) -> SkillMetadata {
    SkillMetadata {
        id: id.to_string(),
        slug: slug.to_string(),
        title: title.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        deterministic,
        outputs: outputs.iter().map(|s| s.to_string()).collect(),
        input_schema,
        sample_prompts: sample_prompts.iter().map(|s| s.to_string()).collect(),
        source: source.to_string(),
        source_lines: source.lines().count(),
    }
}

/// Complete catalog of all 12 built-in Rune skills shipped with Ergon.
pub fn all_skills() -> Vec<SkillMetadata> {
    vec![
        meta(
            "write_document",
            "write-document",
            "Write Document",
            "documents",
            "Formats a title and structured section blocks into a finished Markdown document in the workspace.",
            true,
            &["markdown"],
            vec![
                field("title", "string", true, "Document title (also becomes `<title>.md`).", json!("Weekly Status")),
                field("sections", "string_list", true, "Ordered Markdown section strings.", json!(["## Highlights\nShipped v0.5.0 milestones."])),
            ],
            &[
                "Write a weekly status report covering shipped items and next week's priorities.",
                "Compile our onboarding checklist into a clean workspace document.",
            ],
            skills::WRITE_DOCUMENT_SCRIPT,
        ),
        meta(
            "make_spreadsheet",
            "make-spreadsheet",
            "Make Spreadsheet",
            "data",
            "Formats column headers and row cells with RFC-4180 CSV quoting into `<title>.csv` in the workspace.",
            true,
            &["csv"],
            vec![
                field("title", "string", true, "Spreadsheet title (writes `<title>.csv`).", json!("Team Budget")),
                field("columns", "string_list", true, "Ordered header column names.", json!(["Item", "Category", "Cost"])),
                field("rows", "string_rows", true, "2D array of string cell values.", json!([["Cloud Hosting", "Infra", "420.00"]])),
            ],
            &[
                "Turn my Q4 software expenses into a clean CSV spreadsheet with Vendor, Category, and Cost columns.",
                "Build a structured inventory CSV from the items listed in my notes.",
            ],
            skills::MAKE_SPREADSHEET_SCRIPT,
        ),
        meta(
            "research",
            "research",
            "Web Research",
            "research",
            "Fetches one or more web pages, records fetch status honestly per source, and compiles sourced notes into `Research - <topic>.md`.",
            false,
            &["markdown"],
            vec![
                field("topic", "string", true, "Research subject used in the heading and filename.", json!("Cloud Pricing")),
                field("urls", "string_list", true, "List of HTTP(S) URLs to fetch and compile.", json!(["https://example.com"])),
            ],
            &[
                "Research vendor compliance documentation across these three URLs and compile sourced notes.",
                "Fetch these release-note pages and save a single cited research summary.",
            ],
            skills::RESEARCH_SCRIPT,
        ),
        meta(
            "update_memory",
            "update-memory",
            "Update Memory",
            "memory",
            "Records a durable single-line preference or learned fact into the employee's standing `memory.md`.",
            true,
            &["memory"],
            vec![
                field("note", "string", true, "Single-line preference or fact to append to standing memory.", json!("Always include currency totals rounded to two decimals.")),
            ],
            &[
                "Remember that I prefer concise bullet summaries before detailed tables.",
                "Record in standing memory that all finance exports use USD with two decimal places.",
            ],
            skills::UPDATE_MEMORY_SCRIPT,
        ),
        meta(
            "reconcile_ledger",
            "reconcile-ledger",
            "Reconcile Ledger",
            "finance",
            "Reconciles one or more workspace CSV invoice/expense ledgers with penny-exact arithmetic, flagging duplicate references, missing receipts, and cross-file mismatches alongside vendor and category rollups.",
            true,
            &["markdown", "csv"],
            vec![
                field("source_files", "file_path_list", true, "One or more workspace-relative CSV ledger paths.", json!(["invoices.csv", "expenses.csv"])),
                field("output_title", "string", false, "Output title prefix for the Markdown report and summary CSV.", json!("Ledger Reconciliation")),
            ],
            &[
                "Reconcile invoices.csv and expenses.csv, flag duplicate invoice numbers and missing receipts, and total spend by vendor and category.",
                "Audit Q4-expenses.csv for missing receipts, duplicate references, and anomalous amounts.",
            ],
            skills::RECONCILE_LEDGER_SCRIPT,
        ),
        meta(
            "summarize_meeting",
            "summarize-meeting",
            "Summarize Meeting",
            "operations",
            "Extracts participants, line-cited decisions, action items with owners and due dates, open questions/risks, and ready-to-send team and per-owner follow-up drafts from meeting notes.",
            true,
            &["markdown"],
            vec![
                field("notes_file", "file_path", true, "Workspace-relative path to the meeting transcript or notes file.", json!("standup-notes.md")),
                field("output_title", "string", false, "Title for the generated Markdown summary.", json!("Meeting Summary")),
            ],
            &[
                "Summarize standup-notes.md into decisions, action items with owners and due dates, and follow-up drafts.",
                "Extract every commitment, owner, deadline, and open blocker from q4-planning-transcript.txt.",
            ],
            skills::SUMMARIZE_MEETING_SCRIPT,
        ),
        meta(
            "audit_contract",
            "audit-contract",
            "Audit Contract",
            "legal",
            "Checks contracts and vendor agreements for required clauses, extracts renewal/term/notice dates, flags missing provisions, and emits a line-cited `[L##]` risk table with redline guidance.",
            true,
            &["markdown"],
            vec![
                field("contract_file", "file_path", true, "Workspace-relative path to the contract text or Markdown file.", json!("vendor-msa.md")),
                field("required_clauses", "string_list", false, "Optional checklist of required clauses (uses standard 8-clause corporate checklist if omitted).", json!(["termination", "liability", "confidentiality", "data protection", "renewal"])),
                field("output_title", "string", false, "Title for the generated contract audit report.", json!("Contract Audit")),
            ],
            &[
                "Audit vendor-msa.md for auto-renewal traps, unlimited liability, missing confidentiality/data-protection clauses, and produce a line-cited risk table.",
                "Review supplier-agreement.txt against termination, indemnification, payment, and governing law requirements.",
            ],
            skills::AUDIT_CONTRACT_SCRIPT,
        ),
        meta(
            "triage_inbox",
            "triage-inbox",
            "Triage Inbox",
            "operations",
            "Classifies messages, emails, or support tickets (Markdown, plain text, CSV, or JSON) into Urgent, Action, and FYI buckets with priority scores, rationale, and concrete next steps.",
            true,
            &["markdown"],
            vec![
                field("inbox_file", "file_path", true, "Workspace-relative path to inbox export (.md, .txt, .csv, or .json).", json!("inbox.md")),
                field("output_title", "string", false, "Title for the generated triage report.", json!("Inbox Triage")),
            ],
            &[
                "Triage inbox.md into Urgent, Action, and FYI queues with concrete next steps for every item.",
                "Prioritize support-tickets.csv so critical outages and overdue billing alerts float to the top.",
            ],
            skills::TRIAGE_INBOX_SCRIPT,
        ),
        meta(
            "clean_dataset",
            "clean-dataset",
            "Clean Dataset",
            "data",
            "Normalizes CSV or JSON schemas to lowercase snake_case, repairs ragged rows, standardizes null tokens, deduplicates rows, flags statistical outliers (|z| >= 2.5), and writes a clean CSV plus diff metrics report.",
            true,
            &["csv", "markdown"],
            vec![
                field("input_file", "file_path", true, "Workspace-relative path to the messy .csv or .json dataset.", json!("raw-orders.csv")),
                field("key_column", "string", false, "Optional deduplication key column name (empty deduplicates by full normalized row).", json!("order_id")),
                field("output_title", "string", false, "Title prefix for the cleaned CSV and diff metrics Markdown report.", json!("Cleaned Dataset")),
            ],
            &[
                "Clean raw-orders.csv: normalize headers, deduplicate on order_id, standardize N/A values, flag numeric outliers, and produce a diff report.",
                "Normalize crm-export.json into a deduplicated snake_case CSV with a column-level null and outlier audit.",
            ],
            skills::CLEAN_DATASET_SCRIPT,
        ),
        meta(
            "compare_documents",
            "compare-documents",
            "Compare Documents",
            "documents",
            "Performs linear-time line-by-line diffing between two workspace files, reporting added/removed/unchanged counts, similarity %, heading changes, and unified `[L##]`/`[R##]` context hunks.",
            true,
            &["markdown"],
            vec![
                field("left_file", "file_path", true, "Baseline (before) workspace file path.", json!("policy-v1.md")),
                field("right_file", "file_path", true, "Revised (after) workspace file path.", json!("policy-v2.md")),
                field("output_title", "string", false, "Title for the generated comparison report.", json!("Document Comparison")),
            ],
            &[
                "Compare policy-v1.md against policy-v2.md and summarize every added, removed, or modified clause.",
                "Diff contract-draft-1.txt and contract-draft-2.txt with line-numbered hunks.",
            ],
            skills::COMPARE_DOCUMENTS_SCRIPT,
        ),
        meta(
            "draft_brief",
            "draft-brief",
            "Draft Brief",
            "documents",
            "Drafts an executive memo or SOP grounded strictly in named workspace files and standing memory (`memory.md`), citing `[file:L##]` for every finding and halting if zero readable sources exist.",
            true,
            &["markdown"],
            vec![
                field("topic", "string", true, "Subject of the executive memo or SOP.", json!("Q4 Spend Controls")),
                field("source_files", "file_path_list", true, "Workspace files to ground the brief in.", json!(["reconciliation.md", "meeting-notes.md"])),
                field("audience", "string", false, "Intended readers (e.g. Executive leadership, Finance ops).", json!("Executive leadership")),
                field("output_title", "string", false, "Optional output document title (defaults to `Brief - <topic>`).", json!("Q4 Spend Controls Brief")),
            ],
            &[
                "Draft an executive brief on Q4 Spend Controls grounded in reconciliation.md and meeting-notes.md using my standing preferences.",
                "Turn ops-notes.md and audit.md into a cited SOP brief for the operations team.",
            ],
            skills::DRAFT_BRIEF_SCRIPT,
        ),
        meta(
            "extract_tables",
            "extract-tables",
            "Extract Tables",
            "data",
            "Parses unstructured Markdown pipe tables, key=value structured logs, TSV blocks, or Key: Value records from a workspace file into clean RFC-4180 CSV spreadsheets.",
            true,
            &["csv"],
            vec![
                field("source_file", "file_path", true, "Workspace-relative path to Markdown, text, or log file.", json!("service.log")),
                field("output_title", "string", false, "Title prefix for extracted CSV spreadsheet(s).", json!("Extracted Table")),
            ],
            &[
                "Extract the structured key=value records from service.log into a clean CSV spreadsheet.",
                "Pull all Markdown tables out of quarterly-review.md into separate CSV files.",
            ],
            skills::EXTRACT_TABLES_SCRIPT,
        ),
    ]
}

/// Look up one built-in skill by either snake_case `id` or kebab-case `slug`.
pub fn get_skill(id_or_slug: &str) -> Option<SkillMetadata> {
    let q = id_or_slug.trim().to_ascii_lowercase();
    let q_snake = q.replace('-', "_");
    all_skills()
        .into_iter()
        .find(|s| s.id == q_snake || s.slug == q)
}

/// Ready-to-serve JSON payload for `GET /api/skills`.
pub fn catalog_json() -> Value {
    let list = all_skills();
    let mut categories: Vec<String> = Vec::new();
    for s in &list {
        if !categories.contains(&s.category) {
            categories.push(s.category.clone());
        }
    }
    json!({
        "count": list.len(),
        "categories": categories,
        "skills": list.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
    })
}

/// Compile-check all 12 built-in Rune skills against the real host Rune VM
/// context and verify metadata integrity (unique ids/slugs, non-empty prompts).
pub fn validate_all_skills(cfg: &Config) -> SkillValidationReport {
    let started = Instant::now();
    let list = all_skills();
    let mut results = Vec::with_capacity(list.len());
    let mut seen_ids: Vec<String> = Vec::new();
    let mut seen_slugs: Vec<String> = Vec::new();
    let mut passed = 0usize;

    for s in &list {
        let mut err: Option<String> = None;
        if seen_ids.contains(&s.id) {
            err = Some(format!("duplicate skill id: {}", s.id));
        } else if seen_slugs.contains(&s.slug) {
            err = Some(format!("duplicate skill slug: {}", s.slug));
        } else if s.sample_prompts.is_empty() || s.input_schema.is_empty() {
            err = Some("missing sample_prompts or input_schema".to_string());
        } else if let Err(compile_err) = skills::check_script(cfg, &s.source) {
            err = Some(compile_err);
        }

        seen_ids.push(s.id.clone());
        seen_slugs.push(s.slug.clone());

        let compiled = err.is_none();
        if compiled {
            passed += 1;
        }
        results.push(SkillValidationEntry {
            id: s.id.clone(),
            slug: s.slug.clone(),
            title: s.title.clone(),
            compiled,
            source_lines: s.source_lines,
            error: err,
        });
    }

    let total = results.len();
    let failed = total - passed;
    SkillValidationReport {
        total,
        passed,
        failed,
        all_ok: failed == 0,
        duration_ms: started.elapsed().as_millis() as u64,
        results,
    }
}

struct EphemeralSandbox {
    root: PathBuf,
    cfg: Config,
}

impl EphemeralSandbox {
    fn new(tag: &str) -> Result<Self, String> {
        let seq = DRY_RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let clean_tag: String = tag
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(24)
            .collect();
        let root = std::env::temp_dir().join(format!("ergon-skill-dryrun-{pid}-{seq}-{clean_tag}"));
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "dry-run".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs()
            .map_err(|e| format!("create dry-run sandbox: {e}"))?;
        Ok(EphemeralSandbox { root, cfg })
    }

    fn seed_file(&self, rel: &str, content: &str) -> Result<(), String> {
        let full = sandbox_path(&self.cfg.workspace_dir, rel)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("seed mkdir {rel}: {e}"))?;
        }
        std::fs::write(&full, content).map_err(|e| format!("seed write {rel}: {e}"))
    }

    fn seed_from_value(&self, seed_files: &Value) -> Result<Vec<String>, String> {
        let mut seeded = Vec::new();
        match seed_files {
            Value::Object(map) => {
                for (path, val) in map {
                    let content = val
                        .as_str()
                        .ok_or_else(|| format!("seed file '{path}' content must be a string"))?;
                    self.seed_file(path, content)?;
                    seeded.push(path.clone());
                }
            }
            Value::Array(items) => {
                for item in items {
                    let path = item["path"]
                        .as_str()
                        .ok_or("seed file array items require a string 'path'")?;
                    let content = item["content"]
                        .as_str()
                        .ok_or("seed file array items require a string 'content'")?;
                    self.seed_file(path, content)?;
                    seeded.push(path.to_string());
                }
            }
            Value::Null => {}
            _ => return Err("seed files must be a JSON object or array of {path, content}".to_string()),
        }
        Ok(seeded)
    }

    fn collect_written_files(&self, pre_existing: &[String]) -> Vec<DryRunFile> {
        fn walk(root: &Path, dir: &Path, pre: &[String], out: &mut Vec<DryRunFile>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    walk(root, &p, pre, out);
                    continue;
                }
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                if pre.iter().any(|s| s == &rel) {
                    continue;
                }
                let bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                let preview = std::fs::read_to_string(&p)
                    .map(|s| truncate(&s, 2_000))
                    .unwrap_or_default();
                out.push(DryRunFile {
                    path: rel,
                    bytes,
                    preview,
                });
            }
        }
        let mut out = Vec::new();
        walk(&self.cfg.workspace_dir, &self.cfg.workspace_dir, pre_existing, &mut out);
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }
}

impl Drop for EphemeralSandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Spawn a tiny bounded one-shot local HTTP server used only when running
/// the built-in dry-run smoke test for `research` with default args so it
/// executes offline with zero external network dependencies.
fn spawn_local_sample_http(body: &'static str) -> Option<(String, std::thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    let _ = listener.set_nonblocking(true);
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
    Some((format!("http://127.0.0.1:{port}/sample"), handle))
}

/// Built-in realistic dry-run fixture (`args`, `seed_files`, `seed_memory`)
/// used when `dry_run_skill` is invoked with empty/null args so `POST /api/skills/test`
/// can smoke-test any of the 12 skills with one click.
fn default_fixture_for(
    canonical_id: &str,
    local_research_url: Option<&str>,
) -> (Value, Value, Option<&'static str>) {
    match canonical_id {
        "write_document" => (
            json!({
                "title": "Sample Ops Update",
                "sections": ["## Highlights\nCompleted weekly close with zero blockers.", "## Next Steps\nPublish Q4 budget summary."]
            }),
            json!({}),
            None,
        ),
        "make_spreadsheet" => (
            json!({
                "title": "Sample Budget",
                "columns": ["Item", "Department", "Amount"],
                "rows": [
                    ["Design Licenses", "Product", "240.00"],
                    ["Cloud Compute, US-East", "Engineering", "1180.50"]
                ]
            }),
            json!({}),
            None,
        ),
        "research" => {
            let url = local_research_url.unwrap_or("http://127.0.0.1:9/offline");
            (
                json!({
                    "topic": "Sample Topic",
                    "urls": [url]
                }),
                json!({}),
                None,
            )
        }
        "update_memory" => (
            json!({ "note": "Prefer concise executive bullet lists ahead of raw tables." }),
            json!({}),
            None,
        ),
        "reconcile_ledger" => (
            json!({
                "source_files": ["invoices.csv"],
                "output_title": "Sample Ledger Reconciliation"
            }),
            json!({
                "invoices.csv": "date,ref,vendor,category,amount,receipt\n2026-11-01,INV-101,Northwind Supply,Operations,420.00,verified\n2026-11-02,INV-102,Boreal Optics,Equipment,985.50,missing\n2026-11-03,INV-101,Northwind Supply,Operations,420.00,verified\n"
            }),
            None,
        ),
        "summarize_meeting" => (
            json!({
                "notes_file": "meeting.md",
                "output_title": "Sample Meeting Summary"
            }),
            json!({
                "meeting.md": "Attendees: Alice, Bob, Priya\nAlice: We decided to freeze schema changes on Thursday.\nAction: Bob will publish the migration guide by Friday.\nPriya: Owner: Priya — audit staging receipts due 2026-11-28.\nBob: Is the backup retention window confirmed?\n"
            }),
            None,
        ),
        "audit_contract" => (
            json!({
                "contract_file": "vendor-msa.md",
                "output_title": "Sample Contract Audit"
            }),
            json!({
                "vendor-msa.md": "# Master Services Agreement\nEffective Date: 2026-12-01. Initial Term: 12 months.\nThis agreement shall auto-renew for successive 12-month periods unless 90 days prior written notice is given.\nProvider may modify terms at its sole discretion.\nPayment terms are Net 60 from invoice receipt.\nGoverning Law: State of Delaware.\nTermination: Either party may terminate for material breach.\n"
            }),
            None,
        ),
        "triage_inbox" => (
            json!({
                "inbox_file": "inbox.md",
                "output_title": "Sample Inbox Triage"
            }),
            json!({
                "inbox.md": "From: alerts@infra.local\nSubject: SEV1 Production outage on billing API\nImmediate escalation required.\n---\nFrom: finance@partner.local\nSubject: Please approve Q4 invoice #442\nCan you review and sign off today?\n---\nFrom: newsletter@digest.local\nSubject: Weekly product digest\nFYI: automated release notes.\n"
            }),
            None,
        ),
        "clean_dataset" => (
            json!({
                "input_file": "orders.csv",
                "key_column": "Order ID",
                "output_title": "Sample Cleaned Orders"
            }),
            json!({
                "orders.csv": "Order ID,Customer Name,Order Amount,Notes\nORD-1,Alice,100.00,ok\nORD-2,Bob,105.00,N/A\nORD-3,Cara,98.00,ok\nORD-4,Dan,102.00,none\nORD-5,Eli,101.00,ok\nORD-6,Fay,99.00,ok\nORD-7,Gus,9850.00,outlier\nORD-2,Bob,105.00,duplicate\n"
            }),
            None,
        ),
        "compare_documents" => (
            json!({
                "left_file": "policy-v1.md",
                "right_file": "policy-v2.md",
                "output_title": "Sample Policy Diff"
            }),
            json!({
                "policy-v1.md": "# Expense Policy\nReceipts required above $50.\nApproval SLA: 5 business days.\n",
                "policy-v2.md": "# Expense Policy\nReceipts required above $25.\nApproval SLA: 2 business days.\n## International Travel\nPre-approval required.\n"
            }),
            None,
        ),
        "draft_brief" => (
            json!({
                "topic": "Monthly Close Controls",
                "source_files": ["close-notes.md"],
                "audience": "Finance leadership",
                "output_title": "Sample Monthly Close Brief"
            }),
            json!({
                "close-notes.md": "# Close Notes\n- Total spend reconciled: $14,250.00 across 3 cost centers.\n- Risk: 2 invoices missing receipts.\n- Must verify duplicate references before ledger lock.\n"
            }),
            Some("# Memory\n\n- Always open executive briefs with verified dollar totals.\n"),
        ),
        "extract_tables" => (
            json!({
                "source_file": "metrics.md",
                "output_title": "Sample Extracted Metrics"
            }),
            json!({
                "metrics.md": "# Service Metrics\n\n| Service | Latency_ms | Status |\n|---|---:|---|\n| auth | 18 | ok |\n| billing | 42 | ok |\n"
            }),
            None,
        ),
        _ => (json!({}), json!({}), None),
    }
}

/// Run a built-in Rune skill in a fresh ephemeral sandbox workspace.
/// When `args` is null or an empty object and `seed_files` is `None`,
/// uses realistic built-in sample inputs for that skill so the caller
/// can verify compile + execution end-to-end with zero manual setup.
pub fn dry_run_skill(
    skill_id: &str,
    args: &Value,
    seed_files: Option<&Value>,
    seed_memory: Option<&str>,
) -> SkillDryRunResult {
    let started = Instant::now();
    let Some(meta) = get_skill(skill_id) else {
        return SkillDryRunResult {
            skill_id: skill_id.to_string(),
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(format!("unknown built-in skill: {skill_id}")),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    };

    let sandbox = match EphemeralSandbox::new(&meta.id) {
        Ok(s) => s,
        Err(e) => {
            return SkillDryRunResult {
                skill_id: meta.id,
                ok: false,
                compiled: false,
                output: String::new(),
                error: Some(e),
                files_written: Vec::new(),
                memory_after: None,
                duration_ms: started.elapsed().as_millis() as u64,
            };
        }
    };

    if let Err(e) = skills::check_script(&sandbox.cfg, &meta.source) {
        return SkillDryRunResult {
            skill_id: meta.id,
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(e),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    }

    let args_is_empty = args.is_null() || args.as_object().map(|m| m.is_empty()).unwrap_or(false);
    let mut local_http_handle: Option<std::thread::JoinHandle<()>> = None;

    let (effective_args, effective_seeds, effective_mem) = if args_is_empty && seed_files.is_none()
    {
        let local_url = if meta.id == "research" {
            if let Some((url, handle)) = spawn_local_sample_http(
                "<html><body><h1>Sample Source</h1><p>Verified offline research snippet.</p></body></html>",
            ) {
                local_http_handle = Some(handle);
                Some(url)
            } else {
                None
            }
        } else {
            None
        };
        let (def_args, def_files, def_mem) = default_fixture_for(&meta.id, local_url.as_deref());
        (def_args, Some(def_files), seed_memory.or(def_mem))
    } else {
        (args.clone(), seed_files.cloned(), seed_memory)
    };

    if let Some(mem) = effective_mem {
        let _ = std::fs::write(&sandbox.cfg.memory_path, mem);
    }

    let pre_seeded = match effective_seeds.as_ref() {
        Some(v) => match sandbox.seed_from_value(v) {
            Ok(paths) => paths,
            Err(e) => {
                if let Some(h) = local_http_handle {
                    let _ = h.join();
                }
                return SkillDryRunResult {
                    skill_id: meta.id,
                    ok: false,
                    compiled: true,
                    output: String::new(),
                    error: Some(e),
                    files_written: Vec::new(),
                    memory_after: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        },
        None => Vec::new(),
    };

    let exec = skills::dispatch_builtin_skill(&sandbox.cfg, &meta.id, &effective_args)
        .unwrap_or_else(|| Err(format!("skill {} is not dispatchable", meta.id)));

    if let Some(h) = local_http_handle {
        let _ = h.join();
    }

    let files_written = sandbox.collect_written_files(&pre_seeded);
    let memory_after = if meta.id == "update_memory" {
        Some(sandbox.cfg.read_memory())
    } else {
        None
    };

    match exec {
        Ok(output) => SkillDryRunResult {
            skill_id: meta.id,
            ok: true,
            compiled: true,
            output,
            error: None,
            files_written,
            memory_after,
            duration_ms: started.elapsed().as_millis() as u64,
        },
        Err(err) => SkillDryRunResult {
            skill_id: meta.id,
            ok: false,
            compiled: true,
            output: String::new(),
            error: Some(err),
            files_written,
            memory_after,
            duration_ms: started.elapsed().as_millis() as u64,
        },
    }
}

/// Compile and execute an arbitrary Rune library script in a fresh ephemeral
/// sandbox workspace (strict 5M instruction budget & 128 MB allocation cap).
pub fn dry_run_custom_source(
    source: &str,
    params: &[String],
    seed_files: Option<&Value>,
    seed_memory: Option<&str>,
) -> SkillDryRunResult {
    let started = Instant::now();
    let sandbox = match EphemeralSandbox::new("custom") {
        Ok(s) => s,
        Err(e) => {
            return SkillDryRunResult {
                skill_id: "custom".to_string(),
                ok: false,
                compiled: false,
                output: String::new(),
                error: Some(e),
                files_written: Vec::new(),
                memory_after: None,
                duration_ms: started.elapsed().as_millis() as u64,
            };
        }
    };

    if let Err(e) = skills::check_script(&sandbox.cfg, source) {
        return SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(e),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    }

    if let Some(mem) = seed_memory {
        let _ = std::fs::write(&sandbox.cfg.memory_path, mem);
    }
    let pre_seeded = match seed_files {
        Some(v) => match sandbox.seed_from_value(v) {
            Ok(paths) => paths,
            Err(e) => {
                return SkillDryRunResult {
                    skill_id: "custom".to_string(),
                    ok: false,
                    compiled: true,
                    output: String::new(),
                    error: Some(e),
                    files_written: Vec::new(),
                    memory_after: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        },
        None => Vec::new(),
    };

    match skills::run_library_skill(&sandbox.cfg, source, params) {
        Ok(output) => SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: true,
            compiled: true,
            output,
            error: None,
            files_written: sandbox.collect_written_files(&pre_seeded),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        },
        Err(err) => SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: false,
            compiled: true,
            output: String::new(),
            error: Some(err),
            files_written: sandbox.collect_written_files(&pre_seeded),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        },
    }
}

/// High-level JSON handler for `POST /api/skills/test`.
///
/// Supports either:
/// - `{ "skill_id": "reconcile_ledger", "args": {...}, "files": {...}, "memory": "..." }`
/// - `{ "source": "pub fn main(x) { ... }", "params": ["..."], "files": {...} }`
/// - `{ "validate_all": true }` (runs compile check on all 12 built-in skills)
pub fn handle_skill_test_request(body: &Value) -> Value {
    if body.get("validate_all").and_then(|v| v.as_bool()).unwrap_or(false) {
        let sandbox = match EphemeralSandbox::new("validate-all") {
            Ok(s) => s,
            Err(e) => return json!({ "ok": false, "error": e }),
        };
        return validate_all_skills(&sandbox.cfg).to_json();
    }

    let seed_files = body.get("files").or_else(|| body.get("seed_files"));
    let seed_memory = body.get("memory").and_then(|v| v.as_str());

    if let Some(source) = body.get("source").and_then(|v| v.as_str()) {
        if !source.trim().is_empty() {
            let params: Vec<String> = body
                .get("params")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|i| i.as_str().map(|s| s.to_string()).unwrap_or_else(|| i.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            return dry_run_custom_source(source, &params, seed_files, seed_memory).to_json();
        }
    }

    let skill_id = body
        .get("skill_id")
        .or_else(|| body.get("id"))
        .or_else(|| body.get("slug"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if skill_id.trim().is_empty() {
        return json!({
            "ok": false,
            "compiled": false,
            "error": "request requires 'skill_id', 'source', or 'validate_all': true"
        });
    }

    let args = body.get("args").unwrap_or(&Value::Null);
    dry_run_skill(skill_id, args, seed_files, seed_memory).to_json()
}
