// Work Entry cards & inline Approval/File-Diff card for the central Conversation Thread.
// Keeps clean typography, tabular-nums timestamps, inline file diff preview, 1-click Word/Excel & SHA-256 receipt export.
import { memo, useEffect, useState } from "react";
import { Check, ChevronDown, Copy, FileDown, FileSpreadsheet, FileText, Hourglass, PackageCheck, Repeat, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { computeLineDiff } from "@/lib/diff";
import {
  exportClientPackDownload,
  exportReceiptDownload,
  preflightClientPack,
  verifyCitations,
  type CitationReport,
  type PackPreflightReport,
} from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { extractCitations, extractPlaybookFromRun, fileIconFor, savedFile, toolLabel } from "@/lib/words";
import {
  call,
  exportTarget,
  type Decision,
  type FileDiffLine,
  type FileRevision,
  type Msg,
  type WorkspaceFileContent,
} from "@/lib/api";

export function FileGlyph({ name, className }: { name: string; className?: string }) {
  const Icon = fileIconFor(name);
  return <Icon className={cn("size-4 shrink-0 text-ink-2", className)} strokeWidth={1.5} aria-hidden="true" />;
}

/** Accessible collapsible disclosure without heavy animation runtime dependencies. */
export function Details({ label = "Details", children }: { label?: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="w-full">
      <button
        type="button"
        aria-expanded={open}
        aria-label={label}
        onClick={() => setOpen((v) => !v)}
        className="focus-ring flex min-h-11 w-full items-center justify-between gap-2 rounded-sm border border-hairline bg-board/40 px-3 text-left text-sm font-medium text-ink outline-none hover:bg-board md:min-h-9"
      >
        <span>{label}</span>
        <ChevronDown
          className={cn(
            "size-4 shrink-0 text-icon motion-safe:transition-transform motion-safe:duration-(--dur-swap) motion-safe:ease-(--ease-house)",
            open && "rotate-180"
          )}
          strokeWidth={1.5}
          aria-hidden
        />
      </button>
      {open && <div className="mt-2 flex flex-col gap-2.5">{children}</div>}
    </div>
  );
}

export function EntryShell({
  time,
  title,
  status,
  label,
  children,
}: {
  time: string;
  title: string;
  status?: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <article
      aria-label={label}
      className="overflow-hidden rounded-entry border border-hairline bg-surface"
    >
      <header className="entry-head flex min-w-0 flex-wrap items-center gap-2.5 border-b border-hairline px-4 py-2.5">
        {time && <span className="mono shrink-0 text-xs text-ink-2">{time}</span>}
        <span className="min-w-0 flex-1 truncate text-sm font-medium text-ink">{title}</span>
        {status}
      </header>
      {children}
    </article>
  );
}

export function FinishedPill() {
  return (
    <Pill className="status-finished border-0 px-2.5 py-0.5 text-xs font-medium">
      <Check className="size-3.5" strokeWidth={1.75} aria-hidden />Finished
    </Pill>
  );
}

export function WaitingPill() {
  return (
    <Pill className="status-waiting border-0 px-2.5 py-0.5 text-xs font-medium">
      <Hourglass className="size-3.5" strokeWidth={1.75} aria-hidden />Waiting on you
    </Pill>
  );
}

export function Cells({ children }: { children: React.ReactNode }) {
  return <div className="entry-rule grid grid-cols-2 gap-px border-b border-hairline">{children}</div>;
}

export function Cell({ label, children, span }: { label: string; children: React.ReactNode; span?: boolean }) {
  return (
    <div className={cn("min-w-0 bg-surface px-3.5 py-2.5", span && "col-span-2")}>
      <p className="text-xs text-ink-3">{label}</p>
      <div className="mt-0.5 min-w-0 break-words text-sm font-medium text-ink">{children}</div>
    </div>
  );
}

export function FileRow({ files, onOpenFile }: { files: string[]; onOpenFile?: (path: string) => void }) {
  if (!files.length) return null;
  return (
    <div className="flex min-w-0 flex-wrap items-center justify-between gap-2 border-b border-hairline bg-board/40 px-4 py-2.5">
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        {files.map((name, idx) =>
          onOpenFile ? (
            <Button
              key={name + idx}
              size="sm"
              variant="outline"
              aria-label={`Open ${name}`}
              onClick={() => onOpenFile(name)}
              className="max-w-full gap-1.5 rounded-chip border-hairline bg-surface px-2.5 text-xs font-normal text-ink hover:bg-board max-md:min-h-11"
            >
              <FileGlyph name={name} />
              <span className="mono min-w-0 truncate">{name}</span>
              <span className="text-ink-2">Inspect</span>
            </Button>
          ) : (
            <Pill key={name + idx} variant="secondary" className="max-w-full border border-hairline bg-surface px-2.5 py-1 text-xs font-normal text-ink">
              <FileGlyph name={name} />
              <span className="mono min-w-0 truncate">{name}</span>
            </Pill>
          )
        )}
      </div>
      <span className="text-xs text-ink-3">Saved in workspace</span>
    </div>
  );
}

export function Note({ text }: { text: string }) {
  return (
    <div className="rounded-card border border-hairline bg-board px-3.5 py-2.5 text-sm text-ink-2">
      <span className="font-medium text-ink">Check this. </span>{String(text ?? "").replace(/^Note:\s*/, "")}
    </div>
  );
}

export function LedgerRow({ raw }: { raw: string }) {
  const safe = String(raw ?? "");
  const nl = safe.indexOf("\n");
  const head = nl < 0 ? safe : safe.slice(0, nl);
  const body = nl < 0 ? "" : safe.slice(nl + 1);
  const m = /^Tool: (\S+)(?: \(via ([^)]+)\))?$/.exec(head);
  const label = (m && toolLabel(m[1], m[2], body)) || head;
  const toolName = m ? m[1] : "";
  return (
    <li className="flex flex-col gap-1.5 py-2.5">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <span className="flex items-start gap-2 text-sm font-medium text-ink">
          <Check className="mt-0.5 size-4 shrink-0 text-dot-green" strokeWidth={1.75} aria-hidden />
          <span>{label}</span>
        </span>
        {toolName && <span className="mono rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">{toolName}</span>}
      </div>
      <pre className="whitespace-pre-wrap break-words rounded-sm bg-board px-2.5 py-1.5 font-mono text-xs text-ink-2">{safe}</pre>
    </li>
  );
}

