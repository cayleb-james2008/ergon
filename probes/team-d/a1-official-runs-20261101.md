# A1 official runs — prep branch + post-merge main (2026-11-01 12:58–13:01 ET)

Checker: `board/reports/team-d-a1-highway-spec-checker-verdict.md` — **PASS** on
`prep/a1-highway-spec-20261101` @ `1abe303` (only `tests/studio/highway.spec.luau`, +67/−20, base `b67abdd`,
`main` untouched, sealed artifacts unchanged). Re-run required and observed below. MCP-only, text/JSON, no build, no
re-pin, no host input, no images.

## Run 1 — official A1 on the sealed candidate sources (prep branch)

Log: `board/probes/team-d/a1-in-studio-final-20261101.log`

| Item | Value |
|---|---|
| Branch / commit | `prep/a1-highway-spec-20261101` @ `1abe30391caf88db9d76bd424d87461899b5cdf9` (base `b67abdd`) |
| Studio | `usg-desktop.rbxlx`, Edit (MCP `list` + `play state` before the run) |
| `sync --test` | `{"synced":101}`, rc 0 |
| `tests` | **rc 0 — all 11 Studio spec modules pass**; `highway.spec = {"passed":2,"seedsPassed":100,`
`"expectedTrafficCount":54,"minimumTrafficSpacingSlots":0.6694870414212347,"roadSamples":3142,"groundSamples":5,`
`"graphNodesChecked":20686}` |
| Sealed sha256 BEFORE | `84324d43…f6ce build/release.rbxlx` · `c6651793…fe98 build/test.rbxlx` · `c6651793…fe98` runner place |
| Sealed sha256 AFTER | **identical** (all three) |

## Run 2 — post-merge A1 on `main` (checker-requested integration run)

Log: `board/probes/team-d/a1-in-studio-postmerge-main-20261101.log`

- Pre-merge `main` was `8dfb204`; merged `prep/a1-highway-spec-20261101` locally with a **no-ff merge commit**
  **`de361ba848db84a084e4446e828ad4f09bf3dd92`** ("Merge A1 highway.spec design-contract alignment (checker PASS
  1abe303)"). Delta vs old main: **only `tests/studio/highway.spec.luau`** (`git diff --name-status 8dfb204..HEAD`).
  Repo has **no remote** → nothing pushed; `main` is a local integration only.
- Studio `usg-desktop.rbxlx`, Edit; `sync --test` `{"synced":101}` rc 0; `tests` **rc 0 — all 11 modules pass**;
  `highway.spec` reports the same values: `passed=2, seedsPassed=100, expectedTrafficCount=54,
  minimumTrafficSpacingSlots=0.6694870414212347, roadSamples=3142, groundSamples=5, graphNodesChecked=20686`.
- Sealed sha256 BEFORE/AFTER: identical, same three hashes as run 1.
- Limit stated honestly: main sources are 16 commits ahead of the sealed candidate; this run syncs **main** sources
  into the sealed open place to prove the aligned spec passes on main too. The sealed place **file** was not rebuilt
  or changed; the sealed candidate run is run 1.

## Status

- A1 evidence is **clean on both required runs**: candidate-source run (run 1) and post-merge main run (run 2), with
  the sealed candidate artifacts byte-identical before and after (`84324d43…`/`c6651793…`).
- The spec change itself passed the independent checker (`team-d-a1-highway-spec-checker-verdict.md`).
- Per the Lead's conditions, **A1 is ready for the green declaration**; **B1 stays blocked until that declaration**.
  Nothing was built, re-pinned, published, uploaded or spent. Studio remains open in Edit.
