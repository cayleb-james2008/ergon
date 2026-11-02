# TEAM B — 0.3.1 NON-INSTALLED BUILD CANDIDATE (for Lead approval; not installed)

Built 2026-11-02 ~00:45–01:05Z by `fleet-lead` as the hedge against the dark strike seat. **Nothing live was
touched**: no install, no restart, no fleet-profile write, no credential value, no publication. `$0` spend. The
tarball is a scratch-worktree artifact for the Lead's go/no-go; strike still owns the release line, and this
candidate is mergeable into it.

## 1. Branch / commits / worktree

| | |
|---|---|
| Worktree | `/home/cayleb/workspace/harness-build/candidate-031` (new; never `strike-release`) |
| Branch | **`candidate/0.3.1`** in the shared clone `/home/cayleb/workspace/harness-build/dsh-fleet/.git` |
| Tip | **`aa7415eb0d60de8da85036e577e7cf03308b7ff8`** |
| Graph | `7d4eb95` (strike/0.3.0) → `0653f82` (rebased `bus` content) → `e98f87a` (F1–F4 fix round) → `70ee748` (0.3.1 patch set) → `6dc1348` (bus removed from release path) → **`aa7415e`** (Stage-4 touchpoints) |

`src/bus` is **deleted from this candidate's release path** (commit `6dc1348`: files removed from tree, `./bus`
export dropped, `checks/stage5-check.sh` + bus fixtures removed). Stage 5 keeps its own wiring round; this
candidate is roster-only.

## 2. Stage-4 touchpoints applied (commit `aa7415e`)

| # | Component | What landed | Test |
|---|---|---|---|
| 1 | fleet-core | `setLaneRoots(fn)` one provider slot (throws if taken; disposer clears only its own slot); `agentRoots(agent)` = selftest roots + lane roots; exported on `ctx.fleetCore` | core `plugin.test.mjs` "Stage-4 touchpoint 1" + service API member list |
| 2a | fleet-orders | `setVerdictGate(fn)` one slot; `recordVerdict` calls it before applying and refuses with its code; gate not installed ⇒ behaviour unchanged | orders `test/stage4-touchpoints.test.mjs` |
| 2b | fleet-orders | `invalidCheck(harness, task, …)`: `built`→`open`, no beginCheck, round unchanged, `rounds_allowed+1`, one harness verdict row + one `invalid-check` post, order not blocked; harness-only, built-only | same file |
| 2c | fleet-orders | harness post kinds `stall` + `invalid-check` accepted beside `budget-alert`; unknown kinds refused | same file |
| 2d | fleet-orders | `releaseClaims(harness, principal)` releases claimed/in_progress tasks to `open`, no attempt, no round, token dead | same file |
| 2e | fleet-orders views | **No edit needed at this base**: `getOrder` already carries `size/fixture/checkpoint_at/goal/context`; `getTask` carries `budget_usd`, `facts.claimed_at`, `built.{commit,artefact_hashes,evidence}`, `maker` (verified in `views.js`) | covered by orders suite |
| 3a | fleet-ledger | `record()` takes `priceProvider` (engine stored as `claude-code`/`codex`, priced under `anthropic/openai`); event view gains `source` + `session` | ledger `ledger.test.mjs` "touchpoint 3a/3c" |
| 3c | fleet-ledger | `spend-get` owner op (`real_usd/synthetic_usd/engine_usd/cap_usd/remaining_usd`, day split via `forDay()`) | same |
| 5 | bundle patch | `fleet-roster` row inserted after `fleet-orders` with the NOTES.md config (`checksRoot`, `gitRoot=/var/lib/fleet/gitwork`, `engineRoot=/var/lib/fleet/engines`, `credentialStore`, `sessionRoots`, `engineSearchRoots`); `npm run release` re-run → `rows.manifest.json` unchanged (**8 pinned / 12 disabled**; inserted bundle rows are not pinned) | manifest regen + rows/core tests |

