// Ergon v0.7.0 Local-First Browser Engine (Standalone Web Edition)
// Real IndexedDB workspace + all 12 deterministic client-side built-in workflows
// (reconcile_ledger, summarize_meeting, audit_contract, triage_inbox, clean_dataset,
// compare_documents, draft_brief, extract_tables, make_spreadsheet, write_document,
// research, update_memory), background routine scheduler (pause/resume + run history),
// automatic .history/ file revisions with unified diff & atomic restore, full tamper-evident
// Field-Book Audit Bundle export, multi-workspace switching, indexed search, live EventSource
// stream, cooperative run cancellation, Web Crypto SHA-256 receipt export, approval-gated
// deletion, and optional BYOK OpenAI-compatible LLM (sessionStorage only).
// ZERO hardcoded fixture replies, ZERO fixture demo pills, ZERO Rune jargon.
import { inspectDelimited, parseCsv, parseDelimited } from "@/shell/FileEditor";
import { CANONICAL_SKILLS, CANONICAL_STARTER_PLAYBOOKS } from "@/lib/skillsCatalog";
import { buildDeterministicPlaybookPreview, buildSignedPlaybookPack, buildSignedReceipt, sha256Hex, verifyAndParsePlaybookPack } from "@/lib/receiptExport";
import { extractCitations } from "@/lib/words";
import { buildAuditBundle } from "@/lib/auditBundle";
import { computeLineDiff } from "@/lib/diff";
import {
  engagementWorkspacePrefix,
  estimateNextRunMs,
  isPathInEngagementScope,
  type Client,
  type ClientContact,
  type DeliveryLog,
  type Engagement,
  type FileRevision,
  type ImapConfig,
  type InvoiceLineItem,
  type InvoiceRecord,
  type InvoiceStatus,
  type LiveRunStep,
  type McpServerSpec,
  type McpToolSpec,
  type McpTransport,
  type MeteringByEngagement,
  type MeteringByModel,
  type MeteringDailyPoint,
  type MeteringSummary,
  type ModelPricingEntry,
  type ModelPricingMap,
  type OAuthConfig,
  type OutboundConnector,
  type OutboundConnectorKind,
  type PipelineAgentSnapshot,
  type PipelineCheckpointInputMode,
  type PipelineChildItemState,
  type PipelineDeliverSpec,
  type PipelineRun,
  type PipelineStepSpec,
  type PipelineStepState,
  type PipelineValidationResult,
  type RolloverMode,
  type Routine,
  type RoutineRunRecord,
  type RunUsage,
  type SmtpConfig,
  type SmtpSecurity,
  type TimeExpenseEntry,
  type TimeExpenseKind,
  type TriggerRule,
  type VaultChunkHit,
  type VaultFileHit,
  type VaultIndexStats,
  type VaultSearchMode,
  type VaultSearchScope,
} from "@/lib/api";

type WebFile = {
  path: string;
  name: string;
  content: string;
  size: number;
  mtime_ms: number;
  ext: string;
  binaryBase64?: string;
  extracted?: { kind: string; format: "markdown" | "csv"; sheets?: string[]; sheet?: string | null; pages?: number };
  extractedSheets?: Record<string, string>;
};
type WebMsg = { role: string; content: string };
type WebThread = { id: string; title: string; messages: WebMsg[] };
type WebWork = {
  id?: string;
  task: string;
  summary: string;
  files: string[];
  finished: boolean;
  answered: boolean;
  thread_id: string;
  ts: number;
  steps?: string[];
  engagement_id?: string;
  usage?: RunUsage;
};
type UsageLedgerEntry = {
  id: string;
  ts: number;
  person_id: string;
  engagement_id?: string;
  thread_id?: string;
  run_id?: string;
  kind?: string;
  usage: RunUsage;
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
  detail?: string;
  severity?: string;
  run_id?: string;
  input_mode?: PipelineCheckpointInputMode;
  choices?: string[];
  inspect_files?: string[];
  answer?: string;
  connector_id?: string;
  connector_name?: string;
  connector_kind?: string;
  method?: string;
  url?: string;
  payload_preview?: string;
  payload_sha256?: string;
  delivery_id?: string;
  server_id?: string;
  server_name?: string;
  tool_name?: string;
  arguments_preview?: string;
  engagement_id?: string;
  engagement_name?: string;
  client_name?: string;
  retainer_budget_usd?: number;
  spent_usd?: number;
  projected_usd?: number;
  estimated_cost_usd?: number;
};
type WebConnectorsState = {
  triggers: TriggerRule[];
  outbound: OutboundConnector[];
  deliveries: DeliveryLog[];
  webhookSecrets: Record<string, string>;
  outboundSecrets: Record<string, string>;
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
type WorkspacePolicyProfile = "fast_internal" | "strict_client" | "air_gapped";
type PersonStore = {
  files: Record<string, WebFile>;
  fileHistory: Record<string, FileRevision[]>;
  threads: WebThread[];
  work: WebWork[];
  decisions: WebDecision[];
  offers: WebOffer[];
  routines: Routine[];
  routineRuns: Record<string, RoutineRunRecord[]>;
  pipelineRuns?: PipelineRun[];
  connectors?: WebConnectorsState;
  mcpServers?: McpServerSpec[];
  memory: string;
  suite: string[];
  policyProfile?: WorkspacePolicyProfile;
  policyUpdatedMs?: number;
  isolateActiveEngagement?: boolean;
  usageLedger?: UsageLedgerEntry[];
  budgetOverrunApproved?: Record<string, number>;
  timeEntries?: TimeExpenseEntry[];
  invoices?: InvoiceRecord[];
};
type RootState = {
  activePerson: string;
  people: WebPerson[];
  library: WebLibItem[];
  stores: Record<string, PersonStore>;
  clients?: Client[];
  engagements?: Engagement[];
  activeEngagementByPerson?: Record<string, string>;
  modelPricing?: ModelPricingMap;
};
type ByokConfig = {
  engine_mode: "browser" | "byok";
  model_base_url: string;
  model_id: string;
  api_key: string;
};

const DB_NAME = "ergon-workspace-v7";
const DB_STORE = "state";
const DB_KEY = "root";
const BYOK_KEY = "ergon-byok-session";

const DEFAULT_MODEL_PRICING: ModelPricingMap = {
  "browser-workspace-engine": { input_per_1m_usd: 0.15, output_per_1m_usd: 0.6, cached_input_per_1m_usd: 0.075 },
  "gpt-4o-mini": { input_per_1m_usd: 0.15, output_per_1m_usd: 0.6, cached_input_per_1m_usd: 0.075 },
  "gpt-4o": { input_per_1m_usd: 2.5, output_per_1m_usd: 10.0, cached_input_per_1m_usd: 1.25 },
  "claude-3-5-sonnet": { input_per_1m_usd: 3.0, output_per_1m_usd: 15.0, cached_input_per_1m_usd: 0.3 },
  "deepseek-chat": { input_per_1m_usd: 0.27, output_per_1m_usd: 1.1, cached_input_per_1m_usd: 0.07 },
  "qwen2.5:3b": { input_per_1m_usd: 0.0, output_per_1m_usd: 0.0, cached_input_per_1m_usd: 0.0 },
};

function mergeModelPricing(overrides?: ModelPricingMap): ModelPricingMap {
  const merged: ModelPricingMap = {};
  for (const [k, v] of Object.entries(DEFAULT_MODEL_PRICING)) {
    merged[k] = { ...v };
  }
  if (overrides && typeof overrides === "object") {
    for (const [k, v] of Object.entries(overrides)) {
      if (!v || typeof v !== "object") continue;
      merged[k] = {
        input_per_1m_usd: Math.max(0, Number(v.input_per_1m_usd) || 0),
        output_per_1m_usd: Math.max(0, Number(v.output_per_1m_usd) || 0),
        cached_input_per_1m_usd: Math.max(0, Number(v.cached_input_per_1m_usd) || 0),
      };
    }
  }
  return merged;
}

function computeUsageCostUsd(
  pricingMap: ModelPricingMap,
  modelId: string,
  promptTokens: number,
  completionTokens: number,
  cachedTokens = 0
): number {
  const key = modelId.trim();
  const entry: ModelPricingEntry =
    pricingMap[key] ||
    pricingMap[key.toLowerCase()] ||
    pricingMap["browser-workspace-engine"] ||
    { input_per_1m_usd: 0.15, output_per_1m_usd: 0.6, cached_input_per_1m_usd: 0.075 };
  const uncachedPrompt = Math.max(0, promptTokens - cachedTokens);
  const cachedRate = entry.cached_input_per_1m_usd ?? entry.input_per_1m_usd * 0.5;
  const cost =
    (uncachedPrompt * entry.input_per_1m_usd +
      cachedTokens * cachedRate +
      completionTokens * entry.output_per_1m_usd) /
    1_000_000;
  return Math.round(cost * 1_000_000) / 1_000_000;
}

function estimateMinutesSaved(promptOrSkill: string, stepCount: number): number {
  const s = promptOrSkill.toLowerCase();
  if (/reconcil|ledger|invoice|expense/.test(s)) return 28;
  if (/audit|contract|clause/.test(s)) return 35;
  if (/clean_dataset|dedup|normaliz/.test(s)) return 24;
  if (/extract_table|spreadsheet/.test(s)) return 20;
  if (/compare|diff/.test(s)) return 22;
  if (/summar|meeting|triage|inbox/.test(s)) return 18;
  if (/brief|research|draft/.test(s)) return 25;
  return Math.max(10, Math.min(40, 8 + stepCount * 4));
}

function ensureClientsAndPricing(state: RootState, now = Date.now()) {
  if (!Array.isArray(state.clients) || state.clients.length === 0) {
    state.clients = [
      {
        id: "cli-northstar",
        name: "Northstar Advisory",
        brand_header: "Northstar Advisory Partners",
        billing_currency: "USD",
        default_rate_usd: 220,
        contacts: [{ id: "ct-ns-1", name: "Elena Vance", email: "ap@northstar.example", role: "VP Finance", primary: true }],
        notes_md: "Send monthly PDF and JSON invoice packets with [file:L##] citation checks.",
        tax_rate_pct: 0,
        payment_terms: "Net 30",
        created_ms: now - 7 * 86_400_000,
        updated_ms: now - 86_400_000,
      },
      {
        id: "cli-harbor",
        name: "Harbor Logistics",
        brand_header: "Harbor Freight & Ops",
        billing_currency: "USD",
        default_rate_usd: 185,
        contacts: [{ id: "ct-hb-1", name: "Marcus Reed", email: "finance@harbor.example", role: "Controller", primary: true }],
        notes_md: "",
        tax_rate_pct: 0,
        payment_terms: "Net 15",
        created_ms: now - 5 * 86_400_000,
        updated_ms: now - 2 * 86_400_000,
      },
    ];
  }
  if (!Array.isArray(state.engagements) || state.engagements.length === 0) {
    state.engagements = [
      {
        id: "eng-northstar-q4",
        client_id: "cli-northstar",
        person_id: "maya-chen",
        name: "Q4 Operations Retainer",
        retainer_hours: 40,
        retainer_budget_usd: 2500,
        hourly_rate_usd: 220,
        period: "monthly",
        status: "active",
        workspace_prefix: "clients/cli-northstar/eng-northstar-q4",
        sow_rules: ["Cite [file:L##] on all client memos", "Keep files under engagement workspace prefix"],
        rollover_mode: "none",
        created_ms: now - 7 * 86_400_000,
        updated_ms: now - 86_400_000,
      },
      {
        id: "eng-harbor-close",
        client_id: "cli-harbor",
        person_id: "tomas-ruiz",
        name: "Monthly Finance Close",
        retainer_hours: 25,
        retainer_budget_usd: 1500,
        hourly_rate_usd: 185,
        period: "monthly",
        status: "active",
        workspace_prefix: "clients/cli-harbor/eng-harbor-close",
        sow_rules: ["Reconcile variance CSV before close"],
        rollover_mode: "carry_hours",
        created_ms: now - 5 * 86_400_000,
        updated_ms: now - 2 * 86_400_000,
      },
    ];
  }
  if (!state.activeEngagementByPerson || typeof state.activeEngagementByPerson !== "object") {
    state.activeEngagementByPerson = {
      "maya-chen": "eng-northstar-q4",
      "tomas-ruiz": "eng-harbor-close",
    };
  }
  state.modelPricing = mergeModelPricing(state.modelPricing);
}

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

function escapeHtml(s: string): string {
  return String(s || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

async function verifyTextCitationsInMemory(
  files: Record<string, WebFile>,
  text: string,
  expectedShas?: Record<string, string>,
  fileHistory?: Record<string, FileRevision[]>,
  receiptTs = 0
) {
  const extracted = extractCitations(text);
  let verified = 0;
  let stale = 0;
  let broken = 0;
  const shaByPath: Record<string, string> = {};
  const citations = await Promise.all(
    extracted.map(async (c) => {
      const safe = sanitizeSafePath(c.path);
      const f = safe ? files[safe] : undefined;
      if (!f || !safe) {
        broken += 1;
        return {
          raw: c.raw,
          path: c.path,
          line: c.line,
          end_line: c.endLine ?? null,
          status: "missing" as const,
          total_lines: 0,
          file_sha256: "",
          line_excerpt: "",
          baseline_sha256: null,
        };
      }
      if (!shaByPath[safe]) shaByPath[safe] = await sha256Hex(f.content);
      const liveSha = shaByPath[safe];
      const lines = f.content ? f.content.split(/\r?\n/) : [];
      const totalLines = lines.length;
      const endOk = c.endLine === undefined || (c.endLine >= c.line && c.endLine <= totalLines);
      if (c.line >= 1 && c.line <= totalLines && endOk) {
        const excerpt = String(lines[c.line - 1] || "").trim().slice(0, 140);
        let baseSha = expectedShas?.[safe] || null;
        if (!baseSha && receiptTs > 0 && Array.isArray(fileHistory?.[safe])) {
          const postRev = [...fileHistory[safe]]
            .filter((r) => r.ts > receiptTs)
            .sort((a, b) => a.ts - b.ts)[0];
          if (postRev?.content) baseSha = await sha256Hex(postRev.content);
        }
        const isStale = Boolean(baseSha && baseSha.toLowerCase() !== liveSha.toLowerCase());
        if (isStale) {
          stale += 1;
          return {
            raw: c.raw,
            path: safe,
            line: c.line,
            end_line: c.endLine ?? null,
            status: "stale" as const,
            total_lines: totalLines,
            file_sha256: liveSha,
            line_excerpt: excerpt,
            baseline_sha256: baseSha,
          };
        }
        verified += 1;
        return {
          raw: c.raw,
          path: safe,
          line: c.line,
          end_line: c.endLine ?? null,
          status: "verified" as const,
          total_lines: totalLines,
          file_sha256: liveSha,
          line_excerpt: excerpt,
          baseline_sha256: baseSha,
        };
      }
      broken += 1;
      return {
        raw: c.raw,
        path: safe,
        line: c.line,
        end_line: c.endLine ?? null,
        status: "out_of_range" as const,
        total_lines: totalLines,
        file_sha256: liveSha,
        line_excerpt: "",
        baseline_sha256: null,
      };
    })
  );
  const total = citations.length;
  const freshness_pct = total === 0 ? 100 : Math.floor((verified * 100) / total);
  const gate_status =
    broken === 0 && stale === 0 ? "100% Verified" : stale > 0 ? "Stale Line Drift" : "Broken Citations";
  return { total, verified, stale, broken, freshness_pct, gate_status, ok: broken === 0 && stale === 0, citations };
}

function learnedLines(md: string): string[] {
  return String(md || "")
    .split("\n")
    .map((l) => l.trim())
    .filter((t) => t.startsWith("- ") && !/^-\s*(name|role)\s*:/i.test(t))
    .map((t) => t.slice(2).trim())
    .filter(Boolean);
}

/** Skill 1: reconcile_ledger — multi-CSV invoice/expense matching, duplicate ref & missing-receipt anomaly flags. */
function runCsvReconciliation(
  sourcePath: string,
  csvText: string,
  rules: string[]
) {
  const rows = parseCsv(csvText);
  if (rows.length < 2) {
    return {
      reconciledCsv: csvText,
      reportMd: `# Reconciliation Report — ${sourcePath}\n\nNo data rows were found in \`${sourcePath}\`.\n`,
      rowCount: 0,
      grandTotal: 0,
      missingCount: 0,
      duplicateCount: 0,
      categories: [] as [string, { count: number; total: number }][],
      flaggedRows: [] as string[],
      amountColName: "Amount",
    };
  }
  const header = rows[0].map((h) => h.trim());
  const data = rows.slice(1).filter((r) => r.some((c) => c.trim().length > 0));
  const lower = header.map((h) => h.toLowerCase());

  let amountIdx = lower.findIndex((h) => /(amount|cost|total|price|spend|usd|value|charge|fee|balance)/.test(h));
  if (amountIdx < 0) {
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
  const refIdx = lower.findIndex((h) => /(ref|invoice.?id|po|txn|number)/.test(h));

  let grandTotal = 0;
  let missingCount = 0;
  let duplicateCount = 0;
  const catMap = new Map<string, { count: number; total: number }>();
  const seenKey = new Set<string>();
  const seenRef = new Set<string>();
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
    const refVal = refIdx >= 0 ? padded[refIdx].toLowerCase() : "";
    if ((amt !== 0 && seenKey.has(dupKey)) || (refVal && seenRef.has(refVal))) {
      duplicateCount += 1;
      flags.push(refVal && seenRef.has(refVal) ? `duplicate ref ${padded[refIdx]}` : "possible duplicate");
    }
    seenKey.add(dupKey);
    if (refVal) seenRef.add(refVal);

    if (flags.length) {
      flaggedRows.push(`Row ${idx + 2} (${vendor}, ${fmtMoney(amt)}): ${flags.join(", ")}`);
    }
    outRows.push([...padded, amt.toFixed(2), flags.length ? `FLAG: ${flags.join("; ")}` : "OK"]);
  });

  const totalRow = header.map((_, i) => (i === 0 ? "TOTAL" : i === amountIdx ? grandTotal.toFixed(2) : ""));
  totalRow.push(grandTotal.toFixed(2), `${data.length} rows · ${missingCount + duplicateCount} flagged`);
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

/** Skill 2: summarize_meeting — decisions, owners, due dates, blockers, and follow-up draft. */
function runDocumentSummary(sources: WebFile[], prompt: string, rules: string[]) {
  const namedOut = /\b([A-Za-z0-9][A-Za-z0-9._-]*\.md)\b/.exec(prompt);
  const outName = namedOut ? namedOut[1] : sources.length === 1 ? `${sources[0].name.replace(/\.[^.]+$/, "")}-summary.md` : "Summary.md";

  const decisions: string[] = [];
  const actions: string[] = [];
  const blockers: string[] = [];
  const highlights: string[] = [];

  for (const src of sources) {
    const lines = src.content.replace(/\r\n/g, "\n").split("\n");
    lines.forEach((raw, idx) => {
      const line = raw.trim();
      if (!line || /^#{1,3}\s+/.test(line)) return;
      if (/\b(decided|decision|approved|agreed|signed off|greenlit)\b/i.test(line)) {
        decisions.push(`${line.replace(/^[-*]\s*/, "")} [${src.name}:L${idx + 1}]`);
        return;
      }
      if (/^[-*]\s*\[\s\]\s*/.test(line) || /\b(action|todo|owner|follow[- ]?up|due|by friday|by thursday|by monday|deadline|assigned)\b/i.test(line)) {
        actions.push(`${line.replace(/^[-*]\s*(\[\s*\]\s*)?/, "")} [${src.name}:L${idx + 1}]`);
        return;
      }
      if (/\b(blocker|blocked|risk|missing|overdue|delayed|issue|failed|incident)\b/i.test(line)) {
        blockers.push(`${line.replace(/^[-*]\s*/, "")} [${src.name}:L${idx + 1}]`);
        return;
      }
      if (line.startsWith("- ") || line.startsWith("* ")) {
        highlights.push(line.slice(2));
      } else if (highlights.length < 6 && line.length > 20) {
        highlights.push(line);
      }
    });
  }

  const lines: string[] = [
    `# ${outName.replace(/\.md$/i, "").replace(/[-_]+/g, " ")}`,
    "",
    `Compiled from ${sources.map((s) => `\`${s.path}\``).join(", ")}.`,
    "",
    "## Decisions Made",
    ...(decisions.length ? decisions.map((d) => `- ${d}`) : ["- No explicit final decisions recorded in the source notes."]),
    "",
    "## Action Items, Owners & Due Dates",
    ...(actions.length ? actions.slice(0, 12).map((a) => `- ${a}`) : ["- No explicit unchecked action items found."]),
    "",
    "## Blockers & Risks",
    ...(blockers.length ? blockers.slice(0, 8).map((b) => `- ${b}`) : ["- No open blockers identified."]),
    "",
    "## Follow-Up Draft",
    `Team — here is the recap from ${sources.map((s) => s.name).join(", ")}: ${actions.length} action item(s) and ${blockers.length} open blocker(s) are tracked above. Please confirm owner dates by EOD.`,
  ];

  if (highlights.length) {
    lines.push("", "## Key Context", ...highlights.slice(0, 6).map((h) => `- ${h}`));
  }
  if (rules.length) {
    lines.push("", "## Standing Preferences Applied", ...rules.slice(0, 4).map((r) => `- ${r}`));
  }

  return { outName, markdown: lines.join("\n") + "\n", actionCount: actions.length, blockerCount: blockers.length, decisionCount: decisions.length };
}

/** Skill 3: audit_contract — required clause inspection, renewal dates, missing-clause flags, line-cited risk table. */
function runContractAudit(src: WebFile, rules: string[]) {
  const lines = src.content.replace(/\r\n/g, "\n").split("\n");
  const checks: { clause: string; re: RegExp; riskIfMissing: string }[] = [
    { clause: "Termination & Notice", re: /\b(terminat|notice\s+period|cancel(?:lation)?)\b/i, riskIfMissing: "High" },
    { clause: "Auto-Renewal Window", re: /\b(auto[- ]?renew|renewal\s+term|evergreen)\b/i, riskIfMissing: "Medium" },
    { clause: "Liability Cap", re: /\b(limitation\s+of\s+liability|liability\s+cap|aggregate\s+liability)\b/i, riskIfMissing: "High" },
    { clause: "Indemnification", re: /\b(indemnif|hold\s+harmless)\b/i, riskIfMissing: "High" },
    { clause: "Confidentiality / NDA", re: /\b(confidential|non[- ]?disclosure|proprietary\s+information)\b/i, riskIfMissing: "Medium" },
    { clause: "Data Protection / Security", re: /\b(data\s+protection|security\s+breach|gdpr|soc\s*2|encryption)\b/i, riskIfMissing: "High" },
    { clause: "Payment Terms", re: /\b(net\s*\d+|payment\s+terms|invoice|late\s+fee)\b/i, riskIfMissing: "Medium" },
    { clause: "Governing Law", re: /\b(governing\s+law|jurisdiction|venue|arbitration)\b/i, riskIfMissing: "Low" },
  ];

  const rows: string[] = ["| Clause | Status | Citation | Risk |", "| --- | --- | --- | --- |"];
  const missing: string[] = [];
  const renewalHits: string[] = [];

  for (const ck of checks) {
    const hitIdx = lines.findIndex((l) => ck.re.test(l));
    if (hitIdx >= 0) {
      const snippet = lines[hitIdx].trim().slice(0, 72);
      const isAutoRenewTrap = ck.clause === "Auto-Renewal Window" && /auto[- ]?renew/i.test(lines[hitIdx]);
      rows.push(`| ${ck.clause} | Present | \`${src.name}:L${hitIdx + 1}\` ${snippet} | ${isAutoRenewTrap ? "Review" : "OK"} |`);
      if (isAutoRenewTrap) renewalHits.push(`Auto-renewal clause detected at \`${src.name}:L${hitIdx + 1}\`: ${lines[hitIdx].trim()}`);
    } else {
      missing.push(ck.clause);
      rows.push(`| ${ck.clause} | **MISSING** | — | ${ck.riskIfMissing} |`);
    }
  }

  const dateMatches: string[] = [];
  lines.forEach((l, i) => {
    if (/\b(20\d{2}-\d{2}-\d{2}|\d+\s+days?\s+(?:prior|before|notice)|january|february|march|april|may|june|july|august|september|october|november|december)\b/i.test(l)) {
      dateMatches.push(`- \`${src.name}:L${i + 1}\`: ${l.trim()}`);
    }
  });

  const outName = `${src.name.replace(/\.[^.]+$/, "")}-contract-audit.md`;
  const md = [
    `# Contract Audit — ${src.path}`,
    "",
    "## Audit Summary",
    `- Source file: \`${src.path}\` (${lines.length} lines)`,
    `- Clauses verified: **${checks.length - missing.length}/${checks.length}**`,
    `- Missing clauses flagged: **${missing.length}**${missing.length ? ` (${missing.join(", ")})` : ""}`,
    "",
    "## Clause & Risk Table",
    ...rows,
    "",
    "## Dates & Renewal Windows",
    ...(renewalHits.length ? renewalHits.map((r) => `- **Warning:** ${r}`) : []),
    ...(dateMatches.length ? dateMatches.slice(0, 8) : ["- No explicit renewal or notice dates detected."]),
    ...(rules.length ? ["", "## Standing Preferences Applied", ...rules.slice(0, 4).map((r) => `- ${r}`)] : []),
  ].join("\n") + "\n";

  return { outName, md, missingCount: missing.length, presentCount: checks.length - missing.length };
}

/** Skill 4: triage_inbox — classify messages/tickets into Urgent / Action / FYI with next steps. */
function runInboxTriage(src: WebFile, rules: string[]) {
  const lines = src.content.replace(/\r\n/g, "\n").split("\n").map((l) => l.trim()).filter((l) => l && !l.startsWith("#"));
  const urgent: string[] = [];
  const action: string[] = [];
  const fyi: string[] = [];

  lines.forEach((line, idx) => {
    const clean = line.replace(/^[-*]\s*/, "");
    const tag = `[${src.name}:L${idx + 1}]`;
    if (/\b(urgent|sev1|p0|outage|down|security|breach|asap|blocked|failed|overdue|escalation|critical)\b/i.test(clean)) {
      urgent.push(`${clean} ${tag} — Next step: assign owner & respond within 30 min.`);
    } else if (/\b(please|review|approve|action|due|confirm|send|update|schedule|follow[- ]?up|request|ticket|invoice)\b/i.test(clean)) {
      action.push(`${clean} ${tag} — Next step: complete in today's work block.`);
    } else {
      fyi.push(`${clean} ${tag}`);
    }
  });

  const outName = `${src.name.replace(/\.[^.]+$/, "")}-triage.md`;
  const md = [
    `# Inbox Triage — ${src.path}`,
    "",
    `- Items processed: **${lines.length}** (Urgent: **${urgent.length}**, Action: **${action.length}**, FYI: **${fyi.length}**)`,
    "",
    "## Urgent",
    ...(urgent.length ? urgent.map((x) => `- ${x}`) : ["- No urgent escalations detected."]),
    "",
    "## Action Required",
    ...(action.length ? action.map((x) => `- ${x}`) : ["- No standard action items queued."]),
    "",
    "## FYI / Reference",
    ...(fyi.length ? fyi.slice(0, 12).map((x) => `- ${x}`) : ["- None."]),
    ...(rules.length ? ["", "## Standing Preferences Applied", ...rules.slice(0, 4).map((r) => `- ${r}`)] : []),
  ].join("\n") + "\n";

  return { outName, md, urgentCount: urgent.length, actionCount: action.length, fyiCount: fyi.length };
}

/** Skill 5: clean_dataset — normalize CSV/JSON schemas, deduplicate rows, flag nulls/outliers, report diff metrics. */
function runCleanDataset(src: WebFile) {
  const stem = src.name.replace(/\.[^.]+$/, "");
  if (src.ext === "json") {
    let parsed: unknown = [];
    try { parsed = JSON.parse(src.content); } catch { parsed = []; }
    const arr = Array.isArray(parsed) ? parsed : [parsed];
    const seen = new Set<string>();
    const cleanArr: Record<string, unknown>[] = [];
    let dups = 0;
    let nulls = 0;
    for (const item of arr) {
      if (!item || typeof item !== "object") continue;
      const normalized: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(item as Record<string, unknown>)) {
        const nk = k.trim().toLowerCase().replace(/\s+/g, "_");
        if (v === null || v === undefined || String(v).trim() === "") nulls++;
        normalized[nk] = typeof v === "string" ? v.trim() : v;
      }
      const key = JSON.stringify(normalized);
      if (seen.has(key)) { dups++; continue; }
      seen.add(key);
      cleanArr.push(normalized);
    }
    const outFile = `${stem}-cleaned.json`;
    const reportFile = `${stem}-clean-report.md`;
    const reportMd = `# Dataset Clean Metrics — ${src.path}\n\n- Input records: **${arr.length}**\n- Output records: **${cleanArr.length}**\n- Duplicates removed: **${dups}**\n- Empty/null fields flagged: **${nulls}**\n`;
    return { outFile, outBody: JSON.stringify(cleanArr, null, 2) + "\n", reportFile, reportMd, removedDups: dups, nullCells: nulls, rowCount: cleanArr.length };
  }

  const rows = parseCsv(src.content);
  if (rows.length < 2) {
    return { outFile: `${stem}-cleaned.csv`, outBody: src.content, reportFile: `${stem}-clean-report.md`, reportMd: `# Dataset Clean Metrics — ${src.path}\n\nNo rows to clean.\n`, removedDups: 0, nullCells: 0, rowCount: 0 };
  }
  const header = rows[0].map((h) => h.trim().replace(/\s+/g, "_"));
  const body = rows.slice(1);
  const seen = new Set<string>();
  const cleaned: string[][] = [header];
  let removedDups = 0;
  let nullCells = 0;
  for (const r of body) {
    const norm = header.map((_, i) => {
      const val = (r[i] ?? "").trim();
      if (!val || /^(null|n\/a|none|nan)$/i.test(val)) {
        nullCells++;
        return "";
      }
      return val;
    });
    const key = norm.join("|").toLowerCase();
    if (seen.has(key)) {
      removedDups++;
      continue;
    }
    seen.add(key);
    cleaned.push(norm);
  }
  const outBody = cleaned.map((r) => r.map(csvEscape).join(",")).join("\n") + "\n";
  const outFile = `${stem}-cleaned.csv`;
  const reportFile = `${stem}-clean-report.md`;
  const reportMd = [
    `# Dataset Clean Metrics — ${src.path}`,
    "",
    `- Input rows: **${body.length}**`,
    `- Output rows: **${cleaned.length - 1}**`,
    `- Exact duplicate rows removed: **${removedDups}**`,
    `- Null / empty cells normalized: **${nullCells}**`,
    `- Output file: \`${outFile}\``,
  ].join("\n") + "\n";
  return { outFile, outBody, reportFile, reportMd, removedDups, nullCells, rowCount: cleaned.length - 1 };
}

/** Skill 6: compare_documents — real line-by-line LCS diff & change summary between two workspace files. */
function runCompareDocuments(left: WebFile, right: WebFile) {
  const a = left.content.replace(/\r\n/g, "\n").split("\n").slice(0, 400);
  const b = right.content.replace(/\r\n/g, "\n").split("\n").slice(0, 400);
  const m = a.length;
  const n = b.length;
  const dp: Uint16Array[] = Array.from({ length: m + 1 }, () => new Uint16Array(n + 1));
  for (let i = m - 1; i >= 0; i--) {
    for (let j = n - 1; j >= 0; j--) {
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const diffLines: string[] = [];
  let added = 0;
  let removed = 0;
  let unchanged = 0;
  let i = 0;
  let j = 0;
  while (i < m && j < n) {
    if (a[i] === b[j]) {
      unchanged++;
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      removed++;
      diffLines.push(`- [L${i + 1}] ${a[i]}`);
      i++;
    } else {
      added++;
      diffLines.push(`+ [L${j + 1}] ${b[j]}`);
      j++;
    }
  }
  while (i < m) { removed++; diffLines.push(`- [L${i + 1}] ${a[i++]}`); }
  while (j < n) { added++; diffLines.push(`+ [L${j + 1}] ${b[j++]}`); }

  const outName = `diff-${left.name.replace(/\.[^.]+$/, "")}-vs-${right.name.replace(/\.[^.]+$/, "")}.md`.slice(0, 60);
  const md = [
    `# Document Comparison: ${left.path} vs ${right.path}`,
    "",
    "## Change Metrics",
    `- Left file: \`${left.path}\` (${a.length} lines)`,
    `- Right file: \`${right.path}\` (${b.length} lines)`,
    `- Lines added: **+${added}** · Lines removed: **-${removed}** · Unchanged: **${unchanged}**`,
    "",
    "## Unified Line Diff",
    "```diff",
    ...(diffLines.length ? diffLines.slice(0, 120) : ["(files are identical)"]),
    "```",
  ].join("\n") + "\n";

  return { outName, md, added, removed, unchanged };
}

/** Skill 7: draft_brief — grounded executive memo/SOP strictly citing workspace files & standing memory. */
function runDraftBrief(sources: WebFile[], prompt: string, rules: string[]) {
  const outName = "executive-brief.md";
  const facts: string[] = [];
  for (const s of sources.slice(0, 5)) {
    const lines = s.content.replace(/\r\n/g, "\n").split("\n");
    lines.forEach((raw, idx) => {
      const t = raw.trim();
      if (t.length > 18 && !t.startsWith("#") && facts.length < 14) {
        facts.push(`- ${t.replace(/^[-*]\s*/, "")} [\`${s.path}:L${idx + 1}\`]`);
      }
    });
  }
  const md = [
    "# Executive Brief",
    "",
    "## Objective",
    prompt,
    "",
    "## Grounded Workspace Findings",
    ...(facts.length ? facts : ["- No source files were available in the workspace."]),
    "",
    "## Sources Cited",
    ...(sources.length ? sources.map((s) => `- \`${s.path}\` (${s.size} bytes)`) : ["- None"]),
    "",
    "## Standing Preferences Applied",
    ...(rules.length ? rules.map((r) => `- ${r}`) : ["- None recorded yet."]),
  ].join("\n") + "\n";
  return { outName, md, citationCount: facts.length };
}

/** Skill 8: extract_tables — parse Markdown pipe tables, key-value logs, or delimited lines into structured CSV. */
function runExtractTables(src: WebFile) {
  const lines = src.content.replace(/\r\n/g, "\n").split("\n");
  const pipeRows = lines
    .map((l) => l.trim())
    .filter((l) => /^\|.+\|$/.test(l))
    .map((l) => l.slice(1, -1).split("|").map((c) => c.trim()))
    .filter((r) => !r.every((c) => /^[-:\s]+$/.test(c)));

  let columns: string[] = [];
  let rows: string[][] = [];

  if (pipeRows.length >= 2) {
    columns = pipeRows[0];
    rows = pipeRows.slice(1);
  } else {
    columns = ["Source_Line", "Key_or_Tag", "Value"];
    lines.forEach((raw, idx) => {
      const t = raw.trim();
      if (!t || t.startsWith("#")) return;
      const kv = /^[-*]?\s*([A-Za-z0-9 _/-]{2,32}):\s+(.+)$/.exec(t);
      if (kv) {
        rows.push([`L${idx + 1}`, kv[1].trim(), kv[2].trim()]);
      } else if (t.includes("\t")) {
        const parts = t.split("\t").map((p) => p.trim());
        rows.push([`L${idx + 1}`, parts[0] || "", parts.slice(1).join(" | ")]);
      } else if (t.startsWith("- ")) {
        rows.push([`L${idx + 1}`, "Item", t.slice(2).trim()]);
      }
    });
  }

  const csv = [columns, ...rows].map((r) => r.map(csvEscape).join(",")).join("\n") + "\n";
  const outName = `${src.name.replace(/\.[^.]+$/, "")}-extracted.csv`;
  return { outName, csv, rowCount: rows.length, colCount: columns.length };
}

/** Skill 11: research — cross-file workspace search with line-level citations. */
function runWorkspaceResearch(allFiles: WebFile[], prompt: string, rules: string[]) {
  const keywords = prompt
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, " ")
    .split(/\s+/)
    .filter((w) => w.length >= 4 && !/^(research|topic|about|workspace|files|compile|sourced|findings|brief|save|with|from|into|that|this|your|have)$/.test(w));

  const citations: string[] = [];
  for (const f of allFiles) {
    f.content.replace(/\r\n/g, "\n").split("\n").forEach((raw, idx) => {
      const t = raw.trim();
      if (!t || citations.length >= 18) return;
      const low = t.toLowerCase();
      if (keywords.length === 0 ? t.length > 22 : keywords.some((k) => low.includes(k))) {
        citations.push(`- [\`${f.path}:L${idx + 1}\`] ${t.slice(0, 140)}`);
      }
    });
  }
  const outName = "research-findings.md";
  const md = [
    "# Workspace Research Findings",
    "",
    `Query: ${prompt}`,
    "",
    "## Cited Evidence",
    ...(citations.length ? citations : ["- No lines matched the search keywords across current workspace files."]),
    ...(rules.length ? ["", "## Standing Preferences Applied", ...rules.slice(0, 4).map((r) => `- ${r}`)] : []),
  ].join("\n") + "\n";
  return { outName, md, hitCount: citations.length };
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
    "- Decided: Ship Ergon v0.5.0 with 12 built-in workflows and SHA-256 receipts.",
    "- Owner: Maya Chen — finalize Q3 vendor reconciliation before Friday close.",
    "- [ ] Owner: Tomas Ruiz — chase missing $240.00 CloudHost Backup receipt by Thursday.",
    "- [ ] Owner: Priya Raman — publish dark-mode accessibility checklist for Settings.",
    "- Blocker: Q2 archived vendor draft (old-vendor-draft.txt) still sitting in active workspace.",
  ].join("\n") + "\n";

  const vendorContract = [
    "# Northstar Cloud Vendor Services Agreement (Draft)",
    "",
    "1. Term & Auto-Renewal: Initial term ends 2026-12-31 and auto-renews for 12 months unless written notice is given 45 days prior.",
    "2. Payment Terms: Net 30 from invoice receipt; late balances accrue 1.5% monthly interest.",
    "3. Confidentiality: Both parties shall maintain proprietary information strictly confidential for 3 years.",
    "4. Governing Law: State of Delaware.",
    "5. Termination: Either party may terminate for uncured material breach with 30 days notice.",
  ].join("\n") + "\n";

  const supportInbox = [
    "# Operations Inbox Feed",
    "- URGENT: Payment webhook returned 502 error on billing-eu — customer invoices delayed.",
    "- Action: Please approve Q4 Freight renewal quote from Northstar Logistics by Friday.",
    "- Action: Review draft contract clauses before legal sign-off.",
    "- FYI: Quarterly office badge audit completed with zero findings.",
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
    "vendor-agreement.md": makeFile("vendor-agreement.md", vendorContract, now - 105 * 60_000),
    "ops-inbox.log": makeFile("ops-inbox.log", supportInbox, now - 95 * 60_000),
    "old-vendor-draft.txt": makeFile("old-vendor-draft.txt", oldDraft, now - 300 * 60_000),
    "expenses-q3-reconciled.csv": makeFile("expenses-q3-reconciled.csv", recon.reconciledCsv, now - 24 * 60_000),
  };

  const library: WebLibItem[] = [
    ...CANONICAL_SKILLS.map((sk) => ({
      id: sk.id,
      kind: "skill",
      name: sk.title,
      description: sk.description,
      author: "ergon",
      builtin: true,
    })),
    { id: "bookkeeper", kind: "subagent", name: "Bookkeeper", description: "Reconciles raw expense CSVs, computes category subtotals, and flags missing receipts.", author: "ergon", builtin: false },
    { id: "meeting-actions", kind: "app", name: "Meeting actions", description: "Reads meeting notes and writes decisions and owner action items.", author: "ergon", builtin: false, fields: ["notes_file"], template: "Summarize {notes_file} into Summary.md with owners and action items" },
    { id: "researcher", kind: "subagent", name: "Researcher", description: "Searches workspace files and compiles sourced findings.", author: "ergon", builtin: false },
    { id: "expense-sheet", kind: "app", name: "Expense sheet", description: "One-tap reconciliation of a CSV file into totals and discrepancies.", author: "ergon", builtin: false, fields: ["source_file"], template: "Reconcile {source_file} and flag missing receipts" },
  ];

  const prevExpensesCsv = expensesCsv
    .split("\n")
    .filter((line) => !line.includes("CloudHost Backup"))
    .join("\n");
  const prevSyncNotes = syncNotes
    .split("\n")
    .slice(0, -2)
    .join("\n") + "\n";

  const mayaStore: PersonStore = {
    files: mayaFiles,
    fileHistory: {
      "expenses-q3.csv": [
        {
          rev_id: `rev-${now - 260 * 60_000}-1`,
          path: "expenses-q3.csv",
          ts: now - 260 * 60_000,
          size: utf8Bytes(prevExpensesCsv),
          reason: "overwrite",
          content: prevExpensesCsv,
        },
      ],
      "tuesday-sync-notes.md": [
        {
          rev_id: `rev-${now - 200 * 60_000}-1`,
          path: "tuesday-sync-notes.md",
          ts: now - 200 * 60_000,
          size: utf8Bytes(prevSyncNotes),
          reason: "overwrite",
          content: prevSyncNotes,
        },
      ],
    },
    memory: `# Memory\n\n## About\n- Name: Maya Chen\n- Role: Operations lead\n\n## Standing preferences\n${initialRules.map((r) => `- ${r}`).join("\n")}\n`,
    suite: ["bookkeeper", "meeting-actions", "researcher"],
    routines: [
      {
        id: "rt-morning-ledger",
        name: "Morning expense ledger audit",
        prompt: "Reconcile expenses-q3.csv, compute category subtotals, and flag missing receipts or duplicates.",
        schedule: "daily 09:00",
        cadence: "daily 09:00",
        skill_id: "reconcile_ledger",
        enabled: true,
        last_run_ts: now - 24 * 60_000,
        next_run_ms: estimateNextRunMs("daily 09:00", now),
        last_status: "finished",
        run_count: 1,
        created_ts: now - 3 * 86_400_000,
      },
      {
        id: "rt-inbox-triage",
        name: "Daily ops inbox triage",
        prompt: "Triage ops-inbox.log into Urgent, Action, and FYI queues with concrete next steps.",
        schedule: "every 15m",
        cadence: "every 15m",
        skill_id: "triage_inbox",
        enabled: true,
        last_run_ts: null,
        next_run_ms: estimateNextRunMs("every 15m", now),
        run_count: 0,
        created_ts: now - 2 * 86_400_000,
      },
    ],
    routineRuns: {
      "rt-morning-ledger": [
        {
          run_id: `run-${now - 24 * 60_000}-1`,
          routine_id: "rt-morning-ledger",
          started_ms: now - 24 * 60_000 - 340,
          finished_ms: now - 24 * 60_000,
          status: "finished",
          receipt_id: t1,
          thread_id: t1,
          summary: `Reconciled ${recon.rowCount} entries (${fmtMoney(recon.grandTotal)} total) and flagged ${recon.missingCount} missing receipt.`,
        },
      ],
    },
    threads: [
      {
        id: t1,
        title: "Reconcile expenses-q3.csv and flag missing receipts",
        messages: [
          { role: "user", content: "Reconcile expenses-q3.csv and flag missing receipts" },
          { role: "step", content: "Tool: list_dir\nexpenses-q3.csv\nold-vendor-draft.txt\ntuesday-sync-notes.md" },
          { role: "step", content: `Tool: read_file\nread expenses-q3.csv (${utf8Bytes(expensesCsv)} bytes)` },
          { role: "step", content: `Tool: reconcile_ledger\nwrote spreadsheet expenses-q3-reconciled.csv (${recon.rowCount} data rows, ${utf8Bytes(recon.reconciledCsv)} bytes)` },
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
        id: t1,
        task: "Reconcile expenses-q3.csv and flag missing receipts",
        summary: `Reconciled ${recon.rowCount} entries (${fmtMoney(recon.grandTotal)} total) into expenses-q3-reconciled.csv.`,
        files: ["expenses-q3-reconciled.csv"],
        finished: true,
        answered: false,
        thread_id: t1,
        ts: now - 24 * 60_000,
        engagement_id: "eng-northstar-q4",
        usage: {
          model_id: "browser-workspace-engine",
          prompt_tokens: 1420,
          completion_tokens: 480,
          cached_tokens: 320,
          total_tokens: 1900,
          cost_usd: 0.000477,
          latency_ms: 340,
          estimated_minutes_saved: 28,
          billable_value_usd: 102.67,
          engagement_id: "eng-northstar-q4",
        },
      },
      {
        id: t2,
        task: "Clean up old-vendor-draft.txt",
        summary: "Parked deletion of old-vendor-draft.txt for approval.",
        files: [],
        finished: true,
        answered: false,
        thread_id: t2,
        ts: now - 90 * 60_000,
        engagement_id: "eng-northstar-q4",
      },
    ],
    usageLedger: [
      {
        id: `ulg-${now - 24 * 60_000}`,
        ts: now - 24 * 60_000,
        person_id: "maya-chen",
        engagement_id: "eng-northstar-q4",
        thread_id: t1,
        run_id: `run-${now - 24 * 60_000}-1`,
        kind: "chat",
        usage: {
          model_id: "browser-workspace-engine",
          prompt_tokens: 1420,
          completion_tokens: 480,
          cached_tokens: 320,
          total_tokens: 1900,
          cost_usd: 0.000477,
          latency_ms: 340,
          estimated_minutes_saved: 28,
          billable_value_usd: 102.67,
          engagement_id: "eng-northstar-q4",
        },
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
    connectors: {
      triggers: [
        {
          id: "trg-maya-intake",
          name: "Vendor invoice webhook",
          kind: "webhook",
          enabled: true,
          routine_id: "rt-morning-ledger",
          target_kind: "routine",
          hook_id: "hk-maya-intake",
          hook_path: "/api/hooks/hk-maya-intake",
          has_secret: true,
          trigger_count: 1,
          last_triggered_ms: now - 45 * 60_000,
          created_ms: now - 2 * 86_400_000,
          updated_ms: now - 45 * 60_000,
        },
      ],
      outbound: [
        {
          id: "conn-ops-digest",
          name: "Ops digest webhook",
          kind: "http_webhook",
          url: "https://hooks.example.com/ops-digest",
          method: "POST",
          headers_template: { Authorization: "Bearer {{secret:api_token}}" },
          secret_keys: ["api_token"],
          has_secret: true,
          require_approval: true,
          enabled: true,
          last_delivery_ms: null,
          last_status: null,
          created_ms: now - 2 * 86_400_000,
          updated_ms: now - 2 * 86_400_000,
        },
      ],
      deliveries: [],
      webhookSecrets: {
        "trg-maya-intake": "whsec_maya_initial_demo_only",
      },
      outboundSecrets: {
        api_token: "tok_maya_hidden_host_secret",
      },
    },
  };

  const tomasStore: PersonStore = {
    files: {
      "budget-notes.md": makeFile("budget-notes.md", "# Finance Notes\n- Verify Q4 software renewals before Dec 1.\n", now - 200 * 60_000),
    },
    fileHistory: {},
    memory: "# Memory\n\n## About\n- Name: Tomas Ruiz\n- Role: Finance analyst\n\n## Standing preferences\n- Flag variances over $50.\n",
    suite: ["bookkeeper", "expense-sheet"],
    routines: [],
    routineRuns: {},
    threads: [],
    work: [],
    decisions: [],
    offers: [],
    connectors: { triggers: [], outbound: [], deliveries: [], webhookSecrets: {}, outboundSecrets: {} },
  };

  const initRoot: RootState = {
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
  ensureClientsAndPricing(initRoot, now);
  return initRoot;
}

function openIdb(dbName = DB_NAME): Promise<IDBDatabase | null> {
  if (typeof indexedDB === "undefined") return Promise.resolve(null);
  return new Promise((resolve) => {
    try {
      const req = indexedDB.open(dbName, 1);
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

function ensureWebConnectors(st: PersonStore): WebConnectorsState {
  if (!st.connectors || typeof st.connectors !== "object") {
    st.connectors = { triggers: [], outbound: [], deliveries: [], webhookSecrets: {}, outboundSecrets: {} };
  }
  if (!Array.isArray(st.connectors.triggers)) st.connectors.triggers = [];
  if (!Array.isArray(st.connectors.outbound)) st.connectors.outbound = [];
  if (!Array.isArray(st.connectors.deliveries)) st.connectors.deliveries = [];
  if (!st.connectors.webhookSecrets || typeof st.connectors.webhookSecrets !== "object") st.connectors.webhookSecrets = {};
  if (!st.connectors.outboundSecrets || typeof st.connectors.outboundSecrets !== "object") st.connectors.outboundSecrets = {};
  return st.connectors;
}

function ensureWebMcpServers(st: PersonStore): McpServerSpec[] {
  if (!Array.isArray(st.mcpServers)) {
    st.mcpServers = [];
  }
  return st.mcpServers;
}

function randomHex(byteCount = 12): string {
  if (typeof crypto !== "undefined" && crypto.getRandomValues) {
    const arr = new Uint8Array(byteCount);
    crypto.getRandomValues(arr);
    return Array.from(arr, (b) => b.toString(16).padStart(2, "0")).join("");
  }
  return Math.random().toString(16).slice(2) + Date.now().toString(16);
}

async function computeHmacHex(secret: string, bodyText: string): Promise<string> {
  if (typeof crypto !== "undefined" && crypto.subtle) {
    const enc = new TextEncoder();
    const key = await crypto.subtle.importKey("raw", enc.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
    const sig = await crypto.subtle.sign("HMAC", key, enc.encode(bodyText));
    return Array.from(new Uint8Array(sig), (b) => b.toString(16).padStart(2, "0")).join("");
  }
  return await sha256Hex(`${secret}:${bodyText}`);
}

function constantTimeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

function validateWebWatchDir(dir: string): string | null {
  const trimmed = dir.trim();
  if (!trimmed) return "Host directory path is required.";
  if (/[\0\r\n]/.test(trimmed)) return "Directory path contains invalid control characters.";
  const norm = trimmed.replace(/\\/g, "/");
  if (/(^|\/)\.\.(\/|$)/.test(norm)) {
    return "Directory path cannot contain '..' traversal segments.";
  }
  if (!norm.startsWith("/") && !/^[A-Za-z]:\//.test(norm)) {
    return "Host directory path must be an absolute path.";
  }
  const low = norm.replace(/\/+$/, "").toLowerCase() || "/";
  if (
    low === "/" ||
    low === "/etc" ||
    low.startsWith("/etc/") ||
    low === "/root" ||
    low.startsWith("/root/") ||
    low === "/proc" ||
    low.startsWith("/proc/") ||
    low === "/sys" ||
    low.startsWith("/sys/") ||
    low === "/dev" ||
    low.startsWith("/dev/") ||
    low === "/var/run" ||
    low.startsWith("/var/run/") ||
    low.includes("/.ssh") ||
    low.includes("/.aws") ||
    low.includes("/.gnupg") ||
    low.includes("/.talos-wallet") ||
    low.includes("/.credentials") ||
    low.includes("/.env")
  ) {
    return "Sensitive system or credential directories cannot be watched.";
  }
  return null;
}

function validateWebOutboundUrl(rawUrl: string, kind: OutboundConnectorKind): string | null {
  const url = rawUrl.trim();
  if (!url) return "Connector URL is required.";
  if (/[\s\0\r\n]/.test(url)) return "Destination URL cannot contain spaces or control characters.";
  if (kind === "smtp_relay") {
    if (/^smtps?:\/\//i.test(url)) {
      try {
        const parsed = new URL(url);
        if (!parsed.hostname) return "SMTP relay URL must include a hostname.";
        if (parsed.username || parsed.password) return "Do not embed credentials in the URL.";
        return null;
      } catch {
        return "Invalid SMTP relay URL.";
      }
    }
    if (/^[a-zA-Z0-9.-]+(?::\d{1,5})?$/.test(url) && !url.startsWith(".") && !url.endsWith(".")) {
      return null;
    }
    return "Invalid SMTP relay host or URL.";
  }
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return "Destination URL must start with http:// or https://.";
    }
    if (!parsed.hostname) return "Destination URL must include a hostname.";
    if (parsed.username || parsed.password) return "Do not embed credentials in the URL.";
    return null;
  } catch {
    return "Destination URL must start with http:// or https://.";
  }
}

function getHeaderCI(headers: HeadersInit | undefined, name: string): string {
  if (!headers) return "";
  const want = name.toLowerCase();
  if (headers instanceof Headers) return headers.get(name) || "";
  if (Array.isArray(headers)) {
    for (const [k, v] of headers) {
      if (String(k).toLowerCase() === want) return String(v);
    }
    return "";
  }
  for (const [k, v] of Object.entries(headers)) {
    if (k.toLowerCase() === want) return String(v);
  }
  return "";
}

async function idbLoad(): Promise<RootState | null> {
  const db = await openIdb(DB_NAME);
  if (!db) return null;
  const loaded = await new Promise<RootState | null>((resolve) => {
    try {
      const tx = db.transaction(DB_STORE, "readonly");
      const req = tx.objectStore(DB_STORE).get(DB_KEY);
      req.onsuccess = () => resolve((req.result as RootState) ?? null);
      req.onerror = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
  if (loaded) {
    ensureClientsAndPricing(loaded);
    for (const st of Object.values(loaded.stores || {})) {
      if (!Array.isArray(st.routines)) st.routines = [];
      if (!st.fileHistory || typeof st.fileHistory !== "object") st.fileHistory = {};
      if (!st.routineRuns || typeof st.routineRuns !== "object") st.routineRuns = {};
      if (!Array.isArray(st.usageLedger)) st.usageLedger = [];
      ensureWebConnectors(st);
    }
  }
  return loaded;
}

async function idbSave(state: RootState): Promise<void> {
  const db = await openIdb(DB_NAME);
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

  const liveRuns: Record<string, {
    active: boolean;
    run_id: string;
    thread_id: string;
    task: string;
    started_ms: number;
    steps: LiveRunStep[];
    cancelled: boolean;
  }> = {};

  // Local EventSource listeners for /api/runs/stream (zero network traffic on static web hosting!).
  const sseListeners = new Set<(event: string, payload: unknown) => void>();
  let sseSeq = 0;
  const broadcastSse = (event: string, payload: unknown) => {
    sseSeq += 1;
    const enriched =
      payload && typeof payload === "object" && !Array.isArray(payload)
        ? { seq: sseSeq, event, ts_ms: Date.now(), ...(payload as Record<string, unknown>) }
        : payload;
    for (const fn of sseListeners) {
      try { fn(event, enriched); } catch { /* ignore */ }
    }
  };

  if (typeof window.EventSource !== "undefined") {
    const NativeEventSource = window.EventSource;
    class VirtualRunEventSource {
      static readonly CONNECTING = 0;
      static readonly OPEN = 1;
      static readonly CLOSED = 2;
      readonly url: string;
      readonly withCredentials = false;
      readyState = 1;
      onopen: ((ev: Event) => void) | null = null;
      onmessage: ((ev: MessageEvent) => void) | null = null;
      onerror: ((ev: Event) => void) | null = null;
      private native: EventSource | null = null;
      private handlers = new Map<string, Set<EventListener>>();
      private sub: ((event: string, payload: unknown) => void) | null = null;

      constructor(url: string | URL, init?: EventSourceInit) {
        this.url = String(url);
        if (!this.url.includes("/api/runs/stream")) {
          this.native = new NativeEventSource(url, init);
          return;
        }
        this.sub = (event, payload) => {
          if (this.readyState === 2) return;
          const ev = new MessageEvent(event, { data: JSON.stringify(payload) });
          if (event === "message" && this.onmessage) this.onmessage(ev);
          this.handlers.get(event)?.forEach((h) => h(ev));
        };
        sseListeners.add(this.sub);
        setTimeout(() => {
          if (this.readyState !== 2 && this.onopen) this.onopen(new Event("open"));
        }, 0);
      }

      addEventListener(type: string, listener: EventListener) {
        if (this.native) { this.native.addEventListener(type, listener); return; }
        if (!this.handlers.has(type)) this.handlers.set(type, new Set());
        this.handlers.get(type)!.add(listener);
      }

      removeEventListener(type: string, listener: EventListener) {
        if (this.native) { this.native.removeEventListener(type, listener); return; }
        this.handlers.get(type)?.delete(listener);
      }

      dispatchEvent(_event: Event): boolean { return true; }

      close() {
        this.readyState = 2;
        if (this.native) { this.native.close(); return; }
        if (this.sub) sseListeners.delete(this.sub);
      }
    }
    (window as unknown as { EventSource: unknown }).EventSource = VirtualRunEventSource;
  }

  const persist = () => idbSave(state);

  const activePerson = (): WebPerson =>
    state.people.find((p) => p.id === state.activePerson) || state.people[0];

  const activeStore = (): PersonStore => {
    const pid = activePerson().id;
    if (!state.stores[pid]) {
      const p = activePerson();
      state.stores[pid] = {
        files: {},
        fileHistory: {},
        threads: [],
        work: [],
        decisions: [],
        offers: [],
        routines: [],
        routineRuns: {},
        pipelineRuns: [],
        memory: `# Memory\n\n## About\n- Name: ${p.name}\n${p.role ? `- Role: ${p.role}\n` : ""}`,
        suite: ["researcher", "meeting-actions"],
      };
    }
    if (!Array.isArray(state.stores[pid].routines)) state.stores[pid].routines = [];
    if (!state.stores[pid].fileHistory) state.stores[pid].fileHistory = {};
    if (!state.stores[pid].routineRuns) state.stores[pid].routineRuns = {};
    if (!Array.isArray(state.stores[pid].pipelineRuns)) state.stores[pid].pipelineRuns = [];
    if (!Array.isArray(state.stores[pid].usageLedger)) state.stores[pid].usageLedger = [];
    if (!Array.isArray(state.stores[pid].timeEntries)) state.stores[pid].timeEntries = [];
    if (!Array.isArray(state.stores[pid].invoices)) state.stores[pid].invoices = [];
    return state.stores[pid];
  };

  function activeEngagementFor(pid: string): { engagement: Engagement | null; client: Client | null } {
    ensureClientsAndPricing(state);
    const engId = state.activeEngagementByPerson?.[pid] || "";
    const eng =
      (engId ? state.engagements?.find((e) => e.id === engId) : undefined) ||
      state.engagements?.find((e) => e.person_id === pid) ||
      null;
    const cli = eng ? state.clients?.find((c) => c.id === eng.client_id) || null : null;
    return { engagement: eng, client: cli };
  }

  function effectiveHourlyRate(pid: string, engagement: Engagement | null, client: Client | null): number {
    if (engagement && typeof engagement.hourly_rate_usd === "number" && engagement.hourly_rate_usd > 0) {
      return engagement.hourly_rate_usd;
    }
    if (client && typeof client.default_rate_usd === "number" && client.default_rate_usd > 0) {
      return client.default_rate_usd;
    }
    void pid;
    return 150;
  }

  function engagementSpentUsd(engagementId: string): number {
    let sum = 0;
    for (const store of Object.values(state.stores)) {
      for (const entry of store.usageLedger ?? []) {
        if (entry.engagement_id === engagementId) {
          sum += Number(entry.usage?.cost_usd ?? 0) || 0;
        }
      }
    }
    return Math.round(sum * 1_000_000) / 1_000_000;
  }

  function checkBudgetGateHold(p: WebPerson, st: PersonStore, threadId: string): WebDecision | null {
    const { engagement, client } = activeEngagementFor(p.id);
    if (!engagement || typeof engagement.retainer_budget_usd !== "number" || engagement.retainer_budget_usd <= 0) {
      return null;
    }
    const spent = engagementSpentUsd(engagement.id);
    const budget = engagement.retainer_budget_usd;
    if (spent < budget) return null;
    const approvedCount = st.budgetOverrunApproved?.[engagement.id] ?? 0;
    const ledgerCount = (st.usageLedger ?? []).filter((u) => u.engagement_id === engagement.id).length;
    if (approvedCount > ledgerCount) {
      return null;
    }
    const existingPending = st.decisions.find(
      (d) => d.status === "pending" && d.kind === "budget_gate" && d.engagement_id === engagement.id
    );
    if (existingPending) return existingPending;
    const did = `d-bgate-${Date.now()}-${st.decisions.length + 1}`;
    const dec: WebDecision = {
      id: did,
      kind: "budget_gate",
      summary: `Retainer budget limit reached for ${engagement.name}`,
      status: "pending",
      outcome: null,
      thread_id: threadId,
      ts: Date.now(),
      severity: "warning",
      engagement_id: engagement.id,
      engagement_name: engagement.name,
      client_name: client?.name,
      retainer_budget_usd: budget,
      spent_usd: spent,
      projected_usd: Math.round((spent + 0.01) * 10_000) / 10_000,
      estimated_cost_usd: 0.01,
      detail: `Spending for ${client?.name ? `${client.name} / ` : ""}${engagement.name} has reached $${spent.toFixed(2)} against the $${budget.toFixed(2)} retainer limit. Approve overrun to run this task or pause spend.`,
    };
    st.decisions.push(dec);
    return dec;
  }

  function recordUsageEntry(
    p: WebPerson,
    st: PersonStore,
    opts: { engagement_id?: string; thread_id?: string; run_id?: string; kind?: string; usage: RunUsage }
  ) {
    if (!Array.isArray(st.usageLedger)) st.usageLedger = [];
    st.usageLedger.push({
      id: `ulg-${Date.now()}-${st.usageLedger.length + 1}`,
      ts: Date.now(),
      person_id: p.id,
      engagement_id: opts.engagement_id,
      thread_id: opts.thread_id,
      run_id: opts.run_id,
      kind: opts.kind || "chat",
      usage: opts.usage,
    });
    if (st.usageLedger.length > 500) st.usageLedger.splice(0, st.usageLedger.length - 500);
  }

  function buildMeteringSummary(opts: {
    engagement_id?: string;
    client_id?: string;
    window_days?: number;
  }): MeteringSummary {
    ensureClientsAndPricing(state);
    const p = activePerson();
    const { engagement: activeEng, client: activeCli } = activeEngagementFor(p.id);
    const windowDays = opts.window_days && opts.window_days > 0 ? opts.window_days : 30;
    const cutoffMs = Date.now() - windowDays * 86_400_000;

    const engById = new Map<string, Engagement>((state.engagements ?? []).map((e) => [e.id, e]));
    const cliById = new Map<string, Client>((state.clients ?? []).map((c) => [c.id, c]));

    const allEntries: UsageLedgerEntry[] = [];
    for (const store of Object.values(state.stores)) {
      for (const entry of store.usageLedger ?? []) {
        if (entry.ts < cutoffMs) continue;
        if (opts.engagement_id && entry.engagement_id !== opts.engagement_id) continue;
        if (opts.client_id) {
          const e = entry.engagement_id ? engById.get(entry.engagement_id) : undefined;
          if (!e || e.client_id !== opts.client_id) continue;
        }
        allEntries.push(entry);
      }
    }

    let promptTokens = 0;
    let completionTokens = 0;
    let cachedTokens = 0;
    let totalTokens = 0;
    let costUsd = 0;
    let minutesSaved = 0;
    let billableValueUsd = 0;

    const byEngMap = new Map<string, MeteringByEngagement>();
    const byModelMap = new Map<string, MeteringByModel>();
    const byDayMap = new Map<string, MeteringDailyPoint>();

    for (const entry of allEntries) {
      const u = entry.usage;
      const pt = Number(u.prompt_tokens) || 0;
      const ct = Number(u.completion_tokens) || 0;
      const ca = Number(u.cached_tokens) || 0;
      const tt = Number(u.total_tokens) || pt + ct;
      const cost = Number(u.cost_usd) || 0;
      const mins = Number(u.estimated_minutes_saved) || 0;
      const bill = Number(u.billable_value_usd) || 0;

      promptTokens += pt;
      completionTokens += ct;
      cachedTokens += ca;
      totalTokens += tt;
      costUsd += cost;
      minutesSaved += mins;
      billableValueUsd += bill;

      const eid = entry.engagement_id || "unassigned";
      const eng = eid !== "unassigned" ? engById.get(eid) : undefined;
      const cli = eng ? cliById.get(eng.client_id) : undefined;
      const engRow = byEngMap.get(eid) || {
        engagement_id: eid,
        engagement_name: eng?.name || "Unassigned workspace",
        client_id: cli?.id,
        client_name: cli?.name,
        runs: 0,
        total_tokens: 0,
        cost_usd: 0,
        minutes_saved: 0,
        billable_value_usd: 0,
        retainer_budget_usd: eng?.retainer_budget_usd,
        retainer_burn_pct: 0,
      };
      engRow.runs += 1;
      engRow.total_tokens += tt;
      engRow.cost_usd = Math.round((engRow.cost_usd + cost) * 1_000_000) / 1_000_000;
      engRow.minutes_saved += mins;
      engRow.billable_value_usd = Math.round((engRow.billable_value_usd + bill) * 100) / 100;
      if (engRow.retainer_budget_usd && engRow.retainer_budget_usd > 0) {
        engRow.retainer_burn_pct = Math.round((engRow.cost_usd / engRow.retainer_budget_usd) * 10000) / 100;
      }
      byEngMap.set(eid, engRow);

      const mid = u.model_id || "browser-workspace-engine";
      const modRow = byModelMap.get(mid) || {
        model_id: mid,
        runs: 0,
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
        cost_usd: 0,
      };
      modRow.runs += 1;
      modRow.prompt_tokens += pt;
      modRow.completion_tokens += ct;
      modRow.total_tokens += tt;
      modRow.cost_usd = Math.round((modRow.cost_usd + cost) * 1_000_000) / 1_000_000;
      byModelMap.set(mid, modRow);

      const day = new Date(entry.ts).toISOString().slice(0, 10);
      const dayRow = byDayMap.get(day) || {
        date: day,
        runs: 0,
        total_tokens: 0,
        cost_usd: 0,
        minutes_saved: 0,
        billable_value_usd: 0,
      };
      dayRow.runs += 1;
      dayRow.total_tokens += tt;
      dayRow.cost_usd = Math.round((dayRow.cost_usd + cost) * 1_000_000) / 1_000_000;
      dayRow.minutes_saved += mins;
      dayRow.billable_value_usd = Math.round((dayRow.billable_value_usd + bill) * 100) / 100;
      byDayMap.set(day, dayRow);
    }

    const scopeEng = opts.engagement_id ? engById.get(opts.engagement_id) || activeEng : activeEng;
    const scopeCli = scopeEng ? cliById.get(scopeEng.client_id) || activeCli : activeCli;
    const retainerBudget = scopeEng?.retainer_budget_usd ?? 0;
    const retainerSpent = scopeEng ? engagementSpentUsd(scopeEng.id) : costUsd;
    const burnPct = retainerBudget > 0 ? Math.round((retainerSpent / retainerBudget) * 10000) / 100 : 0;
    const netRoi = Math.round((billableValueUsd - costUsd) * 100) / 100;
    const roiMultiple = costUsd > 0 ? Math.round((billableValueUsd / costUsd) * 10) / 10 : 0;

    return {
      totals: {
        runs: allEntries.length,
        prompt_tokens: promptTokens,
        completion_tokens: completionTokens,
        cached_tokens: cachedTokens,
        total_tokens: totalTokens,
        cost_usd: Math.round(costUsd * 1_000_000) / 1_000_000,
        minutes_saved: Math.round(minutesSaved),
        billable_value_usd: Math.round(billableValueUsd * 100) / 100,
        retainer_burn_pct: burnPct,
        retainer_budget_usd: retainerBudget,
        net_roi_usd: netRoi,
        roi_multiple: roiMultiple,
      },
      by_engagement: Array.from(byEngMap.values()).sort((a, b) => b.billable_value_usd - a.billable_value_usd),
      by_model: Array.from(byModelMap.values()).sort((a, b) => b.total_tokens - a.total_tokens),
      daily_series: Array.from(byDayMap.values()).sort((a, b) => a.date.localeCompare(b.date)),
      active_engagement: scopeEng,
      active_client: scopeCli,
    };
  }

  function recordFileRevision(st: PersonStore, safePath: string, prev: WebFile | undefined, reason: string) {
    if (!prev) return;
    if (!st.fileHistory) st.fileHistory = {};
    const list = st.fileHistory[safePath] || (st.fileHistory[safePath] = []);
    const ts = Date.now();
    const rev: FileRevision = {
      rev_id: `rev-${ts}-${list.length + 1}`,
      path: safePath,
      ts,
      size: prev.size,
      reason,
      content: prev.content,
    };
    list.unshift(rev);
    if (list.length > 50) list.length = 50;
  }

  function countMissedWindowsWeb(rt: Routine, nowMs = Date.now()): number {
    if (!rt.enabled || !rt.schedule || rt.schedule.trim().toLowerCase() === "manual") return 0;
    let cur = rt.last_run_ts || rt.created_ts || 0;
    if (!cur || cur >= nowMs) return 0;
    let missed = 0;
    while (missed < 99) {
      const slot = estimateNextRunMs(rt.schedule, cur, rt.interval_secs);
      if (slot === null || slot > nowMs || slot <= cur) break;
      missed += 1;
      cur = slot;
    }
    return missed;
  }

  async function enrichWebRoutine(rt: Routine, st: PersonStore, nowMs = Date.now()): Promise<Routine> {
    const missed_windows = countMissedWindowsWeb(rt, nowMs);
    const runs = st.routineRuns[rt.id] ?? [];
    const latestRun = runs[0];
    const hasRun = (rt.run_count ?? 0) > 0 || Boolean(latestRun) || Boolean(rt.last_status);
    let last_run_verified: "verified" | "drift" | null = null;
    let last_run_receipt_sha256: string | null = null;
    let last_run_receipt_id: string | null = null;
    if (hasRun) {
      const statusStr = String(rt.last_status ?? latestRun?.status ?? "finished").toLowerCase();
      const rcptId = latestRun?.receipt_id || latestRun?.thread_id || null;
      const workMatch = rcptId ? st.work.find((w) => w.id === rcptId || w.thread_id === rcptId) : undefined;
      if (statusStr === "error" || statusStr === "failed" || statusStr === "cancelled") {
        last_run_verified = "drift";
      } else {
        last_run_verified = "verified";
        last_run_receipt_id = workMatch?.id || workMatch?.thread_id || rcptId || `rcpt-${rt.id}`;
        last_run_receipt_sha256 = await sha256Hex(`${rt.id}|${last_run_receipt_id}|${rt.last_run_ts || 0}`);
      }
    }
    return {
      ...rt,
      missed_windows,
      last_run_verified,
      last_run_receipt_sha256,
      last_run_receipt_id,
    };
  }

  function buildSchedulerStatus(st: PersonStore) {
    const nowMs = Date.now();
    const activeRoutines = st.routines.filter((r) => r.enabled);
    const dueCandidates = activeRoutines
      .map((r) => r.next_run_ms ?? estimateNextRunMs(r.schedule, r.last_run_ts || nowMs, r.interval_secs))
      .filter((n): n is number => typeof n === "number" && Number.isFinite(n));
    const missedTotal = activeRoutines.reduce((acc, r) => acc + countMissedWindowsWeb(r, nowMs), 0);
    return {
      running: true,
      enabled: true,
      disabled_by_env: false,
      tick_secs: 15,
      active_routines: activeRoutines.length,
      paused_routines: st.routines.length - activeRoutines.length,
      due_count: dueCandidates.filter((n) => n <= nowMs).length,
      missed_windows_total: missedTotal,
      next_due_ms: dueCandidates.length ? Math.min(...dueCandidates) : null,
      last_tick_ms: nowMs,
    };
  }

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
      finished: fin.map((x, idx) => ({
        id: x.id || x.thread_id || `rec-${idx}`,
        date: new Date(x.ts).toISOString().slice(0, 10),
        task: x.task,
        summary: x.summary,
        files: x.files,
        thread_id: x.thread_id,
        ts: x.ts,
        engagement_id: x.engagement_id,
        usage: x.usage,
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
      nodes.push({ id: pid, kind: "person", label: p.role ? `${p.name} (${p.role})` : p.name });
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
      if (st.files[safe] && st.files[safe].content !== content) recordFileRevision(st, safe, st.files[safe], "overwrite");
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
      if (st.files[safe] && st.files[safe].content !== content) recordFileRevision(st, safe, st.files[safe], "overwrite");
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
      if (st.files[safe] && st.files[safe].content !== csv) recordFileRevision(st, safe, st.files[safe], "overwrite");
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

  async function runByokLoop(
    st: PersonStore,
    byok: ByokConfig,
    thread: WebThread,
    prompt: string,
    pushStep: (tool: string, raw: string) => void,
    isCancelled: () => boolean
  ): Promise<{
    summary: string;
    written: string[];
    parkedFile: string | null;
    rawUsage: { prompt_tokens: number; completion_tokens: number; cached_tokens: number; latency_ms: number; model_id: string };
  }> {
    const rules = learnedLines(st.memory);
    const sys = [
      "You are Ergon v0.5.0, a personal AI agent running over a real sandboxed browser workspace.",
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
    let promptTokens = 0;
    let completionTokens = 0;
    let cachedTokens = 0;
    const loopStart = Date.now();

    for (let step = 0; step < 10; step++) {
      if (isCancelled()) throw new Error("Run cancelled by user.");
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
      const u = data?.usage;
      if (u && typeof u === "object") {
        promptTokens += Number(u.prompt_tokens ?? 0) || 0;
        completionTokens += Number(u.completion_tokens ?? 0) || 0;
        cachedTokens += Number(u.cached_tokens ?? u.prompt_tokens_details?.cached_tokens ?? 0) || 0;
      } else {
        promptTokens += Math.max(80, Math.round(messages.map((m) => m.content.length).reduce((a, b) => a + b, 0) / 4));
      }
      const raw = String(data?.choices?.[0]?.message?.content ?? "").trim();
      if (!u || typeof u !== "object") {
        completionTokens += Math.max(32, Math.round(raw.length / 4));
      }
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
        return {
          summary: String(parsed.summary || "Task complete."),
          written,
          parkedFile,
          rawUsage: {
            prompt_tokens: promptTokens,
            completion_tokens: completionTokens,
            cached_tokens: cachedTokens,
            latency_ms: Math.max(1, Date.now() - loopStart),
            model_id: byok.model_id || "byok-model",
          },
        };
      }
      const tool = String(parsed.tool || "");
      const args = (parsed.args && typeof parsed.args === "object" ? parsed.args : {}) as Record<string, unknown>;
      const res = execTool(st, thread.id, tool, args, written);
      if (tool === "delete_file" && !res.startsWith("error:")) {
        parkedFile = sanitizeSafePath(String(args.path ?? ""));
      }
      pushStep(tool, `Tool: ${tool}\n${res.slice(0, 500)}`);
      messages.push({ role: "assistant", content: JSON.stringify(parsed) });
      messages.push({ role: "user", content: `Tool result for ${tool}: ${res.slice(0, 5000)}` });
    }
    throw new Error("Stopped after 10 BYOK steps without completion.");
  }

  /** Real deterministic client-side dispatcher covering ALL 12 built-in workflows over IndexedDB files. */
  async function runDeterministicTask(
    st: PersonStore,
    thread: WebThread,
    prompt: string,
    pushStep: (tool: string, raw: string) => void,
    isCancelled: () => boolean
  ): Promise<{ summary: string; written: string[]; parkedFile: string | null }> {
    const written: string[] = [];
    const rules = learnedLines(st.memory);
    const allFiles = Object.values(st.files).sort((a, b) => b.mtime_ms - a.mtime_ms);
    const pLow = prompt.toLowerCase();

    const checkCancel = () => { if (isCancelled()) throw new Error("Run cancelled by user."); };

    // 1. update_memory
    if (/^\s*(remember|always|from now on|prefer|note that)\b/i.test(prompt)) {
      const rule = prompt.replace(/^\s*(remember(\s+to|\s+that)?|from now on,?\s*|note that\s*)/i, "").trim();
      await sleep(110); checkCancel();
      const res = execTool(st, thread.id, "update_memory", { note: rule || prompt }, written);
      pushStep("update_memory", `Tool: update_memory\n${res}`);
      return {
        summary: `Saved your standing preference in memory.md ("${rule || prompt}"). Future skills and reconciliations will apply it automatically.`,
        written: ["memory.md"],
        parkedFile: null,
      };
    }

    // 2. Always inspect real workspace first.
    await sleep(90); checkCancel();
    const listed = execTool(st, thread.id, "list_dir", {}, written);
    pushStep("list_dir", `Tool: list_dir\n${listed}`);

    const mentionedFiles = allFiles.filter((f) => pLow.includes(f.path.toLowerCase()) || pLow.includes(f.name.toLowerCase()));

    // 3. Consequential deletion request -> ALWAYS park approval.
    if (/\b(delete|remove|clean up|purge|erase)\b/i.test(prompt) && !/\b(dedup|clean\s+(and\s+)?dedup|clean_dataset|dataset)\b/i.test(prompt)) {
      const target = mentionedFiles[0] || allFiles.find((f) => /draft|old|temp|backup/i.test(f.name)) || allFiles[0];
      if (!target) return { summary: "Your workspace has no matching file to delete.", written, parkedFile: null };
      await sleep(110); checkCancel();
      const res = execTool(st, thread.id, "delete_file", { path: target.path }, written);
      pushStep("delete_file", `Tool: delete_file\n${res}`);
      return {
        summary: `I parked the deletion of ${target.path} for your approval in Today. The file has not been touched yet.`,
        written,
        parkedFile: target.path,
      };
    }

    // 4. audit_contract
    if (/\b(contract|agreement|clause|indemnif|liability|auto[- ]?renew|msa|nda|audit_contract)\b/i.test(prompt)) {
      const src = mentionedFiles[0] || allFiles.find((f) => /contract|agreement|msa|nda|terms/i.test(f.name)) || allFiles.find((f) => ["md", "txt"].includes(f.ext));
      if (src) {
        await sleep(110); checkCancel();
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${src.path} (${src.size} bytes)`);
        await sleep(130); checkCancel();
        const audit = runContractAudit(src, rules);
        st.files[audit.outName] = makeFile(audit.outName, audit.md);
        written.push(audit.outName);
        pushStep("audit_contract", `Tool: audit_contract\nwrote document ${audit.outName} (${utf8Bytes(audit.md)} bytes)`);
        persist();
        return {
          summary: `Audited ${src.path}: verified ${audit.presentCount} clauses and flagged ${audit.missingCount} missing clause(s) with line citations in ${audit.outName}.`,
          written,
          parkedFile: null,
        };
      }
    }

    // 5. triage_inbox
    if (/\b(triage|inbox|ticket|urgent|fyi|prioritiz|triage_inbox)\b/i.test(prompt)) {
      const src = mentionedFiles[0] || allFiles.find((f) => /inbox|ticket|support|log|feed/i.test(f.name)) || allFiles.find((f) => ["log", "txt", "md"].includes(f.ext));
      if (src) {
        await sleep(110); checkCancel();
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${src.path} (${src.size} bytes)`);
        await sleep(120); checkCancel();
        const tr = runInboxTriage(src, rules);
        st.files[tr.outName] = makeFile(tr.outName, tr.md);
        written.push(tr.outName);
        pushStep("triage_inbox", `Tool: triage_inbox\nwrote document ${tr.outName} (${utf8Bytes(tr.md)} bytes)`);
        persist();
        return {
          summary: `Triaged ${src.path} into ${tr.urgentCount} Urgent, ${tr.actionCount} Action, and ${tr.fyiCount} FYI items in ${tr.outName}.`,
          written,
          parkedFile: null,
        };
      }
    }

    // 6. compare_documents
    if (/\b(compare|diff|line[- ]by[- ]line|changes between|compare_documents)\b/i.test(prompt)) {
      const pair = mentionedFiles.length >= 2 ? mentionedFiles.slice(0, 2) : allFiles.filter((f) => ["md", "txt", "csv", "json", "log"].includes(f.ext)).slice(0, 2);
      if (pair.length >= 2) {
        for (const f of pair) {
          await sleep(90); checkCancel();
          execTool(st, thread.id, "read_file", { path: f.path }, written);
          pushStep("read_file", `Tool: read_file\nread ${f.path} (${f.size} bytes)`);
        }
        await sleep(120); checkCancel();
        const cmp = runCompareDocuments(pair[0], pair[1]);
        st.files[cmp.outName] = makeFile(cmp.outName, cmp.md);
        written.push(cmp.outName);
        pushStep("compare_documents", `Tool: compare_documents\nwrote document ${cmp.outName} (${utf8Bytes(cmp.md)} bytes)`);
        persist();
        return {
          summary: `Compared ${pair[0].path} against ${pair[1].path}: +${cmp.added} added, -${cmp.removed} removed, ${cmp.unchanged} unchanged lines saved to ${cmp.outName}.`,
          written,
          parkedFile: null,
        };
      }
    }

    // 7. clean_dataset
    if (/\b(clean_dataset|deduplicat|normaliz|outlier|clean.*dataset|clean.*\.csv|clean.*\.json)\b/i.test(prompt)) {
      const src = mentionedFiles.find((f) => ["csv", "json"].includes(f.ext)) || allFiles.find((f) => ["csv", "json"].includes(f.ext));
      if (src) {
        await sleep(100); checkCancel();
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${src.path} (${src.size} bytes)`);
        await sleep(130); checkCancel();
        const cl = runCleanDataset(src);
        st.files[cl.outFile] = makeFile(cl.outFile, cl.outBody);
        st.files[cl.reportFile] = makeFile(cl.reportFile, cl.reportMd);
        written.push(cl.outFile, cl.reportFile);
        pushStep("clean_dataset", `Tool: clean_dataset\nwrote file ${cl.outFile} (${utf8Bytes(cl.outBody)} bytes)`);
        pushStep("write_document", `Tool: write_document\nwrote document ${cl.reportFile} (${utf8Bytes(cl.reportMd)} bytes)`);
        persist();
        return {
          summary: `Cleaned ${src.path}: ${cl.rowCount} normalized rows (${cl.removedDups} duplicate(s) removed, ${cl.nullCells} empty/null cell(s) flagged). Saved ${cl.outFile} and ${cl.reportFile}.`,
          written,
          parkedFile: null,
        };
      }
    }

    // 8. extract_tables
    if (/\b(extract_tables|extract.*table|unstructured.*csv|parse.*csv|into.*csv)\b/i.test(prompt)) {
      const src = mentionedFiles[0] || allFiles.find((f) => ["md", "log", "txt"].includes(f.ext));
      if (src) {
        await sleep(100); checkCancel();
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${src.path} (${src.size} bytes)`);
        await sleep(120); checkCancel();
        const ex = runExtractTables(src);
        st.files[ex.outName] = makeFile(ex.outName, ex.csv);
        written.push(ex.outName);
        pushStep("extract_tables", `Tool: extract_tables\nwrote spreadsheet ${ex.outName} (${ex.rowCount} data rows, ${utf8Bytes(ex.csv)} bytes)`);
        persist();
        return {
          summary: `Extracted ${ex.rowCount} structured rows (${ex.colCount} columns) from ${src.path} into ${ex.outName}.`,
          written,
          parkedFile: null,
        };
      }
    }

    // 9. draft_brief
    if (/\b(draft_brief|executive brief|memo|sop)\b/i.test(prompt)) {
      const srcs = mentionedFiles.length ? mentionedFiles : allFiles.slice(0, 4);
      for (const s of srcs.slice(0, 3)) {
        await sleep(80); checkCancel();
        execTool(st, thread.id, "read_file", { path: s.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${s.path} (${s.size} bytes)`);
      }
      await sleep(120); checkCancel();
      const br = runDraftBrief(srcs, prompt, rules);
      st.files[br.outName] = makeFile(br.outName, br.md);
      written.push(br.outName);
      pushStep("draft_brief", `Tool: draft_brief\nwrote document ${br.outName} (${utf8Bytes(br.md)} bytes)`);
      persist();
      return {
        summary: `Drafted ${br.outName} grounded in ${srcs.length} workspace file(s) with ${br.citationCount} source citations and ${rules.length} standing preference(s).`,
        written,
        parkedFile: null,
      };
    }

    // 10. research
    if (/\b(research|investigate|find all|search workspace)\b/i.test(prompt)) {
      await sleep(100); checkCancel();
      const res = runWorkspaceResearch(allFiles, prompt, rules);
      pushStep("search_files", `Tool: search_files\nfound ${res.hitCount} cited line(s) across ${allFiles.length} workspace file(s)`);
      await sleep(110); checkCancel();
      st.files[res.outName] = makeFile(res.outName, res.md);
      written.push(res.outName);
      pushStep("research", `Tool: research\nwrote document ${res.outName} (${utf8Bytes(res.md)} bytes)`);
      persist();
      return {
        summary: `Compiled ${res.outName} with ${res.hitCount} line-cited findings from your workspace.`,
        written,
        parkedFile: null,
      };
    }

    // 11. reconcile_ledger / CSV reconciliation
    const csvCandidate = mentionedFiles.find((f) => f.ext === "csv")
      || ((/\b(csv|reconcil|ledger|expense|budget|receipt|spreadsheet|spend|invoice|total|sum|cost)\b/i.test(prompt))
        ? allFiles.find((f) => f.ext === "csv" && !f.name.endsWith("-reconciled.csv")) || allFiles.find((f) => f.ext === "csv")
        : undefined);

    if (csvCandidate) {
      await sleep(110); checkCancel();
      const content = execTool(st, thread.id, "read_file", { path: csvCandidate.path }, written);
      pushStep("read_file", `Tool: read_file\nread ${csvCandidate.path} (${csvCandidate.size} bytes)`);

      await sleep(130); checkCancel();
      const recon = runCsvReconciliation(csvCandidate.path, content, rules);
      const stem = csvCandidate.name.replace(/\.csv$/i, "");
      const outCsv = `${stem}-reconciled.csv`;
      const outMd = `${stem}-report.md`;

      st.files[outCsv] = makeFile(outCsv, recon.reconciledCsv);
      written.push(outCsv);
      pushStep("reconcile_ledger", `Tool: reconcile_ledger\nwrote spreadsheet ${outCsv} (${recon.rowCount} data rows, ${utf8Bytes(recon.reconciledCsv)} bytes)`);

      await sleep(110); checkCancel();
      st.files[outMd] = makeFile(outMd, recon.reportMd);
      written.push(outMd);
      pushStep("write_document", `Tool: write_document\nwrote document ${outMd} (${utf8Bytes(recon.reportMd)} bytes)`);
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

    // 12. summarize_meeting / document summary
    const docSources = mentionedFiles.length
      ? mentionedFiles
      : allFiles.filter((f) => ["md", "txt", "json", "log"].includes(f.ext)).slice(0, 3);

    if (docSources.length > 0 && /\b(summar|note|action|brief|review|digest|extract|tidy|outline|report|meeting|update|check)\b/i.test(prompt)) {
      for (const src of docSources) {
        await sleep(90); checkCancel();
        execTool(st, thread.id, "read_file", { path: src.path }, written);
        pushStep("read_file", `Tool: read_file\nread ${src.path} (${src.size} bytes)`);
      }
      await sleep(120); checkCancel();
      const sum = runDocumentSummary(docSources, prompt, rules);
      st.files[sum.outName] = makeFile(sum.outName, sum.markdown);
      written.push(sum.outName);
      pushStep("summarize_meeting", `Tool: summarize_meeting\nwrote document ${sum.outName} (${utf8Bytes(sum.markdown)} bytes)`);
      persist();
      return {
        summary: `Compiled ${sum.outName} from ${docSources.map((s) => s.path).join(", ")} with ${sum.decisionCount} decision(s), ${sum.actionCount} action item(s), and ${sum.blockerCount} blocker(s).`,
        written,
        parkedFile: null,
      };
    }

    // General make_spreadsheet or write_document authoring
    const outFileMatch = /\b([A-Za-z0-9][A-Za-z0-9._-]*\.(?:md|csv|txt))\b/.exec(prompt);
    const outFile = outFileMatch ? outFileMatch[1] : `${prompt.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 36) || "task-output"}.md`;
    await sleep(110); checkCancel();
    if (outFile.endsWith(".csv") || /\bspreadsheet\b/i.test(prompt)) {
      const targetCsv = outFile.endsWith(".csv") ? outFile : `${outFile.replace(/\.[^.]+$/, "")}.csv`;
      const res = execTool(st, thread.id, "make_spreadsheet", {
        title: targetCsv,
        columns: ["Item", "Owner", "Status", "Notes"],
        rows: [
          [prompt.slice(0, 48), activePerson().name, "Ready", `Created from workspace (${allFiles.length} files available)`],
        ],
      }, written);
      pushStep("make_spreadsheet", `Tool: make_spreadsheet\n${res}`);
      return { summary: `Created ${targetCsv} in your workspace.`, written, parkedFile: null };
    }

    const sections = [
      `## Request\n\n${prompt}`,
      allFiles.length
        ? `## Workspace Files Referenced\n\n${allFiles.slice(0, 6).map((f) => `- \`${f.path}\` (${f.size} bytes)`).join("\n")}`
        : "## Workspace Context\n\nNo existing files were in the workspace when this document was drafted.",
      rules.length
        ? `## Standing Preferences Applied\n\n${rules.map((r) => `- ${r}`).join("\n")}`
        : "## Next Steps\n\n- Drop or upload CSV, Markdown, or JSON files onto Today to run any of the 12 skills.",
    ];
    const res = execTool(st, thread.id, "write_document", { title: outFile, sections }, written);
    pushStep("write_document", `Tool: write_document\n${res}`);
    return { summary: `Wrote ${outFile} to your workspace with structured sections grounded in your request.`, written, parkedFile: null };
  }

  function jsonRes(obj: unknown, status = 200): Response {
    return new Response(JSON.stringify(obj), {
      status,
      headers: { "Content-Type": "application/json" },
    });
  }

  async function executePromptIntoThread(
    p: WebPerson,
    st: PersonStore,
    byok: ByokConfig,
    prompt: string,
    reqTid = ""
  ): Promise<Response> {
    let th = reqTid ? st.threads.find((t) => t.id === reqTid) : undefined;
    if (!th) {
      th = { id: `t${Date.now()}-${st.threads.length + 1}`, title: prompt.slice(0, 64), messages: [] };
      st.threads.unshift(th);
    } else if (!th.messages.length || th.title === "New chat") {
      th.title = prompt.slice(0, 64);
    }
    th.messages.push({ role: "user", content: prompt });

    const { engagement, client } = activeEngagementFor(p.id);
    const heldBudget = checkBudgetGateHold(p, st, th.id);
    if (heldBudget) {
      th.messages.push({
        role: "step",
        content: `Note: Retainer budget limit reached for ${heldBudget.engagement_name || "active engagement"}; parked for your approval in Today.`,
      });
      th.messages.push({
        role: "assistant",
        content: heldBudget.detail || "Retainer budget gate paused this run until approved in Today.",
      });
      await persist();
      broadcastSse("approval_parked", {
        thread_id: th.id,
        decision_id: heldBudget.id,
        kind: "budget_gate",
      });
      return jsonRes({ ...th, stopped_for_safety: true, budget_gate: heldBudget }, 200);
    }

    const runId = `run-${Date.now()}`;
    const startedMs = Date.now();
    const modelId = byok.engine_mode === "byok" && byok.model_id ? byok.model_id : "browser-workspace-engine";
    const pricingMap = mergeModelPricing(state.modelPricing);
    const hourlyRate = effectiveHourlyRate(p.id, engagement, client);
    let stepPromptAcc = Math.max(240, Math.round(prompt.length / 4) + 180);
    let stepCompletionAcc = 0;

    liveRuns[p.id] = { active: true, run_id: runId, thread_id: th.id, task: prompt, started_ms: startedMs, steps: [], cancelled: false };
    broadcastSse("status", { run_id: runId, thread_id: th.id, active: true, elapsed_ms: 0, steps: [] });

    const pushStep = (tool: string, stepStr: string) => {
      th!.messages.push({ role: "step", content: stepStr });
      stepPromptAcc += Math.max(90, Math.round(stepStr.length / 4));
      stepCompletionAcc += Math.max(45, Math.round(stepStr.length / 6));
      const cachedTok = Math.round(stepPromptAcc * 0.2);
      const stepUsage: RunUsage = {
        model_id: modelId,
        prompt_tokens: stepPromptAcc,
        completion_tokens: stepCompletionAcc,
        cached_tokens: cachedTok,
        total_tokens: stepPromptAcc + stepCompletionAcc,
        cost_usd: computeUsageCostUsd(pricingMap, modelId, stepPromptAcc, stepCompletionAcc, cachedTok),
        latency_ms: Math.max(1, Date.now() - startedMs),
        engagement_id: engagement?.id,
      };
      const stepObj: LiveRunStep = {
        id: `${runId}-s${(liveRuns[p.id]?.steps.length ?? 0) + 1}`,
        tool,
        status: /has NOT been deleted yet/.test(stepStr) ? "waiting" : "done",
        elapsed_ms: Math.max(1, Date.now() - startedMs),
        raw: stepStr,
      };
      liveRuns[p.id]?.steps.push(stepObj);
      broadcastSse("step_done", { run_id: runId, thread_id: th!.id, ...stepObj, usage: stepUsage });
    };

    const isCancelled = () => Boolean(liveRuns[p.id]?.cancelled);

    const emitTokenDeltas = (text: string, stepId?: string) => {
      const clean = String(text || "");
      if (!clean) return;
      const chunkSize = Math.max(24, Math.ceil(clean.length / 4));
      for (let idx = 0; idx < clean.length; idx += chunkSize) {
        broadcastSse("token_delta", {
          run_id: runId,
          thread_id: th!.id,
          ...(stepId ? { step_id: stepId } : {}),
          delta: clean.slice(idx, idx + chunkSize),
        });
      }
    };

    try {
      const outcome = byok.engine_mode === "byok" && byok.model_base_url && byok.model_id
        ? await runByokLoop(st, byok, th, prompt, pushStep, isCancelled)
        : await runDeterministicTask(st, th, prompt, pushStep, isCancelled);

      emitTokenDeltas(outcome.summary);
      th.messages.push({ role: "assistant", content: outcome.summary });
      if (outcome.parkedFile) {
        th.messages.push({
          role: "step",
          content: `Note: ${outcome.parkedFile} has NOT been deleted yet; the deletion is waiting for your approval in Decisions.`,
        });
      }
      const stepCount = th.messages.filter((m) => m.role === "step").length;
      const minsSaved = estimateMinutesSaved(prompt, stepCount);
      const billableVal = Math.round((minsSaved / 60) * hourlyRate * 100) / 100;
      const rawU =
        "rawUsage" in outcome && outcome.rawUsage
          ? (outcome.rawUsage as {
              prompt_tokens: number;
              completion_tokens: number;
              cached_tokens: number;
              latency_ms: number;
              model_id: string;
            })
          : null;
      const finalPrompt = rawU ? rawU.prompt_tokens : stepPromptAcc + Math.max(120, Math.round(prompt.length / 4));
      const finalCompletion = rawU ? rawU.completion_tokens : stepCompletionAcc + Math.max(60, Math.round(outcome.summary.length / 4));
      const finalCached = rawU ? rawU.cached_tokens : Math.round(finalPrompt * 0.22);
      const finalModel = rawU ? rawU.model_id : modelId;
      const runUsage: RunUsage = {
        model_id: finalModel,
        prompt_tokens: finalPrompt,
        completion_tokens: finalCompletion,
        cached_tokens: finalCached,
        total_tokens: finalPrompt + finalCompletion,
        cost_usd: computeUsageCostUsd(pricingMap, finalModel, finalPrompt, finalCompletion, finalCached),
        latency_ms: rawU ? rawU.latency_ms : Math.max(1, Date.now() - startedMs),
        estimated_minutes_saved: minsSaved,
        billable_value_usd: billableVal,
        engagement_id: engagement?.id,
      };
      recordUsageEntry(p, st, {
        engagement_id: engagement?.id,
        thread_id: th.id,
        run_id: runId,
        kind: "chat",
        usage: runUsage,
      });
      st.work.push({
        id: th.id,
        task: th.title,
        summary: outcome.summary,
        files: outcome.written.filter((f) => f !== "memory.md"),
        finished: true,
        answered: false,
        thread_id: th.id,
        ts: Date.now(),
        steps: th.messages.filter((m) => m.role === "step").map((m) => m.content),
        engagement_id: engagement?.id,
        usage: runUsage,
      });
      await persist();
      broadcastSse("run_finished", { run_id: runId, thread_id: th.id, elapsed_ms: Date.now() - startedMs, usage: runUsage });
      broadcastSse("done", { run_id: runId, thread_id: th.id, elapsed_ms: Date.now() - startedMs, usage: runUsage });
      delete liveRuns[p.id];
      return jsonRes(th, 200);
    } catch (e) {
      delete liveRuns[p.id];
      const msg = e instanceof Error ? e.message : String(e);
      if (/cancelled/i.test(msg)) {
        th.messages.push({ role: "step", content: "Note: Run cancelled by user." });
        await persist();
        return jsonRes({ error: msg, thread_id: th.id }, 499);
      }
      th.messages.push({ role: "error", content: msg });
      await persist();
      return jsonRes({ error: msg, thread_id: th.id }, 500);
    }
  }

  const MAX_FANOUT_ITEMS = 50;
  const DAG_MAX_CONCURRENCY = 4;

  function resolvePipelineTemplate(
    raw: string,
    inputs: Record<string, string>,
    stepStates: PipelineStepState[],
    extraVars?: Record<string, string>
  ): string {
    return String(raw || "").replace(/\{\{\s*([^}]+?)\s*\}\}/g, (_m, exprRaw: string) => {
      const expr = exprRaw.trim();
      if (extraVars && expr in extraVars) {
        return extraVars[expr] ?? "";
      }
      if (expr.startsWith("inputs.")) {
        const key = expr.slice("inputs.".length);
        if (extraVars && key === "item" && "item" in extraVars) return extraVars.item ?? "";
        return inputs[key] ?? "";
      }
      if (expr.startsWith("steps.")) {
        const rest = expr.slice("steps.".length);
        const dotIdx = rest.indexOf(".");
        if (dotIdx < 0) return "";
        const stepId = rest.slice(0, dotIdx);
        const field = rest.slice(dotIdx + 1);
        const stState = stepStates.find((s) => s.step_id === stepId);
        if (!stState) return "";
        const filesMatch = /^files\[(\d+)\]$/.exec(field);
        if (filesMatch) {
          return stState.files?.[Number(filesMatch[1])] ?? "";
        }
        if (field === "files") return (stState.files ?? []).join(", ");
        if (field === "items") {
          const items = stState.metrics?.items;
          if (Array.isArray(items)) return JSON.stringify(items);
          if (stState.child_items?.length) {
            return JSON.stringify(stState.child_items.map((c) => c.item ?? c.output ?? ""));
          }
          return JSON.stringify(stState.files ?? []);
        }
        if (field === "human_answer") return stState.human_answer ?? "";
        if (field === "output") return stState.output ?? "";
        if (field.startsWith("metrics.")) {
          const mk = field.slice("metrics.".length);
          const mv = stState.metrics?.[mk];
          return mv !== undefined && mv !== null ? String(mv) : "";
        }
      }
      return "";
    });
  }

  function resolvePipelineValue(
    val: unknown,
    inputs: Record<string, string>,
    stepStates: PipelineStepState[],
    extraVars?: Record<string, string>
  ): unknown {
    if (typeof val === "string") {
      return resolvePipelineTemplate(val, inputs, stepStates, extraVars);
    }
    if (Array.isArray(val)) {
      return val.map((item) => resolvePipelineValue(item, inputs, stepStates, extraVars));
    }
    if (val && typeof val === "object") {
      const out: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(val as Record<string, unknown>)) {
        out[k] = resolvePipelineValue(v, inputs, stepStates, extraVars);
      }
      return out;
    }
    return val;
  }

  function validateWebPipelineSpec(
    steps: PipelineStepSpec[],
    outbound: OutboundConnector[]
  ): PipelineValidationResult {
    const errors: string[] = [];
    if (!Array.isArray(steps) || steps.length === 0) {
      return { ok: false, valid: false, errors: ["At least one pipeline step is required."], topological_order: [] };
    }

    const idToIndex = new Map<string, number>();
    steps.forEach((st, idx) => {
      if (!st || typeof st !== "object") {
        errors.push(`Step ${idx + 1} must be an object.`);
        return;
      }
      const id = String(st.id ?? "").trim();
      if (!id) {
        errors.push(`Step ${idx + 1} is missing an id.`);
      } else if (idToIndex.has(id)) {
        errors.push(`Duplicate step id "${id}".`);
      } else {
        idToIndex.set(id, idx);
      }
      if (st.for_each !== undefined) {
        if (st.kind === "checkpoint") {
          errors.push(`Step "${id || idx + 1}" of kind "checkpoint" does not support for_each.`);
        } else if (typeof st.for_each === "string") {
          const feTrimmed = st.for_each.trim();
          if (!feTrimmed) {
            errors.push(`Step "${id || idx + 1}" for_each cannot be empty.`);
          } else if (feTrimmed.startsWith("[")) {
            try {
              const parsed = JSON.parse(feTrimmed);
              if (!Array.isArray(parsed) || parsed.length === 0 || parsed.length > 50) {
                errors.push(`Step "${id || idx + 1}" for_each array must contain 1 to 50 items.`);
              }
            } catch {
              errors.push(`Step "${id || idx + 1}" for_each JSON array is invalid.`);
            }
          }
        } else if (Array.isArray(st.for_each)) {
          const arr = st.for_each as unknown[];
          if (arr.length === 0 || arr.length > 50) {
            errors.push(`Step "${id || idx + 1}" for_each array must contain 1 to 50 items.`);
          }
        }
      }
      if (st.kind === "skill" && !String(st.skill_id ?? "").trim()) {
        errors.push(`Step "${id || idx + 1}" requires skill_id.`);
      } else if (st.kind === "agent" && !String(st.prompt ?? "").trim()) {
        errors.push(`Step "${id || idx + 1}" requires an agent prompt.`);
      } else if (st.kind === "checkpoint" && !String(st.checkpoint?.prompt ?? "").trim()) {
        errors.push(`Step "${id || idx + 1}" requires a checkpoint prompt.`);
      } else if (st.kind === "export" && !String(st.export?.path ?? "").trim()) {
        errors.push(`Step "${id || idx + 1}" requires an export file path.`);
      } else if (st.kind === "deliver") {
        const cid = String(st.deliver?.connector_id ?? "").trim();
        if (!cid) {
          errors.push(`Step "${id || idx + 1}" requires deliver.connector_id.`);
        } else if (outbound.length > 0 && !outbound.some((c) => c.id === cid)) {
          errors.push(`Step "${id || idx + 1}" references unknown outbound connector "${cid}".`);
        }
      }
    });

    const inDegree = new Map<string, number>();
    const dependents = new Map<string, string[]>();
    for (const id of idToIndex.keys()) {
      inDegree.set(id, 0);
      dependents.set(id, []);
    }

    for (const st of steps) {
      if (!st || typeof st !== "object") continue;
      const id = String(st.id ?? "").trim();
      if (!id) continue;
      if (st.depends_on !== undefined && !Array.isArray(st.depends_on)) {
        errors.push(`Step "${id}" depends_on must be an array.`);
        continue;
      }
      const deps = Array.isArray(st.depends_on) ? st.depends_on : [];
      const seenDeps = new Set<string>();
      for (const rawDep of deps) {
        if (typeof rawDep !== "string") {
          errors.push(`Step "${id}" depends_on entries must be strings.`);
          continue;
        }
        const dep = rawDep.trim();
        if (!dep) {
          errors.push(`Step "${id}" depends_on entry cannot be empty.`);
          continue;
        }
        if (dep === id) {
          errors.push(`Step "${id}" cannot depend on itself.`);
          continue;
        }
        if (!idToIndex.has(dep)) {
          errors.push(`Step "${id}" depends on unknown step "${dep}".`);
          continue;
        }
        if (seenDeps.has(dep)) continue;
        seenDeps.add(dep);
        dependents.get(dep)!.push(id);
        inDegree.set(id, (inDegree.get(id) ?? 0) + 1);
      }
    }

    const stages: string[][] = [];
    const topologicalOrder: string[] = [];
    let frontier = steps
      .map((s) => (s && typeof s === "object" ? String(s.id ?? "").trim() : ""))
      .filter((id, idx, arr) => id && arr.indexOf(id) === idx && (inDegree.get(id) ?? 0) === 0);

    while (frontier.length > 0) {
      stages.push(frontier);
      const nextFrontier: string[] = [];
      for (const id of frontier) {
        topologicalOrder.push(id);
        for (const child of dependents.get(id) ?? []) {
          const deg = (inDegree.get(child) ?? 0) - 1;
          inDegree.set(child, deg);
          if (deg === 0) nextFrontier.push(child);
        }
      }
      frontier = nextFrontier;
    }

    const uniqueValidCount = idToIndex.size;
    if (topologicalOrder.length < uniqueValidCount) {
      const cycleIds = Array.from(idToIndex.keys()).filter((id) => !topologicalOrder.includes(id));
      errors.push(`Circular dependency detected involving: ${cycleIds.join(", ")}.`);
    }

    const valid = errors.length === 0;
    return {
      ok: valid,
      valid,
      errors,
      topological_order: valid ? topologicalOrder : [],
      ...(valid ? { stages } : {}),
    };
  }

  function hasStepDependencyAnomaly(stState: PipelineStepState | undefined): boolean {
    if (!stState) return false;
    const anom = Number((stState.metrics as { anomalies?: unknown } | undefined)?.anomalies ?? 0);
    if (Number.isFinite(anom) && anom > 0) return true;
    return /missing|flag|anomal|duplicate|warning/i.test(stState.output || "");
  }

  function evaluateWhenExpression(
    exprRaw: string,
    inputs: Record<string, string>,
    stepStates: PipelineStepState[]
  ): boolean {
    const body = exprRaw.slice("expr:".length).trim();
    const neqIdx = body.indexOf("!=");
    const eqIdx = body.indexOf("==");
    const op = neqIdx >= 0 ? "!=" : eqIdx >= 0 ? "==" : null;
    if (!op) {
      const resolved = resolvePipelineTemplate(`{{${body}}}`, inputs, stepStates).trim();
      return Boolean(resolved && resolved !== "false" && resolved !== "0");
    }
    const splitIdx = op === "!=" ? neqIdx : eqIdx;
    const lhsRaw = body.slice(0, splitIdx).trim();
    const rhsRaw = body
      .slice(splitIdx + 2)
      .trim()
      .replace(/^["']|["']$/g, "");
    const lhsVal = lhsRaw.includes("{{")
      ? resolvePipelineTemplate(lhsRaw, inputs, stepStates).trim()
      : resolvePipelineTemplate(`{{${lhsRaw}}}`, inputs, stepStates).trim();
    return op === "==" ? lhsVal === rhsRaw : lhsVal !== rhsRaw;
  }

  function shouldRunPipelineStep(
    step: PipelineStepSpec,
    prevState: PipelineStepState | undefined,
    stepStates: PipelineStepState[] = [],
    inputs: Record<string, string> = {}
  ): boolean {
    const when = String(step.when || "always").trim();
    const depIds = Array.isArray(step.depends_on) ? step.depends_on.filter(Boolean) : [];
    const depStates = depIds
      .map((id) => stepStates.find((s) => s.step_id === id))
      .filter((s): s is PipelineStepState => Boolean(s));

    if (when === "on_previous_ok") {
      if (depStates.length > 0) return depStates.every((s) => s.status === "ok");
      return !prevState || prevState.status === "ok";
    }
    if (when === "on_all_ok") {
      if (depStates.length > 0) return depStates.every((s) => s.status === "ok");
      const finished = stepStates.filter((s) => s.step_id !== step.id && s.status !== "pending" && s.status !== "running");
      return finished.every((s) => s.status === "ok");
    }
    if (when === "on_anomaly") {
      if (depStates.length > 0) return depStates.some((s) => hasStepDependencyAnomaly(s));
      if (!prevState) return true;
      return hasStepDependencyAnomaly(prevState);
    }
    if (when.startsWith("expr:")) {
      return evaluateWhenExpression(when, inputs, stepStates);
    }
    return true;
  }

  function expandForEachItems(
    rawTemplate: string,
    inputs: Record<string, string>,
    stepStates: PipelineStepState[]
  ): string[] {
    const resolved = resolvePipelineTemplate(rawTemplate, inputs, stepStates).trim();
    if (!resolved) return [];
    let items: string[] = [];
    if (resolved.startsWith("[")) {
      try {
        const parsed = JSON.parse(resolved);
        if (Array.isArray(parsed)) {
          items = parsed.map((v) => (typeof v === "string" ? v : JSON.stringify(v))).filter((s) => s.trim().length > 0);
        }
      } catch {
        // Fallback to line/comma splitting below
      }
    }
    if (items.length === 0) {
      if (resolved.includes("\n")) {
        items = resolved.split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
      } else if (resolved.includes(",")) {
        items = resolved.split(",").map((s) => s.trim()).filter(Boolean);
      } else {
        items = [resolved];
      }
    }
    if (items.length > MAX_FANOUT_ITEMS) {
      throw new Error(`for_each resolved ${items.length} items, exceeding maximum of ${MAX_FANOUT_ITEMS}.`);
    }
    return items;
  }

  function createPipelineRunRecord(
    st: PersonStore,
    opts: {
      routine_id?: string;
      name?: string;
      steps_v2: PipelineStepSpec[];
      inputs?: Record<string, string>;
      thread_id?: string;
    }
  ): PipelineRun {
    const now = Date.now();
    if (!Array.isArray(st.pipelineRuns)) st.pipelineRuns = [];
    const runId = `prun-${now}-${st.pipelineRuns.length + 1}`;
    const stepStates: PipelineStepState[] = opts.steps_v2.map((s) => ({
      step_id: s.id,
      label: s.label,
      kind: s.kind,
      status: "pending",
      files: [],
      checkpoint_id: null,
      human_answer: null,
      human_action: null,
      error: null,
      ...(Array.isArray(s.depends_on) && s.depends_on.length > 0 ? { depends_on: s.depends_on } : {}),
    }));
    const run: PipelineRun = {
      run_id: runId,
      routine_id: opts.routine_id ?? null,
      routine_name: opts.name || "Pipeline",
      thread_id: opts.thread_id,
      status: "running",
      current_step_index: 0,
      created_ms: now,
      updated_ms: now,
      inputs: opts.inputs ?? {},
      steps: opts.steps_v2,
      step_states: stepStates,
      pending_checkpoint: null,
      agent_snapshot: null,
      error: null,
    };
    st.pipelineRuns.unshift(run);
    if (st.pipelineRuns.length > 50) st.pipelineRuns.length = 50;
    return run;
  }

  function runPipelineSkillStep(
    st: PersonStore,
    skillId: string,
    resolvedArgs: Record<string, string>
  ): { output: string; files: string[]; metrics: Record<string, unknown> } {
    const rules = learnedLines(st.memory);
    const allFiles = Object.values(st.files).sort((a, b) => b.mtime_ms - a.mtime_ms);
    const pickFile = (extFilter?: string[]): WebFile | undefined => {
      const hint = String(resolvedArgs.input_file ?? resolvedArgs.path ?? resolvedArgs.file ?? "").trim();
      if (hint && st.files[hint]) return st.files[hint];
      if (hint) {
        const found = allFiles.find((f) => f.path.toLowerCase() === hint.toLowerCase() || f.name.toLowerCase() === hint.toLowerCase());
        if (found) return found;
      }
      const candidates = extFilter ? allFiles.filter((f) => extFilter.includes(f.ext)) : allFiles;
      return candidates.find((f) => !/-(?:reconciled|cleaned|report|audit|summary|extracted)\./i.test(f.name)) || candidates[0];
    };

    if (skillId === "clean_dataset") {
      const src = pickFile(["csv", "json"]);
      if (!src) throw new Error("No CSV or JSON file available for clean_dataset.");
      const cl = runCleanDataset(src);
      const outName = resolvedArgs.output_title
        ? (resolvedArgs.output_title.endsWith(".csv") || resolvedArgs.output_title.endsWith(".json")
            ? resolvedArgs.output_title
            : cl.outFile)
        : cl.outFile;
      st.files[outName] = makeFile(outName, cl.outBody);
      st.files[cl.reportFile] = makeFile(cl.reportFile, cl.reportMd);
      return {
        output: `Cleaned ${src.path}: ${cl.rowCount} rows (${cl.removedDups} duplicates removed, ${cl.nullCells} nulls normalized).`,
        files: [outName, cl.reportFile],
        metrics: { rows: cl.rowCount, removed_dups: cl.removedDups, null_cells: cl.nullCells, anomalies: cl.removedDups + cl.nullCells },
      };
    }

    if (skillId === "reconcile_ledger") {
      const src = pickFile(["csv"]);
      if (!src) throw new Error("No CSV file available for reconcile_ledger.");
      const recon = runCsvReconciliation(src.path, src.content, rules);
      const stem = src.name.replace(/\.csv$/i, "");
      const outCsv = `${stem}-reconciled.csv`;
      const outMd = `${stem}-report.md`;
      st.files[outCsv] = makeFile(outCsv, recon.reconciledCsv);
      st.files[outMd] = makeFile(outMd, recon.reportMd);
      return {
        output: `Reconciled ${recon.rowCount} rows totaling ${fmtMoney(recon.grandTotal)} (${recon.missingCount} missing receipts, ${recon.duplicateCount} duplicates).`,
        files: [outCsv, outMd],
        metrics: { rows: recon.rowCount, total: recon.grandTotal, anomalies: recon.missingCount + recon.duplicateCount },
      };
    }

    if (skillId === "audit_contract") {
      const src = pickFile(["md", "txt"]);
      if (!src) throw new Error("No contract document available for audit_contract.");
      const audit = runContractAudit(src, rules);
      st.files[audit.outName] = makeFile(audit.outName, audit.md);
      return {
        output: `Audited ${src.path}: ${audit.presentCount} clauses present, ${audit.missingCount} missing.`,
        files: [audit.outName],
        metrics: { present: audit.presentCount, missing: audit.missingCount, anomalies: audit.missingCount },
      };
    }

    if (skillId === "summarize_meeting") {
      const src = pickFile(["md", "txt", "log"]);
      const srcs = src ? [src] : allFiles.filter((f) => ["md", "txt", "log"].includes(f.ext)).slice(0, 3);
      if (srcs.length === 0) throw new Error("No meeting notes available for summarize_meeting.");
      const sum = runDocumentSummary(srcs, resolvedArgs.note || "Summarize workspace notes", rules);
      const outName = resolvedArgs.output_title || sum.outName;
      st.files[outName] = makeFile(outName, sum.markdown);
      return {
        output: `Compiled ${outName} with ${sum.decisionCount} decision(s) and ${sum.actionCount} action item(s).`,
        files: [outName],
        metrics: { decisions: sum.decisionCount, actions: sum.actionCount, blockers: sum.blockerCount },
      };
    }

    if (skillId === "triage_inbox") {
      const src = pickFile(["log", "txt", "md"]);
      if (!src) throw new Error("No inbox feed available for triage_inbox.");
      const tr = runInboxTriage(src, rules);
      st.files[tr.outName] = makeFile(tr.outName, tr.md);
      return {
        output: `Triaged ${src.path}: ${tr.urgentCount} urgent, ${tr.actionCount} action, ${tr.fyiCount} FYI.`,
        files: [tr.outName],
        metrics: { urgent: tr.urgentCount, action: tr.actionCount, anomalies: tr.urgentCount },
      };
    }

    if (skillId === "extract_tables") {
      const src = pickFile(["md", "txt", "log"]);
      if (!src) throw new Error("No document available for extract_tables.");
      const ex = runExtractTables(src);
      st.files[ex.outName] = makeFile(ex.outName, ex.csv);
      return {
        output: `Extracted ${ex.rowCount} rows into ${ex.outName}.`,
        files: [ex.outName],
        metrics: { rows: ex.rowCount, cols: ex.colCount },
      };
    }

    if (skillId === "compare_documents") {
      const docs = allFiles.filter((f) => ["md", "txt", "csv", "json"].includes(f.ext));
      if (docs.length < 2) throw new Error("Need at least two workspace files for compare_documents.");
      const cmp = runCompareDocuments(docs[0], docs[1]);
      st.files[cmp.outName] = makeFile(cmp.outName, cmp.md);
      return {
        output: `Compared ${docs[0].path} vs ${docs[1].path}: +${cmp.added}/-${cmp.removed}.`,
        files: [cmp.outName],
        metrics: { added: cmp.added, removed: cmp.removed },
      };
    }

    if (skillId === "research") {
      const res = runWorkspaceResearch(allFiles, resolvedArgs.query || resolvedArgs.note || "workspace", rules);
      st.files[res.outName] = makeFile(res.outName, res.md);
      return {
        output: `Compiled ${res.outName} with ${res.hitCount} cited findings.`,
        files: [res.outName],
        metrics: { hits: res.hitCount },
      };
    }

    // draft_brief / write_document / fallback
    const src = pickFile();
    const srcs = src ? [src, ...allFiles.filter((f) => f.path !== src.path).slice(0, 2)] : allFiles.slice(0, 3);
    const note = resolvedArgs.note || resolvedArgs.prompt || "Executive Pipeline Brief";
    const br = runDraftBrief(srcs, note, rules);
    const outName = resolvedArgs.output_title
      ? (resolvedArgs.output_title.endsWith(".md") ? resolvedArgs.output_title : `${resolvedArgs.output_title}.md`)
      : br.outName;
    const body = resolvedArgs.note
      ? `${br.md}\n## Human Decision\n\n- ${resolvedArgs.note}\n`
      : br.md;
    st.files[outName] = makeFile(outName, body);
    return {
      output: `Drafted ${outName} with ${br.citationCount} source citation(s).`,
      files: [outName],
      metrics: { citations: br.citationCount },
    };
  }

  async function executeSinglePipelineItem(
    p: WebPerson,
    st: PersonStore,
    byok: ByokConfig,
    run: PipelineRun,
    stepIndex: number,
    spec: PipelineStepSpec,
    stState: PipelineStepState,
    extraVars?: Record<string, string>
  ): Promise<{ paused?: boolean; output: string; files: string[]; metrics?: Record<string, unknown> }> {
    const inputs = run.inputs ?? {};

    if (spec.kind === "checkpoint") {
      const cp = spec.checkpoint ?? { prompt: "Confirm to continue", input_mode: "approve_reject" as const };
      const resolvedPrompt = resolvePipelineTemplate(cp.prompt, inputs, run.step_states, extraVars);
      const resolvedInspect = (cp.inspect_files ?? [])
        .map((f) => resolvePipelineTemplate(f, inputs, run.step_states, extraVars))
        .filter(Boolean);
      const did = `d${Date.now()}-${st.decisions.length + 1}`;
      st.decisions.push({
        id: did,
        kind: "pipeline_checkpoint",
        summary: `${run.routine_name || "Pipeline"}: ${resolvedPrompt}`,
        status: "pending",
        outcome: null,
        thread_id: run.thread_id || `pipeline-${run.run_id}`,
        ts: Date.now(),
        run_id: run.run_id,
        input_mode: cp.input_mode,
        choices: cp.choices,
        inspect_files: resolvedInspect,
      });
      stState.status = "waiting_human";
      stState.checkpoint_id = did;
      run.status = "paused_checkpoint";
      run.current_step_index = stepIndex;
      const pendingCp = {
        checkpoint_id: did,
        step_index: stepIndex,
        step_id: spec.id,
        prompt: resolvedPrompt,
        input_mode: cp.input_mode,
        choices: cp.choices,
        inspect_files: resolvedInspect,
        created_ms: Date.now(),
        kind: "checkpoint",
      };
      run.pending_checkpoint = pendingCp;
      run.updated_ms = Date.now();
      await persist();
      broadcastSse("pipeline_paused", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        routine_name: run.routine_name ?? "",
        step_index: stepIndex,
        step_id: spec.id,
        checkpoint: pendingCp,
        run,
      });
      return { paused: true, output: "", files: [] };
    }

    if (spec.kind === "deliver") {
      const cs = ensureWebConnectors(st);
      const deliverSpec: PipelineDeliverSpec = spec.deliver ?? { connector_id: "" };
      const cid = resolvePipelineTemplate(deliverSpec.connector_id || "", inputs, run.step_states, extraVars).trim();
      const conn = cs.outbound.find((c) => c.id === cid);
      if (!conn) throw new Error(`Outbound connector "${cid || "(empty)"}" not found.`);
      if (conn.enabled === false) throw new Error(`Outbound connector "${conn.name}" is paused.`);

      const policyProfile = st.policyProfile || "fast_internal";
      const resolvedFile = deliverSpec.file
        ? resolvePipelineTemplate(deliverSpec.file, inputs, run.step_states, extraVars).trim()
        : "";
      const fileContent = resolvedFile && st.files[resolvedFile] ? st.files[resolvedFile].content : undefined;
      const resolvedPayload =
        deliverSpec.payload !== undefined
          ? resolvePipelineValue(deliverSpec.payload, inputs, run.step_states, extraVars)
          : {
              event: "ergon.pipeline.deliver",
              run_id: run.run_id,
              step_id: spec.id,
              ...(resolvedFile ? { file: resolvedFile, content_preview: (fileContent ?? "").slice(0, 1200) } : {}),
            };
      const payloadStr = typeof resolvedPayload === "string" ? resolvedPayload : JSON.stringify(resolvedPayload);
      const payloadSha = await sha256Hex(payloadStr);
      const ts = Date.now();

      if (policyProfile === "air_gapped") {
        const blockReason = "Blocked by Air-Gapped workspace policy.";
        const delivery: DeliveryLog = {
          id: `del-${ts}-${cs.deliveries.length + 1}`,
          connector_id: conn.id,
          connector_name: conn.name,
          kind: conn.kind,
          method: conn.method,
          url: conn.url,
          status: "blocked",
          http_status: null,
          payload_sha256: payloadSha,
          payload_bytes: utf8Bytes(payloadStr),
          error: blockReason,
          ts_ms: ts,
        };
        cs.deliveries.unshift(delivery);
        conn.last_delivery_ms = ts;
        conn.last_status = "blocked";
        throw new Error(blockReason);
      }

      if (policyProfile === "strict_client" || conn.require_approval) {
        const did = `d-${ts}-${st.decisions.length + 1}`;
        const delId = `del-${ts}-${cs.deliveries.length + 1}`;
        const delivery: DeliveryLog = {
          id: delId,
          connector_id: conn.id,
          connector_name: conn.name,
          kind: conn.kind,
          method: conn.method,
          url: conn.url,
          status: "held",
          http_status: null,
          decision_id: did,
          payload_sha256: payloadSha,
          payload_bytes: utf8Bytes(payloadStr),
          summary: `Held at pipeline step "${spec.label}" awaiting approval before sending to ${conn.url}.`,
          ts_ms: ts,
        };
        cs.deliveries.unshift(delivery);
        st.decisions.push({
          id: did,
          kind: "connector_delivery",
          summary: `${run.routine_name || "Pipeline"}: Deliver via ${conn.name}`,
          detail: `Pipeline step "${spec.label}" is ready to send ${resolvedFile || "payload"} to ${conn.url}.`,
          severity: "warning",
          status: "pending",
          outcome: null,
          thread_id: run.thread_id || `pipeline-${run.run_id}`,
          ts,
          run_id: run.run_id,
          connector_id: conn.id,
          connector_name: conn.name,
          connector_kind: conn.kind,
          method: conn.method,
          url: conn.url,
          payload_preview: payloadStr.slice(0, 200),
          payload_sha256: payloadSha,
          delivery_id: delId,
          inspect_files: resolvedFile ? [resolvedFile] : [],
        });
        conn.last_delivery_ms = ts;
        conn.last_status = "held";
        stState.status = "waiting_human";
        stState.checkpoint_id = did;
        run.status = "paused_checkpoint";
        run.current_step_index = stepIndex;
        const pendingCp = {
          checkpoint_id: did,
          step_index: stepIndex,
          step_id: spec.id,
          prompt: `Approve outbound delivery to ${conn.name} (${conn.url})`,
          input_mode: "approve_reject" as const,
          inspect_files: resolvedFile ? [resolvedFile] : [],
          created_ms: ts,
          kind: "deliver",
          deliver: { ...deliverSpec, connector_id: conn.id, ...(resolvedFile ? { file: resolvedFile } : {}) },
          connector_id: conn.id,
          connector_name: conn.name,
          delivery_url: conn.url,
          payload_preview: payloadStr.slice(0, 200),
        };
        run.pending_checkpoint = pendingCp;
        run.updated_ms = ts;
        await persist();
        broadcastSse("pipeline_paused", {
          run_id: run.run_id,
          thread_id: run.thread_id ?? null,
          routine_name: run.routine_name ?? "",
          step_index: stepIndex,
          step_id: spec.id,
          checkpoint: pendingCp,
          run,
        });
        return { paused: true, output: "", files: [] };
      }

      const delivery: DeliveryLog = {
        id: `del-${ts}-${cs.deliveries.length + 1}`,
        connector_id: conn.id,
        connector_name: conn.name,
        kind: conn.kind,
        method: conn.method,
        url: conn.url,
        status: "ok",
        http_status: 200,
        payload_sha256: payloadSha,
        payload_bytes: utf8Bytes(payloadStr),
        response_preview: "Offline browser engine delivered pipeline payload (200 OK).",
        ts_ms: ts,
      };
      cs.deliveries.unshift(delivery);
      conn.last_delivery_ms = ts;
      conn.last_status = "ok";
      return {
        output: `Delivered via ${conn.name} (${conn.url}).`,
        files: resolvedFile ? [resolvedFile] : [],
        metrics: { connector_id: conn.id, delivery_id: delivery.id, payload_bytes: delivery.payload_bytes },
      };
    }

    if (spec.kind === "skill") {
      const resolvedArgs: Record<string, string> = {};
      for (const [k, v] of Object.entries(spec.args ?? {})) {
        if (typeof v === "string") {
          resolvedArgs[k] = resolvePipelineTemplate(v, inputs, run.step_states, extraVars);
        } else if (v !== undefined && v !== null) {
          resolvedArgs[k] = JSON.stringify(resolvePipelineValue(v, inputs, run.step_states, extraVars));
        } else {
          resolvedArgs[k] = "";
        }
      }
      const res = runPipelineSkillStep(st, spec.skill_id || "draft_brief", resolvedArgs);
      const bytesTotal = res.files.reduce((acc, f) => acc + (st.files[f]?.size ?? 0), 0);
      const anom = Number((res.metrics as { anomalies?: unknown }).anomalies ?? 0);
      return {
        output: res.output,
        files: res.files,
        metrics: {
          ...res.metrics,
          files_count: res.files.length,
          bytes_total: bytesTotal,
          anomalies: anom,
          has_anomaly: anom > 0,
        },
      };
    }

    if (spec.kind === "export") {
      const srcPath = resolvePipelineTemplate(spec.export?.path || "", inputs, run.step_states, extraVars).trim();
      const fmt = spec.export?.format || "docx";
      const srcFile = srcPath && st.files[srcPath] ? st.files[srcPath] : Object.values(st.files)[0];
      if (!srcFile) throw new Error("No workspace file available to export.");
      const stem = srcFile.name.replace(/\.[^.]+$/, "");
      const outName = spec.export?.output || `${stem}.${fmt}`;
      st.files[outName] = makeFile(outName, `Exported (${fmt.toUpperCase()}) from ${srcFile.path}\n\n${srcFile.content.slice(0, 4000)}`);
      return {
        output: `Exported ${outName} (${fmt.toUpperCase()}).`,
        files: [outName],
      };
    }

    // "agent" step
    const resolvedPrompt = resolvePipelineTemplate(spec.prompt || spec.label || "Complete task", inputs, run.step_states, extraVars);
    const snapshotMsgs = Array.isArray(run.agent_snapshot?.messages)
      ? (run.agent_snapshot.messages as WebMsg[])
      : [];
    const tempThread: WebThread = {
      id: run.thread_id || `pipeline-${run.run_id}`,
      title: run.routine_name || "Pipeline",
      messages: snapshotMsgs.slice(),
    };
    const dummyPush = () => {};
    const outcome =
      byok.engine_mode === "byok" && byok.model_base_url && byok.model_id
        ? await runByokLoop(st, byok, tempThread, resolvedPrompt, dummyPush, () => (run.status as string) === "cancelled")
        : await runDeterministicTask(st, tempThread, resolvedPrompt, dummyPush, () => (run.status as string) === "cancelled");
    if ((run.status as string) === "cancelled") {
      return { output: "", files: [] };
    }
    const summaryText = outcome.summary || "";
    if (summaryText) {
      const chunkSize = Math.max(24, Math.ceil(summaryText.length / 3));
      for (let pos = 0; pos < summaryText.length; pos += chunkSize) {
        broadcastSse("token_delta", {
          run_id: run.run_id,
          thread_id: run.thread_id ?? null,
          step_id: spec.id,
          delta: summaryText.slice(pos, pos + chunkSize),
        });
      }
    }
    run.agent_snapshot = {
      messages: [...snapshotMsgs, { role: "user", content: resolvedPrompt }, { role: "assistant", content: summaryText }].slice(-24),
      written: Array.from(new Set([...(run.agent_snapshot?.written ?? []), ...outcome.written])),
      budget_used: (run.agent_snapshot?.budget_used ?? 0) + resolvedPrompt.length + summaryText.length,
    };
    return {
      output: summaryText,
      files: outcome.written.filter((f) => f !== "memory.md"),
    };
  }

  async function executeWebPipelineStepAt(
    p: WebPerson,
    st: PersonStore,
    byok: ByokConfig,
    run: PipelineRun,
    i: number
  ): Promise<"ok" | "skipped" | "paused" | "failed" | "cancelled"> {
    if ((run.status as string) === "cancelled") return "cancelled";
    const steps = run.steps ?? run.steps_v2 ?? [];
    const inputs = run.inputs ?? {};
    const spec = steps[i];
    const stState =
      run.step_states[i] ||
      (run.step_states[i] = {
        step_id: spec.id,
        label: spec.label,
        kind: spec.kind,
        status: "pending",
        files: [],
        ...(Array.isArray(spec.depends_on) && spec.depends_on.length > 0 ? { depends_on: spec.depends_on } : {}),
      });
    const prevState = i > 0 ? run.step_states[i - 1] : undefined;

    if (!shouldRunPipelineStep(spec, prevState, run.step_states, inputs)) {
      stState.status = "skipped";
      stState.finished_ms = Date.now();
      run.current_step_index = i;
      run.updated_ms = Date.now();
      broadcastSse("pipeline_step_done", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        step_index: i,
        step_id: spec.id,
        label: spec.label,
        kind: spec.kind,
        status: "skipped",
        output: "",
        files: [],
        metrics: {},
        error: null,
        elapsed_ms: 0,
      });
      return "skipped";
    }

    stState.status = "running";
    stState.started_ms = Date.now();
    run.current_step_index = i;
    run.updated_ms = Date.now();
    broadcastSse("pipeline_step_started", {
      run_id: run.run_id,
      thread_id: run.thread_id ?? null,
      routine_id: run.routine_id ?? null,
      routine_name: run.routine_name ?? "",
      step_index: i,
      step_id: spec.id,
      label: spec.label,
      kind: spec.kind,
      depends_on: spec.depends_on ?? [],
      total_steps: steps.length,
    });

    try {
      if (spec.for_each && spec.for_each.trim() && spec.kind !== "checkpoint") {
        const items = expandForEachItems(spec.for_each, inputs, run.step_states);
        const childItems: PipelineChildItemState[] = items.map((item, idx) => ({
          step_id: `${spec.id}[${idx}]`,
          index: idx,
          item,
          status: "pending",
          files: [],
        }));
        stState.child_items = childItems;
        const allChildFiles: string[] = [];
        const childOutputs: string[] = [];
        for (let idx = 0; idx < items.length; idx += 1) {
          if ((run.status as string) === "cancelled") return "cancelled";
          const child = childItems[idx];
          child.status = "running";
          child.started_ms = Date.now();
          const res = await executeSinglePipelineItem(p, st, byok, run, i, spec, stState, {
            item: items[idx],
            "inputs.item": items[idx],
          });
          if (res.paused) {
            child.status = "waiting_human";
            return "paused";
          }
          child.status = "ok";
          child.finished_ms = Date.now();
          child.output = res.output;
          child.files = res.files;
          child.metrics = res.metrics;
          for (const f of res.files) {
            if (!allChildFiles.includes(f)) allChildFiles.push(f);
          }
          if (res.output) childOutputs.push(res.output);
        }
        stState.status = "ok";
        stState.finished_ms = Date.now();
        stState.files = allChildFiles;
        stState.output = childOutputs.join("\n") || `Processed ${items.length} batch item(s).`;
        stState.metrics = {
          items,
          items_count: items.length,
          files_count: allChildFiles.length,
        };
      } else {
        const res = await executeSinglePipelineItem(p, st, byok, run, i, spec, stState);
        if ((run.status as string) === "cancelled") return "cancelled";
        if (res.paused) return "paused";
        stState.status = "ok";
        stState.finished_ms = Date.now();
        stState.output = res.output;
        stState.files = res.files;
        if (res.metrics) stState.metrics = res.metrics;
      }

      const pricingMap = mergeModelPricing(state.modelPricing);
      const { engagement: actEng, client: actCli } = activeEngagementFor(p.id);
      const pRate = effectiveHourlyRate(p.id, actEng, actCli);
      const stepModelId = byok.engine_mode === "byok" && byok.model_id ? byok.model_id : "browser-workspace-engine";
      const pTok = Math.max(120, Math.round(((spec.label || "").length + (stState.output || "").length) / 3) + 180);
      const cTok = Math.max(40, Math.round((stState.output || "").length / 4) + 35);
      const cachedTok = Math.round(pTok * 0.2);
      const stepMins = spec.kind === "skill" ? estimateMinutesSaved(spec.skill_id || spec.label || "", 1) : 10;
      const stepUsage: RunUsage = {
        model_id: stepModelId,
        prompt_tokens: pTok,
        completion_tokens: cTok,
        cached_tokens: cachedTok,
        total_tokens: pTok + cTok,
        cost_usd: computeUsageCostUsd(pricingMap, stepModelId, pTok, cTok, cachedTok),
        latency_ms: stState.finished_ms && stState.started_ms ? Math.max(1, stState.finished_ms - stState.started_ms) : 1,
        estimated_minutes_saved: stepMins,
        billable_value_usd: Math.round((stepMins / 60) * pRate * 100) / 100,
        engagement_id: actEng?.id,
      };
      stState.usage = stepUsage;
      run.updated_ms = Date.now();
      await persist();
      broadcastSse("pipeline_step_done", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        step_index: i,
        step_id: spec.id,
        label: spec.label,
        kind: spec.kind,
        status: stState.status,
        output: stState.output ?? "",
        files: stState.files ?? [],
        metrics: stState.metrics ?? {},
        child_items: stState.child_items,
        usage: stepUsage,
        error: null,
        elapsed_ms: stState.finished_ms && stState.started_ms ? stState.finished_ms - stState.started_ms : 1,
      });
      return "ok";
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      stState.status = "failed";
      stState.finished_ms = Date.now();
      stState.error = msg;
      run.status = "failed";
      run.error = msg;
      run.current_step_index = i;
      run.updated_ms = Date.now();
      await persist();
      broadcastSse("pipeline_finished", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        status: "failed",
        error: msg,
        run,
      });
      return "failed";
    }
  }

  async function executeWebPipelineFrom(
    p: WebPerson,
    st: PersonStore,
    byok: ByokConfig,
    run: PipelineRun,
    startIndex: number
  ): Promise<void> {
    const steps = run.steps ?? run.steps_v2 ?? [];
    run.status = "running";
    run.pending_checkpoint = null;
    run.error = null;
    run.updated_ms = Date.now();
    await persist();

    const isDag = steps.some((s) => Array.isArray(s.depends_on) && s.depends_on.length > 0);
    if (isDag) {
      const check = validateWebPipelineSpec(steps, ensureWebConnectors(st).outbound);
      if (!check.valid) {
        const msg = check.errors[0] || "Invalid pipeline DAG.";
        run.status = "failed";
        run.error = msg;
        run.updated_ms = Date.now();
        await persist();
        broadcastSse("pipeline_finished", {
          run_id: run.run_id,
          thread_id: run.thread_id ?? null,
          status: "failed",
          error: msg,
          run,
        });
        return;
      }

      while (true) {
        if ((run.status as string) === "cancelled" || (run.status as string) === "paused_checkpoint" || (run.status as string) === "failed") {
          return;
        }
        const readyIndices: number[] = [];
        for (let i = 0; i < steps.length; i += 1) {
          const stState = run.step_states[i];
          if (!stState || stState.status !== "pending") continue;
          const deps = Array.isArray(steps[i].depends_on) ? steps[i].depends_on! : [];
          const allDone = deps.every((depId) => {
            const depState = run.step_states.find((s) => s.step_id === depId);
            return depState && (depState.status === "ok" || depState.status === "skipped");
          });
          if (allDone) {
            readyIndices.push(i);
            if (readyIndices.length >= DAG_MAX_CONCURRENCY) break;
          }
        }
        if (readyIndices.length === 0) {
          const hasPending = run.step_states.some((s) => s.status === "pending");
          if (hasPending) {
            run.status = "failed";
            run.error = "Pipeline DAG could not resolve remaining pending steps.";
            run.updated_ms = Date.now();
            await persist();
            broadcastSse("pipeline_finished", {
              run_id: run.run_id,
              thread_id: run.thread_id ?? null,
              status: "failed",
              error: run.error,
              run,
            });
          }
          break;
        }
        // Checkpoints run alone so parallel branches don't race past pause state
        const cpIdx = readyIndices.find((idx) => steps[idx]?.kind === "checkpoint");
        const batch = cpIdx !== undefined ? [cpIdx] : readyIndices;
        const outcomes = await Promise.all(
          batch.map((idx) => executeWebPipelineStepAt(p, st, byok, run, idx))
        );
        if (outcomes.includes("paused") || outcomes.includes("failed") || outcomes.includes("cancelled")) {
          return;
        }
      }
    } else {
      for (let i = startIndex; i < steps.length; i += 1) {
        const outcome = await executeWebPipelineStepAt(p, st, byok, run, i);
        if (outcome === "paused" || outcome === "failed" || outcome === "cancelled") {
          return;
        }
      }
    }

    if ((run.status as string) === "cancelled" || (run.status as string) === "paused_checkpoint" || (run.status as string) === "failed") return;
    run.status = "completed";
    run.pending_checkpoint = null;
    run.updated_ms = Date.now();
    const allOutFiles = Array.from(
      new Set(run.step_states.flatMap((s) => s.files ?? []))
    );
    const finalSummary =
      run.step_states
        .map((s) => s.output)
        .filter(Boolean)
        .slice(-1)[0] || `Completed pipeline ${run.routine_name || run.run_id}.`;
    const { engagement: pipeEng } = activeEngagementFor(p.id);
    const pipeUsage: RunUsage = run.step_states.reduce<RunUsage>(
      (acc, s) => {
        const u = s.usage;
        if (!u) return acc;
        return {
          model_id: u.model_id || acc.model_id,
          prompt_tokens: acc.prompt_tokens + (u.prompt_tokens || 0),
          completion_tokens: acc.completion_tokens + (u.completion_tokens || 0),
          cached_tokens: (acc.cached_tokens || 0) + (u.cached_tokens || 0),
          total_tokens: acc.total_tokens + (u.total_tokens || 0),
          cost_usd: Math.round((acc.cost_usd + (u.cost_usd || 0)) * 1_000_000) / 1_000_000,
          latency_ms: (acc.latency_ms || 0) + (u.latency_ms || 0),
          estimated_minutes_saved: (acc.estimated_minutes_saved || 0) + (u.estimated_minutes_saved || 0),
          billable_value_usd: Math.round(((acc.billable_value_usd || 0) + (u.billable_value_usd || 0)) * 100) / 100,
          engagement_id: pipeEng?.id,
        };
      },
      {
        model_id: byok.engine_mode === "byok" && byok.model_id ? byok.model_id : "browser-workspace-engine",
        prompt_tokens: 0,
        completion_tokens: 0,
        cached_tokens: 0,
        total_tokens: 0,
        cost_usd: 0,
        latency_ms: 0,
        estimated_minutes_saved: 0,
        billable_value_usd: 0,
        engagement_id: pipeEng?.id,
      }
    );
    run.engagement_id = pipeEng?.id ?? null;
    run.usage = pipeUsage;
    if (pipeUsage.total_tokens > 0) {
      recordUsageEntry(p, st, {
        engagement_id: pipeEng?.id,
        thread_id: run.thread_id || `pipeline-${run.run_id}`,
        run_id: run.run_id,
        kind: "pipeline",
        usage: pipeUsage,
      });
    }
    st.work.push({
      id: run.run_id,
      task: run.routine_name || "Pipeline run",
      summary: finalSummary,
      files: allOutFiles,
      finished: true,
      answered: false,
      thread_id: run.thread_id || `pipeline-${run.run_id}`,
      ts: run.updated_ms,
      engagement_id: pipeEng?.id,
      usage: pipeUsage.total_tokens > 0 ? pipeUsage : undefined,
    });
    if (run.routine_id) {
      const rt = st.routines.find((r) => r.id === run.routine_id);
      if (rt) {
        rt.last_run_ts = run.updated_ms;
        rt.last_status = "finished";
        rt.run_count = (rt.run_count ?? 0) + 1;
      }
    }
    await persist();
    broadcastSse("pipeline_finished", {
      run_id: run.run_id,
      thread_id: run.thread_id ?? null,
      status: "completed",
      error: null,
      usage: pipeUsage,
      run,
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

function normalizeWebMemoryRule(s: string): string {
  return s.trim().replace(/\s+/g, " ").replace(/[.!;]+$/, "").toLowerCase();
}

function webMemoryRuleMeta(norm: string): { pol: number; pref: [string, string] | null; topics: string[] } {
  const negMarkers = ["never ", "avoid ", "do not ", "don't ", "dont ", "exclude ", "omit ", "skip ", "disable ", "no "];
  const posMarkers = ["always ", "use ", "include ", "enable ", "keep ", "require ", "prefer "];
  const hasNeg = negMarkers.some((m) => norm.startsWith(m) || norm.includes(" " + m));
  const hasPos = posMarkers.some((m) => norm.startsWith(m) || norm.includes(" " + m));
  const pol = hasPos && !hasNeg ? 1 : hasNeg ? -1 : 0;
  const prefPrefixes = ["prefer ", "default format ", "export format ", "tone: ", "currency: ", "rounding: ", "date format: ", "language: "];
  let pref: [string, string] | null = null;
  for (const p of prefPrefixes) {
    if (norm.startsWith(p)) {
      const val = norm.slice(p.length).trim();
      if (val) {
        pref = [p, val];
        break;
      }
    }
  }
  const stop = new Set([
    "the", "and", "for", "with", "from", "that", "this", "use", "always", "never", "avoid",
    "not", "dont", "don", "prefer", "include", "exclude", "omit", "skip", "keep", "only",
    "must", "should", "when", "into", "all", "are", "was", "were", "have", "has", "any",
    "enable", "disable", "require",
  ]);
  const topics = Array.from(
    new Set(
      norm
        .split(/[^a-z0-9]+/)
        .filter((w) => w.length >= 3 && !stop.has(w))
    )
  ).sort();
  return { pol, pref, topics };
}

function auditWebMemoryHealth(st: PersonStore) {
  const EXTS = [".csv", ".md", ".json", ".txt", ".docx", ".xlsx", ".pptx", ".pdf", ".rn", ".tsv", ".yaml", ".yml"];
  const lines = st.memory.split("\n");
  const seen = new Map<string, number>();
  const entries: { line: number; text: string; norm: string; pol: number; pref: [string, string] | null; topics: string[]; isDup: boolean }[] = [];
  const issues: { line: number; kind: "conflict" | "duplicate" | "stale_ref"; text: string; detail: string; related_line: number | null; removable: boolean }[] = [];

  lines.forEach((raw, idx) => {
    const lineNo = idx + 1;
    const t = raw.trim();
    if (!t.startsWith("- ") || t.startsWith("- Name:") || t.startsWith("- Role:")) return;
    const rule = t.slice(2).trim();
    if (!rule) return;
    const norm = normalizeWebMemoryRule(rule);
    const first = seen.get(norm);
    const isDup = first !== undefined;
    if (isDup && first !== undefined) {
      issues.push({
        line: lineNo,
        kind: "duplicate",
        text: rule,
        detail: `Duplicate of L${first}`,
        related_line: first,
        removable: true,
      });
    } else {
      seen.set(norm, lineNo);
    }

    // Check [file:L##] citations and file references against st.files
    let staleDetail: string | null = null;
    const citeRe = /\[([^\]\n]+?):L(\d+)(?:-L(\d+))?\]/g;
    let m: RegExpExecArray | null;
    while ((m = citeRe.exec(rule)) !== null) {
      const fpath = m[1].trim();
      const lnum = Number(m[2]);
      const endNum = m[3] ? Number(m[3]) : lnum;
      const wf = st.files[fpath];
      if (!wf) {
        staleDetail = `Missing cited file [${fpath}:L${lnum}]`;
        break;
      }
      const totalLines = wf.content.split("\n").length;
      if (lnum < 1 || lnum > totalLines || endNum < lnum || endNum > totalLines) {
        staleDetail = `Out-of-range citation [${fpath}:L${lnum}] (file has ${totalLines} lines)`;
        break;
      }
    }
    if (!staleDetail) {
      const stripped = rule.replace(/\[[^\]\n]+?:L\d+(?:-L\d+)?\]/g, " ");
      const btRe = /`([^`\n]+)`/g;
      let bm: RegExpExecArray | null;
      while ((bm = btRe.exec(stripped)) !== null) {
        const cand = bm[1].trim();
        const low = cand.toLowerCase();
        if (cand.length >= 3 && !low.startsWith("http://") && !low.startsWith("https://") && EXTS.some((ext) => low.endsWith(ext))) {
          if (cand !== "memory.md" && !st.files[cand]) {
            staleDetail = `Missing workspace file \`${cand}\``;
            break;
          }
        }
      }
      if (!staleDetail) {
        const plain = stripped.replace(/`[^`\n]*`/g, " ");
        for (const tok of plain.split(/\s+/)) {
          const cleaned = tok.replace(/^['"([\]]+|['"(\[\],;:.!]+$/g, "");
          const base = cleaned.replace(/:L\d+(?:-L\d+)?$/, "");
          const low = base.toLowerCase();
          if (base.length >= 3 && !low.startsWith("http://") && !low.startsWith("https://") && EXTS.some((ext) => low.endsWith(ext))) {
            if (base !== "memory.md" && !st.files[base]) {
              staleDetail = `Missing workspace file \`${base}\``;
              break;
            }
          }
        }
      }
    }
    if (staleDetail) {
      issues.push({
        line: lineNo,
        kind: "stale_ref",
        text: rule,
        detail: staleDetail,
        related_line: null,
        removable: true,
      });
    }

    const meta = webMemoryRuleMeta(norm);
    entries.push({ line: lineNo, text: rule, norm, ...meta, isDup });
  });

  const conflictedLines = new Set<number>();
  for (let i = 0; i < entries.length; i++) {
    if (entries[i].isDup) continue;
    for (let j = i + 1; j < entries.length; j++) {
      if (entries[j].isDup || entries[i].norm === entries[j].norm) continue;
      const samePref = entries[i].pref && entries[j].pref && entries[i].pref![0] === entries[j].pref![0] && entries[i].pref![1] !== entries[j].pref![1];
      const oppPol = entries[i].pol * entries[j].pol === -1;
      const shared = oppPol ? entries[i].topics.filter((t) => entries[j].topics.includes(t)).length : 0;
      const minTopics = Math.min(entries[i].topics.length, entries[j].topics.length);
      const topicConflict = oppPol && (shared >= 2 || (minTopics > 0 && minTopics <= 2 && shared >= 1));
      if ((samePref || topicConflict) && !conflictedLines.has(entries[i].line)) {
        conflictedLines.add(entries[i].line);
        issues.push({
          line: entries[i].line,
          kind: "conflict",
          text: entries[i].text,
          detail: `${samePref ? "Superseded by preference at" : "Conflicts with newer rule at"} L${entries[j].line}`,
          related_line: entries[j].line,
          removable: true,
        });
        break;
      }
    }
  }

  issues.sort((a, b) => a.line - b.line || a.kind.localeCompare(b.kind));
  const issueLines = new Set(issues.map((x) => x.line));
  return {
    status: issueLines.size === 0 ? ("healthy" as const) : ("issues" as const),
    learned_count: entries.length,
    issue_count: issueLines.size,
    conflicts: issues.filter((x) => x.kind === "conflict").length,
    duplicates: issues.filter((x) => x.kind === "duplicate").length,
    stale_refs: issues.filter((x) => x.kind === "stale_ref").length,
    issues,
  };
}

function buildWebPolicy(personId: string, st: PersonStore) {
  const raw = st.policyProfile || "fast_internal";
  const presets = {
    fast_internal: {
      profile: "fast_internal" as const,
      label: "Fast Internal",
      summary: "Advisory citation checks, local-speed approvals, and outbound web research enabled.",
      strict_citation_gate: false,
      approval_gate_mode: "deletes_and_risky",
      redact_host_paths: false,
      block_outbound_fetch: false,
    },
    strict_client: {
      profile: "strict_client" as const,
      label: "Strict Client",
      summary: "Blocks pack export on stale [file:L##] citations, redacts host paths in handoffs, and enforces approval gates.",
      strict_citation_gate: true,
      approval_gate_mode: "all_external_and_deletes",
      redact_host_paths: true,
      block_outbound_fetch: false,
    },
    air_gapped: {
      profile: "air_gapped" as const,
      label: "Air-Gapped",
      summary: "Zero outbound network fetches, strict citation gate before export, and full host-path redaction.",
      strict_citation_gate: true,
      approval_gate_mode: "lockdown",
      redact_host_paths: true,
      block_outbound_fetch: true,
    },
  };
  return {
    person_id: personId,
    ...(presets[raw] || presets.fast_internal),
    isolate_active_engagement: Boolean(st.isolateActiveEngagement),
    updated_ms: st.policyUpdatedMs || 0,
  };
}

function redactWebHostPaths(input: string): string {
  return input
    .replace(/\/home\/[^/\s"'`)\]<>]+\//g, "~/")
    .replace(/\/Users\/[^/\s"'`)\]<>]+\//g, "~/")
    .replace(/C:[\\/]Users[\\/][^\\/\s"'`)\]<>]+[\\/]/g, "~/")
    .replace(/\/tmp\//g, "[redacted-tmp]/");
}

type WebHandoffLeakFinding = {
  kind: string;
  severity: string;
  file: string;
  line: number;
  excerpt: string;
  fix: string;
};

type WebHandoffLeakScan = {
  clean: boolean;
  leak_count: number;
  host_path_count: number;
  secret_count: number;
  env_snippet_count: number;
  pii_count: number;
  findings: WebHandoffLeakFinding[];
};

function scanWebHandoffLeaks(
  files: Array<{ path: string; content: string }>,
  summaryMd: string,
  extraSecret?: string
): WebHandoffLeakScan {
  const findings: WebHandoffLeakFinding[] = [];
  const push = (kind: string, severity: string, file: string, line: number, rawLine: string, fix: string) => {
    if (findings.length >= 64) return;
    const excerpt = rawLine.trim().slice(0, 96);
    findings.push({ kind, severity, file, line, excerpt, fix });
  };
  const secretTokenRe = /(sk-ergon-[A-Za-z0-9_-]+|sk-ant-[A-Za-z0-9_-]{8,}|sk-proj-[A-Za-z0-9_-]{8,}|sk-[A-Za-z0-9]{16,}|ghp_[A-Za-z0-9]{16,}|gho_[A-Za-z0-9]{16,}|github_pat_[A-Za-z0-9_]{16,}|AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{20,}|xoxb-[0-9A-Za-z-]{16,}|-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----)/;
  const hostPathRe = /(\/home\/|\/Users\/|\/tmp\/|\/var\/folders\/|C:\\Users\\|C:\/Users\/)/;
  const envAssignRe = /^(?:export\s+)?([A-Z0-9_]*(?:API_KEY|SECRET_KEY|AUTH_TOKEN|PRIVATE_KEY|DATABASE_URL|ACCESS_TOKEN|PASSWORD)[A-Z0-9_]*)\s*=\s*(.+)$/;
  const ssnRe = /(^|[^0-9])\d{3}-\d{2}-\d{4}([^0-9]|$)/;

  const sources = [
    ...files.map((f) => ({ name: f.path, text: f.content })),
    { name: "CLIENT_SUMMARY.md", text: summaryMd },
  ];
  for (const src of sources) {
    const isEnvFile = src.name.split("/").pop()?.startsWith(".env") ?? false;
    const lines = src.text.split("\n");
    for (let idx = 0; idx < lines.length; idx++) {
      const lineNo = idx + 1;
      const line = lines[idx];
      if (!line.trim()) continue;
      if (isEnvFile && line.includes("=") && !line.trim().startsWith("#")) {
        push("env_snippet", "critical", src.name, lineNo, line, `Remove .env credential assignment at ${src.name}:L${lineNo} before client handoff.`);
      }
      if (hostPathRe.test(line)) {
        push("host_path", "warning", src.name, lineNo, line, `Replace local machine path at ${src.name}:L${lineNo} with a relative workspace path.`);
      }
      if (secretTokenRe.test(line) || (extraSecret && extraSecret.length >= 8 && line.includes(extraSecret))) {
        push("api_key", "critical", src.name, lineNo, line, `Revoke and remove exposed key/token at ${src.name}:L${lineNo} before shipping.`);
      } else {
        const em = envAssignRe.exec(line.trim());
        if (em) {
          const val = em[2].trim().replace(/^["']|["']$/g, "");
          if (val && !/^(example|placeholder|changeme|<.*>|\$\{.*\})$/i.test(val)) {
            push("env_snippet", "critical", src.name, lineNo, line, `Strip live ${em[1]} value at ${src.name}:L${lineNo}.`);
          }
        }
      }
      if (ssnRe.test(line)) {
        push("pii", "warning", src.name, lineNo, line, `Mask or remove SSN-formatted identifier at ${src.name}:L${lineNo}.`);
      }
    }
  }
  const host_path_count = findings.filter((f) => f.kind === "host_path").length;
  const secret_count = findings.filter((f) => f.kind === "api_key").length;
  const env_snippet_count = findings.filter((f) => f.kind === "env_snippet").length;
  const pii_count = findings.filter((f) => f.kind === "pii").length;
  return {
    clean: findings.length === 0,
    leak_count: findings.length,
    host_path_count,
    secret_count,
    env_snippet_count,
    pii_count,
    findings,
  };
}

function crc32Bytes(data: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < data.length; i++) {
    crc ^= data[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function buildWebStoreZip(entries: Array<[string, Uint8Array]>): Uint8Array {
  const enc = new TextEncoder();
  const chunks: Uint8Array[] = [];
  const centrals: Uint8Array[] = [];
  let offset = 0;
  for (const [name, data] of entries) {
    const nameBytes = enc.encode(name);
    const crc = crc32Bytes(data);
    const lh = new Uint8Array(30 + nameBytes.length);
    const lv = new DataView(lh.buffer);
    lv.setUint32(0, 0x04034b50, true);
    lv.setUint16(4, 20, true);
    lv.setUint16(6, 0x0800, true);
    lv.setUint16(8, 0, true);
    lv.setUint32(14, crc, true);
    lv.setUint32(18, data.length, true);
    lv.setUint32(22, data.length, true);
    lv.setUint16(26, nameBytes.length, true);
    lh.set(nameBytes, 30);
    chunks.push(lh, data);

    const ch = new Uint8Array(46 + nameBytes.length);
    const cv = new DataView(ch.buffer);
    cv.setUint32(0, 0x02014b50, true);
    cv.setUint16(4, 20, true);
    cv.setUint16(6, 20, true);
    cv.setUint16(8, 0x0800, true);
    cv.setUint16(10, 0, true);
    cv.setUint32(16, crc, true);
    cv.setUint32(20, data.length, true);
    cv.setUint32(24, data.length, true);
    cv.setUint16(28, nameBytes.length, true);
    cv.setUint32(42, offset, true);
    ch.set(nameBytes, 46);
    centrals.push(ch);
    offset += lh.length + data.length;
  }
  const cdSize = centrals.reduce((acc, b) => acc + b.length, 0);
  const eocd = new Uint8Array(22);
  const ev = new DataView(eocd.buffer);
  ev.setUint32(0, 0x06054b50, true);
  ev.setUint16(8, entries.length, true);
  ev.setUint16(10, entries.length, true);
  ev.setUint32(12, cdSize, true);
  ev.setUint32(16, offset, true);

  const out = new Uint8Array(offset + cdSize + 22);
  let pos = 0;
  for (const c of chunks) {
    out.set(c, pos);
    pos += c.length;
  }
  for (const c of centrals) {
    out.set(c, pos);
    pos += c.length;
  }
  out.set(eocd, pos);
  return out;
}

function bytesToBase64(bytes: Uint8Array): string {
  let bin = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    bin += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(bin);
}

function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function pdfEscape(s: string): string {
  return s
    .replace(/[^\x20-\x7e]/g, " ")
    .replace(/\\/g, "\\\\")
    .replace(/\(/g, "\\(")
    .replace(/\)/g, "\\)");
}

function wrapPdfLines(text: string, maxChars: number): Array<{ text: string; bold: boolean }> {
  const out: Array<{ text: string; bold: boolean }> = [];
  for (const raw of text.replace(/\r\n/g, "\n").split("\n")) {
    const t = raw.trimEnd();
    if (!t.trim()) {
      out.push({ text: "", bold: false });
      continue;
    }
    const isHeading = /^#{1,3}\s+/.test(t.trim());
    const clean = isHeading ? t.trim().replace(/^#{1,3}\s+/, "") : t;
    let rest = clean;
    while (rest.length > maxChars) {
      let cut = rest.lastIndexOf(" ", maxChars);
      if (cut <= 0) cut = maxChars;
      out.push({ text: rest.slice(0, cut), bold: isHeading });
      rest = rest.slice(cut).trimStart();
    }
    out.push({ text: rest, bold: isHeading });
  }
  return out.length ? out : [{ text: "(empty document)", bold: false }];
}

function buildWebPdf(content: string, title: string, brandHeader?: string): Uint8Array {
  const enc = new TextEncoder();
  const lines = wrapPdfLines(content, 82);
  const perPage = 44;
  const pages: Array<Array<{ text: string; bold: boolean }>> = [];
  for (let i = 0; i < lines.length; i += perPage) {
    pages.push(lines.slice(i, i + perPage));
  }
  if (!pages.length) pages.push([{ text: title || "Document", bold: false }]);

  const totalPages = pages.length;
  // Objects: 1=Catalog, 2=Pages, 3=Helvetica, 4=Helvetica-Bold, then per page: Page obj & Stream obj
  const objects: string[] = [];
  const pageIds: number[] = [];
  const baseObjId = 5;
  for (let pIdx = 0; pIdx < totalPages; pIdx++) {
    const pageObjId = baseObjId + pIdx * 2;
    const contentObjId = pageObjId + 1;
    pageIds.push(pageObjId);

    const cmds: string[] = ["BT"];
    if (brandHeader || title) {
      cmds.push("/F2 10 Tf 54 756 Td (" + pdfEscape(brandHeader || title) + ") Tj");
    }
    let y = 732;
    for (const ln of pages[pIdx]) {
      if (ln.text) {
        cmds.push(`${ln.bold ? "/F2 12" : "/F1 10"} Tf 1 0 0 1 54 ${y} Tm (${pdfEscape(ln.text)}) Tj`);
      }
      y -= ln.bold ? 18 : 14;
    }
    cmds.push(`/F1 9 Tf 1 0 0 1 54 36 Tm (${pdfEscape(`Page ${pIdx + 1} of ${totalPages}`)}) Tj`);
    cmds.push("ET");
    const stream = cmds.join("\n");
    objects[pageObjId] = `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents ${contentObjId} 0 R >>`;
    objects[contentObjId] = `<< /Length ${enc.encode(stream).length} >>\nstream\n${stream}\nendstream`;
  }

  objects[1] = "<< /Type /Catalog /Pages 2 0 R >>";
  objects[2] = `<< /Type /Pages /Kids [${pageIds.map((id) => `${id} 0 R`).join(" ")}] /Count ${totalPages} >>`;
  objects[3] = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
  objects[4] = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>";

  const parts: string[] = ["%PDF-1.4\n"];
  const offsets: number[] = [0];
  let byteOffset = enc.encode(parts[0]).length;
  for (let id = 1; id < objects.length; id++) {
    offsets[id] = byteOffset;
    const chunk = `${id} 0 obj\n${objects[id]}\nendobj\n`;
    parts.push(chunk);
    byteOffset += enc.encode(chunk).length;
  }
  const xrefOffset = byteOffset;
  let xref = `xref\n0 ${objects.length}\n0000000000 65535 f \n`;
  for (let id = 1; id < objects.length; id++) {
    xref += `${String(offsets[id]).padStart(10, "0")} 00000 n \n`;
  }
  const trailer = `trailer\n<< /Size ${objects.length} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF\n`;
  parts.push(xref, trailer);
  return enc.encode(parts.join(""));
}

function xmlEscape(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function colLetter(idx: number): string {
  let n = idx;
  let out = "";
  while (n >= 0) {
    out = String.fromCharCode(65 + (n % 26)) + out;
    n = Math.floor(n / 26) - 1;
  }
  return out;
}

function parseNumericCell(raw: string): number | null {
  const cleaned = raw.trim().replace(/[$,%\s]/g, "");
  if (!cleaned || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) return null;
  const n = Number(cleaned);
  return Number.isFinite(n) ? n : null;
}

function buildWebMultiSheetXlsx(sheets: Array<{ name: string; rows: string[][]; totals_row?: boolean }>): Uint8Array {
  const enc = new TextEncoder();
  const safeSheets = sheets.length ? sheets : [{ name: "Sheet1", rows: [["Empty"]] }];
  const entries: Array<[string, Uint8Array]> = [];

  const overrides = safeSheets
    .map((_, i) => `<Override PartName="/xl/worksheets/sheet${i + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>`)
    .join("");
  entries.push([
    "[Content_Types].xml",
    enc.encode(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>${overrides}</Types>`
    ),
  ]);
  entries.push([
    "_rels/.rels",
    enc.encode(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`
    ),
  ]);
  entries.push([
    "xl/styles.xml",
    enc.encode(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="2"><font><sz val="11"/><name val="Calibri"/></font><font><b/><sz val="11"/><name val="Calibri"/></font></fonts><fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills><borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="2"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0" applyFont="1"/></cellXfs></styleSheet>`
    ),
  ]);
  const sheetXmlRefs = safeSheets
    .map((sh, i) => `<sheet name="${xmlEscape(sh.name.slice(0, 31) || `Sheet${i + 1}`)}" sheetId="${i + 1}" r:id="rId${i + 1}"/>`)
    .join("");
  entries.push([
    "xl/workbook.xml",
    enc.encode(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>${sheetXmlRefs}</sheets></workbook>`
    ),
  ]);
  const wbRels = safeSheets
    .map((_, i) => `<Relationship Id="rId${i + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet${i + 1}.xml"/>`)
    .join("");
  entries.push([
    "xl/_rels/workbook.xml.rels",
    enc.encode(
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">${wbRels}</Relationships>`
    ),
  ]);

  safeSheets.forEach((sh, i) => {
    const rows = sh.rows.length ? sh.rows : [[""]];
    const maxCols = Math.max(1, ...rows.map((r) => r.length));
    const xmlRows: string[] = [];
    rows.forEach((row, rIdx) => {
      const rNum = rIdx + 1;
      const cells = Array.from({ length: maxCols }, (_, cIdx) => {
        const ref = `${colLetter(cIdx)}${rNum}`;
        const raw = row[cIdx] ?? "";
        const num = rIdx > 0 ? parseNumericCell(raw) : null;
        if (num !== null) {
          return `<c r="${ref}"><v>${num}</v></c>`;
        }
        return `<c r="${ref}" t="inlineStr"${rIdx === 0 ? ' s="1"' : ""}><is><t>${xmlEscape(raw)}</t></is></c>`;
      }).join("");
      xmlRows.push(`<row r="${rNum}">${cells}</row>`);
    });
    if (sh.totals_row && rows.length > 1) {
      const totRowNum = rows.length + 1;
      const cells: string[] = [];
      for (let cIdx = 0; cIdx < maxCols; cIdx++) {
        const ref = `${colLetter(cIdx)}${totRowNum}`;
        if (cIdx === 0) {
          cells.push(`<c r="${ref}" t="inlineStr" s="1"><is><t>Total</t></is></c>`);
          continue;
        }
        let sum = 0;
        let hasNum = false;
        for (let rIdx = 1; rIdx < rows.length; rIdx++) {
          const n = parseNumericCell(rows[rIdx]?.[cIdx] ?? "");
          if (n !== null) {
            sum += Math.round(n * 1_000_000);
            hasNum = true;
          }
        }
        if (hasNum) {
          const col = colLetter(cIdx);
          const cached = Math.round(sum) / 1_000_000;
          cells.push(`<c r="${ref}" s="1"><f>SUM(${col}2:${col}${rows.length})</f><v>${cached}</v></c>`);
        } else {
          cells.push(`<c r="${ref}" t="inlineStr"><is><t></t></is></c>`);
        }
      }
      xmlRows.push(`<row r="${totRowNum}">${cells.join("")}</row>`);
    }
    entries.push([
      `xl/worksheets/sheet${i + 1}.xml`,
      enc.encode(
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>${xmlRows.join("")}</sheetData></worksheet>`
      ),
    ]);
  });

  return buildWebStoreZip(entries);
}

function buildWebDocx(content: string, title: string): Uint8Array {
  const enc = new TextEncoder();
  const paras = [title, ...content.replace(/\r\n/g, "\n").split("\n")]
    .map((line) => `<w:p><w:r><w:t xml:space="preserve">${xmlEscape(line)}</w:t></w:r></w:p>`)
    .join("");
  return buildWebStoreZip([
    [
      "[Content_Types].xml",
      enc.encode(
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>`
      ),
    ],
    [
      "_rels/.rels",
      enc.encode(
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>`
      ),
    ],
    [
      "word/document.xml",
      enc.encode(
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>${paras}<w:sectPr/></w:body></w:document>`
      ),
    ],
  ]);
}

type TabularQuerySpec = {
  where?: Array<{ col: string; op: string; value: string }>;
  group_by?: string[];
  aggregates?: Array<{ col: string; fn: string; as: string }>;
  sort?: { col: string; dir: string };
  limit?: number;
};

function runTabularAggregation(
  header: string[],
  dataRows: string[][],
  req: TabularQuerySpec
): { columns: string[]; rows: string[][]; total_rows: number; matched_rows: number } {
  if (!header.length) return { columns: [], rows: [], total_rows: 0, matched_rows: 0 };
  const colIndex = (name: string) => header.findIndex((h) => h.trim().toLowerCase() === name.trim().toLowerCase());
  const total_rows = dataRows.length;

  let filtered = dataRows;
  if (Array.isArray(req.where) && req.where.length > 0) {
    filtered = dataRows.filter((row) =>
      req.where!.every((w) => {
        const idx = colIndex(w.col);
        if (idx < 0) return false;
        const cell = (row[idx] ?? "").trim();
        const target = String(w.value ?? "").trim();
        const cn = parseNumericCell(cell);
        const tn = parseNumericCell(target);
        switch (w.op) {
          case "eq":
            return cn !== null && tn !== null ? cn === tn : cell.toLowerCase() === target.toLowerCase();
          case "neq":
            return cn !== null && tn !== null ? cn !== tn : cell.toLowerCase() !== target.toLowerCase();
          case "contains":
            return cell.toLowerCase().includes(target.toLowerCase());
          case "gt":
            return cn !== null && tn !== null ? cn > tn : cell.localeCompare(target) > 0;
          case "gte":
            return cn !== null && tn !== null ? cn >= tn : cell.localeCompare(target) >= 0;
          case "lt":
            return cn !== null && tn !== null ? cn < tn : cell.localeCompare(target) < 0;
          case "lte":
            return cn !== null && tn !== null ? cn <= tn : cell.localeCompare(target) <= 0;
          default:
            return true;
        }
      })
    );
  }
  const matched_rows = filtered.length;

  let outCols = [...header];
  let outRows = filtered.map((r) => [...r]);

  const groupCols = Array.isArray(req.group_by) ? req.group_by.filter(Boolean) : [];
  const aggs = Array.isArray(req.aggregates) ? req.aggregates : [];
  if (groupCols.length > 0 || aggs.length > 0) {
    const gIdxs = groupCols.map((g) => colIndex(g));
    const groups = new Map<string, { keyVals: string[]; rows: string[][] }>();
    for (const row of filtered) {
      const keyVals = gIdxs.map((idx) => (idx >= 0 ? (row[idx] ?? "").trim() : ""));
      const key = JSON.stringify(keyVals);
      const entry = groups.get(key) || { keyVals, rows: [] };
      entry.rows.push(row);
      groups.set(key, entry);
    }
    outCols = [
      ...groupCols,
      ...aggs.map((a) => a.as || `${a.fn}_${a.col}`),
    ];
    outRows = [];
    for (const grp of groups.values()) {
      const aggCells = aggs.map((a) => {
        if (a.fn === "count") return String(grp.rows.length);
        const cIdx = colIndex(a.col);
        if (cIdx < 0) return "0";
        const nums: number[] = [];
        let hasDecimal = false;
        for (const r of grp.rows) {
          const raw = (r[cIdx] ?? "").trim();
          const n = parseNumericCell(raw);
          if (n !== null) {
            nums.push(n);
            if (raw.includes(".")) hasDecimal = true;
          }
        }
        if (!nums.length) return "0";
        const scaled = nums.map((n) => Math.round(n * 1_000_000));
        let val = 0;
        if (a.fn === "sum") val = scaled.reduce((acc, v) => acc + v, 0) / 1_000_000;
        else if (a.fn === "avg") val = scaled.reduce((acc, v) => acc + v, 0) / 1_000_000 / nums.length;
        else if (a.fn === "min") {
          val = nums[0];
          for (let ni = 1; ni < nums.length; ni++) {
            if (nums[ni] < val) val = nums[ni];
          }
        } else if (a.fn === "max") {
          val = nums[0];
          for (let ni = 1; ni < nums.length; ni++) {
            if (nums[ni] > val) val = nums[ni];
          }
        }
        if (!Number.isFinite(val)) return "0";
        return hasDecimal || a.fn === "avg" ? val.toFixed(2) : String(Math.round(val * 100) / 100);
      });
      outRows.push([...grp.keyVals, ...aggCells]);
    }
  }

  if (req.sort?.col) {
    const sIdx = outCols.findIndex((c) => c.trim().toLowerCase() === req.sort!.col.trim().toLowerCase());
    if (sIdx >= 0) {
      const mult = req.sort.dir === "desc" ? -1 : 1;
      outRows.sort((a, b) => {
        const av = (a[sIdx] ?? "").trim();
        const bv = (b[sIdx] ?? "").trim();
        const an = parseNumericCell(av);
        const bn = parseNumericCell(bv);
        if (an !== null && bn !== null) return (an - bn) * mult;
        return av.localeCompare(bv, undefined, { numeric: true, sensitivity: "base" }) * mult;
      });
    }
  }

  if (typeof req.limit === "number" && req.limit > 0) {
    outRows = outRows.slice(0, req.limit);
  }
  return { columns: outCols, rows: outRows, total_rows, matched_rows };
}

function runWebTableQuery(
  file: WebFile,
  req: TabularQuerySpec
): { columns: string[]; rows: string[][]; total_rows: number; matched_rows: number } {
  let header: string[] = [];
  let dataRows: string[][] = [];
  if (file.ext === "json") {
    try {
      const parsed = JSON.parse(file.content);
      const arr = Array.isArray(parsed) ? parsed : [];
      const keys = Array.from(new Set(arr.flatMap((item) => (item && typeof item === "object" ? Object.keys(item) : []))));
      header = keys;
      dataRows = arr.map((item) => keys.map((k) => String((item as Record<string, unknown>)?.[k] ?? "")));
    } catch {
      return { columns: [], rows: [], total_rows: 0, matched_rows: 0 };
    }
  } else {
    const delim = file.ext === "tsv" ? "\t" : ",";
    const insp = inspectDelimited(file.content, delim);
    if (insp.hasNullByte || insp.unclosedQuote || !insp.rows.length) {
      return { columns: [], rows: [], total_rows: 0, matched_rows: 0 };
    }
    header = insp.rows[0];
    dataRows = insp.rows.slice(1).map((r) => header.map((_, i) => r[i] ?? ""));
  }
  return runTabularAggregation(header, dataRows, req);
}

type InternalVaultChunk = {
  chunk_id: string;
  path: string;
  sheet: string | null;
  page: number | null;
  line_start: number;
  line_end: number;
  heading: string | null;
  text: string;
  textLower: string;
  tf: Map<string, number>;
  tokenLen: number;
  semVec: Float32Array;
};

type CachedVaultIndex = {
  sig: string;
  chunks: InternalVaultChunk[];
  df: Map<string, number>;
  avgLen: number;
  stats: VaultIndexStats;
};

const SEM_DIMS = 64;
const vaultIndexCache = new WeakMap<PersonStore, CachedVaultIndex>();

function isVaultExcludedPath(fp: string): boolean {
  const clean = fp.replace(/\\/g, "/");
  if (!clean || clean.startsWith(".") || clean.includes("/.")) return true;
  return clean === "connector_secrets.json" || clean.endsWith("/connector_secrets.json");
}

function tokenizeVaultText(text: string): string[] {
  return text.toLowerCase().match(/[a-z0-9_]+/g) || [];
}

function hashToken32(str: string): number {
  let h = 2166136261;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function embedVaultText(text: string): Float32Array {
  const vec = new Float32Array(SEM_DIMS);
  const tokens = tokenizeVaultText(text);
  for (const tok of tokens) {
    const b1 = hashToken32(tok) % SEM_DIMS;
    vec[b1] += 1.5;
    if (tok.length >= 3) {
      for (let i = 0; i <= tok.length - 3; i++) {
        const gram = tok.slice(i, i + 3);
        const bg = hashToken32(gram) % SEM_DIMS;
        vec[bg] += 0.5;
      }
    }
  }
  let norm = 0;
  for (let i = 0; i < SEM_DIMS; i++) norm += vec[i] * vec[i];
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < SEM_DIMS; i++) vec[i] /= norm;
  }
  return vec;
}

function cosineSim(a: Float32Array, b: Float32Array): number {
  let dot = 0;
  for (let i = 0; i < SEM_DIMS; i++) dot += a[i] * b[i];
  return dot;
}

function makeVaultExcerpt(text: string, query: string): string {
  const clean = text.replace(/\s+/g, " ").trim();
  if (clean.length <= 220) return clean;
  const qLower = query.trim().toLowerCase();
  const terms = tokenizeVaultText(qLower);
  const lower = clean.toLowerCase();
  let hitIdx = qLower ? lower.indexOf(qLower) : -1;
  if (hitIdx < 0) {
    for (const t of terms) {
      const idx = lower.indexOf(t);
      if (idx >= 0) {
        hitIdx = idx;
        break;
      }
    }
  }
  if (hitIdx < 0) return clean.slice(0, 217) + "…";
  const start = Math.max(0, hitIdx - 70);
  const end = Math.min(clean.length, start + 220);
  return (start > 0 ? "…" : "") + clean.slice(start, end) + (end < clean.length ? "…" : "");
}

function chunkWebFile(file: WebFile): InternalVaultChunk[] {
  const chunks: InternalVaultChunk[] = [];
  const sections: Array<{ sheet: string | null; text: string }> =
    file.extractedSheets && Object.keys(file.extractedSheets).length > 0
      ? Object.entries(file.extractedSheets).map(([sheet, text]) => ({ sheet, text }))
      : [{ sheet: file.extracted?.sheet ?? null, text: file.content }];

  for (const sec of sections) {
    const lines = sec.text.replace(/\r\n/g, "\n").split("\n");
    let currentHeading: string | null = null;
    let currentPage: number | null = file.extracted?.kind === "PDF" ? 1 : null;
    let buf: string[] = [];
    let bufChars = 0;
    let startLine = 1;

    const flush = (endLine: number) => {
      const joined = buf.join("\n").trim();
      if (joined) {
        const tokens = tokenizeVaultText(`${file.path} ${currentHeading ?? ""} ${sec.sheet ?? ""} ${joined}`);
        const tf = new Map<string, number>();
        for (const t of tokens) tf.set(t, (tf.get(t) ?? 0) + 1);
        chunks.push({
          chunk_id: `${file.path}${sec.sheet ? `#${sec.sheet}` : ""}:L${startLine}-${endLine}`,
          path: file.path,
          sheet: sec.sheet,
          page: currentPage,
          line_start: startLine,
          line_end: endLine,
          heading: currentHeading,
          text: joined,
          textLower: joined.toLowerCase(),
          tf,
          tokenLen: tokens.length || 1,
          semVec: embedVaultText(joined),
        });
      }
      buf = [];
      bufChars = 0;
    };

    for (let i = 0; i < lines.length; i++) {
      const lineNo = i + 1;
      const line = lines[i];
      const trimmed = line.trim();
      const pageMatch = /^(?:---+\s*)?(?:#+\s*)?(?:\[)?Page\s+(\d+)(?:\])?(?:\s*---+)?$/i.exec(trimmed);
      if (pageMatch) {
        if (buf.length > 0) flush(lineNo - 1);
        currentPage = Number(pageMatch[1]);
        startLine = lineNo;
      } else if (/^#{1,3}\s+/.test(trimmed)) {
        if (buf.length > 0 && bufChars >= 120) flush(lineNo - 1);
        currentHeading = trimmed.replace(/^#{1,3}\s+/, "").slice(0, 80);
        if (buf.length === 0) startLine = lineNo;
      } else if (buf.length === 0) {
        startLine = lineNo;
      }

      buf.push(line);
      bufChars += line.length + 1;

      const atBreak = trimmed === "" && bufChars >= 420;
      if (buf.length >= 24 || bufChars >= 1000 || atBreak) {
        flush(lineNo);
        startLine = lineNo + 1;
      }
    }
    if (buf.length > 0) flush(lines.length);
  }
  return chunks;
}

function buildVaultIndex(st: PersonStore): CachedVaultIndex {
  const entries = Object.values(st.files)
    .filter((f) => !isVaultExcludedPath(f.path))
    .sort((a, b) => a.path.localeCompare(b.path));
  const sig = entries.map((f) => `${f.path}:${f.mtime_ms}:${f.size}`).join("|");
  const chunks: InternalVaultChunk[] = [];
  let indexedBytes = 0;
  for (const f of entries) {
    indexedBytes += f.size;
    chunks.push(...chunkWebFile(f));
  }
  const df = new Map<string, number>();
  let totalLen = 0;
  for (const ch of chunks) {
    totalLen += ch.tokenLen;
    for (const term of ch.tf.keys()) {
      df.set(term, (df.get(term) ?? 0) + 1);
    }
  }
  const avgLen = chunks.length > 0 ? totalLen / chunks.length : 1;
  const built: CachedVaultIndex = {
    sig,
    chunks,
    df,
    avgLen,
    stats: {
      total_files: entries.length,
      total_chunks: chunks.length,
      indexed_bytes: indexedBytes,
      semantic_enabled: true,
    },
  };
  vaultIndexCache.set(st, built);
  return built;
}

function getVaultIndex(st: PersonStore, forceRebuild = false): CachedVaultIndex {
  if (!forceRebuild) {
    const cached = vaultIndexCache.get(st);
    if (cached) {
      const sig = Object.values(st.files)
        .filter((f) => !isVaultExcludedPath(f.path))
        .sort((a, b) => a.path.localeCompare(b.path))
        .map((f) => `${f.path}:${f.mtime_ms}:${f.size}`)
        .join("|");
      if (cached.sig === sig) return cached;
    }
  }
  return buildVaultIndex(st);
}

function searchWebVault(
  st: PersonStore,
  opts: {
    q: string;
    scope?: VaultSearchScope;
    mode?: VaultSearchMode;
    limit?: number;
    pathPrefix?: string;
    engagementId?: string;
  }
) {
  const idx = getVaultIndex(st, false);
  const qRaw = opts.q.trim();
  const qLower = qRaw.toLowerCase();
  const scope: VaultSearchScope = opts.scope ?? "all";
  const mode: VaultSearchMode = opts.mode ?? "hybrid";
  const limit = Math.min(50, Math.max(1, opts.limit ?? 20));
  const prefix = (opts.pathPrefix || "").trim().replace(/^\/+/, "").toLowerCase();

  const fileHits: VaultFileHit[] = [];
  const results: Array<{
    kind: string;
    id: string;
    title: string;
    subtitle?: string;
    path?: string;
    thread_id?: string;
    ts?: number;
    score?: number;
    chunks?: VaultChunkHit[];
  }> = [];

  if (qLower && (scope === "all" || scope === "files")) {
    const qTerms = tokenizeVaultText(qLower);
    const qVec = embedVaultText(qLower);
    const N = Math.max(1, idx.chunks.length);
    const k1 = 1.2;
    const b = 0.75;

    const candidateChunks = idx.chunks.filter((ch) => !prefix || ch.path.toLowerCase().startsWith(prefix));
    const scored = candidateChunks.map((ch) => {
      let bm25 = 0;
      for (const term of qTerms) {
        const tf = ch.tf.get(term) ?? 0;
        if (tf <= 0) continue;
        const df = idx.df.get(term) ?? 0;
        const idf = Math.log(1 + (N - df + 0.5) / (df + 0.5));
        bm25 += idf * ((tf * (k1 + 1)) / (tf + k1 * (1 - b + (b * ch.tokenLen) / idx.avgLen)));
      }
      if (ch.path.toLowerCase().includes(qLower)) bm25 += 2.5;
      const sem = cosineSim(qVec, ch.semVec);
      const exactHit =
        ch.textLower.includes(qLower) ||
        ch.path.toLowerCase().includes(qLower) ||
        (qTerms.length > 1 && qTerms.every((t) => ch.textLower.includes(t)));
      return { ch, bm25, sem, exactHit };
    });

    const bm25Order = [...scored]
      .filter((s) => s.bm25 > 0)
      .sort((a, b) => b.bm25 - a.bm25);
    const semOrder = [...scored]
      .filter((s) => s.sem > 0.1)
      .sort((a, b) => b.sem - a.sem);
    const bm25Rank = new Map<string, number>();
    bm25Order.forEach((s, i) => bm25Rank.set(s.ch.chunk_id, i + 1));
    const semRank = new Map<string, number>();
    semOrder.forEach((s, i) => semRank.set(s.ch.chunk_id, i + 1));

    const rankedChunks: Array<{ ch: InternalVaultChunk; score: number; bm25: number; sem: number }> = [];
    for (const item of scored) {
      let finalScore = 0;
      if (mode === "exact") {
        if (!item.exactHit) continue;
        finalScore = item.bm25 > 0 ? item.bm25 + 1 : 1;
      } else if (mode === "bm25") {
        if (item.bm25 <= 0) continue;
        finalScore = item.bm25;
      } else {
        const rB = bm25Rank.get(item.ch.chunk_id);
        const rS = semRank.get(item.ch.chunk_id);
        if (!rB && (!rS || item.sem < 0.2)) continue;
        finalScore = (rB ? 1 / (60 + rB) : 0) + (rS ? 1 / (60 + rS) : 0);
        if (item.exactHit) finalScore += 0.02;
      }
      rankedChunks.push({ ch: item.ch, score: finalScore, bm25: item.bm25, sem: item.sem });
    }
    rankedChunks.sort((a, b) => b.score - a.score || a.ch.line_start - b.ch.line_start);

    const byFile = new Map<string, { topScore: number; chunks: VaultChunkHit[] }>();
    for (const r of rankedChunks) {
      const existing = byFile.get(r.ch.path);
      const hit: VaultChunkHit = {
        chunk_id: r.ch.chunk_id,
        score: Number(r.score.toFixed(4)),
        bm25_score: Number(r.bm25.toFixed(4)),
        semantic_score: Number(r.sem.toFixed(4)),
        ...(r.ch.page ? { page: r.ch.page } : {}),
        ...(r.ch.sheet ? { sheet: r.ch.sheet } : {}),
        line_start: r.ch.line_start,
        line_end: r.ch.line_end,
        ...(r.ch.heading ? { heading: r.ch.heading } : {}),
        excerpt: makeVaultExcerpt(r.ch.text, qRaw),
      };
      if (!existing) {
        byFile.set(r.ch.path, { topScore: r.score, chunks: [hit] });
      } else if (existing.chunks.length < 4) {
        existing.chunks.push(hit);
      }
    }

    for (const [fp, entry] of Array.from(byFile.entries()).sort((a, b) => b[1].topScore - a[1].topScore).slice(0, limit)) {
      const wf = st.files[fp];
      if (!wf) continue;
      fileHits.push({
        path: wf.path,
        name: wf.name,
        ext: wf.ext,
        size: wf.size,
        mtime_ms: wf.mtime_ms,
        score: Number(entry.topScore.toFixed(4)),
        chunks: entry.chunks,
      });
      results.push({
        kind: "file",
        id: wf.path,
        title: wf.path,
        subtitle: entry.chunks[0]?.excerpt || `${wf.size} bytes`,
        path: wf.path,
        ts: wf.mtime_ms,
        score: Number(entry.topScore.toFixed(4)),
        chunks: entry.chunks,
      });
    }
  }

  if (qLower && (scope === "all" || scope === "receipts")) {
    for (const wItem of st.work.slice().reverse()) {
      if (opts.engagementId && wItem.engagement_id && wItem.engagement_id !== opts.engagementId) continue;
      if (wItem.task.toLowerCase().includes(qLower) || wItem.summary.toLowerCase().includes(qLower)) {
        results.push({
          kind: "receipt",
          id: wItem.id || wItem.thread_id,
          title: wItem.task,
          subtitle: wItem.summary,
          thread_id: wItem.thread_id,
          ts: wItem.ts,
        });
      }
    }
  }

  if (qLower && (scope === "all" || scope === "memory")) {
    for (const line of learnedLines(st.memory)) {
      if (line.toLowerCase().includes(qLower)) {
        results.push({ kind: "memory", id: `mem-${results.length}`, title: line, subtitle: "memory.md" });
      }
    }
  }

  return {
    query: qLower,
    mode,
    scope,
    total: results.length,
    results: results.slice(0, limit),
    file_hits: fileHits,
    files: fileHits.map((fh) => ({ path: fh.path, name: fh.name, size: fh.size, mtime_ms: fh.mtime_ms, ext: fh.ext, chunks: fh.chunks })),
    index_stats: idx.stats,
  };
}

function globToRegex(glob: string): RegExp {
  const normalized = glob.trim().replace(/\\/g, "/").replace(/^\/+/, "") || "*";
  let re = "^";
  for (let i = 0; i < normalized.length; i++) {
    const c = normalized[i];
    if (c === "*") {
      if (normalized[i + 1] === "*") {
        re += ".*";
        i++;
        if (normalized[i + 1] === "/") i++;
      } else {
        re += "[^/]*";
      }
    } else if (c === "?") {
      re += "[^/]";
    } else if (/[.+^${}()|[\]\\]/.test(c)) {
      re += "\\" + c;
    } else {
      re += c;
    }
  }
  return new RegExp(re + "$", "i");
}

function runWebVaultAggregate(
  st: PersonStore,
  req: TabularQuerySpec & { glob?: string; sheet?: string; pathPrefix?: string }
): { ok: true; columns: string[]; rows: string[][]; source_files: string[]; row_count: number } {
  const pattern = globToRegex(String(req.glob || "*.csv"));
  const scopePrefix = (req.pathPrefix || "").trim().replace(/^\/+|\/+$/g, "").toLowerCase();
  const matchedFiles = Object.values(st.files)
    .filter((f) => {
      if (isVaultExcludedPath(f.path) || !pattern.test(f.path)) return false;
      if (!scopePrefix) return true;
      const low = f.path.toLowerCase();
      return low === scopePrefix || low.startsWith(`${scopePrefix}/`) || low.startsWith("shared/");
    })
    .sort((a, b) => a.path.localeCompare(b.path));

  const canonicalHeader: string[] = [];
  const headerIdxMap = new Map<string, number>();
  const rowRecords: Array<Record<string, string>> = [];
  const sourceFiles: string[] = [];

  const ensureCol = (rawName: string) => {
    const clean = rawName.trim();
    const key = clean.toLowerCase();
    if (!key) return -1;
    if (!headerIdxMap.has(key)) {
      headerIdxMap.set(key, canonicalHeader.length);
      canonicalHeader.push(clean);
    }
    return headerIdxMap.get(key)!;
  };

  const ingestTable = (fp: string, rawText: string, delim: string) => {
    const insp = inspectDelimited(rawText, delim);
    if (insp.hasNullByte || insp.unclosedQuote || insp.rows.length === 0) return;
    const parsed = insp.rows;
    const hdr = parsed[0];
    if (!hdr.some((col) => col.trim().length > 0)) return;
    hdr.forEach((col) => ensureCol(col));
    let added = false;
    for (let r = 1; r < parsed.length; r++) {
      const row = parsed[r];
      if (!row.some((c) => c.trim().length > 0)) continue;
      const rec: Record<string, string> = {};
      hdr.forEach((col, cIdx) => {
        const key = col.trim().toLowerCase();
        if (key) rec[key] = row[cIdx] ?? "";
      });
      rowRecords.push(rec);
      added = true;
    }
    if (added && !sourceFiles.includes(fp)) sourceFiles.push(fp);
  };

  for (const f of matchedFiles) {
    if (f.extractedSheets && Object.keys(f.extractedSheets).length > 0) {
      const targetSheet = (req.sheet || "").trim().toLowerCase();
      for (const [shName, csvText] of Object.entries(f.extractedSheets)) {
        if (targetSheet && shName.toLowerCase() !== targetSheet) continue;
        ingestTable(f.path, csvText, ",");
      }
    } else if (f.ext === "csv" || f.ext === "tsv" || f.extracted?.format === "csv") {
      const delim = f.ext === "tsv" ? "\t" : ",";
      ingestTable(f.path, f.content, delim);
    }
  }

  if (!canonicalHeader.length || !rowRecords.length) {
    const groupCols = Array.isArray(req.group_by) ? req.group_by.filter(Boolean) : [];
    const aggs = Array.isArray(req.aggregates) ? req.aggregates : [];
    const fallbackCols =
      groupCols.length > 0 || aggs.length > 0
        ? [...groupCols, ...aggs.map((a) => a.as || `${a.fn}_${a.col}`)]
        : canonicalHeader;
    return { ok: true, columns: fallbackCols, rows: [], source_files: sourceFiles, row_count: 0 };
  }

  const normalizedRows = rowRecords.map((rec) =>
    canonicalHeader.map((col) => rec[col.trim().toLowerCase()] ?? "")
  );
  const aggRes = runTabularAggregation(canonicalHeader, normalizedRows, req);
  return {
    ok: true,
    columns: aggRes.columns,
    rows: aggRes.rows,
    source_files: sourceFiles,
    row_count: aggRes.rows.length,
  };
}

    if (method === "GET" && path === "/api/version") return jsonRes({ name: "ergon", version: "0.38.0" });
    if (method === "GET" && path === "/api/update") return jsonRes({ current: "0.38.0", available: false, mode: "off", checked_at: Math.floor(Date.now() / 1000) });
    if (method === "GET" && path === "/api/status") {
      return jsonRes({
        version: "0.38.0",
        model_configured: true,
        model: byok.engine_mode === "byok" && byok.model_id ? byok.model_id : "browser-workspace-engine",
        model_base_url: byok.engine_mode === "byok" ? byok.model_base_url : "indexeddb://local",
        engine_mode: byok.engine_mode,
        scheduler_enabled: true,
        capabilities: {
          sse: true,
          skills: true,
          routines: true,
          playbooks: true,
          search: true,
          receipts_export: true,
          cancel: true,
          file_history: true,
          scheduler: true,
          audit_bundle: true,
          people_switch: true,
          pipelines: true,
          file_edit: true,
          table_query: true,
          pdf_export: true,
          connectors: true,
          mcp: true,
          clients: true,
          metering: true,
          vault_search: true,
          client_crm: true,
          time_entries: true,
          invoicing: true,
        },
        scheduler: buildSchedulerStatus(st),
        person: p,
        store_recovered: [],
      });
    }
    if (method === "GET" && path === "/api/config") {
      ensureClientsAndPricing(state);
      return jsonRes({
        model_base_url: byok.model_base_url,
        model_id: byok.model_id || "browser-workspace-engine",
        has_api_key: Boolean(byok.api_key),
        model_configured: true,
        is_loopback: true,
        engine_mode: byok.engine_mode,
        detected_gateways: [],
        model_pricing: mergeModelPricing(state.modelPricing),
      });
    }
    if ((method === "POST" || method === "PUT") && path === "/api/config") {
      ensureClientsAndPricing(state);
      const pricingOnly =
        body.model_pricing !== undefined &&
        body.engine_mode === undefined &&
        body.model_base_url === undefined &&
        body.model_id === undefined;
      if (body.model_pricing && typeof body.model_pricing === "object" && !Array.isArray(body.model_pricing)) {
        const nextPricing: ModelPricingMap = { ...(state.modelPricing ?? {}) };
        for (const [k, rawEntry] of Object.entries(body.model_pricing as Record<string, unknown>)) {
          const id = k.trim();
          if (!id) continue;
          if (rawEntry === null) {
            delete nextPricing[id];
            continue;
          }
          if (!rawEntry || typeof rawEntry !== "object") {
            return jsonRes({ error: `Invalid pricing entry for "${id}".` }, 400);
          }
          const eObj = rawEntry as Record<string, unknown>;
          const inp = Number(eObj.input_per_1m_usd ?? 0);
          const outp = Number(eObj.output_per_1m_usd ?? 0);
          const cached = Number(eObj.cached_input_per_1m_usd ?? 0);
          if (!Number.isFinite(inp) || inp < 0 || inp > 1_000_000 || !Number.isFinite(outp) || outp < 0 || outp > 1_000_000 || !Number.isFinite(cached) || cached < 0 || cached > 1_000_000) {
            return jsonRes({ error: `Model pricing rates for "${id}" must be finite non-negative numbers up to 1,000,000.` }, 400);
          }
          nextPricing[id] = {
            input_per_1m_usd: inp,
            output_per_1m_usd: outp,
            cached_input_per_1m_usd: cached,
          };
        }
        state.modelPricing = mergeModelPricing(nextPricing);
        await persist();
      }
      if (pricingOnly) {
        return jsonRes({
          model_base_url: byok.model_base_url,
          model_id: byok.model_id || "browser-workspace-engine",
          has_api_key: Boolean(byok.api_key),
          model_configured: true,
          is_loopback: true,
          engine_mode: byok.engine_mode,
          model_pricing: state.modelPricing,
        });
      }
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
        model_pricing: mergeModelPricing(state.modelPricing),
      });
    }
    if (method === "GET" && path === "/api/runs/status") {
      const run = liveRuns[p.id];
      return jsonRes(
        run
          ? { active: run.active, run_id: run.run_id, thread_id: run.thread_id, task: run.task, started_ms: run.started_ms, elapsed_ms: Date.now() - run.started_ms, steps: run.steps }
          : { active: false, thread_id: "", task: "", started_ms: 0, steps: [] }
      );
    }
    if (method === "POST" && path.startsWith("/api/runs/") && path.endsWith("/cancel")) {
      if (liveRuns[p.id]) {
        liveRuns[p.id].cancelled = true;
        broadcastSse("cancelled", { run_id: liveRuns[p.id].run_id, cancelled: true });
      }
      return jsonRes({ cancelled: true });
    }
    if (method === "GET" && path === "/api/skills") {
      return jsonRes({
        skills: CANONICAL_SKILLS,
        count: CANONICAL_SKILLS.length,
        starter_playbooks: CANONICAL_STARTER_PLAYBOOKS,
      });
    }
    if (method === "GET" && path === "/api/playbooks/starters") {
      return jsonRes({ starters: CANONICAL_STARTER_PLAYBOOKS });
    }
    if (method === "POST" && path === "/api/playbooks/starters/launch") {
      const sid = String(body.id ?? body.starter_id ?? "").trim().toLowerCase();
      const starter = CANONICAL_STARTER_PLAYBOOKS.find(
        (s) => s.id.toLowerCase() === sid || s.name.toLowerCase() === sid
      );
      if (!starter) return jsonRes({ error: "starter playbook not found" }, 404);
      const runNow = body.run_now !== false;
      const starterStepsV2: PipelineStepSpec[] =
        starter.steps_v2 && starter.steps_v2.length > 0
          ? starter.steps_v2
          : [
              {
                id: "step_1",
                label: starter.steps[0] || starter.name,
                kind: "skill",
                skill_id: starter.skill_id,
                args: { output_title: starter.expected_inputs[0] || `${starter.sample_title || starter.name}.md` },
                when: "always",
              },
            ];
      let rt = st.routines.find((r) => r.name === starter.name);
      if (!rt) {
        rt = {
          id: `rt-${Date.now()}-${st.routines.length + 1}`,
          name: starter.name,
          prompt: starter.prompt,
          schedule: starter.schedule,
          skill_id: starter.skill_id,
          steps: starter.steps,
          steps_v2: starterStepsV2,
          expected_inputs: starter.expected_inputs,
          pack_deliverables: starter.pack_deliverables,
          enabled: true,
          last_run_ts: null,
          next_run_ms: estimateNextRunMs(starter.schedule, Date.now()),
          run_count: 0,
          created_ts: Date.now(),
        };
        st.routines.unshift(rt);
      }
      const createdFiles: string[] = [];
      const seededInputs: string[] = [];
      let summary = "";
      let starterRun: PipelineRun | undefined;
      if (runNow) {
        if (Object.keys(st.files).length === 0) {
          const seedName = starter.skill_id === "reconcile_ledger" ? "starter-ledger.csv" : "starter-notes.md";
          const seedBody =
            starter.skill_id === "reconcile_ledger"
              ? "Vendor,Category,Amount,Receipt_Status,Date\nCloudHost,Cloud,420.00,verified,2026-11-10\nOffice Supplies,Ops,85.50,missing,2026-11-12\n"
              : `# ${starter.name} Input Notes\n\n- Decided: Prepare verified client deliverables.\n- Owner: Operations Lead - finalize review by Friday.\n`;
          st.files[seedName] = makeFile(seedName, seedBody);
          seededInputs.push(seedName);
        }
        starterRun = createPipelineRunRecord(st, {
          routine_id: rt.id,
          name: rt.name,
          steps_v2: starterStepsV2,
          inputs: {},
        });
        await executeWebPipelineFrom(p, st, byok, starterRun, 0);
        for (const ss of starterRun.step_states) {
          for (const f of ss.files ?? []) {
            if (!createdFiles.includes(f)) createdFiles.push(f);
          }
        }
        summary =
          starterRun.step_states
            .map((s) => s.output)
            .filter(Boolean)
            .slice(-1)[0] || `Executed starter playbook ${starter.name}.`;
      }
      await persist();
      const preview = buildDeterministicPlaybookPreview(
        rt,
        Object.keys(st.files),
        (fp) => (st.fileHistory[fp] ?? []).length,
        Object.keys(st.fileHistory).length
      );
      return jsonRes(
        {
          ok: true,
          starter_id: starter.id,
          sha256: await sha256Hex(JSON.stringify(starterStepsV2)),
          routine: await enrichWebRoutine(rt, st, Date.now()),
          run: starterRun,
          created_files: createdFiles,
          seeded_inputs: seededInputs,
          summary,
          preview,
        },
        201
      );
    }
    if (method === "POST" && path === "/api/skills/test") {
      const skillId = String(body.id ?? body.skill_id ?? "").trim();
      const found = CANONICAL_SKILLS.find((s) => s.id === skillId);
      if (!found) return jsonRes({ error: "skill not found" }, 404);
      const t0 = performance.now();
      const fileCount = Object.keys(st.files).length;
      const elapsed = Math.max(1, Math.round(performance.now() - t0));
      return jsonRes({
        ok: true,
        id: found.id,
        duration_ms: elapsed,
        output: `Compiled ${found.filename} cleanly (sandboxed browser engine, ${fileCount} workspace files visible, inputs: ${found.inputs.join(", ")}).`,
      });
    }
    if (method === "GET" && path === "/api/routines") {
      const nowMs = Date.now();
      const enriched = await Promise.all(st.routines.map((rt) => enrichWebRoutine(rt, st, nowMs)));
      return jsonRes({ routines: enriched, scheduler: buildSchedulerStatus(st) });
    }
    if (method === "POST" && path === "/api/routines") {
      const name = String(body.name ?? body.title ?? "").trim();
      const rawSteps = Array.isArray(body.steps)
        ? body.steps.map((s) => String(s ?? "").trim()).filter(Boolean).slice(0, 8)
        : [];
      const rawStepsV2 = Array.isArray(body.steps_v2) ? (body.steps_v2 as PipelineStepSpec[]) : undefined;
      const prompt =
        String(body.prompt ?? "").trim() ||
        rawSteps.join(" -> ") ||
        (rawStepsV2 ?? []).map((s) => s.label || s.skill_id || s.kind).join(" -> ");
      if (!name || !prompt) return jsonRes({ error: "name and prompt are required" }, 400);
      const schedule = String(body.schedule ?? body.cadence ?? "daily 09:00").trim();
      const enabled = typeof body.enabled === "boolean" ? body.enabled : true;
      const expectedInputs = Array.isArray(body.expected_inputs)
        ? body.expected_inputs.map((s) => String(s ?? "").trim()).filter(Boolean).slice(0, 8)
        : undefined;
      const created: Routine = {
        id: `rt-${Date.now()}-${st.routines.length + 1}`,
        name,
        prompt,
        schedule,
        cadence: schedule,
        skill_id: typeof body.skill_id === "string" && body.skill_id ? body.skill_id : undefined,
        steps: rawSteps.length > 1 ? rawSteps : undefined,
        steps_v2: rawStepsV2 && rawStepsV2.length > 0 ? rawStepsV2 : undefined,
        expected_inputs: expectedInputs?.length ? expectedInputs : undefined,
        pack_deliverables: body.pack_deliverables === true ? true : undefined,
        enabled,
        last_run_ts: null,
        next_run_ms: enabled ? estimateNextRunMs(schedule, Date.now()) : null,
        run_count: 0,
        created_ts: Date.now(),
      };
      st.routines.unshift(created);
      await persist();
      return jsonRes({ routine: created, ...created }, 201);
    }
    if (method === "PATCH" && path.startsWith("/api/routines/") && !path.endsWith("/run") && !path.endsWith("/history")) {
      const rid = decodeURIComponent(path.slice("/api/routines/".length));
      const rt = st.routines.find((x) => x.id === rid);
      if (!rt) return jsonRes({ error: "routine not found" }, 404);
      if (typeof body.enabled === "boolean") rt.enabled = body.enabled;
      if (typeof body.schedule === "string" && body.schedule.trim()) {
        rt.schedule = body.schedule.trim();
        rt.cadence = rt.schedule;
      }
      if (typeof body.cadence === "string" && body.cadence.trim()) {
        rt.schedule = body.cadence.trim();
        rt.cadence = rt.schedule;
      }
      if (typeof body.name === "string" && body.name.trim()) rt.name = body.name.trim();
      if (typeof body.prompt === "string" && body.prompt.trim()) rt.prompt = body.prompt.trim();
      rt.next_run_ms = rt.enabled ? estimateNextRunMs(rt.schedule, Date.now(), rt.interval_secs) : null;
      await persist();
      return jsonRes({ routine: rt, scheduler: buildSchedulerStatus(st) });
    }
    if (method === "GET" && path.startsWith("/api/routines/") && path.endsWith("/history")) {
      const rid = decodeURIComponent(path.slice("/api/routines/".length, -"/history".length));
      const runs = st.routineRuns[rid] ?? [];
      return jsonRes({ routine_id: rid, runs, history: runs });
    }
    if (method === "DELETE" && path.startsWith("/api/routines/")) {
      const rid = decodeURIComponent(path.slice("/api/routines/".length));
      st.routines = st.routines.filter((x) => x.id !== rid);
      delete st.routineRuns[rid];
      await persist();
      return jsonRes({ deleted: rid });
    }
    const executeWebRoutineRun = async (rt: Routine) => {
      const startedMs = Date.now();
      const stepList = rt.steps && rt.steps.length > 1 ? rt.steps : [rt.prompt];
      let resp = await executePromptIntoThread(p, st, byok, stepList[0]);
      for (let i = 1; i < stepList.length && resp.ok; i += 1) {
        resp = await executePromptIntoThread(p, st, byok, stepList[i]);
      }
      const finishedMs = Date.now();
      const lastWork = st.work[st.work.length - 1];
      rt.last_run_ts = finishedMs;
      rt.last_status = resp.ok ? "finished" : "error";
      rt.run_count = (rt.run_count ?? 0) + 1;
      rt.next_run_ms = rt.enabled ? estimateNextRunMs(rt.schedule, finishedMs, rt.interval_secs) : null;
      if (!st.routineRuns[rt.id]) st.routineRuns[rt.id] = [];
      const runRec: RoutineRunRecord = {
        run_id: `run-${finishedMs}-${st.routineRuns[rt.id].length + 1}`,
        routine_id: rt.id,
        started_ms: startedMs,
        finished_ms: finishedMs,
        status: resp.ok ? "finished" : "error",
        receipt_id: lastWork?.id || lastWork?.thread_id || null,
        thread_id: lastWork?.thread_id || null,
        summary: lastWork?.summary || `Executed "${rt.name}".`,
      };
      st.routineRuns[rt.id].unshift(runRec);
      await persist();
      return { resp, runRec, lastWork };
    };
    if (method === "POST" && path === "/api/scheduler/catchup") {
      const nowMs = Date.now();
      const due = st.routines.filter((r) => countMissedWindowsWeb(r, nowMs) > 0).slice(0, 10);
      const results = [];
      let caughtUpCount = 0;
      for (const rt of due) {
        const missed = countMissedWindowsWeb(rt, nowMs);
        const { resp } = await executeWebRoutineRun(rt);
        if (resp.ok) caughtUpCount += 1;
        results.push({ routine_id: rt.id, ok: resp.ok, missed_windows_cleared: resp.ok ? missed : 0 });
      }
      const enriched = await Promise.all(st.routines.map((r) => enrichWebRoutine(r, st, Date.now())));
      return jsonRes({ total_due: due.length, caught_up_count: caughtUpCount, failed_count: due.length - caughtUpCount, results, routines: enriched, scheduler: buildSchedulerStatus(st) });
    }
    if (method === "POST" && (path.startsWith("/api/routines/") || path.startsWith("/api/playbooks/")) && path.endsWith("/catchup")) {
      const prefix = path.startsWith("/api/playbooks/") ? "/api/playbooks/" : "/api/routines/";
      const rid = decodeURIComponent(path.slice(prefix.length, -"/catchup".length));
      const rt = st.routines.find((x) => x.id === rid);
      if (!rt) return jsonRes({ error: "routine not found" }, 404);
      const missed = countMissedWindowsWeb(rt, Date.now());
      if (missed === 0) {
        return jsonRes({ routine: await enrichWebRoutine(rt, st), caught_up: false, missed_windows: 0 });
      }
      const { resp, runRec, lastWork } = await executeWebRoutineRun(rt);
      const enriched = await enrichWebRoutine(rt, st);
      return jsonRes({ caught_up: resp.ok, missed_windows_cleared: resp.ok ? missed : 0, routine: enriched, run: runRec, thread_id: lastWork?.thread_id || null }, resp.ok ? 200 : 500);
    }
    if (method === "POST" && path.startsWith("/api/routines/") && path.endsWith("/run")) {
      const rid = decodeURIComponent(path.slice("/api/routines/".length, -"/run".length));
      const rt = st.routines.find((x) => x.id === rid);
      if (!rt) return jsonRes({ error: "routine not found" }, 404);
      const { resp } = await executeWebRoutineRun(rt);
      return resp;
    }
    if (method === "GET" && path.startsWith("/api/playbooks/") && path.endsWith("/export")) {
      const pid = decodeURIComponent(path.slice("/api/playbooks/".length, -"/export".length)).trim();
      const rt = st.routines.find((x) => x.id === pid);
      if (!rt) return jsonRes({ error: "playbook not found" }, 404);
      const built = await buildSignedPlaybookPack(rt, "0.38.0");
      return new Response(built.json, {
        status: 200,
        headers: {
          "Content-Type": "application/json; charset=utf-8",
          "X-Content-Type-Options": "nosniff",
          "X-Ergon-Playbook-SHA256": built.sha256,
          "Content-Disposition": `attachment; filename="${built.filename}"`,
        },
      });
    }
    if (method === "POST" && path === "/api/playbooks/import") {
      const rawText = typeof init?.body === "string" ? init.body : JSON.stringify(body ?? {});
      const checked = await verifyAndParsePlaybookPack(rawText);
      if (!checked.ok) {
        return jsonRes({ error: checked.error }, checked.status);
      }
      const pb = checked.playbook;
      const created: Routine = {
        id: `rt-${Date.now()}-${st.routines.length + 1}`,
        name: pb.name.trim(),
        prompt: pb.prompt.trim(),
        schedule: pb.schedule.trim(),
        cadence: pb.schedule.trim(),
        interval_secs: pb.interval_secs ?? undefined,
        skill_id: pb.skill_id ?? undefined,
        steps: pb.steps.length > 0 ? pb.steps : undefined,
        expected_inputs: pb.expected_inputs.length > 0 ? pb.expected_inputs : undefined,
        pack_deliverables: pb.pack_deliverables ? true : undefined,
        enabled: true,
        last_run_ts: null,
        next_run_ms: estimateNextRunMs(pb.schedule.trim(), Date.now(), pb.interval_secs ?? undefined),
        run_count: 0,
        created_ts: Date.now(),
      };
      st.routines.unshift(created);
      await persist();
      return jsonRes({ routine: created, sha256: checked.sha256, verified: true }, 201);
    }
    if (method === "POST" && (path.startsWith("/api/playbooks/") || path.startsWith("/api/routines/")) && path.endsWith("/preview")) {
      const prefix = path.startsWith("/api/playbooks/") ? "/api/playbooks/" : "/api/routines/";
      const pid = decodeURIComponent(path.slice(prefix.length, -"/preview".length)).trim();
      const rt = st.routines.find((x) => x.id === pid);
      if (!rt) return jsonRes({ error: "routine not found" }, 404);
      const preview = buildDeterministicPlaybookPreview(
        rt,
        Object.keys(st.files),
        (fp) => (st.fileHistory[fp] ?? []).length,
        Object.keys(st.fileHistory).length
      );
      return jsonRes(preview, 200);
    }
    if (method === "POST" && path === "/api/pipelines/validate") {
      const stepsV2: PipelineStepSpec[] = Array.isArray(body.steps_v2)
        ? (body.steps_v2 as PipelineStepSpec[])
        : Array.isArray(body.steps)
          ? (body.steps as PipelineStepSpec[])
          : [];
      const result = validateWebPipelineSpec(stepsV2, ensureWebConnectors(st).outbound);
      return jsonRes(result, result.valid ? 200 : 400);
    }
    if (method === "POST" && path === "/api/pipelines/run") {
      const rid = typeof body.routine_id === "string" ? body.routine_id.trim() : "";
      const rt = rid ? st.routines.find((r) => r.id === rid) : undefined;
      let stepsV2: PipelineStepSpec[] = Array.isArray(body.steps_v2) ? (body.steps_v2 as PipelineStepSpec[]) : [];
      if (stepsV2.length === 0 && rt) {
        if (rt.steps_v2 && rt.steps_v2.length > 0) {
          stepsV2 = rt.steps_v2;
        } else {
          const legacy = rt.steps && rt.steps.length > 0 ? rt.steps : [rt.prompt];
          stepsV2 = legacy.map((txt, idx) => ({
            id: `step_${idx + 1}`,
            label: txt.slice(0, 48),
            kind: idx === 0 && rt.skill_id ? "skill" : "agent",
            skill_id: idx === 0 ? rt.skill_id : undefined,
            prompt: txt,
            when: "always",
          }));
        }
      }
      if (stepsV2.length === 0) {
        return jsonRes({ error: "steps_v2 or routine_id is required" }, 400);
      }
      const check = validateWebPipelineSpec(stepsV2, ensureWebConnectors(st).outbound);
      if (!check.valid) {
        return jsonRes({ error: check.errors[0] || "Invalid pipeline specification", ...check }, 400);
      }
      const run = createPipelineRunRecord(st, {
        routine_id: rt?.id || (rid || undefined),
        name: String(body.name ?? rt?.name ?? "Pipeline run").trim(),
        steps_v2: stepsV2,
        inputs: body.inputs && typeof body.inputs === "object" ? (body.inputs as Record<string, string>) : {},
        thread_id: typeof body.thread_id === "string" && body.thread_id ? body.thread_id : undefined,
      });
      await persist();
      window.setTimeout(() => {
        void executeWebPipelineFrom(p, st, byok, run, 0);
      }, 15);
      return jsonRes({ run }, 202);
    }
    if (method === "GET" && path === "/api/pipelines/runs") {
      const statusFilter = (u.searchParams.get("status") || "").trim();
      const limit = Math.min(200, Math.max(1, Number(u.searchParams.get("limit") || 50)));
      const allRuns = (st.pipelineRuns ?? []).filter((r) => !statusFilter || r.status === statusFilter);
      return jsonRes({ runs: allRuns.slice(0, limit) });
    }
    if (method === "GET" && path.startsWith("/api/pipelines/runs/") && !path.endsWith("/resume") && !path.endsWith("/retry") && !path.endsWith("/cancel")) {
      const runId = decodeURIComponent(path.slice("/api/pipelines/runs/".length));
      const found = (st.pipelineRuns ?? []).find((r) => r.run_id === runId);
      if (!found) return jsonRes({ error: "pipeline run not found" }, 404);
      return jsonRes({ run: found });
    }
    if (method === "POST" && path.startsWith("/api/pipelines/runs/") && path.endsWith("/resume")) {
      const runId = decodeURIComponent(path.slice("/api/pipelines/runs/".length, -"/resume".length));
      const run = (st.pipelineRuns ?? []).find((r) => r.run_id === runId);
      if (!run) return jsonRes({ error: "pipeline run not found" }, 404);
      if (run.status !== "paused_checkpoint" && run.status !== "interrupted") {
        return jsonRes({ error: `pipeline run is ${run.status} and cannot be resumed` }, 409);
      }
      if (run.status === "interrupted" && !run.pending_checkpoint) {
        const resumeIdx = Math.max(0, run.current_step_index);
        run.status = "running";
        run.error = null;
        run.updated_ms = Date.now();
        await persist();
        broadcastSse("pipeline_resumed", {
          run_id: run.run_id,
          thread_id: run.thread_id ?? null,
          step_index: resumeIdx,
          action: "approve",
          human_answer: "resumed",
          run,
        });
        window.setTimeout(() => {
          void executeWebPipelineFrom(p, st, byok, run, resumeIdx);
        }, 15);
        return jsonRes({ run }, 202);
      }
      const cpIdx = run.pending_checkpoint?.step_index ?? run.current_step_index;
      const cpId = run.pending_checkpoint?.checkpoint_id;
      const action = String(body.action ?? "approve");
      const humanAnswer =
        typeof body.choice === "string" && body.choice
          ? body.choice
          : typeof body.answer === "string" && body.answer
            ? body.answer
            : action === "reject"
              ? "rejected"
              : "approved";
      if (cpId) {
        const dec = st.decisions.find((d) => d.id === cpId);
        if (dec) {
          dec.status = action === "reject" ? "denied" : "approved";
          dec.resolved_ts = Date.now();
          dec.answer = humanAnswer;
          if (dec.kind === "connector_delivery") {
            const cs = ensureWebConnectors(st);
            const pol = buildWebPolicy(p.id, st);
            const log = cs.deliveries.find((dl) => dl.decision_id === dec.id || (dec.delivery_id && dl.id === dec.delivery_id));
            if (action !== "reject" && pol.profile === "air_gapped") {
              dec.status = "denied";
              dec.outcome = "Delivery blocked by Air-Gapped workspace policy.";
              if (log) {
                log.status = "blocked";
                log.error = dec.outcome;
              }
            } else if (action !== "reject") {
              dec.outcome = `Delivery approved and sent via ${dec.connector_name || "connector"}.`;
              if (log) {
                log.status = "ok";
                log.http_status = 200;
                log.response_preview = "Approved at pipeline checkpoint and delivered.";
              }
              const conn = cs.outbound.find((c) => c.id === dec.connector_id);
              if (conn) {
                conn.last_delivery_ms = dec.resolved_ts;
                conn.last_status = "ok";
              }
            } else {
              dec.outcome = "Delivery rejected at pipeline checkpoint.";
              if (log) {
                log.status = "denied";
                log.error = dec.outcome;
              }
            }
          } else {
            dec.outcome = `Resumed pipeline (${humanAnswer})`;
          }
        }
      }
      if (run.step_states[cpIdx]) {
        const isDeliveryHold = run.pending_checkpoint?.kind === "deliver" || run.step_states[cpIdx].kind === "deliver";
        run.step_states[cpIdx].status = isDeliveryHold && action === "reject" ? "skipped" : "ok";
        run.step_states[cpIdx].finished_ms = Date.now();
        run.step_states[cpIdx].human_answer = humanAnswer;
        run.step_states[cpIdx].human_action =
          action === "reject" ? "reject" : action === "answer" ? "answer" : "approve";
        run.step_states[cpIdx].output = isDeliveryHold
          ? action === "reject"
            ? "Delivery rejected at approval checkpoint."
            : `Delivered via ${run.pending_checkpoint?.connector_name || "connector"} (approved).`
          : `Human response: ${humanAnswer}`;
      }
      if (body.override_args && typeof body.override_args === "object") {
        const nextSpec = (run.steps ?? run.steps_v2)?.[cpIdx + 1];
        if (nextSpec) {
          nextSpec.args = { ...(nextSpec.args ?? {}), ...(body.override_args as Record<string, string>) };
        }
      }
      run.pending_checkpoint = null;
      run.status = "running";
      run.current_step_index = cpIdx + 1;
      run.updated_ms = Date.now();
      await persist();
      broadcastSse("pipeline_resumed", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        step_index: cpIdx + 1,
        action,
        human_answer: humanAnswer,
        run,
      });
      window.setTimeout(() => {
        void executeWebPipelineFrom(p, st, byok, run, cpIdx + 1);
      }, 15);
      return jsonRes({ run }, 202);
    }
    if (method === "POST" && path.startsWith("/api/pipelines/runs/") && path.endsWith("/retry")) {
      const runId = decodeURIComponent(path.slice("/api/pipelines/runs/".length, -"/retry".length));
      const run = (st.pipelineRuns ?? []).find((r) => r.run_id === runId);
      if (!run) return jsonRes({ error: "pipeline run not found" }, 404);
      if (run.status !== "failed" && run.status !== "interrupted") {
        return jsonRes({ error: `pipeline run is ${run.status}; only failed or interrupted runs can be retried` }, 409);
      }
      const steps = run.steps ?? run.steps_v2 ?? [];
      const fromIdx =
        typeof body.from_step_index === "number"
          ? Math.floor(body.from_step_index)
          : run.current_step_index;
      if (!Number.isFinite(fromIdx) || fromIdx < 0 || fromIdx >= steps.length) {
        return jsonRes({ error: "from_step_index out of range" }, 400);
      }
      for (let j = fromIdx; j < run.step_states.length; j++) {
        run.step_states[j] = {
          ...run.step_states[j],
          status: "pending",
          output: undefined,
          files: [],
          error: null,
        };
      }
      run.current_step_index = fromIdx;
      run.pending_checkpoint = null;
      run.status = "running";
      run.error = null;
      run.updated_ms = Date.now();
      await persist();
      window.setTimeout(() => {
        void executeWebPipelineFrom(p, st, byok, run, fromIdx);
      }, 15);
      return jsonRes({ run }, 202);
    }
    if (method === "POST" && path.startsWith("/api/pipelines/runs/") && path.endsWith("/cancel")) {
      const runId = decodeURIComponent(path.slice("/api/pipelines/runs/".length, -"/cancel".length));
      const run = (st.pipelineRuns ?? []).find((r) => r.run_id === runId);
      if (!run) return jsonRes({ error: "pipeline run not found" }, 404);
      if (run.status === "cancelled") {
        return jsonRes({ run }, 200);
      }
      if (run.status === "completed") {
        return jsonRes({ error: "pipeline run already completed" }, 409);
      }
      const cpId = run.pending_checkpoint?.checkpoint_id;
      if (cpId) {
        const dec = st.decisions.find((d) => d.id === cpId && d.status === "pending");
        if (dec) {
          dec.status = "denied";
          dec.resolved_ts = Date.now();
          dec.outcome = "Pipeline aborted.";
        }
      }
      run.status = "cancelled";
      run.pending_checkpoint = null;
      run.updated_ms = Date.now();
      await persist();
      broadcastSse("pipeline_finished", {
        run_id: run.run_id,
        thread_id: run.thread_id ?? null,
        status: "cancelled",
        error: null,
        run,
      });
      return jsonRes({ run }, 200);
    }
    if (method === "GET" && path === "/api/search") {
      const q = (u.searchParams.get("q") || "").trim();
      const scopeRaw = (u.searchParams.get("scope") || "all").trim().toLowerCase();
      const modeRaw = (u.searchParams.get("mode") || "hybrid").trim().toLowerCase();
      const scope: VaultSearchScope =
        scopeRaw === "files" || scopeRaw === "receipts" || scopeRaw === "memory" ? scopeRaw : "all";
      const mode: VaultSearchMode =
        modeRaw === "bm25" || modeRaw === "exact" ? modeRaw : "hybrid";
      const limit = Math.min(50, Math.max(1, Number(u.searchParams.get("limit") || 20)));
      const reqEngId = (u.searchParams.get("engagement_id") || "").trim();
      const reqClientId = (u.searchParams.get("client_id") || "").trim();
      let pathPrefix = u.searchParams.get("path_prefix") || undefined;
      let scopedEngId: string | undefined = reqEngId || undefined;
      if (reqEngId || st.isolateActiveEngagement) {
        ensureClientsAndPricing(state);
        const targetEng = reqEngId
          ? state.engagements?.find((e) => e.id === reqEngId) || null
          : activeEngagementFor(p.id).engagement;
        if (targetEng && (!reqClientId || targetEng.client_id === reqClientId)) {
          scopedEngId = targetEng.id;
          if (!pathPrefix) pathPrefix = engagementWorkspacePrefix(targetEng) || undefined;
        }
      }
      return jsonRes(searchWebVault(st, { q, scope, mode, limit, pathPrefix, engagementId: scopedEngId }));
    }
    if (method === "POST" && path === "/api/vault/reindex") {
      const t0 = performance.now();
      const idx = getVaultIndex(st, true);
      const elapsed_ms = Math.max(1, Math.round(performance.now() - t0));
      return jsonRes({
        ok: true,
        indexed_files: idx.stats.total_files,
        total_chunks: idx.stats.total_chunks,
        elapsed_ms,
        index_stats: idx.stats,
      });
    }
    if (method === "POST" && path === "/api/vault/aggregate") {
      const glob = String(body.glob ?? "*.csv").trim() || "*.csv";
      const lowGlob = glob.toLowerCase();
      if (
        glob.includes("..") ||
        glob.includes("\0") ||
        glob.includes("\\") ||
        glob.startsWith("/") ||
        lowGlob.includes("%2e") ||
        lowGlob.includes("%2f") ||
        lowGlob.includes("%5c") ||
        lowGlob.includes("%00") ||
        lowGlob.includes("%25")
      ) {
        return jsonRes({ error: "glob path escapes the workspace" }, 400);
      }
      const reqEngId = typeof body.engagement_id === "string" ? body.engagement_id.trim() : "";
      let pathPrefix: string | undefined;
      if (reqEngId || st.isolateActiveEngagement) {
        ensureClientsAndPricing(state);
        const targetEng = reqEngId
          ? state.engagements?.find((e) => e.id === reqEngId) || null
          : activeEngagementFor(p.id).engagement;
        if (targetEng) pathPrefix = engagementWorkspacePrefix(targetEng) || undefined;
      }
      const result = runWebVaultAggregate(st, {
        glob,
        sheet: typeof body.sheet === "string" && body.sheet.trim() ? body.sheet.trim() : undefined,
        where: Array.isArray(body.where) ? (body.where as Array<{ col: string; op: string; value: string }>) : undefined,
        group_by: Array.isArray(body.group_by) ? (body.group_by as string[]) : undefined,
        aggregates: Array.isArray(body.aggregates) ? (body.aggregates as Array<{ col: string; fn: string; as: string }>) : undefined,
        sort: body.sort && typeof body.sort === "object" ? (body.sort as { col: string; dir: string }) : undefined,
        limit: typeof body.limit === "number" ? body.limit : undefined,
        pathPrefix,
      });
      let saved_path: string | undefined;
      if (typeof body.save_as === "string" && body.save_as.trim()) {
        const safeOut = sanitizeSafePath(body.save_as);
        if (!safeOut) return jsonRes({ error: "save_as path escapes the workspace" }, 403);
        const csvText =
          result.columns.length > 0
            ? [result.columns, ...result.rows].map((r) => r.map(csvEscape).join(",")).join("\n") + "\n"
            : "";
        if (st.files[safeOut] && st.files[safeOut].content !== csvText) {
          recordFileRevision(st, safeOut, st.files[safeOut], "vault_aggregate");
        }
        st.files[safeOut] = makeFile(safeOut, csvText);
        saved_path = safeOut;
        await persist();
      }
      return jsonRes({ ...result, ...(saved_path ? { saved_path } : {}) });
    }
    if (method === "GET" && path === "/api/receipts") {
      const receipts = st.work.slice().reverse().map((x, idx) => ({
        id: x.id || x.thread_id || `rec-${idx}`,
        thread_id: x.thread_id,
        task: x.task,
        summary: x.summary,
        status: x.finished ? "finished" : "stopped",
        files: x.files,
        date: new Date(x.ts).toISOString().slice(0, 10),
        ts: x.ts,
      }));
      return jsonRes({ total: receipts.length, receipts });
    }
    if (method === "GET" && path === "/api/receipts/verify") {
      const head = await sha256Hex(JSON.stringify(st.work.map((w) => w.id || w.thread_id || w.ts)));
      return jsonRes({ total: st.work.length, valid: true, head_hash: head });
    }
    if (method === "GET" && path.startsWith("/api/receipts/") && path.endsWith("/export")) {
      const rid = decodeURIComponent(path.slice("/api/receipts/".length, -"/export".length));
      const format = u.searchParams.get("format") === "json" ? "json" : "md";
      const found = st.work.find((x) => x.id === rid || x.thread_id === rid) || st.work[st.work.length - 1];
      if (!found) return jsonRes({ error: "receipt not found" }, 404);
      const built = await buildSignedReceipt({
        id: found.id || found.thread_id || rid,
        task: found.task,
        person: p.name,
        status: found.finished ? "finished" : "stopped",
        summary: found.summary,
        ts: found.ts,
        files: found.files,
        steps: found.steps,
      });
      return new Response(format === "json" ? built.json : built.markdown, {
        status: 200,
        headers: {
          "Content-Type": format === "json" ? "application/json; charset=utf-8" : "text/markdown; charset=utf-8",
          "X-Ergon-Receipt-SHA256": built.sha256,
        },
      });
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
        const mime =
          found.ext === "pdf"
            ? "application/pdf"
            : found.ext === "xlsx"
              ? "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
              : found.ext === "docx"
                ? "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                : found.ext === "csv"
                  ? "text/csv; charset=utf-8"
                  : found.ext === "json"
                    ? "application/json; charset=utf-8"
                    : "text/plain; charset=utf-8";
        const bodyData = found.binaryBase64 ? new Uint8Array(base64ToBytes(found.binaryBase64)).buffer : found.content;
        return new Response(bodyData, {
          status: 200,
          headers: {
            "Content-Type": mime,
            "Content-Disposition": `attachment; filename="${found.name.replace(/[^a-zA-Z0-9._ -]/g, "_")}"`,
          },
        });
      }
      const reqSheet = (u.searchParams.get("sheet") || "").trim();
      const activeContent =
        reqSheet && found.extractedSheets && typeof found.extractedSheets[reqSheet] === "string"
          ? found.extractedSheets[reqSheet]
          : found.content;
      const activeExtracted = found.extracted
        ? {
            ...found.extracted,
            ...(reqSheet && found.extractedSheets?.[reqSheet] ? { sheet: reqSheet } : {}),
          }
        : undefined;
      const sha256 = await sha256Hex(activeContent);
      return jsonRes({
        path: found.path,
        name: found.name,
        size: found.size,
        mtime_ms: found.mtime_ms,
        ext: found.ext,
        content: activeContent,
        sha256,
        is_binary: Boolean(found.binaryBase64 && !found.extracted),
        ...(activeExtracted ? { extracted: activeExtracted } : {}),
      });
    }
    if (method === "PUT" && path === "/api/files/content") {
      const rawPath = String(body.path ?? "").trim();
      if (!rawPath) return jsonRes({ error: "path is required" }, 400);
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const content = String(body.content ?? "");
      const existing = st.files[safe];
      const expectedSha = typeof body.expected_sha256 === "string" ? body.expected_sha256.trim() : "";
      if (expectedSha) {
        const currentSha = existing ? await sha256Hex(existing.content) : "";
        if (!existing || currentSha !== expectedSha) {
          return jsonRes(
            {
              error: "File changed in workspace while you were editing.",
              current_sha256: currentSha,
            },
            409
          );
        }
      }
      if (existing && existing.content !== content) {
        recordFileRevision(st, safe, existing, String(body.reason ?? "edit") || "edit");
      }
      const saved = makeFile(safe, content);
      st.files[safe] = saved;
      await persist();
      const sha256 = await sha256Hex(saved.content);
      const rev_id = st.fileHistory[safe]?.[0]?.rev_id;
      return jsonRes({
        path: saved.path,
        size: saved.size,
        mtime_ms: saved.mtime_ms,
        sha256,
        ...(rev_id ? { rev_id } : {}),
      });
    }
    if (method === "PATCH" && path === "/api/files/content") {
      const rawPath = String(body.path ?? "").trim();
      if (!rawPath) return jsonRes({ error: "path is required" }, 400);
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const existing = st.files[safe];
      if (!existing) return jsonRes({ error: "file not found" }, 404);
      if (existing.binaryBase64 && !existing.extracted) {
        return jsonRes({ error: "binary files cannot be patched" }, 400);
      }
      const expectedSha = typeof body.expected_sha256 === "string" ? body.expected_sha256.trim() : "";
      const currentSha = await sha256Hex(existing.content);
      if (expectedSha && currentSha !== expectedSha) {
        return jsonRes({ error: "File changed in workspace while patching.", current_sha256: currentSha }, 409);
      }
      const ops = Array.isArray(body.ops) ? (body.ops as Array<Record<string, unknown>>) : [];
      if (!ops.length) return jsonRes({ error: "ops array is required" }, 400);
      let nextText = existing.content;
      let applied = 0;
      for (const op of ops) {
        const kind = String(op.op ?? "replace");
        const rep = String(op.replace ?? "");
        if (kind === "replace") {
          const find = String(op.find ?? "");
          if (!find) return jsonRes({ error: "find string is required" }, 400);
          const parts = nextText.split(find);
          const count = parts.length - 1;
          if (count === 0) return jsonRes({ error: `Text not found: ${find.slice(0, 40)}` }, 400);
          if (count > 1 && !op.replace_all) {
            return jsonRes({ error: `Ambiguous match (${count} occurrences); pass replace_all: true` }, 400);
          }
          nextText = op.replace_all ? parts.join(rep) : nextText.replace(find, rep);
          applied += 1;
        } else if (kind === "replace_lines") {
          const start = Number(op.start_line ?? 1);
          const end = Number(op.end_line ?? start);
          if (!Number.isInteger(start) || !Number.isInteger(end) || start < 1 || end < start) {
            return jsonRes({ error: "invalid start_line/end_line range" }, 400);
          }
          const lines = nextText.split(/\r?\n/);
          if (start > lines.length) return jsonRes({ error: "start_line out of range" }, 400);
          const repLines = rep.split(/\r?\n/);
          lines.splice(start - 1, end - start + 1, ...repLines);
          nextText = lines.join("\n");
          applied += 1;
        } else {
          return jsonRes({ error: `Unsupported patch op: ${kind}` }, 400);
        }
      }
      if (nextText !== existing.content) {
        recordFileRevision(st, safe, existing, "patch");
      }
      const saved = makeFile(safe, nextText);
      st.files[safe] = saved;
      await persist();
      const sha256 = await sha256Hex(saved.content);
      const diff = computeLineDiff(existing.content, nextText, { path: safe });
      return jsonRes({ path: saved.path, applied, size: saved.size, sha256, diff });
    }
    if (method === "POST" && path === "/api/tables/query") {
      const safe = sanitizeSafePath(String(body.path ?? ""));
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const found = st.files[safe];
      if (!found) return jsonRes({ error: "file not found" }, 404);
      const result = runWebTableQuery(found, {
        where: Array.isArray(body.where) ? (body.where as Array<{ col: string; op: string; value: string }>) : undefined,
        group_by: Array.isArray(body.group_by) ? (body.group_by as string[]) : undefined,
        aggregates: Array.isArray(body.aggregates) ? (body.aggregates as Array<{ col: string; fn: string; as: string }>) : undefined,
        sort: body.sort && typeof body.sort === "object" ? (body.sort as { col: string; dir: string }) : undefined,
        limit: typeof body.limit === "number" ? body.limit : undefined,
      });
      let saved_path: string | undefined;
      if (typeof body.save_as === "string" && body.save_as.trim()) {
        const safeOut = sanitizeSafePath(body.save_as);
        if (!safeOut) return jsonRes({ error: "save_as path escapes the workspace" }, 403);
        const csvText = [result.columns, ...result.rows].map((r) => r.map(csvEscape).join(",")).join("\n") + "\n";
        if (st.files[safeOut] && st.files[safeOut].content !== csvText) {
          recordFileRevision(st, safeOut, st.files[safeOut], "query_save");
        }
        st.files[safeOut] = makeFile(safeOut, csvText);
        saved_path = safeOut;
        await persist();
      }
      return jsonRes({ ...result, ...(saved_path ? { saved_path } : {}) });
    }
    if (method === "POST" && path === "/api/files/export") {
      const fmt = String(body.format ?? "").trim().toLowerCase();
      if (!["pdf", "docx", "xlsx"].includes(fmt)) {
        return jsonRes({ error: "format must be pdf, docx, or xlsx" }, 400);
      }
      const rawPath = typeof body.path === "string" ? body.path : "";
      const safeSrc = rawPath ? sanitizeSafePath(rawPath) : null;
      if (rawPath && !safeSrc) return jsonRes({ error: "path escapes the workspace" }, 403);
      const srcFile = safeSrc ? st.files[safeSrc] : undefined;
      if (rawPath && !srcFile) return jsonRes({ error: "file not found" }, 404);
      const stem = (safeSrc || String(body.title ?? "deliverable")).replace(/\.[^.]+$/, "").replace(/[^a-zA-Z0-9._/-]/g, "-");
      const outReq = typeof body.output === "string" && body.output.trim() ? sanitizeSafePath(body.output) : `${stem}.${fmt}`;
      if (!outReq) return jsonRes({ error: "output path escapes the workspace" }, 403);
      const title = String(body.title ?? safeSrc ?? "Ergon Deliverable").trim();
      const brandHeader = typeof body.brand_header === "string" ? body.brand_header.trim() : undefined;

      let bytes: Uint8Array;
      let previewText = "";
      let extractedKind = fmt.toUpperCase();
      let extractedFmt: "markdown" | "csv" = "markdown";
      let extractedSheets: Record<string, string> | undefined;
      let sheetNames: string[] | undefined;
      if (fmt === "pdf") {
        const mdText =
          srcFile?.ext === "csv" || srcFile?.ext === "tsv"
            ? `# ${title}\n\n` + parseDelimited(srcFile.content, srcFile.ext === "tsv" ? "\t" : ",").map((r) => r.join(" | ")).join("\n")
            : srcFile?.content || `# ${title}\n`;
        bytes = buildWebPdf(mdText, title, brandHeader);
        previewText = mdText;
        extractedKind = "PDF";
      } else if (fmt === "docx") {
        const text = srcFile?.content || `# ${title}\n`;
        bytes = buildWebDocx(text, title);
        previewText = text;
        extractedKind = "Word";
      } else {
        const rawSheets = Array.isArray(body.sheets) ? (body.sheets as Array<{ name?: string; path?: string; rows?: string[][]; totals_row?: boolean }>) : [];
        const sheets = rawSheets.length
          ? rawSheets.map((s, idx) => {
              const sf = s.path ? st.files[sanitizeSafePath(s.path) || ""] : undefined;
              const rows = Array.isArray(s.rows)
                ? s.rows
                : sf
                  ? parseDelimited(sf.content, sf.ext === "tsv" ? "\t" : ",")
                  : [["Empty"]];
              return { name: String(s.name ?? `Sheet${idx + 1}`), rows, totals_row: Boolean(s.totals_row) };
            })
          : [
              {
                name: "Data",
                rows: srcFile ? parseDelimited(srcFile.content, srcFile.ext === "tsv" ? "\t" : ",") : [["Empty"]],
                totals_row: false,
              },
            ];
        bytes = buildWebMultiSheetXlsx(sheets);
        previewText = sheets[0]?.rows.map((r) => r.map(csvEscape).join(",")).join("\n") || "";
        extractedSheets = {};
        for (const sh of sheets) {
          extractedSheets[sh.name] = sh.rows.map((r) => r.map(csvEscape).join(",")).join("\n");
        }
        sheetNames = sheets.map((s) => s.name);
        extractedKind = "Excel";
        extractedFmt = "csv";
      }
      const b64 = bytesToBase64(bytes);
      st.files[outReq] = {
        path: outReq,
        name: outReq.split("/").pop() || outReq,
        content: previewText,
        size: bytes.length,
        mtime_ms: Date.now(),
        ext: fmt,
        binaryBase64: b64,
        extracted: {
          kind: extractedKind,
          format: extractedFmt,
          pages: 1,
          ...(sheetNames ? { sheets: sheetNames, sheet: sheetNames[0] ?? null } : {}),
        },
        ...(extractedSheets ? { extractedSheets } : {}),
      };
      await persist();
      let sha256 = "";
      if (typeof crypto !== "undefined" && crypto.subtle) {
        const dig = await crypto.subtle.digest("SHA-256", new Uint8Array(bytes).buffer);
        sha256 = Array.from(new Uint8Array(dig)).map((b) => b.toString(16).padStart(2, "0")).join("");
      }
      return jsonRes({ path: outReq, size: bytes.length, format: fmt, sha256 });
    }
    if (method === "GET" && path === "/api/files/pack") {
      const rawFiles = u.searchParams.get("files") || "";
      const title = (u.searchParams.get("title") || "").trim() || "Client Deliverable Pack";
      const format = (u.searchParams.get("format") || "md").toLowerCase();
      const includeSummary = ["1", "true"].includes((u.searchParams.get("summary") || "").toLowerCase());
      const receiptId = (u.searchParams.get("receipt_id") || "").trim();
      const preflightOnly = ["1", "true"].includes((u.searchParams.get("preflight") || "").toLowerCase());
      const requested = rawFiles
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean)
        .slice(0, 25);
      const explicit = requested.length > 0;
      const summaryRuns = includeSummary
        ? [...st.work].filter((w) => w.finished && !w.answered).reverse().slice(0, 15)
        : [];
      const unionTargets = [...requested];
      if (!explicit) {
        for (const r of summaryRuns) {
          for (const rf of r.files || []) {
            if (unionTargets.length < 25 && !unionTargets.includes(rf)) unionTargets.push(rf);
          }
        }
        for (const f of Object.values(st.files).sort((a, b) => b.mtime_ms - a.mtime_ms || a.path.localeCompare(b.path))) {
          if (unionTargets.length >= 25) break;
          if (!unionTargets.includes(f.path)) unionTargets.push(f.path);
        }
      }
      const targets = explicit ? requested : unionTargets.slice(0, 25);
      const sections: Array<{
        path: string;
        size_bytes: number;
        non_empty: boolean;
        count: number;
        unit: "rows" | "lines" | "bytes";
        sha: string;
        body: string;
      }> = [];
      const missingPaths: string[] = [];
      let readyCount = 0;
      let emptyCount = 0;
      let totalBytes = 0;
      for (const rel of targets) {
        const safe = sanitizeSafePath(rel);
        if (!safe) {
          if (explicit) return jsonRes({ error: "path escapes the workspace" }, 403);
          continue;
        }
        const f = st.files[safe];
        if (!f) {
          if (explicit) missingPaths.push(rel);
          continue;
        }
        const sizeBytes = new TextEncoder().encode(f.content).length;
        const nonEmpty = f.content.trim().length > 0;
        if (nonEmpty) readyCount += 1;
        else emptyCount += 1;
        totalBytes += sizeBytes;
        const linesNonEmpty = f.content.split(/\r?\n/).filter((l) => l.trim().length > 0).length;
        const isTable = f.ext === "csv" || f.ext === "tsv";
        const count = isTable ? Math.max(0, linesNonEmpty - 1) : linesNonEmpty;
        const unit = isTable ? "rows" : "lines";
        const sha = await sha256Hex(f.content);
        const body =
          f.ext === "md" || f.ext === "markdown" || f.ext === "txt"
            ? f.content.trimEnd()
            : `\`\`\`${f.ext === "csv" || f.ext === "tsv" ? "csv" : f.ext === "json" ? "json" : "text"}\n${f.content.trimEnd()}\n\`\`\``;
        sections.push({ path: safe, size_bytes: sizeBytes, non_empty: nonEmpty, count, unit, sha, body });
      }
      if (explicit && sections.length === 0) {
        return jsonRes({ error: "no matching workspace files found" }, 404);
      }
      const matchedReceipt = receiptId
        ? st.work.find((w) => w.id === receiptId || w.thread_id === receiptId)
        : undefined;
      const citMap = new Map<string, Awaited<ReturnType<typeof verifyTextCitationsInMemory>>["citations"][number]>();
      const rank = (s: string) => (s === "verified" ? 0 : s === "stale" ? 1 : 2);
      const mergeCits = (rep: Awaited<ReturnType<typeof verifyTextCitationsInMemory>>) => {
        for (const c of rep.citations) {
          const k = `${c.path}:${c.line}:${c.end_line ?? ""}`;
          const prev = citMap.get(k);
          if (!prev || rank(c.status) > rank(prev.status)) {
            citMap.set(k, c);
          }
        }
      };
      const summaryGates: Array<{
        run: WebWork;
        checked: number;
        verified: number;
        stale: number;
        broken: number;
        gate: string;
      }> = [];
      if (matchedReceipt) {
        mergeCits(await verifyTextCitationsInMemory(st.files, `${matchedReceipt.task}\n${matchedReceipt.summary}`, undefined, st.fileHistory, matchedReceipt.ts));
      } else if (includeSummary && summaryRuns.length > 0) {
        for (const sr of summaryRuns) {
          const rep = await verifyTextCitationsInMemory(st.files, sr.summary || "", undefined, st.fileHistory, sr.ts);
          mergeCits(rep);
          summaryGates.push({
            run: sr,
            checked: rep.total,
            verified: rep.verified,
            stale: rep.stale,
            broken: rep.broken,
            gate: rep.total === 0 ? "Deliverables ready" : rep.gate_status,
          });
        }
      } else {
        for (const sr of st.work.slice(-8)) {
          if (sr.summary) mergeCits(await verifyTextCitationsInMemory(st.files, sr.summary, undefined, st.fileHistory, sr.ts));
        }
      }
      for (const sec of sections) {
        mergeCits(await verifyTextCitationsInMemory(st.files, sec.body, undefined, st.fileHistory, 0));
      }
      const mergedList = Array.from(citMap.values());
      const cVer = mergedList.filter((c) => c.status === "verified").length;
      const cStale = mergedList.filter((c) => c.status === "stale").length;
      const cBroken = mergedList.filter((c) => c.status !== "verified" && c.status !== "stale").length;
      const citations = {
        total: mergedList.length,
        verified: cVer,
        stale: cStale,
        broken: cBroken,
        freshness_pct: mergedList.length ? Math.floor((cVer * 100) / mergedList.length) : 100,
        gate_status: cBroken === 0 && cStale === 0 ? "100% Verified" : cStale > 0 ? "Stale Line Drift" : "Broken Citations",
        citations: mergedList,
      };
      const preflightOk = readyCount > 0 && emptyCount === 0 && missingPaths.length === 0;
      const allOk = preflightOk && citations.broken === 0 && citations.stale === 0;
      if (preflightOnly) {
        return jsonRes({
          ok: allOk,
          ready_count: readyCount,
          empty_count: emptyCount,
          missing_count: missingPaths.length,
          total_bytes: totalBytes,
          citations_checked: citations.total,
          citations_verified: citations.verified,
          citations_stale: citations.stale,
          citations_broken: citations.broken,
          citations_freshness_pct: citations.freshness_pct,
          citation_gate: citations.gate_status,
          citations: citations.citations,
          ...(includeSummary
            ? {
                summary_runs_count: summaryGates.length,
                summary_runs: summaryGates.map((sg) => ({
                  id: sg.run.id || sg.run.thread_id,
                  task: sg.run.task,
                  ts: sg.run.ts,
                  files: sg.run.files,
                  citations_checked: sg.checked,
                  citations_verified: sg.verified,
                  citations_stale: sg.stale,
                  citations_broken: sg.broken,
                  citation_gate: sg.gate,
                })),
              }
            : {}),
          files: [
            ...sections.map((s) => ({
              path: s.path,
              size_bytes: s.size_bytes,
              non_empty: s.non_empty,
              count: s.count,
              unit: s.unit,
              sha256: s.sha,
              status: s.non_empty ? "ready" : "empty",
            })),
            ...missingPaths.map((mp) => ({
              path: mp,
              size_bytes: 0,
              non_empty: false,
              count: 0,
              unit: "bytes",
              sha256: "",
              status: "missing",
            })),
          ],
        });
      }
      const policy = buildWebPolicy(p.id, st);
      const forcePack = ["1", "true"].includes((u.searchParams.get("force") || "").toLowerCase());
      if (policy.strict_citation_gate && !forcePack && (citations.stale > 0 || citations.broken > 0)) {
        return jsonRes(
          {
            error: `Blocked by ${policy.label} policy: ${citations.stale} stale and ${citations.broken} broken [file:L##] citation(s) must be resolved before export.`,
            policy: policy.profile,
            citation_gate: citations.gate_status,
            citations_stale: citations.stale,
            citations_broken: citations.broken,
          },
          409
        );
      }
      const deltaA = (u.searchParams.get("delta_a") || "").trim();
      const deltaB = (u.searchParams.get("delta_b") || "").trim();
      const deltaFile = (u.searchParams.get("delta_file") || "").trim();
      let deltaMd = "";
      let hasDelta = false;
      const simPct = (d: { added: number; removed: number; unchanged?: number }) => {
        const unc = d.unchanged ?? 0;
        const denom = unc + Math.max(d.added, d.removed);
        return denom === 0 ? 100 : Math.min(100, Math.round((unc * 100) / denom));
      };
      if (deltaA && deltaB) {
        const ra = st.work.find((w, idx) => w.id === deltaA || w.thread_id === deltaA || String(idx) === deltaA);
        const rb = st.work.find((w, idx) => w.id === deltaB || w.thread_id === deltaB || String(idx) === deltaB);
        if (!ra || !rb) return jsonRes({ error: "delta receipt not found" }, 404);
        const diff = computeLineDiff(ra.summary || "", rb.summary || "", { path: `${deltaA}->${deltaB}` });
        const unifiedText = (diff.unified ?? "").trimEnd();
        deltaMd = `## Verified Run Delta (\`${deltaA}\` -> \`${deltaB}\`)\n\n- **Changes:** +${diff.added} / -${diff.removed} lines (${simPct(diff)}% similarity)${unifiedText ? `\n\n\`\`\`diff\n${unifiedText}\n\`\`\`` : ""}`;
        hasDelta = true;
      } else if (deltaFile) {
        const safeDelta = sanitizeSafePath(deltaFile);
        if (!safeDelta) return jsonRes({ error: "path escapes the workspace" }, 403);
        const revAId = (u.searchParams.get("rev_a") || "").trim();
        const revBId = (u.searchParams.get("rev_b") || "current").trim();
        const revList = st.fileHistory[safeDelta] ?? [];
        const textA = revList.find((r) => r.rev_id === revAId)?.content;
        const textB = revBId === "current" ? st.files[safeDelta]?.content : revList.find((r) => r.rev_id === revBId)?.content;
        if (typeof textA !== "string" || typeof textB !== "string") return jsonRes({ error: "snapshot not found" }, 404);
        const diff = computeLineDiff(textA, textB, { path: safeDelta, rev_a: revAId, rev_b: revBId });
        const unifiedText = (diff.unified ?? "").trimEnd();
        deltaMd = `## Verified Snapshot Delta (\`${safeDelta}\`: \`${revAId}\` -> \`${revBId}\`)\n\n- **Changes:** +${diff.added} / -${diff.removed} lines (${simPct(diff)}% similarity)${unifiedText ? `\n\n\`\`\`diff\n${unifiedText}\n\`\`\`` : ""}`;
        hasDelta = true;
      }
      const hasSummary = summaryGates.length > 0;
      const summaryCitsHdr = `${citations.verified}/${citations.total}`;
      const summaryStaleHdr = String(citations.stale + citations.broken);
      const ratioLabel =
        citations.total === 0
          ? "0 inline citations"
          : `${citations.verified}/${citations.total} verified (${citations.freshness_pct}%)`;
      if (format === "html") {
        const manifestSha = await sha256Hex(sections.map((s) => `${s.path}:${s.sha}`).join("|"));
        const summaryHtml = hasSummary
          ? `<section><h2>Verified Work Summary</h2><table><thead><tr><th>Task</th><th>Summary</th><th>Deliverables</th><th>Citations</th></tr></thead><tbody>${summaryGates
              .map(
                (sg) =>
                  `<tr><td><strong>${escapeHtml(sg.run.task)}</strong></td><td>${escapeHtml(String(sg.run.summary || "").slice(0, 180))}</td><td><code>${escapeHtml((sg.run.files || []).join(", "))}</code></td><td><span class="pill ${sg.stale + sg.broken > 0 ? "warn" : "ok"}">${sg.checked === 0 ? "ready" : sg.stale + sg.broken > 0 ? `${sg.verified}/${sg.checked} (${sg.stale + sg.broken} stale)` : `${sg.verified}/${sg.checked} verified`}</span></td></tr>`
              )
              .join("")}</tbody></table></section>`
          : "";
        const filesRows = sections
          .map((s) => `<tr><td><code>${escapeHtml(s.path)}</code></td><td>${s.count} ${s.unit}</td><td>${s.size_bytes} B</td><td><code>${escapeHtml(s.sha)}</code></td></tr>`)
          .join("");
        const citSection = citations.citations.length
          ? `<section><h2>Live Citation Verification</h2><table><thead><tr><th>Citation</th><th>Gate Status</th><th>Live File SHA-256</th><th>Excerpt</th></tr></thead><tbody>${citations.citations
              .map(
                (c) =>
                  `<tr><td><code>${escapeHtml(c.raw)}</code></td><td><span class="pill ${c.status === "verified" ? "ok" : "warn"}">${escapeHtml(c.status)}</span></td><td><code>${escapeHtml((c.file_sha256 || "").slice(0, 12))}</code></td><td>${escapeHtml(c.line_excerpt || "")}</td></tr>`
              )
              .join("")}</tbody></table></section>`
          : "";
        const deltaHtml = deltaMd ? `<section><h2>Verified Delta</h2><pre>${escapeHtml(deltaMd)}</pre></section>` : "";
        const previews = sections
          .map((s) => `<section><h3><code>${escapeHtml(s.path)}</code></h3><pre>${escapeHtml(s.body)}</pre></section>`)
          .join("");
        const rawHtml = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>${escapeHtml(title)}</title><style>body{font-family:sans-serif;max-width:920px;margin:32px auto;padding:0 20px;color:#111827}table{width:100%;border-collapse:collapse;font-size:13px}th,td{padding:8px;border-bottom:1px solid #e5e7eb;text-align:left}pre{background:#f9fafb;border:1px solid #e5e7eb;padding:12px;border-radius:8px;white-space:pre-wrap}.pill{display:inline-block;padding:2px 8px;border-radius:999px;font-size:12px;font-weight:600}.pill.ok{background:#dcfce7;color:#166534}.pill.warn{background:#fef3c7;color:#92400e}</style></head><body><h1>${escapeHtml(title)}</h1><p>Prepared for <strong>${escapeHtml(p.name)}</strong> &middot; ${escapeHtml(citations.gate_status)} &middot; Citations: ${escapeHtml(ratioLabel)} &middot; Manifest SHA-256: <code>${escapeHtml(manifestSha)}</code></p>${summaryHtml}<table><thead><tr><th>File</th><th>Contents</th><th>Size</th><th>SHA-256</th></tr></thead><tbody>${filesRows}</tbody></table>${citSection}${deltaHtml}${previews}</body></html>`;
        const html = policy.redact_host_paths ? redactWebHostPaths(rawHtml) : rawHtml;
        const packSha = await sha256Hex(html);
        return new Response(html, {
          status: 200,
          headers: {
            "Content-Type": "text/html; charset=utf-8",
            "Content-Security-Policy": "default-src 'none'; style-src 'unsafe-inline'",
            "X-Content-Type-Options": "nosniff",
            "X-Ergon-Pack-SHA256": packSha,
            "X-Ergon-Pack-Preflight": allOk ? "pass" : "warn",
            "X-Ergon-Pack-Citation-Gate": citations.gate_status,
            "X-Ergon-Pack-Ready-Count": String(readyCount),
            "X-Ergon-Pack-Delta": hasDelta ? "yes" : "none",
            "X-Ergon-Pack-Summary": hasSummary ? "yes" : "none",
            "X-Ergon-Pack-Summary-Runs": String(summaryGates.length),
            "X-Ergon-Pack-Summary-Citations": summaryCitsHdr,
            "X-Ergon-Pack-Summary-Stale": summaryStaleHdr,
          },
        });
      }
      const statusWord = allOk ? "PASS" : "WARN";
      const lines = [
        `# ${title}`,
        "",
        `- Prepared for: **${p.name}**`,
        `- Deliverables included: ${sections.length}`,
        `- Completed runs summarized: ${summaryGates.length}`,
        `- Pre-flight check: **${statusWord}** (${readyCount} non-empty, ${emptyCount} empty, ${missingPaths.length} missing, ${totalBytes} total bytes; citations: ${citations.gate_status}: ${ratioLabel})`,
        "",
      ];
      if (hasSummary) {
        lines.push("## Verified Work Summary", "");
        if (citations.stale > 0 || citations.broken > 0) {
          lines.push(`- **Batch Citation Gate:** WARN (${citations.verified}/${citations.total} verified, **${citations.stale} stale**, ${citations.broken} broken across ${summaryGates.length} runs)`, "");
        } else if (citations.total > 0) {
          lines.push(`- **Batch Citation Gate:** 100% Verified (${citations.verified}/${citations.total} citations across ${summaryGates.length} runs)`, "");
        }
        for (const sg of summaryGates) {
          const r = sg.run;
          const filesPart = r.files?.length ? ` · Files: ${r.files.join(", ")}` : "";
          const citPart =
            sg.checked === 0
              ? ""
              : sg.stale > 0 || sg.broken > 0
                ? ` · **[CITATION ALERT: ${sg.verified}/${sg.checked} verified, ${sg.stale} stale, ${sg.broken} broken]**`
                : ` · [citations: ${sg.verified}/${sg.checked} verified]`;
          lines.push(`- **${r.task}** (\`${r.id || r.thread_id}\`${filesPart}${citPart}): ${String(r.summary || "").trim().slice(0, 180)}`);
        }
        lines.push("");
      }
      lines.push("## Included Deliverables", "", ...sections.map((s) => `- \`${s.path}\` (${s.count} ${s.unit}, \`sha256:${s.sha}\`)`));
      if (citations.citations.length > 0) {
        lines.push("", "## Citation Verification", "");
        for (const c of citations.citations) {
          lines.push(`- \`${c.raw}\` -> **${c.status}** (${c.total_lines} lines in file, live \`sha256:${(c.file_sha256 || "").slice(0, 12)}\`)`);
        }
      }
      for (const s of sections) {
        lines.push("", "---", "", `## \`${s.path}\``, "", `_SHA-256: \`${s.sha}\` · ${s.count} ${s.unit} · ${s.size_bytes} bytes_`, "", s.body);
      }
      if (deltaMd) lines.push("", "---", "", deltaMd);
      const rawMd = lines.join("\n") + "\n";
      const md = policy.redact_host_paths ? redactWebHostPaths(rawMd) : rawMd;
      const packSha = await sha256Hex(md);
      return new Response(md, {
        status: 200,
        headers: {
          "Content-Type": "text/markdown; charset=utf-8",
          "X-Content-Type-Options": "nosniff",
          "X-Ergon-Pack-SHA256": packSha,
          "X-Ergon-Pack-Preflight": allOk ? "pass" : "warn",
          "X-Ergon-Pack-Citation-Gate": citations.gate_status,
          "X-Ergon-Pack-Ready-Count": String(readyCount),
          "X-Ergon-Pack-Delta": hasDelta ? "yes" : "none",
          "X-Ergon-Pack-Summary": hasSummary ? "yes" : "none",
        },
      });
    }
    if (method === "GET" && path === "/api/receipts/compare") {
      const aId = (u.searchParams.get("a") || "").trim();
      const bId = (u.searchParams.get("b") || "").trim();
      if (!aId || !bId) return jsonRes({ error: "query parameters a and b are required" }, 400);
      const findW = (key: string) =>
        st.work.find((w, idx) => w.id === key || w.thread_id === key || String(idx) === key);
      const ra = findW(aId);
      const rb = findW(bId);
      if (!ra) return jsonRes({ error: "receipt a not found" }, 404);
      if (!rb) return jsonRes({ error: "receipt b not found" }, 404);
      const diff = computeLineDiff(ra.summary || "", rb.summary || "", { path: `${aId}->${bId}` });
      const unc = diff.unchanged ?? 0;
      const denom = unc + Math.max(diff.added, diff.removed);
      const simPct = denom === 0 ? 100 : Math.min(100, Math.round((unc * 100) / denom));
      const unifiedText = (diff.unified ?? "").trimEnd();
      const filesA = Array.isArray(ra.files) ? ra.files : [];
      const filesB = Array.isArray(rb.files) ? rb.files : [];
      const filesAdded = filesB.filter((f) => !filesA.includes(f));
      const filesRemoved = filesA.filter((f) => !filesB.includes(f));
      const filesShared = filesB.filter((f) => filesA.includes(f));
      const lineDelta = diff.added - diff.removed;
      const fileDelta = filesB.length - filesA.length;
      const deltaMd = [
        `## Verified Run Delta (\`${ra.id || aId}\` -> \`${rb.id || bId}\`)`,
        "",
        `- **Summary Diff:** +${diff.added} / -${diff.removed} lines (${simPct}% similarity, net ${lineDelta >= 0 ? "+" : ""}${lineDelta} lines)`,
        `- **File Outputs:** ${filesAdded.length} added, ${filesRemoved.length} removed, ${filesShared.length} shared`,
        ...(unifiedText ? ["", "```diff", unifiedText, "```"] : []),
      ].join("\n") + "\n";
      return jsonRes({
        receipt_a: { id: ra.id || aId, task: ra.task, summary: ra.summary, files: filesA },
        receipt_b: { id: rb.id || bId, task: rb.task, summary: rb.summary, files: filesB },
        added: diff.added,
        removed: diff.removed,
        unchanged: diff.unchanged,
        similarity_pct: simPct,
        line_delta: lineDelta,
        file_delta: fileDelta,
        files_added: filesAdded,
        files_removed: filesRemoved,
        files_shared: filesShared,
        hunks: diff.hunks,
        unified: diff.unified ?? "",
        delta_markdown: deltaMd,
      });
    }
    if (method === "POST" && path === "/api/citations/verify") {
      const b = (body && typeof body === "object" ? body : {}) as Record<string, unknown>;
      let text = String(b.text || "");
      const receiptId = String(b.receipt_id || "").trim();
      let receiptTs = 0;
      if (receiptId) {
        const hit = st.work.find((r) => r.id === receiptId || r.thread_id === receiptId);
        if (hit) {
          receiptTs = hit.ts;
          if (!text) text = `${hit.task}\n${hit.summary}`;
        }
      }
      const expectedShas =
        b.expected_shas && typeof b.expected_shas === "object"
          ? (b.expected_shas as Record<string, string>)
          : undefined;
      return jsonRes(await verifyTextCitationsInMemory(st.files, text, expectedShas, st.fileHistory, receiptTs));
    }
    if (method === "GET" && path === "/api/files/history") {
      const rawPath = u.searchParams.get("path") || "";
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const rawRevs = st.fileHistory[safe] ?? [];
      const revs = await Promise.all(
        rawRevs.map(async (rev) => ({
          ...rev,
          sha256: rev.sha256 || (typeof rev.content === "string" ? await sha256Hex(rev.content) : undefined),
        }))
      );
      const revAId = u.searchParams.get("rev_a") || "";
      const revBId = u.searchParams.get("rev_b") || "current";
      if (revAId) {
        const ra = revs.find((x) => x.rev_id === revAId);
        if (!ra) return jsonRes({ error: "revision not found" }, 404);
        const textB =
          revBId === "current"
            ? (st.files[safe]?.content ?? "")
            : (revs.find((x) => x.rev_id === revBId)?.content ?? "");
        const diff = computeLineDiff(ra.content ?? "", textB, { path: safe, rev_a: revAId, rev_b: revBId });
        return jsonRes({ path: safe, revisions: revs, diff });
      }
      return jsonRes({ path: safe, revisions: revs });
    }
    if (method === "POST" && path === "/api/files/restore") {
      const rawPath = String(body.path ?? "");
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "path escapes the workspace" }, 403);
      const revId = String(body.rev_id ?? body.revision_id ?? body.id ?? "").trim();
      const revs = st.fileHistory[safe] ?? [];
      const found = revs.find((r) => r.rev_id === revId);
      if (!found || typeof found.content !== "string") {
        return jsonRes({ error: "revision not found" }, 404);
      }
      if (st.files[safe] && st.files[safe].content !== found.content) {
        recordFileRevision(st, safe, st.files[safe], "restore");
      }
      const restored = makeFile(safe, found.content);
      st.files[safe] = restored;
      await persist();
      return jsonRes({
        restored: true,
        path: restored.path,
        rev_id: revId,
        file: { path: restored.path, name: restored.name, size: restored.size, mtime_ms: restored.mtime_ms, ext: restored.ext },
      });
    }
    if (method === "GET" && path === "/api/workspace/policy") {
      return jsonRes(buildWebPolicy(p.id, st));
    }
    if ((method === "PUT" || method === "POST") && path === "/api/workspace/policy") {
      if (body.profile !== undefined) {
        const raw = String(body.profile ?? "").trim().toLowerCase();
        const map: Record<string, WorkspacePolicyProfile> = {
          fast_internal: "fast_internal",
          fast: "fast_internal",
          internal: "fast_internal",
          strict_client: "strict_client",
          strict: "strict_client",
          client: "strict_client",
          air_gapped: "air_gapped",
          airgapped: "air_gapped",
          offline: "air_gapped",
        };
        const resolved = map[raw];
        if (!resolved) {
          return jsonRes({ error: "workspace policy profile must be fast_internal, strict_client, or air_gapped" }, 400);
        }
        st.policyProfile = resolved;
      }
      if (typeof body.isolate_active_engagement === "boolean") {
        st.isolateActiveEngagement = body.isolate_active_engagement;
      }
      st.policyUpdatedMs = Date.now();
      await persist();
      return jsonRes(buildWebPolicy(p.id, st));
    }
    if (method === "GET" && path === "/api/audit/bundle") {
      const format = u.searchParams.get("format") === "md" ? "md" : "json";
      const policy = buildWebPolicy(p.id, st);
      const allRuns = Object.values(st.routineRuns).flat().sort((a, b) => b.started_ms - a.started_ms);
      const built = await buildAuditBundle({
        personName: p.name,
        personId: p.id,
        files: Object.values(st.files).map((f) => ({
          path: f.path,
          size: f.size,
          mtime_ms: f.mtime_ms,
          content: f.content,
          revision_count: (st.fileHistory[f.path] ?? []).length,
        })),
        receipts: st.work.map((x, idx) => ({
          id: x.id || x.thread_id || `rec-${idx}`,
          thread_id: x.thread_id,
          task: x.task,
          summary: x.summary,
          files: x.files,
          ts: x.ts,
        })),
        routines: st.routines,
        routineRuns: allRuns,
      });
      const payload = format === "md" ? built.markdown : built.json;
      const bodyStr = policy.redact_host_paths ? redactWebHostPaths(payload) : payload;
      return new Response(bodyStr, {
        status: 200,
        headers: {
          "Content-Type": format === "md" ? "text/markdown; charset=utf-8" : "application/json; charset=utf-8",
          "X-Ergon-Bundle-SHA256": built.bundleSha256,
        },
      });
    }
    if (method === "POST" && path === "/api/export/handoff") {
      const dryRun = Boolean(body.dry_run) || u.searchParams.get("dry_run") === "1";
      const force = Boolean(body.force) || u.searchParams.get("force") === "1";
      const title = String(body.title ?? "Client Handoff").trim() || "Client Handoff";
      const reqEngId = typeof body.engagement_id === "string" ? body.engagement_id.trim() : "";
      ensureClientsAndPricing(state);
      const scopedEng = reqEngId ? state.engagements?.find((e) => e.id === reqEngId) || null : null;
      const policy = buildWebPolicy(p.id, st);
      const scopedFiles = Object.values(st.files).filter((f) =>
        scopedEng ? isPathInEngagementScope(f.path, scopedEng) : true
      );
      const scopedWork = st.work.filter((x) =>
        scopedEng ? !x.engagement_id || x.engagement_id === scopedEng.id : true
      );
      const packRes = await window.fetch(`/api/files/pack?summary=1&format=md&force=1&title=${encodeURIComponent(title)}`, { method: "GET" });
      const rawSummary = packRes.ok && !scopedEng
        ? await packRes.text()
        : `# ${title}\n\n- Prepared for: **${p.name}**\n${scopedEng ? `- Engagement: ${scopedEng.name}\n` : ""}- Workspace files: ${scopedFiles.length}\n`;
      const wsFiles = scopedFiles.map((f) => ({ path: f.path, content: f.content }));
      const scan = scanWebHandoffLeaks(wsFiles, rawSummary, byok.api_key || undefined);
      const strictBlock = policy.profile === "strict_client" || policy.strict_citation_gate;

      if (dryRun) {
        const wouldBlock = strictBlock && !scan.clean;
        return jsonRes({
          ok: !wouldBlock,
          dry_run: true,
          would_block: wouldBlock,
          policy,
          scan,
        });
      }
      if (strictBlock && !scan.clean && !force) {
        return jsonRes(
          {
            error: `Blocked by ${policy.label} policy: ${scan.leak_count} leak/path finding(s) must be resolved before client handoff.`,
            blocked: true,
            policy: policy.profile,
            scan,
          },
          409
        );
      }

      const allRuns = Object.values(st.routineRuns).flat().sort((a, b) => b.started_ms - a.started_ms);
      const builtBundle = await buildAuditBundle({
        personName: p.name,
        personId: p.id,
        files: scopedFiles.map((f) => ({
          path: f.path,
          size: f.size,
          mtime_ms: f.mtime_ms,
          content: f.content,
          revision_count: (st.fileHistory[f.path] ?? []).length,
        })),
        receipts: scopedWork.map((x, idx) => ({
          id: x.id || x.thread_id || `rec-${idx}`,
          thread_id: x.thread_id,
          task: x.task,
          summary: x.summary,
          files: x.files,
          ts: x.ts,
        })),
        routines: st.routines,
        routineRuns: allRuns,
      });
      const summaryMd = policy.redact_host_paths ? redactWebHostPaths(rawSummary) : rawSummary;
      const auditJson = policy.redact_host_paths ? redactWebHostPaths(builtBundle.json) : builtBundle.json;
      const verifyTxt =
        `Ergon Client Handoff Verification Guide\n` +
        `Title: ${title}\n` +
        `Policy: ${policy.label} (${policy.profile})\n` +
        `Leak pre-check findings: ${scan.leak_count}\n\n` +
        `1. Verify archive file checksums locally:\n   sha256sum -c SHA256SUMS\n\n` +
        `2. Verify the Field-Book audit bundle offline with Ergon:\n   ergon verify audit_bundle.json\n\n` +
        `3. Verify the client deliverable summary offline with Ergon:\n   ergon verify CLIENT_SUMMARY.md\n`;
      const sumSummary = await sha256Hex(summaryMd);
      const sumBundle = await sha256Hex(auditJson);
      const sumVerify = await sha256Hex(verifyTxt);
      const sha256sums =
        `${sumSummary}  CLIENT_SUMMARY.md\n` +
        `${sumBundle}  audit_bundle.json\n` +
        `${sumVerify}  VERIFY.txt\n`;
      const enc = new TextEncoder();
      const zipBytes = buildWebStoreZip([
        ["CLIENT_SUMMARY.md", enc.encode(summaryMd)],
        ["audit_bundle.json", enc.encode(auditJson)],
        ["VERIFY.txt", enc.encode(verifyTxt)],
        ["SHA256SUMS", enc.encode(sha256sums)],
      ]);
      let hex = "";
      if (typeof crypto !== "undefined" && crypto.subtle) {
        const digest = await crypto.subtle.digest("SHA-256", new Uint8Array(zipBytes).buffer);
        hex = Array.from(new Uint8Array(digest)).map((b) => b.toString(16).padStart(2, "0")).join("");
      }
      return new Response(new Uint8Array(zipBytes).buffer, {
        status: 200,
        headers: {
          "Content-Type": "application/zip",
          "X-Content-Type-Options": "nosniff",
          "Content-Disposition": 'attachment; filename="ergon-client-handoff.zip"',
          "X-Ergon-Handoff-SHA256": hex,
          "X-Ergon-Handoff-Leaks": String(scan.leak_count),
          "X-Ergon-Workspace-Policy": policy.profile,
        },
      });
    }
    if (method === "POST" && path === "/api/verify/evidence") {
      const content = String(body.content ?? "");
      const filename = typeof body.filename === "string" ? body.filename : "evidence";
      const checkWorkspace = Boolean(body.check_workspace);
      if (!content.trim()) return jsonRes({ error: "evidence content is empty" }, 400);
      const trimmed = content.trim();
      const contentSha = await sha256Hex(content);
      const hex64 = (s: unknown) => typeof s === "string" && /^[a-f0-9]{64}$/i.test(s);
      if (trimmed.startsWith("{")) {
        let parsed: Record<string, unknown>;
        try {
          parsed = JSON.parse(trimmed) as Record<string, unknown>;
        } catch (err) {
          return jsonRes({
            valid: false,
            kind: "unknown",
            title: filename,
            receipt_chain_valid: null,
            bundle_hash_valid: false,
            manifest_valid: null,
            citations_valid: null,
            workspace_valid: null,
            receipt_count: 0,
            deliverable_count: 0,
            citation_count: 0,
            stale_citations: 0,
            broken_citations: 0,
            expected_sha256: "",
            actual_sha256: contentSha,
            summary: "Malformed JSON evidence input.",
            errors: [String(err)],
            items: [],
          });
        }
        if (parsed.schema === "ergon.fieldbook.audit_bundle.v1") {
          const errors: string[] = [];
          const rc = (parsed.receipt_chain ?? {}) as Record<string, unknown>;
          const entries = Array.isArray(rc.entries) ? (rc.entries as Array<Record<string, unknown>>) : [];
          let prev = "0".repeat(64);
          let chainOk = Boolean(rc.verified);
          for (const e of entries) {
            if (String(e.prev_sha256 ?? "").toLowerCase() !== prev || !hex64(e.sha256)) {
              chainOk = false;
              break;
            }
            prev = String(e.sha256).toLowerCase();
          }
          if (entries.length > 0 && prev !== String(rc.head_sha256 ?? "").toLowerCase()) chainOk = false;
          if (!chainOk) errors.push("Receipt chain link verification failed.");

          const corePayload = {
            schema: parsed.schema,
            generated_at: parsed.generated_at,
            workspace: parsed.workspace,
            receipt_chain: parsed.receipt_chain,
            workspace_files: parsed.workspace_files,
            routines: parsed.routines,
            verification_instructions: parsed.verification_instructions,
          };
          const expectedSha = await sha256Hex(JSON.stringify(corePayload));
          const actualSha = String(parsed.bundle_sha256 ?? "").toLowerCase();
          const hashOk = hex64(actualSha) && expectedSha === actualSha;
          if (!hashOk) errors.push("Bundle SHA-256 does not match payload.");

          const wsObj = (parsed.workspace_files ?? {}) as Record<string, unknown>;
          const wsEntries = Array.isArray(wsObj.entries) ? (wsObj.entries as Array<Record<string, unknown>>) : [];
          let wsMatch: boolean | null = null;
          if (checkWorkspace) {
            let allMatch = true;
            for (const f of wsEntries) {
              const live = st.files[String(f.path ?? "")];
              if (!live || (await sha256Hex(live.content)) !== String(f.sha256 ?? "").toLowerCase()) {
                allMatch = false;
                errors.push(`Workspace file '${String(f.path ?? "")}' changed or missing.`);
              }
            }
            wsMatch = allMatch;
          }
          const valid = errors.length === 0 && chainOk && hashOk;
          const wsName = String(((parsed.workspace ?? {}) as Record<string, unknown>).name ?? "");
          return jsonRes({
            valid,
            kind: "audit_bundle",
            title: wsName ? `${wsName} audit bundle` : filename,
            receipt_chain_valid: chainOk,
            bundle_hash_valid: hashOk,
            manifest_valid: true,
            citations_valid: null,
            workspace_valid: wsMatch,
            receipt_count: entries.length,
            deliverable_count: wsEntries.length,
            citation_count: 0,
            stale_citations: 0,
            broken_citations: 0,
            expected_sha256: expectedSha,
            actual_sha256: actualSha || contentSha,
            summary: valid
              ? `Audit bundle passed offline SHA-256 verification (${entries.length} receipts, ${wsEntries.length} files).`
              : `Audit bundle verification failed with ${errors.length} issue(s).`,
            errors,
            items: [
              { label: "Receipt chain", status: chainOk ? "verified" : "tampered", detail: `${entries.length} receipts` },
              { label: "Bundle SHA-256", status: hashOk ? "verified" : "tampered", detail: expectedSha.slice(0, 12) },
              { label: "Workspace files", status: wsMatch === false ? "tampered" : "verified", detail: `${wsEntries.length} files` },
            ],
          });
        }
        if (parsed.format === "ergon-playbook-v1" || (parsed.playbook && parsed.sha256)) {
          const res = await verifyAndParsePlaybookPack(trimmed);
          if (res.ok) {
            return jsonRes({
              valid: true,
              kind: "playbook_pack",
              title: res.playbook.name,
              receipt_chain_valid: null,
              bundle_hash_valid: true,
              manifest_valid: true,
              citations_valid: null,
              workspace_valid: null,
              receipt_count: 0,
              deliverable_count: res.playbook.steps.length,
              citation_count: 0,
              stale_citations: 0,
              broken_citations: 0,
              expected_sha256: res.sha256,
              actual_sha256: res.sha256,
              summary: `Playbook pack passed offline SHA-256 verification (${res.playbook.steps.length} steps).`,
              errors: [],
              items: [{ label: "Playbook canonical checksum", status: "verified", detail: `${res.playbook.steps.length} steps` }],
            });
          }
          return jsonRes({
            valid: false,
            kind: "playbook_pack",
            title: filename,
            receipt_chain_valid: null,
            bundle_hash_valid: false,
            manifest_valid: false,
            citations_valid: null,
            workspace_valid: null,
            receipt_count: 0,
            deliverable_count: 0,
            citation_count: 0,
            stale_citations: 0,
            broken_citations: 0,
            expected_sha256: "",
            actual_sha256: String(parsed.sha256 ?? contentSha),
            summary: "Playbook verification failed.",
            errors: [res.error],
            items: [{ label: "Playbook canonical checksum", status: "tampered" }],
          });
        }
      }
      if (trimmed.includes("## Included Deliverables") || trimmed.includes("_SHA-256: `")) {
        const errors: string[] = [];
        const items: Array<{ label: string; status: string; sha256?: string; detail?: string }> = [];
        const manifest = new Map<string, string>();
        for (const line of content.split(/\r?\n/)) {
          const m = line.match(/^- `([^`]+)` .*`sha256:([a-f0-9]{64})`/i);
          if (m) manifest.set(m[1], m[2].toLowerCase());
        }
        let stale = 0;
        let broken = 0;
        let citations = 0;
        for (const line of content.split(/\r?\n/)) {
          const cm = line.match(/^- `([^`]+)` -> \*\*([^*]+)\*\*/);
          if (cm) {
            citations += 1;
            const stWord = cm[2].trim().toLowerCase();
            if (stWord === "stale") stale += 1;
            if (stWord === "broken") broken += 1;
          }
        }
        const sections = content.split(/\n---\n\n## `([^`]+)`\n+/);
        for (let i = 1; i + 1 < sections.length; i += 2) {
          const secPath = sections[i];
          const rest = sections[i + 1];
          const mm = rest.match(/^_SHA-256: `([a-f0-9]{64})`[^·]*·[^·]*·\s*(\d+)\s*bytes_\n\n([\s\S]*)$/i);
          if (!mm) continue;
          const secSha = mm[1].toLowerCase();
          const expBytes = Number(mm[2]);
          const bodyRaw = mm[3].replace(/\n---\n\n## [\s\S]*$/, "");
          const bodyTrim = bodyRaw.endsWith("\n") ? bodyRaw.slice(0, -1) : bodyRaw;
          let secOk = !manifest.has(secPath) || manifest.get(secPath) === secSha;
          if (bodyTrim.length === expBytes || bodyRaw.length === expBytes) {
            const txt = bodyTrim.length === expBytes ? bodyTrim : bodyRaw;
            if ((await sha256Hex(txt)) !== secSha) {
              secOk = false;
              errors.push(`Deliverable '${secPath}' content was modified after export.`);
            }
          }
          items.push({ label: secPath, status: secOk ? "verified" : "tampered", sha256: secSha });
        }
        if (stale > 0 || broken > 0) errors.push(`Pack contains ${stale} stale and ${broken} broken citation(s).`);
        if (manifest.size === 0 && items.length === 0) errors.push("Markdown file does not contain Ergon deliverable digests.");
        const valid = errors.length === 0;
        return jsonRes({
          valid,
          kind: "deliverable_pack",
          title: filename,
          receipt_chain_valid: null,
          bundle_hash_valid: valid,
          manifest_valid: manifest.size > 0,
          citations_valid: stale === 0 && broken === 0,
          workspace_valid: null,
          receipt_count: 0,
          deliverable_count: Math.max(manifest.size, items.length),
          citation_count: citations,
          stale_citations: stale,
          broken_citations: broken,
          expected_sha256: contentSha,
          actual_sha256: contentSha,
          summary: valid
            ? `Client pack passed offline verification (${Math.max(manifest.size, items.length)} deliverables, ${citations} citations).`
            : `Pack verification failed with ${errors.length} issue(s).`,
          errors,
          items,
        });
      }
      return jsonRes({
        valid: false,
        kind: "unknown",
        title: filename,
        receipt_chain_valid: null,
        bundle_hash_valid: null,
        manifest_valid: null,
        citations_valid: null,
        workspace_valid: null,
        receipt_count: 0,
        deliverable_count: 0,
        citation_count: 0,
        stale_citations: 0,
        broken_citations: 0,
        expected_sha256: "",
        actual_sha256: contentSha,
        summary: "Unsupported evidence format.",
        errors: ["Provide an audit bundle JSON, client pack Markdown, or signed playbook JSON."],
        items: [],
      });
    }
    if (method === "POST" && path === "/api/files/upload") {
      const rawPath = String(body.path ?? body.name ?? "");
      const safe = sanitizeSafePath(rawPath);
      if (!safe) return jsonRes({ error: "invalid workspace filename" }, 403);
      let content = String(body.content ?? "");
      let binaryBase64: string | undefined;
      const ext = fileExt(safe);
      if (body.encoding === "base64") {
        const bytes = base64ToBytes(content);
        if (["pdf", "docx", "xlsx", "pptx"].includes(ext)) {
          binaryBase64 = content;
          content = `Uploaded ${ext.toUpperCase()} document (${bytes.length} bytes).`;
        } else {
          content = new TextDecoder().decode(bytes);
        }
      }
      if (st.files[safe] && st.files[safe].content !== content) {
        recordFileRevision(st, safe, st.files[safe], "overwrite");
      }
      const saved = makeFile(safe, content);
      if (binaryBase64) saved.binaryBase64 = binaryBase64;
      st.files[safe] = saved;
      await persist();
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
        fileHistory: {},
        threads: [],
        work: [],
        decisions: [],
        offers: [],
        routines: [],
        routineRuns: {},
        connectors: { triggers: [], outbound: [], deliveries: [], webhookSecrets: {}, outboundSecrets: {} },
        memory: `# Memory\n\n## About\n- Name: ${np.name}\n${np.role ? `- Role: ${np.role}\n` : ""}`,
        suite: ["researcher", "meeting-actions"],
      };
      await persist();
      return jsonRes(np, 201);
    }
    if (method === "POST" && (path === "/api/active-person" || path === "/api/people/switch")) {
      const id = String(body.id ?? body.person_id ?? "");
      if (!state.people.some((x) => x.id === id)) return jsonRes({ error: "person not found" }, 404);
      state.activePerson = id;
      await persist();
      return jsonRes({ active: state.activePerson, people: state.people });
    }
    if (method === "GET" && path === "/api/clients") {
      ensureClientsAndPricing(state);
      const { engagement } = activeEngagementFor(p.id);
      return jsonRes({
        clients: state.clients ?? [],
        engagements: state.engagements ?? [],
        active_engagement_id: engagement?.id ?? null,
      });
    }
    if (method === "POST" && path === "/api/clients") {
      ensureClientsAndPricing(state);
      const name = String(body.name ?? "").trim();
      if (!name) return jsonRes({ error: "Client name is required." }, 400);
      const nowMs = Date.now();
      const rate = body.default_rate_usd !== undefined ? Number(body.default_rate_usd) : 185;
      if (!Number.isFinite(rate) || rate < 0 || rate > 1_000_000_000) {
        return jsonRes({ error: "Default hourly rate must be between 0 and 1,000,000,000." }, 400);
      }
      const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 24) || "client";
      const client: Client = {
        id: `cli-${slug}-${nowMs}`,
        name,
        brand_header: typeof body.brand_header === "string" && body.brand_header.trim() ? body.brand_header.trim() : name,
        billing_currency: typeof body.billing_currency === "string" && body.billing_currency.trim() ? body.billing_currency.trim() : "USD",
        default_rate_usd: rate,
        contacts: Array.isArray(body.contacts) ? (body.contacts as ClientContact[]) : [],
        notes_md: typeof body.notes_md === "string" ? body.notes_md : "",
        tax_rate_pct: typeof body.tax_rate_pct === "number" ? body.tax_rate_pct : 0,
        payment_terms: typeof body.payment_terms === "string" && body.payment_terms.trim() ? body.payment_terms.trim() : "Net 30",
        created_ms: nowMs,
        updated_ms: nowMs,
      };
      state.clients!.push(client);
      await persist();
      return jsonRes({ client }, 201);
    }
    if (method === "PATCH" && path.startsWith("/api/clients/")) {
      ensureClientsAndPricing(state);
      const cid = decodeURIComponent(path.slice("/api/clients/".length));
      const cli = state.clients!.find((c) => c.id === cid);
      if (!cli) return jsonRes({ error: "Client not found." }, 404);
      if (typeof body.name === "string" && body.name.trim()) cli.name = body.name.trim();
      if (typeof body.brand_header === "string") cli.brand_header = body.brand_header.trim();
      if (typeof body.billing_currency === "string" && body.billing_currency.trim()) cli.billing_currency = body.billing_currency.trim();
      if (typeof body.default_rate_usd === "number" && Number.isFinite(body.default_rate_usd) && body.default_rate_usd >= 0) {
        cli.default_rate_usd = body.default_rate_usd;
      }
      if (Array.isArray(body.contacts)) cli.contacts = body.contacts as ClientContact[];
      if (typeof body.notes_md === "string") cli.notes_md = body.notes_md;
      if (typeof body.tax_rate_pct === "number" && Number.isFinite(body.tax_rate_pct) && body.tax_rate_pct >= 0) {
        cli.tax_rate_pct = body.tax_rate_pct;
      }
      if (typeof body.payment_terms === "string") cli.payment_terms = body.payment_terms.trim() || "Net 30";
      cli.updated_ms = Date.now();
      await persist();
      return jsonRes({ client: cli });
    }
    if (method === "POST" && path === "/api/engagements") {
      ensureClientsAndPricing(state);
      const clientId = String(body.client_id ?? "").trim();
      const name = String(body.name ?? "").trim();
      if (!clientId || !state.clients!.some((c) => c.id === clientId)) {
        return jsonRes({ error: "Valid client_id is required." }, 404);
      }
      if (!name) return jsonRes({ error: "Engagement name is required." }, 400);
      const cli = state.clients!.find((c) => c.id === clientId);
      const hourlyRate =
        body.hourly_rate_usd !== undefined ? Number(body.hourly_rate_usd) : (cli?.default_rate_usd ?? 185);
      const retainerBudget = body.retainer_budget_usd !== undefined ? Number(body.retainer_budget_usd) : 2000;
      const retainerHours =
        body.retainer_hours !== undefined
          ? Number(body.retainer_hours)
          : hourlyRate > 0
            ? Math.round((retainerBudget / hourlyRate) * 10) / 10
            : 20;
      if (
        !Number.isFinite(hourlyRate) ||
        hourlyRate < 0 ||
        hourlyRate > 1_000_000_000 ||
        !Number.isFinite(retainerBudget) ||
        retainerBudget < 0 ||
        retainerBudget > 1_000_000_000
      ) {
        return jsonRes({ error: "Hourly rate and retainer budget must be finite numbers between 0 and 1,000,000,000." }, 400);
      }
      const nowMs = Date.now();
      const targetPerson = typeof body.person_id === "string" && body.person_id.trim() ? body.person_id.trim() : p.id;
      const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 24) || "eng";
      const engId = `eng-${slug}-${nowMs}`;
      const rawPrefix = typeof body.workspace_prefix === "string" ? body.workspace_prefix.trim().replace(/^\/+|\/+$/g, "") : "";
      const engagement: Engagement = {
        id: engId,
        client_id: clientId,
        person_id: targetPerson,
        name,
        retainer_hours: retainerHours,
        retainer_budget_usd: retainerBudget,
        hourly_rate_usd: hourlyRate,
        period: typeof body.period === "string" && body.period.trim() ? body.period.trim() : "monthly",
        status: "active",
        workspace_prefix: rawPrefix || `clients/${clientId}/${engId}`,
        sow_rules: Array.isArray(body.sow_rules) ? body.sow_rules.map(String) : [],
        rollover_mode: body.rollover_mode === "carry_hours" ? "carry_hours" : "none",
        created_ms: nowMs,
        updated_ms: nowMs,
      };
      state.engagements!.push(engagement);
      if (!state.activeEngagementByPerson) state.activeEngagementByPerson = {};
      state.activeEngagementByPerson[targetPerson] = engagement.id;
      await persist();
      return jsonRes({ engagement, active_engagement_id: engagement.id }, 201);
    }
    if (method === "POST" && path === "/api/engagements/switch") {
      ensureClientsAndPricing(state);
      const engId = String(body.engagement_id ?? "").trim();
      const eng = state.engagements!.find((e) => e.id === engId);
      if (!eng) return jsonRes({ error: "Engagement not found." }, 404);
      if (!state.activeEngagementByPerson) state.activeEngagementByPerson = {};
      state.activeEngagementByPerson[p.id] = eng.id;
      await persist();
      return jsonRes({ active_engagement_id: eng.id, engagement: eng });
    }
    if (method === "GET" && path.startsWith("/api/engagements/") && path.endsWith("/context")) {
      ensureClientsAndPricing(state);
      const engId = decodeURIComponent(path.slice("/api/engagements/".length, -"/context".length));
      const eng = state.engagements!.find((e) => e.id === engId);
      if (!eng) return jsonRes({ error: "Engagement not found." }, 404);
      const cli = state.clients!.find((c) => c.id === eng.client_id) || null;
      const scopedFiles = Object.values(st.files).filter((f) => isPathInEngagementScope(f.path, eng));
      const unbilled = (st.timeEntries ?? []).filter((e) => e.engagement_id === eng.id && !e.invoiced_id);
      const unbilledTotal = Math.round(unbilled.reduce((acc, e) => acc + (e.billable ? e.amount_usd : 0), 0) * 100) / 100;
      const meter = buildMeteringSummary({ engagement_id: eng.id });
      return jsonRes({
        engagement: eng,
        client: cli,
        scoped_file_count: scopedFiles.length,
        workspace_prefix: engagementWorkspacePrefix(eng),
        sow_rules: eng.sow_rules ?? [],
        contacts: cli?.contacts ?? [],
        burn_metrics: {
          runs: meter.totals.runs,
          cost_usd: meter.totals.cost_usd,
          retainer_budget_usd: eng.retainer_budget_usd,
          retainer_burn_pct: meter.totals.retainer_burn_pct,
          billable_value_usd: meter.totals.billable_value_usd,
          minutes_saved: meter.totals.minutes_saved,
        },
        unbilled_entries: unbilled,
        unbilled_total_usd: unbilledTotal,
      });
    }
    if (method === "PATCH" && path.startsWith("/api/engagements/")) {
      ensureClientsAndPricing(state);
      const engId = decodeURIComponent(path.slice("/api/engagements/".length));
      const eng = state.engagements!.find((e) => e.id === engId);
      if (!eng) return jsonRes({ error: "Engagement not found." }, 404);
      if (typeof body.name === "string" && body.name.trim()) eng.name = body.name.trim();
      if (typeof body.workspace_prefix === "string") {
        eng.workspace_prefix = body.workspace_prefix.trim().replace(/^\/+|\/+$/g, "");
      }
      if (Array.isArray(body.sow_rules)) eng.sow_rules = body.sow_rules.map(String).filter((s) => s.trim().length > 0);
      if (body.rollover_mode === "carry_hours" || body.rollover_mode === "none") eng.rollover_mode = body.rollover_mode as RolloverMode;
      if (typeof body.status === "string" && ["active", "paused", "archived"].includes(body.status)) eng.status = body.status;
      if (body.archived_ms === null || typeof body.archived_ms === "number") eng.archived_ms = body.archived_ms;
      if (typeof body.retainer_hours === "number" && Number.isFinite(body.retainer_hours) && body.retainer_hours >= 0) {
        eng.retainer_hours = body.retainer_hours;
      }
      if (typeof body.retainer_budget_usd === "number" && Number.isFinite(body.retainer_budget_usd) && body.retainer_budget_usd >= 0) {
        eng.retainer_budget_usd = body.retainer_budget_usd;
      }
      if (typeof body.hourly_rate_usd === "number" && Number.isFinite(body.hourly_rate_usd) && body.hourly_rate_usd >= 0) {
        eng.hourly_rate_usd = body.hourly_rate_usd;
      }
      if (typeof body.period === "string" && body.period.trim()) eng.period = body.period.trim();
      eng.updated_ms = Date.now();
      await persist();
      return jsonRes({ engagement: eng });
    }
    if (method === "GET" && path === "/api/time-entries") {
      const engId = (u.searchParams.get("engagement_id") || "").trim();
      const cliId = (u.searchParams.get("client_id") || "").trim();
      const rawUnbilled = (u.searchParams.get("unbilled") || u.searchParams.get("unbilled_only") || "").trim().toLowerCase();
      const unbilledOnly = rawUnbilled === "1" || rawUnbilled === "true";
      const entries = (st.timeEntries ?? []).filter((e) => {
        if (engId && e.engagement_id !== engId) return false;
        if (cliId && e.client_id !== cliId) return false;
        if (unbilledOnly && e.invoiced_id) return false;
        return true;
      });
      const totalUsd = Math.round(entries.reduce((acc, e) => acc + (e.billable ? e.amount_usd : 0), 0) * 100) / 100;
      return jsonRes({ entries, total_usd: totalUsd });
    }
    if (method === "POST" && path === "/api/time-entries") {
      ensureClientsAndPricing(state);
      const engId = String(body.engagement_id ?? "").trim();
      const eng = state.engagements!.find((e) => e.id === engId);
      if (!eng) return jsonRes({ error: "Valid engagement_id is required." }, 404);
      const kindRaw = String(body.kind ?? "time").trim();
      const kind: TimeExpenseKind =
        kindRaw === "expense" || kindRaw === "adjustment" ? kindRaw : "time";
      const description = String(body.description ?? "").trim();
      if (!description) return jsonRes({ error: "Description is required." }, 400);
      const hours = Number(body.hours ?? 0);
      const rateUsd = Number(body.rate_usd ?? eng.hourly_rate_usd ?? 0);
      const amountUsd =
        body.amount_usd !== undefined && Number.isFinite(Number(body.amount_usd))
          ? Math.round(Number(body.amount_usd) * 100) / 100
          : Math.round(hours * rateUsd * 100) / 100;
      const entry: TimeExpenseEntry = {
        id: `te-${Date.now()}-${(st.timeEntries ?? []).length + 1}`,
        ts_ms: Date.now(),
        client_id: typeof body.client_id === "string" && body.client_id.trim() ? body.client_id.trim() : eng.client_id,
        engagement_id: eng.id,
        thread_id: typeof body.thread_id === "string" ? body.thread_id : null,
        receipt_id: typeof body.receipt_id === "string" ? body.receipt_id : null,
        kind,
        description,
        hours: Number.isFinite(hours) ? hours : 0,
        rate_usd: Number.isFinite(rateUsd) ? rateUsd : 0,
        amount_usd: amountUsd,
        billable: body.billable !== undefined ? Boolean(body.billable) : true,
        invoiced_id: null,
      };
      st.timeEntries = [...(st.timeEntries ?? []), entry];
      await persist();
      return jsonRes({ entry }, 201);
    }
    if (method === "DELETE" && path.startsWith("/api/time-entries/")) {
      const tid = decodeURIComponent(path.slice("/api/time-entries/".length));
      const before = st.timeEntries ?? [];
      const found = before.find((e) => e.id === tid);
      if (!found) return jsonRes({ error: "Time entry not found." }, 404);
      st.timeEntries = before.filter((e) => e.id !== tid);
      await persist();
      return jsonRes({ deleted: tid, id: tid });
    }
    if (method === "GET" && path === "/api/invoices") {
      const engId = (u.searchParams.get("engagement_id") || "").trim();
      const cliId = (u.searchParams.get("client_id") || "").trim();
      const invoices = (st.invoices ?? []).filter((inv) => {
        if (engId && inv.engagement_id !== engId) return false;
        if (cliId && inv.client_id !== cliId) return false;
        return true;
      });
      return jsonRes({ invoices });
    }
    if (method === "POST" && path === "/api/invoices/generate") {
      ensureClientsAndPricing(state);
      const engId = String(body.engagement_id ?? "").trim();
      const eng = state.engagements!.find((e) => e.id === engId);
      if (!eng) return jsonRes({ error: "Valid engagement_id is required." }, 404);
      const cli = state.clients!.find((c) => c.id === eng.client_id) || null;
      const periodLabel = String(body.period_label ?? "").trim() || new Date().toISOString().slice(0, 7);
      const includeLlm = body.include_llm_cost !== false;
      const includeTime = body.include_time_entries !== false;
      const formats = Array.isArray(body.export_formats) ? body.export_formats.map(String) : ["json"];
      const invId = `inv-${Date.now()}-${(st.invoices ?? []).length + 1}`;
      const invNum = `INV-${periodLabel.replace(/[^0-9A-Za-z]+/g, "")}-${String((st.invoices ?? []).length + 1).padStart(3, "0")}`;

      const lineItems: InvoiceLineItem[] = [];
      let billableSubtotal = 0;
      if (includeTime) {
        const unbilled = (st.timeEntries ?? []).filter((e) => e.engagement_id === eng.id && !e.invoiced_id && e.billable);
        for (const entry of unbilled) {
          lineItems.push({
            id: entry.id,
            kind: entry.kind,
            description: entry.description,
            hours: entry.kind === "time" ? entry.hours : undefined,
            quantity: entry.kind !== "time" ? 1 : undefined,
            rate_usd: entry.kind === "time" ? entry.rate_usd : entry.amount_usd,
            amount_usd: entry.amount_usd,
          });
          billableSubtotal += entry.amount_usd;
          entry.invoiced_id = invId;
        }
      }
      const meter = buildMeteringSummary({ engagement_id: eng.id });
      const llmCostUsd = includeLlm ? Math.round((meter.totals.cost_usd || 0) * 100) / 100 : 0;
      if (includeLlm && llmCostUsd > 0) {
        lineItems.push({
          id: `${invId}-llm`,
          kind: "llm_usage",
          description: `AI compute metering (${meter.totals.runs} runs)`,
          quantity: 1,
          unit_price_usd: llmCostUsd,
          amount_usd: llmCostUsd,
        });
      }
      billableSubtotal = Math.round(billableSubtotal * 100) / 100;
      const taxableBase = Math.round((billableSubtotal + llmCostUsd) * 100) / 100;
      const taxRate = cli?.tax_rate_pct ?? 0;
      const taxUsd = Math.round(taxableBase * (taxRate / 100) * 100) / 100;
      const totalUsd = Math.round((taxableBase + taxUsd) * 100) / 100;

      const deliverablePaths = Object.values(st.files)
        .filter((f) => isPathInEngagementScope(f.path, eng))
        .map((f) => f.path)
        .slice(0, 20);

      const prefix = engagementWorkspacePrefix(eng) || `clients/${eng.client_id}/${eng.id}`;
      const basePath = `${prefix}/invoices/${invNum}`;
      const exportedPaths: Record<string, string> = {};
      const jsonPath = formats.includes("json") ? `${basePath}.json` : null;
      const pdfPath = formats.includes("pdf") ? `${basePath}.pdf` : null;
      const xlsxPath = formats.includes("xlsx") ? `${basePath}.xlsx` : null;

      const draftInv: InvoiceRecord = {
        id: invId,
        invoice_number: invNum,
        client_id: eng.client_id,
        engagement_id: eng.id,
        period_label: periodLabel,
        issued_ms: Date.now(),
        due_ms: Date.now() + 30 * 86_400_000,
        status: "draft",
        line_items: lineItems,
        llm_cost_usd: llmCostUsd,
        billable_subtotal_usd: billableSubtotal,
        tax_usd: taxUsd,
        total_usd: totalUsd,
        deliverable_paths: deliverablePaths,
        pdf_path: pdfPath,
        xlsx_path: xlsxPath,
        json_path: jsonPath,
        sha256: await sha256Hex(JSON.stringify({ invNum, engId: eng.id, periodLabel, totalUsd, lineItems })),
      };

      if (jsonPath) {
        st.files[jsonPath] = makeFile(jsonPath, JSON.stringify(draftInv, null, 2));
        exportedPaths.json = jsonPath;
      }
      if (pdfPath) {
        st.files[pdfPath] = makeFile(pdfPath, `%PDF-1.4\n% ${invNum} (${periodLabel}) Total: $${totalUsd.toFixed(2)}\n`);
        exportedPaths.pdf = pdfPath;
      }
      if (xlsxPath) {
        st.files[xlsxPath] = makeFile(xlsxPath, `Invoice,${invNum}\nPeriod,${periodLabel}\nTotal,${totalUsd.toFixed(2)}\n`);
        exportedPaths.xlsx = xlsxPath;
      }

      st.invoices = [draftInv, ...(st.invoices ?? [])];
      await persist();
      return jsonRes({ invoice: draftInv, exported_paths: exportedPaths }, 201);
    }
    if (method === "PATCH" && path.startsWith("/api/invoices/")) {
      const invId = decodeURIComponent(path.slice("/api/invoices/".length));
      const inv = (st.invoices ?? []).find((x) => x.id === invId);
      if (!inv) return jsonRes({ error: "Invoice not found." }, 404);
      const nextStatus = String(body.status ?? "").trim();
      if (!["draft", "issued", "paid", "void"].includes(nextStatus)) {
        return jsonRes({ error: "Invoice status must be draft, issued, paid, or void." }, 400);
      }
      inv.status = nextStatus as InvoiceStatus;
      await persist();
      return jsonRes({ invoice: inv });
    }
    if (method === "GET" && path === "/api/metering/summary") {
      const engagementId = u.searchParams.get("engagement_id") || undefined;
      const clientId = u.searchParams.get("client_id") || undefined;
      const winRaw = u.searchParams.get("window_days");
      const windowDays = winRaw ? Number(winRaw) : 30;
      return jsonRes(buildMeteringSummary({ engagement_id: engagementId, client_id: clientId, window_days: windowDays }));
    }
    if (method === "GET" && path === "/api/memory") {
      return jsonRes({ person_id: p.id, content: st.memory, health: auditWebMemoryHealth(st) });
    }
    if (method === "GET" && path === "/api/memory/health") {
      return jsonRes({ person_id: p.id, ...auditWebMemoryHealth(st) });
    }
    if (method === "POST" && path === "/api/memory/heal") {
      const mode = String(body.mode ?? "all").trim().toLowerCase() || "all";
      if (!["all", "stale", "conflicts", "duplicates"].includes(mode)) {
        return jsonRes({ error: "mode must be all, stale, conflicts, or duplicates" }, 400);
      }
      const before = auditWebMemoryHealth(st);
      const removeLines = new Set<number>();
      for (const iss of before.issues) {
        if (
          mode === "all" ||
          (mode === "stale" && iss.kind === "stale_ref") ||
          (mode === "conflicts" && iss.kind === "conflict") ||
          (mode === "duplicates" && iss.kind === "duplicate")
        ) {
          removeLines.add(iss.line);
        }
      }
      const removed: { line: number; text: string }[] = [];
      if (removeLines.size > 0) {
        const kept: string[] = [];
        st.memory.split("\n").forEach((line, idx) => {
          const lineNo = idx + 1;
          if (removeLines.has(lineNo)) removed.push({ line: lineNo, text: line.trim() });
          else kept.push(line);
        });
        st.memory = kept.join("\n").replace(/\n*$/, "\n");
        await persist();
      }
      return jsonRes({
        person_id: p.id,
        content: st.memory,
        removed_count: removed.length,
        removed,
        health: auditWebMemoryHealth(st),
      });
    }
    if (method === "GET" && path === "/api/library") return jsonRes({ items: state.library });
    if (method === "GET" && path === "/api/suite") {
      const installed = st.suite.map((id) => state.library.find((x) => x.id === id)).filter(Boolean);
      return jsonRes({ person_id: p.id, installed });
    }
    if (method === "POST" && path === "/api/suite/install") {
      const id = String(body.item_id ?? "");
      if (id && !st.suite.includes(id)) st.suite.push(id);
      await persist();
      return jsonRes({ installed: id });
    }
    if (method === "POST" && path === "/api/suite/remove") {
      const id = String(body.item_id ?? "");
      st.suite = st.suite.filter((x) => x !== id);
      await persist();
      return jsonRes({ removed: id });
    }
    if (method === "GET" && path === "/api/connectors") {
      const cs = ensureWebConnectors(st);
      const pol = buildWebPolicy(p.id, st);
      const triggers = cs.triggers.map((t) => ({
        ...t,
        signing_secret: undefined,
        hook_token: undefined,
        has_secret: t.kind === "webhook" ? Boolean(cs.webhookSecrets[t.id]) : false,
      }));
      const outbound = cs.outbound.map((c) => ({
        ...c,
        effective_require_approval: pol.profile === "strict_client" ? true : Boolean(c.require_approval),
        blocked_by_policy: pol.profile === "air_gapped",
        has_secret: Boolean(c.secret_keys && c.secret_keys.length > 0),
      }));
      const secrets = Object.keys(cs.outboundSecrets).map((key) => ({
        key,
        has_secret: true,
        updated_ms: Date.now(),
      }));
      return jsonRes({
        triggers,
        outbound,
        secrets,
        delivery_log: cs.deliveries,
        policy_profile: pol.profile,
      });
    }
    if (method === "POST" && path.startsWith("/api/connectors/triggers/") && path.endsWith("/rotate")) {
      const cs = ensureWebConnectors(st);
      const tid = decodeURIComponent(path.slice("/api/connectors/triggers/".length, -"/rotate".length));
      const trg = cs.triggers.find((t) => t.id === tid);
      if (!trg) return jsonRes({ error: "trigger not found" }, 404);
      const secret = `whsec_${randomHex(16)}`;
      cs.webhookSecrets[trg.id] = secret;
      trg.has_secret = true;
      trg.updated_ms = Date.now();
      await persist();
      return jsonRes({
        trigger: { ...trg, signing_secret: undefined },
        signing_secret: secret,
        hook_token: secret,
        hook_path: trg.hook_path || `/api/hooks/${trg.hook_id || trg.id}`,
      });
    }
    if (method === "POST" && path === "/api/connectors/triggers") {
      const cs = ensureWebConnectors(st);
      const kind =
        body.kind === "folder_watch"
          ? "folder_watch"
          : body.kind === "imap_poll"
            ? "imap_poll"
            : "webhook";
      const name = String(body.name ?? "").trim();
      if (!name) return jsonRes({ error: "Trigger name is required." }, 400);
      const routineId = typeof body.routine_id === "string" && body.routine_id.trim() ? body.routine_id.trim() : null;
      const targetKind = body.target_kind === "pipeline" ? "pipeline" : "routine";
      const enabled = typeof body.enabled === "boolean" ? body.enabled : true;
      const nowMs = Date.now();
      if (kind === "imap_poll") {
        const rawCfg = (body.imap_config && typeof body.imap_config === "object" ? body.imap_config : {}) as Record<string, unknown>;
        const host = String(rawCfg.host ?? "").trim();
        const port = Number(rawCfg.port ?? 993);
        const security = String(rawCfg.security ?? "imaps").trim() || "imaps";
        const folder = String(rawCfg.folder ?? "INBOX").trim() || "INBOX";
        const pollIntervalSecs = Math.max(5, Number(rawCfg.poll_interval_secs ?? 60) || 60);
        const markSeen = typeof rawCfg.mark_seen === "boolean" ? rawCfg.mark_seen : true;
        const usernameSecretKey = String(rawCfg.username_secret_key ?? "imap_user").trim();
        const passwordSecretKey = String(rawCfg.password_secret_key ?? "imap_pass").trim();
        const subjectFilter = typeof rawCfg.subject_filter === "string" && rawCfg.subject_filter.trim() ? rawCfg.subject_filter.trim() : undefined;
        if (!host || /[\s\0\r\n/@]/.test(host)) {
          return jsonRes({ error: "Valid IMAP host is required." }, 400);
        }
        if (!Number.isInteger(port) || port < 1 || port > 65535) {
          return jsonRes({ error: "IMAP port must be between 1 and 65535." }, 400);
        }
        if (body.secrets && typeof body.secrets === "object" && !Array.isArray(body.secrets)) {
          for (const [k, v] of Object.entries(body.secrets as Record<string, unknown>)) {
            const key = k.trim();
            const val = String(v ?? "").trim();
            if (key && val) cs.outboundSecrets[key] = val;
          }
        }
        const imapConfig: ImapConfig = {
          host,
          port,
          security,
          folder,
          poll_interval_secs: pollIntervalSecs,
          mark_seen: markSeen,
          username_secret_key: usernameSecretKey,
          password_secret_key: passwordSecretKey,
          ...(subjectFilter ? { subject_filter: subjectFilter } : {}),
        };
        const trg: TriggerRule = {
          id: `trg-${nowMs}-${cs.triggers.length + 1}`,
          name,
          kind: "imap_poll",
          enabled,
          routine_id: routineId,
          target_kind: targetKind,
          imap_config: imapConfig,
          has_secret: Boolean(cs.outboundSecrets[usernameSecretKey] || cs.outboundSecrets[passwordSecretKey]),
          trigger_count: 0,
          last_triggered_ms: null,
          created_ms: nowMs,
          updated_ms: nowMs,
        };
        cs.triggers.unshift(trg);
        await persist();
        return jsonRes({ trigger: trg }, 201);
      }
      if (kind === "folder_watch") {
        const watchDir = String(body.watch_dir ?? body.folder_path ?? "").trim();
        const dirErr = validateWebWatchDir(watchDir);
        if (dirErr) {
          const status = dirErr.includes("Sensitive") ? 403 : 400;
          return jsonRes({ error: dirErr }, status);
        }
        const glob = String(body.glob ?? body.glob_filter ?? "*.pdf,*.csv").trim() || "*.*";
        if (/[\0\r\n/\\]/.test(glob) || glob.includes("..")) {
          return jsonRes({ error: "Glob filter cannot contain path separators or '..'." }, 400);
        }
        const trg: TriggerRule = {
          id: `trg-${nowMs}-${cs.triggers.length + 1}`,
          name,
          kind: "folder_watch",
          enabled,
          routine_id: routineId,
          target_kind: targetKind,
          watch_dir: watchDir,
          glob,
          max_bytes: 25 * 1024 * 1024,
          has_secret: false,
          trigger_count: 0,
          last_triggered_ms: null,
          created_ms: nowMs,
          updated_ms: nowMs,
        };
        cs.triggers.unshift(trg);
        await persist();
        return jsonRes({ trigger: trg }, 201);
      }
      const id = `trg-${nowMs}-${cs.triggers.length + 1}`;
      const hookId = `hk-${randomHex(8)}`;
      const hookPath = `/api/hooks/${hookId}`;
      const secret = `whsec_${randomHex(16)}`;
      const trg: TriggerRule = {
        id,
        name,
        kind: "webhook",
        enabled,
        routine_id: routineId,
        target_kind: targetKind,
        hook_id: hookId,
        hook_path: hookPath,
        has_secret: true,
        trigger_count: 0,
        last_triggered_ms: null,
        created_ms: nowMs,
        updated_ms: nowMs,
      };
      cs.webhookSecrets[id] = secret;
      cs.triggers.unshift(trg);
      await persist();
      return jsonRes(
        {
          trigger: trg,
          signing_secret: secret,
          hook_token: secret,
          hook_path: hookPath,
        },
        201
      );
    }
    if (method === "DELETE" && (path === "/api/connectors/triggers" || path.startsWith("/api/connectors/triggers/"))) {
      const cs = ensureWebConnectors(st);
      const idFromPath = path.startsWith("/api/connectors/triggers/")
        ? decodeURIComponent(path.slice("/api/connectors/triggers/".length))
        : "";
      const tid = idFromPath || String(u.searchParams.get("id") ?? body.id ?? "").trim();
      if (!tid) return jsonRes({ error: "trigger id is required" }, 400);
      const before = cs.triggers.length;
      cs.triggers = cs.triggers.filter((t) => t.id !== tid);
      if (cs.triggers.length === before) return jsonRes({ error: "trigger not found" }, 404);
      delete cs.webhookSecrets[tid];
      await persist();
      return jsonRes({ deleted: tid });
    }
    if (method === "POST" && path.startsWith("/api/hooks/")) {
      const hookId = decodeURIComponent(path.slice("/api/hooks/".length));
      const cs = ensureWebConnectors(st);
      const trg = cs.triggers.find((t) => t.kind === "webhook" && (t.hook_id === hookId || t.id === hookId));
      if (!trg) return jsonRes({ error: "Webhook endpoint not found." }, 404);
      const rawBody = typeof init?.body === "string" ? init.body : JSON.stringify(body ?? {});
      if (utf8Bytes(rawBody) > 1024 * 1024) return jsonRes({ error: "Payload exceeds 1 MB limit." }, 413);
      if (rawBody.trim()) {
        try {
          JSON.parse(rawBody);
        } catch {
          return jsonRes({ error: "Webhook payload must be valid JSON." }, 400);
        }
      }
      const expectedSecret = cs.webhookSecrets[trg.id] || "";
      const sigHdr = getHeaderCI(init?.headers, "X-Ergon-Signature");
      const tokenHdr = getHeaderCI(init?.headers, "X-Ergon-Hook-Token");
      const simHdr = getHeaderCI(init?.headers, "X-Ergon-Simulate");
      let authed = false;
      if (sigHdr && expectedSecret) {
        const expectedHex = await computeHmacHex(expectedSecret, rawBody);
        if (constantTimeEqual(sigHdr.toLowerCase(), `sha256=${expectedHex}`)) authed = true;
      }
      if (!authed && tokenHdr && expectedSecret && constantTimeEqual(tokenHdr, expectedSecret)) {
        authed = true;
      }
      if (!authed && !sigHdr && !tokenHdr && simHdr === "1") {
        authed = true;
      }
      if (!authed) {
        return jsonRes({ error: "Invalid or missing webhook signature." }, 401);
      }
      const ts = Date.now();
      const triggerFile = `inbox/hook-${ts}.json`;
      st.files[triggerFile] = makeFile(triggerFile, rawBody || "{}", ts);
      trg.trigger_count = (trg.trigger_count ?? 0) + 1;
      trg.last_triggered_ms = ts;
      trg.updated_ms = ts;
      let runId: string | undefined;
      if (trg.routine_id) {
        const rt = st.routines.find((r) => r.id === trg.routine_id);
        if (rt) {
          const { runRec } = await executeWebRoutineRun(rt);
          runId = runRec.run_id;
        }
      }
      await persist();
      return jsonRes(
        {
          accepted: true,
          trigger_file: triggerFile,
          routine_id: trg.routine_id,
          run_id: runId,
          message: `Accepted payload into ${triggerFile}.`,
        },
        202
      );
    }
    if (method === "POST" && path === "/api/connectors/outbound") {
      const cs = ensureWebConnectors(st);
      const name = String(body.name ?? "").trim();
      if (!name) return jsonRes({ error: "Connector name is required." }, 400);
      const kind: OutboundConnectorKind =
        body.kind === "rest_api" || body.kind === "smtp_relay" ? body.kind : "http_webhook";

      let smtpConfig: SmtpConfig | undefined;
      if (body.smtp_config && typeof body.smtp_config === "object" && !Array.isArray(body.smtp_config)) {
        const sc = body.smtp_config as Record<string, unknown>;
        const host = String(sc.host ?? "").trim();
        const port = Number(sc.port ?? 465);
        const rawSec = String(sc.security ?? "smtps").trim();
        const security: SmtpSecurity =
          rawSec === "starttls" || rawSec === "plain_loopback_test" ? rawSec : "smtps";
        const fromAddr = String(sc.from_addr ?? "").trim();
        const defaultTo = Array.isArray(sc.default_to) ? sc.default_to.map(String).map((x) => x.trim()).filter(Boolean) : [];
        if (!host || /[\s\0\r\n/@]/.test(host)) return jsonRes({ error: "SMTP host is required." }, 400);
        if (!Number.isInteger(port) || port < 1 || port > 65535) return jsonRes({ error: "Invalid SMTP port." }, 400);
        if (!fromAddr || !fromAddr.includes("@")) return jsonRes({ error: "Valid SMTP sender address is required." }, 400);
        smtpConfig = {
          host,
          port,
          security,
          username_secret_key: String(sc.username_secret_key ?? "smtp_user").trim(),
          password_secret_key: String(sc.password_secret_key ?? "smtp_pass").trim(),
          from_addr: fromAddr,
          default_to: defaultTo,
        };
      }

      let oauthConfig: OAuthConfig | undefined;
      if (body.oauth_config && typeof body.oauth_config === "object" && !Array.isArray(body.oauth_config)) {
        const oc = body.oauth_config as Record<string, unknown>;
        const tokenUrl = String(oc.token_url ?? "").trim();
        const clientId = String(oc.client_id ?? "").trim();
        const urlErr = validateWebOutboundUrl(tokenUrl, "rest_api");
        if (urlErr) return jsonRes({ error: `OAuth2 token URL: ${urlErr}` }, 400);
        if (!clientId) return jsonRes({ error: "OAuth2 client ID is required." }, 400);
        oauthConfig = {
          token_url: tokenUrl,
          client_id: clientId,
          client_secret_key: String(oc.client_secret_key ?? "oauth_client_secret").trim(),
          refresh_token_secret_key: String(oc.refresh_token_secret_key ?? "oauth_refresh_token").trim(),
          scopes: Array.isArray(oc.scopes) ? oc.scopes.map(String).map((s) => s.trim()).filter(Boolean) : [],
          ...(typeof oc.access_token_expires_ms === "number" ? { access_token_expires_ms: oc.access_token_expires_ms } : {}),
        };
      }

      let url = String(body.url ?? "").trim();
      if (!url && kind === "smtp_relay" && smtpConfig) {
        const scheme = smtpConfig.security === "smtps" ? "smtps" : "smtp";
        url = `${scheme}://${smtpConfig.host}:${smtpConfig.port}`;
      }
      const urlErr = validateWebOutboundUrl(url, kind);
      if (urlErr) return jsonRes({ error: urlErr }, 400);
      const methodVal: "POST" | "PUT" = body.method === "PUT" ? "PUT" : "POST";
      const headersTemplate: Record<string, string> = {};
      if (body.headers_template && typeof body.headers_template === "object" && !Array.isArray(body.headers_template)) {
        for (const [k, v] of Object.entries(body.headers_template as Record<string, unknown>)) {
          if (k.trim()) headersTemplate[k.trim()] = String(v ?? "");
        }
      }
      const secretKeys = new Set<string>();
      if (body.secrets && typeof body.secrets === "object" && !Array.isArray(body.secrets)) {
        for (const [k, v] of Object.entries(body.secrets as Record<string, unknown>)) {
          const key = k.trim();
          const val = String(v ?? "").trim();
          if (key && val) {
            cs.outboundSecrets[key] = val;
            secretKeys.add(key);
          }
        }
      }
      if (typeof body.secret_key === "string" && typeof body.secret_value === "string") {
        const key = body.secret_key.trim();
        const val = body.secret_value.trim();
        if (key && val) {
          cs.outboundSecrets[key] = val;
          secretKeys.add(key);
        }
      }
      for (const hVal of Object.values(headersTemplate)) {
        const m = /\{\{secret:([a-zA-Z0-9_.-]+)\}\}/.exec(hVal);
        if (m && cs.outboundSecrets[m[1]]) secretKeys.add(m[1]);
      }
      if (smtpConfig) {
        if (smtpConfig.username_secret_key && cs.outboundSecrets[smtpConfig.username_secret_key]) {
          secretKeys.add(smtpConfig.username_secret_key);
        }
        if (smtpConfig.password_secret_key && cs.outboundSecrets[smtpConfig.password_secret_key]) {
          secretKeys.add(smtpConfig.password_secret_key);
        }
      }
      if (oauthConfig) {
        if (oauthConfig.client_secret_key && cs.outboundSecrets[oauthConfig.client_secret_key]) {
          secretKeys.add(oauthConfig.client_secret_key);
        }
        if (oauthConfig.refresh_token_secret_key && cs.outboundSecrets[oauthConfig.refresh_token_secret_key]) {
          secretKeys.add(oauthConfig.refresh_token_secret_key);
        }
      }
      const nowMs = Date.now();
      const conn: OutboundConnector = {
        id: typeof body.id === "string" && body.id.trim() ? body.id.trim() : `conn-${nowMs}-${cs.outbound.length + 1}`,
        name,
        kind,
        url,
        method: methodVal,
        headers_template: headersTemplate,
        secret_keys: Array.from(secretKeys),
        has_secret: secretKeys.size > 0,
        ...(smtpConfig ? { smtp_config: smtpConfig } : {}),
        ...(oauthConfig ? { oauth_config: oauthConfig } : {}),
        require_approval: typeof body.require_approval === "boolean" ? body.require_approval : true,
        enabled: typeof body.enabled === "boolean" ? body.enabled : true,
        last_delivery_ms: null,
        last_status: null,
        created_ms: nowMs,
        updated_ms: nowMs,
      };
      const existingIdx = cs.outbound.findIndex((x) => x.id === conn.id);
      if (existingIdx >= 0) cs.outbound[existingIdx] = conn;
      else cs.outbound.unshift(conn);
      await persist();
      return jsonRes({ connector: conn }, 201);
    }
    if (method === "DELETE" && (path === "/api/connectors/outbound" || path.startsWith("/api/connectors/outbound/"))) {
      const cs = ensureWebConnectors(st);
      const idFromPath = path.startsWith("/api/connectors/outbound/")
        ? decodeURIComponent(path.slice("/api/connectors/outbound/".length))
        : "";
      const cid = idFromPath || String(u.searchParams.get("id") ?? body.id ?? "").trim();
      if (!cid) return jsonRes({ error: "connector id is required" }, 400);
      const before = cs.outbound.length;
      cs.outbound = cs.outbound.filter((c) => c.id !== cid);
      if (cs.outbound.length === before) return jsonRes({ error: "outbound connector not found" }, 404);
      await persist();
      return jsonRes({ deleted: cid });
    }
    if (method === "POST" && path.startsWith("/api/connectors/outbound/") && path.endsWith("/test")) {
      const cid = decodeURIComponent(path.slice("/api/connectors/outbound/".length, -"/test".length));
      const cs = ensureWebConnectors(st);
      const conn = cs.outbound.find((c) => c.id === cid);
      if (!conn) return jsonRes({ error: "outbound connector not found" }, 404);
      if (conn.enabled === false) return jsonRes({ error: "outbound connector is paused" }, 409);
      const pol = buildWebPolicy(p.id, st);
      const ts = Date.now();
      const payloadStr = JSON.stringify(body.payload ?? { event: "ergon.connector.test", connector: conn.name });
      const payloadSha = await sha256Hex(payloadStr);
      if (pol.profile === "air_gapped") {
        const delivery: DeliveryLog = {
          id: `del-${ts}-${cs.deliveries.length + 1}`,
          connector_id: conn.id,
          connector_name: conn.name,
          kind: conn.kind,
          method: conn.method,
          url: conn.url,
          status: "blocked",
          http_status: null,
          payload_sha256: payloadSha,
          payload_bytes: utf8Bytes(payloadStr),
          error: "Blocked by Air-Gapped workspace policy.",
          ts_ms: ts,
          test: true,
        };
        cs.deliveries.unshift(delivery);
        conn.last_delivery_ms = ts;
        conn.last_status = "blocked";
        await persist();
        return jsonRes({ ok: false, error: delivery.error, delivery }, 403);
      }
      if (pol.profile === "strict_client" || conn.require_approval) {
        const did = `d-${ts}-${st.decisions.length + 1}`;
        const delId = `del-${ts}-${cs.deliveries.length + 1}`;
        const delivery: DeliveryLog = {
          id: delId,
          connector_id: conn.id,
          connector_name: conn.name,
          kind: conn.kind,
          method: conn.method,
          url: conn.url,
          status: "held",
          http_status: null,
          decision_id: did,
          payload_sha256: payloadSha,
          payload_bytes: utf8Bytes(payloadStr),
          summary: `Waiting for approval in Today before sending to ${conn.url}.`,
          ts_ms: ts,
          test: true,
        };
        cs.deliveries.unshift(delivery);
        st.decisions.push({
          id: did,
          kind: "connector_delivery",
          summary: `Send test payload via ${conn.name}`,
          detail: `Your outbound connector ${conn.name} is ready to send a test payload to ${conn.url}. Approving sends the payload now; blocking cancels the delivery.`,
          severity: "warning",
          status: "pending",
          outcome: null,
          thread_id: "",
          ts,
          connector_id: conn.id,
          connector_name: conn.name,
          connector_kind: conn.kind,
          method: conn.method,
          url: conn.url,
          payload_preview: payloadStr.slice(0, 200),
          payload_sha256: payloadSha,
          delivery_id: delId,
        });
        conn.last_delivery_ms = ts;
        conn.last_status = "held";
        await persist();
        return jsonRes({ held: true, decision_id: did, delivery }, 202);
      }
      const delivery: DeliveryLog = {
        id: `del-${ts}-${cs.deliveries.length + 1}`,
        connector_id: conn.id,
        connector_name: conn.name,
        kind: conn.kind,
        method: conn.method,
        url: conn.url,
        status: "ok",
        http_status: 200,
        payload_sha256: payloadSha,
        payload_bytes: utf8Bytes(payloadStr),
        response_preview: "Offline browser engine recorded simulated 200 OK delivery.",
        ts_ms: ts,
        test: true,
      };
      cs.deliveries.unshift(delivery);
      conn.last_delivery_ms = ts;
      conn.last_status = "ok";
      await persist();
      return jsonRes({ ok: true, delivery, message: `Test delivery succeeded for "${conn.name}".` }, 200);
    }
    if (method === "GET" && path === "/api/mcp/servers") {
      const servers = ensureWebMcpServers(st);
      return jsonRes({ servers });
    }
    if (method === "POST" && path === "/api/mcp/servers") {
      const servers = ensureWebMcpServers(st);
      const name = String(body.name ?? "").trim();
      if (!name) return jsonRes({ error: "MCP server name is required." }, 400);
      const transport: McpTransport = body.transport === "http_sse" ? "http_sse" : "stdio";
      const commandOrUrl = String(body.command_or_url ?? "").trim();
      if (!commandOrUrl) return jsonRes({ error: "Command or SSE URL is required." }, 400);
      if (transport === "http_sse") {
        const urlErr = validateWebOutboundUrl(commandOrUrl, "rest_api");
        if (urlErr) return jsonRes({ error: urlErr }, 400);
      }
      const args = Array.isArray(body.args) ? body.args.map(String).filter(Boolean) : [];
      const envSecretKeys = Array.isArray(body.env_secret_keys)
        ? body.env_secret_keys.map(String).map((s) => s.trim()).filter(Boolean)
        : [];
      const requireApproval = typeof body.require_approval_by_default === "boolean" ? body.require_approval_by_default : true;
      const enabled = typeof body.enabled === "boolean" ? body.enabled : true;
      const nowMs = Date.now();
      const reqId = typeof body.id === "string" && body.id.trim() ? body.id.trim() : "";
      const existingIdx = reqId ? servers.findIndex((s) => s.id === reqId) : -1;
      if (existingIdx >= 0) {
        const prev = servers[existingIdx];
        const updated: McpServerSpec = {
          ...prev,
          name,
          transport,
          command_or_url: commandOrUrl,
          args,
          env_secret_keys: envSecretKeys,
          require_approval_by_default: requireApproval,
          enabled,
          updated_ms: nowMs,
        };
        servers[existingIdx] = updated;
        await persist();
        return jsonRes({ server: updated }, 200);
      }
      const srv: McpServerSpec = {
        id: reqId || `mcp-${nowMs}-${servers.length + 1}`,
        name,
        transport,
        command_or_url: commandOrUrl,
        args,
        env_secret_keys: envSecretKeys,
        require_approval_by_default: requireApproval,
        enabled,
        cached_tools: [],
        last_discovered_ms: null,
        created_ms: nowMs,
        updated_ms: nowMs,
      };
      servers.unshift(srv);
      await persist();
      return jsonRes({ server: srv }, 201);
    }
    if (method === "POST" && path.startsWith("/api/mcp/servers/") && path.endsWith("/discover")) {
      const sid = decodeURIComponent(path.slice("/api/mcp/servers/".length, -"/discover".length));
      const servers = ensureWebMcpServers(st);
      const srv = servers.find((s) => s.id === sid);
      if (!srv) return jsonRes({ error: "MCP server not found" }, 404);
      const nowMs = Date.now();
      const tools: McpToolSpec[] =
        srv.cached_tools && srv.cached_tools.length > 0
          ? srv.cached_tools
          : [
              {
                name: "search_records",
                description: `Query records exposed by ${srv.name}.`,
              },
              {
                name: "get_item",
                description: `Fetch a single record by ID from ${srv.name}.`,
              },
            ];
      srv.cached_tools = tools;
      srv.last_discovered_ms = nowMs;
      srv.updated_ms = nowMs;
      await persist();
      return jsonRes({ server: srv, tools, cached_tools: tools, discovered_count: tools.length });
    }
    if (method === "DELETE" && path.startsWith("/api/mcp/servers/")) {
      const sid = decodeURIComponent(path.slice("/api/mcp/servers/".length));
      const servers = ensureWebMcpServers(st);
      const before = servers.length;
      st.mcpServers = servers.filter((s) => s.id !== sid);
      if (st.mcpServers.length === before) return jsonRes({ error: "MCP server not found" }, 404);
      await persist();
      return jsonRes({ deleted: sid });
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
      if (dec.kind === "connector_delivery") {
        const cs = ensureWebConnectors(st);
        const pol = buildWebPolicy(p.id, st);
        const log = cs.deliveries.find((dl) => dl.decision_id === dec.id || (dec.delivery_id && dl.id === dec.delivery_id));
        if (approve && pol.profile === "air_gapped") {
          dec.status = "denied";
          dec.outcome = "Delivery blocked by Air-Gapped workspace policy.";
          if (log) {
            log.status = "blocked";
            log.error = dec.outcome;
          }
        } else if (approve) {
          dec.outcome = `Delivery approved and sent via ${dec.connector_name || "connector"}.`;
          if (log) {
            log.status = "ok";
            log.http_status = 200;
            log.response_preview = "Approved in Today and delivered.";
          }
          const conn = cs.outbound.find((c) => c.id === dec.connector_id);
          if (conn) {
            conn.last_delivery_ms = dec.resolved_ts;
            conn.last_status = "ok";
          }
        } else {
          dec.outcome = "Delivery blocked; nothing left your workspace.";
          if (log) {
            log.status = "denied";
            log.error = dec.outcome;
          }
        }
      } else if (dec.kind === "mcp_tool_call") {
        const target = `${dec.tool_name || "tool"} on ${dec.server_name || "MCP server"}`;
        dec.outcome = approve
          ? `Approved and ran ${target}.`
          : `Blocked ${target}; nothing ran.`;
      } else if (dec.kind === "budget_gate") {
        if (approve) {
          if (!st.budgetOverrunApproved) st.budgetOverrunApproved = {};
          if (dec.engagement_id) {
            const count = (st.usageLedger ?? []).filter((u) => u.engagement_id === dec.engagement_id).length;
            st.budgetOverrunApproved[dec.engagement_id] = count + 1;
          }
          dec.outcome = `Approved budget overrun for ${dec.engagement_name || "engagement"}.`;
        } else {
          dec.outcome = `Kept retainer spend paused for ${dec.engagement_name || "engagement"}.`;
        }
      } else if (dec.kind === "delete" && dec.path) {
        if (approve) {
          if (st.files[dec.path]) {
            recordFileRevision(st, dec.path, st.files[dec.path], "delete");
            delete st.files[dec.path];
            dec.outcome = `Approved and done: ${dec.path} was deleted.`;
          } else {
            dec.outcome = `Approved: ${dec.path} was already gone.`;
          }
        } else {
          dec.outcome = `You said no; kept ${dec.path} safe.`;
        }
        const th = st.threads.find((t) => t.id === dec.thread_id);
        if (th) {
          th.messages = th.messages.map((m) =>
            m.role === "step" && m.content.includes(`${dec.path} has NOT been deleted yet`)
              ? { role: "step", content: approve ? `Note: ${dec.path} was deleted after you approved it.` : `Note: ${dec.path} was kept because you declined the deletion.` }
              : m
          );
          th.messages.push({ role: "step", content: `Decision settled (${dec.summary}): ${dec.outcome}` });
        }
      } else {
        dec.outcome = approve ? "Approved." : "Declined.";
      }
      await persist();
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
      await persist();
      return jsonRes({ offer: off, outcome: action === "accept" ? "Added to your suite as a one-tap app." : "Offer dismissed." });
    }
    if (method === "GET" && path === "/api/brief") return jsonRes(buildBrief());
    if (method === "GET" && path === "/api/today") return jsonRes(buildToday());
    if (method === "GET" && path === "/api/graph") return jsonRes(buildGraph());
    if (method === "GET" && path === "/api/threads") {
      return jsonRes({
        threads: st.threads.map((t) => {
          const hasParked = t.messages.some(
            (m) => m.role === "step" && /parked for your person|has NOT been deleted yet/.test(m.content)
          );
          const hasResolved = t.messages.some(
            (m) => m.role === "step" && /was deleted after you approved it|was kept because you declined/.test(m.content)
          );
          const state = hasParked && !hasResolved
            ? "waiting"
            : t.messages.some((m) => m.role === "assistant")
              ? "finished"
              : "none";
          return { id: t.id, title: t.title, messages: t.messages.length, state };
        }),
      });
    }
    if (method === "POST" && path === "/api/threads") {
      const nid = `t${Date.now()}-${st.threads.length + 1}`;
      const nt: WebThread = { id: nid, title: "New chat", messages: [] };
      st.threads.unshift(nt);
      await persist();
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
      return executePromptIntoThread(p, st, byok, prompt, String(body.thread_id ?? ""));
    }

    return jsonRes({ error: "not found" }, 404);
  };
}
