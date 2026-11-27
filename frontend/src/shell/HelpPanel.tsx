// Persistent Employee Help, Plain-Language Troubleshooting, Contact Your Admin card,
// and One-Click IT Diagnostic Bundle download modal.
import { useEffect, useState } from "react";
import {
  AlertCircle,
  BookOpen,
  CheckCircle2,
  Download,
  ExternalLink,
  HelpCircle,
  LifeBuoy,
  Mail,
  Phone,
  ShieldAlert,
  SlidersHorizontal,
} from "lucide-react";
import { toast } from "sonner";
import { PanelModal } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import {
  downloadDiagnosticBundle,
  fetchDiagnosticsSummary,
  type DiagnosticsSummary,
  type HelpConfigPayload,
} from "@/lib/api";

function formatUptime(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return `${h}h ${m}m`;
}

type GuideItem = {
  id: string;
  question: string;
  answer: string;
  steps: string[];
};

const TROUBLESHOOTING_GUIDES: GuideItem[] = [
  {
    id: "model-disconnected",
    question: "Why does Ergon say \"Setup needed\" or \"not connected to a model\"?",
    answer:
      "Live chat needs an AI model connection configured by you or your IT administrator. Built-in workflows and file tools still work offline.",
    steps: [
      "Open Settings to pick a detected local model or verify the hosted model connection.",
      "If your company manages the model key on a shared server, reach out to your admin below.",
      "Attach a diagnostic bundle so IT can see whether the connection timed out or lacked credentials.",
    ],
  },
  {
    id: "blocked-file-or-site",
    question: "Why was a file or website blocked?",
    answer:
      "Ergon protects your computer and company data by keeping file edits inside your workspace and blocking untrusted web addresses.",
    steps: [
      "To work on a file from your computer, drag it onto Ergon or use Attach files so a copy is placed in your workspace.",
      "If a legitimate vendor or internal website was blocked, ask your IT admin to add that domain to the allowed hosts list in Settings -> Security.",
    ],
  },
  {
    id: "approval-waiting",
    question: "Why is a task paused and waiting for approval?",
    answer:
      "Ergon always asks a human before deleting workspace files, sending outbound emails or webhooks, or running external connector actions.",
    steps: [
      "Open the Today tab (or scroll to the card in your active chat) to inspect what the agent wants to do.",
      "Choose Approve to continue or Hold to cancel safely without making the change.",
    ],
  },
  {
    id: "interrupted-run",
    question: "What if a task fails, stalls, or stops unexpectedly?",
    answer:
      "Ergon automatically retries temporary outages and restores unsent drafts. Any unexpected crash is recorded safely for IT.",
    steps: [
      "Wait a moment and retry the request — transient model errors recover automatically.",
      "If the issue repeats, click Download IT Diagnostic Bundle below and send the ZIP file to your admin.",
    ],
  },
];

