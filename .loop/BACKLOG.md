# Backlog — Ergon scale-up

| id | what is wrong | how to reproduce | severity | status |
|---|---|---|---|---|
| B1 | Threads live in server memory only; a restart loses all task history (breaks C2) | start app, create thread, restart app on same data dir, thread list is empty | high | open |
| B2 | One global person only: single memory.md, workspace and thread list shared by everyone (breaks C3) | inspect src/lib.rs: Config has one memory_path/workspace_dir; no people anywhere | high | open |
| B3 | No spreadsheet tool: the agent cannot produce a spreadsheet file (breaks C4a) | execute_tool has no spreadsheet arm; tool list in system_prompt | medium | open |
| B4 | No file search tool (breaks C4b) | execute_tool has no search arm | medium | open |
| B5 | No move/rename tool (breaks C4c) | execute_tool has no move arm | medium | open |
| B6 | No research skill: cannot compile a notes document from several fetched pages (breaks C4d) | skills.rs has only write_document and update_memory | medium | open |
| B7 | Learning does not visibly shape behaviour: memory is dumped wholesale into the prompt, there is no standing-preferences surfacing and no memory view in the UI (breaks C5, C11) | read system_prompt in lib.rs and ui.rs (no memory view) | medium | open |
| B8 | No light/dark themes and no settings: the UI is light-only with hard-coded colours (breaks C8, C11) | read ui.rs: single palette, no data-theme, no settings element | medium | open |
| B9 | UI craft gaps (impeccable floor): no :focus-visible styling, composer input has no label, several off-grid spacing values, send button state only partially handled, no reduced-motion respect needed but no loading text swap | read ui.rs | low | open |
| B10 | Sandbox is lexical only: a symlink planted inside the workspace pointing outside is followed by read_file/write_file and the Rune host functions (breaks C7) | plant symlink ws/escape -> /tmp/outside, read_file "escape/secret.txt" succeeds | medium | kept |
| B11 | Replacement staging does not exist (breaks C12) | ls ~/workspace/restart-20261117/replacement/ergon/ fails | medium | open |

## Leads

- run_agent discards the tool ok flag (`let _ = ok;`) — check whether any behaviour depends on it.
