# D-2 — Candidate identity, rebuilt and pinned by receipt (2026-11-01 08:53–09:15 ET)

Machine receipt: `board/probes/team-d/d2-candidate-identity-20261101.json`. Raw build log:
`board/probes/team-d/d2-rebuild-20261101.log`. Scratch rebuild trees: `~/.cache/team-d/rebuild-20261101/`
(live `build/` in both repos was **not** touched).

## 1 · Verdict

**Both candidates are now pinned by deterministic rebuild and a receipt. Candidate identity is CLOSED for both
jobs.** One correction to the brief and to the record: the street game's sealed candidate is **commit `b67abdd`**,
not current `HEAD` — rebuilding `HEAD` produces *different bytes*, so "rebuild from main before testing" would
silently test the wrong build. The card game's candidate is `55cef5d3…` as already preserved; its doc top line
(`be972098…`) is stale and now has a dated correction.

## 2 · Street game — `~/Work/projects/untitled-street-game`

Live artifacts (untouched, mtime 2026-10-21 03:42:13 ET):

```
84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce  build/release.rbxlx  1,990,548 B
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  build/test.rbxlx     2,403,691 B
```

Rebuild method: `git archive <commit> | tar -x` into a scratch dir, copy the vendored `.tools/bin/`, then
`python3 tools/usg build` twice in the same tree. Toolchain: Rojo 7.7.0 (`7ff204d7…`), Lune 0.10.5
(`4de91e1e…`) — identical in both repos and unchanged since October.

| Commit built | release sha256 | bytes | test sha256 | bytes | verdict |
|---|---|---|---|---|---|
| `b67abdd` (2026-10-21 03:22) | `84324d43…f6ce` | 1,990,548 | `c6651793…fe98` | 2,403,691 | **MATCH live, twice** |
| `b1e5ecd` (2026-10-21 03:08, tree-equal for the Rojo source set) | `84324d43…` | 1,990,548 | `c6651793…` | 2,403,691 | MATCH |
| `d1b75b6` / `2db69c8` | `14e07125…` | — | `97f7d8fd…` | — | no |
| `81044ed` | `adf57754…` | — | `afb920a6…` | — | no |
| `1bef02f` (STATE.md "current main") | `c34e4cc2…` | 1,992,286 | `43aa732d…` | 2,405,429 | no |
| **`8dfb204` = HEAD** | `c34e4cc2…` | 1,992,286 | `43aa732d…` | 2,405,429 | **no** |

Determinism: every rebuilt commit produced the **same hash on both runs** (same tree → same bytes). The difference
between `b67abdd` and `HEAD` is real source drift (`dddeafa`, `81044ed`, `f0c9299`, `8a5bc28`, `73bb404`,
`2da5aca`, `1bef02f`, `8dfb204` all landed after the 03:42 build), not a toolchain change.

**Pin: street sealed candidate = commit `b67abdd253029378a960a0899d06c672470e9072`** (tree-equal `b1e5ecd…`),
release `84324d43…`/1,990,548 B, test `c6651793…`/2,403,691 B, rebuilt twice. Consequence for the window: the test
place is already on disk and already copied to `~/.var/…/data/usg/usg-desktop.rbxlx` (`c6651793…`, verified
08:59 ET). **Do not run `tools/usg build` before the window** — it would replace the sealed artifacts with the
HEAD build. If a rebuild is ever needed, build the `b67abdd` tree, never HEAD.

Why the record said "main = 1bef02f": `docs/STATE.md` was written at `1bef02f` and the repo's `HEAD` later moved
to `8dfb204`; the sealed artifact was produced at `b67abdd` in between. The doc is right about the *source
baseline* and wrong about the *artifact*; this receipt is the artifact pin.

## 3 · Card game — `~/Work/projects/untitled-anime-card-game`

```
$ git rev-parse HEAD        -> 43574a56c9508cc17f2003b7fca29bd87ba1ca31   (branch preserve/candidate-55cef5d3)
$ git status --short        -> (empty)
$ sha256sum build/UntitledAnimeCardGame.rbxlx
55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9   520,846 B
```

Rebuild method: `git archive 43574a5` into scratch, copy `.tools/bin/`, `bash tools/build.sh` twice (the script
rojo-builds, stamps the source-manifest hash, and rebinds `ops/release/build-receipt.json`).

| Run | sha256 | bytes | receipt |
|---|---|---|---|
| build 1 | `55cef5d3…d4c9` | 520,846 | `place_sha256 = 55cef5d3…` |
| build 2 | `55cef5d3…d4c9` | 520,846 | same |

**Pin: card candidate = `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` / 520,846 B**, source state
preserved at branch `preserve/candidate-55cef5d3` (pin commit `add37b6`, tip `43574a5`), rebuilt twice. The stale
top line `be972098…` / 520,854 B is the superseded 2026-09-27 candidate; it is **not** recoverable from the repo
and does not need to be.

## 4 · Side work authorised this batch (audio tool) — candidate untouched

`tools/upload_audio.py` written to the D10 spec and committed on a **separate branch** so the preservation branch
is not moved: branch `prep/audio-upload-20261101`, commit `e7f885c` (parent `43574a5`). Contents: the tool, a
`.gitignore` line for `ops/release/target.json` (the project's own example says "never commit the filled file"),
and a dated candidate-of-record note in `CURRENT-STATE.md` (the R2-queued correction).

Proof the tool does not change the candidate: a scratch copy of tree `43574a5` **plus** `tools/upload_audio.py`
rebuilds to `55cef5d3…`/520,846 B (twice). The tool is not part of the Rojo source set.

## 5 · What is still needed

Nothing for identity. Two carry-forward notes: (a) the street candidate is pinned at `b67abdd`, so the window's
test place must stay `c6651793…`; (b) the card repo's working tree is now checked out on
`prep/audio-upload-20261101` (source identical to the candidate); the preservation branch and pin are intact.
