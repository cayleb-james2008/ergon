# D-5 — Successor: window-day runbook (one sitting at a time)

Status: **WAITING-ON-OWNER-CONFIRMATION** (Lead ruling 2026-11-01: do not drive the open Studio until Cayleb
confirms through Nox that the session is his and may be driven). Everything below is staged so the first sitting is
mechanical. Tier A, `$0`, no publication, no credential handling.

## 0 · Entry conditions (all already true)

| # | Condition | Evidence |
|---|---|---|
| 1 | Studio route repaired and **live** | `d1-route-state-20261101.md`: config sha `752a7a2f…` (no disabled line, 7 servers), MCP `initialize`+`tools/list` 33 tools, `list_roblox_studios` → live Studio, `get_studio_state` → Edit |
| 2 | Studio open and signed in (owner-confirmation pending) | log `Authenticated : YES` 2026-10-31T23:31:52Z, userId 1170918805; Studio pid 3336735, **0 Plays** |
| 3 | Sealed candidate pinned | street `b67abdd` → release `84324d43…`/1,990,548 B, test `c6651793…`/2,403,691 B (rebuilt twice) |
| 4 | Runner place refreshed to the sealed test place | `~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` = `c6651793…` (08:59 ET) |
| 5 | Host quiet, latch free, no stale helper | `hostcheck` `quiet: true`; `vinegar_run.py status` `latch: null, lock: free, plays: 0`; preflight's **only** refusal is `listed: ['Place1']` |
| 6 | Runnable plan verified | `d4-play-plan-20261101.json`: 51 Plays / 19 openings, 0 fixture/order problems |

**Trigger:** Lead relays Cayleb's confirmation + "go". Do not start any Studio-changing step before that.

## 1 · T-0 — agent pre-checks (5 min, no Studio change)

```bash
# 1. the repair is still as recorded
sha256sum ~/.codex/config.toml            # expect 752a7a2f0e80183f6679efa332930af7fca2d492bd9a6529f35ebcd40eff3ac8
# 2. nothing stale is holding the port
flatpak ps --columns=instance,application; ss -ltnp | grep 13469; pgrep -af StudioMCP
# 3. host + latch
python3 tools/hostcheck.py preflight      # expect {"quiet": true, ...} exit 0
python3 tools/vinegar_run.py status       # expect latch null, lock free, studio plays 0
# 4. the bookends
sha256sum build/test.rbxlx build/release.rbxlx   # c6651793… / 84324d43…
sha256sum ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx   # c6651793…
```

If any line differs from the expectation, stop and report — do not improvise a second repair (the rollback is
`~/.codex/config.toml.pre-studio-repair-20261031T183910`, only if the route itself failed).

## 1b · STANDARD RESUME PROTOCOL after any Studio reopen (added 2026-11-01)

The runner's `verify-clean` compares every Studio Source hash against the checkout it runs from. If the place was last
synced from a different branch/tree (e.g. the A1 prep branch), the first resume fails with
`verify-clean failed: Source differs from the checkout: …` and latches. Standard steps, in order:

1. Confirm the reopened Studio and place via MCP: `list_roblox_studios` name = `usg-desktop.rbxlx`, `get_studio_state`
   = Edit.
2. **Re-sync from the tree the queue will run from**: `python3 tools/studio.py sync --test` (must print
   `{"synced":101}`).
3. `python3 tools/vinegar_run.py verify-clean` (expect `clean: true`); if latched, `verify-clean --clear-latch`
   (expect `latchCleared: true`).
4. Then `python3 tools/vinegar_run.py queue docs/verify/2026-10-19/vinegar/v1.queue --resume` (or the next script).
5. Record the sealed sha256 of `build/release.rbxlx`, `build/test.rbxlx` and the runner place before and after.

Learned on 2026-11-01: a B1 resume failed exactly this way after A1 had synced the aligned spec into the open place;
the re-sync + clear-latch fixed it without touching the place file or the candidate.

## 1c · CAPTURE-RESOLUTION PRECONDITION for exact-viewport cells (B3/look; added 2026-11-01)

The Studio MCP capture is capped at a **1920 px long edge** (proven 2026-11-01: at window 3416×1390 the image is
1920×786 / 1920×713 for viewports 2769×1134 / 3053×1134; at fitted window 2280×1000 the image is 1917×744 =
viewport, ratio 1.0). `look_metrics` requires the image to equal the recorded viewport, so before any look/atlas cell:

1. `winctl get --target studio` → must be `floating: true`, window width **≤ 2200** (Edit viewport = width − 363;
   2280 leaves only 3 px margin — insufficient). Height: viewport = height − 256.
