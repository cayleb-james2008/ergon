# TEAM D — BATCH 8 REPORT (Jobs 16-17): Plan 3 frozen — generation instrumentation + offline lighting technology — 2026-11-02

Machine: `outis-desktop` (Linux 7.2.8-9-omarchy, Hyprland/Wayland), user `cayleb`. Lead: `roblox-lead` (Team D).
Tier A. **Spend $0 of $0.** Publication not authorised and not attempted. No credential value asked for, printed,
copied, logged or committed. Maker never checker: **nothing here is a DONE**; the fresh checker certifies Plan 3.
Batch window: **2026-11-02 00:30Z – 01:20Z** (19:30–20:20 ET). Plan 2 verdict observed: **ACCEPT** (checker-roblox).

Repo: `~/Work/projects/untitled-street-game`, branch `prep/b1-capture-format-20261101`.
**Plan 3 frozen commit: `90ac9326dfb0037d5af7505d64bd230d1213fb88`** (`90ac932`; parent `29a626b` = accepted Plan 2;
branch tip; tree `8a488c490ab8a82622b68ec6323d754714f7a472`; tracked tree clean; 792 untracked evidence-dir lines,
pre-existing). 17 files, +779/−90.

## 0 · Verdict in one paragraph

Plan 3 is implemented, tested, mutation-checked and **frozen at `90ac932`** (one commit, two bodies of work):
(a) **bounded traffic generation instrumentation** (`generationEvents` ≤20, assertion stays fail-closed) for the
diagnostic Play; (b) the **confirmed offline `Lighting.Technology` derivation** (Lead+Nox ruling) wired into the capture
fixtures and the judge — the required sidecar field is now hash-bound to the sealed place XML, never read from the
engine (proven unreadable). Evidence: runner **83 OK**, full Python **311 OK**, full Lune **919 passed / 0 failed
(126 specs)** rc 0, sealed `84324d43…`/`c6651793…` unchanged, tracked tree clean, six mutation proofs each caught and
restored. No Play, no build, no Studio input, $0. **Plan 4 is not started** (the order is freeze → fresh checker →
next). One test-env follow-on is included: `tests/unit/traffic-runtime.spec.luau` now maps the new `GenerationGuard`
helper (it loads the probe module); the earlier Lune run caught the missing mapping, the final full run is green.

## 1 · Plan 3a — bounded traffic generation instrumentation (assertion stays fail-closed)

**Why.** The 2026-11-01 abort-contact Play failed closed on `physical generation retired/replaced` (the probe held one
physical proxy and one `car.generation`). `TrafficController._retire` legitimately increments `car.generation` and
replaces bodies on contact/recovery/teleport paths (`src/server/TrafficController.luau:117,141-151,222,226`), so the
bare assert could not say **which** identity leg changed, **when**, or what the probe was doing — the datum needed to
decide "contact-correlated replacement (legit)" vs "unrelated churn (product bug)".

**Instrumentation (tests helper + report only; no production code).**
- `tests/diagnostics/GenerationGuard.luau` (new, pure Luau, no game services so Lune can load it): at most **20**
  events. Each heartbeat it compares `(body identity, body.Parent, car.generation)` with the last tuple; on any change
  it appends one event
  `{t, step, oldGeneration, newGeneration, oldBodyAlive, newBodyAlive, bodyReplaced, hasChange, progress, cutInAt,
  abortObservedAt, contactDriveAt, contactObservedAt, contacts, touches, bodiesCount, bodyToken, proxyToken,
  source="step"}` and **returns** the fail-closed message.
- `tests/traffic/Server.luau`: requires/installs the guard as a **fourth helper** (`EXPECTED_HELPERS` is computed, now
  4: HandlingServer, ExactReportTransport, GenerationGuard, HandlingServerStarter), replaces the bare assert with
  `local violation=guard:observe(...)`, still `error(violation)` — the attempt always stops; `report.diagnostic` carries
  the bounded events on pass and failure (`finish()` attaches `guard:report(car.generation)`).
- `tools/traffic_acceptance.py`: `validate_diagnostic()` shape/bound (≤20 events; finite clocks/generations/counters;
  boolean legs; `source="step"`) and a **passing** runtime must show `generationStable=true`, `generationEvents=[]`,
  `finalGeneration == fixture.generation`. A failed runtime stays rejected with its events kept as evidence; the
  manifest/hashes and helper bookkeeping include the new helper.
- Captures: **none** in this fixture (traffic report via chunk transport; no sidecars), so the diagnostic Play output is
  the runtime report + `result.json` only.

