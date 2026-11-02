# TEAM D — FOGENABLED OFFLINE FIX (Nox-ratified), frozen 2026-11-02

Approval: Lead GO 2026-11-02T03:40Z; **Nox run-6 ruling 20261102T033600Z ratifies it exactly as scoped** (guarded
capability accessors that record availability/absence instead of raising; never claim a fog-off write that did not
happen; R4 wording + R2 cosmetic; offline tests; freeze → fresh checker delta; no Play on unfrozen/unchecked code).
**Offline only — no Play in this fix round.** Machine `outis-desktop`, spend $0.

**Fix commit: `a231e3ffc36218822c285b3c25cdd60fed4216b3`** (`a231e3f`; parent `35caa02` = accepted Instance-root fix;
branch tip; tree `765a3b085a9c51c1da14b9d8eea15e012b3395c4`; tracked tree clean; 4 files, **+182/−6**).
Sealed artifacts unchanged (`84324d43…` release, `c6651793…` test/place).
Log: `board/probes/team-d/fog-guard-fix-20261102.log`.

## 1 · Why (layer chain, pattern recorded)

Each atlas layer has unmasked exactly one masked legacy-API defect: starter `require` (silent crash) → `requireApplied`
root-check (`type(root)=="table"` vs Instance) → **client fog read (`Lighting.FogEnabled` absent on Studio 0.745)**.
This fix closes layer 3: `FogEnabled` is not a valid member on this build while `FogStart/FogEnd/FogColor` still read,
and `tests/atlas/Server.luau:95` wrote it unconditionally for the tiles-fogoff batch. Layer status after this freeze:
starter — clean; named-seed identity/assert-and-skip — clean (`fastPathApplied=true` recorded in Play); fog capability —
clean offline, pending the Play that exercises it; **the atlas cell still has never captured a view** (that is the next
Play's job).

## 2 · What changed

- `tests/atlas/Client.luau`: `capability(object,name)` (pcall; a nil read counts as absent — Instances error, tables read
  nil) and `readFog()` record
  `{available={start,finish,color,enabled}, start/finish/color/enabled (only when present), legacy=true when FogEnabled is
  absent, fogOffApplied=<the server's AtlasFogOff application record>}`. No direct `Lighting.Fog*` member access; absence
  is recorded, never raised, never claimed as a value.
- `tests/atlas/Server.luau` (fog-off batch): `FogEnabled` is written only when `pcall(function() return
  Lighting.FogEnabled end)` succeeds; the atmosphere density/haze off is always applied; `inventory.fogOffDetail =
  {applied, fogEnabledAvailable, atmosphere, method}` and the client-visible `AtlasFogOff` attribute are published — the
  record cannot claim a write that did not happen.
- `tools/atlas_acceptance.py`: capability-aware honest contract. The capability map is required for every view; absent
  members must be recorded as legacy (`enabled` null + `legacy` true), not as a value; the fog-off batch additionally
  requires the server record, client/server `FogEnabled` availability agreement, and on a legacy build
  `method=="atmosphere"` plus the attributed client marker (never a fog-enabled claim); non-fog-off still forbids
  `enabled=false`.
- **R4 wording + R2 cosmetic folded**: report/evidence wording is tightened (no stale "rebuild-hang" phrasing; superseded
  reading kept explicitly marked superseded) and the untracked evidence-dir count (792 lines) is recorded as a
  pre-existing cosmetic artifact, not a change signal.

## 3 · Tests (all offline)

- `tests/tools/test_atlas_acceptance.py` (+4 tests, 20 total): fog capability contract (modern accepted; legacy
  `enabled:null`+`legacy:true` accepted; rejected: missing legacy marker, a value claimed while unavailable, unavailable
  member with a recorded value, incomplete capability map, `enabled=false` outside the fog-off batch); fog-off honesty
  (fog-enabled path accepted; legacy atmosphere-only path accepted; rejected: method claiming a fog write, client/server
  availability disagreement, missing legacy claim marker, missing application record); source-shape (no direct
  `Lighting.Fog*` member access in code — comments excluded; guarded server write + `fogOffDetail` + `AtlasFogOff`);
  **Lune test executing the REAL `readFog` block** against modern/legacy/sparse fake Lighting (`available` false /
  `legacy` true / no raise).
- Existing fixtures updated to the capability shape; the previous 16 atlas tests stay green.

## 4 · Freeze evidence (fresh checkout of `a231e3f`)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `a231e3f…`; tracked tree clean (792 pre-existing untracked evidence-dir lines — cosmetic, R2) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 85 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` | **Ran 20 tests … OK** |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 319 tests … OK** rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau atlas-paths` | **1 passed, 0 failed** (it reads Client/Server sources) |
| 6 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 920 passed, 0 failed (126 specs)** rc 0 (~12 min); log `board/probes/team-d/lune-unit-fogfix-20261102.log` |
| 7 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 8 | `git show --stat a231e3f` | 4 files, +182/−6 (`Client.luau`, `Server.luau`, `atlas_acceptance.py`, acceptance test) |

**Mutation checks (maker self-check, each caught, restored, tracked clean):** M4 unguard the client fog read (direct
`Lighting.FogEnabled`) → atlas acceptance FAILED (2); M5 unguard the server write (unconditional
`Lighting.FogEnabled=false`) → FAILED (1); M6 remove the legacy fog-off write-claim check → FAILED (1). Restored:
atlas acceptance 20 OK.

## 5 · Sequencing (per Nox run-6)

1. This fog freeze (`a231e3f`) → **fresh checker delta** — no Play until it ACCEPTs.
2. After delta ACCEPT: `studiocycle` to a fresh 3-Play opening → **ONE next atlas Play** (`atlas:scout --seed
   2134783819`, sealed before/after, capture result/trace, judge, report) — the cell has still never captured a view.
3. Then **ABORT-CONTACT FIXTURE A+C** (Nox ruled: closed-loop pursuit — proxy's current lane position, closing speed
   `clamp(proxy+8,12,60)`, keep the 40 studs/s² cap; both named guards: >700-of-800 observation radius and the 12 s
   no-touch timer; B only if A demonstrably cannot close, with exact bounded values; D rejected) — fixture/test files
   only, full offline test list (Lune helper spec, Python source-shape, synthetic Play-1 series regression), freeze →
   checker delta before any Play.
4. Then Play B (abort-contact), one Play, sealed before/after; the reserved third Play stays reserved.
5. Named-seed blocker: **CLOSED** (Nox ratified: assert-and-skip suffices; fallback (a) closed; the rebuild-hang reading
   stays superseded).
