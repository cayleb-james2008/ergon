// Workspace Switcher (v0.7.0):
// - PersonSelect: Left Rail active-desk button triggering the external WorkspaceSwitcherPanel
//   (no Radix Popover nested inside mobile Vaul Drawer.Content, preventing 390x844/320x568 tap traps).
// - WorkspaceSwitcherPanel: MorphingDialog strictly mounted OUTSIDE Vaul <Drawer.Content> (z-50),
//   refreshing GET /api/people on open and switching via POST /api/people/switch (with fallback).
// - WorkspaceSwitcherList: Reusable compact switcher list also embedded inside SettingsPanel.
import { useEffect, useMemo, useState } from "react";
import { Check, ChevronsUpDown, Search, UserPlus, Users, X } from "lucide-react";
import {
  MorphingDialog,
  MorphingDialogClose,
  MorphingDialogContainer,
  MorphingDialogContent,
  MorphingDialogDescription,
  MorphingDialogTitle,
  useMorphingDialog,
} from "@/components/ui/morphing-dialog";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Pill } from "@/components/kibo-ui/pill";
import type { Person } from "@/lib/api";
import { cn } from "@/lib/utils";
import { open as openPreset } from "@/motion";

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
  disabled,
  expanded,
  onOpen,
}: {
  people: Person[];
  active: string;
  disabled: boolean;
  expanded?: boolean;
  onOpen: () => void;
}) {
  const safePeople = Array.isArray(people) ? people : [];
  const current = safePeople.find((p) => p.id === active);
  const currentName = current ? (String(current.name ?? "").trim() || "Unnamed") : "Choose a workspace";
  const currentRole = current ? String(current.role ?? "").trim() : "";

  return (
    <button
      type="button"
      id="person-select"
      data-thing="workspace-btn"
      aria-labelledby="person-label"
      aria-haspopup="dialog"
      aria-expanded={expanded}
      disabled={disabled}
      onClick={onOpen}
      className="focus-ring mt-1 flex min-h-11 w-full cursor-pointer items-center justify-between gap-2.5 rounded-sm border border-hairline bg-board/60 px-2.5 py-1.5 text-left text-sm text-ink outline-none hover:bg-board disabled:cursor-not-allowed disabled:bg-track disabled:text-ink-3 md:min-h-10"
    >
      <span className="flex min-w-0 items-center gap-2.5">
        <span aria-hidden className="grid size-7 shrink-0 place-items-center rounded-full bg-av-sky-bg text-xs font-medium text-av-sky-fg">
          {personInitials(currentName)}
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-medium text-ink">{currentName}</span>
          {currentRole && <span className="block truncate text-xs text-ink-2">{currentRole}</span>}
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
}: {
  people: Person[];
  active: string;
  onRefresh: () => void;
  onSwitch: (id: string) => Promise<void>;
  onAddPerson: () => void;
}) {
  const { isOpen, setIsOpen } = useMorphingDialog();
  const [query, setQuery] = useState("");
  const [switchingId, setSwitchingId] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      onRefresh();
    }
  }, [isOpen, onRefresh]);

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
    if (id === active) {
      setIsOpen(false);
      return;
    }
    setSwitchingId(id);
    await onSwitch(id);
    setSwitchingId(null);
    setIsOpen(false);
  };

  return (
    <div id="workspace-switcher-panel" className="flex flex-col gap-3.5 pt-1">
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
          switchingId={switchingId}
          onSwitch={handleSelect}
        />
      )}

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
}: {
  people: Person[];
  active: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRefresh: () => void;
  onSwitch: (id: string) => Promise<void>;
  onAddPerson: () => void;
}) {
  return (
    <MorphingDialog transition={openPreset} open={open} onOpenChange={onOpenChange}>
      <MorphingDialogContainer>
        <MorphingDialogContent
          style={{ borderRadius: 18 }}
          className="relative mx-4 flex max-h-[84vh] w-[520px] max-w-[calc(100%-2rem)] flex-col gap-3 overflow-y-auto border border-hairline bg-surface p-5 text-ink shadow-lift"
        >
          <MorphingDialogClose className="focus-ring absolute right-3 top-3 grid size-11 place-items-center rounded-sm text-icon outline-none hover:bg-board md:size-9" />
          <MorphingDialogTitle className="pr-10 text-title font-medium text-ink">
            Switch workspace
          </MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">
            <WorkspaceSwitcherBody
              people={people}
              active={active}
              onRefresh={onRefresh}
              onSwitch={onSwitch}
              onAddPerson={onAddPerson}
            />
          </MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}
