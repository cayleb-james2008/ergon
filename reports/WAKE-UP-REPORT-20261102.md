# WAKE-UP REPORT — 2026-11-02 (refreshed 02:42Z / 21:42 ET; owner asleep)
Consolidated for the owner's morning. Everything below is on the board with checker evidence; nothing is smoothed.
Fences held overnight: no spend, no publication, no credential values in chat/board/logs, no image reads in the app, desks untouched.

## THE TWO THINGS THAT NEED THE OWNER (nothing else)
1. **Key entry for the fleet roster (one act).** Run in a terminal as yourself:
   `node /home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs first-run --credentials`
   Field he will see: `OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:` — echo off; the value goes to the fleet credential
   store (0600) via the uid-1000 control op and is never printed, logged or sent anywhere. On success (we verify automatically) the
   roster's real provider route activates and we run the ruled proof (a real `fleet-roster lane-spawn` turn on the cloud OpenCode Go
   route), then fire the staged first real order — the plugin's visible surfaces (own sidebar section + phone-VM and Windows-VM panels)
   — and file its order/task ids. Value never crosses chat/board/logs; success/failure only.
2. **Grade the Ergon v2 direction (his eyes only).** The live animated pages are staged in the DSH app's right-column browser: index
   first, then the seven comps + storyboard tabs, motion running (storyboard has Replay + ×0.25 aids). Direction: **Ergon Lumen** —
   translucent layered panels with real depth (blur/hairlines/light-through-surfaces), neural-net character, technical-editorial taste
   from the Prime Intellect reference, live motion. Fresh-checker FINAL ACCEPT on the frozen set (`ea68e398…`, C1–C5 verified); the
   rendered look is his grade. Suggested order: index → storyboard (Replay, then ×0.25) → 07-states → 02 Review + Approve once →
   01 send/character tap → 05 tab swap → 06 → 03 → 04.

## FINISH LINE 1 — FLEET PLUGIN: INSTALL PROOF COMPLETE (checker-verified)
- **Installed 0.3.1** (tarball `5ede7484…`, 123 members) through the reviewed wrapper via the guard-tested install script;
  `INSTALL-0.3.1: PASS`. `fleet-dsh` active; identity readback 123/123, version 0.3.1.
- **Installed-output acceptance:** the plugin's own UNMODIFIED `stage2-check.sh` on the installed host →
  **`pass=60 fail=0`, `STAGE 2: PASS`** (fresh checker, 130-file evidence). Rows 8 pinned / 12 disabled; MCP nine tools on
  `fleet-mcp 0.3.1-mcp`; listeners loopback + tailnet only.
- **Roster machinery mounted:** `fleet-roster` row, doctrine table (DeepSeek V4.1 Flash lanes via opencode-go; Claude lead/checker
  seats), `rates` row, `opencode-go` route (apiKeyEnv only; credential absent until the owner act), **sub-agent ceiling 32** shipped.
- **Activation is the only remaining step and it is gated solely on the key entry** — no real-provider turn has run yet and no
  activation is claimed. The first real order is staged and fires on activation.
- **Named residuals:** facade `session.create {mode:"model"}` is an unconditional refusal → the activation proof is the roster
  lane-spawn turn (Nox ruled Option A); the facade gap is a named defect queued for a 0.3.2 candidate; one stale unchanged-battery
  expectation (T7); roundtrip `real_lane` label prints a conservative value (doc/code mismatch); `dshVersion` null; rollback armed
  (0.3.0 `8173d13e…` in the drop) but not executed.

## FINISH LINE 2 — ERGON END-TO-END WITH THE VM SURFACES: PARTIALLY MET (honest; re-file pending)
- **Phone VM — PROVEN REAL** (checker ACCEPT, independently re-run): headless Android 15/API 35 `rig-phone` boot; scripted native tap
  through Settings → "Network & internet"; the frozen v2 mobile comp rendered in the phone's Chrome over the guest→host transport
  (tokens, `blur(14px)`, animations settling to 0 running/infinite, approve→pill, tab swap, scoped remember wording verified).
- **Windows VM — INSTALLED AND BOOTING (the overnight blocker is RESOLVED):** Windows 11 Enterprise Eval installed end-to-end
  (qcow2 393 KB → ~15.9 GiB), boots its own UEFI "Windows Boot Manager" from disk0, two OOBE reboots rode through. Two rig root causes
  found and fixed with evidence: (1) stale/foreign OVMF vars + missed El-Torito "press any key" (fresh vars + continuous sendkey);
  (2) answer medium Windows could never see — 8.3-only FAT16 `AUTOUNAT.XML` + `usb-storage removable=off` → vvfat over
  `win/unattend/` + `removable=true` (answer reads 8.7 KB → 187–195 KB, then unattended DiskConfiguration ran). Launcher patch
  produced (`run-win11.sh.patched` + diff) — the rig launcher must adopt it.
