import { useCallback, useEffect, useMemo, useState } from "react";
import { Check, Cloud, Copy, Cpu, KeyRound, Layers, LogOut, Play, Plug, Search, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { PanelModal } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  disconnectPluginOAuth,
  listPluginOAuthSuites,
  listPlugins,
  runPluginAction,
  savePluginOAuthApp,
  startPluginOAuth,
  togglePlugin,
  type PluginOAuthSuite,
  type PluginSpecItem,
} from "@/lib/api";

type SuiteFilter = "all" | "google" | "microsoft" | "business" | "local";
type ModeFilter = "all" | "local" | "hybrid" | "cloud";

const SUITE_TABS: Array<{ id: SuiteFilter; label: string }> = [
  { id: "all", label: "All" },
  { id: "google", label: "Google Workspace" },
  { id: "microsoft", label: "Microsoft 365" },
  { id: "business", label: "Business" },
  { id: "local", label: "On-Device" },
];

const MODE_TABS: Array<{ id: ModeFilter; label: string }> = [
  { id: "all", label: "Any mode" },
  { id: "local", label: "Local" },
  { id: "hybrid", label: "Hybrid" },
  { id: "cloud", label: "Cloud" },
];

function statusLabel(p: PluginSpecItem): { text: string; tone: "ok" | "accent" | "warn" | "muted" } {
  switch (p.status) {
    case "connected_hybrid":
      return { text: "Hybrid · Live", tone: "ok" };
    case "connected_cloud":
      return { text: "Cloud · Live", tone: "ok" };
    case "active_local":
      return { text: "Local · Ready", tone: "accent" };
    case "enabled_needs_auth":
      return { text: "Needs OAuth2", tone: "warn" };
    default:
      return { text: "Available", tone: "muted" };
  }
}

