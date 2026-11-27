# Ergon Before-and-After Stopwatch Capture Sheets

Printable field-capture sheets for running real human before-and-after timing measurements against Ergon `v0.38.0` (`28c3a7354`). Built directly on the 2026-11-26 timing audit protocol (`evidence/timing/2026-11-26/README.md`).

## Mandatory Timing Rules (Read Before Starting Any Sheet)

1. **Real human runs only.** Never enter simulated timings, LLM API latencies alone, or estimates into `timings.csv`.
2. **Full human overhead stays on the clock.**
   - **`mode=manual`:** Start the stopwatch when opening the real source packet. Stop only when the finished deliverable meets every Done criterion below.
   - **`mode=ergon`:** Start the stopwatch **before** exporting/uploading files into Ergon and writing the prompt. Keep the clock running continuously through upload, model wait, FileViewer review, independent verification, and every manual fix/rework edit. Stop only when the finished deliverable meets the **identical** Done criteria.
3. **Never pause the clock** for model latency, retries, formatting fixes, or export steps. If a run fails or is aborted, record it honestly in `notes` — never cherry-pick or omit bad runs.
4. **Canonical CSV schema (`timings.csv`):**
   ```csv
   task,role,mode,run,start,end,seconds,errors,notes
   ```
   - **Allowed `task` values (exact):** `ledger-reconcile`, `contract-risk-audit`, `quote-sheet-build`, `inbox-triage`, `meeting-action-log`
   - **Allowed `role` values (exact):** `Office/Admin`, `Account Executive`, `Project Manager`, `Installer/Field Ops`
   - **Allowed `mode` values (exact):** `manual`, `ergon`
   - **Minimum sample size:** `n >= 3` `manual` runs + `n >= 3` `ergon` runs per workflow (`30` real timed runs total across all 5 workflows).
5. **Privacy rule:** Record anonymized packet IDs (e.g., `pkt-2026-w41`) and role titles only. Keep real company files, customer/vendor names, and employee PII strictly offline — never commit them to git.

---

## Sheet 1 of 5 — `ledger-reconcile` (Job Cost & Vendor Invoice Reconciliation)

| Field | Value |
|---|---|
| **Canonical `task`** | `ledger-reconcile` |
| **Canonical `role`** | `[ ] Office/Admin` |
| **Ergon Skill & Outputs** | `reconcile_ledger` — writes **2 files** (`<title>.md`, `<title> - Summary.csv`) |
| **Required Real Inputs** | 3 distinct closed weekly/monthly historical periods; each packet has 2 real CSV exports (vendor invoices CSV + bank/card or job-costing CSV, `>= 50` line items each). |
| **Counterbalance Order** | Run 1: `[ ] Manual first` / `[ ] Ergon first` · Run 2: `[ ] Manual first` / `[ ] Ergon first` · Run 3: `[ ] Manual first` / `[ ] Ergon first` |

### Run Capture Table (`mode=manual` vs `mode=ergon`)

