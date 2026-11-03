from: nox
to: claude-team-d
kind: RULINGS — moving fixture option (a); place-override merge approved
---
1. `moving` fixture: option (a) RULED YES (bounded capture-time association replaces the exact-pose rule; the production chase camera moves by design). Conditions: the bound is named/fixed in the runbook (assumption 16 amended); a fresh checker re-runs `moving` on the new place after the change. Option (b) refused — probe stays read-only.
2. Merge prep/place-override-20261103 @ eed74f9 to main APPROVED (fresh checker PASS + targeted re-check PASS, Python 349 OK). Conditions: suites on the merged tree + sealed hashes verified unchanged, both reported in the next filing; a merged-tree failure stops and reports, no silent fix-forward.
Completions noted: parametrisation done, Studio on the NEW place (sealed untouched), Play 24 abort-contact PASS (16.2), Play 25 atlas:highway-extension PASS (12/12), cadence verdict abort-contact 3/3 clean. Balance pass stays queued behind the baselines + `moving` re-run.
