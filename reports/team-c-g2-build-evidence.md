# TEAM C — G2 BUILD EVIDENCE v3 (HOLD-02 closing list) — 1 Nov 2026

trading-lead, 2026-11-01 ~12:1x ET / 17:1xZ. **Build evidence only — no gate signature, no live action, no spend,
nothing armed.** Binding: **G1-SIG-2026-11-01-TS-03** against design v4 sha256
`ea0ec96e1402fd4427706371428fa9cd3316d9c692455a98f87f1fca0e883c1e` — **verified unchanged** (v4 not edited; config mtime
unchanged). Answers the fresh G2 checker's HOLD (`board/reports/team-c-g2-checker-verdict.md`; prior HOLD archived).

## 0. Package
- `~/Work/projects/openbell/live/exec/` — commits **`1916d48`** (build v2) + **`436e5f7`** (README) + **`0893497`** (this
  HOLD-02 fix round), branch `research/kronos-stage1-verdict`, **local only (no remote configured, not pushed)**;
  `git status live/exec` clean; engine/config untouched.
- **New digest: `32dd2bea4bb0296b4d24f8e595efa8b197cd2bec699bcb6f418b0fc8e13384f9`** (was `2a184f26…`). Scope changed
  deliberately: **sorted `*.py` + `deny.json` + `exec_config.json`** (B1-A1 binding). Per-file: `exec-g2a/package-files.txt`.
- Tests: `PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q` → **75 passed** (was 65).

## 1. B1-A1 — the deny anchor is now bound and cannot be neutered away
1. **Integrity binding:** `proof.package_digest()` and `guard.package_sha256()` now cover **`deny.json` + `exec_config.json`**
   in addition to sorted `*.py`. A neutered deny file or an edited rails config changes the digest → the proof is
   `third_party:proof_stale` for the canonical tree. `guard.verify_static_proof` additionally pins **`deny_sha256`** against
   the actual file (`third_party:deny_stale`).
2. **Hardcoded minimum rule set** (`proof.MIN_REQUIRED` / `MIN_COUNTS`), independent of `deny.json`: `rules_sane` checks the
   hardcoded minimums **first**, then the file's own metadata. Removing `required_members`/`min_counts` cannot make it vacuous.
   A missing/malformed `deny.json` no longer raises `KeyError` — `rules_sane` fails closed.
3. **Evidence (planted negatives, `exec-g2a/planted-negatives.json`, /tmp copies):**

