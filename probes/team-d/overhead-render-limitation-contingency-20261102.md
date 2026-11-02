# ATLAS OVERHEAD FRAMING — NAMED RENDERER/WINE LIMITATION (adopted per Nox run-7, 2026-11-02)

**Ruling:** Nox 20261102T051200Z adopted **B + A**; **C** rejected for now (settings/draw-distance experiments change
what the cell tests; revisit only if per-zone views hit the same freeze); **D** rejected.

**A — the limitation, named.** The single **whole-highway overhead framing** (`highway-overhead`, camera fitted to the
whole highway model) **fails closed on this build: Studio `0.745.0.7451137` / Wine 11.0 (Vinegar 1.9.4, proton prefix
`studio`)**. Evidence (Play 7, run `20261102T045824Z-atlas-scout`, 2026-11-02T04:58:24Z): after the bounded **1.0 s
overhead render settle**, **three captures 1.0 s apart returned byte-identical frames** (`sha 74d4f7e9934dd49d…`, the
view-020 `city-overhead` image) with **`cameraDelta:0`**; the client had acknowledged view 021; `capture-attempts`
audit **21/21 consistent** (`result.json.captureAttempts` == `capture-attempts.json`). Verdict for that one framing:
**fail closed (named), never a pass claim.** Supporting/adjacent evidence: Play 6's intermittent **userland** Studio
minidump (106,116,768 B at Play shutdown after `StopPlaySoloShutdownDataModels`) did **not** repeat in Play 7 (clean
abort); `gpuJournal:null`; no kernel `amdgpu|MES|ring|GPU reset` lines. The stall/crash mechanism is **not** established
beyond the persistent-previous-frame fact; no kernel GPU fault is observed.
**Retirement rule:** one retest of the whole-highway framing per **new Studio build** is authorised; if it captures a
fresh frame, the limitation closes (record the build id + capture evidence).

**B — adopted substitution (implemented).** The single whole-highway overhead is replaced in the scout plan by **four
bounded per-zone functional overheads** (zone views `highway-zone-1..4-overhead`, each anchored on ONE native highway
road: `Highway/Roads/Road_h_top_05__h_top_06`, `…h_east_03__h_east_04`, `…h_access_030__h_access_031`,
`…h_access_062__h_bottom_09`). They go through the **same audited capture path** (settle/retry/`capture-attempts`
consistency, fog-capability and viewport records); the sealed place is untouched. Scout plan: **27 views** (24 − 1 + 4).

**C/D:** C rejected for now; D rejected. No plan/validator change beyond the B substitution.

## History (pre-ruling analysis, retained)

**Status update 2026-11-02T04:59Z (Play 7):** the 1.0 s overhead render settle did **not** unfreeze the frame —
view-021 `highway-overhead` returned view-020's exact bytes on all three attempts after a 1.0 s settle
(`sha 74d4f7e9934dd49d…`, cameraDelta 0, attempts recorded; audit == result list, 21/21), and the client had acknowledged
view 021. No Studio crash this Play (abort clean, `newCrashDumps:null`). So the **limitation is established at the
"1.0 s bounded settle insufficient" evidence level**; the options below are now for the Lead/Nox ruling, and no
plan/validator change is implemented. See `board/reports/team-d-diagnostic-play7.md`.

Author: Team D (`roblox-lead`). Date: 2026-11-02. Offline only — **no plan/validator change implemented**. For the
Lead/Nox ruling **if the 1.0 s overhead settle fix does not unfreeze the frame** in the next Play.
Evidence base: `board/reports/team-d-diagnostic-play5.md`, `board/reports/team-d-diagnostic-play6.md`;
run `docs/verify/2026-10-19/vinegar/20261102T042225Z-atlas-scout`; layer-5 fix
`board/reports/team-d-layer5-settle-fix-20261102.md`.

## 1 · The finding (what the evidence supports)

- Plan order: view-020 `city-overhead`, view-021 `highway-overhead`, view-022 `world-overhead` (plus `tile`/`cell-roof`
  kinds in other batches). The stall appeared at the **first overhead→overhead transition after `city-overhead`**, i.e.
  the `highway-overhead` view, whose camera fit spans the whole highway.
