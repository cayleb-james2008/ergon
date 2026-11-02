# INTERIM RULING — sidecar `Lighting.Technology` read path (Lead → Team D, 2026-11-02)

Status: **accepted by the Lead as an interim; filed by the Lead to Nox for confirmation** (Nox's rule: interims
return to him the same hour). If Nox rejects it, Team D adapts; Plans 3/4 continue on this basis in the meantime.

## The problem being ruled

`docs/art/SIDECAR.md` requires `report.lighting.technology` (`Lighting.Technology.Name`, expect `"Future"`).
`Lighting.Technology` is unreadable from any `execute_luau` context — MCP Edit probe 2026-11-01T23:45:08Z returned
`"The current thread cannot read 'Technology' (lacking capability RobloxScript)"`; the 2026-11-01 moving Play probe
also lacked it. Evidence: `board/probes/team-d/lighting-technology-capability-probe-20261101.{json,py}` and
`board/reports/team-d-batch7.md` §2.

## Ruling: option (1) — offline, hash-bound derivation from the sealed place XML

The runner derives the value **offline** from the sealed place XML (`Technology=4` = `Enum.Technology.Future` per the
place schema) and records it as a run field. Conditions (all binding):

1. The record carries: the **XML file path**, its **sha256**, the **parsed raw integer**, and the **enum-mapping
   evidence** (`Technology 4 = Future` per the place schema).
2. The sidecar documents the source explicitly: `technologySource: "sealed-place-xml"` (or equivalent) and must **NOT**
   claim an engine/`execute_luau` read happened (the capability is proven unavailable).
3. `look_metrics` verifies the recorded technology against the **same build hash** (hash-bound derivation: a stale or
   wrong XML cannot pass).
4. The field stays **REQUIRED** — only the read path changes, never the acceptance bar. No other sidecar requirement is
   softened.
5. Plan 2's gate correctly does not require the field; it records only (frozen commit `29a626b` — unchanged by this
   ruling).

## Which captures depend on the derived field

- `moving` frames (`frames/frame-NNN.json`): **yes** — sidecar via `frames.look_report`.
- All atlas views (`atlas:scout`, `atlas:pilot`, `atlas:highway-extension`, `atlas:edge`, `atlas:tiles-fogoff`):
  **yes** — same `frames.sidecar` path (`tools/vinegar_run.py:643`).
- B3 look packs / `look_metrics` measure/compare: **yes** (they consume those sidecars).
- `first-fare`: **no capture**, no sidecar; only the world-ready gate runs (its `worldReady.lighting.technology` stays
  observed/unreadable).
- `race`, handling/city/traffic fixtures: no capture sidecars, not affected.

## Scope for Team D

Implement with Plan 3/Plan 4 freezes (runner/sidecar/look_metrics changes + tests for the hash-bound derivation and the
explicit source field), so the diagnostic block does not stall. Lead's one diagnostic Play remains gated on Plans 3+4
freezes + ACCEPTs + the Lead's explicit go. No Play, no build, no Studio input in the meantime.
