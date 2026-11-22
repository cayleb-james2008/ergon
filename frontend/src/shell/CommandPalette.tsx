// Global Command Palette (Cmd+K / Ctrl+K) backed by COSS UI Command (cmdk):
// Instant keyboard-navigable (ArrowUp/ArrowDown/Enter/Esc) search & jump across:
// - Quick Actions (New Task, Upload File, Export Receipt .md/.json, Skills, Routines, Settings, Views)
// - All 12 Rune Skills (jump/run in Today)
// - Workspace Files (500+ ready, capped visible slice)
// - Receipts & Conversations (5,000+ ready + debounced server GET /api/search?q=... on v0.5.0+)
// - Recurring Routines (Run Now or manage)
import { useEffect, useMemo, useState } from "react";
import {
  ArrowRight,
  Blocks,
  CalendarClock,
  FileDown,
  FolderOpen,
  GitBranch,
  MessageSquarePlus,
  Newspaper,
  Play,
  ReceiptText,
  SlidersHorizontal,
  Sparkles,
  Upload,
} from "lucide-react";
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
import { FileGlyph } from "@/chat/Entry";
import {
  call,
  hasV050Api,
  type BriefJob,
  type Routine,
  type SearchHit,
  type Status,
  type ThreadRow,
  type WorkspaceFile,
} from "@/lib/api";
import type { SkillMeta } from "@/lib/skillsCatalog";
import { relTime } from "@/lib/words";

