# Gate report: dsh-fleet design (Cayleb's harness as one DeepSeek Harness plugin)

**To:** Nox, for Cayleb's approval. **From:** Claude Code, outis-desktop. **Date:** Fri 30 Oct 2026, about 20:40 ET.
**Gate:** design only. **No build work has started.** XIRYN was never touched; Stage 0 was cancelled before any action.

## Verdict

**The design is ready for Cayleb's grade.** It is v1.2, after two fresh-context check rounds.
- **Round 1.** Four checkers attacked v1.0: two citation verifiers (124 citations checked), one brief reviewer and one security skeptic. They returned 4 critical, 11 high, 20 medium and 21 low issues, and v1.1 fixed all of them.
- **Round 2.** Two checkers attacked v1.1: one on containment and the kill switch, one on consistency with the brief. They returned 21 concrete breaks (2 critical, 6 high, 7 medium, 6 low), and v1.2 applies all 21.
- **Not re-checked.** Per Nox's rule there is no third round, so v1.2's own edits have not been re-checked by a fresh checker. They close only what round 2 named.

## What is in the design

The design is `fleet-plugin-design-v1.2.md`, in sections:
1. **§0 Environment verification**, plus 5 security flags.
2. **§1 The shape.** `@cayleb/dsh-fleet` is one npm tarball that DSH treats as one **bundle**. It mounts and tweaks about 15 official DSH plugins, and it adds 11 new modules where nothing in the ecosystem fits. It also ships a reviewed one-time owner setup script and two small cayleb-side services: hands and liaison.
3. **§2 Ecosystem inventory.**
   - Coverage: about 60 official packages, the 7,723-entry community index, about 1,310 DSH npm packages, and Cayleb's Ergon work.
   - Each package gets a fit verdict, the exact tweak, and the tweak mode.
   - Overrules of a survey reader's verdict are flagged ⚑ with reasons.
   - Adopted include:
     - the DSH voice input plugin (local SenseVoice, model pre-seeded, no downloads);
     - agent presets, sandboxed bash, terminals, skills (lessons never as skills), session query, Playwright (vendored so it runs sandboxed), and token meter and stats.
     - Two third-party plugins are vendored after review: `dsh-telegram`, stripped down with Tier A nonces added, and `dsh-budget`.
   - Rejected for the fleet include Agent Teams (pattern only), Auto review, the cua and computer-use providers, the stock `workspace-changes` row, and session-search.
4. **§3 Composition map:** one install, many components, one behaviour, with the mission flow end to end.
5. **§4 Component designs**, each against the real Cordis/DSH APIs with file:line citations.
6. **§5 Comms evaluation and recommendation.**
7. **§6 What ports from spec v2 unchanged,** with a where-it-lives table, and what changes because this is a plugin.
8. **§7 Staged build plan.**
   - Ten stages (0–9). Each closes on a named `stageN-check.sh` with machine-checkable subtests, run by a fresh-context checker. Nox judges Tier-2 stages 3, 5, 8 and 9.
   - Stage 9.3 is the mission done-when: Nox posts an order → the Lead claims it atomically → a Builder executes it → a fresh Checker verifies it → the report reaches Nox and Cayleb's phone. The kill switch and action log are demonstrated there too.
9. **§8 Batched decisions and owner acts.**
10. **§9 Risks.**
11. **§10 Check dispositions.**
12. **§11 Known limitations**, stated plainly.

## Comms recommendation (final)

**Option A: the spec v2 §2.5.2 board, built into the plugin as its own transport (fleet-bus).**
- **What it is:** a tailnet-only listener the plugin owns, with per-principal tokens and JSON plus MCP routes. It is the single record.
- **Evidence:** it scores 5/5 on custody and delivery semantics (leased atomic claims, idempotency, seq and ack). The git board's measured data shows that pull-and-claim works and pushing into chats fails (7 of 7 and 5 of 13 failed).
- **Nox** connects natively from `muse`.
- **The git `team` board** stays as fallback and migration bridge only.
- **Claude Duet** is pull-only and cannot be pushed to by any design. Phase 1 is a liaison with no public exposure. Phase 2 is gated on a 2-minute probe by Cayleb, then his decision on a spec §2.2 exception.
- **Crosswalk is not adopted.** It has no leased claims, its operator can read private content, and it is one anonymous operator about 7 weeks old. **This revises Cayleb's 17:33 bus ruling and needs his grade.**

## Decisions for Cayleb (one grade)

1. Approve the design.
2. Comms: Option A plus the Duet probe; no Crosswalk.
3. Containment: a separate `fleet` user, DSH as a hardened system unit, bots in a masked offline sandbox, kill via systemd and polkit, and a one-time sudo setup.
4. Tier B approvals are a desktop confirm plus his password at a polkit prompt. The spec v2 "Nox relays" path is dropped, because relayed approvals never count.
5. Auto review stays off for the fleet.
6. Provider and caps: an Anthropic API key at Stage 4, with $5 a day and $0.50 per drill order proposed.

## What could not reach the bar (stated plainly)

- **Nothing has been built or run on a live DSH.** Every API claim was read in the pinned source, and fresh checkers verified 124 citations, but the stage checks are the real proof.
- **Unverified until the named stage runs:**
  - HTTP from `muse` to the tailnet listener (Stage 5.1);
  - whether Duet can use custom connectors at all (Stage 6.1);
  - computer-use-linux on Hyprland 0.56.2 (Stage 8.2);
  - how a budget-blocked stream surfaces in the agent loop (Stage 3.5).
- **DSH is an unaudited alpha.** Containment is same-kernel. Hands runs as Cayleb, so its self-protection beyond the namespace and outright refusals is best effort. The Tier B password prompt is as strong as his desktop session. See §11.
- **The v1.2 edits were not re-checked by a third round,** per Nox's instruction.

## Files

- Design: `docs/harness-build/fleet-plugin-design-v1.2.md` on the ergon `team` branch, and locally at `~/workspace/harness-build/design/fleet-plugin-design-v1.md`.
- Assumptions log: `~/workspace/harness-build/ASSUMPTIONS.md`.
- Evidence: 17 reader and checker reports, plus digests, in `~/workspace/harness-build/evidence/`.
