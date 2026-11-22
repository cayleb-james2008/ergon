// Canonical catalog of all 12 Ergon v0.5.0 Rune skills with progressive merge over GET /api/skills.
// Keeps full metadata, categories, input schemas, sample prompts, and Rune previews available even
// before a server advertises /api/skills, while seamlessly upgrading to live server catalog payloads.

export type SkillCategory =
  | "Finance"
  | "Documents"
  | "Data"
  | "Compliance"
  | "Operations"
  | "Research"
  | "Memory";

export type SkillTone = "ops" | "mint" | "plan" | "research";

export type SkillMeta = {
  id: string;
  title: string;
  category: SkillCategory;
  tone: SkillTone;
  filename: string;
  description: string;
  inputs: string[];
  sample_prompt: string;
  sample_prompts: string[];
  source: string;
  builtin: boolean;
};

export const CANONICAL_SKILLS: SkillMeta[] = [
  {
    id: "reconcile_ledger",
    title: "Reconcile ledger",
    category: "Finance",
    tone: "mint",
    filename: "reconcile_ledger.rn",
    description:
      "Matches multi-CSV invoices and expense logs, computes vendor and category subtotals, and flags duplicate references or missing receipts.",
    inputs: ["csv_files", "threshold?"],
    sample_prompt: "Reconcile expenses-q3.csv, compute vendor and category totals, and flag missing receipts or duplicates.",
    sample_prompts: [
      "Reconcile expenses-q3.csv, compute vendor and category totals, and flag missing receipts or duplicates.",
      "Cross-check all workspace expense CSVs and write a reconciled ledger plus audit summary.",
    ],
    source: `// reconcile_ledger.rn — deterministic multi-CSV expense & invoice audit
pub fn run(args) {
  let rows = ergon::parse_csv(ergon::read_file(args.path)?);
  let audit = ergon::reconcile_rows(rows);
  ergon::write_csv(args.out_csv, audit.normalized_rows)?;
  ergon::write_markdown(args.out_md, audit.summary_sections)?;
  Ok(audit.receipt)
}`,
    builtin: true,
  },
  {
    id: "summarize_meeting",
    title: "Summarize meeting",
    category: "Documents",
    tone: "ops",
    filename: "summarize_meeting.rn",
    description:
      "Distills meeting transcripts and raw notes into decisions made, owners, due dates, open blockers, and a ready-to-send follow-up draft.",
    inputs: ["notes_file", "audience?"],
    sample_prompt: "Summarize tuesday-sync-notes.md into decisions, action owners, due dates, and a follow-up draft.",
    sample_prompts: [
      "Summarize tuesday-sync-notes.md into decisions, action owners, due dates, and a follow-up draft.",
      "Extract all unchecked action items and owners from my latest meeting notes.",
    ],
    source: `// summarize_meeting.rn — structured decisions, owners, due dates & follow-up
pub fn run(args) {
  let notes = ergon::read_file(args.notes_file)?;
  let digest = ergon::extract_meeting_digest(notes);
  ergon::write_document(args.out_file, digest.sections)?;
  Ok(digest.stats)
}`,
    builtin: true,
  },
  {
    id: "audit_contract",
    title: "Audit contract",
    category: "Compliance",
    tone: "plan",
    filename: "audit_contract.rn",
    description:
      "Audits contracts and vendor agreements for renewal dates, liability caps, indemnity, termination notice, and missing clauses with line citations.",
    inputs: ["contract_file", "required_clauses?"],
    sample_prompt: "Audit the vendor agreement in my workspace for renewal windows, liability terms, and missing clauses with line citations.",
    sample_prompts: [
      "Audit the vendor agreement in my workspace for renewal windows, liability terms, and missing clauses with line citations.",
      "Check workspace agreements for auto-renewal traps and output a line-cited risk table.",
    ],
    source: `// audit_contract.rn — clause presence check, renewal dates & line-cited risk table
pub fn run(args) {
  let text = ergon::read_file(args.contract_file)?;
  let findings = ergon::scan_contract_clauses(text, args.required_clauses);
  ergon::write_document(args.out_file, findings.report_sections)?;
  Ok(findings.risk_summary)
}`,
    builtin: true,
  },
  {
    id: "triage_inbox",
    title: "Triage inbox",
    category: "Operations",
    tone: "ops",
    filename: "triage_inbox.rn",
    description:
      "Sorts incoming messages, tickets, or ops requests into Urgent, Action, and FYI queues with concrete next steps and owner suggestions.",
    inputs: ["source_file"],
    sample_prompt: "Triage workspace tickets and messages into Urgent, Action, and FYI queues with concrete next steps.",
    sample_prompts: [
      "Triage workspace tickets and messages into Urgent, Action, and FYI queues with concrete next steps.",
      "Prioritize open support notes and draft a morning triage queue.",
    ],
    source: `// triage_inbox.rn — Urgent / Action / FYI priority routing
pub fn run(args) {
  let items = ergon::read_file(args.source_file)?;
  let queue = ergon::classify_triage(items);
  ergon::write_document(args.out_file, queue.sections)?;
  Ok(queue.counts)
}`,
    builtin: true,
  },
  {
    id: "clean_dataset",
    title: "Clean dataset",
    category: "Data",
    tone: "mint",
    filename: "clean_dataset.rn",
    description:
      "Normalizes CSV/JSON column headers and value formats, removes duplicate records, flags nulls and numeric outliers, and reports exact diff metrics.",
    inputs: ["dataset_file"],
    sample_prompt: "Clean and deduplicate expenses-q3.csv, flag nulls or numeric outliers, and save a diff metrics report.",
    sample_prompts: [
      "Clean and deduplicate expenses-q3.csv, flag nulls or numeric outliers, and save a diff metrics report.",
      "Normalize whitespace and schema headers across my uploaded CSV dataset.",
    ],
    source: `// clean_dataset.rn — schema normalization, deduplication & outlier metrics
pub fn run(args) {
  let raw = ergon::read_file(args.dataset_file)?;
  let cleaned = ergon::normalize_dataset(raw);
  ergon::write_file(args.out_file, cleaned.body)?;
  ergon::write_document(args.report_file, cleaned.metrics_md)?;
  Ok(cleaned.metrics)
}`,
    builtin: true,
  },
  {
    id: "compare_documents",
    title: "Compare documents",
    category: "Compliance",
    tone: "plan",
    filename: "compare_documents.rn",
    description:
      "Computes a deterministic line-by-line diff between two workspace files and writes a clear change summary highlighting additions and removals.",
    inputs: ["left_file", "right_file"],
    sample_prompt: "Compare two documents in my workspace line by line and write a structured change summary.",
    sample_prompts: [
      "Compare two documents in my workspace line by line and write a structured change summary.",
      "Diff old-vendor-draft.txt against tuesday-sync-notes.md and summarize every change.",
    ],
    source: `// compare_documents.rn — deterministic line-by-line diff & change summary
pub fn run(args) {
  let a = ergon::read_file(args.left_file)?;
  let b = ergon::read_file(args.right_file)?;
  let diff = ergon::diff_lines(a, b);
  ergon::write_document(args.out_file, diff.sections)?;
  Ok(diff.stats)
}`,
    builtin: true,
  },
  {
    id: "draft_brief",
    title: "Draft brief",
    category: "Documents",
    tone: "research",
    filename: "draft_brief.rn",
    description:
      "Authors an executive memo or SOP grounded strictly in your workspace files and standing memory rules, citing every source file used.",
    inputs: ["topic", "sources?"],
    sample_prompt: "Draft an executive brief grounded in my workspace files and standing preferences.",
    sample_prompts: [
      "Draft an executive brief grounded in my workspace files and standing preferences.",
      "Write a concise operational SOP from the notes in my workspace.",
    ],
    source: `// draft_brief.rn — workspace-grounded executive memo & SOP author
pub fn run(args) {
  let mem = ergon::read_memory()?;
  let files = ergon::collect_sources(args.sources)?;
  let memo = ergon::compose_brief(args.topic, files, mem);
  ergon::write_document(args.out_file, memo.sections)?;
  Ok(memo.citations)
}`,
    builtin: true,
  },
  {
    id: "extract_tables",
    title: "Extract tables",
    category: "Data",
    tone: "mint",
    filename: "extract_tables.rn",
    description:
      "Parses unstructured Markdown tables, key-value logs, or bulleted metrics into clean, verified RFC-4180 CSV spreadsheets.",
    inputs: ["source_file", "output_csv?"],
    sample_prompt: "Extract structured tables from my workspace notes into a clean CSV spreadsheet.",
    sample_prompts: [
      "Extract structured tables from my workspace notes into a clean CSV spreadsheet.",
      "Parse key-value lines from logs or notes into extracted-table.csv.",
    ],
    source: `// extract_tables.rn — unstructured Markdown/log text -> verified CSV
pub fn run(args) {
  let text = ergon::read_file(args.source_file)?;
  let table = ergon::extract_structured_rows(text);
  ergon::make_spreadsheet(args.output_csv, table.columns, table.rows)?;
  Ok(table.row_count)
}`,
    builtin: true,
  },
  {
    id: "make_spreadsheet",
    title: "Make spreadsheet",
    category: "Data",
    tone: "mint",
    filename: "make_spreadsheet.rn",
    description:
      "Writes verified RFC-4180 CSV spreadsheets with checked column totals, normalized numeric columns, and clean headers.",
    inputs: ["title", "columns", "rows"],
    sample_prompt: "Make a clean Q4 budget spreadsheet in CSV format with checked column totals.",
    sample_prompts: [
      "Make a clean Q4 budget spreadsheet in CSV format with checked column totals.",
      "Build a vendor tracking CSV from my workspace files.",
    ],
    source: `// make_spreadsheet.rn — verified CSV authoring with checked column totals
pub fn run(args) {
  let csv = ergon::format_csv(args.columns, args.rows);
  ergon::write_file(args.title, csv)?;
  Ok(args.rows.len())
}`,
    builtin: true,
  },
  {
    id: "write_document",
    title: "Write document",
    category: "Documents",
    tone: "ops",
    filename: "write_document.rn",
    description:
      "Formats structured Markdown deliverables, reports, and checklists into your sandboxed workspace while applying standing memory rules.",
    inputs: ["title", "sections"],
    sample_prompt: "Write a structured weekly status document summarizing finished workspace deliverables.",
    sample_prompts: [
      "Write a structured weekly status document summarizing finished workspace deliverables.",
      "Tidy my workspace notes into a clear Markdown summary document.",
    ],
    source: `// write_document.rn — structured Markdown report writer
pub fn run(args) {
  let body = ergon::format_markdown(args.title, args.sections);
  ergon::write_file(args.title, body)?;
  Ok(body.len())
}`,
    builtin: true,
  },
  {
    id: "research",
    title: "Research",
    category: "Research",
    tone: "research",
    filename: "research.rn",
    description:
      "Cross-searches workspace files and notes to compile sourced research findings with explicit file and line citations.",
    inputs: ["topic", "query?"],
    sample_prompt: "Research all workspace files for vendor risks and deadlines, and save a cited findings brief.",
    sample_prompts: [
      "Research all workspace files for vendor risks and deadlines, and save a cited findings brief.",
      "Search workspace files for Q3/Q4 deliverables and compile sourced research notes.",
    ],
    source: `// research.rn — sourced multi-file workspace research & citation compiler
pub fn run(args) {
  let hits = ergon::search_files(args.query)?;
  let brief = ergon::compile_research(args.topic, hits);
  ergon::write_document(args.out_file, brief)?;
  Ok(hits.len())
}`,
    builtin: true,
  },
  {
    id: "update_memory",
    title: "Update memory",
    category: "Memory",
    tone: "plan",
    filename: "update_memory.rn",
    description:
      "Records standing employee preferences, formatting rules, and review standards into personal memory.md so future tasks follow them automatically.",
    inputs: ["note"],
    sample_prompt: "Remember to lead every executive brief with top risks, owners, and exact dates.",
    sample_prompts: [
      "Remember to lead every executive brief with top risks, owners, and exact dates.",
      "Always flag expense variances over $250 in reconciliations.",
    ],
    source: `// update_memory.rn — standing personal preferences persisted in memory.md
pub fn run(args) {
  ergon::append_memory_rule(args.note)?;
  Ok("recorded")
}`,
    builtin: true,
  },
];

