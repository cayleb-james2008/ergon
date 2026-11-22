// Ergon v0.3.0 — Personal Employee Agent Desk
// Unified shell:
// - Persistent 256px Left Rail on >=768px (Vaul Drawer <768px): Ergon wordmark, active person desk,
//   primary New chat action, `/` conversation filter with Kbd hint, AnimatedBackground conversation glide,
//   Learned memory & Rune Suite/Library & Add-person MorphingDialogs with count chips, Light/Dark SegmentedControl, version row.
// - Top Navigation & Command Bar: COSS UI Tabs (Chat · Today [pending Pill] · Work Graph), Cmd/Ctrl+K task jump,
//   fixture honesty Pill, Agent readiness Pill, and BANNER-FIX-BRIEF.md-compliant UpdateStrip.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { MotionConfig } from "motion/react";
import { Drawer } from "vaul";
import { Check, Hourglass, Menu, Moon, Plus, Search, Sun, X } from "lucide-react";
import { Toaster } from "sonner";
import { AnimatedBackground } from "@/components/ui/animated-background";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Tabs, TabsList, TabsTab } from "@/components/ui/tabs";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Kbd } from "@/components/ui/kbd";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { Composer, ContourField, Thread } from "@/chat/Chat";
import { useErgon, type TState } from "./state";
import { UpdateStrip, VersionRow, useUpdate } from "@/shell/Update";
import { AddPersonPanel, LearnedPanel, SuitePanel } from "@/shell/Panels";
import { PaperFace, usePaper } from "@/views/Paper";
import { GraphFace } from "@/views/Graph";
import { PersonSelect } from "@/shell/PersonSelect";
import { cn } from "@/lib/utils";
import { relTime, threadMs } from "@/lib/words";
import { glide } from "@/motion";

type Face = "chat" | "paper" | "graph";
type Theme = "light" | "dark";
const RECOVERED_KEY = "ergon-recovered-dismissed";

function initialFace(): Face {
  try {
    const q = new URLSearchParams(window.location.search).get("face");
    if (q === "chat") return "chat";
    if (q === "graph") return "graph";
    if (q === "today" || q === "paper") return "paper";
    if (sessionStorage.getItem("ergon-thread")) return "chat";
  } catch { /* ignore */ }
  return "paper";
}

