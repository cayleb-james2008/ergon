#!/usr/bin/env node
/**
 * Self-check for the Ergon Lumen v2 direction comps (maker's own evidence; a fresh checker still re-runs it).
 * Asserts: design-comp labels · self-containment (no external refs/assets) · token discipline (no colour literals
 * outside tokens.json/tokens.css) · tokens.css generated-current · contrast floors on the translucent stack ·
 * accessibility floors present · motion budgets and no banned motion tells.
 * Run: node check-comps.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { build } from './build-tokens.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const read = (f) => fs.readFileSync(path.join(HERE, f), 'utf8');
const exists = (f) => fs.existsSync(path.join(HERE, f));
const findings = [];
const check = (ok, msg) => { if (!ok) findings.push(msg); };

const PAGES = ['index.html', 'motion-storyboard.html', ...fs.readdirSync(path.join(HERE, 'comps')).filter((f) => f.endsWith('.html')).map((f) => `comps/${f}`)];
const CODE = ['comp-kit.css', 'comp-motion.js', 'index.html', 'motion-storyboard.html', ...PAGES.filter((p) => p.startsWith('comps/'))];
const tokens = JSON.parse(read('tokens.json'));

/* ---------- 1. design-comp labels ---------- */
for (const page of PAGES) check(/DESIGN COMP — NOT A BUILT SURFACE/.test(read(page)), `${page}: missing the "DESIGN COMP — NOT A BUILT SURFACE" label`);

/* ---------- 2. self-containment: no external refs, no remote assets, no webfonts ---------- */
for (const file of CODE) {
  // The only permitted URL-shaped string is the SVG XML namespace identifier used by createElementNS — it is a name,
  // never a network fetch. Strip it, then assert nothing else URL-shaped remains.
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
}

