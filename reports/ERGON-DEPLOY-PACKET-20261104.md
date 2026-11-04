# ERGON deploy packet — warehouse board remainders (one page, the owner's yes only)

Prepared 4 Nov 2026 01:25 ET, **revised 02:25 ET** after the branch moved. Nothing has been deployed: live production is `455a9da`.

## Which commit you are actually deploying (the branch moved four times tonight)

```
cd ~/Work/projects/rhl-board-rebuild/board
git fetch . 2>/dev/null; TIP=$(git rev-parse --short fix/board-remainders-20261103); echo "tip=$TIP"
git diff 9cc7d95..$TIP --name-only | grep -v '^docs/' | grep -v '^test/'    # must print NOTHING
```

- `9cc7d95` is the **last commit that changes product code**: the board read gets a 15 s deadline and is single-flight, because the
  health probe asks `board_token` — a different RPC from the board read — so with the data endpoints hung and the probe answering the
  label could sit `Live` over a frozen board (measured: 26 s). Everything after it is **tests and docs only**
  (`26e6c9e` = the hung-path rule rework, `b559e68` = the 60-job measurement doubled + the "about 11 s" docs figure).
- **The invariant above is proven, not asserted:** building `9cc7d95` and `b559e68` with the same `RHL_BUILD_ID` produces a
  **byte-identical `dist` tree** (`19ad0173…`). So the certification below, run against `9cc7d95`, describes the artifact of whatever
  tip the second command shows — and if that command ever prints a file, the certification must be re-run (one command, below).

## What you are saying yes to (all four in one deploy)

1. **Phone sizing** (the round-9 complaint): at 390x844 `--fs` 14 -> **12**, row 56 -> **52** px, card 280 -> **232** px wide, no page
   scroll; the Add sheet, its field and Save stay inside the visible area with the keyboard up. 1366x768 unchanged.
2. **Connection honesty**: no false `Reconnecting` on a healthy load (live v3 claims it 95-140 ms into **every** load); no `Live`
   before a board has been read; `Error` for a hung or failed read (within ~2 s on a fast link, up to the measured ~11 s on a link
   that was already slow); a slow-but-working link (2 s, 3 s per answer) ends `Live` instead of flickering; a board that never loaded
   shows no word at all.
3. **Legend**: initials only (v3 still carries a `title="JA"` tooltip; no name exists either way).
4. **A suite that certifies the above and is deterministic under load**: it was 3 failures in 100; it is now 0 fail from load 3.9 to
   load 38, with the flakiness attacked at its causes instead of its assertions, and a real injected regression still fails.

Same 19 lines, same cards, same editing, same keys, same `/api/keepalive`.

## The deploy (two commands) and the one trap

```
cd ~/Work/projects/rhl-board-rebuild/board
TIP=$(git rev-parse --short fix/board-remainders-20261103)
vercel deploy --prod --build-env RHL_BUILD_ID=$TIP
```

The trap: a CLI deploy has no git, so **without `RHL_BUILD_ID`** the version tooltip reads `local` and nobody can prove which build a
device is running. Rollback is `vercel rollback` (about 4 s; the alias follows in 6-9 s). `docs/OPERATIONS.md` has the full procedure.

## Certification after the deploy (one command, read-only)

```
node ~/fleet-scratch/overnight/stage/postdeploy-check.mjs \
  --url https://rhl-warehouse-board.vercel.app --expect-version $(git -C ~/Work/projects/rhl-board-rebuild/board rev-parse --short fix/board-remainders-20261103)
```

Seven checks, each with its number: `/api/keepalive` 200 (and Supabase answered the count-only HEAD), label blank-then-Live and never
`Reconnecting`, `data-version` = the tip, no console errors, phone sizing at 390x844, legend initials only.

