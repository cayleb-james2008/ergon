// The chat face: the thread (the run as items), the empty state with suggestion keys, and the composer.
import { useEffect, useRef } from "react";
import { ArrowRight, Send } from "lucide-react";
import { TextShimmer } from "@/components/ui/text-shimmer";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupTextarea } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Failure, FinishedEntry, Item, Reply, UserLine, WaitingEntry, LedgerRow, buildItems } from "./Entry";
import type { Msg } from "@/lib/api";
import { relTime, savedFile, threadMs } from "@/lib/words";

export function Thread({ messages, busy, threadId, onReview, onStarter, starters, status, pendingNote }: {
  messages: Msg[]; busy: boolean; threadId: string | null; onReview: () => void;
  onStarter: (text: string) => void; starters: string[]; status: "ok" | "wait" | "none";
  pendingNote: boolean;
}) {
  const items = buildItems(messages);
  const ms = threadMs(threadId ?? "");
  const time = ms ? relTime(ms) : "";
  const endRef = useRef<HTMLDivElement>(null);
  useEffect(() => { endRef.current?.scrollIntoView({ block: "end" }); }, [messages.length, busy]);
  if (!items.length && !busy) return <Empty status={status} starters={starters} onStarter={onStarter} />;
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
    case "step": return <ul className="rounded-card border border-hairline bg-surface px-3"><LedgerRow raw={it.text} /></ul>;
    case "finished": return <FinishedEntry time={time} task={it.task} saved={savedOf(it.steps)} steps={it.steps} result={it.text} notes={it.notes} />;
    case "waiting": return <WaitingEntry time={time} name={it.name} result={it.text} onReview={onReview} />;
    case "error": return <Failure raw={it.text} />;
    default: return <Reply text={it.text} />;
  }
}

/** The file the run saved, from its own save step. */
function savedOf(steps: string[]): string {
  for (const s of steps) {
    const nl = s.indexOf("\n");
    const head = nl < 0 ? s : s.slice(0, nl);
    const m = /^Tool: (\S+)/.exec(head);
    const f = m ? savedFile(m[1], nl < 0 ? "" : s.slice(nl + 1)) : "";
    if (f) return f;
  }
  return "";
}

function Empty({ status, starters, onStarter }: { status: "ok" | "wait" | "none"; starters: string[]; onStarter: (t: string) => void }) {
  const sub = status !== "wait"
    ? "It works step by step, right here, until the job is done."
    : "Ergon is not connected to a model yet (see the note above). Anything you type is kept in the thread.";
  return (
    <section aria-label="Start a task" className="mx-auto flex max-w-[640px] flex-col gap-6 pt-[8vh]">
      <div>
        <h2 className="text-[30px] leading-tight text-ink">What should your agent finish for you?</h2>
        <p className="mt-2 text-ui text-ink-2">{sub}</p>
      </div>
      <div className="flex flex-col gap-2">
        {starters.map((text) => (
          <button key={text} type="button" onClick={() => onStarter(text)} title={text}
            className="flex min-h-12 items-center justify-between gap-3 rounded-[6px] border border-hairline bg-surface px-4 text-left text-ui font-medium text-ink shadow-raised outline-none transition-[transform,background-color] duration-(--dur-press) ease-(--ease-house) hover:bg-board focus-visible:ring-2 focus-visible:ring-blue motion-safe:active:translate-y-px">
            {text}
            <ArrowRight className="size-4 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
          </button>
        ))}
      </div>
      <p className="text-sm text-ink-2">Tap one to fill the box, then press Send.</p>
    </section>
  );
}

export function Composer({ value, onChange, onSend, busy, disabled }: {
  value: string; onChange: (v: string) => void; onSend: () => void; busy: boolean; disabled: boolean;
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    const el = ref.current; if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  }, [value]);
  return (
    <form aria-label="Send a task" className="mx-auto w-full max-w-[760px]" onSubmit={(e) => { e.preventDefault(); onSend(); }}>
      <InputGroup className="items-end rounded-card border-hairline bg-surface shadow-raised">
        <label htmlFor="composer-input" className="sr-only">Describe a task for your agent</label>
        <InputGroupTextarea
          id="composer-input" ref={ref} rows={1} value={value} disabled={disabled}
          placeholder="Describe a task in plain words" className="min-h-11 max-h-[200px] text-ui"
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) { e.preventDefault(); onSend(); } }}
        />
        <InputGroupAddon align="inline-end" className="pb-1.5 pr-1.5">
          <Button type="submit" disabled={disabled || !value.trim()} aria-label={busy ? "Working" : "Send"} className="shadow-primary">
            <Send className="size-4" strokeWidth={1.75} aria-hidden />{busy ? "Working…" : "Send"}
          </Button>
        </InputGroupAddon>
      </InputGroup>
      <p className="mt-2 flex items-center gap-1.5 text-xs text-ink-2"><Kbd>Enter</Kbd> sends. <Kbd>Shift</Kbd>+<Kbd>Enter</Kbd> starts a new line.</p>
    </form>
  );
}
