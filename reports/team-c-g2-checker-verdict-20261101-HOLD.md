# TEAM C — G2 CHECKER VERDICT (B2 exec package) — 1 Nov 2026

**Seat:** fresh-context, independent Tier S checker with VETO. Did not build the package; reports to the Lead only.
**Subject:** `~/Work/projects/openbell/live/exec/` (G2 build) against design v4 `ea0ec96e…` under **G1-SIG-2026-11-01-TS-03**.
**Fences observed:** read-only on the package, engine and config; no live action, no order, no wallet, no credential, no
spend, no network send. Only write = this report. All probes ran from `/tmp` copies or in-memory; the one filesystem side
effect (`live/exec/state/trader.lock` created by the guard probe) was removed — package left as found.
**Checked at:** 2026-11-01 11:05–11:18 ET (15:05–15:18Z).

## VERDICT: **HOLD** — 7 of 9 checked items pass; item 4 fails on evidence, and two v4-named G2 evidence items are not met as specified.

Not FAIL: nothing can send (mode `dry-run`, no enable artefact, no transport, 0 sockets observed, guard refuses), and the
§A.2 prohibition itself is genuinely implemented. Not PASS: the package's own static-proof record asserts a **runtime canary**
and a **filesystem assertion** that do not exist, and the **flat-day shape** (§F(v)) cannot be produced by the harness. Both
are cheap to close (see §10). Signing now would sign a false evidence record.

| # | Item | Verdict | Decisive evidence |
|---|---|---|---|
| 1 | Package integrity, digest, design untouched, repo status | **PASS** | `9848c164…` reproduced exactly; v4 `ea0ec96e…`; only `?? live/exec/` new |
| 2 | `pytest . -q` → 56 passed | **PASS** | `56 passed in 0.17s`, exit 0, twice |
| 3 | Closed-day dry-run byte-identical, chain, compare 0/0/0 | **PASS** (narrow) | 7/7 artefacts byte-identical; chain independently recomputed; n=1 card |
| 4 | Static proof + S-1 implementation | **FAIL** (partly) | AST/deny real; `canary_ok`/`filesystem_ok` are the *same scan*, not those tests |
| 5 | S-2 R11 gate rails | **PASS** | status→day→ET window→bands→active band; `checks` never read |
| 6 | S-4 retained; S-5 dry-run, no enable, no CDP, no network | **PASS** | guard refuses; 0 sockets/0 subprocess under audit hook |
| 7 | R1–R19 refusal table | **PASS** (one clause missing) | all 19 named+tested; R19 "fresh decision ref" has no code |
| 8 | S-3 config keys | **RULING: satisfied in substance, conditional** | keys present **and proven read**; drift risk real; do not move now |
| 9 | Honest limits | **STATED, two overstated** | "dry-run-exercised" → unit-tested only; flat-day claim unsupported |

---

## 1. Package integrity — PASS

```
$ cd ~/Work/projects/openbell/live/exec && python3 -c "import guard;print(guard.package_sha256('.'))"
9848c164bef8b25b9cc88971574aa6a6bf9e9e03743778c3e6eda97d60ad2ccc        # 2026-11-01T11:16:51-05:00
```
**Digest scope (corrected for the record):** `guard.package_sha256` = sha256 over `sorted(*.py)`, feeding each *filename
bytes then file bytes*. It is **not** a digest of the whole directory: **`exec_config.json` and `README.md` are excluded**.
Consequence — see §8 — the R14/R19 config values are **not** bound by the digest or by `guard.verify_static_proof`; a config
edit leaves the package "proof-bound" and unchanged. Acceptable for its stated purpose (send-boundary/AST proof), but it
must not be read as "the package including its rails config is hash-pinned".

Per-file: all 14 manifest entries in `board/probes/team-c/exec-g2a/package-files.txt` reproduce byte-exactly
(`sha256sum *` matched line for line; `executor.py c894b71b…`, `rails.py f6e2ea2b…`, `venue.py 572fe3f0…`,
`exec_config.json ed105056…`). `README.md` (`c6ad7b87…`) is not in the manifest — noted, not a defect.

Design v4 untouched: `sha256sum board/reports/team-c-autonomous-path-design-v4.md` → `ea0ec96e1402fd44…` ✓; mtime
`2026-11-01 10:04:00 -0500`, i.e. **before** the build window (10:31–10:40), consistent with "not edited after G1".

