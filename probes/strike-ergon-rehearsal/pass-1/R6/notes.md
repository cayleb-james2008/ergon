# R6 — Voice / message flow (pass 1)

- Step: R6 — Voice / message flow
- Pass: 1
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator uses the voice button on a device to dictate a message; the assistant answers; the voice-control badge appears after the optional in-app setup step
- Expected: dictation -> answer round trip, and the voice-control badge reads "Voice control activated" while typing still works
- Actual: 2/2 hard checks held; 1 named pending; 1 recorded finding(s)
- Word: **pending**
- Evidence files: `R6/check-voices.txt`, `R6/installed-surface-rows.json`
- Timestamp (UTC): 2026-11-01T22:07:35Z .. 2026-11-01T22:07:35Z
- Blocker / finding: voice/message flow — voice surface PACKAGE-PENDING (P20-F5): no voice button, dictation client, transcript round trip or voice-control badge exists on the installed plugin; the Windows-only surfaces are stand-in/pending, never clean. Only the fixture's written-voices check ran (labelled: not the voice surface).

## Checks
- PASS supporting fixture check: the cast's written voices separate blind (not the voice surface) — ALL VOICES PASS
- PASS installed plugin bundle mounts no voice or phone surface row — bundle rows=75, voice/phone rows=0

## Findings recorded (not a failure of this step)
- fixture week voice acts — The simulated week's voice acts (fixtures/week/*.json) are fixture-side stand-ins for the voice surface; they are not the surface and are not counted here.

## Named PENDING (never recorded clean)
- voice/message flow — voice surface PACKAGE-PENDING (P20-F5): no voice button, dictation client, transcript round trip or voice-control badge exists on the installed plugin; the Windows-only surfaces are stand-in/pending, never clean. Only the fixture's written-voices check ran (labelled: not the voice surface).

## Recorded metadata
```json
{
  "fleetRows": 75,
  "surfaceRows": 0
}
```
