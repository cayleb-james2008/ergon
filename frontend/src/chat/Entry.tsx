// The signature object: the field-book entry. A head strip (time, title, status), a ruled grid, a file row,
// the result, and the steps behind a Cult UI Expandable key. Also the step ledger and the run builder.
import { Check, ChevronDown, FileText, Hourglass } from "lucide-react";
import { Expandable, ExpandableContent, ExpandableTrigger } from "@/components/ui/expandable";
import { Pill } from "@/components/kibo-ui/pill";
import { cn } from "@/lib/utils";
import { savedFile, toolLabel } from "@/lib/words";
import type { Msg } from "@/lib/api";

/** A disclosure key: a moulded row with a chevron. Closed by default; it pushes what sits below it. */
export function Details({ label = "Details", children }: { label?: string; children: React.ReactNode }) {
  return (
    <Expandable expandDirection="vertical" expandBehavior="push" className="w-full">
      {({ isExpanded }) => (
        <div className="w-full">
          <ExpandableTrigger aria-label={label} aria-expanded={isExpanded}
            className="mould focus-ring flex min-h-10 w-full items-center justify-between gap-2 rounded-[6px] border border-hairline bg-surface px-3 text-left text-sm font-medium text-ink outline-none">
            {label}
            <ChevronDown className={cn("size-4 shrink-0 text-icon motion-safe:transition-transform motion-safe:duration-(--dur-swap) motion-safe:ease-(--ease-house)", isExpanded && "rotate-180")} strokeWidth={1.75} aria-hidden />
          </ExpandableTrigger>
          <ExpandableContent preset="fade">
            <div className="mt-2 flex flex-col gap-3">{children}</div>
          </ExpandableContent>
        </div>
      )}
    </Expandable>
  );
}

/** The entry frame: an ink head (time in mono, the title, a status) and the body under it. */
export function EntryShell({ time, title, status, label, children }: {
  time: string; title: string; status?: React.ReactNode; label: string; children: React.ReactNode;
}) {
  return (
    <article aria-label={label} className="overflow-hidden rounded-entry border border-ink bg-surface shadow-lift">
      <header className="flex min-w-0 items-center gap-3 border-b border-ink bg-(--entry-head) px-4 py-2.5 text-(--entry-head-ink) dark:border-hairline">
        <span className="shrink-0 font-mono text-xs tabular-nums">{time}</span>
        <span className="min-w-0 flex-1 truncate text-sm font-medium">{title}</span>
        {status}
      </header>
      {children}
    </article>
  );
}

export function FinishedPill() {
  return <Pill className="status-finished border-0 px-2.5 py-1 text-xs font-medium"><Check className="size-3.5" strokeWidth={2} aria-hidden />Finished</Pill>;
}

export function WaitingPill() {
  return <Pill className="status-waiting border-0 bg-blue px-2.5 py-1 text-xs font-medium text-blue-fg"><Hourglass className="size-3.5" strokeWidth={2} aria-hidden />Waiting on you</Pill>;
}

/** The ruled grid: 1px ink lines between every cell, all four sides. */
export function Cells({ children }: { children: React.ReactNode }) {
  return <div className="grid grid-cols-2 gap-px border-b border-ink bg-ink">{children}</div>;
}

export function Cell({ label, children, span }: { label: string; children: React.ReactNode; span?: boolean }) {
  return (
    <div className={cn("min-w-0 bg-surface px-3 py-2.5", span && "col-span-2")}>
      <p className="text-xs text-ink-3">{label}</p>
      <div className="mt-0.5 min-w-0 break-words text-ui text-ink">{children}</div>
    </div>
  );
}

export function FileRow({ name }: { name: string }) {
  return (
    <div className="flex min-w-0 items-center gap-2.5 border-b border-hairline px-4 py-2.5">
      <FileText className="size-4 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
      <span className="min-w-0 truncate font-mono text-sm text-ink">{name}</span>
      <span className="ml-auto shrink-0 text-xs text-ink-3">in your Ergon workspace folder</span>
    </div>
  );
}

export function Note({ text }: { text: string }) {
  return (
    <div className="rounded-card border border-hairline bg-board px-3 py-2 text-sm text-ink-2">
      <span className="font-medium text-ink">Check this. </span>{text.replace(/^Note:\s*/, "")}
    </div>
  );
}

/** One step: the plain label, and the raw step text inside Details. */
export function LedgerRow({ raw }: { raw: string }) {
  const nl = raw.indexOf("\n");
  const head = nl < 0 ? raw : raw.slice(0, nl);
  const body = nl < 0 ? "" : raw.slice(nl + 1);
  const m = /^Tool: (\S+)(?: \(via ([^)]+)\))?$/.exec(head);
  const label = (m && toolLabel(m[1], m[2], body)) || head;
  return (
    <li className="flex flex-col gap-1 py-2">
      <span className="flex items-start gap-2 text-sm text-ink"><Check className="mt-0.5 size-4 shrink-0 text-ink-2" strokeWidth={1.75} aria-hidden />{label}</span>
      <pre className="whitespace-pre-wrap break-words pl-6 font-mono text-xs text-ink-3">{raw}</pre>
    </li>
  );
}

