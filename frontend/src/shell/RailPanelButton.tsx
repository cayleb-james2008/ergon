import type React from "react";

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
          {count}
        </span>
      )}
    </button>
  );
}
