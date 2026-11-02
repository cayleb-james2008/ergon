# TEAM D — DIAGNOSTIC PLAY 7 REPORT: overhead settle Play — frame still frozen ⇒ renderer limitation established (2026-11-02)

Lead GO: 2026-11-02T04:58Z (recovered opening). Machine `outis-desktop`. Sequence: read-only state + sealed hashes
BEFORE → `preflight` → **one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER. No extra Plays, no
build/publication/credential, no image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T045824Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play7-20261102.log`.

## 1 · Verdict in one paragraph

The layer-5 settle fix **executed exactly as designed and established the renderer/Wine limitation**: at view-021
`highway-overhead` the runner applied the **1.0 s overhead settle**, then took **three captures** (1.0 s retry waits) and
**all three returned view-020 `city-overhead`'s exact bytes** (`sha 74d4f7e9934dd49d…`, `cameraDelta:0`) while the client
had acknowledged view 021 — the frame never advanced even after ~3 s of bounded settle+retries. The runner failed closed
with the named `… on all 3 attempts (stale capture; kind=overhead settle=1.0s; attempts recorded)`; the audit is
**consistent** (`result.json.captureAttempts` == `capture-attempts.json`, 21/21 entries — the delta fix holds on this
failing path). **No Studio crash this Play** (`newCrashDumps:null`, abort `stop/cleanup/verifyClean` all ok) — the earlier
Play-6 minidump is therefore an intermittent Wine shutdown crash, not deterministic. 20/24 views captured again (20
distinct sha256, `viewportMatchesPng:true` all, fog legacy record, derived `Future` technology). **This is the
renderer/Wine-limitation point: per the Lead, no plan/validator change is made; the honest ruling note with options is
filed for Nox.** Sealed unchanged; plays **1/3** in the recovered opening.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state (as found) | Studio pid **814951**, window `0x558c7f831f20`, 2200×1000 floating, **Edit**, recovered opening **0/3** (play record still belongs to dead pid 797879 with 2/3); place == build == `c6651793…`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 04:58:24Z, finished 04:58:56Z (**32 s**); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 3.492 → PREPARED 3.602 → PLAYING 5.492 → RAN 8.404`; exit **1** |
| Error | `FixtureFailed: view-021: screen_capture returned the same bytes as an earlier view on all 3 attempts (stale capture; kind=overhead settle=1.0s; attempts recorded)` |
| `captureAttempts` (result.json, 21 entries) | views 001–020 one attempt each; **view-020 `overhead` settle 1.0 s, 1 attempt (fresh)**; **view-021 `overhead` settle 1.0 s, 3 attempts — all `sha256 74d4f7e9934dd49d…` == view-020 `city-overhead`'s sha, `duplicate:true`, `cameraDelta:0`** |
| Audit consistency | `result.json.captureAttempts` == `capture-attempts.json` → **21/21 entries equal** (the layer-5 delta fix holds on the failure path) |
| Captures | **20 sidecars**, 20 **distinct** sha256, `viewportMatchesPng:true` on all, `accepted:true`, `stationary`, JPEG, 1837×744; fog record `legacy:true`; technology `"Future"` via `sealed-place-xml` |
| `worldTrace` | `fastPathApplied=true`, `actualSeed=2134783819`, `actualLayout=2` == expected, `rebuild=false`, starter applied before `rootFoundAt` |
| Crash state | **no crash this Play**: `newCrashDumps:null`, `gpuJournal:null`, `host {stalls:0,label:"clean"}`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}`; newest minidump remains Play-6's `2742f914-…` (04:22:48Z) |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **814951**; plays **1/3** in the recovered opening |

## 3 · Judgement (batch 7 §6 + Lead's exact criterion)

- **Did the overhead settle unblock views 21–24?** **No.** The settle ran (1.0 s before the first capture, 1.0 s retries)
  and the frame stayed on the previous overhead image for ~3 s total; the capture path returned the prior frame each time.
  The retry/settle machinery is not at fault and no dishonest evidence was stored.
- **This is the renderer/Wine-limitation point** (Lead's criterion): no plan, validator or production change is made. The
  honest ruling note is filed at
  `board/probes/team-d/overhead-render-limitation-contingency-20261102.md` (updated in place; now titled as the ruling
  note) with options A (documented limitation + named view), B (per-zone functional views replacing the single
  whole-highway overhead — plan/validator change), C (reduce framing cost), D (cap the overhead set); recommended **B+A**.
  It states plainly what the evidence supports (persistent previous-frame capture on the whole-highway overhead; userland
  minidump in the earlier Play; no kernel GPU evidence) and what is unknown (exact render/capture stall mechanism,
  `world-overhead` behaviour — never reached).
- **Crash class correction:** Play 6's 106 MB userland minidump at `StopPlaySoloShutdownDataModels` did **not** repeat —
  this Play aborted cleanly with no crash. So the crash is an **intermittent Wine Studio shutdown crash**, not caused by
  the overhead view or the settle; sealed integrity held both times.
- **What it proves:** the bounded settle+retry executes and is auditable; the duplicate is a genuinely frozen overhead
  frame (not a short race); the retry/audit contract stays fail-closed and consistent; 20/24 captures are repeatable;
  `fastPathApplied=true` again; no crash on a clean abort.
- **What it does not prove:** no views 21–24, no runtime/validator/look evidence, **no cell pass**; the atlas cell remains
  blocked on the overhead-render limitation until the Lead/Nox option ruling.

## 4 · Next

Awaiting the Lead/Nox ruling on the limitation options (no plan change implemented). Fixture **A+C** (abort-contact,
Nox-ruled: closed-loop pursuit `clamp(proxy+8,12,60)`, 40 studs/s² cap, >700/800 radius guard, 12 s no-touch timer; B
fallback only with exact bounded values; D rejected) is the next separate offline freeze and does not consume a Play.
The recovered opening is at **1/3**; the remaining Plays stay reserved pending the limitation ruling and Play B.
