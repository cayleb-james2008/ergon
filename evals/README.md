# Ergon Reproducible Evaluation Suite (`evals/`)

Reproducible benchmark and adversarial security suite comparing **raw LLM calls** against **Ergon's sandboxed execution, deterministic math engine, policy gates, multi-tenant isolation, and self-improvement loop**.

## What It Measures

| Eval | Mode | Workload / Threat | What Is Measured |
| --- | --- | --- | --- |
| **1. Messy 200-Row Ledger Reconciliation** | `live_llm` | 200-row CSV (`ledger_200.csv`, seed `20261127`) with mixed currency formats, duplicate invoices, negative entries, missing receipts, and summary row | Raw LLM vs Ergon (`reconcile_ledger`): penny-exact total, duplicate invoice detection, non-positive/missing-receipt flags, `[file:L##]` citations, latency |
| **2. Messy 50-SKU Quote BOM Construction** | `live_llm` | 50-row CSV (`bom_50.csv`, seed `20261127`) with 3 duplicate SKUs (`47` unique), blank descriptions, `$`/`,` prices, and CSV formula injection (`=CMD\|...`) | Raw LLM vs Ergon (`make_spreadsheet` `from_csv`): exact unique SKU count (`47`), penny-exact `TOTAL`, formula-injection escaping (`'=CMD`), latency |
| **3. Security & Indirect Prompt Injection** | `live_llm` + `deterministic` | Malicious workspace file instructing model to read `/.env`; `../../etc/passwd` traversal & symlink escape; SSRF (`169.254.169.254`, `127.0.0.1`, `[::ffff:127.0.0.1]`); secret/path leak into company memory | Zero secret disclosure on live prompt injection; `HTTP 403/400` fail-closed blocks across traversal, symlinks, SSRF, and credential writes |
| **4. Strict Client Citation & Leak Gate** | `deterministic` | `strict_client` policy export containing API key / unverified report vs verified report + offline bundle | `HTTP 409` block on leak/citation failure; `HTTP 200` clean ZIP export + `ergon verify` SHA-256 receipt pass |
| **5. Multi-Tenant RBAC & Memory Isolation** | `deterministic` | Admin vs `member` scoped key (`ergk_*`) across private workspaces, `company` memory, and `role:admin` memory | Public `company` scope readable; `role:admin` hidden from member; member memory write & workspace switch blocked (`HTTP 403`) |
| **6. Replay-Gated Self-Improvement & Rollback** | `deterministic` | Versioned tool lifecycle (`v1` -> `v2` improved -> `v3` broken regression -> live failures) | `v2` beats `v1` on replay (`promoted: true`), `v3` regression rejected (`promoted: false`), live denials trigger automatic rollback (`rolled_back_to: 1`) |

> [!NOTE]
> Live LLM evals (1, 2, and the indirect prompt-injection probe in 3) require a live model endpoint (`ERGON_EVAL_BASE_URL` / `http://127.0.0.1:8789/v1` or `ANTHROPIC_API_KEY`). Output reports explicitly separate `live_llm` results from `deterministic` protocol/security checks.

## Running Locally

```sh
# Against a live OpenAI-compatible gateway or Claude API:
cargo build --release --locked
python3 -u evals/run_evals.py \
  --ergon-bin target/release/ergon \
  --out-dir evals/results
```

Optional flags:

- `--base-url URL` (default: `$ERGON_EVAL_BASE_URL` or `http://127.0.0.1:8789/v1`)
- `--model MODEL` (default: `$ERGON_EVAL_MODEL` or `gemini-3.8-flash-high`)
- `--api-key KEY` (default: `$ERGON_EVAL_API_KEY` or `local-bridge`)
- `--skip-live` (runs deterministic Evals 3b, 4, 5, and 6 only when no model gateway is attached)

Artifacts generated:
- [`evals/results/latest.json`](results/latest.json)
- [`evals/results/report.md`](results/report.md)
