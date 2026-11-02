# Team A — batch 8 (follow-up): PR #14 merged; checker nit fixed in PR #15; standing queue recorded

Lane: Team A / `ergon-lead`. Report to: the Lead. Written 2026-11-02 ~00:05Z. **Spend $0** (no purchases, no model calls,
no generation service, no credential handling, no image reads, no host input). No gate was re-opened by this batch.

## 1. PR #14 (simulation-bank fold-in) — MERGED, checker ACCEPT

- **Merged** as **`8822d479ce694b9c569dd13300379b941f0c320a`** (`Merge pull request #14 from cayleb-james2008/docs/l5-simulation-bank`), `merged_at 2026-11-02T00:00:56Z`; previous `main` `c3d86a5`; merge-commit convention, no force; branch **deleted** on the remote (`git ls-remote --heads origin refs/heads/docs/l5-simulation-bank` → 0 lines). Verified with `gh api repos/cayleb-james2008/ergon/pulls/14` → `state=closed merged=true merge_commit=8822d47…` and `git fetch origin '+refs/heads/main:refs/remotes/origin/main'` → `origin/main = 8822d47…`.
- **Checker report** on the board: `board/reports/checker-ergon-optiona-pr14.md` — quote: **"VERDICT: ACCEPT — every Option-A ruling condition is met and every required gate reproduces green from a fresh clone; one non-blocking documentation nit is named in §6 (EXIT-CODES harness row's stream wording), which does not violate any ruling condition."** The checker's own stream exercise (§5.9) recorded: `harness.mjs` (no command) → exit 2, **usage on stdout (163 B stdout / 0 B stderr)** → stream claim in the row was wrong; `harness.mjs bogus` → exit 2, usage on **stderr** (correct); `harness.mjs verify` without `--out` → exit 2 (correct); `--help` → exit 0 (correct).
- Nothing was needed from this lane on the merge itself. The fold-in's proof runs (DeepSeek acting agent + one Haiku comparison pass) remain gated before any RHL deployment claim — unchanged.

## 2. Checker nit fixed — PR #15 (one phrase, docs-only)

**The fix:** in `docs/EXIT-CODES.md`, the `docs/acceptance/simulation/harness.mjs` row said the usage text goes to stderr
in every case. The no-command case prints it on **stdout**; argument errors print it on **stderr**. One line changed.

- Branch **`docs/l5-exit-codes-harness-usage`** cut from current `main` **`8822d479`**; head **`cb21688a8b850d5f2f84c7f923246e0cd20f2dbd`** (`cb21688`), committed `2026-11-02T00:03Z`.
- **PR #15**: https://github.com/cayleb-james2008/ergon/pull/15 — `state=open`, base `main@8822d479`, head `cb21688`, `mergeable=true`, `changed_files=1`, `additions=1`, `deletions=1`, created `2026-11-02T00:04:06Z`.
- Diff: `docs/EXIT-CODES.md | 2 +-` — one row, nothing else.

**Before → after (the changed row):**

```
- | `docs/acceptance/simulation/harness.mjs` | 0, 1, 2 | 0: a run plan … or `--help`. 1: `verify` found a missing or misnamed BANK SELF-TEST label, … 2: usage (no command, an unknown command or option, or a missing `--out`), usage text on stderr. |
+ | `docs/acceptance/simulation/harness.mjs` | 0, 1, 2 | 0: a run plan … or `--help`. 1: `verify` found a missing or misnamed BANK SELF-TEST label, … 2: usage (no command, an unknown command or option, or a missing `--out`); the no-command case prints the usage text on stdout, argument errors print it on stderr. |
```

**Exit codes unchanged and correct: 0 (plan/self-test written, verify PASS, `--help`), 1 (verify FAIL, unknown company,
missing output dir), 2 (usage).** Empirical streams on this head (split `stdout`/`stderr`):

- `node docs/acceptance/simulation/harness.mjs` → **exit 2**; stdout = four-line usage block (**163 B**); stderr = empty.
- `node docs/acceptance/simulation/harness.mjs bogus --out /tmp/x` → **exit 2**; stdout empty; stderr = `unknown command: bogus` + usage block.
- `node docs/acceptance/simulation/harness.mjs verify --out /tmp/does-not-exist-xyz` → **exit 1** (`no such output directory: …` on stderr); `node … --help` → **exit 0** on stdout (163 B stdout / 0 B stderr).

