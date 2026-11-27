// Cross-platform helper ensuring ../rune/node_modules points to frontend/node_modules
// so TypeScript and Vite can resolve bare package imports from ../rune/used/* on fresh clones.
import fs from "node:fs";
import path from "node:path";

const frontendDir = path.resolve(import.meta.dirname, "..");
const frontendNodeModules = path.join(frontendDir, "node_modules");
const runeDir = path.resolve(frontendDir, "../rune");
const runeNodeModules = path.join(runeDir, "node_modules");

if (!fs.existsSync(frontendNodeModules) || !fs.existsSync(runeDir)) {
  process.exit(0);
}

let needsLink = true;
try {
  const stat = fs.lstatSync(runeNodeModules);
  if (stat.isSymbolicLink() || stat.isDirectory()) {
    // Verify target resolves cleanly to an existing directory.
    if (fs.existsSync(runeNodeModules) && fs.statSync(runeNodeModules).isDirectory()) {
      needsLink = false;
    } else {
      fs.rmSync(runeNodeModules, { recursive: true, force: true });
    }
  }
} catch {
  // Does not exist yet; create below.
}

if (needsLink) {
  if (process.platform === "win32") {
    // NTFS directory junctions work on Windows without Administrator or Developer Mode.
    fs.symlinkSync(frontendNodeModules, runeNodeModules, "junction");
  } else {
    const relTarget = path.relative(runeDir, frontendNodeModules);
    fs.symlinkSync(relTarget, runeNodeModules, "dir");
  }
}