**The script is trusted because it has been run both ways.** Against live v3 it **fails exactly the two defects being fixed**
(`Reconnecting@102-125 ms`; `fs=14 / row=56 / card=280`); against a clean export it **passes 7/7**
(`blank@90 ms -> Live@139 ms`; `fs=12 / row=52 / card=232x44 / pageOverflow=0`). Evidence:
`evidence/postdeploy-check-both-halves.txt`, `evidence/deploy-cert-9cc7d95.txt`.

If anything fails: `vercel rollback` first, read the number afterwards.

## What is proved about the artifact itself

- **From zero, as Vercel does it**: `git archive 9cc7d95` -> `npm ci` (exit 0) -> build (exit 0) -> **unit 121/121**.
- Full e2e from a clean export of **this tip**: **108 tests / 103 pass / 0 fail / 5 skipped, exit 0**, 94.8 s at load 5.5
  (`evidence/board-e2e-b559e68-20261104.log`; the 5 are the live-Supabase cases, below).
- Agent suite: **22 pass / 0 fail / 12 skipped** (live-gated).
- Gate 1 multi-width lint: **PASS**, 7 widths x 2 themes (no overflow, contrast >= 4.5/3, visible focus at every stop).
- Production bundle: `__rhlDemo`, demo keys, `service_role`, `sb_secret` -> **0 files**; the tip sha is baked in so the tooltip is
  checkable after the deploy.
- **The hosted backend already answers the branch's probe**: `POST /rest/v1/rpc/board_token` on the real project returned `200`
  `{"e":12,...,"l":19,...,"r":12}` with the same anon key the live site serves every visitor — so the deployed build uses the probe
  path on day one, not the documented fallback (`evidence/deploy-readiness-hosted-backend.txt`).
- Independent checks, all on the board: `reports/BOARD-REMAINDERS-CHECK-20261104.md`,
  `reports/20261104-board-flakefix-check.md`, `BOARD-FINAL-TIP-CHECK.md` (found the 3 s defect), `BOARD-FINAL-TIP-CHECK-2.md`,
  `BOARD-FINAL-CONNECTION-CHECK.md`, `BOARD-DATA-READ-CHECK-3.md`.

## The price, stated plainly (this is what the yes accepts)

- A link that is **slow once and then dies** is labelled `Live` for **~11 s** (v3: 2.5 s), up to ~12 s with the tick. The docs now say
  ~11 s; anything shorter would be the docs lying about their own worst case.
- **~72 requests/min** while the tab is visible (~35 KB/min, ~4.5 GB/month at the 132,000 viewer-minute model, near the free egress
  allowance). Unchanged by this branch; the lever is the 1 s probe interval (2 s halves it, worst case Error ~3.5 s).
- Under heavy host load the suite's "update cost" pair can still fail on a single 102-112 ms spike against its 100 ms bound; the tip
  now measures that board twice and asserts the cheaper run, and a 60-job-only regression at 146 ms still fails.

## What is still NOT proved (so the yes is informed)

- **Two of the five live-Supabase e2e cases have never run**: realtime delivery between two devices, and the polling fallback with the
  websocket blocked. The local stack would not start tonight (`sh: can't fork` inside `supabase start`'s db container — eleven
  controlled experiments in `evidence/supabase-start-blocker-diagnosis.txt` reproduced every container-level variable clean and still
  could not make the CLI's own start work). The other three live cases were proven directly against the shipping migration instead
  (`evidence/ergon-sqlproof-run.txt`), plus a second REST-level contract suite (30/32, 2 harness-only failures).
- **The owner's own working key has not been minted against this build** — his hands only.
- That the hosted `save_entry` treats `expected_revision` exactly as the migration does. Revisions are visibly live on the real project
  (`r: 12`) and v3 already runs the same flow, but the only honest proof is a real edit on the real board.

## If the yes is "not yet"

Nothing here goes stale quickly: the branch is self-contained, the invariant command above says whether the artifact still matches the
certified commit, and the only thing that needs re-running after further changes is the certification with the new `--expect-version`.
