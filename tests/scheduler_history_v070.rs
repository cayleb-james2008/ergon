//! Comprehensive unit, integration, and security tests for Ergon v0.7.0:
//! - `file_history`: `.history/` snapshots, line-by-line diffs, atomic restore,
//!   SHA-256 blob tamper detection, retention cap, and traversal fuzzing.
//! - `scheduler`: cadence parser, UTC calendar `next_run_ms` math (incl. leap year
//!   & weekly rollovers), pause/resume, due detection, and atomic `RoutineRunRecord`
//!   history persistence.
//! - `audit_bundle`: tamper-evident Field-Book Audit Bundle JSON/Markdown export,
//!   offline chain + manifest verification, workspace cross-check, and tamper tests.

use ergon_rs::{audit_bundle, file_history, records, routines, scheduler, Config, Person};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    root: PathBuf,
    cfg: Config,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-v070-{}-{}-{name}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        TestEnv { root, cfg }
    }

    fn write_ws(&self, rel: &str, content: &[u8]) {
        let full = self.cfg.workspace_dir.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }

    fn read_ws_str(&self, rel: &str) -> String {
        std::fs::read_to_string(self.cfg.workspace_dir.join(rel)).unwrap()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// ---------------------------------------------------------------------------
// 1. file_history tests
// ---------------------------------------------------------------------------

#[test]
fn file_history_record_list_diff_and_restore_roundtrip() {
    let env = TestEnv::new("fh-roundtrip");

    // First-time creation: no existing file yet -> Ok(None).
    let none_rev = file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        "overwrite",
    )
    .unwrap();
    assert!(none_rev.is_none());

    // Write v1 -> snapshot before overwriting with v2.
    env.write_ws(
        "reports/budget.md",
        b"# Q4 Budget\nTravel: $1,200\nSoftware: $800\nTotal: $2,000\n",
    );
    let rev1 = file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        "overwrite",
    )
    .unwrap()
    .expect("v1 snapshot recorded");
    assert_eq!(rev1.reason, "overwrite");
    assert_eq!(rev1.line_count, 4);
    assert!(rev1.is_text);

    // Overwrite with v2 -> snapshot before deleting.
    env.write_ws(
        "reports/budget.md",
        b"# Q4 Budget\nTravel: $1,500\nSoftware: $800\nHardware: $400\nTotal: $2,700\n",
    );
    let rev2 = file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        "delete",
    )
    .unwrap()
    .expect("v2 snapshot recorded");
    assert_eq!(rev2.reason, "delete");
    assert_ne!(rev1.sha256, rev2.sha256);

    // Diff rev1 vs rev2.
    let diff = file_history::diff_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        &rev1.rev_id,
        &rev2.rev_id,
    )
    .unwrap();
    assert!(!diff.is_binary);
    assert_eq!(diff.unchanged, 2); // "# Q4 Budget" and "Software: $800"
    assert_eq!(diff.removed, 2); // Travel $1,200 and Total $2,000
    assert_eq!(diff.added, 3); // Travel $1,500, Hardware $400, Total $2,700
    assert!(!diff.hunks.is_empty());
    assert!(diff.unified.contains("-Travel: $1,200"), "{}", diff.unified);
    assert!(diff.unified.contains("+Hardware: $400"), "{}", diff.unified);

    // Diff rev1 vs live "current" file.
    let diff_cur = file_history::diff_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        &rev1.rev_id,
        "current",
    )
    .unwrap();
    assert_eq!(diff_cur.added, 3);
    assert_eq!(diff_cur.removed, 2);

    // Restore rev1 over live v2 -> creates pre-restore snapshot and restores v1 bytes atomically.
    let restored = file_history::restore_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        &rev1.rev_id,
    )
    .unwrap();
    assert_eq!(restored.restored_rev_id, rev1.rev_id);
    assert!(restored.pre_restore_rev_id.is_some());
    assert_eq!(restored.sha256, rev1.sha256);
    assert_eq!(
        env.read_ws_str("reports/budget.md"),
        "# Q4 Budget\nTravel: $1,200\nSoftware: $800\nTotal: $2,000\n"
    );

    // List revisions newest-first: pre-restore, delete (rev2), overwrite (rev1).
    let list = file_history::list_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
    )
    .unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].reason, "pre-restore");
    assert_eq!(list[1].rev_id, rev2.rev_id);
    assert_eq!(list[2].rev_id, rev1.rev_id);

    // Delete live file completely, then restore rev2 back from .history/!
    std::fs::remove_file(env.cfg.workspace_dir.join("reports/budget.md")).unwrap();
    let payload_deleted = file_history::history_payload_json(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
    )
    .unwrap();
    assert_eq!(payload_deleted["exists"], json!(false));
    assert_eq!(payload_deleted["count"], json!(3));

    let resurrected = file_history::restore_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "reports/budget.md",
        &rev2.rev_id,
    )
    .unwrap();
    assert!(resurrected.pre_restore_rev_id.is_none());
    assert!(env.read_ws_str("reports/budget.md").contains("Hardware: $400"));
}

