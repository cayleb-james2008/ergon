// Plain words, file-type glyph helpers, starter cards, and update error translations.
import { File, FileCode, FileImage, FileSpreadsheet, FileText } from "lucide-react";
import type { SuiteItem, Update, WorkspaceFile } from "./api";

export type StarterCard = {
  label: string;
  category: string;
  tone: "ops" | "mint" | "plan" | "research";
  prompt: string;
  detail: string;
  skill: string;
};

export type DetectedWorkflow = {
  skillId: string;
  label: string;
  file: string;
  prompt: string;
};

/** Picks the highest-leverage built-in workflow for a workspace file from its name & extension. */
export function detectWorkflowForFile(path: string): DetectedWorkflow | null {
  const clean = String(path || "").trim();
  if (!clean) return null;
  const low = clean.toLowerCase();
  const ext = low.split(".").pop() || "";

  if (ext === "csv" || ext === "tsv" || ext === "xlsx" || ext === "xls") {
    if (/(expense|invoice|ledger|spend|budget|receipt|ap|ar|payout|billing|cost)/.test(low)) {
      return {
        skillId: "reconcile_ledger",
        label: "Reconcile ledger",
        file: clean,
        prompt: `Reconcile ${clean} with reconcile_ledger, flag duplicates or missing receipts, and export a verified Excel (.xlsx) workbook`,
      };
    }
    return {
      skillId: "clean_dataset",
      label: "Clean & summarize sheet",
      file: clean,
      prompt: `Clean and profile ${clean} with clean_dataset, normalize columns, and export a clean Excel (.xlsx) summary`,
    };
  }
  if (ext === "pdf" || ext === "docx" || ext === "doc") {
    if (/(contract|nda|msa|agreement|terms|lease|vendor|policy|clause)/.test(low)) {
      return {
        skillId: "audit_contract",
        label: "Audit contract",
        file: clean,
        prompt: `Audit ${clean} with audit_contract for risks, missing clauses, and action owners`,
      };
    }
    return {
      skillId: "extract_tables",
      label: "Extract & summarize",
      file: clean,
      prompt: `Read ${clean}, extract structured tables to CSV with extract_tables, and summarize key takeaways`,
    };
  }
  if (ext === "md" || ext === "markdown" || ext === "txt" || ext === "log") {
    if (/(meeting|standup|sync|call|transcript|agenda|minutes)/.test(low)) {
      return {
        skillId: "summarize_meeting",
        label: "Summarize meeting",
        file: clean,
        prompt: `Summarize ${clean} with summarize_meeting into decisions, action items, and owners, then export as Word (.docx)`,
      };
    }
    if (/(inbox|email|mail|triage|ticket|support)/.test(low)) {
      return {
        skillId: "triage_inbox",
        label: "Triage inbox",
        file: clean,
        prompt: `Triage ${clean} with triage_inbox into Urgent, Action, and FYI with suggested replies`,
      };
    }
    if (/(contract|nda|msa|agreement|terms)/.test(low)) {
      return {
        skillId: "audit_contract",
        label: "Audit contract",
        file: clean,
        prompt: `Audit ${clean} with audit_contract for risks, missing clauses, and action owners`,
      };
    }
    return {
      skillId: "write_document",
      label: "Polish to Word",
      file: clean,
      prompt: `Tidy ${clean} into an executive-ready summary document and export as Word (.docx)`,
    };
  }
  if (ext === "json") {
    return {
      skillId: "clean_dataset",
      label: "Analyze JSON",
      file: clean,
      prompt: `Inspect ${clean}, extract key structured records into a clean CSV/Markdown summary, and flag anomalies`,
    };
  }
  return null;
}

/** Finds the single best next workflow across the active workspace's files. */
export function detectWorkspaceWorkflow(files: WorkspaceFile[]): DetectedWorkflow | null {
  if (!Array.isArray(files) || files.length === 0) return null;
  for (const f of files) {
    if (!f?.path || /(-reconciled|-audit|-summary|-cleaned)\./i.test(f.path)) continue;
    const hit = detectWorkflowForFile(f.path);
    if (hit) return hit;
  }
  return detectWorkflowForFile(files[0].path);
}

