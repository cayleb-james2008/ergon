//! Cryptographically chained SHA-256 Field-Book receipts & provenance export.
//!
//! Every completed work record in `work.json` (plus any external seeded
//! `receipts.json`) is linked into a deterministic SHA-256 hash chain so each
//! receipt can be verified or exported as Markdown or JSON with workspace file
//! SHA-256 digests.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::records;

pub const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Debug)]
pub struct ChainedReceipt {
    pub id: String,
    pub index: usize,
    pub ts: u64,
    pub task: String,
    pub summary: String,
    pub files: Vec<String>,
    pub thread_id: String,
    pub finished: bool,
    pub answered: bool,
    pub prev_hash: String,
    pub sha256: String,
    pub engagement_id: Option<String>,
    pub usage: Option<Value>,
}

pub fn canonical_json(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, Value> =
                std::collections::BTreeMap::new();
            for (k, val) in map {
                sorted.insert(k.clone(), canonical_json(val));
            }
            let mut out = serde_json::Map::new();
            for (k, val) in sorted {
                out.insert(k, val);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

pub fn compute_usage_digest(engagement_id: Option<&str>, usage: Option<&Value>) -> String {
    let eid = engagement_id.map(str::trim).filter(|s| !s.is_empty());
    let can_usage = usage.filter(|u| !u.is_null()).map(canonical_json);
    let payload = json!({
        "engagement_id": eid,
        "usage": can_usage,
    });
    sha256_hex(payload.to_string().as_bytes())
}

impl ChainedReceipt {
    pub fn usage_digest(&self) -> Option<String> {
        let has_eid = self
            .engagement_id
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty());
        let has_usage = self.usage.as_ref().is_some_and(|u| !u.is_null());
        if has_eid || has_usage {
            Some(compute_usage_digest(
                self.engagement_id.as_deref(),
                self.usage.as_ref(),
            ))
        } else {
            None
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "id": self.id,
            "index": self.index,
            "ts": self.ts,
            "task": self.task,
            "summary": self.summary,
            "files": self.files,
            "thread_id": self.thread_id,
            "finished": self.finished,
            "answered": self.answered,
            "prev_hash": self.prev_hash,
            "sha256": self.sha256,
        });
        if let Some(eid) = self
            .engagement_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            obj["engagement_id"] = json!(eid);
        }
        if let Some(u) = self.usage.as_ref().filter(|u| !u.is_null()) {
            obj["usage"] = u.clone();
        }
        if let Some(digest) = self.usage_digest() {
            obj["usage_digest"] = json!(digest);
        }
        obj
    }
}

pub use crate::tripwire::sha256_hex;

