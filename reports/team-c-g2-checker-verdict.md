# TEAM C — G2 CHECKER VERDICT v3 (HOLD-02 closure round) — 1 Nov 2026

**Seat:** fresh-context, independent Tier S checker with VETO. Did not build anything here; reports to the Lead only.
**Subject:** `~/Work/projects/openbell/live/exec/` at commit **`0893497`** (chain `1916d48 -> 436e5f7 -> 0893497`),
branch `research/kronos-stage1-verdict` (local only; `git remote -v` empty), against design v4
`ea0ec96e1402fd4427706371428fa9cd3316d9c692455a98f87f1fca0e883c1e` under **G1-SIG-2026-11-01-TS-03**.
**Prior verdicts preserved:** HOLD-01 `board/reports/team-c-g2-checker-verdict-20261101-HOLD.md`
(sha256 `4c4db6d1309e87fe4479d11149dc5697afcc72eda3376a29b49552c893b5fa20`); HOLD-02 preserved byte-identically as
`board/reports/team-c-g2-checker-verdict-20261101-HOLD2.md`
(sha256 `54dccb25f577f9bfd51e9e342e8e66c8a3d5a43a19625401aba6885187830922`, 30192 bytes).
**Fences observed:** read-only on the package, engine and config; every mutation ran on `/tmp/g2check3` copies or in
memory; no live action, order, wallet, credential, spend or network send. Only write = this report (plus the HOLD2
preservation copy above). Transient `__pycache__` / `.pytest_cache` created by my runs were removed; final tree re-verified
clean.
**Checked:** 2026-11-01 **16:55–17:13Z** (11:55–12:13 ET). Timestamps below are each run's own `date -u`, not reconstructed.

---

## VERDICT: **PASS (sign with conditions)**

The five HOLD-02 closures reproduce on the shipped tree: the widened digest is `32dd2bea…`, v4 and `live/config.json`
are untouched, `live/exec` is git-clean, 75 tests pass; all five enumerated deny-neutering shapes fail closed; the
2026-10-26 claim is withdrawn and its artefact deleted; a missing ledger exits 2 with `ledger_missing`; the flat-day
fixture reproduces byte-identically and the G2 leg is recorded as code-path PASS / live occurrence OWED to G3; and every
C-note is fixed in code and tests. The cards-day regression is byte-identical (`1121cdbb…`, 0/0/0), six flags are true,
`problems: []`, and nothing in the package can send (`mode: dry-run`, no enable/lock/static-proof/state artefact,
0 sockets / 0 subprocess / 0 deny-path opens in the audit).

The veto lifts for the **shipped tree** subject to three signature conditions (C1–C3 below). C1 is the only adversarial
finding: the hardcoded deny minimum pins only a subset of the rule data, so a *coherently weakened* (not emptied) deny
file plus an allowlisted non-minimum import yields `all_ok: true` on that copy. A proof built over that copy is still
rejected on the canonical tree (`third_party:proof_stale`, and `deny_stale` if the package digest is forged), so this is
**not** a re-open of the prior canonical-tree acceptance hole; it is a G3 hardening condition and is labelled **R-1**.
If the Lead applies the strict test "any shape that still passes = B1-A1 open", R-1 is that shape and the verdict should
be HOLD; the shipped tree itself is bound by `32dd2bea…` and is not weakened.

