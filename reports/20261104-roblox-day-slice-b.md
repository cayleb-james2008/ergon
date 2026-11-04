# Slice B — Nox reply pointer + frozen values + record corrections

Date: 2026-11-04
Fences: board only. This slice does not re-file, re-capture, or merge.

## Pointer (already filed — do not re-file)

The batched Nox reply for Slice B is already on `origin/team` at:

`inbox/2026-11/20261104T105815Z-roblox-day-lead-to-nox.md`

Source on disk: `fleet-scratch/overnight/roblox-day/SLICE-B-NOX-REPLY.md`. That file is the reply: merges done, margin/void frozen values, one ask. Nothing else in this slice re-asserts it.

## Frozen values carried by that reply

Baseline: scout pack `20261103T125806Z-atlas-scout` (1837×744, 18 valid street views), re-measured with `tools/look_metrics.py`:

- `fleckEnergy.all` max **≤ 270** (baseline max 336.9, 20% margin)
- `cyanFraction.all` max **≤ 0.63** (baseline max 0.785, 20% margin)
- `voidFraction` max **≤ 0.05, scoped to `edge-*` views only**, provisional

Status: the committed freeze in `docs/art/thresholds.json` is **PROVISIONAL and must not be used as a gate** (mixed-viewport packs; the 8253 fleck bound is a 282×100 artifact). `roadLuminance.midOverNear` (min 0.6) and `facadeRepetition.worstWindowCount` (max 1) stay as declared goals, not no-worse bounds. Margin reasoning: 0% protects nothing (unchanged baseline passes); 20% asks the two known offenders to come down while other valid views pass. A flat void bound would fail the street baseline (only 1 of 18 street views ≤ 0.05), so scoping is a schema requirement, to land with the refreeze.

Held for Nox's yes (not merged): `prep/look-metrics-viewport-guard` @ `618316f` — one viewport per pack + `--min-viewport` off by default; suite 372 tests OK (main 369, +3). No threshold value touched.

## The two record corrections this reply forced

1. **50% fleck fails 2 of 18, not 5 of 18.** Measured: 336.9 → 168.4 fails 2 of 18 street views at baseline (`city-garage`, `city-road-n_6_5__n_6_6-forward`; 8 of 27 measured views including overheads). The earlier "5 of 18" count was wrong; the direction (50% is too strict and sets a bar before the road-material ceiling is known) holds.
2. **Determinism claim untestable from disk.** "Deterministic to 3–4 digits across three Plays" cannot be tested from the packs on disk: only 2 of 18 views share a byte-identical image across the two 296×398 runs (and those two measure identically); the packs hold different shots. It stays a report claim; no recommended bound depends on it. Same status section for Play numbers 33–36 (no artifact records a Play number; run dirs are timestamp-named only) and the missing 3 Nov freeze-run output (re-derivable, but the run's own output was never filed).
