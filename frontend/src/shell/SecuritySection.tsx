// Settings → Security tripwires (native Ergon v0.8.0+, GET /api/security):
// what is armed, what fired recently, whether the tamper-evident log still
// verifies, scanners currently blocked, and the addresses the person allowed
// or blocked from Today — each removable in one tap from this machine.
import { useCallback, useEffect, useState } from "react";
import { ShieldAlert, ShieldCheck, X } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Pill } from "@/components/kibo-ui/pill";
import { Details } from "@/chat/Entry";
import { call, hasTripwire, type SecurityPayload, type Status } from "@/lib/api";
import { relTime } from "@/lib/words";
import { cn } from "@/lib/utils";

function sevDot(sev: string): string {
  return sev === "critical" ? "bg-dot-red" : sev === "warning" ? "bg-dot-amber" : "bg-icon";
}

function HostChips({
  label,
  hosts,
  canEdit,
  onRemove,
}: {
  label: string;
  hosts: string[];
  canEdit: boolean;
  onRemove: (h: string) => void;
}) {
  if (hosts.length === 0) return null;
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs text-ink-2">{label}</span>
      <div className="flex flex-wrap gap-1.5">
        {hosts.map((h) => (
          <span key={h} className="mono inline-flex items-center gap-1 rounded-chip border border-hairline bg-surface px-2 py-0.5 text-xs text-ink">
            {h}
            {canEdit && (
              <button
                type="button"
                aria-label={`Remove ${h}`}
                className="focus-ring -mr-1 inline-flex size-6 items-center justify-center rounded-sm text-icon outline-none hover:bg-board max-md:size-9"
                onClick={() => onRemove(h)}
              >
                <X className="size-3" strokeWidth={1.5} aria-hidden />
              </button>
            )}
          </span>
        ))}
      </div>
    </div>
  );
}

export function SecuritySection({ status }: { status: Status | null | "error" }) {
  const [data, setData] = useState<SecurityPayload | null>(null);
  const [failed, setFailed] = useState(false);
  const enabled = hasTripwire(status);

  const load = useCallback(async () => {
    const r = await call<SecurityPayload>("/api/security");
    if (r.ok && Array.isArray(r.data.events) && Array.isArray(r.data.armed)) {
      setData(r.data);
      setFailed(false);
    } else {
      setFailed(true);
    }
  }, []);

  useEffect(() => {
    if (enabled) load();
  }, [enabled, load]);

  const policy = useCallback(
    async (body: Record<string, unknown>, done: string) => {
      const r = await call<unknown>("/api/security/policy", { method: "POST", body });
      if (!r.ok) {
        toast.error(r.error ? "That did not go through: " + r.error : "That did not go through.");
        return;
      }
      toast.success(done);
      await load();
    },
    [load]
  );

  if (!enabled) return null;

  const visible = (data?.events ?? []).filter((e) => e.severity !== "info").slice(0, 6);
  const quietCount = (data?.events ?? []).filter((e) => e.severity === "info").length;

  return (
    <section
      id="settings-security-section"
      aria-labelledby="settings-security-h"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <ShieldCheck className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          <h3 id="settings-security-h" className="text-sm font-medium text-ink">Security tripwires</h3>
        </div>
        {data && (
          <Pill variant="secondary" className="bg-pill-active-bg px-2.5 py-0.5 text-xs font-medium text-pill-active-fg">
            <span aria-hidden className="size-2 rounded-full bg-dot-green" />
            <span>{data.armed.length} armed</span>
          </Pill>
        )}
      </div>
      <p className="text-xs leading-5 text-ink-2">
        Decoys no honest task ever touches. When one fires, Ergon blocks it, stops the task if needed, and tells you in Today. Your own computer is never locked out.
      </p>

      {failed && <p className="text-xs text-ink-2">Security status could not be loaded. Try reopening Settings.</p>}

      {data && (
        <>
          <div className="mono flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-2">
            <span>
              {data.last_24h.events} {data.last_24h.events === 1 ? "event" : "events"} · {data.last_24h.blocked} blocked · last 24h
            </span>
            <span className={cn("inline-flex items-center gap-1.5", data.log.verified ? "text-ink-2" : "text-ink")}>
              <span aria-hidden className={cn("size-2 rounded-full", data.log.verified ? "bg-dot-green" : "bg-dot-red")} />
              {data.log.verified ? `Log verified (${data.log.length})` : "Log was edited outside Ergon"}
            </span>
          </div>

          {visible.length === 0 ? (
            <p className="rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-ink-2">
              Nothing has tripped.{quietCount > 0 ? ` ${quietCount} quiet ${quietCount === 1 ? "note" : "notes"} (content that tried to instruct your agent was fenced off as data).` : ""}
            </p>
          ) : (
            <ul className="divide-y divide-hairline rounded-sm border border-hairline bg-surface" aria-label="Recent security events">
              {visible.map((e) => (
                <li key={e.id} className="flex items-start gap-2.5 px-3 py-2">
                  <span aria-hidden className={cn("mt-1.5 size-2 shrink-0 rounded-full", sevDot(e.severity))} />
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-baseline justify-between gap-x-2">
                      <p className="min-w-0 text-xs font-medium text-ink [overflow-wrap:anywhere]">{e.title}</p>
                      <span className="mono shrink-0 text-[11px] text-ink-3">{relTime(e.ts)}</span>
                    </div>
                    <p className="mono mt-0.5 text-[11px] text-ink-3">
                      {e.code} · {e.action}
                      {e.repeats ? ` · +${e.repeats} similar` : ""}
                    </p>
                  </div>
                </li>
              ))}
            </ul>
          )}

          {data.bans.length > 0 && (
            <div className="flex flex-col gap-1.5">
              <span className="text-xs text-ink-2">Scanners blocked right now</span>
              <ul className="flex flex-col gap-1.5">
                {data.bans.map((b) => (
                  <li key={b.ip} className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-3 py-1.5">
                    <span className="mono inline-flex items-center gap-1.5 text-xs text-ink">
                      <ShieldAlert className="size-3.5 text-dot-amber" strokeWidth={1.5} aria-hidden />
                      {b.ip}
                    </span>
                    {data.can_edit && (
                      <Button type="button" size="xs" variant="outline" className="max-md:min-h-11" onClick={() => policy({ unban: b.ip }, `Unblocked ${b.ip}.`)}>
                        Unblock
                      </Button>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          )}

          <HostChips
            label="Private addresses your agent may open"
            hosts={data.policy.allow_hosts}
            canEdit={data.can_edit}
            onRemove={(h) => policy({ list: "allow", host: h, add: false }, `${h} is blocked again.`)}
          />
          <HostChips
            label="Sites your agent will not open"
            hosts={data.policy.block_hosts}
            canEdit={data.can_edit}
            onRemove={(h) => policy({ list: "block", host: h, add: false }, `${h} can be opened again.`)}
          />

          <Details label="What is armed">
            <ul className="flex flex-col gap-1.5">
              {data.armed.map((a) => (
                <li key={a.id} className="text-xs leading-5 text-ink-2">
                  <span className="font-medium text-ink">{a.label}</span> — {a.detail}
                </li>
              ))}
            </ul>
          </Details>
        </>
      )}
    </section>
  );
}
