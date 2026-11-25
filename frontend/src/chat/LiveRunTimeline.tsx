import { useEffect, useState } from "react";
import { Check, Clock, Square, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BorderTrail } from "@/components/ui/border-trail";
import { TextShimmer } from "@/components/ui/text-shimmer";
import type { LiveRunStep } from "@/lib/api";
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
  return (
    <div
      id="live-run-timeline"
      role="status"
      aria-live="polite"
      className="relative flex flex-col gap-3 overflow-hidden rounded-card border border-hairline bg-surface p-4 shadow-card"
    >
      <BorderTrail size={56} className="opacity-70" />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2.5">
          <span className="pulse-dot size-2.5 rounded-full bg-blue" aria-hidden />
          <TextShimmer as="span" className="text-sm font-medium">
            Agent is working on it
          </TextShimmer>
          {streamMode && (
            <span className="mono rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">
              {streamMode.toUpperCase()}
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
