# Bar — Ergon scale-up (ultra loop, project run)

Kind: code + interface (a local web app: a Rust binary serving an embedded chat UI).
Delivered = commits on branch `loop/work` in this repo. Nothing is pushed or published by this run.
Facts never to invent: live-model behaviour (the OpenCode Go key is NOT available in this
environment; the live path is real code proven against a stub, and must be stated as such),
customer deployments, user counts, company names.

Owner brief (the goal, approved — do not drift it): ERGON is an AI agent for every employee,
personal to each person, learns how they work, and FINISHES THEIR DIGITAL WORK AUTONOMOUSLY,
whatever it may be digitally. No approval ceremony in the core loop.

Conditions:

- C1 (cmd): `bash .loop/verify <label>` passes: cargo build clean and the full test suite green.
- C2 (cmd): Threads and task history persist across a server restart: a test creates a thread,
  runs a stub-model task, restarts the app on the same data dir, and the thread with its
  messages is still listed and readable.
- C3 (cmd): Multiple people: at least two people, each with their own memory file, workspace
  and thread list; a test proves isolation — one person's learned note never appears in the
  other person's memory, context or threads.
- C4 (cmd): Broader real toolset, each proven by a stub-model end-to-end run or a direct tool
  test: (a) build a spreadsheet file (CSV) from structured rows via a Rune skill;
  (b) search workspace files by text; (c) move/rename a workspace file;
  (d) compile a research notes document from multiple fetched pages via a Rune skill.
- C5 (cmd): Learning visibly shapes behaviour: a preference learned in one task appears in the
  standing-preferences section of a later task's system prompt (asserted on the recorded stub
  request), and the person's memory is viewable in the UI.
- C6 (cmd): No-key honesty (baseline behaviour, must not regress): with no key, chat refuses
  with the honest error and nothing is fabricated.
- C7 (cmd): Sandbox: file tools cannot escape the active person's workspace — `..`, absolute
  paths and symlinks pointing outside are refused, through the direct tools and the Rune host
  functions alike.
- C8 (cmd): Shell and themes: the served page carries the sidebar, thread list, composer and a
  settings area with an appearance toggle; both light and dark theme definitions are present
  with dark as the default.
- C9 (judge): The rendered UI (both themes) reads as a familiar, quiet chat product — sidebar,
  thread, composer — in Ergon's own words, with no AI-slop signs (no hue gradients, glows,
  gradient text, emoji icons, shouty copy). Example that should pass: this app's shell with its
  own tagline "Describe the work — it finishes it". Example that should fail: the same layout
  with a purple-blue gradient hero, a sparkle icon and the headline "Supercharge your
  workflow" — it would fit any product.
- C10 (owner): The owner grades the rendered UI's look. The loop never marks this passed.
- C11 (cmd): Settings surface works: the appearance toggle persists the choice (localStorage),
  and the memory view shows the active person's actual memory file content.
- C12 (cmd): Replacement staging exists at ~/workspace/restart-20261117/replacement/ergon/:
  README.md, the résumé-site project page (HTML, in the current site's style) and CLAIMS.md,
  with every factual claim tied to a proof in this repo.

## DIRECTION v1 (interface)

- Product: Ergon — a personal agent for one employee that finishes their digital work
  autonomously; the surface is the chat they talk to it in.
- Character: plain-spoken, steady, workmanlike.
- World: a well-kept desk at the start of a workday — paper, ink, one warm accent (the
  incumbent terracotta #d97757 stays).
- Moments: the finished-work summary in the thread is the one place that gets weight and the
  accent; everything else stays quiet.
- Signature: work steps are shown as a plain ledger of tool calls ("Tool: list_dir") in the
  thread — the receipt of work actually done.
