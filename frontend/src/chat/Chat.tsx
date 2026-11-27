// Claude-Desktop central conversation desk:
// - Serene empty start canvas: greeting + elevated floating composer + 1 quiet horizontal row of quick-start chips
// - Inline Approval & File-Diff cards directly inside the thread (Approve / Hold / Revise)
// - Live SSE / poll run status banner (#live-run-timeline) with TextShimmer + BorderTrail (live work only) + cancel
// - Elevated floating composer (#composer-input) with multi-file upload & drag-drop
import { useEffect, useMemo, useRef, useState } from "react";
import {
  FileCheck2,
  FileSpreadsheet,
  FileText,
  FolderOpen,
  Paperclip,
  Send,
  ShieldCheck,
  Sparkles,
  X,
} from "lucide-react";
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
import { LiveRunTimeline } from "@/chat/LiveRunTimeline";
import { PipelineRunStrip } from "@/shell/CheckpointCard";
import { OnboardingCard } from "@/shell/OnboardingCard";
import { cn } from "@/lib/utils";
import {
  call,
  cancelLiveRun,
  type AdminContactConfig,
  type Decision,
  type LiveRun,
  type LiveRunStatus,
  type LiveRunStep,
  type Msg,
  type PipelineResumePayload,
  type PipelineRun,
} from "@/lib/api";
import { detectBatchWorkflow, detectWorkflowForFile, type StarterCard } from "@/lib/words";

export type Recent = { task: string; date: string; files: string[] };
export { LiveRunTimeline };

