// Rail capability & person dialogs (Motion Primitives MorphingDialog with @/motion `open` preset):
// 1. LearnedPanel ("What my agent has learned" + count chip)
// 2. SuitePanel ("My suite and the library" + count chip, install/remove/run Rune apps)
// 3. AddPersonPanel ("Add a person" MorphingDialog form with clean close & focus return)
import { useEffect, useState } from "react";
import { Blocks, BookOpen, UserPlus } from "lucide-react";
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
import { SlidingNumber } from "@/components/ui/sliding-number";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Details } from "@/chat/Entry";
import { call, type LibraryItem, type Msg, type Person, type SuiteItem } from "@/lib/api";
import { open as openPreset } from "@/motion";

const TRIGGER = "focus-ring flex w-full min-h-11 md:min-h-9 items-center justify-between gap-1.5 rounded-sm px-1.5 text-left text-sm text-ink outline-none hover:bg-board cursor-pointer";

export function RailPanelButton({
  id,
  domId,
  label,
  icon,
  count,
  answers,
  expanded,
  onClick,
}: {
  id: string;
  domId?: string;
  label: string;
  icon: React.ReactNode;
  count?: number;
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
      {typeof count === "number" && (
        <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
          <SlidingNumber value={count} />
        </span>
      )}
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
    <MorphingDialog transition={openPreset} open={open} onOpenChange={onOpenChange}>
      {!hideTrigger && (
        <MorphingDialogTrigger thing={id} className={TRIGGER} style={{ borderRadius: 7 }}>
          <span className="flex min-w-0 items-center gap-1.5" data-answers={answers}>
            {icon}
            <span className="leading-tight">{label}</span>
          </span>
          {typeof count === "number" && (
            <span className="mono shrink-0 rounded-chip bg-track px-1.5 py-0.5 text-xs text-ink-2">
              <SlidingNumber value={count} />
            </span>
          )}
        </MorphingDialogTrigger>
      )}
      <MorphingDialogContainer>
        <MorphingDialogContent
          style={{ borderRadius: 18 }}
          className="relative mx-4 flex max-h-[82vh] w-[520px] max-w-[calc(100vw-2rem)] flex-col gap-3 overflow-y-auto border border-hairline bg-surface p-5 text-ink shadow-lift"
        >
          <MorphingDialogClose className="focus-ring absolute right-3 top-3 grid size-11 place-items-center rounded-sm text-icon outline-none hover:bg-board md:size-9" />
          <MorphingDialogTitle className="pr-10 text-title font-medium text-ink">{title}</MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">
            {children}
          </MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}

function LearnedBody() {
  const { isOpen } = useMorphingDialog();
  const [text, setText] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    call<{ content?: string }>("/api/memory").then((r) => {
      if (r.ok && typeof r.data.content === "string") {
        setFailed(false);
        setText(r.data.content.trim());
      } else {
        setFailed(true);
      }
    });
  }, [isOpen]);

  const learned = (text ?? "").split("\n").some((l) => {
    const t = l.trim();
    return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:");
  });

  if (failed) return <p role="alert">Memory could not be loaded. Is the Ergon server running?</p>;
  if (text === null) return <p>Loading…</p>;
  if (!learned) {
    return <p id="memory-panel">Nothing learned yet. Preferences and facts your agent picks up as it works will appear here.</p>;
  }
  return <LearnedList content={text} />;
}

export function LearnedPanel({
  count,
  open,
  onOpenChange,
  hideTrigger,
}: {
  count: number;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  hideTrigger?: boolean;
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
      <LearnedBody />
    </PanelShell>
  );
}

function LearnedList({ content }: { content: string }) {
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
  return (
    <div id="memory-panel" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1.5">{out}</div>
      <Details label="Raw memory.md">
        <pre className="whitespace-pre-wrap break-words rounded-sm bg-board p-2.5 font-mono text-xs text-ink-2">{content}</pre>
      </Details>
    </div>
  );
}

function SuiteBody({ onRun, onChanged }: { onRun: (id: string, msgs: Msg[]) => void; onChanged: () => void }) {
  const { isOpen, setIsOpen } = useMorphingDialog();
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
  const { setIsOpen } = useMorphingDialog();
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
      <p className="text-sm text-ink-2">Each employee gets an isolated workspace folder, conversation log, Rune skill suite, and standing memory.</p>
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
