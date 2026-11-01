# Job 03 run-evidence skeleton — RHL dress rehearsal (pass 1 + pass 2)

**Status: SKELETON ONLY. NO PASS HAS BEEN RECORDED. NOTHING HERE IS EVIDENCE.** This is the directory and template set
for the rehearsal's two passes (done-bar: "end-to-end tested multiple times"). It is prepared ahead of the RUN so that
the moment the installed output exists, the recorder fills a known shape instead of inventing one. The step spec and the
operating rules are in the repo files [REHEARSAL-R0-R7.md](../../../docs/acceptance/demo-company/REHEARSAL-R0-R7.md) and
[RUN-SHEET.md](../../../docs/acceptance/demo-company/RUN-SHEET.md) (branch `docs/l5-acceptance-r0-r7`, PR #10; not merged
at the time of filing — if the paths differ after merge, the repo copy wins).

## What is here

| Path | What it is |
|---|---|
| `pass-1/`, `pass-2/` | One directory per pass. `pass-2` is the mandatory second pass from scratch (R7). |
| `pass-<n>/index.md` | The pass index: run metadata, verdict per R0–R7 step, secrets check, checker sign-off block. |
| `pass-<n>/R0-pre/` | Team-side pre-pass evidence: host fingerprint, package SHA-256, run-config acceptance, project id, absence of company state. `r0-preflight.sh` fills these. |
| `pass-<n>/R1/` … `R6/` | One `notes.md` per operator step: action, expected, actual, word, evidence files, timestamp, blocker. Screens/logs go beside it. |
| `pass-<n>/R7-close/` | Close-out: state record, host teardown, pass diff (`pass-2` only) and notes. |
| `check-skeleton.mjs` | Validates this skeleton (structure, required fields, no secret-like content). Run it after any edit. |
| `r0-preflight.sh` | Generic-host R0 preflight: writes the R0 evidence files. Read-only except its own output directory; no secrets, no sudo, no network calls. |

## Rules that bind the recorder

- **Fill only from a driven pass.** A pass that was not driven is a progress report, never a completion. Never pre-fill
  "clean".
- **Word vocabulary:** `clean` / `failed` / `pending` / `stand-in` / `not run`. A missing product piece is `pending`
  (PACKAGE-PENDING), not `failed`; a Windows-only surface is `stand-in`/`pending`, never `clean`.
- **No secrets, ever.** No credential values, API keys, tokens, passwords, or personal phone content in any file here.
  Only names/labels/ids may appear. `check-skeleton.mjs` refuses secret-like strings.
- **Maker never checker.** Team A records; a fresh-context checker who built no part of the candidate signs the index.
  This lane never signs its own rehearsal.
- **Timestamp in UTC** (`YYYY-MM-DDTHH:MM:SSZ`) on every note.
- **The second pass is mandatory.** `pass-2` repeats R0–R6 from scratch on a fresh host; `R7-close/pass-diff.md` names
  every difference.

## Use

```bash
# 1. On the fresh host, before the clock (team side):
bash r0-preflight.sh /path/to/pass-1/R0-pre \
  --repo /path/to/ergon \
  --package /path/to/ergon-package-<ver>.tgz

# 2. Fill each step's notes.md as the operator completes it (screens/logs beside the notes).
# 3. Before handing the pack to the checker:
node check-skeleton.mjs            # must exit 0 with "SKELETON OK"
# 4. Checker signs pass-<n>/index.md; the Lead routes the verdict.
```

## Relation to the run sheet gates

This skeleton is produced under gate G5 preparation only. It does not unlock the RUN: G1 (Lane 1's installed output),
G2 (C1–C6 freeze), G3 (client-shape arbitration), G4 (host + company data), G5 (checker named), G6 (run config
accepted), G7 (no-secrets rule) must all be true before the clock starts. Until then this directory stays empty of
claims — templates only.
