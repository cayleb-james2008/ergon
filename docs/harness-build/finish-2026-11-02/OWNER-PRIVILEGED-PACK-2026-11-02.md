# Owner privileged list: verified content pack (2 Nov 2026, prepared 13:55 ET)

For: Cayleb, home between 14:30 and 15:00 ET. Display-and-verify material only. Nothing here was run by any agent. Every fact below was read from this machine today (outis-desktop) with read-only commands; where a fact could not be read, it says so.

## Status at a glance

| # | item | state | needs sudo | what you should expect |
|---|---|---|---|---|
| 1 | Tailscale SSH on | READY | no (you are the Tailscale operator) | one command, one read-back |
| 2 | OpenSSH server | READY WITH A CHANGE (read below) | yes | hardened config first, then start; no remote login until you add a key |
| 3 | sudoers rule for fleet-dsh | HELD FOR SECURITY REVIEW (necessity question first) | yes | do not run; Lane 6 decides whether it is needed |
| 3b | remove stale V3 drop-ins | HELD FOR SECURITY REVIEW | yes | named and verified; runs only after Lane 6 clears it |
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

## 3. sudoers rule for fleet-dsh: HELD FOR SECURITY REVIEW (do not run anything from this section)

**The first question is whether the rule is needed at all.** Lane 6 settles that before any sudoers line exists. If polkit already gives you passwordless stop/start of `fleet-dsh`, this item closes as VERIFIED-NOT-NEEDED (Lane 6's word plus the live read-back at the sitting below). The sudoers line ships only if Lane 6 finds the polkit coverage insufficient.

What the record and the machine say about that (read-only, 2 Nov):
- The unit `fleet-dsh.service` is `inactive` and disabled. It is the retired legacy host. The live fleet is the user unit `dsh-fleet-consolidated`, which you control without any privilege (`systemctl --user`). So day to day, nothing needs root control of `fleet-dsh`.
- The design assumed a standing polkit rule, `50-dsh-fleet.rules`, grants you stop/start/restart of `fleet-dsh`. **That premise is doubtful.** On 1 Nov the Team B V3 note (its section E) recorded `pkcheck` answering `auth_admin_keep` for `manage-units`, meaning NOT authorised without a password. The V3 block then added its own temporary polkit rule, `49-dsh-identity-v3.rules`, and the passwordless stop on 2 Nov at 06:16:52 most likely came from that temporary rule, which is meant to be removed (item 3b). I cannot read `/etc/polkit-1/rules.d` from this account and `pkcheck` can't answer from my sandbox, so **this is unverified**; Lane 6 reads the directory as root.
- If that reading is right, removing the temporary V3 rules (3b) also removes passwordless control of `fleet-dsh`, and the sudoers rule would be the only way to get it back. Given that `fleet-dsh` is retired, Lane 6 may well conclude nobody needs it.

Lane 6, as root: `sudo ls -la /etc/polkit-1/rules.d/` and `sudo grep -l fleet /etc/polkit-1/rules.d/*` (which rule files exist, and does any of them grant `fleet-dsh` start/stop to cayleb).

Live read-back at the sitting, only if Lane 6 asks for it (the unit is inactive, so a stop is a no-op; the answer is whether polkit permits it):

    systemctl --no-ask-password stop fleet-dsh.service ; echo "rc=$?"      # rc=0: authorised without a password. A nonzero rc with "Interactive authentication required": not authorised.

For review only, not for you to run: the candidate is a new file `/etc/sudoers.d/fleet-dsh-control` (root:root 0440, no dot in the name), exact arguments, no wildcard:

    cayleb ALL=(root) NOPASSWD: /usr/bin/systemctl stop fleet-dsh.service
    cayleb ALL=(root) NOPASSWD: /usr/bin/systemctl start fleet-dsh.service

An independent read (**not a Lane 6 review, do not cite it as one**) recommends dropping `status` (it needs no privilege and would run a pager as root under sudo) and `restart` (stop plus start covers it), using the literal `fleet-dsh.service` form (a bare `fleet-dsh` will not match), and found the unit itself cannot turn a start into root code (root-owned file, `User=fleet`, `NoNewPrivileges=yes`, no hooks or drop-ins, `/var/lib/fleet` not traversable by your uid). The offline syntax check (`visudo -c -f` on a scratch copy) parsed OK. What Lane 6 must still check: installed ownership and mode with a clean `visudo -c` over the whole tree; that nothing a lane can change feeds the code the unit runs (the existing `dsh-fleet-admin plugin-add`/`install-dsh` verbs can change `/var/lib/fleet/npm`, as the `fleet` user); that no root-run hook can be added; whether the legacy unit should be startable while the consolidated fleet is live (two hosts on one state).

### 3b. Remove the stale Team B "V3 identity" drop-ins: HELD FOR SECURITY REVIEW (Nox ruling, Case 3 in the security settlements registry)

What is wrong: the V3 block's sudoers drop-in is still installed although the board records it as removed on 1 Nov (message 20261101T162153Z). Verified read-only on 2 Nov at 13:53 ET: `sudo -n -l` lists, for cayleb, `(root) NOPASSWD:` `/usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107` and `/usr/bin/dd of=/run/dsh-fleet-id-cf93107/{caller.py,identity_snapshot_core.py,manifest.txt} conv=excl status=none`, `chmod`, `stat` and `sha256sum` of those files, and `(fleet) NOPASSWD:` a `python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py` run. Today `conv=excl` stops overwriting the three existing files; after a reboot clears `/run`, any process running as your user could write a file there as root and run it as the `fleet` user. The stage directory `/run/dsh-fleet-id-cf93107` still exists (root:root, 3 read-only files). The V3 note's own polkit rule `49-dsh-identity-v3.rules` was installed by the same block and may also still be present (not readable from this account).

The exact removal, taken from the V3 block's own removal section (D) and unchanged (file paths are the V3 block's own: `/etc/sudoers.d/dsh-identity-v3` and `/etc/polkit-1/rules.d/49-dsh-identity-v3.rules`). Run only after Lane 6 has cleared it:

    sudo /usr/bin/rm -f /etc/sudoers.d/dsh-identity-v3 /etc/polkit-1/rules.d/49-dsh-identity-v3.rules
    sudo /usr/bin/visudo -c
    test ! -e /etc/sudoers.d/dsh-identity-v3 && echo SUDOERS-GONE
    test ! -e /etc/polkit-1/rules.d/49-dsh-identity-v3.rules && echo POLKIT-GONE

