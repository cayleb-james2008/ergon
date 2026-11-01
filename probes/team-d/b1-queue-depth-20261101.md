# B1 queue depth + next-block prerequisites (2026-11-01 13:22 ET)

## v1.queue state

- Done: entry 1/7 — `smoke` PASS (run `20261101T181436Z-smoke`, checks S1–S5 true; see
  `b1-capture-fix-smoke-20261101.md`).
- Remaining 6 Plays: `traffic:abort-contact` ×3, `atlas:scout`, `atlas:highway-extension`, `moving`.
- Grouping after the two retry smokes (each failed smoke consumed one of the 3 Plays per Studio process):
  **2 openings of 3 Plays** — (next) 3× `traffic:abort-contact`; then `atlas:scout` + `atlas:highway-extension` +
  `moving` (queue completes). The SPRINTS kit assumed smoke + 2 tac in opening 1; the retries shift it; record kept.
- After v1.queue, B1 still owes the 3 all-flags-off baseline runs (`atlas:scout`, `atlas:edge`, `moving` with
  `--visual-flags "...=0"`): 3 Plays, 1–2 openings.
- Plays used in current Studio process: **3/3**; queue paused, `next=traffic:abort-contact`; owner reopen required.
  Standard resume protocol is in `d5-window-day-runbook-20261101.md` §1b (MCP verify → re-sync from B1 tree →
  verify-clean (+clear-latch) → `--resume`, sealed hashes before/after).

## What each later sprint needs before it can start

| Sprint | Owner needs | Lead decisions | Other gates |
|---|---|---|---|
| **B1 remainder** | Studio reopens (2 for v1.queue + 1–2 for baselines) | none | quiet host; MCP-only; sealed hashes unchanged |
| **B2** (4 Plays / 2 openings) | 2 Studio reopens | none beyond reopen availability | quiet host for mandatory fixtures (handling, city:blocked-grade, city:junction, traffic:pass, traffic:abort-contact); probe/validator evidence |
| **B3** (12 Plays / 4 openings) | 4 Studio reopens | (i) **capture-resolution ruling** — memo `b3-capture-resolution-memo-20261101.md`: 1920×786 JPEG vs viewport 2769×1134, look_metrics rejects exact-viewport mismatch → look cells BLOCKED-pending-resolution; one owner-resized capture decides between normalise-in-judge and fit-window; (ii) **fresh independent critic seat** (Lead spawns) | look metrics vs locked anchor board; captures for human review only (no agent image reads) |
| **B4** (6 Plays / 2 openings) | 2 Studio reopens | none (unless a cell's evidence is a look image → same resolution ruling) | quiet host; performance/feel metrics via Luau probes |

## Holds in force

No Studio actions while the reopen is pending; MCP-only, text/JSON, no agent image reads, no host input, no
uploads/publication/credits. Sealed candidate artifacts unchanged (`84324d43…` release, `c6651793…` test/place).


## Play ceiling — it is OURS, not Studio/MCP (code evidence)

- `tools/vinegar_lib/checks.py:24`: `PLAYS_PER_OPEN = 3  # the 4th Play in one Studio process crashed it on 2026-10-16`.
- Enforced by `tools/vinegar_run.py play_budget()` and by `queue()`; the CLI `--plays-per-open` is clamped to
  `1..checks.PLAYS_PER_OPEN` (`:1241`, `:1251`). No Studio/MCP play-count limit is involved; the CAP is a
  crash-avoidance margin, not a platform rule.
- Counter: `~/.cache/usg/vinegar-plays.json`, keyed by Studio `(pid, start)`; a **new Studio process starts at 0** —
  i.e. only a full close+relaunch resets it (no in-place reset; unreadable counter fails closed).
- Raising the ceiling to 6 would be a code change (constant + CLI clamp) and would drop the 2026-10-16 crash margin;
  not recommended without fresh crash-trend evidence.

## Batched owner-opens plan (31 Plays remaining)

31 Plays at ≤3/open ⇒ **11 owner opens minimum** (10 full openings + 1 single), excluding failures.

| Block | Plays | Openings | Packing |
|---|---|---|---|
| B1 remainder | 9 (6 v1.queue + 3 baselines) | 3 | tac×3 · atlas:scout+atlas:highway-extension+moving · atlas:scout/atlas:edge/moving all-flags-off |
| B2 | 4 | 2 | first-fare, handling, city:blocked-grade · city:junction (1 play; fill with B3 plays 1–2 if B3 unblocked) |
| B3 | 12 | 4 | ladder order, 3 per opening (if B2 spare slots used, 12th lands in the next opening) |
| B4 | 6 | 2 | 3 + 3 |

Back-to-back while the owner stays at the machine. Natural batching: **block 1 = B1+B2** (13 Plays / 5 opens, all with
the owner present) and **block 2 = B3+B4** (18 Plays / 6 opens) once B3's capture-resolution ruling and the fresh critic
seat are in place. B3/B4 stay blocked until then; B1's next resume starts with the three `traffic:abort-contact` plays.
