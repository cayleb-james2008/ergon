// House-frontend build for a Rune (Rust) app: React + Tailwind 4, the library parts live in ../rune/used.
// Standard builds emit a slim embedded shell at ../assets/index.html plus lazy-loaded /chunks/* assets
// embedded at compile time; VITE_WEB_STANDALONE=1 inlines everything into dist/ergon-web.html.
import fs from "node:fs";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const root = import.meta.dirname;
const used = path.resolve(root, "../rune/used");
const isStandaloneWeb = process.env.VITE_WEB_STANDALONE === "1";

// "@/x" is the app's own src/x first, then the copied library part rune/used/x.
function runeUsed(): Plugin {
  const exts = ["", ".tsx", ".ts", ".css", "/index.tsx", "/index.ts"];
  const VIRTUAL_BOOT = "\0ergon-boot";
  return {
    name: "rune-used",
    enforce: "pre",
    resolveId(id) {
      if (id === "virtual:ergon-boot") return VIRTUAL_BOOT;
      if (!id.startsWith("@/")) return null;
      for (const base of [path.resolve(root, "src"), used]) {
        for (const e of exts) {
          const p = path.join(base, id.slice(2) + e);
          if (fs.existsSync(p) && fs.statSync(p).isFile()) return p;
        }
      }
      return null;
    },
    load(id) {
      if (id !== VIRTUAL_BOOT) return null;
      if (isStandaloneWeb) {
        return `export { installBrowserEngine } from "@/lib/webEngine";`;
      }
      return `export async function installBrowserEngine() {}`;
    },
  };
}

// Inline the entry script + stylesheet into one HTML shell; copy lazy /chunks/* into ../assets/chunks/.
function singleFile(): Plugin {
  return {
    name: "single-file",
    enforce: "post",
    closeBundle() {
      const dist = path.resolve(root, "dist");
      let html = fs.readFileSync(path.join(dist, "index.html"), "utf8");
      html = html.replace(/<script[^>]*src="([^"]+)"[^>]*><\/script>/g, (_m, src) => {
        const rel = src.replace(/^\/?\.?\//, "");
        const js = fs
          .readFileSync(path.join(dist, rel), "utf8")
          .replace(/(["'])(?:\.\.?\/)+chunks\//g, "$1/chunks/")
          .replace(/<\/script/gi, "<\\/script");
        return `<script type="module">${js}</script>`;
      });
      html = html.replace(/<link[^>]*rel="stylesheet"[^>]*href="([^"]+)"[^>]*>/g, (_m, href) => {
        const rel = href.replace(/^\/?\.?\//, "");
        const css = fs.readFileSync(path.join(dist, rel), "utf8");
        return `<style>/* Hallmark · shape: app screen · genre: modern-minimal · theme: furrow (ergon) · scope: app · checked: 2026-11-19 · v2.0 */\n${css}</style>`;
      });
      // Two build-time token substitutions in the library CSS: the ring offset literal becomes the surface token,
      // and the system fallback family leaves the font list (the three house faces are the family list).
      html = html.replace(/\.transition\{[^}]*\}/g, "");
      html = html.replace(/--tw-ring-offset-color:#fff/g, "--tw-ring-offset-color:var(--surface)").replace(/(-apple-system|BlinkMacSystemFont|"Segoe UI"|Roboto|"Helvetica Neue"|"Noto Sans"),\s*/gi, "");
      const webOut = process.env.ERGON_WEB_OUT || (isStandaloneWeb ? path.resolve(root, "dist/ergon-web.html") : "");
      if (webOut) {
        fs.mkdirSync(path.dirname(webOut), { recursive: true });
        fs.writeFileSync(webOut, html);
        console.log(`standalone web app: ${webOut} (${(html.length / 1024).toFixed(0)} kB)`);
      } else {
        const out = path.resolve(root, "../assets/index.html");
        const chunksSrc = path.join(dist, "chunks");
        const chunksDest = path.resolve(root, "../assets/chunks");
        fs.rmSync(chunksDest, { recursive: true, force: true });
        fs.mkdirSync(chunksDest, { recursive: true });
        let chunkCount = 0;
        if (fs.existsSync(chunksSrc)) {
          for (const name of fs.readdirSync(chunksSrc)) {
            const srcPath = path.join(chunksSrc, name);
            if (name.endsWith(".js")) {
              const code = fs.readFileSync(srcPath, "utf8");
              if (/index-[A-Za-z0-9_-]+\.js|"\.\.\/index-|'\.\.\/index-/.test(code)) {
                throw new Error(`Chunk ${name} back-imports the inlined entry script; extract shared modules into chunks/shared.`);
              }
            }
            fs.copyFileSync(srcPath, path.join(chunksDest, name));
            chunkCount += 1;
          }
        }
        fs.writeFileSync(out, html);
        console.log(`single shell: ${out} (${(html.length / 1024).toFixed(0)} kB) + ${chunkCount} embedded chunks`);
      }
    },
  };
}

export default defineConfig({
  plugins: [runeUsed(), react(), tailwindcss(), singleFile()],
  base: isStandaloneWeb ? "./" : "/",
  build: {
    target: "es2022",
    assetsInlineLimit: isStandaloneWeb ? 100_000_000 : 0,
    cssCodeSplit: false,
    chunkSizeWarningLimit: 4000,
    modulePreload: false,
    rollupOptions: isStandaloneWeb
      ? { output: { codeSplitting: false } }
      : {
          output: {
            entryFileNames: "index-[hash].js",
            chunkFileNames: "chunks/[name]-[hash].js",
            assetFileNames: "chunks/[name]-[hash][extname]",
            manualChunks(id) {
              const normalized = id.replaceAll("\\", "/");
              if (
                normalized.includes("/node_modules/") ||
                normalized.includes("/rune/used/") ||
                normalized.includes("/src/lib/") ||
                normalized.includes("/src/components/") ||
                normalized.endsWith("/src/motion.ts") ||
                normalized.endsWith("/src/chat/Entry.tsx") ||
                normalized.endsWith("/src/chat/LiveRunTimeline.tsx") ||
                normalized.endsWith("/src/shell/PersonSelect.tsx") ||
                normalized.endsWith("/src/shell/CheckpointCard.tsx") ||
                normalized.endsWith("/src/shell/OnboardingCard.tsx") ||
                normalized.endsWith("/src/shell/Update.tsx") ||
                normalized.endsWith("/src/views/usePaper.ts")
              ) {
                return "shared";
              }
            },
          },
        },
  },
  server: { fs: { allow: [path.resolve(root, "..")] }, proxy: process.env.API_PORT ? { "/api": { target: `http://127.0.0.1:${process.env.API_PORT}`, changeOrigin: true, headers: { Origin: `http://127.0.0.1:${process.env.API_PORT}` } } } : undefined },
});
