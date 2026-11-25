// Recurring Routines MorphingDialog (v0.7.0, always mounted OUTSIDE Vaul Drawer):
// - Scheduler status pill (active tick cadence / paused / on-demand) + active/paused counts
// - Pause / Resume toggle (PATCH /api/routines/:id)
// - Live Next run countdown badge ticked every 15s
// - Expandable Run History (GET /api/routines/:id/history) with linked SHA-256 receipts & threads
// - Create new recurring routine (POST /api/routines) with scheduler cadence presets
// - Run Now (POST /api/routines/:id/run) & Delete routine (DELETE /api/routines/:id)
import { useCallback, useEffect, useRef, useState } from "react";
import {
  CalendarClock,
  ChevronDown,
  ChevronRight,
  Clock,
  Download,
  FileDown,
  History,
  Pause,
  Play,
  Plus,
  Trash2,
  Upload,
} from "lucide-react";
import { toast } from "sonner";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Pill } from "@/components/kibo-ui/pill";
import {
  estimateNextRunMs,
  formatCountdown,
  hasV050Api,
  hasV070Api,
  type Routine,
  type RoutineRunRecord,
  type SchedulerStatus,
  type Status,
} from "@/lib/api";
import { exportPlaybookDownload, exportReceiptDownload, MAX_PLAYBOOK_PACK_BYTES } from "@/lib/receiptExport";
import type { SkillMeta } from "@/lib/skillsCatalog";
import { cn } from "@/lib/utils";
import { relTime } from "@/lib/words";

const SCHEDULES = [
  "every 15m",
  "hourly",
  "daily 09:00",
  "weekly mon",
  "every 30m",
];

