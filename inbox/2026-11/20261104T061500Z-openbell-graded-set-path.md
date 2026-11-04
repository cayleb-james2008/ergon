# OPENBELL — the set to grade is `renders/20261104T1245Z/`, not `1030Z` (4 Nov 2026 06:15Z / 01:15 ET)

Short, one defect, no new claims. Addendum to `20261104T062500Z-openbell-final.md` and to
`board/reports/OPENBELL-MORNING-20261104.md`.

## The defect

The morning record (written 01:04 ET) points the owner at `design/watchover/renders/20261104T1030Z/` in its
"BLOCKED ON OWNER" line and in "The smallest acts, in order". **That directory no longer exists.** It was an
intermediate taken before the A8 viewport row landed and it was removed when the final set was committed. The same
record's §1 table already names the correct set, so the record contradicted itself in two places.

Verified, not assumed — `design/watchover/renders/` on `design/openbell-watchover` holds exactly two runs:

| dir | status |
|---|---|
| `20261104T0620Z` | superseded, kept on purpose for the before/after comparison (it is the set that passed 169/11) |
| **`20261104T1245Z`** | **the set to grade** — 179 PNGs, 438 rows, 413 pass / 25 fail, commit `bd8f0451…`, build `c709562f…`, tool `53e28190…`, `dirty: false` |
| `20261104T1030Z` | **absent** |

A dated correction block has been appended to the morning record (its own text is untouched; the append says which two
lines to read differently). No re-run is needed: the app bytes in `1245Z` are `c709562f…`, which is a fresh build of
`bd8f0451` byte for byte.

## Re-checked at 01:12 ET, unchanged

- **No cutover.** `openbell-dash.service` active, `WorkingDirectory=/home/cayleb/Work/projects/openbell`, `:8791`
  serves `849e2c1c…` / 219,311 B with **zero** `watchover`/`scene-` references. The old dashboard is still the live one.
- **Futures loop on schedule:** `01:07:00` cycle fired, oneshot clean, `decision HOLD`, `chain ok`; next fire 02:07.
  `openbell-live`, `openbell-executor`, `openbell-dash` all `active`.
- Nothing armed, no order placed, no rail touched, no credential read.

## Still open for Nox (unchanged)

1. Three rulings, all with numbers in `board/reports/openbell-watchover-ruling-request-20261104.md`: A29 (iii) ring
   contrast basis, A29 (iii) adjacent-band clause vs the approved palette, and A8's viewport clause vs §5.3's stage
   formula on a 900 px-tall window.
2. The Kalshi apply on the Nox VM (`apply_exact_verdict.py` against the decide loop) — the decide loop is not on this
   machine, so the patch stays packaged and unapplied here by design.
3. Wednesday's four acts: fresh enable, five-leg attestation, different-seat inspect, R11 record in 09:00–09:15 ET.

Checker 4 (independent seat, read-only, own copy) is still gating `bd8f0451` + `1245Z`; its verdict is the last open
item on this stream.
