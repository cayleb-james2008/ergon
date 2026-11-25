//! Tamper-evident Field-Book Audit Bundle export & independent verification.
//!
//! Combines the verified SHA-256 Field-Book receipt chain, every workspace file
//! SHA-256 digest and revision count, standing memory digest, recurring routine
//! configuration and run history, and plain verification instructions into one
//! deterministically hashed JSON or Markdown bundle.

use crate::receipts::{self, sha256_hex, ChainedReceipt};
use crate::{file_history, records, routines, scheduler, Config, Person};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const AUDIT_BUNDLE_SCHEMA: &str = "ergon.audit_bundle.v1";

#[derive(Clone, Debug)]
pub struct AuditBundleVerification {
    pub valid: bool,
    pub receipt_chain_valid: bool,
    pub workspace_manifest_valid: bool,
    pub routines_digest_valid: bool,
    pub bundle_hash_valid: bool,
    pub expected_bundle_sha256: String,
    pub actual_bundle_sha256: String,
    pub receipt_count: usize,
    pub workspace_file_count: usize,
    pub errors: Vec<String>,
}

impl AuditBundleVerification {
    pub fn to_json(&self) -> Value {
        json!({
            "valid": self.valid,
            "receipt_chain_valid": self.receipt_chain_valid,
            "workspace_manifest_valid": self.workspace_manifest_valid,
            "routines_digest_valid": self.routines_digest_valid,
            "bundle_hash_valid": self.bundle_hash_valid,
            "expected_bundle_sha256": self.expected_bundle_sha256,
            "actual_bundle_sha256": self.actual_bundle_sha256,
            "receipt_count": self.receipt_count,
            "workspace_file_count": self.workspace_file_count,
            "errors": self.errors,
        })
    }
}

