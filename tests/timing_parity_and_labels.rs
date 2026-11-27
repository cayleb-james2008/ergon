//! Gap 9 (MVP Row 10) Verification Suite:
//! 1. Exact Rust (`src/records.rs`) <-> Browser (`frontend/src/lib/meteringFormula.ts` & `webEngine.ts`)
//!    parity on `compute_minutes_saved`, `compute_billable_value_usd`, and `$0/hr` unassigned fallback.
//! 2. Rebuilt embedded UI `assets/` clearly label all rendered time-saved & billable-value metrics
//!    as estimates (`est.`) with `"Formula estimate — not yet calibrated to stopwatch timings"`.
//! 3. Timing measurement kit under `evidence/timing/kit/` matches the 5 canonical workflows,
//!    4 canonical roles, exact CSV schema, passes its Python unittest suite, and produces clean
//!    `BLOCKED (n=0)` output on a header-only `timings.csv`.

use ergon_rs::records::{compute_billable_value_usd, compute_minutes_saved};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_dir(label: &str) -> PathBuf {
    let seq = TMP_SEQ.fetch_add(1, Ordering::SeqCst);
    let base = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target"));
    let dir = base
        .join("tmp-timing-parity")
        .join(format!("{label}-{}-{seq}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn rust_and_browser_metering_formula_exact_parity() {
    // Baseline sanity assertions verifying src/records.rs minutes formula is untouched.
    assert_eq!(compute_minutes_saved(false, false, 0, 0, 500), 0.0);
    assert_eq!(compute_minutes_saved(false, false, 2, 1, 500), 8.0);
    assert_eq!(compute_minutes_saved(true, true, 0, 0, 0), 2.0);
    assert_eq!(compute_minutes_saved(true, true, 0, 0, 10_000), 12.0);
    assert_eq!(compute_minutes_saved(true, false, 1, 0, 0), 7.0);
    assert_eq!(compute_minutes_saved(true, false, 1, 1, 0), 15.0);
    assert_eq!(compute_minutes_saved(true, false, 1, 2, 0), 23.0);
    assert_eq!(compute_billable_value_usd(23.0, 0.0), 0.0);
    assert_eq!(compute_billable_value_usd(23.0, -50.0), 0.0);
    assert_eq!(compute_billable_value_usd(f64::NAN, 150.0), 0.0);

    // Shared cross-language test vectors fed to both Rust and Node (meteringFormula.ts).
    let vectors = vec![
        (false, false, 0usize, 0usize, 0u64, None, None, 0.0f64),
        (false, false, 2, 1, 900, None, None, 0.0),
        (true, true, 0, 0, 0, None, None, 0.0),
        (true, true, 0, 0, 625, None, Some(185.0), 185.0),
        (true, true, 0, 0, 9999, Some(220.0), Some(185.0), 220.0),
        (true, false, 1, 0, 0, None, None, 0.0),
        (true, false, 1, 1, 0, Some(0.0), Some(140.0), 140.0),
        (true, false, 1, 2, 0, Some(0.0), Some(0.0), 0.0),
        (true, false, 3, 1, 480, Some(220.0), Some(185.0), 220.0),
        (true, false, 2, 2, 3750, Some(175.5), None, 175.5),
        (true, false, 4, 3, 8000, Some(95.25), Some(120.0), 95.25),
    ];

    let payload: Vec<Value> = vectors
        .iter()
        .map(
            |&(finished, answered, ok_steps, files_written, completion_tokens, eng_rate, cli_rate, _)| {
                json!({
                    "finished": finished,
                    "answered": answered,
                    "ok_steps": ok_steps,
                    "files_written": files_written,
                    "completion_tokens": completion_tokens,
                    "engagement": eng_rate.map(|r| json!({ "hourly_rate_usd": r })),
                    "client": cli_rate.map(|r| json!({ "default_rate_usd": r })),
                })
            },
        )
        .collect();

    let script = repo_root()
        .join("frontend")
        .join("scripts")
        .join("test-metering-parity.mjs");
    assert!(script.is_file(), "missing {}", script.display());

    // First run the standalone Node assertion script.
    let direct = Command::new("node")
        .arg(&script)
        .output()
        .expect("run node test-metering-parity.mjs");
    assert!(
        direct.status.success(),
        "node parity assertions failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&direct.stdout),
        String::from_utf8_lossy(&direct.stderr)
    );

    // Then cross-check every vector between Rust and Node side-by-side.
    let mut child = Command::new("node")
        .arg(&script)
        .arg("--eval-vectors")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn node --eval-vectors");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&payload).unwrap().as_bytes())
        .expect("write vectors");
    let out = child.wait_with_output().expect("wait node");
    assert!(
        out.status.success(),
        "node --eval-vectors failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let js_results: Vec<Value> =
        serde_json::from_slice(&out.stdout).expect("parse node JSON results");
    assert_eq!(js_results.len(), vectors.len());

    for (idx, (&(finished, answered, ok_steps, files_written, completion_tokens, _, _, expected_rate), js)) in
        vectors.iter().zip(js_results.iter()).enumerate()
    {
        let rust_mins = compute_minutes_saved(
            finished,
            answered,
            ok_steps,
            files_written,
            completion_tokens,
        );
        let rust_bill = compute_billable_value_usd(rust_mins, expected_rate);

        let js_mins = js["minutes_saved"].as_f64().expect("js minutes_saved");
        let js_rate = js["effective_hourly_rate_usd"]
            .as_f64()
            .expect("js effective_hourly_rate_usd");
        let js_bill = js["billable_value_usd"]
            .as_f64()
            .expect("js billable_value_usd");

        assert!(
            (rust_mins - js_mins).abs() < 1e-9,
            "vector {idx}: minutes mismatch rust={rust_mins} js={js_mins}"
        );
        assert!(
            (expected_rate - js_rate).abs() < 1e-9,
            "vector {idx}: hourly rate mismatch expected={expected_rate} js={js_rate}"
        );
        assert!(
            (rust_bill - js_bill).abs() < 1e-9,
            "vector {idx}: billable value mismatch rust={rust_bill} js={js_bill}"
        );
    }
}

#[test]
fn embedded_assets_contain_estimate_labels_and_uncalibrated_tooltip() {
    let assets_dir = repo_root().join("assets");
    let index_html = fs::read_to_string(assets_dir.join("index.html"))
        .expect("read assets/index.html");
    assert!(!index_html.is_empty(), "assets/index.html must be non-empty");

    let chunks_dir = assets_dir.join("chunks");
    let mut paper_bundle = String::new();
    let mut settings_bundle = String::new();
    for entry in fs::read_dir(&chunks_dir).expect("read assets/chunks") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("js") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let content = fs::read_to_string(&path).expect("read chunk");
        if name.starts_with("Paper-") {
            paper_bundle.push_str(&content);
        } else if name.starts_with("SettingsPanel-") {
            settings_bundle.push_str(&content);
        }
    }

    assert!(
        !paper_bundle.is_empty(),
        "expected compiled Paper-*.js chunk under assets/chunks/"
    );
    assert!(
        paper_bundle.contains("Est. minutes saved"),
        "embedded Paper chunk must label minutes saved as an estimate"
    );
    assert!(
        paper_bundle.contains("Est. billable value"),
        "embedded Paper chunk must label billable value as an estimate"
    );
    assert!(
        paper_bundle.contains("Formula estimate — not yet calibrated to stopwatch timings"),
        "embedded Paper chunk must include honest uncalibrated formula subtitle/tooltip"
    );
    assert!(
        paper_bundle.contains(" (est.)"),
        "embedded Paper chunk must tag rendered values with (est.)"
    );
    assert!(
        paper_bundle.contains("No rate set ($0/hr)"),
        "embedded Paper chunk must surface $0/hr unassigned rate honestly"
    );
    assert!(
        paper_bundle.contains("saved (est.)"),
        "embedded Paper chunk must tag per-receipt saved minutes with (est.)"
    );
    assert!(
        !paper_bundle.contains("Estimated work hours"),
        "old unlabeled 'Estimated work hours' subtitle should be replaced"
    );
    assert!(
        !settings_bundle.is_empty() && settings_bundle.contains("estimated billable value"),
        "embedded SettingsPanel chunk must label billable value metering as estimated"
    );

    // Ensure webEngine.ts has no $150/hr fallback left.
    let web_engine = fs::read_to_string(
        repo_root()
            .join("frontend")
            .join("src")
            .join("lib")
            .join("webEngine.ts"),
    )
    .expect("read webEngine.ts");
    assert!(
        !web_engine.contains("function estimateMinutesSaved("),
        "legacy regex estimateMinutesSaved must be removed from webEngine.ts"
    );
    assert!(
        web_engine.contains("resolveEffectiveHourlyRate"),
        "webEngine.ts must use shared resolveEffectiveHourlyRate ($0/hr fallback)"
    );
}

#[test]
fn timing_measurement_kit_files_and_analyzer_n0_blocked() {
    let kit_dir = repo_root().join("evidence").join("timing").join("kit");
    let sheets_path = kit_dir.join("STOPWATCH-SHEETS.md");
    let checklist_path = kit_dir.join("RUN-CHECKLIST.md");
    let analyzer_path = kit_dir.join("analyze_timings.py");
    let test_path = kit_dir.join("test_analyze_timings.py");

    for p in [&sheets_path, &checklist_path, &analyzer_path, &test_path] {
        assert!(p.is_file(), "missing required kit file {}", p.display());
    }

    let sheets = fs::read_to_string(&sheets_path).expect("read STOPWATCH-SHEETS.md");
    let checklist = fs::read_to_string(&checklist_path).expect("read RUN-CHECKLIST.md");
    let analyzer = fs::read_to_string(&analyzer_path).expect("read analyze_timings.py");

    let workflows = [
        "ledger-reconcile",
        "contract-risk-audit",
        "quote-sheet-build",
        "inbox-triage",
        "meeting-action-log",
    ];
    let roles = [
        "Office/Admin",
        "Account Executive",
        "Project Manager",
        "Installer/Field Ops",
    ];
    let csv_header = "task,role,mode,run,start,end,seconds,errors,notes";

    for wf in workflows {
        assert!(sheets.contains(wf), "STOPWATCH-SHEETS.md missing {wf}");
        assert!(checklist.contains(wf), "RUN-CHECKLIST.md missing {wf}");
        assert!(analyzer.contains(wf), "analyze_timings.py missing {wf}");
    }
    for role in roles {
        assert!(sheets.contains(role), "STOPWATCH-SHEETS.md missing {role}");
        assert!(checklist.contains(role), "RUN-CHECKLIST.md missing {role}");
        assert!(analyzer.contains(role), "analyze_timings.py missing {role}");
    }
    assert!(sheets.contains(csv_header), "STOPWATCH-SHEETS.md missing exact CSV header");
    assert!(checklist.contains(csv_header), "RUN-CHECKLIST.md missing exact CSV header");

    // Run Python unittest suite for analyze_timings.py (-B avoids writing __pycache__)
    let py_test = Command::new("python3")
        .args(["-B", "-m", "unittest"])
        .arg(&test_path)
        .current_dir(repo_root())
        .output()
        .expect("run python3 -m unittest on test_analyze_timings.py");
    assert!(
        py_test.status.success(),
        "python unittest failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&py_test.stdout),
        String::from_utf8_lossy(&py_test.stderr)
    );

    // Run analyze_timings.py against a fresh header-only timings.csv -> prove n=0 / BLOCKED
    let tmp = temp_dir("n0-blocked");
    let header_csv = tmp.join("timings.csv");
    fs::write(&header_csv, format!("{csv_header}\n")).expect("write header-only timings.csv");
    let py_run = Command::new("python3")
        .arg("-B")
        .arg(&analyzer_path)
        .arg(&header_csv)
        .args(["--format", "json"])
        .output()
        .expect("run analyze_timings.py on header-only CSV");
    assert!(
        py_run.status.success(),
        "analyze_timings.py failed on header-only CSV: {}",
        String::from_utf8_lossy(&py_run.stderr)
    );
    let report: Value =
        serde_json::from_slice(&py_run.stdout).expect("parse analyzer JSON output");
    assert_eq!(report["status"], "BLOCKED");
    assert_eq!(report["total_runs"], 0);
    assert_eq!(report["tasks_ready"], 0);
    assert_eq!(report["roles_ready"], 0);

    let _ = fs::remove_dir_all(&tmp);

    // Confirm no stray host path leaks in kit files.
    for p in [&sheets_path, &checklist_path, &analyzer_path, &test_path] {
        let body = fs::read_to_string(p).unwrap_or_default();
        assert!(
            !body.contains("/home/") && !body.contains("/Users/") && !body.contains("C:\\Users\\"),
            "host path leaked in {}",
            Path::new(p).display()
        );
    }
}
