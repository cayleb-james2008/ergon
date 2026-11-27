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
            "flex max-h-[85dvh] max-w-2xl flex-col gap-0 overflow-hidden rounded-dialog border border-hairline bg-surface p-0 text-ink shadow-lift focus:outline-none motion-safe:transition-opacity motion-safe:duration-(--dur-reveal) motion-safe:ease-(--ease-house)",
            className
          )}
        >
          <div className="shrink-0 border-b border-hairline px-5 pt-5 pb-3.5 pr-12 sm:px-6 sm:pt-5 sm:pb-4 sm:pr-12">
            <DialogTitle className="min-w-0 text-title font-medium text-ink [overflow-wrap:anywhere]">{title}</DialogTitle>
            {description ? (
              <DialogDescription className="mt-1 min-w-0 text-sm text-ink-2 [overflow-wrap:anywhere]">{description}</DialogDescription>
            ) : (
              <DialogDescription className="sr-only min-w-0">{title}</DialogDescription>
            )}
          </div>
          <div id={panelId} className="min-h-0 w-full flex-1 overflow-x-hidden overflow-y-auto px-5 pt-4 pb-5 sm:px-6 sm:pb-6">
            {children}
          </div>
        </DialogContent>
      </Dialog>
    </ModalCtx.Provider>
  );
}
