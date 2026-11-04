
---

## ADDENDUM — final state after two fresh checkers (4 Nov 07:00Z / 02:00 ET)

**The set to grade is `renders/20261104T0725Z/`** (it supersedes `0650Z` named above, and the other seat's `064858Z`):
179 PNGs, **446 rows, 420 pass / 26 fail**, `dirty: false`, `dirty_paths: []`, build `0c357fc2…` (332,145 B / **100,955
gzip**, the build's own figure), tool `5b0fab7e…`, 118/118 expected captures, inventory 179/179. No cutover: `:8791`
still serves `849e2c1c…`.

The app changed under this stream once more: commit `90a2c48` (another OPENBELL seat) fixed a real text-on-text collision
in `preview-missing` at 1440 and added "a repeated MISSING cause prints once", so the page went `e641db13…` →
`0c357fc2…`. Both earlier sets therefore measure a page that is no longer HEAD; `0725Z` is the first set that measures
the current page with the current tool.

### Two checkers, same verdict, same three doc defects

Checker 5 (mine) and checker 6 (the other seat's) both returned **PASS WITH KNOWN SPEC CONFLICT** and both found the same
documentation overstatements in `RESOLUTION.md`, now corrected in `9b32789`:

| claim | was | is |
|---|---|---|
| gzip | 100,442 | **100,955** (the build's own figure; 100,442 was Python zlib-9 on the same bytes) |
| `frozen` | "hidden and shown" | **3 captures, canvas-off only** — the freeze is a DOM state, the 3D layer is not refrozen; A30's literal list is 120 frames vs the 118 shipped, and that deviation is now stated, not hidden in a passing row |
| padding | "12/14" | **12 / 14 / 12 / 20** (spec 12/14 plus the 20 px glyph gutter) |
| the 18 px probe | ambiguous | capping the **box width**; a font-size cap does not reproduce it |

Checker 6's reproduction is the strongest evidence this pack has had: a full run with **0 of 446 verdict differences** and
**179/179 PNGs byte-identical**; checker 5 re-captured 71 images with none differing. Both re-derived A29's numbers from
the images alone and both confirmed the `figures` rule bites in both directions (a 96 px number cap → 2 lines; a narrow
word box → `"not running" 3 line(s) of 2 word(s)`; unmutated → 0 figure failures).

### Three gate defects they found, fixed in `f6014f1`

1. **Hang**: `serve()`'s `close()` awaited `once('exit')` on a child that was already reaped, so killing the preview
   server mid-run left the gate alive for ever with no `results.json`. It now returns at once and escalates
   SIGTERM → SIGKILL. Proof: `OLD close(): HUNG (no return in 3000 ms)` vs `NEW close(): returned`.
2. **Population**: A29 (iii) judged only the boxes the scene reported, so a body that failed to mount shrank the
   population and the row could pass. New row `a29 (iii) body population` compares the scene's front boxes with the
   front plates that declare `data-body` (reads "6 … vs 6" here).
3. **A30 compliance**: the notes now carry the A29 tone-step record (none baked in) with the path to the three-step
   evidence, which A30 requires.

### Still open — unchanged, both need Nox/owner

- **A29 (iii)**: 6/6 front bodies fail at 0 steps; the spec's three-step remedy reaches **2/6** ring contrast and leaves
  coverage below 0.60 on 4/6, and worsens the band clause (1.227 → 1.212). Unreachable inside its own ceiling.
- **A8 viewport**: 24 rows, 203–316 px below the fold at 1440×900. Checker 6 added the sharpest phrasing: the **front**
  row ends at 817.7 px (inside the window) and the whole **shelf** row starts at 829.7 px, so at 1440×900 the shelf is
  not "a few pixels over" — it is entirely below the fold. That is a layout decision for the owner, not a bug to patch.
