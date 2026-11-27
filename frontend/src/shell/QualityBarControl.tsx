import { useState } from "react";
import { Check, TrendingUp } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/kibo-ui/pill";
import {
  qualityFamilyKey,
  submitQualityFeedback,
  type BriefJob,
  type QualityBar,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export function matchQualityBar(job: BriefJob, bars: QualityBar[]): QualityBar | undefined {
  if (!bars.length) return undefined;
  const directFamily = job.task_family || job.quality_bar?.task_family;
  if (directFamily) {
    const found = bars.find((b) => b.task_family === directFamily);
    if (found) return found;
  }
  if (job.id) {
    const byRun = bars.find((b) => b.runs.some((r) => r.run_id === job.id));
    if (byRun) return byRun;
  }
  const taskLower = (job.task || "").trim().toLowerCase();
  if (taskLower) {
    const byTask = bars.find(
      (b) =>
        b.label.trim().toLowerCase() === taskLower ||
        b.runs.some((r) => r.task.trim().toLowerCase() === taskLower)
    );
    if (byTask) return byTask;
    const key = qualityFamilyKey(job.task);
    return bars.find((b) => b.task_family === key);
  }
  return undefined;
}

export function qualityBarScore(bar: QualityBar): number {
  if (bar.runs.length > 0) {
    const latest = bar.runs[bar.runs.length - 1];
    if (typeof latest.score === "number") return latest.score;
  }
  return typeof bar.best_score === "number" ? bar.best_score : 0;
}

export function QualityBarControls({
  bar,
  mobile = false,
  compact = false,
  onUpdated,
}: {
  bar: QualityBar;
  mobile?: boolean;
  compact?: boolean;
  onUpdated: (updated?: QualityBar) => void | Promise<void>;
}) {
  const [noteOpen, setNoteOpen] = useState(false);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState<"accept" | "improve" | null>(null);

  const isAccepted = bar.status === "accepted_good_enough";
  const score = qualityBarScore(bar);
  const tapClass = mobile ? "min-h-11" : "max-md:min-h-11";
  const improveLabel = mobile ? "Improve next run" : "Improve";

  const handleAccept = async () => {
    setBusy("accept");
    const r = await submitQualityFeedback(bar.task_family, "accept_good_enough");
    setBusy(null);
    if (!r.ok) {
      toast.error(r.error || "Could not lock quality bar.");
      return;
    }
    toast.success("Locked as good enough.");
    setNoteOpen(false);
    await onUpdated(r.data.quality_bar ?? r.data.bar);
  };

  const handleImproveSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const clean = note.trim();
    setBusy("improve");
    const r = await submitQualityFeedback(
      bar.task_family,
      "improve",
      clean || undefined,
      clean ? [clean] : undefined
    );
    setBusy(null);
    if (!r.ok) {
      toast.error(r.error || "Could not save improvement note.");
      return;
    }
    toast.success("Saved feedback for the next run.");
    setNote("");
    setNoteOpen(false);
    await onUpdated(r.data.quality_bar ?? r.data.bar);
  };

  return (
    <div
      data-quality-bar={bar.task_family}
      data-quality-status={bar.status}
      className={cn("flex flex-col gap-2", compact ? "w-full pt-2" : "w-full")}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Pill
          variant="secondary"
          data-quality-badge={bar.task_family}
          className={cn(
            "px-2 py-0.5 text-xs font-medium",
            isAccepted
              ? "bg-pill-active-bg text-pill-active-fg"
              : "bg-tag-ops-bg text-tag-ops-fg"
          )}
        >
          {isAccepted ? (
            <Check className="size-3.5" strokeWidth={1.5} aria-hidden />
          ) : (
            <TrendingUp className="size-3.5" strokeWidth={1.5} aria-hidden />
          )}
          <span>{isAccepted ? "Good enough" : "Improving"}</span>
        </Pill>

        <span className="mono text-xs text-ink-2">
          Score {score.toFixed(2)}
        </span>

        {bar.runs.length > 0 && (
          <span className="mono text-xs text-ink-2">
            {bar.runs.length} {bar.runs.length === 1 ? "run" : "runs"}
          </span>
        )}
      </div>

      {bar.criteria.length > 0 && (
        <p className="text-xs text-ink-2 [overflow-wrap:anywhere]">
          Bar: {bar.criteria.slice(-2).join("; ")}
        </p>
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Button
          type="button"
          size={mobile ? "default" : "xs"}
          variant={isAccepted ? "secondary" : "outline"}
          disabled={busy !== null || isAccepted}
          data-quality-accept={bar.task_family}
          className={cn(tapClass, mobile && "flex-1")}
          onClick={() => void handleAccept()}
        >
          <Check className="size-3.5" strokeWidth={1.5} aria-hidden />
          <span>{busy === "accept" ? "Locking…" : "Good enough"}</span>
        </Button>

        <Button
          type="button"
          size={mobile ? "default" : "xs"}
          variant="outline"
          disabled={busy !== null}
          data-quality-improve={bar.task_family}
          aria-expanded={noteOpen}
          className={cn(tapClass, mobile && "flex-1")}
          onClick={() => setNoteOpen((v) => !v)}
        >
          <TrendingUp className="size-3.5" strokeWidth={1.5} aria-hidden />
          <span>{improveLabel}</span>
        </Button>
      </div>

      {noteOpen && (
        <form onSubmit={(e) => void handleImproveSubmit(e)} className="flex flex-wrap items-center gap-2 pt-0.5">
          <Input
            type="text"
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder="What should the next run do better?"
            aria-label={`Improvement note for ${bar.label}`}
            data-quality-note-input={bar.task_family}
            className={cn("min-w-[180px] flex-1 text-sm", tapClass)}
          />
          <Button
            type="submit"
            size={mobile ? "default" : "xs"}
            disabled={busy !== null}
            data-quality-note-submit={bar.task_family}
            className={tapClass}
          >
            {busy === "improve" ? "Saving…" : "Save"}
          </Button>
          <Button
            type="button"
            size={mobile ? "default" : "xs"}
            variant="ghost"
            disabled={busy !== null}
            className={tapClass}
            onClick={() => {
              setNoteOpen(false);
              setNote("");
            }}
          >
            Cancel
          </Button>
        </form>
      )}
    </div>
  );
}
