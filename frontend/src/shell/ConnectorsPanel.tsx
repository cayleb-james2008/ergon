// Connectors configuration dialog (always mounted OUTSIDE Vaul Drawer):
// 1. Inbound Webhooks: create signed hooks, copy endpoint URL, rotate signing secret (one-time reveal), simulate payload
// 2. Watched Folders: monitor user-approved host directory with glob filter -> target routine
// 3. Outbound Actions: configure credential-isolated connectors (masked secret input, never exposes stored secrets),
//    Test Delivery button, policy gate awareness, and Delivery Log
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Check,
  Copy,
  EyeOff,
  FolderInput,
  KeyRound,
  Lock,
  Play,
  Plus,
  RotateCw,
  Send,
  ShieldAlert,
  Trash2,
  Webhook,
} from "lucide-react";
import { toast } from "sonner";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Pill } from "@/components/kibo-ui/pill";
import {
  call,
  deleteOutboundConnector,
  deleteTriggerRule,
  listConnectors,
  rotateTriggerSecret,
  saveOutboundConnector,
  saveTriggerRule,
  simulateWebhookTrigger,
  testOutboundConnector,
  type DeliveryLog,
  type OutboundConnector,
  type OutboundConnectorKind,
  type Routine,
  type Status,
  type TriggerRule,
  type WorkspacePolicy,
  type WorkspacePolicyProfile,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import { relTime } from "@/lib/words";

type ConnectorTab = "webhooks" | "folders" | "outbound";

function parseHeadersText(raw: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const idx = trimmed.indexOf(":");
    if (idx <= 0) continue;
    const key = trimmed.slice(0, idx).trim();
    const val = trimmed.slice(idx + 1).trim();
    if (key) out[key] = val;
  }
  return out;
}

function deliveryStatusBadge(status: string): { label: string; cls: string } {
  const s = status.toLowerCase();
  if (s === "ok" || s === "sent" || s === "delivered") {
    return { label: "Sent", cls: "bg-pill-active-bg text-pill-active-fg" };
  }
  if (s === "pending_approval" || s === "waiting" || s === "held") {
    return { label: "Waiting approval", cls: "bg-pill-warn-bg text-pill-warn-fg" };
  }
  if (s === "blocked" || s === "denied") {
    return { label: s === "denied" ? "Declined" : "Blocked", cls: "bg-status-stopped-bg text-status-stopped-fg" };
  }
  return { label: "Failed", cls: "bg-status-stopped-bg text-status-stopped-fg" };
}

function kindLabel(kind: OutboundConnectorKind | string): string {
  if (kind === "rest_api") return "REST API";
  if (kind === "smtp_relay") return "SMTP Relay";
  return "HTTP Webhook";
}

