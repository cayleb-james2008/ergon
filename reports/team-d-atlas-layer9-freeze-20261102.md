# TEAM D — ATLAS LAYER-9 INSTRUMENTATION FREEZE (camera-type restore diagnostics), 2026-11-02

Lead approved (2026-11-02T08:34Z) the exact proposal: record `saved.type` and the post-restore `Camera.CameraType` (with
one `RenderStepped` wait + re-check) in the client cleanup payload; diagnostics only, no behaviour change, plan/validator
untouched. **Freeze commit: `7c7c1e777507001d88e91b8342cc6cc57128704e`** (`7c7c1e7`; parent `e8d18f4`; tree
`d781041d88b76d5801922b5a44951f182b316eee`; tracked clean; **2 files +29/−2**; sealed `84324d43…`/`c6651793…`
unchanged). Log: `board/probes/team-d/atlas-layer9-freeze-20261102.log`.

## 1 · Why (Play 16)

Play 16 (`20261102T082650Z-atlas-scout`) completed the **first 30/30 atlas run** — all views captured (30 distinct sha256,
`viewportMatchesPng:true` all), the four highway-zone and four world-region overheads fresh, `captureAttempts` 30 ==
`capture-attempts.json` 30, 9 × 1.0 s settles with zero retries, `fastPathApplied=true`, runtime ok, no crash — and failed
the validator only on `runtime restoration missing`: the client reported **`cameraTypeRestored:false`** (`restored:false`)
while every other restoration check was true. That path had never been reached before.

## 2 · What changed (diagnostics only)

`tests/atlas/Client.luau` `restore()` records before/after the existing restore:
`savedType`, `beforeType`, `afterType`, `afterFrameType` (after one extra `Run.RenderStepped:Wait()`), the existing
`cameraTypeRestored`, plus `cameraTypeRestoredAfterFrame` from a fresh `State.check`, and `sameCamera`/`frameSameCamera`;
it returns `(details, diagnostics)` and the `finish` payload carries **`cameraTypeDiagnostics`**. The restoration verdict
(`checkAll(details)`), the client checks and the validator are **unchanged**; no plan/validator edit.

## 3 · Tests / freeze evidence

- New test: source-shape pins for every recorded field and the payload key, plus a validator-tolerance test (an extra
  `cameraTypeDiagnostics` key on the client report does not affect `validate_report`).
- **M30** (diagnostics payload key removed) → atlas acceptance **FAILED (1)**; restored, tracked clean.
- Suites: atlas acceptance **25 OK**; runner **91 OK**; full Python **334 OK**; Lune atlas **10/0 (4 specs)**; full Lune
  **924/0 (127 specs)**; sealed unchanged; **checker-ready**.

## 4 · Next

Fresh-checker delta → on ACCEPT, **one atlas Play in the current recovered opening (1/3 used, 2 reserved)** to read
`cameraTypeDiagnostics` and name the camera-type restore cause (no real fix before that). The abort-contact item is on
HOLD pending Nox; the C achieved-vs-commanded evidence note is filed at
`board/probes/team-d/abort-contact-c-achieved-acceleration-20261102.md`.