- **Three captures 0.3 s apart were byte-identical to view-020** (`sha 0ac9595d3372a880…`, the previous overhead frame),
  while the client had acknowledged view-021 (`view-021` in the collector phases) and its two-settled-frame camera check
  had passed. So: the pose changed but the **capture path kept returning the previous rendered frame** for at least
  ~0.6 s.
- After the fail-closed stop (abort `stop:"ok"`), Studio **crashed**: new **userland minidump 106,116,768 B**
  (`2742f914-…dmp`, 2026-11-02T04:22:48.583Z); the Studio log ends right after `StopPlaySoloShutdownDataModels` begins;
  `cleanup`/`verifyClean` returned `Disconnected('no response to tools/call within 180s')`; `newCrashDumps:1`,
  `exit 5`. **`gpuJournal:null`**, no kernel `amdgpu|MES|ring|GPU reset|drm` lines in the window, hostcheck `{stalls 0,
  label "clean"}`.
- Sealed place/build hashes were unchanged before/after the crash and the `studiocycle open` recovery
  (`c6651793…`/`84324d43…`); 20/24 views captured repeatably (20 distinct sha256, `viewportMatchesPng:true`).

## 2 · What the evidence does NOT establish (must not be overclaimed)

- **No kernel-level GPU fault is observed.** The only crash artefact is a userland Studio/Wine minidump; there is no
  amdgpu/MES/ring-reset line in the host journal for the window. The crash is therefore **not proven to be a GPU driver
  fault** — it is proven to be a Studio/Wine process crash after the Play stop.
- **The stall mechanism is unproven.** "The renderer had not finished the expensive whole-highway frame within ~0.6 s" is
  the leading hypothesis (it fits the previous-frame persistence and the frame's cost), but it cannot be distinguished
  from a capture-side cache/reuse without instrumentation of the capture path (e.g. a Play-side `ViewportFrame`/render
  counter, not available through the allowed tools) or without a longer-settle Play.
- Whether `world-overhead` (an even larger fit) would behave the same is unknown (never reached).

## 3 · Options if the 1.0 s settle does not resolve it (FOR RULING — none implemented)

| Option | Description | Cost / risk |
|---|---|---|
| **A. Documented fixture limitation + named view** | Keep the plan but record the overhead view as a named, documented limitation: the atlas cell is reported as "views 1–19 captured; overhead views blocked by renderer/Wine stall" rather than a pass. No evidence is fabricated. | Honest but the cell cannot pass; `look`/B3 cells that need the overhead views stay blocked. |
| **B. Replace the single whole-highway overhead with per-zone functional views** | Split `highway-overhead` into bounded per-zone tile/overhead views (like the `tiles-fogoff` batch's bounded tile views) so each frame renders a bounded area. This is a **plan/validator change** (map_atlas plan, atlas_acceptance counts, capture index) and needs its own checker delta. | Larger change; must keep anchors/coverage semantics and the frozen seed; needs a fresh plan and a new baseline. |
| **C. Reduce the overhead framing cost** | E.g. take the overhead capture at a smaller viewport or with streaming/draw limits for that view. | Touches production look/streaming or the capture contract; not recommended without Nox (frozen capture-resolution decision). |
| **D. Cap/lower the plan's overhead set** | Drop the 3 overhead views from the scout plan (documented). | Weakens the cell; needs a plan ruling. |

**Recommended if the settle fails: B + A together** — per-zone bounded views as the durable fix, with the current
whole-highway overhead kept as a named documented limitation until the per-zone plan is frozen. Nothing is implemented
here; the next Play decides between "settle works" (no change) and one of the above.

## 4 · What evidence would decide it next

A Play on the layer-5 fix: `captureAttempts` will record `kind`, `settledSeconds` and each attempt's sha/camera delta. If
the first capture after a 1.0 s settle is fresh, the hypothesis is confirmed and no plan change is needed. If all three
attempts are still byte-identical to the previous view (and camera delta is within tolerance), the frame is genuinely
stalled and option B/A needs the ruling.
