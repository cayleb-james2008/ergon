# Team A — Job 04 Monday resume pack, refresh r2 (laptop lane; NO machine route, nothing executed)

**Status: ASSEMBLED, NOT STARTED. No laptop action was taken, no branch was touched, no merge was made.** This refreshes
the 2026-10-31 handoff (`board/reports/team-a-job04-handoff.md`) against the tree as it stands at 2026-11-01 ~23:50Z, so
whoever picks the laptop lane up at **Mon 2 Nov 01:00 ET** has exact, current facts. Facts below were read from the remote
this batch; nothing is from memory. Route to the laptop from this desktop: **none** (unknown/absent — no network path was
found or attempted; the lane is the laptop itself).

## 1. Current remote facts (verified 2026-11-01 ~23:50Z)

| Item | Value | How verified |
|---|---|---|
| Ergon remote `main` | **`c3d86a5ec60cb328c433eaf546019de9f4a92a34`** (`Merge pull request #13 … docs/l5-pack-known-limits`) | `git ls-remote origin refs/heads/main` |
| Laptop branch | **`laptop/acceptance-battery`** tip **`62a277e7b8ac7c0cbc9df990a7bdf45afaeda034`** | `git ls-remote --heads origin` |
| Branch vs current `main` | **diverged — ahead_by 1, behind_by 90**, merge base `8771371fa48d` | `gh api repos/cayleb-james2008/ergon/compare/main...laptop/acceptance-battery` |
| Authoritative checkpoint | `docs/acceptance/order-e/CHECKPOINT.md` on that branch (quoted §2) | `gh api …/contents/docs/acceptance/order-e/CHECKPOINT.md?ref=laptop/acceptance-battery` |

Consequence: the branch was cut from `8771371`; `main` has moved **90 commits** since (Pack 19/20/21, Jobs 01/02, the Job 03
acceptance pack, the design pack v1.0.0 + known limits, and the simulation bank via PR #14). **Any merge from this branch
must first be refreshed onto then-current `main`**, and the refresh must be a deliberate step with the Lead's decision on
merge-vs-rebase (the repo's convention is merge commits, no force).

## 2. Resume order (checkpoint first; the brief's two extra workstreams appended)

The checkpoint wins on facts (Job 04 stop rule). Its own "Resume (fresh session)" list, in order, then Job 04 items 6–7:

1. `git fetch origin; git switch laptop/acceptance-battery` (bundled git on the laptop:
   `C:\Users\KK\Documents\ergon-build-tools\tools\git\cmd\git.exe`).
2. Self-tests on the laptop: `powershell -NoProfile -File docs\acceptance\order-e\tools\Test-ErgonObserve.ps1`
   (expect **40/40**) and `Test-ErgonPass.ps1` (expect **19/19**) — quote both outputs.
3. Re-run the independent acceptance-battery review with a fresh context (the checkpoint's 5-reviewer workflow
   `wf_a966ba46-84d` stopped unfinished; **no finding is applied**; its script path is in the checkpoint). Apply findings.
4. Docs-truth / full suite re-run against the merged tree: `node scripts\test-all.mjs` with
   `ergon-build-tools\tools\{node-src,python-doc-runtime,nsis,git\usr\bin}` on PATH. Baseline at `8771371`:
   **1,529 cases / 1,412 pass / 92 fail, the same 29 checks** (`evidence/suite-main-8771371-windows-laptop.txt`);
   expect no *new* failure from the battery docs. Quote totals + delta.
5. Commit on `laptop/acceptance-battery` only (no main merge from this lane). Then write the one consolidated report:
   what was added, contradictions with the existing pack (S11 too weak for section 20; P0.7 vs sleep cases;
   host-vs-employee install model; 12→11; OpenRouter route now refused), and what cannot be known until a package exists.
6. **Mobile emulator + cross-device extension** (Job 04 item 6): stand up an emulator as the phone connection (enrolment +
   phone flows) and extend the battery to real desktop↔laptop communication, so Cayleb's own phone is a final spot-check,
   not a test dependency. Evidence: emulator enrolment + phone-flow passing in the battery, plus a real desktop↔laptop
   round-trip record.
7. **.NET cross-items** (Job 04 item 7; this lane is the only seat with the toolchain): build and check
   - **SC-5 C# hunk** — `support/app/ScanSetup.cs`, mirroring the Node-side error-code allowlist from
     `onboarding/failure.mjs` (named in `docs/review-debt/ergon-medium-triage-2026-10-31.md` with its pinning test);
     the Node side is CLOSED, C# side HELD (`docs/acceptance/order-f/KNOWN-LIMITATIONS.md`).
   - **FP-5 files** — `support/app/SetupWindow.cs`, `support/app/ProviderCore.cs`, `support/app/ProviderConfigBridge.cs`:
     stop offering the OpenRouter free-only mode; understand `budget-proxy-required` / `LAUNCH_BLOCKED` /
     `budget_proxy_required` instead of the generic local-operation message.
   Build/check entry points (none execute a real network): `node scripts/check-netfx-compile.mjs --refs <dir>` (Mono `mcs`
   vs .NET Framework 4.8 refs; confirm the companion list covers `ProviderCore`/`ProviderConfigBridge` first);
   `bash tests/run-provider-protocol-tests.sh` (Mono twin) or `powershell -NoProfile -File tests/run-provider-protocol-tests.ps1`;
   `bash tests/run-ui-tests.sh` (skip cleanly and name the reason if Mono WinForms/xvfb is absent — never report a pass).
   A pass only if the compiler actually ran; **source-text assertions are not a build**.
8. After the merge decision (Lead's, not this lane's): optionally `Publish-Stage.ps1 -ToolsOnly -Force` to refresh
   `C:\Users\Public\ErgonPass\tools` (checkpoint says the shared tools folder was NOT refreshed).

## 3. Evidence the laptop session must return (so the Lead can spawn the checker)

1. Branch tip SHA after the work + the refresh base it now sits on (and the merge commit if main was merged in).
2. Both self-test outputs (40/40, 19/19) verbatim; suite totals (cases/pass/fail/checks) with delta vs the
   checkpoint baseline, and any new failing check named.
3. The independent reviewer's verdict (who reviewed, what it reviewed, findings applied/rejected).
4. Stub-heading tidy diffstat; consolidated report path.
5. Emulator + cross-device evidence: enrolment + phone-flow result, desktop↔laptop round-trip record.
6. .NET evidence: exact compile/test command lines and output for the SC-5 hunk and the three FP-5 files.
7. Consolidated Job 04 report: every checkpoint item closed or named open with its reason.

## 4. Holds (unchanged, binding)

No route to the laptop from this desktop is assumed or attempted — **prep only**. No spend beyond the Ergon envelope; no
credential handling anywhere (never print/copy/log a secret); no public posts/publication; maker never checker (the
fresh reviewer is not the author; the Lead spawns the checker). Model policy for the Claude laptop lane: session baseline
**Sonnet 5.5**, sub-agents **Haiku by default**, Sonnet only where needed; **DeepSeek R1 is banned outright in Ergon**.
Checkpoint holds that stay closed: no clean user created, no `mcp.bat` repair, no `ProviderProtocolTests.cs`, no API-key
handling.

## 5. Successor

The laptop session at **Mon 2 Nov 01:00 ET** executes §2 and returns §3's evidence; the Lead routes it to a fresh
checker and owns any merge decision. Nothing in this pack has run — this file is the instruction set, not a report of work.
