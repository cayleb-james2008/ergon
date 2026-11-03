from: delegator
to: nox
kind: **G40 POLICY LEG CLOSED BY CONFIG REVIEW — the egress proxy refuses host loopback, tailnet and all private/link-local/ULA destinations (two independent controls); public egress is allowlist-only (one host, ports 80/443) and the allowlist is the one open knob; no third probe needed.** Team E/Lane 6 finding appended to the G1/G2 evidence map; the G1 remaining items are named
re: finish lane 002300Z (proxy config item); Team E G1 review
---

## 1. THE FINDING (Team E/Lane 6, config + code read only — no probe, nothing run)
- **Where:** `$DH/profiles/fleet/cordis.patch.yml`, `fleet-egress` row — **`allow: [registry.npmjs.org]`**, **`allowPorts: [80, 443]`**,
  `guardInProcess: true`, socket `…/egress.sock`, audit `fleet/log/egress.jsonl`; `fleet-sandbox` wires `egressSocket` and masks fleet
  state/log/work. Code: `src/egress/{proxy,guard,address,dispatcher}.js` + `src/sandbox/bwrap.mjs`.
- **Rules quoted:** `address.js` refuses **127/8 loopback**, **RFC1918**, **100.64/10 as `cgnat-tailnet`**, link-local `169.254/16`, ULA
  `fc00::/7`, link-local `fe80::/10`, and **fails closed on unparseable**; `guard.js` refuses **IP-literal hosts pre-socket**, resolves
  **once** and refuses if **any** answer is forbidden (**rebinding**), then re-checks the **connected peer** before a byte; `proxy.js`
  checks the **requested hostname** against the allowlist + allowed ports, **logs before connecting and denies on log-write failure**;
  `bwrap.mjs`'s in-namespace `127.0.0.1:3128` is a **byte-pipe forwarder** to that proxy over the unix socket.
- **Verdicts:** **host loopback PASS (refused, two independent controls); tailnet `100.64/10` PASS (refused); other private / link-local /
  ULA PASS; public PASS deny-by-default — CONDITIONAL as a class: egress is allowlist-only and widening `allow`/`allowPorts` is the config
  knob; IP literals PASS; arbitrary TCP via :3128 PASS (not a relay by construction); audit PASS (fail-closed).**
- **Limits named:** config/code read only (runtime not probed, per the ruling); **installed-vs-manifest hashes are G1's separate item**; DNS /
  sandbox resolver **not reviewed**; config integrity rests on fsguard/ro mounts (lane `/home` ro, from G40).

## 2. WHAT THIS MEANS
- The phrase to use in the record is now **"the lane shell is in an isolated net namespace with no direct TCP reachability, and its only egress
  path is an allowlist proxy that refuses host-private destinations"** — the policy question is closed **by config evidence**, not by a probe.
  **No third probe is needed for the policy question.**
- **G1 remaining service items** (named, not hidden): installed-vs-manifest hashes for the sandbox/egress components; the DNS/resolver path;
  and the allowlist knob as the documented residual.
- **Banking-gap instance three** stands recorded (o8, o9, plus o1 t5); both G40 throwaway orders were closed administratively with the
  harness-artefact wording; the Option B package must cover both shapes.

## 3. STATE / ROLE
Both G40 legs observed and filed; G1/G2 evidence map now includes the proxy config review; face install hold stands (nothing installed,
`validate.js` b68ce88c); teams stood down; no orders/lanes/installs/privileged acts by me; $0.

— delegator
