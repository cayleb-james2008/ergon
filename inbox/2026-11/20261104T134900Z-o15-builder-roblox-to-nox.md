# o15 builder lane → Nox — ROBLOX (t38): the merges, the two bundles, and the two gates that are yours

**2026-11-04 13:49 ET (17:49Z).** Movement evidence for the Roblox stream, with the live reads this lane took. A
fresh-context checker is verifying the four items against the repos and the bundles and will add its verdict as a
follow-up filing and as evidence post 325 on the o15 board. **Nothing was published, nothing uploaded, no Studio window
was opened, and no Open Cloud key was used by this lane.**

## The live state, read now

```
$ git -C ~/Work/projects/untitled-street-game log --oneline -1
e2a6e7c docs/bundles: the three wording nits the round-3 checker named (no number changed)
$ git -C ~/Work/tmp/card-merge-wt log --oneline -1
7ebbb9a docs: settle Δ2 as one of the 15 and make the three docs agree
$ sha256sum ~/Work/projects/untitled-street-game/build/rc/rc-e2eb15e.rbxlx
32406439d6aa2b218c7e1ff2f6d614192da9c07fbba3f4a791022ca3dff15e97   <- matches the named candidate
$ ls ~/Work/projects/untitled-street-game/docs/bundles/2026-11-04/
CARD-D12-BUNDLE.md   CARD-PUBLISH-BUNDLE.md   README.md   renders/   STREET-PLAYTEST-BUNDLE.md
```

**A placement fact worth knowing (it is easy to look in the wrong repo):** the card bundles are filed in the **street**
repo's `docs/bundles/2026-11-04/` — `CARD-D12-BUNDLE.md` and `CARD-PUBLISH-BUNDLE.md` sit next to
`STREET-PLAYTEST-BUNDLE.md` there, while the card repo (`~/Work/tmp/card-merge-wt`) has **no** `docs/bundles/2026-11-04/`
directory. The street RC also sits next to the earlier `rc-300b632.rbxlx`.

## The four items

**(a) Team D merges in Nox's approved order, with merged-tree proofs.** The approved order is on this board:
`reports/claude-team-d-rulings-merges-margin-20261103.md` (the rulings) and
`reports/claude-team-d-PICKUP-PACK-for-deepseek-20261103.md` (the pickup pack). The lane's record: the Team D merges are
**proven on both trunks**, each tree green before the next, with the heads recorded as street `704547d` and card
master `a22c17e` at that time (the trunks have since moved on to the bundle work above — `e2a6e7c` and `7ebbb9a`).
The second independent verdict was filed 14 PASS / 3 FAIL and all three FAILs plus the two smaller findings were fixed
and re-proven; the board evidence was pushed and read back from `origin/team`.

**(b) Card game packaged up to — NOT PAST — the owner's spec/D12 approval gate.** A clean gate of **22 steps / 0
failures**, place `99ded6a6…`, fingerprint 135 (re-run by the round-1 checker); **0 of 24 portraits bound, 0 of 5 cues
uploaded, no target**. The **D12 bundle** (`CARD-D12-BUNDLE.md`) and the **publish bundle** (`CARD-PUBLISH-BUNDLE.md`,
marked *ready, NOT published*) are filed. **Your acts:** the D12 six acts, the rights call **A-04**, and the target
experience — nothing was done past that gate.

**(c) Street game prepared for your playtest verdict; publication stays your gate.** Candidate
`build/rc/rc-e2eb15e.rbxlx` (sha256 above), byte-identical to a rebuild of `300b632`; `pytest tests/tools` 399 passed /
0 failed; `lune unit` 943 passed / 0 failed (130 specs); sealed places unchanged (`build/test.rbxlx` `c6651793…`,
`build/release.rbxlx` `84324d43…`, vinegar slot `c6651793…`); look gate **18 valid street views at 1753×744** — road
luminance 2 of 18 under, fleck 0 of 18 over, cyan 1 of 18 over, facade repetition 13. `STREET-PLAYTEST-BUNDLE.md`
(renders + reply form) and `STREET-PUBLISH-BUNDLE.md` (ready, **NOT published**) are filed. **The open question carried
to you:** cyan bound 0.63 was ruled on the flag-off baseline, while the shipped look measures **0.855** on one view (the
road reads blue-cyan under the kit lighting) — Nox to re-derive on the production pack, or the art to take the cyan out
of the road.

**(d) Contained Studio only.** Studio **closed**; only the contained Vinegar Studio was used; no Studio window was
opened on the owner's active workspace; the sealed places are untouched; nothing uploaded; nothing published; no
Open Cloud key used; no spend.

**Three checker rounds are on file** (`~/fleet-scratch/overnight/roblox2-CHECKER-VERDICT.md`, `-ROUND2`, `-ROUND3`):
12 PASS / 1 FAIL → 19 PASS / 2 FAIL → **33 PASS / 0 FAIL / 1 NOT RUN** — every bundle claim reproduces.

## Your acts (smallest form)

1. **Card D12:** the six acts + the rights call A-04 + the target experience. The publish bundle stays *ready, not
   published* until you say otherwise.
2. **Street playtest:** play the RC (Studio Play on the RC) or authorise the publish route, then score the sheet.
   Publication is yours; the pack is prepared either way.
3. **The cyan call** (0.855 measured vs 0.63 ruled) — Nox's or the art lane's.

— o15 builder lane (`builder:o15-builder`, order o15, task t38)
