// Workspace Switcher (v0.7.0):
// - PersonSelect: Left Rail active-desk button triggering the external WorkspaceSwitcherPanel
//   (no Radix Popover nested inside mobile Vaul Drawer.Content, preventing 390x844/320x568 tap traps).
// - WorkspaceSwitcherPanel: MorphingDialog strictly mounted OUTSIDE Vaul <Drawer.Content> (z-50),
//   refreshing GET /api/people on open and switching via POST /api/people/switch (with fallback).
// - WorkspaceSwitcherList: Reusable compact switcher list also embedded inside SettingsPanel.
import { useEffect, useMemo, useRef, useState } from "react";
import { Archive, Briefcase, Check, ChevronsUpDown, Lock, Pause, Play, Plus, Search, ShieldCheck, Trash2, UserPlus, Users, X } from "lucide-react";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Textarea } from "@/components/ui/textarea";
import { Pill } from "@/components/kibo-ui/pill";
import {
  engagementWorkspacePrefix,
  type Client,
  type ClientContact,
  type Engagement,
  type EngagementContext,
  type Person,
  type RolloverMode,
  type WorkspacePolicy,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export function personLabel(p: Person): string {
  const n = String(p?.name ?? "").trim() || "Unnamed";
  const r = String(p?.role ?? "").trim();
  return r ? `${n}, ${r}` : n;
}

export function personInitials(name: unknown): string {
  return String(name ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("") || "E";
}

export function PersonSelect({
  people,
  active,
  activeClient,
  activeEngagement,
  disabled,
  expanded,
  onOpen,
}: {
  people: Person[];
  active: string;
  activeClient?: Client | null;
  activeEngagement?: Engagement | null;
  disabled: boolean;
  expanded?: boolean;
  onOpen: () => void;
}) {
  const safePeople = Array.isArray(people) ? people : [];
  const current = safePeople.find((p) => p.id === active);
  const currentName = current ? (String(current.name ?? "").trim() || "Unnamed") : "Choose a workspace";
  const currentRole = current ? String(current.role ?? "").trim() : "";
  const subLabel =
    activeClient && activeEngagement
      ? `${activeClient.name} / ${activeEngagement.name}`
      : activeEngagement
        ? activeEngagement.name
        : currentRole;

  return (
    <button
      type="button"
      id="person-select"
      data-thing="workspace-btn"
      aria-label={`Working for ${currentName}${subLabel ? `, ${subLabel}` : ""}`}
      aria-haspopup="dialog"
      aria-expanded={expanded}
      disabled={disabled}
      onClick={onOpen}
      className="focus-ring flex min-h-11 min-w-[44px] max-w-[210px] cursor-pointer items-center justify-between gap-2 rounded-sm border border-hairline bg-board/60 px-2.5 py-1 text-left text-sm text-ink outline-none hover:bg-board disabled:cursor-not-allowed disabled:bg-track disabled:text-ink-2 sm:max-w-[250px] md:min-h-9"
    >
      <span className="flex min-w-0 items-center gap-2.5">
        <span aria-hidden className="grid size-7 shrink-0 place-items-center rounded-full bg-av-sky-bg text-xs font-medium text-av-sky-fg">
          {personInitials(currentName)}
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-medium text-ink">{currentName}</span>
          {subLabel && <span className="block truncate text-xs text-ink-2">{subLabel}</span>}
        </span>
      </span>
      <ChevronsUpDown className="size-4 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
    </button>
  );
}

export function WorkspaceSwitcherList({
  people,
  active,
  switchingId,
  onSwitch,
  compact = false,
}: {
  people: Person[];
  active: string;
  switchingId?: string | null;
  onSwitch: (id: string) => void;
  compact?: boolean;
}) {
  const safePeople = Array.isArray(people) ? people : [];

  if (!safePeople.length) {
    return (
      <p className="rounded-sm border border-hairline bg-board/40 px-3 py-2.5 text-sm text-ink-2">
        No employee workspaces available yet.
      </p>
    );
  }

  return (
    <ul className="flex flex-col gap-1.5" role="list" aria-label="Employee workspaces">
      {safePeople.map((p) => {
        const isCurrent = p.id === active;
        const pName = String(p?.name ?? "").trim() || "Unnamed";
        const pRole = String(p?.role ?? "").trim() || "Personal workspace";
        const busyThis = switchingId === p.id;
        return (
          <li key={p.id}>
            <button
              type="button"
              data-workspace-id={p.id}
              aria-current={isCurrent ? "true" : undefined}
              disabled={Boolean(switchingId)}
              onClick={() => onSwitch(p.id)}
              className={cn(
                "focus-ring flex min-h-11 w-full cursor-pointer items-center justify-between gap-2.5 rounded-sm border px-3 text-left outline-none disabled:cursor-wait disabled:bg-track disabled:text-ink-3",
                compact ? "py-2" : "py-2.5",
                isCurrent
                  ? "border-blue bg-blue-soft text-ink"
                  : "border-hairline bg-surface text-ink hover:bg-board"
              )}
            >
              <span className="flex min-w-0 items-center gap-2.5">
                <span
                  aria-hidden
                  className={cn(
                    "grid size-8 shrink-0 place-items-center rounded-full text-xs font-medium",
                    isCurrent ? "bg-ink text-canvas" : "bg-av-sky-bg text-av-sky-fg"
                  )}
                >
                  {personInitials(pName)}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-medium text-ink">{pName}</span>
                  <span className="block truncate text-xs text-ink-2">{pRole}</span>
                </span>
              </span>
              <span className="flex shrink-0 items-center gap-1.5">
                {busyThis ? (
                  <span className="mono text-xs text-ink-2">Switching…</span>
                ) : isCurrent ? (
                  <Pill variant="secondary" className="bg-pill-active-bg px-2 py-0.5 text-xs font-medium text-pill-active-fg">
                    <Check className="size-3" strokeWidth={2} aria-hidden />
                    Active
                  </Pill>
                ) : (
                  <span className="text-xs font-medium text-ink-2">Switch</span>
                )}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

function WorkspaceSwitcherBody({
  people,
  active,
  onRefresh,
  onSwitch,
  onAddPerson,
  clientsEnabled = false,
  clients = [],
  engagements = [],
  activeEngagementId = "",
  workspacePolicy = null,
  activeEngagementContext = null,
  onRefreshClients,
  onSwitchEngagement,
  onCreateClient,
  onUpdateClient,
  onCreateEngagement,
  onUpdateEngagement,
  onUpdateWorkspacePolicy,
  onLoadEngagementContext,
}: {
  people: Person[];
  active: string;
  onRefresh: () => void;
  onSwitch: (id: string) => Promise<void>;
  onAddPerson: () => void;
  clientsEnabled?: boolean;
  clients?: Client[];
  engagements?: Engagement[];
  activeEngagementId?: string;
  workspacePolicy?: WorkspacePolicy | null;
  activeEngagementContext?: EngagementContext | null;
  onRefreshClients?: () => void;
  onSwitchEngagement?: (id: string) => Promise<void>;
  onCreateClient?: (input: { name: string; brand_header?: string; default_rate_usd?: number }) => Promise<Client | null>;
  onUpdateClient?: (
    id: string,
    patch: {
      name?: string;
      brand_header?: string;
      billing_currency?: string;
      default_rate_usd?: number;
      contacts?: ClientContact[];
      notes_md?: string;
      tax_rate_pct?: number;
      payment_terms?: string;
    }
  ) => Promise<Client | null>;
  onCreateEngagement?: (input: {
    client_id: string;
    name: string;
    person_id?: string;
    hourly_rate_usd?: number;
    retainer_budget_usd?: number;
  }) => Promise<Engagement | null>;
  onUpdateEngagement?: (
    id: string,
    patch: {
      name?: string;
      retainer_hours?: number;
      retainer_budget_usd?: number;
      hourly_rate_usd?: number;
      period?: string;
      workspace_prefix?: string;
      sow_rules?: string[];
      rollover_mode?: RolloverMode;
      status?: string;
      archived_ms?: number | null;
    }
  ) => Promise<Engagement | null>;
  onUpdateWorkspacePolicy?: (patch: { isolate_active_engagement?: boolean }) => Promise<WorkspacePolicy | null>;
  onLoadEngagementContext?: (id: string) => Promise<EngagementContext | null>;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const [query, setQuery] = useState("");
  const [switchingId, setSwitchingId] = useState<string | null>(null);
  const [switchingEngId, setSwitchingEngId] = useState<string | null>(null);
  const [selectedClientId, setSelectedClientId] = useState<string>("");
  const [createOpen, setCreateOpen] = useState(false);
  const [crmOpen, setCrmOpen] = useState(false);
  const [crmTab, setCrmTab] = useState<"sow" | "crm">("sow");
  const [createNewClientMode, setCreateNewClientMode] = useState(false);
  const [clientNameInput, setClientNameInput] = useState("");
  const [engagementNameInput, setEngagementNameInput] = useState("");
  const [hourlyRateInput, setHourlyRateInput] = useState("200");
  const [retainerBudgetInput, setRetainerBudgetInput] = useState("2500");
  const [createError, setCreateError] = useState("");
  const [creating, setCreating] = useState(false);

  const [crmContacts, setCrmContacts] = useState<ClientContact[]>([]);
  const [crmContactName, setCrmContactName] = useState("");
  const [crmContactEmail, setCrmContactEmail] = useState("");
  const [crmContactRole, setCrmContactRole] = useState("");
  const [crmBrandHeader, setCrmBrandHeader] = useState("");
  const [crmTaxRate, setCrmTaxRate] = useState("0");
  const [crmPaymentTerms, setCrmPaymentTerms] = useState("Net 30");
  const [crmNotesMd, setCrmNotesMd] = useState("");
  const [savingClientCrm, setSavingClientCrm] = useState(false);

  const [editingEngId, setEditingEngId] = useState("");
  const [sowPrefix, setSowPrefix] = useState("");
  const [sowRules, setSowRules] = useState<string[]>([]);
  const [sowRuleInput, setSowRuleInput] = useState("");
  const [sowRollover, setSowRollover] = useState<RolloverMode>("none");
  const [savingEngSow, setSavingEngSow] = useState(false);
  const [togglingIsolate, setTogglingIsolate] = useState(false);

  const opLockRef = useRef(false);
  const isBusyOp = Boolean(switchingId || switchingEngId || creating || savingClientCrm || savingEngSow);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setCreateOpen(false);
      setCreateError("");
      onRefresh();
      onRefreshClients?.();
    } else {
      setCrmOpen(false);
    }
  }, [isOpen, onRefresh, onRefreshClients]);

  const prevSyncKeyRef = useRef("");

  useEffect(() => {
    if (!clientsEnabled) return;
    if (!isOpen) {
      prevSyncKeyRef.current = "";
      return;
    }
    const syncKey = `open:${activeEngagementId}`;
    const activeEng = engagements.find((e) => e.id === activeEngagementId);
    if (prevSyncKeyRef.current !== syncKey && activeEng && clients.some((c) => c.id === activeEng.client_id)) {
      prevSyncKeyRef.current = syncKey;
      setSelectedClientId(activeEng.client_id);
      return;
    }
    if (clients.length > 0 && (!selectedClientId || !clients.some((c) => c.id === selectedClientId))) {
      prevSyncKeyRef.current = syncKey;
      setSelectedClientId(activeEng?.client_id ?? clients[0]?.id ?? "");
    }
  }, [clientsEnabled, clients, engagements, activeEngagementId, isOpen, selectedClientId]);

  const visibleEngagements = useMemo(() => {
    if (!selectedClientId) return engagements;
    return engagements.filter((e) => e.client_id === selectedClientId);
  }, [engagements, selectedClientId]);

  const selectedClient = useMemo(
    () => clients.find((c) => c.id === selectedClientId) || null,
    [clients, selectedClientId]
  );

  const editingEngagement = useMemo(() => {
    const activeInClient = visibleEngagements.find((e) => e.id === editingEngId);
    if (activeInClient) return activeInClient;
    const currentActive = visibleEngagements.find((e) => e.id === activeEngagementId);
    return currentActive || visibleEngagements[0] || null;
  }, [visibleEngagements, editingEngId, activeEngagementId]);

  useEffect(() => {
    if (!selectedClient) {
      setCrmContacts([]);
      setCrmBrandHeader("");
      setCrmTaxRate("0");
      setCrmPaymentTerms("Net 30");
      setCrmNotesMd("");
      return;
    }
    setCrmContacts(Array.isArray(selectedClient.contacts) ? selectedClient.contacts : []);
    setCrmBrandHeader(selectedClient.brand_header ?? "");
    setCrmTaxRate(typeof selectedClient.tax_rate_pct === "number" ? String(selectedClient.tax_rate_pct) : "0");
    setCrmPaymentTerms(selectedClient.payment_terms ?? "Net 30");
    setCrmNotesMd(selectedClient.notes_md ?? "");
  }, [selectedClient]);

  useEffect(() => {
    if (!editingEngagement) {
      setSowPrefix("");
      setSowRules([]);
      setSowRollover("none");
      return;
    }
    setEditingEngId(editingEngagement.id);
    setSowPrefix(engagementWorkspacePrefix(editingEngagement));
    setSowRules(Array.isArray(editingEngagement.sow_rules) ? editingEngagement.sow_rules : []);
    setSowRollover(editingEngagement.rollover_mode === "carry_hours" ? "carry_hours" : "none");
  }, [editingEngagement?.id, editingEngagement?.workspace_prefix, editingEngagement?.rollover_mode, editingEngagement?.sow_rules]);

  useEffect(() => {
    if (crmOpen && editingEngagement?.id && onLoadEngagementContext) {
      void onLoadEngagementContext(editingEngagement.id);
    }
  }, [crmOpen, editingEngagement?.id, onLoadEngagementContext]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return people;
    return people.filter(
      (p) =>
        String(p.name ?? "").toLowerCase().includes(q) ||
        String(p.role ?? "").toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q)
    );
  }, [people, query]);

  const handleSelect = async (id: string) => {
    if (opLockRef.current) return;
    if (id === active) {
      setIsOpen(false);
      return;
    }
    opLockRef.current = true;
    setSwitchingId(id);
    try {
      await onSwitch(id);
      setIsOpen(false);
    } finally {
      opLockRef.current = false;
      setSwitchingId(null);
    }
  };

  const handleSelectEngagement = async (engId: string) => {
    if (!onSwitchEngagement || opLockRef.current) return;
    if (engId === activeEngagementId) {
      setIsOpen(false);
      return;
    }
    opLockRef.current = true;
    setSwitchingEngId(engId);
    try {
      await onSwitchEngagement(engId);
      setIsOpen(false);
    } finally {
      opLockRef.current = false;
      setSwitchingEngId(null);
    }
  };

  const handleQuickCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!onCreateEngagement || opLockRef.current) return;
    const engName = engagementNameInput.trim();
    const rawRate = hourlyRateInput.trim();
    const rawBudget = retainerBudgetInput.trim();
    const rate = rawRate === "" ? 0 : Number(rawRate);
    const budget = rawBudget === "" ? 0 : Number(rawBudget);
    const useNewClient = createNewClientMode || clients.length === 0;
    const cliName = clientNameInput.trim();

    if (useNewClient && !cliName) {
      setCreateError("Enter a client name.");
      return;
    }
    if (!useNewClient && !selectedClientId) {
      setCreateError("Select a client first.");
      return;
    }
    if (!engName) {
      setCreateError("Enter an engagement name.");
      return;
    }
    if (!Number.isFinite(rate) || rate < 0 || !Number.isFinite(budget) || budget < 0) {
      setCreateError("Hourly rate and retainer budget must be zero or greater.");
      return;
    }
    if (rate > 1_000_000_000 || budget > 1_000_000_000) {
      setCreateError("Hourly rate and retainer budget cannot exceed $1,000,000,000.");
      return;
    }

    opLockRef.current = true;
    setCreateError("");
    setCreating(true);
    try {
      let targetClientId = selectedClientId;
      if (useNewClient && onCreateClient) {
        const createdCli = await onCreateClient({ name: cliName, default_rate_usd: rate });
        if (!createdCli) {
          setCreateError("Could not create client.");
          return;
        }
        targetClientId = createdCli.id;
        setSelectedClientId(createdCli.id);
      }
      const createdEng = await onCreateEngagement({
        client_id: targetClientId,
        name: engName,
        person_id: active || undefined,
        hourly_rate_usd: rate,
        retainer_budget_usd: budget,
      });
      if (!createdEng) {
        setCreateError("Could not create engagement.");
        return;
      }
      if (onSwitchEngagement) {
        await onSwitchEngagement(createdEng.id);
      }
      setClientNameInput("");
      setEngagementNameInput("");
      setCreateOpen(false);
      setIsOpen(false);
    } finally {
      opLockRef.current = false;
      setCreating(false);
    }
  };

  return (
    <div id="workspace-switcher-panel" className="flex flex-col gap-3.5 pt-1">
      {clientsEnabled && (
        <section id="client-engagement-section" aria-label="Client and engagement selector" className="flex flex-col gap-2.5">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="flex items-center gap-1.5 text-xs font-medium text-ink">
              <Briefcase className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
              Client and engagement
            </span>
            <div className="flex flex-wrap items-center gap-1.5">
              {(onUpdateClient || onUpdateEngagement) && clients.length > 0 && (
                <Button
                  id="client-crm-drawer-toggle"
                  type="button"
                  size="xs"
                  variant={crmOpen ? "default" : "outline"}
                  disabled={isBusyOp}
                  className="max-md:min-h-11 gap-1"
                  onClick={() => {
                    setCrmOpen((v) => !v);
                    setCreateOpen(false);
                  }}
                >
                  <ShieldCheck className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {crmOpen ? "Close CRM & SOW" : "CRM & SOW"}
                </Button>
              )}
              {onCreateEngagement && (
                <Button
                  id="engagement-quick-create-toggle"
                  type="button"
                  size="xs"
                  variant="outline"
                  disabled={isBusyOp}
                  className="max-md:min-h-11 gap-1"
                  onClick={() => {
                    setCreateOpen((v) => !v);
                    setCrmOpen(false);
                    setCreateNewClientMode(clients.length === 0);
                    setCreateError("");
                  }}
                >
                  <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
                  {createOpen ? "Close form" : "New client / engagement"}
                </Button>
              )}
            </div>
          </div>

          {crmOpen && selectedClient && (
            <div
              id="client-crm-sow-drawer"
              className="flex flex-col gap-3 rounded-sm border border-hairline bg-board/40 p-3"
            >
              <div className="flex flex-wrap items-center justify-between gap-2 border-b border-hairline pb-2">
                <div className="flex items-center gap-1.5" role="tablist" aria-label="CRM and SOW section">
                  <Button
                    id="crm-tab-sow"
                    type="button"
                    size="xs"
                    variant={crmTab === "sow" ? "default" : "outline"}
                    className="max-md:min-h-11"
                    onClick={() => setCrmTab("sow")}
                  >
                    Engagement SOW
                  </Button>
                  <Button
                    id="crm-tab-client"
                    type="button"
                    size="xs"
                    variant={crmTab === "crm" ? "default" : "outline"}
                    className="max-md:min-h-11"
                    onClick={() => setCrmTab("crm")}
                  >
                    Client CRM ({crmContacts.length})
                  </Button>
                </div>
                {onUpdateWorkspacePolicy && (
                  <Button
                    id="crm-isolate-engagement-toggle"
                    type="button"
                    size="xs"
                    variant={workspacePolicy?.isolate_active_engagement ? "default" : "outline"}
                    aria-pressed={Boolean(workspacePolicy?.isolate_active_engagement)}
                    disabled={togglingIsolate}
                    className="max-md:min-h-11 gap-1"
                    onClick={async () => {
                      setTogglingIsolate(true);
                      try {
                        await onUpdateWorkspacePolicy({
                          isolate_active_engagement: !workspacePolicy?.isolate_active_engagement,
                        });
                      } finally {
                        setTogglingIsolate(false);
                      }
                    }}
                  >
                    <Lock className="size-3" strokeWidth={1.5} aria-hidden />
                    {workspacePolicy?.isolate_active_engagement ? "Isolation: On" : "Isolation: Off"}
                  </Button>
                )}
              </div>

              {crmTab === "sow" && editingEngagement && (
                <div id="crm-sow-panel" className="flex flex-col gap-2.5">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex min-w-0 flex-1 flex-col gap-1">
                      <label htmlFor="crm-eng-select" className="text-xs font-medium text-ink">
                        Engagement
                      </label>
                      <select
                        id="crm-eng-select"
                        value={editingEngagement.id}
                        onChange={(e) => setEditingEngId(e.target.value)}
                        className="h-9 w-full rounded-sm border border-input bg-surface px-2.5 text-xs text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                      >
                        {visibleEngagements.map((e) => (
                          <option key={e.id} value={e.id}>
                            {e.name} ({e.status || "active"})
                          </option>
                        ))}
                      </select>
                    </div>
                    {activeEngagementContext && activeEngagementContext.engagement?.id === editingEngagement.id && (
                      <span id="crm-eng-context-meta" className="mono text-xs text-ink-2">
                        {activeEngagementContext.scoped_file_count} scoped files
                      </span>
                    )}
                  </div>

                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                    <div className="flex flex-col gap-1">
                      <label htmlFor="crm-eng-prefix" className="text-xs font-medium text-ink">
                        Workspace prefix
                      </label>
                      <Input
                        id="crm-eng-prefix"
                        value={sowPrefix}
                        onChange={(e) => setSowPrefix(e.target.value)}
                        placeholder={`clients/${editingEngagement.client_id}/${editingEngagement.id}`}
                        className="mono text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
                      <label htmlFor="crm-eng-rollover" className="text-xs font-medium text-ink">
                        Retainer rollover
                      </label>
                      <select
                        id="crm-eng-rollover"
                        value={sowRollover}
                        onChange={(e) => setSowRollover(e.target.value === "carry_hours" ? "carry_hours" : "none")}
                        className="h-9 w-full rounded-sm border border-input bg-surface px-2.5 text-xs text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                      >
                        <option value="none">None (reset period)</option>
                        <option value="carry_hours">Carry unused hours</option>
                      </select>
                    </div>
                  </div>

                  <div className="flex flex-col gap-1.5">
                    <label htmlFor="crm-sow-rule-input" className="text-xs font-medium text-ink">
                      SOW rules ({sowRules.length})
                    </label>
                    {sowRules.length > 0 && (
                      <ul id="crm-sow-rules-list" className="flex flex-col gap-1">
                        {sowRules.map((rule, idx) => (
                          <li
                            key={`${idx}-${rule}`}
                            className="flex items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-2.5 py-1.5 text-xs text-ink"
                          >
                            <span className="min-w-0 flex-1 break-words">{rule}</span>
                            <Button
                              type="button"
                              size="icon-xs"
                              variant="ghost"
                              aria-label={`Remove SOW rule ${idx + 1}`}
                              className="max-md:min-h-11 max-md:min-w-11 shrink-0"
                              onClick={() => setSowRules((prev) => prev.filter((_, i) => i !== idx))}
                            >
                              <Trash2 className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                            </Button>
                          </li>
                        ))}
                      </ul>
                    )}
                    <div className="flex items-center gap-1.5">
                      <Input
                        id="crm-sow-rule-input"
                        value={sowRuleInput}
                        onChange={(e) => setSowRuleInput(e.target.value)}
                        placeholder="Add SOW constraint (e.g. Cite sources on every brief)"
                        className="flex-1 text-xs max-md:min-h-11"
                      />
                      <Button
                        id="crm-add-sow-rule-btn"
                        type="button"
                        size="xs"
                        variant="outline"
                        className="max-md:min-h-11 shrink-0"
                        onClick={() => {
                          const next = sowRuleInput.trim();
                          if (!next) return;
                          setSowRules((prev) => [...prev, next]);
                          setSowRuleInput("");
                        }}
                      >
                        Add rule
                      </Button>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                    <div className="flex flex-wrap items-center gap-1.5">
                      {onUpdateEngagement && (
                        <>
                          <Button
                            id="crm-pause-engagement-btn"
                            type="button"
                            size="xs"
                            variant="outline"
                            disabled={savingEngSow}
                            className="max-md:min-h-11 gap-1"
                            onClick={async () => {
                              setSavingEngSow(true);
                              try {
                                const nextStatus = editingEngagement.status === "paused" ? "active" : "paused";
                                await onUpdateEngagement(editingEngagement.id, { status: nextStatus, archived_ms: null });
                              } finally {
                                setSavingEngSow(false);
                              }
                            }}
                          >
                            {editingEngagement.status === "paused" ? (
                              <Play className="size-3" strokeWidth={1.5} aria-hidden />
                            ) : (
                              <Pause className="size-3" strokeWidth={1.5} aria-hidden />
                            )}
                            {editingEngagement.status === "paused" ? "Resume" : "Pause"}
                          </Button>
                          <Button
                            id="crm-archive-engagement-btn"
                            type="button"
                            size="xs"
                            variant="outline"
                            disabled={savingEngSow}
                            className="max-md:min-h-11 gap-1"
                            onClick={async () => {
                              setSavingEngSow(true);
                              try {
                                const isArchived = editingEngagement.status === "archived";
                                await onUpdateEngagement(editingEngagement.id, {
                                  status: isArchived ? "active" : "archived",
                                  archived_ms: isArchived ? null : Date.now(),
                                });
                              } finally {
                                setSavingEngSow(false);
                              }
                            }}
                          >
                            <Archive className="size-3" strokeWidth={1.5} aria-hidden />
                            {editingEngagement.status === "archived" ? "Restore" : "Archive"}
                          </Button>
                        </>
                      )}
                    </div>
                    {onUpdateEngagement && (
                      <Button
                        id="crm-save-engagement-btn"
                        type="button"
                        size="xs"
                        disabled={savingEngSow}
                        className="max-md:min-h-11"
                        onClick={async () => {
                          setSavingEngSow(true);
                          try {
                            await onUpdateEngagement(editingEngagement.id, {
                              workspace_prefix: sowPrefix.trim(),
                              sow_rules: sowRules,
                              rollover_mode: sowRollover,
                            });
                          } finally {
                            setSavingEngSow(false);
                          }
                        }}
                      >
                        {savingEngSow ? "Saving…" : "Save SOW"}
                      </Button>
                    )}
                  </div>
                </div>
              )}

              {crmTab === "crm" && (
                <div id="crm-client-panel" className="flex flex-col gap-2.5">
                  <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
                    <div className="flex flex-col gap-1">
                      <label htmlFor="crm-brand-header" className="text-xs font-medium text-ink">
                        Brand header
                      </label>
                      <Input
                        id="crm-brand-header"
                        value={crmBrandHeader}
                        onChange={(e) => setCrmBrandHeader(e.target.value)}
                        placeholder={selectedClient.name}
                        className="text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
                      <label htmlFor="crm-tax-rate" className="text-xs font-medium text-ink">
                        Tax rate (%)
                      </label>
                      <Input
                        id="crm-tax-rate"
                        type="number"
                        min="0"
                        max="100"
                        step="any"
                        value={crmTaxRate}
                        onChange={(e) => setCrmTaxRate(e.target.value)}
                        className="mono text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
                      <label htmlFor="crm-payment-terms" className="text-xs font-medium text-ink">
                        Payment terms
                      </label>
                      <Input
                        id="crm-payment-terms"
                        value={crmPaymentTerms}
                        onChange={(e) => setCrmPaymentTerms(e.target.value)}
                        placeholder="Net 30"
                        className="text-xs max-md:min-h-11"
                      />
                    </div>
                  </div>

                  <div className="flex flex-col gap-1.5">
                    <span className="text-xs font-medium text-ink">Contacts ({crmContacts.length})</span>
                    {crmContacts.length > 0 && (
                      <ul id="crm-contacts-list" className="flex flex-col gap-1">
                        {crmContacts.map((c) => (
                          <li
                            key={c.id}
                            className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-2.5 py-1.5 text-xs"
                          >
                            <span className="min-w-0 flex-1">
                              <span className="font-medium text-ink">{c.name}</span>
                              {c.role ? <span className="text-ink-2"> ({c.role})</span> : null}
                              {c.email ? <span className="mono block truncate text-ink-2">{c.email}</span> : null}
                            </span>
                            <div className="flex items-center gap-1">
                              <Button
                                type="button"
                                size="xs"
                                variant={c.primary ? "default" : "outline"}
                                className="max-md:min-h-11"
                                onClick={() =>
                                  setCrmContacts((prev) =>
                                    prev.map((item) => ({ ...item, primary: item.id === c.id }))
                                  )
                                }
                              >
                                {c.primary ? "Primary" : "Make primary"}
                              </Button>
                              <Button
                                type="button"
                                size="icon-xs"
                                variant="ghost"
                                aria-label={`Remove contact ${c.name}`}
                                className="max-md:min-h-11 max-md:min-w-11"
                                onClick={() => setCrmContacts((prev) => prev.filter((item) => item.id !== c.id))}
                              >
                                <Trash2 className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                              </Button>
                            </div>
                          </li>
                        ))}
                      </ul>
                    )}
                    <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-[1fr_1fr_0.8fr_auto]">
                      <Input
                        id="crm-contact-name"
                        aria-label="Contact name"
                        value={crmContactName}
                        onChange={(e) => setCrmContactName(e.target.value)}
                        placeholder="Contact name"
                        className="text-xs max-md:min-h-11"
                      />
                      <Input
                        id="crm-contact-email"
                        aria-label="Contact email"
                        value={crmContactEmail}
                        onChange={(e) => setCrmContactEmail(e.target.value)}
                        placeholder="billing@client.example"
                        className="text-xs max-md:min-h-11"
                      />
                      <Input
                        id="crm-contact-role"
                        aria-label="Contact role"
                        value={crmContactRole}
                        onChange={(e) => setCrmContactRole(e.target.value)}
                        placeholder="GC / AP Lead"
                        className="text-xs max-md:min-h-11"
                      />
                      <Button
                        id="crm-add-contact-btn"
                        type="button"
                        size="xs"
                        variant="outline"
                        className="max-md:min-h-11"
                        onClick={() => {
                          const nm = crmContactName.trim();
                          if (!nm) return;
                          setCrmContacts((prev) => [
                            ...prev,
                            {
                              id: `ct-${Date.now()}-${prev.length + 1}`,
                              name: nm,
                              email: crmContactEmail.trim() || undefined,
                              role: crmContactRole.trim() || undefined,
                              primary: prev.length === 0,
                            },
                          ]);
                          setCrmContactName("");
                          setCrmContactEmail("");
                          setCrmContactRole("");
                        }}
                      >
                        Add
                      </Button>
                    </div>
                  </div>

                  <div className="flex flex-col gap-1">
                    <label htmlFor="crm-client-notes" className="text-xs font-medium text-ink">
                      Client notes
                    </label>
                    <Textarea
                      id="crm-client-notes"
                      rows={2}
                      value={crmNotesMd}
                      onChange={(e) => setCrmNotesMd(e.target.value)}
                      placeholder="Billing preferences, escalation contacts, or delivery rules"
                      className="text-xs"
                    />
                  </div>

                  {onUpdateClient && (
                    <div className="flex justify-end">
                      <Button
                        id="crm-save-client-btn"
                        type="button"
                        size="xs"
                        disabled={savingClientCrm}
                        className="max-md:min-h-11"
                        onClick={async () => {
                          const parsedTax = Number(crmTaxRate);
                          setSavingClientCrm(true);
                          try {
                            await onUpdateClient(selectedClient.id, {
                              brand_header: crmBrandHeader.trim(),
                              tax_rate_pct: Number.isFinite(parsedTax) && parsedTax >= 0 ? parsedTax : 0,
                              payment_terms: crmPaymentTerms.trim() || "Net 30",
                              contacts: crmContacts,
                              notes_md: crmNotesMd,
                            });
                          } finally {
                            setSavingClientCrm(false);
                          }
                        }}
                      >
                        {savingClientCrm ? "Saving…" : "Save client CRM"}
                      </Button>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {createOpen && (
            <form
              id="engagement-quick-create-form"
              noValidate
              onSubmit={handleQuickCreate}
              className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-board/40 p-3"
            >
              {clients.length > 0 && (
                <div className="flex flex-wrap items-center gap-1.5">
                  <Button
                    type="button"
                    size="xs"
                    variant={!createNewClientMode ? "default" : "outline"}
                    className="max-md:min-h-11"
                    onClick={() => setCreateNewClientMode(false)}
                  >
                    Existing client
                  </Button>
                  <Button
                    type="button"
                    size="xs"
                    variant={createNewClientMode ? "default" : "outline"}
                    className="max-md:min-h-11"
                    onClick={() => setCreateNewClientMode(true)}
                  >
                    New client
                  </Button>
                </div>
              )}

              {createNewClientMode || clients.length === 0 ? (
                <div className="flex flex-col gap-1">
                  <label htmlFor="quick-client-name" className="text-xs font-medium text-ink">Client name</label>
                  <Input
                    id="quick-client-name"
                    value={clientNameInput}
                    onChange={(e) => setClientNameInput(e.target.value)}
                    placeholder="Acme Legal LLP"
                    className="max-md:min-h-11"
                  />
                </div>
              ) : (
                <div className="flex flex-col gap-1">
                  <label htmlFor="quick-client-select" className="text-xs font-medium text-ink">Client</label>
                  <select
                    id="quick-client-select"
                    value={selectedClientId}
                    onChange={(e) => setSelectedClientId(e.target.value)}
                    className="h-9 w-full rounded-sm border border-input bg-surface px-2.5 text-sm text-ink outline-none focus-visible:ring-2 focus-visible:ring-ring max-md:min-h-11"
                  >
                    {clients.map((c) => (
                      <option key={c.id} value={c.id}>{c.name}</option>
                    ))}
                  </select>
                </div>
              )}

              <div className="flex flex-col gap-1">
                <label htmlFor="quick-engagement-name" className="text-xs font-medium text-ink">Engagement name</label>
                <Input
                  id="quick-engagement-name"
                  value={engagementNameInput}
                  onChange={(e) => setEngagementNameInput(e.target.value)}
                  placeholder="Q1 Compliance Retainer"
                  className="max-md:min-h-11"
                />
              </div>

              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div className="flex flex-col gap-1">
                  <label htmlFor="quick-engagement-rate" className="text-xs font-medium text-ink">Hourly rate (USD)</label>
                  <Input
                    id="quick-engagement-rate"
                    type="number"
                    min="0"
                    step="any"
                    value={hourlyRateInput}
                    onChange={(e) => setHourlyRateInput(e.target.value)}
                    className="mono max-md:min-h-11"
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <label htmlFor="quick-engagement-budget" className="text-xs font-medium text-ink">Retainer budget (USD)</label>
                  <Input
                    id="quick-engagement-budget"
                    type="number"
                    min="0"
                    step="any"
                    value={retainerBudgetInput}
                    onChange={(e) => setRetainerBudgetInput(e.target.value)}
                    className="mono max-md:min-h-11"
                  />
                </div>
              </div>

              {createError && (
                <p id="quick-engagement-error" role="alert" className="text-xs text-destructive-foreground">
                  {createError}
                </p>
              )}

              <div className="flex items-center justify-end gap-2">
                <Button id="quick-engagement-submit" type="submit" size="sm" className="max-md:min-h-11" disabled={creating}>
                  {creating ? "Creating…" : "Create engagement"}
                </Button>
              </div>
            </form>
          )}

          {clients.length > 0 && (
            <div className="flex flex-wrap gap-1.5" role="group" aria-label="Clients">
              {clients.map((c) => {
                const isSel = c.id === selectedClientId;
                return (
                  <Button
                    key={c.id}
                    type="button"
                    size="xs"
                    variant={isSel ? "default" : "outline"}
                    data-client-id={c.id}
                    disabled={isBusyOp}
                    className="max-md:min-h-11"
                    onClick={() => setSelectedClientId(c.id)}
                  >
                    {c.name}
                  </Button>
                );
              })}
            </div>
          )}

          {visibleEngagements.length === 0 ? (
            <p className="rounded-sm border border-hairline bg-board/40 px-3 py-2 text-xs text-ink-2">
              No engagements under this client yet.
            </p>
          ) : (
            <ul className="flex flex-col gap-1.5" role="list" aria-label="Client engagements">
              {visibleEngagements.map((eng) => {
                const isCurrent = eng.id === activeEngagementId;
                const busyThis = switchingEngId === eng.id;
                const rateLabel =
                  typeof eng.hourly_rate_usd === "number" && eng.hourly_rate_usd > 0
                    ? `$${eng.hourly_rate_usd}/hr`
                    : "";
                const budgetLabel =
                  typeof eng.retainer_budget_usd === "number" && eng.retainer_budget_usd > 0
                    ? `$${eng.retainer_budget_usd} cap`
                    : "No retainer cap";
                const prefixLabel = eng.workspace_prefix ? eng.workspace_prefix : "";
                const statusBadge =
                  eng.status && eng.status !== "active" ? eng.status : "";
                const metaLabel = [rateLabel, budgetLabel, prefixLabel].filter(Boolean).join(", ");
                return (
                  <li key={eng.id}>
                    <button
                      type="button"
                      data-engagement-id={eng.id}
                      aria-current={isCurrent ? "true" : undefined}
                      disabled={isBusyOp}
                      onClick={() => void handleSelectEngagement(eng.id)}
                      className={cn(
                        "focus-ring flex min-h-11 w-full cursor-pointer items-center justify-between gap-2.5 rounded-sm border px-3 py-2 text-left outline-none disabled:cursor-wait disabled:bg-track disabled:text-ink-3",
                        isCurrent ? "border-blue bg-blue-soft text-ink" : "border-hairline bg-surface text-ink hover:bg-board"
                      )}
                    >
                      <span className="min-w-0 flex-1">
                        <span className="flex items-center gap-1.5">
                          <span className="truncate text-sm font-medium text-ink">{eng.name}</span>
                          {statusBadge && (
                            <span className="mono rounded-chip bg-track px-1.5 py-0.5 text-[11px] text-ink-2">
                              {statusBadge}
                            </span>
                          )}
                        </span>
                        {metaLabel && <span className="mono block truncate text-xs text-ink-2">{metaLabel}</span>}
                      </span>
                      <span className="flex shrink-0 items-center gap-1.5">
                        {busyThis ? (
                          <span className="mono text-xs text-ink-2">Switching…</span>
                        ) : isCurrent ? (
                          <Pill variant="secondary" className="bg-pill-active-bg px-2 py-0.5 text-xs font-medium text-pill-active-fg">
                            <Check className="size-3" strokeWidth={2} aria-hidden />
                            Active
                          </Pill>
                        ) : (
                          <span className="text-xs font-medium text-ink-2">Use</span>
                        )}
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </section>
      )}

      <div className={cn("flex flex-col gap-2.5", clientsEnabled && "border-t border-hairline pt-3")}>
        <p className="text-sm text-ink-2">
          Each person keeps an isolated workspace, file history, learned memory, routines, and SHA-256 receipt ledger.
        </p>

        {people.length > 4 && (
          <InputGroup className="rounded-sm bg-board/50 [&_input]:max-md:min-h-11">
            <InputGroupAddon align="inline-start" className="pl-2.5 text-icon">
              <Search className="size-3.5" strokeWidth={1.5} aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              id="workspace-switcher-search"
              type="search"
              aria-label="Filter workspaces"
              placeholder="Find a person or role…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="text-sm max-md:min-h-11"
            />
            {query && (
              <InputGroupAddon align="inline-end" className="pr-1.5">
                <Button
                  type="button"
                  size="icon-xs"
                  variant="ghost"
                  className="max-md:min-h-11 max-md:min-w-11"
                  aria-label="Clear workspace search"
                  onClick={() => setQuery("")}
                >
                  <X className="size-3.5" strokeWidth={1.5} aria-hidden />
                </Button>
              </InputGroupAddon>
            )}
          </InputGroup>
        )}

        {filtered.length === 0 ? (
          <p className="rounded-sm border border-hairline bg-board/40 px-3 py-2.5 text-sm text-ink-2">
            No workspace matches &ldquo;{query}&rdquo;.
          </p>
        ) : (
          <WorkspaceSwitcherList
            people={filtered}
            active={active}
            switchingId={switchingId || (isBusyOp ? "__busy__" : null)}
            onSwitch={handleSelect}
          />
        )}
      </div>

      <div className="flex flex-wrap items-center justify-between gap-2 border-t border-hairline pt-3">
        <span className="mono flex items-center gap-1.5 text-xs text-ink-2">
          <Users className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
          {people.length} {people.length === 1 ? "workspace" : "workspaces"}
        </span>
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="max-md:min-h-11"
          onClick={() => {
            setIsOpen(false);
            setTimeout(onAddPerson, 30);
          }}
        >
          <UserPlus className="size-3.5" strokeWidth={1.5} aria-hidden />
          Add a person
        </Button>
      </div>
    </div>
  );
}

export function WorkspaceSwitcherPanel({
  people,
  active,
  open,
  onOpenChange,
  onRefresh,
  onSwitch,
  onAddPerson,
  clientsEnabled,
  clients,
  engagements,
  activeEngagementId,
  workspacePolicy,
  activeEngagementContext,
  onRefreshClients,
  onSwitchEngagement,
  onCreateClient,
  onUpdateClient,
  onCreateEngagement,
  onUpdateEngagement,
  onUpdateWorkspacePolicy,
  onLoadEngagementContext,
}: {
  people: Person[];
  active: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRefresh: () => void;
  onSwitch: (id: string) => Promise<void>;
  onAddPerson: () => void;
  clientsEnabled?: boolean;
  clients?: Client[];
  engagements?: Engagement[];
  activeEngagementId?: string;
  workspacePolicy?: WorkspacePolicy | null;
  activeEngagementContext?: EngagementContext | null;
  onRefreshClients?: () => void;
  onSwitchEngagement?: (id: string) => Promise<void>;
  onCreateClient?: (input: { name: string; brand_header?: string; default_rate_usd?: number }) => Promise<Client | null>;
  onUpdateClient?: (
    id: string,
    patch: {
      name?: string;
      brand_header?: string;
      billing_currency?: string;
      default_rate_usd?: number;
      contacts?: ClientContact[];
      notes_md?: string;
      tax_rate_pct?: number;
      payment_terms?: string;
    }
  ) => Promise<Client | null>;
  onCreateEngagement?: (input: {
    client_id: string;
    name: string;
    person_id?: string;
    hourly_rate_usd?: number;
    retainer_budget_usd?: number;
  }) => Promise<Engagement | null>;
  onUpdateEngagement?: (
    id: string,
    patch: {
      name?: string;
      retainer_hours?: number;
      retainer_budget_usd?: number;
      hourly_rate_usd?: number;
      period?: string;
      workspace_prefix?: string;
      sow_rules?: string[];
      rollover_mode?: RolloverMode;
      status?: string;
      archived_ms?: number | null;
    }
  ) => Promise<Engagement | null>;
  onUpdateWorkspacePolicy?: (patch: { isolate_active_engagement?: boolean }) => Promise<WorkspacePolicy | null>;
  onLoadEngagementContext?: (id: string) => Promise<EngagementContext | null>;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Switch workspace"
      description="Choose a client engagement or person workspace to work inside."
      className="max-w-xl"
    >
      <WorkspaceSwitcherBody
        people={people}
        active={active}
        onRefresh={onRefresh}
        onSwitch={onSwitch}
        onAddPerson={onAddPerson}
        clientsEnabled={clientsEnabled}
        clients={clients}
        engagements={engagements}
        activeEngagementId={activeEngagementId}
        workspacePolicy={workspacePolicy}
        activeEngagementContext={activeEngagementContext}
        onRefreshClients={onRefreshClients}
        onSwitchEngagement={onSwitchEngagement}
        onCreateClient={onCreateClient}
        onUpdateClient={onUpdateClient}
        onCreateEngagement={onCreateEngagement}
        onUpdateEngagement={onUpdateEngagement}
        onUpdateWorkspacePolicy={onUpdateWorkspacePolicy}
        onLoadEngagementContext={onLoadEngagementContext}
      />
    </PanelModal>
  );
}