| # | HOLD-02 item | Verdict | Decisive evidence |
|---|---|---|---|
| 1 | Digest `32dd2bea…`, scope includes `deny.json` + `exec_config.json`; v4 `ea0ec96e…` untouched; config mtime unchanged; `live/exec` git-clean; 75 tests | **PASS** | independent digest over 18 `*.py` + `deny.json` + `exec_config.json` = `32dd2bea…`; sensitivity proven for each extra; v4 hash and config mtime unchanged; `git status` empty; **75 passed** (16:55:44Z, 17:06:28Z, 17:09:42Z) |
| 2 | B1-A1 — `deny_sha256` pinned (`third_party:deny_stale`); hardcoded `MIN_REQUIRED`/`MIN_COUNTS` checked before file metadata; every neutering shape fails; neutered proof rejected on canonical tree | **PASS** (five required closures) | `deny_stale` / `proof_stale` reproduced; clean all-true; lists emptied, metadata+lists emptied, `{}`, coherently neutered all `rules_sane:false` / `all_ok:false`; proof over a neutered full copy rejected. **Residual R-1 open** (partial weakening; not canonical-tree acceptance) |
| 3 | B2-E1 — 2026-10-26 claim withdrawn, artefact deleted, nothing in evidence asserts it as flat | **PASS** (evidence) / doc residual | build evidence §2 and README withdraw it; no `dryrun-flat-2026-10-26` anywhere; 26 Oct log/ledger show 2 pulse BUY signals + 2 fills, not flat. Residual stale docstring in `g2_dryrun.py:10` ("pre-engine") |
| 4 | B2-E2 — missing `--ledger` exits 2 with `{"error":"ledger_missing"}`; `--allow-missing-ledger` fixture-only and documented; tests; README flat command | **PASS** | exact JSON + exit 2 reproduced; flag run `pass_empty:true`, `ledger.exists:false`; existing 0-byte fixture `exists:true`; `test_g2_harness.py` has all three; README §Commands has the flat command |
| 5 | B2-E3 — Lead waiver with three conditions recorded; G2 flat-day leg code-path PASS / live OWED G3; fixture PASS-empty, zero head, 0/0/0, byte-identical, `ledger exists:true` | **PASS** (substance) / **UNVERIFIED** (independent verbatim provenance) | build evidence §4 + README record conditions (a)(b)(c) and the G2/G3 split; flat rerun byte-identical to stored artefact; all fields reproduced. No independently signed Lead quote exists in the record (only the maker's attribution); the fixture is labelled by filename/0 bytes, not by `flat_day.note` |
| 6 | C-notes — `check_replace` real/new refs + real original; `must_disable` stops loop (venue once; rest `disabled_after_venue_error`; raw exception not a rail reason); R8 no literal fallback; open-count wording; shared audit gate documented | **PASS** | independent cases: all missing/empty/same-ref/over-count refused, fresh allowed; Failing venue: 1 call, refused 3, `venue_disabled:1` + `disabled_after_venue_error:2`, raw name only in `venue_error`; R8 missing key → `config_missing:feed_stale_s`, 61s→`stale_feed`, 59s→clean; README documents the shared `ran`/`canary_ok` gate |
| 7 | Regression — six flags true, `problems: []`, cards-day byte-identical (`1121cdbb…`, compare 0/0/0), no enable/transport/network | **PASS** | cards rerun exit 0 and `diff -rq` IDENTICAL; chain independently recomputed; compare 1 allowed / 0 missing / 0 extra / 0 mismatch; actual proof accepted by guard; no enable/lock/static-proof/state; audit 74 opens / 0 sockets / 0 subprocess / 0 deny opens |

---

## 1. Package integrity, digest, design, config, git, tests — PASS

```
$ cd ~/Work/projects/openbell/live/exec && date -u +%FT%TZ
2026-11-01T16:55:35Z
$ python3 - <<'EOF'            # independent: sorted *.py, then deny.json, then exec_config.json
... sha256(name bytes + file bytes) ...
32dd2bea4bb0296b4d24f8e595efa8b197cd2bec699bcb6f418b0fc8e13384f9
files 20
EOF
```

The 20 files are 18 `*.py` + `deny.json` + `exec_config.json`. Each per-file hash reproduces
`exec-g2a/package-files.txt` exactly (`diff` of recomputed vs stored = empty, 17:04:27Z). The extra-file binding is real,
not incidental: changing only `deny.json` changes `package_digest`; changing only `exec_config.json` changes it too
(`digest_changes_when_deny_changes: true`, `digest_changes_when_exec_config_changes: true`, 16:57:52Z).

```
$ sha256sum board/reports/team-c-autonomous-path-design-v4.md
ea0ec96e1402fd4427706371428fa9cd3316d9c692455a98f87f1fca0e883c1e       # 17:09:42Z
$ stat -c '%y' ~/Work/projects/openbell/live/config.json
2026-10-27 18:43:34.427401225 -0400                                     # = 22:43:34Z, unchanged
$ git -C ~/Work/projects/openbell status --porcelain live/exec live/config.json live/engine.py
<empty>                                                                 # 0 lines (17:09:51Z)
$ git -C ~/Work/projects/openbell log --format='%h %p %s' -3
0893497 436e5f7 live/exec: G2 HOLD-02 fixes — …        (HEAD)
436e5f7 1916d48 live/exec: README v2 — …
1916d48 d555b50 live/exec: G2 build v2 — …
```

`git remote -v` is empty (local only, not pushed); 0893497 changes only `live/exec/*` (12 files, no engine/config/v4
write). The rest of the repo has the pre-existing 9-file dirt noted by the prior checker (`live/dash*`, `RUNBOOK.md`,
fixtures, `test_operator_brief.py`); `live/exec`, `live/config.json`, `live/engine.py` are clean.

```
$ PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q
75 passed in 0.82s                                                      # 17:06:28Z
$ PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q -p no:cacheprovider
75 passed in 0.84s                                                      # 17:09:42Z
```

No skips/xfails. The exact claimed command passes. (The `.pytest_cache` it creates was removed; final digest re-checked
`32dd2bea…` and `git status` clean at 17:09:51Z.)

**Minor observation (R-2).** `proof.package_digest()` / `guard.package_sha256()` do `if f.exists()` for the two extras.
A *deleted* `exec_config.json` is therefore not bound by the digest. `deny.json` deletion is separately caught by
`guard.verify_static_proof` (`third_party:deny_missing`); `exec_config.json` deletion is not explicitly checked. A genuine
`build_proof` over that tree sets the three runtime flags false (audit child cannot read the config), so a genuine proof
is still refused; only a forged all-true proof is accepted (out of the genuine-proof threat model). Cheap hardening:
require both files to exist in `package_digest`/`verify_static_proof`.

## 2. B1-A1 — the deny anchor: five required closures PASS; residual R-1

**Code confirmed.** `guard.verify_static_proof` checks the six flags, compares `package_sha256` to the tree
(`third_party:proof_stale`) and compares `deny_sha256` to the actual `deny.json` (`third_party:deny_stale`)
(`guard.py:142–160`). `proof.MIN_REQUIRED` / `proof.MIN_COUNTS` are module constants; `rules_sane` checks them **first**,
then the file's own `required_members` / `min_counts` (`proof.py:32–39, 65–84`). The clean record:
`rules_sane/imports_ok/ast_deny_ok/canary_ok/filesystem_ok/network_ok = true`, `all_ok: true`, `problems: []`,
`audit {ran:true, canary_ok:true, opens:74, sockets:0, subprocess:0, deny_opens:[]}` (17:09:42Z).

**Every required shape re-run on full `/tmp/g2check3` copies (16:57:16Z).** Each copy contains all 18 `.py` + both extras,
so only the named mutation differs:

| `deny.json` shape | `rules_sane` | other flags | `all_ok` | notes |
|---|---|---|---|---|
| clean copy | **true** | all true | **true** | 74 opens, 0/0/0 |
| lists emptied, metadata kept | **false** | imports true; audit child unparsable → canary/fs/net false | **false** | hardcoded minimum + file metadata both fire |
| metadata + lists emptied (old test shape) | **false** | — | **false** | no longer vacuous |
| `{}` | **false** | imports false; audit unparsable → three false | **false** | **no crash**; `build_proof` returns a record |
| coherently neutered (all four lists empty, real imports allowlisted, no metadata) | **false** | imports true, canary false | **false** | rules_sane problems list all four `hardcoded_missing` groups first |
| proof built over the coherently neutered full copy, verified on canonical tree | — | — | **REJECTED** | `third_party:rules_sane`, `third_party:canary_ok` (flags checked before digest); with an all-true forged record carrying the neutered package digest: `third_party:proof_stale`; with canonical package digest but neutered `deny_sha256`: `third_party:deny_stale` |

The actual stored proof is accepted for the shipped tree, and the two pins have teeth independently:
`tamper canary_ok → third_party:canary_ok`; `tamper package_sha256 → third_party:proof_stale`;
`tamper deny_sha256 → third_party:deny_stale` (17:06:00Z). The enumerated closures are therefore met.

**Residual R-1 — partial (non-minimum) weakening still passes the six flags.** The hardcoded minimum is a *subset*:
required sizes are 5 substrings / 4 methods / 5 import roots / 5 path fragments against a 18 / 4 / 24 / 11 rule file
(min counts 15 / 4 / 15 / 8). I removed **one non-minimum** root (`urllib`) from `deny_import_roots`, added it to
`safe_import_roots`, and planted `import urllib.request` in `executor.py` on a full copy:

```
flags {rules_sane:true, imports_ok:true, ast_deny_ok:true, canary_ok:true, filesystem_ok:true, network_ok:true}
all_ok: true   problems: []   audit {opens:130, sockets:0, subprocess:0, deny_opens:[]}
package_sha256 955ae78b…   (canonical 32dd2bea…)
```

The shipped tree is not affected: the weakened proof on the canonical tree is rejected
`['third_party:proof_stale']`, and forging the canonical package digest still rejects
`['third_party:deny_stale']` (17:12:43Z). So this is **not** the prior "guard accepts a neutered proof for the canonical
tree" hole; it is a limitation of the minimum-set approach: rules outside the hardcoded minimum remain data-editable, so
a *coherently weakened* (not emptied) file can produce an all-true proof on its own tree. Before G3, either hardcode the
full deny set or pin `deny.json` to a constant independent of the proof record, and add this shape as a planted negative.
If the Lead's standard is "any shape that still passes = B1-A1 open", this is the shape that meets it.

**Two smaller B1 observations.** (a) `test_proof.py::test_neutered_proof_rejected_on_canonical_tree` copies only the 10
`RUNTIME` files, not the full package, so its `package_sha256 != clean` assertion would pass even if `deny.json` were not
in the digest (the copy is missing 10 `test_*.py` files). `test_guard.py::test_static_proof_deny_digest_checked` and
`test_package_digest_covers_deny_and_config` are the strong tests; the weak one should be strengthened (C4).
(b) `proof.build_proof` over a *missing* `deny.json` raises `FileNotFoundError`; over a malformed non-object
(`[]`, `null`, `"x"`) raises `AttributeError`. Only `{}` is the required no-crash shape and it passes. The evidence's
"missing/malformed … fails closed" wording is broader than the code; the guard still refuses a missing deny
(`deny_missing`), so this is a wording/robustness issue, not a pass.

## 3. B2-E1 — the 2026-10-26 claim is withdrawn; artefact deleted — PASS (doc residual)

Build evidence §2 now says plainly: "The earlier evidence called 26 Oct a 'real pre-engine day'. **It is not.** … two
`SHADOW OpenBell BUY 2 MES` cards … two shadow fills and a `-$17.50` shadow exit … no `live/ledger/2026-10-26.jsonl`
… The artefact `exec-g2a/dryrun-flat-2026-10-26/` has been **deleted**; the only flat-shape evidence kept is the
**labelled 0-byte fixture**." README §"G2 flat-day evidence" repeats that 26 Oct is **pre-ledger** (not pre-engine),
carried pulse signals/fills and is **not flat**; the earlier claim is withdrawn.

I confirmed the underlying fact independently (16:58:28Z):

```
$ grep -c '"event": "signal"' live/ledger/selftest-pulse-2026-10-26.jsonl
2
$ grep -n "SHADOW OpenBell BUY\|shadow fill" live/logs/engine-2026-10-26.log | head
4: 2026-10-26 16:22:01 … SHADOW OpenBell BUY 2 MES … (signal on the 16:21 ET bar)
5: 2026-10-26 16:23:00 … shadow fill +1 @ 7714.25 …
10: 2026-10-26 16:34:01 … SHADOW OpenBell BUY 2 MES … (signal on the 16:33 ET bar)
11: 2026-10-26 16:35:01 … shadow fill +1 @ 7715.00 …
$ ls live/ledger/2026-10-26.jsonl
No such file or directory
$ find board -iname '*2026-10-26*'
<empty>                                  # the dryrun-flat-2026-10-26 artefact is gone
```

Nothing in the build evidence or `exec-g2a/` asserts 26 Oct as flat; the only hits are the withdrawal text itself. **One
stale code docstring remains:** `g2_dryrun.py:10` still says "(27-30 Oct each carry >=1 signal; 2026-10-26 is a
pre-engine day)". That is false and contradicts §2/README; it does not assert "flat", but it repeats the exact phrase the
HOLD retired. Correct it to "pre-ledger, with pulse signals/fills" before signature (C2).

