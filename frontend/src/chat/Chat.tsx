// Chat face: active Field-Book conversation thread, live BorderTrail + TextShimmer step ledger,
// rich role-tailored Starter Task Cards (with category tags & Rune skill pills), and the pinned task composer.
import { useEffect, useRef } from "react";
import { ArrowRight, Send, Sparkles } from "lucide-react";
import { BorderTrail } from "@/components/ui/border-trail";
import { TextShimmer } from "@/components/ui/text-shimmer";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { InputGroup, InputGroupAddon, InputGroupTextarea } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Pill } from "@/components/kibo-ui/pill";
import { Failure, FileGlyph, FinishedEntry, FinishedPill, type Item, LedgerRow, Reply, UserLine, WaitingEntry, buildItems } from "./Entry";
import type { Msg } from "@/lib/api";
import { normalizeFiles, relTime, type StarterCard, threadMs } from "@/lib/words";
import { cn } from "@/lib/utils";

export function greeting(name: string, hour: number): string {
  const part = hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";
  return name ? `${part}, ${name}.` : `${part}.`;
}

export type Recent = { task: string; date: string; files?: string[] };

const TAG_TONE: Record<StarterCard["tone"], string> = {
  ops: "bg-tag-ops-bg text-tag-ops-fg border-transparent",
  mint: "bg-tag-mint-bg text-tag-mint-fg border-transparent",
  plan: "bg-tag-plan-bg text-tag-plan-fg border-transparent",
  research: "bg-tag-research-bg text-tag-research-fg border-transparent",
};

export function Thread({ messages, busy, threadId, onReview, onStarter, starters, status, pendingNote, name, recent }: {
  messages: Msg[];
  busy: boolean;
  threadId: string | null;
  onReview: () => void;
  onStarter: (text: string) => void;
  starters: StarterCard[];
  status: "ok" | "wait";
  pendingNote: boolean;
  name: string;
  recent: Recent[];
}) {
  const items = buildItems(messages);
  const ms = threadMs(threadId ?? "");
  const time = ms ? relTime(ms) : "";
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (items.length || busy) endRef.current?.scrollIntoView({ block: "end" });
  }, [items.length, busy]);

  if (!items.length && !busy) {
    return <Empty status={status} starters={starters} onStarter={onStarter} name={name} recent={recent} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[780px] flex-col gap-5">
      {items.map((it, i) => <Row key={i} it={it} time={time} onReview={onReview} />)}
      {busy && (
        <div role="status" className="relative overflow-hidden rounded-card border border-hairline bg-surface px-4 py-3.5">
          <BorderTrail size={56} />
          <div className="flex items-center gap-2.5 text-sm font-medium text-ink">
            <span aria-hidden className="size-2 rounded-full bg-blue motion-safe:animate-pulse" />
            <TextShimmer as="span">Agent is working on it</TextShimmer>
          </div>
          <p className="mt-1 text-xs text-ink-2">Running sandboxed workspace tools and verifying written files…</p>
        </div>
      )}
      {pendingNote && (
        <p id="pending-note" role="status" className="rounded-card border border-hairline bg-board px-3.5 py-2.5 text-sm text-ink-2">
          Your last message did not go through. It is back in the box below.
        </p>
      )}
      <div ref={endRef} />
    </div>
  );
}

function Row({ it, time, onReview }: { it: Item; time: string; onReview: () => void }) {
  switch (it.k) {
    case "user": return <UserLine text={it.text} />;
    case "step": return <ol className="rounded-card border border-hairline bg-surface px-3.5"><LedgerRow raw={it.text} /></ol>;
    case "finished": return <FinishedEntry time={time} task={it.task} steps={it.steps} result={it.text} notes={it.notes} />;
    case "waiting": return <WaitingEntry time={time} name={it.name} steps={it.steps} result={it.text} onReview={onReview} />;
    case "error": return <Failure raw={it.text} />;
    default: return <Reply text={it.text} />;
  }
}

/** Static topographic survey contour field drawn from --hairline (zero gradient, zero motion). */
function contours(): string[] {
  const out: string[] = [];
  for (let i = 1; i <= 8; i++) {
    const R = 44 + i * 36;
    let d = "";
    for (let k = 0; k <= 72; k++) {
      const t = (k / 72) * 2 * Math.PI;
      const r = R * (1 + 0.08 * Math.sin(3 * t + i * 0.7) + 0.04 * Math.cos(5 * t - i));
      d += (k ? "L" : "M") + (500 + r * Math.cos(t) * 1.65).toFixed(1) + " " + (250 + r * Math.sin(t) * 0.72).toFixed(1);
    }
    out.push(d + "Z");
  }
  return out;
}
const CONTOURS = contours();

export function ContourField() {
  return (
    <svg aria-hidden viewBox="0 0 1000 560" preserveAspectRatio="xMidYMid slice" className="contour-field pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-[0.3]">
      {CONTOURS.map((d, i) => <path key={i} d={d} fill="none" stroke="currentColor" strokeWidth={i % 4 === 0 ? 1.3 : 0.85} />)}
    </svg>
  );
}

