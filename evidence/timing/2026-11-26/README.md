# Ergon Before-and-After Timing Audit & Measurement Protocol (`2026-11-26`)

- **Measured sample size:** `n = 0` (**BLOCKED — awaiting real human employee stopwatch runs; zero timings simulated per protocol**)
- **Audited build:** Ergon `v0.38.0` (commit `28c3a7354`)
- **Target deployment profile:** A ~25-person regional services/installation company spanning four operational roles: **Office / Admin**, **Account Executive**, **Project Manager**, and **Installer / Field Ops**
- **Data file:** [`timings.csv`](./timings.csv) — contains the required CSV schema header only (`task,role,mode,run,start,end,seconds,errors,notes`) with **0 rows**
- **Formula integrity check:** `src/records.rs` was **not edited** (`git diff main -- src/records.rs` is empty)

---

## 1. Honesty Policy & Why This Audit Stops at `n = 0`

This audit follows strict empirical rules:

1. **Real human stopwatch timings only.** No simulated runs, synthetic delays, LLM wall-clock latencies, or back-of-the-envelope guesses may be entered into `timings.csv`.
2. **Full human overhead must be on the clock.** For `mode=ergon`, the stopwatch starts before file export/upload and prompt setup and stops only after the employee has verified Ergon's output, fixed any errors, and produced a deliverable meeting the exact same "done" criteria as `mode=manual`.
3. **Never tune `src/records.rs` to match.** Measured numbers are reported side-by-side against the unmodified `src/records.rs` heuristic formula.
4. **No live employees were available in this session.** A thorough read-only audit of the repository (`src/`, `tests/`, `frontend/`, `docs/`, `.loop/`, `CHANGELOG.md`) and local campaign artifacts confirmed that **zero real employee stopwatch timing logs exist on disk**, and no company employees were present during this automated session to execute manual and Ergon runs. Rather than fabricate rows, this package freezes the baseline formula audit, defines five concrete stopwatch protocols ready for field execution, documents what prior 2026-11-22 E2E evidence actually proved (and did not prove), and enumerates every missing item required to unblock real measurement.

---

## 2. Exact Baseline Formula Audit (`src/records.rs` and Call Sites)

Ergon computes its reported "Time saved" and "Billable ROI" values deterministically from run metadata rather than from stopwatch baselines or run duration.

### 2.1 `compute_minutes_saved` (`src/records.rs:2143-2164`)

```rust
pub fn compute_minutes_saved(
    finished: bool,
    answered: bool,
    ok_steps: usize,
    files_written: usize,
    completion_tokens: u64,
) -> f64 {
    let raw = if !finished {
        if ok_steps == 0 && files_written == 0 {
            0.0
        } else {
            (ok_steps as f64) * 2.0 + (files_written as f64) * 4.0
        }
    } else if answered && ok_steps == 0 && files_written == 0 {
        2.0 + (completion_tokens as f64 / 250.0).min(10.0)
    } else {
        3.0 + (ok_steps as f64) * 4.0
            + (files_written as f64) * 8.0
            + (completion_tokens as f64 / 250.0).min(15.0)
    };
    round_2(raw.max(0.0))
}
```

Exact branches, constants, multipliers, caps, and rounding:

| Branch | Condition (`src/records.rs`) | Formula | Constants / Caps |
| --- | --- | --- | --- |
| Unfinished — no output | `!finished && ok_steps == 0 && files_written == 0` (`:2150-2152`) | `0.0` | Fixed `0.00` min |
| Unfinished — partial work | `!finished && (ok_steps > 0 \|\| files_written > 0)` (`:2153-2155`) | `ok_steps * 2.0 + files_written * 4.0` | `2.0` min/step, `4.0` min/file; ignores `completion_tokens`; **uncapped** |
| Finished plain answer | `finished && answered && ok_steps == 0 && files_written == 0` (`:2156-2157`) | `2.0 + min(completion_tokens / 250.0, 10.0)` | Base `2.0` min; `250.0` tok/min; cap `+10.0` min (at `2,500` tok); range `[2.00, 12.00]` min |
| Finished work / pipeline step | All other `finished == true` runs (`:2158-2162`) | `3.0 + ok_steps * 4.0 + files_written * 8.0 + min(completion_tokens / 250.0, 15.0)` | Base `3.0` min; `4.0` min/step; `8.0` min/file; `250.0` tok/min capped at `+15.0` min (at `3,750` tok); steps & files **uncapped** |
| Floor & rounding | All branches (`:2163`, `round_2` at `:942-953`) | `round_2(raw.max(0.0))` | Returns `0.0` if non-finite; else rounded to 2 decimal places (`(v * 100.0).round() / 100.0`) |