function fmtDuration(startMs: number, endMs?: number | null): string {
  if (!endMs || endMs < startMs) return "";
  const ms = endMs - startMs;
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function RoutinesBody({
  routines,
  skills,
  status,
  scheduler,
  historyRev,
  personName,
  presetSchedule,
  presetNonce,
  onRefresh,
  onCreate,
  onToggleEnabled,
  onLoadHistory,
  onDelete,
  onRunNow,
  onOpenThread,
  onImportPlaybook,
}: {
  routines: Routine[];
  skills: SkillMeta[];
  status: Status | null | "error";
  scheduler: SchedulerStatus | null;
  historyRev: number;
  personName: string;
  presetSchedule?: string | null;
  presetNonce?: number;
  onRefresh: () => void;
  onCreate: (input: {
    name: string;
    schedule: string;
    prompt: string;
    skill_id?: string;
    steps?: string[];
    expected_inputs?: string[];
    pack_deliverables?: boolean;
  }) => Promise<boolean>;
  onToggleEnabled: (id: string, enabled: boolean) => Promise<boolean>;
  onLoadHistory: (id: string) => Promise<RoutineRunRecord[]>;
  onDelete: (id: string) => Promise<void>;
  onRunNow: (routine: Routine) => Promise<void>;
  onOpenThread?: (threadId: string) => void;
  onImportPlaybook?: (rawText: string) => Promise<{ ok: boolean; sha256?: string; error?: string }>;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const importInputRef = useRef<HTMLInputElement | null>(null);
  const [importing, setImporting] = useState(false);
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState("");
  const [schedule, setSchedule] = useState(SCHEDULES[2]);
  const [skillId, setSkillId] = useState("");
  const [prompt, setPrompt] = useState("");
  const [extraSteps, setExtraSteps] = useState<string[]>([]);
  const [packDeliverables, setPackDeliverables] = useState(false);
  const [saving, setSaving] = useState(false);
  const [runningId, setRunningId] = useState<string | null>(null);
  const [togglingId, setTogglingId] = useState<string | null>(null);
  const [expandedHistory, setExpandedHistory] = useState<Record<string, boolean>>({});
  const [historyByRoutine, setHistoryByRoutine] = useState<Record<string, RoutineRunRecord[]>>({});
  const [historyLoading, setHistoryLoading] = useState<Record<string, boolean>>({});
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [err, setErr] = useState("");

  useEffect(() => {
    if (presetNonce && presetSchedule) {
      setSchedule(presetSchedule);
      setAdding(true);
    }
  }, [presetSchedule, presetNonce]);

  useEffect(() => {
    if (!isOpen) return;
    onRefresh();
    setNowMs(Date.now());
    const timer = window.setInterval(() => setNowMs(Date.now()), 15_000);
    return () => window.clearInterval(timer);
  }, [isOpen, onRefresh]);

  const refreshExpandedHistory = useCallback(
    async (id: string) => {
      setHistoryLoading((prev) => ({ ...prev, [id]: true }));
      const runs = await onLoadHistory(id);
      setHistoryByRoutine((prev) => ({ ...prev, [id]: runs }));
      setHistoryLoading((prev) => ({ ...prev, [id]: false }));
    },
    [onLoadHistory]
  );

  // Refresh any open routine history drawers when a background routine fires.
  useEffect(() => {
    if (!isOpen) return;
    for (const [id, open] of Object.entries(expandedHistory)) {
      if (open) void refreshExpandedHistory(id);
    }
  }, [historyRev, isOpen]); // eslint-disable-line react-hooks/exhaustive-deps

  const toggleHistory = (rt: Routine) => {
    const next = !expandedHistory[rt.id];
    setExpandedHistory((prev) => ({ ...prev, [rt.id]: next }));
    if (next) void refreshExpandedHistory(rt.id);
  };

  const handleSkillPick = (id: string) => {
    setSkillId(id);
    const found = skills.find((s) => s.id === id);
    if (found && !prompt.trim()) {
      setPrompt(found.sample_prompt);
      if (!name.trim()) setName(`${found.title} routine`);
    }
  };

  const applyPlaybookPreset = (preset: "close" | "contract") => {
    setAdding(true);
    setErr("");
    if (preset === "close") {
      setName("Month-End Close Playbook");
      setSchedule("weekly mon");
      setSkillId("reconcile_ledger");
      setPrompt("Clean and normalize workspace expense sheets with clean_dataset");
      setExtraSteps([
        "Reconcile expenses and flag duplicates with reconcile_ledger",
        "Draft an executive financial brief and export Word (.docx)",
      ]);
      setPackDeliverables(true);
    } else {
      setName("Vendor Contract Audit Playbook");
      setSchedule("daily 09:00");
      setSkillId("audit_contract");
      setPrompt("Audit workspace contract documents with audit_contract for risks and missing clauses");
      setExtraSteps([
        "Summarize contract findings and action owners into a Word (.docx) review memo",
      ]);
      setPackDeliverables(true);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !prompt.trim()) {
      setErr("Enter both a routine name and the task prompt your agent should run.");
      return;
    }
    const cleanChain = [prompt.trim(), ...extraSteps.map((s) => s.trim()).filter(Boolean)];
    setSaving(true);
    setErr("");
    const ok = await onCreate({
      name: name.trim(),
      schedule: schedule.trim() || SCHEDULES[2],
      prompt: prompt.trim(),
      skill_id: skillId || undefined,
      steps: cleanChain.length > 1 ? cleanChain : undefined,
      pack_deliverables: packDeliverables || undefined,
    });
    setSaving(false);
    if (!ok) {
      setErr("Could not save routine. Check the fields and try again.");
      return;
    }
    setName("");
    setPrompt("");
    setExtraSteps([]);
    setPackDeliverables(false);
    setSkillId("");
    setSchedule(SCHEDULES[2]);
    setAdding(false);
  };

  const handleExportRunReceipt = async (rt: Routine, run: RoutineRunRecord) => {
    const useServer = hasV050Api(status, "receipts_export") && Boolean(run.receipt_id);
    const res = await exportReceiptDownload(
      {
        id: run.receipt_id || run.run_id,
        task: `Routine: ${rt.name}`,
        person: personName || "Active employee",
        status: run.status || "finished",
        summary: run.summary || `Scheduled execution of "${rt.name}" (${rt.schedule}).`,
        ts: run.finished_ms || run.started_ms,
        files: [],
      },
      "md",
      useServer
    );
    toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
  };

  const handleExportPlaybook = async (rt: Routine) => {
    const useServer = hasV050Api(status, "routines");
    const res = await exportPlaybookDownload(rt, useServer);
    toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
  };

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file || !onImportPlaybook) return;
    if (file.size > MAX_PLAYBOOK_PACK_BYTES) {
      toast.error(`Playbook pack exceeds the 256 KB limit (${file.size} bytes).`);
      return;
    }
    setImporting(true);
    try {
      const rawText = await file.text();
      await onImportPlaybook(rawText);
    } finally {
      setImporting(false);
    }
  };

  const activeCount = routines.filter((r) => r.enabled).length;
  const pausedCount = routines.length - activeCount;
  const hasV070 = hasV070Api(status);
  const statusSchedDisabled =
    status && status !== "error" && (status.scheduler_enabled === false || status.capabilities?.scheduler === false);
  const schedulerPausedByEnv = Boolean(scheduler?.disabled_by_env) || scheduler?.enabled === false || Boolean(statusSchedDisabled);
  const schedulerActive = hasV070 && !schedulerPausedByEnv && scheduler?.running !== false;
  const tickSecs = scheduler?.tick_secs ?? 15;

  return (
    <div id="routines-panel" className="flex flex-col gap-4 pt-1">
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-board/40 px-3 py-2.5">
        <div className="flex flex-wrap items-center gap-2">
          <Pill
            id="scheduler-status-pill"
            variant="secondary"
            className={cn(
              "px-2.5 py-0.5 text-xs font-medium",
              schedulerActive
                ? "bg-pill-active-bg text-pill-active-fg"
                : schedulerPausedByEnv
                  ? "bg-pill-warn-bg text-pill-warn-fg"
                  : "bg-track text-ink-2"
            )}
          >
            <span
              aria-hidden
              className={cn(
                "size-2 rounded-full",
                schedulerActive ? "bg-dot-green" : schedulerPausedByEnv ? "bg-dot-amber" : "bg-icon"
              )}
            />
            <span>
              {schedulerActive
                ? `Scheduler active (${tickSecs}s tick)`
                : schedulerPausedByEnv
                  ? "Scheduler paused"
                  : "On-demand routines"}
            </span>
          </Pill>
          <span className="mono text-xs text-ink-2">
            {activeCount} active{pausedCount > 0 ? `, ${pausedCount} paused` : ""}
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
          {onImportPlaybook && (
            <>
              <input
                ref={importInputRef}
                id="playbook-import-input"
                type="file"
                accept=".json,.ergon-playbook.json,application/json"
                className="sr-only"
                onChange={handleImportFile}
              />
              <Button
                id="playbook-import-btn"
                type="button"
                size="sm"
                variant="outline"
                className="max-md:min-h-11"
                disabled={importing}
                onClick={() => importInputRef.current?.click()}
              >
                <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
                {importing ? "Importing…" : "Import playbook"}
              </Button>
            </>
          )}
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
                placeholder="daily 09:00 or every 15m"
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

          <div className="flex flex-col gap-1.5">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <span className="text-xs font-medium text-ink-2">Playbook templates</span>
              <div className="flex flex-wrap gap-1.5">
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  onClick={() => applyPlaybookPreset("close")}
                >
                  Month-End Close (3 steps)
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  onClick={() => applyPlaybookPreset("contract")}
                >
                  Contract Audit + Pack (2 steps)
                </Button>
              </div>
            </div>
            <label htmlFor="routine-prompt" className="text-xs font-medium text-ink">
              {extraSteps.length > 0 ? "Step 1 prompt" : "Task prompt"}
            </label>
            <Textarea
              id="routine-prompt"
              rows={3}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Describe what your agent should finish each time this routine runs…"
            />
            {extraSteps.map((stepVal, sIdx) => (
              <div key={sIdx} className="flex items-start gap-2 pt-1">
                <span className="mono mt-2 shrink-0 text-xs text-ink-2">Step {sIdx + 2}</span>
                <Input
                  value={stepVal}
                  aria-label={`Playbook step ${sIdx + 2}`}
                  placeholder="Next chained step prompt…"
                  onChange={(e) =>
                    setExtraSteps((prev) => prev.map((v, i) => (i === sIdx ? e.target.value : v)))
                  }
                />
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  aria-label={`Remove step ${sIdx + 2}`}
                  onClick={() => setExtraSteps((prev) => prev.filter((_, i) => i !== sIdx))}
                >
                  Remove
                </Button>
              </div>
            ))}
            <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
              {extraSteps.length < 5 && (
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  onClick={() => setExtraSteps((prev) => [...prev, ""])}
                >
                  <Plus className="size-3" strokeWidth={1.5} aria-hidden />
                  Chain step
                </Button>
              )}
              <label className="inline-flex cursor-pointer items-center gap-1.5 text-xs text-ink-2">
                <input
                  type="checkbox"
                  checked={packDeliverables}
                  onChange={(e) => setPackDeliverables(e.target.checked)}
                />
                Auto-bundle Deliverable Pack (.md)
              </label>
            </div>
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
          No recurring routines yet. Create one above for scheduled expense audits, meeting digests, or contract checks.
        </div>
      ) : (
        <ul className="flex flex-col gap-2.5">
          {routines.map((rt) => {
            const nextMs = rt.enabled
              ? (rt.next_run_ms ?? estimateNextRunMs(rt.schedule, rt.last_run_ts || nowMs, rt.interval_secs))
              : null;
            const nextLabel = rt.enabled ? `Next run ${formatCountdown(nextMs, nowMs)}` : "Paused";
            const histOpen = Boolean(expandedHistory[rt.id]);
            const runs = historyByRoutine[rt.id] ?? rt.recent_runs ?? [];
            const loadingRuns = Boolean(historyLoading[rt.id]);

            return (
              <li
                key={rt.id}
                data-routine-id={rt.id}
                data-routine-enabled={rt.enabled ? "true" : "false"}
                className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex min-w-0 flex-wrap items-center gap-2">
                    <CalendarClock className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                    <span className="truncate text-sm font-medium text-ink">{rt.name}</span>
                    <Pill
                      variant="secondary"
                      className={cn(
                        "px-2 py-0.5 text-xs font-medium",
                        rt.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-track text-ink-2"
                      )}
                    >
                      {rt.enabled ? "Active" : "Paused"}
                    </Pill>
                    {rt.skill_id && (
                      <Badge variant="outline" size="sm" className="mono text-xs">
                        {rt.skill_id}
                      </Badge>
                    )}
                    {rt.steps && rt.steps.length > 1 && (
                      <Badge variant="outline" size="sm" className="mono text-xs">
                        Playbook · {rt.steps.length} steps
                      </Badge>
                    )}
                  </div>
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Pill variant="secondary" className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2">
                      <span className="mono">{rt.schedule}</span>
                    </Pill>
                    <Badge
                      variant="outline"
                      size="sm"
                      data-next-run={rt.id}
                      className="mono gap-1 text-xs text-ink-2"
                    >
                      <Clock className="size-3 text-icon" strokeWidth={1.5} aria-hidden />
                      {nextLabel}
                    </Badge>
                  </div>
                </div>

                {rt.steps && rt.steps.length > 1 ? (
                  <ol className="flex flex-col gap-1 text-xs leading-5 text-ink-2">
                    {rt.steps.map((stText, sIdx) => (
                      <li key={sIdx} className="flex gap-1.5">
                        <span className="mono shrink-0 text-ink-3">{sIdx + 1}.</span>
                        <span>{stText}</span>
                      </li>
                    ))}
                  </ol>
                ) : (
                  <p className="text-sm leading-5 text-ink-2">{rt.prompt}</p>
                )}

                <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-2.5">
                  <div className="mono flex flex-wrap items-center gap-2 text-xs text-ink-2">
                    <span className="whitespace-nowrap">
                      {rt.run_count ?? 0} {(rt.run_count ?? 0) === 1 ? "run" : "runs"}
                    </span>
                    {rt.last_run_ts ? (
                      <span>Last {relTime(rt.last_run_ts)}</span>
                    ) : (
                      <span>Not run yet</span>
                    )}
                  </div>

                  <div className="flex flex-wrap items-center gap-1.5">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      data-action="toggle-routine"
                      aria-pressed={rt.enabled}
                      aria-label={rt.enabled ? `Pause routine ${rt.name}` : `Resume routine ${rt.name}`}
                      className="max-md:min-h-11"
                      disabled={togglingId === rt.id}
                      onClick={async () => {
                        setTogglingId(rt.id);
                        await onToggleEnabled(rt.id, !rt.enabled);
                        setTogglingId(null);
                      }}
                    >
                      {rt.enabled ? (
                        <>
                          <Pause className="size-3.5" strokeWidth={1.5} aria-hidden />
                          Pause
                        </>
                      ) : (
                        <>
                          <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                          Resume
                        </>
                      )}
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      data-action="routine-history"
                      aria-expanded={histOpen}
                      className="max-md:min-h-11"
                      onClick={() => toggleHistory(rt)}
                    >
                      {histOpen ? (
                        <ChevronDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                      ) : (
                        <ChevronRight className="size-3.5" strokeWidth={1.5} aria-hidden />
                      )}
                      <History className="size-3.5" strokeWidth={1.5} aria-hidden />
                      Run history
                    </Button>
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
                      data-action="export-playbook"
                      aria-label={`Export playbook ${rt.name}`}
                      className="max-md:min-h-11"
                      onClick={() => handleExportPlaybook(rt)}
                    >
                      <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
                      Export
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

                {histOpen && (
                  <div
                    data-routine-history={rt.id}
                    className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-3"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-xs font-medium text-ink">Execution history</span>
                      <span className="mono text-xs text-ink-2">{runs.length} recorded</span>
                    </div>
                    {loadingRuns ? (
                      <p className="text-xs text-ink-2">Loading run history…</p>
                    ) : runs.length === 0 ? (
                      <p className="text-xs text-ink-2">
                        No runs recorded yet. Click Run now or wait for the next scheduled tick.
                      </p>
                    ) : (
                      <ul className="divide-y divide-hairline rounded-sm border border-hairline bg-surface">
                        {runs.slice(0, 10).map((run) => {
                          const okRun = run.status === "finished" || run.status === "ok" || run.status === "completed";
                          const errRun = run.status === "error" || run.status === "failed";
                          const durStr = fmtDuration(run.started_ms, run.finished_ms);
                          return (
                            <li
                              key={run.run_id}
                              data-run-id={run.run_id}
                              className="flex flex-wrap items-center justify-between gap-2 px-2.5 py-2 text-xs"
                            >
                              <div className="min-w-0 flex-1">
                                <div className="flex flex-wrap items-center gap-1.5">
                                  <span
                                    aria-hidden
                                    className={cn(
                                      "size-2 shrink-0 rounded-full",
                                      okRun ? "bg-dot-green" : errRun ? "bg-dot-red" : "bg-dot-amber"
                                    )}
                                  />
                                  <span className="font-medium capitalize text-ink">{run.status}</span>
                                  <span className="mono text-ink-2">{relTime(run.started_ms)}</span>
                                  {durStr && <span className="mono text-ink-3">· {durStr}</span>}
                                  {run.receipt_id && (
                                    <Badge variant="outline" size="sm" className="mono text-xs">
                                      {run.receipt_id.slice(0, 14)}
                                    </Badge>
                                  )}
                                </div>
                                {run.summary && (
                                  <p className="mt-0.5 truncate text-ink-2">{run.summary}</p>
                                )}
                              </div>
                              <div className="flex shrink-0 items-center gap-1">
                                {(run.receipt_id || okRun) && (
                                  <Button
                                    type="button"
                                    size="xs"
                                    variant="ghost"
                                    className="max-md:min-h-11 gap-1 text-xs"
                                    aria-label={`Export receipt for run ${run.run_id}`}
                                    onClick={() => handleExportRunReceipt(rt, run)}
                                  >
                                    <FileDown className="size-3" strokeWidth={1.5} aria-hidden />
                                    Receipt
                                  </Button>
                                )}
                                {run.thread_id && onOpenThread && (
                                  <Button
                                    type="button"
                                    size="xs"
                                    variant="ghost"
                                    className="max-md:min-h-11 gap-1 text-xs"
                                    onClick={() => {
                                      setIsOpen(false);
                                      onOpenThread(run.thread_id!);
                                    }}
                                  >
                                    Chat
                                  </Button>
                                )}
                              </div>
                            </li>
                          );
                        })}
                      </ul>
                    )}
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

export function RoutinesPanel({
  routines,
  skills,
  status,
  scheduler,
  historyRev,
  personName,
  presetSchedule,
  presetNonce,
  open,
  onOpenChange,
  onRefresh,
  onCreate,
  onToggleEnabled,
  onLoadHistory,
  onDelete,
  onRunNow,
  onOpenThread,
  onImportPlaybook,
}: {
  routines: Routine[];
  skills: SkillMeta[];
  status: Status | null | "error";
  scheduler: SchedulerStatus | null;
  historyRev: number;
  personName: string;
  presetSchedule?: string | null;
  presetNonce?: number;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRefresh: () => void;
  onCreate: (input: {
    name: string;
    schedule: string;
    prompt: string;
    skill_id?: string;
    steps?: string[];
    expected_inputs?: string[];
    pack_deliverables?: boolean;
  }) => Promise<boolean>;
  onToggleEnabled: (id: string, enabled: boolean) => Promise<boolean>;
  onLoadHistory: (id: string) => Promise<RoutineRunRecord[]>;
  onDelete: (id: string) => Promise<void>;
  onRunNow: (routine: Routine) => Promise<void>;
  onOpenThread?: (threadId: string) => void;
  onImportPlaybook?: (rawText: string) => Promise<{ ok: boolean; sha256?: string; error?: string }>;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Recurring routines"
      description="Schedule plain-English routines to run automatically or on demand."
      className="max-w-2xl"
    >
      <RoutinesBody
        routines={routines}
        skills={skills}
        status={status}
        scheduler={scheduler}
        historyRev={historyRev}
        personName={personName}
        presetSchedule={presetSchedule}
        presetNonce={presetNonce}
        onRefresh={onRefresh}
        onCreate={onCreate}
        onToggleEnabled={onToggleEnabled}
        onLoadHistory={onLoadHistory}
        onDelete={onDelete}
        onRunNow={onRunNow}
        onOpenThread={onOpenThread}
        onImportPlaybook={onImportPlaybook}
      />
    </PanelModal>
  );
}
