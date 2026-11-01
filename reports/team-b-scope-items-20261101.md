# TEAM B — adopted rulings → scope items (per stage: pattern, gap, where it lands)

Filed 2026-11-01T15:03Z by `fleet-lead`. Consolidates the morning's rulings that change work already queued
(Nox `20261101T132410Z`, `132614Z`, the mandate/effects/router ruling relayed via the Lead, and the provider
set). **Design adopted, code NOT imported** — nothing from any prototype is vendored, run, or copied; each
pattern names the gap it fills and lands in an existing stage build.

| Ruling | Pattern | Gap it fills | Where it lands (stage/lane) | Status |
|---|---|---|---|---|
| Mandate kernel, **observe mode default** | authorisation layer for self-initiated fleet work: records what it *would* accept, does nothing, until rules promote it | today lanes act under granted calls but there is no durable model of "what is this lane mandated to initiate next"; observe-first closes the autonomy gap without widening authority | **Stage 5 steering build** (model-agnostic slots + meters already there); feeds Stage 8/9 self-initiated work | scope item, design only; no prototype code |
| **Write-first Effects/Outbox** for orders | effects recorded BEFORE execution; retry with backoff; restart-recovery notes | this weekend's mid-run lane deaths lost work in flight (delegator context death; lanes retired mid-order). A write-first record + outbox makes an interrupted order resumable instead of re-derived | **Lane 2's order/board build items** (candidate repair: hands race fix + bus package assembly) and **Stage 9 cold-start/restore**; the board schema change is additive | scope item — the highest operational priority of the three |
| **Router/Worker session topology** | the plugin's session shape: a lead session routes, worker sessions execute; lead+crew, recursive, span 3–5 | current fleet lanes are flat slots; Cayleb's lead+crew shape has no native expression | **Stage 4 roster / Stage 5 steering** (fleet-roster lanes → router/worker), panel/session surfaces in Lane 4 | scope item, ratifies the existing roster direction |
| **Our divergence: mandate strength revalidated at resume/fan-out** | any later message resuming work under an old mandate is refused; mandate is re-checked whenever a worker resumes or fans out | stale mandates outliving their grant window are a real authority leak; this is our own addition, not the prototype's | **Stage 8 hands + Stage 9 mission proof** rails; ties to the existing single-use / 10-minute / revalidate-at-resume rules | scope item (deliberate divergence, documented) |
| **MCP facade amendment** | facade must carry (i) Nox session-start via MCP and (ii) full bus + hands access over the same MCP connection | the finish line ("Nox delegates a real work order remotely") cannot pass on a facade that lacks these; the gate would be wrong to pass without them | **Lane 4 (Stage 5 MCP scope widened — spec being written)**; Stage 8 owner owns hands-over-MCP exposure with same rails/kill switch | scope widened; not built |
| **Crew depth 3–5, recursive** | every lead may stand up its own crew as the standard orchestration shape | single-session lanes cap throughput and context; depth is the harness's shipped shape | orchestration policy for all lanes; this Lead's crews are spawned by the delegator seat (session child-limit), not by team leads | ruling noted; no code |
| **Provider set (minus Codex)** | five routes: OpenCode Go live; DeepSeek direct / OpenRouter / xAI Grok / Cursor-bridge prepared up to the key field | the plugin has no `src/engines/` at all; the set defines the build's provider matrix and meter story | **Lane 3 provider scope** (`team-b-lane3-provider-scope.md`); Stage 4b build | matrix filed; Grok shows the named subscription≠API-key gap; Cursor bridge flagged unofficial/off-by-default |
| **Prototype access model** | **REJECTED** | workers running danger-full-access as the operator would break fleet containment | nowhere — hard fence: separate fleet user, sandbox, Tier B remain the access model | fence recorded |

## Sequencing notes

- Effects/Outbox is the item with operational teeth (lane deaths this weekend); it lands with Lane 2's
  build items and must not delay Lane 1's proof chain.
- The MCP widening is a **scope change to Stage 5's existing gate**, not a new stage: the Stage 5 gate must not
  be able to pass without session-start + bus/hands over the facade.
- Mandate kernel / router-worker / revalidation are Stage 5/8/9 **build items**; they are recorded here so the
  builds pull them and no lane re-derives them.
- No prototype code, no import, no vendoring, no run. Design language only.

## ADDENDUM 2026-11-01T17:10Z — adopted component map (Nox) + memory replacement order

**Re-derived component map (working map, Nox order).** Subscription-routing adoption is **DEAD in full** — API
keys only; the subscription/key gap is served by DSH's built-in provider machinery (no custom subscription
router); the **Claude-engine provider stays custom**. Messaging narrows to **`dsh-notifier`** (Telegram leg
only). **MCP facade** swaps to **`dsh-web-mcp` under a heavy wrap** (bus auth + tailnet, pinned, full source
review) — the only candidate with session-create semantics; this feeds the Stage 5 MCP widening contract
(`board/reports/team-b-lane4-mcp-facade-contract.md`). **Hands consolidate**: desktop core custom; Windows/Android
legs via the VM-panel adoptions (Lane 4 rig). **Sub-agent ceiling** is treated as a packaged-config/defaults
lever; Team B's on-box check of the pinned build closes it either way and, if hard-coded, it is filed as a named
host gap. **Crash isolation is a host property** (DSH runs plugins in-process, no sandbox); no component fixes it —
mitigations are containment, staged admission, rehearsed disable; the crash evidence pack is the upstream
escalation. This is repeated in every stage report.

**MEMORY REPLACEMENT (Lane 3/4 scope, planning only — no admission work until the Lead confirms the fleet-profile
staging shape).**
- Crashing component (record): **`@furongjun1999/dsh-memory` 0.9.21** — host-stopped crash logs; **not**
  `sage-mem`. The memory component is an **OPEN GAP in the single package** until the replacement lands; each
  stage report must name it as such.
- **ADOPT + WRAP `sage-mem`** (pinned npm version, Apache-2.0, DSH-native, zero-service, file protocol: four
  memory types / MEMORY.md index / anti-bloat write rules / line-cited JIT recall with bounded injection /
  consolidation with rollback). **Pinned version only; minimal surface; staged in a FLEET profile, never the
  owner's personal profile.**
- **ADMISSION SOAK GATE:** 50+ cycles, fresh checker, **one-strike rollback** (any crash or host instability
  attributable during soak removes it and the runner-up becomes the build). Platform fact stated in stage
  reporting: plugins run in-process, so no component is crash-proof by selection; soak + rollback is the mitigation.
- **Data:** the uninstalled plugin's store is archived untouched at
  `~/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/`; after sage-mem passes its
  gates, content migrates **one-way** into typed markdown with checker-verified counts + sample read-back; the
  archive is retained until Cayleb personally says discard.
- **Runner-up only if sage-mem fails soak:** a small custom Cordis component implementing Anthropic's memory-tool
  pattern over the existing file store.
