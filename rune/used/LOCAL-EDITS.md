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