export function StepList({ steps }: { steps: string[] }) {
  return <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface px-3">{steps.map((s, i) => <LedgerRow key={i} raw={s} />)}</ol>;
}

/** A finished task as a field-book entry. */
export function FinishedEntry({ time, task, steps, result, notes }: {
  time: string; task: string; steps: string[]; result: string; notes: string[];
}) {
  const saved = savedOf(steps);
  return (
    <EntryShell label="Finished work" time={time} title={task || "Your request"} status={<FinishedPill />}>
      <Cells>
        <Cell label="Took">{steps.length === 1 ? "1 step" : steps.length ? steps.length + " steps" : "No steps"}</Cell>
        <Cell label="Saved">{saved ? "1 file" : "Nothing saved"}</Cell>
      </Cells>
      {saved && <FileRow name={saved} />}
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui text-ink">{result}</p>
        {notes.map((n) => <Note key={n} text={n} />)}
        {steps.length > 0 && <Details label="Details"><StepList steps={steps} /></Details>}
      </div>
    </EntryShell>
  );
}

/** A task waiting on the person: the same frame with the tangerine status and the one decision. */
export function WaitingEntry({ time, name, result, onReview }: { time: string; name: string; result: string; onReview: () => void }) {
  return (
    <EntryShell label="Waiting on you" time={time} title={"Delete " + (name || "a file") + "?"} status={<WaitingPill />}>
      <Cells>
        <Cell label="Waiting for">{name || "your approval"}</Cell>
        <Cell label="Decision">Approve or keep</Cell>
      </Cells>
      <div className="flex flex-col gap-3 px-4 py-4">
        <p className="whitespace-pre-wrap break-words text-ui text-ink">{result}</p>
        <div><button type="button" onClick={onReview} className="mould focus-ring min-h-10 rounded-[6px] border border-hairline bg-surface px-4 text-sm font-medium text-ink">Review in Today</button></div>
      </div>
    </EntryShell>
  );
}

/** A reply with no steps behind it. */
export function Reply({ text }: { text: string }) {
  return <p className="whitespace-pre-wrap break-words text-ui text-ink">{text}</p>;
}

/** A failed run: one plain sentence; the server's words behind Details. */
export function Failure({ raw }: { raw: string }) {
  return (
    <div role="alert" className="rounded-entry border border-hairline bg-surface px-4 py-3">
      <p className="text-sm font-medium text-ink">The agent could not finish this.</p>
      <p className="mt-1 text-sm text-ink-2">Try again, or ask whoever runs Ergon to check the connection.</p>
      <div className="mt-3"><Details label="Details"><pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-3">{raw}</pre></Details></div>
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

/** The file a save step wrote, read from its own step text. */
export function savedOf(steps: string[]): string {
  for (const s of steps) {
    const nl = s.indexOf("\n");
    const head = nl < 0 ? s : s.slice(0, nl);
    const m = /^Tool: (\S+)/.exec(head);
    const f = m ? savedFile(m[1], nl < 0 ? "" : s.slice(nl + 1)) : "";
    if (f) return f;
  }
  return "";
}

export type Item =
  | { k: "user"; text: string }
  | { k: "step"; text: string }
  | { k: "finished"; text: string; task: string; steps: string[]; notes: string[] }
  | { k: "waiting"; text: string; name: string; task: string; steps: string[]; notes: string[] }
  | { k: "reply"; text: string }
  | { k: "error"; text: string };

const PARKED = /parked for your person|has NOT been deleted yet/;
const RESOLVED = /was deleted after you approved it|was kept because you declined/;

/** A saved run as items, in order. Steps fold into the card that answers them. Port of the old renderRun. */
export function buildItems(messages: Msg[]): Item[] {
  const out: Item[] = [];
  let seg: Msg[] = []; let task = "";
  const flush = () => {
    const hasStep = seg.some((m) => m.role === "step" && !/^Note:/.test(m.content));
    const waiting = seg.some((m) => m.role === "step" && PARKED.test(m.content)) && !seg.some((m) => m.role === "step" && RESOLVED.test(m.content));
    let name = "";
    for (const m of seg) {
      const hit = m.role === "step" && /(\S+) has NOT been deleted yet/.exec(m.content);
      if (hit) { name = hit[1]; break; }
    }
    let pending: string[] = []; let card: Item | null = null;
    for (const m of seg) {
      if (m.role === "step" && /^Note:/.test(m.content) && card) {
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

/** A thread's state for the rail: waiting on the person, finished, or neither. */
export function threadState(messages: Msg[]): "waiting" | "finished" | "none" {
  if (messages.some((m) => m.role === "step" && PARKED.test(m.content) && !messages.some((x) => x.role === "step" && RESOLVED.test(x.content)))) return "waiting";
  return messages.some((m) => m.role === "assistant") ? "finished" : "none";
}
