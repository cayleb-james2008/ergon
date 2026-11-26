// Ergon server JSON API client. Every failed or malformed call resolves to { ok: false }, never an uncaught throw.
export type Msg = { role: string; content: string };
export type Person = { id: string; name: string; role?: string };
export type ServerCapabilities = {
  sse?: boolean;
  skills?: boolean;
  routines?: boolean;
  playbooks?: boolean;
  search?: boolean;
  receipts_export?: boolean;
  cancel?: boolean;
  file_history?: boolean;
  scheduler?: boolean;
  audit_bundle?: boolean;
  people_switch?: boolean;
  tripwire?: boolean;
  pipelines?: boolean;
  file_edit?: boolean;
  table_query?: boolean;
  pdf_export?: boolean;
  connectors?: boolean;
  clients?: boolean;
  metering?: boolean;
  vault_search?: boolean;
};

export type Client = {
  id: string;
  name: string;
  brand_header?: string;
  billing_currency?: string;
  default_rate_usd?: number;
  created_ms?: number;
  updated_ms?: number;
};

export type Engagement = {
  id: string;
  client_id: string;
  person_id?: string;
  name: string;
  retainer_hours?: number;
  retainer_budget_usd?: number;
  hourly_rate_usd?: number;
  period?: string;
  status?: string;
  created_ms?: number;
  updated_ms?: number;
};

export type RunUsage = {
  model_id?: string;
  prompt_tokens: number;
  completion_tokens: number;
  cached_tokens?: number;
  total_tokens: number;
  cost_usd: number;
  latency_ms?: number;
  estimated_minutes_saved?: number;
  billable_value_usd?: number;
  engagement_id?: string;
};

export type ModelPricingEntry = {
  input_per_1m_usd: number;
  output_per_1m_usd: number;
  cached_input_per_1m_usd?: number;
};

export type ModelPricingMap = Record<string, ModelPricingEntry>;

export type MeteringTotals = {
  runs: number;
  prompt_tokens: number;
  completion_tokens: number;
  cached_tokens?: number;
  total_tokens: number;
  cost_usd: number;
  minutes_saved: number;
  billable_value_usd: number;
  retainer_burn_pct: number;
  retainer_budget_usd?: number;
  net_roi_usd?: number;
  roi_multiple?: number;
};

export type MeteringByEngagement = {
  engagement_id: string;
  engagement_name?: string;
  client_id?: string;
  client_name?: string;
  runs: number;
  total_tokens: number;
  cost_usd: number;
  minutes_saved: number;
  billable_value_usd: number;
  retainer_budget_usd?: number;
  retainer_burn_pct?: number;
};

export type MeteringByModel = {
  model_id: string;
  runs: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cost_usd: number;
};

export type MeteringDailyPoint = {
  date: string;
  runs: number;
  total_tokens: number;
  cost_usd: number;
  minutes_saved: number;
  billable_value_usd: number;
};

export type MeteringSummary = {
  totals: MeteringTotals;
  by_engagement: MeteringByEngagement[];
  by_model: MeteringByModel[];
  daily_series: MeteringDailyPoint[];
  active_engagement?: Engagement | null;
  active_client?: Client | null;
};
export type SchedulerStatus = {
  running?: boolean;
  enabled?: boolean;
  disabled_by_env?: boolean;
  tick_secs?: number;
  active_routines?: number;
  paused_routines?: number;
  due_count?: number;
  missed_windows_total?: number;
  next_due_ms?: number | null;
  last_tick_ms?: number | null;
};
export type Status = {
  version: string;
  model_configured: boolean;
  model?: string;
  model_base_url?: string;
  is_loopback_model?: boolean;
  auto_detected?: boolean;
  engine_mode?: "native" | "browser" | "byok";
  capabilities?: ServerCapabilities;
  scheduler?: SchedulerStatus;
  scheduler_enabled?: boolean;
  person?: Person;
  store_recovered?: string[];
};
export type DetectedGateway = {
  base_url: string;
  model_id: string;
  label: string;
  models?: string[];
};
export type ModelConfigPayload = {
  model_base_url: string;
  model_id: string;
  has_api_key: boolean;
  model_configured: boolean;
  is_loopback: boolean;
  auto_detected?: boolean;
  engine_mode?: "native" | "browser" | "byok";
  detected_gateways?: DetectedGateway[];
  model_pricing?: ModelPricingMap;
};
export type WorkspaceFile = {
  path: string;
  name: string;
  size: number;
  mtime_ms: number;
  ext: string;
};
export type WorkspaceFileContent = {
  path: string;
  name: string;
  size: number;
  mtime_ms: number;
  ext: string;
  is_binary: boolean;
  content: string;
  sha256?: string;
  /** PDF, Word, Excel and PowerPoint files preview as their extracted text (v0.9.0+). */
  extracted?: { kind: string; format: "markdown" | "csv"; sheets?: string[]; sheet?: string | null; pages?: number };
  extract_error?: string;
};

/** Largest file the desk uploads (matches the server). */
export const MAX_UPLOAD_BYTES = 25 * 1024 * 1024;

/** A file as an upload body, byte-exact: every file travels base64-encoded,
 *  so PDFs, Office files, images and non-UTF-8 CSVs arrive uncorrupted. */
export async function fileUploadBody(file: File): Promise<{ content: string; encoding: "base64" }> {
  const buf = new Uint8Array(await file.arrayBuffer());
  let bin = "";
  const chunk = 0x8000;
  for (let i = 0; i < buf.length; i += chunk) {
    bin += String.fromCharCode(...buf.subarray(i, i + chunk));
  }
  return { content: btoa(bin), encoding: "base64" };
}

