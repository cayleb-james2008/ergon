// Interactive Deliverable Studio (v0.30.0):
// - Split Markdown / text editor with live preview, Cmd/Ctrl+S save shortcut,
//   dirty indicator, and 409 optimistic-lock conflict resolution banner.
// - Inline editable CSV/TSV grid (cell edit, add/delete row, column sort, Save Changes)
//   plus deterministic Table Query / Group-By bar (Where, Group By, Aggregate, Run Query, Save query as CSV).
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Columns2,
  Eye,
  FileSpreadsheet,
  Pencil,
  Play,
  Plus,
  RefreshCw,
  RotateCcw,
  Save,
  Trash2,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Kbd } from "@/components/ui/kbd";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Textarea } from "@/components/ui/textarea";
import {
  queryTable,
  saveFileContent,
  type TableAggregateFn,
  type TableQueryResult,
  type TableWhereOp,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export type DelimitedInspection = {
  rows: string[][];
  unclosedQuote: boolean;
  hasNullByte: boolean;
  raggedRows: number;
  maxCols: number;
  error: string | null;
};

/** Parse delimited text and surface structural issues (unclosed quotes, NUL bytes, or ragged extra columns). */
export function inspectDelimited(text: string, delimiter = ","): DelimitedInspection {
  const hasNullByte = text.includes("\0");
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
    } else if (ch === delimiter) {
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

  const headerLen = rows.length > 0 ? rows[0].length : 0;
  let maxCols = headerLen;
  let raggedRows = 0;
  for (let r = 1; r < rows.length; r++) {
    const len = rows[r].length;
    if (len > maxCols) maxCols = len;
    if (len > headerLen) raggedRows += 1;
  }

  let error: string | null = null;
  if (hasNullByte) {
    error = "This file contains binary NUL bytes and cannot be safely edited in the spreadsheet grid.";
  } else if (inQuotes) {
    error = "Unclosed quotation mark detected in delimited data. Switch to raw text to repair quotes before saving.";
  } else if (raggedRows > 0) {
    error = `${raggedRows} ${raggedRows === 1 ? "row has" : "rows have"} more columns than the header (${headerLen} header cols vs up to ${maxCols} data cols). Grid edits are paused to prevent dropping columns.`;
  }

  return { rows, unclosedQuote: inQuotes, hasNullByte, raggedRows, maxCols, error };
}

/** RFC-4180 compliant delimited parser handling quoted fields and doubled quotes. */
export function parseDelimited(text: string, delimiter = ","): string[][] {
  return inspectDelimited(text, delimiter).rows;
}

export function parseCsv(text: string): string[][] {
  return parseDelimited(text, ",");
}

export function serializeDelimited(rows: string[][], delimiter = ","): string {
  const escapeCell = (val: string) => {
    const s = String(val ?? "");
    if (s.includes('"') || s.includes(delimiter) || s.includes("\n") || s.includes("\r")) {
      return `"${s.replace(/"/g, '""')}"`;
    }
    return s;
  };
  return rows.map((r) => r.map(escapeCell).join(delimiter)).join("\n") + (rows.length ? "\n" : "");
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

export function MarkdownView({ content }: { content: string }) {
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
          {list.map((item, idx) => (
            <li key={idx}>{renderInlineFormatting(item)}</li>
          ))}
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
                  <th key={ci} className="whitespace-nowrap px-3 py-2 font-medium">
                    {c}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-hairline">
              {body.map((r, ri) => (
                <tr key={ri}>
                  {head.map((_, ci) => (
                    <td key={ci} className="px-3 py-2 text-ink">
                      {renderInlineFormatting(r[ci] ?? "")}
                    </td>
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
            <pre
              key={"code-" + idx}
              className="overflow-x-auto rounded-sm border border-hairline bg-board p-3 font-mono text-xs leading-5 text-ink"
            >
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
        blocks.push(
          <h4 key={"h3-" + idx} id={id} className="scroll-mt-3 pt-2 text-sm font-semibold text-ink">
            {renderInlineFormatting(text)}
          </h4>
        );
      } else if (t.startsWith("## ")) {
        flushList(idx);
        const text = t.slice(3);
        const id = headingSlug(text, idx);
        headings.push({ id, level: 2, text });
        blocks.push(
          <h3 key={"h2-" + idx} id={id} className="scroll-mt-3 pt-3 text-base font-medium text-ink">
            {renderInlineFormatting(text)}
          </h3>
        );
      } else if (t.startsWith("# ")) {
        flushList(idx);
        const text = t.slice(2);
        const id = headingSlug(text, idx);
        headings.push({ id, level: 1, text });
        blocks.push(
          <h2 key={"h1-" + idx} id={id} className="scroll-mt-3 text-title font-medium text-ink">
            {renderInlineFormatting(text)}
          </h2>
        );
      } else if (/^[-*]\s+/.test(t)) {
        list.push(t.replace(/^[-*]\s+/, ""));
      } else if (/^\d+\.\s+/.test(t)) {
        list.push(t.replace(/^\d+\.\s+/, ""));
      } else if (/^---+$/.test(t)) {
        flushList(idx);
        blocks.push(<hr key={"hr-" + idx} className="my-2 border-hairline" />);
      } else {
        flushList(idx);
        blocks.push(
          <p key={"p-" + idx} className="text-sm leading-6 text-ink">
            {renderInlineFormatting(t)}
          </p>
        );
      }
    });
    flushList(lines.length + 1);
    flushTable(lines.length + 2);
    if (codeLines !== null) {
      blocks.push(
        <pre
          key="code-tail"
          className="overflow-x-auto rounded-sm border border-hairline bg-board p-3 font-mono text-xs leading-5 text-ink"
        >
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
        <nav
          aria-label="Markdown section jump"
          className="flex flex-wrap items-center gap-1.5 rounded-sm border border-hairline bg-surface p-2"
        >
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
      <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/30 p-4">{parsed.blocks}</div>
    </div>
  );
}

export function parseSortableNumber(raw: string): number | null {
  const cleaned = raw.trim().replace(/[$,%\s]/g, "");
  if (!cleaned || !/^-?\d+(?:\.\d+)?$/.test(cleaned)) return null;
  const n = Number(cleaned);
  return Number.isFinite(n) ? n : null;
}

/** Clamp a 1-based line (and optional end line) to valid [1, totalLines] bounds. */
export function clampLineBounds(
  totalLines: number,
  line?: number,
  endLine?: number
): { line?: number; endLine?: number } {
  if (typeof line !== "number" || !Number.isFinite(line) || totalLines <= 0) {
    return { line: undefined, endLine: undefined };
  }
  const safeTotal = Math.max(1, Math.floor(totalLines));
  const clampedStart = Math.max(1, Math.min(safeTotal, Math.floor(line)));
  const clampedEnd =
    typeof endLine === "number" && Number.isFinite(endLine)
      ? Math.max(clampedStart, Math.min(safeTotal, Math.floor(endLine)))
      : undefined;
  return { line: clampedStart, endLine: clampedEnd };
}

export type FileEditorProps = {
  filePath: string;
  initialContent: string;
  ext: string;
  initialSha256?: string;
  isExtracted?: boolean;
  extractedFormat?: "markdown" | "csv";
  extractedKind?: string;
  sheet?: string | null;
  jumpToLine?: number;
  onSaved: (res: { path: string; content: string; size: number; mtime_ms: number; sha256: string }) => void;
  onReloadFromServer: () => Promise<boolean | void>;
  onDirtyChange?: (dirty: boolean) => void;
  onOpenSavedFile?: (path: string) => void;
  onWorkspaceMutated?: () => void;
};

export function FileEditor({
  filePath,
  initialContent,
  ext,
  initialSha256,
  isExtracted = false,
  extractedFormat,
  extractedKind,
  sheet,
  jumpToLine,
  onSaved,
  onReloadFromServer,
  onDirtyChange,
  onOpenSavedFile,
  onWorkspaceMutated,
}: FileEditorProps) {
  const normExt = ext.toLowerCase();
  const isTabular = normExt === "csv" || normExt === "tsv" || extractedFormat === "csv";
  const delimiter = normExt === "tsv" ? "\t" : ",";
  const isMarkdown = normExt === "md" || normExt === "markdown" || extractedFormat === "markdown";

  const [savedContent, setSavedContent] = useState(initialContent);
  const [sha256, setSha256] = useState<string | undefined>(initialSha256);
  const [saving, setSaving] = useState(false);
  const [reloading, setReloading] = useState(false);
  const [conflictSha, setConflictSha] = useState<string | null>(null);
  const [saveErr, setSaveErr] = useState("");
  const saveLockRef = useRef(false);
  const queryLockRef = useRef(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const prevFilePathRef = useRef(filePath);

  // Text / Markdown editor state
  const [textDraft, setTextDraft] = useState(initialContent);
  const [viewMode, setViewMode] = useState<"split" | "write" | "preview">("split");
  const [rawCsvMode, setRawCsvMode] = useState(false);
  const [csvNormalized, setCsvNormalized] = useState(false);

  // CSV / TSV grid state
  const csvInspection = useMemo(() => {
    if (!isTabular) return null;
    return inspectDelimited(savedContent, delimiter);
  }, [savedContent, delimiter, isTabular]);

  const parsedInitial = useMemo(() => {
    if (!isTabular) return { header: [] as string[], body: [] as string[][] };
    const raw = inspectDelimited(initialContent, delimiter).rows;
    if (!raw.length) return { header: ["Column 1"], body: [] as string[][] };
    const header = raw[0];
    const body = raw.slice(1).map((r) => header.map((_, i) => r[i] ?? ""));
    return { header, body };
  }, [initialContent, delimiter, isTabular]);

  const [tableHeader, setTableHeader] = useState<string[]>(parsedInitial.header);
  const [tableRows, setTableRows] = useState<string[][]>(parsedInitial.body);
  const [sortCol, setSortCol] = useState<number | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  // Table Query state
  const [whereCol, setWhereCol] = useState<string>("");
  const [whereOp, setWhereOp] = useState<TableWhereOp>("contains");
  const [whereVal, setWhereVal] = useState<string>("");
  const [groupCol, setGroupCol] = useState<string>("");
  const [aggFn, setAggFn] = useState<"" | TableAggregateFn>("");
  const [aggCol, setAggCol] = useState<string>(parsedInitial.header[1] ?? parsedInitial.header[0] ?? "");
  const [querySortDir, setQuerySortDir] = useState<"asc" | "desc">("desc");
  const [queryBusy, setQueryBusy] = useState(false);
  const [queryErr, setQueryErr] = useState("");
  const [queryResult, setQueryResult] = useState<TableQueryResult | null>(null);
  const stem = useMemo(() => filePath.replace(/\.[^.]+$/, ""), [filePath]);
  const [saveQueryName, setSaveQueryName] = useState<string>(`${stem}-query.csv`);

  const gridBlockedByMalformed = Boolean(isTabular && csvInspection?.error && !rawCsvMode && !csvNormalized);

  const currentSerialized = useMemo(() => {
    if (!isTabular || rawCsvMode) return textDraft;
    return serializeDelimited([tableHeader, ...tableRows], delimiter);
  }, [isTabular, rawCsvMode, textDraft, tableHeader, tableRows, delimiter]);

  const gridBaseline = useMemo(() => {
    if (!isTabular) return savedContent;
    const raw = inspectDelimited(savedContent, delimiter).rows;
    if (!raw.length) return serializeDelimited([["Column 1"]], delimiter);
    const hdr = raw[0];
    const body = raw.slice(1).map((r) => hdr.map((_, i) => r[i] ?? ""));
    return serializeDelimited([hdr, ...body], delimiter);
  }, [isTabular, savedContent, delimiter]);

  const dirty = !isExtracted && (
    !isTabular || rawCsvMode
      ? currentSerialized !== savedContent
      : currentSerialized !== gridBaseline
  );

  // Sync baseline when filePath or server content refreshes (never silently discard unsaved edits on same path)
  useEffect(() => {
    const pathChanged = prevFilePathRef.current !== filePath;
    prevFilePathRef.current = filePath;
    if (!pathChanged && dirty && initialSha256 && initialSha256 !== sha256) {
      setConflictSha(initialSha256);
      setSaveErr("File changed in the workspace while you were editing.");
      return;
    }
    setSavedContent(initialContent);
    setTextDraft(initialContent);
    setSha256(initialSha256);
    setConflictSha(null);
    setSaveErr("");
    setQueryResult(null);
    setQueryErr("");
    setRawCsvMode(false);
    setCsvNormalized(false);
    setSaveQueryName(`${filePath.replace(/\.[^.]+$/, "")}-query.csv`);
    if (isTabular) {
      const raw = inspectDelimited(initialContent, delimiter).rows;
      const hdr = raw.length ? raw[0] : ["Column 1"];
      const body = raw.slice(1).map((r) => hdr.map((_, i) => r[i] ?? ""));
      setTableHeader(hdr);
      setTableRows(body);
      setSortCol(null);
      setAggCol((prev) => (hdr.includes(prev) ? prev : (hdr[1] ?? hdr[0] ?? "")));
      setWhereCol((prev) => (hdr.includes(prev) ? prev : ""));
      setGroupCol((prev) => (hdr.includes(prev) ? prev : ""));
    }
  }, [filePath, initialContent, initialSha256, delimiter, isTabular]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    onDirtyChange?.(dirty);
  }, [dirty, onDirtyChange]);

  useEffect(() => {
    if (isTabular && !rawCsvMode) return;
    const lines = textDraft.split(/\r?\n/);
    const { line: clampedLine } = clampLineBounds(lines.length, jumpToLine);
    if (!clampedLine) return;
    const el = textareaRef.current;
    if (!el) return;
    let startOffset = 0;
    for (let i = 0; i < clampedLine - 1; i++) {
      startOffset += lines[i].length + 1;
    }
    const endOffset = startOffset + (lines[clampedLine - 1]?.length ?? 0);
    const raf = window.requestAnimationFrame(() => {
      try {
        el.setSelectionRange(startOffset, endOffset);
      } catch {
        // Ignore selection errors if textarea is not focusable
      }
      const lineHeight = 20;
      el.scrollTop = Math.max(0, (clampedLine - 3) * lineHeight);
    });
    return () => window.cancelAnimationFrame(raf);
  }, [filePath, jumpToLine, isTabular, rawCsvMode]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!dirty) return;
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      e.preventDefault();
      e.returnValue = "";
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [dirty]);

  const handleSave = useCallback(
    async (forceOverwrite = false) => {
      if (isExtracted || saving || saveLockRef.current || gridBlockedByMalformed) return;
      saveLockRef.current = true;
      setSaving(true);
      setSaveErr("");
      try {
        const nextContent = currentSerialized;
        const expected = forceOverwrite ? (conflictSha ? conflictSha : undefined) : sha256;
        const res = await saveFileContent({
          path: filePath,
          content: nextContent,
          ...(expected ? { expected_sha256: expected } : {}),
          reason: "edit",
        });
        if (!res.ok) {
          if (res.status === 409) {
            setConflictSha(res.current_sha256 ?? "");
            setSaveErr(res.error || "File changed in the workspace while you were editing.");
            return;
          }
          setSaveErr(res.error || "Could not save changes.");
          toast.error(res.error || "Could not save changes.");
          return;
        }
        setConflictSha(null);
        setSavedContent(nextContent);
        setTextDraft(nextContent);
        setSha256(res.data.sha256);
        onSaved({
          path: res.data.path,
          content: nextContent,
          size: res.data.size,
          mtime_ms: res.data.mtime_ms,
          sha256: res.data.sha256,
        });
        toast.success(`Saved ${filePath.split("/").pop()}.`);
      } finally {
        saveLockRef.current = false;
        setSaving(false);
      }
    },
    [isExtracted, saving, gridBlockedByMalformed, currentSerialized, conflictSha, sha256, filePath, onSaved]
  );

  const handleReloadLatest = useCallback(async () => {
    if (reloading) return;
    setReloading(true);
    setSaveErr("");
    try {
      const ok = await onReloadFromServer();
      if (ok === false) {
        setSaveErr("Could not reload the latest file copy from the server.");
        return;
      }
      setConflictSha(null);
    } catch {
      setSaveErr("Could not reload the latest file copy from the server.");
    } finally {
      setReloading(false);
    }
  }, [reloading, onReloadFromServer]);

  const handleRevert = () => {
    if (isTabular && !rawCsvMode) {
      const raw = inspectDelimited(savedContent, delimiter).rows;
      const hdr = raw.length ? raw[0] : ["Column 1"];
      setTableHeader(hdr);
      setTableRows(raw.slice(1).map((r) => hdr.map((_, i) => r[i] ?? "")));
      setSortCol(null);
    } else {
      setTextDraft(savedContent);
    }
    setCsvNormalized(false);
    setConflictSha(null);
    setSaveErr("");
  };

  const handleNormalizeRaggedGrid = () => {
    if (!csvInspection || csvInspection.unclosedQuote || csvInspection.hasNullByte) return;
    const raw = csvInspection.rows;
    const maxCols = Math.max(1, csvInspection.maxCols);
    const baseHdr = raw[0] ?? [];
    const hdr = Array.from({ length: maxCols }, (_, i) => baseHdr[i] ?? `Column ${i + 1}`);
    const body = raw.slice(1).map((r) => Array.from({ length: maxCols }, (_, i) => r[i] ?? ""));
    const normalized = serializeDelimited([hdr, ...body], delimiter);
    setTableHeader(hdr);
    setTableRows(body);
    setTextDraft(normalized);
    setCsvNormalized(true);
    setRawCsvMode(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (dirty && !isExtracted) void handleSave(false);
    }
  };

  const toggleGridSort = (colIdx: number) => {
    const nextDir: "asc" | "desc" = sortCol === colIdx && sortDir === "asc" ? "desc" : "asc";
    setSortCol(colIdx);
    setSortDir(nextDir);
    const mult = nextDir === "asc" ? 1 : -1;
    setTableRows((prev) =>
      prev.toSorted((a, b) => {
        const av = (a[colIdx] ?? "").trim();
        const bv = (b[colIdx] ?? "").trim();
        const an = parseSortableNumber(av);
        const bn = parseSortableNumber(bv);
        if (an !== null && bn !== null) return (an - bn) * mult;
        return av.localeCompare(bv, undefined, { numeric: true, sensitivity: "base" }) * mult;
      })
    );
  };

  const updateCell = (rIdx: number, cIdx: number, val: string) => {
    setTableRows((prev) => prev.map((row, i) => (i === rIdx ? row.map((c, j) => (j === cIdx ? val : c)) : row)));
  };

  const addRow = () => {
    setTableRows((prev) => [...prev, tableHeader.map(() => "")]);
  };

  const deleteRow = (rIdx: number) => {
    setTableRows((prev) => prev.filter((_, i) => i !== rIdx));
  };

  const runTableQuery = async (saveAsPath?: string) => {
    if (queryLockRef.current || queryBusy) return;
    queryLockRef.current = true;
    setQueryBusy(true);
    setQueryErr("");
    try {
      const where =
        whereCol && whereVal.trim() !== ""
          ? [{ col: whereCol, op: whereOp, value: whereVal.trim() }]
          : undefined;
      const groupBy = groupCol ? [groupCol] : undefined;
      const aggAlias = aggFn && aggCol ? `${aggFn}_${aggCol.toLowerCase().replace(/[^a-z0-9]+/g, "_")}` : "";
      const aggregates =
        aggFn && aggCol ? [{ col: aggCol, fn: aggFn, as: aggAlias }] : undefined;
      const sort = aggAlias
        ? { col: aggAlias, dir: querySortDir }
        : groupCol
          ? { col: groupCol, dir: querySortDir }
          : undefined;

      const res = await queryTable({
        path: filePath,
        ...(sheet ? { sheet } : {}),
        where,
        group_by: groupBy,
        aggregates,
        sort,
        limit: 500,
        ...(saveAsPath ? { save_as: saveAsPath } : {}),
      });
      if (!res.ok) {
        setQueryErr(res.error || "Query could not run.");
        toast.error(res.error || "Table query failed.");
        return;
      }
      setQueryResult(res.data);
      if (res.data.saved_path) {
        toast.success(`Saved ${res.data.saved_path} to workspace.`);
        onWorkspaceMutated?.();
      }
    } finally {
      queryLockRef.current = false;
      setQueryBusy(false);
    }
  };

  return (
    <div id="file-editor" className="flex flex-col gap-3" onKeyDown={handleKeyDown}>
      {/* Editor toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board/40 px-3 py-2">
        <div className="flex flex-wrap items-center gap-2">
          {isExtracted ? (
            <Badge variant="outline" size="sm">
              Extracted {extractedKind || "document"} (read-only source)
            </Badge>
          ) : dirty ? (
            <Badge variant="outline" size="sm" className="status-waiting border-hairline">
              Unsaved changes
            </Badge>
          ) : (
            <Badge variant="outline" size="sm" className="status-finished border-hairline">
              Saved
            </Badge>
          )}
          <span className="mono text-xs text-ink-2">
            {isTabular && !rawCsvMode
              ? `${tableRows.length} ${tableRows.length === 1 ? "row" : "rows"}, ${tableHeader.length} cols`
              : `${textDraft.split(/\r?\n/).length} lines`}
          </span>
          {!isExtracted && (
            <span className="hidden items-center gap-1 text-xs text-ink-2 sm:inline-flex">
              <Kbd>Ctrl/Cmd</Kbd>
              <span>+</span>
              <Kbd>S</Kbd>
            </span>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {(!isTabular || rawCsvMode) && !isExtracted && (
            <SegmentedControl aria-label="Editor layout mode">
              <SegmentedControlItem
                data-id="split"
                aria-pressed={viewMode === "split"}
                data-checked={viewMode === "split" ? "" : undefined}
                className="min-h-8 gap-1 px-2.5 max-md:min-h-11"
                onClick={() => setViewMode("split")}
              >
                <Columns2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Split</span>
              </SegmentedControlItem>
              <SegmentedControlItem
                data-id="write"
                aria-pressed={viewMode === "write"}
                data-checked={viewMode === "write" ? "" : undefined}
                className="min-h-8 gap-1 px-2.5 max-md:min-h-11"
                onClick={() => setViewMode("write")}
              >
                <Pencil className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Write</span>
              </SegmentedControlItem>
              <SegmentedControlItem
                data-id="preview"
                aria-pressed={viewMode === "preview"}
                data-checked={viewMode === "preview" ? "" : undefined}
                className="min-h-8 gap-1 px-2.5 max-md:min-h-11"
                onClick={() => setViewMode("preview")}
              >
                <Eye className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Preview</span>
              </SegmentedControlItem>
            </SegmentedControl>
          )}

          {isTabular && !isExtracted && !rawCsvMode && !gridBlockedByMalformed && (
            <Button
              id="csv-add-row"
              type="button"
              size="sm"
              variant="outline"
              className="max-md:min-h-11"
              onClick={addRow}
            >
              <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
              Add row
            </Button>
          )}

          {isTabular && !isExtracted && rawCsvMode && (
            <Button
              id="csv-grid-return-btn"
              type="button"
              size="sm"
              variant="outline"
              className="max-md:min-h-11"
              onClick={() => {
                const check = inspectDelimited(textDraft, delimiter);
                if (check.error) {
                  setSaveErr(check.error);
                  return;
                }
                const hdr = check.rows.length ? check.rows[0] : ["Column 1"];
                setTableHeader(hdr);
                setTableRows(check.rows.slice(1).map((r) => hdr.map((_, i) => r[i] ?? "")));
                setCsvNormalized(true);
                setRawCsvMode(false);
                setSaveErr("");
              }}
            >
              <FileSpreadsheet className="size-3.5" strokeWidth={1.5} aria-hidden />
              Use grid view
            </Button>
          )}

          {dirty && !isExtracted && (
            <Button
              id="file-editor-revert"
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={handleRevert}
            >
              <RotateCcw className="size-3.5" strokeWidth={1.5} aria-hidden />
              Revert
            </Button>
          )}

          {!isExtracted && (
            <Button
              id="file-editor-save"
              type="button"
              size="sm"
              className="max-md:min-h-11"
              disabled={!dirty || saving || gridBlockedByMalformed}
              onClick={() => void handleSave(false)}
            >
              <Save className="size-3.5" strokeWidth={1.5} aria-hidden />
              {saving ? "Saving…" : "Save changes"}
            </Button>
          )}
        </div>
      </div>

      {/* Optimistic lock 409 Conflict banner */}
      {conflictSha !== null && (
        <div
          id="file-editor-conflict"
          role="alert"
          className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-board p-3 text-sm text-ink"
        >
          <div className="min-w-0 flex-1">
            <p className="font-medium">This file changed while you were editing</p>
            <p className="text-xs text-ink-2">
              Another run or tab updated {filePath}
              {conflictSha ? ` (server SHA ${conflictSha.slice(0, 10)})` : ""}. Reload the latest copy or overwrite with your draft.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              id="file-editor-reload"
              type="button"
              size="sm"
              variant="outline"
              className="max-md:min-h-11"
              disabled={reloading || saving}
              onClick={() => void handleReloadLatest()}
            >
              <RefreshCw className="size-3.5" strokeWidth={1.5} aria-hidden />
              {reloading ? "Reloading…" : "Reload latest"}
            </Button>
            <Button
              id="file-editor-force-save"
              type="button"
              size="sm"
              className="max-md:min-h-11"
              disabled={saving || reloading || gridBlockedByMalformed}
              onClick={() => void handleSave(true)}
            >
              {saving ? "Overwriting…" : "Overwrite anyway"}
            </Button>
          </div>
        </div>
      )}

      {saveErr && conflictSha === null && (
        <p role="alert" className="rounded-sm border border-hairline bg-board px-3 py-2 text-sm text-ink">
          {saveErr}
        </p>
      )}

      {/* Tabular mode: Malformed CSV error state, Table Query Bar & Inline Editable Grid */}
      {isTabular && !rawCsvMode ? (
        <div className="flex flex-col gap-3">
          {gridBlockedByMalformed && csvInspection?.error && (
            <div
              id="csv-grid-error"
              role="alert"
              className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-status-stopped-bg p-3 text-sm text-status-stopped-fg"
            >
              <div className="min-w-0 flex-1">
                <p className="font-medium">Malformed CSV cannot open safely in grid mode</p>
                <p className="text-xs opacity-90">{csvInspection.error}</p>
              </div>
              {!isExtracted && (
                <div className="flex flex-wrap items-center gap-2">
                  {!csvInspection.unclosedQuote && !csvInspection.hasNullByte && csvInspection.raggedRows > 0 && (
                    <Button
                      id="csv-normalize-btn"
                      type="button"
                      size="sm"
                      variant="outline"
                      className="max-md:min-h-11 bg-surface text-ink"
                      onClick={handleNormalizeRaggedGrid}
                    >
                      Pad columns to grid
                    </Button>
                  )}
                  {!csvInspection.hasNullByte && (
                    <Button
                      id="csv-raw-fallback-btn"
                      type="button"
                      size="sm"
                      className="max-md:min-h-11"
                      onClick={() => {
                        setTextDraft(savedContent);
                        setRawCsvMode(true);
                        setViewMode("write");
                        setSaveErr("");
                      }}
                    >
                      Edit as raw text
                    </Button>
                  )}
                </div>
              )}
            </div>
          )}
          {/* Table Query / Group-By Bar */}
          <section
            id="table-query-bar"
            aria-label="Table query and rollup"
            className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-surface p-3"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-1.5">
                <FileSpreadsheet className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                <span className="text-xs font-medium text-ink">Table query &amp; group-by rollup</span>
              </div>
              {dirty && (
                <span className="text-xs text-ink-2">Save grid edits first so queries read your latest cells.</span>
              )}
            </div>

            <div className="grid grid-cols-1 gap-2 md:grid-cols-6">
              <div className="flex flex-col gap-1">
                <label htmlFor="table-query-where-col" className="text-xs text-ink-2">
                  Where column
                </label>
                <select
                  id="table-query-where-col"
                  aria-label="Query filter column"
                  value={whereCol}
                  onChange={(e) => setWhereCol(e.target.value)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="">All rows</option>
                  {tableHeader.map((col, idx) => (
                    <option key={idx} value={col}>
                      {col || `Column ${idx + 1}`}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="table-query-where-op" className="text-xs text-ink-2">
                  Condition
                </label>
                <select
                  id="table-query-where-op"
                  aria-label="Query filter operator"
                  value={whereOp}
                  onChange={(e) => setWhereOp(e.target.value as TableWhereOp)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none max-md:min-h-11"
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
                <label htmlFor="table-query-where-val" className="text-xs text-ink-2">
                  Filter value
                </label>
                <Input
                  id="table-query-where-val"
                  aria-label="Query filter value"
                  placeholder="Match value…"
                  value={whereVal}
                  onChange={(e) => setWhereVal(e.target.value)}
                  className="h-9 text-xs max-md:min-h-11"
                />
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="table-query-group-col" className="text-xs text-ink-2">
                  Group by
                </label>
                <select
                  id="table-query-group-col"
                  aria-label="Group by column"
                  value={groupCol}
                  onChange={(e) => setGroupCol(e.target.value)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="">No grouping</option>
                  {tableHeader.map((col, idx) => (
                    <option key={idx} value={col}>
                      {col || `Column ${idx + 1}`}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="table-query-agg-fn" className="text-xs text-ink-2">
                  Aggregate
                </label>
                <select
                  id="table-query-agg-fn"
                  aria-label="Aggregate function"
                  value={aggFn}
                  onChange={(e) => setAggFn(e.target.value as "" | TableAggregateFn)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="">None</option>
                  <option value="sum">SUM</option>
                  <option value="avg">AVG</option>
                  <option value="min">MIN</option>
                  <option value="max">MAX</option>
                  <option value="count">COUNT</option>
                </select>
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="table-query-agg-col" className="text-xs text-ink-2">
                  Target column
                </label>
                <select
                  id="table-query-agg-col"
                  aria-label="Aggregate column"
                  value={aggCol}
                  disabled={!aggFn}
                  onChange={(e) => setAggCol(e.target.value)}
                  className="focus-ring h-9 min-h-9 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none disabled:cursor-not-allowed disabled:bg-board disabled:text-ink-3 max-md:min-h-11"
                >
                  {tableHeader.map((col, idx) => (
                    <option key={idx} value={col}>
                      {col || `Column ${idx + 1}`}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 pt-0.5">
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  id="table-query-run"
                  type="button"
                  size="sm"
                  className="max-md:min-h-11"
                  disabled={queryBusy}
                  onClick={() => void runTableQuery()}
                >
                  <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {queryBusy ? "Running…" : "Run Query"}
                </Button>
                <select
                  id="table-query-sort-dir"
                  aria-label="Query sort direction"
                  value={querySortDir}
                  onChange={(e) => setQuerySortDir(e.target.value as "asc" | "desc")}
                  className="focus-ring h-8 min-h-8 rounded-sm border border-input bg-board/40 px-2 text-xs text-ink outline-none max-md:min-h-11"
                >
                  <option value="desc">Sort desc</option>
                  <option value="asc">Sort asc</option>
                </select>
                {queryResult && (
                  <Button
                    id="table-query-clear"
                    type="button"
                    size="sm"
                    variant="ghost"
                    className="max-md:min-h-11"
                    onClick={() => {
                      setQueryResult(null);
                      setQueryErr("");
                    }}
                  >
                    <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Clear query
                  </Button>
                )}
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Input
                  id="table-query-save-name"
                  aria-label="Output CSV filename"
                  value={saveQueryName}
                  onChange={(e) => setSaveQueryName(e.target.value)}
                  className="mono h-8 w-48 text-xs max-md:min-h-11"
                />
                <Button
                  id="table-query-save-csv"
                  type="button"
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={queryBusy || !saveQueryName.trim()}
                  onClick={() => void runTableQuery(saveQueryName.trim())}
                >
                  <Save className="size-3.5" strokeWidth={1.5} aria-hidden />
                  Save query as CSV
                </Button>
              </div>
            </div>

            {queryErr && (
              <p role="alert" className="text-xs text-ink">
                {queryErr}
              </p>
            )}
          </section>

          {/* Query Results */}
          {queryResult && (
            <section
              id="table-query-result"
              aria-label="Table query results"
              className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/30 p-3"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-xs font-medium text-ink">Query result</span>
                  <Badge variant="outline" size="sm" className="mono">
                    {queryResult.rows.length} {queryResult.rows.length === 1 ? "row" : "rows"} ({queryResult.matched_rows} of {queryResult.total_rows} matched)
                  </Badge>
                  {queryResult.saved_path && (
                    <Badge variant="outline" size="sm" className="status-finished mono border-hairline">
                      Saved: {queryResult.saved_path}
                    </Badge>
                  )}
                </div>
                {queryResult.saved_path && onOpenSavedFile && (
                  <Button
                    type="button"
                    size="xs"
                    variant="outline"
                    className="max-md:min-h-11"
                    onClick={() => onOpenSavedFile(queryResult.saved_path!)}
                  >
                    Open {queryResult.saved_path}
                  </Button>
                )}
              </div>

              <div className="max-h-56 overflow-auto rounded-sm border border-hairline bg-surface">
                <table className="w-full border-collapse text-left text-xs">
                  <thead className="sticky top-0 border-b border-hairline bg-board text-ink">
                    <tr>
                      {queryResult.columns.map((col, idx) => (
                        <th key={idx} className="whitespace-nowrap px-3 py-2 font-medium">
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-hairline">
                    {queryResult.rows.length === 0 ? (
                      <tr>
                        <td colSpan={Math.max(1, queryResult.columns.length)} className="px-3 py-3 text-center text-ink-2">
                          No rows matched your query.
                        </td>
                      </tr>
                    ) : (
                      queryResult.rows.map((row, rIdx) => (
                        <tr key={rIdx} className="hover:bg-board/40">
                          {queryResult.columns.map((_, cIdx) => (
                            <td key={cIdx} className="whitespace-nowrap px-3 py-1.5 text-ink">
                              {row[cIdx] ?? ""}
                            </td>
                          ))}
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </section>
          )}

          {/* Inline Editable CSV Grid */}
          {!gridBlockedByMalformed && (
          <div className="max-h-[42vh] overflow-auto rounded-sm border border-hairline bg-surface">
            <table id="csv-editor-table" className="w-full border-collapse text-left text-xs">
              <thead className="sticky top-0 z-10 border-b border-hairline bg-board text-ink">
                <tr>
                  <th className="w-10 px-2 py-2 text-center font-medium text-ink-2">#</th>
                  {tableHeader.map((col, idx) => {
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
                          onClick={() => toggleGridSort(idx)}
                          className="focus-ring flex min-h-8 w-full items-center gap-1.5 px-2.5 py-1.5 text-left text-xs font-medium text-ink outline-none hover:bg-track max-md:min-h-11"
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
                  {!isExtracted && <th className="w-10 px-2 py-2 text-center font-medium text-ink-2">Row</th>}
                </tr>
              </thead>
              <tbody className="divide-y divide-hairline">
                {tableRows.length === 0 ? (
                  <tr>
                    <td
                      colSpan={tableHeader.length + (isExtracted ? 1 : 2)}
                      className="px-3 py-4 text-center text-sm text-ink-2"
                    >
                      No data rows yet. Click Add row above to insert one.
                    </td>
                  </tr>
                ) : (
                  tableRows.map((row, rIdx) => (
                    <tr key={rIdx} className="hover:bg-board/40">
                      <td className="mono px-2 py-1 text-center text-xs text-ink-2">{rIdx + 1}</td>
                      {tableHeader.map((col, cIdx) => (
                        <td key={cIdx} className="p-1">
                          {isExtracted ? (
                            <span className="block px-2 py-1 text-xs text-ink">{row[cIdx] ?? ""}</span>
                          ) : (
                            <input
                              type="text"
                              aria-label={`Row ${rIdx + 1} ${col || `Column ${cIdx + 1}`}`}
                              value={row[cIdx] ?? ""}
                              onChange={(e) => updateCell(rIdx, cIdx, e.target.value)}
                              className="h-7 min-h-7 w-full min-w-[96px] rounded-xs border border-transparent bg-transparent px-2 py-1 text-xs text-ink focus:border-blue focus:bg-surface focus:outline-none max-md:min-h-11"
                            />
                          )}
                        </td>
                      ))}
                      {!isExtracted && (
                        <td className="p-1 text-center">
                          <Button
                            type="button"
                            size="icon-xs"
                            variant="ghost"
                            aria-label={`Delete row ${rIdx + 1}`}
                            className="min-h-7 min-w-7 text-ink-2 hover:text-ink max-md:min-h-11 max-md:min-w-11"
                            onClick={() => deleteRow(rIdx)}
                          >
                            <Trash2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                          </Button>
                        </td>
                      )}
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
          )}
        </div>
      ) : (
        /* Split Markdown / Text Editor */
        <div
          className={cn(
            "grid gap-3",
            viewMode === "split" ? "grid-cols-1 md:grid-cols-2" : "grid-cols-1"
          )}
        >
          {(viewMode === "split" || viewMode === "write") && (
            <div className="flex flex-col gap-1.5">
              <label htmlFor="file-editor-textarea" className="text-xs font-medium text-ink-2">
                Editor
              </label>
              <Textarea
                ref={textareaRef}
                id="file-editor-textarea"
                aria-label={`Edit ${filePath}`}
                readOnly={isExtracted}
                value={textDraft}
                onChange={(e) => setTextDraft(e.target.value)}
                rows={16}
                className="min-h-[320px] resize-y font-mono text-xs leading-5"
              />
            </div>
          )}

          {(viewMode === "split" || viewMode === "preview") && (
            <div className="flex flex-col gap-1.5">
              <span className="text-xs font-medium text-ink-2">Live preview</span>
              <div className="max-h-[420px] min-h-[320px] overflow-y-auto rounded-sm border border-hairline bg-surface p-3">
                {isMarkdown ? (
                  <MarkdownView content={textDraft} />
                ) : (
                  <pre className="whitespace-pre-wrap break-words font-mono text-xs leading-5 text-ink">
                    {textDraft || "(empty file)"}
                  </pre>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
