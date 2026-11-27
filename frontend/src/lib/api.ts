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
  backups?: boolean;
  people_switch?: boolean;
  tripwire?: boolean;
  pipelines?: boolean;
  file_edit?: boolean;
  table_query?: boolean;
  pdf_export?: boolean;
  connectors?: boolean;
  mcp?: boolean;
  clients?: boolean;
  metering?: boolean;
  vault_search?: boolean;
  client_crm?: boolean;
  time_entries?: boolean;
  invoicing?: boolean;
  diagnostics?: boolean;
  help?: boolean;
  auth?: boolean;
  oidc?: boolean;
  tls?: boolean;
};

export type AdminContactConfig = {
  name: string;
  email: string;
  phone: string;
  channel_url: string;
  notes: string;
  configured: boolean;
  source?: string;
  updated_ms?: number;
};

export type HelpConfigPayload = {
  onboarding_completed: boolean;
  onboarding_completed_at_ms?: number | null;
  admin_contact: AdminContactConfig;
  can_edit_admin_contact: boolean;
  diagnostics_url?: string;
};

export type DiagnosticsSummary = {
  app_version: string;
  os: string;
  arch: string;
  uptime_secs: number;
  workspace_count: number;
  model_configured: boolean;
  log_files: number;
  crash_reports: number;
};

export type ClientContact = {
  id: string;
  name: string;
  email?: string;
  role?: string;
  primary?: boolean;
};

export type Client = {
  id: string;
  name: string;
  brand_header?: string;
  billing_currency?: string;
  default_rate_usd?: number;
  contacts?: ClientContact[];
  notes_md?: string;
  tax_rate_pct?: number;
  payment_terms?: string;
  created_ms?: number;
  updated_ms?: number;
};

export type RolloverMode = "none" | "carry_hours";

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
  workspace_prefix?: string;
  sow_rules?: string[];
  rollover_mode?: RolloverMode;
  period_start_ms?: number;
  period_end_ms?: number;
  archived_ms?: number | null;
  created_ms?: number;
  updated_ms?: number;
};

export type TimeExpenseKind = "time" | "expense" | "adjustment";

export type TimeExpenseEntry = {
  id: string;
  ts_ms: number;
  client_id: string;
  engagement_id: string;
  thread_id?: string | null;
  receipt_id?: string | null;
  kind: TimeExpenseKind;
  description: string;
  hours: number;
  rate_usd: number;
  amount_usd: number;
  billable: boolean;
  invoiced_id?: string | null;
};

export type InvoiceStatus = "draft" | "issued" | "paid" | "void";

export type InvoiceLineItem = {
  id?: string;
  kind?: string;
  description: string;
  quantity?: number;
  hours?: number;
  unit_price_usd?: number;
  rate_usd?: number;
  amount_usd: number;
};

export type InvoiceRecord = {
  id: string;
  invoice_number: string;
  client_id: string;
  engagement_id: string;
  period_label: string;
  issued_ms: number;
  due_ms?: number | null;
  status: InvoiceStatus;
  line_items: InvoiceLineItem[];
  llm_cost_usd: number;
  billable_subtotal_usd: number;
  tax_usd: number;
  total_usd: number;
  deliverable_paths: string[];
  pdf_path?: string | null;
  xlsx_path?: string | null;
  json_path?: string | null;
  sha256: string;
};