## 4. B2-E2 — a missing ledger is a data gap, not a flat day — PASS

```
$ python3 g2_dryrun.py --day 2026-10-31 --cards ../logs/cards.jsonl \
    --ledger /tmp/g2check3/does-not-exist.jsonl \
    --r11-fixture <board>/…/r11-fixture-2026-10-30.json --out /tmp/g2check3/flat-missing
{"error": "ledger_missing", "day": "2026-10-31", "path": "/tmp/g2check3/does-not-exist.jsonl",
 "hint": "a missing ledger is a data gap, not a flat day; use --allow-missing-ledger only for a labelled fixture"}
exit=2                                                              # 16:58:28Z
```

With `--allow-missing-ledger` the run exits 0 and records the absence instead of hiding it:
`flat_day.pass_empty:true`, `chain_head 0×64`, `zero_hash_chain_head:true`, `ledger {exists:false,
allow_missing_ledger:true, path:…}`, `compare_vs_cards {clean:true, 0/0/0}`, `rerun_identical:true`,
`intents_bytes_identical:true`, `chain_verified:true`. The labelled existing 0-byte fixture gives
`ledger {exists:true, allow_missing_ledger:false}` (stored flat artefact; re-run §5). The tests are in
`test_g2_harness.py` (missing→exit 2; flag→PASS-empty + `exists:false`; existing 0-byte→PASS-empty) and all 75 pass.
README §Commands now contains the flat command (`--ledger <board>/…/flat-ledger-fixture-2026-10-31.jsonl`,
`--out …/dryrun-flat-2026-10-31`) and the missing-ledger rule.

