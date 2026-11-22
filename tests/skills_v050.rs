//! Comprehensive unit + integration tests for the v0.5.0 12-skill Rune library,
//! `skill_catalog` metadata / validation / dry-run harness, strict sandbox bounds,
//! and large-scale 500-row CSV execution under the 5M instruction budget.

use ergon_rs::{skill_catalog, skills, Config};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

struct TestEnv {
    root: PathBuf,
    cfg: Config,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ergon-skills-v050-{}-{}-{name}",
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

    fn write_ws(&self, rel: &str, content: &str) {
        let full = self.cfg.workspace_dir.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }

    fn read_ws(&self, rel: &str) -> String {
        std::fs::read_to_string(self.cfg.workspace_dir.join(rel)).unwrap()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn catalog_exposes_all_12_skills_and_compiles_every_rune_script() {
    let env = TestEnv::new("validate-all");
    let catalog = skill_catalog::all_skills();
    assert_eq!(catalog.len(), 12, "expected exactly 12 built-in Rune skills");

    for id in skills::ALL_BUILTIN_SKILL_IDS {
        let meta = skill_catalog::get_skill(id)
            .unwrap_or_else(|| panic!("missing catalog entry for {id}"));
        assert_eq!(meta.id, id);
        assert!(
            skill_catalog::get_skill(&meta.slug).is_some(),
            "slug lookup failed for {}",
            meta.slug
        );
        assert!(!meta.title.is_empty());
        assert!(!meta.category.is_empty());
        assert!(!meta.description.is_empty());
        assert!(!meta.input_schema.is_empty());
        assert!(!meta.sample_prompts.is_empty());
        assert!(!meta.source.is_empty());
    }

    let report = skill_catalog::validate_all_skills(&env.cfg);
    for entry in &report.results {
        assert!(
            entry.compiled,
            "skill '{}' failed to compile in Rune VM: {:?}",
            entry.id, entry.error
        );
    }
    assert!(report.all_ok);
    assert_eq!(report.total, 12);
    assert_eq!(report.passed, 12);
    assert_eq!(report.failed, 0);

    let json_payload = skill_catalog::catalog_json();
    assert_eq!(json_payload["count"], json!(12));
    assert_eq!(json_payload["skills"].as_array().unwrap().len(), 12);
}

#[test]
fn dry_run_harness_executes_all_12_skills_with_default_fixtures() {
    for id in skills::ALL_BUILTIN_SKILL_IDS {
        let res = skill_catalog::dry_run_skill(id, &json!({}), None, None);
        assert!(
            res.ok && res.compiled,
            "dry_run_skill({id}) failed: error={:?} output={}",
            res.error,
            res.output
        );
        if id == "update_memory" {
            let mem = res.memory_after.unwrap_or_default();
            assert!(mem.contains("Prefer concise executive bullet lists"), "{mem}");
        } else {
            assert!(
                !res.files_written.is_empty(),
                "skill {id} should produce at least one workspace file"
            );
            for f in &res.files_written {
                assert!(f.bytes > 0, "skill {id} wrote empty file {}", f.path);
            }
        }
    }
}

#[test]
fn handle_skill_test_request_supports_builtin_custom_and_validate_all() {
    let val = skill_catalog::handle_skill_test_request(&json!({"validate_all": true}));
    assert_eq!(val["all_ok"], json!(true));
    assert_eq!(val["total"], json!(12));

    let custom_ok = skill_catalog::handle_skill_test_request(&json!({
        "source": "pub fn main(name) { let r = host_workspace_write(\"hello.txt\", `hi ${name}`); if r != \"ok\" { return r; } \"file:hello.txt\" }",
        "params": ["Ada"]
    }));
    assert_eq!(custom_ok["ok"], json!(true), "{custom_ok}");
    assert_eq!(custom_ok["files_written"][0]["path"], json!("hello.txt"));

    let custom_bad_syntax = skill_catalog::handle_skill_test_request(&json!({
        "source": "pub fn main( {"
    }));
    assert_eq!(custom_bad_syntax["ok"], json!(false));
    assert_eq!(custom_bad_syntax["compiled"], json!(false));

    let custom_escape = skill_catalog::handle_skill_test_request(&json!({
        "source": "pub fn main() { host_workspace_write(\"../pwn.txt\", \"nope\") }"
    }));
    assert_eq!(custom_escape["ok"], json!(false));
    assert!(
        custom_escape["error"]
            .as_str()
            .unwrap_or("")
            .contains("escapes"),
        "{custom_escape}"
    );
}

#[test]
fn reconcile_ledger_multi_csv_matching_anomalies_and_exact_totals() {
    let env = TestEnv::new("reconcile-multi");
    env.write_ws(
        "invoices.csv",
        "Date,Invoice Ref,Vendor,Category,Amount,Receipt\n\
         2026-11-01,INV-1001,Northwind Supply,Operations,$420.50,verified\n\
         2026-11-02,INV-1002,Boreal Optics,Equipment,$985.25,missing\n\
         2026-11-03,INV-1001,Northwind Supply,Operations,$420.50,verified\n\
         2026-11-04,INV-1003,Alder Forge,Logistics,\"$1,250.00\",none\n",
    );
    env.write_ws(
        "bank_feed.csv",
        "posted,reference,merchant,department,total,status\n\
         2026-11-01,INV-1001,Northwind Supply,Operations,420.50,verified\n\
         2026-11-02,INV-1002,Boreal Optics,Equipment,999.00,verified\n\
         2026-11-05,INV-1004,Vesper Crane,Equipment,300.00,verified\n",
    );

    let out = skills::reconcile_ledger(
        &env.cfg,
        vec!["invoices.csv".to_string(), "bank_feed.csv".to_string()],
        "Q4 Reconciliation",
    )
    .unwrap();
    assert!(out.contains("Q4 Reconciliation.md"), "{out}");
    assert!(out.contains("Q4 Reconciliation - Summary.csv"), "{out}");

    let report = env.read_ws("Q4 Reconciliation.md");
    // Exact combined sum: 420.50 + 985.25 + 420.50 + 1250.00 + 420.50 + 999.00 + 300.00 = 4,795.75
    assert!(report.contains("$4,795.75"), "exact penny total missing:\n{report}");
    assert!(report.contains("| Duplicate references flagged | 1 |"), "{report}");
    assert!(report.contains("invoices.csv:L4"), "duplicate citation missing:\n{report}");
    assert!(report.contains("| Missing receipts flagged | 2 |"), "{report}");
    assert!(report.contains("Boreal Optics"), "{report}");
    assert!(report.contains("Alder Forge"), "{report}");
    assert!(report.contains("| Cross-file matched references | 2 |"), "{report}");
    assert!(report.contains("| Cross-file amount mismatches | 1 |"), "{report}");
    assert!(report.contains("$985.25") && report.contains("$999.00"), "{report}");

    let summary_csv = env.read_ws("Q4 Reconciliation - Summary.csv");
    assert!(summary_csv.contains("vendor,Boreal Optics,2,1984.25"), "{summary_csv}");
    assert!(summary_csv.contains("category,Equipment,3,2284.25"), "{summary_csv}");
}

#[test]
fn summarize_meeting_extracts_decisions_owners_dates_and_followup_drafts() {
    let env = TestEnv::new("meeting");
    env.write_ws(
        "weekly-sync.md",
        "Attendees: Maya, Devon, Kenji\n\
         Maya: We decided to adopt the new CSV reconciliation gate before Monday.\n\
         Action: Devon will migrate the billing export by Friday.\n\
         Kenji: Owner: Kenji - update vendor policy doc due 2026-11-30.\n\
         Maya: Approved moving storage backups to nightly snapshots.\n\
         Devon: Blocker: Are regional invoice IDs guaranteed unique?\n",
    );

    let out = skills::summarize_meeting(&env.cfg, "weekly-sync.md", "Weekly Sync Recap").unwrap();
    assert!(out.contains("Weekly Sync Recap.md"), "{out}");

    let md = env.read_ws("Weekly Sync Recap.md");
    assert!(md.contains("Maya, Devon, Kenji"), "{md}");
    assert!(md.contains("[L2]"), "decision line citation missing:\n{md}");
    assert!(md.contains("| [L3] | Devon | Friday |"), "action owner/date missing:\n{md}");
    assert!(md.contains("| [L4] | Kenji | 2026-11-30 |"), "owner/due missing:\n{md}");
    assert!(md.contains("#### Draft to Devon"), "per-owner follow-up draft missing:\n{md}");
    assert!(md.contains("#### Draft to Kenji"), "per-owner follow-up draft missing:\n{md}");
    assert!(md.contains("[L6]"), "open question/blocker missing:\n{md}");
}

#[test]
fn audit_contract_flags_missing_clauses_renewals_and_line_cited_risks() {
    let env = TestEnv::new("contract");
    env.write_ws(
        "supplier-msa.txt",
        "MASTER SERVICES AGREEMENT\n\
         1. Effective Date: 2026-12-01 with an Initial Term of 24 months.\n\
         2. Renewal: This agreement shall automatically renew unless 90 days prior written notice is delivered.\n\
         3. Changes: Supplier may modify fees and terms at its sole discretion.\n\
         4. Liability: Customer accepts unlimited liability for indirect claims.\n\
         5. Payment: Invoices are payable Net 90 with a 2% monthly late fee.\n\
         6. Termination: Either party may terminate for uncured breach after 30 days.\n",
    );

    let out = skills::audit_contract(
        &env.cfg,
        "supplier-msa.txt",
        vec![
            "termination".to_string(),
            "liability".to_string(),
            "confidentiality".to_string(),
            "data protection".to_string(),
            "renewal".to_string(),
            "escrow".to_string(),
        ],
        "Supplier MSA Audit",
    )
    .unwrap();
    assert!(out.contains("Supplier MSA Audit.md"), "{out}");

    let md = env.read_ws("Supplier MSA Audit.md");
    assert!(md.contains("**HIGH**"), "{md}");
    assert!(md.contains("| confidentiality | **MISSING** |"), "{md}");
    assert!(md.contains("| data protection | **MISSING** |"), "{md}");
    assert!(md.contains("| escrow | **MISSING** |"), "{md}");
    assert!(md.contains("| termination | **PRESENT** | [L7] |"), "{md}");
    assert!(md.contains("| **HIGH** | Auto-Renewal Lock-In | [L3] |"), "{md}");
    assert!(md.contains("| **HIGH** | Unilateral Amendment | [L4] |"), "{md}");
    assert!(md.contains("| **HIGH** | Unlimited Liability | [L5] |"), "{md}");
    assert!(md.contains("| **MEDIUM** | Extended Payment Terms | [L6] |"), "{md}");
}

#[test]
fn triage_inbox_prioritizes_urgent_action_and_fyi_across_formats() {
    let env = TestEnv::new("triage");
    env.write_ws(
        "tickets.csv",
        "id,from,subject,body,priority\n\
         T-101,ops@company.local,SEV1 Production outage on checkout,Payment gateway is down and escalating,critical\n\
         T-102,ap@vendor.local,Please approve invoice #882,Can you review and sign off by tomorrow?,normal\n\
         T-103,no-reply@news.local,Weekly digest and release notes,FYI: automated newsletter update,low\n\
         T-104,sec@company.local,Unauthorized login alert,Potential security breach detected on admin token,high\n",
    );

    let out = skills::triage_inbox(&env.cfg, "tickets.csv", "Support Triage").unwrap();
    assert!(out.contains("Support Triage.md"), "{out}");

    let md = env.read_ws("Support Triage.md");
    assert!(md.contains("| **Urgent** | 2 |"), "{md}");
    assert!(md.contains("| **Action** | 1 |"), "{md}");
    assert!(md.contains("| **FYI** | 1 |"), "{md}");
    assert!(md.contains("[T-101]"), "{md}");
    assert!(md.contains("[T-104]"), "{md}");
    assert!(md.contains("- [ ] **[URGENT]**"), "{md}");
}

#[test]
fn clean_dataset_normalizes_json_and_csv_dedupes_flags_nulls_and_outliers() {
    let env = TestEnv::new("clean-json");
    env.write_ws(
        "customers.json",
        r#"[
            {"Order ID": "A1", "Customer Name": "  Maya  ", "Total USD": "100.00", "Region": "West"},
            {"Order ID": "A2", "Customer Name": "Bob", "Total USD": "104.00", "Region": "N/A"},
            {"Order ID": "A3", "Customer Name": "Cara", "Total USD": "98.50", "Region": "East"},
            {"Order ID": "A4", "Customer Name": "Dan", "Total USD": "101.25", "Region": "none"},
            {"Order ID": "A5", "Customer Name": "Eli", "Total USD": "99.75", "Region": "North"},
            {"Order ID": "A6", "Customer Name": "Fay", "Total USD": "102.00", "Region": "South"},
            {"Order ID": "A7", "Customer Name": "Gus", "Total USD": "9950.00", "Region": "West"},
            {"Order ID": "A2", "Customer Name": "Bob Duplicate", "Total USD": "104.00", "Region": "West"}
        ]"#,
    );

    let out = skills::clean_dataset(&env.cfg, "customers.json", "Order ID", "Clean Customers")
        .unwrap();
    assert!(out.contains("Clean Customers.csv"), "{out}");
    assert!(out.contains("Clean Customers - Diff Report.md"), "{out}");

    let csv = env.read_ws("Clean Customers.csv");
    let rows = skills::parse_csv_rows(&csv);
    assert_eq!(rows.len(), 8, "1 header + 7 unique rows:\n{csv}");
    assert!(rows[0].contains(&"order_id".to_string()), "{:?}", rows[0]);
    assert!(rows[0].contains(&"total_usd".to_string()), "{:?}", rows[0]);

    let report = env.read_ws("Clean Customers - Diff Report.md");
    assert!(report.contains("| Input data rows | 8 |"), "{report}");
    assert!(report.contains("| Output clean rows | 7 |"), "{report}");
    assert!(report.contains("| Duplicate rows removed | 1 |"), "{report}");
    assert!(report.contains("| Null / empty cells flagged | 2 |"), "{report}");
    assert!(report.contains("9950.00"), "outlier missing:\n{report}");
}

#[test]
fn compare_documents_detects_added_removed_and_heading_changes() {
    let env = TestEnv::new("compare");
    env.write_ws(
        "handbook-v1.md",
        "# Employee Handbook\n\
         ## Expenses\n\
         Meals cap is $40 per day.\n\
         Receipts required above $50.\n\
         ## Remote Work\n\
         Core hours are 10am to 4pm.\n",
    );
    env.write_ws(
        "handbook-v2.md",
        "# Employee Handbook\n\
         ## Expenses\n\
         Meals cap is $60 per day.\n\
         Receipts required above $25.\n\
         ## Security\n\
         Hardware keys are mandatory.\n\
         ## Remote Work\n\
         Core hours are 10am to 4pm.\n",
    );

    let out = skills::compare_documents(
        &env.cfg,
        "handbook-v1.md",
        "handbook-v2.md",
        "Handbook Diff",
    )
    .unwrap();
    assert!(out.contains("Handbook Diff.md"), "{out}");

    let md = env.read_ws("Handbook Diff.md");
    assert!(md.contains("| Unchanged lines | 4 |"), "{md}");
    assert!(md.contains("| Lines added (+) | 4 |"), "{md}");
    assert!(md.contains("| Lines removed (-) | 2 |"), "{md}");
    assert!(md.contains("Added: [R5] ## Security"), "{md}");
    assert!(md.contains("- [L3] Meals cap is $40 per day."), "{md}");
    assert!(md.contains("+ [R3] Meals cap is $60 per day."), "{md}");
}

#[test]
fn draft_brief_grounds_in_sources_and_memory_and_halts_on_missing_sources() {
    let env = TestEnv::new("brief");
    skills::update_memory(
        &env.cfg,
        "Always list dollar figures with two decimals and flag missing receipts first.",
    )
    .unwrap();
    env.write_ws(
        "q4-audit.md",
        "# Q4 Expense Audit\n\
         - Total audited spend: $28,410.50 across 4 departments.\n\
         - Risk: 3 travel expenses are missing itemized receipts.\n\
         1. Must hold reimbursement until missing receipts are uploaded.\n\
         2. Verify duplicate references before closing the monthly ledger.\n",
    );

    let out = skills::draft_brief(
        &env.cfg,
        "Q4 Reimbursement Controls",
        vec!["q4-audit.md".to_string(), "missing-annex.md".to_string()],
        "Finance Ops",
        "Q4 Controls Memo",
    )
    .unwrap();
    assert!(out.contains("Q4 Controls Memo.md"), "{out}");

    let md = env.read_ws("Q4 Controls Memo.md");
    assert!(md.contains("[q4-audit.md:L2]"), "line citation missing:\n{md}");
    assert!(md.contains("$28,410.50"), "{md}");
    assert!(
        md.contains("Always list dollar figures with two decimals"),
        "standing memory not applied:\n{md}"
    );
    assert!(
        md.contains("missing-annex.md"),
        "missing requested source must be disclosed:\n{md}"
    );

    // Zero readable sources must halt instead of fabricating.
    let halt = skills::draft_brief(
        &env.cfg,
        "Empty Topic",
        vec!["does-not-exist.md".to_string()],
        "Execs",
        "Should Not Exist",
    );
    assert!(halt.is_err(), "expected halt when zero sources exist");
    assert!(!env.cfg.workspace_dir.join("Should Not Exist.md").exists());
}

#[test]
fn extract_tables_parses_markdown_tables_and_structured_kv_logs() {
    let env = TestEnv::new("extract-tables");
    env.write_ws(
        "review.md",
        "# Quarterly Review\n\n\
         | Region | Revenue | Growth |\n\
         |---|---:|---:|\n\
         | North, East | 125000 | 12% |\n\
         | West | 98000 | 8% |\n\n\
         Notes between tables.\n\n\
         | Owner | Deliverable |\n\
         |---|---|\n\
         | Maya | Forecast |\n",
    );

    let out = skills::extract_tables(&env.cfg, "review.md", "Review Export").unwrap();
    assert!(out.contains("Review Export.csv"), "{out}");
    assert!(out.contains("Review Export - Table 2.csv"), "{out}");

    let csv1 = env.read_ws("Review Export.csv");
    assert!(csv1.contains("\"North, East\",125000,12%"), "{csv1}");
    let csv2 = env.read_ws("Review Export - Table 2.csv");
    assert!(csv2.contains("Owner,Deliverable\nMaya,Forecast"), "{csv2}");

    env.write_ws(
        "billing.log",
        "2026-11-20T09:00:01Z [INFO] service=billing invoice=INV-1 amount=42.50 msg=\"charged ok\"\n\
         2026-11-20T09:00:02Z [ERROR] service=billing invoice=INV-2 amount=99.00 msg=\"card declined, retry\"\n",
    );
    let out_log = skills::extract_tables(&env.cfg, "billing.log", "Billing Events").unwrap();
    assert!(out_log.contains("Billing Events.csv"), "{out_log}");
    let log_csv = env.read_ws("Billing Events.csv");
    let rows = skills::parse_csv_rows(&log_csv);
    assert_eq!(rows.len(), 3, "{log_csv}");
    assert!(rows[0].contains(&"timestamp".to_string()));
    assert!(rows[0].contains(&"service".to_string()));
    assert!(rows[2].contains(&"card declined, retry".to_string()));
}

#[test]
fn strict_sandbox_bounds_block_traversal_symlinks_and_runaway_loops() {
    let env = TestEnv::new("sandbox-bounds");

    // Path traversal refused across skills
    assert!(skills::reconcile_ledger(&env.cfg, vec!["../secret.csv".into()], "Out").is_err());
    assert!(skills::summarize_meeting(&env.cfg, "/etc/passwd", "Out").is_err());
    assert!(skills::audit_contract(&env.cfg, "../../etc/hosts", vec![], "Out").is_err());
    assert!(skills::triage_inbox(&env.cfg, "../inbox.md", "Out").is_err());
    assert!(skills::clean_dataset(&env.cfg, "../data.csv", "", "Out").is_err());
    assert!(skills::compare_documents(&env.cfg, "../a.txt", "b.txt", "Out").is_err());
    assert!(skills::draft_brief(&env.cfg, "Test", vec!["../secret.txt".into()], "Exec", "Out").is_err());
    assert!(skills::extract_tables(&env.cfg, "../logs.txt", "Out").is_err());

    // Output title traversal refused
    env.write_ws("ok.md", "# Title\n- item\n");
    assert!(skills::summarize_meeting(&env.cfg, "ok.md", "../escaped-report").is_err());

    // Symlink escape refused
    #[cfg(unix)]
    {
        let outside = env.root.join("outside-secret.csv");
        std::fs::write(&outside, "vendor,amount\nEvil,999\n").unwrap();
        std::os::unix::fs::symlink(&outside, env.cfg.workspace_dir.join("link.csv")).unwrap();
        let res = skills::reconcile_ledger(&env.cfg, vec!["link.csv".into()], "Leak");
        assert!(res.is_err(), "symlink escape must be refused: {res:?}");
    }

    // Runaway infinite loop terminated cleanly by 5M budget
    let runaway = skill_catalog::dry_run_custom_source("pub fn main() { loop {} }", &[], None, None);
    assert!(!runaway.ok);
    assert!(runaway.compiled);
    assert!(runaway.error.is_some());
}

#[test]
fn large_500_row_csv_and_doc_scale_under_strict_rune_budget() {
    let env = TestEnv::new("scale-500");

    // 1. Build a realistic 500-row CSV with known totals, 5 duplicate refs, 25 missing receipts,
    //    and 2 statistical outliers!
    let mut csv_in = String::from("Date,Invoice Ref,Vendor,Category,Amount,Receipt\n");
    let mut rows_for_sheet: Vec<Vec<String>> = Vec::with_capacity(500);
    let mut expected_total_cents: i64 = 0;

    let vendors = ["Northwind Supply", "Boreal Optics", "Alder Forge", "Vesper Crane", "Cedar Cloud"];
    let categories = ["Operations", "Equipment", "Logistics", "Software", "Facilities"];

    for i in 0..500usize {
        let vendor = vendors[i % vendors.len()];
        let cat = categories[i % categories.len()];
        // 5 duplicate refs at rows 495..500 duplicating INV-0001..INV-0005
        let ref_id = if i >= 495 {
            format!("INV-{:04}", i - 494)
        } else {
            format!("INV-{:04}", i + 1)
        };
        // 2 deliberate outliers (i == 100 and i == 300)
        let cents: i64 = if i == 100 || i == 300 {
            25_000_000 // $250,000.00
        } else {
            10_000 + ((i % 25) as i64) * 125 // $100.00 .. $130.00
        };
        expected_total_cents += cents;
        let amt = format!("{}.{:02}", cents / 100, cents % 100);
        // 25 missing receipts (every 20th row)
        let receipt = if i % 20 == 0 { "missing" } else { "verified" };
        let date = format!("2026-11-{:02}", (i % 28) + 1);
        csv_in.push_str(&format!("{date},{ref_id},{vendor},{cat},{amt},{receipt}\n"));
        rows_for_sheet.push(vec![
            date,
            ref_id,
            vendor.to_string(),
            cat.to_string(),
            amt,
            receipt.to_string(),
        ]);
    }
    env.write_ws("ledger-500.csv", &csv_in);

    // A) make_spreadsheet on 500 rows
    let t0 = Instant::now();
    let sheet_res = skills::make_spreadsheet(
        &env.cfg,
        "Export 500",
        vec![
            "Date".into(),
            "Ref".into(),
            "Vendor".into(),
            "Category".into(),
            "Amount".into(),
            "Receipt".into(),
        ],
        rows_for_sheet,
    )
    .unwrap();
    assert!(sheet_res.contains("500 data rows"), "{sheet_res}");
    assert!(t0.elapsed().as_secs() < 10, "make_spreadsheet 500 rows too slow");

    // B) reconcile_ledger on 500-row CSV
    let t1 = Instant::now();
    let rec_res = skills::reconcile_ledger(
        &env.cfg,
        vec!["ledger-500.csv".into()],
        "Scale 500 Reconciliation",
    )
    .unwrap();
    assert!(rec_res.contains("Scale 500 Reconciliation.md"), "{rec_res}");
    assert!(t1.elapsed().as_secs() < 10, "reconcile_ledger 500 rows too slow");

    let rec_md = env.read_ws("Scale 500 Reconciliation.md");
    assert!(rec_md.contains("| Total line items processed | 500 |"), "{rec_md}");
    assert!(rec_md.contains("| Duplicate references flagged | 5 |"), "{rec_md}");
    assert!(rec_md.contains("| Missing receipts flagged | 25 |"), "{rec_md}");
    // Verify penny-exact expected total
    let dollars = expected_total_cents / 100;
    let rem = expected_total_cents % 100;
    let d_str = dollars.to_string();
    let mut formatted = String::new();
    for (idx, ch) in d_str.chars().enumerate() {
        if idx > 0 && (d_str.len() - idx) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }
    let expected_money = format!("${formatted}.{rem:02}");
    assert!(
        rec_md.contains(&expected_money),
        "expected exact total {expected_money} in report:\n{rec_md}"
    );

    // C) clean_dataset on 500-row CSV (dedups the 5 duplicate refs -> 495 rows, flags 2 outliers!)
    let t2 = Instant::now();
    let clean_res = skills::clean_dataset(
        &env.cfg,
        "ledger-500.csv",
        "Invoice Ref",
        "Scale 500 Clean",
    )
    .unwrap();
    assert!(clean_res.contains("Scale 500 Clean.csv"), "{clean_res}");
    assert!(t2.elapsed().as_secs() < 10, "clean_dataset 500 rows too slow");

    let clean_md = env.read_ws("Scale 500 Clean - Diff Report.md");
    assert!(clean_md.contains("| Input data rows | 500 |"), "{clean_md}");
    assert!(clean_md.contains("| Output clean rows | 495 |"), "{clean_md}");
    assert!(clean_md.contains("| Duplicate rows removed | 5 |"), "{clean_md}");
    assert!(
        clean_md.contains("| Statistical outliers flagged (|z| >= 2.5) | 2 |"),
        "{clean_md}"
    );

    // D) compare_documents on two 500-line files
    let mut left_500 = String::new();
    let mut right_500 = String::new();
    for i in 1..=500 {
        left_500.push_str(&format!("Clause {i}: Standard operating rule number {i}.\n"));
        if i % 50 == 0 {
            right_500.push_str(&format!("Clause {i}: REVISED operating rule number {i}.\n"));
        } else {
            right_500.push_str(&format!("Clause {i}: Standard operating rule number {i}.\n"));
        }
    }
    env.write_ws("doc-500-a.txt", &left_500);
    env.write_ws("doc-500-b.txt", &right_500);
    let t3 = Instant::now();
    let diff_res = skills::compare_documents(
        &env.cfg,
        "doc-500-a.txt",
        "doc-500-b.txt",
        "Scale 500 Diff",
    )
    .unwrap();
    assert!(diff_res.contains("Scale 500 Diff.md"), "{diff_res}");
    assert!(t3.elapsed().as_secs() < 10, "compare_documents 500 lines too slow");
    let diff_md = env.read_ws("Scale 500 Diff.md");
    assert!(diff_md.contains("| Unchanged lines | 490 |"), "{diff_md}");
    assert!(diff_md.contains("| Lines added (+) | 10 |"), "{diff_md}");
    assert!(diff_md.contains("| Lines removed (-) | 10 |"), "{diff_md}");
}

#[test]
fn csv_and_json_parsers_handle_rfc4180_bom_newlines_and_wrappers() {
    let csv = "\u{feff}Name,Notes,Amount\r\n\"Acme, \"\"North\"\"\",\"line1\nline2\",12.50\r\nBeta,plain,4.00\r\n\r\n";
    let rows = skills::parse_csv_rows(csv);
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert_eq!(rows[0], vec!["Name", "Notes", "Amount"]);
    assert_eq!(rows[1], vec!["Acme, \"North\"", "line1\nline2", "12.50"]);
    assert_eq!(rows[2], vec!["Beta", "plain", "4.00"]);

    let wrapped = r#"{"meta":{"v":1},"data":[{"id":1,"name":"Alpha","active":true},{"id":2,"name":null,"active":false}]}"#;
    let tbl = skills::parse_json_table_rows(wrapped);
    assert_eq!(tbl.len(), 3, "{tbl:?}");
    assert_eq!(tbl[0], vec!["active", "id", "name"]);
    assert_eq!(tbl[1], vec!["true", "1", "Alpha"]);
    assert_eq!(tbl[2], vec!["false", "2", ""]);

    let aoa = r#"[["k","v"],["x",10]]"#;
    let tbl2 = skills::parse_json_table_rows(aoa);
    assert_eq!(tbl2, vec![vec!["k", "v"], vec!["x", "10"]]);

    let single = r#"{"vendor":"Solstice","amount":45}"#;
    let tbl3 = skills::parse_json_table_rows(single);
    assert_eq!(tbl3.len(), 2);

    assert!(skills::parse_json_table_rows("{not valid json").is_empty());
}

#[test]
fn reconcile_ledger_catches_second_file_duplicates_and_accounting_negatives() {
    let env = TestEnv::new("reconcile-edge");
    env.write_ws(
        "file_a.csv",
        "Date,Ref,Vendor,Amount\n\
         2026-11-01,REF-1,Northwind,100.00\n",
    );
    env.write_ws(
        "file_b.csv",
        "Date,Ref,Vendor,Amount\n\
         2026-11-01,REF-1,Northwind,100.00\n\
         2026-11-02,REF-1,Northwind,100.00\n\
         2026-11-03,REF-2,Boreal,($25.50)\n",
    );

    skills::reconcile_ledger(
        &env.cfg,
        vec!["file_a.csv".into(), "file_b.csv".into()],
        "Edge Reconcile",
    )
    .unwrap();
    let md = env.read_ws("Edge Reconcile.md");
    assert!(md.contains("| Duplicate references flagged | 1 |"), "{md}");
    assert!(md.contains("file_b.csv:L3"), "second-file duplicate must be cited:\n{md}");
    assert!(md.contains("| Missing receipts flagged | 0 |"), "no receipt col => 0 missing:\n{md}");
    assert!(md.contains("| Amount anomalies flagged | 1 |"), "{md}");
    assert!(md.contains("non-positive amount"), "{md}");
    // 100 + 100 + 100 - 25.50 = 274.50
    assert!(md.contains("$274.50"), "{md}");
}

#[test]
fn clean_dataset_handles_empty_key_duplicate_headers_and_ragged_rows() {
    let env = TestEnv::new("clean-edge");
    env.write_ws(
        "messy.csv",
        "Item Name,Amount,amount,Notes\n\
         Widget A,10.00,1.00,ok\n\
         Widget A,10.00,1.00,ok\n\
         Widget B,12.00\n\
         Widget C,15.00,2.00,N/A,extra-cell\n",
    );

    skills::clean_dataset(&env.cfg, "messy.csv", "", "Messy Clean").unwrap();
    let csv = env.read_ws("Messy Clean.csv");
    let rows = skills::parse_csv_rows(&csv);
    assert_eq!(rows[0], vec!["item_name", "amount", "amount_2", "notes"]);
    assert_eq!(rows.len(), 4, "1 header + 3 unique rows:\n{csv}");

    let report = env.read_ws("Messy Clean - Diff Report.md");
    assert!(report.contains("| Duplicate rows removed | 1 |"), "{report}");
    assert!(report.contains("| Schema-repaired ragged rows | 2 |"), "{report}");
    assert!(report.contains("full normalized row"), "{report}");
}

#[test]
fn extract_tables_supports_tsv_colon_blocks_and_500_line_kv_logs() {
    let env = TestEnv::new("extract-edge");

    // TSV
    env.write_ws("tsv.txt", "Host\tCPU\tRegion\nweb-1\t42%\tus-east\nweb-2\t38%\tus-west\n");
    skills::extract_tables(&env.cfg, "tsv.txt", "Tsv Export").unwrap();
    let tsv_csv = env.read_ws("Tsv Export.csv");
    assert!(tsv_csv.contains("Host,CPU,Region\nweb-1,42%,us-east"), "{tsv_csv}");

    // Colon Key: Value blocks
    env.write_ws(
        "cards.txt",
        "Name: Maya Lin\nTeam: Operations\nStatus: Active\n\nName: Devon Brooks\nTeam: Finance\nStatus: Active\n",
    );
    skills::extract_tables(&env.cfg, "cards.txt", "Cards Export").unwrap();
    let cards_csv = env.read_ws("Cards Export.csv");
    assert!(cards_csv.contains("Maya Lin,Operations,Active"), "{cards_csv}");
    assert!(cards_csv.contains("Devon Brooks,Finance,Active"), "{cards_csv}");

    // 500-line structured KV log
    let mut big_log = String::new();
    for i in 1..=500 {
        big_log.push_str(&format!(
            "2026-11-21T10:{:02}:{:02}Z [INFO] service=worker job_id=JOB-{:04} ms={} status=ok\n",
            (i / 60) % 60,
            i % 60,
            i,
            10 + (i % 50)
        ));
    }
    env.write_ws("big.log", &big_log);
    skills::extract_tables(&env.cfg, "big.log", "Big Log Export").unwrap();
    let log_rows = skills::parse_csv_rows(&env.read_ws("Big Log Export.csv"));
    assert_eq!(log_rows.len(), 501, "header + 500 log rows");

    // Non-tabular prose must fail honestly instead of writing an empty CSV
    env.write_ws("prose.txt", "Just a plain paragraph with no tables or records at all.\n");
    assert!(skills::extract_tables(&env.cfg, "prose.txt", "No Table").is_err());
}

#[test]
fn compare_documents_identical_and_triage_json_and_dispatcher_and_library_seed() {
    let env = TestEnv::new("misc-edge");

    // Identical files -> 100% similarity
    env.write_ws("same-a.md", "# Spec\nLine one.\nLine two.\n");
    env.write_ws("same-b.md", "# Spec\nLine one.\nLine two.\n");
    skills::compare_documents(&env.cfg, "same-a.md", "same-b.md", "Same Diff").unwrap();
    let same_md = env.read_ws("Same Diff.md");
    assert!(same_md.contains("| Line similarity | 100% |"), "{same_md}");
    assert!(same_md.contains("byte-identical"), "{same_md}");

    // Triage JSON array
    env.write_ws(
        "inbox.json",
        r#"[
            {"id":"M1","from":"sre@local","subject":"Production down","body":"Critical outage"},
            {"id":"M2","from":"pm@local","subject":"Review roadmap","body":"Can you review?"},
            {"id":"M3","from":"noreply@local","subject":"Weekly update","body":"FYI: release notes"}
        ]"#,
    );
    skills::triage_inbox(&env.cfg, "inbox.json", "Json Triage").unwrap();
    let jmd = env.read_ws("Json Triage.md");
    assert!(jmd.contains("| **Urgent** | 1 |"), "{jmd}");
    assert!(jmd.contains("| **Action** | 1 |"), "{jmd}");
    assert!(jmd.contains("| **FYI** | 1 |"), "{jmd}");

