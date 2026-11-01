# TEAM B PROBE — seed 1546: reproduction, candidate fix, and the fix-or-scope-ruling pack

Owner: `fleet-lead` (Team B / DSH FLEET). Written 2026-10-31 18:05–18:15 ET (22:05–22:15Z).
Fences held: throwaway copies under `/tmp` only; **no writes to any mission branch, no worktree registered, no merge, no deploy, no install, no force-push, no spend, no credential touch, no write to `~/Documents/Codex/**`**.
This is a probe, not a gate pass. The authoritative gate run on the candidate is **Nox's** (`board/probes/nox-rulings-20261031T215204Z.md` §1); nothing here substitutes for it.

## 0. Facts used (observed live, not inherited)

```
$ cd ~/workspace/harness-build/dsh-fleet && git ls-remote origin main next bus
e8c8a4c2ad37ea9286c2b97b3d33af762fb19e74	refs/heads/bus
4a8a1d3f080416ccb545bee60655117584c960fd	refs/heads/main
0769797e2d32d89d3c22d0bd1cb2d94aef92ca52	refs/heads/next
   (2026-10-31T21:49:34Z = 17:49:34 ET)

$ git rev-parse main next origin/main origin/next
abfb842ef668e8e636bbd91211d11ab11a54d011	<- local main (1 ahead of origin/main)
0769797e2d32d89d3c22d0bd1cb2d94aef92ca52	<- next
4a8a1d3f080416ccb545bee60655117584c960fd	<- origin/main
0769797e2d32d89d3c22d0bd1cb2d94aef92ca52	<- origin/next
```

Two corrections to the brief and to the earlier lane record:
1. **The candidate `0769797` is on the remote** (`refs/heads/next`), not "committed on the desktop, locally" as Job 06's stale state paragraph says.
2. **The gate evidence under `evidence/r6-gates/` was produced on `next` @ `5184978`** — the R6 WIP — not on `main` @ `4a8a1d3`. `5184978` is `0769797`'s parent; `4a8a1d3` does not contain either. A reproduction run on `4a8a1d3` therefore uses the *pre-R6* code and cannot reproduce the finding (my first attempt did exactly that and came back clean — recorded here so the error is not repeated).

Method: `git archive <sha> | tar -x -C /tmp/team-b-b2/<tree>` for each of `5184978` (pre-fix), `0769797` (candidate), `4a8a1d3` (main). No `.git` in the copies, no refs written, nothing in the repo touched.

## 1. Reproduction on the pre-fix R6 WIP `5184978` — deterministic, 3/3

Command (identical for runs 1–3 except `WALK_DBROOT`):

```
$ cd /tmp/team-b-b2/pre-5184978/package/src/orders
$ TMPDIR=/tmp/team-b-b2/tmp-main WALK_RESTART=mixed WALK_DBROOT=/tmp/team-b-b2/tmp-main/db-pre1 \
  nice -n 19 /usr/bin/node --import ../core/test/_register.mjs test/walker.mjs replay 1546 400

  "findings": [
   { "seed": 1546, "step": 10149, "phase": "complete", "kind": "strand-never-completes",
     "detail": "o2 in_progress report=122 dw=command: t13:doneBr1/3 t14:openFr0/3 t15:doneBr1/3 t16:doneBr1/3" },
   { "seed": 1546, "step": 10149, "phase": "complete", "kind": "strand-never-completes",
     "detail": "o3 in_progress report=50 dw=command: t1:blocked(budget-exhausted)r0/3 t2:openFr0/3" }
  ],
  "levels": { "o3": "needed-amend", "o7": "needed-amend" },
  "stats": { "finisherVerdicts": 0, "leaseLost": 20, "spendTickets": 14, "exhausted": 15,
             "raises": 8, "resumed": 7, "clockSweeps": 10, "restarts": { "plain": 0, "host": 4 } }
```

