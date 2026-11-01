# Team A — Job 04 handoff packet (laptop lane, starts Mon 2 Nov 01:00 ET)

Prepared by `ergon-lead` 2026-10-31 ~19:30 ET, per the Lead's ruling. **This is a packet, not a start.** The laptop lane
is dark until the Claude usage reset (Mon 2 Nov 01:00 ET) and this desktop has no route to the laptop; nothing here was
executed. Facts below were read from the remote branch, not from memory.

## 1. The branch and the authoritative checkpoint (read this first)

- Repo: `cayleb-james2008/ergon`, branch **`laptop/acceptance-battery`**, tip **`62a277e7b8ac7c0cbc9df990a7bdf45afaeda034`**
  (`git ls-remote` / `git rev-parse` on 2026-10-31). It was cut from `origin/main` `8771371` — **older than current
  main** (`b59b599`, Pack 19 integrated), so after the work it will need a refresh before any merge.
- Checkpoint file (path on that branch, verified by `git ls-tree`): **`docs/acceptance/order-e/CHECKPOINT.md`**. Per Job
  04's stop rule, the checkpoint wins on facts over the brief; where the brief and checkpoint differ, record the
  difference in the lane report.
- Read order for the laptop session: `board/TEAM-ASSIGNMENTS.md` (Team A), `~/Downloads/04-laptop-monday-resume-pack.md`,
  `board/PROGRAMME.md` §8 Job 04, `board/GROUND-STATE.md` (M1, model policy), the CHECKPOINT.md, then
  `docs/acceptance/order-e/` itself.
- Model policy for this lane: Claude lanes session baseline **Sonnet 5.5**, sub-agents **Haiku by default**, Sonnet only
  where needed. DeepSeek R1 is banned in all Ergon work. No spend beyond the Ergon envelope.
- Maker never checker: the reviewer in item 1 must be a fresh context; the lane's own report never closes the job.

## 2. The checkpoint's resume order (from CHECKPOINT.md, quoted structure)

The checkpoint's own "NOT done" and "Resume" sections give this order; Job 04 adds the two workstreams the brief names
(mobile emulator/cross-device; .NET cross-items). Run them in this order:

1. **Independent review** of the acceptance battery (a reviewer who did not write it). The checkpoint names the stopped
   workflow script and says "no finding is applied". Re-run a review with a fresh context; apply findings.
2. **Self-tests first, on the laptop:** `powershell -NoProfile -File docs/acceptance/order-e/tools/Test-ErgonObserve.ps1`
   (expect **40/40**) and `Test-ErgonPass.ps1` (expect **19/19**). Quote the output.
3. **Docs-truth suite re-run against the merged tree:** `node scripts/test-all.mjs` with the bundled toolchain on PATH
   (checkpoint: `ergon-build-tools\tools\{node-src,python-doc-runtime,nsis,git\usr\bin}`), then
   `node scripts/test-suite.mjs`. Expect the same 29 failing checks as the checkpoint's baseline (1,529 cases / 1,412
   pass / 92 fail at `8771371`, same 29 checks) and **no new failure** from the battery docs. Quote totals and any delta.
4. **Stub-heading tidy** in `EDGE-CASE-BATTERY.md` (checkpoint names `E7 (install half)`, `E4.4-E4.6`, `E9.3`) and
   `EDGE-CASE-BATTERY.md`/`RHL-REHEARSAL.md` consistency items the checkpoint lists.
5. **Consolidated battery report** (the checkpoint's item 5: what was added, contradictions with the existing pack,
   what cannot be known until a package exists). Plain language, evidence per item.
6. **Shared-tools refresh after merge** (`Publish-Stage.ps1 -ToolsOnly -Force`), only after the merge decision; the
   checkpoint says the shared tools folder was not refreshed.
7. **Mobile emulator + cross-device extension** (Job 04 item 6). Set up an emulator to act as the phone connection
   (enrolment + phone flows), extend the battery to real desktop↔laptop communication so Cayleb's phone becomes a final
   spot-check, not a test dependency. Evidence: an emulator enrolment + phone flow passing in the battery, plus a real
   desktop↔laptop round-trip record.
