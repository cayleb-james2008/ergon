from: claude-ergon
to: nox
kind: FOURTH INSTRUMENT CHECK FILED: NOT READY (1 high, rest medium/low); all fixed and committed; a fifth fresh check is running. Still no v3 run; gate still closed.
re: your standing request to report each checker's verdict
---
Fourth fresh Opus checker (simbank-v3 `332d34c1`, tiered-plans `c0683a56`). It confirmed the third round's items mostly closed (CH-D4 code out of view, wrong-action pointer, vacuous tests, NL supplier addresses), ran the full suites green, and found nothing to reject outright in the 38 fixtures. It then attacked aggregate3 and the plan gate together with 16 scenarios of sloppy-operator and tamper cases. Its remaining defects, all accepted:
1. HIGH: a missing final re-read (R) file was not a problem, so the re-read could be dropped wholly or per packet and the rule still passed. Now every packet must have one.
2. MED-HIGH: the model was only the proof record's word; a missing `model` passed. Now aggregate3 records the model it read from the response files and the gate compares it.
3. MED: nothing stopped three "runs" being copies or links of one run, stub-named or junk files, or a wrong id/tag inside a file. Now: exactly `<scenario-id>.json` per scenario, id/tag/model/session checks, session ids unique across all responses, links refused.
4. MED: a packet was not bound to its response (swapping the run label of two packets passed). Now each map entry carries the SHA-256 of its response file and aggregate3 checks it; `judge3-packets` writes it.
5. MED: option paths and verdict/response hashes were not recorded. Now recorded in scores3.json.
6. MED: judge line 14 contradicted lines 8-9 on a review request that states the doubt (now caught everywhere). Clean-rubric steps the request never asked for (six rubrics) were removed and the judge file now says the request decides what is required. Rubric camelCase keys vs hyphenated labels are spelled out.
7. LOW: a stale scores file survived a crash (fixed); the act path wrote no `mode` so the full/control role was unreadable (full runs now write `mode: full`); README/RUNBOOK overstatements corrected.
Tests: 32 aggregate3 tests (24 before) and the gate suites 74/74; the checker's three surviving mutants plus mine are killed (each link/regular-file/contiguity-count/control-tag/model check now fails its own test).
Accepted trust line, now stated in README and RUNBOOK: `scores3.json` is self-attesting. The gate pins only its hash and never re-runs aggregate3, so anyone with repository write access can forge scores and record; the commit review and your order are the actual control.
Commits: simbank-v3 `3c64c21a`, tiered-plans `a8abf320`. Nothing pushed or merged. No DeepSeek call, no v3 run (small_model console check still the gate). I will mail the fifth verdict when it files.
