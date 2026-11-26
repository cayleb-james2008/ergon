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
    use std::io::Write as _;
    let mut w = ShaWriter(Sha256::new());
    for f in files {
        let p = f["path"].as_str().unwrap_or("");
        let sz = f["size_bytes"].as_u64().unwrap_or(0);
        let h = f["sha256"].as_str().unwrap_or("");
        let _ = writeln!(&mut w, "{p}|{sz}|{h}");
    }
    let digest = w.0.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
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
        engagement_id: v["engagement_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        usage: v
            .get("usage")
            .filter(|u| !u.is_null())
            .map(receipts::canonical_json),
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

#[derive(Clone, Debug)]
pub struct EvidenceVerification {
    pub valid: bool,
    pub kind: &'static str,
    pub title: String,
    pub receipt_chain_valid: Option<bool>,
    pub bundle_hash_valid: Option<bool>,
    pub manifest_valid: Option<bool>,
    pub citations_valid: Option<bool>,
    pub workspace_valid: Option<bool>,
    pub receipt_count: usize,
    pub deliverable_count: usize,
    pub citation_count: usize,
    pub stale_citations: usize,
    pub broken_citations: usize,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub summary: String,
    pub errors: Vec<String>,
    pub items: Vec<Value>,
}

impl EvidenceVerification {
    pub fn to_json(&self) -> Value {
        json!({
            "valid": self.valid,
            "kind": self.kind,
            "title": self.title,
            "receipt_chain_valid": self.receipt_chain_valid,
            "bundle_hash_valid": self.bundle_hash_valid,
            "manifest_valid": self.manifest_valid,
            "citations_valid": self.citations_valid,
            "workspace_valid": self.workspace_valid,
            "receipt_count": self.receipt_count,
            "deliverable_count": self.deliverable_count,
            "citation_count": self.citation_count,
            "stale_citations": self.stale_citations,
            "broken_citations": self.broken_citations,
            "expected_sha256": self.expected_sha256,
            "actual_sha256": self.actual_sha256,
            "summary": self.summary,
            "errors": self.errors,
            "items": self.items,
        })
    }
}

fn is_hex_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

#[inline(never)]
fn ev_item(label: &str, status: &str, detail: &str, sha256: Option<&str>) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("label".into(), Value::String(label.to_string()));
    obj.insert("status".into(), Value::String(status.to_string()));
    if !detail.is_empty() {
        obj.insert("detail".into(), Value::String(detail.to_string()));
    }
    if let Some(h) = sha256 {
        obj.insert("sha256".into(), Value::String(h.to_string()));
    }
    Value::Object(obj)
}

