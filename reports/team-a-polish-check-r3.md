# Team A — final Lumen re-freeze (r3): last verify

**Checker:** checker-ergon (fresh context; read-only probes; no image reads, no owner/maker contact, no page pull,
no browser, no installs, $0). **Date:** 2026-11-02. **Scope:** the last re-freeze only — freeze, delta since the note-fix
freeze, and the six note closures; earlier rounds are not re-litigated.
**Subject:** `board/probes/team-a-design-v2/` (17 content files + `FREEZE.sha256`), digest
**`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`** (2,290 lines), report
`board/reports/team-a-design-v2.md` §6d/§6e/§7, evidence `board/probes/team-a-design-v2-polish-evidence/`
(`real-05/07/01-settled.json`).

## VERDICT: **ACCEPT-WITH-NOTES**

The frozen set, the delta discipline, N1–N6 and the live re-probe all pass. Two **documentation-only** notes remain (no
re-freeze needed, nothing in `board/probes/team-a-design-v2/`): the §6e N6 cell still quotes 92 checks (final set 94), and
three hand-off docs carry a carried-over "re-verified 06:39:45Z" timestamp that predates the digests they pin.

## 1. Freeze — PASS

- Fresh copy: `sha256sum -c FREEZE.sha256` → **17/17 OK**; `sha256sum FREEZE.sha256` =
  **`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`**.
- `node build-tokens.mjs --check` → `TOKENS: CURRENT — tokens.css matches tokens.json`; `node check-comps.mjs` →
  `V2 COMPS CHECK: PASS` (9 pages, 0 colour literals outside token files, 74 props / 0 unresolved, floors line, freeze
  manifest matching, exact directory set).
- 17 content files / **2,290 lines** = §6e/§7 claim; report §6e "Current freeze (recheck round)" table vs `FREEZE.sha256`:
  **17 rows, 0 mismatches**.
- Digest pins consistent: report §6e + §7 handoff, token contract §7, component-layer admission skeleton, pack-change gate
  checklist and grading intake all carry `2a8cc21b…` (the only other digests present are marked superseded).

## 2. Delta since `7799a87f` — PASS

Hash-compare of all 18 entries: **only `check-comps.mjs` changed (+ the manifest, expected); 16 files byte-identical** —
`comp-kit.css`, `comp-motion.js`, all seven comps, `index.html`, storyboard, tokens and the rest are untouched.
`check-comps.mjs` diff: adds a **brace-balanced `balancedBlock()` extractor**; `fcBlock` is now the balanced
`@media (forced-colors: active)` block (was `slice(…to EOF)`), `hcBlock` is sliced to before
`[data-contrast="high"] body`; `hc2`/`fc2` reuse those; **2 isolation assertions** added
(`!fc2.includes('data-contrast')`, `!hc2.includes('@media (forced-colors')`).
**No assertion was relaxed:** 0 removed `check()` lines; all hunks are additions/replacements with equal or narrower
scope; **94 `check(` occurrences** (92 + 2), `check-comps` PASS. Kit tail confirms the only rule after the HC `body` rule is
that body rule, so the truncation drops no asserted rule.

## 3. N1 — PASS (corrected in the report body)

§6d lines 207–208 now name the **nine raised planes** (glass-1/2/3, card, sheet, phone, desktop, composer, light-pool) as
carrying lift+rim+shade, state the **tab bar and banner carry the rim only**, and the **sidebar is a bordered full-height
plane with no shadow by design** ("corrected per checker N1"). Matches the artifact: 9 `--depth-shade` uses; `.sidebar`
has no box-shadow (comp-kit.css:175); tab bar/banner `box-shadow: var(--depth-rim)` only.

## 4. N4 / N5 — PASS

- N4: §6e now attributes the transport log correctly — `httpd-r2.log` has **64 GETs (43×200, 14×304, 7×404 favicon-only)**
  plus the recorded serve argv (`--directory board/probes/team-a-design-v2`), and `httpd-access-r2.log` is the **22-line
  filtered extract** (16×200). Both numbers verified from the files.
- N5: contract header says the polish round is **applied** and N1–N6 are **closed**, timestamp **`2026-11-02T14:55Z`**
  (true UTC; file mtime 09:53:54 ET = 14:53:54Z); the stale "polish round is in progress …" sentence in §7 is replaced by
  "This contract is final for this cycle"; `--depth-shade` is in the plane mapping (line 20: raised planes lift+rim+shade,
  tab bar/banner rim only, sidebar border only); pin = new digest.

