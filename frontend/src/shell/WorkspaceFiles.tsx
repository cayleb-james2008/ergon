// Real Workspace Files panel & Workspace File Viewer modal:
// - Lists active person's workspace files (GET /api/files)
// - Uploads real files from disk or creates new text/CSV/Markdown files (POST /api/files/upload)
// - Opens any saved file chip or Open button into a real previewer (GET /api/files/content?path=...)
//   rendering CSV as a clean scrollable table, Markdown as structured nodes, JSON pretty-printed,
//   and plain text with line numbers, plus real Download & Copy buttons.
import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUpRight, Copy, Download, FilePlus2, FolderOpen, MessageSquarePlus, Upload } from "lucide-react";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  MorphingDialog,
  MorphingDialogClose,
  MorphingDialogContainer,
  MorphingDialogContent,
  MorphingDialogDescription,
  MorphingDialogTitle,
  MorphingDialogTrigger,
  useMorphingDialog,
} from "@/components/ui/morphing-dialog";
import { SlidingNumber } from "@/components/ui/sliding-number";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { FileGlyph } from "@/chat/Entry";
import {
  call,
  downloadWorkspaceFile,
  type WorkspaceFile,
  type WorkspaceFileContent,
} from "@/lib/api";
import { relTime } from "@/lib/words";
import { open as openPreset } from "@/motion";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board";

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

function renderInlineCode(line: string): React.ReactNode {
  const parts = line.split(/(`[^`]+`)/g);
  if (parts.length === 1) return line;
  return parts.map((p, i) =>
    p.startsWith("`") && p.endsWith("`") && p.length > 2
      ? <code key={i} className="mono rounded-chip bg-board px-1.5 py-0.5 text-xs text-ink">{p.slice(1, -1)}</code>
      : <span key={i}>{p}</span>
  );
}

function MarkdownView({ content }: { content: string }) {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  const blocks: React.ReactNode[] = [];
  let list: string[] = [];
  let codeLines: string[] | null = null;
  const flushList = (key: number) => {
    if (!list.length) return;
    blocks.push(
      <ul key={"ul-" + key} className="flex list-disc flex-col gap-1 pl-5 text-sm leading-6 text-ink">
        {list.map((item, idx) => <li key={idx}>{renderInlineCode(item)}</li>)}
      </ul>
    );
    list = [];
  };
  lines.forEach((line, idx) => {
    const t = line.trim();
    if (t.startsWith("```")) {
      flushList(idx);
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
    if (!t) {
      flushList(idx);
      return;
    }
    if (t.startsWith("### ")) {
      flushList(idx);
      blocks.push(<h4 key={"h3-" + idx} className="pt-2 text-sm font-semibold text-ink">{renderInlineCode(t.slice(4))}</h4>);
    } else if (t.startsWith("## ")) {
      flushList(idx);
      blocks.push(<h3 key={"h2-" + idx} className="pt-3 text-base font-medium text-ink">{renderInlineCode(t.slice(3))}</h3>);
    } else if (t.startsWith("# ")) {
      flushList(idx);
      blocks.push(<h2 key={"h1-" + idx} className="text-title font-medium text-ink">{renderInlineCode(t.slice(2))}</h2>);
    } else if (/^[-*]\s+/.test(t)) {
      list.push(t.replace(/^[-*]\s+/, ""));
    } else if (/^\d+\.\s+/.test(t)) {
      list.push(t.replace(/^\d+\.\s+/, ""));
    } else if (/^---+$/.test(t)) {
      flushList(idx);
      blocks.push(<hr key={"hr-" + idx} className="my-2 border-hairline" />);
    } else {
      flushList(idx);
      blocks.push(<p key={"p-" + idx} className="text-sm leading-6 text-ink">{renderInlineCode(t)}</p>);
    }
  });
  flushList(lines.length + 1);
  if (codeLines !== null) {
    blocks.push(
      <pre key="code-tail" className="overflow-x-auto rounded-sm border border-hairline bg-board p-3 font-mono text-xs leading-5 text-ink">
        {(codeLines as string[]).join("\n")}
      </pre>
    );
  }
  return <div className="flex flex-col gap-2">{blocks}</div>;
}