#[inline(never)]
fn verify_playbook_pack_evidence(v: &Value, fallback_title: &str) -> EvidenceVerification {
    let mut errors = Vec::new();
    let format = v["format"].as_str().unwrap_or("");
    if format != routines::PLAYBOOK_PACK_FORMAT {
        errors.push(format!("Unexpected playbook pack format '{format}'."));
    }
    let pb = &v["playbook"];
    let name = pb["name"].as_str().unwrap_or("").trim();
    let prompt = pb["prompt"].as_str().unwrap_or("").trim();
    let schedule = pb["schedule"].as_str().unwrap_or("manual").trim();
    if name.is_empty() || prompt.is_empty() {
        errors.push("Playbook pack is missing required name or prompt.".to_string());
    }
    let str_vec = |val: &Value| -> Vec<String> {
        val.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let opts = routines::PlaybookOptions {
        steps: str_vec(&pb["steps"]),
        expected_inputs: str_vec(&pb["expected_inputs"]),
        pack_deliverables: pb["pack_deliverables"].as_bool().unwrap_or(false),
    };
    let expected = routines::playbook_canonical_sha256(
        name,
        prompt,
        if schedule.is_empty() { "manual" } else { schedule },
        pb["interval_secs"].as_u64(),
        pb["skill_id"].as_str(),
        &opts,
    );
    let actual = v["sha256"].as_str().unwrap_or("").trim().to_ascii_lowercase();
    let hash_ok = is_hex_sha256(&actual) && actual == expected;
    if !hash_ok {
        errors.push("Playbook SHA-256 digest does not match canonical payload.".to_string());
    }
    let valid = errors.is_empty();
    let title = if name.is_empty() {
        fallback_title.to_string()
    } else {
        name.to_string()
    };
    let summary = if valid {
        format!(
            "Playbook pack passed offline SHA-256 verification ({} steps).",
            opts.steps.len()
        )
    } else {
        format!("Playbook verification failed with {} issue(s).", errors.len())
    };
    EvidenceVerification {
        valid,
        kind: "playbook_pack",
        title,
        receipt_chain_valid: None,
        bundle_hash_valid: Some(hash_ok),
        manifest_valid: Some(format == routines::PLAYBOOK_PACK_FORMAT && !name.is_empty()),
        citations_valid: None,
        workspace_valid: None,
        receipt_count: 0,
        deliverable_count: opts.steps.len(),
        citation_count: 0,
        stale_citations: 0,
        broken_citations: 0,
        expected_sha256: expected,
        actual_sha256: actual,
        summary,
        errors,
        items: vec![ev_item(
            "Playbook canonical checksum",
            if hash_ok { "verified" } else { "tampered" },
            &format!("{} steps", opts.steps.len()),
            None,
        )],
    }
}

#[inline(never)]
fn verify_deliverable_pack_markdown(
    content: &str,
    fallback_title: &str,
    workspace_dir: Option<&Path>,
) -> EvidenceVerification {
    let mut errors = Vec::new();
    let mut items = Vec::new();
    let content_sha = sha256_hex(content.as_bytes());
    let mut title = fallback_title.to_string();
    let mut declared_deliverables: Option<usize> = None;
    let mut manifest: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut cit_rows: Vec<(String, String, String)> = Vec::new();
    let mut citation_alerts = 0usize;
    let mut section = "";

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if title == fallback_title {
            if let Some(t) = line.strip_prefix("# ") {
                let t = t.trim();
                if !t.is_empty() {
                    title = t.to_string();
                }
            }
        }
        if let Some(rest) = line.strip_prefix("- Deliverables included:") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                declared_deliverables = Some(n);
            }
        } else if line.starts_with("## ") {
            if line == "## Included Deliverables" {
                section = "deliverables";
            } else if line == "## Citation Verification" {
                section = "citations";
            } else if line == "## Verified Work Summary" {
                section = "summary";
            } else {
                section = "other";
            }
            continue;
        } else if line == "---" {
            section = "other";
            continue;
        }

        if section == "summary" && line.contains("[CITATION ALERT:") {
            citation_alerts += 1;
        } else if section == "deliverables" {
            if let Some(after_tick) = line.strip_prefix("- `") {
                if let Some((path, rest)) = after_tick.split_once('`') {
                    if let Some((_, sha_tail)) = rest.split_once("`sha256:") {
                        if let Some((sha, _)) = sha_tail.split_once('`') {
                            let sha_lc = sha.trim().to_ascii_lowercase();
                            if is_hex_sha256(&sha_lc) {
                                manifest.insert(path.to_string(), sha_lc);
                            }
                        }
                    }
                }
            }
        } else if section == "citations" {
            if let Some(after_tick) = line.strip_prefix("- `") {
                if let Some((raw_cit, rest)) = after_tick.split_once("` -> **") {
                    if let Some((status, tail)) = rest.split_once("**") {
                        let live_prefix = tail
                            .split_once("`sha256:")
                            .and_then(|(_, s)| s.split_once('`'))
                            .map(|(h, _)| h.trim().to_ascii_lowercase())
                            .unwrap_or_default();
                        cit_rows.push((raw_cit.to_string(), status.trim().to_string(), live_prefix));
                    }
                }
            }
        }
    }

    // Parse embedded deliverable sections: `\n---\n\n## `<path>`\n\n_SHA-256: `<64hex>` · ... · <N> bytes_\n\n<body>`
    let mut embedded_count = 0usize;
    let mut body_hash_failures = 0usize;
    let mut ws_checked = 0usize;
    let mut ws_failures = 0usize;
    let marker = "\n---\n\n## `";
    let mut pos = 0usize;
    while let Some(rel_idx) = content[pos..].find(marker) {
        let head_start = pos + rel_idx + marker.len();
        pos = head_start;
        let Some(path_end) = content[head_start..].find("`\n") else {
            continue;
        };
        let sec_path = &content[head_start..head_start + path_end];
        let after_path = &content[head_start + path_end + 2..];
        let body_start_rel = match after_path.strip_prefix("\n_SHA-256: `") {
            Some(_) => 1usize,
            None if after_path.starts_with("_SHA-256: `") => 0usize,
            _ => continue,
        };
        let meta_line_start = &after_path[body_start_rel + "_SHA-256: `".len()..];
        let Some((sec_sha_raw, meta_rest)) = meta_line_start.split_once('`') else {
            continue;
        };
        let sec_sha = sec_sha_raw.trim().to_ascii_lowercase();
        let Some(meta_nl) = meta_rest.find('\n') else {
            continue;
        };
        let meta_tail = meta_rest[..meta_nl].trim().trim_end_matches('_');
        let declared_bytes = meta_tail
            .rsplit('·')
            .next()
            .and_then(|s| s.trim().strip_suffix("bytes"))
            .and_then(|s| s.trim().parse::<usize>().ok());
        let body_rest = &meta_rest[meta_nl + 1..];
        let body_rest = body_rest.strip_prefix('\n').unwrap_or(body_rest);
        let next_sep = body_rest
            .find("\n---\n\n## ")
            .unwrap_or(body_rest.len());
        let raw_body = &body_rest[..next_sep];
        let trimmed_body = raw_body.strip_suffix('\n').unwrap_or(raw_body);
        embedded_count += 1;

        let mut sec_ok = is_hex_sha256(&sec_sha);
        if !sec_ok {
            errors.push(format!("Deliverable '{sec_path}' has an invalid SHA-256 digest."));
        }
        if let Some(m_sha) = manifest.get(sec_path) {
            if m_sha != &sec_sha {
                sec_ok = false;
                errors.push(format!(
                    "Deliverable '{sec_path}' section digest differs from manifest digest."
                ));
            }
        }
        let mut hash_checked = false;
        if let Some(expected_len) = declared_bytes {
            let candidate = if trimmed_body.len() == expected_len {
                Some(trimmed_body)
            } else if raw_body.len() == expected_len {
                Some(raw_body)
            } else {
                None
            };
            if let Some(text_bytes) = candidate {
                hash_checked = true;
                let actual_body_sha = sha256_hex(text_bytes.as_bytes());
                if actual_body_sha != sec_sha {
                    sec_ok = false;
                    body_hash_failures += 1;
                    errors.push(format!(
                        "Deliverable '{sec_path}' content was modified after export."
                    ));
                }
            }
        }
        if let Some(ws) = workspace_dir {
            ws_checked += 1;
            match crate::sandbox_path_resolving(ws, sec_path, true)
                .ok()
                .and_then(|p| std::fs::read(p).ok())
            {
                Some(disk_bytes) => {
                    let disk_sha = sha256_hex(&disk_bytes);
                    if disk_sha != sec_sha {
                        sec_ok = false;
                        ws_failures += 1;
                        errors.push(format!(
                            "Workspace file '{sec_path}' changed on disk since pack export."
                        ));
                    }
                }
                None => {
                    sec_ok = false;
                    ws_failures += 1;
                    errors.push(format!(
                        "Workspace file '{sec_path}' is missing on disk."
                    ));
                }
            }
        }
        items.push(ev_item(
            sec_path,
            if sec_ok { "verified" } else { "tampered" },
            if hash_checked { "SHA-256 body matched" } else { "Manifest SHA-256 checked" },
            Some(&sec_sha),
        ));
    }

    if manifest.is_empty() && embedded_count == 0 {
        errors.push("Markdown file does not contain Ergon deliverable digests.".to_string());
    }
    if let Some(decl) = declared_deliverables {
        let count = manifest.len().max(embedded_count);
        if decl != count {
            errors.push(format!(
                "Declared {decl} deliverable(s), found {count} in manifest."
            ));
        }
    }

    let mut stale_cits = 0usize;
    let mut broken_cits = 0usize;
    for (raw_cit, status, live_prefix) in &cit_rows {
        let st_low = status.to_ascii_lowercase();
        if st_low == "stale" {
            stale_cits += 1;
        } else if st_low == "broken" {
            broken_cits += 1;
        }
        if !live_prefix.is_empty() {
            let cit_file = raw_cit
                .trim_matches(|c| c == '[' || c == ']')
                .rsplit_once(":L")
                .map(|(f, _)| f)
                .unwrap_or("");
            if let Some(m_sha) = manifest.get(cit_file) {
                if !m_sha.starts_with(live_prefix) {
                    errors.push(format!(
                        "Citation '{raw_cit}' live digest prefix does not match deliverable manifest."
                    ));
                }
            }
        }
    }
    if stale_cits > 0 || broken_cits > 0 || citation_alerts > 0 {
        errors.push(format!(
            "Pack contains {stale_cits} stale citation(s) and {broken_cits} broken citation(s)."
        ));
    }

    let manifest_ok = !manifest.is_empty() && body_hash_failures == 0;
    let citations_ok = stale_cits == 0 && broken_cits == 0 && citation_alerts == 0;
    let ws_ok_opt = if ws_checked > 0 {
        Some(ws_failures == 0)
    } else {
        None
    };
    let deliverable_count = manifest.len().max(embedded_count);
    let valid = errors.is_empty() && manifest_ok && citations_ok && ws_ok_opt.unwrap_or(true);
    let summary = if valid {
        format!(
            "Client pack passed offline verification ({deliverable_count} deliverables, {} citations).",
            cit_rows.len()
        )
    } else {
        format!("Pack verification failed with {} issue(s).", errors.len())
    };
    EvidenceVerification {
        valid,
        kind: "deliverable_pack",
        title,
        receipt_chain_valid: None,
        bundle_hash_valid: Some(body_hash_failures == 0 && errors.is_empty()),
        manifest_valid: Some(manifest_ok),
        citations_valid: Some(citations_ok),
        workspace_valid: ws_ok_opt,
        receipt_count: 0,
        deliverable_count,
        citation_count: cit_rows.len(),
        stale_citations: stale_cits,
        broken_citations: broken_cits,
        expected_sha256: content_sha.clone(),
        actual_sha256: content_sha,
        summary,
        errors,
        items,
    }
}

