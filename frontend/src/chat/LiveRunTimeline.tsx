import { useEffect, useMemo, useState } from "react";
import { Check, Clock, GitBranch, Square, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BorderTrail } from "@/components/ui/border-trail";
import { TextShimmer } from "@/components/ui/text-shimmer";
import type { LiveRunStep, PipelineChildItemState } from "@/lib/api";
import { toolLabel } from "@/lib/words";

function fmtElapsed(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "0.0s";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
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
}: {
  steps: LiveRunStep[];
  elapsedMs: number;
  startedMs?: number;
  streamMode: "sse" | "poll" | null;
  cancelling: boolean;
  onCancel: () => void;
}) {
  const [tokenPreview, setTokenPreview] = useState("");
  const [childItemsByStep, setChildItemsByStep] = useState<Record<string, PipelineChildItemState[]>>({});

  useEffect(() => {
    setTokenPreview("");
    setChildItemsByStep({});
  }, [startedMs]);

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
          setTokenPreview((prev) => {
            const next = prev + piece;
            return next.length > 1000 ? next.slice(-1000) : next;
          });
        } catch {
          // Ignore malformed delta frames
        }
      });
      const handleChildUpdate = (ev: Event) => {
        if (closed) return;
        try {
          const parsed = JSON.parse((ev as MessageEvent).data) as {
            step_id?: string;
            child_items?: PipelineChildItemState[];
          };
          if (parsed.step_id && Array.isArray(parsed.child_items)) {
            setChildItemsByStep((prev) => ({ ...prev, [parsed.step_id!]: parsed.child_items! }));
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

  const runningBranches = useMemo(
    () => steps.filter((st) => st.status === "running").length,
    [steps]
  );

  return (
    <div
      id="live-run-timeline"
      role="status"
      aria-live="polite"
      className="relative flex flex-col gap-3 overflow-hidden rounded-card border border-hairline bg-surface p-4 shadow-card"
    >
      <BorderTrail size={56} className="opacity-70" />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2.5">
          <span className="pulse-dot size-2.5 rounded-full bg-blue" aria-hidden />
          <TextShimmer as="span" className="text-sm font-medium">
            Agent is working on it
          </TextShimmer>
          {streamMode && (
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
        <div className="flex items-center gap-2">
          <span className="mono flex items-center gap-1 text-xs text-ink-2">
            <Clock className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            <LiveElapsedClock startedMs={startedMs} serverElapsedMs={elapsedMs} />
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
          {steps.map((st, idx) => {
            const childItems =
              st.child_items ??
              (st.step_id ? childItemsByStep[st.step_id] : undefined) ??
              (st.id ? childItemsByStep[st.id] : undefined);
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
          id="live-token-delta-preview"
          data-live-token-delta="true"
          className="border-t border-hairline pt-2.5 text-xs leading-5 text-ink-2 whitespace-pre-wrap break-words"
        >
          {tokenPreview}
        </div>
      )}
    </div>
  );
}
