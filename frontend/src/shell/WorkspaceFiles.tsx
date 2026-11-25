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
  ChevronDown,
  ChevronRight,
  Copy,
  Download,
  FileDown,
  FilePlus2,
  FileSpreadsheet,
  FileText,
  FolderOpen,
  History,
  MessageSquarePlus,
  RefreshCw,
  RotateCcw,
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
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { FileGlyph } from "@/chat/Entry";
import {
  call,
  downloadWorkspaceFile,
  exportTarget,
  fileUploadBody,
  hasV070Api,
  type FileDiffPayload,
  type FileRevision,
  type Status,
  type WorkspaceFile,
  type WorkspaceFileContent,
} from "@/lib/api";
import { computeLineDiff, normalizeServerDiff } from "@/lib/diff";
import { exportReceiptDownload } from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { detectWorkflowForFile, relTime } from "@/lib/words";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board";
const FILES_PAGE_SIZE = 25;
const CSV_PAGE_SIZE = 100;

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} kB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

/** RFC-4180 compliant CSV parser handling quoted fields, commas inside quotes, and doubled quotes. */
export function parseCsv(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = "";
  let inQuotes = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          cell += '"';
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        cell += ch;
      }
    } else if (ch === '"') {
      inQuotes = true;
    } else if (ch === ",") {
      row.push(cell);
      cell = "";
    } else if (ch === "\r") {
      if (text[i + 1] === "\n") i++;
      row.push(cell);
      if (row.some((c) => c.length > 0)) rows.push(row);
      row = [];
      cell = "";
    } else if (ch === "\n") {
      row.push(cell);
      if (row.some((c) => c.length > 0)) rows.push(row);
      row = [];
      cell = "";
    } else {
      cell += ch;
    }
  }
  row.push(cell);
  if (row.some((c) => c.length > 0)) rows.push(row);
  return rows;
}

