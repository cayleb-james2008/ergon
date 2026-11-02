# WAKE-UP REPORT — 2026-11-02 (refreshed 01:10 ET; owner asleep) — refresh #5 (B+A zones/layer-6, Play 9/10, two rulings with Nox, Windows final)
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
- **Windows VM — installed + OOBE first logon + RDP re-confirmed — driveable via RDP.** Deterministic install to a
  fresh qcow2 completed in ≈6 min 17 s; the guest boots its own UEFI "Windows Boot Manager" from disk0; the answer file's
  FirstLogonCommands ran to completion — text proof: host marker `?step=start` at 03:08:35Z and `?step=done` at 03:26:46Z — and
  **RDP answered at protocol level** (TPKT + X.224 Connection Confirm through 127.0.0.1:3389; reproduced by two seats).
  **Settled-probe re-confirmation — captured:** RDP **re-confirmed 6/6** back-to-back probes 03:37:36–03:38:26Z (TPKT + X.224
  Connection Confirm + RDP negotiation) — the surface is driveable via RDP. **SSH is carried as a named pending (Nox ruled, run 7):**
  TCP reaches the QEMU hostfwd but 6/6 probes got no SSH banner and the key round-trip timed out during banner exchange after the
  `?step=done` marker; the cause is guest-side and unverified. RDP is the driveable channel and SSH is not required for the surface
  claim; only periodic post-settle read-only re-probes are approved — no reinstall, restart or blind input. Guest state preserved
  (QEMU PID 757632 + both disks). The two rig root causes were found and
  fixed with evidence: (1) stale/foreign OVMF vars + missed El-Torito "press any key"; (2) an answer medium Windows could never see
  (8.3-only FAT16 name + `usb-storage removable=off`) → vvfat over `win/unattend/` + `removable=true`. Launcher patch filed
  (`run-win11.sh.patched` + diff; finalize-timing lesson recorded) — the rig launcher must adopt it. A fresh checker re-verified the
  upgraded filing (ACCEPT, honest) with two corrections being applied (companion-artefact wording; a rig-local placeholder password in
  the board copy of the answer file — redacted from both board copies within minutes, no real credential involved; Nox ruled the
  placeholder history is NOT scrubbed — fence item closed).
- **Windows residuals (named, not blockers):** SSH/OpenSSH still not answering (capability install finished, sshd not up — being
  diagnosed — the exact blocker is filed: no serving sshd while RDP is live); the OOBE desktop itself is not claimed (no image reads — the marker sequence and RDP handshake are the text/protocol
  proofs); the Ergon Windows companion surfaces remain package-blocked — companion SOURCES exist in the OpenClaw baseline repo but
  there is **no compiled/packaged companion artefact and no DSH-era Ergon package** (G1 whole).
- **S8 pendings dispositioned:** R4 phone surface = fixture-proven on the phone VM + product surface package-pending; R3 real
  shareable URL = carried (host + package absent); R6 voice = carried (package absent); R2 model-quality = proof pre-written, runs the
  moment roster activation lands; Windows companions = package-pending on an installed+bootable guest. The upgraded FL2 verdict will
  checker spot-verified (ACCEPT with two corrections, now applied as r3) and a settled probe re-confirmation is pending before any wording upgrade.

## ROBLOX — one-line status for the morning
The instrumented block is complete and five diagnostic Plays have each converted a blind failure into a named, checker-verified cause:
truncated upload channels (closed), starter/require and Instance-root legacy APIs (fixed), guarded fog (fixed), bounded retries + audit
(fixed), and now the **renderer/Wine limitation is isolated to whole-map overhead framings** (whole-highway, whole-world) while
`city-overhead` and 20 ordinary views capture fresh with `viewportMatchesPng:true`. The authorised **B substitution** (per-zone
functional overhead views) is implemented, checker-verified and partly exercised; the **A limitation** is recorded with the exact
build, fail-closed verdict and one-retest-per-new-build rule. Two small rulings are with Nox (abort-contact pursuit ramp; extending the
B substitution to world-overhead + optional zone ordering). **No cell pass is claimed** — the cell remains blocked on those two
rulings; every step is on disk with the checker evidence. Plays are budgeted and reserved; the sealed place/release has been unchanged
through every Play, crash and recovery.

## ROBLOX (Team D) — instrumented diagnostic block COMPLETE; two diagnostic Plays ran honestly
- Plans 2/3/4 + the C1 fix round all fresh-checker ACCEPTED (suites and mutation proofs reproduced from clean copies; sealed place
  `c6651793…` and release `84324d43…` unchanged throughout).
- **Diagnostic Play 1** (`traffic:abort-contact`, 1 of 3): cell FAIL fail-closed — instrumentation attributed it precisely: the proxy
  outran the ego, crossed `RETIRE_RADIUS=800` at t=22.283 s and the controller's DOCUMENTED `unobservedSince > 1` rule retired the body
  0.95 s later. **Not a product defect**; the remaining cell defect is fixture geometry (contact leg unreachable). A scenario-change
  design note is filed and awaits a ruling (offline, no Play).
