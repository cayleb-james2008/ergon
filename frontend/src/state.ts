// Ergon v0.5.0 client state: conversations, live SSE (/api/runs/stream) + /api/runs/status fallback,
// active run cancellation, multi-file upload, 12 built-in workflows catalog, recurring routines, drafts,
// pending recovery, person isolation, and live rail/tab counts.
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import {
  call,
  hasV050Api,
  type LiveRunStatus,
  type LiveRunStep,
  type Msg,
  type Person,
  type Routine,
  type Status,
  type SuiteItem,
  type Thread,
  type ThreadRow,
  type WorkspaceFile,
} from "@/lib/api";
import { CANONICAL_SKILLS, mergeServerSkills, type SkillMeta } from "@/lib/skillsCatalog";
import { buildStarterCards, toolLabel, type StarterCard } from "@/lib/words";
import { threadState } from "@/chat/Entry";
import type { Recent } from "@/chat/Chat";

const DRAFT = "ergon-draft:";
const THREAD_KEY = "ergon-thread";
const PENDING = "ergon-pending";
const LOCAL_ROUTINES_KEY = "ergon-local-routines-v5";

const store = {
  get(k: string): string { try { return sessionStorage.getItem(k) || ""; } catch { return ""; } },
  set(k: string, v: string) { try { v ? sessionStorage.setItem(k, v) : sessionStorage.removeItem(k); } catch { /* convenience */ } },
};

function loadLocalRoutines(): Routine[] {
  try {
    const raw = localStorage.getItem(LOCAL_ROUTINES_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter((x): x is Routine => !!x && typeof x === "object" && typeof x.id === "string") : [];
  } catch {
    return [];
  }
}

function saveLocalRoutines(list: Routine[]) {
  try {
    localStorage.setItem(LOCAL_ROUTINES_KEY, JSON.stringify(list));
  } catch { /* ignore */ }
}

export type TState = "waiting" | "finished" | "none";

function countLearnedLines(md: string): number {
  return String(md || "")
    .split("\n")
    .filter((l) => {
      const t = l.trim();
      return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:");
    }).length;
}

function normalizeStepItem(raw: string | LiveRunStep, idx: number, runStarted: number): LiveRunStep {
  if (raw && typeof raw === "object") {
    const tool = String(raw.tool ?? "").trim();
    const rawStr = String(raw.raw ?? (tool ? `Tool: ${tool}` : raw.label ?? ""));
    return {
      id: raw.id || `step-${idx}`,
      tool,
      label: raw.label || (tool ? toolLabel(tool, undefined, rawStr) || tool : rawStr),
      status: raw.status || "done",
      elapsed_ms: typeof raw.elapsed_ms === "number" ? raw.elapsed_ms : undefined,
      started_ms: raw.started_ms,
      finished_ms: raw.finished_ms,
      raw: rawStr,
      file: raw.file,
    };
  }
  const safe = String(raw ?? "");
  const nl = safe.indexOf("\n");
  const head = nl < 0 ? safe : safe.slice(0, nl);
  const body = nl < 0 ? "" : safe.slice(nl + 1);
  const m = /^Tool: (\S+)(?: \(via ([^)]+)\))?$/.exec(head);
  const tool = m ? m[1] : "";
  const label = (m && toolLabel(m[1], m[2], body)) || head;
  const waiting = /parked for your person|has NOT been deleted yet/i.test(safe);
  const isErr = /^error:/i.test(body.trim());
  return {
    id: `step-${idx}-${tool || "s"}`,
    tool,
    label,
    status: waiting ? "waiting" : isErr ? "error" : "done",
    elapsed_ms: runStarted > 0 ? Math.max(1, Date.now() - runStarted) : undefined,
    raw: safe,
  };
}

