# OP-07 — Complete the Retirement Order (cleanup, approved by Cayleb 31 Oct)

State: Cayleb approved the HOLD-list removals AND the archive wipe (press-list pick, 31 Oct 20:16). Executed so far (your 23:39 report): Prime + voice-agent moved into the removal archive, inode-verified. Archive /home/cayleb/Work/_archive/harness-removal-20261030/ NOT yet wiped — correctly sequenced last (it holds auth tokens; the wipe is permanent).

Held items — disposition each, one by one, with the dependency resolved or the hold justified in one line with evidence:
- OpenCode app/runtime/config/service/launchers: the four timers (outisz-watchdog, outisz-freshness, outisz-state-maintenance, hardening-verify) — preserve their health/state/alert functions; replace or retire only the OpenCode-specific probes/inputs; expected-state changes to hardening-verify need a security review that weakens nothing else. Shared Go/Zen runtime/login dependencies resolved explicitly before any removal.
- Grok Bot app: the live trader's comms path — no removal until a verified safe replacement/offboarding exists.
- Paperclip/Pantheon: five repos without remotes, 75 linked worktrees (six dirty), thirteen Claude skill links into the removal tree — preservation copies verified BEFORE any removal.
- outis-pilot: offboarding/replacement decision + a permitted service-management route.
- browser-use uv tool, Muse CLI cache (Nox's dependency unresolved — raise to Nox, do not guess), herdr (privileged package-removal route), Talos orphan units (runtime/security effects dispositioned; wallet/credential data excluded), Windows VM scratch (media links + planned VM dependencies).

Done when: every held item is either removed-and-verified or closed as keep-with-reason on the record; then the archive wipe executes and is verified (path absent, spot-checks clean). One consolidated completion report to Nox. Nothing with an unresolved dependency gets force-removed — a named hold beats a broken machine.
