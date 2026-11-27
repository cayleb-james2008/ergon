// Settings → Backups & Recovery (native Ergon, /api/backups*):
// Manual snapshot trigger, verified snapshot list with SHA-256 & download,
// schedule/retention/offsite folder or S3 configuration, and two-step restore confirmation.
import { useCallback, useEffect, useState } from "react";
import {
  Archive,
  CheckCircle2,
  Download,
  FolderSync,
  HardDrive,
  RefreshCw,
  RotateCcw,
  ShieldAlert,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Pill } from "@/components/kibo-ui/pill";
import {
  call,
  hasBackups,
  type BackupListEntry,
  type BackupRestoreResponse,
  type BackupRunResponse,
  type BackupsPayload,
  type Status,
} from "@/lib/api";
import { relTime } from "@/lib/words";

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function BackupsSection({
  status,
  onRestored,
}: {
  status: Status | null | "error";
  onRestored?: () => void;
}) {
  const enabled = hasBackups(status);
  const [data, setData] = useState<BackupsPayload | null>(null);
  const [lockedRemote, setLockedRemote] = useState(false);
  const [loadErr, setLoadErr] = useState("");
  const [running, setRunning] = useState(false);
  const [savingCfg, setSavingCfg] = useState(false);
  const [cfgErr, setCfgErr] = useState("");
  const [confirmRestore, setConfirmRestore] = useState<BackupListEntry | null>(null);
  const [restoring, setRestoring] = useState(false);

  // Editable schedule & offsite fields
  const [schedEnabled, setSchedEnabled] = useState(false);
  const [intervalHours, setIntervalHours] = useState("24");
  const [retentionCount, setRetentionCount] = useState("14");
  const [localDir, setLocalDir] = useState("");
  const [offsiteKind, setOffsiteKind] = useState<"none" | "folder" | "s3">("none");
  const [offsiteFolder, setOffsiteFolder] = useState("");
  const [s3Endpoint, setS3Endpoint] = useState("");
  const [s3Bucket, setS3Bucket] = useState("");
  const [s3Region, setS3Region] = useState("us-east-1");
  const [s3Prefix, setS3Prefix] = useState("ergon-backups");

  const syncForm = useCallback((payload: BackupsPayload) => {
    const c = payload.config;
    setSchedEnabled(c.enabled);
    setIntervalHours(String(c.interval_hours || 24));
    setRetentionCount(String(c.retention_count || 14));
    setLocalDir(c.local_dir || "");
    setOffsiteKind(c.offsite?.kind || "none");
    setOffsiteFolder(c.offsite?.folder_path || "");
    setS3Endpoint(c.offsite?.s3_endpoint || "");
    setS3Bucket(c.offsite?.s3_bucket || "");
    setS3Region(c.offsite?.s3_region || "us-east-1");
    setS3Prefix(c.offsite?.s3_prefix ?? "ergon-backups");
  }, []);

  const load = useCallback(async () => {
    setLoadErr("");
    setLockedRemote(false);
    const r = await call<BackupsPayload>("/api/backups");
    if (!r.ok) {
      if (r.status === 403) {
        setLockedRemote(true);
      } else {
        setLoadErr(r.error || "Could not load backup status.");
      }
      return;
    }
    setData(r.data);
    syncForm(r.data);
  }, [syncForm]);

  useEffect(() => {
    if (enabled) load();
  }, [enabled, load]);

  if (!enabled) return null;

  const runBackupNow = async () => {
    setRunning(true);
    setLoadErr("");
    const r = await call<BackupRunResponse>("/api/backups/run", {
      method: "POST",
      body: {},
    });
    setRunning(false);
    if (!r.ok) {
      const msg = r.error || "Backup failed.";
      setLoadErr(msg);
      toast.error(msg);
      return;
    }
    const b = r.data.backup;
    setData((prev) =>
      prev
        ? { ...prev, state: r.data.state, backups: r.data.backups }
        : prev
    );
    toast.success(
      `Backup saved (${b.file_count} files, ${formatBytes(b.zip_bytes)})`
    );
  };

  const saveConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    setCfgErr("");
    const ih = Number(intervalHours);
    const rc = Number(retentionCount);
    if (!Number.isInteger(ih) || ih < 1 || ih > 8760) {
      setCfgErr("Backup interval must be a whole number between 1 and 8760 hours.");
      return;
    }
    if (!Number.isInteger(rc) || rc < 1 || rc > 1000) {
      setCfgErr("Retention count must be a whole number between 1 and 1000 snapshots.");
      return;
    }
    if (offsiteKind === "folder" && !offsiteFolder.trim()) {
      setCfgErr("Enter a destination folder path for local offsite replication.");
      return;
    }
    if (offsiteKind === "s3" && !s3Bucket.trim()) {
      setCfgErr("Enter an S3 bucket name for S3 offsite replication.");
      return;
    }

    setSavingCfg(true);
    const r = await call<{ ok: boolean; config: BackupsPayload["config"] }>(
      "/api/backups/config",
      {
        method: "POST",
        body: {
          enabled: schedEnabled,
          interval_hours: ih,
          retention_count: rc,
          local_dir: localDir.trim(),
          offsite: {
            kind: offsiteKind,
            folder_path: offsiteFolder.trim(),
            s3_endpoint: s3Endpoint.trim(),
            s3_bucket: s3Bucket.trim(),
            s3_region: s3Region.trim() || "us-east-1",
            s3_prefix: s3Prefix.trim(),
            s3_path_style: true,
          },
        },
      }
    );
    setSavingCfg(false);
    if (!r.ok) {
      setCfgErr(r.error || "Could not save backup settings.");
      return;
    }
    toast.success("Backup schedule and target saved.");
    await load();
  };

  const executeRestore = async (entry: BackupListEntry) => {
    setRestoring(true);
    const r = await call<BackupRestoreResponse>("/api/backups/restore", {
      method: "POST",
      body: { name: entry.name, confirm: true },
    });
    setRestoring(false);
    if (!r.ok) {
      toast.error(r.error || "Restore failed.");
      return;
    }
    setConfirmRestore(null);
    const res = r.data.restore;
    toast.success(
      `Restored ${res.restored_files} verified files from ${entry.name}.`
    );
    setData((prev) => (prev ? { ...prev, backups: r.data.backups } : prev));
    onRestored?.();
  };

  const backups = data?.backups ?? [];

  return (
    <section
      id="settings-backups-section"
      aria-labelledby="settings-backups-h"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Archive className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          <h3 id="settings-backups-h" className="text-sm font-medium text-ink">
            Backups &amp; Recovery
          </h3>
        </div>
        {data && (
          <div className="flex flex-wrap items-center gap-1.5">
            <Pill
              variant="secondary"
              className="border border-hairline bg-surface px-2.5 py-0.5 text-xs text-ink"
            >
              <span
                aria-hidden
                className={`size-2 rounded-full ${data.config.enabled ? "bg-dot-green" : "bg-icon"}`}
              />
              <span>
                {data.config.enabled
                  ? `Every ${data.config.interval_hours}h`
                  : "Manual"}
              </span>
            </Pill>
            <Pill
              variant="secondary"
              className="mono border border-hairline bg-surface px-2 py-0.5 text-xs text-ink-2 tabular-nums"
            >
              {backups.length} {backups.length === 1 ? "snapshot" : "snapshots"}
            </Pill>
          </div>
        )}
      </div>

      <p className="text-xs leading-5 text-ink-2">
        Create full-instance ZIP snapshots with a SHA-256 manifest of every workspace file, receipt chain, client record, routine, and memory rule. Restore verifies every checksum before replacing live data.
      </p>

      {lockedRemote ? (
        <p className="text-sm text-ink-2">
          Backups and recovery can only be managed directly on the machine running Ergon (`127.0.0.1` or `localhost`).
        </p>
      ) : (
        <>
          {/* Primary action row */}
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Button
              id="settings-run-backup-btn"
              type="button"
              size="sm"
              className="max-md:min-h-11 gap-1.5"
              disabled={running || restoring}
              onClick={runBackupNow}
            >
              <HardDrive className="size-3.5" strokeWidth={1.5} aria-hidden />
              {running ? "Creating snapshot…" : "Run Backup Now"}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="max-md:min-h-11 gap-1.5"
              disabled={running || restoring}
              onClick={load}
            >
              <RefreshCw className="size-3.5" strokeWidth={1.5} aria-hidden />
              Refresh
            </Button>
          </div>

          {data?.state?.last_run_ms && (
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-2">
              <span className="inline-flex items-center gap-1.5">
                <CheckCircle2 className="size-3.5 text-dot-green" strokeWidth={1.5} aria-hidden />
                Last snapshot {relTime(data.state.last_run_ms)}
              </span>
              {data.state.last_backup_sha256 && (
                <span className="mono text-xs text-ink-3 tabular-nums">
                  SHA-256 {data.state.last_backup_sha256.slice(0, 12)}
                </span>
              )}
              {data.state.last_offsite_status && (
                <span className="text-xs text-ink-2">
                  Offsite: {data.state.last_offsite_status}
                </span>
              )}
            </div>
          )}

          {(loadErr || data?.state?.last_error) && (
            <p role="alert" className="text-xs text-destructive-foreground">
              {loadErr || data?.state?.last_error}
            </p>
          )}

          {/* Restore confirmation flow */}
          {confirmRestore && (
            <div
              id="settings-restore-confirm"
              role="alertdialog"
              aria-labelledby="restore-confirm-title"
              className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-surface p-3"
            >
              <div className="flex items-start gap-2">
                <ShieldAlert className="mt-0.5 size-4 shrink-0 text-dot-amber" strokeWidth={1.5} aria-hidden />
                <div className="min-w-0 flex-1">
                  <p id="restore-confirm-title" className="text-xs font-medium text-ink">
                    Restore full instance from <span className="mono">{confirmRestore.name}</span>?
                  </p>
                  <p className="mt-1 text-xs leading-5 text-ink-2">
                    Ergon verifies every manifest SHA-256 checksum first, saves a pre-restore safety snapshot of current live data, and atomically swaps in the verified files.
                  </p>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  id="settings-confirm-restore-btn"
                  type="button"
                  size="sm"
                  className="max-md:min-h-11 gap-1.5"
                  disabled={restoring}
                  onClick={() => executeRestore(confirmRestore)}
                >
                  <RotateCcw className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {restoring ? "Verifying & restoring…" : "Confirm Restore"}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={restoring}
                  onClick={() => setConfirmRestore(null)}
                >
                  Cancel
                </Button>
              </div>
            </div>
          )}

          {/* Snapshots list */}
          <div className="flex flex-col gap-1.5">
            <span className="text-xs font-medium text-ink">Available snapshots</span>
            {backups.length === 0 ? (
              <p className="rounded-sm border border-hairline bg-surface px-3 py-2.5 text-xs text-ink-2">
                No snapshots created yet. Click Run Backup Now to write your first verified archive.
              </p>
            ) : (
              <ul
                id="settings-backups-list"
                aria-label="Backup snapshots"
                className="divide-y divide-hairline rounded-sm border border-hairline bg-surface"
              >
                {backups.map((b) => (
                  <li
                    key={b.name}
                    data-backup-name={b.name}
                    className="flex flex-col gap-2 px-3 py-2.5 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-1.5">
                        <span className="mono truncate text-xs font-medium text-ink">
                          {b.name}
                        </span>
                        {b.kind === "pre_restore" && (
                          <Pill variant="secondary" className="bg-board px-1.5 py-0 text-xs text-ink-2">
                            Safety snapshot
                          </Pill>
                        )}
                      </div>
                      <div className="mono mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-ink-2 tabular-nums">
                        <span>{b.created_at_utc || relTime(b.modified_ms)}</span>
                        <span>{formatBytes(b.size_bytes)}</span>
                        {b.file_count !== null && (
                          <span>
                            {b.file_count} {b.file_count === 1 ? "file" : "files"}
                          </span>
                        )}
                        {b.sha256 && <span>sha256 {b.sha256.slice(0, 12)}</span>}
                      </div>
                    </div>
                    <div className="flex shrink-0 items-center gap-1.5">
                      <a
                        href={`/api/backups/download?name=${encodeURIComponent(b.name)}`}
                        download={b.name}
                        className="focus-ring inline-flex min-h-8 items-center gap-1 rounded-sm border border-hairline bg-board/50 px-2.5 text-xs font-medium text-ink outline-none hover:bg-board max-md:min-h-11"
                      >
                        <Download className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                        Download
                      </a>
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        className="max-md:min-h-11 gap-1"
                        disabled={running || restoring}
                        onClick={() => setConfirmRestore(b)}
                      >
                        <RotateCcw className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                        Restore
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>

          {/* Schedule, retention & offsite configuration */}
          <form
            id="settings-backup-config-form"
            onSubmit={saveConfig}
            className="flex flex-col gap-3 border-t border-hairline pt-3"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-1.5">
                <FolderSync className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                <span className="text-xs font-medium text-ink">Schedule, retention &amp; offsite target</span>
              </div>
              <SegmentedControl aria-label="Automated backup schedule">
                <SegmentedControlItem
                  data-id="backup-off"
                  aria-pressed={!schedEnabled}
                  data-checked={!schedEnabled ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setSchedEnabled(false)}
                >
                  Manual only
                </SegmentedControlItem>
                <SegmentedControlItem
                  data-id="backup-on"
                  aria-pressed={schedEnabled}
                  data-checked={schedEnabled ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setSchedEnabled(true)}
                >
                  Scheduled
                </SegmentedControlItem>
              </SegmentedControl>
            </div>

            <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
              <div className="flex flex-col gap-1">
                <label htmlFor="backup-interval-hours" className="text-xs font-medium text-ink">
                  Interval (hours)
                </label>
                <Input
                  id="backup-interval-hours"
                  type="number"
                  min={1}
                  max={8760}
                  value={intervalHours}
                  onChange={(e) => setIntervalHours(e.target.value)}
                  className="mono tabular-nums"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="backup-retention-count" className="text-xs font-medium text-ink">
                  Keep last snapshots
                </label>
                <Input
                  id="backup-retention-count"
                  type="number"
                  min={1}
                  max={1000}
                  value={retentionCount}
                  onChange={(e) => setRetentionCount(e.target.value)}
                  className="mono tabular-nums"
                />
              </div>
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="backup-local-dir" className="text-xs font-medium text-ink">
                Local backup directory (optional)
              </label>
              <Input
                id="backup-local-dir"
                value={localDir}
                onChange={(e) => setLocalDir(e.target.value)}
                placeholder="Default: $ERGON_DATA_DIR/backups"
                autoComplete="off"
              />
            </div>

            <div className="flex flex-col gap-2">
              <span className="text-xs font-medium text-ink">Offsite replication</span>
              <SegmentedControl aria-label="Offsite backup target">
                <SegmentedControlItem
                  data-id="offsite-none"
                  aria-pressed={offsiteKind === "none"}
                  data-checked={offsiteKind === "none" ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setOffsiteKind("none")}
                >
                  None
                </SegmentedControlItem>
                <SegmentedControlItem
                  data-id="offsite-folder"
                  aria-pressed={offsiteKind === "folder"}
                  data-checked={offsiteKind === "folder" ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setOffsiteKind("folder")}
                >
                  Mounted folder
                </SegmentedControlItem>
                <SegmentedControlItem
                  data-id="offsite-s3"
                  aria-pressed={offsiteKind === "s3"}
                  data-checked={offsiteKind === "s3" ? "" : undefined}
                  className="min-h-11 px-2.5 text-xs md:min-h-8"
                  onClick={() => setOffsiteKind("s3")}
                >
                  S3 compatible
                </SegmentedControlItem>
              </SegmentedControl>

              {offsiteKind === "folder" && (
                <div className="flex flex-col gap-1">
                  <label htmlFor="backup-offsite-folder" className="text-xs font-medium text-ink">
                    Offsite destination folder
                  </label>
                  <Input
                    id="backup-offsite-folder"
                    value={offsiteFolder}
                    onChange={(e) => setOffsiteFolder(e.target.value)}
                    placeholder="Mounted NAS or external drive path"
                    autoComplete="off"
                  />
                </div>
              )}

              {offsiteKind === "s3" && (
                <div className="flex flex-col gap-2">
                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                    <div className="flex flex-col gap-1">
                      <label htmlFor="backup-s3-bucket" className="text-xs font-medium text-ink">
                        S3 bucket
                      </label>
                      <Input
                        id="backup-s3-bucket"
                        value={s3Bucket}
                        onChange={(e) => setS3Bucket(e.target.value)}
                        placeholder="company-ergon-backups"
                        autoComplete="off"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
                      <label htmlFor="backup-s3-region" className="text-xs font-medium text-ink">
                        Region
                      </label>
                      <Input
                        id="backup-s3-region"
                        value={s3Region}
                        onChange={(e) => setS3Region(e.target.value)}
                        placeholder="us-east-1"
                        autoComplete="off"
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                    <div className="flex flex-col gap-1">
                      <label htmlFor="backup-s3-endpoint" className="text-xs font-medium text-ink">
                        Endpoint URL (optional)
                      </label>
                      <Input
                        id="backup-s3-endpoint"
                        value={s3Endpoint}
                        onChange={(e) => setS3Endpoint(e.target.value)}
                        placeholder="https://s3.us-east-1.amazonaws.com"
                        autoComplete="off"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
                      <label htmlFor="backup-s3-prefix" className="text-xs font-medium text-ink">
                        Key prefix
                      </label>
                      <Input
                        id="backup-s3-prefix"
                        value={s3Prefix}
                        onChange={(e) => setS3Prefix(e.target.value)}
                        placeholder="ergon-backups"
                        autoComplete="off"
                      />
                    </div>
                  </div>
                  <p className="text-xs leading-5 text-ink-2">
                    S3 credentials are read only from server environment variables (<span className="mono">ERGON_BACKUP_S3_ACCESS_KEY</span> / <span className="mono">ERGON_BACKUP_S3_SECRET_KEY</span>):{" "}
                    <span className="font-medium text-ink">
                      {data?.config.offsite.s3_credentials_configured
                        ? "credentials detected"
                        : "not set in environment"}
                    </span>
                    .
                  </p>
                </div>
              )}
            </div>

            {cfgErr && (
              <p role="alert" className="text-xs text-destructive-foreground">
                {cfgErr}
              </p>
            )}

            <div className="flex flex-wrap items-center gap-2">
              <Button
                id="settings-save-backup-config-btn"
                type="submit"
                size="sm"
                variant="outline"
                className="max-md:min-h-11"
                disabled={savingCfg}
              >
                {savingCfg ? "Saving…" : "Save backup settings"}
              </Button>
            </div>
          </form>
        </>
      )}
    </section>
  );
}
