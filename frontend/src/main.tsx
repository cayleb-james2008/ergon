import { createRoot } from "react-dom/client";
import "./index.css";
import { App } from "./App";
import { installBrowserEngine } from "virtual:ergon-boot";

async function boot() {
  await installBrowserEngine();
  createRoot(document.getElementById("root")!).render(<App />);
}

void boot();