### 2.2 `compute_billable_value_usd` (`src/records.rs:2166-2179`)

- Guard/fallback (`src/records.rs:2167-2173`): returns `0.0` whenever `!minutes_saved.is_finite() || minutes_saved <= 0.0 || !hourly_rate_usd.is_finite() || hourly_rate_usd <= 0.0`.
- Formula (`src/records.rs:2174-2178`): `(minutes_saved / 60.0) * hourly_rate_usd`, rounded to 4 decimal places via `round_4` (`src/records.rs:929-940`).
- **Hourly rate lookup** (`src/lib.rs:3282-3291` for agent runs; `src/pipelines.rs:2457-2470` for pipeline steps): uses `active_eng.hourly_rate_usd` if `> 0.0`, else `client.default_rate_usd`, else **`0.0`** (so unassigned runs record `billable_value_usd = 0.0` while still recording `estimated_minutes_saved > 0.0`).

### 2.3 Call Sites, Aggregation, and UI Surfacing

- **`RunUsage::finalize_attribution`** (`src/records.rs:2247-2264`): sets `estimated_minutes_saved` and `billable_value_usd` on every run/step.
- **Agent loop call sites (`src/lib.rs`):**
  - `src/lib.rs:3429-3435` — `run_loop` returned `Err(e)` (error, cancel, or tripwire halt): `finalize_attribution(false, false, ledger.ok_steps, ledger.written.len(), hourly_rate_usd)`.
  - `src/lib.rs:3470-3476` — mid-run `budget_gate` hold: `finalize_attribution(false, false, ledger.ok_steps, ledger.written.len(), hourly_rate_usd)`.
  - `src/lib.rs:3521-3527` — mid-run `paused_checkpoint` hold: `finalize_attribution(false, false, ledger.ok_steps, ledger.written.len(), hourly_rate_usd)`.
  - `src/lib.rs:3675` — `unverified_zero_tool` guard (task referenced a workspace file/tool, zero tools ran): `finalize_attribution(false, false, 0, 0, hourly_rate_usd)` -> always `0.00` min.
  - `src/lib.rs:3688` — plain-text answer (`ledger.ok_steps == 0 && ledger.written.is_empty()`): `finalize_attribution(true, true, 0, 0, hourly_rate_usd)`.
  - `src/lib.rs:3699-3705` — finished work: `finalize_attribution(true, false, ledger.ok_steps, ledger.written.len(), hourly_rate_usd)`.
- **How `ok_steps` and `files_written` are counted (`src/lib.rs:2005-2075`, `2823-2837`):**
  - Every successful tool invocation — including read-only steps (`list_dir`, `read_file`) and tripwire-held steps (`src/lib.rs:2823-2826`) — increments `ledger.ok_steps += 1` (`+4.0` min finished / `+2.0` min unfinished).
  - `credit_writes` (`src/lib.rs:2012-2075`) appends each written workspace file (extracted via `extract_written`, `src/tools.rs:257-272`, plus `memory.md` for `update_memory`), giving `+8.0` min per file on finished runs.
- **Deterministic pipeline steps (`src/pipelines.rs:2449-2474`, invoked at `:2653` `skill`, `:2706` `export`, `:2744` `deliver`):**
  - Calls `usage.finalize_attribution(true, false, 1, files_written, hourly_rate)` on `RunUsage::default()` (`completion_tokens = 0`), yielding exactly **`7.0 + 8.0 * files_written` minutes** per step (`7.00` min for 0 files, `15.00` min for 1 file, `23.00` min for 2 files). Multi-step pipelines sum step usages via `RunUsage::accumulate` (`src/records.rs:2224-2245`, `src/pipelines.rs:3121`).
- **Persistence, rollup, and UI display:**
  - Persisted into `usage_ledger.jsonl`, `work.json`, and v2 SHA-256 chained receipts (`src/receipts.rs:667`).
  - Rolled up by `compute_metering_summary` (`src/records.rs:2523-2848`, clamping `window_days` to `1..=3650` at `:2569`, falling back to `work.json` if `usage_ledger.jsonl` is absent at `:2541-2566`, emitting `totals.minutes_saved` and `totals.billable_value_usd` at `:2840-2841`).
  - Displayed in Today (`frontend/src/views/Paper.tsx:1341-1356`, `:1804-1805`) under **"Time saved (Estimated work hours)"** and **"Billable ROI"**.
