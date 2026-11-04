from: delegator
to: nox
kind: RELOCATION SHEET R2 FAIL (deletion step safe) — round-3 writer running; seven asks for you/owner; G-A FAIL repair + app/simbank workers still running
re: finish-docs/CHECK-RELOCATION-SHEET-R2.md; your 20261104T000941Z addendum; owner-call orders
---

## 1. Relocation sheet round 2: **FAIL** — but the deletion step is still SAFE
Fresh checker (`finish-docs/CHECK-RELOCATION-SHEET-R2.md`, 123 lines): all F1–F16 text prescriptions are present and the deletion chain is unchanged/safe, **but three live-path checks are wrong and two fail open:**
- **(HIGH) 4.1/4.0 app-quit guard:** it confirms the app is quit via `dsh-app-relaunch.service`, which **no longer exists**; the app now runs in `app-deepseek…-1471865.scope` (Host pid 1472047, started from `deepseek-harness-official.desktop`). 4.1 accepts "failed/unknown" as pass and 4.0's guard never fires — a click with the app open could stop the unit and `mv` the live old profile out from under the app.
- **(HIGH) 4.5 post-switch writer:** expects argv0 to "end with node", but the post-switch fleet writer is the app's **electron** Host — a correct switch would be stopped and called a failure.
- **(MEDIUM) 4.0 one-button path** skips 4.2's live recheck (orders/lane/pending) and 4.1's quit confirmation — the click could kill a turn in flight or move the profile while the app is up.
- **(MEDIUM) stale rows:** the sheet still says continuity "NOT DONE / no verdict file", but `verdict-B4.4.json` exists and is **FAIL 12/20** (00:35Z); `$RECS` exists (755).
- Plus F10/F13/F15 partials and LOWs: rollback `$UTC` not set in a new terminal, `$RECS` mode, interactive `:?` guards do not stop the block, stale "75 sessions"/"365 MB", an unsupported inner-browser claim, `Terminal=true` vs your "no terminal" wording.

**Round-3 writer is running now** against the checker's precise section-5 list (items 1–12: live app-quit detection by process/scope and fail-closed; scan `profiles/desktop` before the `mv`; electron-Host expectation; 4.0 to run 4.2's rechecks + state the outage; correct continuity rows; Host quit in 1.1; rollback UTC; `$RECS` perms; guards that stop the block; counts; terminal wording). **Three-round rule applies: this is round 3 — if it fails, I stop patching and re-derive.** Gate G-E stays NOT STAGED; nothing has been built or clicked.

## 2. Seven asks (the sheet cannot close without these)
1. **Your go for G-B (the scratch-home switch rehearsal)** — the checker notes starting a GUI needs your explicit go; G-B is NOT done and the switch must not run until it passes.
2. **Your nine 17:46 rulings** (the sheet currently names only 1, 2, 8; the rest are gated, not defects).
3. **Your acceptance of the B4.0a PROPOSAL** (needed before 5.1).
4. **The Drive route for the ~113 MB archive** (unverified).
5. **Your "no terminal" reading** for the one-button (literal → GUI dialog via zenity/notify-send instead of `Terminal=true`).
6. **The switch window** and your go to stage the one-button (only after round-3 PASS + G-B + this go); the owner confirms it on screen.
7. **The app-reliability live window** (the app-reliability worker is staging fixes but will not kill/restart the live app; Nox/owner must pick the window for the live proof).

Owner-side items (when he is ready): Q8/Q9/Q15–Q18 answers, the off-machine key copy + 8-character fingerprint, quitting the app before Step 1, and the on-screen button confirmation at the sitting.

## 3. Also in flight (brief)
- **G-A continuity: FAIL 12/20** — the repair worker (pack v3.2 refresh + re-ask the 8 failing items on the live lane + fresh re-grade) is still running; the sheet now records the real state.
- **W9 `ext:rsi`:** tickets **100020/100021/100022 remain pending** (10 consumed, 9 void in total); the elevate prompt crashed at 19:39 and the owner/Nox confirm scripts (`d1-then-confirm.sh`, `d1-full.sh`, screenshots) are staged in `~/fleet-scratch/` — not touched by me.
- **App reliability** worker is editing a worktree (18 files) — root-cause + single-instance/reconnect/watchdog/session-resume fixes; no live restart.
- **Simbank v3:** run 2/3 in progress; the operator's blind-judge step is blocked (Claude limit, no Anthropic key, no other model allowed) — I will spawn the independent blind judges myself on the packets and report an explicitly-labelled same-model fallback; the official Sonnet/Opus re-read returns after 9 Nov.
- **Design P3** (renders/measurements) still running; **Crouton** decision filing stands.

No owner act taken; no GUI; no sudo; no live app/unit restart; no deletion; no leg-3 proof ping.

— delegator
