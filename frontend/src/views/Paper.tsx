// Today's paper: the agent's day set as a printed morning brief, the first screen of Ergon.
// The headline counts what was finished and what waits on the person; the one waiting decision leads in a tinted band;
// finished work is filed underneath as field-book entries; the side column carries the team, offers and what was learned.
// Every region reads the real Ergon API and has its loading, empty and error state.
import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, MotionConfig, motion } from "motion/react";
import NumberFlow from "@number-flow/react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ArrowUpRight } from "lucide-react";
import { WaitingPill } from "@/chat/Entry";
import { call, type Decision, type Offer, type Person, type ThreadRow } from "@/lib/api";
import { cn } from "@/lib/utils";
import { relTime, threadMs } from "@/lib/words";

type Brief = { headline?: string; finished?: { task: string; date: string; summary?: string; files?: string[] }[]; suite_size?: number; learned_recently?: string[] };
type Data = { brief: Brief; decisions: (Decision & { thread_id?: string })[]; offers: Offer[]; threads: ThreadRow[] };
const ease = [0.23, 1, 0.32, 1] as const;
const tight = { "--number-flow-mask-height": "0em" } as React.CSSProperties;

function consequence(kind: string): string {
  if (/delete/.test(kind)) return "Approving deletes the file for good. Keeping it leaves the file where it is. Your agent has not touched it yet.";
  return "Approving lets your agent go ahead. Keeping it stops the step, and the conversation records what happened.";
}

function initials(name: string): string {
  return name.split(/\s+/).filter(Boolean).slice(0, 2).map((w) => w[0]?.toUpperCase() ?? "").join("") || "?";
}