- **Windows residuals (named, not blockers):** OOBE-to-desktop is NOT claimed (no text channel into the guest yet — no agent, no
  serial, VNC is video-only); a bounded deterministic re-install (new qcow2, AutoLogon + FirstLogonCommands enabling RDP/OpenSSH +
  host marker) is running now to make the surface DRIVEABLE and text-verifiable; the Ergon Windows companion surfaces remain
  package-pending (the Ergon Windows package does not exist; G1 whole).
- **S8 pendings dispositioned:** R4 phone surface = fixture-proven on the phone VM + product surface package-pending; R3 real
  shareable URL = carried (host + package absent); R6 voice = carried (package absent); R2 model-quality = proof pre-written, runs the
  moment roster activation lands; Windows companions = package-pending on an installed+bootable guest. The upgraded FL2 verdict will
  be re-filed and checker spot-verified after the driveable-channel attempt.

## ROBLOX (Team D) — instrumented diagnostic block COMPLETE; two diagnostic Plays ran honestly
- Plans 2/3/4 + the C1 fix round all fresh-checker ACCEPTED (suites and mutation proofs reproduced from clean copies; sealed place
  `c6651793…` and release `84324d43…` unchanged throughout).
- **Diagnostic Play 1** (`traffic:abort-contact`, 1 of 3): cell FAIL fail-closed — instrumentation attributed it precisely: the proxy
  outran the ego, crossed `RETIRE_RADIUS=800` at t=22.283 s and the controller's DOCUMENTED `unobservedSince > 1` rule retired the body
  0.95 s later. **Not a product defect**; the remaining cell defect is fixture geometry (contact leg unreachable). A scenario-change
  design note is filed and awaits a ruling (offline, no Play).
- **Diagnostic Play 2** (`atlas:scout --seed 2134783819`, 2 of 3): fail-closed timeout with the 45 s bound firing correctly — and the
  studio console revealed the real root cause: the seed patch called `apply` on an un-`require`d ModuleScript
  (`S.apply is not a valid member of …AtlasWorldSeed`), the starter crashed silently (outside its xpcall) so no runtime error/trace was
  written. **This SUPERSEDES the old "in-Play City+Highway rebuild hang" reading — the rebuild never ran.** The fix (require the
  modules; wrap the whole seed patch in the error-reporting xpcall; offline Lune/Python tests) is approved and being implemented; one
  Play after it tells us `fastPathApplied`. Plays used: 2/3 — Studio will be cycled to a fresh opening for the next run.
- Studio: pid 388820, sealed place/build unchanged, 2/3 Plays used in the current opening; no publication, no build.

## MONDAY DESK (Team C) — observation day, locked
- Runbook v7 (sha `512752e4…`) **P0.1 SIGNED** by a fresh Tier-S checker; ORB15 **HOLD** ruled → Monday claims 0 fills, honestly:
  09:05 guard, five 09:15 attestations individually, (c2) binding after 09:30, 09:45 observation window, 16:05 flat, 16:20 outcome
  with decision-log completeness as the headline. Nightly tick proof from the 18:00 ET Globex open PASSED.
- Kalshi shadow-only: both owed readiness self-checks 9 PASS/0 FAIL; two-scorer AGREE 6853/0; no live order, no gate movement.

## ESTATE / UPKEEP (Team E)
- Nox's N1/N3/N4 executed with before/after evidence (outis-pilot stopped+disabled, data kept; two dead Talos units archived;
  caches removed ≈299 MiB; wallet paths untouched). N2+N6 root relay prepared and HELD (not overnight-sent). N5 scheduled Monday
  post-16:20. Estate sweep: the seven held PRs were closed by the owner himself; four repos archived by the owner — the Job-18
  decision pack is moot by his own acts. Monday 07:45 sweep armed.

## NAMED RESIDUALS / OPEN RULINGS (for the morning conversation, not action items)
facade model-mode defect (0.3.2 candidate) · Windows OOBE/desktop + driveable channel (attempt running) · Ergon Windows companion
package (G1 whole) · R3/R6 blockers · abort-contact fixture-geometry change (needs a ruling before that cell can pass) · T7 stale
battery expectation · conservative roundtrip `real_lane` label (doc/code mismatch) · `dshVersion` null · the Ergon v2 rendered look
is his grade (the only visual judgement we cannot make for him).
