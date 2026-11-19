// The signature object: a finished task as a field-book entry, and the step ledger rows it folds from.
import { Check, Clock } from "lucide-react";
import { Pill, PillStatus } from "@/components/kibo-ui/pill";
import { Button } from "@/components/ui/button";
import { Expandable, ExpandableContent, ExpandableTrigger } from "@/components/ui/expandable";
import { cn } from "@/lib/utils";
import { toolLabel, savedFile } from "@/lib/words";
import type { Msg } from "@/lib/api";

export type Step = { raw: string };

/** One step of the work: a plain label, and the raw step text behind Details. */
export function LedgerRow({ raw }: Step) {
  const nl = raw.indexOf("\n");
  const head = nl < 0 ? raw : raw.slice(0, nl);
  const body = nl < 0 ? "" : raw.slice(nl + 1);
  const m = /^Tool: (\S+)(?: \(via ([^)]+)\))?$/.exec(head);
  const label = (m && toolLabel(m[1], m[2], body)) || head;
  const file = savedFile(m ? m[1] : "", body);
  return (
    <li className="flex flex-col gap-1 py-2">
      <div className="flex items-start gap-2 text-sm text-ink">
        <Check className="mt-0.5 size-4 shrink-0 text-ink-2" strokeWidth={1.75} aria-hidden />
        <span>{label}</span>
      </div>
      <Details>
        {file && <p className="text-xs text-ink-2">File: <span className="font-mono">{file}</span>. Files are in your Ergon workspace folder.</p>}
        <pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-2">{raw}</pre>
      </Details>
    </li>
  );
}

/** A quiet disclosure: closed by default, pushes its neighbours when open. */
export function Details({ children }: { children: React.ReactNode }) {
  return (
    <Expandable expandDirection="vertical" expandBehavior="push" className="w-full">
      {({ isExpanded }) => (
        <div className="w-full">
          <ExpandableTrigger aria-label={isExpanded ? "Hide details" : "Details"} className="rounded-sm text-xs font-medium text-ink-2 underline-offset-2 outline-none hover:text-ink hover:underline focus-visible:ring-2 focus-visible:ring-blue">
            {isExpanded ? "Hide details" : "Details"}
          </ExpandableTrigger>
          <ExpandableContent preset="fade">
            <div className="mt-2 flex flex-col gap-2 border-l-2 border-hairline pl-3">{children}</div>
          </ExpandableContent>
        </div>
      )}
    </Expandable>
  );
}

function Cell({ label, children, className }: { label: string; children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("min-w-0 bg-surface px-4 py-3", className)}>
      <p className="text-xs text-ink-2">{label}</p>
      <div className="mt-1 min-w-0 break-words text-sm text-ink">{children}</div>
    </div>
  );
}

/** A finished task: ink head, ruled cells, the result, and the steps behind Details. */
export function FinishedEntry({ time, task, saved, steps, result, notes }: {
  time: string; task: string; saved: string; steps: string[]; result: string; notes: string[];
}) {
  return (
    <article aria-label="Finished work" className="overflow-hidden rounded-entry border border-ink bg-surface shadow-lift">
      <header className="flex items-center justify-between gap-3 bg-ink px-4 py-2.5 text-canvas">
        <span className="text-sm font-medium">Finished</span>
        <span className="font-mono text-xs tabular-nums opacity-80">{time}</span>
      </header>
      <div className="grid grid-cols-1 gap-px bg-ink sm:grid-cols-2">
        <Cell label="Task">{task || "Your request"}</Cell>
        <Cell label="Saved to">{saved ? <span className="font-mono text-sm">{saved}</span> : <span className="text-ink-2">Nothing saved to a file</span>}</Cell>
        <Cell label="Took">{steps.length ? (steps.length === 1 ? "1 step" : steps.length + " steps") : "No steps"}</Cell>
        <Cell label="Status">
          <PillStatus className="border-0 p-0">
            <Pill className="bg-pill-active-bg text-pill-active-fg">
              <Check className="size-3.5" strokeWidth={2} aria-hidden />Finished
            </Pill>
          </PillStatus>
        </Cell>
      </div>
      <div className="px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui text-ink">{result}</p>
        {notes.map((n) => (
          <div key={n} className="mt-3 rounded-card border border-hairline bg-board px-3 py-2 text-sm text-ink-2">
            <span className="font-medium text-ink">Check this. </span>{n.replace(/^Note:\s*/, "")}
          </div>
        ))}
        {steps.length > 0 && (
          <div className="mt-4 border-t border-hairline pt-3">
            <Details>
              <ol className="divide-y divide-hairline">{steps.map((s, i) => <LedgerRow key={i} raw={s} />)}</ol>
            </Details>
          </div>
        )}
      </div>
    </article>
  );
}

