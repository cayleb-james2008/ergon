# OpenBell cutover — pre-cutover verification checklist

Written by Nox, 31 Oct 2026, from a direct read of the merged repo
(cayleb-james2008/openbell @ master 6f582f8) and the desk's live record.
Run this BEFORE any live cutover to the unified app. The repo's
docs/CUTOVER.md covers repo hygiene only (history, secrets, research
artefacts) — it says nothing about desk state. This checklist is the
desk-state half. The desk stand-down plan (stop supervisor → 0 open →
copy ledger → fresh desktop wallet → Cayleb moves SOL himself → start
desktop live → stop box supervisor; never two live traders) runs as one
operation with it, stand-down first.

## 1. Exit rule B must survive the cutover — verify, never assume

- The rule lives in DATA, not code: `live/exit_rule.json`
  {variant, maxHoldH}. The repo's seed copy
  (`meme/desk/seed/live/exit_rule.json`) currently carries the correct
  state: variant B, maxHoldH 0.5, promotedAt 2026-10-23, with the
  22 Oct head-to-head evidence (closedA 30, closedB 31, eqB 1.048 >
  eqA 0.988, ddB 7.3% < ddA 11.3%). Seed files are copied into DATA
  when missing, so a fresh deployment starts on B — good.
- The regression path: if the deployed DATA/live/exit_rule.json is
  missing, dropped, or corrupted, `promote.js` falls back to
  {variant A, maxHoldH 24} and `trader.js` falls back to MAX_HOLD_H
  (default 24). A silent return to 24-hour holds is the failure mode.
- CHECKS: (a) after deployment, read the live DATA/live/exit_rule.json
  and confirm variant B / maxHoldH 0.5 byte-for-byte against the seed;
  (b) read the trader's startup log line — it prints the loaded rule
  ("exit rule B …: maxHoldH 0.5") and any override of MAX_HOLD_H;
  (c) confirm MAX_HOLD_H env, if set anywhere in the deployment, is
  0.5 or unset — never 24.
- Note: the unification session's report line "live stays on rule A
  until paper books are chosen" describes the promotion mechanism's
  stance, NOT the shipped default. The shipped default is B (above).
  Do not let that sentence re-open the question at cutover: rule B is
  Cayleb's approved live rule (22 Oct 2026) and stands.

## 2. The 5-second exit polling (Luke stop fix) — locate and carry it

- Live on RSI's box since 30 Oct 20:23–20:27 ET (final trader pid
  2193662): exit-path price checks every 5 s per open position, capped
  ~15 extra calls/min; paper deliberately left on the old code;
  rollback `rollback_exitpoll_20261030-2030.sh` on the box.
- WARNING from Nox's repo read (31 Oct): the merged repo's trader.js
  shows NO 5 s exit-path poll — its poll default is 20 s (POLL_SEC)
  and the copy-exit path carries the known ~40 s lag note. The fix may
  live in the box's env/config or in code that did not come through
  the import. DO NOT assume the unified deployment has it.
- CHECKS: (a) on the live box, identify exactly where the 5 s exit
  polling is implemented (env var, config, or code diff vs the repo);
  (b) carry that exact mechanism into the unified deployment;
  (c) prove it post-cutover from the exits log (trigger / fill /
  detection-lag fields) on the first live exit — until an exit occurs
  under the new deployment, mark this item UNPROVEN in the cutover
  report, the same standard the box fix itself is held to.

## 3. Other cutover prerequisites

- JUP_API_KEY: the unified app needs the key in config before live
  trading. NO new signup: per Cayleb's key-sourcing ruling (28 Oct
  2026, reaffirmed 31 Oct), the desk's live setup already holds a
  working JUP_API_KEY (free tier) and Cayleb does not hand-carry
  keys. Carry the value over from the desk's existing config at
  cutover, agent-side, never printed in chat/log/git — same
  handling as the ledgers.
- ntfy topic rotation: the old topic appears in the imported repo
  history's original SHAs — rotate before or at cutover, and update
  every desk config that publishes to it.
- Cloudflare Worker redeploy so the phone page shows the "unknown"
  status correctly.
- Ledger continuity: the stand-down plan copies the box ledger; verify
  the unified app's DATA starts from that copy (both ledgers —
  live/ledger.jsonl and ledger.jsonl — plus demotion scores), not from
  the repo seed ledgers.
- Rule-B + 5 s polling are also the two settings the DESKTOP PORT was
  already required to carry (gate handover, 29 Oct) — this checklist
  is the same requirement applied to the unified app.

## 3b. Desktop tree archive (Cayleb's pick, 31 Oct 2026)

- The old desk tree on Cayleb's desktop (~/Work/projects/openbell-meme-desk/)
  stays in place until this cutover — it is fenced (PrivatePIDs) and the
  desk service is failed/inert, so it cannot signal his interactive apps.
- At cutover: repoint the desktop host layer off that tree first
  (memedesk-desk.service BindPaths in desk-fence.conf, desk-start.sh,
  memedesk-sync, memedesk-uptime heartbeat, host tests + fence probe),
  verify the unified repo serves each role, THEN move the tree into
  ~/Work/_archive/harness-removal-20261030/. Until the repoint is verified,
  do not move the tree: a missing BindPaths source crash-loops the unit
  and the sync logs errors every 2 minutes.

## 4. Known gaps accepted at merge (not cutover blockers, on record)

- Copy exits on a source sell can lag ~40 s and can be missed if the
  source wallet makes 15+ transactions inside the window
  (unification session's final report).
- Report-only backtester issue at bt/features.py:218 (hash-guarded,
  not edited).
- The merged master's own baseline in a fresh worktree shows ~10
  pre-existing failing suites (environment-sensitive: missing
  node_modules, gitignored data/caches, frozen-hash tests) — the
  Kalshi lane's adopted baseline. Cutover gating is "no new failures
  outside that baseline", not a green-everything demand.
