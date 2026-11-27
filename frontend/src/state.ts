// Ergon v0.7.0 client state: conversations, live SSE (/api/runs/stream + routine_fired),
// /api/runs/status fallback, active run cancellation, multi-file upload, 12 built-in workflows,
// scheduled routines (pause/resume, next_run_ms, run history), workspace switching, drafts,
// pending recovery, person isolation, and live rail/tab counts.
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import {
  call,
  cancelLiveRun,
  cancelPipelineRun,
  createClient,
  createEngagement,
  createTimeEntry,
  deleteTimeEntry,
  deleteMcpServer,
  discoverMcpServer,
  estimateNextRunMs,
  fetchClients,
  fetchEngagementContext,
  fetchInvoices,
  fetchMeteringSummary,
  fetchTimeEntries,
  fileUploadBody,
  generateInvoice,
  getPipelineRun,
  hasClientCrmInvoicingApi,
  hasClientsApi,
  hasMcpApi,
  hasMeteringApi,
  hasPipelinesApi,
  MAX_UPLOAD_BYTES,
  hasV050Api,
  hasV070Api,
  listMcpServers,
  listPipelineRuns,
  resumePipelineRun,
  retryPipelineRun,
  saveMcpServer,
  startPipelineRun,
  switchEngagement,
  updateClient,
  updateEngagement,
  updateInvoiceStatus,
  type Client,
  type ClientContact,
  type Engagement,
  type EngagementContext,
  type ImapConfig,
  type ImapSecurity,
  type InvoiceRecord,
  type InvoiceStatus,
  type LiveRun,
  type LiveRunStatus,
  type LiveRunStep,
  type McpServerSpec,
  type McpToolSpec,
  type McpTransport,
  type MemoryHealth,
  type MeteringSummary,
  type Msg,
  type OAuthConfig,
  type OutboundConnector,
  type OutboundConnectorKind,
  type Person,
  type PipelineResumePayload,
  type PipelineRun,
  type PipelineStepSpec,
  type PlaybookPreviewResult,
  type RolloverMode,
  type Routine,
  type RoutineRunRecord,
  type RunUsage,
  type SchedulerStatus,
  type SmtpConfig,
  type SmtpSecurity,
  type StarterLaunchResult,
  type StarterPlaybook,
  type Status,
  type SuiteItem,
  type Thread,
  type ThreadRow,
  type TimeExpenseEntry,
  type TimeExpenseKind,
  type TriggerRule,
  type WorkspaceFile,
  type WorkspacePolicy,
  type WorkspacePolicyProfile,
} from "@/lib/api";

export type {
  ImapConfig,
  ImapSecurity,
  McpServerSpec,
  McpToolSpec,
  McpTransport,
  OAuthConfig,
  OutboundConnector,
  OutboundConnectorKind,
  SmtpConfig,
  SmtpSecurity,
  TriggerRule,
};
import {
  CANONICAL_SKILLS,
  CANONICAL_STARTER_PLAYBOOKS,
  mergeServerSkills,
  mergeStarterPlaybooks,
  type SkillMeta,
} from "@/lib/skillsCatalog";
import { buildDeterministicPlaybookPreview, MAX_PLAYBOOK_PACK_BYTES, verifyAndParsePlaybookPack } from "@/lib/receiptExport";
import { buildStarterCards, toolLabel, type StarterCard } from "@/lib/words";
import { threadState } from "@/chat/Entry";
import type { Recent } from "@/chat/Chat";

const DRAFT = "ergon-draft:";
const THREAD_KEY = "ergon-thread";
const PENDING = "ergon-pending";
const LOCAL_ROUTINES_KEY = "ergon-local-routines-v5";

const store = {
  get(k: string): string { try { return sessionStorage.getItem(k) || ""; } catch { return ""; } },
  set(k: string, v: string) { try { v ? sessionStorage.setItem(k, v) : sessionStorage.removeItem(k); } catch { /* convenience */ } },
};

function loadLocalRoutines(): Routine[] {
  try {
    const raw = localStorage.getItem(LOCAL_ROUTINES_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter((x): x is Routine => !!x && typeof x === "object" && typeof x.id === "string") : [];
  } catch {
    return [];
  }
}

function saveLocalRoutines(list: Routine[]) {
  try {
    localStorage.setItem(LOCAL_ROUTINES_KEY, JSON.stringify(list));
  } catch { /* ignore */ }
}

export type TState = "waiting" | "finished" | "none";

function countLearnedLines(md: string): number {
  return String(md || "")
    .split("\n")
    .filter((l) => {
      const t = l.trim();
      return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:");
    }).length;
}

function normalizeLiveRunItem(raw: unknown, fallbackStarted = Date.now()): LiveRun | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const runId = String(o.run_id ?? o.id ?? "").trim();
  const threadId = String(o.thread_id ?? "").trim();
  if (!runId && !threadId) return null;
  const startedMs = typeof o.started_ms === "number" && o.started_ms > 0 ? o.started_ms : fallbackStarted;
  const rawSteps = Array.isArray(o.steps) ? (o.steps as (string | LiveRunStep)[]) : [];
  return {
    active: o.active !== false && !o.cancelled,
    run_id: runId || threadId,
    thread_id: threadId,
    task: String(o.task ?? o.routine_name ?? "Active run"),
    started_ms: startedMs,
    elapsed_ms: typeof o.elapsed_ms === "number" ? o.elapsed_ms : Math.max(0, Date.now() - startedMs),
    steps: rawSteps.map((s, idx) => normalizeStepItem(s, idx, startedMs)),
    cancelled: Boolean(o.cancelled),
    kind: typeof o.kind === "string" ? o.kind : undefined,
    routine_id: typeof o.routine_id === "string" ? o.routine_id : null,
    routine_name: typeof o.routine_name === "string" ? o.routine_name : null,
  };
}

function normalizeStepItem(raw: string | LiveRunStep, idx: number, runStarted: number): LiveRunStep {
  if (raw && typeof raw === "object") {
    const tool = String(raw.tool ?? "").trim();
    const rawStr = String(raw.raw ?? (tool ? `Tool: ${tool}` : raw.label ?? ""));
    return {
      id: raw.id || `step-${idx}`,
      tool,
      label: raw.label || (tool ? toolLabel(tool, undefined, rawStr) || tool : rawStr),
      status: raw.status || "done",
      elapsed_ms: typeof raw.elapsed_ms === "number" ? raw.elapsed_ms : undefined,
      started_ms: raw.started_ms,
      finished_ms: raw.finished_ms,
      raw: rawStr,
      file: raw.file,
    };
  }
  const safe = String(raw ?? "");
  const nl = safe.indexOf("\n");
  const head = nl < 0 ? safe : safe.slice(0, nl);
  const body = nl < 0 ? "" : safe.slice(nl + 1);
  const m = /^Tool: (\S+)(?: \(via ([^)]+)\))?$/.exec(head);
  const tool = m ? m[1] : "";
  const label = (m && toolLabel(m[1], m[2], body)) || head;
  const waiting = /parked for your person|has NOT been deleted yet/i.test(safe);
  const isErr = /^error:/i.test(body.trim());
  return {
    id: `step-${idx}-${tool || "s"}`,
    tool,
    label,
    status: waiting ? "waiting" : isErr ? "error" : "done",
    elapsed_ms: runStarted > 0 ? Math.max(1, Date.now() - runStarted) : undefined,
    raw: safe,
  };
}

function normalizeRoutineRun(raw: unknown, fallbackRoutineId = ""): RoutineRunRecord | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const runId = String(o.run_id ?? o.id ?? "").trim();
  if (!runId) return null;
  const started =
    typeof o.started_ms === "number"
      ? o.started_ms
      : typeof o.started_at === "number"
        ? o.started_at
        : typeof o.ts === "number"
          ? o.ts
          : Date.now();
  const finished =
    typeof o.finished_ms === "number"
      ? o.finished_ms
      : typeof o.finished_at === "number"
        ? o.finished_at
        : null;
  return {
    run_id: runId,
    routine_id: String(o.routine_id ?? fallbackRoutineId),
    started_ms: started,
    finished_ms: finished,
    status: String(o.status ?? "finished"),
    receipt_id: typeof o.receipt_id === "string" && o.receipt_id ? o.receipt_id : null,
    thread_id: typeof o.thread_id === "string" && o.thread_id ? o.thread_id : null,
    summary: typeof o.summary === "string" ? o.summary : typeof o.outcome === "string" ? o.outcome : undefined,
  };
}

function normalizeRoutine(raw: unknown): Routine | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const id = String(o.id ?? "").trim();
  const name = String(o.name ?? o.title ?? "").trim();
  const prompt = String(o.prompt ?? "").trim();
  if (!id || !name || !prompt) return null;
  const schedule = String(o.schedule ?? o.cadence ?? "daily 09:00").trim();
  const enabled = typeof o.enabled === "boolean" ? o.enabled : true;
  const intervalSecs = typeof o.interval_secs === "number" ? o.interval_secs : null;
  const tz = typeof o.tz === "string" && o.tz.trim() ? o.tz.trim() : undefined;
  const tzOffsetMins =
    typeof o.tz_offset_mins === "number" && Number.isFinite(o.tz_offset_mins)
      ? o.tz_offset_mins
      : null;
  const lastRunTs =
    typeof o.last_run_ms === "number"
      ? o.last_run_ms
      : typeof o.last_run_ts === "number"
        ? o.last_run_ts
        : null;
  const nextRunMs =
    typeof o.next_run_ms === "number"
      ? o.next_run_ms
      : typeof o.next_due_ms === "number"
        ? o.next_due_ms
        : enabled
          ? estimateNextRunMs(schedule, lastRunTs || Date.now(), intervalSecs, tzOffsetMins, tz)
          : null;
  const rawRuns = Array.isArray(o.recent_runs)
    ? o.recent_runs
    : Array.isArray(o.history)
      ? o.history
      : [];
  const recentRuns = rawRuns
    .map((item) => normalizeRoutineRun(item, id))
    .filter((x): x is RoutineRunRecord => x !== null);
  const verifiedRaw = o.last_run_verified;
  const lastRunVerified: "verified" | "drift" | null =
    verifiedRaw === "verified" || verifiedRaw === "drift" ? verifiedRaw : null;
  return {
    id,
    name,
    prompt,
    schedule,
    cadence: typeof o.cadence === "string" ? o.cadence : schedule,
    interval_secs: intervalSecs,
    tz,
    tz_offset_mins: tzOffsetMins,
    skill_id: typeof o.skill_id === "string" && o.skill_id ? o.skill_id : undefined,
    enabled,
    last_run_ts: lastRunTs,
    next_run_ms: enabled ? nextRunMs : null,
    missed_windows: typeof o.missed_windows === "number" ? o.missed_windows : 0,
    last_status: typeof o.last_status === "string" ? o.last_status : undefined,
    last_run_verified: lastRunVerified,
    last_run_receipt_sha256: typeof o.last_run_receipt_sha256 === "string" ? o.last_run_receipt_sha256 : null,
    last_run_receipt_id: typeof o.last_run_receipt_id === "string" ? o.last_run_receipt_id : null,
    run_count: typeof o.run_count === "number" ? o.run_count : 0,
    created_ts: typeof o.created_ms === "number" ? o.created_ms : typeof o.created_ts === "number" ? o.created_ts : undefined,
    steps: Array.isArray(o.steps) ? o.steps.filter((s): s is string => typeof s === "string") : undefined,
    steps_v2: Array.isArray(o.steps_v2) ? (o.steps_v2 as PipelineStepSpec[]) : undefined,
    expected_inputs: Array.isArray(o.expected_inputs) ? o.expected_inputs.filter((s): s is string => typeof s === "string") : undefined,
    pack_deliverables: typeof o.pack_deliverables === "boolean" ? o.pack_deliverables : undefined,
    recent_runs: recentRuns.length ? recentRuns : undefined,
  };
}

