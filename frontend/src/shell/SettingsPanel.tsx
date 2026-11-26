// Unified Settings dialog ("Model, Workspace & Appearance") triggered from the Left Rail footer
// and strictly mounted OUTSIDE mobile Vaul Drawer.Content (z-50 over z-40):
// 1. Appearance (Light / Dark COSS SegmentedControl — moved out of main rail chrome per owner ruling)
// 2. Workspace Switcher (GET /api/people & POST /api/people/switch) + Field-Book Audit Bundle export
// 3. Live Model Settings (GET/POST /api/config + detected local gateways in native Rust mode;
//    Browser IndexedDB Engine / BYOK OpenAI-compatible in sessionStorage / Connect to Local Ergon in Web mode)
// 4. Version & Update check
import { useEffect, useState } from "react";
import { Coins, Cpu, ExternalLink, FileCheck2, Moon, Plus, SlidersHorizontal, Sun, Trash2, UserPlus, Users } from "lucide-react";
import { toast } from "sonner";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { SegmentedControl, SegmentedControlItem } from "@/components/ui/segmented-control";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/kibo-ui/pill";
import { WorkspaceSwitcherList } from "@/shell/PersonSelect";
import { SecuritySection } from "@/shell/SecuritySection";
import { VersionRow, type useUpdate } from "@/shell/Update";
import {
  call,
  isWebStandalone,
  type DetectedGateway,
  type ModelConfigPayload,
  type ModelPricingMap,
  type Person,
  type Status,
} from "@/lib/api";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board";

export type Theme = "light" | "dark";