/// Independently verify an Ergon Audit Bundle (`.json`), Signed Playbook Pack
/// (`.ergon-playbook.json`), or Client Deliverable Pack (`.md`) completely
/// offline without a running server or model endpoint.
pub fn verify_evidence(
    content: &str,
    filename: Option<&str>,
    workspace_dir: Option<&Path>,
) -> EvidenceVerification {
    let trimmed = content.trim_start_matches('\u{feff}').trim();
    let fallback_title = filename
        .and_then(|p| Path::new(p).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("Evidence artifact");
    if trimmed.is_empty() {
        return EvidenceVerification {
            valid: false,
            kind: "unknown",
            title: fallback_title.to_string(),
            receipt_chain_valid: None,
            bundle_hash_valid: None,
            manifest_valid: None,
            citations_valid: None,
            workspace_valid: None,
            receipt_count: 0,
            deliverable_count: 0,
            citation_count: 0,
            stale_citations: 0,
            broken_citations: 0,
            expected_sha256: String::new(),
            actual_sha256: String::new(),
            summary: "Evidence input is empty.".to_string(),
            errors: vec!["Evidence input is empty.".to_string()],
            items: Vec::new(),
        };
    }

    if trimmed.starts_with('{') {
        match serde_json::from_str::<Value>(trimmed) {
            Ok(v) => {
                if v.get("schema").and_then(Value::as_str) == Some(AUDIT_BUNDLE_SCHEMA)
                    || (v.get("receipt_chain").is_some() && v.get("integrity").is_some())
                {
                    let av = verify_audit_bundle(&v);
                    let mut errors = av.errors.clone();
                    let mut items = vec![
                        ev_item("Receipt chain", if av.receipt_chain_valid { "verified" } else { "tampered" }, &format!("{} receipts", av.receipt_count), None),
                        ev_item("Workspace manifest", if av.workspace_manifest_valid { "verified" } else { "tampered" }, &format!("{} files", av.workspace_file_count), None),
                        ev_item("Routines digest", if av.routines_digest_valid { "verified" } else { "tampered" }, "SHA-256", None),
                        ev_item("Bundle SHA-256", if av.bundle_hash_valid { "verified" } else { "tampered" }, &av.expected_bundle_sha256[..12.min(av.expected_bundle_sha256.len())], None),
                    ];
                    let ws_ok_opt = workspace_dir.map(|ws| {
                        let wv = verify_workspace_against_bundle(&v, ws);
                        let matched = wv["all_matched"].as_bool().unwrap_or(false);
                        let mod_c = wv["modified_count"].as_u64().unwrap_or(0);
                        let miss_c = wv["missing_count"].as_u64().unwrap_or(0);
                        if !matched {
                            errors.push(format!(
                                "Workspace mismatch: {mod_c} modified file(s), {miss_c} missing file(s)."
                            ));
                        }
                        items.push(ev_item(
                            "Workspace files on disk",
                            if matched { "verified" } else { "tampered" },
                            &format!("{} matched, {mod_c} modified, {miss_c} missing", wv["matched_count"].as_u64().unwrap_or(0)),
                            None,
                        ));
                        matched
                    });
                    let valid = av.valid && ws_ok_opt.unwrap_or(true);
                    let person_name = v["person"]["name"].as_str().unwrap_or("");
                    let title = if person_name.is_empty() {
                        fallback_title.to_string()
                    } else {
                        format!("{person_name} audit bundle")
                    };
                    let summary = if valid {
                        format!(
                            "Audit bundle passed offline SHA-256 verification ({} receipts, {} files).",
                            av.receipt_count, av.workspace_file_count
                        )
                    } else {
                        format!("Audit bundle verification failed with {} issue(s).", errors.len())
                    };
                    return EvidenceVerification {
                        valid,
                        kind: "audit_bundle",
                        title,
                        receipt_chain_valid: Some(av.receipt_chain_valid),
                        bundle_hash_valid: Some(av.bundle_hash_valid),
                        manifest_valid: Some(av.workspace_manifest_valid && av.routines_digest_valid),
                        citations_valid: None,
                        workspace_valid: ws_ok_opt,
                        receipt_count: av.receipt_count,
                        deliverable_count: av.workspace_file_count,
                        citation_count: 0,
                        stale_citations: 0,
                        broken_citations: 0,
                        expected_sha256: av.expected_bundle_sha256,
                        actual_sha256: av.actual_bundle_sha256,
                        summary,
                        errors,
                        items,
                    };
                }
                if v.get("format").and_then(Value::as_str) == Some(routines::PLAYBOOK_PACK_FORMAT)
                    || (v.get("playbook").is_some() && v.get("sha256").is_some())
                {
                    return verify_playbook_pack_evidence(&v, fallback_title);
                }
                return EvidenceVerification {
                    valid: false,
                    kind: "unknown",
                    title: fallback_title.to_string(),
                    receipt_chain_valid: None,
                    bundle_hash_valid: None,
                    manifest_valid: None,
                    citations_valid: None,
                    workspace_valid: None,
                    receipt_count: 0,
                    deliverable_count: 0,
                    citation_count: 0,
                    stale_citations: 0,
                    broken_citations: 0,
                    expected_sha256: String::new(),
                    actual_sha256: sha256_hex(content.as_bytes()),
                    summary: "Unrecognized JSON evidence format.".to_string(),
                    errors: vec!["JSON document is neither an Ergon audit bundle nor a signed playbook pack.".to_string()],
                    items: Vec::new(),
                };
            }
            Err(e) => {
                return EvidenceVerification {
                    valid: false,
                    kind: "unknown",
                    title: fallback_title.to_string(),
                    receipt_chain_valid: None,
                    bundle_hash_valid: Some(false),
                    manifest_valid: None,
                    citations_valid: None,
                    workspace_valid: None,
                    receipt_count: 0,
                    deliverable_count: 0,
                    citation_count: 0,
                    stale_citations: 0,
                    broken_citations: 0,
                    expected_sha256: String::new(),
                    actual_sha256: sha256_hex(content.as_bytes()),
                    summary: "Malformed JSON evidence input.".to_string(),
                    errors: vec![format!("Invalid JSON syntax: {e}")],
                    items: Vec::new(),
                };
            }
        }
    }

    if trimmed.contains("## Included Deliverables")
        || trimmed.contains("_SHA-256: `")
        || trimmed.contains("## Verified Work Summary")
    {
        return verify_deliverable_pack_markdown(content, fallback_title, workspace_dir);
    }

    EvidenceVerification {
        valid: false,
        kind: "unknown",
        title: fallback_title.to_string(),
        receipt_chain_valid: None,
        bundle_hash_valid: None,
        manifest_valid: None,
        citations_valid: None,
        workspace_valid: None,
        receipt_count: 0,
        deliverable_count: 0,
        citation_count: 0,
        stale_citations: 0,
        broken_citations: 0,
        expected_sha256: String::new(),
        actual_sha256: sha256_hex(content.as_bytes()),
        summary: "Unsupported evidence format.".to_string(),
        errors: vec!["Provide an audit bundle (.json), client deliverable pack (.md), or signed playbook (.ergon-playbook.json).".to_string()],
        items: Vec::new(),
    }
}

/// Render a human-readable Markdown version of an Audit Bundle suitable for
/// downloading as `ergon-audit-bundle.md`.
#[inline(never)]
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

fn write_chained_receipt_json(w: &mut impl std::io::Write, r: &ChainedReceipt) -> std::io::Result<()> {
    // Alphabetical key order matching serde_json::Value(BTreeMap) for canonical bundle SHA-256 parity.
    w.write_all(b"{\"answered\":")?;
    serde_json::to_writer(&mut *w, &r.answered).map_err(std::io::Error::other)?;
    if let Some(eid) = r.engagement_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        w.write_all(b",\"engagement_id\":")?;
        serde_json::to_writer(&mut *w, eid).map_err(std::io::Error::other)?;
    }
    w.write_all(b",\"files\":")?;
    serde_json::to_writer(&mut *w, &r.files).map_err(std::io::Error::other)?;
    w.write_all(b",\"finished\":")?;
    serde_json::to_writer(&mut *w, &r.finished).map_err(std::io::Error::other)?;
    w.write_all(b",\"id\":")?;
    serde_json::to_writer(&mut *w, &r.id).map_err(std::io::Error::other)?;
    w.write_all(b",\"index\":")?;
    serde_json::to_writer(&mut *w, &r.index).map_err(std::io::Error::other)?;
    w.write_all(b",\"prev_hash\":")?;
    serde_json::to_writer(&mut *w, &r.prev_hash).map_err(std::io::Error::other)?;
    w.write_all(b",\"sha256\":")?;
    serde_json::to_writer(&mut *w, &r.sha256).map_err(std::io::Error::other)?;
    w.write_all(b",\"summary\":")?;
    serde_json::to_writer(&mut *w, &r.summary).map_err(std::io::Error::other)?;
    w.write_all(b",\"task\":")?;
    serde_json::to_writer(&mut *w, &r.task).map_err(std::io::Error::other)?;
    w.write_all(b",\"thread_id\":")?;
    serde_json::to_writer(&mut *w, &r.thread_id).map_err(std::io::Error::other)?;
    w.write_all(b",\"ts\":")?;
    serde_json::to_writer(&mut *w, &r.ts).map_err(std::io::Error::other)?;
    if let Some(u) = r.usage.as_ref().filter(|u| !u.is_null()) {
        let can = receipts::canonical_json(u);
        w.write_all(b",\"usage\":")?;
        serde_json::to_writer(&mut *w, &can).map_err(std::io::Error::other)?;
    }
    w.write_all(b"}")
}