Repo status (`git status --porcelain=v1` at HEAD `d555b50`, branch `research/kronos-stage1-verdict`):
```
 M live/.gitignore            M live/dash.html          M live/test_dash.py
 M live/RUNBOOK.md            M live/dash.py            ?? live/exec/
 M live/agent/nightly_status.json  M live/fixtures/… (2)  ?? live/test_operator_brief.py
```
Only new **directory** is `live/exec/`. `live/config.json` and `live/engine.py` are **not** in the status → untouched. Every
modified path carries an mtime of 2026-10-28…10-30, i.e. **pre-existing dirt**, before the build. `live/test_operator_brief.py`
is a pre-existing untracked file (mtime 2026-10-28 17:58:33), not produced by this build. **Claim holds.**

## 2. Test suite — PASS

```
$ PYTHONDONTWRITEBYTECODE=1 python3 -m pytest . -q -p no:cacheprovider
........................................................                 [100%]
56 passed in 0.17s
EXIT=0          # START 2026-11-01T11:07:50-05:00  END 2026-11-01T11:07:50-05:00
```
Re-run a second time under an audit hook (§6): `56 passed in 0.17s`, rc 0. No `.pytest_cache`, no `__pycache__` left
(bytecode and cache plugin disabled for the run).

## 3. Closed-day dry-run (2026-10-30) — PASS, narrow

```
$ python3 g2_dryrun.py --day 2026-10-30 --cards ../logs/cards.jsonl --ledger ../ledger/2026-10-30.jsonl \
    --r11-fixture <board>/probes/team-c/exec-g2a/r11-fixture-2026-10-30.json --out /tmp/g2rerun
EXIT=0          # 2026-11-01T11:10:39-05:00 → 11:10:39
```
My re-run vs the maker's committed artefacts (`cmp`, 2026-11-01T11:11 ET): **IDENTICAL for all 7** —
`run-a/{intents.jsonl,summary.json,venue_calls.json}`, `run-b/{…}`, `compare.json`.
- intents sha256 `f83cde7401d0f52036a9dccadf279a49f743827f3cd8b99c1630a7e7163d0493` ✓ (claimed `f83cde74…`)
- chain head `1121cdbb04b61ece23d5ab65d6058e582766222627fbdc833a52e090b0fbf492` ✓ (claimed `1121cdbb…`)
- `compare.json` sha256 `ee2c165d556fafa26d2a960b96c015b1bb63cbd0ab00794848a46bed0c3796ac` ✓

Chain verified **by my own recomputation**, not `executor.verify_chain`: re-hash `record` minus `record_hash` with
`json.dumps(sort_keys,separators)` → `chain_ok: True`, `head: 1121cdbb…`; flipping `intent.qty` → tamper detected (`True`).
Inputs are the real ones: `logs/cards.jsonl` holds 26 cards, **1** entry card on 2026-10-30 (`OpenBell SELL 1 MES`,
price 7649.75, stop 10.50, target 21.00); `ledger/2026-10-30.jsonl` holds exactly **1** `signal` row (side −1, stop 10.5,
target 21.0, px 7649.75, `time_exit_et` 15:55). Compare 0 missing / 0 extra / 0 mismatched is therefore **over n=1** — real,
but nearly vacuous.

**Gap:** design §F(v) requires "two day shapes (30 Oct cards day + **one flat day**)". The harness cannot produce the second:
```
$ python3 g2_dryrun.py --day 2026-10-31 … --out /tmp/g2flat3
{"error": "no_signal_rows", "day": "2026-10-31"}     exit=2      # 2026-11-01T11:17:07-05:00, no artefact dir created
```
`g2_dryrun.py:148-150` returns exit 2 when no signal exists — there is no zero-intent artefact path. The build evidence's
claim that the flat shape "is covered by the `PASS-empty`/no-card path in tests" is **not supported**: `grep -rn
"PASS-empty\|empty\|no_signal"` finds only this exit-2 branch, and no test builds an empty card list.

## 4. Static proof / S-1 — **FAIL (evidence, not behaviour)**

