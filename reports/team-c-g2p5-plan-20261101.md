# TEAM C — G2.5 (CDP/DOM gate): PLAN — 1 Nov 2026

trading-lead, 2026-11-01 12:1x ET / 17:1xZ. **Text only: no tree edits, no live action, no order, no credential, no
spend.** G2 is signed against `live/exec` digest `32dd2bea…` / commit `0893497` and the tree stays frozen; G2.5's evidence
work is read-only page work + text, and the executor-side guard implementation lands in the **G3 package** (which re-binds
a new digest under its own checker gate). Gate definition: design v4 §D.

## 1. What G2.5 must produce (checker signs, veto)
| Evidence | Required | Veto if |
|---|---|---|
| **Page snapshot** | recorded snapshot hash of the order-ticket page (DOM/HTML + screenshot), with capture timestamp and the page URL | snapshot not reproducible / page identity unclear |
| **Pinned selector map** | every control the executor may use (qty, stop, target, buy, sell, flatten, positions panel/rows) mapped to a selector pinned against that snapshot hash | any needed selector unpinned |
| **Tab-navigation test** | ticket ↔ positions navigation, selectors re-resolve, and the **expected-ticket-page guard** refuses any send while the page is not the expected ticket | a send is possible off the expected ticket page |
| **Expected-page guard test** | the guard's identity check (page hash/required elements) passes on the ticket and refuses elsewhere | guard can be satisfied by another page |
| **Per-step screenshots** | screenshot per step in the file, no step sends | any step places/cancels/replaces |

## 2. Preconditions and the window (CONFIRMED by the Lead, 2026-11-01)
**Window: post-16:20 ET Monday 2 Nov — after the 16:20 outcome check, desk flat. Not earlier. Two conditions:**
(a) the capture needs a **signed-in Tradovate tab** (weekend tab missing; Monday's 09:05 guard session provides it);
(b) it **must not overlap the live window 09:05–16:20 ET**.
**Same post-16:20 slot also carries the approved N5 stop** (Codex extension-host PID 2960775, quick, **never touching
Chromium PID 2960519**). Sequencing recommendation: **N5 first** → verify `ss -ltnp | grep :9222` still shows chromium
2960519 and the desk feed error class is unchanged (no-tab / feed-retrying, not "no Chromium on CDP port 9222") → then
**G2.5 read-only capture**. N5 must target PID 2960775 only; if it respawns as a Chromium child, note it and leave it (no
broader kill). G2.5 remains 100% read-only: no clicks on ticket controls, no typing, no sends.

### 2.1 Original preconditions (unchanged)
- The executor drives the **already signed-in `trader.tradovate.com` tab** on Chromium `127.0.0.1:9222` (today: Chromium
  pid 2960519 up; the Tradovate tab is **missing** on the weekend, so the capture waits for a signed-in tab).
- **Key constraint:** that tab is also the live feed's attachment target. Navigating it away from
  `trader.tradovate.com` can break the feed. Therefore G2.5 page work runs **outside Monday's 09:05–16:20 ET trading
  window** (recommended: Monday after 16:20 ET, desk flat and idle; or a later non-trading window), with the guard owner
  informed and the tab left on `trader.tradovate.com` afterwards. No order actions at any point.
- No credential, no sign-in: a fresh credential/2FA demand = STOP for Cayleb. All reads obey §A.2 (no cookies, storage,
  tokens, profile material; DOM text/attributes of the ticket and positions only).

### 2.2 Post-16:20 Monday run-sheet (planned, no tree edits)
1. 16:20 outcome check complete, desk flat (per packet v7).
2. N5 stop: PID 2960775 only (upkeep-lead executes; never Chromium 2960519).
3. Verify: `ss -ltnp | grep :9222` → chromium 2960519 still listening; `tabkeeper.json` + engine feed error class unchanged;
   Tradovate tab still present and signed in.
4. G2.5 P2 snapshot (read-only) → P3 selector map (time permitting); leave the tab on `trader.tradovate.com`; write evidence
   under `board/probes/team-c/g2p5-2026-11-02/`.
5. If the tab is missing or the feed breaks, STOP and record — do not improvise a sign-in (fresh credential/2FA = STOP FOR CAYLEB).

## 3. Batches (each ~30–60 min, checkable)
| Batch | Work | Done-condition |
|---|---|---|
| **P1** | Window + scope confirmation; checker pre-sign request (fresh seat): what will be captured, the no-send boundary, the tab-risk note | pre-sign recorded in board/probes/team-c/g2p5-* |
| **P2** | Read-only page snapshot: `GET /json/list` → the Tradovate tab; DOM text/HTML hash + one screenshot; recorded capture timestamp/URL | snapshot hash + screenshot on disk, reproducible twice |
| **P3** | Pinned selector map: locate each declared control by text/attribute and pin it to the snapshot hash; record how each was verified | every control mapped or explicitly reported missing (missing = veto) |
| **P4** | Tab-navigation + expected-page guard test: navigate ticket↔positions (test tab), verify selector resolution and that the guard refuses off-page; per-step screenshots | navigation + guard results recorded, no send |
| **P5** | Gate evidence pack + checker review: snapshot, map, tests, screenshots, risks; the guard **implementation** is specified for the G3 package (code lands there, not on the signed tree) | checker signs G2.5 or lists veto conditions |

## 4. What this must never do
No submit/click on ticket controls, no typing into the ticket, no cancel/replace/flatten, no credential entry, no
session-material reads, no changes to `live/exec` (signed tree frozen), no changes to `live/config.json`/engine, no action
inside Monday's trading window.

## 5. Sequencing after G2.5 → G3
1. **G3 package** (new digest, own checker gate) carries: C2/C4/C5/C6 fixes, R-1/R-2 hardening (full deny set or a
   constant pin + weakened-deny negative test), `flat_day.note` "fixture" wording + evidence correction, C3 waiver attached
   verbatim, the expected-page guard implementation + pinned selector map from G2.5, and the **first live zero-signal
   session capture** (same command; no completeness claim before it).
2. **G3 demo venue** (needs owner assets): demo environment + demo access (Cayleb/Nox); place/cancel/replace/dup-id/
   partial/reject + 1:1 joins; no live account.
3. **G4** after G1–G3 signatures + Cayleb's written go + kill-path drill + fresh firm-page re-read.

## 6. Artefacts / owners
- Evidence dir: `board/probes/team-c/g2p5-<date>/` (snapshot, selector map, navigation/guard results, screenshots).
- Owner: trading-lead; backup: lead; checker: fresh Tier S seat (pre-sign before the page work, sign after).
- Fences: no tree edits (`live/exec` frozen at `32dd2bea…`), no live action/order/credential/spend; Monday cards/guard-only.
