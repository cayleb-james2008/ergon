# Team A — design-pack token contract for the FLEET FACE (sidebar/panels)

**FOR TEAM B (plugin face build). STATUS: CONTRACT — tokens/floors only; no built surface, no code shipped, no owner-facing
action.** This is the exact token contract for the fleet plugin's sidebar face (fleet-state panels, phone-VM + Windows-VM
panels, contained Studio instances, agent-activity view). Build **against these tokens only** — no hand-typed hex. Written
by `ergon-lead` (Team A) 2026-11-02T09:05Z under the owner-approved Lumen direction; the **polish round is in progress**, so
the values below are the approved token set and the digest pinned at the end is updated when the polish re-freezes.

**Source of truth:** `board/probes/team-a-design-v2/tokens.json` → generated `tokens.css`; design rules
`.../PACK-CHANGE-PROPOSAL.md`; curation gate `board/reports/team-a-parts-bin-curation-gate.md`. This contract references the
token **names**; the builder imports the generated `tokens.css` (or an identical generated artifact) and uses `var(--name)`.

## 1. Sidebar surfaces (planes)

| Role | Tokens to use | Rule |
|---|---|---|
| Sidebar background | `--surface-1` over `--base`, `backdrop-filter: blur(var(--blur-panel)) saturate(112%)`, `border-right: 1px solid var(--line)` | The sidebar is **plane 1**; do not paint it a solid new colour |
| Panel / card | `--surface-2`, `border: 1px solid var(--line)`, `border-radius: var(--radius-card)`, `box-shadow: var(--depth-lift), var(--depth-rim)` | Plane 2; **depth is the stack**, not a new shadow |
| Selected / raised row / active tab | `--surface-3`, `border-color: var(--line-strong)`, `box-shadow: var(--depth-lift), var(--depth-rim-strong)` | Plane 3; one selected plane per region |
| Overlay / popover / sheet | `--surface-3` + `blur(var(--blur-sheet))`, `border-color: var(--line-strong)`, `--depth-lift-high`, `--depth-rim-strong`; scrim `--scrim` | Brightest plane is for transient surfaces only |
| Inset well (log/output area) | `--surface-1`, `--depth-well`, radius `--radius-inner`, border `--line` | Holds dense text; no extra border stacking |
| Light pool (decorative) | `--light-1` + `blur(var(--blur-deep))` | **Decorative only — never carries text** (floors forbid text on it) |
| Page/window base | `--base` (`--base-deep` for the deepest backdrop) | No ambient radial gradient inside the plugin face; the shell owns the page field |

**Lines:** dividers `--line` (1 px, non-essential); component/control boundaries `--line-bright` (measured ≥ 3:1 non-text);
field boundaries `--field-line` (measured 3.30:1 vs field fill). Do not use `--line` for an interactive control boundary.

## 2. Text and status roles

| Role | Token | Floor |
|---|---|---|
| Primary text | `--ink` | ≥ 7:1 on the brightest text plane (measured 9.21:1) |
| Secondary text / captions | `--muted` | ≥ 4.5:1 (measured 5.62:1) |
| Quiet text (footnotes, disabled labels) | `--subtle` | ≥ 4.5:1 (6.14:1 on surface-1); **base/panel contexts only** — never on `--light-1` or the brightest plane |
| Accent text (and the one forward action) | `--accent`; text on accent fill `--on-accent` | ≥ 4.5:1 text (5.01:1) |
| Status | `--ok`/`--warn`/`--danger` text on `--wash-ok`/`--wash-warn`/`--wash-danger`; edges `--edge-ok/--edge-warn/--edge-danger` | ≥ 4.5:1 (4.66 / 4.93 / 4.77:1) — **always with a word**, colour never the only signal |

## 3. Component mapping (build exactly this)

| Component | Spec |
|---|---|
| **List row** | default: transparent, `--ink`/`--muted` text; hover: `--surface-2`; selected: `--surface-3` + `--line-strong` border; focus-visible: 2 px `--accent` ring + `--focus-gap` offset; min-height `--target` (44) |
| **Card / panel** | `--surface-2`, radius `--radius-card`, border `--line`, `--depth-lift` + `--depth-rim`; title `--ink` semibold 600; caption `--muted`; one action max |
| **Buttons** | primary: `--accent` fill, `--on-accent` text, hover `--accent-hover`, press `--accent-press`; secondary: `--surface-2`, `--line-bright` border, `--ink`; ghost: transparent, `--muted` → `--ink` on hover; disabled: `--surface-1`, `--subtle`, no hover; min-height `--target`, radius `--radius-control`; focus ring as above; **one accent (primary) action per visible surface** |
| **Fields** | `--surface-1` fill, `border: 1px solid var(--field-line)`, radius `--radius-control`, min-height `--target`, text `--ink`, placeholder `--subtle`; focus: `--accent` ring + border `--accent`; error: `--danger` border + words, never colour alone |
| **Pills / status chips** | radius `--radius-pill`, wash background, status text + 7 px dot, **always words**, min-height 24 |
| **Empty state** | `--surface-1`, `border: 1px dashed var(--line-bright)`, radius `--radius-card`, one line of true state + one next step; no illustration dependency |
| **Tooltip** | `--surface-3` + `--line`, no arrow glow/shadow; 120 ms (`--motion-tap`); must never be the only label |
| **Tab / segmented control** | track `--surface-2`, selected `--surface-3` + `--line-strong`; indicator 2 px `--accent`; swap `--motion-swap` |
| **Divider** | 1 px `--line` |
| **Character / agent-activity mark** | the neural-net classes from `comp-kit.css`: `.nn` with `--nn-hue` = a single hue token, data-density 6/10/14, states idle/working/needs-you (core colour + word); one-shot signal only, no loops |
| **VM panel frames** | plane `--surface-1` + border `--line-strong`, radius `--radius-panel`, inside content plane `--surface-2`; panel title row uses `--muted` mono label + `--ink` name; no bespoke chrome colours |

