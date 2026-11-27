// Real Workspace Files panel & upgraded Workspace File Viewer modal (v0.7.0):
// - Lists active person's workspace files (GET /api/files) with instant search filter,
//   extension filter chips, multi-file drag-and-drop upload, and paginated rendering for 500+ files.
// - Upgraded FileViewerModal (GET /api/files/content?path=...):
//   * Preview tab: CSV column sort/filter/paging, Markdown section jump TOC, JSON collapsible tree
//   * History & Diff tab (GET /api/files/history?path=..., POST /api/files/restore): revision timestamps,
//     unified diff highlights (+added/-removed), revision-to-revision comparison, and Restore version
//   * One-click SHA-256 Receipt Export download (.md / .json)
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Briefcase,
  ChevronDown,
  ChevronRight,
  Copy,
  Download,
  FileDown,
  FilePlus2,
  FileSpreadsheet,
  FileText,
  History,
  Layers,
  MessageSquarePlus,
  Pencil,
  Play,
  RefreshCw,
  RotateCcw,
  Save,
  Search,
  Sparkles,
  Upload,
  X,
} from "lucide-react";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { usePanelModalClose } from "@/shell/PanelModal";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { FileGlyph } from "@/chat/Entry";
import {
  aggregateVault,
  call,
  canExportPdf,
  downloadWorkspaceFile,
  engagementWorkspacePrefix,
  exportFile,
  exportTarget,
  fileUploadBody,
  hasV070Api,
  isPathInEngagementScope,
  reindexVault,
  searchVault,
  type Client,
  type Engagement,
  type FileDiffPayload,
  type FileRevision,
  type Status,
  type TableAggregateFn,
  type TableWhereOp,
  type VaultAggregateResult,
  type VaultChunkHit,
  type VaultFileHit,
  type VaultIndexStats,
  type VaultSearchMode,
  type WorkspaceFile,
  type WorkspaceFileContent,
} from "@/lib/api";
import { FileEditor, MarkdownView, clampLineBounds, inspectDelimited, parseCsv, parseSortableNumber } from "@/shell/FileEditor";
export { parseCsv };
import { computeLineDiff, normalizeServerDiff } from "@/lib/diff";
import { exportClientPackDownload, exportReceiptDownload } from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { detectWorkflowForFile, parseCitationTarget, relTime } from "@/lib/words";

