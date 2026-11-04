# OPENBELL — Wednesday de-risked: the R11 record has a working generator (4 Nov 06:45Z / 01:40 ET)

Addendum to `SEAL-FUTURES-2026-11-03.md` (§10). Read-only verification; nothing armed, no order, no rail touched, no
credential read, nothing written to `state/r11/`.

## The fact that changes Wednesday

`live/exec/ARMING.md` §0 says *"No generator exists in the tree and none was produced Mon 2 Nov; without it EVERY entry
is refused (`price_limit:missing`)."* **That is stale.** `live/exec_r11.py` exists — commit `a111119`, written 05:52 ET
on 3 Nov, eighteen minutes *after* the runbook that says it is missing. R11 is therefore a 15-minute morning act with a
tool, not a missing component, and a missing R11 record is no longer a reason Wednesday cannot trade.

## Proof (measured at 01:38 ET, in memory, nothing written)

```
generator:  live/exec_r11.py  ->  schema 2, MESZ6, phase challenge, bands ±7/13/20 % from the reference
rail:       live/exec/rails.py::_r11_reasons, window from exec_config.json -> guard_window_et ['09:00','09:15']

record read_at 09:05 ET  ->  _r11_reasons = []                        (gate PASSES, at 09:06 and at 11:00)
record read_at 10:30 ET  ->  ['price_limit:read_outside_guard']       (gate REFUSES)
```

The window rule bites in both directions, so the record must be built **inside 09:00–09:15 ET** — the tool cannot be run
the night before. `exec/state/r11/` currently holds only `cme_limits-2026-11-03.json`; 4 Nov's does not exist yet and
should not.

## What Nox/owner does on Wednesday, in order

1. **R11 (09:00–09:15 ET)** — read the ES December CME fixing, then
   `python3 -m live.exec_r11 --day 2026-11-04 --ref <fixing> --ref-ts <ts> --source "CME Fixing Prices, ES December 2026, read 2026-11-04T09:0x ET"`.
   The bar stopgap (`--from-bars --accept-stopgap`) is still the ruled alternative, not a default.
2. **Enable** — fresh `go_id` + expiry (§5).
3. **Attest** (five legs) then **inspect** by a different seat — the record is day-scoped, so 3 Nov's does not carry.
4. **Flat proof** must be *positive*: `flat_proof.json` currently reads `positive: false`, which `flat_proof_ok()`
   refuses by design. A quiet panel is not a flat proof.

`live/exec/ARMING.md` §0 now carries this correction inline. `live/exec/exec_config.json` was already modified before
this stream arrived and was **not** touched — it is the running executor's own config (`mode: live`, `venue:
tradovate`), which is why the seal's §7 correction matters: a valid enable on Wednesday arms a real broker path.

## Also re-verified this hour (unchanged)

`01:07:00` cycle: `decision HOLD`, `chain ok`, next fire 02:07. `openbell-live`, `openbell-executor`, `openbell-dash` all
active. Executor `day-2026-11-04.json`: `halt: null`, `trades 0`, `placed []`; `enabled.json` expired (`disabled:expired`
— confirmed by calling the guard on the file, not by reading it). Meme desk: `memedesk-desk` still failed,
`hb_age null`, refresh/awake/uptime active.