function parseRunUsage(raw: unknown): RunUsage | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
  const o = raw as Record<string, unknown>;
  const pt = Number(o.prompt_tokens ?? 0);
  const ct = Number(o.completion_tokens ?? 0);
  const tt = Number(o.total_tokens ?? pt + ct);
  const cost = Number(o.cost_usd ?? 0);
  if (!Number.isFinite(pt) && !Number.isFinite(ct) && !Number.isFinite(cost)) return null;
  return {
    model_id: typeof o.model_id === "string" ? o.model_id : undefined,
    prompt_tokens: Number.isFinite(pt) ? pt : 0,
    completion_tokens: Number.isFinite(ct) ? ct : 0,
    cached_tokens: typeof o.cached_tokens === "number" ? o.cached_tokens : undefined,
    total_tokens: Number.isFinite(tt) ? tt : 0,
    cost_usd: Number.isFinite(cost) ? cost : 0,
    latency_ms: typeof o.latency_ms === "number" ? o.latency_ms : undefined,
    estimated_minutes_saved: typeof o.estimated_minutes_saved === "number" ? o.estimated_minutes_saved : undefined,
    billable_value_usd: typeof o.billable_value_usd === "number" ? o.billable_value_usd : undefined,
    engagement_id: typeof o.engagement_id === "string" ? o.engagement_id : undefined,
  };
}