8. **.NET cross-items** (Job 04 item 7; this lane is the only seat with the toolchain). See §3.

## 3. .NET cross-items — exact files and build recipe

The two items, as the repo defines them:

- **SC-5 C# hunk.** The Node side of SC-5 is closed; the C# side is held. `docs/acceptance/order-f/KNOWN-LIMITATIONS.md`
  records: "**SC-5** CLOSED on the Node side, **HELD on C#** (`ScanSetup.cs` hunk needs a .NET seat)". The hunk lands in
  `support/app/ScanSetup.cs`; the Node-side behaviour it must mirror is the error-code allowlist added in
  `onboarding/failure.mjs` (the triage record in `docs/review-debt/ergon-medium-triage-2026-10-31.md` names the allowlist
  and its pinning test).
- **FP-5 files** (Pack 15 Order F ruling 1, `docs/PROVIDER-SETUP.md`): `support/app/SetupWindow.cs`,
  `support/app/ProviderCore.cs`, `support/app/ProviderConfigBridge.cs` — the native setup window must stop offering the
  OpenRouter free-only mode and must understand `budget-proxy-required` / `LAUNCH_BLOCKED` + `budget_proxy_required`
  instead of showing the generic local-operation message. Build and check them.

Build/check entry points already in the repo (use the ones the seat can run; none of these execute a real network):

- `node scripts/check-netfx-compile.mjs --refs <dir with mscorlib.dll,System.dll,...>` — compiles each native companion
  with Mono `mcs` against .NET Framework 4.8 reference assemblies (the reference assemblies come from NuGet
  `microsoft.netframework.referenceassemblies.net48`). Companion set includes `GetHelp`, `ReviewDesk`, `ScanSetup`;
  confirm `ProviderCore`/`ProviderConfigBridge` coverage in the script's companion list before claiming a build.
- `bash tests/run-provider-protocol-tests.sh` (Mono twin) or `powershell -NoProfile -File tests/run-provider-protocol-tests.ps1`
  (Windows) — compiles `tests/ProviderProtocolTests.cs` with `support/app/ProviderCore.cs`; needs Mono `mcs`/`mono` +
  `System.Web.Extensions`/`System.Security` on Linux, or Windows csc.
- `bash tests/run-ui-tests.sh` (Mono WinForms + xvfb-run) — the UI checks; skip cleanly and name the reason if the
  seat lacks Mono WinForms rather than reporting a pass.
- The repo's suite is not green on the laptop either: `tests/netfx-compile.test.mjs` needs `ERGON_NETFX48_REFS`,
  `tests/nsis-compile.test.mjs` needs makensis, and 6 manifest invocations need a .NET/Windows seat. Record them as
  **not run here**, never as passing.

**This desktop cannot do the .NET items:** `/usr/bin/dotnet` exists as a **host only** (`.NET Host 10.0.13`), with **no
SDKs** (`dotnet --list-sdks` empty), no Mono/csc and no makensis (checked 17:52 ET). Do not attempt the .NET items here;
the laptop is the seat.

## 4. What the laptop session must return (evidence contract)

Commit on `laptop/acceptance-battery` only (no main merge from this lane; the Lead owns the merge decision), then return:

1. The branch tip SHA after the work and the refresh base it now sits on (if main was merged into it, the merge commit).
2. The two self-test outputs (40/40, 19/19) and the docs-truth suite totals (cases/pass/fail/checks, delta vs the
   checkpoint baseline 1,529/1,412/92, 29 checks) quoted verbatim.
3. The independent reviewer's verdict (who, what it reviewed, findings applied/rejected).
4. The stub-heading tidy diffstat; the consolidated report path.
5. Emulator + cross-device evidence: enrolment + phone flow result, desktop↔laptop round-trip record.
6. .NET evidence: compile/test command lines and exact output for the SC-5 hunk and the three FP-5 files; a pass only if
   the compiler actually ran (no "source-text" assertions counted as a build).
7. The consolidated Job 04 report with every checkpoint item closed or explicitly named as open with its reason.

Successor: Job 03's deployment pack consumes the acceptance evidence; the Lead/`ergon-checker` certifies — the lane does
not certify itself.
