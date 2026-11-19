// The three panels that grow out of the rail (Motion Primitives MorphingDialog): what the agent learned,
// my suite and the library, and add a person. Their words are the old page's.
import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogDescription, MorphingDialogTitle, MorphingDialogTrigger } from "@/components/ui/morphing-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Details } from "@/chat/Entry";
import { call, type LibraryItem, type Msg, type Person, type SuiteItem } from "@/lib/api";

const TRIGGER = "flex w-full min-h-10 items-center justify-between gap-2 rounded-sm px-2 text-left text-sm text-ink outline-none hover:bg-board focus-visible:ring-2 focus-visible:ring-blue";
const SPRING = { type: "spring" as const, bounce: 0.05, duration: 0.3 };

function Panel({ id, label, title, children }: { id: string; label: string; title: string; children: React.ReactNode }) {
  return (
    <MorphingDialog transition={SPRING}>
      <MorphingDialogTrigger thing={id} className={TRIGGER} style={{ borderRadius: 6 }}>
        <span>{label}</span>
      </MorphingDialogTrigger>
      <MorphingDialogContainer>
        <MorphingDialogContent style={{ borderRadius: 18 }} className="relative flex max-h-[80vh] w-[520px] max-w-full flex-col gap-3 overflow-auto border border-hairline bg-surface p-5 text-ink shadow-lift">
          <MorphingDialogClose className="absolute right-3 top-3 grid size-9 place-items-center rounded-sm text-icon outline-none hover:bg-board focus-visible:ring-2 focus-visible:ring-blue" aria-label="Close" />
          <MorphingDialogTitle className="pr-10 text-title font-semibold text-ink">{title}</MorphingDialogTitle>
          <MorphingDialogDescription disableLayoutAnimation className="text-sm text-ink-2">{children}</MorphingDialogDescription>
        </MorphingDialogContent>
      </MorphingDialogContainer>
    </MorphingDialog>
  );
}

export function LearnedPanel() {
  const [text, setText] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    call<{ content: string }>("/api/memory").then((r) => { if (r.ok) setText((r.data.content || "").trim()); else setFailed(true); });
  }, []);
  const learned = (text ?? "").split("\n").some((l) => { const t = l.trim(); return t.startsWith("- ") && !t.startsWith("- Name:") && !t.startsWith("- Role:"); });
  return (
    <Panel id="learned" label="What my agent has learned" title="What your agent has learned">
      {failed ? "Memory could not be loaded. Is the Ergon server running?"
        : text === null ? "Loading…"
        : !learned ? "Nothing learned yet. Preferences and facts your agent picks up as it works will appear here."
        : <LearnedList content={text} />}
    </Panel>
  );
}

function LearnedList({ content }: { content: string }) {
  const out: React.ReactNode[] = []; let list: string[] = [];
  const flush = (k: number) => { if (list.length) { out.push(<ul key={"l" + k} className="flex list-disc flex-col gap-1.5 pl-5 text-ui text-ink">{list.map((x, i) => <li key={i}>{x}</li>)}</ul>); list = []; } };
  content.split("\n").forEach((raw, k) => {
    const line = raw.trim();
    if (line.startsWith("## ")) { flush(k); out.push(<p key={"h" + k} className="pt-2 text-sm font-medium text-ink">{line.slice(3)}</p>); }
    else if (line.startsWith("- ")) list.push(line.slice(2));
    else if (line && !line.startsWith("# ")) { flush(k); out.push(<p key={"p" + k} className="text-ui text-ink">{line}</p>); }
  });
  flush(999);
  return <div className="flex flex-col gap-2"><div>{out}</div><Details><pre className="whitespace-pre-wrap break-words font-mono text-xs text-ink-2">{content}</pre></Details></div>;
}