fn write_receipt_chain_json(
    w: &mut impl std::io::Write,
    chain: &[ChainedReceipt],
    verified: bool,
    head_hash: &str,
) -> std::io::Result<()> {
    w.write_all(b"{\"algorithm\":\"SHA-256\",\"entries\":[")?;
    for (i, r) in chain.iter().enumerate() {
        if i > 0 {
            w.write_all(b",")?;
        }
        write_chained_receipt_json(&mut *w, r)?;
    }
    w.write_all(b"],\"genesis_hash\":")?;
    serde_json::to_writer(&mut *w, receipts::GENESIS_HASH).map_err(std::io::Error::other)?;
    w.write_all(b",\"head_hash\":")?;
    serde_json::to_writer(&mut *w, head_hash).map_err(std::io::Error::other)?;
    w.write_all(b",\"length\":")?;
    serde_json::to_writer(&mut *w, &chain.len()).map_err(std::io::Error::other)?;
    w.write_all(b",\"verified\":")?;
    serde_json::to_writer(&mut *w, &verified).map_err(std::io::Error::other)?;
    w.write_all(b"}")
}

/// Stream a complete tamper-evident JSON Audit Bundle directly into `Vec<u8>`,
/// optionally borrowing an already-loaded `ChainedReceipt` slice so warm
/// `/api/audit/bundle` calls never allocate a duplicate 5k-receipt vector.
pub fn build_audit_bundle_json_bytes_with_chain(
    cfg: &Config,
    person: &Person,
    security: &Value,
    cached: Option<(&[ChainedReceipt], bool, &str)>,
) -> Vec<u8> {
    let now_ms = records::now_millis();
    let owned_chain;
    let (chain, chain_verified, head_hash): (&[ChainedReceipt], bool, String) = match cached {
        Some((c, v, h)) => (c, v, h.to_string()),
        None => {
            owned_chain = receipts::load_chain(&cfg.data_dir);
            let (v, h) = receipts::verify_chain(&owned_chain);
            (&owned_chain, v, h)
        }
    };
    let chain_len = chain.len();

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

    // Compute canonical bundle SHA-256 directly via ShaWriter (alphabetical top-level keys).
    let mut sha = ShaWriter(Sha256::new());
    let _ = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        sha.write_all(b"{\"app_version\":")?;
        serde_json::to_writer(&mut sha, env!("CARGO_PKG_VERSION")).map_err(std::io::Error::other)?;
        sha.write_all(b",\"generated_at_ms\":")?;
        serde_json::to_writer(&mut sha, &now_ms).map_err(std::io::Error::other)?;
        sha.write_all(b",\"person\":")?;
        serde_json::to_writer(&mut sha, &person_val).map_err(std::io::Error::other)?;
        sha.write_all(b",\"receipt_chain\":")?;
        write_receipt_chain_json(&mut sha, chain, chain_verified, &head_hash)?;
        sha.write_all(b",\"routines\":")?;
        serde_json::to_writer(&mut sha, &routines_val).map_err(std::io::Error::other)?;
        sha.write_all(b",\"schema\":")?;
        serde_json::to_writer(&mut sha, AUDIT_BUNDLE_SCHEMA).map_err(std::io::Error::other)?;
        sha.write_all(b",\"standing_memory\":")?;
        serde_json::to_writer(&mut sha, &standing_memory_val).map_err(std::io::Error::other)?;
        sha.write_all(b",\"workspace_files\":")?;
        serde_json::to_writer(&mut sha, &workspace_files_val).map_err(std::io::Error::other)?;
        sha.write_all(b"}")
    })();
    let digest = sha.0.finalize();
    let mut bundle_sha256 = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut bundle_sha256, "{b:02x}");
    }

    let verification_instructions = vec![
        "1. Verify each workspace file locally with `sha256sum <path>` and compare against `workspace_files[].sha256`.",
        "2. Recompute the Field-Book receipt chain from `receipt_chain.genesis_hash` (64 zeroes) over `prev_hash|index|ts|thread_id|finished|answered|task|summary|files` using SHA-256 and confirm the final hash matches `receipt_chain.head_hash`.",
        "3. Recompute `integrity.workspace_manifest_sha256` over sorted `<path>|<size_bytes>|<sha256>\\n` lines and confirm it matches.",
        "4. Recompute `integrity.bundle_sha256` over the canonical JSON object `{app_version, generated_at_ms, person, receipt_chain, routines, schema, standing_memory, workspace_files}` (alphabetical keys, compact JSON) to confirm zero tampering.",
    ];

    // Reserve conservatively so 5k-receipt bundles never double-reallocate
    // mid-stream above the mmap threshold.
    let mut out = Vec::with_capacity(
        49_152 + chain.len() * 392 + workspace_files_val.as_array().map_or(0, |a| a.len() * 192),
    );
    let _ = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        out.write_all(b"{\"schema\":")?;
        serde_json::to_writer(&mut out, AUDIT_BUNDLE_SCHEMA).map_err(std::io::Error::other)?;
        out.write_all(b",\"app_version\":")?;
        serde_json::to_writer(&mut out, env!("CARGO_PKG_VERSION")).map_err(std::io::Error::other)?;
        out.write_all(b",\"generated_at_ms\":")?;
        serde_json::to_writer(&mut out, &now_ms).map_err(std::io::Error::other)?;
        out.write_all(b",\"generated_at_iso\":")?;
        serde_json::to_writer(&mut out, &format_utc_iso(now_ms)).map_err(std::io::Error::other)?;
        out.write_all(b",\"person\":")?;
        serde_json::to_writer(&mut out, &person_val).map_err(std::io::Error::other)?;
        out.write_all(b",\"receipt_chain\":")?;
        write_receipt_chain_json(&mut out, chain, chain_verified, &head_hash)?;
        out.write_all(b",\"workspace_files\":")?;
        serde_json::to_writer(&mut out, &workspace_files_val).map_err(std::io::Error::other)?;
        out.write_all(b",\"standing_memory\":")?;
        serde_json::to_writer(&mut out, &standing_memory_val).map_err(std::io::Error::other)?;
        out.write_all(b",\"routines\":")?;
        serde_json::to_writer(&mut out, &routines_val).map_err(std::io::Error::other)?;
        out.write_all(b",\"integrity\":")?;
        let integrity = json!({
            "algorithm": "SHA-256",
            "receipt_chain_sha256": receipt_chain_sha256,
            "workspace_manifest_sha256": workspace_manifest_sha256,
            "routines_sha256": routines_sha256,
            "bundle_sha256": bundle_sha256,
            "security_events_sha256": security["events_sha256"],
        });
        serde_json::to_writer(&mut out, &integrity).map_err(std::io::Error::other)?;
        out.write_all(b",\"security\":")?;
        serde_json::to_writer(&mut out, security).map_err(std::io::Error::other)?;
        out.write_all(b",\"verification_instructions\":")?;
        serde_json::to_writer(&mut out, &verification_instructions).map_err(std::io::Error::other)?;
        out.write_all(b"}")
    })();
    out
}

