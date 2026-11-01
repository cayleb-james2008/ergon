# TEAM C — G2 CHECKER VERDICT v2 (fresh seat) — 1 Nov 2026

**Seat:** fresh-context, independent Tier S checker with VETO. Did not build anything here; reports to the Lead only.
**Subject:** `~/Work/projects/openbell/live/exec/` at commit **436e5f7** (build `1916d48`), branch
`research/kronos-stage1-verdict` (local; no remote configured), against design v4 `ea0ec96e…` under
**G1-SIG-2026-11-01-TS-03**.
**Prior verdict kept intact:** the HOLD of 11:19 ET is archived byte-identically at
`board/reports/team-c-g2-checker-verdict-20261101-HOLD.md` (sha256 `4c4db6d1309e87fe4479d11149dc5697afcc72eda3376a29b49552c893b5fa20`);
this file is the new verdict at the canonical path.
**Fences observed:** read-only on the package, engine and config; no live action, order, wallet, credential, spend or
network send. All mutations ran in `/tmp` copies or in memory. Only write = this report (plus the archived copy above).
Two cache dirs I created in `live/exec` (`__pycache__`, `.pytest_cache`) were removed; the tree is back to `git status` clean
for `live/exec`, `live/config.json`, `live/engine.py`. No `state/` dir, no lock, no `enabled.json` created.
**Checked:** 2026-11-01 11:43–11:52 ET (16:43–16:52Z). Consolidated re-run: 16:49:24–16:49:25Z.

## VERDICT: **HOLD**

The B1 *mechanism* the prior HOLD named is now real, and the B2 *code path* now exists and runs. Both are reproduced
below on the shipped tree. The veto does **not** lift because two claims in the evidence record are still false or
unsupported, and one of them is the only "real day" flat evidence:

1. **B2 — the 2026-10-26 "real pre-engine day" is not a flat day and not pre-engine.** The engine log for that day records
   a running `pulse` engine with **two BUY signals** (16:21 and 16:33 ET bars), two shadow fills and a `-$17.50` shadow
   exit. The harness's `PASS-empty` came from the **path it was given not existing** — `load_jsonl()` returns `[]` for a
   missing path — not from a zero-signal day; a 10-26 ledger *does* exist (`live/ledger/selftest-pulse-2026-10-26.jsonl`,
   2 `signal` rows), just not under the day-keyed name the run used. A missing ledger is silently a flat day (I reproduced that on a day I chose,
   with a non-existent path: exit 0, `pass_empty: true`). One valid flat test exists (the labelled 0-byte fixture); the
   prior B2's "one real flat day" is still not met.
2. **B1 — the deny "data anchor" is self-referential and not bound.** Emptying the *lists* while keeping
   `required_members`/`min_counts` does fail `rules_sane` (the intended case). But emptying the metadata too — the exact
   shape in `test_proof.py` — leaves `rules_sane: true` and only `imports_ok` fails; `{}` raises `KeyError`; and a coherently
   neutered `deny.json` (empty rules + the real imports allowlisted) yields **all six flags true, `all_ok: true`**, and
   `guard.verify_static_proof` **accepts** that proof for the canonical tree, because `package_sha256` covers sorted `*.py`
   only and `deny_sha256` is recorded but never checked. So "emptying it fails `rules_sane`" is not true in general, and
   the proof can be regenerated over a package with no deny rules.

Neither is a FAIL of the send boundary: nothing in the package can send (mode `dry-run`, no enable artefact, no CDP
transport, 0 sockets/0 subprocess under audit). The remaining work is cheap in code (one evidence paragraph, one existence check,
one digest binding); the real flat-day capture is an event, not a code change. The prior HOLD's "signing now would sign a false evidence record" still applies to §3 of the build
evidence, not to the code.