#[test]
fn file_history_binary_large_diff_tamper_and_retention_cap() {
    let env = TestEnv::new("fh-edge");

    // Binary diff handles non-UTF-8 / NUL cleanly without panicking.
    let bin1 = file_history::record_revision_bytes(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "img.bin",
        &[0x00, 0xff, 0x10, 0x20],
        "overwrite",
    )
    .unwrap();
    assert!(!bin1.is_text);
    let bin2 = file_history::record_revision_bytes(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "img.bin",
        &[0x00, 0xff, 0x99, 0x20, 0x30],
        "overwrite",
    )
    .unwrap();
    let bdiff = file_history::diff_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "img.bin",
        &bin1.rev_id,
        &bin2.rev_id,
    )
    .unwrap();
    assert!(bdiff.is_binary);
    assert_eq!(bdiff.similarity_pct, 0);

    // Large file diff (>600 lines exercises linear-time hash-anchor fallback).
    let mut big_a = String::new();
    let mut big_b = String::new();
    for i in 1..=600 {
        big_a.push_str(&format!("Row {i}: baseline value {i}\n"));
        if i == 150 || i == 450 {
            big_b.push_str(&format!("Row {i}: UPDATED value {i}\n"));
        } else {
            big_b.push_str(&format!("Row {i}: baseline value {i}\n"));
        }
    }
    let ra = file_history::record_revision_bytes(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "big.txt",
        big_a.as_bytes(),
        "overwrite",
    )
    .unwrap();
    let rb = file_history::record_revision_bytes(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "big.txt",
        big_b.as_bytes(),
        "overwrite",
    )
    .unwrap();
    let big_diff = file_history::diff_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "big.txt",
        &ra.rev_id,
        &rb.rev_id,
    )
    .unwrap();
    assert_eq!(big_diff.unchanged, 598);
    assert_eq!(big_diff.added, 2);
    assert_eq!(big_diff.removed, 2);

    // Tampering with a stored blob on disk is caught by SHA-256 check on read & restore.
    let history_root = file_history::history_dir(&env.cfg.data_dir);
    let mut corrupted_blob = false;
    for entry in std::fs::read_dir(&history_root).unwrap().flatten() {
        let candidate = entry.path().join("blobs").join(format!("{}.blob", ra.rev_id));
        if candidate.is_file() {
            std::fs::write(&candidate, b"TAMPERED BYTES").unwrap();
            corrupted_blob = true;
            break;
        }
    }
    assert!(corrupted_blob, "expected to locate stored blob for tamper test");
    let err = file_history::restore_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "big.txt",
        &ra.rev_id,
    )
    .unwrap_err();
    assert!(err.contains("integrity"), "{err}");

    // Retention cap: writing MAX_REVISIONS_PER_FILE + 6 revisions keeps exactly MAX_REVISIONS_PER_FILE.
    for i in 0..(file_history::MAX_REVISIONS_PER_FILE + 6) {
        file_history::record_revision_bytes(
            &env.cfg.data_dir,
            &env.cfg.workspace_dir,
            "capped.txt",
            format!("version {i}").as_bytes(),
            "overwrite",
        )
        .unwrap();
    }
    let capped = file_history::list_revisions(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "capped.txt",
    )
    .unwrap();
    assert_eq!(capped.len(), file_history::MAX_REVISIONS_PER_FILE);
}