export function SuitePanel({ onRun }: { onRun: (id: string, msgs: Msg[]) => void }) {
  const [suite, setSuite] = useState<SuiteItem[] | null>(null);
  const [lib, setLib] = useState<LibraryItem[]>([]);
  const [err, setErr] = useState("");
  const load = async () => {
    const [s, l] = await Promise.all([call<{ installed: SuiteItem[] }>("/api/suite"), call<{ items: LibraryItem[] }>("/api/library")]);
    if (!s.ok || !l.ok) { setErr("The suite could not be loaded. Check that Ergon is running."); return; }
    setErr(""); setSuite(s.data.installed ?? []); setLib(l.data.items ?? []);
  };
  useEffect(() => { load(); }, []);
  const ids = (suite ?? []).map((i) => i.id);
  const available = lib.filter((i) => !i.builtin && !ids.includes(i.id));
  return (
    <Panel id="suite" label="My suite and the library" title="My suite and the library">
      {err ? err : suite === null ? "Loading…" : (
        <div className="flex flex-col gap-4">
          <section className="flex flex-col gap-3">
            <p className="text-sm font-medium text-ink">In your suite</p>
            {!suite.length && <p>Nothing yet. Add an item from the library below.</p>}
            {suite.map((item) => <SuiteRow key={item.id} item={item} onRemove={async () => { await call("/api/suite/remove", { method: "POST", body: { item_id: item.id } }); load(); }} onRun={onRun} />)}
          </section>
          <section className="flex flex-col gap-3">
            <p className="text-sm font-medium text-ink">From the company library</p>
            {!available.length && <p>Everything in the library is already in your suite.</p>}
            {available.map((item) => (
              <div key={item.id} className="flex flex-col gap-1.5 border-t border-hairline pt-3">
                <div className="flex items-center gap-2"><Badge variant="outline" size="sm">{item.kind}</Badge><span className="text-ui text-ink">{item.name}</span></div>
                <p className="text-sm text-ink-2">{item.description} <span>by {item.author}</span></p>
                <div><Button size="sm" variant="outline" onClick={async () => { await call("/api/suite/install", { method: "POST", body: { item_id: item.id } }); load(); }}>Add to my suite</Button></div>
              </div>
            ))}
          </section>
        </div>
      )}
    </Panel>
  );
}

function SuiteRow({ item, onRemove, onRun }: { item: SuiteItem; onRemove: () => void; onRun: (id: string, msgs: Msg[]) => void }) {
  const [fields, setFields] = useState<Record<string, string>>({});
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState("");
  const run = async () => {
    setRunning(true); setErr("");
    const r = await call<{ id: string; messages: Msg[] }>("/api/apps/" + encodeURIComponent(item.id) + "/run", { method: "POST", body: { fields } });
    setRunning(false);
    if (!r.ok) { setErr(r.error || "That app could not run."); return; }
    onRun(r.data.id, r.data.messages ?? []);
  };
  return (
    <div className="flex flex-col gap-2 rounded-card border border-hairline p-3">
      <div className="flex items-center gap-2"><Badge variant="outline" size="sm">{item.kind}</Badge><span className="text-ui text-ink">{item.name}</span></div>
      <p className="text-sm text-ink-2">{item.description}</p>
      {item.kind === "app" && (item.fields ?? []).map((f) => (
        <Input key={f} aria-label={f} placeholder={f} value={fields[f] ?? ""} onChange={(e) => setFields({ ...fields, [f]: e.target.value })} />
      ))}
      {err && <p role="alert" className="text-sm text-ink">{err}</p>}
      <div className="flex gap-2">
        {item.kind === "app" && <Button size="sm" disabled={running} onClick={run}>{running ? "Running…" : "Run"}</Button>}
        <Button size="sm" variant="outline" onClick={onRemove}>Remove</Button>
      </div>
    </div>
  );
}

/** Add a person: the old page's inline form, opened from the rail row (not a dialog: its close key did not reach the library dialog). */
export function AddPersonPanel({ onAdded }: { onAdded: (p: Person) => void }) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState(""); const [role, setRole] = useState(""); const [err, setErr] = useState("");
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) { setErr("Give the person a name first."); return; }
    const r = await call<Person>("/api/people", { method: "POST", body: { name: name.trim(), role: role.trim() } });
    if (!r.ok) { setErr(r.status === 0 ? "Could not reach the Ergon server." : r.error || "That person could not be added."); return; }
    await call("/api/active-person", { method: "POST", body: { id: r.data.id } });
    setName(""); setRole(""); setErr(""); setOpen(false);
    onAdded(r.data);
  };
  return (
    <div className="flex flex-col">
      <button type="button" aria-expanded={open} aria-controls="person-form" onClick={() => setOpen(!open)} className={TRIGGER}>
        <span>Add a person</span><Plus className="size-4 text-icon" strokeWidth={1.75} aria-hidden />
      </button>
      {open && (
        <form id="person-form" onSubmit={submit} className="mt-2 flex flex-col gap-2 px-2 pb-2">
          <label className="sr-only" htmlFor="person-name">Name</label>
          <Input id="person-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="Name" autoComplete="off" maxLength={80} autoFocus />
          <label className="sr-only" htmlFor="person-role">Role</label>
          <Input id="person-role" value={role} onChange={(e) => setRole(e.target.value)} placeholder="Role (optional)" autoComplete="off" maxLength={120} />
          {err && <p role="alert" className="text-sm text-ink">{err}</p>}
          <div><Button size="sm" type="submit">Add person</Button></div>
        </form>
      )}
    </div>
  );
}
