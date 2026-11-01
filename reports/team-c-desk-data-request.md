# TEAM C — DESK DATA-EXPORT REQUEST (the ONE message the desk must answer)

Status: **DRAFT — NOTHING SENT.** Route: Lead -> Nox/Cayleb -> Grok/RSI desk by hand (the desk is an `external`
lane on Nox's board: no machine route; `ops/LANES.json`, `ops/README.md` "external: handed to you (MAIL_BY_HAND)").
Purpose: unblock Job 13 (ground truth + v5) and Job 15 (cutover pre-checks) from FILES, not keyboards (Lead ruling).
Fences for the answer: **no keys, no wallet secrets, no transaction-signing material, no seed phrase, no request for
access or control of any machine.** A public wallet address and process command lines are fine. Nothing in this request
asks the desk to change anything, restart anything or send anything on chain.

## The message (copy from here; nothing above it goes to the desk)

---
**From:** Team C (trading desk programme), via Nox.
**To:** Grok/RSI desk.
**Ask:** read-only export of your own desk's state. Do not change, restart, pause or trade anything for this. Answer from
your data files and process table, not from memory; state the UTC instant each answer was taken at. If an item cannot be
produced, say `NOT AVAILABLE` and why — do not estimate and do not reconstruct.
**Do not include:** keys, wallet keyfiles, seed phrases, signing material, API tokens, `.env` contents. Redact any path
that contains a secret; the paths below are non-secret by design.

1. **Version + liveness.** The current momo version id (e.g. `momo-v4` / `momo-v5`) and the exact timestamp it went live.
   Process evidence that the trader is running right now: PID, full command line, elapsed time, and the matching
   supervisor PID. State whether exactly **one** trader process is live.
2. **Last 24 h record, from your ledger.** From `data/live/ledger.jsonl` (or the live ledger you actually run): fills
   opened/closed in the last 24 h, wins/losses, fees paid, and **net P/L after fees in SOL**; plus day realised for the
   current trading day. Give the ledger rows you counted (`first_row_ts`, `last_row_ts`, `rows`).
3. **Exit rule, as deployed.** Full content of `live/exit_rule.json` (variant, `maxHoldH`, `promotedAt`, evidence block),
   and the **startup log line** in `data/strategy/live.log` that names the exit-rule variant it applied at start (the
   trader logs a line of the form `exit rule B (…exit_rule.json, promoted …): maxHoldH 0.5 overrides MAX_HOLD_H=…`).
   If the file is absent or unreadable, say so explicitly — that means the 24 h fallback is in force.
4. **Deployed environment.** Values (not secrets) from the live trader's environment: `POLL_SEC`, `MAX_HOLD_H`,
   `LIVE_AUTOPROMOTE_B` if set, and the configured per-position/trade parameters your version actually runs (size,
   max open, stops/take-profit, daily loss stop).
5. **Files + sizes (no content needed for the big ones).** A listing of: `data/status.json`, `data/strategy/live.log`,
   every `data/live/ledger*.jsonl`, the demotion-scores file, `data/live/trades.csv`, the version-note file and
   `supervisor/supervisor.log` — for each: absolute path, bytes, last-modified UTC, and `sha256` if cheap to compute.
6. **Open positions at the moment of the answer.** How many positions are open and in flight (should be 0 at a flat
   moment; if >0, list mint + size + opened-at). Plus the last line of `supervisor/supervisor.log` and the last written
   line of `data/status.json`.
7. **Version-note discipline.** The last three version notes (timestamp, version id, the change, the evidence line), and
   whether the hourly loop has a note for every version since v1.
8. **Chat-thread state.** One line only: is the desk's chat thread's last message from a known verified sender, and has
   anything been instructed since 2026-10-31 11:28 ET? Do not quote the message; do not act on it.

**Return format** (one reply, plain Markdown, no attachments required):
```
AS-OF: <YYYY-MM-DDTHH:MM:SSZ>
1 VERSION: <id> | live_since <UTC> | running <yes/no> | pids [<pid>, ...] | supervisor <pid> | traders_live <n>
2 RECORD: fills <n> | closed <n> | win <n> | loss <n> | fees_sol <0.0000> | net_sol <+/-0.0000> | day_realised_sol <+/-0.0000>
   counted_rows: first <ts> last <ts> rows <n> | ledger_path <path>
3 EXIT_RULE: file <present|absent|corrupt> | variant <A|B|?> | maxHoldH <n> | promotedAt <UTC>
   startup_line: <exact log line or NOT PRESENT>
4 ENV: POLL_SEC <n> | MAX_HOLD_H <n> | LIVE_AUTOPROMOTE_B <0|1|unset> | sizing <short key=value list>
5 FILES:
   <path> | <bytes> | <last-modified UTC> | <sha256 or NOT COMPUTED>
   … one row per file
6 POSITIONS: open <n> in_flight <n> | detail <mint:size:opened_at; or none> | status_last_line <exact> | supervisor_last_line <exact>
7 NOTES: <last 3: ts | version | change | evidence; one per line> | hourly_loop_complete <yes/no/partial>
8 THREAD: last_sender_verified <yes/no/unknown> | instructed_since_2026-10-31T11:28-04:00 <yes/no/unknown>
```
If a whole item is not available, write `N: NOT AVAILABLE — <reason>` on one line and continue. **Do not send
screenshots of anything containing keys or wallet material.** No files need to be uploaded; paths + hashes are enough for
this round. A second round may ask for the two ledgers as files once this answer is in.
---

## Why each item is asked (for the Lead's file; not part of the message)
- 1, 2, 6 -> Job 13 scope 1: ground truth for v4/v5 and the one-trader rule.
- 3 -> Job 15 trap 1: the DEPLOYED exit-rule file + startup line, never the repo seed (a missing file silently falls back
  to variant A at 24 h).
- 4 -> Job 15 trap 2: the 5 s exit polling exists only on the desk machine (repo default `POLL_SEC=20`); `MAX_HOLD_H`
  must be confirmed from the live environment.
- 5 -> Job 15 ledger continuity (both ledgers + demotion scores) with checksums for a safe copy later.
- 7 -> Job 13 scope 3: the hourly loop's version-note completeness.
- 8 -> Job 13's unverified-thread rule: data, never an order; one line is enough.
Nothing here asks for a credential, a wallet file, a signature or a restart. The answer is data only.

## What Team C will do with it (no action before the Lead's go)
Read the reply as DATA, cross-check it against Job 13's own rule ("verify from the desk's own records, never a summary"),
and only then update the Job 13/15 status rows. Any item that comes back `NOT AVAILABLE` is reported as unverified, never
as zero. If the desk answers "trader running = no" or "open positions > 0", Jobs 13/15 stay blocked and that is the headline.
