# R2 — First personalised assistants (pass 1)

- Step: R2 — First personalised assistants
- Pass: 1
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator adds the first employees and onboards each through the product's onboarding conversation, then confirms the assistant draft; here the fixture answer set stands in for the employee's own answers, and the model-quality conversation leg cannot run (named PENDING)
- Expected: each cast member has a personalised agent profile and an onboarding session; the product's onboarding protocol reaches `active` with a confirmed operating model; a real fleet agent session exists for each cast member on the installed plugin.
- Actual: 8/8 hard checks held; 1 named pending; 2 recorded finding(s)
- Word: **pending**
- Evidence files: `R2/fixture-assistant-records.json`, `R2/onboarding-drive.json`, `R2/fleet-sessions.json`, `R2/stub-turn-probes.json`, `R2/model-leg.txt`
- Timestamp (UTC): 2026-11-01T23:13:38Z .. 2026-11-01T23:13:38Z
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
- dana-whitcomb → lead:selftest-lf-1-dana-whitcomb: real session fleet-selftest-lead-fe3c2887-4749-4c69-8155-775df46c6874, call selftest-a7250550-b7ac-4c9c-a881-a00087d9e882, board scope mine
- priya-lindqvist → researcher:selftest-lf-1-priya-lindqvist: real session fleet-selftest-researcher-d5b2dcca-3cce-4f94-9731-6b51288c250f, call selftest-5d1fede3-25cd-4b1d-9d7d-84c5caaa0fcc, board scope mine
- marcus-oyelaran → builder:selftest-lf-1-marcus-oyelaran: real session fleet-selftest-builder-480292ef-441c-4d55-aa56-a7d6458c8c9d, call selftest-b888001b-6251-4c4d-af55-c78fad658365, board scope mine
- teo-brandvold → builder:selftest-lf-1-teo-brandvold: real session fleet-selftest-builder-06e8f435-4c8b-459c-833b-5a207c301188, call selftest-0746dd25-a7cb-47c6-9b63-0229989f25bd, board scope mine
- rosa-quintero-hale → researcher:selftest-lf-1-rosa-quintero-hale: real session fleet-selftest-researcher-528dd9d2-388e-468e-88bc-24c4ffac9384, call selftest-afa058ae-39c2-479b-b77b-65145d2f7138, board scope mine
- sam-akintola → checker:selftest-lf-1-sam-akintola: real session fleet-selftest-checker-4d305a8e-8f56-4ccf-8194-0f0171c54fd6, call selftest-87a19538-a9fc-4876-b2fb-ce965a3f75e8, board scope mine

## Recorded metadata
```json
{
  "sessions": 6,
  "onboardingActive": 6,
  "stubProbes": 2
}
```
