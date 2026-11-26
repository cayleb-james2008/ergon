import { useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle,
  Check,
  Clock,
  PauseCircle,
  Play,
  RotateCcw,
  Send,
  Square,
  XCircle,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Pill } from "@/components/kibo-ui/pill";
import { FileGlyph } from "@/chat/Entry";
import type { PipelineResumePayload, PipelineRun } from "@/lib/api";
import { cn } from "@/lib/utils";
import { relTime } from "@/lib/words";

function collectInspectableFiles(run: PipelineRun): string[] {
  const seen = new Set<string>();
  const add = (p: string) => {
    const clean = String(p || "").trim();
    if (!clean || clean.includes("{{")) return;
    seen.add(clean);
  };
  for (const f of run.pending_checkpoint?.inspect_files ?? []) add(f);
  for (const st of run.step_states ?? []) {
    for (const f of st.files ?? []) add(f);
  }
  return Array.from(seen);
}

export function CheckpointCard({
  run,
  onResume,
  onRetry,
  onCancel,
  onOpenFile,
  compact = false,
}: {
  run: PipelineRun;
  onResume: (runId: string, payload: PipelineResumePayload) => Promise<void>;
  onRetry?: (runId: string) => Promise<void>;
  onCancel: (runId: string) => Promise<void>;
  onOpenFile: (path: string) => void;
  compact?: boolean;
}) {
  const cp = run.pending_checkpoint ?? null;
  const inputMode = cp?.input_mode ?? "approve_reject";
  const choices = cp?.choices ?? [];

  const [selectedChoice, setSelectedChoice] = useState<string>(() => choices[0] ?? "");
  const [textAnswer, setTextAnswer] = useState("");
  const [busyAction, setBusyAction] = useState<"resume" | "retry" | "cancel" | null>(null);
  const [busyChoice, setBusyChoice] = useState<"approve" | "reject" | "answer" | null>(null);
  const [localErr, setLocalErr] = useState("");
  const actionLockRef = useRef(false);

  useEffect(() => {
    setSelectedChoice(cp?.choices?.[0] ?? "");
    setTextAnswer("");
    setLocalErr("");
    setBusyAction(null);
    setBusyChoice(null);
    actionLockRef.current = false;
  }, [run.run_id, cp?.checkpoint_id]);

  const inspectFiles = useMemo(() => collectInspectableFiles(run), [run]);
  const specSteps = run.steps ?? run.steps_v2 ?? [];
  const pausedSpec = specSteps[cp?.step_index ?? run.current_step_index];
  const isDeliveryHold =
    cp?.kind === "deliver" ||
    Boolean(cp?.connector_id || cp?.deliver) ||
    pausedSpec?.kind === "deliver";
  const deliverTarget = cp?.connector_name || cp?.connector_id || cp?.deliver?.connector_id || pausedSpec?.deliver?.connector_id;
  const deliverFile = cp?.deliver?.file || pausedSpec?.deliver?.file;
  const totalSteps = Math.max(run.step_states.length, specSteps.length, run.current_step_index + 1);
  const currentStepNum = Math.min(totalSteps, run.current_step_index + 1);
  const completedCount = run.step_states.filter((s) => s.status === "ok").length;
  const stepsWithChildren = run.step_states.filter((s) => Array.isArray(s.child_items) && s.child_items.length > 0);
  const isFailed = run.status === "failed";
  const isInterrupted = run.status === "interrupted";

  const handleResume = async (overrideAction?: "approve" | "reject") => {
    if (actionLockRef.current || busyAction !== null) return;
    setLocalErr("");
    if (!isFailed && !isInterrupted) {
      if (inputMode === "choice" && choices.length > 0 && !(selectedChoice || choices[0])) {
        setLocalErr("Select one of the choices before resuming.");
        return;
      }
      if (inputMode === "text" && !textAnswer.trim()) {
        setLocalErr("Enter an answer for downstream steps before resuming.");
        return;
      }
    }
    actionLockRef.current = true;
    setBusyAction("resume");
    setBusyChoice(inputMode === "choice" || inputMode === "text" ? "answer" : (overrideAction ?? "approve"));
    try {
      if (inputMode === "choice") {
        const picked = selectedChoice || choices[0] || "";
        await onResume(run.run_id, {
          action: "answer",
          choice: picked,
          answer: picked,
        });
      } else if (inputMode === "text") {
        await onResume(run.run_id, {
          action: "answer",
          answer: textAnswer.trim(),
        });
      } else {
        const act = overrideAction ?? "approve";
        await onResume(run.run_id, {
          action: act,
          answer: act === "approve" ? "approved" : "rejected",
        });
      }
    } catch {
      setLocalErr("Could not resume pipeline. Try again.");
    } finally {
      actionLockRef.current = false;
      setBusyAction(null);
      setBusyChoice(null);
    }
  };

  const handleRetry = async () => {
    if (!onRetry || actionLockRef.current || busyAction !== null) return;
    actionLockRef.current = true;
    setBusyAction("retry");
    setLocalErr("");
    try {
      await onRetry(run.run_id);
    } catch {
      setLocalErr("Could not retry failed step. Try again.");
    } finally {
      actionLockRef.current = false;
      setBusyAction(null);
    }
  };

  const handleCancel = async () => {
    if (actionLockRef.current || busyAction !== null) return;
    actionLockRef.current = true;
    setBusyAction("cancel");
    setLocalErr("");
    try {
      await onCancel(run.run_id);
    } catch {
      setLocalErr("Could not abort pipeline. Try again.");
    } finally {
      actionLockRef.current = false;
      setBusyAction(null);
    }
  };

  const resumeDisabled =
    busyAction !== null ||
    (!isFailed && !isInterrupted && inputMode === "choice" && !selectedChoice && choices.length > 0) ||
    (!isFailed && !isInterrupted && inputMode === "text" && !textAnswer.trim());

  return (
    <section
      data-pipeline-checkpoint={run.run_id}
      aria-label={run.routine_name ? `Checkpoint for ${run.routine_name}` : "Pipeline checkpoint"}
      className={cn(
        "flex flex-col gap-3 rounded-card border border-hairline",
        isFailed ? "bg-surface" : "bg-blue-soft",
        compact ? "p-3.5" : "p-5"
      )}
    >
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="flex min-w-0 flex-col gap-1">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" size="sm" className="bg-surface text-xs text-ink-2">
              Step {currentStepNum} of {totalSteps}
            </Badge>
            <span className="text-xs text-ink-2">
              {completedCount} completed
            </span>
          </div>
          <h3 className="text-base font-medium text-ink [overflow-wrap:anywhere]">
            {run.routine_name || "Resumable pipeline job"}
          </h3>
        </div>

        <Pill
          variant="secondary"
          className={cn(
            "px-2.5 py-0.5 text-xs font-medium",
            isFailed
              ? "bg-status-stopped-bg text-status-stopped-fg"
              : "bg-status-waiting-bg text-status-waiting-fg"
          )}
        >
          {isFailed ? (
            <AlertTriangle className="size-3.5" strokeWidth={1.5} aria-hidden />
          ) : (
            <PauseCircle className="size-3.5" strokeWidth={1.5} aria-hidden />
          )}
          <span>
            {isFailed
              ? "Step failed"
              : isInterrupted
                ? "Interrupted, ready to resume"
                : "Waiting on you"}
          </span>
        </Pill>
      </div>

      {cp?.prompt && (
        <p className="text-sm leading-6 text-ink [overflow-wrap:anywhere]">{cp.prompt}</p>
      )}

      {isDeliveryHold && (
        <div
          data-delivery-hold-details="true"
          className="flex flex-col gap-1 rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-ink-2"
        >
          <div className="flex items-center gap-1.5 font-medium text-ink">
            <Send className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            <span>Outbound delivery held for approval</span>
          </div>
          {deliverTarget && (
            <div className="[overflow-wrap:anywhere]">
              Target: {deliverTarget}
              {cp?.delivery_url ? ` (${cp.delivery_url})` : ""}
            </div>
          )}
          {deliverFile && <div className="[overflow-wrap:anywhere]">File: {deliverFile}</div>}
          {cp?.payload_preview && (
            <div className="rounded-sm bg-board/60 px-2 py-1 text-ink-2 [overflow-wrap:anywhere]">
              {cp.payload_preview}
            </div>
          )}
        </div>
      )}

      {run.agent_snapshot && (run.agent_snapshot.messages.length > 0 || run.agent_snapshot.written.length > 0) && (
        <div data-agent-snapshot-badge="true" className="text-xs text-ink-2">
          Agent context preserved ({run.agent_snapshot.messages.length} turns, {run.agent_snapshot.written.length} files)
        </div>
      )}

      {isFailed && run.error && (
        <p className="rounded-sm border border-hairline bg-board/60 px-3 py-2 text-xs text-ink-2 [overflow-wrap:anywhere]">
          {run.error}
        </p>
      )}

      {run.step_states.length > 0 && (
        <ol className="flex flex-wrap items-center gap-1.5 text-xs">
          {run.step_states.map((st, idx) => {
            const childTotal = st.child_items?.length ?? 0;
            const childDone = st.child_items?.filter((c) => c.status === "ok").length ?? 0;
            return (
              <li
                key={`${st.step_id}-${idx}`}
                className="inline-flex items-center gap-1 rounded-chip border border-hairline bg-surface px-2 py-0.5 text-ink-2"
              >
                {st.status === "ok" ? (
                  <Check className="size-3 text-dot-green" strokeWidth={1.75} aria-hidden />
                ) : st.status === "failed" ? (
                  <XCircle className="size-3 text-dot-red" strokeWidth={1.75} aria-hidden />
                ) : st.status === "waiting_human" ? (
                  <Clock className="size-3 text-dot-amber" strokeWidth={1.75} aria-hidden />
                ) : st.status === "running" ? (
                  <span className="pulse-dot size-1.5 rounded-full bg-blue" aria-hidden />
                ) : (
                  <span className="size-1.5 rounded-full bg-icon" aria-hidden />
                )}
                <span>{st.step_id}</span>
                {childTotal > 0 && (
                  <span className="mono text-ink-3">
                    {childDone}/{childTotal}
                  </span>
                )}
              </li>
            );
          })}
        </ol>
      )}

      {stepsWithChildren.length > 0 && (
        <div className="flex flex-col gap-1 border-t border-hairline pt-2 text-xs text-ink-2">
          {stepsWithChildren.map((st) => (
            <div key={st.step_id} data-child-items-for={st.step_id} className="flex flex-wrap items-center gap-1.5">
              <span className="font-medium text-ink">{st.step_id} batch:</span>
              {st.child_items!.map((child, cIdx) => (
                <span
                  key={`${child.step_id}-${cIdx}`}
                  className="inline-flex items-center gap-1 rounded-chip border border-hairline bg-surface px-2 py-0.5 text-xs text-ink-2"
                >
                  {child.status === "ok" ? (
                    <Check className="size-3 text-dot-green" strokeWidth={1.75} aria-hidden />
                  ) : child.status === "failed" ? (
                    <XCircle className="size-3 text-dot-red" strokeWidth={1.75} aria-hidden />
                  ) : (
                    <span className="size-1.5 rounded-full bg-blue" aria-hidden />
                  )}
                  <span className="truncate max-w-[140px]">{child.item || child.step_id}</span>
                </span>
              ))}
            </div>
          ))}
        </div>
      )}

      {inspectFiles.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-xs font-medium text-ink-2">Inspect files:</span>
          {inspectFiles.map((path) => (
            <Button
              key={path}
              type="button"
              size="xs"
              variant="outline"
              aria-label={`Inspect ${path}`}
              onClick={() => onOpenFile(path)}
              className="max-md:min-h-11 max-w-full gap-1.5 rounded-chip border-hairline bg-surface px-2 text-xs font-normal text-ink hover:bg-board"
            >
              <FileGlyph name={path} />
              <span className="truncate">{path}</span>
            </Button>
          ))}
        </div>
      )}

      {!isFailed && !isInterrupted && inputMode === "choice" && choices.length > 0 && (
        <div className="flex flex-col gap-1.5" role="radiogroup" aria-label="Checkpoint choices">
          {choices.map((ch) => {
            const active = selectedChoice === ch;
            return (
              <button
                key={ch}
                type="button"
                role="radio"
                aria-checked={active}
                onClick={() => setSelectedChoice(ch)}
                className={cn(
                  "flex min-h-10 items-center justify-between gap-2 rounded-sm border px-3 py-2 text-left text-sm transition-[color,background-color,border-color] max-md:min-h-11",
                  active
                    ? "border-blue bg-surface font-medium text-ink"
                    : "border-hairline bg-surface/75 text-ink-2 hover:bg-surface hover:text-ink"
                )}
              >
                <span className="min-w-0 [overflow-wrap:anywhere]">{ch}</span>
                {active && <Check className="size-4 shrink-0 text-blue" strokeWidth={1.75} aria-hidden />}
              </button>
            );
          })}
        </div>
      )}

      {!isFailed && !isInterrupted && inputMode === "text" && (
        <div className="flex flex-col gap-1">
          <label htmlFor={`cp-answer-${run.run_id}`} className="text-xs font-medium text-ink-2">
            Your answer for downstream steps
          </label>
          <Textarea
            id={`cp-answer-${run.run_id}`}
            rows={2}
            value={textAnswer}
            onChange={(e) => setTextAnswer(e.target.value)}
            placeholder="Enter instructions or decision details..."
            className="bg-surface text-sm"
          />
        </div>
      )}

      {localErr && (
        <p role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-3 py-1.5 text-xs text-status-stopped-fg">
          {localErr}
        </p>
      )}

      <div className="flex flex-wrap items-center gap-2 pt-0.5">
        {isFailed && onRetry ? (
          <Button
            type="button"
            size="sm"
            className="max-md:min-h-11 gap-1.5"
            disabled={busyAction !== null}
            onClick={() => void handleRetry()}
          >
            <RotateCcw className="size-3.5" strokeWidth={1.5} aria-hidden />
            {busyAction === "retry" ? "Retrying..." : "Retry failed step"}
          </Button>
        ) : (
          <Button
            type="button"
            size="sm"
            data-action="resume-pipeline"
            className="max-md:min-h-11 gap-1.5"
            disabled={resumeDisabled}
            onClick={() => void handleResume("approve")}
          >
            <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
            {busyAction === "resume" && busyChoice !== "reject" ? "Resuming..." : "Resume Job"}
          </Button>
        )}

        {!isFailed && !isInterrupted && inputMode === "approve_reject" && (
          <Button
            type="button"
            size="sm"
            variant="outline"
            data-action="reject-pipeline"
            className="max-md:min-h-11"
            disabled={busyAction !== null}
            onClick={() => void handleResume("reject")}
          >
            {busyAction === "resume" && busyChoice === "reject" ? "Rejecting..." : "Reject & continue"}
          </Button>
        )}

        <Button
          type="button"
          size="sm"
          variant="outline"
          data-action="abort-pipeline"
          className="max-md:min-h-11 gap-1.5"
          disabled={busyAction !== null}
          onClick={() => void handleCancel()}
        >
          <Square className="size-3" strokeWidth={1.5} aria-hidden />
          {busyAction === "cancel" ? "Aborting..." : "Abort"}
        </Button>
      </div>
    </section>
  );
}

