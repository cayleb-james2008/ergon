# TEAM D — BATCH 6 REPORT (Jobs 16-17): D-1…D-5, 2026-11-01

Machine: `outis-desktop` (Linux 7.2.8-9-omarchy, Hyprland/Wayland), user `cayleb`. Lead: `roblox-lead` (Team D).
Tier A. **Envelope $0 — nothing spent.** Publication **not** authorised and not attempted. No credential value was
asked for, printed, copied, logged or committed. Maker never checker: **nothing in this report is a DONE**; every
claim below is quoted from a run with a timestamp, and the gate evidence still needs a fresh checker.

Window: **2026-11-01 08:29–09:40 ET** (batch). Files: this report + `board/probes/team-d/d1-route-state-20261101.md`,
`d2-candidate-identity-20261101.{md,json}`, `d2-rebuild-20261101.log`, `d3-binding-prep-20261101.md`,
`d3-cast-table-20261101.json`, `style-brief-20261101.md`, `d4-test-matrix-20261101.md`,
`d4-play-plan-20261101.json`, `d5-window-day-runbook-20261101.md`, `a0-offline-baseline-20261101.log`,
`mcp-handshake-20261101T084546.txt`, `mcp-connect-wait-20261101T084841.txt`. The 31 Oct phase-1 report
(`team-d-phase1.md`) remains the history; this file is the current state.

## 0 · Verdict in one paragraph

The Studio route is **live and proven end-to-end** (MCP handshake 33 tools, Studio connected in Edit mode), and the
street game's sealed candidate is now **pinned by deterministic rebuild** — plus its whole offline baseline
**passed this morning** (914/0 across 125 specs, Python OK, store_check 3/3). The card candidate is likewise pinned
(`55cef5d3…`/520,846 B, rebuilt twice) and the authorised audio-upload tool is written, dry-run-proven and committed.
Three new facts change the plan: (1) **Studio is already open and signed in** with a baseplate, so the owner's
remaining act is to open the runner place — but the Lead has ruled we may not drive it until Cayleb confirms the
session is his; (2) the street candidate is **commit `b67abdd`, not `HEAD`** — `HEAD` rebuilds to different bytes, so
"rebuild before testing" must never run; (3) the card upload/bind route has **three concrete tooling gaps** (no
receipt generator, moderation not captured, no per-block upload filter) that must be ruled on before the upload
window can honestly complete binding. Everything else is staged for one-sitting windows.

## 1 · D-1 — Studio route, exact live state

See `d1-route-state-20261101.md` for the full evidence. Headlines, all re-verified at 08:39–09:05 ET:

- Config repair intact: `sha256 ~/.codex/config.toml = 752a7a2f0e80183f6679efa332930af7fca2d492bd9a6529f35ebcd40eff3ac8`,
  the DISABLED line count is 0, TOML parses (17 top keys, 7 servers, `roblox_studio` keys = `['command']`), backup
  `~/.codex/config.toml.pre-studio-repair-20261031T183910` = `e10f08e4…`. New fact: the file's **mtime moved to
  2026-11-01 08:14 ET with byte-identical content** — content is the correct repair state; do not restore a backup.
- Install/pin intact: Vinegar 1.9.4 + Sober 1.8.0; only version folder `version-7062aec9dbe74b14` (Oct 27) with
  `RobloxStudioBeta.exe` + `StudioMCP.exe`; `mcp.bat` pins that existing folder — **no launcher repair needed here**.
- **Studio is open and signed in**: pid 3336735 started **2026-10-31 19:31:42 ET** via `gtk-launch`; log
  `0.745.0.7451137_20261031T233146Z` shows `Authenticated : NO` 23:31:48.990Z → `Authenticated : YES`
  23:31:52.175Z, user scope entered, WebView2 userId **1170918805** (`Chrono_H3X`), no logout since; baseplate
  `placeId 95206881` opened 23:37:23Z; **0 Plays**. The team did not open it; a Sober player session started
  08:26 ET is also not ours. Lead ruling: do not drive it until Cayleb confirms it is his.
- Route proven read-only: MCP `initialize` + `tools/list` (33 tools), bridge listens `127.0.0.1:13469`,
  `list_roblox_studios` → `{"id":"54682709-…","name":"Place1"}`, `get_studio_state` → **Edit**. Raw bytes saved.
- Runner prep: refreshed `~/.var/…/data/usg/usg-desktop.rbxlx` from the sealed candidate's `build/test.rbxlx`
  (`c6651793…`, was stale `74f6673d…` Oct 16). `vinegar_run.py preflight` now has **one refusal left**:
  `need exactly one listed studio named usg-desktop.rbxlx; listed: ['Place1']`; `hostcheck` quiet, latch free,
  `plays: 0`. So the route is one owner act from runnable.

## 2 · D-2 — candidate identity (both jobs CLOSED)

Receipt: `d2-candidate-identity-20261101.json`; method `git archive <commit>` → scratch → `tools/usg build` (street)
/ `bash tools/build.sh` (card), twice each; live `build/` untouched.

