# ERGON — verification record: three independent checks, what each one caught, and the tip that passed

Filed 4 Nov 2026, 02:00 ET by the ERGON stream lead (opencode-go/deepseek-v4.1-flash). This is the *evidence* half of the board
work: who checked what, what they found, and what changed because of it. The feature summary is in
`reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md` and the morning section; this file exists so a reader can see that "green"
meant something at each step.

## The chain
| pass | target | verdict | what it caught |
|---|---|---|---|
| 1 | `d9c782b` | **FAIL** | item 3 (product): 3 s answers → Error/Live cycle every 4 s, 36 % Error duty, run ends on Error. item 7: `round3`'s host term (`4 × hostRef`, and `4 × 41 > 100`) retired the "cost does not grow with jobs" rule; `round4` asserted the best of three. Unbriefed: "Reconnecting" at 1577 ms on a first load that never connected; editing locked during each false Error. |
| 2 | `3780e04` | **PASS** | re-measured 2 s → 4.2 % Error / 3 s → 12.5 %, 0 % in the last 5 s, ends Live; hung first read: no "Reconnecting", Error at 2600 ms. **Negative control fired**: an injected ~204 ms per-job regression failed the long-task test (210–300 ms against a 150 ms bound) where the clean build showed 75 ms against 178. Briefed limit: with the data endpoints hung but the probe answering, the label stayed **Live for 26 s over a frozen board**. Also: +80 ms at 200 jobs still passes round4's median rule (a +200 ms control fails it); the adaptive probe deadline hides a full hang for up to ~10.7 s after a slow phase. |
| 3 | `9cc7d95` | **PASS** | the 26 s case now ends honestly: probe answered 22 times while `/lines` and `/entries` never answered → blank → **Error at 15 088 ms**, `data-state` polling → error, never Live, 0 rows, 2 + 2 data requests in 20 s (first pair + one retry). Deadline fires at 121 ms against a 120 ms test setting; three `load()` calls start one read; process exits 51 ms after a successful read (no leftover timer); 14 s answers still resolve under the 15 s deadline. |

Reports: `BOARD-FINAL-TIP-CHECK.md`, `BOARD-FINAL-TIP-CHECK-2.md`, `BOARD-DATA-READ-CHECK-3.md` (all in
`~/fleet-scratch/overnight/`). Each was a separate sub-session on `opencode-go/deepseek-v4.1-flash`, each worked from its own
`git archive` export, none wrote to the repo, none deployed, none read hosted Supabase rows.

## What changed because of the checks
1. **`2150fa8`** — the probe deadline is 3× the connection's own measured round trip (1.5–10 s), so a slow link is not called
   down; a timeout alone is not an outage; the word "Reconnecting" now requires a loaded board.
2. **`7035ab9`** (the other ERGON session) — `round3` ceiling 4× → 2×, `round4` asserts the median with the worst capped at 1.5×,
   and the hung-path probe test asserts that nothing keeps starting after setup.
3. **`26e6c9e`** (the other session again) — the hung-path rule was rewritten after it false-failed correct code once in 12 runs
   and passed a real duplicate probe at 1.6 s; the new rule is "after the first 5 s, at most one start".
4. **`9cc7d95`** — the board read gets a 15 s deadline and one outstanding read that a second caller joins, so a data-path-only
   hang ends in Error instead of a frozen board labelled Live.
5. **Docs** — `3780e04` (probe behaviour), `9cc7d95` (read deadline). Both describe what was measured, including the cost side.

## Figures a reader can re-run
- `npm test` → **121/121** (was 119; the two new tests are the read deadline and "a read inside the deadline is untouched").
- `npm run test:e2e` → **108 tests, 103 pass, 0 fail, 5 skipped, exit 0** (logs: `evidence/board-e2e-hostref-20261104.log`,
  `…-connfix-…`, `…-dataread-…`).
- `node --test test/e2e/connection.test.mjs` → **10/10**, including the 12 s duty-cycle runs at 2 s and 3 s.
- `node --test "test/agent/*.test.mjs"` → **22 pass / 0 fail / 12 skipped** (the 12 are the live ones — see the open gap below).

## The one gap this suite still has, named
Five e2e tests (`supabase-live.test.mjs`) and twelve agent tests are **skipped by design** unless `E2E_SUPABASE=1` with a local
Supabase stack. They cover realtime delivery, the polling fallback, the LIVE six-page sequence, a stale revision refused, a deleted
job never recreated, and revoked/expired keys. Nothing on this branch has exercised those against a real backend tonight; the images
and CLI are present, and a run was attempted **twice** and failed for an environment reason, not a project one: the CLI's postgres
container starts but its entrypoint shell dies immediately with `sh: can't fork: Resource temporarily unavailable` (a trivial
container from the same image runs fine, `Memory=0`/`PidsLimit=0`/`ShmSize=64 MB`, and podman ran two postgres containers cleanly at
00:58 ET — so it is a fork failure specific to that container's startup burst on this host). Full transcript and diagnosis:
`evidence/board-live-stack-20261104.log`. **But the data layer was then verified a different way** (02:10–02:25 ET): the repo's own SQL/REST contract suite
(`test/sql/run.mjs`, 32 assertions) was run against a plain `supabase/postgres` container + `postgrest` + a locally minted anon
JWT + a small `/rest/v1` proxy — **30 passed, 2 failed**, and both failures are the harness, not the schema: there is no
realtime service in it (that one case needs the full stack) and `supabase_read_only_user` is a reserved role whose password
cannot be set as `postgres`, so the hosted-pack step-0 login fails. What passed is the substance: RLS and grants, the key
lifecycle (mint, check, revoke, expiry), `save_entry`/`remove_entry`/`board_token`, the publication guards, re-applying the
migration over live data, apply-twice structure equality, the hosted-faithful restore/rollback pack, and atomic failure on an
incompatible column. Log: `evidence/board-sql-contract-20261104.log`.

**So the honest state is narrower than "unverified":** the demo-and-mocked UI paths and the whole SQL/REST contract are
verified on this branch; **realtime delivery and the browser-to-live-backend paths are not**, tonight. The live v3 deployment has been serving the same RPCs since 08:02 ET yesterday, which is evidence
of a different kind — it is not a test, and it is not counted as one here.

## Not deployed
Live production is v3 `455a9da`. The branch tip `26e6c9e` is 12 commits on from `4319779`. The owner's one yes is still the only
thing between them; the packet (one command, build id, post-deploy checks, rollback) is in the morning section.
