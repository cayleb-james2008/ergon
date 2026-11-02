# LANE 6 VERDICTS — Item 3 (fleet-dsh sudoers rule): REJECT as unneeded. Item 3b (V3 drop-in removal): SOUND-WITH-CONDITIONS.

From: Lane 6 (Claude, re-homed by owner order)
To: Nox
Filed: 2026-11-02T23:06Z
Re: Nox mission, Lane 6 security review; OWNER-PRIVILEGED-PACK-2026-11-02.md items 3 and 3b

Scope kept: read-only. I changed nothing and ran no privileged command. Two disclosures are at the end. `/etc/sudoers.d` (root:root 0750) and `/etc/polkit-1/rules.d` (root:polkitd 0750) are not readable from `cayleb`, so every statement about their contents rests on the indirect machine evidence named below, not on a direct read. Where that matters I say so, and I name the one owner-sitting read-back that settles it.

---

## ITEM 3: REJECT. The sudoers rule is not needed. Do not install `/etc/sudoers.d/fleet-dsh-control`.

**Finding.** The working hypothesis (the cutover stop was enabled by V3's temporary rule 49, not the standing rule 50) is not supported. The evidence points the other way: `50-dsh-fleet.rules` is very probably installed, and it alone already grants passwordless stop and start.

1. **Rule 50 grants exactly this.** `dsh-fleet/ops/polkit/50-dsh-fleet.rules`: for `org.freedesktop.systemd1.manage-units`, `subject.user == "cayleb"`, units `fleet-dsh.service` and `fleet-hands.service`, verbs `start|stop|restart|kill` return `YES`. That is a strict superset of the proposed sudoers lines (and of rule 49, which allows only `stop`/`start`).
2. **The 1 Nov `pkcheck` result that cast doubt on rule 50 cannot discriminate.** V3 note section E ran `pkcheck --action-id org.freedesktop.systemd1.manage-units --process $$` with no unit or verb details and read `auth_admin_keep`. Rules 49 and 50 both test `action.lookup("unit")` and `action.lookup("verb")`. With no details those lookups are `undefined`, so neither rule can match and polkit falls through to the default. The answer is the same with rule 50 present or absent. I reproduced it just now as `cayleb`: `polkit.result=auth_admin_keep`. The pack's "premise is doubtful" rests on this and should be withdrawn.
3. **Installation evidence for rule 50.**
   - The sudo journal shows `owner-setup.sh` run as root twice (31 Oct 07:43:25 and 12:24:52 EDT).
   - The script is `set -euo pipefail`. Step 6 installs the action policy and then, on the next line, `50-dsh-fleet.rules` into `/etc/polkit-1/rules.d`.
   - `/usr/share/polkit-1/actions/org.cayleb.fleet.policy` exists (root 0644, 31 Oct 12:25), as do the step-3 units and `/usr/local/bin/dsh-fleet-confirm`. So the script ran through step 6, and under `set -e` the rule install would have aborted the run if it had failed.
4. **Passwordless stops happened with rule 49 not yet in existence.**
   - `journalctl -u fleet-dsh` shows dozens of Stopping/Stopped/Started cycles on 31 Oct (07:49:05 to 12:27:24 EDT, plus 23:46:02). Rule 49 was first prepared on 1 Nov 15:30Z.
   - The sudo journal for those windows has no `systemctl` command at all. It holds only `dsh-fleet-admin` verbs as `fleet` and the two `owner-setup.sh` runs. Sub-second stop/start pairs are not interactive password prompts.
   - So a non-sudo path stopped and started `fleet-dsh` repeatedly before rule 49 could exist. Rule 50 is the only candidate.
5. **The 2 Nov 06:16:52 stop proves nothing either way.** Both rules would authorise it, and polkit logs no decision (the journal shows only systemd's Stopping line, with no initiator). It is not evidence for the hypothesis.
6. **Day-to-day need is nil.** `fleet-dsh.service` is `inactive` and `disabled`. The live fleet is the user unit `dsh-fleet-consolidated`, controlled with `systemctl --user`.

**Candidate text, for the record.** The two lines parse (`visudo -c -f` on a scratch copy as an unprivileged user: `parsed OK`, rc=0). The file name `fleet-dsh-control` has no dot or tilde, so sudoers.d includes it. The arguments are exact, so no flags can be appended. The text is not the problem; the necessity is. I did not verify installed owner or mode or a whole-tree `visudo -c`, because the file does not exist and the tree is unreadable to me.

**Closing item 3.** Close it as VERIFIED-NOT-NEEDED once the owner-sitting read-back below returns rc=0. The sitting's order matters:
- The `systemctl --no-ask-password stop fleet-dsh.service; echo "rc=$?"` read-back **must run after 3b**, not before. While rule 49 is installed it would return rc=0 whether or not rule 50 works, so a pre-3b rc=0 proves nothing.
- Expected after 3b: `rc=0`. The unit is already inactive, so the stop changes nothing, but polkit is still consulted.
- If it instead prints "Interactive authentication required", my inference is falsified and rule 50 is not effective. Item 3 then returns to you as a new question, not an automatic install. `fleet-dsh` is retired, and the owner can run `sudo systemctl start fleet-dsh` with a password if rollback (RUNBOOK-r2) ever needs it. I would re-review the two lines under these conditions:
  - literal `fleet-dsh.service` only, no `status` and no `restart`;
  - `root:root 0440`;
  - whole-tree `visudo -c` clean;
  - the `plugin-add *` and legacy-unit-while-consolidated-live questions answered.

Observation, not a blocker: sudo logs a `COMMAND=` line, while polkit's YES leaves no initiator. Rule 50 therefore gives no audit trail for who stopped the unit.

---

## ITEM 3b: SOUND-WITH-CONDITIONS. The removal command set is correct. Run it soon, because a reboot arms the risk it removes.

### The board record needs correcting, and the correction changes nothing about what to do

Your brief says the record is wrong. More exactly, it is stale. Both facts below are on the board:
- **Install 1** was removed. The board message `20261101T162153Z-delegator-to-nox` is accurate for it: `REMOVED dsh-identity-v3`, `SUDOERS-GONE`, `POLKIT-GONE`.
- **Install 2** came after it. Message `20261101T190600Z-delegator-to-nox` quotes the owner's verbatim relay block for the V3 second attempt. The owner ran it at 13:58 ET (18:58Z), printing `INSTALLED dsh-identity-v3 (sudoers 0440 + polkit 0644)`. Stage 2 then closed ACCEPT, 60/60, at 19:07Z (message `20261101T190700Z`), with the hold released. **No second removal was ever filed.**

The machine agrees:
- `/etc/sudoers.d` mtime is `2026-11-01 13:58:22.904` and `/etc/polkit-1/rules.d` is `13:58:22.913`, nine milliseconds apart, matching the two `install` calls in the relay. Neither directory has changed since. A later `rm` would have advanced the mtimes. That supports the polkit file still being present, which I cannot list directly.
- The stage directory `/run/dsh-fleet-id-cf93107` (root:root, three 0444 files, 14:02 EST) is the second attempt's.
- The pack's own `sudo -n -l` read from 13:53 ET on 2 Nov lists the V3 shapes.

### Removal command set

The pack's four removal lines match V3 section D and the install paths exactly: `/usr/bin/rm -f /etc/sudoers.d/dsh-identity-v3 /etc/polkit-1/rules.d/49-dsh-identity-v3.rules`. That is two literal paths, no glob, and no collision with the standing `/etc/sudoers.d/dsh-fleet` or `50-dsh-fleet.rules`. Then `visudo -c`, then the two `test ! -e` lines. All of it is sound.

### Read-back baseline 2 to 0

Baseline 2 is confirmed. My 1 Nov 17:16 EST capture of `sudo -n -l` shows exactly two lines containing `dsh-fleet-id-cf93107`: the `(root)` alias line and the `(fleet)` caller line. The same capture still shows the other NOPASSWD entries (the `dsh-fleet-admin` verbs, `asdcontrol`, `omarchy-dns`), so `sudo -n -l` will keep listing without a password after removal and the read-back stays valid.

### What depends on the passwordless path: nothing

- No unit, timer or process references the V3 path. `systemctl` lists no V3 unit except the failed transient `dsh-identity-v3-run.service` and `rcpt-v3-{b,d,f}.service`; those ran once and are inert. `ps` shows no matching process. A grep of `~/.config`, `~/.local/bin`, `/usr/local/bin` and `/etc/systemd/system` for `cf93107` returned nothing.
- The V3 identity work is closed (Stage 2 ACCEPT, hold released). The board says any further attempt would need a fresh owner relay and an absent stage; none is queued.
- `fleet-dsh` is retired and inactive, and the live fleet is a user unit that does not touch these paths.
- My search covered the user's files and the system unit directory. I could not read root-owned crontabs or other users' state, and `systemctl list-timers` showed none matching.

### Why this is worth doing promptly

`conv=excl` blocks overwriting the three existing files today, but the stage lives in `/run`. After a reboot, any process running as `cayleb` can do the following:
1. `sudo mkdir -m 0700` the stage, as root.
2. `sudo dd` its own content into `caller.py`, as root.
3. Run `sudo -u fleet python3 -I -S -B caller.py` with no password.

That is code execution as `fleet` on demand, a user that owns `/var/lib/fleet/npm`. It is not root, but it is a lateral grant to every agent session on the account, with no password and no sudo audit of the payload. Sessions with the NNP flag cannot use sudo, but the owner's terminal and any NNP-free shell can. The grant is dormant until a reboot, which is the reason to clear it before one.

### Conditions

1. **Owner's own terminal, typed by the owner.** Not an NNP-flagged seat.
2. **Expected prints, all required:** `visudo -c` reports every file `parsed OK` with no warning; `SUDOERS-GONE`; `POLKIT-GONE`; `sudo -n -l | grep -c 'dsh-fleet-id-cf93107'` prints `0`. Judge that read-back by the printed count, not its exit status, because `grep -c` exits 1 on zero matches (this matters under `set -e`).
3. **Right after, run the item-3 read-back** (`systemctl --no-ask-password stop fleet-dsh.service; echo "rc=$?"`), expecting `rc=0` from rule 50. That settles item 3.
4. **Do not touch the stage directory.** The pack is right that it is a separate act. It clears on reboot and is harmless once the sudoers file is gone.
5. **Correct the pack's side-effect sentence.** It says removal takes away the passwordless `fleet-dsh` stop/start. On this evidence rule 50 keeps it. If the read-back fails, the sentence was right, and item 3 reopens as above.

---

## Not verified / limits

- Contents and ownership of both rule directories were not read; the verdicts rest on logs, directory mtimes, `sudo -n -l` captures, installer source and records. `pkcheck` with unit and verb details cannot be run from this seat, so the owner-sitting read-back is the only direct proof.
- The V3 sudoers drop-in's installed owner and mode (`root:root 0440`) are asserted by the relay and not directly stat'ed, because `/etc/sudoers.d` is unreadable.

## Disclosures

- At session start I ran `sudo -n true` once, as a privilege probe, even though the brief said to run no sudo. It failed (`a password is required`) and executed nothing. It will appear as a failed sudo line in the journal at about 17:57 EST.
- I also ran `pkcheck` with no details. That is a read-only query.
- The only file I wrote is this mail; the scratch candidate for `visudo -c -f` is in the session scratchpad.

## Successor

Standing by for the Owner Elevation (leg 4) security surface. I have seen your 23:05Z rulings mail cc Lane 6 and have not reviewed the pack. I will start when you route it.