    // Unified dispatcher + snake_case / kebab-case lookup
    for id in skills::ALL_BUILTIN_SKILL_IDS {
        assert!(skills::is_builtin_skill(id));
        let slug = id.replace('_', "-");
        assert!(skills::is_builtin_skill(&slug));
    }
    assert!(!skills::is_builtin_skill("nonexistent_skill"));
    assert!(skills::dispatch_builtin_skill(&env.cfg, "nonexistent_skill", &json!({})).is_none());

    let dispatched = skills::dispatch_builtin_skill(
        &env.cfg,
        "compare-documents",
        &json!({
            "left_file": "same-a.md",
            "right_file": "same-b.md",
            "output_title": "Dispatched Diff"
        }),
    )
    .unwrap()
    .unwrap();
    assert!(dispatched.contains("Dispatched Diff.md"), "{dispatched}");

    // Seed library contains all 12 built-in skills with inspectable Rune source
    let seeded = ergon_rs::library::seed_items();
    let builtin_skills: Vec<_> = seeded
        .iter()
        .filter(|i| i.kind == "skill" && i.builtin)
        .collect();
    assert_eq!(builtin_skills.len(), 12);
    for bs in builtin_skills {
        assert!(!bs.source.is_empty(), "built-in skill {} missing Rune source", bs.id);
    }
}
