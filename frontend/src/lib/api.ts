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
  tool: string;
  label?: string;
  status: "running" | "done" | "error" | "waiting";
  elapsed_ms?: number;
  started_ms?: number;
  finished_ms?: number;
  raw: string;
  file?: string;
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
export type SearchHit = {
  kind: "file" | "receipt" | "skill" | "routine" | "memory" | "thread";
  id: string;
  title: string;
  subtitle?: string;
  path?: string;
  thread_id?: string;
  ts?: number;
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
};
export type Offer = { id: string; title: string; detail: string; status: string; item_id?: string; ts?: number };
export type SuiteItem = { id: string; kind: string; name: string; description: string; fields?: string[] };
export type LibraryItem = { id: string; kind: string; name: string; description: string; author: string; builtin?: boolean };
export type BriefJob = { id?: string; task: string; date?: string; summary?: string; files?: string[]; thread_id?: string; ts?: number; sha256?: string };
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

export type Res<T> = { ok: true; data: T } | { ok: false; error: string; status: number };

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
    const data = parsed as T & { error?: unknown };
    if (!r.ok) {
      return { ok: false, error: typeof data.error === "string" ? data.error : "", status: r.status };
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
