import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Check,
  Cloud,
  Copy,
  Cpu,
  Globe,
  KeyRound,
  Layers,
  LogOut,
  Play,
  Plug,
  Plus,
  RefreshCw,
  Search,
  ShieldCheck,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { PanelModal } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  disconnectPluginOAuth,
  dismissPluginRecommendation,
  installRegistryPlugin,
  listPluginOAuthSuites,
  listPlugins,
  runPluginAction,
  savePluginOAuthApp,
  scanPluginRecommendations,
  searchPluginRegistry,
  startPluginOAuth,
  syncPluginRegistry,
  togglePlugin,
  uninstallRegistryPlugin,
  updatePluginRegistryConfig,
  type PluginOAuthSuite,
  type PluginRecommendation,
  type PluginRegistryStatus,
  type PluginSpecItem,
} from "@/lib/api";
import { friendlyModeLabel, relTime } from "@/lib/words";

type SuiteFilter = "all" | "google" | "microsoft" | "business" | "local" | "community";
type ModeFilter = "all" | "local" | "hybrid" | "cloud";

const SUITE_TABS: Array<{ id: SuiteFilter; label: string }> = [
  { id: "all", label: "All" },
  { id: "google", label: "Google Workspace" },
  { id: "microsoft", label: "Microsoft 365" },
  { id: "business", label: "Business" },
  { id: "local", label: "On-Device" },
  { id: "community", label: "Directory" },
];

const MODE_TABS: Array<{ id: ModeFilter; label: string }> = [
  { id: "all", label: "Any mode" },
  { id: "local", label: "Local" },
  { id: "hybrid", label: "Hybrid" },
  { id: "cloud", label: "Cloud" },
];

function statusLabel(
  p: PluginSpecItem,
  simpleMode: boolean
): { text: string; tone: "ok" | "accent" | "warn" | "muted" } {
  switch (p.status) {
    case "connected_hybrid":
      return { text: simpleMode ? "Connected" : "Hybrid · Live", tone: "ok" };
    case "connected_cloud":
      return { text: simpleMode ? "Connected" : "Cloud · Live", tone: "ok" };
    case "active_local":
      return { text: simpleMode ? "Works offline" : "Local · Ready", tone: "accent" };
    case "enabled_needs_auth":
      return { text: simpleMode ? "Sign in needed" : "Needs OAuth2", tone: "warn" };
    default:
      return { text: "Available", tone: "muted" };
  }
}

