// Today's Desk (Morning Brief, Drag-and-Drop Workspace & Operational Queue):
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import {
  CheckCircle2,
  Clock,
  FileCheck2,
  FileDown,
  FolderOpen,
  Search,
  ShieldAlert,
  Sparkles,
  Upload,
  X,
  XCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Pill } from "@/components/kibo-ui/pill";
import { Details, FileGlyph, WaitingPill } from "@/chat/Entry";
import { LiveRunTimeline } from "@/chat/Chat";
import { personInitials } from "@/shell/PersonSelect";
import {
  call,
  hasV050Api,
  type Brief,
  type BriefJob,
  type Decision,
  type LiveRunStep,
  type Offer,
  type Person,
  type Status,
  type ThreadRow,
  type TodayEvent,
  type TodayPayload,
  type WorkspaceFile,
} from "@/lib/api";
import { exportReceiptDownload } from "@/lib/receiptExport";
import { cn } from "@/lib/utils";
import { normalizeFiles, relTime, threadMs } from "@/lib/words";

const RECEIPTS_PAGE_SIZE = 20;
const EVENTS_PAGE_SIZE = 10;

type Data = {
  brief: Brief;
  decisions: Decision[];
  offers: Offer[];
  todayEvents: TodayEvent[];
  threads: ThreadRow[];
  extraReceipts: BriefJob[];
};

type DecisionCopy = {
  body: string;
  approve: string;
  /** null: a notice with a single acknowledgement. */
  deny: string | null;
  approvedToast: string;
  deniedToast: string;
  security: boolean;
};

const SECURITY_KINDS = new Set(["security", "security_notice", "allow_host", "memory_note", "publish_skill"]);

/** Plain consequence copy and button words for each kind of decision. */
function decisionCopy(d: Decision): DecisionCopy {
  const kind = String(d.kind ?? "");
  const detail = typeof d.detail === "string" ? d.detail.trim() : "";
  const security = SECURITY_KINDS.has(kind);
  if (/delete/.test(kind)) {
    return {
      body: "Approving deletes the file for good. Keeping it leaves the file where it is. Your agent has not touched it yet.",
      approve: "Approve",
      deny: "Keep the file",
      approvedToast: "Approved and done.",
      deniedToast: "You said no; kept safe.",
      security,
    };
  }
  if (kind === "security") {
    const src = d.source?.kind;
    const approve = src === "url" || src === "host" ? "Block that site" : src === "file" ? "Quarantine the file" : "Got it";
    return {
      body: detail || "Ergon blocked something suspicious while your agent worked. Nothing harmful went through.",
      approve,
      deny: src === "url" || src === "host" || src === "file" ? "Dismiss" : null,
      approvedToast: approve === "Got it" ? "Noted." : "Done — it is out of your agent's reach.",
      deniedToast: "Dismissed.",
      security,
    };
  }
  if (kind === "allow_host") {
    return {
      body: detail || "Ergon blocked an address on your private network that did not come from you.",
      approve: "Allow it",
      deny: "Keep blocked",
      approvedToast: "Allowed for future tasks.",
      deniedToast: "Kept blocked.",
      security,
    };
  }
  if (kind === "memory_note") {
    return {
      body: detail || "Your agent wanted to remember something Ergon thought you should see first.",
      approve: "Remember it",
      deny: "Discard",
      approvedToast: "Saved to your agent's memory.",
      deniedToast: "Discarded.",
      security,
    };
  }
  if (kind === "publish_skill") {
    return {
      body: detail || "Your agent wants to share a skill with everyone's library.",
      approve: "Publish",
      deny: "Don't publish",
      approvedToast: "Published to the library.",
      deniedToast: "Not published.",
      security,
    };
  }
  if (kind === "security_notice") {
    return {
      body: detail || "Ergon's tripwires caught something and handled it. Nothing real was exposed.",
      approve: "Got it",
      deny: null,
      approvedToast: "Noted.",
      deniedToast: "Noted.",
      security,
    };
  }
  return {
    body: "Your agent raised this while it worked and carried on. Your answer is recorded here; to have it act on the answer, tell it in the conversation.",
    approve: "Approve",
    deny: "Decline",
    approvedToast: "Approved and done.",
    deniedToast: "You said no; kept safe.",
    security,
  };
}

