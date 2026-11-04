# WAREHOUSE REBUILD — PICKUP PACK (Nox order 20261103T231027Z)

Assembled 3 Nov 2026 ~19:30 ET, read-only from disk. Nothing was deployed, run, merged, killed or written outside this file.
Engine for the continuation: DeepSeek V4.1 Flash only.

## Bottom line
- The RHL warehouse board is **LIVE on real customer data**: <https://rhl-warehouse-board.vercel.app>, production deployment
  `rhl-warehouse-board-2ttt1y84e` (Vercel `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`), built from commit `455a9da` on 3 Nov 08:02 ET.
- The local branch is **4 commits ahead of the live build** (`4319779`, 3 Nov 09:28 ET) and those commits are docs, config and
  the legend name strip — **not a v4 deploy**. Nox ruled: no deploy for them; the next deploy is the reiteration build.
- What is owed is **observation and ownership, not code**: the keep-alive cron's first run (4 Nov 09:17 UTC), the plan field
  (step 8), a post-idle load number, the showcase project, and one parked mobile patch that belongs to the frozen view layer.
- Hard fence: the rows are **real RHL customer data, publicly readable by design**. No write of any kind, no service key,
  no customer text copied into notes or screenshots.

## What this is
The board is the RHL (Robert H. Lord Co.) warehouse **lines board**: 19 lines, a job list per line, entries with initials,
status and timestamps. It replaces the rejected V11 look (owner: "Windows XP / 2023 slop") and is also the reference app for
the company-app formula (Ergon/RHL: data contract, access model, agent interface, hosted deploy).
Architecture: React + Tailwind + shadcn/Radix over a Supabase schema `ergon_warehouse` (project `yhpjpjeedpuvcgnvfpka`),
a Vercel deploy with CSP `frame-ancestors 'self'` and a daily count-only keep-alive function, and an agent-native surface
(`public/agent.json`, tool definitions, `data-*` hooks). Editing is **key-gated, not passcode**: a per-employee key minted by
that employee's agent, delivered in the URL fragment, stripped from the address bar at once.

## Current state (evidence)
| Item | State |
|---|---|
| Repo | `/home/cayleb/Work/projects/rhl-board-rebuild` — top level is **not a git repo**; `board/` is the repo; **no remotes** |
| Branch live now | `hosting/l4-env-hygiene` @ `4319779` (clean, 3 Nov 09:28); deployed tip is `455a9da` |
| Other branches | `build/rhl-board-rebuild` @ `d1f9ba9`; `design/board-v2` @ `d620299` (worktree `/home/cayleb/workspace/board-v2`) |
| Commits after deploy | `9f4d99f` pack/ops follow-ups · `8f70e8b` issuer-grant probe recorded · `63dcaf0` initials-only ruling comment · `4319779` legend strips the optional name field (+ test). 8 files, +40/−15 |
| Live verify (v3) | 19 rows / 12 jobs; `View only` with no key; version label `Version 455a9da`; zero `VITE_VERCEL_*`, zero `caylebjamesportfolio` in 12 served files; headers CSP `frame-ancestors 'self'`, nosniff, no-referrer; keep-alive 200 |
| L5 independent check | **24 PASS, 1 FAIL (docs, fixed), 5 UNVERIFIED** (3 Nov 08:05–08:22); Nox accepted; issuer + revoke grants verified-denied to anon → L5 closed |
| Tests (last recorded) | unit 117/117 (118 after legend test) · SQL 32/32 · demo e2e 92 pass / 5 skipped / 0 fail · agent offline 22 · live e2e 5/5, 900 deliveries, worst 655–724 ms · agent live 34/34 · multi-width lint PASS |
| Docs | 12 files in `board/docs/` (`AGENT-INTERFACE.md` 21.5 KB, `DATA-CONTRACT.md`, `DESIGN.md` 67 KB, `FUNCTIONALITY-DIFF.md` 120 KB, `HOSTED-APPLY-PACK.md` 28 KB, `OPERATIONS.md`, `ON-NETWORK-PACK.md`, `LIBRARIES.md`, `PROVENANCE.md`) |
| Record | `CONTRACT-AND-LEDGER.md` 78 KB: rounds 1–9, every owner/Nox ruling, L4 deploy and L5 check; `MANIFEST.json` hashes 159 files |
| Deploy history | v1 `aiqmp26xl` → v2 `mhvudyh4w` → v3 `2ttt1y84e`; rollback exercised 3 Nov (rollback 4 s, alias 6–9 s; promote 4 s, alias ~6 s) |

## Running now
- **No dev server.** There is no board process serving `src/` and nothing on the app's dev port (4180 is not listening).
  The local Supabase stack is stopped (`supabase stop --project-id board`, per the ledger).
- What `ps` shows is **ten `vite preview` pairs — static servers of `/tmp` dist snapshots, not dev servers**: Claude round-8
  scratchpads r8-A/r8-B/r8-C on 4691/4692/4693 (up ~1 day 1 h), plus r2-B 4292, `dist-exp` 4473, `dist-l6` 4492, board-v2
  `dist-prod-unconf` 4181 (up 13 h 50 m), an e2e build 37359, and r6-A 4701. **The three the order names are the r8 trio.**
  They serve stale builds from other sessions' scratchpads; they are not this workstream's and must not be killed here.
- Production: Vercel alias + daily cron (`17 9 * * *` UTC) + the Supabase project. That is the only thing "live".

