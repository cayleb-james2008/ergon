# ERGON — reiteration remainders, verified at `b559e68` (Slice C)

**Date:** 2026-11-04 · **Branch:** `fix/board-remainders-20261103` · **Tip:** `b559e68` · **Tree:** clean, unchanged
**Nothing deployed.** The live site (`455a9da`/`4319779` lineage) was read with GETs as the control only.

Built from a clean export, not the working tree: `git archive b559e68` → `npm ci` (exit 0) → `VITE_BACKEND=demo RHL_BUILD_ID=b559e68 npm run build` (exit 0) → `vite preview`. Honesty measured on a `supabase` build against a faked origin (nothing left the machine).

## Verdicts

| Remainder | Verdict | The number that decides it |
|---|---|---|
| Mobile default sizing, 390×844 | **PASS** | `--fs 12px`, row **52px**, card **232×44px**, `pageOverflow 0px`, docWidth 390 = winWidth 390 — same with touch/`pointer: coarse` and at DPR 3 |
| Desktop unchanged, 1366×768 | **PASS** | branch `--fs 14px`, row 30px, card height 24px, overflow 0 — identical to live; card width differs only because demo's first row holds 2 jobs and live's holds 1 |
| Connection honesty | **PASS** | blank→Live@279 ms healthy, never "Reconnecting"; 2 s answers → Error duty **4 %**, **0 %** in the last 5 s, ends **Live**; hung path → Error@**2174 ms**, never Live after; first read hung → blank→Error@2601 ms, never Live, `data-state` `polling` while unread; hung board read with the token still answering → Error at read-start + **15.0 s** (19.0 s from the write), one read at a time |

Control, so the instrument is known to detect the defects: the same script against the live site measures `--fs 14px / row 56px / card 280×48` at 390×844. Note for the record: live showed **no** page overflow at 390×844 (`docWidth 390 = winWidth 390`), so the "page overflow" half of the old description is not reproducible; the oversize is.

## Suites (clean export, sequential, `timeout 600`, host load 5.7 → 10.2)

| suite | tests | pass | fail | skip | exit |
|---|---|---|---|---|---|
| `npm test` | 121 | 121 | 0 | 0 | 0 |
| connection e2e alone | 10 | 10 | 0 | 0 | 0 |
| `npm run test:e2e` | 108 | 103 | 0 | 5 | 0 |
| `npm run test:agent` | 34 | 22 | 0 | 12 | 0 |

17 skips are the live-gated tests (`E2E_SUPABASE=1`): realtime parity, RLS/key flows, the live walkthrough. **A gap, not green** — no real Supabase, RLS or websocket was exercised in this slice.

## Changed

Nothing. Both remainders were already correct at `b559e68`; no commit was made and the app tree is untouched.

## Gaps

Real device / iOS Safari sizing (Chromium emulation only, five phone sizes); the soft-keyboard path; a separate 3 s slow-link figure (covered inside the suite test); residual worst case of 19 s Live→Error after a write whose read hangs (15 s reconcile + 15 s deadline, by design); production-bundle secret scan not repeated here.

Full detail with commands and output: `~/fleet-scratch/overnight/SLICE-C-REITERATION-REMAINDERS.md`. Instruments: `~/fleet-scratch/overnight/sliceC-instrument/`.