**Tests (Plan 3a).**
- New Lune spec `tests/unit/traffic-generation-guard.spec.luau` (5 cases): stable/no event; generation bump records one
  event + fail-closed message names `step N: old->new` + carries hasChange/progress/cutInAt/abortObservedAt/
  contactDriveAt/contactObservedAt/contacts/touches; replaced body at same generation and a body losing its parent are
  both violations; 21 changes → 20 events + `truncated` + a message every time; `finalGeneration` fallback.
- `tests/tools/test_traffic_acceptance.py`: shape/bound cases, pass-with-instability rejected, failed runtime keeps
  events and is still rejected (`did not finish`), manifest now binds the GenerationGuard file (+ source-shape test:
  probe uses the guard and the bare assert is gone).
- `tests/unit/traffic-timing.spec.luau` env gains a guard stub; `tests/unit/traffic-runtime.spec.luau` env maps the
  new helper (its mock `require` needs the key) — caught by the first full Lune run, fixed before the freeze.

## 2 · Plan 3b — offline `Lighting.Technology` derivation (confirmed Lead+Nox ruling)

**Ruling implemented exactly as confirmed** (`board/probes/team-d/sidecar-technology-ruling-20261102.md`): offline
sealed-XML derivation; record carries XML path + sha256 + parsed raw integer + enum-mapping evidence; sidecar states
`technologySource: "sealed-place-xml"` and makes **no** engine read claim; `look_metrics` verifies against the **same
build hash**; the field stays **required**; nothing else softened. Plan 2's gate is unchanged (`29a626b`).

- `tools/vinegar_lib/place_lighting.py` (new): parse the place XML's Lighting `Technology` token → frozen
  `Enum.Technology` table `{0:Legacy,1:Voxel,2:Compatibility,3:ShadowMap,4:Future}` → cross-check the project's declared
  name (`test.project.json` `tree.Lighting.$properties.Technology`) → record
  `{source, file, sha256, raw, value, enum, mapping[, project]}`. Fail-closed on unreadable/ambiguous XML, unknown raw
  values, project disagreement. `verify()` re-derives and refuses a record whose sha256/raw/value is not THIS build.
- `tools/vinegar_lib/frames.py`: `look_report(probe, technology)` / `sidecar(..., technology)` write
  `report.lighting.technology` + `technologySource` + `technologyEvidence`; a missing record fails closed. The probe's
  own `technologyReadable:false` observation is kept as evidence only, never required.
- `tools/vinegar_run.py`: derives the record once per run **before any Studio change** (`build/test.rbxlx` +
  `test.project.json`); a derivation failure is a clean refusal (exit 2, no latch, no Play). Passed to every capture
  sidecar (atlas + moving paths).
- `tools/look_metrics.py`: `technology_problems()` requires the `Future` value, the explicit `sealed-place-xml` source,
  valid enum-mapping evidence and a hash-bound re-derivation against `--build` (default `build/test.rbxlx`;
  `--build-project` default `test.project.json`). Stale/wrong XML, fabricated source, unknown raw or member mismatch →
  capture invalid. `make measure` CLI gained `--build` / `--build-project`.
- `docs/art/SIDECAR.md`: documents the offline read path, `technologySource`/`technologyEvidence` and the hash-binding
  rule (dated 2026-11-02 note); the runbook §2 note points at it.

**Which captures depend on the derived field (as requested):**
- `moving` frames (`frames/frame-NNN.json`) — **yes** (`frames.sidecar`).
- **all `atlas:*` views** (`atlas:scout`, `atlas:pilot`, `atlas:highway-extension`, `atlas:edge`,
  `atlas:tiles-fogoff`) — **yes** (same `frames.sidecar` path, `tools/vinegar_run.py:648`).
- **B3 look packs** measured by `look_metrics` — **yes** (they consume those sidecars; `look_metrics` now also requires
  the build-bound evidence).
- Not affected: `first-fare` (no frames; gate only), `race`, `handling`/`city:*`/`traffic:*` (no capture sidecars).
Consequence for the diagnostic block: `moving` and all atlas cells can now produce a complete, required sidecar; the
atlas cells still need Plan 4's assert-and-skip for their named-seed hang, and the Play remains gated.

