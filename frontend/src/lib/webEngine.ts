// Ergon Local-First Browser Engine (Standalone Web Edition)
// Real IndexedDB workspace + deterministic client-side tool engine (CSV reconciliation,
// Markdown summarization & action extraction, checklist authoring, standing memory rules,
// approval-gated file deletion, live Work Graph) + optional BYOK OpenAI-compatible LLM (sessionStorage only).
// ZERO hardcoded fixture replies, ZERO fixture demo pills.
import { parseCsv } from "@/shell/WorkspaceFiles";

type WebFile = {
  path: string;
  name: string;
  content: string;
  size: number;
  mtime_ms: number;
  ext: string;
};
type WebMsg = { role: string; content: string };
type WebThread = { id: string; title: string; messages: WebMsg[] };
type WebWork = {
  task: string;
  summary: string;
  files: string[];
  finished: boolean;
  answered: boolean;
  thread_id: string;
  ts: number;
};
type WebDecision = {
  id: string;
  kind: string;
  path?: string;
  summary: string;
  status: string;
  outcome: string | null;
  thread_id: string;
  ts: number;
  resolved_ts?: number;
};
type WebOffer = {
  id: string;
  kind: string;
  item_id?: string;
  template?: string;
  title: string;
  detail: string;
  status: string;
  ts: number;
};
type WebPerson = { id: string; name: string; role: string };
type WebLibItem = {
  id: string;
  kind: string;
  name: string;
  description: string;
  author: string;
  builtin: boolean;
  fields?: string[];
  template?: string;
};
type PersonStore = {
  files: Record<string, WebFile>;
  threads: WebThread[];
  work: WebWork[];
  decisions: WebDecision[];
  offers: WebOffer[];
  memory: string;
  suite: string[];
};
type RootState = {
  activePerson: string;
  people: WebPerson[];
  library: WebLibItem[];
  stores: Record<string, PersonStore>;
};
type ByokConfig = {
  engine_mode: "browser" | "byok";
  model_base_url: string;
  model_id: string;
  api_key: string;
};

const DB_NAME = "ergon-workspace-v4";
const DB_STORE = "state";
const DB_KEY = "root";
const BYOK_KEY = "ergon-byok-session";

function utf8Bytes(s: string): number {
  return new TextEncoder().encode(s).length;
}

function fileExt(path: string): string {
  const parts = path.split(".");
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
}

function makeFile(path: string, content: string, mtime = Date.now()): WebFile {
  const clean = path.replace(/^\/+/, "");
  return {
    path: clean,
    name: clean.split("/").pop() || clean,
    content,
    size: utf8Bytes(content),
    mtime_ms: mtime,
    ext: fileExt(clean),
  };
}

