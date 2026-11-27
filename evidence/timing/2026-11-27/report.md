# Ergon Timing Study Analysis Report

- **Source CSV:** `timings.csv`
- **Study Status:** **`READY`**
- **Total Timed Runs:** `30` (`manual=15`, `ergon=15`)
- **Gate Coverage (`n >= 3` per mode):** `5/5` workflows ready, `4/4` roles ready

## Per-Workflow Summary (Median Minutes, Min–Max Range, Net Delta, Error Rate)

| Task | Status | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (Manual vs Ergon) |
|---|---|---|---|---|---|
| `ledger-reconcile` | `READY` | 2.25 min (2.23–2.38, n=3) | 2.82 min (2.02–5.37, n=3) | -0.57 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |
| `contract-risk-audit` | `READY` | 2.05 min (1.42–2.38, n=3) | 1.53 min (0.77–2.65, n=3) | +0.52 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |
| `quote-sheet-build` | `READY` | 1.27 min (1.25–1.27, n=3) | 4.80 min (3.37–5.52, n=3) | -3.53 min | Manual 0.00/run (0 err) vs Ergon 0.33/run (1 err) |
| `inbox-triage` | `READY` | 4.05 min (3.73–5.52, n=3) | 0.90 min (0.65–1.63, n=3) | +3.15 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |
| `meeting-action-log` | `READY` | 0.27 min (0.22–0.27, n=3) | 1.55 min (1.30–1.90, n=3) | -1.28 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |

## Per-Role Summary

| Role | Status | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (Manual vs Ergon) |
|---|---|---|---|---|---|
| Office/Admin | `READY` | 2.32 min (2.23–5.52, n=4) | 2.42 min (0.90–5.37, n=4) | -0.10 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |
| Account Executive | `READY` | 1.73 min (1.25–2.38, n=4) | 2.09 min (0.77–4.80, n=4) | -0.36 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |
| Project Manager | `READY` | 0.77 min (0.22–1.27, n=4) | 2.63 min (1.55–5.52, n=4) | -1.87 min | Manual 0.00/run (0 err) vs Ergon 0.25/run (1 err) |
| Installer/Field Ops | `READY` | 3.73 min (0.27–4.05, n=3) | 1.30 min (0.65–1.63, n=3) | +2.43 min | Manual 0.00/run (0 err) vs Ergon 0.00/run (0 err) |