- **Diagnostic Play 2** (`atlas:scout --seed 2134783819`): fail-closed timeout with the 45 s bound firing correctly — and the studio
  console revealed the real root cause: the seed patch called `apply` on an un-`require`d ModuleScript
  (`S.apply is not a valid member of …AtlasWorldSeed`), the starter crashed silently (outside its xpcall) so no runtime error/trace was
  written. **This SUPERSEDES the old "in-Play City+Highway rebuild hang" reading — the rebuild never ran.** Fixed (require the modules;
  wrap the whole seed patch in the error-reporting xpcall; offline Lune/Python tests), then a fresh 3-Play opening and further Plays
  closed the question: the repeat Play recorded a clean **`fastPathApplied=true`** (seed 2134783819 / layout 2, actual==expected,
  `rebuild=false`, starter applied before the world build, 4.25 s build) — **assert-and-skip suffices; no fallback needed; the
  named-seed blocker is resolved.** Each fix unmasked the next legacy-API layer (starter require → root check → client `FogEnabled`);
  the fog fix is approved offline and the next atlas Play follows its checker delta.
- Studio: pid 759088 (fresh opening after `studiocycle`), sealed place/build unchanged (`c6651793…`/`84324d43…`), 2/3 Plays used
  in the current opening (third reserved); no publication, no build.
- **Diagnostic Play 6 — retry net proven, defect attributed:** 3 retries at view-021 all returned byte-identical to view-020 → the
  duplicate is a **persistent overhead-frame render/capture stall, not retry logic** (the guard correctly refused dishonest evidence).
  Studio crashed on the fail-closed Play shutdown (106 MB userland minidump, no kernel GPU evidence) and recovered cleanly with
  `studiocycle` (sealed place/release unchanged through crash + recovery; fresh 0/3 opening). A bounded overhead-settle fix (1.0 s settle + re-probe for overhead kinds, failure-path audit) is **ratified and frozen**; if frames
  still freeze after it, an honest renderer/Wine-limitation note with plan options goes for a Nox ruling — no plan/validator change,
  view replacement or drop without it. Plays: 1/3 in the recovered opening (one reserved).
- **Diagnostic Play 5 (fresh opening, 1/3) — the cell now captures real evidence:** **20/24 views captured (20 distinct sha256, all
  accepted/stationary, viewport 1837×744)** and **`viewportMatchesPng:true` observed on every view — the first observation of that
  outstanding datum**; the guarded fog layer is clean in Play (legacy absence recorded, no false claim); `fastPathApplied=true` again.
  The run then fail-closed honestly at view-021 on a **stale duplicate capture**, which the guard correctly refused to keep — the
  capture-path layer is named and a bounded fresh-capture retry (max 2 attempts, stale rule stays final) is approved and being frozen.
  Sealed place/release unchanged; no cell "pass" is claimed (views 21–24 + runtime/validator still absent).
- **Diagnostic Play 7 — the honest limitation point:** the bounded overhead settle+retry ran (view-020 settled fresh; view-021 returned
  byte-identical frames on all 3 attempts, `cameraDelta:0`) ⇒ the **overhead frame is genuinely frozen on this Studio/Wine build**, not a
  race; the audit file stays consistent (21/21) and there was no crash this Play (Play-6's minidump = intermittent Wine shutdown crash).
  20/24 views captured with `viewportMatchesPng:true`; sealed unchanged; plays 1/3. **A ruling is queued for Nox on the limitation
  (documented limitation + named view vs per-zone functional overhead views vs framing-cost experiments vs capping the set; Team D
  recommends per-zone functional views + a documented limitation).** No plan/validator change will be made without that ruling; the
  ruled abort-contact fixture A+C fix is being implemented offline in parallel (no Play consumed), with Play B after its checker delta.
- **Next (ruled):** the client `FogEnabled` legacy-API fix is approved and being implemented (guarded capability accessors that
  record availability instead of raising) → freeze → checker delta → fresh 3-Play opening → one atlas Play; then the
  **abort-contact fixture geometry fix ruled A+C** (closed-loop pursuit with clamped closing speed + two named fail-fast guards;
  fixture/test files only) → freeze → checker delta → Play B in a fresh opening; third Play reserved.

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
facade model-mode defect (0.3.2 candidate) · Windows OOBE desktop not claimed + SSH channel pending + settled probe re-confirmation · Ergon Windows companion
package (G1 whole) · R3/R6 blockers · abort-contact fixture-geometry change (needs a ruling before that cell can pass) · T7 stale
battery expectation · conservative roundtrip `real_lane` label (doc/code mismatch) · `dshVersion` null · the Ergon v2 rendered look
is his grade (the only visual judgement we cannot make for him).
