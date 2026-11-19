// Work Graph: the work, as a map. People on the left, suite items (the agents' tools) in the middle, finished work
// and files on the right. Pins sit on a deterministic grid per band; an edge is drawn only where /api/graph returns it.
// Hover or focus on a pin brightens its own routes. The text list is the accessible equivalent and shows the same data.
import { useEffect, useMemo, useState } from "react";
import { call } from "@/lib/api";

type Node = { id: string; kind: string; label?: string; detail?: string };
type Graph = { nodes?: Node[]; edges?: { from: string; to: string }[] };

export const KIND_WORD: Record<string, string> = { person: "Person", item: "Suite item", work: "Finished work", file: "File" };
const KIND_ORDER = ["person", "item", "work", "file"];
const BAND_TITLE: Record<string, string> = { person: "People", item: "Suite items", work: "Finished work", file: "Files" };
const BAND_X: Record<string, number> = { person: 160, item: 400, work: 640, file: 870 };
const W = 1000;

/** One pin: the shape is the kind (disc, diamond, triangle, square). */
export function Pin({ kind, x, y, hot }: { kind: string; x: number; y: number; hot: boolean }) {
  const work = kind === "work";
  const style = { fill: work ? "var(--ink)" : "var(--surface)", stroke: kind === "file" ? "var(--ink-3)" : "var(--ink)" };
  const sw = hot ? 2.5 : 1.5;
  if (kind === "person") return <circle cx={x} cy={y} r={7} style={style} strokeWidth={sw} />;
  if (kind === "item") return <polygon points={`${x},${y - 8} ${x + 8},${y} ${x},${y + 8} ${x - 8},${y}`} style={style} strokeWidth={sw} />;
  if (kind === "work") return <polygon points={`${x},${y - 8} ${x + 8},${y + 7} ${x - 8},${y + 7}`} style={style} strokeWidth={sw} />;
  return <rect x={x - 6} y={y - 6} width={12} height={12} style={style} strokeWidth={sw} />;
}

/** Deterministic layout: nodes sorted by label within a band; the row step shrinks as a band fills. */
export function layout(nodes: Node[]) {
  const bands: Record<string, Node[]> = { person: [], item: [], work: [], file: [] };
  for (const n of nodes) (bands[bands[n.kind] ? n.kind : "work"]).push(n);
  for (const k of KIND_ORDER) bands[k].sort((a, b) => (a.label || "").localeCompare(b.label || "") || a.id.localeCompare(b.id));
  const rows = Math.max(1, ...KIND_ORDER.map((k) => bands[k].length));
  const step = rows <= 8 ? 84 : rows <= 14 ? 56 : 38;
  const top = 92;
  const height = top + rows * step + 40;
  const pos: Record<string, { x: number; y: number; n: Node; kind: string }> = {};
  for (const k of KIND_ORDER) bands[k].forEach((n, i) => { pos[n.id] = { x: BAND_X[k], y: top + i * step + step / 2, n, kind: k }; });
  return { pos, height, step, bands };
}

