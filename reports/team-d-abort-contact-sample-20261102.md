# TEAM D — ABORT-CONTACT B-BOUNDED 3-SAMPLE EVIDENCE (for the Nox ruling), 2026-11-02

Scope: three `traffic:abort-contact` Plays on the frozen B-bounded fixture (`e6ae134`, unchanged; current tip `746a99e`
differs only in atlas files). No code/fixture changes between runs; one Play per run; `verify-clean` between. Evidence
only. Log: `board/probes/team-d/abort-contact-sample-20261102.log`.

## 1 · Per-run table (all three are the same, stable phase)

| # | Play / run dir | cutInAt | abortObservedAt | gap @ abort | min gap (pre-leg) | time-to-min | max gap | gap @ named fail | timeout hit | contacts/touches | crash | plays |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Play 12 `20261102T065310Z` | 2.7622 | 3.7621 | **73.55** | 73.55 | 0.0 s (at abort) | 123.97 | **123.9** @ t=7.778 | **yes (4.0 s)** | 0 / 0 | none | 1/3 (pid 923305) |
| 2 | Play 13 `20261102T070954Z` | 2.7831 | 3.7824 | **73.05** | 73.59 | 0.1 s | 122.93 | **122.9** @ t=7.783 | **yes (4.0 s)** | 0 / 0 | **yes — userland minidump 101,859,264 B @ 07:10:10Z (Wine shutdown); abort stop/cleanup/verifyClean Disconnected(60 s); exit 5** | 2/3 (opening died) |
| 3 | Play 14 `20261102T071208Z` (recovered fresh opening) | 2.7788 | 3.7780 | **73.48** | 74.02 | 0.1 s | 122.49 | **122.4** @ t=7.794 | **yes (4.0 s)** | 0 / 0 | none | 1/3 (pid 933721) |

Proxy speed phase (samples, all runs): ~40 at t≈0.1 → dip **8–12** around t≈1.5–1.8 and again **~8** at t≈4.6 → rising
**17 → 29** by t≈7.7. Sealed artifacts (`84324d43…`/`c6651793…`) unchanged before/after every run; the crashed opening was
recovered with `studiocycle open` (sealed place hash before==after), its latch cleared (`clean:true, latchCleared:true`),
and Play 14 ran in that recovered fresh opening (1/3 used; two Plays reserved).

## 2 · What varies / what does not

- **Does not vary (3/3):** the phase timing (`cutInAt 2.76–2.78`, `abortObservedAt 3.76–3.78`), the gap at abort
  (**73.05–73.55**), the monotone growth to **122.4–124.0** over the 4 s pre-leg, the minimum gap being **at the abort
  instant** (time-to-min 0–0.1 s), and the named bounded pre-leg fail at the 4 s bound with no trigger/contact.
- **Varies:** only the crash outcome (1 of 3 runs crashed on shutdown after the failure was posted) and the exact gap
  numbers by ~1 stud.

## 3 · Assessment (evidence only; no implementation)

- **Does B-bounded converge?** **No — 0/3.** The gap never approaches 25; it is ~73 at the trigger window's start and
  grows monotonically. The `contactLegTrigger` predicate is unreachable in this phase.
- **Would a longer bounded pre-leg timeout help?** **No.** All three samples show the gap at its minimum at the abort
  instant and increasing afterwards (to ~123 by 4 s); extending the wait would only extend a diverging phase and weaken
  the fail-fast property.
- **Would option A (window 12 → 20 s) help?** **Not by itself** — the 12 s window only starts once the leg starts; with
  no trigger the leg never starts, so a window extension cannot change this outcome.
- **Would option C (pursuit clamp) help?** **No** for the same reason (no leg start); C remains relevant only if a leg
  starts and still misses.
- **What would plausibly change the outcome (design only, Nox ruling needed):** the leg-start geometry is phase-fixed
  ~73 studs, so either (a) **raise the pre-leg rolling speed** above 30 so the probe is closer when the abort lands
  (safety constraint: stay >17 studs so there is no pre-abort touch), or (b) **change the leg-start trigger semantics**
  (e.g. trigger on the closest pre-leg point with a bounded gap window like 40–90, or a time-based trigger), which
  changes the ruled 12–25 start geometry and needs Nox, or (c) **reduce the initial ego offset further** together with a
  tuned pre-leg speed (phase-specific; changes the scenario). All are fixture/phase changes, not law/guard changes.
- **Crash class (residual, honest):** 2nd intermittent userland Wine Studio crash on Play shutdown (Play 6: 106.1 MB;
  Play 13: 101.9 MB), both after the runtime failure was posted; sealed unchanged; no kernel GPU evidence. Recovery via
  `studiocycle` works; the latch was cleared on the recovered sealed place.

## 4 · Recommendation to the Lead/Nox

With 0/3 convergence, **B-bounded as ruled cannot start the leg in this phase**, so the fair-sample evidence points to a
**fixture-phase change** (a/b/c above) rather than A or a longer pre-leg bound. The three sample runs, the monotone gap
series, and the crash class are filed here for the ruling. Nothing implemented beyond the frozen fixture; the recovered
opening (1/3 used, two Plays reserved) is idle.