2. MCP `execute_luau` (Edit) → `workspace.CurrentCamera.ViewportSize` both dims **≤ 1920**.
3. First Play-mode capture at the fitted size → sidecar `viewportMatchesPng: true` (observe, do not infer). If false,
   stop and report before B3.
4. If not fitted: owner/Lead runs `winctl set-size --target studio --width 2200 --height 1000`; the agent does not
   resize windows without authorisation. **Confirmed 2026-11-01 13:34 ET**: at 2200×1000 the Edit viewport is
   1837×744 and the Edit capture is 1837×744 (ratio 1.0, 83 px margin; `winctl verify` rc 0). (winctl is allowlisted Hyprland-Lua window control; audit log
   `~/.local/state/winctl/actions.jsonl`.)

## 1d · ROBLOX HttpService POST CAP — 1024 KB (recorded 2026-11-01)

Roblox's HttpService rejects any single POST body over **1024 KB** (`Post data too large. Limit: 1024 KB. Post size:
1493 KB`). The traffic fixtures' runtime report is ~1.39–1.49 MB, so a single-POST helper can never deliver it — the
collector never receives the runtime phase and the queue entry times out after 155 s. `traffic:pass` and
`traffic:abort-contact` must use the existing `ExactReportTransport` chunk protocol (`/chunk`, 256 KiB parts, 600 KiB
wire) with `chunks=True` in the fixture registry; `tools/vinegar_lib/collector.py` already assembles and hashes the
chunks. Until that transport fix is implemented, independently checked, and merged, **B1 is blocked-pending-transport-fix**
— this is a harness/transport limit, not a product failure. Diagnostic Plays made while finding this are evidence,
not retries of the cell. A single POST body must never exceed the cap: fail fast locally with a clear message.

## 1e · QUEUE seeds and failure semantics (added 2026-11-01, Plan 1)

Queue lines may carry the `run` subcommand's own options: `atlas:scout --seed 2134783819`,
`atlas:highway-extension --seed 2134783819`, `moving --timeout 240`. Parsing is `shlex.split` only (no shell);
options are validated exactly like `run` (seed only atlas:*/race and 1..4294967290; timeout 30–600; every numeric;
visual-flags only for atlas/moving/first-fare). The frozen atlas plan seed is **2134783819** (layoutVersion 2).

`done` holds **only exit-0** entries. A non-zero result is recorded in `failed` with its reason class
(`cell` for a fixture finding, `harness` for refused/latched/host/crash, `smoke-failure` for smoke) and the queue
**stops**; rerun it with an explicit `--resume --rerun` (exactly one retry per invocation, attempt history in
`attempts`). There is no automatic retry loop. Any change to the queue file text (sha256) invalidates the checkpoint
and refuses to resume. Queue `smoke` lines take **no options** (rejected; it always runs the fixed S0-S6 check).
Known limitation: `--every` mirrors the `run` subcommand exactly, so non-finite values (`nan`/`inf`) are accepted by
the parser; the only consumer is `moving`, where a non-finite interval is a fixture-level finding, not a queue error.

## 2 · S1 — the owner's one act (repeat verbatim; never paraphrase)

> Studio is already open and signed in on your desktop. In that one Studio window choose **File → Open**, pick
> `Z:\home\cayleb\.var\app\org.vinegarhq.Vinegar\data\usg\usg-desktop.rbxlx`, leave it in **Edit** (choose
> **Don't Save** if asked), and say **go**. If Studio was closed, open Studio first and sign in as **Chrono_H3X**
> only if it asks — if it asks for a password or 2FA, stop and tell me. Never open a second Studio.

Never: open a second Studio, run `vinegar --help`, press Play outside the runner, kill his other jobs.
One Studio per Roblox login, ever (two revoked the login on 2026-10-16).

## 3 · S2 — prove the route in-window (5 min, always first)

| Step | Command | Pass looks like | Save as |
|---|---|---|---|
| 1 | MCP handshake (`initialize` + `tools/list`, via `tools/mcp_call.py` or a probe script) | response received; `list_roblox_studios` names **`usg-desktop.rbxlx`** | raw bytes → `board/probes/team-d/mcp-handshake-<stamp>.txt` |
| 2 | `get_studio_state` | `Current Studio Mode: Edit` | capture in the same file |
| 3 | Studio's own UI: version + signed-in account | `0.745.0.7451137` / `Chrono_H3X` | screenshot (grim once the window is focused; crop the title/top-right) |
| 4 | `python3 tools/vinegar_run.py preflight` | **exit 0**, `place.matchesBuild: true`, refusals `[]`, `plays: 0` | full JSON → probe file |

If step 4 still refuses on the place name, the wrong file was opened: stop, ask the owner to open the named file
only (no other change).

## 4 · S3 — the first sitting (60–75 min of agent time)

Order matters: the 14 Studio specs first (they do not consume a Play), then the first queue block.

1. **A1 — Studio behaviour: 14 specs.**
   `python3 tools/usg test --studio` (it builds, then `tools/studio.py sync --test` + `tools/studio.py tests` through
   the same MCP bridge). Expect per-spec pass/fail for `tests/studio/{baseline-mechanics, customization, driving-modes,
   driving, gameplay-settings, game, highway, showroom, traffic-shared, ui, visual}.spec.luau` and the 3 runtime
   `.luau`. Save the full log + exit code. **Exit 1 is a real finding — keep it, do not re-run to pass.**
2. **`python3 tools/vinegar_run.py verify-clean`** — expect exit 0.
3. **B1 opening 1 — queue Plays 1–3.**
   `python3 tools/vinegar_run.py queue docs/verify/2026-10-19/vinegar/v1.queue`
   (smoke → traffic:abort-contact ×2). Per Play: exit code + JSON result; the runner pauses at 3 Plays.
4. **`verify-clean`** then tell the owner: close Studio (**Don't Save**), reopen on the same place when we ask.
5. Write the session header (date, machine, Studio version, account, place + sha at open/close, who drove) and the
   per-cell result rows into `docs/verify/2026-11-01/` — failures kept as failures.

Successor after sitting 1: **sitting 2** = B1 openings 2–4 (queue `--resume`, baseline packs; 7 Plays / 3 reopens),
then **sitting 3** = B2 (4 Plays / 2 reopens), **sitting 4** = B3 look ladder (12 Plays / 4 reopens + independent
critic), **sitting 5** = B4 (6 Plays / 2 reopens) — the 51-Play plan is in `d4-play-plan-20261101.json`. Device,
persistence and receipt cells wait for the upload authorisation (W2/W3 in D-4).

## 5 · Stop rules (binding inside the window)

| Signal | Meaning | Do |
|---|---|---|
| exit 0 | pass | next Play |
| exit 1 | real finding | keep evidence, diagnose, **never re-run to pass** |
| exit 2 | preflight refused, nothing changed in Studio | fix the stated reason (Studio closed/wrong place/host busy/3 Plays/latch) |
| exit 3 | cleanup latched | `status` → `verify-clean` → `verify-clean --clear-latch` |
| exit 4 | host-contaminated | label it; a queue reruns such an entry once on the next reopen |
| exit 5 | crash / MCP disconnect | owner reopens (fresh place copy + `verify-clean` first); nothing improvised |
| same step fails twice | stop rule | stop the window, report the failure and the evidence — no third way |
| fresh credential/2FA demand | owner's | stop; never handle a credential |

## 6 · Job 17 leg (staged behind the uploads — do not start it in the first sitting)

Only after W2 (owner upload yes) + U0 target + P9 key:
`U1` audio `python3 tools/upload_audio.py` (dry → `--live`) → `U2–U4` portraits (requires the G1–G3 tooling ruling in
`d3-binding-prep-20261101.md`) → `U5` evidence files + `apply_art_ids.py` binds 24/24 → rebuild → **separate Studio
sitting**: 24 per-card screenshots + 8 cue trigger records per `studio-evidence-sheet.md` (each card id + asset id +
screenshot hash + log line + version; a deliberately wrong binding must fail the check). The card game's
legacy-profile safety proof is by test (definition ruled: everything except `UACG_DEV_v1` / `uacg-dev-probe`).

## 7 · Close-down and rollback

- The owner closes Studio with **Don't Save**, then `hyprctl reload` (he confirms). The config line **stays
  deleted** — the repair serves both games.
- Record the close: build sha at open/close unchanged, `verify-clean` exit, artefacts directory, who drove.
- If the route failed: restore `~/.codex/config.toml.pre-studio-repair-20261031T183910` and report the handshake
  error — do not improvise a second repair. Nox's condition stands: if a fleet gate/fixture that depended on the
  disabled state surfaces, record it and do not re-disable.
- Spend stays `$0`; no upload, no publish, no purchase, no credential.

## 8 · What the successor after this window is

1. Sitting 2+ for the street cells (above) → then device/persistence/receipt cells once W2/W3 clear.
2. Job 17 upload round → binding → card Studio sittings.
3. Packaging: release hash + one plain paragraph for Cayleb + prompts-state note + rollback line (both games), with a
   maker ≠ checker gate: the Lead spawns a fresh checker to run the actual evidence set before anything is called done.
