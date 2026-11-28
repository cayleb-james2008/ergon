//! Deterministic `make_spreadsheet` `from_csv` BOM/Quote Proof (Phase B2)
//!
//! Verifies:
//! 1. Deduplicates repeated SKUs from a 50-SKU raw pricing CSV cleanly (first-seen wins).
//! 2. Repairs `N/A` / blank Extended cells via exact integer-cent `Qty * UnitCost`.
//! 3. Appends a penny-exact `TOTAL` row across all unique items.
//! 4. Escapes formula-injection cells (`=`, `+`, `-`, `@`) and blocks traversal paths.
//! 5. Built-in skill count stays strictly 12.

use ergon_rs::skill_catalog::all_skills;
use ergon_rs::skills::dispatch_builtin_skill;
use ergon_rs::Config;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_cfg(label: &str) -> (PathBuf, Config) {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let base = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    let root = base
        .join("test-data")
        .join(format!("bom-csv-{label}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let cfg = Config {
        data_dir: root.join("data"),
        workspace_dir: root.join("workspace"),
        memory_path: root.join("data").join("memory.md"),
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "offline".to_string(),
        api_key: None,
    };
    cfg.ensure_dirs().unwrap();
    (root, cfg)
}

#[test]
fn make_spreadsheet_from_csv_deduplicates_50_skus_with_penny_exact_total() {
    assert_eq!(all_skills().len(), 12);
    let (root, cfg) = temp_cfg("50sku");

    let mut csv = String::from("SKU,Description,Category,Qty,UnitCost,ExtendedRaw\n");
    let mut expected_cents: i128 = 0;
    for i in 1..=50 {
        let qty = ((i % 4) + 1) as i128;
        let unit_cents: i128 = 1250 + (i as i128) * 137;
        let ext_cents = qty * unit_cents;
        expected_cents += ext_cents;
        let ext_cell = if i % 5 == 0 {
            "N/A".to_string()
        } else {
            format!("${:.2}", ext_cents as f64 / 100.0)
        };
        let desc = if i == 1 {
            "=EVIL_FORMULA()"
        } else if i == 2 {
            ""
        } else {
            "Hardware Item"
        };
        let row = format!(
            "SKU-{i:03},{desc},Materials,{qty},${:.2},{ext_cell}\n",
            unit_cents as f64 / 100.0
        );
        csv.push_str(&row);
        if matches!(i, 4 | 19 | 37) {
            // Intentional duplicate lines in raw vendor export
            csv.push_str(&row);
        }
    }
    csv.push_str("TOTAL,,,,,$999999.00\n");

    std::fs::write(cfg.workspace_dir.join("raw_pricing.csv"), csv).unwrap();

    let res = dispatch_builtin_skill(
        &cfg,
        "make_spreadsheet",
        &json!({
            "title": "Clean Quote",
            "from_csv": "raw_pricing.csv",
            "dedup_key": "SKU",
            "append_total": true
        }),
    )
    .expect("make_spreadsheet dispatched")
    .expect("make_spreadsheet succeeded");

    assert!(res.contains("Clean Quote.csv"), "{res}");
    assert!(res.contains("51 data rows"), "{res}"); // 50 SKUs + 1 TOTAL row

    let out = std::fs::read_to_string(cfg.workspace_dir.join("Clean Quote.csv")).unwrap();
    let expected_total = format!("{}.{:02}", expected_cents / 100, expected_cents % 100);
    assert!(
        out.contains(&format!("TOTAL,,,,,{expected_total}")),
        "expected penny-exact TOTAL {expected_total} in:\n{out}"
    );
    assert!(!out.contains("999999.00"), "prior TOTAL row must be ignored");
    assert!(out.contains("'=EVIL_FORMULA()"), "formula injection must be escaped");
    assert!(out.contains("SKU-002,N/A,"), "blank description must fill N/A");
    assert_eq!(out.matches("SKU-019,").count(), 1, "duplicate SKU collapsed");

    // Direct execute_tool routing must also accept from_csv cleanly
    let tool_res = ergon_rs::tools::execute_tool(
        &cfg,
        "make_spreadsheet",
        &json!({
            "title": "Clean Quote Via Tool",
            "from_csv": "raw_pricing.csv",
            "dedup_key": "SKU",
            "append_total": true
        }),
    )
    .expect("execute_tool make_spreadsheet from_csv succeeded");
    assert!(tool_res.contains("Clean Quote Via Tool.csv"), "{tool_res}");
    assert!(cfg.workspace_dir.join("Clean Quote Via Tool.csv").exists());

    // Traversal must be rejected
    let err = dispatch_builtin_skill(
        &cfg,
        "make_spreadsheet",
        &json!({"title": "Bad", "from_csv": "../../etc/passwd"}),
    )
    .unwrap()
    .unwrap_err();
    assert!(!err.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}