- rc=0 each run; 402-line trace each; **findings and stats identical 3/3** (`db-pre1..3`).
- This matches the committed artifact `evidence/r6-gates/replay-1546.txt` (same two findings at step 10149; same stats: `leaseLost 20, spendTickets 14, exhausted 15, raises 8, resumed 7, host restarts 4, clockSweeps 10`). The walk is reproducible for this seed; only claim tokens / checker ids differ run to run (crypto randoms, never compared).
- Artifacts: `rep-pre-1.log` 18:02:23 ET sha256 `664552a3f0b1c61b…`, `rep-pre-2.log` 18:02:25 `0adeed5cf163c425…`, `rep-pre-3.log` 18:02:26 `4eb6e21bdc3f55bb…`.

## 2. The candidate `0769797` is green at seed 1546 — 2/2

Same command against `/tmp/team-b-b2/next-0769797/package/src/orders`:

```
"findings": [],
"levels": { "o3": "needed-amend", "o7": "needed-amend" },
```

rc=0, 18:03:11 and 18:03:12 ET. Artifacts `rep-cand-1.log` sha256 `fc063060d850fb99…`, `rep-cand-2.log` `713695565f065e52…`.
Note the walker's own level label for `o3` is still `needed-amend` — the walker itself classifies that order as a phase-2 gap, and the candidate's green is produced in the benign phase, not by `o3` reaching done.

## 3. Differential experiment — which change actually turns the seed green?

Four configurations, same seed 1546 @400 steps, same command:

| # | Package | Walker driver | Findings | Artifact |
|---|---|---|---|---|
| 1 | pre-fix `5184978` | pre-fix `5184978` | **2 strand-never-completes** | 3/3, §1 |
| 2 | candidate `0769797` | candidate `0769797` | **0** | 2/2, §2 |
| 3 | candidate `0769797` | **pre-fix** `5184978` | **2 strand-never-completes** (identical detail lines) | 2/2 |
| 4 | **pre-fix** `5184978` | pre-fix + **only** the candidate's 2-line bully-raise hunk | **0** | 2/2 |

Config 3 command: candidate tree copied to `/tmp/team-b-b2/cand-with-old-walker`, then
`cp pre-5184978/package/src/orders/test/walker.mjs …/cand-with-old-walker/package/src/orders/test/walker.mjs` and replay.
Config 4 command: pre-fix tree copied to `/tmp/team-b-b2/pre-with-newraise`, then patched only this hunk in `test/walker.mjs`:

```
-    const args = { order: oref, task: ref('task', t.id), amount_usd: 0.5, reason: `benign ${round}` }
+    // Cover the recorded overrun in one permitted raise when possible; tiny raises can otherwise starve unrelated retry timers.
+    const amount = Math.min(100000, Math.max(0.5, spentOf(t.id) - (t.budget_usd ?? 0) + 0.5))
+    const args = { order: oref, task: ref('task', t.id), amount_usd: amount, reason: `benign ${round}` }
```

Results: config 3 = 2 findings (2/2, `rep-cand-oldwalker-1.log` sha256 `5538c313189f5a12…`); config 4 = **0 findings (2/2**, `rep-pre-newraise-1.log` 18:05:32 ET sha256 `935eebac341843ec…`, `rep-pre-newraise-2.log` 18:05:33 ET sha256 `55cb09c6485c0296…`).

**Conclusion from the differential: seed 1546's two strands are resolved by the walker's benign-completion raise retune alone. The three package changes in `0769797` (order-level spend cap in `ledger.js`, live-task overspend in `orders/tasks.js` commitments, `staleAuto` voiding of ended-task spend tickets in `policy.js`) are not what makes seed 1546 pass.**

## 4. Exact walker delta `5184978` → `0769797`

`diff -u` shows **two hunks only**:
1. the benign raise amount (above);
2. a **header-comment** change that drops the claim "the envelopes plus what finished tasks spent stay within the order's budget" and replaces it with "The aggregate envelope limit is checked by the composed spend regression tests."

The emitted invariant set is unchanged: both files emit the same **52** finding kinds (no assertion kind was removed). So the header edit is a scope statement, not a deleted check, but it is still a change to what the judge claims to cover.

