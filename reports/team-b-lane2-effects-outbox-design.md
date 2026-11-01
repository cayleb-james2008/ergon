# LANE 2 — design: write-first durable Effects/Outbox for orders (adopted pattern; no prototype code)

Filed 2026-11-01T15:12Z by `fleet-lead` (Team B, Lane 2) under the Lead's relay of the Nox ruling that adopts
the **pattern, not the code**: nothing is imported, vendored or run from the prototype. This is the adopted
pattern with the most operational teeth — the evidence is this weekend's own lane deaths (the delegator
context died at 100%; lanes were retired mid-run; work in flight was lost and had to be re-derived).

## 1. Gap this fills (named)

In-flight work has no durable record between "the lane decided to act" and "the board shows the result". When a
session dies mid-turn, the intended side effect (a board post, a claim, a build step, a verdict) is either
lost or re-derived by guesswork; retries are unsafe because no record says what was already attempted. The
outbox makes every consequential step **recorded before execution**, retryable with backoff, and
recoverable at restart — so an interrupted order is resumed from facts instead of re-guessed.

## 2. Model (additive; board schema change only, no new service)

New table `effects` (migration `orders/00X-effects`, append-only states with a current-state row):

```
effects(
  id INTEGER PRIMARY KEY,
  order_id, task_id NULL,
  kind TEXT,                 -- 'board.post' | 'board.claim' | 'task.transition' | 'evidence.post' | 'verdict.post' | 'bank' | 'notify'
  payload TEXT,              -- canonical JSON; payload_sha256 recorded beside it
  payload_sha256 TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  state TEXT NOT NULL,       -- 'intended' | 'sent' | 'observed' | 'failed' | 'abandoned'
  attempts INTEGER NOT NULL DEFAULT 0,
  next_attempt_at INTEGER NULL,
  last_error TEXT NULL,
  causation TEXT NULL,       -- the op/session/turn that generated the effect
  created_ts, updated_ts
)
UNIQUE (idempotency_key)
```

Invariants:
1. **No effect without a record.** A consequential action is written `intended` *in the same transaction* as
   the state change that motivates it. If the record cannot be written, the action does not run (fail closed —
   the same rule the action log already enforces for guarded calls).
2. **At-least-once execution, idempotent effects.** Every effect carries an idempotency key derived from
   (order, task, kind, payload_sha256); the executor (re)applies it and the receiving op deduplicates on the
   key (the board's ops already take idempotency keys; the bus already has leased claims).
3. **State machine:** `intended → sent → observed` (or `failed` with `next_attempt_at`), `abandoned` only by an
   explicit owner/lead decision with a reason. `observed` requires the receiving side's answer (a board seq, a
   verdict id), not a local assumption.
4. **Retry:** exponential backoff (1s → 2s → … capped 60s; attempts recorded); a poison effect stops after N
   attempts and surfaces as `failed` in the order view — never silently dropped.
5. **Restart recovery:** at service start, scan `intended` + `sent` + stale `failed`; for each, reconcile with
   the receiving side by idempotency key, then re-send or mark `observed`; write a recovery note per effect
   (`recovered_at`). Recovery is level-triggered, not a one-shot pass (the ledger's settle pass is the model).
6. **Crash-window analysis** (must be in the design review): (a) crash after record, before send → recovered
   and sent once; (b) crash after send, before `sent` commit → reconciled by key, no duplicate; (c) crash after
   send, after `sent`, before `observed` → reconcile; (d) crash during recovery → idempotent, safe to repeat.
7. **Read models:** order/task views show "effects pending/failed" so a lead sees work in flight; the status
   digest reports counts only (ids/states/counts/holds — never content).

## 3. What changes in the code (build items, in order)

1. `orders` migration + `effects` table + executor loop (bounded per tick, no unbounded fan-out).
2. Wrap the existing consequential writes: board post/claim/release, task transitions, evidence/verdict/bank.
   The guarded tool path (fleet-policy) keeps its intent row; the outbox is the *durable* layer beneath it,
   not a second authorisation layer.
3. Restart-recovery pass wired into `Service.init` (like the ledger's settle pass) + a `recover-now` op for
   tests.
4. Tests: one test per branch of §2.6, plus a "kill -9 mid-effect" test driving the real store; the existing
   race/lease tests must stay green.
5. Read-model additions (pending/failed counts) + digest counts.

## 4. Fences and non-goals

- **No prototype import.** Pattern only; no code, no coupling to alpha DSH internals.
- **Not an authority layer:** authorisation stays fleet-policy/Tier B; Effects/Outbox never grants or widens.
- **Not a queue for model work:** model calls stay metered by the ledger; the outbox is for consequential
  board/side-effecting steps, not for token routing.
- **Minimalism:** one table + one executor + one recovery pass; if a step can be made idempotent by existing
  board semantics without an effect row, it does not get one. Each row must name the failure it prevents.

## 5. Acceptance (for a fresh checker)

- Crash/kill matrix: each of §2.6 (a)-(d) demonstrated; no duplicate board post for one logical action; no lost
  intended action across a service restart; a poison effect surfaces `failed` and stops retrying.
- Existing Stage 2 board/race/lease suites and the isolated-tree scratch acceptance stay green (Lane 2 baseline:
  60/60, `board/reports/team-b-lane2-scratch-acceptance.md`).
- A worked example: kill the service mid-order, restart, and the order resumes with every effect accounted for
  (the weekend's failure becomes a passing test).

## 6. Where it lands

Lane 2 build queue (with hands race fix + bus package assembly), then Stage 9 cold-start/restore. Design only in
this batch; the build is the next Lane 2 pull after the race fix.
