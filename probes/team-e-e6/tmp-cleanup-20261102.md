# /tmp CLEANUP — 2026-11-02T17:03 ET (22:03Z) — bounded, no root
**Why:** `/tmp` is a 32 GB tmpfs that sat at **90% (28 GB used, 3.4 GB free)**; the ~04:50–05:00 ET tool-write failures ("UNKNOWN: unknown error, write") match tmpfs pressure. Goal: remove only provably stale scratch.
**Gate used for every deletion:** `lsof +D`/`lsof -nP | grep "<dir>/"` = **0 handles**, no process cwd/root under the dir (`/proc/*/cwd` scan), mtime **> 2 h** (cutoff 15:02 ET), realpath under `/tmp`, not in the protected set. **`fuser -m` was NOT used** (it false-positives on a single tmpfs mount — Lead's correction).
**No root, no service changes, no touching live session files or credential content.**

## Result
| | Before | After |
|---|---|---|
| `/tmp` used | **28 G (90%)** | **21 G (66%)** |
| free | 3.4 G | **11 G** |
| top-level entries | 20,938 | 19,328 |
| **freed (df delta)** | — | **≈7 GB** |

## Removed (all gate-clean; reports/evidence filed on the board)
- **11 checker working copies — 7,087 MB:** `checker-d1`, `checker-d2` (1,091 MB each), `checker-l9`/`-mut`, `checker-diag2`/`-mut`, `checker-r3`/`-mut`, `checker-fix19`/`-mut` (546/544 MB each), `checker-a639fa2` (541 MB). Reports: `reports/checker-layer9-delta.md`, `checker-diag2-delta.md`, `checker-round3-delta.md`, `checker-fix19-delta.md`, `checker-b2-delta2.md`, `checker-abort-i-delta.md` + `probes/checker-*/`.
- **Ergon scratch — 788 MB:** `ergon-gate-462277c` (722 MB), `ergon-pr11-verify-20261101T164139Z` (66 MB).
- **~1,596 stale tiny Ergon sentinel dirs** (`ergon-budget-proxy-*`, `ergon-lock-*`, `ergon-join-*`, `ergon-dispatch-locklost-*`; 0–216 KB each, 30 Oct–1 Nov) — same gate, removed to cut the entry count.
- **My own lsof capture** `/tmp/.tmpclean-lsof.txt` (117 MB) — deleted after the gate; not part of the host's state.

## Skipped / kept (named, with reason)
- **NOT removable without root — 6 Ergon copies owned by uid 100000 (container subuid):** `ergon-orderb-final`, `ergon-orderb-d6`, `ergon-final-test`, `ergon-clean-v3`, `ergon-clean-v2`, `ergon-clean-user` — 629 MB each (≈3.8 GB). `/tmp` sticky bit + non-owner ⇒ `Operation not permitted` / `Permission denied`. **Left intact; no root attempted.**
- **~496 tiny Ergon sentinel dirs, same uid 100000** — left (same reason).
- **Two cayleb-owned copies with foreign-owned files inside:** `ergon-portable-successor-run`, `ergon-portable-aggregate-deps` (622 MB each) — `rm` removed the deletable entries but was denied on root/100000-owned files under `node_modules`; left (stale scratch, **0 handles**, nothing live uses them).
- **Protected, never touched:** `/tmp/claude-1000` (live Claude Desktop temp, 6.46 GB, growing), `/tmp/.wine-1000` (live Wine server socket), current-session `/tmp/dsh-*`, `/tmp/checker-fixb` + `-mut` (mtime < 2 h — today's active checker work).
- **Out of the named class → kept:** `cc-demo`/`cc-pristine`/`cc-065e0fc` (Roblox project copies, 1.62 GB total), `uacg-*` (5 Roblox copies, 1.83 GB), `g3-exec` (openbell copy, 309 MB), `node-compile-cache` (306 MB), `fulltest-022` (69 MB). Not deleted because the task bounded removal to checker copies whose reports are filed + ergon scratch; these are the next-best candidates **if the Lead names them**.

## Risk notes
- No live process, session, service or socket was touched; every removed path had 0 handles and was >2 h old.
- The remaining stale scratch is **permission-bound, not usage-bound**: a root act or an owner-side cleanup of uid-100000 trees is the only way to reclaim the ~5 GB left; not authorised here.
- `/tmp` is now at 66% with 11 GB free — the immediate write-failure pressure class is relieved. A second pass can remove the cc-/uacg-/g3 copies if the Lead rules them in.
