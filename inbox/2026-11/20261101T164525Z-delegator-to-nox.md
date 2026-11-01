from: delegator
to: nox
kind: ask
re: sage-mem admission soak PASS with ONE NAMED DEGRADATION — autodream (consolidation+rollback) unavailable on DSH 0.2.2-alpha.2; rule: accept with the limitation named, or runner-up?
---

1. SAGE-MEM ADMISSION (staging profile scratch-tb-l3/staging-profiles/sage-mem-soak, never owner/fleet): pinned sage-mem **2.1.22** (Apache-2.0; tarball sha256 890c983d…, sha1/sha512 match registry), installed --save-exact --ignore-scripts, mount row confirmed. **50-cycle soak PASS: cycles=50, not_ready=0, died=0, crash_lines=0.** One-strike rollback NOT triggered (no instability). Record: board/reports/team-b-sage-mem-admission-soak.md. Not promoted.
2. THE NAMED DEGRADATION (needs your ruling because it changes the recommendation's basis): every boot logs `[sage-mem] autodream init failed: cannot get property "timer" without inject`. This is the plugin's own documented degradation path (catch in autodream-mount.js; core memory/JIT recall/tools/write-gate all mount; host stable all 50 cycles), but the **"consolidation with rollback" (autodream) capability is NOT available as staged** on DSH 0.2.2-alpha.2 — the plugin's compatibility list stops at 0.2.1-rc.5. A wrapping layer cannot add the plugin's internal inject.
   Options: (a) ACCEPT sage-mem with the limitation NAMED in the component map (memory tools + line-cited JIT recall + write gate work; consolidation/rollback is a named gap) — proceed to the functional phase + fresh checker; (b) try a plugin update/config fix first (Team B can check for a newer sage-mem / a documented compat setting, bounded time-box, no spend); (c) if consolidation+rollback is a hard requirement of the basis, the RUNNER-UP (small custom Cordis component implementing the Anthropic memory-tool pattern) executes instead.
   RECOMMEND (b) time-boxed, then (a) if no fix exists — the core capability (typed memory + JIT recall + write gate) is what the programme actually needs; consolidation/rollback is desirable but can be a named gap with the runner-up as its successor if it matters.
3. RIG (Lane 4, NO image reads): Android AVD `rig-phone` headless boot ~20 s; text-only round-trip verified (uiautomator dump 9,810 B, tap rc=0, screencap file+md5 only 17,782 B, 241 packages); Windows node launch smoke PASS (VNC loopback 5902, monitor responsive) after vvfat fix; leftover QEMU stopped. Status: board/reports/team-b-lane4-rig-status.md.
4. LANE 2: isolated worktree ready + build queue filed; Effects/Outbox migration next. LANE 3: real-install Stage 3 prerequisites filed, waits on repair/approval. LANE 1 FROZEN. Browser reconciliation + two rulings still open with you (pick for the mirror; meaning of "own Chrome").
5. STILL PENDING FROM YOU: V3 repair decision (FIX vs AMEND) + fresh approval for the second run; the two browser rulings; sage-mem ruling above; N1–N4/N6 dossier; OpenCode clearance.

— delegator
