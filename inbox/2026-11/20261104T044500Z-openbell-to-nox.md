# OPENBELL stream → Nox / board — overnight evidence (4 Nov 2026, 04:45Z)

Lead: OPENBELL stream, model `opencode-go/deepseek-v4.1-flash`. Orders: `20261104T034900Z`, `20261104T035300Z` (§3).
This filing is evidence, not a completion claim: one row per ordered item, what is proven, what is open, what the
smallest next act is. No cutover, no live order, no rail touched, no credential read.

## 1. Watchover dashboard — Phase 2 finished, fixture build, render gate (OLD DASHBOARD STILL LIVE)

Worktree `/home/cayleb/workspace/openbell-watchover`, branch `design/openbell-watchover`.
Commits: **`b10291c`** (code) then **`61c3ebd`** (the render set). Not pushed.

**Done (proven by running):**
- **Seven SPEC §20 fixture roots authored** (`design/watchover/fixtures/`, `tools/make-fixtures.py`), each verified
  in-process against `live.dash.snapshot` before it is kept: `engine-stopped`, `feed-down`, `watcher-off`, `restored`,
  `grok-frost`, `arena-missing`, `meme-paused`. `--check` → 7/7 PASS. Two roots needed more than the literal edit and
  the reason is in the script: a stopped watcher has not rewritten `account_state.json`, so `t` goes too; a restored
  balance only reads as restored when `dom` is cleared.
- **Render gate** (`tools/render-gate.mjs`, `npm run render`): A10/A30 screenshot set (15 served previews + 7 roots ×
  390/1440/1920 × canvas off/on), A23 silhouette matrix (6 bodies × 9 poses, 64×64, pairwise Hamming), A29 (i) DOM
  sizes + sampled contrast, (ii) 4× distance stand-in, (iii) body-box coverage/contrast. One command, reproducible.
- **Run `renders/20261104T0620Z/`**: **176 PNGs**, `notes.md` (commit, page digest, fixtures, pass table) +
  `results.json` (every capture, its state words, sha256). **169 pass / 11 fail** — the failures are named below.
- **A23: pairwise minimum Hamming 493** (sentinel:dead vs orbiter:alive), threshold 20.
- **A29 (i)/(ii)** pass: front figures 28 px, shelf 24 px, state words 13 px at weight 700, smallest other text 12 px
  (chip only), sampled contrast min 12.00:1 (threshold 7:1), figure glyph 6 px and state-word 3 px at the 4×
  distance stand-in (thresholds 5 and 2).
- **`docs/game/STATES.md`** now exists: the expectation every captured state word is read against, and the captured
  column matches it for all 8 roots (e.g. `engine-stopped` → orb15 `DEAD`; `grok-frost` → grok `DEAD`;
  `arena-missing` → both arena versions `DEAD`; `watcher-off`/`restored` → account-1 `DEAD`).
- **Two live layout defects found by the render work and fixed** (both would have made the owner grade broken
  renders): the Details panel was never hidden — `hidden={!open}` lost to the panel's own `flex` display, so every
  plate was ~1,500 px tall and the shelf row sat below the fold; and the front row/record tables forced the page
  wider than the viewport (1,978 px at 1920, 1,500 px at 1440, 454-504 px at 390). After the fix: 1920/1920,
  1440/1440 and 390/390 for the base fixture and six of the seven roots.
- Gates: `npm run build` **331,320 B / 100,739 gzip** (budget 112,640); `npm test` **429 / 422 pass / 7 self-skip /
  0 fail**; `npm run csp` ok; `npm run check` (tokens + palette) ok.

**Open — 11 failing gate rows, recorded not massaged (this is the whole open list):**
1. `preview-memestale`, `preview-memedark`, `preview-memestopped` at 1440x900 (canvas off and on): page 1,485-1,494 px
   wide in a 1,440 viewport.
2. `preview-memegone` at 1920 (1,940/1,920), 1440 (1,540/1,440) and the same plate in the other meme states.
3. `A29 (iii)` body-box coverage 0.28 against the spec's 0.60 (contrast half passes).

**Why they are open, stated plainly:** at desktop the front row holds 6 cells; at 1440 a cell is 224 px and its
content width is ~130 px after the 48 px glyph, padding and gap. The spec requires a **28 px** primary figure there,
and a 28 px `$50,361.80` is ~168 px — it cannot fit beside the glyph. The meme plate adds its threat/feed rows on top.
So A8 (all plates inside a 1440 viewport) and A29's 28 px minimum cannot both hold with the current plate
arrangement at 6 cells: **this needs a ruling or a plate-lane redesign (glyph placement / age column), not a CSS
patch**. A29 (iii)'s 0.60 coverage looks unreachable for an articulated body inside its projected box (the best body
measures 0.28 while the render looks correct); the tone-lift failure rule in A29 does not affect coverage, so a
ruling is needed on that threshold too. Both are written into the run's `notes.md`.

**No cutover.** `openbell-dash.service` still serves the OLD page from `/home/cayleb/Work/projects/openbell/live`
(read-only to this stream; `git status` there shows no change from this workstream). The owner grades renders only.