/** Whether a workspace file can be saved as Word (.md/.txt) or Excel (.csv/.tsv). */
export function exportTarget(path: string): "docx" | "xlsx" | null {
  const ext = (path.split(".").pop() || "").toLowerCase();
  if (ext === "md" || ext === "markdown" || ext === "txt") return "docx";
  if (ext === "csv" || ext === "tsv") return "xlsx";
  return null;
}
export type LiveRunStep = {
  id?: string;
  step_id?: string;
  tool: string;
  label?: string;
  status: "running" | "done" | "error" | "waiting";
  elapsed_ms?: number;
  started_ms?: number;
  finished_ms?: number;
  raw: string;
  file?: string;
  depends_on?: string[];
  child_items?: PipelineChildItemState[];
};
export type LiveRunStatus = {
  active: boolean;
  run_id?: string;
  thread_id: string;
  task: string;
  started_ms: number;
  elapsed_ms?: number;
  steps: (string | LiveRunStep)[];
  cancelled?: boolean;
};
export type FileRevision = {
  rev_id: string;
  path: string;
  ts: number;
  size: number;
  sha256?: string;
  reason?: string;
  content?: string;
  preview?: string;
};
export type FileDiffLine = {
  kind: "context" | "add" | "del" | "hunk";
  text: string;
  old_line?: number | null;
  new_line?: number | null;
};
export type FileDiffHunk = {
  header?: string;
  old_start?: number;
  old_count?: number;
  new_start?: number;
  new_count?: number;
  lines: FileDiffLine[];
};
export type FileDiffPayload = {
  path?: string;
  rev_a?: string;
  rev_b?: string;
  added: number;
  removed: number;
  unchanged?: number;
  hunks: FileDiffHunk[];
  unified?: string;
};
export type PipelineStepKind = "skill" | "agent" | "checkpoint" | "export" | "deliver";
export type PipelineCheckpointInputMode = "approve_reject" | "choice" | "text";
export type PipelineWhen = "always" | "on_anomaly" | "on_previous_ok" | "on_all_ok" | (string & {});

export type PipelineCheckpointSpec = {
  prompt: string;
  input_mode: PipelineCheckpointInputMode;
  choices?: string[];
  inspect_files?: string[];
};

export type PipelineExportSpec = {
  path: string;
  format: "docx" | "xlsx" | "pdf";
  output?: string | null;
};

export type PipelineDeliverSpec = {
  connector_id: string;
  payload?: unknown;
  file?: string;
};

export type PipelineStepSpec = {
  id: string;
  label: string;
  kind: PipelineStepKind;
  skill_id?: string;
  args?: Record<string, string>;
  prompt?: string;
  checkpoint?: PipelineCheckpointSpec;
  export?: PipelineExportSpec;
  deliver?: PipelineDeliverSpec;
  when?: PipelineWhen;
  depends_on?: string[];
  for_each?: string;
};

export type PipelineChildItemState = {
  step_id: string;
  index?: number;
  item?: string;
  status: "ok" | "waiting_human" | "failed" | "skipped" | "pending" | "running";
  started_ms?: number;
  finished_ms?: number;
  output?: string;
  files?: string[];
  metrics?: Record<string, unknown>;
  error?: string | null;
};

export type PipelineStepState = {
  step_id: string;
  label?: string;
  kind: PipelineStepKind | string;
  status: "ok" | "waiting_human" | "failed" | "skipped" | "pending" | "running";
  started_ms?: number;
  finished_ms?: number;
  output?: string;
  files?: string[];
  metrics?: Record<string, unknown>;
  checkpoint_id?: string | null;
  human_answer?: string | null;
  human_action?: "approve" | "reject" | "answer" | null;
  error?: string | null;
  usage?: RunUsage;
  depends_on?: string[];
  child_items?: PipelineChildItemState[];
};

export type PipelinePendingCheckpoint = {
  checkpoint_id: string;
  step_index: number;
  step_id?: string;
  prompt: string;
  input_mode: PipelineCheckpointInputMode;
  choices?: string[];
  inspect_files?: string[];
  created_ms?: number;
  kind?: string;
  deliver?: PipelineDeliverSpec;
  connector_id?: string;
  connector_name?: string;
  delivery_url?: string;
  payload_preview?: string;
};

export type PipelineAgentSnapshot = {
  messages: unknown[];
  written: string[];
  budget_used: number;
};

export type PipelineRunStatus =
  | "running"
  | "paused_checkpoint"
  | "interrupted"
  | "completed"
  | "failed"
  | "cancelled";

export type PipelineRun = {
  run_id: string;
  routine_id?: string | null;
  routine_name?: string;
  thread_id?: string;
  status: PipelineRunStatus;
  current_step_index: number;
  created_ms: number;
  updated_ms: number;
  inputs?: Record<string, string>;
  steps?: PipelineStepSpec[];
  steps_v2?: PipelineStepSpec[];
  step_states: PipelineStepState[];
  pending_checkpoint?: PipelinePendingCheckpoint | null;
  agent_snapshot?: PipelineAgentSnapshot | null;
  error?: string | null;
  engagement_id?: string | null;
  usage?: RunUsage;
};

export type PipelineValidationResult = {
  ok: boolean;
  valid: boolean;
  errors: string[];
  topological_order: string[];
  stages?: string[][];
};

export type PipelineResumePayload = {
  action: "approve" | "reject" | "answer";
  answer?: string;
  choice?: string;
  override_args?: Record<string, unknown>;
};

