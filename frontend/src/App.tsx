// Ergon's shell: rail (person, conversations grouped Today / Earlier, footer panels, version row), top bar
// (Chat / Today / Work Graph with count chips and a glide), update strip, and the face. Under 768px the rail is a Vaul drawer.
import { useEffect, useRef, useState } from "react";
import { Drawer } from "vaul";
import { Check, Hourglass, Menu, Moon, Plus, Sun } from "lucide-react";
import { Toaster } from "sonner";
import { AnimatedBackground } from "@/components/ui/animated-background";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { Composer, Thread } from "@/chat/Chat";
import { useErgon, type TState } from "./state";
import { useUpdate, UpdateStrip, VersionRow } from "@/shell/Update";
import { AddPersonPanel, LearnedPanel, SuitePanel } from "@/shell/Panels";
import { TodayFace } from "@/views/Today";
import { GraphFace } from "@/views/Graph";
import { cn } from "@/lib/utils";
import { relTime, threadMs } from "@/lib/words";

type Face = "chat" | "today" | "graph";
type Theme = "light" | "dark";
const THEME_KEY = "ergon-theme";
const RECOVERED_KEY = "ergon-recovered-dismissed";

function useTheme(): [Theme, (t: Theme) => void] {
  const [t, setT] = useState<Theme>(() => (document.documentElement.getAttribute("data-theme") === "dark" ? "dark" : "light"));
  const set = (next: Theme) => {
    setT(next);
    document.documentElement.setAttribute("data-theme", next);
    try { localStorage.setItem("ergon-theme", next); } catch { /* the choice still holds for this page */ }
  };
  return [t, set];
}

function useNarrow() {
  const q = "(max-width: 767px)";
  const [narrow, setNarrow] = useState(() => window.matchMedia(q).matches);
  useEffect(() => {
    const m = window.matchMedia(q);
    const on = () => setNarrow(m.matches);
    m.addEventListener("change", on);
    return () => m.removeEventListener("change", on);
  }, []);
  return narrow;
}

