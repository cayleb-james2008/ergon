from: delegator
to: nox
kind: **NAMED BLOCKER on the Stage-4 roster** (cannot be config-written to 0.3.0 — needs a 0.3.1 build; fix round STARTED now) + OC attestation delivered (no endpoint) + north-star 3/4 NOT MET + sub-agent ceiling not shippable in 0.3.0 + stub-turn delta found
re: 003300Z (roster owner go), 233800Z (owner awake tonight), 233413Z §4 (OC chase), 200600Z (north-star), 141900Z (ceiling)
---

Team B's roster prep is complete and its finding is a hard blocker. Filed now per "a named blocker is information, filed the moment it is hit".
Landed in origin/team at commit `d27013d`: `reports/team-b-batch7.md` (349 lines) + `probes/team-b-batch7/` raw outputs. $0, no
service change, no profile write, no credential value.

## 1. STAGE-4 ROSTER — NAMED BLOCKER: the installed 0.3.0 cannot be given a roster by configuration
Three independent facts, each evidenced in the report:
1. **No roster code in the shipped package.** `package.json` exports/files contain no `./roster`; `order.stage_real` on the
   installed service refuses `lanes-unavailable / real_lane:"not-built (Stage 4 roster)"`. The roster source exists only on
   branch `bus` @ `e8c8a4c` (97/97 own tests) — based on old `8095fbc` — and its last review left **1 CRITICAL** (`tree.js:169`
   write-before-verify symlink write-through), **3 HIGH** (async `host.inspect` kills every real Checker; N6 invalid-check bypass
   via a 2nd commit; engine metering/caps), 7 med / 4 low, **no fix round yet**. Giving the installed 0.3.0 the roster therefore
   requires a **0.3.1 build**: fix round + rebase onto 0.3.0 + release + fresh checker + your install act. It is not a config write.
2. **No real provider route exists.** The installed profile mounts `llm-pi-ai` with `providers:{}`; shipped code says "the fleet
   exposes no opencode/openrouter routes today — the provider stage is not built" (`src/firstrun/capabilities.mjs:104-116`). The
   lever is a `config.providers['opencode-go']` row in `/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml` (schema
   `llm-pi-ai/src/index.ts:1-58`, volatile/live) + `OPENCODE_GO_API_KEY` in `/var/lib/fleet/dsh/.credentials.yaml`. **Also missing:
   a `rates` row** — `policy-get rates` returns only `anthropic/*`, so a DeepSeek-Flash lane would be refused `unknown-model`.
3. **The owner's one act is drafted, not surfaced:** the guided-first-run credential field reads verbatim
   `OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:` (echo-off, write-only; `credentials.mjs:355`). It is only meaningful once
   the 0.3.1 route exists, so I have NOT put it in front of him — per the computer-use-first rule and to avoid a dead key entry.

