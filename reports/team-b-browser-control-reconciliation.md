# BROWSER-CONTROL RECONCILIATION — installed `dsh-inner-browser-use` vs audit pick `dsh-builtin-browser`

Filed 2026-11-01T17:05Z by `fleet-lead` (Team B, Lane 4) per the Lead's order. **Neither is absorbed; Nox rules the
pick.** Evidence: live profile inventory + installed code inspection + published tarballs with registry integrity.

## 0. The two candidates (same gap, different components)

| | A. `dsh-inner-browser-use` **0.41.0** (actually installed) | B. `dsh-builtin-browser` **0.5.2** (audit pick) |
|---|---|---|
| Licence / repo | MIT · regrevia · `github.com/regrevia/dsh-inner-browser-use` | MIT · `wqty123/dsh-browser` · npm publisher `shanyuhui` |
| Integrity | registry sha1 `5659e097…`, sha512 `zZZyHwj/…`; **installed == published (diff -rq clean)** | registry sha1 `1216fec5…`, sha512 `mXAqUbl2…`; downloaded tarball **matches both** (sha256 `81a9cb11…`) |
| Profile status | installed + enabled in the desktop profile; **in use by this session** (right-column mirror, `~/.dsh/inner-browser-use` data dir) | not installed; 93 files; `postinstall` downloads an Electron binary (`scripts/install-electron.mjs`), `engines.dsh >=0.1.1-rc.2 <0.3.0`, many `@deepseek-ai/*` peer deps |

## 1. The two legs — head to head

| Leg | A. inner-browser-use | B. builtin-browser |
|---|---|---|
| **(a) side-panel mirrored background browser** | **YES, without host-file mutation.** Its client injects into `@deepseek-ai/dsh-client-ui-sidebar-right` (`dsh.client.inject`), registers a right-sidebar tab/slot (`client.js:87,108,257-262`), and mirrors a browsable child Chrome via CDP screencast. Working today in this profile. | **YES, but only through the desktop bridge** — README: official side panel "when the shell and the bridge support it", otherwise a standalone window. The web side has **no side panel** (self-hosted window). |
| **(b) control of Cayleb's own Chrome** | Partly: launches **his installed Chrome/Chromium binary** (`/usr/bin/google-chrome`, `/usr/bin/chromium`, macOS and Windows paths; Edge on Windows) as a CDP-driven child — but **with its own profile dir** (`--remote-debugging-port=0` + its own `--user-data-dir`), i.e. a new browser, not his running session. | Same shape: "your own installed Chrome / Edge" is a **carrier choice** — system-browser runtime launches it with a plugin-owned `--user-data-dir` and `--remote-debugging-port=0` (`lib/browser-electron/system-browser.js:745-746`). Not his running session either. |
| **Leg-(b) reconciliation (decisive)** | **Neither component controls Cayleb's *running* Chrome / his live session.** Both drive a fresh Chrome/Chromium instance with a separate profile. If the acceptance wording "control of Cayleb's own Chrome" means his **installed browser as the agent's browser**, both cover it (A via binary auto-detection, B via a settings carrier). If it means his **live session/profile**, **neither covers it** — that is a separate CDP-attach capability (requires him to start Chrome with a debug port; his profile/cookies risk) and must be named as a gap, not implied by either adoption. |

## 2. Failure behaviour / host impact (the audit's key concern)

| Aspect | A. inner-browser-use | B. builtin-browser |
|---|---|---|
| Does it modify the host app? | **No host files patched.** Writes only its own state (`~/.dsh/inner-browser-use`: tabs/flash json, screenshots, encrypted logins). No main.js / app-dir writes; no desktop bridge. | **Yes — the desktop bridge patches the desktop app**: `desktop-bridge/install.mjs` locates `resources/app/lib/main.js` (or the asar), appends a guarded import, keeps `main.js.before-bridge`, supports `--revert`; must be reinstalled after every plugin/shell upgrade; docs warn to back up the app's `resources/`. **This is exactly the audit-flagged crash class**: a bad bridge/module at host startup crashes the host app. |
| Process separation | Browser is a **separate child process** (`spawn(chrome, ['--remote-debugging-port=0',…])` with a POSIX watcher that kills Chrome with the app; Windows uses `taskkill /T /F`). Browser crashes do not take the host down. | Browser also a separate child process; but the **bridge runs inside the desktop host process** (the patch imports a bridge module into main.js), so a bridge defect is in the host's startup path. |
| Plugin code itself | Runs **in-process** (platform property: DSH plugins are in-process, no sandbox — no component is crash-proof by selection). Mitigation = staged admission + soak + rollback, not selection. | Same platform property, plus the host-mutation class above. |
| Platform reality on this host | Linux paths present; **proven working on this desktop right now**. | Badge says **Windows verified**; docs say Linux/macOS browser lookup is adapted but **end-to-end untested** on those platforms; bridge installer claims any-platform but targets the desktop app's install dir (on this host the desktop app is a **dev build** — patching dev `main.js` is an extra hazard, and HMR/dev restarts would fight it). |

## 3. Integration friction under our gates

- B additionally needs: a **`postinstall` script** to provision Electron (our authorised install path runs
  `--ignore-scripts` ⇒ would not provision it; fallback = manual Electron preparation or system Chrome), a
  **bridge install into the app directory** (a privileged/owner act on the desktop app, re-done after upgrades),
  and a new profile mount. A additionally needs: absorbing its client inject + browser bundle into the single
  package (its own `postinstall`/vendor question is lighter: no Electron dependency; it uses the system browser).
- Neither is "crash-proof": the crash-isolation fact is a host property; the mitigation is the adopted
  staged-admission/soak/one-strike-rollback regime.

## 4. Maintenance / churn

- A: 0.41.0, single maintainer, active (Flash/mirror/media features, docs/doctor scripts); installed+working here.
- B: 0.5.2 published 2026-11-01, extremely active (0.4.0–0.5.2 changelog in weeks, repeated "defects that
  crashed the host" fixes, a 36-round hardening log); higher churn and a bigger surface (34 tools, bridge, Electron
  download), Windows-first.

## 5. Recommendation to Nox (pick is his; neither absorbed meanwhile)

1. **Do not adopt B's main.js bridge on this host** as part of a single-package acceptance: it is the audit's own
   crash class, it mutates the desktop app (dev build here), it must be re-run after every upgrade, and its
   Linux E2E is explicitly untested. If B is chosen, the bridge is a separate owner-authorised act and the
   side-panel leg must be re-scoped to "standalone window" until that bridge is admitted.
2. **A already satisfies leg (a) on this host without any host-file mutation**, is integrity-verified, MIT, and
   observed working in the live profile — it is the lower-risk absorb candidate for the side-panel mirror.
3. **Rule leg (b)'s meaning.** If "Cayleb's own Chrome" = his **installed browser as carrier**, A covers it
   (auto-detected binary) and B also offers it; if it means his **running session**, neither covers it — record a
   named gap and decide a separate CDP-attach capability (with the profile-risk decision) before claiming leg (b).
4. Keep the installed component (A) in place meanwhile; no absorb/remove action until this ruling. Both records
   carry the same leg-(b) gap so the acceptance wording cannot be satisfied by implication.

## 6. Evidence retained

`/tmp/cand/b.tgz` (B tarball, sha256 `81a9cb11…`, sha1/sha512 matched), `/tmp/cand/bx/package/` (extracted,
main.js bridge code/docs), installed A at
`…/profiles/desktop/node_modules/dsh-inner-browser-use/` (file digests in
`board/reports/team-b-lane4-browser-control-absorb.md` §2), registry metadata for both versions.
