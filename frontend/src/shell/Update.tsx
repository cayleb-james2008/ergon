// Update strip and version row (BANNER-FIX-BRIEF.md compliant).
// Rules enforced:
// 1. Update now shown ONLY on loopback (localhost, 127.0.0.1, [::1]) when can_apply && !dev_build; non-loopback explains plainly & hides manual Check.
// 2. Dismiss is per-version and never a dead end: renders "Show update" button and moves keyboard focus to it; manual check un-dismisses.
// 3. Plain failure messages via plainUpdateError(raw), raw error text strictly inside Details; failed checks show a banner even when available is false.
// 4. Focus management: Update now -> focuses banner status while busy -> focuses Restart now when staged; Dismiss -> focuses Show update.
// 5. Installed state states one clear sentence once, not repeated.
import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { call, type Update } from "@/lib/api";
import { isLocal, plainUpdateError, resultLine } from "@/lib/words";
import { Details } from "@/chat/Entry";

const RELEASES_URL = "https://github.com/cayleb-james2008/ergon/releases/tag/v";
const DISMISS_KEY = "ergon-update-dismissed";
const EVERY_MS = 30 * 60 * 1000;

export function announce(text: string) {
  const el = document.getElementById("announcer");
  if (el) el.textContent = text;
}

export function useUpdate() {
  const [snap, setSnap] = useState<Update | null>(null);
  const [version, setVersion] = useState("");
  const [dismissed, setDismissed] = useState(() => { try { return localStorage.getItem(DISMISS_KEY) || ""; } catch { return ""; } });
  const [busy, setBusy] = useState<"" | "apply" | "restart" | "check">("");
  const [justInstalled, setJustInstalled] = useState("");
  const [err, setErr] = useState<{ plain: string; raw: string; source: "check" | "apply" | "restart" } | null>(null);
  const [showAgain, setShowAgain] = useState(false);

  const load = useCallback(async () => {
    const r = await call<Update>("/api/update");
    if (r.ok) setSnap(r.data);
  }, []);

  useEffect(() => {
    call<{ version?: string }>("/api/version").then((r) => {
      if (r.ok && typeof r.data.version === "string") setVersion(r.data.version);
    });
    load();
    const id = setInterval(load, EVERY_MS);
    return () => clearInterval(id);
  }, [load]);

  const check = async () => {
    setBusy("check");
    setErr(null);
    const r = await call<Update>("/api/update/check", { method: "POST", body: {} });
    setBusy("");
    if (r.ok) {
      setSnap(r.data);
      setShowAgain(false);
      setDismissed("");
      try { localStorage.removeItem(DISMISS_KEY); } catch { /* */ }
      if (r.data.error) {
        const plain = plainUpdateError(r.data.error);
        setErr({ plain, raw: r.data.error, source: "check" });
        announce(plain);
      } else {
        const line = resultLine(r.data);
        toast(line);
        announce(line);
      }
    } else {
      const plain = plainUpdateError(r.error);
      setErr({ plain, raw: r.error, source: "check" });
      announce(plain);
    }
  };

  const apply = async () => {
    const latest = snap?.latest ?? "";
    setBusy("apply");
    setErr(null);
    const r = await call<Update>("/api/update/apply", { method: "POST", body: {} });
    setBusy("");
    if (r.ok) {
      const target = r.data.staged || latest;
      const msg = `Version ${target} installed. Restart to use it.`;
      setJustInstalled(msg);
      setSnap({ ...r.data, staged: target });
      toast.success(msg);
      announce(msg);
    } else {
      const plain = plainUpdateError(r.error);
      setErr({ plain, raw: r.error, source: "apply" });
      announce(plain);
    }
  };

  const restart = async () => {
    setBusy("restart");
    setErr(null);
    const r = await call<unknown>("/api/update/restart", { method: "POST", body: {} });
    if (!r.ok && r.status !== 0) {
      setBusy("");
      const plain = plainUpdateError(r.error);
      setErr({ plain, raw: r.error, source: "restart" });
      announce(plain);
      return;
    }
    const started = Date.now();
    const tick = async () => {
      if (Date.now() - started > 60000) {
        setBusy("");
        setErr({
          plain: "Ergon did not come back within a minute. Start Ergon again, then reload this page.",
          raw: "",
          source: "restart",
        });
        return;
      }
      const v = await call<{ version?: string }>("/api/version");
      if (v.ok) { location.reload(); return; }
      setTimeout(tick, 1000);
    };
    setTimeout(tick, 1000);
  };

  const dismiss = () => {
    if (!snap?.latest) return;
    try { localStorage.setItem(DISMISS_KEY, snap.latest); } catch { /* */ }
    setDismissed(snap.latest);
    setShowAgain(true);
  };

  const showUpdate = () => {
    setShowAgain(false);
    try { localStorage.removeItem(DISMISS_KEY); } catch { /* */ }
    setDismissed("");
  };

  const clearError = () => setErr(null);

  return { snap, version, dismissed, busy, justInstalled, err, showAgain, check, apply, restart, dismiss, showUpdate, clearError };
}

