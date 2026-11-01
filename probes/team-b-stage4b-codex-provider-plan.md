# TEAM B — Stage 4b Codex engine provider: scope/branch plan (PREPARED, ON HOLD)

Written by `fleet-lead` 2026-10-31 20:35 ET, per the Lead's steer (Nox manager mail 00:20Z). **Status: HOLD — do not start.** The credential decision is made (D2); the **sequencing/base ruling is still open** — no branch, no code, no credential touch until the Lead says GO.

## Ruling recorded (D2)

- **"Use my ChatGPT login."** The Codex engine provider authenticates with **Cayleb's ChatGPT login under the fleet user**.
- Binding mechanics: **NO credential is copied anywhere**; the login itself is a **Cayleb-present step at the Stage 4b gate** (he signs in; the fleet user holds the resulting session); **nothing credential-touching is improvised by any lane before that step**.
- The credential decision no longer gates the provider build. The provider remains an **engine integration under the fleet user** — never dispatch into Cayleb's personal ChatGPT, Claude Desktop or Codex desktop apps (standing exclusion).

## Scope (from the canonical adoption row 4)

Canonical row: *Engine providers incl. subscriptions — `V1ki/dsh-plugin-subscriptions` + `lcestou/dsh-oh-my-claude`; adopt + wrap (fleet-user containment, credential service, ledger pricing, kill/meter hooks; per-provider subscription ToS risk flagged at the gate, never silently adopted); stage 4b.*

- Implement the **Codex engine provider (check 4.11)** in the plugin's `src/engines/`: sandboxed execution, credential via the fleet credential service (ChatGPT-login session under the fleet user), usage priced into the ledger from the engine's own usage reporting.
- Wrap the adopted subscriptions plugin(s) per the canonical row: fleet-user containment, credential service, ledger pricing, kill/meter hooks. **Per-provider subscription ToS risk flagged at the gate** — never silently adopted; where terms are unresolved the default stays API keys and the route is flagged.
- Carried equally for every engine: Harness-squared + memory protocol + accumulated skills/lessons.
- **Excluded from this scope:** order/budget/rails/policy code, service/profile writes, any credential copy, any dispatch into Cayleb's personal apps, any packaging/absorb-or-remove change (that stays with the operator-path build).

## Branch/worktree plan (not created until GO)

- Dedicated **isolated branch + worktree** off the base Nox rules, e.g. branch `provider/codex-chatgpt-login` in `~/workspace/harness-build/dsh-fleet-4b` (name/base final at GO). **Single Team B maker: `fleet-lead`** — no parallel implementation.
- Scope confined to the provider package + its tests + that provider's package metadata (`package/package.json` exports/files entries and the provider's rows/manifest entries only).
- No writes to `main`/`next`, no force-push, no service or profile writes.

## Anticipated files (indicative, confirmed at GO)

- `package/src/engines/` — Codex provider module + API/NOTES; tests alongside.
- `package/package.json` — exports/files entries for the new provider path.
- Provider package metadata/rows entries for the single provider (no cross-module edits).
- Composed/e2e tests where the provider is mounted.

## Evidence and gate (check 4.11)

- `4.11` check output on the sandboxed provider + a **ledger extract** showing an engine session's usage priced in from real usage reporting.
- **ToS flag line** for the subscription route in the gate report.
- Credential-step evidence: Cayleb's sign-in happened at the gate; the fleet user holds the session — **no secret value is printed, copied or logged** anywhere.
- Certified by a **fresh `stage4b-checker`** (not `fleet-checker`); maker never checker.
- Budget: **zero** until an envelope is stated (Stage 4b's cap is still unstated; Stage 4's $5/day + $0.50/drill are Stage 4 caps, not 4b's — do not assume).

## Hold state (as at 20:35 ET)

- [x] D2 recorded
- [x] Scope/branch plan prepared
- [ ] Nox sequencing/base ruling — **OPEN**
- [ ] Lead GO — **not given**
- No branch created, no code written, no credential touched, no spend.