export type RoutineRunRecord = {
  run_id: string;
  routine_id: string;
  started_ms: number;
  finished_ms?: number | null;
  status: string;
  receipt_id?: string | null;
  thread_id?: string | null;
  summary?: string;
};
export type Routine = {
  id: string;
  name: string;
  prompt: string;
  schedule: string;
  cadence?: string;
  interval_secs?: number | null;
  skill_id?: string;
  enabled: boolean;
  last_run_ts?: number | null;
  next_run_ms?: number | null;
  missed_windows?: number;
  last_status?: string;
  last_run_verified?: "verified" | "drift" | null;
  last_run_receipt_sha256?: string | null;
  last_run_receipt_id?: string | null;
  run_count?: number;
  created_ts?: number;
  steps?: string[];
  steps_v2?: PipelineStepSpec[];
  expected_inputs?: string[];
  pack_deliverables?: boolean;
  recent_runs?: RoutineRunRecord[];
};
export type MemoryIssue = {
  line: number;
  kind: "conflict" | "duplicate" | "stale_ref";
  text: string;
  detail: string;
  related_line?: number | null;
  removable?: boolean;
};
export type MemoryHealth = {
  status: "healthy" | "issues";
  learned_count: number;
  issue_count: number;
  conflicts: number;
  duplicates: number;
  stale_refs: number;
  issues: MemoryIssue[];
};
export type WorkspacePolicyProfile = "fast_internal" | "strict_client" | "air_gapped";
export type WorkspacePolicy = {
  person_id?: string;
  profile: WorkspacePolicyProfile;
  label: string;
  summary: string;
  strict_citation_gate: boolean;
  approval_gate_mode: string;
  redact_host_paths: boolean;
  block_outbound_fetch: boolean;
  updated_ms?: number;
};
export type HandoffLeakFinding = {
  kind: string;
  severity: string;
  file: string;
  line: number;
  excerpt: string;
  fix: string;
};
export type HandoffLeakScan = {
  clean: boolean;
  leak_count: number;
  host_path_count: number;
  secret_count: number;
  env_snippet_count: number;
  pii_count: number;
  findings: HandoffLeakFinding[];
};
export type HandoffDryRunResult = {
  ok: boolean;
  dry_run: boolean;
  would_block: boolean;
  policy: WorkspacePolicy;
  scan: HandoffLeakScan;
};
export type PlaybookPreviewStep = {
  step: number;
  prompt: string;
  skill_id?: string | null;
  mode: "read_only" | "read_and_write";
  referenced_paths: string[];
  predicted_outputs: string[];
};
export type PlaybookFileImpact = {
  path: string;
  role: string;
  step?: string | null;
  exists_in_workspace: boolean;
  history_revisions: number;
  reversible: boolean;
};
export type PlaybookPreviewResult = {
  routine_id: string;
  name: string;
  schedule: string;
  dry_run: boolean;
  zero_side_effects: boolean;
  step_count: number;
  steps: PlaybookPreviewStep[];
  expected_inputs: string[];
  matched_workspace_files: string[];
  file_impacts: PlaybookFileImpact[];
  pack_deliverables: boolean;
  rollback: {
    safe: boolean;
    badge: string;
    history_snapshots_enabled: boolean;
    existing_snapshots: number;
    reversible_paths: number;
    note: string;
  };
  generated_ms: number;
};
export type StarterPlaybook = {
  id: string;
  name: string;
  summary: string;
  schedule: string;
  skill_id: string;
  prompt: string;
  steps: string[];
  steps_v2?: PipelineStepSpec[];
  expected_inputs: string[];
  pack_deliverables: boolean;
  sample_title?: string;
  sample_sections?: string[];
  sha256?: string;
  verified?: boolean;
};
export type StarterLaunchResult = {
  ok: boolean;
  starter_id: string;
  sha256?: string;
  routine?: Routine;
  run?: PipelineRun;
  created_files?: string[];
  seeded_inputs?: string[];
  summary?: string;
  preview?: PlaybookPreviewResult | null;
};
export type VaultSearchMode = "hybrid" | "bm25" | "exact";
export type VaultSearchScope = "all" | "files" | "receipts" | "memory";

export type VaultChunkHit = {
  chunk_id: string;
  score: number;
  bm25_score?: number;
  semantic_score?: number;
  page?: number | null;
  sheet?: string | null;
  line_start: number;
  line_end: number;
  heading?: string | null;
  excerpt: string;
};

export type VaultFileHit = {
  path: string;
  name?: string;
  ext?: string;
  size?: number;
  mtime_ms?: number;
  score?: number;
  chunks: VaultChunkHit[];
};

export type VaultIndexStats = {
  total_files: number;
  total_chunks: number;
  indexed_bytes: number;
  semantic_enabled: boolean;
};

export type VaultSearchResponse = {
  query?: string;
  mode?: VaultSearchMode;
  scope?: VaultSearchScope;
  total?: number;
  results?: SearchHit[];
  files?: Array<{
    path?: string;
    name?: string;
    size?: number;
    mtime_ms?: number;
    ext?: string;
    chunks?: VaultChunkHit[];
  }>;
  file_hits?: VaultFileHit[];
  receipts?: Array<{ id?: string; thread_id?: string; task?: string; summary?: string; ts?: number }>;
  memory?: Array<{ line?: number | string; text?: string }>;
  index_stats?: VaultIndexStats;
};

export type VaultReindexResult = {
  ok: boolean;
  indexed_files: number;
  total_chunks: number;
  indexed_bytes?: number;
  semantic_enabled?: boolean;
  elapsed_ms: number;
  index_stats?: VaultIndexStats;
};

export type VaultAggregateRequest = {
  glob: string;
  sheet?: string;
  where?: TableWhereFilter[];
  group_by?: string[];
  aggregates: TableAggregateSpec[];
  sort?: { col: string; dir: "asc" | "desc" };
  limit?: number;
  save_as?: string;
};

export type VaultAggregateResult = {
  ok: boolean;
  columns: string[];
  rows: string[][];
  source_files: string[];
  row_count: number;
  saved_path?: string;
};