export function PluginsPanel({
  open,
  onOpenChange,
  canManageConnectors,
  uiMode = "simple",
  focusPluginId = null,
  onFocusHandled,
  onOpenConnectors,
  onChanged,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  canManageConnectors: boolean;
  uiMode?: "simple" | "advanced";
  focusPluginId?: string | null;
  onFocusHandled?: () => void;
  onOpenConnectors: () => void;
  onChanged?: (enabledCount: number, recommendedCount?: number) => void;
}) {
  const simpleMode = uiMode === "simple";
  const [plugins, setPlugins] = useState<PluginSpecItem[]>([]);
  const [recommendations, setRecommendations] = useState<PluginRecommendation[]>([]);
  const [scanningRecs, setScanningRecs] = useState(false);
  const [inlineRecSetupId, setInlineRecSetupId] = useState<string | null>(null);
  const [registry, setRegistry] = useState<PluginRegistryStatus | null>(null);
  const [suites, setSuites] = useState<PluginOAuthSuite[]>([]);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [searchingLive, setSearchingLive] = useState(false);
  const [remoteResults, setRemoteResults] = useState<PluginSpecItem[]>([]);
  const [liveSearchQuery, setLiveSearchQuery] = useState("");
  const [liveSearchError, setLiveSearchError] = useState("");
  const [showSources, setShowSources] = useState(false);
  const [customSourceUrl, setCustomSourceUrl] = useState("");
  const [customSourceName, setCustomSourceName] = useState("");
  const [savingSource, setSavingSource] = useState(false);

  const [err, setErr] = useState("");
  const [suite, setSuite] = useState<SuiteFilter>("all");
  const [modeFilter, setModeFilter] = useState<ModeFilter>("all");
  const [query, setQuery] = useState("");
  const [busyId, setBusyId] = useState<string | null>(null);
  const [oauthBusy, setOauthBusy] = useState<string | null>(null);
  const [editingSuite, setEditingSuite] = useState<"google" | "microsoft" | null>(null);
  const [clientIdDraft, setClientIdDraft] = useState("");
  const [clientSecretDraft, setClientSecretDraft] = useState("");
  const [tenantDraft, setTenantDraft] = useState("common");
  const [copiedUri, setCopiedUri] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<Record<string, string>>({});

  const [installTarget, setInstallTarget] = useState<PluginSpecItem | null>(null);
  const [credDrafts, setCredDrafts] = useState<Record<string, string>>({});

  const refresh = useCallback(async () => {
    setLoading(true);
    setErr("");
    try {
      const [r, oauthRes] = await Promise.all([listPlugins(), listPluginOAuthSuites()]);
      if (!r.ok) {
        setErr(r.error || "Could not load your apps library.");
        return;
      }
      setPlugins(r.data.plugins);
      const recs = Array.isArray(r.data.recommendations) ? r.data.recommendations : [];
      setRecommendations(recs);
      if (r.data.registry) setRegistry(r.data.registry);
      onChanged?.(r.data.summary.enabled_count, recs.length);
      if (oauthRes.ok) {
        setSuites(oauthRes.data.suites);
      }
    } finally {
      setLoading(false);
    }
  }, [onChanged]);

  useEffect(() => {
    if (open) void refresh();
  }, [open, refresh]);

  useEffect(() => {
    if (!open || !focusPluginId || plugins.length === 0) return;
    const match = plugins.find((p) => p.id === focusPluginId);
    if (match) {
      setSuite("all");
      setQuery("");
      if (match.install_spec && !match.installed) {
        setInstallTarget(match);
        setCredDrafts({});
      }
      requestAnimationFrame(() => {
        const el = document.querySelector<HTMLElement>(
          `#plugins-panel [data-plugin-id="${match.id}"]`
        );
        el?.scrollIntoView({ behavior: "smooth", block: "center" });
      });
    }
    onFocusHandled?.();
  }, [open, focusPluginId, plugins, onFocusHandled]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return plugins.filter((p) => {
      if (suite !== "all" && p.suite !== suite) return false;
      if (!simpleMode && modeFilter !== "all" && p.mode !== modeFilter) return false;
      if (!q) return true;
      return (
        p.name.toLowerCase().includes(q) ||
        p.tagline.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.suite_label.toLowerCase().includes(q)
      );
    });
  }, [plugins, suite, modeFilter, query, simpleMode]);

  const summary = useMemo(() => {
    const enabledCount = plugins.filter((p) => p.enabled).length;
    const localCount = plugins.filter((p) => p.local_ready).length;
    const cloudCount = plugins.filter((p) => p.cloud_connected).length;
    return { total: plugins.length, enabledCount, localCount, cloudCount };
  }, [plugins]);

  const suiteByProvider = useMemo(() => {
    const map: Partial<Record<"google" | "microsoft", PluginOAuthSuite>> = {};
    for (const s of suites) {
      map[s.provider] = s;
    }
    return map;
  }, [suites]);

  const handleSyncNow = async () => {
    setSyncing(true);
    try {
      const r = await syncPluginRegistry();
      if (!r.ok) {
        toast.error(r.error || "Could not sync the app directory.");
        return;
      }
      setRegistry(r.data);
      await refresh();
      toast.success(
        `App library updated (${r.data.curated_count + r.data.community_count} apps available).`
      );
    } finally {
      setSyncing(false);
    }
  };

  const handleSearchLiveDirectory = async (searchText?: string) => {
    const q = (searchText ?? query).trim();
    if (!q) {
      toast("Type an app name to search the live directory.");
      return;
    }
    setSearchingLive(true);
    setLiveSearchError("");
    setLiveSearchQuery(q);
    try {
      const r = await searchPluginRegistry(q);
      if (!r.ok) {
        setLiveSearchError(r.error || "Live directory search failed.");
        return;
      }
      const existingIds = new Set(plugins.map((p) => p.id));
      const newOnes = r.data.results.filter((item) => !existingIds.has(item.id));
      setRemoteResults(newOnes);
      if (r.data.live_error) setLiveSearchError(r.data.live_error);
      if (newOnes.length === 0 && !r.data.live_error) {
        toast(`All matching apps for "${q}" are already shown above.`);
      }
    } finally {
      setSearchingLive(false);
    }
  };

  const openInstallFor = (p: PluginSpecItem) => {
    setInstallTarget(p);
    setCredDrafts({});
  };

  const handleInstallSubmit = async (p: PluginSpecItem) => {
    setBusyId(p.id);
    try {
      const r = await installRegistryPlugin({
        plugin_id: p.id,
        spec: p.source === "community_registry" ? p : undefined,
        credentials: credDrafts,
      });
      if (!r.ok) {
        toast.error(r.error || `Could not connect ${p.name}.`);
        return;
      }
      setInstallTarget(null);
      setInlineRecSetupId(null);
      setCredDrafts({});
      setRemoteResults((prev) => prev.filter((x) => x.id !== p.id));
      await refresh();
      toast.success(`Connected ${p.name} for your agent.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleRescanRecommendations = async () => {
    setScanningRecs(true);
    try {
      const r = await scanPluginRecommendations();
      if (!r.ok) {
        toast.error(r.error || "Could not scan for apps.");
        return;
      }
      setRecommendations(r.data.recommendations);
      const enabledCount = plugins.filter((p) => p.enabled).length;
      onChanged?.(enabledCount, r.data.recommendations.length);
      toast.success(
        r.data.recommendations.length > 0
          ? `Found ${r.data.recommendations.length} suggested ${r.data.recommendations.length === 1 ? "app" : "apps"} from your work.`
          : "All detected apps are already set up."
      );
    } finally {
      setScanningRecs(false);
    }
  };

  const handleDismissRecommendation = async (rec: PluginRecommendation) => {
    setBusyId(rec.plugin_id);
    try {
      const r = await dismissPluginRecommendation(rec.plugin_id);
      if (!r.ok) {
        toast.error(r.error || "Could not dismiss suggestion.");
        return;
      }
      setRecommendations(r.data.recommendations);
      if (inlineRecSetupId === rec.plugin_id) setInlineRecSetupId(null);
      const enabledCount = plugins.filter((p) => p.enabled).length;
      onChanged?.(enabledCount, r.data.recommendations.length);
      toast(`Dismissed ${rec.name} suggestion.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleRecommendationSetup = async (rec: PluginRecommendation) => {
    const match = plugins.find((p) => p.id === rec.plugin_id);
    if (rec.setup_kind === "oauth_google" || rec.setup_kind === "oauth_microsoft") {
      await handleOAuthConnect(rec.setup_kind === "oauth_google" ? "google" : "microsoft");
      return;
    }
    if (rec.setup_kind === "oauth_google_setup" || rec.setup_kind === "oauth_microsoft_setup") {
      const prov = rec.setup_kind === "oauth_google_setup" ? "google" : "microsoft";
      const s = suiteByProvider[prov];
      if (canManageConnectors && s) {
        openOAuthEditor(s);
        toast(`Enter your company ${s.label} sign-in ID below.`);
      } else {
        toast("Ask your workspace admin to configure single sign-in in Connected Apps.");
      }
      return;
    }
    if (rec.setup_kind === "one_click_connect" && match) {
      const fields = match.install_spec?.credential_fields ?? [];
      if (fields.length > 0) {
        setInlineRecSetupId(match.id);
        setCredDrafts({});
        return;
      }
      await handleInstallSubmit(match);
      return;
    }
    if (rec.setup_kind === "enable_local" && match) {
      await handleToggle(match);
      return;
    }
    if (canManageConnectors && rec.setup_kind === "connector") {
      onOpenChange(false);
      onOpenConnectors();
      return;
    }
    if (match) {
      await handleToggle(match);
    }
  };

  const handleUninstall = async (p: PluginSpecItem) => {
    setBusyId(p.id);
    try {
      const r = await uninstallRegistryPlugin(p.id);
      if (!r.ok) {
        toast.error(r.error || `Could not remove ${p.name}.`);
        return;
      }
      await refresh();
      toast.success(`Disconnected ${p.name}.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleToggle = async (p: PluginSpecItem) => {
    setBusyId(p.id);
    try {
      const r = await togglePlugin(p.id, { enabled: !p.enabled });
      if (!r.ok) {
        toast.error(r.error || `Could not update ${p.name}.`);
        return;
      }
      await refresh();
      toast.success(!p.enabled ? `Turned on ${p.name} for your agent.` : `Paused ${p.name}.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleModeChange = async (p: PluginSpecItem, nextMode: "auto" | "local" | "cloud") => {
    setBusyId(p.id);
    try {
      const r = await togglePlugin(p.id, { preferred_mode: nextMode });
      if (!r.ok) {
        toast.error(r.error || `Could not set mode for ${p.name}.`);
        return;
      }
      await refresh();
      toast.success(`${p.name} set to ${nextMode} mode.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleQuickTest = async (p: PluginSpecItem) => {
    const readAction = p.actions.find((a) => !a.requires_approval && a.access === "read") ?? p.actions[0];
    if (!readAction) return;
    setBusyId(p.id);
    try {
      const preferred =
        p.cloud_connected && p.preferred_mode !== "local"
          ? "cloud"
          : p.local_ready && !p.cloud_connected
            ? "local"
            : undefined;
      const r = await runPluginAction(p.id, readAction.name, {
        mode: preferred,
      });
      if (!r.ok) {
        toast.error(r.error || `${p.name} returned an error.`);
        return;
      }
      setLastResult((prev) => ({ ...prev, [p.id]: r.data.output }));
      toast.success(`${p.name} ran cleanly.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleOAuthConnect = async (provider: "google" | "microsoft") => {
    setOauthBusy(provider);
    try {
      const r = await startPluginOAuth(provider, "/");
      if (!r.ok) {
        toast.error(r.error || `Could not start ${provider} sign-in.`);
        return;
      }
      if (r.data.demo_connected) {
        await refresh();
        toast.success(
          provider === "google"
            ? "Connected Google Workspace to your agent."
            : "Connected Microsoft 365 & Outlook to your agent."
        );
        return;
      }
      if (r.data.authorization_url) {
        window.location.assign(r.data.authorization_url);
      }
    } finally {
      setOauthBusy(null);
    }
  };

  const handleOAuthDisconnect = async (provider: "google" | "microsoft") => {
    setOauthBusy(provider);
    try {
      const r = await disconnectPluginOAuth(provider);
      if (!r.ok) {
        toast.error(r.error || "Could not disconnect account.");
        return;
      }
      await refresh();
      toast.success(
        provider === "google" ? "Disconnected Google Workspace." : "Disconnected Microsoft 365 & Outlook."
      );
    } finally {
      setOauthBusy(null);
    }
  };

  const openOAuthEditor = (s: PluginOAuthSuite) => {
    setEditingSuite(s.provider);
    setClientIdDraft(s.client_id || "");
    setClientSecretDraft("");
    setTenantDraft(s.tenant || "common");
  };

  const handleSaveOAuthApp = async (provider: "google" | "microsoft") => {
    if (!clientIdDraft.trim()) {
      toast.error("Client ID is required.");
      return;
    }
    setOauthBusy(provider);
    try {
      const r = await savePluginOAuthApp({
        provider,
        client_id: clientIdDraft.trim(),
        ...(clientSecretDraft.trim() ? { client_secret: clientSecretDraft.trim() } : {}),
        ...(provider === "microsoft" ? { tenant: tenantDraft.trim() || "common" } : {}),
      });
      if (!r.ok) {
        toast.error(r.error || "Could not save sign-in app settings.");
        return;
      }
      setEditingSuite(null);
      setClientSecretDraft("");
      await refresh();
      toast.success("Saved securely. Employees can now sign in with one click.");
    } finally {
      setOauthBusy(null);
    }
  };

  const handleSetAutoUpdate = async (mode: "auto" | "manual" | "off") => {
    setSavingSource(true);
    try {
      const r = await updatePluginRegistryConfig({ auto_update: mode });
      if (!r.ok) {
        toast.error(r.error || "Could not update catalog settings.");
        return;
      }
      setRegistry(r.data);
      toast.success(`Catalog auto-updates set to ${mode}.`);
    } finally {
      setSavingSource(false);
    }
  };

  const handleAddSource = async () => {
    if (!customSourceUrl.trim()) {
      toast.error("Enter a catalog URL to add.");
      return;
    }
    setSavingSource(true);
    try {
      const r = await updatePluginRegistryConfig({
        add_source: {
          name: customSourceName.trim() || "Team Catalog",
          kind: "ergon_manifest",
          url: customSourceUrl.trim(),
          enabled: true,
        },
      });
      if (!r.ok) {
        toast.error(r.error || "Could not add catalog source.");
        return;
      }
      setRegistry(r.data);
      setCustomSourceUrl("");
      setCustomSourceName("");
      toast.success("Added catalog source.");
    } finally {
      setSavingSource(false);
    }
  };

  const copyRedirectUri = (uri: string, provider: string) => {
    void navigator.clipboard?.writeText(uri);
    setCopiedUri(provider);
    setTimeout(() => setCopiedUri((cur) => (cur === provider ? null : cur)), 1600);
  };

  const lastSyncLabel =
    registry && registry.last_sync_ms > 0 ? relTime(registry.last_sync_ms) : "not synced yet";

  const renderAppCard = (p: PluginSpecItem, isRemoteCandidate = false) => {
    const st = statusLabel(p, simpleMode);
    const isBusy = busyId === p.id;
    const safeRead = p.actions.find((a) => !a.requires_approval && a.access === "read");
    const oauthSuite = p.oauth_provider ? suiteByProvider[p.oauth_provider] : undefined;
    const fields = p.install_spec?.credential_fields ?? [];
    const hasInstallSpec = Boolean(p.install_spec);
    const isConfiguring = installTarget?.id === p.id;
    const modeText = simpleMode ? friendlyModeLabel(p.mode) : p.mode;

    return (
      <article
        key={p.id}
        data-plugin-id={p.id}
        className="flex min-w-0 flex-col justify-between gap-2.5 rounded-card border border-hairline bg-surface p-3"
      >
        <div className="flex flex-col gap-1.5">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-1.5">
                <span className="text-sm font-semibold text-ink">{p.name}</span>
                {modeText && (
                  <Badge variant="outline" size="sm" className="gap-1">
                    {p.mode === "local" ? (
                      <Cpu className="size-3" strokeWidth={1.5} aria-hidden />
                    ) : p.mode === "cloud" ? (
                      <Cloud className="size-3" strokeWidth={1.5} aria-hidden />
                    ) : (
                      <Layers className="size-3" strokeWidth={1.5} aria-hidden />
                    )}
                    <span>{modeText}</span>
                  </Badge>
                )}
              </div>
              <p className="text-xs text-ink-3">{p.suite_label}</p>
            </div>
            <Badge
              variant={st.tone === "ok" ? "success" : st.tone === "warn" ? "warning" : "secondary"}
              size="sm"
            >
              {st.text}
            </Badge>
          </div>

          <p className="text-xs text-ink-2">{p.description}</p>

          {p.connected_via && (
            <p className="mono truncate text-xs text-accent">
              Connected: {p.connected_via}
            </p>
          )}

          <div className="flex flex-wrap gap-1 pt-0.5">
            {p.actions.slice(0, simpleMode ? 3 : 6).map((a) => (
              <span
                key={a.name}
                title={a.description}
                className="inline-flex items-center gap-1 rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-xs text-ink-2"
              >
                {a.requires_approval && (
                  <ShieldCheck className="size-3 text-warn" strokeWidth={1.5} aria-hidden />
                )}
                <span>{a.label}</span>
              </span>
            ))}
          </div>

          {lastResult[p.id] && (
            <p className="mono line-clamp-2 rounded-sm bg-board px-2 py-1 text-xs text-ink-2">
              {lastResult[p.id]}
            </p>
          )}

          {isConfiguring && (
            <div className="mt-1 flex flex-col gap-2 rounded-sm border border-hairline bg-board/50 p-2.5">
              <p className="text-xs font-medium text-ink">
                Connect {p.name}
              </p>
              {fields.length === 0 ? (
                <p className="text-xs text-ink-2">No API key required — click below to add to your agent.</p>
              ) : (
                fields.map((f) => (
                  <div key={f.key} className="flex flex-col gap-1">
                    <label htmlFor={`cred-${p.id}-${f.key}`} className="text-xs font-medium text-ink">
                      {f.label}{f.is_required === false ? " (optional)" : ""}
                    </label>
                    <Input
                      id={`cred-${p.id}-${f.key}`}
                      type={f.is_secret !== false ? "password" : "text"}
                      value={credDrafts[f.key] ?? ""}
                      onChange={(e) =>
                        setCredDrafts((prev) => ({ ...prev, [f.key]: e.target.value }))
                      }
                      placeholder={f.description || `Enter ${f.label}`}
                      className="h-9 text-xs max-md:min-h-11"
                    />
                  </div>
                ))
              )}
              <div className="flex items-center justify-end gap-1.5 pt-1">
                <Button
                  type="button"
                  size="xs"
                  variant="ghost"
                  className="max-md:min-h-11"
                  onClick={() => {
                    setInstallTarget(null);
                    setCredDrafts({});
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="default"
                  disabled={isBusy}
                  className="max-md:min-h-11 gap-1"
                  onClick={() => void handleInstallSubmit(p)}
                >
                  <Check className="size-3" strokeWidth={1.75} aria-hidden />
                  <span>{isBusy ? "Connecting…" : "Connect securely"}</span>
                </Button>
              </div>
            </div>
          )}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-1.5 border-t border-hairline pt-2">
          <div className="flex flex-wrap items-center gap-1">
            {isRemoteCandidate ? (
              <Button
                type="button"
                size="xs"
                variant="default"
                disabled={isBusy}
                onClick={() => (fields.length > 0 ? openInstallFor(p) : void handleInstallSubmit(p))}
                className="max-md:min-h-11 gap-1"
              >
                <Plus className="size-3" strokeWidth={1.75} aria-hidden />
                <span>Add app</span>
              </Button>
            ) : (
              <Button
                type="button"
                size="xs"
                variant={p.enabled ? "secondary" : "default"}
                disabled={isBusy}
                onClick={() => void handleToggle(p)}
                className="max-md:min-h-11"
              >
                {p.enabled ? "Active" : "Turn on"}
              </Button>
            )}
            {!simpleMode && !isRemoteCandidate && p.mode === "hybrid" && p.local_ready && (
              <div className="inline-flex rounded-sm border border-hairline bg-board/50 p-0.5">
                {(["auto", "local", "cloud"] as const).map((m) => (
                  <button
                    key={m}
                    type="button"
                    disabled={isBusy}
                    onClick={() => void handleModeChange(p, m)}
                    className={`rounded-sm px-2 py-0.5 text-xs ${
                      p.preferred_mode === m
                        ? "bg-surface font-medium text-ink"
                        : "text-ink-2 hover:text-ink"
                    }`}
                  >
                    {m}
                  </button>
                ))}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-center gap-1">
            {!isRemoteCandidate && (p.local_ready || p.cloud_connected) && safeRead && (
              <Button
                type="button"
                size="xs"
                variant="outline"
                disabled={isBusy}
                onClick={() => void handleQuickTest(p)}
                className="max-md:min-h-11 gap-1"
              >
                <Play className="size-3" strokeWidth={1.5} aria-hidden />
                <span>Try</span>
              </Button>
            )}
            {oauthSuite && !oauthSuite.connected && oauthSuite.configured && (
              <Button
                type="button"
                size="xs"
                variant="outline"
                disabled={oauthBusy === oauthSuite.provider}
                onClick={() => void handleOAuthConnect(oauthSuite.provider)}
                className="max-md:min-h-11 gap-1"
              >
                <KeyRound className="size-3" strokeWidth={1.5} aria-hidden />
                <span>Sign in</span>
              </Button>
            )}
            {!isRemoteCandidate && hasInstallSpec && !p.installed && !isConfiguring && (
              <Button
                type="button"
                size="xs"
                variant="outline"
                disabled={isBusy}
                onClick={() => (fields.length > 0 ? openInstallFor(p) : void handleInstallSubmit(p))}
                className="max-md:min-h-11 gap-1"
              >
                <Plug className="size-3" strokeWidth={1.5} aria-hidden />
                <span>Connect</span>
              </Button>
            )}
            {p.installed && (
              <Button
                type="button"
                size="xs"
                variant="ghost"
                disabled={isBusy}
                onClick={() => void handleUninstall(p)}
                className="max-md:min-h-11 gap-1 text-ink-2 hover:text-ink"
              >
                <Trash2 className="size-3" strokeWidth={1.5} aria-hidden />
                <span>Remove</span>
              </Button>
            )}
            {!isRemoteCandidate &&
              !hasInstallSpec &&
              p.connect_preset &&
              !p.cloud_connected &&
              (!oauthSuite || !oauthSuite.configured) &&
              canManageConnectors &&
              !simpleMode && (
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  onClick={() => {
                    onOpenChange(false);
                    onOpenConnectors();
                  }}
                  className="max-md:min-h-11 gap-1"
                >
                  <Plug className="size-3" strokeWidth={1.5} aria-hidden />
                  <span>Connect</span>
                </Button>
              )}
          </div>
        </div>
      </article>
    );
  };

  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Connected Apps"
      description="Turn on apps for your assistant or search the live app directory."
      className="max-w-3xl"
    >
      <div id="plugins-panel" className="flex min-w-0 flex-col gap-4">
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-board/50 px-3 py-2.5">
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-ink">
              {summary.enabledCount} active of {summary.total} apps
            </p>
            <p className="text-xs text-ink-2">
              Auto-updates {registry?.auto_update === "off" ? "off" : "on"} · synced {lastSyncLabel}. Sending or posting still asks your OK first.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-1.5">
            <Button
              id="plugins-sync-now-btn"
              type="button"
              size="xs"
              variant="outline"
              disabled={syncing}
              onClick={() => void handleSyncNow()}
              className="max-md:min-h-11 gap-1.5"
            >
              <RefreshCw
                className={syncing ? "size-3.5 motion-safe:animate-spin" : "size-3.5"}
                strokeWidth={1.5}
                aria-hidden
              />
              <span>{syncing ? "Syncing…" : "Sync now"}</span>
            </Button>
            {canManageConnectors && (
              <Button
                id="plugins-registry-sources-toggle"
                type="button"
                size="xs"
                variant={showSources ? "secondary" : "ghost"}
                onClick={() => setShowSources((v) => !v)}
                className="max-md:min-h-11"
              >
                Sources
              </Button>
            )}
            {canManageConnectors && !simpleMode && (
              <Button
                type="button"
                size="xs"
                variant="outline"
                onClick={() => {
                  onOpenChange(false);
                  onOpenConnectors();
                }}
                className="max-md:min-h-11 shrink-0 gap-1.5"
              >
                <Plug className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                <span>Connectors</span>
              </Button>
            )}
          </div>
        </div>

        {showSources && registry && (
          <div
            id="plugins-registry-sources"
            className="flex flex-col gap-2.5 rounded-card border border-hairline bg-board/40 p-3 text-xs"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <span className="font-medium text-ink">Auto-update schedule</span>
              <div className="flex items-center gap-1">
                {(["auto", "manual", "off"] as const).map((m) => (
                  <Button
                    key={m}
                    type="button"
                    size="xs"
                    disabled={savingSource}
                    variant={registry.auto_update === m ? "default" : "outline"}
                    onClick={() => void handleSetAutoUpdate(m)}
                    className="max-md:min-h-11 capitalize"
                  >
                    {m}
                  </Button>
                ))}
              </div>
            </div>
            <ul className="flex flex-col gap-1.5">
              {registry.sources.map((src) => (
                <li
                  key={src.id}
                  className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-2.5 py-1.5"
                >
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-medium text-ink">{src.name}</p>
                    <p className="mono truncate text-xs text-ink-3">{src.url}</p>
                  </div>
                  <Badge variant={src.status === "error" ? "warning" : "secondary"} size="sm">
                    {src.status === "error" ? "Offline" : `${src.item_count ?? 0} apps`}
                  </Badge>
                </li>
              ))}
            </ul>
            <div className="flex flex-wrap items-center gap-1.5">
              <Input
                value={customSourceName}
                onChange={(e) => setCustomSourceName(e.target.value)}
                placeholder="Catalog name (optional)"
                aria-label="Custom catalog name"
                className="h-9 flex-1 text-xs sm:max-w-[160px]"
              />
              <Input
                value={customSourceUrl}
                onChange={(e) => setCustomSourceUrl(e.target.value)}
                placeholder="https://example.com/plugins.json"
                aria-label="Custom catalog URL"
                className="h-9 flex-1 text-xs"
              />
              <Button
                type="button"
                size="xs"
                variant="outline"
                disabled={savingSource}
                onClick={() => void handleAddSource()}
                className="max-md:min-h-11"
              >
                Add feed
              </Button>
            </div>
          </div>
        )}

        {recommendations.length > 0 && (
          <section
            id="plugins-recommendations"
            aria-label="Suggested app setups"
            className="flex flex-col gap-2.5 rounded-card border border-hairline bg-blue-soft/60 p-3"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-1.5">
                <Sparkles className="size-4 text-accent" strokeWidth={1.5} aria-hidden />
                <h3 className="text-sm font-semibold text-ink">
                  Suggested from your work
                </h3>
                <Badge variant="secondary" size="sm">
                  {recommendations.length}
                </Badge>
              </div>
              <Button
                id="plugins-rescan-btn"
                type="button"
                size="xs"
                variant="ghost"
                disabled={scanningRecs}
                onClick={() => void handleRescanRecommendations()}
                className="max-md:min-h-11 gap-1"
              >
                <RefreshCw
                  className={scanningRecs ? "size-3 motion-safe:animate-spin" : "size-3"}
                  strokeWidth={1.5}
                  aria-hidden
                />
                <span>{scanningRecs ? "Scanning…" : "Rescan"}</span>
              </Button>
            </div>
            <div className="grid min-w-0 gap-2.5 md:grid-cols-2">
              {recommendations.map((rec) => {
                const matchPlugin = plugins.find((p) => p.id === rec.plugin_id);
                const fields = matchPlugin?.install_spec?.credential_fields ?? [];
                const inlineOpen = inlineRecSetupId === rec.plugin_id && matchPlugin;
                const isBusy = busyId === rec.plugin_id;
                return (
                  <article
                    key={rec.plugin_id}
                    data-recommended-plugin={rec.plugin_id}
                    className="flex min-w-0 flex-col justify-between gap-2 rounded-card border border-hairline bg-surface p-3"
                  >
                    <div className="flex flex-col gap-1">
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0">
                          <p className="truncate text-sm font-semibold text-ink">{rec.name}</p>
                          <p className="truncate text-xs text-ink-3">{rec.suite_label}</p>
                        </div>
                        <Badge
                          variant={rec.confidence === "high" ? "info" : "secondary"}
                          size="sm"
                        >
                          {rec.confidence === "high" ? "Detected" : "Suggested"}
                        </Badge>
                      </div>
                      <p className="text-xs leading-5 text-ink-2">{rec.reason_summary}</p>
                      {rec.setup_note && (
                        <p className="text-xs text-ink-3">{rec.setup_note}</p>
                      )}
                      {inlineOpen && (
                        <div className="mt-1 flex flex-col gap-2 rounded-sm border border-hairline bg-board/50 p-2.5">
                          {fields.map((f) => (
                            <div key={f.key} className="flex flex-col gap-1">
                              <label
                                htmlFor={`rec-cred-${rec.plugin_id}-${f.key}`}
                                className="text-xs font-medium text-ink"
                              >
                                {f.label}
                                {f.is_required === false ? " (optional)" : ""}
                              </label>
                              <Input
                                id={`rec-cred-${rec.plugin_id}-${f.key}`}
                                type={f.is_secret !== false ? "password" : "text"}
                                value={credDrafts[f.key] ?? ""}
                                onChange={(e) =>
                                  setCredDrafts((prev) => ({ ...prev, [f.key]: e.target.value }))
                                }
                                placeholder={f.description || `Enter ${f.label}`}
                                className="h-9 text-xs max-md:min-h-11"
                              />
                            </div>
                          ))}
                          <div className="flex items-center justify-end gap-1.5 pt-0.5">
                            <Button
                              type="button"
                              size="xs"
                              variant="ghost"
                              className="max-md:min-h-11"
                              onClick={() => {
                                setInlineRecSetupId(null);
                                setCredDrafts({});
                              }}
                            >
                              Cancel
                            </Button>
                            <Button
                              type="button"
                              size="xs"
                              variant="default"
                              disabled={isBusy}
                              className="max-md:min-h-11 gap-1"
                              onClick={() => void handleInstallSubmit(matchPlugin)}
                            >
                              <Check className="size-3" strokeWidth={1.75} aria-hidden />
                              <span>{isBusy ? "Connecting…" : "Connect securely"}</span>
                            </Button>
                          </div>
                        </div>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center justify-between gap-1.5 border-t border-hairline pt-2">
                      {!inlineOpen ? (
                        <Button
                          type="button"
                          size="xs"
                          variant="default"
                          disabled={isBusy}
                          data-setup-recommended={rec.plugin_id}
                          onClick={() => void handleRecommendationSetup(rec)}
                          className="max-md:min-h-11 gap-1"
                        >
                          <Plug className="size-3" strokeWidth={1.5} aria-hidden />
                          <span>{rec.setup_label}</span>
                        </Button>
                      ) : (
                        <span className="text-xs text-ink-2">Ready to connect</span>
                      )}
                      <Button
                        type="button"
                        size="xs"
                        variant="ghost"
                        disabled={isBusy}
                        data-dismiss-recommended={rec.plugin_id}
                        aria-label={`Dismiss ${rec.name} recommendation`}
                        onClick={() => void handleDismissRecommendation(rec)}
                        className="max-md:min-h-11 gap-1 text-ink-2 hover:text-ink"
                      >
                        <X className="size-3" strokeWidth={1.5} aria-hidden />
                        <span>Dismiss</span>
                      </Button>
                    </div>
                  </article>
                );
              })}
            </div>
          </section>
        )}

        {suites.length > 0 && (
          <div id="plugins-oauth-strip" className="grid min-w-0 gap-2.5 md:grid-cols-2">
            {suites.map((s) => {
              const isBusy = oauthBusy === s.provider;
              const isEditing = editingSuite === s.provider;
              return (
                <div
                  key={s.provider}
                  data-oauth-provider={s.provider}
                  className="flex min-w-0 flex-col gap-2 rounded-card border border-hairline bg-surface p-3"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="min-w-0">
                      <p className="truncate text-sm font-semibold text-ink">{s.label}</p>
                      <p className="truncate text-xs text-ink-2">
                        {s.connected
                          ? `Signed in${s.account_email ? ` as ${s.account_email}` : ""}`
                          : s.configured
                            ? "Sign in once across 6 workspace apps"
                            : canManageConnectors
                              ? "Enter your company app ID once so employees can sign in with 1 click"
                              : "Ask your admin to finish single sign-in setup; local files work offline"}
                      </p>
                    </div>
                    <Badge
                      variant={s.connected ? "success" : s.configured ? "secondary" : "warning"}
                      size="sm"
                    >
                      {s.connected ? "Connected" : s.configured ? "Ready" : "Setup needed"}
                    </Badge>
                  </div>

                  <div className="flex flex-wrap items-center gap-1.5">
                    {s.connected ? (
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        disabled={isBusy}
                        onClick={() => void handleOAuthDisconnect(s.provider)}
                        className="max-md:min-h-11 gap-1"
                      >
                        <LogOut className="size-3" strokeWidth={1.5} aria-hidden />
                        <span>Disconnect</span>
                      </Button>
                    ) : s.configured ? (
                      <Button
                        type="button"
                        size="xs"
                        variant="default"
                        disabled={isBusy}
                        onClick={() => void handleOAuthConnect(s.provider)}
                        className="max-md:min-h-11 gap-1"
                      >
                        <KeyRound className="size-3" strokeWidth={1.5} aria-hidden />
                        <span>{s.provider === "google" ? "Sign in with Google" : "Sign in with Microsoft"}</span>
                      </Button>
                    ) : null}

                    {canManageConnectors && (
                      <Button
                        type="button"
                        size="xs"
                        variant={isEditing ? "secondary" : "outline"}
                        disabled={isBusy}
                        onClick={() => (isEditing ? setEditingSuite(null) : openOAuthEditor(s))}
                        className="max-md:min-h-11"
                      >
                        {isEditing ? "Close setup" : s.configured ? "Edit sign-in" : "Setup sign-in"}
                      </Button>
                    )}
                  </div>

                  {isEditing && (
                    <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/50 p-2.5">
                      <div className="flex items-center gap-1.5">
                        <Input
                          value={s.redirect_uri}
                          readOnly
                          aria-label={`${s.label} Redirect URI`}
                          className="mono h-8 flex-1 text-xs"
                        />
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          onClick={() => copyRedirectUri(s.redirect_uri, s.provider)}
                          className="max-md:min-h-11 shrink-0 gap-1"
                        >
                          {copiedUri === s.provider ? (
                            <Check className="size-3" strokeWidth={1.5} aria-hidden />
                          ) : (
                            <Copy className="size-3" strokeWidth={1.5} aria-hidden />
                          )}
                          <span>{copiedUri === s.provider ? "Copied" : "Copy URI"}</span>
                        </Button>
                      </div>
                      <Input
                        value={clientIdDraft}
                        onChange={(e) => setClientIdDraft(e.target.value)}
                        placeholder={
                          s.provider === "google"
                            ? "Google OAuth Client ID (*.apps.googleusercontent.com)"
                            : "Microsoft Entra Application (client) ID"
                        }
                        className="h-9 text-xs"
                      />
                      <Input
                        type="password"
                        value={clientSecretDraft}
                        onChange={(e) => setClientSecretDraft(e.target.value)}
                        placeholder={
                          s.has_client_secret
                            ? "Client Secret saved (leave blank to keep)"
                            : "Client Secret (optional)"
                        }
                        className="h-9 text-xs"
                      />
                      {s.provider === "microsoft" && (
                        <Input
                          value={tenantDraft}
                          onChange={(e) => setTenantDraft(e.target.value)}
                          placeholder="Tenant ID (common or directory GUID)"
                          className="h-9 text-xs"
                        />
                      )}
                      <div className="flex items-center justify-end gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="ghost"
                          className="max-md:min-h-11"
                          onClick={() => setEditingSuite(null)}
                        >
                          Cancel
                        </Button>
                        <Button
                          type="button"
                          size="xs"
                          variant="default"
                          disabled={isBusy}
                          className="max-md:min-h-11"
                          onClick={() => void handleSaveOAuthApp(s.provider)}
                        >
                          Save app
                        </Button>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}

        <div className="flex flex-col gap-2">
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative min-w-[200px] flex-1">
              <Search
                className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-ink-3"
                strokeWidth={1.5}
                aria-hidden
              />
              <Input
                id="plugins-search"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && query.trim().length >= 2) {
                    void handleSearchLiveDirectory(query);
                  }
                }}
                placeholder="Search apps (Gmail, Outlook, Slack, Linear, Stripe…)"
                className="pl-8 text-sm max-md:min-h-11"
              />
            </div>
            <Button
              id="plugins-live-search-btn"
              type="button"
              size="sm"
              variant="outline"
              disabled={searchingLive}
              onClick={() => void handleSearchLiveDirectory()}
              className="max-md:min-h-11 gap-1.5"
            >
              <Globe className="size-3.5" strokeWidth={1.5} aria-hidden />
              <span>{searchingLive ? "Searching…" : "Search directory"}</span>
            </Button>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap gap-1" role="group" aria-label="Filter apps by suite">
              {SUITE_TABS.map((t) => {
                const active = suite === t.id;
                return (
                  <Button
                    key={t.id}
                    type="button"
                    size="xs"
                    variant={active ? "default" : "outline"}
                    onClick={() => setSuite(t.id)}
                    className="max-md:min-h-11"
                  >
                    {t.label}
                  </Button>
                );
              })}
            </div>
            {!simpleMode && (
              <div className="flex flex-wrap gap-1" role="group" aria-label="Filter apps by mode">
                {MODE_TABS.map((m) => {
                  const active = modeFilter === m.id;
                  return (
                    <Button
                      key={m.id}
                      type="button"
                      size="xs"
                      variant={active ? "secondary" : "ghost"}
                      onClick={() => setModeFilter(m.id)}
                      className="max-md:min-h-11"
                    >
                      {m.label}
                    </Button>
                  );
                })}
              </div>
            )}
          </div>
        </div>

        {err && (
          <div role="alert" className="rounded-card border border-warn/40 bg-warn-soft px-3 py-2 text-xs text-warn">
            {err}
          </div>
        )}

        {loading && plugins.length === 0 ? (
          <p className="py-6 text-center text-sm text-ink-2">Loading apps…</p>
        ) : filtered.length === 0 ? (
          <div className="flex flex-col items-center gap-2 py-6 text-center text-sm text-ink-2">
            <p>No local apps match &ldquo;{query}&rdquo;.</p>
            {query.trim().length >= 2 && (
              <Button
                type="button"
                size="xs"
                variant="outline"
                disabled={searchingLive}
                onClick={() => void handleSearchLiveDirectory(query)}
                className="max-md:min-h-11 gap-1.5"
              >
                <Globe className="size-3.5" strokeWidth={1.5} aria-hidden />
                <span>Search live app directory for &ldquo;{query.trim()}&rdquo;</span>
              </Button>
            )}
          </div>
        ) : (
          <div className="grid min-w-0 gap-3 md:grid-cols-2">
            {filtered.map((p) => renderAppCard(p, false))}
          </div>
        )}

        {liveSearchError && (
          <div role="alert" className="rounded-card border border-hairline bg-board/50 px-3 py-2 text-xs text-ink-2">
            Live directory check skipped: {liveSearchError}
          </div>
        )}

        {remoteResults.length > 0 && (
          <section aria-label="Live app directory results" className="flex flex-col gap-2 border-t border-hairline pt-3">
            <div className="flex items-center justify-between gap-2">
              <p className="text-xs font-medium text-ink">
                Directory matches for &ldquo;{liveSearchQuery}&rdquo; ({remoteResults.length})
              </p>
              <Button
                type="button"
                size="xs"
                variant="ghost"
                onClick={() => setRemoteResults([])}
                className="max-md:min-h-11"
              >
                Clear
              </Button>
            </div>
            <div className="grid min-w-0 gap-3 md:grid-cols-2">
              {remoteResults.map((p) => renderAppCard(p, true))}
            </div>
          </section>
        )}
      </div>
    </PanelModal>
  );
}
