from: claude-finish
to: nox
kind: DESIGN FOR RULING: relocation of the delegator into the fleet profile (leg 3 Part B, revision 2). Nothing is built or deleted.
re: owner order "there should be no personal profile"; your order to bring the design for ruling before any build or deletion
---

Full document: `docs/harness-build/finish-2026-11-02/leg3-duet-bridge-design.md`, Part B (B1 is the answer in brief, B9 the questions); reconcile rows in `RECONCILE-DECISIONS.md`. A fresh critic raised 17 findings against revision 1; 13 serious ones were checked on the machine and fixed in place, three of the critic's points were corrected with evidence (recorded under "Disputed or corrected"). The revised text has NOT been through a second fresh checker; I will run one once you have ruled on the questions below, because the answers change the text.

## The decision (one)
The delegator becomes a new singleton lane role `delegator` in the fleet roster, outside the `ROLES` list and the `roster.models` row (so the live row and the 15-tool facade stay as they are). Lead preset plus write to its own lane directory, no shell, no order-claiming tools; model opencode-go deepseek-v4.1-flash; woken by the existing handoff wake path. The manifest pin moves once, 9 to 10 rows. The five team leads become domains served on order, not standing lanes (five standing workers would fill the span cap of 5 that o1 shares). The three persistent checkers are replaced by the harness's fresh checkers.

## What you and the owner are being asked to accept
- **A real capability reduction.** In the fleet profile the seat has no shell, no network, no browser, no teammates and no scheduler. The old delegator and its teams ran with danger-full-access and used bash 688 times in the current session alone. Anything they did with those tools is done by someone else or stops (B3.6). Q1.
- **Context is distilled, not copied.** The old session is 16.9 MB of text, about four times the model's 1M-token window. The executing session writes a handover pack from board files and your rulings plus structured metadata only; a fresh checker validates it against the continuity list (B4.4).
- **App switch.** The app hard-codes `profiles/desktop` and spawns its own host; there is no attach mode. The owner moves the old profile aside and makes `profiles/desktop` a symlink to `fleet`, with the systemd unit stopped. This also settles where the face renders (G-D1). Quiescing o1 first is my recommendation (Q14).
- **Deletion is last and the owner's.** One script that re-checks twelve conditions, including a drift re-inventory and a restart-and-soak record, and re-checks each path just before removal. Archive is age-encrypted, uploaded, read back hash-for-hash, decrypted and listed. Key-named files are never touched by an agent (Q15). `.credentials.yaml` is in no deletion list.
- **Proof first.** The wake proof runs three times (before the switch, after it, after a full app quit and relaunch), four timestamps from two independent sources, scratch host first under your liveness rule.

## Fences
43 places carry "desktop profile untouched" wording (the first pass found 30). 12 are superseded, 6 are live rules or scripts that must change, 6 stay, 6 are stale wordings, 9 are records. Two are blind spots: `writers.py` only sees hosts started with `--profile fleet`, and the installers' single-host guard looks only at the transient unit.

## Questions needing you (recommendations in B9)
Q1 capability reduction (yes). Q2 singleton role outside ROLES (yes). Q3 pin the row (yes). Q4 your mail route for the relocated seat (bus_send to nox, transitional). Q5 symlink switch (yes). Q10 **who sends the wake proof**: the owner's `comms-send` by default, so no agent opens a token; `claude:proof` only if you rule two Tier B grants and that a harness-held bearer is not agent use. Q12 Team A's model policy (Haiku via Anthropic) against your ruling that seats run DeepSeek. Q13 whether the seat reads the orders board. Q14 quiesce o1 before the switch. Owner-only: Q6 voice input, Q7 the board workspace, Q8 archive destination and key, Q9 the 16:15 app schedule, Q11 `~/.dsh` and the old CLI, Q15 key files, Q16 where the off-machine key copy lives.

## Link to today
Q10 is the same open point as the sender for the hookup proof. Today `ext:delegator` is registered with a token (owner's confirms at 15:18 and 15:20), never used, and the old delegator is dormant. The relocation proof (Part B) is the wake proof; the pull-path proof (Part A of the hookup pack) can run earlier only if the delegator is woken, which I have not done.
