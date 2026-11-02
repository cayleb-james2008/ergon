# Team A — Lumen v2 POLISH round: fresh check

**Checker:** checker-ergon (fresh context; read-only probes only; no image reads, no owner/maker contact, no
installs, no service/unit/Play/Studio actions, $0). **Date:** 2026-11-02.
**Subject:** frozen set `board/probes/team-a-design-v2/` (17 content files + `FREEZE.sha256`), sync of that set to
`~/Work/shared-workflows/lanes/board/probes/team-a-design-v2/`, report `board/reports/team-a-design-v2.md` §6d,
live evidence `board/probes/team-a-design-v2-polish-evidence/`, token contract
`board/reports/team-a-fleet-face-token-contract.md`.

## VERDICT: **ACCEPT-WITH-NOTES**

The frozen artifact passes every gate I can run independently: integrity, finish-only delta, direction
untouched, floors, accessibility floors, unslop and the live-page claim. Six notes (N1–N6) are exact defects in
the **report wording, dormant polish states, HC-preview finish, live evidence chain and token contract** — none
invalidates the direction grade; N1/N2/N3 should be corrected before the owner sees §6d presented as proof, and
a set-level fix of N2/N3 requires a new freeze + fresh check.

## 1. Freeze integrity — PASS

- Fresh copy of the frozen dir; `sha256sum -c FREEZE.sha256` → **17/17 OK**; `sha256sum FREEZE.sha256` =
  **`222364fd85e34079f4f874c11111e253508442670633834863d350433b5323de`** (matches §6d, §7 and contract pin).
- `node build-tokens.mjs --check` → `TOKENS: CURRENT — tokens.css matches tokens.json`.
- `node check-comps.mjs` → `V2 COMPS CHECK: PASS` (9 pages labelled, self-contained, 0 colour literals outside
  token files, 74 custom properties / 0 unresolved, floors, freeze manifest present and matching, exact
  directory set enforced).
- 17 content files / **2,235 lines**; report §6d hash table compared programmatically against `FREEZE.sha256`:
  **17/17 rows, 0 mismatches, 0 missing/extra**.
- Working board dir byte-identical to my post-freeze copy (no edits after freeze); working↔shared board
  **18/18 identical** (12 top-level entries incl. `comps/`, all 7 comps included).
- Pre-polish baseline for the delta: shared-board commit `3947f1c5^` (freeze `ea68e3980…`) extracted with
  `git archive`, 17/17 OK — correct parent.

## 2. Polish delta — finish-only, direction untouched — PASS

Hash-compared all 17 files vs `ea68e398…`: **6 changed, 11 unchanged** — all 7 comps, `motion-storyboard.html`,
`build-tokens.mjs`, `README.md`, `PACK-CHANGE-PROPOSAL.md` are byte-identical.

| File | Polish change (named) |
|---|---|
| `index.html` | +4 lines, descriptive only: "Direction approved by the owner (2026-11-02) … polish round" note and a "Polish round (post-grade)" honest-notes line. |
| `tokens.json` | +1 line: `depth.shade = inset 0 -1px 0 rgba(2,3,5,0.42)`; every existing colour/blur/depth/radius/space/type/target/motion value byte-identical. |
| `tokens.css` | +1 generated line `--depth-shade` (generator current). |
| `comp-kit.css` | 47 lines, finish-only: `--depth-shade` added to glass1/2/3, card, sheet, phone, desktop, composer, light-pool (9 uses); lift+rim on composer; rim on tab bar and banner; `a:focus-visible`; token `::selection`; thin token scrollbars; `input:disabled`/`input[readonly]`; `.display` line-height 1.12; pill `white-space: nowrap`; `.nn` cursor/hover/focus-visible ring + signal opacity transition; `.sidebar nav button:hover`; selected sidebar row `surface-2/line → surface-3/line-strong`; `.card[data-resolved="true"]` edge-ok; tab `::before` base opacity 0 + `transition: opacity var(--motion-swap)` + selected opacity 1; negative outline-offset for tab buttons at the phone frame edge. |
| `comp-motion.js` | +8 lines: character becomes a control — removes `aria-hidden`, `role=button`, `tabIndex=0`, `aria-label "<name> — agent character; activate to signal"`, Enter/Space → `signal()`. |
| `check-comps.mjs` | +18 lines, **all additive** (one required-token list gains `--depth-shade`; 14 new `check(...)` assertions). No existing floor/assertion relaxed, removed or weakened. |

