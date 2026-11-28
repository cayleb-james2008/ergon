// Work Graph: four-band causal map (People -> Suite items -> Finished work -> Files) drawn at 1:1 CSS pixels
// with a clean dot-grid canvas, pill nodes, multi-hop lineage tracing, search/kind filters, and selected-node inspector.
import { useEffect, useMemo, useRef, useState } from "react";
import { RefreshCw, Search, X } from "lucide-react";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FileGlyph } from "@/chat/Entry";
import { call, type GraphEdge, type GraphNode, type GraphPayload } from "@/lib/api";
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

export function Pin({
  kind,
  x,
  y,
  hot,
  active,
}: {
  kind: string;
  x: number;
  y: number;
  hot: boolean;
  active?: boolean;
}) {
  const work = kind === "work";
  const accent =
    active
      ? "var(--blue)"
      : kind === "work"
        ? "var(--dot-green)"
        : kind === "item"
          ? "var(--dot-amber)"
          : kind === "person"
            ? "var(--blue)"
            : "var(--ink-3)";
  const style = {
    fill: active ? "var(--blue)" : work ? "var(--ink)" : "var(--surface)",
    stroke: accent,
  };
  const sw = hot || active ? 2.2 : 1.75;
  if (kind === "person") return <circle cx={x} cy={y} r={6.5} style={style} strokeWidth={sw} />;
  if (kind === "item") return <polygon points={`${x},${y - 7.5} ${x + 7.5},${y} ${x},${y + 7.5} ${x - 7.5},${y}`} style={style} strokeWidth={sw} />;
  if (kind === "work") return <polygon points={`${x},${y - 7.5} ${x + 7.5},${y + 6.5} ${x - 7.5},${y + 6.5}`} style={style} strokeWidth={sw} />;
  return <rect x={x - 6} y={y - 6} width={12} height={12} style={style} strokeWidth={sw} />;
}

export function layout(nodes: GraphNode[], W: number) {
  const safeW = Math.max(MIN_MAP, W);
  const gap = (safeW - 2 * PAD) / 4;
  const bandX = Object.fromEntries(KIND_ORDER.map((k, i) => [k, Math.round(PAD + 8 + i * gap)]));
  const room = Math.max(68, gap - 42);
  const bands: Record<string, GraphNode[]> = { person: [], item: [], work: [], file: [] };
  for (const n of nodes) {
    if (!n || typeof n !== "object" || typeof n.id !== "string") continue;
    (bands[bands[n.kind] ? n.kind : "work"]).push(n);
  }
  for (const k of KIND_ORDER) {
    bands[k].sort((a, b) => String(a.label ?? "").localeCompare(String(b.label ?? "")) || a.id.localeCompare(b.id));
  }
  const rows = Math.max(1, ...KIND_ORDER.map((k) => bands[k].length));
  const step = rows <= 8 ? 58 : rows <= 14 ? 44 : 36;
  const top = 66;
  const height = top + rows * step + 28;
  const fitText = (label: unknown) => {
    const str = String(label ?? "");
    const max = Math.max(6, Math.floor((room - 20) / 7.8));
    return str.length > max ? str.slice(0, max - 1) + "…" : str;
  };
  const pos: Record<string, { x: number; y: number; n: GraphNode; kind: string; text: string; backW: number }> = {};
  for (const k of KIND_ORDER) {
    bands[k].forEach((n, i) => {
      const text = fitText(n.label);
      const backW = Math.min(room, Math.max(44, 18 + text.length * 7.6));
      pos[n.id] = { x: bandX[k], y: top + i * step + step / 2, n, kind: k, text, backW };
    });
  }
  return { pos, height, step, bands, bandX, gap, room };
}

