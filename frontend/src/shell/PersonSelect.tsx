// Workspace Switcher (v0.7.0):
// - PersonSelect: Left Rail active-desk button triggering the external WorkspaceSwitcherPanel
//   (no Radix Popover nested inside mobile Vaul Drawer.Content, preventing 390x844/320x568 tap traps).
// - WorkspaceSwitcherPanel: MorphingDialog strictly mounted OUTSIDE Vaul <Drawer.Content> (z-50),
//   refreshing GET /api/people on open and switching via POST /api/people/switch (with fallback).
// - WorkspaceSwitcherList: Reusable compact switcher list also embedded inside SettingsPanel.
import { useEffect, useMemo, useRef, useState } from "react";
import { Briefcase, Check, ChevronsUpDown, Plus, Search, UserPlus, Users, X } from "lucide-react";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Pill } from "@/components/kibo-ui/pill";
import type { Client, Engagement, Person } from "@/lib/api";
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
  onRefreshClients,
  onSwitchEngagement,
  onCreateClient,
  onCreateEngagement,
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
  onRefreshClients?: () => void;
  onSwitchEngagement?: (id: string) => Promise<void>;
  onCreateClient?: (input: { name: string; brand_header?: string; default_rate_usd?: number }) => Promise<Client | null>;
  onCreateEngagement?: (input: {
    client_id: string;
    name: string;
    person_id?: string;
    hourly_rate_usd?: number;
    retainer_budget_usd?: number;
  }) => Promise<Engagement | null>;
}) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const [query, setQuery] = useState("");
  const [switchingId, setSwitchingId] = useState<string | null>(null);
  const [switchingEngId, setSwitchingEngId] = useState<string | null>(null);
  const [selectedClientId, setSelectedClientId] = useState<string>("");
  const [createOpen, setCreateOpen] = useState(false);
  const [createNewClientMode, setCreateNewClientMode] = useState(false);
  const [clientNameInput, setClientNameInput] = useState("");
  const [engagementNameInput, setEngagementNameInput] = useState("");
  const [hourlyRateInput, setHourlyRateInput] = useState("200");
  const [retainerBudgetInput, setRetainerBudgetInput] = useState("2500");
  const [createError, setCreateError] = useState("");
  const [creating, setCreating] = useState(false);
  const opLockRef = useRef(false);
  const isBusyOp = Boolean(switchingId || switchingEngId || creating);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setCreateOpen(false);
      setCreateError("");
      onRefresh();
      onRefreshClients?.();
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
                  setCreateNewClientMode(clients.length === 0);
                  setCreateError("");
                }}
              >
                <Plus className="size-3.5" strokeWidth={1.5} aria-hidden />
                {createOpen ? "Close form" : "New client / engagement"}
              </Button>
            )}
          </div>

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
                const metaLabel = [rateLabel, budgetLabel].filter(Boolean).join(", ");
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
                        <span className="block truncate text-sm font-medium text-ink">{eng.name}</span>
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
  onRefreshClients,
  onSwitchEngagement,
  onCreateClient,
  onCreateEngagement,
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
  onRefreshClients?: () => void;
  onSwitchEngagement?: (id: string) => Promise<void>;
  onCreateClient?: (input: { name: string; brand_header?: string; default_rate_usd?: number }) => Promise<Client | null>;
  onCreateEngagement?: (input: {
    client_id: string;
    name: string;
    person_id?: string;
    hourly_rate_usd?: number;
    retainer_budget_usd?: number;
  }) => Promise<Engagement | null>;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title="Switch workspace"
      description="Choose a client engagement or person workspace to work inside."
      className="max-w-lg"
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
        onRefreshClients={onRefreshClients}
        onSwitchEngagement={onSwitchEngagement}
        onCreateClient={onCreateClient}
        onCreateEngagement={onCreateEngagement}
      />
    </PanelModal>
  );
}
