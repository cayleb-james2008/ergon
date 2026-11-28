import { cn } from "@/lib/utils";

/**
 * Ergon geometric brand mark on a 24x24 square grid:
 * 3 stepped horizontal work-beams anchored to a structural spine, paired with
 * a detached 4x4 verified-receipt square. Uses `currentColor` to match ink tokens.
 */
export function ErgonMark({
  className,
  label,
}: {
  className?: string;
  label?: string;
}) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      className={cn("shrink-0", className)}
    >
      <path
        d="M4.25 3h15.5C20.44 3 21 3.56 21 4.25v1.5C21 6.44 20.44 7 19.75 7H7v3h6.25c.69 0 1.25.56 1.25 1.25v1.5c0 .69-.56 1.25-1.25 1.25H7v3h12.75c.69 0 1.25.56 1.25 1.25v1.5c0 .69-.56 1.25-1.25 1.25H4.25C3.56 21 3 20.44 3 19.75V4.25C3 3.56 3.56 3 4.25 3Z"
        fill="currentColor"
      />
      <rect x="17" y="10" width="4" height="4" rx="1.1" fill="currentColor" />
    </svg>
  );
}

/**
 * Horizontal Ergon brand lockup pairing the geometric mark with a tight
 * wordmark using house ink tokens.
 */
export function ErgonLogo({
  size = "md",
  markOnly = false,
  className,
}: {
  size?: "sm" | "md" | "lg";
  markOnly?: boolean;
  className?: string;
}) {
  const markSize = size === "sm" ? "size-4" : size === "lg" ? "size-6" : "size-[18px]";
  const textSize =
    size === "sm"
      ? "text-[13px]"
      : size === "lg"
        ? "text-base"
        : "text-[15px]";

  return (
    <span className={cn("inline-flex items-center gap-2 text-ink select-none", className)}>
      <ErgonMark className={markSize} />
      {markOnly ? (
        <span className="sr-only">Ergon</span>
      ) : (
        <span className={cn("font-semibold leading-none tracking-[-0.03em] text-ink", textSize)}>
          Ergon
        </span>
      )}
    </span>
  );
}
