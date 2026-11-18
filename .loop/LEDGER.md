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

- Review round 1 (thermonuclear + interface lens, fresh context, frozen copy): 16 thermonuclear findings + 13 lens findings, all reproduced. Fixed and regression-tested (25/25): Rune moved-value tool errors, About-section learning exclusion, dangling-symlink escape, Content-Length DoS, move overwrite, search partial-coverage disclosure, global-person pointer (session cookies), no-key message loss, thread history to the model, unverified done claims flagged, corrupt-store backup, id validation, memory-structure injection, keyboard/aria-live/busy-guard/contrast/empty-state UI batch. Verify fix-round-1 green 25/25.
- Dispositions: C12 staging = B11 (still to do, not a code defect). README staleness = fixed by the B11 replacement README. status.person now consumed by the UI (Working-for label).

- Checks-finder report (ran on the pre-fix frozen copy): dangling-symlink + move-overwrite findings duplicated round 1 (already fixed); NEW: 'error:'-titled document misreported — fixed via 'file:'-prefixed skill returns + regression test; theme-persistence and preference-parser test pins added. Verify fix-round-2 green 28/28.

## Review round 2 (thermonuclear, fresh context) — verdict: NOT DONE
- F1 CRITICAL: symlink resolver escaped via a link whose target contains another symlink (lexical substitution). FIXED: resolver rewritten as a component queue — every resolution re-enters the walk; containment checked per step; 16-hop cap kept. Pinned by symlink_target_containing_a_symlink_is_rewalked (read+write) + legit-link direction.
- F2 HIGH: session ids predictable (ms+counter), client-chosen ids pinned (fixation), browser switch still moved the global pointer. FIXED: ids are 16 random bytes hex; unknown/forged ids are never adopted (server issues its own, incl. stale-cookie re-issue on GET /); a known-session switch moves only that session, cookie-less API clients keep global semantics. Pinned by forged_cookies_are_replaced_and_sessions_do_not_leak_globally.
- F3 HIGH: done-gate failed both directions (verb list flagged inline answers + update_memory/move work; missed synonym lies). FIXED: gate now checks files NAMED in the summary against the workspace + artefact claims against the work ledger (writes, memory, moves). Pinned both directions (done_claim_without_writes…, honest_done_summaries_are_not_flagged).
- F4 MED: over-limit request silently dropped. FIXED: real 413 with the limit stated; 30s read timeout per connection. Pinned.
- F5 MED: repeated corruption overwrote .bak; semantic corruption (valid JSON, wrong shape) got no backup; people.json had none. FIXED: first-free-suffix backups (.bak/.bak2/…), shape validation on both stores, people.json included. Pinned.
- F6 MED: search cap note false at exactly 50; hidden dir counted as 1 skip; single-file path silently empty. FIXED: cap flag set only by a real 51st match; skip note reworded to entries (files or folders); single-file paths searched directly. Pinned.
- F7 LOW: identity exclusion exact-case only. FIXED: case/space-tolerant identity check. Pinned.
- F8 LOW: move_file moved a symlink's target. FIXED: source resolved with resolve_final=false — the link itself moves. Pinned.
- F10: parser mutation gap — already closed by the direct parser pin added in the checks-finder round (standing_preferences_excludes_only_identity_lines) plus the casing pin.
- Credits held: history_json hygiene, no-key flow, legit symlinks, restart persistence.
- Verify fix-round-3b: 36/36 green.