**Note (C6).** The flag is *documented* fixture-only (help text and README) but not *enforced*: it can be passed for any
day/path. The CLI is the evidence path and the README says "only for a labelled fixture"; a filename/label check or an
explicit operator-discipline line would make the "only" mechanical. Direct callers of `load_jsonl`/`load_day` can still
turn a missing path into an empty list; the CLI guard is what the G2 evidence uses.

## 5. B2-E3 — flat-day waiver, bounded — PASS (substance) / UNVERIFIED (independent verbatim provenance)

Build evidence §4 records the ruling and all three conditions: "(a) the fixture is clearly labelled
(`flat-ledger-fixture-2026-10-31.jsonl`, 0 bytes; stated as a fixture in `flat_day.note`); (b) the report states plainly
that **no real zero-signal trading session exists in the record** (27–30 Oct each carry ≥1 `signal`; 26 Oct is pre-ledger
**with** pulse signals); (c) the first live flat session is a named **G3 / Monday-conditional obligation** — captured with
the same command, attached to the G3 package, **no completeness claim before it**. **G2 flat-day leg = code-path PASS /
live occurrence OWED to G3.**" README §"G2 flat-day evidence — Lead waiver, recorded" repeats the same three conditions
and the same split. The fixture run (`16:59:41Z`) reproduces the stored artefact exactly:

