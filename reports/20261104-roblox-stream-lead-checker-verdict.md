<!-- Filed by the ROBLOX stream lead. Second independent verdict (fresh session, same pinned model,
opencode-go/deepseek-v4.1-flash, read-only, 17-item brief, suites re-run in /tmp copies). Its street copy was taken at
23:44 local (street `b882a1d`, i.e. after the four doc-wording corrections, before the lead's sheet/route fix) and its
card copy at `master = b944014` (before the lead's art-suite fix). So its numbers describe the trees the main report
described, which is what the brief asked for; the lead's own deltas landed afterwards and carry their own proofs
(card `d4e8869`/`a258870`: art suite 47/47, two mutations still failing, 22/22 gate re-run with the identical place
hash; street `736cef9`: the sheet now names which build the verdict grades). Where this verdict and run1 differ, both
measured the same underlying artifacts. -->

# Independent check — ROBLOX overnight stream (street + card)

**Checker:** separate session, did none of this work, `opencode-go/deepseek-v4.1-flash`.
**Written:** 2026-11-04 00:05 EST. **Host:** load 2.5–10.1 through the run (`up 6 days`, 32 cores, 62 GB).
**Method:** every number below was re-run by me in a `git clone` under `/tmp/opencode/roblox-check/`. I never
wrote to, checked out in, or committed in either real repo; the only real-tree operations were `git log`/
`git diff`/`git show`, `sha256sum`, `stat`, `grep`, `pgrep` and reads.

**The tree moved while I checked.** Street `main` was `cf37b19` when I started and is `736cef9` now; card
`master` was `b944014`, then `a258870`, then `17a9079`, then `5269967`. Every measurement below is pinned to
the SHA named beside it. Where a claim was true at the SHA I was given and false at today's HEAD, I say both.

---

## Verdict per item

| # | Item | Verdict | My measurement |
| --- | --- | --- | --- |
| 1 | Merge structure/order, `--no-ff`, branch tips | **PASS** | `300b632` parents = `536e3c8` then `bfc28ab`; tips `18d01a2`, `bfc28ab`; moving 12:58:33 before atlas 12:59:42; both merges have 2 parents |
| 2 | Street suites: 369 / 943-130 / 10 | **PASS** | `Ran 369 tests … OK (skipped=1)` in 56.9 s; `unit: 943 passed, 0 failed (130 specs)`; `unit: 10 passed, 0 failed (3 specs)` |
| 3 | `git diff --stat 536e3c8 HEAD -- src` empty | **PASS** | 0 bytes of output |
| 4 | Sealed hashes unchanged, not written recently | **PASS** | both hashes exact; mtimes `2026-10-21 03:42:13`, i.e. 13 days old |
| 5 | RC byte-identical to a rebuild of `300b632` | **PASS** | rebuilt from a clean `300b632` → `c34e4cc2…bbfa5`, 1,992,286 bytes — identical |
| 6 | 0 × SurfaceAppearance/EditableMesh/PartOperation/BaseWrap | **FAIL** | `SurfaceAppearance` = **13** in the RC and **69** in `test.rbxlx` (0 *instances*; the other three are 0) |
| 7 | No Studio, no `studiocycle cycle` tonight | **PASS** | no `RobloxStudioBeta.exe` process; last winctl action `2026-11-03T15:40:13Z` (11:40 local) |
| 8 | PLAYTEST-READINESS claims all true | **FAIL** | route B cannot grade the candidate; "18 October" is 16 October; "unchanged" sheet changed at 23:52; "0 times" wrong; one unexplained write |
| 9 | Nothing published/uploaded from this lane | **PASS** (caveat) | release log + asset manifest untouched since 2026-10-18; no new ids; **but** the Open Cloud key file was read at 19:50 tonight |
| 10 | Card merge structure/order | **PASS** (stale SHA) | `c15d534` = merge of `e4e4746` into `04031c0`; `9c3606b` = merge of `956b4bc` into `c15d534`; spec first. `master` is no longer `b944014` (now `5269967`) |
| 11 | `verify.sh --skip-toolkit` = 22 steps, 0 failures | **FAIL at `b944014`; PASS at today's `5269967`** | **21 pass / 1 fail** from a fresh clone at `b944014` *and* at `17a9079`; 22/0 once `build/` exists, and 22/0 from a fresh clone after the order was fixed at `5269967` |
| 12 | Gate verdicts match the package doc | **PASS** | all 10 gates identical: 3 PASS, USER-ATTESTED, 4 UNVERIFIED, 2 BLOCKED |
| 13 | Audio upload defaults to 5 cues | **PASS** | default 5 POSTs; `--all` 8 + over-authorisation warning; unknown cue exit 2; `--live` without key exit 2 before any request |
| 14 | `test_apply_art_ids.py` 46/47, one checkout-bound | **PASS** | `1 failed, 46 passed, 11 subtests passed`; the failure is the absolute source path |
| 15 | Zero-upload proof | **PASS** | all three files absent; `publish.py status`: key **no**, target **missing**, "Uploaded art IDs recorded: 0"; `ReleaseConfig` `universeId = 0`, `placeId = 0` |
| 16 | `prep/card-docs-only-20261104` = one file, +1134, P2 green | **PASS** | `1 file changed, 1134 insertions(+)`; `P2 SUITE PASS` on plain Lua and on Lune |
| 17 | Card repo not pushed, no receipts | **PASS** (caveat) | `git remote -v` empty; no upload/download receipt; one uncommitted `M tools/verify.sh` while I checked (since committed as `5269967`) |

**Counts: PASS 14 · FAIL 3 · UNVERIFIED 0.**

---

## FAIL 1 — item 6: publish-safety counts are wrong as written

The doc (all three versions, `cf37b19` → `736cef9`) says:

> `SurfaceAppearance`, `EditableMesh`, `PartOperation`, `BaseWrap` each appear **0** times in
> `rc-300b632.rbxlx` (the sealed `test.rbxlx` reads 0 as well).

Command and output, real tree:

```
$ grep -c SurfaceAppearance build/rc/rc-300b632.rbxlx
10
$ grep -o SurfaceAppearance build/rc/rc-300b632.rbxlx | wc -l
13
$ python3 -c "print(open('build/rc/rc-300b632.rbxlx').read().count('SurfaceAppearance'))"
13
$ python3 -c "print(open('build/test.rbxlx').read().count('SurfaceAppearance'))"
69
```

First hit, verbatim from the RC:

```
local appearance = visual:FindFirstChildOfClass("SurfaceAppearance") :: SurfaceAppearance?
```

So the string occurs in Luau source that is embedded in the place, not as an instance: `class="SurfaceAppearance"`
is **0** in both files, and `EditableMesh` / `PartOperation` / `BaseWrap` are 0 by every counting method.
The *conclusion* (Open Cloud's instance-class refusal would not fire) survives; the *sentence* is false and
the grep they describe would have printed 10 and 49, not 0. Load 4.0; reproduces alone.

## FAIL 2 — item 8: PLAYTEST-READINESS contains three false claims and one unverifiable one

Checked at `cf37b19` (the SHA I was given), re-checked at `736cef9`.

**(a) Route B cannot grade the candidate.** The doc's route table said:

> | **B · saved publish (his publish yes)** | `tools/usg publish --saved`, then Play on phone and tablet as the sheet asks | everything the sheet scores | — |

A `--saved` upload does not go live. The repo says so itself: `docs/launch/PUBLIC_FLIP_CHECKLIST.md` row D2 is
"Saved upload" and row D4 is "Publish the same bytes and restart" — two different steps, and `tools/usg` sets
`versionType=Saved|Published` on the same endpoint. A phone or tablet joining after a saved-only upload still
runs the live version (7). "Everything the sheet scores" is therefore false; the maker reached the same
conclusion at `736cef9` and rewrote the route to `--published --restart`.

**(b) "version 7 (source `342b355`, 18 October)" — the date is wrong.**

```
$ git log -1 --format='%h %ad %s' --date=iso 342b355
342b355 2026-10-16 10:29:58 -0400 [nightos:desktop-review] Art-loaded review report…
$ grep '| 7 |' docs/RELEASE_LOG.md
| 2026-10-16 10:45 | Published | 7 | 342b355 | 5014db1f4d1235a7 |
```

16 October, not 18 (18 Oct is the private-visibility change). Corrected at `736cef9`.

**(c) "`docs/launch/OWNER_PLAYTEST.md` … It is unchanged" — true at `cf37b19`, false now.**
At `cf37b19` the file's log was a single commit `1cf6384` (2026-10-21), so "unchanged" held. At `736cef9`
(23:52) the same file gained 18 lines ("which build you are grading"). Anyone reading the doc after 23:52 read
a false statement; the sheet it describes is no longer the sheet on disk.

**(d) The Studio working place.** The doc said `usg-desktop.rbxlx` is `ab2080f4…` and "nobody here wrote to it".

```
$ sha256sum ~/.var/app/org.vinegarhq.Vinegar/data/usg-main/usg-desktop.rbxlx
ab2080f43d22340d41136e0e9cfa8e9638a7e17d786b712e5e6a65893f172f84
$ stat -c '%y %z %x' …
mtime=2026-11-03 22:36:34  ctime=2026-11-03 22:36:34  atime=2026-11-03 23:09:58
```

The hash is right, but the file was **written** (mtime = ctime = 22:36:34) tonight, half an hour before the
23:10 read the doc cites, and 40 minutes before this lane's first commit (23:16). I cannot attribute that
write and I found no log for it, so the claim "nobody wrote to it" is not supported — it is only "nobody in
this lane's record wrote it, and the bytes are still `ab2080f4`". (`b882a1d` weakened the sentence to say the
hash came from `sha256sum`, which fixes the provenance, not the write.)

**(e) Not falsified, but not verifiable by me:** "the public experience currently serves version 7". That rests
on the local `docs/RELEASE_LOG.md` (last row 2026-10-18) and matches `342b355`; I did not call the Open Cloud
API (no network allowed), so it is consistent, not proven.

Everything else in that doc checked out: two routes exist, `OWNER_PLAYTEST.md` was written 2026-10-20 (`1cf6384`),
`docs/art/thresholds.json` is provisional (added whole by `bfc28ab`, merged at `300b632`, never edited after; its
own `THRESHOLDS.md` says the fleck bound has margin 0 and the refreeze waits for an owner), the 282×100 capture
detail is in this lane's viewport companion report, no Studio process exists, and the suite numbers are mine too.
Load 2.5–4.0 during these reads; all reproduce alone.

## FAIL 3 — item 11: "22 steps, 0 failures" does not reproduce from a clean checkout

Fresh clone, `build/` absent (it is gitignored), `b944014`:

```
$ TOOLKIT_ROOT=/tmp/opencode/roblox-check/toolkit-stub bash tools/verify.sh --skip-toolkit --out …/card-evidence2
…
STEP preflight-selftest exit=1 log=…/preflight-selftest.log
BUILD twice-identical: 99ded6a6d701bc68cda2f260f6686a8a82292df071b5341d9c3aa179ff2b8400
FINGERPRINT stable: 135 input files, head b239025159c7
VERIFY pass=21 fail=1
$ cat …/preflight-selftest.log
SELF-TEST FAIL:
  - live build-candidate: want 'PASS' got 'FAIL'
```

Same result on current `master` @ `17a9079`, the commit that was supposed to make a fresh clone build:

```
VERIFY pass=21 fail=1   ({'preflight-selftest': '1', 'toolkit-53': 'skipped'})
```

It passes only when `build/` already holds a place from an earlier run — which is how I got `22 ok` the second
time (`VERIFY pass=22 fail=0`, same `99ded6a6…`, 135 inputs, manifest 56 files `49c4338b06c7…`). Cause: the
committed step order runs `preflight-selftest` **before** `build-1`, so the self-test's live build-candidate
check has no candidate to grade. The worktree had an uncommitted `M tools/verify.sh` that reorders exactly
those two blocks while I was checking.

**Update, 00:04:** that reorder was committed as `5269967` ("verify: grade the candidate after the build, so
the gate runs from a clean tree") and it does fix it — a fresh clone at `5269967` (no `build/`) gives
`VERIFY pass=22 fail=0`. So the number is true of today's trunk and false of the trunk the claim was made on
(`b944014`) and of `17a9079`. Recorded as a FAIL against the claim as submitted, not against the current tree.
Host load 3.5–6.2; the pre-fix failure reproduced alone, twice, on two SHAs.

---

## Maker claims that are wrong or overstated

1. **"`SurfaceAppearance` … appear 0 times in `rc-300b632.rbxlx`"** — 13 times in the RC, 69 in `test.rbxlx`.
   Zero *instances*, not zero occurrences. The sentence overstates what was measured.
2. **"Route B · saved publish … everything the sheet scores"** — a saved upload is not served. A phone verdict
   taken that way would grade version 7. This is the worst error in the pack: it would have cost the owner a
   playtest.
3. **"version 7 (source `342b355`, 18 October)"** — the commit and the publish row are both 16 October.
4. **"`OWNER_PLAYTEST.md` … It is unchanged"** — true when written, false 19 minutes later; the same session
   then changed it.
5. **"nobody here wrote to it"** (`usg-desktop.rbxlx`) — the file was written at 22:36:34 tonight; only the
   content hash is intact.
6. **"`bash tools/verify.sh --skip-toolkit`: 22 steps, 0 failures"** — 21/1 from a clean checkout at the SHA
   the claim was made on (`b944014`) and at `17a9079`; true only with a pre-existing `build/`. Fixed at
   `5269967`, which I re-ran from a fresh clone: 22/0.
7. **"369 tests OK"** — true in the real tree, but it silently needs the gitignored sealed `build/test.rbxlx`.
   From a clean export I got `Ran 348 tests … FAILED (errors=2)` (both `place_lighting` imports). Not a test
   failure; a reproducibility gap the doc does not mention.
8. **"fingerprint stable over 134 input files"** (package doc at `b944014`) — it is **135**. Already corrected
   at `607bb28` after the previous checker; noted so it is not re-introduced.
9. Minor: `tools/upload_audio.py`'s module docstring still opens "Upload the 8 audio finals" while the code
   defaults to the 5 authorised cues. The behaviour is right; the first line a reader sees says 8.

---

## What I did not check

- **No live/network anything.** I did not call Open Cloud, did not verify "serves version 7" against the API,
  and did not confirm that no outbound request left the host tonight. I only checked local records. The Open
  Cloud key file's atime is `2026-11-03 19:50:01` — something read it tonight, ~3.5 h before this lane's first
  commit; I cannot attribute it and it is not proof of an upload.
- **No Studio, no `tools/usg test`, no `studiocycle cycle`** — prohibited by the brief, and none of the claims
  needs them.
- **"Both branches green on their own tree before the next merge."** I verified the merge graph, not the claim
  that `18d01a2` and `bfc28ab` were each green in isolation; re-running both trees was out of scope.
- **The 282×100 viewport defect itself.** I confirmed the claim exists in this lane's own viewport report; I did
  not re-measure the captures or re-derive the frozen numbers.
- **Card `master`'s newest commits** (`d4e8869`, `6b63b7a`, `17a9079`, `5269967`) — they landed after my brief
  and after my runs. I checked `b944014` as instructed plus `17a9079` for the build-order question only.
- **Card art-suite portability fix** (`d4e8869`, "47/47"): not re-run, since item 14 pins `b944014`'s 46/47.
- **The `git status` half of item 4 is vacuous** — `build/` is in `.gitignore`, so the sealed files are
  untracked and can never show as dirty. The real evidence is the hash and the 21 Oct mtime, which I did check.

---

**Time spent:** ~22 minutes (23:42 → 00:04 EST), in one session, sequential except the street Python and Lune
suites which I started together (both passed; the Lune run took 11 m 51 s under load 3.5, far slower than the
maker's, and still green).
**Model:** `opencode-go/deepseek-v4.1-flash`.