export function CommandPalette({
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
  onOpenSettings,
  onOpenFiles,
  onOpenSkills,
  onOpenRoutines,
  onOpenFile,
  onOpenThread,
  onRunSkill,
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
  onOpenSettings: () => void;
  onOpenFiles: () => void;
  onOpenSkills: () => void;
  onOpenRoutines: () => void;
  onOpenFile: (path: string) => void;
  onOpenThread: (id: string) => void;
  onRunSkill: (skill: SkillMeta) => void;
  onRunRoutine: (routine: Routine) => void;
  onSwitchFace: (face: "chat" | "paper" | "graph") => void;
}) {
  const [query, setQuery] = useState("");
  const [remoteHits, setRemoteHits] = useState<SearchHit[]>([]);
  const [searching, setSearching] = useState(false);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setRemoteHits([]);
      setSearching(false);
    }
  }, [open]);

  // Query server indexed search (GET /api/search?q=...) on v0.5.0+ when typing >= 2 chars.
  useEffect(() => {
    const q = query.trim();
    if (!open || q.length < 2 || !hasV050Api(status, "search")) {
      setRemoteHits([]);
      setSearching(false);
      return;
    }
    let cancelled = false;
    setSearching(true);
    const timer = window.setTimeout(async () => {
      const r = await call<{
        results?: SearchHit[];
        files?: { path?: string; name?: string }[];
        receipts?: { id?: string; thread_id?: string; task?: string; summary?: string; ts?: number }[];
        memory?: { line?: string; text?: string }[];
      }>("/api/search?q=" + encodeURIComponent(q) + "&limit=16");
      if (cancelled) return;
      setSearching(false);
      if (!r.ok) return;
      const hits: SearchHit[] = [];
      if (Array.isArray(r.data.results)) {
        for (const h of r.data.results) {
          if (h && typeof h === "object" && typeof h.title === "string") hits.push(h);
        }
      }
      if (Array.isArray(r.data.files)) {
        for (const f of r.data.files.slice(0, 8)) {
          const fp = String(f?.path ?? f?.name ?? "").trim();
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
      setRemoteHits(hits.slice(0, 16));
    }, 110);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [open, query, status]);

  const qLower = query.trim().toLowerCase();

  // Pre-filter and cap large lists (500+ files, 5,000+ receipts) so cmdk stays instant at 60fps.
  const matchedFiles = useMemo(() => {
    const list = qLower
      ? files.filter((f) => f.path.toLowerCase().includes(qLower) || f.ext.toLowerCase().includes(qLower))
      : files;
    return list.slice(0, 12);
  }, [files, qLower]);

  const matchedReceipts = useMemo(() => {
    const combined: { id: string; title: string; subtitle: string; thread_id?: string; ts?: number }[] = [];
    const seen = new Set<string>();
    for (const r of receipts) {
      const title = String(r.task ?? "").trim();
      if (!title) continue;
      const key = `${r.thread_id || ""}|${title}`;
      if (seen.has(key)) continue;
      seen.add(key);
      if (!qLower || title.toLowerCase().includes(qLower) || String(r.summary ?? "").toLowerCase().includes(qLower)) {
        combined.push({
          id: r.id || r.thread_id || key,
          title,
          subtitle: Array.isArray(r.files) && r.files.length ? r.files.slice(0, 2).join(", ") : (r.date || ""),
          thread_id: r.thread_id,
          ts: r.ts,
        });
      }
      if (combined.length >= 10) break;
    }
    if (combined.length < 10) {
      for (const t of threads) {
        const title = String(t.title ?? "").trim() || "Untitled task";
        const key = `${t.id}|${title}`;
        if (seen.has(key)) continue;
        seen.add(key);
        if (!qLower || title.toLowerCase().includes(qLower)) {
          combined.push({
            id: t.id,
            title,
            subtitle: typeof t.messages === "number" ? `${t.messages} msgs` : "Conversation",
            thread_id: t.id,
          });
        }
        if (combined.length >= 10) break;
      }
    }
    return combined;
  }, [receipts, threads, qLower]);

  const matchedSkills = useMemo(() => {
    const list = qLower
      ? skills.filter(
          (s) =>
            s.title.toLowerCase().includes(qLower) ||
            s.id.toLowerCase().includes(qLower) ||
            s.category.toLowerCase().includes(qLower) ||
            s.description.toLowerCase().includes(qLower)
        )
      : skills;
    return list.slice(0, 12);
  }, [skills, qLower]);

  const matchedRoutines = useMemo(() => {
    const list = qLower
      ? routines.filter(
          (r) =>
            r.name.toLowerCase().includes(qLower) ||
            r.prompt.toLowerCase().includes(qLower) ||
            r.schedule.toLowerCase().includes(qLower)
        )
      : routines;
    return list.slice(0, 8);
  }, [routines, qLower]);

  const quickActions = useMemo(() => {
    const all = [
      { id: "action-new-task", keywords: "new task start chat composer", label: "New task", badge: "Chat", icon: MessageSquarePlus, run: onNewTask },
      { id: "action-upload-file", keywords: "upload file csv markdown drop workspace", label: "Upload file to workspace", badge: "Files", icon: Upload, run: onUploadFile },
      { id: "action-export-receipt-md", keywords: "export receipt sha256 markdown download", label: "Export latest receipt (SHA-256 Markdown)", badge: ".md", icon: FileDown, run: () => onExportLatestReceipt("md") },
      { id: "action-export-receipt-json", keywords: "export receipt sha256 json download", label: "Export latest receipt (SHA-256 JSON)", badge: ".json", icon: FileDown, run: () => onExportLatestReceipt("json") },
      { id: "action-open-settings", keywords: "open settings appearance theme model", label: "Open Settings (Model & Appearance)", badge: "Panel", icon: SlidersHorizontal, run: onOpenSettings },
      { id: "action-open-skills", keywords: "browse skills rune catalog 12", label: `Browse all ${skills.length} Rune skills`, badge: String(skills.length), icon: Sparkles, run: onOpenSkills },
      { id: "action-open-routines", keywords: "routines recurring schedule cron automate", label: "Manage recurring routines", badge: String(routines.length), icon: CalendarClock, run: onOpenRoutines },
      { id: "action-open-files", keywords: "workspace files browser", label: "Open Workspace files", badge: String(files.length), icon: FolderOpen, run: onOpenFiles },
      { id: "action-face-today", keywords: "today paper desk brief", label: "Jump to Today's desk", badge: "View", icon: Newspaper, run: () => onSwitchFace("paper") },
      { id: "action-face-graph", keywords: "work graph map links", label: "Jump to Work Graph", badge: "View", icon: GitBranch, run: () => onSwitchFace("graph") },
    ];
    if (!qLower) return all;
    return all.filter((a) => a.keywords.includes(qLower) || a.label.toLowerCase().includes(qLower));
  }, [qLower, onNewTask, onUploadFile, onExportLatestReceipt, onOpenSettings, skills.length, onOpenSkills, routines.length, onOpenRoutines, files.length, onOpenFiles, onSwitchFace]);

  const uniqueRemoteHits = useMemo(() => {
    const filePaths = new Set(matchedFiles.map((f) => f.path));
    const recIds = new Set(matchedReceipts.map((r) => r.id));
    const recTitles = new Set(matchedReceipts.map((r) => r.title));
    return remoteHits.filter((h) => {
      if (h.kind === "file" && h.path && filePaths.has(h.path)) return false;
      if (h.kind === "receipt" && (recIds.has(h.id) || recTitles.has(h.title))) return false;
      return true;
    });
  }, [remoteHits, matchedFiles, matchedReceipts]);

  const runAndClose = (fn: () => void) => {
    onOpenChange(false);
    setTimeout(fn, 20);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        id="command-palette-dialog"
        className="max-h-[82vh] max-w-2xl overflow-hidden border border-hairline bg-surface p-0 text-ink shadow-lift"
      >
        <DialogTitle className="sr-only">Command palette</DialogTitle>
        <DialogDescription className="sr-only">
          Search workspace files, receipts, 12 Rune skills, recurring routines, and quick actions.
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
            placeholder="Search files, receipts, 12 Rune skills, routines, or actions…"
            className="h-12 pr-12 text-sm text-ink placeholder:text-ink-3"
          />
          <CommandList className="max-h-[58vh] p-1.5">
            <CommandEmpty className="py-6 text-center text-sm text-ink-2">
              {searching ? "Searching indexed workspace…" : "No matching files, receipts, skills, or actions."}
            </CommandEmpty>

            {quickActions.length > 0 && (
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
                <CommandGroup heading={`Rune skills (${skills.length})`}>
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
                <CommandGroup heading={`Routines (${routines.length})`}>
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
                      </span>
                      <span className="mono shrink-0 text-xs text-ink-2">{rt.schedule}</span>
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
              {files.length} files · {skills.length} skills · {receipts.length} receipts
            </span>
          </div>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
