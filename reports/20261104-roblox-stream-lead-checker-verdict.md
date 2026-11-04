# Independent checker verdict — ROBLOX stream, overnight 2026-11-04

Checker: separate session, same pinned model (`opencode-go/deepseek-v4.1-flash`), read-only on both repos. Maker ≠ checker:
this session produced none of the work below and re-ran every proof on its own.

## Verdict

**The work stands, with four claims corrected.** Every measured artifact reproduced, including both places byte-for-byte
where the build is deterministic: the street RC `c34e4cc2…` rebuilt to the identical bytes, and the card place
`99ded6a6…` reproduced exactly once the real `stamp_manifest_hash.py` step was applied (a bare `rojo build` differs
because the stamp writes the manifest hash into the place — the checker ran the stamp in a `/tmp` sandbox and `cmp`
came back identical).

| Check | Verdict | Evidence |
| --- | --- | --- |
| 1 · street merge order/parents | PASS (wording corrected) | `300b632` parents `536e3c8` + `bfc28ab`; `536e3c8` parents `5ee9f63` + `18d01a2`. **`main` is not `300b632`** — three docs commits on top (`15a5c35`, `cf37b19`, since `b882a1d`) |
| 2 · no gameplay change in the merges | PASS | `git diff --stat 536e3c8 HEAD -- src` empty; first-parent diffs of both merges touch docs/tests/tools only |
| 3 · sealed place hashes | PASS | `84324d43…6ce`, `c6651793…e98` exact |
| 4 · street Python suite | PASS | `Ran 369 tests in 60.1s`, `OK`, exit 0 |
| 5 · street Lune suites | PASS | unit `943 passed, 0 failed (130 specs)`; `edge` `10 passed, 0 failed (3 specs)` |
| 6 · street RC reproducibility | PASS | rebuild → `c34e4cc2…bbfa5`, 1,992,286 bytes, byte-identical |
| 7 · no publication | PASS | no `2026-11-04` entry in `docs/RELEASE_LOG.md`; no publish artefacts in `build/` |
| 8 · Studio state / threshold claim | PASS (Studio) · **corrected** (thresholds) | no `RobloxStudioBeta.exe`; but `docs/art/thresholds.json` is a **new file added by `bfc28ab`, merged at `300b632`** — "no threshold was edited" was false as worded; correct form is "added by that merge, nothing edited after it" |
| 9 · card merge topology | PASS | `c15d534` = `04031c0` + `e4e4746`, then `9c3606b` = `c15d534` + `956b4bc`, in that order |
| 10 · card suites both runtimes | PASS (runner name corrected) | 7 suites green on lua **and** lune; combat lives at `tests/run_all.luau` (there is no `tests/combat/run_all.luau`); only combat/integration/presentation print `TOTAL failed=` |
| 11 · balance wall | PASS | `Config.ROUTE_ENEMY_LEVEL` line 104 `["w3-enc-1"] = 8`; `test_balance_pass` `PASS (39 assertions)` |
| 12 · card RC + manifest | PASS (stronger than claimed) | place `99ded6a6…`, manifest `49c4338b…`; stamp applied in `/tmp` reproduces the place byte-for-byte |
| 13 · upload safety fix | PASS | default 5 cues exactly as authorised; `--all` → 8 + overrun warning; `--live` without key → exit 2, no POST; 7 tests pass |
| 14 · art suite | PASS | `46 passed, 1 failed`, and the failure is the checkout-bound absolute-path comparison (`BC-SEA-01.png` `9b1937ba…` identical in both checkouts) |
| 15 · zero uploads | PASS (one clause reworded) | `publish.py status`: uploaded art IDs 0, key no, target missing; `ops/release/audio-bindings.json` **does not exist yet** (created by the first upload), so that absence is the evidence, not an empty file |
| 16 · verify evidence | PASS | `exits.json` = 22 × `"0"` + `toolkit-53: skipped`; gate report STAGING with the three offline gates PASS and the two owner gates BLOCKED; `build-1.sha256` == `build-2.sha256` |
| 17 · package doc vs reality | PASS with two numeric corrections | 56-file manifest, 22 steps, 39-assertion balance test, 1331-assertion journey, docs-only branch one file/+1134 all confirmed; **input count is 135, not 134**, and `verify/build-2.sha256` is not a worktree path |

Not verifiable from either repo, flagged not failed: the Studio working place hash `ab2080f4…` (sealed file the checker
did not read; the maker measured it with `sha256sum`) and the rights history behind A-04 (that comes from the spec and the
owner channel, not from an artifact).

## Where the corrections landed

- Street repo `b882a1d`: `main` vs "last code merge" wording in `docs/STATE.md` and
  `docs/verify/2026-11-04/PLAYTEST-READINESS.md`; the threshold file's real history; the `usg-desktop.rbxlx` hash now
  says how it was measured.
- Card repo `607bb28`: 135 inputs, the evidence path for the place hash, the real suite-runner names, the audio
  registry's absence stated as absence, and `master` head updated.
- No number in either package changed as a result: every correction was to the words around a measurement, not to a
  measurement.

## What the checker did not do

No commits, no branch changes, no build written inside either repo (all builds went to `/tmp`), no `tools/usg`
subcommand, no publish, no network call, no sealed place file read or written.