function sameDay(ms: number): boolean {
  const a = new Date(ms), b = new Date();
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

export function App() {
  const e = useErgon();
  const u = useUpdate();
  const [theme, setTheme] = useTheme();
  const narrow = useNarrow();
  const [face, setFace] = useState<Face>("chat");
  const [menuOpen, setMenuOpen] = useState(false);
  const [recDismissed, setRecDismissed] = useState(false);
  const menuRef = useRef<HTMLButtonElement>(null);
  const newChatRef = useRef<HTMLButtonElement>(null);
  const wasOpen = useRef(false);

  // Focus: opening the drawer moves focus to New chat; closing it returns focus to the Menu key.
  useEffect(() => {
    if (menuOpen) { wasOpen.current = true; setTimeout(() => newChatRef.current?.focus(), 0); }
    else if (wasOpen.current) { wasOpen.current = false; menuRef.current?.focus(); }
  }, [menuOpen]);

  const modelOff = e.status !== null && e.status !== "error" && !e.status.model_configured;
  const chip = e.status === null ? { cls: "", text: "Checking the agent" }
    : e.status === "error" ? { cls: "", text: "Could not check the agent" }
    : modelOff ? { cls: "wait", text: "Not connected to a model yet" }
    : e.runFailed ? { cls: "wait", text: "Last run did not finish" }
    : { cls: "ok", text: "Agent ready" };
  const recovered = e.recovered.length > 0 && !recDismissed && (() => { try { return localStorage.getItem(RECOVERED_KEY) !== e.recovered.join("|"); } catch { return true; } })();
  const dismissRecovered = () => { try { localStorage.setItem(RECOVERED_KEY, e.recovered.join("|")); } catch { /* */ } setRecDismissed(true); };
  const personName = e.status && e.status !== "error" ? e.status.person?.name ?? "" : "";
  const name = personName === "You" ? "" : personName;

  const threadRows = e.threads ?? [];
  const todays = threadRows.filter((t) => { const ms = threadMs(t.id); return ms !== null && sameDay(ms); });
  const earlier = threadRows.filter((t) => !todays.includes(t));

  const stateIcon = (s: TState | undefined) => s === "waiting"
    ? <Hourglass className="size-3.5 text-blue" strokeWidth={2} aria-label="Waiting on you" />
    : s === "finished" ? <Check className="size-3.5 text-dot-green" strokeWidth={2} aria-label="Finished" /> : null;

  const row = (t: { id: string; title?: string; messages?: number }) => {
    const ms = threadMs(t.id);
    const active = t.id === e.threadId;
    const s = e.states[t.id];
    return (
      <li key={t.id}>
        <button type="button" aria-current={active ? "true" : undefined} onClick={() => { setMenuOpen(false); setFace("chat"); e.openThread(t.id); }}
          className={cn("thread-btn focus-ring flex w-full min-w-0 flex-col items-start gap-0.5 rounded-sm py-2 pl-3 pr-2 text-left outline-none",
            active ? "bg-blue-soft shadow-[inset_2px_0_0_var(--blue)]" : "hover:bg-board")}>
          <span className="block w-full truncate text-sm font-medium text-ink">{t.title || "Untitled task"}</span>
          <span className="flex min-w-0 items-center gap-1.5 text-xs text-ink-2">
            {stateIcon(s)}
            <span className="truncate">{ms ? relTime(ms) : ""}{typeof t.messages === "number" ? (ms ? " · " : "") + (t.messages === 1 ? "1 message" : t.messages + " messages") : ""}</span>
          </span>
        </button>
      </li>
    );
  };

  const rail = (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center gap-2 px-4 pb-3 pt-4 text-title font-semibold text-ink">
        <svg viewBox="0 0 24 24" aria-hidden className="size-6 text-ink"><rect x="3" y="3" width="18" height="18" rx="5" fill="none" stroke="currentColor" strokeWidth="1.6" /><path d="M8 12.5l2.6 2.6L16 9.5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>
        Ergon
      </div>
      <div id="sidebar" className="flex min-h-0 flex-1 flex-col">
        <div className="px-4">
          <label id="person-label" htmlFor="person-select" className="text-xs text-ink-2">Working for</label>
          <select id="person-select" aria-labelledby="person-label" value={e.people?.active ?? ""} disabled={!e.people}
            onChange={(ev) => e.switchPerson(ev.target.value)}
            className="focus-ring mt-1 h-10 w-full rounded-sm border border-ink-3 bg-surface px-2.5 text-sm text-ink outline-none">
            {(e.people?.people ?? []).map((p) => <option key={p.id} value={p.id}>{p.role ? p.name + ", " + p.role : p.name}</option>)}
          </select>
        </div>
        <div className="px-4 pt-3">
          <Button ref={newChatRef} className="w-full shadow-primary" onClick={() => { setFace("chat"); setMenuOpen(false); e.newThread(); }}><Plus className="size-4" strokeWidth={1.75} aria-hidden />New chat</Button>
        </div>
        <div className="mt-5 flex min-h-0 flex-1 flex-col">
          <ul id="thread-list" aria-label="Conversations" className="flex min-h-0 flex-1 flex-col overflow-y-auto px-2 pb-2">
            {e.threadsError && <li className="px-2 py-2 text-sm text-ink-2">Could not load your conversations. Check that Ergon is running. <Button size="sm" variant="outline" className="mt-2" onClick={e.loadThreads}>Retry</Button></li>}
            {!e.threadsError && e.threads && !e.threads.length && <li className="px-2 py-2 text-sm text-ink-2">No conversations yet</li>}
            {todays.length > 0 && <li className="px-3 pb-1 pt-2 text-xs font-medium text-ink-2">Today</li>}
            {todays.map(row)}
            {earlier.length > 0 && <li className="px-3 pb-1 pt-3 text-xs font-medium text-ink-2">Earlier</li>}
            {earlier.map(row)}
          </ul>
        </div>
        {e.sideError && <p role="alert" className="px-4 pb-2 text-sm text-ink">{e.sideError}</p>}
        <div id="settings" className="flex flex-col gap-1 border-t border-hairline px-2 py-3">
          <div className="flex items-center justify-between gap-2 px-2 py-1">
            <span className="text-sm text-ink-2">Appearance</span>
            <SegmentedControl aria-label="Appearance" size="sm">
              {(["light", "dark"] as const).map((t) => (
                <SegmentedControlItem key={t} data-id={t} aria-pressed={theme === t} data-checked={theme === t ? "" : undefined} onClick={() => setTheme(t)}>
                  {t === "light" ? <Sun className="size-3.5" strokeWidth={1.75} aria-hidden /> : <Moon className="size-3.5" strokeWidth={1.75} aria-hidden />}
                  {t === "light" ? "Light" : "Dark"}
                </SegmentedControlItem>
              ))}
            </SegmentedControl>
          </div>
          <LearnedPanel />
          <SuitePanel onRun={(id, msgs) => { e.showRun(id, msgs); setFace("chat"); setMenuOpen(false); }} />
          <AddPersonPanel onAdded={() => { e.refreshPerson(); setMenuOpen(false); }} />
          <VersionRow u={u} />
        </div>
      </div>
    </div>
  );

  const tabs = [
    { id: "chat", label: "Chat", count: 0 },
    { id: "today", label: "Today", count: e.todayCount },
    { id: "graph", label: "Work Graph", count: 0 },
  ] as const;

  return (
    <div className="flex h-dvh min-h-0 overflow-hidden bg-canvas text-ink">
      {!narrow && <aside aria-label="Ergon navigation" className="flex w-[264px] shrink-0 flex-col border-r border-hairline bg-surface">{rail}</aside>}
      {narrow && (
        <Drawer.Root open={menuOpen} onOpenChange={setMenuOpen} direction="left">
          <Drawer.Portal>
            <Drawer.Overlay className="fixed inset-0 bg-ink/40" />
            <Drawer.Content aria-label="Ergon navigation" className="fixed inset-y-0 left-0 z-50 flex w-[300px] max-w-[86vw] flex-col border-r border-hairline bg-surface outline-none">
              <Drawer.Title className="sr-only">Ergon navigation</Drawer.Title>
              {rail}
            </Drawer.Content>
          </Drawer.Portal>
        </Drawer.Root>
      )}

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex flex-wrap items-center justify-between gap-3 px-6 pt-4">
          <div className="flex min-w-0 items-center gap-3">
            {narrow && (
              <Button ref={menuRef} variant="outline" size="sm" aria-label="Open menu" aria-expanded={menuOpen} onClick={() => setMenuOpen(true)}>
                <Menu className="size-4" strokeWidth={1.75} aria-hidden />Menu
              </Button>
            )}
            <AnimatedBackground defaultValue={face} onValueChange={(id) => { if (id) setFace(id as Face); }}
              className="flex rounded-sm bg-track p-1" transition={{ type: "spring", bounce: 0.1, duration: 0.35 }}>
              {tabs.map((t) => {
                const on = face === t.id;
                return (
                  <button key={t.id} type="button" data-id={t.id} role="tab" aria-selected={on} data-checked={on ? "" : undefined}
                    aria-label={t.id === "today" ? (t.count ? `Today, ${t.count} waiting` : "Today") : undefined}
                    className={cn("focus-ring relative z-10 flex min-h-9 items-center gap-2 rounded-sm px-3 text-sm font-medium outline-none", on ? "text-ink" : "text-ink-2 hover:text-ink")}>
                    {t.label}
                    {t.count > 0 && <span aria-hidden className="rounded-full bg-blue px-1.5 font-mono text-xs tabular-nums text-blue-fg">{t.count}</span>}
                  </button>
                );
              })}
            </AnimatedBackground>
          </div>
          <div role="status" id="ready-chip" className="flex items-center">
            <Pill variant="secondary" className={chip.cls === "ok" ? "bg-pill-active-bg text-pill-active-fg" : chip.cls === "wait" ? "bg-pill-warn-bg text-pill-warn-fg" : "text-ink-2"}>
              <span aria-hidden className={"size-2 rounded-full " + (chip.cls === "ok" ? "bg-dot-green" : chip.cls === "wait" ? "bg-dot-amber" : "bg-icon")} />
              <span id="ready-text">{chip.text}</span>
            </Pill>
          </div>
        </header>

        <UpdateStrip u={u} />

        {modelOff && (
          <div id="no-key-banner" className="mx-6 mt-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink">
            Ergon is not connected to a model yet, so the agent cannot run. Nothing here is simulated, and Ergon will not invent answers. Anything you type is kept in the thread.
            <span className="mt-1 block text-ink-2">For whoever runs Ergon: set ERGON_MODEL_API_KEY on the server.</span>
          </div>
        )}
        {recovered && (
          <div id="recovered-banner" className="mx-6 mt-3 flex items-center justify-between gap-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink">
            <span>Earlier conversations were moved to a backup file ({e.recovered.join(", ")}) and Ergon started fresh. Nothing was deleted.</span>
            <Button size="sm" variant="outline" onClick={dismissRecovered}>Dismiss</Button>
          </div>
        )}

        {face === "chat" && (
          <>
            <section id="thread" aria-live="polite" aria-label="Chat" className="min-h-0 flex-1 overflow-y-auto px-6 pb-6 pt-6">
              <Thread messages={e.messages} busy={e.busy} threadId={e.threadId} onReview={() => setFace("today")}
                onStarter={(t) => e.setDraft(t)} starters={e.starters}
                status={modelOff ? "wait" : "ok"} pendingNote={e.pendingNote} name={name} recent={e.recent} />
            </section>
            <div className="border-t border-hairline bg-canvas px-6 py-4">
              <Composer value={e.draft} onChange={e.setDraft} onSend={e.send} busy={e.busy} disabled={e.busy} />
            </div>
          </>
        )}
        {face === "today" && <div className="min-h-0 flex-1 overflow-y-auto"><TodayFace onChanged={e.refreshCounts} /></div>}
        {face === "graph" && <div className="min-h-0 flex-1 overflow-y-auto"><GraphFace /></div>}
      </div>
      <Toaster position="bottom-right" />
    </div>
  );
}
