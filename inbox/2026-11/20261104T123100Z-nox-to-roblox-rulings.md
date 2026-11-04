# Nox rulings — Roblox lane (from the publish-readiness audit, 4 Nov 2026)

For the Roblox day lead. The audit (~/workspace/goals/roblox-street-and-card-games/hidden_files/publish-readiness-audit-20261104.md on Nox's VM) resolved the record: card merges are DONE (master head a22c17e, checker-corroborated); the five parked rulings are answered here.

RULINGS (Nox, 4 Nov):
1. Viewport guard: YES. Adopt min size 1600x700 as proposed.
2. Void bound: YES — 0.05 scoped to edge-* views, marked PROVISIONAL.
3. Margin freeze: YES — freeze fleck <=270 / cyan <=0.63 (the corrected 2-of-18 offender data governs; the old "5 of 18 at 50%" figure is retired).
4. Refreeze on branch: YES — the frozen thresholds file is defective (mixed-size packs; fleck bound 8253 is a 282x100 artifact) and is PROVISIONAL ONLY, never a gate, until the refreeze lands.
5. Card trunk pick: A (whole stack, a22c17e) CONFIRMED — it was the only executable reading and it is already merged and checked. The docs-only branch (7b50877) stays parked as the fallback reading; do not switch.

Work order on these rulings: merge-and-switch-on the look-gate guard branch (prep/look-metrics-viewport-guard @ 618316f, 372 tests), run the refreeze, fix the studiocycle Wayland window-finder in one focused session with Studio closed. The street art-polish items (road luminance 7/18 under 0.6; facade repetition worst 13 vs goal <=1; the two offender views) and Slice D (still untouched) sit with the publish-ready mission already dispatched (day-roblox-2).

Owner gates unchanged: street playtest verdict, card D12 (Delta-2 + Delta-16 open), publication per game, plus a NEW owner item — an owner-present re-capture session (~3 Plays) for the refreeze captures, which the lane cannot do unattended.
