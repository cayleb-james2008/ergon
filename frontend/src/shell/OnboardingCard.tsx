// First-run 3-step employee orientation:
// - Non-blocking inline card (`OnboardingCard`) shown on fresh workspaces until completed/dismissed
//   (never blocks composer, rail, or automated clicks).
// - Re-openable modal (`OnboardingModal`) launched from Help ("Replay getting started guide").
import { useState } from "react";
import {
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  FolderOpen,
  LifeBuoy,
  ShieldCheck,
  Sparkles,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { PanelModal } from "@/shell/PanelModal";
import { ErgonMark } from "@/shell/ErgonLogo";
import type { AdminContactConfig } from "@/lib/api";
import { cn } from "@/lib/utils";

type OnboardingStep = {
  id: string;
  shortLabel: string;
  title: string;
  summary: string;
  bullets: string[];
};

const STEPS: OnboardingStep[] = [
  {
    id: "workspace",
    shortLabel: "1. Workspace",
    title: "1. Your workspace & files",
    summary:
      "Every document, spreadsheet, or note you attach or create lives safely inside your private workspace folder.",
    bullets: [
      "Drag and drop spreadsheets, contracts, PDFs, or meeting notes directly into the chat box.",
      "Open the Workspace panel on the right to preview files, check version history, or restore an earlier copy in one click.",
      "Your agent never touches files outside your assigned workspace folder.",
    ],
  },
  {
    id: "workflows",
    shortLabel: "2. Workflows",
    title: "2. Built-in workflows & approvals",
    summary:
      "Use 12 ready-made workflows for reports, invoice reconciliation, contract checks, and recaps — with human approval before anything risky.",
    bullets: [
      "Open Workflows in the left sidebar or pick a quick-start chip below the prompt box to start immediately.",
      "Ergon pauses and asks for your approval before deleting files or sending external emails and webhooks.",
      "Review pending approvals and completed receipts anytime in the Today tab.",
    ],
  },
  {
    id: "help",
    shortLabel: "3. Help & IT",
    title: "3. Getting help & admin contact",
    summary:
      "If something looks stuck or disconnected, Help gives you plain-language fixes, your IT contact, and a one-click diagnostic bundle.",
    bullets: [
      "Click Help & support in the left sidebar anytime to see troubleshooting tips and your admin's contact details.",
      "Click Download IT Diagnostic Bundle to save a single ZIP with recent logs and crash reports for IT.",
      "API keys, passwords, and private tokens are automatically removed before the diagnostic ZIP is created.",
    ],
  },
];

function StepContent({
  stepIndex,
  setStepIndex,
  adminContact,
  saving,
  onComplete,
  onOpenFiles,
  onOpenSkills,
  onOpenHelp,
  compact = false,
}: {
  stepIndex: number;
  setStepIndex: (idx: number) => void;
  adminContact?: AdminContactConfig | null;
  saving?: boolean;
  onComplete: () => void;
  onOpenFiles?: () => void;
  onOpenSkills?: () => void;
  onOpenHelp?: () => void;
  compact?: boolean;
}) {
  const step = STEPS[stepIndex] ?? STEPS[0];
  const isFirst = stepIndex === 0;
  const isLast = stepIndex === STEPS.length - 1;
  const StepIcon = stepIndex === 0 ? FolderOpen : stepIndex === 1 ? ShieldCheck : LifeBuoy;

  return (
    <div className="flex min-w-0 flex-col gap-3">
      <div className="flex flex-wrap items-center gap-1.5" role="tablist" aria-label="Getting started steps">
        {STEPS.map((s, idx) => {
          const active = idx === stepIndex;
          const done = idx < stepIndex;
          return (
            <button
              key={s.id}
              type="button"
              role="tab"
              aria-selected={active}
              onClick={() => setStepIndex(idx)}
              className={cn(
                "focus-ring inline-flex min-h-11 items-center gap-1.5 rounded-chip border px-2.5 py-1 text-xs font-medium outline-none md:min-h-8",
                active
                  ? "border-primary bg-primary text-primary-foreground"
                  : done
                    ? "border-hairline bg-board text-ink hover:bg-track"
                    : "border-hairline bg-surface text-ink-2 hover:bg-board hover:text-ink"
              )}
            >
              {done && <CheckCircle2 className="size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />}
              <span>{s.shortLabel}</span>
            </button>
          );
        })}
      </div>

      <div className="flex min-w-0 flex-col gap-2 rounded-card border border-hairline bg-board/45 p-3.5">
        <div className="flex items-start gap-2.5">
          <span className="mt-0.5 grid size-7 shrink-0 place-items-center rounded-chip border border-hairline bg-surface text-ink">
            <StepIcon className="size-4" strokeWidth={1.5} aria-hidden />
          </span>
          <div className="min-w-0 flex-1">
            <h3 className="text-sm font-medium text-ink">{step.title}</h3>
            <p className="mt-0.5 text-sm leading-5 text-ink-2">{step.summary}</p>
          </div>
        </div>

        <ul className="flex flex-col gap-1.5 pl-1 text-xs leading-5 text-ink-2">
          {step.bullets.map((line) => (
            <li key={line} className="flex items-start gap-2">
              <span aria-hidden className="mt-2 size-1.5 shrink-0 rounded-full bg-ink-3" />
              <span className="min-w-0 flex-1">{line}</span>
            </li>
          ))}
        </ul>

        {stepIndex === 2 && adminContact?.configured && (
          <div className="mt-1 rounded-sm border border-hairline bg-surface px-3 py-2 text-xs text-ink-2">
            <span className="font-medium text-ink">Your IT contact: </span>
            {[adminContact.name, adminContact.email, adminContact.phone].filter(Boolean).join(" — ") ||
              "Configured in Help"}
          </div>
        )}

        <div className="flex flex-wrap items-center gap-2 pt-1">
          {stepIndex === 0 && onOpenFiles && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5"
              onClick={onOpenFiles}
            >
              <FolderOpen className="size-3.5" strokeWidth={1.5} aria-hidden />
              Open workspace files
            </Button>
          )}
          {stepIndex === 1 && onOpenSkills && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5"
              onClick={onOpenSkills}
            >
              <Sparkles className="size-3.5" strokeWidth={1.5} aria-hidden />
              Browse workflows
            </Button>
          )}
          {stepIndex === 2 && onOpenHelp && (
            <Button
              type="button"
              size="xs"
              variant="outline"
              className="max-md:min-h-11 gap-1.5"
              onClick={onOpenHelp}
            >
              <LifeBuoy className="size-3.5" strokeWidth={1.5} aria-hidden />
              Open Help & IT support
            </Button>
          )}
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-2 pt-0.5">
        <span className="mono text-xs text-ink-3">
          Step {stepIndex + 1} of {STEPS.length}
        </span>
        <div className="flex flex-wrap items-center gap-2">
          {!isFirst && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="max-md:min-h-11 gap-1"
              onClick={() => setStepIndex(Math.max(0, stepIndex - 1))}
            >
              <ChevronLeft className="size-3.5" strokeWidth={1.5} aria-hidden />
              Back
            </Button>
          )}
          {!isLast ? (
            <Button
              id={compact ? "onboarding-next-btn" : "onboarding-modal-next-btn"}
              type="button"
              size="sm"
              className="max-md:min-h-11 gap-1"
              onClick={() => setStepIndex(Math.min(STEPS.length - 1, stepIndex + 1))}
            >
              Next
              <ChevronRight className="size-3.5" strokeWidth={1.5} aria-hidden />
            </Button>
          ) : (
            <Button
              id={compact ? "onboarding-finish-btn" : "onboarding-modal-finish-btn"}
              type="button"
              size="sm"
              disabled={saving}
              className="max-md:min-h-11 gap-1.5"
              onClick={onComplete}
            >
              <CheckCircle2 className="size-3.5" strokeWidth={1.5} aria-hidden />
              {saving ? "Saving…" : "Got it, finish guide"}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

export function OnboardingCard({
  adminContact,
  saving,
  onComplete,
  onOpenFiles,
  onOpenSkills,
  onOpenHelp,
}: {
  adminContact?: AdminContactConfig | null;
  saving?: boolean;
  onComplete: () => void;
  onOpenFiles?: () => void;
  onOpenSkills?: () => void;
  onOpenHelp?: () => void;
}) {
  const [stepIndex, setStepIndex] = useState(0);

  return (
    <section
      id="onboarding-card"
      aria-labelledby="onboarding-heading"
      className="flex w-full min-w-0 flex-col gap-3 rounded-dialog border border-hairline bg-surface p-4 shadow-card"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <h2 id="onboarding-heading" className="flex items-center gap-2 text-sm font-medium text-ink">
            <ErgonMark className="size-4 shrink-0 text-ink" />
            <span>Welcome to Ergon — 3-step quick start</span>
          </h2>
          <p className="mt-0.5 text-xs text-ink-2">
            A quick plain-language tour of your workspace. You can re-open this anytime from Help.
          </p>
        </div>
        <Button
          id="onboarding-dismiss-btn"
          type="button"
          size="xs"
          variant="ghost"
          aria-label="Skip getting started guide"
          disabled={saving}
          onClick={onComplete}
          className="max-md:min-h-11 shrink-0 gap-1 text-ink-2 hover:text-ink"
        >
          <span>Skip</span>
          <X className="size-3.5" strokeWidth={1.5} aria-hidden />
        </Button>
      </div>

      <StepContent
        stepIndex={stepIndex}
        setStepIndex={setStepIndex}
        adminContact={adminContact}
        saving={saving}
        onComplete={onComplete}
        onOpenFiles={onOpenFiles}
        onOpenSkills={onOpenSkills}
        onOpenHelp={onOpenHelp}
        compact
      />
    </section>
  );
}

export function OnboardingModal({
  open,
  onOpenChange,
  adminContact,
  saving,
  onComplete,
  onOpenFiles,
  onOpenSkills,
  onOpenHelp,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  adminContact?: AdminContactConfig | null;
  saving?: boolean;
  onComplete: () => void;
  onOpenFiles?: () => void;
  onOpenSkills?: () => void;
  onOpenHelp?: () => void;
}) {
  const [stepIndex, setStepIndex] = useState(0);

  return (
    <PanelModal
      open={open}
      onOpenChange={(next) => {
        if (!next) setStepIndex(0);
        onOpenChange(next);
      }}
      title="Getting started with Ergon"
      description="Three quick essentials for using your personal workspace safely."
      panelId="onboarding-modal"
      className="max-w-xl"
    >
      <StepContent
        stepIndex={stepIndex}
        setStepIndex={setStepIndex}
        adminContact={adminContact}
        saving={saving}
        onComplete={() => {
          onComplete();
          onOpenChange(false);
          setStepIndex(0);
        }}
        onOpenFiles={
          onOpenFiles
            ? () => {
                onOpenChange(false);
                onOpenFiles();
              }
            : undefined
        }
        onOpenSkills={
          onOpenSkills
            ? () => {
                onOpenChange(false);
                onOpenSkills();
              }
            : undefined
        }
        onOpenHelp={
          onOpenHelp
            ? () => {
                onOpenChange(false);
                onOpenHelp();
              }
            : undefined
        }
      />
    </PanelModal>
  );
}
