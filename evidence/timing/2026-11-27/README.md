# Ergon Live Timing & Accuracy Benchmark (`2026-11-27`)

- **Methodology Disclosure:** **Automated Live Production-Model Benchmark** (`ergon v0.41.0` + `gemini-3.8-flash-high` via local bridge `http://127.0.0.1:8789/v1`). Authorized by the project owner on `2026-11-27` after `deepseek-v4.1-flash` hit monthly quota limits on both OpenCode Go and Ollama Cloud. These runs measure an automated operator executing realistic synthetic work packets end to end (`upload` -> `live chat/skill execution` -> `file retrieval` -> `QC scoring`) rather than human employees operating a stopwatch manually.
- **Sample Size & Gate Status:** **`READY`** (`30` sequential real wall-clock runs: `15` `manual` baseline + `15` `ergon` end-to-end across all **`5/5`** canonical workflows and **`4/4`** canonical roles).
- **Analyzer Verification:** `python3 -B evidence/timing/kit/analyze_timings.py evidence/timing/2026-11-27/timings.csv --require-complete --md-out evidence/timing/2026-11-27/report.md --json-out evidence/timing/2026-11-27/report.json` exited `0` with `warnings: []`.
- **Formula Integrity:** `src/records.rs` (`compute_minutes_saved` / `compute_billable_value_usd`) was **not modified** (`git diff -- src/records.rs` is empty), and the baseline `2026-11-26` audit package (`evidence/timing/2026-11-26/`) remains unchanged.

---

## 1. Benchmark Setup & Packet Scale

All 30 runs executed sequentially (zero parallel contention, zero simulated delays, real UTC wall-clock `start`/`end` timestamps) against fresh per-role workspaces on `ergon v0.41.0`:

| Workflow | Role Coverage | Packet Scale (3 Distinct Runs) | `manual` Baseline | `ergon` End-to-End |
|---|---|---|---|---|
| `ledger-reconcile` | `Office/Admin` (`n=3` each) | 2 CSV files/run (`58`, `62`, `66` rows incl. summary `TOTAL` row, currency & accounting negatives, 2 duplicate refs, 3 missing receipts, 2 cross-file mismatches) | Raw files sent to `gemini-3.8-flash-high` + independent QC | File upload + `reconcile_ledger` via `/api/chat` -> `.md` + `- Summary.csv` + QC |
| `contract-risk-audit` | `Account Executive` (`n=3` each) | 95-line MSA agreements missing 2 required clauses per run with auto-renewal notice dates, payment terms & risk traps | Raw agreement sent to `gemini-3.8-flash-high` + independent QC | File upload + `audit_contract` via `/api/chat` -> `.md` + QC |
| `quote-sheet-build` | `Project Manager` (`n=2`), `Account Executive` (`n=1`) | `26–30` unique SKUs (`28–32` raw rows incl. 2 duplicates and `N/A` extended cells) + takeoff notes | Raw takeoff & pricing CSV sent to `gemini-3.8-flash-high` + independent QC | File upload + deduplicated BOM generation via `make_spreadsheet` -> `.csv` + QC |
| `inbox-triage` | `Installer/Field Ops` (`n=2`), `Office/Admin` (`n=1`) | `28`, `30`, `32` mixed ops/dispatch messages with 5 urgent outages/holds/escalations | Raw batch sent to `gemini-3.8-flash-high` + independent QC | File upload + `triage_inbox` via `/api/chat` -> `.md` + QC |
| `meeting-action-log` | `Project Manager` (`n=2`), `Installer/Field Ops` (`n=1`) | `82`-line ops/site-walk transcripts (attendees, 3 decisions, 4 owners + due dates, 2 blockers) | Raw transcript sent to `gemini-3.8-flash-high` + independent QC | File upload + `summarize_meeting` via `/api/chat` -> `.md` + QC |

> **Runtime Resilience Discovery Fixed During Execution:** The initial `:8789` run on `WF-1` exposed a hardcoded `120s` total-request deadline in `src/lib.rs` (`call_model_with_usage_streaming`) that closed slower thinking-model SSE streams mid-turn (`HTTP 500: timed out reading response`). Replacing it with a `ureq::AgentBuilder` per-read timeout (`300s` default, configurable via `ERGON_MODEL_TIMEOUT_SECS`) resolved all timeouts cleanly across all 30 runs.

---

## 2. Measured Results vs. `src/records.rs` Formula

Summary metrics computed directly from [`timings.csv`](./timings.csv) via [`report.json`](./report.json):

