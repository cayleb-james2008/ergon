# TEAM D — DIAGNOSTIC PLAY 8 REPORT: per-zone overheads Play — server allowlist cap blocks the 27-view plan (2026-11-02)

Lead GO: 2026-11-02T05:40Z. Machine `outis-desktop`. Sequence: read-only state + sealed hashes BEFORE → `preflight` →
**one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER. No extra Plays, no build/publication/
credential, no image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T054613Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play8-20261102.log`; console:
`board/probes/team-d/console-after-atlas-play8-20261102.json`.

## 1 · Verdict in one paragraph

The per-zone plan reached the Play and the fixture preparation succeeded (`prepared`: candidate pinned, **101 sources
verified**, `productionSourcesUntouched`, namedSeed 2134783819), but the run **failed closed before view-001**: the
server-side **allowlist bound still caps the plan at 24 views** —
`ServerScriptService.Nightshift.AtlasServer:56: candidate/allowlist mismatch` — so the **27-view zone plan is rejected**
(`#manifest.views<=24` is stale after the B+A substitution; the client cap was raised to 32 but the server's was not).
Consequence: **0 views captured** (planned 27), no runtime/validator/index/look evidence, no cell verdict; the runner
reported `ValueError: runtime arrived before view-001` (exit 1). **No Studio crash** (`newCrashDumps:null`; abort
`stop/cleanup/verifyClean` all ok), sealed artifacts unchanged, `verify-clean` clean, plays **2/3** in the recovered
opening. This is **layer 6 = stale server-side view-count allowlist bound** — a one-line fixture fix, not a zone-view
defect.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **814951**, window `0x558c7f831f20`, 2200×1000 floating, **Edit**, plays **1/3**; place == build == `c6651793…`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 05:46:13Z, ended 05:46:31Z (~18 s); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 1.257 → PREPARED 1.366 → PLAYING 3.256 → RAN 6.406`; exit **1** |
| Error | `ValueError: runtime arrived before view-001: {"error": "ServerScriptService.Nightshift.AtlasServer:56: candidate/allowlist mismatch…", "starter": "{\"applied\":2134783819,\"t0\":4753.769686699961,\"seed\":2134783819}"}` |
| Root cause (exact line) | `tests/atlas/Server.luau:56`: `assert(manifest.candidate==cfg.candidate and #manifest.views<=24,"candidate/allowlist mismatch")` — the server-side bound was not updated when the scout plan went to 27 views (client cap is 32; only this one `<=24` remains in the atlas sources) |
| Prepared (before the failure) | `prepared.report`: `localPlace:0`, candidate pinned, `namedSeed:2134783819`, `productionSourcesUntouched:true`, `verifiedSources:101`, `skipOpeningFares:2` |
| Captures | **0** (`views/` absent/empty; planned 27 incl. the four zone overheads); no fog/viewport/technology records this Play |
| Collector | phases `["prepared","runtime","cleanup"]` (no view acks), rejections `[]`; chunks `null` |
| `worldTrace` | `null` — the server asserted before publishing the trace; console confirms the same error from `AtlasServerStarter:1` (stack `AtlasServer:56 function start`) |
| Crash state | **no crash**: `newCrashDumps:null`, `gpuJournal:null`, `host {stalls:0,label:"clean"}`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}` |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **814951**; plays **2/3** in the recovered opening |

## 3 · Judgement (batch 7 §6 + Lead's criterion)

- **Did the per-zone overheads capture?** **No** — the Play never reached view-001: the server's stale 24-view allowlist
  bound rejected the substituted 27-view plan before any camera work. The four zone views themselves are unexercised in
  Play; their plan/validator/server wiring is verified offline only (checker delta on `d83efa8` ACCEPT; suites green).
- **Did the audited capture path hold?** Not reached — no captures, so no `capture-attempts` this run (the audit file
  would mirror 0 views); the runner's fail-closed early-stop kept the evidence honest (`worldTrace:null`, collector
  phases, console).
- **What it proves:** the preparation/candidate pin, named seed, owned-prefix and Play lifecycle all work with the
  substituted plan; the fail-closed early stop and cleanup behave; sealed integrity holds; **the B+A plan itself still
  needs the server-side bound fix before it can be exercised**.
- **What it does not prove:** nothing about the per-zone overhead captures, fog/viewport records, runtime/validator/
  index/look evidence, or cell acceptance; the cell does not pass.

## 4 · Proposed named fix (NOT implemented — for the Lead's ruling; smallest change)

Layer 6 fix: in `tests/atlas/Server.luau:56`, raise the server-side allowlist bound to match the client/plan contract —
`#manifest.views<=32` (fixture-only; identical to the client's `<=32`), plus offline tests that pin the bound agreement
and reject a >32 plan. Optionally fold the checker's R3 note into the same offline freeze: a **stricter full-list
assertion** so a whole-highway view cannot re-enter the *full* planned list (currently only the scout pin, so an unused
whole-highway view in the full list stays green). Freeze → checker delta → the next atlas Play validates the four zone
overheads (captures/`capture-attempts`/fog/viewport/runtime/validator/index).
Play budget: this Play consumed the second Play of the recovered opening → **2/3 used, 1 reserved**; a per-zone Play
would need the Lead's sequencing (current opening's last Play or a fresh opening).

## 5 · Fresh opening for Play B (performed after this report, as instructed)

`studiocycle cycle` (05:47:44Z): close `method=killed-after-dialog` (autorecoveries parked, nothing saved; place sha
unchanged `c6651793…`), reopen **pid 881911**, window `0x558c7f892380`, 2200×1000; sealed place sha before==after
`c6651793…`; sealed artifacts unchanged (`84324d43…`/`c6651793…`). `preflight` rc 0, refusals `[]`, **Edit, plays 0/3**
(fresh process; the record shown belongs to the closed pid 814951 with 2/3), place == build == `c6651793…`, host quiet.
**Play B not run** — it waits for the Lead's GO; no extra Plays consumed.