export type SearchHit = {
  kind: "file" | "receipt" | "skill" | "routine" | "memory" | "thread";
  id: string;
  title: string;
  subtitle?: string;
  path?: string;
  thread_id?: string;
  ts?: number;
  score?: number;
  chunks?: VaultChunkHit[];
  line_start?: number;
  line_end?: number;
  page?: number;
  sheet?: string;
};
export type ReceiptRecord = {
  id: string;
  thread_id?: string;
  task: string;
  summary?: string;
  status: "finished" | "waiting" | "stopped";
  files: string[];
  date?: string;
  ts: number;
  sha256?: string;
  prev_sha256?: string;
  engagement_id?: string;
  usage?: RunUsage;
};
export type ThreadRow = { id: string; title?: string; messages?: number; state?: "waiting" | "finished" | "none" };
export type Thread = { id: string; title?: string; messages: Msg[] };
export type Update = {
  current?: string;
  latest?: string;
  available?: boolean;
  staged?: string;
  mode?: string;
  can_apply?: boolean;
  dev_build?: boolean;
  notes?: string;
  release_url?: string;
  error?: string;
  checked_at?: number;
};
export type Decision = {
  id: string;
  kind: string;
  summary: string;
  status: string;
  outcome?: string;
  thread_id?: string;
  path?: string;
  ts?: number;
  resolved_ts?: number;
  /** Tripwire decisions: plain-language detail, severity and the likely source. */
  detail?: string;
  severity?: string;
  stopped?: boolean;
  source?: { kind?: string; value?: string } | null;
  run_id?: string;
  input_mode?: PipelineCheckpointInputMode;
  choices?: string[];
  inspect_files?: string[];
  answer?: string;
  connector_id?: string;
  connector_name?: string;
  connector_kind?: string;
  target_url?: string;
  url?: string;
  method?: string;
  payload_preview?: string;
  payload_sha256?: string;
  engagement_id?: string;
  engagement_name?: string;
  client_name?: string;
  retainer_budget_usd?: number;
  spent_usd?: number;
  projected_usd?: number;
  estimated_cost_usd?: number;
};
export type Offer = { id: string; title: string; detail: string; status: string; item_id?: string; ts?: number };
export type SuiteItem = { id: string; kind: string; name: string; description: string; fields?: string[] };
export type LibraryItem = { id: string; kind: string; name: string; description: string; author: string; builtin?: boolean };
export type BriefJob = {
  id?: string;
  task: string;
  date?: string;
  summary?: string;
  files?: string[];
  thread_id?: string;
  ts?: number;
  sha256?: string;
  engagement_id?: string;
  usage?: RunUsage;
};
export type Brief = {
  date?: string;
  person?: string;
  headline?: string;
  finished?: BriefJob[];
  stopped?: BriefJob[];
  suite_size?: number;
  library_size?: number;
  learned_recently?: string[];
};
export type TodayEvent = { ts?: number; kind?: string; title?: string; detail?: string; files?: string[]; severity?: string; action?: string };
export type SecurityEvent = {
  id: string;
  ts: number;
  code: string;
  severity: "info" | "notice" | "warning" | "critical" | string;
  title: string;
  detail: string;
  action: string;
  repeats?: number;
};
export type SecurityPayload = {
  armed: { id: string; label: string; detail: string }[];
  events: SecurityEvent[];
  last_24h: { events: number; critical: number; blocked: number };
  log: { verified: boolean; length: number; head_hash: string };
  bans: { ip: string; until_ms: number; bans: number; probed?: string[] }[];
  policy: { allow_hosts: string[]; block_hosts: string[]; allow_private_env?: boolean };
  can_edit: boolean;
};

/** Tripwire honeytraps exist on native Ergon v0.8.0+ (never in the browser edition). */
export function hasTripwire(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone() || !status || status === "error") return false;
  return status.capabilities?.tripwire === true;
}
export type TodayPayload = { date?: string; events?: TodayEvent[] };
export type GraphNode = { id: string; kind: string; label?: string; detail?: string };
export type GraphEdge = { from: string; to: string; kind?: string };
export type GraphPayload = { nodes?: GraphNode[]; edges?: GraphEdge[] };

export type Res<T> =
  | { ok: true; data: T }
  | { ok: false; error: string; status: number; current_sha256?: string };

export function isWebStandalone(): boolean {
  return typeof window !== "undefined" && Boolean((window as unknown as { __ERGON_WEB_ENGINE__?: boolean }).__ERGON_WEB_ENGINE__);
}

function isAtLeastMinor(version: string | undefined, minMinor: number): boolean {
  if (!version) return false;
  const m = /^v?(\d+)\.(\d+)/.exec(version.trim());
  if (!m) return false;
  const major = Number(m[1]);
  const minor = Number(m[2]);
  return major > 0 || minor >= minMinor;
}

/** Progressive v0.5.0 API detector: avoids 404 console noise against v0.4.0 servers while
 *  lighting up live SSE, /api/skills, /api/routines, /api/search, and /api/receipts automatically on v0.5.0+. */
export function hasV050Api(
  status: Status | null | "error" | undefined,
  cap?: keyof ServerCapabilities
): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (cap && typeof status.capabilities?.[cap] === "boolean") {
    return Boolean(status.capabilities[cap]);
  }
  return isAtLeastMinor(status.version, 5);
}

/** Progressive v0.7.0 API detector: activates file history/diff/restore, background scheduler
 *  controls/run history, audit bundle export, and /api/people/switch automatically on v0.7.0+
 *  while keeping pre-v0.7.0 servers completely free of 404 console errors. */
export function hasV070Api(
  status: Status | null | "error" | undefined,
  cap?: keyof ServerCapabilities
): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (cap && typeof status.capabilities?.[cap] === "boolean") {
    return Boolean(status.capabilities[cap]);
  }
  if (!cap && status.capabilities) {
    if (
      status.capabilities.file_history ||
      status.capabilities.audit_bundle ||
      typeof status.capabilities.scheduler === "boolean"
    ) {
      return true;
    }
  }
  return isAtLeastMinor(status.version, 7);
}

/** Parses v0.7.0 routine cadence strings ("every 15m", "hourly", "daily 09:00", "weekly mon",
 *  or interval_secs) to estimate the next scheduled execution epoch ms when the server omits it. */
export function estimateNextRunMs(
  schedule: string,
  fromMs = Date.now(),
  intervalSecs?: number | null
): number | null {
  if (typeof intervalSecs === "number" && Number.isFinite(intervalSecs) && intervalSecs > 0) {
    return fromMs + intervalSecs * 1000;
  }
  const raw = String(schedule || "").trim().toLowerCase();
  if (!raw) return null;
  const everyMatch = /^every\s+(\d+)\s*(s|sec|secs|seconds?|m|min|mins|minutes?|h|hr|hrs|hours?|d|day|days?)$/.exec(raw);
  if (everyMatch) {
    const n = Math.max(1, Number(everyMatch[1]));
    const unit = everyMatch[2];
    const mult = unit.startsWith("s")
      ? 1000
      : unit.startsWith("m")
        ? 60_000
        : unit.startsWith("h")
          ? 3_600_000
          : 86_400_000;
    return fromMs + n * mult;
  }
  if (raw === "hourly" || raw === "every hour") return fromMs + 3_600_000;
  const dailyMatch = /^(?:daily|every\s+day|every\s+weekday)(?:\s+(\d{1,2}):(\d{2}))?$/.exec(raw);
  if (dailyMatch) {
    const hh = Math.min(23, Number(dailyMatch[1] ?? 9));
    const mm = Math.min(59, Number(dailyMatch[2] ?? 0));
    const next = new Date(fromMs);
    next.setSeconds(0, 0);
    next.setHours(hh, mm, 0, 0);
    if (next.getTime() <= fromMs) next.setDate(next.getDate() + 1);
    if (raw.includes("weekday")) {
      while (next.getDay() === 0 || next.getDay() === 6) next.setDate(next.getDate() + 1);
    }
    return next.getTime();
  }
  const weeklyMatch = /^(?:weekly|every)\s+(sun|mon|tue|wed|thu|fri|sat)[a-z]*(?:\s+(\d{1,2}):(\d{2}))?$/.exec(raw);
  if (weeklyMatch) {
    const days = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    const targetDay = days.indexOf(weeklyMatch[1]);
    const hh = Math.min(23, Number(weeklyMatch[2] ?? 9));
    const mm = Math.min(59, Number(weeklyMatch[3] ?? 0));
    const next = new Date(fromMs);
    next.setSeconds(0, 0);
    next.setHours(hh, mm, 0, 0);
    let delta = (targetDay - next.getDay() + 7) % 7;
    if (delta === 0 && next.getTime() <= fromMs) delta = 7;
    next.setDate(next.getDate() + delta);
    return next.getTime();
  }
  return fromMs + 3_600_000;
}