export function PaperSide({
  people,
  teamError,
  active,
  onSwitch,
  offers,
  learned,
  files,
  failed,
  onAct,
  onOpenFile,
  onOpenFilesPanel,
  onUploadMany,
}: {
  people: Person[] | null;
  teamError?: string;
  active: string;
  onSwitch: (id: string) => void;
  offers: Offer[] | null;
  learned: string[] | null;
  files: WorkspaceFile[];
  failed?: boolean;
  onAct: (path: string, done: string) => void;
  onOpenFile: (path: string) => void;
  onOpenFilesPanel: () => void;
  onUploadMany: (files: FileList | File[]) => Promise<number>;
}) {
  const [fileQuery, setFileQuery] = useState("");
  const sideUploadRef = useRef<HTMLInputElement>(null);

  const filteredFiles = useMemo(() => {
    const q = fileQuery.trim().toLowerCase();
    const list = q
      ? files.filter((f) => f.path.toLowerCase().includes(q) || f.ext.toLowerCase().includes(q))
      : files;
    return list.slice(0, 8);
  }, [files, fileQuery]);

  return (
    <div className="flex flex-col gap-6" data-answers="4">
      {/* Workspace Files Quick Search & Upload */}
      <section aria-labelledby="today-files-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <FolderOpen className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
            <h2 id="today-files-h" className="text-base font-medium text-ink">Workspace files</h2>
          </div>
          <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
            {files.length}
          </span>
        </div>

        {files.length > 0 && (
          <InputGroup className="rounded-sm bg-board/50 [&_input]:max-md:min-h-11">
            <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
              <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              id="today-file-search"
              type="search"
              aria-label="Filter workspace files"
              placeholder="Search files…"
              value={fileQuery}
              onChange={(e) => setFileQuery(e.target.value)}
              className="text-sm max-md:min-h-11"
            />
            {fileQuery && (
              <InputGroupAddon align="inline-end" className="pr-1.5">
                <Button
                  type="button"
                  size="icon-xs"
                  variant="ghost"
                  className="max-md:min-h-11 max-md:min-w-11"
                  aria-label="Clear workspace file search"
                  onClick={() => setFileQuery("")}
                >
                  <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                </Button>
              </InputGroupAddon>
            )}
          </InputGroup>
        )}

        {files.length === 0 ? (
          <p className="text-sm leading-5 text-ink-2">
            Drop CSV, Markdown, or JSON files anywhere on Today, or upload below.
          </p>
        ) : filteredFiles.length === 0 ? (
          <p className="text-sm text-ink-2">No files match &ldquo;{fileQuery}&rdquo;.</p>
        ) : (
          <ul className="divide-y divide-hairline rounded-sm border border-hairline bg-board/30">
            {filteredFiles.map((f) => (
              <li key={f.path} className="flex items-center justify-between gap-2 px-2.5 py-1.5">
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  aria-label={`Open ${f.path}`}
                  onClick={() => onOpenFile(f.path)}
                  className="h-auto min-h-11 w-full justify-between gap-2 px-1 text-left font-normal text-ink hover:bg-board md:min-h-8"
                >
                  <span className="flex min-w-0 items-center gap-1.5">
                    <FileGlyph name={f.path} className="size-3.5" />
                    <span className="mono truncate text-xs">{f.path}</span>
                  </span>
                  <span className="text-xs text-ink-3">Open</span>
                </Button>
              </li>
            ))}
          </ul>
        )}

        <div className="flex flex-wrap items-center gap-2">
          <input
            ref={sideUploadRef}
            type="file"
            multiple
            className="hidden"
            accept=".csv,.md,.txt,.json,.log,.tsv,.py,.ts,.tsx,.js,.rs,.toml,.yaml,.yml,.html"
            onChange={async (ev) => {
              if (ev.target.files?.length) await onUploadMany(ev.target.files);
              ev.target.value = "";
            }}
          />
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="max-md:min-h-11"
            onClick={() => sideUploadRef.current?.click()}
          >
            <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
            Upload files
          </Button>
          {files.length > 0 && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="max-md:min-h-11"
              onClick={onOpenFilesPanel}
            >
              Browse all ({files.length})
            </Button>
          )}
        </div>
      </section>

      <section id="offers-list" aria-labelledby="offers-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-surface p-4">
        <div className="flex items-center justify-between gap-2">
          <h2 id="offers-h" className="text-base font-medium text-ink">Offers from your agent</h2>
          {offers && offers.length > 0 && (
            <Pill variant="secondary" className="bg-tag-plan-bg px-2 py-0.5 text-xs font-medium text-tag-plan-fg">
              <Sparkles className="size-3" strokeWidth={1.5} aria-hidden />
              <span className="mono">{offers.length}</span>
            </Pill>
          )}
        </div>
        {!offers && (
          <p className="text-sm text-ink-2">
            {failed ? "Offers could not be loaded. Retry the desk to try again." : "Loading offers…"}
          </p>
        )}
        <div className="flex flex-col gap-3">
          {(offers ?? []).map((o) => (
            <div
              key={o.id}
              className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-board/60 p-3.5"
            >
              <p className="text-sm font-medium leading-5 text-ink">{String(o.title ?? "Suggested capability")}</p>
              <p className="text-sm leading-5 text-ink-2">{String(o.detail ?? "")}</p>
              <div className="flex flex-wrap gap-2 pt-0.5">
                <Button
                  size="sm"
                  className="max-md:min-h-11"
                  onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/accept`, "Added to your suite as a one-tap app.")}
                >
                  Make it one tap
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  onClick={() => onAct(`/api/offers/${encodeURIComponent(o.id)}/dismiss`, "Offer dismissed.")}
                >
                  No, thanks
                </Button>
              </div>
            </div>
          ))}
        </div>
        {offers && !offers.length && (
          <p className="text-sm leading-5 text-ink-2">
            No offers right now. Your agent proposes a one-tap app when you ask for the same kind of work three times.
          </p>
        )}
      </section>

      <section aria-labelledby="learned-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <h2 id="learned-h" className="text-base font-medium text-ink">What your agent has learned</h2>
        {!learned && <p className="text-sm text-ink-2">{failed ? "Could not be loaded with the brief." : "Loading…"}</p>}
        {learned && learned.length > 0 && (
          <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm leading-5 text-ink-2">
            {learned.map((l, i) => <li key={i} className="text-ink">{String(l)}</li>)}
          </ul>
        )}
        {learned && !learned.length && (
          <p className="text-sm leading-5 text-ink-2">
            Nothing learned yet. Preferences and corrections your agent picks up appear here.
          </p>
        )}
      </section>

      <section aria-labelledby="team-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-4">
        <h2 id="team-h" className="text-base font-medium text-ink">Team</h2>
        {!people && <p role={teamError ? "alert" : undefined} className="text-sm text-ink-2">{teamError || "Loading the team…"}</p>}
        {people && !people.length && (
          <p className="text-sm text-ink-2">No people yet. Add someone from the rail and their agent keeps its own workspace and memory.</p>
        )}
        {people && people.length > 0 && (
          <ul className="flex flex-col gap-1">
            {people.map((p) => {
              const isCurrent = p.id === active;
              const pName = String(p?.name ?? "").trim() || "Unnamed";
              const pRole = String(p?.role ?? "").trim() || "Personal workspace";
              return (
                <li key={p.id}>
                  <Button
                    variant="ghost"
                    aria-current={isCurrent ? "true" : undefined}
                    onClick={() => onSwitch(p.id)}
                    className={cn(
                      "h-auto sm:h-auto min-h-11 w-full justify-start gap-2.5 whitespace-normal px-2.5 py-1.5 text-left font-normal",
                      isCurrent ? "bg-board" : "hover:bg-board"
                    )}
                  >
                    <span
                      aria-hidden
                      className={cn(
                        "grid size-7 shrink-0 place-items-center rounded-full text-xs font-medium",
                        isCurrent ? "bg-ink text-canvas" : "bg-blue-soft text-ink"
                      )}
                    >
                      {personInitials(pName)}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className={cn("block truncate text-sm text-ink", isCurrent && "font-medium")}>{pName}</span>
                      <span className="block truncate text-xs text-ink-2">{pRole}</span>
                    </span>
                  </Button>
                </li>
              );
            })}
          </ul>
        )}
      </section>
    </div>
  );
}

export function usePaper(onChanged: () => void, status?: Status | null | "error") {
  const [data, setData] = useState<Data | null>(null);
  const [failed, setFailed] = useState(false);

  const load = useCallback(async () => {
    const [b, d, o, td, t] = await Promise.all([
      call<Brief>("/api/brief"),
      call<{ decisions: Decision[] }>("/api/decisions"),
      call<{ offers: Offer[] }>("/api/offers"),
      call<TodayPayload>("/api/today"),
      call<{ threads: ThreadRow[] }>("/api/threads"),
    ]);
    const shaped = (x: unknown, key: string) => !!x && typeof x === "object" && Array.isArray((x as Record<string, unknown>)[key]);
    if (
      !b.ok || !d.ok || !o.ok || !td.ok || !t.ok
      || !shaped(b.data, "finished")
      || !shaped(d.data, "decisions")
      || !shaped(o.data, "offers")
      || !shaped(td.data, "events")
      || !shaped(t.data, "threads")
    ) {
      setFailed(true);
      return;
    }

    let extraReceipts: BriefJob[] = [];
    if (status && hasV050Api(status, "receipts_export")) {
      const rec = await call<{ receipts?: BriefJob[] }>("/api/receipts?limit=5000");
      if (rec.ok && Array.isArray(rec.data.receipts)) {
        extraReceipts = rec.data.receipts.filter((x): x is BriefJob => !!x && typeof x === "object");
      }
    }

    setFailed(false);
    setData({
      brief: b.data,
      decisions: (d.data.decisions ?? []).filter((x): x is Decision => !!x && typeof x === "object"),
      offers: (o.data.offers ?? []).filter((x): x is Offer => !!x && typeof x === "object"),
      todayEvents: (td.data.events ?? []).filter((x): x is TodayEvent => !!x && typeof x === "object"),
      threads: (t.data.threads ?? []).filter((x): x is ThreadRow => !!x && typeof x === "object"),
      extraReceipts,
    });
  }, [status]);

  useEffect(() => { load(); }, [load]);

  const act = useCallback(async (path: string, done: string) => {
    const r = await call<{ outcome?: string }>(path, { method: "POST", body: {} });
    if (!r.ok) {
      toast.error(r.error ? "That did not go through: " + r.error : "That did not go through. Try again.");
      return;
    }
    toast.success(typeof r.data?.outcome === "string" && r.data.outcome ? r.data.outcome : done);
    await load();
    onChanged();
  }, [load, onChanged]);

  return { data, failed, load, act };
}

export function PaperFace({
  paper,
  status,
  personName,
  dateLabel,
  people,
  teamError,
  activePerson,
  files,
  busy,
  liveTimeline,
  liveElapsedMs,
  runStartedMs,
  streamMode,
  cancelling,
  onCancel,
  onSwitchPerson,
  onAsk,
  onOpen,
  onOpenFile,
  onOpenFilesPanel,
  onUploadMany,
  onExportAuditBundle,
}: {
  paper: ReturnType<typeof usePaper>;
  status: Status | null | "error";
  personName: string;
  dateLabel: string;
  people: Person[] | null;
  teamError: string;
  activePerson: string;
  files: WorkspaceFile[];
  busy: boolean;
  liveTimeline: LiveRunStep[];
  liveElapsedMs: number;
  runStartedMs?: number;
  streamMode: "sse" | "poll";
  cancelling: boolean;
  onCancel: () => void;
  onSwitchPerson: (id: string) => void;
  onAsk: () => void;
  onOpen: (threadId: string) => void;
  onOpenFile: (path: string) => void;
  onOpenFilesPanel: () => void;
  onUploadMany: (files: FileList | File[]) => Promise<number>;
  onExportAuditBundle?: (format: "json" | "md") => Promise<void>;
}) {
  const { data, failed, load, act } = paper;
  const [dragOver, setDragOver] = useState(false);
  const [exportingAudit, setExportingAudit] = useState(false);
  const [receiptQuery, setReceiptQuery] = useState("");
  const [onlyWithFiles, setOnlyWithFiles] = useState(false);
  const [receiptPage, setReceiptPage] = useState(0);
  const [eventsPage, setEventsPage] = useState(0);

  // Most urgent first: a task Ergon stopped for safety, then real choices
  // (deletions, questions, approvals), then informational notices —
  // newest first within each rank.
  const urgency = (d: Decision): number => {
    if (d.severity === "critical" || d.stopped) return 3;
    if (d.kind === "security_notice" || (d.kind === "security" && !d.source)) return 1;
    return 2;
  };
  const pending = data
    ? data.decisions
        .slice()
        .reverse()
        .filter((d) => d.status === "pending")
        .map((d, i) => ({ d, i }))
        .sort((a, b) => urgency(b.d) - urgency(a.d) || a.i - b.i)
        .map((x) => x.d)
    : [];
  const settled = data ? data.decisions.slice().reverse().filter((d) => d.status !== "pending") : [];
  const threadOf = useCallback(
    (x: BriefJob) => x.thread_id ?? data?.threads.find((t) => t.title === x.task)?.id,
    [data?.threads]
  );
  const when = (x: BriefJob) => {
    const id = threadOf(x);
    const ms = typeof x.ts === "number" ? x.ts : (id ? threadMs(id) : null);
    return ms ? relTime(ms) : String(x.date ?? "");
  };
  const waitingThreads = useMemo(() => new Set(pending.map((d) => d.thread_id).filter(Boolean)), [pending]);

  const finished = useMemo(() => {
    const briefFin = (data?.brief.finished ?? [])
      .filter((x): x is BriefJob => !!x && typeof x === "object")
      .filter((x) => { const id = threadOf(x); return !id || !waitingThreads.has(id); });
    const extra = data?.extraReceipts ?? [];
    if (!extra.length) return briefFin;
    const matchKey = (x: BriefJob) =>
      x.thread_id
        ? `t:${x.thread_id}|${x.task}`
        : x.id
          ? `id:${x.id}`
          : `tk:${x.task}|${x.ts || 0}`;
    const byKey = new Map<string, number>();
    const merged = briefFin.map((x, idx) => {
      byKey.set(matchKey(x), idx);
      return { ...x };
    });
    const seenIds = new Set(merged.map((x) => x.id).filter(Boolean));
    for (const r of extra) {
      const meta = r as BriefJob & { finished?: boolean; answered?: boolean };
      if (meta.finished === false || meta.answered === true) continue;
      const tid = threadOf(r);
      if (tid && waitingThreads.has(tid)) continue;
      const k = matchKey(r);
      const existingIdx = byKey.get(k);
      if (existingIdx !== undefined) {
        if (!merged[existingIdx].id && r.id) merged[existingIdx].id = r.id;
        if (!merged[existingIdx].ts && r.ts) merged[existingIdx].ts = r.ts;
        if (!merged[existingIdx].sha256 && r.sha256) merged[existingIdx].sha256 = r.sha256;
        if (r.id) seenIds.add(r.id);
        continue;
      }
      if (r.id && seenIds.has(r.id)) continue;
      if (r.id) seenIds.add(r.id);
      byKey.set(k, merged.length);
      merged.push(r);
    }
    return merged;
  }, [data?.brief.finished, data?.extraReceipts, threadOf, waitingThreads]);

  const stopped = useMemo(
    () => (data?.brief.stopped ?? []).filter((x): x is BriefJob => !!x && typeof x === "object"),
    [data?.brief.stopped]
  );

  useEffect(() => {
    setReceiptPage(0);
  }, [receiptQuery, onlyWithFiles, finished.length]);

  const filteredFinished = useMemo(() => {
    const q = receiptQuery.trim().toLowerCase();
    return finished.filter((x) => {
      const fList = normalizeFiles(x.files);
      if (onlyWithFiles && fList.length === 0) return false;
      if (!q) return true;
      return (
        String(x.task ?? "").toLowerCase().includes(q) ||
        String(x.summary ?? "").toLowerCase().includes(q) ||
        fList.some((f) => f.toLowerCase().includes(q))
      );
    });
  }, [finished, receiptQuery, onlyWithFiles]);

  const totalReceiptPages = Math.max(1, Math.ceil(filteredFinished.length / RECEIPTS_PAGE_SIZE));
  const safeReceiptPage = Math.min(receiptPage, totalReceiptPages - 1);
  const visibleFinished = filteredFinished.slice(
    safeReceiptPage * RECEIPTS_PAGE_SIZE,
    (safeReceiptPage + 1) * RECEIPTS_PAGE_SIZE
  );

  const allEvents = data?.todayEvents ?? [];
  const totalEventPages = Math.max(1, Math.ceil(allEvents.length / EVENTS_PAGE_SIZE));
  const safeEventPage = Math.min(eventsPage, totalEventPages - 1);
  const visibleEvents = allEvents.slice(safeEventPage * EVENTS_PAGE_SIZE, (safeEventPage + 1) * EVENTS_PAGE_SIZE);

  const lead = pending[0];
  const leadCopy = lead ? decisionCopy(lead) : null;
  const n = finished.length;
  const w = pending.length;
  const totalSavedFiles = useMemo(
    () => finished.reduce((acc, job) => acc + normalizeFiles(job.files).length, 0),
    [finished]
  );

  const useServerExport = hasV050Api(status, "receipts_export");

  const handleRowReceiptExport = async (job: BriefJob, idx: number, format: "md" | "json") => {
    const id = job.id || job.thread_id || `receipt-${idx}`;
    const res = await exportReceiptDownload(
      {
        id,
        task: String(job.task ?? "Completed task"),
        person: personName || "Active employee",
        status: "finished",
        summary: String(job.summary ?? ""),
        ts: typeof job.ts === "number" ? job.ts : Date.now(),
        files: normalizeFiles(job.files),
      },
      format,
      useServerExport && Boolean(job.id || job.thread_id)
    );
    toast.success(`Exported ${res.filename} (sha256: ${res.sha256.slice(0, 12)}…)`);
  };

  const sideColumn = (
    <PaperSide
      people={people}
      teamError={teamError}
      active={activePerson}
      onSwitch={onSwitchPerson}
      offers={data ? data.offers.slice().reverse().filter((o) => o.status === "pending") : null}
      learned={data ? (Array.isArray(data.brief.learned_recently) ? data.brief.learned_recently : []) : null}
      files={files}
      failed={failed}
      onAct={act}
      onOpenFile={onOpenFile}
      onOpenFilesPanel={onOpenFilesPanel}
      onUploadMany={onUploadMany}
    />
  );

  return (
    <div
      className="relative mx-auto w-full max-w-[1160px] px-4 pb-14 pt-5 md:px-6"
      onDragOver={(e) => {
        if (e.dataTransfer?.types?.includes("Files")) {
          e.preventDefault();
          setDragOver(true);
        }
      }}
      onDragLeave={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setDragOver(false);
      }}
      onDrop={async (e) => {
        if (!e.dataTransfer?.files?.length) return;
        e.preventDefault();
        setDragOver(false);
        await onUploadMany(e.dataTransfer.files);
      }}
    >
      {dragOver && (
        <div
          id="today-drop-overlay"
          role="status"
          className="pointer-events-none fixed inset-4 z-30 flex flex-col items-center justify-center gap-2 rounded-panel border-2 border-dashed border-blue bg-blue-soft p-6 text-center text-ink shadow-lift"
        >
          <Upload className="size-7 text-ink" strokeWidth={1.5} aria-hidden />
          <p className="text-base font-medium text-ink">Drop files onto Today to upload to your workspace</p>
          <p className="mono text-xs text-ink-2">CSV, Markdown, JSON, TXT, TSV, LOG (multi-file supported)</p>
        </div>
      )}

      <div className="grid grid-cols-[minmax(0,1fr)] gap-8 xl:grid-cols-[minmax(0,1fr)_312px]">
        <main className="grid min-w-0 grid-cols-[minmax(0,1fr)] content-start gap-7" aria-label="Today's paper">
          {failed && (
            <div role="alert" className="rounded-entry border border-hairline bg-surface p-5">
              <h1 className="text-title font-medium text-ink">Today&apos;s paper could not be loaded</h1>
              <p className="mt-1 text-sm text-ink-2">Check that Ergon is running, then try again.</p>
              <Button className="mt-3 max-md:min-h-11" variant="outline" onClick={load}>Retry</Button>
            </div>
          )}

          {!failed && !data && (
            <div aria-busy="true" className="grid gap-4">
              <div className="h-12 w-4/5 rounded-sm bg-board" />
              <div className="h-20 rounded-card bg-board" />
              <div className="h-32 rounded-card bg-board" />
              <span className="sr-only">Loading today&apos;s paper</span>
            </div>
          )}

          {data && (
            <>
              <header className="grid gap-3 border-b border-hairline pb-5">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <p id="dateline" className="text-sm font-medium text-ink-2">
                    {dateLabel}{personName ? `, for ${personName === "You" ? "you" : personName}` : ""}
                  </p>
                  <div className="flex flex-wrap items-center gap-2">
                    {onExportAuditBundle && (
                      <Button
                        id="export-audit-bundle-btn"
                        type="button"
                        variant="outline"
                        size="sm"
                        className="max-md:min-h-11"
                        disabled={exportingAudit}
                        onClick={async () => {
                          setExportingAudit(true);
                          await onExportAuditBundle("json");
                          setExportingAudit(false);
                        }}
                      >
                        <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                        {exportingAudit ? "Exporting…" : "Export Audit Bundle"}
                      </Button>
                    )}
                    <Button variant="outline" size="sm" className="max-md:min-h-11" data-answers="3" onClick={onAsk}>
                      Ask your agent for something
                    </Button>
                  </div>
                </div>
                <h1
                  id="brief-headline"
                  data-answers="1"
                  className="text-[28px] leading-[1.15] font-medium tracking-[-0.03em] text-ink [text-wrap:wrap] md:text-[36px]"
                >
                  {n ? (
                    <span>{n} {n === 1 ? "thing" : "things"} finished, </span>
                  ) : w ? (
                    <span>Nothing finished yet, </span>
                  ) : (
                    <span>Nothing finished yet.</span>
                  )}
                  {w ? (
                    <span>{w} {w === 1 ? "waits" : "wait"} on you.</span>
                  ) : n ? (
                    <span>nothing waits on you.</span>
                  ) : null}
                </h1>
                <p className="max-w-[64ch] text-ui leading-6 text-ink-2">
                  {lead
                    ? `The decision below is the only item parked on you${w > 1 ? " first" : " today"}. Autonomous tasks finished are filed underneath with their workspace files and SHA-256 receipts.`
                    : n
                      ? "Your agent has nothing waiting on your approval. Completed work and workspace files are filed below."
                      : "Describe a task in plain words or drop files onto this desk. Your agent runs real workspace tools and asks here before deleting anything."}
                </p>
                <div className="mono flex flex-wrap items-center gap-x-4 gap-y-1 pt-1 text-xs text-ink-2">
                  <span className="inline-flex items-center gap-1.5 text-ink">
                    <span aria-hidden className="size-2 rounded-full bg-dot-green" />
                    {n} finished
                  </span>
                  <span className="inline-flex items-center gap-1.5 text-ink">
                    <span aria-hidden className="size-2 rounded-full bg-dot-amber" />
                    {w} waiting
                  </span>
                  {stopped.length > 0 && (
                    <span className="inline-flex items-center gap-1.5 text-ink">
                      <span aria-hidden className="size-2 rounded-full bg-dot-red" />
                      {stopped.length} stopped
                    </span>
                  )}
                  <span>{totalSavedFiles} {totalSavedFiles === 1 ? "file" : "files"} saved</span>
                </div>
              </header>

              {busy && (
                <LiveRunTimeline
                  steps={liveTimeline}
                  elapsedMs={liveElapsedMs}
                  startedMs={runStartedMs}
                  streamMode={streamMode}
                  cancelling={cancelling}
                  onCancel={onCancel}
                />
              )}

              {/* Waiting on you Approval Queue */}
              <div id="decisions-list">
                {lead ? (
                  <section
                    key={lead.id}
                    aria-labelledby="lead-h"
                    className="grid gap-3.5 rounded-card border border-hairline bg-blue-soft p-5"
                  >
                      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                        <h2 id="lead-h" className="flex min-w-0 items-start gap-2 text-[22px] leading-snug font-medium tracking-[-0.025em] text-ink [overflow-wrap:anywhere]">
                          {leadCopy?.security && (
                            <ShieldAlert
                              className={cn("mt-1 size-5 shrink-0", lead.severity === "critical" ? "text-dot-red" : "text-dot-amber")}
                              strokeWidth={1.5}
                              aria-label="Security"
                            />
                          )}
                          <span className="min-w-0">{String(lead.summary ?? "Decision waiting on you")}</span>
                        </h2>
                        <WaitingPill />
                      </div>
                      <p className="max-w-[60ch] whitespace-pre-line text-sm leading-6 text-ink-2">{leadCopy?.body}</p>
                      <div className="flex flex-wrap gap-2">
                        <Button
                          className="min-h-11 px-4"
                          onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/approve`, leadCopy?.approvedToast ?? "Approved and done.")}
                        >
                          {leadCopy?.approve ?? "Approve"}
                        </Button>
                        {leadCopy?.deny !== null && (
                          <Button
                            className="min-h-11 px-4"
                            variant="outline"
                            onClick={() => act(`/api/decisions/${encodeURIComponent(lead.id)}/deny`, leadCopy?.deniedToast ?? "You said no; kept safe.")}
                          >
                            {leadCopy?.deny ?? "Decline"}
                          </Button>
                        )}
                      </div>
                      {w > 1 && (
                        <p className="text-xs text-ink-2">
                          <span className="mono">{w - 1}</span> more {w - 1 === 1 ? "decision waits" : "decisions wait"} after this one.
                        </p>
                      )}
                    </section>
                  ) : n > 0 ? (
                    <p
                      key="quiet"
                      className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2"
                    >
                      Nothing waits on you. Your agent asks here before it deletes anything.
                    </p>
                  ) : null}
                {settled.length > 0 && (
                  <div className="mt-3">
                    <Details label={`Recently settled (${settled.length})`}>
                      <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface px-3.5">
                        {settled.slice(0, 5).map((s) => (
                          <li key={s.id} className="flex flex-wrap items-center justify-between gap-2 py-2.5 text-sm">
                            <span className="min-w-0 truncate font-medium text-ink">{String(s.summary ?? "")}</span>
                            <span className="text-xs text-ink-2">{String(s.outcome ?? s.status ?? "")}</span>
                          </li>
                        ))}
                      </ul>
                    </Details>
                  </div>
                )}
              </div>

              {/* Virtualized / Paginated Finished Work & Receipts Ledger */}
              {n > 0 && (
                <section aria-labelledby="finished-h" className="grid gap-3">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <h2 id="finished-h" className="text-title font-medium text-ink">Finished</h2>
                    <span className="mono text-xs text-ink-2">
                      {filteredFinished.length === n
                        ? `${n} ${n === 1 ? "receipt" : "receipts"}`
                        : `${filteredFinished.length} of ${n} ${n === 1 ? "receipt" : "receipts"}`}
                    </span>
                  </div>

                  {n >= 3 && (
                    <div className="flex flex-wrap items-center gap-2">
                      <InputGroup className="min-w-[200px] flex-1 rounded-sm bg-surface [&_input]:max-md:min-h-11">
                        <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
                          <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
                        </InputGroupAddon>
                        <InputGroupInput
                          id="finished-search"
                          type="search"
                          aria-label="Filter finished receipts"
                          placeholder="Filter receipts…"
                          value={receiptQuery}
                          onChange={(e) => setReceiptQuery(e.target.value)}
                          className="text-sm max-md:min-h-11"
                        />
                        {receiptQuery && (
                          <InputGroupAddon align="inline-end" className="pr-1.5">
                            <Button
                              type="button"
                              size="icon-xs"
                              variant="ghost"
                              className="max-md:min-h-11 max-md:min-w-11"
                              aria-label="Clear receipt filter"
                              onClick={() => setReceiptQuery("")}
                            >
                              <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                            </Button>
                          </InputGroupAddon>
                        )}
                      </InputGroup>
                      <Button
                        type="button"
                        size="sm"
                        variant={onlyWithFiles ? "default" : "outline"}
                        aria-pressed={onlyWithFiles}
                        className="max-md:min-h-11"
                        onClick={() => setOnlyWithFiles((v) => !v)}
                      >
                        With saved files
                      </Button>
                    </div>
                  )}

                  {visibleFinished.length === 0 ? (
                    <p className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2">
                      No finished receipts match your filter.
                    </p>
                  ) : (
                    <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                      {visibleFinished.map((x, i) => {
                        const globalIdx = safeReceiptPage * RECEIPTS_PAGE_SIZE + i;
                        const id = threadOf(x);
                        const jobFiles = normalizeFiles(x.files);
                        const taskTitle = String(x.task ?? "Completed task");
                        return (
                          <li
                            key={(x.id ?? x.thread_id ?? taskTitle) + "-" + globalIdx}
                            className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 px-4 py-3.5 md:grid-cols-[104px_minmax(0,1fr)_auto]"
                          >
                            <span className="mono col-span-2 text-xs text-ink-2 md:col-span-1">{when(x)}</span>
                            <div className="min-w-0">
                              <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                              <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1.5 text-xs text-ink-2">
                                {jobFiles.length ? (
                                  <>
                                    <span>Saved</span>
                                    {jobFiles.map((f, k) => (
                                      <Button
                                        key={f + k}
                                        size="xs"
                                        variant="outline"
                                        aria-label={`Open ${f}`}
                                        onClick={() => onOpenFile(f)}
                                        className="num max-md:min-h-11 max-w-full gap-1.5 rounded-chip border-hairline bg-board/60 px-2 text-xs font-normal text-ink hover:bg-board"
                                      >
                                        <FileGlyph name={f} />
                                        <span className="truncate">{f}</span>
                                      </Button>
                                    ))}
                                  </>
                                ) : (
                                  "Nothing saved to your workspace"
                                )}
                              </div>
                            </div>
                            <div className="flex flex-wrap items-center justify-end gap-1">
                              <Button
                                type="button"
                                size="xs"
                                variant="ghost"
                                className="max-md:min-h-11 gap-1 text-xs text-ink-2 hover:text-ink"
                                aria-label={`Export SHA-256 receipt for ${taskTitle}`}
                                onClick={() => handleRowReceiptExport(x, globalIdx, "md")}
                              >
                                <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                                <span>Receipt</span>
                              </Button>
                              {id && (
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  className="min-h-11"
                                  aria-label={`Open the conversation: ${taskTitle}`}
                                  onClick={() => onOpen(id)}
                                >
                                  Open
                                </Button>
                              )}
                            </div>
                          </li>
                        );
                      })}
                    </ol>
                  )}

                  {totalReceiptPages > 1 && (
                    <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
                      <span className="mono">
                        Showing {safeReceiptPage * RECEIPTS_PAGE_SIZE + 1}–{Math.min((safeReceiptPage + 1) * RECEIPTS_PAGE_SIZE, filteredFinished.length)} of {filteredFinished.length}
                      </span>
                      <div className="flex items-center gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          disabled={safeReceiptPage === 0}
                          onClick={() => setReceiptPage((p) => Math.max(0, p - 1))}
                        >
                          Prev
                        </Button>
                        <span className="mono px-1">
                          {safeReceiptPage + 1} / {totalReceiptPages}
                        </span>
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          disabled={safeReceiptPage >= totalReceiptPages - 1}
                          onClick={() => setReceiptPage((p) => Math.min(totalReceiptPages - 1, p + 1))}
                        >
                          Next
                        </Button>
                      </div>
                    </div>
                  )}
                </section>
              )}

              {/* Stopped / Did not finish */}
              {stopped.length > 0 && (
                <section aria-labelledby="stopped-h" className="grid gap-3">
                  <h2 id="stopped-h" className="text-title font-medium text-ink">Did not finish</h2>
                  <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                    {stopped.slice(0, 20).map((x, i) => {
                      const id = threadOf(x);
                      const taskTitle = String(x.task ?? "Stopped task");
                      return (
                        <li
                          key={(x.thread_id ?? taskTitle) + i}
                          className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 px-4 py-3.5 md:grid-cols-[104px_minmax(0,1fr)_auto]"
                        >
                          <span className="mono col-span-2 text-xs text-ink-2 md:col-span-1">{when(x)}</span>
                          <div className="min-w-0">
                            <p className="text-ui font-medium leading-6 text-ink [overflow-wrap:anywhere]">{taskTitle}</p>
                            {x.summary && <p className="mt-0.5 text-sm leading-5 text-ink-2 [overflow-wrap:anywhere]">{String(x.summary)}</p>}
                          </div>
                          {id && (
                            <Button
                              size="sm"
                              variant="ghost"
                              className="min-h-11"
                              aria-label={`Open the conversation: ${taskTitle}`}
                              onClick={() => onOpen(id)}
                            >
                              Open
                            </Button>
                          )}
                        </li>
                      );
                    })}
                  </ol>
                </section>
              )}

              {/* Today so far (/api/today paginated chronological activity stream) */}
              <section aria-labelledby="today-stream-h" className="grid gap-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h2 id="today-stream-h" className="text-title font-medium text-ink">Today so far</h2>
                  <span className="mono text-xs text-ink-2">{allEvents.length} {allEvents.length === 1 ? "event" : "events"}</span>
                </div>
                {allEvents.length === 0 ? (
                  <p className="rounded-card border border-hairline bg-surface px-4 py-3 text-sm text-ink-2">
                    No activity logged yet today. New tool runs, decisions, and capability offers appear here in order.
                  </p>
                ) : (
                  <>
                    <ol className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                      {visibleEvents.map((ev, i) => {
                        const evFiles = normalizeFiles(ev.files);
                        const kind = String(ev.kind ?? "work");
                        const EventIcon = kind === "decision" ? Clock : kind === "offer" ? Sparkles : kind === "security" ? ShieldAlert : CheckCircle2;
                        const iconCls =
                          kind === "decision"
                            ? "text-dot-amber"
                            : kind === "offer"
                              ? "text-dot-clay"
                              : kind === "security"
                                ? ev.severity === "critical" ? "text-dot-red" : "text-dot-amber"
                                : "text-dot-green";
                        const timeStr = typeof ev.ts === "number" ? relTime(ev.ts) : "";
                        return (
                          <li key={String(ev.ts ?? i) + "-" + i} className="flex items-start gap-3 px-4 py-3">
                            {kind === "stopped" ? (
                              <XCircle className="mt-0.5 size-4 shrink-0 text-dot-red" strokeWidth={1.5} aria-hidden />
                            ) : (
                              <EventIcon className={cn("mt-0.5 size-4 shrink-0", iconCls)} strokeWidth={1.5} aria-hidden />
                            )}
                            <div className="min-w-0 flex-1">
                              <div className="flex flex-wrap items-baseline justify-between gap-2">
                                <p className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{String(ev.title ?? "Activity")}</p>
                                {timeStr && <span className="mono shrink-0 text-xs text-ink-3">{timeStr}</span>}
                              </div>
                              {ev.detail && <p className="mt-0.5 text-sm text-ink-2 [overflow-wrap:anywhere]">{String(ev.detail)}</p>}
                              {evFiles.length > 0 && (
                                <div className="mt-1.5 flex flex-wrap items-center gap-2">
                                  {evFiles.map((f, idx) => (
                                    <Button
                                      key={f + idx}
                                      size="xs"
                                      variant="outline"
                                      aria-label={`Open ${f}`}
                                      onClick={() => onOpenFile(f)}
                                      className="mono max-md:min-h-11 max-w-full gap-1 rounded-chip border-hairline bg-board px-2 text-xs font-normal text-ink"
                                    >
                                      <FileGlyph name={f} className="size-3.5" />
                                      <span className="truncate">{f}</span>
                                    </Button>
                                  ))}
                                </div>
                              )}
                            </div>
                          </li>
                        );
                      })}
                    </ol>

                    {totalEventPages > 1 && (
                      <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-ink-2">
                        <span className="mono">
                          Events {safeEventPage * EVENTS_PAGE_SIZE + 1}–{Math.min((safeEventPage + 1) * EVENTS_PAGE_SIZE, allEvents.length)} of {allEvents.length}
                        </span>
                        <div className="flex items-center gap-1.5">
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            className="max-md:min-h-11"
                            disabled={safeEventPage === 0}
                            onClick={() => setEventsPage((p) => Math.max(0, p - 1))}
                          >
                            Prev
                          </Button>
                          <span className="mono px-1">
                            {safeEventPage + 1} / {totalEventPages}
                          </span>
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            className="max-md:min-h-11"
                            disabled={safeEventPage >= totalEventPages - 1}
                            onClick={() => setEventsPage((p) => Math.min(totalEventPages - 1, p + 1))}
                          >
                            Next
                          </Button>
                        </div>
                      </div>
                    )}
                  </>
                )}
              </section>
            </>
          )}
        </main>

        <aside aria-label="Offers, memory and team">{sideColumn}</aside>
      </div>
    </div>
  );
}
