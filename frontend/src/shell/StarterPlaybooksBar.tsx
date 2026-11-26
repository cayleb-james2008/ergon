import { useState } from "react";
import { Eye, FileDown, Play, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { exportHandoffBundleZip, type StarterLaunchResult, type StarterPlaybook } from "@/lib/api";

export function StarterPlaybooksBar({
  starters,
  compact = false,
  onLaunch,
  onOpenPlaybooks,
}: {
  starters: StarterPlaybook[];
  compact?: boolean;
  onLaunch: (starterId: string, runNow?: boolean) => Promise<StarterLaunchResult | null>;
  onOpenPlaybooks?: () => void;
}) {
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);

  if (starters.length === 0) return null;

  const handleLaunch = async (starter: StarterPlaybook, exportZip: boolean) => {
    const key = `${starter.id}:${exportZip ? "zip" : "run"}`;
    setBusyKey(key);
    try {
      const res = await onLaunch(starter.id, true);
      if (!res?.ok) return;
      if (exportZip) {
        const zipRes = await exportHandoffBundleZip({ title: starter.name });
        if (zipRes.ok) {
          toast.success(
            zipRes.sha256
              ? `Launched ${starter.name} and exported ergon-client-handoff.zip (sha256: ${zipRes.sha256.slice(0, 12)}…)`
              : `Launched ${starter.name} and exported ergon-client-handoff.zip`
          );
        } else {
          toast.error(zipRes.error);
        }
      } else {
        const filesNote = res.created_files?.length ? ` (${res.created_files.join(", ")})` : "";
        toast.success(`Launched starter playbook "${starter.name}"${filesNote}.`);
      }
    } finally {
      setBusyKey(null);
    }
  };

  return (
    <section
      id={compact ? "today-starter-launchers" : "starter-playbooks-strip"}
      aria-label="Starter playbooks"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="text-sm font-medium text-ink">Starter playbooks</h2>
          <Badge variant="outline" size="sm" className="mono text-xs text-ink-2">
            Zero setup
          </Badge>
        </div>
        {onOpenPlaybooks && (
          <Button
            type="button"
            size="xs"
            variant="ghost"
            className="text-xs text-ink-2 hover:text-ink"
            onClick={onOpenPlaybooks}
          >
            Open all routines
          </Button>
        )}
      </div>

      <p className="text-xs leading-5 text-ink-2">
        Launch a pre-verified workflow in one click or run it directly into a signed Client Handoff ZIP archive.
      </p>

      <div className={compact ? "grid grid-cols-1 gap-2.5 md:grid-cols-3" : "grid grid-cols-1 gap-2.5"}>
        {starters.map((s) => {
          const isPreview = previewId === s.id;
          const runBusy = busyKey === `${s.id}:run`;
          const zipBusy = busyKey === `${s.id}:zip`;
          return (
            <div
              key={s.id}
              data-starter-playbook={s.id}
              className="flex flex-col justify-between gap-2.5 rounded-card border border-hairline bg-surface p-3"
            >
              <div className="flex flex-col gap-1.5">
                <div className="flex flex-wrap items-center justify-between gap-1.5">
                  <span className="text-sm font-medium text-ink">{s.name}</span>
                  <span className="mono rounded-chip bg-track px-2 py-0.5 text-xs text-ink-2">
                    {s.schedule}
                  </span>
                </div>
                <p className="text-xs leading-5 text-ink-2">{s.summary}</p>
              </div>

              {isPreview && (
                <div
                  data-starter-preview={s.id}
                  className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-board/40 p-2.5 text-xs"
                >
                  <div className="flex flex-wrap items-center justify-between gap-1">
                    <span className="font-medium text-ink">Dry-run steps ({s.steps.length})</span>
                    <Pill variant="secondary" className="bg-pill-active-bg px-2 py-0.5 text-xs text-pill-active-fg">
                      <ShieldCheck className="size-3" strokeWidth={1.5} aria-hidden />
                      <span>Reversible via .history/</span>
                    </Pill>
                  </div>
                  <ol className="flex flex-col gap-1 text-ink-2">
                    {s.steps.map((step, idx) => (
                      <li key={idx}>
                        <span className="mono mr-1 text-ink-3">{idx + 1}.</span>
                        {step}
                      </li>
                    ))}
                  </ol>
                  {s.expected_inputs.length > 0 && (
                    <div className="mono text-xs text-ink-2">
                      Output: {s.expected_inputs.join(", ")}
                    </div>
                  )}
                </div>
              )}

              <div className="flex flex-wrap items-center gap-1.5">
                <Button
                  type="button"
                  size="xs"
                  variant={isPreview ? "default" : "outline"}
                  data-starter-preview-btn={s.id}
                  aria-expanded={isPreview}
                  className="max-md:min-h-11"
                  onClick={() => setPreviewId(isPreview ? null : s.id)}
                >
                  <Eye className="size-3.5" strokeWidth={1.5} aria-hidden />
                  <span>{isPreview ? "Hide" : "Preview"}</span>
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  data-starter-launch-btn={s.id}
                  disabled={busyKey !== null}
                  className="max-md:min-h-11"
                  onClick={() => void handleLaunch(s, false)}
                >
                  <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                  <span>{runBusy ? "Running…" : "Launch"}</span>
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="default"
                  data-starter-handoff-btn={s.id}
                  disabled={busyKey !== null}
                  className="max-md:min-h-11"
                  onClick={() => void handleLaunch(s, true)}
                >
                  <FileDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                  <span>{zipBusy ? "Bundling…" : "Handoff (.zip)"}</span>
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
