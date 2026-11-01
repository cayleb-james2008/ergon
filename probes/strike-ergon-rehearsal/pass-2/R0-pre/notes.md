# R0 — Pre-pass install proof (read-only) (pass 2)

- Step: R0 — Pre-pass install proof (read-only)
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: (none — team-side pre-pass; no operator action)
- Expected: control socket answers status with integrity ok and lanes allowed; fleet-dsh active as user fleet; the installed release tarball hashes to its published digest; the wrapper reports the pinned DSH; the repo preflight passes every install-related check.
- Actual: 20/20 hard checks held; 0 named pending; 2 recorded finding(s)
- Word: **clean**
- Evidence files: `R0-pre/control-status.json`, `R0-pre/policy-get.json`, `R0-pre/fleet-unit.txt`, `R0-pre/package-identity.json`, `R0-pre/package-identity.txt`, `R0-pre/wrapper-version.txt`, `R0-pre/wrapper-status.txt`, `R0-pre/wrapper-dump-config-excerpt.txt`, `R0-pre/wrapper-dump-config.json`, `R0-pre/repo-pin.txt`, `R0-pre/preflight/run.log`, `R0-pre/preflight/summary.txt`, `R0-pre/preflight-failclosed.txt`
- Timestamp (UTC): 2026-11-01T22:07:36Z .. 2026-11-01T22:07:36Z
- Blocker / finding: none

## Checks
- PASS control socket answers status — name=fleet-core protocol=1 pid=427587
- PASS fleet-core identity — name=fleet-core protocol=1 node=v26.12.0
- PASS lanes allowed — lanes_allowed=true
- PASS integrity ok (chain, sandbox, rows, guard) — integrity.ok=true chain.ok=true verified_at_boot=42154 sandbox.ok=true rows.ok=true guard.ok=true
- PASS control peer is uid 1000 (normal user, no root) — peer_uid=1000 accepted=694 rejected=0
- PASS selftest flag on (fixture:true orders and selftest sessions are gated by it) — policy.selftest=true source=default
- PASS policy-get answers — rows=19
- PASS policy pins spend ceiling + selftest — spend.hard_ceiling_usd=5 selftest=true
- PASS fleet-dsh unit active — is-active=active ActiveState=active SubState=running MainPID=427587
- PASS unit runs as user fleet — User=fleet Group=fleet
- PASS unit is contained (ProtectSystem strict, NoNewPrivileges, egress IP deny) — ProtectSystem=strict NoNewPrivileges=yes IPAddressDeny=169.254.0.0/16 100.64.0.0/10 ::/0 192.168.0.0/16 172.16.0.0/12 10.0.0.0/8 0.0.0.0/8
- PASS release tarball hashes to its published digest — sha256=37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177
- PASS release tarball names version 0.2.2 — cayleb-dsh-fleet-0.2.2.tgz
- PASS wrapper version runs read-only — exit=0 version=0.2.2-alpha.2
- PASS wrapper status confirms the fleet install — exit=0 (uid/version lines present)
- PASS dump-config exits 0 and mounts the fleet bundle rows — exit=0 bundle rows=75
- PASS repo pin recorded — HEAD=e284135264f4025a1527c6754261b23907620dea branch=strike/ergon-rehearsal dirty=0
- PASS repo r0-preflight ran and every install-related check passed — exit=1 PASS=5 FAIL=1 → FAIL  company project id not recorded (R0 blocking: write id + region only, no secret)
- PASS r0-preflight names its only blocking item as the company Supabase project (P5 owner act) if it blocks — exit=1 because project-id.txt is absent: the company Supabase project is a P5 owner act, out of scope for this strike (named, not silently skipped)
- PASS preflight fails closed with no evidence supplied — exit=1 stdin=R0 PREFLIGHT: FAIL (3 blocking)

## Findings recorded (not a failure of this step)
- host — This pass runs on the platform host itself (outis-desktop), where the fleet plugin 0.2.2 is installed. The real R0 also wants a dedicated fresh cloud host + DNS + company Supabase project: those are owner acts (R0-PROVISIONING P1/P4/P5) and out of scope for this strike by its anti-goals. They are named in the pass index, not skipped silently.
- no company state on the deploy paths — The generic preflight checked /srv/ergon /opt/ergon /var/lib/ergon /etc/ergon: absent (see preflight/absence-of-company-state.txt).

## Recorded metadata
```json
{
  "control": {
    "name": "fleet-core",
    "protocol": 1,
    "node": "v26.12.0",
    "uptime_s": 1384,
    "db": {
      "path": "/var/lib/fleet/state/fleet.db",
      "journal_mode": "wal",
      "events": 43695,
      "head": {
        "seq": 43695,
        "hash": "d5c5d059691c520115d300f1685539a88931b1e70f10fa51f4b558168cb44489"
      }
    },
    "peer_uid": 1000,
    "lanes_allowed": true
  },
  "unit": {
    "ActiveState": "active",
    "SubState": "running",
    "MainPID": "427587",
    "User": "fleet",
    "ExecStart": "{ path=/var/lib/fleet/npm/bin/dsh ; argv[]=/var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091 ; ignore_errors=no ; start_time=[Sun 2026-11-01 16:44:30 EST] ; stop_time=[n/a] ; pid=427587 ; code=(null) ; status=0/0 }"
  },
  "identityDigest": "37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177",
  "identityFile": "cayleb-dsh-fleet-0.2.2.tgz",
  "wrapper": {
    "version": "0.2.2-alpha.2",
    "bundleRows": 75,
    "dumpConfigSha256": "0614f4f796ba7fff13be968982096f0ca2213b527877b01b85bd7ff7c8de4756"
  },
  "repo": {
    "HEAD": "e284135264f4025a1527c6754261b23907620dea",
    "branch": "strike/ergon-rehearsal",
    "dirty": 0
  },
  "startedUtc": "2026-11-01T22:07:36Z"
}
```
