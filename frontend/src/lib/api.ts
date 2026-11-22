// Ergon server JSON API client. Every failed or malformed call resolves to { ok: false }, never an uncaught throw.
export type Msg = { role: string; content: string };
export type Person = { id: string; name: string; role?: string };
export type Status = {
  version: string;
  model_configured: boolean;
  model?: string;
  model_base_url?: string;
  person?: Person;
  store_recovered?: string[];
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
export type Decision = { id: string; kind: string; summary: string; status: string; outcome?: string; thread_id?: string; ts?: number; resolved_ts?: number };
export type Offer = { id: string; title: string; detail: string; status: string; item_id?: string; ts?: number };
export type SuiteItem = { id: string; kind: string; name: string; description: string; fields?: string[] };
export type LibraryItem = { id: string; kind: string; name: string; description: string; author: string; builtin?: boolean };
export type BriefJob = { task: string; date?: string; summary?: string; files?: string[]; thread_id?: string; ts?: number };
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

export async function call<T>(path: string, opts: { method?: string; body?: unknown } = {}): Promise<Res<T>> {
  try {
    const r = await fetch(path, {
      method: opts.method ?? "GET",
      cache: "no-store",
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
    return { ok: false, error: e instanceof TypeError ? "Could not reach the Ergon server." : String(e), status: 0 };
  }
}