function ConnectorsBody({
  routines,
  status,
  onChanged,
  onOpenToday,
}: {
  routines: Routine[];
  status: Status | null | "error";
  onChanged?: () => void;
  onOpenToday?: () => void;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const [tab, setTab] = useState<ConnectorTab>("webhooks");
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");
  const [triggers, setTriggers] = useState<TriggerRule[]>([]);
  const [outbound, setOutbound] = useState<OutboundConnector[]>([]);
  const [deliveries, setDeliveries] = useState<DeliveryLog[]>([]);
  const [policyProfile, setPolicyProfile] = useState<WorkspacePolicyProfile>("fast_internal");

  // Transient one-time signing secrets created or rotated during this modal session.
  const [ephemeralSecrets, setEphemeralSecrets] = useState<Record<string, string>>({});
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Inbound webhook builder & simulator state
  const [addingHook, setAddingHook] = useState(false);
  const [hookName, setHookName] = useState("");
  const [hookRoutineId, setHookRoutineId] = useState("");
  const [savingHook, setSavingHook] = useState(false);
  const [rotatingId, setRotatingId] = useState<string | null>(null);
  const [simOpenId, setSimOpenId] = useState<string | null>(null);
  const [simPayloads, setSimPayloads] = useState<Record<string, string>>({});
  const [simTokens, setSimTokens] = useState<Record<string, string>>({});
  const [simBusyId, setSimBusyId] = useState<string | null>(null);
  const [simResultById, setSimResultById] = useState<Record<string, { ok: boolean; text: string }>>({});

  // Watched folder builder state
  const [addingFolder, setAddingFolder] = useState(false);
  const [folderName, setFolderName] = useState("");
  const [folderPath, setFolderPath] = useState("");
  const [folderGlob, setFolderGlob] = useState("*.pdf,*.csv");
  const [folderRoutineId, setFolderRoutineId] = useState("");
  const [savingFolder, setSavingFolder] = useState(false);

  // Outbound connector builder state (secret value is strictly write-only and cleared immediately)
  const [addingOutbound, setAddingOutbound] = useState(false);
  const [outName, setOutName] = useState("");
  const [outKind, setOutKind] = useState<OutboundConnectorKind>("http_webhook");
  const [outMethod, setOutMethod] = useState<"POST" | "PUT">("POST");
  const [outUrl, setOutUrl] = useState("");
  const [outHeadersText, setOutHeadersText] = useState("Authorization: Bearer {{secret:api_token}}");
  const [outSecretKey, setOutSecretKey] = useState("api_token");
  const [outSecretValue, setOutSecretValue] = useState("");
  const [outRequireApproval, setOutRequireApproval] = useState(true);
  const [savingOutbound, setSavingOutbound] = useState(false);
  const [testingOutboundId, setTestingOutboundId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setErr("");
    const [res, polRes] = await Promise.all([
      listConnectors(),
      call<WorkspacePolicy>("/api/workspace/policy"),
    ]);
    setLoading(false);
    if (!res.ok) {
      setErr(res.error || "Could not load connectors.");
      return;
    }
    setTriggers(res.data.triggers);
    setOutbound(res.data.outbound);
    setDeliveries(res.data.deliveries);
    if (res.data.policy_profile) {
      setPolicyProfile(res.data.policy_profile);
    } else if (polRes.ok && polRes.data.profile) {
      setPolicyProfile(polRes.data.profile);
    }
  }, []);

  useEffect(() => {
    if (!isOpen) return;
    void load();
  }, [isOpen, load, status]);

  useEffect(() => {
    if (routines.length > 0) {
      if (!hookRoutineId) setHookRoutineId(routines[0].id);
      if (!folderRoutineId) setFolderRoutineId(routines[0].id);
    }
  }, [routines, hookRoutineId, folderRoutineId]);

  const routineNameMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const r of routines) map.set(r.id, r.name);
    return map;
  }, [routines]);

  const webhookTriggers = useMemo(
    () => triggers.filter((t) => t.kind === "webhook"),
    [triggers]
  );
  const folderTriggers = useMemo(
    () => triggers.filter((t) => t.kind === "folder_watch"),
    [triggers]
  );

  const copyText = async (id: string, value: string, label: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedId(id);
      toast.success(`Copied ${label}.`);
      window.setTimeout(() => {
        setCopiedId((cur) => (cur === id ? null : cur));
      }, 1800);
    } catch {
      toast.error("Clipboard access is unavailable here.");
    }
  };

  const fullHookUrl = (rule: TriggerRule) => {
    const rel = rule.hook_path || rule.webhook_url || `/api/hooks/${encodeURIComponent(rule.hook_id || rule.id)}`;
    if (/^https?:\/\//i.test(rel)) return rel;
    return `${window.location.origin}${rel.startsWith("/") ? "" : "/"}${rel}`;
  };

  const handleCreateWebhook = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = hookName.trim();
    if (!name || !hookRoutineId) {
      setErr("Enter a webhook name and choose a target routine.");
      return;
    }
    setSavingHook(true);
    setErr("");
    const r = await saveTriggerRule({
      name,
      kind: "webhook",
      routine_id: hookRoutineId,
      enabled: true,
    });
    setSavingHook(false);
    if (!r.ok) {
      setErr(r.error || "Could not create inbound webhook.");
      return;
    }
    const createdId = r.data.trigger?.id;
    const oneTimeSecret = r.data.signing_secret || r.data.trigger?.signing_secret || r.data.hook_token;
    if (createdId && oneTimeSecret) {
      setEphemeralSecrets((prev) => ({ ...prev, [createdId]: oneTimeSecret }));
    }
    setHookName("");
    setAddingHook(false);
    toast.success(`Created webhook "${name}".`);
    await load();
    onChanged?.();
  };

  const handleRotateSecret = async (rule: TriggerRule) => {
    setRotatingId(rule.id);
    setErr("");
    const r = await rotateTriggerSecret(rule.id);
    setRotatingId(null);
    if (!r.ok) {
      toast.error(r.error || "Could not rotate signing secret.");
      return;
    }
    const nextSecret = r.data.signing_secret || r.data.trigger?.signing_secret || r.data.hook_token;
    if (nextSecret) {
      setEphemeralSecrets((prev) => ({ ...prev, [rule.id]: nextSecret }));
    }
    toast.success(`Rotated signing secret for "${rule.name}".`);
    await load();
  };

  const handleSimulateWebhook = async (rule: TriggerRule) => {
    const hookId = rule.hook_id || rule.id;
    const rawPayload =
      simPayloads[rule.id] ??
      JSON.stringify(
        {
          event: "intake.received",
          source: rule.name,
          received_at: new Date().toISOString(),
          amount: 420.5,
        },
        null,
        2
      );
    setSimBusyId(rule.id);
    const token = simTokens[rule.id] ?? ephemeralSecrets[rule.id] ?? "";
    const r = await simulateWebhookTrigger(hookId, rawPayload, token);
    setSimBusyId(null);
    if (!r.ok) {
      setSimResultById((prev) => ({
        ...prev,
        [rule.id]: { ok: false, text: r.error || "Webhook simulation failed." },
      }));
      toast.error(r.error || "Webhook simulation failed.");
      return;
    }
    const detail = r.data.trigger_file
      ? `Saved payload to ${r.data.trigger_file} and triggered routine.`
      : r.data.message || "Webhook payload accepted and target routine triggered.";
    setSimResultById((prev) => ({
      ...prev,
      [rule.id]: { ok: true, text: detail },
    }));
    toast.success(detail);
    await load();
    onChanged?.();
  };

  const handleCreateFolderWatch = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = folderName.trim();
    const dir = folderPath.trim();
    if (!name || !dir || !folderRoutineId) {
      setErr("Enter a rule name, host directory path, and target routine.");
      return;
    }
    setSavingFolder(true);
    setErr("");
    const r = await saveTriggerRule({
      name,
      kind: "folder_watch",
      watch_dir: dir,
      glob: folderGlob.trim() || "*.*",
      routine_id: folderRoutineId,
      enabled: true,
    });
    setSavingFolder(false);
    if (!r.ok) {
      setErr(r.error || "Could not create watched folder rule.");
      return;
    }
    setFolderName("");
    setFolderPath("");
    setFolderGlob("*.pdf,*.csv");
    setAddingFolder(false);
    toast.success(`Watching "${dir}" for ${folderGlob.trim() || "*.*"}.`);
    await load();
    onChanged?.();
  };

  const handleCreateOutbound = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = outName.trim();
    const url = outUrl.trim();
    if (!name || !url) {
      setErr("Enter a connector name and destination URL.");
      return;
    }
    setSavingOutbound(true);
    setErr("");
    const headers_template = parseHeadersText(outHeadersText);
    const secrets: Record<string, string> = {};
    const skey = outSecretKey.trim();
    const sval = outSecretValue.trim();
    if (skey && sval) {
      secrets[skey] = sval;
    }
    const r = await saveOutboundConnector({
      name,
      kind: outKind,
      url,
      method: outMethod,
      headers_template,
      secrets: Object.keys(secrets).length > 0 ? secrets : undefined,
      require_approval: policyProfile === "strict_client" ? true : outRequireApproval,
      enabled: true,
    });
    setSavingOutbound(false);
    // Always wipe the secret input immediately so no raw secret lingers in React state.
    setOutSecretValue("");
    if (!r.ok) {
      setErr(r.error || "Could not save outbound connector.");
      return;
    }
    setOutName("");
    setOutUrl("");
    setAddingOutbound(false);
    toast.success(`Saved outbound connector "${name}".`);
    await load();
    onChanged?.();
  };

  const handleTestOutbound = async (c: OutboundConnector) => {
    setTestingOutboundId(c.id);
    setErr("");
    const r = await testOutboundConnector(c.id, {
      event: "ergon.connector.test",
      connector: c.name,
      sent_at: new Date().toISOString(),
    });
    setTestingOutboundId(null);
    if (!r.ok) {
      toast.error(r.error || `Delivery test failed for "${c.name}".`);
      await load();
      return;
    }
    const st = String(r.data.delivery?.status || r.data.status || "ok").toLowerCase();
    if (r.data.held || st === "held" || st === "pending_approval" || r.data.decision_id) {
      toast.warning(`Held "${c.name}" for your approval in Today.`);
    } else if (st === "blocked") {
      toast.error(r.data.error || r.data.message || `Blocked "${c.name}" by workspace policy.`);
    } else if (r.data.ok === false || st === "error") {
      toast.error(r.data.error || r.data.message || `Delivery test failed for "${c.name}".`);
    } else {
      toast.success(r.data.message || `Test delivery succeeded for "${c.name}".`);
    }
    await load();
    onChanged?.();
  };

  const handleDeleteTrigger = async (rule: TriggerRule) => {
    const r = await deleteTriggerRule(rule.id);
    if (!r.ok) {
      toast.error(r.error || "Could not remove trigger.");
      return;
    }
    setEphemeralSecrets((prev) => {
      const next = { ...prev };
      delete next[rule.id];
      return next;
    });
    toast.success(`Removed "${rule.name}".`);
    await load();
    onChanged?.();
  };

  const handleDeleteOutbound = async (c: OutboundConnector) => {
    const r = await deleteOutboundConnector(c.id);
    if (!r.ok) {
      toast.error(r.error || "Could not remove connector.");
      return;
    }
    toast.success(`Removed "${c.name}".`);
    await load();
    onChanged?.();
  };

  return (
    <div id="connectors-panel" className="flex flex-col gap-4 pt-1">
      <p className="text-sm text-ink-2">
        Receive signed webhooks or new host files into a routine, and send results through credential-isolated outbound connectors.
      </p>

      {err && (
        <div role="alert" className="flex items-start gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg">
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
          <span>{err}</span>
        </div>
      )}

      <Tabs value={tab} onValueChange={(v) => { setErr(""); setTab(v as ConnectorTab); }}>
        <TabsList aria-label="Connector categories" className="w-full sm:w-fit">
          <TabsTrigger value="webhooks" data-id="webhooks" className="max-md:min-h-11 gap-1.5 px-2 text-xs sm:px-3 sm:text-sm">
            <Webhook className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Webhooks</span>
            <span className="hidden sm:inline">Inbound Webhooks</span>
            <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{webhookTriggers.length}</span>
          </TabsTrigger>
          <TabsTrigger value="folders" data-id="folders" className="max-md:min-h-11 gap-1.5 px-2 text-xs sm:px-3 sm:text-sm">
            <FolderInput className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Folders</span>
            <span className="hidden sm:inline">Watched Folders</span>
            <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{folderTriggers.length}</span>
          </TabsTrigger>
          <TabsTrigger value="outbound" data-id="outbound" className="max-md:min-h-11 gap-1.5 px-2 text-xs sm:px-3 sm:text-sm">
            <Send className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Outbound</span>
            <span className="hidden sm:inline">Outbound Actions</span>
            <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{outbound.length}</span>
          </TabsTrigger>
        </TabsList>

        {/* Tab 1: Inbound Webhooks */}
        <TabsContent value="webhooks" className="flex flex-col gap-3 pt-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs text-ink-2">
              External tools post JSON to a signed endpoint and trigger a routine automatically.
            </span>
            <Button
              type="button"
              size="sm"
              variant={addingHook ? "outline" : "default"}
              className="max-md:min-h-11"
              onClick={() => setAddingHook((v) => !v)}
            >
              <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
              {addingHook ? "Close form" : "New webhook"}
            </Button>
          </div>

          {addingHook && (
            <form
              id="webhook-create-form"
              onSubmit={handleCreateWebhook}
              className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex flex-col gap-1">
                  <label htmlFor="webhook-name" className="text-xs font-medium text-ink">Webhook name</label>
                  <Input
                    id="webhook-name"
                    value={hookName}
                    onChange={(e) => setHookName(e.target.value)}
                    placeholder="Stripe invoice intake"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="webhook-routine" className="text-xs font-medium text-ink">Target routine</label>
                  <select
                    id="webhook-routine"
                    value={hookRoutineId}
                    onChange={(e) => setHookRoutineId(e.target.value)}
                    className="h-9 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    {routines.length === 0 ? (
                      <option value="">No routines available yet</option>
                    ) : (
                      routines.map((rt) => (
                        <option key={rt.id} value={rt.id}>{rt.name}</option>
                      ))
                    )}
                  </select>
                </div>
              </div>
              <div className="flex flex-wrap items-center justify-end gap-2">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => setAddingHook(false)}>
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingHook || routines.length === 0}>
                  {savingHook ? "Creating…" : "Create webhook"}
                </Button>
              </div>
            </form>
          )}

          {loading && webhookTriggers.length === 0 ? (
            <p className="py-4 text-sm text-ink-2">Loading inbound webhooks…</p>
          ) : webhookTriggers.length === 0 ? (
            <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
              No inbound webhooks yet. Create one to accept signed payloads into your workspace inbox.
            </div>
          ) : (
            <ul className="flex flex-col gap-2.5">
              {webhookTriggers.map((rule) => {
                const url = fullHookUrl(rule);
                const oneTimeSecret = ephemeralSecrets[rule.id];
                const simOpen = simOpenId === rule.id;
                const simRes = simResultById[rule.id];
                const targetName = (rule.routine_id && routineNameMap.get(rule.routine_id)) || rule.routine_id || "default handler";
                return (
                  <li
                    key={rule.id}
                    data-trigger-id={rule.id}
                    data-trigger-kind="webhook"
                    className="flex flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="flex flex-wrap items-start justify-between gap-2">
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-sm font-medium text-ink">{rule.name}</span>
                          <Pill
                            variant="secondary"
                            className={cn(
                              "px-2 py-0.5 text-xs",
                              rule.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-pill-inactive-bg text-pill-inactive-fg"
                            )}
                          >
                            {rule.enabled ? "Active" : "Paused"}
                          </Pill>
                          {rule.has_secret !== false && (
                            <Badge variant="outline" size="sm" className="gap-1 text-xs">
                              <Lock className="size-3" strokeWidth={1.5} aria-hidden />
                              HMAC signed
                            </Badge>
                          )}
                        </div>
                        <p className="mt-1 text-xs text-ink-2">
                          Triggers <span className="font-medium text-ink">{(rule.routine_id && routineNameMap.get(rule.routine_id)) || rule.routine_id || "default handler"}</span>
                          {typeof rule.trigger_count === "number" && (
                            <span className="mono text-ink-3"> · {rule.trigger_count} {rule.trigger_count === 1 ? "run" : "runs"}</span>
                          )}
                          {rule.last_triggered_ms ? (
                            <span className="mono text-ink-3"> · {relTime(rule.last_triggered_ms)}</span>
                          ) : null}
                        </p>
                      </div>
                      <Button
                        type="button"
                        size="icon-sm"
                        variant="ghost"
                        aria-label={`Delete webhook ${rule.name}`}
                        className="max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
                        onClick={() => void handleDeleteTrigger(rule)}
                      >
                        <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                      </Button>
                    </div>

                    <div className="flex flex-wrap items-center gap-2 rounded-sm border border-hairline bg-board/50 px-2.5 py-1.5">
                      <code className="mono min-w-0 flex-1 truncate text-xs text-ink">{url}</code>
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        data-action="copy-hook-url"
                        className="max-md:min-h-11 shrink-0"
                        onClick={() => void copyText(`url-${rule.id}`, url, "webhook URL")}
                      >
                        {copiedId === `url-${rule.id}` ? (
                          <Check className="size-3.5" strokeWidth={1.5} aria-hidden />
                        ) : (
                          <Copy className="size-3.5" strokeWidth={1.5} aria-hidden />
                        )}
                        Copy URL
                      </Button>
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        data-action="rotate-secret"
                        disabled={rotatingId === rule.id}
                        className="max-md:min-h-11 shrink-0"
                        onClick={() => void handleRotateSecret(rule)}
                      >
                        <RotateCw className="size-3.5" strokeWidth={1.5} aria-hidden />
                        {rotatingId === rule.id ? "Rotating…" : "Rotate secret"}
                      </Button>
                      <Button
                        type="button"
                        size="xs"
                        variant={simOpen ? "secondary" : "outline"}
                        data-action="toggle-simulate"
                        className="max-md:min-h-11 shrink-0"
                        onClick={() => setSimOpenId((cur) => (cur === rule.id ? null : rule.id))}
                      >
                        <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                        Simulate payload
                      </Button>
                    </div>

                    {oneTimeSecret && (
                      <div
                        data-one-time-secret={rule.id}
                        className="flex flex-col gap-2 rounded-sm border border-hairline bg-pill-warn-bg/40 p-2.5 text-xs"
                      >
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <span className="inline-flex items-center gap-1.5 font-medium text-ink">
                            <KeyRound className="size-3.5 text-dot-amber" strokeWidth={1.5} aria-hidden />
                            New signing secret (shown once)
                          </span>
                          <div className="flex items-center gap-1.5">
                            <Button
                              type="button"
                              size="xs"
                              variant="outline"
                              className="max-md:min-h-11"
                              onClick={() => void copyText(`sec-${rule.id}`, oneTimeSecret, "signing secret")}
                            >
                              <Copy className="size-3" strokeWidth={1.5} aria-hidden />
                              Copy secret
                            </Button>
                            <Button
                              type="button"
                              size="xs"
                              variant="ghost"
                              className="max-md:min-h-11"
                              onClick={() =>
                                setEphemeralSecrets((prev) => {
                                  const next = { ...prev };
                                  delete next[rule.id];
                                  return next;
                                })
                              }
                            >
                              Dismiss
                            </Button>
                          </div>
                        </div>
                        <code className="mono break-all rounded-sm border border-hairline bg-surface px-2 py-1 text-xs text-ink">
                          {oneTimeSecret}
                        </code>
                        <span className="text-ink-2">
                          Save this secret in your sender configuration. Ergon never exposes stored secrets after you dismiss this box.
                        </span>
                      </div>
                    )}

                    {simOpen && (
                      <div
                        data-simulate-panel={rule.id}
                        className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-3"
                      >
                        <div className="flex flex-col gap-1">
                          <label htmlFor={`sim-payload-${rule.id}`} className="text-xs font-medium text-ink">
                            Test JSON payload
                          </label>
                          <Textarea
                            id={`sim-payload-${rule.id}`}
                            rows={3}
                            className="mono text-xs"
                            value={
                              simPayloads[rule.id] ??
                              JSON.stringify(
                                {
                                  event: "intake.received",
                                  source: rule.name,
                                  amount: 420.5,
                                },
                                null,
                                2
                              )
                            }
                            onChange={(e) =>
                              setSimPayloads((prev) => ({ ...prev, [rule.id]: e.target.value }))
                            }
                          />
                        </div>
                        {!oneTimeSecret && (
                          <div className="flex flex-col gap-1">
                            <label htmlFor={`sim-token-${rule.id}`} className="text-xs font-medium text-ink">
                              Signing secret or hook token (optional if simulating locally)
                            </label>
                            <Input
                              id={`sim-token-${rule.id}`}
                              type="password"
                              autoComplete="new-password"
                              placeholder="Leave blank to sign with rotated session secret or simulate"
                              value={simTokens[rule.id] ?? ""}
                              onChange={(e) =>
                                setSimTokens((prev) => ({ ...prev, [rule.id]: e.target.value }))
                              }
                            />
                          </div>
                        )}
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <span className="text-xs text-ink-2">
                            Writes payload to <code className="mono">workspace/inbox/</code> and triggers {targetName}.
                          </span>
                          <Button
                            type="button"
                            size="sm"
                            data-action="simulate-webhook"
                            disabled={simBusyId === rule.id}
                            className="max-md:min-h-11"
                            onClick={() => void handleSimulateWebhook(rule)}
                          >
                            <Play className="size-3.5" strokeWidth={1.5} aria-hidden />
                            {simBusyId === rule.id ? "Sending…" : "Send test payload"}
                          </Button>
                        </div>
                        {simRes && (
                          <p
                            role="status"
                            className={cn(
                              "rounded-sm border border-hairline px-2.5 py-1.5 text-xs",
                              simRes.ok
                                ? "bg-pill-active-bg text-pill-active-fg"
                                : "bg-status-stopped-bg text-status-stopped-fg"
                            )}
                          >
                            {simRes.text}
                          </p>
                        )}
                      </div>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
        </TabsContent>

        {/* Tab 2: Watched Folders */}
        <TabsContent value="folders" className="flex flex-col gap-3 pt-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs text-ink-2">
              Copy stable files matching a glob filter from an approved host folder into your workspace intake.
            </span>
            <Button
              type="button"
              size="sm"
              variant={addingFolder ? "outline" : "default"}
              className="max-md:min-h-11"
              onClick={() => setAddingFolder((v) => !v)}
            >
              <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
              {addingFolder ? "Close form" : "Watch folder"}
            </Button>
          </div>

          {addingFolder && (
            <form
              id="folder-watch-form"
              onSubmit={handleCreateFolderWatch}
              className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex flex-col gap-1">
                  <label htmlFor="folder-watch-name" className="text-xs font-medium text-ink">Rule name</label>
                  <Input
                    id="folder-watch-name"
                    value={folderName}
                    onChange={(e) => setFolderName(e.target.value)}
                    placeholder="Vendor statements dropbox"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="folder-watch-routine" className="text-xs font-medium text-ink">Target routine</label>
                  <select
                    id="folder-watch-routine"
                    value={folderRoutineId}
                    onChange={(e) => setFolderRoutineId(e.target.value)}
                    className="h-9 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    {routines.length === 0 ? (
                      <option value="">No routines available yet</option>
                    ) : (
                      routines.map((rt) => (
                        <option key={rt.id} value={rt.id}>{rt.name}</option>
                      ))
                    )}
                  </select>
                </div>
              </div>
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="flex flex-col gap-1 sm:col-span-2">
                  <label htmlFor="folder-watch-path" className="text-xs font-medium text-ink">Host directory path</label>
                  <Input
                    id="folder-watch-path"
                    className="mono text-xs"
                    value={folderPath}
                    onChange={(e) => setFolderPath(e.target.value)}
                    placeholder="/home/you/Documents/VendorInbox"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="folder-watch-glob" className="text-xs font-medium text-ink">Glob filter</label>
                  <Input
                    id="folder-watch-glob"
                    className="mono text-xs"
                    value={folderGlob}
                    onChange={(e) => setFolderGlob(e.target.value)}
                    placeholder="*.pdf,*.csv"
                  />
                </div>
              </div>
              <p className="text-xs text-ink-2">
                Sensitive system and credential folders are blocked automatically. Only files under 25 MB with steady size and timestamp are ingested.
              </p>
              <div className="flex flex-wrap items-center justify-end gap-2">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => setAddingFolder(false)}>
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingFolder || routines.length === 0}>
                  {savingFolder ? "Saving…" : "Save watched folder"}
                </Button>
              </div>
            </form>
          )}

          {loading && folderTriggers.length === 0 ? (
            <p className="py-4 text-sm text-ink-2">Loading watched folders…</p>
          ) : folderTriggers.length === 0 ? (
            <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
              No watched folders configured yet. Add an approved local folder to route incoming PDFs or spreadsheets into a routine.
            </div>
          ) : (
            <ul className="flex flex-col gap-2.5">
              {folderTriggers.map((rule) => {
                const targetName = (rule.routine_id && routineNameMap.get(rule.routine_id)) || rule.routine_id || "default handler";
                const dirPath = rule.watch_dir || rule.folder_path || "";
                const globText = rule.glob || rule.glob_filter || "*.*";
                return (
                  <li
                    key={rule.id}
                    data-trigger-id={rule.id}
                    data-trigger-kind="folder_watch"
                    className="flex flex-wrap items-start justify-between gap-3 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-sm font-medium text-ink">{rule.name}</span>
                        <Pill
                          variant="secondary"
                          className={cn(
                            "px-2 py-0.5 text-xs",
                            rule.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-pill-inactive-bg text-pill-inactive-fg"
                          )}
                        >
                          {rule.enabled ? "Watching" : "Paused"}
                        </Pill>
                        <Badge variant="outline" size="sm" className="mono text-xs">
                          {globText}
                        </Badge>
                      </div>
                      <p className="mono mt-1 truncate text-xs text-ink-2">{dirPath}</p>
                      <p className="mt-1 text-xs text-ink-2">
                        Target routine: <span className="font-medium text-ink">{targetName}</span>
                        {typeof rule.trigger_count === "number" && (
                          <span className="mono text-ink-3"> · {rule.trigger_count} ingested</span>
                        )}
                      </p>
                    </div>
                    <Button
                      type="button"
                      size="icon-sm"
                      variant="ghost"
                      aria-label={`Delete watched folder ${rule.name}`}
                      className="max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
                      onClick={() => void handleDeleteTrigger(rule)}
                    >
                      <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                    </Button>
                  </li>
                );
              })}
            </ul>
          )}
        </TabsContent>

        {/* Tab 3: Outbound Actions & Delivery Log */}
        <TabsContent value="outbound" className="flex flex-col gap-4 pt-2">
          {policyProfile === "air_gapped" && (
            <div className="flex items-start gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg">
              <ShieldAlert className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
              <span>Air-Gapped workspace policy is active. All outbound connector deliveries are blocked.</span>
            </div>
          )}
          {policyProfile === "strict_client" && (
            <div className="flex items-start gap-2 rounded-sm border border-hairline bg-pill-warn-bg px-3 py-2 text-xs text-pill-warn-fg">
              <Lock className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
              <span>Strict Client policy is active. Every outbound delivery waits for your approval in Today before leaving your desk.</span>
            </div>
          )}

          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs text-ink-2">
              Credentials stay inside host storage and substitute <code className="mono">{"{{secret:key}}"}</code> only after policy checks pass.
            </span>
            <Button
              type="button"
              size="sm"
              variant={addingOutbound ? "outline" : "default"}
              className="max-md:min-h-11"
              onClick={() => setAddingOutbound((v) => !v)}
            >
              <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
              {addingOutbound ? "Close builder" : "New connector"}
            </Button>
          </div>

          {addingOutbound && (
            <form
              id="outbound-create-form"
              onSubmit={handleCreateOutbound}
              className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="flex flex-col gap-1">
                  <label htmlFor="outbound-name" className="text-xs font-medium text-ink">Connector name</label>
                  <Input
                    id="outbound-name"
                    value={outName}
                    onChange={(e) => setOutName(e.target.value)}
                    placeholder="Billing webhook"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="outbound-kind" className="text-xs font-medium text-ink">Connector kind</label>
                  <select
                    id="outbound-kind"
                    value={outKind}
                    onChange={(e) => setOutKind(e.target.value as OutboundConnectorKind)}
                    className="h-9 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    <option value="http_webhook">HTTP Webhook</option>
                    <option value="rest_api">REST API</option>
                    <option value="smtp_relay">SMTP Relay</option>
                  </select>
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="outbound-method" className="text-xs font-medium text-ink">HTTP method</label>
                  <select
                    id="outbound-method"
                    value={outMethod}
                    onChange={(e) => setOutMethod(e.target.value === "PUT" ? "PUT" : "POST")}
                    className="h-9 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                  </select>
                </div>
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="outbound-url" className="text-xs font-medium text-ink">Destination URL</label>
                <Input
                  id="outbound-url"
                  className="mono text-xs"
                  value={outUrl}
                  onChange={(e) => setOutUrl(e.target.value)}
                  placeholder="https://hooks.example.com/ergon"
                />
              </div>

              <div className="flex flex-col gap-1">
                <label htmlFor="outbound-headers" className="text-xs font-medium text-ink">
                  Headers template (one Header: Value per line)
                </label>
                <Textarea
                  id="outbound-headers"
                  rows={2}
                  className="mono text-xs"
                  value={outHeadersText}
                  onChange={(e) => setOutHeadersText(e.target.value)}
                  placeholder="Authorization: Bearer {{secret:api_token}}"
                />
              </div>

              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex flex-col gap-1">
                  <label htmlFor="outbound-secret-key" className="text-xs font-medium text-ink">Secret key name</label>
                  <Input
                    id="outbound-secret-key"
                    className="mono text-xs"
                    value={outSecretKey}
                    onChange={(e) => setOutSecretKey(e.target.value)}
                    placeholder="api_token"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="outbound-secret-value" className="text-xs font-medium text-ink">
                    Secret value (write-only, never read back)
                  </label>
                  <Input
                    id="outbound-secret-value"
                    type="password"
                    autoComplete="new-password"
                    value={outSecretValue}
                    onChange={(e) => setOutSecretValue(e.target.value)}
                    placeholder="Paste bearer token or relay password"
                  />
                </div>
              </div>

              <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                <input
                  id="outbound-require-approval"
                  type="checkbox"
                  checked={policyProfile === "strict_client" ? true : outRequireApproval}
                  disabled={policyProfile === "strict_client"}
                  onChange={(e) => setOutRequireApproval(e.target.checked)}
                  className="size-4 rounded border-input accent-primary"
                />
                <span>
                  Require approval in Today before sending
                  {policyProfile === "strict_client" ? " (required by Strict Client policy)" : ""}
                </span>
              </label>

              <div className="flex flex-wrap items-center justify-end gap-2">
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="max-md:min-h-11"
                  onClick={() => {
                    setOutSecretValue("");
                    setAddingOutbound(false);
                  }}
                >
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingOutbound}>
                  {savingOutbound ? "Saving…" : "Save connector"}
                </Button>
              </div>
            </form>
          )}

          {loading && outbound.length === 0 ? (
            <p className="py-4 text-sm text-ink-2">Loading outbound connectors…</p>
          ) : outbound.length === 0 ? (
            <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
              No outbound connectors configured yet. Add one to send approved deliverables or webhook events.
            </div>
          ) : (
            <ul className="flex flex-col gap-2.5">
              {outbound.map((c) => {
                const secretStored = Boolean(c.has_secret || (c.secret_keys && c.secret_keys.length > 0));
                const approvalRequired = Boolean(c.effective_require_approval ?? c.require_approval) || policyProfile === "strict_client";
                return (
                  <li
                    key={c.id}
                    data-outbound-id={c.id}
                    className="flex flex-col gap-2 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="flex flex-wrap items-start justify-between gap-2">
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-sm font-medium text-ink">{c.name}</span>
                          <Badge variant="outline" size="sm" className="text-xs">
                            {kindLabel(c.kind)}
                          </Badge>
                          <Pill
                            variant="secondary"
                            className={cn(
                              "px-2 py-0.5 text-xs",
                              c.blocked_by_policy || policyProfile === "air_gapped"
                                ? "bg-status-stopped-bg text-status-stopped-fg"
                                : approvalRequired
                                  ? "bg-pill-warn-bg text-pill-warn-fg"
                                  : "bg-pill-active-bg text-pill-active-fg"
                            )}
                          >
                            {c.blocked_by_policy || policyProfile === "air_gapped"
                              ? "Blocked by policy"
                              : approvalRequired
                                ? "Approval required"
                                : "Direct send"}
                          </Pill>
                          {secretStored && (
                            <Badge variant="secondary" size="sm" className="gap-1 text-xs">
                              <EyeOff className="size-3" strokeWidth={1.5} aria-hidden />
                              Secret stored
                            </Badge>
                          )}
                        </div>
                        <p className="mono mt-1 truncate text-xs text-ink-2">
                          {c.method} {c.url}
                        </p>
                      </div>
                      <div className="flex items-center gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          data-action="test-delivery"
                          disabled={testingOutboundId === c.id}
                          className="max-md:min-h-11"
                          onClick={() => void handleTestOutbound(c)}
                        >
                          <Send className="size-3.5" strokeWidth={1.5} aria-hidden />
                          {testingOutboundId === c.id ? "Testing…" : "Test Delivery"}
                        </Button>
                        <Button
                          type="button"
                          size="icon-sm"
                          variant="ghost"
                          aria-label={`Delete connector ${c.name}`}
                          className="max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
                          onClick={() => void handleDeleteOutbound(c)}
                        >
                          <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                        </Button>
                      </div>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}

          {/* Delivery Log */}
          <section id="connector-delivery-log" aria-labelledby="delivery-log-h" className="flex flex-col gap-2 border-t border-hairline pt-3">
            <div className="flex items-center justify-between gap-2">
              <h3 id="delivery-log-h" className="text-sm font-medium text-ink">Delivery log</h3>
              <span className="mono text-xs text-ink-3">{deliveries.length}</span>
            </div>
            {deliveries.length === 0 ? (
              <p className="text-xs text-ink-2">No outbound deliveries recorded yet.</p>
            ) : (
              <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                {deliveries.slice(0, 20).map((d) => {
                  const badge = deliveryStatusBadge(d.status);
                  const tsVal = typeof d.ts_ms === "number" ? d.ts_ms : typeof d.ts === "number" ? d.ts : 0;
                  const note = d.error || d.summary || d.response_preview || "";
                  const isHeld = d.status === "pending_approval" || d.status === "held";
                  return (
                    <li
                      key={d.id}
                      data-delivery-id={d.id}
                      className="flex flex-wrap items-center justify-between gap-2 px-3 py-2.5 text-xs"
                    >
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="font-medium text-ink">{d.connector_name || d.connector_id}</span>
                          <Pill variant="secondary" className={cn("px-2 py-0.5 text-xs", badge.cls)}>
                            {badge.label}
                          </Pill>
                          {typeof d.http_status === "number" && (
                            <span className="mono text-ink-2">HTTP {d.http_status}</span>
                          )}
                          {d.test && (
                            <Badge variant="outline" size="sm" className="text-xs">Test</Badge>
                          )}
                        </div>
                        {note && (
                          <p className="mt-0.5 text-ink-2 [overflow-wrap:anywhere]">{note}</p>
                        )}
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        {isHeld && onOpenToday && (
                          <Button
                            type="button"
                            size="xs"
                            variant="outline"
                            className="max-md:min-h-11"
                            onClick={() => {
                              setIsOpen(false);
                              onOpenToday();
                            }}
                          >
                            Review in Today
                          </Button>
                        )}
                        <span className="mono text-ink-3">{tsVal ? relTime(tsVal) : ""}</span>
                      </div>
                    </li>
                  );
                })}
              </ul>
            )}
          </section>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export function ConnectorsPanel({
  open,
  onOpenChange,
  routines,
  status,
  onChanged,
  onOpenToday,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  routines: Routine[];
  status: Status | null | "error";
  onChanged?: () => void;
  onOpenToday?: () => void;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Connectors"
      description="Configure inbound webhooks, watched folders, and credential-isolated outbound actions."
      panelId="connectors-panel"
    >
      <ConnectorsBody
        routines={routines}
        status={status}
        onChanged={onChanged}
        onOpenToday={onOpenToday}
      />
    </PanelModal>
  );
}
