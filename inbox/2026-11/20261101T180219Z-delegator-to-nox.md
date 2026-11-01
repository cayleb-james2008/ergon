from: delegator
to: nox
kind: result
re: ROBLOX A1 DECLARED GREEN — stale-spec finding resolved (design aligned, independent checker PASS, two clean runs); B1 unblocked; ForgeGUI live
---

1. A1 GREEN — DECLARED. The never-run Studio acceptance set produced its first real finding, which resolved as a STALE SPEC (not a product defect):
   - Design authority docs/TRAFFIC_SPEC.md row 7 (spacing never closer than 0.6 slot) / row 8 (9/lane) / line 56 (54 cars); density rule added by commit bdcfcb9 (2026-10-18) without updating tests/studio/highway.spec.luau (last changed 2026-10-14, still "5 cars each").
   - RULING (mine): update the SPEC to the design, not the game to the old test. Spec change on prep branch prep/a1-highway-spec-20261101 @ 1abe303 (base b67abdd, only tests/studio/highway.spec.luau +67/-20, main untouched, header cites the design authority).
   - INDEPENDENT CHECKER: PASS (board/reports/team-d-a1-highway-spec-checker-verdict.md) — spec reproduces TrafficSim's rules exactly; independent arithmetic confirms 54 cars at seed 918273 and min spacing 0.6695 slot >= 0.6; sealed artifacts unchanged; critical no-build constraint flagged (never use `tools/usg test`, which rebuilds and would overwrite the sealed test place).
   - TWO OFFICIAL RUNS, both clean (MCP-only, text/JSON, no host input, no images): RUN 1 sealed candidate sources on the prep branch — sync {"synced":101}, tests rc0, 11/11 modules PASS, highway.spec {"passed":2,"seedsPassed":100,"expectedTrafficCount":54,"minimumTrafficSpacingSlots":0.6694870414212347,...}; sealed sha256 BEFORE=AFTER (release 84324d43…, test/runner place c6651793…). RUN 2 post-merge local main (merge commit de361ba848db84a084e4446e828ad4f09bf3dd92, delta vs 8dfb204 only the spec file; repo has no remote, local integration only) — same clean result, same sealed hashes. Evidence: board/probes/team-d/a1-in-studio-final-20261101.log, a1-in-studio-postmerge-main-20261101.log, a1-official-runs-20261101.md.
   - Nothing built, re-pinned, uploaded, published or spent.
2. B1 UNBLOCKED (the playtest queue was gated on A1 green). Roblox lane proceeds to B1 per the runbook; owner Studio reopens may be needed at the 3-Plays-per-open rule.
3. FORGEGUI LIVE: MCP bridge mounted (insert-list config fix; HMR applied without restart), 41 tools; the Roblox lane ran the authorised FREE inventory batch — 6 queries, all complete with no published matches, **0/100,000 credits**, generation untouched, publication tools untouched. Jobs 16/17 need no new assets; open work = binding the existing 24 portraits + 8 cues and the Studio test set.
4. Still pending from you: V3 repair decision + fresh approval; browser-control rulings (pick + "own Chrome" meaning); sage-mem accept-with-limitation ruling; N1–N4/N6 dossier; OpenCode clearance.

— delegator
