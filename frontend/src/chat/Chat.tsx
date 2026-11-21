// The chat face: the thread, the empty state (greeting, moulded keys, contour texture, recent field-book entries), the composer.
import { useEffect, useRef } from "react";
import { ArrowRight, BookOpen, ListChecks, Search, Send, Bot, FileText } from "lucide-react";
import { TextShimmer } from "@/components/ui/text-shimmer";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupTextarea } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Cell, Cells, EntryShell, FinishedPill, Failure, FinishedEntry, Item, LedgerRow, Reply, UserLine, WaitingEntry, buildItems } from "./Entry";
import type { Msg } from "@/lib/api";
import { relTime, threadMs } from "@/lib/words";
import { cn } from "@/lib/utils";

export function greeting(name: string, hour: number): string {
  const part = hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";
  return name ? `${part}, ${name}.` : `${part}.`;
}

export type Recent = { task: string; date: string };

export function Thread({ messages, busy, threadId, onReview, onStarter, starters, status, pendingNote, name, recent }: {
  messages: Msg[]; busy: boolean; threadId: string | null; onReview: () => void;
  onStarter: (text: string) => void; starters: string[]; status: "ok" | "wait";
  pendingNote: boolean; name: string; recent: Recent[];
}) {
  const items = buildItems(messages);
  const ms = threadMs(threadId ?? "");
  const time = ms ? relTime(ms) : "";
  const endRef = useRef<HTMLDivElement>(null);
  useEffect(() => { endRef.current?.scrollIntoView({ block: "end" }); }, [messages.length, busy]);
  if (!items.length && !busy) return <Empty status={status} starters={starters} onStarter={onStarter} name={name} recent={recent} />;
  return (
    <div className="flex flex-col gap-5">
      {items.map((it, i) => <Row key={i} it={it} time={time} onReview={onReview} />)}
      {busy && (
        <p role="status" className="flex items-center gap-2 text-sm text-ink-2">
          <span aria-hidden className="size-2 rounded-full bg-blue motion-safe:animate-pulse" />
          <TextShimmer as="span">Agent is working on it</TextShimmer>
        </p>
      )}
      {pendingNote && <p role="status" className="text-sm text-ink-2">Your last message did not go through. It is back in the box.</p>}
      <div ref={endRef} />
    </div>
  );
}

function Row({ it, time, onReview }: { it: Item; time: string; onReview: () => void }) {
  switch (it.k) {
    case "user": return <UserLine text={it.text} />;
    case "step": return <ol className="rounded-card border border-hairline bg-surface px-3"><LedgerRow raw={it.text} /></ol>;
    case "finished": return <FinishedEntry time={time} task={it.task} steps={it.steps} result={it.text} notes={it.notes} />;
    case "waiting": return <WaitingEntry time={time} name={it.name} result={it.text} onReview={onReview} />;
    case "error": return <Failure raw={it.text} />;
    default: return <Reply text={it.text} />;
  }
}

const ICONS = [FileText, Bot, ListChecks, BookOpen];

/** Contour lines of a survey sheet, drawn once from fixed numbers (no randomness). */
function contours(): string[] {
  const out: string[] = [];
  for (let i = 1; i <= 9; i++) {
    const R = 40 + i * 34;
    let d = "";
    for (let k = 0; k <= 72; k++) {
      const t = (k / 72) * 2 * Math.PI;
      const r = R * (1 + 0.09 * Math.sin(3 * t + i * 0.7) + 0.05 * Math.cos(5 * t - i));
      d += (k ? "L" : "M") + (500 + r * Math.cos(t) * 1.6).toFixed(1) + " " + (240 + r * Math.sin(t) * 0.7).toFixed(1);
    }
    out.push(d + "Z");
  }
  return out;
}
const CONTOURS = contours();