```
$ python3 g2_dryrun.py --day 2026-10-31 --cards ../logs/cards.jsonl \
    --ledger <board>/…/flat-ledger-fixture-2026-10-31.jsonl \
    --r11-fixture <board>/…/r11-fixture-2026-10-30.json --out /tmp/g2check3/flat-rerun
exit=0
$ diff -rq /tmp/g2check3/flat-rerun <board>/…/dryrun-flat-2026-10-31
IDENTICAL
flat_day: pass_empty true, statement "PASS-empty — no signals today, no completeness claim", signals 0, intents 0
chain: 000…000 (64×0), chain_verified true, rerun_identical true, intents_bytes_identical true
compare: allowed_intents 0, clean true, missing/extra/mismatch []   counts: cards 0 / intents 0 / allowed 0 / refused 0
ledger: {exists: true, allow_missing_ledger: false, path: …/flat-ledger-fixture-2026-10-31.jsonl}
```

Condition (b) and (c) are met on the record; the fixture is labelled by its filename (`flat-ledger-fixture-…`, 0 bytes)
and by README. **Two provenance defects:** (i) `flat_day.note` itself does **not** contain the word "fixture" (it says
"zero-signal shape (design §F(v)); the same command will capture the first live flat session"), so the parenthetical
"stated as a fixture in `flat_day.note`" is inaccurate; (ii) the waiver is recorded only as the maker's attribution
("Ruling (Lead, 2026-11-01): WAIVE …") — there is no independently signed Lead quote, message id, or countersignature in
the record. The substance is right and the G2 leg is correctly bounded; the Lead should countersign/attach the exact
waiver text (or confirm it here) before the evidence is signed (C3). That is why the verbatim-provenance sub-item is
**UNVERIFIED**, not FAIL.

