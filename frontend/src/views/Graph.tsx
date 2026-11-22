// Work Graph: four-band survey map (People -> Suite items -> Finished work -> Files) drawn at 1:1 CSS pixels
// so every SVG <text> label renders >= 13px at 768px..1440px without overlap or clipping, paired with a
// SegmentedControl view toggle and an accessible structured list.
import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUpRight } from "lucide-react";
import { SlidingNumber } from "@/components/ui/sliding-number";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { FileGlyph } from "@/chat/Entry";
import { call, type GraphNode, type GraphPayload } from "@/lib/api";
import { cn } from "@/lib/utils";

export const KIND_WORD: Record<string, string> = {
  person: "Person",
  item: "Suite item",
  work: "Finished work",
  file: "File",
};
const KIND_ORDER = ["person", "item", "work", "file"];
const BAND_TITLE: Record<string, string> = {
  person: "People",
  item: "Suite items",
  work: "Finished work",
  file: "Files",
};
/** Narrowest container width where 4 columns keep >=13px non-overlapping labels (768px viewport - 256px rail - 32px pad = 480px). */
const MIN_MAP = 440;
const PAD = 14;

export function Pin({ kind, x, y, hot }: { kind: string; x: number; y: number; hot: boolean }) {
  const work = kind === "work";
  const style = { fill: work ? "var(--ink)" : "var(--surface)", stroke: kind === "file" ? "var(--ink-3)" : "var(--ink)" };
  const sw = hot ? 2.4 : 1.5;
  if (kind === "person") return <circle cx={x} cy={y} r={6.5} style={style} strokeWidth={sw} />;
  if (kind === "item") return <polygon points={`${x},${y - 7.5} ${x + 7.5},${y} ${x},${y + 7.5} ${x - 7.5},${y}`} style={style} strokeWidth={sw} />;
  if (kind === "work") return <polygon points={`${x},${y - 7.5} ${x + 7.5},${y + 6.5} ${x - 7.5},${y + 6.5}`} style={style} strokeWidth={sw} />;
  return <rect x={x - 6} y={y - 6} width={12} height={12} style={style} strokeWidth={sw} />;
}

export function layout(nodes: GraphNode[], W: number) {
  const safeW = Math.max(MIN_MAP, W);
  const gap = (safeW - 2 * PAD) / 4;
  const bandX = Object.fromEntries(KIND_ORDER.map((k, i) => [k, Math.round(PAD + 8 + i * gap)]));
  const room = Math.max(68, gap - 28);
  const bands: Record<string, GraphNode[]> = { person: [], item: [], work: [], file: [] };
  for (const n of nodes) {
    if (!n || typeof n !== "object" || typeof n.id !== "string") continue;
    (bands[bands[n.kind] ? n.kind : "work"]).push(n);
  }
  for (const k of KIND_ORDER) {
    bands[k].sort((a, b) => String(a.label ?? "").localeCompare(String(b.label ?? "")) || a.id.localeCompare(b.id));
  }
  const rows = Math.max(1, ...KIND_ORDER.map((k) => bands[k].length));
  const step = rows <= 8 ? 72 : rows <= 14 ? 52 : 38;
  const top = 76;
  const height = top + rows * step + 32;
  const pos: Record<string, { x: number; y: number; n: GraphNode; kind: string }> = {};
  for (const k of KIND_ORDER) {
    bands[k].forEach((n, i) => {
      pos[n.id] = { x: bandX[k], y: top + i * step + step / 2, n, kind: k };
    });
  }
  return { pos, height, step, bands, bandX, room };
}

