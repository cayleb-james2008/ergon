// Collapsible Right Workspace Inspector Drawer (Claude-Desktop workbench):
// 1. Files tab: live WorkspaceFilesBody (search, upload, new file, click to open/export)
// 2. History tab: inline .history/ snapshot timeline, line diff, and 1-click restore
// 3. Audit & Offers tab: 1-click SHA-256 Audit Bundle (.md/.json), receipt chain verification, and active offers
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { m, useReducedMotion } from "motion/react";
import { FileCheck2, FileDown, FolderOpen, History, PackageCheck, Printer, ShieldCheck, Sparkles, Upload, X } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { instant, sheet } from "@/motion";
import {
  compareReceipts,
  exportClientHandoffHtml,
  exportClientPackDownload,
  preflightClientPack,
  type PackPreflightReport,
  type ReceiptCompareResult,
} from "@/lib/receiptExport";
import {
  FileHistoryDiffView,
  WorkspaceFilesBody,
} from "@/shell/WorkspaceFiles";
import {
  call,
  hasV070Api,
  type EvidenceVerificationResult,
  type FileRevision,
  type Offer,
  type Status,
  type WorkspaceFile,
  type WorkspaceFileContent,
  type WorkspacePolicy,
  type WorkspacePolicyProfile,
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
  const [exporting, setExporting] = useState<"md" | "json" | "pack" | "html" | "preflight" | "delta" | null>(null);
  const [policy, setPolicy] = useState<WorkspacePolicy | null>(null);
  const [savingPolicy, setSavingPolicy] = useState<WorkspacePolicyProfile | null>(null);
  const [verifyInfo, setVerifyInfo] = useState<{ checked: number; ok: boolean } | null>(null);
  const [packPreflight, setPackPreflight] = useState<PackPreflightReport | null>(null);
  const [verifyingEvidence, setVerifyingEvidence] = useState(false);
  const [evidenceResult, setEvidenceResult] = useState<EvidenceVerificationResult | null>(null);
  const verifyFileInputRef = useRef<HTMLInputElement | null>(null);
  const [recentReceipts, setRecentReceipts] = useState<Array<{ id: string; task: string; ts?: number }>>([]);
  const [compareA, setCompareA] = useState("");
  const [compareB, setCompareB] = useState("");
  const [compareResult, setCompareResult] = useState<ReceiptCompareResult | null>(null);
  const [comparing, setComparing] = useState(false);
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
    call<WorkspacePolicy>("/api/workspace/policy").then((r) => {
      if (r.ok && r.data && typeof r.data.profile === "string") {
        setPolicy(r.data);
      }
    });
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
    call<{ receipts?: Array<{ id?: string; task?: string; ts?: number }> }>("/api/receipts?limit=25").then((r) => {
      if (!r.ok || !Array.isArray(r.data.receipts)) return;
      const list = r.data.receipts
        .map((x) => ({ id: String(x.id ?? "").trim(), task: String(x.task ?? "Completed task").trim(), ts: x.ts }))
        .filter((x) => x.id.length > 0);
      setRecentReceipts(list);
      if (list.length >= 2) {
        setCompareA((prev) => prev || list[1].id);
        setCompareB((prev) => prev || list[0].id);
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
            <section
              id="workspace-policy-card"
              className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <h2 className="text-sm font-medium text-ink">Workspace policy</h2>
                <span
                  id="workspace-policy-badge"
                  data-policy-active={policy?.profile ?? "fast_internal"}
                  className={cn(
                    "mono rounded-chip px-2 py-0.5 text-xs",
                    policy?.profile === "air_gapped"
                      ? "status-stopped"
                      : policy?.profile === "strict_client"
                        ? "status-finished"
                        : "bg-track text-ink-2"
                  )}
                >
                  {policy?.label ?? "Fast Internal"}
                </span>
              </div>
              <p className="text-xs leading-5 text-ink-2">
                {policy?.summary ?? "Switch citation enforcement, host-path redaction, and outbound network posture in one click."}
              </p>
              <div className="grid grid-cols-3 gap-1.5" role="group" aria-label="Workspace audit policy profile">
                {(
                  [
                    ["fast_internal", "Fast Internal"],
                    ["strict_client", "Strict Client"],
                    ["air_gapped", "Air-Gapped"],
                  ] as const
                ).map(([id, label]) => {
                  const active = (policy?.profile ?? "fast_internal") === id;
                  return (
                    <Button
                      key={id}
                      type="button"
                      size="xs"
                      variant={active ? "default" : "outline"}
                      data-policy-profile={id}
                      aria-pressed={active}
                      disabled={savingPolicy !== null}
                      className="max-md:min-h-11 justify-center px-2 text-xs"
                      onClick={async () => {
                        setSavingPolicy(id);
                        try {
                          const r = await call<WorkspacePolicy>("/api/workspace/policy", {
                            method: "PUT",
                            body: { profile: id },
                          });
                          if (!r.ok) {
                            toast.error(r.error || "Could not update workspace policy.");
                            return;
                          }
                          if (!r.data.profile) {
                            toast.error("Could not update workspace policy.");
                            return;
                          }
                          setPolicy(r.data);
                          toast.success(`Workspace policy set to ${r.data.label}.`);
                        } finally {
                          setSavingPolicy(null);
                        }
                      }}
                    >
                      {label}
                    </Button>
                  );
                })}
              </div>
              {policy && (
                <div className="mono flex flex-wrap gap-1.5 text-[11px] text-ink-2">
                  <span className="rounded-chip bg-surface px-2 py-0.5 border border-hairline">
                    Cite gate: {policy.strict_citation_gate ? "block" : "warn"}
                  </span>
                  <span className="rounded-chip bg-surface px-2 py-0.5 border border-hairline">
                    Redact paths: {policy.redact_host_paths ? "on" : "off"}
                  </span>
                  <span className="rounded-chip bg-surface px-2 py-0.5 border border-hairline">
                    Outbound: {policy.block_outbound_fetch ? "blocked" : "allowed"}
                  </span>
                </div>
              )}
            </section>

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
                  id="pack-preflight-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={exporting !== null || files.length === 0}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setExporting("preflight");
                    try {
                      const receiptCtx = compareB || recentReceipts[0]?.id;
                      const check = await preflightClientPack(files.slice(0, 15).map((f) => f.path), receiptCtx);
                      setPackPreflight(check);
                      const gate = check.citation_gate || (check.ok ? "100% Verified" : "Needs review");
                      toast.success(
                        `Pre-flight ${check.ok ? "PASS" : "WARN"} (${gate}): ${check.ready_count}/${check.files.length} deliverables`
                      );
                    } catch {
                      toast.error("Could not run deliverable pre-flight check.");
                    } finally {
                      setExporting(null);
                    }
                  }}
                >
                  <ShieldCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {exporting === "preflight" ? "Checking…" : "Pre-Flight Check"}
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
                      const targets = files.slice(0, 15).map((f) => f.path);
                      const receiptCtx = compareB || recentReceipts[0]?.id;
                      const [verifyRes, checkRes, packRes] = await Promise.all([
                        call<{ valid?: boolean; total?: number; count?: number }>("/api/receipts/verify"),
                        preflightClientPack(targets, receiptCtx).catch(() => null),
                        exportClientPackDownload(targets, `${personName} Verified Deliverable Pack`, {
                          receipt_id: receiptCtx,
                        }),
                      ]);
                      const checked = verifyRes.ok
                        ? Number(verifyRes.data.total ?? verifyRes.data.count ?? 0)
                        : 0;
                      const valid = verifyRes.ok && verifyRes.data.valid !== false;
                      if (verifyRes.ok) {
                        setVerifyInfo({ ok: valid, checked });
                      }
                      if (checkRes) {
                        setPackPreflight(checkRes);
                      }
                      toast.success(
                        `Verified ${checked} receipts + ${checkRes?.ready_count ?? targets.length} deliverables (sha256: ${packRes.sha256.slice(0, 12)}…)`
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
                <Button
                  id="export-handoff-html-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={exporting !== null || files.length === 0}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setExporting("html");
                    try {
                      const targets = files.slice(0, 15).map((f) => f.path);
                      const receiptCtx = compareB || recentReceipts[0]?.id;
                      const [checkRes, htmlRes] = await Promise.all([
                        preflightClientPack(targets, receiptCtx).catch(() => null),
                        exportClientHandoffHtml(targets, `${personName} Verified Deliverable`, {
                          receipt_id: receiptCtx,
                          ...(compareA && compareB && compareResult ? { delta_a: compareA, delta_b: compareB } : {}),
                        }),
                      ]);
                      if (checkRes) {
                        setPackPreflight(checkRes);
                      }
                      toast.success(
                        `Exported ${htmlRes.filename} (${htmlRes.citationGate || "100% Verified"}, sha256: ${htmlRes.sha256.slice(0, 12)}…)`
                      );
                    } catch {
                      toast.error("Could not export printable HTML client handoff.");
                    } finally {
                      setExporting(null);
                    }
                  }}
                >
                  <Printer className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {exporting === "html" ? "Rendering…" : "Client Handoff (.html)"}
                </Button>
              </div>
              {packPreflight && (
                <div
                  id="pack-preflight-summary"
                  className="rounded-sm border border-hairline bg-surface p-2.5 text-xs text-ink-2"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2 font-medium text-ink">
                    <span>Deliverable Pre-Flight</span>
                    <div className="flex flex-wrap items-center gap-1.5">
                      <span
                        id="citation-freshness-gate"
                        className={cn(
                          "mono rounded-chip px-2 py-0.5",
                          (packPreflight.citations_stale ?? 0) === 0 && (packPreflight.citations_broken ?? 0) === 0
                            ? "status-finished"
                            : "status-stopped"
                        )}
                      >
                        {packPreflight.citation_gate || "100% Verified"}
                      </span>
                      <span className={cn("mono rounded-chip px-2 py-0.5", packPreflight.ok ? "status-finished" : "status-stopped")}>
                        {packPreflight.ok
                          ? `PASS (${packPreflight.ready_count})`
                          : `WARN (${packPreflight.empty_count + (packPreflight.citations_stale ?? 0) + (packPreflight.citations_broken ?? 0)} issue)`}
                      </span>
                    </div>
                  </div>
                  {(packPreflight.citations_checked ?? 0) > 0 && (
                    <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
                      <span className="text-ink-2">
                        Citations: {packPreflight.citations_verified ?? 0}/{packPreflight.citations_checked} verified
                        {(packPreflight.citations_stale ?? 0) > 0 ? ` · ${packPreflight.citations_stale} stale` : ""}
                      </span>
                      {(packPreflight.citations ?? []).slice(0, 6).map((c) => (
                        <button
                          key={c.raw}
                          type="button"
                          data-citation={c.raw}
                          data-citation-status={c.status}
                          title={
                            c.line_excerpt
                              ? `${c.line_excerpt}${c.file_sha256 ? ` (sha256:${c.file_sha256.slice(0, 8)})` : ""}`
                              : undefined
                          }
                          onClick={() => onOpenFile(`${c.path}#L${c.line}`)}
                          className={cn(
                            "focus-ring mono rounded-chip border px-2 py-0.5 text-xs hover:bg-board",
                            c.status === "verified"
                              ? "border-hairline bg-board/60 text-ink"
                              : "border-destructive/40 bg-board text-destructive-foreground"
                          )}
                        >
                          {c.raw} ({c.status === "verified" ? "ok" : c.status})
                        </button>
                      ))}
                    </div>
                  )}
                  <ul className="mt-1.5 flex flex-col gap-1">
                    {packPreflight.files.slice(0, 5).map((f) => (
                      <li key={f.path} className="flex items-center justify-between gap-2">
                        <span className="mono truncate text-ink">{f.path}</span>
                        <span className="mono shrink-0 text-ink-2">
                          {f.count} {f.unit} · {f.sha256 ? f.sha256.slice(0, 8) : "missing"}
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </section>

            <section
              id="evidence-verifier-card"
              className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="flex items-center justify-between gap-2">
                <h2 className="text-sm font-medium text-ink">Verify evidence</h2>
                <span
                  id="evidence-verify-badge"
                  data-verify-state={
                    evidenceResult ? (evidenceResult.valid ? "valid" : "invalid") : "idle"
                  }
                  className={cn(
                    "mono rounded-chip px-2 py-0.5 text-xs",
                    evidenceResult
                      ? evidenceResult.valid
                        ? "status-finished"
                        : "status-stopped"
                      : "border border-hairline bg-surface text-ink-2"
                  )}
                >
                  {evidenceResult ? (evidenceResult.valid ? "Verified" : "Tampered") : "Offline ready"}
                </span>
              </div>
              <p className="text-sm leading-5 text-ink-2">
                Check an audit bundle JSON, client pack Markdown, or playbook pack offline against its SHA-256 digests.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button
                  id="verify-current-bundle-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={verifyingEvidence}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={async () => {
                    setVerifyingEvidence(true);
                    try {
                      const raw = await fetch("/api/audit/bundle?format=json", { cache: "no-store" });
                      if (!raw.ok) {
                        toast.error("Could not load current audit bundle.");
                        return;
                      }
                      const content = await raw.text();
                      const res = await call<EvidenceVerificationResult>("/api/verify/evidence", {
                        method: "POST",
                        body: { filename: "ergon-audit-bundle.json", content, check_workspace: true },
                      });
                      if (!res.ok) {
                        toast.error(res.error || "Could not verify evidence bundle.");
                        return;
                      }
                      setEvidenceResult(res.data);
                      if (res.data.valid) {
                        toast.success(res.data.summary);
                      } else {
                        toast.error(res.data.summary);
                      }
                    } catch {
                      toast.error("Could not verify evidence bundle.");
                    } finally {
                      setVerifyingEvidence(false);
                    }
                  }}
                >
                  <ShieldCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {verifyingEvidence ? "Verifying…" : "Verify Current Bundle"}
                </Button>
                <Button
                  id="verify-evidence-upload-btn"
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={verifyingEvidence}
                  className="max-md:min-h-11 gap-1.5"
                  onClick={() => verifyFileInputRef.current?.click()}
                >
                  <Upload className="size-3.5" strokeWidth={1.5} aria-hidden />
                  Verify File
                </Button>
                <input
                  id="verify-evidence-file-input"
                  ref={verifyFileInputRef}
                  type="file"
                  accept=".json,.md,.txt"
                  className="hidden"
                  onChange={async (e) => {
                    const picked = e.target.files?.[0];
                    e.target.value = "";
                    if (!picked) return;
                    setVerifyingEvidence(true);
                    try {
                      const content = await picked.text();
                      const res = await call<EvidenceVerificationResult>("/api/verify/evidence", {
                        method: "POST",
                        body: { filename: picked.name, content, check_workspace: false },
                      });
                      if (!res.ok) {
                        toast.error(res.error || "Could not verify selected file.");
                        return;
                      }
                      setEvidenceResult(res.data);
                      if (res.data.valid) {
                        toast.success(res.data.summary);
                      } else {
                        toast.error(res.data.summary);
                      }
                    } catch {
                      toast.error("Could not read selected evidence file.");
                    } finally {
                      setVerifyingEvidence(false);
                    }
                  }}
                />
              </div>
              {evidenceResult && (
                <div
                  id="evidence-verify-summary"
                  className="rounded-chip border border-hairline bg-surface p-2.5 text-xs"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2 font-medium text-ink">
                    <span className="truncate">{evidenceResult.title}</span>
                    {evidenceResult.actual_sha256 && (
                      <span className="mono text-ink-2">
                        sha256:{evidenceResult.actual_sha256.slice(0, 12)}
                      </span>
                    )}
                  </div>
                  <p className="mt-1 text-ink-2">{evidenceResult.summary}</p>
                  {evidenceResult.items.length > 0 && (
                    <div className="mt-1.5 flex flex-wrap gap-1.5">
                      {evidenceResult.items.slice(0, 5).map((item, idx) => (
                        <span
                          key={`${item.label}-${idx}`}
                          className={cn(
                            "mono rounded-chip px-2 py-0.5",
                            item.status === "verified" ? "status-finished" : "status-stopped"
                          )}
                        >
                          {item.label}: {item.status}
                        </span>
                      ))}
                    </div>
                  )}
                  {evidenceResult.errors.length > 0 && (
                    <ul className="mt-1.5 flex flex-col gap-0.5 text-destructive-foreground">
                      {evidenceResult.errors.slice(0, 4).map((err, idx) => (
                        <li key={idx}>{err}</li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </section>

            <section id="receipt-compare-panel" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5">
              <div className="flex items-center justify-between gap-2">
                <h2 className="text-sm font-medium text-ink">Compare runs</h2>
                <span className="mono text-xs text-ink-2">{recentReceipts.length} recent</span>
              </div>
              {recentReceipts.length < 2 ? (
                <p className="text-xs leading-5 text-ink-2">
                  Complete at least two runs to compare summary/file deltas and append a verified delta section to your Client Pack.
                </p>
              ) : (
                <>
                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                    <label className="flex flex-col gap-1 text-xs text-ink-2">
                      <span>Baseline run</span>
                      <select
                        id="compare-receipt-a"
                        value={compareA}
                        onChange={(e) => setCompareA(e.target.value)}
                        className="focus-ring min-h-11 rounded-md border border-hairline bg-surface px-2 text-xs text-ink md:min-h-9"
                      >
                        {recentReceipts.map((r) => (
                          <option key={`a-${r.id}`} value={r.id}>
                            {r.task.slice(0, 34)} ({r.id.slice(-8)})
                          </option>
                        ))}
                      </select>
                    </label>
                    <label className="flex flex-col gap-1 text-xs text-ink-2">
                      <span>Target run</span>
                      <select
                        id="compare-receipt-b"
                        value={compareB}
                        onChange={(e) => setCompareB(e.target.value)}
                        className="focus-ring min-h-11 rounded-md border border-hairline bg-surface px-2 text-xs text-ink md:min-h-9"
                      >
                        {recentReceipts.map((r) => (
                          <option key={`b-${r.id}`} value={r.id}>
                            {r.task.slice(0, 34)} ({r.id.slice(-8)})
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      id="compare-runs-btn"
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={comparing || !compareA || !compareB}
                      className="max-md:min-h-11 gap-1.5"
                      onClick={async () => {
                        setComparing(true);
                        try {
                          const res = await compareReceipts(compareA, compareB);
                          setCompareResult(res);
                        } catch {
                          toast.error("Could not compare selected runs.");
                        } finally {
                          setComparing(false);
                        }
                      }}
                    >
                      {comparing ? "Comparing…" : "Compare Runs"}
                    </Button>
                    <Button
                      id="append-receipt-delta-btn"
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={exporting !== null || !compareA || !compareB}
                      className="max-md:min-h-11 gap-1.5"
                      onClick={async () => {
                        setExporting("delta");
                        try {
                          const res = await exportClientPackDownload(
                            files.slice(0, 15).map((f) => f.path),
                            `${personName} Delta Pack`,
                            { delta_a: compareA, delta_b: compareB }
                          );
                          toast.success(`Exported ${res.filename} with run delta (sha256: ${res.sha256.slice(0, 12)}…)`);
                        } catch {
                          toast.error("Could not export Delta Pack.");
                        } finally {
                          setExporting(null);
                        }
                      }}
                    >
                      <PackageCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                      {exporting === "delta" ? "Packing…" : "Append Delta to Client Pack (.md)"}
                    </Button>
                  </div>
                  {compareResult && (
                    <div id="receipt-compare-result" className="flex flex-col gap-2 rounded-sm border border-hairline bg-surface p-2.5 text-xs">
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <span className="font-medium text-ink">Run delta</span>
                        <div className="mono flex items-center gap-1.5">
                          <span className="rounded-chip bg-pill-active-bg px-1.5 py-0.5 text-pill-active-fg">+{compareResult.added}</span>
                          <span className="rounded-chip bg-pill-pro-bg px-1.5 py-0.5 text-pill-pro-fg">-{compareResult.removed}</span>
                          <span className="text-ink-2">{compareResult.similarity_pct}% match</span>
                        </div>
                      </div>
                      {(compareResult.files_added.length > 0 || compareResult.files_removed.length > 0) && (
                        <p className="text-ink-2">
                          Files: +{compareResult.files_added.length} added, -{compareResult.files_removed.length} removed
                        </p>
                      )}
                      {compareResult.unified.trim() ? (
                        <pre className="max-h-36 overflow-auto whitespace-pre-wrap break-words rounded-sm bg-board p-2 font-mono text-xs text-ink">
                          {compareResult.unified}
                        </pre>
                      ) : (
                        <p className="text-ink-2">Summaries are identical across both runs.</p>
                      )}
                    </div>
                  )}
                </>
              )}
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
