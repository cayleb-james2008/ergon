# TEAM D — DIAGNOSTIC PLAY 10 REPORT: FINAL ATLAS PLAY — 20/27 captured, freeze moved to world-overhead (2026-11-02)

Lead GO: 2026-11-02T06:02Z. Machine `outis-desktop`. Sequence: read-only state + sealed BEFORE → `preflight` → **one**
`run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER. No extra Plays, no build/publication/credential, no
image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T060333Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play10-20261102.log`.

**State correction (stated exactly as found):** the Lead's expected opening (pid 814951, 2/3) had been closed by the
`studiocycle` that was itself instructed for Play B; the current opening is **pid 881911** with **plays 1/3** (Play B used
one). This Play therefore ran as **2/3** and **one Play remains unused** — no reserved Play was consumed beyond the
instructed final atlas Play.

## 1 · Verdict in one paragraph

The per-zone plan reached the first overhead and **`city-overhead` captured fresh (view-020)**, then the run failed closed
at the **next whole-map framing, view-021 `world-overhead`**: three captures 1.0 s apart were **byte-identical to
view-020's image** (`sha 0ac9595d3372a880…`, `cameraDelta:0`) after the 1.0 s overhead settle — the **same renderer/Wine
stall class** that previously hit `highway-overhead`. **20/27 views captured** (20 distinct sha256, `viewportMatchesPng:true`
on all, fog legacy record, `accepted`/`stationary`); the four new zone overheads (ordinals 22–25) were **never reached**
because `world-overhead` (ordinal 21) precedes them, so the B substitution's captures remain unexercised. The
`capture-attempts` audit is consistent (**21/21** == `result.json`); `fastPathApplied=true`; **no crash** (abort
`stop/cleanup/verifyClean` ok); sealed artifacts unchanged; plays **2/3**. Named limitation extended: it is **whole-map
overhead framings (whole highway AND whole world)**, not the highway alone — `city-overhead` is fine.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state (as found) | Studio pid **881911**, window `0x558c7f892380`, 2200×1000 floating, **Edit**, plays **1/3**; place == build == `c6651793…`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 06:03:33Z, ended 06:04:09Z (~36 s); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 3.335 → PREPARED 3.442 → PLAYING 5.335 → RAN 8.253`; exit **1** |
| Error | `FixtureFailed: view-021: screen_capture returned the same bytes as an earlier view on all 3 attempts (stale capture; kind=overhead settle=1.0s; attempts recorded)` |
| Captures | **20 sidecars** (views 1–20 of 27), 20 **distinct** sha256, `viewportMatchesPng:true` on all, `accepted:true`, `stationary`, fog `legacy:true`; view-020 `city-overhead` = **fresh** |
| view-021 `world-overhead` | 3 attempts, settle 1.0 s, 1.0 s retry waits; **all three `sha256 0ac9595d3372a880…` = view-020 `city-overhead`'s bytes**, `duplicate:true`, `cameraDelta:0` |
| `captureAttempts` | `result.json` 21 entries == `capture-attempts.json` 21 entries (**consistent**, the layer-5 fix holds) |
| `worldTrace` | `fastPathApplied=true`, actual==expected seed `2134783819` / layout `2`, `rebuild=false`, starter applied before `rootFoundAt`; `world-trace.json` written |
| Collector | phases `prepared` + `view-001 … view-021` (21 acks), rejections `[]`; chunks `null` (actual) |
| Runtime/validator/index/look | **not reached** — the run stops at the capture failure; no runtime/validator/index/look outputs, no cell verdict |
| Crash state | **no crash**: `newCrashDumps:null`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}`, hostcheck clean |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **881911**; **plays 2/3** (one Play remains unused in this opening) |

## 3 · Judgement

- **Do the per-zone views capture?** **Not exercised** — they were never reached (ordinals 22–25 sit after
  `world-overhead` at ordinal 21). Their plan/validator/server wiring is checker-accepted offline; the Play blocked first.
- **Did runtime/validator/index/look complete?** **No** — the cell fails at the capture stage, so there is no validator,
  index or look evidence and **no cell pass**.
- **What it proves:** the B substitution + allowlist fix work far enough for the plan to run to the **second** whole-map
  overhead; the first whole-map overhead in this order (`city-overhead`) captures fresh; the audited capture path
  (settle/retry/`capture-attempts` consistency) holds on the failing path; the fast path, named seed, sealed integrity
  all hold; the failure is **named, fail-closed and evidence-preserving**.
- **Class correction (important):** the named renderer/Wine limitation is **whole-map overhead framings — `highway-overhead`
  AND `world-overhead`** — not the highway framing alone; `city-overhead` captured fresh this Play. The limitation note
  (`board/probes/team-d/overhead-render-limitation-contingency-20261102.md`) is updated with this Play-10 evidence and the
  correction; nothing is implemented beyond the ruling.
- **What it does not prove:** the zone views' captures; anything downstream of views 21–27 (runtime, coverage index, look);
  the cell does not pass.

## 4 · Next (for the Lead/Nox ruling; no plan change made)

The blocker is the **second whole-map overhead framing**. Options (NOT implemented): extend the same **B substitution** to
`world-overhead` (e.g. bounded world/quadrant overhead sections through the same audited path, or drop it as a documented
named limitation), which needs the Nox ruling because it is a plan/validator change; reordering cannot help (`world-overhead`
expires the cell anyway). The one Play left in this opening and the reserved Plays stay untouched pending that ruling. The
Play 9 report + abort-contact options note are filed separately (`team-d-diagnostic-play9.md`,
`board/probes/team-d/abort-contact-window-options-20261102.md`).