function renderInlineFormatting(line: string): React.ReactNode {
  const tokens = line.split(/(`[^`]+`|\*\*[^*]+\*\*)/g);
  if (tokens.length === 1) return line;
  return tokens.map((tok, i) => {
    if (tok.startsWith("`") && tok.endsWith("`") && tok.length > 2) {
      return (
        <code key={i} className="mono rounded-chip bg-board px-1.5 py-0.5 text-xs text-ink">
          {tok.slice(1, -1)}
        </code>
      );
    }
    if (tok.startsWith("**") && tok.endsWith("**") && tok.length > 4) {
      return (
        <strong key={i} className="font-medium text-ink">
          {tok.slice(2, -2)}
        </strong>
      );
    }
    return <span key={i}>{tok}</span>;
  });
}

function headingSlug(text: string, idx: number): string {
  const clean = text.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 32);
  return `md-sec-${idx}-${clean || "h"}`;
}

function MarkdownView({ content }: { content: string }) {
  const parsed = useMemo(() => {
    const lines = content.replace(/\r\n/g, "\n").split("\n");
    const blocks: React.ReactNode[] = [];
    const headings: { id: string; level: number; text: string }[] = [];
    let list: string[] = [];
    let codeLines: string[] | null = null;
    let tableRows: string[][] = [];

    const flushList = (key: number) => {
      if (!list.length) return;
      blocks.push(
        <ul key={"ul-" + key} className="flex list-disc flex-col gap-1 pl-5 text-sm leading-6 text-ink">
          {list.map((item, idx) => <li key={idx}>{renderInlineFormatting(item)}</li>)}
        </ul>
      );
      list = [];
    };

    const flushTable = (key: number) => {
      if (!tableRows.length) return;
      const head = tableRows[0];
      const body = tableRows.slice(1).filter((r) => !r.every((c) => /^[-:\s]+$/.test(c)));
      blocks.push(
        <div key={"tbl-" + key} className="overflow-x-auto rounded-sm border border-hairline bg-surface">
          <table className="w-full border-collapse text-left text-xs">
            <thead className="border-b border-hairline bg-board text-ink">
              <tr>
                {head.map((c, ci) => (
                  <th key={ci} className="whitespace-nowrap px-3 py-2 font-medium">{c}</th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-hairline">
              {body.map((r, ri) => (
                <tr key={ri}>
                  {head.map((_, ci) => (
                    <td key={ci} className="px-3 py-2 text-ink">{renderInlineFormatting(r[ci] ?? "")}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
      tableRows = [];
    };

    lines.forEach((line, idx) => {
      const t = line.trim();
      if (t.startsWith("```")) {
        flushList(idx);
        flushTable(idx);
        if (codeLines === null) {
          codeLines = [];
        } else {
          blocks.push(
            <pre key={"code-" + idx} className="overflow-x-auto rounded-sm border border-hairline bg-board p-3 font-mono text-xs leading-5 text-ink">
              {codeLines.join("\n")}
            </pre>
          );
          codeLines = null;
        }
        return;
      }
      if (codeLines !== null) {
        codeLines.push(line);
        return;
      }
      if (/^\|.+\|$/.test(t)) {
        flushList(idx);
        tableRows.push(t.slice(1, -1).split("|").map((c) => c.trim()));
        return;
      } else {
        flushTable(idx);
      }
      if (!t) {
        flushList(idx);
        return;
      }
      if (t.startsWith("### ")) {
        flushList(idx);
        const text = t.slice(4);
        const id = headingSlug(text, idx);
        headings.push({ id, level: 3, text });
        blocks.push(<h4 key={"h3-" + idx} id={id} className="scroll-mt-3 pt-2 text-sm font-semibold text-ink">{renderInlineFormatting(text)}</h4>);
      } else if (t.startsWith("## ")) {
        flushList(idx);
        const text = t.slice(3);
        const id = headingSlug(text, idx);
        headings.push({ id, level: 2, text });
        blocks.push(<h3 key={"h2-" + idx} id={id} className="scroll-mt-3 pt-3 text-base font-medium text-ink">{renderInlineFormatting(text)}</h3>);
      } else if (t.startsWith("# ")) {
        flushList(idx);
        const text = t.slice(2);
        const id = headingSlug(text, idx);
        headings.push({ id, level: 1, text });
        blocks.push(<h2 key={"h1-" + idx} id={id} className="scroll-mt-3 text-title font-medium text-ink">{renderInlineFormatting(text)}</h2>);
      } else if (/^[-*]\s+/.test(t)) {
        list.push(t.replace(/^[-*]\s+/, ""));
      } else if (/^\d+\.\s+/.test(t)) {
        list.push(t.replace(/^\d+\.\s+/, ""));
      } else if (/^---+$/.test(t)) {
        flushList(idx);
        blocks.push(<hr key={"hr-" + idx} className="my-2 border-hairline" />);
      } else {
        flushList(idx);
        blocks.push(<p key={"p-" + idx} className="text-sm leading-6 text-ink">{renderInlineFormatting(t)}</p>);
      }
    });
    flushList(lines.length + 1);
    flushTable(lines.length + 2);
    if (codeLines !== null) {
      blocks.push(
        <pre key="code-tail" className="overflow-x-auto rounded-sm border border-hairline bg-board p-3 font-mono text-xs leading-5 text-ink">
          {(codeLines as string[]).join("\n")}
        </pre>
      );
    }
    return { blocks, headings };
  }, [content]);

  const jumpTo = (id: string) => {
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: "auto", block: "start" });
  };

  return (
    <div className="flex flex-col gap-3">
      {parsed.headings.length > 1 && (
        <nav aria-label="Markdown section jump" className="flex flex-wrap items-center gap-1.5 rounded-sm border border-hairline bg-surface p-2">
          <span className="px-1 text-xs font-medium text-ink-2">Jump to section:</span>
          {parsed.headings.map((h) => (
            <Button
              key={h.id}
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 max-w-[220px] truncate text-xs"
              onClick={() => jumpTo(h.id)}
            >
              {h.text}
            </Button>
          ))}
        </nav>
      )}
      <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/30 p-4">
        {parsed.blocks}
      </div>
    </div>
  );
}

function parseSortableNumber(raw: string): number | null {
  const cleaned = raw.trim().replace(/[$,%\s]/g, "");
  if (!cleaned || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) return null;
  const n = Number(cleaned);
  return Number.isFinite(n) ? n : null;
}

