// Signature object: the Field-Book Work Entry.
// Ink head strip (timestamp in Geist Mono, task title, semantic status pill), ruled metadata grid,
// workspace saved-file rows with file-type glyphs, summary prose, one-click SHA-256 receipt export,
// and Cult UI Expandable raw tool-step inspector.
import { useState } from "react";
import { ArrowUpRight, Check, ChevronDown, FileDown, Hourglass } from "lucide-react";
import { toast } from "sonner";
import { Expandable, ExpandableContent, ExpandableTrigger } from "@/components/ui/expandable";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { cn } from "@/lib/utils";
import { exportReceiptDownload } from "@/lib/receiptExport";
import { fileIconFor, savedFile, toolLabel } from "@/lib/words";
import type { Msg } from "@/lib/api";

export function FileGlyph({ name, className }: { name: string; className?: string }) {
  const Icon = fileIconFor(name);
  return <Icon className={cn("size-4 shrink-0 text-ink-2", className)} strokeWidth={1.5} aria-hidden="true" />;
}

/** Disclosure key backed by Cult UI Expandable (ExpandableTrigger manages aria-expanded and keyboard Enter/Space). */
export function Details({ label = "Details", children }: { label?: string; children: React.ReactNode }) {
  return (
    <Expandable expandDirection="vertical" expandBehavior="push" className="w-full">
      {({ isExpanded }) => (
        <div className="w-full">
          <ExpandableTrigger
            aria-label={label}
            className="mould focus-ring flex min-h-11 w-full items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-3 text-left text-sm font-medium text-ink outline-none md:min-h-9"
          >
            <span>{label}</span>
            <ChevronDown
              className={cn(
                "size-4 shrink-0 text-icon motion-safe:transition-transform motion-safe:duration-(--dur-swap) motion-safe:ease-(--ease-house)",
                isExpanded && "rotate-180"
              )}
              strokeWidth={1.5}
              aria-hidden
            />
          </ExpandableTrigger>
          <ExpandableContent preset="fade">
            <div className="mt-2 flex flex-col gap-2.5">{children}</div>
          </ExpandableContent>
        </div>
      )}
    </Expandable>
  );
}

export function EntryShell({ time, title, status, label, children }: {
  time: string;
  title: string;
  status?: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <article aria-label={label} className="overflow-hidden rounded-entry border border-ink bg-surface shadow-card dark:border-hairline dark:shadow-none">
      <header className="entry-head flex min-w-0 flex-wrap items-center gap-2.5 border-b border-ink px-4 py-2.5 dark:border-hairline">
        {time && <span className="mono shrink-0 text-xs">{time}</span>}
        <span className="min-w-0 flex-1 truncate text-sm font-medium">{title}</span>
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
    <div className="flex min-w-0 flex-wrap items-center justify-between gap-2 border-b border-hairline bg-board/50 px-4 py-2.5">
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        {files.map((name, idx) => (
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
              <span className="text-ink-2">Open</span>
              <ArrowUpRight className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
            </Button>
          ) : (
            <Pill key={name + idx} variant="secondary" className="max-w-full border border-hairline bg-surface px-2.5 py-1 text-xs font-normal text-ink">
              <FileGlyph name={name} />
              <span className="mono min-w-0 truncate">{name}</span>
            </Pill>
          )
        ))}
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

export function FinishedEntry({ time, task, steps, result, notes, receiptId, personName, useServerExport, onOpenFile }: {
  time: string;
  task: string;
  steps: string[];
  result: string;
  notes: string[];
  receiptId?: string;
  personName?: string;
  useServerExport?: boolean;
  onOpenFile?: (path: string) => void;
}) {
  const [exporting, setExporting] = useState(false);
  const files = savedList(steps);
  const stepWord = steps.length === 1 ? "1 tool step" : `${steps.length} tool steps`;

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

  return (
    <EntryShell label="Finished work" time={time} title={task || "Completed task"} status={<FinishedPill />}>
      <Cells>
        <Cell label="Execution"><span className="mono">{steps.length ? stepWord : "Direct completion"}</span></Cell>
        <Cell label="Workspace output"><span className="mono">{files.length ? (files.length === 1 ? "1 file saved" : `${files.length} files saved`) : "No new files"}</span></Cell>
      </Cells>
      <FileRow files={files} onOpenFile={onOpenFile} />
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">{result}</p>
        {notes.map((n, idx) => <Note key={idx} text={n} />)}
        <div className="flex flex-wrap items-center gap-2 pt-0.5">
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
        </div>
        {steps.length > 0 && (
          <Details label={`Inspect tool steps (${steps.length})`}>
            <StepList steps={steps} />
          </Details>
        )}
      </div>
    </EntryShell>
  );
}

export function WaitingEntry({ time, name, steps, result, onReview }: {
  time: string;
  name: string;
  steps?: string[];
  result: string;
  onReview: () => void;
}) {
  return (
    <EntryShell label="Waiting on you" time={time} title={"Delete " + (name || "a file") + "?"} status={<WaitingPill />}>
      <Cells>
        <Cell label="Waiting for"><span className="mono">{name || "your approval"}</span></Cell>
        <Cell label="Decision">Approve or keep</Cell>
      </Cells>
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">{result}</p>
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="outline" size="sm" className="max-md:min-h-11" onClick={onReview}>Review in Today</Button>
        </div>
        {steps && steps.length > 0 && (
          <Details label={`Inspect tool steps (${steps.length})`}>
            <StepList steps={steps} />
          </Details>
        )}
      </div>
    </EntryShell>
  );
}

export function Reply({ text }: { text: string }) {
  return (
    <div className="rounded-card border border-hairline bg-surface px-4 py-3">
      <p className="whitespace-pre-wrap break-words text-ui leading-6 text-ink">{text}</p>
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
