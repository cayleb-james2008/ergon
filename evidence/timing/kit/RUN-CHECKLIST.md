# Day-Of Facilitator Checklist (`RUN-CHECKLIST.md`)

Operational runbook for facilitating real employee before-and-after stopwatch measurements for Ergon `v0.38.0` (`28c3a7354`) across the **5 benchmark workflows** and **4 operational roles**:

- **5 canonical `task` IDs:** `ledger-reconcile`, `contract-risk-audit`, `quote-sheet-build`, `inbox-triage`, `meeting-action-log`
- **4 canonical `role` names:** `Office/Admin`, `Account Executive`, `Project Manager`, `Installer/Field Ops`
- **Target minimum sample:** `n >= 3` `manual` runs + `n >= 3` `ergon` runs per workflow (`30` timed human runs total across all 4 roles).

---

## 1. Workstation & Environment Prep (T-30 Minutes)

- [ ] **Pin and verify the Ergon build:** Confirm `ergon --version` reports `0.38.0` (`28c3a7354`) and confirm `src/records.rs` has **not** been modified before measurement.
- [ ] **Configure the agreed production-representative model endpoint:** Verify whether the pilot uses a local gateway (e.g., Ollama) or hosted API, and record the exact `model_id` that will be logged in every `mode=ergon` row's `notes`.
- [ ] **Prepare clean isolated workspaces per role:** Give each participant a clean workspace/profile without pre-uploaded answers from prior tests, and confirm receipt logging (`usage_ledger.jsonl` / `work.json`) is active.
- [ ] **Verify normal manual tools are ready:** Open Excel/Google Sheets, PDF viewer, Word/Docs, and the exported inbox/notes folder in a clean default state so neither mode suffers artificial login delays unrelated to the task.
- [ ] **Sync system clocks & stopwatch:** Ensure workstation clocks are NTP-synced so ISO-8601 `start` and `end` timestamps match integer `seconds = end - start` within `<= 2` seconds.
- [ ] **Print or open `STOPWATCH-SHEETS.md`:** Have one capture sheet ready per workflow and confirm every participant understands the workflow's **Verifiable Done Criteria** before the clock starts.

---

## 2. Real Work Packets & Counterbalancing (A/B Crossover)

- [ ] **Stage 15 real historical work packets offline (3 comparable packets per workflow):**
  - `ledger-reconcile` (`Office/Admin`): 3 closed weeks/months of real vendor invoice + bank/job-costing CSV pairs (`>= 50` rows each).
  - `contract-risk-audit` (`Account Executive`): 3 real 5–20 page vendor/subcontractor/customer agreements + company clause checklist.
  - `quote-sheet-build` (`Account Executive` / `Project Manager`): 3 real job-scoping packets (`20–60` line items each).
  - `inbox-triage` (`Office/Admin` / `Installer/Field Ops`): 3 real single-day dispatch/ops inbox exports (`25–60` messages each).
  - `meeting-action-log` (`Project Manager` / `Installer/Field Ops`): 3 real 30–60 minute weekly ops/site-walk notes or transcripts.
- [ ] **Assign anonymized packet IDs (`pkt-01` .. `pkt-15`)** on a private offline key sheet that never enters git.
- [ ] **Counterbalance order to prevent learning and fatigue bias:**
  - Alternate which mode runs first across runs and across participants:
    - Run 1: `manual` first, then `ergon` on a distinct comparable packet (or A/B crossover with another participant on the paired packet).
    - Run 2: `ergon` first, then `manual` on a distinct comparable packet.
    - Run 3: `manual` first (or alternate per participant so half the sessions begin with `manual` and half begin with `ergon`).
  - **Never** have the same employee immediately repeat the exact same document in the second mode (memorization invalidates the timing). If reusing packets for a paired A/B design, assign Participant A (`manual` on Packet X, `ergon` on Packet Y) and Participant B (`ergon` on Packet X, `manual` on Packet Y), or separate paired repeats by at least 2 weeks.
  - Log `order=manual-first` or `order=ergon-first` in the CSV `notes` field.

---

## 3. Live Stopwatch Execution Rules

- [ ] **Start the clock before ALL work begins:**
  - For `mode=manual`: click Start the moment the participant opens the packet files.
  - For `mode=ergon`: click Start **before** the participant exports/uploads files into Ergon, chooses a workflow/skill, or writes the prompt.
