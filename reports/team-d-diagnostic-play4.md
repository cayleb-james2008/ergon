# TEAM D — DIAGNOSTIC PLAY 4 REPORT: repeat Play A — clean `fastPathApplied=true` (2026-11-02)

Lead GO: 2026-11-02T03:30Z (repeat Play A in the current fresh opening). Machine `outis-desktop`. Sequence: read-only
state + sealed hashes BEFORE → `preflight` → **one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed
hashes AFTER. No extra Plays, no retries, no build/publication/credential, no image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T033042Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play4-20261102.log`.

## 1 · Verdict in one paragraph

**The clean `fastPathApplied=true` verdict was recorded, and it settles the named-seed question: assert-and-skip
suffices.** The starter applied the frozen plan seed before the world was built (`starter.t0=1623.243 s <
rootFoundAt=1623.285 s`, `starter.applied=2134783819`), Bootstrap built the named day (world ready in **4.249 s**), the
server's `requireApplied` matched seed+layout (`actualSeed=2134783819`, `actualLayout=2` == expected, `fastPathApplied
=true`, `rebuild=false`) and published the trace **before** any capture — no in-Play rebuild, no fallback (a) needed.
The cell still exits **1 fail-closed**, but now on the next masked legacy bug, client-side: `AtlasClient:16` reads
`Lighting.FogEnabled`, which is **not a valid member** on this Studio build (`FogEnabled is not a valid member of
Lighting "Lighting"`, stack `AtlasClient:16 readLighting ← AtlasClient:63 start ← AtlasStarter:1`). It was hidden behind
the server failures until now. The same property is also written in `tests/atlas/Server.luau:95` (tiles-fogoff branch),
so that batch would hit it too. Sealed artifacts unchanged; **2/3 Plays used**; Studio alive and clean.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **759088**, window `0x558c7fa45500`, 2200×1000 floating, **Edit**, plays **1/3**; place == build == `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 03:30:42Z, finished 03:30:58Z (**16 s** total); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 2.199 → PREPARED 2.317 → PLAYING 4.216 → RAN 7.235`; exit **1** |
| `worldTrace` — `result.json.worldTrace` **and** `world-trace.json` (340 B, identical) | `{"t0":1623.2847044999944,"rootFoundAt":1623.284709199972,"worldReadyAt":1627.5338381999754,"actualSeed":2134783819,"actualLayout":2,"expectedSeed":2134783819,"expectedLayout":2,"rebuild":false,"fastPathApplied":true,"starter":"{\"applied\":2134783819,\"t0\":1623.242966699996,\"seed\":2134783819}"}` |
| Identity evidence | `actualSeed == expectedSeed == 2134783819`; `actualLayout == expectedLayout == 2`; `rebuild=false`; `starter.applied=2134783819` before `rootFoundAt`; world build 4.249 s; `fastPathApplied=true` |
| Failure (new, fail-closed) | runtime arrived before view-001 → `FogEnabled is not a valid member of Lighting "Lighting"` / `Players.Chrono_H3X.PlayerScripts.AtlasClient:16 function readLighting` / `AtlasClient:63 function start` / `AtlasStarter:1`; abort `{stop: ok, cleanup: ok, verifyClean: ok}` |
| 45 s bound | not hit — the early-stop arrived in **16 s**, trace attached (`world-trace.json` + `result.json.worldTrace`); the bound remains for the hang class |
| Collector | phases `["prepared","runtime","cleanup"]`, rejections `[]`; chunks `null` (small runtime error POST, direct path) |
| Prepare | `namedSeed 2134783819`, `verifiedSources 101`, `localPlace 0`, `skipOpeningFares 2`, `productionSourcesUntouched true` |
| Host / crash | hostcheck `{stalls 0, label "clean"}`; no new crash dumps |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **759088**; plays **2/3** for this opening; place == build == `c6651793…` |

## 3 · Judgement per batch 7 §6 (atlas:scout)

- **Verdict class:** **fastPathApplied = true** ⇒ **assert-and-skip suffices** — record it; the atlas cell can proceed
  on that basis. No named-seed rebuild, no fallback (a); the 2026-10-31/11-01 "in-Play rebuild hang" reading stays
  superseded.
- **Cell verdict:** still **FAIL (exit 1) fail-closed**, now for an unrelated **client-side legacy API** issue
  (`Lighting.FogEnabled` removed/invalid on this Studio build) — a distinct, named defect that was masked until the
  server path worked. This is not a named-seed or world-build problem.
- **What it proves:** the frozen plan seed reaches the world through the daily-seed fast path; assert-and-skip is the
  correct and sufficient route; the trace is published before the check and retained on success/fail-fast; the 45 s
  bound is not needed for this path; the starter/root fixes are complete for the named seed.
- **What it does not prove:** no view was captured, the atlas cell does not pass, and the look/`viewportMatchesPng`
  evidence this batch is meant to supply is still absent. The `FogEnabled` issue affects `atlas:scout` (client read) and
  would affect `atlas:tiles-fogoff` (server write at `tests/atlas/Server.luau:95`).

## 4 · Proposed fix (NOT implemented — needs the Lead's go, next offline change)

Make the atlas fixtures tolerant of Roblox API drift for the fog properties (test-only files, no production code):
`tests/atlas/Client.luau::readLighting` should read each lighting property through a guarded accessor (the file already
has a `readable()` pattern elsewhere / `pcall`) and record availability instead of raising (e.g.
`fog={... enabled=read("FogEnabled"), available={...}}`); `tests/atlas/Server.luau:95` (`fogOff` branch for
tiles-fogoff) should use the same guarded write and record whether it applied. Offline tests: Lune mock of
`readLighting` with a fake Lighting that lacks `FogEnabled` (must record unavailable, not raise) and one that has it;
source-shape assertions that no direct `Lighting.FogEnabled` member access remains unguarded; keep validator semantics
(absence of fog detail must not be silently accepted as fog-off evidence — the tiles batch's verdict must say whether
the fog-off actually applied). Fold **R4 (wording)** and **R2 (cosmetic untracked-count)** residuals into this same
offline change. Then repeat the atlas Play (third Play of this opening stays reserved; it can carry the fixed client) —
Play B (abort-contact) remains gated behind the fixture-geometry ruling.