export function PluginsPanel({
  open,
  onOpenChange,
  canManageConnectors,
  onOpenConnectors,
  onChanged,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  canManageConnectors: boolean;
  onOpenConnectors: () => void;
  onChanged?: (enabledCount: number) => void;
}) {
  const [plugins, setPlugins] = useState<PluginSpecItem[]>([]);
  const [suites, setSuites] = useState<PluginOAuthSuite[]>([]);
  const [loading, setLoading] = useState(false);
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

  const refresh = useCallback(async () => {
    setLoading(true);
    setErr("");
    try {
      const [r, oauthRes] = await Promise.all([listPlugins(), listPluginOAuthSuites()]);
      if (!r.ok) {
        setErr(r.error || "Could not load the plugin library.");
        return;
      }
      setPlugins(r.data.plugins);
      onChanged?.(r.data.summary.enabled_count);
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

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return plugins.filter((p) => {
      if (suite !== "all" && p.suite !== suite) return false;
      if (modeFilter !== "all" && p.mode !== modeFilter) return false;
      if (!q) return true;
      return (
        p.name.toLowerCase().includes(q) ||
        p.tagline.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.suite_label.toLowerCase().includes(q)
      );
    });
  }, [plugins, suite, modeFilter, query]);

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

  const handleToggle = async (p: PluginSpecItem) => {
    setBusyId(p.id);
    try {
      const r = await togglePlugin(p.id, { enabled: !p.enabled });
      if (!r.ok) {
        toast.error(r.error || `Could not update ${p.name}.`);
        return;
      }
      await refresh();
      toast.success(!p.enabled ? `Enabled ${p.name} for your agent.` : `Paused ${p.name}.`);
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
      toast.success(`${p.name} (${r.data.mode}) ran cleanly.`);
    } finally {
      setBusyId(null);
    }
  };

  const handleOAuthConnect = async (provider: "google" | "microsoft") => {
    setOauthBusy(provider);
    try {
      const r = await startPluginOAuth(provider, "/");
      if (!r.ok) {
        toast.error(r.error || `Could not start ${provider} OAuth2 sign-in.`);
        return;
      }
      if (r.data.demo_connected) {
        await refresh();
        toast.success(
          provider === "google"
            ? "Connected Google Workspace (demo) to your agent."
            : "Connected Microsoft 365 & Outlook (demo) to your agent."
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
        toast.error(r.error || "Could not disconnect OAuth2 account.");
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
        toast.error(r.error || "Could not save OAuth app settings.");
        return;
      }
      setEditingSuite(null);
      setClientSecretDraft("");
      await refresh();
      toast.success("OAuth app saved securely. Employees can now sign in with one click.");
    } finally {
      setOauthBusy(null);
    }
  };

  const copyRedirectUri = (uri: string, provider: string) => {
    void navigator.clipboard?.writeText(uri);
    setCopiedUri(provider);
    setTimeout(() => setCopiedUri((cur) => (cur === provider ? null : cur)), 1600);
  };

  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Default App Plugin Library"
      className="max-w-3xl"
    >
      <div id="plugins-panel" className="flex min-w-0 flex-col gap-4">
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-card border border-hairline bg-board/50 px-3 py-2.5">
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-ink">
              Starting apps for your agent — cloud &amp; on-device
            </p>
            <p className="text-xs text-ink-2">
              {summary.enabledCount} active of {summary.total} plugins · {summary.localCount} offline-capable ·{" "}
              {summary.cloudCount} cloud-connected. External writes always park for /phone approval.
            </p>
          </div>
          {canManageConnectors && (
            <Button
              type="button"
              size="sm"
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
                            ? "One-click OAuth2 PKCE ready across 6 plugins"
                            : canManageConnectors
                              ? "Register your org OAuth Client ID to enable 1-click sign-in"
                              : "Ask an admin to register OAuth; local tools work offline now"}
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
                        {isEditing ? "Close setup" : s.configured ? "Edit OAuth" : "Setup OAuth"}
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
                          className="mono h-7 flex-1 text-[11px]"
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
                        className="h-8 text-xs"
                      />
                      <Input
                        type="password"
                        value={clientSecretDraft}
                        onChange={(e) => setClientSecretDraft(e.target.value)}
                        placeholder={
                          s.has_client_secret
                            ? "Client Secret saved (leave blank to keep)"
                            : "Client Secret (optional for PKCE desktop clients)"
                        }
                        className="h-8 text-xs"
                      />
                      {s.provider === "microsoft" && (
                        <Input
                          value={tenantDraft}
                          onChange={(e) => setTenantDraft(e.target.value)}
                          placeholder="Tenant ID (common, organizations, or directory GUID)"
                          className="h-8 text-xs"
                        />
                      )}
                      <div className="flex items-center justify-end gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="ghost"
                          onClick={() => setEditingSuite(null)}
                        >
                          Cancel
                        </Button>
                        <Button
                          type="button"
                          size="xs"
                          variant="default"
                          disabled={isBusy}
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
          <div className="relative">
            <Search
              className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-ink-3"
              strokeWidth={1.5}
              aria-hidden
            />
            <Input
              id="plugins-search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Filter Google, Outlook, Office, Slack, Notion, or Local apps..."
              className="pl-8 text-sm"
            />
          </div>

          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap gap-1" role="group" aria-label="Filter plugins by suite">
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
            <div className="flex flex-wrap gap-1" role="group" aria-label="Filter plugins by mode">
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
          </div>
        </div>

        {err && (
          <div role="alert" className="rounded-card border border-warn/40 bg-warn-soft px-3 py-2 text-xs text-warn">
            {err}
          </div>
        )}

        {loading && plugins.length === 0 ? (
          <p className="py-6 text-center text-sm text-ink-2">Loading plugins…</p>
        ) : filtered.length === 0 ? (
          <p className="py-6 text-center text-sm text-ink-2">No plugins match your current filter.</p>
        ) : (
          <div className="grid min-w-0 gap-3 md:grid-cols-2">
            {filtered.map((p) => {
              const st = statusLabel(p);
              const isBusy = busyId === p.id;
              const safeRead = p.actions.find((a) => !a.requires_approval && a.access === "read");
              const oauthSuite = p.oauth_provider ? suiteByProvider[p.oauth_provider] : undefined;
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
                          <Badge variant="outline" size="sm" className="gap-1">
                            {p.mode === "local" ? (
                              <Cpu className="size-3" strokeWidth={1.5} aria-hidden />
                            ) : p.mode === "cloud" ? (
                              <Cloud className="size-3" strokeWidth={1.5} aria-hidden />
                            ) : (
                              <Layers className="size-3" strokeWidth={1.5} aria-hidden />
                            )}
                            <span>{p.mode}</span>
                          </Badge>
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
                      <p className="mono truncate text-[11px] text-accent">
                        Bound: {p.connected_via}
                      </p>
                    )}

                    <div className="flex flex-wrap gap-1 pt-0.5">
                      {p.actions.map((a) => (
                        <span
                          key={a.name}
                          title={a.description}
                          className="inline-flex items-center gap-1 rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-[11px] text-ink-2"
                        >
                          {a.requires_approval && (
                            <ShieldCheck className="size-3 text-warn" strokeWidth={1.5} aria-hidden />
                          )}
                          <span>{a.label}</span>
                        </span>
                      ))}
                    </div>

                    {lastResult[p.id] && (
                      <p className="mono line-clamp-2 rounded-sm bg-board px-2 py-1 text-[11px] text-ink-2">
                        {lastResult[p.id]}
                      </p>
                    )}
                  </div>

                  <div className="flex flex-wrap items-center justify-between gap-1.5 border-t border-hairline pt-2">
                    <div className="flex flex-wrap items-center gap-1">
                      <Button
                        type="button"
                        size="xs"
                        variant={p.enabled ? "secondary" : "default"}
                        disabled={isBusy}
                        onClick={() => void handleToggle(p)}
                        className="max-md:min-h-11"
                      >
                        {p.enabled ? "Enabled" : "Enable"}
                      </Button>
                      {p.mode === "hybrid" && p.local_ready && (
                        <div className="inline-flex rounded-sm border border-hairline bg-board/50 p-0.5">
                          {(["auto", "local", "cloud"] as const).map((m) => (
                            <button
                              key={m}
                              type="button"
                              disabled={isBusy}
                              onClick={() => void handleModeChange(p, m)}
                              className={`rounded-sm px-1.5 py-0.5 text-[11px] ${
                                p.preferred_mode === m
                                  ? "bg-surface font-medium text-ink"
                                  : "text-ink-3 hover:text-ink"
                              }`}
                            >
                              {m}
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-1">
                      {(p.local_ready || p.cloud_connected) && safeRead && (
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          disabled={isBusy}
                          onClick={() => void handleQuickTest(p)}
                          className="max-md:min-h-11 gap-1"
                        >
                          <Play className="size-3" strokeWidth={1.5} aria-hidden />
                          <span>Run</span>
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
                      {p.connect_preset && !p.cloud_connected && (!oauthSuite || !oauthSuite.configured) && canManageConnectors && (
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
            })}
          </div>
        )}
      </div>
    </PanelModal>
  );
}
