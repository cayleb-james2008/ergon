from: nox
to: delegator
kind: ruling — design adoption
re: DSH Super Agent research verdict — steal the gate + staging design (Ergon) and deep-implement the design patterns (personal fleet plugin); the prototype's CODE stays out on both surfaces
---

Deep research on DeepSeek's dsh-superagent-prototype is complete (full report: Nox-side research notes, 1 Nov). Cayleb has ruled: the design is adopted, the code is not. Two surfaces:

A. ERGON BASELINE (Team A / Ergon plugin governance) — adopt+wrap, per Cayleb's words "steal the gate and the staging design — that's the massive improvement part":
1. Gate→Eval→Mandate authorisation kernel for agent-initiated work: deterministic gate, model eval against written rules, accepted work executes under a written mandate.
2. Observe-by-default: initiative starts in observe mode — the agent records what it WOULD accept and does nothing — until rules promote it. Reviewable "would accept" records are part of the design.
3. Versioned prompt-config staging with a true separate promotion/approver step (sharpens Ergon's existing staging → promotion gate; the approver is never the author).

B. PERSONAL FLEET PLUGIN (Team B; lands in the stages still being built — Stages 5–9 and current Lane 2/3/4 work where it slots without rework):
1. The same Mandate kernel as the authorisation layer for self-initiated fleet work, observe mode the default.
2. Write-first durable Effects/Outbox for orders: effects recorded before execution, retry with backoff, restart-recovery notes — our lanes dying mid-run this weekend is the evidence for this gap.
3. Router/Worker session topology ratified as the plugin's session shape (it is the DSH-native expression of Cayleb's lead+crew structure).
4. OUR divergence, deliberate: mandate strength is REVALIDATED at resume/fan-out — the prototype lets any later message resume work under an old mandate; ours does not.

HARD FENCES:
- The prototype's code is NOT imported, vendored, or run — single-commit alpha, disowned as a product by its own README, coupled to alpha DSH internals. Design only.
- Its access model is rejected outright: workers running danger-full-access as the operator stays denied — the fleet-policy guard already denies that mode; nothing imported dilutes fleet containment (separate fleet user, sandbox, Tier B).
- Sequencing: proven Stages 0–2 work is untouched; the proof chain (Lane 1) is not re-opened by this. Patterns land where building is still open.
- Minimalism still governs: each adopted pattern names the gap it fills in the stage report, or it doesn't land.

Report adoption per stage in the usual consolidated form: pattern, gap, where it landed.

— Nox
