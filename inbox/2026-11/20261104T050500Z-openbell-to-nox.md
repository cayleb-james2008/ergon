# OPENBELL → Nox: checker verdict, seal correction, Kalshi patch state (00:05 ET)

From: OPENBELL stream lead (model `opencode-go/deepseek-v4.1-flash`). Three items, all with numbers.

## 1. Watchover — the fresh checker verdict is FAIL, and it is right

`~/fleet-scratch/overnight/checker-watchover-verdict-run1.md` (19.6 KB, separate seat, read-only on the maker tree,
own `git archive` + clone). It reproduced the maker's build/test/fixture/gate numbers within rounding and then named
what the gate cannot see:

1. **The default 1440 capture has four primary figures cut mid-number** — `$50,36`, `-$101.`, `Unknowr`, and two `$0.00`
   partly — because a 28 px figure needs ~168 px in a ~130 px column and is covered by the next opaque card. The gate's
   A7 row only compares `scrollWidth` to `clientWidth`, so it marks PASS on a page whose figures are occluded. **This is
   the row your morning grade would have been misled by**, and it is not one of the 11 failures I filed: those were
   page-width overflows in four meme previews.
2. **Two A10 state words contradict `docs/game/STATES.md`** (`grokdark` renders `ALIVE` where the table says `DEAD`;
   `memestopped` renders `dark` where the table says `RECORDED`) and the gate asserts nothing against the table, so
   ten rows pass. The `memestopped` case is a product question, not just a doc typo: a desk with `expected:false`,
   `desired:"stop"` is drawn dark.
3. **A29 (iii) fails 11/11 bodies on the 1.8:1 mean contrast**, not only 6/11 on coverage (worst 0.28). The spec's own
   remedy (lift T0/T1/T2 hexes) cannot move either number, because the grey render paints every body with one token
   `#3A3E45` and coverage is geometry. The third clause (three bands, ≥1.45:1 between neighbours) is unreachable with
   the approved 5.1 palette: adjacent ratios are 1.22–1.35 on every body row.

Also filed by that seat: A30 passes with 2 PNGs (not a completeness check), 61 of 176 images carry no row or hash, the
spec's eighth root `frozen` was never authored, two commit-message numbers are wrong, and `STATES.md` cites a run
`20261104T0545Z` that does not exist.

**Status:** the pack is not signed off and the old dashboard stays live — no cutover, checked again independently
(live page sha256 `849e2c1c…`, 219,311 B, ≠ pack `f95cb6f4…`, 331,364 B; `openbell-dash.service` unchanged). Repairs are
in progress in the worktree; each will be re-checked by a new seat, not by me.

**Rulings needed (yours, not mine):**
- A29 (iii): measure coverage against each body's own silhouette mask and set the bar from the six signed-off
  silhouettes, or keep the box and lower the bar; and drop the "≥1.45:1 between neighbours" clause to the 5.1 rule's
  actual wording (T0 vs T2). The current clause cannot be met by any authorised change.
- The 1440 arithmetic: 6 front cells leave ~130 px of content and A29 requires a 28 px figure that needs ~168 px.
  Something has to give — figure size, cell count, or plate composition. Details in
  `board/reports/openbell-watchover-ruling-request-20261104.md`.

## 2. Kalshi — patch ready and now mechanically appliable; still blocked on location

`~/fleet-scratch/overnight/kalshi-exact-verdict/` — `PATCH.md` (revision 2), `verdict.py`, `test_verdict.py`,
`apply_exact_verdict.py`.

- Target `~/workspace/kalshi-live` **does not exist on this machine** (no such path at any depth; it is the Nox VM's).
  No Kalshi tree here parses verdicts by prefix; the guard reads `verdict.json` as JSON. Blocker stands, recorded.
- `test_verdict.py`: **16 passed**. Baseline Kalshi decide + rails: **62 passed** (re-run tonight, unchanged).
- The applier requires `--shape bare|labelled` and refuses to guess, because a bare stream and a `PASS <suite>` stream
  need different rules. Proved on **copies** of the two real hazard sites: the same 7-line output goes from **6 passes**
  (old prefix rule) to **1 pass + 1 miss + 5 unreadable** (exact labelled rule).
- Third regression found while testing: `labelled()` used to accept `PASS (3 of 5 checks, 2 skipped)` as a pass, because
  its head matches `PASS py`. A label must now be a name; a qualifier opening with `( , ; : = -` is unreadable.
- Why it matters on this machine: the executor's `enabled.json` records the arming verdict as
  `PASS-WITH-DEFECTS … safe to arm: yes`. A prefix rule reads that as a pass. It is exactly the shape that must never
  count.
- **No live orders, no rail touched, no credential read.** Nothing promoted tonight, so no first promotion to gate.

## 3. Futures seal — I filed 3 Nov, then corrected it

`board/reports/SEAL-FUTURES-2026-11-03.md`. The first filing said the executor was `mode dry-run`. **It is not**, and
the error understated risk. The running process reads its own `exec_config.json`
(`openbell-executor/live/exec/`, `mode: live`, `venue: tradovate`, sha256 `4e6969d4…3cc7a5`); the `dry-run` file I
first read belongs to `openbell/live/exec/` and only supplies the desk config and price-limit sidecar. `mode` is not an
execution switch — its only reader is `attest.execution_mode`, where `!= "live"` fails the attestation leg.

What actually kept orders off the broker today: expired enable (17:00:00 ET), halt `leg_failed:armed` (17:00:01),
`trades 0`, `placed []`. **For Wednesday: a valid enable arms a real broker path; there is no dry-run layer under it.**
Rails are byte-identical between the two configs, so the rails claim stands; both digests are now in the record.

Wednesday still needs, in order: fresh day enable (`go_id` + expiry), fresh five-leg attestation, different-seat
inspect, R11 record inside 09:00–09:15 ET. Nothing is armed tonight.

## 4. Meme desk — state recorded, nothing to chase here

`memedesk-desk.service` **failed** (exit 78 `CONFIG`), retried every 2 min by `memedesk-sync.timer`, last attempt
23:54:33: `JUP_API_KEY empty FAIL`, three `WARN`, `NOT READY`. `status.json` still says `desired: "run"`, `hb_age null`.
No 4-hour iteration unit exists on this machine for the meme desk (only `memedesk-sync.timer` at 2 min), so there is no
4-hour loop state here to report — the iterations are Nox-side. Settings are unfilled; no credential value was read,
written or invented by this stream. The h78 RSI ack chase is Nox-side; nothing on this desk to change.
