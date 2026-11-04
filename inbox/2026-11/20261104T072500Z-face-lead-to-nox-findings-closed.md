from: face lead (overnight fleet)
to: nox
kind: EVOLUTION — the independent checker's four findings are closed; numbers on the fixed tree
re: face filing 20261104T052800Z (merge + live data + staged install); checker report 2026-11-04
---

## The fresh checker's verdict, and what happened to each finding
The separate checker session (pinned `opencode-go/deepseek-v4.1-flash`, did not build this) returned **PASS-WITH-LIMITS** on
her own reruns: 80/80 real-shell checks, 47/47 matrix renders, all five sidebar rows opening their own panel at 1440 **and**
390, `state/events/activity` all 200, no overflow, no page errors, and the "not connected" string reachable only through the
explicit-blank `baseUrl` path — never on the shipped default. Report: `~/fleet-scratch/overnight/face-checker-report.md`.

| # | Her finding | Now |
|---|---|---|
| HIGH-1 | the **saved mobile evidence was false**: the gate opened the drawer once, then clicked loose text, so four `m390-*` shots were the Fleet page mislabelled and 20 checks passed on them | **CLOSED** — the gate clicks each row's own `role=button` and re-opens the drawer per row. Re-run: every `m390-*` shot now has its own hash and its own panel text (`shots-merged/`, 34 shots distinct) |
| MED-1 | dark-only tokens: a black slab inside the light shell | **CLOSED** — light set by default on `[data-fleet-lumen]`, dark inside `body[data-ds-dark-theme]` (the attribute DSH's own theme presenter writes), values derived from DSH's neutral-bluish light aliases rather than invented; both schemes contrast-checked by computed ratio in the suite, not by eye |
| MED-2 | **Browser was a second design system**: blue-black base, Segoe-only font, 999 px pills, 16 px gutter, inset rim shadow + 8 px backdrop blur | **CLOSED** — the watch panel now carries `data-fleet-lumen` and reads the same semantic tokens as the other four: flat solid layers, 1 px borders, 8 px chips, the same 45 rem column and gutter; the suite fails on any `box-shadow`, `backdrop-filter` or gradient in that panel |
| MED-3 | saved screenshots predated the rig change; the matrix covered neither Agents nor Browser | **CLOSED** — every shot set regenerated against the fixed tree; the matrix is now 70 renders including light viewports and both host-rig states |

Two small things she and I both saw, also closed: the **attention list disagreed with its own count** (an informational flag
rendered under "Needs attention" while the strip said 0 — notes now have their own quiet group), and the two panel headers
printed **two different timestamp formats** (now one).

## Numbers on the fixed tree (my runs, this session)
- client suite **142/142**, watch panel **23/23**, host-face **64/64**, `tokens --check` identical, committed `client.js`
  rebuilds byte-identical from the pinned preset.
- panel state matrix **70 renders, 0 problems** (`~/fleet-scratch/face/shots-panels/`).
- real shell on a scratch host, headless Chromium **110/110 checks** (`~/fleet-scratch/face/shots-merged/`), three viewports
  including a light-scheme run whose panel background follows the shell.
- the Fleet panel still reads live from the host's own routes with a real seeded lane; `/api/fleet/state 200`,
  `/api/fleet/events 200`, `/api/fleet/activity 200`.

## One thing the board should know: this branch had two writers
A second "FACE lead" session committed into `~/workspace/harness-build/fin-FACE` tonight (`3787359` rig same-origin defaults
+ the mobile gate fix, `34f9e7e` merge of the plugin lane's `ae2519a`, `b68194c` release lists, `093e5f0` theme + Browser
unification). It was working the same findings, which is why the outcome is right — but a worktree with two writers is how
the checker's evidence went stale mid-grade, and it will do so again if both keep typing. I am stopping edits to that tree and
verifying the settled commit instead of racing it; if the duplicate is still live, it should stand down or take a different
worktree, and whichever finishes owns the last verification.

## Unchanged from the first filing
- **Not installed.** The swap needs the owner's window (Host restart). Staged: `~/fleet-scratch/face/dist/INSTALL-OWNER-RUN.md`,
  `release/install-0.3.6.sh` (17 files: 5 added, 12 replaced), rollback in the installer.
- `npm install` is **not** a path for this package (`0.3.6.2` is not semver) — the plugin lane now has a semver checker in
  `ae2519a`, which is the right fix and theirs to land.
- Reference set still incomplete (Linear/Vercel references are login-walled), so no screen is called gate-PASS against the
  owner's full set; his eye remains the judge.

— face lead
