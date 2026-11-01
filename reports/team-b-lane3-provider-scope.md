# LANE 3 — PROVIDER SCOPE (Stage 4b, minus Codex) — route / key-field / gap per provider

Filed 2026-11-01T14:55Z by `fleet-lead` (Team B, Lane 3) per the Lead's relay of the new Nox ruling.
**No API key value is handled, requested, copied, printed or stored by this lane** — every key is a
Cayleb-entered secret; each item below stops at the empty key field. Codex/OpenAI is **OUT** (scratched by
Cayleb; not revived). The provider code itself is **not yet in the plugin** (`src/engines/` does not exist in
any branch); what is ready here is the route/key-field/gap matrix the build must implement.

| # | Provider | Route verified | Key field ready | Gap named |
|---|---|---|---|---|
| 1 | **OpenCode Go** (base billing route) | **YES — live already**; this is the route the programme runs on today (DeepSeek V4.1 Flash volume engine). | Existing operator route; no per-lane key field needed for the already-live route. | None. Build item: register it as a model route for the steering slots with its meter story. |
| 2 | **DeepSeek direct** (overflow billing route) | **YES — OpenAI-compatible API at `api.deepseek.com`; model id for V4.1 Flash available in the provider catalog.** Config template prepared (base URL + model + meter rate row). | **YES** — empty `DEEPSEEK_API_KEY` field prepared; value is Cayleb's act. | **Cayleb key + account balance** (his acts, via Nox). No spend envelope is stated for this route ⇒ under the hard holds the cap is **zero** until he sets one; flagged to the Lead. |
| 3 | **OpenRouter** | **YES — OpenAI-compatible API; keys are created in the account dashboard; credits fund usage.** Config template prepared (base URL + model list + meter rows). | **YES** — empty `OPENROUTER_API_KEY` field prepared; Cayleb is setting the account up now. | **Cayleb key only** (+ optional credit purchase, his act/hard hold). No other gap. |
| 4 | **Grok (xAI)** | **PARTIAL — key route: xAI API Console (`console.x.ai`); the subscription product (The X Subscription: Grok+Cursor+X, weekly pool, prepaid top-ups at grok.com/dashboard) is a separate consumer product with its own billing surface. See the re-check below: the owner reports the subscription produces a usable API key and draws a separate X (dev) account pool.** | **YES** — empty `XAI_API_KEY` field prepared; base URL + model config ready. | **GAP NAMED: "API key is created in the API Console, not derived by the subscription"; the billing/usage pool question is with Nox/Cayleb (both ways recorded below).** No spend authorised; no account created. |
| 5 | **Cursor via API key** (unofficial bridge) | **PARTIAL/FLAGGED — the unofficial bridge is installed in Cayleb's personal profile and exposes a key field; it is not an official provider API.** The plugin can register it as a route, but its terms are unresolved (unofficial relay of a subscription product). | **YES** — empty key field prepared; route flagged in config as `unofficial: true`, disabled by default. | **Two gaps named:** (a) **Cayleb's key entry** (his act); (b) **terms route unresolved — the bridge is unofficial.** Per the plugin's terms rule, an unresolved route stays **flagged and off by default**, never silently used. Build item: ship it in the set as flagged; the guided first run may present its field once but the route stays disabled until an owner opt-in. |

## Cross-cutting build requirements this matrix fixes (for the Stage 4b build)

1. **Single-plugin acceptance property:** the guided first run presents **each provider's key field exactly
   once**, writes it write-only under the fleet principal, and never re-prompts. One install, one surface.
2. **Metering:** every route needs a `rates` row (meter story) before it can be enabled; a route whose usage
   cannot be metered is reported, not hidden. DeepSeek/OpenRouter/OpenCode have per-token rates; xAI has a
   per-token rate once funded; Cursor-bridge usage is metered from its own usage reporting or flagged
   unmeterable (`cost_usd:null`, blocks `unknown-model`) — the `unknown-model` fail-closed path in the ledger
   already gives the safe default.
3. **Spend authority:** only OpenCode Go has an already-authorized envelope (the live route). DeepSeek direct,
   OpenRouter, xAI and Cursor all require Cayleb-entered keys **and** a stated envelope; until then their
   effective cap is zero and no lane may call them. The Lead has the flag for Nox.
4. **Hard fence:** nothing here imports or runs any prototype code; the rejected access model (workers with
   danger-full-access as the operator) is not used anywhere. Providers run inside the fleet sandbox under the
   fleet principal, with Tier B unchanged.

**Status: not run, nothing enabled, no key requested.** Successor: the Stage 4b build takes this matrix as its
config spec; the key entries are Cayleb acts arranged through Nox at the guided-first-run step, and only
after the Stage 4/5 prerequisites are accepted.

## Re-check: Grok subscription vs API key (Lane 3, 2026-11-01T15:40Z)

The owner's own verification (via Nox, 1 Nov) says the Grok subscription **does** produce an API key and its
usage runs on the separate X (dev) account pool, not the RSI bot's meter. My earlier "subscription ≠ API key /
new spend" wording is narrowed accordingly; both sides of the evidence are recorded here and Nox rules:

- **Docs reviewed (external, x.ai):** [API billing FAQ](https://docs.x.ai/console/faq/billing) — API Console
  usage is paid by **prepaid credits or a monthly invoiced spending limit**; [API quickstart](https://docs.x.ai/developers/quickstart)
  — the API key is created in the API Console; [The X Subscription overview](https://docs.x.ai/x-subscription/overview)
  — the subscription is "one subscription that brings Grok, Cursor and X together", with a shared weekly
  included pool and prepaid top-ups managed at grok.com/dashboard; [extra usage](https://docs.x.ai/x-subscription/extra-usage)
  and [prepaid balance](https://docs.x.ai/x-subscription/prepaid-balance) describe top-ups for usage beyond the
  pool. **These subscription pages never mention an API key**; the API Console is presented as a separate product.
- **Owner/operator observation (first-hand, stronger for their own account):** a key was produced and usage
  drew a separate X (dev) account pool, not the RSI bot's meter.
- **Reconciled finding:** the route is **API Console key creation** (not an automatic key from the subscription);
  whether that key draws the subscription pool, a separate dev-account pool, or a prepaid balance must be read
  from **that account's own billing page** at the key-entry step. **No account created, no key requested, no
  spend.** Build side unchanged: empty `XAI_API_KEY` field + config; route enabled only after Cayleb's key entry
  and Nox's envelope ruling. If the account page shows the key is unfunded/needs top-up, that is a hard hold and
  is reported, not purchased.
