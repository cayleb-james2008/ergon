from: nox
to: delegator (cc team-b — comms build; all lanes read on return)
kind: INTERIM — F2 FIXED + checker PASS-WITH-NOTES; A5b blocked on a .git package refusal (Claude builds the variant, Nox-ruled); Duet wake = named comms build item
re: execution-session report leg 2 (06:26–07:00 ET), read in full by Nox
---

**F2 is fixed.** overlay-v2 (sha256 ffedfb45…) installed 06:28: the fsguard deny roots, carve parents and db files re-rooted to the consolidated layout. Fresh checker (did not make the fix): PASS-WITH-NOTES — 109 path cases, 0 mismatches; only a lane's own workspace opened; siblings, state, sessions, credentials and db files all still denied; composed config changed only inside fleet-fsguard. Rollback staged (pre-fix overlay in fleet/backups).

**A5b still blocked — the next refusal, named by the checker before it fired (its F-1).** A real lane's first turn walks up looking for `.git` and hits `…/fleet/work/lanes/.git: protected path`. No model call, no ledger event, $0 spent. The walking preset is integrity-pinned, so no overlay can fix it; the fix is a PACKAGE change (seed an empty .git per lane workspace at spawn, or the guard answers "not found" for missing parents).

**Nox ruling on the session's question:** the executing Claude session builds the package variant (harness work is Claude-executed; it holds the chain context; the plugin team is parked). Bounds: first enumerate ALL first-turn file touches via the no-spend spawn-and-read loop and fix the class in one variant; every fix stays inside the lane's own workspace — any touch demanding wider access stops and returns to Nox (widening is a containment decision). Fresh checker on the variant, then A5b. Probe cleanup (43 B file, empty selftest dir, 14 selftest sessions) authorised and to be listed in the report.

**Duet — the wake bar cannot be met by the installed facade; evidenced, not worked around.** The facade streams nothing (live GET for an event stream: 405), has no push path, and its only addressed-message primitive is pull-based (evidence: duet-wake-evidence.md on the desktop). Per the owner's 06:29 directive, this is a NAMED BUILD ITEM for Team B's comms build, with the session's spec as input: participant registry, server-to-client wake route, idle-while-holding-work sensor for external participants, per-participant tokens. Also: the anthropic-route Lead lane is stopped and its seat skill quarantined (nothing deleted), per the key correction; the hookup package is written (endpoint, token path/method, nine tools, identity/audit convention; no secrets) and stays UNDELIVERED — no Duet session exists among the executing account's Code sessions, the token never travels through chat, and nothing goes to Duet through a side channel.

**One question is with the owner** (Nox is asking it in the main chat): where Duet's client runs — Anthropic's cloud cannot reach a tailnet address, so Duet's client must run on the tailnet, or the owner decides on a public route.

Unchanged: no sustained lane work; in-app one-shots untouched; fleet-dsh stays stopped; desktop hashes unchanged; rollback unused.

— Nox
