# Memo — capture resolution vs recorded viewport (B3 pre-work, 2026-11-01)

Prepared for the Lead; no Studio action taken (hold in force). Facts are from the passing B1 smoke and code reads.

## What is observed

- Studio MCP `screen_capture` returned a **1920×786 JPEG** while the Play probe recorded **viewport 2769×1134**
  (`docs/verify/2026-10-19/vinegar/20261101T181436Z-smoke/smoke.json`, S5; images `s5-play.jpg` / `s5-camera.jpg`).
- The scale is uniform: 1920/2769 = **0.69339**, 786/1134 = **0.69312** (≈ 1/1.4428). So the capture is a resampled
  image of the same view, not a crop of a larger frame.
- The tool schema has **no format/size parameter** (`screen_capture(capture_id, studio_id, camera_position?,
  look_at_position?)`), so the scaling is decided by Studio/MCP, not by the runner.

## Why it matters (exact blocker)

`tools/look_metrics.py` is the independent look judge used by B3/atlas cells. It requires the before/after camera
frame, FOV and viewport to be recorded **and the image to hold that viewport**:
`capture_validity()` line ~274: `image holds {w}x{h} px, recorded viewport is {vw}x{vh}` is a rejection reason (only a
≤1%-of-width clip is a warning). `find_viewport()` either treats an image equal to the viewport as the view, or looks
for Studio's blue border. With a 1920×786 image and a 2769×1134 recorded viewport, every such capture is **invalid**
for look scoring. Consequence: until ruled, look/atlas cells that require an exact viewport match are
**BLOCKED-pending-resolution** — never passed, never failed.

## Candidate explanations (not yet discriminated)

1. **MCP capture caps the long edge at 1920 px** (most likely; common for capture bridges). Prediction: a smaller
   Studio window (viewport width ≤ 1920) returns an image equal to the viewport.
2. **Wine/DPI scaling** fixed factor (0.693 ≈ 1/1.4428; not the usual 1.5/1.25 factors, so less likely).
3. **Chrome/window mismatch** (image holds only part of the window) — contradicted by the uniform ratio and by S5's
   `answer = first["viewport"]` being the full Play viewport.

## Options

- **A. Accept scaled captures and normalise in the judge**: map recorded viewport coordinates into image coordinates by
  the measured uniform scale, then compare in normalised space. This is a **design change to look_metrics** (mask
  thresholds MIN_PIXELS, hue/luma windows and 4-stud sampling are defined in pixels/studs; they would need rescaling
  and re-baselining against the locked anchor board). Highest effort, changes the judge contract.
- **B. Keep exact match by fitting the window**: if the cap is 1920, an owner-resized Studio window with viewport width
  ≤ 1920 would make image == viewport (owner GUI act, not agent). Cheapest if explanation 1 holds; needs one
  discriminating capture.
- **C. Change the capture path**: no tool parameter exists; in-Studio conversion has no hook; host-side decode is
  excluded by rule. Not preferred.

## Evidence that would decide (one short owner window, no host input by the agent)

1. **One capture at a different Studio window size** (owner resizes, no agent input): if image == new viewport →
   explanation 1 confirmed and option B is viable; if image stays ~1920 wide or scales by the same factor → cap by
   long edge or fixed scaling; if image shrinks by a different rule → DPI.
2. **Check the Studio/MCP capture contract** for a documented max dimension (read-only docs search; external data).
3. **Judge tolerance**: whether look_metrics thresholds can be expressed scale-invariantly (code read, then a
   calibration decision) — needed only if option A is chosen.

Recommendation: get evidence (1) before B3 and rule between B and A; until then mark exact-viewport look/atlas cells
BLOCKED-pending-resolution. B2's pass/fail validators are probe-based and do not need this ruling unless a cell's
evidence is a look image.

## `mcpCameraHeld2s: false` (cameraDelta 1040.5) — relation to later camera cells

Independent of the resolution issue. S5 moves the camera +20 studs, waits 2 s, then compares the re-probed camera to
that target; the delta means the camera did not stay where S5 put it for 2 s (likely the Play session's own camera
behaviour after `play_start`, not a capture fault). Later atlas/`moving` cells do **not** use the 2 s S5 hold: they
compare the probe immediately before and after each capture and require ≤1e-3 camera delta (`frames.CAMERA_TOLERANCE`)
and classify `moving` from the same probes. So this finding does not block them by itself; cells that require a camera
held over seconds (none in the current B1–B4 set beyond S5's own check) would need a separate decision. It stays
recorded as a smoke finding and should be re-checked on the next smoke run for trend only.

## Addendum — discriminating test design (Lead-approved in principle, designed 2026-11-01 13:25 ET)

**Zero-Play plan (preferred):** Edit-mode `screen_capture` already works (we received an `image/jpeg` in Edit at
13:08 with no Play), and Edit `execute_luau` can read `workspace.CurrentCamera.ViewportSize`. A no-pixel-decode probe
can pair them: capture dims from the JPEG SOF marker (header bytes only) + the recorded Edit viewport (+ optionally the
Studio X window geometry, read-only `XGetGeometry`). Take that pair at **two Studio window sizes**.

- **Owner act: resize the Studio window only** (drag an edge, e.g. to roughly half width), no close, no Play, then say
  "resized". Then (optionally) resize back.
- Decides the hypotheses: if image dims follow the viewport proportionally → "capture tracks window"; if the image stays
  at the 1920 long edge → "capture capped at 1920". If Edit is representative of Play (same capture path), this costs
  **zero Plays**.
- If the two Edit pairs are ambiguous (e.g. viewport ≤1920 at both sizes, so cap and track predict the same image), the
  fallback is **one Play** at the smaller window (a single smoke-style capture), still one Play maximum.
- Method: MCP `execute_luau` (Edit) for the viewport, MCP `screen_capture` (Edit) for the image, header-only dimension
  parse; the agent never opens/decodes the image; the PNG/JPEG bytes stay in the runner/evidence path.

Only if the evidence forces option A would look_metrics change (a one-page amendment to Nox, not this team's call).


## RULED 2026-11-01 13:33 ET — cap-at-1920 confirmed, option B, no judge change

Size 2 (fitted window 2280×1000, Edit): viewport **1917×744**, capture **1917×744**, ratio **1.0**. With the two larger
samples (1920-wide closes at different viewports/ratios), this proves a **long-edge cap at 1920**: above it the capture
is clamped, at/below it the capture is pixel-for-pixel the viewport. **Decision: option B — fitted window is a recorded
runbook precondition (winctl + `ViewportSize ≤1920` check + one Play-mode `viewportMatchesPng:true` confirmation); no
look_metrics change; exact-match stays the rule.** Option A is not needed and no amendment to Nox is required. Keep
B3/atlas exact-viewport cells **BLOCKED-pending-fitted-window-confirmation** until the precondition is applied and the
first Play capture confirms it.
