// Today's Desk (Morning Brief, Drag-and-Drop Workspace & Operational Queue):
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import {
  Briefcase,
  Check,
  CheckCircle2,
  Clock,
  Coins,
  FileCheck2,
  FileDown,
  FolderOpen,
  Plus,
  Printer,
  ReceiptText,
  Repeat,
  Search,
  Send,
  ShieldAlert,
  Sparkles,
  Trash2,
  TrendingUp,
  Upload,
  Wallet,
  Wrench,
  X,
  XCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Pill } from "@/components/kibo-ui/pill";
import { Details, FileGlyph, WaitingPill } from "@/chat/Entry";
import { LiveRunTimeline } from "@/chat/LiveRunTimeline";
import { CheckpointCard, PipelineRunStrip } from "@/shell/CheckpointCard";
import { PanelModal } from "@/shell/PanelModal";
import { personInitials } from "@/shell/PersonSelect";
import { StarterPlaybooksBar } from "@/shell/StarterPlaybooksBar";
import { usePaper } from "@/views/usePaper";
import { FORMULA_ESTIMATE_TOOLTIP, resolveEffectiveHourlyRate } from "@/lib/meteringFormula";
import {
  call,
  downloadWorkspaceFile,
  exportHandoffBundleZip,
  hasV050Api,
  type BriefJob,
  type Client,
  type Decision,
  type Engagement,
  type InvoiceRecord,
  type InvoiceStatus,
  type LiveRunStep,
  type MemoryHealth,
  type MeteringSummary,
  type Offer,
  type Person,
  type PipelineResumePayload,
  type PipelineRun,
  type RunUsage,
  type StarterLaunchResult,
  type StarterPlaybook,
  type Status,
  type TimeExpenseEntry,
  type TimeExpenseKind,
  type WorkspaceFile,
} from "@/lib/api";
import { exportClientSummaryPack, exportReceiptDownload, fetchSummaryPreflight, type PackPreflightReport } from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { extractPlaybookFromRun, normalizeFiles, relTime, threadMs } from "@/lib/words";

export { usePaper };

const RECEIPTS_PAGE_SIZE = 20;
const EVENTS_PAGE_SIZE = 10;

type DecisionCopy = {
  body: string;
  approve: string;
  /** null: a notice with a single acknowledgement. */
  deny: string | null;
  approvedToast: string;
  deniedToast: string;
  security: boolean;
};

