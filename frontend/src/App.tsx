// Ergon's shell: rail (person, conversations, footer panels), top bar (Chat / Today / Work Graph), update strip, thread and composer.
import { useEffect, useState } from "react";
import { Drawer } from "vaul";
import { Menu, Moon, Sun, Plus } from "lucide-react";
import { Toaster } from "sonner";
import { Tabs, TabsList, TabsTab } from "@/components/ui/tabs";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { Composer, Thread } from "@/chat/Chat";
import { useErgon } from "./state";
import { useUpdate, UpdateStrip, VersionRow } from "@/shell/Update";
import { AddPersonPanel, LearnedPanel, SuitePanel } from "@/shell/Panels";
import { relTime, threadMs } from "@/lib/words";
import { LaterFace } from "@/views/Later";

type Face = "chat" | "today" | "graph";
type Theme = "light" | "dark";
const THEME_KEY = "ergon-theme";
const RECOVERED_KEY = "ergon-recovered-dismissed";

function useTheme(): [Theme, (t: Theme) => void] {
  const [t, setT] = useState<Theme>(() => (document.documentElement.getAttribute("data-theme") === "dark" ? "dark" : "light"));
  const set = (next: Theme) => {
    setT(next);
    document.documentElement.setAttribute("data-theme", next);
    try { localStorage.setItem(THEME_KEY, next); } catch { /* the choice still holds for this page */ }
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
  const [face, setFace] = useState<Face>("chat");
  const [menuOpen, setMenuOpen] = useState(false);
  const [recDismissed, setRecDismissed] = useState(false);

  const modelOff = e.status !== null && e.status !== "error" && !e.status.model_configured;
  const chip = e.status === null ? { cls: "", text: "Checking the agent" }
    : e.status === "error" ? { cls: "", text: "Could not check the agent" }
    : modelOff ? { cls: "wait", text: "Not connected to a model yet" }
    : e.runFailed ? { cls: "wait", text: "Last run did not finish" }
    : { cls: "ok", text: "Agent ready" };
  const recovered = e.recovered.length > 0 && !recDismissed && (() => { try { return localStorage.getItem(RECOVERED_KEY) !== e.recovered.join("|"); } catch { return true; } })();
  const dismissRecovered = () => { try { localStorage.setItem(RECOVERED_KEY, e.recovered.join("|")); } catch { /* the banner still hides for this view */ } setRecDismissed(true); };
  const personName = e.status && e.status !== "error" ? e.status.person?.name : "";

  const rail = (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center gap-2 px-4 pb-3 pt-4 text-title font-semibold text-ink">
        <svg viewBox="0 0 24 24" aria-hidden className="size-6 text-ink"><rect x="3" y="3" width="18" height="18" rx="5" fill="none" stroke="currentColor" strokeWidth="1.6" /><path d="M8 12.5l2.6 2.6L16 9.5" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>
        Ergon
      </div>
      <div id="sidebar" className="contents">
        <div className="px-4">
          <p id="person-label" className="text-xs text-ink-2">{personName ? "Working for " + personName : "Working for"}</p>
          <select id="person-select" aria-labelledby="person-label" value={e.people?.active ?? ""} disabled={!e.people}
            onChange={(ev) => e.switchPerson(ev.target.value)}
            className="mt-1 h-10 w-full rounded-sm border border-ink-3 bg-surface px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-blue">
            {(e.people?.people ?? []).map((p) => <option key={p.id} value={p.id}>{p.role ? p.name + ", " + p.role : p.name}</option>)}
          </select>
        </div>
        <div className="px-4 pt-3">
          <Button className="w-full shadow-primary" onClick={e.newThread}><Plus className="size-4" strokeWidth={1.75} aria-hidden />New chat</Button>
        </div>
        <div className="conv mt-5 flex min-h-0 flex-1 flex-col">
          <p className="px-4 text-xs text-ink-2">Conversations</p>
          <ul id="thread-list" aria-label="Conversations" className="mt-1 flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-2">
            {e.threadsError && (
              <li className="px-2 py-2 text-sm text-ink-2">Could not load your conversations. Check that Ergon is running. <Button size="sm" variant="outline" className="ml-2" onClick={e.loadThreads}>Retry</Button></li>
            )}
            {!e.threadsError && e.threads && !e.threads.length && <li className="px-2 py-2 text-sm text-ink-2">No conversations yet</li>}
            {(e.threads ?? []).map((t) => {
              const ms = threadMs(t.id);
              const active = t.id === e.threadId;
              return (
                <li key={t.id}>
                  <button type="button" onClick={() => { setMenuOpen(false); e.openThread(t.id); }} aria-current={active ? "true" : undefined}
                    className={"flex w-full min-w-0 min-h-11 flex-col items-start rounded-sm px-2.5 py-1.5 text-left outline-none focus-visible:ring-2 focus-visible:ring-blue " + (active ? "bg-blue-soft" : "hover:bg-board")}>
                    <span className="block w-full truncate text-sm font-medium text-ink">{t.title || "Untitled task"}</span>
                    {typeof t.messages === "number" && <span className="block w-full truncate text-xs text-ink-2">{(t.messages === 1 ? "1 message" : t.messages + " messages") + (ms ? ", " + relTime(ms) : "")}</span>}
                  </button>
                </li>
              );
            })}
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
          <SuitePanel onRun={(id, msgs) => { e.showRun(id, msgs); setFace("chat"); }} />
          <AddPersonPanel onAdded={() => { e.refreshPerson(); setMenuOpen(false); }} />
          <VersionRow u={u} />
        </div>
      </div>
    </div>
  );

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
          <div className="flex min-w-0 items-center gap-2">
            {narrow && (
              <Button variant="outline" size="sm" aria-label="Open menu" aria-expanded={menuOpen} onClick={() => setMenuOpen(true)}>
                <Menu className="size-4" strokeWidth={1.75} aria-hidden />Menu
              </Button>
            )}
            <Tabs value={face} onValueChange={(v) => setFace(v as Face)}>
              <TabsList variant="underline" aria-label="Views" id="face-bar">
                <TabsTab value="chat" data-face="chat">Chat</TabsTab>
                <TabsTab value="today" data-face="today">Today</TabsTab>
                <TabsTab value="graph" data-face="graph">Work Graph</TabsTab>
              </TabsList>
            </Tabs>
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
          <div role="status" id="no-key-banner" className="mx-6 mt-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink">
            Ergon is not connected to a model yet, so the agent cannot run. Nothing here is simulated, and Ergon will not invent answers. Anything you type is kept in the thread.
            <span className="mt-1 block text-ink-2">For whoever runs Ergon: set ERGON_MODEL_API_KEY on the server.</span>
          </div>
        )}
        {recovered && (
          <div role="status" id="recovered-banner" className="mx-6 mt-3 flex items-center justify-between gap-3 rounded-card border border-hairline bg-board px-4 py-2.5 text-sm text-ink">
            <span>Earlier conversations were moved to a backup file ({e.recovered.join(", ")}) and Ergon started fresh. Nothing was deleted.</span>
            <Button size="sm" variant="outline" onClick={dismissRecovered}>Dismiss</Button>
          </div>
        )}

        {face === "chat" && (
          <>
            <section id="thread" aria-live="polite" aria-label="Chat" className="min-h-0 flex-1 overflow-y-auto px-6 pb-6 pt-6">
              <Thread messages={e.messages} busy={e.busy} threadId={e.threadId} onReview={() => setFace("today")}
                onStarter={(t) => e.setDraft(t)} starters={e.starters}
                status={modelOff ? "wait" : "ok"} pendingNote={e.pendingNote} />
            </section>
            <div className="border-t border-hairline bg-canvas px-6 py-4">
              <Composer value={e.draft} onChange={e.setDraft} onSend={e.send} busy={e.busy} disabled={e.busy} />
            </div>
          </>
        )}
        {face !== "chat" && <LaterFace face={face} />}
      </div>
      <Toaster position="bottom-right" />
    </div>
  );
}
