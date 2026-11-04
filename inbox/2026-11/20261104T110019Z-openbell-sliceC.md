# Slice C → day lead: meme-desk one-app merge/cutover pack — checker-ready, NOT EXECUTED

**From:** Slice C owner (`opencode-go/deepseek-v4.1-flash`)
**To:** OPENBELL day lead (and Nox, for the box reads)
**When:** 2026-11-04 11:00 UTC
**Pack:** `~/fleet-scratch/overnight/meme-merge-20261104/` — `PACK.md`, `STANDDOWN-PLAN.md`, `PRECHECKS.md`,
`ROLLBACK.md`, `CHECKER-BRIEF.md`, `verify.sh`, `RESULT.md`, `evidence/` (25 files).
**Progress:** `~/fleet-scratch/overnight/openbell-sliceC-progress.md` (the shared `openbell-progress.md` is untouched).

## Verdict

The pack is checker-ready and nothing was executed. **11 PASS · 9 FAIL · 7 UNKNOWN** of 27 precheck rows,
reproduced by `bash verify.sh` (`evidence/24-verify-run.txt`).

## The one thing that decides the cutover

**The one-app has no live-trader mode.** `meme/host/bin/desk-start.sh` exports `DESK_MODE=shadow` and unsets
`CONFIRM_LIVE`/`WALLET_KEYFILE`; `meme/host/sync/sync.py:350-352` flags `desk-mode is not shadow` as a fault;
`docs/GOAL.md` and `meme/host/RUNBOOK.md` both say *"The desktop runs no live trader."* So Nox's order —
fresh desktop wallet → Cayleb moves SOL → **start desktop live** → stop the box supervisor — has nothing to
start at the fifth step. Either the desktop stays shadow (the box keeps the only live trader and no stand-down is
needed), or a live mode is designed and reviewed into the host layer. That is your call plus Nox's, not a file to
copy.

## The six findings that change how the order reads

1. **The 5-second exit poll is not on this machine.** `trader.js:23` still polls every 20 s (`POLL_SEC`), no
   `EXIT_POLL`/`exitpoll` mechanism exists in any local tree, and the desk tree was packaged **28 Oct 09:15** —
   42 h before the box fix (30 Oct 20:23–20:27). FAIL for the tree, UNKNOWN for the box. If it is a `POLL_SEC`
   override, the carry is one line; if it is a code patch, the unified `trader.js` needs it plus a test.
2. **Ledger continuity is not satisfied and the reason is concrete:** the one-app's `data/ledger.jsonl`
   (13,479 lines) and `data/live/ledger.jsonl` (10,698) are **byte-identical to `seed/`**. They are seed data,
   not a box copy. The plan captures the box ledger at step 3 and places it at step 6 before the first start.
3. **Exit rule B is safe** — byte-identical in seed, one-app DATA and the old desk DATA
   (`ff19993f…f0572cd`), loaded at startup and pinned by a test. Nox §1(a)+(b) PASS on the desktop side; the box
   read stays UNKNOWN.
4. **The host layer has never been installed**, and `install.sh` would refuse today because
   `~/Work/projects/openbell` `master` has no `meme/` (the merge is on `github/master`). One `fetch`+`merge` fixes
   it; machine preconditions (pasta/ip/setpriv/Node 20.20.2) all pass.
5. **`watchdog/personas.json` has no target.** The live persona registry is untracked in the old tree and
   `meme/desk/data/watchdog/` does not exist. Only the RUNBOOK's manual `cp` creates it; the plan carries it.
6. **The desk's retry is the sync timer, not the unit** (`NRestarts=0`, `RestartPreventExitStatus=78`). So plan
   step 0 stops `memedesk-sync.timer` + `memedesk.target` before anything else — it is the only autonomous actor
   that can restart a unit or fast-forward a tree under the operator.

## The invariant

`A` = funded wallets able to trade (hard: exactly 1, 0 and 2 are stops); `P` = `pgrep -f 'trader\.js --mode=live'`
processes (advisory; a second one must be unfunded and flat). The check is the desk's own predicate
(`paths.sh:live_pids_host`), the same one `run.sh live` refuses on. Today: `A=0, P=0` here; the box holds the one
live trader. The SOL move is one transaction precisely so `A` changes hands without a 0-window.

## What I need from you

- **You/Nox:** the B1 decision above.
- **Nox:** the four box reads (exit rule digest, where the 5 s poll lives, the ledger handoff, the live-trader
  count) and the box-side steps 1–3; plus the Worker redeploy, whose one `PUSH_SECRET` the box's live pusher
  already holds.
- **Cayleb:** `JUP_API_KEY` supplied agent-side (`settings.py ingest`, never printed), and the unit/wallet/SOL/
  install/archive acts, which the plan marks **C** at each step.

## NOT EXECUTED

No unit was started, stopped, restarted, enabled, disabled or inhibited. No settings, config, state, ledger,
fence or unit file was written. No credential or topic literal was read or printed (names and hashes only). No
tree was moved, copied into place or archived. The only writes are the pack folder and this filing.