## 6. C-notes — PASS

- **`check_replace`** (`rails.py:194–211`) now requires a non-empty `replaces`, a non-empty **new** `decision_ref`, a
  non-empty **original** ref, and `new != original`; it also refuses `replace_count > max_replaces_per_intent`.
  Independent cases (17:00:28Z): no ref → False; empty ref → False; `None` original → False; `{}` original → False; same
  ref → False; fresh ref → True; no `replaces` → False; count 2 vs cap 1 → False. Tests
  `test_check_replace_helper` and `test_check_replace_requires_real_decision_refs` pass.
- **`must_disable` stops the loop** (`executor.py:62–73`): the `disabled` gate is at the top of the card loop; after one
  fatal venue error the remaining cards are refused `disabled_after_venue_error` with **no further venue call**. My
  independent `Failing` venue through `run_dry`: `disabled true, allowed 0, refused 3, refusal_counts
  {venue_disabled:1, disabled_after_venue_error:2}`, venue calls **1**, card-0 `venue_error: "SessionMaterialError"`,
  reasons `[["venue_disabled"],["disabled_after_venue_error"],["disabled_after_venue_error"]]`, chain verifies. The raw
  exception name is kept only in `venue_error`, not counted as a rail reason. `test_venue_fatal_error_disables` asserts
  the same (calls == 1, counts, no raw name in `refusal_counts`).
- **R8 has no literal fallback** (`rails.py:133–137`): `"feed_stale_s" not in rails` → `config_missing:feed_stale_s`;
  with the key present, age 61 → `stale_feed`, age 59 → no `stale_feed`. Re-run at 17:00:28Z. Test
  `test_r8_threshold_comes_from_config_not_literal` passes; no literal `60` fallback remains.
- **Open-count wording**: README "Honest limits" says `opens` varies with bytecode caches (73–74 observed) and the gated
  numbers are 0 sockets / 0 subprocess / 0 deny-path opens; build evidence §5 says the same. The recorded run says
  `opens:74`; that is a run number, not a gate. PASS.