pub fn build_audit_bundle_json_bytes(cfg: &Config, person: &Person, security: &Value) -> Vec<u8> {
    build_audit_bundle_json_bytes_with_chain(cfg, person, security, None)
}

#[derive(Clone, Debug)]
pub struct HandoffLeakFinding {
    pub kind: &'static str,
    pub severity: &'static str,
    pub file: String,
    pub line: usize,
    pub excerpt: String,
    pub fix: String,
}

impl HandoffLeakFinding {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "severity": self.severity,
            "file": self.file,
            "line": self.line,
            "excerpt": self.excerpt,
            "fix": self.fix,
        })
    }
}

#[derive(Clone, Debug)]
pub struct HandoffLeakScan {
    pub clean: bool,
    pub leak_count: usize,
    pub host_path_count: usize,
    pub secret_count: usize,
    pub env_snippet_count: usize,
    pub pii_count: usize,
    pub findings: Vec<HandoffLeakFinding>,
}

impl HandoffLeakScan {
    pub fn to_json(&self) -> Value {
        json!({
            "clean": self.clean,
            "leak_count": self.leak_count,
            "host_path_count": self.host_path_count,
            "secret_count": self.secret_count,
            "env_snippet_count": self.env_snippet_count,
            "pii_count": self.pii_count,
            "findings": self.findings.iter().map(HandoffLeakFinding::to_json).collect::<Vec<_>>(),
        })
    }
}

pub struct HandoffExportResult {
    pub zip_bytes: Vec<u8>,
    pub sha256: String,
    pub summary_md: String,
    pub audit_bundle_bytes: Vec<u8>,
    pub verify_txt: String,
    pub sha256sums: String,
    pub scan: HandoffLeakScan,
}

pub struct HandoffExportBlocked {
    pub scan: HandoffLeakScan,
    pub policy: crate::WorkspacePolicy,
}

impl HandoffExportBlocked {
    pub fn to_json(&self) -> Value {
        json!({
            "error": format!(
                "Blocked by {} policy: {} leak/path finding(s) must be resolved before client handoff.",
                self.policy.label, self.scan.leak_count
            ),
            "blocked": true,
            "policy": self.policy.profile,
            "scan": self.scan.to_json(),
        })
    }
}

