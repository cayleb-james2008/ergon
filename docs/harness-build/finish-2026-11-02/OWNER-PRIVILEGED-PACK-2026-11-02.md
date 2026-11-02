# Owner privileged list: verified content pack (2 Nov 2026, prepared 13:55 ET)

For: Cayleb, home between 14:30 and 15:00 ET. Display-and-verify material only. Nothing here was run by any agent. Every fact below was read from this machine today (outis-desktop) with read-only commands; where a fact could not be read, it says so.

## Status at a glance

| # | item | state | needs sudo | what you should expect |
|---|---|---|---|---|
| 1 | Tailscale SSH on | READY | no (you are the Tailscale operator) | one command, one read-back |
| 2 | OpenSSH server | READY WITH A CHANGE (read below) | yes | hardened config first, then start; no remote login until you add a key |
| 3 | sudoers rule for fleet-dsh | see section 3 | yes | see section 3 |
| 4 | Owner Elevation first real run | NOT ON TODAY'S LIST | n/a | leg 4 is unbuilt; nothing to run today |

## 1. `tailscale set --ssh` (READY, no sudo)

Verified read-only at 13:42 ET: `RunSSH = False`, `OperatorUser = 'cayleb'`, `WantRunning = True`, tailscaled active and enabled (Tailscale 1.104.2). Because you are the operator, no sudo is needed.

Run (as cayleb, in any terminal):

    tailscale set --ssh

Read-back that proves it took (expect `"RunSSH": true`):

    tailscale debug prefs | grep -E '"RunSSH"'

Undo: `tailscale set --ssh=false`.
Note: Tailscale SSH also depends on your tailnet's access-control policy allowing SSH to this node. That is set in the Tailscale admin console and cannot be read from this machine. The prefs flag above proves the setting took; a real login from your phone proves the policy.

## 2. OpenSSH server (READY WITH A CHANGE)

What I verified (read-only):
- The binary is complete: `/usr/bin/sshd`, package `openssh 10.6p1-2`. No install is needed.
- The unit `sshd.service` exists, `disabled` and `inactive`. It `Wants=sshdgenkeys.service`, which creates the missing host keys (none exist in `/etc/ssh/` yet) on first start, so no manual key step is required.
- Nothing listens on port 22 now.
- **The stock configuration is not safe to switch on as it stands.** With no changes, the effective settings are `PasswordAuthentication yes`, `AuthenticationMethods any`, no `AllowUsers`, listening on `0.0.0.0:22` and `[::]:22`. Plain `systemctl enable --now sshd` would accept password logins for any account.
- The firewall (`ufw`) is active with default input policy DROP and has no rule for port 22, so the server would be reachable from this machine only until you add one.
- A hardened `sshd_config_nox` and `nox_host_ed25519_key` from 23 Oct exist but no unit uses them. The `nox` account exists with a `nologin` shell. I did not use or change them.

So the safe sequence adds one hardening file BEFORE the service starts. I checked the candidate offline as a normal user (scratch config plus a throwaway host key in /tmp): `sshd -t` exit 0, and `sshd -T` shows `passwordauthentication no`, `permitrootlogin no`, `authenticationmethods publickey`, `allowusers cayleb`.

Run (each line separately; you will be asked for your password by sudo):

    sudo tee /etc/ssh/sshd_config.d/10-fleet-hardening.conf >/dev/null <<'EOF'
    PasswordAuthentication no
    KbdInteractiveAuthentication no
    PermitRootLogin no
    AuthenticationMethods publickey
    PubkeyAuthentication yes
    AllowUsers cayleb
    EOF
    sudo ssh-keygen -A
    sudo sshd -t && echo CONFIG-OK
    sudo systemctl enable --now sshd.service

(`10-` sorts before the stock `99-archlinux.conf`; sshd uses the first value it sees, so the hardening wins.)

Read-backs (the first three need no sudo):

    systemctl is-active sshd.service ; systemctl is-enabled sshd.service    # expect: active / enabled
    ss -ltn | grep ':22'                                                    # expect: 0.0.0.0:22 and [::]:22 LISTEN
    ls /etc/ssh/ssh_host_*                                                  # expect: the generated host keys
    sudo sshd -T | grep -E '^(passwordauthentication|permitrootlogin|authenticationmethods|allowusers)'
                                                                            # expect: no / no / publickey / cayleb
    ssh -o PreferredAuthentications=password -o PubkeyAuthentication=no cayleb@localhost
                                                                            # expect: Permission denied (publickey)

Two things this does NOT do, on purpose:
1. **No remote login yet.** You have no `~/.ssh` directory and no authorized key, so key-only SSH will refuse everyone, including you, until a public key is added (a user-level step, no sudo: create `~/.ssh` with mode 700 and append the public key of the device you want to log in from to `~/.ssh/authorized_keys` with mode 600). Tell Nox which device, and what key, if you want that done.
2. **No firewall change.** Reaching port 22 from the tailnet needs one more privileged line, only if you want it: `sudo ufw allow in on tailscale0 to any port 22 proto tcp` (read-back: `sudo ufw status | grep 22`). Item 1 gives tailnet SSH without it.

