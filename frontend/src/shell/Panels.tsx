// Rail capability & person dialogs (Motion Primitives MorphingDialog with @/motion `open` preset):
// 1. LearnedPanel ("What my agent has learned" + count chip)
// 2. SuitePanel ("My suite and the library" + count chip, install/remove/run built-in workflows)
// 3. AddPersonPanel ("Add a person" MorphingDialog form with clean close & focus return)
import { useEffect, useState } from "react";
import { Blocks, BookOpen, CheckCircle2, ShieldAlert, UserPlus } from "lucide-react";
import { toast } from "sonner";
import { PanelModal, usePanelModalClose } from "@/shell/PanelModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Details } from "@/chat/Entry";
import {
  call,
  type LibraryItem,
  type MemoryHealth,
  type Msg,
  type Person,
  type SuiteItem,
} from "@/lib/api";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board cursor-pointer";

export function RailPanelButton({
  id,
  domId,
  label,
  icon,
  count,
  issueCount,
  answers,
  expanded,
  onClick,
}: {
  id: string;
  domId?: string;
  label: string;
  icon: React.ReactNode;
  count?: number;
  issueCount?: number;
  answers?: string;
  expanded?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      id={domId}
      data-thing={id}
      aria-haspopup="dialog"
      aria-expanded={expanded}
      onClick={onClick}
      className={TRIGGER}
      style={{ borderRadius: 7 }}
    >
      <span className="flex min-w-0 items-center gap-1.5" data-answers={answers}>
        {icon}
        <span className="leading-tight">{label}</span>
      </span>
      <span className="flex shrink-0 items-center gap-1">
        {typeof issueCount === "number" && issueCount > 0 && (
          <span
            data-memory-rail-badge="issues"
            className="mono rounded-chip bg-warn-soft px-1.5 py-0.5 text-[11px] font-medium text-warn"
            title={`${issueCount} memory conflict or stale path`}
          >
            {issueCount} warn
          </span>
        )}
        {typeof count === "number" && (
          <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
            {count}
          </span>
        )}
      </span>
    </button>
  );
}

function PanelShell({
  id,
  label,
  icon,
  count,
  title,
  answers,
  open,
  onOpenChange,
  hideTrigger,
  children,
}: {
  id: string;
  label: string;
  icon: React.ReactNode;
  count?: number;
  title: string;
  answers?: string;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
  children: React.ReactNode;
}) {
  return (
    <PanelModal
      open={open}
      onOpenChange={onOpenChange}
      title={title}
      className="max-w-xl"
      trigger={
        !hideTrigger ? (
          <button type="button" data-thing={id} className={TRIGGER} style={{ borderRadius: 7 }}>
            <span className="flex min-w-0 items-center gap-1.5" data-answers={answers}>
              {icon}
              <span className="leading-tight">{label}</span>
            </span>
            {typeof count === "number" && (
              <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
                {count}
              </span>
            )}
          </button>
        ) : undefined
      }
    >
      {children}
    </PanelModal>
  );
}

function LearnedBody({ onHealed }: { onHealed?: () => void }) {
  const { isOpen } = usePanelModalClose();
  const [text, setText] = useState<string | null>(null);
  const [health, setHealth] = useState<MemoryHealth | null>(null);
  const [healing, setHealing] = useState(false);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    call<{ content?: string; health?: MemoryHealth }>("/api/memory").then((r) => {
      if (r.ok && typeof r.data.content === "string") {
        setFailed(false);
        setText(r.data.content.trim());
        setHealth(r.data.health ?? null);
      } else {
        setFailed(true);
      }
    });
  }, [isOpen]);

  const handleHeal = async () => {
    setHealing(true);
    try {
      const r = await call<{ content?: string; removed_count?: number; health?: MemoryHealth }>(
        "/api/memory/heal",
        {
          method: "POST",
          body: { mode: "all" },
        }
      );
      if (!r.ok) {
        toast.error("Could not clean up stale memory rules.");
        return;
      }
      if (typeof r.data.content === "string") setText(r.data.content.trim());
      if (r.data.health) setHealth(r.data.health);
      const removed = Number(r.data.removed_count ?? 0);
      toast.success(
        removed > 0
          ? `Pruned ${removed} stale or conflicting memory ${removed === 1 ? "rule" : "rules"}.`
          : "Memory is already clean."
      );
      onHealed?.();
    } finally {
      setHealing(false);
    }
  };

  const learned = (text ?? "").split("\n").some((l) => {
    const t = l.trim();
    return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:");
  });

  if (failed) return <p role="alert">Memory could not be loaded. Is the Ergon server running?</p>;
  if (text === null) return <p>Loading…</p>;
  if (!learned) {
    return (
      <div id="memory-panel" className="flex flex-col gap-3">
        <div
          id="memory-health-banner"
          className="flex flex-wrap items-center justify-between gap-2 rounded-entry border border-hairline bg-board px-3 py-2"
        >
          <span className="inline-flex items-center gap-1.5 text-xs text-ink-2">
            <Badge
              data-memory-health="healthy"
              className="border-transparent bg-ok-soft text-ok"
            >
              Healthy
            </Badge>
            <span>No conflicts or broken workspace references.</span>
          </span>
        </div>
        <p>Nothing learned yet. Preferences and facts your agent picks up as it works will appear here.</p>
      </div>
    );
  }
  return <LearnedList content={text} health={health} healing={healing} onHeal={handleHeal} />;
}

