# SAGE-MEM ADMISSION — staging profile, integrity, 50-cycle soak result, named degradation

Filed 2026-11-01T16:47Z by `fleet-lead` (Team B, Lane 3) per Nox's memory-replacement order and the Lead's
staging-shape confirmation (`scratch-tb-l3/staging-profiles/sage-mem-soak`, dedicated staging DSH profile —
**never** the owner's personal profile and **never** the installed fleet profile; Rule W). No installed service,
owner profile, personal data or memory archive touched.

## 1. Staged artefact (pinned, integrity-verified)

| Field | Value |
|---|---|
| Package | **`sage-mem` 2.1.22** (npm; latest at fetch time) — Apache-2.0, repo `github.com/gezi-wen/sage-mem`, 68 files |
| Tarball | `/tmp/sm/sage.tgz`, sha256 `890c983d…`, sha1 `e10d4951…` (matches registry `dist.shasum`), sha512 `nvvcMW…` (matches registry `dist.integrity`) |
| Mount | `dsh plugin --profile sage add --save-exact --ignore-scripts` into the staging DSH home; bundle patch mounts row `sage-mem`; `--dump-config` shows the row |
| Staging home | `~/workspace/harness-build/scratch-tb-l3/staging-profiles/sage-mem-soak/{home,ht,logs}` (dedicated `DSH_HOME`) |
| DSH runtime | the preserved CLI runtime (same version family as the fleet install) |

## 2. 50-cycle admission soak — host stability PASS

Runner: `…/sage-mem-soak/soak.sh` (sha256 `2b48b542…`); one host start → ready (`dsh web:` line) → alive check →
crash-signature scan → stop, 50 times.
**Result (`logs/soak-20261101T164331Z.log`, sha256 `09ca12eb…`): `SOAK-RESULT cycles=50 not_ready=0 died=0
crash_lines=0` — 50/50 cycles ready and alive, no crash lines, no host instability.** One-strike rollback not
triggered by stability.

## 3. NAMED DEGRADATION — autodream cannot initialise on this DSH version

Every boot logs exactly one `[sage-mem]` line:

```
[sage-mem] autodream init failed: Error: cannot get property "timer" without inject
    at AutodreamEngine.svc (…/sage-mem/lib/autodream/engine.js:130:32)
```

- The plugin's own mount code classifies this as a **documented degradation, not a failure**:
  `lib/assembly/autodream-mount.js` wraps autodream in `try/catch`, warns, and continues — core memory (JIT recall
  injection, memory tools, observe, write gate) still mounts. This matches the observed host: alive, no crash.
- Cause: the plugin's `AutodreamEngine` reads the `timer` service without declaring the cordis injection that the
  current runtime requires (`cannot get property "timer" without inject`). The package's own compatibility list
  stops at **DSH 0.2.1-rc.5** and does **not** list 0.2.2-alpha.2 (engines allows `>=0.1.2-rc.1`, so it installs).
- **Capability gap to name:** the "consolidation with rollback" (autodream) capability promised by the adoption is
  **not available as staged** on this DSH version. It is not a crash risk (proven over 50 cycles) but it is a
  functional gap that a wrapped adoption must either (i) fix (plugin update / explicit injection) or (ii) name as
  a limitation in the component map. Wrapping at our layer cannot supply the plugin's internal service declaration.

## 4. What the soak does and does not establish

- **Established:** the package installs and mounts in a dedicated staging profile; 50/50 host cycles stable, no
  crash signatures; degradation is bounded to autodream.
- **Not yet established (next, fresh checker + functional phase):** actual memory tool/JIT-recall behaviour and
  the write-gate/file protocol (needs a DSH *session* in the staging profile driving the plugin's tools, text-only
  verification); memory-data migration readiness (archive at
  `~/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/` stays untouched until PASS);
  packaged/admission acceptance by a fresh checker.
- **Platform fact recorded for stage reporting:** DSH runs plugins in-process with no sandbox; no component is
  crash-proof by selection — the soak + one-strike rollback is the mitigation, and this soak is its first run.

## 5. Status

**Admission phase 1 PASS with one named degradation (autodream).** Not promoted; memory component remains the
single package's OPEN GAP until the functional phase + fresh checker pass and Nox's component-map ruling. Runner-up
(custom Cordis memory tool over the existing file store) is not triggered. Data archive untouched; owner/fleet
profiles untouched.
