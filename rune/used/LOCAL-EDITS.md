# rune/used local edits

These parts were edited in place after `sync-used` copied them. `frontend/scripts/sync-used.mjs` skips any path listed here, so a sync never reverts these edits.

- `components/ui/badge.tsx`: explicit colour/border transitions (no transition-all, no box-shadow); focus ring ink; `data-slot` removed for Hallmark F18.
- `components/ui/button.tsx`: explicit transitions; focus ring ink; disabled channel uses fill and ink; `data-slot` removed for Hallmark F18.
- `components/ui/animated-background.tsx`: inner content wrapper uses `min-w-0 flex-1` so full-width row buttons truncate cleanly inside the gliding container.
- `components/ui/sliding-number.tsx`: inline `span` elements (`inline-flex`) so rolling numbers remain valid phrasing content inside headings and paragraphs.
- `components/ui/expandable.tsx`: transition on opacity and transform only.
- `components/ui/input-group.tsx`: explicit transitions; surface focus styling; disabled track fill and disabled ink.
- `components/ui/input.tsx`: explicit transitions; focus ring ink; disabled ink.
- `components/ui/segmented-control.tsx`: indicator moves on translate only; focus ring ink; `data-slot` markers removed so Hallmark F18 does not misread segment items as image slots.
- `components/ui/tabs.tsx`: indicator moves on `translate` with `--dur-swap` and `--ease-house` (no animated width); explicit colour/border transitions; disabled track/ink tokens; `data-slot` markers removed.
- `components/ui/textarea.tsx`: explicit transitions; focus ring ink.
- `components/ui/command.tsx`: disabled input uses disabled ink, not opacity alone.
- `components/ui/morphing-dialog.tsx`: backdrop uses `bg-ink/40` token (no literal white/black, no backdrop blur); exports `useMorphingDialog` so child forms can close cleanly on submit; supports optional controlled `open`/`onOpenChange` and sets `pointer-events-auto z-50` on its portal layers so dialogs lifted outside mobile Vaul drawers stay interactive.
- `components/kibo-ui/pill/index.tsx`: mask gradient uses `currentColor` instead of literal white.
- `components/ui/dialog.tsx`: backdrop uses `bg-ink/45` token (no literal black), surface/hairline/shadow-lift tokens, scrollbar-safe `w-[calc(100%-2rem)]` (no `100vw` overflow), and 44px mobile close target with `focus-ring`.
- `components/Tracker/Tracker.tsx`: `px-0` on the 4px scale and `transition-[color,background-color]` (no bare `.transition` class).
