// Chat face: active Field-Book conversation thread, live SSE BorderTrail + TextShimmer step timeline
// with elapsed ms & active cancellation, rich role-tailored Starter Task Cards, and pinned task composer
// with drag-and-drop multi-file workspace upload.
import { useEffect, useRef, useState } from "react";
import { ArrowRight, ArrowUpRight, Check, Clock, Paperclip, Radio, Send, Sparkles, Square, XCircle } from "lucide-react";
import { BorderTrail } from "@/components/ui/border-trail";
import { TextShimmer } from "@/components/ui/text-shimmer";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { InputGroup, InputGroupAddon, InputGroupTextarea } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Pill } from "@/components/kibo-ui/pill";
import {
  Failure,
  FileGlyph,
  FinishedEntry,
  FinishedPill,
  type Item,
  LedgerRow,
  Reply,
  UserLine,
  WaitingEntry,
  buildItems,
} from "./Entry";
import type { LiveRunStep, Msg, WorkspaceFile } from "@/lib/api";
import { normalizeFiles, relTime, type StarterCard, threadMs } from "@/lib/words";
import { cn } from "@/lib/utils";

export function greeting(name: string, hour: number): string {
  const part = hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";
  return name ? `${part}, ${name}.` : `${part}.`;
}

export type Recent = { task: string; date: string; files?: string[] };

const TAG_TONE: Record<StarterCard["tone"], string> = {
  ops: "bg-tag-ops-bg text-tag-ops-fg border-transparent",
  mint: "bg-tag-mint-bg text-tag-mint-fg border-transparent",
  plan: "bg-tag-plan-bg text-tag-plan-fg border-transparent",
  research: "bg-tag-research-bg text-tag-research-fg border-transparent",
};