## 4. Motion (pack tokens, no ad-hoc durations)

`--motion-tap` 120 ms (press/selection) · `--motion-swap` 200 ms (tab/view replacement) · `--motion-enter` 250 ms (content
arriving) · `--motion-sheet` 350 ms (sheet/popover in/out) · one curve `--ease`. **No loops, no pulse/ping, no
`transition-all`, no parallax, no autoplay.** Animate transform/opacity/colour only. Reduced motion: travel and stagger
removed, state changes instant.

## 5. Floors — locked (measured on the composited translucent stack, both ambient washes)

| Floor | Requirement | Measured |
|---|---|---|
| `--ink` on brightest text plane | ≥ 7:1 | 9.21:1 |
| `--muted` on brightest text plane | ≥ 4.5:1 | 5.62:1 |
| `--subtle` (base/panel only) | ≥ 4.5:1 | 6.14:1 |
| `--accent` text | ≥ 4.5:1 | 5.01:1 |
| status on wash | ≥ 4.5:1 | 4.66 / 4.93 / 4.77:1 |
| focus ring (2 px accent + 2 px gap, never animated) | ≥ 3:1 non-text | 6.47:1 |
| control boundary `--line-bright` | ≥ 3:1 non-text | 3.28:1 |
| field boundary `--field-line` | ≥ 3:1 non-text | 3.30:1 |
| targets | 44 min / 56 employee | enforced by `--target` / `--target-employee` |
| reduced motion | `prefers-reduced-motion: reduce` honoured (CSS + JS) | required |
| forced colours / High Contrast | translucency, blur and soft depth replaced by system colours + 1 px borders; no reliance on the theme | `@media (forced-colors: active)` + `[data-contrast="high"]` equivalents |
| unslop | no gradient text/buttons, glows, text-shadow, emoji-as-icons, sparkle badges, 800 weights, uppercase tracked labels, `transition-all`, pulse/ping loops | required |

If the host shell paints a background **lighter** than `--base`, re-run the contrast composite against that actual
background before calling the face conformant — the floors are measured against the real backdrop, not assumed.

## 6. "No hand-typed hex" rule (hard)

1. The face imports a **generated** token file (`tokens.css` or its generated equivalent). Values live only in
   `tokens.json` + generated output — never in component code.
2. Allowed: `inherit`, `transparent`, `currentColor`, `Canvas`/`CanvasText`/`ButtonFace`/`ButtonText`/`Highlight`/
   `HighlightText`/`Field`/`FieldText`/`LinkText` (system colours, High Contrast path), and `var(--token)`.
3. Forbidden: `#rrggbb`, `rgb()/rgba()`, `hsl()/hsla()`, named palette colours (`white`, `black`, `red`, …), and
   hard-coded `px` values for type/space/target/radius/motion that duplicate a token.
4. Self-check for the built face:
   `grep -rE '#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\(' <built-face-files>` → **0 hits** (excluding the generated token file
   itself); plus the token-resolution check: every `var(--…)` used must be defined by the generated file.
5. Team A reviews the **built face against this contract** (token conformance only; maker-never-checker — Team B builds,
   Team A does not self-certify its own design tokens, and any deviation goes back as a named defect list).

## 7. Pinning and change control

- **Pinned set:** `board/probes/team-a-design-v2/` — 17 content files, freeze digest
  **`222364fd85e34079f4f874c11111e253508442670633834863d350433b5323de`** (polish round, post-owner-grade; supersedes the `ea68e398…` pre-polish digest).
  **The polish round is in progress; when it re-freezes, this contract is re-issued with the new digest and any token
  delta — the builder must pin the digest it built against.**
- Any change to a token or to this contract goes through a re-freeze and, where the pack gate is touched, a fresh checker.
  No per-face token overrides (no "fleet-local" variants) without a proposal.
- Owner-facing: the fleet face is presented rendered by whoever owns that presentation; this contract authorises no
  presentation and no owner contact.

## 8. What this contract does NOT cover

Product behaviour, panel content/data contracts (Team B), the DSH shell's own chrome, Studio containment, or any
screenshot/visual approval — the face's visual grade is the owner's; this contract only fixes tokens, floors and the
no-hand-typed-hex discipline.