**Tests on `cb21688`:**

| Command | Result |
|---|---|
| `node --test tests/exit-codes.test.mjs` | **45 pass / 0 fail** — must stay green; it does |
| `node scripts/lint.mjs` | **872 files checked, 0 findings** |

**Merge state:** PR #15 is **open and awaiting the Lead's merge** — maker never checker; this lane did not self-merge. It is
a docs-only one-phrase correction, not a gate re-open; no fresh checker was spawned by this lane (the Lead's call whether one
is wanted). Merge-commit convention, no force.

## 3. STANDING QUEUE NOTE (for the record — next pulls are external-blocked, not idle)

Nothing in Team A is waiting on something it can do itself; every pull below is named to a **specific external block**.
When the Lead/Nox rules one free, this lane resumes immediately on the item named.

| # | Next pull | Exact external block | What "free" looks like (the trigger) | What I do the moment it frees |
|---|---|---|---|---|
| **Q1** | **Job 03 RUN** (R0–R7 dress rehearsal) — nothing to build before it | **G1** (ruled DSH-era Ergon package installed on a fresh host + install record + tarball sha + recorded `dsh` version), **G2/G3** (Nox rules the C1–C6 freeze set + the ADD-only `main[key]` W1 arbitration), **G4** (owner host purchase + DNS/Supabase project), **G5** (Lead names the RUN checker) | Nox rules G1 + C1–C6/W1; host + package exist; RUN checker named | Execute the RUN sheet as written (R0 team-side, R1–R7 operator steps), record per-step evidence, hand the pack to the fresh checker. **Nothing built before that.** |
| **Q2** | **Parts-bin admission** (component layer) | **Owner's grading of the design comps** (`board/probes/team-a-design-comps/`, filed for OWNER GRADING) — standing instruction: do NOT rework the comps | Graded verdict / graded deltas arrive | Component-layer proposal **S1–S8** through the gate (proposal → all-states → screenshot compare → fresh checker), in order S1/S2/S4 first; licence rulings R1–R4 apply (Animate UI in-app only; Uiverse provenance; Forge UI/Uilora patterns only). |
| **Q3** | **Hallmark intake mission** (Nox `004500Z`, design lane) | Queued **behind the roster/finish work** by the Lead's explicit ordering | Lead removes it from the queue | Pull the intake under the design lane, produce the titled deliverable, proposal-only fences as usual. |
| **Q4** | **Job 04 laptop lane** | Laptop lane is dark until **Mon 2 Nov 01:00 ET**; there is **no route to the laptop** from this desktop | The laptop session starts (or a route appears) | Hand over `board/reports/team-a-job04-monday-resume-pack-r2.md` (checkpoint order + .NET SC-5/FP-5); evidence returns to the Lead's checker. |
| (standing) | **Job 05 Kimi handback** | No handback exists (remote: no kimi/redo branch — re-checked) | A verifiable `kimi`/`redo` branch appears on the remote | Verify branch on remote first, then the alignment audit per the worksheet. |
| (standing) | **Simulation-bank proof runs** (DeepSeek acting agent + one Haiku comparison pass) | Explicitly gated by Nox; the fold-in did not waive them | Nox/Lead opens the proof runs | Run them and report per the bank's evidence rules; they stay required before any RHL deployment claim. |

**Explicit statement for the record:** the blocks above are **external** (Nox rulings, owner acts, the Monday laptop
window, Lead queue order) — not idle lanes. This lane has no self-startable work left in its current scope; the last
self-startable item (the checker nit) is now PR #15.

## 4. Artefacts this batch

| Path / ref | What |
|---|---|
| PR #15 https://github.com/cayleb-james2008/ergon/pull/15 | the one-phrase EXIT-CODES fix (head `cb21688`, base `8822d479`), awaiting Lead merge |
| `board/reports/team-a-batch8.md` | this record (merge confirmation + nit fix + standing queue) |
| `board/reports/checker-ergon-optiona-pr14.md` | the fresh checker's ACCEPT verdict on PR #14 (Lead-landed) |
| PR #14 merge `8822d479` | simulation-bank fold-in on `main` (branch deleted) |

**Fences held:** no spend; no purchases; no credential values; no public sends; no image reads; no host input; docs-only
change (1 line); maker never checker; no self-merge.
