# Slice A — per-merge proof (Roblox day lead, fallback engine)

Date: 2026-11-04
Source: `fleet-scratch/overnight/roblox-day/SLICE-A-PROOF.md` (condensed; full logs `/tmp/opencode/slice-a/`)
Fences: board only. No product-repo edits, no merges, no push by this slice. No credentials. Suites run in copies under `/tmp/opencode/slice-a/`; real checkouts untouched.

## Verdict

Merged trunk matches Nox's ruling in topology and order. Street trunk was green *at each merge commit*. Card trunk was green *at each merge commit* on every game suite; the full `verify.sh` pipeline was green *only with later fixes* — exactly as BRIEF-A predicted. Sealed street places unchanged end to end.

## Merges

| # | Merge | Parents (exact, in order) | Content |
|---|---|---|---|
| 1 | street `536e3c8` ← `prep/moving-association-20261103` @ `18d01a2` | `5ee9f63 18d01a2` — both ancestors ✓ | `git diff --stat 18d01a2 536e3c8` empty (verbatim) ✓ |
| 2 | street `300b632` ← `prep/atlas-edge-diag-20261103` @ `bfc28ab` | `536e3c8 bfc28ab` — both ancestors ✓ | branch tip diff empty; `-- src` diff empty (no production source) ✓ |
| 3 | card `c15d534` ← `prep/card-game-design-spec` @ `e4e4746` | `04031c0 e4e4746` — both ancestors ✓ | 840 files, +39800/−628 (design-spec merge) ✓ |
| 4 | card `9c3606b` ← `prep/balance-pass-20261103` @ `956b4bc` | `c15d534 956b4bc` — both ancestors ✓ | 8 files, +201/−13 (balance pass) ✓ |

Order: `536e3c8 → 300b632` and `c15d534 → 9c3606b`. Topology **PASS** on all four.

## Suites at the merge commits (not HEAD)

**Street `536e3c8`** (merge 1): Python 352 passed / 1 skipped / 454 subtests (exit 0, 69.63 s) · Lune unit 943 passed / 0 failed / 130 specs (exit 0) · Edge 10 passed / 0 failed (exit 0). **PASS — green at the commit.**

**Street `300b632`** (merge 2): Python 368 passed / 1 skipped / 463 subtests (exit 0) · Lune unit 943/0 · Edge 10/0. Delta +16 Python tests consistent with atlas-edge additions. **PASS — green at the commit.**

**Card `c15d534` (merge 3) and `9c3606b` (merge 4)** — same command `TOOLKIT_ROOT=<empty dir> bash tools/verify.sh --skip-toolkit`:
- All game suites exit=0 identically at both: bootstrap, p2/p3/p4/p6 lua+lune, server-conc lua+lune, offers lua+lune, journey lua+lune, studio lua+lune; `preflight-report` exit=0; `audio-pytest` exit=0; `toolkit-53` skipped.
- `preflight-selftest` exit=1 — `SELF-TEST FAIL: live build-candidate want PASS got FAIL` (pre-`5269967` ordering defect; `5269967` not an ancestor of either merge).
- `build-1` exit=1 — rojo `No such file or directory` on `build/UntitledAnimeCardGame.rbxlx` (pre-`17a9079` defect; `build/` dir missing).

Per-merge verdict: **FAIL on the two build-gate steps; PASS on every game suite.** Fixes `17a9079` and `5269967` landed later and are on the current trunk. Nothing was fixed forward in this verification.

## Sealed hashes

Re-verified via `sha256sum` on the real checkout (`untitled-street-game/build/`), before and after:
- `build/release.rbxlx` `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce` — unchanged
- `build/test.rbxlx` `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` — unchanged

**PASS.** Sealed place never rebuilt; `tools/usg test` never run.

## Not reproducible at the merges (and why)

1. **Card 22/0 + twice-identical place `99ded6a6…` + 135-input fingerprint**: not at `c15d534` or `9c3606b`. Both predate the build-gate fixes, so `build-1`/`preflight-selftest` fail there for the predicted reasons; no place file was produced, so no hash comparison was possible. Measured fingerprints (133 at `c15d534`, 134 at `9c3606b`) differ from the 135-input expectation, reflecting later file additions. The full-pipeline green claim is therefore bound to the later-fix commits, not to the merge commits.
2. **Street "Ran 369 tests"**: measured 352 at merge 1 and 368 at merge 2. The merge trees predate the docs/test additions that later reached HEAD (`a43b7e4`, well past `300b632`); 369 is not claimable for the merge commits.

What is still owed: nothing for the merges themselves; the board should carry the "green only with later fixes" distinction for the card pipeline.
