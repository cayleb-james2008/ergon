// Work Graph: the work, as a map. A survey-sheet canvas with pins for people, suite items, finished work and files,
// dashed routes between them. Only what /api/graph returns is drawn; the list is the same data in text.
import { useEffect, useState } from "react";
import { call } from "@/lib/api";

type Node = { id: string; kind: string; label?: string; detail?: string };
type Graph = { nodes?: Node[]; edges?: { from: string; to: string }[] };

const KIND_WORD: Record<string, string> = { person: "Person", item: "Suite item", work: "Finished work", file: "File" };
const KIND_ORDER = ["person", "item", "work", "file"];
const COL: Record<string, number> = { person: 110, item: 370, work: 620, file: 840 };

function Pin({ kind, x, y }: { kind: string; x: number; y: number }) {
  const common = { fill: kind === "work" ? "var(--blue)" : "var(--surface)", stroke: kind === "work" ? "var(--blue)" : "var(--ink)", strokeWidth: 1.5 };
  if (kind === "person") return <circle cx={x} cy={y} r={7} {...common} />;
  if (kind === "item") return <polygon points={`${x},${y - 8} ${x + 8},${y} ${x},${y + 8} ${x - 8},${y}`} {...common} />;
  if (kind === "work") return <polygon points={`${x},${y - 8} ${x + 8},${y + 7} ${x - 8},${y + 7}`} {...common} />;
  return <rect x={x - 6} y={y - 6} width={12} height={12} {...common} />;
}

export function GraphFace() {
  const [graph, setGraph] = useState<Graph | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    call<Graph>("/api/graph").then((r) => { if (r.ok) setGraph(r.data); else setFailed(true); });
  }, []);

  if (failed) return <section aria-label="Work graph" className="mx-auto w-full max-w-[1000px] px-6 pt-6"><p role="alert" className="text-sm text-ink-2">The graph could not be loaded. Is the Ergon server running?</p></section>;
  if (!graph) return <section aria-label="Work graph" className="mx-auto w-full max-w-[1000px] px-6 pt-6"><p className="text-sm text-ink-2">Loading…</p></section>;

  const nodes = graph.nodes ?? [];
  const edges = graph.edges ?? [];
  const count = (k: string) => nodes.filter((n) => n.kind === k).length;
  const perCol: Record<string, number> = { person: 0, item: 0, work: 0, file: 0 };
  const pos: Record<string, { x: number; y: number; n: Node }> = {};
  for (const n of nodes) {
    const k = COL[n.kind] != null ? n.kind : "work";
    const row = perCol[k]++;
    pos[n.id] = { x: COL[k], y: 70 + row * 84, n };
  }
  const rows = Math.max(1, ...Object.values(perCol));
  const height = Math.max(240, 140 + rows * 84);
  const sorted = nodes.slice().sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind));

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

      {!count("work") && <p className="text-sm text-ink-2">No finished work yet. The graph fills in as your agent finishes jobs.</p>}

      <div className="relative hidden overflow-hidden rounded-entry border border-ink bg-surface md:block">
        <svg role="img" aria-label="Graph of people, their finished work, the files it produced and the suite items they carry" viewBox={`0 0 1000 ${height}`} className="block h-auto w-full" style={{ color: "var(--contour)" }}>
          <defs>
            <pattern id="ergon-contours" width="72" height="48" patternUnits="userSpaceOnUse">
              <path d="M0 24 Q18 12 36 24 T72 24" fill="none" stroke="currentColor" strokeWidth="1" />
            </pattern>
          </defs>
          <rect x="0" y="0" width="1000" height={height} fill="url(#ergon-contours)" opacity="0.7" />
          {edges.map((e, i) => {
            const a = pos[e.from], b = pos[e.to];
            if (!a || !b) return null;
            return <line key={i} x1={a.x} y1={a.y} x2={b.x} y2={b.y} style={{ stroke: "var(--ink-3)" }} strokeWidth={1.25} strokeDasharray="6 6" />;
          })}
          {Object.values(pos).map(({ x, y, n }) => (
            <g key={n.id}>
              <Pin kind={n.kind} x={x} y={y} />
              <rect x={x + 12} y={y - 17} width={Math.min(200, 26 + (n.label || "").length * 7.4)} height={36} rx={4} style={{ fill: "var(--surface)" }} opacity="0.9" />
              <text x={x + 18} y={y - 2} style={{ fill: "var(--ink)" }} fontSize="14" fontWeight="500">{(n.label || "").length > 24 ? (n.label || "").slice(0, 23) + "…" : n.label}</text>
              <text x={x + 18} y={y + 16} style={{ fill: "var(--ink-3)" }} fontSize="12">{KIND_WORD[n.kind] || n.kind}</text>
              <title>{(n.label || "") + (n.detail ? ": " + n.detail : "")}</title>
            </g>
          ))}
        </svg>
      </div>
      <p className="text-xs text-ink-3 md:hidden">The map is drawn on wider screens. The same work, as a list:</p>

      <ol aria-label="Work graph as a list" className="divide-y divide-hairline rounded-entry border border-hairline bg-surface">
        {sorted.map((n) => (
          <li key={n.id} className="flex items-baseline gap-3 px-4 py-2.5">
            <span className="w-28 shrink-0 text-xs text-ink-3">{KIND_WORD[n.kind] || n.kind}</span>
            <span className="min-w-0 truncate text-sm text-ink">{n.label || ""}</span>
          </li>
        ))}
      </ol>
    </section>
  );
}