export function formatCountdown(targetMs: number | null | undefined, nowMs = Date.now()): string {
  if (!targetMs || !Number.isFinite(targetMs)) return "Scheduled";
  const diffSec = Math.round((targetMs - nowMs) / 1000);
  if (diffSec <= 0) return "Due now";
  if (diffSec < 60) return `in ${diffSec}s`;
  const mins = Math.floor(diffSec / 60);
  if (mins < 60) return `in ${mins}m`;
  const hrs = Math.floor(mins / 60);
  const remM = mins % 60;
  if (hrs < 24) return remM > 0 ? `in ${hrs}h ${remM}m` : `in ${hrs}h`;
  const days = Math.floor(hrs / 24);
  const remH = hrs % 24;
  return remH > 0 ? `in ${days}d ${remH}h` : `in ${days}d`;
}

export async function call<T>(
  path: string,
  opts: { method?: string; body?: unknown; signal?: AbortSignal } = {}
): Promise<Res<T>> {
  try {
    const r = await fetch(path, {
      method: opts.method ?? "GET",
      cache: "no-store",
      signal: opts.signal,
      headers: opts.body !== undefined ? { "Content-Type": "application/json" } : undefined,
      body: opts.body !== undefined ? JSON.stringify(opts.body) : undefined,
    });
    const raw = await r.text();
    let parsed: unknown = null;
    try {
      parsed = raw ? JSON.parse(raw) : {};
    } catch {
      return { ok: false, error: r.ok ? "The server returned an unreadable reply." : "", status: r.status };
    }
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return { ok: false, error: "The server returned an unexpected data format.", status: r.status };
    }
    const data = parsed as T & { error?: unknown; current_sha256?: unknown };
    if (!r.ok) {
      return {
        ok: false,
        error: typeof data.error === "string" ? data.error : "",
        status: r.status,
        ...(typeof data.current_sha256 === "string" ? { current_sha256: data.current_sha256 } : {}),
      };
    }
    return { ok: true, data };
  } catch (e) {
    if (e instanceof DOMException && e.name === "AbortError") {
      return { ok: false, error: "Run cancelled.", status: 499 };
    }
    return { ok: false, error: e instanceof TypeError ? "Could not reach the Ergon server." : String(e), status: 0 };
  }
}

export async function downloadWorkspaceFile(filePath: string): Promise<boolean> {
  const url = "/api/files/content?path=" + encodeURIComponent(filePath) + "&download=1";
  try {
    const r = await fetch(url, { cache: "no-store" });
    if (!r.ok) return false;
    const blob = await r.blob();
    const dlUrl = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = dlUrl;
    a.download = filePath.split("/").pop() || "workspace-file";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(dlUrl), 1500);
    return true;
  } catch {
    return false;
  }
}

export type EvidenceVerificationItem = {
  label: string;
  status: string;
  sha256?: string;
  detail?: string;
};

export type EvidenceVerificationResult = {
  valid: boolean;
  kind: "audit_bundle" | "deliverable_pack" | "playbook_pack" | "unknown";
  title: string;
  receipt_chain_valid: boolean | null;
  bundle_hash_valid: boolean | null;
  manifest_valid: boolean | null;
  citations_valid: boolean | null;
  workspace_valid: boolean | null;
  receipt_count: number;
  deliverable_count: number;
  citation_count: number;
  stale_citations: number;
  broken_citations: number;
  expected_sha256: string;
  actual_sha256: string;
  summary: string;
  errors: string[];
  items: EvidenceVerificationItem[];
};

export type HandoffZipDownloadResult =
  | { ok: true; sha256: string; leaks: number }
  | { ok: false; blocked: boolean; status: number; error: string; scan?: HandoffLeakScan };

export async function exportHandoffBundleZip(
  opts: { title?: string; force?: boolean } = {}
): Promise<HandoffZipDownloadResult> {
  const resp = await fetch("/api/export/handoff", {
    method: "POST",
    cache: "no-store",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title: opts.title || "Client Handoff", force: Boolean(opts.force) }),
  });
  if (resp.status === 409) {
    const errJson = (await resp.json().catch(() => ({}))) as {
      error?: string;
      scan?: HandoffLeakScan;
    };
    return {
      ok: false,
      blocked: true,
      status: 409,
      error: errJson.error || "Handoff blocked by Strict Client policy.",
      scan: errJson.scan,
    };
  }
  if (!resp.ok) {
    return {
      ok: false,
      blocked: false,
      status: resp.status,
      error: `Handoff export failed (HTTP ${resp.status}).`,
    };
  }
  const blob = await resp.blob();
  const sha256 = resp.headers.get("X-Ergon-Handoff-SHA256") ?? "";
  const leaks = Number(resp.headers.get("X-Ergon-Handoff-Leaks") ?? "0") || 0;
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "ergon-client-handoff.zip";
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  return { ok: true, sha256, leaks };
}

export function hasPipelinesApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.pipelines === "boolean") {
    return Boolean(status.capabilities.pipelines);
  }
  return isAtLeastMinor(status.version, 29);
}

