# LANE 2 — Effects/Outbox: first build step (isolated branch, tested)

Filed 2026-11-01T17:22Z by `fleet-lead` (Team B, Lane 2). Isolated worktree only:
`~/workspace/harness-build/scratch-tb-l2/build-effects`, branch **`tb/effects-outbox`** off `integration/0.2.2`
@ `cf93107`. The integration tree is untouched; nothing installs from this lane.

## 1. What was built (write-first durable effects)

| File | Change | sha256 |
|---|---|---|
| `package/src/orders/schema.js` | +`EFFECTS_MIGRATION = 'orders/006-effects'`, `EFFECTS_STATEMENTS` (table `effects` + two indexes) | see `git diff` in branch |
| `package/src/orders/effects.js` (new) | `createEffectsApi(K)` — the outbox API | see branch |
| `package/src/orders/store.js` | registers the migration, constructs `effects`, exposes it on the orders API | see branch |
| `package/src/orders/test/effects.test.mjs` (new) | 5 focused tests (below) | see branch |

`effects` row: `order_id/task_id/kind/payload/payload_sha256/idempotency_key(unique)/state/attempts/
next_attempt_at/last_error/causation/created_at/updated_at`, state ∈ `intended|sent|observed|failed|abandoned`.

API: `record` (write-first, idempotent on key), `markSent` (attempts+1), `observe` (receiving side confirmed),
`fail` (backoff; poison ⇒ `abandoned` at `MAX_ATTEMPTS=5`), `due` (level-triggered work list), `recoverStale`
(restart recovery: stale `sent` → `intended`), `counts` (per-order read model), `view`.

## 2. Test evidence (run in the worktree, plain node, real store)

```
effects.test.mjs: ℹ pass 5  ℹ fail 0
```
- write-first record + same key does not duplicate + invalid kind/payload refused;
- forward-only states (`intended → sent → observed`; invalid transitions refused);
- backoff/poison: not due before backoff, due after; `MAX_ATTEMPTS` ⇒ `abandoned`, never due again;
- restart recovery: stale `sent` → `intended` (idempotent resend), becomes due;
- per-order counts read model.

**Regression — existing orders suites on the same worktree (all green):**
`board 8/8 · claims 18/18 · cycle 21/21 · flow 15/15 · freshness 6/6 · liveness 40/40 · plugin 18/18`.

## 3. What is NOT done yet (next build pulls, in order)

1. **Wire the consequential writes through the outbox** — board post/claim/release, task transitions,
   evidence/verdict/bank: record `intended` in the same transaction as the state change, then execute/confirm.
2. **Executor loop** — bounded per tick, backoff scheduling from `next_attempt_at`, poison surfacing in the order
   view; wire into `start()`/boot pass like the lease sweeper.
3. **Restart-recovery pass** wired into `Service.init` + a `recover-now` op for tests.
4. **Crash-matrix tests** (design §2.6 a–d): kill -9 mid-effect, duplicate-free resend, recovery-repeat; plus a
   worked "kill the service mid-order then resume" scenario.
5. **Regressions:** isolated-tree scratch Stage 2 acceptance stays 60/60; desk test-suite green; then a fresh
   checker (maker never checker) before any merge/propagation.

## 4. Status

First build step **implemented + unit-tested + no regression** on the isolated branch; not merged, not installed,
no service action. Recorded for the Ledger/Outbox acceptance track.
