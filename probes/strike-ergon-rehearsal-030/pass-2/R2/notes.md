# R2 — First personalised assistants (pass 2)

- Step: R2 — First personalised assistants
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator adds the first employees and onboards each through the product's onboarding conversation, then confirms the assistant draft; here the fixture answer set stands in for the employee's own answers, and the model-quality conversation leg cannot run (named PENDING)
- Expected: each cast member has a personalised agent profile and an onboarding session; the product's onboarding protocol reaches `active` with a confirmed operating model; a real fleet agent session exists for each cast member on the installed plugin.
- Actual: 8/8 hard checks held; 1 named pending; 2 recorded finding(s)
- Word: **pending**
- Evidence files: `R2/fixture-assistant-records.json`, `R2/onboarding-drive.json`, `R2/fleet-sessions.json`, `R2/stub-turn-probes.json`, `R2/model-leg.txt`
- Timestamp (UTC): 2026-11-01T23:08:13Z .. 2026-11-01T23:08:13Z
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
- dana-whitcomb → lead:selftest-lf-2-dana-whitcomb: real session fleet-selftest-lead-023042c7-5543-4626-9317-e42ea38098c2, call selftest-c308b723-921d-4f09-a9db-a3ed95a579a3, board scope mine
- priya-lindqvist → researcher:selftest-lf-2-priya-lindqvist: real session fleet-selftest-researcher-732f3915-ad91-4d2c-a78d-e3f84b58149a, call selftest-8e624999-e321-47db-b855-375ea576acc8, board scope mine
- marcus-oyelaran → builder:selftest-lf-2-marcus-oyelaran: real session fleet-selftest-builder-12d1afcd-67ee-4b43-bbea-4168fafafbc3, call selftest-3ad4a298-2b17-4e7b-bf53-ec5cdc2f517c, board scope mine
- teo-brandvold → builder:selftest-lf-2-teo-brandvold: real session fleet-selftest-builder-074d6b4c-26c7-4081-aa99-cdc675597014, call selftest-6eb900fe-35b2-4791-a1cd-7565ec64d4ea, board scope mine
- rosa-quintero-hale → researcher:selftest-lf-2-rosa-quintero-hale: real session fleet-selftest-researcher-35e515ea-b99e-428d-9460-4f625c4e0998, call selftest-fc2aa260-74e2-4a1a-9666-52d111c5a04f, board scope mine
- sam-akintola → checker:selftest-lf-2-sam-akintola: real session fleet-selftest-checker-6464cb86-042d-4b12-b7b0-2c0efcca52a3, call selftest-a8565bf2-7169-423b-93c0-8a796ba41a85, board scope mine

## Recorded metadata
```json
{
  "sessions": 6,
  "onboardingActive": 6,
  "stubProbes": 2
}
```
