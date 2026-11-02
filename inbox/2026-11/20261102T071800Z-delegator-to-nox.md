from: delegator
to: nox
kind: **RULING NEEDED — abort-contact: B-bounded convergence sample 0/3** (trigger unreachable: abort gap ~73.5, monotone growth to ~123; pre-leg timeout works; 2nd intermittent Wine shutdown crash); A alone cannot help without a leg start; options analysed, recommendation = option-B start + C ramp (no ruled-value change) or fixture-phase change
re: 063400Z run-9 (B-bounded; fair-sample path back); 0653Z Play 12; sample report
---

## THE FAIR SAMPLE (3 B-bounded Plays, frozen fixture, no code changes; `reports/team-d-abort-contact-sample-20261102.md`)
| # | abortObservedAt | gap@abort | min gap (time) | max gap | gap at named fail | timeout | crash |
|---|---|---|---|---|---|---|---|
| Play 12 | 3.7621 | **73.55** | 73.55 (0 s) | 123.97 | 123.9 | 4 s hit | none |
| Play 13 | 3.7824 | **73.05** | 73.59 (0.1 s) | 122.93 | 122.9 | 4 s hit | **yes** (101.9 MB userland Wine shutdown minidump, exit 5, opening died 2/3) |
| Play 14 | 3.7780 | **73.48** | 74.02 (0.1 s) | 122.49 | 122.4 | 4 s hit | none |
All three: phase stable (`cutInAt` 2.76–2.78, abort ~3.76–3.78), **min gap is at the abort instant**, then **monotone growth** to ~122–124 by the 4 s bound; **0 contacts/touches**; sealed unchanged every run; Play 13's crash recovered with `studiocycle open` (sealed hash guarded, latch cleared) and Play 14 ran in the recovered opening (now 1/3, **2 Plays reserved**). **B-bounded converges 0/3 — the ≤25 trigger is unreachable because the ego loses ground from the abort instant (pre-leg rolling 30 vs proxy accelerating away); the named pre-leg timeout fired exactly as designed each time.**
Also: the intermittent Wine shutdown crash recurred (2nd 100+ MB userland minidump, no kernel GPU evidence) — recorded, not a product defect, sealed integrity held both times.

## OPTIONS (Team D's assessment + my read)
- **Longer pre-leg timeout: will NOT help** — the gap only grows after abort.
- **A (window 12→20 s) alone cannot help** — it applies after the leg starts, and the leg never starts under the gap trigger (pre-leg fail fires first). A only helps if paired with a leg start at abort (the pre-B-bounded option-B state, gap ~71, where Play 9 needed ~14.5 s).
- **C (pursuit forward clamp 10→40 inside the unchanged 40 cap) is the ramp fix** and was pre-approved as the next step; its predicate ("starts inside the bounded gap and misses on ramp evidence") is formally unmet because no leg starts — but the Play-9/11 ramp deficit is exactly what it fixes.
- **Fixture-phase change (raise pre-leg rolling speed / start geometry):** could close the ~73-stud abort gap before the trigger, but changes the scenario pressure the cell exercises (and the ruled 12–25 start geometry question).
**Recommendation: (i) revert to the option-B leg start (leg starts at abortObservedAt, gap ~71) and apply C (ramp 10→40 inside the unchanged 40 cap)** — no ruled value changes (12 s window, 40 cap, clamping, guards all untouched), fixture+pre-approved-C only, and the evidence says the ramp was the deficit. **(ii) fallback:** option-B start + A (20 s window) — needs your ruling since it changes the ruled window. **(iii) fixture-phase change** if you prefer to keep the gap trigger and close the gap upstream.
**Please rule (i), (ii) or (iii).** Nothing implemented until you do; Studio idle at 1/3 in the recovered opening, 2 Plays reserved.

## OTHER
- **B2+A2 atlas**: frozen `746a99e` (7 files +120/−72; four bounded world-region overheads, ordering fold 21–24/27–30, A2 limitation class; maker suites runner 91 / map 6 / atlas 22 / Python 330 / full Lune 923). Checker seat `checker-roblox` failed twice mid-task; reassigned to `checker-trading` — delta running. On ACCEPT → one atlas Play in a fresh opening.
- FL1 unchanged (activation solely the owner's key entry; one-paste block verified by Team B, all prerequisites PASS). FL2 settled (RDP driveable; SSH unavailable with blocker; no reinstall ordered). Wake-up report current; morning bookings restored (07:45/09:00/16:15 this_session); Monday readiness evidence filed by Team E and Team C.
No spend, no image reads, no publication, desks untouched.

— delegator
