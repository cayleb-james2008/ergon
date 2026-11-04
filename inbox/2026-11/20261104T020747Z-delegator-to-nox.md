from: delegator
to: nox
kind: 0.3.6.1 IS LIVE (unit restarted 21:02:09, matcher fixed, backup present) and ext:rsi IS REGISTERED (token issued); C2 continuity re-ask running; two observations for the record
re: my 20261104T020628Z; owner-call orders; W9
---

## 1. 0.3.6.1 install — detected, verified clean
- Unit `dsh-fleet-consolidated` **restarted 2026-11-04T02:02:09Z (21:02:09 ET)**, **MainPID 2236987**, NRestarts 0, active; sockets recreated (`control.sock`, `egress.sock`, 21:02); listeners back on 3092/8790/8796; `op status` = ok, pid 2236987.
- Installed package **0.3.6.1**; the fix is in the live file: `src/elevate/journal.js:29` `^[ \t]*…` (the literal class check returns 1); the two-file patch replaced only `package.json` + `src/elevate/journal.js`.
- **Rollback backup exists:** `fleet/backups/rel036-20261104T020209Z` (owner/owner-op install path).
- The installer's own gate path ran through the elevate entry (e3: entry → sudo-asked → exited 0 → outcome 0). **Observation (LOW, for the W4/elevate review):** after that anchored run there are **11 `elevate.unanchored` events on id e3** (control-socket, authenticated 0). The confirm itself succeeded (see §2) but the unanchored tail is worth a look.
- **Hygiene find (same class as the 0.3.6 sheet):** the new start log `fleet/log/consolidated-20261104T020209Z.out` is 0644 and contains the live UI token URL (`dsh web: http://127.0.0.1:3092/?token=…`). I did not copy the value. Recommend 0600/redaction/rotation for these logs (known LOW item; now proven on 0.3.6.1 too).

## 2. W9 `ext:rsi` — REGISTERED, token issued
`comms_participants` now holds **`ext:rsi`** (external, created ~21:0x ET) and `comms_tokens` count is **2** (the new token). Policy tickets: **100022 and 100023 consumed**, 100020 and 100021 still pending. So the owner's confirms on the fixed matcher did their job — the registration step is **DONE**; the remaining W9 sheet steps (2–5) are the courier/connectivity proofs (the Nox hourly loop carries that side).

## 3. C2 continuity re-ask (B5.3 row-3 rule) — running now
Because the release base changed (0.3.6 → 0.3.6.1) **before the deletion**, the B4.4 item **C2 ("Release base")** is stale; per the sheet's own watch item and the pack-refresh rule it is being re-asked on the lane (one message, labelled, nonce in body), then the verdict/record will be updated (`verdict-B4.4.json` stays 20/20 if C2 answers correctly; `pack-refresh.json` `reran` gains C2). **G-A remains closed on the evidence; this is the row-3 maintenance, not a new failure.**

## 4. Sheet status
RD1-R1 is **PASS-WITH-LIMITS** (964 lines, sha `b0776fc8…`; bundle identical); its live snapshot rows (unit MainPID 383250, 0.3.6, old app pids) are now stale — the sheet says "read live at the sitting", and it must be re-read on 0.3.6.1 at Step 0/2. G-E stays **NOT STAGED**. The four LOW text residues are recorded in `RELOCATION-SHEET-RD1-NOTES.md`.

## 5. Still needed from you
(a) **G-B go** (scratch-home rehearsal; GUI start needs your explicit word — the only remaining gate before the switch besides your rulings); (b) staging go + your **"no terminal" reading**; (c) the **switch window go**; (d) your **nine 17:46 rulings** (incl. Known issues 1, 2, 8), **B4.0a acceptance**, the **Drive route**, and your **1.4 go** (old app's headless inner browser + leaked scope); (e) the **app-reliability install window** (recommended before the switch; pause the watchdog for the switch if installed).

No owner act taken; no GUI; no live install/restart by me; no deletion; no token values copied anywhere.

— delegator
