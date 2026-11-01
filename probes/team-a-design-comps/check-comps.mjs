#!/usr/bin/env node
// Verifies the Ergon design-comp pack: seven comps + motion storyboard + index, grading banner everywhere,
// self-containment (no URLs/scripts/images/external assets), tokens-only (no hex literals), and that the generated
// tokens.css matches the Ergon pack's design/tokens.json. Workspace tooling only; not part of any product.
// Usage: node check-comps.mjs --tokens /path/to/design/tokens.json [--root DIR]
// Exit 0: pack valid. 1: problems named. 2: usage.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const argv = process.argv.slice(2);
const opt = { root: HERE };
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === '--help' || argv[i] === '-h') {
    process.stdout.write('Usage: node check-comps.mjs --tokens /path/to/design/tokens.json [--root DIR]\n');
    process.exit(0);
  }
  if (argv[i] === '--tokens' || argv[i] === '--root') {
    const v = argv[++i];
    if (!v) { process.stderr.write(`${argv[i - 1]} needs a value\n`); process.exit(2); }
    opt[argv[i - 1].slice(2)] = v;
  } else { process.stderr.write(`unknown argument: ${argv[i]}\nUsage: node check-comps.mjs --tokens PATH\n`); process.exit(2); }
}
const KEBAB = (n) => n.replace(/[A-Z]/g, (m) => '-' + m.toLowerCase());
if (!opt.tokens) { process.stderr.write('Usage: node check-comps.mjs --tokens /path/to/design/tokens.json\n'); process.exit(2); }
if (!fs.existsSync(opt.tokens)) { process.stderr.write(`tokens file not found: ${opt.tokens}\n`); process.exit(1); }
const root = path.resolve(opt.root);
const problems = [];
const check = (ok, msg) => { if (!ok) problems.push(msg); };
const read = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');

const COMP_FILES = ['01-mobile-chat.html', '02-mobile-triage-approvals.html', '03-mobile-tasks.html',
  '04-mobile-make.html', '05-desktop-workspace.html', '06-character-sheet.html', '07-states.html'];
const STATIC_FILES = ['index.html', 'motion-storyboard.html'];

// Token expectations, computed from the pack's single source.
const tokens = JSON.parse(fs.readFileSync(opt.tokens, 'utf8'));
const expected = new Map();
for (const [name, value] of Object.entries(tokens.color)) expected.set('--' + KEBAB(name), value.toUpperCase());
for (const name of tokens.wash.of) {
  const hex = tokens.color[name].replace('#', '');
  const r = parseInt(hex.slice(0, 2), 16), g = parseInt(hex.slice(2, 4), 16), b = parseInt(hex.slice(4, 6), 16);
  expected.set('--' + KEBAB(name) + '-wash', `rgba(${r},${g},${b},${(tokens.wash.alpha / 255).toFixed(2)})`);
}
expected.set('--body', `${tokens.type.it.body}px`); expected.set('--heading', `${tokens.type.it.heading}px`); expected.set('--caption', `${tokens.type.it.caption}px`);
expected.set('--ember-body', `${tokens.type.employee.body}px`); expected.set('--ember-heading', `${tokens.type.employee.heading}px`); expected.set('--ember-caption', `${tokens.type.employee.caption}px`);
expected.set('--radius', `${tokens.radius.panel}px`); expected.set('--radius-sm', `${tokens.radius.control}px`); expected.set('--radius-pill', '999px');
expected.set('--target', `${tokens.target.min}px`); expected.set('--target-employee', `${tokens.target.employee}px`);
tokens.space.forEach((px, i) => expected.set(`--space-${i + 1}`, `${px}px`));
expected.set('--ui', `"${tokens.type.family}",system-ui,sans-serif`); expected.set('--mono', `"${tokens.type.mono}",ui-monospace,monospace`);

// 1. Structure.
for (const f of [...STATIC_FILES, 'README.md', 'tokens.css', 'comp-kit.css', 'build-tokens.mjs']) check(fs.existsSync(path.join(root, f)), `${f} missing`);
for (const f of COMP_FILES) check(fs.existsSync(path.join(root, 'comps', f)), `comps/${f} missing`);
const compsDir = path.join(root, 'comps');
const compHtml = fs.existsSync(compsDir) ? fs.readdirSync(compsDir).filter((f) => f.endsWith('.html')).sort() : [];
check(compHtml.length === COMP_FILES.length, `comps/ must hold exactly the ${COMP_FILES.length} comps, found ${compHtml.length}`);
check(JSON.stringify(compHtml) === JSON.stringify([...COMP_FILES].sort()), 'comps/ holds only the seven expected comp files');

