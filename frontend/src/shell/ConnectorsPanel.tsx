// Connectors configuration dialog (always mounted OUTSIDE Vaul Drawer):
// 1. Inbound Webhooks: create signed hooks, copy endpoint URL, rotate signing secret (one-time reveal), simulate payload
// 2. Watched Folders & IMAP Inbox: monitor host directory or poll IMAP mailbox -> target routine
// 3. Outbound Actions: HTTP webhook, REST API, SMTP Relay (smtp_config), optional OAuth2 credentials,
//    write-only secret handling, Test Delivery button, policy gate awareness, and Delivery Log
// 4. MCP Tools: configure stdio / http_sse MCP servers, discover cached tools, toggle approval gate, delete
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  Check,
  Copy,
  EyeOff,
  FolderInput,
  KeyRound,
  Lock,
  Mail,
  Play,
  Plug,
  Plus,
  RefreshCw,
  RotateCw,
  Send,
  ShieldAlert,
  Trash2,
  Webhook,
  Wrench,
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
  deleteMcpServer,
  deleteOutboundConnector,
  deleteTriggerRule,
  discoverMcpServer,
  hasMcpApi,
  listConnectors,
  listMcpServers,
  rotateTriggerSecret,
  saveMcpServer,
  saveOutboundConnector,
  saveTriggerRule,
  simulateWebhookTrigger,
  testOutboundConnector,
  type ConnectorSecretMeta,
  type DeliveryLog,
  type ImapConfig,
  type ImapSecurity,
  type McpServerSpec,
  type McpTransport,
  type OAuthConfig,
  type OutboundConnector,
  type OutboundConnectorKind,
  type Routine,
  type SmtpConfig,
  type SmtpSecurity,
  type Status,
  type TriggerRule,
  type WorkspacePolicy,
  type WorkspacePolicyProfile,
} from "@/lib/api";
import { cn } from "@/lib/utils";
import { relTime } from "@/lib/words";

type ConnectorTab = "webhooks" | "folders" | "outbound" | "mcp";

const SECRET_KEY_RE = /^[a-zA-Z0-9_.-]+$/;

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

function validateHeadersText(raw: string): string | null {
  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const idx = trimmed.indexOf(":");
    if (idx <= 0) {
      return `Invalid header line "${trimmed.slice(0, 40)}". Use "Header-Name: value" format.`;
    }
    const key = trimmed.slice(0, idx).trim();
    if (!/^[A-Za-z0-9-]+$/.test(key)) {
      return `Invalid header name "${key}".`;
    }
  }
  return null;
}

function validateWatchDir(raw: string): string | null {
  const dir = raw.trim();
  if (!dir) return "Enter a host directory path to watch.";
  if (/[\0\r\n]/.test(dir)) return "Directory path contains invalid control characters.";
  const norm = dir.replace(/\\/g, "/");
  if (/(^|\/)\.\.(\/|$)/.test(norm)) {
    return "Directory path cannot contain '..' traversal segments.";
  }
  const isAbsUnix = norm.startsWith("/");
  const isAbsWin = /^[A-Za-z]:\//.test(norm);
  if (!isAbsUnix && !isAbsWin) {
    return "Enter an absolute host directory path (for example /home/you/Documents/Inbox).";
  }
  const low = norm.replace(/\/+$/, "").toLowerCase() || "/";
  if (
    low === "/" ||
    low === "/etc" ||
    low.startsWith("/etc/") ||
    low === "/root" ||
    low.startsWith("/root/") ||
    low === "/proc" ||
    low.startsWith("/proc/") ||
    low === "/sys" ||
    low.startsWith("/sys/") ||
    low === "/dev" ||
    low.startsWith("/dev/") ||
    low === "/var/run" ||
    low.startsWith("/var/run/") ||
    low.includes("/.ssh") ||
    low.includes("/.aws") ||
    low.includes("/.gnupg") ||
    low.includes("/.talos-wallet") ||
    low.includes("/.credentials") ||
    low.includes("/.env")
  ) {
    return "Sensitive system or credential directories cannot be watched.";
  }
  return null;
}

function validateHttpUrl(raw: string, label: string): string | null {
  const url = raw.trim();
  if (!url) return `Enter a ${label}.`;
  if (/[\s\0\r\n]/.test(url)) return `${label} cannot contain spaces or control characters.`;
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return `${label} must start with http:// or https://.`;
    }
    if (!parsed.hostname) return `${label} must include a valid host.`;
    if (parsed.username || parsed.password) {
      return `Do not embed credentials in ${label}; use a secret key reference.`;
    }
    return null;
  } catch {
    return `Enter a valid http:// or https:// ${label}.`;
  }
}

function validateHost(raw: string, label: string): string | null {
  const host = raw.trim();
  if (!host) return `Enter a ${label}.`;
  if (/[\s\0\r\n/@]/.test(host) || host.startsWith(".") || host.endsWith(".")) {
    return `${label} must be a valid hostname or IP address.`;
  }
  return null;
}

function parsePort(raw: string): number | null {
  const n = Number(raw.trim());
  if (!Number.isInteger(n) || n < 1 || n > 65535) return null;
  return n;
}

function parseListTokens(raw: string): string[] {
  return raw
    .split(/[\s,]+/)
    .map((s) => s.trim())
    .filter(Boolean);
}