| shape | rules_sane | other flags | all_ok |
|---|---|---|---|
| clean copy | true | all true | **true** |
| lists emptied, metadata kept | false | — | false |
| metadata+lists emptied (the old test's shape) | **false** | — | false |
| `deny.json` = `{}` | **false** (no crash) | imports_ok false | false |
| **coherently neutered** (empty rules, real imports allowlisted, metadata empty) | **false** | canary false | **false** |
| neutered proof presented for the canonical tree | — | — | **rejected** (`third_party:proof_stale`, and `deny_stale` if the digest is forged) |

Run on the current tree: `all_ok: true, problems: []`, `audit: {ran: true, canary_ok: true, opens: 74, sockets: 0,
subprocess: 0, deny_opens: []}` (`exec-g2a/static-proof.json`, package digest `32dd2bea…`, `integrity_files: ["*.py",
"deny.json", "exec_config.json"]`).

## 2. B2-E1 — the 2026-10-26 claim is WITHDRAWN (it was wrong)
The earlier evidence called 26 Oct a "real pre-engine day". **It is not.** The engine log for that day records a running
`pulse` engine: two `SHADOW OpenBell BUY 2 MES` cards (16:21 / 16:33 ET bars), two shadow fills and a `-$17.50` shadow exit
(`live/logs/engine-2026-10-26.log`). There is simply **no `live/ledger/2026-10-26.jsonl`**, so the harness's "0 signals"
came from the missing file, not from the day. The artefact `exec-g2a/dryrun-flat-2026-10-26/` has been **deleted**; the only
flat-shape evidence kept is the **labelled 0-byte fixture**.

## 3. B2-E2 — a missing ledger is a data gap, not a flat day
`g2_dryrun.py` now refuses a non-existent `--ledger` unless the caller explicitly passes `--allow-missing-ledger` (documented
as fixture-only):
```
$ python3 g2_dryrun.py --day 2026-10-31 --cards ../logs/cards.jsonl --ledger /tmp/does-not-exist.jsonl …
{"error":"ledger_missing","day":"2026-10-31","path":"/tmp/does-not-exist.jsonl","hint":"a missing ledger is a data gap, not a flat day; use --allow-missing-ledger only for a labelled fixture"}   exit 2
$ python3 g2_dryrun.py … --ledger <labelled 0-byte fixture>  → exit 0, pass_empty true, ledger {exists: true}
```
Tests: `test_g2_harness.py` (missing → exit 2 + `ledger_missing`; flag → PASS-empty with `ledger.exists:false`; existing
0-byte fixture → PASS-empty). The flat command is now documented in `live/exec/README.md`.

## 4. B2-E3 — Lead waiver recorded (flat-day evidence, bounded)
**Ruling (Lead, 2026-11-01): WAIVE the labelled 0-byte fixture as the G2 flat-day evidence**, on three conditions, all met
here: (a) the fixture is clearly labelled (`flat-ledger-fixture-2026-10-31.jsonl`, 0 bytes; stated as a fixture in
`flat_day.note`); (b) the report states plainly that **no real zero-signal trading session exists in the record** (27–30 Oct
each carry ≥1 `signal`; 26 Oct is pre-ledger **with** pulse signals); (c) the first live flat session is a named
**G3 / Monday-conditional obligation** — captured with the same command, attached to the G3 package, **no completeness claim
before it**. **G2 flat-day leg = code-path PASS / live occurrence OWED to G3.**
Fixture run (`exec-g2a/dryrun-flat-2026-10-31/compare.json`): `pass_empty: true`, statement
"PASS-empty — no signals today, no completeness claim", `zero_hash_chain_head: true`, `rerun_identical: true`,
`intents_bytes_identical: true`, `chain_verified: true`, `compare_vs_cards {missing:[], extra:[], mismatch:[], clean:true}`,
`cards 0 / intents 0 / allowed 0 / refused 0`.

## 5. C-notes from the HOLD — closed
| Note | Fix |
|---|---|
| `check_replace` accepted a missing/empty `decision_ref` (and no original) | strict now: requires a non-empty **new** ref, a real original ref, and refuses when either is missing (`rails.check_replace`); tested (`test_check_replace_requires_real_decision_refs`) |
| `must_disable` set the flag but the loop kept calling the venue | `run_dry` has a **disabled gate at the top of the card loop**: after one fatal venue error the remaining cards are refused as `disabled_after_venue_error` **without any further venue call**; the raw exception name is no longer counted as a rail reason (kept in `venue_error`). Test: venue called **once**, remaining cards refused, chain verifies |
| R8 literal `60` fallback | removed: threshold comes **only** from `rails.feed_stale_s`; if absent, `config_missing:feed_stale_s` is a refusal (tested) |
| "74 opens" wording | corrected: `opens` varies with bytecode caches (73–74 observed); the gated numbers are **0 sockets / 0 subprocess / 0 deny-path opens** |
| runtime trio shares one audit gate | documented: the audit child reports `ran` + `canary_ok` separately from the event counts, and a failed audit run conservatively falsifies the three (extra false, never a false pass) |

## 6. Regression and proof numbers (all re-run 2026-11-01 ~17:0x–17:1xZ)
- Suite: **75 passed**.
- Proof: six flags true, `problems: []`, package digest `32dd2bea…`, deny digest `9b82d268…`, audit 74 opens / 0 sockets /
  0 subprocess / 0 deny opens.
- Planted negatives: clean `all_ok:true`; `smtplib`+`telegram` in executor.py → `imports_ok:false`; cookie read in
  **venue.py** → `ast_deny_ok:false` + `filesystem_ok:false`; canary bypass → `canary_ok:false` (sentinel leak caught);
  emptied/neutered deny → `rules_sane:false`; all `all_ok:false`.
- Cards day 2026-10-30 (real inputs): `rerun_identical`, `intents_bytes_identical`, `chain_verified` true; compare 0/0/0;
  chain head `1121cdbb04b61ece23d5ab65d6058e582766222627fbdc833a52e090b0fbf492`; reconcile clean (1 intent, 1 decision).
- Flat fixture: PASS-empty, zero hash, 0/0/0, byte-identical (see §4).

## 7. S-3 (unchanged from HOLD-01 closure)
Three keys stay in `live/exec/exec_config.json` (`daily_consistency_cap_usd 1000`, `max_replaces_per_intent 1`,
`max_cancel_replace_per_10min 3`); `guard.assert_desk_config()` reads `live/config.json` **read-only** and fails closed on
any of 10 shared rail drifts (real config → `[]` today); `live/config.json` mtime unchanged (2026-10-27T22:43:34Z).

## 8. Honest limits
- `canary_ok` = in-process hostile-transport exercise of the venue surface (S-5 forbids a real CDP transport at G2); the real
  page/screenshot canary remains a **G2.5 obligation**.
- Flat-day live occurrence owed to G3 (Lead waiver, §4).
- Live executor loop and replace **path** are G3 work (predicate/handler contract tested now).
- G2.5 / G3 separate; nothing armed; `mode: dry-run`, no `enabled.json`, no transport instantiated, no network.

## 9. Artefacts for the third-seat checker
`exec-g2a/`: `static-proof.json`, `planted-negatives.json`, `package-files.txt`, `build-record.txt`,
`dryrun-2026-10-30/`, `dryrun-flat-2026-10-31/`, `flat-ledger-fixture-2026-10-31.jsonl`, `r11-fixture-2026-10-30.json`.
Reproduce: `PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q`; `python3 -c "import proof;print(proof.build_proof('.'))"`;
`python3 g2_dryrun.py …` (README §Commands). Fences: no live action/order/credential/spend; no engine/config/v4 edit.
