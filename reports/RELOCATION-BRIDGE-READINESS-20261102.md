# RELOCATION + DUET-BRIDGE READINESS SHEET — one page for Nox's ruling (2 Nov 17:00 ET, Lead)
Purpose: map the **Lane 6 review's conditions** (Team E, `team-e-lane6-duet-bridge-elevation-review.md`, CONDITIONAL/no FAIL) and the
**leg-3/leg-4 designs** onto concrete owners, current states and next actions, so one ruling can move the whole critical path. The owner's
declared end-state: **once Duet's connection is proven (a bus handoff wakes Duet, ack observed), the Claude desktop app shuts down and DSH is
the only harness** — so this is the critical path.

## A. Global precondition
| item | owner | state | next |
|---|---|---|---|
| **G40 W-vs-C must run first** | executing session (with Lane 6 observing) | not run | run it before any Part-B live step; its result gates the rest |

## B. Ranked must-fixes (Lane 6)
| # | must-fix | why (Lane 6) | owner | state | next action |
|---|---|---|---|---|---|
| 1 | **Single-writer guards before the app switch** | `writers.py` only sees hosts started `--profile fleet`; installers guard only the transient unit → a second writer on `fleet.db` is possible | fleet source (Claude lane) | not fixed | fix + mutation-test that a non-`--profile fleet` host is detected/refused; re-run gate |
| 2 | **Courier sandbox C0** | courier must run under the fleet sandbox; else route `manual` | Team B (courier) + executing session (sandbox) | courier built offline; C0 unverified | prove C0 on a scratch host; if it cannot hold, route `manual` per ruling |
| 3 | **L4-A ticket-detail prompt** | confirm must show the ticket's real detail, not `tb:<hash12>`; forge-flip only after A2 on the real machine | Claude lane (helper) + Team B spec | spec filed; helper not built | build helper with real-detail prompt; Lane 6 re-checks; A2 proof on real machine |
| 4 | **Archive key + decrypt-all** | off-machine key copy attested + `--decrypt-all` PASS before deletion | owner (key custody) + executing session (archive) | design only | owner-sitting item: key destination; archive script proves decrypt-all before delete |

## C. Path A — Duet bridge + connectivity proof (Part A)
| item | owner | state | next |
|---|---|---|---|
| Relay session naming (R3) | **Nox** | open ask | name the designated fleet Claude session (or confirm pull-only proof) |
| `duet` registration + token (Tier B) | owner (home sitting) | not done | after L4-A/elevation path? No — Tier B confirm is an owner act; schedule in the same sitting |
| Scope narrowing C1 (`report-pickup` recipient binding) + C2 server-side envelope validation | fleet source (Claude lane) | not built (Lane 6 conditions) | implement + tests; Lane 6 re-check |
| Courier (Drive mailbox, outbound-only, delivery/pickup/ack, `wake_claimed:false`) | Team B | offline prep + tests | hook to the real mailbox after owner Google consent/folder |
| Owner Google consent + mailbox folder | owner | not done | owner-sitting item |
| Proof: handoff through the bus wakes Duet, ack observed, four timestamps + measured wait | executing session + Duet | not started | after the above; proof is the end-state trigger |

## D. Path B — relocation of the delegator into the fleet profile (Part B)
| item | owner | state | next |
|---|---|---|---|
| Design ruling (Part B: inventory/migrate → prove → delete last) | **Nox** | with Nox | rule; then executing session builds the migration |
| Handover pack | executing session | not written | use `reports/DELEGATOR-CONTINUITY-NOTE-20261102.md` (bus identity, team roster, seat-only items, superseded hotfixes, **schedule automations that die with the profile**) |
| 43-fence sweep (desktop-profile "untouched" wording superseded by the owner's order) | Nox (supersede his own rulings) + mirrors | not done | supersede in writing; sync mirrors; gate script runs |
| App switch (symlink `profiles/desktop` → `fleet`) | owner + executing session | not started | after must-fix 1 + G40; single-writer proof first |
| Wake proof (Part B: handoff wakes the relocated delegator) | executing session | not started | after the switch; four timestamps |
| Archive → delete (age-encrypted, uploaded, read back, key-named files never touched) | owner | design only | after proof; must-fix 4 |

## E. Related gates that ride the same window
- **Face live install** (`0.3.5.1-face-prep` ready/scratch-proven): needs the installer self-check + must-keep hash gate + Nox's go — the
  executing session owns that gate; the face renders in the fleet profile (:3092), which the app switch makes the owner's app surface.
- **o1 Option B route fix** (rubric-final task can complete): next package by the executing session; not this window.
- **Abort-contact fallback** (Play 21: authority proven, geometry blocks closing): ruling pending — option 1 fixture geometry (recommended) or
  option 2 named limit.
- **Lumen final grade**: owner's open act; the presented copy (222364fd) stays in his browser, final set 2a8cc21b differs only by six
  note-fixes.

## F. What I can move without a ruling
Nothing live. My lanes are in their ruled states (B waiting on the install gate; A idle after the reel intake; C desk closed; D stopped and
awaiting the abort-contact ruling; E post-review). I will keep the board current and the continuity inputs ready; the moment a ruling lands
each lane has its next action named above.