**Named deviations / not done (honest):** (i) the `verdicts` table CHECK at this schema version is
`('checker','external')`, so the harness `invalid-check` row is stored with source `checker` and principal
`harness` — widening the CHECK needs a table-rebuild migration, named, not faked; (ii) Codex model id + its
`openai/<model>` rates row are still owed (Job 10 decision) — Codex stays unpriced ⇒ `unknown-model` fail-closed;
(iii) the day-cap reservation across concurrently launching engine lanes is still open (engines gate once at launch
via `ledger.gate`); (iv) `lessons-list` op (touchpoint 4) not added — the roster only needs `pending`; (v) the
`egress.allow` additions for the engine CLIs are a Tier-B policy act (owner), untouched; (vi) no live readback
could be taken (nothing installed).

## 3. Suites (clean tree, run 2026-11-02 ~01:00Z)

| Suite | Result |
|---|---|
| roster | **102/102 pass, 0 fail**; mutation suite **8/8 mutants killed** (freshness + lane cap; unmutated baseline passes) |
| core | **100/100** (incl. the new lane-roots test and rows-manifest identity) |
| orders | **200/200** (196 prior + 4 new touchpoint tests) |
| ledger | **13/13** (12 prior + new 3a/3c test) |
| policy | **65/65** |
| **total** | **480 pass, 0 fail** |

Commands: `cd package/src/<c> && TMPDIR=<disk dir> /usr/bin/node --import ../core/test/_register.mjs --test
test/*.test.mjs` (from `core`: `--import ./test/_register.mjs`); mutants: `cd package/src/roster && node test/mutate.mjs`.

## 4. Tarball (built the 0.3.0 way: `npm run release` = rows-manifest regen + `npm pack --pack-destination ../dist`)

| Fact | Value |
|---|---|
| Path | `/home/cayleb/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz` |
| sha256 | **`b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891`** (copy in `cayleb-dsh-fleet-0.3.1.tgz.sha256`, `SHA256SUMS.candidate`) |
| Size / members | 337,149 bytes; **123 members** (0.3.0: 104; +19 roster files; bus excluded) |
| Modes | **120×`-rw-r--r--` (0644) + 3×`-rwxr-xr-x` (0755)** — the 0.3.0 discipline ({0644,0755}); no other mode |
| Package version | 0.3.1; `package.json` sha256 `631eaeb18a257173765ed3fd7334d78fd213697bb52cbb219ef148386b87de81` (same bytes in tarball) |
| Rows manifest | `rows.manifest.json` sha256 **`1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`**, 8 pinned / 12 disabled |
| Acceptance script | `checks/stage2-check.sh` **unchanged** (`7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95`, = the 0.3.0/S1 script) |
| Stage-4 check | `checks/stage4-check.sh` present (sha256 `dca37c12…`) — **not runnable before the owner key**; not part of S4B |

## 5. Install command (NOT RUN — for Lead approval) and rollback

```bash
# 1. stage the drop (root/owner act; the conventional sidecar is required by stage2-check 2.1a)
install -m 0644 -o root -g root dist/cayleb-dsh-fleet-0.3.1.tgz        /var/lib/fleet-drop/
install -m 0644 -o root -g root dist/cayleb-dsh-fleet-0.3.1.tgz.sha256 /var/lib/fleet-drop/
# 2. stop, install through the fixed wrapper (derived-count / normalise path), start
systemctl --no-ask-password stop fleet-dsh            # polkit-permitted for cayleb on unit fleet-dsh
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin plugin-add \
  b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891 cayleb-dsh-fleet-0.3.1.tgz
systemctl --no-ask-password start fleet-dsh
# 3. identity readback
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin status            # version + integrity
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin dump-config       # rows, subagent 32, llm route, fleet-roster row
```
**Rollback** (no profile/data surgery needed; migrations are additive): stop → `plugin-add
8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070 cayleb-dsh-fleet-0.3.0.tgz` (the 0.3.0
tarball is already in `/var/lib/fleet-drop`) → start → status. The new roster tables/rows are additive; the 0.3.0
tree ignores them.