Undo: `sudo systemctl disable --now sshd.service` and `sudo rm /etc/ssh/sshd_config.d/10-fleet-hardening.conf`.

## 3. sudoers rule for fleet-dsh: NOT READY (do not run anything from this section)

**Reason: no Lane 6 security review of this rule is on record.** I searched the team board (origin/team at 684110bd), the delegator board, the plugin's `ops/` and the finish docs for `sudoers`, `NOPASSWD`, `fleet-dsh-control` and every Team E / Lane 6 file. The only reviews on record cover other things: the `dsh-fleet-admin` wrapper kit, a temporary one-run block, and the Owner Elevation helper. A Lane 6 review has been requested through the delegator (see the delivery note); until it lands, no sudoers line is shown to you to run.

For Lane 6 (and Nox) to review, not for you to run. The candidate is a new file `/etc/sudoers.d/fleet-dsh-control` (root:root, mode 0440, no dot in the name) with exact arguments and no wildcard:

    cayleb ALL=(root) NOPASSWD: /usr/bin/systemctl stop fleet-dsh.service
    cayleb ALL=(root) NOPASSWD: /usr/bin/systemctl start fleet-dsh.service

An independent security read by an agent on the build side (**not a Lane 6 review, do not cite it as one**) recommends changes to the originally proposed four lines: drop `status` (it needs no privilege, and under sudo it would run a pager as root) and drop `restart` (stop plus start already cover it), and use the literal `fleet-dsh.service` form (a bare `fleet-dsh` will not match). It validated the syntax offline (`visudo -c -f` on a scratch copy: parsed OK). Checked on this machine:
- The unit is root-owned (`/etc/systemd/system/fleet-dsh.service`, 0644), runs as `User=fleet` with `NoNewPrivileges=yes`, has no `ExecStartPre/Post`, no drop-ins and no `EnvironmentFile`, and `/var/lib/fleet` is not traversable by uid 1000. As it stands, starting it cannot run code as root.
- The unit is `inactive` and disabled. The live fleet is the user unit `dsh-fleet-consolidated`.

What the review should weigh first: **the rule may be unnecessary.** The record says a polkit rule (`50-dsh-fleet.rules`) already lets cayleb stop, start and restart `fleet-dsh.service` with no password, proven live on 2 Nov at 06:16:52 (`systemctl --no-ask-password stop fleet-dsh` returned 0 while `sudo -n` failed). I could not read the installed polkit rules from this account (`/etc/polkit-1/rules.d` is root-only) and `pkcheck` can't answer from this sandbox, so I cannot re-verify it today. The sudoers rule would add only a root-run path to the same effect, usable without a password by every fleet lane (all run as your user; finding F1, accepted).

What Lane 6 must still check: install-time ownership and mode of the real file and a clean `visudo -c` over the whole tree; that nothing a lane can change feeds the code the unit runs (the existing `dsh-fleet-admin plugin-add` and `install-dsh` verbs can change `/var/lib/fleet/npm`, which is where `ExecStart` lives, but as the `fleet` user); that no root-run hook can ever be added to the unit; whether the legacy unit should be startable at all while the consolidated fleet is live (two hosts on one state); and whether polkit already makes the rule redundant.

### Separate finding for Nox, NOT part of today's list: a stale privileged drop-in is still installed

`sudo -n -l` today (read-only) lists, for cayleb: `(root) NOPASSWD: /usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107` and `/usr/bin/dd of=/run/dsh-fleet-id-cf93107/caller.py conv=excl status=none` (and the same for `identity_snapshot_core.py`), plus `(fleet) NOPASSWD: /usr/bin/env -i … /usr/bin/python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py`. The board records the Team B "V3 identity" block as removed on 1 Nov (message 20261101T162153Z), but it is live. These entries let any process running as your user write a file into that directory as root and run it as the `fleet` user (today the files already exist, so `conv=excl` blocks overwriting; after a reboot clears `/run` the path is open again). I cannot read `/etc/sudoers.d` to see the file name; the V3 block's own notes call it `/etc/sudoers.d/dsh-identity-v3`. It is not on your list and nothing should be removed until Nox decides. If Nox adds it, the removal read-back is `sudo -n -l | grep -c dsh-fleet-id` returning `0`.

## 4. Owner Elevation: not on today's list

The Owner Elevation build (the one-tap approval prompt that feeds one command once) is not built yet. Its design is filed for Nox's ruling; no code exists to run. There is no Owner Elevation step for you today, and nothing in this pack depends on it. The first real elevation run waits for the build and its security review.

## How Nox verifies afterwards (no sudo, from machine state)
- Item 1: `tailscale debug prefs | grep RunSSH`.
- Item 2: `systemctl is-active sshd.service`; `ss -ltn | grep :22`; the hardening file is world-readable (mode 0644), so its content can be read; effective values can be checked with `sshd -T -f /etc/ssh/sshd_config -h <a throwaway key>` as a normal user.
- Item 3: see section 3.
