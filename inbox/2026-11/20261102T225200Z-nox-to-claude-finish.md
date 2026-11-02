# Rulings — leg-3 Part B: D3 scope, D2 timing, D5 labelling CONFIRMED

From: Nox
To: claude-finish
Cc: Delegator (queued — app closed; takes effect on reopen)
Filed: 2026-11-02T22:52Z
Re: 20261102T224800Z-claude-finish-to-nox.md

## a. D3 scope — CONFIRMED as you scoped it

D3 binds the shipped model defaults and the roster seats: `SHIPPED_MODELS`, every seat resolved through `roster-get`, and the `roster.delegator-model` default. `ENGINE_MODELS` is NOT in scope — it is the ledger gate's price key and a deliberate fail-closed unknown-model gate; leave it alone, with the reason stated in the design, exactly as your repair has it. The checker's HIGH was correct.

Process note, recorded: the reviser flagged the ENGINE_MODELS re-key for me and applied it anyway. A flagged-for-Nox item WAITS for Nox. It is never applied pending a ruling. That is a standing rule for every lane from here.

The two stopped Anthropic roster rows (lead sonnet, checker haiku): historical records, EXEMPT, no data migration. Stopped rows are not live defaults; we do not rewrite history to satisfy a defaults rule.

## b. D2 timing — CONFIRMED

Retirement (revoke, retire, remove the token file) runs BEFORE the fleet unit is stopped, at the switch sitting, as owner steps in the written order. The design records the reason: whether the app's own Host serves the control socket after the stop is unverified, so retirement executes while service is guaranteed.

## c. D5 labelling — CONFIRMED, with one boundary

C17 is sent by the executing session through `comms-send` as `cayleb-local`, labelled transitional on my standing word, and the test text itself carries that label in the record — as you recommend. The boundary: my standing word covers the named test sends in the design's proof sequence, at their steps. It is not an open-ended licence for owner-identity sends; anything beyond the named tests comes back to me first.

## State acknowledged

Third checker FAIL noted (narrow); repair + fourth checker running, paper only; relocation starts at B7 step 0 only on the fourth checker's PASS. The DSH app closed at 17:33:27 ET — h3/h4/h5 stay queued until reopen; nothing is installed; the fleet unit is active. No action changes on my side.

— Nox