export function buildStarterCards(installed: SuiteItem[], role: string, files: WorkspaceFile[] = []): StarterCard[] {
  const safeRole = typeof role === "string" ? role.trim() : "";
  const docFile = files.find((f) => /\.(md|markdown|txt)$/i.test(f.path) && !/(summary|brief|audit)\./i.test(f.path));
  const sheetFile = files.find((f) => /\.(csv|tsv|xlsx)$/i.test(f.path) && !/-reconciled\./i.test(f.path));
  const contractFile = files.find((f) => /(contract|nda|msa|agreement|vendor|policy)/i.test(f.path))
    || files.find((f) => /\.(pdf|docx)$/i.test(f.path));

  const cards: StarterCard[] = [
    {
      label: "Summarize to Word",
      category: "Word (.docx)",
      tone: "ops",
      prompt: docFile
        ? `Tidy ${docFile.path} into a summary document and export as Word`
        : "Tidy my notes into a summary document and export as Word",
      detail: docFile
        ? `Reads ${docFile.path}, writes a clean summary, and prepares a formatted Word (.docx) deliverable.`
        : "Reads workspace notes, writes Summary.md, and prepares a formatted Word deliverable.",
      skill: "write_document",
    },
  ];
  const tones: ("mint" | "plan" | "research")[] = ["mint", "plan", "research"];
  const safeInstalled = Array.isArray(installed) ? installed.filter((x) => x && typeof x.name === "string") : [];
  safeInstalled.slice(0, 1).forEach((item, idx) => {
    const kindWord = item.kind === "subagent" ? "Specialist" : item.kind === "app" ? "One-tap app" : "Workflow";
    cards.push({
      label: item.name,
      category: kindWord,
      tone: tones[idx % tones.length],
      prompt: `Ask my ${item.name} for help`,
      detail: typeof item.description === "string" && item.description ? item.description : "Runs this capability inside your sandboxed workspace.",
      skill: item.id || item.name,
    });
  });
  cards.push({
    label: "Reconcile to Excel",
    category: "Excel (.xlsx)",
    tone: "mint",
    prompt: sheetFile
      ? `Reconcile ${sheetFile.path} into a clean spreadsheet and export as Excel`
      : "Reconcile expenses into a clean spreadsheet and export as Excel",
    detail: sheetFile
      ? `Audits ${sheetFile.path}, checks line-item totals twice, and exports an .xlsx workbook.`
      : "Checks line-item totals twice, saves a verified CSV, and exports an .xlsx workbook.",
    skill: "make_spreadsheet",
  });
  cards.push({
    label: "Audit contract",
    category: safeRole ? `${safeRole} audit` : "Contract audit",
    tone: "plan",
    prompt: contractFile
      ? `Audit ${contractFile.path} for risks, missing clauses, and action owners`
      : "Audit my workspace documents for risks, missing clauses, and action owners",
    detail: contractFile
      ? `Scans ${contractFile.path}, flags gaps with exact line citations, and builds an audit report.`
      : "Scans contracts and briefs, flags gaps with exact line citations, and builds an audit report.",
    skill: "audit_contract",
  });
  cards.push({
    label: "Executive brief",
    category: "Executive brief",
    tone: "research",
    prompt: "Draft a weekly executive brief from finished deliverables and open items",
    detail: "Compiles recent work, open approvals, and cited findings into Weekly update.md.",
    skill: "draft_brief",
  });
  return cards.slice(0, 4);
}

