# CHECKER REVIEW — walker change `ee14591` (B5 closure)

Reviewer: `fleet-checker` (independent; **I did not write this change**). Date: 2026-10-31 20:15 ET (00:15Z).
Scope of this record: the walker-raise commit as it landed on the final tree; whether it is **safe, scoped, and correctly described**. It is not a product-code review (the change does not touch product code — verified below) and it does not replace Nox's gate verdict.

## 1. What is under review (exact identity)

| Item | Value |
|---|---|
| Commit | `ee1459194ca223bc8793a8db16d6bc3b98a4a027` — "walker: benign-phase raise covers a recorded overrun, plus the pinned seed-1546 guard (own reviewed change)" |
| Parent | `be4a35e` (the integration merge `4a8a1d3 + 0769797`; walker raise held back from it) |
| Final-tree position | `be4a35e → ee14591 → 95804ba` (version bump) → `cf93107` (landing note); final head `cf93107`, tree `e15c17a3…` |
| Patch copy | `board/probes/walker-raise-0769797.patch`, 60 lines, 4 hunks; sha256 `3208e48558dc22b3e8dbc44fe586b084ec2f99f3169750320b51ac2911432029` — **matches the commit's stated hash** |
| Diff size | 2 files, +12/−5: `package/src/orders/test/walker.mjs` (+5/−3), `package/src/orders/test/walk.test.mjs` (+7/−2) |
| Author | `Cayleb <cayleb@example.com>` (the lane's commit convention; the content is the R6 lane's work) |

## 2. What it changes (all three items, verified line by line)

1. **Benign-phase raise amount** (`walker.mjs:932`): `amount_usd: 0.5` → `Math.min(100000, Math.max(0.5, spentOf(t.id) - (t.budget_usd ?? 0) + 0.5))`. `spentOf(id)` is `SELECT COALESCE(SUM(cost_usd),0) FROM ledger_events WHERE task_id = ?` (`walker.mjs:293`), so the request is "enough to cover the task's recorded overrun, plus a 0.5 margin", floored at 0.5 and capped at 100000.
2. **Header comment** (`walker.mjs:19-21`): the claim "the envelopes plus what finished tasks spent stay within the order's budget" is removed and replaced with "The aggregate envelope limit is checked by the composed spend regression tests." **This is a comment-only hunk** — I confirmed no assertion code was removed (`git diff` shows only `//` lines; the emitted finding-kind set is byte-for-byte the same set: **52 kinds before and after**, set-diff empty).
3. **`walk.test.mjs`**: `walk(from,to,dbRoot,env)` gains an explicit `steps = STEPS` parameter (spawns the walker at `steps` instead of always the env default), and a new additive test `R6-10: seed 1546 completes after usage greatly exceeds the task envelope` runs seed 1546 at 400 steps with a 60 s timeout and asserts `findings == []`. Default behaviour of every existing call is unchanged (`steps` defaults to the previous `STEPS`).

## 3. Scope — verified as TEST-ONLY, cannot reach the shipped package

- Both changed files are under `package/src/orders/test/`. The shipped `0.2.2` artefact (`cayleb-dsh-fleet-0.2.2.tgz`, sha256 `37e3a542…`, extracted and inspected by me) contains **no `test/` directory, no `walker.mjs`, no `walk.test.mjs`** — `package/src/orders/` in the tarball is exactly the 17 product files. `package.json.files` enumerates only `src/**/*.js`, `src/sandbox/*.mjs`, `bin/*.mjs`, rows/presets/client — test paths are not packaged.
- Therefore the change **cannot change installed product behaviour**; it changes the fixture driver and its suite only. This is the correct home for a fixture-completion correction (Nox batch-2 condition: "the walker-raise itself lands as its own reviewed change").

## 4. Safety — decided from the code and from runs, not from the commit message

| Question | Finding |
|---|---|
| Can the raise amount be malformed (NaN/negative/absurd)? | No. `spentOf` is `COALESCE(SUM(...),0)`; `budget_usd ?? 0`; outer `Math.max(0.5, …)` then `Math.min(100000, …)` ⇒ always a finite number in **[0.5, 100000]**. Checked at the boundaries: (0, 0.5)→0.5; (0.6,0.5)→0.6; (334.38,140.56)→194.32; (1e9,0.5)→100000; (null,null)→0.5. |
| Does a larger raise bypass the policy gate? | No. It is a `budget_raise_request` hold — same hold path as before; tiering (`A`/`B` by ceiling) and the benign confirm are unchanged; the walker only changes the requested amount. |
| Does it weaken any walker invariant? | No invariant code changed. The walker still asserts the Stage-3 invariants (lease suspended while work waits on the owner, 80/100% alerts, ticket uniqueness, grant execution, raise-once-per-ticket, granted raise resumes). |
| Does it remove an assertion? | **Not from code.** It removes a *comment claim* that the walker asserted the aggregate envelope. That is a genuine narrowing of what the walker *claims*, and it is honestly declared in the header. Replacement coverage is the composed spend regression tests (`orders/test/spend.test.mjs`, the policy settle/tier tests), which are green on the final tree (orders 196/196 incl. spend 9/9; policy 65/65). |
| Is it safe to keep the walker as a gate judge after this? | Yes, with the known limit stated (below). The walker is a fixture judge; product envelope/ceiling behaviour is covered by the product suites and Stage-3 checks, not by this driver. |

**Empirical evidence I ran (independent of the maker):**
- Four-way differential on seed 1546: pre-fix tree 3/3 red with identical findings/stats; candidate 2/2 green; **candidate package + pre-fix walker = red**; **pre-fix package + this raise hunk only = green** ⇒ this hunk is the discriminating change, and the package fixes are not implicated (full table: `board/reports/fleet-checker-baseline.md` §4).
- Final-tree sweeps with the changed walker: n200 1060 seeds, n400 1060 seeds, biased 500, crash 500 — **all 0 findings, rc=0**; pinned seed-1546 test passes; 52-kind set unchanged.

## 5. Known limits the gate report must carry (from this change)

1. **Aggregate envelope is no longer walker-asserted.** The gate report must say so explicitly (known limit) and cite the replacement coverage (composed spend regression tests). Do not let the removed claim become a silent gap.
2. **The pinned `R6-10` test is a fixture-path guard, not a product regression.** It also passes on the pre-fix package under the new walker (my config 4), so its label/comment should say it guards the fixture completion path; the gate report should not present it as evidence that a product defect was fixed.
3. **Commit message language**: it says "own reviewed change"; this review is that independent review record (B5). The gate pack should cite this file.

## 6. Verdict

**SAFE and CORRECTLY SCOPED.** The change is confined to the test driver and its suite (cannot reach the shipped package), the raise formula is bounded and stays on the existing policy hold path, no assertion code was removed, and the one reduced claim (aggregate envelope) is honestly declared with real replacement coverage. Cleared for the final tree **subject to the three known-limit statements in §5 being carried in the gate report**.

— `fleet-checker`, 2026-10-31 20:15 ET