No colour token value, layout structure, type scale or comp markup changed → translucent Lumen vein,
technical-editorial restraint and neural characters are untouched (direction locked by the owner's grade).

## 3. Floors — PASS (independently re-derived)

`check-comps.mjs` (both ambient washes composited): ink **9.21** · muted **5.62** · subtle(panel) **6.14** ·
accent **5.01** · status **4.66 / 4.93 / 4.77** · focus **6.47** · control boundary **3.28** · field boundary
**3.30** — identical pre- and post-polish and to the report claim.

My own unrounded recompute from `tokens.json` (01–02 rounding differences only): ink 9.24, muted 5.64, accent
5.02, field-line-over-input 3.30, line-bright-over-p2 3.30, **composer field-line over composer surface 3.46** —
the live claim 3.46 is exactly reproduced. `subtle` on the brightest plane measures 4.02, which is not a text
floor context (contract fixes subtle to base/panel = 6.14, asserted PASS).

Presence checks: `prefers-reduced-motion` (CSS universal + `[data-motion="reduced"]`, JS `matchMedia`), forced-
colors block, 14 `[data-contrast="high"]` rules (inputs/messages/links/character included), 2 px accent focus
ring, `--target 44` / `--target-employee 56`, one accent hue family, unslop clean (0 gradient text, 0
text-shadow, 0 `transition-all`, 0 infinite/pulse loop, 0 emoji, 0 uppercase tracked labels; face grep from the
contract = 0 hand-typed hex/rgb/hsl hits).

## 4. Polish claims verified in the shipped bytes + live proof — PASS, with dormant-state notes (N1/N2)

Depth-shade token and 9 kit uses; 200 ms tab cross-fade (`opacity 1` selected / `0` unselected / `0.2s`, not a
hard swap); focusable, labelled character (role=button, tabIndex 0, aria-label, Enter/Space, accent focus ring;
uiautomator exposes "Marlow — agent character; activate to signal"); composer depth; tab-bar and banner rim;
link focus ring; token selection + thin scrollbars; tabular numerals (live comp 05 `tabular-nums`); display
leading; nowrap pills; resolved-approval state (`.card[data-resolved]` + ok edge — live-wired at
`comp-motion.js:148` on approve); frame-edge focus ring; character signal transition; motion settles 0 running /
0 infinite. These are recorded in `board/probes/team-a-polish-check/02-delta/P3–P7`.

## 5. Live-page proof — SUPPORTED, one evidence-chain gap (N4)

The three proof JSONs are real CDP `Runtime.evaluate` responses from a page at
`http://10.0.2.2:8792/comps/<page>.html?polish=1|2` (target title/url recorded). Every live value resolves
exactly to the frozen files: `--depth-shade` `inset 0 -1px 0 rgba(2,3,5,0.42)` (tokens.css:46); card shadow =
depth-lift `0 10px 30px rgba(2,3,5,.5)` + rim `.07` + shade `.42`; phone `surface-1` + `blur(14px)`;
composer border `rgba(206,228,255,0.46)` = `--field-line`; **8/8 checked surfaces inset rim**; composer
boundary **3.46**; tab `aria-selected` swap; approval "Approval needed" → "Approved — one action, recorded";
label present; **0 running / 0 infinite**. `u.xml` shows the rendered page text; `emulator.log` shows Android
emulator 37.2.13 + Chromium `VkInstance` create/destroy; `teardown.txt` shows emulator stopped, http 8792
closed, port 8791 (live desk) listener untouched, adb empty. Evidence mtimes 09:04:40–09:06:57 ET are all
**after** the freeze writes (09:03:17–09:04:19), and the URLs are cache-busted (no page code reads
`location.search`, so the query is a pure bust).

Gap (N4): `httpd.log` contains only a failed duplicate `python3 -m http.server` start
(`OSError: [Errno 98] Address already in use`) and **no GET/access lines** — the served directory and the
read-only invocation are inferred (stdlib `http.server` is GET/HEAD-only by design), not logged. The claim
survives on the CDP/UI/emulator evidence, but transport provenance is not byte-proof.

## 6. Token contract — PASS with drift notes (N5)

`board/reports/team-a-fleet-face-token-contract.md` pins the correct freeze digest, 17 content files, floors
table matching measured values exactly, motion values 120/200/250/350 ms matching tokens.css, and the
"no hand-typed hex" rule with its grep + token-resolution check; applied to the frozen face files the grep
returns 0 hits, and every contract token name resolves ( `--nn-hue` is a kit alias to `--accent`, acceptable).
Drift: N5a header still says "polish round is in progress … digest is updated when the polish re-freezes" while
§7 already pins the post-polish digest; N5b the new `--depth-shade` token is **not** in the contract's plane
mapping (0 mentions), so "build exactly this" would omit the polish bottom shade; N5c header timestamp
`2026-11-02T09:05Z` is inconsistent with the file mtime (09:07 ET = 14:07Z) and with the digest it pins.

## 7. Named notes (exact)

- **N1 (report overstatement — doc fix).** §6d: "composer, tab bar, banner, sidebar and sheet now carry their
  own depth/rim … no flat chrome among layers" and "every translucent plane now carries lift + top rim +
  bottom shade". The `.sidebar` rule has **no box-shadow at all** (comp-kit.css:175) — their own probe says
  `insetSurfaces.sidebars: 0` — and tab bar/banner carry rim only (no lift/shade). Depth-shade uses: 9, on the
  surfaces that have it.
- **N2 (two dormant polish states).** The selected-sidebar-row rule (`surface-3`/`line-strong`) cannot render:
  no frozen comp has `aria-current` on a `.sidebar` button (only tab bars in comps 03/04 do), and the desktop
  probe returns `selectedRowBg/Border: null`. `input:disabled` / `input[readonly]` were added, but no frozen
  comp contains a disabled/readonly input (comps are unchanged), so the states are not demonstrated anywhere.
  §6d lists them as proved states; they are CSS-declared only — either wire a demo state into a comp at the
  next freeze or qualify §6d. (The resolved-approval rule is *not* dormant — `comp-motion.js:148` sets
  `data-resolved="true"` on approve and the live pill change was probed; only its evidence capture is thin.)
- **N3 (HC-preview finish gap).** Polish added soft depth to `.phone .composer`, `.phone .tabbar`, `.banner`,
  but those selectors are absent from both the `@media (forced-colors: active)` and the
  `[data-contrast="high"]` `box-shadow: none; backdrop-filter: none` reset lists (comp-kit.css:229–248). The
  Floors-preview High-Contrast path therefore still shows author shadows on those three layers, against
  contract §5 and comps/07's own "soft depth replaced by system colours + 1 px borders" line. Text/contrast
  floors are unaffected; the background/blur part of this gap pre-exists the polish, the new depth widens it.
- **N4.** Live transport unlogged (see §5): keep the server access log and record `--directory` in future runs.
- **N5.** Contract drift (see §6): add `--depth-shade` to the plane mapping, fix "polish in progress" wording
  and the header timestamp before Team B builds against it.
- **N6 (trivial).** §6d says "13 polish assertions added"; the `check-comps.mjs` diff adds **14**
  `check(...)` lines (all additive — the direction of the discrepancy is conservative).

**No change to the frozen set is required for the direction grade.** If N1–N3 are fixed inside the set, that
is a new freeze (hash changes) and needs a fresh check.

## Evidence index (`board/probes/team-a-polish-check/`)

- `01-integrity/` P1–P11: freeze 17/17 + digest, tokens check, comps check (PASS output), counts, final gate
  rerun, working↔shared sync.
- `02-delta/` P1–P10 + `patches/`: file delta, unified patches for all 6 changed files, depth-shade, tab
  cross-fade, character control, finish details, unslop, direction invariants, hash compare, report-table vs
  FREEZE.
- `03-floors/` P1–P5: tokens audit, independent recompute (post- and pre-polish), floors present.
- `04-live/` P1–P8: evidence inventory, proof parsing, claim gaps (N1–N3), live-evidence assessment (N4).
- `05-token-contract/` P1: contract pass/drift (N5).