/** A task waiting on the person: the same card, a tangerine head, and the one decision as keys. */
export function WaitingEntry({ time, name, result, onReview }: { time: string; name: string; result: string; onReview: () => void }) {
  return (
    <article aria-label="Waiting on you" className="overflow-hidden rounded-entry border border-ink bg-surface shadow-lift">
      <header className="flex items-center justify-between gap-3 bg-blue px-4 py-2.5 text-blue-fg">
        <span className="flex items-center gap-2 text-sm font-medium"><Clock className="size-4" strokeWidth={1.75} aria-hidden />Waiting on you</span>
        <span className="font-mono text-xs tabular-nums opacity-80">{time}</span>
      </header>
      <div className="px-4 py-4">
        <p className="text-sm font-medium text-ink">Waiting for your approval to delete {name || "a file"}</p>
        <p className="mt-2 whitespace-pre-wrap break-words text-ui text-ink">{result}</p>
        <Button className="mt-4" variant="outline" onClick={onReview}>Review in Today</Button>
      </div>
    </article>
  );
}

/** A reply with no steps behind it: plain text, no finished card. */
export function Reply({ text }: { text: string }) {
  return <p className="whitespace-pre-wrap break-words text-ui text-ink">{text}</p>;
}

/** A failed run: one plain sentence; the server's words behind Details. */
export function Failure({ raw }: { raw: string }) {
  return (
    <div role="alert" className="rounded-card border border-hairline bg-surface px-4 py-3">
      <p className="text-sm font-medium text-ink">The agent could not finish this.</p>
      <p className="mt-1 text-sm text-ink-2">Try again, or ask whoever runs Ergon to check the connection.</p>
      <div className="mt-2"><Details><pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-2">{raw}</pre></Details></div>
    </div>
  );
}

export function UserLine({ text }: { text: string }) {
  return (
    <div className="flex justify-end">
      <p className="max-w-[80%] whitespace-pre-wrap break-words rounded-card bg-track px-3.5 py-2.5 text-ui text-ink">{text}</p>
    </div>
  );
}

export type Item =
  | { k: "user"; text: string }
  | { k: "step"; text: string }
  | { k: "finished"; text: string; task: string; steps: string[]; notes: string[] }
  | { k: "waiting"; text: string; name: string; task: string; steps: string[]; notes: string[] }
  | { k: "reply"; text: string }
  | { k: "error"; text: string };

/** The saved run as items, in order. Finished only after a real step; notes join their card. Port of the old renderRun. */
export function buildItems(messages: Msg[]): Item[] {
  const PARKED = /parked for your person|has NOT been deleted yet/;
  const RESOLVED = /was deleted after you approved it|was kept because you declined/;
  const out: Item[] = [];
  let seg: Msg[] = []; let task = "";
  const flush = () => {
    const hasStep = seg.some((m) => m.role === "step" && !/^Note:/.test(m.content));
    const parked = seg.some((m) => m.role === "step" && PARKED.test(m.content));
    const resolved = seg.some((m) => m.role === "step" && RESOLVED.test(m.content));
    const waiting = parked && !resolved;
    let name = "";
    for (const m of seg) {
      const hit = m.role === "step" && /(\S+) has NOT been deleted yet/.exec(m.content);
      if (hit) { name = hit[1]; break; }
    }
    // Steps wait here until the answer arrives; the answer folds them into its card's Details.
    let pending: string[] = []; let card: Item | null = null;
    for (const m of seg) {
      if (m.role === "step" && /^Note:/.test(m.content) && card) {
        // The parked-deletion note is the headline of its card, not a second box.
        if (!(waiting && /has NOT been deleted yet/.test(m.content)) && (card.k === "finished" || card.k === "waiting")) card.notes.push(m.content);
      } else if (m.role === "step") {
        pending.push(m.content); card = null;
      } else if (m.role === "assistant") {
        const steps = pending; pending = [];
        card = waiting ? { k: "waiting", text: m.content, name, task, steps, notes: [] }
          : hasStep ? { k: "finished", text: m.content, task, steps, notes: [] } : { k: "reply", text: m.content };
        out.push(card);
      } else {
        card = null;
        out.push({ k: m.role === "error" ? "error" : "reply", text: m.content });
      }
    }
    for (const s of pending) out.push({ k: "step", text: s });
    seg = [];
  };
  for (const m of messages) {
    if (m.role === "user") { flush(); task = m.content; out.push({ k: "user", text: m.content }); }
    else seg.push(m);
  }
  flush();
  return out;
}