**Tests (Plan 3b).** New `tests/tools/test_place_lighting.py` (6: derivation + file/hash/raw/mapping record; every enum
member; malformed/ambiguous XML; project disagreement; hash-bound verify + stale record/other build; sealed test build
= raw 4/Future); `test_look_metrics.py` (22 OK) gains offline-source, enum-mapping, unknown-raw, stale-build-hash,
missing-build and measure-level hash-binding tests; `test_vinegar_run.py` (83 OK) frame tests use the derived record,
prove a probe without Technology still yields a complete sidecar, and prove the two fail-closed paths (missing record,
derivation failure = clean refusal with no Play). `tests/tools/test_collector_chunks.py` helper-count comment follows
the computed 4.

## 3 · Freeze evidence (commands + results; full log `board/probes/team-d/plan3-generation-instrumentation-20261102.log`)

| # | Command (fresh checkout of `90ac932`) | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `90ac9326dfb…`; tracked tree clean (792 untracked evidence-dir lines, pre-existing) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 83 tests … OK** (10.7 s) |
| 3 | `python3 -m unittest discover -s tests/tools` | **Ran 311 tests … OK** rc 0 (42.9 s) |
| 4 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 919 passed, 0 failed (126 specs)** rc 0 (~12 min); log `board/probes/team-d/lune-unit-plan3-20261102.log` |
| 5 | `python3 -m unittest discover -s tests/tools -p 'test_traffic_acceptance.py'` / `'test_place_lighting.py'` | 7 OK / 6 OK |
| 6 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…` (1,990,548 B), `c6651793…` (2,403,691 B), `c6651793…` — unchanged before/after |
| 7 | `git show --stat 90ac932` | 17 files, +779/−90 (see §1–2) |

**Mutation proofs (maker self-check, each caught, files restored, final tree clean):** (A) remove `validate_diagnostic`
call → traffic acceptance FAILED (3 failures); (B) remove the ≤20 bound check → FAILED (1); (C) remove the
pass-with-instability check → FAILED (3); (D) remove `error(violation)` from the probe → FAILED (1, source-shape test);
(E) remove the look_metrics build-hash binding → look_metrics FAILED (2); (F) remove the recorder's event bound →
Lune guard spec 4 passed/1 failed.

**Exact checker reproduction:** `git checkout 90ac9326dfb0037d5af7505d64bd230d1213fb88` in a fresh worktree →
`git status` empty; run rows 2–5 above (Lune takes ~12 min; no Play, no Studio needed for any of them); sealed hashes as
row 6; spot-check the two contracts (a) `GenerationGuard.MAX_EVENTS == 20`, probe calls `guard:observe` and still
`error(violation)`, passing runtime requires `generationStable=true`/no events; (b) frames sidecar has
`technologySource:"sealed-place-xml"` + evidence `{file,sha256,raw,enum,mapping}` and `look_metrics` re-derives against
`--build` (stale XML invalidates). No Play needed to check any of it.

## 4 · Plan 4 — not started (by order)

Plan 4 (atlas named-seed **assert-and-skip**, never rebuild City+Highway in Play, 45 s instrumentation trace) starts
only after the Plan 3 checker ACCEPTs. Its fixture expectations now include the §2 derived technology (atlas sidecars
and any look pack must be build-hash-bound), so Plan 4's end-to-end tests will exercise the new path as well.

## 5 · Studio / holds / spend

Studio (read-only check 01:07Z): **pid 388820**, window `0x558c7f6b0160`, 2200×1000 floating, Edit, current opening
**0/3 Plays** (the play record belongs to the previous pid 382409), place == build ==
`c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98`, sealed release `84324d43…`. No Play, no Studio
launch/close/resize/input, no build (`tools/usg build` never run), no upload/publication/purchase, no credential value,
no image reads. Only Studio interaction: none this batch (the read-only checks were `ps`/`winctl get`). $0 of $0.

## 6 · Next step + successor

**Plan 3 is frozen and ready for the fresh checker** (`90ac9326dfb0037d5af7505d64bd230d1213fb88`; reproduce block §3).
On ACCEPT: Plan 4 (freeze → fresh checker), then the single diagnostic Play sequence from batch 7 §6 — gated on Plans
3+4 freezes + ACCEPTs + the Lead's explicit go. The `traffic:abort-contact` Play will now produce bounded
`generationEvents` and `diagnostic` in its runtime report; the judging rules are unchanged (batch 7 §6).

**Evidence files:** `board/probes/team-d/plan3-generation-instrumentation-20261102.log`,
`lune-unit-plan3-20261102.log`, `sidecar-technology-ruling-20261102.md`,
`lighting-technology-capability-probe-20261101.{json,py}`; plans under `board/plans/team-d/`.