function SettingsBody({
  theme,
  onTheme,
  status,
  people,
  activePerson,
  onRefreshPeople,
  onSwitchPerson,
  onAddPerson,
  onExportAuditBundle,
  onConfigSaved,
  u,
}: {
  theme: Theme;
  onTheme: (t: Theme) => void;
  status: Status | null | "error";
  people: Person[];
  activePerson: string;
  onRefreshPeople: () => void;
  onSwitchPerson: (id: string) => Promise<void>;
  onAddPerson: () => void;
  onExportAuditBundle: (format: "json" | "md") => Promise<void>;
  onConfigSaved: () => void;
  u: ReturnType<typeof useUpdate>;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const webMode = isWebStandalone();
  const [cfg, setCfg] = useState<ModelConfigPayload | null>(null);
  const [baseUrl, setBaseUrl] = useState("");
  const [modelId, setModelId] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [engineMode, setEngineMode] = useState<"browser" | "byok">("browser");
  const [localProbe, setLocalProbe] = useState<"idle" | "checking" | "found" | "missing">("idle");
  const [saving, setSaving] = useState(false);
  const [switchingId, setSwitchingId] = useState<string | null>(null);
  const [exportingBundle, setExportingBundle] = useState<"json" | "md" | null>(null);
  const [err, setErr] = useState("");
  const [lockedRemote, setLockedRemote] = useState(false);
  const [pricingRows, setPricingRows] = useState<{ model: string; input: string; output: string; cached: string }[]>([]);
  const [newPricingModel, setNewPricingModel] = useState("");
  const [savingPricing, setSavingPricing] = useState(false);
  const [pricingErr, setPricingErr] = useState("");

  useEffect(() => {
    if (!isOpen) return;
    setErr("");
    setPricingErr("");
    setLockedRemote(false);
    onRefreshPeople();
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
      const mp = r.data.model_pricing ?? {};
      const rows = Object.entries(mp).map(([model, entry]) => ({
        model,
        input: String(entry?.input_per_1m_usd ?? 0),
        output: String(entry?.output_per_1m_usd ?? 0),
        cached: String(entry?.cached_input_per_1m_usd ?? 0),
      }));
      setPricingRows(rows);
    });
  }, [isOpen, onRefreshPeople]);

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

  const addPricingRow = () => {
    const id = newPricingModel.trim();
    if (!id) {
      setPricingErr("Enter a model ID before adding a pricing row.");
      return;
    }
    if (pricingRows.some((row) => row.model.toLowerCase() === id.toLowerCase())) {
      setPricingErr(`Pricing for "${id}" is already listed.`);
      return;
    }
    setPricingErr("");
    setPricingRows((prev) => [...prev, { model: id, input: "0.50", output: "1.50", cached: "0.25" }]);
    setNewPricingModel("");
  };

  const savePricing = async (e: React.FormEvent) => {
    e.preventDefault();
    setPricingErr("");
    const map: ModelPricingMap = {};
    for (const row of pricingRows) {
      const m = row.model.trim();
      if (!m) continue;
      const inp = Number(row.input);
      const outp = Number(row.output);
      const cached = Number(row.cached || "0");
      if (!Number.isFinite(inp) || inp < 0 || !Number.isFinite(outp) || outp < 0 || !Number.isFinite(cached) || cached < 0) {
        setPricingErr(`Rates for "${m}" must be non-negative numbers.`);
        return;
      }
      map[m] = {
        input_per_1m_usd: inp,
        output_per_1m_usd: outp,
        cached_input_per_1m_usd: cached,
      };
    }
    setSavingPricing(true);
    const r = await call<ModelConfigPayload>("/api/config", {
      method: "POST",
      body: { model_pricing: map },
    });
    setSavingPricing(false);
    if (!r.ok) {
      setPricingErr(r.error || "Could not save model pricing overrides.");
      return;
    }
    setCfg((prev) => (prev ? { ...prev, ...r.data, model_pricing: r.data.model_pricing ?? map } : r.data));
    toast.success("Model pricing overrides saved.");
    onConfigSaved();
  };

  const probeLocalErgon = async () => {
    setLocalProbe("checking");
    for (const port of [8787, 8473]) {
      const ctrl = new AbortController();
      const timer = setTimeout(() => ctrl.abort(), 1200);
      try {
        await fetch(`http://127.0.0.1:${port}/api/version`, { mode: "no-cors", signal: ctrl.signal });
        clearTimeout(timer);
        setLocalProbe("found");
        window.open(`http://127.0.0.1:${port}/`, "_blank", "noopener");
        toast.success(`Connected: opened your local Ergon desk on 127.0.0.1:${port}.`);
        return;
      } catch {
        clearTimeout(timer);
      }
    }
    setLocalProbe("missing");
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

      {/* 2. Workspace Switcher */}
      <section
        id="settings-workspace-section"
        aria-labelledby="settings-workspace-h"
        className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
      >
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Users className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
            <h3 id="settings-workspace-h" className="text-sm font-medium text-ink">Workspace switcher</h3>
          </div>
          <Button
            type="button"
            size="xs"
            variant="outline"
            className="max-md:min-h-11"
            onClick={() => {
              setIsOpen(false);
              setTimeout(onAddPerson, 30);
            }}
          >
            <UserPlus className="size-3.5" strokeWidth={1.5} aria-hidden />
            Add workspace
          </Button>
        </div>
        <WorkspaceSwitcherList
          people={people}
          active={activePerson}
          switchingId={switchingId}
          compact
          onSwitch={async (id) => {
            if (id === activePerson) return;
            setSwitchingId(id);
            await onSwitchPerson(id);
            setSwitchingId(null);
            toast.success("Switched active workspace.");
          }}
        />
      </section>

      {/* 3. Field-Book Audit Bundle */}
      <section
        id="settings-audit-section"
        aria-labelledby="settings-audit-h"
        className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3.5"
      >
        <div className="flex items-center gap-2">
          <FileCheck2 className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          <h3 id="settings-audit-h" className="text-sm font-medium text-ink">Field-Book Audit Bundle</h3>
        </div>
        <p className="text-xs leading-5 text-ink-2">
          Export a tamper-evident audit package combining the verified SHA-256 receipt chain, every workspace file digest, routine execution history, and offline verification instructions.
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            id="settings-export-audit-json"
            type="button"
            size="sm"
            variant="outline"
            className="max-md:min-h-11"
            disabled={exportingBundle !== null}
            onClick={async () => {
              setExportingBundle("json");
              await onExportAuditBundle("json");
              setExportingBundle(null);
            }}
          >
            <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
            {exportingBundle === "json" ? "Exporting…" : "Export Full Audit Bundle (.json)"}
          </Button>
          <Button
            id="settings-export-audit-md"
            type="button"
            size="sm"
            variant="outline"
            className="max-md:min-h-11"
            disabled={exportingBundle !== null}
            onClick={async () => {
              setExportingBundle("md");
              await onExportAuditBundle("md");
              setExportingBundle(null);
            }}
          >
            <FileCheck2 className="size-3.5" strokeWidth={1.5} aria-hidden />
            {exportingBundle === "md" ? "Exporting…" : "Audit Report (.md)"}
          </Button>
        </div>
      </section>

      {/* Security tripwires (native v0.8.0+) */}
      <SecuritySection status={status} />

      {/* 4. Model Provider & Runtime */}
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
                  All 12 deterministic workspace workflows (ledger reconciliation, meeting summaries, contract audits, inbox triage, dataset cleaning, document diffs, table extraction, and memory rules), scheduled routines, file version history, and SHA-256 audit bundles run locally over your IndexedDB workspace.
                </p>
                <div className="flex flex-wrap items-center gap-2">
                  <Button type="submit" size="sm" variant="outline" className="h-auto max-w-full whitespace-normal py-2 text-left max-md:min-h-11" disabled={saving}>
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
                  className="h-auto max-w-full whitespace-normal py-2 text-left max-md:min-h-11"
                  disabled={localProbe === "checking"}
                  onClick={probeLocalErgon}
                >
                  <ExternalLink className="size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
                  <span>{localProbe === "checking" ? "Checking :8787…" : "Connect to Local Ergon (127.0.0.1:8787)"}</span>
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

      {/* 5. Model Pricing Overrides */}
      {!lockedRemote && (
        <section
          id="settings-pricing-section"
          aria-labelledby="settings-pricing-h"
          className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <Coins className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
              <h3 id="settings-pricing-h" className="text-sm font-medium text-ink">Model pricing overrides (USD per 1M tokens)</h3>
            </div>
          </div>
          <p className="text-xs leading-5 text-ink-2">
            Set input, output, and cached token rates per model ID for retainer burn and billable ROI metering.
          </p>

          <form noValidate onSubmit={savePricing} className="flex flex-col gap-2.5">
            {pricingRows.length === 0 ? (
              <p className="rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-ink-2">
                No custom pricing overrides yet. Add a model ID below to customize rates.
              </p>
            ) : (
              <div className="flex flex-col gap-2">
                {pricingRows.map((row, idx) => (
                  <div
                    key={row.model}
                    data-pricing-model={row.model}
                    className="flex flex-col gap-2 rounded-sm border border-hairline bg-surface p-2.5"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="mono truncate text-xs font-medium text-ink">{row.model}</span>
                      <Button
                        type="button"
                        size="icon-xs"
                        variant="ghost"
                        aria-label={`Remove pricing for ${row.model}`}
                        className="max-md:min-h-11 max-md:min-w-11"
                        onClick={() => setPricingRows((prev) => prev.filter((_, i) => i !== idx))}
                      >
                        <Trash2 className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                      </Button>
                    </div>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
                      <div className="flex flex-col gap-1">
                        <label htmlFor={`price-in-${idx}`} className="text-xs text-ink-2">Input / 1M</label>
                        <Input
                          id={`price-in-${idx}`}
                          aria-label={`${row.model} input per 1M USD`}
                          type="number"
                          min="0"
                          step="any"
                          value={row.input}
                          onChange={(e) => {
                            const val = e.target.value;
                            setPricingRows((prev) => prev.map((r, i) => (i === idx ? { ...r, input: val } : r)));
                          }}
                          className="mono max-md:min-h-11"
                        />
                      </div>
                      <div className="flex flex-col gap-1">
                        <label htmlFor={`price-out-${idx}`} className="text-xs text-ink-2">Output / 1M</label>
                        <Input
                          id={`price-out-${idx}`}
                          aria-label={`${row.model} output per 1M USD`}
                          type="number"
                          min="0"
                          step="any"
                          value={row.output}
                          onChange={(e) => {
                            const val = e.target.value;
                            setPricingRows((prev) => prev.map((r, i) => (i === idx ? { ...r, output: val } : r)));
                          }}
                          className="mono max-md:min-h-11"
                        />
                      </div>
                      <div className="flex flex-col gap-1">
                        <label htmlFor={`price-cached-${idx}`} className="text-xs text-ink-2">Cached / 1M</label>
                        <Input
                          id={`price-cached-${idx}`}
                          aria-label={`${row.model} cached per 1M USD`}
                          type="number"
                          min="0"
                          step="any"
                          value={row.cached}
                          onChange={(e) => {
                            const val = e.target.value;
                            setPricingRows((prev) => prev.map((r, i) => (i === idx ? { ...r, cached: val } : r)));
                          }}
                          className="mono max-md:min-h-11"
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <div className="flex flex-wrap items-center gap-2 pt-0.5">
              <Input
                id="settings-new-pricing-model"
                aria-label="New model ID for pricing override"
                value={newPricingModel}
                onChange={(e) => setNewPricingModel(e.target.value)}
                placeholder="Model ID (for example gpt-4o)"
                className="min-w-[180px] flex-1 max-md:min-h-11"
              />
              <Button
                id="settings-add-pricing-btn"
                type="button"
                size="sm"
                variant="outline"
                className="max-md:min-h-11 gap-1"
                onClick={addPricingRow}
              >
                <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
                Add model
              </Button>
            </div>

            {pricingErr && (
              <p id="settings-pricing-error" role="alert" className="text-xs text-destructive-foreground">
                {pricingErr}
              </p>
            )}

            <div className="flex flex-wrap items-center gap-2 pt-0.5">
              <Button id="settings-save-pricing-btn" type="submit" size="sm" variant="outline" className="max-md:min-h-11" disabled={savingPricing}>
                {savingPricing ? "Saving…" : "Save pricing overrides"}
              </Button>
            </div>
          </form>
        </section>
      )}

      {/* 6. Version & Updates */}
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
  people,
  activePerson,
  onRefreshPeople,
  onSwitchPerson,
  onAddPerson,
  onExportAuditBundle,
  onConfigSaved,
  u,
  open,
  onOpenChange,
  hideTrigger,
}: {
  theme: Theme;
  onTheme: (t: Theme) => void;
  status: Status | null | "error";
  people: Person[];
  activePerson: string;
  onRefreshPeople: () => void;
  onSwitchPerson: (id: string) => Promise<void>;
  onAddPerson: () => void;
  onExportAuditBundle: (format: "json" | "md") => Promise<void>;
  onConfigSaved: () => void;
  u: ReturnType<typeof useUpdate>;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Settings"
      description="Appearance, workspaces, model configuration, security, and audit bundle export."
      className="max-w-xl"
      trigger={
        !hideTrigger ? (
          <button type="button" data-thing="settings-btn" className={TRIGGER} style={{ borderRadius: 7 }}>
            <span className="flex min-w-0 items-center gap-1.5">
              <SlidersHorizontal className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
              <span className="leading-tight">Settings</span>
            </span>
          </button>
        ) : undefined
      }
    >
      <SettingsBody
        theme={theme}
        onTheme={onTheme}
        status={status}
        people={people}
        activePerson={activePerson}
        onRefreshPeople={onRefreshPeople}
        onSwitchPerson={onSwitchPerson}
        onAddPerson={onAddPerson}
        onExportAuditBundle={onExportAuditBundle}
        onConfigSaved={onConfigSaved}
        u={u}
      />
    </PanelModal>
  );
}