function inferCategory(raw: unknown, fallback: SkillCategory = "Operations"): SkillCategory {
  const s = String(raw ?? "").trim().toLowerCase();
  if (/fin|ledger|budget|expense|book/.test(s)) return "Finance";
  if (/doc|meet|brief|write|memo|summary/.test(s)) return "Documents";
  if (/data|csv|table|sheet|clean/.test(s)) return "Data";
  if (/comp|audit|contract|diff|legal|risk/.test(s)) return "Compliance";
  if (/res|search|invest/.test(s)) return "Research";
  if (/mem|pref|learn/.test(s)) return "Memory";
  if (/ops|triage|inbox|work/.test(s)) return "Operations";
  return fallback;
}

function toneForCategory(cat: SkillCategory): SkillTone {
  switch (cat) {
    case "Finance":
    case "Data":
      return "mint";
    case "Compliance":
    case "Memory":
      return "plan";
    case "Research":
      return "research";
    default:
      return "ops";
  }
}

function extractInputs(raw: unknown, fallback: string[]): string[] {
  if (Array.isArray(raw)) {
    const arr = raw
      .map((x) => {
        if (x && typeof x === "object") {
          const obj = x as { name?: unknown; required?: unknown };
          const n = String(obj.name ?? "").trim();
          if (!n) return "";
          return obj.required === false ? `${n}?` : n;
        }
        return String(x ?? "").trim();
      })
      .filter(Boolean);
    if (arr.length) return arr;
  }
  if (raw && typeof raw === "object") {
    const props = (raw as { properties?: Record<string, unknown> }).properties;
    const keys = props && typeof props === "object" ? Object.keys(props) : Object.keys(raw as Record<string, unknown>);
    if (keys.length) return keys;
  }
  return fallback;
}

