// Today's Desk (Morning Brief & Operational Queue):
// - Morning brief headline with SlidingNumber counters
// - Tremor 14-slot Task Outcome Tracker strip + Geist Mono completion metrics
// - Waiting on you decision approval/decline band with plain consequence copy
// - Finished work & Today so far (/api/today) chronological stream with file-type glyphs
// - Proactive agent capability offers, learned preferences, and desk team roster
import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { toast } from "sonner";
import { ArrowUpRight, CheckCircle2, Clock, Sparkles, XCircle } from "lucide-react";
import { Tracker, type TrackerBlockProps } from "@/components/Tracker/Tracker";
import { SlidingNumber } from "@/components/ui/sliding-number";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { Details, FileGlyph, WaitingPill } from "@/chat/Entry";
import { personInitials } from "@/shell/PersonSelect";
import { call, type Brief, type BriefJob, type Decision, type Offer, type Person, type ThreadRow, type TodayEvent, type TodayPayload } from "@/lib/api";
import { cn } from "@/lib/utils";
import { normalizeFiles, relTime, threadMs } from "@/lib/words";
import { dur, ease, layout } from "@/motion";

type Data = {
  brief: Brief;
  decisions: Decision[];
  offers: Offer[];
  todayEvents: TodayEvent[];
  threads: ThreadRow[];
};

function consequence(kind: string): string {
  if (/delete/.test(String(kind ?? ""))) {
    return "Approving deletes the file for good. Keeping it leaves the file where it is. Your agent has not touched it yet.";
  }
  return "Your agent raised this while it worked and carried on. Your answer is recorded here; to have it act on the answer, tell it in the conversation.";
}

export function PaperSide({ people, teamError, active, onSwitch, offers, learned, failed, onAct }: {
  people: Person[] | null;
  teamError?: string;
  active: string;
  onSwitch: (id: string) => void;
  offers: Offer[] | null;
  learned: string[] | null;
  failed?: boolean;
  onAct: (path: string, done: string) => void;
}) {
  return (
    <div className="flex flex-col gap-6" data-answers="4">
      <section id="offers-list" aria-labelledby="offers-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4">
        <div className="flex items-center justify-between gap-2">
          <h2 id="offers-h" className="text-base font-medium text-ink">Offers from your agent</h2>
          {offers && offers.length > 0 && (
            <Pill variant="secondary" className="bg-tag-plan-bg px-2 py-0.5 text-xs font-medium text-tag-plan-fg">
              <Sparkles className="size-3" strokeWidth={1.5} aria-hidden />
              <span className="mono">{offers.length}</span>
            </Pill>
          )}
        </div>
        {!offers && (
          <p className="text-sm text-ink-2">
            {failed ? "Offers could not be loaded. Retry the desk to try again." : "Loading offers…"}
          </p>
        )}
        <AnimatePresence initial={false}>
          {(offers ?? []).map((o) => (
            <motion.div
              key={o.id}
              layout
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.98 }}
              transition={{ duration: dur.enter, ease }}
              className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-board/60 p-3.5"
            >
              <p className="text-sm font-medium leading-5 text-ink">{String(o.title ?? "Suggested capability")}</p>
              <p className="text-sm leading-5 text-ink-2">{String(o.detail ?? "")}</p>
              <div className="flex flex-wrap gap-2 pt-0.5">
                <Button
                  size="sm"
                  className="max-md:min-h-11"
                  onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/accept`, "Added to your suite as a one-tap app.")}
                >
                  Make it one tap
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/dismiss`, "Offer dismissed.")}
                >
                  No, thanks
                </Button>
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
        {offers && !offers.length && (
          <p className="text-sm leading-5 text-ink-2">
            No offers right now. Your agent proposes a one-tap app when you ask for the same kind of work three times.
          </p>
        )}
      </section>

      <section aria-labelledby="learned-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <h2 id="learned-h" className="text-base font-medium text-ink">What your agent has learned</h2>
        {!learned && <p className="text-sm text-ink-2">{failed ? "Could not be loaded with the brief." : "Loading…"}</p>}
        {learned && learned.length > 0 && (
          <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm leading-5 text-ink-2">
            {learned.map((l, i) => <li key={i} className="text-ink">{String(l)}</li>)}
          </ul>
        )}
        {learned && !learned.length && (
          <p className="text-sm leading-5 text-ink-2">
            Nothing learned yet. Preferences and corrections your agent picks up appear here.
          </p>
        )}
      </section>

      <section aria-labelledby="team-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <h2 id="team-h" className="text-base font-medium text-ink">Team</h2>
        {!people && <p role={teamError ? "alert" : undefined} className="text-sm text-ink-2">{teamError || "Loading the team…"}</p>}
        {people && !people.length && (
          <p className="text-sm text-ink-2">No people yet. Add someone from the rail and their agent keeps its own workspace and memory.</p>
        )}
        {people && people.length > 0 && (
          <ul className="flex flex-col gap-1">
            {people.map((p) => {
              const isCurrent = p.id === active;
              const pName = String(p?.name ?? "").trim() || "Unnamed";
              const pRole = String(p?.role ?? "").trim() || "Personal workspace";
              return (
                <li key={p.id}>
                  <Button
                    variant="ghost"
                    aria-current={isCurrent ? "true" : undefined}
                    onClick={() => onSwitch(p.id)}
                    className={cn(
                      "h-auto sm:h-auto min-h-11 w-full justify-start gap-2.5 whitespace-normal px-2.5 py-1.5 text-left font-normal",
                      isCurrent ? "bg-board" : "hover:bg-board"
                    )}
                  >
                    <span
                      aria-hidden
                      className={cn(
                        "grid size-7 shrink-0 place-items-center rounded-full text-xs font-medium",
                        isCurrent ? "bg-ink text-canvas" : "bg-blue-soft text-ink"
                      )}
                    >
                      {personInitials(pName)}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className={cn("block truncate text-sm text-ink", isCurrent && "font-medium")}>{pName}</span>
                      <span className="block truncate text-xs text-ink-2">{pRole}</span>
                    </span>
                  </Button>
                </li>
              );
            })}
          </ul>
        )}
      </section>
    </div>
  );
}

