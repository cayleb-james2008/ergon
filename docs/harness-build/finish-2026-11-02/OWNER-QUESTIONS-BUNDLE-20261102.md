# Questions for Cayleb's next sitting (2 Nov 2026)

These are the seven questions in the plan to move the delegator and its teams into the fleet plugin that are yours to answer and not Nox's. Nox has already answered the other nine. The wording and the recommendations below come from section B9 of `leg3-duet-bridge-design.md` (Part B). Nothing here has been done. No agent deletes anything: when the time comes you run the deletion script yourself, at your own terminal.

## At a glance

| # | Question in a few words | What waits on your answer |
|---|---|---|
| Q6 | Keep voice input? | Nothing |
| Q7 | Keep the old board folder? | Nothing |
| Q8 | Where do the encrypted backups go? | The deletion step only |
| Q9 | The old app's timers | Nothing, but you do one act at the sitting |
| Q11 | Is the older DSH install in "everything else"? | Nothing |
| Q15 | The seven old memory-plugin key files | The deletion step only |
| Q16 | Where does the off-machine copy of the backup key live? | The deletion step only |

Q8, Q15 and Q16 block only the deletion step. The migration and the test that a message wakes the relocated delegator do not wait for them. (The backup upload sits on the road to deletion, so it also waits for Q8 and Q16.)

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

**Recommendation.** Read the timer's prompt first (a copy is in `BOOKINGS.md`). The 16:15 timer has now fired: the schedule file, read at 16:45 ET, shows all three timers as ended. At the quiet point before the switch, you remove any timer that is still active. The fleet profile has no timers, so none can fire after the switch.

**What it blocks.** Nothing. It is a record the deletion checks look for ("what happened to this timer").

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
