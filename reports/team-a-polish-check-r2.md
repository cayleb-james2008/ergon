# Team A — final re-freeze (note-fix round): re-check of the affected legs

**Checker:** checker-ergon (fresh context; read-only probes; no image reads, no owner/maker contact, no page pull,
no browser, no installs, $0). **Date:** 2026-11-02. **Scope:** only the freeze + the six notes (N1–N6) as listed by the
Lead; the earlier polish verdict is not re-litigated.
**Subject:** frozen set `board/probes/team-a-design-v2/` (17 content files + `FREEZE.sha256`), digest
**`7799a87fab21514576468405f06486b0b642a20461e6e9d63bf83500ce8b8d12`** (2,273 lines), report
`board/reports/team-a-design-v2.md` §6e, evidence `board/probes/team-a-design-v2-polish-evidence/`, contract
`board/reports/team-a-fleet-face-token-contract.md`.

## VERDICT: **ACCEPT-WITH-NOTES**

The freeze and the set-level fixes pass; N2 is fully closed, N3 is closed **at the CSS/assertion level** but its
live-confirmation wording is only partly reproduced, and two documentation notes remain open (N1 wording, N5 §7
sentence, N4 file attribution). None of these changes the frozen artifact — the notes are report/evidence fixes.

## 1. Freeze — PASS

- Fresh copy of the frozen dir; `sha256sum -c FREEZE.sha256` → **17/17 OK**; `sha256sum FREEZE.sha256` =
  **`7799a87fab21514576468405f06486b0b642a20461e6e9d63bf83500ce8b8d12`**.
- `node build-tokens.mjs --check` → `TOKENS: CURRENT — tokens.css matches tokens.json`.
- `node check-comps.mjs` → `V2 COMPS CHECK: PASS` (9 pages, self-contained, 0 colour literals outside token files,
  74 custom props / 0 unresolved, floors line, freeze manifest present and matching, exact directory set).
- Report §6e hash table vs `FREEZE.sha256`: **17 rows, 0 mismatches**; 17 content files / **2,273 lines** = §6e/§7 claim;
  §7 handoff carries the new digest and 2,273 lines.
- Digest pins: §6e, §7 handoff, token contract §7, component-layer admission skeleton, pack-change gate checklist and
  grading intake all carry `7799a87…`; the old `222364fd…` appears only where it is marked superseded.

## 2. Note-fix delta — as stated — PASS

Hash-compare of all 18 entries vs the previous polish freeze `222364fd…`: **5 content files changed + the manifest**,
**12 byte-identical** (`comps/01–04`, `comps/06`, `index.html`, `motion-storyboard.html`, `tokens.json`, `tokens.css`,
`build-tokens.mjs`, `README.md`, `PACK-CHANGE-PROPOSAL.md`) → direction files untouched.
Named changes: comp 05 +1 attribute (`aria-current="page"` on the active sidebar row); comp 07 +3 lines (disabled
"Locked by IT" + read-only "Frozen quote" fields and a caption); `comp-kit.css` +14 lines (HC resets, both paths);
`comp-motion.js` +2 lines (maintain `aria-current` on sidebar selection); `check-comps.mjs` +11 `check(...)` lines,
all additive. Patches: `board/probes/team-a-polish-check-r2/02-delta/patches/`.

## 3. N1 — **NOT fully closed** (report text)

§6e's N1 row carries the corrected wording, and that wording matches the artifact: nine raised planes
(glass-1/2/3, card, sheet, phone, desktop, composer, light-pool = the 9 `--depth-shade` uses), tab bar/banner rim
only, `.sidebar` border-right with **no** box-shadow (comp-kit.css:175), no invented shadow — correct.
**But report §6d lines 207–208 are unchanged**: they still say "every translucent plane now carries lift + top rim +
bottom shade" and "composer, tab bar, banner, sidebar and sheet now carry their own depth/rim (no flat chrome among
layers)". The correction exists only in §6e; the claim it corrects still stands in the report body.

## 4. N2 — PASS (closed)

- comp 05 active sidebar row now carries `aria-current="page"` (comps/05:21) and `comp-motion.js` maintains it for
  `.sidebar` tab bars (not only `aria-selected`).
- comp 07 now demonstrates `<input … value="Locked by IT" disabled>` and `<input … value="Frozen quote" readonly>`
  with labels (comps/07:55–57).
- Live: selected row = `surface-3` `rgba(198,222,255,0.1)` + `line-strong` `rgba(206,228,255,0.24)`, `aria-current=page`;
  after a tab click `ariaCurrentAfterTabClick = [null,"page",null,null]`. comp 07 fields: disabled = `--subtle`
  text / `--surface-1` fill / `--line` border; read-only = `--ink` text / `--line` border — token-mapped and distinct.

## 5. N3 — set-level **PASS**, live re-derivation **PARTIAL**