export function usePaper(onChanged: () => void) {
  const [data, setData] = useState<Data | null>(null);
  const [failed, setFailed] = useState(false);

  const load = useCallback(async () => {
    const [b, d, o, td, t] = await Promise.all([
      call<Brief>("/api/brief"),
      call<{ decisions: Decision[] }>("/api/decisions"),
      call<{ offers: Offer[] }>("/api/offers"),
      call<TodayPayload>("/api/today"),
      call<{ threads: ThreadRow[] }>("/api/threads"),
    ]);
    const shaped = (x: unknown, key: string) => !!x && typeof x === "object" && Array.isArray((x as Record<string, unknown>)[key]);
    if (
      !b.ok || !d.ok || !o.ok || !td.ok || !t.ok
      || !shaped(b.data, "finished")
      || !shaped(d.data, "decisions")
      || !shaped(o.data, "offers")
      || !shaped(td.data, "events")
      || !shaped(t.data, "threads")
    ) {
      setFailed(true);
      return;
    }
    setFailed(false);
    setData({
      brief: b.data,
      decisions: (d.data.decisions ?? []).filter((x): x is Decision => !!x && typeof x === "object"),
      offers: (o.data.offers ?? []).filter((x): x is Offer => !!x && typeof x === "object"),
      todayEvents: (td.data.events ?? []).filter((x): x is TodayEvent => !!x && typeof x === "object"),
      threads: (t.data.threads ?? []).filter((x): x is ThreadRow => !!x && typeof x === "object"),
    });
  }, []);

  useEffect(() => { load(); }, [load]);

  const act = useCallback(async (path: string, done: string) => {
    const r = await call<{ outcome?: string }>(path, { method: "POST", body: {} });
    if (!r.ok) {
      toast.error(r.error ? "That did not go through: " + r.error : "That did not go through. Try again.");
      return;
    }
    toast.success(typeof r.data?.outcome === "string" && r.data.outcome ? r.data.outcome : done);
    await load();
    onChanged();
  }, [load, onChanged]);

  return { data, failed, load, act };
}