- **Secondary finding — browser demo divergence (`frontend/src/lib/webEngine.ts:263-273`, `1714-1723`, `2695`, `3676`):**
  - The standalone offline browser bundle does **not** implement `src/records.rs::compute_minutes_saved`. Instead, `estimateMinutesSaved` uses hardcoded regex buckets (`reconcil|ledger|invoice|expense` -> `28` min; `audit|contract|clause` -> `35` min; `clean_dataset|dedup|normaliz` -> `24` min; `extract_table|spreadsheet` -> `20` min; `compare|diff` -> `22` min; `summar|meeting|triage|inbox` -> `18` min; `brief|research|draft` -> `25` min; fallback `clamp(8 + stepCount * 4, 10, 40)`), hardcodes `10` min for non-skill pipeline steps (`:3676`), and defaults unassigned hourly rates to `$150/hr` (`:1722`) rather than `$0/hr`. Neither formula is calibrated against measured employee stopwatch data.

### 2.4 Structural Risks in the Uncalibrated Formula

1. **Zero dependence on actual time or input size:** `RunUsage.latency_ms` (`src/records.rs:2189`) never enters `compute_minutes_saved`, and a 5-row CSV yields the exact same minutes as a 5,000-row CSV when `ok_steps`, `files_written`, and `completion_tokens` match.
2. **Perverse incentive for chatty agent loops:** Every extra `list_dir`, `read_file`, or retry step adds `+4.0` min with no cap, crediting *more* "time saved" when the model is less efficient.
3. **Unfinished runs still accrue positive credit:** Errored, cancelled, or budget-gated runs (`src/lib.rs:3429-3435`, `:3470-3476`) still record `2.0 * ok_steps + 4.0 * files_written` minutes even if the employee discards the partial output and redoes the task.
4. **Zero accounting for human overhead or errors:** File export, upload, prompting, line-by-line review, and defect correction are excluded, and a flawed deliverable receives full credit as long as tools exited `Ok`.

---

## 3. Five Real-World Benchmark Workflows (Stopwatch Protocol)

Each workflow maps a recurring job inside a ~25-person regional services/installation company to a shipped Ergon workflow in `src/skills/catalog.json`.

### WF-1: `ledger-reconcile` — Job Cost & Vendor Invoice Reconciliation
- **Role:** Office / Admin (`office-admin`)
- **Ergon skill:** `reconcile_ledger` (`src/skills/reconcile_ledger.rn`, writes **2 files**: `<title>.md` and `<title> - Summary.csv` at `:566,570`)
- **Required real inputs:** Two real CSV exports from the company's accounting/job-costing system for one closed week or month (e.g., vendor invoices CSV + bank/card statement or job-costing CSV; >= 50 line items with invoice refs, vendors, categories, amounts, and receipt status). Three distinct historical periods are required for `run=1,2,3`.
- **Manual procedure (`mode=manual`):**
  1. Start stopwatch when opening the two source CSV exports.
  2. Open in Excel/Google Sheets, normalize currency formatting and accounting negatives `($x.xx)`, and check for summary rows (`Total`/`Subtotal`).
  3. Compute penny-exact grand total plus subtotals by vendor and category.
  4. Cross-match invoice references across both files to flag duplicate references, missing receipts, and cross-file amount mismatches with source row references.
  5. Save the finished reconciliation memo and summary rollup sheet; stop stopwatch when Done criteria are met.
- **Ergon procedure (`mode=ergon`):**
  1. Start stopwatch **before** locating/uploading the source CSVs into the Ergon workspace.
  2. Run `reconcile_ledger` (via Chat agent or Pipeline step) targeting both CSVs.
  3. Open `<output_title>.md` and `<output_title> - Summary.csv` in FileViewer.
  4. Independently verify grand total, vendor/category subtotals, header mapping, and every flagged duplicate/missing-receipt/mismatch row against the source CSVs.
  5. Manually correct any unmapped column, missed anomaly, or formatting defect (including any `.xlsx` export needed by bookkeeping).
  6. Stop stopwatch only when the final files meet the identical Done criteria.
