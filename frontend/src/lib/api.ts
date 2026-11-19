// The Ergon server's JSON API (same routes and shapes as the old page). A failed call is a value, never a throw.
export type Msg = { role: string; content: string };
export type Person = { id: string; name: string; role?: string };
export type Status = { version: string; model_configured: boolean; person?: Person; store_recovered?: string[] };
export type ThreadRow = { id: string; title?: string; messages?: number };
export type Thread = { id: string; title?: string; messages: Msg[] };
export type Update = {
  current?: string; latest?: string; available?: boolean; staged?: string; mode?: string;
  can_apply?: boolean; dev_build?: boolean; notes?: string; release_url?: string; error?: string; checked_at?: number;
};
export type Decision = { id: string; kind: string; summary: string; status: string; outcome?: string };
export type Offer = { id: string; title: string; detail: string; status: string };
export type SuiteItem = { id: string; kind: string; name: string; description: string; fields?: string[] };
export type LibraryItem = { id: string; kind: string; name: string; description: string; author: string; builtin?: boolean };

export type Res<T> = { ok: true; data: T } | { ok: false; error: string; status: number };

export async function call<T>(path: string, opts: { method?: string; body?: unknown } = {}): Promise<Res<T>> {
  try {
    const r = await fetch(path, {
      method: opts.method ?? "GET",
      cache: "no-store",
      headers: opts.body !== undefined ? { "Content-Type": "application/json" } : undefined,
      body: opts.body !== undefined ? JSON.stringify(opts.body) : undefined,
    });
    const data = (await r.json().catch(() => ({}))) as T & { error?: string };
    if (!r.ok) return { ok: false, error: data.error ?? "", status: r.status };
    return { ok: true, data };
  } catch (e) {
    return { ok: false, error: e instanceof TypeError ? "Could not reach the Ergon server." : String(e), status: 0 };
  }
}
