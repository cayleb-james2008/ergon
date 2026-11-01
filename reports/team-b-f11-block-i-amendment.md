# F11 — Contract Block I amendment (ledger pricing: long-context tiers + cache rates)

**Status: AMENDMENT TEXT, applied to the gate pack per Nox ruling `20261101T034100Z`.** Maker: Team B
(`fleet-lead`). Text only — **no code change**, the code already implements the amended text; the contract text
was behind the code, not the reverse. Filed 2026-11-01T14:20Z. F11 closes when this text lands in the
consolidated gate pack (this file + its reference from `stage2-gate-report-FINAL.md`).

## Nox's ruling (verbatim substance, 034100Z)

> "ITEM (4) — F11: RULED BY NOX. The ledger's long-context tiers + cache rates reflect real provider pricing;
> the contract text was behind the code, not the reverse. Closure = amend contract Block I to name the tiers
> and cache rates as the pricing basis. Team B applies the text amendment; F11 closes when the amended text
> lands in the gate pack. Direction-of-harm noted: current behaviour over-prices (fail-conservative)."

## Amended Block I text (replacement wording; this is the text to file into the contract)

> **Block I — the cost book (ledger pricing).**
>
> Every model call is priced once, at record time, from the policy rate table row `rates["<provider>/<model>"]`:
>
> `cost_usd = (input_tokens*input_per_mtok + output_tokens*output_per_mtok
>            + cache_read_tokens*cache_read_per_mtok + cache_write_tokens*cache_write_per_mtok) / 1e6`
>
> - The four token classes are as reported by the provider; **cache tokens are disjoint from `input_tokens`**, and a
>   rate row that omits a cache rate falls back to that row's `input_per_mtok` for that class.
> - **Long-context tiers are part of the pricing basis.** When the prompt size (`input_tokens + cache_read_tokens +
>   cache_write_tokens`) exceeds a `tiers[]` entry's `input_tokens_above`, the highest such entry's rates **override**
>   the row's base rates for that call. The shipped defaults mirror the pinned catalog's long-context prices
>   (e.g. `anthropic/claude-haiku-5-5` base `0.1/0.5`, cache `0.01/0.125`, tier above 100,000 prompt tokens
>   `0.5/2.5`, cache `0.05/0.625` USD per Mtok; `claude-sonnet-5-5` `2/10`, cache `0.1/2.5`;
>   `claude-opus-5-5` `4/20`, cache `0.2/5`).
> - A model absent from the table has `cost_usd = null`: it adds nothing to spend and blocks the task
>   `unknown-model` with no ticket (fail closed).
> - **Direction of harm (recorded):** pricing with tiers and cache rates can exceed the pre-amendment
>   two-term formula (e.g. a 200k-prompt haiku call prices `0.1025` USD vs `0.0205` USD), so a budget can be
>   reached sooner. The behaviour **over-prices, never under-prices** — fail-conservative for spend safety.

## Evidence this text matches the shipped candidate (tree of record `cf93107`)

| Artifact | SHA-256 | Where |
|---|---|---|
| `package/src/ledger/ledger.js` (cost function `costOf`; default `rates` incl. tiers + cache; `ALERT_AT=0.8`) | `0a682cb4d26df5c19fff3b0b74c2c75c6106ed46489754763120a65cdc5ec8a7` | cf93107 |
| `package/src/ledger/API.md` ("Pricing." paragraph + `rates` policy row) | `864826e3f4b036b04c274025c3faba15f641c765778e3e9abe7bffb08c2b07a7` | cf93107 |
| `package/src/ledger/index.js` | `a60f5d923c0d15c4d1d29f394445fc22efbe01f9d4cf90a51a5bafd3c55693a4` | cf93107 |

Review provenance: `evidence/review-results/review_contract.txt` finding **F11** (severity low, file
`package/src/ledger/ledger.js` line 60) reproduced the formula difference (`input 200000/output 1000`:
package `0.1025` vs contract-I `0.0205`) and recorded the fix as one of: amend Block I to name tiers and cache
rates, or drop them from defaults. Nox ruled the former.

## Closure statement (for the gate pack, not a PASS claim)

F11 is closed **as a contract-text finding** by this amendment: the contract now names long-context tiers and
cache rates as the pricing basis, matching the shipped code, with the harm direction recorded. No behavioural
change, no re-test is implied; the existing ledger suite (6/6 on the final tree) and the gate checks stand as
implemented. Once this text is filed into the contract, F11 has no surviving open item from Team B's side.
