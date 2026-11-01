# R0 — Pre-pass install proof (read-only) (pass 2)

- Step: R0 — Pre-pass install proof (read-only)
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: (none — team-side pre-pass; no operator action)
- Expected: control socket answers status with integrity ok and lanes allowed; fleet-dsh active as user fleet; the installed release tarball hashes to its published digest; the wrapper reports the pinned DSH; the repo preflight passes every install-related check.
- Actual: 22/22 hard checks held; 0 named pending; 3 recorded finding(s)
- Word: **clean**
- Evidence files: `R0-pre/control-status.json`, `R0-pre/policy-get.json`, `R0-pre/fleet-unit.txt`, `R0-pre/package-identity.json`, `R0-pre/package-identity.txt`, `R0-pre/wrapper-version.txt`, `R0-pre/wrapper-status.txt`, `R0-pre/wrapper-dump-config-excerpt.txt`, `R0-pre/wrapper-dump-config.json`, `R0-pre/repo-pin.txt`, `R0-pre/preflight/run.log`, `R0-pre/preflight/summary.txt`, `R0-pre/preflight-failclosed.txt`
- Timestamp (UTC): 2026-11-01T23:08:12Z .. 2026-11-01T23:08:12Z
- Blocker / finding: none

## Checks
- PASS control socket answers status — name=fleet-core protocol=1 pid=510804
- PASS fleet-core identity — name=fleet-core protocol=1 node=v26.12.0
- PASS lanes allowed — lanes_allowed=true
- PASS integrity ok (chain, sandbox, rows, guard) — integrity.ok=true chain.ok=true verified_at_boot=44218 sandbox.ok=true rows.ok=true guard.ok=true
- PASS control peer is uid 1000 (normal user, no root) — peer_uid=1000 accepted=37 rejected=0
- PASS selftest flag on (fixture:true orders and selftest sessions are gated by it) — policy.selftest=true source=default
- PASS policy-get answers — rows=19
- PASS policy pins spend ceiling + selftest — spend.hard_ceiling_usd=5 selftest=true
- PASS fleet-dsh unit active — is-active=active ActiveState=active SubState=running MainPID=510804
- PASS unit runs as user fleet — User=fleet Group=fleet
- PASS unit is contained (ProtectSystem strict, NoNewPrivileges, egress IP deny) — ProtectSystem=strict NoNewPrivileges=yes IPAddressDeny=169.254.0.0/16 100.64.0.0/10 ::/0 192.168.0.0/16 172.16.0.0/12 10.0.0.0/8 0.0.0.0/8
- PASS installed release identity recorded from the install's own audit line — cayleb-dsh-fleet-0.3.0.tgz sha256=8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070
- PASS the installed release tarball is present and hashes to the installed digest — cayleb-dsh-fleet-0.3.0.tgz sha256=8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070 expected=8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070
- PASS installed release version recorded from the installed release name (not assumed) — version=0.3.0
- PASS drop directory holds no ambiguity about what is installed (staged releases enumerated) — cayleb-dsh-fleet-0.3.0.tgz=8173d13ebee5…, cayleb-dsh-fleet-0.2.2.tgz=37e3a542a32f…, cayleb-dsh-fleet-0.2.1.tgz=55516f9ae953…, cayleb-dsh-fleet-0.2.0.tgz=842e76f258fc…
- PASS wrapper version runs read-only and reports a DSH version — exit=0 version=0.2.2-alpha.2
- PASS wrapper status confirms the fleet install — exit=0 (uid/version lines present)
- PASS dump-config exits 0 and mounts the fleet bundle rows — exit=0 bundle rows=77
- PASS repo pin recorded — HEAD=cb909b4f03ff529349abcfafb36d1e419bf3ac52 branch=strike/ergon-rehearsal dirty=0
- PASS repo r0-preflight ran and every install-related check passed — exit=1 PASS=5 FAIL=1 → FAIL  company project id not recorded (R0 blocking: write id + region only, no secret)
- PASS r0-preflight names its only blocking item as the company Supabase project (P5 owner act) if it blocks — exit=1 because project-id.txt is absent: the company Supabase project is a P5 owner act, out of scope for this strike (named, not silently skipped)
- PASS preflight fails closed with no evidence supplied — exit=1 stdin=R0 PREFLIGHT: FAIL (3 blocking)

## Findings recorded (not a failure of this step)
- staged but not installed — The drop directory also carries cayleb-dsh-fleet-0.2.2.tgz, cayleb-dsh-fleet-0.2.1.tgz, cayleb-dsh-fleet-0.2.0.tgz — listed in package-identity.json but NOT claimed as installed.
- host — This pass runs on the platform host itself (outis-desktop), where the installed @cayleb/dsh-fleet release R0 resolved (version 0.3.0, cayleb-dsh-fleet-0.3.0.tgz, digest verified; source: journald install audit line (dsh-fleet-admin plugin-add) — what the install reports) is reported by the install. The real R0 also wants a dedicated fresh cloud host + DNS + company Supabase project: those are owner acts (R0-PROVISIONING P1/P4/P5) and out of scope for this strike by its anti-goals. They are named in the pass index, not skipped silently.
- no company state on the deploy paths — The generic preflight checked /srv/ergon /opt/ergon /var/lib/ergon /etc/ergon: absent (see preflight/absence-of-company-state.txt).

## Recorded metadata
```json
{
  "control": {
    "name": "fleet-core",
    "protocol": 1,
    "node": "v26.12.0",
    "uptime_s": 60,
    "db": {
      "path": "/var/lib/fleet/state/fleet.db",
      "journal_mode": "wal",
      "events": 44271,
      "head": {
        "seq": 44271,
        "hash": "f05e9c51d7d471da2b613a4c4a1d0d21e32408f965417cdaffbe6d4bed3f95e3"
      }
    },
    "peer_uid": 1000,
    "lanes_allowed": true
  },
  "unit": {
    "ActiveState": "active",
    "SubState": "running",
    "MainPID": "510804",
    "User": "fleet",
    "ExecStart": "{ path=/var/lib/fleet/npm/bin/dsh ; argv[]=/var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091 ; ignore_errors=no ; start_time=[Sun 2026-11-01 18:07:10 EST] ; stop_time=[n/a] ; pid=510804 ; code=(null) ; status=0/0 }"
  },
  "identityDigest": "8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070",
  "identityFile": "cayleb-dsh-fleet-0.3.0.tgz",
  "identityVersion": "0.3.0",
  "identitySource": "journald install audit line (dsh-fleet-admin plugin-add) — what the install reports",
  "wrapper": {
    "version": "0.2.2-alpha.2",
    "bundleRows": 77,
    "dumpConfigSha256": "0eee39d2dc1fd60eb78028edb36ee73b39704dab049ed016b477e816df5918c6"
  },
  "repo": {
    "HEAD": "cb909b4f03ff529349abcfafb36d1e419bf3ac52",
    "branch": "strike/ergon-rehearsal",
    "dirty": 0
  },
  "startedUtc": "2026-11-01T23:08:12Z"
}
```