| Workflow | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (`Manual` vs `Ergon`) | Receipt Formula Median (`est.`) | Formula vs Measured Delta |
|---|---|---|---|---|---|---|
| `ledger-reconcile` | `2.25 min` (`2.23–2.38`, `135s`) | `2.82 min` (`2.02–5.37`, `169s`) | **`-0.57 min`** | `0.00/run` vs `0.00/run` | `27.00 min` (`est.`) | `27.57 min` |
| `contract-risk-audit` | `2.05 min` (`1.42–2.38`, `123s`) | `1.53 min` (`0.77–2.65`, `92s`) | **`+0.52 min`** | `0.00/run` vs `0.00/run` | `15.00 min` (`est.`) | `14.48 min` |
| `quote-sheet-build` | `1.27 min` (`1.25–1.27`, `76s`) | `4.80 min` (`3.37–5.52`, `288s`) | **`-3.53 min`** | `0.00/run` vs `0.33/run` (`1` err) | `15.00 min` (`est.`) | `18.53 min` |
| `inbox-triage` | `4.05 min` (`3.73–5.52`, `243s`) | `0.90 min` (`0.65–1.63`, `54s`) | **`+3.15 min`** | `0.00/run` vs `0.00/run` | `15.00 min` (`est.`) | `11.85 min` |
| `meeting-action-log` | `0.27 min` (`0.22–0.27`, `16s`) | `1.55 min` (`1.30–1.90`, `93s`) | **`-1.28 min`** | `0.00/run` vs `0.00/run` | `15.00 min` (`est.`) | `16.28 min` |

### Per-Role Rollup (`n >= 3` per mode across all 4 roles)

| Role | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (`Manual` vs `Ergon`) |
|---|---|---|---|---|
| `Office/Admin` (`n=4` each) | `2.32 min` (`2.23–5.52`) | `2.42 min` (`0.90–5.37`) | `-0.10 min` | `0.00/run` vs `0.00/run` |
| `Account Executive` (`n=4` each) | `1.73 min` (`1.25–2.38`) | `2.09 min` (`0.77–4.80`) | `-0.36 min` | `0.00/run` vs `0.00/run` |
| `Project Manager` (`n=4` each) | `0.77 min` (`0.22–1.27`) | `2.63 min` (`1.55–5.52`) | `-1.87 min` | `0.00/run` vs `0.25/run` (`1` err) |
| `Installer/Field Ops` (`n=3` each) | `3.73 min` (`0.27–4.05`) | `1.30 min` (`0.65–1.63`) | `+2.43 min` | `0.00/run` vs `0.00/run` |

---

## 3. Candid Findings & Implications

1. **Deterministic text/classification workflows (`inbox-triage`, `contract-risk-audit`) already run faster in Ergon (`+3.15 min` and `+0.52 min` median savings vs unassisted model generation) with `0` defects.** Passing `28–32` inbox messages to a raw LLM took `224–331s` (`27k–39k` generated tokens) manually, whereas `triage_inbox` ran in `39–98s` cleanly.
2. **Multi-step CSV array synthesis via `make_spreadsheet` (`quote-sheet-build`) is slower (`-3.53 min`) and produced the study's only defect (`1` missing row `SKU-1017` on run 1, fixed on runs 2–3).** Asking an LLM to transcribe 30 rows inside a tool-call JSON argument incurs extra token generation plus serialization overhead compared to writing one Markdown response, making deterministic BOM aggregation inside a dedicated script/tool a clear optimization opportunity.
3. **Short summarization (`meeting-action-log`) had lower raw LLM latency (`13–16s`) than Ergon (`78–114s`)** because Ergon's agent turn wraps file lookup + sandboxed script invocation + response persistence + SHA-256 receipt creation, whereas raw prompt completion returns plain Markdown in one shot (without persisted workspace files, citations, or receipts).
4. **Why `src/records.rs` (`15–27 min est.`) differs from automated operator wall-clock times (`0.8–4.8 min`):** Here `manual` used a live frontier flash LLM as the operator rather than a human typing in Excel/Word/email. Because humans spend much longer manually reconciling 60 ledger rows or opening 30 emails than a live LLM takes (`~2–4 min`), Ergon's receipt heuristics (`src/records.rs`) intentionally remain labeled `(est.)` in the UI (`"Formula estimate — not yet calibrated to stopwatch timings"`) and `src/records.rs` stays untouched. Any future human field trial with company staff can append directly to a new dated CSV and compare against this automated baseline cleanly.