fn contains_host_path(line: &str) -> bool {
    line.contains("/home/")
        || line.contains("/Users/")
        || line.contains("/tmp/")
        || line.contains("/var/folders/")
        || line.contains("C:\\Users\\")
        || line.contains("C:/Users/")
}

fn token_boundary(prev: Option<u8>) -> bool {
    prev.map(|b| !b.is_ascii_alphanumeric() && b != b'_' && b != b'-')
        .unwrap_or(true)
}

fn detect_secret_prefix(line: &str) -> Option<&'static str> {
    if line.contains("-----BEGIN ") && line.contains("PRIVATE KEY-----") {
        return Some("private key block");
    }
    let bytes = line.as_bytes();
    let prefixes: [(&[u8], usize, &'static str); 10] = [
        (b"sk-ergon-", 6, "bait API key"),
        (b"sk-ant-", 10, "Anthropic API key"),
        (b"sk-proj-", 10, "OpenAI API key"),
        (b"sk-", 16, "API secret key"),
        (b"ghp_", 16, "GitHub token"),
        (b"gho_", 16, "GitHub OAuth token"),
        (b"github_pat_", 16, "GitHub PAT"),
        (b"AKIA", 16, "AWS access key"),
        (b"AIza", 20, "Google API key"),
        (b"xoxb-", 10, "Slack bot token"),
    ];
    for (i, _) in bytes.iter().enumerate() {
        let prev = if i > 0 { Some(bytes[i - 1]) } else { None };
        if !token_boundary(prev) {
            continue;
        }
        let tail = &bytes[i..];
        for (pfx, min_rest, label) in prefixes {
            if tail.starts_with(pfx) {
                let rest = &tail[pfx.len()..];
                let ok_len = rest
                    .iter()
                    .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_' || **b == b'-')
                    .count();
                if ok_len >= min_rest {
                    return Some(label);
                }
            }
        }
    }
    None
}

fn detect_env_assignment(line: &str) -> Option<String> {
    let trimmed = line.trim().strip_prefix("export ").unwrap_or(line.trim());
    let (lhs, rhs) = trimmed.split_once('=')?;
    let key = lhs.trim();
    if key.len() < 3
        || !key
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
    {
        return None;
    }
    let sensitive_key = key.ends_with("_API_KEY")
        || key.ends_with("_SECRET")
        || key.ends_with("_SECRET_KEY")
        || key.ends_with("_TOKEN")
        || key.ends_with("_PASSWORD")
        || key == "API_KEY"
        || key == "SECRET_KEY"
        || key == "AUTH_TOKEN"
        || key == "PRIVATE_KEY"
        || key == "DATABASE_URL"
        || key == "PASSWORD";
    if !sensitive_key {
        return None;
    }
    let val = rhs.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    if val.is_empty() || val.starts_with('<') || val.eq_ignore_ascii_case("changeme") || val.eq_ignore_ascii_case("example") {
        return None;
    }
    Some(key.to_string())
}

fn detect_ssn(line: &str) -> bool {
    let b = line.as_bytes();
    if b.len() < 11 {
        return false;
    }
    for i in 0..=b.len() - 11 {
        let prev_ok = i == 0 || !b[i - 1].is_ascii_digit();
        let next_ok = i + 11 == b.len() || !b[i + 11].is_ascii_digit();
        if !prev_ok || !next_ok {
            continue;
        }
        if b[i + 3] == b'-'
            && b[i + 6] == b'-'
            && b[i..i + 3].iter().all(u8::is_ascii_digit)
            && b[i + 4..i + 6].iter().all(u8::is_ascii_digit)
            && b[i + 7..i + 11].iter().all(u8::is_ascii_digit)
        {
            let area = &b[i..i + 3];
            let group = &b[i + 4..i + 6];
            let serial = &b[i + 7..i + 11];
            if area != b"000" && area != b"666" && area[0] != b'9' && group != b"00" && serial != b"0000" {
                return true;
            }
        }
    }
    false
}

fn scan_text_into(
    rel: &str,
    text: &str,
    configured_secret: Option<&str>,
    scan: &mut HandoffLeakScan,
) {
    let is_env_file = Path::new(rel)
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|n| n == ".env" || n.starts_with(".env."));
    let mut push_finding = |kind: &'static str, severity: &'static str, line: usize, excerpt: String, fix: &str| {
        scan.leak_count += 1;
        match kind {
            "host_path" => scan.host_path_count += 1,
            "api_key" => scan.secret_count += 1,
            "env_snippet" => scan.env_snippet_count += 1,
            "pii" => scan.pii_count += 1,
            _ => {}
        }
        if scan.findings.len() < 50 {
            scan.findings.push(HandoffLeakFinding {
                kind,
                severity,
                file: rel.to_string(),
                line,
                excerpt,
                fix: fix.to_string(),
            });
        }
    };

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        if contains_host_path(line) {
            let safe_preview = crate::redact_host_paths(line);
            push_finding(
                "host_path",
                "high",
                line_no,
                crate::truncate(safe_preview.trim(), 120),
                &format!("Replace local host path at {rel}:L{line_no} with a workspace-relative path or enable Strict Client redaction."),
            );
        }
        let has_cfg_secret = configured_secret
            .is_some_and(|s| !s.is_empty() && crate::tripwire::contains_secret(line, s));
        if let Some(label) = detect_secret_prefix(line).or(if has_cfg_secret { Some("configured API key") } else { None }) {
            push_finding(
                "api_key",
                "high",
                line_no,
                format!("Detected {label} at {rel}:L{line_no} (redacted)."),
                &format!("Remove {label} from {rel}:L{line_no} before sharing client artifacts."),
            );
        } else if let Some(key_name) = detect_env_assignment(line) {
            push_finding(
                "env_snippet",
                "high",
                line_no,
                format!("{key_name}=****"),
                &format!("Remove .env credential assignment {key_name} at {rel}:L{line_no} before handoff."),
            );
        } else if is_env_file && line.contains('=') && !line.trim_start().starts_with('#') {
            let k = line.split('=').next().unwrap_or("").trim();
            push_finding(
                "env_snippet",
                "high",
                line_no,
                format!("{k}=****"),
                &format!("Exclude {rel} from the workspace before exporting a client handoff."),
            );
        }
        if detect_ssn(line) {
            push_finding(
                "pii",
                "medium",
                line_no,
                format!("Potential government ID number at {rel}:L{line_no} (redacted)."),
                &format!("Mask or remove the identification number at {rel}:L{line_no} before sharing."),
            );
        }
    }
}