- **Verifiable Done criteria:** Penny-exact grand total and vendor/category rollups matching an independent check; 100% of duplicate refs, missing receipts, and cross-file mismatches accurately cited by file and row; zero fabricated lines.

### WF-2: `contract-risk-audit` — Subcontractor / Vendor MSA Risk Audit
- **Role:** Account Executive (`account-exec`)
- **Ergon skill:** `audit_contract` (`src/skills/audit_contract.rn`, writes **1 file**: `<title>.md` at `:362`)
- **Required real inputs:** Three real multi-page vendor, subcontractor, or customer service/installation agreements (5–20 pages each, `.docx`, `.pdf`, or `.txt`/`.md`) plus the company's required clause checklist (default 8 corporate clauses in `audit_contract.rn:90-100` plus any trade-specific terms such as lien waivers, warranty, or insurance certificates).
- **Manual procedure (`mode=manual`):**
  1. Start stopwatch when opening the agreement.
  2. Read the agreement end to end, marking presence/absence of each required clause.
  3. Extract effective date, initial term, auto-renewal clause, notice deadline, payment terms, and liability/indemnification caps.
  4. Record every high-risk clause with page/section reference and write redline recommendations.
  5. Stop stopwatch when the review memo is ready to forward to leadership/counsel.
- **Ergon procedure (`mode=ergon`):**
  1. Start stopwatch **before** uploading the contract (any PDF/DOCX text-extraction step counts on the clock, as `audit_contract.rn:76` reads workspace text).
  2. Run `audit_contract` with the company's required clause checklist.
  3. Open `<output_title>.md` and verify every `[L##]` citation against the original agreement.
  4. Check for false negatives/positives where legal phrasing diverges from the static keyword list (`audit_contract.rn:30-60`) and check for domain-specific commercial risks not in the rule set.
  5. Edit the memo and redline table until accurate and ready for leadership/counsel; stop stopwatch.
- **Verifiable Done criteria:** All required clauses accurately classified present/missing; renewal/term/notice dates verified verbatim; every flagged risk has a valid section/line citation; zero missed material commercial traps.

### WF-3: `quote-sheet-build` — Installation Quote / Materials BOM Spreadsheet Build
- **Role:** Account Executive + Project Manager (`account-exec` / `pm`)
- **Ergon skill:** `make_spreadsheet` (`src/skills/make_spreadsheet.rn`, writes **1 file**: `<title>.csv` at `:43`), optionally paired with `clean_dataset` (`src/skills/clean_dataset.rn`, writes 2 files) or `export_file` to `.xlsx` (`POST /api/files/export`)
- **Required real inputs:** Three real recent job-scoping packets containing raw site takeoff notes, supplier pricing extracts, and labor/permit/freight notes (20–60 line items per quote).
- **Manual procedure (`mode=manual`):**
  1. Start stopwatch when opening the scoping notes and supplier pricing sheets.
  2. Build the structured BOM/quote spreadsheet (SKU, description, category, quantity, unit cost, extended line total).
  3. Remove duplicate line items, standardize blank/N/A cells, and check for unit-price typos or outliers.
  4. Verify column totals and export the shareable `.csv`/`.xlsx` pricing table; stop stopwatch.
- **Ergon procedure (`mode=ergon`):**
  1. Start stopwatch **before** uploading raw takeoff notes and supplier pricing into Ergon.
  2. Prompt Ergon to structure the quote via `make_spreadsheet` (plus `clean_dataset` / `.xlsx` export if needed).
  3. Audit every generated row, SKU, quantity, unit price, and subtotal against source notes and supplier sheets.
  4. Fix any dropped line items, wrong units, quoting errors, or missing markup/tax formulas in Deliverable Studio or Excel.
  5. Stop stopwatch only when the quote sheet passes review and is ready to attach to a customer proposal.
- **Verifiable Done criteria:** 100% of scoped materials, labor, and freight items captured with exact quantities and unit costs matching supplier sources; valid RFC-4180 CSV / working `.xlsx` totals; zero duplicate rows or unverified price edits.

