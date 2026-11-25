// Accessible, lightweight Dialog modal wrapper (backed by Radix Dialog via @/components/ui/dialog).
// Replaces heavy animation-library wrappers while preserving outside-drawer portal layering,
// controlled/uncontrolled open state, Escape closing, and usePanelModalClose for child forms.
import { createContext, useContext, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";

const ModalCtx = createContext<{ isOpen: boolean; setIsOpen: (open: boolean) => void }>({
  isOpen: false,
  setIsOpen: () => {},
});

export function usePanelModalClose() {
  return useContext(ModalCtx);
}

export function PanelModal({
  open: controlledOpen,
  onOpenChange,
  trigger,
  title,
  description,
  panelId,
  className,
  children,
}: {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  trigger?: React.ReactNode;
  title: string;
  description?: string;
  panelId?: string;
  className?: string;
  children: React.ReactNode;
}) {
  const [innerOpen, setInnerOpen] = useState(false);
  const isOpen = controlledOpen !== undefined ? controlledOpen : innerOpen;
  const setOpen = (next: boolean) => {
    if (controlledOpen === undefined) setInnerOpen(next);
    onOpenChange?.(next);
  };

  return (
    <ModalCtx.Provider value={{ isOpen, setIsOpen: setOpen }}>
      <Dialog open={isOpen} onOpenChange={setOpen}>
        {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
        <DialogContent
          className={cn(
            "max-h-[85vh] max-w-2xl overflow-y-auto rounded-dialog border border-hairline bg-surface p-5 text-ink shadow-lift focus:outline-none motion-safe:transition-opacity motion-safe:duration-(--dur-reveal) motion-safe:ease-(--ease-house) sm:p-6",
            className
          )}
        >
          <DialogTitle className="pr-10 text-title font-medium text-ink">{title}</DialogTitle>
          {description ? (
            <DialogDescription className="text-sm text-ink-2">{description}</DialogDescription>
          ) : (
            <DialogDescription className="sr-only">{title}</DialogDescription>
          )}
          <div id={panelId} className="mt-1">
            {children}
          </div>
        </DialogContent>
      </Dialog>
    </ModalCtx.Provider>
  );
}