/// Deterministic secret, credential, PII, and host-path leak scanner across
/// workspace deliverables and handoff texts.
#[cold]
#[inline(never)]
pub fn scan_handoff_leaks(
    cfg: &Config,
    extra_texts: &[(&str, &str)],
    configured_secret: Option<&str>,
) -> HandoffLeakScan {
    fn walk_ws(dir: &Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
        if depth > 8 || out.len() >= 500 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == ".history" || name == ".git" || (name.starts_with('.') && !name.starts_with(".env")) {
                continue;
            }
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let p = entry.path();
            if ft.is_dir() {
                walk_ws(&p, depth + 1, out);
            } else {
                out.push(p);
            }
        }
    }

    let mut scan = HandoffLeakScan {
        clean: true,
        leak_count: 0,
        host_path_count: 0,
        secret_count: 0,
        env_snippet_count: 0,
        pii_count: 0,
        findings: Vec::new(),
    };
    let mut paths = Vec::new();
    walk_ws(&cfg.workspace_dir, 0, &mut paths);
    paths.sort();
    let mut read_buf = Vec::with_capacity(8192);
    for p in paths {
        let rel = p
            .strip_prefix(&cfg.workspace_dir)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        if let Ok(meta) = std::fs::metadata(&p) {
            if meta.len() > 1024 * 1024 {
                continue;
            }
        }
        read_buf.clear();
        if let Ok(mut f) = std::fs::File::open(&p) {
            use std::io::Read as _;
            if f.by_ref().take(1024 * 1024 + 1).read_to_end(&mut read_buf).is_ok()
                && read_buf.len() <= 1024 * 1024
                && !read_buf.contains(&0)
            {
                let text = String::from_utf8_lossy(&read_buf);
                scan_text_into(&rel, &text, configured_secret, &mut scan);
            }
        }
    }
    drop(read_buf);
    for (name, text) in extra_texts {
        scan_text_into(name, text, configured_secret, &mut scan);
    }
    scan.clean = scan.leak_count == 0;
    scan
}

