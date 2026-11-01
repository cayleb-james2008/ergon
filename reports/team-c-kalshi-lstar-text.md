# KALSHI — L\* = 30 s: the exact text applied, for Nox's review

From: trading-lead (Team C). To: Nox (via the delegator). Date: 2026-10-31 18:5x ET.
Authority: Nox batch-2 answer 6 ("L\* = 30 s CONFIRMED … DECISIONS.md's provisional 60 s is superseded — apply the log line
and send the text for my review"). Scope of the edit: **shadow docs only** — `research/kalshi15/DECISIONS.md` and
`research/kalshi15/SPEC.md`. Nothing in the live path, the execution package, the gateway, `params.provisional.json` or any
generated results file was touched.

## 1. What was applied — `research/kalshi15/DECISIONS.md`

**(a) The 13:17 ET confirmation's item 2 was marked superseded (history kept, not rewritten):**
> 2. The provisional row **L\* = 60 s** is used for θ and X selection until the gateway's dry-run timestamps produce the measured
>    end-to-end latency; it matches the system's current lag. When the measured figure lands, the provisional is replaced and the
>    replacement is logged in this file the same way. Reversible before the freeze. **[SUPERSEDED 2026-10-31 by Amendment 2
>    below: the measured row is L\* = 30 s.]**

**(b) A new section was appended (verbatim as written in the file):**
> ## Amendment 2 (Nox, 2026-10-31 ~18:45 ET) — latency row L* = 30 s, applied by the build side
>
> Nox's ruling (batch-2 answer 6, `nox-rulings-batch2-20261031T224500Z.md`): "L* = 30 s CONFIRMED. Already ruled in
> Amendment 3 and the Monday resume pack (item 1c); your new measurement (~19–20 s end-to-end; smallest row of record ≥
> that is 30 s; 30 s sits above the data-time p90 of 27.7 s) matches the ruling. DECISIONS.md's provisional 60 s is
> superseded — apply the log line and send the text for my review. θ re-derives at the 30 s row."
>
> | Item | Ruling (applied) |
> |---|---|
> | Latency (amendment 2) | Provisional **L\* = 30 s** replaces the 60 s row for θ/X selection, on the measured availability of 2026-10-31: decisions readable **16.127 s p50 / 16.735 s p90** after the decision tick and 24.586 / 27.713 s after `a_asof` (`results/availability_2026-10-31.json`: 81 decision lines, 9 ticks, 10 min); with intent pull ~1–2 s and gateway <1 s the end-to-end from the tick is ≈ **18–19 s**, so 30 s is the smallest row of record at or above it; the margin over the data-time p90 is **0.29 s** (27.713 + 2.0 = 29.713 ≤ 30). *(Corrected 2026-10-31 19:4x ET per the trading-checker's audit: the earlier draft said ≈19–20 s and did not state the margin.)* The 60 s row stays a row of record; the live gate still applies at the row matching the gateway's **measured** end-to-end latency, never the best row; if that measurement lands above 30 s the threshold re-derives at that row before any flip. Rows 2 s and 5 s stay informational. Reversible before the freeze. |
>
> Recorded by: trading-lead (Team C), on Nox's ruling; applied 2026-10-31 18:5x ET. Text sent to Nox for review in
> `board/reports/team-c-kalshi-lstar-text.md`. `results/scorer_agreement.json` still carries `latency.L_star = 60` (a
> generated artefact) and `params.provisional.json` still carries `l_star_s: 60`; neither was changed by this amendment —
> see the review note for the two one-line flips that remain.

## 2. What was applied — `research/kalshi15/SPEC.md` Appendix T (new Amendment 4)
> - **Amendment 4 (SPEC, 2026-10-31 18:5x ET; Nox's Amendment-2 ruling on the latency row, applied by the build side):**
>   the provisional row for θ/X selection is **L\* = 30 s**, not 60 s. Measured availability of 2026-10-31: decisions
>   readable 16.127 s p50 / 16.735 s p90 after the decision tick, 24.586 / 27.713 s after `a_asof`; with the intent pull
>   (~1–2 s) and the gateway (<1 s) end-to-end from the tick is ≈ **18–19 s**, so 30 s is the smallest row of record at or
>   above it (margin 0.29 s over the data-time p90: 27.713 + 2.0 = 29.713 ≤ 30). The 60 s row stays a row of record; the live gate still applies at the row matching the gateway's measured
>   end-to-end latency, never the best row. `latency.L_star` in generated JSON becomes 30 once the artefact is regenerated;
>   until then the generated files' 60 is a stale field, not a second ruling. See DECISIONS.md Amendment 2.

## 3. Deliberately NOT changed (two one-line flips left for your word)
1. `research/kalshi15/params.provisional.json` still has `"l_star_s": 60`. It is a frozen-candidate **data** file, not a doc;
   I did not edit it. Consequence if it stays: a Stage A run that reads the file's default would pick θ/X at the 60 s row
   unless `--L 30` is passed explicitly. **Recommendation:** authorise the flip `60 -> 30` before the Stage A final run
   (or tell the runner to pass `--L 30`); the freeze tool (`freeze.py`) takes `--L` explicitly and records it, so the freeze
   itself cannot silently use 60.
2. `research/kalshi15/results/scorer_agreement.json` still carries `"latency": {"L_star": 60, ...}`. It is a generated
   artefact (13:45 ET); regenerating it is a scoring run, not a doc edit. **Recommendation:** leave it until the next
   `stage_a.py dump-s` run regenerates it, then confirm the field reads 30; do not hand-edit a generated file.

## 4. Verification of the edit (quote)
```
$ git -C ~/Work/projects/openbell-kalshi diff --stat research/kalshi15/DECISIONS.md research/kalshi15/SPEC.md
 research/kalshi15/DECISIONS.md | 19 ++++++++++++++++++-
 research/kalshi15/SPEC.md      |  7 +++++++
 2 files changed, 25 insertions(+), 1 deletion(-)
```
No other file in the worktree changed by this edit (the pre-existing `data/kalshi15/manifest.json` modification and the
three untracked `first_scorer.*` result files were already there before tonight's session).

## 5. If the wording needs to differ
Say which sentence and it will be re-applied verbatim; nothing here is a new ruling, only the log line for yours. If you
prefer the 60 s row described as "retained but not used", say so and the same text will say that instead.

## 6. Correction applied after the trading-checker's audit (2026-10-31 19:4x ET)
The checker's audit (V1-V4 MATCH, V5 NOT FIT TO SIGN) flagged four record corrections; the L\*-related two are now applied
in the shadow docs and in this file:
1. End-to-end latency is **≈18–19 s** (not ≈19–20 s), and the L\* = 30 s margin over the data-time p90 is **0.29 s**
   (27.713 + 2.0 = 29.713 ≤ 30). Applied to `DECISIONS.md` Amendment 2 and `SPEC.md` Amendment 4 with a dated correction note.
2. The two scorer outputs are identical **within tol 1e-9** (integers exact), not "byte-identical"; the JSON's own wording
   already said "identical (tol 1e-9, integers exact)" and no doc is changed for this — the phrase "byte-identical" belongs
   only to the WP10 dry-run artifact rerun (six files, equal sha256), which is a different claim.
3. The WP10 dry-run `end_to_end_s` p50 64.5 s is a **construction of the batch rehearsal** (row 60), not a measurement of the
   live path — the measured availability is the 16.127/16.735 s pair above. No doc change needed; recorded here so the two
   numbers are never quoted interchangeably.
4. Nothing in this amendment was reversed; only the two arithmetic statements above were corrected.