## Review round 3 (thermonuclear, fresh context) — F1/F2 hold; F3 gate rebuilt
- F1 (symlink resolver) and F2 (sessions): FULL battery passed, unbroken — mixed chains, sibling-person escape, 16/17-hop boundary, 75 concurrent session cycles, chat honours the session's person. C7 PASS.
- F3 HIGH (third round on this gate): verb/noun heuristics still failed both directions. REBUILT: the gate now tracks WHICH files the run actually wrote (write_file/move_file args; skill results' named files). A named file that does not exist = caution; a named file that exists but this run did not write = a factual ledger line ("this run wrote X; the summary also names Y, which this run did not write") — the pre-existing-file lie is exposed without accusing honest input references. named_files tokenisation hardened (every non-filename char separates — colon/exclamation evasions closed; URLs excluded). Pinned: done_gate_checks_named_files_against_the_write_ledger + the two earlier gate tests still pass.
- F5 MED residual: people.json shapes [], {}, 42 and all-invalid-ids loaded with NO backup (missing key treated as fine). FIXED: valid = object with people array; all-invalid entries also back up. Pinned for all four shapes.
- F8 MED/LOW: dangling link could not be moved (exists() follows links); moving onto a dangling link landed at its target while reporting the link's name. FIXED: both sides resolve_final=false + symlink_metadata existence; onto-a-link refuses as an existing destination. Pinned.
- F4 LOW residuals: 413 could be reset away when the body was in flight — now the server drains (bounded, in-deadline) before answering; per-read timeout replaced by a TOTAL 30s request deadline (slow-loris drips die at the wall). Pinned (413 with body in flight).
- F6 LOW: single-file search of an unreadable file said a bare "(no matches)". FIXED: counted as skipped and disclosed. Pinned.
- F7 LOW: fullwidth colon smuggled identity lines. FIXED: normalised in the identity check. Pinned. (A learned bullet literally starting "Name:" remains indistinguishable from identity — accepted; the full memory file is in the prompt regardless.)
- Sessions map bounded (cleared past 10k entries; sessions re-issue transparently).
- Verify fix-round-4: 42/42 green.

## Review round 4 (confirmation, fresh context) — residuals all pass; gate filename model rebuilt
- All round-3 residual fixes PASS under probe (people.json backups in 6 shapes, dangling-link move symmetry, 413 received at 6 MB in flight, total-deadline both directions, binary single-file search disclosure, fullwidth colon).
- F1 HIGH: named_files split on spaces — every spaced filename (the product's DEFAULT: "Research - <topic>.md", human-titled documents) was truncated to its last word and falsely cautioned even when the run wrote it. REBUILT: mention_candidates expands leftwards from each extension occurrence over filename characters incl. spaces; candidates resolve longest-first against the run's written files, then the workspace, with basename fallback. Skill-written files are extracted from result text by the same matcher against the workspace (extract_written).
- F2 MED: update_memory-only runs naming memory.md were cautioned (the v3 ledger credited no memory work). FIXED: update_memory counts in the write ledger as memory.md.
- F3 LOW/MED: the canonical move summary ("Renamed a.txt to b.txt") cautioned on the source. FIXED: move sources are recorded as consumed — naming them is explained, silent.
- F4 LOW: subpath writes named by basename cautioned. FIXED by the basename fallback in check_mentions.
- F5 LOW: caution suppressed the ledger line (else-if). FIXED: both lines emit when both apply. Pinned.
- Verify fix-round-5b: 44/44 green. Gate battery now covers spaced titles end-to-end (document, research, memory, move) plus direct matcher pins.

## Sealed exam (independent examiner + runner) — 19/19 PASS
- Exam at ~/workspace/restart-20261117/exams/ergon/ (exam.sh, checks.py, stub_model.py; 19 checks E1–E19: shell, themes, no-key honesty, people isolation, forged cookies, stub-driven E2E into a created person's workspace, learning reaching the prompt, persistence, sandbox escapes, done-gate both directions incl. spaced names, 413 with body, corrupt threads.json backup).
- Run 1 (manifest exam.sh 4529c0f5…, checks.py bf8be221…, stub 47c455bd…): 11/19 — E8, E11–E17 all failed at "phase B setup: task 1 chat failed st=500"; non-stub checks passed.
- Root cause (examiner-side debug, wire evidence): EXAM bug, app innocent. The stub's do_POST required the path to start with /chat/completions and 404'd /v1/chat/completions (the app's legitimate POST target) before recording or serving. App's raw request captured and verified correct (auth header, x-opencode-session, UA, OpenAI-style body); with a path-accepting capture server the same flow returned 200.
- Patch (harness wiring only; checks.py byte-identical): stub accepts any path ending /chat/completions; exam.sh extraction tar gained --no-same-owner. New manifest: exam.sh 3558cb2a…, checks.py bf8be221… (unchanged), stub_model.py b9a05004….
- Run 2 (independent runner, hashes verified, run dir ~/workspace/exam-runs/ergon-20261118-015111-15140): PASS E1–E19, EXAM RESULT 19/19, exit 0. App tree unmodified throughout (git status clean).

## Completion programme (18 Nov 2026) — Cayleb's order: full objective functionality
- Gap list: .loop/COMPLETION.md (G1–G12 against the canonical objective).
- Commit 38bc3c3: company library (seeded, agents publish via compile-gated
  publish_skill; cross-person install+run proven), per-person suites
  (role-shaped starters; installed skills are real tools; apps run from
  fields), subagent delegation (suite profiles, shared honesty ledger),
  decisions inbox (delete_file parks; approval deletes for real),
  work log, deterministic offers engine, morning brief, Work Graph /
  Conversation Spine / Today River faces, phone face at /phone,
  ERGON_GATE_TOKEN hosting gate, install.sh (smoke-proven end to end).
- Tests: 56/56 (44 base preserved + 12 new). One real bug found by the
  new suite and fixed: app-run field validation ran after substitution,
  so an empty field silently produced a holed task — now validated first.
- Sealed exam extended E20–E31 (phase D) — run pending at time of writing.
- Checker round 1 (fresh context) dispatched — verdict pending.
| thermo-fix | Thermonuclear findings F1–F13 fixed: symlink delete resolves the link not the target; records + suite stores locked (24-thread regression: 24/24 stored, unique ids); subagent tool restrictions enforced in the loop, not just the prompt; library lock dropped before Rune runs + every skill under a 5M-instruction budget and 128MB limit; one shared 24-call budget across parent + subagents, distinct subagent provider sessions; publish_app kind/author-checked; install-then-call credited by the honesty gate; offers count finished work only, skip app re-runs, two-word pattern keys; library dedupe + ghost-free graph/brief; install.sh absolute paths, quoted ExecStart, single-quoted launcher values, honest service readiness; local-timezone dates via a TZif reader (Today flips at local midnight); add-person full reset; escAttr for attribute interpolation. 65/65 tests, sealed exam 31/31 (run ergon-20261118-121402-51122). | pending | kept | F4's residual: the budget cannot preempt a host function mid-call — fetch's 20 s timeout is the bound there |
| elevation | Hallmark-Rune front-end elevation: both pages migrated to hr-template v1 tokens (paper-warm/terracotta per the Ergon anchor row), R5 caps removed, R4 pill theme buttons replaced, 11px text floor, 44px touch targets at ≤768px, compact small-width sidebar, ledger-railed step rows, memory drawn as grouped facts with the verbatim file behind a disclosure, responsive 2×2 graph layout under 560px. Lint: desktop 1 FAIL = L7 "#face" (ID selectors #face-bar/#face-graph/#face-today — hand-verified false positive, independently confirmed), phone 0 FAIL. Fresh-eyes checker 1: FAIL on two items (--err-ink dangling after the token rename; two R8 middle-dot strings) — both fixed, plus R3 kind labels out of the accent, currentTarget capture in the panel handlers, compact graph. Re-grade checker: PASS on the render. 65/65 tests; exam re-run at final HEAD. | pending | kept | Weakest remaining: the signature ledger still lives inside the chat face — the graph face leads with palette + diagram, not the ledger itself |