function traceLineage(focusId: string | null, edges: GraphEdge[]): { nodes: Set<string>; edgeIndices: Set<number> } {
  const activeNodes = new Set<string>();
  const activeEdges = new Set<number>();
  if (!focusId) return { nodes: activeNodes, edgeIndices: activeEdges };

  activeNodes.add(focusId);

  // Forward pass (person -> work -> file, person -> item)
  const fwd = new Set<string>([focusId]);
  let changed = true;
  while (changed) {
    changed = false;
    edges.forEach((e, idx) => {
      if (fwd.has(e.from)) {
        activeEdges.add(idx);
        activeNodes.add(e.to);
        if (!fwd.has(e.to)) {
          fwd.add(e.to);
          changed = true;
        }
      }
    });
  }

  // Backward pass (file -> work -> person, work -> person, item -> person)
  const bwd = new Set<string>([focusId]);
  changed = true;
  while (changed) {
    changed = false;
    edges.forEach((e, idx) => {
      if (bwd.has(e.to)) {
        activeEdges.add(idx);
        activeNodes.add(e.from);
        if (!bwd.has(e.from)) {
          bwd.add(e.from);
          changed = true;
        }
      }
    });
  }

  return { nodes: activeNodes, edgeIndices: activeEdges };
}

export function GraphFace({ onOpenFile }: { onOpenFile?: (path: string) => void }) {
  const [graph, setGraph] = useState<GraphPayload | null>(null);
  const [failed, setFailed] = useState(false);
  const [loading, setLoading] = useState(false);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [kindFilter, setKindFilter] = useState<string>("all");
  const [viewMode, setViewMode] = useState<"map" | "list">("map");
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(() =>
    typeof window !== "undefined" ? Math.max(0, Math.min(1112, window.innerWidth - (window.innerWidth >= 768 ? 256 + 48 : 32))) : 0
  );

  const load = () => {
    setLoading(true);
    call<GraphPayload>("/api/graph").then((r) => {
      setLoading(false);
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

  const nodes = useMemo(
    () => (Array.isArray(graph?.nodes) ? graph!.nodes.filter((n): n is GraphNode => !!n && typeof n === "object" && typeof n.id === "string") : []),
    [graph]
  );
  const edges = useMemo(() => (Array.isArray(graph?.edges) ? graph!.edges.filter((e) => !!e && typeof e === "object") : []), [graph]);
  const nodeMap = useMemo(() => new Map(nodes.map((n) => [n.id, n])), [nodes]);
  const W = Math.max(MIN_MAP, width - 2);
  const lay = useMemo(() => layout(nodes, W), [nodes, W]);
  const canShowMap = width >= MIN_MAP;
  const showMap = viewMode === "map" && canShowMap;

  const activeId = hoveredId ?? selectedId;
  const lineage = useMemo(() => traceLineage(activeId, edges), [activeId, edges]);

  const normalizedQuery = query.trim().toLowerCase();
  const matchesFilter = (n: GraphNode) => {
    if (kindFilter !== "all" && n.kind !== kindFilter) return false;
    if (!normalizedQuery) return true;
    return `${String(n.label ?? "")} ${String(n.detail ?? "")} ${KIND_WORD[n.kind] ?? n.kind}`
      .toLowerCase()
      .includes(normalizedQuery);
  };
  const hasActiveSearchOrKind = normalizedQuery.length > 0 || kindFilter !== "all";
  const matchedIds = useMemo(() => new Set(nodes.filter(matchesFilter).map((n) => n.id)), [nodes, normalizedQuery, kindFilter]);

  const selectedNode = selectedId ? nodeMap.get(selectedId) ?? null : null;
  const selectedConnections = useMemo(() => {
    if (!selectedNode) return { inbound: [] as GraphNode[], outbound: [] as GraphNode[] };
    const inbound: GraphNode[] = [];
    const outbound: GraphNode[] = [];
    const seenIn = new Set<string>();
    const seenOut = new Set<string>();
    for (const e of edges) {
      if (e.to === selectedNode.id && !seenIn.has(e.from)) {
        const src = nodeMap.get(e.from);
        if (src) {
          inbound.push(src);
          seenIn.add(e.from);
        }
      }
      if (e.from === selectedNode.id && !seenOut.has(e.to)) {
        const dst = nodeMap.get(e.to);
        if (dst) {
          outbound.push(dst);
          seenOut.add(e.to);
        }
      }
    }
    return { inbound, outbound };
  }, [selectedNode, edges, nodeMap]);

  const sorted = useMemo(
    () =>
      nodes
        .filter(matchesFilter)
        .sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind) || String(a.label ?? "").localeCompare(String(b.label ?? ""))),
    [nodes, normalizedQuery, kindFilter]
  );

  const fit = (label: unknown) => {
    const str = String(label ?? "");
    const max = Math.max(6, Math.floor((lay.room - 20) / 7.8));
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
  const sortedNodes = sorted;
  const drawn = edges.map((e, idx) => ({ e, idx })).filter(({ e }) => lay.pos[e.from] && lay.pos[e.to]);
  const focusLabel = (id: string) => {
    const p = lay.pos[id];
    return p ? `${KIND_WORD[p.kind] || p.kind}: ${String(p.n.label ?? "")}` : "";
  };

  return (
    <section
      id="face-graph"
      aria-label="Work graph"
      onKeyDown={(ev) => {
        if (ev.key === "Escape" && selectedId) setSelectedId(null);
      }}
      className="mx-auto flex w-full max-w-[1160px] flex-col gap-5 px-4 pb-12 pt-5 md:px-6"
    >
      <div className="flex flex-wrap items-end justify-between gap-4 border-b border-hairline pb-4">
        <div>
          <h1 className="text-[28px] leading-tight font-medium tracking-[-0.03em] text-ink md:text-[34px]">
            Who the work belongs to, and what it touched
          </h1>
          <p className="mt-2 flex flex-wrap gap-x-5 gap-y-1 text-sm text-ink-2">
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink">{count("person")}</span><span>people</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink">{count("work")}</span><span>finished jobs</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink">{count("file")}</span><span>files produced</span></span>
            <span className="inline-flex items-center gap-1.5"><span className="mono font-medium text-ink">{count("item")}</span><span>suite items in play</span></span>
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            size="sm"
            variant="outline"
            aria-label="Refresh work graph"
            disabled={loading}
            onClick={load}
            className="max-md:min-h-11 gap-1.5"
          >
            <RefreshCw className={cn("size-3.5 text-icon", loading && "motion-safe:animate-spin")} strokeWidth={1.5} aria-hidden />
            <span>Refresh</span>
          </Button>
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
      </div>

      {!nodes.length && (
        <p className="rounded-card border border-hairline bg-surface px-4 py-4 text-sm text-ink-2">
          No finished work yet. The graph fills in automatically as your agent completes tasks and writes workspace files.
        </p>
      )}

      {nodes.length > 0 && (
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="relative min-w-[220px] flex-1 sm:max-w-xs">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-icon" strokeWidth={1.5} aria-hidden />
            <Input
              id="graph-search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Filter people, jobs, tools, or files…"
              aria-label="Filter work graph nodes"
              className="h-9 bg-surface pl-8 pr-8 text-sm max-md:min-h-11"
            />
            {query && (
              <button
                type="button"
                aria-label="Clear work graph search"
                onClick={() => setQuery("")}
                className="focus-ring absolute right-1.5 top-1/2 grid size-7 -translate-y-1/2 place-items-center rounded-sm text-icon hover:bg-board hover:text-ink"
              >
                <X className="size-3.5" strokeWidth={1.5} aria-hidden />
              </button>
            )}
          </div>

          <div role="group" aria-label="Filter by node kind" className="flex flex-wrap items-center gap-1.5">
            {[
              { id: "all", label: "All", total: nodes.length },
              ...KIND_ORDER.map((k) => ({ id: k, label: BAND_TITLE[k], total: count(k) })),
            ].map((chip) => {
              const active = kindFilter === chip.id;
              return (
                <button
                  key={chip.id}
                  type="button"
                  aria-pressed={active}
                  onClick={() => setKindFilter(chip.id)}
                  className={cn(
                    "focus-ring inline-flex min-h-11 items-center gap-1.5 rounded-chip border px-2.5 py-1 text-xs font-medium outline-none md:min-h-8",
                    active
                      ? "border-ink bg-ink text-surface"
                      : "border-hairline bg-surface text-ink-2 hover:bg-board hover:text-ink"
                  )}
                >
                  <span>{chip.label}</span>
                  <span className={cn("mono text-xs", active ? "opacity-80" : "text-ink-3")}>{chip.total}</span>
                </button>
              );
            })}
          </div>
        </div>
      )}

      <div ref={box} className="w-full">
        {nodes.length > 0 && (
          <>
            {showMap && (
              <div className="flex flex-col gap-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <ul aria-label="Legend" className="flex flex-wrap gap-x-5 gap-y-2 text-xs text-ink-2">
                    {KIND_ORDER.map((k) => (
                      <li key={k} className="flex items-center gap-2">
                        <svg aria-hidden viewBox="-8 -8 16 16" className="size-4">
                          <Pin kind={k} x={0} y={0} hot={false} />
                        </svg>
                        {KIND_WORD[k]}
                      </li>
                    ))}
                    <li className="flex items-center gap-2">
                      <svg aria-hidden viewBox="0 -4 24 8" className="h-2 w-6">
                        <line x1="0" y1="0" x2="24" y2="0" style={{ stroke: "var(--blue)" }} strokeWidth="2" />
                      </svg>
                      Traced causal path (hover or click any node)
                    </li>
                  </ul>
                  {selectedId && (
                    <Button
                      type="button"
                      size="xs"
                      variant="ghost"
                      className="max-md:min-h-11 gap-1 text-ink-2 hover:text-ink"
                      onClick={() => setSelectedId(null)}
                    >
                      <X className="size-3" strokeWidth={1.5} aria-hidden />
                      <span>Clear pin</span>
                    </Button>
                  )}
                </div>

                <div className="relative overflow-hidden rounded-card border border-hairline bg-surface shadow-card">
                  <svg
                    id="graph-svg"
                    role="group"
                    aria-label="Map of people, suite items, finished work and files. Hover or click a pin to trace its causal path."
                    viewBox={`0 0 ${W} ${lay.height}`}
                    className="block h-auto w-full"
                    onMouseLeave={() => setHoveredId(null)}
                    onClick={(ev) => {
                      if ((ev.target as Element)?.tagName === "rect" && (ev.target as Element)?.getAttribute("data-bg") === "1") {
                        setSelectedId(null);
                      }
                    }}
                  >
                    <defs>
                      <pattern id="ergon-dot-grid" width="20" height="20" patternUnits="userSpaceOnUse">
                        <circle cx="10" cy="10" r="0.9" fill="currentColor" />
                      </pattern>
                    </defs>
                    <rect
                      data-bg="1"
                      x="0"
                      y="0"
                      width={W}
                      height={lay.height}
                      fill="url(#ergon-dot-grid)"
                      style={{ color: "var(--hairline)" }}
                      opacity="0.7"
                    />

                    {/* Subtle column header band & hairline dividers */}
                    <rect x="0" y="0" width={W} height="46" style={{ fill: "var(--board)" }} opacity="0.55" />
                    <line x1="0" y1="46" x2={W} y2="46" style={{ stroke: "var(--hairline)" }} strokeWidth="1" />
                    {KIND_ORDER.slice(1).map((k, idx) => {
                      const sepX = Math.round(lay.bandX[k] - 18);
                      return (
                        <line
                          key={`sep-${idx}`}
                          x1={sepX}
                          y1="0"
                          x2={sepX}
                          y2={lay.height}
                          style={{ stroke: "var(--hairline)" }}
                          strokeWidth="1"
                          opacity="0.55"
                        />
                      );
                    })}

                    {KIND_ORDER.map((k) => (
                      <text key={k} x={lay.bandX[k] - 6} y={28} fontSize="13" fontWeight={600} style={{ fill: "var(--ink-2)" }}>
                        {fit(`${BAND_TITLE[k]} (${lay.bands[k].length})`)}
                      </text>
                    ))}

                    {/* Non-active edges drawn first, highlighted causal lineage on top */}
                    {drawn.map(({ e, idx }) => {
                      const a = lay.pos[e.from];
                      const b = lay.pos[e.to];
                      const on = activeId ? lineage.edgeIndices.has(idx) : false;
                      const x1 = Math.min(a.x + 11 + a.backW + 2, b.x - 18);
                      const x2 = b.x - 10;
                      const mid = (x1 + x2) / 2;
                      const edgeColor = on ? "var(--blue)" : "var(--ink-3)";
                      const opacity = activeId ? (on ? 0.92 : 0.1) : 0.35;
                      return (
                        <g key={idx}>
                          <path
                            d={`M${x1} ${a.y} C${mid} ${a.y}, ${mid} ${b.y}, ${x2} ${b.y}`}
                            fill="none"
                            style={{ stroke: edgeColor }}
                            strokeWidth={on ? 2 : 1.15}
                            strokeDasharray={on ? undefined : "4 4"}
                            opacity={opacity}
                          />
                          {on && <circle cx={x2} cy={b.y} r={2.5} style={{ fill: "var(--blue)" }} opacity={0.95} />}
                        </g>
                      );
                    })}

                    {Object.values(lay.pos).map(({ x, y, n, kind, text, backW }) => {
                      const isPinned = selectedId === n.id;
                      const isDirectHot = activeId === n.id;
                      const inLineage = activeId ? lineage.nodes.has(n.id) : true;
                      const inFilter = matchedIds.has(n.id);
                      const dimmed =
                        (activeId !== null && !inLineage) ||
                        (hasActiveSearchOrKind && !inFilter && !isDirectHot);
                      const highlighted = isDirectHot || (activeId !== null && inLineage) || (hasActiveSearchOrKind && inFilter);
                      return (
                        <g
                          key={n.id}
                          data-pin={n.id}
                          tabIndex={0}
                          aria-label={focusLabel(n.id)}
                          aria-pressed={isPinned}
                          onMouseEnter={() => setHoveredId(n.id)}
                          onFocus={() => setHoveredId(n.id)}
                          onBlur={() => setHoveredId(null)}
                          onClick={(ev) => {
                            ev.stopPropagation();
                            setSelectedId((prev) => (prev === n.id ? null : n.id));
                          }}
                          onKeyDown={(ev) => {
                            if (ev.key === "Enter" || ev.key === " ") {
                              ev.preventDefault();
                              setSelectedId((prev) => (prev === n.id ? null : n.id));
                            } else if (ev.key === "Escape") {
                              setSelectedId(null);
                            }
                          }}
                          style={{ opacity: dimmed ? 0.24 : 1 }}
                          className="cursor-pointer focus:outline-none"
                        >
                          <Pin kind={kind} x={x} y={y} hot={isDirectHot || inLineage && activeId !== null} active={isPinned} />
                          <rect
                            x={x + 11}
                            y={y - 13}
                            width={backW}
                            height={26}
                            rx={6}
                            opacity="1"
                            style={{
                              fill: isPinned || isDirectHot ? "var(--blue-soft)" : "var(--surface)",
                              stroke: isPinned || isDirectHot ? "var(--blue)" : highlighted && (activeId || hasActiveSearchOrKind) ? "var(--ink-3)" : "var(--hairline)",
                            }}
                            strokeWidth={isPinned || isDirectHot ? 1.4 : 1}
                          />
                          <text
                            x={x + 18}
                            y={y + 4.5}
                            fontSize="13"
                            fontWeight={isPinned || isDirectHot ? 600 : 400}
                            style={{ fill: "var(--ink)" }}
                          >
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

      {selectedNode && (
        <aside
          id="graph-inspector"
          aria-label="Selected node details"
          className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4 shadow-card"
        >
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="flex min-w-0 flex-1 items-center gap-2.5">
              <Badge variant="outline" size="sm" className="shrink-0">
                {KIND_WORD[selectedNode.kind] || selectedNode.kind}
              </Badge>
              {selectedNode.kind === "file" && <FileGlyph name={String(selectedNode.label ?? "")} />}
              <h2 className="min-w-0 truncate text-sm font-semibold text-ink">
                {String(selectedNode.label ?? "")}
              </h2>
              {selectedNode.detail && (
                <span className="mono shrink-0 text-xs text-ink-3">{String(selectedNode.detail)}</span>
              )}
            </div>
            <div className="flex shrink-0 items-center gap-1.5">
              {selectedNode.kind === "file" && onOpenFile && selectedNode.label && (
                <Button
                  type="button"
                  size="xs"
                  className="max-md:min-h-11"
                  onClick={() => onOpenFile(String(selectedNode.label))}
                >
                  Open file
                </Button>
              )}
              <Button
                type="button"
                size="icon-xs"
                variant="ghost"
                aria-label="Close node inspector"
                className="max-md:min-h-11 max-md:min-w-11"
                onClick={() => setSelectedId(null)}
              >
                <X className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
              </Button>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {selectedConnections.inbound.length > 0 && (
              <div className="flex flex-col gap-1.5">
                <span className="text-xs font-medium text-ink-2">Upstream sources ({selectedConnections.inbound.length})</span>
                <div className="flex flex-wrap gap-1.5">
                  {selectedConnections.inbound.map((src) => (
                    <button
                      key={src.id}
                      type="button"
                      onClick={() => setSelectedId(src.id)}
                      className="focus-ring inline-flex min-h-11 items-center gap-1.5 rounded-chip border border-hairline bg-board/60 px-2.5 py-1 text-xs text-ink hover:bg-board md:min-h-7"
                    >
                      <span className="text-ink-3">{KIND_WORD[src.kind] || src.kind}:</span>
                      <span className="max-w-[200px] truncate font-medium">{String(src.label ?? "")}</span>
                    </button>
                  ))}
                </div>
              </div>
            )}
            {selectedConnections.outbound.length > 0 && (
              <div className="flex flex-col gap-1.5">
                <span className="text-xs font-medium text-ink-2">Connected outputs ({selectedConnections.outbound.length})</span>
                <div className="flex flex-wrap gap-1.5">
                  {selectedConnections.outbound.map((dst) => (
                    <button
                      key={dst.id}
                      type="button"
                      onClick={() => setSelectedId(dst.id)}
                      className="focus-ring inline-flex min-h-11 items-center gap-1.5 rounded-chip border border-hairline bg-board/60 px-2.5 py-1 text-xs text-ink hover:bg-board md:min-h-7"
                    >
                      <span className="text-ink-3">{KIND_WORD[dst.kind] || dst.kind}:</span>
                      <span className="max-w-[200px] truncate font-medium">{String(dst.label ?? "")}</span>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </aside>
      )}

      {nodes.length > 0 && (
        <ol id="graph-list" aria-label="Work graph as a list" className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
          {sorted.length === 0 ? (
            <li className="px-4 py-4 text-sm text-ink-2">
              No nodes match &ldquo;{query.trim()}&rdquo;. Clear the filter to see all {nodes.length} nodes.
            </li>
          ) : (
            sorted.map((n) => {
              const linked = edges.filter((e) => e.from === n.id || e.to === n.id).length;
              const label = String(n.label ?? "");
              const isSelected = selectedId === n.id;
              const inLineage = activeId ? lineage.nodes.has(n.id) : false;
              return (
                <li
                  key={n.id}
                  onMouseEnter={() => setHoveredId(n.id)}
                  onMouseLeave={() => setHoveredId(null)}
                  className={cn(
                    "flex flex-wrap items-center justify-between gap-3 px-4 py-3 motion-safe:transition-[background-color]",
                    isSelected ? "bg-blue-soft/60" : inLineage ? "bg-board/40" : "hover:bg-board/30"
                  )}
                >
                  <button
                    type="button"
                    aria-pressed={isSelected}
                    onClick={() => setSelectedId((prev) => (prev === n.id ? null : n.id))}
                    className="focus-ring flex min-h-11 min-w-0 flex-1 items-center gap-2.5 rounded-sm text-left outline-none md:min-h-8"
                  >
                    <Badge variant={isSelected ? "secondary" : "outline"} size="sm" className="w-26 shrink-0 justify-center">
                      {KIND_WORD[n.kind] || n.kind}
                    </Badge>
                    {n.kind === "file" && <FileGlyph name={label} />}
                    <span className={cn("min-w-0 truncate text-sm text-ink", isSelected && "font-medium", n.kind === "file" && "mono")}>
                      {label}
                    </span>
                  </button>
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
                      </Button>
                    )}
                    <span className="mono text-xs text-ink-2">
                      {linked} {linked === 1 ? "link" : "links"}
                    </span>
                  </div>
                </li>
              );
            })
          )}
        </ol>
      )}
    </section>
  );
}