Post-removal read-back (no password needed for the listing). **Baseline before removal: 2 matching lines. Expected after: 0.**

    sudo -n -l | grep -c 'dsh-fleet-id-cf93107'

The stage directory itself is NOT removed by this item, and nobody `rm -rf`s an unconfirmed stage; its removal is a separate act once its contents are confirmed. Side effect to know: removing the V3 polkit rule takes away the temporary passwordless `fleet-dsh` stop/start, so stopping or starting that retired unit asks for your password again. Nothing running today depends on it.

**If Lane 6 has cleared neither item 3 nor 3b before the window, both stay on the list named and HELD FOR SECURITY REVIEW; nothing is dropped.** Lane 6 owns 3b until the removal read-back lands.

## 4. Owner Elevation: not on today's list

The Owner Elevation build (the one-tap approval prompt that feeds one command once) is not built yet. Its design is filed for Nox's ruling; no code exists to run. There is no Owner Elevation step for you today, and nothing in this pack depends on it. The first real elevation run waits for the build and its security review.

## How Nox verifies afterwards (no sudo, from machine state)
- Item 1: `tailscale debug prefs | grep RunSSH`.
- Item 2: `systemctl is-active sshd.service`; `ss -ltn | grep :22`; the hardening file is world-readable (mode 0644), so its content can be read; effective values can be checked with `sshd -T -f /etc/ssh/sshd_config -h <a throwaway key>` as a normal user.
- Item 3: see section 3.
