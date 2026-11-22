// Recurring Routines MorphingDialog (always mounted OUTSIDE Vaul Drawer):
// - Lists standing routines from GET /api/routines (with localStorage progressive fallback on pre-v0.5.0 servers)
// - Create new recurring routine (POST /api/routines) with cadence presets & linked workflow
// - Run Now (POST /api/routines/:id/run, falling back to running prompt directly in Today/Chat)
// - Delete routine (DELETE /api/routines/:id)
import { useEffect, useState } from "react";
import { CalendarClock, Play, Plus, Trash2 } from "lucide-react";
import {
  MorphingDialog,
  MorphingDialogClose,
  MorphingDialogContainer,
  MorphingDialogContent,
  MorphingDialogDescription,
  MorphingDialogTitle,
  useMorphingDialog,
} from "@/components/ui/morphing-dialog";
import { SlidingNumber } from "@/components/ui/sliding-number";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Pill } from "@/components/kibo-ui/pill";
import type { Routine } from "@/lib/api";
import type { SkillMeta } from "@/lib/skillsCatalog";
import { relTime } from "@/lib/words";
import { open as openPreset } from "@/motion";

const SCHEDULES = [
  "Every weekday 08:30",
  "Daily 09:00",
  "Weekly Monday 09:00",
  "Every Friday 16:00",
  "Hourly",
];

