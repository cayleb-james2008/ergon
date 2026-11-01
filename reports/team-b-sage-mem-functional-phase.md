# SAGE-MEM FUNCTIONAL PHASE — text-only evidence on the staging profile

Filed 2026-11-01T16:52Z by `fleet-lead` (Team B, Lane 3). Staging only
(`~/workspace/harness-build/scratch-tb-l3/staging-profiles/sage-mem-soak`); no owner profile, no fleet profile, no
installed service, no memory archive touched, **no images read** (all evidence text/JSON).

## 1. Time-boxed compatibility check (no spend, no account)

- **No newer `sage-mem` exists:** npm `latest` = **2.1.22** (published 2026-11-01T04:48Z), 95 versions total, no
  later dist-tag. The v2.1.22 release notes fix an unrelated typert-descriptor cache issue and state their
  real-machine verification was on **DSH 0.2.1-rc.5** — confirming this plugin's compatibility baseline stops
  before our 0.2.2-alpha.2.
- **No documented compatibility setting for the autodream/timer failure:** the plugin documents autodream as an
  optional feature with settings (`autodream.json`, `DEFAULT_CONFIG.enabled = false`), but there is **no
  documented env/config switch that prevents the `timer`-service injection failure at mount**. The failure is
  inside the plugin's own service code (`AutodreamEngine.svc` reads `timer` without the inject declaration the
  current cordis requires); a wrapper cannot supply it.
- **Result, plainly: no fix exists today** (no newer version, no documented compat knob). The named limitation
  stands; the successor is the runner-up only if consolidation+rollback is a hard basis requirement.

## 2. Functional phase — plugin core exercised directly (text-only)

Harness: `…/sage-mem-soak/functional/run.mjs`, run 2026-11-01T16:47Z against the installed pinned package; output
JSON `logs/functional-2026-11-01T16-47-06-117Z.json` (sha256 `612eddc9…`). Sample library: four typed memories
(`user_who.md` baseline, `feedback_style.md`, `project_fleet.md`, `reference_links.md`) written through the
plugin's own write path; isolated `SAGE_MEM_DIR`/`SAGE_MEM_STATE_DIR` under the staging dir.

| Check | Result |
|---|---|
| **File protocol write** (`writeFileAtomic`) | PASS — four files created; mode **0600** (the 2.1.19 conservative-mode behaviour) |
| **Create semantics** (`writeFileExclusive`) | PASS — refuses an existing file (no overwrite) |
| **Index maintenance** (`MEMORY.md`) | PASS — index created/updated automatically: `indexEntries=4`, all four listed, no dangling, no unlisted |
| **Directory scan** (`scanMemoryFilesDetailed`) | PASS — `files=4, complete=true, missing=false, errors=[]` |
| **JIT recall selection** (`selectRecall`) | PASS — relevant `project_fleet.md` picked for the test question; **baseline memory injected exactly once** (`baselineTaken=1`); dedupe signature present |
| **Bounded injection** (`buildCandidates`/`composeDelivery`/`composeRecallText`) | PASS — assembled injection text **1,084 chars**, header + citation discipline + per-file bodies (per-file truncation path in code), no unbounded dump |
| **Memory-directory audit** (`auditMemoryDir`) | PASS after fixtures renamed to the plugin's `<type>_<name>.md` convention: `problems=0, fileCount=4, indexEntries=4`, no dangling/unlisted/typeMismatch/noFrontmatter/CRLF |
| Host cycles (previous phase) | 50/50 stable; autodream degradation named (see soak record) |

Note: the first audit run reported 4 `typeMismatch` items because the test fixtures used hyphens; the plugin's
convention is `<type>_<name>.md` (prefix = `f.split('_')[0]`). Fixtures fixed → audit 0 problems. This is a
fixture-naming finding, not a product defect.

## 3. What this establishes and what it does not

- **Established:** core memory is functional as staged — file protocol, conservative modes, create-exclusive,
  index maintenance, scan completeness, JIT recall with baseline-once, bounded injection, and the audit pipeline.
- **Not established / next:** behaviour inside a real *session* (tool calls through the agent surface, write-gate
  rules); the memory-data migration (archive untouched, one-way typed-markdown migration after PASS); formal
  admission by a **fresh checker**; Nox's component-map ruling. Not promoted; one-strike rollback stays armed.

## 4. Named limitation (for the component map, either way)

> **Memory: `sage-mem` 2.1.22 (Apache-2.0) adopted and wrapped in a fleet/staging profile only.** Core memory is
> functional (file protocol, conservative modes, index, JIT recall with baseline-once, bounded injection, audit
> clean on a sample library). **Consolidation+rollback (autodream) is unavailable as staged on DSH
> 0.2.2-alpha.2**: its init fails on the `timer` service injection (plugin compat baseline stops at DSH
> 0.2.1-rc.5; no newer version or documented compat knob exists today), and the plugin itself degrades to core
> memory. **The runner-up (custom Cordis memory component over the existing file store) is the successor only if
> consolidation+rollback is a hard basis requirement.** Home-stability soak: 50/50 cycles, 0 crashes.