#[test]
fn file_history_blocks_traversal_symlinks_and_hostile_rev_ids() {
    let env = TestEnv::new("fh-security");
    env.write_ws("ok.txt", b"safe content\n");
    let valid_rev = file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "ok.txt",
        "overwrite",
    )
    .unwrap()
    .unwrap();

    // Hostile workspace path vectors
    let bad_paths = [
        "",
        "   ",
        "../secret.txt",
        "../../etc/passwd",
        "/etc/passwd",
        "sub/../../escape.txt",
        ".history/manifest.json",
        "a/.history/b",
        "foo\\..\\bar",
        "file\0name.txt",
        "bad\npath.txt",
        ".",
        "..",
    ];
    for p in bad_paths {
        assert!(
            file_history::record_revision(&env.cfg.data_dir, &env.cfg.workspace_dir, p, "overwrite")
                .is_err(),
            "record_revision accepted hostile path: {p:?}"
        );
        assert!(
            file_history::list_revisions(&env.cfg.data_dir, &env.cfg.workspace_dir, p).is_err(),
            "list_revisions accepted hostile path: {p:?}"
        );
        assert!(
            file_history::restore_revision(
                &env.cfg.data_dir,
                &env.cfg.workspace_dir,
                p,
                &valid_rev.rev_id
            )
            .is_err(),
            "restore_revision accepted hostile path: {p:?}"
        );
    }

    // Hostile rev_id vectors
    let bad_revs = [
        "",
        "..",
        ".",
        "../manifest",
        "rev/../../etc/passwd",
        "rev\\..\\evil",
        "rev\0id",
        "rev id",
        "%2e%2e",
        "current", // disallowed as restore target
        &"a".repeat(120),
    ];
    for r in bad_revs {
        assert!(
            file_history::restore_revision(&env.cfg.data_dir, &env.cfg.workspace_dir, "ok.txt", r)
                .is_err(),
            "restore_revision accepted hostile rev_id: {r:?}"
        );
    }

    // Symlink escape refusal
    #[cfg(unix)]
    {
        let outside = env.root.join("outside.txt");
        std::fs::write(&outside, b"top secret").unwrap();
        std::os::unix::fs::symlink(&outside, env.cfg.workspace_dir.join("escape.txt")).unwrap();
        assert!(
            file_history::record_revision(
                &env.cfg.data_dir,
                &env.cfg.workspace_dir,
                "escape.txt",
                "overwrite"
            )
            .is_err(),
            "symlink escaping workspace must be rejected"
        );
        assert!(
            file_history::restore_revision(
                &env.cfg.data_dir,
                &env.cfg.workspace_dir,
                "escape.txt",
                &valid_rev.rev_id
            )
            .is_err(),
            "restoring onto a symlink must be rejected"
        );
        assert_eq!(std::fs::read(&outside).unwrap(), b"top secret");
    }
}

// ---------------------------------------------------------------------------
// 2. scheduler tests
// ---------------------------------------------------------------------------

