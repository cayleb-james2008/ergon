# B1 runtime-timeout ROOT CAUSE — payload exceeds Roblox's 1 MiB HttpService request limit (2026-11-01 14:16–14:22 ET)

Instrumented retry (Lead-approved GO): real prepare → Play → runtime probe → console dump on timeout. Two Plays ran
(2nd and 3rd of the 3-play budget), **both exit 1, no crash this time** (Studio alive, `verify-clean` clean, latch
null). Log: `b1-queue-retry-instrumented-20261101.log`. Run dirs:
`docs/verify/2026-10-19/vinegar/20261101T191648Z-traffic-abort-contact/` and `…20261101T191935Z-traffic-abort-contact/`.
Sealed artifacts unchanged (`84324d43…`, `c6651793…`).

## What the console shows (the new evidence channel)

`console-runtime-timeout.txt` (text only):

- Play 2 (191648Z, sha256 `ab6afbba72f4c41ce17ed5b1108608389109aa68a1afe908c117b39d70473db8`):
  `Traffic probe collector: Post data too large. Limit: 1024 KB. Post size: 1493 KB`
- Play 3 (191935Z, sha256 `e6d7ba527342b5ddd2952aab64bb857ec211ec05a9216f8a2fc03b9590a15144`):
  `Traffic probe collector: Post data too large. Limit: 1024 KB. Post size: 1390 KB`

This is Roblox's **HttpService engine limit** on a request body (1024 KB), raised inside the helper's
`pcall(Http:PostAsync(...))` and printed by `tests/traffic/Server.luau:92` (`warn("Traffic probe collector:", why)`).
The helper's own pre-check is `assert(#encoded<=2097152)` (`tests/traffic/Server.luau:89`) — 2 MiB — but it cannot
lift the engine's 1 MiB POST limit. The collector never receives the runtime phase → 155 s timeout.

## Answers to the two open questions

- **Prepare's Edit-time `HttpEnabled=true` DOES survive into Play** — Lead-requested extra datum:
  `runtime-http.json` in both run dirs (sha256 `f3b01d83ca3f0ddb18cc6d1a36aaaa8313f0fe3986c5afc53fde1c42989b30f0`):
  `{"running": true, "server": true, "httpEnabled": true}`. Play server ready, helper executes, the POST is attempted.
- **Not a product bug**: the game boots clean; the traffic helper runs and produces a complete report — it simply cannot
  transmit ~1.4 MB through one HttpService POST.

## Cause classification

**(b) tooling/harness limit** (not (a) helper script error — it is not an error, it is an engine size cap; not (c)
Wine instability — both Plays completed and the abort path was clean this time). Precisely: the traffic fixture's
runtime report (1,390–1,493 KB of JSON) exceeds Roblox's single-POST limit; the collector's own `max_report` is 2 MiB
and the helper's check is 2 MiB, so the mismatch is invisible until the engine rejects the POST.

## Fix options (design decision; nothing changed)

1. **Chunked runtime transport (recommended; preserves full evidence).** The collector already implements the
   `ExactReportTransport` chunk protocol (`/chunk`: `CHUNK=256 KiB`, `CHUNK_WIRE=600 KiB`, count/bytes/sha256,
   in-order) and `city:*` fixtures already run with `chunks=True`. `tests/traffic/Server.luau` has **no chunk code**
   today and the two traffic fixtures pass `chunks=False`; the fix is a tests-helper change (post the report in
   ≤600 KB chunks) + `chunks=True` in the fixture registry. `tests/` sources are synced into the place (no sealed
   rebuild needed) — but it is still a code change and needs the Lead/checker process.
2. **Trim the payload under 1 MiB.** Reduce the traffic report's row/touch/CPU-sample budgets (the timing rows and
   per-row frame data dominate) and align the helper's self-check to 1024 KB. Smaller change, but the abort-contact
   evidence becomes weaker (fewer samples); a design trade-off, not a silent fix.
3. **Raise the limit** — impossible: it is a Roblox engine cap, not ours.

No retry made; queue paused at `next=atlas:scout` with 3/3 Plays used (Lead cycles Studio). Studio alive in Edit;
`verify-clean` clean; latch null.