// 2. Every HTML artefact: grading banner, self-contained, tokens-only.
const htmlFiles = [...STATIC_FILES, ...COMP_FILES.map((f) => 'comps/' + f)];
for (const rel of htmlFiles) {
  if (!fs.existsSync(path.join(root, rel))) continue;
  const html = read(rel);
  check(/DESIGN COMP — NOT A BUILT SURFACE/.test(html), `${rel}: missing the "DESIGN COMP — NOT A BUILT SURFACE" banner`);
  check(!/https?:\/\//.test(html), `${rel}: contains an external URL`);
  check(!/@import/i.test(html), `${rel}: contains @import`);
  check(!/url\s*\(/i.test(html), `${rel}: contains url() resource`);
  check(!/<script/i.test(html), `${rel}: contains a script (comps are static, no code)`);
  check(!/<img\b/i.test(html), `${rel}: contains an image element`);
  check(!/<[^>]+\bsrc\s*=/i.test(html), `${rel}: contains a src= attribute`);
  check(!/(?<!&)#[0-9a-f]{3,8}\b/i.test(html), `${rel}: contains a hex colour literal (use var(--token) from tokens.css)`);
  const sheets = [...html.matchAll(/<link\s+rel="stylesheet"\s+href="([^"]+)"/g)].map((m) => m[1]);
  for (const href of sheets) {
    check(!/^[a-z]+:/i.test(href), `${rel}: stylesheet href ${href} is not relative`);
    check(fs.existsSync(path.resolve(path.dirname(path.join(root, rel)), href)), `${rel}: stylesheet ${href} does not resolve`);
  }
  if (rel.startsWith('comps/')) {
    check(sheets.includes('../tokens.css'), `${rel}: must link ../tokens.css`);
    check(sheets.includes('../comp-kit.css'), `${rel}: must link ../comp-kit.css`);
  } else {
    check(sheets.includes('tokens.css'), `${rel}: must link tokens.css`);
    check(sheets.includes('comp-kit.css'), `${rel}: must link comp-kit.css`);
  }
}

// 3. Index links every comp + the storyboard.
if (fs.existsSync(path.join(root, 'index.html'))) {
  const index = read('index.html');
  for (const f of COMP_FILES) check(index.includes(`comps/${f}`), `index.html does not link comps/${f}`);
  check(index.includes('motion-storyboard.html'), 'index.html does not link the motion storyboard');
}

// 4. Generated tokens.css matches the pack exactly (single source; no drift).
const css = read('tokens.css');
const m = css.match(/:root\{[^}]*\}/);
check(Boolean(m), 'tokens.css has no :root block');
if (m) {
  const declared = new Map(m[0].slice(':root{'.length, -1).split(';').filter(Boolean).map((d) => { const i = d.indexOf(':'); return [d.slice(0, i), d.slice(i + 1)]; }));
  declared.delete('color-scheme'); // housekeeping property written by build-tokens.mjs, not a design token
  check(declared.size === expected.size, `tokens.css declares ${declared.size} properties, expected ${expected.size}`);
  for (const [name, value] of expected) check(declared.get(name) === value, `tokens.css ${name} is ${JSON.stringify(declared.get(name))}, expected ${JSON.stringify(value)}`);
  for (const name of declared.keys()) check(expected.has(name), `tokens.css declares unexpected property ${name}`);
}
check(/GENERATED by build-tokens\.mjs/.test(css), 'tokens.css is not marked as generated');

if (problems.length) {
  process.stdout.write(`COMP PACK PROBLEMS (${problems.length}):\n` + problems.map((p) => `  - ${p}`).join('\n') + '\n');
  process.exit(1);
}
process.stdout.write(`COMP PACK OK: ${COMP_FILES.length} comps + motion storyboard + index (${htmlFiles.length} static HTML files); ` +
  `banner in every file; no external assets/scripts/images/hex literals; tokens.css ${expected.size}/${expected.size} properties match the pack tokens\n`);
