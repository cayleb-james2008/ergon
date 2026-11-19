# rune/used local edits

These parts were edited in place after `sync-used` copied them. `frontend/scripts/sync-used.mjs` skips any path listed here, so a sync never reverts these edits. Each edit is also described in `frontend/DESIGN.md` under "Local edits to parts".

- `components/ui/badge.tsx`: transitions list explicit (no transition-all, no box-shadow); focus ring ink.
- `components/ui/button.tsx`: transitions explicit; focus ring ink; disabled channel adds fill and ink (no opacity-only disabled).
- `components/ui/expandable.tsx`: transition on opacity and transform only.
- `components/ui/input-group.tsx`: transitions explicit; focus ring ink.
- `components/ui/input.tsx`: transitions explicit; focus ring ink.
- `components/ui/segmented-control.tsx`: indicator moves on translate only; focus ring ink.
- `components/ui/textarea.tsx`: transitions explicit; focus ring ink.
- `components/kibo-ui/pill/index.tsx`: disabled channel and focus ring not in this part; only the mask stays as is (flagged, not changed).
- `components/ui/input-group.tsx` (round 3): the focus-within border rule became a surface fill and the disabled opacity rule became track fill and disabled ink (no opacity-only disabled state, no focus through the border).
- `components/kibo-ui/pill/index.tsx` (round 3): the mask gradient uses `currentColor` instead of the literal white.
- `components/ui/command.tsx` (round 3): the disabled input uses the disabled ink, not opacity alone.
- `components/ui/button.tsx`, `badge.tsx`, `expandable.tsx`, `input-group.tsx`, `input.tsx`, `textarea.tsx`, `segmented-control.tsx`, `morphing-dialog.tsx`: round 3 lint edits (explicit transitions, no dark-variant backgrounds without text, disabled ink). `morphing-dialog.tsx` was restored after a bad regex and is unchanged in round 3.
