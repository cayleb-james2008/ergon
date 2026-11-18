# Ledger — Ergon scale-up (append-only)

| row | first failing check + message | commit | outcome | cause / note |
|---|---|---|---|---|
| setup | baseline run: `verify[baseline]: build ok; tests ok; 4 passed; 0 failed` | 162dd57 | kept | starting point recorded before any change |
| B10 | new test sandbox_refuses_symlink_escape FAILED (read through symlink succeeded) | (see git log) | kept | sandbox_path was lexical only; added canonical-ancestor check inside the canonical root |
| B1 | new test threads_survive_restart FAILED (thread gone after restart) | (B1+B2 commit) | kept | Store gained load/save to threads.json (atomic tmp+rename) on every mutation |
| B2 | new test people_are_isolated FAILED (no /api/people, 404) | (same commit — the two changes are one mechanism) | kept | PeopleState in people.json; per-person Config (people/<id>/), runtime per person, seeded memory |
| B3 | new test spreadsheet_skill_writes_real_csv FAILED (unknown tool) | (B3-B6 commit) | kept | make_spreadsheet Rune skill, RFC 4180 quoting proven byte-exact |
| B4 | new test search_and_move_tools_work FAILED (unknown tool) | (same commit) | kept | search_files: recursive, case-insensitive, symlink-safe, capped |
| B5 | (same test) | (same commit) | kept | move_file with both ends sandbox-checked |
| B6 | new test research_skill_compiles_notes_from_pages FAILED, then Err("Cannot read, value is M-000000") | (same commit) | kept | research Rune skill; root cause: passing `url` to a host fn moves it in Rune — heading is now built before the fetch (bisected V1–V6 in a scratch harness) |
| B7 | new test learned_preferences_shape_the_next_task FAILED (no preferences section in prompt) | (B7 commit) | kept | standing_preferences() extracts learned bullets (outside ## About); system prompt carries them as binding instructions; /api/memory shows the person what was learned |
| B8 | new test shell_has_themes_settings_people_and_memory_view FAILED (no data-theme, no settings) | (B8+B9 commit) | kept | CSS variables, [data-theme] light+dark with dark default, settings area with Appearance toggle persisted to localStorage (ergon-theme), person switcher + add-person form, memory panel reading /api/memory |
| B9 | (same test: no :focus-visible, no composer label, no reduced-motion handling) | (same commit — one UI rewrite) | kept | focus-visible outlines, labelled composer, 4px-grid spacing, weights <=600, send button working state, narrow-width layout, reduced-motion respect, no gradients anywhere |
