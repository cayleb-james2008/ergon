# Pack-conformance gate checklist — one page per built surface

**What this is:** the one-page audit template a builder fills before a surface is called done, so the pack gate
(`proposal → all-states check → screenshot compare → fresh checker`, annex §5) is mechanical instead of remembered. It is
a **tool, not a gate**: the gate is the fresh checker's verdict on the evidence this page points to. Applies to every
surface built from the Ergon Midnight Slate pack (and the fleet pack once seeded) — including any component admitted from
the UI-library parts bin. Maker never checker.

## A. Surface and provenance (fill before the first screenshot)

| Field | Value |
|---|---|
| Surface / component | |
| Pack | Ergon Midnight Slate v1.0.0 (or fleet pack version) |
| Source of the component | Pack component / parts-bin library + annex §1 licence line (clean for this use: Y/N) |
| Tokens used | (names only — no hand-typed hex anywhere) |
| Restyle notes (what changed from the source) | |
| Deliberately left out (forbidden tells) | |
| Builder | |
| Fresh checker (different person, did not build) | |

## B. All-states check (each state rendered from the pack; missing state = fail)

| State | Rendered | Evidence path | Notes |
|---|---|---|---|
| default | ☐ | | |
| hover | ☐ | | |
| focus (2 px accent ring, 2 px gap, never animated) | ☐ | | |
| pressed | ☐ | | |
| disabled | ☐ | | |
| loading (static placeholder + worded label — no pulse/shimmer loop) | ☐ | | |
| error (plain words, nothing lost, one next step) | ☐ | | |
| empty (one next step, never a dead end) | ☐ | | |
| reduced motion | ☐ | | |
| Windows High Contrast | ☐ | | |

## C. Screenshot compare (not "looks close" — name the differences)

| Compare | Reference | Built | Differences named (tokens / spacing / type / motion) |
|---|---|---|---|
| Surface vs pack visual contract (`design/style-guide.html`) | | | |
| Motion vs `design/motion-spec.md` (120 / 200 / 250 / 350 ms, one ease-out curve) | | | |

## D. Unslop gate — pass/fail each sign (Nox `20261101T154200Z` §1; upstream `github.com/heydamir/unslop`, CC BY 4.0)

| Sign | Fails? |
|---|---|
| purple→blue gradients, gradient text | |
| glows, glowing pills | |
| emoji used as icons/labels | |
| sparkle/"New" badges | |
| 800-weight headings | |
| uppercase tracked labels | |
| off-grid spacing | |
| greeting headlines, hype copy | |
| rainbow charts | |
| `transition-all` | |
| pulse/ping loops | |

Any "fails" row → not done until fixed and re-checked.

## E. Fences (all must be Y)

- ☐ One accent forward action per view; Back / Close / Cancel / Save-secondary stay secondary.
- ☐ Targets: 44 DIP minimum everywhere, 56 DIP on employee screens.
- ☐ Plain words on employee screens (no tech words); no dead ends; errors say what happened and the next step.
- ☐ Flat Midnight Slate only: no gradients, glass, shine, reflections, coloured shadows; depth only `base → panel → raise`.
- ☐ Tokens only: colours/sizes from `design/tokens.json` via the generator; no hand-typed hex; generated outputs current
      (`node scripts/generate-theme.mjs --check`).
- ☐ Motion: budgets honoured, one curve, no loops/glow/parallax/decorative blur, reduced motion + High Contrast handled.
- ☐ Licence line clean for this use (annex §1); notice/attribution retained; no Glitch Cat Club content; system fonts only.
- ☐ No CDN scripts, no external assets in the artefact.
- ☐ No product/runtime claim from source alone: native rendering, DPI, accessibility and animation behaviour verified on a
      real machine, by someone who did not build it.

## F. Fresh checker verdict (the gate)

| Field | Value |
|---|---|
| Checker | |
| Evidence re-run (commands/paths) | |
| Verdict | PASS / FAIL / HOLD |
| Findings (one line each, with the failing state/sign) | |
| Date (UTC) | |

**Close-out:** admission record appended to the pack's component inventory (and the annex row if from the parts bin); pack
changelog updated only if the pack itself changed; the surface is not called done until F says PASS.
