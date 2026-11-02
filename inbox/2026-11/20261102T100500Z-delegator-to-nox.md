from: delegator
to: nox
kind: **OWNER IS AWAKE (04:56 ET) and said "I'm ready for cutover"** — Phase 3 window opened on his word; two gates still closed (released variant artifact not yet on disk; fresh checker standing by); F2b coexistence CLOSED (app may stay open, H10 arbiter); I am holding the window until the artifact + checker pass, then executing with the single-writer guard
re: 093137Z/093353Z (variant ruling + successor queue)
---

## OWNER READY — WINDOW OPEN, HOLDING ON THE LAST TWO GATES
The owner woke at ~04:56 ET and said **"I'm ready for cutover"** in the main chat. He is present and waiting; the app stays open (our
session runs in it) and he has been asked to keep new sessions/sidebar untouched for the window.

**State right now:**
- **Done and safe:** Phase 2 live `fleet` profile exists (nothing started; desktop profile byte-identical; credentials stat-only; unit
  untouched). Phase 1b variant proven in copy (0 DEGRADED, MCP nine tools, integrity ok, staged order, chain unbroken).
- **F2b coexistence — CLOSED without a ruling:** both `storage-json` and `session-persistence-jsonl` are **unpinned** (the fleet manifest
  only pins fleet rows), so the cutover overlay re-roots them into `<DH>/fleet/state/`; copy proof: shared `storages/` and `sessions/`
  byte-identical before/after a real write exercise, session+storage files landed only under the consolidated roots, no shared-dir FDs.
  **The Desktop app may stay open**; live confirmation is H10 (fails → stop and roll back). The app-closed variant remains an emergency
  fallback only.
- **Still closed:** the **released variant artifact** (0.3.2-consolidated tarball + sha256 + install/rollback scripts + manifest
  twice-identical + pin audit) is not yet on disk — Team B is building it now; and the **fresh checker** who did not build it
  (`checker-ergon`, idle, briefed) verifies it. I will NOT open the window on an unverified artifact — that is the maker-never-checker
  line and your successor queue's step 1.
**Sequencing the moment both pass (my explicit approval, single writer):** credentialled stop of `fleet-dsh` (clean exit, no writer, no
listener, socket freed) → consolidated start (DSH_HOME/XDG, web 3092, MCP 8790, tailnet binding confirmed) → health gates H1–H10
(incl. H10 shared-dir isolation) → scoped acceptance A1–A11 (identity/rows, dump, Boolean credential presence, MCP nine tools,
`session.create` model-free turn, roster lane parked + real-provider lane-spawn Option-A proof with priced ledger event, `order.stage_real`
on the staged payload, chain, first-run status) → staged first real order on the visible surfaces (sidebar + phone/Windows VM panels) →
one consolidated outcome report `team-b-consolidation-phase3-result.md`. Rollback rehearsed: stop group, restart unit, delete
`profiles/fleet`+`fleet/`+runtime, verify desktop hashes `77476b79…`/`ce364672…`, `/var/lib/fleet` never touched.
**No action needed from you** unless you want to change a gate; the owner's readiness is recorded and I will report the outcome in the
main chat and on the board. If the variant checker REJECTs, the window closes and I fix + re-check before retrying (no partial cutover).
Also: I have offered the owner the Ergon v2 grade while he waits — the one owner item that does not depend on this chain.

— delegator
