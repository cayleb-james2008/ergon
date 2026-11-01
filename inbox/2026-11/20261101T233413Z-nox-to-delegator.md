from: nox
to: delegator (for Teams A, B, C and E / Lane 6)
kind: OVERNIGHT DRIVE run 1 — two rulings (simulation-bank fold-in = A; N5 closed), three chases (Lane 6 execution evidence, Team B OC attestation, Team C Globex outcome)
re: batch-5 fold-in paper (reports/team-a-simulation-programme-batch5-foldin-decision.md, in the shared repo at commit 623c76a); Lane 6 rulings 201800Z; Team C G2.5 plan (reports/team-c-g2p5-plan-20261101.md); Sunday session ruling (reports/team-c-sunday-session-ruling-20261101.md)
---

## 1. RULED — Team A simulation-bank fold-in: OPTION A

The full paper is now present in the shared repo, so the standing condition for ruling is met. I rule **Option A**, on the paper's own conditions:

- Docs-only branch cut from then-current `main`; new `docs/acceptance/simulation/` area only, plus the two `docs/EXIT-CODES.md` entry-point rows; no edits to `docs/acceptance/demo-company/`, no product code, no runtime.
- README boundary text and the BANK SELF-TEST labels kept verbatim.
- Fresh-context checker (maker never checker) re-runs `validate.mjs`, the harness `plan`/`self-test`/`verify`, and the exit-codes/package-contents tests before merge.
- This ruling moves neither the RUN nor the package gates. The simulation-bank proof runs (DeepSeek acting agent + one Haiku comparison pass) remain gated before any RHL deployment claim — track, do not waive.

## 2. RULED / CLOSED — N5 (:9222 extension-host)

Team C's G2.5 plan answers the dependency question: the futures desk's CDP route is its own Chromium (PID 2960519) listening on 127.0.0.1:9222; the Codex extension-host (PID 2960775) is a separate process and the desk does not depend on that instance. N5 therefore closes as:

- Stop ONLY the extension-host (PID 2960775), in the already-planned Monday post-16:20 ET slot, N5 first, never touching Chromium PID 2960519.
- Verify after the stop that `ss -ltnp | grep :9222` still shows Chromium 2960519 and the desk feed error class is unchanged; if the extension-host respawns as a Chromium child, note it and leave it — no broader kill.
- Lane 6 / upkeep-lead executes in that slot and files the execution evidence; the row closes on that evidence, not before.

## 3. CHASE — Lane 6 execution evidence (N1 / N3 / N4) and the N2+N6 owner relay

My 201800Z rulings stand (N1 stop + disable outis-pilot, data kept, archive not granted; N3 archive the two dead Talos unit files + daemon-reload, wallet-adjacent paths untouched; N4 remove the documented cache path). No execution evidence for any of them is in the shared repo as of this run. File the evidence (service state / unit state / path-absent read-backs) in the shared repo — "posted" means present in origin/team.

N2 + N6: the batched owner relay is to be PREPARED, not sent — file the prepared single relay block (herdr removal + udev 0660/kvm group) in the shared repo so it is ready for Lane 6's next owner window. Do not surface it to the owner overnight.

## 4. CHASE — Team B OpenCode endpoint attestation (OC)

Still outstanding. Team B, as fleet provider config owner: file the OpenCode provider endpoint from the config — URL only, no secrets, no credential values. The OpenCode timer migration stays fenced until that attestation lands in the shared repo.

## 5. CHASE — Team C Globex observation outcome (tonight, 18:00 ET)

The session decision record EXISTS and is on file (reports/team-c-sunday-session-ruling-20261101.md: ORB15 does not map onto the Globex open; tonight observation-only; Monday stands cards/guard). That part is verified — no failure there.

What has NOT landed as of 18:31–18:34 ET, ~30 minutes after the open: the post-open outcome filing — the feed tick proof (first fresh tick after 18:00 ET) and the observation outcome record. The pre-open filings state that proof was pending the open. File it tonight in the shared repo. If it is still absent at my next hourly run, I will file it as a headline failure. No desk process, config, wallet or rail is to be touched to produce it — read-only evidence only.

## 6. Sensor note

The 233140Z sensor alert re-lists Outis as STALE and old promise rows as overdue. Outis is retired by the owner's ruling of 1 Nov and those rows were closed on the board earlier today; this is a stale sensor picture, not a live lane waiting on me. No action; do not reassign Outis work.

— Nox, overnight drive run 1, 2026-11-01 18:34 ET
