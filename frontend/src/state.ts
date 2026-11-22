// Ergon client state: conversations, runs, drafts, pending recovery, person isolation, and live rail/tab counts.
import { useCallback, useEffect, useRef, useState } from "react";
import { call, type Msg, type Person, type Status, type SuiteItem, type Thread, type ThreadRow } from "@/lib/api";
import { buildStarterCards, type StarterCard } from "@/lib/words";
import { threadState } from "@/chat/Entry";
import type { Recent } from "@/chat/Chat";

const DRAFT = "ergon-draft:";
const THREAD_KEY = "ergon-thread";
const PENDING = "ergon-pending";
const store = {
  get(k: string): string { try { return sessionStorage.getItem(k) || ""; } catch { return ""; } },
  set(k: string, v: string) { try { v ? sessionStorage.setItem(k, v) : sessionStorage.removeItem(k); } catch { /* convenience */ } },
};

export type TState = "waiting" | "finished" | "none";

function countLearnedLines(md: string): number {
  return String(md || "")
    .split("\n")
    .filter((l) => {
      const t = l.trim();
      return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:");
    }).length;
}

export function useErgon() {
  const [status, setStatus] = useState<Status | null | "error">(null);
  const [runFailed, setRunFailed] = useState(false);
  const [people, setPeople] = useState<{ people: Person[]; active: string } | null>(null);
  const [threads, setThreads] = useState<ThreadRow[] | null>(null);
  const [threadsError, setThreadsError] = useState(false);
  const [states, setStates] = useState<Record<string, TState>>({});
  const [threadId, setThreadIdState] = useState<string | null>(null);
  const threadRef = useRef<string | null>(null);
  const [messages, setMessages] = useState<Msg[]>([]);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const [draft, setDraftState] = useState("");
  const [pendingNote, setPendingNote] = useState(false);
  const [sideError, setSideError] = useState("");
  const [starters, setStarters] = useState<StarterCard[]>(() => buildStarterCards([], ""));
  const [recovered, setRecovered] = useState<string[]>([]);
  const [recent, setRecent] = useState<Recent[]>([]);
  const [todayCount, setTodayCount] = useState(0);
  const [learnedCount, setLearnedCount] = useState(0);
  const [suiteCount, setSuiteCount] = useState(0);

  const setThread = (id: string | null) => { threadRef.current = id; setThreadIdState(id); };
  const draftKey = (id: string | null) => DRAFT + (id || "new");
  const loadDraftFor = (id: string | null) => setDraftState(store.get(draftKey(id)));
  const setDraft = (v: string) => { setDraftState(v); store.set(draftKey(threadRef.current), v); };

  const loadStatus = useCallback(async () => {
    const r = await call<Status>("/api/status");
    if (!r.ok || typeof r.data.model_configured !== "boolean") { setStatus("error"); return; }
    setStatus(r.data);
    setRecovered(Array.isArray(r.data.store_recovered) ? r.data.store_recovered.map(String) : []);
  }, []);

  const loadPeople = useCallback(async () => {
    const r = await call<{ people: Person[]; active: string }>("/api/people");
    if (!r.ok || !Array.isArray(r.data.people)) {
      setPeople(null);
      setSideError("Could not load the people list. Is the Ergon server running?");
      return;
    }
    setPeople({
      people: r.data.people.filter((p): p is Person => !!p && typeof p === "object" && typeof p.id === "string"),
      active: typeof r.data.active === "string" ? r.data.active : "",
    });
    setSideError("");
  }, []);

  const loadThreads = useCallback(async () => {
    const r = await call<{ threads: ThreadRow[] }>("/api/threads");
    if (!r.ok || !Array.isArray(r.data.threads)) { setThreadsError(true); return; }
    setThreadsError(false);
    const clean = r.data.threads.filter((t): t is ThreadRow => !!t && typeof t === "object" && typeof t.id === "string");
    setThreads(clean);
    const found = await Promise.all(clean.slice(0, 40).map(async (t) => {
      const th = await call<Thread>("/api/threads/" + encodeURIComponent(t.id));
      return [t.id, th.ok && Array.isArray(th.data.messages) ? threadState(th.data.messages) : "none"] as const;
    }));
    setStates(Object.fromEntries(found));
  }, []);

  const loadRecent = useCallback(async () => {
    const r = await call<{ finished?: { task?: unknown; date?: unknown; files?: unknown }[] }>("/api/brief");
    if (!r.ok || !Array.isArray(r.data.finished)) { setRecent([]); return; }
    setRecent(
      r.data.finished
        .filter((f) => f && typeof f === "object")
        .map((f) => ({
          task: String(f.task ?? "Completed task"),
          date: String(f.date ?? ""),
          files: Array.isArray(f.files) ? f.files.map(String) : [],
        }))
    );
  }, []);

  const refreshCounts = useCallback(async () => {
    const [d, m] = await Promise.all([
      call<{ decisions: { status?: string }[] }>("/api/decisions"),
      call<{ content?: string }>("/api/memory"),
    ]);
    const dec = d.ok && Array.isArray(d.data.decisions) ? d.data.decisions.filter((x) => x && x.status === "pending").length : 0;
    setTodayCount(dec);
    if (m.ok && typeof m.data.content === "string") setLearnedCount(countLearnedLines(m.data.content));
  }, []);

  const loadStarters = useCallback(async (role: string) => {
    const r = await call<{ installed: SuiteItem[] }>("/api/suite");
    const inst = r.ok && Array.isArray(r.data.installed) ? r.data.installed : [];
    setSuiteCount(inst.length);
    setStarters(buildStarterCards(inst, role));
  }, []);

  const openThread = useCallback(async (id: string) => {
    const r = await call<Thread>("/api/threads/" + encodeURIComponent(id));
    if (!r.ok || !Array.isArray(r.data.messages)) {
      store.set(THREAD_KEY, "");
      setMessages([{ role: "error", content: "That conversation could not be opened. It may have been removed." }]);
      return;
    }
    setThread(id);
    try { sessionStorage.setItem(THREAD_KEY, id); } catch { /* convenience */ }
    loadDraftFor(id);
    setMessages(r.data.messages);
    loadThreads();
  }, [loadThreads]);

  const newThread = useCallback(async () => {
    const r = await call<{ id: string }>("/api/threads", { method: "POST", body: {} });
    if (!r.ok || typeof r.data.id !== "string") { setSideError("Could not start a conversation. Is the Ergon server running?"); return; }
    setThread(r.data.id);
    store.set(THREAD_KEY, r.data.id);
    loadDraftFor(r.data.id);
    setMessages([]);
    setPendingNote(false);
    loadThreads();
  }, [loadThreads]);

  const showRun = useCallback((id: string, msgs: Msg[]) => {
    setThread(id);
    try { sessionStorage.setItem(THREAD_KEY, id); } catch { /* convenience */ }
    setMessages(Array.isArray(msgs) ? msgs : []);
    loadThreads();
    refreshCounts();
    loadRecent();
  }, [loadThreads, refreshCounts, loadRecent]);

  const refreshPerson = useCallback(async () => {
    setThread(null);
    store.set(THREAD_KEY, "");
    loadDraftFor(null);
    setMessages([]);
    setPendingNote(false);
    await Promise.all([loadThreads(), loadStatus(), loadPeople(), loadRecent(), refreshCounts()]);
  }, [loadThreads, loadStatus, loadPeople, loadRecent, refreshCounts]);

  const switchPerson = useCallback(async (id: string) => {
    await call("/api/active-person", { method: "POST", body: { id } });
    await refreshPerson();
  }, [refreshPerson]);

  const send = useCallback(async () => {
    if (busyRef.current) return;
    const text = draft.trim();
    if (!text) return;
    busyRef.current = true;
    setBusy(true);
    store.set(draftKey(threadRef.current), "");
    setPendingNote(false);
    store.set(PENDING, JSON.stringify({ thread: threadRef.current || "new", text }));
    setDraftState("");
    setMessages((m) => [...m, { role: "user", content: text }]);
    const failRun = (msg: string) => {
      setMessages((m) => [...m, { role: "error", content: msg }]);
      setRunFailed(true);
      setDraft(text);
    };
    let id = threadRef.current;
    if (!id) {
      const c = await call<{ id: string }>("/api/threads", { method: "POST", body: {} });
      if (c.ok && typeof c.data.id === "string") { id = c.data.id; setThread(id); }
    }
    if (!id) {
      store.set(PENDING, "");
      failRun("Could not reach the Ergon server.");
    } else {
      const r = await call<{ messages: Msg[] }>("/api/chat", { method: "POST", body: { thread_id: id, message: text } });
      store.set(PENDING, "");
      if (!r.ok) {
        failRun(r.status === 0 ? "Could not reach the Ergon server." : r.error || "The agent could not run.");
      } else if (Array.isArray(r.data.messages)) {
        setRunFailed(false);
        setMessages(r.data.messages);
      } else {
        failRun("The agent could not run.");
      }
    }
    busyRef.current = false;
    setBusy(false);
    loadThreads();
    refreshCounts();
    loadRecent();
  }, [draft, loadThreads, refreshCounts, loadRecent]);

  useEffect(() => {
    (async () => {
      loadStatus();
      loadPeople();
      loadThreads();
      loadRecent();
      refreshCounts();
      const pend = (() => { try { const v = JSON.parse(store.get(PENDING) || "null"); return v && typeof v.text === "string" ? v : null; } catch { return null; } })();
      const reopen = store.get(THREAD_KEY);
      if (reopen) await openThread(reopen);
      else { loadDraftFor(null); setMessages([]); }
      if (!pend) return;
      store.set(PENDING, "");
      let landed = false;
      if (pend.thread !== "new") {
        const t = await call<Thread>("/api/threads/" + encodeURIComponent(pend.thread));
        landed = t.ok && Array.isArray(t.data.messages) && t.data.messages.some((m) => m.role === "user" && String(m.content).trim() === pend.text);
      }
      if (landed) return;
      setDraft(pend.text);
      setPendingNote(true);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (status && status !== "error") loadStarters(String(status.person?.role ?? ""));
  }, [status, loadStarters]);

  const isDemo = Boolean(
    typeof window !== "undefined" && (
      (window as unknown as { __ERGON_DEMO__?: boolean }).__ERGON_DEMO__
      || new URLSearchParams(window.location.search).get("demo") === "1"
    )
  );

  return {
    status, runFailed, people, threads, threadsError, states, threadId, messages, busy, draft, setDraft,
    pendingNote, sideError, starters, recovered, recent, todayCount, learnedCount, suiteCount, isDemo,
    send, openThread, newThread, switchPerson, refreshPerson, showRun, loadThreads, loadStatus, loadPeople, refreshCounts, loadRecent, loadStarters,
    setSideError, setRecovered,
  };
}