export function StepList({ steps }: { steps: string[] }) {
  return (
    <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface px-3.5">
      {steps.map((s, i) => <LedgerRow key={i} raw={s} />)}
    </ol>
  );
}

const INLINE_CITATION_RE =
  /\[([A-Za-z0-9_./ -]+\.(?:md|markdown|csv|tsv|txt|json|log|py|ts|tsx|js|rs|rn|toml|ya?ml|html)):L?(\d+)(?:-L?(\d+))?\]|(?<![A-Za-z0-9_./-])([A-Za-z0-9_./-]+\.(?:md|markdown|csv|tsv|txt|json|log|py|ts|tsx|js|rs|rn|toml|ya?ml|html)):L(\d+)(?:-L?(\d+))?/gi;

function CitedText({
  text,
  onOpenFile,
}: {
  text: string;
  onOpenFile?: (path: string) => void;
}) {
  if (!onOpenFile) return <>{text}</>;
  const safe = String(text ?? "");
  const parts: React.ReactNode[] = [];
  let last = 0;
  INLINE_CITATION_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  let idx = 0;
  while ((m = INLINE_CITATION_RE.exec(safe)) !== null) {
    const filePath = (m[1] ?? m[4] ?? "").trim();
    const startLine = Number(m[2] ?? m[5]);
    const endLine = m[3] || m[6] ? Number(m[3] || m[6]) : undefined;
    if (!filePath || filePath.includes("://") || !Number.isFinite(startLine) || startLine < 1) {
      continue;
    }
    if (m.index > last) parts.push(safe.slice(last, m.index));
    const display = `${filePath}:L${startLine}${endLine ? `-L${endLine}` : ""}`;
    const target = `${filePath}#L${startLine}${endLine ? `-L${endLine}` : ""}`;
    parts.push(
      <button
        key={`cit-${idx++}`}
        type="button"
        data-citation={display}
        onClick={() => onOpenFile(target)}
        className="focus-ring mono mx-0.5 inline-flex items-center rounded-chip border border-hairline bg-board/80 px-1.5 py-0.5 text-xs font-medium text-ink underline-offset-2 hover:bg-board hover:underline"
      >
        {display}
      </button>
    );
    last = m.index + m[0].length;
  }
  if (last === 0) return <>{safe}</>;
  if (last < safe.length) parts.push(safe.slice(last));
  return <>{parts}</>;
}

