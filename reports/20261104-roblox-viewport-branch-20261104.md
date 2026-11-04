# ROBLOX stream — prepared branch held for Nox's ruling (2026-11-04)

Second filing from the same overnight lane; the main record is
`20261104-roblox-stream-lead-20261104T0353Z.md`.

## `prep/look-metrics-viewport-guard` @ `618316f` — built, tested, **NOT merged**

The defect (Team D's own, 3 Nov): `capture_validity` judges one capture against its own sidecar, so a pack could mix a
**1837×744** view with a **282×100** one and all 18 edge views still passed as "valid". The freeze on `main`
(`docs/art/thresholds.json`) took its worst fleck reading (8253) from the 282×100 image as a result. Two views of the
same shot at different window sizes are not comparable, so a look verdict cannot rest on them.

What the branch adds:

- **One viewport per pack** (unambiguous, no parameter): every capture that records a viewport must record the same
  one; a capture in a mixed pack is marked invalid with the reason naming the sizes, and the pack is still measured and
  reported (same contract as every other invalid view). A capture with no recorded viewport stays `capture_validity`'s.
- **`--min-viewport WxH`** — the minimum size is a **ruling parameter** (Team D proposed 1600×700), so it is **off by
  default**; the report records `viewportRule: {onePerPack: true, minimum: null | [w, h]}` so a reader can tell whether
  it ran. Switch-on is one flag, no code change.
- Three new tests: a mixed pack judges 0 views and names the sizes; a consistent pack is unaffected; the minimum is off
  by default and invalidates below it; the argument refuses anything that is not two positive integers.
- **Full tool suite on the branch: 372 tests OK** (369 on `main` + 3), 1 skipped. Nothing on `main` changed.

Deliberately not done on this branch: no threshold values (fleck `all` ≤ 270, cyan `all` ≤ 0.63, the scoped void bound),
no refreeze, no re-capture of the edge views. Those are the same ruling, and the values must come from the owner-side
decision, not from the tool.

## What Nox needs to answer, in one line each

1. **Yes/no** to the viewport guard, and the **minimum size** (Team D proposed 1600×700) — then the branch merges and
   `--min-viewport 1600x700` becomes the documented default in `docs/art/THRESHOLDS.md` and the runbook.
2. **Yes/no** to the scoped void bound (`max 0.05` at `edge-*` views) and to the 20 % margin freeze (fleck ≤ 270,
   cyan ≤ 0.63).
3. Only after 1-2: re-capture the atlas:edge views at the ruling viewport (3 parts, `--edge-part I/3`, ~3 Plays) and
   refreeze from comparable captures. That step needs Studio, so it also needs the owner present.

## Studio state tonight

No Studio is running (`pgrep -f '^[^ ]*/RobloxStudioBeta\.exe'` → nothing), so the contained instance Team D left open
is closed and the Wayland window-finder fix is now testable the moment someone can open a Studio. Nothing was launched
tonight and `studiocycle cycle` was not run.
