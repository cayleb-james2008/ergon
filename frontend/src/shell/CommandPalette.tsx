// Global Command Palette (Cmd+K / Ctrl+K) backed by COSS UI Command (cmdk):
// Instant keyboard-navigable (ArrowUp/ArrowDown/Enter/Esc) search & jump across:
// - Quick Actions (New Task, Upload File, Export Receipt .md/.json, Skills, Routines, Settings, Views)
// - All 12 built-in workflows (jump/run in Today)
// - Workspace Files (500+ ready, capped visible slice)
// - Receipts & Conversations (5,000+ ready + debounced server GET /api/search?q=... on v0.5.0+)
// - Recurring Routines (Run Now or manage)
import { memo, useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowRight,
  Blocks,
  CalendarClock,
  FileCheck2,
  FileDown,
  FolderOpen,
  GitBranch,
  History,
  MessageSquarePlus,
  Newspaper,
  Play,
  ReceiptText,
  RefreshCw,
  SlidersHorizontal,
  Sparkles,
  Upload,
  Users,
  Webhook,
} from "lucide-react";
import { toast } from "sonner";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from "@/components/ui/command";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { FileGlyph } from "@/chat/Entry";
import {
  call,
  hasV050Api,
  reindexVault,
  searchVault,
  type BriefJob,
  type Routine,
  type SearchHit,
  type Status,
  type ThreadRow,
  type VaultChunkHit,
  type VaultFileHit,
  type VaultIndexStats,
  type VaultSearchMode,
  type WorkspaceFile,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import type { SkillMeta } from "@/lib/skillsCatalog";
import { detectWorkspaceWorkflow, extractCitations, relTime } from "@/lib/words";

function buildChunkJumpTarget(path: string, ch: VaultChunkHit): string {
  const sheetPart = ch.sheet ? `?sheet=${encodeURIComponent(ch.sheet)}` : "";
  const linePart =
    ch.line_start > 0
      ? `#L${ch.line_start}${ch.line_end > ch.line_start ? `-L${ch.line_end}` : ""}`
      : "";
  return `${path}${sheetPart}${linePart}`;
}

/** Lightweight fuzzy scorer: exact substring > multi-token match > ordered subsequence. */
function fuzzyScore(query: string, ...fields: Array<string | undefined>): number {
  const q = query.trim().toLowerCase();
  if (!q) return 1;
  const hay = fields
    .filter((f): f is string => typeof f === "string" && f.length > 0)
    .join(" ")
    .toLowerCase();
  if (!hay) return 0;
  if (hay.includes(q)) return 100 - Math.min(40, hay.indexOf(q));
  const tokens = q.split(/\s+/).filter(Boolean);
  if (tokens.length > 1 && tokens.every((t) => hay.includes(t))) return 60;
  let qi = 0;
  let gaps = 0;
  for (let i = 0; i < hay.length && qi < q.length; i++) {
    if (hay[i] === q[qi]) qi++;
    else if (qi > 0) gaps++;
  }
  if (qi === q.length) return Math.max(10, 45 - Math.floor(gaps / 4));
  return 0;
}

export const CommandPalette = memo(function CommandPalette({
  open,
  onOpenChange,
  status,
  files,
  receipts,
  threads,
  skills,
  routines,
  onNewTask,
  onUploadFile,
  onExportLatestReceipt,
  onExportAuditBundle,
  onOpenWorkspaceSwitcher,
  onOpenSettings,
  onOpenFiles,
  onOpenHistory,
  onOpenSkills,
  onOpenRoutines,
  onOpenConnectors,
  onOpenFile,
  onOpenThread,
  onRunSkill,
  onRunPrompt,
  onRunRoutine,
  onSwitchFace,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  status: Status | null | "error";
  files: WorkspaceFile[];
  receipts: BriefJob[];
  threads: ThreadRow[];
  skills: SkillMeta[];
  routines: Routine[];
  onNewTask: () => void;
  onUploadFile: () => void;
  onExportLatestReceipt: (format: "md" | "json") => void;
  onExportAuditBundle?: (format: "json" | "md") => void;
  onOpenWorkspaceSwitcher?: () => void;
  onOpenSettings: () => void;
  onOpenFiles: () => void;
  onOpenHistory?: () => void;
  onOpenSkills: () => void;
  onOpenRoutines: () => void;
  onOpenConnectors?: () => void;
  onOpenFile: (path: string) => void;
  onOpenThread: (id: string) => void;
  onRunSkill: (skill: SkillMeta) => void;
  onRunPrompt?: (prompt: string) => void;
  onRunRoutine: (routine: Routine) => void;
  onSwitchFace: (face: "chat" | "paper" | "graph") => void;
}) {
  const [query, setQuery] = useState("");
  const [searchMode, setSearchMode] = useState<VaultSearchMode>("hybrid");
  const [remoteHits, setRemoteHits] = useState<SearchHit[]>([]);
  const [fileHits, setFileHits] = useState<VaultFileHit[]>([]);
  const [indexStats, setIndexStats] = useState<VaultIndexStats | null>(null);
  const [reindexing, setReindexing] = useState(false);
  const [searchNonce, setSearchNonce] = useState(0);
  const [ledgerReceipts, setLedgerReceipts] = useState<BriefJob[]>([]);
  const [searching, setSearching] = useState(false);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setRemoteHits([]);
      setFileHits([]);
      setSearching(false);
      return;
    }
    let cancelled = false;
    call<{ receipts?: BriefJob[] }>("/api/receipts?limit=30").then((r) => {
      if (!cancelled && r.ok && Array.isArray(r.data.receipts)) {
        setLedgerReceipts(r.data.receipts);
      }
    });
    if (hasV050Api(status, "search")) {
      searchVault({ q: "", limit: 1 }).then((r) => {
        if (!cancelled && r.ok && r.data.index_stats) {
          setIndexStats(r.data.index_stats);
        }
      });
    }
    return () => {
      cancelled = true;
    };
  }, [open, status]);

  // Query server indexed search (GET /api/search?q=...) on v0.5.0+ when typing >= 2 chars.
  useEffect(() => {
    const q = query.trim();
    if (!open || q.length < 2 || !hasV050Api(status, "search")) {
      setRemoteHits([]);
      setFileHits([]);
      setSearching(false);
      return;
    }
    let cancelled = false;
    const controller = new AbortController();
    setSearching(true);
    const timer = window.setTimeout(async () => {
      const r = await searchVault({
        q,
        mode: searchMode,
        limit: 16,
        signal: controller.signal,
      });
      if (cancelled) return;
      setSearching(false);
      if (!r.ok) return;
      if (r.data.index_stats) {
        setIndexStats(r.data.index_stats);
      }

      const nextFileHits: VaultFileHit[] = [];
      if (Array.isArray(r.data.file_hits)) {
        for (const fh of r.data.file_hits) {
          if (fh && typeof fh.path === "string" && fh.path.trim()) {
            nextFileHits.push({
              ...fh,
              chunks: Array.isArray(fh.chunks) ? fh.chunks : [],
            });
          }
        }
      }

      const hits: SearchHit[] = [];
      if (Array.isArray(r.data.results)) {
        for (const h of r.data.results) {
          if (h && typeof h === "object" && typeof h.title === "string") {
            if (h.kind === "file" && h.path && Array.isArray(h.chunks) && h.chunks.length > 0 && !nextFileHits.some((x) => x.path === h.path)) {
              nextFileHits.push({ path: h.path, score: h.score, chunks: h.chunks });
            }
            hits.push(h);
          }
        }
      }
      if (Array.isArray(r.data.files)) {
        for (const f of r.data.files.slice(0, 8)) {
          const fp = String(f?.path ?? f?.name ?? "").trim();
          if (fp && Array.isArray(f?.chunks) && f.chunks.length > 0 && !nextFileHits.some((x) => x.path === fp)) {
            nextFileHits.push({ path: fp, name: f.name, size: f.size, mtime_ms: f.mtime_ms, ext: f.ext, chunks: f.chunks });
          }
          if (fp && !hits.some((x) => x.kind === "file" && x.path === fp)) {
            hits.push({ kind: "file", id: `f:${fp}`, title: fp, path: fp });
          }
        }
      }
      if (Array.isArray(r.data.receipts)) {
        for (const rec of r.data.receipts.slice(0, 8)) {
          const title = String(rec?.task ?? "").trim();
          if (title && !hits.some((x) => x.kind === "receipt" && x.title === title)) {
            hits.push({
              kind: "receipt",
              id: String(rec?.id ?? rec?.thread_id ?? title),
              title,
              subtitle: String(rec?.summary ?? ""),
              thread_id: rec?.thread_id,
              ts: rec?.ts,
            });
          }
        }
      }
      if (Array.isArray(r.data.memory)) {
        for (const mem of r.data.memory.slice(0, 6)) {
          const text = String(mem?.text ?? "").trim();
          if (text && !hits.some((x) => x.kind === "memory" && x.title === text)) {
            hits.push({
              kind: "memory",
              id: `mem-${String(mem?.line ?? hits.length)}`,
              title: text,
              subtitle: typeof mem?.line === "number" ? `memory.md:L${mem.line}` : "memory.md",
            });
          }
        }
      }
      setFileHits(nextFileHits.slice(0, 10));
      setRemoteHits(hits.slice(0, 16));
    }, 110);
    return () => {
      cancelled = true;
      controller.abort();
      window.clearTimeout(timer);
    };
  }, [open, query, searchMode, searchNonce, status]);

  const handleReindexVault = async () => {
    if (reindexing) return;
    setReindexing(true);
    const r = await reindexVault();
    setReindexing(false);
    if (!r.ok) {
      toast.error(r.error || "Could not reindex workspace vault.");
      return;
    }
    if (r.data.index_stats) {
      setIndexStats(r.data.index_stats);
    } else {
      setIndexStats((prev) => ({
        total_files: r.data.indexed_files,
        total_chunks: r.data.total_chunks,
        indexed_bytes: r.data.indexed_bytes ?? prev?.indexed_bytes ?? 0,
        semantic_enabled: r.data.semantic_enabled ?? prev?.semantic_enabled ?? true,
      }));
    }
    toast.success(`Indexed ${r.data.indexed_files} files (${r.data.total_chunks} chunks) in ${r.data.elapsed_ms} ms.`);
    setSearchNonce((n) => n + 1);
  };

  const qLower = query.trim().toLowerCase();

  // Pre-filter and cap large lists (500+ files, 5,000+ receipts) with fuzzy scoring so cmdk stays instant at 60fps.
  const matchedFiles = useMemo(() => {
    if (!qLower) return files.slice(0, 12);
    return files
      .map((f) => ({ f, score: fuzzyScore(qLower, f.path, f.ext) }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 12)
      .map((x) => x.f);
  }, [files, qLower]);

  const matchedCitations = useMemo(() => {
    const out: Array<{ raw: string; path: string; line: number; endLine?: number; sourceTask: string }> = [];
    const seen = new Set<string>();
    const combined = [...ledgerReceipts, ...receipts];
    for (const r of combined.slice(0, 80)) {
      const text = `${String(r.task ?? "")}\n${String(r.summary ?? "")}`;
      for (const c of extractCitations(text)) {
        if (seen.has(c.raw)) continue;
        seen.add(c.raw);
        if (!qLower || fuzzyScore(qLower, c.raw, c.path, r.task) > 0) {
          out.push({ ...c, sourceTask: String(r.task ?? "") });
        }
        if (out.length >= 8) return out;
      }
    }
    for (const h of remoteHits) {
      const text = `${String(h.title ?? "")}\n${String(h.subtitle ?? "")}`;
      for (const c of extractCitations(text)) {
        if (seen.has(c.raw)) continue;
        seen.add(c.raw);
        if (!qLower || fuzzyScore(qLower, c.raw, c.path, h.title) > 0) {
          out.push({ ...c, sourceTask: String(h.title ?? "") });
        }
        if (out.length >= 8) return out;
      }
    }
    return out;
  }, [ledgerReceipts, receipts, remoteHits, qLower]);

  const matchedReceipts = useMemo(() => {
    const combined: { id: string; title: string; subtitle: string; thread_id?: string; ts?: number; score: number }[] = [];
    const seen = new Set<string>();
    for (const r of receipts) {
      const title = String(r.task ?? "").trim();
      if (!title) continue;
      const key = `${r.thread_id || ""}|${title}`;
      if (seen.has(key)) continue;
      seen.add(key);
      const score = qLower ? fuzzyScore(qLower, title, String(r.summary ?? ""), (r.files ?? []).join(" ")) : 1;
      if (score > 0) {
        combined.push({
          id: r.id || r.thread_id || key,
          title,
          subtitle: Array.isArray(r.files) && r.files.length ? r.files.slice(0, 2).join(", ") : (r.date || ""),
          thread_id: r.thread_id,
          ts: r.ts,
          score,
        });
      }
      if (combined.length >= 40) break;
    }
    if (combined.length < 10) {
      for (const t of threads) {
        const title = String(t.title ?? "").trim() || "Untitled task";
        const key = `${t.id}|${title}`;
        if (seen.has(key)) continue;
        seen.add(key);
        const score = qLower ? fuzzyScore(qLower, title, t.id) : 1;
        if (score > 0) {
          combined.push({
            id: t.id,
            title,
            subtitle: typeof t.messages === "number" ? `${t.messages} msgs` : "Conversation",
            thread_id: t.id,
            score,
          });
        }
        if (combined.length >= 40) break;
      }
    }
    return combined.sort((a, b) => b.score - a.score).slice(0, 10);
  }, [receipts, threads, qLower]);

  const matchedSkills = useMemo(() => {
    if (!qLower) return skills.slice(0, 12);
    return skills
      .map((s) => ({ s, score: fuzzyScore(qLower, s.title, s.id, s.category, s.description) }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 12)
      .map((x) => x.s);
  }, [skills, qLower]);

  const matchedRoutines = useMemo(() => {
    if (!qLower) return routines.slice(0, 8);
    return routines
      .map((r) => ({
        r,
        score: fuzzyScore(qLower, r.name, r.prompt, r.schedule, (r.steps ?? []).join(" "), (r.expected_inputs ?? []).join(" ")),
      }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 8)
      .map((x) => x.r);
  }, [routines, qLower]);

  const recommended = useMemo(() => detectWorkspaceWorkflow(files), [files]);

  const quickActions = useMemo(() => {
    const all = [
      { id: "action-new-task", keywords: "new task start chat composer", label: "New task", badge: "Chat", icon: MessageSquarePlus, run: onNewTask },
      ...(recommended && onRunPrompt
        ? [{ id: "action-recommended-workflow", keywords: `recommended auto workflow ${recommended.label} ${recommended.file}`, label: `${recommended.label}: ${recommended.file}`, badge: "1-click", icon: Sparkles, run: () => onRunPrompt(recommended.prompt) }]
        : []),
      { id: "action-upload-file", keywords: "upload file csv markdown drop workspace", label: "Upload file to workspace", badge: "Files", icon: Upload, run: onUploadFile },
      { id: "action-export-receipt-md", keywords: "export receipt sha256 markdown download", label: "Export latest receipt (SHA-256 Markdown)", badge: ".md", icon: FileDown, run: () => onExportLatestReceipt("md") },
      { id: "action-export-receipt-json", keywords: "export receipt sha256 json download", label: "Export latest receipt (SHA-256 JSON)", badge: ".json", icon: FileDown, run: () => onExportLatestReceipt("json") },
      ...(onExportAuditBundle
        ? [
            { id: "action-export-audit-json", keywords: "export full audit bundle sha256 compliance json", label: "Export Full Audit Bundle (JSON)", badge: "Bundle", icon: FileCheck2, run: () => onExportAuditBundle("json") },
            { id: "action-export-audit-md", keywords: "export audit bundle report markdown", label: "Export Audit Bundle Report (Markdown)", badge: ".md", icon: FileCheck2, run: () => onExportAuditBundle("md") },
          ]
        : []),
      ...(onOpenWorkspaceSwitcher
        ? [{ id: "action-switch-workspace", keywords: "switch workspace person employee desk team", label: "Switch employee workspace", badge: "Desk", icon: Users, run: onOpenWorkspaceSwitcher }]
        : []),
      { id: "action-open-settings", keywords: "open settings appearance theme model", label: "Open Settings (Model & Appearance)", badge: "Panel", icon: SlidersHorizontal, run: onOpenSettings },
      { id: "action-open-skills", keywords: "browse workflows skills catalog 12", label: `Browse all ${skills.length} built-in workflows`, badge: String(skills.length), icon: Sparkles, run: onOpenSkills },
      { id: "action-open-routines", keywords: "routines recurring schedule cron automate", label: "Manage recurring routines", badge: String(routines.length), icon: CalendarClock, run: onOpenRoutines },
      ...(onOpenConnectors
        ? [{ id: "action-open-connectors", keywords: "connectors webhooks watched folders outbound actions", label: "Manage connectors & webhooks", badge: "Panel", icon: Webhook, run: onOpenConnectors }]
        : []),
      { id: "action-open-files", keywords: "workspace files browser", label: "Open Workspace files", badge: "⌘I", icon: FolderOpen, run: onOpenFiles },
      ...(onOpenHistory
        ? [{ id: "action-open-history", keywords: "file version history diff restore snapshot", label: "Open file version history & diff", badge: "Diff", icon: History, run: onOpenHistory }]
        : []),
      { id: "action-face-today", keywords: "today paper desk brief", label: "Jump to Today's desk", badge: "View", icon: Newspaper, run: () => onSwitchFace("paper") },
      { id: "action-face-graph", keywords: "work graph map links", label: "Jump to Work Graph", badge: "View", icon: GitBranch, run: () => onSwitchFace("graph") },
    ];
    if (!qLower) return all;
    return all.filter((a) => a.keywords.toLowerCase().includes(qLower) || a.label.toLowerCase().includes(qLower));
  }, [qLower, recommended, onRunPrompt, onNewTask, onUploadFile, onExportLatestReceipt, onExportAuditBundle, onOpenWorkspaceSwitcher, onOpenSettings, skills.length, onOpenSkills, routines.length, onOpenRoutines, onOpenFiles, onOpenHistory, onSwitchFace]);

  const uniqueRemoteHits = useMemo(() => {
    const filePaths = new Set(matchedFiles.map((f) => f.path));
    for (const fh of fileHits) filePaths.add(fh.path);
    const recIds = new Set(matchedReceipts.map((r) => r.id));
    const recTitles = new Set(matchedReceipts.map((r) => r.title));
    return remoteHits.filter((h) => {
      if (h.kind === "file" && h.path && filePaths.has(h.path)) return false;
      if (h.kind === "receipt" && (recIds.has(h.id) || recTitles.has(h.title))) return false;
      return true;
    });
  }, [remoteHits, fileHits, matchedFiles, matchedReceipts]);

  const ranRef = useRef(false);
  const runAndClose = (fn: () => void) => {
    ranRef.current = true;
    onOpenChange(false);
    setTimeout(fn, 30);
  };

  return (
    <Dialog open={open} onOpenChange={(next) => { if (next) ranRef.current = false; onOpenChange(next); }}>
      <DialogContent
        id="command-palette-dialog"
        onCloseAutoFocus={(e) => {
          if (ranRef.current) {
            e.preventDefault();
            ranRef.current = false;
          }
        }}
        className="max-h-[82vh] max-w-2xl overflow-hidden border border-hairline bg-surface p-0 text-ink shadow-lift"
      >
        <DialogTitle className="sr-only">Command palette</DialogTitle>
        <DialogDescription className="sr-only">
          Search workspace files, receipts, 12 built-in workflows, recurring routines, and quick actions.
        </DialogDescription>

        <Command
          shouldFilter={false}
          className="bg-surface text-ink [&_[cmdk-group-heading]]:px-2.5 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-ink-2"
        >
          <CommandInput
            data-cmdk-input="command-palette"
            aria-label="Search command palette"
            value={query}
            onValueChange={setQuery}
            placeholder="Search files, passages, workflows, receipts…"
            className="h-12 pr-12 text-sm text-ink placeholder:text-ink-3"
          />
          <div
            id="palette-vault-toolbar"
            className="flex flex-wrap items-center justify-between gap-2 border-b border-hairline bg-board/40 px-3 py-1.5"
          >
            <SegmentedControl aria-label="Vault search mode">
              {(["hybrid", "bm25", "exact"] as const).map((m) => (
                <SegmentedControlItem
                  key={m}
                  data-id={m}
                  aria-pressed={searchMode === m}
                  data-checked={searchMode === m ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-7"
                  onClick={() => setSearchMode(m)}
                >
                  {m === "hybrid" ? "Hybrid" : m === "bm25" ? "BM25" : "Exact"}
                </SegmentedControlItem>
              ))}
            </SegmentedControl>
            <div className="flex flex-wrap items-center gap-1.5">
              <Badge id="palette-index-stats" variant="outline" size="sm" className="text-xs text-ink-2 tabular-nums">
                {indexStats
                  ? `${indexStats.total_chunks} chunks, ${indexStats.total_files} files`
                  : `${files.length} files`}
              </Badge>
              <Button
                id="palette-reindex-btn"
                type="button"
                size="xs"
                variant="outline"
                className="min-h-11 gap-1.5 text-xs md:min-h-7"
                disabled={reindexing}
                onClick={() => void handleReindexVault()}
              >
                <RefreshCw className={cn("size-3", reindexing && "animate-spin")} strokeWidth={1.5} aria-hidden />
                <span>{reindexing ? "Reindexing…" : "Reindex Vault"}</span>
              </Button>
            </div>
          </div>
          <CommandList className="max-h-[58vh] p-1.5">
            <CommandEmpty className="py-6 text-center text-sm text-ink-2">
              {searching ? "Searching indexed workspace…" : "No matching files, receipts, workflows, or actions."}
            </CommandEmpty>

            {qLower.length >= 2 && fileHits.length > 0 && (
              <CommandGroup heading="Ranked vault passages">
                {fileHits.flatMap((fh) => {
                  const chunks = fh.chunks.slice(0, 3);
                  if (chunks.length === 0) {
                    return [
                      <CommandItem
                        key={`vh-${fh.path}`}
                        value={`vault-file-${fh.path}`}
                        onSelect={() => runAndClose(() => onOpenFile(fh.path))}
                        className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                      >
                        <span className="flex min-w-0 items-center gap-2">
                          <FileGlyph name={fh.path} className="size-4 shrink-0" />
                          <span className="truncate font-medium">{fh.path}</span>
                        </span>
                      </CommandItem>,
                    ];
                  }
                  return chunks.map((ch) => (
                    <CommandItem
                      key={`vc-${fh.path}-${ch.chunk_id}`}
                      value={`vault-chunk-${fh.path}-${ch.chunk_id}`}
                      data-vault-chunk={ch.chunk_id}
                      data-vault-path={fh.path}
                      data-line-start={ch.line_start}
                      onSelect={() => runAndClose(() => onOpenFile(buildChunkJumpTarget(fh.path, ch)))}
                      className="min-h-11 cursor-pointer flex-col items-start gap-1 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board"
                    >
                      <div className="flex w-full flex-wrap items-center justify-between gap-1.5">
                        <span className="flex min-w-0 items-center gap-1.5">
                          <FileGlyph name={fh.path} className="size-4 shrink-0" />
                          <span className="truncate font-medium text-ink">{fh.path}</span>
                          {ch.heading && (
                            <span className="truncate text-xs text-ink-2">{ch.heading}</span>
                          )}
                        </span>
                        <span className="flex shrink-0 flex-wrap items-center gap-1">
                          {typeof ch.page === "number" && ch.page > 0 && (
                            <Badge variant="outline" size="sm" className="text-xs tabular-nums">
                              Page {ch.page}
                            </Badge>
                          )}
                          {ch.sheet && (
                            <Badge variant="outline" size="sm" className="text-xs">
                              #{ch.sheet}
                            </Badge>
                          )}
                          <Badge variant="outline" size="sm" className="text-xs tabular-nums">
                            L{ch.line_start}{ch.line_end > ch.line_start ? `-L${ch.line_end}` : ""}
                          </Badge>
                        </span>
                      </div>
                      <p className="line-clamp-2 w-full text-xs leading-5 text-ink-2">
                        {ch.excerpt}
                      </p>
                    </CommandItem>
                  ));
                })}
              </CommandGroup>
            )}

            {quickActions.length > 0 && (
              <>
                {qLower.length >= 2 && fileHits.length > 0 && <CommandSeparator className="my-1 bg-hairline" />}
                <CommandGroup heading="Quick actions">
                {quickActions.map((act) => {
                  const Icon = act.icon;
                  return (
                    <CommandItem
                      key={act.id}
                      value={act.id}
                      onSelect={() => runAndClose(act.run)}
                      className="min-h-11 cursor-pointer gap-2.5 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <Icon className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
                      <span>{act.label}</span>
                      <CommandShortcut className="mono text-xs text-ink-2">{act.badge}</CommandShortcut>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
              </>
            )}

            {matchedFiles.length > 0 && (
              <>
                {quickActions.length > 0 && <CommandSeparator className="my-1 bg-hairline" />}
                <CommandGroup heading={`Workspace files (${files.length})`}>
                  {matchedFiles.map((f) => (
                    <CommandItem
                      key={f.path}
                      value={`file-${f.path}`}
                      onSelect={() => runAndClose(() => onOpenFile(f.path))}
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <FileGlyph name={f.path} className="size-4 shrink-0" />
                        <span className="mono truncate">{f.path}</span>
                      </span>
                      <span className="mono shrink-0 text-xs text-ink-2">{f.ext || "file"}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}

            {matchedSkills.length > 0 && (
              <>
                <CommandSeparator className="my-1 bg-hairline" />
                <CommandGroup heading={`Built-in workflows (${skills.length})`}>
                  {matchedSkills.map((sk) => (
                    <CommandItem
                      key={sk.id}
                      value={`skill-${sk.id}`}
                      onSelect={() => runAndClose(() => onRunSkill(sk))}
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <Blocks className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                        <span className="truncate font-medium">{sk.title}</span>
                        <Badge variant="outline" size="sm" className="shrink-0 text-xs">
                          {sk.category}
                        </Badge>
                      </span>
                      <span className="mono flex shrink-0 items-center gap-1 text-xs text-ink-2">
                        <span>{sk.filename}</span>
                        <ArrowRight className="size-3.5" strokeWidth={1.5} aria-hidden />
                      </span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}

            {matchedRoutines.length > 0 && (
              <>
                <CommandSeparator className="my-1 bg-hairline" />
                <CommandGroup heading={`Playbooks & routines (${routines.length})`}>
                  {matchedRoutines.map((rt) => (
                    <CommandItem
                      key={rt.id}
                      value={`routine-${rt.id}`}
                      onSelect={() => runAndClose(() => onRunRoutine(rt))}
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <Play className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                        <span className="truncate">{rt.name}</span>
                        {(rt.steps?.length ?? 0) > 1 && (
                          <Badge variant="outline" size="sm" className="mono shrink-0 text-xs">
                            {rt.steps!.length} steps
                          </Badge>
                        )}
                      </span>
                      <span className="mono shrink-0 text-xs text-ink-2">{rt.schedule}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}

            {matchedCitations.length > 0 && (
              <>
                <CommandSeparator className="my-1 bg-hairline" />
                <CommandGroup heading="Source citations ([file:L##])">
                  {matchedCitations.map((cit) => (
                    <CommandItem
                      key={`cit-${cit.raw}`}
                      value={`citation-${cit.raw}`}
                      onSelect={() =>
                        runAndClose(() =>
                          onOpenFile(`${cit.path}#L${cit.line}${cit.endLine ? `-L${cit.endLine}` : ""}`)
                        )
                      }
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <FileGlyph name={cit.path} className="size-4 shrink-0" />
                        <span className="mono truncate">{cit.raw}</span>
                      </span>
                      <span className="truncate text-xs text-ink-2">{cit.sourceTask}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}

            {matchedReceipts.length > 0 && (
              <>
                <CommandSeparator className="my-1 bg-hairline" />
                <CommandGroup heading={`Receipts & conversations (${receipts.length})`}>
                  {matchedReceipts.map((rec) => (
                    <CommandItem
                      key={rec.id}
                      value={`receipt-${rec.id}`}
                      onSelect={() =>
                        runAndClose(() => {
                          if (rec.thread_id) onOpenThread(rec.thread_id);
                          else onSwitchFace("paper");
                        })
                      }
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <ReceiptText className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                        <span className="truncate">{rec.title}</span>
                      </span>
                      <span className="mono shrink-0 text-xs text-ink-2">
                        {rec.ts ? relTime(rec.ts) : rec.subtitle}
                      </span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}

            {uniqueRemoteHits.length > 0 && (
              <>
                <CommandSeparator className="my-1 bg-hairline" />
                <CommandGroup heading="Indexed server matches">
                  {uniqueRemoteHits.map((hit) => (
                    <CommandItem
                      key={`remote-${hit.kind}-${hit.id}`}
                      value={`remote-${hit.kind}-${hit.id}`}
                      onSelect={() =>
                        runAndClose(() => {
                          if (hit.kind === "file" && hit.path) onOpenFile(hit.path);
                          else if (hit.thread_id) onOpenThread(hit.thread_id);
                          else onSwitchFace("paper");
                        })
                      }
                      className="min-h-11 cursor-pointer justify-between gap-2 rounded-sm px-2.5 py-2 text-sm text-ink data-[selected=true]:bg-board md:min-h-9"
                    >
                      <span className="truncate">{hit.title}</span>
                      <Badge variant="outline" size="sm" className="mono shrink-0 text-xs">
                        {hit.kind}
                      </Badge>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </>
            )}
          </CommandList>

          <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline bg-board/50 px-3 py-2 text-xs text-ink-2">
            <span>Use ↑↓ to move, Enter to run, Esc to close</span>
            <span className="mono">
              {files.length} files, {skills.length} workflows, {receipts.length} receipts
            </span>
          </div>
        </Command>
      </DialogContent>
    </Dialog>
  );
});