function CsvTableView({ content }: { content: string }) {
  const rows = useMemo(() => parseCsv(content), [content]);
  if (!rows.length) {
    return <p className="text-sm text-ink-2">This CSV file is empty.</p>;
  }
  const header = rows[0];
  const body = rows.slice(1);
  const isNumeric = (v: string) => /^-?\$?\d[\d,]*(?:\.\d+)?%?$/.test(v.trim());
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between text-xs text-ink-2">
        <span className="mono">{body.length} {body.length === 1 ? "row" : "rows"} · {header.length} {header.length === 1 ? "column" : "columns"}</span>
      </div>
      <div className="max-h-[52vh] overflow-auto rounded-sm border border-hairline bg-surface">
        <table className="w-full border-collapse text-left text-xs">
          <thead className="sticky top-0 border-b border-hairline bg-board text-ink">
            <tr>
              {header.map((col, idx) => (
                <th key={idx} className="whitespace-nowrap px-3 py-2 font-medium">{col || `Column ${idx + 1}`}</th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-hairline">
            {body.map((r, rIdx) => (
              <tr key={rIdx} className="hover:bg-board/50">
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
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export function FileViewerModal({
  filePath,
  onClose,
  onAskAbout,
}: {
  filePath: string | null;
  onClose: () => void;
  onAskAbout?: (path: string) => void;
}) {
  const [data, setData] = useState<WorkspaceFileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");

  useEffect(() => {
    if (!filePath) {
      setData(null);
      setErr("");
      return;
    }
    let cancelled = false;
    setLoading(true);
    setErr("");
    call<WorkspaceFileContent>("/api/files/content?path=" + encodeURIComponent(filePath)).then((r) => {
      if (cancelled) return;
      setLoading(false);
      if (!r.ok) {
        setData(null);
        setErr(r.status === 404 ? `"${filePath}" is no longer in this workspace (it may have been deleted or moved).` : r.error || "Could not open this file.");
        return;
      }
      setData(r.data);
    });
    return () => { cancelled = true; };
  }, [filePath]);

  const ext = (data?.ext || filePath?.split(".").pop() || "").toLowerCase();
  const formattedJson = useMemo(() => {
    if (ext !== "json" || !data?.content) return null;
    try {
      return JSON.stringify(JSON.parse(data.content), null, 2);
    } catch {
      return data.content;
    }
  }, [ext, data?.content]);

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

  return (
    <Dialog open={!!filePath} onOpenChange={(open) => { if (!open) onClose(); }}>
      <DialogContent id="file-viewer-dialog" className="max-h-[86vh] max-w-3xl overflow-hidden">
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

        <div className="flex flex-wrap items-center gap-2 border-y border-hairline py-2.5">
          <Button size="sm" variant="outline" className="max-md:min-h-11" disabled={!filePath || !!err} onClick={handleDownload}>
            <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
            Download
          </Button>
          {data && !data.is_binary && (
            <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={copyContent}>
              <Copy className="size-3.5" strokeWidth={1.5} aria-hidden />
              Copy text
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
              Ask agent about this file
            </Button>
          )}
        </div>

        <div id="file-viewer-body" className="max-h-[58vh] min-h-24 overflow-y-auto pr-1">
          {loading && <p className="text-sm text-ink-2">Loading file…</p>}
          {!loading && err && <p role="alert" className="rounded-sm border border-hairline bg-board p-3.5 text-sm text-ink">{err}</p>}
          {!loading && !err && data && (
            data.is_binary ? (
              <p className="text-sm text-ink-2">Binary file ({formatBytes(data.size)}). Use Download above to save it to your computer.</p>
            ) : ext === "csv" ? (
              <CsvTableView content={data.content} />
            ) : ext === "md" || ext === "markdown" ? (
              <div className="rounded-sm border border-hairline bg-board/30 p-4">
                <MarkdownView content={data.content} />
              </div>
            ) : ext === "json" && formattedJson !== null ? (
              <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded-sm border border-hairline bg-board p-3.5 font-mono text-xs leading-5 text-ink">
                {formattedJson}
              </pre>
            ) : (
              <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded-sm border border-hairline bg-board p-3.5 font-mono text-xs leading-5 text-ink">
                {data.content || "(empty file)"}
              </pre>
            )
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function WorkspaceFilesBody({
  files,
  onRefresh,
  onOpenFile,
  onUpload,
}: {
  files: WorkspaceFile[];
  onRefresh: () => void;
  onOpenFile: (path: string) => void;
  onUpload: (path: string, content: string) => Promise<boolean>;
}) {
  const { isOpen, setIsOpen } = useMorphingDialog();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newContent, setNewContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState("");

  useEffect(() => {
    if (isOpen) onRefresh();
  }, [isOpen, onRefresh]);

  const handleFilePick = async (ev: React.ChangeEvent<HTMLInputElement>) => {
    const picked = ev.target.files;
    if (!picked || !picked.length) return;
    setErr("");
    for (const file of Array.from(picked)) {
      const content = await file.text();
      const ok = await onUpload(file.name, content);
      if (!ok) setErr(`Could not upload ${file.name}.`);
    }
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
    <div id="workspace-files-panel" className="flex flex-col gap-4">
      <p className="text-sm text-ink-2">
        Real files in this person&apos;s sandboxed workspace. Upload spreadsheets, notes, or documents for your agent to read and process.
      </p>

      <div className="flex flex-wrap items-center gap-2">
        <input
          ref={fileInputRef}
          id="workspace-file-input"
          type="file"
          multiple
          className="sr-only"
          accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
          onChange={handleFilePick}
        />
        <Button size="sm" className="max-md:min-h-11" onClick={() => fileInputRef.current?.click()}>
          <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
          Upload file
        </Button>
        <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={() => setCreating((v) => !v)}>
          <FilePlus2 className="size-3.5" strokeWidth={1.5} aria-hidden />
          {creating ? "Cancel" : "New file"}
        </Button>
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

      {err && <p role="alert" className="text-sm text-destructive-foreground">{err}</p>}

      {!files.length ? (
        <p className="rounded-card border border-hairline bg-board/40 p-3.5 text-sm text-ink-2">
          No files in this workspace yet. Upload a CSV or Markdown file above, or ask your agent to create one.
        </p>
      ) : (
        <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
          {files.map((f) => (
            <li key={f.path} className="flex flex-wrap items-center justify-between gap-2 px-3 py-2.5">
              <div className="flex min-w-0 flex-1 items-center gap-2">
                <FileGlyph name={f.path} />
                <div className="min-w-0 flex-1">
                  <p className="mono truncate text-sm font-medium text-ink">{f.path}</p>
                  <p className="mono text-xs text-ink-3">
                    {formatBytes(f.size)}{f.mtime_ms > 0 ? ` · ${relTime(f.mtime_ms)}` : ""}
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
                    setIsOpen(false);
                    setTimeout(() => onOpenFile(f.path), 40);
                  }}
                >
                  Open
                  <ArrowUpRight className="size-3.5" strokeWidth={1.5} aria-hidden />
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
      )}
    </div>
  );
}

export function WorkspaceFilesPanel({
  files,
  onRefresh,
  onOpenFile,
  onUpload,
  open,
  onOpenChange,
  hideTrigger,
}: {
  files: WorkspaceFile[];
  onRefresh: () => void;
  onOpenFile: (path: string) => void;
  onUpload: (path: string, content: string) => Promise<boolean>;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
}) {
  return (
    <MorphingDialog transition={openPreset} open={open} onOpenChange={onOpenChange}>
      {!hideTrigger && (
        <MorphingDialogTrigger thing="files-btn" className={TRIGGER} style={{ borderRadius: 7 }}>
          <span className="flex min-w-0 items-center gap-1.5">
            <FolderOpen className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
            <span className="leading-tight">Workspace files</span>
          </span>
          <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
            <SlidingNumber value={files.length} />
          </span>
        </MorphingDialogTrigger>
      )}
      <MorphingDialogContainer>
        <MorphingDialogContent
          style={{ borderRadius: 18 }}
          className="relative mx-4 flex max-h-[84vh] w-[560px] max-w-[calc(100vw-2rem)] flex-col gap-3 overflow-y-auto border border-hairline bg-surface p-5 text-ink shadow-lift"
        >
          <MorphingDialogClose className="focus-ring absolute right-3 top-3 grid size-11 place-items-center rounded-sm text-icon outline-none hover:bg-board md:size-9" />
          <MorphingDialogTitle className="pr-10 text-title font-medium text-ink">Workspace files</MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">
            <WorkspaceFilesBody files={files} onRefresh={onRefresh} onOpenFile={onOpenFile} onUpload={onUpload} />
          </MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}
