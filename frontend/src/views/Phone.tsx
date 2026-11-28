import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { m, AnimatePresence } from "motion/react";
import { toast } from "sonner";
import {
  Bell,
  BookOpen,
  Check,
  Database,
  FileText,
  Layers,
  LogOut,
  MessageSquare,
  Monitor,
  Moon,
  Play,
  Plus,
  RefreshCw,
  Send,
  ShieldAlert,
  SlidersHorizontal,
  Sparkles,
  Sun,
  Trash2,
  Wrench,
  X,
} from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Pill } from "@/components/kibo-ui/pill";
import { PanelModal } from "@/shell/PanelModal";
import { QualityBarControls, matchQualityBar } from "@/shell/QualityBarControl";
import { ErgonMark } from "@/shell/ErgonLogo";
import {
  addCompanyMemory,
  analyzeOffers,
  call,
  deleteCompanyMemory,
  fetchCompanyMemory,
  fetchNotifications,
  fetchQualityBars,
  fetchTools,
  markNotificationRead,
  resolveOffer,
  runTool,
  subscribePush,
  type AuthMePayload,
  type Brief,
  type CompanyMemoryCategory,
  type CompanyMemoryEntry,
  type Decision,
  type LibraryItem,
  type Msg,
  type NotificationItem,
  type Offer,
  type Person,
  type QualityBar,
  type Status,
  type SuiteItem,
  type Thread,
  type ThreadRow,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import { relTime } from "@/lib/words";
import { glide, open as openAnim } from "@/motion";
import type { Theme } from "@/shell/SettingsPanel";

type PhoneTab = "decisions" | "brief" | "chat" | "suite";

type LiveBanner = {
  id: string;
  title: string;
  body: string;
  cardType?: string;
};

type RunTarget = {
  id: string;
  kind: string;
  name: string;
  description: string;
  params: string[];
};

const NAV_TABS: { id: PhoneTab; label: string; icon: typeof ShieldAlert }[] = [
  { id: "decisions", label: "Decisions", icon: ShieldAlert },
  { id: "brief", label: "Brief", icon: FileText },
  { id: "chat", label: "Chat", icon: MessageSquare },
  { id: "suite", label: "Suite", icon: Layers },
];

const DECISION_LABELS: Record<string, string> = {
  tool_offer: "Tool offer",
  delete: "Delete file",
  connector_delivery: "Outbound delivery",
  mcp_tool_call: "MCP tool",
  budget_gate: "Budget gate",
  allow_host: "Network host",
  security: "Security",
  security_notice: "Security notice",
  memory_note: "Memory note",
  publish_skill: "Library publish",
  question: "Question",
};

function toolOfferTestSummary(d: Decision): string {
  const passed = d.test_report?.passed;
  const total = d.test_report?.total;
  if (typeof passed === "number" && typeof total === "number") {
    return `${passed}/${total} sandbox tests passed`;
  }
  const match = typeof d.detail === "string" ? d.detail.match(/(\d+\/\d+)\s+passing sandbox tests/i) : null;
  if (match) return `${match[1]} sandbox tests passed`;
  return "Sandbox tests passed";
}

function decisionSummaryDetail(d: Decision): string {
  if (d.kind === "tool_offer") {
    return d.description || d.detail || "Verified in an isolated sandbox. Approve to install in your suite and share with the company library.";
  }
  if (d.detail && d.detail.trim()) return d.detail.trim();
  if (d.kind === "delete") return d.path ? `Approving deletes ${d.path} permanently. Denying keeps the file safe.` : "Approving deletes the file permanently.";
  if (d.kind === "connector_delivery") {
    const dest = d.url || d.target_url || d.connector_name || "the external destination";
    return `Approving sends the payload to ${dest}. Denying blocks delivery.`;
  }
  if (d.kind === "mcp_tool_call") {
    return `Approving runs ${d.tool_name || "the requested tool"} on ${d.server_name || d.server_id || "the MCP server"}.`;
  }
  if (d.kind === "budget_gate") {
    return `Engagement spend reached its budget limit for ${d.engagement_name || "this engagement"}.`;
  }
  return "Your agent paused and needs your decision before continuing.";
}

function fireBrowserNotification(title: string, body: string) {
  try {
    if (typeof Notification !== "undefined" && Notification.permission === "granted") {
      new Notification(title, { body });
    }
  } catch {
    // Browser notifications may be unavailable in headless or restricted contexts.
  }
}

export function PhoneFace({
  auth,
  theme,
  setTheme,
  status,
  people,
  activePersonId,
  onSignOut,
}: {
  auth: AuthMePayload | null;
  theme: Theme;
  setTheme: (t: Theme) => void;
  status: Status | null | "error";
  people: Person[];
  activePersonId: string;
  onSignOut: () => Promise<void> | void;
}) {
  const [tab, setTab] = useState<PhoneTab>("decisions");
  const [decisions, setDecisions] = useState<Decision[]>([]);
  const [offers, setOffers] = useState<Offer[]>([]);
  const [qualityBars, setQualityBars] = useState<QualityBar[]>([]);
  const [brief, setBrief] = useState<Brief | null>(null);
  const [notifications, setNotifications] = useState<NotificationItem[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [threads, setThreads] = useState<ThreadRow[]>([]);
  const [activeThreadId, setActiveThreadId] = useState<string>("");
  const [threadMessages, setThreadMessages] = useState<Msg[]>([]);
  const [chatDraft, setChatDraft] = useState("");
  const [chatSending, setChatSending] = useState(false);

  const [suiteItems, setSuiteItems] = useState<SuiteItem[]>([]);
  const [libraryItems, setLibraryItems] = useState<LibraryItem[]>([]);
  const [toolsList, setToolsList] = useState<LibraryItem[]>([]);
  const [companyMemory, setCompanyMemory] = useState<CompanyMemoryEntry[]>([]);
  const [memCategory, setMemCategory] = useState<CompanyMemoryCategory>("fact");
  const [memScope, setMemScope] = useState("company");
  const [memContent, setMemContent] = useState("");
  const [savingMem, setSavingMem] = useState(false);

  const [sseState, setSseState] = useState<"connecting" | "live" | "offline">("connecting");
  const [liveBanner, setLiveBanner] = useState<LiveBanner | null>(null);
  const [pushSubscribed, setPushSubscribed] = useState(false);
  const [busyDecisionId, setBusyDecisionId] = useState<string | null>(null);
  const [busyOfferId, setBusyOfferId] = useState<string | null>(null);
  const [scanningOffers, setScanningOffers] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [runTarget, setRunTarget] = useState<RunTarget | null>(null);
  const [runInputs, setRunInputs] = useState<Record<string, string>>({});
  const [runningTool, setRunningTool] = useState(false);
  const [runOutput, setRunOutput] = useState<{ ok: boolean; text: string } | null>(null);

  const mountedRef = useRef(true);
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const effectiveQualityBars = useMemo(() => {
    const map = new Map<string, QualityBar>();
    for (const b of brief?.quality_bars ?? []) map.set(b.task_family, b);
    for (const b of qualityBars) map.set(b.task_family, b);
    return Array.from(map.values());
  }, [brief?.quality_bars, qualityBars]);

  const activePerson = useMemo(
    () => people.find((p) => p.id === activePersonId) ?? people[0] ?? null,
    [people, activePersonId]
  );
  const employeeName = auth?.user?.display_name || activePerson?.name || "Workspace";
  const employeeRole = auth?.user?.role || activePerson?.role || "member";
  const canWriteCompanyMemory =
    !auth?.auth_required || ["admin", "pm", "ae", "office"].includes(auth?.user?.role ?? "");

  const loadQualityBars = useCallback(async () => {
    const r = await fetchQualityBars();
    if (mountedRef.current && r.ok) {
      setQualityBars(r.data.quality_bars);
    }
  }, []);

  const loadDecisionsAndOffers = useCallback(async () => {
    const [d, o, n, q] = await Promise.all([
      call<{ decisions?: Decision[] }>("/api/decisions"),
      call<{ offers?: Offer[] }>("/api/offers"),
      fetchNotifications(),
      fetchQualityBars(),
    ]);
    if (!mountedRef.current) return;
    if (d.ok && Array.isArray(d.data.decisions)) {
      setDecisions(d.data.decisions.filter((x): x is Decision => !!x && typeof x === "object"));
    }
    if (o.ok && Array.isArray(o.data.offers)) {
      setOffers(o.data.offers.filter((x): x is Offer => !!x && typeof x === "object"));
    }
    if (n.ok) {
      setNotifications(n.data.notifications);
      setUnreadCount(n.data.unread_count);
    }
    if (q.ok) {
      setQualityBars(q.data.quality_bars);
    }
  }, []);

  const loadBrief = useCallback(async () => {
    const r = await call<Brief>("/api/brief");
    if (mountedRef.current && r.ok && r.data) {
      setBrief(r.data);
    }
  }, []);

  const handleQualityUpdated = useCallback(
    async (updated?: QualityBar) => {
      if (updated) {
        setQualityBars((prev) => {
          const exists = prev.some((b) => b.task_family === updated.task_family);
          return exists
            ? prev.map((b) => (b.task_family === updated.task_family ? updated : b))
            : [updated, ...prev];
        });
      }
      await Promise.all([loadQualityBars(), loadBrief()]);
    },
    [loadQualityBars, loadBrief]
  );

  const loadThreads = useCallback(async () => {
    const r = await call<{ threads?: ThreadRow[] }>("/api/threads");
    if (!mountedRef.current || !r.ok) return;
    const list = Array.isArray(r.data.threads) ? r.data.threads : [];
    setThreads(list);
    setActiveThreadId((prev) => (prev && list.some((t) => t.id === prev) ? prev : list[0]?.id ?? ""));
  }, []);

  const loadThreadMessages = useCallback(async (tid: string) => {
    if (!tid) {
      setThreadMessages([]);
      return;
    }
    const r = await call<Thread>(`/api/threads/${encodeURIComponent(tid)}`);
    if (mountedRef.current && r.ok && Array.isArray(r.data.messages)) {
      setThreadMessages(r.data.messages);
    }
  }, []);

  const loadSuiteAndMemory = useCallback(async () => {
    const [s, l, t, cm] = await Promise.all([
      call<{ installed?: SuiteItem[] }>("/api/suite"),
      call<{ items?: LibraryItem[] }>("/api/library"),
      fetchTools(),
      fetchCompanyMemory(),
    ]);
    if (!mountedRef.current) return;
    if (s.ok && Array.isArray(s.data.installed)) setSuiteItems(s.data.installed);
    if (l.ok && Array.isArray(l.data.items)) setLibraryItems(l.data.items);
    if (t.ok) setToolsList(t.data.tools);
    if (cm.ok) setCompanyMemory(cm.data.entries);
  }, []);

  const refreshAll = useCallback(async () => {
    await Promise.all([loadDecisionsAndOffers(), loadBrief(), loadThreads(), loadSuiteAndMemory()]);
  }, [loadDecisionsAndOffers, loadBrief, loadThreads, loadSuiteAndMemory]);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  useEffect(() => {
    if (activeThreadId) {
      void loadThreadMessages(activeThreadId);
    }
  }, [activeThreadId, loadThreadMessages]);

  // Live SSE listener for push_notification, approval_parked, and run_finished
  useEffect(() => {
    if (typeof EventSource === "undefined") {
      setSseState("offline");
      return;
    }
    setSseState("connecting");
    const es = new EventSource("/api/runs/stream");

    es.onopen = () => {
      if (mountedRef.current) setSseState("live");
    };
    es.onerror = () => {
      if (mountedRef.current) setSseState("offline");
    };

    const handlePush = (ev: MessageEvent<string>) => {
      if (!mountedRef.current) return;
      try {
        const parsed = JSON.parse(ev.data) as {
          id?: string;
          title?: string;
          body?: string;
          card_type?: string;
          unread_count?: number;
        };
        const title = parsed.title || "New alert";
        const body = parsed.body || "Check Decisions for details.";
        setLiveBanner({
          id: parsed.id || String(Date.now()),
          title,
          body,
          cardType: parsed.card_type,
        });
        if (typeof parsed.unread_count === "number") {
          setUnreadCount(parsed.unread_count);
        }
        fireBrowserNotification(title, body);
      } catch {
        // Ignore malformed SSE payload
      }
      void loadDecisionsAndOffers();
    };

    const handleParked = (ev: MessageEvent<string>) => {
      if (!mountedRef.current) return;
      let summary = "A decision needs your approval.";
      try {
        const parsed = JSON.parse(ev.data) as { summary?: string };
        if (parsed.summary) summary = parsed.summary;
      } catch {
        // Ignore
      }
      setLiveBanner({ id: `park-${Date.now()}`, title: "Decision waiting", body: summary, cardType: "decision" });
      fireBrowserNotification("Decision waiting", summary);
      void loadDecisionsAndOffers();
    };

    const handleFinished = (ev: MessageEvent<string>) => {
      if (!mountedRef.current) return;
      let task = "Background work finished.";
      try {
        const parsed = JSON.parse(ev.data) as { task?: string; summary?: string };
        if (parsed.summary || parsed.task) task = parsed.summary || parsed.task || task;
      } catch {
        // Ignore
      }
      setLiveBanner({ id: `fin-${Date.now()}`, title: "Work finished", body: task, cardType: "brief" });
      fireBrowserNotification("Work finished", task);
      void loadBrief();
      void loadDecisionsAndOffers();
    };

    es.addEventListener("push_notification", handlePush as EventListener);
    es.addEventListener("approval_parked", handleParked as EventListener);
    es.addEventListener("run_finished", handleFinished as EventListener);

    return () => {
      es.removeEventListener("push_notification", handlePush as EventListener);
      es.removeEventListener("approval_parked", handleParked as EventListener);
      es.removeEventListener("run_finished", handleFinished as EventListener);
      es.close();
    };
  }, [loadBrief, loadDecisionsAndOffers]);

  const pendingDecisions = useMemo(
    () => decisions.filter((d) => d.status === "pending"),
    [decisions]
  );
  const pendingOffers = useMemo(
    () => offers.filter((o) => o.status === "pending"),
    [offers]
  );
  const unreadNotifications = useMemo(
    () => notifications.filter((n) => !n.read),
    [notifications]
  );
  const actionCount = pendingDecisions.length + pendingOffers.length;

  const handleEnablePush = async () => {
    try {
      if (typeof Notification !== "undefined" && typeof Notification.requestPermission === "function") {
        await Notification.requestPermission();
      }
    } catch {
      // Headless or denied notification permissions must not block subscription
    }
    const r = await subscribePush({
      channel: "web_push",
      endpoint: "browser://local",
      label: "Phone Browser",
    });
    if (!r.ok) {
      toast.error(r.error || "Could not enable push alerts.");
      return;
    }
    setPushSubscribed(true);
    toast.success("Phone push alerts enabled.");
  };

  const handleDecision = async (id: string, action: "approve" | "deny") => {
    setBusyDecisionId(id);
    const r = await call<{ outcome?: string }>(`/api/decisions/${encodeURIComponent(id)}/${action}`, {
      method: "POST",
      body: {},
    });
    setBusyDecisionId(null);
    if (!r.ok) {
      toast.error(r.error || "Could not update decision.");
      return;
    }
    toast.success(r.data.outcome || (action === "approve" ? "Approved." : "Denied."));
    await Promise.all([loadDecisionsAndOffers(), loadSuiteAndMemory()]);
  };

  const handleOffer = async (id: string, action: "approve" | "deny") => {
    setBusyOfferId(id);
    const r = await resolveOffer(id, action);
    setBusyOfferId(null);
    if (!r.ok) {
      toast.error(r.error || "Could not update offer.");
      return;
    }
    toast.success(r.data.outcome || (action === "approve" ? "Offer approved." : "Offer dismissed."));
    await Promise.all([loadDecisionsAndOffers(), loadSuiteAndMemory()]);
  };

  const handleScanOffers = async () => {
    setScanningOffers(true);
    const r = await analyzeOffers();
    setScanningOffers(false);
    if (!r.ok) {
      toast.error(r.error || "Could not scan recent work.");
      return;
    }
    const count = r.data.generated.length + (r.data.staged_package ? 1 : 0);
    toast.success(
      count > 0
        ? `Found ${count} new ${count === 1 ? "suggestion" : "suggestions"}.`
        : "No new suggestions right now."
    );
    await loadDecisionsAndOffers();
  };

  const handleMarkNotificationRead = async (id: string) => {
    const r = await markNotificationRead(id);
    if (!r.ok) {
      toast.error(r.error || "Could not mark notification read.");
      return;
    }
    await loadDecisionsAndOffers();
  };

  const handleSendChat = async (e: React.FormEvent) => {
    e.preventDefault();
    const msg = chatDraft.trim();
    if (!msg || chatSending) return;
    setChatSending(true);
    let tid = activeThreadId;
    if (!tid) {
      const created = await call<ThreadRow>("/api/threads", { method: "POST", body: {} });
      if (created.ok && created.data?.id) {
        tid = created.data.id;
        setActiveThreadId(tid);
      }
    }
    setThreadMessages((prev) => [...prev, { role: "user", content: msg }]);
    setChatDraft("");
    const r = await call<{ reply?: string; thread_id?: string }>("/api/chat", {
      method: "POST",
      body: { message: msg, thread_id: tid || undefined },
    });
    setChatSending(false);
    if (!r.ok) {
      toast.error(r.error || "Message failed.");
      return;
    }
    const targetTid = r.data.thread_id || tid;
    if (targetTid) {
      setActiveThreadId(targetTid);
      await Promise.all([loadThreads(), loadThreadMessages(targetTid), loadDecisionsAndOffers()]);
    }
  };

  const handleNewChat = async () => {
    const r = await call<ThreadRow>("/api/threads", { method: "POST", body: {} });
    if (!r.ok) {
      toast.error(r.error || "Could not create chat.");
      return;
    }
    if (!r.data?.id) {
      toast.error("Could not create chat.");
      return;
    }
    setActiveThreadId(r.data.id);
    setThreadMessages([]);
    await loadThreads();
  };

  const handleInstallItem = async (item: LibraryItem) => {
    const r = await call("/api/suite/install", { method: "POST", body: { item_id: item.id } });
    if (!r.ok) {
      toast.error(r.error || "Could not install item.");
      return;
    }
    toast.success(`Installed ${item.name}.`);
    await loadSuiteAndMemory();
  };

  const openRunDialog = (target: RunTarget) => {
    setRunTarget(target);
    setRunOutput(null);
    const initial: Record<string, string> = {};
    for (const p of target.params) initial[p] = "";
    setRunInputs(initial);
  };

  const handleExecuteRunTarget = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!runTarget) return;
    setRunningTool(true);
    setRunOutput(null);
    if (runTarget.kind === "tool") {
      const args = runTarget.params.map((p) => (runInputs[p] ?? "").trim());
      const r = await runTool(runTarget.id, args);
      setRunningTool(false);
      if (!r.ok) {
        setRunOutput({ ok: false, text: r.error || "Tool run failed." });
        return;
      }
      setRunOutput({
        ok: r.data.ok !== false,
        text: r.data.output || r.data.error || "Tool finished.",
      });
      return;
    }
    const r = await call<{ reply?: string }>(`/api/apps/${encodeURIComponent(runTarget.id)}/run`, {
      method: "POST",
      body: { fields: runInputs },
    });
    setRunningTool(false);
    if (!r.ok) {
      setRunOutput({ ok: false, text: r.error || "Could not run item." });
      return;
    }
    setRunOutput({ ok: true, text: r.data.reply || "Finished." });
    await loadBrief();
  };

  const handleAddCompanyMemory = async (e: React.FormEvent) => {
    e.preventDefault();
    const content = memContent.trim();
    if (!content) return;
    setSavingMem(true);
    const r = await addCompanyMemory({
      category: memCategory,
      scope: memScope.trim() || "company",
      content,
    });
    setSavingMem(false);
    if (!r.ok) {
      toast.error(r.error || "Could not save company memory.");
      return;
    }
    setMemContent("");
    toast.success("Saved to shared company memory.");
    await loadSuiteAndMemory();
  };

  const handleDeleteCompanyMemory = async (id: string) => {
    const r = await deleteCompanyMemory(id);
    if (!r.ok) {
      toast.error(r.error || "Could not delete entry.");
      return;
    }
    toast.success("Removed company memory entry.");
    await loadSuiteAndMemory();
  };

  const installedIds = useMemo(() => new Set(suiteItems.map((s) => s.id)), [suiteItems]);
  const availableLibrary = useMemo(
    () => libraryItems.filter((item) => !item.builtin && !installedIds.has(item.id)),
    [libraryItems, installedIds]
  );

  const toolsById = useMemo(() => {
    const map = new Map<string, LibraryItem>();
    for (const t of toolsList) map.set(t.id, t);
    for (const l of libraryItems) if (!map.has(l.id)) map.set(l.id, l);
    return map;
  }, [toolsList, libraryItems]);

  return (
    <div id="phone-shell" className="min-h-dvh bg-canvas text-ink pb-24">
      {/* Sticky Mobile Top Header */}
      <header className="sticky top-0 z-30 border-b border-hairline bg-surface">
        <div className="mx-auto flex max-w-lg flex-wrap items-center justify-between gap-2 px-3 py-2">
          <div className="flex min-w-0 items-center gap-2">
            <ErgonMark className="size-4 shrink-0 text-ink" />
            <span className="truncate text-sm font-medium text-ink">{employeeName}</span>
            <Pill variant="secondary" className="bg-track px-2 py-0.5 text-xs font-medium text-ink">
              {employeeRole}
            </Pill>
            <span
              data-phone-sse={sseState}
              className="inline-flex items-center gap-1.5 text-xs text-ink-2"
              title={sseState === "live" ? "Live stream connected" : "Reconnecting"}
            >
              <span
                aria-hidden
                className={cn(
                  "size-2 rounded-full",
                  sseState === "live" ? "bg-dot-online" : "bg-dot-amber"
                )}
              />
              <span>{sseState === "live" ? "Live" : "Sync"}</span>
            </span>
          </div>

          <div className="flex shrink-0 items-center gap-1.5">
            <Button
              id="phone-push-btn"
              type="button"
              size="sm"
              variant={pushSubscribed ? "secondary" : "outline"}
              className="min-h-11 gap-1.5 px-2.5"
              onClick={() => void handleEnablePush()}
            >
              <Bell className="size-4" strokeWidth={1.5} aria-hidden />
              <span>{pushSubscribed ? "Alerts on" : "Alerts"}</span>
              {unreadCount > 0 && (
                <span
                  id="phone-unread-badge"
                  className="mono rounded-chip bg-pill-warn-bg px-1.5 py-0.5 text-xs font-medium text-pill-warn-fg"
                >
                  {unreadCount}
                </span>
              )}
            </Button>

            <Button
              id="phone-settings-btn"
              type="button"
              size="icon"
              variant="ghost"
              aria-label="Settings"
              className="min-h-11 min-w-11"
              onClick={() => setSettingsOpen(true)}
            >
              <SlidersHorizontal className="size-4" strokeWidth={1.5} aria-hidden />
            </Button>

            <a
              id="phone-desktop-link"
              href="/"
              className={cn(buttonVariants({ size: "sm", variant: "ghost" }), "min-h-11 gap-1.5 px-2.5")}
            >
              <Monitor className="size-4" strokeWidth={1.5} aria-hidden />
              <span>Desktop</span>
            </a>
          </div>
        </div>
      </header>

      {/* Main Single-Column Container */}
      <main className="mx-auto flex w-full max-w-lg flex-col gap-4 px-3 pt-4">
        <AnimatePresence>
          {liveBanner && (
            <m.div
              id="phone-live-banner"
              role="status"
              initial={{ opacity: 0, y: -6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={openAnim}
              className="flex items-start justify-between gap-3 rounded-card border border-hairline bg-blue-soft p-3.5"
            >
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium text-ink">{liveBanner.title}</p>
                <p className="mt-0.5 text-sm text-ink-2 [overflow-wrap:anywhere]">{liveBanner.body}</p>
              </div>
              <Button
                type="button"
                size="icon"
                variant="ghost"
                aria-label="Dismiss alert"
                className="min-h-11 min-w-11 shrink-0"
                onClick={() => setLiveBanner(null)}
              >
                <X className="size-4" strokeWidth={1.5} aria-hidden />
              </Button>
            </m.div>
          )}
        </AnimatePresence>

        {/* TAB 1: DECISIONS (DEFAULT) */}
        {tab === "decisions" && (
          <div id="phone-tab-decisions" className="flex flex-col gap-4">
            <div className="flex items-center justify-between gap-2">
              <h1 className="text-title font-medium text-ink">Decisions</h1>
              <Button
                type="button"
                size="sm"
                variant="ghost"
                className="min-h-11 gap-1.5"
                onClick={() => void loadDecisionsAndOffers()}
              >
                <RefreshCw className="size-4" strokeWidth={1.5} aria-hidden />
                <span>Refresh</span>
              </Button>
            </div>

            {unreadNotifications.length > 0 && (
              <section id="phone-notifications-list" className="flex flex-col gap-2 rounded-card border border-hairline bg-surface p-3.5">
                <div className="flex items-center justify-between gap-2">
                  <h2 className="text-sm font-medium text-ink">Unread alerts</h2>
                  <span className="mono rounded-chip bg-pill-warn-bg px-2 py-0.5 text-xs font-medium text-pill-warn-fg">
                    {unreadNotifications.length}
                  </span>
                </div>
                <div className="flex flex-col divide-y divide-hairline">
                  {unreadNotifications.slice(0, 5).map((n) => (
                    <div key={n.id} className="flex items-start justify-between gap-2 py-2.5 first:pt-1 last:pb-1">
                      <div className="min-w-0 flex-1">
                        <p className="text-sm font-medium text-ink [overflow-wrap:anywhere]">{n.title}</p>
                        <p className="text-xs text-ink-2 [overflow-wrap:anywhere]">{n.body}</p>
                      </div>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="min-h-11 shrink-0"
                        onClick={() => void handleMarkNotificationRead(n.id)}
                      >
                        Mark read
                      </Button>
                    </div>
                  ))}
                </div>
              </section>
            )}

            <section id="phone-decisions-list" aria-label="Pending decisions" className="flex flex-col gap-3">
              {pendingDecisions.length === 0 ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  Nothing waits on you right now. Approvals, safety gates, and tool offers will appear here.
                </div>
              ) : (
                pendingDecisions.map((d) => {
                  const isToolOffer = d.kind === "tool_offer";
                  const kindLabel = DECISION_LABELS[d.kind] || d.kind;
                  const busy = busyDecisionId === d.id;
                  return (
                    <article
                      key={d.id}
                      data-phone-decision={d.id}
                      data-decision-kind={d.kind}
                      className={cn(
                        "flex flex-col gap-3 rounded-card border border-hairline p-4",
                        isToolOffer ? "bg-blue-soft" : "bg-surface"
                      )}
                    >
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <Badge variant="outline" className="text-xs">
                          {isToolOffer ? "Would this help?" : kindLabel}
                        </Badge>
                        {typeof d.ts === "number" && (
                          <span className="mono text-xs text-ink-2">{relTime(d.ts)}</span>
                        )}
                      </div>

                      <h2 className="text-base font-medium text-ink [overflow-wrap:anywhere]">
                        {d.summary}
                      </h2>

                      {isToolOffer && (
                        <div className="flex flex-wrap items-center gap-2">
                          <Pill variant="secondary" className="bg-pill-active-bg px-2.5 py-0.5 text-xs font-medium text-pill-active-fg">
                            {toolOfferTestSummary(d)}
                          </Pill>
                          {Array.isArray(d.params) && d.params.length > 0 && (
                            <span className="mono text-xs text-ink-2">
                              Params: {d.params.join(", ")}
                            </span>
                          )}
                        </div>
                      )}

                      <p className="text-sm leading-5 text-ink-2 [overflow-wrap:anywhere]">
                        {decisionSummaryDetail(d)}
                      </p>

                      {(d.url || d.target_url || d.tool_name || d.path) && !isToolOffer && (
                        <p className="mono rounded-sm bg-board px-2.5 py-1.5 text-xs text-ink [overflow-wrap:anywhere]">
                          {d.path || d.url || d.target_url || d.tool_name}
                        </p>
                      )}

                      <div className="flex flex-wrap gap-2 pt-1">
                        <Button
                          type="button"
                          className="min-h-11 flex-1"
                          disabled={busy}
                          data-phone-decision-approve={d.id}
                          onClick={() => void handleDecision(d.id, "approve")}
                        >
                          <Check className="size-4" strokeWidth={1.5} aria-hidden />
                          <span>Approve</span>
                        </Button>
                        {d.kind !== "security_notice" && (
                          <Button
                            type="button"
                            variant="outline"
                            className="min-h-11 flex-1"
                            disabled={busy}
                            data-phone-decision-deny={d.id}
                            onClick={() => void handleDecision(d.id, "deny")}
                          >
                            <X className="size-4" strokeWidth={1.5} aria-hidden />
                            <span>Deny</span>
                          </Button>
                        )}
                      </div>
                    </article>
                  );
                })
              )}
            </section>

            {/* Proactive Offers */}
            <section id="phone-offers-list" aria-labelledby="phone-offers-h" className="flex flex-col gap-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <Sparkles className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
                  <h2 id="phone-offers-h" className="text-base font-medium text-ink">
                    Would this help?
                  </h2>
                </div>
                <Button
                  id="phone-scan-offers-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={scanningOffers}
                  className="min-h-11"
                  onClick={() => void handleScanOffers()}
                >
                  {scanningOffers ? "Scanning…" : "Scan work for offers"}
                </Button>
              </div>

              {pendingOffers.length === 0 ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  No proactive offers right now. Tap Scan work for offers to analyze recent tasks.
                </div>
              ) : (
                pendingOffers.map((o) => {
                  const busy = busyOfferId === o.id;
                  return (
                    <article
                      key={o.id}
                      data-phone-offer={o.id}
                      className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4"
                    >
                      <div className="flex items-center justify-between gap-2">
                        <Pill variant="secondary" className="bg-tag-plan-bg px-2 py-0.5 text-xs font-medium text-tag-plan-fg">
                          Would this help?
                        </Pill>
                        {o.author && <span className="mono text-xs text-ink-2">by {o.author}</span>}
                      </div>
                      <h3 className="text-base font-medium text-ink [overflow-wrap:anywhere]">{o.title}</h3>
                      <p className="text-sm leading-5 text-ink-2 [overflow-wrap:anywhere]">{o.detail}</p>
                      <div className="flex flex-wrap gap-2 pt-1">
                        <Button
                          type="button"
                          className="min-h-11 flex-1"
                          disabled={busy}
                          data-phone-offer-approve={o.id}
                          onClick={() => void handleOffer(o.id, "approve")}
                        >
                          Approve
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          className="min-h-11 flex-1"
                          disabled={busy}
                          data-phone-offer-deny={o.id}
                          onClick={() => void handleOffer(o.id, "deny")}
                        >
                          Deny
                        </Button>
                      </div>
                    </article>
                  );
                })
              )}
            </section>

            {effectiveQualityBars.length > 0 && (
              <section id="phone-quality-check" aria-labelledby="phone-quality-h" className="flex flex-col gap-3">
                <div className="flex items-center justify-between gap-2">
                  <h2 id="phone-quality-h" className="text-base font-medium text-ink">
                    Quality check
                  </h2>
                  <span className="mono text-xs text-ink-2">
                    {effectiveQualityBars.length}
                  </span>
                </div>
                {effectiveQualityBars.map((bar) => (
                  <article
                    key={bar.task_family}
                    data-phone-quality-card={bar.task_family}
                    className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4"
                  >
                    <h3 className="text-base font-medium text-ink [overflow-wrap:anywhere]">
                      {bar.label}
                    </h3>
                    <QualityBarControls
                      bar={bar}
                      mobile
                      onUpdated={handleQualityUpdated}
                    />
                  </article>
                ))}
              </section>
            )}
          </div>
        )}

        {/* TAB 2: BRIEF */}
        {tab === "brief" && (
          <div id="phone-tab-brief" className="flex flex-col gap-4">
            <div className="flex items-center justify-between gap-2">
              <h1 className="text-title font-medium text-ink">Morning brief</h1>
              {brief?.date && <span className="mono text-xs text-ink-2">{brief.date}</span>}
            </div>

            <section className="rounded-card border border-hairline bg-surface p-4">
              <p id="phone-brief-headline" className="text-base font-medium leading-6 text-ink">
                {brief?.headline || "Loading morning brief…"}
              </p>
            </section>

            <section className="flex flex-col gap-2.5">
              <h2 className="text-sm font-medium text-ink">
                Recently finished ({brief?.finished?.length ?? 0})
              </h2>
              {!brief?.finished?.length ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  No finished jobs yet today.
                </div>
              ) : (
                (() => {
                  const shownFamilies = new Set<string>();
                  return brief.finished.slice(0, 12).map((job, idx) => {
                    const bar = matchQualityBar(job, effectiveQualityBars);
                    const showControls = bar && !shownFamilies.has(bar.task_family);
                    if (bar) shownFamilies.add(bar.task_family);
                    return (
                      <div key={job.id || `${job.task}-${idx}`} className="flex flex-col gap-1.5 rounded-card border border-hairline bg-surface p-3.5">
                        <div className="flex items-start justify-between gap-2">
                          <p className="text-sm font-medium text-ink [overflow-wrap:anywhere]">{job.task}</p>
                          {typeof job.ts === "number" && (
                            <span className="mono shrink-0 text-xs text-ink-2">{relTime(job.ts)}</span>
                          )}
                        </div>
                        {job.summary && <p className="text-sm text-ink-2 [overflow-wrap:anywhere]">{job.summary}</p>}
                        {job.files && job.files.length > 0 && (
                          <span className="mono text-xs text-ink-2">
                            {job.files.length} {job.files.length === 1 ? "file" : "files"}
                          </span>
                        )}
                        {showControls && bar && (
                          <div className="mt-1 border-t border-hairline pt-2">
                            <QualityBarControls
                              bar={bar}
                              mobile
                              compact
                              onUpdated={handleQualityUpdated}
                            />
                          </div>
                        )}
                      </div>
                    );
                  });
                })()
              )}
            </section>

            {(brief?.stopped?.length ?? 0) > 0 && (
              <section className="flex flex-col gap-2.5">
                <h2 className="text-sm font-medium text-ink">
                  Stopped jobs ({brief!.stopped!.length})
                </h2>
                {brief!.stopped!.map((job, idx) => (
                  <div key={job.id || `stop-${idx}`} className="flex flex-col gap-1 rounded-card border border-hairline bg-surface p-3.5">
                    <p className="text-sm font-medium text-ink [overflow-wrap:anywhere]">{job.task}</p>
                    {job.summary && <p className="text-sm text-ink-2 [overflow-wrap:anywhere]">{job.summary}</p>}
                  </div>
                ))}
              </section>
            )}

            <section className="flex flex-col gap-2 rounded-card border border-hairline bg-surface p-4">
              <h2 className="text-sm font-medium text-ink">What was learned</h2>
              {!brief?.learned_recently?.length ? (
                <p className="text-sm text-ink-2">No new standing notes yet.</p>
              ) : (
                <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm text-ink">
                  {brief.learned_recently.map((item, i) => (
                    <li key={i} className="[overflow-wrap:anywhere]">{item}</li>
                  ))}
                </ul>
              )}
            </section>
          </div>
        )}

        {/* TAB 3: CHAT */}
        {tab === "chat" && (
          <div id="phone-tab-chat" className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h1 className="text-title font-medium text-ink">Chat</h1>
              <Button type="button" size="sm" variant="outline" className="min-h-11 gap-1.5" onClick={() => void handleNewChat()}>
                <Plus className="size-4" strokeWidth={1.5} aria-hidden />
                <span>New chat</span>
              </Button>
            </div>

            {threads.length > 0 && (
              <div className="flex items-center gap-2 overflow-x-auto pb-1">
                {threads.slice(0, 8).map((t) => (
                  <Button
                    key={t.id}
                    type="button"
                    size="sm"
                    variant={t.id === activeThreadId ? "default" : "outline"}
                    className="min-h-11 shrink-0"
                    onClick={() => setActiveThreadId(t.id)}
                  >
                    <span className="max-w-[160px] truncate">{t.title || "Conversation"}</span>
                  </Button>
                ))}
              </div>
            )}

            <div className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5">
              {threadMessages.length === 0 ? (
                <p className="text-sm text-ink-2">Ask your agent to handle a task or check your workspace.</p>
              ) : (
                threadMessages.map((msg, i) => (
                  <div
                    key={i}
                    className={cn(
                      "rounded-sm px-3 py-2 text-sm leading-5 [overflow-wrap:anywhere]",
                      msg.role === "user"
                        ? "ml-6 bg-board text-ink"
                        : msg.role === "step"
                          ? "border border-hairline bg-canvas text-xs text-ink-2"
                          : "mr-4 bg-surface text-ink"
                    )}
                  >
                    <span className="mb-0.5 flex items-center gap-1.5 text-xs font-medium text-ink-2">
                      {msg.role === "user" ? (
                        "You"
                      ) : msg.role === "step" ? (
                        "Step"
                      ) : (
                        <>
                          <ErgonMark className="size-3 text-ink" />
                          <span>Ergon</span>
                        </>
                      )}
                    </span>
                    <p className="whitespace-pre-wrap">{msg.content}</p>
                  </div>
                ))
              )}
            </div>

            <form id="phone-chat-form" onSubmit={(e) => void handleSendChat(e)} className="flex flex-col gap-2">
              <label htmlFor="phone-chat-input" className="sr-only">Message Ergon</label>
              <Textarea
                id="phone-chat-input"
                value={chatDraft}
                onChange={(e) => setChatDraft(e.target.value)}
                placeholder="Ask Ergon to run a task…"
                rows={3}
                className="min-h-11 bg-surface text-sm text-ink"
              />
              <Button id="phone-chat-send" type="submit" disabled={chatSending || !chatDraft.trim()} className="min-h-11 gap-1.5">
                <Send className="size-4" strokeWidth={1.5} aria-hidden />
                <span>{chatSending ? "Sending…" : "Send"}</span>
              </Button>
            </form>
          </div>
        )}

        {/* TAB 4: SUITE & COMPANY MEMORY */}
        {tab === "suite" && (
          <div id="phone-tab-suite" className="flex flex-col gap-5">
            <h1 className="text-title font-medium text-ink">Suite &amp; Memory</h1>

            {/* Installed Suite & Tools */}
            <section id="phone-suite-installed" className="flex flex-col gap-2.5">
              <h2 className="text-sm font-medium text-ink">
                Installed tools &amp; workflows ({suiteItems.length})
              </h2>
              {suiteItems.length === 0 ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  No tools or workflows installed yet.
                </div>
              ) : (
                suiteItems.map((item) => {
                  const full = toolsById.get(item.id);
                  const params = item.params ?? full?.params ?? item.fields ?? [];
                  return (
                    <div key={item.id} className="flex items-start justify-between gap-3 rounded-card border border-hairline bg-surface p-3.5">
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant="outline" className="text-xs">{item.kind}</Badge>
                          <span className="text-sm font-medium text-ink [overflow-wrap:anywhere]">{item.name}</span>
                        </div>
                        <p className="mt-1 text-sm text-ink-2 [overflow-wrap:anywhere]">{item.description}</p>
                      </div>
                      <Button
                        type="button"
                        size="sm"
                        className="min-h-11 shrink-0 gap-1"
                        onClick={() =>
                          openRunDialog({
                            id: item.id,
                            kind: item.kind,
                            name: item.name,
                            description: item.description,
                            params,
                          })
                        }
                      >
                        <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                        <span>Run</span>
                      </Button>
                    </div>
                  );
                })
              )}
            </section>

            {/* Company Library */}
            <section id="phone-company-library" className="flex flex-col gap-2.5">
              <h2 className="text-sm font-medium text-ink">
                Company library ({availableLibrary.length})
              </h2>
              {availableLibrary.length === 0 ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  All shared company items are already installed in your suite.
                </div>
              ) : (
                availableLibrary.map((item) => (
                  <div key={item.id} className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <Badge variant="outline" className="text-xs">{item.kind}</Badge>
                        <span className="text-sm font-medium text-ink [overflow-wrap:anywhere]">{item.name}</span>
                      </div>
                      <span className="mono text-xs text-ink-2">by {item.author}</span>
                    </div>
                    <p className="text-sm text-ink-2 [overflow-wrap:anywhere]">{item.description}</p>
                    {item.test_report && (
                      <span className="mono text-xs text-pill-active-fg">
                        {item.test_report.passed}/{item.test_report.total} sandbox tests passed
                      </span>
                    )}
                    <div>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="min-h-11 gap-1.5"
                        onClick={() => void handleInstallItem(item)}
                      >
                        <Wrench className="size-3.5" strokeWidth={1.5} aria-hidden />
                        <span>Install</span>
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </section>

            {/* Company Memory */}
            <section id="phone-company-memory" className="flex flex-col gap-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <Database className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
                  <h2 className="text-sm font-medium text-ink">
                    Company memory ({companyMemory.length})
                  </h2>
                </div>
                <Badge variant="secondary" className="text-xs">Shared</Badge>
              </div>

              {companyMemory.length === 0 ? (
                <div className="rounded-card border border-hairline bg-surface p-4 text-sm text-ink-2">
                  No shared company memory entries visible for your role yet.
                </div>
              ) : (
                <div className="flex flex-col gap-2">
                  {companyMemory.map((entry) => (
                    <div key={entry.id} className="flex items-start justify-between gap-2 rounded-card border border-hairline bg-surface p-3.5">
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-1.5">
                          <Badge variant="outline" className="text-xs">{entry.category}</Badge>
                          <span className="mono rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">
                            {entry.scope}
                          </span>
                        </div>
                        <p className="mt-1.5 text-sm text-ink [overflow-wrap:anywhere]">{entry.content}</p>
                      </div>
                      {canWriteCompanyMemory && (
                        <Button
                          type="button"
                          size="icon"
                          variant="ghost"
                          aria-label={`Delete memory ${entry.id}`}
                          className="min-h-11 min-w-11 shrink-0"
                          onClick={() => void handleDeleteCompanyMemory(entry.id)}
                        >
                          <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                        </Button>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {canWriteCompanyMemory && (
                <form
                  onSubmit={(e) => void handleAddCompanyMemory(e)}
                  className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
                >
                  <p className="text-sm font-medium text-ink">Add shared company note</p>
                  <div className="grid grid-cols-2 gap-2">
                    <div>
                      <label htmlFor="phone-mem-category" className="mb-1 block text-xs text-ink-2">Category</label>
                      <select
                        id="phone-mem-category"
                        value={memCategory}
                        onChange={(e) => setMemCategory(e.target.value as CompanyMemoryCategory)}
                        className="min-h-11 w-full rounded-sm border border-hairline bg-surface px-2.5 text-sm text-ink"
                      >
                        <option value="fact">fact</option>
                        <option value="procedure">procedure</option>
                        <option value="preference">preference</option>
                        <option value="policy">policy</option>
                      </select>
                    </div>
                    <div>
                      <label htmlFor="phone-mem-scope" className="mb-1 block text-xs text-ink-2">Scope</label>
                      <Input
                        id="phone-mem-scope"
                        value={memScope}
                        onChange={(e) => setMemScope(e.target.value)}
                        placeholder="company"
                        className="min-h-11"
                      />
                    </div>
                  </div>
                  <div>
                    <label htmlFor="phone-mem-content" className="mb-1 block text-xs text-ink-2">Content</label>
                    <Textarea
                      id="phone-mem-content"
                      value={memContent}
                      onChange={(e) => setMemContent(e.target.value)}
                      placeholder="Shared procedure, policy, or preference…"
                      rows={2}
                      className="min-h-11 text-sm"
                    />
                  </div>
                  <Button type="submit" disabled={savingMem || !memContent.trim()} className="min-h-11">
                    {savingMem ? "Saving…" : "Save to company memory"}
                  </Button>
                </form>
              )}
            </section>
          </div>
        )}
      </main>

      {/* Bottom Fixed Navigation Bar */}
      <nav
        id="phone-bottom-nav"
        aria-label="Phone navigation"
        className="fixed inset-x-0 bottom-0 z-30 border-t border-hairline bg-surface"
      >
        <div role="tablist" aria-label="Phone views" className="mx-auto grid max-w-lg grid-cols-4 gap-1 px-2 py-1.5">
          {NAV_TABS.map((item) => {
            const Icon = item.icon;
            const active = tab === item.id;
            return (
              <button
                key={item.id}
                type="button"
                role="tab"
                aria-selected={active}
                data-phone-tab={item.id}
                onClick={() => setTab(item.id)}
                className={cn(
                  "relative flex min-h-11 flex-col items-center justify-center gap-0.5 rounded-sm px-2 py-1 text-xs font-medium outline-none focus-visible:outline-2 focus-visible:outline-ring",
                  active ? "text-ink" : "text-ink-2"
                )}
              >
                {active && (
                  <m.span
                    layoutId="phone-nav-indicator"
                    transition={glide}
                    className="absolute inset-0 -z-10 rounded-sm bg-board"
                  />
                )}
                <span className="relative inline-flex items-center gap-1">
                  <Icon className="size-4" strokeWidth={1.5} aria-hidden />
                  {item.id === "decisions" && actionCount > 0 && (
                    <span className="mono rounded-chip bg-pill-warn-bg px-1.5 py-0.5 text-xs font-medium text-pill-warn-fg">
                      {actionCount}
                    </span>
                  )}
                </span>
                <span>{item.label}</span>
              </button>
            );
          })}
        </div>
      </nav>

      {/* Root-mounted Settings Modal (Light/Dark toggle lives ONLY in Settings #67) */}
      <PanelModal
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        title="Phone Settings"
        description="Appearance, notifications, and workspace session."
        panelId="phone-settings-panel"
      >
        <div className="flex flex-col gap-4">
          <section aria-labelledby="phone-appearance-h" className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-board p-3.5">
            <div>
              <h3 id="phone-appearance-h" className="text-sm font-medium text-ink">Appearance</h3>
              <p className="text-xs text-ink-2">Follow system appearance or choose Light / Dark for this device.</p>
            </div>
            <SegmentedControl aria-label="Appearance">
              <SegmentedControlItem
                data-id="system"
                aria-pressed={theme === "system"}
                data-checked={theme === "system" ? "" : undefined}
                className="min-h-11 gap-1.5 px-3"
                onClick={() => setTheme("system")}
              >
                <Monitor className="size-4" strokeWidth={1.5} aria-hidden />
                <span>System</span>
              </SegmentedControlItem>
              <SegmentedControlItem
                data-id="light"
                aria-pressed={theme === "light"}
                data-checked={theme === "light" ? "" : undefined}
                className="min-h-11 gap-1.5 px-3"
                onClick={() => setTheme("light")}
              >
                <Sun className="size-4" strokeWidth={1.5} aria-hidden />
                <span>Light</span>
              </SegmentedControlItem>
              <SegmentedControlItem
                data-id="dark"
                aria-pressed={theme === "dark"}
                data-checked={theme === "dark" ? "" : undefined}
                className="min-h-11 gap-1.5 px-3"
                onClick={() => setTheme("dark")}
              >
                <Moon className="size-4" strokeWidth={1.5} aria-hidden />
                <span>Dark</span>
              </SegmentedControlItem>
            </SegmentedControl>
          </section>

          <section className="flex flex-col gap-2 rounded-card border border-hairline bg-board p-3.5">
            <h3 className="text-sm font-medium text-ink">Session</h3>
            <p className="text-sm text-ink-2">
              Signed in as <span className="font-medium text-ink">{employeeName}</span> ({employeeRole})
            </p>
            {status && status !== "error" && status.model && (
              <p className="mono text-xs text-ink-2">Model: {status.model}</p>
            )}
            <div className="flex flex-wrap gap-2 pt-1">
              <a
                href="/"
                className={cn(buttonVariants({ size: "sm", variant: "outline" }), "min-h-11")}
              >
                Open Desktop Workbench
              </a>
              {auth?.auth_required && (
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  className="min-h-11 gap-1.5"
                  onClick={() => {
                    setSettingsOpen(false);
                    void onSignOut();
                  }}
                >
                  <LogOut className="size-4" strokeWidth={1.5} aria-hidden />
                  <span>Sign out</span>
                </Button>
              )}
            </div>
          </section>
        </div>
      </PanelModal>

      {/* Root-mounted Run Tool / Workflow Modal */}
      <PanelModal
        open={Boolean(runTarget)}
        onOpenChange={(v) => {
          if (!v) setRunTarget(null);
        }}
        title={runTarget ? `Run ${runTarget.name}` : "Run"}
        description={runTarget?.description}
        panelId="phone-run-modal"
      >
        {runTarget && (
          <form onSubmit={(e) => void handleExecuteRunTarget(e)} className="flex flex-col gap-3">
            {runTarget.params.map((param) => (
              <div key={param} className="flex flex-col gap-1">
                <label htmlFor={`run-param-${param}`} className="text-xs font-medium text-ink-2">
                  {param}
                </label>
                <Input
                  id={`run-param-${param}`}
                  value={runInputs[param] ?? ""}
                  onChange={(e) => setRunInputs((prev) => ({ ...prev, [param]: e.target.value }))}
                  className="min-h-11"
                />
              </div>
            ))}
            <div className="flex flex-wrap gap-2">
              <Button type="submit" disabled={runningTool} className="min-h-11 flex-1 gap-1.5">
                <Play className="size-4" strokeWidth={1.5} aria-hidden />
                <span>{runningTool ? "Running…" : "Run now"}</span>
              </Button>
              <Button type="button" variant="outline" className="min-h-11" onClick={() => setRunTarget(null)}>
                Close
              </Button>
            </div>
            {runOutput && (
              <pre
                className={cn(
                  "mono max-h-60 overflow-auto whitespace-pre-wrap break-words rounded-sm border border-hairline p-3 text-xs",
                  runOutput.ok ? "bg-board text-ink" : "bg-pill-warn-bg text-pill-warn-fg"
                )}
              >
                {runOutput.text}
              </pre>
            )}
          </form>
        )}
      </PanelModal>
    </div>
  );
}

export default PhoneFace;