/** Merge live /api/skills payload with the canonical 12-skill catalog without losing metadata. */
export function mergeServerSkills(payload: unknown): SkillMeta[] {
  const byId = new Map<string, SkillMeta>(CANONICAL_SKILLS.map((s) => [s.id, { ...s }]));
  const rawList: unknown[] = Array.isArray(payload)
    ? payload
    : payload && typeof payload === "object"
      ? (Array.isArray((payload as { skills?: unknown[] }).skills)
          ? (payload as { skills: unknown[] }).skills
          : Array.isArray((payload as { items?: unknown[] }).items)
            ? (payload as { items: unknown[] }).items
            : [])
      : [];

  for (const entry of rawList) {
    if (!entry || typeof entry !== "object") continue;
    const obj = entry as Record<string, unknown>;
    const id = String(obj.id ?? "").trim().replace(/-/g, "_");
    if (!id) continue;
    const base = byId.get(id);
    const category = inferCategory(obj.category, base?.category ?? "Operations");
    const samplesRaw = Array.isArray(obj.sample_prompts)
      ? obj.sample_prompts.map((x) => String(x ?? "").trim()).filter(Boolean)
      : [];
    const singleSample = String(obj.sample_prompt ?? "").trim();
    const sample_prompts = samplesRaw.length
      ? samplesRaw
      : singleSample
        ? [singleSample]
        : (base?.sample_prompts ?? [`Run ${id.replace(/_/g, " ")} on my workspace files.`]);
    byId.set(id, {
      id,
      title: String(obj.title ?? obj.name ?? base?.title ?? id.replace(/_/g, " ")).trim(),
      category,
      tone: base?.tone ?? toneForCategory(category),
      filename: String(obj.filename ?? base?.filename ?? `${id}.rn`).trim(),
      description: String(obj.description ?? base?.description ?? "Sandboxed Rune workflow skill.").trim(),
      inputs: extractInputs(obj.input_schema ?? obj.params ?? obj.inputs, base?.inputs ?? ["input"]),
      sample_prompt: singleSample || sample_prompts[0],
      sample_prompts,
      source: String(obj.source ?? obj.rune_source ?? base?.source ?? "").trim(),
      builtin: typeof obj.builtin === "boolean" ? obj.builtin : (base?.builtin ?? true),
    });
  }

  return Array.from(byId.values());
}
