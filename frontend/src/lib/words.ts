// Plain words, file-type glyph helpers, starter cards, and update error translations.
import { File, FileCode, FileImage, FileSpreadsheet, FileText } from "lucide-react";
import type { SuiteItem, Update } from "./api";

export type StarterCard = {
  category: string;
  tone: "ops" | "mint" | "plan" | "research";
  prompt: string;
  detail: string;
  skill: string;
};

export function buildStarterCards(installed: SuiteItem[], role: string): StarterCard[] {
  const safeRole = typeof role === "string" ? role.trim() : "";
  const cards: StarterCard[] = [
    {
      category: "Documents",
      tone: "ops",
      prompt: "Tidy my notes into a summary",
      detail: "Reads workspace notes and writes a structured Markdown brief.",
      skill: "write_document",
    },
  ];
  const tones: ("mint" | "plan" | "research")[] = ["mint", "plan", "research"];
  const safeInstalled = Array.isArray(installed) ? installed.filter((x) => x && typeof x.name === "string") : [];
  safeInstalled.slice(0, 2).forEach((item, idx) => {
    const kindWord = item.kind === "subagent" ? "Specialist" : item.kind === "app" ? "One-tap app" : "Workflow";
    cards.push({
      category: kindWord,
      tone: tones[idx % tones.length],
      prompt: `Ask my ${item.name} for help`,
      detail: typeof item.description === "string" && item.description ? item.description : "Runs this capability inside your sandboxed workspace.",
      skill: item.id || item.name,
    });
  });
  if (safeRole) {
    cards.push({
      category: safeRole,
      tone: "plan",
      prompt: "Draft my weekly update",
      detail: `Compiles finished ${safeRole.toLowerCase()} deliverables and open blockers into Weekly update.md.`,
      skill: "write_document",
    });
  }
  cards.push({
    category: "Spreadsheet",
    tone: "mint",
    prompt: "Reconcile expenses into a clean CSV",
    detail: "Checks line-item totals twice and writes a verified CSV table.",
    skill: "make_spreadsheet",
  });
  cards.push({
    category: "Research",
    tone: "research",
    prompt: "Research a topic for me",
    detail: "Reads sources and saves cited findings into your workspace.",
    skill: "research",
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