export const FinishedEntry = memo(function FinishedEntry({
  time,
  task,
  steps,
  result,
  notes,
  receiptId,
  personName,
  useServerExport,
  onOpenFile,
  onSaveRoutine,
}: {
  time: string;
  task: string;
  steps: string[];
  result: string;
  notes: string[];
  receiptId?: string;
  personName?: string;
  useServerExport?: boolean;
  onOpenFile?: (path: string) => void;
  onSaveRoutine?: (
    name: string,
    prompt: string,
    schedule?: string,
    playbook?: { steps?: string[]; expected_inputs?: string[]; pack_deliverables?: boolean }
  ) => Promise<boolean>;
}) {
  const [exporting, setExporting] = useState(false);
  const [packing, setPacking] = useState(false);
  const [packPreflight, setPackPreflight] = useState<PackPreflightReport | null>(null);
  const [citationReport, setCitationReport] = useState<CitationReport | null>(null);
  const [officeBusy, setOfficeBusy] = useState(false);
  const [savingRoutine, setSavingRoutine] = useState(false);
  const [routineSaved, setRoutineSaved] = useState(false);
  const files = savedList(steps);
  const hasCitations = extractCitations(result).length > 0;

  useEffect(() => {
    if (!hasCitations) {
      setCitationReport(null);
      return;
    }
    let alive = true;
    verifyCitations(result)
      .then((rep) => {
        if (alive && rep.total > 0) setCitationReport(rep);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [hasCitations, result]);
  const stepWord = steps.length === 1 ? "1 tool step" : `${steps.length} tool steps`;

  const docCandidate = files.find((f) => /\.(md|markdown|txt)$/i.test(f));
  const sheetCandidate = files.find((f) => /\.(csv|tsv)$/i.test(f));

  const handleExport = async (format: "md" | "json") => {
    setExporting(true);
    try {
      const res = await exportReceiptDownload(
        {
          id: receiptId || `fb-${Date.now()}`,
          task: task || "Completed task",
          person: personName || "Active employee",
          status: "finished",
          summary: result,
          files,
          steps,
        },
        format,
        Boolean(useServerExport && receiptId)
      );
      toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
    } catch {
      toast.error("Could not export receipt.");
    } finally {
      setExporting(false);
    }
  };

  const handleOfficeExport = async (path: string, format: "docx" | "xlsx") => {
    setOfficeBusy(true);
    try {
      const r = await call<{ output_path?: string; path?: string }>("/api/files/export", {
        method: "POST",
        body: { path, format },
      });
      if (!r.ok) {
        toast.error(r.error || `Could not export ${format.toUpperCase()}.`);
        return;
      }
      const outPath = r.data.output_path || r.data.path || `${path}.${format}`;
      toast.success(`Exported ${outPath}`);
      if (onOpenFile && outPath) onOpenFile(outPath);
    } finally {
      setOfficeBusy(false);
    }
  };

  const handleCopySummary = async () => {
    const lines = [
      `# ${task || "Completed deliverable"}`,
      "",
      result,
      ...(files.length ? ["", "Saved workspace files:", ...files.map((f) => `- ${f}`)] : []),
    ];
    try {
      await navigator.clipboard.writeText(lines.join("\n"));
      toast.success("Copied client-ready summary.");
    } catch {
      toast.error("Could not access clipboard.");
    }
  };

  return (
    <EntryShell label="Finished work" time={time} title={task || "Completed task"} status={<FinishedPill />}>
      <Cells>
        <Cell label="Execution"><span className="mono">{steps.length ? stepWord : "Direct completion"}</span></Cell>
        <Cell label="Workspace output"><span className="mono">{files.length ? (files.length === 1 ? "1 file saved" : `${files.length} files saved`) : "No new files"}</span></Cell>
      </Cells>
      <FileRow files={files} onOpenFile={onOpenFile} />
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">
          <CitedText text={result} onOpenFile={onOpenFile} />
        </p>
        {citationReport && citationReport.total > 0 && (
          <div className="flex flex-wrap items-center gap-1.5 rounded-sm border border-hairline bg-board/40 px-2.5 py-1.5 text-xs text-ink-2">
            <ShieldCheck className={cn("size-3.5 shrink-0", citationReport.ok ? "text-dot-green" : "text-dot-amber")} strokeWidth={1.75} aria-hidden />
            <span className="font-medium text-ink">
              Citations: {citationReport.verified}/{citationReport.total} verified
            </span>
            {citationReport.citations.slice(0, 6).map((c) => (
              <button
                key={c.raw}
                type="button"
                data-citation={c.raw}
                onClick={() => onOpenFile?.(`${c.path}#L${c.line}`)}
                className={cn(
                  "focus-ring mono rounded-chip border px-1.5 py-0.5 text-xs hover:bg-board",
                  c.status === "verified"
                    ? "border-hairline bg-surface text-ink"
                    : "border-destructive/40 bg-surface text-destructive-foreground"
                )}
              >
                {c.raw}
              </button>
            ))}
          </div>
        )}
        {notes.map((n, idx) => <Note key={idx} text={n} />)}
        <div className="flex flex-wrap items-center gap-2 pt-0.5">
          {docCandidate && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5 text-xs"
              disabled={officeBusy}
              onClick={() => handleOfficeExport(docCandidate, "docx")}
            >
              <FileText className="size-3.5" strokeWidth={1.5} aria-hidden />
              Export Word (.docx)
            </Button>
          )}
          {sheetCandidate && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5 text-xs"
              disabled={officeBusy}
              onClick={() => handleOfficeExport(sheetCandidate, "xlsx")}
            >
              <FileSpreadsheet className="size-3.5" strokeWidth={1.5} aria-hidden />
              Export Excel (.xlsx)
            </Button>
          )}
          <Button
            type="button"
            size="xs"
            variant="outline"
            className="max-md:min-h-11 gap-1.5 text-xs"
            disabled={exporting}
            onClick={() => handleExport("md")}
          >
            <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
            SHA-256 Receipt (.md)
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            className="max-md:min-h-11 gap-1.5 text-xs"
            disabled={exporting}
            onClick={() => handleExport("json")}
          >
            <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
            Receipt (.json)
          </Button>
          {files.length >= 2 && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5 text-xs"
              disabled={packing}
              onClick={async () => {
                setPacking(true);
                try {
                  const [check, res] = await Promise.all([
                    preflightClientPack(files).catch(() => null),
                    exportClientPackDownload(files, task || "Client Deliverable Pack"),
                  ]);
                  if (check) setPackPreflight(check);
                  const checkLabel = check
                    ? `${check.ready_count}/${check.files.length} non-empty`
                    : res.preflight || "verified";
                  toast.success(`Exported ${res.filename} (${checkLabel} · sha256: ${res.sha256.slice(0, 12)}…)`);
                } catch {
                  toast.error("Could not export client pack.");
                } finally {
                  setPacking(false);
                }
              }}
            >
              <PackageCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
              {packing ? "Packing…" : "Client Pack (.md)"}
            </Button>
          )}
          {onSaveRoutine && task.trim() && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              data-action="promote-playbook"
              className="max-md:min-h-11 gap-1.5 text-xs"
              disabled={savingRoutine || routineSaved}
              onClick={async () => {
                setSavingRoutine(true);
                try {
                  const pb = extractPlaybookFromRun(task, steps, files);
                  const ok = await onSaveRoutine(pb.name, pb.prompt, "manual", {
                    steps: pb.steps,
                    expected_inputs: pb.expected_inputs,
                    pack_deliverables: pb.pack_deliverables,
                  });
                  if (ok) setRoutineSaved(true);
                } finally {
                  setSavingRoutine(false);
                }
              }}
            >
              {routineSaved ? (
                <Check className="size-3.5 text-dot-green" strokeWidth={1.75} aria-hidden />
              ) : (
                <Repeat className="size-3.5" strokeWidth={1.5} aria-hidden />
              )}
              {routineSaved ? "Promoted to Playbook" : savingRoutine ? "Saving…" : "Promote to Playbook"}
            </Button>
          )}
          <Button
            type="button"
            size="xs"
            variant="ghost"
            className="max-md:min-h-11 gap-1.5 text-xs text-ink-2 hover:text-ink"
            onClick={() => void handleCopySummary()}
          >
            <Copy className="size-3.5" strokeWidth={1.5} aria-hidden />
            Copy summary
          </Button>
        </div>
        {packPreflight && (
          <div className="rounded-sm border border-hairline bg-board/50 px-3 py-2 text-xs text-ink-2">
            <span className="font-medium text-ink">
              Pre-flight {packPreflight.ok ? "PASS" : "CHECK"}:
            </span>{" "}
            {packPreflight.ready_count} non-empty deliverables verified (
            {packPreflight.files.map((f) => `${f.path}: ${f.count} ${f.unit}`).join(", ")})
          </div>
        )}
        {steps.length > 0 && (
          <Details label={`Inspect tool steps (${steps.length})`}>
            <StepList steps={steps} />
          </Details>
        )}
      </div>
    </EntryShell>
  );
});

