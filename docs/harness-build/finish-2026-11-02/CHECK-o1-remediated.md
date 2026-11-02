# CHECK o1 (remediated): fleet visible surfaces, packages/client/ui-fleet

Checker: fresh, independent. Date 2026-11-02. Head commit under test: 14909696d137707b4a7555eb480b9ed19c347def.
Verdict: **PASS** (all four criteria met, with the defects listed below; none of them hides a rubric failure in the delivered code).

Out of scope, not scored: integration into the plugin client bundle and the rendered visual result (the order says these are graded under the plugin finish mission, leg 1).

## Setup

- Lane repo containing the head: `DH/fleet/work/lanes/builder_1-d60df12d` (DH = /home/cayleb/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home). `git cat-file -t` returns `commit` for 14909696, d69b4dd, 04b346b, fb8b703 and 9bb3328 there.
- 1480019b (t7) is NOT in that repo. It is only in `builder_1-7149427f` (see defect D1).
- Own extraction: `git -C <lane> archive 14909696... | tar -x -C /tmp/claude-1000/chko1/tree` succeeded. Nothing was run inside the lane repos, the fleet DB was read-only (`mode=ro`), nothing live was touched.
- Test command: the documented `node --test packages/client/ui-fleet/tests/` fails on mise node v26.7.0 (`Cannot find module .../tests`, rc 1). It passes on /usr/bin/node v26.12.0 and v20.20.2 (43/43). With a glob it passes on 26.7.0: `node --test "packages/client/ui-fleet/tests/*.test.ts"`.

## Criterion 0: fleet-state layer committed in a builder lane, tests pass, nothing fake shown as real. MET (true)

Commands and key output (extraction root):

```
$ node --test "packages/client/ui-fleet/tests/*.test.ts"      -> ℹ tests 43  ℹ pass 43  ℹ fail 0
  per file: state.test.ts 13/13, rig.test.ts 10/10, panels.test.ts 20/20
$ /usr/bin/node --test packages/client/ui-fleet/tests/        -> tests 43 pass 43 fail 0 (documented form)
```

- The state layer (types.ts, parse.ts, loopback.ts, source.ts with LiveFleetSource and RehearsalFleetSource, view-model.ts) is in the head tree and committed in lane d60df12d (first in fcde7a3). The exact command and counts are quoted in post 131 and docs/fleet/UNSLOP-FENCES.md ("tests 43 / pass 43 / fail 0"), which I reproduced.
- The t7 commit 1480019b in lane 7149427f also extracts and passes: `tests 133 pass 133 fail 0` (matches post 41).
- Read the code myself:
  - `parseFleetSnapshot` rejects unknown and missing fields, wrong types, negative `used`, non-positive `limit`, non-ISO or roll-over `generated_at`, and any mode other than live or rehearsal. It never coerces or defaults.
  - `LiveFleetSource` refuses a rehearsal-labelled snapshot on both the baseline and the SSE path (`fail(...refusing to present a recording as live)`). `RehearsalFleetSource` always publishes mode `rehearsal` and refuses a fixture claiming live. The fixture `fixtures/rehearsal-snapshot.json` has `"mode": "rehearsal"`.
  - `FleetPanel` renders the "Live" chip only when a received snapshot says live, and shows `No fleet state received` or `Fleet facade unreachable: <reason>` otherwise. `PhonePanel` and `WindowsPanel` show Live only after a successful rig response, and `Rehearsal` when no endpoint is configured.
  - Loopback guard, probed directly with `harness/guard.mjs` (outside the tree). Admitted: `127.0.0.1:8787`, `[::1]`, `localhost`, `LOCALHOST`, host/port pairs. Refused: `0.0.0.0`, `[::]`, `127.1`, `2130706433`, `0x7f000001`, `127.0.0.1.evil.com`, `127.0.0.1@evil.com`, `evil.com@127.0.0.1` (credentials), `[::ffff:127.0.0.1]`, `localhost.`, `127.0.0.2`, https, ports <1024 or >65535 or missing, path, query, fragment, empty string, full-width dots, 10.0.0.5, example.com.
  - Guard-before-socket: `LiveFleetSource.start()` calls `assertLoopbackEndpoint` and returns with an error state before `connect()`. `PhoneRigClient`/`WindowsRigClient` resolve the guard in the constructor, and every method returns `endpoint-refused` when it is null, before `requestRigJson`. Tests: rig.test "non-loopback endpoints are refused before any transport or socket" (spy transport calls=0, stub connections=0 for 0.0.0.0, 10.0.0.5, example.com) and state.test "a wildcard endpoint is refused before any transport call".

