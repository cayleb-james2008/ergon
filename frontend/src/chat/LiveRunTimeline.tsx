import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Clock, GitBranch, Square, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BorderTrail } from "@/components/ui/border-trail";
import { TextShimmer } from "@/components/ui/text-shimmer";
import type { LiveRun, LiveRunStep, PipelineChildItemState } from "@/lib/api";
import { toolLabel } from "@/lib/words";

const MAX_TOKEN_PREVIEW_CHARS = 1000;
const MAX_CHILD_ITEMS_PER_STEP = 50;

function fmtElapsed(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "0.0s";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function toStepObj(raw: string | LiveRunStep, idx: number): LiveRunStep {
  if (raw && typeof raw === "object") {
    const tool = String(raw.tool ?? "").trim();
    const rawStr = String(raw.raw ?? (tool ? `Tool: ${tool}` : raw.label ?? ""));
    return {
      ...raw,
      id: raw.id || `step-${idx}`,
      tool,
      label: raw.label || (tool ? toolLabel(tool, undefined, rawStr) || tool : rawStr),
      status: raw.status || "done",
      raw: rawStr,
    };
  }
  const safe = String(raw ?? "");
  return {
    id: `step-${idx}`,
    tool: "",
    label: safe,
    status: "done",
    raw: safe,
  };
}

function LiveElapsedClock({ startedMs, serverElapsedMs }: { startedMs?: number; serverElapsedMs: number }) {
  const [tickMs, setTickMs] = useState(serverElapsedMs);
  useEffect(() => {
    if (!startedMs) {
      setTickMs(serverElapsedMs);
      return;
    }
    const update = () => setTickMs(Math.max(serverElapsedMs, Date.now() - startedMs));
    update();
    const timer = window.setInterval(update, 250);
    return () => window.clearInterval(timer);
  }, [startedMs, serverElapsedMs]);
  return <>{fmtElapsed(tickMs)}</>;
}

export function LiveRunTimeline({
  steps,
  elapsedMs,
  startedMs,
  streamMode,
  cancelling,
  onCancel,
  runId,
  runs,
  cancellingRunIds,
  onCancelRun,
}: {
  steps: LiveRunStep[];
  elapsedMs: number;
  startedMs?: number;
  streamMode: "sse" | "poll" | null;
  cancelling: boolean;
  onCancel: (runId?: string) => void;
  runId?: string | null;
  runs?: LiveRun[];
  cancellingRunIds?: Record<string, boolean>;
  onCancelRun?: (runId: string) => void | Promise<void>;
}) {
  const [tokenPreviewByRun, setTokenPreviewByRun] = useState<Record<string, string>>({});
  const [childItemsByKey, setChildItemsByKey] = useState<Record<string, PipelineChildItemState[]>>({});
  const cancelLockRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    setTokenPreviewByRun({});
    setChildItemsByKey({});
    cancelLockRef.current.clear();
  }, [startedMs]);

  useEffect(() => {
    if (!cancelling) {
      cancelLockRef.current.clear();
    }
  }, [cancelling]);

  useEffect(() => {
    if (streamMode !== "sse" || typeof EventSource === "undefined") return;
    let closed = false;
    let es: EventSource | null = null;
    try {
      es = new EventSource("/api/runs/stream");
      es.addEventListener("token_delta", (ev) => {
        if (closed) return;
        try {
          const parsed = JSON.parse((ev as MessageEvent).data) as {
            run_id?: unknown;
            delta?: unknown;
            text?: unknown;
            content?: unknown;
          };
          const piece =
            typeof parsed.delta === "string"
              ? parsed.delta
              : typeof parsed.text === "string"
                ? parsed.text
                : typeof parsed.content === "string"
                  ? parsed.content
                  : "";
          if (!piece) return;
          const tailPiece =
            piece.length > MAX_TOKEN_PREVIEW_CHARS
              ? piece.slice(-MAX_TOKEN_PREVIEW_CHARS)
              : piece;
          const key = typeof parsed.run_id === "string" && parsed.run_id ? parsed.run_id : "__default__";
          setTokenPreviewByRun((prev) => {
            const combined = (prev[key] ?? "") + tailPiece;
            return {
              ...prev,
              [key]:
                combined.length > MAX_TOKEN_PREVIEW_CHARS
                  ? combined.slice(-MAX_TOKEN_PREVIEW_CHARS)
                  : combined,
            };
          });
        } catch {
          // Ignore malformed delta frames
        }
      });
      const handleChildUpdate = (ev: Event) => {
        if (closed) return;
        try {
          const parsed = JSON.parse((ev as MessageEvent).data) as {
            run_id?: string;
            step_id?: string;
            child_items?: PipelineChildItemState[];
          };
          if (parsed.step_id && Array.isArray(parsed.child_items)) {
            const bounded = parsed.child_items.slice(0, MAX_CHILD_ITEMS_PER_STEP);
            const runPrefix = parsed.run_id ? `${parsed.run_id}:` : "";
            setChildItemsByKey((prev) => ({
              ...prev,
              [`${runPrefix}${parsed.step_id}`]: bounded,
              [parsed.step_id!]: bounded,
            }));
          }
        } catch {
          // Ignore malformed frames
        }
      };
      es.addEventListener("pipeline_step_started", handleChildUpdate);
      es.addEventListener("pipeline_step_done", handleChildUpdate);
      es.onerror = () => {
        if (es) {
          es.close();
          es = null;
        }
      };
    } catch {
      // Ignore if EventSource cannot open
    }
    return () => {
      closed = true;
      if (es) es.close();
    };
  }, [streamMode, startedMs]);

  const normalizedRuns = useMemo(() => {
    if (Array.isArray(runs) && runs.length > 0) {
      return runs.map((r) => ({
        ...r,
        steps: (r.steps ?? []).map((s, idx) => toStepObj(s, idx)),
      }));
    }
    return [
      {
        active: true,
        run_id: runId || "__default__",
        thread_id: "",
        task: "Agent is working on it",
        started_ms: startedMs ?? 0,
        elapsed_ms: elapsedMs,
        steps,
      },
    ];
  }, [runs, runId, startedMs, elapsedMs, steps]);

  const triggerCancel = (targetRunId: string) => {
    if (cancelLockRef.current.has(targetRunId)) return;
    const isRunCancelling = Boolean(cancellingRunIds?.[targetRunId]) || (normalizedRuns.length === 1 && cancelling);
    if (isRunCancelling) return;
    cancelLockRef.current.add(targetRunId);
    if (onCancelRun && targetRunId !== "__default__") {
      void onCancelRun(targetRunId);
    } else {
      onCancel(targetRunId === "__default__" ? undefined : targetRunId);
    }
  };

  return (
    <div
      id="live-run-timeline"
      role="status"
      aria-live="polite"
      className="relative flex min-w-0 flex-col gap-3 overflow-hidden rounded-card border border-hairline bg-surface p-4 shadow-card"
    >
      <BorderTrail size={56} className="opacity-70" />
      {normalizedRuns.map((run, runIdx) => {
        const runSteps = run.steps as LiveRunStep[];
        const runningBranches = runSteps.filter((st) => st.status === "running").length;
        const isRunCancelling =
          Boolean(cancellingRunIds?.[run.run_id]) || (runIdx === 0 && cancelling && normalizedRuns.length === 1);
        const tokenPreview =
          tokenPreviewByRun[run.run_id] ||
          (normalizedRuns.length === 1
            ? tokenPreviewByRun.__default__ || Object.values(tokenPreviewByRun)[0] || ""
            : "");
        const displayTitle =
          run.routine_name ||
          (run.task && run.task !== "Active run" ? run.task : "Agent is working on it");
        const sourceBadge =
          run.kind === "routine" || run.routine_id
            ? "Scheduled"
            : run.kind === "pipeline"
              ? "Pipeline"
              : normalizedRuns.length > 1
                ? "Chat"
                : null;

        return (
          <div
            key={run.run_id || `run-${runIdx}`}
            data-live-run={run.run_id}
            className={runIdx > 0 ? "flex min-w-0 flex-col gap-2.5 border-t border-hairline pt-3" : "flex min-w-0 flex-col gap-2.5"}
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex min-w-0 flex-wrap items-center gap-2">
                <span className="pulse-dot size-2.5 shrink-0 rounded-full bg-blue" aria-hidden />
                {runIdx === 0 && normalizedRuns.length === 1 ? (
                  <TextShimmer as="span" className="text-sm font-medium">
                    {displayTitle}
                  </TextShimmer>
                ) : (
                  <span className="truncate text-sm font-medium text-ink [overflow-wrap:anywhere]">
                    {displayTitle}
                  </span>
                )}
                {sourceBadge && (
                  <span className="rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-xs text-ink-2">
                    {sourceBadge}
                  </span>
                )}
                {runIdx === 0 && streamMode && (
                  <span className="rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">
                    {streamMode.toUpperCase()}
                  </span>
                )}
                {runningBranches > 1 && (
                  <span
                    data-live-parallel-branches={runningBranches}
                    className="inline-flex items-center gap-1 rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-xs text-ink-2"
                  >
                    <GitBranch className="size-3 text-icon" strokeWidth={1.5} aria-hidden />
                    {runningBranches} parallel
                  </span>
                )}
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <span className="mono flex items-center gap-1 text-xs text-ink-2">
                  <Clock className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                  <LiveElapsedClock startedMs={run.started_ms || startedMs} serverElapsedMs={run.elapsed_ms ?? elapsedMs} />
                </span>
                <Button
                  id={runIdx === 0 ? "cancel-run-btn" : undefined}
                  type="button"
                  size="xs"
                  variant="outline"
                  data-action="cancel-live-run"
                  data-run-id={run.run_id}
                  aria-label={`Stop ${displayTitle}`}
                  disabled={isRunCancelling}
                  onClick={() => triggerCancel(run.run_id)}
                  className="max-md:min-h-11 gap-1.5"
                >
                  <Square className="size-3" strokeWidth={1.75} aria-hidden />
                  {isRunCancelling ? "Stopping…" : "Stop"}
                </Button>
              </div>
            </div>

            {runSteps.length > 0 && (
              <ul className="flex flex-col gap-1.5 border-t border-hairline pt-2.5 text-sm">
                {runSteps.map((st, idx) => {
                  const childItems =
                    st.child_items ??
                    (st.step_id ? childItemsByKey[`${run.run_id}:${st.step_id}`] ?? childItemsByKey[st.step_id] : undefined) ??
                    (st.id ? childItemsByKey[`${run.run_id}:${st.id}`] ?? childItemsByKey[st.id] : undefined);
                  const childTotal = childItems?.length ?? 0;
                  const childDone = childItems?.filter((c) => c.status === "ok").length ?? 0;
                  return (
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
                        {childTotal > 0 && (
                          <span className="mono shrink-0 rounded-chip border border-hairline bg-board/60 px-1.5 py-0.5 text-xs text-ink-2">
                            {childDone}/{childTotal}
                          </span>
                        )}
                      </span>
                      {typeof st.elapsed_ms === "number" && st.elapsed_ms > 0 && (
                        <span className="mono shrink-0 text-xs text-ink-3">{fmtElapsed(st.elapsed_ms)}</span>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}

            {tokenPreview && (
              <div
                id={runIdx === 0 ? "live-token-delta-preview" : undefined}
                data-live-token-delta="true"
                className="max-h-32 overflow-y-auto border-t border-hairline pt-2.5 text-xs leading-5 text-ink-2 whitespace-pre-wrap break-words"
              >
                {tokenPreview}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