- **Shared audit gate documented**: README says the three runtime flags share one audit child by construction, a failed
  audit run conservatively falsifies them, and `ran`/`canary_ok` are reported separately from the event counts.
  `proof_audit.py` prints `ran`, `canary_ok`, `problems`, `opens`, `sockets`, `subprocess`, `deny_opens` separately;
  `build_proof` gates `canary_ok` on `ran and audit.canary_ok`, `filesystem_ok` on `ran and not deny_opens`,
  `network_ok` on `ran and counts==0`. The `{}`/partial-empty shapes show the conservative direction: audit failure →
  all three false, never a false pass. **Stale docstring:** `proof.py:2–3` still says "Each flag is computed by its OWN
  scan/run … so one check cannot ride on another's result", which is not exact for the three runtime flags; the README is
  correct, the module docstring should match it (C2).

## 7. Regression — PASS

Cards day 2026-10-30, real inputs, re-run 17:00:58Z:

```
$ python3 g2_dryrun.py --day 2026-10-30 --cards ../logs/cards.jsonl \
    --ledger ../ledger/2026-10-30.jsonl --r11-fixture <board>/…/r11-fixture-2026-10-30.json \
    --out /tmp/g2check3/cards-rerun
exit=0
$ diff -rq /tmp/g2check3/cards-rerun <board>/…/dryrun-2026-10-30
IDENTICAL
chain_head 1121cdbb04b61ece23d5ab65d6058e582766222627fbdc833a52e090b0fbf492
rerun_identical true, intents_bytes_identical true, chain_verified true
compare_vs_cards {allowed_intents:1, entry_cards:1, missing:[], extra:[], mismatch:[], clean:true}
summary {cards:1, intents:1, allowed:1, refused:0, disabled:false}
reconcile {clean:true, fills:0, intents:1, decisions:1, unmatched_*:[]}
```

I recomputed the chain independently (`json.dumps(sort_keys=True, separators=(",", ":"))` minus `record_hash`, chained
from `0`×64): `chain_ok True`, head `1121cdbb…`, 1 record. Six flags true, `problems: []`, proof accepted by
`guard.verify_static_proof` (17:09:42Z). No enable/lock/static-proof artefact and no `state/` dir in the package;
`exec_config.mode` = `dry-run`; `find` for `enabled.json`/`trader.lock`/`static_proof.json` is empty; the six runtime
modules have no socket/transport import (`CDPVenue` appears only in the canary and session-material tests); audit
0 sockets / 0 subprocess / 0 deny-path opens. S-3 unchanged: `daily_consistency_cap_usd:1000`,
`max_replaces_per_intent:1`, `max_cancel_replace_per_10min:3`; `guard.assert_desk_config(real live/config.json, CFG)` →
`[]`; shipped-config `guard.startup` refuses `['disabled:no_enable_artefact','third_party:static_proof_missing',
'not_flat']` and creates no lock.

---

## 8. Residual findings and signature conditions

