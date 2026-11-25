// Claude-Desktop central conversation desk:
// - Serene centered thread (max-w-3xl) with generous vertical rhythm
// - 4 1-click Business Deliverable Starter cards when empty
// - Inline Approval & File-Diff cards directly inside the thread (Approve / Hold / Revise)
// - Live SSE / poll run status banner (#live-run-timeline) + cancel (#cancel-run-btn)
// - Pinned composer (#composer-input) with multi-file upload & drag-drop
import { useEffect, useRef, useState } from "react";
import { Check, Clock, FileCheck2, FolderOpen, Paperclip, Send, Sparkles, Square, XCircle } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  buildItems,
  Failure,
  FinishedEntry,
  Reply,
  UserLine,
  WaitingEntry,
} from "@/chat/Entry";
import { cn } from "@/lib/utils";
import type { Decision, LiveRunStep, Msg } from "@/lib/api";
import { type StarterCard, toolLabel } from "@/lib/words";

export type Recent = { task: string; date: string; files: string[] };

function fmtElapsed(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "0.0s";
  if (ms < 1000) return `${ms}ms`;
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
  streamMode: "sse" | "poll" | null;
  cancelling: boolean;
  onCancel: () => void;
}) {
  return (
    <div
      id="live-run-timeline"
      role="status"
      aria-live="polite"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2.5">
          <span className="pulse-dot size-2.5 rounded-full bg-blue" aria-hidden />
          <span className="text-sm font-medium text-ink">Agent is working on it</span>
          {streamMode && (
            <span className="mono rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">
              {streamMode.toUpperCase()}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <span className="mono flex items-center gap-1 text-xs text-ink-2">
            <Clock className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            {fmtElapsed(elapsedMs)}
          </span>
          <Button
            id="cancel-run-btn"
            type="button"
            size="xs"
            variant="outline"
            disabled={cancelling}
            onClick={onCancel}
            className="max-md:min-h-11 gap-1.5"
          >
            <Square className="size-3" strokeWidth={1.75} aria-hidden />
            {cancelling ? "Stopping…" : "Stop"}
          </Button>
        </div>
      </div>

      {steps.length > 0 && (
        <ul className="flex flex-col gap-1.5 border-t border-hairline pt-2.5 text-sm">
          {steps.map((st, idx) => (
            <li key={(st.id || "step") + idx} className="flex items-center justify-between gap-2">
              <span className="flex min-w-0 items-center gap-2 text-ink">
                {st.status === "error" ? (
                  <XCircle className="size-3.5 shrink-0 text-dot-red" strokeWidth={1.75} aria-hidden />
                ) : st.status === "done" ? (
                  <Check className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.75} aria-hidden />
                ) : (
                  <span className="pulse-dot size-2 shrink-0 rounded-full bg-blue" aria-hidden />
                )}
                <span className="truncate">{st.label || toolLabel(st.tool, undefined, st.raw) || st.raw}</span>
              </span>
              {typeof st.elapsed_ms === "number" && st.elapsed_ms > 0 && (
                <span className="mono shrink-0 text-xs text-ink-3">{fmtElapsed(st.elapsed_ms)}</span>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function Chat({
  threadId,
  personName,
  role,
  messages,
  decisions,
  busy,
  cancelling,
  liveElapsedMs,
  streamMode,
  liveSteps,
  liveTimeline,
  draft,
  setDraft,
  pendingNote,
  starters,
  onSend,
  onCancel,
  onReview,
  onResolveDecision,
  onOpenFile,
  onOpenFiles,
  onExportAuditBundle,
  onUploadFiles,
  inputRef,
}: {
  threadId: string | null;
  personName: string;
  role: string;
  messages: Msg[];
  decisions: Decision[];
  busy: boolean;
  cancelling: boolean;
  liveElapsedMs: number;
  streamMode: "sse" | "poll" | null;
  liveSteps: string[];
  liveTimeline: LiveRunStep[];
  draft: string;
  setDraft: (s: string) => void;
  pendingNote: boolean;
  starters: StarterCard[];
  onSend: () => void;
  onCancel: () => void;
  onReview: () => void;
  onResolveDecision: (id: string, action: "approve" | "deny", label: string) => Promise<void>;
  onOpenFile: (path: string) => void;
  onOpenFiles: () => void;
  onExportAuditBundle: (format: "json" | "md") => Promise<void>;
  onUploadFiles: (files: FileList | File[]) => Promise<number>;
  inputRef: React.RefObject<HTMLTextAreaElement | null>;
}) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const attachRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const items = buildItems(messages);
  const cards = starters;

  useEffect(() => {
    if (items.length > 0 || busy) {
      bottomRef.current?.scrollIntoView({ behavior: "auto" });
    }
  }, [items.length, busy, liveSteps.length]);

  const handleFiles = async (list: FileList | File[] | null) => {
    if (!list || !list.length) return;
    const ok = await onUploadFiles(list);
    if (ok > 0) toast.success(`Uploaded ${ok} ${ok === 1 ? "file" : "files"} to workspace.`);
  };

  const handleRevise = (prefix: string) => {
    setDraft(prefix);
    requestAnimationFrame(() => {
      const el = inputRef.current;
      if (el) {
        el.focus();
        el.setSelectionRange(prefix.length, prefix.length);
      }
    });
  };

  return (
    <div
      className="flex h-full min-h-0 flex-col"
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
        if (e.dataTransfer?.files?.length) await handleFiles(e.dataTransfer.files);
      }}
    >
      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-6 sm:px-6">
        <div className="mx-auto flex w-full max-w-[780px] flex-col gap-5">
          {items.length === 0 && !busy ? (
            <div className="flex flex-col gap-6 pt-4 sm:pt-8">
              <div className="flex flex-col gap-2">
                <p className="text-xs font-medium text-ink-2" aria-label={`Working for ${personName}`}>
                  Working for {personName}{role ? ` (${role})` : ""}
                </p>
                <h1 className="text-title font-medium tracking-tight text-ink sm:text-[24px] sm:leading-8">
                  What would you like finished today?
                </h1>
                <p className="max-w-xl text-sm leading-6 text-ink-2">
                  Pick a deliverable workflow below, drop spreadsheets or documents here, or describe a task in plain words.
                </p>
                <div className="flex flex-wrap items-center gap-2 pt-1">
                  <Button
                    type="button"
                    size="xs"
                    variant="outline"
                    className="max-md:min-h-11 gap-1.5"
                    onClick={onOpenFiles}
                  >
                    <FolderOpen className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Browse workspace files
                  </Button>
                  <Button
                    type="button"
                    size="xs"
                    variant="outline"
                    className="max-md:min-h-11 gap-1.5"
                    onClick={() => void onExportAuditBundle("md")}
                  >
                    <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Export Audit Bundle (.md)
                  </Button>
                </div>
              </div>

              <section aria-label="Start a task" className="grid gap-3 sm:grid-cols-2">
                {cards.map((card, i) => (
                  <button
                    key={card.prompt + i}
                    type="button"
                    data-starter-card
                    onClick={() => {
                      setDraft(card.prompt);
                      requestAnimationFrame(() => inputRef.current?.focus());
                    }}
                    className="focus-ring flex min-h-11 flex-col items-start justify-between gap-2 rounded-card border border-hairline bg-surface p-4 text-left outline-none motion-safe:transition-[background-color,border-color] motion-safe:duration-(--dur-press) hover:border-ink-3/40 hover:bg-board"
                  >
                    <div className="flex w-full items-center justify-between gap-2">
                      <span
                        className={cn(
                          "rounded-chip px-2 py-0.5 text-xs font-medium",
                          card.tone === "ops" && "bg-tag-ops-bg text-tag-ops-fg",
                          card.tone === "mint" && "bg-tag-mint-bg text-tag-mint-fg",
                          card.tone === "plan" && "bg-tag-plan-bg text-tag-plan-fg",
                          card.tone === "research" && "bg-tag-research-bg text-tag-research-fg"
                        )}
                      >
                        {card.category}
                      </span>
                      <Sparkles className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                    </div>
                    <div className="flex flex-col gap-1">
                      <span className="text-sm font-medium text-ink">{card.prompt}</span>
                      <span className="text-xs leading-5 text-ink-2">{card.detail}</span>
                    </div>
                  </button>
                ))}
              </section>
            </div>
          ) : (
            <div id="thread" className="flex flex-col gap-4" aria-live="polite">
              {items.map((item, idx) => {
                if (item.k === "user") return <UserLine key={idx} text={item.text} />;
                if (item.k === "finished") {
                  return (
                    <FinishedEntry
                      key={idx}
                      time=""
                      task={item.task}
                      steps={item.steps}
                      result={item.text}
                      notes={item.notes}
                      receiptId={threadId ?? undefined}
                      personName={personName}
                      onOpenFile={onOpenFile}
                    />
                  );
                }
                if (item.k === "waiting") {
                  return (
                    <WaitingEntry
                      key={idx}
                      time=""
                      name={item.name}
                      steps={item.steps}
                      result={item.text}
                      decisions={decisions}
                      onResolveDecision={onResolveDecision}
                      onRevise={handleRevise}
                      onOpenFile={onOpenFile}
                      onReview={onReview}
                    />
                  );
                }
                if (item.k === "error") return <Failure key={idx} raw={item.text} />;
                if (item.k === "step") {
                  return (
                    <div key={idx} className="rounded-sm border border-hairline bg-board/50 px-3 py-2 text-xs text-ink-2">
                      {item.text}
                    </div>
                  );
                }
                return <Reply key={idx} text={item.text} />;
              })}
            </div>
          )}

          {busy && (
            <LiveRunTimeline
              steps={liveTimeline}
              elapsedMs={liveElapsedMs}
              streamMode={streamMode}
              cancelling={cancelling}
              onCancel={onCancel}
            />
          )}

          <div ref={bottomRef} />
        </div>
      </div>

      <div className="safe-b shrink-0 border-t border-hairline bg-surface px-4 py-3 sm:px-6">
        <div className="mx-auto flex w-full max-w-[780px] flex-col gap-2">
          {pendingNote && (
            <p className="text-xs text-ink-2">
              Your previous draft was restored below because the connection interrupted before it reached the server.
            </p>
          )}
          {dragOver && (
            <div className="rounded-sm border border-dashed border-blue bg-blue-soft px-3 py-2 text-xs font-medium text-blue">
              Drop files to upload them straight into {personName}&rsquo;s workspace.
            </div>
          )}
          <div className="flex flex-col gap-2 rounded-card border border-hairline bg-board/40 p-2.5">
            <Textarea
              ref={inputRef}
              id="composer-input"
              rows={2}
              value={draft}
              disabled={busy}
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  if (!busy && draft.trim()) onSend();
                }
              }}
              placeholder="Describe a task in plain words — write a report, spreadsheet, or audit…"
              aria-label="Describe a task for your agent"
              className="min-h-14 resize-none rounded-md border border-input bg-surface px-3 py-2 text-ui leading-6"
            />
            <div className="flex flex-wrap items-center justify-between gap-2 px-1">
              <div className="flex items-center gap-1.5">
                <input
                  ref={attachRef}
                  type="file"
                  multiple
                  className="hidden"
                  accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html,.docx,.xlsx,.pdf"
                  onChange={(e) => {
                    void handleFiles(e.target.files);
                    e.target.value = "";
                  }}
                />
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  className="max-md:min-h-11 gap-1.5 text-ink-2 hover:text-ink"
                  onClick={() => attachRef.current?.click()}
                >
                  <Paperclip className="size-3.5" strokeWidth={1.5} aria-hidden />
                  Attach files
                </Button>
              </div>
              <Button
                id="send-btn"
                type="button"
                size="sm"
                disabled={busy || !draft.trim()}
                onClick={onSend}
                className="max-md:min-h-11 gap-1.5 disabled:border disabled:border-hairline disabled:bg-track disabled:text-ink-2"
              >
                <Send className="size-3.5" strokeWidth={1.5} aria-hidden />
                {busy ? "Working…" : "Send"}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
