# TEAM B → RELEASE LANE — Stage 2 / 0.2.2 handoff bundle (DRAFT — INTEGRATION BRANCH PUSHED AND VERIFIED; NOT A GATE PASS)

Written by `fleet-lead` (Team B); updated 2026-10-31 20:15 ET. Division (accepted, Outis mail 22:52Z): **Team B owns verification, package assembly and the integrated candidate; the release lane (Outis) owns release and desktop-install legs — no parallel candidate gate or package assembly.** Gate report (`board/reports/stage2-gate-report-FINAL.md`) is the authoritative leg table; desktop-residue evidence index `board/probes/team-b-final-tree/MANIFEST.md`.

## CURRENT STATE — integration branch pushed, remote head verified

| Field | Value |
|---|---|
| Branch | **`integration/0.2.2`** — remote `origin/integration/0.2.2`, local worktree `~/workspace/harness-build/dsh-fleet-integration` |
| Head (local == remote, verified) | **`cf93107bc6e31ed265332b005d75b63d8acb300e`** |
| Tree SHA | `e15c17a3c93919de6b627e791a469dd9ddeefb28` |
| Package subtree SHA | `61ce54e44b3a42ee09329f661d4d23bf78db0025` |
| Package version | **0.2.2** (`package/package.json` sha256 `ef93187aec289d70bd29446c66c5b63abe44f78f4d2342871af2338274b4f52c`) |
| Push | `git push origin integration/0.2.2` (no force), **19:49:30 ET**; `git ls-remote origin` after: `integration/0.2.2 = cf93107…`, `main = 4a8a1d3…` (unchanged), `next = 0769797…` (unchanged) |
| Base / integrated | accepted remote main `4a8a1d3f080416ccb545bee60655117584c960fd`; `next` `0769797e2d32d89d3c22d0bd1cb2d94aef92ca52` integrated on top (merge `be4a35e`, both parents); `abfb842` excluded |
| Commits | `be4a35e` integration merge (walker held back) → `ee14591` walker change (own commit) → `95804ba` version bump 0.2.2 → `cf93107` landing note |
| Working tree | clean |

**Note on the landing note:** its "nothing pushed" line was true when committed; the branch was pushed afterwards on the Lead's authorisation. This board record is the current statement; the certified head `cf93107` was not rewritten.

## Required bundle items

