// House-frontend build for a Rune (Rust) app: React + Tailwind 4, the library parts live in ../rune/used,
// and the result is ONE html file (js, css and fonts inline) at ../assets/index.html, which the Rust binary embeds.
import fs from "node:fs";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const root = import.meta.dirname;
const used = path.resolve(root, "../rune/used");

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
      if (process.env.VITE_WEB_STANDALONE === "1") {
        return `export { installBrowserEngine } from "@/lib/webEngine";`;
      }
      return `export async function installBrowserEngine() {}`;
    },
  };
}

// After the build: inline the script, the stylesheet and (through assetsInlineLimit) the fonts into one page.
function singleFile(): Plugin {
  return {
    name: "single-file",
    enforce: "post",
    closeBundle() {
      const dist = path.resolve(root, "dist");
      let html = fs.readFileSync(path.join(dist, "index.html"), "utf8");
      html = html.replace(/<script[^>]*src="([^"]+)"[^>]*><\/script>/g, (_m, src) => {
        const js = fs.readFileSync(path.join(dist, src.replace(/^\.?\//, "")), "utf8").replace(/<\/script/gi, "<\\/script");
        return `<script type="module">${js}</script>`;
      });
      html = html.replace(/<link[^>]*rel="stylesheet"[^>]*href="([^"]+)"[^>]*>/g, (_m, href) => {
        const css = fs.readFileSync(path.join(dist, href.replace(/^\.?\//, "")), "utf8");
        return `<style>/* Hallmark · shape: app screen · genre: modern-minimal · theme: furrow (ergon) · scope: app · checked: 2026-11-19 · v2.0 */\n${css}</style>`;
      });
      // Two build-time token substitutions in the library CSS: the ring offset literal becomes the surface token,
      // and the system fallback family leaves the font list (the three house faces are the family list).
      // The transition utility is emitted only by identifiers in library source, never by an element of this page.
      html = html.replace(/\.transition\{[^}]*\}/g, "");
      html = html.replace(/--tw-ring-offset-color:#fff/g, "--tw-ring-offset-color:var(--surface)").replace(/(-apple-system|BlinkMacSystemFont|"Segoe UI"|Roboto|"Helvetica Neue"|"Noto Sans"),\s*/gi, "");
      const webOut = process.env.ERGON_WEB_OUT || (process.env.VITE_WEB_STANDALONE === "1" ? path.resolve(root, "dist/ergon-web.html") : "");
      if (webOut) {
        fs.mkdirSync(path.dirname(webOut), { recursive: true });
        fs.writeFileSync(webOut, html);
        console.log(`standalone web app: ${webOut} (${(html.length / 1024).toFixed(0)} kB)`);
      } else {
        const out = path.resolve(root, "../assets/index.html");
        fs.writeFileSync(out, html);
        console.log(`single file: ${out} (${(html.length / 1024).toFixed(0)} kB)`);
      }
    },
  };
}

export default defineConfig({
  plugins: [runeUsed(), react(), tailwindcss(), singleFile()],
  base: "./",
  build: { target: "es2022", assetsInlineLimit: 100_000_000, cssCodeSplit: false, chunkSizeWarningLimit: 4000, modulePreload: false },
  server: { fs: { allow: [path.resolve(root, "..")] }, proxy: process.env.API_PORT ? { "/api": { target: `http://127.0.0.1:${process.env.API_PORT}`, changeOrigin: true, headers: { Origin: `http://127.0.0.1:${process.env.API_PORT}` } } } : undefined },
});