export function useErgon() {
  const [status, setStatus] = useState<Status | null | "error">(null);
  const statusRef = useRef<Status | null | "error">(null);
  const [runFailed, setRunFailed] = useState(false);
  const [people, setPeople] = useState<{ people: Person[]; active: string } | null>(null);
  const [threads, setThreads] = useState<ThreadRow[] | null>(null);
  const [threadsError, setThreadsError] = useState(false);
  const [states, setStates] = useState<Record<string, TState>>({});
  const [threadId, setThreadIdState] = useState<string | null>(null);
  const threadRef = useRef<string | null>(null);
  const [messages, setMessages] = useState<Msg[]>([]);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const activeRunIdRef = useRef<string | null>(null);
  const [runStartedMs, setRunStartedMs] = useState<number>(0);
  const [liveElapsedMs, setLiveElapsedMs] = useState<number>(0);
  const [streamMode, setStreamMode] = useState<"sse" | "poll">("poll");
  const [cancelling, setCancelling] = useState(false);
  const [cancellingRunIds, setCancellingRunIds] = useState<Record<string, boolean>>({});
  const [liveRuns, setLiveRuns] = useState<LiveRun[]>([]);
  const liveRunsRef = useRef<LiveRun[]>([]);
  const [liveStreamError, setLiveStreamError] = useState("");
  const [liveSteps, setLiveSteps] = useState<string[]>([]);
  const [liveTimeline, setLiveTimeline] = useState<LiveRunStep[]>([]);
  const abortRef = useRef<AbortController | null>(null);
  const [draft, setDraftState] = useState("");
  const [pendingNote, setPendingNote] = useState(false);
  const [sideError, setSideError] = useState("");
  const [starters, setStarters] = useState<StarterCard[]>(() => buildStarterCards([], "", []));
  const [recovered, setRecovered] = useState<string[]>([]);
  const [recent, setRecent] = useState<Recent[]>([]);
  const [todayCount, setTodayCount] = useState(0);
  const [learnedCount, setLearnedCount] = useState(0);
  const [memoryHealth, setMemoryHealth] = useState<MemoryHealth | null>(null);
  const [suiteCount, setSuiteCount] = useState(0);
  const [files, setFiles] = useState<WorkspaceFile[]>([]);
  const filesRef = useRef<WorkspaceFile[]>([]);
  const [lastUploaded, setLastUploaded] = useState<string[]>([]);
  const [skills, setSkills] = useState<SkillMeta[]>(() => CANONICAL_SKILLS);
  const [starterPlaybooks, setStarterPlaybooks] = useState<StarterPlaybook[]>(() => CANONICAL_STARTER_PLAYBOOKS);
  const [routines, setRoutines] = useState<Routine[]>(() => loadLocalRoutines());
  const [routinesLoading, setRoutinesLoading] = useState(false);
  const [routinesError, setRoutinesError] = useState("");
  const [routineSaveError, setRoutineSaveError] = useState("");
  const [pipelineRuns, setPipelineRuns] = useState<PipelineRun[]>([]);
  const pipelineRunsRef = useRef<PipelineRun[]>([]);
  const [scheduler, setScheduler] = useState<SchedulerStatus | null>(null);
  const [routineHistoryRev, setRoutineHistoryRev] = useState(0);
  const [clients, setClients] = useState<Client[]>([]);
  const [engagements, setEngagements] = useState<Engagement[]>([]);
  const [activeEngagementId, setActiveEngagementId] = useState("");
  const [activeEngagementContext, setActiveEngagementContext] = useState<EngagementContext | null>(null);
  const [timeEntries, setTimeEntries] = useState<TimeExpenseEntry[]>([]);
  const [invoices, setInvoices] = useState<InvoiceRecord[]>([]);
  const [workspacePolicy, setWorkspacePolicy] = useState<WorkspacePolicy | null>(null);
  const [engagementScopeOnly, setEngagementScopeOnly] = useState(false);
  const [metering, setMetering] = useState<MeteringSummary | null>(null);
  const [liveRunUsage, setLiveRunUsage] = useState<RunUsage | null>(null);
  const [mcpServers, setMcpServers] = useState<McpServerSpec[]>([]);
  const onRoutineFiredRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  useEffect(() => {
    filesRef.current = files;
  }, [files]);

  useEffect(() => {
    liveRunsRef.current = liveRuns;
  }, [liveRuns]);

  useEffect(() => {
    pipelineRunsRef.current = pipelineRuns;
  }, [pipelineRuns]);

  const setThread = (id: string | null) => { threadRef.current = id; setThreadIdState(id); };
  const draftKey = (id: string | null) => DRAFT + (id || "new");
  const loadDraftFor = (id: string | null) => setDraftState(store.get(draftKey(id)));
  const setDraft = (v: string) => { setDraftState(v); store.set(draftKey(threadRef.current), v); };

  const loadStatus = useCallback(async () => {
    const r = await call<Status>("/api/status");
    if (!r.ok || typeof r.data.model_configured !== "boolean") { setStatus("error"); return; }
    statusRef.current = r.data;
    setStatus(r.data);
    if (r.data.scheduler && typeof r.data.scheduler === "object") {
      setScheduler(r.data.scheduler);
    }
    setRecovered(Array.isArray(r.data.store_recovered) ? r.data.store_recovered.map(String) : []);
  }, []);

  const loadPeople = useCallback(async () => {
    const r = await call<{ people: Person[]; active: string }>("/api/people");
    if (!r.ok || !Array.isArray(r.data.people)) {
      setPeople(null);
      setSideError("Could not load the people list. Is the Ergon server running?");
      return;
    }
    setPeople({
      people: r.data.people.filter((p): p is Person => !!p && typeof p === "object" && typeof p.id === "string"),
      active: typeof r.data.active === "string" ? r.data.active : "",
    });
    setSideError("");
  }, []);

  const loadClients = useCallback(async () => {
    if (!hasClientsApi(statusRef.current)) {
      setClients([]);
      setEngagements([]);
      setActiveEngagementId("");
      return;
    }
    const r = await fetchClients();
    if (!r.ok) return;
    setClients(r.data.clients);
    setEngagements(r.data.engagements);
    setActiveEngagementId(r.data.active_engagement_id || "");
  }, []);

  const loadMetering = useCallback(async (opts: { engagement_id?: string; client_id?: string; window_days?: number } = {}) => {
    if (!hasMeteringApi(statusRef.current)) {
      setMetering(null);
      return;
    }
    const r = await fetchMeteringSummary(opts);
    if (r.ok) {
      setMetering(r.data);
    }
  }, []);

  const loadMcpServers = useCallback(async () => {
    if (!hasMcpApi(statusRef.current)) {
      setMcpServers([]);
      return;
    }
    const r = await listMcpServers();
    if (r.ok) {
      setMcpServers(r.data.servers);
    }
  }, []);

  const saveMcpServerConfig = useCallback(
    async (body: {
      id?: string;
      name: string;
      transport: McpTransport;
      command_or_url: string;
      args: string[];
      env_secret_keys: string[];
      require_approval_by_default: boolean;
      enabled: boolean;
    }): Promise<McpServerSpec | null> => {
      const r = await saveMcpServer(body);
      if (!r.ok) {
        toast.error(r.error || "Could not save MCP server.");
        return null;
      }
      await loadMcpServers();
      return r.data.server ?? null;
    },
    [loadMcpServers]
  );

  const discoverMcpServerTools = useCallback(
    async (id: string): Promise<McpToolSpec[] | null> => {
      const r = await discoverMcpServer(id);
      if (!r.ok) {
        toast.error(r.error || "Could not discover MCP server tools.");
        return null;
      }
      await loadMcpServers();
      return r.data.server?.cached_tools ?? r.data.tools ?? r.data.cached_tools ?? [];
    },
    [loadMcpServers]
  );

  const removeMcpServer = useCallback(
    async (id: string): Promise<boolean> => {
      const r = await deleteMcpServer(id);
      if (!r.ok) {
        toast.error(r.error || "Could not delete MCP server.");
        return false;
      }
      setMcpServers((prev) => prev.filter((s) => s.id !== id));
      return true;
    },
    []
  );

  const loadWorkspacePolicy = useCallback(async () => {
    const r = await call<WorkspacePolicy>("/api/workspace/policy");
    if (r.ok && r.data && typeof r.data.profile === "string") {
      setWorkspacePolicy(r.data);
    }
  }, []);

  const updateWorkspacePolicyState = useCallback(
    async (patch: { profile?: WorkspacePolicyProfile; isolate_active_engagement?: boolean }): Promise<WorkspacePolicy | null> => {
      const r = await call<WorkspacePolicy>("/api/workspace/policy", {
        method: "PUT",
        body: patch,
      });
      if (!r.ok) {
        toast.error(r.error || "Could not update workspace policy.");
        return null;
      }
      setWorkspacePolicy(r.data);
      return r.data;
    },
    []
  );

  const loadEngagementContext = useCallback(async (engagementId?: string) => {
    const targetId = engagementId ?? activeEngagementId;
    if (!targetId || !hasClientCrmInvoicingApi(statusRef.current)) {
      setActiveEngagementContext(null);
      return null;
    }
    const r = await fetchEngagementContext(targetId);
    if (r.ok) {
      setActiveEngagementContext(r.data);
      return r.data;
    }
    return null;
  }, [activeEngagementId]);

  const loadTimeEntries = useCallback(async (opts: { engagement_id?: string; client_id?: string; force?: boolean } = {}) => {
    if (!opts.force && !hasClientCrmInvoicingApi(statusRef.current)) {
      setTimeEntries([]);
      return;
    }
    const r = await fetchTimeEntries({ engagement_id: opts.engagement_id, client_id: opts.client_id });
    if (r.ok) {
      setTimeEntries(r.data.entries);
    }
  }, []);

  const loadInvoices = useCallback(async (opts: { engagement_id?: string; client_id?: string; force?: boolean } = {}) => {
    if (!opts.force && !hasClientCrmInvoicingApi(statusRef.current)) {
      setInvoices([]);
      return;
    }
    const r = await fetchInvoices({ engagement_id: opts.engagement_id, client_id: opts.client_id });
    if (r.ok) {
      setInvoices(r.data.invoices);
    }
  }, []);

  const createNewClient = useCallback(
    async (input: {
      name: string;
      brand_header?: string;
      default_rate_usd?: number;
      billing_currency?: string;
      contacts?: ClientContact[];
      notes_md?: string;
      tax_rate_pct?: number;
      payment_terms?: string;
    }): Promise<Client | null> => {
      const r = await createClient(input);
      if (!r.ok) {
        toast.error(r.error || "Could not create client.");
        return null;
      }
      await loadClients();
      toast.success(`Created client "${r.data.client.name}".`);
      return r.data.client;
    },
    [loadClients]
  );

  const saveClientCrm = useCallback(
    async (
      id: string,
      patch: {
        name?: string;
        brand_header?: string;
        billing_currency?: string;
        default_rate_usd?: number;
        contacts?: ClientContact[];
        notes_md?: string;
        tax_rate_pct?: number;
        payment_terms?: string;
      }
    ): Promise<Client | null> => {
      const r = await updateClient(id, patch);
      if (!r.ok) {
        toast.error(r.error || "Could not update client CRM details.");
        return null;
      }
      await Promise.all([loadClients(), loadMetering(), loadEngagementContext()]);
      toast.success(`Saved CRM details for "${r.data.client.name}".`);
      return r.data.client;
    },
    [loadClients, loadMetering, loadEngagementContext]
  );

  const createNewEngagement = useCallback(
    async (input: {
      client_id: string;
      name: string;
      person_id?: string;
      retainer_hours?: number;
      retainer_budget_usd?: number;
      hourly_rate_usd?: number;
      period?: string;
      workspace_prefix?: string;
      sow_rules?: string[];
      rollover_mode?: RolloverMode;
    }): Promise<Engagement | null> => {
      const r = await createEngagement(input);
      if (!r.ok) {
        toast.error(r.error || "Could not create engagement.");
        return null;
      }
      await Promise.all([loadClients(), loadMetering()]);
      toast.success(`Created engagement "${r.data.engagement.name}".`);
      return r.data.engagement;
    },
    [loadClients, loadMetering]
  );

  const saveEngagementSow = useCallback(
    async (
      id: string,
      patch: {
        name?: string;
        retainer_hours?: number;
        retainer_budget_usd?: number;
        hourly_rate_usd?: number;
        period?: string;
        workspace_prefix?: string;
        sow_rules?: string[];
        rollover_mode?: RolloverMode;
        status?: "active" | "paused" | "archived" | string;
        archived_ms?: number | null;
      }
    ): Promise<Engagement | null> => {
      const r = await updateEngagement(id, patch);
      if (!r.ok) {
        toast.error(r.error || "Could not update engagement SOW.");
        return null;
      }
      await Promise.all([loadClients(), loadMetering(), loadEngagementContext(id)]);
      toast.success(`Updated "${r.data.engagement.name}".`);
      return r.data.engagement;
    },
    [loadClients, loadMetering, loadEngagementContext]
  );

  const addTimeEntry = useCallback(
    async (input: {
      client_id?: string;
      engagement_id: string;
      kind: TimeExpenseKind;
      description: string;
      hours?: number;
      rate_usd?: number;
      amount_usd?: number;
      billable?: boolean;
    }): Promise<TimeExpenseEntry | null> => {
      const r = await createTimeEntry(input);
      if (!r.ok) {
        toast.error(r.error || "Could not log time or expense entry.");
        return null;
      }
      await Promise.all([
        loadTimeEntries({ engagement_id: input.engagement_id, force: true }),
        loadEngagementContext(input.engagement_id),
        loadMetering(),
      ]);
      toast.success("Logged billable entry.");
      return r.data.entry;
    },
    [loadTimeEntries, loadEngagementContext, loadMetering]
  );

  const removeTimeEntry = useCallback(
    async (id: string, engagementId?: string): Promise<boolean> => {
      const r = await deleteTimeEntry(id);
      if (!r.ok) {
        toast.error(r.error || "Could not remove entry.");
        return false;
      }
      await Promise.all([
        loadTimeEntries({ engagement_id: engagementId, force: true }),
        loadEngagementContext(engagementId),
        loadMetering(),
      ]);
      toast.success("Removed entry.");
      return true;
    },
    [loadTimeEntries, loadEngagementContext, loadMetering]
  );

  const createGeneratedInvoice = useCallback(
    async (input: {
      engagement_id: string;
      period_label: string;
      include_llm_cost: boolean;
      include_time_entries: boolean;
      export_formats: Array<"pdf" | "xlsx" | "json">;
    }): Promise<InvoiceRecord | null> => {
      const r = await generateInvoice(input);
      if (!r.ok) {
        toast.error(r.error || "Could not generate invoice.");
        return null;
      }
      await Promise.all([
        loadInvoices({ engagement_id: input.engagement_id, force: true }),
        loadTimeEntries({ engagement_id: input.engagement_id, force: true }),
        loadEngagementContext(input.engagement_id),
        loadMetering(),
      ]);
      toast.success(`Generated invoice ${r.data.invoice.invoice_number}.`);
      return r.data.invoice;
    },
    [loadInvoices, loadTimeEntries, loadEngagementContext, loadMetering]
  );

  const setInvoiceStatus = useCallback(
    async (id: string, nextStatus: InvoiceStatus, engagementId?: string): Promise<InvoiceRecord | null> => {
      const r = await updateInvoiceStatus(id, { status: nextStatus });
      if (!r.ok) {
        toast.error(r.error || "Could not update invoice status.");
        return null;
      }
      await loadInvoices({ engagement_id: engagementId, force: true });
      toast.success(`Invoice marked ${nextStatus}.`);
      return r.data.invoice;
    },
    [loadInvoices]
  );

  const loadFiles = useCallback(async () => {
    const r = await call<{ files?: WorkspaceFile[] }>("/api/files");
    if (!r.ok || !Array.isArray(r.data.files)) return;
    setFiles(
      r.data.files.filter(
        (f): f is WorkspaceFile => !!f && typeof f === "object" && typeof f.path === "string"
      )
    );
  }, []);

  const loadSkills = useCallback(async () => {
    if (!hasV050Api(statusRef.current, "skills")) {
      setSkills(CANONICAL_SKILLS);
      setStarterPlaybooks(CANONICAL_STARTER_PLAYBOOKS);
      return;
    }
    const r = await call<unknown>("/api/skills");
    if (r.ok) {
      setSkills(mergeServerSkills(r.data));
      setStarterPlaybooks(mergeStarterPlaybooks(r.data));
    } else {
      setSkills(CANONICAL_SKILLS);
      setStarterPlaybooks(CANONICAL_STARTER_PLAYBOOKS);
    }
  }, []);

  const loadRoutines = useCallback(async () => {
    if (!hasV050Api(statusRef.current, "routines")) {
      setRoutinesError("");
      setRoutines(loadLocalRoutines());
      return;
    }
    setRoutinesLoading(true);
    try {
      const r = await call<{ routines?: unknown[]; scheduler?: SchedulerStatus }>("/api/routines");
      if (!r.ok) {
        if (r.status === 404 || r.status === 405) {
          setRoutinesError("");
          setRoutines(loadLocalRoutines());
          return;
        }
        setRoutinesError(r.error || "Could not load recurring routines.");
        return;
      }
      if (Array.isArray(r.data.routines)) {
        const clean = r.data.routines.map(normalizeRoutine).filter((x): x is Routine => x !== null);
        setRoutines(clean);
        setRoutinesError("");
        if (r.data.scheduler && typeof r.data.scheduler === "object") {
          setScheduler(r.data.scheduler);
        }
        return;
      }
      setRoutinesError("");
      setRoutines(loadLocalRoutines());
    } finally {
      setRoutinesLoading(false);
    }
  }, []);

  const toggleRoutine = useCallback(
    async (id: string, enabled: boolean): Promise<boolean> => {
      if (hasV070Api(statusRef.current)) {
        const r = await call<{ routine?: unknown }>(`/api/routines/${encodeURIComponent(id)}`, {
          method: "PATCH",
          body: { enabled },
        });
        if (r.ok) {
          toast.success(enabled ? "Routine resumed." : "Routine paused.");
          await loadRoutines();
          return true;
        }
        if (r.status !== 404 && r.status !== 405) {
          toast.error(r.error || "Could not update routine status.");
          return false;
        }
      }
      const next = loadLocalRoutines().map((x) =>
        x.id === id
          ? { ...x, enabled, next_run_ms: enabled ? estimateNextRunMs(x.schedule, Date.now(), x.interval_secs) : null }
          : x
      );
      saveLocalRoutines(next);
      setRoutines((prev) =>
        prev.map((x) =>
          x.id === id
            ? { ...x, enabled, next_run_ms: enabled ? estimateNextRunMs(x.schedule, Date.now(), x.interval_secs) : null }
            : x
        )
      );
      toast.success(enabled ? "Routine resumed." : "Routine paused.");
      return true;
    },
    [loadRoutines]
  );

  const loadRoutineHistory = useCallback(async (id: string): Promise<RoutineRunRecord[]> => {
    if (hasV070Api(statusRef.current)) {
      const r = await call<{
        runs?: unknown[];
        history?: unknown[];
        records?: unknown[];
      }>(`/api/routines/${encodeURIComponent(id)}/history`);
      if (r.ok) {
        const arr = Array.isArray(r.data.runs)
          ? r.data.runs
          : Array.isArray(r.data.history)
            ? r.data.history
            : Array.isArray(r.data.records)
              ? r.data.records
              : [];
        return arr
          .map((item) => normalizeRoutineRun(item, id))
          .filter((x): x is RoutineRunRecord => x !== null)
          .sort((a, b) => b.started_ms - a.started_ms);
      }
      if (r.status !== 404 && r.status !== 405) {
        throw new Error(r.error || "Could not load routine execution history.");
      }
    }
    const local = loadLocalRoutines().find((x) => x.id === id);
    return local?.recent_runs ?? [];
  }, []);

  const createRoutine = useCallback(
    async (input: {
      name: string;
      schedule: string;
      prompt: string;
      tz?: string;
      tz_offset_mins?: number | null;
      skill_id?: string;
      steps?: string[];
      steps_v2?: PipelineStepSpec[];
      expected_inputs?: string[];
      pack_deliverables?: boolean;
    }): Promise<boolean> => {
      setRoutineSaveError("");
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call<{ routine?: unknown } & Record<string, unknown>>("/api/routines", {
          method: "POST",
          body: {
            name: input.name,
            title: input.name,
            schedule: input.schedule,
            cadence: input.schedule,
            ...(input.tz ? { tz: input.tz } : {}),
            ...(typeof input.tz_offset_mins === "number" ? { tz_offset_mins: input.tz_offset_mins } : {}),
            prompt: input.prompt,
            skill_id: input.skill_id,
            steps: input.steps,
            steps_v2: input.steps_v2,
            expected_inputs: input.expected_inputs,
            pack_deliverables: input.pack_deliverables,
            enabled: true,
          },
        });
        if (r.ok) {
          setRoutineSaveError("");
          toast.success(`Saved routine "${input.name}".`);
          await loadRoutines();
          return true;
        }
        if (r.status !== 404 && r.status !== 405) {
          const msg = r.error || "Could not create routine.";
          setRoutineSaveError(msg);
          toast.error(msg);
          return false;
        }
      }
      const created: Routine = {
        id: `rt-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        name: input.name,
        schedule: input.schedule,
        tz: input.tz,
        tz_offset_mins: input.tz_offset_mins ?? null,
        prompt: input.prompt,
        skill_id: input.skill_id,
        steps: input.steps,
        steps_v2: input.steps_v2,
        enabled: true,
        last_run_ts: null,
        next_run_ms: estimateNextRunMs(input.schedule, Date.now(), null, input.tz_offset_mins, input.tz),
        run_count: 0,
        created_ts: Date.now(),
      };
      const next = [created, ...loadLocalRoutines()];
      saveLocalRoutines(next);
      setRoutines(next);
      toast.success(`Saved routine "${input.name}".`);
      return true;
    },
    [loadRoutines]
  );

  const deleteRoutine = useCallback(
    async (id: string) => {
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call(`/api/routines/${encodeURIComponent(id)}`, { method: "DELETE" });
        if (r.ok) {
          toast("Deleted routine.");
          await loadRoutines();
          return;
        }
      }
      const next = loadLocalRoutines().filter((x) => x.id !== id);
      saveLocalRoutines(next);
      setRoutines((prev) => prev.filter((x) => x.id !== id));
      toast("Deleted routine.");
    },
    [loadRoutines]
  );

  const importPlaybook = useCallback(
    async (rawText: string): Promise<{ ok: boolean; sha256?: string; error?: string }> => {
      const byteLen = new TextEncoder().encode(rawText).length;
      if (byteLen > MAX_PLAYBOOK_PACK_BYTES) {
        const msg = `Playbook pack exceeds the 256 KB limit (${byteLen} bytes).`;
        toast.error(msg);
        return { ok: false, error: msg };
      }
      if (hasV050Api(statusRef.current, "routines")) {
        try {
          const resp = await fetch("/api/playbooks/import", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: rawText,
          });
          const data = (await resp.json().catch(() => ({}))) as {
            routine?: unknown;
            sha256?: string;
            error?: string;
          };
          if (resp.ok) {
            const rt = normalizeRoutine(data.routine);
            const digest = typeof data.sha256 === "string" ? data.sha256 : "";
            toast.success(
              `Imported "${rt?.name || "playbook"}"${digest ? ` (verified sha256:${digest.slice(0, 12)}…)` : ""}.`
            );
            await loadRoutines();
            return { ok: true, sha256: digest };
          }
          if (resp.status !== 404 && resp.status !== 405) {
            const msg = data.error || `Playbook import failed (${resp.status}).`;
            toast.error(msg);
            return { ok: false, error: msg };
          }
        } catch (err) {
          const msg = err instanceof Error ? err.message : "Could not import playbook.";
          toast.error(msg);
          return { ok: false, error: msg };
        }
      }
      const verified = await verifyAndParsePlaybookPack(rawText);
      if (!verified.ok) {
        toast.error(verified.error);
        return { ok: false, error: verified.error };
      }
      const pb = verified.playbook;
      const created: Routine = {
        id: `rt-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        name: pb.name.trim(),
        prompt: pb.prompt.trim(),
        schedule: pb.schedule.trim(),
        interval_secs: pb.interval_secs ?? undefined,
        skill_id: pb.skill_id ?? undefined,
        steps: pb.steps.length > 0 ? pb.steps : undefined,
        expected_inputs: pb.expected_inputs.length > 0 ? pb.expected_inputs : undefined,
        pack_deliverables: pb.pack_deliverables ? true : undefined,
        enabled: true,
        last_run_ts: null,
        run_count: 0,
        created_ts: Date.now(),
      };
      const next = [created, ...loadLocalRoutines()];
      saveLocalRoutines(next);
      setRoutines(next);
      toast.success(`Imported "${created.name}" (verified sha256:${verified.sha256.slice(0, 12)}…).`);
      return { ok: true, sha256: verified.sha256 };
    },
    [loadRoutines]
  );

  const previewPlaybook = useCallback(
    async (routine: Routine): Promise<PlaybookPreviewResult | null> => {
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call<PlaybookPreviewResult>(
          `/api/playbooks/${encodeURIComponent(routine.id)}/preview`,
          { method: "POST", body: {} }
        );
        if (r.ok) {
          return r.data;
        }
        if (r.status !== 404 && r.status !== 405) {
          toast.error(r.error || "Could not preview playbook.");
          return null;
        }
      }
      return buildDeterministicPlaybookPreview(
        routine,
        files.map((f) => f.path),
        () => 0,
        0
      );
    },
    [files]
  );

  const uploadFile = useCallback(async (path: string, content: string, silent = false, encoding?: "base64"): Promise<boolean> => {
    const r = await call<WorkspaceFile>("/api/files/upload", {
      method: "POST",
      body: encoding ? { path, content, encoding } : { path, content },
    });
    if (!r.ok) {
      if (!silent) toast.error(r.error || `Could not save ${path}.`);
      return false;
    }
    if (!silent) {
      toast.success(`Saved ${r.data.path} to your workspace.`);
      await loadFiles();
    }
    return true;
  }, [loadFiles]);

  const uploadFiles = useCallback(async (picked: FileList | File[]): Promise<number> => {
    const list = Array.from(picked);
    if (!list.length) return 0;
    let okCount = 0;
    const savedNames: string[] = [];
    const tooBig: string[] = [];
    for (const file of list) {
      if (file.size > MAX_UPLOAD_BYTES) {
        tooBig.push(file.name);
        continue;
      }
      try {
        const body = await fileUploadBody(file);
        const ok = await uploadFile(file.name, body.content, true, body.encoding);
        if (ok) {
          okCount += 1;
          savedNames.push(file.name);
        }
      } catch {
        // continue remaining files
      }
    }
    await loadFiles();
    if (tooBig.length) {
      toast.error(`${tooBig.join(", ")} ${tooBig.length === 1 ? "is" : "are"} over 25 MB, too large to upload.`);
    }
    if (savedNames.length > 0) {
      setLastUploaded(savedNames);
    }
    if (okCount === 1) {
      toast.success(`Saved ${savedNames[0]} to your workspace.`);
    } else if (okCount > 1) {
      toast.success(`Uploaded ${okCount} files to your workspace.`);
    } else if (!tooBig.length) {
      toast.error("Could not upload the selected files.");
    }
    return okCount;
  }, [uploadFile, loadFiles]);

  const loadThreads = useCallback(async () => {
    const r = await call<{ threads: ThreadRow[] }>("/api/threads");
    if (!r.ok || !Array.isArray(r.data.threads)) { setThreadsError(true); return; }
    setThreadsError(false);
    const clean = r.data.threads.filter((t): t is ThreadRow => !!t && typeof t === "object" && typeof t.id === "string");
    setThreads(clean);
    const slice = clean.slice(0, 40);
    const nextStates: Record<string, TState> = {};
    const missing: ThreadRow[] = [];
    for (const t of slice) {
      if (t.state === "waiting" || t.state === "finished" || t.state === "none") {
        nextStates[t.id] = t.state;
      } else {
        missing.push(t);
      }
    }
    if (missing.length > 0) {
      const found = await Promise.all(missing.map(async (t) => {
        const th = await call<Thread>("/api/threads/" + encodeURIComponent(t.id));
        return [t.id, th.ok && Array.isArray(th.data.messages) ? threadState(th.data.messages) : "none"] as const;
      }));
      for (const [id, st] of found) nextStates[id] = st;
    }
    setStates(nextStates);
  }, []);

  const loadRecent = useCallback(async () => {
    const r = await call<{ finished?: { task?: unknown; date?: unknown; files?: unknown }[] }>("/api/brief");
    if (!r.ok || !Array.isArray(r.data.finished)) { setRecent([]); return; }
    setRecent(
      r.data.finished
        .filter((x) => !!x && typeof x === "object")
        .slice(0, 4)
        .map((x) => ({
          task: String(x.task ?? "Completed task"),
          date: String(x.date ?? ""),
          files: Array.isArray(x.files) ? x.files.map(String) : [],
        }))
    );
  }, []);

  const refreshCounts = useCallback(async () => {
    const [dec, mem, sui] = await Promise.all([
      call<{ decisions?: { status?: string }[] }>("/api/decisions"),
      call<{ content?: string; health?: MemoryHealth }>("/api/memory"),
      call<{ installed?: SuiteItem[] }>("/api/suite"),
    ]);
    if (dec.ok && Array.isArray(dec.data.decisions)) {
      setTodayCount(dec.data.decisions.filter((d) => d && d.status === "pending").length);
    }
    if (mem.ok && typeof mem.data.content === "string") {
      setLearnedCount(countLearnedLines(mem.data.content));
      if (mem.data.health && typeof mem.data.health === "object") {
        setMemoryHealth(mem.data.health);
      }
    }
    if (sui.ok && Array.isArray(sui.data.installed)) {
      setSuiteCount(sui.data.installed.length);
    }
  }, []);

  const loadPipelineRuns = useCallback(async () => {
    if (!hasPipelinesApi(statusRef.current)) {
      setPipelineRuns([]);
      return;
    }
    const r = await listPipelineRuns({ limit: 50 });
    if (r.ok && Array.isArray(r.data.runs)) {
      setPipelineRuns(
        r.data.runs.filter((x): x is PipelineRun => !!x && typeof x === "object" && typeof x.run_id === "string")
      );
    }
  }, []);

  const upsertPipelineRun = useCallback((run: PipelineRun) => {
    if (!run || !run.run_id) return;
    setPipelineRuns((prev) => {
      const idx = prev.findIndex((r) => r.run_id === run.run_id);
      if (idx >= 0) return prev.map((r, i) => (i === idx ? run : r));
      return [run, ...prev];
    });
  }, []);

  const startPipeline = useCallback(
    async (input: {
      routine_id?: string;
      name?: string;
      steps_v2?: PipelineStepSpec[];
      inputs?: Record<string, string>;
      thread_id?: string;
    }): Promise<PipelineRun | null> => {
      const r = await startPipelineRun(input);
      if (!r.ok) {
        toast.error(r.error || "Could not start pipeline.");
        return null;
      }
      if (r.data.run) {
        upsertPipelineRun(r.data.run);
        toast.success(`Started pipeline "${r.data.run.routine_name || input.name || "Pipeline"}".`);
        if (r.data.run.status === "completed") {
          await Promise.all([loadFiles(), loadThreads(), loadRecent(), refreshCounts(), loadRoutines()]);
        } else if (r.data.run.status === "paused_checkpoint") {
          await Promise.all([refreshCounts(), loadFiles()]);
        }
        return r.data.run;
      }
      await loadPipelineRuns();
      return null;
    },
    [upsertPipelineRun, loadFiles, loadThreads, loadRecent, refreshCounts, loadRoutines, loadPipelineRuns]
  );

  const pipelineOpLockRef = useRef<Set<string>>(new Set());

  const resumePipeline = useCallback(
    async (runId: string, payload: PipelineResumePayload): Promise<void> => {
      if (pipelineOpLockRef.current.has(runId)) return;
      pipelineOpLockRef.current.add(runId);
      try {
        const r = await resumePipelineRun(runId, payload);
        if (!r.ok) {
          const msg = r.error || "Could not resume pipeline.";
          toast.error(msg);
          throw new Error(msg);
        }
        if (r.data.run) {
          upsertPipelineRun(r.data.run);
        }
        toast.success("Pipeline resumed.");
        await Promise.all([loadPipelineRuns(), loadFiles(), loadThreads(), loadRecent(), refreshCounts()]);
        onRoutineFiredRef.current?.();
      } finally {
        pipelineOpLockRef.current.delete(runId);
      }
    },
    [upsertPipelineRun, loadPipelineRuns, loadFiles, loadThreads, loadRecent, refreshCounts]
  );

  const retryPipeline = useCallback(
    async (runId: string, fromStepIndex?: number): Promise<void> => {
      if (pipelineOpLockRef.current.has(runId)) return;
      pipelineOpLockRef.current.add(runId);
      try {
        const r = await retryPipelineRun(runId, { from_step_index: fromStepIndex });
        if (!r.ok) {
          const msg = r.error || "Could not retry failed step.";
          toast.error(msg);
          throw new Error(msg);
        }
        if (r.data.run) {
          upsertPipelineRun(r.data.run);
        }
        toast.success("Retrying failed step.");
        await Promise.all([loadPipelineRuns(), loadFiles(), loadThreads(), loadRecent(), refreshCounts()]);
        onRoutineFiredRef.current?.();
      } finally {
        pipelineOpLockRef.current.delete(runId);
      }
    },
    [upsertPipelineRun, loadPipelineRuns, loadFiles, loadThreads, loadRecent, refreshCounts]
  );

  const cancelPipeline = useCallback(
    async (runId: string): Promise<void> => {
      if (pipelineOpLockRef.current.has(runId)) return;
      pipelineOpLockRef.current.add(runId);
      try {
        const r = await cancelPipelineRun(runId);
        if (!r.ok) {
          const msg = r.error || "Could not abort pipeline.";
          toast.error(msg);
          throw new Error(msg);
        }
        if (r.data.run) {
          upsertPipelineRun(r.data.run);
        }
        toast("Pipeline aborted.");
        await Promise.all([loadPipelineRuns(), refreshCounts()]);
        onRoutineFiredRef.current?.();
      } finally {
        pipelineOpLockRef.current.delete(runId);
      }
    },
    [upsertPipelineRun, loadPipelineRuns, refreshCounts]
  );

  const launchStarterPlaybook = useCallback(
    async (starterId: string, runNow = true): Promise<StarterLaunchResult | null> => {
      const r = await call<StarterLaunchResult>("/api/playbooks/starters/launch", {
        method: "POST",
        body: { id: starterId, run_now: runNow },
      });
      if (!r.ok) {
        toast.error(r.error || "Could not launch starter playbook.");
        return null;
      }
      if (r.data.run) upsertPipelineRun(r.data.run);
      await Promise.all([loadRoutines(), loadPipelineRuns(), loadFiles(), refreshCounts()]);
      return r.data;
    },
    [upsertPipelineRun, loadRoutines, loadPipelineRuns, loadFiles, refreshCounts]
  );

  const healMemory = useCallback(
    async (mode = "all"): Promise<number | null> => {
      const r = await call<{ content?: string; removed_count?: number; health?: MemoryHealth }>(
        "/api/memory/heal",
        {
          method: "POST",
          body: { mode },
        }
      );
      if (!r.ok) {
        toast.error("Could not clean up stale memory rules.");
        return null;
      }
      if (typeof r.data.content === "string") {
        setLearnedCount(countLearnedLines(r.data.content));
      }
      if (r.data.health && typeof r.data.health === "object") {
        setMemoryHealth(r.data.health);
      }
      const removed = typeof r.data.removed_count === "number" ? r.data.removed_count : 0;
      toast.success(
        removed > 0
          ? `Pruned ${removed} stale or conflicting memory ${removed === 1 ? "rule" : "rules"}.`
          : "Memory is already clean."
      );
      return removed;
    },
    []
  );

  const loadStarters = useCallback(async (role: string) => {
    const r = await call<{ installed?: SuiteItem[] }>("/api/suite");
    const items = r.ok && Array.isArray(r.data.installed) ? r.data.installed : [];
    setStarters(buildStarterCards(items, role, filesRef.current));
    setSuiteCount(items.length);
  }, []);

  const openThread = useCallback(async (id: string) => {
    if (busyRef.current) return;
    setThread(id);
    store.set(THREAD_KEY, id);
    loadDraftFor(id);
    setPendingNote(false);
    const r = await call<Thread>("/api/threads/" + encodeURIComponent(id));
    if (!r.ok || !Array.isArray(r.data.messages)) {
      setThread(null);
      store.set(THREAD_KEY, "");
      setMessages([{ role: "error", content: "That conversation could not be opened. It may have been removed." }]);
      loadThreads();
      return;
    }
    setMessages(r.data.messages);
  }, [loadThreads]);

  const newThread = useCallback(async () => {
    if (busyRef.current) return;
    const r = await call<{ id?: string }>("/api/threads", { method: "POST", body: {} });
    if (!r.ok || typeof r.data.id !== "string") {
      setSideError("Could not start a new conversation. Is the Ergon server running?");
      return;
    }
    setSideError("");
    setThread(r.data.id);
    store.set(THREAD_KEY, r.data.id);
    loadDraftFor(r.data.id);
    setMessages([]);
    setPendingNote(false);
    loadThreads();
  }, [loadThreads]);

  const refreshPerson = useCallback(async () => {
    setThread(null);
    store.set(THREAD_KEY, "");
    setMessages([]);
    loadDraftFor(null);
    setPendingNote(false);
    await Promise.all([loadStatus(), loadPeople(), loadClients(), loadMetering(), loadWorkspacePolicy(), loadThreads(), loadRecent(), refreshCounts(), loadFiles(), loadSkills(), loadRoutines(), loadPipelineRuns()]);
  }, [loadStatus, loadPeople, loadClients, loadMetering, loadWorkspacePolicy, loadThreads, loadRecent, refreshCounts, loadFiles, loadSkills, loadRoutines, loadPipelineRuns]);

  const switchPerson = useCallback(async (id: string) => {
    if (busyRef.current) return;
    let r = hasV070Api(statusRef.current)
      ? await call("/api/people/switch", { method: "POST", body: { id, person_id: id } })
      : await call("/api/active-person", { method: "POST", body: { id } });
    if (!r.ok && (r.status === 404 || r.status === 405)) {
      r = await call("/api/active-person", { method: "POST", body: { id } });
    }
    if (!r.ok) {
      setSideError("Could not switch person. Check that Ergon is running.");
      return;
    }
    setSideError("");
    await refreshPerson();
  }, [refreshPerson]);

  const switchActiveEngagement = useCallback(
    async (engagementId: string): Promise<boolean> => {
      if (busyRef.current) return false;
      const r = await switchEngagement(engagementId);
      if (!r.ok) {
        toast.error(r.error || "Could not switch engagement.");
        return false;
      }
      setActiveEngagementId(r.data.active_engagement_id || engagementId);
      await Promise.all([loadClients(), loadMetering(), loadFiles(), loadThreads(), loadRecent(), refreshCounts(), loadTimeEntries({ engagement_id: engagementId }), loadInvoices({ engagement_id: engagementId }), loadEngagementContext(engagementId)]);
      toast.success(`Switched engagement${r.data.engagement?.name ? ` to "${r.data.engagement.name}"` : ""}.`);
      return true;
    },
    [loadClients, loadMetering, loadFiles, loadThreads, loadRecent, refreshCounts, loadTimeEntries, loadInvoices, loadEngagementContext]
  );

  const showRun = useCallback((id: string, msgs: Msg[]) => {
    setThread(id);
    store.set(THREAD_KEY, id);
    setMessages(Array.isArray(msgs) ? msgs : []);
    loadThreads();
    loadRecent();
    refreshCounts();
    loadFiles();
  }, [loadThreads, loadRecent, refreshCounts, loadFiles]);

  const handlePipelineSseEvent = useCallback(
    (eventKind: string, rawData: string) => {
      if (!rawData) return;
      let parsed: Record<string, unknown> | null = null;
      try {
        const j = JSON.parse(rawData);
        if (j && typeof j === "object") parsed = j as Record<string, unknown>;
      } catch {
        return;
      }
      if (!parsed) return;

      if (parsed.run && typeof parsed.run === "object") {
        upsertPipelineRun(parsed.run as PipelineRun);
      } else if (typeof parsed.run_id === "string" && parsed.run_id) {
        const rid = parsed.run_id;
        const stepIdx = typeof parsed.step_index === "number" ? parsed.step_index : -1;
        if ((eventKind === "pipeline_step_started" || eventKind === "pipeline_step_done") && stepIdx >= 0) {
          setPipelineRuns((prev) => {
            const idx = prev.findIndex((r) => r.run_id === rid);
            if (idx < 0) return prev;
            const cur = prev[idx];
            const nextStates = cur.step_states.slice();
            while (nextStates.length <= stepIdx) {
              nextStates.push({
                step_id: String(parsed.step_id ?? `step_${nextStates.length + 1}`),
                kind: String(parsed.kind ?? "skill"),
                status: "pending",
              });
            }
            nextStates[stepIdx] = {
              ...nextStates[stepIdx],
              step_id: String(parsed.step_id ?? nextStates[stepIdx].step_id),
              label: typeof parsed.label === "string" ? parsed.label : nextStates[stepIdx].label,
              kind: String(parsed.kind ?? nextStates[stepIdx].kind),
              status:
                eventKind === "pipeline_step_started"
                  ? "running"
                  : ((parsed.status as "ok" | "failed" | "skipped" | "waiting_human") || "ok"),
              output: typeof parsed.output === "string" ? parsed.output : nextStates[stepIdx].output,
              files: Array.isArray(parsed.files) ? parsed.files.map(String) : nextStates[stepIdx].files,
              metrics:
                parsed.metrics && typeof parsed.metrics === "object"
                  ? (parsed.metrics as Record<string, unknown>)
                  : nextStates[stepIdx].metrics,
              error: typeof parsed.error === "string" ? parsed.error : nextStates[stepIdx].error,
            };
            const updated: PipelineRun = {
              ...cur,
              current_step_index: stepIdx,
              updated_ms: typeof parsed.ts_ms === "number" ? parsed.ts_ms : Date.now(),
              step_states: nextStates,
            };
            return prev.map((r, i) => (i === idx ? updated : r));
          });
        } else {
          void getPipelineRun(rid).then((res) => {
            if (res.ok && res.data.run) upsertPipelineRun(res.data.run);
          });
        }
      }

      const uDelta = parseRunUsage(parsed.usage);
      if (uDelta) {
        setLiveRunUsage(uDelta);
      }

      if (eventKind === "pipeline_paused") {
        toast.warning("Pipeline paused at a human checkpoint.");
        void loadPipelineRuns();
        void refreshCounts();
        onRoutineFiredRef.current?.();
      } else if (eventKind === "pipeline_finished") {
        const st = String(parsed.status ?? (parsed.run as { status?: string } | undefined)?.status ?? "");
        if (st === "failed") toast.error("Pipeline run failed.");
        else if (st === "cancelled") toast("Pipeline aborted.");
        else toast.success("Pipeline finished.");
        void loadPipelineRuns();
        void loadFiles();
        void loadThreads();
        void loadRecent();
        void refreshCounts();
        void loadRoutines();
        void loadMetering();
        onRoutineFiredRef.current?.();
      } else if (eventKind === "pipeline_resumed") {
        void loadPipelineRuns();
      }
    },
    [upsertPipelineRun, loadPipelineRuns, refreshCounts, loadFiles, loadThreads, loadRecent, loadRoutines, loadMetering]
  );

  // Live SSE (/api/runs/stream) subscription with automatic fallback to /api/runs/status polling + live elapsed timer.
  const sseReady = hasV050Api(status, "sse");
  useEffect(() => {
    if (!busy) {
      setLiveSteps([]);
      setLiveTimeline([]);
      setLiveElapsedMs(0);
      setActiveRunId(null);
      activeRunIdRef.current = null;
      setLiveRunUsage(null);
      setCancelling(false);
      setLiveStreamError("");
      return;
    }

    const started = runStartedMs || Date.now();
    let cancelled = false;
    let pollTimer: number | null = null;
    let es: EventSource | null = null;
    let consecutivePollErrors = 0;

    const applyStatusPayload = (payload: Partial<LiveRunStatus>) => {
      if (cancelled) return;
      if (Array.isArray(payload.runs)) {
        const normalizedRuns = payload.runs
          .map((item) => normalizeLiveRunItem(item, started))
          .filter((x): x is LiveRun => x !== null && x.active && !x.cancelled);
        setLiveRuns(normalizedRuns);
        const primary =
          normalizedRuns.find(
            (r) =>
              (activeRunIdRef.current && r.run_id === activeRunIdRef.current) ||
              (threadRef.current && r.thread_id === threadRef.current)
          ) || normalizedRuns[0];
        if (primary) {
          activeRunIdRef.current = primary.run_id;
          setActiveRunId(primary.run_id);
          if (typeof primary.elapsed_ms === "number") setLiveElapsedMs(primary.elapsed_ms);
          const stepsArr = primary.steps as LiveRunStep[];
          setLiveTimeline(stepsArr);
          setLiveSteps(stepsArr.map((s) => s.raw));
          return;
        }
      } else if (payload.active) {
        const single = normalizeLiveRunItem(payload, started);
        if (single) {
          setLiveRuns((prev) => {
            const idx = prev.findIndex((r) => r.run_id === single.run_id);
            return idx >= 0 ? prev.map((r, i) => (i === idx ? single : r)) : [single, ...prev];
          });
        }
      }
      if (typeof payload.run_id === "string" && payload.run_id) {
        activeRunIdRef.current = payload.run_id;
        setActiveRunId(payload.run_id);
      }
      if (typeof payload.elapsed_ms === "number") {
        setLiveElapsedMs(payload.elapsed_ms);
      }
      if (Array.isArray(payload.steps)) {
        const normalized = payload.steps.map((s, idx) => normalizeStepItem(s, idx, payload.started_ms || started));
        setLiveTimeline(normalized);
        setLiveSteps(normalized.map((s) => s.raw));
      }
    };

    const startPolling = () => {
      if (cancelled || pollTimer !== null) return;
      setStreamMode("poll");
      const tick = async () => {
        const r = await call<LiveRunStatus>("/api/runs/status");
        if (cancelled) return;
        if (r.ok) {
          consecutivePollErrors = 0;
          setLiveStreamError("");
          if (r.data.active) {
            applyStatusPayload(r.data);
          } else if (Array.isArray(r.data.runs)) {
            setLiveRuns([]);
          }
          return;
        }
        consecutivePollErrors += 1;
        if (consecutivePollErrors >= 2) {
          setLiveStreamError(r.error || "Live connection lost. Could not reconcile active run status.");
        }
      };
      void tick();
      pollTimer = window.setInterval(() => void tick(), 350);
    };

    const handleSsePayload = (rawData: string, eventKind?: string) => {
      if (cancelled || !rawData) return;
      setLiveStreamError("");
      let parsed: Record<string, unknown> | null = null;
      try {
        const j = JSON.parse(rawData);
        if (j && typeof j === "object") parsed = j as Record<string, unknown>;
      } catch {
        return;
      }
      if (!parsed) return;

      const evRunId = typeof parsed.run_id === "string" ? parsed.run_id : "";
      const evThreadId = typeof parsed.thread_id === "string" ? parsed.thread_id : "";
      if (eventKind === "run_finished" || eventKind === "done" || eventKind === "cancelled") {
        if (evRunId) {
          setLiveRuns((prev) => prev.filter((r) => r.run_id !== evRunId));
        }
      }
      if (evRunId && (!activeRunIdRef.current || activeRunIdRef.current === evRunId || (evThreadId && evThreadId === threadRef.current))) {
        activeRunIdRef.current = evRunId;
        setActiveRunId(evRunId);
      }
      if (typeof parsed.elapsed_ms === "number") {
        setLiveElapsedMs(parsed.elapsed_ms);
      }
      const uDelta = parseRunUsage(parsed.usage);
      if (uDelta) {
        setLiveRunUsage(uDelta);
      }
      if (Array.isArray(parsed.runs) || Array.isArray(parsed.steps)) {
        applyStatusPayload(parsed as Partial<LiveRunStatus>);
        return;
      }

      const tool = String(parsed.tool ?? "").trim();
      const rawStep = String(parsed.raw ?? parsed.detail ?? parsed.message ?? (tool ? `Tool: ${tool}` : "")).trim();
      if (!tool && !rawStep) {
        if (eventKind === "run_started" && evRunId) {
          const startedRun = normalizeLiveRunItem(parsed, started);
          if (startedRun) {
            setLiveRuns((prev) => {
              const idx = prev.findIndex((r) => r.run_id === startedRun.run_id);
              return idx >= 0 ? prev : [startedRun, ...prev];
            });
          }
        }
        return;
      }

      const stepStatus: LiveRunStep["status"] =
        eventKind === "tool_start" || eventKind === "step_start" || eventKind === "run_started" || parsed.status === "running"
          ? "running"
          : eventKind === "approval" || eventKind === "approval_parked" || parsed.status === "waiting"
            ? "waiting"
            : eventKind === "error" || parsed.status === "error"
              ? "error"
              : "done";

      const incoming: LiveRunStep = {
        id: String(parsed.step_id ?? parsed.id ?? `${tool || "step"}-${Date.now()}`),
        tool,
        label: String(parsed.label ?? (tool ? toolLabel(tool, undefined, rawStep) || tool : rawStep)),
        status: stepStatus,
        elapsed_ms: typeof parsed.elapsed_ms === "number" ? parsed.elapsed_ms : Math.max(1, Date.now() - started),
        raw: rawStep || `Tool: ${tool}`,
        file: typeof parsed.file === "string" ? parsed.file : typeof parsed.path === "string" ? parsed.path : undefined,
      };

      if (evRunId) {
        setLiveRuns((prev) => {
          const rIdx = prev.findIndex((r) => r.run_id === evRunId);
          if (rIdx < 0) {
            return [
              {
                active: true,
                run_id: evRunId,
                thread_id: evThreadId,
                task: String(parsed.task ?? parsed.routine_name ?? "Active run"),
                started_ms: started,
                steps: [incoming],
              },
              ...prev,
            ];
          }
          return prev.map((r, i) => {
            if (i !== rIdx) return r;
            const curSteps = r.steps as LiveRunStep[];
            const sIdx = curSteps.findIndex((p) => (incoming.id && p.id === incoming.id) || (p.status === "running" && p.tool === incoming.tool));
            const nextSteps = sIdx >= 0 ? curSteps.map((it, j) => (j === sIdx ? { ...it, ...incoming } : it)) : [...curSteps, incoming];
            return { ...r, steps: nextSteps };
          });
        });
      }

      if (!evRunId || !activeRunIdRef.current || evRunId === activeRunIdRef.current) {
        setLiveTimeline((prev) => {
          const idx = prev.findIndex((p) => (incoming.id && p.id === incoming.id) || (p.status === "running" && p.tool === incoming.tool));
          const next = idx >= 0 ? prev.map((item, i) => (i === idx ? { ...item, ...incoming } : item)) : [...prev, incoming];
          setLiveSteps(next.map((s) => s.raw));
          return next;
        });
      }
    };

    if (sseReady && typeof EventSource !== "undefined") {
      try {
        es = new EventSource("/api/runs/stream");
        setStreamMode("sse");
        es.onmessage = (ev) => handleSsePayload(ev.data, "message");
        for (const evName of [
          "status",
          "run_started",
          "step",
          "step_start",
          "step_done",
          "tool_start",
          "tool_done",
          "file_write",
          "file_written",
          "approval",
          "approval_parked",
          "run_cancelling",
          "run_finished",
          "done",
          "cancelled",
          "error",
        ]) {
          es.addEventListener(evName, (ev) => handleSsePayload((ev as MessageEvent).data, evName));
        }
        for (const pEv of [
          "pipeline_step_started",
          "pipeline_step_done",
          "pipeline_paused",
          "pipeline_resumed",
          "pipeline_finished",
        ]) {
          es.addEventListener(pEv, (ev) => handlePipelineSseEvent(pEv, (ev as MessageEvent).data));
        }
        es.onerror = () => {
          if (es) {
            es.close();
            es = null;
          }
          startPolling();
        };
      } catch {
        startPolling();
      }
    } else {
      startPolling();
    }

    return () => {
      cancelled = true;
      if (pollTimer !== null) window.clearInterval(pollTimer);
      if (es) es.close();
    };
  }, [busy, runStartedMs, sseReady]);

  const cancelRun = useCallback(async (targetRunId?: string) => {
    const specificId = typeof targetRunId === "string" && targetRunId.trim() ? targetRunId.trim() : "";
    const runTarget = specificId || activeRunIdRef.current || threadRef.current || "active";
    const isPrimaryChatRun =
      !specificId ||
      specificId === "active" ||
      specificId === activeRunIdRef.current ||
      specificId === threadRef.current;

    if (!specificId && !busyRef.current) return;
    if (isPrimaryChatRun) setCancelling(true);
    setCancellingRunIds((prev) => ({ ...prev, [runTarget]: true }));
    try {
      if (hasV050Api(statusRef.current, "cancel") || specificId) {
        const r = await cancelLiveRun(runTarget);
        if (!r.ok && r.status !== 404) {
          const msg = r.error || "Could not stop the active run. Try again.";
          setLiveStreamError(msg);
          toast.error(msg);
          return;
        }
      }
      setLiveStreamError("");
      setLiveRuns((prev) => prev.filter((r) => r.run_id !== runTarget && r.thread_id !== runTarget));
      if (isPrimaryChatRun && busyRef.current) {
        abortRef.current?.abort();
        abortRef.current = null;
        busyRef.current = false;
        setBusy(false);
      }
      toast("Run cancelled.");
    } finally {
      if (isPrimaryChatRun) setCancelling(false);
      setCancellingRunIds((prev) => {
        const next = { ...prev };
        delete next[runTarget];
        return next;
      });
    }
  }, []);

  const sendPrompt = useCallback(
    async (rawText: string) => {
      if (busyRef.current) return;
      const text = rawText.trim();
      if (!text) return;
      const cur = threadRef.current;
      const ctrl = new AbortController();
      abortRef.current = ctrl;
      busyRef.current = true;
      setBusy(true);
      setRunStartedMs(Date.now());
      setLiveElapsedMs(0);
      setLiveSteps([]);
      setLiveTimeline([]);
      setPendingNote(false);
      setDraft("");
      store.set(PENDING, JSON.stringify({ thread: cur || "new", text }));
      setMessages((m) => [...m, { role: "user", content: text }]);
      const r = await call<Thread & { thread_id?: string }>("/api/chat", {
        method: "POST",
        body: { thread_id: cur || "", message: text },
        signal: ctrl.signal,
      });
      abortRef.current = null;
      store.set(PENDING, "");
      busyRef.current = false;
      setBusy(false);
      setLiveSteps([]);
      setLiveTimeline([]);
      if (r.ok && Array.isArray(r.data.messages)) {
        const id = r.data.id || r.data.thread_id || cur;
        setThread(id);
        if (id) store.set(THREAD_KEY, id);
        setMessages(r.data.messages);
        setRunFailed(false);
        if ((r.data as { stopped_for_safety?: boolean }).stopped_for_safety) {
          toast.warning("Ergon stopped this task to keep your work safe. The details are waiting in Today.");
        }
        if ((r.data as { budget_gated?: boolean }).budget_gated) {
          toast.warning("Retainer budget limit reached. Approve overrun in Today to continue.");
        }
        const pausedRunId = (r.data as { paused_pipeline_run_id?: string }).paused_pipeline_run_id;
        if (pausedRunId) {
          void loadPipelineRuns();
          toast.warning("Task paused at a human checkpoint.");
        }
      } else if (!r.ok && r.status === 499) {
        setMessages((m) => [...m, { role: "step", content: "Note: Run cancelled by you before completion." }]);
      } else {
        const errMsg = !r.ok
          ? (r.status === 0 ? "Could not reach the Ergon server." : r.error || `The request failed (HTTP ${r.status}).`)
          : "The agent returned an unexpected response.";
        setMessages((m) => [...m, { role: "error", content: errMsg }]);
        setRunFailed(true);
      }
      loadThreads();
      loadRecent();
      refreshCounts();
      loadFiles();
      loadMetering();
    },
    [loadThreads, loadRecent, refreshCounts, loadFiles, loadPipelineRuns, loadMetering]
  );

  const send = useCallback(async () => {
    await sendPrompt(draft);
  }, [draft, sendPrompt]);

  const runRoutineNow = useCallback(
    async (routine: Routine) => {
      if (busyRef.current) return;
      if (routine.steps_v2 && routine.steps_v2.length > 0 && hasPipelinesApi(statusRef.current)) {
        await startPipeline({
          routine_id: routine.id,
          name: routine.name,
          steps_v2: routine.steps_v2,
          thread_id: threadRef.current || undefined,
        });
        await loadRoutines();
        return;
      }
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call<{ id?: string; thread_id?: string; messages?: Msg[]; outcome?: string }>(
          `/api/routines/${encodeURIComponent(routine.id)}/run`,
          { method: "POST", body: {} }
        );
        if (r.ok) {
          toast.success(r.data.outcome || `Ran routine "${routine.name}".`);
          const tid = r.data.id || r.data.thread_id;
          if (tid && Array.isArray(r.data.messages)) {
            showRun(tid, r.data.messages);
          } else {
            loadThreads();
            loadRecent();
            refreshCounts();
            loadFiles();
          }
          await loadRoutines();
          return;
        }
      }
      // Graceful fallback: update local routine run metadata and execute its prompt live.
      const updated = loadLocalRoutines().map((x) =>
        x.id === routine.id ? { ...x, last_run_ts: Date.now(), last_status: "finished", run_count: (x.run_count ?? 0) + 1 } : x
      );
      saveLocalRoutines(updated);
      setRoutines((prev) =>
        prev.map((x) =>
          x.id === routine.id ? { ...x, last_run_ts: Date.now(), last_status: "finished", run_count: (x.run_count ?? 0) + 1 } : x
        )
      );
      await sendPrompt(routine.prompt);
    },
    [showRun, loadThreads, loadRecent, refreshCounts, loadFiles, loadRoutines, sendPrompt]
  );

  const catchupRoutine = useCallback(
    async (routine: Routine): Promise<boolean> => {
      if (busyRef.current) return false;
      if (hasV070Api(statusRef.current)) {
        const r = await call<{
          caught_up?: boolean;
          missed_windows_cleared?: number;
          routine?: unknown;
        }>(`/api/routines/${encodeURIComponent(routine.id)}/catchup`, { method: "POST", body: {} });
        if (r.ok) {
          const cleared = r.data.missed_windows_cleared ?? 0;
          toast.success(
            r.data.caught_up
              ? `Caught up "${routine.name}" (${cleared || 1} missed window${cleared === 1 ? "" : "s"} cleared).`
              : `"${routine.name}" is already up to date.`
          );
          await loadRoutines();
          loadThreads();
          loadRecent();
          refreshCounts();
          loadFiles();
          return true;
        }
        if (r.status !== 404 && r.status !== 405) {
          toast.error(r.error || `Could not catch up "${routine.name}".`);
          return false;
        }
      }
      await runRoutineNow(routine);
      return true;
    },
    [loadRoutines, loadThreads, loadRecent, refreshCounts, loadFiles, runRoutineNow]
  );

  const catchupAllRoutines = useCallback(async (): Promise<boolean> => {
    if (busyRef.current) return false;
    if (hasV070Api(statusRef.current)) {
      const r = await call<{ caught_up_count?: number; failed_count?: number; total_due?: number }>(
        "/api/scheduler/catchup",
        { method: "POST", body: {} }
      );
      if (r.ok) {
        const count = r.data.caught_up_count ?? 0;
        const failed = r.data.failed_count ?? 0;
        if (failed > 0) {
          toast.warning(`Caught up ${count} routine(s); ${failed} failed (check model connection).`);
        } else {
          toast.success(count > 0 ? `Caught up ${count} due routine(s) with verified receipts.` : "All routines are up to date.");
        }
        await loadRoutines();
        loadThreads();
        loadRecent();
        refreshCounts();
        loadFiles();
        return failed === 0;
      }
      toast.error(r.error || "Could not catch up missed routines.");
      return false;
    }
    return false;
  }, [loadRoutines, loadThreads, loadRecent, refreshCounts, loadFiles]);

  useEffect(() => {
    (async () => {
      await loadStatus();
      loadPeople();
      loadClients();
      loadMetering();
      loadThreads();
      loadRecent();
      refreshCounts();
      loadFiles();
      loadSkills();
      loadRoutines();
      loadPipelineRuns();
      const pend = (() => { try { const v = JSON.parse(store.get(PENDING) || "null"); return v && typeof v.text === "string" ? v : null; } catch { return null; } })();
      const reopen = store.get(THREAD_KEY);
      if (reopen) await openThread(reopen);
      else { loadDraftFor(null); setMessages([]); }
      if (!pend) return;
      store.set(PENDING, "");
      let landed = false;
      if (pend.thread !== "new") {
        const t = await call<Thread>("/api/threads/" + encodeURIComponent(pend.thread));
        landed = t.ok && Array.isArray(t.data.messages) && t.data.messages.some((m) => m.role === "user" && String(m.content).trim() === pend.text);
      }
      if (landed) return;
      setDraft(pend.text);
      setPendingNote(true);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setOnRoutineFired = useCallback((fn: (() => void) | null) => {
    onRoutineFiredRef.current = fn;
  }, []);

  const reconcileLiveRuns = useCallback(async () => {
    const r = await call<LiveRunStatus>("/api/runs/status");
    if (!r.ok) {
      setLiveStreamError(r.error || "Could not reconcile live run status.");
      return;
    }
    setLiveStreamError("");
    if (Array.isArray(r.data.runs)) {
      const normalizedRuns = r.data.runs
        .map((item) => normalizeLiveRunItem(item, Date.now()))
        .filter((x): x is LiveRun => x !== null && x.active && !x.cancelled);
      setLiveRuns(normalizedRuns);
    } else if (!r.data.active) {
      setLiveRuns([]);
    }
    await Promise.all([loadPipelineRuns(), loadRoutines()]);
  }, [loadPipelineRuns, loadRoutines]);

  // Listen for background scheduler `routine_fired` and `pipeline_*` SSE notifications when idle.
  useEffect(() => {
    if (busy || (!hasV070Api(status, "scheduler") && !hasPipelinesApi(status)) || typeof EventSource === "undefined") return;
    let closed = false;
    let es: EventSource | null = null;
    let reconnectTimer: number | null = null;
    let fallbackPollTimer: number | null = null;
    let backoffMs = 2000;

    const reconcileIdle = async () => {
      if (closed) return;
      const r = await call<LiveRunStatus>("/api/runs/status");
      if (closed) return;
      if (r.ok) {
        setLiveStreamError("");
        if (Array.isArray(r.data.runs)) {
          const normalizedRuns = r.data.runs
            .map((item) => normalizeLiveRunItem(item, Date.now()))
            .filter((x): x is LiveRun => x !== null && x.active && !x.cancelled);
          setLiveRuns(normalizedRuns);
        } else if (!r.data.active) {
          setLiveRuns([]);
        }
      } else if (
        liveRunsRef.current.length > 0 ||
        pipelineRunsRef.current.some((p) => p.status === "running")
      ) {
        setLiveStreamError(r.error || "Live stream disconnected. Could not refresh active run status.");
      }
      if (hasPipelinesApi(statusRef.current)) {
        await loadPipelineRuns();
      }
    };

    const stopFallbackPoll = () => {
      if (fallbackPollTimer !== null) {
        window.clearInterval(fallbackPollTimer);
        fallbackPollTimer = null;
      }
    };

    const startFallbackPoll = () => {
      if (closed || fallbackPollTimer !== null) return;
      fallbackPollTimer = window.setInterval(() => {
        if (
          liveRunsRef.current.length > 0 ||
          pipelineRunsRef.current.some((p) => p.status === "running")
        ) {
          void reconcileIdle();
        }
      }, 2000);
    };

    const openIdleStream = () => {
      if (closed) return;
      try {
        es = new EventSource("/api/runs/stream");
        es.onopen = () => {
          backoffMs = 2000;
          setLiveStreamError("");
          stopFallbackPoll();
        };
        const onFired = (ev: Event) => {
          if (closed) return;
          setLiveStreamError("");
          let label = "Scheduled routine";
          try {
            const parsed = JSON.parse((ev as MessageEvent).data || "{}") as Record<string, unknown>;
            if (typeof parsed.name === "string" && parsed.name) label = parsed.name;
            else if (typeof parsed.routine_name === "string" && parsed.routine_name) label = parsed.routine_name;
          } catch { /* ignore */ }
          toast.success(`Background routine "${label}" finished.`);
          setRoutineHistoryRev((n) => n + 1);
          loadRoutines();
          loadFiles();
          loadThreads();
          loadRecent();
          refreshCounts();
          onRoutineFiredRef.current?.();
        };
        es.addEventListener("routine_fired", onFired);
        for (const uEv of ["status", "run_started", "step", "step_start", "step_done", "tool_start", "tool_done", "run_finished", "done", "cancelled"]) {
          es.addEventListener(uEv, (ev) => {
            if (closed) return;
            setLiveStreamError("");
            try {
              const parsed = JSON.parse((ev as MessageEvent).data || "{}") as Record<string, unknown>;
              if (Array.isArray(parsed.runs)) {
                const normalizedRuns = parsed.runs
                  .map((item) => normalizeLiveRunItem(item, Date.now()))
                  .filter((x): x is LiveRun => x !== null && x.active && !x.cancelled);
                setLiveRuns(normalizedRuns);
              } else if (uEv === "run_finished" || uEv === "done" || uEv === "cancelled") {
                const rid = typeof parsed.run_id === "string" ? parsed.run_id : "";
                if (rid) setLiveRuns((prev) => prev.filter((r) => r.run_id !== rid));
              } else if (uEv === "run_started" || uEv === "status") {
                const startedRun = normalizeLiveRunItem(parsed, Date.now());
                if (startedRun && startedRun.active) {
                  setLiveRuns((prev) => {
                    const idx = prev.findIndex((r) => r.run_id === startedRun.run_id);
                    return idx >= 0 ? prev.map((r, i) => (i === idx ? startedRun : r)) : [startedRun, ...prev];
                  });
                }
              }
              const uDelta = parseRunUsage(parsed.usage);
              if (uDelta) setLiveRunUsage(uDelta);
              if (uEv === "run_finished" || uEv === "done") {
                void loadMetering();
              }
            } catch { /* ignore */ }
          });
        }
        for (const pEv of [
          "pipeline_step_started",
          "pipeline_step_done",
          "pipeline_paused",
          "pipeline_resumed",
          "pipeline_finished",
        ]) {
          es.addEventListener(pEv, (ev) => {
            if (!closed) {
              setLiveStreamError("");
              handlePipelineSseEvent(pEv, (ev as MessageEvent).data);
            }
          });
        }
        es.onerror = () => {
          if (es) {
            es.close();
            es = null;
          }
          if (closed) return;
          void reconcileIdle();
          startFallbackPoll();
          const waitMs = backoffMs;
          backoffMs = Math.min(10_000, backoffMs * 2);
          reconnectTimer = window.setTimeout(openIdleStream, waitMs);
        };
      } catch {
        startFallbackPoll();
      }
    };

    openIdleStream();
    return () => {
      closed = true;
      if (reconnectTimer !== null) window.clearTimeout(reconnectTimer);
      stopFallbackPoll();
      if (es) es.close();
    };
  }, [busy, status, loadRoutines, loadFiles, loadThreads, loadRecent, refreshCounts, loadMetering, handlePipelineSseEvent, loadPipelineRuns]);

  useEffect(() => {
    if (status && status !== "error") {
      loadStarters(String(status.person?.role ?? ""));
      loadSkills();
      loadRoutines();
      loadPipelineRuns();
      loadClients();
      loadMetering();
      loadWorkspacePolicy();
      loadTimeEntries();
      loadInvoices();
      loadMcpServers();
    }
  }, [status, files, loadStarters, loadSkills, loadRoutines, loadPipelineRuns, loadClients, loadMetering, loadWorkspacePolicy, loadTimeEntries, loadInvoices, loadMcpServers]);

  useEffect(() => {
    if (status && status !== "error" && activeEngagementId) {
      void loadEngagementContext(activeEngagementId);
      void loadTimeEntries({ engagement_id: activeEngagementId });
      void loadInvoices({ engagement_id: activeEngagementId });
    } else if (!activeEngagementId) {
      setActiveEngagementContext(null);
    }
  }, [status, activeEngagementId, loadEngagementContext, loadTimeEntries, loadInvoices]);

  return {
    status, runFailed, people, threads, threadsError, states, threadId, messages,
    busy, activeRunId, runStartedMs, liveElapsedMs, streamMode, cancelling, cancellingRunIds, liveRuns, liveStreamError, reconcileLiveRuns, liveSteps, liveTimeline,
    clients, engagements, activeEngagementId, activeEngagementContext, timeEntries, invoices, workspacePolicy, engagementScopeOnly, setEngagementScopeOnly, metering, liveRunUsage,
    draft, setDraft, pendingNote, sideError, starters, recovered, recent,
    todayCount, learnedCount, memoryHealth, suiteCount, files, lastUploaded, setLastUploaded, skills, routines, routinesLoading, routinesError, routineSaveError, pipelineRuns, scheduler, routineHistoryRev,
    send, sendPrompt, cancelRun, openThread, newThread, switchPerson, refreshPerson, showRun,
    loadThreads, loadStatus, loadPeople, loadClients, loadMetering, loadWorkspacePolicy, updateWorkspacePolicyState,
    mcpServers, loadMcpServers, saveMcpServerConfig, discoverMcpServerTools, removeMcpServer,
    loadEngagementContext, loadTimeEntries, loadInvoices,
    createNewClient, saveClientCrm, createNewEngagement, saveEngagementSow, switchActiveEngagement,
    addTimeEntry, removeTimeEntry, createGeneratedInvoice, setInvoiceStatus,
    refreshCounts, healMemory, loadRecent, loadStarters,
    loadFiles, uploadFile, uploadFiles, loadSkills, starterPlaybooks, launchStarterPlaybook, loadRoutines, createRoutine, deleteRoutine, importPlaybook, previewPlaybook,
    loadPipelineRuns, startPipeline, resumePipeline, retryPipeline, cancelPipeline,
    toggleRoutine, loadRoutineHistory, runRoutineNow, catchupRoutine, catchupAllRoutines, setOnRoutineFired,
    setSideError, setRecovered,
  };
}
