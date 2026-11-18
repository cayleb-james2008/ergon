//! Embedded web UI — plain HTML/CSS/JS, no framework, no build step.
//! Familiar chat shell: left sidebar with threads, one scrolling thread,
//! one composer row pinned at the bottom. Light + dark themes (dark is
//! the default), the switch in Settings; one agent per person, switched
//! in the sidebar; what the agent has learned is viewable under Memory.

pub const PHONE_HTML: &str = include_str!("../assets/phone.html");

pub const INDEX_HTML: &str = include_str!("../assets/index.html");

/*
 * hallmark-rune override log — Ergon (kept Reflex items, on the record)
 *
 * override · R9 · guard: the sidebar is a thread list, and the list IS
 *   the work (the category pattern the objective mandates) · reason:
 *   the rail is earned here; on small widths it collapses to a compact
 *   head block rather than being removed.
 * override · R1 · guard: step rows and the memory file view render in
 *   mono · reason: the ledger IS data returned by the API (tool calls
 *   with real targets, the verbatim memory file); every UI label,
 *   heading and button stays in the body face.
 * refused · R5 · tracked-caps kind labels ("WORK", "SKILL") read as
 *   emphasis: hierarchy comes from size, weight and position — the
 *   labels are sentence case, untracked, one step smaller.
 * refused · R4 · pill theme buttons: the theme buttons are discrete
 *   bordered buttons at the control radius; pills remain only on
 *   count badges, which are chips by job.
 * refused · R3 · kind labels in the accent: a category word is not
 *   action, selection or live state, so kind labels (suite, stream,
 *   phone items) render in the muted ink, not the accent.
 */