function normalizeRoutine(raw: unknown): Routine | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const id = String(o.id ?? "").trim();
  const name = String(o.name ?? o.title ?? "").trim();
  const prompt = String(o.prompt ?? "").trim();
  if (!id || !name || !prompt) return null;
  return {
    id,
    name,
    prompt,
    schedule: String(o.schedule ?? o.cadence ?? "Daily 09:00").trim(),
    skill_id: typeof o.skill_id === "string" && o.skill_id ? o.skill_id : undefined,
    enabled: typeof o.enabled === "boolean" ? o.enabled : true,
    last_run_ts: typeof o.last_run_ms === "number" ? o.last_run_ms : typeof o.last_run_ts === "number" ? o.last_run_ts : null,
    last_status: typeof o.last_status === "string" ? o.last_status : undefined,
    run_count: typeof o.run_count === "number" ? o.run_count : 0,
    created_ts: typeof o.created_ms === "number" ? o.created_ms : typeof o.created_ts === "number" ? o.created_ts : undefined,
  };
}

export function useErgon() {
  const [status, setStatus] = useState<Status | null | "error">(null);
  const statusRef = useRef<Status | null | "error">(null);
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
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const activeRunIdRef = useRef<string | null>(null);
  const [runStartedMs, setRunStartedMs] = useState<number>(0);
  const [liveElapsedMs, setLiveElapsedMs] = useState<number>(0);
  const [streamMode, setStreamMode] = useState<"sse" | "poll">("poll");
  const [cancelling, setCancelling] = useState(false);
  const [liveSteps, setLiveSteps] = useState<string[]>([]);
  const [liveTimeline, setLiveTimeline] = useState<LiveRunStep[]>([]);
  const abortRef = useRef<AbortController | null>(null);
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
  const [skills, setSkills] = useState<SkillMeta[]>(() => CANONICAL_SKILLS);
  const [routines, setRoutines] = useState<Routine[]>(() => loadLocalRoutines());

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const setThread = (id: string | null) => { threadRef.current = id; setThreadIdState(id); };
  const draftKey = (id: string | null) => DRAFT + (id || "new");
  const loadDraftFor = (id: string | null) => setDraftState(store.get(draftKey(id)));
  const setDraft = (v: string) => { setDraftState(v); store.set(draftKey(threadRef.current), v); };

  const loadStatus = useCallback(async () => {
    const r = await call<Status>("/api/status");
    if (!r.ok || typeof r.data.model_configured !== "boolean") { setStatus("error"); return; }
    statusRef.current = r.data;
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

  const loadSkills = useCallback(async () => {
    if (!hasV050Api(statusRef.current, "skills")) {
      setSkills(CANONICAL_SKILLS);
      return;
    }
    const r = await call<unknown>("/api/skills");
    if (r.ok) setSkills(mergeServerSkills(r.data));
    else setSkills(CANONICAL_SKILLS);
  }, []);

  const loadRoutines = useCallback(async () => {
    if (!hasV050Api(statusRef.current, "routines")) {
      setRoutines(loadLocalRoutines());
      return;
    }
    const r = await call<{ routines?: unknown[] }>("/api/routines");
    if (r.ok && Array.isArray(r.data.routines)) {
      const clean = r.data.routines.map(normalizeRoutine).filter((x): x is Routine => x !== null);
      setRoutines(clean);
      return;
    }
    setRoutines(loadLocalRoutines());
  }, []);

  const createRoutine = useCallback(
    async (input: { name: string; schedule: string; prompt: string; skill_id?: string }): Promise<boolean> => {
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call<{ routine?: unknown } & Record<string, unknown>>("/api/routines", {
          method: "POST",
          body: {
            name: input.name,
            title: input.name,
            schedule: input.schedule,
            cadence: input.schedule,
            prompt: input.prompt,
            skill_id: input.skill_id,
            enabled: true,
          },
        });
        if (r.ok) {
          toast.success(`Saved routine "${input.name}".`);
          await loadRoutines();
          return true;
        }
        if (r.status !== 404 && r.status !== 405) {
          toast.error(r.error || "Could not create routine.");
          return false;
        }
      }
      const created: Routine = {
        id: `rt-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        name: input.name,
        schedule: input.schedule,
        prompt: input.prompt,
        skill_id: input.skill_id,
        enabled: true,
        last_run_ts: null,
        run_count: 0,
        created_ts: Date.now(),
      };
      const next = [created, ...loadLocalRoutines()];
      saveLocalRoutines(next);
      setRoutines(next);
      toast.success(`Saved routine "${input.name}".`);
      return true;
    },
    [loadRoutines]
  );

  const deleteRoutine = useCallback(
    async (id: string) => {
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call(`/api/routines/${encodeURIComponent(id)}`, { method: "DELETE" });
        if (r.ok) {
          toast("Deleted routine.");
          await loadRoutines();
          return;
        }
      }
      const next = loadLocalRoutines().filter((x) => x.id !== id);
      saveLocalRoutines(next);
      setRoutines((prev) => prev.filter((x) => x.id !== id));
      toast("Deleted routine.");
    },
    [loadRoutines]
  );

  const uploadFile = useCallback(async (path: string, content: string, silent = false): Promise<boolean> => {
    const r = await call<WorkspaceFile>("/api/files/upload", {
      method: "POST",
      body: { path, content },
    });
    if (!r.ok) {
      if (!silent) toast.error(r.error || `Could not save ${path}.`);
      return false;
    }
    if (!silent) {
      toast.success(`Saved ${r.data.path} to your workspace.`);
      await loadFiles();
    }
    return true;
  }, [loadFiles]);

  const uploadFiles = useCallback(async (picked: FileList | File[]): Promise<number> => {
    const list = Array.from(picked);
    if (!list.length) return 0;
    let okCount = 0;
    const savedNames: string[] = [];
    for (const file of list) {
      try {
        const content = await file.text();
        const ok = await uploadFile(file.name, content, true);
        if (ok) {
          okCount += 1;
          savedNames.push(file.name);
        }
      } catch {
        // continue remaining files
      }
    }
    await loadFiles();
    if (okCount === 1) {
      toast.success(`Saved ${savedNames[0]} to your workspace.`);
    } else if (okCount > 1) {
      toast.success(`Uploaded ${okCount} files to your workspace.`);
    } else {
      toast.error("Could not upload the selected files.");
    }
    return okCount;
  }, [uploadFile, loadFiles]);

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
    await Promise.all([loadStatus(), loadPeople(), loadThreads(), loadRecent(), refreshCounts(), loadFiles(), loadSkills(), loadRoutines()]);
  }, [loadStatus, loadPeople, loadThreads, loadRecent, refreshCounts, loadFiles, loadSkills, loadRoutines]);

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

  // Live SSE (/api/runs/stream) subscription with automatic fallback to /api/runs/status polling + live elapsed timer.
  useEffect(() => {
    if (!busy) {
      setLiveSteps([]);
      setLiveTimeline([]);
      setLiveElapsedMs(0);
      setActiveRunId(null);
      activeRunIdRef.current = null;
      setCancelling(false);
      return;
    }

    const started = runStartedMs || Date.now();
    let cancelled = false;
    let pollTimer: number | null = null;
    let es: EventSource | null = null;

    const elapsedTick = window.setInterval(() => {
      if (!cancelled) setLiveElapsedMs(Math.max(0, Date.now() - started));
    }, 250);

    const applyStatusPayload = (payload: Partial<LiveRunStatus>) => {
      if (cancelled) return;
      if (typeof payload.run_id === "string" && payload.run_id) {
        activeRunIdRef.current = payload.run_id;
        setActiveRunId(payload.run_id);
      }
      if (typeof payload.elapsed_ms === "number") {
        setLiveElapsedMs(payload.elapsed_ms);
      }
      if (Array.isArray(payload.steps)) {
        const normalized = payload.steps.map((s, idx) => normalizeStepItem(s, idx, payload.started_ms || started));
        setLiveTimeline(normalized);
        setLiveSteps(normalized.map((s) => s.raw));
      }
    };

    const startPolling = () => {
      if (cancelled || pollTimer !== null) return;
      setStreamMode("poll");
      const tick = async () => {
        const r = await call<LiveRunStatus>("/api/runs/status");
        if (!cancelled && r.ok && r.data.active) {
          applyStatusPayload(r.data);
        }
      };
      void tick();
      pollTimer = window.setInterval(() => void tick(), 350);
    };

    const handleSsePayload = (rawData: string, eventKind?: string) => {
      if (cancelled || !rawData) return;
      let parsed: Record<string, unknown> | null = null;
      try {
        const j = JSON.parse(rawData);
        if (j && typeof j === "object") parsed = j as Record<string, unknown>;
      } catch {
        return;
      }
      if (!parsed) return;

      if (typeof parsed.run_id === "string" && parsed.run_id) {
        activeRunIdRef.current = parsed.run_id;
        setActiveRunId(parsed.run_id);
      }
      if (typeof parsed.elapsed_ms === "number") {
        setLiveElapsedMs(parsed.elapsed_ms);
      }
      if (Array.isArray(parsed.steps)) {
        applyStatusPayload(parsed as Partial<LiveRunStatus>);
        return;
      }

      const tool = String(parsed.tool ?? "").trim();
      const rawStep = String(parsed.raw ?? parsed.detail ?? parsed.message ?? (tool ? `Tool: ${tool}` : "")).trim();
      if (!tool && !rawStep) return;

      const stepStatus: LiveRunStep["status"] =
        eventKind === "tool_start" || eventKind === "step_start" || eventKind === "run_started" || parsed.status === "running"
          ? "running"
          : eventKind === "approval" || eventKind === "approval_parked" || parsed.status === "waiting"
            ? "waiting"
            : eventKind === "error" || parsed.status === "error"
              ? "error"
              : "done";

      const incoming: LiveRunStep = {
        id: String(parsed.step_id ?? parsed.id ?? `${tool || "step"}-${Date.now()}`),
        tool,
        label: String(parsed.label ?? (tool ? toolLabel(tool, undefined, rawStep) || tool : rawStep)),
        status: stepStatus,
        elapsed_ms: typeof parsed.elapsed_ms === "number" ? parsed.elapsed_ms : Math.max(1, Date.now() - started),
        raw: rawStep || `Tool: ${tool}`,
        file: typeof parsed.file === "string" ? parsed.file : typeof parsed.path === "string" ? parsed.path : undefined,
      };

      setLiveTimeline((prev) => {
        const idx = prev.findIndex((p) => (incoming.id && p.id === incoming.id) || (p.status === "running" && p.tool === incoming.tool));
        const next = idx >= 0 ? prev.map((item, i) => (i === idx ? { ...item, ...incoming } : item)) : [...prev, incoming];
        setLiveSteps(next.map((s) => s.raw));
        return next;
      });
    };

    if (hasV050Api(statusRef.current, "sse") && typeof EventSource !== "undefined") {
      try {
        es = new EventSource("/api/runs/stream");
        setStreamMode("sse");
        es.onmessage = (ev) => handleSsePayload(ev.data, "message");
        for (const evName of [
          "status",
          "run_started",
          "step",
          "step_start",
          "step_done",
          "tool_start",
          "tool_done",
          "file_write",
          "file_written",
          "approval",
          "approval_parked",
          "run_cancelling",
          "run_finished",
          "done",
          "cancelled",
          "error",
        ]) {
          es.addEventListener(evName, (ev) => handleSsePayload((ev as MessageEvent).data, evName));
        }
        es.onerror = () => {
          if (es) {
            es.close();
            es = null;
          }
          startPolling();
        };
      } catch {
        startPolling();
      }
    } else {
      startPolling();
    }

    return () => {
      cancelled = true;
      window.clearInterval(elapsedTick);
      if (pollTimer !== null) window.clearInterval(pollTimer);
      if (es) es.close();
    };
  }, [busy, runStartedMs]);

  const cancelRun = useCallback(async () => {
    if (!busyRef.current) return;
    setCancelling(true);
    const runTarget = activeRunIdRef.current || threadRef.current || "active";
    if (hasV050Api(statusRef.current, "cancel")) {
      await call(`/api/runs/${encodeURIComponent(runTarget)}/cancel`, { method: "POST", body: {} });
    }
    abortRef.current?.abort();
    abortRef.current = null;
    busyRef.current = false;
    setBusy(false);
    setCancelling(false);
    toast("Run cancelled.");
  }, []);

  const sendPrompt = useCallback(
    async (rawText: string) => {
      if (busyRef.current) return;
      const text = rawText.trim();
      if (!text) return;
      const cur = threadRef.current;
      const ctrl = new AbortController();
      abortRef.current = ctrl;
      busyRef.current = true;
      setBusy(true);
      setRunStartedMs(Date.now());
      setLiveElapsedMs(0);
      setLiveSteps([]);
      setLiveTimeline([]);
      setPendingNote(false);
      setDraft("");
      store.set(PENDING, JSON.stringify({ thread: cur || "new", text }));
      setMessages((m) => [...m, { role: "user", content: text }]);
      const r = await call<Thread & { thread_id?: string }>("/api/chat", {
        method: "POST",
        body: { thread_id: cur || "", message: text },
        signal: ctrl.signal,
      });
      abortRef.current = null;
      store.set(PENDING, "");
      busyRef.current = false;
      setBusy(false);
      setLiveSteps([]);
      setLiveTimeline([]);
      if (r.ok && Array.isArray(r.data.messages)) {
        const id = r.data.id || r.data.thread_id || cur;
        setThread(id);
        if (id) store.set(THREAD_KEY, id);
        setMessages(r.data.messages);
        setRunFailed(false);
      } else if (!r.ok && r.status === 499) {
        setMessages((m) => [...m, { role: "step", content: "Note: Run cancelled by you before completion." }]);
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
    },
    [loadThreads, loadRecent, refreshCounts, loadFiles]
  );

  const send = useCallback(async () => {
    await sendPrompt(draft);
  }, [draft, sendPrompt]);

  const runRoutineNow = useCallback(
    async (routine: Routine) => {
      if (busyRef.current) return;
      if (hasV050Api(statusRef.current, "routines")) {
        const r = await call<{ id?: string; thread_id?: string; messages?: Msg[]; outcome?: string }>(
          `/api/routines/${encodeURIComponent(routine.id)}/run`,
          { method: "POST", body: {} }
        );
        if (r.ok) {
          toast.success(r.data.outcome || `Ran routine "${routine.name}".`);
          const tid = r.data.id || r.data.thread_id;
          if (tid && Array.isArray(r.data.messages)) {
            showRun(tid, r.data.messages);
          } else {
            loadThreads();
            loadRecent();
            refreshCounts();
            loadFiles();
          }
          await loadRoutines();
          return;
        }
      }
      // Graceful fallback: update local routine run metadata and execute its prompt live.
      const updated = loadLocalRoutines().map((x) =>
        x.id === routine.id ? { ...x, last_run_ts: Date.now(), last_status: "finished", run_count: (x.run_count ?? 0) + 1 } : x
      );
      saveLocalRoutines(updated);
      setRoutines((prev) =>
        prev.map((x) =>
          x.id === routine.id ? { ...x, last_run_ts: Date.now(), last_status: "finished", run_count: (x.run_count ?? 0) + 1 } : x
        )
      );
      await sendPrompt(routine.prompt);
    },
    [showRun, loadThreads, loadRecent, refreshCounts, loadFiles, loadRoutines, sendPrompt]
  );

  useEffect(() => {
    (async () => {
      await loadStatus();
      loadPeople();
      loadThreads();
      loadRecent();
      refreshCounts();
      loadFiles();
      loadSkills();
      loadRoutines();
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
    if (status && status !== "error") {
      loadStarters(String(status.person?.role ?? ""));
      loadSkills();
      loadRoutines();
    }
  }, [status, loadStarters, loadSkills, loadRoutines]);

  return {
    status, runFailed, people, threads, threadsError, states, threadId, messages,
    busy, activeRunId, liveElapsedMs, streamMode, cancelling, liveSteps, liveTimeline,
    draft, setDraft, pendingNote, sideError, starters, recovered, recent,
    todayCount, learnedCount, suiteCount, files, skills, routines,
    send, sendPrompt, cancelRun, openThread, newThread, switchPerson, refreshPerson, showRun,
    loadThreads, loadStatus, loadPeople, refreshCounts, loadRecent, loadStarters,
    loadFiles, uploadFile, uploadFiles, loadSkills, loadRoutines, createRoutine, deleteRoutine, runRoutineNow,
    setSideError, setRecovered,
  };
}
