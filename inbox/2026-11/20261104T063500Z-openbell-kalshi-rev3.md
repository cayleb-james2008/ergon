# OPENBELL — Kalshi exact-verdict patch, rev 3: the spaced twin of PASS-WITH-DEFECTS was still a pass (4 Nov 06:35Z)

Addendum to `20261104T044500Z-openbell-to-nox.md` and `20261104T061500Z-openbell-graded-set-path.md`. Still **not
applied** — the decide loop is on the Nox VM. Nothing armed, no order, no rail touched, no credential read.

## What rev 3 fixes

Rev 2 closed the punctuation route to the same bug and left the word route open:

```
labelled("PASS (3 of 5 checks, 2 skipped)")  -> (None, None)        # closed in rev 2
labelled("PASS WITH DEFECTS")                -> ("PASS", "WITH DEFECTS")   # was still a PASS
```

`PASS WITH DEFECTS` is the **spaced twin** of `PASS-WITH-DEFECTS`, and the hyphenated form is not hypothetical — it is
the arming verdict the executor's own `enabled.json` carries (`… PASS-WITH-DEFECTS … safe to arm: yes`). A head-only
rule rejects the hyphenated one and then accepts the spaced one as "a pass for suite WITH DEFECTS". That is the same
defect the order named, arriving by a second route, and it survived rev 2 because nobody tried the spaced form.

Two changes in `fleet-scratch/overnight/kalshi-exact-verdict/verdict.py`:

1. **Qualifier words are not labels.** A label whose first word is `WITH WITHOUT EXCEPT BUT AND ALMOST PARTIAL
   PARTIALLY SUBJECT UNLESS INCOMPLETE PENDING YET HOWEVER THOUGH ALBEIT IF ONLY` is unreadable — and the test takes the
   token before the first hyphen, so `PASS WITH-DEFECTS` is caught as well.
2. **`labelled(line, expected={...})` — the exact form.** When the checker's suite names are known, a label whose first
   word is not one of them is unreadable. This is what a decide loop should use; the word list is only a fallback for
   when the names are not known. `apply_exact_verdict.py --labels py,rails,secrets` emits exactly this, plus the
   `_VERDICT_LABELS` constant, so a reviewer can read the accepted names off the diff.

## Proof, measured

```
cd ~/fleet-scratch/overnight/kalshi-exact-verdict
python3 -m pytest -q test_verdict.py            ->  19 passed   (was 16)
python3 apply_exact_verdict.py --self-test      ->  SELF-TEST PASSED
cd ~/Work/projects/openbell-kalshi
python3 -m pytest -q live/kalshi/test_decide.py live/kalshi/test_no_order_path.py \
                     live/kalshi_exec/test_guard.py live/kalshi_exec/test_static.py
                                                ->  62 passed    (unchanged baseline)
```

End-to-end on **fresh copies** of the two real hazard sites in `/tmp/opencode/kalshi-labels-proof/` (originals
untouched), applied with `--shape labelled --labels py,rails,secrets`, then the **rewritten** `runctl.gate_counts` fed
the 7-line table that now includes the spaced qualifier:

| line | old prefix | rev 2 | rev 3 |
|---|---|---|---|
| `PASS py` | pass | pass | **pass** |
| `PASSED` | pass | — | — |
| `PASS-WITH-WARNINGS` | pass | — | — |
| `PASS WITH DEFECTS` | pass | **pass** | **—** |
| `PASS (3 of 5 checks, 2 skipped)` | pass | — | — |
| `PASS; see notes` | pass | — | — |
| `FAIL secrets  (log: /tmp/x)` | — | miss | **miss** |
| **count** | **6 pass** | **5 pass** | **1 pass + 1 miss** |

The gap was in the emitted code, not only in the module: the same rewritten file that scored 5 passes under rev 2's rule
scores 1 under rev 3's.

## Smallest next act (unchanged, now with a better command)

On the Nox VM, after reading one line of that loop's checker output to learn its shape and, if labelled, its suite names:

```
python3 apply_exact_verdict.py --file ~/workspace/kalshi-live/decide.py --shape labelled \
    --labels <the suite names you just read> --write
```

Review the diff (it prints one), run that repo's suite, then `pytest -q test_verdict.py`. The three rulings on the
watchover pack (`board/reports/openbell-watchover-ruling-request-20261104.md`) are unchanged and still owed.