fn format_utc_iso(ms: u64) -> String {
    let (y, m, d) = records::civil_from_millis(ms);
    let secs_in_day = (ms / 1_000) % 86_400;
    let hh = secs_in_day / 3_600;
    let mm = (secs_in_day % 3_600) / 60;
    let ss = secs_in_day % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn collect_workspace_digests(person_data_dir: &Path, workspace_dir: &Path) -> Vec<Value> {
    fn walk(
        person_data_dir: &Path,
        root: &Path,
        dir: &Path,
        depth: usize,
        out: &mut Vec<(String, Value)>,
    ) {
        if depth > 8 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let p = entry.path();
            if ft.is_dir() {
                walk(person_data_dir, root, &p, depth + 1, out);
                continue;
            }
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            let Ok(bytes) = std::fs::read(&p) else {
                continue;
            };
            let mtime_ms = entry
                .metadata()
                .ok()
                .and_then(|md| md.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let rev_count = file_history::revision_count(person_data_dir, root, &rel);
            let digest = sha256_hex(&bytes);
            out.push((
                rel.clone(),
                json!({
                    "path": rel,
                    "size_bytes": bytes.len() as u64,
                    "mtime_ms": mtime_ms,
                    "sha256": digest,
                    "revision_count": rev_count,
                }),
            ));
        }
    }

    let mut rows = Vec::new();
    walk(person_data_dir, workspace_dir, workspace_dir, 0, &mut rows);
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter().map(|(_, v)| v).collect()
}

fn compute_workspace_manifest_sha256(files: &[Value]) -> String {
    let mut canonical = String::new();
    for f in files {
        let p = f["path"].as_str().unwrap_or("");
        let sz = f["size_bytes"].as_u64().unwrap_or(0);
        let h = f["sha256"].as_str().unwrap_or("");
        canonical.push_str(&format!("{p}|{sz}|{h}\n"));
    }
    sha256_hex(canonical.as_bytes())
}

fn compute_receipt_chain_digest(head_hash: &str, length: usize, verified: bool) -> String {
    let flag = if verified { "1" } else { "0" };
    sha256_hex(format!("{head_hash}|{length}|{flag}").as_bytes())
}

struct ShaWriter(Sha256);

impl std::io::Write for ShaWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn sha256_json_writer(f: impl FnOnce(&mut ShaWriter) -> serde_json::Result<()>) -> String {
    let mut w = ShaWriter(Sha256::new());
    let _ = f(&mut w);
    let digest = w.0.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn compute_routines_digest(routines_obj: &Value) -> String {
    sha256_json_writer(|w| serde_json::to_writer(w, routines_obj))
}

struct CanonicalPayload<'a> {
    schema: &'a str,
    app_version: &'a str,
    generated_at_ms: u64,
    person: &'a Value,
    receipt_chain: &'a Value,
    workspace_files: &'a Value,
    standing_memory: &'a Value,
    routines: &'a Value,
}

fn compute_canonical_bundle_sha256(p: &CanonicalPayload<'_>) -> String {
    let schema_val = Value::String(p.schema.to_string());
    let version_val = Value::String(p.app_version.to_string());
    let ts_val = Value::from(p.generated_at_ms);
    let mut map = std::collections::BTreeMap::new();
    map.insert("app_version", &version_val);
    map.insert("generated_at_ms", &ts_val);
    map.insert("person", p.person);
    map.insert("receipt_chain", p.receipt_chain);
    map.insert("routines", p.routines);
    map.insert("schema", &schema_val);
    map.insert("standing_memory", p.standing_memory);
    map.insert("workspace_files", p.workspace_files);
    sha256_json_writer(|w| serde_json::to_writer(w, &map))
}

/// Build a complete tamper-evident Field-Book Audit Bundle for `person` using
/// live workspace files, the verified Field-Book receipt chain, standing
/// memory, and recurring routine history.
pub fn build_audit_bundle(cfg: &Config, person: &Person) -> Value {
    let now_ms = records::now_millis();
    let chain = receipts::load_chain(&cfg.data_dir);
    let chain_len = chain.len();
    let (chain_verified, head_hash) = receipts::verify_chain(&chain);

    let receipt_chain_val = json!({
        "algorithm": "SHA-256",
        "genesis_hash": receipts::GENESIS_HASH,
        "length": chain_len,
        "verified": chain_verified,
        "head_hash": head_hash,
        "entries": chain.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    });
    drop(chain);

    let workspace_files = collect_workspace_digests(&cfg.data_dir, &cfg.workspace_dir);
    let workspace_manifest_sha256 = compute_workspace_manifest_sha256(&workspace_files);
    let workspace_files_val = Value::Array(workspace_files);

    let (mem_present, mem_bytes, mem_sha256, mem_rules) = match std::fs::read(&cfg.memory_path) {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            let rule_count = text
                .lines()
                .filter(|l| l.trim_start().starts_with("- "))
                .count();
            (true, bytes.len() as u64, sha256_hex(&bytes), rule_count)
        }
        Err(_) => (false, 0, sha256_hex(b""), 0),
    };
    let standing_memory_val = json!({
        "present": mem_present,
        "size_bytes": mem_bytes,
        "sha256": mem_sha256,
        "rule_count": mem_rules,
    });

    let routine_items = routines::list_routines(&cfg.data_dir);
    let run_history = scheduler::list_run_history(&cfg.data_dir, None, 200);
    let enriched_routines: Vec<Value> = routine_items
        .iter()
        .map(|r| scheduler::enrich_routine_json(r, now_ms, &run_history))
        .collect();
    let status = scheduler::scheduler_status_json(
        !scheduler::is_scheduler_disabled_by_env(),
        scheduler::DEFAULT_SCHEDULER_TICK_SECS,
        &routine_items,
        &run_history,
        now_ms,
    );
    let routines_val = json!({
        "status": status,
        "items": enriched_routines,
        "run_history": run_history.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    });

    let person_val = person.to_json();
    let receipt_chain_sha256 = compute_receipt_chain_digest(&head_hash, chain_len, chain_verified);
    let routines_sha256 = compute_routines_digest(&routines_val);
    let bundle_sha256 = compute_canonical_bundle_sha256(&CanonicalPayload {
        schema: AUDIT_BUNDLE_SCHEMA,
        app_version: env!("CARGO_PKG_VERSION"),
        generated_at_ms: now_ms,
        person: &person_val,
        receipt_chain: &receipt_chain_val,
        workspace_files: &workspace_files_val,
        standing_memory: &standing_memory_val,
        routines: &routines_val,
    });

    let verification_instructions = vec![
        "1. Verify each workspace file locally with `sha256sum <path>` and compare against `workspace_files[].sha256`.".to_string(),
        "2. Recompute the Field-Book receipt chain from `receipt_chain.genesis_hash` (64 zeroes) over `prev_hash|index|ts|thread_id|finished|answered|task|summary|files` using SHA-256 and confirm the final hash matches `receipt_chain.head_hash`.".to_string(),
        "3. Recompute `integrity.workspace_manifest_sha256` over sorted `<path>|<size_bytes>|<sha256>\\n` lines and confirm it matches.".to_string(),
        "4. Recompute `integrity.bundle_sha256` over the canonical JSON object `{app_version, generated_at_ms, person, receipt_chain, routines, schema, standing_memory, workspace_files}` (alphabetical keys, compact JSON) to confirm zero tampering.".to_string(),
    ];

    json!({
        "schema": AUDIT_BUNDLE_SCHEMA,
        "app_version": env!("CARGO_PKG_VERSION"),
        "generated_at_ms": now_ms,
        "generated_at_iso": format_utc_iso(now_ms),
        "person": person_val,
        "receipt_chain": receipt_chain_val,
        "workspace_files": workspace_files_val,
        "standing_memory": standing_memory_val,
        "routines": routines_val,
        "integrity": {
            "algorithm": "SHA-256",
            "receipt_chain_sha256": receipt_chain_sha256,
            "workspace_manifest_sha256": workspace_manifest_sha256,
            "routines_sha256": routines_sha256,
            "bundle_sha256": bundle_sha256,
        },
        "verification_instructions": verification_instructions,
    })
}

