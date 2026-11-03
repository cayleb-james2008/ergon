# claude-team-d -> Nox: atlas:edge parts 2/3 and 3/3 PASS; part 1 needs a re-run; two of my Studios are open and I was refused the kill

from: claude-team-d
to: nox
kind: completion + hard blocker (small)

atlas:edge (flag-off baseline, seed 2134783819) now runs live, in 3 parts (the 56 runs cannot fit one 588 s window). Branch prep/atlas-edge-diag-20261103 @ dae2abc, nothing merged.
- Part 2/3 PASS (run 20261103T161812Z-atlas-edge, 16 runs, 0 reached the wall). Part 3/3 PASS (run 20261103T163557Z-atlas-edge, 16 runs, 5 reached the wall). Part 1/3 (Play 33) drove and reported all 24 runs (4 reached the wall at 2-6 studs, all in bounds) but failed my own over-strict validator; the corrected validator (dae2abc) passes that saved data offline. Part 1 still needs a fresh Play to count: it needs a new Studio opening.
- Root causes fixed since the last mail: wrong GameService path; client half never started; car name (production Driving attaches only to Player_<id>); keyed heading; MenuOpen zeroing the throttle; report size; stale samples; time budget (3 parts; two checkers).
- Gate fix per your note applied (argv[0]-anchored pgrep); the Play 35 relaunch ran with it.

HARD BLOCKER: I ran `studiocycle cycle` for the next opening. Studio is now a native Wayland window (class robloxstudiobeta.exe), which studiocycle's window finder does not see, so close said "no-studio-window" and open launched a SECOND Studio. Two of my Vinegar Studios (pids 4142639, 270149) are open on the new place. `flatpak kill org.vinegarhq.Vinegar` was refused by the permission classifier; I did not work around it. Please close both (or tell me which route you allow); then I run part 1/3 in a fresh opening. The place file is unchanged (new-place hash 2df9cdb4...; sealed hashes c6651793.../84324d43... unchanged).
