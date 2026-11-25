// The one source of motion values for screen code (Emil Kowalski Motion Contract v2.0).
// CSS has the same values in tokens.css (--dur-press, --dur-reveal, --dur-swap, --dur-enter,
// --dur-sheet and --ease-house); Tailwind takes them as duration-(--dur-press) and ease-(--ease-house).
// Cause first: anything used 100+/day or triggered from the keyboard uses `instant`.

/** Strong exponential ease-out for UI entrances and exits. Equals --ease-house. */
export const ease = [0.23, 1, 0.32, 1] as [number, number, number, number];

/** Seconds. Equals --dur-press (150ms), --dur-reveal (180ms), --dur-swap (200ms), --dur-enter (220ms), --dur-sheet (220ms). */
export const dur = { press: 0.15, reveal: 0.18, swap: 0.2, enter: 0.22, sheet: 0.22 };

/** Press feedback on a button or chip. */
export const press = { duration: dur.press, ease };

/** Tab pill, segmented indicator, or active rail highlight glide (interruptible spring). */
export const glide = { type: "spring" as const, stiffness: 450, damping: 35, mass: 0.7 };

/** Modal/dialog entrance (opacity + 0.97->1 scale / 4-6px lift, never scale(0)). */
export const open = { duration: dur.enter, ease };

/** Drawer/sheet slide & fade. */
export const sheet = { duration: dur.sheet, ease };

/** Popover, tooltip, or disclosure reveal. */
export const reveal = { duration: dur.reveal, ease };

/** No motion: keyboard shortcuts, initial load, and prefers-reduced-motion. */
export const instant = { duration: 0 };