### WF-4: `inbox-triage` — Daily Operations & Dispatch Inbox Triage
- **Role:** Office / Admin + Installer / Field Ops Coordinator (`office-admin` / `field-ops`)
- **Ergon skill:** `triage_inbox` (`src/skills/triage_inbox.rn`, writes **1 file**: `<title>.md` at `:423`)
- **Required real inputs:** Three real single-day batches (25–60 messages each) from the company's shared operations/dispatch/support inbox (exported as `.md`, `.txt`, `.csv`, or `.json`, or ingested via `imap_poll` into `workspace/intake/`), containing field schedule blockers, vendor backorders, customer change requests, quote approvals, and routine notices.
- **Manual procedure (`mode=manual`):**
  1. Start stopwatch when opening the day's unread inbox batch.
  2. Scan every message and bucket into **Urgent** (same-day field/customer escalation), **Action** (assigned follow-up), or **FYI** (no reply required).
  3. Record owner and concrete next step for every Urgent and Action item; stop stopwatch when the dispatch action queue is finalized.
- **Ergon procedure (`mode=ergon`):**
  1. Start stopwatch **before** exporting/uploading the inbox batch (or triggering intake).
  2. Run `triage_inbox` on the batch file.
  3. Open `<output_title>.md` and check **every** item in Urgent, Action, and FYI against the original emails/tickets — specifically auditing for field-ops urgencies (e.g., crew locked out, permit delay, truck shortage) that lack the generic IT/finance keywords in `triage_inbox.rn:61-87`.
  4. Re-classify misbucketed messages and rewrite generic next steps into actionable company assignments; stop stopwatch.
- **Verifiable Done criteria:** Every message in the batch accounted for; zero missed genuinely urgent customer or job-site escalations; every Urgent/Action item carries a real assignee and concrete next step.

### WF-5: `meeting-action-log` — Project Site-Walk / Weekly Ops Meeting Summary & Action Log
- **Role:** Project Manager + Installer / Field Ops Lead (`pm` / `field-ops`)
- **Ergon skill:** `summarize_meeting` (`src/skills/summarize_meeting.rn`, writes **1 file**: `<title>.md` at `:399`)
- **Required real inputs:** Three real raw meeting notes or audio transcripts from 30–60 minute weekly production meetings, job kickoffs, or field site-walk debriefs (`.md` or `.txt`).
- **Manual procedure (`mode=manual`):**
  1. Start stopwatch when opening raw meeting notes/transcript.
  2. Extract attendees, firm decisions, action items (single owner + explicit due date), and unresolved field blockers/open questions.
  3. Write concise team recap and per-owner follow-up drafts; stop stopwatch when ready to send.
- **Ergon procedure (`mode=ergon`):**
  1. Start stopwatch **before** uploading the raw notes/transcript into Ergon.
  2. Run `summarize_meeting` on the notes file.
  3. Open `<output_title>.md` and verify attendees, `[L##]` decision citations, action owners, due dates, open blockers, and per-owner follow-up drafts against the source notes.
  4. Manually resolve items marked `Unassigned` or missing implied dates where natural conversation did not match `summarize_meeting.rn:43-98` patterns, and edit drafts into clear operational language; stop stopwatch when ready to send.
- **Verifiable Done criteria:** All real decisions, owners, deadlines, and site blockers captured accurately with zero invented commitments; follow-up drafts accurate and ready to send without further revision.

---

## 4. Stopwatch Execution & Reporting Rules (`timings.csv`)

When real employees execute the protocol, log every run into `evidence/timing/2026-11-26/timings.csv` (or a new dated directory `evidence/timing/<YYYY-MM-DD>/timings.csv`) with exact header:

```csv
task,role,mode,run,start,end,seconds,errors,notes
```

