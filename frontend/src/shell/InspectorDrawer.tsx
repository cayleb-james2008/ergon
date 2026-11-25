// Collapsible Right Workspace Inspector Drawer (Claude-Desktop workbench):
// 1. Files tab: live WorkspaceFilesBody (search, upload, new file, click to open/export)
// 2. History tab: inline .history/ snapshot timeline, line diff, and 1-click restore
// 3. Audit & Offers tab: 1-click SHA-256 Audit Bundle (.md/.json), receipt chain verification, and active offers
import { memo, useCallback, useEffect, useState } from "react";
import { m, useReducedMotion } from "motion/react";
import { FileCheck2, FileDown, FolderOpen, History, PackageCheck, ShieldCheck, Sparkles, X } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { instant, sheet } from "@/motion";
import { exportClientPackDownload } from "@/lib/receiptExport";
import {
  FileHistoryDiffView,
  WorkspaceFilesBody,
} from "@/shell/WorkspaceFiles";
import {
  call,
  hasV070Api,
  type FileRevision,
  type Offer,
  type Status,
  type WorkspaceFile,
  type WorkspaceFileContent,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export type InspectorTab = "files" | "history" | "audit";

export const InspectorDrawer = memo(function InspectorDrawer({
  open,
  tab,
  onTabChange,
  onClose,
  files,
  status,
  personName,
  offers,
  selectedHistoryPath,
  onSelectHistoryPath,
  onRefreshFiles,
  onOpenFile,
  onUploadFile,
  onUploadMany,
  onExportAuditBundle,
  onOfferAction,
}: {
  open: boolean;
  tab: InspectorTab;
  onTabChange: (t: InspectorTab) => void;
  onClose: () => void;
  files: WorkspaceFile[];
  status: Status | null | "error";
  personName: string;
  offers: Offer[];
  selectedHistoryPath: string;
  onSelectHistoryPath: (p: string) => void;
  onRefreshFiles: () => void;
  onOpenFile: (path: string) => void;
  onUploadFile: (path: string, content: string, silent?: boolean, encoding?: "base64") => Promise<boolean>;
  onUploadMany: (picked: FileList | File[]) => Promise<number>;
  onExportAuditBundle: (format: "json" | "md") => Promise<void>;
  onOfferAction: (url: string, msg: string) => Promise<void>;
}) {
  const [exporting, setExporting] = useState<"md" | "json" | "pack" | null>(null);
  const [verifyInfo, setVerifyInfo] = useState<{ checked: number; ok: boolean } | null>(null);
  const [histLoading, setHistLoading] = useState(false);
  const [histErr, setHistErr] = useState("");
  const [revisions, setRevisions] = useState<FileRevision[]>([]);
  const [currentText, setCurrentText] = useState("");

  const activeFilePath =
    selectedHistoryPath && files.some((f) => f.path === selectedHistoryPath)
      ? selectedHistoryPath
      : files[0]?.path || "";

  const loadHistory = useCallback(async () => {
    if (!activeFilePath) {
      setRevisions([]);
      setCurrentText("");
      setHistErr("");
      return;
    }
    setHistLoading(true);
    setHistErr("");
    try {
      const [contentRes, histRes] = await Promise.all([
        call<WorkspaceFileContent>("/api/files/content?path=" + encodeURIComponent(activeFilePath)),
        hasV070Api(status, "file_history")
          ? call<{ revisions?: FileRevision[]; history?: FileRevision[] }>(
              "/api/files/history?path=" + encodeURIComponent(activeFilePath)
            )
          : Promise.resolve({ ok: true as const, status: 200, data: { revisions: [] as FileRevision[] }, error: "" }),
      ]);
      if (contentRes.ok) setCurrentText(String(contentRes.data.content ?? ""));
      if (histRes.ok) {
        const payload = histRes.data as { revisions?: FileRevision[]; history?: FileRevision[] };
        const list = Array.isArray(payload.revisions)
          ? payload.revisions
          : Array.isArray(payload.history)
            ? payload.history
            : [];
        setRevisions(list);
      } else if (histRes.status !== 404) {
        setHistErr(histRes.error || "Could not load version history.");
      }
    } finally {
      setHistLoading(false);
    }
  }, [activeFilePath, status]);

  useEffect(() => {
    if (open && tab === "history") void loadHistory();
  }, [open, tab, loadHistory]);

  useEffect(() => {
    if (!open || tab !== "audit") return;
    call<{ valid?: boolean; verified?: boolean; ok?: boolean; total?: number; count?: number; receipts_checked?: number; head_hash?: string }>("/api/receipts/verify").then((r) => {
      if (r.ok) {
        const isOk = typeof r.data.valid === "boolean"
          ? r.data.valid
          : r.data.verified !== false && r.data.ok !== false;
        setVerifyInfo({
          ok: isOk,
          checked: Number(r.data.total ?? r.data.receipts_checked ?? r.data.count ?? 0),
        });
      }
    });
  }, [open, tab]);

  const reduceMotion = useReducedMotion();

  if (!open) return null;

  return (
    <m.aside
      id="workspace-inspector"
      aria-label="Workspace inspector"
      initial={reduceMotion ? false : { opacity: 0, x: 6 }}
      animate={{ opacity: 1, x: 0 }}
      exit={reduceMotion ? undefined : { opacity: 0, x: 6 }}
      transition={reduceMotion ? instant : sheet}
      className="flex h-full w-full flex-col border-l border-hairline bg-surface xl:w-[340px] xl:shrink-0"
    >
      <div className="flex h-12 shrink-0 items-center justify-between gap-2 border-b border-hairline px-3.5">
        <div className="min-w-0 flex-1">
          <Tabs value={tab} onValueChange={(v) => onTabChange(v as InspectorTab)}>
            <TabsList aria-label="Inspector section" className="w-full">
              <TabsTrigger value="files" className="flex-1 gap-1 px-2 text-xs">
                <FolderOpen className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Files</span>
              </TabsTrigger>
              <TabsTrigger value="history" className="flex-1 gap-1 px-2 text-xs">
                <History className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>History</span>
              </TabsTrigger>
              <TabsTrigger value="audit" className="flex-1 gap-1 px-2 text-xs">
                <ShieldCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Audit</span>
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
        <Button
          type="button"
          size="icon-sm"
          variant="ghost"
          aria-label="Close workspace inspector"
          onClick={onClose}
          className="shrink-0 max-md:min-h-11 max-md:min-w-11"
        >
          <X className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-3.5">
        {tab === "files" && (
          <WorkspaceFilesBody
            embedded
            files={files}
            onRefresh={onRefreshFiles}
            onOpenFile={onOpenFile}
            onUpload={onUploadFile}
            onUploadMany={onUploadMany}
          />
        )}

        {tab === "history" && (
          <div id="inspector-history-panel" className="flex flex-col gap-3">
            {files.length === 0 ? (
              <p className="rounded-card border border-hairline bg-board/50 p-3.5 text-sm text-ink-2">
                No workspace files yet. Files edited by your agent automatically record version snapshots here.
              </p>
            ) : (
              <>
                <div className="flex flex-col gap-1.5">
                  <label htmlFor="inspector-history-file" className="text-xs font-medium text-ink-2">
                    Inspect versions for
                  </label>
                  <select
                    id="inspector-history-file"
                    value={activeFilePath}
                    onChange={(e) => onSelectHistoryPath(e.target.value)}
                    className="focus-ring min-h-9 w-full rounded-md border border-hairline bg-surface px-2.5 text-sm text-ink"
                  >
                    {files.map((f) => (
                      <option key={f.path} value={f.path}>
                        {f.path}
                      </option>
                    ))}
                  </select>
                </div>
                <FileHistoryDiffView
                  filePath={activeFilePath}
                  currentContent={currentText}
                  status={status}
                  revisions={revisions}
                  loading={histLoading}
                  err={histErr}
                  onReloadHistory={loadHistory}
                  onRestored={async () => {
                    onRefreshFiles();
                    await loadHistory();
                  }}
                />
              </>
            )}
          </div>
        )}

        {tab === "audit" && (
          <div id="inspector-audit-panel" className="flex flex-col gap-4">
            <section className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5">
              <div className="flex items-center justify-between gap-2">
                <h2 className="text-sm font-medium text-ink">Audit bundle</h2>
                {verifyInfo && (
                  <span
                    className={cn(
                      "mono rounded-chip px-2 py-0.5 text-xs",
                      verifyInfo.ok ? "status-finished" : "status-stopped"
                    )}
                  >
                    {verifyInfo.ok ? `Verified (${verifyInfo.checked})` : "Check chain"}
                  </span>
                )}
              </div>
              <p className="text-sm leading-5 text-ink-2">
                Download a complete SHA-256 verified record of work, decisions, and workspace deliverables for {personName}.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={exporting !== null}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setExporting("md");
                    try {
                      await onExportAuditBundle("md");
                    } finally {
                      setExporting(null);
                    }
                  }}
                >
                  <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {exporting === "md" ? "Exporting…" : "Audit Bundle (.md)"}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={exporting !== null}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setExporting("json");
                    try {
                      await onExportAuditBundle("json");
                    } finally {
                      setExporting(null);
                    }
                  }}
                >
                  <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {exporting === "json" ? "Exporting…" : "Bundle (.json)"}
                </Button>
                <Button
                  id="package-verify-btn"
                  type="button"
                  size="sm"
                  variant="default"
                  disabled={exporting !== null || files.length === 0}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setExporting("pack");
                    try {
                      const [verifyRes, packRes] = await Promise.all([
                        call<{ valid?: boolean; total?: number; count?: number }>("/api/receipts/verify"),
                        exportClientPackDownload(
                          files.slice(0, 15).map((f) => f.path),
                          `${personName} Verified Deliverable Pack`
                        ),
                      ]);
                      const checked = verifyRes.ok
                        ? Number(verifyRes.data.total ?? verifyRes.data.count ?? 0)
                        : 0;
                      const valid = verifyRes.ok && verifyRes.data.valid !== false;
                      if (verifyRes.ok) {
                        setVerifyInfo({ ok: valid, checked });
                      }
                      toast.success(
                        `Verified ${checked} receipts + packaged ${packRes.filename} (sha256: ${packRes.sha256.slice(0, 12)}…)`
                      );
                    } catch {
                      toast.error("Could not package and verify deliverables.");
                    } finally {
                      setExporting(null);
                    }
                  }}
                >
                  <PackageCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {exporting === "pack" ? "Packaging…" : "Package & Verify (.md)"}
                </Button>
              </div>
            </section>

            <section className="flex flex-col gap-2.5">
              <div className="flex items-center justify-between gap-2">
                <h2 className="text-sm font-medium text-ink">Automation suggestions</h2>
                {offers.length > 0 && (
                  <span className="mono rounded-chip bg-tag-plan-bg px-2 py-0.5 text-xs text-tag-plan-fg">
                    {offers.length}
                  </span>
                )}
              </div>
              {offers.length === 0 ? (
                <p className="rounded-card border border-hairline bg-board/30 p-3 text-sm text-ink-2">
                  When you ask for similar tasks repeatedly, one-tap app offers appear here.
                </p>
              ) : (
                <div className="flex flex-col gap-2">
                  {offers.map((o) => (
                    <div key={o.id} className="flex flex-col gap-2 rounded-card border border-hairline bg-surface p-3">
                      <p className="flex items-center gap-1.5 text-sm font-medium text-ink">
                        <Sparkles className="size-3.5 shrink-0 text-dot-amber" strokeWidth={1.5} aria-hidden />
                        <span>{String(o.title ?? "Suggested capability")}</span>
                      </p>
                      <p className="text-xs leading-4 text-ink-2">{String(o.detail ?? "")}</p>
                      <div className="flex flex-wrap gap-1.5">
                        <Button
                          size="xs"
                          className="max-md:min-h-11"
                          onClick={async () => {
                            try {
                              await onOfferAction(`/api/offers/${encodeURIComponent(o.id)}/accept`, "Added to your suite as a one-tap app.");
                            } catch {
                              toast.error("Could not accept offer.");
                            }
                          }}
                        >
                          Make it one tap
                        </Button>
                        <Button
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => onOfferAction(`/api/offers/${encodeURIComponent(o.id)}/dismiss`, "Offer dismissed.")}
                        >
                          Not now
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </section>
          </div>
        )}
      </div>
    </m.aside>
  );
});