export function GraphFace({ onOpenFile }: { onOpenFile?: (path: string) => void }) {
  const [graph, setGraph] = useState<GraphPayload | null>(null);
  const [failed, setFailed] = useState(false);
  const [hot, setHot] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<"map" | "list">("map");
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(() => (typeof window !== "undefined" ? Math.max(0, Math.min(1112, window.innerWidth - (window.innerWidth >= 768 ? 256 + 48 : 32))) : 0));

  const load = () => {
    call<GraphPayload>("/api/graph").then((r) => {
      if (r.ok && Array.isArray(r.data.nodes) && Array.isArray(r.data.edges)) {
        setFailed(false);
        setGraph(r.data);
      } else {
        setFailed(true);
      }
    });
  };

  useEffect(() => { load(); }, []);

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const measure = () => setWidth(Math.floor(el.getBoundingClientRect().width));
    measure();
    const ro = new ResizeObserver(([e]) => setWidth(Math.floor(e.contentRect.width)));
    ro.observe(el);
    return () => ro.disconnect();
  }, [graph, viewMode]);

  const nodes = useMemo(() => (Array.isArray(graph?.nodes) ? graph!.nodes.filter((n): n is GraphNode => !!n && typeof n === "object" && typeof n.id === "string") : []), [graph]);
  const edges = useMemo(() => (Array.isArray(graph?.edges) ? graph!.edges.filter((e) => !!e && typeof e === "object") : []), [graph]);
  const W = Math.max(MIN_MAP, width - 2);
  const lay = useMemo(() => layout(nodes, W), [nodes, W]);
  const canShowMap = width >= MIN_MAP;
  const showMap = viewMode === "map" && canShowMap;

  const fit = (label: unknown) => {
    const str = String(label ?? "");
    const max = Math.max(6, Math.floor((lay.room - 14) / 7.8));
    return str.length > max ? str.slice(0, max - 1) + "…" : str;
  };

  if (failed) {
    return (
      <section aria-label="Work graph" className="mx-auto w-full max-w-[1160px] px-4 pt-6 md:px-6">
        <div role="alert" className="rounded-entry border border-hairline bg-surface p-5">
          <p className="text-base font-medium text-ink">The graph could not be loaded.</p>
          <p className="mt-1 text-sm text-ink-2">Check that the Ergon server is running, then try again.</p>
          <Button variant="outline" className="mt-3 max-md:min-h-11" onClick={load}>Retry</Button>
        </div>
      </section>
    );
  }

  if (!graph) {
    return (
      <section aria-label="Work graph" className="mx-auto w-full max-w-[1160px] px-4 pt-6 md:px-6">
        <div aria-busy="true" className="h-64 rounded-entry bg-board">
          <span className="sr-only">Loading work graph…</span>
        </div>
      </section>
    );
  }

  const count = (k: string) => nodes.filter((n) => n.kind === k).length;
  const sorted = [...nodes].sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind));
  const drawn = edges.filter((e) => lay.pos[e.from] && lay.pos[e.to]);
  const focusLabel = (id: string) => {
    const p = lay.pos[id];
    return p ? `${KIND_WORD[p.kind] || p.kind}: ${String(p.n.label ?? "")}` : "";
  };

  return (
    <section id="face-graph" aria-label="Work graph" className="mx-auto flex w-full max-w-[1160px] flex-col gap-5 px-4 pb-12 pt-5 md:px-6">
      <div className="flex flex-wrap items-end justify-between gap-4 border-b border-hairline pb-4">
        <div>
          <h1 className="text-[28px] leading-tight font-medium tracking-[-0.03em] text-ink md:text-[34px]">
            Who the work belongs to, and what it touched
          </h1>
          <p className="mt-2 flex flex-wrap gap-x-5 gap-y-1 text-sm text-ink-2">
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink"><SlidingNumber value={count("person")} /></span><span>people</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink"><SlidingNumber value={count("work")} /></span><span>finished jobs</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink"><SlidingNumber value={count("file")} /></span><span>files produced</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink"><SlidingNumber value={count("item")} /></span><span>suite items in play</span></span>
          </p>
        </div>
        {nodes.length > 0 && canShowMap && (
          <SegmentedControl aria-label="Work graph view">
            <SegmentedControlItem
              data-id="map"
              aria-pressed={viewMode === "map"}
              data-checked={viewMode === "map" ? "" : undefined}
              className="max-md:min-h-11"
              onClick={() => setViewMode("map")}
            >
              Map &amp; list
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="list"
              aria-pressed={viewMode === "list"}
              data-checked={viewMode === "list" ? "" : undefined}
              className="max-md:min-h-11"
              onClick={() => setViewMode("list")}
            >
              List only
            </SegmentedControlItem>
          </SegmentedControl>
        )}
      </div>

      {!nodes.length && (
        <p className="rounded-card border border-hairline bg-surface px-4 py-4 text-sm text-ink-2">
          No finished work yet. The graph fills in automatically as your agent completes tasks and writes workspace files.
        </p>
      )}

      <div ref={box} className="w-full">
        {nodes.length > 0 && (
          <>
            {showMap && (
              <div className="flex flex-col gap-3">
                <ul aria-label="Legend" className="flex flex-wrap gap-x-5 gap-y-2 text-xs text-ink-2">
                  {KIND_ORDER.map((k) => (
                    <li key={k} className="flex items-center gap-2">
                      <svg aria-hidden viewBox="-8 -8 16 16" className="size-4"><Pin kind={k} x={0} y={0} hot={false} /></svg>
                      {KIND_WORD[k]}
                    </li>
                  ))}
                  <li className="flex items-center gap-2">
                    <svg aria-hidden viewBox="0 -4 24 8" className="h-2 w-6">
                      <line x1="0" y1="4" x2="24" y2="-4" style={{ stroke: "var(--ink-3)" }} strokeDasharray="4 4" />
                    </svg>
                    Linked in the work
                  </li>
                </ul>

                <div className="relative overflow-hidden rounded-entry border border-ink bg-surface dark:border-hairline">
                  <svg
                    id="graph-svg"
                    role="group"
                    aria-label="Map of people, suite items, finished work and files. Hover or focus a pin to see its links."
                    viewBox={`0 0 ${W} ${lay.height}`}
                    className="block h-auto w-full"
                    onMouseLeave={() => setHot(null)}
                  >
                    <defs>
                      <pattern id="ergon-contours" width="72" height="48" patternUnits="userSpaceOnUse">
                        <path d="M0 24 Q18 12 36 24 T72 24" fill="none" stroke="currentColor" strokeWidth="1" />
                      </pattern>
                    </defs>
                    <rect x="0" y="0" width={W} height={lay.height} fill="url(#ergon-contours)" style={{ color: "var(--contour)" }} opacity="0.45" />
                    {KIND_ORDER.map((k) => (
                      <text key={k} x={lay.bandX[k] - 6} y={38} fontSize="13" fontWeight={500} style={{ fill: "var(--ink-2)" }}>
                        {fit(BAND_TITLE[k])}
                      </text>
                    ))}
                    {drawn.map((e, i) => {
                      const a = lay.pos[e.from];
                      const b = lay.pos[e.to];
                      const on = hot === e.from || hot === e.to;
                      const x1 = a.x + 10;
                      const x2 = b.x - 10;
                      const mid = (x1 + x2) / 2;
                      return (
                        <path
                          key={i}
                          d={`M${x1} ${a.y} C${mid} ${a.y}, ${mid} ${b.y}, ${x2} ${b.y}`}
                          fill="none"
                          style={{ stroke: on ? "var(--ink)" : "var(--ink-3)" }}
                          strokeWidth={on ? 2.2 : 1}
                          strokeDasharray="5 5"
                          opacity={hot && !on ? 0.14 : on ? 0.95 : 0.38}
                        />
                      );
                    })}
                    {Object.values(lay.pos).map(({ x, y, n, kind }) => {
                      const on = hot === n.id;
                      const text = fit(n.label);
                      const backW = Math.min(lay.room, 14 + text.length * 7.8);
                      return (
                        <g
                          key={n.id}
                          data-pin={n.id}
                          tabIndex={0}
                          aria-label={focusLabel(n.id)}
                          onMouseEnter={() => setHot(n.id)}
                          onFocus={() => setHot(n.id)}
                          onBlur={() => setHot(null)}
                          className="cursor-pointer focus:outline-none"
                        >
                          {on && <circle cx={x} cy={y} r={13} fill="none" style={{ stroke: "var(--ink)" }} strokeWidth={2} />}
                          <Pin kind={kind} x={x} y={y} hot={on} />
                          <rect x={x + 11} y={y - 11} width={backW} height={22} rx={4} opacity="1" style={{ fill: "var(--surface)" }} />
                          <text x={x + 16} y={y + 4.5} fontSize="13" fontWeight={on ? 500 : 400} style={{ fill: "var(--ink)" }}>
                            {text}
                          </text>
                          <title>{focusLabel(n.id) + (n.detail ? ": " + String(n.detail) : "")}</title>
                        </g>
                      );
                    })}
                  </svg>
                </div>
              </div>
            )}

            {width > 0 && !canShowMap && (
              <p className="mb-3 text-xs text-ink-3">The map is drawn on wider screens. The same work, as a structured list:</p>
            )}
          </>
        )}
      </div>

      {nodes.length > 0 && (
        <ol id="graph-list" aria-label="Work graph as a list" className="divide-y divide-hairline rounded-entry border border-hairline bg-surface">
          {sorted.map((n) => {
            const linked = edges.filter((e) => e.from === n.id || e.to === n.id).length;
            const label = String(n.label ?? "");
            return (
              <li
                key={n.id}
                tabIndex={-1}
                onMouseEnter={() => setHot(n.id)}
                onMouseLeave={() => setHot(null)}
                className="flex flex-wrap items-center justify-between gap-3 px-4 py-3"
              >
                <div className="flex min-w-0 flex-1 items-center gap-2.5">
                  <Badge variant="outline" size="sm" className="w-26 shrink-0 justify-center">
                    {KIND_WORD[n.kind] || n.kind}
                  </Badge>
                  {n.kind === "file" && <FileGlyph name={label} />}
                  <span className={cn("min-w-0 truncate text-sm text-ink", n.kind === "file" && "mono")}>
                    {label}
                  </span>
                </div>
                <div className="flex shrink-0 items-center gap-3">
                  {n.detail && <span className="mono text-xs text-ink-3">{String(n.detail)}</span>}
                  {n.kind === "file" && onOpenFile && label && (
                    <Button
                      size="xs"
                      variant="ghost"
                      className="max-md:min-h-11"
                      aria-label={`Open ${label}`}
                      onClick={() => onOpenFile(label)}
                    >
                      Open
                      <ArrowUpRight className="size-3.5" strokeWidth={1.5} aria-hidden />
                    </Button>
                  )}
                  <span className="mono text-xs text-ink-2">
                    {linked} {linked === 1 ? "link" : "links"}
                  </span>
                </div>
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}