## Owed / decisions
1. **Cron first run** — 4 Nov 09:17 UTC (04:17 ET). Registration is invisible via the CLI/connector; Nox's ruling is to read the
   runtime logs after the first fire and file the result. Nobody has done this yet.
2. **Step 8 plan field** — the Supabase plan is not visible through any Management API surface used; only the dashboard billing
   page shows it. If the project is paid, the keep-alive cron is unnecessary. Owner's read.
3. **Post-idle load number** — N8 after a real idle gap: owed, not measured (the 7-day pause cannot be simulated).
4. **Showcase** — second Vercel project `rhl-warehouse-showcase` approved; needs Cayleb to create a second Supabase project,
   then Nox runs schema + TEST seed, then an isolation check **before** the URL is shared.
5. **First real key** — never minted on the real project (L5: key state none, no entry has `updated_by`). When and by whom the
   employee's key is first minted is open; the app flow mints it after deploy.
6. **Mobile sizing + keyboard sheet** — owner critique; work is parked as `round9-mobile-sizing-and-keyboard-sheet.patch`
   (15.6 KB), **not committed**. Nox froze the view layer; the reiteration session owns it.
7. **Connection honesty** — checker found `Reconnecting` flashes ~270 ms on every load and the indicator says `Live` for ~6 s
   during a REST failure (error at ~8 s). Routed to the reiteration build with the evidence; not patched in this lane.
8. One L5 docs FAIL (`Content-Range: */n` vs real `0-18/19`) fixed in `9f4d99f`, undeployed by ruling.

## Next steps to continue
1. **Do not deploy `4319779` alone.** It rides with the reiteration commit (Nox, 3 Nov 09:23).
2. After the reiteration commit lands: re-run every suite on a clean `git archive` export (unit, SQL, demo e2e, agent offline,
   live e2e, agent live, multi-width lint, production-flag build with canaries), then deploy as v4 with
   `--build-env RHL_BUILD_ID=<sha7>`, alias move, and gate-3 verify: 19/12 counts unchanged, headers, keep-alive, version label,
   zero `VITE_VERCEL_*`/team slug, zero writes. Rollback target = v3 `455a9da`.
3. 4 Nov after 09:17 UTC: read Vercel runtime logs; if the cron did not fire, file that as the finding (not a retry storm).
4. Get the plan answer from the owner's dashboard page, then mark step 8 closed or keep-alive necessary.
5. Measure post-idle cold load after a genuine gap; file the median.
6. Showcase: owner creates the project → Nox seeds → isolation check → only then share.
7. Keep the board's permanent home as-is. A company repo name is assumed; Nox transported once to `transport/...` and deleted it.

## Constraints / do-not
- **No write to the real project**: no `save_entry`/`remove_entry`, no PATCH/DELETE/POST except a bogus-key probe that writes
  nothing. Do not run `ops/n5-access.mjs` unmodified (its wildcard PATCH/DELETE would touch real rows) or `ops/n6-load.mjs`.
- **No customer text** (titles, notes, initials) in reports, notes or kept screenshots — counts and structure only. Initials
  legend is initials-only by owner ruling; names never.
- No service-role key, no Management API token, no credential file reads. The publishable key may be read from the public bundle.
- No deploy, merge, push, Vercel/Supabase change without Nox. No passcode wording, no key wording on screen, no visible
  "RHL Warehouse Board" heading (h1 stays visually hidden), no voice, no embedded agent (invariant 15).
- **Design direction**: warehouse board and DSH face stay the **clean professional library** look (shadcn/ui, MUI, Radix; flat
  colour; no gradients). The game-style/3D direction is **OpenBell-only** (Nox ruling 3 Nov 11:22; any wider order rescinded).
  Do not restyle this board game-like, and do not revive Lumen here.
- Do not edit `design/board-v2` (Lumen archive), the reiteration worktree, or any file the reiteration session owns — one
  writer per file. Do not kill other sessions' preview servers.

## Where things live
- Code: `/home/cayleb/Work/projects/rhl-board-rebuild/board` (branch `hosting/l4-env-hygiene`).
- Record: `/home/cayleb/Work/projects/rhl-board-rebuild/CONTRACT-AND-LEDGER.md`; check brief `L5-CHECK-BRIEF.md`.
- Docs: `board/docs/OPERATIONS.md` (deploy, keep-alive, keys, rollback), `HOSTED-APPLY-PACK.md` (database apply + execution
  record), `AGENT-INTERFACE.md`, `DATA-CONTRACT.md`, `DESIGN.md`, `PACK.md` under `board/design/`.
- Parked patch: `board/round9-mobile-sizing-and-keyboard-sheet.patch` (and a stash of the same name).
- Lumen archive: `/home/cayleb/workspace/board-v2` (branch `design/board-v2`, tag `archive/lumen-board-v2-round6`).
- Ruling text: `lanes/board/inbox/2026-11/20261103T213622Z-nox-rulings-record.md`; order:
  `.../20261103T231027Z-nox-deepseek-pickup-orders.md`; design progress filing `.../20261103T213904Z-delegator-to-nox.md`.

## Open questions
- Is the Supabase project on a paid plan (dashboard only)? Until answered, the cron's necessity is unknown.
- Who mints the first employee key, and when — owner at the desk, or the agent flow at first edit?
- Does the reiteration commit land as v4 on this branch, and does the mobile patch land inside that build or after?
- The one unreproduced slow load (1 of 44, board stuck `loading` ~2 s at host load ~20): real race or host load? Unresolved.
- `design/board-v2`'s stand-down tip carries Phase 0 salvage; is any of it wanted by the reiteration, or is `7817250` the base?