function parseCommaList(raw: string): string[] {
  return raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

function defaultSmtpPort(security: SmtpSecurity): string {
  if (security === "starttls") return "587";
  if (security === "plain_loopback_test") return "2525";
  return "465";
}

function defaultImapPort(security: ImapSecurity): string {
  if (security === "starttls") return "143";
  if (security === "plain_loopback_test") return "1143";
  return "993";
}

function validateOutboundUrl(raw: string, kind: OutboundConnectorKind): string | null {
  const url = raw.trim();
  if (!url) return "Enter a destination URL.";
  if (/[\s\0\r\n]/.test(url)) return "Destination URL cannot contain spaces or control characters.";
  if (kind === "smtp_relay") {
    if (/^smtps?:\/\//i.test(url)) {
      try {
        const parsed = new URL(url);
        if (!parsed.hostname) return "SMTP relay URL must include a hostname.";
        if (parsed.username || parsed.password) {
          return "Do not embed credentials in the URL; use the write-only secret field below.";
        }
        return null;
      } catch {
        return "Enter a valid SMTP relay URL (for example smtps://smtp.example.com:465).";
      }
    }
    if (/^[a-zA-Z0-9.-]+(?::\d{1,5})?$/.test(url) && !url.startsWith(".") && !url.endsWith(".")) {
      return null;
    }
    return "Enter a valid SMTP relay host or smtps:// URL (for example smtps://smtp.example.com:465).";
  }
  return validateHttpUrl(url, "Destination URL");
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
  const [loadErr, setLoadErr] = useState("");
  const [triggers, setTriggers] = useState<TriggerRule[]>([]);
  const [outbound, setOutbound] = useState<OutboundConnector[]>([]);
  const [deliveries, setDeliveries] = useState<DeliveryLog[]>([]);
  const [storedSecrets, setStoredSecrets] = useState<ConnectorSecretMeta[]>([]);
  const [policyProfile, setPolicyProfile] = useState<WorkspacePolicyProfile>("fast_internal");

  // Transient one-time signing secrets created or rotated during this modal session.
  const [ephemeralSecrets, setEphemeralSecrets] = useState<Record<string, string>>({});
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Inbound webhook builder & simulator state
  const [addingHook, setAddingHook] = useState(false);
  const [hookName, setHookName] = useState("");
  const [hookRoutineId, setHookRoutineId] = useState("");
  const [hookFormErr, setHookFormErr] = useState("");
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
  const [folderFormErr, setFolderFormErr] = useState("");
  const [savingFolder, setSavingFolder] = useState(false);

  // IMAP Inbox trigger builder state
  const [addingImap, setAddingImap] = useState(false);
  const [imapName, setImapName] = useState("");
  const [imapRoutineId, setImapRoutineId] = useState("");
  const [imapHost, setImapHost] = useState("");
  const [imapPort, setImapPort] = useState("993");
  const [imapSecurity, setImapSecurity] = useState<ImapSecurity>("imaps");
  const [imapFolder, setImapFolder] = useState("INBOX");
  const [imapPollInterval, setImapPollInterval] = useState("60");
  const [imapMarkSeen, setImapMarkSeen] = useState(true);
  const [imapSubjectFilter, setImapSubjectFilter] = useState("");
  const [imapUserKey, setImapUserKey] = useState("imap_user");
  const [imapPassKey, setImapPassKey] = useState("imap_pass");
  const [imapFormErr, setImapFormErr] = useState("");
  const [savingImap, setSavingImap] = useState(false);

  // Outbound connector builder state (secret values are strictly write-only and cleared immediately)
  const [addingOutbound, setAddingOutbound] = useState(false);
  const [outName, setOutName] = useState("");
  const [outKind, setOutKind] = useState<OutboundConnectorKind>("http_webhook");
  const [outMethod, setOutMethod] = useState<"POST" | "PUT">("POST");
  const [outUrl, setOutUrl] = useState("");
  const [outHeadersText, setOutHeadersText] = useState("Authorization: Bearer {{secret:api_token}}");
  const [outSecretKey, setOutSecretKey] = useState("api_token");
  const [outSecretValue, setOutSecretValue] = useState("");
  const [outRequireApproval, setOutRequireApproval] = useState(true);
  const [outboundFormErr, setOutboundFormErr] = useState("");
  const [savingOutbound, setSavingOutbound] = useState(false);
  const [testingOutboundId, setTestingOutboundId] = useState<string | null>(null);

  // SMTP Relay builder state
  const [smtpHost, setSmtpHost] = useState("");
  const [smtpPort, setSmtpPort] = useState("465");
  const [smtpSecurity, setSmtpSecurity] = useState<SmtpSecurity>("smtps");
  const [smtpFromAddr, setSmtpFromAddr] = useState("");
  const [smtpDefaultTo, setSmtpDefaultTo] = useState("");
  const [smtpUserKey, setSmtpUserKey] = useState("smtp_user");
  const [smtpUserSecret, setSmtpUserSecret] = useState("");
  const [smtpPassKey, setSmtpPassKey] = useState("smtp_pass");
  const [smtpPassSecret, setSmtpPassSecret] = useState("");

  // OAuth2 Credentials config helper state
  const [oauthEnabled, setOauthEnabled] = useState(false);
  const [oauthTokenUrl, setOauthTokenUrl] = useState("");
  const [oauthClientId, setOauthClientId] = useState("");
  const [oauthScopes, setOauthScopes] = useState("");
  const [oauthClientSecretKey, setOauthClientSecretKey] = useState("oauth_client_secret");
  const [oauthClientSecretValue, setOauthClientSecretValue] = useState("");
  const [oauthRefreshKey, setOauthRefreshKey] = useState("oauth_refresh_token");
  const [oauthRefreshValue, setOauthRefreshValue] = useState("");

  // MCP Tools tab state
  const mcpSupported = hasMcpApi(status);
  const [mcpServers, setMcpServers] = useState<McpServerSpec[]>([]);
  const [mcpLoadErr, setMcpLoadErr] = useState("");
  const [addingMcp, setAddingMcp] = useState(false);
  const [mcpName, setMcpName] = useState("");
  const [mcpTransport, setMcpTransport] = useState<McpTransport>("stdio");
  const [mcpCommandOrUrl, setMcpCommandOrUrl] = useState("");
  const [mcpArgsText, setMcpArgsText] = useState("");
  const [mcpEnvSecretKeys, setMcpEnvSecretKeys] = useState("");
  const [mcpRequireApproval, setMcpRequireApproval] = useState(true);
  const [mcpEnabled, setMcpEnabled] = useState(true);
  const [mcpFormErr, setMcpFormErr] = useState("");
  const [savingMcp, setSavingMcp] = useState(false);
  const [discoveringMcpId, setDiscoveringMcpId] = useState<string | null>(null);
  const [togglingMcpId, setTogglingMcpId] = useState<string | null>(null);

  const wipeOutboundWriteOnlySecrets = useCallback(() => {
    setOutSecretValue("");
    setSmtpUserSecret("");
    setSmtpPassSecret("");
    setOauthClientSecretValue("");
    setOauthRefreshValue("");
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setErr("");
    setLoadErr("");
    setMcpLoadErr("");
    const [res, polRes, mcpRes] = await Promise.all([
      listConnectors(),
      call<WorkspacePolicy>("/api/workspace/policy"),
      mcpSupported ? listMcpServers() : Promise.resolve(null),
    ]);
    setLoading(false);
    if (!res.ok) {
      const msg = res.error || "Could not load connectors and delivery history.";
      setErr(msg);
      setLoadErr(msg);
    } else {
      setTriggers(res.data.triggers);
      setOutbound(res.data.outbound);
      setDeliveries(res.data.deliveries);
      setStoredSecrets(Array.isArray(res.data.secrets) ? res.data.secrets : []);
      if (res.data.policy_profile) {
        setPolicyProfile(res.data.policy_profile);
      } else if (polRes.ok && polRes.data.profile) {
        setPolicyProfile(polRes.data.profile);
      }
    }
    if (mcpRes) {
      if (mcpRes.ok) {
        setMcpServers(mcpRes.data.servers);
      } else if (mcpRes.status !== 404) {
        setMcpLoadErr(mcpRes.error || "Could not load MCP servers.");
      }
    } else {
      setMcpServers([]);
    }
  }, [mcpSupported]);

  useEffect(() => {
    if (!isOpen) {
      // Wipe any transient secret state as soon as the modal closes.
      wipeOutboundWriteOnlySecrets();
      setSimTokens({});
      setEphemeralSecrets({});
      return;
    }
    void load();
  }, [isOpen, load, status, wipeOutboundWriteOnlySecrets]);

  useEffect(() => {
    if (routines.length > 0) {
      if (!hookRoutineId) setHookRoutineId(routines[0].id);
      if (!folderRoutineId) setFolderRoutineId(routines[0].id);
      if (!imapRoutineId) setImapRoutineId(routines[0].id);
    }
  }, [routines, hookRoutineId, folderRoutineId, imapRoutineId]);

  const routineNameMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const r of routines) map.set(r.id, r.name);
    return map;
  }, [routines]);

  const webhookTriggers = useMemo(
    () => triggers.filter((t) => t.kind === "webhook"),
    [triggers]
  );
  const folderAndImapTriggers = useMemo(
    () => triggers.filter((t) => t.kind === "folder_watch" || t.kind === "imap_poll"),
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
      setHookFormErr("Enter a webhook name and choose a target routine.");
      return;
    }
    setSavingHook(true);
    setHookFormErr("");
    setErr("");
    const r = await saveTriggerRule({
      name,
      kind: "webhook",
      routine_id: hookRoutineId,
      enabled: true,
    });
    setSavingHook(false);
    if (!r.ok) {
      setHookFormErr(r.error || "Could not create inbound webhook.");
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
    const token = (simTokens[rule.id] ?? "").trim() || ephemeralSecrets[rule.id] || "";
    // Always wipe user-entered signing secret from React state on submit.
    setSimTokens((prev) => {
      if (!(rule.id in prev)) return prev;
      const next = { ...prev };
      delete next[rule.id];
      return next;
    });
    if (rawPayload.trim()) {
      try {
        JSON.parse(rawPayload);
      } catch {
        setSimResultById((prev) => ({
          ...prev,
          [rule.id]: { ok: false, text: "Test payload must be valid JSON." },
        }));
        return;
      }
    }
    setSimBusyId(rule.id);
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
    const glob = folderGlob.trim() || "*.*";
    if (!name || !folderRoutineId) {
      setFolderFormErr("Enter a rule name and choose a target routine.");
      return;
    }
    const dirErr = validateWatchDir(dir);
    if (dirErr) {
      setFolderFormErr(dirErr);
      return;
    }
    if (/[\0\r\n/\\]/.test(glob) || glob.includes("..")) {
      setFolderFormErr("Glob filter cannot contain path separators or '..'.");
      return;
    }
    setSavingFolder(true);
    setFolderFormErr("");
    setErr("");
    const r = await saveTriggerRule({
      name,
      kind: "folder_watch",
      watch_dir: dir,
      glob,
      routine_id: folderRoutineId,
      enabled: true,
    });
    setSavingFolder(false);
    if (!r.ok) {
      setFolderFormErr(r.error || "Could not create watched folder rule.");
      return;
    }
    setFolderName("");
    setFolderPath("");
    setFolderGlob("*.pdf,*.csv");
    setAddingFolder(false);
    toast.success(`Watching "${dir}" for ${glob}.`);
    await load();
    onChanged?.();
  };

  const handleCreateImap = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = imapName.trim();
    const host = imapHost.trim();
    const folder = imapFolder.trim() || "INBOX";
    const port = parsePort(imapPort);
    const pollSecs = Number(imapPollInterval.trim());
    const userKey = imapUserKey.trim();
    const passKey = imapPassKey.trim();
    const subjectFilter = imapSubjectFilter.trim();

    if (!name || !imapRoutineId) {
      setImapFormErr("Enter a trigger name and choose a target routine.");
      return;
    }
    const hostErr = validateHost(host, "IMAP server host");
    if (hostErr) {
      setImapFormErr(hostErr);
      return;
    }
    if (!port) {
      setImapFormErr("IMAP port must be a valid port between 1 and 65535.");
      return;
    }
    if (!Number.isInteger(pollSecs) || pollSecs < 5) {
      setImapFormErr("Poll interval must be at least 5 seconds.");
      return;
    }
    if (!userKey || !SECRET_KEY_RE.test(userKey) || !passKey || !SECRET_KEY_RE.test(passKey)) {
      setImapFormErr("Username and password secret keys must use letters, numbers, underscores, dots, or dashes.");
      return;
    }

    setSavingImap(true);
    setImapFormErr("");
    setErr("");
    const imap_config: ImapConfig = {
      host,
      port,
      security: imapSecurity,
      folder,
      poll_interval_secs: pollSecs,
      mark_seen: imapMarkSeen,
      username_secret_key: userKey,
      password_secret_key: passKey,
      ...(subjectFilter ? { subject_filter: subjectFilter } : {}),
    };
    const r = await saveTriggerRule({
      name,
      kind: "imap_poll",
      routine_id: imapRoutineId,
      enabled: true,
      imap_config,
    });
    setSavingImap(false);
    if (!r.ok) {
      setImapFormErr(r.error || "Could not save IMAP inbox trigger.");
      return;
    }
    setImapName("");
    setImapHost("");
    setImapSubjectFilter("");
    setAddingImap(false);
    toast.success(`Polling "${folder}" on ${host} into routine.`);
    await load();
    onChanged?.();
  };

  const handleCreateOutbound = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = outName.trim();
    const skey = outSecretKey.trim();
    const sval = outSecretValue.trim();
    const sUserVal = smtpUserSecret.trim();
    const sPassVal = smtpPassSecret.trim();
    const oaClientSecVal = oauthClientSecretValue.trim();
    const oaRefreshVal = oauthRefreshValue.trim();
    // Strictly wipe all plaintext secret inputs from React state immediately on submit.
    wipeOutboundWriteOnlySecrets();

    if (!name) {
      setOutboundFormErr("Enter a connector name.");
      return;
    }

    const secrets: Record<string, string> = {};
    let resolvedUrl = outUrl.trim();
    let smtpConfig: SmtpConfig | undefined;
    let oauthConfig: OAuthConfig | undefined;

    if (outKind === "smtp_relay") {
      const host = smtpHost.trim();
      const port = parsePort(smtpPort);
      const fromAddr = smtpFromAddr.trim();
      const uKey = smtpUserKey.trim();
      const pKey = smtpPassKey.trim();
      const hostErr = validateHost(host, "SMTP host");
      if (hostErr) {
        setOutboundFormErr(hostErr);
        return;
      }
      if (!port) {
        setOutboundFormErr("SMTP port must be a valid port between 1 and 65535.");
        return;
      }
      if (!fromAddr || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(fromAddr)) {
        setOutboundFormErr("Enter a valid sender email address (for example billing@example.com).");
        return;
      }
      if (!uKey || !SECRET_KEY_RE.test(uKey) || !pKey || !SECRET_KEY_RE.test(pKey)) {
        setOutboundFormErr("SMTP secret key names must use letters, numbers, underscores, dots, or dashes.");
        return;
      }
      const defaultTo = parseCommaList(smtpDefaultTo);
      if (defaultTo.some((addr) => !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(addr))) {
        setOutboundFormErr("Each default recipient must be a valid email address.");
        return;
      }
      smtpConfig = {
        host,
        port,
        security: smtpSecurity,
        username_secret_key: uKey,
        password_secret_key: pKey,
        from_addr: fromAddr,
        default_to: defaultTo,
      };
      if (!resolvedUrl) {
        const scheme = smtpSecurity === "smtps" ? "smtps" : "smtp";
        resolvedUrl = `${scheme}://${host}:${port}`;
      } else {
        const urlErr = validateOutboundUrl(resolvedUrl, outKind);
        if (urlErr) {
          setOutboundFormErr(urlErr);
          return;
        }
      }
      if (sUserVal) secrets[uKey] = sUserVal;
      if (sPassVal) secrets[pKey] = sPassVal;
    } else {
      const urlErr = validateOutboundUrl(resolvedUrl, outKind);
      if (urlErr) {
        setOutboundFormErr(urlErr);
        return;
      }
      const hdrErr = validateHeadersText(outHeadersText);
      if (hdrErr) {
        setOutboundFormErr(hdrErr);
        return;
      }
      if (sval && (!skey || !SECRET_KEY_RE.test(skey))) {
        setOutboundFormErr("Secret key name must use letters, numbers, underscores, dots, or dashes.");
        return;
      }
      if (skey && sval) {
        secrets[skey] = sval;
      }
    }

    if (oauthEnabled) {
      const tokenUrlErr = validateHttpUrl(oauthTokenUrl, "OAuth2 token URL");
      if (tokenUrlErr) {
        setOutboundFormErr(tokenUrlErr);
        return;
      }
      const clientId = oauthClientId.trim();
      const cSecKey = oauthClientSecretKey.trim();
      const rTokKey = oauthRefreshKey.trim();
      if (!clientId) {
        setOutboundFormErr("Enter an OAuth2 client ID.");
        return;
      }
      if (!cSecKey || !SECRET_KEY_RE.test(cSecKey) || !rTokKey || !SECRET_KEY_RE.test(rTokKey)) {
        setOutboundFormErr("OAuth2 secret key references must use letters, numbers, underscores, dots, or dashes.");
        return;
      }
      oauthConfig = {
        token_url: oauthTokenUrl.trim(),
        client_id: clientId,
        client_secret_key: cSecKey,
        refresh_token_secret_key: rTokKey,
        scopes: parseListTokens(oauthScopes),
      };
      if (oaClientSecVal) secrets[cSecKey] = oaClientSecVal;
      if (oaRefreshVal) secrets[rTokKey] = oaRefreshVal;
    }

    setSavingOutbound(true);
    setOutboundFormErr("");
    setErr("");
    const headers_template = outKind === "smtp_relay" ? {} : parseHeadersText(outHeadersText);
    const r = await saveOutboundConnector({
      name,
      kind: outKind,
      url: resolvedUrl,
      method: outMethod,
      headers_template,
      secrets: Object.keys(secrets).length > 0 ? secrets : undefined,
      ...(smtpConfig ? { smtp_config: smtpConfig } : {}),
      ...(oauthConfig ? { oauth_config: oauthConfig } : {}),
      require_approval: policyProfile === "strict_client" ? true : outRequireApproval,
      enabled: true,
    });
    setSavingOutbound(false);
    if (!r.ok) {
      setOutboundFormErr(r.error || "Could not save outbound connector.");
      return;
    }
    setOutName("");
    setOutUrl("");
    setSmtpHost("");
    setSmtpFromAddr("");
    setSmtpDefaultTo("");
    setOauthEnabled(false);
    setOauthTokenUrl("");
    setOauthClientId("");
    setOauthScopes("");
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

  const handleCreateMcpServer = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = mcpName.trim();
    const commandOrUrl = mcpCommandOrUrl.trim();
    if (!name) {
      setMcpFormErr("Enter a server name.");
      return;
    }
    if (mcpTransport === "http_sse") {
      const urlErr = validateHttpUrl(commandOrUrl, "MCP SSE URL");
      if (urlErr) {
        setMcpFormErr(urlErr);
        return;
      }
    } else if (!commandOrUrl || /[\0\r\n]/.test(commandOrUrl)) {
      setMcpFormErr("Enter a valid executable command for stdio transport.");
      return;
    }
    const envKeys = parseCommaList(mcpEnvSecretKeys);
    if (envKeys.some((k) => !SECRET_KEY_RE.test(k))) {
      setMcpFormErr("Environment secret keys must use letters, numbers, underscores, dots, or dashes.");
      return;
    }
    const args = mcpArgsText
      .trim()
      .split(/\s+/)
      .map((s) => s.trim())
      .filter(Boolean);

    setSavingMcp(true);
    setMcpFormErr("");
    setErr("");
    const r = await saveMcpServer({
      name,
      transport: mcpTransport,
      command_or_url: commandOrUrl,
      args,
      env_secret_keys: envKeys,
      require_approval_by_default: policyProfile === "strict_client" ? true : mcpRequireApproval,
      enabled: mcpEnabled,
    });
    setSavingMcp(false);
    if (!r.ok) {
      setMcpFormErr(r.error || "Could not save MCP server.");
      return;
    }
    setMcpName("");
    setMcpCommandOrUrl("");
    setMcpArgsText("");
    setMcpEnvSecretKeys("");
    setAddingMcp(false);
    toast.success(`Added MCP server "${name}".`);
    await load();
    onChanged?.();
  };

  const handleDiscoverMcp = async (srv: McpServerSpec) => {
    setDiscoveringMcpId(srv.id);
    setErr("");
    const r = await discoverMcpServer(srv.id);
    setDiscoveringMcpId(null);
    if (!r.ok) {
      toast.error(r.error || `Could not discover tools from "${srv.name}".`);
      return;
    }
    const tools = r.data.server?.cached_tools ?? r.data.tools ?? r.data.cached_tools ?? [];
    toast.success(`Discovered ${tools.length} tool${tools.length === 1 ? "" : "s"} on "${srv.name}".`);
    await load();
    onChanged?.();
  };

  const handleToggleMcpApproval = async (srv: McpServerSpec) => {
    if (policyProfile === "strict_client" && srv.require_approval_by_default) {
      toast.warning("Strict Client policy requires approval for all MCP tool calls.");
      return;
    }
    setTogglingMcpId(srv.id);
    const nextRequire = !srv.require_approval_by_default;
    const r = await saveMcpServer({
      id: srv.id,
      name: srv.name,
      transport: srv.transport,
      command_or_url: srv.command_or_url,
      args: srv.args ?? [],
      env_secret_keys: srv.env_secret_keys ?? [],
      require_approval_by_default: nextRequire,
      enabled: srv.enabled,
    });
    setTogglingMcpId(null);
    if (!r.ok) {
      toast.error(r.error || `Could not update approval setting for "${srv.name}".`);
      return;
    }
    toast.success(
      nextRequire
        ? `Tool calls on "${srv.name}" now require approval.`
        : `Tool calls on "${srv.name}" can run without prior approval.`
    );
    await load();
    onChanged?.();
  };

  const handleDeleteMcp = async (srv: McpServerSpec) => {
    const r = await deleteMcpServer(srv.id);
    if (!r.ok) {
      toast.error(r.error || `Could not remove MCP server "${srv.name}".`);
      return;
    }
    toast.success(`Removed MCP server "${srv.name}".`);
    await load();
    onChanged?.();
  };

  return (
    <div className="flex min-w-0 w-full max-w-full flex-col gap-4 pt-1">
      <p className="text-sm text-ink-2">
        Receive signed webhooks, watched host files, or IMAP emails into a routine, send outbound payloads, and inspect MCP tool servers.
      </p>

      {err && (
        <div role="alert" className="flex items-start gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg">
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
          <span>{err}</span>
        </div>
      )}

      <Tabs
        value={tab}
        className="min-w-0 w-full"
        onValueChange={(v) => {
          setErr("");
          setHookFormErr("");
          setFolderFormErr("");
          setImapFormErr("");
          setOutboundFormErr("");
          setMcpFormErr("");
          wipeOutboundWriteOnlySecrets();
          setTab(v as ConnectorTab);
        }}
      >
        <TabsList aria-label="Connector categories" className="grid h-auto w-full min-w-0 grid-cols-2 gap-1 p-1 sm:flex sm:w-fit sm:max-w-full sm:flex-wrap">
          <TabsTrigger value="webhooks" data-id="webhooks" className="min-w-0 w-full sm:w-auto justify-center max-md:min-h-11 gap-1 px-1.5 text-xs sm:gap-1.5 sm:px-2.5 sm:text-sm">
            <Webhook className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Hooks</span>
            <span className="hidden sm:inline">Inbound Webhooks</span>
            <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{webhookTriggers.length}</span>
          </TabsTrigger>
          <TabsTrigger value="folders" data-id="folders" className="min-w-0 w-full sm:w-auto justify-center max-md:min-h-11 gap-1 px-1.5 text-xs sm:gap-1.5 sm:px-2.5 sm:text-sm">
            <FolderInput className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Folders</span>
            <span className="hidden sm:inline">Folders & IMAP</span>
            <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{folderAndImapTriggers.length}</span>
          </TabsTrigger>
          <TabsTrigger value="outbound" data-id="outbound" className="min-w-0 w-full sm:w-auto justify-center max-md:min-h-11 gap-1 px-1.5 text-xs sm:gap-1.5 sm:px-2.5 sm:text-sm">
            <Send className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">Outbound</span>
            <span className="hidden sm:inline">Outbound Actions</span>
            <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{outbound.length}</span>
          </TabsTrigger>
          <TabsTrigger value="mcp" data-id="mcp" className="min-w-0 w-full sm:w-auto justify-center max-md:min-h-11 gap-1 px-1.5 text-xs sm:gap-1.5 sm:px-2.5 sm:text-sm">
            <Plug className="hidden size-3.5 shrink-0 sm:inline" strokeWidth={1.5} aria-hidden />
            <span className="sm:hidden">MCP</span>
            <span className="hidden sm:inline">MCP Tools</span>
            <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">{mcpServers.length}</span>
          </TabsTrigger>
        </TabsList>

        {/* Tab 1: Inbound Webhooks */}
        <TabsContent value="webhooks" className="flex min-w-0 w-full flex-col gap-3 pt-2">
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
              className="flex min-w-0 w-full flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="webhook-name" className="text-xs font-medium text-ink">Webhook name</label>
                  <Input
                    id="webhook-name"
                    value={hookName}
                    onChange={(e) => setHookName(e.target.value)}
                    placeholder="Stripe invoice intake"
                    className="w-full min-w-0"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="webhook-routine" className="text-xs font-medium text-ink">Target routine</label>
                  <select
                    id="webhook-routine"
                    value={hookRoutineId}
                    onChange={(e) => setHookRoutineId(e.target.value)}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
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
              {hookFormErr && (
                <p id="webhook-form-error" role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg [overflow-wrap:anywhere]">
                  {hookFormErr}
                </p>
              )}
              <div className="sticky bottom-0 z-10 -mx-3.5 -mb-3.5 mt-1 flex flex-wrap items-center justify-end gap-2 rounded-b-sm border-t border-hairline bg-surface/95 px-3.5 py-2.5 backdrop-blur-xs">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => { setHookFormErr(""); setAddingHook(false); }}>
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
            <ul className="flex min-w-0 w-full flex-col gap-2.5">
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
                    className="flex min-w-0 w-full max-w-full flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="flex min-w-0 w-full flex-wrap items-start justify-between gap-2">
                      <div className="min-w-[min(100%,14rem)] flex-1 overflow-hidden">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{rule.name}</span>
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
                          Triggers <span className="font-medium text-ink">{targetName}</span>
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

                    <div className="flex min-w-0 w-full flex-wrap items-center gap-2 rounded-sm border border-hairline bg-board/50 px-2.5 py-1.5">
                      <code className="mono min-w-0 max-w-full flex-1 break-all text-xs text-ink">{url}</code>
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

        {/* Tab 2: Watched Folders & IMAP Inbox */}
        <TabsContent value="folders" className="flex min-w-0 w-full flex-col gap-3 pt-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs text-ink-2">
              Ingest new files from an approved host folder or poll an IMAP inbox into a routine.
            </span>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="button"
                size="sm"
                variant={addingFolder ? "outline" : "default"}
                className="max-md:min-h-11"
                onClick={() => {
                  setAddingImap(false);
                  setAddingFolder((v) => !v);
                }}
              >
                <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
                {addingFolder ? "Close folder form" : "Watch folder"}
              </Button>
              <Button
                id="imap-add-btn"
                type="button"
                size="sm"
                variant={addingImap ? "outline" : "secondary"}
                className="max-md:min-h-11"
                onClick={() => {
                  setAddingFolder(false);
                  setAddingImap((v) => !v);
                }}
              >
                <Mail className="size-3.5" strokeWidth={1.5} aria-hidden />
                {addingImap ? "Close IMAP form" : "New IMAP inbox"}
              </Button>
            </div>
          </div>

          {addingFolder && (
            <form
              id="folder-watch-form"
              onSubmit={handleCreateFolderWatch}
              className="flex min-w-0 w-full flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="folder-watch-name" className="text-xs font-medium text-ink">Rule name</label>
                  <Input
                    id="folder-watch-name"
                    value={folderName}
                    onChange={(e) => setFolderName(e.target.value)}
                    placeholder="Vendor statements dropbox"
                    className="w-full min-w-0"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="folder-watch-routine" className="text-xs font-medium text-ink">Target routine</label>
                  <select
                    id="folder-watch-routine"
                    value={folderRoutineId}
                    onChange={(e) => setFolderRoutineId(e.target.value)}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
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
                <div className="flex min-w-0 flex-col gap-1 sm:col-span-2">
                  <label htmlFor="folder-watch-path" className="text-xs font-medium text-ink">Host directory path</label>
                  <Input
                    id="folder-watch-path"
                    className="mono w-full min-w-0 text-xs"
                    value={folderPath}
                    onChange={(e) => setFolderPath(e.target.value)}
                    placeholder="/home/you/Documents/VendorInbox"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="folder-watch-glob" className="text-xs font-medium text-ink">Glob filter</label>
                  <Input
                    id="folder-watch-glob"
                    className="mono w-full min-w-0 text-xs"
                    value={folderGlob}
                    onChange={(e) => setFolderGlob(e.target.value)}
                    placeholder="*.pdf,*.csv"
                  />
                </div>
              </div>
              <p className="text-xs text-ink-2">
                Sensitive system and credential folders are blocked automatically. Only files under 25 MB with steady size and timestamp are ingested.
              </p>
              {folderFormErr && (
                <p id="folder-watch-error" role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg [overflow-wrap:anywhere]">
                  {folderFormErr}
                </p>
              )}
              <div className="sticky bottom-0 z-10 -mx-3.5 -mb-3.5 mt-1 flex flex-wrap items-center justify-end gap-2 rounded-b-sm border-t border-hairline bg-surface/95 px-3.5 py-2.5 backdrop-blur-xs">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => { setFolderFormErr(""); setAddingFolder(false); }}>
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingFolder || routines.length === 0}>
                  {savingFolder ? "Saving…" : "Save watched folder"}
                </Button>
              </div>
            </form>
          )}

          {addingImap && (
            <form
              id="imap-poll-form"
              onSubmit={handleCreateImap}
              className="flex min-w-0 w-full flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="flex items-center gap-2 text-xs font-medium text-ink">
                <Mail className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                <span>IMAP Inbox trigger</span>
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-name" className="text-xs font-medium text-ink">Trigger name</label>
                  <Input
                    id="imap-name"
                    value={imapName}
                    onChange={(e) => setImapName(e.target.value)}
                    placeholder="Client invoices inbox"
                    className="w-full min-w-0"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-routine" className="text-xs font-medium text-ink">Target routine</label>
                  <select
                    id="imap-routine"
                    value={imapRoutineId}
                    onChange={(e) => setImapRoutineId(e.target.value)}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
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
              <div className="flex flex-wrap items-center gap-1.5 text-xs text-ink-soft">
                <span>Quick fill:</span>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-xs"
                  onClick={() => {
                    setImapHost("outlook.office365.com");
                    setImapSecurity("imaps");
                    setImapPort("993");
                    setImapPassKey("oauth_access_token");
                  }}
                >
                  Microsoft 365 / Outlook
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 px-2 text-xs"
                  onClick={() => {
                    setImapHost("imap.gmail.com");
                    setImapSecurity("imaps");
                    setImapPort("993");
                    setImapPassKey("oauth_access_token");
                  }}
                >
                  Google Workspace / Gmail
                </Button>
              </div>
              <div className="grid gap-3 sm:grid-cols-4">
                <div className="flex min-w-0 flex-col gap-1 sm:col-span-2">
                  <label htmlFor="imap-host" className="text-xs font-medium text-ink">IMAP host</label>
                  <Input
                    id="imap-host"
                    className="mono w-full min-w-0 text-xs"
                    value={imapHost}
                    onChange={(e) => setImapHost(e.target.value)}
                    placeholder="imap.example.com"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-security" className="text-xs font-medium text-ink">Security</label>
                  <select
                    id="imap-security"
                    value={imapSecurity}
                    onChange={(e) => {
                      const nextSec = e.target.value as ImapSecurity;
                      setImapSecurity(nextSec);
                      setImapPort(defaultImapPort(nextSec));
                    }}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    <option value="imaps">IMAPS (TLS)</option>
                    <option value="starttls">STARTTLS</option>
                    <option value="plain_loopback_test">Plain loopback test</option>
                  </select>
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-port" className="text-xs font-medium text-ink">Port</label>
                  <Input
                    id="imap-port"
                    type="number"
                    min={1}
                    max={65535}
                    className="mono w-full min-w-0 text-xs"
                    value={imapPort}
                    onChange={(e) => setImapPort(e.target.value)}
                  />
                </div>
              </div>
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-folder" className="text-xs font-medium text-ink">Mailbox folder</label>
                  <Input
                    id="imap-folder"
                    className="mono w-full min-w-0 text-xs"
                    value={imapFolder}
                    onChange={(e) => setImapFolder(e.target.value)}
                    placeholder="INBOX"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-poll-interval" className="text-xs font-medium text-ink">Poll interval (seconds)</label>
                  <Input
                    id="imap-poll-interval"
                    type="number"
                    min={5}
                    className="mono w-full min-w-0 text-xs"
                    value={imapPollInterval}
                    onChange={(e) => setImapPollInterval(e.target.value)}
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-subject-filter" className="text-xs font-medium text-ink">Subject filter (optional)</label>
                  <Input
                    id="imap-subject-filter"
                    value={imapSubjectFilter}
                    onChange={(e) => setImapSubjectFilter(e.target.value)}
                    placeholder="Invoice"
                    className="w-full min-w-0"
                  />
                </div>
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-username-secret-key" className="text-xs font-medium text-ink">Username secret key</label>
                  <Input
                    id="imap-username-secret-key"
                    list="connector-secret-keys"
                    className="mono w-full min-w-0 text-xs"
                    value={imapUserKey}
                    onChange={(e) => setImapUserKey(e.target.value)}
                    placeholder="imap_user"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="imap-password-secret-key" className="text-xs font-medium text-ink">Password secret key</label>
                  <Input
                    id="imap-password-secret-key"
                    list="connector-secret-keys"
                    className="mono w-full min-w-0 text-xs"
                    value={imapPassKey}
                    onChange={(e) => setImapPassKey(e.target.value)}
                    placeholder="imap_pass"
                  />
                </div>
              </div>
              <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                <input
                  id="imap-mark-seen"
                  type="checkbox"
                  checked={imapMarkSeen}
                  onChange={(e) => setImapMarkSeen(e.target.checked)}
                  className="size-4 rounded border-input accent-primary"
                />
                <span>Mark matched messages as seen after ingesting</span>
              </label>
              {imapFormErr && (
                <p id="imap-poll-error" role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg [overflow-wrap:anywhere]">
                  {imapFormErr}
                </p>
              )}
              <div className="sticky bottom-0 z-10 -mx-3.5 -mb-3.5 mt-1 flex flex-wrap items-center justify-end gap-2 rounded-b-sm border-t border-hairline bg-surface/95 px-3.5 py-2.5 backdrop-blur-xs">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => { setImapFormErr(""); setAddingImap(false); }}>
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingImap || routines.length === 0}>
                  {savingImap ? "Saving…" : "Save IMAP inbox"}
                </Button>
              </div>
            </form>
          )}

          {loading && folderAndImapTriggers.length === 0 ? (
            <p className="py-4 text-sm text-ink-2">Loading folder and IMAP triggers…</p>
          ) : folderAndImapTriggers.length === 0 ? (
            <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
              No watched folders or IMAP inbox pollers configured yet. Add an approved local folder or IMAP inbox to trigger a routine automatically.
            </div>
          ) : (
            <ul className="flex min-w-0 w-full flex-col gap-2.5">
              {folderAndImapTriggers.map((rule) => {
                const targetName = (rule.routine_id && routineNameMap.get(rule.routine_id)) || rule.routine_id || "default handler";
                if (rule.kind === "imap_poll") {
                  const cfg = rule.imap_config;
                  return (
                    <li
                      key={rule.id}
                      data-trigger-id={rule.id}
                      data-trigger-kind="imap_poll"
                      className="flex min-w-0 w-full max-w-full flex-wrap items-start justify-between gap-3 rounded-card border border-hairline bg-surface p-3.5"
                    >
                      <div className="min-w-0 flex-1 overflow-hidden">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{rule.name}</span>
                          <Pill
                            variant="secondary"
                            className={cn(
                              "px-2 py-0.5 text-xs",
                              rule.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-pill-inactive-bg text-pill-inactive-fg"
                            )}
                          >
                            {rule.enabled ? "Polling" : "Paused"}
                          </Pill>
                          <Badge variant="outline" size="sm" className="gap-1 text-xs">
                            <Mail className="size-3" strokeWidth={1.5} aria-hidden />
                            IMAP Inbox
                          </Badge>
                          {cfg && (
                            <Badge variant="secondary" size="sm" className="mono h-auto max-w-full whitespace-normal break-all py-0.5 text-left text-xs">
                              {cfg.folder} · every {cfg.poll_interval_secs}s
                            </Badge>
                          )}
                        </div>
                        {cfg && (
                          <p className="mono mt-1 min-w-0 max-w-full break-all text-xs text-ink-2">
                            {cfg.security}://{cfg.host}:{cfg.port}
                            {cfg.subject_filter ? ` · subject "${cfg.subject_filter}"` : ""}
                            {cfg.mark_seen ? " · mark seen" : ""}
                          </p>
                        )}
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
                        aria-label={`Delete IMAP trigger ${rule.name}`}
                        className="shrink-0 max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
                        onClick={() => void handleDeleteTrigger(rule)}
                      >
                        <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                      </Button>
                    </li>
                  );
                }
                const dirPath = rule.watch_dir || rule.folder_path || "";
                const globText = rule.glob || rule.glob_filter || "*.*";
                return (
                  <li
                    key={rule.id}
                    data-trigger-id={rule.id}
                    data-trigger-kind="folder_watch"
                    className="flex min-w-0 w-full max-w-full flex-wrap items-start justify-between gap-3 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="min-w-0 flex-1 overflow-hidden">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{rule.name}</span>
                        <Pill
                          variant="secondary"
                          className={cn(
                            "px-2 py-0.5 text-xs",
                            rule.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-pill-inactive-bg text-pill-inactive-fg"
                          )}
                        >
                          {rule.enabled ? "Watching" : "Paused"}
                        </Pill>
                        <Badge variant="outline" size="sm" className="mono h-auto max-w-full whitespace-normal break-all text-left text-xs">
                          {globText}
                        </Badge>
                      </div>
                      <p className="mono mt-1 min-w-0 max-w-full break-all text-xs text-ink-2">{dirPath}</p>
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
                      className="shrink-0 max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
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
        <TabsContent value="outbound" className="flex min-w-0 w-full flex-col gap-4 pt-2">
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
              className="flex min-w-0 w-full flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="outbound-name" className="text-xs font-medium text-ink">Connector name</label>
                  <Input
                    id="outbound-name"
                    value={outName}
                    onChange={(e) => setOutName(e.target.value)}
                    placeholder={outKind === "smtp_relay" ? "Client billing email" : "Billing webhook"}
                    className="w-full min-w-0"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="outbound-kind" className="text-xs font-medium text-ink">Connector kind</label>
                  <select
                    id="outbound-kind"
                    value={outKind}
                    onChange={(e) => setOutKind(e.target.value as OutboundConnectorKind)}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    <option value="http_webhook">HTTP Webhook</option>
                    <option value="rest_api">REST API</option>
                    <option value="smtp_relay">SMTP Relay</option>
                  </select>
                </div>
                {outKind !== "smtp_relay" && (
                  <div className="flex min-w-0 flex-col gap-1">
                    <label htmlFor="outbound-method" className="text-xs font-medium text-ink">HTTP method</label>
                    <select
                      id="outbound-method"
                      value={outMethod}
                      onChange={(e) => setOutMethod(e.target.value === "PUT" ? "PUT" : "POST")}
                      className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                    >
                      <option value="POST">POST</option>
                      <option value="PUT">PUT</option>
                    </select>
                  </div>
                )}
              </div>

              {outKind === "smtp_relay" ? (
                <div id="smtp-config-section" className="flex min-w-0 flex-col gap-3 rounded-sm border border-hairline bg-surface p-3">
                  <div className="flex items-center gap-2 text-xs font-medium text-ink">
                    <Mail className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                    <span>SMTP Relay configuration</span>
                  </div>
                  <div className="flex flex-wrap items-center gap-1.5 text-xs text-ink-soft">
                    <span>Quick fill:</span>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="h-7 px-2 text-xs"
                      onClick={() => {
                        setSmtpHost("smtp.office365.com");
                        setSmtpSecurity("starttls");
                        setSmtpPort("587");
                        setSmtpPassKey("oauth_access_token");
                        setOauthEnabled(true);
                        setOauthTokenUrl("https://login.microsoftonline.com/common/oauth2/v2.0/token");
                        setOauthScopes(
                          "https://outlook.office365.com/SMTP.Send https://outlook.office365.com/IMAP.AccessAsUser.All offline_access",
                        );
                      }}
                    >
                      Microsoft 365 / Outlook
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="h-7 px-2 text-xs"
                      onClick={() => {
                        setSmtpHost("smtp.gmail.com");
                        setSmtpSecurity("starttls");
                        setSmtpPort("587");
                        setSmtpPassKey("oauth_access_token");
                        setOauthEnabled(true);
                        setOauthTokenUrl("https://oauth2.googleapis.com/token");
                        setOauthScopes("https://mail.google.com/");
                      }}
                    >
                      Google Workspace / Gmail
                    </Button>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-4">
                    <div className="flex min-w-0 flex-col gap-1 sm:col-span-2">
                      <label htmlFor="smtp-host" className="text-xs font-medium text-ink">SMTP host</label>
                      <Input
                        id="smtp-host"
                        className="mono w-full min-w-0 text-xs"
                        value={smtpHost}
                        onChange={(e) => setSmtpHost(e.target.value)}
                        placeholder="smtp.example.com"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-security" className="text-xs font-medium text-ink">Security</label>
                      <select
                        id="smtp-security"
                        value={smtpSecurity}
                        onChange={(e) => {
                          const nextSec = e.target.value as SmtpSecurity;
                          setSmtpSecurity(nextSec);
                          setSmtpPort(defaultSmtpPort(nextSec));
                        }}
                        className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                      >
                        <option value="smtps">SMTPS (TLS)</option>
                        <option value="starttls">STARTTLS</option>
                        <option value="plain_loopback_test">Plain loopback test</option>
                      </select>
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-port" className="text-xs font-medium text-ink">Port</label>
                      <Input
                        id="smtp-port"
                        type="number"
                        min={1}
                        max={65535}
                        className="mono w-full min-w-0 text-xs"
                        value={smtpPort}
                        onChange={(e) => setSmtpPort(e.target.value)}
                      />
                    </div>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-from-addr" className="text-xs font-medium text-ink">Sender address</label>
                      <Input
                        id="smtp-from-addr"
                        type="email"
                        className="mono w-full min-w-0 text-xs"
                        value={smtpFromAddr}
                        onChange={(e) => setSmtpFromAddr(e.target.value)}
                        placeholder="billing@example.com"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-default-to" className="text-xs font-medium text-ink">Default recipients (comma-separated)</label>
                      <Input
                        id="smtp-default-to"
                        className="mono w-full min-w-0 text-xs"
                        value={smtpDefaultTo}
                        onChange={(e) => setSmtpDefaultTo(e.target.value)}
                        placeholder="client@acme.com, finance@acme.com"
                      />
                    </div>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-username-key" className="text-xs font-medium text-ink">Username secret key</label>
                      <Input
                        id="smtp-username-key"
                        list="connector-secret-keys"
                        className="mono w-full min-w-0 text-xs"
                        value={smtpUserKey}
                        onChange={(e) => setSmtpUserKey(e.target.value)}
                        placeholder="smtp_user"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-username-secret" className="text-xs font-medium text-ink">Username value (write-only, optional)</label>
                      <Input
                        id="smtp-username-secret"
                        type="password"
                        autoComplete="new-password"
                        className="w-full min-w-0"
                        value={smtpUserSecret}
                        onChange={(e) => setSmtpUserSecret(e.target.value)}
                        placeholder="Leave blank if key already saved"
                      />
                    </div>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-password-key" className="text-xs font-medium text-ink">Password secret key</label>
                      <Input
                        id="smtp-password-key"
                        list="connector-secret-keys"
                        className="mono w-full min-w-0 text-xs"
                        value={smtpPassKey}
                        onChange={(e) => setSmtpPassKey(e.target.value)}
                        placeholder="smtp_pass"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="smtp-password-secret" className="text-xs font-medium text-ink">Password value (write-only, optional)</label>
                      <Input
                        id="smtp-password-secret"
                        type="password"
                        autoComplete="new-password"
                        className="w-full min-w-0"
                        value={smtpPassSecret}
                        onChange={(e) => setSmtpPassSecret(e.target.value)}
                        placeholder="Paste SMTP app password"
                      />
                    </div>
                  </div>
                </div>
              ) : (
                <>
                  <div className="flex min-w-0 flex-col gap-1">
                    <label htmlFor="outbound-url" className="text-xs font-medium text-ink">Destination URL</label>
                    <Input
                      id="outbound-url"
                      className="mono w-full min-w-0 text-xs"
                      value={outUrl}
                      onChange={(e) => setOutUrl(e.target.value)}
                      placeholder="https://hooks.example.com/ergon"
                    />
                  </div>

                  <div className="flex min-w-0 flex-col gap-1">
                    <label htmlFor="outbound-headers" className="text-xs font-medium text-ink">
                      Headers template (one Header: Value per line)
                    </label>
                    <Textarea
                      id="outbound-headers"
                      rows={2}
                      className="mono w-full min-w-0 text-xs"
                      value={outHeadersText}
                      onChange={(e) => setOutHeadersText(e.target.value)}
                      placeholder="Authorization: Bearer {{secret:api_token}}"
                    />
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="outbound-secret-key" className="text-xs font-medium text-ink">Secret key name</label>
                      <Input
                        id="outbound-secret-key"
                        list="connector-secret-keys"
                        className="mono w-full min-w-0 text-xs"
                        value={outSecretKey}
                        onChange={(e) => setOutSecretKey(e.target.value)}
                        placeholder="api_token"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="outbound-secret-value" className="text-xs font-medium text-ink">
                        Secret value (write-only, never read back)
                      </label>
                      <Input
                        id="outbound-secret-value"
                        type="password"
                        autoComplete="new-password"
                        className="w-full min-w-0"
                        value={outSecretValue}
                        onChange={(e) => setOutSecretValue(e.target.value)}
                        placeholder="Paste bearer token or API secret"
                      />
                    </div>
                  </div>
                </>
              )}

              <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                <input
                  id="outbound-oauth-enabled"
                  type="checkbox"
                  checked={oauthEnabled}
                  onChange={(e) => setOauthEnabled(e.target.checked)}
                  className="size-4 rounded border-input accent-primary"
                />
                <span>Attach OAuth2 credentials (client credentials or refresh token)</span>
              </label>

              {oauthEnabled && (
                <div id="oauth-config-section" className="flex min-w-0 flex-col gap-3 rounded-sm border border-hairline bg-surface p-3">
                  <div className="flex items-center gap-2 text-xs font-medium text-ink">
                    <KeyRound className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                    <span>OAuth2 Credentials</span>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-token-url" className="text-xs font-medium text-ink">Token URL</label>
                      <Input
                        id="oauth-token-url"
                        className="mono w-full min-w-0 text-xs"
                        value={oauthTokenUrl}
                        onChange={(e) => setOauthTokenUrl(e.target.value)}
                        placeholder="https://oauth2.example.com/token"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-client-id" className="text-xs font-medium text-ink">Client ID</label>
                      <Input
                        id="oauth-client-id"
                        className="mono w-full min-w-0 text-xs"
                        value={oauthClientId}
                        onChange={(e) => setOauthClientId(e.target.value)}
                        placeholder="ergon-client-id"
                      />
                    </div>
                  </div>
                  <div className="flex min-w-0 flex-col gap-1">
                    <label htmlFor="oauth-scopes" className="text-xs font-medium text-ink">Scopes (comma or space-separated)</label>
                    <Input
                      id="oauth-scopes"
                      className="mono w-full min-w-0 text-xs"
                      value={oauthScopes}
                      onChange={(e) => setOauthScopes(e.target.value)}
                      placeholder="mail.send, offline_access"
                    />
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-client-secret-key" className="text-xs font-medium text-ink">Client secret key ref</label>
                      <Input
                        id="oauth-client-secret-key"
                        list="connector-secret-keys"
                        className="mono w-full min-w-0 text-xs"
                        value={oauthClientSecretKey}
                        onChange={(e) => setOauthClientSecretKey(e.target.value)}
                        placeholder="oauth_client_secret"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-client-secret-value" className="text-xs font-medium text-ink">Client secret value (write-only, optional)</label>
                      <Input
                        id="oauth-client-secret-value"
                        type="password"
                        autoComplete="new-password"
                        className="w-full min-w-0"
                        value={oauthClientSecretValue}
                        onChange={(e) => setOauthClientSecretValue(e.target.value)}
                        placeholder="Leave blank if key already saved"
                      />
                    </div>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-refresh-secret-key" className="text-xs font-medium text-ink">Refresh token secret key ref</label>
                      <Input
                        id="oauth-refresh-secret-key"
                        list="connector-secret-keys"
                        className="mono w-full min-w-0 text-xs"
                        value={oauthRefreshKey}
                        onChange={(e) => setOauthRefreshKey(e.target.value)}
                        placeholder="oauth_refresh_token"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label htmlFor="oauth-refresh-secret-value" className="text-xs font-medium text-ink">Refresh token value (write-only, optional)</label>
                      <Input
                        id="oauth-refresh-secret-value"
                        type="password"
                        autoComplete="new-password"
                        className="w-full min-w-0"
                        value={oauthRefreshValue}
                        onChange={(e) => setOauthRefreshValue(e.target.value)}
                        placeholder="Leave blank if key already saved"
                      />
                    </div>
                  </div>
                </div>
              )}

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

              {outboundFormErr && (
                <p id="outbound-form-error" role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg [overflow-wrap:anywhere]">
                  {outboundFormErr}
                </p>
              )}
              <div className="sticky bottom-0 z-10 -mx-3.5 -mb-3.5 mt-1 flex flex-wrap items-center justify-end gap-2 rounded-b-sm border-t border-hairline bg-surface/95 px-3.5 py-2.5 backdrop-blur-xs">
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="max-md:min-h-11"
                  onClick={() => {
                    wipeOutboundWriteOnlySecrets();
                    setOutboundFormErr("");
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
            <ul className="flex min-w-0 w-full flex-col gap-2.5">
              {outbound.map((c) => {
                const secretStored = Boolean(c.has_secret || (c.secret_keys && c.secret_keys.length > 0));
                const approvalRequired = Boolean(c.effective_require_approval ?? c.require_approval) || policyProfile === "strict_client";
                return (
                  <li
                    key={c.id}
                    data-outbound-id={c.id}
                    className="flex min-w-0 w-full max-w-full flex-col gap-2 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="flex min-w-0 w-full flex-wrap items-start justify-between gap-2">
                      <div className="min-w-[min(100%,14rem)] flex-1 overflow-hidden">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{c.name}</span>
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
                          {c.oauth_config && (
                            <Badge variant="secondary" size="sm" className="gap-1 text-xs">
                              <KeyRound className="size-3" strokeWidth={1.5} aria-hidden />
                              OAuth2
                            </Badge>
                          )}
                        </div>
                        <p className="mono mt-1 min-w-0 max-w-full break-all text-xs text-ink-2">
                          {c.kind === "smtp_relay" && c.smtp_config
                            ? `${c.smtp_config.security}://${c.smtp_config.host}:${c.smtp_config.port} · from ${c.smtp_config.from_addr}`
                            : `${c.method} ${c.url}`}
                        </p>
                        {c.smtp_config?.default_to && c.smtp_config.default_to.length > 0 && (
                          <p className="mt-0.5 min-w-0 max-w-full break-words text-xs text-ink-2">
                            Default recipients: <span className="mono break-all">{c.smtp_config.default_to.join(", ")}</span>
                          </p>
                        )}
                        {c.oauth_config && (
                          <p className="mt-0.5 min-w-0 max-w-full break-words text-xs text-ink-2">
                            OAuth2 client <span className="mono break-all">{c.oauth_config.client_id}</span>
                            {c.oauth_config.scopes.length > 0 ? ` (${c.oauth_config.scopes.join(", ")})` : ""}
                          </p>
                        )}
                      </div>
                      <div className="flex shrink-0 items-center gap-1.5">
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
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <h3 id="delivery-log-h" className="text-sm font-medium text-ink">Delivery log</h3>
                {deliveries.some((d) => ["error", "failed", "blocked", "denied"].includes(String(d.status).toLowerCase())) && (
                  <Badge variant="outline" size="sm" className="bg-status-stopped-bg text-xs text-status-stopped-fg">
                    {deliveries.filter((d) => ["error", "failed", "blocked", "denied"].includes(String(d.status).toLowerCase())).length} failed/blocked
                  </Badge>
                )}
              </div>
              <span className="mono text-xs text-ink-3">{deliveries.length}</span>
            </div>
            {loadErr ? (
              <div
                id="connector-delivery-error"
                role="alert"
                className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2.5 text-xs text-status-stopped-fg"
              >
                <span>{loadErr}</span>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="max-md:min-h-11 bg-surface text-ink"
                  onClick={() => void load()}
                >
                  Retry
                </Button>
              </div>
            ) : loading && deliveries.length === 0 ? (
              <p className="text-xs text-ink-2">Loading delivery log…</p>
            ) : deliveries.length === 0 ? (
              <p id="connector-delivery-empty" className="rounded-sm border border-hairline bg-board/40 px-3 py-2.5 text-xs text-ink-2">
                {outbound.length === 0
                  ? "No outbound deliveries recorded yet. Add an outbound connector above and run Test Delivery to verify."
                  : "No outbound deliveries recorded yet. Click Test Delivery on a connector above to send a test event."}
              </p>
            ) : (
              <ul className="divide-y divide-hairline rounded-card border border-hairline bg-surface">
                {deliveries.slice(0, 20).map((d) => {
                  const badge = deliveryStatusBadge(d.status);
                  const tsVal = typeof d.ts_ms === "number" ? d.ts_ms : typeof d.ts === "number" ? d.ts : 0;
                  const isHeld = d.status === "pending_approval" || d.status === "held";
                  const note =
                    d.error ||
                    d.summary ||
                    d.response_preview ||
                    (isHeld ? "Waiting for approval in Today before sending." : "");
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

        {/* Tab 4: MCP Tools */}
        <TabsContent value="mcp" className="flex min-w-0 w-full flex-col gap-3 pt-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="text-xs text-ink-2">
              Connect local stdio or remote HTTP/SSE MCP servers, discover tools, and gate execution with Today approvals.
            </span>
            <Button
              id="mcp-add-btn"
              type="button"
              size="sm"
              variant={addingMcp ? "outline" : "default"}
              className="max-md:min-h-11"
              disabled={!mcpSupported}
              onClick={() => setAddingMcp((v) => !v)}
            >
              <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
              {addingMcp ? "Close form" : "Add server"}
            </Button>
          </div>

          {!mcpSupported && (
            <div className="rounded-card border border-hairline bg-board/40 p-3.5 text-xs text-ink-2">
              MCP tool servers require Ergon server v0.38.0 or newer with MCP capabilities enabled.
            </div>
          )}

          {mcpLoadErr && (
            <div id="mcp-load-error" role="alert" className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg">
              <span>{mcpLoadErr}</span>
              <Button type="button" size="xs" variant="outline" className="max-md:min-h-11 bg-surface text-ink" onClick={() => void load()}>
                Retry
              </Button>
            </div>
          )}

          {addingMcp && (
            <form
              id="mcp-create-form"
              onSubmit={handleCreateMcpServer}
              className="flex min-w-0 w-full flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="mcp-name" className="text-xs font-medium text-ink">Server name</label>
                  <Input
                    id="mcp-name"
                    value={mcpName}
                    onChange={(e) => setMcpName(e.target.value)}
                    placeholder="Workspace SQLite MCP"
                    className="w-full min-w-0"
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="mcp-transport" className="text-xs font-medium text-ink">Transport</label>
                  <select
                    id="mcp-transport"
                    value={mcpTransport}
                    onChange={(e) => setMcpTransport(e.target.value === "http_sse" ? "http_sse" : "stdio")}
                    className="h-9 w-full min-w-0 rounded-lg border border-input bg-background px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    <option value="stdio">stdio (local process)</option>
                    <option value="http_sse">http_sse (remote SSE)</option>
                  </select>
                </div>
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="mcp-command-or-url" className="text-xs font-medium text-ink">
                    {mcpTransport === "http_sse" ? "SSE endpoint URL" : "Command"}
                  </label>
                  <Input
                    id="mcp-command-or-url"
                    className="mono w-full min-w-0 text-xs"
                    value={mcpCommandOrUrl}
                    onChange={(e) => setMcpCommandOrUrl(e.target.value)}
                    placeholder={mcpTransport === "http_sse" ? "https://mcp.example.com/sse" : "npx"}
                  />
                </div>
                <div className="flex min-w-0 flex-col gap-1">
                  <label htmlFor="mcp-args" className="text-xs font-medium text-ink">Arguments (space-separated)</label>
                  <Input
                    id="mcp-args"
                    className="mono w-full min-w-0 text-xs"
                    value={mcpArgsText}
                    onChange={(e) => setMcpArgsText(e.target.value)}
                    placeholder="-y @modelcontextprotocol/server-filesystem /path"
                  />
                </div>
              </div>
              <div className="flex min-w-0 flex-col gap-1">
                <label htmlFor="mcp-env-secret-keys" className="text-xs font-medium text-ink">Environment secret keys (comma-separated)</label>
                <Input
                  id="mcp-env-secret-keys"
                  list="connector-secret-keys"
                  className="mono w-full min-w-0 text-xs"
                  value={mcpEnvSecretKeys}
                  onChange={(e) => setMcpEnvSecretKeys(e.target.value)}
                  placeholder="GITHUB_TOKEN, DATABASE_URL"
                />
              </div>
              <div className="flex flex-wrap items-center gap-4">
                <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                  <input
                    id="mcp-require-approval"
                    type="checkbox"
                    checked={policyProfile === "strict_client" ? true : mcpRequireApproval}
                    disabled={policyProfile === "strict_client"}
                    onChange={(e) => setMcpRequireApproval(e.target.checked)}
                    className="size-4 rounded border-input accent-primary"
                  />
                  <span>
                    Require approval by default
                    {policyProfile === "strict_client" ? " (required by Strict Client)" : ""}
                  </span>
                </label>
                <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                  <input
                    id="mcp-enabled"
                    type="checkbox"
                    checked={mcpEnabled}
                    onChange={(e) => setMcpEnabled(e.target.checked)}
                    className="size-4 rounded border-input accent-primary"
                  />
                  <span>Enabled</span>
                </label>
              </div>
              {mcpFormErr && (
                <p id="mcp-form-error" role="alert" className="rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg [overflow-wrap:anywhere]">
                  {mcpFormErr}
                </p>
              )}
              <div className="sticky bottom-0 z-10 -mx-3.5 -mb-3.5 mt-1 flex flex-wrap items-center justify-end gap-2 rounded-b-sm border-t border-hairline bg-surface/95 px-3.5 py-2.5 backdrop-blur-xs">
                <Button type="button" size="sm" variant="ghost" className="max-md:min-h-11" onClick={() => { setMcpFormErr(""); setAddingMcp(false); }}>
                  Cancel
                </Button>
                <Button type="submit" size="sm" className="max-md:min-h-11" disabled={savingMcp}>
                  {savingMcp ? "Saving…" : "Save server"}
                </Button>
              </div>
            </form>
          )}

          {loading && mcpServers.length === 0 ? (
            <p className="py-4 text-sm text-ink-2">Loading MCP servers…</p>
          ) : mcpServers.length === 0 ? (
            <div className="rounded-card border border-hairline bg-board/40 p-4 text-sm text-ink-2">
              No MCP tool servers configured yet. Add a stdio or HTTP/SSE server to discover external tools.
            </div>
          ) : (
            <ul className="flex min-w-0 w-full flex-col gap-2.5">
              {mcpServers.map((srv) => {
                const tools = Array.isArray(srv.cached_tools) ? srv.cached_tools : [];
                const approvalForced = policyProfile === "strict_client";
                const requiresApproval = approvalForced || srv.require_approval_by_default;
                return (
                  <li
                    key={srv.id}
                    data-mcp-id={srv.id}
                    className="flex min-w-0 w-full max-w-full flex-col gap-2.5 rounded-card border border-hairline bg-surface p-3.5"
                  >
                    <div className="flex min-w-0 w-full flex-wrap items-start justify-between gap-2">
                      <div className="min-w-[min(100%,14rem)] flex-1 overflow-hidden">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="min-w-0 text-sm font-medium text-ink [overflow-wrap:anywhere]">{srv.name}</span>
                          <Badge variant="outline" size="sm" className="mono text-xs">
                            {srv.transport}
                          </Badge>
                          <Pill
                            variant="secondary"
                            className={cn(
                              "px-2 py-0.5 text-xs",
                              srv.enabled ? "bg-pill-active-bg text-pill-active-fg" : "bg-pill-inactive-bg text-pill-inactive-fg"
                            )}
                          >
                            {srv.enabled ? "Enabled" : "Paused"}
                          </Pill>
                          <Pill
                            variant="secondary"
                            className={cn(
                              "px-2 py-0.5 text-xs",
                              requiresApproval ? "bg-pill-warn-bg text-pill-warn-fg" : "bg-pill-active-bg text-pill-active-fg"
                            )}
                          >
                            {requiresApproval ? "Approval required" : "Auto-run"}
                          </Pill>
                        </div>
                        <p className="mono mt-1 min-w-0 max-w-full break-all text-xs text-ink-2">
                          {srv.command_or_url}
                          {srv.args && srv.args.length > 0 ? ` ${srv.args.join(" ")}` : ""}
                        </p>
                        {srv.env_secret_keys && srv.env_secret_keys.length > 0 && (
                          <p className="mt-1 min-w-0 max-w-full break-words text-xs text-ink-2">
                            Env secrets: <span className="mono break-all">{srv.env_secret_keys.join(", ")}</span>
                          </p>
                        )}
                      </div>
                      <div className="flex shrink-0 flex-wrap items-center gap-1.5">
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          data-action="discover-mcp"
                          disabled={discoveringMcpId === srv.id}
                          className="max-md:min-h-11"
                          onClick={() => void handleDiscoverMcp(srv)}
                        >
                          <RefreshCw className={cn("size-3.5", discoveringMcpId === srv.id && "motion-safe:animate-spin")} strokeWidth={1.5} aria-hidden />
                          {discoveringMcpId === srv.id ? "Discovering…" : "Discover Tools"}
                        </Button>
                        <Button
                          type="button"
                          size="icon-sm"
                          variant="ghost"
                          aria-label={`Delete MCP server ${srv.name}`}
                          className="max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-destructive-foreground"
                          onClick={() => void handleDeleteMcp(srv)}
                        >
                          <Trash2 className="size-4" strokeWidth={1.5} aria-hidden />
                        </Button>
                      </div>
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-2">
                      <label className="flex min-h-9 cursor-pointer items-center gap-2 text-xs text-ink max-md:min-h-11">
                        <input
                          type="checkbox"
                          data-action="toggle-mcp-approval"
                          checked={requiresApproval}
                          disabled={togglingMcpId === srv.id || (approvalForced && srv.require_approval_by_default)}
                          onChange={() => void handleToggleMcpApproval(srv)}
                          className="size-4 rounded border-input accent-primary"
                        />
                        <span>Require approval before tool calls</span>
                      </label>
                      <span className="mono text-xs text-ink-3">
                        {tools.length} {tools.length === 1 ? "tool" : "tools"}
                        {srv.last_discovered_ms ? ` · ${relTime(srv.last_discovered_ms)}` : ""}
                      </span>
                    </div>

                    <div data-mcp-tools={srv.id} className="rounded-sm border border-hairline bg-board/40 px-3 py-2 text-xs">
                      {tools.length === 0 ? (
                        <p className="text-ink-2">No cached tools yet. Click Discover Tools to query this server.</p>
                      ) : (
                          <ul className="flex min-w-0 flex-col gap-1.5">
                          {tools.map((tool) => (
                            <li key={tool.name} className="flex min-w-0 flex-wrap items-baseline gap-2">
                              <span className="inline-flex min-w-0 max-w-full items-center gap-1 font-medium text-ink">
                                <Wrench className="size-3 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                                <code className="mono min-w-0 break-all">{tool.name}</code>
                              </span>
                              {tool.description && (
                                <span className="min-w-0 text-ink-2 [overflow-wrap:anywhere]">{tool.description}</span>
                              )}
                            </li>
                          ))}
                        </ul>
                      )}
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </TabsContent>
      </Tabs>
      {storedSecrets.length > 0 && (
        <datalist id="connector-secret-keys">
          {storedSecrets.map((sec) => (
            <option key={sec.key} value={sec.key} />
          ))}
        </datalist>
      )}
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
      description="Configure inbound webhooks, watched folders, IMAP inboxes, outbound actions, and MCP tool servers."
      panelId="connectors-panel"
      className="max-w-3xl"
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