#[test]
fn scheduler_parses_cadences_and_computes_exact_utc_next_run_ms() {
    use scheduler::Cadence;

    assert_eq!(scheduler::parse_cadence("manual").unwrap(), Cadence::Manual);
    assert_eq!(scheduler::parse_cadence("").unwrap(), Cadence::Manual);
    assert_eq!(
        scheduler::parse_cadence("every 15m").unwrap(),
        Cadence::IntervalSecs(900)
    );
    assert_eq!(
        scheduler::parse_cadence("EVERY 30 SECONDS").unwrap(),
        Cadence::IntervalSecs(30)
    );
    assert_eq!(
        scheduler::parse_cadence("every 2h").unwrap(),
        Cadence::IntervalSecs(7200)
    );
    assert_eq!(
        scheduler::parse_cadence("every 1d").unwrap(),
        Cadence::IntervalSecs(86_400)
    );
    assert_eq!(
        scheduler::parse_cadence("hourly").unwrap(),
        Cadence::Hourly { minute: 0 }
    );
    assert_eq!(
        scheduler::parse_cadence("hourly :45").unwrap(),
        Cadence::Hourly { minute: 45 }
    );
    assert_eq!(
        scheduler::parse_cadence("daily 09:00").unwrap(),
        Cadence::Daily { hour: 9, minute: 0 }
    );
    assert_eq!(
        scheduler::parse_cadence("weekly mon").unwrap(),
        Cadence::Weekly {
            weekday: 0,
            hour: 9,
            minute: 0
        }
    );
    assert_eq!(
        scheduler::parse_cadence("weekly friday 17:30").unwrap(),
        Cadence::Weekly {
            weekday: 4,
            hour: 17,
            minute: 30
        }
    );
    assert_eq!(
        scheduler::parse_cadence_or_interval(None, Some(45)).unwrap(),
        Cadence::IntervalSecs(45)
    );

    // Invalid / abusive cadences rejected cleanly.
    for bad in [
        "every 0s",
        "every -5m",
        "every 9999d",
        "daily 25:00",
        "daily 09:60",
        "hourly :65",
        "weekly funday",
        "cron * * * * *",
    ] {
        assert!(
            scheduler::parse_cadence(bad).is_err(),
            "expected error for invalid cadence: {bad}"
        );
    }
    assert!(scheduler::parse_cadence_or_interval(None, Some(0)).is_err());

    // Exact calendar math:
    // 2028-02-28T10:00:00Z (2028 is a leap year! Next day MUST be 2028-02-29T09:00:00Z).
    // 2028-02-28 is a Monday; Unix timestamp for 2028-02-28T00:00:00Z = 1_835_308_800s.
    let feb28_1000_ms: u64 = (1_835_308_800 + 10 * 3_600) * 1_000;
    assert_eq!(records::civil_from_millis(feb28_1000_ms), (2028, 2, 28));

    let next_daily = scheduler::next_run_ms(
        &Cadence::Daily { hour: 9, minute: 0 },
        Some(feb28_1000_ms),
        feb28_1000_ms,
    )
    .unwrap();
    assert_eq!(records::civil_from_millis(next_daily), (2028, 2, 29));
    assert_eq!((next_daily / 1_000) % 86_400, 9 * 3_600);

    // Before 09:00 same day -> fires same day at 09:00.
    let feb28_0830_ms: u64 = (1_835_308_800 + 8 * 3_600 + 30 * 60) * 1_000;
    let same_day = scheduler::next_run_ms(
        &Cadence::Daily { hour: 9, minute: 0 },
        Some(feb28_0830_ms),
        feb28_0830_ms,
    )
    .unwrap();
    assert_eq!(records::civil_from_millis(same_day), (2028, 2, 28));
    assert_eq!((same_day / 1_000) % 86_400, 9 * 3_600);

    // Weekly Mon 09:00 from Mon 2028-02-28T10:00Z -> next Mon 2028-03-06T09:00Z (7 days later, across Feb 29!).
    let next_mon = scheduler::next_run_ms(
        &Cadence::Weekly {
            weekday: 0,
            hour: 9,
            minute: 0,
        },
        Some(feb28_1000_ms),
        feb28_1000_ms,
    )
    .unwrap();
    assert_eq!(records::civil_from_millis(next_mon), (2028, 3, 6));
    assert_eq!((next_mon / 1_000) % 86_400, 9 * 3_600);

    // Hourly :30 from 10:10 -> 10:30; from 10:30 -> 11:30.
    let at_1010 = (1_835_308_800 + 10 * 3_600 + 10 * 60) * 1_000;
    let at_1030 = (1_835_308_800 + 10 * 3_600 + 30 * 60) * 1_000;
    let at_1130 = (1_835_308_800 + 11 * 3_600 + 30 * 60) * 1_000;
    assert_eq!(
        scheduler::next_run_ms(&Cadence::Hourly { minute: 30 }, Some(at_1010), at_1010),
        Some(at_1030)
    );
    assert_eq!(
        scheduler::next_run_ms(&Cadence::Hourly { minute: 30 }, Some(at_1030), at_1030),
        Some(at_1130)
    );
}

