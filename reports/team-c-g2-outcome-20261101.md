# TEAM C — G2 OUTCOME RECORD (SIGNED with conditions) — 1 Nov 2026

trading-lead, 2026-11-01 12:1x ET / 17:1xZ; updated 12:1x ET with the Lead's C1 adjudication + C3 verbatim waiver.
**No live action, no spend, nothing armed; the signed tree was not edited after the sign.**

## G2 STATUS: **SIGNED with conditions** (Lead adjudication 2026-11-01)
- **Signed artefact:** `~/Work/projects/openbell/live/exec/` — commit **`0893497`** (chain 1916d48 → 436e5f7 → 0893497),
  **digest `32dd2bea4bb0296b4d24f8e595efa8b197cd2bec699bcb6f418b0fc8e13384f9`** (18 `*.py` + `deny.json` +
  `exec_config.json`), branch `research/kronos-stage1-verdict`, local only, not pushed.
- **Checker:** third fresh Tier S seat, verdict **PASS (sign with conditions)**
  (`board/reports/team-c-g2-checker-verdict.md`, checked 16:55–17:10Z; HOLD-01/02 preserved as
  `…-20261101-HOLD.md` / `…-HOLD2.md`).
- **Design v4 untouched** (`ea0ec96e…`, G1-SIG-2026-11-01-TS-03); `live/config.json` mtime unchanged; engine/config
  untouched; `git status live/exec` clean; 75 tests pass; nothing in the package can send (`mode: dry-run`, no
  enable/lock/static-proof artefact, no CDP transport, audit 0 sockets / 0 subprocess / 0 deny-path opens).
- **C1 ADJUDICATION (Lead): SIGN G2.** R-1 is **not** a HOLD: a coherently weakened deny file can pass its OWN copy's
  proof, but that proof is **rejected for the signed tree** (`proof_stale`, `deny_stale` if the package digest is forged)
  and the shipped tree is digest-bound — the gate's purpose (no weak proof accepted for the shipped artefact) is met.
  **R-1 becomes a NAMED G3 HARDENING OBLIGATION:** hardcode the full deny set (or pin `deny.json` to a constant
  independent of the proof record) **and** add the negative test (weakened-deny shape must fail).
- **C3 WAIVER — verbatim, as ruled by the Lead (recorded exactly):**
  > "RULING ON B2-E3 (flat-day evidence): WAIVE the labelled 0-byte fixture as the G2 flat-day evidence, under these
  > conditions: the fixture is clearly labelled as a fixture in the evidence and the report states plainly that no real
  > zero-signal trading session exists in the record (27-30 Oct all have >=1 signal); the live zero-signal session capture
  > is recorded as a named G3 / Monday-conditional obligation: the FIRST live flat session is captured with the same
  > command and attached to the G3 package, and no completeness claim is made before it; the flat-day code path's
  > correctness stays proven by the fixture (PASS-empty wording, zero-hash head, 0/0/0, byte-identical rerun)."
- **C3 follow-up (recorded honestly):** the `flat_day.note` **in the shipped tree** does not contain the word "fixture"
  (the fixture is labelled by filename `flat-ledger-fixture-2026-10-31.jsonl`, by `live/exec/README.md`, and by this
  record). The note-text correction is a **source change inside the signed digest**, so — per the Lead's "do not touch
  `live/exec` after the sign" — it is **folded into the G3 package** (C3 split: waiver recorded now; note wording + the
  build-evidence parenthetical corrected in G3, which re-binds a new digest under its own gate).
- **C3 HANDLING RULING (Lead, 2026-11-01 — recorded here): leave the wording to G3; NO post-sign revision and NO
  re-sign.** The waiver is recorded now; the fixture is already labelled by filename (`flat-ledger-fixture-2026-10-31.jsonl`),
  by `live/exec/README.md` and by this record, so the `flat_day.note` text is a **wording improvement belonging in the G3
  package**, alongside C2/C4/C5/C6 + R-1/R-2. **`live/exec` must not be touched after the sign** — the signed digest
  `32dd2bea…` / commit `0893497` stands.
- **C2 / C4 / C5 / C6 — folded into the G3 package** (docstrings; stronger neutered-proof test; real ISO `generated_at`;
  enforce "fixture-only" on `--allow-missing-ledger`). They are source changes inside the signed digest and must not be
  made now.

## Reproduced by the checker before the sign (all PASS)
Digest + per-file manifest + sensitivity of each extra file; v4/config/git/tests; five deny-neutering shapes fail closed;
neutered proof rejected on the canonical tree; `deny_sha256` pin; 26 Oct withdrawal + artefact deleted; B2-E2
`ledger_missing` exit 2 + fixture flag + tests + README; B2-E3 fixture byte-identical with the G2/G3 split recorded;
C-notes verified in code; cards-day regression byte-identical (`1121cdbb…`, 0/0/0); six flags true, `problems: []`.

## Conditions and residuals carried into G3 (none affect the signed tree)
| id | Item | Disposition |
|---|---|---|
| **R-1** | Deny-minimum subset: coherently weakened deny file + allowlisted non-minimum import passes its OWN copy's proof (rejected for the signed tree) | **G3 hardening obligation:** hardcode the full deny set / pin a constant + add the weakened-deny negative test |
| C2/R-3 | `g2_dryrun.py:10` "pre-engine" stale; `proof.py:2–3` "each flag own run" imprecise | G3 package |
| C3/R-4 | `flat_day.note` "fixture" wording + evidence parenthetical | G3 package (waiver itself recorded verbatim above) |
| C4/R-5 | weak `test_neutered_proof_rejected_on_canonical_tree` (copies 10/20 files) | G3 package |
| C5/R-6 | placeholder `generated_at` | G3 package (real ISO timestamp) |
| C6/R-7 | `--allow-missing-ledger` documented fixture-only but not enforced | G3 package (filename/label check) |
| R-2 | `package_digest` skips a deleted `exec_config.json` (genuine proof still fails) | G3 hardening (require both extras) |

## G2.5 / G3 sequencing — CONFIRMED by the Lead (started now)
1. **G2.5 — CDP/DOM gate (plan note written, no tree edits):** see
   `board/reports/team-c-g2p5-plan-20261101.md`. Read-only page-snapshot/selector/tab-navigation/expected-page/screenshot
   evidence, checker pre-sign, **no sends**; the page work must happen when the signed-in Tradovate tab exists and outside
   Monday's 09:05–16:20 trading window.
2. **G3 — demo venue package:** requires demo environment + demo access (owner assets: Cayleb/Nox); carries C2/C3/C4/C5/C6,
   R-1/R-2 hardening, `flat_day.note` wording, and the **first live zero-signal session capture** obligation.
3. **Monday 2 Nov — unchanged, cards/guard-only:** 01:00 ET Kalshi resume (packet v7) → 07:45 sweep → 09:05 guard
   (+ R11 CME-limit record before the 09:15 attestations) → 09:15 attestations ((e) "mode=cards — NOT automated") →
   09:35–09:44 binding → 09:45 → 16:05 flat → 16:20 outcome. No autonomous path, no fills claim, nothing armed.

— outcome recorded by trading-lead; signed tree untouched (`32dd2bea…` / `0893497`).
