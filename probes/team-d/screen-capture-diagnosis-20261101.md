# screen_capture diagnosis — root cause = format contract mismatch (JPEG vs PNG), 2026-11-01 13:08 ET

Read-only MCP diagnosis, **metadata only** (no image data decoded, opened, attached or displayed). Raw metadata:
`screen-capture-diagnosis-20261101.txt`.

## Exact failure

The smoke's S5 step did:

```python
raw = ctx.studio.capture()          # tools/vinegar_lib/mcp.py:239
# calls MCP tool screen_capture {studio_id, capture_id:"ScreenCapture_1"}
images = [c for c in result["content"] if c["type"] == "image"]   # found one
raw = base64.b64decode(images[0]["data"], validate=True)          # decode SUCCEEDED
if raw[:8] != b"\x89PNG\r\n\x1a\n" or raw[12:16] != b"IHDR":
    raise ExecError("screen_capture did not return a PNG")        # <-- smoke exit 1
```

The tool call itself succeeds. Direct MCP call in **Edit** mode (no Play needed) returns:

```
isError: False
image: mimeType='image/jpeg'  data_len=16552  base64 first4='/9j/'  keys=['data','mimeType','type']
```

`/9j/` is the base64 start of a **JPEG** (SOI marker), and the tool declares `mimeType: image/jpeg`. So the capture
is working — it returns **JPEG**, while the Vinegar runner hard-requires **PNG** (`tools/vinegar_lib/mcp.py:246` and
`tools/vinegar_lib/frames.py:126`). The smoke failure is a **format-contract mismatch**, not a missing viewport, not a
renderer problem, and not a Play requirement (the Edit-mode capture returned an image without a Play).

## What the tool is/needs

- Tool schema: `screen_capture(capture_id, studio_id, camera_position?, look_at_position?)` — **no format/quality
  parameter**; description: "Capture current edit-time screen, return the image data."
- Works in Edit mode (no Play), returns one `type:"image"` content item with `mimeType:"image/jpeg"`.
- Studio 0.745 / StudioMCP bridge; the same call inside the smoke's Play would return the same JPEG format.
- No host/window-visibility dependency observed (the agent never opened the window; the call succeeded offscreen).

## Smallest fix options (design decision for the Lead; nothing changed)

1. **(Recommended) Runner accepts both formats without decoding pixels**: in `vinegar_lib/mcp.py capture()`, accept
   `image/png` **or** `image/jpeg`, return `(bytes, mime)`; in the S5/fixture writers, keep the captured bytes as-is
   (`.jpg` when JPEG) and replace `frames.png_size()` with a no-pixel-decode dimension reader for PNG **and** JPEG
   (PNG IHDR; JPEG SOF0/SOF2 marker scan). Evidence shape changes PNG→JPEG (file names/type), so this needs the Lead's
   ruling — it is an evidence-format design decision, not a silent skip.
2. Keep PNG: convert the JPEG to PNG **inside Studio/Wine** before returning (no such tool parameter today; would need
   a Studio-side script/plugin — larger, and it would decode pixels inside Wine, not DSH), or convert on the DSH host
   (forbidden by the image-decode crash rule), or run the capture cells in the isolated workspace (fallback path).
3. Adapt/skip capture cells (matrix option b) — evidence degradation, Lead decision.

No `--resume` attempted; B1 held. No image read by the agent, no host input, no uploads/publication/credits.
