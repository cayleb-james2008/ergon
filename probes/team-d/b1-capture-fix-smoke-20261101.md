# B1 capture-contract fix + smoke PASS (2026-11-01 13:12–13:16 ET)

Fixes are on **`prep/b1-capture-format-20261101`** (branch base `b67abdd`; only runner/tooling docs changed, no game
code, no build, no re-pin):

| Commit | Change |
|---|---|
| `32919dd` | `mcp.py capture()` returns `(raw, mime)` and accepts **PNG or JPEG**; `frames.py` adds `image_mime`/`image_ext`/`image_size` with a **header-only** JPEG SOF (SOF0–SOF15) and PNG IHDR dimension reader; all 4 capture sites write the captured bytes with the real extension (`.png`/`.jpg`) and record `mimeType` + `sha256`; `sidecar()` gains `mimeType` (contract keys `png`/`viewportMatchesPng` kept for `look_metrics`); runner docstring + `docs/art/SIDECAR.md` note the widened contract (why: Studio 0.745 MCP returns `image/jpeg`; in-Studio JPEG→PNG conversion not possible — no tool format parameter) |
| `d079f55` | smoke S5 leftover `info["png"]` → `info["image"]` (KeyError found by the resume run) |
| `fe2bbdb` | smoke S5 now records `sha256`/`secondSha256` + `secondMimeType` (Lead condition 3: evidence carries mimeType + sha256) |

**Latch side-note:** the first resume failed with `verify-clean failed: Source differs from the checkout:
tests/studio/highway.spec.luau` because the open place had been synced from the A1 prep branch (aligned spec) while the
B1 checkout is based on `b67abdd`. Fixed by re-syncing the place from the B1 tree (`{"synced":101}`), `verify-clean`
clean, `--clear-latch` (`latchCleared:true`). Operational only; no data lost.

## Smoke result after the fix — **PASS, exit 0**

Run dir `docs/verify/2026-10-19/vinegar/20261101T181436Z-smoke/` (branch `prep/b1-capture-format-20261101` @
`d079f55`, sealed place `c6651793…`):

```
{"smoke": 0, "error": null,
 "checks": {"S1 Edit": true, "S2 unpublished local place": true, "S3 code sizes up to 64 KiB": true,
            "S3 12 KB and 90 KB responses intact": true, "S4 600 KiB loopback POST": true,
            "S4 HttpEnabled restored": true, "S5 Play, capture, Stop, verify-clean": true},
 "findings": {"response110KBTruncated": true, "s4CollectorClose": {"clean": true, ...},
              "pngMatchesViewport": false, "secondCaptureDiffers": true, "mcpCameraHeld2s": false}}
{"queue": "paused", "reason": "3 Plays in this Studio: owner reopens Studio, then --resume",
 "next": "traffic:abort-contact"}
```

S5 metadata: `image=[1920,786]`, `mimeType="image/jpeg"`, `viewport=[2769,1134]`, `cameraDelta=1040.52`,
`verifyClean.sources=101` — captures were written as **`s5-play.jpg`** (247,903 B, sha256
`2f1ba0fdf60d58ab5137b3570faa10bac588011be4997275c06dfe4d84a4e350`) and **`s5-camera.jpg`** (195,201 B, sha256
`523efc4ca6e1a14a6b449ec0f06964a06d939a5d802e60a2bf73bc38c7337e5d`); both start `ff d8 ff e0` (JPEG SOI). No agent read
or decoded either image — metadata/hashes only.

## Findings to carry (smoke passed; these are recorded findings, not failures)

1. `pngMatchesViewport: false` — the capture is scaled to **1920×786** while the probed viewport is **2769×1134**.
   `viewportMatchesPng` in later sidecar captures will therefore be false; look/atlas cells that require exact viewport
   match may need a capture-resolution decision (tool has no resolution parameter). Flag for the Lead before B3 look
   cells; do not improvise.
2. `mcpCameraHeld2s: false` (cameraDelta 1040.5) and `secondCaptureDiffers: true` — S5's camera-move findings as the
   smoke reports them; not a smoke failure.
3. The 3 Play budget for this Studio process is spent (1 failed capture-format smoke + 2 retry smokes). **Owner must
   reopen Studio** (close Don't Save, relaunch the one-line place command) before `--resume`; the queue's next entry is
   `traffic:abort-contact`. Because failed attempts consumed Plays, the remaining v1.queue entries (3×
   traffic:abort-contact, atlas:scout, atlas:highway-extension, moving) now span two openings instead of two of three.

No host input, no agent image reads, no uploads/publication/credits; sealed artifacts unchanged
(`84324d43…`, `c6651793…`); Studio in Edit, `verify-clean` clean before the pause.