| # | Item | Verdict | Decisive evidence |
|---|---|---|---|
| 1 | Digest `2a184f26…`, v4 untouched, config mtime, git status, 65 tests | **PASS** | digest reproduced over sorted `*.py` name+bytes; v4 `ea0ec96e…`; config `2026-10-27 18:43:34 -0400`; `git status` empty; 65 passed |
| 2 | B1 six flags real, venue scanned, audit child, planted negatives | **PASS** (mechanism) | clean all-true, 0/0/0, 0 deny opens; the recorded imports/cookie/canary negatives flip their target flags; canary/filesystem/network each have independent teeth |
| 3 | B1 "deny.json is the data anchor (emptying fails `rules_sane`)" | **FAIL** | metadata-removed empty → `rules_sane: true`; neutered deny → `all_ok: true`; guard accepts it (deny digest unchecked) |
| 4 | B2 flat/no-signal path shape (0 intents, exact wording, zero head, 0/0/0, byte-identical) | **PASS** | fixture run exit 0; `pass_empty: true`; statement exact; `0`×64 head; rerun/bytes/chain true; compare clean |
| 5 | B2 "real pre-engine day 2026-10-26" as flat evidence | **FAIL** | engine log + `selftest-pulse-2026-10-26.jsonl` have 2 pulse BUY signals and fills; the day-keyed ledger used is absent; missing path → `PASS-empty` (also reproduced on 10-31) |
| 6 | C1 R19 fresh-ref + `check_replace()` + reconcile wired/tested | **PASS** (one predicate hole, note) | same ref refused, fresh allowed, missing state key refused; reconcile clean/orphan and in the summary; `check_replace` allows a missing/empty `decision_ref` |
| 7 | C3 `guard.startup` no KeyError, `package_dir` optional/in config, lock only after verdict | **PASS** | shipped config → named `GuardError` (`disabled:…`, `third_party:…`, `not_flat`), no lock file; happy path takes lock; second executor refused |
| 8 | C4 `must_disable` exercised by the dry-run venue-error path | **PASS** (exercise) / note | fatal venue → `disabled: true`, 3 refused; but the loop still calls the venue 3/3 times — `disabled` does not stop it |
| 9 | S-3 three keys stay; `assert_desk_config()` on the real config; 10 drifts; R8 from config | **PASS** / note | keys `1000/1/3`; real config → `[]`; 10/10 mutations and 10/10 missing keys caught; R8 reads `feed_stale_s` — but a literal `60` fallback remains |
| 10 | Regression: cards day chain `1121cdbb…`, compare 0/0/0, no transport/enable/network, R1–R19 | **PASS** | all 7 artefacts byte-identical to the maker's; chain independently recomputed; 0 sockets/subprocess; 19 rails present and tested |

---

## 1. Package integrity, digest, design, config, tests — PASS

```
$ cd ~/Work/projects/openbell/live/exec && python3 - <<'EOF'
> import hashlib, glob
> h=hashlib.sha256()
> for f in sorted(glob.glob('*.py')):
>     h.update(f.encode()); h.update(open(f,'rb').read())
> print(h.hexdigest())
> EOF
2a184f266296731920c711d0f4bf58725c3d24a1dd436c7ccf0d64d0da8805eb      # 2026-11-01T16:49:24Z
```
Reproduced a second time after the cache cleanup; equals the claimed `2a184f26…`. **Digest scope:** sha256 over
**sorted `*.py`, filename bytes then file bytes** — 17 `.py` files, tests included; **`deny.json`, `exec_config.json`,
`README.md` and the evidence files are not covered** (this matters in §2.3). `deny.json` sha256 is separately
`9b82d26815b7720d0d7ec017270c3698bc50c07f158fe6d0622f5a8f3fd3c45b` (matches the proof record).

```
$ sha256sum board/reports/team-c-autonomous-path-design-v4.md
ea0ec96e1402fd4427706371428fa9cd3316d9c692455a98f87f1fca0e883c1e   # mtime 2026-11-01 10:04:00 -0500
$ stat -c '%y' ~/Work/projects/openbell/live/config.json
2026-10-27 18:43:34.427401225 -0400                                   # = 22:43:34Z, unchanged
$ git -C ~/Work/projects/openbell status --porcelain live/exec live/config.json live/engine.py
<empty>                                                               # exit 0
$ git -C ~/Work/projects/openbell log --oneline -2
436e5f7 live/exec: README v2 — G2 HOLD-01 fixes, honest limits …
1916d48 live/exec: G2 build v2 — real static proof (B1) + flat-day path (B2)
```
Branch is local; `git remote` has no `origin` (so "not pushed" is true by construction). Only the pre-existing dirt
(`live/dash*`, `RUNBOOK.md`, fixtures, `test_operator_brief.py`) is modified/untracked; `live/exec` itself is clean and
tracked at `436e5f7`.