## 6. Expected installed facts for the fresh checker (S4B-equivalent acceptance, script UNMODIFIED)

Checker runs, one run at a time, fresh `OUT`, from a release tree (candidate worktree works):
`OUT=<fresh dir> FLEET_CLI=<candidate>/package/bin/dsh-fleet.mjs bash <candidate>/checks/stage2-check.sh`
→ expect `summary: pass=60 fail=0` and `STAGE 2: PASS`, `EXIT=0` (0.3.0 S4B precedent: 60/0/165 s; tarball 123
members now). Before/after it must read back:

| Fact | Expected for 0.3.1 |
|---|---|
| installed version | `@cayleb/dsh-fleet` **0.3.1**; installed-tree file count **123**, modes {0644,0755} |
| tarball | `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz` sha256 `b2ae2185…` **and** its `.sha256` sidecar present (leg 2.1a needs it) |
| rows | **8 pinned / 12 disabled**, failures `[]`, manifest sha of the installed file **`1970f995…`** (0.3.0 was 7/12, `1bcac986…` — 0.3.0 shipped without its own `fleet-mcp` insert row; the regenerated file is authoritative for 0.3.1) |
| unit / listeners | active, new `InvocationID`, MainPID; MCP listeners exactly `127.0.0.1:8789`, `[::1]:8789`, `100.114.17.67:8789` (mcp row unchanged; no catch-all) |
| MCP tools | the same nine: `fleet.status`, `session.create`, `session.prompt`, `session.status`, `board.op`, `order.drive_roundtrip`, `order.stage_real`, `fleet.selftest`, `actions.status` |
| bundle rows | `fleet-roster` mounted (new); `llm-pi-ai` shows `providers.opencode-go` = `{apiKeyEnv: OPENCODE_GO_API_KEY, displayName: …}` with **no key value**; `subagent` row config `maxActiveSubagents: 32` |
| policy | `rates` contains `opencode-go/deepseek-v4.1-flash` (0.15/0.6/0.003 cacheRead per Mtok); after the roster mounts, `roster.models` shows the doctrine table (lead sonnet-5-5, builder/researcher opencode-go/deepseek-v4.1-flash, checker haiku-5-5) and the roster ops (`lane-spawn`, `lane-get`, `providers-get`, …) answer |
| service health | `status` `ok:true`, `integrity.ok:true`, `lanes_allowed:true`, `selftest:true`; `session.create/prompt` stub turn still completes with transport `stub-turn` |

**The checker must NOT certify:** activation (needs the owner's `OPENCODE_GO_API_KEY` in the guided first-run
field — no credential value may be requested/printed), any real-provider turn (`session.create{mode:"model"}`
still refuses `model-credential-not-entered` until then), the Stage-4 gate (`stage4-check.sh`), engine runs
(`claude-code`/`codex` — binaries/keys absent), hands/bus, or lane work (`lane-spawn` may answer
`no-credential`/`unknown-model` until activation; that is expected, not a failure of S4B).

## 7. What remains for strike / Lead before install

1. Nox/Lead go for the release act (strike owns it; this candidate is the zero-delay artifact).
2. If strike's own 0.3.1 differs, merge `aa7415e`'s touchpoint diff (or take the whole branch), re-run the five
   suites + `npm run release`, and use **its** tarball sha for the install and checker.
3. Fresh checker over installed output (Lead spawns; maker never checker): the S4B-equivalent above + the
   "expected installed facts" table; it may not certify activation or real-provider turns.
4. Builder-side limits carried: verdicts CHECK deviation, Codex rates row, day-cap reservation API, egress
   allowlist (Tier B), no live run of any of this (nothing installed).
