#!/usr/bin/env node
// Validates the Job 03 run-evidence skeleton: structure, required fields, and no secret-like content.
// Usage: node check-skeleton.mjs [ROOT] [--passes N]
// Exit 0: skeleton complete (prints "SKELETON OK"). 1: problems (each named). 2: usage.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
if (args.includes('--help') || args.includes('-h')) {
  process.stdout.write('Usage: node check-skeleton.mjs [ROOT] [--passes N]\n');
  process.exit(0);
}
let root = HERE;
let passes = 2;
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--passes') { passes = Number(args[++i]); }
  else if (!args[i].startsWith('-')) { root = path.resolve(args[i]); }
  else { process.stdout.write(`unknown argument: ${args[i]}\n`); process.exit(2); }
}
if (!Number.isInteger(passes) || passes < 1 || passes > 9) { process.stdout.write('--passes must be 1..9\n'); process.exit(2); }

const FINDINGS = [];
function check(ok, message) { if (!ok) FINDINGS.push(message); }
function read(file) { return fs.readFileSync(file, 'utf8'); }
function exists(rel) { return fs.existsSync(path.join(root, rel)); }

const STEP_DIRS = [['R0-pre', 'R0 — pre-pass'], ['R1', 'R1 — company setup'], ['R2', 'R2 — first agents'],
  ['R3', 'R3 — shareable URL'], ['R4', 'R4 — phone enrolment'], ['R5', 'R5 — desk task'], ['R6', 'R6 — voice/message'],
  ['R7-close', 'R7 — close-out and second pass']];
const NOTE_FIELDS = ['- Step:', '- Pass:', '- Owner:', '- Operator action:', '- Expected:', '- Actual:', '- Word:',
  '- Evidence files:', '- Timestamp (UTC):', '- Blocker / finding:'];
const SECRET_PATTERNS = [
  { re: /sk-ant-[A-Za-z0-9_-]{8,}/, why: 'Anthropic key lookalike' },
  { re: /ANTHROPIC_API_KEY\s*[:=]\s*\S{8,}/i, why: 'Anthropic key assignment' },
  { re: /(password|passwd|secret|token|api[_-]?key)\s*[:=]\s*[A-Za-z0-9_\-\/+=]{8,}/i, why: 'secret-like assignment' },
  { re: /service_role[_ ]key/i, why: 'Supabase service-role key' },
  { re: /eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}/, why: 'JWT lookalike' },
];

check(exists('README.md'), 'README.md missing');
check(exists('check-skeleton.mjs'), 'check-skeleton.mjs missing');
check(exists('r0-preflight.sh'), 'r0-preflight.sh missing');

for (let n = 1; n <= passes; n++) {
  const pass = `pass-${n}`;
  check(exists(pass), `${pass}/ missing`);
  if (!exists(pass)) continue;
  check(exists(`${pass}/index.md`), `${pass}/index.md missing`);
  const index = exists(`${pass}/index.md`) ? read(path.join(root, pass, 'index.md')) : '';
  for (const [id] of STEP_DIRS) check(index.includes(`| ${id === 'R7-close' ? 'R7' : id.replace('R0-pre', 'R0')} |`),
    `${pass}/index.md has no verdict row for ${id === 'R7-close' ? 'R7' : id}`);
  check(index.includes('## Checker sign-off'), `${pass}/index.md has no checker sign-off section`);
  check(index.includes('## Secrets check'), `${pass}/index.md has no secrets-check section`);
  for (const [dir, label] of STEP_DIRS) {
    const dirPath = `${pass}/${dir}`;
    check(exists(dirPath), `${dirPath}/ missing`);
    if (!exists(dirPath)) continue;
    const notes = `${dirPath}/notes.md`;
    check(exists(notes), `${notes} missing (${label})`);
    if (!exists(notes)) continue;
    const text = read(path.join(root, notes));
    for (const field of NOTE_FIELDS) check(text.includes(field), `${notes} missing field "${field}"`);
  }
  for (const f of ['host-fingerprint.txt', 'package-sha256.txt', 'run-config-check.txt', 'project-id.txt',
    'absence-of-company-state.txt']) check(exists(`${pass}/R0-pre/${f}`), `${pass}/R0-pre/${f} missing`);
  for (const f of ['state-record.md', 'host-teardown.txt', 'pass-diff.md']) {
    check(exists(`${pass}/R7-close/${f}`), `${pass}/R7-close/${f} missing`);
  }
}

// No secret-like content anywhere in the skeleton.
function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(p);
    else if (entry.isFile() && !entry.name.endsWith('.png') && !entry.name.endsWith('.jpg')) {
      const text = read(p);
      for (const { re, why } of SECRET_PATTERNS) {
        if (re.test(text)) FINDINGS.push(`${path.relative(root, p)} contains a ${why} — remove it before filing`);
      }
    }
  }
}
walk(root);

if (FINDINGS.length) {
  process.stdout.write(`SKELETON PROBLEMS (${FINDINGS.length}):\n` + FINDINGS.map(f => `  - ${f}`).join('\n') + '\n');
  process.exit(1);
}
const stepCount = STEP_DIRS.length * passes;
process.stdout.write(`SKELETON OK: ${passes} passes, ${stepCount} step directories, ` +
  `${NOTE_FIELDS.length} required note fields each, secrets scan clean\n`);
