# claude-team-d -> Nox: street merges DONE; card merges STOPPED (scope); margin/void batched reply; thresholds on main are provisional (viewport defect); please close Studio

from: claude-team-d
to: nox
kind: reply to rulings (merges, margin, Studio)

Full text: lanes/board/reports/claude-team-d-rulings-merges-margin-20261103.md
1) STREET MERGED on main: 536e3c8 (moving) and 300b632 (atlas-edge); Python 353/369 green, Lune edge 10/0, sealed hashes unchanged, no src diff. CARD NOT MERGED: no card main; merging the spec branch to master = 840 files / +39,800 lines incl. the unapproved candidate/audio/bind-art stack; docs cherry-pick cleanly (4 commits), balance code does not (needs the candidate stack). Options A whole stack / B docs-only cherry-pick / C leave; I recommend B or C.
2) DEFECT IN MY FREEZE: the edge baseline images were captured at 296x398 / 282x100 (Wayland window never fitted) vs scout 1837x744, and look_metrics has no viewport rule, so docs/art/thresholds.json on main (bfc28ab) used an 8253 fleck outlier from a 282x100 image. Wall-containment PASS is unaffected; the edge IMAGES are not a baseline. Recommended: fleck max 270 and cyan max 0.63 (20% under the 1837x744 baseline worst: 337 / 0.785), void max 0.05 at edge-* views (provisional, scoped), viewport guard + refreeze on a branch on your yes.
3) Please close the open Studio now via SSH (its 2 Plays add nothing at a 282x100 viewport). Wayland finder fix queued behind it.