export function WaitingEntry({
  time,
  name,
  steps,
  result,
  decisions,
  onResolveDecision,
  onRevise,
  onOpenFile,
  onReview,
}: {
  time: string;
  name: string;
  steps?: string[];
  result: string;
  decisions?: Decision[];
  onResolveDecision?: (id: string, action: "approve" | "deny", label: string) => Promise<void>;
  onRevise?: (prompt: string) => void;
  onOpenFile?: (path: string) => void;
  onReview: () => void;
}) {
  const [acting, setActing] = useState<"approve" | "deny" | null>(null);
  const [diffOpen, setDiffOpen] = useState(false);
  const [diffLoading, setDiffLoading] = useState(false);
  const [diffLines, setDiffLines] = useState<FileDiffLine[] | null>(null);
  const [filePreview, setFilePreview] = useState<string | null>(null);

  const matchedDecision = (decisions ?? []).find((d) => {
    if (d.status && d.status !== "pending") return false;
    const p = (d as { path?: string }).path;
    if (name && p && p === name) return true;
    const hay = `${d.summary ?? ""} ${d.detail ?? ""}`.toLowerCase();
    return name ? hay.includes(name.toLowerCase()) : true;
  }) ?? ((decisions ?? []).find((d) => !d.status || d.status === "pending"));

  const toggleInlineDiff = async () => {
    if (diffOpen) {
      setDiffOpen(false);
      return;
    }
    setDiffOpen(true);
    if (!name || diffLines || filePreview !== null) return;
    setDiffLoading(true);
    try {
      const [contentRes, histRes] = await Promise.all([
        call<WorkspaceFileContent>("/api/files/content?path=" + encodeURIComponent(name)),
        call<{ revisions?: FileRevision[]; history?: FileRevision[] }>(
          "/api/files/history?path=" + encodeURIComponent(name)
        ),
      ]);
      const curText = contentRes.ok ? String(contentRes.data.content ?? "") : "";
      const revs = histRes.ok
        ? (Array.isArray(histRes.data.revisions) ? histRes.data.revisions : histRes.data.history ?? [])
        : [];
      if (revs.length > 0 && typeof revs[0]?.content === "string") {
        const diff = computeLineDiff(revs[0].content, curText);
        setDiffLines(diff.hunks.flatMap((h) => h.lines));
      } else if (curText) {
        setFilePreview(curText.slice(0, 1200));
      } else {
        setFilePreview("(No preview content available for this target.)");
      }
    } finally {
      setDiffLoading(false);
    }
  };

  return (
    <EntryShell label="Waiting on you" time={time} title={name ? `Approve change to ${name}?` : "Waiting on your approval"} status={<WaitingPill />}>
      <Cells>
        <Cell label="Waiting for"><span className="mono">{name || "your approval"}</span></Cell>
        <Cell label="Decision">Approve, hold, or revise</Cell>
      </Cells>
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">{result}</p>

        <div className="flex flex-wrap items-center gap-2">
          {matchedDecision && onResolveDecision ? (
            <>
              <Button
                type="button"
                size="sm"
                className="max-md:min-h-11"
                disabled={acting !== null}
                onClick={async () => {
                  setActing("approve");
                  try {
                    await onResolveDecision(matchedDecision.id, "approve", name || matchedDecision.summary);
                  } finally {
                    setActing(null);
                  }
                }}
              >
                {acting === "approve" ? "Approving…" : "Approve"}
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="max-md:min-h-11"
                disabled={acting !== null}
                onClick={async () => {
                  setActing("deny");
                  try {
                    await onResolveDecision(matchedDecision.id, "deny", name || matchedDecision.summary);
                  } finally {
                    setActing(null);
                  }
                }}
              >
                {acting === "deny" ? "Holding…" : "Hold / Keep"}
              </Button>
            </>
          ) : (
            <Button variant="outline" size="sm" className="max-md:min-h-11" onClick={onReview}>
              Review in Today
            </Button>
          )}
          {onRevise && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={() => onRevise(name ? `Please keep ${name} and revise instead: ` : "Please revise this plan: ")}
            >
              Revise
            </Button>
          )}
          {name && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={toggleInlineDiff}
            >
              {diffOpen ? "Hide file preview" : "Preview diff"}
            </Button>
          )}
          {name && onOpenFile && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={() => onOpenFile(name)}
            >
              Inspect file
            </Button>
          )}
        </div>

        {diffOpen && (
          <div className="max-h-56 overflow-auto rounded-sm border border-hairline bg-board/60 p-3 text-xs leading-5">
            {diffLoading ? (
              <p className="text-ink-2">Loading file preview…</p>
            ) : diffLines && diffLines.length > 0 ? (
              <div className="font-mono">
              {diffLines.slice(0, 80).map((line, i) => (
                <div
                  key={i}
                  className={cn(
                    "whitespace-pre-wrap break-words px-1",
                    line.kind === "add" && "bg-pill-active-bg text-pill-active-fg",
                    line.kind === "del" && "bg-status-stopped-bg text-status-stopped-fg"
                  )}
                >
                  {line.kind === "add" ? "+ " : line.kind === "del" ? "- " : "  "}
                  {line.text}
                </div>
              ))}
              </div>
            ) : (
              <pre className="font-mono whitespace-pre-wrap break-words text-ink-2">{filePreview || "(empty file)"}</pre>
            )}
          </div>
        )}

        {steps && steps.length > 0 && (
          <Details label={`Inspect tool steps (${steps.length})`}>
            <StepList steps={steps} />
          </Details>
        )}
      </div>
    </EntryShell>
  );
}