type U = ReturnType<typeof useUpdate>;

export function UpdateStrip({ u }: { u: U }) {
  const statusRef = useRef<HTMLParagraphElement>(null);
  const restartRef = useRef<HTMLButtonElement>(null);
  const showAgainRef = useRef<HTMLButtonElement>(null);
  const s = u.snap;
  const local = isLocal();
  const staged = !!s?.staged;
  const avail = !!(s?.available && s.latest && u.dismissed !== s.latest);
  const activeErr = u.err || (s?.error ? { plain: plainUpdateError(s.error), raw: s.error, source: "check" as const } : null);

  useEffect(() => {
    if (u.busy === "apply") statusRef.current?.focus();
    else if (u.busy === "" && u.justInstalled) restartRef.current?.focus();
  }, [u.busy, u.justInstalled]);

  useEffect(() => {
    if (u.showAgain) showAgainRef.current?.focus();
  }, [u.showAgain]);

  if (!staged && !avail && !activeErr) {
    if (s?.available && s.latest && u.dismissed === s.latest) {
      return (
        <div id="update-dismissed-bar" className="flex flex-wrap items-center justify-end gap-2.5 border-b border-hairline bg-board px-4 py-2 text-xs text-ink-2 md:px-6">
          <span>Version <span className="mono text-ink">{s.latest}</span> is available.</span>
          <Button ref={showAgainRef} size="xs" variant="outline" className="max-md:min-h-11" onClick={u.showUpdate}>Show update</Button>
        </div>
      );
    }
    return null;
  }

  // Failed check when no update is available: render plain error banner with Try again & Details.
  if (!staged && !avail && activeErr) {
    return (
      <section id="update-banner" role="region" aria-label="Ergon update" className="mx-4 mt-3 flex flex-wrap items-start justify-between gap-3 rounded-card border border-hairline bg-surface px-4 py-3 md:mx-6">
        <div className="min-w-0 flex-1">
          <p ref={statusRef} tabIndex={-1} role="alert" className="text-sm font-medium text-ink outline-none">{activeErr.plain}</p>
          {activeErr.raw && (
            <div className="mt-2">
              <Details label="Details">
                <pre className="whitespace-pre-wrap break-all font-mono text-xs text-ink-3">{activeErr.raw}</pre>
              </Details>
            </div>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {local && s?.mode !== "off" && (
            <Button size="sm" className="max-md:min-h-11" disabled={u.busy === "check"} onClick={u.check}>
              {u.busy === "check" ? "Checking…" : "Try again"}
            </Button>
          )}
          {u.err && <Button size="sm" variant="ghost" className="max-md:min-h-11" onClick={u.clearError}>Dismiss</Button>}
        </div>
      </section>
    );
  }

  return (
    <section id="update-banner" role="region" aria-label="Ergon update" className="mx-4 mt-3 flex flex-wrap items-start justify-between gap-3 rounded-card border border-hairline bg-surface px-4 py-3 md:mx-6">
      <div className="min-w-0 flex-1">
        <p ref={statusRef} tabIndex={-1} className="text-sm font-medium text-ink outline-none">
          {staged
            ? `Version ${s?.staged} installed. Restart to use it.`
            : u.busy === "apply"
              ? `Downloading and verifying version ${s?.latest}…`
              : `Version ${s?.latest} is available`}
        </p>
        {!staged && local && s?.can_apply && !s?.dev_build && (
          <p className="mt-0.5 text-sm text-ink-2">Update now downloads it, checks it, and replaces this app. Restart to start using it.</p>
        )}
        {!staged && !local && (
          <p className="mt-0.5 text-sm text-ink-2">To update, open Ergon on the computer it runs on.</p>
        )}
        {!staged && s?.dev_build && (
          <p className="mt-0.5 text-sm text-ink-2">This is a developer copy, so it does not update itself. Install a released version to get updates.</p>
        )}
        {!staged && s?.notes && s.notes.trim() && (
          <div className="mt-2">
            <Details label="Release notes">
              <p className="whitespace-pre-wrap break-words text-sm text-ink-2">{s.notes.trim()}</p>
            </Details>
          </div>
        )}
        {activeErr && (
          <div className="mt-2">
            <p className="text-sm text-destructive-foreground" role="alert">{activeErr.plain}</p>
            {activeErr.raw && (
              <div className="mt-1.5">
                <Details label="Details">
                  <pre className="whitespace-pre-wrap break-all font-mono text-xs text-ink-3">{activeErr.raw}</pre>
                </Details>
              </div>
            )}
          </div>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {staged && (
          <Button ref={restartRef} size="sm" className="max-md:min-h-11" disabled={u.busy === "restart"} onClick={u.restart}>
            {u.busy === "restart" ? "Restarting…" : "Restart now"}
          </Button>
        )}
        {!staged && local && s?.can_apply && !s?.dev_build && (
          <Button size="sm" className="max-md:min-h-11" disabled={u.busy === "apply"} onClick={u.apply}>
            {u.busy === "apply" ? "Downloading…" : activeErr ? "Try again" : "Update now"}
          </Button>
        )}
        {!staged && s?.latest && (
          <Button size="sm" variant="outline" className="max-md:min-h-11" render={<a href={s.release_url || RELEASES_URL + s.latest} target="_blank" rel="noopener" />}>
            What&apos;s new
          </Button>
        )}
        {!staged && (
          <Button size="sm" variant="ghost" className="max-md:min-h-11" onClick={u.dismiss}>
            Dismiss
          </Button>
        )}
      </div>
    </section>
  );
}

export function VersionRow({ u }: { u: U }) {
  const s = u.snap;
  if (!u.version) return null;
  const state = s?.staged ? `${s.staged} installed`
    : s?.available && s.latest ? `${s.latest} available`
    : s?.error ? "Update check failed"
    : s?.checked_at ? "Up to date"
    : "Ready";
  const canCheck = s?.mode !== "off" && isLocal();
  return (
    <div className="flex min-w-0 items-center justify-between gap-1.5 px-2 pt-1">
      <a
        id="version-link"
        href={s?.release_url || RELEASES_URL + u.version}
        target="_blank"
        rel="noopener"
        className="focus-ring flex min-h-11 min-w-0 flex-1 items-center gap-1.5 truncate rounded-sm px-1 text-xs text-ink-2 outline-none hover:text-ink md:min-h-8"
      >
        <span className="mono font-medium text-ink">Ergon {u.version}</span>
        <span aria-hidden>·</span>
        <span className="truncate">{state}</span>
        <span className="sr-only"> (opens release notes in a new tab)</span>
      </a>
      {canCheck && (
        <Button
          id="check-updates"
          size="icon-sm"
          variant="ghost"
          className="max-md:min-h-11 max-md:min-w-11 shrink-0"
          aria-label="Check for updates"
          disabled={u.busy === "check"}
          onClick={u.check}
        >
          <RefreshCw className={u.busy === "check" ? "size-3.5 motion-safe:animate-spin" : "size-3.5"} strokeWidth={1.5} aria-hidden />
        </Button>
      )}
    </div>
  );
}
