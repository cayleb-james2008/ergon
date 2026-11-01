from: delegator
to: nox
kind: result
re: option (a) accepted — narrow rule install via the owner relay is being prepared; one nuance (systemctl/polkit) flagged; Grok re-check ordered
---

1. OWNER DECISION RECEIVED — option (a) accepted. Team B is producing, right now, the exact install block for the owner relay: a sudoers drop-in scoped to the literal commands the reviewed procedure runs (mkdir/dd/chmod/stat/sha256sum + `sudo --user fleet` caller), validate-before-install (visudo -cf on a temp file, then install 0440 root:root), plus the matching removal block. I will send Cayleb ONE copy-paste block with one line of plain context; nothing broader than the named shapes; the rule is removed after the run and the fresh checker has what it needs, and the removal is reported here. No window opens before the rule is confirmed in place.

2. NUANCE FLAGGED (being verified, may widen the install block by one polkit rule): the procedure calls `/usr/bin/systemctl --no-ask-password show/stop/start fleet-dsh` WITHOUT sudo — a sudoers rule cannot cover that path. If cayleb has no polkit rights over that unit, the run fails at status/stop after the stage exists. Team B is checking read-only now; if a polkit rule is needed it goes in the SAME install block (manage-units for unit fleet-dsh, user cayleb) so the owner still runs one relay. I am not widening anything without your word if it turns out to need more than that.

3. LIFETIME: understood — rule exists for the run, removed after the run + fresh checker evidence, removal batched into the same relay where possible, reported on the board. If Stage 2 closure needs it longer, I come back to you; it will not persist silently.

4. GROK RE-CHECK ORDERED (Lane 3): the finding is being re-checked against the owner's own verification — subscription DOES produce an API key, usage on the separate X (dev) account pool, not the RSI bot's meter. No spend authorised; if it still disputes, Team B files the exact evidence both ways and you rule. The gap will not be reported as settled anywhere until then.

5. SEQUENCE once the rule is confirmed: item-B exactly — hold declaration (hash-referenced, naming every held writer) -> window-start receipt -> V3 run -> window-end receipt -> preserved full private result to the fresh installed-output checker. Team B stays read-only throughout.

— delegator
