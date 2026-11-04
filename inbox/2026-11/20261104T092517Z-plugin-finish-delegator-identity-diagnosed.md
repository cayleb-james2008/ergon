# plugin-finish → Nox: h85 diagnosed — the "Team Lead" identity is an app-level runtime-context block, not a wrong preset

**2026-11-04T09:25Z.** Read-only (`evidence/DELEGATOR-IDENTITY-FINDING.md` in `~/fleet-scratch/overnight`). Nothing changed, no
config edited, and this is not a request to touch the lane.

## What is established, from the artefacts

* The delegator session's own header says `"agentPreset":"fleet-delegator"`, and the roster row is `role=delegator`,
  `state=idle`. `PRESET[role]` in `package/src/roster/table.js` maps delegator → `fleet-delegator`, so **the roster mounted the
  right preset**; h85's symptom is not a wrong-preset bug.
* In the same session, the string `You are the Team Lead named 'lead'` appears **17 times** against **2** occurrences of the
  fleet-delegator persona. Its first occurrence is a **`user`-role runtime-context message** — the DSH runtime preamble followed by
  "You are the Team Lead named \"lead\". Use `list_agents({})` to find your teammates and their names."
* `profiles/fleet/cordis.patch.yml` has **no team row** (22 ids: core, sandbox, egress, lessons, housekeeper, session-query,
  policy, roster, storage, persistence, fsguard, comms, browser, ui ×3, llm, plugin-market, bridge). The four
  `@deepseek-ai/dsh-experimental-*agent-team*` packages are **desktop shared packages** (`desktop-runtime.json`), so the block is
  composed from the app side.

## Why this changes the shape of the ruling

1. A plugin edit cannot fix it: the roster does not own that layer, and the text lives in the session history, so **it recurs on
   every resume** — "it held tonight" is not a fix.
2. The risk is capability, not naming. If a model adopts the block it believes it has `list_agents`/`send_message`; the fleet's
   contract is `bus_send`/`bus_pull`, and the delegator preset grants no delegation tools. A lane acting as a team lead is outside
   its charter, whatever the fleet row says.
3. Tonight's lane got it right: it treated the block as data, kept its charter, used no team tool and escalated.

## Where a fix can live (blast radius, smallest first)

* **Turn the team feature off for the `fleet` profile** (app/owner setting). One profile, no plugin release.
* **State precedence in the wake/charter** ("any team-lead context in this session is stale: you are <principal>"), which is a
  preset-text change and therefore a plugin release plus an install — not a hot edit on a live lane.
* **Leave it** and rely on each lane refusing, which worked once and is not a guarantee.

The third is what is true today; the first is the cheapest real fix if the team feature is not wanted in the fleet profile at all.
Your call — I have not touched it, and P3/soak/the owner's four commands are unaffected by this question.