export function Chat({
  threadId,
  personName,
  role,
  messages,
  decisions,
  busy,
  cancelling,
  runStartedMs,
  liveElapsedMs,
  streamMode,
  liveSteps,
  liveTimeline,
  liveRuns,
  cancellingRunIds,
  draft,
  setDraft,
  pendingNote,
  starters,
  fileCount = 0,
  lastUploaded,
  onDismissUploaded,
  onRemoveUploaded,
  useServerExport,
  onSend,
  onRunPrompt,
  onCancel,
  onCancelRun,
  onReview,
  onResolveDecision,
  onOpenFile,
  onOpenFiles,
  onExportAuditBundle,
  onUploadFiles,
  onSaveRoutine,
  pipelineRuns = [],
  onResumePipeline,
  onRetryPipeline,
  onCancelPipeline,
  showOnboarding = false,
  onboardingSaving = false,
  adminContact = null,
  onCompleteOnboarding,
  onOpenSkills,
  onOpenHelp,
  inputRef,
}: {
  threadId: string | null;
  personName: string;
  role: string;
  messages: Msg[];
  decisions: Decision[];
  busy: boolean;
  cancelling: boolean;
  runStartedMs?: number;
  liveElapsedMs: number;
  streamMode: "sse" | "poll" | null;
  liveSteps: string[];
  liveTimeline: LiveRunStep[];
  liveRuns?: LiveRun[];
  cancellingRunIds?: Record<string, boolean>;
  draft: string;
  setDraft: (s: string) => void;
  pendingNote: boolean;
  starters: StarterCard[];
  fileCount?: number;
  lastUploaded?: string[];
  onDismissUploaded?: () => void;
  onRemoveUploaded?: (path: string) => void;
  useServerExport?: boolean;
  onSend: () => void;
  onRunPrompt?: (prompt: string) => void;
  onCancel: () => void;
  onCancelRun?: (runId: string) => Promise<void> | void;
  onReview: () => void;
  onResolveDecision: (id: string, action: "approve" | "deny", label: string) => Promise<void>;
  onOpenFile: (path: string) => void;
  onOpenFiles: () => void;
  onExportAuditBundle: (format: "json" | "md") => Promise<void>;
  onUploadFiles: (files: FileList | File[]) => Promise<number>;
  onSaveRoutine?: (
    name: string,
    prompt: string,
    schedule?: string,
    playbook?: { steps?: string[]; expected_inputs?: string[]; pack_deliverables?: boolean }
  ) => Promise<boolean>;
  pipelineRuns?: PipelineRun[];
  onResumePipeline?: (runId: string, payload: PipelineResumePayload) => Promise<void>;
  onRetryPipeline?: (runId: string) => Promise<void>;
  onCancelPipeline?: (runId: string) => Promise<void>;
  showOnboarding?: boolean;
  onboardingSaving?: boolean;
  adminContact?: AdminContactConfig | null;
  onCompleteOnboarding?: () => void;
  onOpenSkills?: () => void;
  onOpenHelp?: () => void;
  inputRef: React.RefObject<HTMLTextAreaElement | null>;
}) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const attachRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);
  const [polledRuns, setPolledRuns] = useState<LiveRun[]>([]);
  const [localCancellingIds, setLocalCancellingIds] = useState<Record<string, boolean>>({});

  const items = buildItems(messages);
  const cards = starters;
  const visiblePipelineRuns = useMemo(
    () =>
      (pipelineRuns ?? []).filter(
        (r) =>
          (r.status === "running" ||
            r.status === "paused_checkpoint" ||
            r.status === "interrupted" ||
            r.status === "failed") &&
          (!threadId ||
            !r.thread_id ||
            r.thread_id === threadId ||
            r.status === "running" ||
            r.status === "paused_checkpoint")
      ),
    [pipelineRuns, threadId]
  );

  useEffect(() => {
    let active = true;
    const syncRunsStatus = async () => {
      const r = await call<LiveRunStatus>("/api/runs/status");
      if (!active || !r.ok) return;
      if (Array.isArray(r.data.runs)) {
        setPolledRuns(r.data.runs.filter((run) => run && run.active !== false && !run.cancelled));
      } else if (r.data.active && (r.data.run_id || r.data.thread_id)) {
        setPolledRuns([
          {
            active: true,
            run_id: r.data.run_id || r.data.thread_id || "active",
            thread_id: r.data.thread_id || "",
            task: r.data.task || "Active run",
            started_ms: r.data.started_ms || Date.now(),
            elapsed_ms: r.data.elapsed_ms,
            steps: r.data.steps ?? [],
          },
        ]);
      } else {
        setPolledRuns([]);
      }
    };
    void syncRunsStatus();
    const timer = window.setInterval(() => {
      if (busy || visiblePipelineRuns.some((r) => r.status === "running")) {
        void syncRunsStatus();
      }
    }, 1000);
    let es: EventSource | null = null;
    if (typeof EventSource !== "undefined") {
      try {
        es = new EventSource("/api/runs/stream");
        const onRunEvent = (ev: Event, kind: string) => {
          if (!active) return;
          try {
            const parsed = JSON.parse((ev as MessageEvent).data || "{}") as Record<string, unknown>;
            if (Array.isArray(parsed.runs)) {
              setPolledRuns(
                (parsed.runs as LiveRun[]).filter((run) => run && run.active !== false && !run.cancelled)
              );
              return;
            }
            const rid = typeof parsed.run_id === "string" ? parsed.run_id : "";
            if (kind === "run_finished" || kind === "done" || kind === "cancelled") {
              if (rid) setPolledRuns((prev) => prev.filter((x) => x.run_id !== rid));
              return;
            }
            if ((kind === "run_started" || (kind === "status" && parsed.active)) && rid) {
              const incomingRun: LiveRun = {
                active: true,
                run_id: rid,
                thread_id: typeof parsed.thread_id === "string" ? parsed.thread_id : "",
                task: String(parsed.task ?? parsed.routine_name ?? "Active run"),
                started_ms: typeof parsed.started_ms === "number" ? parsed.started_ms : Date.now(),
                elapsed_ms: typeof parsed.elapsed_ms === "number" ? parsed.elapsed_ms : 0,
                steps: Array.isArray(parsed.steps) ? (parsed.steps as (string | LiveRunStep)[]) : [],
                kind: typeof parsed.kind === "string" ? parsed.kind : undefined,
                routine_id: typeof parsed.routine_id === "string" ? parsed.routine_id : null,
                routine_name: typeof parsed.routine_name === "string" ? parsed.routine_name : null,
              };
              setPolledRuns((prev) => {
                const idx = prev.findIndex((x) => x.run_id === rid);
                return idx >= 0 ? prev.map((x, i) => (i === idx ? incomingRun : x)) : [incomingRun, ...prev];
              });
            }
          } catch {
            // ignore
          }
        };
        for (const evName of ["status", "run_started", "run_finished", "done", "cancelled"]) {
          es.addEventListener(evName, (ev) => onRunEvent(ev, evName));
        }
        es.onerror = () => {
          if (es) {
            es.close();
            es = null;
          }
        };
      } catch {
        // ignore
      }
    }
    return () => {
      active = false;
      window.clearInterval(timer);
      if (es) es.close();
    };
  }, [busy, visiblePipelineRuns.length]);

  const activeLiveRuns = useMemo(() => {
    const map = new Map<string, LiveRun>();
    const addRun = (r: LiveRun) => {
      if (!r || !r.active || r.cancelled) return;
      const key = r.run_id || r.thread_id || "active";
      map.set(key, r);
    };
    for (const r of liveRuns ?? []) addRun(r);
    for (const r of polledRuns) addRun(r);
    if (busy) {
      const existingMatch = Array.from(map.values()).find(
        (r) => !r.routine_id && (!threadId || !r.thread_id || r.thread_id === threadId)
      );
      if (existingMatch) {
        map.set(existingMatch.run_id, {
          ...existingMatch,
          steps: liveTimeline.length > 0 ? liveTimeline : existingMatch.steps,
          elapsed_ms: liveElapsedMs || existingMatch.elapsed_ms,
          started_ms: runStartedMs || existingMatch.started_ms,
        });
      } else {
        map.set("__primary__", {
          active: true,
          run_id: "__primary__",
          thread_id: threadId || "",
          task: "Agent is working on it",
          started_ms: runStartedMs || Date.now(),
          elapsed_ms: liveElapsedMs,
          steps: liveTimeline,
          kind: "chat",
        });
      }
    }
    return Array.from(map.values());
  }, [liveRuns, polledRuns, busy, threadId, liveTimeline, liveElapsedMs, runStartedMs]);

  const mergedCancellingIds = useMemo(
    () => ({ ...(cancellingRunIds ?? {}), ...localCancellingIds }),
    [cancellingRunIds, localCancellingIds]
  );

  const handleCancelSpecific = async (targetRunId: string) => {
    setLocalCancellingIds((prev) => ({ ...prev, [targetRunId]: true }));
    try {
      const isPrimary =
        targetRunId === "__primary__" ||
        (busy &&
          polledRuns.length <= 1 &&
          (!threadId || polledRuns.some((r) => r.run_id === targetRunId && r.thread_id === threadId && !r.routine_id)));
      if (isPrimary) {
        onCancel();
      } else if (onCancelRun) {
        await onCancelRun(targetRunId);
      } else {
        await cancelLiveRun(targetRunId);
      }
      setPolledRuns((prev) => prev.filter((r) => r.run_id !== targetRunId && r.thread_id !== targetRunId));
    } finally {
      setLocalCancellingIds((prev) => {
        const next = { ...prev };
        delete next[targetRunId];
        return next;
      });
    }
  };
  const uploadSuggestion = useMemo(() => {
    if (!lastUploaded || lastUploaded.length === 0) return null;
    if (lastUploaded.length >= 2) return detectBatchWorkflow(lastUploaded);
    return detectWorkflowForFile(lastUploaded[0]);
  }, [lastUploaded]);

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

  const handleLoadStarterPack = async () => {
    const sampleFiles = [
      new File(
        [
          "# Q1 Operations Notes\n\n- Revenue closed 18% above plan across enterprise accounts.\n- Three duplicate contractor invoices flagged in Q1 spend.\n- Vendor agreement renewal notice period requires 60-day review.\n",
        ],
        "quarterly-notes.md",
        { type: "text/markdown" }
      ),
      new File(
        [
          "date,vendor,category,amount,status\n2026-01-12,Northstar Cloud,Infrastructure,1420.00,paid\n2026-01-15,Delta Office,Facilities,380.00,paid\n2026-01-15,Delta Office,Facilities,380.00,duplicate\n2026-02-02,Apex Legal,Advisory,2400.00,pending\n",
        ],
        "q1-expenses.csv",
        { type: "text/csv" }
      ),
      new File(
        [
          "# Master Services Agreement\n\n## Term & Renewal\nAuto-renews annually unless written notice is provided 60 days prior.\n\n## Liability\nLiability is capped at 12 months of fees paid.\n",
        ],
        "vendor-agreement.md",
        { type: "text/markdown" }
      ),
    ];
    const ok = await onUploadFiles(sampleFiles);
    if (ok > 0) toast.success(`Loaded ${ok} starter workspace files.`);
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

  const empty = items.length === 0 && !busy;

  const starterIcon = (skill: string) => {
    if (skill === "write_document") return FileText;
    if (skill === "make_spreadsheet") return FileSpreadsheet;
    if (skill === "audit_contract") return ShieldCheck;
    return Sparkles;
  };

  const composerNode = (
    <div className="flex w-full flex-col gap-2">
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
      {uploadSuggestion && !busy && (
        <div
          id="upload-workflow-suggestion"
          role="status"
          className="flex flex-col gap-2 rounded-card border border-hairline bg-surface px-3 py-2 text-xs"
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex min-w-0 items-center gap-2">
              <Sparkles className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.5} aria-hidden />
              <span className="truncate text-ink">
                Ready:{" "}
                <span className="mono font-medium">
                  {lastUploaded && lastUploaded.length > 1
                    ? `${lastUploaded.length} workspace files`
                    : uploadSuggestion.file}
                </span>
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-1.5">
              <Button
                type="button"
                size="xs"
                className="max-md:min-h-11 gap-1.5"
                onClick={() => {
                  onDismissUploaded?.();
                  if (onRunPrompt) {
                    onRunPrompt(uploadSuggestion.prompt);
                  } else {
                    setDraft(uploadSuggestion.prompt);
                    requestAnimationFrame(() => inputRef.current?.focus());
                  }
                }}
              >
                {uploadSuggestion.label}
              </Button>
              <Button
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11"
                onClick={() => onOpenFile(uploadSuggestion.file)}
              >
                Inspect
              </Button>
              {onDismissUploaded && (
                <Button
                  type="button"
                  size="icon-xs"
                  variant="ghost"
                  aria-label="Dismiss upload suggestion"
                  className="max-md:min-h-11 max-md:min-w-11"
                  onClick={onDismissUploaded}
                >
                  <X className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                </Button>
              )}
            </div>
          </div>
          {lastUploaded && lastUploaded.length > 1 && (
            <div className="flex flex-wrap items-center gap-1.5">
              {lastUploaded.map((fname) => (
                <span
                  key={fname}
                  className="inline-flex max-w-full items-center gap-1 rounded-chip border border-hairline bg-board/60 pl-2 pr-1 py-0.5 text-xs text-ink"
                >
                  <button
                    type="button"
                    onClick={() => onOpenFile(fname)}
                    className="mono max-w-[200px] truncate text-left hover:underline"
                    title={`Inspect ${fname}`}
                  >
                    {fname}
                  </button>
                  {onRemoveUploaded && (
                    <button
                      type="button"
                      aria-label={`Remove ${fname} from queue`}
                      onClick={() => onRemoveUploaded(fname)}
                      className="rounded-full p-0.5 text-ink-3 hover:bg-board hover:text-ink"
                    >
                      <X className="size-3" strokeWidth={1.5} aria-hidden />
                    </button>
                  )}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
      <div className="flex flex-col gap-2 rounded-dialog border border-hairline bg-board/45 p-2.5 shadow-card motion-safe:transition-[border-color] motion-safe:duration-(--dur-reveal) motion-safe:ease-(--ease-house)">
        <Textarea
          ref={inputRef}
          id="composer-input"
          rows={empty ? 3 : 2}
          value={draft}
          disabled={busy}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              if (!busy && draft.trim()) onSend();
            }
          }}
          placeholder="Describe a task in plain words: write a report, spreadsheet, or audit…"
          aria-label="Describe a task for your agent"
          className="min-h-14 resize-none rounded-xl border border-input bg-surface px-1.5 py-1 text-ui leading-6 text-ink"
        />
        <div className="flex flex-wrap items-center justify-between gap-2 pt-0.5">
          <div className="flex flex-wrap items-center gap-1.5">
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
            {empty && (
              <span className="hidden text-xs text-ink-3 sm:inline">Drop spreadsheets or documents anywhere</span>
            )}
          </div>
          <Button
            id="send-btn"
            type="button"
            size="sm"
            disabled={busy || !draft.trim()}
            onClick={onSend}
            className="max-md:min-h-11 gap-1.5"
          >
            <Send className="size-3.5" strokeWidth={1.5} aria-hidden />
            {busy ? "Working…" : "Send"}
          </Button>
        </div>
      </div>
    </div>
  );

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
        <div
          className={cn(
            "mx-auto flex w-full max-w-[740px] flex-col",
            empty ? "min-h-full justify-center gap-6 py-4" : "gap-5"
          )}
        >
          {empty ? (
            <>
              <div className="flex flex-col items-start gap-1.5 sm:items-center sm:text-center">
                <p className="text-xs font-medium text-ink-2" aria-label={`Working for ${personName}`}>
                  Working for {personName}{role ? ` (${role})` : ""}
                </p>
                <h1 className="text-[24px] leading-8 font-medium tracking-[-0.02em] text-ink sm:text-[28px] sm:leading-9">
                  What would you like finished today?
                </h1>
              </div>

              {composerNode}

              {showOnboarding && onCompleteOnboarding && (
                <OnboardingCard
                  adminContact={adminContact}
                  saving={onboardingSaving}
                  onComplete={onCompleteOnboarding}
                  onOpenFiles={onOpenFiles}
                  onOpenSkills={onOpenSkills}
                  onOpenHelp={onOpenHelp}
                />
              )}

              {!busy && activeLiveRuns.length > 0 && (
                <LiveRunTimeline
                  steps={liveTimeline}
                  elapsedMs={liveElapsedMs}
                  startedMs={runStartedMs}
                  streamMode={streamMode}
                  cancelling={cancelling}
                  runs={activeLiveRuns}
                  cancellingRunIds={mergedCancellingIds}
                  onCancel={onCancel}
                  onCancelRun={handleCancelSpecific}
                />
              )}

              {onResumePipeline && onCancelPipeline && visiblePipelineRuns.length > 0 && (
                <PipelineRunStrip
                  runs={visiblePipelineRuns}
                  onResume={onResumePipeline}
                  onRetry={onRetryPipeline}
                  onCancel={onCancelPipeline}
                  onOpenFile={onOpenFile}
                  showPausedCards
                />
              )}

              <div className="flex flex-col gap-3 sm:items-center">
                <section aria-label="Start a task" className="flex flex-wrap items-center gap-2 sm:justify-center">
                  {cards.map((card, i) => {
                    const Icon = starterIcon(card.skill);
                    return (
                      <button
                        key={card.prompt + i}
                        type="button"
                        data-starter-card
                        title={card.detail}
                        onClick={() => {
                          setDraft(card.prompt);
                          requestAnimationFrame(() => inputRef.current?.focus());
                        }}
                        className="focus-ring inline-flex min-h-11 items-center gap-1.5 rounded-full border border-hairline bg-surface px-3.5 py-1.5 text-xs font-medium text-ink-2 outline-none motion-safe:transition-[transform,background-color,border-color,color] motion-safe:duration-(--dur-press) motion-safe:ease-(--ease-house) motion-safe:active:scale-[0.97] hover:border-ink-3/50 hover:bg-board hover:text-ink md:min-h-9"
                      >
                        <Icon className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                        <span>{card.label || card.category}</span>
                      </button>
                    );
                  })}
                </section>
                <div className="flex flex-wrap items-center gap-1.5 sm:justify-center">
                  <Button
                    type="button"
                    size="xs"
                    variant="ghost"
                    className="max-md:min-h-11 gap-1.5 text-ink-3 hover:text-ink"
                    onClick={onOpenFiles}
                  >
                    <FolderOpen className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Browse workspace files
                  </Button>
                  {fileCount === 0 && (
                    <Button
                      id="starter-pack-btn"
                      type="button"
                      size="xs"
                      variant="ghost"
                      className="max-md:min-h-11 gap-1.5 text-ink-3 hover:text-ink"
                      onClick={() => void handleLoadStarterPack()}
                    >
                      <Sparkles className="size-3.5" strokeWidth={1.5} aria-hidden />
                      Load sample workspace
                    </Button>
                  )}
                  <Button
                    type="button"
                    size="xs"
                    variant="ghost"
                    className="max-md:min-h-11 gap-1.5 text-ink-3 hover:text-ink"
                    onClick={() => void onExportAuditBundle("md")}
                  >
                    <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Export Audit Bundle (.md)
                  </Button>
                </div>
              </div>
            </>
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
                      useServerExport={useServerExport}
                      onOpenFile={onOpenFile}
                      onSaveRoutine={onSaveRoutine}
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
                return <Reply key={idx} text={item.text} onOpenFile={onOpenFile} />;
              })}
            </div>
          )}

          {(busy || (!empty && activeLiveRuns.length > 0)) && (
            <LiveRunTimeline
              steps={liveTimeline}
              elapsedMs={liveElapsedMs}
              startedMs={runStartedMs}
              streamMode={streamMode}
              cancelling={cancelling}
              runs={activeLiveRuns}
              cancellingRunIds={mergedCancellingIds}
              onCancel={onCancel}
              onCancelRun={handleCancelSpecific}
            />
          )}

          {!empty && onResumePipeline && onCancelPipeline && visiblePipelineRuns.length > 0 && (
            <PipelineRunStrip
              runs={visiblePipelineRuns}
              onResume={onResumePipeline}
              onRetry={onRetryPipeline}
              onCancel={onCancelPipeline}
              onOpenFile={onOpenFile}
              showPausedCards
            />
          )}

          <div ref={bottomRef} />
        </div>
      </div>

      {!empty && (
        <div className="safe-b shrink-0 border-t border-hairline bg-canvas px-4 py-3 sm:px-6">
          <div className="mx-auto w-full max-w-[740px]">{composerNode}</div>
        </div>
      )}
    </div>
  );
}