/** Full-bleed survey contours behind a region: static, drawn from --hairline, no gradient, parent must be relative. */
export function ContourField() {
  return (
    <svg aria-hidden viewBox="0 0 1000 560" preserveAspectRatio="xMidYMid slice" className="contour-field pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-[0.35]">
      {CONTOURS.map((d, i) => <path key={i} d={d} fill="none" stroke="currentColor" strokeWidth={i % 4 === 0 ? 1.4 : 0.9} />)}
    </svg>
  );
}

function Empty({ status, starters, onStarter, name, recent }: {
  status: "ok" | "wait"; starters: string[]; onStarter: (t: string) => void; name: string; recent: Recent[];
}) {
  const sub = status === "ok"
    ? "It works step by step, right here, until the job is done."
    : "Ergon is not connected to a model yet. Anything you type is kept in the thread.";
  return (
    <section aria-label="Start a task" className="mx-auto flex w-full max-w-[680px] flex-col gap-8 pt-4">
      <div className="relative pb-2">
        <h2 className="text-[40px] leading-[1.05] font-semibold tracking-[-0.035em] text-ink">{greeting(name, new Date().getHours())}</h2>
        <p className="mt-3 text-ui text-ink-2">What should your agent finish for you? {sub}</p>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {starters.map((text, i) => {
          const Icon = ICONS[i % ICONS.length];
          return (
            <button key={text} type="button" onClick={() => onStarter(text)} title={text}
              className="mould focus-ring flex min-h-14 items-center gap-3 rounded-[6px] bg-surface px-3.5 text-left text-ui font-medium text-ink outline-none">
              <span className="grid size-8 shrink-0 place-items-center rounded-sm bg-track text-ink-2"><Icon className="size-4" strokeWidth={1.75} aria-hidden /></span>
              <span className="min-w-0 flex-1">{text}</span>
              <ArrowRight className="size-4 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
            </button>
          );
        })}
      </div>
      <p className="text-sm text-ink-2">Tap one to fill the box, then press Send.</p>
      {recent.length > 0 && (
        <div className="flex flex-col gap-3">
          <p className="text-sm font-medium text-ink">Finished recently</p>
          {recent.slice(0, 3).map((r, i) => (
            <EntryShell key={i} label="Finished work" time={r.date} title={r.task} status={<FinishedPill />}>
              <Cells><Cell label="Status">Finished</Cell><Cell label="Date">{r.date}</Cell></Cells>
            </EntryShell>
          ))}
        </div>
      )}
    </section>
  );
}

export function Composer({ value, onChange, onSend, busy, disabled }: {
  value: string; onChange: (v: string) => void; onSend: () => void; busy: boolean; disabled: boolean;
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
  const canSend = !disabled && !!value.trim();
  useEffect(() => {
    const el = ref.current; if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  }, [value]);
  return (
    <form aria-label="Send a task" className="mx-auto w-full max-w-[760px]" onSubmit={(e) => { e.preventDefault(); onSend(); }}>
      <InputGroup className="min-h-14 items-end rounded-card border-ink-3 bg-surface shadow-raised">
        <label htmlFor="composer-input" className="sr-only">Describe a task for your agent</label>
        <InputGroupTextarea
          id="composer-input" ref={ref} rows={1} value={value} disabled={disabled}
          placeholder="Describe a task in plain words" className="min-h-14 max-h-[200px] py-4 text-ui"
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) { e.preventDefault(); onSend(); } }}
        />
        <InputGroupAddon align="inline-end" className="pb-2 pr-2">
          <Button type="submit" size="lg" variant={canSend ? "default" : "outline"} disabled={!canSend}
            className={cn(canSend ? "shadow-primary" : "shadow-none disabled:bg-transparent disabled:text-ink-3 disabled:border-ink-3")}>
            <Send className="size-4" strokeWidth={1.75} aria-hidden />{busy ? "Working…" : "Send"}
          </Button>
        </InputGroupAddon>
      </InputGroup>
      <p className="mt-2 text-xs text-ink-2">Press <Kbd>Enter</Kbd> to send. <Kbd>Shift</Kbd> <Kbd>Enter</Kbd> starts a new line.</p>
    </form>
  );
}
