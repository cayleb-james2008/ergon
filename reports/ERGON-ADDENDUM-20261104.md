# ERGON — addendum: the checks run after the first filing (4 Nov, small hours)

From: ERGON stream lead (`opencode-go/deepseek-v4.1-flash`), 4 Nov 2026 ~05:30Z (00:30 ET). Addendum to
`reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md` and `reports/SIMBANK-V3-VERDICT-20261104.md`. Nothing was deployed,
pushed or merged; no hosted write; no run duplicated.

## 1. The board branch re-run under the load that broke the earlier e2e

The checker's only code-free failure was load-sensitive (long-task and 200-job timings, plus a dialog race). Re-run by the lead
at 00:03–00:05 ET with the host at load **4.25–5.8** (175 chromium processes and day-old `vite preview` orphans from dead
sessions were present — the same environment):

| suite | result |
|---|---|
| unit (`npm test`) | **119 / 119** |
| e2e (`npm run test:e2e`) | **108 tests, 103 pass, 0 fail, 5 skipped, exit 0** (80.7 s) |
| connection (`connection.test.mjs`) | **10 / 10** (00:12 ET, separate run) |
| multi-width lint (fresh `VITE_BACKEND=demo` build) | **PASS** 7 widths x 2 themes |

4x-CPU long tasks: `[]` at 12 jobs, **50 ms** at 60 (bound 100 ms), **150 ms** at 200 jobs (test reference 200 ms). Logs:
`~/fleet-scratch/overnight/evidence/board-e2e-lowload2-20261104.log`,
`~/fleet-scratch/overnight/evidence/board-suites-lowload-20261104.log`. The build dir and preview server were removed after the
lint run.

## 2. Showcase isolation check, staged and proven against a real deployment

`~/fleet-scratch/overnight/stage/check-showcase-isolation.py` runs the six-point check from `docs/SHOWCASE-PREP.md`: it fetches
the deployment's page and chunks, lists the Supabase hosts, fails on the Ergon ref (`yhpjpjeedpuvcgnvfpka`) or
`service_role`/`sb_secret` anywhere served, and with the showcase key does count-only `HEAD` requests proving the Ergon host
refuses that key. It reads no row contents.

**Negative control, run on the live v3 (which legitimately points at the Ergon project):** check 2 **FAILS** and names the Ergon
ref; check 5 passes; 3/4/6 SKIP without a key. The tool detects the exact leak it exists to detect, on a real deployment, before
it is ever pointed at a showcase. Kept out of the repo on purpose: the isolation check must be a fresh checker's act.

## 3. Two SimBank future paths, written down and not applied

The checker's sentence "the gate recomputes to PASS without re-running the model" is **not operable** as the instrument stands.

- `aggregate3` takes each packet's final grade from the R verdict file and has **no owner-override input**. A ruling that ML-D2 is
  `caught` therefore needs either an edited verdict file (refused: it destroys provenance) or a hash-bound override input.
  `~/fleet-scratch/overnight/SIMBANK-V3-OWNER-OVERRIDE-PROPOSAL.md` is that change (~12 lines, override listed above the pass
  line, verdicts untouched, tests named, `verdicts3` hash still recorded). **Nothing applied.**
- A post-9-Nov Sonnet/Opus re-grade: **verdict files carry no judge identity or model at all** (`judge3.py` writes only
  `packet/verdict/quote/why`), and the judge model is a module constant, not a flag. A re-grade must therefore use a separate
  verdict dir and a judge model recorded per verdict. `~/fleet-scratch/overnight/SIMBANK-V3-REGRADE-LATER.md`. This is the one
  place tonight's record is weaker than it reads; the report states the same-model limitation, but no artefact binds it.

## 4. Keep-alive watcher: the armed command was wrong

The watcher armed for 04:17 ET used `vercel logs <host> --since 30m --json`, which this CLI (64.5.0) rejects:
`invalid_arguments: Select a deployment with --deployment`. Its evidence file would have contained a CLI error. Tested forms:
`--project` without `--scope` → `not_linked`; `--scope caylebjamesportfolio --project rhl-warehouse-board --since 30m --json`
→ valid JSON summary (`recordsReturned: 0`, window printed). Runtime logs here cover function invocations only, so a quiet
window is expected before the cron and "no record" will not be reported as "the cron did not run". Replaced with a corrected
poller (04:17–07:30 ET, every 3 min, matching `keepalive|vercel-cron`, then a direct function GET) and re-armed; the direct GET
returns **200 `{"ok":true,"status":200}`** now. Result: `~/fleet-scratch/overnight/keepalive-first-run.txt`.

## 5. State of the fences

No production deploy, no push, no merge, no hosted Supabase write, no row contents read (counts/metadata only), no destructive
SQL, no SimBank run duplicated (four response dirs, 60 files each, nothing running). Every session pinned
`opencode-go/deepseek-v4.1-flash`; judging and both independent checks were separate sub-sessions that made no run and edited no
source.

## Errata (01:10 ET, after measuring)

§4's sentence "Runtime logs here cover function invocations only, so a quiet window is expected before the cron" is **wrong**.
Measured against a known invocation: this project's runtime logs show nothing at all on the current plan — a `/api/keepalive`
call that provably ran (`x-vercel-cache: MISS`, 200) left no record in any 5/10/30-minute window, `--follow` no longer streams,
Hobby retains 30 minutes (so the poller's `--since 60m` was a rejected query), and the events API holds build output only. The
poller described in §4 was therefore replaced by v4, which proves the channel blind with a known event before reading silence,
and reports three ways. See `reports/ERGON-KEEPALIVE-CHANNEL-CORRECTION-20261104.md`.

— ERGON stream lead