const FILES_PAGE_SIZE = 25;
const CSV_PAGE_SIZE = 100;

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} kB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function CsvTableView({ content }: { content: string }) {
  const inspection = useMemo(() => inspectDelimited(content, ","), [content]);
  const rows = inspection.rows;
  const [query, setQuery] = useState("");
  const [colFilter, setColFilter] = useState<string>("all");
  const [sortCol, setSortCol] = useState<number | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [page, setPage] = useState(0);

  useEffect(() => {
    setPage(0);
  }, [query, colFilter, sortCol, sortDir, content]);

  if (!rows.length) {
    return <p className="text-sm text-ink-2">This CSV file is empty.</p>;
  }
  const header = rows[0];
  const body = rows.slice(1);
  const isNumeric = (v: string) => /^-?\$?\d[\d,]*(?:\.\d+)?%?$/.test(v.trim());

  const toggleSort = (idx: number) => {
    if (sortCol !== idx) {
      setSortCol(idx);
      setSortDir("asc");
    } else if (sortDir === "asc") {
      setSortDir("desc");
    } else {
      setSortCol(null);
    }
  };

  const processed = useMemo(() => {
    const q = query.trim().toLowerCase();
    const colIdx = colFilter === "all" ? -1 : Number(colFilter);
    let out = q
      ? body.filter((r) => {
          if (colIdx >= 0) return (r[colIdx] ?? "").toLowerCase().includes(q);
          return r.some((c) => c.toLowerCase().includes(q));
        })
      : body;

    if (sortCol !== null) {
      const col = sortCol;
      const mult = sortDir === "asc" ? 1 : -1;
      out = [...out].sort((a, b) => {
        const av = (a[col] ?? "").trim();
        const bv = (b[col] ?? "").trim();
        const an = parseSortableNumber(av);
        const bn = parseSortableNumber(bv);
        if (an !== null && bn !== null) return (an - bn) * mult;
        return av.localeCompare(bv, undefined, { numeric: true, sensitivity: "base" }) * mult;
      });
    }
    return out;
  }, [body, query, colFilter, sortCol, sortDir]);

  const totalPages = Math.max(1, Math.ceil(processed.length / CSV_PAGE_SIZE));
  const safePage = Math.min(page, totalPages - 1);
  const visible = processed.slice(safePage * CSV_PAGE_SIZE, (safePage + 1) * CSV_PAGE_SIZE);

  return (
    <div className="flex flex-col gap-2.5">
      {inspection.error && (
        <p role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg">
          {inspection.error}
        </p>
      )}
      <div className="flex flex-wrap items-center gap-2">
        <InputGroup className="min-w-[200px] flex-1 rounded-sm bg-board/50 [&_input]:max-md:min-h-11">
          <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
            <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
          </InputGroupAddon>
          <InputGroupInput
            id="csv-filter-input"
            type="search"
            aria-label="Filter CSV rows"
            placeholder="Filter rows…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="text-sm max-md:min-h-11"
          />
          {query && (
            <InputGroupAddon align="inline-end" className="pr-1.5">
              <Button
                type="button"
                size="icon-xs"
                variant="ghost"
                className="max-md:min-h-11 max-md:min-w-11"
                aria-label="Clear CSV filter"
                onClick={() => setQuery("")}
              >
                <X className="size-3.5" strokeWidth={1.5} aria-hidden />
              </Button>
            </InputGroupAddon>
          )}
        </InputGroup>

        <select
          aria-label="Filter CSV column"
          value={colFilter}
          onChange={(e) => setColFilter(e.target.value)}
          className="focus-ring h-9 min-h-11 rounded-sm border border-input bg-surface px-2.5 text-xs text-ink outline-none md:min-h-9"
        >
          <option value="all">All columns</option>
          {header.map((col, idx) => (
            <option key={idx} value={String(idx)}>
              {col || `Column ${idx + 1}`}
            </option>
          ))}
        </select>

        <span className="mono text-xs text-ink-2">
          {processed.length === body.length
            ? `${body.length} ${body.length === 1 ? "row" : "rows"}, ${header.length} cols`
            : `${processed.length} of ${body.length} rows`}
        </span>
      </div>

      <div className="max-h-[46vh] overflow-auto rounded-sm border border-hairline bg-surface">
        <table className="w-full border-collapse text-left text-xs">
          <thead className="sticky top-0 border-b border-hairline bg-board text-ink">
            <tr>
              {header.map((col, idx) => {
                const active = sortCol === idx;
                const label = col || `Column ${idx + 1}`;
                return (
                  <th
                    key={idx}
                    aria-sort={active ? (sortDir === "asc" ? "ascending" : "descending") : "none"}
                    className="whitespace-nowrap p-0 font-medium"
                  >
                    <button
                      type="button"
                      onClick={() => toggleSort(idx)}
                      className="focus-ring flex min-h-11 w-full items-center gap-1.5 px-3 py-2 text-left text-xs font-medium text-ink outline-none hover:bg-track md:min-h-8"
                    >
                      <span>{label}</span>
                      {active ? (
                        sortDir === "asc" ? (
                          <ArrowUp className="size-3 shrink-0 text-ink" strokeWidth={1.75} aria-hidden />
                        ) : (
                          <ArrowDown className="size-3 shrink-0 text-ink" strokeWidth={1.75} aria-hidden />
                        )
                      ) : (
                        <ArrowUpDown className="size-3 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                      )}
                    </button>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody className="divide-y divide-hairline">
            {visible.length === 0 ? (
              <tr>
                <td colSpan={header.length} className="px-3 py-4 text-center text-sm text-ink-2">
                  No rows match &ldquo;{query}&rdquo;.
                </td>
              </tr>
            ) : (
              visible.map((r, rIdx) => (
                <tr key={safePage * CSV_PAGE_SIZE + rIdx} className="hover:bg-board/50">
                  {header.map((_, cIdx) => {
                    const val = r[cIdx] ?? "";
                    return (
                      <td
                        key={cIdx}
                        className={isNumeric(val) ? "mono whitespace-nowrap px-3 py-2 text-right text-ink" : "px-3 py-2 text-ink"}
                      >
                        {val}
                      </td>
                    );
                  })}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
          <span className="mono">
            Rows {safePage * CSV_PAGE_SIZE + 1} to {Math.min((safePage + 1) * CSV_PAGE_SIZE, processed.length)} of {processed.length}
          </span>
          <div className="flex items-center gap-1.5">
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11"
              disabled={safePage === 0}
              onClick={() => setPage((p) => Math.max(0, p - 1))}
            >
              Prev
            </Button>
            <span className="mono px-1">
              {safePage + 1} / {totalPages}
            </span>
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11"
              disabled={safePage >= totalPages - 1}
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

function JsonTreeNode({
  name,
  value,
  depth,
  forceState,
}: {
  name?: string;
  value: unknown;
  depth: number;
  forceState: { mode: "auto" | "expand" | "collapse"; rev: number };
}) {
  const isArr = Array.isArray(value);
  const isObj = !isArr && value !== null && typeof value === "object";
  const isBranch = isArr || isObj;
  const [open, setOpen] = useState(depth < 2);

  useEffect(() => {
    if (forceState.mode === "expand") setOpen(true);
    else if (forceState.mode === "collapse") setOpen(depth === 0);
  }, [forceState.mode, forceState.rev, depth]);

  if (!isBranch) {
    const formatted =
      typeof value === "string"
        ? JSON.stringify(value)
        : value === null
          ? "null"
          : String(value);
    const valCls =
      typeof value === "number"
        ? "text-info-foreground"
        : typeof value === "boolean"
          ? "text-success-foreground"
          : value === null
            ? "text-ink-2"
            : "text-ink";
    return (
      <div className="flex items-baseline gap-1.5 py-0.5 pl-5 font-mono text-xs leading-5">
        {name !== undefined && <span className="shrink-0 font-medium text-ink-2">{name}:</span>}
        <span className={cn("break-all", valCls)}>{formatted}</span>
      </div>
    );
  }

  const entries: [string, unknown][] = isArr
    ? (value as unknown[]).map((v, i) => [String(i), v])
    : Object.entries(value as Record<string, unknown>);
  const summary = isArr ? `[ ${entries.length} ${entries.length === 1 ? "item" : "items"} ]` : `{ ${entries.length} ${entries.length === 1 ? "key" : "keys"} }`;

  return (
    <div className="py-0.5 font-mono text-xs">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
        className="focus-ring flex min-h-8 items-center gap-1 rounded-sm px-1 text-left text-xs text-ink outline-none hover:bg-track max-md:min-h-11"
      >
        {open ? (
          <ChevronDown className="size-3.5 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
        ) : (
          <ChevronRight className="size-3.5 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
        )}
        {name !== undefined && <span className="font-medium text-ink">{name}:</span>}
        <span className="text-ink-2">{summary}</span>
      </button>
      {open && (
        <div className="ml-3 border-l border-hairline pl-2.5">
          {entries.slice(0, 200).map(([k, v]) => (
            <JsonTreeNode key={k} name={k} value={v} depth={depth + 1} forceState={forceState} />
          ))}
          {entries.length > 200 && (
            <p className="py-1 pl-5 text-xs text-ink-2">… {entries.length - 200} more entries (switch to Raw to view full JSON)</p>
          )}
        </div>
      )}
    </div>
  );
}

function JsonViewer({ content }: { content: string }) {
  const [mode, setMode] = useState<"tree" | "raw">("tree");
  const [forceState, setForceState] = useState<{ mode: "auto" | "expand" | "collapse"; rev: number }>({ mode: "auto", rev: 0 });

  const parsed = useMemo(() => {
    try {
      const value = JSON.parse(content) as unknown;
      return { ok: true as const, value, pretty: JSON.stringify(value, null, 2) };
    } catch {
      return { ok: false as const, value: null, pretty: content };
    }
  }, [content]);

  if (!parsed.ok) {
    return (
      <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded-sm border border-hairline bg-board p-3.5 font-mono text-xs leading-5 text-ink">
        {content}
      </pre>
    );
  }

  return (
    <div className="flex flex-col gap-2.5">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <SegmentedControl aria-label="JSON view mode">
          <SegmentedControlItem
            data-id="tree"
            aria-pressed={mode === "tree"}
            data-checked={mode === "tree" ? "" : undefined}
            className="min-h-11 px-3 md:min-h-8"
            onClick={() => setMode("tree")}
          >
            Collapsible tree
          </SegmentedControlItem>
          <SegmentedControlItem
            data-id="raw"
            aria-pressed={mode === "raw"}
            data-checked={mode === "raw" ? "" : undefined}
            className="min-h-11 px-3 md:min-h-8"
            onClick={() => setMode("raw")}
          >
            Raw JSON
          </SegmentedControlItem>
        </SegmentedControl>

        {mode === "tree" && (
          <div className="flex items-center gap-1.5">
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11"
              onClick={() => setForceState((s) => ({ mode: "expand", rev: s.rev + 1 }))}
            >
              Expand all
            </Button>
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11"
              onClick={() => setForceState((s) => ({ mode: "collapse", rev: s.rev + 1 }))}
            >
              Collapse all
            </Button>
          </div>
        )}
      </div>

      {mode === "tree" ? (
        <div id="json-tree-view" className="rounded-sm border border-hairline bg-board/40 p-3">
          <JsonTreeNode value={parsed.value} depth={0} forceState={forceState} />
        </div>
      ) : (
        <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded-sm border border-hairline bg-board p-3.5 font-mono text-xs leading-5 text-ink">
          {parsed.pretty}
        </pre>
      )}
    </div>
  );
}

function normalizeRevision(raw: unknown, fallbackPath: string): FileRevision | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const revId = String(o.rev_id ?? o.revision_id ?? o.id ?? "").trim();
  if (!revId) return null;
  const ts =
    typeof o.ts === "number"
      ? o.ts
      : typeof o.timestamp_ms === "number"
        ? o.timestamp_ms
        : typeof o.recorded_ms === "number"
          ? o.recorded_ms
          : typeof o.created_ms === "number"
            ? o.created_ms
            : typeof o.mtime_ms === "number"
              ? o.mtime_ms
              : 0;
  return {
    rev_id: revId,
    path: String(o.path ?? fallbackPath),
    ts,
    size:
      typeof o.size_bytes === "number"
        ? o.size_bytes
        : typeof o.size === "number"
          ? o.size
          : typeof o.bytes === "number"
            ? o.bytes
            : 0,
    sha256: typeof o.sha256 === "string" ? o.sha256 : typeof o.digest === "string" ? o.digest : undefined,
    reason: typeof o.reason === "string" ? o.reason : typeof o.action === "string" ? o.action : typeof o.event === "string" ? o.event : undefined,
    content: typeof o.content === "string" ? o.content : undefined,
    preview: typeof o.preview === "string" ? o.preview : undefined,
  };
}

export function FileHistoryDiffView({
  filePath,
  currentContent,
  status,
  revisions,
  loading,
  err,
  onReloadHistory,
  onRestored,
}: {
  filePath: string;
  currentContent: string;
  status?: Status | null | "error";
  revisions: FileRevision[];
  loading: boolean;
  err: string;
  onReloadHistory: () => void;
  onRestored: () => Promise<void>;
}) {
  const [selectedId, setSelectedId] = useState<string>("");
  const [compareTarget, setCompareTarget] = useState<string>("current");
  const [diffMode, setDiffMode] = useState<"unified" | "split">("unified");
  const [serverDiff, setServerDiff] = useState<FileDiffPayload | null>(null);
  const [diffLoading, setDiffLoading] = useState(false);
  const [restoring, setRestoring] = useState(false);

  useEffect(() => {
    if (!revisions.length) {
      setSelectedId("");
      setCompareTarget("current");
      setServerDiff(null);
      return;
    }
    setSelectedId((prev) => (revisions.some((r) => r.rev_id === prev) ? prev : revisions[0].rev_id));
  }, [revisions]);

  const selectedRev = useMemo(
    () => revisions.find((r) => r.rev_id === selectedId) ?? revisions[0] ?? null,
    [revisions, selectedId]
  );

  const compareRev = useMemo(
    () => (compareTarget === "current" ? null : revisions.find((r) => r.rev_id === compareTarget) ?? null),
    [revisions, compareTarget]
  );

  // Compute local diff immediately when revision content is present, or fetch server diff on demand.
  const localDiff = useMemo<FileDiffPayload | null>(() => {
    if (!selectedRev || typeof selectedRev.content !== "string") return null;
    if (compareTarget === "current") {
      return computeLineDiff(selectedRev.content, currentContent, {
        path: filePath,
        rev_a: selectedRev.rev_id,
        rev_b: "current",
      });
    }
    if (compareRev && typeof compareRev.content === "string") {
      return computeLineDiff(selectedRev.content, compareRev.content, {
        path: filePath,
        rev_a: selectedRev.rev_id,
        rev_b: compareRev.rev_id,
      });
    }
    return null;
  }, [selectedRev, compareTarget, compareRev, currentContent, filePath]);

  useEffect(() => {
    if (!selectedRev || localDiff || !hasV070Api(status, "file_history")) {
      setServerDiff(null);
      setDiffLoading(false);
      return;
    }
    let cancelled = false;
    setDiffLoading(true);
    const params = new URLSearchParams({ path: filePath, rev_a: selectedRev.rev_id });
    if (compareTarget && compareTarget !== "current") params.set("rev_b", compareTarget);
    else params.set("rev_b", "current");
    call<Record<string, unknown>>("/api/files/history?" + params.toString()).then((r) => {
      if (cancelled) return;
      setDiffLoading(false);
      if (!r.ok) {
        setServerDiff(null);
        return;
      }
      const normalized = normalizeServerDiff(r.data);
      if (normalized) {
        setServerDiff(normalized);
        return;
      }
      if (typeof r.data.content === "string") {
        setServerDiff(
          computeLineDiff(r.data.content, compareTarget === "current" ? currentContent : (compareRev?.content ?? ""), {
            path: filePath,
            rev_a: selectedRev.rev_id,
            rev_b: compareTarget,
          })
        );
        return;
      }
      setServerDiff(null);
    });
    return () => { cancelled = true; };
  }, [selectedRev, compareTarget, compareRev, localDiff, status, filePath, currentContent]);

  const activeDiff = localDiff ?? serverDiff;

  const handleRestore = async () => {
    if (!selectedRev) return;
    setRestoring(true);
    const r = await call<Record<string, unknown>>("/api/files/restore", {
      method: "POST",
      body: {
        path: filePath,
        rev_id: selectedRev.rev_id,
        revision_id: selectedRev.rev_id,
        id: selectedRev.rev_id,
      },
    });
    setRestoring(false);
    if (!r.ok) {
      toast.error(r.error || "Could not restore this file version.");
      return;
    }
    toast.success(`Restored ${filePath} to version ${selectedRev.rev_id.slice(0, 14)}.`);
    await onRestored();
    onReloadHistory();
  };

  if (!hasV070Api(status, "file_history")) {
    return (
      <div id="file-history-panel" className="rounded-sm border border-hairline bg-board/40 p-4 text-sm text-ink-2">
        Automatic file version snapshots and restore activate on Ergon v0.7.0+ servers (and in the standalone browser workspace).
      </div>
    );
  }

  if (loading) {
    return (
      <div id="file-history-panel" className="py-4 text-sm text-ink-2">
        Loading revision history…
      </div>
    );
  }

  if (err) {
    return (
      <div id="file-history-panel" role="alert" className="rounded-sm border border-hairline bg-board p-3.5 text-sm text-ink">
        {err}
      </div>
    );
  }

  if (!revisions.length) {
    return (
      <div id="file-history-panel" className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-4 text-sm text-ink-2">
        <p className="font-medium text-ink">No prior versions recorded yet</p>
        <p>
          Ergon automatically saves snapshots under <span className="mono">.history/</span> whenever this file is overwritten or deleted, so you can diff changes and restore any version in one click.
        </p>
      </div>
    );
  }

  return (
    <div id="file-history-panel" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1.5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span className="text-xs font-medium text-ink">
            Recorded snapshots ({revisions.length})
          </span>
          {selectedRev && (
            <div className="flex flex-wrap items-center gap-2">
              <label htmlFor="diff-compare-select" className="text-xs text-ink-2">Compare against</label>
              <select
                id="diff-compare-select"
                value={compareTarget}
                onChange={(e) => setCompareTarget(e.target.value)}
                className="focus-ring h-9 min-h-11 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none md:min-h-8"
              >
                <option value="current">Current version</option>
                {revisions
                  .filter((r) => r.rev_id !== selectedRev.rev_id)
                  .map((r) => (
                    <option key={r.rev_id} value={r.rev_id}>
                      {r.rev_id.slice(0, 14)} ({r.ts ? relTime(r.ts) : "snapshot"})
                    </option>
                  ))}
              </select>
            </div>
          )}
        </div>

        <ul role="list" aria-label="File revisions" className="flex max-h-40 flex-col gap-1 overflow-y-auto rounded-sm border border-hairline bg-board/30 p-1.5">
          {revisions.map((rev) => {
            const active = selectedRev?.rev_id === rev.rev_id;
            const iso = rev.ts > 0 ? new Date(rev.ts).toISOString().replace("T", " ").slice(0, 19) + " UTC" : "";
            return (
              <li key={rev.rev_id}>
                <button
                  type="button"
                  data-rev-id={rev.rev_id}
                  aria-pressed={active}
                  onClick={() => setSelectedId(rev.rev_id)}
                  className={cn(
                    "focus-ring flex min-h-11 w-full cursor-pointer flex-wrap items-center justify-between gap-2 rounded-sm border px-2.5 py-1.5 text-left text-xs outline-none md:min-h-9",
                    active
                      ? "border-blue bg-blue-soft text-ink"
                      : "border-transparent bg-surface text-ink hover:bg-board"
                  )}
                >
                  <span className="flex min-w-0 flex-wrap items-center gap-1.5">
                    <span className="mono font-medium text-ink">{rev.rev_id.slice(0, 16)}</span>
                    {rev.reason && (
                      <Badge variant="outline" size="sm" className="text-xs">
                        {rev.reason}
                      </Badge>
                    )}
                    {rev.ts > 0 && (
                      <span className="text-ink-2" title={iso}>
                        {relTime(rev.ts)}
                      </span>
                    )}
                  </span>
                  <span className="mono flex shrink-0 items-center gap-2 text-ink-2">
                    <span>{formatBytes(rev.size)}</span>
                    {rev.sha256 && <span>{rev.sha256.slice(0, 8)}</span>}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      </div>

      {selectedRev && (
        <div className="flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board/50 px-3 py-2">
            <div className="mono flex flex-wrap items-center gap-2 text-xs">
              <span className="font-medium text-ink">
                {selectedRev.rev_id.slice(0, 14)} → {compareTarget === "current" ? "Current" : compareTarget.slice(0, 14)}
              </span>
              {activeDiff && (
                <>
                  <span className="rounded-chip bg-pill-active-bg px-1.5 py-0.5 font-medium text-pill-active-fg">
                    +{activeDiff.added}
                  </span>
                  <span className="rounded-chip bg-pill-pro-bg px-1.5 py-0.5 font-medium text-pill-pro-fg">
                    -{activeDiff.removed}
                  </span>
                </>
              )}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <SegmentedControl aria-label="Diff layout">
                <SegmentedControlItem
                  data-id="unified"
                  aria-pressed={diffMode === "unified"}
                  data-checked={diffMode === "unified" ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setDiffMode("unified")}
                >
                  Unified
                </SegmentedControlItem>
                <SegmentedControlItem
                  data-id="split"
                  aria-pressed={diffMode === "split"}
                  data-checked={diffMode === "split" ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setDiffMode("split")}
                >
                  Side-by-side
                </SegmentedControlItem>
              </SegmentedControl>
              <Button
                id="append-file-delta-btn"
                type="button"
                size="sm"
                variant="outline"
                className="max-md:min-h-11"
                onClick={async () => {
                  try {
                    const res = await exportClientPackDownload(
                      [filePath],
                      `${filePath} Delta Pack`,
                      {
                        delta_file: filePath,
                        rev_a: selectedRev.rev_id,
                        rev_b: compareTarget || "current",
                      }
                    );
                    toast.success(`Exported ${res.filename} with snapshot delta (sha256: ${res.sha256.slice(0, 12)}…)`);
                  } catch {
                    toast.error("Could not append snapshot delta to Client Pack.");
                  }
                }}
              >
                Append Delta to Client Pack (.md)
              </Button>
              <Button
                id="restore-version-btn"
                type="button"
                size="sm"
                className="max-md:min-h-11"
                disabled={restoring}
                onClick={handleRestore}
              >
                <RotateCcw className="size-3.5" strokeWidth={1.5} aria-hidden />
                {restoring ? "Restoring…" : "Restore version"}
              </Button>
            </div>
          </div>

          <div id="file-diff-view" className="max-h-[34vh] overflow-auto rounded-sm border border-hairline bg-surface font-mono text-xs leading-5">
            {diffLoading ? (
              <p className="p-3 text-ink-2">Computing diff…</p>
            ) : !activeDiff ? (
              <p className="p-3 text-ink-2">
                Select Restore version above to revert this file to snapshot <span className="mono">{selectedRev.rev_id}</span>.
              </p>
            ) : activeDiff.hunks.length === 0 ? (
              <p className="p-3 text-ink-2">No line differences between these two versions (identical content).</p>
            ) : diffMode === "split" ? (
              <div className="min-w-0">
                <div className="sticky top-0 z-10 grid grid-cols-2 border-b border-hairline bg-board text-xs font-medium text-ink-2">
                  <div className="truncate border-r border-hairline px-2.5 py-1">
                    Snapshot ({selectedRev.rev_id.slice(0, 14)})
                  </div>
                  <div className="truncate px-2.5 py-1">
                    {compareTarget === "current" ? "Current file" : `Snapshot (${compareTarget.slice(0, 14)})`}
                  </div>
                </div>
                {activeDiff.hunks.map((hunk, hi) => (
                  <div key={hi}>
                    {hunk.header && (
                      <div className="border-y border-hairline bg-board/60 px-3 py-1 text-ink-2">
                        {hunk.header}
                      </div>
                    )}
                    {hunk.lines.map((line, li) => {
                      const isDel = line.kind === "del";
                      const isAdd = line.kind === "add";
                      const leftCls = isDel ? "bg-pill-pro-bg text-pill-pro-fg" : isAdd ? "bg-board/30 text-ink-3" : "bg-surface text-ink";
                      const rightCls = isAdd ? "bg-pill-active-bg text-pill-active-fg" : isDel ? "bg-board/30 text-ink-3" : "bg-surface text-ink";
                      return (
                        <div key={`${hi}-${li}`} className="grid grid-cols-2 border-b border-hairline/40">
                          <div
                            data-diff-kind={isDel ? "del" : undefined}
                            className={cn("grid min-w-0 grid-cols-[2.25rem_minmax(0,1fr)] gap-1 border-r border-hairline px-2 py-0.5", leftCls)}
                          >
                            <span className="select-none text-right opacity-75">{!isAdd ? (line.old_line ?? "") : ""}</span>
                            <span className="whitespace-pre-wrap break-all">{!isAdd ? (line.text || " ") : ""}</span>
                          </div>
                          <div
                            data-diff-kind={isAdd ? "add" : undefined}
                            className={cn("grid min-w-0 grid-cols-[2.25rem_minmax(0,1fr)] gap-1 px-2 py-0.5", rightCls)}
                          >
                            <span className="select-none text-right opacity-75">{!isDel ? (line.new_line ?? "") : ""}</span>
                            <span className="whitespace-pre-wrap break-all">{!isDel ? (line.text || " ") : ""}</span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                ))}
              </div>
            ) : (
              <div className="min-w-0">
                {activeDiff.hunks.map((hunk, hi) => (
                  <div key={hi}>
                    {hunk.header && (
                      <div className="border-y border-hairline bg-board px-3 py-1 text-ink-2">
                        {hunk.header}
                      </div>
                    )}
                    {hunk.lines.map((line, li) => {
                      const cls =
                        line.kind === "add"
                          ? "bg-pill-active-bg text-pill-active-fg"
                          : line.kind === "del"
                            ? "bg-pill-pro-bg text-pill-pro-fg"
                            : "bg-surface text-ink";
                      const sign = line.kind === "add" ? "+" : line.kind === "del" ? "-" : " ";
                      return (
                        <div
                          key={`${hi}-${li}`}
                          data-diff-kind={line.kind}
                          className={cn("grid grid-cols-[2.5rem_2.5rem_1.25rem_minmax(0,1fr)] gap-1 px-2 py-0.5", cls)}
                        >
                          <span className="select-none text-right opacity-75">
                            {line.old_line ?? ""}
                          </span>
                          <span className="select-none text-right opacity-75">
                            {line.new_line ?? ""}
                          </span>
                          <span className="select-none text-center font-semibold">{sign}</span>
                          <span className="whitespace-pre-wrap break-all">{line.text || " "}</span>
                        </div>
                      );
                    })}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function parseViewerTarget(raw: string): { path: string; sheet?: string; line?: number; endLine?: number } {
  const trimmed = String(raw || "").trim();
  const hashIdx = trimmed.indexOf("#");
  const basePart = hashIdx >= 0 ? trimmed.slice(0, hashIdx) : trimmed;
  const hashPart = hashIdx >= 0 ? trimmed.slice(hashIdx) : "";
  const qIdx = basePart.indexOf("?sheet=");
  if (qIdx >= 0) {
    const cleanBase = basePart.slice(0, qIdx);
    const sheet = decodeURIComponent(basePart.slice(qIdx + "?sheet=".length));
    const parsed = parseCitationTarget(`${cleanBase}${hashPart}`);
    return { ...parsed, sheet: sheet || undefined };
  }
  return parseCitationTarget(trimmed);
}

function buildVaultJumpTarget(path: string, ch: VaultChunkHit): string {
  const sheetPart = ch.sheet ? `?sheet=${encodeURIComponent(ch.sheet)}` : "";
  const start = Number.isFinite(ch.line_start) && ch.line_start > 0 ? Math.floor(ch.line_start) : 0;
  const end = Number.isFinite(ch.line_end) && ch.line_end > start ? Math.floor(ch.line_end) : 0;
  const linePart = start > 0 ? `#L${start}${end > start ? `-L${end}` : ""}` : "";
  return `${path}${sheetPart}${linePart}`;
}

function SourceLineView({
  content,
  highlightLine,
  highlightEnd,
}: {
  content: string;
  highlightLine?: number;
  highlightEnd?: number;
}) {
  const targetRef = useRef<HTMLDivElement>(null);
  const lines = useMemo(() => String(content ?? "").split(/\r?\n/), [content]);
  const { line: clampedStart, endLine: clampedEnd } = useMemo(
    () => clampLineBounds(lines.length, highlightLine, highlightEnd),
    [lines.length, highlightLine, highlightEnd]
  );

  useEffect(() => {
    if (!clampedStart) return;
    const scrollTarget = () => {
      targetRef.current?.scrollIntoView({ block: "center", behavior: "auto" });
    };
    scrollTarget();
    const raf = window.requestAnimationFrame(scrollTarget);
    return () => window.cancelAnimationFrame(raf);
  }, [clampedStart, content]);

  const endLine = clampedEnd && clampedEnd >= (clampedStart ?? 0) ? clampedEnd : clampedStart;

  return (
    <div
      id="file-source-lines"
      data-highlighted-line={clampedStart ?? undefined}
      className="overflow-x-auto rounded-sm border border-hairline bg-board/40 font-mono text-xs leading-5"
    >
      {lines.map((line, idx) => {
        const lineNo = idx + 1;
        const active =
          typeof clampedStart === "number" &&
          lineNo >= clampedStart &&
          lineNo <= (endLine ?? clampedStart);
        const isAnchor = lineNo === clampedStart;
        return (
          <div
            key={lineNo}
            id={`viewer-line-${lineNo}`}
            ref={isAnchor ? targetRef : undefined}
            data-active-line={active ? "true" : undefined}
            className={cn(
              "grid grid-cols-[3.25rem_minmax(0,1fr)] gap-3 px-2.5 py-0.5",
              active ? "border-l-2 border-accent bg-tag-plan-bg/60 text-ink" : "text-ink"
            )}
          >
            <span className="select-none text-right text-ink-3 tabular-nums">{lineNo}</span>
            <span className="whitespace-pre-wrap break-words">{line || " "}</span>
          </div>
        );
      })}
    </div>
  );
}

export function FileViewerModal({
  filePath,
  personName,
  status,
  onClose,
  onAskAbout,
  onRunWorkflow,
  onRestored,
}: {
  filePath: string | null;
  personName?: string;
  status?: Status | null | "error";
  onClose: () => void;
  onAskAbout?: (path: string) => void;
  onRunWorkflow?: (prompt: string) => void;
  onRestored?: () => void;
}) {
  const [overridePath, setOverridePath] = useState<string | null>(null);
  const effectiveFilePath = overridePath ?? filePath;
  const target = useMemo(() => parseViewerTarget(effectiveFilePath || ""), [effectiveFilePath]);
  const cleanPath = effectiveFilePath ? target.path : null;
  const [data, setData] = useState<WorkspaceFileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");
  const [exporting, setExporting] = useState(false);
  const [tab, setTab] = useState<"preview" | "edit" | "source" | "history">("preview");
  const [editorDirty, setEditorDirty] = useState(false);
  const [pendingNav, setPendingNav] = useState<
    | { kind: "close" }
    | { kind: "tab"; nextTab: "preview" | "source" | "history" }
    | { kind: "openFile"; nextPath: string }
    | { kind: "ask" }
    | { kind: "workflow"; prompt: string }
    | null
  >(null);
  const [revisions, setRevisions] = useState<FileRevision[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyErr, setHistoryErr] = useState("");

  useEffect(() => {
    setOverridePath(null);
  }, [filePath]);

  useEffect(() => {
    if (!editorDirty) setPendingNav(null);
  }, [editorDirty]);

  const loadContent = useCallback(async (path: string, sheet?: string, preserveOnTransientErr = false): Promise<boolean> => {
    setLoading(true);
    setErr("");
    const r = await call<WorkspaceFileContent>(
      "/api/files/content?path=" + encodeURIComponent(path) + (sheet ? "&sheet=" + encodeURIComponent(sheet) : "")
    );
    setLoading(false);
    if (!r.ok) {
      if (!preserveOnTransientErr || r.status === 404) {
        setData(null);
        setErr(r.status === 404 ? `"${path}" is no longer in this workspace (it may have been deleted or moved).` : r.error || "Could not open this file.");
      }
      return false;
    }
    setData(r.data);
    return true;
  }, []);

  const loadHistory = useCallback(async (path: string) => {
    if (!hasV070Api(status, "file_history")) {
      setRevisions([]);
      setHistoryLoading(false);
      setHistoryErr("");
      return;
    }
    setHistoryLoading(true);
    setHistoryErr("");
    const r = await call<{
      revisions?: unknown[];
      history?: unknown[];
      versions?: unknown[];
    }>("/api/files/history?path=" + encodeURIComponent(path));
    setHistoryLoading(false);
    if (!r.ok) {
      setRevisions([]);
      if (r.status !== 404) setHistoryErr(r.error || "Could not load file version history.");
      return;
    }
    const rawList = Array.isArray(r.data.revisions)
      ? r.data.revisions
      : Array.isArray(r.data.history)
        ? r.data.history
        : Array.isArray(r.data.versions)
          ? r.data.versions
          : [];
    const norm = rawList
      .map((item) => normalizeRevision(item, path))
      .filter((x): x is FileRevision => x !== null)
      .sort((a, b) => b.ts - a.ts);
    setRevisions(norm);
  }, [status]);

  const hasLineTarget = typeof target.line === "number" && Number.isFinite(target.line);
  const totalContentLines = useMemo(
    () => (data && !data.is_binary ? Math.max(1, String(data.content ?? "").split(/\r?\n/).length) : 0),
    [data]
  );
  const { line: clampedLine, endLine: clampedEndLine } = useMemo(() => {
    if (!hasLineTarget) return { line: undefined, endLine: undefined };
    if (totalContentLines > 0) {
      return clampLineBounds(totalContentLines, target.line, target.endLine);
    }
    const start = Math.max(1, Math.floor(target.line!));
    const end =
      typeof target.endLine === "number" && Number.isFinite(target.endLine)
        ? Math.max(start, Math.floor(target.endLine))
        : undefined;
    return { line: start, endLine: end };
  }, [hasLineTarget, totalContentLines, target.line, target.endLine]);

  useEffect(() => {
    setEditorDirty(false);
    setPendingNav(null);
    if (!cleanPath) {
      setData(null);
      setErr("");
      setTab("preview");
      setRevisions([]);
      setHistoryErr("");
      return;
    }
    setTab(hasLineTarget ? "source" : "preview");
    void loadContent(cleanPath, target.sheet);
    void loadHistory(cleanPath);
  }, [cleanPath, hasLineTarget, target.line, target.sheet, loadContent, loadHistory]);

  const ext = (data?.ext || cleanPath?.split(".").pop() || "").toLowerCase();

  const requestTabChange = (nextTab: "preview" | "edit" | "source" | "history") => {
    if (nextTab === tab) return;
    if (tab === "edit" && editorDirty && nextTab !== "edit") {
      setPendingNav({ kind: "tab", nextTab });
      return;
    }
    setPendingNav(null);
    setTab(nextTab);
    if (nextTab === "history" && cleanPath) void loadHistory(cleanPath);
  };

  const requestClose = () => {
    if (tab === "edit" && editorDirty) {
      setPendingNav({ kind: "close" });
      return;
    }
    setPendingNav(null);
    onClose();
  };

  const requestOpenFile = (nextPath: string) => {
    if (tab === "edit" && editorDirty) {
      setPendingNav({ kind: "openFile", nextPath });
      return;
    }
    setPendingNav(null);
    setOverridePath(nextPath);
    setTab("preview");
    onRestored?.();
  };

  const confirmDiscardNav = () => {
    const act = pendingNav;
    setEditorDirty(false);
    setPendingNav(null);
    if (!act) return;
    if (act.kind === "close") {
      onClose();
      return;
    }
    if (act.kind === "openFile") {
      setOverridePath(act.nextPath);
      setTab("preview");
      onRestored?.();
      return;
    }
    if (act.kind === "ask") {
      if (cleanPath && onAskAbout) onAskAbout(cleanPath);
      onClose();
      return;
    }
    if (act.kind === "workflow") {
      onRunWorkflow?.(act.prompt);
      onClose();
      return;
    }
    setTab(act.nextTab);
    if (act.nextTab === "history" && cleanPath) void loadHistory(cleanPath);
  };

  const copyContent = async () => {
    if (!data?.content) return;
    try {
      await navigator.clipboard.writeText(data.content);
      toast.success("Copied file contents.");
    } catch {
      toast.error("Could not access clipboard.");
    }
  };

  const handleDownload = async () => {
    if (!cleanPath) return;
    const ok = await downloadWorkspaceFile(cleanPath);
    if (ok) toast.success(`Downloaded ${cleanPath.split("/").pop()}.`);
    else toast.error("Could not download this file.");
  };

  const expTarget = cleanPath ? exportTarget(cleanPath) : null;
  const canPdf = cleanPath ? canExportPdf(cleanPath, data?.extracted?.format) : false;
  const suggestedWorkflow = cleanPath ? detectWorkflowForFile(cleanPath) : null;
  const [savingAs, setSavingAs] = useState(false);
  const [savingPdf, setSavingPdf] = useState(false);
  const handleSaveAs = async () => {
    if (!cleanPath || !expTarget) return;
    setSavingAs(true);
    const r = await exportFile({ path: cleanPath, format: expTarget });
    setSavingAs(false);
    if (!r.ok) {
      toast.error(r.error ? "Could not save a copy: " + r.error : "Could not save a copy.");
      return;
    }
    toast.success(`Saved ${r.data.path} to your workspace.`);
    await downloadWorkspaceFile(r.data.path);
    onRestored?.();
  };

  const handleSaveAsPdf = async () => {
    if (!cleanPath || !canPdf) return;
    setSavingPdf(true);
    const r = await exportFile({ path: cleanPath, format: "pdf" });
    setSavingPdf(false);
    if (!r.ok) {
      toast.error(r.error ? "Could not export PDF: " + r.error : "Could not export PDF.");
      return;
    }
    toast.success(`Saved ${r.data.path} to your workspace.`);
    await downloadWorkspaceFile(r.data.path);
    onRestored?.();
  };

  const handleReceiptExport = async (format: "md" | "json") => {
    if (!cleanPath || !data) return;
    setExporting(true);
    try {
      const res = await exportReceiptDownload(
        {
          id: `file-${cleanPath}`,
          task: `Workspace file receipt: ${cleanPath}`,
          person: personName || "Active employee",
          status: "verified",
          summary: `Verified workspace file (${formatBytes(data.size)}, ${ext || "text"}).`,
          ts: data.mtime_ms || Date.now(),
          files: [cleanPath],
          filePath: cleanPath,
          fileContent: data.content,
        },
        format,
        false
      );
      toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
    } catch {
      toast.error("Could not export SHA-256 receipt.");
    } finally {
      setExporting(false);
    }
  };

  return (
    <Dialog open={!!filePath} onOpenChange={(open) => { if (!open) requestClose(); }}>
      <DialogContent
        id="file-viewer-dialog"
        className={cn("max-h-[88vh] overflow-hidden", tab === "edit" ? "max-w-5xl" : "max-w-3xl")}
      >
        <DialogHeader className="pr-10 text-left">
          <DialogTitle className="flex min-w-0 items-center gap-2 text-title font-medium text-ink">
            {cleanPath && <FileGlyph name={cleanPath} className="size-4 shrink-0" />}
            <span className="mono min-w-0 truncate">{cleanPath ?? ""}</span>
            {target.sheet && (
              <Badge variant="outline" size="sm" className="shrink-0">
                #{target.sheet}
              </Badge>
            )}
            {clampedLine && (
              <Badge variant="outline" size="sm" className="mono shrink-0">
                L{clampedLine}{clampedEndLine ? ` to L${clampedEndLine}` : ""}
              </Badge>
            )}
          </DialogTitle>
          <DialogDescription className="flex flex-wrap items-center gap-2 text-xs text-ink-2">
            {data ? (
              <>
                <span className="mono">{formatBytes(data.size)}</span>
                {data.mtime_ms > 0 && (
                  <>
                    <span aria-hidden>·</span>
                    <span>Updated {relTime(data.mtime_ms)}</span>
                  </>
                )}
                {ext && <Badge variant="outline" size="sm" className="mono">{ext}</Badge>}
              </>
            ) : (
              <span>Workspace file preview</span>
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-wrap items-center justify-between gap-2 border-y border-hairline py-2.5">
          <SegmentedControl aria-label="File viewer mode">
            <SegmentedControlItem
              data-id="preview"
              aria-pressed={tab === "preview"}
              data-checked={tab === "preview" ? "" : undefined}
              className="min-h-11 px-3 md:min-h-8"
              onClick={() => requestTabChange("preview")}
            >
              Preview
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="edit"
              aria-pressed={tab === "edit"}
              data-checked={tab === "edit" ? "" : undefined}
              disabled={!data || data.is_binary || !!err}
              className="min-h-11 gap-1.5 px-3 md:min-h-8"
              onClick={() => requestTabChange("edit")}
            >
              <Pencil className="size-3.5" strokeWidth={1.5} aria-hidden />
              <span>Edit</span>
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="source"
              aria-pressed={tab === "source"}
              data-checked={tab === "source" ? "" : undefined}
              className="min-h-11 px-3 md:min-h-8"
              onClick={() => requestTabChange("source")}
            >
              {clampedLine ? `Source (L${clampedLine})` : "Lines"}
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="history"
              aria-pressed={tab === "history"}
              data-checked={tab === "history" ? "" : undefined}
              className="min-h-11 gap-1.5 px-3 md:min-h-8"
              onClick={() => requestTabChange("history")}
            >
              <History className="size-3.5" strokeWidth={1.5} aria-hidden />
              <span>History &amp; Diff</span>
              {revisions.length > 0 && (
                <span className="mono rounded-chip bg-track px-1.5 text-xs text-ink-2">
                  {revisions.length}
                </span>
              )}
            </SegmentedControlItem>
          </SegmentedControl>

          <div className="flex flex-wrap items-center gap-2">
            <Button size="sm" variant="outline" className="max-md:min-h-11" disabled={!cleanPath || !!err} onClick={handleDownload}>
              <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
              Download
            </Button>
            {expTarget && data && !err && (
              <Button id="file-save-as" size="sm" variant="outline" className="max-md:min-h-11" disabled={savingAs} onClick={handleSaveAs}>
                {expTarget === "docx" ? (
                  <FileText className="size-3.5" strokeWidth={1.5} aria-hidden />
                ) : (
                  <FileSpreadsheet className="size-3.5" strokeWidth={1.5} aria-hidden />
                )}
                {savingAs ? "Saving…" : expTarget === "docx" ? "Save as Word" : "Save as Excel"}
              </Button>
            )}
            {data && !err && (
              <span
                className="inline-flex"
                title={
                  canPdf
                    ? "Export a formatted PDF 1.4 copy to your workspace"
                    : `PDF export is only available for .md, .txt, .csv, and .tsv files (not .${ext || "binary"})`
                }
              >
                <Button
                  id="file-save-pdf"
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={savingPdf || !canPdf}
                  title={
                    canPdf
                      ? "Export a formatted PDF 1.4 copy to your workspace"
                      : `PDF export is only available for .md, .txt, .csv, and .tsv files (not .${ext || "binary"})`
                  }
                  onClick={handleSaveAsPdf}
                >
                  <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {savingPdf ? "Exporting…" : "Save as PDF"}
                </Button>
              </span>
            )}
            {data && !data.is_binary && (
              <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={copyContent}>
                <Copy className="size-3.5" strokeWidth={1.5} aria-hidden />
                Copy text
              </Button>
            )}
            {data && !err && (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={exporting}
                  onClick={() => handleReceiptExport("md")}
                >
                  <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                  SHA-256 Receipt (.md)
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={exporting}
                  onClick={() => handleReceiptExport("json")}
                >
                  <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                  Receipt (.json)
                </Button>
              </>
            )}
            {suggestedWorkflow && onRunWorkflow && !err && (
              <Button
                size="sm"
                className="max-md:min-h-11 gap-1.5"
                onClick={() => {
                  if (tab === "edit" && editorDirty) {
                    setPendingNav({ kind: "workflow", prompt: suggestedWorkflow.prompt });
                    return;
                  }
                  onRunWorkflow(suggestedWorkflow.prompt);
                  onClose();
                }}
              >
                <Sparkles className="size-3.5" strokeWidth={1.5} aria-hidden />
                {suggestedWorkflow.label}
              </Button>
            )}
            {cleanPath && onAskAbout && !err && (
              <Button
                size="sm"
                variant="ghost"
                className="max-md:min-h-11"
                onClick={() => {
                  if (tab === "edit" && editorDirty) {
                    setPendingNav({ kind: "ask" });
                    return;
                  }
                  onAskAbout(cleanPath);
                  onClose();
                }}
              >
                <MessageSquarePlus className="size-3.5" strokeWidth={1.5} aria-hidden />
                Ask agent
              </Button>
            )}
          </div>
        </div>

        {pendingNav && (
          <div
            id="file-unsaved-guard"
            role="alert"
            className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board px-3 py-2 text-xs text-ink"
          >
            <span>You have unsaved edits in {cleanPath}. Discard changes and leave?</span>
            <div className="flex items-center gap-2">
              <Button
                id="file-unsaved-keep"
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11"
                onClick={() => setPendingNav(null)}
              >
                Keep editing
              </Button>
              <Button
                id="file-unsaved-discard"
                type="button"
                size="xs"
                className="max-md:min-h-11"
                onClick={confirmDiscardNav}
              >
                Discard changes
              </Button>
            </div>
          </div>
        )}

        <div id="file-viewer-body" className="max-h-[58vh] min-h-24 overflow-y-auto pr-1">
          {tab === "edit" && cleanPath && data && !data.is_binary && !err ? (
            <FileEditor
              filePath={cleanPath}
              initialContent={data.content}
              ext={ext}
              initialSha256={data.sha256}
              isExtracted={Boolean(data.extracted)}
              extractedFormat={data.extracted?.format}
              extractedKind={data.extracted?.kind}
              sheet={data.extracted?.sheet}
              jumpToLine={clampedLine}
              onDirtyChange={setEditorDirty}
              onReloadFromServer={async () => {
                return await loadContent(cleanPath, data.extracted?.sheet ?? undefined, true);
              }}
              onSaved={(res) => {
                setData((prev) =>
                  prev
                    ? { ...prev, content: res.content, size: res.size, mtime_ms: res.mtime_ms, sha256: res.sha256 }
                    : prev
                );
                void loadHistory(cleanPath);
                onRestored?.();
              }}
              onOpenSavedFile={requestOpenFile}
              onWorkspaceMutated={() => onRestored?.()}
            />
          ) : tab === "history" && cleanPath ? (
            <FileHistoryDiffView
              filePath={cleanPath}
              currentContent={data?.content ?? ""}
              status={status}
              revisions={revisions}
              loading={historyLoading}
              err={historyErr}
              onReloadHistory={() => void loadHistory(cleanPath)}
              onRestored={async () => {
                await loadContent(cleanPath);
                onRestored?.();
              }}
            />
          ) : (
            <>
              {loading && <p className="text-sm text-ink-2">Loading file…</p>}
              {!loading && err && <p role="alert" className="rounded-sm border border-hairline bg-board p-3.5 text-sm text-ink">{err}</p>}
              {!loading && !err && data && data.extracted && (
                <div className="mb-3 flex flex-wrap items-center gap-2 text-xs text-ink-2">
                  <span>
                    Text read from this {data.extracted.kind}
                    {data.extracted.pages && data.extracted.pages > 1 ? ` (${data.extracted.pages} pages)` : ""}: your agent and every workflow read it the same way.
                  </span>
                  {(data.extracted.sheets?.length ?? 0) > 1 && cleanPath && (
                    <SegmentedControl aria-label="Workbook sheet">
                      {data.extracted.sheets!.slice(0, 8).map((sh) => (
                        <SegmentedControlItem
                          key={sh}
                          data-id={sh}
                          aria-pressed={data.extracted?.sheet === sh}
                          data-checked={data.extracted?.sheet === sh ? "" : undefined}
                          className="min-h-11 px-3 md:min-h-8"
                          onClick={() => void loadContent(cleanPath, sh)}
                        >
                          {sh}
                        </SegmentedControlItem>
                      ))}
                    </SegmentedControl>
                  )}
                </div>
              )}
              {!loading && !err && data && (
                data.is_binary ? (
                  <p className="text-sm text-ink-2">
                    {data.extract_error
                      ? `Ergon could not read this file's text: ${data.extract_error}. Use Download above to open it on your computer.`
                      : `Binary file (${formatBytes(data.size)}). Use Download above to save it to your computer.`}
                  </p>
                ) : tab === "source" ? (
                  <SourceLineView
                    content={data.content}
                    highlightLine={clampedLine}
                    highlightEnd={clampedEndLine}
                  />
                ) : data.extracted?.format === "csv" ? (
                  <CsvTableView content={data.content} />
                ) : data.extracted?.format === "markdown" ? (
                  <MarkdownView content={data.content} />
                ) : ext === "csv" ? (
                  <CsvTableView content={data.content} />
                ) : ext === "md" || ext === "markdown" ? (
                  <MarkdownView content={data.content} />
                ) : ext === "json" ? (
                  <JsonViewer content={data.content} />
                ) : (
                  <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded-sm border border-hairline bg-board p-3.5 font-mono text-xs leading-5 text-ink">
                    {data.content || "(empty file)"}
                  </pre>
                )
              )}
            </>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export function WorkspaceFilesBody({
  files,
  onRefresh,
  onOpenFile,
  onUpload,
  onUploadMany,
  embedded = false,
  activeClient = null,
  activeEngagement = null,
  engagementScopeOnly,
  onToggleEngagementScope,
}: {
  files: WorkspaceFile[];
  onRefresh: () => void;
  onOpenFile: (path: string) => void;
  onUpload: (path: string, content: string, silent?: boolean, encoding?: "base64") => Promise<boolean>;
  onUploadMany?: (picked: FileList | File[]) => Promise<number>;
  embedded?: boolean;
  activeClient?: Client | null;
  activeEngagement?: Engagement | null;
  engagementScopeOnly?: boolean;
  onToggleEngagementScope?: (next: boolean) => void;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const [innerScoped, setInnerScoped] = useState(false);
  const scoped = Boolean(activeEngagement && (engagementScopeOnly !== undefined ? engagementScopeOnly : innerScoped));
  const setScoped = (next: boolean) => {
    if (onToggleEngagementScope) onToggleEngagementScope(next);
    else setInnerScoped(next);
  };
  const activeScopePrefix = useMemo(() => engagementWorkspacePrefix(activeEngagement), [activeEngagement]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newContent, setNewContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState("");
  const [query, setQuery] = useState("");
  const [searchMode, setSearchMode] = useState<VaultSearchMode>("hybrid");
  const [vaultHits, setVaultHits] = useState<VaultFileHit[]>([]);
  const [indexStats, setIndexStats] = useState<VaultIndexStats | null>(null);
  const [vaultSearching, setVaultSearching] = useState(false);
  const [reindexing, setReindexing] = useState(false);
  const [reindexErr, setReindexErr] = useState("");
  const [searchNonce, setSearchNonce] = useState(0);
  const [extFilter, setExtFilter] = useState<"all" | "csv" | "md" | "json" | "other">("all");
  const [page, setPage] = useState(0);
  const [dragOver, setDragOver] = useState(false);
  const searchSeqRef = useRef(0);
  const searchAbortRef = useRef<AbortController | null>(null);
  const reindexLockRef = useRef(false);
  const aggLockRef = useRef(false);

  const [aggOpen, setAggOpen] = useState(false);
  const [aggGlob, setAggGlob] = useState("*.csv");
  const [aggSheet, setAggSheet] = useState("");
  const [aggWhereCol, setAggWhereCol] = useState("");
  const [aggWhereOp, setAggWhereOp] = useState<TableWhereOp>("contains");
  const [aggWhereVal, setAggWhereVal] = useState("");
  const [aggGroupBy, setAggGroupBy] = useState("");
  const [aggFn, setAggFn] = useState<TableAggregateFn>("sum");
  const [aggCol, setAggCol] = useState("Amount");
  const [aggAlias, setAggAlias] = useState("");
  const [aggSaveAs, setAggSaveAs] = useState("vault-rollup.csv");
  const [aggBusy, setAggBusy] = useState(false);
  const [aggErr, setAggErr] = useState("");
  const [aggResult, setAggResult] = useState<VaultAggregateResult | null>(null);

  useEffect(() => {
    if (embedded || isOpen) onRefresh();
  }, [embedded, isOpen, onRefresh]);

  useEffect(() => {
    if (!embedded && !isOpen) return;
    let cancelled = false;
    const controller = new AbortController();
    searchVault({
      q: "",
      limit: 1,
      ...(scoped && activeEngagement
        ? {
            client_id: activeEngagement.client_id,
            engagement_id: activeEngagement.id,
            ...(activeScopePrefix ? { path_prefix: activeScopePrefix } : {}),
          }
        : {}),
      signal: controller.signal,
    }).then((r) => {
      if (!cancelled && !controller.signal.aborted && r.ok && r.data.index_stats) {
        setIndexStats(r.data.index_stats);
      }
    });
    return () => {
      cancelled = true;
      controller.abort();
    };
  }, [embedded, isOpen, files.length, scoped, activeEngagement?.id, activeEngagement?.client_id, activeScopePrefix]);

  useEffect(() => {
    const q = query.trim();
    searchSeqRef.current += 1;
    const reqSeq = searchSeqRef.current;
    searchAbortRef.current?.abort();
    searchAbortRef.current = null;
    if ((!embedded && !isOpen) || q.length < 2) {
      setVaultHits([]);
      setVaultSearching(false);
      return;
    }
    let cancelled = false;
    const controller = new AbortController();
    searchAbortRef.current = controller;
    setVaultSearching(true);
    const timer = window.setTimeout(async () => {
      const r = await searchVault({
        q,
        scope: "files",
        mode: searchMode,
        limit: 20,
        ...(scoped && activeEngagement
          ? {
              client_id: activeEngagement.client_id,
              engagement_id: activeEngagement.id,
              ...(activeScopePrefix ? { path_prefix: activeScopePrefix } : {}),
            }
          : {}),
        signal: controller.signal,
      });
      if (cancelled || controller.signal.aborted || reqSeq !== searchSeqRef.current) return;
      setVaultSearching(false);
      if (!r.ok) {
        setVaultHits([]);
        return;
      }
      if (r.data.index_stats) {
        setIndexStats(r.data.index_stats);
      }
      const nextHits: VaultFileHit[] = [];
      if (Array.isArray(r.data.file_hits)) {
        for (const fh of r.data.file_hits) {
          if (fh && typeof fh.path === "string" && fh.path.trim()) {
            nextHits.push({ ...fh, chunks: Array.isArray(fh.chunks) ? fh.chunks : [] });
          }
        }
      } else if (Array.isArray(r.data.files)) {
        for (const f of r.data.files) {
          const fp = String(f?.path ?? f?.name ?? "").trim();
          if (fp) {
            nextHits.push({
              path: fp,
              name: f.name,
              size: f.size,
              mtime_ms: f.mtime_ms,
              ext: f.ext,
              chunks: Array.isArray(f.chunks) ? f.chunks : [],
            });
          }
        }
      } else if (Array.isArray(r.data.results)) {
        for (const h of r.data.results) {
          if (h.kind === "file" && h.path && Array.isArray(h.chunks)) {
            nextHits.push({ path: h.path, score: h.score, chunks: h.chunks });
          }
        }
      }
      setVaultHits(nextHits);
    }, 120);
    return () => {
      cancelled = true;
      controller.abort();
      if (searchAbortRef.current === controller) {
        searchAbortRef.current = null;
      }
      window.clearTimeout(timer);
    };
  }, [query, searchMode, searchNonce, embedded, isOpen, files.length, scoped, activeEngagement?.id, activeEngagement?.client_id, activeScopePrefix]);

  const handleReindexVault = async () => {
    if (reindexLockRef.current || reindexing) return;
    reindexLockRef.current = true;
    setReindexing(true);
    setReindexErr("");
    try {
      const r = await reindexVault();
      if (!r.ok) {
        const msg = r.error || "Could not reindex workspace vault.";
        setReindexErr(msg);
        toast.error(msg);
        return;
      }
      if (r.data.index_stats) {
        setIndexStats(r.data.index_stats);
      } else {
        setIndexStats((prev) => ({
          total_files: r.data.indexed_files,
          total_chunks: r.data.total_chunks,
          indexed_bytes: r.data.indexed_bytes ?? prev?.indexed_bytes ?? 0,
          semantic_enabled: r.data.semantic_enabled ?? prev?.semantic_enabled ?? true,
        }));
      }
      toast.success(`Indexed ${r.data.indexed_files} files (${r.data.total_chunks} chunks) in ${r.data.elapsed_ms} ms.`);
      setSearchNonce((n) => n + 1);
    } catch {
      const msg = "Could not reindex workspace vault.";
      setReindexErr(msg);
      toast.error(msg);
    } finally {
      reindexLockRef.current = false;
      setReindexing(false);
    }
  };

  const runVaultAggregate = async (savePath?: string) => {
    if (aggLockRef.current || aggBusy) return;
    if (savePath !== undefined && !savePath.trim()) {
      setAggErr("Enter a filename to save the rollup CSV.");
      return;
    }
    const cleanGlob = aggGlob.trim() || "*.csv";
    const groupCols = aggGroupBy
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const targetCol = aggCol.trim() || "*";
    const alias =
      aggAlias.trim() ||
      `${aggFn}_${targetCol === "*" ? "rows" : targetCol.toLowerCase().replace(/[^a-z0-9]+/g, "_")}`;
    aggLockRef.current = true;
    setAggBusy(true);
    setAggErr("");
    try {
      const res = await aggregateVault({
        glob: cleanGlob,
        ...(aggSheet.trim() ? { sheet: aggSheet.trim() } : {}),
        ...(aggWhereCol.trim() && aggWhereVal.trim()
          ? { where: [{ col: aggWhereCol.trim(), op: aggWhereOp, value: aggWhereVal.trim() }] }
          : {}),
        ...(groupCols.length ? { group_by: groupCols } : {}),
        aggregates: [{ col: targetCol, fn: aggFn, as: alias }],
        sort: { col: alias, dir: "desc" },
        limit: 500,
        ...(scoped && activeEngagement
          ? { client_id: activeEngagement.client_id, engagement_id: activeEngagement.id }
          : {}),
        ...(savePath ? { save_as: savePath.trim() } : {}),
      });
      if (!res.ok) {
        const msg = res.error || "Multi-file rollup failed.";
        setAggErr(msg);
        toast.error(msg);
        return;
      }
      setAggResult(res.data);
      if (res.data.saved_path) {
        toast.success(`Saved ${res.data.saved_path} to workspace.`);
        onRefresh();
      }
    } catch {
      const msg = "Multi-file rollup failed.";
      setAggErr(msg);
      toast.error(msg);
    } finally {
      aggLockRef.current = false;
      setAggBusy(false);
    }
  };

  useEffect(() => {
    setPage(0);
  }, [query, extFilter, files.length, vaultHits.length, scoped, activeEngagement?.id]);

  const vaultHitsMap = useMemo(() => {
    const map = new Map<string, VaultFileHit>();
    for (const fh of vaultHits) {
      map.set(fh.path, fh);
    }
    return map;
  }, [vaultHits]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const matchesExt = (f: WorkspaceFile) => {
      const ext = (f.ext || "").toLowerCase();
      if (extFilter === "csv" && ext !== "csv") return false;
      if (extFilter === "md" && ext !== "md" && ext !== "markdown") return false;
      if (extFilter === "json" && ext !== "json") return false;
      if (extFilter === "other" && ["csv", "md", "markdown", "json"].includes(ext)) return false;
      return true;
    };
    const base = files.filter((f) => {
      if (scoped && activeEngagement && !isPathInEngagementScope(f.path, activeEngagement)) return false;
      if (!matchesExt(f)) return false;
      if (!q) return true;
      const ext = (f.ext || "").toLowerCase();
      return f.path.toLowerCase().includes(q) || ext.includes(q) || vaultHitsMap.has(f.path);
    });
    if (q.length >= 2 && vaultHits.length > 0) {
      const rankMap = new Map<string, number>();
      vaultHits.forEach((fh, idx) => rankMap.set(fh.path, idx));
      return [...base].sort((a, b) => {
        const ra = rankMap.get(a.path);
        const rb = rankMap.get(b.path);
        if (ra !== undefined && rb !== undefined) return ra - rb;
        if (ra !== undefined) return -1;
        if (rb !== undefined) return 1;
        return 0;
      });
    }
    return base;
  }, [files, query, extFilter, vaultHits, vaultHitsMap, scoped, activeEngagement]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / FILES_PAGE_SIZE));
  const safePage = Math.min(page, totalPages - 1);
  const visible = filtered.slice(safePage * FILES_PAGE_SIZE, (safePage + 1) * FILES_PAGE_SIZE);

  const ingestFiles = async (list: FileList | File[]) => {
    if (!list || !list.length) return;
    setErr("");
    if (onUploadMany) {
      const okCount = await onUploadMany(list);
      if (okCount === 0) setErr("Could not upload selected files.");
      return;
    }
    for (const file of Array.from(list)) {
      const body = await fileUploadBody(file);
      const ok = await onUpload(file.name, body.content, false, body.encoding);
      if (!ok) setErr(`Could not upload ${file.name}.`);
    }
  };

  const handleFilePick = async (ev: React.ChangeEvent<HTMLInputElement>) => {
    const picked = ev.target.files;
    if (picked && picked.length) await ingestFiles(picked);
    ev.target.value = "";
  };

  const handleCreate = async (ev: React.FormEvent) => {
    ev.preventDefault();
    const clean = newName.trim();
    if (!clean) {
      setErr("Give the file a name (for example expenses.csv or notes.md).");
      return;
    }
    setSaving(true);
    setErr("");
    const ok = await onUpload(clean, newContent);
    setSaving(false);
    if (!ok) {
      setErr("Could not save that file. Check the filename and try again.");
      return;
    }
    setNewName("");
    setNewContent("");
    setCreating(false);
  };

  return (
    <div
      id="workspace-files-panel"
      className="flex flex-col gap-3.5"
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setDragOver(false);
      }}
      onDrop={async (e) => {
        e.preventDefault();
        setDragOver(false);
        if (e.dataTransfer?.files?.length) await ingestFiles(e.dataTransfer.files);
      }}
    >
      <p className="text-sm text-ink-2">
        Real files in this person&apos;s sandboxed workspace. Drop multiple files here or upload spreadsheets, notes, and JSON.
      </p>

      <div
        className={cn(
          "flex flex-wrap items-center justify-between gap-2 rounded-card border border-dashed p-3",
          dragOver ? "border-blue bg-blue-soft" : "border-hairline bg-board/40"
        )}
      >
        <div className="flex flex-wrap items-center gap-2">
          <input
            ref={fileInputRef}
            id="workspace-file-input"
            type="file"
            multiple
            className="hidden"
            accept=".csv,.md,.txt,.json,.log,.tsv,.pdf,.docx,.xlsx,.pptx,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
            onChange={handleFilePick}
          />
          <Button size="sm" className="max-md:min-h-11" onClick={() => fileInputRef.current?.click()}>
            <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
            Upload files
          </Button>
          <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={() => setCreating((v) => !v)}>
            <FilePlus2 className="size-3.5" strokeWidth={1.5} aria-hidden />
            {creating ? "Cancel" : "New file"}
          </Button>
          <Button
            id="vault-aggregate-open-btn"
            type="button"
            size="sm"
            variant="outline"
            className="max-md:min-h-11"
            onClick={() => setAggOpen(true)}
          >
            <Layers className="size-3.5" strokeWidth={1.5} aria-hidden />
            Rollup
          </Button>
          <Button
            id="workspace-refresh-btn"
            type="button"
            size="icon-sm"
            variant="ghost"
            aria-label="Refresh files"
            className="max-md:min-h-11 max-md:min-w-11"
            onClick={onRefresh}
          >
            <RefreshCw className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
          </Button>
        </div>
        <span className="text-xs text-ink-2">
          {dragOver ? "Release to upload files" : "Drag & drop multiple files"}
        </span>
      </div>

      {creating && (
        <form onSubmit={handleCreate} className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/50 p-3.5">
          <div className="flex flex-col gap-1">
            <label htmlFor="new-file-name" className="text-xs font-medium text-ink">Filename</label>
            <Input
              id="new-file-name"
              placeholder="e.g. q4-expenses.csv or meeting-notes.md"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              autoComplete="off"
            />
          </div>
          <div className="flex flex-col gap-1">
            <label htmlFor="new-file-content" className="text-xs font-medium text-ink">Content</label>
            <Textarea
              id="new-file-content"
              rows={5}
              placeholder="Paste CSV rows, Markdown notes, or JSON here…"
              value={newContent}
              onChange={(e) => setNewContent(e.target.value)}
              className="font-mono text-xs"
            />
          </div>
          <div className="flex justify-end">
            <Button type="submit" size="sm" className="max-md:min-h-11" disabled={saving}>
              {saving ? "Saving…" : "Save to workspace"}
            </Button>
          </div>
        </form>
      )}

      {activeEngagement && (
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board/40 px-2.5 py-1.5 text-xs">
          <span className="flex min-w-0 items-center gap-1.5 text-ink-2">
            <Briefcase className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
            <span className="truncate">
              {[activeClient?.name, activeEngagement.name].filter(Boolean).join(" / ")}
            </span>
          </span>
          <Button
            id="workspace-engagement-scope-toggle"
            type="button"
            size="xs"
            variant={scoped ? "default" : "outline"}
            aria-pressed={scoped}
            className="max-md:min-h-11 shrink-0 text-xs"
            onClick={() => setScoped(!scoped)}
          >
            {scoped ? "Active Engagement Scope: On" : "Active Engagement Scope"}
          </Button>
        </div>
      )}

      {files.length > 0 && (
        <div className="flex flex-col gap-2">
          <InputGroup className="rounded-sm bg-board/50 [&_input]:max-md:min-h-11">
            <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
              <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              id="workspace-file-search"
              type="search"
              aria-label="Search workspace files"
              placeholder={`Search ${files.length} workspace ${files.length === 1 ? "file" : "files"}…`}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="text-sm max-md:min-h-11"
            />
            {query && (
              <InputGroupAddon align="inline-end" className="pr-1.5">
                <Button
                  type="button"
                  size="icon-xs"
                  variant="ghost"
                  className="max-md:min-h-11 max-md:min-w-11"
                  aria-label="Clear file search"
                  onClick={() => setQuery("")}
                >
                  <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                </Button>
              </InputGroupAddon>
            )}
          </InputGroup>

          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap gap-1" role="group" aria-label="Filter files by type">
              {(["all", "csv", "md", "json", "other"] as const).map((kind) => (
                <Button
                  key={kind}
                  type="button"
                  size="xs"
                  variant={extFilter === kind ? "default" : "outline"}
                  aria-pressed={extFilter === kind}
                  className="max-md:min-h-11 text-xs"
                  onClick={() => setExtFilter(kind)}
                >
                  {kind === "all" ? "All" : kind === "other" ? "Other" : kind.toUpperCase()}
                </Button>
              ))}
            </div>
            <span className="mono text-xs text-ink-2">
              {filtered.length === files.length
                ? `${files.length} total`
                : `${filtered.length} of ${files.length}`}
            </span>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-2">
            <SegmentedControl aria-label="Workspace vault search mode">
              {(["hybrid", "bm25", "exact"] as const).map((m) => (
                <SegmentedControlItem
                  key={m}
                  data-id={m}
                  aria-pressed={searchMode === m}
                  data-checked={searchMode === m ? "" : undefined}
                  className="min-h-11 px-2 text-xs md:min-h-7"
                  onClick={() => setSearchMode(m)}
                >
                  {m === "hybrid" ? "Hybrid" : m === "bm25" ? "BM25" : "Exact"}
                </SegmentedControlItem>
              ))}
            </SegmentedControl>
            <div className="flex flex-wrap items-center gap-1.5">
              <Badge id="workspace-vault-stats" variant="outline" size="sm" className="text-xs text-ink-2 tabular-nums">
                {indexStats
                  ? `${indexStats.total_chunks} chunks, ${indexStats.total_files} files`
                  : `${files.length} files`}
              </Badge>
              <Button
                id="workspace-vault-reindex-btn"
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11 gap-1 text-xs"
                disabled={reindexing}
                onClick={() => void handleReindexVault()}
              >
                <RefreshCw className={cn("size-3", reindexing && "animate-spin")} strokeWidth={1.5} aria-hidden />
                <span>{reindexing ? "Reindexing…" : "Reindex Vault"}</span>
              </Button>
            </div>
          </div>
        </div>
      )}

      {err && <p role="alert" className="text-sm text-destructive-foreground">{err}</p>}
      {reindexErr && (
        <div
          id="workspace-reindex-error"
          role="alert"
          className="flex items-center justify-between gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg"
        >
          <span>{reindexErr}</span>
          <Button
            type="button"
            size="xs"
            variant="ghost"
            aria-label="Dismiss reindex error"
            onClick={() => setReindexErr("")}
          >
            Dismiss
          </Button>
        </div>
      )}

      {!files.length ? (
        <p className="rounded-card border border-hairline bg-board/40 p-3.5 text-sm text-ink-2">
          No files in this workspace yet. Upload a CSV or Markdown file above, or ask your agent to create one.
        </p>
      ) : !filtered.length ? (
        <p className="rounded-card border border-hairline bg-board/40 p-3.5 text-sm text-ink-2">
          {vaultSearching
            ? "Searching indexed workspace…"
            : `No workspace files match "${query}".`}
        </p>
      ) : (
        <>
          <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
            {visible.map((f) => {
              const hit = query.trim().length >= 2 ? vaultHitsMap.get(f.path) : undefined;
              const chunks = hit?.chunks?.slice(0, 3) ?? [];
              return (
                <li key={f.path} className="flex flex-col gap-2 px-3 py-2.5">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex min-w-0 flex-1 items-center gap-2">
                      <FileGlyph name={f.path} />
                      <div className="min-w-0 flex-1">
                        <p className="mono truncate text-sm font-medium text-ink">{f.path}</p>
                        <p className="mono text-xs text-ink-2">
                          {formatBytes(f.size)}{f.mtime_ms > 0 ? `, ${relTime(f.mtime_ms)}` : ""}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <Button
                        size="sm"
                        variant="ghost"
                        className="max-md:min-h-11"
                        aria-label={`Open ${f.path}`}
                        onClick={() => {
                          if (!embedded) setIsOpen(false);
                          setTimeout(() => onOpenFile(f.path), embedded ? 0 : 40);
                        }}
                      >
                        Open
                      </Button>
                      <Button
                        size="icon-sm"
                        variant="ghost"
                        className="max-md:min-h-11 max-md:min-w-11"
                        aria-label={`Download ${f.path}`}
                        onClick={() => downloadWorkspaceFile(f.path)}
                      >
                        <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
                      </Button>
                    </div>
                  </div>
                  {chunks.length > 0 && (
                    <div className="flex flex-col gap-1.5 border-l-2 border-hairline pl-2.5">
                      {chunks.map((ch) => (
                        <button
                          key={ch.chunk_id}
                          type="button"
                          data-vault-jump={f.path}
                          data-line-start={ch.line_start}
                          onClick={() => {
                            const jumpTarget = buildVaultJumpTarget(f.path, ch);
                            if (!embedded) setIsOpen(false);
                            setTimeout(() => onOpenFile(jumpTarget), embedded ? 0 : 40);
                          }}
                          className="focus-ring flex min-h-11 w-full cursor-pointer flex-col items-start gap-1 rounded-sm bg-board/50 px-2.5 py-1.5 text-left text-xs text-ink outline-none hover:bg-board md:min-h-9"
                        >
                          <div className="flex w-full flex-wrap items-center gap-1">
                            {typeof ch.page === "number" && ch.page > 0 && (
                              <Badge variant="outline" size="sm" className="text-xs tabular-nums">
                                Page {ch.page}
                              </Badge>
                            )}
                            {ch.sheet && (
                              <Badge variant="outline" size="sm" className="text-xs">
                                #{ch.sheet}
                              </Badge>
                            )}
                            <Badge variant="outline" size="sm" className="text-xs tabular-nums">
                              L{ch.line_start}{ch.line_end > ch.line_start ? `-L${ch.line_end}` : ""}
                            </Badge>
                            {ch.heading && (
                              <span className="truncate text-xs font-medium text-ink">{ch.heading}</span>
                            )}
                          </div>
                          <span className="line-clamp-2 w-full text-xs leading-5 text-ink-2">
                            {ch.excerpt}
                          </span>
                        </button>
                      ))}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>

          {totalPages > 1 && (
            <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
              <span className="mono">
                Showing {safePage * FILES_PAGE_SIZE + 1} to {Math.min((safePage + 1) * FILES_PAGE_SIZE, filtered.length)} of {filtered.length}
              </span>
              <div className="flex items-center gap-1.5">
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={safePage === 0}
                  onClick={() => setPage((p) => Math.max(0, p - 1))}
                >
                  Prev
                </Button>
                <span className="mono px-1">
                  {safePage + 1} / {totalPages}
                </span>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={safePage >= totalPages - 1}
                  onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
                >
                  Next
                </Button>
              </div>
            </div>
          )}
        </>
      )}

      <Dialog
        open={aggOpen}
        onOpenChange={(next) => {
          setAggOpen(next);
          if (next) setAggErr("");
        }}
      >
        <DialogContent id="vault-aggregate-dialog" className="max-h-[88vh] max-w-3xl overflow-y-auto">
          <DialogHeader className="pr-10 text-left">
            <DialogTitle className="flex items-center gap-2 text-title font-medium text-ink">
              <Layers className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
              <span>Multi-file table rollup</span>
            </DialogTitle>
            <DialogDescription className="text-xs text-ink-2">
              Aggregate CSV, TSV, and workbook sheets across matching workspace files with exact decimal totals.
            </DialogDescription>
          </DialogHeader>

          <div className="flex flex-col gap-3">
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-glob" className="text-xs text-ink-2">
                  File glob
                </label>
                <Input
                  id="vault-agg-glob"
                  value={aggGlob}
                  onChange={(e) => setAggGlob(e.target.value)}
                  placeholder="e.g. *.csv or invoices/*.csv"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-sheet" className="text-xs text-ink-2">
                  Workbook sheet (optional)
                </label>
                <Input
                  id="vault-agg-sheet"
                  value={aggSheet}
                  onChange={(e) => setAggSheet(e.target.value)}
                  placeholder="All sheets"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-group-by" className="text-xs text-ink-2">
                  Group by columns
                </label>
                <Input
                  id="vault-agg-group-by"
                  value={aggGroupBy}
                  onChange={(e) => setAggGroupBy(e.target.value)}
                  placeholder="e.g. Vendor, Region"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-fn" className="text-xs text-ink-2">
                  Aggregate
                </label>
                <select
                  id="vault-agg-fn"
                  aria-label="Rollup aggregate function"
                  value={aggFn}
                  onChange={(e) => setAggFn(e.target.value as TableAggregateFn)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="sum">SUM</option>
                  <option value="avg">AVG</option>
                  <option value="min">MIN</option>
                  <option value="max">MAX</option>
                  <option value="count">COUNT</option>
                </select>
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-col" className="text-xs text-ink-2">
                  Target column
                </label>
                <Input
                  id="vault-agg-col"
                  value={aggCol}
                  onChange={(e) => setAggCol(e.target.value)}
                  placeholder="e.g. Amount"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-alias" className="text-xs text-ink-2">
                  Output column name
                </label>
                <Input
                  id="vault-agg-alias"
                  value={aggAlias}
                  onChange={(e) => setAggAlias(e.target.value)}
                  placeholder="e.g. total_amount"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-where-col" className="text-xs text-ink-2">
                  Filter column (optional)
                </label>
                <Input
                  id="vault-agg-where-col"
                  value={aggWhereCol}
                  onChange={(e) => setAggWhereCol(e.target.value)}
                  placeholder="e.g. Status"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-where-op" className="text-xs text-ink-2">
                  Filter operator
                </label>
                <select
                  id="vault-agg-where-op"
                  aria-label="Rollup filter operator"
                  value={aggWhereOp}
                  onChange={(e) => setAggWhereOp(e.target.value as TableWhereOp)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="contains">contains</option>
                  <option value="eq">equals (=)</option>
                  <option value="neq">not equal (!=)</option>
                  <option value="gt">greater (&gt;)</option>
                  <option value="gte">at least (&gt;=)</option>
                  <option value="lt">less (&lt;)</option>
                  <option value="lte">at most (&lt;=)</option>
                </select>
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="vault-agg-where-val" className="text-xs text-ink-2">
                  Filter value
                </label>
                <Input
                  id="vault-agg-where-val"
                  value={aggWhereVal}
                  onChange={(e) => setAggWhereVal(e.target.value)}
                  placeholder="Match value…"
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-2.5">
              <Button
                id="vault-agg-run-btn"
                type="button"
                size="sm"
                className="max-md:min-h-11"
                disabled={aggBusy}
                onClick={() => void runVaultAggregate()}
              >
                <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                {aggBusy ? "Running…" : "Run Rollup"}
              </Button>
              <div className="flex flex-wrap items-center gap-2">
                <Input
                  id="vault-agg-save-as"
                  aria-label="Save rollup CSV path"
                  value={aggSaveAs}
                  onChange={(e) => setAggSaveAs(e.target.value)}
                  placeholder="vault-rollup.csv"
                  className="h-9 w-44 text-xs max-md:min-h-11"
                />
                <Button
                  id="vault-agg-save-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={aggBusy || !aggSaveAs.trim()}
                  onClick={() => void runVaultAggregate(aggSaveAs.trim())}
                >
                  <Save className="size-3.5" strokeWidth={1.5} aria-hidden />
                  Save Rollup CSV
                </Button>
              </div>
            </div>

            {aggErr && (
              <div
                id="vault-agg-error"
                role="alert"
                className="flex items-center justify-between gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg"
              >
                <span>{aggErr}</span>
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  aria-label="Dismiss rollup error"
                  onClick={() => setAggErr("")}
                >
                  Dismiss
                </Button>
              </div>
            )}

            {aggResult && (
              <section
                id="vault-agg-result"
                aria-label="Multi-file rollup results"
                className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/30 p-3"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex flex-wrap items-center gap-2 text-xs text-ink-2">
                    <span className="font-medium text-ink">Rollup result</span>
                    <Badge variant="outline" size="sm" className="tabular-nums">
                      {aggResult.row_count} rows from {aggResult.source_files.length} files
                    </Badge>
                    {aggResult.saved_path && (
                      <Badge variant="outline" size="sm" className="status-finished border-hairline">
                        Saved: {aggResult.saved_path}
                      </Badge>
                    )}
                  </div>
                  {aggResult.saved_path && (
                    <Button
                      id="vault-agg-open-saved"
                      type="button"
                      size="xs"
                      variant="outline"
                      className="max-md:min-h-11"
                      onClick={() => {
                        const saved = aggResult.saved_path!;
                        setAggOpen(false);
                        if (!embedded) setIsOpen(false);
                        setTimeout(() => onOpenFile(saved), 40);
                      }}
                    >
                      Open {aggResult.saved_path}
                    </Button>
                  )}
                </div>

                {aggResult.source_files.length > 0 && (
                  <p className="text-xs text-ink-2">
                    Sources: {aggResult.source_files.join(", ")}
                  </p>
                )}

                <div className="max-h-60 overflow-auto rounded-sm border border-hairline bg-surface">
                  <table className="w-full border-collapse text-left text-xs">
                    <thead className="sticky top-0 border-b border-hairline bg-board text-ink">
                      <tr>
                        {aggResult.columns.map((col, idx) => (
                          <th key={idx} className="whitespace-nowrap px-3 py-2 font-medium">
                            {col}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-hairline">
                      {aggResult.rows.length === 0 ? (
                        <tr>
                          <td colSpan={Math.max(1, aggResult.columns.length)} className="px-3 py-4 text-center text-ink-2">
                            No rows matched this rollup query.
                          </td>
                        </tr>
                      ) : (
                        aggResult.rows.map((r, ri) => (
                          <tr key={ri} className="hover:bg-board/40">
                            {aggResult.columns.map((_, ci) => {
                              const val = r[ci] ?? "";
                              const numeric = /^-?\$?\d[\d,]*(?:\.\d+)?%?$/.test(val.trim());
                              return (
                                <td
                                  key={ci}
                                  className={numeric ? "mono whitespace-nowrap px-3 py-1.5 text-right text-ink" : "px-3 py-1.5 text-ink"}
                                >
                                  {val}
                                </td>
                              );
                            })}
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>
                </div>
              </section>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
