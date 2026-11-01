# Capture-scaling test — size 1 recorded (2026-11-01 13:26 ET, Edit MCP only, no Play, no image decode)

Method: MCP `execute_luau` (Edit) read `workspace.CurrentCamera.ViewportSize`; MCP `screen_capture` (Edit,
capture_id `size-size1`); dimensions parsed **header-only** from the JPEG SOF marker; sha256 of the captured bytes
recorded; Studio window geometry read read-only from the compositor (`hyprctl clients -j`). The agent never decoded,
opened, attached or displayed the image. Raw sample: `capture-size-size1-20261101.json`.

## Size 1 (current window, owner has not resized yet)

| Measurement | Value |
|---|---|
| Edit viewport (`CurrentCamera.ViewportSize`) | **3053 × 1134** (cameraType `Enum.CameraType.Fixed`) |
| Edit capture | `image/jpeg`, **12413 B**, dims **1920 × 713**, sha256 `66be3f76852797475d91248ceac8dcb1420437bfe646822cbce156071c4c268a` |
| Studio window (compositor, outer) | at `[12,38]`, size **3416 × 1390**, pid 254971, title `…/usg/usg-desktop.rbxlx - Roblox Studio` |
| image / viewport ratio | **0.628890** w · **0.628748** h (uniform) |

## Preliminary reading (size 2 still owed)

Two independent samples now exist at different viewport widths:

| Sample | viewport | capture | image width | ratio |
|---|---|---|---|---|
| Play (B1 smoke S5) | 2769 × 1134 | 1920 × 786 | **1920** | 0.6934 |
| Edit (size 1) | 3053 × 1134 | 1920 × 713 | **1920** | 0.6289 |

Both captures are **exactly 1920 px wide** although the viewport widths differ (2769 vs 3053) and the ratios differ
(0.6934 vs 0.6289). A fixed proportional scale would give *different* image widths; a fixed ratio would give the same
ratio. The observed pattern is a **long-edge cap at 1920 px**: width is clamped to 1920, height follows the viewport
aspect. That is evidence for "capped at 1920", i.e. option **A territory** unless size 2 shows otherwise.

The size-2 sample (after the owner resizes the window) still matters: with a viewport **≤ 1920 wide**, the cap predicts
image == viewport exactly (no further downscale), while a fixed 0.629 scale predicts ≈ 0.629 × viewport. That is the
clean discriminator between "capped (so a fitted ≤1920 window gives exact match → option B viable)" and "fixed
proportional scale (option A)".

No Play was started; no Studio state was changed (Edit-only MCP calls); sealed hashes unchanged.


## Size 2 (owner/Lead fitted floating window 2280×1000) — recorded 13:32 ET → **decision: CAP AT 1920 CONFIRMED**

| Measurement | Value |
|---|---|
| `winctl get --target studio` | address `0x558c7fa7b7f0`, class `steam_proton`, **floating true**, at `[581,233]`, size **2280 × 1000**, pid 254971, title `…/usg/usg-desktop.rbxlx - Roblox Studio` (audit log `~/.local/state/winctl/actions.jsonl` records the `set-size` action) |
| Edit viewport (`ViewportSize`) | **1917 × 744** (cameraType Fixed) |
| Edit capture | `image/jpeg`, 12,855 B, dims **1917 × 744**, sha256 `e19b0f8d92cba0bed379896e94f2332b0bd04a3d4db9b0f166a361e5ca76f0d1` |
| image / viewport ratio | **1.000000 / 1.000000** (exact match) |
| compositor geometry (hyprctl) | same as winctl: at `[581,233]`, size `2280 × 1000` |

### Decision

**Cap at 1920 long edge — CONFIRMED.** Three samples:

| Sample | window | viewport | capture | ratio |
|---|---|---|---|---|
| Play, tiled | 3416×1390 | 2769×1134 | 1920×786 | 0.6934 (capped) |
| Edit, tiled (size 1) | 3416×1390 | 3053×1134 | 1920×713 | 0.6289 (capped) |
| Edit, fitted (size 2) | 2280×1000 | **1917×744** | **1917×744** | **1.0 (no scaling)** |

Above the cap the long edge is clamped to 1920 (width fixed, height follows aspect); at or below it the capture is
**pixel-for-pixel the recorded viewport**. ⇒ **Option B is viable: a fitted window restores exact viewport match, no
look_metrics change, exact-match stays the rule.** The fitted window becomes a recorded runbook precondition, not a
contract change.

### Recorded runbook precondition (B3 look cells and any exact-viewport cell)

1. `winctl get --target studio` must show `floating: true` with window width **≤ 2200** (a safety margin; observed Edit
   viewport = window width − **363** px, so 2280 gives 1917 = only 3 px under the cap — do not run look cells at 2280;
   2200 gives ≈1837, an 83 px margin). Height: viewport = window height − **256** px.
2. Then MCP `execute_luau` (Edit) must record `workspace.CurrentCamera.ViewportSize` with **both dims ≤ 1920** before any
   look capture.
3. The first **Play-mode** capture at the fitted size must show `viewportMatchesPng: true` in its sidecar (the cap
   behaviour was proven in Edit; Play should behave the same because Play's viewport ≈ window − 647 px horizontally at
   the old layout, but this must be observed, not inferred). If it is false, stop and report before B3.
4. If the fitted condition is not met: the owner/Lead applies `winctl set-size --target studio 2200 1000`
   (`set-size` floats if tiled and verifies). The agent does not resize windows without authorisation.


## Size 3 — fitted-window precondition CONFIRMED at 2200×1000 (2026-11-01 13:34 ET, Edit-only, no Play)

| Measurement | Value |
|---|---|
| `winctl get --target studio` | floating true, at `[621,233]`, size **2200 × 1000**, pid 254971 |
| `winctl verify --width 2200 --height 1000` | `{"ok": true, ...}` rc 0 |
| Edit viewport (`ViewportSize`) | **1837 × 744** (cameraType Fixed) |
| Edit capture | `image/jpeg`, 12,367 B, dims **1837 × 744**, sha256 `01f554843841ff4991425c530b3b3b43448eb2e5d1d407d37b6b22c4f023e090` |
| image / viewport ratio | **1.000000 / 1.000000** |
| margin under the 1920 cap | **1920 − 1837 = 83 px** (the recommended ≥60 px) |

Observed chrome at this DPI is stable: Edit viewport = window − (363, 256). The fitted-window precondition is
**satisfied in Edit**: a fitted 2200-wide window captures pixel-for-pixel the recorded viewport. Exact-viewport/look
cells remain **BLOCKED-pending-Play-confirmation** until the first Play-mode capture at this window shows sidecar
`viewportMatchesPng: true` (a B1 atlas/moving cell will provide it; B3 does not start before that).
