// The "Working for" switcher: a Popover over a Command list (house parts), replacing the native select.
import { useState } from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import type { Person } from "@/lib/api";
import { cn } from "@/lib/utils";

export function personLabel(p: Person): string {
  return p.role ? p.name + ", " + p.role : p.name;
}

export function PersonSelect({ people, active, onChange, disabled }: {
  people: Person[]; active: string; onChange: (id: string) => void; disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const current = people.find((p) => p.id === active);
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger id="person-select" aria-labelledby="person-label" aria-haspopup="listbox" disabled={disabled}
        className="focus-ring mt-1 flex h-10 w-full items-center justify-between gap-2 rounded-sm border border-ink-3 bg-surface px-2.5 text-left text-sm text-ink outline-none disabled:bg-track disabled:text-ink-3">
        <span className="min-w-0 truncate">{current ? personLabel(current) : "Choose a person"}</span>
        <ChevronsUpDown className="size-4 shrink-0 text-icon" strokeWidth={1.75} aria-hidden />
      </PopoverTrigger>
      <PopoverContent align="start" sideOffset={4} className="w-[var(--radix-popover-trigger-width)] min-w-56 rounded-sm border border-hairline bg-surface p-0 text-ink shadow-lift">
        <Command>
          {people.length > 6 && <CommandInput placeholder="Find a person" className="h-10 text-sm" />}
          <CommandList className="max-h-72 p-1">
            <CommandEmpty className="px-2.5 py-2 text-sm text-ink-2">No one matches.</CommandEmpty>
            {people.map((p) => (
              <CommandItem key={p.id} value={personLabel(p) + " " + p.id} onSelect={() => { setOpen(false); onChange(p.id); }}
                className={cn("flex min-h-10 cursor-pointer items-center gap-2 rounded-sm px-2.5 text-sm text-ink data-[selected=true]:bg-board", p.id === active && "font-medium")}>
                <Check className={cn("size-4 shrink-0 text-ink", p.id !== active && "invisible")} strokeWidth={2} aria-hidden />
                <span className="min-w-0 truncate">{personLabel(p)}</span>
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