export function Reply({ text, onOpenFile }: { text: string; onOpenFile?: (path: string) => void }) {
  return (
    <div className="rounded-card border border-hairline bg-surface px-4 py-3">
      <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">
        <CitedText text={text} onOpenFile={onOpenFile} />
      </p>
    </div>
  );
}

export function Failure({ raw }: { raw: string }) {
  return (
    <div role="alert" className="rounded-entry border border-hairline bg-surface px-4 py-3.5">
      <p className="text-sm font-medium text-destructive-foreground">The agent could not finish this.</p>
      <p className="mt-1 text-sm text-ink-2">Try again, or ask whoever runs Ergon to check the model connection.</p>
      <div className="mt-3">
        <Details label="Details">
          <pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-3">{raw}</pre>
        </Details>
      </div>
    </div>
  );
}

export function UserLine({ text }: { text: string }) {
  return (
    <div className="flex justify-end">
      <p className="max-w-[84%] whitespace-pre-wrap break-words rounded-card border border-hairline bg-board px-4 py-2.5 text-ui leading-6 text-ink">{text}</p>
    </div>
  );
}

export function savedList(steps: string[]): string[] {
  const out: string[] = [];
  for (const s of steps) {
    const str = String(s ?? "");
    const nl = str.indexOf("\n");
    const head = nl < 0 ? str : str.slice(0, nl);
    const m = /^Tool: (\S+)/.exec(head);
    const f = m ? savedFile(m[1], nl < 0 ? "" : str.slice(nl + 1)) : "";
    if (f && !out.includes(f)) out.push(f);
  }
  return out;
}