function Empty({ status, starters, onStarter, name, recent }: {
  status: "ok" | "wait";
  starters: StarterCard[];
  onStarter: (t: string) => void;
  name: string;
  recent: Recent[];
}) {
  const sub = status === "ok"
    ? "State the outcome in plain words. Your agent runs real workspace tools and files a verified Field-Book receipt here."
    : "Ergon is not connected to a model yet. Anything you type is kept safely in this conversation.";

  const pickStarter = (prompt: string) => {
    onStarter(prompt);
    setTimeout(() => document.getElementById("composer-input")?.focus(), 0);
  };

  return (
    <section aria-label="Start a task" className="mx-auto flex w-full max-w-[780px] flex-col gap-6 pt-2">
      <div className="flex flex-col gap-2">
        <h2 className="text-[32px] leading-[1.1] font-medium tracking-[-0.03em] text-ink md:text-[38px]">
          {greeting(name, new Date().getHours())}
        </h2>
        <p className="max-w-[62ch] text-ui leading-6 text-ink-2">
          What should your agent finish for you? {sub}
        </p>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {starters.map((card) => (
          <Button
            key={card.prompt}
            variant="ghost"
            onClick={() => pickStarter(card.prompt)}
            className="mould h-auto sm:h-auto min-h-34 w-full flex-col items-stretch justify-between gap-3 whitespace-normal rounded-card border border-hairline bg-surface p-4 text-left font-normal hover:bg-board"
          >
            <span className="flex w-full items-center justify-between gap-2">
              <Badge size="sm" className={cn("px-2 py-0.5 text-xs font-medium", TAG_TONE[card.tone])}>
                {card.category}
              </Badge>
              <Pill variant="secondary" className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2">
                <Sparkles className="size-3 text-icon" strokeWidth={1.5} aria-hidden />
                <span className="mono">{card.skill}</span>
              </Pill>
            </span>
            <span className="flex flex-col gap-1">
              <span className="text-ui font-medium text-ink">{card.prompt}</span>
              <span className="text-sm leading-5 text-ink-2">{card.detail}</span>
            </span>
            <span className="flex w-full items-center justify-between border-t border-hairline pt-2.5 text-xs font-medium text-ink-2">
              <span>Fill task box</span>
              <ArrowRight className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            </span>
          </Button>
        ))}
      </div>

      {recent.length > 0 && (
        <div className="flex flex-col gap-2.5 pt-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-ink">Finished recently</h3>
            <span className="mono text-xs text-ink-3">{recent.length} {recent.length === 1 ? "entry" : "entries"}</span>
          </div>
          <div className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
            {recent.slice(0, 3).map((r, i) => {
              const files = normalizeFiles(r.files);
              return (
                <div key={i} className="flex flex-wrap items-center justify-between gap-3 px-4 py-3">
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-ink">{r.task}</p>
                    <p className="mt-0.5 flex flex-wrap items-center gap-2 text-xs text-ink-2">
                      <span className="mono">{r.date}</span>
                      {files.map((f, idx) => (
                        <span key={f + idx} className="mono inline-flex items-center gap-1 text-ink">
                          <FileGlyph name={f} className="size-3.5" />
                          <span className="truncate">{f}</span>
                        </span>
                      ))}
                    </p>
                  </div>
                  <FinishedPill />
                </div>
              );
            })}
          </div>
        </div>
      )}
    </section>
  );
}

export function Composer({ value, onChange, onSend, busy, disabled }: {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  busy: boolean;
  disabled: boolean;
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
  const canSend = !disabled && !!value.trim();

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  }, [value]);

  return (
    <form id="composer" aria-label="Send a task" className="mx-auto w-full max-w-[780px]" onSubmit={(e) => { e.preventDefault(); onSend(); }}>
      <InputGroup className="min-h-14 items-end rounded-card border-input bg-surface shadow-raised">
        <label htmlFor="composer-input" className="sr-only">Describe a task for your agent</label>
        <InputGroupTextarea
          id="composer-input"
          ref={ref}
          rows={1}
          value={value}
          disabled={disabled}
          placeholder="Describe a task in plain words"
          className="max-h-[200px] min-h-14 py-3.5 text-ui"
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              onSend();
            }
          }}
        />
        <InputGroupAddon align="inline-end" className="pb-2 pr-2">
          <Button
            id="send-btn"
            type="submit"
            size="lg"
            variant={canSend ? "default" : "outline"}
            disabled={!canSend}
            className={cn("min-h-11", canSend ? "shadow-primary" : "shadow-none")}
          >
            <Send className="size-4" strokeWidth={1.5} aria-hidden />
            {busy ? "Working…" : "Send"}
          </Button>
        </InputGroupAddon>
      </InputGroup>
      <p className="mt-2 flex flex-wrap items-center gap-1.5 text-xs text-ink-2">
        <span>Press <Kbd>Enter</Kbd> to send.</span>
        <span className="max-sm:hidden"><Kbd>Shift</Kbd> <Kbd>Enter</Kbd> starts a new line.</span>
        <span className="ml-auto max-sm:hidden"><Kbd>⌘K</Kbd> focuses from anywhere</span>
      </p>
    </form>
  );
}
