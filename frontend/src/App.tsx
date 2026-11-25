// Ergon v0.10.0 — Unified Responsive Claude-Desktop Personal Employee Workbench:
// - Slim Top Bar: Rail toggle, Active Workspace/Company switcher (#person-select), View switcher (Chat / Today / Work graph), Commands (⌘K), Workspace inspector toggle, Agent readiness pill
// - Collapsible Left Rail (desktop pane / mobile Vaul Drawer): New chat, `/` filter, paginated Conversations (#thread-list), Scheduled Routines quick drawer with cadence presets, Workflows / Learned / Suite / Add person / Settings (#67 Light/Dark only in Settings)
// - Central Thread / Today / Work Graph: inline Approvals & File Diffs, 1-click Deliverable Starters, Word/Excel & SHA-256 export
// - Collapsible Right Inspector Drawer: Workspace Files, Version History & Diff, SHA-256 Audit Bundle & Offers
import { Suspense, lazy, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, LazyMotion, MotionConfig, domAnimation, m, useReducedMotion } from "motion/react";
import { Drawer } from "vaul";
import {
  Blocks,
  BookOpen,
  CalendarClock,
  Command as CommandIcon,
  FolderOpen,
  Menu,
  PanelLeft,
  PanelRight,
  Play,
  Plus,
  Search,
  SlidersHorizontal,
  Sparkles,
  UserPlus,
  X,
} from "lucide-react";
import { Toaster, toast } from "sonner";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Chat } from "@/chat/Chat";
import { usePaper } from "@/views/usePaper";
import { RailPanelButton } from "@/shell/RailPanelButton";
import type { InspectorTab } from "@/shell/InspectorDrawer";
import { PersonSelect, WorkspaceSwitcherPanel } from "@/shell/PersonSelect";
import type { Theme } from "@/shell/SettingsPanel";
import { UpdateStrip, VersionRow, useUpdate } from "@/shell/Update";

const PaperFace = lazy(() => import("@/views/Paper"));
const GraphFace = lazy(() => import("@/views/Graph").then((mod) => ({ default: mod.GraphFace })));
const AddPersonPanel = lazy(() => import("@/shell/Panels").then((mod) => ({ default: mod.AddPersonPanel })));
const LearnedPanel = lazy(() => import("@/shell/Panels").then((mod) => ({ default: mod.LearnedPanel })));
const SuitePanel = lazy(() => import("@/shell/Panels").then((mod) => ({ default: mod.SuitePanel })));
const FileViewerModal = lazy(() => import("@/shell/WorkspaceFiles").then((mod) => ({ default: mod.FileViewerModal })));
const InspectorDrawer = lazy(() => import("@/shell/InspectorDrawer").then((mod) => ({ default: mod.InspectorDrawer })));
const SettingsPanel = lazy(() => import("@/shell/SettingsPanel").then((mod) => ({ default: mod.SettingsPanel })));
const SkillsPanel = lazy(() => import("@/shell/SkillsPanel").then((mod) => ({ default: mod.SkillsPanel })));
const RoutinesPanel = lazy(() => import("@/shell/RoutinesPanel").then((mod) => ({ default: mod.RoutinesPanel })));
const CommandPalette = lazy(() => import("@/shell/CommandPalette").then((mod) => ({ default: mod.CommandPalette })));
import { useErgon } from "@/state";
import { call, hasV050Api, type Routine } from "@/lib/api";
import { exportAuditBundleDownload } from "@/lib/auditBundle";
import { exportReceiptDownload } from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { relTime, threadMs } from "@/lib/words";
import { glide, instant, sheet } from "@/motion";

type Face = "chat" | "paper" | "graph";
const THREADS_PAGE_SIZE = 12;

