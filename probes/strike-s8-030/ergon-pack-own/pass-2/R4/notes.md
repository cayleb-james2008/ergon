# R4 — Phone enrolment (pass 2)

- Step: R4 — Phone enrolment
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator opens the same link on the phone, taps "Add this phone", and the manager approves; then tests the assistant with the desktop off
- Expected: the phone is listed as an approved device and the assistant answers from the phone while the desktop is off
- Actual: 3/3 hard checks held; 1 named pending; 2 recorded finding(s)
- Word: **pending**
- Evidence files: `R4/device-registry-standin.json`
- Timestamp (UTC): 2026-11-01T23:13:42Z .. 2026-11-01T23:13:42Z
- Blocker / finding: phone surface (PACKAGE-PENDING: no phone client; a phone cannot use the fleet join — SCENARIOS.md F5 / package boundary item 6): "Add this phone" and the manager approval on the phone cannot be shown, and the desktop-off answer test needs the host agent runtime/model credential. Host-side device registry row for the phone recorded as a stand-in (operator-registered, approved).

## Checks
- PASS a phone device row exists in the host-side device registry — aliases=crew-tablet-1,crew-tablet-2,dana-desktop,front-desk,marcus-desktop,north-kiosk,priya-desktop,sales-laptop,sam-desktop,teo-phone
- PASS phone device is operator-registered and approved — enrolment=operator-registry status=approved employee=teo-brandvold
- PASS phone device belongs to the phone-first cast member — employeeId=teo-brandvold

## Findings recorded (not a failure of this step)
- device registry — Host-side fleet registry: 9 devices, 8 approved, 1 pending (the sales laptop is left waiting by design — a Monday act for the support desk).
- desktop-off answer test — Scenario (e) — assistant answers while the desktop is off — needs the host agent runtime and a model; it is covered by the R2 model-leg PENDING, not claimed here.

## Named PENDING (never recorded clean)
- phone surface (PACKAGE-PENDING: no phone client; a phone cannot use the fleet join — SCENARIOS.md F5 / package boundary item 6): "Add this phone" and the manager approval on the phone cannot be shown, and the desktop-off answer test needs the host agent runtime/model credential. Host-side device registry row for the phone recorded as a stand-in (operator-registered, approved).

## Recorded metadata
```json
{
  "phoneAlias": "teo-phone",
  "phoneEmployee": "teo-brandvold"
}
```