export function PaperFace({ paper, personName, dateLabel, people, teamError, activePerson, onSwitchPerson, onAsk, onOpen, onOpenFile }: {
  paper: ReturnType<typeof usePaper>;
  personName: string;
  dateLabel: string;
  people: Person[] | null;
  teamError: string;
  activePerson: string;
  onSwitchPerson: (id: string) => void;
  onAsk: () => void;
  onOpen: (threadId: string) => void;
  onOpenFile: (path: string) => void;
}) {
  const { data, failed, load, act } = paper;
  const pending = data ? data.decisions.slice().reverse().filter((d) => d.status === "pending") : [];
  const settled = data ? data.decisions.slice().reverse().filter((d) => d.status !== "pending") : [];
  const threadOf = (x: BriefJob) => x.thread_id ?? data?.threads.find((t) => t.title === x.task)?.id;
  const when = (x: BriefJob) => {
    const id = threadOf(x);
    const ms = typeof x.ts === "number" ? x.ts : (id ? threadMs(id) : null);
    return ms ? relTime(ms) : String(x.date ?? "");
  };
  const waitingThreads = new Set(pending.map((d) => d.thread_id).filter(Boolean));
  const finished = (data?.brief.finished ?? [])
    .filter((x): x is BriefJob => !!x && typeof x === "object")
    .filter((x) => { const id = threadOf(x); return !id || !waitingThreads.has(id); });
  const stopped = (data?.brief.stopped ?? []).filter((x): x is BriefJob => !!x && typeof x === "object");
  const lead = pending[0];
  const n = finished.length;
  const w = pending.length;
  const totalSavedFiles = finished.reduce((acc, job) => acc + normalizeFiles(job.files).length, 0);

  const trackerBlocks = useMemo<TrackerBlockProps[]>(() => {
    const SLOTS = 14;
    const real: { ts: number; color: string; tooltip: string }[] = [];
    finished.forEach((f, idx) => {
      const files = normalizeFiles(f.files);
      const filePart = files.length ? ` · Saved ${files.join(", ")}` : "";
      real.push({
        ts: typeof f.ts === "number" ? f.ts : idx + 1,
        color: "bg-dot-green",
        tooltip: `Finished: ${String(f.task ?? "Task")}${filePart}`,
      });
    });
    pending.forEach((d, idx) => {
      real.push({
        ts: typeof d.ts === "number" ? d.ts : 1000 + idx,
        color: "bg-dot-amber",
        tooltip: `Waiting on you: ${String(d.summary ?? "Approval needed")}`,
      });
    });
    stopped.forEach((s, idx) => {
      real.push({
        ts: typeof s.ts === "number" ? s.ts : 500 + idx,
        color: "bg-dot-red",
        tooltip: `Stopped early: ${String(s.task ?? "Task")}`,
      });
    });
    real.sort((a, b) => a.ts - b.ts);
    const recentSlice = real.slice(-SLOTS);
    const padCount = Math.max(0, SLOTS - recentSlice.length);
    const emptyBlocks: TrackerBlockProps[] = Array.from({ length: padCount }, (_, i) => ({
      key: `empty-${i}`,
      color: "bg-track",
      tooltip: "Open run slot — no task recorded here yet",
      hoverEffect: true,
    }));
    return [
      ...emptyBlocks,
      ...recentSlice.map((item, i) => ({
        key: `run-${i}-${item.ts}`,
        color: item.color,
        tooltip: item.tooltip,
        hoverEffect: true,
      })),
    ];
  }, [finished, pending, stopped]);

  const sideColumn = (
    <PaperSide
      people={people}
      teamError={teamError}
      active={activePerson}
      onSwitch={onSwitchPerson}
      offers={data ? data.offers.slice().reverse().filter((o) => o.status === "pending") : null}
      learned={data ? (Array.isArray(data.brief.learned_recently) ? data.brief.learned_recently : []) : null}
      failed={failed}
      onAct={act}
    />
  );

  return (
    <div className="mx-auto w-full max-w-[1160px] px-4 pb-14 pt-5 md:px-6">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-8 xl:grid-cols-[minmax(0,1fr)_312px]">
        <main className="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-7" aria-label="Today's paper">
          {failed && (
            <div role="alert" className="rounded-entry border border-hairline bg-surface p-5">
              <h1 className="text-title font-medium text-ink">Today&apos;s paper could not be loaded</h1>
              <p className="mt-1 text-sm text-ink-2">Check that Ergon is running, then try again.</p>
              <Button className="mt-3 max-md:min-h-11" variant="outline" onClick={load}>Retry</Button>
            </div>
          )}

          {!failed && !data && (
            <div aria-busy="true" className="grid gap-4">
              <div className="h-12 w-4/5 rounded-sm bg-board" />
              <div className="h-20 rounded-card bg-board" />
              <div className="h-32 rounded-card bg-board" />
              <span className="sr-only">Loading today&apos;s paper</span>
            </div>
          )}

          {data && (
            <>
              <header className="grid gap-3 border-b border-hairline pb-5">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <p id="dateline" className="text-sm font-medium text-ink-2">
                    {dateLabel}{personName ? `, for ${personName === "You" ? "you" : personName}` : ""}
                  </p>
                  <Button variant="outline" size="sm" className="max-md:min-h-11" data-answers="3" onClick={onAsk}>
                    Ask your agent for something
                  </Button>
                </div>
                <h1
                  id="brief-headline"
                  data-answers="1"
                  className="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-[30px] leading-[1.12] font-medium tracking-[-0.035em] text-ink [text-wrap:wrap] md:text-[40px]"
                >
                  {n ? (
                    <span className="inline-flex items-center gap-2">
                      <SlidingNumber value={n} />
                      <span>{n === 1 ? "thing" : "things"} finished,</span>
                    </span>
                  ) : w ? (
                    <span>Nothing finished yet,</span>
                  ) : (
                    <span>Nothing finished yet.</span>
                  )}
                  {w ? (
                    <span className="inline-flex items-center gap-2">
                      <SlidingNumber value={w} />
                      <span>{w === 1 ? "waits" : "wait"} on you.</span>
                    </span>
                  ) : n ? (
                    <span>nothing waits on you.</span>
                  ) : null}
                </h1>
                <p className="max-w-[64ch] text-ui leading-6 text-ink-2">
                  {lead
                    ? `The decision below is the only item parked on you${w > 1 ? " first" : " today"}. Autonomous tasks finished in the last 48 hours are filed underneath with their workspace files.`
                    : n
                      ? "Your agent has nothing waiting on your approval. Completed work from the last 48 hours is filed below."
                      : "Describe a task in plain words. Your agent runs real workspace tools, saves the output file, and asks here before deleting anything."}
                </p>
              </header>

              {/* Tremor Operational Outcome Strip */}
              <section
                aria-label="Recent task outcomes"
                data-answers="2"
                className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <h2 className="text-sm font-medium text-ink">Recent run outcomes</h2>
                    <span className="text-xs text-ink-3">(last 14 run slots)</span>
                  </div>
                  <div className="mono flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-2">
                    <span className="inline-flex items-center gap-1.5 text-ink">
                      <span aria-hidden className="size-2 rounded-full bg-dot-green" />
                      <SlidingNumber value={n} /> finished
                    </span>
                    <span className="inline-flex items-center gap-1.5 text-ink">
                      <span aria-hidden className="size-2 rounded-full bg-dot-amber" />
                      <SlidingNumber value={w} /> waiting
                    </span>
                    {stopped.length > 0 && (
                      <span className="inline-flex items-center gap-1.5 text-ink">
                        <span aria-hidden className="size-2 rounded-full bg-dot-red" />
                        <SlidingNumber value={stopped.length} /> stopped
                      </span>
                    )}
                    <span className="inline-flex items-center gap-1 text-ink-2">
                      <SlidingNumber value={totalSavedFiles} />
                      <span>{totalSavedFiles === 1 ? "file" : "files"} saved</span>
                    </span>
                  </div>
                </div>
                <Tracker data={trackerBlocks} hoverEffect />
              </section>

              {/* Waiting on you Approval Queue */}
              <div id="decisions-list">
                <AnimatePresence initial={false} mode="popLayout">
                  {lead ? (
                    <motion.section
                      key={lead.id}
                      layout
                      initial={{ opacity: 0, y: 6 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, scale: 0.98 }}
                      transition={layout}
                      aria-labelledby="lead-h"
                      className="grid gap-3.5 rounded-card border border-hairline bg-blue-soft p-5"
                    >
                      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                        <h2 id="lead-h" className="min-w-0 text-[22px] leading-snug font-medium tracking-[-0.025em] text-ink [overflow-wrap:anywhere]">
                          {String(lead.summary ?? "Decision waiting on you")}
                        </h2>
                        <WaitingPill />
                      </div>
                      <p className="max-w-[60ch] text-sm leading-6 text-ink-2">{consequence(String(lead.kind ?? ""))}</p>
                      <div className="flex flex-wrap gap-2">
                        <Button
                          className="min-h-11 px-4"
                          onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/approve`, "Approved and done.")}
                        >
                          Approve
                        </Button>
                        <Button
                          className="min-h-11 px-4"
                          variant="outline"
                          onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/deny`, "You said no; kept safe.")}
                        >
                          {/delete/.test(String(lead.kind ?? "")) ? "Keep the file" : "Decline"}
                        </Button>
                      </div>
                      {w > 1 && (
                        <p className="text-xs text-ink-2">
                          <span className="mono">{w - 1}</span> more {w - 1 === 1 ? "decision waits" : "decisions wait"} after this one.
                        </p>
                      )}
                    </motion.section>
                  ) : n > 0 ? (
                    <motion.p
                      key="quiet"
                      layout
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      transition={{ duration: dur.swap, ease }}
                      className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2"
                    >
                      Nothing waits on you. Your agent asks here before it deletes anything.
                    </motion.p>
                  ) : null}
                </AnimatePresence>
                {settled.length > 0 && (
                  <div className="mt-3">
                    <Details label={`Recently settled (${settled.length})`}>
                      <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface px-3.5">
                        {settled.slice(0, 5).map((s) => (
                          <li key={s.id} className="flex flex-wrap items-center justify-between gap-2 py-2.5 text-sm">
                            <span className="min-w-0 truncate font-medium text-ink">{String(s.summary ?? "")}</span>
                            <span className="text-xs text-ink-2">{String(s.outcome ?? s.status ?? "")}</span>
                          </li>
                        ))}
                      </ul>
                    </Details>
                  </div>
                )}
              </div>

              {/* Finished Work Ledger */}
              {n > 0 && (
                <section aria-labelledby="finished-h" className="grid gap-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <h2 id="finished-h" className="text-title font-medium text-ink">Finished</h2>
                    <span className="mono text-xs text-ink-3">{n} in last 48h</span>
                  </div>
                  <motion.ol layout className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                    {finished.map((x, i) => {
                      const id = threadOf(x);
                      const files = normalizeFiles(x.files);
                      const taskTitle = String(x.task ?? "Completed task");
                      return (
                        <motion.li
                          key={taskTitle + String(x.date ?? "") + i}
                          layout
                          initial={false}
                          transition={layout}
                          className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 px-4 py-3.5 md:grid-cols-[104px_minmax(0,1fr)_auto]"
                        >
                          <span className="mono col-span-2 text-xs text-ink-2 md:col-span-1">{when(x)}</span>
                          <div className="min-w-0">
                            <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                            <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1.5 text-xs text-ink-2">
                              {files.length ? (
                                <>
                                  <span>Saved</span>
                                  {files.map((f, k) => (
                                    <Button
                                      key={f + k}
                                      size="xs"
                                      variant="outline"
                                      aria-label={`Open ${f}`}
                                      onClick={() => onOpenFile(f)}
                                      className="num max-md:min-h-11 max-w-full gap-1.5 rounded-chip border-hairline bg-board/60 px-2 text-xs font-normal text-ink hover:bg-board"
                                    >
                                      <FileGlyph name={f} />
                                      <span className="truncate">{f}</span>
                                      <ArrowUpRight className="size-3 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                                    </Button>
                                  ))}
                                </>
                              ) : (
                                "Nothing saved to your workspace"
                              )}
                            </div>
                          </div>
                          {id && (
                            <Button
                              size="sm"
                              variant="ghost"
                              className="min-h-11"
                              aria-label={`Open the conversation: ${taskTitle}`}
                              onClick={() => onOpen(id)}
                            >
                              Open
                              <ArrowUpRight className="size-4" strokeWidth={1.5} aria-hidden />
                            </Button>
                          )}
                        </motion.li>
                      );
                    })}
                  </motion.ol>
                </section>
              )}

              {/* Stopped / Did not finish */}
              {stopped.length > 0 && (
                <section aria-labelledby="stopped-h" className="grid gap-3">
                  <h2 id="stopped-h" className="text-title font-medium text-ink">Did not finish</h2>
                  <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                    {stopped.map((x, i) => {
                      const id = threadOf(x);
                      const taskTitle = String(x.task ?? "Stopped task");
                      return (
                        <li
                          key={(x.thread_id ?? taskTitle) + i}
                          className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 px-4 py-3.5 md:grid-cols-[104px_minmax(0,1fr)_auto]"
                        >
                          <span className="mono col-span-2 text-xs text-ink-2 md:col-span-1">{when(x)}</span>
                          <div className="min-w-0">
                            <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                            {x.summary && <p className="mt-0.5 text-sm leading-5 text-ink-2 [overflow-wrap:anywhere]">{String(x.summary)}</p>}
                          </div>
                          {id && (
                            <Button
                              size="sm"
                              variant="ghost"
                              className="min-h-11"
                              aria-label={`Open the conversation: ${taskTitle}`}
                              onClick={() => onOpen(id)}
                            >
                              Open
                              <ArrowUpRight className="size-4" strokeWidth={1.5} aria-hidden />
                            </Button>
                          )}
                        </li>
                      );
                    })}
                  </ol>
                </section>
              )}

              {/* Today so far (/api/today chronological activity stream) */}
              <section aria-labelledby="today-stream-h" className="grid gap-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h2 id="today-stream-h" className="text-title font-medium text-ink">Today so far</h2>
                  <span className="mono text-xs text-ink-3">{data.todayEvents.length} {data.todayEvents.length === 1 ? "event" : "events"}</span>
                </div>
                {data.todayEvents.length === 0 ? (
                  <p className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2">
                    No activity logged yet today. New tool runs, decisions, and capability offers appear here in order.
                  </p>
                ) : (
                  <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                    {data.todayEvents.slice(0, 8).map((ev, i) => {
                      const files = normalizeFiles(ev.files);
                      const kind = String(ev.kind ?? "work");
                      const EventIcon = kind === "decision" ? Clock : kind === "offer" ? Sparkles : kind === "decision_settled" ? CheckCircle2 : CheckCircle2;
                      const iconCls = kind === "decision" ? "text-dot-amber" : kind === "offer" ? "text-dot-clay" : "text-dot-green";
                      const timeStr = typeof ev.ts === "number" ? relTime(ev.ts) : "";
                      return (
                        <li key={String(ev.ts ?? i) + "-" + i} className="flex items-start gap-3 px-4 py-3">
                          {kind === "stopped" ? (
                            <XCircle className="mt-0.5 size-4 shrink-0 text-dot-red" strokeWidth={1.5} aria-hidden />
                          ) : (
                            <EventIcon className={cn("mt-0.5 size-4 shrink-0", iconCls)} strokeWidth={1.5} aria-hidden />
                          )}
                          <div className="min-w-0 flex-1">
                            <div className="flex flex-wrap items-baseline justify-between gap-2">
                              <p className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{String(ev.title ?? "Activity")}</p>
                              {timeStr && <span className="mono shrink-0 text-xs text-ink-3">{timeStr}</span>}
                            </div>
                            {ev.detail && <p className="mt-0.5 text-sm text-ink-2 [overflow-wrap:anywhere]">{String(ev.detail)}</p>}
                            {files.length > 0 && (
                              <div className="mt-1.5 flex flex-wrap items-center gap-2">
                                {files.map((f, idx) => (
                                  <Button
                                    key={f + idx}
                                    size="xs"
                                    variant="outline"
                                    aria-label={`Open ${f}`}
                                    onClick={() => onOpenFile(f)}
                                    className="mono max-md:min-h-11 max-w-full gap-1 rounded-chip border-hairline bg-board px-2 text-xs font-normal text-ink"
                                  >
                                    <FileGlyph name={f} className="size-3.5" />
                                    <span className="truncate">{f}</span>
                                    <ArrowUpRight className="size-3 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                                  </Button>
                                ))}
                              </div>
                            )}
                          </div>
                        </li>
                      );
                    })}
                  </ol>
                )}
              </section>
            </>
          )}
        </main>

        <aside aria-label="Offers, memory and team">{sideColumn}</aside>
      </div>
    </div>
  );
}