## 2. Kalshi — exact-verdict fix: patch prepared and tested; LOCATION BLOCKER

- `~/workspace/kalshi-live` **does not exist on this machine** (no `kalshi-live` directory anywhere). The Kalshi code
  that does live here (`openbell-kalshi` `live/kalshi/` + `live/kalshi_exec/`, `openbell-kalshi-desk`,
  `kalshi-shadow-data`, `kalshi-pack23-wt`) has **no PASS-verdict decide loop**: `decide.py` decides from prices, and
  the order-path guard reads `verdict.json` as JSON (`gate_passed: true`), exact by construction.
  The only two instances of the prefix-match hazard on this machine are `Work/session-artifacts/overnight-dashboard/
  {server.py:290, runctl.py:190}` — a run/gate dashboard, not the Kalshi desk, and not touched.
- **Patch prepared and tested**: `~/fleet-scratch/overnight/kalshi-exact-verdict/` — `verdict.py` (whole-line
  case-exact PASS/FAIL, `tally`, and `labelled()` for a checker that prints `PASS <suite>`), `test_verdict.py`
  (**14 tests, 14 pass**), `PATCH.md` with both call-site shapes and the diff. The regression the order asked for is
  in the test: 10 partial forms (`PASSED`, `PASS-WITH-WARNINGS`, `PASS (3 of 5 checks, 2 skipped)`, `PASS; see notes`,
  `PASSPORT` …) are rejected, and the same three lines score 3 passes under the old prefix rule and 1 under the exact
  rule.
- **Baseline checker re-run** (read-only, this machine): `live/kalshi/test_decide.py` + `test_no_order_path.py` +
  `live/kalshi_exec/test_guard.py` + `test_static.py` → **62 passed**. Order path untouched; no order placed; rails
  untouched. Shadow desk liveness at the same minute: poll `errors_1h 0`, `gaps_today 0`; shadow `decisions_today
  1944`, `errors_1h 0`.
- **Smallest next act (Nox):** copy `verdict.py` + `test_verdict.py` to the Nox VM beside the decide loop, apply shape
  A or B at its one call site, run its own suite plus these 14 tests, record the diff.

## 3. Futures loop + executor liveness for Wednesday, and the 3 Nov seal

- **Seal filed:** `board/reports/SEAL-FUTURES-2026-11-03.md` — the day's facts, the loop's hash-chain verification, the
  evidence digests, the Wednesday requirements, and what the seal does not claim.
- **Loop:** `openbell-futures-loop.timer` active; `python3 -m live.loop.cycle --status` → `FRESH … chain ok`; 13 cycles
  on 3 Nov, all `HOLD`, cause `FEED_SILENCE`; chain head `f0f53954…21335`.
- **Engine:** running, feed `ws`, `market_open true`, `pos null`, 0 trades.
- **Executor:** running since 11:14:32 ET in **dry-run**, 0 trades, `placed []`; attestation **PASS** (five legs,
  different-seat inspect); enable expired 17:00:00 ET and the day halted `leg_failed:armed` at 17:00:01 — as designed.
- **Rails unchanged** (`exec_config.json` sha256 `c924d3fd…315eff`, mtime 2026-11-01; `deny.json`, `rails.py` likewise).
- **Recorded, not hidden:** `flat_proof.json` at 23:09 read `positive:false, "not flat or unreadable"` while both
  trading records show no position — an unproven flat at the read.
- **Wednesday needs (nothing armed by this stream):** a fresh day enable (`go_id` + expiry), a fresh five-leg
  attestation, a different-seat inspect, and the R11 record inside 09:00-09:15 ET.

## 4. Meme desk — state recorded, blocker named

- `memedesk-desk.service` (SHADOW) **FAILED**: `desk-start.sh` exits 78 `CONFIG` on every 2-minute sync retry —
  `JUP_API_KEY empty FAIL`, `LIVE_WALLET_PUBKEY / HELIUS_API_KEY / RPC_URLS empty WARN`, `NOT READY`.
  `var/uptime/status.json`: `desired: "run"`, desk `failed`, `hb_age null` (no heartbeat ever for this state).
- Running: `memedesk-refresh`, `memedesk-awake`, `memedesk-uptime`; sync timer healthy.
- So the 4-hour iterations are **not running**: the desk cannot start until its settings are filled. No credential was
  read, written or invented by this stream. The RSI h78 ack chase stays Nox-side as ordered.

## Files
- Progress: `~/fleet-scratch/overnight/openbell-progress.md`
- Render set: `/home/cayleb/workspace/openbell-watchover/design/watchover/renders/20261104T0620Z/` (notes.md, results.json)
- Patch: `~/fleet-scratch/overnight/kalshi-exact-verdict/` (PATCH.md)
- Seal: `board/reports/SEAL-FUTURES-2026-11-03.md`
- Checker (fresh seat, pinned): verdict to `~/fleet-scratch/overnight/checker-watchover-verdict.md`, running at filing time.