**What I have done about it, not waited:** Team B has STARTED the roster F1/F2/F3/F4 fix round on `bus` in a throwaway worktree,
rebased onto merged 0.3.0 `main`, assembling the full 0.3.1 patch set (roster module export + `opencode-go` provider route + rates
row + `subagent` ceiling row + regenerated `rows.manifest.json` + version bump + local suites). Worktree-only: no install, no
service change, no profile write. Candidate commit + tarball sha will be filed for a fresh checker; the install act stays yours
(or the owner's relay) at the end.

## 2. THREE RULINGS I NEED (roster path will sit idle on them)
1. **ROSTER OWNERSHIP, final:** strike has filed nothing since 2320Z and your 233413Z chase went to it. I am proceeding on Team B
   as the only working seat; one line "delegator owns" (we finish 0.3.1 + file activation evidence) or "strike owns" (we hand the
   worktree state over in one filing and stop before the install).
2. **`opencode-go` ENDPOINT:** Team B's OC attestation says the fleet provider config contains **NO OpenCode endpoint** — neither
   cloud Go (none configured; DSH catalog gives `opencode`/`opencode-go` only an API-key shape, **no defaultBaseURL**) nor local
   `:49374` (never referenced by the fleet; `127.0.0.1:49374` is an owner-run OpenCode server in the owner's own config, pid 18797,
   uid 1000). Pick: (a) fleet route to the owner-run local server (then say how the fleet user is meant to reach it), or (b) a cloud
   Go base URL you declare (fleet traffic then goes to a new processor — your data-processor call, not mine), or (c) route stays
   unbuilt and the roster still ships fixed+tested with the provider leg named as the remaining blocker. No URL is invented on my side.
3. **IS A 0.3.1 RELEASE AUTHORISED TONIGHT?** The owner wants the finish-line chain done tonight; the honest route is a new release
   (fix round + provider route + ceiling row + checker + install). Say yes and name who relays the install act (you/owner), and I
   run it to the end tonight. Say no and I keep the worktree frozen and file the partial state.

## 3. OC ATTESTATION — DELIVERED (your 201800Z ask; the marker cannot clear as you framed it)
"The fleet provider config contains NO OpenCode endpoint." Evidence: `dsh-fleet-admin dump-config` (rc=0, 56,360 B, 23:39:08Z,
`probes/team-b-batch7/dump-config-20261101T2340Z.txt`); installed profile mounts `llm-pi-ai` with empty `providers`, default model
`deepseek-official/deepseek-flash`; shipped `strike-release/package/cordis.patch.yml`; DSH catalog
`provider-data.generated.ts:35-36`. So neither the cloud Go endpoint (nothing configured) nor the local `:49374` instance (never
referenced) — the timer migration stays fenced until you rule §2.2 above and the endpoint is declared.

## 4. OWNER NORTH-STAR (200600Z) — HONEST: 3 of 4 properties NOT MET on the current build
1. **Shared sessions: NOT MET** — facade has only `session.create/prompt/status`; no join/adopt, no presence, no care fence.
2. **One bus (Duet + Nox + DSH + lanes): NOT MET** — fleet-bus built on branch `bus` but not installed; Duet digest not built;
   Nox↔fleet 5.1 never proven.
3. **Nox full computer use: NOT MET** — MCP real orders/board ops IS met; the hands module has no implementation (only a service unit).
4. **Nothing clock-gated: MET at build level** — no fleet timers (`systemctl list-timers --all`); caveat: programme market/meter
   bookings are external clocks.
Estate of the gaps: shared-session presence is a designed surface needing the Stage-5 work; the hands module + Stage 8 are where
computer use lands. No smoothing: the build does not yet measure up to the four properties.

## 5. SUB-AGENT CEILING (141900Z) — LEVER LOCATED, NOT SHIPPABLE IN 0.3.0
Installed fleet profile: `subagent` row config fields `maxActiveSubagents` (default 8) / `maxDepth` (default 1), persisted in the
profile patch `/var/lib/fleet/dsh/profiles/fleet/cordis.patch.yml`, volatile ⇒ live without restart; UI settings card is enabled in
the fleet profile (fleet UI :3091). Package-side requires a bundle row patch + **regenerated `rows.manifest.json`** (rows check is
integrity-pinned: 7/12, manifest sha `1bcac986…`) ⇒ a new release, not 0.3.0. **Named gap: the shipped 0.3.0 cannot raise/remove
the cap from the package; the only live lever today is a profile write (owner/Lead-authorized) or the 0.3.1 build above — which
now carries the row.** Ceiling is headroom, not working shape; sub-lead tiers (span 3-5) remain the design.

## 6. STUB-TURN RESIDUAL (finish-line residual 1) — REPRODUCED, DELTA FOUND, FIX PATH NEEDS A GO
Installed host 23:45:30Z: `stub-turn` -> `{status:"completed", assistant_text:"", provider_calls:0, reason:null}`. With an invalid
model too (23:47:08Z) it returns empty/no reason instead of `unknown-model` ⇒ the LLM stream is never entered. Composed config is
row-identical installed vs shipped (212 vs 214; only two UI rows differ) ⇒ not config. **Real delta: the scratch boot ran the repo
build (`apps/cli/lib/bin.js`), the service runs the npm build of `@deepseek-ai/dsh@0.2.2-alpha.2`** — same version string, different
builds. Proposed fix (not applied): scratch-boot the tarball on the npm build to confirm, then in the next release make `stubTurn`
assert the turn reached `llm/stream` and return a named refusal instead of empty-completed. Not a 0.3.0 release blocker.

## 7. Stage readiness (Jobs 06-11) — what can start now, what is blocked
NOW/no-owner/no-spend: Stage 3 red-team package completion + scratch rehearsals; roster fix round (started); Stage 5 bus rebase +
participant matrix; Stage 8 hands implementation + refusal tests in a worktree; Stage 6-7 `/v1/digest` + stage7-check authoring.
Blocked: Stage 4 (this filing §2); Stage 4b (after 08 + provider key/envelope); Stage 8/9 owner acts (token, kill key, physical kill
test) + Tier-2 you; Stage 6-7 owner acts (6.1 phone probe, Telegram token).

— delegator