| id | Finding | Severity / effect | Action |
|---|---|---|---|
| **R-1** | Hardcoded minimum pins only a subset: remove non-minimum `urllib` from `deny_import_roots`, allowlist it, plant `import urllib.request` → six flags true, `all_ok:true`, `problems:[]` on that copy. Proof is `proof_stale` on the canonical tree (and `deny_stale` if its package digest is forged). | G3 hardening; not a shipped-tree acceptance hole. The strict "any shape that passes" test is met by this shape. | Before G3: hardcode the full deny set or pin a constant independent of the proof record; add this negative to `planted-negatives.json`; add a test that only `deny.json` changes the digest. |
| **R-2** | `package_digest` skips a missing `exec_config.json`; guard has no explicit `exec_config_missing`. A genuine proof over that tree is still refused (runtime flags false); only a forged all-true proof is accepted. | Low; hardening. | Require both extras to exist in `package_digest`/`verify_static_proof`. |
| **R-3** | `g2_dryrun.py:10` still calls 2026-10-26 a "pre-engine day" (false; §2/README say pre-ledger with pulse signals). `proof.py:2–3` still claims every flag is its own run (the three runtime flags share the audit child). | Documentation; contradicts corrected evidence. | Correct both docstrings before signature (C2). |
| **R-4** | Waiver is recorded only as maker attribution ("Ruling (Lead …)"), with no signed Lead quote/message id; `flat_day.note` does not say "fixture" though build evidence claims it does. | Provenance; the three conditions and G2/G3 split are otherwise recorded. | Lead countersigns/attaches the exact waiver; correct the `flat_day.note` claim or add "fixture" to the note (C3). |
| **R-5** | `test_proof.py::test_neutered_proof_rejected_on_canonical_tree` copies only 10 of 20 files, so its digest-difference assertion passes without the deny binding. | Test quality; `test_guard.py` covers the real pins. | Copy the full package (or assert `package_digest` changes with deny alone) (C4). |
| **R-6** | `exec-g2a/static-proof.json` has `"generated_at": "2026-11-01T17:1xZ"` (placeholder, not ISO); build evidence header uses "~17:1xZ". | Evidence hygiene; no field the guard reads. | Emit a real `date -u` timestamp; use exact times (C5). |
| **R-7** | `--allow-missing-ledger` is documented fixture-only but not enforced; direct `load_jsonl`/`load_day` callers can still turn a missing path into flat. | Operator discipline; CLI evidence path is guarded. | Filename/label check or explicit operator instruction (C6). |

**Signature conditions.** C1: Lead adjudicates R-1 and either accepts the minimum-set limitation for G2 (with the
hardening before G3) or converts this verdict to HOLD. C2: correct the two stale docstrings. C3: Lead countersigns/attaches
the waiver verbatim and the fixture-note claim is corrected. C4–C6 are housekeeping, not G2 blockers.

**Why not HOLD outright:** every substantive HOLD-02 closure that the prior verdict named is now real and reproduced on
the shipped tree (the waiver's independent verbatim provenance remains UNVERIFIED, C3); the send boundary is unchanged
(dry-run, no enable/transport, 0/0/0); the digest binds the exact tree the evidence describes; and the one adversarial
shape (R-1) cannot be accepted for that tree — it is rejected `proof_stale`/`deny_stale`.
**Why not FAIL:** no required closure is false for the shipped tree, and the two prior HOLDs' specific acceptance hole is
closed. The remaining items are documentation, provenance, test-strength, and a G3 hardening gap.

## 9. Footprint / reproducibility

- Read-only on the package, engine and `live/config.json`; all mutations were `/tmp/g2check3` copies. The only writes to
  the workspace are this report and the HOLD2 preservation copy.
- Transient caches created by my runs (`__pycache__`, `.pytest_cache`) were removed; final `git status
  live/exec live/config.json live/engine.py` = empty; final digest `32dd2bea…`; `live/config.json` mtime still
  `2026-10-27 18:43:34.427401225 -0400`; no `state/`, lock, `enabled.json` or `static_proof.json` in the package.
- No live action, order, wallet, credential, spend or network send. The audit's socket/subprocess counters are 0/0 in
  every clean run; no probe opened a socket, connected, or sent a packet.
- Commands were run from `~/Work/projects/openbell/live/exec` unless a path is shown; outputs above are the run's own,
  with `date -u` timestamps. Reproduction set: `PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q`;
  `python3 -c "import proof;print(proof.build_proof('.'))"`; the cards and flat commands in README §Commands; my shape
  scripts `/tmp/g2check3/{shapes.py,forge.py,weaken.py}` and outputs `/tmp/g2check3/{shapes.out,flat-rerun,cards-rerun}`.

— checker seat, Team C G2 (fresh context). **Verdict: PASS (sign with conditions)** — five HOLD-02 closures reproduced;
R-1 (partial deny weakening) is the one adversarial residual, rejected on the canonical tree but to be hardened before G3;
R-3/R-4 are signature conditions.