| Mode | Run | Packet ID (anon) | Start (ISO-8601 UTC) | End (ISO-8601 UTC) | Elapsed `seconds` (`end - start`) | Overhead Checkboxes (All Included on Clock) | Post-Run QC `errors` |
|---|---:|---|---|---|---:|---|---:|
| `manual` | 1 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 2 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 3 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 1 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 2 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 3 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Review` `[ ] Rework` | `___` |

### Ergon Receipt Pairing (`notes` column — copy verbatim from receipt / `usage_ledger.jsonl`)

| Ergon Run | `ok_steps` | `files_written` | `completion_tokens` | `latency_ms` | `estimated_minutes_saved` | `model_id` | Input Size (rows) |
|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 2 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 3 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |

### Independent QC Error Tally (Count 1 Defect Per Occurrence)

- `[ ]` Grand total off by `>= $0.01` vs independent check (`count: ___`)
- `[ ]` Vendor or category subtotal mismatch (`count: ___`)
- `[ ]` Missed duplicate invoice reference or false-positive duplicate (`count: ___`)
- `[ ]` Missed missing-receipt row or cross-file amount mismatch (`count: ___`)
- `[ ]` Broken/wrong source file or row reference, or fabricated line (`count: ___`)

### Verifiable Done Criteria

- Penny-exact grand total and vendor/category rollups matching an independent check.
- 100% of duplicate invoice references, missing receipts, and cross-file amount mismatches accurately cited by file and row.
- Zero fabricated lines; summary rollup and reconciliation memo saved in bookkeeping-ready format.

**Sign-off (no employee names):** Role: `Office/Admin` · Facilitator: `____` · QC Reviewer: `____` · Date: `____-__-__`

---

## Sheet 2 of 5 — `contract-risk-audit` (Subcontractor / Vendor MSA Risk Audit)

| Field | Value |
|---|---|
| **Canonical `task`** | `contract-risk-audit` |
| **Canonical `role`** | `[ ] Account Executive` |
| **Ergon Skill & Outputs** | `audit_contract` — writes **1 file** (`<title>.md`) |
| **Required Real Inputs** | 3 real multi-page vendor, subcontractor, or customer service/installation agreements (`5–20` pages each, `.pdf`, `.docx`, or `.txt`/`.md`) + company required-clause checklist (8 baseline clauses + trade-specific clauses such as lien waivers, warranty, insurance certificates). |
| **Counterbalance Order** | Run 1: `[ ] Manual first` / `[ ] Ergon first` · Run 2: `[ ] Manual first` / `[ ] Ergon first` · Run 3: `[ ] Manual first` / `[ ] Ergon first` |

### Run Capture Table (`mode=manual` vs `mode=ergon`)

| Mode | Run | Packet ID (anon) | Start (ISO-8601 UTC) | End (ISO-8601 UTC) | Elapsed `seconds` (`end - start`) | Overhead Checkboxes (All Included on Clock) | Post-Run QC `errors` |
|---|---:|---|---|---|---:|---|---:|
| `manual` | 1 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/read` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 2 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/read` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 3 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/read` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 1 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload/extract` `[ ] Prompt/run` `[ ] Citation review` `[ ] Rework` | `___` |
| `ergon` | 2 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload/extract` `[ ] Prompt/run` `[ ] Citation review` `[ ] Rework` | `___` |
| `ergon` | 3 | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload/extract` `[ ] Prompt/run` `[ ] Citation review` `[ ] Rework` | `___` |

### Ergon Receipt Pairing (`notes` column)

| Ergon Run | `ok_steps` | `files_written` | `completion_tokens` | `latency_ms` | `estimated_minutes_saved` | `model_id` | Input Size (pages) |
|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 2 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 3 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |

### Independent QC Error Tally (Count 1 Defect Per Occurrence)

- `[ ]` Required clause misclassified (false present or false missing) (`count: ___`)
- `[ ]` Wrong or omitted effective date, term, auto-renewal, notice deadline, or payment term (`count: ___`)
- `[ ]` Missed material commercial trap (liability cap, indemnification, lien waiver, warranty, insurance) (`count: ___`)
- `[ ]` Invalid or mismatched `[L##]` / section citation (`count: ___`)

### Verifiable Done Criteria

- Every required clause accurately classified as present or missing.
- Effective date, initial term, auto-renewal, notice deadline, payment terms, and liability/indemnification caps verified verbatim.
- Every flagged risk carries a valid section/line citation; zero missed material commercial traps; memo ready for leadership/counsel.

**Sign-off (no employee names):** Role: `Account Executive` · Facilitator: `____` · QC Reviewer: `____` · Date: `____-__-__`

---

## Sheet 3 of 5 — `quote-sheet-build` (Installation Quote / Materials BOM Spreadsheet Build)

| Field | Value |
|---|---|
| **Canonical `task`** | `quote-sheet-build` |
| **Canonical `role`** | `[ ] Account Executive` · `[ ] Project Manager` |
| **Ergon Skill & Outputs** | `make_spreadsheet` (`<title>.csv`), optionally paired with `clean_dataset` or `.xlsx` export |
| **Required Real Inputs** | 3 real recent job-scoping packets containing raw site takeoff notes, supplier pricing extracts, and labor/permit/freight notes (`20–60` line items per quote). |
| **Counterbalance Order** | Run 1: `[ ] Manual first` / `[ ] Ergon first` · Run 2: `[ ] Manual first` / `[ ] Ergon first` · Run 3: `[ ] Manual first` / `[ ] Ergon first` |

### Run Capture Table (`mode=manual` vs `mode=ergon`)

| Mode | Run | Role (`Account Executive` / `Project Manager`) | Packet ID (anon) | Start (ISO-8601 UTC) | End (ISO-8601 UTC) | Elapsed `seconds` (`end - start`) | Overhead Checkboxes (All Included on Clock) | Post-Run QC `errors` |
|---|---:|---|---|---|---|---:|---|---:|
| `manual` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open/prep` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Row audit` `[ ] Rework/export` | `___` |
| `ergon` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Row audit` `[ ] Rework/export` | `___` |
| `ergon` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Row audit` `[ ] Rework/export` | `___` |

### Ergon Receipt Pairing (`notes` column)

| Ergon Run | `ok_steps` | `files_written` | `completion_tokens` | `latency_ms` | `estimated_minutes_saved` | `model_id` | Input Size (line items) |
|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 2 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 3 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |

### Independent QC Error Tally (Count 1 Defect Per Occurrence)

- `[ ]` Dropped scoped material, labor, permit, or freight item (`count: ___`)
- `[ ]` Wrong SKU, unit of measure, quantity, or supplier unit cost (`count: ___`)
- `[ ]` Duplicate row or unverified price/markup/tax edit (`count: ___`)
- `[ ]` Broken extended line total, column total, or invalid CSV/XLSX formatting (`count: ___`)

### Verifiable Done Criteria

- 100% of scoped materials, labor, and freight items captured with exact quantities and unit costs matching supplier sources.
- Valid RFC-4180 `.csv` and/or working `.xlsx` totals ready to attach to a customer proposal.
- Zero duplicate rows and zero unverified price edits.

**Sign-off (no employee names):** Role: `__________________` · Facilitator: `____` · QC Reviewer: `____` · Date: `____-__-__`

---

## Sheet 4 of 5 — `inbox-triage` (Daily Operations & Dispatch Inbox Triage)

| Field | Value |
|---|---|
| **Canonical `task`** | `inbox-triage` |
| **Canonical `role`** | `[ ] Office/Admin` · `[ ] Installer/Field Ops` |
| **Ergon Skill & Outputs** | `triage_inbox` — writes **1 file** (`<title>.md`) |
| **Required Real Inputs** | 3 real single-day batches (`25–60` messages each) from the shared operations/dispatch/support inbox (exported as `.md`, `.txt`, `.csv`, or `.json`, or ingested via `imap_poll`), covering field blockers, backorders, change requests, quote approvals, and routine notices. |
| **Counterbalance Order** | Run 1: `[ ] Manual first` / `[ ] Ergon first` · Run 2: `[ ] Manual first` / `[ ] Ergon first` · Run 3: `[ ] Manual first` / `[ ] Ergon first` |

### Run Capture Table (`mode=manual` vs `mode=ergon`)

| Mode | Run | Role (`Office/Admin` / `Installer/Field Ops`) | Packet ID (anon) | Start (ISO-8601 UTC) | End (ISO-8601 UTC) | Elapsed `seconds` (`end - start`) | Overhead Checkboxes (All Included on Clock) | Post-Run QC `errors` |
|---|---:|---|---|---|---|---:|---|---:|
| `manual` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open batch` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open batch` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open batch` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Full-batch review` `[ ] Rework` | `___` |
| `ergon` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Full-batch review` `[ ] Rework` | `___` |
| `ergon` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Export/upload` `[ ] Prompt/run` `[ ] Full-batch review` `[ ] Rework` | `___` |

### Ergon Receipt Pairing (`notes` column)

| Ergon Run | `ok_steps` | `files_written` | `completion_tokens` | `latency_ms` | `estimated_minutes_saved` | `model_id` | Input Size (messages) |
|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 2 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 3 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |

### Independent QC Error Tally (Count 1 Defect Per Occurrence)

- `[ ]` Missed genuinely urgent customer or job-site escalation (e.g., crew locked out, permit delay, truck shortage) (`count: ___`)
- `[ ]` Dropped message not accounted for in Urgent, Action, or FYI (`count: ___`)
- `[ ]` Missing or wrong real assignee on an Urgent/Action item (`count: ___`)
- `[ ]` Generic/non-actionable next step left unedited (`count: ___`)

### Verifiable Done Criteria

- Every message in the batch accounted for in **Urgent**, **Action**, or **FYI**.
- Zero missed genuinely urgent customer or job-site escalations.
- Every Urgent and Action item carries a real assignee and concrete next step ready for dispatch execution.

**Sign-off (no employee names):** Role: `__________________` · Facilitator: `____` · QC Reviewer: `____` · Date: `____-__-__`

---

## Sheet 5 of 5 — `meeting-action-log` (Site-Walk / Weekly Ops Meeting Summary & Action Log)

| Field | Value |
|---|---|
| **Canonical `task`** | `meeting-action-log` |
| **Canonical `role`** | `[ ] Project Manager` · `[ ] Installer/Field Ops` |
| **Ergon Skill & Outputs** | `summarize_meeting` — writes **1 file** (`<title>.md`) |
| **Required Real Inputs** | 3 real raw meeting notes or audio transcripts from `30–60` minute weekly production meetings, job kickoffs, or field site-walk debriefs (`.md` or `.txt`). |
| **Counterbalance Order** | Run 1: `[ ] Manual first` / `[ ] Ergon first` · Run 2: `[ ] Manual first` / `[ ] Ergon first` · Run 3: `[ ] Manual first` / `[ ] Ergon first` |

### Run Capture Table (`mode=manual` vs `mode=ergon`)

| Mode | Run | Role (`Project Manager` / `Installer/Field Ops`) | Packet ID (anon) | Start (ISO-8601 UTC) | End (ISO-8601 UTC) | Elapsed `seconds` (`end - start`) | Overhead Checkboxes (All Included on Clock) | Post-Run QC `errors` |
|---|---:|---|---|---|---|---:|---|---:|
| `manual` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open notes` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open notes` `[ ] Review` `[ ] Rework` | `___` |
| `manual` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Open notes` `[ ] Review` `[ ] Rework` | `___` |
| `ergon` | 1 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Citation/owner review` `[ ] Rework` | `___` |
| `ergon` | 2 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Citation/owner review` `[ ] Rework` | `___` |
| `ergon` | 3 | `__________________` | `____________` | `____-__-__T__:__:__Z` | `____-__-__T__:__:__Z` | `______` | `[ ] Upload` `[ ] Prompt/run` `[ ] Citation/owner review` `[ ] Rework` | `___` |

### Ergon Receipt Pairing (`notes` column)

| Ergon Run | `ok_steps` | `files_written` | `completion_tokens` | `latency_ms` | `estimated_minutes_saved` | `model_id` | Input Size (lines/min) |
|---:|---:|---:|---:|---:|---:|---|---:|
| 1 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 2 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |
| 3 | `___` | `___` | `______` | `______` | `______` | `________________` | `_____` |

### Independent QC Error Tally (Count 1 Defect Per Occurrence)

- `[ ]` Missed firm decision or unresolved site blocker (`count: ___`)
- `[ ]` Invented commitment, attendee, or deadline not supported by notes (`count: ___`)
- `[ ]` Action item left `Unassigned`, wrong owner, or missing explicit due date (`count: ___`)
- `[ ]` Invalid `[L##]` decision citation or follow-up draft requiring further revision (`count: ___`)

### Verifiable Done Criteria

- All real decisions, owners, deadlines, and site blockers captured accurately with zero invented commitments.
- Every action item carries a single owner and explicit due date.
- Per-owner follow-up drafts are accurate and ready to send without further revision.

**Sign-off (no employee names):** Role: `__________________` · Facilitator: `____` · QC Reviewer: `____` · Date: `____-__-__`

---

## CSV Transcription Reference (`timings.csv`)

Transcribe completed sheets into `evidence/timing/<YYYY-MM-DD>/timings.csv` (keeping the exact 9-column header) and validate with `evidence/timing/kit/analyze_timings.py`:

```csv
task,role,mode,run,start,end,seconds,errors,notes
```

Recommended `notes` format for `mode=ergon` rows (anonymized packet ID + verbatim receipt fields):
```text
pkt=<anon-id>; input=<size>; model=<model_id>; ok_steps=<n>; files_written=<n>; completion_tokens=<n>; latency_ms=<n>; estimated_minutes_saved=<n.nn>; order=<manual-first|ergon-first>
```
