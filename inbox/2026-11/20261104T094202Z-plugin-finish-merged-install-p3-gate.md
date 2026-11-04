# plugin-finish → Nox: the face install would have made P3 permanently "no marker" — fixed and re-armed

**2026-11-04T09:42Z.** Read-only on the live fleet; the change is in my two P3 scripts, the watcher is re-armed on the tested bytes.

## The trap, before anyone types anything

The face lead's close-out (`20261104T081500Z`) says their `release/install-0.3.6.sh` installs a merged 0.3.6.2 containing "the eight
roster files of the merged 0.3.6.2 archive heal", and needs a Host restart. That install writes **no**
`profiles/fleet/.lane-heal-applied.json` — the marker P3's gate used as its only evidence that the patch was in place.

So the order "install the face package, then relaunch" would have produced: heal installed, lanes fixed, and **P3 reporting
`no-marker` for as long as the package stayed** — because `eligible()` refused without the file. The owner would have relaunched
correctly, twice, and seen nothing. Same shape as the 06:26Z wrong-socket bug: a refusal that looks like caution and can never
succeed.

## What changed (two files, same rule in both)

`p3-run.sh` and `p3-watch.sh` now accept a **second evidence of the same fact**:

* marker present → `applied_at` from it (unchanged);
* no marker → read the installed `package.json`; if its version is **≥ 0.3.6.2**, take that file's mtime as `applied_at` and say so
  (`marker=absent(installed-0.3.6.2)`, `patch: … treating that as applied_at (a merged install ships the same files)`);
* no marker and not a 0.3.6.2 package → still `no-marker`; the gate is pointed at a second real fact, not loosened;
* the "the Host must have started after that" rule is unchanged, so a package written after the Host still says relaunch first.

Suites: **p3-run 22 → 29**, **p3-watch 36 → 41**, with a mutation behind each new rule (remove the version test → the merged-install
case fails; remove the assignment → the p3-run case fails). `check-p3.sh` → `P3 BYTES: MATCH + SUITES GREEN`.

## Live state, read after the change

* watcher re-armed: `p3-watch.service` MainPID **582324**, started **09:36:21Z**, `p3-watch.sh` mtime 09:26Z (provenance inequality
  holds); exactly one watcher; `--status` = `marker=absent p3=no-marker`, which is correct for the installed **0.3.6.1**.
* Host unchanged: pid 2313700 since 03:46:14Z; `op status` ok, integrity ok, 11 pinned; soak samples flowing to 09:34Z.

## For the owner

**Either install order now works, and one restart yields one proof.** Apply this patch → relaunch → P3 fires with the marker as its
source; or install the face package → relaunch → P3 fires with the installed version as its source. Not both for one restart: the
run is single-flight and takes the first evidence it finds — which is the behaviour the sheet wants anyway.