The §A.2 prohibition is **genuinely implemented** in `venue.py`, and I verified it:
- `DENY_SUBSTRINGS` (incl. `document.cookie`, `localstorage`, `sessionstorage`, `indexeddb`, `navigator.credentials`,
  `authorization`, `csrf`/`xsrf`, `keyring`, `password`, `secret`, `profile`) and `DENY_METHODS` = `Network.getCookies`,
  `Network.getAllCookies`, `Storage.getCookies`, `Storage.getStorageKeyForFrame` (N-1 done) ✓
- `validate_expression` is **deny-first, then allowlist**; exactly **5** templates (`read_inner_text`, `read_text_content`,
  `read_attribute`, `set_input`, `click_control`); unmatched → `UnmatchedEvaluateError`; value injection
  (`2";document.cookie;//`) → refused ✓
- `assert_package_boundaries` on the real tree: `problems == []`; no transport import outside `venue.py` ✓
- `must_disable()` truth table tested ✓

**The defect is in the proof record.** `venue.static_proof()` sets all four flags from **one** computation:
```
static_proof():  problems = assert_package_boundaries(dir)
                 session_material_ok = ast_deny_ok = canary_ok = filesystem_ok = not problems
```
So `static-proof.json`'s `canary_ok` and `filesystem_ok` are **not** the §A.2 runtime canary and filesystem assertion — they
are the static scan wearing two other names. Demonstrated on a `/tmp` copy (2026-11-01T11:08–11:09 ET), each planted defect
leaving `problems: []` and **all four flags true**:

| Planted in a copy of the package | proof says |
|---|---|
| `import smtplib` + `import telegram` in `executor.py` | all true, `problems: []` |
| real `open("~/.config/chromium/Default/Cookies")` + `/proc/self/environ` read in `venue.py` | all true, `problems: []` |
| real `open("/home/x/.config/chromium/Default/Cookies")` in `reconcile.py` | all true, `problems: []` |

Three structural reasons: (a) **`venue.py` is exempt from both scans** (`if path.name != "venue.py"`) — it must be, it
contains the deny literals — so the one module that could touch a profile is outside the proof; (b) `TRANSPORT_MODULES` has
no messaging SDK (`smtplib`, `telegram`, `slack`, `discord`, `twilio`), so §D's "**no messaging client**" is satisfied by
absence, not enforced; (c) the "filesystem assertion" test (`test_filesystem_assertion_no_profile_paths`) is a **source-text
grep that also skips `venue.py`**. Appending a real profile read to `venue.py` leaves **all 9 session-material tests passing**
(`9 passed in 0.13s`, /tmp copy 11:09 ET). The same read in `reconcile.py` *is* caught (1 failed) — the test has teeth
everywhere except the module that matters.

