# R3 — Shareable non-localhost URL from a second device (pass 2)

- Step: R3 — Shareable non-localhost URL from a second device
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator opens the shareable non-localhost URL from a second device and signs in through the device approval flow
- Expected: the same company and assistant are visible from a second device through a real DNS name (never localhost, never a raw IP); the device approval record exists
- Actual: 5/5 hard checks held; 1 named pending; 1 recorded finding(s)
- Word: **pending**
- Evidence files: `R3/url-check.txt`, `R3/listener.json`
- Timestamp (UTC): 2026-11-01T23:08:13Z .. 2026-11-01T23:08:13Z
- Blocker / finding: hosted shareable non-localhost URL + DNS name + second-device reachability: not built in this strike. The fixture origin (https://lf-host-01.example-tailnet.ts.net:8443) is a made-up name that never resolves, the current host binds 127.0.0.1 only, and a dedicated host/DNS name are owner acts out of scope (no purchase, no DNS work). Device-approval via a second device therefore cannot be shown; the host-side device registry is covered as a stand-in under R4.

## Checks
- PASS company host origin recorded from the fixture config — origin=https://lf-host-01.example-tailnet.ts.net:8443 port=0 (0 = ephemeral loopback at run time)
- PASS origin is a DNS name, not localhost and not a raw IP — hostname=lf-host-01.example-tailnet.ts.net scheme=https: port=8443
- PASS origin is recorded as a made-up name that is never resolved (fixture note) — The Anthropic budget proxy's activation gate accepts only an https://*.ts.net:8443 origin (support/server/anthropic-budget-proxy.mjs validOrigin). The name is made up and never resolved; the rehearsal
- PASS the shareable name does not resolve (hosted URL/DNS absent) — getent exit=2 stdout=(none)
- PASS current host code listens on 127.0.0.1 only (no shareable listener) — address=127.0.0.1 port=36023 base=http://127.0.0.1:36023

## Findings recorded (not a failure of this step)
- device approval flow — The product's device approval flow is exercised host-side by the fixture setup (USB-kit join + operator approval asserted at R1/R4). What R3 needs and cannot show is the second device reaching a real URL.

## Named PENDING (never recorded clean)
- hosted shareable non-localhost URL + DNS name + second-device reachability: not built in this strike. The fixture origin (https://lf-host-01.example-tailnet.ts.net:8443) is a made-up name that never resolves, the current host binds 127.0.0.1 only, and a dedicated host/DNS name are owner acts out of scope (no purchase, no DNS work). Device-approval via a second device therefore cannot be shown; the host-side device registry is covered as a stand-in under R4.

## Recorded metadata
```json
{
  "origin": "https://lf-host-01.example-tailnet.ts.net:8443",
  "listener": {
    "address": "127.0.0.1",
    "family": "IPv4",
    "port": 36023,
    "base": "http://127.0.0.1:36023"
  }
}
```
