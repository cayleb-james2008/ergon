// Today: the morning brief, waiting decisions, offers, what the agent learned, and today so far, as field-book entries.
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Cell, Cells, Details, EntryShell, WaitingPill, FinishedPill } from "@/chat/Entry";
import { call, type Decision, type Offer } from "@/lib/api";
import { FAILURE_RE } from "@/lib/words";

type Brief = { headline?: string; finished?: { task: string; date: string }[]; suite_size?: number; library_size?: number; learned_recently?: string[] };
type Stream = { events?: { ts?: number; kind?: string; title?: string; detail?: string }[] };
type Data = { brief: Brief; decisions: Decision[]; offers: Offer[]; today: Stream };

function consequence(kind: string): string {
  if (/delete/.test(kind)) return "Approving deletes the file for good. Declining keeps it.";
  return "Approving lets your agent go ahead. Declining stops it, and the thread records what happened.";
}

export function TodayFace({ onChanged }: { onChanged: () => void }) {
  const [data, setData] = useState<Data | null>(null);
  const [failed, setFailed] = useState(false);

  const load = async () => {
    const [b, d, o, t] = await Promise.all([
      call<Brief>("/api/brief"), call<{ decisions: Decision[] }>("/api/decisions"),
      call<{ offers: Offer[] }>("/api/offers"), call<Stream>("/api/today"),
    ]);
    if (!b.ok || !d.ok || !o.ok || !t.ok) { setFailed(true); return; }
    setFailed(false);
    setData({ brief: b.data, decisions: d.data.decisions ?? [], offers: o.data.offers ?? [], today: t.data });
  };
  useEffect(() => { load(); }, []);

  const act = async (path: string, done: string) => {
    const r = await call(path, { method: "POST", body: {} });
    if (!r.ok) { toast("That did not go through. Try again."); return; }
    toast(done);
    await load(); onChanged();
  };

  if (failed) {
    return (
      <section aria-label="Today" className="mx-auto w-full max-w-[760px] px-6 pt-6">
        <div role="alert" className="rounded-entry border border-hairline bg-surface px-4 py-4">
          <h2 className="text-title font-semibold text-ink">Today could not be loaded</h2>
          <p className="mt-1 text-sm text-ink-2">Check that Ergon is running, then try again.</p>
          <Button className="mt-3" variant="outline" onClick={load}>Retry</Button>
        </div>
      </section>
    );
  }
  const today = new Date().toLocaleDateString([], { month: "short", day: "numeric" });
  if (!data) return <section aria-label="Today" className="mx-auto w-full max-w-[760px] px-6 pt-6"><p className="text-sm text-ink-2">Loading…</p></section>;

  const pending = data.decisions.slice().reverse().filter((d) => d.status === "pending");
  const settled = data.decisions.slice().reverse().filter((d) => d.status !== "pending");
  const offers = data.offers.slice().reverse();
  const finished = data.brief.finished ?? [];
  const learned = data.brief.learned_recently ?? [];
  const events = data.today.events ?? [];
  return (
    <section aria-label="Today" className="mx-auto flex w-full max-w-[760px] flex-col gap-6 px-6 pb-10 pt-6">
      <EntryShell label="Morning brief" time={today} title="Morning brief">
        <div className="flex flex-col gap-3 px-4 py-4">
          <h2 className="text-title font-semibold text-ink">{data.brief.headline || "Nothing to report yet."}</h2>
          {finished.length ? (
            <ul className="divide-y divide-hairline">
              <li className="pb-1 text-xs text-ink-3">Finished in the last 48 hours</li>
              {finished.map((f, i) => <li key={i} className="flex items-center justify-between gap-3 py-2 text-sm text-ink"><span className="min-w-0 truncate">{f.task}</span><span className="shrink-0 font-mono text-xs text-ink-3">{f.date}</span></li>)}
            </ul>
          ) : <p className="text-sm text-ink-2">Nothing finished in the last 48 hours. Work your agent completes lands here.</p>}
        </div>
        <Cells>
          <Cell label="Items in your suite">{String(data.brief.suite_size ?? 0)}</Cell>
          <Cell label="In the company library">{String(data.brief.library_size ?? 0)}</Cell>
        </Cells>
      </EntryShell>

      <section aria-label="Waiting on you" className="flex flex-col gap-3">
        <h2 className="text-sm font-medium text-ink">Waiting on you</h2>
        {!pending.length && <p className="text-sm text-ink-2">Nothing is waiting on you.</p>}
        {pending.map((d) => (
          <EntryShell key={d.id} label="Waiting on you" time={today} title={d.summary} status={<WaitingPill />}>
            <div className="flex flex-col gap-3 px-4 py-4">
              <p className="text-sm text-ink-2">{consequence(d.kind)}</p>
              <div className="flex flex-wrap gap-2">
                <Button onClick={() => act(`/api/decisions/${encodeURIComponent(d.id)}/approve`, "Approved.")}>Approve</Button>
                <Button variant="outline" onClick={() => act(`/api/decisions/${encodeURIComponent(d.id)}/deny`, "Declined.")}>Decline</Button>
              </div>
            </div>
          </EntryShell>
        ))}
        {settled.length > 0 && (
          <Details label={`Recently settled (${settled.length})`}>
            {settled.map((d) => (
              <div key={d.id} className="flex flex-col gap-1 rounded-card border border-hairline bg-surface px-3 py-2.5">
                <span className="text-sm text-ink">{d.summary}</span>
                <span className="text-xs text-ink-2">{d.status}{d.outcome ? ": " + d.outcome : ""}</span>
              </div>
            ))}
          </Details>
        )}
      </section>

      <section aria-label="Offers from your agent" className="flex flex-col gap-3">
        <h2 className="text-sm font-medium text-ink">Offers from your agent</h2>
        {!offers.length && <p className="text-sm text-ink-2">No offers right now. When your agent notices work you keep asking for, it offers to take it over, here.</p>}
        {offers.map((o) => (
          <EntryShell key={o.id} label="Offer" time={today} title={o.title}>
            <div className="flex flex-col gap-3 px-4 py-4">
              <p className="text-sm text-ink-2">{o.detail}</p>
              {o.status === "pending" && (
                <div className="flex flex-wrap gap-2">
                  <Button onClick={() => act(`/api/offers/${encodeURIComponent(o.id)}/accept`, "Accepted.")}>Accept</Button>
                  <Button variant="outline" onClick={() => act(`/api/offers/${encodeURIComponent(o.id)}/dismiss`, "Dismissed.")}>Dismiss</Button>
                </div>
              )}
            </div>
          </EntryShell>
        ))}
      </section>

      <section aria-label="What your agent learned" className="flex flex-col gap-2">
        <h2 className="text-sm font-medium text-ink">What your agent learned</h2>
        {learned.length ? <ul className="flex list-disc flex-col gap-1.5 pl-5 text-ui text-ink">{learned.map((l, i) => <li key={i}>{l}</li>)}</ul>
          : <p className="text-sm text-ink-2">Nothing learned recently. When your agent picks up a preference or a fact, it shows here.</p>}
      </section>

      <section aria-label="Today so far" className="flex flex-col gap-3">
        <h2 className="text-sm font-medium text-ink">Today so far</h2>
        {!events.length && <p className="text-sm text-ink-2">Nothing yet today.</p>}
        {events.map((e, i) => {
          const failed = e.kind === "work" && FAILURE_RE.test(String(e.detail || ""));
          const when = e.ts ? new Date(e.ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : "";
          return (
            <div key={i} className="grid grid-cols-[64px_1fr] gap-3 rounded-card border border-hairline bg-surface px-3 py-2.5">
              <span className="font-mono text-xs tabular-nums text-ink-3">{when}</span>
              <div className="min-w-0">
                <p className="text-sm font-medium text-ink">{e.title}</p>
                <p className="text-xs text-ink-3">{failed ? "did not finish" : String(e.kind || "").replace("_", " ")}</p>
                {failed ? <div className="mt-2"><p className="text-sm text-ink-2">The agent did not finish this. Try again, or ask whoever runs Ergon to check the connection.</p><div className="mt-2"><Details label="Details"><pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-3">{e.detail}</pre></Details></div></div>
                  : e.detail ? <p className="mt-1 text-sm text-ink-2">{e.detail}</p> : null}
              </div>
            </div>
          );
        })}
      </section>
      <span className="hidden"><FinishedPill /></span>
    </section>
  );
}
