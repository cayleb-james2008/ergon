# OPENBELL — final evidence, second-checker verdict on the watchover pack (4 Nov 2026 ~06:25Z)

Addendum to `20261104T044500Z-openbell-to-nox.md` (same seat) and `20261104T055000Z-openbell-collision.md` (the
duplicate-lead note). Read-only seat since the collision: this filing adds no edits, only a verdict.

## Watchover — ready for the owner's grade, one spec conflict open

Final commits on `design/openbell-watchover`: **`3e52d34`** (fix + gate) and **`59f525a`** (docs), tree clean.
Graded render set: **`design/watchover/renders/20261104T1030Z/`**.

| check (my own run on HEAD, read-only) | measured |
|---|---|
| `npm run build` | 332,063 B / **100,924 gzip** (budget 112,640); sha256 `c709562f…` — **byte-identical to the digest in the graded run's notes** |
| `npm test` | **429 / 429 pass / 0 skip / 0 fail** |
| gate rows | **415: 413 pass, 2 fail** — both are A29 (iii) (coverage 0.28 vs 0.60 and ring contrast 1.35 vs 1.8; the tonal-band clause 1.23:1 vs 1.45) |
| A10 completeness | **118/118** captures, `frozen` included at 390/1440/1920 |
| figure fix, independent ink test | every `[data-value]` painted inside its plate and on one line at 1440 / 1920 / 390 and at 1440 `preview=memegone` (61/61/61/56); Account 1 reads `$50,361.80` whole |
| gate teeth (mutation) | removing the compact-plate block in my own copy reproduces the original defect (4 figures past the plate at 1440, none at 1920) **and the gate fails it**: `figures … 4/5 … past plate`, `containment … 70 element(s)` |
| cutover | **none** — `openbell-dash.service` active, `:8791` serves the OLD dashboard (no watchover/scene- references) |

Verdict: **PASS WITH ONE KNOWN SPEC CONFLICT.** Full report: `fleet-scratch/overnight/checker2-watchover-verdict.md`.

Bookkeeping nit for the morning record, not a reason to re-run: the graded run's `notes.md` names commit `9f6b399`
("dirty tree at render time") because the run started before the fix was committed, while its page digest matches a
fresh build of HEAD byte for byte. The graded artefact is the committed artefact.

**Open for Nox/owner:** A29 (iii) as written is unsatisfiable for an articulated body inside its own bounding box, and its
tone-lift remedy cannot move coverage, geometry or the 1.43:1 grey. Ruling request with the arithmetic:
`board/reports/openbell-watchover-ruling-request-20261104.md`. The compact plate lane now in the pack is option 1 of that
memo applied narrowly (760–1550 px), so the ruling also decides whether it stays.

## The other three items, unchanged and still standing

- **Kalshi:** patch + 14 tests in `fleet-scratch/overnight/kalshi-exact-verdict/`; baseline 62 passed; no order, no rail
  touched; **blocker** `~/workspace/kalshi-live` is not on this machine and no local Kalshi decide loop parses verdicts.
- **Futures:** `SEAL-FUTURES-2026-11-03.md` filed — loop chain `ok`, 13 cycles HOLD (FEED_SILENCE), engine live 0 trades,
  executor dry-run 0 trades, attestation PASS with a different-seat inspect, rails unchanged; Wednesday needs a fresh
  enable, attestation, inspect and the R11 record.
- **Meme desk:** `memedesk-desk.service` fails every 2 min on `NOT READY` (JUP_API_KEY empty); `hb_age null`; the 4-hour
  iterations are not running. No credential read or written.

## Coordination

Two leads on one worktree cost one killed gate run, four render runs where one was needed, and the same bug fixed,
re-broken and re-fixed inside an hour. The delegator's resume prompt is not idempotent; a resume should detect a live
seat (`CLAIM` file or pid) before spawning, or partition the work by path.
