// Ergon's client state: the old page's behaviour (threads, runs, drafts, pending sends, person switch) in one hook.
import { useCallback, useEffect, useRef, useState } from "react";
import { call, type Msg, type Person, type Status, type ThreadRow, type Thread, type SuiteItem } from "@/lib/api";

const DRAFT = "ergon-draft:";
const THREAD_KEY = "ergon-thread";
const PENDING = "ergon-pending";
const store = {
  get(k: string): string { try { return sessionStorage.getItem(k) || ""; } catch { return ""; } },
  set(k: string, v: string) { try { v ? sessionStorage.setItem(k, v) : sessionStorage.removeItem(k); } catch { /* a draft is a convenience */ } },
};

export const STARTER_FALLBACK = "Research a topic for me";

export function starterPrompts(installed: SuiteItem[], role: string): string[] {
  const out = ["Tidy my notes into a summary"];
  for (const item of installed.slice(0, 2)) out.push("Ask my " + item.name + " for help");
  if (role) out.push("Draft my weekly update");
  out.push(STARTER_FALLBACK);
  return out.slice(0, 4);
}

export function useErgon() {
  const [status, setStatus] = useState<Status | null | "error">(null);
  const [runFailed, setRunFailed] = useState(false);
  const [people, setPeople] = useState<{ people: Person[]; active: string } | null>(null);
  const [threads, setThreads] = useState<ThreadRow[] | null>(null);
  const [threadsError, setThreadsError] = useState(false);
  const [threadId, setThreadIdState] = useState<string | null>(null);
  const threadRef = useRef<string | null>(null);
  const [messages, setMessages] = useState<Msg[]>([]);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const [draft, setDraftState] = useState("");
  const [pendingNote, setPendingNote] = useState(false);
  const [sideError, setSideError] = useState("");
  const [starters, setStarters] = useState<string[]>([]);
  const [recovered, setRecovered] = useState<string[]>([]);

  const setThread = (id: string | null) => { threadRef.current = id; setThreadIdState(id); };
  const draftKey = (id: string | null) => DRAFT + (id || "new");
  const loadDraftFor = (id: string | null) => setDraftState(store.get(draftKey(id)));
  const setDraft = (v: string) => { setDraftState(v); store.set(draftKey(threadRef.current), v); };

  const loadStatus = useCallback(async () => {
    const r = await call<Status>("/api/status");
    if (!r.ok || typeof r.data.model_configured !== "boolean") { setStatus("error"); return; }
    setStatus(r.data);
    setRecovered(r.data.store_recovered ?? []);
  }, []);

  const loadPeople = useCallback(async () => {
    const r = await call<{ people: Person[]; active: string }>("/api/people");
    if (!r.ok) { setSideError("Could not load the people list. Is the Ergon server running?"); return; }
    setPeople(r.data);
    setSideError("");
  }, []);

  const loadThreads = useCallback(async () => {
    const r = await call<{ threads: ThreadRow[] }>("/api/threads");
    if (!r.ok) { setThreadsError(true); return; }
    setThreadsError(false);
    setThreads(r.data.threads);
  }, []);

  const loadStarters = useCallback(async (role: string) => {
    const r = await call<{ installed: SuiteItem[] }>("/api/suite");
    setStarters(starterPrompts(r.ok ? r.data.installed ?? [] : [], role));
  }, []);

  const showEmpty = useCallback(() => { setMessages([]); }, []);

  const openThread = useCallback(async (id: string) => {
    const r = await call<Thread>("/api/threads/" + encodeURIComponent(id));
    if (!r.ok) {
      store.set(THREAD_KEY, "");
      setMessages([{ role: "error", content: "That conversation could not be opened. It may have been removed." }]);
      return;
    }
    setThread(id);
    try { sessionStorage.setItem(THREAD_KEY, id); } catch { /* a convenience */ }
    loadDraftFor(id);
    setMessages(r.data.messages ?? []);
    loadThreads();
  }, [loadThreads]);

  const newThread = useCallback(async () => {
    const r = await call<{ id: string }>("/api/threads", { method: "POST", body: {} });
    if (!r.ok) { setSideError("Could not start a conversation. Is the Ergon server running?"); return; }
    setThread(r.data.id);
    store.set(THREAD_KEY, r.data.id);
    loadDraftFor(r.data.id);
    setMessages([]);
    setPendingNote(false);
    loadThreads();
  }, [loadThreads]);

  /** Shows a finished app run in the chat, as the old page did. */
  const showRun = useCallback((id: string, msgs: Msg[]) => {
    setThread(id);
    try { sessionStorage.setItem(THREAD_KEY, id); } catch { /* a convenience */ }
    setMessages(msgs);
    loadThreads();
  }, [loadThreads]);

  /** Person switch or add: the view resets to that person's empty chat. */
  const resetPersonView = useCallback(async () => {
    setThread(null);
    store.set(THREAD_KEY, "");
    loadDraftFor(null);
    setMessages([]);
    setPendingNote(false);
  }, []);

  /** After a person is added or switched: the view resets, and the lists reload for that person. */
  const refreshPerson = useCallback(async () => {
    await resetPersonView();
    await Promise.all([loadThreads(), loadStatus(), loadPeople()]);
  }, [resetPersonView, loadThreads, loadStatus, loadPeople]);

  const switchPerson = useCallback(async (id: string) => {
    await call("/api/active-person", { method: "POST", body: { id } });
    await refreshPerson();
  }, [refreshPerson]);

  const send = useCallback(async () => {
    if (busyRef.current) return;
    const text = draft.trim();
    if (!text) return;
    busyRef.current = true; setBusy(true);
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
      if (c.ok) { id = c.data.id; setThread(id); }
    }
    if (!id) { store.set(PENDING, ""); failRun("Could not reach the Ergon server."); }
    else {
      const r = await call<{ messages: Msg[] }>("/api/chat", { method: "POST", body: { thread_id: id, message: text } });
      store.set(PENDING, "");
      if (r.ok) { setRunFailed(false); setMessages(r.data.messages ?? []); }
      else failRun(r.status === 0 ? "Could not reach the Ergon server." : r.error || "The agent could not run.");
    }
    busyRef.current = false; setBusy(false);
    loadThreads();
  }, [draft, loadThreads]);

  useEffect(() => {
    if (status && status !== "error") loadStarters(status.person?.role ?? "");
  }, [status, loadStarters]);

  /** On load: reopen the conversation, then put back a send that never landed. */
  useEffect(() => {
    (async () => {
      loadStatus(); loadPeople(); loadThreads();
      const pend = (() => { try { const v = JSON.parse(store.get(PENDING) || "null"); return v && typeof v.text === "string" ? v : null; } catch { return null; } })();
      const reopen = store.get(THREAD_KEY);
      if (reopen) await openThread(reopen); else { loadDraftFor(null); showEmpty(); }
      if (!pend) return;
      store.set(PENDING, "");
      let landed = false;
      if (pend.thread !== "new") {
        const t = await call<Thread>("/api/threads/" + encodeURIComponent(pend.thread));
        landed = t.ok && (t.data.messages ?? []).some((m) => m.role === "user" && String(m.content).trim() === pend.text);
      }
      if (landed) return;
      setDraft(pend.text);
      setPendingNote(true);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    status, runFailed, people, threads, threadsError, threadId, messages, busy, draft, setDraft,
    pendingNote, sideError, starters, recovered,
    send, openThread, newThread, switchPerson, refreshPerson, showRun, loadStatus, loadPeople, loadThreads, loadStarters,
    setSideError,
  };
}