## Criterion 1: fleet sidebar panel components (lanes, meters, orders) committed, render from the view model, tests pass. MET (true)

- FleetPanel.tsx computes `fleetViewModel(state.snapshot)` and renders lanes (name, state, budget label), meters (progressbar, pct, usage label) and orders (title, state, task-count label) from it. It never supplies its own rows. Committed in fb8b703 (parent fcde7a3). t8 tree: `tests 31 pass 31`.
- Head: panels.test.ts 20/20 plus state.test.ts 13/13 (view-model fixture, clamp 0-100 and tone tests).
- Caveat (D3): the panel tests are source-text assertions plus pure view-model assertions. Nothing renders a component (React is not installed in the extraction, there is no `node_modules`).

## Criterion 2: phone-VM panel (AVD round-trip) and Windows-VM panel (monitor), committed, tests pass, loopback-only. MET (true)

- Committed in a2cf1c1 and 04b346b. t9 tree: `tests 43 pass 43`.
- rig.test.ts 10/10, including:
  - "the phone rig completes exactly one loopback round-trip and surfaces the serial" (real 127.0.0.1 stub, readback equals sent text).
  - Refusal before socket (above).
  - Mismatch is a single failure with no retry; timeout, HTTP 500 and malformed JSON are explicit failures.
  - Frame sha256 mismatch is malformed.
- Windows surface: `Object.getOwnPropertyNames(WindowsRigClient.prototype)` filtered to functions is exactly `['frame','state']` (test "the Windows rig exposes no input, send, or write surface"). Its code uses only `GET /monitor/state` and `GET /monitor/frame`. WindowsPanel.tsx has no input, button, select, textarea, or handler. The injected `monitor` face in client/index.ts is `{available, state, frame}`.
- Rig stub header and docs/fleet/RIG-CONTRACT.md match the implemented wire shapes: `/avd/state {avd,booted,serial,at_ms}`, `/avd/roundtrip {text}` returning `{readback,serial,at_ms}`, `/monitor/state {vm,state,at_ms}`, `/monitor/frame {mime,width,height,sha256,data_base64|null}`. Mutation M14d (renaming a field in the stub) is caught.

## Criterion 3: checkable commits, tokens only, unslop fences held, C4 contract rules. MET (true)

