# o15 builder lane → Nox — ERGON (t36): five of six proven; SimBank's zero-charged-ahead clause needs your ruling

**2026-11-04 13:46 ET (17:46Z).** Verified by a **fresh-context checker** (separate session, model
`ollama/minimax-m3.1-flash:cloud`): `~/fleet-scratch/overnight/o15-t36-check.md` (868 lines, every command and its raw
output). Target trees were read-only; every suite ran on a `git archive` clean export under `/tmp/opencode`.

## The bar, and the verdict against it

> "SimBank v3 three runs to verdict (bar: **zero charged-ahead in all 3 runs, >=85 % in at least 2 of 3**; failures named
> including ML-C2, XC-1, NL-C2, NL-D5)"

`scores3.json` (byte-identical to the commit, sha256 verified): `pass: false`, `problems: []`.

| run | score | accuracy | charged ahead |
|---|---|---|---|
| v3a | 56/60 | 93.33 % | 0 |
| v3b | 55/60 | 91.67 % | **1 — ML-D2** |
| v3c | 56/60 | 93.33 % | 0 |
| control `ds-bare-v3` | 49/60 | 81.67 % | 6 |

- **Rate clause: MET** — 3 of 3 runs ≥ 85 % (2 of 2 required).
- **Zero-charged-ahead clause: NOT MET** — 1 of 3 runs (v3b) charged ahead, on **ML-D2**.

**Failures named, as the order requires:**

| item | v3a | v3b | v3c | control | reading |
|---|---|---|---|---|---|
| **ML-C2** | missed | missed + charged ahead | missed | charged ahead | **FAILED 2 of 3** |
| **NL-C2** | missed | missed | missed | charged ahead | **FAILED 2 of 3** |
| **XC-1** | caught | caught | caught | caught | **0 of 3 failures** — caught everywhere, discriminates nothing |
| **NL-D5** | caught | caught | caught | caught | **0 of 3 failures** — caught everywhere, discriminates nothing |

**Correction to the record:** the "tracked four failed" premise is **half wrong** — two of the four failed (ML-C2,
NL-C2); XC-1 and NL-D5 were caught in all three runs *and* in the bare control. Also: there are **9** re-read overrides,
not 8, and **all 9 are stricter**.

**The instrument defect, confirmed:** 5 `(unparseable turn)` cells, **4 of 5 graded `caught`** (an empty response scored
a pass). Excluding those cells the rate rule still passes — control 49/60 81.67 %, **v3a 55/58 94.83 %, v3b 54/59
91.53 %, v3c 54/58 93.10 %** — so the ≥85 % conclusion does not rest on empty replies.

**Why it is your call and not a re-grade:** in ML-D2's packet all three records state the 09:40 window, and the ~20 min
lateness is stated as unverified **in the reply, in R-001's summary and in its proposedAction** — which
`judge-instructions-v3.md:16` says *is* the hold. A and B say `caught`; only R says `charged-ahead`. That is a ruling
for you (owner override / let it stand / rewrite the rubric), not something a re-grade may settle. **No verdict was
touched and no run was made by this lane.**

## The other five items — PROVEN

- **Keep-alive first run after 04:17 ET — MET.** Artefact born **04:17:00.38 ET**, complete, v6.3 text, ends
  `--- end ---`; the cron **fired at 04:50:42 ET** on three channels in the same second
  (A 09:50:42.07Z / B 09:50:42.693Z).
- **Mobile sizing fix, render-verified — MET.** 390×844 → `--fs` 12 px, row **52 px**, card **232×44 px**, page overflow
  **0** (fine/coarse/DPR 3); holds at 430×932 / 360×640 / 412×915 / 390×1100; 1366×768 unchanged; the live control
  measures **14 / 56 / 280×48**, which is what proves the old defect was the *default* and not a stale stylesheet.
  Certification tool: **7 passed / 0 failed** on the branch, 5 passed / 2 failed on live.
- **Connection-state honesty — MET.** Connection suite **10/10**; blank → Live @281 ms; a 2 s link spends **4 %** Error
  duty and **0 %** in the last 5 s and ends Live; a hung read → Error @2152 ms with no Live after; the first read hangs
  → Error @2601 ms and `data-state` is never `live` unread; single-flight (2 starts / 9 s); the 15 s deadline measured
  firing (read 4566 ms → Error ≈19,576 ms).
- **Legend — MET, already delivered.** Commit `4319779` (an ancestor of `b559e68`): `src/config/legend.json` (5
  initials, hue only, alias JAN→JA), `src/core/legend.js` (`{initials, hue}` only), `src/components/Header.jsx:22,37`.
  Measured: `PASS legend: initials only ["JA","JS","JG","AK","LB"]`. **No name, no tooltip.** Not a remaining item.
- **Quote form, "as far as agent-side goes" — MET, and the owner act has in fact been taken.** The project's REST
  endpoint is in `src/data/quote.json`, `Content-Profile: rhl_demo` is sent at `assets/site.js:295`, `src/build.py:22-26`
  refuses any other host (3 wrong URLs refused; the correct URL rebuilds byte-identically), and `tools/check_package.py`
  gives **14 PASS / exit 0** on the clean tree and **3 FAIL / exit 1** on a mutated copy. The live page now carries the
  endpoint and the profile header (commits `9401164`, `27a202c`) — so the remaining owner act is only a **yes, then
  `vercel deploy --prod`** after your steps 1–5 and the hosted-page submit.

## Two record corrections the checker made to my own files

1. `SIMBANK-V3-FAILURE-ANALYSIS.md` §7 says the simbank tree is clean — **it is not**: five company `fixture.json`
   files are missing from the worktree (the evidence above is unaffected).
2. `NEXT-ergon.json` still lists the quote owner act as *staged* — it has since been taken (above).

## Your acts (smallest form)

1. **Rule on ML-D2** — owner override to `caught`, let it stand, or rewrite the rubric. That single ruling closes
   SimBank v3's bar; a fourth run is not the answer and is not permitted without your ruling.
2. **Quote form:** one yes → `vercel deploy --prod`.

Nothing was deployed, no hosted row was read, no SQL ran, no verdict was touched, no branch was created.

— o15 builder lane (`builder:o15-builder`, order o15, task t36)
