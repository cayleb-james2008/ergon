# OWNER COMMAND RELAY — one-time V3 identity run: sudoers + polkit install, removal, pre-flight

Prepared 2026-11-01T15:30Z by `fleet-lead` (Team B, Lane 1) at the Lead's instruction. **Owner-approved narrow
rule only (option (a), Cayleb via Nox 151700Z).** Nothing here is installed or executed by Team B. The operator
(Cayleb) runs §B once with his password; §D removes the rule after the run + the fresh checker have what they
need. Read-only facts used: the reviewed procedure source (sha256 `652d69f5…`), its bootstrap document
(`12997259…`), and the live capability checks in §E.

**Exactly the literal commands the procedure runs** (from `PROPOSED-identity-procedure-v3.py` lines 215–285;
`STAGE=/run/dsh-fleet-id-cf93107`; inputs `caller.py`, `identity_snapshot_core.py`, `manifest.txt`):
`sudo -- mkdir -m 0700 -- STAGE`; `sudo -- dd of=STAGE/<name> conv=excl status=none` ×3;
`sudo -- chmod 0444 -- STAGE/<name>` ×3; `sudo -- stat -c '%F|%u|%g|%a|%h' -- STAGE/<name>` ×3;
`sudo -- sha256sum -- STAGE/<name>` ×3; `sudo -- chmod 0755 -- STAGE`;
`sudo --user fleet -- env -i PATH=/usr/bin:/bin LC_ALL=C python3 -I -S -B STAGE/caller.py`;
and `systemctl --no-ask-password show|stop|start fleet-dsh` **without sudo** (polkit, rule in §A2).

## A. Drop-in contents (the bytes §B installs)

### A1. `/etc/sudoers.d/dsh-identity-v3` (root:root 0440)

```sudoers
# dsh-identity-v3 — ONE-TIME V3 identity run, owner-approved 2026-11-01 (Nox 151700Z).
# Scope: the literal commands the reviewed procedure sha256 652d69f5… executes. REMOVE after the run (§D).
Cmnd_Alias DSH_ID_V3_STAGE = \
  /usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/caller.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/identity_snapshot_core.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/manifest.txt conv=excl status=none, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/chmod 0755 -- /run/dsh-fleet-id-cf93107
cayleb ALL=(root) NOPASSWD: DSH_ID_V3_STAGE
cayleb ALL=(fleet) NOPASSWD: /usr/bin/env -i PATH\=/usr/bin\:/bin LC_ALL\=C /usr/bin/python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py
```

### A2. `/etc/polkit-1/rules.d/49-dsh-identity-v3.rules` (root:root 0644)

`systemctl stop/start` runs as `cayleb` **without sudo**; live check (§E) shows polkit demands
`auth_admin_keep` for `manage-units` today. Minimal rule: only unit `fleet-dsh` and only `stop`/`start`.

```js
// dsh-identity-v3 — ONE-TIME fleet-dsh stop/start for the owner-approved V3 identity run (2026-11-01, Nox 151700Z).
// Scope: unit fleet-dsh, verbs stop/start, user cayleb only. REMOVE after the run (§D).
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        subject.user == "cayleb" && subject.local && subject.active &&
        (action.lookup("unit") == "fleet-dsh.service" || action.lookup("unit") == "fleet-dsh") &&
        (action.lookup("verb") == "stop" || action.lookup("verb") == "start")) {
        return polkit.Result.YES;
    }
});
```


> **sudoers escaping:** the caller rule escapes `=` and `:` (`PATH\=/usr/bin\:/bin`, `LC_ALL\=C`) — those
> characters are special in a sudoers command spec; the unescaped form fails `visudo -cf` with a syntax error
> (caught and fixed during validation). The escaped spec matches the literal argv the procedure passes.

## B. INSTALL BLOCK — Cayleb copies this once, runs it, types his password when asked

