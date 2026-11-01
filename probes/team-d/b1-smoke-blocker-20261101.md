# B1 opening 1 — smoke stops on a screen_capture failure; HOLD for a ruling (2026-11-01 13:04 ET)

A1 is GREEN; B1 was unblocked and the V1 queue's opening 1 (Plays 1-3) was started per the runbook:
`python3 tools/vinegar_run.py queue docs/verify/2026-10-19/vinegar/v1.queue`
Log: `board/probes/team-d/b1-queue-opening1-20261101.log`; run dir:
`docs/verify/2026-10-19/vinegar/20261101T180431Z-smoke/smoke.json`.

## Result

```
{"smoke": 1, "runDir": ".../20261101T180431Z-smoke",
 "error": "ExecError: screen_capture did not return a PNG",
 "checks": {"S1 Edit": true, "S2 unpublished local place": true,
            "S3 code sizes up to 64 KiB": true, "S3 12 KB and 90 KB responses intact": true,
            "S4 600 KiB loopback POST": true, "S4 HttpEnabled restored": true},
 "findings": {"response110KBTruncated": true, "s4CollectorClose": {"clean": true, ...}}}
{"queue": "stopped", "entry": "smoke", "exit": 1, "resume": "--resume retries this entry"}
```

- S1–S4 PASS (Edit state, unpublished local place, Luau payload sizes, collector loopback).
- **S5 failed**: the smoke's Play started, then the runner's own `screen_capture` step
  (`tools/vinegar_run.py` S5, `ctx.studio.capture()`) raised "screen_capture did not return a PNG" —
  the MCP `screen_capture` result contained no PNG image content. This is **exit 1 = a real finding**; the queue stopped.
- After the failure the abort path ran: `verify-clean` **exit 0, `clean: true`**, latch `null`; status shows the
  Studio process has **1 Play used** (2 remain before a reopen), queue stopped at `smoke`.

## The conflict that needs a ruling

B1's smoke (S5) and many later fixture cells capture viewport frames through the Studio MCP's `screen_capture` and
write PNG evidence. The current hard-stop ruling says **no `screen_capture` and no image reads** because the DSH host
crashes decoding images. The smoke's use is *inside the runner* (it writes `s5-play.png` / `s5-camera.png` to the
repo's own evidence directory; the DSH agent never reads or displays them), but the tool call is still
`screen_capture`, and it is currently **failing anyway** ("did not return a PNG") — so there are two separate issues:

1. **Permission/constraint**: may the runner use `screen_capture` for its own evidence files while DSH never decodes
   an image? (This is a design/security call, not mine to make.)
2. **Function**: even if permitted, `screen_capture` returns no PNG in this Studio/Wine session right now — a real
   capture defect that the smoke test is doing its job to surface. Not retried (exit 1 = keep the failure).

## State / next

- No host input, no images read/displayed by the agent, no uploads/publication/credits; candidate still
  `b67abdd`/`84324d43…` (place file `c6651793…` unchanged); Studio open in Edit; place clean, latch free,
  1/3 Plays used.
- **HOLD** for the Lead's ruling: (a) allow the runner's internal `screen_capture` for evidence with a no-decode
  guard, (b) adapt/skip the capture cells (changes the evidence shape — a design decision), or (c) move B1 to the
  isolated workspace (`agent-workspace-linux`) under the earlier fallback rule. Do not retry `--resume` before the
  ruling.


## DIAGNOSIS (13:08 ET) — format contract mismatch, not a broken capture

Direct read-only MCP `screen_capture` (Edit mode, no Play) returns **`mimeType: image/jpeg`**, `data_len=16552`,
base64 prefix `/9j/` (JPEG SOI), `isError: false`. The tool has **no format parameter** and works offscreen without a
Play. The runner's `capture()` (`tools/vinegar_lib/mcp.py:239-246`) base64-decodes the image and then **requires the
PNG signature** (`frames.png_size`, `:126`) — hence "screen_capture did not return a PNG". It is a JPEG-vs-PNG contract
mismatch between the current Studio MCP and the Vinegar runner, not a viewport/renderer/Play problem. Full metadata:
`screen-capture-diagnosis-20261101.txt` and summary `screen-capture-diagnosis-20261101.md`.