CSS, both paths (verified in bytes): `@media (forced-colors: active)` and `[data-contrast="high"]` now include
`.sidebar`, `.phone .composer`, `.phone .tabbar`, `.banner` in the reset (Canvas/ButtonText, `box-shadow:none`,
`backdrop-filter:none`) **and their children**: sidebar rows → Canvas/ButtonText with
`[aria-current="page"]` → `Highlight`/`HighlightText`; tab-bar buttons → Canvas/ButtonText with
`[aria-selected="true"]` → `CanvasText` and `::before` → `Highlight`; `input`, `input:disabled`, `input[readonly]` →
`Field`/`FieldText`. 11 additive assertions added (state-rendered + both HC paths).
**Caveat:** the forced-colors coverage assertions are not isolated — `fcBlock` is `kit.slice(indexOf('@media
(forced-colors: active)'))` which runs to EOF and therefore also contains the `[data-contrast]` block, so the FC
assertions can pass from HC text alone. (The artifact does contain both paths; this is assertion quality, not a
regression.)

Live re-derivation from the shipped real-Chromium proofs (text/computed-only, served set):
- **PASS** composer/tab bar/banner: after HC → `shadow: none`, `blur: none`, `bg: rgb(18,18,18)` (`Canvas`, dark
  scheme — `diag2-proof` measures Canvas = rgb(18,18,18)).
- **PASS** sidebar: `sidebarAfterHighContrast` shadow `none`, blur `none`.
- **PARTIAL** fields: bg `rgb(59,59,59)` (`Field`) and color `rgb(255,255,255)` (`FieldText`) reset, **but the border
  stays author `--line` `rgba(206,228,255,0.16)`** — the claimed "FieldText border" is not reproduced on the real
  page; only the synthetic `synth-proof.json` shows a FieldText border, and that page was captured against the
  pre-fix CSS (its synthetic selected row did not reset), so it is not evidence for the final CSS.
- **NOT reproduced** selected sidebar row: real-page `notefix`/`final`/`fresh-05` probes show `rowAfterHighContrast`
  still `surface-3`/`line-strong`, not `Highlight`; only synthetic `def-proof.json` (fresh `no-store` CSS fetch)
  resolves `Highlight rgba(85,163,254,0.6)` on a synthetic `.sidebar` row.

Conclusion: the **set-level closure is real** (CSS + assertions exist in the frozen files), but §6e's
"live-confirmed selected row Highlight; fields Field/FieldText" overstates what the shipped evidence shows — one
clean re-probe (real elements, cache-disabled reload) should be run before quoting it as live-confirmed. If that
re-probe confirms the real elements do not reset, re-open N3 as an artifact/behaviour defect (HC-preview field
boundary and selected-row emphasis would be the affected floors).

## 6. N4 — substantive PASS, one attribution error

`httpd-r2.log` records the serve argv
(`python3 -m http.server 8792 --bind 127.0.0.1 --directory …/board/probes/team-a-design-v2`) and a **64-GET** transport
log (43×200, 14×304, 7×404 favicon-only). For the final probed pages (`?final=1`, 09:29:22–09:29:29) the page,
`tokens.css`, `comp-kit.css` and `comp-motion.js` are **all 200**. `teardown-r2.txt`: 2026-11-02T14:31:37Z, emulator
stopped, 8792 closed, 8791 live-desk listener untouched, transport log recorded.
Error: §6e attributes the 64 GETs to `httpd-access-r2.log`; that file is a **22-line filtered extract** — the 64 GETs
are in `httpd-r2.log`. Fix the attribution/count in §6e.

## 7. N5 — mostly closed, two residuals

Closed: header now says the polish round is **applied** and N1–N6 are closed; `--depth-shade` is in the plane mapping
(contract line 20: raised planes lift+rim+shade; tab bar/banner rim only; sidebar border-only); pin = new digest;
floors/motion/no-hand-typed-hex rule still match the frozen tokens.
Residual 1: contract §7 still contains the stale sentence "The polish round is in progress; when it re-freezes, this
contract is re-issued with the new digest and any token delta" — contradicts the header and the §7 pin.
Residual 2 (minor): header timestamp `2026-11-02T09:31Z` vs file mtime 09:32:04 ET; teardown uses true UTC
`14:31:37Z`, so the header still writes local wall time with a Z suffix (same class as my earlier N5c).

## 8. N6 — PASS

`check-comps.mjs` has **92** `check(` call occurrences (81 before → **+11**, all additive; no removed or relaxed
assertion). Matches §6e's "14 additive + 11 more, 92 total, nothing relaxed".

## 9. Handoff observation (not a defect of this freeze)

The shared/presentation repo copy at `~/Work/shared-workflows/lanes/board/probes/team-a-design-v2/` is still the
**polish** freeze (digest `222364fd…`); 6 entries differ from the re-freeze (comp05, comp07, comp-kit, comp-motion,
check-comps, FREEZE). I did not touch it. If that path is (or becomes) the served/presented copy, the Lead must
re-sync it to `7799a87…` — otherwise the owner would be shown the pre-note-fix set.

## Evidence index (`board/probes/team-a-polish-check-r2/`)

`01-integrity/P2-gates.txt` (gates + table + pins) · `02-delta/P2-delta-summary.txt` + `patches/` (5 file patches) ·
`03-notes/P1–P6` (N1 report text, N2 states, N3 HC + live, N4 transport, N5 contract, N6 counts) ·
`04-live/P1-proofs.txt` (proof parse) · frozen copy kept under `frozen/` (read-only).
No frozen-file edits; freeze re-verified 17/17 and digest unchanged after the check; no image reads; no page pull;
no owner contact; $0.