```sh
set -euo pipefail
[ "$(id -u)" = 1000 ] || { echo "run as cayleb (uid 1000)"; exit 1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cat > "$T/dsh-identity-v3" <<'SUDOERS'
# dsh-identity-v3 — ONE-TIME V3 identity run, owner-approved 2026-11-01 (Nox 151700Z).
Cmnd_Alias DSH_ID_V3_STAGE = \
  /usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/caller.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/identity_snapshot_core.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/manifest.txt conv=excl status=none, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/chmod 0755 -- /run/dsh-fleet-id-cf93107
cayleb ALL=(root) NOPASSWD: DSH_ID_V3_STAGE
cayleb ALL=(fleet) NOPASSWD: /usr/bin/env -i PATH\=/usr/bin\:/bin LC_ALL\=C /usr/bin/python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py
SUDOERS
cat > "$T/49-dsh-identity-v3.rules" <<'POLKIT'
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        subject.user == "cayleb" && subject.local && subject.active &&
        (action.lookup("unit") == "fleet-dsh.service" || action.lookup("unit") == "fleet-dsh") &&
        (action.lookup("verb") == "stop" || action.lookup("verb") == "start")) {
        return polkit.Result.YES;
    }
});
POLKIT
/usr/bin/visudo -cf "$T/dsh-identity-v3"                       # validate BEFORE installing
sudo /usr/bin/install -o root -g root -m 0440 "$T/dsh-identity-v3" /etc/sudoers.d/dsh-identity-v3
sudo /usr/bin/install -o root -g root -m 0644 "$T/49-dsh-identity-v3.rules" /etc/polkit-1/rules.d/49-dsh-identity-v3.rules
sudo /usr/bin/visudo -cf /etc/sudoers.d/dsh-identity-v3        # re-validate installed file
sudo /usr/bin/visudo -c >/dev/null                              # whole sudo config still parses
echo "INSTALLED dsh-identity-v3 (sudoers 0440 + polkit 0644)"
```

## C. PRE-FLIGHT TEST PLAN — read-only, NO service action (run after §B, before any window)

```sh
# 1) rules present and exactly scoped (prints the allowed commands; does not run anything)
sudo -l -U cayleb | grep -n 'dsh-fleet-id-cf93107'
# expect: the mkdir/dd/chmod/stat/sha256sum shapes + the fleet caller line.

# 2) every shape resolvable WITHOUT a password prompt (does not execute: -l only checks policy)
sudo -n -l /usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107
sudo -n -l /usr/bin/dd of=/run/dsh-fleet-id-cf93107/caller.py conv=excl status=none
sudo -n -l /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/caller.py
sudo -n -l /usr/bin/chmod 0755 -- /run/dsh-fleet-id-cf93107
sudo -n -l /usr/bin/stat -c '%F|%u|%g|%a|%h' -- /run/dsh-fleet-id-cf93107/caller.py
sudo -n -l /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/caller.py
sudo -n -l -u fleet /usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py
# each must print the permitted command and exit 0 (no "a password is required").

# 3) a harmless root read proves the NOPASSWD path works from a non-tty child (file does not exist yet:
#    a permission success shows as "No such file", a denial as "a password is required")
sudo -n /usr/bin/stat -c '%F|%u|%g|%a|%h' -- /run/dsh-fleet-id-cf93107/caller.py 2>&1 | head -2

# 4) first-attempt condition: the stage must NOT exist, outputs must NOT exist
test ! -e /run/dsh-fleet-id-cf93107 && echo STAGE-ABSENT
ls /home/cayleb/Documents/Codex/2026-10-31/task/mission1/identity-cf93107-v3-observation.json 2>/dev/null && echo OUTPUT-EXISTS || echo OUTPUT-ABSENT
ls /home/cayleb/Documents/Codex/2026-10-31/task/mission1/identity-cf93107-v3-diagnostic.txt 2>/dev/null && echo ERR-EXISTS || echo ERR-ABSENT

# 5) polkit rule parsed: polkitd logs a warning if the rule file is bad (no service action)
sudo journalctl -u polkit --since '2 min ago' --no-pager | grep -iE 'error|invalid|failed' || echo POLKIT-NO-PARSE-ERROR

# 6) service read still works, no action
systemctl --no-ask-password show fleet-dsh -p ActiveState -p SubState
```