| # | Item | Status (19:50 ET) | Where / note |
|---|---|---|---|
| 1 | Exact final candidate SHA | **INTEGRATION HEAD `cf93107bc6e31ed265332b005d75b63d8acb300e`** (remote-verified) — **FORMAL STAGE 2 PASS SIGNED** by the independent checker (21:32 ET) on this tree; **Mission 1 close-out not complete** (row 18 install outstanding) | Base `4a8a1d3`; integrated `0769797`; merge `be4a35e`; walker `ee14591`; bump `95804ba`; note `cf93107`. Sign-off: `board/reports/fleet-checker-stage2-signoff.md`; final gate report: `board/reports/stage2-gate-report-FINAL.md`. |
| 2 | Separately reviewed walker change | **LANDED AS ITS OWN COMMIT `ee14591`; INDEPENDENT REVIEW RECORD LANDED (B5) — `board/reports/fleet-checker-walker-review-ee14591.md`, fleet-checker, 2026-10-31 20:15 ET; verdict: SAFE and CORRECTLY SCOPED.** Test-only (both files under `package/src/orders/test/`; the shipped 0.2.2 tarball contains no `test/` dir or walker files — verified by extraction); raise formula bounded **[0.5, 100000]** with boundary cases checked; hold path/tiering/benign confirm unchanged; **no assertion code removed** (52 finding-kind set unchanged; only a comment claim narrowed and honestly declared); four-way differential confirms this hunk is the discriminating change. Not committed into the certified head `cf93107` (no head change). | Patch `board/probes/walker-raise-0769797.patch`, sha256 `3208e48558dc22b3e8dbc44fe586b084ec2f99f3169750320b51ac2911432029` (4 hunks); review record path above. Known limits to carry: aggregate envelope no longer walker-asserted (replacement = composed spend regression tests); pinned R6-10 test is a fixture-path guard, not a product regression; cite the review file in the gate pack. |
| 3 | Candidate-bound gate results | **SUBSTANTIVE, CONDITIONAL** | Nox's independent candidate run on `0769797` (manager mail 23:05Z: suites/race/lint green, client disposition proven, seed 1546 fixed, n400 1001–2000 green on his vantage); desktop residue on `cf93107` green for composed-host plugin (core 24/24, policy 7/7, ledger 6/6, orders 18/18, lessons 8/8), composed-host e2e 15/15, gate-sandbox 52/52, gate-egress 14/14, gate-fsguard 98/98, client conformance 10/10, pinned single-seed R6-10 1/1; host-judged mutants: policy R6 32/32 killed, orders R6 16/16 killed, full orders mutation in flight. **Open:** full n400 sweep (1001–2000 @400), other walker ranges, rehearsals — see below. |
| 4 | Fresh-checker verdict | **SIGNED — FORMAL STAGE 2 PASS** | `board/reports/fleet-checker-stage2-signoff.md` (2026-10-31 21:32 ET), over rows 1–10, 14, 16, 19; rows 11/12/13/18/15/16 declared open as ruled; three B5 known limits carried; `abfb842` excluded. **Mission 1 close-out not claimed.** Final report: `board/reports/stage2-gate-report-FINAL.md`. |
| 5 | 0.2.2 artifact name + SHA-256 | **READY** | **`cayleb-dsh-fleet-0.2.2.tgz`**, size 210,358 B, sha256 **`37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`**; sidecar `cayleb-dsh-fleet-0.2.2.tgz.sha256`; packed from the `cf93107` tree (`npm pack` in `package/`, no prepack scripts). **Complete package file list (one archive path per line, 82 files): `board/reports/team-b-package-file-list-0.2.2.txt`** — independently reviewed: **PENDING** (the list is generated from the tarball; 0 files differ from the tree, 109 tracked files are excluded by the `files[]` filter — provenance statement below). |
| 6 | Install + running-identity readback (release lane) | **NOT STARTED — release lane owns it** | Gate row 18 (RELEASE-IDENTITY). See below. |

**Generated-file provenance (for item 5):** the tarball was produced by `npm pack` from the clean `cf93107` tree; `package.json` declares no `prepack`/`prepare` scripts; every one of the 82 archive files is a file of the `cf93107` tree (`git ls-files package` comparison: 0 generated/untracked entries). The 109 tracked files not in the archive are the normal `files[]` filter (tests, docs, NOTES, demo, etc.).

## CHECKER INSTRUCTIONS (for `fleet-checker`, desktop-residue certification on cf93107)

- **Reproducible from remote:** `git fetch origin integration/0.2.2 && git rev-parse origin/integration/0.2.2` → must be `cf93107bc6e31ed265332b005d75b63d8acb300e`; tree `e15c17a3c93919de6b627e791a469dd9ddeefb28`; package subtree `61ce54e44b3a42ee09329f661d4d23bf78db0025`; version 0.2.2.
- **Named local path:** `~/workspace/harness-build/dsh-fleet-integration` (branch `integration/0.2.2`, clean at `cf93107`).
- **Evidence manifest:** `board/probes/team-b-final-tree/MANIFEST.md` (every leg records commit/tree/clean/version/package.json sha/command/raw output/timestamp, bound to `cf93107`).
- **Artifact:** `~/workspace/harness-build/dsh-fleet-integration/dist/cayleb-dsh-fleet-0.2.2.tgz` sha256 `37e3a542…`; file list `board/reports/team-b-package-file-list-0.2.2.txt`.
- **Still OPEN on cf93107 — must not be inferred from the single-seed pass:** full n400 sweep (seeds 1001–2000 @400), n200 (1–1000 @200), biased 500 @300, crash 500 @300, the rehearsals, the release-identity/install leg, and the fresh-checker veto itself.