function formatElapsed(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "0ms";
  if (ms < 1000) return `${Math.round(ms)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function LiveRunTimeline({
  steps,
  elapsedMs,
  streamMode,
  cancelling,
  onCancel,
}: {
  steps: LiveRunStep[];
  elapsedMs: number;
  streamMode: "sse" | "poll";
  cancelling: boolean;
  onCancel: () => void;
}) {
  return (
    <div id="live-run-timeline" role="status" className="relative overflow-hidden rounded-card border border-hairline bg-surface px-4 py-3.5">
      <BorderTrail size={56} />
      <div className="flex flex-wrap items-center justify-between gap-2 text-sm font-medium text-ink">
        <span className="flex items-center gap-2.5">
          <span aria-hidden className="size-2 rounded-full bg-blue motion-safe:animate-pulse" />
          <TextShimmer as="span">Agent is working on it</TextShimmer>
        </span>
        <div className="flex flex-wrap items-center gap-2">
          <Pill variant="secondary" className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2">
            <Radio className="size-3 text-dot-green" strokeWidth={1.75} aria-hidden />
            <span className="mono">{streamMode === "sse" ? "Live SSE" : "Live status"}</span>
          </Pill>
          <span className="mono text-xs text-ink-2">{formatElapsed(elapsedMs)}</span>
          {steps.length > 0 && (
            <span className="mono text-xs text-ink-2">
              {steps.length} {steps.length === 1 ? "step" : "steps"}
            </span>
          )}
          <Button
            id="cancel-run-btn"
            type="button"
            size="xs"
            variant="outline"
            disabled={cancelling}
            className="max-md:min-h-11 gap-1.5 text-xs"
            onClick={onCancel}
          >
            <Square className="size-3 fill-current" strokeWidth={1.5} aria-hidden />
            {cancelling ? "Cancelling…" : "Cancel run"}
          </Button>
        </div>
      </div>

      {steps.length === 0 ? (
        <p className="mt-1.5 text-xs text-ink-2">Streaming sandboxed workspace tools and verifying written files…</p>
      ) : (
        <ol aria-label="Live tool progress" className="mt-2.5 flex flex-col gap-1.5 border-t border-hairline pt-2.5">
          {steps.map((s, idx) => (
            <li key={s.id || idx} className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink">
              <span className="flex min-w-0 items-center gap-2">
                {s.status === "running" ? (
                  <span aria-hidden className="size-2 shrink-0 rounded-full bg-blue motion-safe:animate-pulse" />
                ) : s.status === "waiting" ? (
                  <Clock className="size-3.5 shrink-0 text-dot-amber" strokeWidth={1.75} aria-hidden />
                ) : s.status === "error" ? (
                  <XCircle className="size-3.5 shrink-0 text-dot-red" strokeWidth={1.75} aria-hidden />
                ) : (
                  <Check className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.75} aria-hidden />
                )}
                <span className="truncate">{s.label || s.tool || "Running step"}</span>
              </span>
              <span className="flex shrink-0 items-center gap-1.5">
                {s.tool && <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-ink-2">{s.tool}</span>}
                {typeof s.elapsed_ms === "number" && (
                  <span className="mono text-ink-2">{formatElapsed(s.elapsed_ms)}</span>
                )}
              </span>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

export function Thread({
  messages,
  busy,
  liveTimeline,
  liveElapsedMs,
  streamMode,
  cancelling,
  onCancel,
  threadId,
  personName,
  useServerExport,
  onReview,
  onStarter,
  onOpenFile,
  starters,
  status,
  pendingNote,
  name,
  recent,
}: {
  messages: Msg[];
  busy: boolean;
  liveTimeline: LiveRunStep[];
  liveElapsedMs: number;
  streamMode: "sse" | "poll";
  cancelling: boolean;
  onCancel: () => void;
  threadId: string | null;
  personName?: string;
  useServerExport?: boolean;
  onReview: () => void;
  onStarter: (text: string) => void;
  onOpenFile: (path: string) => void;
  starters: StarterCard[];
  status: "ok" | "wait";
  pendingNote: boolean;
  name: string;
  recent: Recent[];
}) {
  const items = buildItems(messages);
  const ms = threadMs(threadId ?? "");
  const time = ms ? relTime(ms) : "";
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (items.length || busy) endRef.current?.scrollIntoView({ block: "end" });
  }, [items.length, busy, liveTimeline.length]);

  if (!items.length && !busy) {
    return (
      <Empty
        status={status}
        starters={starters}
        onStarter={onStarter}
        onOpenFile={onOpenFile}
        name={name}
        recent={recent}
      />
    );
  }

  return (
    <div className="mx-auto flex w-full max-w-[780px] flex-col gap-5">
      {items.map((it, i) => (
        <Row
          key={i}
          it={it}
          time={time}
          receiptId={threadId || undefined}
          personName={personName}
          useServerExport={useServerExport}
          onReview={onReview}
          onOpenFile={onOpenFile}
        />
      ))}
      {busy && (
        <LiveRunTimeline
          steps={liveTimeline}
          elapsedMs={liveElapsedMs}
          streamMode={streamMode}
          cancelling={cancelling}
          onCancel={onCancel}
        />
      )}
      {pendingNote && (
        <p id="pending-note" role="status" className="rounded-card border border-hairline bg-board px-3.5 py-2.5 text-sm text-ink-2">
          Your last message did not go through. It is back in the box below.
        </p>
      )}
      <div ref={endRef} />
    </div>
  );
}

function Row({ it, time, receiptId, personName, useServerExport, onReview, onOpenFile }: {
  it: Item;
  time: string;
  receiptId?: string;
  personName?: string;
  useServerExport?: boolean;
  onReview: () => void;
  onOpenFile: (path: string) => void;
}) {
  switch (it.k) {
    case "user": return <UserLine text={it.text} />;
    case "step": return <ol className="rounded-card border border-hairline bg-surface px-3.5"><LedgerRow raw={it.text} /></ol>;
    case "finished":
      return (
        <FinishedEntry
          time={time}
          task={it.task}
          steps={it.steps}
          result={it.text}
          notes={it.notes}
          receiptId={receiptId}
          personName={personName}
          useServerExport={useServerExport}
          onOpenFile={onOpenFile}
        />
      );
    case "waiting": return <WaitingEntry time={time} name={it.name} steps={it.steps} result={it.text} onReview={onReview} />;
    case "error": return <Failure raw={it.text} />;
    default: return <Reply text={it.text} />;
  }
}

/** Static topographic survey contour field drawn from --hairline (zero gradient, zero motion). */
function contours(): string[] {
  const out: string[] = [];
  for (let i = 1; i <= 8; i++) {
    const R = 44 + i * 36;
    let d = "";
    for (let k = 0; k <= 72; k++) {
      const t = (k / 72) * 2 * Math.PI;
      const r = R * (1 + 0.08 * Math.sin(3 * t + i * 0.7) + 0.04 * Math.cos(5 * t - i));
      d += (k ? "L" : "M") + (500 + r * Math.cos(t) * 1.65).toFixed(1) + " " + (250 + r * Math.sin(t) * 0.72).toFixed(1);
    }
    out.push(d + "Z");
  }
  return out;
}
const CONTOURS = contours();

export function ContourField() {
  return (
    <svg aria-hidden viewBox="0 0 1000 560" preserveAspectRatio="xMidYMid slice" className="contour-field pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-[0.3]">
      {CONTOURS.map((d, i) => <path key={i} d={d} fill="none" stroke="currentColor" strokeWidth={i % 4 === 0 ? 1.3 : 0.85} />)}
    </svg>
  );
}

function Empty({ status, starters, onStarter, onOpenFile, name, recent }: {
  status: "ok" | "wait";
  starters: StarterCard[];
  onStarter: (t: string) => void;
  onOpenFile: (path: string) => void;
  name: string;
  recent: Recent[];
}) {
  const sub = status === "ok"
    ? "State the outcome in plain words. Your agent runs real workspace tools and files a verified Field-Book receipt here."
    : "Ergon is not connected to a model yet. Anything you type is kept safely in this conversation.";

  const pickStarter = (prompt: string) => {
    onStarter(prompt);
    setTimeout(() => document.getElementById("composer-input")?.focus(), 0);
  };

  return (
    <section aria-label="Start a task" className="mx-auto flex w-full max-w-[780px] flex-col gap-6 pt-2">
      <div className="flex flex-col gap-2">
        <h2 className="text-[32px] leading-[1.1] font-medium tracking-[-0.03em] text-ink md:text-[38px]">
          {greeting(name, new Date().getHours())}
        </h2>
        <p className="max-w-[62ch] text-ui leading-6 text-ink-2">
          What should your agent finish for you? {sub}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {starters.map((card) => (
          <Button
            key={card.prompt}
            data-starter-card=""
            variant="ghost"
            onClick={() => pickStarter(card.prompt)}
            className="mould h-auto sm:h-auto min-h-34 w-full flex-col items-stretch justify-between gap-3 whitespace-normal rounded-card border border-hairline bg-surface p-4 text-left font-normal hover:bg-board"
          >
            <span className="flex w-full items-center justify-between gap-2">
              <Badge size="sm" className={cn("px-2 py-0.5 text-xs font-medium", TAG_TONE[card.tone])}>
                {card.category}
              </Badge>
              <Pill variant="secondary" className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2">
                <Sparkles className="size-3 text-icon" strokeWidth={1.5} aria-hidden />
                <span className="mono">{card.skill}</span>
              </Pill>
            </span>
            <span className="flex flex-col gap-1">
              <span className="text-ui font-medium text-ink">{card.prompt}</span>
              <span className="text-sm leading-5 text-ink-2">{card.detail}</span>
            </span>
            <span className="flex w-full items-center justify-between border-t border-hairline pt-2.5 text-xs font-medium text-ink-2">
              <span>Fill task box</span>
              <ArrowRight className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            </span>
          </Button>
        ))}
      </div>

      {recent.length > 0 && (
        <div className="flex flex-col gap-2.5 pt-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-ink">Finished recently</h3>
            <span className="mono text-xs text-ink-3">{recent.length} {recent.length === 1 ? "entry" : "entries"}</span>
          </div>
          <div className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
            {recent.slice(0, 3).map((r, i) => {
              const files = normalizeFiles(r.files);
              return (
                <div key={i} className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-ink">{r.task}</p>
                    <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-ink-2">
                      <span className="mono">{r.date}</span>
                      {files.map((f, idx) => (
                        <Button
                          key={f + idx}
                          size="xs"
                          variant="outline"
                          aria-label={`Open ${f}`}
                          onClick={() => onOpenFile(f)}
                          className="mono max-md:min-h-11 max-w-full gap-1 rounded-chip border-hairline bg-board/60 px-2 text-xs font-normal text-ink"
                        >
                          <FileGlyph name={f} className="size-3.5" />
                          <span className="truncate">{f}</span>
                          <ArrowUpRight className="size-3 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                        </Button>
                      ))}
                    </div>
                  </div>
                  <FinishedPill />
                </div>
              );
            })}
          </div>
        </div>
      )}
    </section>
  );
}

export function Composer({
  value,
  onChange,
  onSend,
  busy,
  disabled,
  files,
  onUploadMany,
  onOpenFile,
}: {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  busy: boolean;
  disabled: boolean;
  files: WorkspaceFile[];
  onUploadMany: (picked: FileList | File[]) => Promise<number>;
  onOpenFile: (path: string) => void;
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
  const uploadRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);
  const canSend = !disabled && !!value.trim();

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  }, [value]);

  const handleFiles = async (ev: React.ChangeEvent<HTMLInputElement>) => {
    const picked = ev.target.files;
    if (picked && picked.length) await onUploadMany(picked);
    ev.target.value = "";
  };

  return (
    <form
      id="composer"
      aria-label="Send a task"
      className="mx-auto w-full max-w-[780px]"
      onSubmit={(e) => { e.preventDefault(); onSend(); }}
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
        if (e.dataTransfer?.files?.length) await onUploadMany(e.dataTransfer.files);
      }}
    >
      {files.length > 0 && (
        <div className="mb-2 flex flex-wrap items-center gap-1.5">
          <span className="text-xs text-ink-3">Workspace:</span>
          {files.slice(0, 4).map((f) => (
            <Button
              key={f.path}
              type="button"
              size="xs"
              variant="outline"
              aria-label={`Open ${f.path}`}
              onClick={() => onOpenFile(f.path)}
              className="mono max-md:min-h-11 max-w-[200px] gap-1 rounded-chip border-hairline bg-surface px-2 text-xs font-normal text-ink"
            >
              <FileGlyph name={f.path} className="size-3.5" />
              <span className="truncate">{f.path}</span>
            </Button>
          ))}
          {files.length > 4 && (
            <span className="mono text-xs text-ink-3">+{files.length - 4} more</span>
          )}
        </div>
      )}
      <InputGroup
        className={cn(
          "min-h-14 items-end rounded-card bg-surface shadow-raised",
          dragOver ? "border-blue bg-blue-soft" : "border-input"
        )}
      >
        <input
          ref={uploadRef}
          id="composer-file-input"
          type="file"
          multiple
          className="hidden"
          accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
          onChange={handleFiles}
        />
        <InputGroupAddon align="inline-start" className="pb-2 pl-2">
          <Button
            type="button"
            size="icon-sm"
            variant="ghost"
            disabled={disabled}
            aria-label="Upload file to workspace"
            className="min-h-11 min-w-11 md:min-h-9 md:min-w-9"
            onClick={() => uploadRef.current?.click()}
          >
            <Paperclip className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          </Button>
        </InputGroupAddon>
        <label htmlFor="composer-input" className="sr-only">Describe a task for your agent</label>
        <InputGroupTextarea
          id="composer-input"
          ref={ref}
          rows={1}
          value={value}
          disabled={disabled}
          placeholder={dragOver ? "Drop files to add to your workspace…" : "Describe a task in plain words"}
          className="max-h-[200px] min-h-14 py-3.5 text-ui"
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              onSend();
            }
          }}
        />
        <InputGroupAddon align="inline-end" className="pb-2 pr-2">
          <Button
            id="send-btn"
            type="submit"
            size="lg"
            variant={canSend ? "default" : "outline"}
            disabled={!canSend}
            className={cn("min-h-11", canSend ? "shadow-primary" : "shadow-none")}
          >
            <Send className="size-4" strokeWidth={1.5} aria-hidden />
            {busy ? "Working…" : "Send"}
          </Button>
        </InputGroupAddon>
      </InputGroup>
      <p className="mt-2 flex flex-wrap items-center gap-1.5 text-xs text-ink-2">
        <span>Press <Kbd>Enter</Kbd> to send or drop files here.</span>
        <span className="max-sm:hidden"><Kbd>Shift</Kbd> <Kbd>Enter</Kbd> starts a new line.</span>
        <span className="ml-auto max-sm:hidden"><Kbd>⌘K</Kbd> command palette</span>
      </p>
    </form>
  );
}
