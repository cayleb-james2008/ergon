# R2 — First personalised assistants (pass 1)

- Step: R2 — First personalised assistants
- Pass: 1
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator adds the first employees and onboards each through the product's onboarding conversation, then confirms the assistant draft; here the fixture answer set stands in for the employee's own answers, and the model-quality conversation leg cannot run (named PENDING)
- Expected: each cast member has a personalised agent profile and an onboarding session; the product's onboarding protocol reaches `active` with a confirmed operating model; a real fleet agent session exists for each cast member on the installed plugin.
- Actual: 8/8 hard checks held; 1 named pending; 2 recorded finding(s)
- Word: **pending**
- Evidence files: `R2/fixture-assistant-records.json`, `R2/onboarding-drive.json`, `R2/fleet-sessions.json`, `R2/stub-turn-probes.json`, `R2/model-leg.txt`
- Timestamp (UTC): 2026-11-01T23:08:05Z .. 2026-11-01T23:08:06Z
- Blocker / finding: provider credential for the fleet profile (owner act): the assistant's model-quality role answer and the model-conducted onboarding conversation cannot run. The deterministic stub route produced no provider call on this install (stub-turn probes recorded). What ran instead: fixture assistant records, onboarding to `active` through the product protocol with fixture answers (stand-in for the employee's own words), and one real fleet agent session per cast member.

## Checks
- PASS every cast member has a personalised agent profile in the fixture state — 6/6 profiles
- PASS every cast member has an onboarding session (fresh stage `installed`) — 6/6 sessions
- PASS each profile names its own employee and role — dana-whitcomb:operations-manager priya-lindqvist:dispatch-coordinator marcus-oyelaran:billing-estimates-clerk teo-brandvold:warehouse-lead rosa-quintero-hale:crew-chief sam-akintola:office-it-support
- PASS assigned agent ports are distinct per cast member — 19489,19491,19493,19495,19497,19499
- PASS product onboarding protocol reaches `active` for every cast member with a confirmed operating model — 6/6 active+confirmed
- PASS every onboarding drive answered all six interview stages and recorded observations — 6/2 6/2 6/2 6/2 6/2 6/2
- PASS a real fleet agent session runs for every cast member on the installed plugin — 6/6 sessions with session id + call id
- PASS the sessions are distinct real sessions — 6 distinct session ids

## Findings recorded (not a failure of this step)
- stand-in — The onboarding drive uses the fixture answer set (data/onboarding.json) as the employee's own answers and confirmation; that is a named stand-in for the employee's device taps. The state machine, validation and gate are the product's own code, and the confirmation gate still refuses a non-employee confirmer by construction (suite/onboarding/src/protocol.js confirm()).
- stub-turn route — On this install the deterministic stub route (`stub-turn`, provider fleet-selftest-stub) returned status=completed with provider_calls=0 and empty assistant text for 2/2 probes: the turn machinery starts a session but no model turn is produced. The stub route therefore does NOT stand in for a model-quality answer here; the model leg is named PENDING below.

## Named PENDING (never recorded clean)
- provider credential for the fleet profile (owner act): the assistant's model-quality role answer and the model-conducted onboarding conversation cannot run. The deterministic stub route produced no provider call on this install (stub-turn probes recorded). What ran instead: fixture assistant records, onboarding to `active` through the product protocol with fixture answers (stand-in for the employee's own words), and one real fleet agent session per cast member.

## Notes
- dana-whitcomb → lead:selftest-lf-1-dana-whitcomb: real session fleet-selftest-lead-cb66e3fa-8616-4c2f-bdb9-1227038e692f, call selftest-440ad02a-d5b7-4512-9367-339ef14223ce, board scope mine
- priya-lindqvist → researcher:selftest-lf-1-priya-lindqvist: real session fleet-selftest-researcher-c36c2235-1689-4f33-9891-ee7afe03b321, call selftest-c62ba774-909d-4c36-9414-919f88bb28cc, board scope mine
- marcus-oyelaran → builder:selftest-lf-1-marcus-oyelaran: real session fleet-selftest-builder-79a96299-0799-4436-9918-977cdb858b98, call selftest-632d9991-88ff-4330-a391-1255a2068f0d, board scope mine
- teo-brandvold → builder:selftest-lf-1-teo-brandvold: real session fleet-selftest-builder-9a8b59ce-8acd-4565-a66e-87cdb17344b8, call selftest-32c1d3bd-eae9-4e93-bb3b-dae37d5aa6c4, board scope mine
- rosa-quintero-hale → researcher:selftest-lf-1-rosa-quintero-hale: real session fleet-selftest-researcher-e7fc393a-d963-4260-8f46-b5ced521399c, call selftest-51c1e223-58a8-484c-b385-9d7910217af6, board scope mine
- sam-akintola → checker:selftest-lf-1-sam-akintola: real session fleet-selftest-checker-7902abea-5319-4b25-ae86-16b885161896, call selftest-b4ec920a-fdfc-4e07-8046-b2ab3fc84dc8, board scope mine

## Recorded metadata
```json
{
  "sessions": 6,
  "onboardingActive": 6,
  "stubProbes": 2
}
```