The "runtime canary" does not exist as a test: `FakeTransport.evaluate` returns the constant `"no positions"`; the test
asserts only that the *emitted expressions* are allowlisted. There is no fixture page and no cookie-like value that could be
"touched", so the canary cannot fail. Note the design tension: S-5 forbids instantiating a CDP transport at G2, and a canary
page needs one — so the canary is structurally a **G2.5** artefact (G2.5 is "pinned selectors against a recorded page
snapshot"). The right fix is to say so and stop asserting `canary_ok`, not to fake it.

**Compensating evidence I ran as checker** (the assertion the package doesn't do): full suite under a `sys.addaudithook`
(2026-11-01T11:16:51-05:00):
```
56 passed in 0.17s
AUDIT rc=0 opens=1740 distinct=994 sockets=0 process_events=0 profile_opens=0
```
i.e. **0** `socket.__new__`/`connect`/`getaddrinfo`/`bind`, **0** `subprocess`/`os.system`, **0** opens of
`.config/chromium`, `Cookies`, `Login Data`, `Local State`, keyring or `/proc/self/environ`. So today's *behaviour* is clean;
only the *record* overstates. `guard.verify_static_proof` consumes that record at startup and requires all four true — the
startup gate therefore rests on a self-produced, low-discrimination JSON.

## 5. S-2 (R11 gate rails) — PASS

`rails._r11_reasons` order is exactly S-2: `status=="SET"` → `day == state["day"]` → `read_at` parsed and converted to ET
(`astimezone(NY)`) → ET date == trade day → `09:00 ≤ HH:MM ≤ 09:15` from `exec_config.r11.guard_window_et` → **bands
validated** (`ref_price` numeric > 0; `b7/b13/b20` each dict; `upper`/`lower` each 2-element numeric list with `pair[0] ≤
pair[1]`) → active-session band test. The sidecar's literal `checks` block and its `contract` literal are **never read** —
verified by inspection and by battery:

| Probe (2026-11-01T11:14 ET, direct `rails.check`) | Result |
|---|---|
| `checks` all true **but** `refuse_bands` = b7 only | `price_limit:reduced` ✓ |
| `checks` all true **but** `refuse_bands` absent | `price_limit:reduced` ✓ |
| `read_at` 09:15 ET (boundary) | allow ✓ |
| `read_at` 09:16 / 08:59 ET | `price_limit:read_outside_guard` ✓ |
| `read_at` previous day 09:04 ET | `price_limit:stale_read` ✓ |
| `ref_price` as string `"7614.06"` | `price_limit:reduced` ✓ |
| band pair reversed `[hi,lo]` | `price_limit:reduced` ✓ |
| price at band edges 7994.76 / 8147.04 | `price_limit:band` (inclusive) ✓; 7994.75 allows ✓ |
| active set at 15:25 ET: b7 band hit ignored, b20 band hit refused; b7 active again at 16:00 | matches §B.1 session mapping ✓ |

End-to-end through the real harness (2026-11-01T11:16:51-05:00) — day 2026-10-29 run with the 2026-10-30 fixture:
`allowed 0 refused 1 {'price_limit:stale_day': 1}`, `venue_calls []`, `chain_verified True`, `rerun_identical True`, exit 1.
A stale/mismatched R11 record produces **NO TRADE and no send**.

Two residual notes (not defects against v4): (a) the sidecar's `contract` is never compared to `exec_config.instrument.symbol`
— a `MESH6` sidecar with valid bands is **allowed**; v4 §B.1 assigns the contract-month check to the generator, so this is
within design but is a defence-in-depth gap the executor could close for free; (b) a timezone-less `read_at`
(`2026-11-02T09:04:00`) is accepted as **host-local** — correct on this ET host, and it fails closed on a UTC host.

## 6. S-4 retained; S-5 (dry-run, nothing armed) — PASS

- **S-4** is still in the gate table: v4:185 G4 row "fresh re-read of all four firm pages" ✓ (unchanged; label loss cosmetic).
- `exec_config.json` `"mode": "dry-run"` ✓.
- No `state/` directory, no `enabled.json` anywhere under `live/exec/` (`find . -name enabled.json` → empty) ✓.
- No CDP transport instantiated: `CDPVenue(` appears only in `test_session_material.py` with a `FakeTransport`; the package
  imports no transport module at all (AST inventory of the runtime modules: `ast, json, re, os, hashlib, pathlib, typing,
  fcntl, datetime, zoneinfo, argparse, sys, __future__` — stdlib only) ✓.
- No network: audit-hook run above, 0 socket events, 0 subprocess ✓.
- Guard refuses on the real config (2026-11-01T11:16:51-05:00):
```
mode = dry-run
REFUSED: ['disabled:no_enable_artefact', 'third_party:static_proof_missing', 'not_flat']
```
**Note (latent wiring defect, G3):** `guard.startup` needs `paths["package_dir"]`, which is **not** in
`exec_config.paths` — driving it straight from the config raises `KeyError: 'package_dir'` instead of a named
`GuardError`. The startup path has therefore never been exercised with the shipped config; `test_guard` supplies
`package_dir` by hand. Also, a refused startup still creates `state/trader.lock` (lock is taken before the gate verdict).

## 7. R1–R19 refusal table — PASS, one clause missing

All 19 rows are present with a named reason and exercised by `test_rails.py` (one mutation per rail), plus R11's five gate
paths and both band sides; `test_every_reason_in_vocabulary` bounds the vocabulary. Mapping verified rail-by-rail:
R1 `daily_loss_stop` (incl. stop-out crossing) · R2 `daily_profit_stop` · R3 `max_trades` · R4 `cooldown` · R5 `late_entry` ·
R6 `instrument` · R7 `min_stop` · R8 `stale_feed` (age + `retrying`) · R9 `outside_window` (window + holiday) ·
R10 `micro_scalp` · R11 `price_limit:*` · R12 `illiquid_or_gap` · R13 `bracket_shape`/`hedging` · R14 `consistency` ·
R15 `mll` · R16 `third_party` · R17 `trader_lock`/`not_flat`/`disabled` · R18 `ambiguity` · R19
`replace_without_decision`/`cancel_rate`. Missing state fails closed (`missing_state:<key>`).

"Each asserted to produce no send" is **structural, not per-rail**: `rails.check` is pure and holds no venue handle, and one
integration test drives `enabled=False` through the executor (`allowed == 0`). No test asserts `venue_calls == []` per rail.
Acceptable in substance (a pure predicate cannot send); the letter of §F(iii) is one test short.

**Gap:** R19's clause "any replacement needs a **fresh decision ref**" has **no code**. `replace_without_decision` fires only
when `replaces_for_intent > max_replaces_per_intent`; no module compares a replacement's `decision_ref` against the original,
and no replace path exists at all (`venue.replace` is a stub, `run_dry` never replaces). Vacuous today, so not a G2 blocker —
but it must land with the replace path at G3, not after it. Likewise `reconcile.py` has **no test and no caller**;
"reconcile after every transition / disable on orphan" (§C.5) is not wired.

## 8. S-3 RULING — the separate exec config satisfies S-3 **in substance**; keep it, but close the drift

**Facts.** `live/exec/exec_config.json` carries `daily_consistency_cap_usd: 1000`, `max_replaces_per_intent: 1`,
`max_cancel_replace_per_10min: 3`. `live/config.json` carries none of them. The three keys are **really read from config**,
not literals — mutating a `/tmp` copy flips the rail (2026-11-01T11:13 ET):
```
max_replaces_per_intent=0            -> ['replace_without_decision']
max_cancel_replace_per_10min=0       -> ['cancel_rate']
daily_consistency_cap_usd=10         -> ['consistency']
```
That is precisely what G-3/S-3 asked for ("asserted from config, like R14's key"), and the tests load `CFG` from the file.

**Why the keys are not in `live/config.json`, verified.** `live/config.json` mtime = `2026-10-27T22:43:34Z` (epoch
1793141014, `os.stat().st_mtime`); engine start = `2026-10-28T19:40:08Z` (epoch 1793216408, `ps -o lstart= -p 1484`; the
Monday packet's `19:40:09`/`…409` is the same start, 1 s of rounding) — the config is **20.9 h older**, process 1484
`python3 -m live.engine --config …/live/config.json`, up 3 d 20 h. So `CONFIG_OLDER_THAN_PROCESS: PASS` is currently true
**because** that file has not been touched. Editing it before Monday's session flips Monday's (d) attestation to
`CONFIG_CHANGED_AFTER_START: FAIL` (runbook line 143) and voids the evidence frame.

**Ruling.** S-3's letter says "must exist in config"; its parenthetical locates the absence in *live* config, so the letter is
ambiguous. I rule on purpose and on risk:
1. **The letter is satisfied by `exec_config.json`** — the condition exists to stop magic numbers in code, and it does: keys
   are named, reviewable, read at runtime, and flipping them changes behaviour. **PASS for G2 on S-3.**
2. **The keys must NOT move before Monday.** Moving them into `live/config.json` now is the one action that breaks the
   Monday frame for a paperwork point.
3. **The real exposure is drift, not location.** `exec_config.json` duplicates 8 desk-rail values that also live in
   `live/config.json` (`daily_loss_stop_usd −400`, `daily_profit_stop_usd 900`, `max_trades_per_day 2`,
   `cooldown_min_after_loss 15`, `flat_et 16:05`, `min_stop_pts 2.0`, `symbol MESZ6`, `contract_id 4470322`, `contracts 1`).
   Today they agree; nothing enforces that they keep agreeing. Separately, R8's 60 s threshold is a **literal in
   `rails.py:134`** although `live/config.json` has `stale_s: 60` — the same class of defect S-3 was raised about.
4. **Fix that does not touch Monday's frame:** have the executor **read** `live/config.json` (read-only — reading does not
   change mtime) for the shared desk-rail subset at startup and **fail closed on any mismatch** with `exec_config.json`,
   keeping only the three new keys (R14 cap, both R19 keys) in `exec_config.json`. Optionally add `stale_s` to the read.
5. **If the Lead nevertheless requires the keys inside `live/config.json`:** do it only *after* Monday's session, in a planned
   window — stop the engine → edit → restart → re-run all five attestations so the new `config_mtime` < new process start. The
   Monday frame's PASS is then a **new** frame; it must not be carried over.

## 9. Honest limits — stated, two of them overstated

Correctly disclosed by the maker: the closed-day R11 input is a **labelled fixture** (`"fixture": true` — I confirmed the flag
and the note in `r11-fixture-2026-10-30.json`), G2.5/G3 are separate, nothing is armed. Two claims need correcting:
- **"the live loop's catch-and-disable wiring is dry-run-exercised only."** There is **no catch site**: `must_disable` is
  defined and unit-tested (truth table) and **called by nothing** (`grep -rn must_disable` → definition + test only). It is
  specified and unit-tested, not exercised. No live loop exists in the package at all.
- **"the flat/no-signal day shape is covered by the `PASS-empty`/no-card path in tests."** No such test exists; the harness
  exits 2 on a signal-free day (§3).

Still true and material: the **real R11 record from a live 09:05 guard does not exist** (Monday 2 Nov will produce it via
`fetch-cme-limits.sh`); the live loop, `reconcile` wiring, R19's decision-ref clause and guard startup against the shipped
config are all G3 work.

## 10. What remains before a G2 signature (the HOLD, closing list)

| # | Blocker | Closes when |
|---|---|---|
| **B1** | `static-proof.json` asserts `canary_ok` and `filesystem_ok`; neither test exists, and both flags are the same static scan (venue.py exempt, messaging SDKs uncovered) | Either (a) implement them — a real filesystem assertion (audit hook, as run in §4) and a canary page; or (b) correct `venue.static_proof()`/the JSON to report **only** `session_material_ok` + `ast_deny_ok`, and move the canary to **G2.5** with a named obligation there (S-5 forbids a transport at G2, so a canary page cannot exist yet). Extend `TRANSPORT_MODULES` with `smtplib/telegram/slack/discord/twilio` either way. |
| **B2** | §F(v) second day shape (flat day) not produced and not producible | Add a zero-signal path to `g2_dryrun.py` (0 intents → artefact + `compare 0/0/0` + zero-hash chain head) and run one real flat day. |
| **C1** | R19 "fresh decision ref" has no code; `reconcile` unwired | Implement with the replace path at G3 (record as a G3 obligation, not a G2 blocker). |
| **C2** | Desk-rail drift between `exec_config.json` and `live/config.json`; R8's 60 s literal | Read-only load + mismatch fail-closed (§8.4) before G3/G4. |
| **C3** | `guard.startup` `KeyError` on `package_dir`; lock file created on refusal | Add `package_dir` to the config's paths block and gate the lock after the verdict (G3 wiring). |
| **C4** | Evidence wording ("dry-run-exercised", "filesystem assertion", "runtime canary") | Correct the build evidence/README when B1 is resolved. |

B1 and B2 are the signature blockers; C1–C4 are conditions to attach to the signature. With B1+B2 closed and C1–C4
recorded, this seat's veto lifts: items 1, 2, 3, 5, 6, 7 pass on re-run and the §A.2 prohibition itself is real.

## 11. Footprint / reproducibility

- Package, engine and config were **not modified**. One side effect — `live/exec/state/trader.lock` created by the guard
  probe — was deleted; `ls -d state` → "No such file or directory" (2026-11-01T11:16:51-05:00). The package digest is
  unaffected (it covers `*.py` only) and was re-verified afterwards.
- All other probes ran from `/tmp` copies (`/tmp/g2probe`, `/tmp/g2probe2`, `/tmp/g2cfg`) or in memory; dry-run outputs to
  `/tmp/g2rerun`, `/tmp/g2cross2`, `/tmp/g2flat3`.
- Engine state at check time (untouched, for the record): heartbeat `2026-11-01T11:06:06-05:00`, `status running`,
  `mode cards`, `market_open false`, `feed retrying (RuntimeError)`, `feed_age_s 155166` — the desk was not trading and the
  exec package is not wired to it.
- Commands and outputs quoted above were produced 2026-11-01 11:07–11:18 ET; timestamps are the ones printed by the runs.

— checker seat, Team C G2. **Verdict: HOLD.**
