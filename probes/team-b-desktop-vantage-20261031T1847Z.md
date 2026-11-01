# TEAM B — DESKTOP-VANTAGE EVIDENCE: Stage 2 service-state legs

Ruling applied: Nox batch-2 (18:45 ET, `board/probes/nox-rulings-batch2-20261031T224500Z.md` §4) — the service-state legs (installed unit state, bus reachability) are **DESKTOP-vantage legs**: Team B executes them on this desktop and files the evidence; Nox checker-verifies from the artefacts. The earlier "Operation not permitted" was the Outis cloud shell's system-bus permission artefact, not the desktop's answer.
Executed by `fleet-lead` on `outis-desktop` as user `cayleb` (uid 1000), read-only, 2026-10-31 18:46–18:48 ET. No unit, service, package or config was changed; no credential was read.

## 1. Installed unit state — `fleet-dsh.service`

```
$ systemctl is-active fleet-dsh
active
[rc=0]

$ systemctl is-enabled fleet-dsh
disabled
[rc=0]

$ systemctl show fleet-dsh -p ActiveState -p SubState -p MainPID -p ExecMainStartTimestamp -p FragmentPath --no-pager
ActiveState=active
SubState=running
FragmentPath=/etc/systemd/system/fleet-dsh.service
MainPID=2602074
ExecMainStartTimestamp=Sat 2026-10-31 12:27:26 EDT
[rc=0]
   (18:46:46 ET)

$ systemctl show fleet-dsh -p User -p Group -p ExecStart -p ExecStartPre -p WorkingDirectory -p Environment -p InvocationID -p ActiveEnterTimestamp -p LoadState --no-pager
LoadState=loaded
ActiveEnterTimestamp=Sat 2026-10-31 12:27:26 EDT
InvocationID=df02e9346ebd43a680d21ae9eccc9f07
ExecStart={ path=/var/lib/fleet/npm/bin/dsh ; argv[]=/var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091 ; ignore_errors=no ; start_time=[Sat 2026-10-31 12:27:26 EDT] ; stop_time=[n/a] ; pid=2602074 ; code=(null) ; status=0/0 }
Environment=HOME=/var/lib/fleet DSH_HOME=/var/lib/fleet/dsh DSH_TELEMETRY_DISABLED=1 DSH_TELEMETRY_MODE=DISABLED PATH=/var/lib/fleet/npm/bin:/usr/bin
WorkingDirectory=/var/lib/fleet
User=fleet
Group=fleet
   (18:47:59 ET)
```

Process view (read-only `ps`, corroborates the unit):

```
$ ps -eo user,pid,lstart,args | grep -i fleet
fleet    2602074 Sat Oct 31 12:27:26 2026 node /var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091
   (18:46:46 ET)
```

**Recorded facts:** unit loaded and **active/running** since 12:27:26 EDT (InvocationID `df02e9346ebd43a680d21ae9eccc9f07`, MainPID 2602074), running as `User=fleet` with `HOME=/var/lib/fleet`, `DSH_HOME=/var/lib/fleet/dsh`, telemetry disabled, bound to `127.0.0.1:3091`. **`is-enabled` = `disabled`** — the unit runs now but is not enabled at boot; recorded as a fact for the gate report (it is not a pass/fail on its own, and changing it is not this team's act).

## 2. Installed bundle artefact (drop dir) — version evidence

```
$ sha256sum /var/lib/fleet-drop/*.tgz
842e76f258fcfbd1575d6c7fa48b5b10e1603a2b0d6fbc9dbf672d3351661415  /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.0.tgz
55516f9ae9535dc585da96587af45f55d83edac67c88e48d3bdc32ad652d10d4  /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.1.tgz
$ cat /var/lib/fleet-drop/*.sha256   # sidecars match the hashes above
   (18:48 ET)
```

**Limit, stated plainly:** the installed tree lives under `/var/lib/fleet`, which is `Permission denied` for `cayleb` (`ls /var/lib/fleet` denied), so the installed package's own `package.json` cannot be read from this vantage. The installed **version is evidenced as 0.2.1** by the drop-dir tarball + matching sidecar + the lane record; the byte-identity of the installed tree is **not** verifiable from this seat and stays for Nox's artefact check. No 0.2.2 artefact exists (only 0.2.0 and 0.2.1).

## 3. Bus reachability — fleet control/egress sockets

```
$ ss -xl | grep -iE 'dsh|fleet'
u_str LISTEN 0 511 /run/fleet/egress.sock  39227280 * 0
u_str LISTEN 0 511 /run/fleet/control.sock 39227282 * 0

$ ls -la /run/fleet/
ls: cannot open directory '/run/fleet/': Permission denied

# bare connect, no payload, 2 s timeout
/run/fleet/control.sock: CONNECT OK (reachable as uid 1000)
/run/fleet/egress.sock: CONNECT FAILED -> PermissionError: [Errno 13] Permission denied
   (18:48 ET)
```

**Recorded facts:** both fleet sockets are listening; from the `cayleb` vantage the **control socket is reachable** (connect succeeds) and the **egress socket is denied by policy** (connect refused) — the intended split. No payload was sent; this is a reachability probe only. `ss` was readable from the desktop; `/run/fleet` directory listing is denied, so socket modes are not readable from this seat.

## 4. What this closes / what it does not

- **Closes (desktop vantage, artefacts for Nox's checker):** installed unit state (active/running, identity fields above); bus reachability split (control reachable, egress denied).
- **Does not close:** installed-tree byte identity (needs a fleet-scope or checker vantage); 0.2.2 install/identity (no artefact exists yet); formal Stage 2 pass (fresh checker with veto).
- These artefacts are cited in `board/reports/stage2-gate-report-DRAFT.md` rows 13/14 and the §6 service-state section. They are **evidence, not a verdict**; the verdict is Nox's checker's.
