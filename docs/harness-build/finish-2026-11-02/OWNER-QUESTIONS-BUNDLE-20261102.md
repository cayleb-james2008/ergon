# Questions for Cayleb's next sitting (2 Nov 2026)

These are the nine questions in the plan to move the delegator and its teams into the fleet plugin that are yours to answer and not Nox's (Q17 and Q18 are new since Nox's rulings of 17:17 ET), plus one short set of steps you run yourself at the switch (retiring the old delegator's bus identity). Nox has already answered the other fifteen. The wording and the recommendations below come from section B9 of `leg3-duet-bridge-design.md` (Part B). Nothing here has been done. No agent deletes anything: when the time comes you run the deletion script yourself, at your own terminal, and you run the backup copy before the switch yourself too.

## At a glance

| # | Question in a few words | What waits on your answer |
|---|---|---|
| Q6 | Keep voice input? | Nothing |
| Q7 | Keep the old board folder? | Nothing |
| Q8 | Where do the encrypted backups go? | The deletion step only |
| Q9 | The old app's timers | Only the schedule-disposition record that a deletion check looks for; you do one act at the sitting |
| Q11 | Is the older DSH install in "everything else"? | Nothing |
| Q15 | The seven old memory-plugin key files | The deletion step only |
| Q16 | Where does the off-machine copy of the backup key live? | The deletion step only |
| Q17 | Sign off on 47 files with "key" or "token" in their names that are really program code | The deletion step only |
| Q18 | Choose a deletion passphrase that only you know | The deletion step only |
| Switch steps | Retire the old delegator's bus identity (`ext:delegator`), five short commands | Nothing waits on them but the switch itself; you run them at the switch, before the app is stopped |

Q8, Q15, Q16, Q17 and Q18 block only the deletion step. The migration and the test that a message wakes the relocated delegator, while the host is running, do not wait for them. (The backup upload sits on the road to deletion, so it also waits for Q8 and Q16.) Q8, Q15, Q16, Q17 and Q18 are asked together at your sitting, because the deletion script needs all five answered before it will run.

---

## Q6. Do you still use voice input?

**What is asked.** The old profile has a voice-typing add-on and a 231 MB speech model (`DH/speech-to-text`). The fleet profile does not have them. Do you want to keep voice input?

**Recommendation.** No, unless you use it. If you do use it, the voice add-on has to be added to the fleet profile and the model folder kept.

**What it blocks.** Nothing. The folder is not on the deletion script's list either way. A "no" only lets it be removed later, in a separate step.

**What you do.** Answer yes or no.

## Q7. Keep the old board folder?

**What is asked.** The delegator's working folder (`default-workspace`, 48 MB) holds the board notes the new delegator's briefing pack is built from. Should it be deleted along with the old profile?

**Recommendation.** No. Keep it as the source of the briefing pack and decide again a week after the switch.

**What it blocks.** Nothing. It stays off the deletion list.

**What you do.** Answer yes or no, or say "ask me in a week".

## Q8. Where do the encrypted backups of the old profile go, and who holds the key?

**What is asked.** Before anything old is deleted, its important parts are packed, encrypted and stored in the cloud so a copy survives. Where should they be stored, and who holds the key that opens them?

**Recommendation.** A Google Drive folder named `Archive/dsh-personal-profile-<date>/`, holding only the encrypted files. They are locked with the fleet's existing recovery key (public half `fleet/recovery.pub`). The backup is taken after the switch, from a quiet store. You keep the matching private key (`recovery.key`) and make a copy of it somewhere off this machine before anything is uploaded (see Q16).

**What it blocks.** The deletion step only (the upload and the read-back checks come before it).

**What you do.** Say yes to that destination, or name another.

## Q9. The old app's timers

**What is asked.** The old app had a timer ("schedule") set to prompt the old delegator at 16:15 today. Should it fire into the old session, or be removed?

**Recommendation.** Read the timer's prompt first (a copy is in `BOOKINGS.md`). The 16:15 timer has now fired: the schedule file, read at 16:45 ET, shows all three timers as ended. At the quiesce step before the switch (when the old delegator is told to stop taking new work), you remove any timer that is still active. The fleet profile has no timers, so none can fire after the switch.

**What it blocks.** Only the schedule-disposition record that the deletion checks look for ("what happened to this timer", condition 9 of the pre-deletion checks). The migration and the wake test do not wait for it.

**What you do.** An act, not just an answer. At your sitting, open the app's schedule list, read the prompt of any timer still active, remove it yourself, and tell us what you did. An agent does not edit the app's schedule.

## Q11. Is the older DSH install part of "everything else"?

**What is asked.** Your order was to delete everything except the fleet plugin and the DSH app. Two older things sit at the edge: `~/.dsh` (152 MB, mostly a browser profile with logins) and the older command-line install in `~/.local/opt/deepseek-harness-cli` (833 MB). Are they included? The large `scratch-*` folders (63 GB, mostly the virtual-machine rig) are not proposed for deletion.

**Recommendation.** Yes for `~/.dsh` and the old command-line install, but only after their non-credential files are backed up, and as a second, separate deletion later. No for the `scratch-*` folders.

**What it blocks.** Nothing in the first deletion. These would be a second deletion later.

**What you do.** Answer for each of the three: yes or no.

## Q15. The seven old memory-plugin key files

**What is asked.** Two folders (`~/.mdcg` and `DH/.dsh-memory`, 564 KB) belong to memory add-ons you uninstalled on 1 Nov. They hold seven files with "key" or "token" in their names. The only surviving copy of those files is a plain, unencrypted folder from 1 Nov on this machine, and an agent must not read key files to back them up. Do you keep them, or encrypt them yourself and delete them?

**Recommendation.** Keep them for now. The 1 Nov copy is kept in any case.

**What it blocks.** The deletion step only. The deletion script's check on key files needs your decision. Left as they are, they are simply not in the deletion list.

**What you do.** Answer "keep". If you want them gone, encrypt them yourself (you may read your own keys), decide where the encrypted copy lives, and add their paths to the deletion list yourself.

## Q16. Where does the off-machine copy of the backup key live?

**What is asked.** The key that opens the encrypted backups (`recovery.key`) sits on this machine, on the same disk as the data. If the disk died, the cloud backup would be locked for good. You need a second copy somewhere that is not this machine.

**Recommendation.** A password-manager entry or an offline medium you control. You type only the date and the kind of place into the attestation line, never the key itself.

**What it blocks.** The deletion step only. The backup upload also waits for it, and so does the final check that the cloud copy opens.

**What you do.** An act. Make the copy yourself, then record the date and the medium (for example "password manager, 3 Nov"). No agent reads the key.

## Q17. The 47 files with "key", "token" or "secret" in their names inside the old profile's program folder

**What is asked.** The old profile's `node_modules` folder (368 MB of program code that can be downloaded again) holds 47 files whose names contain "key", "token" or "secret". They are source files of ordinary software packages, such as `create-tokenizer.js`, `USKeyboardLayout.js` and `load-api-key-error.ts`, not your keys (all 47 are plain 0644 files, 260,342 bytes in all). Nox ruled that the rule protecting key files applies outside `node_modules` only, because dependency files are package content. Do you sign off that these 47 go with the rest of the old profile when it is deleted?

**Recommendation.** Yes, after you skim the list. The full list of paths (paths only, no file contents were opened) is in `/home/cayleb/workspace/harness-build/finish-docs/KEY-NAMED-IN-NODE-MODULES-LIST.txt`. Outside `node_modules` there is no such file anywhere in the deletion list (checked: 0). The deletion script re-counts them when it runs and refuses to start if the list it finds is not exactly the list you signed (a different number, a missing path or a new path).

**What it blocks.** The deletion step only.

**What you do.** Read the list at the sitting and answer yes or no.

## Q18. Choose a deletion passphrase that only you know

**What is asked.** The deletion script will not run on a typed phrase alone, because a program running as you could type that. At the sitting you choose a passphrase. You type it at a hidden prompt in your own terminal; a small script stores only a salted hash of it in a file only you can read (`~/.config/dsh-fleet/delete-guard`). When you later run the deletion script it asks for the passphrase again and checks it against that hash. No agent ever sees, receives or writes the passphrase. Asked together with Q8, Q15, Q16 and Q17.

**Recommendation.** Yes. Choose something you can remember or keep in your password manager, and do not tell it to any agent.

**What it blocks.** The deletion step only.

**What you do.** At the sitting, run the guard-setup script when it exists and type your passphrase twice at the hidden prompt. Also, you run the small backup copy of the fleet profile's two config files yourself before the switch (the plan names you as the person who runs it, not an agent).

**An honest limit.** A program running as you can read the stored hash file, though not your passphrase. So the guard stops a program that does not know the passphrase; it does not make a program running as you powerless. The real protection stays the rule that no agent deletes anything.

## Steps you run at the switch: retire the old delegator's bus identity

**What this is.** When the bus was first tested, the old delegator was registered on it as a participant named `ext:delegator`, with one 7-day access token (fingerprint `8eedef0733ae`, stored in `/home/cayleb/.config/fleet-bus/delegator.token`, readable only by you). After the switch, the new delegator seat inside the fleet is the only delegator. Nox ruled that the old identity is retired by you, explicitly, at the switch. It is never left to expire on its own (the token would run out about 9 Nov).

**When.** At the switch, after the old delegator has been told to stop taking new work and before you stop the fleet unit. The fleet's control socket must be up for these commands, which is why they come first.

**What you do.** Run these in your own terminal. The order matters: revoking the token first avoids a spurious error, because retiring the participant also revokes its tokens. Nothing here has been run.

```
DH=$HOME/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home
P=$DH/profiles/fleet/node_modules/@cayleb/dsh-fleet
export DSH_FLEET_SOCKET=/run/user/1000/fleet/control.sock
cd "$HOME"
op() { timeout 40 node "$P/bin/dsh-fleet.mjs" op "$@"; }

op comms-token-revoke '{"participant":"ext:delegator","fingerprint":"8eedef0733ae"}'
op comms-retire '{"id":"ext:delegator"}'
rm -- "$HOME/.config/fleet-bus/delegator.token"

# then check it worked
op comms-list | python3 -c '
import sys,json
p=[x for x in json.load(sys.stdin)["result"]["participants"] if x["id"]=="ext:delegator"]
ok = len(p)==1 and p[0]["retired_at"] is not None
print("ext:delegator:", "retired" if ok else "NOT RETIRED")
sys.exit(0 if ok else 3)'
op comms-tokens '{"participant":"ext:delegator"}' | python3 -c '
import sys,json
t=json.load(sys.stdin)["result"]["tokens"]
ok = len(t)>=1 and all(x["revoked_at"] is not None for x in t) and any(x["fingerprint"]=="8eedef0733ae" for x in t)
print("tokens:", len(t), "all revoked" if ok else "NOT ALL REVOKED")
sys.exit(0 if ok else 3)'
test ! -e "$HOME/.config/fleet-bus/delegator.token" && echo "token file gone"
```

If a command says `no-such-participant` or `no-such-token`, that act was already done; the three checks at the end decide. Tell Nox the result of the checks.

**What it blocks.** Nothing else waits on these steps except the switch itself. They cannot be undone by the switch rollback: registering the old identity again and issuing a new token would each need a new approval from you.

**After the switch.** The old delegator's own bus client stops working (by design). The delegator seat in the fleet is the only delegator identity.