/// Build a self-contained client handoff zip archive (`CLIENT_SUMMARY.md`,
/// `audit_bundle.json`, `VERIFY.txt`, and `SHA256SUMS`) after running the
/// mandatory leak pre-check.
#[cold]
#[inline(never)]
pub fn build_handoff_bundle_zip(
    cfg: &Config,
    person: &Person,
    title: &str,
    summary_markdown: &str,
    security: &Value,
    policy: &crate::WorkspacePolicy,
    force: bool,
) -> Result<HandoffExportResult, Box<HandoffExportBlocked>> {
    build_handoff_bundle_zip_with_chain(
        cfg,
        person,
        title,
        summary_markdown,
        security,
        policy,
        force,
        None,
        true,
    )
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub fn build_handoff_bundle_zip_with_chain(
    cfg: &Config,
    person: &Person,
    title: &str,
    summary_markdown: &str,
    security: &Value,
    policy: &crate::WorkspacePolicy,
    force: bool,
    cached_chain: Option<(&[receipts::ChainedReceipt], bool, &str)>,
    keep_parts: bool,
) -> Result<HandoffExportResult, Box<HandoffExportBlocked>> {
    let scan = scan_handoff_leaks(
        cfg,
        &[("CLIENT_SUMMARY.md", summary_markdown)],
        cfg.api_key.as_deref(),
    );
    let strict_block = policy.profile == "strict_client" || policy.strict_citation_gate;
    if strict_block && !scan.clean && !force {
        return Err(Box::new(HandoffExportBlocked {
            scan,
            policy: policy.clone(),
        }));
    }

    let mut summary_md = summary_markdown.to_string();
    let mut audit_bytes = build_audit_bundle_json_bytes_with_chain(cfg, person, security, cached_chain);
    if policy.redact_host_paths {
        summary_md = crate::redact_host_paths(&summary_md);
        let redacted = {
            let s = String::from_utf8_lossy(&audit_bytes);
            if contains_host_path(&s) {
                Some(crate::redact_host_paths(&s).into_bytes())
            } else {
                None
            }
        };
        if let Some(new_bytes) = redacted {
            audit_bytes = new_bytes;
        }
    }

    let verify_txt = format!(
        "Ergon Client Handoff Verification Guide\n\
         Title: {title}\n\
         Policy: {label} ({profile})\n\
         Leak pre-check findings: {leaks}\n\n\
         1. Verify archive file checksums locally:\n\
            sha256sum -c SHA256SUMS\n\n\
         2. Verify the Field-Book audit bundle offline with Ergon:\n\
            ergon verify audit_bundle.json\n\n\
         3. Verify the client deliverable summary offline with Ergon:\n\
            ergon verify CLIENT_SUMMARY.md\n",
        label = policy.label,
        profile = policy.profile,
        leaks = scan.leak_count
    );

    let sum_summary = sha256_hex(summary_md.as_bytes());
    let sum_bundle = sha256_hex(&audit_bytes);
    let sum_verify = sha256_hex(verify_txt.as_bytes());
    let sha256sums = format!(
        "{sum_summary}  CLIENT_SUMMARY.md\n\
         {sum_bundle}  audit_bundle.json\n\
         {sum_verify}  VERIFY.txt\n"
    );

    let zip_bytes = crate::docs::zip_write_refs(&[
        ("CLIENT_SUMMARY.md", summary_md.as_bytes()),
        ("audit_bundle.json", &audit_bytes),
        ("VERIFY.txt", verify_txt.as_bytes()),
        ("SHA256SUMS", sha256sums.as_bytes()),
    ]);
    let sha256 = sha256_hex(&zip_bytes);

    if !keep_parts {
        drop(audit_bytes);
        drop(summary_md);
        return Ok(HandoffExportResult {
            zip_bytes,
            sha256,
            summary_md: String::new(),
            audit_bundle_bytes: Vec::new(),
            verify_txt,
            sha256sums,
            scan,
        });
    }

    Ok(HandoffExportResult {
        zip_bytes,
        sha256,
        summary_md,
        audit_bundle_bytes: audit_bytes,
        verify_txt,
        sha256sums,
        scan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDir(std::path::PathBuf);
    impl TestDir {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!(
                "ergon_test_{tag}_{}_{}",
                std::process::id(),
                records::now_millis()
            ));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn verify_evidence_covers_bundles_packs_and_playbooks() {
        let dir = TestDir::new("verify_evidence");
        let data_dir = dir.path().to_path_buf();
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        std::fs::create_dir_all(&workspace_dir).unwrap();
        let cfg = Config {
            data_dir,
            workspace_dir: workspace_dir.clone(),
            memory_path,
            model_base_url: "http://127.0.0.1:11434/v1".into(),
            model_id: "test".into(),
            api_key: None,
        };
        std::fs::write(cfg.workspace_dir.join("report.md"), "# Q4 Summary\nLine 2 ok.\n").unwrap();

        let person = Person {
            id: "default".into(),
            name: "You".into(),
            role: "Operations".into(),
        };
        let sec = json!({"events_sha256": "0".repeat(64), "events": []});
        let bundle_bytes = build_audit_bundle_json_bytes(&cfg, &person, &sec);
        let bundle_str = String::from_utf8(bundle_bytes).unwrap();

        let ok_bundle = verify_evidence(&bundle_str, Some("ergon-audit-bundle.json"), Some(&cfg.workspace_dir));
        assert!(ok_bundle.valid, "valid bundle failed: {:?}", ok_bundle.errors);
        assert_eq!(ok_bundle.kind, "audit_bundle");
        assert_eq!(ok_bundle.workspace_valid, Some(true));

        let tampered_bundle = bundle_str.replace("Operations", "Tampered");
        let bad_bundle = verify_evidence(&tampered_bundle, None, None);
        assert!(!bad_bundle.valid);
        assert_eq!(bad_bundle.bundle_hash_valid, Some(false));

        let report_bytes = std::fs::read(cfg.workspace_dir.join("report.md")).unwrap();
        let report_sha = sha256_hex(&report_bytes);
        let pack_md = format!(
            "# Client Pack\n\n- Prepared for: **You**\n- Deliverables included: 1\n\n## Included Deliverables\n\n- `report.md` (2 lines, `sha256:{report_sha}`)\n\n## Citation Verification\n\n- `[report.md:L2]` -> **Verified** (2 lines in file, live `sha256:{}`)\n\n---\n\n## `report.md`\n\n_SHA-256: `{report_sha}` · 2 lines · {} bytes_\n\n# Q4 Summary\nLine 2 ok.\n",
            &report_sha[..12],
            report_bytes.len()
        );
        let ok_pack = verify_evidence(&pack_md, Some("pack.md"), Some(&cfg.workspace_dir));
        assert!(ok_pack.valid, "valid pack failed: {:?}", ok_pack.errors);
        assert_eq!(ok_pack.kind, "deliverable_pack");
        assert_eq!(ok_pack.citations_valid, Some(true));

        let tampered_pack = pack_md.replace("Line 2 ok.", "Line 2 MOD");
        let bad_pack = verify_evidence(&tampered_pack, Some("pack.md"), None);
        assert!(!bad_pack.valid);

        let rt_opts = routines::PlaybookOptions {
            steps: vec!["Read report.md".into()],
            expected_inputs: vec!["report.md".into()],
            pack_deliverables: true,
        };
        let pb_sha = routines::playbook_canonical_sha256("Weekly Review", "Audit report.md", "manual", None, None, &rt_opts);
        let pb_json = json!({
            "format": routines::PLAYBOOK_PACK_FORMAT,
            "playbook": {
                "name": "Weekly Review",
                "prompt": "Audit report.md",
                "schedule": "manual",
                "steps": ["Read report.md"],
                "expected_inputs": ["report.md"],
                "pack_deliverables": true
            },
            "sha256": pb_sha,
        })
        .to_string();
        let ok_pb = verify_evidence(&pb_json, Some("weekly.ergon-playbook.json"), None);
        assert!(ok_pb.valid, "valid playbook failed: {:?}", ok_pb.errors);
        assert_eq!(ok_pb.kind, "playbook_pack");

        let bad_pb = pb_json.replace("Audit report.md", "Audit tampered.md");
        assert!(!verify_evidence(&bad_pb, None, None).valid);
    }

    #[test]
    fn handoff_leak_scanner_blocks_secrets_and_paths_under_strict_client_and_builds_valid_zip() {
        let dir = TestDir::new("handoff_zip");
        let data_dir = dir.path().to_path_buf();
        let workspace_dir = data_dir.join("workspace");
        let memory_path = data_dir.join("memory.md");
        std::fs::create_dir_all(&workspace_dir).unwrap();
        let cfg = Config {
            data_dir: data_dir.clone(),
            workspace_dir: workspace_dir.clone(),
            memory_path,
            model_base_url: "http://127.0.0.1:11434/v1".into(),
            model_id: "test".into(),
            api_key: None,
        };
        std::fs::write(
            workspace_dir.join("deliverable.md"),
            "Deliverable ready.\nLocal trace at /home/alice/secret.txt\nAPI_KEY=sk-ergon-test123456\n",
        )
        .unwrap();
        let person = Person {
            id: "default".into(),
            name: "You".into(),
            role: "Operations".into(),
        };
        let sec = json!({"events_sha256": "0".repeat(64), "events": []});
        let strict = crate::WorkspacePolicy::for_profile("strict_client", 1000).unwrap();

        let blocked = build_handoff_bundle_zip(
            &cfg,
            &person,
            "Q4 Handoff",
            "# Q4 Handoff\n\nAll good.\n",
            &sec,
            &strict,
            false,
        )
        .err()
        .expect("strict_client must block leak findings");
        assert!(!blocked.scan.clean);
        assert!(blocked.scan.host_path_count >= 1);
        assert!(blocked.scan.secret_count >= 1 || blocked.scan.env_snippet_count >= 1);
        assert!(blocked.scan.findings.iter().any(|f| f.file == "deliverable.md" && f.line >= 2));

        // Clean workspace produces valid ZIP with all 4 artifacts and 0 leaks.
        std::fs::write(workspace_dir.join("deliverable.md"), "# Q4 Summary\nAll figures verified.\n").unwrap();
        let ok = build_handoff_bundle_zip(
            &cfg,
            &person,
            "Q4 Handoff",
            "# Q4 Handoff\n\nAll figures verified.\n",
            &sec,
            &strict,
            false,
        )
        .ok()
        .expect("clean workspace should succeed under strict_client");
        assert!(ok.scan.clean);
        assert_eq!(ok.scan.leak_count, 0);
        assert!(ok.zip_bytes.starts_with(b"PK\x03\x04"));
        assert!(ok.sha256sums.contains("CLIENT_SUMMARY.md"));
        assert!(ok.sha256sums.contains("audit_bundle.json"));
        assert!(ok.sha256sums.contains("VERIFY.txt"));
        assert!(ok.verify_txt.contains("ergon verify audit_bundle.json"));
        let verified = verify_evidence(
            std::str::from_utf8(&ok.audit_bundle_bytes).unwrap(),
            Some("audit_bundle.json"),
            Some(&workspace_dir),
        );
        assert!(verified.valid, "handoff audit_bundle.json invalid: {:?}", verified.errors);
    }
}