function useTheme(): [Theme, (t: Theme) => void] {
  const [theme, setTheme] = useState<Theme>(() => {
    try {
      const saved = localStorage.getItem("ergon-theme");
      if (saved === "light" || saved === "dark") return saved;
    } catch { /* ignore */ }
    return typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const set = useCallback((t: Theme) => {
    setTheme(t);
    document.documentElement.dataset.theme = t;
    try {
      localStorage.setItem("ergon-theme", t);
    } catch { /* ignore */ }
  }, []);

  return [theme, set];
}

function initialFace(): Face {
  try {
    const q = new URLSearchParams(window.location.search).get("face");
    if (q === "today" || q === "paper") return "paper";
    if (q === "graph") return "graph";
    if (q === "chat") return "chat";
  } catch { /* ignore */ }
  return "chat";
}

function Banner({
  id,
  tone,
  children,
}: {
  id: string;
  tone: "warn" | "info";
  children: React.ReactNode;
}) {
  return (
    <div
      id={id}
      role={tone === "warn" ? "alert" : "status"}
      className={cn(
        "flex flex-wrap items-center justify-between gap-3 border-b border-hairline px-4 py-2 text-sm",
        tone === "warn" ? "bg-pill-warn-bg text-ink" : "bg-surface text-ink-2"
      )}
    >
      {children}
    </div>
  );
}

function NoKeyBanner({ onOpenSettings }: { onOpenSettings: () => void }) {
  return (
    <Banner id="no-key-banner" tone="warn">
      <div className="flex min-w-0 flex-1 flex-wrap items-baseline gap-2">
        <span className="font-medium text-ink">Your agent is not connected to a model yet.</span>
        <span className="text-ink-2">
          Connect a local or hosted AI model in Settings (or set ERGON_MODEL_API_KEY) to run tasks live.
        </span>
      </div>
      <Button size="xs" variant="outline" className="max-md:min-h-11 shrink-0" onClick={onOpenSettings}>
        Open Settings
      </Button>
    </Banner>
  );
}

function RecoveredBanner({ files, onDismiss }: { files: string[]; onDismiss: () => void }) {
  return (
    <Banner id="recovered-banner" tone="info">
      <div className="min-w-0 flex-1">
        <span>Saved history recovered for {files.join(", ")}.</span>
      </div>
      <Button size="xs" variant="ghost" className="max-md:min-h-11 shrink-0" onClick={onDismiss}>
        Dismiss
      </Button>
    </Banner>
  );
}

export function App() {
  const desk = useErgon();
  const [theme, setTheme] = useTheme();
  const u = useUpdate();
  const reduceMotion = useReducedMotion();
  const [face, setFace] = useState<Face>(initialFace);
  const [railOpen, setRailOpen] = useState(() =>
    typeof window !== "undefined" ? window.innerWidth >= 1024 : true
  );
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>("files");
  const [selectedHistoryPath, setSelectedHistoryPath] = useState("");
  const [isXl, setIsXl] = useState(() =>
    typeof window !== "undefined" ? window.innerWidth >= 1280 : true
  );
  const [isMd, setIsMd] = useState(() =>
    typeof window !== "undefined" ? window.innerWidth >= 768 : true
  );
  const [mobileOpen, setMobileOpen] = useState(false);

  const [switcherOpen, setSwitcherOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [skillsOpen, setSkillsOpen] = useState(false);
  const [routinesOpen, setRoutinesOpen] = useState(false);
  const [learnedOpen, setLearnedOpen] = useState(false);
  const [suiteOpen, setSuiteOpen] = useState(false);
  const [addPersonOpen, setAddPersonOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [viewerPath, setViewerPath] = useState<string | null>(null);
  const [threadFilter, setThreadFilter] = useState("");
  const [threadPage, setThreadPage] = useState(0);
  const [routinesExpanded, setRoutinesExpanded] = useState(true);

  const [graphSeed, setGraphSeed] = useState(0);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const fileUploadRef = useRef<HTMLInputElement>(null);

  const onPaperChanged = useCallback(() => {
    desk.loadThreads();
    desk.loadRecent();
    desk.refreshCounts();
    desk.loadFiles();
    setGraphSeed((s) => s + 1);
  }, [desk]);

  const paper = usePaper(onPaperChanged, desk.status);

  useEffect(() => {
    const mqXl = window.matchMedia?.("(min-width: 1280px)");
    const mqMd = window.matchMedia?.("(min-width: 768px)");
    const sync = () => {
      if (mqXl) setIsXl(mqXl.matches);
      if (mqMd) setIsMd(mqMd.matches);
    };
    sync();
    mqXl?.addEventListener("change", sync);
    mqMd?.addEventListener("change", sync);
    return () => {
      mqXl?.removeEventListener("change", sync);
      mqMd?.removeEventListener("change", sync);
    };
  }, []);

  useEffect(() => {
    desk.setOnRoutineFired(() => {
      paper.load();
      setGraphSeed((s) => s + 1);
    });
    return () => desk.setOnRoutineFired(null);
  }, [desk, paper]);

  const statusObj = desk.status && desk.status !== "error" ? desk.status : null;
  const peopleList = useMemo(() => desk.people?.people ?? [], [desk.people]);
  const activePersonId = desk.people?.active || statusObj?.person?.id || "";
  const activePerson = useMemo(
    () => peopleList.find((p) => p.id === activePersonId) || statusObj?.person,
    [peopleList, activePersonId, statusObj?.person]
  );
  const personName = String(activePerson?.name ?? "").trim() || "Personal desk";
  const pendingDecisions = useMemo(
    () => (paper.data?.decisions ?? []).filter((d) => d.status === "pending"),
    [paper.data]
  );
  const pendingCount = paper.data ? pendingDecisions.length : desk.todayCount;
  const threadList = useMemo(() => desk.threads ?? [], [desk.threads]);

  const filteredThreads = useMemo(() => {
    const q = threadFilter.trim().toLowerCase();
    if (!q) return threadList;
    return threadList.filter((t) => String(t.title ?? "").toLowerCase().includes(q));
  }, [threadList, threadFilter]);

  const totalThreadPages = Math.max(1, Math.ceil(filteredThreads.length / THREADS_PAGE_SIZE));
  const safeThreadPage = Math.min(threadPage, totalThreadPages - 1);
  const visibleThreads = filteredThreads.slice(
    safeThreadPage * THREADS_PAGE_SIZE,
    (safeThreadPage + 1) * THREADS_PAGE_SIZE
  );

  useEffect(() => {
    setThreadPage(0);
  }, [threadFilter, threadList.length]);

  const switchFace = useCallback(
    (next: Face) => {
      setFace(next);
      setMobileOpen(false);
      if (next === "paper") paper.load();
      if (next === "graph") setGraphSeed((s) => s + 1);
    },
    [paper]
  );

  const focusComposer = useCallback(() => {
    setFace("chat");
    setMobileOpen(false);
    setPaletteOpen(false);
    window.setTimeout(() => inputRef.current?.focus(), 80);
  }, []);

  const openFilesInspector = useCallback(() => {
    setInspectorTab("files");
    setInspectorOpen(true);
    setMobileOpen(false);
    desk.loadFiles();
  }, [desk]);

  const openHistoryInspector = useCallback(() => {
    setInspectorTab("history");
    setInspectorOpen(true);
    setMobileOpen(false);
    desk.loadFiles();
  }, [desk]);

  const handleResolveDecision = useCallback(
    async (id: string, action: "approve" | "deny", label: string) => {
      const url = `/api/decisions/${encodeURIComponent(id)}/${action}`;
      const r = await call<{ outcome?: string }>(url, { method: "POST", body: {} });
      if (!r.ok) {
        toast.error(r.error || "Could not update decision.");
        return;
      }
      const rawOutcome = typeof r.data?.outcome === "string" ? r.data.outcome.trim() : "";
      const toastMsg =
        action === "approve"
          ? rawOutcome
            ? (/approved/i.test(rawOutcome) ? rawOutcome : `Approved: ${rawOutcome}`)
            : `Approved action for ${label}.`
          : rawOutcome || `Kept ${label} safe.`;
      toast.success(toastMsg);
      await Promise.all([
        paper.load(),
        desk.threadId ? desk.openThread(desk.threadId) : Promise.resolve(),
      ]);
      onPaperChanged();
    },
    [desk, onPaperChanged, paper]
  );

  const handleExportAuditBundle = useCallback(
    async (format: "json" | "md") => {
      const res = await exportAuditBundleDownload({
        format,
        status: desk.status,
        personName,
        personId: activePerson?.id,
        files: desk.files,
        receipts: paper.data?.brief.finished ?? [],
        routines: desk.routines,
      });
      toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
    },
    [activePerson?.id, desk.files, desk.routines, desk.status, paper.data, personName]
  );

  const handleExportLatestReceipt = useCallback(
    async (format: "md" | "json") => {
      const latest = paper.data?.brief.finished?.[0];
      if (!latest) {
        toast("No finished receipt to export yet.");
        return;
      }
      const res = await exportReceiptDownload(
        {
          id: latest.id || "latest",
          task: String(latest.task ?? "Completed task"),
          person: personName,
          status: "finished",
          summary: String(latest.summary ?? ""),
          files: Array.isArray(latest.files) ? latest.files.map(String) : [],
        },
        format,
        hasV050Api(desk.status, "receipts_export") && Boolean(latest.id)
      );
      toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
    },
    [desk.status, paper.data, personName]
  );

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if ((ev.metaKey || ev.ctrlKey) && ev.key.toLowerCase() === "k") {
        ev.preventDefault();
        setPaletteOpen((v) => !v);
        return;
      }
      if ((ev.metaKey || ev.ctrlKey) && ev.key.toLowerCase() === "i") {
        ev.preventDefault();
        setInspectorOpen((v) => {
          const next = !v;
          if (next) desk.loadFiles();
          return next;
        });
        return;
      }
      if (ev.key === "Escape") {
        if (!isXl && inspectorOpen && !document.querySelector('[role="dialog"]')) {
          setInspectorOpen(false);
        }
        return;
      }
      const target = ev.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);
      if (!typing && ev.key === "/") {
        ev.preventDefault();
        if (!railOpen) setRailOpen(true);
        if (window.innerWidth < 768) setMobileOpen(true);
        requestAnimationFrame(() => searchRef.current?.focus());
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [desk, railOpen, isXl, inspectorOpen]);

  const runAfterPersonChange = useCallback(() => {
    paper.load();
    desk.loadFiles();
    desk.loadRoutines();
    setGraphSeed((s) => s + 1);
  }, [desk, paper]);

  const dateLabel = paper.data?.brief.date || new Date().toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
  });

  const readinessState =
    desk.status === "error"
      ? "Disconnected"
      : desk.busy
        ? "Busy"
        : statusObj && !statusObj.model_configured
          ? "Setup needed"
          : "Ready";

  const renderRailContent = () => (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 flex-col gap-2.5 p-3">
        <div className="flex items-center justify-between gap-2">
          <span className="text-sm font-medium tracking-tight text-ink">Ergon</span>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() => {
              desk.newThread();
              focusComposer();
            }}
            className="max-md:min-h-11 gap-1.5"
          >
            <Plus className="size-3.5" strokeWidth={1.75} aria-hidden />
            New chat
          </Button>
        </div>

        <div className="relative flex items-center">
          <Search className="pointer-events-none absolute left-2.5 size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
          <Input
            ref={searchRef}
            id="chat-search"
            value={threadFilter}
            onChange={(e) => setThreadFilter(e.target.value)}
            placeholder="Filter conversations (/)"
            aria-label="Filter conversations"
            className="h-9 border-input bg-surface pl-8 pr-8 text-sm max-md:min-h-11"
          />
          {threadFilter && (
            <button
              type="button"
              aria-label="Clear conversation filter"
              onClick={() => setThreadFilter("")}
              className="focus-ring absolute right-1.5 grid size-7 place-items-center rounded-sm text-icon hover:bg-board hover:text-ink"
            >
              <X className="size-3.5" strokeWidth={1.5} aria-hidden />
            </button>
          )}
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        <div className="flex items-center justify-between px-2 py-1">
          <span className="text-xs font-medium text-ink-3">Conversations</span>
          {threadList.length > 0 && (
            <span className="mono text-xs text-ink-3">{filteredThreads.length}</span>
          )}
        </div>

        {desk.threadsError ? (
          <div role="alert" className="mx-1 rounded-sm border border-hairline bg-board p-2.5 text-xs text-ink-2">
            <p>Could not load your conversations.</p>
            <Button size="xs" variant="ghost" className="mt-1 px-0" onClick={() => desk.loadThreads()}>
              Retry
            </Button>
          </div>
        ) : visibleThreads.length === 0 ? (
          <p className="px-2 py-2 text-xs text-ink-3">
            {threadFilter ? "No matching conversations." : "No past conversations yet."}
          </p>
        ) : (
          <ul id="thread-list" className="flex flex-col gap-0.5">
            {visibleThreads.map((t) => {
              const active = desk.threadId === t.id && face === "chat";
              const st = desk.states[t.id];
              const ms = threadMs(t.id);
              return (
                <li key={t.id} className="relative">
                  {active && (
                    <span
                      aria-hidden
                      className="pointer-events-none absolute inset-0 rounded-sm bg-board"
                    />
                  )}
                  <button
                    type="button"
                    aria-current={active ? "page" : undefined}
                    onClick={() => {
                      desk.openThread(t.id);
                      setFace("chat");
                      setMobileOpen(false);
                    }}
                    className={cn(
                      "thread-btn focus-ring relative z-10 flex min-h-11 w-full flex-col gap-0.5 rounded-sm px-2.5 py-2 text-left outline-none md:min-h-9",
                      active ? "text-ink" : "text-ink-2 hover:bg-board/60 hover:text-ink"
                    )}
                  >
                    <span className="flex w-full items-center gap-2">
                      {st === "waiting" ? (
                        <span className="size-2 shrink-0 rounded-full bg-dot-amber" aria-label="Waiting on you" />
                      ) : st === "finished" ? (
                        <span className="size-2 shrink-0 rounded-full bg-dot-green" aria-label="Finished" />
                      ) : (
                        <span className="size-2 shrink-0 rounded-full bg-hairline" aria-hidden />
                      )}
                      <span className={cn("min-w-0 flex-1 truncate text-sm", active && "font-medium text-ink")}>
                        {String(t.title ?? "Conversation")}
                      </span>
                    </span>
                    <span className="mono flex items-center gap-2 pl-4 text-xs text-ink-3">
                      {ms ? <span>{relTime(ms)}</span> : null}
                      {typeof t.messages === "number" && (
                        <span>{t.messages === 1 ? "1 msg" : `${t.messages} msgs`}</span>
                      )}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}

        {totalThreadPages > 1 && (
          <div className="mt-1 flex items-center justify-between px-2 py-1 text-xs text-ink-3">
            <Button
              type="button"
              size="xs"
              variant="ghost"
              disabled={safeThreadPage === 0}
              onClick={() => setThreadPage((p) => Math.max(0, p - 1))}
            >
              Prev
            </Button>
            <span className="mono">{safeThreadPage + 1} / {totalThreadPages}</span>
            <Button
              type="button"
              size="xs"
              variant="ghost"
              disabled={safeThreadPage >= totalThreadPages - 1}
              onClick={() => setThreadPage((p) => Math.min(totalThreadPages - 1, p + 1))}
            >
              Next
            </Button>
          </div>
        )}

        {/* Scheduled Routines Quick Drawer */}
        <section aria-label="Scheduled routines" className="mt-3 border-t border-hairline pt-2.5">
          <div className="flex items-center justify-between gap-1 px-1 py-1">
            <button
              type="button"
              onClick={() => setRoutinesExpanded((v) => !v)}
              className="focus-ring flex min-h-8 items-center gap-1.5 rounded-sm px-1.5 text-xs font-medium text-ink-2 hover:bg-board/60 hover:text-ink max-md:min-h-11"
            >
              <CalendarClock className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
              <span>Scheduled routines</span>
            </button>
            <button
              type="button"
              data-thing="routines-btn"
              onClick={() => {
                setMobileOpen(false);
                setRoutinesOpen(true);
              }}
              className="focus-ring min-h-8 rounded-sm px-2 text-xs text-ink-2 hover:bg-board hover:text-ink max-md:min-h-11"
            >
              Manage ({desk.routines.length})
            </button>
          </div>

          {routinesExpanded && (
            <div className="mt-1 flex flex-col gap-1 px-1">
              {desk.routines.length === 0 ? (
                <p className="px-2 py-1 text-xs text-ink-3">No routines scheduled yet.</p>
              ) : (
                desk.routines.slice(0, 3).map((rt: Routine) => (
                  <div
                    key={rt.id}
                    className="flex items-center justify-between gap-1.5 rounded-sm px-2 py-1.5 hover:bg-board/60"
                  >
                    <button
                      type="button"
                      onClick={() => {
                        setMobileOpen(false);
                        setRoutinesOpen(true);
                      }}
                      className="focus-ring flex min-w-0 flex-1 flex-col text-left"
                    >
                      <span className="flex items-center gap-1.5 truncate text-xs font-medium text-ink">
                        <span
                          aria-hidden
                          className={cn(
                            "size-1.5 shrink-0 rounded-full",
                            rt.enabled !== false ? "bg-dot-green" : "bg-ink-3"
                          )}
                        />
                        <span className="truncate">{rt.name}</span>
                      </span>
                      <span className="mono pl-3 text-xs text-ink-3">{rt.schedule}</span>
                    </button>
                    <Button
                      type="button"
                      size="icon-xs"
                      variant="ghost"
                      aria-label={`Run routine ${rt.name}`}
                      onClick={() => void desk.runRoutineNow(rt)}
                      className="shrink-0 max-md:min-h-11 max-md:min-w-11"
                    >
                      <Play className="size-3 text-icon" strokeWidth={1.75} aria-hidden />
                    </Button>
                  </div>
                ))
              )}
            </div>
          )}
        </section>
      </div>

      <div className="flex shrink-0 flex-col gap-0.5 border-t border-hairline p-2">
        <RailPanelButton
          id="skills-btn"
          label="Workflows"
          icon={<Sparkles className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
          count={desk.skills.length}
          onClick={() => {
            setMobileOpen(false);
            setSkillsOpen(true);
          }}
        />
        <RailPanelButton
          id="memory-btn"
          label="What my agent has learned"
          icon={<BookOpen className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
          count={desk.learnedCount}
          onClick={() => {
            setMobileOpen(false);
            setLearnedOpen(true);
          }}
        />
        <RailPanelButton
          id="suite-btn"
          label="My suite"
          icon={<Blocks className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
          count={desk.suiteCount}
          onClick={() => {
            setMobileOpen(false);
            setSuiteOpen(true);
          }}
        />
        <RailPanelButton
          id="add-person-btn"
          label="Add a person"
          icon={<UserPlus className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
          onClick={() => {
            setMobileOpen(false);
            setAddPersonOpen(true);
          }}
        />
        <RailPanelButton
          id="settings-btn"
          label="Settings"
          icon={<SlidersHorizontal className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
          onClick={() => {
            setMobileOpen(false);
            setSettingsOpen(true);
          }}
        />
        <div className="pt-1">
          <VersionRow u={u} />
        </div>
      </div>
    </div>
  );

  const handleSaveRoutineFromWork = useCallback(
    (
      name: string,
      prompt: string,
      schedule = "daily 09:00",
      playbook?: { steps?: string[]; expected_inputs?: string[]; pack_deliverables?: boolean }
    ) => desk.createRoutine({ name, prompt, schedule, ...playbook }),
    [desk.createRoutine]
  );

  const handleDismissUploaded = useCallback(() => desk.setLastUploaded([]), [desk.setLastUploaded]);
  const handleRemoveUploaded = useCallback(
    (path: string) => desk.setLastUploaded((prev) => prev.filter((p) => p !== path)),
    [desk.setLastUploaded]
  );

  return (
    <LazyMotion features={domAnimation} strict>
    <MotionConfig reducedMotion="user">
    <div className="flex h-dvh flex-col overflow-hidden bg-canvas text-ink">
      <Toaster
        position="bottom-right"
        theme={theme}
        toastOptions={{
          className: "!bg-surface !text-ink !border !border-hairline !shadow-lift",
        }}
      />
      <div id="announcer" role="status" aria-live="polite" className="sr-only" />

      <input
        ref={fileUploadRef}
        type="file"
        multiple
        className="hidden"
        accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html,.docx,.xlsx,.pdf"
        onChange={async (e) => {
          if (e.target.files?.length) await desk.uploadFiles(e.target.files);
          e.target.value = "";
        }}
      />

      {statusObj && !statusObj.model_configured && (
        <NoKeyBanner onOpenSettings={() => setSettingsOpen(true)} />
      )}
      {desk.recovered.length > 0 && (
        <RecoveredBanner files={desk.recovered} onDismiss={() => desk.setRecovered([])} />
      )}
      <UpdateStrip u={u} />

      {/* Slim Desktop-Workbench Top Bar */}
      <header className="safe-t flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-hairline bg-surface px-3 py-1.5 sm:h-12 sm:flex-nowrap sm:px-4 sm:py-0">
        <div className="flex min-w-0 items-center gap-2">
          <Button
            type="button"
            size="icon-sm"
            variant="ghost"
            aria-label="Open menu"
            onClick={() => setMobileOpen(true)}
            className="min-h-11 min-w-11 md:hidden"
          >
            <Menu className="size-4" strokeWidth={1.5} aria-hidden />
          </Button>
          <Button
            type="button"
            size="icon-sm"
            variant="ghost"
            aria-label="Toggle conversations rail"
            aria-expanded={railOpen}
            onClick={() => setRailOpen((v) => !v)}
            className="hidden md:inline-flex"
          >
            <PanelLeft className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          </Button>

          <PersonSelect
            people={peopleList}
            active={activePersonId}
            disabled={desk.busy}
            expanded={switcherOpen}
            onOpen={() => setSwitcherOpen(true)}
          />
        </div>

        <Tabs
          value={face}
          onValueChange={(v) => switchFace(v as Face)}
          className="order-3 w-full sm:order-none sm:w-auto"
        >
          <TabsList aria-label="Workspace view" className="w-full sm:w-fit">
            <TabsTrigger value="chat" data-id="chat" className="max-md:min-h-11 flex-1 sm:flex-initial">Chat</TabsTrigger>
            <TabsTrigger
              value="paper"
              data-id="paper"
              aria-label={pendingCount > 0 ? `Today's paper, ${pendingCount} waiting` : "Today's paper"}
              className="max-md:min-h-11 flex-1 sm:flex-initial"
            >
              <span>Today</span>
              {pendingCount > 0 && (
                <span className="mono ml-1 rounded-chip bg-status-waiting-bg px-1.5 py-0.5 text-xs font-medium text-status-waiting-fg">
                  {pendingCount}
                </span>
              )}
            </TabsTrigger>
            <TabsTrigger value="graph" data-id="graph" className="max-md:min-h-11 flex-1 sm:flex-initial">Work graph</TabsTrigger>
          </TabsList>
        </Tabs>

        <div className="flex shrink-0 items-center gap-1.5">
          <Button
            type="button"
            size="xs"
            variant="ghost"
            aria-label="Open command palette"
            onClick={() => setPaletteOpen(true)}
            className="hidden gap-1.5 sm:inline-flex"
          >
            <CommandIcon className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
            <span>Commands</span>
          </Button>
          <Button
            type="button"
            size="xs"
            variant={inspectorOpen ? "secondary" : "ghost"}
            data-thing="inspector-toggle"
            aria-label="Toggle workspace inspector"
            aria-expanded={inspectorOpen}
            onClick={() => {
              const next = !inspectorOpen;
              setInspectorOpen(next);
              if (next) desk.loadFiles();
            }}
            className="min-h-11 min-w-11 gap-1.5 md:min-h-8 md:min-w-0"
          >
            <FolderOpen className="size-3.5 text-icon sm:hidden" strokeWidth={1.5} aria-hidden />
            <PanelRight className="hidden size-3.5 text-icon sm:block" strokeWidth={1.5} aria-hidden />
            <span className="hidden sm:inline">Workspace</span>
            <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
              {desk.files.length}
            </span>
          </Button>
          <button
            id="ready-chip"
            type="button"
            onClick={() => {
              if (readinessState !== "Ready") setSettingsOpen(true);
            }}
            aria-label={`Agent status: ${readinessState}`}
            className="focus-ring hidden items-center gap-1.5 rounded-chip border border-hairline bg-board/50 px-2.5 py-1 text-xs text-ink-2 hover:bg-board lg:inline-flex"
          >
            <span
              aria-hidden
              className={cn(
                "size-2 rounded-full",
                readinessState === "Ready"
                  ? "bg-dot-online"
                  : readinessState === "Busy"
                    ? "pulse-dot bg-blue"
                    : "bg-dot-amber"
              )}
            />
            <span id="ready-text">{readinessState}</span>
          </button>
        </div>
      </header>

      {/* Mobile Left Drawer (<768px only) */}
      {!isMd && (
        <Drawer.Root open={mobileOpen} onOpenChange={setMobileOpen} direction="left">
          <Drawer.Portal>
            <Drawer.Overlay className="fixed inset-0 z-40 bg-ink/40" />
            <Drawer.Content
              aria-label="Ergon navigation drawer"
              className="fixed inset-y-0 left-0 z-40 flex w-[280px] max-w-[85vw] flex-col border-r border-hairline bg-surface text-ink outline-none"
            >
              <Drawer.Title className="sr-only">Ergon navigation</Drawer.Title>
              {renderRailContent()}
            </Drawer.Content>
          </Drawer.Portal>
        </Drawer.Root>
      )}

      {/* Main 3-Pane Workbench Body */}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        {isMd && railOpen && (
          <aside
            aria-label="Ergon navigation"
            className="flex w-[256px] shrink-0 flex-col border-r border-hairline bg-surface"
          >
            {renderRailContent()}
          </aside>
        )}

        <main className="min-w-0 flex-1 overflow-hidden bg-canvas">
          {face === "chat" && (
            <Chat
              threadId={desk.threadId}
              personName={personName}
              role={String(activePerson?.role ?? "")}
              messages={desk.messages}
              decisions={pendingDecisions}
              busy={desk.busy}
              cancelling={desk.cancelling}
              runStartedMs={desk.runStartedMs}
              liveElapsedMs={desk.liveElapsedMs}
              streamMode={desk.streamMode}
              liveSteps={desk.liveSteps}
              liveTimeline={desk.liveTimeline}
              draft={desk.draft}
              setDraft={desk.setDraft}
              pendingNote={desk.pendingNote}
              starters={desk.starters}
              fileCount={desk.files.length}
              lastUploaded={desk.lastUploaded}
              onDismissUploaded={handleDismissUploaded}
              onRemoveUploaded={handleRemoveUploaded}
              useServerExport={hasV050Api(desk.status, "receipts_export")}
              onSend={desk.send}
              onRunPrompt={(prompt) => {
                setFace("chat");
                desk.setDraft(prompt);
                requestAnimationFrame(() => inputRef.current?.focus());
              }}
              onCancel={desk.cancelRun}
              onReview={() => switchFace("paper")}
              onResolveDecision={handleResolveDecision}
              onOpenFile={(p) => setViewerPath(p)}
              onOpenFiles={openFilesInspector}
              onExportAuditBundle={handleExportAuditBundle}
              onUploadFiles={desk.uploadFiles}
              onSaveRoutine={handleSaveRoutineFromWork}
              inputRef={inputRef}
            />
          )}

          {face === "paper" && (
            <div id="face-today" aria-label="Today's paper" className="h-full overflow-y-auto px-4 py-6 sm:px-8">
              <Suspense fallback={null}>
              <PaperFace
                paper={paper}
                status={desk.status}
                personName={personName}
                dateLabel={dateLabel}
                people={desk.people ? desk.people.people : null}
                teamError={desk.sideError}
                activePerson={activePersonId}
                files={desk.files}
                busy={desk.busy}
                liveTimeline={desk.liveTimeline}
                liveElapsedMs={desk.liveElapsedMs}
                runStartedMs={desk.runStartedMs}
                streamMode={desk.streamMode}
                cancelling={desk.cancelling}
                onCancel={desk.cancelRun}
                onSwitchPerson={async (id) => {
                  await desk.switchPerson(id);
                  runAfterPersonChange();
                }}
                onAsk={focusComposer}
                onOpen={(id) => {
                  desk.openThread(id);
                  setFace("chat");
                }}
                onOpenFile={(p) => setViewerPath(p)}
                onOpenFilesPanel={openFilesInspector}
                onUploadMany={desk.uploadFiles}
                onExportAuditBundle={handleExportAuditBundle}
                onSaveRoutine={handleSaveRoutineFromWork}
              />
              </Suspense>
            </div>
          )}

          {face === "graph" && (
            <div id="face-graph" aria-label="Work graph" className="h-full overflow-y-auto px-4 py-6 sm:px-8">
              <Suspense fallback={null}>
              <GraphFace
                key={graphSeed}
                onOpenFile={(p) => setViewerPath(p)}
              />
              </Suspense>
            </div>
          )}
        </main>

        {/* Inline Right Inspector Drawer on >=1280px; Slide-over Drawer on <1280px */}
        <Suspense fallback={null}>
        <AnimatePresence>
          {isXl && inspectorOpen && (
            <InspectorDrawer
              key="desktop-inspector"
              open={inspectorOpen}
              tab={inspectorTab}
              onTabChange={setInspectorTab}
              onClose={() => setInspectorOpen(false)}
              files={desk.files}
              status={desk.status}
              personName={personName}
              offers={(paper.data?.offers ?? []).filter((o) => o.status === "pending")}
              selectedHistoryPath={selectedHistoryPath}
              onSelectHistoryPath={setSelectedHistoryPath}
              onRefreshFiles={desk.loadFiles}
              onOpenFile={(p) => setViewerPath(p)}
              onUploadFile={desk.uploadFile}
              onUploadMany={desk.uploadFiles}
              onExportAuditBundle={handleExportAuditBundle}
              onOfferAction={async (url, msg) => {
                await paper.act(url, msg);
              }}
            />
          )}
          {!isXl && inspectorOpen && (
            <m.div
              key="mobile-inspector"
              initial={reduceMotion ? false : { opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={reduceMotion ? undefined : { opacity: 0 }}
              transition={reduceMotion ? instant : sheet}
              className="fixed inset-0 z-40 flex justify-end bg-ink/40"
              onClick={(e) => {
                if (e.target === e.currentTarget) setInspectorOpen(false);
              }}
            >
              <div className="h-full w-full max-w-sm bg-surface shadow-lift">
                <InspectorDrawer
                  open={inspectorOpen}
                  tab={inspectorTab}
                  onTabChange={setInspectorTab}
                  onClose={() => setInspectorOpen(false)}
                  files={desk.files}
                  status={desk.status}
                  personName={personName}
                  offers={(paper.data?.offers ?? []).filter((o) => o.status === "pending")}
                  selectedHistoryPath={selectedHistoryPath}
                  onSelectHistoryPath={setSelectedHistoryPath}
                  onRefreshFiles={desk.loadFiles}
                  onOpenFile={(p) => setViewerPath(p)}
                  onUploadFile={desk.uploadFile}
                  onUploadMany={desk.uploadFiles}
                  onExportAuditBundle={handleExportAuditBundle}
                  onOfferAction={async (url, msg) => {
                    await paper.act(url, msg);
                  }}
                />
              </div>
            </m.div>
          )}
        </AnimatePresence>
        </Suspense>
      </div>

      {/* External Modals mounted OUTSIDE Vaul Drawer (z-50 portals) */}
      <Suspense fallback={null}>
      {paletteOpen && (
      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        status={desk.status}
        files={desk.files}
        receipts={paper.data?.brief.finished ?? []}
        threads={threadList}
        skills={desk.skills}
        routines={desk.routines}
        onNewTask={() => {
          desk.newThread();
          focusComposer();
        }}
        onUploadFile={() => fileUploadRef.current?.click()}
        onExportLatestReceipt={(fmt) => void handleExportLatestReceipt(fmt)}
        onExportAuditBundle={(fmt) => void handleExportAuditBundle(fmt)}
        onOpenWorkspaceSwitcher={() => setSwitcherOpen(true)}
        onOpenSettings={() => setSettingsOpen(true)}
        onOpenFiles={openFilesInspector}
        onOpenHistory={openHistoryInspector}
        onOpenSkills={() => setSkillsOpen(true)}
        onOpenRoutines={() => setRoutinesOpen(true)}
        onOpenFile={(p) => setViewerPath(p)}
        onOpenThread={(id) => {
          desk.openThread(id);
          setFace("chat");
        }}
        onRunSkill={(sk) => {
          setFace("chat");
          desk.setDraft(sk.sample_prompt || `Run ${sk.title}`);
          requestAnimationFrame(() => inputRef.current?.focus());
        }}
        onRunPrompt={(prompt) => {
          setFace("chat");
          desk.setDraft(prompt);
          requestAnimationFrame(() => inputRef.current?.focus());
        }}
        onRunRoutine={(rt) => {
          setFace("chat");
          void desk.runRoutineNow(rt);
        }}
        onSwitchFace={switchFace}
      />
      )}

      <WorkspaceSwitcherPanel
        people={peopleList}
        active={activePersonId}
        open={switcherOpen}
        onOpenChange={setSwitcherOpen}
        onRefresh={desk.loadPeople}
        onSwitch={async (id) => {
          await desk.switchPerson(id);
          runAfterPersonChange();
        }}
        onAddPerson={() => setAddPersonOpen(true)}
      />

      {addPersonOpen && (
      <AddPersonPanel
        open={addPersonOpen}
        onOpenChange={setAddPersonOpen}
        hideTrigger
        onAdded={async () => {
          await desk.refreshPerson();
          runAfterPersonChange();
        }}
      />
      )}

      {learnedOpen && (
      <LearnedPanel
        open={learnedOpen}
        onOpenChange={setLearnedOpen}
        hideTrigger
        count={desk.learnedCount}
      />
      )}

      {suiteOpen && (
      <SuitePanel
        open={suiteOpen}
        onOpenChange={setSuiteOpen}
        hideTrigger
        count={desk.suiteCount}
        onChanged={() => {
          desk.refreshCounts();
          desk.loadStarters(String(activePerson?.role ?? ""));
          paper.load();
          setGraphSeed((s) => s + 1);
        }}
        onRun={(id, msgs) => {
          setFace("chat");
          desk.showRun(id, msgs);
          desk.refreshCounts();
        }}
      />
      )}

      {skillsOpen && (
      <SkillsPanel
        skills={desk.skills}
        status={desk.status}
        open={skillsOpen}
        onOpenChange={setSkillsOpen}
        onRefresh={desk.loadSkills}
        onRunInToday={(sk) => {
          setFace("chat");
          desk.setDraft(sk.sample_prompt || `Run ${sk.title}`);
          requestAnimationFrame(() => inputRef.current?.focus());
        }}
      />
      )}

      {routinesOpen && (
      <RoutinesPanel
        routines={desk.routines}
        skills={desk.skills}
        status={desk.status}
        scheduler={desk.scheduler}
        historyRev={desk.routineHistoryRev}
        personName={personName}
        open={routinesOpen}
        onOpenChange={setRoutinesOpen}
        onRefresh={desk.loadRoutines}
        onCreate={desk.createRoutine}
        onToggleEnabled={desk.toggleRoutine}
        onLoadHistory={desk.loadRoutineHistory}
        onDelete={desk.deleteRoutine}
        onImportPlaybook={desk.importPlaybook}
        onPreviewPlaybook={desk.previewPlaybook}
        onRunNow={async (rt) => {
          setFace("chat");
          await desk.runRoutineNow(rt);
        }}
        onOpenThread={(id) => {
          desk.openThread(id);
          setFace("chat");
        }}
      />
      )}

      {settingsOpen && (
      <SettingsPanel
        theme={theme}
        onTheme={setTheme}
        status={desk.status}
        people={peopleList}
        activePerson={activePersonId}
        onRefreshPeople={desk.loadPeople}
        onSwitchPerson={async (id) => {
          await desk.switchPerson(id);
          runAfterPersonChange();
        }}
        onAddPerson={() => setAddPersonOpen(true)}
        onExportAuditBundle={handleExportAuditBundle}
        onConfigSaved={() => {
          desk.loadStatus();
          paper.load();
        }}
        u={u}
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        hideTrigger
      />
      )}

      {viewerPath && (
      <FileViewerModal
        filePath={viewerPath}
        status={desk.status}
        personName={personName}
        onClose={() => setViewerPath(null)}
        onAskAbout={(p) => {
          setViewerPath(null);
          setFace("chat");
          desk.setDraft(`Review ${p} and summarize key findings.`);
          requestAnimationFrame(() => inputRef.current?.focus());
        }}
        onRunWorkflow={(prompt) => {
          setViewerPath(null);
          setFace("chat");
          desk.setDraft(prompt);
          requestAnimationFrame(() => inputRef.current?.focus());
        }}
        onRestored={() => {
          desk.loadFiles();
          paper.load();
        }}
      />
      )}
      </Suspense>
    </div>
    </MotionConfig>
    </LazyMotion>
  );
}
export default App;