```
$ PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q -p no:cacheprovider
65 passed in 0.45s      # 16:49:24Z; also 0.47–0.48s on two earlier runs
```
No skips, no xfails. 65 = the claimed count (was 56 at the prior HOLD).

## 2. B1 — the six flags are real; the anchor is not anchored

### 2.1 Clean tree — reproduced exactly

```
$ python3 -c "import json,proof;p=proof.build_proof('.');print(json.dumps(...))"
{"package_sha256":"2a184f26…","deny_sha256":"9b82d268…",
 "flags":{"rules_sane":true,"imports_ok":true,"ast_deny_ok":true,
          "canary_ok":true,"filesystem_ok":true,"network_ok":true},
 "all_ok":true,
 "audit":{"ok":true,"opens":74,"sockets":0,"subprocess":0,"deny_opens":[]},
 "problems":[]}                                                     # 16:44Z
```
One consolidated re-run at 16:49:25Z reported `opens: 73`; every other run, including cache-clean re-runs at
16:52Z, reported `74`. The open count is context, not a gate, and is not perfectly invariant (73–74); the gated
numbers were `0` sockets / `0` subprocess / `0` deny opens in every run. The audit child is a real separate
process (`subprocess.run([sys.executable, proof_audit.py], cwd=root, env={PYTHONDONTWRITEBYTECODE…})`), with
`sys.addaudithook` installed **before** the package imports; it drives `venue.canary()` and a real `executor.run_dry`
inside that process. This is not the prior "one scan wearing three names".

### 2.2 The planted negatives — re-run by me on `/tmp` copies (16:45–16:46Z)

| Copy (only that defect added) | flags that flipped | all_ok | notes |
|---|---|---|---|
| clean | none | true | 74 opens, 0/0/0 |
| `import smtplib` + `import telegram` in `executor.py` | `imports_ok: false` — **and** `canary_ok/filesystem_ok/network_ok: false` | false | child died importing `telegram` → `audit:unparsable_audit`; the target flag did flip |
| real `open('/tmp/g2/.config/chromium/Default/Cookies')` in `venue.py` | `ast_deny_ok: false`, `filesystem_ok: false` | false | `venue.py:deny_literal:chromium/cookies/profile`; audit saw the open |
| venue bypass emitting `()=>document.cookie` | `canary_ok: false` (+ `ast_deny_ok: false`) | false | `canary:session-like value returned: SENTINEL-COOKIE`; transport touched |
| emptied `deny.json` (in the evidence table, **not** in `planted-negatives.json`) | depends on the emptied shape — see 2.3 | false / **true** if coherently neutered | the `test_proof.py` shape flips `imports_ok`, not `rules_sane` |

I also separated the runtime flags with probes the maker did **not** record, to test independence rather than just
"the defect flips something":