## RELEASE-IDENTITY requirement (gate cannot close without it)

Outis's preflight finding: **neither `main` `4a8a1d3` nor `next` `0769797`'s Stage 2 check calls `plugin-add`**, so a tarball race pass or sidecar match **cannot** establish that the service loaded 0.2.2. The release lane must:
1. install the exact accepted package through the **authorised documented wrapper** (command + full output recorded);
2. record the artefact name + SHA-256 it installed (`cayleb-dsh-fleet-0.2.2.tgz`, `37e3a542…`);
3. **independently establish the running package identity** from a permitted scope (unit/service identity, loaded package/version readback) **before** the formal check;
4. show the readback matches `cf93107` / tree `e15c17a3…`.

The fresh checker verifies (1)–(4) before the formal Stage 2 check runs. No tarball/race pass or sidecar match substitutes for (3)–(4).

## RELEASE MERGE INSTRUCTIONS (for the release lane; no force-push)

- Source: `origin/integration/0.2.2` @ `cf93107bc6e31ed265332b005d75b63d8acb300e` (single named integration branch per Nox's ruling; neither lane force-pushes it).
- Merge into `main` **without force** after the fresh checker's decision **naming `cf93107`**; record the merge commit and its parents. Expected parent chain: `be4a35e` (parents `4a8a1d3` + `0769797`) → `ee14591` (walker) → `95804ba` (bump) → `cf93107` (landing note).
- `abfb842` stays excluded (not an ancestor of `cf93107`; verified).
- If any part of the merge is rejected, report it rather than forcing; no main/next writes until then.

## Status carried honestly (no gate pass)

- **Release-identity B2: BLOCKED ON THE OWNER** — install paused under Cayleb's Wait; **no install exists**; `fleet-dsh` still runs **0.2.1**; `/var/lib/fleet-drop` holds `0.2.0`/`0.2.1` only.
- **Rehearsals: RAN on `cf93107`** — scratch Stage 2 **PASS 60/60**; scratch Stage 3 default mode **73 PASS / 1 FAIL / 24 MANUAL**, the single FAIL (`3.4c` journald fresh-head) **classified SCRATCH LIMIT** (shared host journal; `3.4a` chain verification passed); evidence index `board/probes/team-b-final-tree/rehearsals/INDEX.md`. **Leg open** until the checker verifies; the journald leg needs the real install.
- **Five-high repro: UNVERIFIED** — no valid verdict.
- **Orders mutation full run: in progress by the fresh checker (B1)**; supersedes Team B's host-judged full run, which died mid-run and is not evidence.
- **`abfb842`: EXCLUDED** from the integrated history.
- **Formal Stage 2 PASS: SIGNED** — `board/reports/fleet-checker-stage2-signoff.md` (21:32 ET) on `cf93107`/`e15c17a3`/0.2.2; **Mission 1 close-out is NOT complete** (row 18 install + Nox's merge ruling outstanding). Final gate report: `board/reports/stage2-gate-report-FINAL.md`.
- **No merge to `main`; nothing here is a Mission-1 close-out claim.**

## Desktop-vantage evidence already filed (for the checker, not a verdict)

`board/probes/team-b-desktop-vantage-20261031T1847Z.md` — installed unit state (`fleet-dsh` active/running, MainPID 2602074, ExecStart `/var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091`, `User=fleet`, InvocationID `df02e934…`) and bus reachability (`/run/fleet/control.sock` reachable from `cayleb`; `/run/fleet/egress.sock` denied). Installed tree byte identity is **not** readable from the `cayleb` vantage — that is the release-identity leg's job.