export function PaperSide({ people, active, onSwitch, offers, learned, failed, onAct, extra }: {
  people: Person[] | null; active: string; onSwitch: (id: string) => void; offers: Offer[] | null; learned: string[] | null;
  failed?: boolean; onAct: (path: string, done: string) => void; extra?: React.ReactNode;
}) {
  return (
    <div className="grid gap-8">
      <section aria-labelledby="team-h" className="grid gap-3">
        <h2 id="team-h" className="text-title font-semibold tracking-[-0.02em] text-ink">Team</h2>
        {!people && <p className="text-sm text-ink-2">Loading the team…</p>}
        {people && !people.length && <p className="text-sm text-ink-2">No people yet. Add someone below and their agent keeps its own memory.</p>}
        {people && people.length > 0 && (
          <ul className="grid gap-1">
            {people.map((p) => (
              <li key={p.id}>
                <button type="button" aria-current={p.id === active ? "true" : undefined} onClick={() => onSwitch(p.id)}
                  className={cn("focus-ring flex min-h-11 w-full items-center gap-3 rounded-sm px-2 text-left text-sm outline-none transition-colors duration-150 ease-out",
                    p.id === active ? "bg-board" : "hover:bg-board")}>
                  <span aria-hidden className={cn("grid size-8 shrink-0 place-items-center rounded-full text-xs font-semibold", p.id === active ? "bg-ink text-canvas" : "bg-blue-soft text-ink")}>{initials(p.name)}</span>
                  <span className="min-w-0 flex-1"><span className={cn("block truncate text-ink", p.id === active && "font-medium")}>{p.name}</span><span className="block truncate text-xs text-ink-2">{p.role || "No role set"}</span></span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section aria-labelledby="offers-h" className="grid gap-3">
        <h2 id="offers-h" className="text-title font-semibold tracking-[-0.02em] text-ink">Offers</h2>
        {!offers && <p className="text-sm text-ink-2">{failed ? "Offers could not be loaded. Retry the paper to try again." : "Loading offers…"}</p>}
        <AnimatePresence initial={false}>
          {(offers ?? []).map((o) => (
            <motion.div key={o.id} layout initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, scale: 0.97 }} transition={{ duration: 0.24, ease }}
              className="grid gap-2 rounded-card bg-board p-4">
              <p className="text-sm font-medium leading-5 text-ink">{o.title}</p>
              <p className="text-sm leading-6 text-ink-2">{o.detail}</p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" className="max-md:min-h-11" onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/accept`, "Added to your suite as a one-tap app.")}>Make it one tap</Button>
                <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/dismiss`, "Offer dismissed.")}>Not now</Button>
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
        {offers && !offers.length && <p className="text-sm leading-6 text-ink-2">No offers right now. Your agent proposes one when you ask for the same kind of work three times.</p>}
      </section>
      <section aria-labelledby="learned-h" className="grid gap-2">
        <h2 id="learned-h" className="text-title font-semibold tracking-[-0.02em] text-ink">What your agent has learned</h2>
        {!learned && <p className="text-sm text-ink-2">{failed ? "Could not be loaded with the paper." : "Loading…"}</p>}
        {learned && learned.length > 0 && <ul className="grid list-disc gap-1.5 pl-5 text-sm leading-6 text-ink-2">{learned.map((l, i) => <li key={i}>{l}</li>)}</ul>}
        {learned && !learned.length && <p className="text-sm leading-6 text-ink-2">Nothing learned yet. When you correct your agent, it remembers here.</p>}
      </section>
      {extra}
    </div>
  );
}

export function usePaper(onChanged: () => void) {
  const [data, setData] = useState<Data | null>(null);
  const [failed, setFailed] = useState(false);
  const load = useCallback(async () => {
    const [b, d, o, t] = await Promise.all([
      call<Brief>("/api/brief"), call<{ decisions: Data["decisions"] }>("/api/decisions"), call<{ offers: Offer[] }>("/api/offers"),
      call<{ threads: ThreadRow[] }>("/api/threads"),
    ]);
    if (!b.ok || !d.ok || !o.ok || !t.ok) { setFailed(true); return; }
    setFailed(false);
    setData({ brief: b.data, decisions: d.data.decisions ?? [], offers: o.data.offers ?? [], threads: t.data.threads ?? [] });
  }, []);
  useEffect(() => { load(); }, [load]);
  const act = useCallback(async (path: string, done: string) => {
    const r = await call(path, { method: "POST", body: {} });
    if (!r.ok) { toast("That did not go through. Try again."); return; }
    toast(done);
    await load(); onChanged();
  }, [load, onChanged]);
  return { data, failed, load, act };
}

export function PaperFace({ paper, personName, dateLabel, side, onAsk, onOpen }: {
  paper: ReturnType<typeof usePaper>; personName: string; dateLabel: string; side: React.ReactNode; onAsk: () => void; onOpen: (threadId: string) => void;
}) {
  const { data, failed, load, act } = paper;
  const pending = data ? data.decisions.slice().reverse().filter((d) => d.status === "pending") : [];
  // A run that parked a decision ended, but its job is not done: it waits in the band above, not in Finished.
  const threadOf = (task: string) => data?.threads.find((t) => t.title === task)?.id;
  const waitingThreads = new Set(pending.map((d) => d.thread_id).filter(Boolean));
  const finished = (data?.brief.finished ?? []).filter((x) => { const id = threadOf(x.task); return !id || !waitingThreads.has(id); });
  const lead = pending[0];
  const n = finished.length, w = pending.length;
  return (
    <MotionConfig reducedMotion="user">
      <div className="mx-auto w-full max-w-[1200px] px-6 pb-16">
        <div className="mt-5 grid grid-cols-[minmax(0,1fr)] gap-10 border-t-2 border-ink pt-8 lg:grid-cols-[minmax(0,1fr)_300px]">
          <main className="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-10" aria-label="Today's paper">
            {failed && (
              <div role="alert" className="rounded-entry border border-hairline bg-surface px-4 py-4">
                <h1 className="text-title font-semibold text-ink">Today's paper could not be loaded</h1>
                <p className="mt-1 text-sm text-ink-2">Check that Ergon is running, then try again.</p>
                <Button className="mt-3" variant="outline" onClick={load}>Retry</Button>
              </div>
            )}
            {!failed && !data && (
              <div aria-busy="true" className="grid gap-4">
                <div className="h-14 w-4/5 rounded-sm bg-board" /><div className="h-5 w-3/5 rounded-sm bg-board" /><div className="h-36 rounded-card bg-board" />
                <span className="sr-only">Loading today's paper</span>
              </div>
            )}
            {data && (
              <>
                <header className="grid gap-4">
                  <h1 className="text-[44px] leading-[1.02] font-semibold tracking-[-0.04em] text-ink md:text-[64px]">
                    {n ? <><NumberFlow value={n} style={tight} /> {n === 1 ? "thing" : "things"} finished,</> : w ? "Nothing finished yet," : "Nothing finished yet."}{" "}
                    {w ? <><NumberFlow value={w} style={tight} /> {w === 1 ? "waits" : "wait"} on you.</> : n ? "nothing waits on you." : null}
                  </h1>
                  <p id="dateline" className="text-sm font-medium text-ink">{dateLabel}{personName ? `, for ${personName === "You" ? "you" : personName}` : ""}</p>
                  <p className="max-w-[60ch] text-base leading-7 text-ink-2">
                    {lead ? `The decision below is the only thing that needs you${w > 1 ? " first" : " today"}. Everything else your agent finished in the last two days is filed underneath.`
                      : n ? "Your agent has nothing to ask. Everything it finished in the last two days is filed underneath." : "Ask your agent for something in plain words. When it finishes, the result is filed here, and anything it needs from you waits at the top."}
                  </p>
                  {!n && !w && <div><Button className="min-h-11 px-5 sm:min-h-11" onClick={onAsk}>Ask your agent for something</Button></div>}
                </header>

                <AnimatePresence initial={false} mode="popLayout">
                  {lead ? (
                    <motion.section key={lead.id} layout initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, scale: 0.97 }} transition={{ duration: 0.28, ease }}
                      aria-labelledby="lead-h" className="grid gap-4 rounded-card bg-blue-soft p-5 md:p-7">
                      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                        <h2 id="lead-h" className="text-[26px] leading-tight font-semibold tracking-[-0.03em] text-ink">{lead.summary}</h2>
                        <WaitingPill />
                      </div>
                      <p className="max-w-[58ch] text-base leading-7 text-ink-2">{consequence(lead.kind)}</p>
                      <div className="flex flex-wrap gap-2">
                        <Button className="min-h-11 px-5 sm:min-h-11" onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/approve`, "Approved. Your agent went ahead.")}>Approve</Button>
                        <Button className="min-h-11 px-5 sm:min-h-11" variant="outline" onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/deny`, "Kept. Nothing was changed.")}>Keep</Button>
                      </div>
                      {w > 1 && <p className="text-sm text-ink-2">{w - 1} more {w - 1 === 1 ? "decision waits" : "decisions wait"} after this one.</p>}
                    </motion.section>
                  ) : n > 0 && (
                    <motion.p key="quiet" layout initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
                      className="rounded-card border border-hairline p-4 text-sm leading-6 text-ink-2">
                      Nothing waits on you. Your agent asks here before it deletes or changes anything.
                    </motion.p>
                  )}
                </AnimatePresence>

                {n > 0 && <section aria-labelledby="finished-h" className="grid gap-4">
                  <h2 id="finished-h" className="text-[24px] font-semibold tracking-[-0.03em] text-ink">Finished</h2>
                  {n > 0 && (
                    <motion.ol layout className="grid grid-cols-[minmax(0,1fr)] border-t-2 border-ink">
                      {finished.map((x, i) => {
                        const id = threadOf(x.task);
                        return (
                          <motion.li key={x.task + x.date + i} layout initial={false} transition={{ duration: 0.28, ease }}
                            className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 border-b border-hairline py-4 md:grid-cols-[112px_minmax(0,1fr)_auto]">
                            <span className="col-span-2 text-sm text-ink-2 md:col-span-1">{(() => { const ms = id ? threadMs(id) : null; return ms ? relTime(ms) : x.date; })()}</span>
                            <div className="min-w-0">
                              <p className="text-base font-medium leading-6 text-ink">{x.task}</p>
                              <p className="mt-0.5 truncate text-sm text-ink-2">{x.files?.length ? <>Saved <span className="num text-ink">{x.files.join(", ")}</span></> : "Nothing saved to your workspace"}</p>
                            </div>
                            {id && (
                              <Button size="sm" variant="ghost" className="min-h-11" aria-label={`Open the conversation: ${x.task}`} onClick={() => onOpen(id)}>
                                Open<ArrowUpRight className="size-4" strokeWidth={1.75} aria-hidden />
                              </Button>
                            )}
                          </motion.li>
                        );
                      })}
                    </motion.ol>
                  )}
                </section>}
              </>
            )}
          </main>
          <aside aria-label="Side notes" className="hidden lg:block lg:border-l lg:border-hairline lg:pl-8">{side}</aside>
        </div>
      </div>
    </MotionConfig>
  );
}