export function HelpPanel({
  open,
  onOpenChange,
  helpConfig,
  onRefreshHelpConfig,
  onOpenSettings,
  onReplayOnboarding,
  onOpenToday,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  helpConfig: HelpConfigPayload | null;
  onRefreshHelpConfig: () => Promise<void>;
  onOpenSettings: () => void;
  onReplayOnboarding: () => void;
  onOpenToday: () => void;
}) {
  const [summary, setSummary] = useState<DiagnosticsSummary | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [lastDownload, setLastDownload] = useState<{ filename: string; sha256: string; bytes: number } | null>(null);
  const [downloadErr, setDownloadErr] = useState("");

  useEffect(() => {
    if (!open) return;
    setDownloadErr("");
    void onRefreshHelpConfig();
    fetchDiagnosticsSummary().then((r) => {
      if (r.ok) setSummary(r.data);
    });
  }, [open, onRefreshHelpConfig]);

  const handleDownloadDiagnostics = async () => {
    setDownloading(true);
    setDownloadErr("");
    const res = await downloadDiagnosticBundle();
    setDownloading(false);
    if (!res.ok) {
      setDownloadErr(res.error);
      toast.error(res.error);
      return;
    }
    setLastDownload({ filename: res.filename, sha256: res.sha256, bytes: res.bytes });
    toast.success(`Downloaded ${res.filename} (secrets automatically removed).`);
    fetchDiagnosticsSummary().then((r) => {
      if (r.ok) setSummary(r.data);
    });
  };

  const contact = helpConfig?.admin_contact;
  const canEdit = Boolean(helpConfig?.can_edit_admin_contact);

  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Help & IT Support"
      description="Plain-language troubleshooting, your IT administrator contact, and one-click diagnostic logs."
      panelId="help-panel"
      className="max-w-2xl"
    >
      <div className="flex flex-col gap-4 pt-1">
        {/* 1. Contact your admin */}
        <section
          id="help-admin-contact-card"
          aria-labelledby="help-admin-contact-h"
          className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <LifeBuoy className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
              <h3 id="help-admin-contact-h" className="text-sm font-medium text-ink">
                Contact your admin
              </h3>
            </div>
            {canEdit && (
              <Button
                id="help-edit-contact-btn"
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11 gap-1.5"
                onClick={() => {
                  onOpenChange(false);
                  setTimeout(onOpenSettings, 30);
                }}
              >
                <SlidersHorizontal className="size-3.5" strokeWidth={1.5} aria-hidden />
                {contact?.configured ? "Edit in Settings" : "Set up in Settings"}
              </Button>
            )}
          </div>

          {contact?.configured ? (
            <div className="flex flex-col gap-2 text-sm text-ink">
              {contact.name && <p className="font-medium text-ink">{contact.name}</p>}
              <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs text-ink-2">
                {contact.email && (
                  <a
                    href={`mailto:${contact.email}`}
                    className="focus-ring inline-flex min-h-8 items-center gap-1.5 rounded-sm text-blue hover:underline"
                  >
                    <Mail className="size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
                    <span>{contact.email}</span>
                  </a>
                )}
                {contact.phone && (
                  <a
                    href={`tel:${contact.phone}`}
                    className="focus-ring inline-flex min-h-8 items-center gap-1.5 rounded-sm text-blue hover:underline"
                  >
                    <Phone className="size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
                    <span className="mono">{contact.phone}</span>
                  </a>
                )}
                {contact.channel_url && (
                  <a
                    href={contact.channel_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="focus-ring inline-flex min-h-8 items-center gap-1.5 rounded-sm text-blue hover:underline"
                  >
                    <ExternalLink className="size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
                    <span>Open support channel / ticket portal</span>
                  </a>
                )}
              </div>
              {contact.notes && (
                <p className="rounded-sm border border-hairline bg-surface px-3 py-2 text-xs leading-5 text-ink-2">
                  {contact.notes}
                </p>
              )}
            </div>
          ) : (
            <div className="flex items-start gap-2.5 rounded-sm border border-hairline bg-surface p-3 text-xs leading-5 text-ink-2">
              <AlertCircle className="mt-0.5 size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
              <div className="min-w-0 flex-1">
                <p className="font-medium text-ink">Admin contact details have not been added yet.</p>
                <p className="mt-0.5">
                  Ask your IT administrator to add a support name, email, phone number, or helpdesk link under{" "}
                  <span className="font-medium text-ink">Settings -&gt; IT Support Contact</span> (or set{" "}
                  <span className="mono">ERGON_ADMIN_CONTACT_*</span> when deploying).
                </p>
              </div>
            </div>
          )}
        </section>

        {/* 2. One-click IT Diagnostic Bundle */}
        <section
          id="help-diagnostics-section"
          aria-labelledby="help-diagnostics-h"
          className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <ShieldAlert className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
              <h3 id="help-diagnostics-h" className="text-sm font-medium text-ink">
                IT Diagnostic Bundle
              </h3>
            </div>
            {summary && (
              <span className="mono text-xs text-ink-2">
                v{summary.app_version} ({summary.os}/{summary.arch})
              </span>
            )}
          </div>

          <p className="text-xs leading-5 text-ink-2">
            Download one ZIP package to share with IT when reporting an issue. It bundles recent rotating logs,
            crash reports, security event status, and connector status — with all API keys, passwords, bearer
            tokens, and secrets automatically redacted.
          </p>

          {summary && (
            <div className="flex flex-wrap items-center gap-2 text-xs text-ink-2">
              <span className="mono rounded-chip border border-hairline bg-surface px-2 py-0.5">
                Uptime: {formatUptime(summary.uptime_secs)}
              </span>
              <span className="mono rounded-chip border border-hairline bg-surface px-2 py-0.5">
                Log files: {summary.log_files}
              </span>
              <span className="mono rounded-chip border border-hairline bg-surface px-2 py-0.5">
                Crash reports: {summary.crash_reports}
              </span>
              <span className="mono rounded-chip border border-hairline bg-surface px-2 py-0.5">
                Workspaces: {summary.workspace_count}
              </span>
            </div>
          )}

          <div className="flex flex-wrap items-center gap-2 pt-0.5">
            <Button
              id="help-download-diagnostics-btn"
              type="button"
              size="sm"
              disabled={downloading}
              className="max-md:min-h-11 gap-1.5"
              onClick={() => void handleDownloadDiagnostics()}
            >
              <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
              {downloading ? "Preparing diagnostic ZIP…" : "Download IT Diagnostic Bundle (.zip)"}
            </Button>
          </div>

          {downloadErr && (
            <p role="alert" className="text-xs text-destructive-foreground">
              {downloadErr}
            </p>
          )}
          {lastDownload && (
            <div
              id="help-diagnostics-downloaded"
              role="status"
              className="flex flex-wrap items-center gap-2 rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-ink-2"
            >
              <CheckCircle2 className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.5} aria-hidden />
              <span className="min-w-0 flex-1 break-words">
                Saved <span className="mono font-medium text-ink">{lastDownload.filename}</span> (
                <span className="mono">{Math.max(1, Math.round(lastDownload.bytes / 1024))} KB</span>
                {lastDownload.sha256 ? (
                  <>
                    , sha256 <span className="mono">{lastDownload.sha256.slice(0, 12)}</span>
                  </>
                ) : null}
                ) — all secrets redacted.
              </span>
            </div>
          )}
        </section>

        {/* 3. Plain-language troubleshooting */}
        <section
          id="help-troubleshooting-section"
          aria-labelledby="help-troubleshooting-h"
          className="flex flex-col gap-2 rounded-card border border-hairline bg-board/40 p-3.5"
        >
          <div className="flex items-center gap-2">
            <HelpCircle className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
            <h3 id="help-troubleshooting-h" className="text-sm font-medium text-ink">
              Common questions & troubleshooting
            </h3>
          </div>
          <div className="flex flex-col gap-1.5">
            {TROUBLESHOOTING_GUIDES.map((item, idx) => (
              <details
                key={item.id}
                open={idx === 0}
                className="rounded-sm border border-hairline bg-surface px-3 py-2"
              >
                <summary className="focus-ring cursor-pointer list-none text-sm font-medium text-ink">
                  {item.question}
                </summary>
                <p className="mt-1.5 text-xs leading-5 text-ink-2">{item.answer}</p>
                <ul className="mt-1.5 flex flex-col gap-1 pl-1 text-xs leading-5 text-ink-2">
                  {item.steps.map((s) => (
                    <li key={s} className="flex items-start gap-2">
                      <span aria-hidden className="mt-2 size-1.5 shrink-0 rounded-full bg-ink-3" />
                      <span className="min-w-0 flex-1">{s}</span>
                    </li>
                  ))}
                </ul>
                {item.id === "approval-waiting" && (
                  <div className="mt-2">
                    <Button
                      type="button"
                      size="xs"
                      variant="outline"
                      className="max-md:min-h-11"
                      onClick={() => {
                        onOpenChange(false);
                        onOpenToday();
                      }}
                    >
                      Open Today&rsquo;s approvals
                    </Button>
                  </div>
                )}
              </details>
            ))}
          </div>
        </section>

        {/* 4. Replay onboarding */}
        <section
          aria-label="Getting started guide"
          className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-board/40 px-3.5 py-2.5"
        >
          <div className="flex items-center gap-2 text-xs text-ink-2">
            <BookOpen className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
            <span>Want a refresher on workspaces, approvals, and support?</span>
          </div>
          <Button
            id="help-replay-onboarding-btn"
            type="button"
            size="xs"
            variant="outline"
            className="max-md:min-h-11 shrink-0"
            onClick={() => {
              onOpenChange(false);
              setTimeout(onReplayOnboarding, 30);
            }}
          >
            Replay 3-step getting started guide
          </Button>
        </section>
      </div>
    </PanelModal>
  );
}