Mechanism, from the lane's own record (`~/Documents/Codex/2026-10-31/task/mission1/MISSION1-HANDOFF.md`): a synthetic task had spent **334.38 against a 140.56 envelope**; the benign phase made **157 repeated $0.50 raises**; the continual progress prevented time advancing for an unrelated final-task retry. That is the `o2` strand; `o3` is the budget-exhausted task the tiny raises never cover.

## 5. Status of the ruling (already recorded; this probe corroborates it)

`~/Documents/Codex/2026-10-31/task/mission1/NOX-EVIDENCE-20261031.md` records the operative ruling:
> "The ruling is **fixture-completion starvation for this reproduced seed**, not proof that the product can never strand work… Preserve the production overspend/stale-ticket fixes and the separate full gates."

So the *fix-or-scope* question is already answered in the direction of **scope/fixture**, and this probe independently confirms it: the package fixes are not needed for 1546, and the walker retune is sufficient. What remains for Nox is to rule on the **record**, not on the mechanism.

## 6. Fix-or-scope-ruling pack — decisions Nox/Lead still owe

**D1. Gate wording.** Does the Stage 2 gate report record the walker leg as: "seed 1546 = fixture-completion starvation; walker benign phase corrected to raise enough to cover a recorded overrun; 400-step normal sweep green under the corrected walker (Nox's candidate run)" — with the mechanism and the differential above cited? *Recommendation: yes; and record explicitly that the band-aid is in the fixture driver, not the product.*

**D2. Commit separation.** `0769797`'s message is "R6 candidate: account for overspend and stale tickets" yet it also changes the walker driver and the walker's header claim. For a clean record, the candidate should be split (or annotated in the gate report) into (i) package fix (ledger/tasks/policy + their tests), (ii) walker/fixture fix (raise retune + header claim + pinned seed test). *Recommendation: annotate in the gate report if splitting is not authorised tonight; do not rewrite pushed history.*

**D3. Pinned regression test scope.** The candidate's new test `R6-10: seed 1546 completes after usage greatly exceeds the task envelope` (walk.test.mjs) passes on the pre-fix package when the new walker is used (config 4) — so it guards the fixture path, not a package behaviour. *Recommendation: label it as a fixture-completion guard in the test name/comment; keep it.*

**D4. Lost invariant.** The header claim "envelopes + finished spend stay within the order's budget" is no longer asserted. The replacement claim ("composed spend regression tests") covers the reported allocation cases only. *Recommendation: record as a known limit in the gate report ("aggregate envelope is not walker-asserted; specific allocation cases covered by composed tests") — no new machinery tonight.*

**D5. Survivors.** The full orders mutation run at `5184978` has **10 survivors** (164 killed / 174). `docs/RESUME.md` says the R5 run also had 10 and the candidate's `settle` pass may cover them, but "equivalence on this candidate has not been established" (`MISSION1-HANDOFF.md`). Nox's candidate mutation run is the evidence; if survivors persist, the gate report must disposition each one or name the leg unverified. *Recommendation: no gate pass on the mutation leg until Nox's run lands and survivors are dispositioned.*

## 7. What this probe does NOT establish

- Not the 1000-seed 400-step sweep, not the unit suites, not the race suite, not the mutation runs, not the client suite — **those are Nox's candidate run**, in flight tonight.
- Not a release, merge or install verdict; no 0.2.2 artifact exists (`/var/lib/fleet-drop` holds `0.2.0`/`0.2.1`; `dist/` holds `0.2.1`; `package.json` is `0.2.1` on both main and next).
- The seeded replay is deterministic for this seed, but the 400-step failure was found in the 1000-seed sweep; passing seed 1546 in isolation is necessary, not sufficient, for the sweep leg.

## 8. Artifacts on disk (throwaway, safe to delete)

`/tmp/team-b-b2/` — `pre-5184978/`, `next-0769797/`, `main-4a8a1d3/`, `cand-with-old-walker/`, `pre-with-newraise/`, `out/` (logs above), `tmp-main/` (walker DB roots).
