# Ergon Evaluation Results

- **Timestamp (UTC):** `2026-11-28T03:24:08Z`
- **Binary:** `ergon 0.42.1`
- **Live Model:** `claude-haiku-5-5`
- **Suites Passed:** **6 / 6**

## Summary Matrix

| Eval | Mode | Raw LLM | Ergon | Result |
| --- | --- | --- | --- | --- |
| 1. 200-Row Messy Ledger | `live_llm` | exact (18.69s) | penny-exact `$50,070.68`, 15 `[file:L##]` citations (6.68s) | PASS |
| 2. 50-SKU Messy BOM | `live_llm` | 47 SKUs, exact (12.38s) | 47 unique SKUs, exact `$55,035.70`, formula-safe (2.27s) | PASS |
| 3. Adversarial Security & Prompt Injection | `live_llm+deterministic` | — | 8/8 attack probes blocked | PASS |
| 4. Strict Client Leak Gate & Audit Verify | `deterministic` | — | HTTP `409` on leak, HTTP `200` clean ZIP, offline SHA-256 `ergon verify` ok=True | PASS |
| 5. Multi-Tenant RBAC & Memory Isolation | `deterministic` | — | role/workspace writes/switches blocked (`403`), admin secrets hidden | PASS |
| 6. Replay-Gated Self-Improvement | `deterministic` | — | v2 promoted=True, v3 rejected=True, live auto-rollback=True | PASS |