- **Street sealed candidate = commit `b67abdd253029378a960a0899d06c672470e9072`** (tree-equal `b1e5ecd`):
  - release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce` / 1,990,548 B — **matches live, twice**
  - test `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` / 2,403,691 B — **matches live, twice**
  - **`HEAD` (`8dfb204`) does NOT reproduce it**: `c34e4cc2…`/1,992,286 B and `43aa732d…`/2,405,429 B. Source drift
    landed after the 03:42 ET build (`dddeafa`, `81044ed`, `f0c9299`, `8a5bc28`, `73bb404`, `2da5aca`, `1bef02f`,
    `8dfb204`). `1bef02f` (the commit `STATE.md` calls "current main") also gives `c34e4cc2…`.
  - Consequence: **never run `tools/usg build` before the window**; the test place on disk is already the candidate
    and already copied to the runner place.
- **Card candidate = `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` / 520,846 B**, rebuilt twice
  from `preserve/candidate-55cef5d3` (pin `add37b6`, tip `43574a5`), matches live file and `build-receipt.json`.
  The doc's `be972098…`/520,854 B is superseded (no commit reproduces it; not needed).
- Authorised side work: `tools/upload_audio.py` committed on **separate branch** `prep/audio-upload-20261101`
  (`e7f885c`, parent `43574a5`); preservation branch untouched. Proof the tool does not change candidate bytes:
  tree `43574a5` + tool rebuilds to `55cef5d3…` twice.

## 3 · D-3 — Job 17 binding prep

`d3-binding-prep-20261101.md` + `style-brief-20261101.md` + `d3-cast-table-20261101.json`. Delivered:

- **U0 exact owner screen** (create.roblox.com/dashboard/creations → Create new experience → Baseplate → copy the
  address-bar URL; account top-right must be `Chrono_H3X`; do not publish/configure anything) and the exact files
  the team then writes/validates. Records hold `universeId = 0, placeId = 0`; the team will not fabricate them.
- **P9 key** exact dashboard path/permission (Assets API Read+Write, owner's own account) and the secure-route rule.
- **Binding flow** U0→U5 with per-asset record fields, naming, privacy, moderation handling, stop rules, and the
  Studio proof shape (24 per-card screenshots + 8 per-cue trigger records; data-file assertions inadmissible).
- **Three tooling gaps found** (this batch): **G1** no receipt generator (`apply_art_ids.py` needs two hash-pinned
  evidence files per card; nothing emits them); **G2** the uploader does not persist
  `moderationResult.moderationState`; **G3** `upload-art --live` has no block filter (all 24 in one loop; franchise
  blocks in the plan are unsupported). **Needs a Lead ruling** before the upload window: extend `publish.py` /
  add a small companion so evidence is captured, or accept a manual transcription with no raw evidence (not
  recommended).
- **Audio tool** written to spec and dry-run proven: 8 POST lines with `x-api-key: <redacted>`, zero network, key
  never read; local gate measures WAV/FLAC duration (2/2/2/2/1/1/45/40 s), all 8 sha256 unique; quota/warning
  (non-updatable, 10/month if not ID-verified) printed; `--live` refuses without key + target. Not run live.
- Style brief derived only from the records (untitled identity, monochrome flat card face, 740×1024, one canonical
  form per roster, single treatment pass, provenance contract, alias≠canonical, 24-row id/franchise/alias/file table,
  audio cue map). No cast invented; no image generated (D13 ruling).

## 4 · D-4 / A0 — Job 16 test matrix, and the first cell actually run

- Machine plan `d4-play-plan-20261101.json`: **51 Plays / 19 Studio openings** parsed from `SPRINTS.md`, validated
  against the runner's `FIXTURES` + `DESIGN_ORDER` — **0 problems** (B1 10/4, B2 4/2, B3 12/4, B4 6/2, C1-sprint 7/3,
  D2-sprint 12/4). `v1.queue` (7 entries) unchanged. `d4-test-matrix-20261101.md` carries the 21-cell matrix with
  the exact evidence each cell must produce and the blocker per cell.
- **A0 ran and PASSED on the pinned candidate** (09:23–09:36 ET, `a0-offline-baseline-20261101.log`):
  `build ok: release.rbxlx sha256 84324d4342667dd1 (1990548 bytes)`; Lune **914 passed / 0 failed (125 specs)**;
  Python `tests/tools` **273 tests OK** (39.2 s, exit 0); `store_check` **3/3 PASS (0 problems)**; all three exit 0; hashes after the
  suite still `84324d43…` / `c6651793…`. (The docs' 926/128 figures belong to a later tree, not this candidate.)
- Blockers named: **W1** Studio place open (owner confirmation pending), **W2** candidate **version upload** yes +
  Open Cloud key (needed for persistence/receipt/phone cells — not yet in the ASK), **W3** 39 R$ purchase yes for the
  real receipt leg, **W4** owner device + audio-listening time, **W5** independent critic seat for the look gate.

## 5 · D-5 — Successor

`d5-window-day-runbook-20261101.md`: T-0 pre-checks; the one owner act (verbatim); in-window route proof; first
sitting = A1 14 specs + `verify-clean` + B1 queue Plays 1–3 (~60–75 min); stop-rule table; per-sitting successors for
B1 openings 2–4 → B2 → B3 (+critic) → B4; Job 17 leg staged behind W2/U0/P9 + the G1–G3 ruling; close/rollback;
packaging gate with a fresh checker. Existing `studio-run-sheet.md` updated with a dated state banner pointing at it.

## 6 · Holds respected / not done

No Studio launch/close, no Play, no Luau execution, no place edit beyond the runner place copy, no upload, no
publication, no credential touched, no money. The open Studio was probed **read-only** and then left alone per the
Lead's ruling. Writes were limited to `board/reports/team-d-*`, `board/probes/team-d/*`, the runner place copy, and
the authorised card-repo prep branch. `$0` of `$0`.

## 7 · Rulings needed now (for the Lead)

1. **Owner confirmation** that the open Studio (pid 3336735) is Cayleb's and may be driven — already routed; the
   first sitting cannot start without it.
2. **W2 name-in-ASK**: the street game needs the owner's yes to upload the candidate as a **Saved/Published version**
   for the persistence/receipt/phone cells (the live private place is version 7, not the candidate) + the Open Cloud
   key through the secure route. This is separate from D12's art binding and is currently missing from the ask.
3. **G1–G3 tooling ruling** for art binding (extend `publish.py` to capture raw responses + emit the two evidence
   files, or a small companion; without it U5 cannot be completed honestly).
4. **Checker seat**: spawn the fresh Roblox checker when the first sitting's evidence exists (A0 is agent-run; it is
   not certified). Same for the B3 independent critic.
5. **Successor**: on confirmation → D-5 first sitting; on W2 → U0 audio/art upload round. Nothing else blocks.


## Addendum 2026-11-01 12:14 ET — two more offline cells closed (before the Studio crash)

`board/probes/team-d/offline-twins-20261101.log` (pinned street candidate `b67abdd` tree, `./.tools/bin/lune run …`):

- **E2 rejoin twin — PASS, exit 0**: `saved: CR 4918 L3 xp 485 fares 1 car shinku_sedan part true loadout street_dampers` → `rejoined: …` identical → `OK: nothing lost across a mid-shift leave`. Labelled SIMULATED (not a live rejoin).
- **F2 store twin — PASS, exit 0**: 6 prompt calls recorded (passes 2037837874 / 2040741283 / 2037897750, products 3719525535 / 3719525571, subscription); stale key `CreditStack` correctly refused with "That item is not available yet."; monetization extras printed. SIMULATED — no Roblox prompt UI exists offline.

These are agent-run offline results (maker-run); the fresh checker certifies at the gate. D-4 matrix rows E2/F2 updated.


## Addendum 2026-11-01 12:20 ET — Street A1 ran in-Studio on the pinned candidate (RED, real finding)

Owner opened `usg-desktop.rbxlx`; Studio MCP verified (`list_roblox_studios` name matches, state Edit). A1 was run
**from the pinned b67abdd tree**, MCP-only, without rebuild: `tools/studio.py sync --test` → `{"synced":101}` rc 0;
`tools/studio.py tests` → **rc 1**: 10/11 spec modules PASS (counts in `a1-highway-finding-20261101.md`),
**`highway.spec` FAILED** on its own assertion that highway traffic is five cars per directional lane-offset
(2 directions × 3 offsets): "highway traffic must spread evenly across both directions and all six lanes".
Kept as a failure, not re-run to pass. The Oct-18 full studio gate passed the same spec on an older build
(`31ee69cb…`, 1,834,345 B), so this is regression evidence on the Oct-21 candidate. Pinned artifacts unchanged
before/after (`84324d43…`, `c6651793…`); Studio still open in Edit. B1 is blocked until A1 is green.
Evidence: `board/probes/team-d/a1-in-studio-20261101.log`, `board/probes/team-d/a1-highway-finding-20261101.md`.


## Addendum 2026-11-01 12:24 ET — A1 highway finding diagnosed: STALE SPEC, not a code regression (STOPPED)

MCP `execute_luau` diagnosis on the pinned sources (seed 918273): 6 lanes (2 directions × offsets 12/24/36) with
**9 cars each = 54 total**; poly lengths 10,467–10,925 vs baselines 5,965–6,423 (`poly ≈ 1.7 × baseline`) and the
production rule `count = min(10, ceil(5 * poly/baseline))` (added in commit `bdcfcb9`, 2026-10-18) explains it.
Minimum spacing **0.669 slot** — the design contract (`docs/TRAFFIC_SPEC.md` row 7) requires ≥0.6 slot, and the doc
itself documents "**26 of 54 cars**" as the current density. `tests/studio/highway.spec.luau` was last changed
2026-10-14, before the design commit, and still asserts 5/lane and a gap/loop-length ≥0.1 metric; the design commit
did not touch it. **Finding: stale acceptance spec, not a traffic defect.** Per the Lead's constraint, no spec and no
code was changed — it needs a design ruling (recommended: update the spec to the design contract; no production-code
change). A1 stays RED, B1 stays blocked. Evidence:
`board/probes/team-d/a1-highway-design-decision-20261101.md`, `a1-highway-diagnosis-20261101.txt`,
`a1-highway-spacing-20261101.txt`.


## Addendum 2026-11-01 12:27 ET — A1 highway spec aligned to the design contract (prep branch; checker pending)

Design decision approved (option 1). The change is on **`prep/a1-highway-spec-20261101` @ `1abe303`** (base
`b67abdd`, never `main`); only `tests/studio/highway.spec.luau` changed (+67/−20). It now derives the per-lane count
from the graph density rule (same inputs TrafficSim uses), asserts one shared per-lane count, asserts the total from
that derivation (**54** at seed 918273), and measures spacing in **slots ≥ 0.6** per `docs/TRAFFIC_SPEC.md` row 7
(design authority: row 7, line 56, commit `bdcfcb9`). No production code, no build, no re-pin; sealed artifacts
`84324d43…`/`c6651793…` unchanged. Maker self-check in the open Studio: `sync --test` `{"synced":101}`, all 11 Studio
spec modules pass, `highway.spec {"passed":2,"seedsPassed":100,"expectedTrafficCount":54,
"minimumTrafficSpacingSlots":0.6694870414212347}`, `tests_rc=0`. Diff for the independent checker:
`board/probes/team-d/a1-highway-spec-change-20261101.diff`; note: `a1-highway-spec-change-20261101.md`. **A1 is not
declared green until the fresh checker PASSes; B1 stays blocked.**


## Addendum 2026-11-01 13:01 ET — A1 official runs clean (spec checker PASS; candidate + post-merge main)

Checker PASS on `prep/a1-highway-spec-20261101` @ `1abe303` (`board/reports/team-d-a1-highway-spec-checker-verdict.md`).
Official A1 run from the prep branch on the sealed open place, MCP-only, no rebuild: `sync --test` `{"synced":101}`
rc 0; `tools/studio.py tests` **rc 0, all 11 Studio spec modules pass**; `highway.spec {"passed":2,
"seedsPassed":100,"expectedTrafficCount":54,"minimumTrafficSpacingSlots":0.6694870414212347,"roadSamples":3142,
"groundSamples":5,"graphNodesChecked":20686}`. Sealed sha256 identical before/after:
`84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce build/release.rbxlx`,
`c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98 build/test.rbxlx`,
`c6651793…` runner place. Then merged the prep branch into local `main` (no-ff **`de361ba`**; delta vs `8dfb204` is
only the spec; repo has no remote, nothing pushed) and re-ran A1 from main: same clean result. Logs:
`a1-in-studio-final-20261101.log`, `a1-in-studio-postmerge-main-20261101.log`; summary
`a1-official-runs-20261101.md`. **A1 evidence complete — green declaration is the Lead's; B1 stays blocked until
then.** No build, no re-pin, no upload/publication/credits; Studio remains open in Edit.


## Note 2026-11-01 13:05 ET — post-merge main is LOCAL ONLY

The A1 spec-alignment merge (`de361ba848db84a084e4446e828ad4f09bf3dd92`, delta vs `8dfb204` = only
`tests/studio/highway.spec.luau`) exists **only in the local checkout** of `~/Work/projects/untitled-street-game`.
That repo has **no remote configured** (`git remote -v` empty), so **nothing was pushed**; the post-merge local `main`
must not be treated as a published/integrated remote state. The sealed candidate pin is unchanged (`b67abdd` /
`84324d43…` / 1,990,548 B); the prep branch `prep/a1-highway-spec-20261101` @ `1abe303` remains the change branch.


## Addendum 2026-11-01 13:05 ET — B1 opening 1 STARTED then HOLD: smoke fails on screen_capture

A1 green → B1 unblocked → opening 1 of the V1 queue started on the sealed candidate place (`b67abdd` sources synced,
place file `c6651793…`, MCP-only, no rebuild). Result: **smoke exit 1** — S1–S4 PASS (Edit, unpublished local place,
Luau payload sizes, collector loopback), **S5 failed** with `ExecError: screen_capture did not return a PNG` (the
runner's own viewport-capture step). Abort path clean: `verify-clean` exit 0 (`clean:true`, latch none), 1/3 Plays
used, queue stopped at `smoke`. Not retried (exit 1 = real finding). Two issues surfaced: (i) the runner's internal
`screen_capture` use sits against the no-screen_capture/no-image ruling — needs a Lead ruling (allow internal evidence
captures with a no-decode guard / adapt cells / isolated workspace); (ii) capture returns no PNG in this Studio session
anyway. Evidence: `b1-queue-opening1-20261101.log`, `b1-smoke-blocker-20261101.md`,
`docs/verify/2026-10-19/vinegar/20261101T180431Z-smoke/smoke.json`. Studio open in Edit; no uploads/publication/
credits; candidate unchanged.


## Addendum 2026-11-01 13:08 ET — screen_capture root cause found (JPEG vs PNG contract)

Read-only MCP diagnosis (metadata only, no image decode): the tool **works** and returns **`mimeType: image/jpeg`**
(Edit mode, no Play needed; `data_len=16552`, base64 prefix `/9j/` = JPEG SOI). The Vinegar runner's `capture()`
(`tools/vinegar_lib/mcp.py:239`) base64-decodes the image and then **hard-requires the PNG signature**
(`frames.py:126`), so the smoke's S5 step fails with "screen_capture did not return a PNG". This is a **format-contract
mismatch** between the current Studio MCP and the runner — not a broken capture, not a viewport/renderer/Play issue.
Recommended fix (needs Lead ruling because it changes evidence format): accept PNG *or* JPEG in the runner, keep the
captured bytes as `.jpg` when JPEG, and add a no-pixel-decode JPEG dimension reader; alternatives are an in-Studio
JPEG→PNG conversion, host-side conversion (forbidden: image decode on DSH), or matrix adaptation. B1 stays held; no
`--resume`, 1/3 Plays used, place clean, candidate unchanged. Evidence:
`board/probes/team-d/screen-capture-diagnosis-20261101.md`, `screen-capture-diagnosis-20261101.txt`.


## Addendum 2026-11-01 13:16 ET — capture-contract fix approved + B1 opening-1 smoke PASS

Root cause was JPEG vs PNG (Studio 0.745 MCP returns `image/jpeg`; runner required PNG). Approved fix on
`prep/b1-capture-format-20261101` (base `b67abdd`): `32919dd` widens the contract (PNG or JPEG, header-only dimensions,
correct extensions, mimeType+sha256 in evidence, runner/SIDECAR notes), `d079f55` fixes the smoke S5 leftover
`info["png"]` KeyError, `fe2bbdb` records sha256 for both S5 captures. A branch-aligned re-sync + `--clear-latch` was
needed because the open place carried the A1 prep-branch spec. **Smoke now PASSES (exit 0)**: S1–S5 all true, captures
`s5-play.jpg` (sha256 2f1ba0fd…)/`s5-camera.jpg` (sha256 523efc4c…), `mimeType:image/jpeg`, `verify-clean` clean,
sealed artifacts unchanged. Queue **paused at 3/3 Plays**; next entry `traffic:abort-contact`; **owner must reopen
Studio** before `--resume`. Findings carried: capture scaled 1920×786 vs viewport 2769×1134 (`pngMatchesViewport:false`
— flag for B3 look cells) and `mcpCameraHeld2s:false`. No host input, no agent image reads, no uploads/publication/
credits. Record: `board/probes/team-d/b1-capture-fix-smoke-20261101.md`, log
`board/probes/team-d/b1-queue-opening1-resume3-20261101.log`.


## Addendum 2026-11-01 14:10 ET — B1 opening 2 stopped on a real finding (runtime helper never reports; Studio crashed)

After the owner reopen (pid 302763, fitted window 2200×1000 — Edit precondition confirmed: viewport 1837×744 = capture
1837×744, ratio 1.0) the standard resume ran clean (MCP verify → re-sync 101 sources → verify-clean → `--resume`). Then:
Play 1 `traffic:abort-contact` **exit 1** — `TimeoutError: collector did not receive runtime within 155s`; prepare
`ok` (bootstrapDisabled, helpers 2, httpEnabledForRuntime, verifiedSources 101), Play server+client running, Edit abort
clean. Retry Play 2 **exit 5** — same timeout, MCP disconnect (`no response to tools/call`), `newCrashDumps:1`; Studio
process gone (`verify-clean` exit 2 “no Studio running”). Plays 2/3 consumed. **Stop rule honoured: no retry.**
Crash evidence: minidumps 14:07 93af6366… 97.6 MB and 13:59 ee5db613… 70.5 MB; wineserver SIGSEGV 14:00:41 on the
reopen. Sealed artifacts unchanged. Proposed one-Play diagnostic (console capture + minimal Play-side HttpService POST
to separate helper-script failure from HTTP-path/Wine instability) awaits the Lead; a runner-side console dump on
runtime timeout is the optional tooling add. Finding: `board/probes/team-d/b1-opening2-runtime-timeout-finding-20261101.md`.


## Addendum 2026-11-01 14:24 ET — B1 runtime timeout root-caused: report exceeds Roblox's 1 MiB HttpService POST limit

The Lead-approved instrumented retry (console-dump tooling `c9d9a58` + runtime-http probe `271b601`, runner/tooling
only) ran two Plays (2nd/3rd of the budget), both **exit 1, no crash**; verify-clean clean, Studio alive. Console text
settles the cause: `Traffic probe collector: Post data too large. Limit: 1024 KB. Post size: 1493 KB` (play 2) and
`… 1390 KB` (play 3). Roblox's HttpService rejects request bodies over 1024 KB; the traffic helper's own check allows
2 MiB (`tests/traffic/Server.luau:89`), so it tries a single POST and the collector never sees the runtime phase →
155 s timeout. So: **not a product bug** (game boots clean; helper runs; `runtime-http.json` confirms
`httpEnabled:true` in Play — the prepare's Edit-time setting survives), and **not (c)** this time (no crash; earlier
crashes appear separate). Classification: **(b) tooling/harness limit.** Fix options (Lead decision, nothing changed):
(1) chunked transport — collector already has `ExactReportTransport` `/chunk` (600 KiB wire) and `city:*` fixtures use
`chunks=True`, but `tests/traffic/Server.luau` has no chunk code and the traffic fixtures pass `chunks=False`;
(2) trim the report under 1 MiB (weaker abort-contact evidence). Queue paused, 3/3 Plays used, `next=atlas:scout`;
Studio alive in Edit; sealed artifacts unchanged. Records: `board/probes/team-d/b1-runtime-timeout-rootcause-20261101.md`,
log `board/probes/team-d/b1-queue-retry-instrumented-20261101.log`.


## Addendum 2026-11-01 14:28 ET — B1 blocked-pending-transport-fix (1024 KB HTTP POST cap; not a product failure)

The instrumented retry root-caused B1's runtime timeout: the traffic runtime report (~1.39–1.49 MB) exceeds Roblox
HttpService's **1024 KB single-POST engine cap**, so the helper's `pcall(Http:PostAsync(...))` is rejected and the
collector never receives the runtime phase (console: `Traffic probe collector: Post data too large. Limit: 1024 KB.
Post size: 1493 KB` / `1390 KB`). Status change: **B1 blocked-pending-transport-fix**, not a product failure; the
diagnostic Plays are evidence, not retries of the cell. Fixed by the Lead-approved option 1 (chunked transport;
plan `board/plans/team-d/chunk-transport-plan-20261101.md`, implemented after approval + fresh checker). Clean negative
proof recorded: the prepare's Edit-time `HttpEnabled=true` survives into Play — `runtime-http.json` (both instrumented
runs, sha256 `f3b01d83ca3f0ddb18cc6d1a36aaaa8313f0fe3986c5afc53fde1c42989b30f0`) = `{"running":true,"server":true,
"httpEnabled":true}`, which eliminates the HTTP-enablement question permanently. Runbook §1d records the 1024 KB cap.


## Addendum 2026-11-01 14:56 ET — chunked runtime transport implemented, tested, frozen for checker

B1's transport fix (Lead-approved option 1) is implemented on `prep/b1-capture-format-20261101` at
**`ec1c394`**, tracked tree clean, frozen pending the fresh independent checker. Runtime reports now go through
`ExactReportTransport` chunking (256 KiB parts, 600 KiB wire) when >512 KiB, so a >1024 KB single POST can no longer
occur; failure messaging names the engine cap and the failing chunk index/count. `tools/traffic_acceptance.py` installs
the transport module as a third helper and hashes it; `tools/vinegar_run.py` sets `chunks=True` **only** for
`traffic:pass` and `traffic:abort-contact`. New offline suite `tests/tools/test_collector_chunks.py` proves
byte-identical reassembly vs the direct `/report` payload (one `runtime` delivery) and the fail-closed negatives with
asserted status/exit codes (dropped chunk, conflicting duplicate, identical duplicate idempotent, out-of-order, wrong
sha256, identity/nonce mismatch, count/bytes mismatch, oversized wire 413). Fixture candidate digests updated and
documented: pass `a8778b1f…`, abort-contact `b145a164…` (previous abort `ddfcb24c…`). Test evidence: new chunk suite
9/9 pytest; traffic acceptance 4/4; handling 8/8; runner suite 68/68; atlas capture index 7/7; Luau timing spec 4/4;
full Lune 914/914; full Python suite rc 0. Sealed artifacts unchanged
(`84324d43…` release, `c6651793…` test/place). No Play until the checker passes; then one `traffic:abort-contact`
retry. Record: `board/probes/team-d/chunk-transport-implementation-20261101.md`, log
`board/probes/team-d/chunk-transport-implementation-20261101.log`.


## Addendum 2026-11-01 15:26 ET — checker REJECT blockers fixed, re-frozen at `065e0fc`

The fresh checker rejected the transport fix on bookkeeping + test realism (transport core passed). Fixed and
re-frozen: **`065e0fc`** on `prep/b1-capture-format-20261101`, tracked tree clean; untracked = 779 evidence-dir lines
(`docs/verify`, `.tools/probe`) — "tracked clean; untracked = evidence dirs", not an empty worktree.

- **Blocker 1 (real bug):** traffic registry entries passed `helpers=2` while the installed helper+starter count is 3,
  so `handling_phase(2)` marked `cleanup.cleaned=False` with HTTP 200 and the validator ignored it. Reproduced:
  `BEFORE helpers=2 → cleaned=False` / `AFTER EXPECTED_HELPERS → cleaned=True`; both registry entries now pass
  `traffic_acceptance.EXPECTED_HELPERS`, `validate_report` requires `cleaned is True`, synthetic fixture updated.
- **Blocker 2:** `test_collector_chunks.py` now exercises the production `collector.handling_phase(3)` with
  production-shaped envelopes (no explicit phase; phase injected from `report.stage`; identity nonce in report) and a
  pinned shared identity so direct and chunked paths receive the same bytes; adds a cleaned-flag reproduction test.
- **Minors:** runner test pins the full fixture→chunks map and traffic `helpers==EXPECTED_HELPERS`; identity negative
  mutates session/request/nonce; 413 case asserts runner exit code 1.
- Re-freeze evidence: chunk suite `10 passed, 5 subtests`; traffic 4/4; runner 69/69; Luau timing 4/4; full Lune
  914/914; full Python rc 0; sealed `84324d43…`/`c6651793…` unchanged. Log:
  `board/probes/team-d/chunk-transport-checker-fixes-20261101.log`; report
  `board/probes/team-d/chunk-transport-implementation-20261101.md`. Frozen for the same checker; no Play.


## Addendum 2026-11-01 15:37 ET — re-check narrow REJECT fixed (mutation guard), re-frozen at `c0d4b22`

The re-check accepted the prior blockers but found the new `validate_report` `cleanup.cleaned is True` clause had no
test (mutation survived). Test-only fix at **`c0d4b22`** (parent `065e0fc`; 2 files, +5/−2): the two missing
fail-closed cases (`cleaned=False`, `cleaned` absent) now live in
`test_traffic_acceptance::test_rejects_corrupt_or_overclaimed_native_evidence`, and `test_collector_chunks.py` imports
`traffic_acceptance.EXPECTED_HELPERS` instead of hardcoding 3. Mutation proof recorded: with the clause removed in an
in-memory copy both cases are accepted and the test method fails 2/2; with the clause restored both raise `ValueError`
and the test passes. Re-freeze evidence: full Python suite **286 tests OK rc 0**, chunk suite 10/10, traffic acceptance
4/4, Luau timing 4/4; sealed `84324d43…`/`c6651793…` unchanged; tracked tree clean (untracked = 779 evidence-dir
lines). Log `chunk-transport-checker-fixes-20261101.log`. Frozen for the fresh checker; no Play.


## Addendum 2026-11-01 15:48 ET — post-ACCEPT opening: traffic retry skipped by checkpoint; atlas/moving findings

The accepted chunk fix (`c0d4b22`) was put to its single Play retry, but `queue --resume` skipped it: the checkpoint's
`done` list already held the three `traffic:abort-contact` entries (exit 1 from the pre-fix attempts; the queue
appends usable results and advances). The opening therefore spent its 3 Plays on the remaining entries and completed
the queue: `atlas:scout` exit 1, `atlas:highway-extension` exit 1, `moving` exit 1. **The chunk transport is still
unproven in Play**; the queue cannot rerun it (`--resume` on a complete queue refuses), so the next attempt is a
single direct `run traffic:abort-contact`. New findings filed: (A) atlas cells fail
`AtlasServer:74: actual seed/layout differ` — the frozen plan (seed 2134783819/layout 2) needs the named-seed path,
but the queue has no per-entry `--seed`, so atlas ran unseeded; intended `run atlas:scout --seed 2134783819`, a
queue/runbook wiring decision; (B) `moving` fails with `frame-001.jpg: probe reply lacks sidecar field(s) seed,
lighting.technology` — the JPEG capture was written (309,009 B, capture path works in Play) but the probe raced
world-ready; fail-closed, no sidecar. `viewportMatchesPng:true` still not observed. verify-clean clean; sealed
`84324d43…`/`c6651793…` unchanged; 3/3 Plays used; Studio alive; no retry (stop rule). Record:
`board/probes/team-d/b1-opening3-atlas-moving-findings-20261101.md`, log `b1-queue-retry-chunk-20261101.log`.


## Addendum 2026-11-01 15:52 ET — direct opening: chunk transport PROVEN in Play; Play 1 fails on a scenario finding

The accepted chunk fix was exercised in Play for the first time (direct `run traffic:abort-contact`, Studio pid 382409,
fitted window). **Transport passes:** the collector received `runtime` via six chunks
(`{bytes:1,393,473, sha256:94f937af…, count:6}`), report.json 2,295,405 B (`8f77c8bb…`), `runtime-http.json`
`httpEnabled:true`; the 1024 KB HttpService POST-cap finding is closed. The cell still exits 1 on a new **scenario**
finding: runtime `state:"failed"`, reason `ServerScriptService.Nightshift.HandlingServer:165: physical generation
retired/replaced` — during the abort-contact maneuver the traffic controller retired/replaced the physical proxy body
while the probe held it, so the probe failed closed after collecting timing/contact evidence; abort path clean
(`stop/cleanup/verifyClean ok`), `verify-clean` clean, latch null. Per instruction no retry and no atlas Plays; Plays
1/3 used; sealed `84324d43…`/`c6651793…` unchanged; Studio alive. Pending Lead ruling on the scenario finding and on
the previously approved runner fixes (queue per-entry seed, checkpoint semantics so a non-zero entry is not silently
done, moving world-ready gate). Record: `board/probes/team-d/b1-traffic-play1-generation-retired-20261101.md`, log
`board/probes/team-d/b1-direct-opening-20261101.log`.


## Addendum 2026-11-01 16:04 ET — seeded atlas Play 2 times out before view-001

Direct `run atlas:scout --seed 2134783819` (Studio pid 382409, fitted window) prepared cleanly with
`namedSeed=2134783819` and started Play, but **no view-001 arrived within 600 s** (collector phases only
`prepared`+`cleanup`; no runtime error posted either). The unseeded identity failure is gone, so the seed path was
reached; offline reading points at the named-seed **rebuild** path: atlas prepare leaves Bootstrap enabled, so the
world is already built with the day's seed and `WorldSeed.ensure` calls `World.generateSafe → World.generate` on the
populated world, which never returned in this Play. This is a fixture/tooling finding (needs bounded instrumentation:
seed before/after, ensure entry/exit, generate phase timings; options: disable Bootstrap and build named-day world
from scratch, pre-seed before Play, or enforce the fast path). Play 3 (atlas:highway-extension) **not attempted** per
the stop rule. Abort clean, verify-clean `clean:true`, latch null; Plays 2/3 used; Studio alive; sealed
`84324d43…`/`c6651793…` unchanged; `viewportMatchesPng:true` still not observed. Record:
`board/probes/team-d/b1-atlas-seeded-timeout-20261101.md`, log `board/probes/team-d/b1-atlas-seeded-opening-20261101.log`.


## Addendum 2026-11-01 16:14 ET — Plan 1 implemented and frozen (`a639fa2`)

Queue per-entry options + failure-aware checkpoint semantics are in at **`a639fa2`** (parent `c0d4b22`), frozen for a
fresh checker. Queue lines may carry `run`'s own options (`atlas:scout --seed 2134783819`, `moving --timeout 240`),
shlex-parsed and validated exactly like `run`; `done` holds only exit-0 entries, any non-zero result is recorded in
`failed` with class `cell`/`harness`/`smoke-failure` and the queue stops, and `--resume --rerun` retries that entry
exactly once per invocation with attempt history (no automatic retry); a queue-text sha256 change invalidates the
checkpoint. Runbook §1e records the seed invocation and the frozen atlas seed 2134783819. Tests: seed reaches the
fixture; failed≠done with class/exitName; plain `--resume` refuses; `--rerun` retries once; harness codes classified;
smoke failure class; 3-Play pause/resume; changed-text refusal; the chunk suite's oversized-wire case is now
deterministic (header-only Content-Length refusal). Evidence: runner suite 73/73 OK, chunk suite 10/10 OK, full Python
`tests/tools` **290 tests OK rc 0**, sealed `84324d43…`/`c6651793…` unchanged; tracked tree clean (untracked = 792
evidence-dir lines). Log `board/probes/team-d/plan1-queue-seed-checkpoint-20261101.log`. Next: Plan 2 (moving
world-ready gate) on approval of this report; no Play until the instrumented diagnostic block is ready.


## Addendum 2026-11-01 16:26 ET — Plan 1 checker fixes (mutation proofs), re-frozen `d2a94db`

The Plan 1 re-check rejected on test-proof only (production behaviour correct): mutation (c) removing the
`--rerun requires --resume` CLI guard and mutation (d2) auto-continuing after a failed rerun both survived. Fixed at
**`d2a94db`** (2 files, +35/−3): new test asserts `--rerun` without `--resume` exits 2 with the message; new test
asserts a failing rerun stops after exactly one fixture call with attempts `[0,1,1]`, entry still `failed`, no
auto-continue. Mutation runs now fail as required and pass on restore (log
`board/probes/team-d/plan1-queue-seed-checkpoint-20261101.log`). Help/doc truth fixed (`--resume` help + module
docstring `queue FILE [--resume] [--rerun]`), queue `smoke` options are rejected, and `--every` keeps mirroring `run`
(non-finite accepted) as a documented known limitation in runbook §1e. Evidence: runner suite 75/75 OK; full Python
`tests/tools` **292 tests OK rc 0**; tracked tree clean (untracked = 792 evidence-dir lines); sealed
`84324d43…`/`c6651793…` unchanged. Frozen for the fresh checker; no Play; Plan 2 follows on ACCEPT.
