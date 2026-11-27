#!/usr/bin/env python3
"""Analyze real employee before-and-after stopwatch timings for Ergon.

Standard-library Python 3 only. Reads a timings.csv matching the 2026-11-26
audit schema, validates every row strictly, enforces the n >= 3 per-mode sample
size gate (refusing annual hour/dollar extrapolations when n < 3), computes
per-task and per-role medians, min-max ranges, net deltas, and error rates, and
outputs a clean Markdown report plus JSON summary.

NEVER fabricates or simulates timing data. An empty header-only timings.csv
produces a clean BLOCKED (n=0) report.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import re
import statistics
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

EXPECTED_HEADER = [
    "task",
    "role",
    "mode",
    "run",
    "start",
    "end",
    "seconds",
    "errors",
    "notes",
]

CANONICAL_TASKS = (
    "ledger-reconcile",
    "contract-risk-audit",
    "quote-sheet-build",
    "inbox-triage",
    "meeting-action-log",
)

CANONICAL_ROLES = (
    "Office/Admin",
    "Account Executive",
    "Project Manager",
    "Installer/Field Ops",
)

CANONICAL_MODES = ("manual", "ergon")
MIN_RUNS_PER_MODE = 3
TIMESTAMP_TOLERANCE_SECONDS = 2

_FORMULA_MIN_RE = re.compile(
    r"estimated_minutes_saved\s*=\s*([0-9]+(?:\.[0-9]+)?)"
)


class ValidationError(ValueError):
    """Raised when timings.csv or --rates-json fails schema/data validation."""

    def __init__(self, errors: list[str]) -> None:
        self.errors = errors
        super().__init__("\n".join(errors))


def _round2(value: float) -> float:
    return round(float(value), 2)


def _round4(value: float) -> float:
    return round(float(value), 4)


def _parse_iso8601(raw: str) -> datetime:
    text = raw.strip()
    if not text:
        raise ValueError("empty timestamp")
    # Support trailing Z across all Python 3 versions
    if text.endswith("Z") or text.endswith("z"):
        text = text[:-1] + "+00:00"
    dt = datetime.fromisoformat(text)
    if dt.tzinfo is None:
        raise ValueError(
            "timestamp must include timezone offset or 'Z' (ISO-8601 UTC/offset)"
        )
    return dt


def load_and_validate_csv(csv_path: Path) -> list[dict[str, Any]]:
    """Read and strictly validate timings.csv; returns validated row dicts."""
    if not csv_path.exists():
        raise ValidationError([f"CSV file not found: {csv_path}"])
    if not csv_path.is_file():
        raise ValidationError([f"Path is not a regular file: {csv_path}"])

    errors: list[str] = []
    rows: list[dict[str, Any]] = []
    seen_keys: dict[tuple[str, str, str, int], int] = {}

    with csv_path.open("r", encoding="utf-8", newline="") as fh:
        reader = csv.reader(fh)
        try:
            header = next(reader)
        except StopIteration:
            raise ValidationError(["CSV file is empty; expected header row."])

        # Strip optional UTF-8 BOM on first column only if present
        if header and header[0].startswith("\ufeff"):
            header[0] = header[0].lstrip("\ufeff")

        if header != EXPECTED_HEADER:
            raise ValidationError(
                [
                    f"Invalid CSV header: got {header!r}, expected {EXPECTED_HEADER!r}"
                ]
            )

        for line_no, raw_row in enumerate(reader, start=2):
            if not raw_row or all(cell.strip() == "" for cell in raw_row):
                errors.append(
                    f"Line {line_no}: blank line not allowed in timings.csv"
                )
                continue

            if len(raw_row) != len(EXPECTED_HEADER):
                errors.append(
                    f"Line {line_no}: expected {len(EXPECTED_HEADER)} columns, got {len(raw_row)}"
                )
                continue

            (
                task,
                role,
                mode,
                run_s,
                start_s,
                end_s,
                seconds_s,
                errors_s,
                notes,
            ) = [c.strip() for c in raw_row]

            row_errors: list[str] = []

            if task not in CANONICAL_TASKS:
                row_errors.append(
                    f"unknown task {task!r} (expected one of {list(CANONICAL_TASKS)})"
                )
            if role not in CANONICAL_ROLES:
                row_errors.append(
                    f"unknown role {role!r} (expected one of {list(CANONICAL_ROLES)})"
                )
            if mode not in CANONICAL_MODES:
                row_errors.append(
                    f"invalid mode {mode!r} (expected 'manual' or 'ergon')"
                )

            run_num: int | None = None
            try:
                if not re.fullmatch(r"[0-9]+", run_s):
                    raise ValueError
                run_num = int(run_s)
                if run_num < 1:
                    raise ValueError
            except ValueError:
                row_errors.append(f"invalid run {run_s!r} (must be integer >= 1)")

            start_dt: datetime | None = None
            end_dt: datetime | None = None
            try:
                start_dt = _parse_iso8601(start_s)
            except ValueError as exc:
                row_errors.append(f"invalid start timestamp {start_s!r}: {exc}")
            try:
                end_dt = _parse_iso8601(end_s)
            except ValueError as exc:
                row_errors.append(f"invalid end timestamp {end_s!r}: {exc}")

            seconds_val: int | None = None
            try:
                if not re.fullmatch(r"[0-9]+", seconds_s):
                    raise ValueError
                seconds_val = int(seconds_s)
                if seconds_val <= 0:
                    raise ValueError
            except ValueError:
                row_errors.append(
                    f"invalid seconds {seconds_s!r} (must be integer > 0)"
                )

            if start_dt is not None and end_dt is not None:
                elapsed = (end_dt - start_dt).total_seconds()
                if elapsed <= 0:
                    row_errors.append(
                        f"end ({end_s}) must be strictly after start ({start_s})"
                    )
                elif (
                    seconds_val is not None
                    and abs(elapsed - seconds_val) > TIMESTAMP_TOLERANCE_SECONDS
                ):
                    row_errors.append(
                        f"seconds ({seconds_val}) disagrees with end - start ({elapsed:.1f}s; tolerance {TIMESTAMP_TOLERANCE_SECONDS}s)"
                    )

            err_count: int | None = None
            try:
                if not re.fullmatch(r"[0-9]+", errors_s):
                    raise ValueError
                err_count = int(errors_s)
                if err_count < 0:
                    raise ValueError
            except ValueError:
                row_errors.append(
                    f"invalid errors {errors_s!r} (must be integer >= 0)"
                )

            if run_num is not None and task in CANONICAL_TASKS and role in CANONICAL_ROLES and mode in CANONICAL_MODES:
                key = (task, role, mode, run_num)
                if key in seen_keys:
                    row_errors.append(
                        f"duplicate (task, role, mode, run)={key} (first seen on line {seen_keys[key]})"
                    )
                else:
                    seen_keys[key] = line_no

            if row_errors:
                errors.append(f"Line {line_no}: " + "; ".join(row_errors))
                continue

            formula_match = _FORMULA_MIN_RE.search(notes)
            formula_est_min = (
                float(formula_match.group(1)) if formula_match else None
            )

            rows.append(
                {
                    "line_no": line_no,
                    "task": task,
                    "role": role,
                    "mode": mode,
                    "run": run_num,
                    "start": start_s,
                    "end": end_s,
                    "seconds": seconds_val,
                    "minutes": seconds_val / 60.0,  # type: ignore[operator]
                    "errors": err_count,
                    "notes": notes,
                    "formula_minutes_saved": formula_est_min,
                }
            )

    if errors:
        raise ValidationError(errors)

    return rows


def load_and_validate_rates(rates_path: Path) -> dict[str, Any]:
    """Validate optional --rates-json configuration for est. projections."""
    if not rates_path.exists() or not rates_path.is_file():
        raise ValidationError([f"Rates JSON file not found: {rates_path}"])
    try:
        data = json.loads(rates_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValidationError([f"Invalid JSON in {rates_path}: {exc}"])

    if not isinstance(data, dict):
        raise ValidationError(["Rates JSON root must be an object."])

    errors: list[str] = []
    roles_cfg = data.get("roles", {})
    tasks_cfg = data.get("tasks", {})

    if not isinstance(roles_cfg, dict):
        errors.append("rates.roles must be an object mapping role -> {hourly_rate_usd}")
        roles_cfg = {}
    if not isinstance(tasks_cfg, dict):
        errors.append("rates.tasks must be an object mapping task -> {runs_per_year, role?}")
        tasks_cfg = {}

    clean_roles: dict[str, float] = {}
    for role, cfg in roles_cfg.items():
        if role not in CANONICAL_ROLES:
            errors.append(f"rates.roles: unknown role {role!r}")
            continue
        if isinstance(cfg, (int, float)) and not isinstance(cfg, bool):
            rate = float(cfg)
        elif isinstance(cfg, dict) and isinstance(cfg.get("hourly_rate_usd"), (int, float)) and not isinstance(cfg.get("hourly_rate_usd"), bool):
            rate = float(cfg["hourly_rate_usd"])
        else:
            errors.append(f"rates.roles[{role!r}]: expected number or {{hourly_rate_usd: number}}")
            continue
        if not math.isfinite(rate) or rate < 0:
            errors.append(f"rates.roles[{role!r}]: hourly_rate_usd must be finite and >= 0")
            continue
        clean_roles[role] = rate

    clean_tasks: dict[str, dict[str, Any]] = {}
    for task, cfg in tasks_cfg.items():
        if task not in CANONICAL_TASKS:
            errors.append(f"rates.tasks: unknown task {task!r}")
            continue
        if not isinstance(cfg, dict):
            errors.append(f"rates.tasks[{task!r}]: must be an object")
            continue
        rpy = cfg.get("runs_per_year")
        if not isinstance(rpy, (int, float)) or isinstance(rpy, bool) or not math.isfinite(float(rpy)) or float(rpy) < 0:
            errors.append(f"rates.tasks[{task!r}]: runs_per_year must be finite and >= 0")
            continue
        role_ref = cfg.get("role")
        if role_ref is not None and role_ref not in CANONICAL_ROLES:
            errors.append(f"rates.tasks[{task!r}]: unknown role {role_ref!r}")
            continue
        rate_override = cfg.get("hourly_rate_usd")
        if rate_override is not None:
            if not isinstance(rate_override, (int, float)) or isinstance(rate_override, bool) or not math.isfinite(float(rate_override)) or float(rate_override) < 0:
                errors.append(f"rates.tasks[{task!r}]: hourly_rate_usd must be finite and >= 0")
                continue
            rate_override = float(rate_override)
        clean_tasks[task] = {
            "runs_per_year": float(rpy),
            "role": role_ref,
            "hourly_rate_usd": rate_override,
        }

    if errors:
        raise ValidationError(errors)

    return {"roles": clean_roles, "tasks": clean_tasks}


def _summarize_mode(mode_rows: list[dict[str, Any]]) -> dict[str, Any]:
    n = len(mode_rows)
    if n == 0:
        return {
            "n": 0,
            "meets_min_n": False,
            "median_minutes": None,
            "min_minutes": None,
            "max_minutes": None,
            "median_seconds": None,
            "min_seconds": None,
            "max_seconds": None,
            "total_errors": 0,
            "runs_with_errors": 0,
            "error_rate_per_run": None,
            "defect_free_rate": None,
            "median_formula_minutes_saved": None,
        }

    secs = [r["seconds"] for r in mode_rows]
    mins = [r["minutes"] for r in mode_rows]
    errs = [r["errors"] for r in mode_rows]
    runs_with_err = sum(1 for e in errs if e > 0)
    formula_vals = [
        r["formula_minutes_saved"]
        for r in mode_rows
        if r["formula_minutes_saved"] is not None
    ]

    return {
        "n": n,
        "meets_min_n": n >= MIN_RUNS_PER_MODE,
        "median_minutes": _round2(statistics.median(mins)),
        "min_minutes": _round2(min(mins)),
        "max_minutes": _round2(max(mins)),
        "median_seconds": _round2(statistics.median(secs)),
        "min_seconds": min(secs),
        "max_seconds": max(secs),
        "total_errors": sum(errs),
        "runs_with_errors": runs_with_err,
        "error_rate_per_run": _round4(sum(errs) / n),
        "defect_free_rate": _round4((n - runs_with_err) / n),
        "median_formula_minutes_saved": (
            _round2(statistics.median(formula_vals)) if formula_vals else None
        ),
    }


def _summarize_group(
    name: str,
    group_rows: list[dict[str, Any]],
    warnings: list[str],
    scope_label: str,
) -> dict[str, Any]:
    manual_rows = [r for r in group_rows if r["mode"] == "manual"]
    ergon_rows = [r for r in group_rows if r["mode"] == "ergon"]
    manual_stats = _summarize_mode(manual_rows)
    ergon_stats = _summarize_mode(ergon_rows)

    ready = manual_stats["meets_min_n"] and ergon_stats["meets_min_n"]
    if manual_stats["n"] == 0 and ergon_stats["n"] == 0:
        status = "UNMEASURED"
    elif not ready:
        status = "INSUFFICIENT_N"
        warnings.append(
            f"WARNING [{scope_label} {name}]: manual n={manual_stats['n']}, "
            f"ergon n={ergon_stats['n']} (requires n >= {MIN_RUNS_PER_MODE} per mode) — "
            "annual hour/dollar extrapolation REFUSED."
        )
    else:
        status = "READY"

    median_delta_minutes: float | None = None
    error_rate_delta: float | None = None
    formula_vs_measured_delta: float | None = None

    if manual_stats["n"] > 0 and ergon_stats["n"] > 0:
        raw_manual_med = statistics.median([r["minutes"] for r in manual_rows])
        raw_ergon_med = statistics.median([r["minutes"] for r in ergon_rows])
        median_delta_minutes = _round2(raw_manual_med - raw_ergon_med)
        error_rate_delta = _round4(
            ergon_stats["error_rate_per_run"] - manual_stats["error_rate_per_run"]
        )
        if ergon_stats["median_formula_minutes_saved"] is not None:
            formula_vs_measured_delta = _round2(
                ergon_stats["median_formula_minutes_saved"] - median_delta_minutes
            )

    observed_roles = sorted({r["role"] for r in group_rows})

    return {
        "name": name,
        "status": status,
        "meets_min_n": ready,
        "observed_roles": observed_roles,
        "manual": manual_stats,
        "ergon": ergon_stats,
        "median_delta_minutes": median_delta_minutes,
        "error_rate_delta": error_rate_delta,
        "formula_vs_measured_delta_minutes": formula_vs_measured_delta,
    }


def analyze(
    rows: list[dict[str, Any]],
    rates: dict[str, Any] | None = None,
    source_label: str = "timings.csv",
) -> dict[str, Any]:
    """Compute complete per-task, per-role, and optional est. projection summary."""
    warnings: list[str] = []

    by_task: dict[str, dict[str, Any]] = {}
    for task in CANONICAL_TASKS:
        task_rows = [r for r in rows if r["task"] == task]
        by_task[task] = _summarize_group(task, task_rows, warnings, "task")

    by_role: dict[str, dict[str, Any]] = {}
    for role in CANONICAL_ROLES:
        role_rows = [r for r in rows if r["role"] == role]
        by_role[role] = _summarize_group(role, role_rows, warnings, "role")

    total_runs = len(rows)
    manual_total = sum(1 for r in rows if r["mode"] == "manual")
    ergon_total = sum(1 for r in rows if r["mode"] == "ergon")
    tasks_ready = sum(1 for t in by_task.values() if t["meets_min_n"])
    roles_ready = sum(1 for r in by_role.values() if r["meets_min_n"])

    if total_runs == 0:
        overall_status = "BLOCKED"
        warnings.append(
            "BLOCKED: n=0 real employee stopwatch runs recorded. "
            f"Collect n >= {MIN_RUNS_PER_MODE} manual + n >= {MIN_RUNS_PER_MODE} ergon runs "
            "per workflow across all 4 roles before calibrating formulas or projecting savings."
        )
    elif tasks_ready == len(CANONICAL_TASKS) and roles_ready == len(CANONICAL_ROLES):
        overall_status = "READY"
    else:
        overall_status = "INCOMPLETE"
        if tasks_ready < len(CANONICAL_TASKS):
            warnings.append(
                f"INCOMPLETE: {tasks_ready}/{len(CANONICAL_TASKS)} workflows meet "
                f"n >= {MIN_RUNS_PER_MODE} per mode ({roles_ready}/{len(CANONICAL_ROLES)} roles ready)."
            )

    projections: dict[str, Any] | None = None
    if rates is not None:
        task_projections: dict[str, Any] = {}
        refused_tasks: list[str] = []
        est_total_hours_yr = 0.0
        est_total_usd_yr = 0.0
        all_configured_ready = bool(rates["tasks"])

        for task, tcfg in rates["tasks"].items():
            tstats = by_task[task]
            if not tstats["meets_min_n"]:
                all_configured_ready = False
                refused_tasks.append(task)
                task_projections[task] = {
                    "status": "REFUSED_INSUFFICIENT_N",
                    "reason": (
                        f"Requires n >= {MIN_RUNS_PER_MODE} manual and n >= {MIN_RUNS_PER_MODE} ergon "
                        f"(observed manual={tstats['manual']['n']}, ergon={tstats['ergon']['n']})."
                    ),
                    "est_hours_saved_per_year": None,
                    "est_usd_saved_per_year": None,
                    "label": "REFUSED (n < 3)",
                }
                continue

            role_name = tcfg["role"] or (
                tstats["observed_roles"][0]
                if len(tstats["observed_roles"]) == 1
                else None
            )
            hourly_rate = tcfg["hourly_rate_usd"]
            if hourly_rate is None and role_name is not None:
                hourly_rate = rates["roles"].get(role_name)
            if hourly_rate is None:
                hourly_rate = 0.0

            delta_min = tstats["median_delta_minutes"]
            assert delta_min is not None
            est_hours_yr = _round2((delta_min / 60.0) * tcfg["runs_per_year"])
            est_usd_yr = _round2(est_hours_yr * hourly_rate)
            est_total_hours_yr += est_hours_yr
            est_total_usd_yr += est_usd_yr

            task_projections[task] = {
                "status": "ESTIMATED",
                "label": "est.",
                "role": role_name,
                "runs_per_year_est": tcfg["runs_per_year"],
                "hourly_rate_usd_est": _round2(hourly_rate),
                "median_delta_minutes": delta_min,
                "est_hours_saved_per_year": est_hours_yr,
                "est_usd_saved_per_year": est_usd_yr,
            }

        if refused_tasks:
            warnings.append(
                "EXTRAPOLATION REFUSED for task(s) with n < 3 per mode: "
                + ", ".join(refused_tasks)
            )

        projections = {
            "label": "est.",
            "extrapolation_allowed_for_all_configured_tasks": all_configured_ready,
            "refused_tasks": refused_tasks,
            "by_task": task_projections,
            "totals_est": (
                {
                    "label": "est.",
                    "est_hours_saved_per_year": _round2(est_total_hours_yr),
                    "est_usd_saved_per_year": _round2(est_total_usd_yr),
                }
                if all_configured_ready
                else None
            ),
        }

    return {
        "source": source_label,
        "status": overall_status,
        "min_runs_per_mode": MIN_RUNS_PER_MODE,
        "total_runs": total_runs,
        "manual_runs": manual_total,
        "ergon_runs": ergon_total,
        "tasks_ready": tasks_ready,
        "tasks_total": len(CANONICAL_TASKS),
        "roles_ready": roles_ready,
        "roles_total": len(CANONICAL_ROLES),
        "warnings": warnings,
        "by_task": by_task,
        "by_role": by_role,
        "projections_est": projections,
    }


def _fmt_mode_cell(stats: dict[str, Any]) -> str:
    if stats["n"] == 0:
        return "UNMEASURED (n=0)"
    flag = "" if stats["meets_min_n"] else " [n<3]"
    return (
        f"{stats['median_minutes']:.2f} min "
        f"({stats['min_minutes']:.2f}–{stats['max_minutes']:.2f}, n={stats['n']}{flag})"
    )


def _fmt_error_cell(manual: dict[str, Any], ergon: dict[str, Any]) -> str:
    if manual["n"] == 0 and ergon["n"] == 0:
        return "UNMEASURED (n=0)"
    m_str = (
        f"{manual['error_rate_per_run']:.2f}/run ({manual['total_errors']} err)"
        if manual["n"] > 0
        else "n=0"
    )
    e_str = (
        f"{ergon['error_rate_per_run']:.2f}/run ({ergon['total_errors']} err)"
        if ergon["n"] > 0
        else "n=0"
    )
    return f"Manual {m_str} vs Ergon {e_str}"


def _fmt_delta_cell(group: dict[str, Any]) -> str:
    if group["median_delta_minutes"] is None:
        return "BLOCKED (n=0)" if group["status"] == "UNMEASURED" else "INCOMPLETE"
    sign = "+" if group["median_delta_minutes"] >= 0 else ""
    warn = "" if group["meets_min_n"] else " (n<3 warning)"
    return f"{sign}{group['median_delta_minutes']:.2f} min{warn}"


def render_markdown(report: dict[str, Any], include_json_block: bool = True) -> str:
    """Format analysis output as clean GitHub-Flavored Markdown."""
    lines: list[str] = [
        "# Ergon Timing Study Analysis Report",
        "",
        f"- **Source CSV:** `{report['source']}`",
        f"- **Study Status:** **`{report['status']}`**",
        f"- **Total Timed Runs:** `{report['total_runs']}` (`manual={report['manual_runs']}`, `ergon={report['ergon_runs']}`)",
        f"- **Gate Coverage (`n >= {report['min_runs_per_mode']}` per mode):** "
        f"`{report['tasks_ready']}/{report['tasks_total']}` workflows ready, "
        f"`{report['roles_ready']}/{report['roles_total']}` roles ready",
        "",
    ]

    if report["warnings"]:
        lines.append("## Warnings & Guardrails")
        lines.append("")
        for w in report["warnings"]:
            lines.append(f"- {w}")
        lines.append("")

    lines.extend(
        [
            "## Per-Workflow Summary (Median Minutes, Min–Max Range, Net Delta, Error Rate)",
            "",
            "| Task | Status | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (Manual vs Ergon) |",
            "|---|---|---|---|---|---|",
        ]
    )
    for task in CANONICAL_TASKS:
        g = report["by_task"][task]
        lines.append(
            f"| `{task}` | `{g['status']}` | {_fmt_mode_cell(g['manual'])} | "
            f"{_fmt_mode_cell(g['ergon'])} | {_fmt_delta_cell(g)} | "
            f"{_fmt_error_cell(g['manual'], g['ergon'])} |"
        )
    lines.append("")

    lines.extend(
        [
            "## Per-Role Summary",
            "",
            "| Role | Status | Manual Median (Range) | Ergon Median (Range) | Net Saved (`Manual - Ergon`) | Error Rate (Manual vs Ergon) |",
            "|---|---|---|---|---|---|",
        ]
    )
    for role in CANONICAL_ROLES:
        g = report["by_role"][role]
        lines.append(
            f"| {role} | `{g['status']}` | {_fmt_mode_cell(g['manual'])} | "
            f"{_fmt_mode_cell(g['ergon'])} | {_fmt_delta_cell(g)} | "
            f"{_fmt_error_cell(g['manual'], g['ergon'])} |"
        )
    lines.append("")

    proj = report["projections_est"]
    if proj is not None:
        lines.extend(
            [
                "## Annualized Projections (`est.` — Strictly Refused When `n < 3` Per Mode)",
                "",
                "| Task | Status | Runs/Yr (`est.`) | Hourly Rate (`est.`) | Median Delta | Annual Hours Saved (`est.`) | Annual Value (`est.`) |",
                "|---|---|---:|---:|---:|---:|---:|",
            ]
        )
        for task, p in proj["by_task"].items():
            if p["status"] != "ESTIMATED":
                lines.append(
                    f"| `{task}` | `REFUSED (n < 3)` | — | — | — | REFUSED | REFUSED |"
                )
            else:
                lines.append(
                    f"| `{task}` | `est.` | {p['runs_per_year_est']:.0f} (`est.`) | "
                    f"${p['hourly_rate_usd_est']:.2f}/hr (`est.`) | "
                    f"{p['median_delta_minutes']:+.2f} min | "
                    f"{p['est_hours_saved_per_year']:+.2f} hr/yr (`est.`) | "
                    f"${p['est_usd_saved_per_year']:,.2f}/yr (`est.`) |"
                )
        lines.append("")
        if proj["totals_est"] is not None:
            tot = proj["totals_est"]
            lines.append(
                f"**Configured Portfolio Total (`est.`):** "
                f"`{tot['est_hours_saved_per_year']:+.2f} hr/yr (est.)` / "
                f"`${tot['est_usd_saved_per_year']:,.2f}/yr (est.)` "
                "(hourly rates and annual run frequencies are estimates)."
            )
        else:
            lines.append(
                "**Portfolio Rollup:** `REFUSED` — all configured tasks must reach "
                f"`n >= {report['min_runs_per_mode']}` manual and `n >= {report['min_runs_per_mode']}` "
                "ergon runs before extrapolating annual totals."
            )
        lines.append("")

    if include_json_block:
        lines.extend(
            [
                "## JSON Summary",
                "",
                "```json",
                json.dumps(report, indent=2),
                "```",
                "",
            ]
        )

    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Validate and analyze Ergon before-and-after employee stopwatch timings "
            "(standard-library Python 3; refuses extrapolation when n < 3 per mode)."
        )
    )
    parser.add_argument("csv_path", type=Path, help="Path to timings.csv")
    parser.add_argument(
        "--rates-json",
        type=Path,
        default=None,
        help="Optional JSON file with role hourly rates and task runs/year (all projections tagged est.).",
    )
    parser.add_argument(
        "--format",
        choices=("both", "markdown", "json"),
        default="both",
        help="Stdout format (default: both = Markdown report + JSON summary block).",
    )
    parser.add_argument(
        "--md-out",
        type=Path,
        default=None,
        help="Optional path to write the Markdown report.",
    )
    parser.add_argument(
        "--json-out",
        type=Path,
        default=None,
        help="Optional path to write the pure JSON summary.",
    )
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="Exit non-zero (1) unless all 5 workflows and 4 roles meet n >= 3 per mode.",
    )
    args = parser.parse_args(argv)

    try:
        rows = load_and_validate_csv(args.csv_path)
        rates = (
            load_and_validate_rates(args.rates_json)
            if args.rates_json is not None
            else None
        )
    except ValidationError as exc:
        print("VALIDATION FAILED:", file=sys.stderr)
        for err in exc.errors:
            print(f"  - {err}", file=sys.stderr)
        return 2

    report = analyze(rows, rates=rates, source_label=args.csv_path.name)
    md_text = render_markdown(
        report, include_json_block=(args.format == "both")
    )
    json_text = json.dumps(report, indent=2) + "\n"

    if args.md_out is not None:
        args.md_out.write_text(
            render_markdown(report, include_json_block=False) + "\n",
            encoding="utf-8",
        )
    if args.json_out is not None:
        args.json_out.write_text(json_text, encoding="utf-8")

    if args.format == "json":
        sys.stdout.write(json_text)
    else:
        sys.stdout.write(md_text)

    if args.require_complete and report["status"] != "READY":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