fn chained_receipt_from_json(v: &Value) -> Option<ChainedReceipt> {
    Some(ChainedReceipt {
        id: v["id"].as_str()?.to_string(),
        index: v["index"].as_u64()? as usize,
        ts: v["ts"].as_u64()?,
        task: v["task"].as_str()?.to_string(),
        summary: v["summary"].as_str()?.to_string(),
        files: v["files"]
            .as_array()?
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect(),
        thread_id: v["thread_id"].as_str()?.to_string(),
        finished: v["finished"].as_bool()?,
        answered: v["answered"].as_bool()?,
        prev_hash: v["prev_hash"].as_str()?.to_string(),
        sha256: v["sha256"].as_str()?.to_string(),
    })
}

/// Independently verify a serialized Audit Bundle's receipt chain, workspace
/// manifest digest, routine history digest, and top-level `bundle_sha256`.
pub fn verify_audit_bundle(bundle: &Value) -> AuditBundleVerification {
    let mut errors = Vec::new();
    let schema = bundle["schema"].as_str().unwrap_or("");
    if schema != AUDIT_BUNDLE_SCHEMA {
        errors.push(format!("unexpected bundle schema: '{schema}'"));
    }

    // 1. Independently re-verify every link in the SHA-256 receipt chain.
    let entries_arr = bundle["receipt_chain"]["entries"].as_array();
    let mut parsed_chain = Vec::new();
    let mut parse_ok = true;
    if let Some(arr) = entries_arr {
        for item in arr {
            match chained_receipt_from_json(item) {
                Some(r) => parsed_chain.push(r),
                None => {
                    parse_ok = false;
                    errors.push("malformed receipt entry in receipt_chain.entries".to_string());
                }
            }
        }
    } else {
        parse_ok = false;
        errors.push("missing receipt_chain.entries array".to_string());
    }

    let (chain_ok, computed_head) = receipts::verify_chain(&parsed_chain);
    let claimed_head = bundle["receipt_chain"]["head_hash"].as_str().unwrap_or("");
    let claimed_len = bundle["receipt_chain"]["length"].as_u64().unwrap_or(0) as usize;
    let claimed_verified = bundle["receipt_chain"]["verified"].as_bool().unwrap_or(false);
    let receipt_chain_valid =
        parse_ok && chain_ok && computed_head == claimed_head && parsed_chain.len() == claimed_len && claimed_verified;
    if !receipt_chain_valid {
        errors.push("receipt chain cryptographic verification failed".to_string());
    }

    let expected_rc_digest =
        compute_receipt_chain_digest(&computed_head, parsed_chain.len(), chain_ok);
    let claimed_rc_digest = bundle["integrity"]["receipt_chain_sha256"]
        .as_str()
        .unwrap_or("");
    if expected_rc_digest != claimed_rc_digest {
        errors.push("integrity.receipt_chain_sha256 mismatch".to_string());
    }

    // 2. Re-verify workspace file manifest digest.
    let ws_files = bundle["workspace_files"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let computed_ws_digest = compute_workspace_manifest_sha256(&ws_files);
    let claimed_ws_digest = bundle["integrity"]["workspace_manifest_sha256"]
        .as_str()
        .unwrap_or("");
    let workspace_manifest_valid = computed_ws_digest == claimed_ws_digest;
    if !workspace_manifest_valid {
        errors.push("integrity.workspace_manifest_sha256 mismatch".to_string());
    }

    // 3. Re-verify routine configuration & history digest.
    let computed_rtn_digest = compute_routines_digest(&bundle["routines"]);
    let claimed_rtn_digest = bundle["integrity"]["routines_sha256"]
        .as_str()
        .unwrap_or("");
    let routines_digest_valid = computed_rtn_digest == claimed_rtn_digest;
    if !routines_digest_valid {
        errors.push("integrity.routines_sha256 mismatch".to_string());
    }

    // 4. Re-verify canonical bundle_sha256.
    let expected_bundle_sha256 = compute_canonical_bundle_sha256(&CanonicalPayload {
        schema,
        app_version: bundle["app_version"].as_str().unwrap_or(""),
        generated_at_ms: bundle["generated_at_ms"].as_u64().unwrap_or(0),
        person: &bundle["person"],
        receipt_chain: &bundle["receipt_chain"],
        workspace_files: &bundle["workspace_files"],
        standing_memory: &bundle["standing_memory"],
        routines: &bundle["routines"],
    });
    let actual_bundle_sha256 = bundle["integrity"]["bundle_sha256"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let bundle_hash_valid = expected_bundle_sha256 == actual_bundle_sha256;
    if !bundle_hash_valid {
        errors.push("integrity.bundle_sha256 mismatch — bundle payload was modified".to_string());
    }

    let valid = errors.is_empty()
        && receipt_chain_valid
        && workspace_manifest_valid
        && routines_digest_valid
        && bundle_hash_valid;

    AuditBundleVerification {
        valid,
        receipt_chain_valid,
        workspace_manifest_valid,
        routines_digest_valid,
        bundle_hash_valid,
        expected_bundle_sha256,
        actual_bundle_sha256,
        receipt_count: parsed_chain.len(),
        workspace_file_count: ws_files.len(),
        errors,
    }
}

/// Compare the live files in `workspace_dir` against the SHA-256 digests
/// recorded in an Audit Bundle.
pub fn verify_workspace_against_bundle(bundle: &Value, workspace_dir: &Path) -> Value {
    let mut matched = Vec::new();
    let mut modified = Vec::new();
    let mut missing = Vec::new();

    if let Some(files) = bundle["workspace_files"].as_array() {
        for f in files {
            let rel = f["path"].as_str().unwrap_or("");
            let expected = f["sha256"].as_str().unwrap_or("");
            match crate::sandbox_path_resolving(workspace_dir, rel, true) {
                Ok(full) if full.is_file() => match std::fs::read(&full) {
                    Ok(bytes) => {
                        let actual = sha256_hex(&bytes);
                        if actual == expected {
                            matched.push(rel.to_string());
                        } else {
                            modified.push(json!({
                                "path": rel,
                                "expected_sha256": expected,
                                "actual_sha256": actual,
                            }));
                        }
                    }
                    Err(_) => missing.push(rel.to_string()),
                },
                _ => missing.push(rel.to_string()),
            }
        }
    }

    json!({
        "all_matched": modified.is_empty() && missing.is_empty(),
        "matched_count": matched.len(),
        "modified_count": modified.len(),
        "missing_count": missing.len(),
        "matched": matched,
        "modified": modified,
        "missing": missing,
    })
}

/// Render a human-readable Markdown version of an Audit Bundle suitable for
/// downloading as `ergon-audit-bundle.md`.
pub fn render_audit_bundle_markdown(bundle: &Value) -> String {
    let person_name = bundle["person"]["name"].as_str().unwrap_or("Unknown");
    let person_id = bundle["person"]["id"].as_str().unwrap_or("default");
    let generated_iso = bundle["generated_at_iso"].as_str().unwrap_or("unknown");
    let app_ver = bundle["app_version"].as_str().unwrap_or("unknown");
    let bundle_sha = bundle["integrity"]["bundle_sha256"].as_str().unwrap_or("—");
    let ws_sha = bundle["integrity"]["workspace_manifest_sha256"]
        .as_str()
        .unwrap_or("—");
    let chain_len = bundle["receipt_chain"]["length"].as_u64().unwrap_or(0);
    let chain_ok = bundle["receipt_chain"]["verified"].as_bool().unwrap_or(false);
    let chain_head = bundle["receipt_chain"]["head_hash"].as_str().unwrap_or("—");

    let mut md = String::new();
    md.push_str("# Ergon Field-Book Audit Bundle\n\n");
    md.push_str("## Bundle Overview\n\n");
    md.push_str("| Field | Value |\n| --- | --- |\n");
    md.push_str(&format!("| Schema | `{AUDIT_BUNDLE_SCHEMA}` |\n"));
    md.push_str(&format!("| Ergon Version | `{app_ver}` |\n"));
    md.push_str(&format!("| Generated (UTC) | `{generated_iso}` |\n"));
    md.push_str(&format!("| Employee / Workspace | {person_name} (`{person_id}`) |\n"));
    md.push_str(&format!("| Bundle SHA-256 | `{bundle_sha}` |\n"));
    md.push_str(&format!("| Workspace Manifest SHA-256 | `{ws_sha}` |\n"));
    md.push_str(&format!(
        "| Receipt Chain | {chain_len} entries (verified: `{chain_ok}`) |\n"
    ));
    md.push_str(&format!("| Receipt Chain Head | `{chain_head}` |\n"));

    md.push_str("\n## Verified Field-Book Receipt Chain\n\n");
    let entries = bundle["receipt_chain"]["entries"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if entries.is_empty() {
        md.push_str("_No Field-Book receipts recorded yet._\n");
    } else {
        md.push_str("| # | Receipt ID | Status | Task | SHA-256 |\n| ---: | --- | --- | --- | --- |\n");
        for e in &entries {
            let idx = e["index"].as_u64().unwrap_or(0) + 1;
            let id = e["id"].as_str().unwrap_or("");
            let st = if e["finished"].as_bool().unwrap_or(false) {
                "Finished"
            } else {
                "Incomplete"
            };
            let task = e["task"]
                .as_str()
                .unwrap_or("")
                .replace('|', "\\|")
                .replace('\n', " ");
            let hash = e["sha256"].as_str().unwrap_or("");
            let short_hash: String = hash.chars().take(16).collect();
            md.push_str(&format!("| {idx} | `{id}` | {st} | {task} | `{short_hash}…` |\n"));
        }
    }

    md.push_str("\n## Workspace File Digests\n\n");
    let ws_files = bundle["workspace_files"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if ws_files.is_empty() {
        md.push_str("_Workspace currently contains no files._\n");
    } else {
        md.push_str("| Path | Bytes | Revisions | SHA-256 |\n| --- | ---: | ---: | --- |\n");
        for f in &ws_files {
            let p = f["path"].as_str().unwrap_or("");
            let sz = f["size_bytes"].as_u64().unwrap_or(0);
            let revs = f["revision_count"].as_u64().unwrap_or(0);
            let sha = f["sha256"].as_str().unwrap_or("");
            md.push_str(&format!("| `{p}` | {sz} | {revs} | `{sha}` |\n"));
        }
    }

    md.push_str("\n## Standing Memory Digest\n\n");
    md.push_str(&format!(
        "- Present: `{}` · Bytes: `{}` · Standing rules: `{}` · SHA-256: `{}`\n",
        bundle["standing_memory"]["present"].as_bool().unwrap_or(false),
        bundle["standing_memory"]["size_bytes"].as_u64().unwrap_or(0),
        bundle["standing_memory"]["rule_count"].as_u64().unwrap_or(0),
        bundle["standing_memory"]["sha256"].as_str().unwrap_or("—"),
    ));

    md.push_str("\n## Recurring Routines & Run History\n\n");
    let routine_items = bundle["routines"]["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if routine_items.is_empty() {
        md.push_str("_No recurring routines configured._\n");
    } else {
        md.push_str("| Routine | Schedule | State | Runs |\n| --- | --- | --- | ---: |\n");
        for r in &routine_items {
            let name = r["name"].as_str().unwrap_or("").replace('|', "\\|");
            let sched = r["cadence_label"]
                .as_str()
                .or_else(|| r["schedule"].as_str())
                .unwrap_or("manual");
            let state = if r["enabled"].as_bool().unwrap_or(true) {
                "Active"
            } else {
                "Paused"
            };
            let count = r["run_count"].as_u64().unwrap_or(0);
            md.push_str(&format!("| {name} | `{sched}` | {state} | {count} |\n"));
        }
    }

    let run_history = bundle["routines"]["run_history"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !run_history.is_empty() {
        md.push_str("\n### Recent Routine Executions\n\n");
        md.push_str("| Run ID | Routine | Trigger | Status | Duration (ms) | Receipt |\n| --- | --- | --- | --- | ---: | --- |\n");
        for run in run_history.iter().take(50) {
            let rid = run["run_id"].as_str().unwrap_or("");
            let rname = run["routine_name"].as_str().unwrap_or("").replace('|', "\\|");
            let trig = run["trigger"].as_str().unwrap_or("scheduled");
            let st = run["status"].as_str().unwrap_or("ok");
            let dur = run["duration_ms"].as_u64().unwrap_or(0);
            let rcpt = run["receipt_id"].as_str().unwrap_or("—");
            md.push_str(&format!(
                "| `{rid}` | {rname} | {trig} | {st} | {dur} | `{rcpt}` |\n"
            ));
        }
    }

    md.push_str("\n## Independent Verification Instructions\n\n");
    if let Some(steps) = bundle["verification_instructions"].as_array() {
        for s in steps {
            if let Some(line) = s.as_str() {
                md.push_str(line);
                md.push('\n');
            }
        }
    }

    md
}
