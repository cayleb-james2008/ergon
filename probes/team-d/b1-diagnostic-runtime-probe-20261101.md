# ONE diagnostic Play — result and honest limits (2026-11-01 14:13 ET)

Approved diagnostic (not a B1 cell; no pass/fail claim for `traffic:abort-contact`). MCP-only, text/JSON, no image
reads. Run dir: `docs/verify/2026-10-19/vinegar/20261101T191335Z-diagnostic-runtime-probe/` (result.json + console
texts + collector-posts.json, all hashed in result.json). Plays: **1 of 3** consumed and counted in the runner budget
(pid 314780, plays=1); verify-clean after = clean; the prior crash latch was cleared with `verify-clean --clear-latch`.

## What was done

MCP verify (Edit, place `usg-desktop.rbxlx`) → start Play (MCP) → wait Play mode + Play-Server ready (`running,server`,
1 player) → `get_console_output` before/during → minimal HttpService POST attempt from the **Play Server** to a local
collector → `get_console_output` again → stop Play (MCP) → back to Edit. No fixture prepare was run (this was a bare
Play of the loaded place, by design of the approved diagnostic).

## What it shows

- **Play starts and the game boots cleanly** in this environment: Play mode reached (Client+Server DataModels), server
  ready with 1 player; console text (`console-during-play.txt`, sha256 `d200da2b…f92a2`) shows
  `Nightshift art: 24 model(s) loaded`, leaderboard preview message, analytics onboarding event — **no error lines**.
- **Play-side HttpService is disabled in the bare place**: `execute_luau` (Server) read `HttpEnabled:false`, matching
  `test.project.json`/`default.project.json` (`"HttpEnabled": false`). The minimal POST therefore could not succeed;
  additionally, the probe's own third argument was wrong (`PostAsync(url, body, 10)` — the third argument is
  `HttpContentType`, so it failed with “Unable to cast double to HttpContentType” **before** the HTTP check). That
  probe is recorded as **invalid for the HTTP question** — honest limit, not a result.
- Collector received **zero** posts.

## What it does NOT settle (the important part)

The fixture's real failure path includes its **Edit-time prepare**, which sets `Http.HttpEnabled=true` and installs the
2 runtime helpers (`handling_acceptance.py:97`, prepare report `httpEnabledForRuntime:true`). This diagnostic skipped
that prepare, so it does **not** discriminate:
(a) runtime-helper script error in Play — product/test finding;
(b) Play-side HttpService / collector path failure despite prepare — environment/tooling;
(c) Wine Play instability (the crash class seen on the previous Play 2).
It does establish that the game itself boots cleanly in Play and that the crash earlier was not a missing renderer or
a broken place.

## Next (needs Lead go — no retry made)

The approved **console-dump-on-runtime-timeout** tooling is now in place (branch
`prep/b1-capture-format-20261101` @ `c9d9a58`). The cheapest discriminator is **one instrumented retry of the queue
entry** (Play 2 of 3 in this Studio): it runs the real prepare (HttpEnabled=true, helpers installed), and if the
runtime still times out the run dir will contain `console-runtime-timeout.txt` with the helper's own error — settling
(a) vs (b), while (c) shows as a disconnect/crash. If it passes, the queue continues with Play 3. No pass/fail claim
unless it passes. Recommendation: authorise that one instrumented retry; if the console shows a helper/product error,
stop and file it; if it shows an HTTP/environment error, stop and file the tooling gap with a proposed fix.

No image reads, no host input; sealed artifacts `84324d43…`/`c6651793…` unchanged; Studio alive in Edit.