export async function startPipelineRun(body: {
  routine_id?: string;
  name?: string;
  steps_v2?: PipelineStepSpec[];
  inputs?: Record<string, string>;
  thread_id?: string;
}): Promise<Res<{ run: PipelineRun }>> {
  return call<{ run: PipelineRun }>("/api/pipelines/run", { method: "POST", body });
}

export async function validatePipeline(
  steps: PipelineStepSpec[]
): Promise<Res<PipelineValidationResult> & { validation?: PipelineValidationResult }> {
  try {
    const r = await fetch("/api/pipelines/validate", {
      method: "POST",
      cache: "no-store",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ steps_v2: steps }),
    });
    const raw = await r.text();
    let parsed: (Partial<PipelineValidationResult> & { error?: unknown }) | null = null;
    try {
      parsed = raw ? (JSON.parse(raw) as Partial<PipelineValidationResult> & { error?: unknown }) : {};
    } catch {
      return { ok: false, error: "The server returned an unreadable validation reply.", status: r.status };
    }
    const errors = Array.isArray(parsed?.errors)
      ? parsed.errors.map(String)
      : typeof parsed?.error === "string" && parsed.error
        ? [parsed.error]
        : [];
    const topologicalOrder = Array.isArray(parsed?.topological_order)
      ? parsed.topological_order.map(String)
      : [];
    const stages = Array.isArray(parsed?.stages)
      ? parsed.stages.map((stage) => (Array.isArray(stage) ? stage.map(String) : []))
      : undefined;
    const valid = Boolean(parsed?.valid ?? (r.ok && errors.length === 0));
    const validation: PipelineValidationResult = {
      ok: Boolean(parsed?.ok ?? (r.ok && valid)),
      valid,
      errors,
      topological_order: topologicalOrder,
      ...(stages ? { stages } : {}),
    };
    if (!r.ok && r.status !== 400) {
      return {
        ok: false,
        error: errors[0] || `Pipeline validation failed (HTTP ${r.status}).`,
        status: r.status,
      };
    }
    return {
      ok: true,
      data: validation,
      validation,
    };
  } catch (e) {
    return {
      ok: false,
      error: e instanceof TypeError ? "Could not reach the Ergon server." : String(e),
      status: 0,
    };
  }
}

export async function listPipelineRuns(opts: { status?: string; limit?: number } = {}): Promise<Res<{ runs: PipelineRun[] }>> {
  const params = new URLSearchParams();
  if (opts.status) params.set("status", opts.status);
  if (typeof opts.limit === "number") params.set("limit", String(opts.limit));
  const qs = params.toString();
  return call<{ runs: PipelineRun[] }>(`/api/pipelines/runs${qs ? `?${qs}` : ""}`);
}

export async function getPipelineRun(runId: string): Promise<Res<{ run: PipelineRun }>> {
  return call<{ run: PipelineRun }>(`/api/pipelines/runs/${encodeURIComponent(runId)}`);
}

export async function resumePipelineRun(
  runId: string,
  body: PipelineResumePayload
): Promise<Res<{ run: PipelineRun }>> {
  return call<{ run: PipelineRun }>(`/api/pipelines/runs/${encodeURIComponent(runId)}/resume`, {
    method: "POST",
    body,
  });
}

export async function retryPipelineRun(
  runId: string,
  body: { from_step_index?: number } = {}
): Promise<Res<{ run: PipelineRun }>> {
  return call<{ run: PipelineRun }>(`/api/pipelines/runs/${encodeURIComponent(runId)}/retry`, {
    method: "POST",
    body,
  });
}

export async function cancelPipelineRun(runId: string): Promise<Res<{ run: PipelineRun }>> {
  return call<{ run: PipelineRun }>(`/api/pipelines/runs/${encodeURIComponent(runId)}/cancel`, {
    method: "POST",
    body: {},
  });
}

export type SaveFileContentPayload = {
  path: string;
  content: string;
  expected_sha256?: string;
  reason?: string;
};

export type SaveFileContentResult = {
  path: string;
  size: number;
  mtime_ms: number;
  sha256: string;
  rev_id?: string;
};

export async function saveFileContent(body: SaveFileContentPayload): Promise<Res<SaveFileContentResult>> {
  return call<SaveFileContentResult>("/api/files/content", { method: "PUT", body });
}

export type PatchFileOp = {
  op: "replace" | "replace_lines";
  find?: string;
  replace: string;
  start_line?: number;
  end_line?: number;
  replace_all?: boolean;
};

export type PatchFilePayload = {
  path: string;
  ops: PatchFileOp[];
  expected_sha256?: string;
};

export type PatchFileResult = {
  path: string;
  applied: number;
  size: number;
  sha256: string;
  diff?: FileDiffPayload;
};

export async function patchFileContent(body: PatchFilePayload): Promise<Res<PatchFileResult>> {
  return call<PatchFileResult>("/api/files/content", { method: "PATCH", body });
}

export type TableWhereOp = "eq" | "neq" | "contains" | "gt" | "gte" | "lt" | "lte";

export type TableWhereFilter = {
  col: string;
  op: TableWhereOp;
  value: string;
};

export type TableAggregateFn = "sum" | "avg" | "min" | "max" | "count";

export type TableAggregateSpec = {
  col: string;
  fn: TableAggregateFn;
  as: string;
};

export type TableQueryRequest = {
  path: string;
  sheet?: string;
  where?: TableWhereFilter[];
  group_by?: string[];
  aggregates?: TableAggregateSpec[];
  sort?: { col: string; dir: "asc" | "desc" };
  limit?: number;
  save_as?: string;
};

export type TableQueryResult = {
  columns: string[];
  rows: string[][];
  total_rows: number;
  matched_rows: number;
  saved_path?: string;
};

export async function queryTable(body: TableQueryRequest): Promise<Res<TableQueryResult>> {
  return call<TableQueryResult>("/api/tables/query", { method: "POST", body });
}

export type ExportXlsxSheet = {
  name: string;
  path?: string;
  rows?: string[][];
  totals_row?: boolean;
};

export type ExportFileRequest = {
  path?: string;
  format: "pdf" | "docx" | "xlsx";
  title?: string;
  brand_header?: string;
  output?: string;
  sheets?: ExportXlsxSheet[];
};

export type ExportFileResult = {
  path: string;
  size: number;
  format?: string;
  sha256?: string;
};