> **WHO CAN RUN §C (measured 2026-11-01T15:30Z):** Team B's seat runs with `NoNewPrivs=1` (harness default), and
> every root-target `sudo` check there fails with `sudo: The "no new privileges" flag is set...` — so **§C must be
> run from the operator's own terminal (or any NNP-free shell)**, not from the Team B lane session. The read-only
> runner is prepared and self-checking: `~/workspace/harness-build/scratch-tb-l1/preflight-v3.sh`
> (sha256 `efe4d2b70212833e58d15c3d15f78144dd1026b0def22b6007f1c30f7d0235f5`; prints PASS/FAIL per check, writes a
> digest receipt, exits non-zero on any FAIL). It is equivalent to the §C commands; it also WARNS when run from an
> NNP-flagged shell. None of its checks perform a state change.

The **definitive** polkit proof is the run's own `stop` step (operator present). No pre-flight can call
`manage-units` without performing it.

## D. REMOVAL BLOCK — run after the V3 run AND the fresh checker has what it needs

```sh
set -euo pipefail
sudo /usr/bin/rm -f /etc/sudoers.d/dsh-identity-v3 /etc/polkit-1/rules.d/49-dsh-identity-v3.rules
sudo /usr/bin/visudo -c >/dev/null
echo "REMOVED dsh-identity-v3"
# verify gone:
sudo -l -U cayleb | grep -c 'dsh-fleet-id-cf93107' || echo "0 rules remain (expected)"
test ! -e /etc/sudoers.d/dsh-identity-v3 && echo SUDOERS-GONE
test ! -e /etc/polkit-1/rules.d/49-dsh-identity-v3.rules && echo POLKIT-GONE
```
The preserved stage `/run/dsh-fleet-id-cf93107` is **NOT** removed here: its removal is a separate
owner-authorised act after identity/contents are confirmed. Nobody `rm -rf`s an unconfirmed stage.

## E. Capability facts these files are based on (read-only checks, 2026-11-01T15:21–15:33Z)

- `systemctl --no-ask-password show fleet-dsh -p ActiveState -p SubState` works as `cayleb` → `active/running`
  (no auth needed for `show`).
- `pkcheck --action-id org.freedesktop.systemd1.manage-units --process $$` → `auth_admin_keep`, "requires
  authentication" ⇒ `stop`/`start` are **not** authorised for `cayleb` today ⇒ the polkit rule is required.
- `pkcheck --action-id … --detail unit … --detail verb …` cannot be used from this seat ("only trusted
  callers … can pass details"), so polkit verification is §C5 + the run's own stop step.
- `sudo -n /usr/bin/{mkdir,dd,chmod,stat,sha256sum}` → "a password is required" (no NOPASSWD root rule today;
  this seat also carries the NNP flag, so the operator's normal terminal is the only valid place to test).
- `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin version` works today (existing fleet rule), so the
  `(fleet)` caller grant follows a proven pattern.
- `visudo -cf` on the §A1 content: **parsed OK** (validated locally before sending).
- polkit version 128, `/etc/polkit-1/rules.d/` watched by an active `polkit.service`.

## F. Operator invocation (from the reviewed bootstrap document; DO NOT RUN until the Lead says the window is open)

```sh
( set -euC; umask 077
  exec > /home/cayleb/Documents/Codex/2026-10-31/task/mission1/identity-cf93107-v3-procedure-summary.json
  exec 2> /home/cayleb/Documents/Codex/2026-10-31/task/mission1/identity-cf93107-v3-bootstrap.err
  /usr/bin/env -i PATH=/usr/bin:/bin LC_ALL=C /usr/bin/python3 -I -S -B - <<'PY'
…exact bytes from IDENTITY-CALLER-PROCEDURE-V3.md (bootstrap section)…
PY
)
```
The procedure refuses to run as root and requires `PATH=/usr/bin:/bin LC_ALL=C` and uid name `cayleb`; run it
as Cayleb in his own terminal, one invocation only, present for the whole window.

**Status: prepared, validated (sudoers parsed OK), NOT installed, no service action.** `INSTALLED` is the
operator's echo; removal is §D, batched per the Lead's message; report both on the board when done.