function formatUsd(amount: number): string {
  const n = Number(amount) || 0;
  if (n > 0 && n < 0.01) return `$${n.toFixed(4)}`;
  return `$${n.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function formatMinutesSaved(mins: number): string {
  const m = Math.max(0, Math.round(mins));
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return rem > 0 ? `${h}h ${rem}m` : `${h}h`;
}

const SECURITY_KINDS = new Set(["security", "security_notice", "allow_host", "memory_note", "publish_skill", "budget_gate"]);

/** Plain consequence copy and button words for each kind of decision. */
function decisionCopy(d: Decision): DecisionCopy {
  const kind = String(d.kind ?? "");
  const detail = typeof d.detail === "string" ? d.detail.trim() : "";
  const security = SECURITY_KINDS.has(kind);
  if (/delete/.test(kind)) {
    return {
      body: "Approving deletes the file for good. Keeping it leaves the file where it is. Your agent has not touched it yet.",
      approve: "Approve",
      deny: "Keep the file",
      approvedToast: "Approved and done.",
      deniedToast: "You said no; kept safe.",
      security,
    };
  }
  if (kind === "security") {
    const src = d.source?.kind;
    const approve = src === "url" || src === "host" ? "Block that site" : src === "file" ? "Quarantine the file" : "Got it";
    return {
      body: detail || "Ergon blocked something suspicious while your agent worked. Nothing harmful went through.",
      approve,
      deny: src === "url" || src === "host" || src === "file" ? "Dismiss" : null,
      approvedToast: approve === "Got it" ? "Noted." : "Done: it is out of your agent's reach.",
      deniedToast: "Dismissed.",
      security,
    };
  }
  if (kind === "allow_host") {
    return {
      body: detail || "Ergon blocked an address on your private network that did not come from you.",
      approve: "Allow it",
      deny: "Keep blocked",
      approvedToast: "Allowed for future tasks.",
      deniedToast: "Kept blocked.",
      security,
    };
  }
  if (kind === "memory_note") {
    return {
      body: detail || "Your agent wanted to remember something Ergon thought you should see first.",
      approve: "Remember it",
      deny: "Discard",
      approvedToast: "Saved to your agent's memory.",
      deniedToast: "Discarded.",
      security,
    };
  }
  if (kind === "publish_skill") {
    return {
      body: detail || "Your agent wants to share a skill with everyone's library.",
      approve: "Publish",
      deny: "Don't publish",
      approvedToast: "Published to the library.",
      deniedToast: "Not published.",
      security,
    };
  }
  if (kind === "tool_offer") {
    const toolName = d.tool_name || "New tool";
    const passed = d.test_report?.passed;
    const total = d.test_report?.total;
    const testsLine =
      typeof passed === "number" && typeof total === "number"
        ? `${passed}/${total} sandbox tests passed.`
        : "Sandbox tests passed.";
    return {
      body: `Would this help? ${d.description || detail || `Install ${toolName} into your suite and share it with the company library.`} ${testsLine}`,
      approve: "Approve & Install",
      deny: "Deny",
      approvedToast: `Installed "${toolName}" into your suite and the company library.`,
      deniedToast: "You said no — tool was not installed or shared.",
      security: false,
    };
  }
  if (kind === "security_notice") {
    return {
      body: detail || "Ergon's tripwires caught something and handled it. Nothing real was exposed.",
      approve: "Got it",
      deny: null,
      approvedToast: "Noted.",
      deniedToast: "Noted.",
      security,
    };
  }
  if (kind === "budget_gate") {
    const hasBudget = typeof d.retainer_budget_usd === "number" && Number.isFinite(d.retainer_budget_usd) && d.retainer_budget_usd > 0;
    const budget = hasBudget ? `$${d.retainer_budget_usd!.toFixed(2)}` : "the retainer limit";
    const spent = typeof d.spent_usd === "number" && Number.isFinite(d.spent_usd) ? `$${d.spent_usd.toFixed(2)}` : "the current spend";
    const target = d.engagement_name || "this engagement";
    return {
      body:
        detail ||
        `Spending for ${target} has reached ${spent} against a ${budget} retainer limit. Approving lets this task continue past the cap. Declining keeps spending paused with no external model calls made.`,
      approve: "Approve overrun",
      deny: "Pause spend",
      approvedToast: "Budget overrun approved; engagement unblocked.",
      deniedToast: "Retainer spend kept paused.",
      security: true,
    };
  }
  if (kind === "connector_delivery") {
    const dest = d.url || d.target_url || d.connector_name || "the configured destination";
    const verb = d.method ? `${d.method} ` : "";
    return {
      body:
        detail ||
        `Approving sends this outbound payload to ${verb}${dest} using your saved host credentials. Blocking stops the delivery and keeps all data inside your workspace.`,
      approve: "Send delivery",
      deny: "Block delivery",
      approvedToast: "Delivery approved and sent.",
      deniedToast: "Delivery blocked; nothing left your workspace.",
      security: true,
    };
  }
  if (kind === "mcp_tool_call") {
    const serverLabel = d.server_name || d.connector_name || d.server_id || "the MCP server";
    const toolName = d.tool_name || "this tool";
    return {
      body:
        detail ||
        `Approving runs ${toolName} on ${serverLabel} using your saved environment credentials. Blocking cancels the call and keeps your workspace unchanged.`,
      approve: "Run tool",
      deny: "Block tool call",
      approvedToast: "MCP tool call approved.",
      deniedToast: "MCP tool call blocked; nothing ran.",
      security: true,
    };
  }
  return {
    body: "Your agent raised this while it worked. Your answer is recorded here and unblocks any waiting workflow step automatically.",
    approve: "Approve",
    deny: "Decline",
    approvedToast: "Approved and done.",
    deniedToast: "You said no; kept safe.",
    security,
  };
}

export function PaperSide({
  people,
  teamError,
  active,
  onSwitch,
  offers,
  learned,
  memoryHealth,
  onHealMemory,
  files,
  failed,
  onAct,
  onOpenFile,
  onOpenFilesPanel,
  onUploadMany,
}: {
  people: Person[] | null;
  teamError?: string;
  active: string;
  onSwitch: (id: string) => void;
  offers: Offer[] | null;
  learned: string[] | null;
  memoryHealth?: MemoryHealth | null;
  onHealMemory?: () => Promise<number | null>;
  files: WorkspaceFile[];
  failed?: boolean;
  onAct: (path: string, done: string) => void;
  onOpenFile: (path: string) => void;
  onOpenFilesPanel: () => void;
  onUploadMany: (files: FileList | File[]) => Promise<number>;
}) {
  const [fileQuery, setFileQuery] = useState("");
  const [healingMemory, setHealingMemory] = useState(false);
  const sideUploadRef = useRef<HTMLInputElement>(null);

  const filteredFiles = useMemo(() => {
    const q = fileQuery.trim().toLowerCase();
    const list = q
      ? files.filter((f) => f.path.toLowerCase().includes(q) || f.ext.toLowerCase().includes(q))
      : files;
    return list.slice(0, 8);
  }, [files, fileQuery]);

  return (
    <div className="flex flex-col gap-6" data-answers="4">
      {/* Workspace Files Quick Search & Upload */}
      <section aria-labelledby="today-files-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <FolderOpen className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
            <h2 id="today-files-h" className="text-base font-medium text-ink">Workspace files</h2>
          </div>
          <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
            {files.length}
          </span>
        </div>

        {files.length > 0 && (
          <InputGroup className="rounded-sm bg-board/50 [&_input]:max-md:min-h-11">
            <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
              <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              id="today-file-search"
              type="search"
              aria-label="Filter workspace files"
              placeholder="Search files…"
              value={fileQuery}
              onChange={(e) => setFileQuery(e.target.value)}
              className="text-sm max-md:min-h-11"
            />
            {fileQuery && (
              <InputGroupAddon align="inline-end" className="pr-1.5">
                <Button
                  type="button"
                  size="icon-xs"
                  variant="ghost"
                  className="max-md:min-h-11 max-md:min-w-11"
                  aria-label="Clear workspace file search"
                  onClick={() => setFileQuery("")}
                >
                  <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                </Button>
              </InputGroupAddon>
            )}
          </InputGroup>
        )}

        {files.length === 0 ? (
          <p className="text-sm leading-5 text-ink-2">
            Drop CSV, Markdown, or JSON files anywhere on Today, or upload below.
          </p>
        ) : filteredFiles.length === 0 ? (
          <p className="text-sm text-ink-2">No files match &ldquo;{fileQuery}&rdquo;.</p>
        ) : (
          <ul className="divide-y divide-hairline rounded-sm border border-hairline bg-board/30">
            {filteredFiles.map((f) => (
              <li key={f.path} className="flex items-center justify-between gap-2 px-2.5 py-1.5">
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  aria-label={`Open ${f.path}`}
                  onClick={() => onOpenFile(f.path)}
                  className="h-auto min-h-11 w-full justify-between gap-2 px-1 text-left font-normal text-ink hover:bg-board md:min-h-8"
                >
                  <span className="flex min-w-0 items-center gap-1.5">
                    <FileGlyph name={f.path} className="size-3.5" />
                    <span className="mono truncate text-xs">{f.path}</span>
                  </span>
                  <span className="text-xs text-ink-3">Open</span>
                </Button>
              </li>
            ))}
          </ul>
        )}

        <div className="flex flex-wrap items-center gap-2">
          <input
            ref={sideUploadRef}
            type="file"
            multiple
            className="hidden"
            accept=".csv,.md,.txt,.json,.log,.tsv,.pdf,.docx,.xlsx,.pptx,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
            onChange={async (ev) => {
              if (ev.target.files?.length) await onUploadMany(ev.target.files);
              ev.target.value = "";
            }}
          />
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="max-md:min-h-11"
            onClick={() => sideUploadRef.current?.click()}
          >
            <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
            Upload files
          </Button>
          {files.length > 0 && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={onOpenFilesPanel}
            >
              Browse all ({files.length})
            </Button>
          )}
        </div>
      </section>

      <section id="offers-list" aria-labelledby="offers-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4">
        <div className="flex items-center justify-between gap-2">
          <h2 id="offers-h" className="text-base font-medium text-ink">Offers from your agent</h2>
          {offers && offers.length > 0 && (
            <Pill variant="secondary" className="bg-tag-plan-bg px-2 py-0.5 text-xs font-medium text-tag-plan-fg">
              <Sparkles className="size-3" strokeWidth={1.5} aria-hidden />
              <span className="mono">{offers.length}</span>
            </Pill>
          )}
        </div>
        {!offers && (
          <p className="text-sm text-ink-2">
            {failed ? "Offers could not be loaded. Retry the desk to try again." : "Loading offers…"}
          </p>
        )}
        <div className="flex flex-col gap-3">
          {(offers ?? []).map((o) => {
            const isProactiveTool =
              o.kind === "install_library_tool" ||
              o.kind === "build_tool" ||
              /^Would this help\?/i.test(String(o.title ?? ""));
            return (
              <div
                key={o.id}
                className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-board/60 p-3.5"
              >
                {isProactiveTool && (
                  <span className="text-xs font-medium text-ink-2">Would this help?</span>
                )}
                <p className="text-sm font-medium leading-5 text-ink">{String(o.title ?? "Suggested capability")}</p>
                <p className="text-sm leading-5 text-ink-2">{String(o.detail ?? "")}</p>
                <div className="flex flex-wrap gap-2 pt-0.5">
                  <Button
                    size="sm"
                    className="max-md:min-h-11"
                    onClick={() =>
                      onAct(
                        isProactiveTool
                          ? `/api/offers/${encodeURIComponent(o.id)}/approve`
                          : `/api/offers/${encodeURIComponent(o.id)}/accept`,
                        isProactiveTool
                          ? "Approved and added to your suite."
                          : "Added to your suite as a one-tap app."
                      )
                    }
                  >
                    {isProactiveTool ? "Approve" : "Make it one tap"}
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    className="max-md:min-h-11"
                    onClick={() =>
                      onAct(
                        isProactiveTool
                          ? `/api/offers/${encodeURIComponent(o.id)}/deny`
                          : `/api/offers/${encodeURIComponent(o.id)}/dismiss`,
                        "Offer dismissed."
                      )
                    }
                  >
                    {isProactiveTool ? "Deny" : "No, thanks"}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
        {offers && !offers.length && (
          <p className="text-sm leading-5 text-ink-2">
            No offers right now. Your agent proposes a one-tap app when you ask for the same kind of work three times.
          </p>
        )}
      </section>

      <section aria-labelledby="learned-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 id="learned-h" className="text-base font-medium text-ink">What your agent has learned</h2>
          {memoryHealth && (learned?.length || memoryHealth.issue_count > 0) ? (
            <span
              data-today-memory-badge={memoryHealth.status}
              className={cn(
                "mono inline-flex items-center gap-1 rounded-chip px-2 py-0.5 text-[11px] font-medium",
                memoryHealth.issue_count === 0 ? "bg-ok-soft text-ok" : "bg-warn-soft text-warn"
              )}
            >
              {memoryHealth.issue_count === 0
                ? "Healthy"
                : `${memoryHealth.issue_count} ${memoryHealth.issue_count === 1 ? "issue" : "issues"}`}
            </span>
          ) : null}
        </div>
        {!learned && <p className="text-sm text-ink-2">{failed ? "Could not be loaded with the brief." : "Loading…"}</p>}
        {learned && learned.length > 0 && (
          <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm leading-5 text-ink-2">
            {learned.map((l, i) => <li key={i} className="text-ink">{String(l)}</li>)}
          </ul>
        )}
        {memoryHealth && memoryHealth.issue_count > 0 && onHealMemory && (
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board px-2.5 py-2 text-xs text-ink-2">
            <span>
              {memoryHealth.conflicts > 0 ? `${memoryHealth.conflicts} conflict` : ""}
              {memoryHealth.conflicts > 0 && memoryHealth.stale_refs > 0 ? " · " : ""}
              {memoryHealth.stale_refs > 0 ? `${memoryHealth.stale_refs} stale path` : ""}
            </span>
            <Button
              type="button"
              size="xs"
              variant="outline"
              data-action="heal-memory-inline"
              disabled={healingMemory}
              onClick={async () => {
                setHealingMemory(true);
                await onHealMemory();
                setHealingMemory(false);
              }}
            >
              {healingMemory ? "Pruning…" : "Resolve & Prune"}
            </Button>
          </div>
        )}
        {learned && !learned.length && (
          <p className="text-sm leading-5 text-ink-2">
            Nothing learned yet. Preferences and corrections your agent picks up appear here.
          </p>
        )}
      </section>

      <section aria-labelledby="team-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <h2 id="team-h" className="text-base font-medium text-ink">Team</h2>
        {!people && <p role={teamError ? "alert" : undefined} className="text-sm text-ink-2">{teamError || "Loading the team…"}</p>}
        {people && !people.length && (
          <p className="text-sm text-ink-2">No people yet. Add someone from the rail and their agent keeps its own workspace and memory.</p>
        )}
        {people && people.length > 0 && (
          <ul className="flex flex-col gap-1">
            {people.map((p) => {
              const isCurrent = p.id === active;
              const pName = String(p?.name ?? "").trim() || "Unnamed";
              const pRole = String(p?.role ?? "").trim() || "Personal workspace";
              return (
                <li key={p.id}>
                  <Button
                    variant="ghost"
                    aria-current={isCurrent ? "true" : undefined}
                    onClick={() => onSwitch(p.id)}
                    className={cn(
                      "h-auto sm:h-auto min-h-11 w-full justify-start gap-2.5 whitespace-normal px-2.5 py-1.5 text-left font-normal",
                      isCurrent ? "bg-board" : "hover:bg-board"
                    )}
                  >
                    <span
                      aria-hidden
                      className={cn(
                        "grid size-7 shrink-0 place-items-center rounded-full text-xs font-medium",
                        isCurrent ? "bg-ink text-canvas" : "bg-blue-soft text-ink"
                      )}
                    >
                      {personInitials(pName)}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className={cn("block truncate text-sm text-ink", isCurrent && "font-medium")}>{pName}</span>
                      <span className="block truncate text-xs text-ink-2">{pRole}</span>
                    </span>
                  </Button>
                </li>
              );
            })}
          </ul>
        )}
      </section>
    </div>
  );
}

export const PaperFace = memo(function PaperFace({
  paper,
  status,
  personName,
  dateLabel,
  people,
  teamError,
  activePerson,
  files,
  busy,
  liveTimeline,
  liveElapsedMs,
  runStartedMs,
  streamMode,
  cancelling,
  onCancel,
  onSwitchPerson,
  onAsk,
  onOpen,
  onOpenFile,
  onOpenFilesPanel,
  onUploadMany,
  memoryHealth,
  onHealMemory,
  onExportAuditBundle,
  onSaveRoutine,
  starterPlaybooks = [],
  onLaunchStarter,
  onOpenPlaybooks,
  pipelineRuns = [],
  onResumePipeline,
  onRetryPipeline,
  onCancelPipeline,
  metering = null,
  liveRunUsage = null,
  clients = [],
  engagements = [],
  activeEngagementId = "",
  onOpenWorkspaceSwitcher,
  timeEntries = [],
  invoices = [],
  onLoadTimeEntries,
  onAddTimeEntry,
  onRemoveTimeEntry,
  onLoadInvoices,
  onGenerateInvoice,
  onUpdateInvoiceStatus,
}: {
  paper: ReturnType<typeof usePaper>;
  status: Status | null | "error";
  personName: string;
  dateLabel: string;
  people: Person[] | null;
  teamError: string;
  activePerson: string;
  files: WorkspaceFile[];
  busy: boolean;
  liveTimeline: LiveRunStep[];
  liveElapsedMs: number;
  runStartedMs?: number;
  streamMode: "sse" | "poll";
  cancelling: boolean;
  onCancel: () => void;
  onSwitchPerson: (id: string) => void;
  onAsk: () => void;
  onOpen: (threadId: string) => void;
  onOpenFile: (path: string) => void;
  onOpenFilesPanel: () => void;
  onUploadMany: (files: FileList | File[]) => Promise<number>;
  memoryHealth?: MemoryHealth | null;
  onHealMemory?: () => Promise<number | null>;
  onExportAuditBundle?: (format: "json" | "md") => Promise<void>;
  onSaveRoutine?: (
    name: string,
    prompt: string,
    schedule?: string,
    playbook?: { steps?: string[]; expected_inputs?: string[]; pack_deliverables?: boolean }
  ) => Promise<boolean>;
  starterPlaybooks?: StarterPlaybook[];
  onLaunchStarter?: (starterId: string, runNow?: boolean) => Promise<StarterLaunchResult | null>;
  onOpenPlaybooks?: () => void;
  pipelineRuns?: PipelineRun[];
  onResumePipeline?: (runId: string, payload: PipelineResumePayload) => Promise<void>;
  onRetryPipeline?: (runId: string) => Promise<void>;
  onCancelPipeline?: (runId: string) => Promise<void>;
  metering?: MeteringSummary | null;
  liveRunUsage?: RunUsage | null;
  clients?: Client[];
  engagements?: Engagement[];
  activeEngagementId?: string;
  onOpenWorkspaceSwitcher?: () => void;
  timeEntries?: TimeExpenseEntry[];
  invoices?: InvoiceRecord[];
  onLoadTimeEntries?: (opts?: { engagement_id?: string; force?: boolean }) => Promise<void>;
  onAddTimeEntry?: (input: {
    client_id?: string;
    engagement_id: string;
    kind: TimeExpenseKind;
    description: string;
    hours?: number;
    rate_usd?: number;
    amount_usd?: number;
    billable?: boolean;
  }) => Promise<TimeExpenseEntry | null>;
  onRemoveTimeEntry?: (id: string, engagementId?: string) => Promise<boolean>;
  onLoadInvoices?: (opts?: { engagement_id?: string; force?: boolean }) => Promise<void>;
  onGenerateInvoice?: (input: {
    engagement_id: string;
    period_label: string;
    include_llm_cost: boolean;
    include_time_entries: boolean;
    export_formats: Array<"pdf" | "xlsx" | "json">;
  }) => Promise<InvoiceRecord | null>;
  onUpdateInvoiceStatus?: (id: string, status: InvoiceStatus, engagementId?: string) => Promise<InvoiceRecord | null>;
}) {
  const { data, failed, load, loadAllReceipts, act } = paper;
  const [dragOver, setDragOver] = useState(false);
  const [exportingAudit, setExportingAudit] = useState(false);
  const [exportingSummary, setExportingSummary] = useState<"md" | "html" | null>(null);
  const [summaryGate, setSummaryGate] = useState<(PackPreflightReport & { summary_runs_count?: number }) | null>(null);
  const [receiptQuery, setReceiptQuery] = useState("");
  const [remoteReceipts, setRemoteReceipts] = useState<BriefJob[] | null>(null);
  const [loadingAll, setLoadingAll] = useState(false);
  const [savedRoutineKeys, setSavedRoutineKeys] = useState<Record<string, boolean>>({});
  const [onlyWithFiles, setOnlyWithFiles] = useState(false);
  const [receiptPage, setReceiptPage] = useState(0);
  const [eventsPage, setEventsPage] = useState(0);
  const [busyDecisionId, setBusyDecisionId] = useState<string | null>(null);
  const decisionLockRef = useRef<Set<string>>(new Set());
  const exportAuditLockRef = useRef(false);

  const [timeModalOpen, setTimeModalOpen] = useState(false);
  const [timeKind, setTimeKind] = useState<TimeExpenseKind>("time");
  const [timeDesc, setTimeDesc] = useState("");
  const [timeHours, setTimeHours] = useState("1.5");
  const [timeRate, setTimeRate] = useState("200");
  const [timeAmount, setTimeAmount] = useState("300");
  const [timeBillable, setTimeBillable] = useState(true);
  const [timeErr, setTimeErr] = useState("");
  const [savingTime, setSavingTime] = useState(false);
  const [deletingEntryId, setDeletingEntryId] = useState<string | null>(null);

  const [scopedHandoffBusy, setScopedHandoffBusy] = useState(false);

  const [invoicesOpen, setInvoicesOpen] = useState(false);
  const [invPeriodLabel, setInvPeriodLabel] = useState(() => new Date().toISOString().slice(0, 7));
  const [invIncludeLlm, setInvIncludeLlm] = useState(true);
  const [invIncludeTime, setInvIncludeTime] = useState(true);
  const [invFormats, setInvFormats] = useState<Array<"pdf" | "xlsx" | "json">>(["pdf", "xlsx", "json"]);
  const [generatingInv, setGeneratingInv] = useState(false);
  const [updatingInvId, setUpdatingInvId] = useState<string | null>(null);
  const [selectedInvoiceId, setSelectedInvoiceId] = useState<string | null>(null);

  const handleDecisionAct = async (id: string, action: "approve" | "deny", toastMsg: string) => {
    if (decisionLockRef.current.has(id)) return;
    decisionLockRef.current.add(id);
    setBusyDecisionId(id);
    try {
      await act(`/api/decisions/${encodeURIComponent(id)}/${action}`, toastMsg);
    } finally {
      decisionLockRef.current.delete(id);
      setBusyDecisionId((cur) => (cur === id ? null : cur));
    }
  };

  const handleSummaryExport = async (fmt: "md" | "html") => {
    setExportingSummary(fmt);
    try {
      const targets = files.slice(0, 15).map((f) => f.path);
      const res = await exportClientSummaryPack(
        targets,
        fmt,
        `${personName || "Client"} Summary Pack`
      );
      const msg = `Exported ${res.filename} (${res.citationGate || "100% Verified"}${res.summaryCitations ? ` · ${res.summaryCitations}` : ""}, sha256: ${res.sha256.slice(0, 12)}…)`;
      if ((res.summaryStale ?? 0) > 0) toast.warning(msg);
      else toast.success(msg);
    } catch {
      toast.error(`Could not export Client Summary Pack (.${fmt}).`);
    } finally {
      setExportingSummary(null);
    }
  };

  // Most urgent first: a task Ergon stopped for safety, then real choices
  // (deletions, questions, approvals), then informational notices —
  // newest first within each rank.
  const urgency = (d: Decision): number => {
    if (d.severity === "critical" || d.stopped) return 3;
    if (d.kind === "security_notice" || (d.kind === "security" && !d.source)) return 1;
    return 2;
  };
  const pending = data
    ? data.decisions
        .slice()
        .reverse()
        .filter((d) => d.status === "pending")
        .map((d, i) => ({ d, i }))
        .sort((a, b) => urgency(b.d) - urgency(a.d) || a.i - b.i)
        .map((x) => x.d)
    : [];
  const settled = data ? data.decisions.slice().reverse().filter((d) => d.status !== "pending") : [];
  const threadOf = useCallback(
    (x: BriefJob) => x.thread_id ?? data?.threads.find((t) => t.title === x.task)?.id,
    [data?.threads]
  );
  const when = (x: BriefJob) => {
    const id = threadOf(x);
    const ms = typeof x.ts === "number" ? x.ts : (id ? threadMs(id) : null);
    return ms ? relTime(ms) : String(x.date ?? "");
  };
  const waitingThreads = useMemo(() => new Set(pending.map((d) => d.thread_id).filter(Boolean)), [pending]);

  const totalReceiptsCount = data?.receiptTotal ?? 0;
  const loadedExtraCount = data?.extraReceipts.length ?? 0;
  const hasMoreReceipts = totalReceiptsCount > loadedExtraCount;

  useEffect(() => {
    const q = receiptQuery.trim();
    if (!q || !hasMoreReceipts || !status || !hasV050Api(status, "receipts_export")) {
      setRemoteReceipts(null);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      call<{ receipts?: BriefJob[] }>(`/api/receipts?limit=200&q=${encodeURIComponent(q)}`).then((r) => {
        if (!cancelled && r.ok && Array.isArray(r.data.receipts)) {
          setRemoteReceipts(r.data.receipts.filter((x): x is BriefJob => !!x && typeof x === "object"));
        }
      });
    }, 180);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [receiptQuery, hasMoreReceipts, status]);

  const finished = useMemo(() => {
    const briefFin = (data?.brief.finished ?? [])
      .filter((x): x is BriefJob => !!x && typeof x === "object")
      .filter((x) => { const id = threadOf(x); return !id || !waitingThreads.has(id); });
    const extra = remoteReceipts ?? (data?.extraReceipts ?? []);
    if (!extra.length) return briefFin;
    const matchKey = (x: BriefJob) =>
      x.thread_id
        ? `t:${x.thread_id}|${x.task}`
        : x.id
          ? `id:${x.id}`
          : `tk:${x.task}|${x.ts || 0}`;
    const byKey = new Map<string, number>();
    const merged = briefFin.map((x, idx) => {
      byKey.set(matchKey(x), idx);
      return { ...x };
    });
    const seenIds = new Set(merged.map((x) => x.id).filter(Boolean));
    for (const r of extra) {
      const meta = r as BriefJob & { finished?: boolean; answered?: boolean };
      if (meta.finished === false || meta.answered === true) continue;
      const tid = threadOf(r);
      if (tid && waitingThreads.has(tid)) continue;
      const k = matchKey(r);
      const existingIdx = byKey.get(k);
      if (existingIdx !== undefined) {
        if (!merged[existingIdx].id && r.id) merged[existingIdx].id = r.id;
        if (!merged[existingIdx].ts && r.ts) merged[existingIdx].ts = r.ts;
        if (!merged[existingIdx].sha256 && r.sha256) merged[existingIdx].sha256 = r.sha256;
        if (!merged[existingIdx].usage && r.usage) merged[existingIdx].usage = r.usage;
        if (!merged[existingIdx].engagement_id && r.engagement_id) merged[existingIdx].engagement_id = r.engagement_id;
        if (r.id) seenIds.add(r.id);
        continue;
      }
      if (r.id && seenIds.has(r.id)) continue;
      if (r.id) seenIds.add(r.id);
      byKey.set(k, merged.length);
      merged.push(r);
    }
    return merged;
  }, [data?.brief.finished, data?.extraReceipts, remoteReceipts, threadOf, waitingThreads]);

  const stopped = useMemo(
    () => (data?.brief.stopped ?? []).filter((x): x is BriefJob => !!x && typeof x === "object"),
    [data?.brief.stopped]
  );

  useEffect(() => {
    if (!data || finished.length === 0) {
      setSummaryGate(null);
      return;
    }
    let cancelled = false;
    fetchSummaryPreflight().then((rep) => {
      if (!cancelled && rep) setSummaryGate(rep);
    });
    return () => {
      cancelled = true;
    };
  }, [data, finished.length, files.length]);

  useEffect(() => {
    setReceiptPage(0);
  }, [receiptQuery, onlyWithFiles, finished.length]);

  const filteredFinished = useMemo(() => {
    const q = receiptQuery.trim().toLowerCase();
    return finished.filter((x) => {
      const fList = normalizeFiles(x.files);
      if (onlyWithFiles && fList.length === 0) return false;
      if (!q) return true;
      return (
        String(x.task ?? "").toLowerCase().includes(q) ||
        String(x.summary ?? "").toLowerCase().includes(q) ||
        fList.some((f) => f.toLowerCase().includes(q))
      );
    });
  }, [finished, receiptQuery, onlyWithFiles]);

  const totalReceiptPages = Math.max(1, Math.ceil(filteredFinished.length / RECEIPTS_PAGE_SIZE));
  const safeReceiptPage = Math.min(receiptPage, totalReceiptPages - 1);
  const visibleFinished = filteredFinished.slice(
    safeReceiptPage * RECEIPTS_PAGE_SIZE,
    (safeReceiptPage + 1) * RECEIPTS_PAGE_SIZE
  );

  const allEvents = data?.todayEvents ?? [];
  const totalEventPages = Math.max(1, Math.ceil(allEvents.length / EVENTS_PAGE_SIZE));
  const safeEventPage = Math.min(eventsPage, totalEventPages - 1);
  const visibleEvents = allEvents.slice(safeEventPage * EVENTS_PAGE_SIZE, (safeEventPage + 1) * EVENTS_PAGE_SIZE);

  const pausedPipelines = useMemo(
    () =>
      (pipelineRuns ?? []).filter(
        (r) =>
          r.status === "paused_checkpoint" ||
          r.status === "interrupted" ||
          r.status === "failed"
      ),
    [pipelineRuns]
  );
  const runningPipelines = useMemo(
    () => (pipelineRuns ?? []).filter((r) => r.status === "running"),
    [pipelineRuns]
  );
  const pendingNonPipeline = useMemo(() => {
    const pausedIds = new Set(pausedPipelines.map((r) => r.run_id));
    const cpIds = new Set(pausedPipelines.map((r) => r.pending_checkpoint?.checkpoint_id).filter(Boolean));
    return pending.filter((d) => (!d.run_id || !pausedIds.has(d.run_id)) && !cpIds.has(d.id));
  }, [pending, pausedPipelines]);

  const lead = pendingNonPipeline[0];
  const leadCopy = lead ? decisionCopy(lead) : null;
  const n = finished.length;
  const w = pendingNonPipeline.length + pausedPipelines.length;
  const totalSavedFiles = useMemo(
    () => finished.reduce((acc, job) => acc + normalizeFiles(job.files).length, 0),
    [finished]
  );

  const useServerExport = hasV050Api(status, "receipts_export");

  const activeEng = useMemo(() => {
    if (activeEngagementId) {
      const matched = engagements.find((e) => e.id === activeEngagementId);
      if (matched) return matched;
      if (metering?.active_engagement?.id === activeEngagementId) return metering.active_engagement;
      return null;
    }
    return metering?.active_engagement || null;
  }, [activeEngagementId, engagements, metering?.active_engagement]);
  const activeCli = useMemo(() => {
    if (activeEng) {
      return (
        clients.find((c) => c.id === activeEng.client_id) ||
        (metering?.active_client?.id === activeEng.client_id ? metering.active_client : null)
      );
    }
    return metering?.active_client || null;
  }, [activeEng, clients, metering?.active_client]);
  const meteringTotals = metering?.totals;
  const activeEngRow = useMemo(
    () => (activeEng && metering?.by_engagement ? metering.by_engagement.find((r) => r.engagement_id === activeEng.id) || null : null),
    [activeEng, metering?.by_engagement]
  );
  const rawBudget = activeEng
    ? (activeEng.retainer_budget_usd ?? 0)
    : (meteringTotals?.retainer_budget_usd ?? 0);
  const retainerBudgetUsd = Number.isFinite(Number(rawBudget)) ? Math.max(0, Number(rawBudget)) : 0;
  const rawSpent = activeEng ? (activeEngRow?.cost_usd ?? 0) : (meteringTotals?.cost_usd ?? 0);
  const retainerSpentUsd = Number.isFinite(Number(rawSpent)) ? Math.max(0, Number(rawSpent)) : 0;
  const rawBurn =
    retainerBudgetUsd > 0
      ? activeEng
        ? (activeEngRow?.retainer_burn_pct ??
            (metering?.active_engagement?.id === activeEng.id ? meteringTotals?.retainer_burn_pct : undefined) ??
            (retainerSpentUsd / retainerBudgetUsd) * 100)
        : (meteringTotals?.retainer_burn_pct ?? (retainerSpentUsd / retainerBudgetUsd) * 100)
      : 0;
  const retainerBurnPct = retainerBudgetUsd > 0 && Number.isFinite(Number(rawBurn)) ? Math.max(0, Number(rawBurn)) : 0;
  const hourlyRateUsd = resolveEffectiveHourlyRate(activeEng, activeCli);

  const engagementEntries = useMemo(() => {
    if (!activeEng) return timeEntries;
    return timeEntries.filter((e) => e.engagement_id === activeEng.id);
  }, [timeEntries, activeEng]);

  const unbilledEntries = useMemo(
    () => engagementEntries.filter((e) => !e.invoiced_id),
    [engagementEntries]
  );

  const unbilledTotalUsd = useMemo(
    () => unbilledEntries.reduce((acc, e) => acc + (e.billable ? Number(e.amount_usd) || 0 : 0), 0),
    [unbilledEntries]
  );

  const engagementInvoices = useMemo(() => {
    if (!activeEng) return invoices;
    return invoices.filter((inv) => inv.engagement_id === activeEng.id);
  }, [invoices, activeEng]);

  const selectedInvoice = useMemo(
    () => engagementInvoices.find((inv) => inv.id === selectedInvoiceId) || engagementInvoices[0] || null,
    [engagementInvoices, selectedInvoiceId]
  );

  const openTimeModal = () => {
    setTimeKind("time");
    setTimeDesc("");
    setTimeHours("1.5");
    const defaultRate = hourlyRateUsd > 0 ? hourlyRateUsd : 200;
    setTimeRate(String(defaultRate));
    setTimeAmount(String(Math.round(1.5 * defaultRate * 100) / 100));
    setTimeBillable(true);
    setTimeErr("");
    setTimeModalOpen(true);
    if (activeEng && onLoadTimeEntries) {
      void onLoadTimeEntries({ engagement_id: activeEng.id, force: true });
    }
  };

  const openInvoicesDrawer = () => {
    setInvoicesOpen(true);
    if (activeEng && onLoadInvoices) {
      void onLoadInvoices({ engagement_id: activeEng.id, force: true });
    }
    if (activeEng && onLoadTimeEntries) {
      void onLoadTimeEntries({ engagement_id: activeEng.id, force: true });
    }
  };

  const handleScopedHandoffExport = async () => {
    if (scopedHandoffBusy) return;
    setScopedHandoffBusy(true);
    try {
      const label = [activeCli?.name, activeEng?.name].filter(Boolean).join(" - ") || "Client Handoff";
      const res = await exportHandoffBundleZip({
        title: label,
        client_id: activeEng?.client_id || activeCli?.id,
        engagement_id: activeEng?.id,
      });
      if (!res.ok) {
        toast.error(res.error);
        return;
      }
      toast.success(
        res.sha256
          ? `Exported scoped handoff ZIP (sha256: ${res.sha256.slice(0, 12)}...)`
          : "Exported scoped handoff ZIP"
      );
    } finally {
      setScopedHandoffBusy(false);
    }
  };

  const handleCreateTimeEntry = async () => {
    if (!onAddTimeEntry || savingTime) return;
    if (!activeEng) {
      setTimeErr("Select an engagement first.");
      return;
    }
    const desc = timeDesc.trim();
    if (!desc) {
      setTimeErr("Enter a description for this entry.");
      return;
    }
    const hours = Number(timeHours);
    const rate = Number(timeRate);
    const amount = Number(timeAmount);
    if (timeKind === "time" && (!Number.isFinite(hours) || hours <= 0)) {
      setTimeErr("Hours must be greater than 0.");
      return;
    }
    if (timeKind !== "time" && (!Number.isFinite(amount) || amount === 0)) {
      setTimeErr("Enter a non-zero dollar amount.");
      return;
    }
    setTimeErr("");
    setSavingTime(true);
    try {
      const computedAmount = timeKind === "time" ? Math.round(hours * (Number.isFinite(rate) ? rate : 0) * 100) / 100 : amount;
      const created = await onAddTimeEntry({
        client_id: activeEng.client_id,
        engagement_id: activeEng.id,
        kind: timeKind,
        description: desc,
        hours: timeKind === "time" ? hours : 0,
        rate_usd: timeKind === "time" && Number.isFinite(rate) ? rate : 0,
        amount_usd: computedAmount,
        billable: timeBillable,
      });
      if (created) {
        setTimeDesc("");
      }
    } finally {
      setSavingTime(false);
    }
  };

  const handleGenerateInvoiceSubmit = async () => {
    if (!onGenerateInvoice || !activeEng || generatingInv) return;
    const period = invPeriodLabel.trim() || new Date().toISOString().slice(0, 7);
    const formats: Array<"pdf" | "xlsx" | "json"> = invFormats.length > 0 ? invFormats : ["json"];
    setGeneratingInv(true);
    try {
      const inv = await onGenerateInvoice({
        engagement_id: activeEng.id,
        period_label: period,
        include_llm_cost: invIncludeLlm,
        include_time_entries: invIncludeTime,
        export_formats: formats,
      });
      if (inv) {
        setSelectedInvoiceId(inv.id);
      }
    } finally {
      setGeneratingInv(false);
    }
  };

  const handleRowReceiptExport = async (job: BriefJob, idx: number, format: "md" | "json") => {
    const id = job.id || job.thread_id || `receipt-${idx}`;
    const res = await exportReceiptDownload(
      {
        id,
        task: String(job.task ?? "Completed task"),
        person: personName || "Active employee",
        status: "finished",
        summary: String(job.summary ?? ""),
        ts: typeof job.ts === "number" ? job.ts : Date.now(),
        files: normalizeFiles(job.files),
      },
      format,
      useServerExport && Boolean(job.id || job.thread_id)
    );
    toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
  };

  const sideColumn = (
    <PaperSide
      people={people}
      teamError={teamError}
      active={activePerson}
      onSwitch={onSwitchPerson}
      offers={data ? data.offers.slice().reverse().filter((o) => o.status === "pending") : null}
      learned={data ? (Array.isArray(data.brief.learned_recently) ? data.brief.learned_recently : []) : null}
      memoryHealth={memoryHealth}
      onHealMemory={onHealMemory}
      files={files}
      failed={failed}
      onAct={act}
      onOpenFile={onOpenFile}
      onOpenFilesPanel={onOpenFilesPanel}
      onUploadMany={onUploadMany}
    />
  );

  return (
    <div
      className="relative mx-auto w-full max-w-[1160px] px-4 pb-14 pt-5 md:px-6"
      onDragOver={(e) => {
        if (e.dataTransfer?.types?.includes("Files")) {
          e.preventDefault();
          setDragOver(true);
        }
      }}
      onDragLeave={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setDragOver(false);
      }}
      onDrop={async (e) => {
        if (!e.dataTransfer?.files?.length) return;
        e.preventDefault();
        setDragOver(false);
        await onUploadMany(e.dataTransfer.files);
      }}
    >
      {dragOver && (
        <div
          id="today-drop-overlay"
          role="status"
          className="pointer-events-none fixed inset-4 z-30 flex flex-col items-center justify-center gap-2 rounded-panel border-2 border-dashed border-blue bg-blue-soft p-6 text-center text-ink shadow-lift"
        >
          <Upload className="size-7 text-ink" strokeWidth={1.5} aria-hidden />
          <p className="text-base font-medium text-ink">Drop files onto Today to upload to your workspace</p>
          <p className="mono text-xs text-ink-2">CSV, Markdown, JSON, TXT, TSV, LOG (multi-file supported)</p>
        </div>
      )}

      <div className="grid grid-cols-[minmax(0,1fr)] gap-8 xl:grid-cols-[minmax(0,1fr)_312px]">
        <main className="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-7" aria-label="Today's paper">
          {failed && (
            <div role="alert" className="rounded-entry border border-hairline bg-surface p-5">
              <h1 className="text-title font-medium text-ink">Today&apos;s paper could not be loaded</h1>
              <p className="mt-1 text-sm text-ink-2">Check that Ergon is running, then try again.</p>
              <Button className="mt-3 max-md:min-h-11" variant="outline" onClick={load}>Retry</Button>
            </div>
          )}

          {!failed && !data && (
            <div aria-busy="true" className="grid gap-4">
              <div className="h-12 w-4/5 rounded-sm bg-board" />
              <div className="h-20 rounded-card bg-board" />
              <div className="h-32 rounded-card bg-board" />
              <span className="sr-only">Loading today&apos;s paper</span>
            </div>
          )}

          {data && (
            <>
              <header className="grid gap-3 border-b border-hairline pb-5">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <p id="dateline" className="text-sm font-medium text-ink-2">
                    {dateLabel}{personName ? `, for ${personName === "You" ? "you" : personName}` : ""}
                  </p>
                  <div className="flex flex-wrap items-center gap-2">
                    {onExportAuditBundle && (
                      <Button
                        id="export-audit-bundle-btn"
                        type="button"
                        variant="outline"
                        size="sm"
                        className="max-md:min-h-11"
                        disabled={exportingAudit}
                        onClick={async () => {
                          if (exportAuditLockRef.current) return;
                          exportAuditLockRef.current = true;
                          setExportingAudit(true);
                          try {
                            await onExportAuditBundle("json");
                          } finally {
                            exportAuditLockRef.current = false;
                            setExportingAudit(false);
                          }
                        }}
                      >
                        <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                        {exportingAudit ? "Exporting…" : "Export Audit Bundle"}
                      </Button>
                    )}
                    <Button variant="outline" size="sm" className="max-md:min-h-11" data-answers="3" onClick={onAsk}>
                      Ask your agent for something
                    </Button>
                  </div>
                </div>
                <h1
                  id="brief-headline"
                  data-answers="1"
                  className="text-[28px] leading-[1.15] font-medium tracking-[-0.03em] text-ink [text-wrap:wrap] md:text-[36px]"
                >
                  {n ? (
                    <span>{n} {n === 1 ? "thing" : "things"} finished, </span>
                  ) : w ? (
                    <span>Nothing finished yet, </span>
                  ) : (
                    <span>Nothing finished yet.</span>
                  )}
                  {w ? (
                    <span>{w} {w === 1 ? "waits" : "wait"} on you.</span>
                  ) : n ? (
                    <span>nothing waits on you.</span>
                  ) : null}
                </h1>
                <p className="max-w-[64ch] text-ui leading-6 text-ink-2">
                  {lead
                    ? `The decision below is the only item parked on you${w > 1 ? " first" : " today"}. Autonomous tasks finished are filed underneath with their workspace files and SHA-256 receipts.`
                    : n
                      ? "Your agent has nothing waiting on your approval. Completed work and workspace files are filed below."
                      : "Describe a task in plain words or drop files onto this desk. Your agent runs real workspace tools and asks here before deleting anything."}
                </p>
                <div className="mono flex flex-wrap items-center gap-x-4 gap-y-1 pt-1 text-xs text-ink-2">
                  <span className="inline-flex items-center gap-1.5 text-ink">
                    <span aria-hidden className="size-2 rounded-full bg-dot-green" />
                    {n} finished
                  </span>
                  <span className="inline-flex items-center gap-1.5 text-ink">
                    <span aria-hidden className="size-2 rounded-full bg-dot-amber" />
                    {w} waiting
                  </span>
                  {stopped.length > 0 && (
                    <span className="inline-flex items-center gap-1.5 text-ink">
                      <span aria-hidden className="size-2 rounded-full bg-dot-red" />
                      {stopped.length} stopped
                    </span>
                  )}
                  <span>{totalSavedFiles} {totalSavedFiles === 1 ? "file" : "files"} saved</span>
                </div>
              </header>

              {(metering || liveRunUsage) && (
                <section
                  id="retainer-roi-card"
                  aria-labelledby="retainer-roi-h"
                  className="grid gap-3.5 rounded-card border border-hairline bg-surface p-4 sm:p-5"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex min-w-0 flex-wrap items-center gap-2">
                      <TrendingUp className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                      <h2 id="retainer-roi-h" className="text-base font-medium text-ink">
                        Retainer burn and est. billable value
                      </h2>
                      {(activeCli || activeEng) && (
                        <span className="inline-flex items-center gap-1 rounded-chip bg-board px-2 py-0.5 text-xs font-medium text-ink-2">
                          <Briefcase className="size-3 text-icon" strokeWidth={1.5} aria-hidden />
                          <span>
                            {[activeCli?.name, activeEng?.name].filter(Boolean).join(" / ")}
                          </span>
                        </span>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      {liveRunUsage && (
                        <Pill
                          id="live-usage-pill"
                          variant="secondary"
                          className="mono bg-blue-soft px-2 py-0.5 text-xs font-medium text-ink"
                        >
                          Live run: {liveRunUsage.total_tokens.toLocaleString()} tok, {formatUsd(liveRunUsage.cost_usd)}
                        </Pill>
                      )}
                      {onOpenWorkspaceSwitcher && (
                        <Button
                          id="retainer-switch-engagement-btn"
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={onOpenWorkspaceSwitcher}
                        >
                          {activeEng ? "Switch engagement" : "Select engagement"}
                        </Button>
                      )}
                    </div>
                  </div>

                  {retainerBudgetUsd > 0 ? (
                    <div className="grid gap-1.5">
                      <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
                        <span className="font-medium text-ink">
                          Engagement retainer burn
                          {activeEng?.retainer_hours ? ` (${activeEng.retainer_hours}h cap)` : ""}
                        </span>
                        <span id="retainer-burn-label" className="mono text-ink-2">
                          {formatUsd(retainerSpentUsd)} of {formatUsd(retainerBudgetUsd)} ({retainerBurnPct.toFixed(1)}%)
                        </span>
                      </div>
                      <div
                        role="progressbar"
                        aria-label="Engagement retainer burn"
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-valuenow={Math.min(100, Math.round(retainerBurnPct))}
                        className="h-2 w-full overflow-hidden rounded-chip bg-track"
                      >
                        <div
                          id="retainer-burn-bar"
                          className={cn(
                            "h-full rounded-chip",
                            retainerBurnPct >= 100
                              ? "bg-dot-red"
                              : retainerBurnPct >= 80
                                ? "bg-dot-amber"
                                : "bg-blue"
                          )}
                          style={{ width: `${Math.min(100, Math.max(0, retainerBurnPct))}%` }}
                        />
                      </div>
                    </div>
                  ) : (
                    <p
                      id="retainer-empty-state"
                      className="rounded-sm border border-hairline bg-board/40 px-3 py-2 text-xs text-ink-2"
                    >
                      {activeEng
                        ? "No retainer cap set ($0 or uncapped). Tracking token spend and est. billable value."
                        : "No engagement retainer selected. Tracking workspace usage totals."}
                    </p>
                  )}

                  <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                    <div className="flex flex-col gap-0.5 rounded-sm bg-board/60 p-3">
                      <span className="text-xs text-ink-2">LLM tokens</span>
                      <span id="metering-tokens-val" className="mono text-base font-medium text-ink">
                        {(meteringTotals?.total_tokens ?? 0).toLocaleString()}
                      </span>
                      <span className="mono text-xs text-ink-2">
                        {(meteringTotals?.prompt_tokens ?? 0).toLocaleString()} in, {(meteringTotals?.completion_tokens ?? 0).toLocaleString()} out
                      </span>
                    </div>
                    <div className="flex flex-col gap-0.5 rounded-sm bg-board/60 p-3">
                      <span className="text-xs text-ink-2">Model spend</span>
                      <span id="metering-cost-val" className="mono text-base font-medium text-ink">
                        {formatUsd(meteringTotals?.cost_usd ?? 0)}
                      </span>
                      <span className="mono text-xs text-ink-2">
                        {meteringTotals?.runs ?? 0} {(meteringTotals?.runs ?? 0) === 1 ? "metered run" : "metered runs"}
                      </span>
                    </div>
                    <div
                      className="flex flex-col gap-0.5 rounded-sm bg-board/60 p-3"
                      title={FORMULA_ESTIMATE_TOOLTIP}
                    >
                      <span className="text-xs text-ink-2">Est. minutes saved</span>
                      <span
                        id="metering-time-val"
                        className="mono text-base font-medium text-ink"
                        title={FORMULA_ESTIMATE_TOOLTIP}
                      >
                        ~{formatMinutesSaved(meteringTotals?.minutes_saved ?? 0)} (est.)
                      </span>
                      <span className="text-xs text-ink-2">
                        Formula estimate — not yet calibrated to stopwatch timings
                      </span>
                    </div>
                    <div
                      className="flex flex-col gap-0.5 rounded-sm bg-board/60 p-3"
                      title={FORMULA_ESTIMATE_TOOLTIP}
                    >
                      <span className="text-xs text-ink-2">Est. billable value</span>
                      <span
                        id="metering-roi-val"
                        className="mono text-base font-medium text-ink"
                        title={FORMULA_ESTIMATE_TOOLTIP}
                      >
                        ~{formatUsd(meteringTotals?.billable_value_usd ?? 0)} (est.)
                      </span>
                      <span className="mono text-xs text-ink-2">
                        {hourlyRateUsd > 0 ? `At $${hourlyRateUsd}/hr (est.)` : "No rate set ($0/hr)"}
                      </span>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-2.5">
                    <div id="retainer-billing-summary" className="mono flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-2">
                      <span>
                        Unbilled: {unbilledEntries.length} ({formatUsd(unbilledTotalUsd)})
                      </span>
                      <span>
                        Invoices: {engagementInvoices.length}
                      </span>
                    </div>
                    <div className="flex flex-wrap items-center gap-1.5">
                      {onAddTimeEntry && (
                        <Button
                          id="retainer-add-time-btn"
                          type="button"
                          size="xs"
                          variant="outline"
                          disabled={!activeEng}
                          className="max-md:min-h-11 gap-1"
                          onClick={openTimeModal}
                        >
                          <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
                          Log time / expense
                        </Button>
                      )}
                      <Button
                        id="retainer-scoped-handoff-btn"
                        type="button"
                        size="xs"
                        variant="outline"
                        disabled={!activeEng || scopedHandoffBusy}
                        className="max-md:min-h-11 gap-1"
                        onClick={() => void handleScopedHandoffExport()}
                      >
                        <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                        {scopedHandoffBusy ? "Exporting…" : "Scoped Handoff"}
                      </Button>
                      {onGenerateInvoice && (
                        <Button
                          id="retainer-invoices-btn"
                          type="button"
                          size="xs"
                          variant="default"
                          disabled={!activeEng}
                          className="max-md:min-h-11 gap-1"
                          onClick={openInvoicesDrawer}
                        >
                          <ReceiptText className="size-3.5" strokeWidth={1.5} aria-hidden />
                          Invoices &amp; Billing
                        </Button>
                      )}
                    </div>
                  </div>

                  {metering && (metering.by_engagement.length > 0 || metering.by_model.length > 0) && (
                    <Details label="Engagement and model breakdown">
                      <div className="grid gap-3 pt-1 text-xs">
                        {metering.by_engagement.length > 0 && (
                          <div>
                            <p className="mb-1 font-medium text-ink">By engagement</p>
                            <ul className="divide-y divide-hairline">
                              {metering.by_engagement.slice(0, 6).map((row) => (
                                <li key={row.engagement_id} className="flex flex-wrap items-center justify-between gap-2 py-1.5">
                                  <span className="min-w-0 truncate text-ink">
                                    {row.client_name ? `${row.client_name} / ` : ""}{row.engagement_name || row.engagement_id}
                                  </span>
                                  <span className="mono text-ink-2" title={FORMULA_ESTIMATE_TOOLTIP}>
                                    {row.total_tokens.toLocaleString()} tok, {formatUsd(row.cost_usd)}, ~{formatUsd(row.billable_value_usd)} est. value
                                  </span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                        {metering.by_model.length > 0 && (
                          <div>
                            <p className="mb-1 font-medium text-ink">By model</p>
                            <ul className="divide-y divide-hairline">
                              {metering.by_model.slice(0, 6).map((m) => (
                                <li key={m.model_id} className="flex flex-wrap items-center justify-between gap-2 py-1.5">
                                  <span className="mono truncate text-ink">{m.model_id}</span>
                                  <span className="mono text-ink-2">
                                    {m.runs} {m.runs === 1 ? "run" : "runs"}, {m.total_tokens.toLocaleString()} tok, {formatUsd(m.cost_usd)}
                                  </span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    </Details>
                  )}
                </section>
              )}

              {busy && (
                <LiveRunTimeline
                  steps={liveTimeline}
                  elapsedMs={liveElapsedMs}
                  startedMs={runStartedMs}
                  streamMode={streamMode}
                  cancelling={cancelling}
                  onCancel={onCancel}
                />
              )}

              {onResumePipeline && onCancelPipeline && runningPipelines.length > 0 && (
                <PipelineRunStrip
                  runs={runningPipelines}
                  onResume={onResumePipeline}
                  onRetry={onRetryPipeline}
                  onCancel={onCancelPipeline}
                  onOpenFile={onOpenFile}
                />
              )}

              {/* Waiting on you Approval Queue */}
              <div id="decisions-list" className="grid gap-3">
                {onResumePipeline && onCancelPipeline && pausedPipelines.map((pr) => (
                  <CheckpointCard
                    key={pr.run_id}
                    run={pr}
                    onResume={onResumePipeline}
                    onRetry={onRetryPipeline}
                    onCancel={onCancelPipeline}
                    onOpenFile={onOpenFile}
                  />
                ))}
                {lead ? (
                  <section
                    key={lead.id}
                    aria-labelledby="lead-h"
                    className="grid gap-3.5 rounded-card border border-hairline bg-blue-soft p-5"
                  >
                      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                        <h2 id="lead-h" className="flex min-w-0 items-start gap-2 text-[22px] leading-snug font-medium tracking-[-0.025em] text-ink [overflow-wrap:anywhere]">
                          {lead.kind === "connector_delivery" ? (
                            <Send
                              className="mt-1 size-5 shrink-0 text-dot-amber"
                              strokeWidth={1.5}
                              aria-label="Outbound delivery"
                            />
                          ) : lead.kind === "mcp_tool_call" ? (
                            <Wrench
                              className="mt-1 size-5 shrink-0 text-dot-amber"
                              strokeWidth={1.5}
                              aria-label="MCP tool call"
                            />
                          ) : lead.kind === "budget_gate" ? (
                            <Wallet
                              className="mt-1 size-5 shrink-0 text-dot-amber"
                              strokeWidth={1.5}
                              aria-label="Retainer budget gate"
                            />
                          ) : lead.kind === "tool_offer" ? (
                            <Sparkles
                              className="mt-1 size-5 shrink-0 text-dot-blue"
                              strokeWidth={1.5}
                              aria-label="Would this help?"
                            />
                          ) : leadCopy?.security ? (
                            <ShieldAlert
                              className={cn("mt-1 size-5 shrink-0", lead.severity === "critical" ? "text-dot-red" : "text-dot-amber")}
                              strokeWidth={1.5}
                              aria-label="Security"
                            />
                          ) : null}
                          <span className="min-w-0">{String(lead.summary ?? "Decision waiting on you")}</span>
                        </h2>
                        <WaitingPill />
                      </div>
                      <p className="max-w-[60ch] whitespace-pre-line text-sm leading-6 text-ink-2">{leadCopy?.body}</p>
                      {lead.kind === "budget_gate" && (
                        <div data-budget-gate={lead.id} className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-surface px-3 py-2.5 text-xs text-ink-2">
                          <div className="flex flex-wrap items-center justify-between gap-2">
                            <span className="flex items-center gap-1.5 font-medium text-ink">
                              <Coins className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                              <span>
                                {[lead.client_name, lead.engagement_name].filter(Boolean).join(" / ") || "Active engagement"}
                              </span>
                            </span>
                            {typeof lead.retainer_budget_usd === "number" && lead.retainer_budget_usd > 0 && (
                              <span className="mono text-ink">
                                Spent {formatUsd(lead.spent_usd ?? 0)} of {formatUsd(lead.retainer_budget_usd)} cap
                              </span>
                            )}
                          </div>
                          {typeof lead.estimated_cost_usd === "number" && (
                            <span className="mono text-ink-2">
                              Estimated next run cost: {formatUsd(lead.estimated_cost_usd)}
                            </span>
                          )}
                        </div>
                      )}
                      {lead.kind === "connector_delivery" && (lead.connector_name || lead.url || lead.target_url || lead.payload_preview) && (
                        <div data-connector-consequence={lead.id} className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-surface px-3 py-2.5 text-xs text-ink-2">
                          <div className="flex flex-wrap items-center gap-2">
                            {lead.connector_name && <span className="font-medium text-ink">{lead.connector_name}</span>}
                            {(lead.url || lead.target_url) && (
                              <span className="mono break-all text-ink-2">
                                {lead.method ? `${lead.method} ` : ""}{lead.url || lead.target_url}
                              </span>
                            )}
                          </div>
                          {lead.payload_preview && (
                            <pre className="mono overflow-x-auto whitespace-pre-wrap break-words text-xs text-ink">
                              {lead.payload_preview}
                            </pre>
                          )}
                          {lead.path && (
                            <div>
                              <Button
                                type="button"
                                size="xs"
                                variant="ghost"
                                className="max-md:min-h-11 px-0 text-xs text-ink hover:underline"
                                onClick={() => onOpenFile(lead.path!)}
                              >
                                Inspect {lead.path}
                              </Button>
                            </div>
                          )}
                        </div>
                      )}
                      {lead.kind === "tool_offer" && (
                        <div data-tool-offer={lead.id} className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-surface px-3 py-2.5 text-xs text-ink-2">
                          <div className="flex flex-wrap items-center justify-between gap-2">
                            <span className="font-medium text-ink">Would this help?</span>
                            <span className="mono rounded-chip bg-pill-active-bg px-2 py-0.5 text-xs font-medium text-pill-active-fg">
                              {typeof lead.test_report?.passed === "number" && typeof lead.test_report?.total === "number"
                                ? `${lead.test_report.passed}/${lead.test_report.total} sandbox tests passed`
                                : "Sandbox tests passed"}
                            </span>
                          </div>
                          {lead.description && <p className="text-sm text-ink-2">{lead.description}</p>}
                          {Array.isArray(lead.params) && lead.params.length > 0 && (
                            <span className="mono text-xs text-ink-2">Parameters: {lead.params.join(", ")}</span>
                          )}
                        </div>
                      )}
                      {lead.kind === "mcp_tool_call" && (lead.server_name || lead.connector_name || lead.server_id || lead.tool_name || lead.arguments_preview || lead.args_preview || lead.payload_preview) && (
                        <div data-mcp-consequence={lead.id} className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-surface px-3 py-2.5 text-xs text-ink-2">
                          <div className="flex flex-wrap items-center gap-2">
                            {(lead.server_name || lead.connector_name || lead.server_id) && (
                              <span className="font-medium text-ink">
                                {lead.server_name || lead.connector_name || lead.server_id}
                              </span>
                            )}
                            {lead.tool_name && (
                              <code className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink">
                                {lead.tool_name}
                              </code>
                            )}
                          </div>
                          {(lead.arguments_preview || lead.args_preview || lead.payload_preview) && (
                            <pre className="mono overflow-x-auto whitespace-pre-wrap break-words text-xs text-ink">
                              {lead.arguments_preview || lead.args_preview || lead.payload_preview}
                            </pre>
                          )}
                        </div>
                      )}
                      <div className="flex flex-wrap gap-2">
                        <Button
                          className="min-h-11 px-4"
                          disabled={busyDecisionId === lead.id}
                          onClick={() => void handleDecisionAct(lead.id, "approve", leadCopy?.approvedToast ?? "Approved and done.")}
                        >
                          {leadCopy?.approve ?? "Approve"}
                        </Button>
                        {leadCopy?.deny !== null && (
                          <Button
                            className="min-h-11 px-4"
                            variant="outline"
                            disabled={busyDecisionId === lead.id}
                            onClick={() => void handleDecisionAct(lead.id, "deny", leadCopy?.deniedToast ?? "You said no; kept safe.")}
                          >
                            {leadCopy?.deny ?? "Decline"}
                          </Button>
                        )}
                      </div>
                      {pendingNonPipeline.length > 1 && (
                        <p className="text-xs text-ink-2">
                          <span className="mono">{pendingNonPipeline.length - 1}</span> more {pendingNonPipeline.length - 1 === 1 ? "decision waits" : "decisions wait"} after this one.
                        </p>
                      )}
                    </section>
                  ) : pausedPipelines.length === 0 && n > 0 ? (
                    <p
                      key="quiet"
                      className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2"
                    >
                      Nothing waits on you. Your agent asks here before it deletes anything.
                    </p>
                  ) : null}
                {settled.length > 0 && (
                  <div className="mt-3">
                    <Details label={`Recently settled (${settled.length})`}>
                      <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface px-3.5">
                        {settled.slice(0, 5).map((s) => (
                          <li key={s.id} className="flex flex-wrap items-center justify-between gap-2 py-2.5 text-sm">
                            <span className="min-w-0 truncate font-medium text-ink">{String(s.summary ?? "")}</span>
                            <span className="text-xs text-ink-2">{String(s.outcome ?? s.status ?? "")}</span>
                          </li>
                        ))}
                      </ul>
                    </Details>
                  </div>
                )}
              </div>

              {onLaunchStarter && starterPlaybooks.length > 0 && (
                <StarterPlaybooksBar
                  compact
                  starters={starterPlaybooks}
                  onLaunch={async (id, runNow) => {
                    const res = await onLaunchStarter(id, runNow);
                    if (res?.ok) load();
                    return res;
                  }}
                  onOpenPlaybooks={onOpenPlaybooks}
                />
              )}

              {/* Virtualized / Paginated Finished Work & Receipts Ledger */}
              {n > 0 && (
                <section aria-labelledby="finished-h" className="grid gap-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <h2 id="finished-h" className="text-title font-medium text-ink">Finished</h2>
                    <div className="flex flex-wrap items-center gap-2">
                      {hasMoreReceipts && (
                        <Button
                          type="button"
                          size="xs"
                          variant="ghost"
                          disabled={loadingAll}
                          className="text-xs text-ink-2 hover:text-ink"
                          onClick={async () => {
                            setLoadingAll(true);
                            try {
                              await loadAllReceipts();
                            } finally {
                              setLoadingAll(false);
                            }
                          }}
                        >
                          {loadingAll ? "Loading…" : `Load all (${totalReceiptsCount})`}
                        </Button>
                      )}
                      {summaryGate && (
                        <Pill
                          id="summary-citation-gate"
                          variant="secondary"
                          className={cn(
                            "mono px-2 py-0.5 text-xs font-medium",
                            (summaryGate.citations_stale ?? 0) + (summaryGate.citations_broken ?? 0) > 0
                              ? "bg-pill-warn-bg text-pill-warn-fg"
                              : "bg-pill-active-bg text-pill-active-fg"
                          )}
                          title={
                            (summaryGate.citations_checked ?? 0) > 0
                              ? `${summaryGate.citations_verified}/${summaryGate.citations_checked} citations verified across summarized runs`
                              : "Deliverables SHA-256 verified"
                          }
                        >
                          {(summaryGate.citations_stale ?? 0) + (summaryGate.citations_broken ?? 0) > 0
                            ? `${summaryGate.citations_verified}/${summaryGate.citations_checked} Verified · ${(summaryGate.citations_stale ?? 0) + (summaryGate.citations_broken ?? 0)} Stale`
                            : (summaryGate.citations_checked ?? 0) > 0
                              ? `${summaryGate.citations_verified}/${summaryGate.citations_checked} Verified`
                              : "Verified"}
                        </Pill>
                      )}
                      <Button
                        id="export-summary-md-btn"
                        type="button"
                        size="xs"
                        variant="outline"
                        disabled={exportingSummary !== null}
                        className="max-md:min-h-11 gap-1"
                        onClick={() => handleSummaryExport("md")}
                      >
                        <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                        <span>{exportingSummary === "md" ? "Packing…" : "Client Summary (.md)"}</span>
                      </Button>
                      <Button
                        id="export-summary-html-btn"
                        type="button"
                        size="xs"
                        variant="outline"
                        disabled={exportingSummary !== null}
                        className="max-md:min-h-11 gap-1"
                        onClick={() => handleSummaryExport("html")}
                      >
                        <Printer className="size-3.5" strokeWidth={1.5} aria-hidden />
                        <span>{exportingSummary === "html" ? "Rendering…" : "Summary (.html)"}</span>
                      </Button>
                      <span className="mono text-xs text-ink-2">
                        {filteredFinished.length === n
                          ? `${n} ${n === 1 ? "receipt" : "receipts"}`
                          : `${filteredFinished.length} of ${n} ${n === 1 ? "receipt" : "receipts"}`}
                      </span>
                    </div>
                  </div>

                  {n >= 3 && (
                    <div className="flex flex-wrap items-center gap-2">
                      <InputGroup className="min-w-[200px] flex-1 rounded-sm bg-surface [&_input]:max-md:min-h-11">
                        <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
                          <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
                        </InputGroupAddon>
                        <InputGroupInput
                          id="finished-search"
                          type="search"
                          aria-label="Filter finished receipts"
                          placeholder="Filter receipts…"
                          value={receiptQuery}
                          onChange={(e) => setReceiptQuery(e.target.value)}
                          className="text-sm max-md:min-h-11"
                        />
                        {receiptQuery && (
                          <InputGroupAddon align="inline-end" className="pr-1.5">
                            <Button
                              type="button"
                              size="icon-xs"
                              variant="ghost"
                              className="max-md:min-h-11 max-md:min-w-11"
                              aria-label="Clear receipt filter"
                              onClick={() => setReceiptQuery("")}
                            >
                              <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                            </Button>
                          </InputGroupAddon>
                        )}
                      </InputGroup>
                      <Button
                        type="button"
                        size="sm"
                        variant={onlyWithFiles ? "default" : "outline"}
                        aria-pressed={onlyWithFiles}
                        className="max-md:min-h-11"
                        onClick={() => setOnlyWithFiles((v) => !v)}
                      >
                        With saved files
                      </Button>
                    </div>
                  )}

                  {visibleFinished.length === 0 ? (
                    <p className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2">
                      No finished receipts match your filter.
                    </p>
                  ) : (
                    <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                      {visibleFinished.map((x, i) => {
                        const globalIdx = safeReceiptPage * RECEIPTS_PAGE_SIZE + i;
                        const id = threadOf(x);
                        const jobFiles = normalizeFiles(x.files);
                        const taskTitle = String(x.task ?? "Completed task");
                        return (
                          <li
                            key={(x.id ?? x.thread_id ?? taskTitle) + "-" + globalIdx}
                            className="flex flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3.5"
                          >
                            <span className="mono w-full shrink-0 text-xs text-ink-2 sm:w-[96px]">{when(x)}</span>
                            <div className="min-w-[13rem] flex-1">
                              <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                              {x.usage && x.usage.total_tokens > 0 && (
                                <p className="mono mt-0.5 text-xs text-ink-2">
                                  {x.usage.total_tokens.toLocaleString()} tokens, {formatUsd(x.usage.cost_usd)}
                                  {typeof x.usage.estimated_minutes_saved === "number" && x.usage.estimated_minutes_saved > 0 ? (
                                    <span title={FORMULA_ESTIMATE_TOOLTIP}>
                                      {`, ~${formatMinutesSaved(x.usage.estimated_minutes_saved)} saved (est.)`}
                                    </span>
                                  ) : (
                                    ""
                                  )}
                                </p>
                              )}
                              <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1.5 text-xs text-ink-2">
                                {jobFiles.length ? (
                                  <>
                                    <span>Saved</span>
                                    {jobFiles.map((f, k) => (
                                      <Button
                                        key={f + k}
                                        size="xs"
                                        variant="outline"
                                        aria-label={`Open ${f}`}
                                        onClick={() => onOpenFile(f)}
                                        className="num max-md:min-h-11 max-w-full gap-1.5 rounded-chip border-hairline bg-board/60 px-2 text-xs font-normal text-ink hover:bg-board"
                                      >
                                        <FileGlyph name={f} />
                                        <span className="truncate">{f}</span>
                                      </Button>
                                    ))}
                                  </>
                                ) : (
                                  "Nothing saved to your workspace"
                                )}
                              </div>
                            </div>
                            <div className="ml-auto flex shrink-0 flex-wrap items-center justify-end gap-1">
                              {onSaveRoutine && taskTitle && (
                                <Button
                                  type="button"
                                  size="xs"
                                  variant="ghost"
                                  className="max-md:min-h-11 gap-1 text-xs text-ink-2 hover:text-ink"
                                  aria-label={`Promote ${taskTitle} to Playbook`}
                                  disabled={Boolean(savedRoutineKeys[x.id ?? `${globalIdx}`])}
                                  onClick={async () => {
                                    const key = x.id ?? `${globalIdx}`;
                                    const pb = extractPlaybookFromRun(taskTitle, [], jobFiles);
                                    const ok = await onSaveRoutine(pb.name, pb.prompt, "manual", {
                                      steps: pb.steps,
                                      expected_inputs: pb.expected_inputs,
                                      pack_deliverables: pb.pack_deliverables,
                                    });
                                    if (ok) setSavedRoutineKeys((prev) => ({ ...prev, [key]: true }));
                                  }}
                                >
                                  {savedRoutineKeys[x.id ?? `${globalIdx}`] ? (
                                    <Check className="size-3.5 text-dot-green" strokeWidth={1.75} aria-hidden />
                                  ) : (
                                    <Repeat className="size-3.5" strokeWidth={1.5} aria-hidden />
                                  )}
                                  <span>{savedRoutineKeys[x.id ?? `${globalIdx}`] ? "Saved" : "Playbook"}</span>
                                </Button>
                              )}
                              <Button
                                type="button"
                                size="xs"
                                variant="ghost"
                                className="max-md:min-h-11 gap-1 text-xs text-ink-2 hover:text-ink"
                                aria-label={`Export SHA-256 receipt for ${taskTitle}`}
                                onClick={() => handleRowReceiptExport(x, globalIdx, "md")}
                              >
                                <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                                <span>Receipt</span>
                              </Button>
                              {id && (
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  className="min-h-11"
                                  aria-label={`Open the conversation: ${taskTitle}`}
                                  onClick={() => onOpen(id)}
                                >
                                  Open
                                </Button>
                              )}
                            </div>
                          </li>
                        );
                      })}
                    </ol>
                  )}

                  {totalReceiptPages > 1 && (
                    <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
                      <span className="mono">
                        Showing {safeReceiptPage * RECEIPTS_PAGE_SIZE + 1} to {Math.min((safeReceiptPage + 1) * RECEIPTS_PAGE_SIZE, filteredFinished.length)} of {filteredFinished.length}
                      </span>
                      <div className="flex items-center gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          disabled={safeReceiptPage === 0}
                          onClick={() => setReceiptPage((p) => Math.max(0, p - 1))}
                        >
                          Prev
                        </Button>
                        <span className="mono px-1">
                          {safeReceiptPage + 1} / {totalReceiptPages}
                        </span>
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          disabled={safeReceiptPage >= totalReceiptPages - 1}
                          onClick={() => setReceiptPage((p) => Math.min(totalReceiptPages - 1, p + 1))}
                        >
                          Next
                        </Button>
                      </div>
                    </div>
                  )}
                </section>
              )}

              {/* Stopped / Did not finish */}
              {stopped.length > 0 && (
                <section aria-labelledby="stopped-h" className="grid gap-3">
                  <h2 id="stopped-h" className="text-title font-medium text-ink">Did not finish</h2>
                  <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                    {stopped.slice(0, 20).map((x, i) => {
                      const id = threadOf(x);
                      const taskTitle = String(x.task ?? "Stopped task");
                      return (
                        <li
                          key={(x.thread_id ?? taskTitle) + i}
                          className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 px-4 py-3.5 md:grid-cols-[104px_minmax(0,1fr)_auto]"
                        >
                          <span className="mono col-span-2 text-xs text-ink-2 md:col-span-1">{when(x)}</span>
                          <div className="min-w-0">
                            <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                            {x.summary && <p className="mt-0.5 text-sm leading-5 text-ink-2 [overflow-wrap:anywhere]">{String(x.summary)}</p>}
                          </div>
                          {id && (
                            <Button
                              size="sm"
                              variant="ghost"
                              className="min-h-11"
                              aria-label={`Open the conversation: ${taskTitle}`}
                              onClick={() => onOpen(id)}
                            >
                              Open
                            </Button>
                          )}
                        </li>
                      );
                    })}
                  </ol>
                </section>
              )}

              {/* Today so far (/api/today paginated chronological activity stream) */}
              <section aria-labelledby="today-stream-h" className="grid gap-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h2 id="today-stream-h" className="text-title font-medium text-ink">Today so far</h2>
                  <span className="mono text-xs text-ink-2">{allEvents.length} {allEvents.length === 1 ? "event" : "events"}</span>
                </div>
                {allEvents.length === 0 ? (
                  <p className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2">
                    No activity logged yet today. New tool runs, decisions, and capability offers appear here in order.
                  </p>
                ) : (
                  <>
                    <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                      {visibleEvents.map((ev, i) => {
                        const evFiles = normalizeFiles(ev.files);
                        const kind = String(ev.kind ?? "work");
                        const EventIcon = kind === "decision" ? Clock : kind === "offer" ? Sparkles : kind === "security" ? ShieldAlert : CheckCircle2;
                        const iconCls =
                          kind === "decision"
                            ? "text-dot-amber"
                            : kind === "offer"
                              ? "text-dot-clay"
                              : kind === "security"
                                ? ev.severity === "critical" ? "text-dot-red" : "text-dot-amber"
                                : "text-dot-green";
                        const timeStr = typeof ev.ts === "number" ? relTime(ev.ts) : "";
                        return (
                          <li key={String(ev.ts ?? i) + "-" + i} className="flex items-start gap-3 px-4 py-3">
                            {kind === "stopped" ? (
                              <XCircle className="mt-0.5 size-4 shrink-0 text-dot-red" strokeWidth={1.5} aria-hidden />
                            ) : (
                              <EventIcon className={cn("mt-0.5 size-4 shrink-0", iconCls)} strokeWidth={1.5} aria-hidden />
                            )}
                            <div className="min-w-0 flex-1">
                              <div className="flex flex-wrap items-baseline justify-between gap-2">
                                <p className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{String(ev.title ?? "Activity")}</p>
                                {timeStr && <span className="mono shrink-0 text-xs text-ink-3">{timeStr}</span>}
                              </div>
                              {ev.detail && <p className="mt-0.5 text-sm text-ink-2 [overflow-wrap:anywhere]">{String(ev.detail)}</p>}
                              {evFiles.length > 0 && (
                                <div className="mt-1.5 flex flex-wrap items-center gap-2">
                                  {evFiles.map((f, idx) => (
                                    <Button
                                      key={f + idx}
                                      size="xs"
                                      variant="outline"
                                      aria-label={`Open ${f}`}
                                      onClick={() => onOpenFile(f)}
                                      className="mono max-md:min-h-11 max-w-full gap-1 rounded-chip border-hairline bg-board px-2 text-xs font-normal text-ink"
                                    >
                                      <FileGlyph name={f} className="size-3.5" />
                                      <span className="truncate">{f}</span>
                                    </Button>
                                  ))}
                                </div>
                              )}
                            </div>
                          </li>
                        );
                      })}
                    </ol>

                    {totalEventPages > 1 && (
                      <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
                        <span className="mono">
                          Events {safeEventPage * EVENTS_PAGE_SIZE + 1} to {Math.min((safeEventPage + 1) * EVENTS_PAGE_SIZE, allEvents.length)} of {allEvents.length}
                        </span>
                        <div className="flex items-center gap-1.5">
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            className="max-md:min-h-11"
                            disabled={safeEventPage === 0}
                            onClick={() => setEventsPage((p) => Math.max(0, p - 1))}
                          >
                            Prev
                          </Button>
                          <span className="mono px-1">
                            {safeEventPage + 1} / {totalEventPages}
                          </span>
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            className="max-md:min-h-11"
                            disabled={safeEventPage >= totalEventPages - 1}
                            onClick={() => setEventsPage((p) => Math.min(totalEventPages - 1, p + 1))}
                          >
                            Next
                          </Button>
                        </div>
                      </div>
                    )}
                  </>
                )}
              </section>
            </>
          )}
        </main>

        <aside aria-label="Offers, memory and team">{sideColumn}</aside>
      </div>
      <PanelModal
        open={timeModalOpen}
        onOpenChange={setTimeModalOpen}
        panelId="time-entry-modal"
        title="Log billable time or expense"
        description={activeEng ? `Record billable work or pass-through expenses for ${activeEng.name}.` : "Select an engagement to log entries."}
        className="max-w-lg"
      >
        <div className="flex flex-col gap-4 pt-1">
          <div className="flex flex-col gap-3 rounded-sm border border-hairline bg-board/40 p-3">
            <div className="flex flex-wrap gap-1.5" role="group" aria-label="Entry kind">
              {(["time", "expense", "adjustment"] as const).map((k) => (
                <Button
                  key={k}
                  type="button"
                  size="xs"
                  data-time-kind={k}
                  variant={timeKind === k ? "default" : "outline"}
                  className="max-md:min-h-11 capitalize"
                  onClick={() => setTimeKind(k)}
                >
                  {k}
                </Button>
              ))}
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="time-entry-desc" className="text-xs font-medium text-ink">
                Description
              </label>
              <Input
                id="time-entry-desc"
                value={timeDesc}
                onChange={(e) => setTimeDesc(e.target.value)}
                placeholder="Client contract review and compliance memo"
                className="text-sm max-md:min-h-11"
              />
            </div>

            {timeKind === "time" ? (
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div className="flex flex-col gap-1">
                  <label htmlFor="time-entry-hours" className="text-xs font-medium text-ink">
                    Hours
                  </label>
                  <Input
                    id="time-entry-hours"
                    type="number"
                    min="0"
                    step="any"
                    value={timeHours}
                    onChange={(e) => setTimeHours(e.target.value)}
                    className="mono text-sm max-md:min-h-11"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="time-entry-rate" className="text-xs font-medium text-ink">
                    Rate (USD/hr)
                  </label>
                  <Input
                    id="time-entry-rate"
                    type="number"
                    min="0"
                    step="any"
                    value={timeRate}
                    onChange={(e) => setTimeRate(e.target.value)}
                    className="mono text-sm max-md:min-h-11"
                  />
                </div>
              </div>
            ) : (
              <div className="flex flex-col gap-1">
                <label htmlFor="time-entry-amount" className="text-xs font-medium text-ink">
                  Amount (USD)
                </label>
                <Input
                  id="time-entry-amount"
                  type="number"
                  step="any"
                  value={timeAmount}
                  onChange={(e) => setTimeAmount(e.target.value)}
                  className="mono text-sm max-md:min-h-11"
                />
              </div>
            )}

            <div className="flex flex-wrap items-center justify-between gap-2">
              <label className="inline-flex min-h-11 items-center gap-2 text-xs text-ink md:min-h-8">
                <input
                  id="time-entry-billable"
                  type="checkbox"
                  checked={timeBillable}
                  onChange={(e) => setTimeBillable(e.target.checked)}
                />
                <span>Billable to client invoice</span>
              </label>
              <Button id="time-entry-submit-btn" type="button" size="sm" disabled={savingTime} className="max-md:min-h-11" onClick={() => void handleCreateTimeEntry()}>
                {savingTime ? "Saving…" : "Add entry"}
              </Button>
            </div>
            {timeErr && (
              <p role="alert" className="text-xs text-destructive-foreground">
                {timeErr}
              </p>
            )}
          </div>

          <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between gap-2">
              <h3 className="text-xs font-medium text-ink">
                Logged entries ({engagementEntries.length})
              </h3>
              <span className="mono text-xs text-ink-2">
                Unbilled: {formatUsd(unbilledTotalUsd)}
              </span>
            </div>
            {engagementEntries.length === 0 ? (
              <p className="rounded-sm border border-hairline bg-board/40 px-3 py-2 text-xs text-ink-2">
                No manual time or expense entries recorded for this engagement yet.
              </p>
            ) : (
              <ul id="time-entries-list" className="divide-y divide-hairline rounded-sm border border-hairline bg-surface">
                {engagementEntries.map((entry) => (
                  <li key={entry.id} className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 text-xs">
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-medium text-ink">{entry.description}</p>
                      <p className="mono text-ink-2">
                        {entry.kind}
                        {entry.kind === "time" ? ` · ${entry.hours}h @ ${formatUsd(entry.rate_usd)}/hr` : ""}
                        {` · ${formatUsd(entry.amount_usd)}`}
                        {entry.invoiced_id ? " · invoiced" : entry.billable ? " · unbilled" : " · non-billable"}
                      </p>
                    </div>
                    {!entry.invoiced_id && onRemoveTimeEntry && (
                      <Button
                        type="button"
                        size="icon-xs"
                        variant="ghost"
                        aria-label={`Delete entry ${entry.description}`}
                        data-delete-time-entry={entry.id}
                        disabled={deletingEntryId === entry.id}
                        className="max-md:min-h-11 max-md:min-w-11"
                        onClick={async () => {
                          setDeletingEntryId(entry.id);
                          try {
                            await onRemoveTimeEntry(entry.id, activeEng?.id);
                          } finally {
                            setDeletingEntryId(null);
                          }
                        }}
                      >
                        <Trash2 className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                      </Button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </PanelModal>

      <PanelModal
        open={invoicesOpen}
        onOpenChange={setInvoicesOpen}
        panelId="invoices-billing-drawer"
        title="Invoices and billing"
        description={activeEng ? `Generate, preview, and settle client invoices for ${activeEng.name}.` : "Select an engagement first."}
        className="max-w-2xl"
      >
        <div className="flex flex-col gap-4 pt-1">
          <div
            id="invoice-generate-form"
            className="flex flex-col gap-3 rounded-sm border border-hairline bg-board/40 p-3"
          >
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <div className="flex flex-col gap-1">
                <label htmlFor="invoice-period-input" className="text-xs font-medium text-ink">
                  Billing period label
                </label>
                <Input
                  id="invoice-period-input"
                  value={invPeriodLabel}
                  onChange={(e) => setInvPeriodLabel(e.target.value)}
                  placeholder="2026-11"
                  className="mono text-xs max-md:min-h-11"
                />
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-ink">Export formats</span>
                <div className="flex flex-wrap items-center gap-1.5">
                  {(["pdf", "xlsx", "json"] as const).map((fmt) => {
                    const active = invFormats.includes(fmt);
                    return (
                      <Button
                        key={fmt}
                        id={`invoice-fmt-${fmt}`}
                        type="button"
                        size="xs"
                        variant={active ? "default" : "outline"}
                        aria-pressed={active}
                        className="max-md:min-h-11 uppercase"
                        onClick={() =>
                          setInvFormats((prev) =>
                            prev.includes(fmt) ? (prev.length > 1 ? prev.filter((x) => x !== fmt) : prev) : [...prev, fmt]
                          )
                        }
                      >
                        {fmt}
                      </Button>
                    );
                  })}
                </div>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex flex-wrap items-center gap-4 text-xs text-ink">
                <label className="inline-flex min-h-11 items-center gap-2 md:min-h-8">
                  <input
                    id="invoice-include-llm"
                    type="checkbox"
                    checked={invIncludeLlm}
                    onChange={(e) => setInvIncludeLlm(e.target.checked)}
                  />
                  <span>Include LLM usage cost</span>
                </label>
                <label className="inline-flex min-h-11 items-center gap-2 md:min-h-8">
                  <input
                    id="invoice-include-time"
                    type="checkbox"
                    checked={invIncludeTime}
                    onChange={(e) => setInvIncludeTime(e.target.checked)}
                  />
                  <span>Include unbilled time ({unbilledEntries.length})</span>
                </label>
              </div>
              <Button id="invoice-generate-btn" type="button" size="sm" disabled={!activeEng || generatingInv} className="max-md:min-h-11" onClick={() => void handleGenerateInvoiceSubmit()}>
                {generatingInv ? "Generating…" : "Generate invoice"}
              </Button>
            </div>
          </div>

          {engagementInvoices.length === 0 ? (
            <p className="rounded-sm border border-hairline bg-board/40 px-3 py-2.5 text-xs text-ink-2">
              No invoices generated for this engagement yet.
            </p>
          ) : (
            <div className="grid gap-3">
              <ul id="invoices-list" className="divide-y divide-hairline rounded-sm border border-hairline bg-surface">
                {engagementInvoices.map((inv) => {
                  const isSel = selectedInvoice?.id === inv.id;
                  const busyThis = updatingInvId === inv.id;
                  return (
                    <li key={inv.id} data-invoice-id={inv.id} className={cn("flex flex-wrap items-center justify-between gap-2 px-3 py-2.5 text-xs", isSel && "bg-blue-soft/40")}>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-1.5">
                          <span className="mono min-w-0 [overflow-wrap:anywhere] font-medium text-ink">{inv.invoice_number}</span>
                          <span className="min-w-0 [overflow-wrap:anywhere] text-ink-2">{inv.period_label}</span>
                          <span
                            data-invoice-status={inv.status}
                            className={cn(
                              "mono rounded-chip px-1.5 py-0.5 text-[11px] font-medium",
                              inv.status === "paid"
                                ? "status-finished"
                                : inv.status === "void"
                                  ? "status-stopped"
                                  : "bg-track text-ink-2"
                            )}
                          >
                            {inv.status}
                          </span>
                        </div>
                        <p className="mono text-ink-2">
                          Total {formatUsd(inv.total_usd)}{inv.sha256 ? ` · sha256:${inv.sha256.slice(0, 12)}` : ""}
                        </p>
                      </div>
                      <div className="flex flex-wrap items-center gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant={isSel ? "default" : "outline"}
                          data-preview-invoice={inv.id}
                          className="max-md:min-h-11"
                          onClick={() => setSelectedInvoiceId(inv.id)}
                        >
                          Preview
                        </Button>
                        {inv.status === "draft" && onUpdateInvoiceStatus && (
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            data-issue-invoice={inv.id}
                            disabled={busyThis}
                            className="max-md:min-h-11"
                            onClick={async () => {
                              setUpdatingInvId(inv.id);
                              try {
                                await onUpdateInvoiceStatus(inv.id, "issued", activeEng?.id);
                              } finally {
                                setUpdatingInvId(null);
                              }
                            }}
                          >
                            Mark Issued
                          </Button>
                        )}
                        {(inv.status === "draft" || inv.status === "issued") && onUpdateInvoiceStatus && (
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            data-pay-invoice={inv.id}
                            disabled={busyThis}
                            className="max-md:min-h-11"
                            onClick={async () => {
                              setUpdatingInvId(inv.id);
                              try {
                                await onUpdateInvoiceStatus(inv.id, "paid", activeEng?.id);
                              } finally {
                                setUpdatingInvId(null);
                              }
                            }}
                          >
                            Mark Paid
                          </Button>
                        )}
                      </div>
                    </li>
                  );
                })}
              </ul>

              {selectedInvoice && (
                <div id="invoice-preview-panel" className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-board/40 p-3 text-xs">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <span className="mono font-medium text-ink">
                      {selectedInvoice.invoice_number} ({selectedInvoice.period_label})
                    </span>
                    <div className="flex flex-wrap items-center gap-1.5">
                      {selectedInvoice.pdf_path && (
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => void downloadWorkspaceFile(selectedInvoice.pdf_path!)}
                        >
                          PDF
                        </Button>
                      )}
                      {selectedInvoice.xlsx_path && (
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => void downloadWorkspaceFile(selectedInvoice.xlsx_path!)}
                        >
                          XLSX
                        </Button>
                      )}
                      {selectedInvoice.json_path && (
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => onOpenFile(selectedInvoice.json_path!)}
                        >
                          JSON
                        </Button>
                      )}
                    </div>
                  </div>

                  <div className="overflow-x-auto rounded-sm border border-hairline bg-surface">
                    <table className="w-full border-collapse text-left text-xs">
                      <thead className="border-b border-hairline bg-board text-ink">
                        <tr>
                          <th className="px-2.5 py-1.5 font-medium">Item</th>
                          <th className="px-2.5 py-1.5 text-right font-medium">Qty / Hrs</th>
                          <th className="px-2.5 py-1.5 text-right font-medium">Rate</th>
                          <th className="px-2.5 py-1.5 text-right font-medium">Amount</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-hairline">
                        {selectedInvoice.line_items.map((item, idx) => (
                          <tr key={item.id || `${idx}-${item.description}`}>
                            <td className="px-2.5 py-1.5 text-ink">{item.description}</td>
                            <td className="mono px-2.5 py-1.5 text-right text-ink-2">
                              {item.hours ?? item.quantity ?? 1}
                            </td>
                            <td className="mono px-2.5 py-1.5 text-right text-ink-2">
                              {formatUsd(item.rate_usd ?? item.unit_price_usd ?? item.amount_usd)}
                            </td>
                            <td className="mono px-2.5 py-1.5 text-right text-ink">
                              {formatUsd(item.amount_usd)}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  <div className="mono flex flex-wrap items-center justify-end gap-4 text-xs text-ink-2">
                    <span>Subtotal: {formatUsd(selectedInvoice.billable_subtotal_usd)}</span>
                    <span>LLM cost: {formatUsd(selectedInvoice.llm_cost_usd)}</span>
                    <span>Tax: {formatUsd(selectedInvoice.tax_usd)}</span>
                    <span className="font-medium text-ink">Total: {formatUsd(selectedInvoice.total_usd)}</span>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </PanelModal>
    </div>
  );
});

export default PaperFace;