#[test]
fn scheduler_pause_resume_due_detection_and_atomic_run_history() {
    let env = TestEnv::new("sched-history");

    let mut r1 = json!({
        "id": "rtn-1",
        "name": "Ledger Check",
        "prompt": "Reconcile invoices.csv",
        "schedule": "every 15m",
        "enabled": true,
        "created_ms": 1_700_000_000_000u64,
        "last_run_ms": Value::Null,
        "run_count": 0
    });
    let r2 = json!({
        "id": "rtn-2",
        "name": "Quick Pulse",
        "prompt": "Triage inbox.md",
        "schedule": "every 30s",
        "enabled": true,
        "created_ms": 1_700_000_000_000u64,
        "last_run_ms": Value::Null,
        "run_count": 0
    });

    // At +20s neither is due; at +40s only r2 is due; at +1000s both are due (r2 earliest).
    assert!(scheduler::find_due_routines(&[r1.clone(), r2.clone()], 1_700_000_020_000).is_empty());
    let due_40 = scheduler::find_due_routines(&[r1.clone(), r2.clone()], 1_700_000_040_000);
    assert_eq!(due_40.len(), 1);
    assert_eq!(due_40[0]["id"], json!("rtn-2"));

    let due_1000 = scheduler::find_due_routines(&[r1.clone(), r2.clone()], 1_700_001_000_000);
    assert_eq!(due_1000.len(), 2);
    assert_eq!(due_1000[0]["id"], json!("rtn-2"));
    assert_eq!(due_1000[1]["id"], json!("rtn-1"));

    // Pause r1 via apply_routine_patch -> never due while paused.
    scheduler::apply_routine_patch(&mut r1, &json!({"enabled": false})).unwrap();
    assert!(!scheduler::is_routine_due(&r1, 1_700_005_000_000));
    assert!(scheduler::compute_routine_next_run_ms(&r1, 1_700_005_000_000).is_none());

    // Resume and switch cadence via interval_secs.
    scheduler::apply_routine_patch(&mut r1, &json!({"enabled": true, "interval_secs": 60})).unwrap();
    assert_eq!(r1["schedule"], json!("every 1m"));
    assert!(scheduler::is_routine_due(&r1, 1_700_000_070_000));

    // Persist atomic RoutineRunRecords + filter + cap + corrupt backup recovery.
    let rec1 = scheduler::new_run_record(
        "rtn-1",
        "Ledger Check",
        1_700_000_060_000,
        1_700_000_061_250,
        "ok",
        Some("rcpt-100-0".to_string()),
        Some("thr-1".to_string()),
        "Reconciled 14 rows with 0 anomalies",
        "scheduled",
        None,
    );
    let rec2 = scheduler::new_run_record(
        "rtn-2",
        "Quick Pulse",
        1_700_000_090_000,
        1_700_000_090_500,
        "error",
        None,
        Some("thr-2".to_string()),
        "Missing inbox.md",
        "manual",
        Some("file not found".to_string()),
    );
    scheduler::record_run_history(&env.cfg.data_dir, rec1.clone()).unwrap();
    scheduler::record_run_history(&env.cfg.data_dir, rec2.clone()).unwrap();

    let all_hist = scheduler::list_run_history(&env.cfg.data_dir, None, 50);
    assert_eq!(all_hist.len(), 2);
    assert_eq!(all_hist[0].run_id, rec2.run_id); // newest-first
    assert_eq!(all_hist[1].duration_ms, 1250);

    let r1_hist = scheduler::list_run_history(&env.cfg.data_dir, Some("rtn-1"), 50);
    assert_eq!(r1_hist.len(), 1);
    assert_eq!(r1_hist[0].receipt_id.as_deref(), Some("rcpt-100-0"));

    let enriched = scheduler::enrich_routine_json(&r1, 1_700_000_070_000, &all_hist);
    assert_eq!(enriched["cadence_label"], json!("every 1m"));
    assert_eq!(enriched["due_now"], json!(true));
    assert_eq!(enriched["history_count"], json!(1));

    let status = scheduler::scheduler_status_json(
        true,
        15,
        &[r1.clone(), r2.clone()],
        &all_hist,
        1_700_000_070_000,
    );
    assert_eq!(status["active_routines"], json!(2));
    assert_eq!(status["due_count"], json!(2));
    assert_eq!(status["total_runs"], json!(2));

    // Corrupt routine_history.json is backed up cleanly, never panics.
    std::fs::write(scheduler::history_path(&env.cfg.data_dir), "{broken").unwrap();
    let recovered = scheduler::list_run_history(&env.cfg.data_dir, None, 10);
    assert!(recovered.is_empty());
    assert!(env.cfg.data_dir.join("routine_history.json.bak").exists());
}

