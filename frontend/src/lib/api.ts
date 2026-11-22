// Ergon server JSON API client. Every failed or malformed call resolves to { ok: false }, never an uncaught throw.
export type Msg = { role: string; content: string };
export type Person = { id: string; name: string; role?: string };
export type ServerCapabilities = {
  sse?: boolean;
  skills?: boolean;
  routines?: boolean;
  search?: boolean;
  receipts_export?: boolean;
  cancel?: boolean;
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
};
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
export type Routine = {
  id: string;
  name: string;
  prompt: string;
  schedule: string;
  skill_id?: string;
  enabled: boolean;
  last_run_ts?: number | null;
  last_status?: string;
  run_count?: number;
  created_ts?: number;
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
export type ThreadRow = { id: string; title?: string; messages?: number };
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
export type Decision = { id: string; kind: string; summary: string; status: string; outcome?: string; thread_id?: string; path?: string; ts?: number; resolved_ts?: number };
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
export type TodayEvent = { ts?: number; kind?: string; title?: string; detail?: string; files?: string[] };
export type TodayPayload = { date?: string; events?: TodayEvent[] };
export type GraphNode = { id: string; kind: string; label?: string; detail?: string };
export type GraphEdge = { from: string; to: string; kind?: string };
export type GraphPayload = { nodes?: GraphNode[]; edges?: GraphEdge[] };

export type Res<T> = { ok: true; data: T } | { ok: false; error: string; status: number };

export function isWebStandalone(): boolean {
  return typeof window !== "undefined" && Boolean((window as unknown as { __ERGON_WEB_ENGINE__?: boolean }).__ERGON_WEB_ENGINE__);
}

function isAtLeast050(version: string | undefined): boolean {
  if (!version) return false;
  const m = /^v?(\d+)\.(\d+)/.exec(version.trim());
  if (!m) return false;
  const major = Number(m[1]);
  const minor = Number(m[2]);
  return major > 0 || minor >= 5;
}

/** Progressive v0.5.0 API detector: avoids 404 console noise against v0.4.0 servers while
 *  lighting up live SSE, /api/skills, /api/routines, /api/search, and /api/receipts automatically on v0.5.0+. */
export function hasV050Api(
  status: Status | null | "error",
  cap?: keyof ServerCapabilities
): boolean {
  if (isWebStandalone()) return true;
  if (!status || status === "error") return false;
  if (cap && typeof status.capabilities?.[cap] === "boolean") {
    return Boolean(status.capabilities[cap]);
  }
  return isAtLeast050(status.version);
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
