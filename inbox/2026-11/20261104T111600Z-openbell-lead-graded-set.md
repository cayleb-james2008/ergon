# OpenBell day lead — Watchover graded set (read this instead of older pointers)

From: OPENBELL day lead (replacement session, fallback engine)
To: Nox / owner
When: 2026-11-04 ~11:15 UTC

Older board records name render sets that no longer exist (20261104T1030Z, 1245Z, 1345Z — removed). The ONE graded set at HEAD 4409829 is:

**renders/20261104T1055Z/** in /home/cayleb/workspace/openbell-watchover (branch design/openbell-watchover), committed at 4409829 (fixture build at 3d34af7).

- 446 rows, 420 pass / 26 fail, 179 PNGs, 118/118 expected captures, inventory 179/179, dirty:false
- Page digest 0c357fc23fa4a72a2ea1d2e866d17231f99145448243936420db411b52f2eb7d (332,145 B / 100,955 gzip) — byte-identical to the checker-verified 0745Z page; 179/179 PNGs byte-identical to 0745Z; 0 verdict differences
- The 26 failures are exactly the two named spec conflicts (ruling request pending Nox): 24 x A8 viewport (plates below the fold at 1440x900, 203-353 px) and 2 x A29 (iii) (ring contrast + adjacent bands, unreachable inside the spec's own 3-step tone-lift remedy, measured)
- Fixture build: `npm run fixtures:build` (tools/fixture-build.mjs) — builds, stages and serves all seven SPEC §20 roots; mutation-proved both ways (a mutated root fails by name, exits NOT READY). SERVE.md in dist/fixture-build/ carries the one-command serve line per root.

NOT a cutover: :8791 still serves the old dashboard (849e2c1c, 219,311 B, zero watchover/scene- references). Old dashboard stays live until the owner grades.

Checker verdicts on the new HEAD (4409829) are in flight now: fixture-build verifier (verdict-A1) and 1055Z checker (checker-watchover-verdict-1055Z). This filing will be superseded by their verdicts; the page digest above will not change — the stable identity of the graded pack is 0c357fc2, not a commit hash.

Slice B (futures sealed record for 3 Nov, owed by Team C) is deferred on this engine per order and stays queued for the DeepSeek engine return. Slice C pack: checker-ready (11/9/7), biggest blocker is the missing live-trader mode on the one-app (B1 decision needs owner + Nox) — filed 20261104T110019Z-openbell-sliceC.md; its independent checker and 3 repair-delta drafts are in flight (verdict-C1, repairs/).
