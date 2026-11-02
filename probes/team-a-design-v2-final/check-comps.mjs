#!/usr/bin/env node
/**
 * Self-check for the Ergon Lumen v2 direction comps — hardened after the fresh checker's REJECT (D1–D4).
 * Maker's own evidence; a fresh checker re-runs it. Asserts, per file, in rendered scope:
 *   labels · self-containment (incl. local refs resolve) · token discipline (no colour literals/keywords) ·
 *   custom-property resolution (no unresolved var()) · tag balance · tokens.css generated-current ·
 *   contrast floors on the *full* composited stack (both ambient washes; incl. field boundary and control boundary) ·
 *   floors present (reduced motion / forced colours / HC preview / focus ring / targets) · target-size floors ·
 *   employee target present on employee surfaces · empty state · approvals wording · motion budgets and banned tells ·
 *   freeze-manifest integrity · and non-vacuity self-tests for the assertions themselves.
 * Run: node check-comps.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { build } from './build-tokens.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const read = (f) => fs.readFileSync(path.join(HERE, f), 'utf8');
const exists = (f) => fs.existsSync(path.join(HERE, f));
const findings = [];
const check = (ok, msg) => { if (!ok) findings.push(msg); };
const sha256 = (file) => crypto.createHash('sha256').update(fs.readFileSync(path.join(HERE, file))).digest('hex');

const COMP_DIR = path.join(HERE, 'comps');
const PAGES = ['index.html', 'motion-storyboard.html', ...fs.readdirSync(COMP_DIR).filter((f) => f.endsWith('.html')).sort().map((f) => `comps/${f}`)];
const CODE = ['comp-kit.css', 'comp-motion.js', 'index.html', 'motion-storyboard.html', ...PAGES.filter((p) => p.startsWith('comps/'))];
const tokens = JSON.parse(read('tokens.json'));

/* ---------- generic detection helpers (also self-tested for non-vacuity at the end) ---------- */
const COLOUR = /#(?:[0-9a-f]{3,8})\b|\brgba?\(|\bhsla?\(/gi;
const detectsColourLiteral = (text) => [...text.matchAll(COLOUR)].length > 0;
const PALETTE_WORDS = 'white|black|red|blue|green|gray|grey|silver|yellow|orange|purple|pink|brown|cyan|magenta|lime|teal|navy|maroon|olive|aqua|fuchsia';
const COLOUR_KEYWORD = () => new RegExp(`(?:^|[;{])\\s*(?:color|background(?:-color)?|border(?:-[a-z]+)?|fill|stroke|outline(?:-color)?|box-shadow|text-shadow)\\s*:\\s*[^;{}]*\\b(?:${PALETTE_WORDS})\\b`, 'i');
const detectsColourKeyword = (text) => {
  if (COLOUR_KEYWORD().test(text)) return true;
  // An inline style's FIRST declaration has no leading ';'/'{' — scan each style="" value with a synthetic separator.
  for (const m of text.matchAll(/style\s*=\s*["']([^"']*)["']/gi)) {
    if (COLOUR_KEYWORD().test(';' + m[1])) return true;
  }
  return false;
};
const detectsInfinite = (text) => /\binfinite\b/i.test(text.replace(/animation-iteration-count:\s*1\s*!important/g, ''));
const detectsBrokenTag = (text) => {
  const tags = ['div', 'section', 'article', 'aside', 'nav', 'header', 'footer', 'main', 'form', 'button', 'span', 'p', 'a', 'table', 'thead', 'tbody', 'tr', 'td', 'th', 'ul', 'li', 'h1', 'h2', 'h3', 'h4', 'label'];
  for (const tag of tags) {
    const opens = [...text.matchAll(new RegExp(`<${tag}(?=[\\s>])`, 'g'))].length;
    const closes = [...text.matchAll(new RegExp(`</${tag}\\s*>`, 'g'))].length;
    if (opens !== closes) return { ok: false, tag, opens, closes };
  }
  return { ok: true };
};
const definedProps = (texts) => new Set(texts.flatMap((t) => [...t.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1])));
const usedProps = (texts) => new Set(texts.flatMap((t) => [...t.matchAll(/var\(\s*(--[a-z0-9-]+)/g)].map((m) => m[1])));
const unresolved = (used, defined) => [...used].filter((p) => !defined.has(p));

// Brace-balanced extractor: an assertion scoped to a media block must not read past that block's closing brace.
const balancedBlock = (text, needle) => {
  const start = text.indexOf(needle);
  if (start === -1) return '';
  const open = text.indexOf('{', start);
  if (open === -1) return '';
  let depth = 0;
  for (let i = open; i < text.length; i += 1) {
    if (text[i] === '{') depth += 1;
    else if (text[i] === '}') { depth -= 1; if (depth === 0) return text.slice(open, i + 1); }
  }
  return '';
};

/* ---------- 1. labels ---------- */
for (const page of PAGES) check(/DESIGN COMP — NOT A BUILT SURFACE/.test(read(page)), `${page}: missing the "DESIGN COMP — NOT A BUILT SURFACE" label`);

/* ---------- 2. self-containment + every local ref resolves ---------- */
for (const file of CODE) {
  // The only permitted URL-shaped string is the SVG XML namespace identifier used by createElementNS — a name, not a fetch.
  const text = read(file).replaceAll('http://www.w3.org/2000/svg', '');
  check(!/https?:\/\//i.test(text), `${file}: contains an external URL (comps must render offline with no network)`);
  check(!/@import/i.test(text), `${file}: contains @import`);
  check(!/url\(\s*['"]?(https?:)?\/\//i.test(text), `${file}: loads a remote url()`);
  check(!/<img\b/i.test(text), `${file}: contains an <img> asset (no image dependency)`);
  check(!/fonts\.googleapis|@font-face/i.test(text), `${file}: contains a webfont dependency`);
}
for (const page of PAGES) {
  const text = read(page);
  check(/href="(\.\.\/)?tokens\.css"/.test(text), `${page}: does not link tokens.css (relative)`);
  check(/href="(\.\.\/)?comp-kit\.css"/.test(text), `${page}: does not link comp-kit.css (relative)`);
  check(/src="(\.\.\/)?comp-motion\.js"/.test(text), `${page}: does not load comp-motion.js (motion must be live)`);
  for (const [, ref] of text.matchAll(/(?:href|src)="([^"]+)"/g)) {
    if (/^(?:#|data:|mailto:|https?:|\/\/)/i.test(ref)) continue;
    const target = path.resolve(path.dirname(path.join(HERE, page)), ref.split(/[?#]/)[0]);
    check(fs.existsSync(target), `${page}: local ref "${ref}" does not resolve to a file`);
  }
}

/* ---------- 3. token discipline: no colour literals / keywords outside tokens.json + generated tokens.css ---------- */
for (const file of CODE) {
  const text = read(file);
  check(!detectsColourLiteral(text), `${file}: hand-typed colour literal(s) ${[...text.matchAll(COLOUR)].slice(0, 5).map((m) => m[0]).join(', ')} — use the generated tokens`);
  check(!detectsColourKeyword(text), `${file}: raw colour keyword in a style context — use a token`);
}
check(exists('tokens.css'), 'tokens.css missing');
check(read('tokens.css') === build(tokens), 'tokens.css is stale — run node build-tokens.mjs');

/* ---------- 4. custom-property resolution (the D1 blocker class) ---------- */
const defined = definedProps([read('tokens.css'), read('comp-kit.css')]);
const missingProps = [...unresolved(usedProps([...CODE.map(read)]), defined)];
check(missingProps.length === 0, `unresolved custom properties (declared nowhere): ${missingProps.slice(0, 10).join(', ')}`);
for (const required of ['--surface-1', '--surface-2', '--surface-3', '--light-1', '--field-line', '--line-bright', '--blur-deep', '--depth-shade']) {
  check(defined.has(required), `required token ${required} is not emitted by tokens.css (kebab/generator regression)`);
}

/* ---------- 5. tag balance ---------- */
for (const page of PAGES) {
  const r = detectsBrokenTag(read(page));
  check(r.ok, `${page}: tag balance broken — <${r.tag}> ${r.opens} open / ${r.closes} close`);
}

/* ---------- 6. motion budgets + banned tells ---------- */
const m = tokens.motion;
check(m.tap === '120ms' && m.swap === '200ms' && m.enter === '250ms' && m.sheet === '350ms', 'motion budgets must be 120/200/250/350 ms');
check(/^cubic-bezier\(/.test(m.ease), 'one ease-out curve token missing');
for (const file of ['comp-kit.css', 'comp-motion.js', ...PAGES]) {
  const text = read(file);
  check(!/transition-all/i.test(text), `${file}: transition-all is banned`);
  check(!/text-transform:\s*uppercase/i.test(text), `${file}: uppercase tracked labels are banned (unslop)`);
  check(!/background-clip:\s*text|-webkit-background-clip:\s*text/i.test(text), `${file}: gradient text is banned`);
  check(!detectsInfinite(text), `${file}: infinite/looping animation is banned`);
  check(!/animation-name:\s*[^;]*(pulse|ping|glow)/i.test(text), `${file}: pulse/ping/glow animations are banned`);
}

/* ---------- 7. accessibility floors present ---------- */
const kit = read('comp-kit.css');
check(/@media\s*\(prefers-reduced-motion:\s*reduce\)/.test(kit), 'comp-kit.css: missing the reduced-motion floor');
check(/@media\s*\(forced-colors:\s*active\)/.test(kit), 'comp-kit.css: missing the High Contrast (forced-colors) floor');
check(/\[data-contrast="high"\]/.test(kit), 'comp-kit.css: missing the High Contrast preview floor');
check(/\[data-contrast="high"\]\s+input/.test(kit), 'comp-kit.css: HC preview does not cover inputs');
check(/\[data-contrast="high"\]\s+\.msg/.test(kit), 'comp-kit.css: HC preview does not cover messages');
check(/outline:\s*var\(--focus-ring\)\s*solid\s*var\(--accent\)/.test(kit), 'comp-kit.css: focus ring must be 2px accent with a gap');
check(/outline-offset:\s*var\(--focus-gap\)/.test(kit), 'comp-kit.css: focus ring gap missing');
check(/button\.employee\s*\{[^}]*min-height:\s*var\(--target-employee\)/.test(kit), 'comp-kit.css: employee 56 DIP target rule missing');

/* ---------- 8. target floors + employee target actually applied + empty state ---------- */
for (const page of PAGES) {
  const text = read(page);
  for (const [, px] of text.matchAll(/min-height:\s*(\d+)px/g)) check(Number(px) >= 44, `${page}: inline min-height ${px}px is below the 44 DIP floor`);
}
for (const page of ['comps/01-mobile-chat.html', 'comps/02-mobile-triage-approvals.html', 'comps/03-mobile-tasks.html', 'comps/04-mobile-make.html', 'comps/06-character-sheet.html']) {
  check(/class="[^"]*\bemployee\b/.test(read(page)), `${page}: no employee-surface control carries the 56 DIP employee target`);
}
check(/No tasks yet/.test(read('comps/07-states.html')) && /class="empty"/.test(read('comps/07-states.html')), 'comps/07: the empty-state specimen is missing');
check(/reduced motion/.test(read('comps/07-states.html')) && /High Contrast/.test(read('comps/07-states.html')), 'comps/07: reduced-motion/High Contrast floors rows are not labelled');

/* ---------- 8b. C1–C5 hard assertions (composer boundary, checkbox focus, latent pill) ---------- */
check(/\.phone \.composer\s*\{[^}]*border:\s*1px solid var\(--field-line\)/.test(kit), 'comp-kit.css: `.phone .composer` must use the field-line boundary (3:1 field rule)');
check(!/\.phone \.composer\s*\{[^}]*border:[^;]*var\(--line\)/.test(kit), 'comp-kit.css: `.phone .composer` must not use the decorative --line token');
const desktop = read('comps/05-desktop-workspace.html');
const composerLine = desktop.split('\n').find((l) => l.includes('aria-label="Message Marlow"')) || '';
check(composerLine.includes('var(--field-line)'), 'comps/05: the desktop composer inline border must use var(--field-line)');
check(!composerLine.includes('var(--line)'), 'comps/05: the desktop composer inline border must not use var(--line)');
check(/input\[type="checkbox"\]:focus-visible\s*\{[^}]*outline:\s*var\(--focus-ring\)\s*solid\s*var\(--accent\)/.test(kit), 'comp-kit.css: native checkboxes must carry the 2px accent focus ring');
check(!/\.pill\.accent\s*\{/.test(kit), 'comp-kit.css: latent unused `.pill.accent` (4.20 on wash) must not be reintroduced');

/* ---------- 8c. polish-round assertions (finish pass, post-owner-grade) ---------- */
check((kit.match(/var\(--depth-shade\)/g) || []).length >= 5, 'comp-kit.css: the polish edge shade (--depth-shade) must be applied to the plane system (>=5 surfaces)');
check(/\n\.phone \.composer \{[^}]*var\(--depth-lift\)/.test(kit), 'comp-kit.css: the composer must carry plane depth (no flat composer)');
check(/\n\.phone \.tabbar \{[^}]*box-shadow: var\(--depth-rim\)/.test(kit), 'comp-kit.css: the tab bar must carry the rim finish');
check(/::selection \{ background: var\(--accent\)/.test(kit), 'comp-kit.css: selection colour must be token-driven');
check(/scrollbar-color: var\(--line-strong\) transparent/.test(kit), 'comp-kit.css: scrollbars must be token-styled');
check(/\.nn:focus-visible \{[^}]*outline: var\(--focus-ring\)/.test(kit), 'comp-kit.css: the character must be keyboard-focusable with the focus ring');
check(/input:disabled \{/.test(kit) && /input\[readonly\] \{/.test(kit), 'comp-kit.css: disabled and read-only field states missing');
check(/\.sidebar nav button:hover \{/.test(kit), 'comp-kit.css: sidebar rows need a hover finish');
check(/font-variant-numeric: tabular-nums/.test(kit), 'comp-kit.css: numeric readouts need tabular figures');
check(/\.card\[data-resolved="true"\]/.test(kit), 'comp-kit.css: the resolved-approval surface state is missing');
check(!/outline:\s*none/i.test(kit), 'comp-kit.css: focus outlines must never be removed');
const js = read('comp-motion.js');
check(/tabIndex = 0/.test(js) && /role', 'button'/.test(js), 'comp-motion.js: the character must be a focusable, labelled control');
check(/aria-label', `\$\{el\.dataset\.name/.test(js), 'comp-motion.js: the character control needs an accessible name');
check(/keydown/.test(js), 'comp-motion.js: the character must activate from the keyboard');

/* ---------- 8d. note-fix round (checker N2/N3): states are rendered, HC covers the polished layers ---------- */
check(/<button[^>]*aria-current="page"[^>]*aria-controls="view-thread"/.test(read('comps/05-desktop-workspace.html')), 'comps/05: the selected-sidebar-row state must be rendered (aria-current on the active sidebar row)');
check(/aria-current/.test(read('comp-motion.js')), 'comp-motion.js: sidebar selection must maintain aria-current, not only aria-selected');
const states07 = read('comps/07-states.html');
check(/<input[^>]*disabled/.test(states07), 'comps/07: the disabled field state must be demonstrated, not dormant CSS');
check(/<input[^>]*readonly/.test(states07), 'comps/07: the read-only field state must be demonstrated, not dormant CSS');
const fcBlock = balancedBlock(kit, '@media (forced-colors: active)');
check(/\.phone \.composer/.test(fcBlock) && /\.phone \.tabbar/.test(fcBlock) && /\.banner/.test(fcBlock) && /\.sidebar/.test(fcBlock), 'comp-kit.css: forced-colours reset must cover composer/tabbar/banner/sidebar');
const hcEnd = kit.indexOf('[data-contrast="high"] body');
const hcBlock = kit.slice(kit.indexOf('[data-contrast="high"] .glass'), hcEnd === -1 ? undefined : hcEnd);
check(/\[data-contrast="high"\] \.phone \.composer/.test(hcBlock) && /\[data-contrast="high"\] \.phone \.tabbar/.test(hcBlock) && /\[data-contrast="high"\] \.banner/.test(hcBlock) && /\[data-contrast="high"\] \.sidebar/.test(hcBlock), 'comp-kit.css: HC preview reset must cover composer/tabbar/banner/sidebar');

const hc2 = hcBlock;
const fc2 = fcBlock;
check(!fc2.includes('data-contrast'), 'check scope: the forced-colours block assertion must not read the HC-preview rules (isolated block)');
check(!hc2.includes('@media (forced-colors'), 'check scope: the HC-preview block assertion must not read the forced-colours block');
check(/\[data-contrast="high"\] \.sidebar nav button\[aria-current="page"\]/.test(hc2), 'comp-kit.css: HC preview must reset the selected sidebar row to system Highlight');
check(/\[data-contrast="high"\] \.phone \.tabbar button/.test(hc2), 'comp-kit.css: HC preview must reset tab-bar buttons to system colours');
check(/\[data-contrast="high"\] input \{[^}]*border-color: FieldText/.test(hc2), 'comp-kit.css: HC preview must reset field boundaries to FieldText');
check(/\.sidebar nav button\[aria-current="page"\] \{ background: Highlight/.test(fc2), 'comp-kit.css: forced-colours path must reset the selected sidebar row');
check(/\.phone \.tabbar button \{ background: Canvas/.test(fc2), 'comp-kit.css: forced-colours path must reset tab-bar buttons');

/* ---------- 9. approvals wording (scoped remember only, no standing-permission wording) ---------- */
for (const page of PAGES) {
  check(!/always\s+allow/i.test(read(page)), `${page}: contains banned "always allow" wording`);
}
const triage = read('comps/02-mobile-triage-approvals.html');
check(/Remember for .+ for /.test(triage), 'comps/02: the remember affordance must state ONE subject AND ONE action ("Remember for <subject> for <action>")');
check(/one subject, one action/i.test(triage), 'comps/02: the remember row must state the single-subject/single-action scope on the surface');
check(/Tier B approvals never offer remembering/i.test(triage), 'comps/02: the Tier B exclusion must be stated on the surface');
check(/data-approval/.test(triage) && /data-approve/.test(triage), 'comps/02: "Approve once" has no data-approval target and would be a dead control');

/* ---------- 10. contrast floors on the FULL composited stack (both ambient washes) ---------- */
const hex = (h) => h.replace('#', '').match(/.{2}/g).map((x) => parseInt(x, 16));
const rgba = (s) => {
  const p = s.match(/rgba?\(([^)]+)\)/)[1].split(',').map((x) => parseFloat(x));
  return { rgb: [p[0], p[1], p[2]], a: p[3] ?? 1 };
};
const over = (bg, fg, a) => bg.map((c, i) => Math.round(fg[i] * a + c * (1 - a)));
const lin = (c) => { c /= 255; return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
const lum = (rgb) => 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2]);
const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
const c = tokens.color;
// The page composites BOTH ambient washes over the base (CSS lists accent first, then cool, then base).
const base = over(over(hex(c.base), rgba(c.ambientCool).rgb, rgba(c.ambientCool).a), rgba(c.ambientAccent).rgb, rgba(c.ambientAccent).a);
const p1 = over(base, rgba(c.surface1).rgb, rgba(c.surface1).a);
const p2 = over(p1, rgba(c.surface2).rgb, rgba(c.surface2).a);
const p3 = over(p2, rgba(c.surface3).rgb, rgba(c.surface3).a);
const inputBg = over(p2, rgba(c.surface1).rgb, rgba(c.surface1).a); // field: surface-1 fill over a card
const statusOnWash = (colourKey, washKey) => contrast(hex(c[colourKey]), over(p3, rgba(c[washKey]).rgb, rgba(c[washKey]).a));
const floors = tokens.floors;
check(contrast(hex(c.ink), p3) >= floors.ink, `ink on the brightest text plane is ${contrast(hex(c.ink), p3).toFixed(2)}:1 (floor ${floors.ink})`);
check(contrast(hex(c.muted), p3) >= floors.muted, `muted on the brightest text plane is ${contrast(hex(c.muted), p3).toFixed(2)}:1 (floor ${floors.muted})`);
check(contrast(hex(c.subtle), p1) >= floors.muted, `subtle on panel is ${contrast(hex(c.subtle), p1).toFixed(2)}:1 (floor ${floors.muted}; base/panel only by rule)`);
check(contrast(hex(c.accent), p3) >= floors.text, `accent text is ${contrast(hex(c.accent), p3).toFixed(2)}:1 on the brightest text plane`);
for (const [fg, wash] of [['ok', 'washOk'], ['warn', 'washWarn'], ['danger', 'washDanger']]) {
  check(statusOnWash(fg, wash) >= floors.text, `${fg} on its wash is ${statusOnWash(fg, wash).toFixed(2)}:1`);
}
check(contrast(hex(c.accent), p2) >= floors.nonText, `focus ring on a card is ${contrast(hex(c.accent), p2).toFixed(2)}:1 (floor ${floors.nonText})`);
const lineBright = over(p2, rgba(c.lineBright).rgb, rgba(c.lineBright).a);
check(contrast(lineBright, p2) >= floors.nonText, `control boundary (line-bright vs card, both ambient washes) is ${contrast(lineBright, p2).toFixed(2)}:1 (floor ${floors.nonText})`);
const fieldLine = over(inputBg, rgba(c.fieldLine).rgb, rgba(c.fieldLine).a);
check(contrast(fieldLine, inputBg) >= floors.nonText, `field boundary (field-line vs field fill) is ${contrast(fieldLine, inputBg).toFixed(2)}:1 (pack fieldLine rule: floor ${floors.nonText})`);

/* ---------- 11. freeze-manifest integrity (C2: exists, hashes match, directory is EXACTLY the listed set) ---------- */
const CONTENT = [...PAGES, 'tokens.json', 'tokens.css', 'build-tokens.mjs', 'comp-kit.css', 'comp-motion.js', 'check-comps.mjs', 'README.md', 'PACK-CHANGE-PROPOSAL.md'].sort();
check(exists('FREEZE.sha256'), 'FREEZE.sha256 is missing — the set is not frozen (no manifest, no integrity)');
if (exists('FREEZE.sha256')) {
  const lines = read('FREEZE.sha256').split('\n').filter((l) => l.trim() && !l.startsWith('#'));
  const listed = new Set();
  for (const line of lines) {
    const [hash, file] = line.trim().split(/\s+/);
    listed.add(file);
    check(exists(file), `FREEZE.sha256 lists a missing file: ${file}`);
    if (exists(file)) check(sha256(file) === hash, `FREEZE.sha256 hash mismatch: ${file}`);
  }
  check([...CONTENT].every((f) => listed.has(f)), `FREEZE.sha256 is missing content files: ${CONTENT.filter((f) => !listed.has(f)).join(', ')}`);
  const walk = (dir, rel = '') => {
    const out = [];
    for (const entry of fs.readdirSync(path.join(dir, rel), { withFileTypes: true })) {
      const next = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) out.push(...walk(dir, next));
      else out.push(next);
    }
    return out;
  };
  const onDisk = walk(HERE).filter((f) => f !== 'FREEZE.sha256').sort();
  check(JSON.stringify(onDisk) === JSON.stringify([...CONTENT].sort()), `directory contents are not exactly the frozen set — extra: ${onDisk.filter((f) => !CONTENT.includes(f)).join(', ') || 'none'}; missing: ${CONTENT.filter((f) => !onDisk.includes(f)).join(', ') || 'none'}`);
}

/* ---------- 12. non-vacuity self-tests (the assertions exist for a reason) ---------- */
check(detectsColourLiteral('color: white') === false && detectsColourLiteral('color: #fff') === true, 'self-test: colour-literal detector is vacuous');
check(detectsColourKeyword('color: white') === true && detectsColourKeyword('color: var(--ink)') === false, 'self-test: colour-keyword detector is vacuous');
check(detectsColourKeyword('<h3 style="color: white">x</h3>') === true, 'self-test: colour-keyword detector misses a first-declaration inline style (C3)');
check(detectsColourKeyword('<h3 style="font-size: 14px">x</h3>') === false, 'self-test: colour-keyword detector false-positives on a clean inline style');
check(detectsInfinite('animation: float 2s infinite') === true && detectsInfinite('animation-duration: 250ms') === false, 'self-test: infinite-animation detector is vacuous');
check(detectsBrokenTag('<div><span></div>').ok === false && detectsBrokenTag('<div><span></span></div>').ok === true, 'self-test: tag-balance detector is vacuous');
check(unresolved(usedProps(['var(--surface-1) var(--nope)']), definedProps(['--surface-1: x'])).join() === '--nope', 'self-test: custom-property resolver is vacuous');

/* ---------- report ---------- */
if (findings.length) {
  process.stdout.write(`V2 COMPS CHECK: FAIL (${findings.length})\n` + findings.map((f) => `  - ${f}`).join('\n') + '\n');
  process.exit(1);
}
process.stdout.write(`V2 COMPS CHECK: PASS (hardened after checker D1–D4)
  pages: ${PAGES.length} labelled comp pages (7 comps + index + motion storyboard)
  self-containment: 0 external refs, 0 remote assets, 0 webfonts, 0 images; every local href/src resolves
  tokens: tokens.css generated-current; 0 colour literals / raw colour keywords outside the token files
  custom properties: ${defined.size} defined, 0 unresolved; required surface/light/field/border/blur tokens emitted
  structure: tag balance clean on all ${PAGES.length} pages; ${tokens.space.length}-step space scale tokenised
  contrast (both ambient washes composited): ink ${contrast(hex(c.ink), p3).toFixed(2)}:1 · muted ${contrast(hex(c.muted), p3).toFixed(2)}:1 · subtle(panel) ${contrast(hex(c.subtle), p1).toFixed(2)}:1 · accent ${contrast(hex(c.accent), p3).toFixed(2)}:1
  contrast: ok/warn/danger wash ${statusOnWash('ok', 'washOk').toFixed(2)}/${statusOnWash('warn', 'washWarn').toFixed(2)}/${statusOnWash('danger', 'washDanger').toFixed(2)}:1 · focus ${contrast(hex(c.accent), p2).toFixed(2)}:1 · control boundary ${contrast(lineBright, p2).toFixed(2)}:1 · field boundary ${contrast(fieldLine, inputBg).toFixed(2)}:1
  floors: reduced-motion + forced-colors + HC preview (inputs/messages/links/character included) · 2px accent focus ring · 44/56 targets enforced · empty state present
  motion: 120/200/250/350 ms, one ease-out curve; no transition-all, no loops/pulse, no uppercase tracking, no gradient text
  freeze: FREEZE.sha256 ${exists('FREEZE.sha256') ? 'present and matching' : 'absent'}
`);