export function savedOf(steps: string[]): string {
  return savedList(steps)[0] ?? "";
}

export type Item =
  | { k: "user"; text: string }
  | { k: "step"; text: string }
  | { k: "finished"; text: string; task: string; steps: string[]; notes: string[] }
  | { k: "waiting"; text: string; name: string; task: string; steps: string[]; notes: string[] }
  | { k: "reply"; text: string }
  | { k: "error"; text: string };

const PARKED = /parked for your person|has NOT been deleted yet/;
const RESOLVED = /was deleted after you approved it|was kept because you declined/;

export function buildItems(messages: Msg[]): Item[] {
  const safe = Array.isArray(messages) ? messages : [];
  const out: Item[] = [];
  let seg: Msg[] = [];
  let task = "";
  const flush = () => {
    const hasStep = seg.some((m) => m.role === "step" && !/^Note:/.test(String(m.content ?? "")));
    const waiting = seg.some((m) => m.role === "step" && PARKED.test(String(m.content ?? "")))
      && !seg.some((m) => m.role === "step" && RESOLVED.test(String(m.content ?? "")));
    let name = "";
    for (const m of seg) {
      const hit = m.role === "step" && /(\S+) has NOT been deleted yet/.exec(String(m.content ?? ""));
      if (hit) { name = hit[1]; break; }
    }
    let pending: string[] = [];
    let card: Item | null = null;
    for (const m of seg) {
      const content = String(m.content ?? "");
      if (m.role === "step" && /^Note:/.test(content) && card) {
        if (!(waiting && /has NOT been deleted yet/.test(content)) && (card.k === "finished" || card.k === "waiting")) {
          card.notes.push(content);
        }
      } else if (m.role === "step") {
        pending.push(content);
        card = null;
      } else if (m.role === "assistant") {
        const steps = pending;
        pending = [];
        card = waiting
          ? { k: "waiting", text: content, name, task, steps, notes: [] }
          : hasStep
            ? { k: "finished", text: content, task, steps, notes: [] }
            : { k: "reply", text: content };
        out.push(card);
      } else {
        card = null;
        out.push({ k: m.role === "error" ? "error" : "reply", text: content });
      }
    }
    for (const s of pending) out.push({ k: "step", text: s });
    seg = [];
  };
  for (const m of safe) {
    if (!m || typeof m !== "object") continue;
    const content = String(m.content ?? "");
    if (m.role === "user") { flush(); task = content; out.push({ k: "user", text: content }); }
    else seg.push({ role: String(m.role ?? "reply"), content });
  }
  flush();
  return out;
}

export function threadState(messages: Msg[]): "waiting" | "finished" | "none" {
  const safe = Array.isArray(messages) ? messages : [];
  if (safe.some((m) => m?.role === "step" && PARKED.test(String(m.content ?? "")))
    && !safe.some((x) => x?.role === "step" && RESOLVED.test(String(x.content ?? "")))) {
    return "waiting";
  }
  return safe.some((m) => m?.role === "assistant") ? "finished" : "none";
}