| Probe (runtime-constructed, so the text scan cannot see it) | flags |
|---|---|
| `read_positions_text` returns `self._t.evaluate("()=>document." + "cookie")` | `ast_deny_ok: true`, `canary_ok: false` (+ sentinel) — **canary has independent teeth** |
| `open('/tmp/g2/.config/chrom' + 'ium/Default/Coo' + 'kies')` in `venue.py` | `ast_deny_ok: true`, `filesystem_ok: false`, canary/network true — **filesystem has independent teeth** |
| `__import__('sub'+'process').run(['/bin/true'])` in `venue.py` | `network_ok: false` alone (`subprocess: 1`) |
| `__import__('so'+'cket').socket()` then `.close()` (no connect, no packet) | `network_ok: false` alone (`sockets: 1`) |
| `open('…Cookies')` literal in `venue.py` (maker's shape) | `ast_deny_ok: false` **and** `filesystem_ok: false` |

**Independence finding (qualification, not a safety failure).** `rules_sane`, `imports_ok` and `ast_deny_ok` have
genuinely separate computations. The last three do **not**: `canary_ok = audit.ok and not problems`,
`filesystem_ok = audit.ok and not deny_opens`, `network_ok = audit.ok and sockets==0 and subprocess==0`. They share
one child run and the same `ok` gate (`venue.canary`), so a canary failure or an audit crash also falsifies
`filesystem_ok` and `network_ok` (seen in the `smtplib` and canary rows above). The direction is conservative — it
produces extra **false** flags, never a false pass — but the claim "six independent flags / one check cannot ride on
another's result" is not exact. A cleaner shape would report the three sub-events independently of the canary
verdict. Not a G2 blocker.

### 2.3 The deny anchor — FAIL as stated

`deny.json` is data, and `rules_sane` enforces `required_members` + `min_counts` read from the **same file**. Four
shapes, run on `/tmp` copies (16:45–16:46Z):

| `deny.json` shape | `rules_sane` | `imports_ok` | `all_ok` |
|---|---|---|---|
| lists emptied, `required_members`/`min_counts` kept | **false** (`missing_required:…`, `below_min_count`) | true | false — the intended anchor works |
| **the shape in `test_proof.py`**: lists **and** metadata emptied, safe list emptied | **true** (vacuous: no required members, no minimums) | false (every import unlisted) | false — the test passes on `imports_ok`, not on the claimed flag |
| `{}` | — | — | `build_proof` raises `KeyError: 'deny_import_roots'` (no proof record, not a reported false) |
| **coherently neutered**: empty deny rules, no probes, the real import roots allowlisted, metadata empty | **true** | true | **true** — all six flags true, `problems: []` |

The last row is the decisive one. I copied the full package, replaced only `deny.json` with that permissive-but-valid
ruleset, and built the proof in the copy:
```
flags: {rules_sane:true, imports_ok:true, ast_deny_ok:true, canary_ok:true, filesystem_ok:true, network_ok:true}
all_ok: true
package_sha256: 2a184f266296731920c711d0f4bf58725c3d24a1dd436c7ccf0d64d0da8805eb  # equals the canonical tree
deny_sha256: d546e8d51a692f00a466f6e4b4f728892b5a0d942cd75491c325d8a7507acacd     # differs
$ guard.verify_static_proof('/tmp/g2check/full-neutered-proof.json', <canonical exec>)
ACCEPTED (no GuardError; returns the record with deny_sha256 d546e8…)
```
`guard.verify_static_proof` requires the six flags and `d["package_sha256"] == package_sha256(package_dir)`; it never
compares `deny_sha256` and the package digest excludes `deny.json`. So a `deny.json` replaced with a rules-free (but
well-formed) file produces a proof the startup gate accepts. At G2 the checker does re-run the proof on the shipped
tree, so the *current* record is true; but the "data trust anchor" is only anchored to itself. Cheap fixes: include
`deny.json` in `package_sha256` (or pin/verify `deny_sha256` against a constant in `guard`), and put an independent
minimum rule set in `proof.py` so removing `required_members` cannot pass. `test_proof.py::test_emptied_deny_file_flips_rules_sane`
should assert `rules_sane is False` (not "or imports_ok"), and the case should appear in `planted-negatives.json`
(it does not: that file has only clean/imports/cookie/canary).

## 3. B2 — the flat path runs; the "real day" evidence is not real

### 3.1 The code path — PASS (exactly as claimed)

```
$ python3 g2_dryrun.py --day 2026-10-31 --cards ../logs/cards.jsonl \
    --ledger …/flat-ledger-fixture-2026-10-31.jsonl \
    --r11-fixture …/r11-fixture-2026-10-30.json --out /tmp/g2check/final-flat
exit 0                                                              # 16:49:25Z
{"flat_day":{"entry_cards":0,"intents":0,"pass_empty":true,"signals":0,
  "statement":"PASS-empty — no signals today, no completeness claim", …},
 "zero_hash_chain_head":true,"chain_verified":true,"rerun_identical":true,
 "intents_bytes_identical":true,
 "compare_vs_cards":{"allowed_intents":0,"clean":true,"entry_cards":0,
                     "extra":[],"mismatch":[],"missing":[]}}
```
- 0 signals → 0 intents → `cards:0, intents:0, allowed:0, refused:0`; empty `intents.jsonl`
  (sha256 `e3b0c442…`), zero-hash chain head `000…000`; `reconcile` 0/0/0 clean; byte-identical run-a/run-b.
- Exact wording matches the code and the stored artefact (`PASS-empty — no signals today, no completeness claim`,
  U+2014); `pass_empty: true`; `compare` 0 missing / 0 extra / 0 mismatch.
- Re-run of the stored artefact: `compare.json`, both `intents.jsonl`, both `summary.json` are **byte-identical** to
  `board/probes/team-c/exec-g2a/dryrun-flat-2026-10-31/`.
- The 0-byte fixture is labelled as a fixture in its name and in the report; that is honest for a shape test.

### 3.2 2026-10-26 — **FAIL**: it was not a flat day

The stored `dryrun-flat-2026-10-26/compare.json` reproduces only if the ledger path does not exist (or is empty). It
reports `signals: 0, entry_cards: 0, pass_empty: true`. But the engine record for that day says otherwise:

```
$ grep -n "2026-10-26 16:2[0-9]\|2026-10-26 16:3[0-9]" ~/Work/projects/openbell/live/logs/engine-2026-10-26.log
2: 2026-10-26 16:20:05 … DRY card: OpenBell engine started | mode=shadow strategy=pulse(…) qty=2 MES
4: 2026-10-26 16:22:01 … DRY card: SHADOW OpenBell BUY 2 MES | … (signal on the 16:21 ET bar) …
5: 2026-10-26 16:23:00 … shadow fill +1 @ 7714.25 stop 7711.25 target 7717.25
8: 2026-10-26 16:33:21 … DRY card: OpenBell engine started | mode=shadow strategy=pulse(every=2, stop=20.0 …)
10: 2026-10-26 16:34:01 … DRY card: SHADOW OpenBell BUY 2 MES | … (signal on the 16:33 ET bar) …
11: 2026-10-26 16:35:01 … shadow fill +1 @ 7715.00 stop 7695.00 target 7735.0
```
Two pulse **BUY signals**, two shadow fills, and a `-$17.50` shadow exit at 16:55. The signals are also in a ledger
file — `live/ledger/selftest-pulse-2026-10-26.jsonl`, 6 rows:
```
$ grep -c '"event": "signal"' live/ledger/selftest-pulse-2026-10-26.jsonl
2
$ head -1 live/ledger/selftest-pulse-2026-10-26.jsonl
{"event": "signal", "side": 1, "tag": "pulse_test", "stop_pts": 3.0, "target_pts": 3.0,
 … "px": 7714.25, "bar_ts": 1793046060, "qty": 2, "mode": "shadow", "t": 1793046121.6}
$ sed -n 5p live/ledger/selftest-pulse-2026-10-26.jsonl
{"event": "exit", … "pts": -1.25, "usd_total": -17.5, "day_pnl": -17.5, "tag": "pulse_test", "why": "time" …}
```
The day-keyed `2026-10-26.jsonl` does not exist, and `logs/cards.jsonl` carries no entry card for that day, so the
harness's `0 signals` is a filename/path artefact, not the day's state. "Pre-engine" is wrong too: the engine ran (on
the pulse test) and logged/ledgered signals. This is a provenance error in §3 of the build evidence, not merely a
label quibble — it is the only "real day" offered for the flat shape.

The maker's honest limit ("no real zero-signal **trading** session exists; 27–30 Oct each carry ≥1 signal; the first
live flat session will be captured with the same command") is true and acceptable **as a disclosure**. I verified the
ledger counts: 27 Oct 2 signals, 28 Oct 1, 29 Oct 1, 30 Oct 1. But the 26 Oct bullet contradicts it, and the fixture
is then the only valid flat test.

### 3.3 Missing ledger silently becomes `PASS-empty` — reproduced

```
$ python3 g2_dryrun.py --day 2026-10-31 --cards ../logs/cards.jsonl \
    --ledger /tmp/does-not-exist-ledger.jsonl … --out /tmp/g2check/flat-1031-absent
exit 0
{"flat_day":{"pass_empty":true,"signals":0,…},"zero_hash_chain_head":true}
```
`g2_dryrun.load_jsonl()` returns `[]` when `Path.exists()` is false, and `main()` treats `not derived` as the flat
branch. A typo, an unmounted ledger, or a missing write therefore looks exactly like a flat day — on a day I chose,
with a path that never existed. Before the "first live flat session" is captured, the command must distinguish
"ledger exists and has 0 signals" from "ledger missing"; otherwise the future capture can produce a false
`PASS-empty`. Suggested: require `--ledger` to exist (or `--allow-missing-ledger` only for the labelled fixture),
and exit non-zero with `ledger_missing`. There is also no flat command in `README.md` — only the cards-day command —
so "the same command" is not documented.

## 4. C1–C4 — closures

### C1 — R19 fresh-ref, `check_replace()`, reconcile — PASS, one predicate hole

```
rails.check(intent(replaces='ob-orig', decision_ref='dec-2'), state(replaced_decision_ref='dec-2')) → ['replace_without_decision']
… replaced_decision_ref='dec-1' → []
… state without the key       → ['replace_without_decision']        # fails closed
… intent without 'replaces'   → []
rails.check_replace(dec-1 → same ref dec-1)              → (False, ['replace_without_decision'])
rails.check_replace(dec-1 → fresh dec-2)                 → (True, [])
rails.check_replace(dec-1 → dec-3, replace_count=2)      → (False, ['replace_without_decision'])
rails.check_replace(dec-1 → {'replaces':'ob-1'})         → (True, [])   # HOLE: no decision_ref at all
rails.check_replace(None   → fresh dec-2)                → (True, [])   # no original required
```
Both tests named in the evidence exist and pass (`test_r19_fresh_decision_ref_clause`,
`test_check_replace_helper`). The replace **path** is still G3 and is disclosed. The hole is in the standalone
predicate: a replacement with a missing/empty `decision_ref` passes because `"" != "dec-1"`. `make_intent`'s schema
would reject such an intent downstream, so it is not reachable through the current dry-run; fix it before the G3
replace path calls `check_replace` directly (require a non-empty ref, and refuse when `original` is absent).

`reconcile.py` is now called by `run_dry` (`reconcile_mod.join(fills=[], intents=allowed, decisions=…)`) and appears
in `summary.json`; `test_reconcile.py` covers clean/orphan-fill/orphan-intent/orphan-decision. Cards day:
`{'clean': True, 'fills': 0, 'intents': 1, 'decisions': 1}`. Note the fills leg is always empty in the dry-run
(there are no fills to join), so "fills↔intents↔decisions" is proven by the unit tests, not by the day run.

### C3 — guard startup — PASS

```
$ guard.startup(shipped CFG, paths without package_dir, NOW, '2026-11-02')
GuardError: ['disabled:no_enable_artefact','third_party:static_proof_missing','not_flat']
lock_file_exists: False                                             # 16:46Z
$ guard.startup(…, with proof/enable/flat/r11/desk, package_dir=exec) → ok; lock exists; second call:
GuardError: ['trader_lock:second_executor']
```
No `KeyError`; `package_dir` defaults to `HERE` and is present in `exec_config.paths` as `"."` (CWD-relative — starting
from another directory makes the proof `third_party:proof_stale`, i.e. fail-closed, but the `"."` is a fragility worth
noting). Lock is taken only after every other gate; no lock file on refusal. `verify_static_proof` also refuses a
false flag (`third_party:canary_ok`) and a stale digest (`third_party:proof_stale`) — I re-ran both.

### C4 — `must_disable` exercise — PASS on exercise, note on semantics

`test_venue_fatal_error_disables` exists and passes; my independent `Failing` venue through `executor.run_dry`:
```
disabled: true, allowed: 0, refused: 3
venue_calls_after_fatal: 3
refusal_counts: {'SessionMaterialError': 3, 'venue_disabled': 3}
```
So the path is exercised (the prior "called by nothing" is fixed). But `disabled` is only a summary flag: after the
first fatal `SessionMaterialError`, the loop still evaluates and calls the venue for every later card. In dry-run that
is harmless (recording venue, no send); in the G3 live loop it would be a send-after-session-material-breach bug. The
claim "the same handler the live loop uses" should be read as the *handler contract*, not as a proven stop; add a
`disabled` gate at the top of the card loop before G3. Also the summary counts the raw exception name
(`SessionMaterialError`) in `refusal_counts` alongside `venue_disabled`; harmless, but it is not a rail reason.

## 5. S-3 — keys, drift, R8

- The three keys stay in `live/exec/exec_config.json`: `daily_consistency_cap_usd: 1000`, `max_replaces_per_intent: 1`,
  `max_cancel_replace_per_10min: 3`. They are read by `rails.check` (the R14/R19 tests load this file).
- `guard.assert_desk_config(real live/config.json, CFG)` → `[]` (no drift today); `live/config.json` mtime unchanged
  (read-only).
- I mutated each of the ten shared rails in memory and each of the ten was caught with a named reason:
  `contracts`, `symbol`, `contract_id`, `max_trades_per_day`, `daily_loss_stop_usd`, `daily_profit_stop_usd`,
  `cooldown_min_after_loss`, `flat_et`, `min_stop_pts`, `feed_stale_s` ↔ `stale_s`. Deleting each desk key produced
  `config_drift:desk_missing:<key>`; 10/10.
- **R8:** the threshold really comes from config when present — age 61 s → `stale_feed`, age 59 s → `[]` with
  `feed_stale_s: 60`. But the code is `float(rails.get("feed_stale_s", 60))` (`rails.py:134`), so the literal **60 is
  still there** as a silent fallback; with the key removed, age 61 is still refused by the 60 default. The build
  evidence's "no literal in `rails.py`" is therefore **false**. The startup drift check catches a missing
  `feed_stale_s` before production (`config_drift:exec_missing`), so the exposure is limited to a direct/pure
  `rails.check` call — but the wording should be corrected or the fallback removed.

## 6. Regression — PASS

Cards day, real inputs (1 ledger `signal` ↔ 1 entry card), re-run 16:49:25Z:
```
chain_head 1121cdbb04b61ece23d5ab65d6058e582766222627fbdc833a52e090b0fbf492
compare_vs_cards {allowed_intents:1, clean:true, entry_cards:1, extra:[], mismatch:[], missing:[]}
rerun_identical true, intents_bytes_identical true, chain_verified true
artefacts_sha256.intents.jsonl f83cde7401d0f52036a9dccadf279a49f743827f3cd8b99c1630a7e7163d0493
reconcile {clean:true, intents:1, decisions:1, fills:0}
$ diff -rq /tmp/g2check/final-cards board/probes/team-c/exec-g2a/dryrun-2026-10-30 → IDENTICAL (all files)
```
I also re-hashed the record independently (`json.dumps(sort_keys, separators)` minus `record_hash`, chained from
`0`×64): `chain_ok True, head 1121cdbb…`. So it is not `verify_chain` grading itself.

No transport / no enable / no network:
- `find . -name enabled.json -o -name trader.lock -o -name static_proof.json` → empty in the package.
- `exec_config.json` `"mode": "dry-run"`.
- No socket/transport import in the six runtime modules; `CDPVenue(` appears only in `venue.canary(HERE)` and the
  session-material tests, both with `HostileTransport`; the dry-run uses `RecordingVenue`.
- Clean audit: 0 sockets, 0 subprocess, 0 deny-path opens; no packet was sent by any probe (the socket probe created
  and closed a socket object only — no connect).

Refusal table R1–R19: still present in `rails.REASONS` and `rails.check`, mapping unchanged (R1 `daily_loss_stop` …
R19 `replace_without_decision`/`cancel_rate`); `test_rails.py` has one mutation per rail, the R11 gate paths, the
missing-state fail-closed test and the vocabulary test; the suite passes. The R19 fresh-ref clause is now included
(§4).

## 7. What remains before the veto lifts (closing list)

| # | Item | Closes when |
|---|---|---|
| **B2-E1** | Build evidence §3 calls 2026-10-26 a "real pre-engine day" and uses it as flat evidence; the engine log and `selftest-pulse-2026-10-26.jsonl` both show 2 pulse signals, 2 fills and a `-$17.50` exit; the day-keyed ledger path used does not exist | Correct §3/README: 26 Oct is a **signal day** (pulse test; selftest ledger exists), not a flat day; keep only the labelled 0-byte fixture as the flat-shape test |
| **B2-E2** | Missing ledger silently → `PASS-empty` (reproduced on 10-31 with a non-existent path) | `g2_dryrun` requires `--ledger` to exist and be readable for a real day (explicit `--allow-missing-ledger` only for the labelled fixture); non-zero `ledger_missing` otherwise; test it; document the flat command in README |
| **B2-E3** | §F(v)'s second day shape is fixture-only; no real zero-signal session exists | Capture the first real zero-signal session with the corrected command and attach `compare.json` — **or** obtain an explicit Lead waiver that the labelled fixture is the G2 flat-day evidence. The maker's disclosure of this limit is honest and acceptable; the 26 Oct claim is not |
| **B1-A1** | Deny anchor self-referential; `rules_sane` vacuous if metadata removed; a coherently neutered `deny.json` gives `all_ok: true` and `guard` accepts the proof (deny digest unchecked; digest excludes `deny.json`) | Include `deny.json` (+ `exec_config.json`) in the integrity binding or pin/verify `deny_sha256`; hardcode a minimum rule set independent of the file; make the emptied-deny planted negative assert `rules_sane: false` and record it in `planted-negatives.json` |
| **C-notes** | `check_replace` allows a missing/empty `decision_ref`; `must_disable` sets `disabled` but the loop keeps calling the venue; R8 literal 60 fallback remains; the "74 opens" is 73–74 depending on run (context only); the runtime trio shares one `audit.ok` gate | Fix/annotate before G3 (the first two are live-path items); the open-count wording and coupling are documentation |

**Not required for G2:** the live executor loop, the replace path, the real CDP page canary (G2.5), and Monday's
cards/guard-only operation. Nothing in this HOLD blocks Monday's cards/guard route; the package is not wired to it.

## 8. Footprint / reproducibility

- No content write to the package, engine, config, or `live/` — only transient `__pycache__`/`.pytest_cache` dirs
  were created by imports and removed; `live/config.json` mtime unchanged; `live/exec` `git status` clean before and
  after. The two cache dirs I created were removed; final digest
  `2a184f26…` re-verified after cleanup. No `state/`, lock, or `enabled.json` left behind.
- All mutations were `/tmp` copies (`/tmp/g2check/neg/*`); the probe scripts are `/tmp/g2check/negatives.py`,
  `probes.py`; outputs `/tmp/g2check/{negatives.json,probes.out,neutered-proof.json,full-neutered-proof.json}` and
  `/tmp/g2check/{flat-fixture,flat-1026-absent,flat-1031-absent,cards-1030,final-*}`.
- No live action, order, wallet, credential, spend or network send; the socket probe created and closed a socket
  object without `connect()`/`send()`, and the audit recorded it for the `network_ok` test.
- Commands were run from `~/Work/projects/openbell/live/exec` unless a path is shown; timestamps are the run's own
  (`date -u`), not reconstructed.

— checker seat, Team C G2 (fresh context). **Verdict: HOLD** — B1 mechanism fixed; B1 anchor and B2 real-day evidence remain.