/* ---------- 3. token discipline: no colour literals outside tokens.json / generated tokens.css ---------- */
const COLOUR = /#(?:[0-9a-f]{3,8})\b|\brgba?\(|\bhsla?\(/gi;
for (const file of CODE) {
  const text = read(file).replace(/@media[^{]*\{/g, ''); // keep it simple; colours are checked anywhere
  const hits = [...text.matchAll(COLOUR)].map((m) => m[0]);
  check(hits.length === 0, `${file}: hand-typed colour literal(s) ${hits.slice(0, 5).join(', ')} — use the generated tokens`);
}
check(exists('tokens.css'), 'tokens.css missing');
check(read('tokens.css') === build(tokens), 'tokens.css is stale — run node build-tokens.mjs');

/* ---------- 4. motion budgets + no banned tells ---------- */
const m = tokens.motion;
check(m.tap === '120ms' && m.swap === '200ms' && m.enter === '250ms' && m.sheet === '350ms', 'motion budgets must be 120/200/250/350 ms');
check(/^cubic-bezier\(/.test(m.ease), 'one ease-out curve token missing');
for (const file of ['comp-kit.css', 'comp-motion.js', ...PAGES.filter((p) => p.includes('/'))]) {
  const text = read(file);
  check(!/transition-all/i.test(text), `${file}: transition-all is banned`);
  check(!/text-transform:\s*uppercase/i.test(text), `${file}: uppercase tracked labels are banned (unslop)`);
  check(!/background-clip:\s*text|-webkit-background-clip:\s*text/i.test(text), `${file}: gradient text is banned`);
  check(!/animation-iteration-count:\s*infinite|@keyframes\s+[^{]*(pulse|ping|glow)/i.test(text), `${file}: looping/pulse animations are banned`);
  check(!/animation-name:\s*[^;]*(pulse|ping|glow)/i.test(text), `${file}: pulse/ping/glow animations are banned`);
}

/* ---------- 5. accessibility floors present ---------- */
const kit = read('comp-kit.css');
check(/@media\s*\(prefers-reduced-motion:\s*reduce\)/.test(kit), 'comp-kit.css: missing the reduced-motion floor');
check(/@media\s*\(forced-colors:\s*active\)/.test(kit), 'comp-kit.css: missing the High Contrast (forced-colors) floor');
check(/\[data-contrast="high"\]/.test(kit), 'comp-kit.css: missing the High Contrast preview floor');
check(/outline:\s*var\(--focus-ring\)\s*solid\s*var\(--accent\)/.test(kit), 'comp-kit.css: focus ring must be 2px accent with a gap');
check(/outline-offset:\s*var\(--focus-gap\)/.test(kit), 'comp-kit.css: focus ring gap missing');
check(/min-height:\s*var\(--target\)|min-height:\s*var\(--target-employee\)/.test(kit), 'comp-kit.css: target floor missing');

/* ---------- 6. contrast floors on the translucent stack ---------- */
const hex = (h) => h.replace('#', '').match(/.{2}/g).map((x) => parseInt(x, 16));
const rgba = (s) => {
  const p = s.match(/rgba?\(([^)]+)\)/)[1].split(',').map((x) => parseFloat(x));
  return { rgb: [p[0], p[1], p[2]], a: p[3] ?? 1 };
};
const over = (bg, fg, a) => bg.map((c, i) => Math.round(fg[i] * a + c * (1 - a)));
const lin = (c) => { c /= 255; return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
const lum = (rgb) => 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2]);
const contrast = (a, b) => { const la = lum(a), lb = lum(b); return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05); };
const base = over(hex(tokens.color.base), rgba(tokens.color.ambientAccent).rgb, rgba(tokens.color.ambientAccent).a);
const p1 = over(base, rgba(tokens.color.surface1).rgb, rgba(tokens.color.surface1).a);
const p2 = over(p1, rgba(tokens.color.surface2).rgb, rgba(tokens.color.surface2).a);
const p3 = over(p2, rgba(tokens.color.surface3).rgb, rgba(tokens.color.surface3).a);
const statusOnWash = (colourKey, washKey) => {
  const w = rgba(tokens.color[washKey]);
  return contrast(hex(tokens.color[colourKey]), over(p3, w.rgb, w.a));
};
const floors = tokens.floors;
check(contrast(hex(tokens.color.ink), p3) >= floors.ink, `ink on the brightest text plane is ${contrast(hex(tokens.color.ink), p3).toFixed(2)}:1 (floor ${floors.ink})`);
check(contrast(hex(tokens.color.muted), p3) >= floors.muted, `muted on the brightest text plane is ${contrast(hex(tokens.color.muted), p3).toFixed(2)}:1 (floor ${floors.muted})`);
check(contrast(hex(tokens.color.subtle), p1) >= floors.muted, `subtle on panel (surface-1) is ${contrast(hex(tokens.color.subtle), p1).toFixed(2)}:1 (floor ${floors.muted}; subtle is base/panel-only by rule)`);
check(contrast(hex(tokens.color.accent), p3) >= floors.text, `accent text is ${contrast(hex(tokens.color.accent), p3).toFixed(2)}:1 on the brightest text plane`);
check(statusOnWash('ok', 'washOk') >= floors.text, `ok on its wash is ${statusOnWash('ok', 'washOk').toFixed(2)}:1`);
check(statusOnWash('warn', 'washWarn') >= floors.text, `warn on its wash is ${statusOnWash('warn', 'washWarn').toFixed(2)}:1`);
check(statusOnWash('danger', 'washDanger') >= floors.text, `danger on its wash is ${statusOnWash('danger', 'washDanger').toFixed(2)}:1`);
check(contrast(hex(tokens.color.accent), p2) >= floors.nonText, `focus ring on a card is ${contrast(hex(tokens.color.accent), p2).toFixed(2)}:1 (floor ${floors.nonText})`);
const lb = over(p2, rgba(tokens.color.lineBright).rgb, rgba(tokens.color.lineBright).a);
// lineBright is translucent over the surface; composite it and compare with the adjacent plane
check(contrast(lb, p2) >= floors.nonText, `control boundary (line-bright) on a card is ${contrast(lb, p2).toFixed(2)}:1 (floor ${floors.nonText})`);

/* ---------- 6b. approvals wording (Nox RUN-2 ruling: scoped remember only, no "always allow") ---------- */
for (const page of PAGES) {
  check(!/always\s+allow/i.test(read(page)), `${page}: contains "always allow" wording (banned by the Tier A remember ruling)`);
}
const triage = read('comps/02-mobile-triage-approvals.html');
check(/Remember for .+ for /.test(triage), 'comps/02: the remember affordance must state ONE subject AND ONE action ("Remember for \<subject\> for \<action\>")');
check(/one subject, one action/i.test(triage), 'comps/02: the remember row must state the single-subject/single-action scope on the surface');
check(/Tier B approvals never offer remembering/i.test(triage), 'comps/02: the Tier B exclusion must be stated on the surface');

/* ---------- 7. inventory ---------- */
for (const need of ['tokens.json', 'tokens.css', 'build-tokens.mjs', 'comp-kit.css', 'comp-motion.js', 'index.html', 'motion-storyboard.html', ...['01-mobile-chat', '02-mobile-triage-approvals', '03-mobile-tasks', '04-mobile-make', '05-desktop-workspace', '06-character-sheet', '07-states'].map((n) => `comps/${n}.html`)]) {
  check(exists(need), `missing required file: ${need}`);
}

/* ---------- report ---------- */
if (findings.length) {
  process.stdout.write(`V2 COMPS CHECK: FAIL (${findings.length})\n` + findings.map((f) => `  - ${f}`).join('\n') + '\n');
  process.exit(1);
}
const text = `V2 COMPS CHECK: PASS
  pages: ${PAGES.length} labelled comp pages (7 comps + index + motion storyboard)
  self-containment: 0 external refs, 0 remote assets, 0 webfonts, 0 images
  tokens: tokens.css generated-current from tokens.json; 0 colour literals outside the token files
  contrast: ink ${contrast(hex(tokens.color.ink), p3).toFixed(2)}:1 · muted ${contrast(hex(tokens.color.muted), p3).toFixed(2)}:1 · subtle(panel) ${contrast(hex(tokens.color.subtle), p1).toFixed(2)}:1 · accent ${contrast(hex(tokens.color.accent), p3).toFixed(2)}:1 · ok/warn/danger wash ${statusOnWash('ok', 'washOk').toFixed(2)}/${statusOnWash('warn', 'washWarn').toFixed(2)}/${statusOnWash('danger', 'washDanger').toFixed(2)}:1 · focus ${contrast(hex(tokens.color.accent), p2).toFixed(2)}:1 · border ${contrast(lb, p2).toFixed(2)}:1
  floors: reduced-motion + forced-colors + High Contrast preview present; 2px accent focus ring with 2px gap
  motion: 120/200/250/350 ms, one ease-out curve; no transition-all, no loops/pulse, no uppercase tracking, no gradient text
`;
process.stdout.write(text);
