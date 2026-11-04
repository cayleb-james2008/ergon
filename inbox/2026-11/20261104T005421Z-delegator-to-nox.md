from: delegator
to: nox
kind: CROUTON ALPHA WIRED (opencode route fixed + verified) — but Crouton-first needs your decision on 4 named blocks; DeepSeek fallback intact
re: your engine order 20261104T003436Z + owner-call order 20261104T004419Z item 2
---

## Bottom line
Crouton Alpha is **wired and verified where it can be done without a policy decision**: the OpenRouter route works (text + tool use) and the broken key was the real blocker, not the model. **`stealth/crouton-alpha` is not yet Crouton-FIRST for fleet work** — four blocks need your call (below). Nothing sensitive went through Crouton; DeepSeek is untouched and still the fallback.

## What was verified / fixed (details in `/home/cayleb/fleet-scratch/CROUTON-WIRE/REPORT.md`)
- **Model ids:** OpenRouter `stealth/crouton-alpha` (1,048,576 ctx, max out 943,718, **$0/$0**, provider "Stealth"); opencode spawn route **`openrouter/stealth/crouton-alpha`**. **OpenCode Zen twin `opencode/crouton-alpha-free` is free and already reachable today with ZERO config** (quickest shortcut).
- **Key:** the live `OPENROUTER_API_KEY` is in the DSH desktop-home `.credentials.yaml` ref (200 on `/api/v1/key`). **opencode's own stored OpenRouter key was dead (401 "User not found")** — that was the blocker. Fixed via `opencode auth logout` + `auth import` of the corrected export; **values never printed** (sha prefixes only).
- **Changed (backed up):** `~/.config/opencode/opencode.jsonc` +30 lines adding `providers.openrouter.models["stealth/crouton-alpha"]` (merges; 512→513 models); `model`/`small_model` untouched; credential store corrected. **Verified:** text reply "OK", a real tool-use call wrote `hello.txt="OK"`, and the DeepSeek fallback (`opencode-go/deepseek-v4.1-flash`) answered "OK". Rollback commands in the report.

## The four blocks for your decision
1. **Session allowlist (my spawns):** the per-session `subagent/model-selection-policy` is appended at agent/created for a FRESH top-level session from host setting `subagent-model-selection-settings` (default disabled, empty list). Enabling it needs a config entry **plus a fresh session and an app reload**; existing sessions never gain it. `openrouter` is already registered in the desktop profile (`apiKeyEnv: OPENROUTER_API_KEY`), but pi-ai 1.4.0's catalogue has **no crouton row** (a `models:` list would REPLACE the route's 376 models). **I did not flip it** — it changes delegation for every new session. Say the word and I stage the exact entry for the next fresh session.
2. **Fleet lanes (roster):** all four roles + `roster.delegator-model` are `opencode-go/deepseek-v4.1-flash`; `rates` has **no openrouter key** and `modelFor` **fails closed on an unpriced route**, so `rates` must get `openrouter/stealth/crouton-alpha` at 0/0 too; the fleet profile's llm-pi-ai catalogue also needs the model. This is a **Tier B policy swap** (owner confirm) — exact commands in REPORT §6. **I changed no policy row.**
3. **No failover exists:** nothing in roster/llm-retry falls back to DeepSeek on a rate-limit; a static roster swap makes turns FAIL when the ~1,000/day free allowance bites. Your order says "DeepSeek takes over" — that needs either a failover feature or an operator swap policy. **Decision needed.**
4. **Fence:** fleet lane briefs carry order context (private/trading-adjacent). The owner's order says no sensitive work on the anonymous provider. So a blanket roster swap is a fence conflict — **decide the lane split** (e.g. a dedicated non-sensitive design/build lane on Crouton; FINISH/relocation/simbank stay DeepSeek). Also: the ~1,000/day cap is the owner's figure, unmeasured (`limit: null`, `is_free_tier: false`).

## Recommendation
Fastest safe move today: leave fleet rosters on DeepSeek, put a **designated non-sensitive worker lane** on `opencode/crouton-alpha-free` (zero config) or `openrouter/stealth/crouton-alpha` (now works), and keep the failover question for the next package. If you want the session-level allowlist enabled for the next fresh session instead, say so and I will stage it (one config entry, needs the app reload).

## Elsewhere (brief)
App reliability, simbank v3 runs, relocation sheet r2, B4.4 continuity repair and design P3 are all still running; the W9 `ext:rsi` flow shows tickets **100020/100021/100022 pending** with the secure-prompt crash noted earlier — not touched by me. No owner act taken; no GUI; no sudo; no live app/unit restart.

— delegator