export function PipelineRunStrip({
  runs,
  onResume,
  onRetry,
  onCancel,
  onOpenFile,
  showPausedCards = false,
}: {
  runs: PipelineRun[];
  onResume: (runId: string, payload: PipelineResumePayload) => Promise<void>;
  onRetry?: (runId: string) => Promise<void>;
  onCancel: (runId: string) => Promise<void>;
  onOpenFile: (path: string) => void;
  showPausedCards?: boolean;
}) {
  const [cancellingIds, setCancellingIds] = useState<Record<string, boolean>>({});
  const cancelLockRef = useRef<Set<string>>(new Set());

  const activeOrPaused = runs.filter(
    (r) =>
      r.status === "running" ||
      r.status === "paused_checkpoint" ||
      r.status === "interrupted" ||
      r.status === "failed"
  );
  if (activeOrPaused.length === 0) return null;

  const running = activeOrPaused.filter((r) => r.status === "running");
  const actionable = activeOrPaused.filter((r) => r.status !== "running");

  const handleStripCancel = async (runId: string) => {
    if (cancelLockRef.current.has(runId)) return;
    cancelLockRef.current.add(runId);
    setCancellingIds((prev) => ({ ...prev, [runId]: true }));
    try {
      await onCancel(runId);
    } finally {
      cancelLockRef.current.delete(runId);
      setCancellingIds((prev) => {
        const next = { ...prev };
        delete next[runId];
        return next;
      });
    }
  };

  return (
    <div id="pipeline-runs-strip" className="flex flex-col gap-2.5">
      {running.map((run) => {
        const doneCount = run.step_states.filter((s) => s.status === "ok").length;
        const activeBranches = run.step_states.filter((s) => s.status === "running").length;
        const childStats = run.step_states.reduce(
          (acc, s) => {
            if (!s.child_items?.length) return acc;
            return {
              done: acc.done + s.child_items.filter((c) => c.status === "ok").length,
              total: acc.total + s.child_items.length,
            };
          },
          { done: 0, total: 0 }
        );
        const total = Math.max(run.step_states.length, (run.steps ?? run.steps_v2 ?? []).length, 1);
        const isCancelling = Boolean(cancellingIds[run.run_id]);
        return (
          <div
            key={run.run_id}
            data-pipeline-running={run.run_id}
            role="status"
            className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-surface px-3.5 py-2.5"
          >
            <div className="flex min-w-0 flex-wrap items-center gap-2">
              <span className="pulse-dot size-2 shrink-0 rounded-full bg-blue" aria-hidden />
              <span className="truncate text-sm font-medium text-ink">
                {run.routine_name || "Pipeline running"}
              </span>
              <span className="text-xs text-ink-2">
                Step <span className="mono">{Math.min(total, doneCount + 1)}/{total}</span>
              </span>
              {activeBranches > 1 && (
                <Badge variant="outline" size="sm" className="text-xs text-ink-2">
                  {activeBranches} parallel
                </Badge>
              )}
              {childStats.total > 0 && (
                <Badge variant="outline" size="sm" className="mono text-xs text-ink-2">
                  {childStats.done}/{childStats.total}
                </Badge>
              )}
              {run.updated_ms > 0 && (
                <span className="hidden text-xs text-ink-3 sm:inline">
                  {relTime(run.updated_ms)}
                </span>
              )}
            </div>
            <Button
              type="button"
              size="xs"
              variant="outline"
              data-action="abort-running-pipeline"
              disabled={isCancelling}
              className="max-md:min-h-11 gap-1"
              onClick={() => void handleStripCancel(run.run_id)}
            >
              <Square className="size-3" strokeWidth={1.5} aria-hidden />
              {isCancelling ? "Aborting..." : "Abort"}
            </Button>
          </div>
        );
      })}

      {showPausedCards &&
        actionable.map((run) => (
          <CheckpointCard
            key={run.run_id}
            run={run}
            onResume={onResume}
            onRetry={onRetry}
            onCancel={onCancel}
            onOpenFile={onOpenFile}
            compact
          />
        ))}
    </div>
  );
}