#[allow(clippy::too_many_arguments)]
fn compute_entry_hash(
    prev_hash: &str,
    index: usize,
    ts: u64,
    thread_id: &str,
    finished: bool,
    answered: bool,
    task: &str,
    summary: &str,
    files: &[String],
    engagement_id: Option<&str>,
    usage: Option<&Value>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(b"|");
    hasher.update(index.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(ts.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(thread_id.as_bytes());
    hasher.update(b"|");
    hasher.update(if finished { b"1" } else { b"0" });
    hasher.update(b"|");
    hasher.update(if answered { b"1" } else { b"0" });
    hasher.update(b"|");
    hasher.update(task.as_bytes());
    hasher.update(b"|");
    hasher.update(summary.as_bytes());
    hasher.update(b"|");
    for (i, f) in files.iter().enumerate() {
        if i > 0 {
            hasher.update(b",");
        }
        hasher.update(f.as_bytes());
    }
    let has_eid = engagement_id.is_some_and(|s| !s.trim().is_empty());
    let has_usage = usage.is_some_and(|u| !u.is_null());
    if has_eid || has_usage {
        let digest = compute_usage_digest(engagement_id, usage);
        hasher.update(b"|v2|");
        hasher.update(digest.as_bytes());
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Load raw receipt entries from `work.json` plus optional `receipts.json`
/// (supports `{"receipts": [...]}`, `{"entries": [...]}`, or a bare JSON array).
pub fn load_raw_entries(person_data_dir: &Path) -> Vec<Value> {
    let mut entries = records::load_work(person_data_dir);
    let extra_path = person_data_dir.join("receipts.json");
    if let Ok(text) = std::fs::read_to_string(&extra_path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            let extra = v["receipts"]
                .as_array()
                .or_else(|| v["entries"].as_array())
                .or_else(|| v.as_array());
            if let Some(arr) = extra {
                entries.extend(arr.iter().cloned());
            }
        }
    }
    entries
}

/// Append a single parsed receipt object onto the hash chain.
fn chain_step(prev_hash: &mut String, index: usize, v: &Value) -> ChainedReceipt {
    let ts = v["ts"].as_u64().unwrap_or(0);
    let task = v["task"]
        .as_str()
        .or_else(|| v["title"].as_str())
        .unwrap_or("")
        .to_string();
    let summary = v["summary"].as_str().unwrap_or("").to_string();
    let thread_id = v["thread_id"].as_str().unwrap_or("").to_string();
    let finished = v["finished"].as_bool().unwrap_or(true);
    let answered = v["answered"].as_bool().unwrap_or(false);
    let files: Vec<String> = v["files"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let id = v["id"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("rcpt-{ts}-{index}"));
    let engagement_id = v["engagement_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let usage = v
        .get("usage")
        .filter(|u| !u.is_null())
        .map(canonical_json);

    let sha256 = compute_entry_hash(
        prev_hash,
        index,
        ts,
        &thread_id,
        finished,
        answered,
        &task,
        &summary,
        &files,
        engagement_id.as_deref(),
        usage.as_ref(),
    );
    let prev = std::mem::replace(prev_hash, sha256.clone());
    ChainedReceipt {
        id,
        index,
        ts,
        task,
        summary,
        files,
        thread_id,
        finished,
        answered,
        prev_hash: prev,
        sha256,
        engagement_id,
        usage,
    }
}

/// Read one JSON byte from a buffered reader, skipping ASCII whitespace when `skip_ws` is true.
fn next_json_byte<R: std::io::BufRead>(r: &mut R, skip_ws: bool) -> Option<u8> {
    loop {
        let buf = r.fill_buf().ok()?;
        if buf.is_empty() {
            return None;
        }
        if !skip_ws {
            let b = buf[0];
            r.consume(1);
            return Some(b);
        }
        if let Some(pos) = buf.iter().position(|b| !b.is_ascii_whitespace()) {
            let b = buf[pos];
            r.consume(pos + 1);
            return Some(b);
        }
        let len = buf.len();
        r.consume(len);
    }
}

/// Read a JSON string literal (starting immediately after the opening `"`), appending raw bytes
/// if `capture` is `Some`. Returns the unescaped text when `decode` is `Some`, or `false` on EOF.
fn scan_json_string<R: std::io::BufRead>(
    r: &mut R,
    capture: Option<&mut Vec<u8>>,
    decode: Option<&mut Vec<u8>>,
) -> bool {
    let mut cap = capture;
    let mut dec = decode;
    let mut escaped = false;
    while let Some(b) = next_json_byte(r, false) {
        if let Some(buf) = cap.as_deref_mut() {
            buf.push(b);
        }
        if escaped {
            if let Some(d) = dec.as_deref_mut() {
                d.push(b);
            }
            escaped = false;
            continue;
        }
        if b == b'\\' {
            escaped = true;
            continue;
        }
        if b == b'"' {
            return true;
        }
        if let Some(d) = dec.as_deref_mut() {
            d.push(b);
        }
    }
    false
}

/// Skip any JSON value whose first non-whitespace byte has already been consumed (`first`).
fn skip_json_value<R: std::io::BufRead>(r: &mut R, first: u8) -> bool {
    match first {
        b'"' => scan_json_string(r, None, None),
        b'{' | b'[' => {
            let mut depth = 1usize;
            while let Some(b) = next_json_byte(r, false) {
                match b {
                    b'"' => {
                        if !scan_json_string(r, None, None) {
                            return false;
                        }
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        _ => loop {
            let buf = match r.fill_buf() {
                Ok(b) if !b.is_empty() => b,
                _ => return true,
            };
            if let Some(pos) = buf
                .iter()
                .position(|b| matches!(b, b',' | b'}' | b']') || b.is_ascii_whitespace())
            {
                r.consume(pos);
                return true;
            }
            let len = buf.len();
            r.consume(len);
        },
    }
}

/// Stream objects from either a top-level JSON array or a named array key (`entries`/`receipts`)
/// inside a top-level JSON object, reusing one tiny buffer so 5k-entry files never materialize a DOM.
fn stream_json_array_objects(
    path: &Path,
    keys: &[&str],
    on_obj: &mut dyn FnMut(Value),
) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return true;
    };
    let mut r = std::io::BufReader::with_capacity(64 * 1024, file);
    let Some(first) = next_json_byte(&mut r, true) else {
        return true;
    };
    let in_array = match first {
        b'[' => true,
        b'{' => {
            let mut found = false;
            let mut key_buf = Vec::with_capacity(32);
            while let Some(b) = next_json_byte(&mut r, true) {
                if b == b'}' {
                    break;
                }
                if b == b',' {
                    continue;
                }
                if b != b'"' {
                    return false;
                }
                key_buf.clear();
                if !scan_json_string(&mut r, None, Some(&mut key_buf)) {
                    return false;
                }
                if next_json_byte(&mut r, true) != Some(b':') {
                    return false;
                }
                let Some(val_first) = next_json_byte(&mut r, true) else {
                    return false;
                };
                let matches_key = std::str::from_utf8(&key_buf)
                    .is_ok_and(|k| keys.contains(&k));
                if matches_key && val_first == b'[' {
                    found = true;
                    break;
                }
                if !skip_json_value(&mut r, val_first) {
                    return false;
                }
            }
            if !found {
                return true;
            }
            true
        }
        _ => return false,
    };
    if !in_array {
        return false;
    }

    let mut obj_buf = Vec::with_capacity(1024);
    while let Some(b) = next_json_byte(&mut r, true) {
        if b == b']' {
            return true;
        }
        if b == b',' {
            continue;
        }
        if b != b'{' {
            return false;
        }
        obj_buf.clear();
        obj_buf.push(b'{');
        let mut depth = 1usize;
        while let Some(ch) = next_json_byte(&mut r, false) {
            obj_buf.push(ch);
            match ch {
                b'"' => {
                    if !scan_json_string(&mut r, Some(&mut obj_buf), None) {
                        return false;
                    }
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return false;
        }
        match serde_json::from_slice::<Value>(&obj_buf) {
            Ok(v) => on_obj(v),
            Err(_) => return false,
        }
    }
    false
}

/// Stream `work.json` + optional `receipts.json` directly into a verified `ChainedReceipt` vector
/// without materializing all raw `serde_json::Value` DOM trees in memory simultaneously.
pub fn load_chain(person_data_dir: &Path) -> Vec<ChainedReceipt> {
    let mut chain = Vec::new();
    let mut prev_hash = GENESIS_HASH.to_string();

    let work_path = records::work_path(person_data_dir);
    let mut work_ok = true;
    if work_path.exists() {
        let mut idx = chain.len();
        let start_len = idx;
        let start_prev = prev_hash.clone();
        work_ok = stream_json_array_objects(&work_path, &["entries"], &mut |v| {
            chain.push(chain_step(&mut prev_hash, idx, &v));
            idx += 1;
        });
        if !work_ok {
            chain.truncate(start_len);
            prev_hash = start_prev;
            for v in records::load_work(person_data_dir) {
                let i = chain.len();
                chain.push(chain_step(&mut prev_hash, i, &v));
            }
        }
    }
    let _ = work_ok;

    let extra_path = person_data_dir.join("receipts.json");
    if extra_path.exists() {
        let start_len = chain.len();
        let start_prev = prev_hash.clone();
        let mut idx = start_len;
        let ok = stream_json_array_objects(&extra_path, &["receipts", "entries"], &mut |v| {
            chain.push(chain_step(&mut prev_hash, idx, &v));
            idx += 1;
        });
        if !ok {
            chain.truncate(start_len);
            prev_hash = start_prev;
            if let Ok(text) = std::fs::read_to_string(&extra_path) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    let extra = v["receipts"]
                        .as_array()
                        .or_else(|| v["entries"].as_array())
                        .or_else(|| v.as_array());
                    if let Some(arr) = extra {
                        for item in arr {
                            let i = chain.len();
                            chain.push(chain_step(&mut prev_hash, i, item));
                        }
                    }
                }
            }
        }
    }
    chain.shrink_to_fit();
    chain
}

pub fn build_chain(entries: &[Value]) -> Vec<ChainedReceipt> {
    let mut chain = Vec::with_capacity(entries.len());
    let mut prev_hash = GENESIS_HASH.to_string();

    for (index, v) in entries.iter().enumerate() {
        chain.push(chain_step(&mut prev_hash, index, v));
    }
    chain
}

pub fn verify_chain(chain: &[ChainedReceipt]) -> (bool, String) {
    let mut expected_prev = GENESIS_HASH.to_string();
    for (i, r) in chain.iter().enumerate() {
        if r.index != i || r.prev_hash != expected_prev {
            return (false, expected_prev);
        }
        let recomputed = compute_entry_hash(
            &r.prev_hash,
            r.index,
            r.ts,
            &r.thread_id,
            r.finished,
            r.answered,
            &r.task,
            &r.summary,
            &r.files,
            r.engagement_id.as_deref(),
            r.usage.as_ref(),
        );
        if recomputed != r.sha256 {
            return (false, expected_prev);
        }
        expected_prev = r.sha256.clone();
    }
    (true, expected_prev)
}

/// Find a receipt by exact receipt id (`rcpt-...`), thread_id (most recent
/// receipt for that thread), or numeric chain index.
pub fn find_receipt<'a>(chain: &'a [ChainedReceipt], key: &str) -> Option<&'a ChainedReceipt> {
    if let Some(r) = chain.iter().find(|r| r.id == key) {
        return Some(r);
    }
    if let Some(r) = chain.iter().rev().find(|r| !r.thread_id.is_empty() && r.thread_id == key) {
        return Some(r);
    }
    if let Ok(idx) = key.parse::<usize>() {
        return chain.get(idx);
    }
    None
}

fn workspace_file_hashes(workspace_dir: &Path, files: &[String]) -> Vec<Value> {
    let mut out = Vec::new();
    for rel in files {
        let hash_info = match crate::sandbox_path_resolving(workspace_dir, rel, true) {
            Ok(full) if full.is_file() => match std::fs::read(&full) {
                Ok(bytes) => json!({
                    "path": rel,
                    "present": true,
                    "bytes": bytes.len(),
                    "sha256": sha256_hex(&bytes),
                }),
                Err(_) => json!({
                    "path": rel,
                    "present": false,
                    "bytes": 0,
                    "sha256": Value::Null,
                }),
            },
            _ => json!({
                "path": rel,
                "present": false,
                "bytes": 0,
                "sha256": Value::Null,
            }),
        };
        out.push(hash_info);
    }
    out
}

pub fn export_receipt_json(
    receipt: &ChainedReceipt,
    workspace_dir: &Path,
    chain_len: usize,
    chain_valid: bool,
) -> Value {
    json!({
        "schema": "ergon.field_book_receipt.v1",
        "exported_at_ms": records::now_millis(),
        "receipt": receipt.to_json(),
        "provenance": {
            "algorithm": "SHA-256",
            "chain_index": receipt.index,
            "chain_length": chain_len,
            "prev_hash": receipt.prev_hash,
            "sha256": receipt.sha256,
            "chain_verified": chain_valid,
        },
        "workspace_files": workspace_file_hashes(workspace_dir, &receipt.files),
    })
}

pub fn export_receipt_markdown(
    receipt: &ChainedReceipt,
    workspace_dir: &Path,
    chain_len: usize,
    chain_valid: bool,
) -> String {
    let status = if receipt.finished {
        "Finished"
    } else {
        "Did not finish"
    };
    let date = if receipt.ts > 0 {
        records::date_string_local(receipt.ts)
    } else {
        "unknown".to_string()
    };
    let file_hashes = workspace_file_hashes(workspace_dir, &receipt.files);
    let mut md = String::new();
    md.push_str(&format!("# Field-Book Receipt: {}\n\n", receipt.id));
    md.push_str(&format!("- **Status:** {status}\n"));
    md.push_str(&format!("- **Date:** {date} (`{}`)\n", receipt.ts));
    if !receipt.thread_id.is_empty() {
        md.push_str(&format!("- **Thread:** `{}`\n", receipt.thread_id));
    }
    md.push_str(&format!(
        "- **Chain Position:** {} of {} (verified: `{}`)\n",
        receipt.index + 1,
        chain_len,
        chain_valid
    ));
    md.push_str(&format!("- **Previous SHA-256:** `{}`\n", receipt.prev_hash));
    md.push_str(&format!("- **Receipt SHA-256:** `{}`\n", receipt.sha256));
    if let Some(eid) = receipt.engagement_id.as_deref().filter(|s| !s.is_empty()) {
        md.push_str(&format!("- **Engagement:** `{eid}`\n"));
    }
    if let Some(digest) = receipt.usage_digest() {
        md.push_str(&format!("- **Usage Digest:** `{digest}`\n"));
    }
    if let Some(u) = receipt.usage.as_ref().and_then(records::RunUsage::from_json) {
        md.push_str(&format!(
            "- **Metering:** {} tokens ({} prompt / {} completion), ${:.6} USD, {:.1} min saved (${:.2} billable)\n",
            u.total_tokens, u.prompt_tokens, u.completion_tokens, u.cost_usd, u.estimated_minutes_saved, u.billable_value_usd
        ));
    }
    md.push('\n');

    md.push_str("## Task\n\n");
    md.push_str(&receipt.task);
    md.push_str("\n\n## Summary\n\n");
    md.push_str(&receipt.summary);
    md.push_str("\n\n## Saved Workspace Files\n\n");
    if file_hashes.is_empty() {
        md.push_str("_No files written during this run._\n");
    } else {
        md.push_str("| Path | Present | Bytes | SHA-256 |\n");
        md.push_str("| --- | --- | ---: | --- |\n");
        for fh in file_hashes {
            let p = fh["path"].as_str().unwrap_or("");
            let present = if fh["present"].as_bool().unwrap_or(false) {
                "yes"
            } else {
                "no"
            };
            let bytes = fh["bytes"].as_u64().unwrap_or(0);
            let h = fh["sha256"].as_str().unwrap_or("—");
            md.push_str(&format!("| `{p}` | {present} | {bytes} | `{h}` |\n"));
        }
    }
    md
}
