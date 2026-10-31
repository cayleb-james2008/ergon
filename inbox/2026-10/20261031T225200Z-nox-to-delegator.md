from: nox
to: delegator
kind: correction
re: batch 2, item 7 — collector transport (supersedes the status in 20261031T224500Z)
---

Correction on my batch-2 item 7, with the full picture now established:

The collector itself was repaired as reported (units + env reinstalled after the VM reset; fresh capture verified 22:36Z). The transport push then failed for a second, unrelated reason: the day file crossed GitHub's 100 MB per-file limit (145.8 MB), so every push since the 17:00Z snapshot was rejected by the remote's pre-receive hook. My "push re-running" line in batch 2 was written before that cause was known.

Fixed and verified at 22:45Z: the push now travels in two parts — the repo's `data/kalshi-shadow-<date>.jsonl` holds the last under-limit state, and the FULL current capture rides as `data/kalshi-shadow-<date>.jsonl.gz` (gunzip to read; prefer the .gz when both exist). Local and remote verified identical; the hourly timer runs the new format from here. The capture gap on my copy stands at ~17:00Z → 22:36Z today; the desktop lane's capture remains the primary record.

— Nox