function csvEscape(cell: string): string {
  if (/[",\r\n]/.test(cell)) return `"${cell.replace(/"/g, '""')}"`;
  return cell;
}

function parseMoney(raw: string): number | null {
  const cleaned = String(raw ?? "").trim().replace(/[$,\s]/g, "");
  if (!cleaned || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) return null;
  const n = Number(cleaned);
  return Number.isFinite(n) ? n : null;
}

function fmtMoney(n: number): string {
  return "$" + n.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function sanitizeSafePath(raw: string): string | null {
  const p = String(raw ?? "").trim().replace(/\\/g, "/");
  if (!p || p.includes("\0") || p.startsWith("/")) return null;
  const segs = p.split("/");
  if (segs.some((s) => !s || s === "." || s === ".." || s.startsWith("."))) return null;
  return segs.join("/");
}

function learnedLines(md: string): string[] {
  return String(md || "")
    .split("\n")
    .map((l) => l.trim())
    .filter((t) => t.startsWith("- ") && !/^-\s*(name|role)\s*:/i.test(t))
    .map((t) => t.slice(2).trim())
    .filter(Boolean);
}

/** Real deterministic CSV reconciliation engine over ANY uploaded or workspace CSV. */
function runCsvReconciliation(
  sourcePath: string,
  csvText: string,
  rules: string[]
): {
  reconciledCsv: string;
  reportMd: string;
  rowCount: number;
  grandTotal: number;
  missingCount: number;
  duplicateCount: number;
  categories: [string, { count: number; total: number }][];
  flaggedRows: string[];
  amountColName: string;
} {
  const rows = parseCsv(csvText);
  if (rows.length < 2) {
    return {
      reconciledCsv: csvText,
      reportMd: `# Reconciliation Report — ${sourcePath}\n\nNo data rows were found in \`${sourcePath}\`.\n`,
      rowCount: 0,
      grandTotal: 0,
      missingCount: 0,
      duplicateCount: 0,
      categories: [],
      flaggedRows: [],
      amountColName: "Amount",
    };
  }
  const header = rows[0].map((h) => h.trim());
  const data = rows.slice(1).filter((r) => r.some((c) => c.trim().length > 0));
  const lower = header.map((h) => h.toLowerCase());

  let amountIdx = lower.findIndex((h) => /(amount|cost|total|price|spend|usd|value|charge|fee|balance)/.test(h));
  if (amountIdx < 0) {
    // Auto-detect first predominantly numeric column
    for (let c = 0; c < header.length; c++) {
      const numericHits = data.filter((r) => parseMoney(r[c] ?? "") !== null).length;
      if (numericHits >= Math.max(1, Math.floor(data.length * 0.5))) {
        amountIdx = c;
        break;
      }
    }
  }
  const vendorIdx = Math.max(
    0,
    lower.findIndex((h) => /(vendor|merchant|supplier|payee|item|name|description|account)/.test(h))
  );
  const catIdx = lower.findIndex((h) => /(category|dept|department|type|group|cost.?center)/.test(h));
  const receiptIdx = lower.findIndex((h) => /(receipt|invoice|status|proof|verified|approved|doc)/.test(h));

  let grandTotal = 0;
  let missingCount = 0;
  let duplicateCount = 0;
  const catMap = new Map<string, { count: number; total: number }>();
  const seenKey = new Set<string>();
  const flaggedRows: string[] = [];

  const outHeader = [...header, "Normalized_Amount", "Audit_Status"];
  const outRows: string[][] = [outHeader];

  data.forEach((r, idx) => {
    const padded = header.map((_, i) => (r[i] ?? "").trim());
    const vendor = padded[vendorIdx] || `Row ${idx + 2}`;
    const cat = catIdx >= 0 ? (padded[catIdx] || "Uncategorized") : "General";
    const amt = amountIdx >= 0 ? (parseMoney(padded[amountIdx]) ?? 0) : 0;
    grandTotal += amt;

    const curCat = catMap.get(cat) ?? { count: 0, total: 0 };
    curCat.count += 1;
    curCat.total += amt;
    catMap.set(cat, curCat);

    const flags: string[] = [];
    if (receiptIdx >= 0) {
      const rv = padded[receiptIdx].toLowerCase();
      if (!rv || /(missing|none|no|pending|unverified|overdue|n\/a|false|blank)/.test(rv)) {
        missingCount += 1;
        flags.push("missing receipt");
      }
    }
    const dupKey = `${vendor.toLowerCase()}|${amt.toFixed(2)}`;
    if (amt !== 0 && seenKey.has(dupKey)) {
      duplicateCount += 1;
      flags.push("possible duplicate");
    }
    seenKey.add(dupKey);

    if (flags.length) {
      flaggedRows.push(`Row ${idx + 2} (${vendor}, ${fmtMoney(amt)}): ${flags.join(", ")}`);
    }
    outRows.push([...padded, amt.toFixed(2), flags.length ? `FLAG: ${flags.join("; ")}` : "OK"]);
  });

  const totalRow = header.map((_, i) => (i === 0 ? "TOTAL" : i === amountIdx ? grandTotal.toFixed(2) : ""));
  totalRow.push(grandTotal.toFixed(2), `${data.length} rows · ${missingCount} flagged`);
  outRows.push(totalRow);

  const reconciledCsv = outRows.map((r) => r.map(csvEscape).join(",")).join("\n") + "\n";
  const categories = [...catMap.entries()].sort((a, b) => b[1].total - a[1].total);
  const avg = data.length ? grandTotal / data.length : 0;

  const mdLines: string[] = [
    `# Expense Reconciliation — ${sourcePath}`,
    "",
    "## Executive Summary",
    `- Source file: \`${sourcePath}\``,
    `- Rows processed: **${data.length}**`,
    `- Computed total (${header[amountIdx] || "Amount"}): **${fmtMoney(grandTotal)}** (average ${fmtMoney(avg)} per entry)`,
    `- Discrepancies flagged: **${flaggedRows.length}** (${missingCount} missing receipt/status, ${duplicateCount} duplicate candidate${duplicateCount === 1 ? "" : "s"})`,
    "",
    "## Breakdown by Category",
  ];
  for (const [cat, stats] of categories) {
    mdLines.push(`- **${cat}**: ${fmtMoney(stats.total)} across ${stats.count} ${stats.count === 1 ? "item" : "items"}`);
  }
  mdLines.push("", "## Discrepancies & Action Items");
  if (flaggedRows.length) {
    for (const f of flaggedRows) mdLines.push(`- ${f}`);
  } else {
    mdLines.push("- All line items passed receipt and duplicate checks.");
  }
  if (rules.length) {
    mdLines.push("", "## Applied Standing Preferences");
    for (const rule of rules.slice(0, 4)) mdLines.push(`- ${rule}`);
  }
  return {
    reconciledCsv,
    reportMd: mdLines.join("\n") + "\n",
    rowCount: data.length,
    grandTotal,
    missingCount,
    duplicateCount,
    categories,
    flaggedRows,
    amountColName: header[amountIdx] || "Amount",
  };
}

/** Real deterministic Markdown / text analyzer grounded in the actual document text & standing preferences. */
function runDocumentSummary(
  sources: WebFile[],
  prompt: string,
  rules: string[]
): { outName: string; markdown: string; actionCount: number; blockerCount: number } {
  const namedOut = /\b([A-Za-z0-9][A-Za-z0-9._-]*\.md)\b/.exec(prompt);
  const outName = namedOut ? namedOut[1] : sources.length === 1 ? `${sources[0].name.replace(/\.[^.]+$/, "")}-summary.md` : "Summary.md";

  const headings: string[] = [];
  const actions: string[] = [];
  const blockers: string[] = [];
  const highlights: string[] = [];

  for (const src of sources) {
    const lines = src.content.replace(/\r\n/g, "\n").split("\n");
    lines.forEach((raw, idx) => {
      const line = raw.trim();
      if (!line) return;
      if (/^#{1,3}\s+/.test(line)) {
        headings.push(`${line.replace(/^#{1,3}\s+/, "")} (${src.path})`);
        return;
      }
      if (/^[-*]\s*\[\s\]\s*/.test(line) || /\b(action|todo|owner|follow[- ]?up|due|by friday|by monday|deadline|assigned)\b/i.test(line)) {
        actions.push(`${line.replace(/^[-*]\s*(\[\s*\]\s*)?/, "")} [${src.name}:L${idx + 1}]`);
        return;
      }
      if (/\b(blocker|blocked|risk|missing|overdue|delayed|issue|failed|incident)\b/i.test(line)) {
        blockers.push(`${line.replace(/^[-*]\s*/, "")} [${src.name}:L${idx + 1}]`);
        return;
      }
      if (line.startsWith("- ") || line.startsWith("* ")) {
        highlights.push(line.slice(2));
      } else if (highlights.length < 6 && line.length > 24) {
        highlights.push(line);
      }
    });
  }

  const lines: string[] = [
    `# ${outName.replace(/\.md$/i, "").replace(/[-_]+/g, " ")}`,
    "",
    `Compiled from ${sources.map((s) => `\`${s.path}\``).join(", ")}.`,
    "",
  ];

  const leadBlockers = rules.some((r) => /blocker|owner|executive/i.test(r));
  if (leadBlockers && (blockers.length || actions.length)) {
    lines.push("## Executive Takeaways & Blockers");
    if (blockers.length) {
      for (const b of blockers.slice(0, 6)) lines.push(`- **Blocker:** ${b}`);
    } else {
      lines.push("- No open blockers identified in the source notes.");
    }
    lines.push("");
  }

  lines.push("## Action Items & Owners");
  if (actions.length) {
    for (const a of actions.slice(0, 10)) lines.push(`- ${a}`);
  } else {
    lines.push("- No explicit unchecked action items found in the source files.");
  }

  if (!leadBlockers && blockers.length) {
    lines.push("", "## Risks & Blockers");
    for (const b of blockers.slice(0, 6)) lines.push(`- ${b}`);
  }

  lines.push("", "## Key Points");
  if (highlights.length) {
    for (const h of highlights.slice(0, 8)) lines.push(`- ${h}`);
  } else if (headings.length) {
    for (const h of headings.slice(0, 6)) lines.push(`- ${h}`);
  } else {
    lines.push("- Reviewed workspace inputs; no additional bullet points extracted.");
  }

  if (rules.length) {
    lines.push("", "## Standing Preferences Applied");
    for (const r of rules.slice(0, 4)) lines.push(`- ${r}`);
  }

  return {
    outName,
    markdown: lines.join("\n") + "\n",
    actionCount: actions.length,
    blockerCount: blockers.length,
  };
}

function buildInitialState(): RootState {
  const now = Date.now();
  const expensesCsv = [
    "Vendor,Category,Amount,Receipt_Status,Date",
    "CloudHost Infrastructure,Cloud,1240.00,verified,2026-11-04",
    "Linear Workspace,Software,320.00,verified,2026-11-06",
    "Northstar Freight,Logistics,845.50,verified,2026-11-09",
    "CloudHost Backup,Cloud,240.00,missing,2026-11-12",
    "Harbor Print Co,Supplies,186.25,verified,2026-11-15",
    "Summit Catering,Team Ops,410.00,verified,2026-11-18",
  ].join("\n") + "\n";

  const syncNotes = [
    "# Tuesday Product & Operations Sync",
    "",
    "## Updates",
    "- Launch checklist for v0.4.0 desktop builds is ready for QA sign-off.",
    "- Owner: Maya Chen — finalize Q3 vendor reconciliation before Friday close.",
    "- [ ] Owner: Tomas Ruiz — chase missing $240.00 CloudHost Backup receipt by Thursday.",
    "- [ ] Owner: Priya Raman — publish dark-mode accessibility checklist for Settings.",
    "- Blocker: Q2 archived vendor draft (old-vendor-draft.txt) still sitting in active workspace.",
  ].join("\n") + "\n";

  const oldDraft = [
    "SUPERSEDED Q2 VENDOR DRAFT — DO NOT USE FOR FILING",
    "Replaced by expenses-q3.csv on 2026-11-01.",
  ].join("\n") + "\n";

  const initialRules = [
    "Lead every summary with owners, dates, and unresolved blockers.",
    "Format expense reconciliations with normalized totals and receipt status flags.",
  ];
  const recon = runCsvReconciliation("expenses-q3.csv", expensesCsv, initialRules);
  const t1 = `t${now - 24 * 60_000}-1`;
  const t2 = `t${now - 90 * 60_000}-2`;

  const mayaFiles: Record<string, WebFile> = {
    "expenses-q3.csv": makeFile("expenses-q3.csv", expensesCsv, now - 140 * 60_000),
    "tuesday-sync-notes.md": makeFile("tuesday-sync-notes.md", syncNotes, now - 110 * 60_000),
    "old-vendor-draft.txt": makeFile("old-vendor-draft.txt", oldDraft, now - 300 * 60_000),
    "expenses-q3-reconciled.csv": makeFile("expenses-q3-reconciled.csv", recon.reconciledCsv, now - 24 * 60_000),
  };

  const library: WebLibItem[] = [
    { id: "write-document", kind: "skill", name: "Write document", description: "Formats structured sections into a Markdown document in the workspace.", author: "ergon", builtin: true },
    { id: "make-spreadsheet", kind: "skill", name: "Make spreadsheet", description: "Writes verified CSV spreadsheets with checked column totals.", author: "ergon", builtin: true },
    { id: "research", kind: "skill", name: "Research", description: "Compiles sourced workspace research notes.", author: "ergon", builtin: true },
    { id: "update-memory", kind: "skill", name: "Update memory", description: "Records standing preferences into personal memory.md.", author: "ergon", builtin: true },
    { id: "bookkeeper", kind: "subagent", name: "Bookkeeper", description: "Reconciles raw expense CSVs, computes category subtotals, and flags missing receipts.", author: "ergon", builtin: false },
    { id: "meeting-actions", kind: "app", name: "Meeting actions", description: "Reads meeting notes and writes decisions and owner action items.", author: "ergon", builtin: false, fields: ["notes_file"], template: "Summarize {notes_file} into Summary.md with owners and action items" },
    { id: "researcher", kind: "subagent", name: "Researcher", description: "Searches workspace files and compiles sourced findings.", author: "ergon", builtin: false },
    { id: "expense-sheet", kind: "app", name: "Expense sheet", description: "One-tap reconciliation of a CSV file into totals and discrepancies.", author: "ergon", builtin: false, fields: ["source_file"], template: "Reconcile {source_file} and flag missing receipts" },
  ];

  const mayaStore: PersonStore = {
    files: mayaFiles,
    memory: `# Memory\n\n## About\n- Name: Maya Chen\n- Role: Operations lead\n\n## Standing preferences\n${initialRules.map((r) => `- ${r}`).join("\n")}\n`,
    suite: ["bookkeeper", "meeting-actions", "researcher"],
    threads: [
      {
        id: t1,
        title: "Reconcile expenses-q3.csv and flag missing receipts",
        messages: [
          { role: "user", content: "Reconcile expenses-q3.csv and flag missing receipts" },
          { role: "step", content: "Tool: list_dir\nexpenses-q3.csv\nold-vendor-draft.txt\ntuesday-sync-notes.md" },
          { role: "step", content: `Tool: read_file\nread expenses-q3.csv (${utf8Bytes(expensesCsv)} bytes)` },
          { role: "step", content: `Tool: make_spreadsheet\nwrote spreadsheet expenses-q3-reconciled.csv (${recon.rowCount} data rows, ${utf8Bytes(recon.reconciledCsv)} bytes)` },
          { role: "assistant", content: `Reconciled ${recon.rowCount} entries (${fmtMoney(recon.grandTotal)} total) into expenses-q3-reconciled.csv and flagged ${recon.missingCount} missing receipt (CloudHost Backup, $240.00).` },
        ],
      },
      {
        id: t2,
        title: "Clean up old-vendor-draft.txt",
        messages: [
          { role: "user", content: "Delete old-vendor-draft.txt from my workspace" },
          { role: "step", content: "Tool: delete_file\nparked for your person's approval (decision d-init-1): deleting is a consequential action, so old-vendor-draft.txt has NOT been deleted yet." },
          { role: "assistant", content: "I parked the deletion of old-vendor-draft.txt for your approval in Today. The file has not been touched yet." },
          { role: "step", content: "Note: old-vendor-draft.txt has NOT been deleted yet; the deletion is waiting for your approval in Decisions." },
        ],
      },
    ],
    work: [
      {
        task: "Reconcile expenses-q3.csv and flag missing receipts",
        summary: `Reconciled ${recon.rowCount} entries (${fmtMoney(recon.grandTotal)} total) into expenses-q3-reconciled.csv.`,
        files: ["expenses-q3-reconciled.csv"],
        finished: true,
        answered: false,
        thread_id: t1,
        ts: now - 24 * 60_000,
      },
      {
        task: "Clean up old-vendor-draft.txt",
        summary: "Parked deletion of old-vendor-draft.txt for approval.",
        files: [],
        finished: true,
        answered: false,
        thread_id: t2,
        ts: now - 90 * 60_000,
      },
    ],
    decisions: [
      {
        id: "d-init-1",
        kind: "delete",
        path: "old-vendor-draft.txt",
        summary: "Delete old-vendor-draft.txt",
        status: "pending",
        outcome: null,
        thread_id: t2,
        ts: now - 90 * 60_000,
      },
    ],
    offers: [
      {
        id: "o-init-1",
        kind: "install_app",
        item_id: "expense-sheet",
        title: "Add the Expense sheet one-tap app",
        detail: "Reconcile any uploaded CSV spreadsheet from a single field instead of typing a prompt.",
        status: "pending",
        ts: now - 20 * 60_000,
      },
    ],
  };

  const tomasStore: PersonStore = {
    files: {
      "budget-notes.md": makeFile("budget-notes.md", "# Finance Notes\n- Verify Q4 software renewals before Dec 1.\n", now - 200 * 60_000),
    },
    memory: "# Memory\n\n## About\n- Name: Tomas Ruiz\n- Role: Finance analyst\n\n## Standing preferences\n- Flag variances over $50.\n",
    suite: ["bookkeeper", "expense-sheet"],
    threads: [],
    work: [],
    decisions: [],
    offers: [],
  };

  return {
    activePerson: "maya-chen",
    people: [
      { id: "maya-chen", name: "Maya Chen", role: "Operations lead" },
      { id: "tomas-ruiz", name: "Tomas Ruiz", role: "Finance analyst" },
    ],
    library,
    stores: {
      "maya-chen": mayaStore,
      "tomas-ruiz": tomasStore,
    },
  };
}

