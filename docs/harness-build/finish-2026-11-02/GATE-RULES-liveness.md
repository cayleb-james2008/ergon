# Gate rule: a listed tool is not a working tool (Nox ruling 2 Nov 14:23 ET)

**Rule.** A service is LIVE only when a checker has driven it end to end on a real host shape. A tool that is listed, declared, counted, exported, or exercised only through fakes is NOT live. Fakes never discharge a liveness claim.

**Why.** 2 Nov: the fleet message bus passed every 0.3.4 gate (15 tools listed) and was not mounted; the first real call (`bus.who`) answered `bus-unavailable`. A false "live" stood for about 3.5 hours. Lesson filed: `harness-squared/library/coding/entries/2026-11-02-listed-tool-is-not-a-working-tool`; record: `LESSON-liveness-claims.md`.

**What the rule requires, and where it is enforced**
| where | enforcement | owner | state |
|---|---|---|---|
| install scripts (`release/install-0.3.5.sh` and every later one) | after start, a post-start self-check CALLS the service through the real facade and refuses to report success on `unavailable` (token read in-process, never printed) | executing session | in the 0.3.5 build brief; the checker must prove it |
| checker briefs for any service or tool | the checker must boot a host with the real config shape (a scratch DSH home), call the service end to end, and try to make the proof fail (boot without the row, flip a manifest byte); a green fake-based suite is recorded as "unit evidence", never as liveness | executing session (workflow checker prompts); Team B/Lane 1 (fleet checker briefs) | applied to the 0.3.5 checker; to be added to the delegator-side checker briefs when it wakes |
| acceptance and health gates (H1-H10, A1-A11 style) | a gate for a service names the CALL it makes, not the tool it lists | executing session | H4 must call `bus.status` from 0.3.5 on |
| reports | a liveness claim cites the call and its output, never the tool list | everyone | in force |
| lanes' own lesson library | the plugin's lessons store has no owner import op today (`importHarness2` is in-process only), so the entry cannot be loaded into lanes yet | Team B | named gap, in the 0.3.5/0.3.6 list |
