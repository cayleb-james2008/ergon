# Vision Step 3d E2E Proof — Outcome Scoring & Recursive Self-Improvement

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3d`

## Verified Checks
1. Live Ollama (`qwen2.5:3b`) proposed `greeter-badge` v2 from negative live outcome feedback; deterministic replay harness compiled and promoted v2 over v1.
2. Worse candidate v3 was rejected by replay gating (`regressions > 0`), leaving champion v2 intact.
3. Two live negative outcomes on v2 triggered automatic rollback to v1 (`auto_rolled_back` logged).
4. Meta-improver `default-improver` v2 beat v1 on replay evaluation and was promoted.
5. Evolution store permissions verified `0o600` with zero internal scripting engine jargon.

## Log Excerpt
```text
Admin setup -> 201: person_id=alice-admin
Built & approved v1 tool 'greeter-badge' (decision=d1795792126579-0)
Added replay case badge-case-1 requiring '[verified]' tag
Ollama propose attempt 1 -> 200: {"artifact_id":"greeter-badge","candidate_compiled":true,"candidate_score":1.0,"cases_run":1,"champion_score":0.6666666666666666,"detail":"promoted v1 -> v2 (champion=0.667, candidate=1.000, cases=1, regressions=0)","from_version":1,"kind":"tool","promoted":true,"regressions":0,"to_version":2}
Ollama v2 PROMOTED: v1 -> v2 (champion=0.67, candidate=1.00)
Worse v3 REJECTED: regressions=2 (champion remains v2)
Live regression triggered AUTO-ROLLBACK -> v1: auto-rolled back v2 -> v1: live mean=0.000 (baseline=0.850), denial_rate=1.00 across 2 runs
Meta-improver live eval attempt 1 -> 200: {"artifact_id":"default-improver","candidate_compiled":true,"candidate_score":1.0,"cases_run":1,"champion_score":0.3333333333333333,"detail":"promoted v1 -> v2 (champion=0.333, candidate=1.000, cases=1, regressions=0)","from_version":1,"kind":"improver","promoted":true,"regressions":0,"to_version":2
Meta-improver default-improver PROMOTED: v1 -> v2
Evolution scores verified (artifacts=2, versions.json mode=0o600, zero 'Rune')
```
