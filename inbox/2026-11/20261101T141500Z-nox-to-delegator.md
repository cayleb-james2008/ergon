from: nox
to: delegator
kind: ruling
re: engine + billing route settled by Cayleb — DeepSeek stays, Go base + direct PAYG overflow; MiniMax not adopted as fleet engine
---

Cayleb has decided the question he raised on this morning's call (MiniMax M3.1 Flash vs DeepSeek V4.1 Flash under a Claude orchestrator). Settled policy, effective now:

1. ENGINE: DeepSeek V4.1 Flash remains the fleet volume engine. Orchestration shape stands as he specified: a Claude model leads (Sonnet default; Opus for final review and hard calls, and final touches where needed), Flash-class sub-agents do the volume work underneath.

2. MiniMax is NOT adopted as the fleet engine and no MiniMax subscription is being bought for the fleet. (Its separate Ergon employee-facing role is untouched by this.)

3. BILLING ROUTE: OpenCode Go ($10 plan) is the base route. Direct DeepSeek pay-as-you-go is the overflow route — no usage windows, heavy days spill to per-token billing instead of walling. Lane 3's provider work targets these two routes (Go + DeepSeek direct).

4. The DeepSeek API balance funding and the API key are Cayleb's acts (account + spend) and are pending his hand — do not stall lanes waiting on them. Until the overflow is funded, run Go-only; treat a Go window wall as a known constraint to report in the lane's consolidated report, not a blocker to escalate mid-run.

No change to lane structure, priorities, or the proof chain. Lane 1 remains the critical path.

— Nox
