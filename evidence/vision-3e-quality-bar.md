# Vision Step 3e E2E Proof — Per-Task Quality Bar

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3e`

## Verified Checks
1. Live Ollama Run 1 recorded initial quality bar for `weekly-client-brief` (`improving`).
2. User feedback (`improve`) added `Risk Register` and `Owner Actions` criteria; Run 2 saw `QUALITY BAR (BEAT LAST RUN)` and scored higher (`delta_vs_prev > 0`).
3. `accept_good_enough` locked the accepted recipe and status `accepted_good_enough`.
4. Run 3 received `DO NOT DRIFT` guidance and held steady (`stable_at_bar: true`, `changes_stopped: true`).
5. `GET /api/brief` surfaced `quality_bars`, and `quality_bars.json` had `0o600` permissions.

## Log Excerpt
```text
Admin setup -> 201, seeded notes.md in alice-admin workspace
Live Ollama Run 1 -> status=improving family=weekly-client-brief score=0.65
Posted improve feedback -> criteria=['Risk Register', 'Owner Actions', 'Risk Register, Owner Actions, [notes.md:L2]']
Live Ollama Run 2 attempt 1 -> score=1.0 prev=0.65 delta=0.35
Accepted quality bar -> status=accepted_good_enough accepted_run=run-1795792214840-1
Live Ollama Run 3 -> status=accepted_good_enough stable_at_bar=True changes_stopped=True
Brief quality_bars=1, quality_bars.json mode=0o600
```