## 5. N3 — PASS (set-level, evidence, assertion scope)

- Artifact: both HC paths (forced-colors media and `[data-contrast="high"]`) reset `.sidebar`, `.phone .composer`,
  `.phone .tabbar`, `.banner` **and children** — sidebar rows (selected → `Highlight`), tab-bar buttons
  (`CanvasText`/`Highlight` indicator), `input` incl. `:disabled`/`[readonly]` → `Field`/`FieldText` (kit unchanged since r2).
- The earlier contradiction is explained and documented: the kit transitions background/border at **120 ms**
  (kit:57/87/107), and the immediate-read probes sampled the transition's start value on a paused background tab.
- Real-element re-probe `probe4.py` (cache-disabled reload, `Page.bringToFront`, 0.6 s settle; data-contrast path **and**
  CDP `Emulation.setEmulatedMedia` forced-colors path):
  - `real-05-settled.json`: selected sidebar row normal `surface-3`/`line-strong` → **both paths** `Highlight`
    `rgba(85,163,254,0.6)` bg **and** border, `HighlightText` rgb(59,59,59).
  - `real-07-settled.json`: disabled "Locked by IT" / read-only "Frozen quote" normal `surface-1`/`line` → **both paths**
    `Field` rgb(59,59,59) bg, `FieldText` white colour **and border**.
  - `real-01-settled.json`: composer/tab bar/banner/phone normal shadows+blur → **both paths** shadow `none`, blur `none`,
    bg `Canvas` rgb(18,18,18).
- Assertion scope fixed as above; `check-comps` PASS with the balanced extractor (the two isolation assertions are the only
  added checks).

## 6. N6 / final count — PASS in artifact, one stale number in the report

94 `check(` occurrences in the final `check-comps.mjs` (92 + 2 scope assertions, all additive, 0 removed). The §6e recheck
paragraph documents the "+2 scope assertions"; however the **§6e N6 proof cell still says "92 `check(...)` calls total"** —
for the final freeze it should read **94**. Doc-only; no re-freeze needed.

## 7. Presented copy vs final set — CONFIRMED

Shared/presentation repo copy (`~/Work/shared-workflows/lanes/board/probes/team-a-design-v2/`) still digests the **polish
freeze `222364fd…`** — untouched by this check, exactly as the Lead's "Presented-copy ordering" decision in §6e describes.
The final r3 set differs from that presented copy in **6 entries** only (`comps/05`, `comps/07`, `comp-kit.css`,
`comp-motion.js`, `check-comps.mjs`, `FREEZE.sha256` = the N2 state demonstrations + N3 HC resets + assertion scope); the
other 12 files (tokens, index, storyboard, comps 01–04/06) are byte-identical → **direction identical**.

## 8. Remaining notes (documentation only)

1. **§6e N6 proof cell** says "92 `check(...)` calls total"; the final set is **94** (92 + the 2 documented scope
   assertions). Restate 94 in the final-freeze section.
2. **Hand-off timestamps:** admission skeleton, pack-change gate checklist and grading intake all say
   "re-verified 2026-11-02T06:39:45Z" while pinning `2a8cc21b…` (files written 09:53 ET = 14:53Z; the digest did not exist
   at 06:39Z). Carried over from an earlier round — update to the true UTC of this verification (the contract already does:
   `14:55Z`).

Neither note touches `board/probes/team-a-design-v2/`; the frozen set remains valid as frozen.

## Evidence index (`board/probes/team-a-polish-check-r3/`)

`01-integrity/P1-final.txt` (gates, table, pins) · `02-delta/P1-delta.txt`, `P2-vs-polish.txt`, `P3-nothing-else.txt`,
`diff/check-comps.mjs.patch` · `03-notes/P1-N1.txt`, `P2-N3.txt`, `P3-N4-N5.txt`, `P4-N6-and-doc-timestamps.txt` ·
`04-live/P1-evidence.txt`, `P2-presented-copy.txt`. No frozen-file edits; freeze re-verified 17/17 and digest unchanged
after the check; no image reads; no page pull; no owner contact; $0.