export type EngagementContext = {
  engagement: Engagement;
  client?: Client | null;
  scoped_file_count: number;
  workspace_prefix?: string;
  sow_rules: string[];
  contacts: ClientContact[];
  burn_metrics?: {
    runs?: number;
    cost_usd?: number;
    retainer_budget_usd?: number;
    retainer_burn_pct?: number;
    billable_value_usd?: number;
    minutes_saved?: number;
  };
  unbilled_entries: TimeExpenseEntry[];
  unbilled_total_usd?: number;
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
export type RecoverableRun = {
  run_id: string;
  thread_id: string;
  person_id: string;
  task: string;
  started_ms: number;
  updated_ms: number;
  steps: string[];
  written_files?: string[];
  status: string;
  error?: string | null;
  resume_count?: number;
  can_resume?: boolean;
};
export type LiveRun = {
  active: boolean;
  run_id: string;
  thread_id: string;
  task: string;
  started_ms: number;
  elapsed_ms?: number;
  steps: (string | LiveRunStep)[];
  cancelled?: boolean;
  kind?: string;
  routine_id?: string | null;
  routine_name?: string | null;
};
export type LiveRunStatus = {
  active: boolean;
  status?: string;
  run_id?: string;
  thread_id: string;
  task: string;
  started_ms: number;
  elapsed_ms?: number;
  steps: (string | LiveRunStep)[];
  cancelled?: boolean;
  model_configured?: boolean;
  recoverable_runs?: RecoverableRun[];
  runs?: LiveRun[];
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
export type PipelineStepKind = "skill" | "agent" | "checkpoint" | "export" | "deliver" | "subpipeline";
export type PipelineCheckpointInputMode = "approve_reject" | "choice" | "text";
export type PipelineWhen = "always" | "on_anomaly" | "on_previous_ok" | "on_all_ok" | (string & {});
export type PipelineOnError = "fail" | "continue" | `goto:${string}`;

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
  args?: Record<string, unknown>;
  prompt?: string;
  checkpoint?: PipelineCheckpointSpec;
  export?: PipelineExportSpec;
  deliver?: PipelineDeliverSpec;
  routine_id?: string;
  inputs?: Record<string, string>;
  when?: PipelineWhen;
  depends_on?: string[];
  for_each?: string;
  timeout_secs?: number;
  retry_max?: number;
  retry_backoff_ms?: number;
  on_error?: PipelineOnError;
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
  attempts?: number;
  subpipeline_run_id?: string;
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

export type PendingCheckpoint = PipelinePendingCheckpoint;

export type PipelineAgentSnapshot = {
  messages: unknown[];
  written: string[];
  budget_used: number;
};

export type AgentSnapshot = PipelineAgentSnapshot;

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
  pending_checkpoints?: PipelinePendingCheckpoint[];
  agent_snapshot?: PipelineAgentSnapshot | null;
  step_snapshots?: Record<string, PipelineAgentSnapshot>;
  error?: string | null;
  engagement_id?: string | null;
  usage?: RunUsage;
};

export function getPendingCheckpoints(run: PipelineRun): PipelinePendingCheckpoint[] {
  const list: PipelinePendingCheckpoint[] = [];
  const seen = new Set<string>();
  const add = (cp: PipelinePendingCheckpoint | null | undefined) => {
    if (!cp || typeof cp !== "object" || !cp.checkpoint_id) return;
    if (seen.has(cp.checkpoint_id)) return;
    seen.add(cp.checkpoint_id);
    list.push(cp);
  };
  if (Array.isArray(run.pending_checkpoints)) {
    for (const cp of run.pending_checkpoints) add(cp);
  }
  add(run.pending_checkpoint);
  return list;
}

export type PipelineValidationResult = {
  ok: boolean;
  valid: boolean;
  errors: string[];
  topological_order: string[];
  stages?: string[][];
};

export type PipelineResumePayload = {
  checkpoint_id?: string;
  decision?: "approve" | "reject" | "answer";
  text?: string;
  action?: "approve" | "reject" | "answer";
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
  tz?: string;
  tz_offset_mins?: number | null;
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

export function isDeterministicRoutine(rt: Pick<Routine, "skill_id" | "steps_v2">): boolean {
  if (Array.isArray(rt.steps_v2) && rt.steps_v2.length > 0) {
    return rt.steps_v2.every(
      (s) => s.kind === "skill" || s.kind === "export" || s.kind === "checkpoint" || s.kind === "subpipeline"
    );
  }
  return Boolean(rt.skill_id && rt.skill_id.trim());
}
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
  isolate_active_engagement?: boolean;
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
  client_id?: string;
  engagement_id?: string;
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
  applied?: string;
  installed?: string;
  mode?: "off" | "notify" | "auto" | string;
  can_apply?: boolean;
  can_manage?: boolean;
  is_admin?: boolean;
  gated?: boolean;
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
  server_id?: string;
  server_name?: string;
  tool_name?: string;
  arguments_preview?: string;
  args_preview?: string;
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

export type BackupOffsiteConfig = {
  kind: "none" | "folder" | "s3";
  folder_path: string;
  s3_endpoint: string;
  s3_bucket: string;
  s3_region: string;
  s3_prefix: string;
  s3_path_style: boolean;
  s3_credentials_configured?: boolean;
};

export type BackupConfig = {
  enabled: boolean;
  interval_hours: number;
  retention_count: number;
  local_dir: string;
  effective_local_dir?: string;
  offsite: BackupOffsiteConfig;
};

export type BackupRuntimeState = {
  last_run_ms: number | null;
  last_backup_name: string | null;
  last_backup_sha256: string | null;
  last_file_count: number | null;
  last_zip_bytes: number | null;
  last_offsite_status: string | null;
  last_error: string | null;
};

export type BackupListEntry = {
  name: string;
  size_bytes: number;
  modified_ms: number;
  sha256: string | null;
  created_at_utc: string | null;
  file_count: number | null;
  kind: "backup" | "pre_restore";
};

export type BackupsPayload = {
  config: BackupConfig;
  state: BackupRuntimeState;
  backups: BackupListEntry[];
  can_edit: boolean;
};

export type BackupRunResponse = {
  ok: boolean;
  backup: {
    filename: string;
    sha256: string;
    manifest_sha256: string;
    file_count: number;
    total_bytes: number;
    zip_bytes: number;
    created_at_utc: string;
    pruned_local: string[];
    offsite?: { kind: string; ok: boolean; target: string; detail: string } | null;
  };
  state: BackupRuntimeState;
  backups: BackupListEntry[];
};

export type BackupRestoreResponse = {
  ok: boolean;
  restore: {
    archive: string;
    restored_files: number;
    total_bytes: number;
    archive_sha256: string;
    manifest_sha256: string;
    verified_on_disk: boolean;
    pre_restore_snapshot: string | null;
  };
  backups: BackupListEntry[];
};

/** Full-instance backups exist on native Ergon (never in standalone browser mode). */
export function hasBackups(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone() || !status || status === "error") return false;
  return status.capabilities?.backups === true;
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

const CRON_MONTHS: Record<string, number> = {
  jan: 1, feb: 2, mar: 3, apr: 4, may: 5, jun: 6,
  jul: 7, aug: 8, sep: 9, oct: 10, nov: 11, dec: 12,
};
const CRON_DOW: Record<string, number> = {
  sun: 0, mon: 1, tue: 2, wed: 3, thu: 4, fri: 5, sat: 6,
};

function parseCronToken(token: string, min: number, max: number, names?: Record<string, number>): number | null {
  const clean = token.trim().toLowerCase();
  if (!clean) return null;
  if (names && clean in names) return names[clean];
  if (!/^\d+$/.test(clean)) return null;
  const n = Number(clean);
  if (!Number.isInteger(n) || n < min || n > max) return null;
  return n;
}

function parseCronField(
  raw: string,
  min: number,
  max: number,
  names?: Record<string, number>,
  wrapSevenToZero = false
): Set<number> | null {
  const field = raw.trim().toLowerCase();
  if (!field) return null;
  const out = new Set<number>();
  for (const part of field.split(",")) {
    const seg = part.trim();
    if (!seg) return null;
    const slashParts = seg.split("/");
    if (slashParts.length > 2) return null;
    const [rangePart, stepPart] = slashParts;
    let step = 1;
    if (stepPart !== undefined) {
      if (!/^\d+$/.test(stepPart)) return null;
      step = Number(stepPart);
      if (!Number.isInteger(step) || step <= 0) return null;
    }
    let start = min;
    let end = max;
    if (rangePart === "*") {
      start = min;
      end = max;
    } else if (rangePart.includes("-")) {
      const dashParts = rangePart.split("-");
      if (dashParts.length !== 2) return null;
      const a = parseCronToken(dashParts[0], min, max, names);
      const b = parseCronToken(dashParts[1], min, max, names);
      if (a === null || b === null || a > b) return null;
      start = a;
      end = b;
    } else {
      const single = parseCronToken(rangePart, min, max, names);
      if (single === null) return null;
      start = single;
      end = stepPart !== undefined ? max : single;
    }
    for (let v = start; v <= end; v += step) {
      out.add(wrapSevenToZero && v === 7 ? 0 : v);
    }
  }
  return out.size > 0 ? out : null;
}

export type ParsedCron = {
  minutes: Set<number>;
  hours: Set<number>;
  dom: Set<number>;
  months: Set<number>;
  dow: Set<number>;
  domWildcard: boolean;
  dowWildcard: boolean;
};

export function parseCronExpression(expr: string): ParsedCron | null {
  const parts = String(expr || "").trim().split(/\s+/);
  if (parts.length !== 5) return null;
  const minutes = parseCronField(parts[0], 0, 59);
  const hours = parseCronField(parts[1], 0, 23);
  const dom = parseCronField(parts[2], 1, 31);
  const months = parseCronField(parts[3], 1, 12, CRON_MONTHS);
  const dow = parseCronField(parts[4], 0, 7, CRON_DOW, true);
  if (!minutes || !hours || !dom || !months || !dow) return null;
  return {
    minutes,
    hours,
    dom,
    months,
    dow,
    domWildcard: parts[2].trim() === "*",
    dowWildcard: parts[4].trim() === "*",
  };
}

export function isCronExpression(expr: string): boolean {
  return String(expr || "").trim().split(/\s+/).length === 5;
}

export function validateCronExpression(expr: string): { valid: boolean; error?: string; summary?: string } {
  const trimmed = String(expr || "").trim();
  if (!trimmed) return { valid: false, error: "Schedule cannot be empty." };
  const parts = trimmed.split(/\s+/);
  if (parts.length !== 5) {
    return { valid: false, error: "Cron expression must have 5 fields: min hour day month weekday." };
  }
  const parsed = parseCronExpression(trimmed);
  if (!parsed) {
    return { valid: false, error: "Invalid 5-field cron syntax (check ranges like 0-59, 0-23, 1-31, 1-12, 0-6)." };
  }
  return { valid: true, summary: `Valid 5-field cron (${parts.join(" ")})` };
}

export function validateRoutineSchedule(schedule: string): { valid: boolean; error?: string } {
  const trimmed = String(schedule || "").trim();
  if (!trimmed) return { valid: false, error: "Schedule cannot be empty." };
  const raw = trimmed.toLowerCase();
  if (raw === "manual" || raw === "hourly" || raw === "every hour") {
    return { valid: true };
  }
  const everyMatch = /^every\s+(\d+)\s*(s|sec|secs|seconds?|m|min|mins|minutes?|h|hr|hrs|hours?|d|day|days?)$/.exec(raw);
  if (everyMatch) {
    return Number(everyMatch[1]) >= 1
      ? { valid: true }
      : { valid: false, error: "Interval must be at least 1." };
  }
  const dailyMatch = /^(?:daily|every\s+day|every\s+weekday)(?:\s+(\d{1,2}):(\d{2}))?$/.exec(raw);
  if (dailyMatch) {
    if (dailyMatch[1] !== undefined) {
      const hh = Number(dailyMatch[1]);
      const mm = Number(dailyMatch[2]);
      if (hh > 23 || mm > 59) {
        return { valid: false, error: "Time must use 24-hour HH:MM (00:00 to 23:59)." };
      }
    }
    return { valid: true };
  }
  const weeklyMatch = /^(?:weekly|every)\s+(sun|mon|tue|wed|thu|fri|sat)[a-z]*(?:\s+(\d{1,2}):(\d{2}))?$/.exec(raw);
  if (weeklyMatch) {
    if (weeklyMatch[2] !== undefined) {
      const hh = Number(weeklyMatch[2]);
      const mm = Number(weeklyMatch[3]);
      if (hh > 23 || mm > 59) {
        return { valid: false, error: "Time must use 24-hour HH:MM (00:00 to 23:59)." };
      }
    }
    return { valid: true };
  }
  return validateCronExpression(trimmed);
}

export function validateRoutineTimezone(tz: string): { valid: boolean; error?: string } {
  const zone = String(tz || "").trim();
  if (!zone) return { valid: false, error: "Select a timezone." };
  if (zone.toUpperCase() === "UTC") return { valid: true };
  try {
    new Intl.DateTimeFormat("en-US", { timeZone: zone }).format(0);
    return { valid: true };
  } catch {
    return { valid: false, error: `Unknown IANA timezone "${zone}".` };
  }
}

export function resolveTzOffsetMins(tz?: string, tzOffsetMins?: number | null, atMs = Date.now()): number {
  if (typeof tzOffsetMins === "number" && Number.isFinite(tzOffsetMins)) {
    return Math.round(tzOffsetMins);
  }
  const zone = String(tz || "").trim();
  if (zone.toUpperCase() === "UTC") return 0;
  if (zone && typeof Intl !== "undefined" && Intl.DateTimeFormat) {
    try {
      const parts = new Intl.DateTimeFormat("en-US", {
        timeZone: zone,
        timeZoneName: "shortOffset",
      }).formatToParts(new Date(atMs));
      const tzPart = parts.find((p) => p.type === "timeZoneName")?.value ?? "";
      if (tzPart === "GMT" || tzPart === "UTC") return 0;
      const m = /^GMT([+-])(\d{1,2})(?::?(\d{2}))?$/i.exec(tzPart);
      if (m) {
        const sign = m[1] === "-" ? -1 : 1;
        return sign * (Number(m[2]) * 60 + Number(m[3] || 0));
      }
    } catch {
      // Fallback to local offset below
    }
  }
  return -new Date(atMs).getTimezoneOffset();
}

function nextCronRunMs(cron: ParsedCron, fromMs: number, offsetMins: number): number | null {
  const offsetMs = offsetMins * 60_000;
  const startMinute = Math.floor((fromMs + offsetMs) / 60_000) + 1;
  const cursor = new Date(startMinute * 60_000);
  const limitMs = startMinute * 60_000 + 366 * 86_400_000;
  while (cursor.getTime() <= limitMs) {
    const mon = cursor.getUTCMonth() + 1;
    if (!cron.months.has(mon)) {
      cursor.setUTCMonth(cursor.getUTCMonth() + 1, 1);
      cursor.setUTCHours(0, 0, 0, 0);
      continue;
    }
    const dom = cursor.getUTCDate();
    const dow = cursor.getUTCDay();
    const dayMatches =
      cron.domWildcard && cron.dowWildcard
        ? true
        : cron.domWildcard
          ? cron.dow.has(dow)
          : cron.dowWildcard
            ? cron.dom.has(dom)
            : cron.dom.has(dom) || cron.dow.has(dow);
    if (!dayMatches) {
      cursor.setUTCDate(dom + 1);
      cursor.setUTCHours(0, 0, 0, 0);
      continue;
    }
    const hr = cursor.getUTCHours();
    if (!cron.hours.has(hr)) {
      cursor.setUTCHours(hr + 1, 0, 0, 0);
      continue;
    }
    const min = cursor.getUTCMinutes();
    if (!cron.minutes.has(min)) {
      cursor.setUTCMinutes(min + 1, 0, 0);
      continue;
    }
    return cursor.getTime() - offsetMs;
  }
  return null;
}

/** Parses routine cadence strings ("every 15m", "hourly", "daily 09:00", "weekly mon",
 *  5-field cron, or interval_secs) with optional timezone offset to estimate next execution epoch ms. */
export function estimateNextRunMs(
  schedule: string,
  fromMs = Date.now(),
  intervalSecs?: number | null,
  tzOffsetMins?: number | null,
  tz?: string
): number | null {
  if (typeof intervalSecs === "number" && Number.isFinite(intervalSecs) && intervalSecs > 0) {
    return fromMs + intervalSecs * 1000;
  }
  const trimmed = String(schedule || "").trim();
  const raw = trimmed.toLowerCase();
  if (!raw || raw === "manual") return null;
  const cron = parseCronExpression(trimmed);
  if (cron) {
    const offset = resolveTzOffsetMins(tz, tzOffsetMins, fromMs);
    return nextCronRunMs(cron, fromMs, offset);
  }
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
    const offset = tz || typeof tzOffsetMins === "number" ? resolveTzOffsetMins(tz, tzOffsetMins, fromMs) : null;
    if (offset !== null) {
      const offsetMs = offset * 60_000;
      const next = new Date(fromMs + offsetMs);
      next.setUTCSeconds(0, 0);
      next.setUTCHours(hh, mm, 0, 0);
      if (next.getTime() <= fromMs + offsetMs) next.setUTCDate(next.getUTCDate() + 1);
      if (raw.includes("weekday")) {
        while (next.getUTCDay() === 0 || next.getUTCDay() === 6) next.setUTCDate(next.getUTCDate() + 1);
      }
      return next.getTime() - offsetMs;
    }
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
  opts: { method?: string; body?: unknown; signal?: AbortSignal; headers?: Record<string, string> } = {}
): Promise<Res<T>> {
  try {
    const headers: Record<string, string> = { ...(opts.headers ?? {}) };
    if (opts.body !== undefined && !("Content-Type" in headers)) {
      headers["Content-Type"] = "application/json";
    }
    const r = await fetch(path, {
      method: opts.method ?? "GET",
      cache: "no-store",
      signal: opts.signal,
      headers: Object.keys(headers).length > 0 ? headers : undefined,
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
  opts: { title?: string; force?: boolean; engagement_id?: string; client_id?: string } = {}
): Promise<HandoffZipDownloadResult> {
  const resp = await fetch("/api/export/handoff", {
    method: "POST",
    cache: "no-store",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      title: opts.title || "Client Handoff",
      force: Boolean(opts.force),
      ...(opts.engagement_id ? { engagement_id: opts.engagement_id } : {}),
      ...(opts.client_id ? { client_id: opts.client_id } : {}),
    }),
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

export async function cancelLiveRun(runId: string): Promise<Res<{ cancelled?: boolean; run_id?: string }>> {
  return call<{ cancelled?: boolean; run_id?: string }>(`/api/runs/${encodeURIComponent(runId || "active")}/cancel`, {
    method: "POST",
    body: {},
  });
}

export async function resumePipelineRun(
  runId: string,
  body: PipelineResumePayload
): Promise<Res<{ run: PipelineRun }>> {
  const decision = body.decision ?? body.action ?? "approve";
  const text = body.text ?? body.choice ?? body.answer;
  return call<{ run: PipelineRun }>(`/api/pipelines/runs/${encodeURIComponent(runId)}/resume`, {
    method: "POST",
    body: {
      ...(body.checkpoint_id ? { checkpoint_id: body.checkpoint_id } : {}),
      decision,
      ...(text !== undefined ? { text } : {}),
      action: body.action ?? decision,
      ...(body.answer !== undefined ? { answer: body.answer } : text !== undefined ? { answer: text } : {}),
      ...(body.choice !== undefined ? { choice: body.choice } : {}),
      ...(body.override_args !== undefined ? { override_args: body.override_args } : {}),
    },
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

export function canExportPdf(path: string, _extractedFormat?: "markdown" | "csv"): boolean {
  const ext = (path.split(".").pop() || "").toLowerCase();
  return ext === "md" || ext === "markdown" || ext === "txt" || ext === "csv" || ext === "tsv";
}

export type TriggerKind = "webhook" | "folder_watch" | "imap_poll";

export type ImapSecurity = "imaps" | "starttls" | "plain_loopback_test" | (string & {});

export type ImapConfig = {
  host: string;
  port: number;
  security: ImapSecurity;
  folder: string;
  poll_interval_secs: number;
  mark_seen: boolean;
  username_secret_key: string;
  password_secret_key: string;
  subject_filter?: string;
};

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
  imap_config?: ImapConfig;
  last_triggered_ms?: number | null;
  trigger_count?: number;
  created_ms?: number;
  updated_ms?: number;
};

export type OutboundConnectorKind = "http_webhook" | "rest_api" | "smtp_relay";

export type SmtpSecurity = "smtps" | "starttls" | "plain_loopback_test";

export type SmtpConfig = {
  host: string;
  port: number;
  security: SmtpSecurity;
  username_secret_key: string;
  password_secret_key: string;
  from_addr: string;
  default_to: string[];
};

export type OAuthConfig = {
  token_url: string;
  client_id: string;
  client_secret_key: string;
  refresh_token_secret_key: string;
  scopes: string[];
  access_token_expires_ms?: number;
};

export type OutboundConnector = {
  id: string;
  name: string;
  kind: OutboundConnectorKind;
  url: string;
  method: "POST" | "PUT";
  headers_template?: Record<string, string>;
  secret_keys?: string[];
  has_secret?: boolean;
  smtp_config?: SmtpConfig;
  oauth_config?: OAuthConfig;
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
  imap_config?: ImapConfig;
  secrets?: Record<string, string>;
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
  smtp_config?: SmtpConfig;
  oauth_config?: OAuthConfig;
  require_approval: boolean;
  enabled?: boolean;
}): Promise<Res<{ connector?: OutboundConnector }>> {
  return call<{ connector?: OutboundConnector }>("/api/connectors/outbound", {
    method: "POST",
    body,
  });
}

export type McpTransport = "stdio" | "http_sse";

export type McpToolSpec = {
  name: string;
  description?: string;
  input_schema?: unknown;
};

export type McpServerSpec = {
  id: string;
  name: string;
  transport: McpTransport;
  command_or_url: string;
  args: string[];
  env_secret_keys: string[];
  require_approval_by_default: boolean;
  enabled: boolean;
  cached_tools?: McpToolSpec[];
  last_discovered_ms?: number | null;
  last_error?: string | null;
  created_ms?: number;
  updated_ms?: number;
};

export function hasMcpApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.mcp === "boolean") {
    return Boolean(status.capabilities.mcp);
  }
  return isAtLeastMinor(status.version, 38);
}

export async function listMcpServers(): Promise<Res<{ servers: McpServerSpec[] }>> {
  const r = await call<{ servers?: McpServerSpec[] }>("/api/mcp/servers");
  if (!r.ok) return r;
  return {
    ok: true,
    data: {
      servers: Array.isArray(r.data.servers) ? r.data.servers : [],
    },
  };
}

export async function saveMcpServer(body: {
  id?: string;
  name: string;
  transport: McpTransport;
  command_or_url: string;
  args: string[];
  env_secret_keys: string[];
  require_approval_by_default: boolean;
  enabled: boolean;
}): Promise<Res<{ server?: McpServerSpec; servers?: McpServerSpec[] }>> {
  return call<{ server?: McpServerSpec; servers?: McpServerSpec[] }>("/api/mcp/servers", {
    method: "POST",
    body,
  });
}

export async function discoverMcpServer(id: string): Promise<
  Res<{
    server?: McpServerSpec;
    tools?: McpToolSpec[];
    cached_tools?: McpToolSpec[];
    discovered_count?: number;
  }>
> {
  return call(`/api/mcp/servers/${encodeURIComponent(id)}/discover`, {
    method: "POST",
    body: {},
  });
}

export async function deleteMcpServer(id: string): Promise<Res<{ deleted?: string }>> {
  return call<{ deleted?: string }>(`/api/mcp/servers/${encodeURIComponent(id)}`, {
    method: "DELETE",
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

export function hasClientCrmInvoicingApi(status: Status | null | "error" | undefined): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (typeof status.capabilities?.invoicing === "boolean") {
    return Boolean(status.capabilities.invoicing);
  }
  if (typeof status.capabilities?.client_crm === "boolean") {
    return Boolean(status.capabilities.client_crm);
  }
  return isAtLeastMinor(status.version, 37);
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
  contacts?: ClientContact[];
  notes_md?: string;
  tax_rate_pct?: number;
  payment_terms?: string;
}): Promise<Res<{ client: Client }>> {
  return call<{ client: Client }>("/api/clients", {
    method: "POST",
    body,
  });
}

export async function updateClient(
  id: string,
  body: {
    name?: string;
    brand_header?: string;
    billing_currency?: string;
    default_rate_usd?: number;
    contacts?: ClientContact[];
    notes_md?: string;
    tax_rate_pct?: number;
    payment_terms?: string;
  }
): Promise<Res<{ client: Client }>> {
  return call<{ client: Client }>(`/api/clients/${encodeURIComponent(id)}`, {
    method: "PATCH",
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
  workspace_prefix?: string;
  sow_rules?: string[];
  rollover_mode?: RolloverMode;
}): Promise<Res<{ engagement: Engagement; active_engagement_id?: string }>> {
  return call<{ engagement: Engagement; active_engagement_id?: string }>("/api/engagements", {
    method: "POST",
    body,
  });
}

export async function updateEngagement(
  id: string,
  body: {
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
): Promise<Res<{ engagement: Engagement }>> {
  return call<{ engagement: Engagement }>(`/api/engagements/${encodeURIComponent(id)}`, {
    method: "PATCH",
    body,
  });
}

export async function fetchEngagementContext(id: string): Promise<Res<EngagementContext>> {
  const r = await call<Record<string, unknown>>(`/api/engagements/${encodeURIComponent(id)}/context`);
  if (!r.ok) return r;
  const raw = r.data || {};
  const client = (raw.client as Client | null | undefined) ?? null;
  const primaryContact = raw.primary_contact as ClientContact | null | undefined;
  const contacts = Array.isArray(raw.contacts)
    ? (raw.contacts as ClientContact[])
    : Array.isArray(client?.contacts)
      ? client.contacts
      : primaryContact
        ? [primaryContact]
        : [];
  const rawEntries = Array.isArray(raw.unbilled_entries)
    ? raw.unbilled_entries
    : Array.isArray(raw.recent_time_entries)
      ? raw.recent_time_entries.filter((e: unknown) => {
          const obj = (e || {}) as Record<string, unknown>;
          return !obj.invoiced_id && !obj.invoiced_invoice_id;
        })
      : [];
  const retainer = (raw.retainer as Record<string, unknown> | undefined) ?? undefined;
  const burnMetrics =
    (raw.burn_metrics as EngagementContext["burn_metrics"] | undefined) ??
    (retainer
      ? {
          cost_usd: Number(retainer.llm_cost_usd ?? 0),
          retainer_budget_usd: Number(retainer.retainer_budget_usd ?? 0),
          retainer_burn_pct: Number(retainer.burn_pct ?? 0),
        }
      : undefined);
  return {
    ok: true,
    data: {
      engagement: raw.engagement as Engagement,
      client,
      scoped_file_count: Number(raw.scoped_file_count ?? raw.scoped_files_count ?? 0),
      workspace_prefix: typeof raw.workspace_prefix === "string" ? raw.workspace_prefix : undefined,
      sow_rules: Array.isArray(raw.sow_rules) ? (raw.sow_rules as string[]) : [],
      contacts,
      burn_metrics: burnMetrics,
      unbilled_entries: rawEntries.map((e) => normalizeTimeEntry(e as Record<string, unknown>)),
      unbilled_total_usd:
        typeof raw.unbilled_total_usd === "number"
          ? raw.unbilled_total_usd
          : typeof retainer?.unbilled_amount_usd === "number"
            ? retainer.unbilled_amount_usd
            : undefined,
    },
  };
}

function normalizeTimeEntry(raw: Record<string, unknown>): TimeExpenseEntry {
  const invoicedId =
    typeof raw.invoiced_id === "string"
      ? raw.invoiced_id
      : typeof raw.invoiced_invoice_id === "string"
        ? raw.invoiced_invoice_id
        : null;
  return {
    ...(raw as unknown as TimeExpenseEntry),
    invoiced_id: invoicedId,
  };
}

function normalizeInvoiceRecord(raw: Record<string, unknown>): InvoiceRecord {
  const lineItems = Array.isArray(raw.line_items) ? (raw.line_items as InvoiceLineItem[]) : [];
  const deliverables = Array.isArray(raw.deliverable_paths)
    ? raw.deliverable_paths.map(String)
    : [];
  const llmFromItems = lineItems
    .filter((li) => li.kind === "llm_usage")
    .reduce((acc, li) => acc + (Number(li.amount_usd) || 0), 0);
  return {
    ...(raw as unknown as InvoiceRecord),
    issued_ms: typeof raw.issued_ms === "number" ? raw.issued_ms : Number(raw.created_ms ?? 0),
    line_items: lineItems,
    llm_cost_usd: typeof raw.llm_cost_usd === "number" ? raw.llm_cost_usd : Math.round(llmFromItems * 100) / 100,
    billable_subtotal_usd: Number(raw.billable_subtotal_usd ?? raw.subtotal_usd ?? 0),
    tax_usd: Number(raw.tax_usd ?? raw.tax_amount_usd ?? 0),
    total_usd: Number(raw.total_usd ?? 0),
    deliverable_paths: deliverables,
    pdf_path:
      typeof raw.pdf_path === "string"
        ? raw.pdf_path
        : deliverables.find((p) => p.toLowerCase().endsWith(".pdf")) ?? null,
    xlsx_path:
      typeof raw.xlsx_path === "string"
        ? raw.xlsx_path
        : deliverables.find((p) => p.toLowerCase().endsWith(".xlsx")) ?? null,
    json_path:
      typeof raw.json_path === "string"
        ? raw.json_path
        : deliverables.find((p) => p.toLowerCase().endsWith(".json")) ?? null,
    sha256: typeof raw.sha256 === "string" ? raw.sha256 : "",
  };
}

export async function switchEngagement(
  engagementId: string
): Promise<Res<{ active_engagement_id: string; engagement?: Engagement | null }>> {
  return call<{ active_engagement_id: string; engagement?: Engagement | null }>("/api/engagements/switch", {
    method: "POST",
    body: { engagement_id: engagementId },
  });
}

export async function fetchTimeEntries(opts: {
  engagement_id?: string;
  client_id?: string;
  unbilled_only?: boolean;
} = {}): Promise<Res<{ entries: TimeExpenseEntry[]; total_usd?: number }>> {
  const params = new URLSearchParams();
  if (opts.engagement_id) params.set("engagement_id", opts.engagement_id);
  if (opts.client_id) params.set("client_id", opts.client_id);
  if (typeof opts.unbilled_only === "boolean") params.set("unbilled", opts.unbilled_only ? "1" : "0");
  const qs = params.toString();
  const r = await call<{
    entries?: Record<string, unknown>[];
    time_entries?: Record<string, unknown>[];
    total_usd?: number;
    totals?: { billable_amount_usd?: number; amount_usd?: number };
  }>(`/api/time-entries${qs ? `?${qs}` : ""}`);
  if (!r.ok) return r;
  const rawList = Array.isArray(r.data.entries)
    ? r.data.entries
    : Array.isArray(r.data.time_entries)
      ? r.data.time_entries
      : [];
  const totalUsd =
    typeof r.data.total_usd === "number"
      ? r.data.total_usd
      : typeof r.data.totals?.billable_amount_usd === "number"
        ? r.data.totals.billable_amount_usd
        : typeof r.data.totals?.amount_usd === "number"
          ? r.data.totals.amount_usd
          : undefined;
  return {
    ok: true,
    data: {
      entries: rawList.map(normalizeTimeEntry),
      total_usd: totalUsd,
    },
  };
}

export async function createTimeEntry(body: {
  client_id?: string;
  engagement_id: string;
  kind: TimeExpenseKind;
  description: string;
  hours?: number;
  rate_usd?: number;
  amount_usd?: number;
  billable?: boolean;
  thread_id?: string;
  receipt_id?: string;
}): Promise<Res<{ entry: TimeExpenseEntry }>> {
  const r = await call<{ entry: Record<string, unknown> }>("/api/time-entries", {
    method: "POST",
    body,
  });
  if (!r.ok) return r;
  return { ok: true, data: { entry: normalizeTimeEntry(r.data.entry) } };
}

export async function deleteTimeEntry(id: string): Promise<Res<{ deleted?: string; id?: string }>> {
  const r = await call<{ deleted?: string; deleted_id?: string; id?: string }>(
    `/api/time-entries/${encodeURIComponent(id)}`,
    { method: "DELETE" }
  );
  if (!r.ok) return r;
  return {
    ok: true,
    data: {
      deleted: r.data.deleted ?? r.data.deleted_id ?? r.data.id,
      id: r.data.id ?? r.data.deleted_id ?? r.data.deleted,
    },
  };
}

export async function fetchInvoices(opts: {
  engagement_id?: string;
  client_id?: string;
} = {}): Promise<Res<{ invoices: InvoiceRecord[] }>> {
  const params = new URLSearchParams();
  if (opts.engagement_id) params.set("engagement_id", opts.engagement_id);
  if (opts.client_id) params.set("client_id", opts.client_id);
  const qs = params.toString();
  const r = await call<{ invoices?: Record<string, unknown>[] }>(`/api/invoices${qs ? `?${qs}` : ""}`);
  if (!r.ok) return r;
  return {
    ok: true,
    data: {
      invoices: Array.isArray(r.data.invoices) ? r.data.invoices.map(normalizeInvoiceRecord) : [],
    },
  };
}

export async function generateInvoice(body: {
  engagement_id: string;
  period_label: string;
  include_llm_cost: boolean;
  include_time_entries: boolean;
  export_formats: Array<"pdf" | "xlsx" | "json">;
}): Promise<Res<{ invoice: InvoiceRecord; exported_paths?: Record<string, string> }>> {
  const r = await call<{
    invoice: Record<string, unknown>;
    exported_paths?: Record<string, string>;
    deliverables?: string[];
  }>("/api/invoices/generate", {
    method: "POST",
    body,
  });
  if (!r.ok) return r;
  const invoice = normalizeInvoiceRecord(r.data.invoice);
  const exportedPaths: Record<string, string> | undefined =
    r.data.exported_paths ??
    (Array.isArray(r.data.deliverables) || invoice.deliverable_paths.length > 0
      ? {
          ...(invoice.pdf_path ? { pdf: invoice.pdf_path } : {}),
          ...(invoice.xlsx_path ? { xlsx: invoice.xlsx_path } : {}),
          ...(invoice.json_path ? { json: invoice.json_path } : {}),
        }
      : undefined);
  return { ok: true, data: { invoice, exported_paths: exportedPaths } };
}

export async function updateInvoiceStatus(
  id: string,
  body: { status: InvoiceStatus }
): Promise<Res<{ invoice: InvoiceRecord }>> {
  const r = await call<{ invoice: Record<string, unknown> }>(`/api/invoices/${encodeURIComponent(id)}`, {
    method: "PATCH",
    body,
  });
  if (!r.ok) return r;
  return { ok: true, data: { invoice: normalizeInvoiceRecord(r.data.invoice) } };
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
  client_id?: string;
  engagement_id?: string;
  signal?: AbortSignal;
}): Promise<Res<VaultSearchResponse>> {
  const params = new URLSearchParams();
  params.set("q", opts.q);
  if (opts.scope) params.set("scope", opts.scope);
  if (opts.mode) params.set("mode", opts.mode);
  if (typeof opts.limit === "number") params.set("limit", String(opts.limit));
  if (opts.path_prefix) params.set("path_prefix", opts.path_prefix);
  if (opts.client_id) params.set("client_id", opts.client_id);
  if (opts.engagement_id) params.set("engagement_id", opts.engagement_id);
  return call<VaultSearchResponse>(`/api/search?${params.toString()}`, { signal: opts.signal });
}

export async function searchWorkspace(
  q: string,
  opts: { limit?: number; scope?: VaultSearchScope; mode?: VaultSearchMode; client_id?: string; engagement_id?: string; signal?: AbortSignal } = {}
): Promise<Res<VaultSearchResponse>> {
  return searchVault({ q, ...opts });
}

export async function reindexVault(): Promise<Res<VaultReindexResult>> {
  return call<VaultReindexResult>("/api/vault/reindex", { method: "POST", body: {} });
}

export function engagementWorkspacePrefix(eng?: Engagement | null): string {
  if (!eng) return "";
  const raw = String(eng.workspace_prefix ?? "").trim().replace(/^\/+|\/+$/g, "");
  if (raw) return raw;
  if (eng.client_id && eng.id) return `clients/${eng.client_id}/${eng.id}`;
  return "";
}

export function isPathInEngagementScope(path: string, eng?: Engagement | null): boolean {
  const prefix = engagementWorkspacePrefix(eng);
  if (!prefix) return true;
  const clean = String(path ?? "").trim().replace(/^\/+/, "");
  return clean === prefix || clean.startsWith(`${prefix}/`) || clean.startsWith("shared/");
}

export async function aggregateVault(body: VaultAggregateRequest): Promise<Res<VaultAggregateResult>> {
  return call<VaultAggregateResult>("/api/vault/aggregate", { method: "POST", body });
}

const LOCAL_HELP_PREFS_KEY = "ergon-help-prefs-v1";

function loadLocalHelpFallback(): HelpConfigPayload {
  try {
    const raw = localStorage.getItem(LOCAL_HELP_PREFS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        onboarding_completed: Boolean(parsed?.onboarding_completed),
        onboarding_completed_at_ms: typeof parsed?.onboarding_completed_at_ms === "number" ? parsed.onboarding_completed_at_ms : null,
        admin_contact: {
          name: String(parsed?.admin_contact?.name ?? ""),
          email: String(parsed?.admin_contact?.email ?? ""),
          phone: String(parsed?.admin_contact?.phone ?? ""),
          channel_url: String(parsed?.admin_contact?.channel_url ?? ""),
          notes: String(parsed?.admin_contact?.notes ?? ""),
          configured: Boolean(
            parsed?.admin_contact?.name ||
            parsed?.admin_contact?.email ||
            parsed?.admin_contact?.phone ||
            parsed?.admin_contact?.channel_url ||
            parsed?.admin_contact?.notes
          ),
          source: "local",
        },
        can_edit_admin_contact: true,
        diagnostics_url: "/api/diagnostics/bundle",
      };
    }
  } catch { /* ignore */ }
  return {
    onboarding_completed: false,
    onboarding_completed_at_ms: null,
    admin_contact: {
      name: "",
      email: "",
      phone: "",
      channel_url: "",
      notes: "",
      configured: false,
      source: "default",
    },
    can_edit_admin_contact: true,
    diagnostics_url: "/api/diagnostics/bundle",
  };
}

export async function fetchHelpConfig(): Promise<Res<HelpConfigPayload>> {
  if (isWebStandalone()) {
    return { ok: true, data: loadLocalHelpFallback() };
  }
  return call<HelpConfigPayload>("/api/help/config");
}

export async function saveHelpConfig(body: {
  onboarding_completed?: boolean;
  admin_contact?: Partial<Pick<AdminContactConfig, "name" | "email" | "phone" | "channel_url" | "notes">>;
}): Promise<Res<HelpConfigPayload>> {
  if (isWebStandalone()) {
    const cur = loadLocalHelpFallback();
    const nextContact = {
      ...cur.admin_contact,
      ...(body.admin_contact ?? {}),
    };
    nextContact.configured = Boolean(
      nextContact.name.trim() ||
      nextContact.email.trim() ||
      nextContact.phone.trim() ||
      nextContact.channel_url.trim() ||
      nextContact.notes.trim()
    );
    const next: HelpConfigPayload = {
      ...cur,
      onboarding_completed:
        typeof body.onboarding_completed === "boolean" ? body.onboarding_completed : cur.onboarding_completed,
      onboarding_completed_at_ms:
        typeof body.onboarding_completed === "boolean"
          ? body.onboarding_completed
            ? Date.now()
            : null
          : cur.onboarding_completed_at_ms,
      admin_contact: nextContact,
    };
    try {
      localStorage.setItem(LOCAL_HELP_PREFS_KEY, JSON.stringify(next));
    } catch { /* ignore */ }
    return { ok: true, data: next };
  }
  return call<HelpConfigPayload>("/api/help/config", { method: "POST", body });
}

export async function fetchDiagnosticsSummary(): Promise<Res<DiagnosticsSummary>> {
  if (isWebStandalone()) {
    return {
      ok: true,
      data: {
        app_version: "0.38.0",
        os: "browser",
        arch: "web",
        uptime_secs: 0,
        workspace_count: 1,
        model_configured: false,
        log_files: 0,
        crash_reports: 0,
      },
    };
  }
  return call<DiagnosticsSummary>("/api/diagnostics/summary");
}

export async function downloadDiagnosticBundle(): Promise<
  | { ok: true; filename: string; sha256: string; bytes: number }
  | { ok: false; error: string }
> {
  if (isWebStandalone()) {
    return { ok: false, error: "Diagnostic bundles are generated by the local Ergon desktop/server binary." };
  }
  try {
    const resp = await fetch("/api/diagnostics/bundle", {
      method: "GET",
      credentials: "same-origin",
    });
    if (!resp.ok) {
      let msg = `Download failed (HTTP ${resp.status})`;
      try {
        const j = await resp.json();
        if (typeof j?.error === "string" && j.error.trim()) msg = j.error;
      } catch { /* ignore */ }
      return { ok: false, error: msg };
    }
    const blob = await resp.blob();
    const cd = resp.headers.get("content-disposition") || "";
    const match = /filename="?([^";\r\n]+)"?/i.exec(cd);
    const filename = match?.[1] || `ergon-diagnostics-${new Date().toISOString().slice(0, 10)}.zip`;
    const sha256 = resp.headers.get("x-ergon-diagnostics-sha256") || "";
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1500);
    return { ok: true, filename, sha256, bytes: blob.size };
  } catch (e) {
    return {
      ok: false,
      error: e instanceof Error ? e.message : "Could not download diagnostic bundle.",
    };
  }
}

// ---------------------------------------------------------------------------
// Multi-user Auth, Sessions, OIDC & RBAC API
// ---------------------------------------------------------------------------

export type AuthRole = "admin" | "pm" | "ae" | "office" | "installer" | "member";

export type AuthPrincipal = {
  id: string;
  username: string;
  display_name: string;
  email?: string | null;
  role: AuthRole;
  person_id: string;
  active_person_id: string;
  auth_method?: string;
  is_admin: boolean;
  break_glass?: boolean;
};

export type AuthMePayload = {
  authenticated: boolean;
  auth_required: boolean;
  needs_setup: boolean;
  oidc_enabled: boolean;
  tls: boolean;
  user: AuthPrincipal | null;
  roles?: AuthRole[];
};

export type AuthUserRecord = {
  id: string;
  username: string;
  email?: string | null;
  display_name: string;
  role: AuthRole;
  person_id: string;
  has_password: boolean;
  oidc_linked: boolean;
  disabled: boolean;
  created_ms?: number;
  updated_ms?: number;
  last_login_ms?: number | null;
};

export type OidcConfigPayload = {
  enabled: boolean;
  configured: boolean;
  issuer: string;
  client_id: string;
  has_client_secret: boolean;
  redirect_uri: string;
  scopes: string;
  default_role: AuthRole;
  role_claim: string;
  auto_provision: boolean;
};

export async function fetchAuthMe(): Promise<Res<AuthMePayload>> {
  return call<AuthMePayload>("/api/auth/me");
}

export async function setupInitialAdmin(body: {
  username: string;
  password: string;
  display_name?: string;
  email?: string;
}): Promise<Res<{ ok: boolean; user: AuthUserRecord; token: string; expires_at_ms: number }>> {
  return call("/api/auth/setup", { method: "POST", body });
}

export async function loginLocal(body: {
  username: string;
  password: string;
}): Promise<Res<{ ok: boolean; user: AuthUserRecord; token: string; expires_at_ms: number }>> {
  return call("/api/auth/login", { method: "POST", body });
}

export async function logoutSession(): Promise<Res<{ ok: boolean }>> {
  return call("/api/auth/logout", { method: "POST", body: {} });
}

export async function changeOwnPassword(body: {
  current_password: string;
  new_password: string;
}): Promise<Res<{ ok: boolean }>> {
  return call("/api/auth/password", { method: "POST", body });
}

export async function listAuthUsers(): Promise<Res<{ users: AuthUserRecord[]; roles: AuthRole[] }>> {
  return call("/api/auth/users");
}

export async function createAuthUser(body: {
  username: string;
  password: string;
  role: AuthRole;
  display_name?: string;
  email?: string;
}): Promise<Res<{ user: AuthUserRecord }>> {
  return call("/api/auth/users", { method: "POST", body });
}

export async function updateAuthUser(
  id: string,
  patch: {
    role?: AuthRole;
    display_name?: string;
    email?: string;
    disabled?: boolean;
    password?: string;
  }
): Promise<Res<{ user: AuthUserRecord }>> {
  return call(`/api/auth/users/${encodeURIComponent(id)}`, { method: "PATCH", body: patch });
}

export async function deleteAuthUser(id: string): Promise<Res<{ ok: boolean }>> {
  return call(`/api/auth/users/${encodeURIComponent(id)}`, { method: "DELETE" });
}

export async function fetchOidcConfig(): Promise<Res<OidcConfigPayload>> {
  return call<OidcConfigPayload>("/api/auth/oidc/config");
}

export async function saveOidcConfig(body: {
  enabled?: boolean;
  issuer?: string;
  client_id?: string;
  client_secret?: string;
  clear_client_secret?: boolean;
  redirect_uri?: string;
  scopes?: string;
  default_role?: AuthRole;
  role_claim?: string;
  auto_provision?: boolean;
}): Promise<Res<OidcConfigPayload>> {
  return call<OidcConfigPayload>("/api/auth/oidc/config", { method: "PUT", body });
}