export async function exportFile(body: ExportFileRequest): Promise<Res<ExportFileResult>> {
  return call<ExportFileResult>("/api/files/export", { method: "POST", body });
}

export function canExportPdf(path: string, extractedFormat?: "markdown" | "csv"): boolean {
  if (extractedFormat === "markdown" || extractedFormat === "csv") return true;
  const ext = (path.split(".").pop() || "").toLowerCase();
  return ext === "md" || ext === "markdown" || ext === "txt" || ext === "csv" || ext === "tsv";
}

export type TriggerKind = "webhook" | "folder_watch";

export type TriggerRule = {
  id: string;
  name: string;
  kind: TriggerKind;
  enabled: boolean;
  routine_id: string | null;
  target_kind?: "routine" | "pipeline";
  hook_id?: string | null;
  hook_path?: string | null;
  webhook_url?: string | null;
  has_secret?: boolean;
  /** Ephemeral: returned once on create or secret rotation; never persisted in client state after dismissal. */
  signing_secret?: string;
  hook_token?: string;
  watch_dir?: string | null;
  glob?: string | null;
  folder_path?: string | null;
  glob_filter?: string | null;
  max_bytes?: number;
  last_triggered_ms?: number | null;
  trigger_count?: number;
  created_ms?: number;
  updated_ms?: number;
};

export type OutboundConnectorKind = "http_webhook" | "rest_api" | "smtp_relay";

export type OutboundConnector = {
  id: string;
  name: string;
  kind: OutboundConnectorKind;
  url: string;
  method: "POST" | "PUT";
  headers_template?: Record<string, string>;
  secret_keys?: string[];
  has_secret?: boolean;
  require_approval: boolean;
  effective_require_approval?: boolean;
  blocked_by_policy?: boolean;
  enabled?: boolean;
  last_delivery_ms?: number | null;
  last_status?: string | null;
  created_ms?: number;
  updated_ms?: number;
};

export type DeliveryLog = {
  id: string;
  connector_id: string;
  connector_name?: string;
  kind?: string;
  method?: string;
  url?: string;
  ts?: number;
  ts_ms?: number;
  status: "ok" | "error" | "held" | "blocked" | "denied" | "pending_approval" | string;
  http_status?: number | null;
  summary?: string;
  response_preview?: string | null;
  payload_sha256?: string;
  payload_bytes?: number;
  error?: string | null;
  decision_id?: string | null;
  thread_id?: string | null;
  test?: boolean;
};

export type ConnectorSecretMeta = {
  key: string;
  has_secret: boolean;
  updated_ms?: number;
};

export type ConnectorsSnapshot = {
  triggers: TriggerRule[];
  outbound: OutboundConnector[];
  deliveries: DeliveryLog[];
  secrets?: ConnectorSecretMeta[];
  policy_profile?: WorkspacePolicyProfile;
};

export function hasConnectorsApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.connectors === "boolean") {
    return Boolean(status.capabilities.connectors);
  }
  return isAtLeastMinor(status.version, 31);
}

export async function listConnectors(): Promise<Res<ConnectorsSnapshot>> {
  const r = await call<{
    triggers?: TriggerRule[];
    outbound?: OutboundConnector[];
    deliveries?: DeliveryLog[];
    delivery_log?: DeliveryLog[];
    logs?: DeliveryLog[];
    secrets?: ConnectorSecretMeta[];
    policy_profile?: WorkspacePolicyProfile;
  }>("/api/connectors");
  if (!r.ok) return r;
  const rawLogs = Array.isArray(r.data.delivery_log)
    ? r.data.delivery_log
    : Array.isArray(r.data.deliveries)
      ? r.data.deliveries
      : Array.isArray(r.data.logs)
        ? r.data.logs
        : [];
  return {
    ok: true,
    data: {
      triggers: Array.isArray(r.data.triggers) ? r.data.triggers : [],
      outbound: Array.isArray(r.data.outbound) ? r.data.outbound : [],
      deliveries: rawLogs.map((d) => ({
        ...d,
        ts: typeof d.ts_ms === "number" ? d.ts_ms : typeof d.ts === "number" ? d.ts : Date.now(),
      })),
      secrets: Array.isArray(r.data.secrets) ? r.data.secrets : undefined,
      policy_profile: r.data.policy_profile,
    },
  };
}

export async function saveTriggerRule(body: {
  id?: string;
  name: string;
  kind: TriggerKind;
  routine_id?: string | null;
  target_kind?: "routine" | "pipeline";
  enabled?: boolean;
  watch_dir?: string;
  glob?: string;
}): Promise<
  Res<{
    trigger?: TriggerRule;
    signing_secret?: string;
    hook_token?: string;
    hook_path?: string;
  }>
> {
  return call("/api/connectors/triggers", {
    method: "POST",
    body,
  });
}

export async function rotateTriggerSecret(
  id: string
): Promise<Res<{ trigger?: TriggerRule; signing_secret?: string; hook_token?: string; hook_path?: string }>> {
  return call(`/api/connectors/triggers/${encodeURIComponent(id)}/rotate`, {
    method: "POST",
    body: {},
  });
}

