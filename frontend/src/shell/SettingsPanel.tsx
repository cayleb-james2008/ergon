// Unified Settings dialog ("Model, Workspace & Appearance") triggered from the Left Rail footer.
// Holds:
// 1. Appearance (Light / Dark COSS SegmentedControl — moved out of main rail chrome per owner ruling)
// 2. Live Model Settings (GET/POST /api/config + detected local gateways in native Rust mode;
//    Browser IndexedDB Engine / BYOK OpenAI-compatible in sessionStorage / Connect to Local Ergon in Web mode)
// 3. Version & Update check
import { useEffect, useState } from "react";
import { Cpu, ExternalLink, Moon, SlidersHorizontal, Sun } from "lucide-react";
import { toast } from "sonner";
import {
  MorphingDialog,
  MorphingDialogClose,
  MorphingDialogContainer,
  MorphingDialogContent,
  MorphingDialogDescription,
  MorphingDialogTitle,
  MorphingDialogTrigger,
  useMorphingDialog,
} from "@/components/ui/morphing-dialog";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/kibo-ui/pill";
import { VersionRow, type useUpdate } from "@/shell/Update";
import {
  call,
  isWebStandalone,
  type DetectedGateway,
  type ModelConfigPayload,
  type Status,
} from "@/lib/api";
import { open as openPreset } from "@/motion";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board";

export type Theme = "light" | "dark";

