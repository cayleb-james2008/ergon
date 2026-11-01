# R2 — First personalised assistants (pass 2)

- Step: R2 — First personalised assistants
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator adds the first employees and onboards each through the product's onboarding conversation, then confirms the assistant draft; here the fixture answer set stands in for the employee's own answers, and the model-quality conversation leg cannot run (named PENDING)
- Expected: each cast member has a personalised agent profile and an onboarding session; the product's onboarding protocol reaches `active` with a confirmed operating model; a real fleet agent session exists for each cast member on the installed plugin.
- Actual: 8/8 hard checks held; 1 named pending; 2 recorded finding(s)
- Word: **pending**
- Evidence files: `R2/fixture-assistant-records.json`, `R2/onboarding-drive.json`, `R2/fleet-sessions.json`, `R2/stub-turn-probes.json`, `R2/model-leg.txt`
- Timestamp (UTC): 2026-11-01T22:31:50Z .. 2026-11-01T22:31:50Z
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
- dana-whitcomb → lead:selftest-lf-2-dana-whitcomb: real session fleet-selftest-lead-b2b4283e-33e4-4357-8ae6-3538141b9d92, call selftest-111d0858-b2f8-42ca-aacd-972e4b79452f, board scope mine
- priya-lindqvist → researcher:selftest-lf-2-priya-lindqvist: real session fleet-selftest-researcher-daec618d-1375-457f-af8f-3efd6cb7f592, call selftest-a8d71064-cdf6-4a0b-a029-a2b07a83dfaa, board scope mine
- marcus-oyelaran → builder:selftest-lf-2-marcus-oyelaran: real session fleet-selftest-builder-b2f3eaf1-8270-484f-bbb0-dfaa96671556, call selftest-b7de63cb-9bda-49ae-977c-aa00a0946df0, board scope mine
- teo-brandvold → builder:selftest-lf-2-teo-brandvold: real session fleet-selftest-builder-0b4a26f5-ba07-49f7-9ad0-a9cc541e6292, call selftest-923a6d97-c888-478c-959c-520789391a96, board scope mine
- rosa-quintero-hale → researcher:selftest-lf-2-rosa-quintero-hale: real session fleet-selftest-researcher-9cd20cd2-2444-43d6-a1fa-a35c0f17b506, call selftest-62f541b5-d162-403a-a9fc-18e7c7e0f9de, board scope mine
- sam-akintola → checker:selftest-lf-2-sam-akintola: real session fleet-selftest-checker-e83b8bd4-aecd-48f5-af84-629d718d2bac, call selftest-cbe5fecb-e1a9-4df6-8a12-76d0e3fdb2af, board scope mine

## Recorded metadata
```json
{
  "sessions": 6,
  "onboardingActive": 6,
  "stubProbes": 2
}
```