- [ ] **Keep the clock running continuously:** Never pause for CSV/PDF conversion, file upload, LLM thinking/streaming, retries, FileViewer inspection, verification against source files, or manual edits/rework.
- [ ] **Stop the clock ONLY when the participant declares the deliverable Done** and saved in its final handoff format matching the sheet's Done criteria.
- [ ] **Log failures and aborts honestly:** If Ergon errors out, hits a retry loop, or outputs unusable formatting that forces manual recovery, keep the clock running through recovery (or log the aborted attempt in `notes` and do not delete the row to inflate averages).
- [ ] **Check timestamp arithmetic immediately:** Verify `seconds == round(end - start)` before moving to the next run.

---

## 4. Pairing Ergon Receipts 1:1

For **every** `mode=ergon` row, capture the exact run telemetry from Ergon's receipt / `usage_ledger.jsonl` right after the run finishes so `src/records.rs`'s uncalibrated formula can be compared 1:1 against measured `manual - ergon` seconds:

- [ ] Record `ok_steps` (number of completed tool/pipeline steps)
- [ ] Record `files_written` (number of workspace files written)
- [ ] Record `completion_tokens` (`0` for deterministic pipeline steps; exact token count for chat runs)
- [ ] Record `latency_ms` (unattended run elapsed ms)
- [ ] Record `estimated_minutes_saved` (verbatim from the Ergon receipt)
- [ ] Record `model_id` and `input=<rows|pages|items|messages|minutes>`
- [ ] Write them into the `notes` column using key=value pairs:
  ```text
  pkt=pkt-01; input=74rows; model=qwen2.5:14b; ok_steps=2; files_written=2; completion_tokens=612; latency_ms=8420; estimated_minutes_saved=29.45; order=ergon-first
  ```

---

## 5. Independent QC Scoring Rules (`errors`)

- [ ] **Use an independent reviewer** (supervisor or peer who did not author the run; ideally blind to whether an output was produced manually or with Ergon).
- [ ] **Separate self-caught rework from escaped defects:**
  - Defects the participant catches and fixes **before** stopping the stopwatch add to **`seconds`** (rework time), **not** to `errors`.
  - Defects that survive into the final "Done" deliverable add `1` each to **`errors`**.
- [ ] **Score identically across `manual` and `ergon`:**
  - `ledger-reconcile`: +1 per wrong grand/vendor/category total (`>= $0.01`), missed duplicate ref, missed missing receipt, missed cross-file mismatch, bad row citation, or fabricated row.
  - `contract-risk-audit`: +1 per misclassified required clause, wrong date/term/notice/cap extraction, missed material commercial trap, or invalid section/line citation.
  - `quote-sheet-build`: +1 per dropped item, wrong SKU/qty/unit cost, duplicate row, unverified price edit, or broken total/CSV/XLSX export.
  - `inbox-triage`: +1 per missed urgent field/customer escalation, unaccounted message, missing/wrong real assignee, or non-actionable generic next step.
  - `meeting-action-log`: +1 per missed decision/blocker, invented commitment/date, unassigned or misassigned action item, wrong due date, or broken citation/unsendable follow-up draft.

---

## 6. Privacy & Public Repository Hygiene Rules

- [ ] **Keep all real company work packets strictly offline / on-premise.** Never commit source CSVs, contracts, BOMs, emails, meeting transcripts, or Ergon workspace outputs into this public repository.
- [ ] **Scrub `notes` before saving `timings.csv`:** Strip all customer names, vendor names, employee names, email addresses, job addresses, invoice numbers, API keys, tokens, and internal company names.
- [ ] **Keep `timings.csv` strictly empirical:** Never add placeholder, synthetic, or simulated rows to `timings.csv`.
- [ ] **Do not edit `src/records.rs` until `n >= 3` real runs exist** for both `manual` and `ergon` across the measured workflows and leadership has reviewed the uncalibrated side-by-side comparison.

---

## 7. Post-Session Transcription & Analysis

1. Save transcribed runs into `evidence/timing/<YYYY-MM-DD>/timings.csv` (matching exact header `task,role,mode,run,start,end,seconds,errors,notes`).
2. Run the standard-library analyzer to validate schema, enforce the `n >= 3` per-mode gate, and compute medians, min–max ranges, net deltas, and error rates:
   ```bash
   python3 evidence/timing/kit/analyze_timings.py evidence/timing/<YYYY-MM-DD>/timings.csv
   ```
3. Optional: once `n >= 3` per mode is collected, pass an offline role-rate & task-volume JSON file (kept strictly labeled `est.`):
   ```bash
   python3 evidence/timing/kit/analyze_timings.py \
     evidence/timing/<YYYY-MM-DD>/timings.csv \
     --rates-json /path/to/offline-rates.json
   ```
   The script **refuses** to extrapolate annual hours or USD savings for any workflow or role with `n < 3` in either mode.