function SettingsBody({
  theme,
  onTheme,
  status,
  onConfigSaved,
  u,
}: {
  theme: Theme;
  onTheme: (t: Theme) => void;
  status: Status | null | "error";
  onConfigSaved: () => void;
  u: ReturnType<typeof useUpdate>;
}) {
  const { isOpen } = useMorphingDialog();
  const webMode = isWebStandalone();
  const [cfg, setCfg] = useState<ModelConfigPayload | null>(null);
  const [baseUrl, setBaseUrl] = useState("");
  const [modelId, setModelId] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [engineMode, setEngineMode] = useState<"browser" | "byok">("browser");
  const [localProbe, setLocalProbe] = useState<"idle" | "checking" | "found" | "missing">("idle");
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState("");
  const [lockedRemote, setLockedRemote] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setErr("");
    setLockedRemote(false);
    call<ModelConfigPayload>("/api/config").then((r) => {
      if (!r.ok) {
        if (r.status === 403) setLockedRemote(true);
        else setErr(r.error || "Could not load model settings.");
        return;
      }
      setCfg(r.data);
      setBaseUrl(r.data.model_base_url || "");
      setModelId(r.data.model_id || "");
      setApiKey("");
      if (r.data.engine_mode === "byok") setEngineMode("byok");
      else setEngineMode("browser");
    });
  }, [isOpen]);

  const applyGateway = (gw: DetectedGateway) => {
    setBaseUrl(gw.base_url);
    setModelId(gw.model_id);
    setApiKey("");
    setErr("");
  };

  const saveModel = async (e: React.FormEvent, clearKey = false) => {
    e.preventDefault();
    setSaving(true);
    setErr("");
    const body = webMode
      ? {
          engine_mode: engineMode,
          model_base_url: baseUrl.trim(),
          model_id: modelId.trim(),
          api_key: apiKey.trim(),
          clear_api_key: clearKey,
        }
      : {
          model_base_url: baseUrl.trim(),
          model_id: modelId.trim(),
          api_key: apiKey.trim() || undefined,
          clear_api_key: clearKey,
        };
    const r = await call<ModelConfigPayload>("/api/config", { method: "POST", body });
    setSaving(false);
    if (!r.ok) {
      setErr(r.error || "Could not save model settings.");
      return;
    }
    setCfg((prev) => ({ ...(prev ?? r.data), ...r.data }));
    setApiKey("");
    toast.success("Model settings saved.");
    onConfigSaved();
  };

  const probeLocalErgon = async () => {
    setLocalProbe("checking");
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), 1600);
    try {
      await fetch("http://127.0.0.1:8787/api/version", { mode: "no-cors", signal: ctrl.signal });
      clearTimeout(timer);
      setLocalProbe("found");
      window.open("http://127.0.0.1:8787/", "_blank", "noopener");
      toast.success("Connected — opened your local Ergon desk on 127.0.0.1:8787.");
    } catch {
      clearTimeout(timer);
      setLocalProbe("missing");
    }
  };

  const gateways = cfg?.detected_gateways ?? [];
  const activeModel = status && status !== "error" ? status.model : cfg?.model_id;

  return (
    <div id="settings-panel" className="flex flex-col gap-5 pt-1">
      {/* 1. Appearance (Light / Dark SegmentedControl) */}
      <section aria-labelledby="settings-appearance-h" className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div>
            <h3 id="settings-appearance-h" className="text-sm font-medium text-ink">Appearance</h3>
            <p className="text-xs text-ink-2">Switch between Light and Dark themes for this desk.</p>
          </div>
          <SegmentedControl aria-label="Appearance">
            <SegmentedControlItem
              data-id="light"
              aria-pressed={theme === "light"}
              data-checked={theme === "light" ? "" : undefined}
              className="min-h-11 gap-1.5 px-3 md:min-h-8"
              onClick={() => onTheme("light")}
            >
              <Sun className="size-3.5" strokeWidth={1.5} aria-hidden />Light
            </SegmentedControlItem>
            <SegmentedControlItem
              data-id="dark"
              aria-pressed={theme === "dark"}
              data-checked={theme === "dark" ? "" : undefined}
              className="min-h-11 gap-1.5 px-3 md:min-h-8"
              onClick={() => onTheme("dark")}
            >
              <Moon className="size-3.5" strokeWidth={1.5} aria-hidden />Dark
            </SegmentedControlItem>
          </SegmentedControl>
        </div>
      </section>

      {/* 2. Model Provider & Runtime */}
      <section aria-labelledby="settings-model-h" className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Cpu className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
            <h3 id="settings-model-h" className="text-sm font-medium text-ink">Model connection</h3>
          </div>
          {activeModel && (
            <Pill variant="secondary" className="border border-hairline bg-surface px-2 py-0.5 text-xs text-ink">
              <span className="mono">{activeModel}</span>
            </Pill>
          )}
        </div>

        {lockedRemote ? (
          <p className="text-sm text-ink-2">
            Model settings can only be changed when opening Ergon directly on the machine running it (`127.0.0.1` or `localhost`).
          </p>
        ) : webMode ? (
          <div className="flex flex-col gap-3">
            <p className="text-xs leading-5 text-ink-2">
              Run real workspace tools directly in your browser (IndexedDB), attach your own OpenAI-compatible API key (stored in `sessionStorage` only), or open your local Rust binary.
            </p>
            <SegmentedControl aria-label="Web engine mode">
              <SegmentedControlItem
                data-id="browser"
                aria-pressed={engineMode === "browser"}
                data-checked={engineMode === "browser" ? "" : undefined}
                className="min-h-11 px-3 md:min-h-8"
                onClick={() => setEngineMode("browser")}
              >
                Browser Engine
              </SegmentedControlItem>
              <SegmentedControlItem
                data-id="byok"
                aria-pressed={engineMode === "byok"}
                data-checked={engineMode === "byok" ? "" : undefined}
                className="min-h-11 px-3 md:min-h-8"
                onClick={() => setEngineMode("byok")}
              >
                BYOK LLM
              </SegmentedControlItem>
            </SegmentedControl>

            {engineMode === "byok" ? (
              <form onSubmit={(e) => saveModel(e, false)} className="flex flex-col gap-2.5">
                <div className="flex flex-col gap-1">
                  <label htmlFor="cfg-base-url" className="text-xs font-medium text-ink">OpenAI-compatible base URL</label>
                  <Input
                    id="cfg-base-url"
                    value={baseUrl}
                    onChange={(e) => setBaseUrl(e.target.value)}
                    placeholder="https://api.openai.com/v1"
                    autoComplete="off"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="cfg-model-id" className="text-xs font-medium text-ink">Model ID</label>
                  <Input
                    id="cfg-model-id"
                    value={modelId}
                    onChange={(e) => setModelId(e.target.value)}
                    placeholder="gpt-4o-mini or deepseek-chat"
                    autoComplete="off"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="cfg-api-key" className="text-xs font-medium text-ink">
                    API key (session only{cfg?.has_api_key ? " · key saved in this tab" : ""})
                  </label>
                  <Input
                    id="cfg-api-key"
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder={cfg?.has_api_key ? "Leave blank to keep tab key" : "sk-..."}
                    autoComplete="off"
                  />
                </div>
                {err && <p role="alert" className="text-xs text-destructive-foreground">{err}</p>}
                <div className="flex flex-wrap items-center gap-2 pt-1">
                  <Button type="submit" size="sm" className="max-md:min-h-11" disabled={saving}>
                    {saving ? "Saving…" : "Use BYOK LLM"}
                  </Button>
                  {cfg?.has_api_key && (
                    <Button type="button" size="sm" variant="outline" className="max-md:min-h-11" onClick={(e) => saveModel(e, true)}>
                      Clear key
                    </Button>
                  )}
                </div>
              </form>
            ) : (
              <form onSubmit={(e) => saveModel(e, true)} className="flex flex-col gap-2">
                <p className="text-xs text-ink-2">
                  Deterministic CSV reconciliation, Markdown summarization, action extraction, learned memory rules, and approval-gated deletion run locally over your IndexedDB workspace.
                </p>
                <div className="flex flex-wrap items-center gap-2">
                  <Button type="submit" size="sm" variant="outline" className="max-md:min-h-11" disabled={saving}>
                    Active: Browser Workspace Engine
                  </Button>
                </div>
              </form>
            )}

            <div className="border-t border-hairline pt-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <span className="text-xs font-medium text-ink">Running Ergon locally?</span>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  className="max-md:min-h-11"
                  disabled={localProbe === "checking"}
                  onClick={probeLocalErgon}
                >
                  <ExternalLink className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {localProbe === "checking" ? "Checking :8787…" : "Connect to Local Ergon (127.0.0.1:8787)"}
                </Button>
              </div>
              {localProbe === "missing" && (
                <p className="mt-1.5 text-xs text-ink-2">
                  No local `ergon` binary responded on `127.0.0.1:8787`. Start `ergon` on your machine, then click again.
                </p>
              )}
            </div>
          </div>
        ) : (
          <form onSubmit={(e) => saveModel(e, false)} className="flex flex-col gap-3">
            {gateways.length > 0 && (
              <div className="flex flex-col gap-1.5">
                <span className="text-xs font-medium text-ink-2">Detected local gateways (no key needed)</span>
                <div className="flex flex-wrap gap-1.5">
                  {gateways.map((gw) => (
                    <Button
                      key={gw.base_url}
                      type="button"
                      size="sm"
                      variant={baseUrl === gw.base_url ? "default" : "outline"}
                      className="max-md:min-h-11 text-xs"
                      onClick={() => applyGateway(gw)}
                    >
                      {gw.label}
                    </Button>
                  ))}
                </div>
              </div>
            )}

            <div className="flex flex-col gap-1">
              <label htmlFor="cfg-base-url" className="text-xs font-medium text-ink">Model base URL</label>
              <Input
                id="cfg-base-url"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="http://127.0.0.1:11434/v1"
                autoComplete="off"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="cfg-model-id" className="text-xs font-medium text-ink">Model ID</label>
              <Input
                id="cfg-model-id"
                value={modelId}
                onChange={(e) => setModelId(e.target.value)}
                placeholder="qwen2.5:3b"
                autoComplete="off"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="cfg-api-key" className="text-xs font-medium text-ink">
                API key (optional on localhost{cfg?.has_api_key ? " · key saved" : ""})
              </label>
              <Input
                id="cfg-api-key"
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={cfg?.has_api_key ? "Leave blank to keep saved key" : "Leave blank for local gateway"}
                autoComplete="off"
              />
            </div>

            {err && <p role="alert" className="text-xs text-destructive-foreground">{err}</p>}

            <div className="flex flex-wrap items-center gap-2 pt-0.5">
              <Button type="submit" size="sm" className="max-md:min-h-11" disabled={saving}>
                {saving ? "Saving…" : "Save model settings"}
              </Button>
              {cfg?.has_api_key && (
                <Button type="button" size="sm" variant="outline" className="max-md:min-h-11" disabled={saving} onClick={(e) => saveModel(e, true)}>
                  Clear saved key
                </Button>
              )}
            </div>
          </form>
        )}
      </section>

      {/* 3. Version & Updates */}
      <section aria-label="Version and updates" className="rounded-card border border-hairline bg-board/40 px-2 py-2">
        <VersionRow u={u} />
      </section>
    </div>
  );
}