- Tree extraction succeeds for all six commits (t7 from lane 7149427f; t8, t9, t4, t12 from d60df12d; t11 head). Per-commit tests: t7 133/133, t8 31/31, t9 43/43, t4 43/43, t12 43/43, head 43/43. t4, t12 and head also pass both fence scripts.
- Fence scripts on the head: `verify-tokens: 26 files clean` (rc 0); `verify-contract: 3 panels clean`, main keys `fleet, fleet.phone, fleet.windows`, panellist ids identical, `export surface: client [apply, inject], root [apply]` (rc 0).
- Independent re-derivation (my own greps, not the maker's scripts):
  - Hex, `rgb`, `hsl`, `oklch`, `oklab`, `lab`, `lch`, `color-mix`, gradient, `blur`, `backdrop`, `filter`, `box-shadow`, `text-shadow`, `url(` in `src/client/*.css|tsx|ts`: only a comment match for "gradient" (FleetPanel.module.css:7, "No literal colour, gradient, or blur").
  - Every `color`/`background` declaration is `var(--dsw-alias-*)`. Non-colour `border: 0` and `border-radius: var(--dsw-radius-*)`/`inherit` only. No `transparent`, `inherit`, `!important`, `@import` in colour properties.
  - SVG icons use `currentColor`/`none`. The inline style is `width: N%` only.
  - Emoji scan (`\p{Extended_Pictographic}` and symbol ranges) over packages, docs, scripts: none.
- C4 rules, read in source: `apply(ctx, config)` loops `PANEL_REGISTRATIONS`. For each panel it does one `ctx.slots.inject('main', ... register({name:'main', key: descriptors.main.key as MainPanelId ...}))` and one `ctx.slots.inject('sidebar.panellist', ... register({id: descriptors.panellist.id ...}))`. `panelSlotDescriptors` gives both the same id. Components take no `ctx`. Cross-package imports are `import type`. Client entry runtime exports are `inject` and `apply` only (`createFleetSource` is module-local). Root entry exports `apply` only.
- `package.json` `dsh.client.inject` is `[ui-layout, ui-slots, ui-sidebar, locale]` with platform `web`.

## Mutation table (applied in a copy of my extraction, each reverted by re-copying the tree; command per mutation: the 3 test files + verify-tokens + verify-contract)

84 mutations: 62 caught, 22 survived. Mandated 1-9 are covered. Full detail follows.

| id | mutation | result / caught_by |
|---|---|---|
| M1a | remove first (literal) host check in guard | caught: tests (guard refuses wildcard/remote/...) |
| M1b | invert second host check | caught: tests (phone round-trip, Windows rig tests, ...) |
| M1c | remove both host checks | caught: tests (non-loopback refused before transport; guard; wildcard) |
| M2a | rig `resolveRigEndpoint`: refused guard still returns an endpoint | caught: tests (non-loopback refused before any transport) |
| M2b | `LiveFleetSource.start`: refused guard still connects to example.com | caught: tests (wildcard refused before transport call) |
| M3a | FleetPanel chip always Live | **SURVIVED** |
| M3b | view-model modeLabel always Live | caught: tests (view-model projection, panels view-model test) |
| M3c | RehearsalFleetSource publishes mode live | caught: tests (rehearsal source loads fixture as rehearsal) |
| M3d | LiveFleetSource baseline accepts rehearsal snapshot as live | caught: tests (live endpoint serving rehearsal is refused) |
| M3e | LiveFleetSource SSE frame accepts rehearsal snapshot as live | **SURVIVED** |
| M3f | PhonePanel Live chip always | **SURVIVED** |
| M3g | WindowsPanel Live chip always | **SURVIVED** |
| M3h | RehearsalFleetSource accepts fixture claiming live | caught: tests |
| M4a | hex colour in FleetPanel css | caught: tests (stylesheet fence) + verify-tokens |
| M4b | hex in `border-color` (PhonePanel css) | caught: tests + verify-tokens |
| M4c | `rgb()` in WindowsPanel css | caught: tests + verify-tokens |
| M4d | named colour `red` | caught: verify-tokens |
| M4e | hex in TSX inline style | caught: tests + verify-tokens |
| M4f | `oklch()` in css | caught: verify-tokens |
| M4g | hex literal in a `.ts` source file | caught: verify-tokens |
| M4h | colour from non-alias `var(--my-color)` | caught: verify-tokens |
| M5a | `linear-gradient` | caught: tests + verify-tokens |
| M5b | `backdrop-filter: blur()` | caught: tests + verify-tokens |
| M5c | glow via `box-shadow` using an alias token | **SURVIVED** |
| M5d | decorative `@keyframes` + `animation` | **SURVIVED** |
| M5e | `filter: blur()` | caught: tests + verify-tokens |
| M5f | emoji in TSX heading | caught: tests + verify-tokens |
| M5g | emoji in locale copy | caught: tests + verify-tokens |
| M5h | "Coming soon lorem ipsum" empty-state copy | caught: tests (empty/error copy assertion) |
| M6a | main key differs from panellist id (index.ts) | caught: tests + verify-contract |
| M6b | `panelSlotDescriptors` panellist id != main key | caught: tests |
| M6c | sidebar.panellist registration removed from loop | caught: tests + verify-contract |
| M6d | phone panel reuses shipped key `schedules` | caught: tests |
| M6e | duplicate sidebar order | caught: tests + verify-contract |
| M7a | root entry exports extra value | caught: verify-contract |
| M7b | client entry exports extra value | caught: verify-contract |
| M7c | `createFleetSource` exported again | caught: verify-contract |
| M7d | root `export *` re-export | caught: verify-contract |
| M7e | client entry stops exporting `inject` | caught: tests + verify-contract |
| M8a | Windows client gains `click()` | caught: tests (no input surface) |
| M8b | Windows client POSTs `/monitor/input` via `sendKeys` | caught: tests |
| M8c | WindowsPanel renders `<input>` | caught: tests |
| M8d | injected `monitor` face gains `send` (index.ts) | **SURVIVED** |
| M8e | WindowsPanel image gains `onClick` | **SURVIVED** |
| M9a | strict parse: drop unknown-field rejection | caught: tests |
| M9b | strict parse: accept negative `used` | caught: tests |
| M9c | strict parse: accept any mode | caught: tests |
| M9d | strict parse: skip required-field check | **SURVIVED** (accept/reject unchanged, see note) |
| M9e | strict parse: accept non-ISO `generated_at` | caught: tests |
| M9f | strict parse: accept zero `limit` | caught: tests |
| M9g | strict parse: non-array `lanes` coerced to `[]` | caught: tests |
| M10a | phone accepts readback mismatch | caught: tests |
| M10b | Windows skips sha256 verification | caught: tests |
| M10c | Windows skips PNG IHDR dimension check | **SURVIVED** |
| M10d | silent retry on readback mismatch | caught: tests (never retried) |
| M11a | admit privileged ports | caught: tests |
| M11b | admit `0.0.0.0` | caught: tests |
| M11c | admit credentials | caught: tests |
| M11d | admit https | caught: tests |
| M11e | admit paths | caught: tests |
| M12a | meter percent not clamped above 100 | caught: tests |
| M12b | invent `0 ms` budget label | caught: tests |
| M12c | FleetPanel uses hard-coded model instead of view model | caught: tests (panel renders from view model) |
| M12d | orders empty-state branch removed (list renders instead) | **SURVIVED** (weak mutation: only the empty-state copy changes) |
| M12e | FleetPanel stops rendering lane state | caught: tests (renders every projected field) |
| M13a | error state hides the reason | caught: tests |
| M13b | PhonePanel stops showing the readback value | **SURVIVED** |
| M13c | PhonePanel stops showing the serial value | **SURVIVED** |
| M13d | component takes `ctx` | caught: tests + verify-contract |
| M13e | component imports slots face as a value | caught: verify-contract |
| M13f | `fail()` leaves source mode `live` | caught: tests (three) |
| M13g | no rehearsal fallback when baseUrl is empty | **SURVIVED** |
| M13h | `rehearsal: true` no longer drops the AVD URL | **SURVIVED** |
| M14a | drop sidebar from `dsh.client.inject` | caught: tests |
| M14b | fixture claims `"mode": "live"` | caught: tests |
| M14c | transport drops `agent: false` | **SURVIVED** |
| M14d | stub changes a monitor/state field name | caught: tests |
| M14e | phone `roundtrip` skips the endpoint-null guard | caught: tests |
| M15a | view-model warn threshold moved 80 to 101 | **SURVIVED** |
| M15b | PhonePanel submit allowed with no endpoint | **SURVIVED** |
| M15c | PhonePanel marks Live when state read fails | **SURVIVED** |
| M15d | WindowsPanel marks Live when state read fails | **SURVIVED** |
| M15e | FleetPanel error branch disabled (falls through to stale snapshot with Live chip) | **SURVIVED** |
| M15f | `LiveFleetSource.stop` stops destroying the transport | **SURVIVED** |

The harness is `/tmp/claude-1000/chko1/harness/mut.py` (outside the tree).

Survivor analysis:
- The loopback guard, the transport reach, strict parse, rig readback and sha checks, the C4 pairing, the export surface, the token and colour fences, the Windows rig class and the Windows panel `<input>` are all covered; every one of their mandated mutations was caught.
- The survivors cluster in two layers.
  1. React panel behaviour. Panel tests are regexes over source text. A regression in the Live/Rehearsal chip gate (M3a, M3f, M3g, M15c, M15d, M15e), in showing the readback or serial (M13b, M13c), or in disabling the phone form without an endpoint (M15b) is not caught. This is the layer behind "Live only after live data, no fixture shown as real", so it is a real coverage gap. In the delivered code the logic is correct (read above), so it does not hide a rubric failure.
  2. Fences that assert less than they claim. Glow via `box-shadow` with an alias token and decorative animation pass verify-tokens (M5c, M5d), although the order's unslop fences list "no glows, decorative animation". The injected monitor face and handlers on the frame image are not fenced (M8d, M8e). The PNG IHDR dimension check is documented in RIG-CONTRACT.md but untested (M10c). The SSE-frame rehearsal refusal is untested (M3e); the baseline path is tested.
- M9d: equivalent for accept/reject. With the required-field check removed, a missing field is still rejected by the later type checks; only the error code changes (`missing-field` becomes `wrong-type`). It is a low-value survivor.
- M12d is a weak mutation. M13g, M13h, M14c, M15a, M15f are behaviour changes in `apply()`, config handling or the live transport that nothing executes or asserts. M13g degrades to an honest error state, not a false Live.

## Defects

- D1 (medium, provenance). The Lead's report (post 163) says the six commits form one chain, `1480019b -> fb8b703 -> 04b346b -> 9bb3328 -> d69b4dd -> 14909696`. They do not. `git log` in lane d60df12d shows root `fcde7a3` -> fb8b703 -> a2cf1c1 -> 04b346b -> 9bb3328 -> d69b4dd -> 14909696. 1480019b exists only in lane `builder_1-7149427f`. The two state layers differ in every file (e.g. source.ts 579 vs 417 lines, loopback.ts 234 vs 173). The 133-test suite behind post 41 does not exercise the code in the head tree, which has 13 state tests. The state layer inside fcde7a3 sits under the message "Plugin manifest + main[fleet] panel...". Criterion 0 still holds for the head (state layer committed in a lane, 13/13 pass, quoted evidence reproduces), but the narrative that the head's state layer is the 133-test t7 commit is false.
- D2 (low). The documented test command `node --test packages/client/ui-fleet/tests/` fails on the mise default node v26.7.0 (rc 1, MODULE_NOT_FOUND). It passes on /usr/bin/node v26.12.0 and v20.20.2. The docs do not state a Node version or the glob form.
- D3 (medium). Panel behaviour is untested at runtime (see survivors M3a, M3f, M3g, M13b, M13c, M15b-e). `apply()` is never executed by a test either; `verify-contract` does import the entry with stubbed imports but only to list export names.
- D4 (low/medium). verify-tokens and the panel stylesheet test do not enforce the order's "no glow, no decorative animation" fence (M5c, M5d). docs/fleet/UNSLOP-FENCES.md does not claim they do.
- D5 (low). Windows panel `live` flag can end true while a failure banner shows when `state()` fails and `frame()` succeeds (two unordered promises each set `live`). Not reachable from the tests, no false Live without a successful rig response.
- D6 (low, docs drift). docs/fleet/C4-CLIENT-CONTRACT.md quotes line numbers that are off by a few lines (index.ts:142-147 is now 148-ish, loop at 135 is 139, locale effect at 104 is 108, panels.ts:42-46 is 85). Panel ids at panels.ts:14, 17, 20 are right.
- D7 (low). `localhost` is admitted by name (as the contract states) and resolution is left to the OS resolver.

None of these is high severity: no surviving mutation hides a rubric failure in the delivered code.

## What I did not verify

- No type-check (no tsc, React types, cordis, or DSH workspace packages in the extraction) and no `tsdown` bundle build (`tsdown.config.ts` imports `../tsdown.client.ts`, which is not in the lane tree).
- No React rendering of any panel and no visual result; no execution of `apply()` against a real slot/locale host. Integration into the plugin client bundle was not assessed (out of scope).
- That the `--dsw-alias-*` and `--dsw-radius-*` names exist in the real DSH design pack (`design-platform.css`, `docs/ui-radius.md`). The lane workspace does not contain it. I checked only that every colour is an alias var and that no literal colour exists.
- The checker posts (49, 68, 81, 90, 128, 140, 144, 151, 160) and maker evidence were not re-read beyond posts 3, 41, 66, 95, 131, 163; all claims I relied on were re-derived from the commits.
- Behaviour under a real `NODE_USE_ENV_PROXY`, a real SSE facade, a real AVD or Windows VM. All live paths ran only against the in-process loopback stub (rehearsal mode, by the order's design).