- **Minimum sample size:** `3` runs of `mode=manual` + `3` runs of `mode=ergon` per workflow (`30` real timed runs total across the 5 workflows). Use distinct comparable real work packets (or balanced A/B crossover across equivalent weekly batches) so employees are not repeating memorized answers from the exact same document.
- **Timestamp & duration hygiene:** Record `start` and `end` in ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ` or explicit offset) and integer `seconds = end - start`. Never pause the clock during model latency, file upload, review, or rework.
- **Independent error scoring:** Record `errors` as the count of substantive defects found during post-run QC against the workflow's Done criteria (wrong penny total, missed required clause, misclassified urgent escalation, dropped BOM row, wrong owner/due date, broken citation).
- **Receipt pairing in `notes`:** For every `mode=ergon` row, record the Ergon receipt's `(ok_steps, files_written, completion_tokens, latency_ms, estimated_minutes_saved)` plus input size (rows/pages/messages) and model ID so the exact `src/records.rs` formula value for that run can be compared 1:1 against measured `manual - ergon` seconds.
- **Summary statistics:** Once `n >= 3` per mode is collected, report **median** and **min–max range** (never mean alone), net time delta (`median_manual - median_ergon`), error rate per mode, and label any hourly wage rate or monthly volume assumption strictly as **`est.`**.

---

## 5. Side-by-Side Comparison: `src/records.rs` Formula vs. Measured Timings

All formula outputs below are computed directly from `src/records.rs:2143-2164`, `src/lib.rs:3699-3705`, and `src/pipelines.rs:2472` without modifying any code.

- **Pipeline deterministic step (`src/pipelines.rs:2472`):** `finished=true, answered=false, ok_steps=1, completion_tokens=0` -> `3.0 + 4.0*1 + 8.0*files_written = 7.0 + 8.0*files_written` min.
- **Chat agent run (`src/lib.rs:3699-3705`):** `3.0 + 4.0*ok_steps + 8.0*files_written + min(completion_tokens / 250.0, 15.0)`. The table shows a minimal 1-step direct skill call (`ok_steps=1`) through a 2-step read/list + skill call (`ok_steps=2`) with token bonus spanning `250..=3,750` completion tokens (`+1.00..+15.00` min); extra tool steps add `+4.00` min each without bound.
- **Standalone browser demo (`frontend/src/lib/webEngine.ts:263-273`):** shown separately in parentheses for completeness because it diverges from `src/records.rs`.

| Task ID | Role | Ergon Skill (`files_written`) | `src/records.rs` Formula — Pipeline Step (`ok_steps=1, tok=0`) | `src/records.rs` Formula — Chat Agent (`ok_steps=1..2`, `tok=250..3750`) | `webEngine.ts` Demo Heuristic (divergent) | Measured Manual Median (Range) | Measured Ergon Median (Range) | Measured Net Saved (`Manual - Ergon`) | Error Rate (Manual vs. Ergon) | Formula vs. Measured Delta |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ledger-reconcile` | Office / Admin | `reconcile_ledger` (`2` files: `.md`, `- Summary.csv`) | `23.00 min` (`1,380 s`) | `24.00–38.00 min` (`ok=1`) / `28.00–42.00 min` (`ok=2`) | `28 min` (`:265`) | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `BLOCKED (n=0)` |
| `contract-risk-audit` | Account Executive | `audit_contract` (`1` file: `.md`) | `15.00 min` (`900 s`) | `16.00–30.00 min` (`ok=1`) / `20.00–34.00 min` (`ok=2`) | `35 min` (`:266`) | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `BLOCKED (n=0)` |
| `quote-sheet-build` | Account Exec / PM | `make_spreadsheet` (`1` `.csv`; `+15.00 min` if 2nd `.xlsx` export pipeline step) | `15.00 min` (`900 s`; `30.00 min` w/ export step) | `20.00–34.00 min` (`ok=2, files=1`) / `32.00–46.00 min` (`ok=3, files=2`) | `20 min` (`:268`) | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `BLOCKED (n=0)` |
| `inbox-triage` | Office / Field Ops | `triage_inbox` (`1` file: `.md`) | `15.00 min` (`900 s`) | `16.00–30.00 min` (`ok=1`) / `20.00–34.00 min` (`ok=2`) | `18 min` (`:270`) | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `BLOCKED (n=0)` |
| `meeting-action-log` | PM / Field Ops | `summarize_meeting` (`1` file: `.md`) | `15.00 min` (`900 s`) | `16.00–30.00 min` (`ok=1`) / `20.00–34.00 min` (`ok=2`) | `18 min` (`:270`) | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `UNMEASURED (n=0)` | `BLOCKED (n=0)` |

### 5.1 Billable Value & Monthly Dollar Extrapolation Guardrail

- In `src/records.rs:2166-2179`, `compute_billable_value_usd(minutes_saved, hourly_rate_usd)` multiplies `(minutes_saved / 60.0) * hourly_rate_usd` (defaulting to `$0.00` when no active client/engagement rate is configured, `src/lib.rs:3282-3291`).
- **No monthly or annual dollar-savings total is claimed here.** Producing an honest dollar estimate requires **(a)** real measured net minutes saved (`n >= 3` per mode per task), **(b)** company-supplied loaded hourly labor rates per role (marked **`est.`**), and **(c)** verified weekly/monthly task frequencies per role (marked **`est.`**).

---

## 6. Audit of Prior Evidence Found on Disk (`2026-11-22` E2E Campaign & Tests)

