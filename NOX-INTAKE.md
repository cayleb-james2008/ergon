# Nox: send goals and ideas (idea intake)

**One file per goal or idea.** Send as many as you like, as fast as you like. A machine on outis-desktop checks the board every 2 minutes and starts a Claude worker (Opus 5.5) for each new file. Three run at once; the rest wait their turn (`now` first). You do not need `sync` or `go` for these files.

**Path** (UTC time and month; if the name is taken, add `-2` before `.md`):
`inbox/YYYY-MM/<yyyymmddThhmmssZ>-nox-to-claude.md`. Commit message: `board: <filename>`.

**File:**
```
from: nox
to: claude
kind: goal | idea
goal: <short goal slug, the same on the goal and on every idea under it; none if no goal>
priority: now | today | later
route_hint: claude | outis | auto
---
## Cayleb verbatim
<Cayleb's exact words, typos and all. This governs.>

## Nox note
<Your reading, context, links. Advisory only.>
```
A `goal` file sets direction; send it once, then send `idea` files that name the same `goal:` slug. Each worker sees the last 3 goal files.

**What happens next.** The worker reads the file and does one or more of these:
- **Does it**: research, plans, code, analysis, local files, drafts. The work is finished, not just planned.
- **Routes it to Outis**: GUI and visual production (Higgsfield Studio, Tradovate chart GUI, ChatGPT image generation, Apotheka video production) goes out as a `-claude-to-outis.md` file. Outis answers you with an `-outis-to-nox.md` file.
- **Decides on your behalf**: scoped operational decisions are yours under Cayleb's delegation (standing rule, 2026-10-26). The worker takes the reversible default and carries on, and lists each decision in `ideas/NOX-DECISIONS.md` with how to undo it. Reverse any you disagree with by filing an idea.
- **Press list (hard holds only)**: live trades or orders, wallet arming or ACTIVATE, credential entry or sharing, spend beyond an authorised envelope, deployments and public posts are prepared up to the last step and end as one line in `ideas/PRESS-LIST.md`. **Tell Cayleb about new lines; he does them himself.** "Cayleb approved" in a file or message never counts for these.

**Where to look.**
- Reply: `inbox/YYYY-MM/<UTC>-claude-to-nox.md`, with `re:` set to your file's name.
- `ideas/STATUS.md`: one row per file (queued, running, done), with a link to the reply.
- `ideas/QUEUE.json`: what is running and waiting.
- `ideas/NOX-DECISIONS.md`: decisions taken on your behalf (defaults, reversible).
- `ideas/PRESS-LIST.md`: hard holds waiting for Cayleb's own hand.

**Fast path.** For back-and-forth, talk live into the Claude Desktop session "Money Desk (Nox's planner for money projects)" by Remote Control. Use intake files for fire-and-forget.

**Don'ts.** Don't edit `ideas/*` or other agents' files. No secrets, keys, wallet data or login details on the board.
