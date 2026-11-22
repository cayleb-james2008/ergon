// sync-used.mjs: copies the library parts this app imports from the house-frontend starter into ../rune/used,
// follows both @/ and relative imports of every copied part, and writes rune/used/MANIFEST.md.
// Run after adding or removing a part import: npm run sync-used
import fs from "node:fs";
import path from "node:path";
const root = path.resolve(import.meta.dirname, "..");
const used = path.resolve(root, "../rune/used");
const HOUSE = process.env.HOUSE_SRC
  || (fs.existsSync("/home/cayleb/.claude/skills/house-frontend/starter/src")
    ? "/home/cayleb/.claude/skills/house-frontend/starter/src"
    : "/home/cayleb/Work/shared-workflows/house-frontend/skill/starter/src");
const exts = ["", ".tsx", ".ts", "/index.tsx", "/index.ts"];
const LIB = [
  [/components\/ui\/(texture-button|expandable)/, "Cult UI (cult-ui.com)", "MIT"],
  [/components\/kibo-ui\//, "Kibo UI (kibo-ui.com)", "MIT"],
  [/components\/(SparkChart|Tracker|ProgressCircle|BarList|CategoryBar|Tooltip)\/|utils\/cx/, "Tremor Raw (tremor.so)", "Apache-2.0"],
  [/components\/ui\/(morphing-dialog|animated-background|animated-group|sliding-number|text-morph|text-shimmer|border-trail|spotlight|magnetic|in-view|scroll-progress|transition-panel|dock|dynamic-island)/, "Motion Primitives (motion-primitives.com)", "MIT"],
  [/components\/ui\/(button|spinner|input-group|input|textarea|kbd|tabs|segmented-control|badge|avatar|table|tooltip)/, "COSS UI (coss.com/ui)", "MIT"],
  [/components\/ui\//, "shadcn/ui on Radix UI primitives", "MIT"],
  [/(lib|motion|hooks)/, "House presets (house-frontend starter)", "owner's"],
];
const lib = (rel) => LIB.find(([re]) => re.test(rel)) || [null, "House frontend starter", "owner's"];
const find = (base, id) => { for (const e of exts) { const p = path.join(base, id + e); if (fs.existsSync(p) && fs.statSync(p).isFile()) return p; } return null; };
const seen = new Map();
const queue = [];
const walkSrc = (d) => fs.readdirSync(d, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walkSrc(path.join(d, e.name)) : /\.(tsx?|css)$/.test(e.name) ? [path.join(d, e.name)] : []));
const importsOf = (file) => [...fs.readFileSync(file, "utf8").matchAll(/(?:from|import)\s+["']([^"']+)["']/g)].map((m) => m[1]);

for (const f of walkSrc(path.join(root, "src"))) {
  for (const spec of importsOf(f)) {
    if (spec.startsWith("@/") && !find(path.join(root, "src"), spec.slice(2))) queue.push([spec.slice(2), path.relative(root, f)]);
  }
}
while (queue.length) {
  const [id, by] = queue.shift();
  const src = find(HOUSE, id);
  if (!src) { console.error(`NOT FOUND in the house starter: @/${id} (imported by ${by})`); process.exitCode = 1; continue; }
  const rel = path.relative(HOUSE, src);
  if (seen.has(rel)) { seen.get(rel).add(by); continue; }
  seen.set(rel, new Set([by]));
  const dest = path.join(used, rel);
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  const local = fs.existsSync(path.join(used, "LOCAL-EDITS.md")) ? fs.readFileSync(path.join(used, "LOCAL-EDITS.md"), "utf8") : "";
  if (!(fs.existsSync(dest) && local.includes("`" + rel + "`"))) fs.copyFileSync(src, dest);
  for (const sub of importsOf(fs.existsSync(dest) ? dest : src)) {
    if (sub.startsWith("@/")) queue.push([sub.slice(2), "rune/used/" + rel]);
    else if (sub.startsWith(".")) {
      const resolved = find(path.dirname(src), sub);
      if (resolved && resolved.startsWith(HOUSE)) queue.push([path.relative(HOUSE, resolved).replace(/\.(tsx?)$/, ""), "rune/used/" + rel]);
    }
  }
}
const have = (d) => (fs.existsSync(d) ? walkSrc(d) : []);
for (const f of have(used)) { const rel = path.relative(used, f); if (!seen.has(rel)) { fs.rmSync(f); console.log("removed unused part", rel); } }
const rows = [...seen.keys()].sort().map((rel) => { const [, l, lic] = lib(rel); return `| \`${rel}\` | ${l} | ${lic} | ${[...seen.get(rel)].sort().join(", ")} |`; });
fs.mkdirSync(used, { recursive: true });
fs.writeFileSync(path.join(used, "MANIFEST.md"), `# rune/used: the library parts this app uses

Copied from the owner's house-frontend starter by \`frontend/scripts/sync-used.mjs\` (local Hallmark-clean edits recorded in \`rune/used/LOCAL-EDITS.md\`). Only parts the screens import are here; \`npm run sync-used\` refreshes this folder and this table.

| Part | Library | Licence | Imported by |
| --- | --- | --- | --- |
${rows.join("\n")}
`);
console.log(`rune/used: ${seen.size} parts`);