function CsvTableView({ content }: { content: string }) {
  const rows = useMemo(() => parseCsv(content), [content]);
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
            Rows {safePage * CSV_PAGE_SIZE + 1}–{Math.min((safePage + 1) * CSV_PAGE_SIZE, processed.length)} of {processed.length}
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
  const [data, setData] = useState<WorkspaceFileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");
  const [exporting, setExporting] = useState(false);
  const [tab, setTab] = useState<"preview" | "history">("preview");
  const [revisions, setRevisions] = useState<FileRevision[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyErr, setHistoryErr] = useState("");

  const loadContent = useCallback(async (path: string, sheet?: string) => {
    setLoading(true);
    setErr("");
    const r = await call<WorkspaceFileContent>(
      "/api/files/content?path=" + encodeURIComponent(path) + (sheet ? "&sheet=" + encodeURIComponent(sheet) : "")
    );
    setLoading(false);
    if (!r.ok) {
      setData(null);
      setErr(r.status === 404 ? `"${path}" is no longer in this workspace (it may have been deleted or moved).` : r.error || "Could not open this file.");
      return;
    }
    setData(r.data);
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

  useEffect(() => {
    if (!filePath) {
      setData(null);
      setErr("");
      setTab("preview");
      setRevisions([]);
      setHistoryErr("");
      return;
    }
    setTab("preview");
    void loadContent(filePath);
    void loadHistory(filePath);
  }, [filePath, loadContent, loadHistory]);

  const ext = (data?.ext || filePath?.split(".").pop() || "").toLowerCase();

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
    if (!filePath) return;
    const ok = await downloadWorkspaceFile(filePath);
    if (ok) toast.success(`Downloaded ${filePath.split("/").pop()}.`);
    else toast.error("Could not download this file.");
  };

  const target = filePath ? exportTarget(filePath) : null;
  const suggestedWorkflow = filePath ? detectWorkflowForFile(filePath) : null;
  const [savingAs, setSavingAs] = useState(false);
  const handleSaveAs = async () => {
    if (!filePath || !target) return;
    setSavingAs(true);
    const r = await call<{ path: string; size: number }>("/api/files/export", {
      method: "POST",
      body: { path: filePath, format: target },
    });
    setSavingAs(false);
    if (!r.ok) {
      toast.error(r.error ? "Could not save a copy: " + r.error : "Could not save a copy.");
      return;
    }
    toast.success(`Saved ${r.data.path} to your workspace.`);
    await downloadWorkspaceFile(r.data.path);
    onRestored?.();
  };

  const handleReceiptExport = async (format: "md" | "json") => {
    if (!filePath || !data) return;
    setExporting(true);
    try {
      const res = await exportReceiptDownload(
        {
          id: `file-${filePath}`,
          task: `Workspace file receipt: ${filePath}`,
          person: personName || "Active employee",
          status: "verified",
          summary: `Verified workspace file (${formatBytes(data.size)}, ${ext || "text"}).`,
          ts: data.mtime_ms || Date.now(),
          files: [filePath],
          filePath,
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
    <Dialog open={!!filePath} onOpenChange={(open) => { if (!open) onClose(); }}>
      <DialogContent id="file-viewer-dialog" className="max-h-[88vh] max-w-3xl overflow-hidden">
        <DialogHeader className="pr-10 text-left">
          <DialogTitle className="flex min-w-0 items-center gap-2 text-title font-medium text-ink">
            {filePath && <FileGlyph name={filePath} className="size-4 shrink-0" />}
            <span className="mono min-w-0 truncate">{filePath ?? ""}</span>
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
              onClick={() => setTab("preview")}
            >
              Preview
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="history"
              aria-pressed={tab === "history"}
              data-checked={tab === "history" ? "" : undefined}
              className="min-h-11 gap-1.5 px-3 md:min-h-8"
              onClick={() => {
                setTab("history");
                if (filePath) void loadHistory(filePath);
              }}
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
            <Button size="sm" variant="outline" className="max-md:min-h-11" disabled={!filePath || !!err} onClick={handleDownload}>
              <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
              Download
            </Button>
            {target && data && !err && (
              <Button id="file-save-as" size="sm" variant="outline" className="max-md:min-h-11" disabled={savingAs} onClick={handleSaveAs}>
                {target === "docx" ? (
                  <FileText className="size-3.5" strokeWidth={1.5} aria-hidden />
                ) : (
                  <FileSpreadsheet className="size-3.5" strokeWidth={1.5} aria-hidden />
                )}
                {savingAs ? "Saving…" : target === "docx" ? "Save as Word" : "Save as Excel"}
              </Button>
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
                  onRunWorkflow(suggestedWorkflow.prompt);
                  onClose();
                }}
              >
                <Sparkles className="size-3.5" strokeWidth={1.5} aria-hidden />
                {suggestedWorkflow.label}
              </Button>
            )}
            {filePath && onAskAbout && !err && (
              <Button
                size="sm"
                variant="ghost"
                className="max-md:min-h-11"
                onClick={() => {
                  onAskAbout(filePath);
                  onClose();
                }}
              >
                <MessageSquarePlus className="size-3.5" strokeWidth={1.5} aria-hidden />
                Ask agent
              </Button>
            )}
          </div>
        </div>

        <div id="file-viewer-body" className="max-h-[58vh] min-h-24 overflow-y-auto pr-1">
          {tab === "history" && filePath ? (
            <FileHistoryDiffView
              filePath={filePath}
              currentContent={data?.content ?? ""}
              status={status}
              revisions={revisions}
              loading={historyLoading}
              err={historyErr}
              onReloadHistory={() => void loadHistory(filePath)}
              onRestored={async () => {
                await loadContent(filePath);
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
                    {data.extracted.pages && data.extracted.pages > 1 ? ` (${data.extracted.pages} pages)` : ""} — your agent and every workflow read it the same way.
                  </span>
                  {(data.extracted.sheets?.length ?? 0) > 1 && filePath && (
                    <SegmentedControl aria-label="Workbook sheet">
                      {data.extracted.sheets!.slice(0, 8).map((sh) => (
                        <SegmentedControlItem
                          key={sh}
                          data-id={sh}
                          aria-pressed={data.extracted?.sheet === sh}
                          data-checked={data.extracted?.sheet === sh ? "" : undefined}
                          className="min-h-11 px-3 md:min-h-8"
                          onClick={() => void loadContent(filePath, sh)}
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
}: {
  files: WorkspaceFile[];
  onRefresh: () => void;
  onOpenFile: (path: string) => void;
  onUpload: (path: string, content: string, silent?: boolean, encoding?: "base64") => Promise<boolean>;
  onUploadMany?: (picked: FileList | File[]) => Promise<number>;
  embedded?: boolean;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newContent, setNewContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState("");
  const [query, setQuery] = useState("");
  const [extFilter, setExtFilter] = useState<"all" | "csv" | "md" | "json" | "other">("all");
  const [page, setPage] = useState(0);
  const [dragOver, setDragOver] = useState(false);

  useEffect(() => {
    if (embedded || isOpen) onRefresh();
  }, [embedded, isOpen, onRefresh]);

  useEffect(() => {
    setPage(0);
  }, [query, extFilter, files.length]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return files.filter((f) => {
      const ext = (f.ext || "").toLowerCase();
      if (extFilter === "csv" && ext !== "csv") return false;
      if (extFilter === "md" && ext !== "md" && ext !== "markdown") return false;
      if (extFilter === "json" && ext !== "json") return false;
      if (extFilter === "other" && ["csv", "md", "markdown", "json"].includes(ext)) return false;
      if (!q) return true;
      return f.path.toLowerCase().includes(q) || ext.includes(q);
    });
  }, [files, query, extFilter]);

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
            accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
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
                  className="max-md:min-h-11 uppercase text-xs"
                  onClick={() => setExtFilter(kind)}
                >
                  {kind === "all" ? "All" : kind}
                </Button>
              ))}
            </div>
            <span className="mono text-xs text-ink-2">
              {filtered.length === files.length
                ? `${files.length} total`
                : `${filtered.length} of ${files.length}`}
            </span>
          </div>
        </div>
      )}

      {err && <p role="alert" className="text-sm text-destructive-foreground">{err}</p>}

      {!files.length ? (
        <p className="rounded-card border border-hairline bg-board/40 p-3.5 text-sm text-ink-2">
          No files in this workspace yet. Upload a CSV or Markdown file above, or ask your agent to create one.
        </p>
      ) : !filtered.length ? (
        <p className="rounded-card border border-hairline bg-board/40 p-3.5 text-sm text-ink-2">
          No workspace files match &ldquo;{query}&rdquo;.
        </p>
      ) : (
        <>
          <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
            {visible.map((f) => (
              <li key={f.path} className="flex flex-wrap items-center justify-between gap-2 px-3 py-2.5">
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
              </li>
            ))}
          </ul>

          {totalPages > 1 && (
            <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
              <span className="mono">
                Showing {safePage * FILES_PAGE_SIZE + 1}–{Math.min((safePage + 1) * FILES_PAGE_SIZE, filtered.length)} of {filtered.length}
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
    </div>
  );
}
