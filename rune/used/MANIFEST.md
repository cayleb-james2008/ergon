# rune/used: the library parts this app uses

Copied from the owner's house-frontend starter by `frontend/scripts/sync-used.mjs` (local Hallmark-clean edits recorded in `rune/used/LOCAL-EDITS.md`). Only parts the screens import are here; `npm run sync-used` refreshes this folder and this table.

| Part | Library | Licence | Imported by |
| --- | --- | --- | --- |
| `components/Tracker/Tracker.tsx` | Tremor Raw (tremor.so) | Apache-2.0 | src/views/Paper.tsx |
| `components/kibo-ui/pill/index.tsx` | Kibo UI (kibo-ui.com) | MIT | src/App.tsx, src/chat/Chat.tsx, src/chat/Entry.tsx, src/views/Paper.tsx |
| `components/ui/animated-background.tsx` | Motion Primitives (motion-primitives.com) | MIT | src/App.tsx |
| `components/ui/avatar.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/kibo-ui/pill/index.tsx |
| `components/ui/badge.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/kibo-ui/pill/index.tsx, src/chat/Chat.tsx, src/shell/Panels.tsx, src/views/Graph.tsx |
| `components/ui/border-trail.tsx` | Motion Primitives (motion-primitives.com) | MIT | src/chat/Chat.tsx |
| `components/ui/button.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/kibo-ui/pill/index.tsx, src/App.tsx, src/chat/Chat.tsx, src/chat/Entry.tsx, src/shell/Panels.tsx, src/shell/Update.tsx, src/views/Graph.tsx, src/views/Paper.tsx |
| `components/ui/command.tsx` | shadcn/ui on Radix UI primitives | MIT | src/shell/PersonSelect.tsx |
| `components/ui/dialog.tsx` | shadcn/ui on Radix UI primitives | MIT | rune/used/components/ui/command.tsx |
| `components/ui/expandable.tsx` | Cult UI (cult-ui.com) | MIT | src/chat/Entry.tsx |
| `components/ui/input-group.tsx` | COSS UI (coss.com/ui) | MIT | src/App.tsx, src/chat/Chat.tsx |
| `components/ui/input.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/ui/input-group.tsx, src/shell/Panels.tsx |
| `components/ui/kbd.tsx` | COSS UI (coss.com/ui) | MIT | src/App.tsx, src/chat/Chat.tsx |
| `components/ui/morphing-dialog.tsx` | Motion Primitives (motion-primitives.com) | MIT | src/shell/Panels.tsx |
| `components/ui/popover.tsx` | shadcn/ui on Radix UI primitives | MIT | src/shell/PersonSelect.tsx |
| `components/ui/segmented-control.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/ui/tabs.tsx, src/App.tsx, src/views/Graph.tsx |
| `components/ui/sliding-number.tsx` | Motion Primitives (motion-primitives.com) | MIT | src/shell/Panels.tsx, src/views/Graph.tsx, src/views/Paper.tsx |
| `components/ui/spinner.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/ui/button.tsx |
| `components/ui/tabs.tsx` | COSS UI (coss.com/ui) | MIT | src/App.tsx |
| `components/ui/text-shimmer.tsx` | Motion Primitives (motion-primitives.com) | MIT | src/chat/Chat.tsx |
| `components/ui/textarea.tsx` | COSS UI (coss.com/ui) | MIT | rune/used/components/ui/input-group.tsx |
| `hooks/useClickOutside.tsx` | House presets (house-frontend starter) | owner's | rune/used/components/ui/morphing-dialog.tsx |
| `lib/utils.ts` | House presets (house-frontend starter) | owner's | rune/used/components/kibo-ui/pill/index.tsx, rune/used/components/ui/animated-background.tsx, rune/used/components/ui/avatar.tsx, rune/used/components/ui/badge.tsx, rune/used/components/ui/border-trail.tsx, rune/used/components/ui/button.tsx, rune/used/components/ui/command.tsx, rune/used/components/ui/dialog.tsx, rune/used/components/ui/expandable.tsx, rune/used/components/ui/input-group.tsx, rune/used/components/ui/input.tsx, rune/used/components/ui/kbd.tsx, rune/used/components/ui/morphing-dialog.tsx, rune/used/components/ui/popover.tsx, rune/used/components/ui/segmented-control.tsx, rune/used/components/ui/spinner.tsx, rune/used/components/ui/tabs.tsx, rune/used/components/ui/text-shimmer.tsx, rune/used/components/ui/textarea.tsx, src/App.tsx, src/chat/Chat.tsx, src/chat/Entry.tsx, src/shell/PersonSelect.tsx, src/views/Graph.tsx, src/views/Paper.tsx |
| `utils/cx.ts` | Tremor Raw (tremor.so) | Apache-2.0 | rune/used/components/Tracker/Tracker.tsx |