export function SettingsPanel({
  theme,
  onTheme,
  status,
  onConfigSaved,
  u,
  open,
  onOpenChange,
  hideTrigger,
}: {
  theme: Theme;
  onTheme: (t: Theme) => void;
  status: Status | null | "error";
  onConfigSaved: () => void;
  u: ReturnType<typeof useUpdate>;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
}) {
  return (
    <MorphingDialog transition={openPreset} open={open} onOpenChange={onOpenChange}>
      {!hideTrigger && (
        <MorphingDialogTrigger thing="settings-btn" className={TRIGGER} style={{ borderRadius: 7 }}>
          <span className="flex min-w-0 items-center gap-1.5">
            <SlidersHorizontal className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
            <span className="leading-tight">Settings</span>
          </span>
        </MorphingDialogTrigger>
      )}
      <MorphingDialogContainer>
        <MorphingDialogContent
          style={{ borderRadius: 18 }}
          className="relative mx-4 flex max-h-[84vh] w-[540px] max-w-[calc(100vw-2rem)] flex-col gap-3 overflow-y-auto border border-hairline bg-surface p-5 text-ink shadow-lift"
        >
          <MorphingDialogClose className="focus-ring absolute right-3 top-3 grid size-11 place-items-center rounded-sm text-icon outline-none hover:bg-board md:size-9" />
          <MorphingDialogTitle className="pr-10 text-title font-medium text-ink">Settings</MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">
            <SettingsBody theme={theme} onTheme={onTheme} status={status} onConfigSaved={onConfigSaved} u={u} />
          </MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}