function useTheme(): [Theme, (t: Theme) => void] {
  const [t, setT] = useState<Theme>(() => (document.documentElement.getAttribute("data-theme") === "dark" ? "dark" : "light"));
  const set = (next: Theme) => {
    setT(next);
    document.documentElement.setAttribute("data-theme", next);
    try { localStorage.setItem("ergon-theme", next); } catch { /* transient fallback */ }
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

export function App() {
  const e = useErgon();
  const u = useUpdate();
  const [theme, setTheme] = useTheme();
  const narrow = useNarrow();
  const [face, setFace] = useState<Face>(initialFace);
  const [chatQuery, setChatQuery] = useState("");
  const paper = usePaper(() => { e.refreshCounts(); e.loadRecent(); });

  useEffect(() => {
    if (face === "paper") paper.load();
  }, [face]); // eslint-disable-line react-hooks/exhaustive-deps

  const [menuOpen, setMenuOpen] = useState(false);
  const [recDismissed, setRecDismissed] = useState(false);
  const menuRef = useRef<HTMLButtonElement>(null);
  const newChatRef = useRef<HTMLButtonElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const wasOpen = useRef(false);

  // Drawer focus return: open -> focus New chat; close -> return focus to Menu button.
  useEffect(() => {
    if (menuOpen) {
      wasOpen.current = true;
      setTimeout(() => newChatRef.current?.focus(), 0);
    } else if (wasOpen.current) {
      wasOpen.current = false;
      menuRef.current?.focus();
    }
  }, [menuOpen]);

  const jumpToComposer = useCallback(() => {
    setMenuOpen(false);
    setFace("chat");
    setTimeout(() => document.getElementById("composer-input")?.focus(), 20);
  }, []);

  // Global keyboard shortcuts: Cmd/Ctrl+K -> jump to task composer; `/` outside inputs -> focus conversation filter.
  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if ((ev.metaKey || ev.ctrlKey) && ev.key.toLowerCase() === "k") {
        ev.preventDefault();
        jumpToComposer();
        return;
      }
      if (ev.key === "/" && !ev.metaKey && !ev.ctrlKey && !ev.altKey) {
        const el = document.activeElement as HTMLElement | null;
        const inField = el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable);
        if (inField) return;
        ev.preventDefault();
        if (narrow) {
          setMenuOpen(true);
          setTimeout(() => document.getElementById("chat-search")?.focus(), 60);
        } else {
          searchRef.current?.focus();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [jumpToComposer, narrow]);

  const modelOff = e.status !== null && e.status !== "error" && !e.status.model_configured;
  const chip = e.status === null
    ? { cls: "", text: "Checking the agent" }
    : e.status === "error"
      ? { cls: "", text: "Could not check the agent" }
      : modelOff
        ? { cls: "wait", text: "Not connected to a model yet" }
        : e.runFailed
          ? { cls: "wait", text: "Last run did not finish" }
          : { cls: "ok", text: "Agent ready" };

  const recovered = e.recovered.length > 0 && !recDismissed && (() => {
    try { return localStorage.getItem(RECOVERED_KEY) !== e.recovered.join("|"); } catch { return true; }
  })();
  const dismissRecovered = () => {
    try { localStorage.setItem(RECOVERED_KEY, e.recovered.join("|")); } catch { /* */ }
    setRecDismissed(true);
  };

  const personName = e.status && e.status !== "error" ? String(e.status.person?.name ?? "") : "";
  const greetingName = personName === "You" ? "" : personName;
  const personRole = e.status && e.status !== "error" ? String(e.status.person?.role ?? "") : "";

  const filteredThreads = useMemo(() => {
    const all = e.threads ?? [];
    const q = chatQuery.trim().toLowerCase();
    if (!q) return all;
    return all.filter((t) => String(t.title ?? "Untitled task").toLowerCase().includes(q));
  }, [e.threads, chatQuery]);

  const pendingDecisionsCount = paper.data
    ? paper.data.decisions.filter((d) => d.status === "pending").length
    : e.todayCount;

  const stateIcon = (s: TState | undefined) => s === "waiting"
    ? <Hourglass className="size-3.5 shrink-0 text-dot-amber" strokeWidth={1.75} aria-label="Waiting on you" />
    : s === "finished"
      ? <Check className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.75} aria-label="Finished" />
      : null;

  const appearance = (
    <SegmentedControl aria-label="Appearance">
      {(["light", "dark"] as const).map((t) => (
        <SegmentedControlItem
          key={t}
          data-id={t}
          aria-pressed={theme === t}
          data-checked={theme === t ? "" : undefined}
          className="max-md:min-h-11"
          onClick={() => setTheme(t)}
        >
          {t === "light" ? <Sun className="size-3.5" strokeWidth={1.5} aria-hidden /> : <Moon className="size-3.5" strokeWidth={1.5} aria-hidden />}
          {t === "light" ? "Light" : "Dark"}
        </SegmentedControlItem>
      ))}
    </SegmentedControl>
  );

  const wordmark = (
    <span className="flex items-center gap-2 text-title font-medium tracking-[-0.03em] text-ink">
      <svg viewBox="0 0 24 24" aria-hidden className="size-5 text-ink">
        <rect x="3" y="3" width="18" height="18" rx="5" fill="none" stroke="currentColor" strokeWidth="1.6" />
        <path d="M8 12.5l2.6 2.6L16 9.5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
      Ergon
    </span>
  );

  const railContent = (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center justify-between px-3.5 pb-2.5 pt-3.5">
        {wordmark}
      </div>

      <div id="sidebar" className="flex min-h-0 flex-1 flex-col">
        <div className="px-3.5">
          <label id="person-label" htmlFor="person-select" className="text-xs text-ink-3">Working for</label>
          <PersonSelect
            people={e.people?.people ?? []}
            active={e.people?.active ?? ""}
            disabled={!e.people}
            onChange={async (id) => {
              await e.switchPerson(id);
              paper.load();
              setMenuOpen(false);
            }}
          />
        </div>

        <div className="px-3.5 pt-2.5">
          <Button
            ref={newChatRef}
            className="w-full shadow-primary max-md:min-h-11"
            onClick={() => {
              setFace("chat");
              setMenuOpen(false);
              e.newThread();
              setTimeout(() => document.getElementById("composer-input")?.focus(), 20);
            }}
          >
            <Plus className="size-4" strokeWidth={1.5} aria-hidden />
            New chat
          </Button>
        </div>

        <div className="px-3.5 pt-2.5">
          <InputGroup className="rounded-sm bg-board/50">
            <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
              <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              id="chat-search"
              ref={searchRef}
              type="search"
              aria-label="Filter conversations"
              placeholder="Filter chats"
              value={chatQuery}
              onChange={(ev) => setChatQuery(ev.target.value)}
              className="max-md:min-h-11 text-sm"
            />
            <InputGroupAddon align="inline-end" className="pr-2">
              {chatQuery ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  className="max-md:min-h-11 max-md:min-w-11"
                  aria-label="Clear conversation filter"
                  onClick={() => setChatQuery("")}
                >
                  <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                </Button>
              ) : (
                <Kbd>/</Kbd>
              )}
            </InputGroupAddon>
          </InputGroup>
        </div>

        <div className="mt-3 flex min-h-0 flex-1 flex-col">
          <div className="flex items-center justify-between px-3.5 pb-1">
            <span className="text-xs font-medium text-ink-3">Conversations</span>
            {e.threads && <span className="mono text-xs text-ink-3">{filteredThreads.length}</span>}
          </div>
          <ul id="thread-list" aria-label="Conversations" className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-2">
            {e.threadsError && (
              <li className="px-2 py-2 text-sm text-ink-2">
                Could not load your conversations. Check that Ergon is running.
                <div className="mt-2">
                  <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={e.loadThreads}>Retry</Button>
                </div>
              </li>
            )}
            {!e.threadsError && e.threads && !e.threads.length && (
              <li className="px-2 py-2 text-sm text-ink-2">No conversations yet</li>
            )}
            {!e.threadsError && e.threads && e.threads.length > 0 && !filteredThreads.length && (
              <li className="px-2 py-2 text-sm text-ink-2">
                No chats match &ldquo;{chatQuery}&rdquo;.
                <div className="mt-1.5">
                  <Button size="xs" variant="outline" className="max-md:min-h-11" onClick={() => setChatQuery("")}>Clear filter</Button>
                </div>
              </li>
            )}
            {filteredThreads.length > 0 && (
              <AnimatedBackground
                defaultValue={face === "chat" && e.threadId ? e.threadId : undefined}
                transition={glide}
                className="rounded-sm bg-blue-soft"
              >
                {filteredThreads.map((t) => {
                  const ms = threadMs(t.id);
                  const active = face === "chat" && t.id === e.threadId;
                  const s = e.states[t.id];
                  const title = String(t.title ?? "").trim() || "Untitled task";
                  return (
                    <li key={t.id} data-id={t.id} className="flex w-full min-w-0">
                      <Button
                        variant="ghost"
                        aria-current={active ? "true" : undefined}
                        onClick={() => {
                          setMenuOpen(false);
                          setFace("chat");
                          e.openThread(t.id);
                        }}
                        className={cn(
                          "thread-btn h-auto sm:h-auto min-h-11 w-full min-w-0 flex-col items-stretch justify-start gap-0.5 whitespace-normal rounded-sm px-2.5 py-1.5 text-left font-normal md:min-h-11",
                          active ? "bg-transparent text-ink" : "hover:bg-board"
                        )}
                      >
                        <span className="block min-w-0 truncate text-sm font-medium text-ink">{title}</span>
                        <span className="flex min-w-0 items-center gap-1.5 text-xs text-ink-2">
                          {stateIcon(s)}
                          <span className="mono truncate">
                            {ms ? relTime(ms) : ""}
                            {typeof t.messages === "number" ? (ms ? " · " : "") + (t.messages === 1 ? "1 msg" : `${t.messages} msgs`) : ""}
                          </span>
                        </span>
                      </Button>
                    </li>
                  );
                })}
              </AnimatedBackground>
            )}
          </ul>
        </div>

        {e.sideError && <p role="alert" className="px-3.5 pb-2 text-xs text-destructive-foreground">{e.sideError}</p>}

        <div id="settings" className="flex flex-col gap-0.5 border-t border-hairline px-1.5 py-2.5">
          <LearnedPanel count={e.learnedCount} />
          <SuitePanel
            count={e.suiteCount}
            onChanged={() => e.loadStarters(personRole)}
            onRun={(id, msgs) => {
              e.showRun(id, msgs);
              setFace("chat");
              setMenuOpen(false);
            }}
          />
          <AddPersonPanel
            onAdded={() => {
              e.refreshPerson();
              paper.load();
              setMenuOpen(false);
            }}
          />
          <div className="flex items-center justify-between gap-2 px-2.5 pt-1.5">
            <span className="text-xs text-ink-2">Appearance</span>
            {appearance}
          </div>
          <VersionRow u={u} />
        </div>
      </div>
    </div>
  );

  const dateLabel = new Date().toLocaleDateString([], { weekday: "long", month: "long", day: "numeric" });

  return (
    <MotionConfig reducedMotion="user">
      <div className="flex h-dvh min-h-0 overflow-hidden bg-canvas text-ink">
        {!narrow && (
          <aside aria-label="Ergon navigation" className="flex w-64 shrink-0 flex-col border-r border-hairline bg-surface">
            {railContent}
          </aside>
        )}

        {narrow && (
          <Drawer.Root open={menuOpen} onOpenChange={setMenuOpen} direction="left">
            <Drawer.Portal>
              <Drawer.Overlay className="fixed top-0 right-0 bottom-0 left-0 z-40 bg-ink/40" />
              <Drawer.Content
                aria-label="Ergon navigation"
                className="fixed inset-y-0 left-0 z-50 flex w-[288px] max-w-[88vw] flex-col border-r border-hairline bg-surface outline-none"
              >
                <Drawer.Title className="sr-only">Ergon navigation</Drawer.Title>
                {railContent}
              </Drawer.Content>
            </Drawer.Portal>
          </Drawer.Root>
        )}

        <div className="relative isolate flex min-w-0 flex-1 flex-col overflow-hidden bg-canvas">
          <ContourField />

          <header className="flex w-full flex-wrap items-center justify-between gap-2.5 border-b border-hairline bg-surface/90 px-4 py-2.5 md:px-6">
            <div className="flex min-w-0 flex-wrap items-center gap-2.5">
              {narrow && (
                <Button
                  ref={menuRef}
                  variant="outline"
                  size="sm"
                  className="min-h-11"
                  aria-label="Open menu"
                  data-answers-via="4"
                  onClick={() => setMenuOpen(true)}
                >
                  <Menu className="size-4" strokeWidth={1.5} aria-hidden />
                  Menu
                </Button>
              )}

              <Tabs
                id="face-bar"
                value={face}
                onValueChange={(v) => { if (v === "chat" || v === "paper" || v === "graph") setFace(v); }}
                className="min-w-0"
              >
                <TabsList aria-label="Ergon views" className="max-w-full overflow-x-auto bg-track p-0.5">
                  <TabsTab value="chat" data-id="chat" data-face="chat" className="max-md:min-h-11 px-3">
                    Chat
                  </TabsTab>
                  <TabsTab
                    value="paper"
                    data-id="paper"
                    data-face="today"
                    aria-label={pendingDecisionsCount > 0 ? `Today's paper, ${pendingDecisionsCount} waiting` : "Today's paper"}
                    className="max-md:min-h-11 gap-1.5 px-3"
                  >
                    Today
                    {pendingDecisionsCount > 0 && (
                      <Pill className="status-waiting border-0 px-1.5 py-0 text-xs font-medium" aria-hidden>
                        <span className="mono">{pendingDecisionsCount}</span>
                      </Pill>
                    )}
                  </TabsTab>
                  <TabsTab value="graph" data-id="graph" data-face="graph" className="max-md:min-h-11 px-3">
                    Work Graph
                  </TabsTab>
                </TabsList>
              </Tabs>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                className="hidden items-center gap-1.5 text-xs text-ink-2 hover:text-ink lg:inline-flex"
                onClick={jumpToComposer}
              >
                <span>New task</span>
                <Kbd>⌘K</Kbd>
              </Button>

              {e.isDemo && (
                <Pill variant="secondary" className="border border-hairline bg-board px-2.5 py-0.5 text-xs font-normal text-ink-2">
                  <span className="mono">fixture demo</span>
                </Pill>
              )}

              <div role="status" id="ready-chip" className="flex items-center">
                <Pill
                  variant="secondary"
                  className={cn(
                    "px-2.5 py-1 text-xs font-medium",
                    chip.cls === "ok"
                      ? "bg-pill-active-bg text-pill-active-fg"
                      : chip.cls === "wait"
                        ? "bg-pill-warn-bg text-pill-warn-fg"
                        : "bg-track text-ink-2"
                  )}
                >
                  <span
                    aria-hidden
                    className={cn(
                      "size-2 rounded-full",
                      chip.cls === "ok" ? "bg-dot-green" : chip.cls === "wait" ? "bg-dot-amber" : "bg-icon"
                    )}
                  />
                  <span id="ready-text">{chip.text}</span>
                </Pill>
              </div>
            </div>
          </header>

          <UpdateStrip u={u} />

          {modelOff && (
            <div id="no-key-banner" className="mx-4 mt-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink md:mx-6">
              Ergon is not connected to a model yet, so the agent cannot run. Nothing here is simulated, and Ergon will not invent answers. Anything you type is kept in the thread.
              <span className="mt-1 block text-ink-2">For whoever runs Ergon: set <span className="mono">ERGON_MODEL_API_KEY</span> on the server.</span>
            </div>
          )}

          {recovered && (
            <div id="recovered-banner" className="mx-4 mt-3 flex flex-wrap items-center justify-between gap-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink md:mx-6">
              <span>Earlier conversations were moved to a backup file ({e.recovered.join(", ")}) and Ergon started fresh. Nothing was deleted.</span>
              <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={dismissRecovered}>Dismiss</Button>
            </div>
          )}

          {face === "chat" && (
            <>
              <section id="thread" aria-live="polite" aria-label="Chat" className="min-h-0 flex-1 overflow-y-auto px-4 pb-6 pt-5 md:px-6">
                <Thread
                  messages={e.messages}
                  busy={e.busy}
                  threadId={e.threadId}
                  onReview={() => setFace("paper")}
                  onStarter={(t) => e.setDraft(t)}
                  starters={e.starters}
                  status={modelOff ? "wait" : "ok"}
                  pendingNote={e.pendingNote}
                  name={greetingName}
                  recent={e.recent}
                />
              </section>
              <div className="border-t border-hairline bg-surface/80 px-4 py-3.5 md:px-6">
                <Composer value={e.draft} onChange={e.setDraft} onSend={e.send} busy={e.busy} disabled={e.busy} />
              </div>
            </>
          )}

          {face === "paper" && (
            <div id="face-today" className="min-h-0 flex-1 overflow-y-auto">
              <PaperFace
                paper={paper}
                personName={personName}
                dateLabel={dateLabel}
                people={e.people?.people ?? null}
                teamError={e.people ? "" : e.sideError}
                activePerson={e.people?.active ?? ""}
                onSwitchPerson={async (id) => {
                  await e.switchPerson(id);
                  paper.load();
                }}
                onAsk={jumpToComposer}
                onOpen={(id) => {
                  setFace("chat");
                  e.openThread(id);
                }}
              />
            </div>
          )}

          {face === "graph" && (
            <div className="min-h-0 flex-1 overflow-y-auto">
              <GraphFace />
            </div>
          )}
        </div>
        <Toaster position="bottom-right" />
      </div>
    </MotionConfig>
  );
}
