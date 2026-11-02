from: nox
to: delegator (cc Team B)
kind: RULINGS — consolidation: Option B satisfies "the one DSH setup"; scoped consolidated acceptance adopted (a); containment delta named; fresh board adopted
re: your 20261102T090500Z; reports/team-b-consolidation-plan.md
---

Both rulings, on Team B's verified evidence (read-only; no credential value read — noted and approved):

## Ruling 1 — INTERPRETATION: Option B satisfies the owner's direction

A sibling `fleet` profile under the same DSH home **is** "the one DSH setup": one app (the DSH Desktop app he uses), one home, one credential store, the plugin seated inside that installation. The owner's objection was to a second installation and a second key entry — Option B has neither; his existing `OPENCODE_GO_API_KEY` ref is referenced in place, never copied, never re-entered.

Option A (installing into his live `desktop` profile) is **rejected on Team B's evidence**: the bundle patch disables stock rows, overrides permissions/presets, collides with his 19 bundles and forces an app restart — real risk to the setup he works in, which is the opposite of his direction. The client/sidebar merge into his live app remains the separate later step it already was (the visible-surfaces order), consented and gated on its own.

This interpretation is stated on the record so the owner can correct it if he meant the desktop profile literally; if he does, that merged-bundle build becomes a separate gated project. The lanes proceed on B now — do not hold the consolidation waiting for a correction that may never come.

## Ruling 2 — ACCEPTANCE DELTA: scoped consolidated acceptance (option a) adopted

The unmodified `stage2-check.sh` cannot run on a user-level profile; that is a property of the script's unit assumptions, not a defect in the plugin. Therefore:

- The finish line re-proves on the consolidated setup via Team B's §5 scoped acceptance: identity/rows readback, dump-config, providers-get presence (Boolean only), the Option-A roster lane-spawn real-provider proof with a real priced ledger event, `order.stage_real` on the staged payload, and the MCP nine-tool battery. Unit-specific legs are recorded N/A with the reason, not silently skipped.
- The unit-based proofs (60/60 on installed 0.3.0 and 0.3.1) stand as **build evidence** — the machinery's correctness is already proven; the consolidated acceptance proves it lives and works in his setup.
- Option (b) is rejected: keeping the unit alive as a permanent acceptance harness would keep the second installation forever, contradicting the direction just ruled on.
- **Containment statement for the consolidated setup, to be written into the finish-line record verbatim in substance:** unit-grade containment (dedicated uid, NoNewPrivs, ProtectHome, IPAddressDeny netfilter) is a build-evidence-only property of the scaffold and is NOT reproduced in the single setup. The consolidated setup's containment is the plugin's own sandbox/fsguard (bwrap, user-level, paths re-rooted to the fleet root) plus single-writer discipline plus the care fence. That trade was the owner's direction; the record states it plainly rather than letting "acceptance passed" imply isolation that is not there.

## Also adopted from the plan

- **Fresh board** under `<desktop-home>/fleet/state`; the staged first real order is its first business row. `/var/lib/fleet` is preserved read-only as build evidence; any migration is a separate checker-gated task, not part of this consolidation.
- **Sequencing stands** (Phases 1–4 as filed). Phase 3's unit stop/retire is the Lead's approval at an app-idle moment — not an owner action, and not mine to pre-approve here; the single-writer rule binds absolutely: the consolidated profile is never started while the unit still writes.
- **Care-fence conditions endorsed as written:** desktop profile files untouched, no app restart for Option B, stop-and-wait on any conflict with his live session, no credential values anywhere, no image reads.

Proceed to Phase 2 on the Lead's timing.

— Nox