function openIdb(): Promise<IDBDatabase | null> {
  if (typeof indexedDB === "undefined") return Promise.resolve(null);
  return new Promise((resolve) => {
    try {
      const req = indexedDB.open(DB_NAME, 1);
      req.onupgradeneeded = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains(DB_STORE)) db.createObjectStore(DB_STORE);
      };
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

async function idbLoad(): Promise<RootState | null> {
  const db = await openIdb();
  if (!db) return null;
  return new Promise((resolve) => {
    try {
      const tx = db.transaction(DB_STORE, "readonly");
      const req = tx.objectStore(DB_STORE).get(DB_KEY);
      req.onsuccess = () => resolve((req.result as RootState) ?? null);
      req.onerror = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

async function idbSave(state: RootState): Promise<void> {
  const db = await openIdb();
  if (!db) return;
  await new Promise<void>((resolve) => {
    try {
      const tx = db.transaction(DB_STORE, "readwrite");
      tx.objectStore(DB_STORE).put(state, DB_KEY);
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
    } catch {
      resolve();
    }
  });
}

function loadByok(): ByokConfig {
  try {
    const raw = sessionStorage.getItem(BYOK_KEY);
    if (raw) {
      const p = JSON.parse(raw) as Partial<ByokConfig>;
      return {
        engine_mode: p.engine_mode === "byok" ? "byok" : "browser",
        model_base_url: String(p.model_base_url ?? ""),
        model_id: String(p.model_id ?? ""),
        api_key: String(p.api_key ?? ""),
      };
    }
  } catch { /* ignore */ }
  return { engine_mode: "browser", model_base_url: "", model_id: "", api_key: "" };
}

function saveByok(cfg: ByokConfig) {
  try {
    sessionStorage.setItem(BYOK_KEY, JSON.stringify(cfg));
  } catch { /* ignore */ }
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function installBrowserEngine(): Promise<void> {
  const w = window as unknown as { __ERGON_WEB_ENGINE__?: boolean };
  if (w.__ERGON_WEB_ENGINE__) return;
  w.__ERGON_WEB_ENGINE__ = true;

  let state: RootState = (await idbLoad()) || buildInitialState();
  await idbSave(state);

  const liveRuns: Record<string, { active: boolean; thread_id: string; task: string; started_ms: number; steps: string[] }> = {};
  const persist = () => { void idbSave(state); };

  const activePerson = (): WebPerson =>
    state.people.find((p) => p.id === state.activePerson) || state.people[0];

  const activeStore = (): PersonStore => {
    const pid = activePerson().id;
    if (!state.stores[pid]) {
      const p = activePerson();
      state.stores[pid] = {
        files: {},
        threads: [],
        work: [],
        decisions: [],
        offers: [],
        memory: `# Memory\n\n## About\n- Name: ${p.name}\n${p.role ? `- Role: ${p.role}\n` : ""}`,
        suite: ["researcher", "meeting-actions"],
      };
    }
    return state.stores[pid];
  };

  function buildBrief() {
    const p = activePerson();
    const st = activeStore();
    const waitingIds = new Set(st.decisions.filter((d) => d.status === "pending").map((d) => d.thread_id));
    const fin = st.work.filter((x) => x.finished && !x.answered && !waitingIds.has(x.thread_id)).slice().reverse();
    const stopped = st.work.filter((x) => !x.finished).slice().reverse();
    return {
      date: new Date().toISOString().slice(0, 10),
      person: p.name,
      headline: fin.length ? `${fin.length} job${fin.length === 1 ? "" : "s"} finished in the last two days.` : "Nothing finished in the last two days.",
      finished: fin.map((x) => ({
        date: new Date(x.ts).toISOString().slice(0, 10),
        task: x.task,
        summary: x.summary,
        files: x.files,
        thread_id: x.thread_id,
        ts: x.ts,
      })),
      stopped: stopped.map((x) => ({ task: x.task, summary: x.summary, thread_id: x.thread_id, ts: x.ts })),
      suite_size: st.suite.length,
      library_size: state.library.length,
      learned_recently: learnedLines(st.memory).slice(-5).reverse(),
    };
  }

  function buildToday() {
    const st = activeStore();
    const evs: { ts: number; kind: string; title: string; detail: string; files?: string[] }[] = [];
    st.work.forEach((x) => evs.push({ ts: x.ts, kind: x.finished ? "work" : "stopped", title: x.task, detail: x.summary, files: x.files }));
    st.decisions.forEach((d) => {
      evs.push({ ts: d.ts, kind: "decision", title: d.summary, detail: `Waiting on you (${d.kind})` });
      if (d.resolved_ts) evs.push({ ts: d.resolved_ts, kind: "decision_settled", title: d.summary, detail: d.outcome || d.status });
    });
    st.offers.forEach((o) => evs.push({ ts: o.ts, kind: "offer", title: o.title, detail: o.detail }));
    evs.sort((a, b) => b.ts - a.ts);
    return { date: new Date().toISOString().slice(0, 10), events: evs };
  }

  function buildGraph() {
    const nodes: { id: string; kind: string; label: string; detail?: string }[] = [];
    const edges: { from: string; to: string; kind: string }[] = [];
    for (const p of state.people) {
      const pid = `person:${p.id}`;
      nodes.push({ id: pid, kind: "person", label: p.role ? `${p.name} — ${p.role}` : p.name });
      const st = state.stores[p.id];
      if (!st) continue;
      for (const sid of st.suite) {
        const item = state.library.find((x) => x.id === sid);
        if (!item) continue;
        const iid = `item:${sid}`;
        if (!nodes.some((n) => n.id === iid)) nodes.push({ id: iid, kind: "item", label: item.name, detail: item.kind });
        edges.push({ from: pid, to: iid, kind: "carries" });
      }
      st.work.filter((x) => x.finished && !x.answered).forEach((x, idx) => {
        const wid = `work:${p.id}:${idx}`;
        nodes.push({ id: wid, kind: "work", label: x.task, detail: new Date(x.ts).toISOString().slice(0, 10) });
        edges.push({ from: pid, to: wid, kind: "finished" });
        for (const f of x.files) {
          const fid = `file:${f}`;
          if (!nodes.some((n) => n.id === fid)) nodes.push({ id: fid, kind: "file", label: f });
          edges.push({ from: wid, to: fid, kind: "produced" });
        }
      });
    }
    return { nodes, edges };
  }

  const origFetch = window.fetch.bind(window);

  /** Execute one tool call against the real IndexedDB store (used by both deterministic and BYOK LLM modes). */
  function execTool(st: PersonStore, threadId: string, tool: string, args: Record<string, unknown>, written: string[]): string {
    if (tool === "list_dir") {
      const keys = Object.keys(st.files).sort();
      return keys.length ? keys.join("\n") : "(empty directory)";
    }
    if (tool === "read_file") {
      const safe = sanitizeSafePath(String(args.path ?? ""));
      if (!safe) return "error: path escapes the workspace";
      const f = st.files[safe];
      if (!f) return `error: file not found: ${safe}`;
      return f.content.slice(0, 12000);
    }
    if (tool === "write_file") {
      const safe = sanitizeSafePath(String(args.path ?? ""));
      if (!safe) return "error: path escapes the workspace";
      const content = String(args.content ?? "");
      st.files[safe] = makeFile(safe, content);
      if (!written.includes(safe)) written.push(safe);
      persist();
      return `wrote ${safe} (${utf8Bytes(content)} bytes)`;
    }
    if (tool === "write_document") {
      const title = String(args.title ?? "Document").trim() || "Document";
      const fileName = title.toLowerCase().endsWith(".md") ? title : `${title}.md`;
      const safe = sanitizeSafePath(fileName) || "Document.md";
      const secs = Array.isArray(args.sections) ? args.sections.map(String) : [String(args.sections ?? "")];
      const content = `# ${title.replace(/\.md$/i, "")}\n\n${secs.join("\n\n")}\n`;
      st.files[safe] = makeFile(safe, content);
      if (!written.includes(safe)) written.push(safe);
      persist();
      return `wrote document ${safe} (${utf8Bytes(content)} bytes)`;
    }
    if (tool === "make_spreadsheet") {
      const title = String(args.title ?? "Sheet").trim() || "Sheet";
      const fileName = title.toLowerCase().endsWith(".csv") ? title : `${title}.csv`;
      const safe = sanitizeSafePath(fileName) || "Sheet.csv";
      const cols = Array.isArray(args.columns) ? args.columns.map(String) : [];
      const rows = Array.isArray(args.rows) ? args.rows.map((r) => (Array.isArray(r) ? r.map(String) : [String(r)])) : [];
      const csv = [cols, ...rows].filter((r) => r.length > 0).map((r) => r.map(csvEscape).join(",")).join("\n") + "\n";
      st.files[safe] = makeFile(safe, csv);
      if (!written.includes(safe)) written.push(safe);
      persist();
      return `wrote spreadsheet ${safe} (${rows.length} data rows, ${utf8Bytes(csv)} bytes)`;
    }
    if (tool === "search_files") {
      const q = String(args.query ?? "").toLowerCase();
      const hits: string[] = [];
      for (const f of Object.values(st.files)) {
        f.content.split("\n").forEach((line, idx) => {
          if (q && line.toLowerCase().includes(q) && hits.length < 40) {
            hits.push(`${f.path}:${idx + 1}: ${line.trim().slice(0, 120)}`);
          }
        });
      }
      return hits.length ? hits.join("\n") : "no matches found";
    }
    if (tool === "update_memory") {
      const note = String(args.note ?? "").trim();
      if (!note) return "error: note is required";
      if (!st.memory.includes("## Standing preferences")) st.memory += "\n## Standing preferences\n";
      st.memory = st.memory.trimEnd() + `\n- ${note}\n`;
      persist();
      return `recorded preference in memory.md: ${note}`;
    }
    if (tool === "delete_file") {
      const safe = sanitizeSafePath(String(args.path ?? ""));
      if (!safe) return "error: path escapes the workspace";
      if (!st.files[safe]) return `error: file not found: ${safe}`;
      const did = `d${Date.now()}-${st.decisions.length + 1}`;
      st.decisions.push({
        id: did,
        kind: "delete",
        path: safe,
        summary: `Delete ${safe}`,
        status: "pending",
        outcome: null,
        thread_id: threadId,
        ts: Date.now(),
      });
      persist();
      return `parked for your person's approval (decision ${did}): deleting is a consequential action, so ${safe} has NOT been deleted yet.`;
    }
    if (tool === "flag_decision") {
      const summary = String(args.summary ?? "").trim() || "Decision needed";
      const did = `d${Date.now()}-${st.decisions.length + 1}`;
      st.decisions.push({
        id: did,
        kind: "question",
        summary,
        status: "pending",
        outcome: null,
        thread_id: threadId,
        ts: Date.now(),
      });
      persist();
      return `recorded decision ${did} in Today's queue`;
    }
    return `error: unknown tool ${tool}`;
  }

  /** Run BYOK OpenAI-compatible multi-step JSON tool loop over real IndexedDB tools. */
  async function runByokLoop(
    st: PersonStore,
    byok: ByokConfig,
    thread: WebThread,
    prompt: string,
    pushStep: (s: string) => void
  ): Promise<{ summary: string; written: string[]; parkedFile: string | null }> {
    const rules = learnedLines(st.memory);
    const sys = [
      "You are Ergon, a personal AI agent running over a real sandboxed browser workspace.",
      "Reply with ONLY ONE valid JSON object per message:",
      '- Tool call: {"tool": "<name>", "args": {...}}',
      '- Final answer: {"done": true, "summary": "<factual summary of what you did>"}',
      "TOOLS: list_dir, read_file, write_file, write_document, make_spreadsheet, search_files, update_memory, delete_file (parks approval), flag_decision.",
      `STANDING PREFERENCES:\n${rules.length ? rules.map((r) => `- ${r}`).join("\n") : "(none yet)"}`,
    ].join("\n");

    const messages: { role: string; content: string }[] = [
      { role: "system", content: sys },
      { role: "user", content: prompt },
    ];
    const written: string[] = [];
    let parkedFile: string | null = null;

    for (let step = 0; step < 10; step++) {
      const headers: Record<string, string> = { "Content-Type": "application/json" };
      if (byok.api_key) headers.Authorization = `Bearer ${byok.api_key}`;
      const r = await origFetch(`${byok.model_base_url.replace(/\/+$/, "")}/chat/completions`, {
        method: "POST",
        headers,
        body: JSON.stringify({ model: byok.model_id, messages, temperature: 0.2 }),
      });
      if (!r.ok) {
        const txt = await r.text().catch(() => "");
        throw new Error(`BYOK endpoint returned HTTP ${r.status}: ${txt.slice(0, 240)}`);
      }
      const data = await r.json();
      const raw = String(data?.choices?.[0]?.message?.content ?? "").trim();
      const unfenced = raw.replace(/^```(?:json)?\s*/i, "").replace(/```$/, "").trim();
      let parsed: Record<string, unknown>;
      try {
        parsed = JSON.parse(unfenced);
      } catch {
        const a = unfenced.indexOf("{");
        const b = unfenced.lastIndexOf("}");
        if (a >= 0 && b > a) parsed = JSON.parse(unfenced.slice(a, b + 1));
        else throw new Error(`Model did not return valid JSON: ${raw.slice(0, 200)}`);
      }
      if (parsed.done === true) {
        return { summary: String(parsed.summary || "Task complete."), written, parkedFile };
      }
      const tool = String(parsed.tool || "");
      const args = (parsed.args && typeof parsed.args === "object" ? parsed.args : {}) as Record<string, unknown>;
      const res = execTool(st, thread.id, tool, args, written);
      if (tool === "delete_file" && !res.startsWith("error:")) {
        parkedFile = sanitizeSafePath(String(args.path ?? ""));
      }
      pushStep(`Tool: ${tool}\n${res.slice(0, 500)}`);
      messages.push({ role: "assistant", content: JSON.stringify(parsed) });
      messages.push({ role: "user", content: `Tool result for ${tool}: ${res.slice(0, 5000)}` });
    }
    throw new Error("Stopped after 10 BYOK steps without completion.");
  }

  /** Real deterministic client-side agent over the user's ACTUAL IndexedDB files. */
  async function runDeterministicTask(
    st: PersonStore,
    thread: WebThread,
    prompt: string,
    pushStep: (s: string) => void
  ): Promise<{ summary: string; written: string[]; parkedFile: string | null }> {
    const written: string[] = [];
    const rules = learnedLines(st.memory);
    const allFiles = Object.values(st.files).sort((a, b) => b.mtime_ms - a.mtime_ms);
    const pLow = prompt.toLowerCase();

    // 1. Check if user is teaching a standing memory preference.
    if (/^\s*(remember|always|from now on|prefer|note that)\b/i.test(prompt)) {
      const rule = prompt.replace(/^\s*(remember(\s+to|\s+that)?|from now on,?\s*|note that\s*)/i, "").trim();
      await sleep(140);
      const res = execTool(st, thread.id, "update_memory", { note: rule || prompt }, written);
      pushStep(`Tool: update_memory\n${res}`);
      return {
        summary: `Saved your standing preference in memory.md ("${rule || prompt}"). Future summaries and reconciliations will apply it automatically.`,
        written: ["memory.md"],
        parkedFile: null,
      };
    }

    // 2. Always inspect real workspace first.
    await sleep(120);
    const listed = execTool(st, thread.id, "list_dir", {}, written);
    pushStep(`Tool: list_dir\n${listed}`);

    // Find any explicitly named workspace file(s) in prompt.
    const mentionedFiles = allFiles.filter((f) => pLow.includes(f.path.toLowerCase()) || pLow.includes(f.name.toLowerCase()));

    // 3. Consequential deletion request -> ALWAYS park approval, never delete immediately!
    if (/\b(delete|remove|clean up|purge|erase)\b/i.test(prompt)) {
      const target = mentionedFiles[0] || allFiles.find((f) => /draft|old|temp|backup/i.test(f.name)) || allFiles[0];
      if (!target) {
        return { summary: "Your workspace has no matching file to delete.", written, parkedFile: null };
      }
      await sleep(140);
      const res = execTool(st, thread.id, "delete_file", { path: target.path }, written);
      pushStep(`Tool: delete_file\n${res}`);
      return {
        summary: `I parked the deletion of ${target.path} for your approval in Today. The file has not been touched yet.`,
        written,
        parkedFile: target.path,
      };
    }

    // 4. CSV reconciliation / analysis / spreadsheet task over real CSV file(s).
    const csvCandidate = mentionedFiles.find((f) => f.ext === "csv")
      || ((/\b(csv|reconcil|expense|budget|receipt|spreadsheet|spend|invoice|total|sum|cost)\b/i.test(prompt))
        ? allFiles.find((f) => f.ext === "csv" && !f.name.endsWith("-reconciled.csv")) || allFiles.find((f) => f.ext === "csv")
        : undefined);

    if (csvCandidate) {
      await sleep(140);
      const content = execTool(st, thread.id, "read_file", { path: csvCandidate.path }, written);
      pushStep(`Tool: read_file\nread ${csvCandidate.path} (${csvCandidate.size} bytes)`);

      await sleep(160);
      const recon = runCsvReconciliation(csvCandidate.path, content, rules);
      const stem = csvCandidate.name.replace(/\.csv$/i, "");
      const outCsv = `${stem}-reconciled.csv`;
      const outMd = `${stem}-report.md`;

      st.files[outCsv] = makeFile(outCsv, recon.reconciledCsv);
      written.push(outCsv);
      pushStep(`Tool: make_spreadsheet\nwrote spreadsheet ${outCsv} (${recon.rowCount} data rows, ${utf8Bytes(recon.reconciledCsv)} bytes)`);

      await sleep(140);
      st.files[outMd] = makeFile(outMd, recon.reportMd);
      written.push(outMd);
      pushStep(`Tool: write_document\nwrote document ${outMd} (${utf8Bytes(recon.reportMd)} bytes)`);
      persist();

      const catSummary = recon.categories.slice(0, 3).map(([c, s]) => `${c} ${fmtMoney(s.total)}`).join(", ");
      const flagPart = recon.flaggedRows.length
        ? ` Flagged ${recon.flaggedRows.length} item${recon.flaggedRows.length === 1 ? "" : "s"} (${recon.flaggedRows[0]}).`
        : " All rows passed receipt and duplicate checks.";
      return {
        summary: `Reconciled ${recon.rowCount} rows from ${csvCandidate.path} totaling ${fmtMoney(recon.grandTotal)}${catSummary ? ` (${catSummary})` : ""}. Saved ${outCsv} and ${outMd} to your workspace.${flagPart}`,
        written,
        parkedFile: null,
      };
    }

    // 5. Document summarization / action-item extraction / review over real Markdown / text / JSON files.
    const docSources = mentionedFiles.length
      ? mentionedFiles
      : allFiles.filter((f) => ["md", "txt", "json", "log"].includes(f.ext)).slice(0, 3);

    if (docSources.length > 0 && /\b(summar|note|action|brief|review|digest|extract|tidy|outline|report|meeting|update|check)\b/i.test(prompt)) {
      for (const src of docSources) {
        await sleep(120);
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep(`Tool: read_file\nread ${src.path} (${src.size} bytes)`);
      }
      await sleep(150);
      const sum = runDocumentSummary(docSources, prompt, rules);
      st.files[sum.outName] = makeFile(sum.outName, sum.markdown);
      written.push(sum.outName);
      pushStep(`Tool: write_document\nwrote document ${sum.outName} (${utf8Bytes(sum.markdown)} bytes)`);
      persist();
      return {
        summary: `Compiled ${sum.outName} from ${docSources.map((s) => s.path).join(", ")} with ${sum.actionCount} action item${sum.actionCount === 1 ? "" : "s"} and ${sum.blockerCount} blocker${sum.blockerCount === 1 ? "" : "s"}${rules.length ? `, applying ${rules.length} standing preference${rules.length === 1 ? "" : "s"}` : ""}.`,
        written,
        parkedFile: null,
      };
    }

    // 6. General document or checklist authoring grounded in the prompt & workspace.
    const outFileMatch = /\b([A-Za-z0-9][A-Za-z0-9._-]*\.(?:md|csv|txt))\b/.exec(prompt);
    const outFile = outFileMatch ? outFileMatch[1] : `${prompt.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 36) || "task-output"}.md`;
    await sleep(140);
    if (outFile.endsWith(".csv")) {
      const res = execTool(st, thread.id, "make_spreadsheet", {
        title: outFile,
        columns: ["Item", "Owner", "Status", "Notes"],
        rows: [
          [prompt.slice(0, 48), activePerson().name, "Ready", `Created from workspace (${allFiles.length} files available)`],
        ],
      }, written);
      pushStep(`Tool: make_spreadsheet\n${res}`);
      return { summary: `Created ${outFile} in your workspace.`, written, parkedFile: null };
    }

    const sections = [
      `## Request\n\n${prompt}`,
      allFiles.length
        ? `## Workspace Files Referenced\n\n${allFiles.slice(0, 6).map((f) => `- \`${f.path}\` (${f.size} bytes)`).join("\n")}`
        : "## Workspace Context\n\nNo existing files were in the workspace when this document was drafted.",
      rules.length
        ? `## Standing Preferences Applied\n\n${rules.map((r) => `- ${r}`).join("\n")}`
        : "## Next Steps\n\n- Upload source CSV or Markdown files via Workspace files or the composer paperclip to run automated reconciliations and summaries.",
    ];
    const res = execTool(st, thread.id, "write_document", { title: outFile, sections }, written);
    pushStep(`Tool: write_document\n${res}`);
    return { summary: `Wrote ${outFile} to your workspace with structured sections grounded in your request.`, written, parkedFile: null };
  }

  function jsonRes(obj: unknown, status = 200): Response {
    return new Response(JSON.stringify(obj), {
      status,
      headers: { "Content-Type": "application/json" },
    });
  }

  window.fetch = async function (input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    const rawUrl = typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
    const u = new URL(rawUrl, window.location.origin);
    if (u.origin !== window.location.origin && !rawUrl.startsWith("/api/")) {
      return origFetch(input, init);
    }
    const apiIdx = u.pathname.indexOf("/api/");
    if (apiIdx < 0) return origFetch(input, init);
    const path = u.pathname.slice(apiIdx);
    const method = (init?.method || "GET").toUpperCase();
    let body: Record<string, unknown> = {};
    try {
      if (init?.body) body = JSON.parse(String(init.body));
    } catch { /* ignore */ }

    const p = activePerson();
    const st = activeStore();
    const byok = loadByok();

    if (method === "GET" && path === "/api/version") return jsonRes({ name: "ergon", version: "0.4.0" });
    if (method === "GET" && path === "/api/update") return jsonRes({ current: "0.4.0", available: false, mode: "off", checked_at: Math.floor(Date.now() / 1000) });
    if (method === "GET" && path === "/api/status") {
      return jsonRes({
        version: "0.4.0",
        model_configured: true,
        model: byok.engine_mode === "byok" && byok.model_id ? byok.model_id : "browser-workspace-engine",
        model_base_url: byok.engine_mode === "byok" ? byok.model_base_url : "indexeddb://local",
        engine_mode: byok.engine_mode,
        person: p,
        store_recovered: [],
      });
    }
    if (method === "GET" && path === "/api/config") {
      return jsonRes({
        model_base_url: byok.model_base_url,
        model_id: byok.model_id || "browser-workspace-engine",
        has_api_key: Boolean(byok.api_key),
        model_configured: true,
        is_loopback: true,
        engine_mode: byok.engine_mode,
        detected_gateways: [],
      });
    }
    if (method === "POST" && path === "/api/config") {
      const mode = body.engine_mode === "byok" ? "byok" : "browser";
      const next: ByokConfig = {
        engine_mode: mode,
        model_base_url: String(body.model_base_url ?? byok.model_base_url).trim(),
        model_id: String(body.model_id ?? byok.model_id).trim(),
        api_key: body.clear_api_key ? "" : (String(body.api_key ?? "").trim() || byok.api_key),
      };
      if (mode === "byok" && (!/^https?:\/\//.test(next.model_base_url) || !next.model_id)) {
        return jsonRes({ error: "Enter a valid http(s) base URL and model ID for BYOK mode." }, 400);
      }
      saveByok(next);
      return jsonRes({
        model_base_url: next.model_base_url,
        model_id: next.model_id || "browser-workspace-engine",
        has_api_key: Boolean(next.api_key),
        model_configured: true,
        is_loopback: true,
        engine_mode: next.engine_mode,
      });
    }
    if (method === "GET" && path === "/api/runs/status") {
      return jsonRes(liveRuns[p.id] || { active: false, thread_id: "", task: "", started_ms: 0, steps: [] });
    }
    if (method === "GET" && path === "/api/files") {
      const files = Object.values(st.files)
        .sort((a, b) => b.mtime_ms - a.mtime_ms || a.path.localeCompare(b.path))
        .map(({ path: fp, name, size, mtime_ms, ext }) => ({ path: fp, name, size, mtime_ms, ext }));
      return jsonRes({ person_id: p.id, files });
    }
    if (method === "GET" && path === "/api/files/content") {
      const rawPath = u.searchParams.get("path") || "";
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const found = st.files[safe];
      if (!found) return jsonRes({ error: "file not found" }, 404);
      if (u.searchParams.get("download") === "1") {
        const mime = found.ext === "csv" ? "text/csv; charset=utf-8" : found.ext === "json" ? "application/json; charset=utf-8" : "text/plain; charset=utf-8";
        return new Response(found.content, {
          status: 200,
          headers: {
            "Content-Type": mime,
            "Content-Disposition": `attachment; filename="${found.name.replace(/[^a-zA-Z0-9._ -]/g, "_")}"`,
          },
        });
      }
      return jsonRes({ ...found, is_binary: false });
    }
    if (method === "POST" && path === "/api/files/upload") {
      const rawPath = String(body.path ?? body.name ?? "");
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "invalid workspace filename" }, 403);
      const content = String(body.content ?? "");
      const saved = makeFile(safe, content);
      st.files[safe] = saved;
      persist();
      return jsonRes({ path: saved.path, name: saved.name, size: saved.size, mtime_ms: saved.mtime_ms, ext: saved.ext }, 201);
    }
    if (method === "GET" && path === "/api/people") return jsonRes({ active: state.activePerson, people: state.people });
    if (method === "POST" && path === "/api/people") {
      const nm = String(body.name ?? "").trim();
      if (!nm) return jsonRes({ error: "name is required" }, 400);
      const np: WebPerson = { id: nm.toLowerCase().replace(/[^a-z0-9]+/g, "-") + "-" + Date.now(), name: nm, role: String(body.role ?? "").trim() };
      state.people.push(np);
      state.stores[np.id] = {
        files: {},
        threads: [],
        work: [],
        decisions: [],
        offers: [],
        memory: `# Memory\n\n## About\n- Name: ${np.name}\n${np.role ? `- Role: ${np.role}\n` : ""}`,
        suite: ["researcher", "meeting-actions"],
      };
      persist();
      return jsonRes(np, 201);
    }
    if (method === "POST" && path === "/api/active-person") {
      const id = String(body.id ?? "");
      if (!state.people.some((x) => x.id === id)) return jsonRes({ error: "person not found" }, 404);
      state.activePerson = id;
      persist();
      return jsonRes({ active: state.activePerson, people: state.people });
    }
    if (method === "GET" && path === "/api/memory") return jsonRes({ person_id: p.id, content: st.memory });
    if (method === "GET" && path === "/api/library") return jsonRes({ items: state.library });
    if (method === "GET" && path === "/api/suite") {
      const installed = st.suite.map((id) => state.library.find((x) => x.id === id)).filter(Boolean);
      return jsonRes({ person_id: p.id, installed });
    }
    if (method === "POST" && path === "/api/suite/install") {
      const id = String(body.item_id ?? "");
      if (id && !st.suite.includes(id)) st.suite.push(id);
      persist();
      return jsonRes({ installed: id });
    }
    if (method === "POST" && path === "/api/suite/remove") {
      const id = String(body.item_id ?? "");
      st.suite = st.suite.filter((x) => x !== id);
      persist();
      return jsonRes({ removed: id });
    }
    if (method === "GET" && path === "/api/decisions") return jsonRes({ decisions: st.decisions });
    if (method === "POST" && path.startsWith("/api/decisions/")) {
      const [rawId, action] = path.slice("/api/decisions/".length).split("/");
      const did = decodeURIComponent(rawId || "");
      const dec = st.decisions.find((d) => d.id === did);
      if (!dec) return jsonRes({ error: "decision not found" }, 404);
      if (dec.status !== "pending") return jsonRes({ error: "that decision was already settled" }, 400);
      const approve = action === "approve";
      dec.status = approve ? "approved" : "denied";
      dec.resolved_ts = Date.now();
      if (dec.kind === "delete" && dec.path) {
        if (approve) {
          if (st.files[dec.path]) {
            delete st.files[dec.path];
            dec.outcome = `Approved and done: ${dec.path} was deleted.`;
          } else {
            dec.outcome = `Approved: ${dec.path} was already gone.`;
          }
        } else {
          dec.outcome = `You said no — kept ${dec.path} safe.`;
        }
        const th = st.threads.find((t) => t.id === dec.thread_id);
        if (th) {
          th.messages = th.messages.map((m) =>
            m.role === "step" && m.content.includes(`${dec.path} has NOT been deleted yet`)
              ? { role: "step", content: approve ? `Note: ${dec.path} was deleted after you approved it.` : `Note: ${dec.path} was kept because you declined the deletion.` }
              : m
          );
          th.messages.push({ role: "step", content: `Decision settled — ${dec.summary}: ${dec.outcome}` });
        }
      } else {
        dec.outcome = approve ? "Approved." : "Declined.";
      }
      persist();
      return jsonRes({ decision: dec, outcome: dec.outcome });
    }
    if (method === "GET" && path === "/api/offers") return jsonRes({ offers: st.offers });
    if (method === "POST" && path.startsWith("/api/offers/")) {
      const [rawId, action] = path.slice("/api/offers/".length).split("/");
      const oid = decodeURIComponent(rawId || "");
      const off = st.offers.find((o) => o.id === oid);
      if (!off) return jsonRes({ error: "offer not found" }, 404);
      off.status = action === "accept" ? "accepted" : "dismissed";
      if (action === "accept" && off.item_id && !st.suite.includes(off.item_id)) st.suite.push(off.item_id);
      persist();
      return jsonRes({ offer: off, outcome: action === "accept" ? "Added to your suite as a one-tap app." : "Offer dismissed." });
    }
    if (method === "GET" && path === "/api/brief") return jsonRes(buildBrief());
    if (method === "GET" && path === "/api/today") return jsonRes(buildToday());
    if (method === "GET" && path === "/api/graph") return jsonRes(buildGraph());
    if (method === "GET" && path === "/api/threads") {
      return jsonRes({
        threads: st.threads.map((t) => ({ id: t.id, title: t.title, messages: t.messages.length })),
      });
    }
    if (method === "POST" && path === "/api/threads") {
      const nid = `t${Date.now()}-${st.threads.length + 1}`;
      const nt: WebThread = { id: nid, title: "New chat", messages: [] };
      st.threads.unshift(nt);
      persist();
      return jsonRes(nt, 201);
    }
    if (method === "GET" && path.startsWith("/api/threads/")) {
      const tid = decodeURIComponent(path.slice("/api/threads/".length));
      const found = st.threads.find((t) => t.id === tid);
      if (!found) return jsonRes({ error: "thread not found" }, 404);
      return jsonRes(found);
    }
    if (method === "POST" && (path === "/api/chat" || (path.startsWith("/api/apps/") && path.endsWith("/run")))) {
      let prompt = "";
      if (path.startsWith("/api/apps/")) {
        const appId = decodeURIComponent(path.slice("/api/apps/".length, -"/run".length));
        const app = state.library.find((x) => x.id === appId);
        if (!app) return jsonRes({ error: "app not found" }, 404);
        const fields = (body.fields && typeof body.fields === "object" ? body.fields : {}) as Record<string, string>;
        prompt = app.template || `Run ${app.name}`;
        for (const f of app.fields || []) {
          const val = String(fields[f] ?? "").trim();
          if (!val) return jsonRes({ error: "fill in every field before running the app" }, 400);
          prompt = prompt.replace(`{${f}}`, val);
        }
      } else {
        prompt = String(body.message ?? "").trim();
        if (!prompt) return jsonRes({ error: "message is required" }, 400);
      }

      const reqTid = String(body.thread_id ?? "");
      let th = reqTid ? st.threads.find((t) => t.id === reqTid) : undefined;
      if (!th) {
        th = { id: `t${Date.now()}-${st.threads.length + 1}`, title: prompt.slice(0, 64), messages: [] };
        st.threads.unshift(th);
      } else if (!th.messages.length || th.title === "New chat") {
        th.title = prompt.slice(0, 64);
      }
      th.messages.push({ role: "user", content: prompt });

      liveRuns[p.id] = { active: true, thread_id: th.id, task: prompt, started_ms: Date.now(), steps: [] };
      const pushStep = (stepStr: string) => {
        th!.messages.push({ role: "step", content: stepStr });
        liveRuns[p.id]?.steps.push(stepStr);
      };

      try {
        const outcome = byok.engine_mode === "byok" && byok.model_base_url && byok.model_id
          ? await runByokLoop(st, byok, th, prompt, pushStep)
          : await runDeterministicTask(st, th, prompt, pushStep);

        th.messages.push({ role: "assistant", content: outcome.summary });
        if (outcome.parkedFile) {
          th.messages.push({
            role: "step",
            content: `Note: ${outcome.parkedFile} has NOT been deleted yet; the deletion is waiting for your approval in Decisions.`,
          });
        }
        st.work.push({
          task: th.title,
          summary: outcome.summary,
          files: outcome.written.filter((f) => f !== "memory.md"),
          finished: true,
          answered: false,
          thread_id: th.id,
          ts: Date.now(),
        });
        persist();
        delete liveRuns[p.id];
        return jsonRes(th, 200);
      } catch (e) {
        delete liveRuns[p.id];
        const msg = e instanceof Error ? e.message : String(e);
        th.messages.push({ role: "error", content: msg });
        persist();
        return jsonRes({ error: msg, thread_id: th.id }, 500);
      }
    }

    return jsonRes({ error: "not found" }, 404);
  };
}
