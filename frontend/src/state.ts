// Ergon client state: conversations, live streamed runs, drafts, pending recovery,
// person isolation, real workspace files, and live rail/tab counts.
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import {
  call,
  type LiveRunStatus,
  type Msg,
  type Person,
  type Status,
  type SuiteItem,
  type Thread,
  type ThreadRow,
  type WorkspaceFile,
} from "@/lib/api";
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
  const [liveSteps, setLiveSteps] = useState<string[]>([]);
  const [draft, setDraftState] = useState("");
  const [pendingNote, setPendingNote] = useState(false);
  const [sideError, setSideError] = useState("");
  const [starters, setStarters] = useState<StarterCard[]>(() => buildStarterCards([], ""));
  const [recovered, setRecovered] = useState<string[]>([]);
  const [recent, setRecent] = useState<Recent[]>([]);
  const [todayCount, setTodayCount] = useState(0);
  const [learnedCount, setLearnedCount] = useState(0);
  const [suiteCount, setSuiteCount] = useState(0);
  const [files, setFiles] = useState<WorkspaceFile[]>([]);

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

  const loadFiles = useCallback(async () => {
    const r = await call<{ files?: WorkspaceFile[] }>("/api/files");
    if (!r.ok || !Array.isArray(r.data.files)) return;
    setFiles(
      r.data.files.filter(
        (f): f is WorkspaceFile => !!f && typeof f === "object" && typeof f.path === "string"
      )
    );
  }, []);

  const uploadFile = useCallback(async (path: string, content: string): Promise<boolean> => {
    const r = await call<WorkspaceFile>("/api/files/upload", {
      method: "POST",
      body: { path, content },
    });
    if (!r.ok) {
      toast.error(r.error || `Could not save ${path}.`);
      return false;
    }
    toast.success(`Saved ${r.data.path} to your workspace.`);
    await loadFiles();
    return true;
  }, [loadFiles]);

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
        .filter((x) => !!x && typeof x === "object")
        .slice(0, 4)
        .map((x) => ({
          task: String(x.task ?? "Completed task"),
          date: String(x.date ?? ""),
          files: Array.isArray(x.files) ? x.files.map(String) : [],
        }))
    );
  }, []);

  const refreshCounts = useCallback(async () => {
    const [dec, mem, sui] = await Promise.all([
      call<{ decisions?: { status?: string }[] }>("/api/decisions"),
      call<{ content?: string }>("/api/memory"),
      call<{ installed?: SuiteItem[] }>("/api/suite"),
    ]);
    if (dec.ok && Array.isArray(dec.data.decisions)) {
      setTodayCount(dec.data.decisions.filter((d) => d && d.status === "pending").length);
    }
    if (mem.ok && typeof mem.data.content === "string") {
      setLearnedCount(countLearnedLines(mem.data.content));
    }
    if (sui.ok && Array.isArray(sui.data.installed)) {
      setSuiteCount(sui.data.installed.length);
    }
  }, []);

  const loadStarters = useCallback(async (role: string) => {
    const r = await call<{ installed?: SuiteItem[] }>("/api/suite");
    const items = r.ok && Array.isArray(r.data.installed) ? r.data.installed : [];
    setStarters(buildStarterCards(items, role));
    setSuiteCount(items.length);
  }, []);

  const openThread = useCallback(async (id: string) => {
    if (busyRef.current) return;
    setThread(id);
    store.set(THREAD_KEY, id);
    loadDraftFor(id);
    setPendingNote(false);
    const r = await call<Thread>("/api/threads/" + encodeURIComponent(id));
    if (!r.ok || !Array.isArray(r.data.messages)) {
      setThread(null);
      store.set(THREAD_KEY, "");
      setMessages([{ role: "error", content: "That conversation could not be opened. It may have been removed." }]);
      loadThreads();
      return;
    }
    setMessages(r.data.messages);
  }, [loadThreads]);

  const newThread = useCallback(async () => {
    if (busyRef.current) return;
    const r = await call<{ id?: string }>("/api/threads", { method: "POST", body: {} });
    if (!r.ok || typeof r.data.id !== "string") {
      setSideError("Could not start a new conversation. Is the Ergon server running?");
      return;
    }
    setSideError("");
    setThread(r.data.id);
    store.set(THREAD_KEY, r.data.id);
    loadDraftFor(r.data.id);
    setMessages([]);
    setPendingNote(false);
    loadThreads();
  }, [loadThreads]);

  const refreshPerson = useCallback(async () => {
    setThread(null);
    store.set(THREAD_KEY, "");
    setMessages([]);
    loadDraftFor(null);
    setPendingNote(false);
    await Promise.all([loadStatus(), loadPeople(), loadThreads(), loadRecent(), refreshCounts(), loadFiles()]);
  }, [loadStatus, loadPeople, loadThreads, loadRecent, refreshCounts, loadFiles]);

  const switchPerson = useCallback(async (id: string) => {
    if (busyRef.current) return;
    const r = await call("/api/active-person", { method: "POST", body: { id } });
    if (!r.ok) {
      setSideError("Could not switch person. Check that Ergon is running.");
      return;
    }
    setSideError("");
    await refreshPerson();
  }, [refreshPerson]);

  const showRun = useCallback((id: string, msgs: Msg[]) => {
    setThread(id);
    store.set(THREAD_KEY, id);
    setMessages(Array.isArray(msgs) ? msgs : []);
    loadThreads();
    loadRecent();
    refreshCounts();
    loadFiles();
  }, [loadThreads, loadRecent, refreshCounts, loadFiles]);

  // Poll live tool steps while an agent run is in flight so progress streams in real time.
  useEffect(() => {
    if (!busy) {
      setLiveSteps([]);
      return;
    }
    let cancelled = false;
    const tick = async () => {
      const r = await call<LiveRunStatus>("/api/runs/status");
      if (!cancelled && r.ok && r.data.active && Array.isArray(r.data.steps)) {
        setLiveSteps(r.data.steps);
      }
    };
    tick();
    const id = window.setInterval(tick, 350);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [busy]);

  const send = useCallback(async () => {
    if (busyRef.current) return;
    const text = draft.trim();
    if (!text) return;
    const cur = threadRef.current;
    busyRef.current = true;
    setBusy(true);
    setLiveSteps([]);
    setPendingNote(false);
    setDraft("");
    store.set(PENDING, JSON.stringify({ thread: cur || "new", text }));
    setMessages((m) => [...m, { role: "user", content: text }]);
    const r = await call<Thread & { thread_id?: string }>("/api/chat", { method: "POST", body: { thread_id: cur || "", message: text } });
    store.set(PENDING, "");
    busyRef.current = false;
    setBusy(false);
    setLiveSteps([]);
    if (r.ok && Array.isArray(r.data.messages)) {
      const id = r.data.id || r.data.thread_id || cur;
      setThread(id);
      if (id) store.set(THREAD_KEY, id);
      setMessages(r.data.messages);
      setRunFailed(false);
    } else {
      const errMsg = !r.ok
        ? (r.status === 0 ? "Could not reach the Ergon server." : r.error || `The request failed (HTTP ${r.status}).`)
        : "The agent returned an unexpected response.";
      setMessages((m) => [...m, { role: "error", content: errMsg }]);
      setRunFailed(true);
    }
    loadThreads();
    loadRecent();
    refreshCounts();
    loadFiles();
  }, [draft, loadThreads, loadRecent, refreshCounts, loadFiles]);

  useEffect(() => {
    (async () => {
      loadStatus();
      loadPeople();
      loadThreads();
      loadRecent();
      refreshCounts();
      loadFiles();
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

  return {
    status, runFailed, people, threads, threadsError, states, threadId, messages, busy, liveSteps, draft, setDraft,
    pendingNote, sideError, starters, recovered, recent, todayCount, learnedCount, suiteCount, files,
    send, openThread, newThread, switchPerson, refreshPerson, showRun, loadThreads, loadStatus, loadPeople, refreshCounts, loadRecent, loadStarters, loadFiles, uploadFile,
    setSideError, setRecovered,
  };
}