export async function deleteTriggerRule(id: string): Promise<Res<{ deleted?: string }>> {
  return call<{ deleted?: string }>(`/api/connectors/triggers/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

async function hmacSha256Hex(secret: string, bodyText: string): Promise<string | null> {
  try {
    if (typeof crypto === "undefined" || !crypto.subtle) return null;
    const enc = new TextEncoder();
    const key = await crypto.subtle.importKey(
      "raw",
      enc.encode(secret),
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["sign"]
    );
    const sig = await crypto.subtle.sign("HMAC", key, enc.encode(bodyText));
    return Array.from(new Uint8Array(sig))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  } catch {
    return null;
  }
}

export async function simulateWebhookTrigger(
  hookId: string,
  payloadText: string,
  secretOrToken?: string
): Promise<Res<{ accepted?: boolean; trigger_file?: string; run_id?: string; thread_id?: string; message?: string }>> {
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  const trimmedSecret = (secretOrToken ?? "").trim();
  if (trimmedSecret) {
    const hex = await hmacSha256Hex(trimmedSecret, payloadText);
    if (hex) headers["X-Ergon-Signature"] = `sha256=${hex}`;
    headers["X-Ergon-Hook-Token"] = trimmedSecret;
  } else {
    headers["X-Ergon-Simulate"] = "1";
  }
  try {
    const r = await fetch(`/api/hooks/${encodeURIComponent(hookId)}`, {
      method: "POST",
      cache: "no-store",
      headers,
      body: payloadText,
    });
    const raw = await r.text();
    let parsed: Record<string, unknown> = {};
    try {
      parsed = raw ? (JSON.parse(raw) as Record<string, unknown>) : {};
    } catch {
      parsed = {};
    }
    if (!r.ok) {
      return {
        ok: false,
        status: r.status,
        error: typeof parsed.error === "string" ? parsed.error : `Webhook test returned HTTP ${r.status}.`,
      };
    }
    return {
      ok: true,
      data: parsed as { accepted?: boolean; trigger_file?: string; run_id?: string; thread_id?: string; message?: string },
    };
  } catch (e) {
    return { ok: false, status: 0, error: String(e) };
  }
}

export async function saveOutboundConnector(body: {
  id?: string;
  name: string;
  kind: OutboundConnectorKind;
  url: string;
  method: "POST" | "PUT";
  headers_template?: Record<string, string>;
  secrets?: Record<string, string>;
  secret_key?: string;
  secret_value?: string;
  require_approval: boolean;
  enabled?: boolean;
}): Promise<Res<{ connector?: OutboundConnector }>> {
  return call<{ connector?: OutboundConnector }>("/api/connectors/outbound", {
    method: "POST",
    body,
  });
}

export async function deleteOutboundConnector(id: string): Promise<Res<{ deleted?: string }>> {
  return call<{ deleted?: string }>(`/api/connectors/outbound/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export async function testOutboundConnector(
  id: string,
  payload: Record<string, unknown> = {}
): Promise<
  Res<{
    ok?: boolean;
    held?: boolean;
    status?: string;
    http_status?: number | null;
    decision_id?: string | null;
    delivery?: DeliveryLog;
    message?: string;
    error?: string;
  }>
> {
  return call(`/api/connectors/outbound/${encodeURIComponent(id)}/test`, {
    method: "POST",
    body: { payload },
  });
}

export function hasClientsApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.clients === "boolean") {
    return Boolean(status.capabilities.clients);
  }
  return isAtLeastMinor(status.version, 33);
}

export function hasMeteringApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.metering === "boolean") {
    return Boolean(status.capabilities.metering);
  }
  return isAtLeastMinor(status.version, 33);
}

export type ClientsPayload = {
  clients: Client[];
  engagements: Engagement[];
  active_engagement_id?: string | null;
};

export async function fetchClients(): Promise<Res<ClientsPayload>> {
  const r = await call<ClientsPayload>("/api/clients");
  if (!r.ok) return r;
  return {
    ok: true,
    data: {
      clients: Array.isArray(r.data.clients) ? r.data.clients : [],
      engagements: Array.isArray(r.data.engagements) ? r.data.engagements : [],
      active_engagement_id: typeof r.data.active_engagement_id === "string" ? r.data.active_engagement_id : null,
    },
  };
}

export async function createClient(body: {
  name: string;
  brand_header?: string;
  default_rate_usd?: number;
  billing_currency?: string;
}): Promise<Res<{ client: Client }>> {
  return call<{ client: Client }>("/api/clients", {
    method: "POST",
    body,
  });
}

export async function createEngagement(body: {
  client_id: string;
  name: string;
  person_id?: string;
  retainer_hours?: number;
  retainer_budget_usd?: number;
  hourly_rate_usd?: number;
  period?: string;
}): Promise<Res<{ engagement: Engagement; active_engagement_id?: string }>> {
  return call<{ engagement: Engagement; active_engagement_id?: string }>("/api/engagements", {
    method: "POST",
    body,
  });
}

export async function switchEngagement(
  engagementId: string
): Promise<Res<{ active_engagement_id: string; engagement?: Engagement | null }>> {
  return call<{ active_engagement_id: string; engagement?: Engagement | null }>("/api/engagements/switch", {
    method: "POST",
    body: { engagement_id: engagementId },
  });
}

export async function fetchMeteringSummary(opts: {
  engagement_id?: string;
  client_id?: string;
  window_days?: number;
} = {}): Promise<Res<MeteringSummary>> {
  const params = new URLSearchParams();
  if (opts.engagement_id) params.set("engagement_id", opts.engagement_id);
  if (opts.client_id) params.set("client_id", opts.client_id);
  if (typeof opts.window_days === "number") params.set("window_days", String(opts.window_days));
  const qs = params.toString();
  return call<MeteringSummary>(`/api/metering/summary${qs ? `?${qs}` : ""}`);
}

export function hasVaultSearchApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.vault_search === "boolean") {
    return Boolean(status.capabilities.vault_search);
  }
  return isAtLeastMinor(status.version, 34);
}

export async function searchVault(opts: {
  q: string;
  scope?: VaultSearchScope;
  mode?: VaultSearchMode;
  limit?: number;
  path_prefix?: string;
  signal?: AbortSignal;
}): Promise<Res<VaultSearchResponse>> {
  const params = new URLSearchParams();
  params.set("q", opts.q);
  if (opts.scope) params.set("scope", opts.scope);
  if (opts.mode) params.set("mode", opts.mode);
  if (typeof opts.limit === "number") params.set("limit", String(opts.limit));
  if (opts.path_prefix) params.set("path_prefix", opts.path_prefix);
  return call<VaultSearchResponse>(`/api/search?${params.toString()}`, { signal: opts.signal });
}

export async function searchWorkspace(
  q: string,
  opts: { limit?: number; scope?: VaultSearchScope; mode?: VaultSearchMode; signal?: AbortSignal } = {}
): Promise<Res<VaultSearchResponse>> {
  return searchVault({ q, ...opts });
}

export async function reindexVault(): Promise<Res<VaultReindexResult>> {
  return call<VaultReindexResult>("/api/vault/reindex", { method: "POST", body: {} });
}

export async function aggregateVault(body: VaultAggregateRequest): Promise<Res<VaultAggregateResult>> {
  return call<VaultAggregateResult>("/api/vault/aggregate", { method: "POST", body });
}