function RoutinesBody({
  routines,
  skills,
  onRefresh,
  onCreate,
  onDelete,
  onRunNow,
}: {
  routines: Routine[];
  skills: SkillMeta[];
  onRefresh: () => void;
  onCreate: (input: { name: string; schedule: string; prompt: string; skill_id?: string }) => Promise<boolean>;
  onDelete: (id: string) => Promise<void>;
  onRunNow: (routine: Routine) => Promise<void>;
}) {
  const { isOpen, setIsOpen } = useMorphingDialog();
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState("");
  const [schedule, setSchedule] = useState(SCHEDULES[0]);
  const [skillId, setSkillId] = useState("");
  const [prompt, setPrompt] = useState("");
  const [saving, setSaving] = useState(false);
  const [runningId, setRunningId] = useState<string | null>(null);
  const [err, setErr] = useState("");

  useEffect(() => {
    if (isOpen) onRefresh();
  }, [isOpen, onRefresh]);

  const handleSkillPick = (id: string) => {
    setSkillId(id);
    const found = skills.find((s) => s.id === id);
    if (found && !prompt.trim()) {
      setPrompt(found.sample_prompt);
      if (!name.trim()) setName(`${found.title} routine`);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !prompt.trim()) {
      setErr("Enter both a routine name and the task prompt your agent should run.");
      return;
    }
    setSaving(true);
    setErr("");
    const ok = await onCreate({
      name: name.trim(),
      schedule: schedule.trim() || SCHEDULES[0],
      prompt: prompt.trim(),
      skill_id: skillId || undefined,
    });
    setSaving(false);
    if (!ok) {
      setErr("Could not save routine. Check the fields and try again.");
      return;
    }
    setName("");
    setPrompt("");
    setSkillId("");
    setSchedule(SCHEDULES[0]);
    setAdding(false);
  };

  return (
    <div id="routines-panel" className="flex flex-col gap-4 pt-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="min-w-0 flex-1 text-sm text-ink-2">
          Standing workflows your agent runs on cadence or on demand with full Field-Book receipts.
        </p>
        <Button
          type="button"
          size="sm"
          variant={adding ? "outline" : "default"}
          className="max-md:min-h-11"
          onClick={() => {
            setAdding((v) => !v);
            setErr("");
          }}
        >
          <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
          {adding ? "Cancel" : "New routine"}
        </Button>
      </div>

      {adding && (
        <form
          id="routine-create-form"
          onSubmit={handleSubmit}
          className="flex flex-col gap-3 rounded-card border border-hairline bg-board/50 p-3.5"
        >
          <div className="flex flex-col gap-1">
            <label htmlFor="routine-name" className="text-xs font-medium text-ink">
              Routine name
            </label>
            <Input
              id="routine-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Morning expense reconciliation"
              autoComplete="off"
              maxLength={96}
            />
          </div>

          <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
            <div className="flex flex-col gap-1">
              <label htmlFor="routine-schedule" className="text-xs font-medium text-ink">
                Cadence
              </label>
              <Input
                id="routine-schedule"
                value={schedule}
                onChange={(e) => setSchedule(e.target.value)}
                placeholder="Every weekday 08:30"
                autoComplete="off"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="routine-skill" className="text-xs font-medium text-ink">
                Built-in workflow (optional)
              </label>
              <select
                id="routine-skill"
                value={skillId}
                onChange={(e) => handleSkillPick(e.target.value)}
                className="focus-ring h-9 min-h-11 w-full rounded-sm border border-input bg-surface px-2.5 text-sm text-ink outline-none md:min-h-9"
              >
                <option value="">Auto-select from prompt</option>
                {skills.map((sk) => (
                  <option key={sk.id} value={sk.id}>
                    {sk.title} ({sk.filename})
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="flex flex-wrap gap-1.5">
            {SCHEDULES.map((preset) => (
              <Button
                key={preset}
                type="button"
                size="xs"
                variant={schedule === preset ? "default" : "outline"}
                className="max-md:min-h-11 text-xs"
                onClick={() => setSchedule(preset)}
              >
                {preset}
              </Button>
            ))}
          </div>

          <div className="flex flex-col gap-1">
            <label htmlFor="routine-prompt" className="text-xs font-medium text-ink">
              Task prompt
            </label>
            <Textarea
              id="routine-prompt"
              rows={3}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Describe what your agent should finish each time this routine runs…"
            />
          </div>

          {err && (
            <p role="alert" className="text-xs text-destructive-foreground">
              {err}
            </p>
          )}

          <div className="flex justify-end">
            <Button type="submit" size="sm" className="max-md:min-h-11" disabled={saving}>
              {saving ? "Saving…" : "Save routine"}
            </Button>
          </div>
        </form>
      )}

      {routines.length === 0 ? (
        <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
          No recurring routines yet. Create one above for daily expense audits, weekly meeting digests, or contract checks.
        </div>
      ) : (
        <ul className="flex flex-col gap-2.5">
          {routines.map((rt) => (
            <li
              key={rt.id}
              data-routine-id={rt.id}
              className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex min-w-0 flex-wrap items-center gap-2">
                  <CalendarClock className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                  <span className="truncate text-sm font-medium text-ink">{rt.name}</span>
                  {rt.skill_id && (
                    <Badge variant="outline" size="sm" className="mono text-xs">
                      {rt.skill_id}
                    </Badge>
                  )}
                </div>
                <Pill variant="secondary" className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2">
                  <span className="mono">{rt.schedule}</span>
                </Pill>
              </div>

              <p className="text-sm leading-5 text-ink-2">{rt.prompt}</p>

              <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-2.5">
                <div className="mono flex flex-wrap items-center gap-2 text-xs text-ink-2">
                  <span>
                    <SlidingNumber value={rt.run_count ?? 0} /> {(rt.run_count ?? 0) === 1 ? "run" : "runs"}
                  </span>
                  {rt.last_run_ts ? (
                    <>
                      <span aria-hidden>·</span>
                      <span>Last {relTime(rt.last_run_ts)}</span>
                    </>
                  ) : (
                    <>
                      <span aria-hidden>·</span>
                      <span>Not run yet</span>
                    </>
                  )}
                </div>

                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    type="button"
                    size="sm"
                    className="max-md:min-h-11"
                    disabled={runningId === rt.id}
                    onClick={async () => {
                      setRunningId(rt.id);
                      setIsOpen(false);
                      await onRunNow(rt);
                      setRunningId(null);
                    }}
                  >
                    <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                    {runningId === rt.id ? "Running…" : "Run now"}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="max-md:min-h-11"
                    aria-label={`Delete routine ${rt.name}`}
                    onClick={() => onDelete(rt.id)}
                  >
                    <Trash2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                    Delete
                  </Button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function RoutinesPanel({
  routines,
  skills,
  open,
  onOpenChange,
  onRefresh,
  onCreate,
  onDelete,
  onRunNow,
}: {
  routines: Routine[];
  skills: SkillMeta[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRefresh: () => void;
  onCreate: (input: { name: string; schedule: string; prompt: string; skill_id?: string }) => Promise<boolean>;
  onDelete: (id: string) => Promise<void>;
  onRunNow: (routine: Routine) => Promise<void>;
}) {
  return (
    <MorphingDialog transition={openPreset} open={open} onOpenChange={onOpenChange}>
      <MorphingDialogContainer>
        <MorphingDialogContent
          style={{ borderRadius: 18 }}
          className="relative mx-4 flex max-h-[84vh] w-[580px] max-w-[calc(100%-2rem)] flex-col gap-3 overflow-y-auto border border-hairline bg-surface p-5 text-ink shadow-lift"
        >
          <MorphingDialogClose className="focus-ring absolute right-3 top-3 grid size-11 place-items-center rounded-sm text-icon outline-none hover:bg-board md:size-9" />
          <MorphingDialogTitle className="pr-10 text-title font-medium text-ink">
            Recurring routines
          </MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">
            <RoutinesBody
              routines={routines}
              skills={skills}
              onRefresh={onRefresh}
              onCreate={onCreate}
              onDelete={onDelete}
              onRunNow={onRunNow}
            />
          </MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}