// ---------------------------------------------------------------------------
// 3. audit_bundle tests
// ---------------------------------------------------------------------------

#[test]
fn audit_bundle_builds_verifies_detects_tampering_and_renders_clean_markdown() {
    let env = TestEnv::new("audit-bundle");

    // Seed workspace files + file revision history.
    env.write_ws("invoices.csv", b"vendor,amount\nNorthwind,420.50\n");
    file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "invoices.csv",
        "overwrite",
    )
    .unwrap();
    env.write_ws(
        "invoices.csv",
        b"vendor,amount\nNorthwind,420.50\nBoreal,985.25\n",
    );
    env.write_ws("notes/summary.md", b"# Summary\nAll reconciled.\n");

    // Seed standing memory.
    std::fs::write(
        &env.cfg.memory_path,
        "# Memory\n\n- Always verify duplicate invoice refs.\n- Round USD to 2 decimals.\n",
    )
    .unwrap();

    // Seed Field-Book work records (feeds receipt chain).
    records::append_work(
        &env.cfg,
        "Reconcile Q4 invoices",
        "Reconciled 2 vendors totaling $1,405.75.",
        &["invoices.csv".to_string(), "notes/summary.md".to_string()],
        "thr-101",
        true,
    );
    records::append_work(
        &env.cfg,
        "Draft weekly summary",
        "Saved notes/summary.md.",
        &["notes/summary.md".to_string()],
        "thr-102",
        true,
    );

    // Seed recurring routine + run history.
    let rtn = routines::create_routine(
        &env.cfg.data_dir,
        "Daily Ledger Audit",
        "Reconcile invoices.csv and flag anomalies",
        "daily 09:00",
        Some("reconcile_ledger"),
    )
    .unwrap();
    let run = scheduler::new_run_record(
        rtn["id"].as_str().unwrap(),
        "Daily Ledger Audit",
        1_700_000_000_000,
        1_700_000_001_500,
        "ok",
        Some("rcpt-1-0".to_string()),
        Some("thr-101".to_string()),
        "Completed daily reconciliation",
        "scheduled",
        None,
    );
    scheduler::record_run_history(&env.cfg.data_dir, run).unwrap();

    let person = Person {
        id: "maya".to_string(),
        name: "Maya Lin".to_string(),
        role: "Finance Operations".to_string(),
    };

    // Build & verify clean Audit Bundle.
    let bundle = audit_bundle::build_audit_bundle(&env.cfg, &person);
    let check = audit_bundle::verify_audit_bundle(&bundle);
    assert!(
        check.valid,
        "fresh audit bundle must verify cleanly: {:?}",
        check.errors
    );
    assert!(check.receipt_chain_valid);
    assert!(check.workspace_manifest_valid);
    assert!(check.routines_digest_valid);
    assert!(check.bundle_hash_valid);
    assert_eq!(check.receipt_count, 2);
    assert_eq!(check.workspace_file_count, 2);

    // Revision count from .history/ is surfaced in workspace_files.
    let inv_entry = bundle["workspace_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == json!("invoices.csv"))
        .unwrap();
    assert_eq!(inv_entry["revision_count"], json!(1));

    // Live workspace cross-check passes before edits, catches edits afterward.
    let ws_ok = audit_bundle::verify_workspace_against_bundle(&bundle, &env.cfg.workspace_dir);
    assert_eq!(ws_ok["all_matched"], json!(true));
    assert_eq!(ws_ok["matched_count"], json!(2));

    env.write_ws("invoices.csv", b"vendor,amount\nTAMPERED,0.01\n");
    let ws_drift = audit_bundle::verify_workspace_against_bundle(&bundle, &env.cfg.workspace_dir);
    assert_eq!(ws_drift["all_matched"], json!(false));
    assert_eq!(ws_drift["modified_count"], json!(1));

    // Tampering with ANY field in the bundle fails independent verification.
    let mut tamper_receipt = bundle.clone();
    tamper_receipt["receipt_chain"]["entries"][0]["summary"] = json!("Fabricated summary");
    let v1 = audit_bundle::verify_audit_bundle(&tamper_receipt);
    assert!(!v1.valid);
    assert!(!v1.receipt_chain_valid);
    assert!(!v1.bundle_hash_valid);

    let mut tamper_file = bundle.clone();
    tamper_file["workspace_files"][0]["sha256"] = json!("deadbeef");
    let v2 = audit_bundle::verify_audit_bundle(&tamper_file);
    assert!(!v2.valid);
    assert!(!v2.workspace_manifest_valid);
    assert!(!v2.bundle_hash_valid);

    let mut tamper_routine = bundle.clone();
    tamper_routine["routines"]["run_history"][0]["status"] = json!("forged");
    let v3 = audit_bundle::verify_audit_bundle(&tamper_routine);
    assert!(!v3.valid);
    assert!(!v3.routines_digest_valid);
    assert!(!v3.bundle_hash_valid);

    // Markdown export contains key sections and ZERO internal Rune jargon.
    let md = audit_bundle::render_audit_bundle_markdown(&bundle);
    assert!(md.contains("# Ergon Field-Book Audit Bundle"), "{md}");
    assert!(md.contains("Maya Lin"), "{md}");
    assert!(md.contains("invoices.csv"), "{md}");
    assert!(md.contains("Daily Ledger Audit"), "{md}");
    assert!(md.contains("Independent Verification Instructions"), "{md}");
    assert!(
        !md.to_lowercase().contains("rune"),
        "audit bundle Markdown must contain zero internal Rune jargon:\n{md}"
    );
}