export function GraphFace() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [failed, setFailed] = useState(false);
  const [hot, setHot] = useState<string | null>(null);
  useEffect(() => {
    call<Graph>("/api/graph").then((r) => { if (r.ok) setGraph(r.data); else setFailed(true); });
  }, []);
  const nodes = graph?.nodes ?? [];
  const edges = graph?.edges ?? [];
  const lay = useMemo(() => layout(nodes), [graph]); // eslint-disable-line react-hooks/exhaustive-deps

  if (failed) return <section aria-label="Work graph" className="mx-auto w-full max-w-[1000px] px-6 pt-6"><p role="alert" className="text-sm text-ink-2">The graph could not be loaded. Is the Ergon server running?</p></section>;
  if (!graph) return <section aria-label="Work graph" className="mx-auto w-full max-w-[1000px] px-6 pt-6"><p className="text-sm text-ink-2">Loading…</p></section>;

  const count = (k: string) => nodes.filter((n) => n.kind === k).length;
  const sorted = [...nodes].sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind));
  const drawn = edges.filter((e) => lay.pos[e.from] && lay.pos[e.to]);
  const focusLabel = (id: string) => { const p = lay.pos[id]; return p ? `${KIND_WORD[p.kind] || p.kind}: ${p.n.label || ""}` : ""; };

  return (
    <section aria-label="Work graph" className="mx-auto flex w-full max-w-[1000px] flex-col gap-5 px-6 pb-10 pt-6">
      <div>
        <h2 className="text-[30px] leading-tight font-semibold tracking-[-0.035em] text-ink">Who the work belongs to, and what it touched</h2>
        <p className="mt-2 flex flex-wrap gap-x-5 gap-y-1 text-sm text-ink-2">
          <span><span className="font-mono tabular-nums text-ink">{count("person")}</span> people</span>
          <span><span className="font-mono tabular-nums text-ink">{count("work")}</span> finished jobs</span>
          <span><span className="font-mono tabular-nums text-ink">{count("file")}</span> files produced</span>
          <span><span className="font-mono tabular-nums text-ink">{count("item")}</span> suite items in play</span>
        </p>
      </div>

      {!nodes.length && <p className="text-sm text-ink-2">No finished work yet. The graph fills in as your agent finishes jobs.</p>}

      {nodes.length > 0 && (
        <>
          <ul aria-label="Legend" className="flex flex-wrap gap-x-5 gap-y-2 text-xs text-ink-2">
            {KIND_ORDER.map((k) => (
              <li key={k} className="flex items-center gap-2">
                <svg aria-hidden viewBox="-8 -8 16 16" className="size-4"><Pin kind={k} x={0} y={0} hot={false} /></svg>
                {KIND_WORD[k]}
              </li>
            ))}
            <li className="flex items-center gap-2"><svg aria-hidden viewBox="0 -4 24 8" className="h-2 w-6"><line x1="0" y1="4" x2="24" y2="-4" style={{ stroke: "var(--ink-3)" }} strokeDasharray="4 4" /></svg>Linked in the work</li>
          </ul>
          <div className="relative hidden overflow-hidden rounded-entry border border-ink bg-surface dark:border-hairline md:block">
            <svg role="group" aria-label="Map of people, suite items, finished work and files. Hover or focus a pin to see its links."
              viewBox={`0 0 ${W} ${lay.height}`} className="block h-auto w-full" onMouseLeave={() => setHot(null)}>
              <defs>
                <pattern id="ergon-contours" width="72" height="48" patternUnits="userSpaceOnUse">
                  <path d="M0 24 Q18 12 36 24 T72 24" fill="none" stroke="currentColor" strokeWidth="1" />
                </pattern>
              </defs>
              <rect x="0" y="0" width={W} height={lay.height} fill="url(#ergon-contours)" style={{ color: "var(--contour)" }} opacity="0.5" />
              {KIND_ORDER.map((k) => (
                <text key={k} x={BAND_X[k] - 8} y={44} fontSize="12" style={{ fill: "var(--ink-3)" }}>{BAND_TITLE[k]}</text>
              ))}
              {drawn.map((e, i) => {
                const a = lay.pos[e.from], b = lay.pos[e.to];
                const on = hot === e.from || hot === e.to;
                const x1 = a.x + 12, x2 = b.x - 12, mid = (x1 + x2) / 2;
                return (
                  <path key={i} d={`M${x1} ${a.y} C${mid} ${a.y}, ${mid} ${b.y}, ${x2} ${b.y}`} fill="none"
                    style={{ stroke: on ? "var(--ink)" : "var(--ink-3)" }} strokeWidth={on ? 2.2 : 1} strokeDasharray="5 5"
                    opacity={hot && !on ? 0.12 : on ? 0.95 : 0.35} />
                );
              })}
              {Object.values(lay.pos).map(({ x, y, n, kind }) => {
                const on = hot === n.id;
                const text = (n.label || "").length > 26 ? (n.label || "").slice(0, 25) + "…" : n.label || "";
                return (
                  <g key={n.id} role="button" tabIndex={0} aria-label={focusLabel(n.id)}
                    onMouseEnter={() => setHot(n.id)} onFocus={() => setHot(n.id)} onBlur={() => setHot(null)}
                    className="focus:outline-none">
                    {on && <circle cx={x} cy={y} r={14} fill="none" style={{ stroke: "var(--ink)" }} strokeWidth={2} />}
                    <Pin kind={kind} x={x} y={y} hot={on} />
                    <rect x={x + 12} y={y - 11} width={Math.min(230, 20 + text.length * 7.2)} height={22} rx={4} style={{ fill: "var(--surface)" }} opacity="0.92" />
                    <text x={x + 18} y={y + 4} fontSize="13" fontWeight={on ? 600 : 500} style={{ fill: "var(--ink)" }}>{text}</text>
                    <title>{focusLabel(n.id) + (n.detail ? ": " + n.detail : "")}</title>
                  </g>
                );
              })}
            </svg>
          </div>
        </>
      )}

      <p className="text-xs text-ink-3 md:hidden">The map is drawn on wider screens. The same work, as a list:</p>
      {nodes.length > 0 && (
        <ol aria-label="Work graph as a list" className="divide-y divide-hairline rounded-entry border border-hairline bg-surface">
          {sorted.map((n) => {
            const linked = edges.filter((e) => e.from === n.id || e.to === n.id).length;
            return (
              <li key={n.id} tabIndex={-1} onMouseEnter={() => setHot(n.id)} onMouseLeave={() => setHot(null)}
                className="flex items-baseline gap-3 px-4 py-2.5">
                <span className="w-28 shrink-0 text-xs text-ink-3">{KIND_WORD[n.kind] || n.kind}</span>
                <span className="min-w-0 flex-1 truncate text-sm text-ink">{n.label || ""}</span>
                <span className="shrink-0 font-mono text-xs tabular-nums text-ink-3">{linked} {linked === 1 ? "link" : "links"}</span>
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}
