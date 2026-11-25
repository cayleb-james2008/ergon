# Role Coverage — Frontend Components in Ergon v0.10.0

Ergon v0.10.0 unifies desktop and mobile into one responsive Claude-Desktop-style Personal Employee Workbench (`assets/index.html`) and keeps only the libraries that earn their byte budget with zero forced bloat:

| Library | Need of this product | Part used and its file | If pruned in v0.10.0: why |
| --- | --- | --- | --- |
| COSS UI | Crisp hairline desk controls: top-bar view tabs (`Chat`, `Today`, `Work graph`), inspector drawer tabs (`Files`, `History`, `Audit`), `Cmd/Ctrl+K` Command Palette (`cmdk`), `/` conversation & file search inputs, task composer, buttons, Light/Dark segmented switch (Settings-only), badges | `Command` (`rune/used/components/ui/command.tsx`), `Dialog` (`rune/used/components/ui/dialog.tsx`), `Tabs` (`rune/used/components/ui/tabs.tsx`), `InputGroup` + `Input` + `Textarea` + `Kbd` (`rune/used/components/ui/input-group.tsx`, `input.tsx`, `textarea.tsx`, `kbd.tsx`), `Button` (`rune/used/components/ui/button.tsx`), `SegmentedControl` (`rune/used/components/ui/segmented-control.tsx`), `Badge` (`rune/used/components/ui/badge.tsx`) | — |
| Kibo UI | Semantic status pills for agent readiness, waiting decisions, finished receipts, scheduler active/paused badges, and starter deliverable tags | `Pill` (`rune/used/components/kibo-ui/pill/index.tsx`) | — |
| Vaul | Gesture-friendly left navigation drawer under 768px carrying New chat, `/` filter, conversation list, scheduled routines quick drawer, and Settings/Workflows triggers | `Drawer` (`vaul`) | — |
| Sonner | Stacked non-blocking notices when inline approvals settle, multi-file uploads land, file versions restore, Word/Excel deliverables or SHA-256 Audit Bundles export, or routines fire | `Toaster` and `toast` (`sonner`) | — |
| Motion Primitives / Motion | — | — | Deliberately pruned in v0.10.0 (`MorphingDialog`, `SlidingNumber`, `AnimatedBackground`, `BorderTrail`, `TextShimmer`, `motion/react`) to eliminate ~190 kB of JS bundle overhead and 10-digit hidden DOM inflation inside headings, replaced by `PanelModal` on Radix `Dialog` + CSS transitions honoring `prefers-reduced-motion`. |
| Cult UI | — | — | Deliberately pruned in v0.10.0 (`Expandable`) in favor of zero-dependency accessible native disclosures in `Details`. |
| Tremor | — | — | Deliberately pruned in v0.10.0 (`Tracker` 14-slot decorative bar) to remove dashboard card-soup clutter from `Today`. |