export function toolLabel(tool: string, via: string | undefined, body: string): string | null {
  const pick = (re: RegExp) => { const m = re.exec(body); return m ? m[1] : ""; };
  let label: string | null;
  switch (tool) {
    case "list_dir": label = "Looked through your files"; break;
    case "read_file": label = "Read a file"; break;
    case "write_file": { const n = pick(/^wrote (.+?) \(\d+ bytes\)/); label = n ? "Saved " + n : "Saved a file"; break; }
    case "write_document": { const n = pick(/^wrote document (.+?) \(/); label = n ? "Saved " + n : "Saved a document"; break; }
    case "make_spreadsheet": { const n = pick(/^wrote spreadsheet (.+?) \(/); label = n ? "Saved spreadsheet " + n : "Saved a spreadsheet"; break; }
    case "move_file": label = "Moved a file"; break;
    case "search_files": label = "Searched your files"; break;
    case "fetch_url": label = "Read a web page"; break;
    case "research": label = "Researched a topic"; break;
    case "update_memory": label = "Remembered something about you"; break;
    case "browse_library": label = "Looked through the company library"; break;
    case "install_item": label = "Added an item to your suite"; break;
    case "publish_skill": label = "Published a skill"; break;
    case "delete_file": { const n = pick(/(\S+) has NOT been deleted/); label = "Asked your approval to delete " + (n || "a file"); break; }
    case "flag_decision": label = "Asked you to decide something"; break;
    case "delegate": label = via ? "Handed work to " + via : "Handed work to a specialist"; break;
    case "reconcile_ledger": { const n = pick(/(?:wrote|reconciled)\s+([^\s(]+)/i); label = n ? "Reconciled ledger " + n : "Reconciled expense ledger"; break; }
    case "summarize_meeting": label = "Summarized meeting notes & owners"; break;
    case "audit_contract": label = "Audited contract clauses & risks"; break;
    case "triage_inbox": label = "Triaged inbox into Urgent / Action / FYI"; break;
    case "clean_dataset": label = "Cleaned & deduplicated dataset"; break;
    case "compare_documents": label = "Compared two documents line by line"; break;
    case "draft_brief": label = "Drafted grounded executive brief"; break;
    case "extract_tables": label = "Extracted structured CSV table"; break;
    default: return null;
  }
  if (via && tool !== "delegate") label += " (for " + via + ")";
  return label;
}

/** The workspace-relative file a save step wrote. */
export function savedFile(tool: string, body: string): string {
  const re = tool === "write_document" ? /^wrote document (.+?) \(/
    : tool === "make_spreadsheet" ? /^wrote spreadsheet (.+?) \(/
    : tool === "write_file" ? /^wrote (.+?) \(\d+ bytes\)/
    : /^(reconcile_ledger|summarize_meeting|audit_contract|triage_inbox|clean_dataset|compare_documents|draft_brief|extract_tables)$/.test(tool)
      ? /^wrote (?:document |spreadsheet |report |file )?(.+?) \(/
    : null;
  const m = re && re.exec(body);
  return m ? m[1] : "";
}

/** Icon component for a workspace filename by extension. */
export function fileIconFor(name: string) {
  const ext = String(name ?? "").split(".").pop()?.toLowerCase() ?? "";
  if (/^(csv|xlsx?|tsv|ods)$/.test(ext)) return FileSpreadsheet;
  if (/^(png|jpe?g|gif|webp|svg)$/.test(ext)) return FileImage;
  if (/^(json|html?|css|js|ts|py|rs|rn|toml|xml|ya?ml)$/.test(ext)) return FileCode;
  if (/^(md|txt|docx?|pdf|rtf|odt)$/.test(ext)) return FileText;
  return File;
}

export function normalizeFiles(files: unknown): string[] {
  if (!Array.isArray(files)) return [];
  return files
    .filter((f): f is string | number => typeof f === "string" || typeof f === "number")
    .map((f) => String(f).trim())
    .filter(Boolean);
}

export const FAILURE_RE = /not a JSON object|no model key|model endpoint returned HTTP|without finishing|could not reach|could not run|did not finish/i;
export const PARKED_RE = /parked for your person|has NOT been deleted yet/;
export const RESOLVED_RE = /was deleted after you approved it|was kept because you declined/;

export function relTime(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "";
  const now = new Date();
  const then = new Date(ms);
  const mins = Math.round((now.getTime() - then.getTime()) / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return mins + " min ago";
  const dayNow = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dayThen = new Date(then.getFullYear(), then.getMonth(), then.getDate());
  const days = Math.round((dayNow.getTime() - dayThen.getTime()) / 86400000);
  if (days === 0) return Math.round(mins / 60) + " h ago";
  if (days === 1) return "Yesterday";
  return then.toLocaleDateString([], { month: "short", day: "numeric" });
}

export function threadMs(id: string): number | null {
  const m = /^t(\d{13})/.exec(String(id || ""));
  return m ? Number(m[1]) : null;
}

/** Turns the server's update error text into one plain sentence. Raw text is shown only in Details. */
export function plainUpdateError(raw: string): string {
  const r = String(raw || "");
  if (/could not reach|Connection/i.test(r)) return "Could not reach GitHub. Check your internet connection, then try again.";
  if (/checksum/i.test(r)) return "The download did not pass its safety check, so it was not installed. Nothing changed. Try again in a few minutes.";
  if (/answered HTTP/i.test(r)) return "GitHub had a problem. Nothing changed. Try again in a few minutes.";
  if (/only allowed from this machine/i.test(r)) return "Updating works only on the computer Ergon runs on. Open it there to update.";
  if (/does not run on this machine/i.test(r)) return "The new version does not run on this computer, so it was not installed. Nothing changed.";
  return "The update did not finish. Nothing changed. Try again in a few minutes.";
}

export function resultLine(s: Update): string {
  if (s.mode === "off") return "Update checks are turned off on this Ergon.";
  if (s.error) return plainUpdateError(s.error);
  if (s.staged) return "Version " + s.staged + " installed. Restart to use it.";
  if (s.available) return "Version " + s.latest + " is available";
  if (!s.checked_at) return "Not checked yet.";
  const mins = Math.round((Date.now() - s.checked_at * 1000) / 60000);
  const when = mins < 1 ? "just now" : mins < 60 ? mins + " min ago" : Math.round(mins / 60) + " h ago";
  return "You are on the newest version (checked " + when + ")";
}

export const isLocal = () => ["localhost", "127.0.0.1", "[::1]"].includes(location.hostname);
