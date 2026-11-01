# B1 Play 1 (direct traffic:abort-contact) — CHUNK TRANSPORT PROVEN; new scenario finding (2026-11-01 15:50–15:52 ET)

Studio pid 382409, window 2200×1000 fitted, sealed place `c6651793…`, release `84324d43…`, 0/3 before the run.
Prep: MCP verify (place + Edit; the thin "RobloxStudio" window is not mapped/blocking — only the main Studio window is
mapped) → re-sync B1 tree `{"synced":101}` → verify-clean clean → direct `run traffic:abort-contact`.

Log: `b1-direct-opening-20261101.log`. Run dir:
`docs/verify/2026-10-19/vinegar/20261101T205056Z-traffic-abort-contact/` (report.json 2,295,405 B, sha256
`8f77c8bb004b927c5d0c66edf03c35b1186cb87ecd1fc92401f76adef3ceb86f`).

## The chunk transport is PROVEN in Play

Collector result: `phases: ["prepared","runtime","cleanup"]`, chunk metadata
`{"bytes": 1393473, "sha256": "94f937af291dd5f9888a8cefd79ce27aff0ceba145ebdf7765f96693d6f89b0b", "count": 6}` — the
1,393,473-byte runtime envelope (six 256 KiB chunks) was reassembled by the collector and accepted; `runtime-http.json`
records `{"running":true,"server":true,"httpEnabled":true}`. The 1024 KB HttpService POST cap finding is closed:
`traffic:abort-contact` no longer times out on delivery.

## Play 1 still exits 1 — new real scenario finding (not transport)

`FixtureFailed: runtime reported failure: {…}`; the delivered runtime report is:

```
state: failed
reason: ServerScriptService.Nightshift.HandlingServer:165: physical generation retired/replaced
cleanup: {runtime:true, connections:{Touched,Heartbeat,PostSimulation,PreSimulation all true}}
```

`tests/traffic/Server.luau:165` asserts
`controller.bodies[car.id]==proxy and proxy.body.Parent and car.generation==report.fixture.generation` on every
simulation step. During the abort-contact scenario the traffic controller retired/replaced the physical proxy body (or
bumped its generation) while the probe held the reference, so the probe aborted by design (fail-closed) after
collecting timing/contacts evidence. Abort path clean: `stop:ok, cleanup:ok, verifyClean:ok`; `verify-clean` after the
run `clean:true`, latch null. This is the first actual execution of the cell — a product/test scenario finding to
diagnose (controller generation churn during the cut-in/abort maneuver), not a harness/transport failure.

## State

Plays 1/3 used, Studio alive in Edit, place clean, sealed artifacts unchanged (`84324d43…` release, `c6651793…`
test/runner place). Per instruction, **stopped on the Play 1 failure — no retry, no atlas Plays attempted**. Atlas
seed invocation (`run atlas:* --seed 2134783819`) and the queued runner fixes (per-entry seed, checkpoint semantics,
moving world-ready gate) remain pending the Lead's ruling.
