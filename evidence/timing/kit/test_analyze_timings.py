#!/usr/bin/env python3
"""Unit tests for evidence/timing/kit/analyze_timings.py.

IMPORTANT — UNIT-TEST SYNTHETIC FIXTURES ONLY:
Every CSV and JSON fixture constructed in this test file is ephemeral synthetic
test data written exclusively inside tempfile.TemporaryDirectory() and deleted
immediately after each test case. These tests NEVER create, edit, or append to
any real timings.csv in the repository.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

KIT_DIR = Path(__file__).resolve().parent
if str(KIT_DIR) not in sys.path:
    sys.path.insert(0, str(KIT_DIR))

import analyze_timings as at  # noqa: E402

HEADER = "task,role,mode,run,start,end,seconds,errors,notes\n"


def _row(
    task: str,
    role: str,
    mode: str,
    run: int,
    seconds: int,
    errors: int = 0,
    notes: str = "unit-test-synthetic-fixture",
    start_hour: int = 9,
) -> str:
    start_sec = 0
    total = start_sec + seconds
    eh = start_hour + (total // 3600)
    em = (total % 3600) // 60
    es = total % 60
    start_iso = f"2026-11-27T{start_hour:02d}:00:00Z"
    end_iso = f"2026-11-27T{eh:02d}:{em:02d}:{es:02d}Z"
    return f"{task},{role},{mode},{run},{start_iso},{end_iso},{seconds},{errors},{notes}\n"


class TestAnalyzeTimings(unittest.TestCase):
    """Tests using temporary synthetic unit-test fixtures only."""

    def _write_temp(self, tmpdir: str, name: str, content: str) -> Path:
        p = Path(tmpdir) / name
        p.write_text(content, encoding="utf-8")
        return p

    def test_empty_header_only_csv_returns_blocked_n0(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            csv_path = self._write_temp(tmp, "timings.csv", HEADER)
            rates_path = self._write_temp(
                tmp,
                "rates.json",
                json.dumps(
                    {
                        "roles": {"Office/Admin": {"hourly_rate_usd": 40}},
                        "tasks": {
                            "ledger-reconcile": {
                                "runs_per_year": 52,
                                "role": "Office/Admin",
                            }
                        },
                    }
                ),
            )

            rows = at.load_and_validate_csv(csv_path)
            self.assertEqual(rows, [])
            rates = at.load_and_validate_rates(rates_path)
            report = at.analyze(rows, rates=rates, source_label="timings.csv")

            self.assertEqual(report["status"], "BLOCKED")
            self.assertEqual(report["total_runs"], 0)
            self.assertEqual(report["tasks_ready"], 0)
            self.assertEqual(report["roles_ready"], 0)
            self.assertTrue(any("BLOCKED" in w for w in report["warnings"]))

            for task in at.CANONICAL_TASKS:
                self.assertEqual(report["by_task"][task]["status"], "UNMEASURED")
                self.assertEqual(report["by_task"][task]["manual"]["n"], 0)
                self.assertEqual(report["by_task"][task]["ergon"]["n"], 0)
                self.assertIsNone(report["by_task"][task]["median_delta_minutes"])

            # Extrapolation MUST be refused at n=0 even when rates are supplied
            proj = report["projections_est"]
            self.assertIsNotNone(proj)
            self.assertFalse(proj["extrapolation_allowed_for_all_configured_tasks"])
            self.assertIsNone(proj["totals_est"])
            self.assertEqual(
                proj["by_task"]["ledger-reconcile"]["status"],
                "REFUSED_INSUFFICIENT_N",
            )
            self.assertIsNone(
                proj["by_task"]["ledger-reconcile"]["est_hours_saved_per_year"]
            )

            md = at.render_markdown(report, include_json_block=True)
            self.assertIn("**`BLOCKED`**", md)
            self.assertIn("UNMEASURED (n=0)", md)
            self.assertIn("REFUSED (n < 3)", md)

    def test_partial_sample_warns_and_refuses_extrapolation(self) -> None:
        # Synthetic unit-test fixture: manual n=3, ergon n=2 -> MUST refuse extrapolation
        csv_text = HEADER + "".join(
            [
                _row("ledger-reconcile", "Office/Admin", "manual", 1, 1800, 1),
                _row("ledger-reconcile", "Office/Admin", "manual", 2, 2100, 0),
                _row("ledger-reconcile", "Office/Admin", "manual", 3, 2400, 2),
                _row("ledger-reconcile", "Office/Admin", "ergon", 1, 600, 0),
                _row("ledger-reconcile", "Office/Admin", "ergon", 2, 900, 1),
            ]
        )
        with tempfile.TemporaryDirectory() as tmp:
            csv_path = self._write_temp(tmp, "partial.csv", csv_text)
            rates_path = self._write_temp(
                tmp,
                "rates.json",
                json.dumps(
                    {
                        "roles": {"Office/Admin": {"hourly_rate_usd": 45}},
                        "tasks": {
                            "ledger-reconcile": {
                                "runs_per_year": 52,
                                "role": "Office/Admin",
                            }
                        },
                    }
                ),
            )
            rows = at.load_and_validate_csv(csv_path)
            rates = at.load_and_validate_rates(rates_path)
            report = at.analyze(rows, rates=rates)

            self.assertEqual(report["status"], "INCOMPLETE")
            t = report["by_task"]["ledger-reconcile"]
            self.assertEqual(t["status"], "INSUFFICIENT_N")
            self.assertFalse(t["meets_min_n"])
            self.assertEqual(t["manual"]["median_minutes"], 35.0)
            self.assertEqual(t["ergon"]["median_minutes"], 12.5)
            self.assertEqual(t["median_delta_minutes"], 22.5)
            self.assertTrue(
                any("ledger-reconcile" in w and "REFUSED" in w for w in report["warnings"])
            )
            self.assertEqual(
                report["projections_est"]["by_task"]["ledger-reconcile"]["status"],
                "REFUSED_INSUFFICIENT_N",
            )
            self.assertIsNone(report["projections_est"]["totals_est"])

    def test_exact_median_range_delta_error_rate_and_est_projection(self) -> None:
        # Synthetic unit-test fixture covering all 5 canonical workflows and all 4 canonical roles
        # with n=3 manual + n=3 ergon per workflow (30 synthetic fixture runs total).
        fixture_rows = [
            # WF-1 ledger-reconcile (Office/Admin): manual [30, 40, 50] med=40; ergon [10, 15, 20] med=15 -> delta=+25.00
            _row("ledger-reconcile", "Office/Admin", "manual", 1, 1800, 1),
            _row("ledger-reconcile", "Office/Admin", "manual", 2, 2400, 2),
            _row("ledger-reconcile", "Office/Admin", "manual", 3, 3000, 0),
            _row(
                "ledger-reconcile",
                "Office/Admin",
                "ergon",
                1,
                600,
                0,
                "pkt=syn-1; estimated_minutes_saved=23.00",
            ),
            _row(
                "ledger-reconcile",
                "Office/Admin",
                "ergon",
                2,
                900,
                1,
                "pkt=syn-2; estimated_minutes_saved=25.00",
            ),
            _row(
                "ledger-reconcile",
                "Office/Admin",
                "ergon",
                3,
                1200,
                0,
                "pkt=syn-3; estimated_minutes_saved=27.00",
            ),
            # WF-2 contract-risk-audit (Account Executive): manual [45, 60, 75] med=60; ergon [25, 30, 35] med=30 -> delta=+30.00
            _row("contract-risk-audit", "Account Executive", "manual", 1, 2700, 2),
            _row("contract-risk-audit", "Account Executive", "manual", 2, 3600, 1),
            _row("contract-risk-audit", "Account Executive", "manual", 3, 4500, 0),
            _row("contract-risk-audit", "Account Executive", "ergon", 1, 1500, 0),
            _row("contract-risk-audit", "Account Executive", "ergon", 2, 1800, 1),
            _row("contract-risk-audit", "Account Executive", "ergon", 3, 2100, 0),
            # WF-3 quote-sheet-build (split AE + PM): manual [20, 30, 40] med=30; ergon [12, 18, 24] med=18 -> delta=+12.00
            _row("quote-sheet-build", "Account Executive", "manual", 1, 1200, 0),
            _row("quote-sheet-build", "Project Manager", "manual", 2, 1800, 1),
            _row("quote-sheet-build", "Project Manager", "manual", 3, 2400, 2),
            _row("quote-sheet-build", "Account Executive", "ergon", 1, 720, 0),
            _row("quote-sheet-build", "Project Manager", "ergon", 2, 1080, 0),
            _row("quote-sheet-build", "Project Manager", "ergon", 3, 1440, 1),
            # WF-4 inbox-triage (split Office/Admin + Installer/Field Ops): negative delta test!
            # manual [10, 12, 14] med=12; ergon [12, 15, 18] med=15 -> delta=-3.00 (Ergon slower)
            _row("inbox-triage", "Office/Admin", "manual", 1, 600, 0),
            _row("inbox-triage", "Installer/Field Ops", "manual", 2, 720, 1),
            _row("inbox-triage", "Installer/Field Ops", "manual", 3, 840, 0),
            _row("inbox-triage", "Office/Admin", "ergon", 1, 720, 1),
            _row("inbox-triage", "Installer/Field Ops", "ergon", 2, 900, 2),
            _row("inbox-triage", "Installer/Field Ops", "ergon", 3, 1080, 0),
            # WF-5 meeting-action-log (split PM + Installer/Field Ops): manual [15, 21, 27] med=21; ergon [6, 9, 12] med=9 -> delta=+12.00
            _row("meeting-action-log", "Project Manager", "manual", 1, 900, 1),
            _row("meeting-action-log", "Installer/Field Ops", "manual", 2, 1260, 0),
            _row("meeting-action-log", "Project Manager", "manual", 3, 1620, 0),
            _row("meeting-action-log", "Project Manager", "ergon", 1, 360, 0),
            _row("meeting-action-log", "Installer/Field Ops", "ergon", 2, 540, 0),
            _row("meeting-action-log", "Project Manager", "ergon", 3, 720, 0),
        ]

        rates_obj = {
            "roles": {
                "Office/Admin": {"hourly_rate_usd": 40.0},
                "Account Executive": {"hourly_rate_usd": 60.0},
                "Project Manager": {"hourly_rate_usd": 70.0},
                "Installer/Field Ops": {"hourly_rate_usd": 50.0},
            },
            "tasks": {
                "ledger-reconcile": {"runs_per_year": 48, "role": "Office/Admin"},
                "contract-risk-audit": {"runs_per_year": 24, "role": "Account Executive"},
                "quote-sheet-build": {"runs_per_year": 60, "role": "Project Manager"},
                "inbox-triage": {"runs_per_year": 200, "role": "Installer/Field Ops"},
                "meeting-action-log": {"runs_per_year": 50, "role": "Project Manager"},
            },
        }

        with tempfile.TemporaryDirectory() as tmp:
            csv_path = self._write_temp(tmp, "full.csv", HEADER + "".join(fixture_rows))
            rates_path = self._write_temp(tmp, "rates.json", json.dumps(rates_obj))

            rows = at.load_and_validate_csv(csv_path)
            self.assertEqual(len(rows), 30)
            rates = at.load_and_validate_rates(rates_path)
            report = at.analyze(rows, rates=rates)

            self.assertEqual(report["status"], "READY")
            self.assertEqual(report["tasks_ready"], 5)
            self.assertEqual(report["roles_ready"], 4)
            self.assertEqual(report["warnings"], [])

            # Check WF-1 exact stats & receipt formula delta
            wf1 = report["by_task"]["ledger-reconcile"]
            self.assertEqual(wf1["manual"]["median_minutes"], 40.0)
            self.assertEqual(wf1["manual"]["min_minutes"], 30.0)
            self.assertEqual(wf1["manual"]["max_minutes"], 50.0)
            self.assertEqual(wf1["manual"]["error_rate_per_run"], 1.0)
            self.assertEqual(wf1["ergon"]["median_minutes"], 15.0)
            self.assertEqual(wf1["ergon"]["min_minutes"], 10.0)
            self.assertEqual(wf1["ergon"]["max_minutes"], 20.0)
            self.assertAlmostEqual(wf1["ergon"]["error_rate_per_run"], 0.3333, places=4)
            self.assertEqual(wf1["median_delta_minutes"], 25.0)
            self.assertEqual(wf1["ergon"]["median_formula_minutes_saved"], 25.0)
            self.assertEqual(wf1["formula_vs_measured_delta_minutes"], 0.0)

            # Check WF-4 negative delta preserved honestly
            wf4 = report["by_task"]["inbox-triage"]
            self.assertEqual(wf4["median_delta_minutes"], -3.0)

            # Check per-role aggregation across all 4 roles
            # Office/Admin: manual [30,40,50,10] -> sorted [10,30,40,50], median=35.0; ergon [10,15,20,12] -> sorted [10,12,15,20], median=13.5 -> delta=21.5
            oa = report["by_role"]["Office/Admin"]
            self.assertTrue(oa["meets_min_n"])
            self.assertEqual(oa["manual"]["n"], 4)
            self.assertEqual(oa["ergon"]["n"], 4)
            self.assertEqual(oa["manual"]["median_minutes"], 35.0)
            self.assertEqual(oa["ergon"]["median_minutes"], 13.5)
            self.assertEqual(oa["median_delta_minutes"], 21.5)

            # Project Manager: manual [30,40,15,27] -> [15,27,30,40] med=28.5; ergon [18,24,6,12] -> [6,12,18,24] med=15.0 -> delta=13.5
            pm = report["by_role"]["Project Manager"]
            self.assertTrue(pm["meets_min_n"])
            self.assertEqual(pm["manual"]["median_minutes"], 28.5)
            self.assertEqual(pm["ergon"]["median_minutes"], 15.0)
            self.assertEqual(pm["median_delta_minutes"], 13.5)

            # Installer/Field Ops: manual [12,14,21] med=14.0; ergon [15,18,9] med=15.0 -> delta=-1.0
            fo = report["by_role"]["Installer/Field Ops"]
            self.assertTrue(fo["meets_min_n"])
            self.assertEqual(fo["manual"]["median_minutes"], 14.0)
            self.assertEqual(fo["ergon"]["median_minutes"], 15.0)
            self.assertEqual(fo["median_delta_minutes"], -1.0)

            # Check est. projections math:
            # WF-1: (25/60)*48 = 20.00 hr * $40 = $800.00
            # WF-2: (30/60)*24 = 12.00 hr * $60 = $720.00
            # WF-3: (12/60)*60 = 12.00 hr * $70 = $840.00
            # WF-4: (-3/60)*200 = -10.00 hr * $50 = -$500.00
            # WF-5: (12/60)*50 = 10.00 hr * $70 = $700.00
            # Total: 44.00 hr, $2560.00
            proj = report["projections_est"]
            self.assertTrue(proj["extrapolation_allowed_for_all_configured_tasks"])
            self.assertEqual(proj["by_task"]["ledger-reconcile"]["label"], "est.")
            self.assertEqual(proj["by_task"]["ledger-reconcile"]["est_hours_saved_per_year"], 20.0)
            self.assertEqual(proj["by_task"]["ledger-reconcile"]["est_usd_saved_per_year"], 800.0)
            self.assertEqual(proj["by_task"]["inbox-triage"]["est_hours_saved_per_year"], -10.0)
            self.assertEqual(proj["by_task"]["inbox-triage"]["est_usd_saved_per_year"], -500.0)
            self.assertEqual(proj["totals_est"]["est_hours_saved_per_year"], 44.0)
            self.assertEqual(proj["totals_est"]["est_usd_saved_per_year"], 2560.0)

            md = at.render_markdown(report, include_json_block=True)
            self.assertIn("(`est.`)", md)
            self.assertIn("+44.00 hr/yr (est.)", md)
            self.assertIn("$2,560.00/yr (est.)", md)

    def test_malformed_rows_and_schema_rejected(self) -> None:
        bad_cases = [
            ("wrong_header", "task,role,mode\n"),
            ("empty_file", ""),
            (
                "unknown_task",
                HEADER + _row("not-a-task", "Office/Admin", "manual", 1, 60),
            ),
            (
                "unknown_role",
                HEADER + _row("ledger-reconcile", "CEO", "manual", 1, 60),
            ),
            (
                "bad_mode",
                HEADER + _row("ledger-reconcile", "Office/Admin", "simulated", 1, 60),
            ),
            (
                "zero_run",
                HEADER + _row("ledger-reconcile", "Office/Admin", "manual", 0, 60),
            ),
            (
                "duplicate_run_key",
                HEADER
                + _row("ledger-reconcile", "Office/Admin", "manual", 1, 60)
                + _row("ledger-reconcile", "Office/Admin", "manual", 1, 120),
            ),
            (
                "end_before_start",
                HEADER
                + "ledger-reconcile,Office/Admin,manual,1,2026-11-27T10:00:00Z,2026-11-27T09:59:00Z,60,0,bad\n",
            ),
            (
                "mismatched_seconds",
                HEADER
                + "ledger-reconcile,Office/Admin,manual,1,2026-11-27T10:00:00Z,2026-11-27T10:05:00Z,120,0,bad\n",
            ),
            (
                "negative_errors",
                HEADER
                + "ledger-reconcile,Office/Admin,manual,1,2026-11-27T10:00:00Z,2026-11-27T10:01:00Z,60,-1,bad\n",
            ),
            (
                "missing_timezone",
                HEADER
                + "ledger-reconcile,Office/Admin,manual,1,2026-11-27T10:00:00,2026-11-27T10:01:00,60,0,bad\n",
            ),
            (
                "wrong_column_count",
                HEADER + "ledger-reconcile,Office/Admin,manual,1\n",
            ),
        ]

        with tempfile.TemporaryDirectory() as tmp:
            for name, content in bad_cases:
                with self.subTest(case=name):
                    p = self._write_temp(tmp, f"{name}.csv", content)
                    with self.assertRaises(at.ValidationError):
                        at.load_and_validate_csv(p)

    def test_cli_header_only_and_malformed_exit_codes(self) -> None:
        script = KIT_DIR / "analyze_timings.py"
        with tempfile.TemporaryDirectory() as tmp:
            empty_csv = self._write_temp(tmp, "timings.csv", HEADER)
            proc = subprocess.run(
                [sys.executable, str(script), str(empty_csv)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertIn("**`BLOCKED`**", proc.stdout)
            self.assertIn('"status": "BLOCKED"', proc.stdout)
            self.assertIn('"total_runs": 0', proc.stdout)

            # --require-complete exits 1 on BLOCKED
            proc_req = subprocess.run(
                [
                    sys.executable,
                    str(script),
                    str(empty_csv),
                    "--format",
                    "json",
                    "--require-complete",
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc_req.returncode, 1)
            payload = json.loads(proc_req.stdout)
            self.assertEqual(payload["status"], "BLOCKED")

            # Malformed CSV exits 2
            bad_csv = self._write_temp(
                tmp,
                "bad.csv",
                HEADER + "bad-task,Office/Admin,manual,1,2026-11-27T09:00:00Z,2026-11-27T09:01:00Z,60,0,x\n",
            )
            proc_bad = subprocess.run(
                [sys.executable, str(script), str(bad_csv)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(proc_bad.returncode, 2)
            self.assertIn("VALIDATION FAILED", proc_bad.stderr)

    def test_invalid_rates_json_rejected(self) -> None:
        bad_rates = [
            ("negative_rate", {"roles": {"Office/Admin": {"hourly_rate_usd": -10}}, "tasks": {}}),
            ("unknown_role", {"roles": {"Executive": {"hourly_rate_usd": 100}}, "tasks": {}}),
            ("unknown_task", {"roles": {}, "tasks": {"bogus": {"runs_per_year": 10}}}),
            ("negative_runs", {"roles": {}, "tasks": {"ledger-reconcile": {"runs_per_year": -5}}}),
        ]
        with tempfile.TemporaryDirectory() as tmp:
            for name, payload in bad_rates:
                with self.subTest(case=name):
                    rp = self._write_temp(tmp, f"{name}.json", json.dumps(payload))
                    with self.assertRaises(at.ValidationError):
                        at.load_and_validate_rates(rp)


if __name__ == "__main__":
    unittest.main()
