# TEAM D — pickup pack for the DeepSeek lane (Claude lane stops here, 3 Nov 2026 ~14:45 ET)

## Merges (your rulings of 12:57)
- DONE on street `main`: `536e3c8` (moving association, was 18d01a2) and `300b632` (atlas:edge chain, was bfc28ab). Python tools 353 then 369 green, Lune edge 10/0, sealed hashes unchanged (build/test.rbxlx c6651793…, release 84324d43…, Vinegar copy c6651793…), no src diff. Local, not pushed.
- STILL PENDING, NOT STARTED: card branches. No card `main`; trunk is `master` @ 04031c0. `prep/card-game-design-spec` (e4e4746) is stacked on the unapproved candidate/audio/bind-art prep: vs master it is 840 files, +39,800 lines. The 4 docs commits (84c4018, 6b9c985, 448cec3, e4e4746) cherry-pick cleanly onto master; the balance code (3d21456, 956b4bc) does not (needs the candidate stack). Options: A whole stack, B docs-only cherry-pick, C leave. Recommended B or C. Δ2 and Δ16 stay OPEN (owner's sitting bundle).

## Numbers you asked for (full basis: reports/claude-team-d-rulings-merges-margin-20261103.md)
- Baseline (scout, 1837×744, 18 valid street views, deterministic): fleck energy `all` max 336.9 / p90 199 / median 75; cyan fraction `all` max 0.785 (one view) / p90 0.107 / median 0.006; mid/near under 0.6 in 7 of 18; facade repetition worst 13.
- Margin protects against calling a pass on a change smaller than the measurement resolves or caused by a different shot (window/opening), not random noise (identical to 3-4 digits across Plays 33/34/35). Recommended margin 20% under the baseline worst view: **fleck `all` max 270, cyan `all` max 0.63**. Rejected 0% (unchanged baseline passes) and 50%.
- Void fraction: recommended **max 0.05 at every `edge-*` view**, provisional (no valid-viewport edge baseline yet; invalid-viewport edge numbers 0.03-1.0). Needs a view-scoped bound in the threshold schema, else it fails every street view (0.03-0.9).
- DEFECT IN MY FREEZE: `docs/art/thresholds.json` on main (bfc28ab) used edge captures at 296×398 and 282×100 (Wayland window never fitted) next to scout at 1837×744, so its fleck max 8253 is an outlier from a 282×100 image. It is PROVISIONAL. Do not use it as a gate. Fix, on your yes: viewport guard in look_metrics (one viewport per pack, minimum about 1600×700), scoped void bound, refreeze from comparable captures, tests, fresh checker, on a branch.

## Studio state
- ONE contained Vinegar Studio is still OPEN (opening B, viewport 282×100, 1 of 3 Plays used by Play 36), on the new place (sha 2df9cdb4…). Its 2 remaining Plays are useless for images; I recommended you close it by SSH. Not closed by me (`flatpak kill` was refused; `studiocycle` cannot see the Wayland window, class robloxstudiobeta.exe).
- NEVER run `studiocycle cycle`: close finds no window, then open launches a SECOND Studio. Open only from a known-closed Studio.
- Launch env that works: HYPRLAND_INSTANCE_SIGNATURE=<ls /run/user/1000/hypr>, WAYLAND_DISPLAY=wayland-1, DISPLAY=:0, STUDIOCYCLE_PLACE=~/.var/app/org.vinegarhq.Vinegar/data/usg-main/usg-desktop.rbxlx; runner env USG_PLACE_FILE=<same file>, USG_BUILD_PLACE=build/main-places/test-main-0c325f8.rbxlx. Studio count gate: `pgrep -f '^[^ ]*/RobloxStudioBeta\.exe'` (anchored; `pgrep -f name` self-counts, `-x` fails >15 chars). Other lanes' W6BS scratch Studios can trip the runner's one-Studio gate: wait, never touch.

## Queue for the next lane
1. Successor item: studiocycle Wayland window-finder gap (fit and close both depend on it; test only with Studio closed).
2. Viewport guard + refreeze + scoped void bound (above), then re-capture the atlas:edge views at 1837×744 (3 parts, `--edge-part I/3`, ~3 Plays; wall drive-in already PASSED: runs 20261103T165617Z / 161812Z / 163557Z).
3. Card merge decision (A/B/C), Δ2 / Δ16 with the owner.
4. Open evidence: Plays used this opening 1/3; all board filings under lanes/board/reports/claude-team-d-*.md; run dirs under street docs/verify/2026-10-19/vinegar/; memory notes team-d-street-abort-contact-passed.md and pgrep-f-counts-its-own-shell.md.
