// Active person desk card in the Left Rail: displays the active employee's initials & role calmly
// (respecting "One signed-in person per session" in the top bar while keeping /api/people switching clean in the rail).
import { useState } from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import type { Person } from "@/lib/api";
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

export function PersonSelect({ people, active, onChange, disabled }: {
  people: Person[];
  active: string;
  onChange: (id: string) => void;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const safePeople = Array.isArray(people) ? people : [];
  const current = safePeople.find((p) => p.id === active);
  const currentName = current ? (String(current.name ?? "").trim() || "Unnamed") : "Choose a person";
  const currentRole = current ? String(current.role ?? "").trim() : "";

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        id="person-select"
        aria-labelledby="person-label"
        aria-haspopup="listbox"
        disabled={disabled}
        className="focus-ring mt-1 flex min-h-11 w-full items-center justify-between gap-2.5 rounded-sm border border-hairline bg-board/60 px-2.5 py-1.5 text-left text-sm text-ink outline-none hover:bg-board disabled:bg-track disabled:text-ink-3 md:min-h-10"
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
      </PopoverTrigger>
      <PopoverContent align="start" sideOffset={4} className="w-[var(--radix-popover-trigger-width)] min-w-56 rounded-sm border border-hairline bg-surface p-0 text-ink shadow-lift">
        <Command>
          {safePeople.length > 5 && <CommandInput placeholder="Find a person" className="h-10 text-sm" />}
          <CommandList className="max-h-72 p-1">
            <CommandEmpty className="px-2.5 py-2 text-sm text-ink-2">No one matches.</CommandEmpty>
            {safePeople.map((p) => (
              <CommandItem
                key={p.id}
                value={personLabel(p) + " " + p.id}
                onSelect={() => { setOpen(false); onChange(p.id); }}
                className={cn(
                  "flex min-h-10 cursor-pointer items-center gap-2 rounded-sm px-2.5 text-sm text-ink data-[selected=true]:bg-board",
                  p.id === active && "font-medium"
                )}
              >
                <Check className={cn("size-4 shrink-0 text-ink", p.id !== active && "invisible")} strokeWidth={1.75} aria-hidden />
                <span className="min-w-0 truncate">{personLabel(p)}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
