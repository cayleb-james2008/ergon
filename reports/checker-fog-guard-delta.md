# CHECKER — fog-guard delta (`a231e3f`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the guarded legacy-fog fix (Nox-ratified) on top of my accepted instance-root delta
(`35caa02`). I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway
clone `/tmp/checker-fog/repo` (2026-11-01T22:49:37-05:00) and a second clone for mutations; maker tree only read.

- **Fix commit:** `a231e3ffc36218822c285b3c25cdd60fed4216b3` (`a231e3f`), parent `35caa02` (my last accepted delta),
  branch `prep/b1-capture-format-20261101` tip, tree `765a3b085a9c51c1da14b9d8eea15e012b3395c4`; tracked mods **0**
  (untracked 800 evidence lines — cosmetic, see R2); sealed `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta:** exactly **4 files, +182/−6** — `tests/atlas/Client.luau` (+25/−1), `tests/atlas/Server.luau` (+8/−2),
  `tests/tools/test_atlas_acceptance.py` (+127/−1), `tools/atlas_acceptance.py` (+22/−2). No `src/`, `assets/`, `build/`
  change; `tools/vinegar_run.py` and `tests/atlas/WorldSeed.luau` untouched (45 s bound and assert-and-skip intact); no
  rebuild path reintroduced.

## Verdict in one line

**ACCEPT** — the client records fog capability honestly (pcall-guarded reads, `available` map, values only when present,
`legacy=true` on absent `FogEnabled`, server's `fogOffApplied`), with **zero direct `Lighting.Fog*` member accesses in
code** and no raise; the server writes `FogEnabled` only when the engine offers it, always applies the atmosphere
fog-off, and records `fogOffDetail{applied,fogEnabledAvailable,atmosphere,method}` + `AtlasFogOff` so a write cannot be
claimed that did not happen; the acceptance contract is capability-aware and enforces legacy absence + fog-off
agreement + legacy atmosphere-only attribution + non-fog-off `enabled!=false`; the delta is pure and all suites are green
(85 / 20 / 319 / 920). Mutations M4/M5/M6 reproduce exactly (2 / 1 / 1 failures).

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Client guarded fog reads, no direct member access, never raises/claims | **PASS** | `tests/atlas/Client.luau:17-37`: `capability()` = `pcall(function() return object[name] end)` + nil-is-absent; `readFog()` calls `capability(Lighting,"FogStart"/"FogEnd"/"FogColor"/"FogEnabled")`, records `fog.available={start,finish,color,enabled}`, sets `fog.start/finish` only when present, `fog.color` only when `typeof(color)=="Color3"`, `fog.enabled` when present else `fog.legacy=true`, and `fog.fogOffApplied` from the server's `AtlasFogOff` attribute (pcall-guarded). Independent code scan (comments stripped): **no `Lighting.FogStart/FogEnd/FogColor/FogEnabled` occurrence**, four capability calls, no raise path. `readLighting` uses `fog=readFog()` |
| ② Server guarded write + honest record | **PASS** | `tests/atlas/Server.luau:93-105` (fog-off batch only): `local fogEnabledAvailable=pcall(function() return Lighting.FogEnabled end); if fogEnabledAvailable then Lighting.FogEnabled=false;table.insert(method,"fog-enabled") end`; atmosphere density/haze always zeroed when an Atmosphere exists (`table.insert(method,"atmosphere")`); `inventory.fogOffDetail={applied=#method>0, fogEnabledAvailable=…, atmosphere=…, method=table.concat(method,"+")}`; `Lighting:SetAttribute("AtlasFogOff",Http:JSONEncode(inventory.fogOffDetail))`. `applied`/`method` derive from actual writes, so the record cannot claim a write that did not run; no unconditional `FogEnabled=false` remains |
| ③ Capability-aware acceptance contract | **PASS** | `tools/atlas_acceptance.py:307-334`: requires a complete boolean capability map; unavailable member ⇒ `enabled is None` + `legacy is True`; present members require their values; fog-off requires `server.fogOff`, `fogOffDetail.applied/method`, and `fogEnabledAvailable == available['enabled']`; if available ⇒ method contains `fog-enabled` and client read `enabled is False`; if legacy ⇒ `method=='atmosphere'` + attributed `fogOffApplied`; non-fog-off ⇒ `enabled is not False` |
| ④ Delta scope / invariants | **PASS** | Exactly the 4 claimed files; no game/asset/sealed path; `tools/vinegar_run.py` untouched (`ATLAS_FIRST_VIEW_TIMEOUT = 45`); `WorldSeed.luau` untouched (`requireApplied` only, no `generateSafe(`); no rebuild path |
| ⑤ Suites (fresh clone) + mutations | **PASS** | runner `Ran 85 tests … OK` (10.939 s); atlas acceptance `Ran 20 … OK` (0.676 s; includes the 4 new fog tests and the Lune `readFog` execution test); full Python `Ran 319 tests … OK` rc 0 (45.208 s); `atlas-paths` `1 passed, 0 failed` (reads the changed Client/Server sources); world-seed `11/0 (2 specs)`; **full Lune `920 passed, 0 failed (126 specs)` rc 0** (22:50:12–23:01:26). Mutations in a throwaway clone, all caught and restored: **M4** client unguard (direct `Lighting.Fog*`) → atlas acceptance **FAILED (failures=2)** (`test_fog_reads_and_writes_are_guarded_in_the_real_sources` + `test_real_read_fog_runs_under_lune_for_modern_legacy_and_sparse_lighting`); **M5** server unguard (unconditional `FogEnabled=false`) → **FAILED (failures=1)** (source-shape); **M6** legacy write-claim check removed → **FAILED (failures=1)** (`test_fog_off_batch_requires_an_honest_application_record`, label `method claims a fog write`). Restored hashes: Client `e70554325dc7…`, Server `3ed6d3cd89a9…`, atlas_acceptance `4dddd7d85ae2…`; git clean |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged. The Play-4 evidence (`board/probes/team-d/diagnostic-play4-20261102.log`) corroborates the root cause (`FogEnabled is not a valid member of Lighting` at `AtlasClient:16 readLighting`, also written at `Server.luau:95`) and records the clean `fastPathApplied=TRUE` (named-seed blocker resolved); I did not reproduce the Play |

## Residuals (named, non-blocking)

- **R1 — none open.** All three claims verified in bytes + by independent runs; the three mutations fail exactly as
  claimed.
- **R2 — cosmetic:** report says "792 pre-existing untracked evidence-dir lines"; the tree now has 800 (Play evidence
  dirs). Tracked tree is clean; no commit impact.
- **R3 — live limit:** no Play, so the Studio 0.745 `FogEnabled` absence path is proven offline (fake Lighting in the
  Lune `readFog` execution test, source-shape, fixtures and mutations), not on the engine; the authorized atlas Play is
  the live proof. The readFog block is executed for real under Lune against modern/legacy/sparse Lighting surfaces.
- **R4 — code nit (non-blocking):** `readFog`'s `fogOffApplied` expression calls `Lighting:GetAttribute("AtlasFogOff")`
  twice (`pcall(...) and Lighting:GetAttribute(...) or nil`); behaviour is correct (first pcall false short-circuits,
  real Lighting always has GetAttribute), but it could cache the value. Not a defect against the claims.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-fog-guard-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
45 s/assert-and-skip + guarded Client/Server code + contract), `suites.log`, `runner.log` (85 OK), `atlas.log` (20 OK),
`python.log` (319 OK), `atlas-paths.log` (1/0), `world-seed.log` (11/0), `lune.log` (920/0, 126 specs),
`mutations.log` (M4 2, M5 1, M6 1; restored hashes), `run-started.txt`. Throwaway copies
`/tmp/checker-fog/{repo,mut}`; maker repo read-only, unchanged, still at `a231e3f`.