#[test]
fn file_history_blocks_symlinked_parent_dir_and_tracks_deleted_files() {
    let env = TestEnv::new("fh-symlink-parent");
    env.write_ws("notes.md", b"v1\n");
    let rev = file_history::record_revision(
        &env.cfg.data_dir,
        &env.cfg.workspace_dir,
        "notes.md",
        "delete",
    )
    .unwrap()
    .unwrap();
    std::fs::remove_file(env.cfg.workspace_dir.join("notes.md")).unwrap();

    // Deleted file remains discoverable via list_tracked_files for recovery UI.
    let tracked = file_history::list_tracked_files(&env.cfg.data_dir);
    assert_eq!(tracked.len(), 1);
    assert_eq!(tracked[0]["path"], json!("notes.md"));
    assert_eq!(tracked[0]["latest_rev_id"], json!(rev.rev_id));

    // Symlinked parent directory pointing outside workspace must be blocked.
    #[cfg(unix)]
    {
        let outside_dir = env.root.join("outside_dir");
        std::fs::create_dir_all(&outside_dir).unwrap();
        std::os::unix::fs::symlink(&outside_dir, env.cfg.workspace_dir.join("dir_escape")).unwrap();
        assert!(
            file_history::record_revision_bytes(
                &env.cfg.data_dir,
                &env.cfg.workspace_dir,
                "dir_escape/pwn.txt",
                b"bad",
                "overwrite",
            )
            .is_err(),
            "symlinked parent directory must be refused"
        );
        assert!(
            file_history::restore_revision(
                &env.cfg.data_dir,
                &env.cfg.workspace_dir,
                "dir_escape/pwn.txt",
                &rev.rev_id,
            )
            .is_err()
        );
        assert!(!outside_dir.join("pwn.txt").exists());
    }
}

