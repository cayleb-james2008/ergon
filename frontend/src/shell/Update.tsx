// Update strip and version row (COMMON.md spec). Results go to Sonner; the page's live region gets the same words.
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

/** Says a line to the page's static live region (index.html) and nowhere else. */
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
  const [err, setErr] = useState<{ plain: string; raw: string } | null>(null);
  const [showAgain, setShowAgain] = useState(false);

  const load = useCallback(async () => { const r = await call<Update>("/api/update"); if (r.ok) setSnap(r.data); }, []);
  useEffect(() => {
    call<{ version: string }>("/api/version").then((r) => { if (r.ok) setVersion(r.data.version); });
    load();
    const id = setInterval(load, EVERY_MS);
    return () => clearInterval(id);
  }, [load]);

  const check = async () => {
    setBusy("check"); setErr(null);
    const r = await call<Update>("/api/update/check", { method: "POST", body: {} });
    setBusy("");
    if (r.ok) {
      setSnap(r.data); setShowAgain(false);
      if (r.data.latest) { setDismissed(""); try { localStorage.removeItem(DISMISS_KEY); } catch { /* */ } }
      const line = resultLine(r.data);
      toast(line); announce(line);
    } else { setErr({ plain: plainUpdateError(r.error), raw: r.error }); toast(plainUpdateError(r.error)); }
  };

  const apply = async () => {
    const latest = snap?.latest ?? "";
    setBusy("apply"); setErr(null);
    const r = await call<Update>("/api/update/apply", { method: "POST", body: {} });
    setBusy("");
    if (r.ok) {
      const msg = "Installed " + latest + ". Restart to use it.";
      setJustInstalled(msg); setSnap({ ...r.data, staged: r.data.staged || latest });
      toast.success(msg); announce(msg);
    } else { setErr({ plain: plainUpdateError(r.error), raw: r.error }); announce(plainUpdateError(r.error)); }
  };

  const restart = async () => {
    setBusy("restart"); setErr(null);
    const r = await call<unknown>("/api/update/restart", { method: "POST", body: {} });
    if (!r.ok && r.status !== 0) { setBusy(""); setErr({ plain: plainUpdateError(r.error), raw: r.error }); return; }
    // The server is going down on purpose: poll for the new process, then reload.
    const started = Date.now();
    const tick = async () => {
      if (Date.now() - started > 60000) { setBusy(""); setErr({ plain: "Ergon did not come back within a minute. Start Ergon again: run ergon, or restart its service. Then reload this page.", raw: "" }); return; }
      const v = await call<{ version: string }>("/api/version");
      if (v.ok) { location.reload(); return; }
      setTimeout(tick, 1000);
    };
    setTimeout(tick, 1000);
  };

  const dismiss = () => {
    if (!snap?.latest) return;
    try { localStorage.setItem(DISMISS_KEY, snap.latest); } catch { /* */ }
    setDismissed(snap.latest); setShowAgain(true);
  };
  const showUpdate = () => { setShowAgain(false); try { localStorage.removeItem(DISMISS_KEY); } catch { /* */ } setDismissed(""); };

  return { snap, version, dismissed, busy, justInstalled, err, showAgain, check, apply, restart, dismiss, showUpdate };
}

type U = ReturnType<typeof useUpdate>;

/** The strip above the thread: shown only when there is something to say. */
export function UpdateStrip({ u }: { u: U }) {
  const statusRef = useRef<HTMLParagraphElement>(null);
  const showAgainRef = useRef<HTMLButtonElement>(null);
  const s = u.snap;
  const staged = !!s?.staged;
  const avail = !!(s?.available && s.latest && u.dismissed !== s.latest);
  useEffect(() => { if (u.busy === "" && u.justInstalled) statusRef.current?.focus(); }, [u.busy, u.justInstalled]);
  useEffect(() => { if (u.showAgain) showAgainRef.current?.focus(); }, [u.showAgain]);
  if (!s) return null;
  if (!staged && !avail) {
    return s.available && s.latest && u.dismissed === s.latest ? (
      <div className="flex items-center justify-end gap-3 px-6 pt-3 text-sm text-ink-2">
        <span>Version {s.latest} is available.</span>
        <Button ref={showAgainRef} size="sm" variant="outline" onClick={u.showUpdate}>Show update</Button>
      </div>
    ) : null;
  }
  const local = isLocal();
  return (
    <section role="region" aria-label="Ergon update" className="mx-6 mt-3 flex flex-wrap items-center justify-between gap-3 rounded-card border border-hairline bg-surface px-4 py-2.5">
      <div className="min-w-0">
        <p ref={statusRef} tabIndex={-1} className="text-sm font-medium text-ink outline-none">
          {staged ? (u.justInstalled || (s.staged + " installed. Restart to use it.")) : "Version " + s.latest + " is available"}
        </p>
        {!staged && <p className="text-sm text-ink-2">Update now downloads it, checks it, and replaces this app. Restart to start using it.</p>}
        {!staged && s.notes && <p className="truncate text-sm text-ink-2">{s.notes.split("\n").find((l) => l.trim())}</p>}
        {!staged && !local && <p className="text-sm text-ink-2">To update, open Ergon on the computer it runs on.</p>}
        {!staged && s.dev_build && <p className="text-sm text-ink-2">This is a developer copy, so it does not update itself. Install a released version to get updates.</p>}
        {u.err && <div className="mt-1"><p className="text-sm text-ink" role="alert">{u.err.plain}</p>{u.err.raw && <Details label="Details"><pre className="whitespace-pre-wrap break-all font-mono text-xs text-ink-3">{u.err.raw}</pre></Details>}</div>}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {staged && <Button disabled={u.busy === "restart"} onClick={u.restart}>{u.busy === "restart" ? "Restarting…" : "Restart now"}</Button>}
        {!staged && local && s.can_apply && !s.dev_build && (
          <Button disabled={u.busy === "apply"} onClick={u.apply}>{u.busy === "apply" ? "Downloading and checking..." : u.err ? "Try again" : "Update now"}</Button>
        )}
        {!staged && <Button variant="outline" render={<a href={s.release_url || RELEASES_URL + s.latest} target="_blank" rel="noopener" />}>What&apos;s new</Button>}
        {!staged && <Button variant="ghost" onClick={u.dismiss}>Dismiss</Button>}
      </div>
    </section>
  );
}

/** One quiet row in the rail: the version and its state, with an icon key that checks now. */
export function VersionRow({ u }: { u: U }) {
  const s = u.snap;
  if (!u.version) return null;
  const state = s?.staged ? "Installed, restart to use it"
    : s?.available && s.latest ? "Version " + s.latest + " available"
    : s?.error ? plainUpdateError(s.error)
    : s?.checked_at ? "Up to date"
    : "Not checked yet";
  const canCheck = s?.mode !== "off" && isLocal();
  return (
    <div className="flex min-w-0 items-center gap-2 px-2 py-1">
      <a href={s?.release_url || RELEASES_URL + u.version} target="_blank" rel="noopener" className="focus-ring min-w-0 flex-1 truncate rounded-sm text-sm text-ink-2 outline-none hover:text-ink">
        Ergon {u.version} · {state}
        <span className="sr-only"> (opens in a new tab)</span>
      </a>
      {canCheck && (
        <Button size="icon-sm" variant="ghost" aria-label="Check for updates" disabled={u.busy === "check"} onClick={u.check}>
          <RefreshCw className={u.busy === "check" ? "size-4 motion-safe:animate-spin" : "size-4"} strokeWidth={1.75} aria-hidden />
        </Button>
      )}
    </div>
  );
}
