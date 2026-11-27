// Update strip, version row, and Settings admin update controls.
// Rules enforced:
// 1. Update actions shown whenever the server grants admin management (`can_manage` / `is_admin`,
//    which includes loopback local admins AND authenticated admins behind ERGON_GATE_TOKEN).
//    Non-admin sessions see clear read-only messaging and hide mutating controls.
// 2. Dismiss is per-version and never a dead end: renders "Show update" button and moves keyboard focus to it; manual check un-dismisses.
// 3. Plain failure messages via plainUpdateError(raw), raw error text strictly inside Details; failed checks show a banner even when available is false.
// 4. Focus management: Update now -> focuses banner status while busy -> focuses Restart now when staged; Dismiss -> focuses Show update.
// 5. Installed state states one clear sentence once, not repeated.
import { useCallback, useEffect, useRef, useState } from "react";
import { Download, Lock, RefreshCw, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { call, type Update } from "@/lib/api";
import { isLocal, plainUpdateError, resultLine } from "@/lib/words";
import { Details } from "@/chat/Entry";

const RELEASES_URL = "https://github.com/cayleb-james2008/ergon/releases/tag/v";
const DISMISS_KEY = "ergon-update-dismissed";
const ADMIN_TOKEN_KEY = "ergon-update-admin-token";
const EVERY_MS = 30 * 60 * 1000;

export type UpdateMode = "off" | "notify" | "auto";

export function announce(text: string) {
  const el = document.getElementById("announcer");
  if (el) el.textContent = text;
}

function readStoredAdminToken(): string {
  try {
    return sessionStorage.getItem(ADMIN_TOKEN_KEY) || "";
  } catch {
    return "";
  }
}

function authHeadersFor(token: string): Record<string, string> | undefined {
  const trimmed = token.trim();
  if (!trimmed) return undefined;
  return {
    Authorization: `Bearer ${trimmed}`,
    "X-Ergon-Admin-Token": trimmed,
  };
}

export function canManageUpdates(s: Update | null): boolean {
  if (typeof s?.can_manage === "boolean") return s.can_manage;
  if (typeof s?.is_admin === "boolean") return s.is_admin;
  return isLocal();
}

export function useUpdate() {
  const [snap, setSnap] = useState<Update | null>(null);
  const [version, setVersion] = useState("");
  const [dismissed, setDismissed] = useState(() => {
    try {
      return localStorage.getItem(DISMISS_KEY) || "";
    } catch {
      return "";
    }
  });
  const [adminToken, setAdminTokenState] = useState<string>(() => readStoredAdminToken());
  const [busy, setBusy] = useState<"" | "apply" | "restart" | "check" | "mode" | "auth">("");
  const [justInstalled, setJustInstalled] = useState("");
  const [err, setErr] = useState<{ plain: string; raw: string; source: "check" | "apply" | "restart" | "mode" | "auth" } | null>(null);
  const [showAgain, setShowAgain] = useState(false);

  const load = useCallback(async (overrideToken?: string) => {
    const tok = overrideToken !== undefined ? overrideToken : adminToken;
    const r = await call<Update>("/api/update", { headers: authHeadersFor(tok) });
    if (r.ok) {
      setSnap(r.data);
      return r.data;
    }
    return null;
  }, [adminToken]);

  useEffect(() => {
    call<{ version?: string }>("/api/version", { headers: authHeadersFor(adminToken) }).then((r) => {
      if (r.ok && typeof r.data.version === "string") setVersion(r.data.version);
    });
    load();
    const id = setInterval(() => {
      load();
    }, EVERY_MS);
    return () => clearInterval(id);
  }, [adminToken, load]);

  const check = async () => {
    setBusy("check");
    setErr(null);
    const r = await call<Update>("/api/update/check", {
      method: "POST",
      body: {},
      headers: authHeadersFor(adminToken),
    });
    setBusy("");
    if (r.ok) {
      setSnap(r.data);
      setShowAgain(false);
      setDismissed("");
      try {
        localStorage.removeItem(DISMISS_KEY);
      } catch {
        /* */
      }
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

  const setMode = async (mode: UpdateMode) => {
    setBusy("mode");
    setErr(null);
    const r = await call<Update>("/api/update/mode", {
      method: "POST",
      body: { mode },
      headers: authHeadersFor(adminToken),
    });
    setBusy("");
    if (r.ok) {
      setSnap(r.data);
      const msg = `Update mode set to ${mode}.`;
      toast.success(msg);
      announce(msg);
    } else {
      const plain = plainUpdateError(r.error);
      setErr({ plain, raw: r.error, source: "mode" });
      announce(plain);
    }
  };

  const apply = async () => {
    const latest = snap?.latest ?? "";
    setBusy("apply");
    setErr(null);
    const r = await call<Update>("/api/update/install", {
      method: "POST",
      body: {},
      headers: authHeadersFor(adminToken),
    });
    setBusy("");
    if (r.ok) {
      const target = r.data.staged || r.data.installed || r.data.applied || latest;
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
    const r = await call<unknown>("/api/update/restart", {
      method: "POST",
      body: {},
      headers: authHeadersFor(adminToken),
    });
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
      const v = await call<{ version?: string }>("/api/version", { headers: authHeadersFor(adminToken) });
      if (v.ok) {
        location.reload();
        return;
      }
      setTimeout(tick, 1000);
    };
    setTimeout(tick, 1000);
  };

  const authenticateAdmin = async (token: string) => {
    const trimmed = token.trim();
    setBusy("auth");
    setErr(null);
    const r = await call<Update>("/api/update", { headers: authHeadersFor(trimmed) });
    setBusy("");
    if (!r.ok) {
      const plain = plainUpdateError(r.error || "Unauthorized");
      setErr({ plain, raw: r.error || `HTTP ${r.status}`, source: "auth" });
      announce(plain);
      return false;
    }
    if (!canManageUpdates(r.data)) {
      const plain = "That token did not grant administrator update permissions.";
      setErr({ plain, raw: "Server returned read-only update status (is_admin=false).", source: "auth" });
      announce(plain);
      return false;
    }
    try {
      sessionStorage.setItem(ADMIN_TOKEN_KEY, trimmed);
    } catch {
      /* */
    }
    setAdminTokenState(trimmed);
    setSnap(r.data);
    toast.success("Admin update controls unlocked for this browser tab.");
    announce("Admin update controls unlocked.");
    return true;
  };

  const clearAdminToken = async () => {
    try {
      sessionStorage.removeItem(ADMIN_TOKEN_KEY);
    } catch {
      /* */
    }
    setAdminTokenState("");
    setErr(null);
    await load("");
  };

  const dismiss = () => {
    if (!snap?.latest) return;
    try {
      localStorage.setItem(DISMISS_KEY, snap.latest);
    } catch {
      /* */
    }
    setDismissed(snap.latest);
    setShowAgain(true);
  };

  const showUpdate = () => {
    setShowAgain(false);
    try {
      localStorage.removeItem(DISMISS_KEY);
    } catch {
      /* */
    }
    setDismissed("");
  };

  const clearError = () => setErr(null);

  return {
    snap,
    version,
    dismissed,
    busy,
    justInstalled,
    err,
    showAgain,
    hasAdminToken: Boolean(adminToken.trim()),
    check,
    setMode,
    apply,
    restart,
    authenticateAdmin,
    clearAdminToken,
    dismiss,
    showUpdate,
    clearError,
  };
}

type U = ReturnType<typeof useUpdate>;

export function UpdateStrip({ u }: { u: U }) {
  const statusRef = useRef<HTMLParagraphElement>(null);
  const restartRef = useRef<HTMLButtonElement>(null);
  const showAgainRef = useRef<HTMLButtonElement>(null);
  const s = u.snap;
  const canManage = canManageUpdates(s);
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
          <span>
            Version <span className="mono text-ink">{s.latest}</span> is available.
          </span>
          <Button ref={showAgainRef} size="xs" variant="outline" className="max-md:min-h-11" onClick={u.showUpdate}>
            Show update
          </Button>
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
          <p ref={statusRef} tabIndex={-1} role="alert" className="text-sm font-medium text-ink outline-none">
            {activeErr.plain}
          </p>
          {activeErr.raw && (
            <div className="mt-2">
              <Details label="Details">
                <pre className="whitespace-pre-wrap break-all font-mono text-xs text-ink-3">{activeErr.raw}</pre>
              </Details>
            </div>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {canManage && s?.mode !== "off" && (
            <Button size="sm" className="max-md:min-h-11" disabled={u.busy === "check"} onClick={u.check}>
              {u.busy === "check" ? "Checking…" : "Try again"}
            </Button>
          )}
          {u.err && (
            <Button size="sm" variant="ghost" className="max-md:min-h-11" onClick={u.clearError}>
              Dismiss
            </Button>
          )}
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
        {!staged && canManage && s?.can_apply && !s?.dev_build && (
          <p className="mt-0.5 text-sm text-ink-2">
            Update now downloads it, checks its SHA-256 checksum, and replaces this binary. Restart to start using it.
          </p>
        )}
        {!staged && !canManage && (
          <p className="mt-0.5 text-sm text-ink-2">
            {s?.gated
              ? "Managed by your administrator — updates are read-only for non-admin sessions."
              : "To update, open Ergon on the computer it runs on or authenticate as an administrator."}
          </p>
        )}
        {!staged && s?.dev_build && (
          <p className="mt-0.5 text-sm text-ink-2">
            This is a developer copy, so it does not update itself. Install a released version to get updates.
          </p>
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
            <p className="text-sm text-destructive-foreground" role="alert">
              {activeErr.plain}
            </p>
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
        {staged && canManage && (
          <Button ref={restartRef} size="sm" className="max-md:min-h-11" disabled={u.busy === "restart"} onClick={u.restart}>
            {u.busy === "restart" ? "Restarting…" : "Restart now"}
          </Button>
        )}
        {!staged && canManage && s?.can_apply && !s?.dev_build && (
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
  const canManage = canManageUpdates(s);
  const state = s?.staged
    ? `${s.staged} installed`
    : s?.available && s.latest
      ? `${s.latest} available`
      : s?.error
        ? "Update check failed"
        : s?.checked_at
          ? "Up to date"
          : !canManage && s?.gated
            ? "Admin managed"
            : "Ready";
  const canCheck = s?.mode !== "off" && canManage;
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

export function UpdateSettingsSection({ u }: { u: U }) {
  const s = u.snap;
  const canManage = canManageUpdates(s);
  const mode = (s?.mode === "off" || s?.mode === "auto" ? s.mode : "notify") as UpdateMode;
  const [tokenInput, setTokenInput] = useState("");
  const [showTokenForm, setShowTokenForm] = useState(false);

  const onSubmitToken = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!tokenInput.trim()) return;
    const ok = await u.authenticateAdmin(tokenInput);
    if (ok) {
      setTokenInput("");
      setShowTokenForm(false);
    }
  };

  return (
    <section
      id="settings-updates-section"
      aria-labelledby="settings-updates-h"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          {canManage ? (
            <ShieldCheck className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          ) : (
            <Lock className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          )}
          <h3 id="settings-updates-h" className="text-sm font-medium text-ink">
            Version &amp; Updates
          </h3>
        </div>
        <span className="text-xs text-ink-2">
          {canManage ? (s?.gated ? "Admin authorized" : "Local admin") : "Read-only"}
        </span>
      </div>

      <VersionRow u={u} />

      {canManage ? (
        <div className="flex flex-col gap-2.5 border-t border-hairline pt-2.5">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-xs font-medium text-ink">Update policy</p>
              <p className="text-xs text-ink-2">Control whether Ergon checks or stages verified releases.</p>
            </div>
            <SegmentedControl aria-label="Update policy">
              {(["off", "notify", "auto"] as const).map((m) => (
                <SegmentedControlItem
                  key={m}
                  data-id={m}
                  aria-pressed={mode === m}
                  data-checked={mode === m ? "" : undefined}
                  disabled={u.busy === "mode"}
                  className="min-h-11 px-2.5 text-xs capitalize md:min-h-8"
                  onClick={() => {
                    if (mode !== m) u.setMode(m);
                  }}
                >
                  {m}
                </SegmentedControlItem>
              ))}
            </SegmentedControl>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {mode !== "off" && (
              <Button
                id="settings-check-updates-btn"
                type="button"
                size="sm"
                variant="outline"
                className="max-md:min-h-11 gap-1.5"
                disabled={u.busy === "check"}
                onClick={u.check}
              >
                <RefreshCw className={u.busy === "check" ? "size-3.5 motion-safe:animate-spin" : "size-3.5"} strokeWidth={1.5} aria-hidden />
                {u.busy === "check" ? "Checking…" : "Check for updates"}
              </Button>
            )}
            {!s?.staged && s?.can_apply && !s?.dev_build && (
              <Button
                id="settings-install-update-btn"
                type="button"
                size="sm"
                className="max-md:min-h-11 gap-1.5"
                disabled={u.busy === "apply"}
                onClick={u.apply}
              >
                <Download className="size-3.5" strokeWidth={1.5} aria-hidden />
                {u.busy === "apply" ? "Installing…" : `Install ${s.latest}`}
              </Button>
            )}
            {s?.staged && (
              <Button
                id="settings-restart-update-btn"
                type="button"
                size="sm"
                className="max-md:min-h-11"
                disabled={u.busy === "restart"}
                onClick={u.restart}
              >
                {u.busy === "restart" ? "Restarting…" : `Restart to use ${s.staged}`}
              </Button>
            )}
            {u.hasAdminToken && (
              <Button
                type="button"
                size="sm"
                variant="ghost"
                className="max-md:min-h-11 text-xs"
                onClick={u.clearAdminToken}
              >
                Lock admin session
              </Button>
            )}
          </div>
        </div>
      ) : (
        <div className="flex flex-col gap-2 border-t border-hairline pt-2.5">
          <p id="settings-updates-readonly" className="text-xs leading-5 text-ink-2">
            {s?.gated
              ? "Updates are read-only for non-admin sessions on this gated host. Contact your IT administrator or unlock with an admin token."
              : "Updates can only be managed locally or by an authenticated administrator."}
          </p>
          {s?.gated && !showTokenForm && (
            <div>
              <Button
                id="settings-unlock-updates-btn"
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11"
                onClick={() => setShowTokenForm(true)}
              >
                Unlock with admin token
              </Button>
            </div>
          )}
          {s?.gated && showTokenForm && (
            <form onSubmit={onSubmitToken} className="flex flex-wrap items-center gap-2">
              <Input
                id="settings-admin-token-input"
                type="password"
                aria-label="Admin update token"
                placeholder="Paste admin token (tab session only)"
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                autoComplete="off"
                className="min-w-[200px] flex-1 max-md:min-h-11"
              />
              <Button type="submit" size="sm" className="max-md:min-h-11" disabled={u.busy === "auth" || !tokenInput.trim()}>
                {u.busy === "auth" ? "Checking…" : "Unlock"}
              </Button>
              <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => setShowTokenForm(false)}>
                Cancel
              </Button>
            </form>
          )}
        </div>
      )}

      {u.err && (
        <div role="alert" className="rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-destructive-foreground">
          <p>{u.err.plain}</p>
          {u.err.raw && (
            <div className="mt-1">
              <Details label="Details">
                <pre className="whitespace-pre-wrap break-all font-mono text-xs text-ink-3">{u.err.raw}</pre>
              </Details>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