export function LearnedPanel({
  count,
  open,
  onOpenChange,
  hideTrigger,
  onHealed,
}: {
  count: number;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
  onHealed?: () => void;
}) {
  return (
    <PanelShell
      id="memory-btn"
      label="What my agent has learned"
      icon={<BookOpen className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
      count={count}
      title="What your agent has learned"
      open={open}
      onOpenChange={onOpenChange}
      hideTrigger={hideTrigger}
    >
      <LearnedBody onHealed={onHealed} />
    </PanelShell>
  );
}

function LearnedList({
  content,
  health,
  healing,
  onHeal,
}: {
  content: string;
  health: MemoryHealth | null;
  healing: boolean;
  onHeal: () => void;
}) {
  const out: React.ReactNode[] = [];
  let list: string[] = [];
  const flush = (k: number) => {
    if (list.length) {
      out.push(
        <ul key={"l" + k} className="flex list-disc flex-col gap-1.5 pl-5 text-sm leading-6 text-ink">
          {list.map((x, i) => <li key={i}>{x}</li>)}
        </ul>
      );
      list = [];
    }
  };
  content.split("\n").forEach((raw, k) => {
    const line = raw.trim();
    if (line.startsWith("## ")) {
      flush(k);
      out.push(<p key={"h" + k} className="pt-2 text-xs font-medium text-ink-2">{line.slice(3)}</p>);
    } else if (line.startsWith("- ")) {
      list.push(line.slice(2));
    } else if (line && !line.startsWith("# ")) {
      flush(k);
      out.push(<p key={"p" + k} className="text-sm text-ink">{line}</p>);
    }
  });
  flush(999);

  const issueCount = health?.issue_count ?? 0;
  const issues = health?.issues ?? [];

  return (
    <div id="memory-panel" className="flex flex-col gap-3">
      {health && (
        <div
          id="memory-health-banner"
          className="flex flex-wrap items-center justify-between gap-2 rounded-entry border border-hairline bg-board px-3 py-2.5"
        >
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            {issueCount === 0 ? (
              <Badge data-memory-health="healthy" className="gap-1 border-transparent bg-ok-soft text-ok">
                <CheckCircle2 className="size-3.5" strokeWidth={1.75} aria-hidden />
                Healthy
              </Badge>
            ) : (
              <Badge data-memory-health="issues" className="gap-1 border-transparent bg-warn-soft text-warn">
                <ShieldAlert className="size-3.5" strokeWidth={1.75} aria-hidden />
                {issueCount} {issueCount === 1 ? "Issue" : "Issues"}
              </Badge>
            )}
            <span className="text-xs text-ink-2">
              {issueCount === 0
                ? "All standing rules and [file:L##] references verified."
                : [
                    health.conflicts > 0 ? `${health.conflicts} conflict${health.conflicts === 1 ? "" : "s"}` : "",
                    health.stale_refs > 0 ? `${health.stale_refs} stale path${health.stale_refs === 1 ? "" : "s"}` : "",
                    health.duplicates > 0 ? `${health.duplicates} duplicate${health.duplicates === 1 ? "" : "s"}` : "",
                  ]
                    .filter(Boolean)
                    .join(" · ")}
            </span>
          </div>
          {issueCount > 0 && (
            <Button
              id="memory-heal-btn"
              type="button"
              size="xs"
              variant="outline"
              disabled={healing}
              onClick={onHeal}
              className="max-md:min-h-11 shrink-0"
            >
              {healing ? "Pruning…" : "Resolve & Prune"}
            </Button>
          )}
        </div>
      )}

      {issues.length > 0 && (
        <ul className="flex flex-col gap-1.5">
          {issues.map((iss, idx) => (
            <li
              key={`${iss.line}-${iss.kind}-${idx}`}
              data-memory-issue={iss.kind}
              className="flex flex-wrap items-start justify-between gap-2 rounded-sm border border-hairline bg-surface px-2.5 py-2 text-xs"
            >
              <div className="min-w-0 flex-1">
                <span className="font-medium text-ink [overflow-wrap:anywhere]">{iss.text}</span>
                <p className="mt-0.5 text-ink-2">{iss.detail}</p>
              </div>
              <span className="mono shrink-0 rounded-chip bg-warn-soft px-1.5 py-0.5 text-[11px] text-warn">
                {iss.kind === "conflict" ? "Conflict" : iss.kind === "stale_ref" ? "Stale path" : "Duplicate"} · L{iss.line}
              </span>
            </li>
          ))}
        </ul>
      )}

      <div className="flex flex-col gap-1.5">{out}</div>
      <Details label="Raw memory.md">
        <pre className="whitespace-pre-wrap break-words rounded-sm bg-board p-2.5 font-mono text-xs text-ink-2">{content}</pre>
      </Details>
    </div>
  );
}

function SuiteBody({ onRun, onChanged }: { onRun: (id: string, msgs: Msg[]) => void; onChanged: () => void }) {
  const { isOpen, setIsOpen } = usePanelModalClose();
  const [suite, setSuite] = useState<SuiteItem[] | null>(null);
  const [lib, setLib] = useState<LibraryItem[]>([]);
  const [err, setErr] = useState("");

  const load = async () => {
    const [s, l] = await Promise.all([
      call<{ installed: SuiteItem[] }>("/api/suite"),
      call<{ items: LibraryItem[] }>("/api/library"),
    ]);
    if (!s.ok || !l.ok || !Array.isArray(s.data.installed) || !Array.isArray(l.data.items)) {
      setErr("The suite could not be loaded. Check that Ergon is running.");
      return;
    }
    setErr("");
    setSuite(s.data.installed);
    setLib(l.data.items);
  };

  useEffect(() => {
    if (isOpen) load();
  }, [isOpen]);

  if (err) return <p role="alert">{err}</p>;
  if (suite === null) return <p>Loading…</p>;

  const ids = suite.map((i) => i.id);
  const available = lib.filter((i) => i && !i.builtin && !ids.includes(i.id));

  return (
    <div id="suite-panel" className="flex flex-col gap-5">
      <section className="flex flex-col gap-2.5">
        <p className="text-xs font-medium text-ink-2">Installed in your suite ({suite.length})</p>
        {!suite.length && <p className="text-sm text-ink-2">Nothing installed yet. Add a skill, specialist, or one-tap app below.</p>}
        {suite.map((item) => (
          <SuiteRow
            key={item.id}
            item={item}
            onRemove={async () => {
              const r = await call("/api/suite/remove", { method: "POST", body: { item_id: item.id } });
              if (r.ok) {
                toast(`Removed ${item.name} from your suite.`);
                await load();
                onChanged();
              }
            }}
            onRun={(id, msgs) => {
              setIsOpen(false);
              onRun(id, msgs);
            }}
          />
        ))}
      </section>
      <section className="flex flex-col gap-2.5">
        <p className="text-xs font-medium text-ink-2">Available in company library ({available.length})</p>
        {!available.length && <p className="text-sm text-ink-2">Everything in the company library is already in your suite.</p>}
        {available.map((item) => (
          <div key={item.id} className="flex flex-col gap-2 rounded-card border border-hairline bg-board/40 p-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Badge variant="outline" size="sm">{item.kind}</Badge>
                <span className="text-sm font-medium text-ink">{item.name}</span>
              </div>
              <span className="mono text-xs text-ink-3">by {item.author}</span>
            </div>
            <p className="text-sm text-ink-2">{item.description}</p>
            <div>
              <Button
                size="sm"
                variant="outline"
                className="max-md:min-h-11"
                onClick={async () => {
                  const r = await call("/api/suite/install", { method: "POST", body: { item_id: item.id } });
                  if (r.ok) {
                    toast.success(`Added ${item.name} to your suite.`);
                    await load();
                    onChanged();
                  }
                }}
              >
                Add to my suite
              </Button>
            </div>
          </div>
        ))}
      </section>
    </div>
  );
}

export function SuitePanel({
  count,
  onRun,
  onChanged,
  open,
  onOpenChange,
  hideTrigger,
}: {
  count: number;
  onRun: (id: string, msgs: Msg[]) => void;
  onChanged: () => void;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
}) {
  return (
    <PanelShell
      id="suite-btn"
      label="My suite and the library"
      icon={<Blocks className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
      count={count}
      title="My suite and the library"
      open={open}
      onOpenChange={onOpenChange}
      hideTrigger={hideTrigger}
    >
      <SuiteBody onRun={onRun} onChanged={onChanged} />
    </PanelShell>
  );
}

function SuiteRow({ item, onRemove, onRun }: { item: SuiteItem; onRemove: () => void; onRun: (id: string, msgs: Msg[]) => void }) {
  const [fields, setFields] = useState<Record<string, string>>({});
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState("");

  const run = async () => {
    setRunning(true);
    setErr("");
    const r = await call<{ id?: string; messages?: Msg[] }>("/api/apps/" + encodeURIComponent(item.id) + "/run", { method: "POST", body: { fields } });
    setRunning(false);
    if (!r.ok || typeof r.data.id !== "string") {
      setErr((!r.ok && r.error) || "That app could not run.");
      return;
    }
    toast.success(`Ran ${item.name}.`);
    onRun(r.data.id, Array.isArray(r.data.messages) ? r.data.messages : []);
  };

  return (
    <div className="flex flex-col gap-2 rounded-card border border-hairline bg-surface p-3">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Badge variant="outline" size="sm">{item.kind}</Badge>
          <span className="text-sm font-medium text-ink">{item.name}</span>
        </div>
        <span className="mono text-xs text-ink-3">{item.id}</span>
      </div>
      <p className="text-sm text-ink-2">{item.description}</p>
      {item.kind === "app" && Array.isArray(item.fields) && item.fields.map((f) => (
        <Input
          key={f}
          aria-label={f}
          placeholder={f}
          value={fields[f] ?? ""}
          onChange={(e) => setFields({ ...fields, [f]: e.target.value })}
        />
      ))}
      {err && <p role="alert" className="text-sm text-destructive-foreground">{err}</p>}
      <div className="flex flex-wrap gap-2">
        {item.kind === "app" && (
          <Button size="sm" className="max-md:min-h-11" disabled={running} onClick={run}>
            {running ? "Running…" : "Run"}
          </Button>
        )}
        <Button size="sm" variant="outline" className="max-md:min-h-11" onClick={onRemove}>Remove</Button>
      </div>
    </div>
  );
}

function AddPersonBody({ onAdded }: { onAdded: (p: Person) => void }) {
  const { setIsOpen } = usePanelModalClose();
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [err, setErr] = useState("");
  const [saving, setSaving] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) { setErr("Give the person a name first."); return; }
    setSaving(true);
    const r = await call<Person>("/api/people", { method: "POST", body: { name: name.trim(), role: role.trim() } });
    setSaving(false);
    if (!r.ok) {
      setErr(r.status === 0 ? "Could not reach the Ergon server." : r.error || "That person could not be added.");
      return;
    }
    if (typeof r.data.id !== "string") {
      setErr("That person could not be added.");
      return;
    }
    await call("/api/active-person", { method: "POST", body: { id: r.data.id } });
    setName("");
    setRole("");
    setErr("");
    setIsOpen(false);
    toast.success(`Created a personal desk for ${r.data.name}.`);
    onAdded(r.data);
  };

  return (
    <form id="person-form" onSubmit={submit} className="flex flex-col gap-3 pt-1">
      <p className="text-sm text-ink-2">Each employee gets an isolated workspace folder, conversation log, built-in workflow suite, and standing memory.</p>
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium text-ink" htmlFor="person-name">Name</label>
        <Input id="person-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. Elena Rostova" autoComplete="off" maxLength={80} />
      </div>
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium text-ink" htmlFor="person-role">Role (optional)</label>
        <Input id="person-role" value={role} onChange={(e) => setRole(e.target.value)} placeholder="e.g. Finance operations" autoComplete="off" maxLength={120} />
      </div>
      {err && <p role="alert" className="text-sm text-destructive-foreground">{err}</p>}
      <div className="flex items-center justify-end gap-2 pt-1">
        <Button type="submit" size="sm" className="max-md:min-h-11" disabled={saving}>
          {saving ? "Adding…" : "Add person"}
        </Button>
      </div>
    </form>
  );
}

export function AddPersonPanel({
  onAdded,
  open,
  onOpenChange,
  hideTrigger,
}: {
  onAdded: (p: Person) => void;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
}) {
  return (
    <PanelShell
      id="add-person-btn"
      label="Add a person"
      icon={<UserPlus className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />}
      title="Add a person"
      open={open}
      onOpenChange={onOpenChange}
      hideTrigger={hideTrigger}
    >
      <AddPersonBody onAdded={onAdded} />
    </PanelShell>
  );
}