#[test]
fn audit_bundle_empty_workspace_and_scale_performance() {
    let env = TestEnv::new("audit-scale");
    let person = Person {
        id: "default".to_string(),
        name: "You".to_string(),
        role: "Operations".to_string(),
    };

    // Empty fresh workspace & 0 receipts must still verify 100% cleanly.
    let empty_bundle = audit_bundle::build_audit_bundle(&env.cfg, &person);
    let empty_check = audit_bundle::verify_audit_bundle(&empty_bundle);
    assert!(empty_check.valid, "{:?}", empty_check.errors);
    assert_eq!(empty_check.receipt_count, 0);
    assert_eq!(empty_check.workspace_file_count, 0);

    // Scale check: 100 workspace files + 500 Field-Book receipts + 505 routine history runs.
    for i in 0..100 {
        env.write_ws(
            &format!("batch/doc-{i:03}.txt"),
            format!("Document {i} verified payload\n").as_bytes(),
        );
    }
    let mut seeded_receipts = Vec::with_capacity(500);
    for i in 0..500u64 {
        seeded_receipts.push(json!({
            "id": format!("rcpt-scale-{i}"),
            "ts": 1_700_000_000_000u64 + i * 1_000,
            "task": format!("Task {i}"),
            "summary": format!("Completed batch item {i}"),
            "files": [format!("batch/doc-{:03}.txt", i % 100)],
            "thread_id": format!("thr-{i}"),
            "finished": true,
            "answered": false
        }));
    }
    std::fs::write(
        env.cfg.data_dir.join("receipts.json"),
        json!({ "receipts": seeded_receipts }).to_string(),
    )
    .unwrap();

    // Routine history cap check (505 inserts -> capped cleanly at MAX_RUN_HISTORY = 500).
    for i in 0..505u64 {
        let rec = scheduler::new_run_record(
            "rtn-scale",
            "Scale Routine",
            1_700_000_000_000 + i * 1_000,
            1_700_000_000_500 + i * 1_000,
            "ok",
            Some(format!("rcpt-scale-{}", i % 500)),
            None,
            "ok",
            "scheduled",
            None,
        );
        scheduler::record_run_history(&env.cfg.data_dir, rec).unwrap();
    }
    let stored_hist = scheduler::list_run_history(&env.cfg.data_dir, None, 0);
    assert_eq!(stored_hist.len(), scheduler::MAX_RUN_HISTORY);

    let started = std::time::Instant::now();
    let big_bundle = audit_bundle::build_audit_bundle(&env.cfg, &person);
    let big_check = audit_bundle::verify_audit_bundle(&big_bundle);
    let elapsed_ms = started.elapsed().as_millis();
    assert!(big_check.valid, "{:?}", big_check.errors);
    assert_eq!(big_check.receipt_count, 500);
    assert_eq!(big_check.workspace_file_count, 100);
    assert!(
        elapsed_ms < 2_000,
        "audit bundle build + verify took too long: {elapsed_ms}ms"
    );
}