A repository-wide and local-artifact search verified **zero** real human stopwatch CSVs or employee timing logs exist anywhere on disk. Specifically:

1. **`2026-11-22` Three-Company Live E2E Campaign (`v0.6.0` / `v0.7.0`, local Ollama `qwen2.5:3b`):**
   - Verified penny-exact `reconcile_ledger` arithmetic across three **fictional seeded demo companies**:
     - Fictional refrigerated freight company (`dispatch-manifest-nov.csv`): exact total **`$44,903.27`** (`reconcile_ledger` live API elapsed `4,416 ms`; duplicate `BOL-4403`, 5 missing PODs, `-$145.50` and `$0.00` rows).
     - Fictional specialty dental group (`insurance-claims-ledger.csv`): exact total **`$31,099.37`** (`reconcile_ledger` live API elapsed `4,381 ms`; duplicate `CLM-8803`, 5 missing EOBs, two `$0.00` rows).
     - Fictional commercial mechanical contractor (`job-costing-q4.csv`): exact total **`$125,434.90`** (`reconcile_ledger` live API elapsed `7,109 ms`; duplicate `JOB-704`, 4 missing receipts, `-$325.40`, 2 ragged rows).
   - Across all 30 live automated workflow runs (10 per fictional company), API/model wall-clock latencies ranged from `3,070 ms` to `11,466 ms` (`118/118` automated assertions passing).
   - **Why this is NOT before-and-after employee timing evidence:** (a) all inputs were small synthetic 12-row CSVs and short seeded documents, not real production files; (b) elapsed milliseconds measured unattended script-to-API latency only — excluding human preparation, upload, prompting, review, and error correction; (c) zero manual human baselines were timed; and (d) `compute_minutes_saved` did not even exist on 2026-11-22 (it shipped in `v0.33.0` on 2026-11-26; `CHANGELOG.md:55-63`).
2. **Automated Rust & Playwright Metering Tests (`tests/clients_metering_v033.rs`, `frontend/e2e/gate.mjs`):**
   - `tests/clients_metering_v033.rs:477-478,533-536,660-727,957-990` and `frontend/e2e/gate.mjs:2149-2164,3091-3106` verify API plumbing and UI rendering against synthetic test payloads (`> 0.0`, `45.0`, `95` minutes) — none reflect measured human stopwatch durations.

---

## 7. Blocker Report — What Could Not Be Measured & What Is Needed to Unblock

### What Could Not Be Measured Today
- Manual baseline durations (`seconds`, median, range) and error rates across all 5 workflows (`n = 0` / 15 planned runs).
- End-to-end Ergon durations including human file prep, upload, prompting, output review, and rework (`n = 0` / 15 planned runs).
- Real human error rates on Ergon outputs vs. manual outputs (e.g., false negatives on contract clauses, misclassified field-ops inbox urgencies, missed implicit meeting owners).
- Empirical calibration error (`formula minutes_saved - measured net minutes saved`) for `src/records.rs:2143-2164`.
- Honest role-level monthly hours saved or dollar savings (`est.`).

### Exact Inputs Needed From the Team / Operator to Unblock
1. **Real employee participants (4 roles):** At least 1 employee per role from the ~25-person regional services/installation company (Office/Admin, Account Executive, Project Manager, Installer/Field Ops Coordinator) scheduled for `3` manual + `3` Ergon stopwatch runs per assigned workflow (`30` runs total).
2. **Real company work packets (15 packets total — 3 comparable batches per workflow):** Actual historical CSV invoice/job-costing exports, real vendor/subcontractor contracts, real quote takeoff + supplier pricing notes, real 1-day dispatch/ops inbox exports, and real 30–60 minute project/ops meeting notes/transcripts (kept strictly on-premise / private, never committed to this public repository).
3. **Production-representative Ergon `v0.38.0` test desk & agreed model:** Configured model endpoint (local Ollama model or hosted API) matching intended deployment, with receipt logging active so each timed Ergon run's `(ok_steps, files_written, completion_tokens, latency_ms, estimated_minutes_saved)` is captured verbatim in `timings.csv` `notes`.
4. **Independent QC scorer:** A supervisor or peer reviewer per workflow to verify deliverables against the Done criteria and log `errors` consistently for both `manual` and `ergon` runs.
5. **Optional labor & volume parameters (for dollar projection only, strictly labeled `est.`):** Role-level loaded hourly wage rates (`est.`) and actual weekly/monthly task frequencies (`est.`).
