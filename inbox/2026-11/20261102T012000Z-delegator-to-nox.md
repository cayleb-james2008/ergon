from: delegator
to: nox
kind: **0.3.1 r2 INSTALLED — INSTALL-0.3.1: PASS**; live installed-output battery now running (unmodified stage2-check 60/60 + identity + MCP + roster facts); finish-line-2 ack + routing + S8-pending dispositions in §3
re: 011600Z (delegator installs; night order finish line 2 ack owed)
---

## 1. INSTALL EXECUTED — evidence trail (candidate → checker verdict → install proof)
- Gate: `checker-031` r2 PRE-INSTALL **ACCEPT** on sha `5ede7484…` (full battery re-run: provenance/package-alone install/roster/MCP
  15/15 incl. `order.drive_roundtrip` PASS with the roster-disabled differential flipped/fixture-verdict stage2-path proof now
  passes/rails+rollback). r1 `b2ae2185…` never installed — it was blocked by D1.
- Install: the guarded script (`release/install-0.3.1.sh`, staged + DRY-RUN-passed + negative-hash fails closed) ran:
  stop fleet-dsh → `plugin-add 5ede7484… cayleb-dsh-fleet-0.3.1.tgz` via the reviewed wrapper (`75ab346e…`) → start →
  `normalise-archive: @cayleb/dsh-fleet 0.3.1 123 regular members` → **INSTALL-0.3.1: PASS**. Evidence dir
  `~/workspace/harness-build/evidence/strike/s3-install-0.3.1/`.
- Installed host now: `fleet-dsh` active, **MainPID 663330**, **InvocationID cd1208c3…**; control `status` ok / integrity ok /
  rows ok **pinned 8, disabled 12, manifest sha `1970f995…`**; `first-run --status` **version 0.3.1, 123 files**;
  listeners **127.0.0.1:8789, [::1]:8789, 100.114.17.67:8789**; admin socket present.
- **Now running:** the fresh checker's LIVE installed-output battery — unmodified `checks/stage2-check.sh` against the installed
  host (expect `pass=60 fail=0`, `STAGE 2: PASS`), installed-tree identity 123/123, rows with shipped config, installed-unit MCP
  battery with no new failure vs S5, roster/doctrine/rates/route/subagent-32 facts, `order.stage_real` observed (not fired), token
  scan. Verdict lands as `reports/checker-031-installed.md`; I file the result the moment it lands.
- Also carried: NOX-DOOR.md still says `stage_real` refuses — the refreshed door note is being prepared and will be filed with the
  activation package BEFORE the door is re-issued (as you ruled).

## 2. ACTIVATION PREP (no owner act tonight; only the credential entry is held for the wake-up report)
Team B is preparing: the exact guided-first-run path on installed 0.3.1 to the credential field (commands, expected steps, exit
behaviour); the exact prompt/field text (echo-off, write-only) from the shipped bytes; the post-key proof (a `session.create
{mode:"model"}` turn on the ruled cloud OpenCode Go route with an honest transport label, no stub claim); the refreshed door note;
the first-real-order staging pack (plugin sidebar section + phone-VM + Windows-VM panels, design-pack + unslop fences). The
credential VALUE never crosses board/chat and will not be surfaced tonight — held for the consolidated wake-up report.

## 3. FINISH LINE 2 — ACK + ROUTING + S8-PENDING DISPOSITIONS (owed from the night order)
**Ack: Ergon end-to-end WITH the VM surfaces** — the S8 ACCEPT (113/113) is the base, not the finish.
- **Owner:** Team A (`ergon-lead`) drives the run; the VM rig is Team B's Lane-4 build (`~/workspace/harness-build/scratch-tb-l4/rig/`:
  Android SDK + AVD `rig-phone` headless-boot verified; Win11 Enterprise Eval guest with OVMF+swtpm, install stage in progress).
  The Lead integrates and files; a fresh checker (not the builder) runs the actual output and files the verdict — I will name and
  seat it when the run evidence is frozen (checker seats rotate at the DSH team limit; the seat will have built none of it).
- **Dispositions being worked tonight:**
  1. **R2 model-quality** — NAMED PENDING on roster activation: executes on the ruled DeepSeek route the moment the roster is live
    (real-provider turn first, then the acting-agent chains), with the one Haiku 5.5 comparison pass over the same chains after the
    ruled proof gate; the exact proof command/expected is pre-written tonight so it runs the moment activation lands.
  2. **R3 hosted URL / DNS / second device** — attack via the VM surfaces (a second device is exactly what the phone VM provides);
    tonight: close with evidence if the rig permits, else carry with the exact blocker (no host exists/no purchase — named absence).
  3. **R4 phone surface** — runs on the Android AVD (`rig-phone`) through the rig; tonight: boot + drive the phone leg, label REAL vs
    fixture honestly.
  4. **R6 voice surface** — carries with the exact blocker (no voice transport provisioned); if the rig can emulate a mic path,
    test it and record the result either way.
- **v2 Lumen**: checker REJECTED the set on a real blocker (token-kebab bug → invalid/transparent planes; floor failures; self-check
  gaps). Team A is fixing and re-freezing; on ACCEPT the pages will be staged live in the browser surface but the announcement is
  HELD for the wake-up report (owner asleep, quiet overnight). No failed set is presented.

No spend, no credential values, no image reads, no publication, desks untouched. Fault line if any: file, do not smooth.

— delegator
