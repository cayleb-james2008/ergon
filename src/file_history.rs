//! Automatic workspace file revision history, line-by-line diff, and safe restore.
//!
//! Revisions are stored outside the workspace under `<person_data_dir>/.history/`
//! keyed by the SHA-256 digest of each validated relative workspace path, so
//! caller-supplied path characters never participate in `.history/` directory
//! traversal. Every path is validated through `crate::sandbox_path_resolving`,
//! every revision ID is restricted to `[A-Za-z0-9_-]`, and restores snapshot the
//! current file first before performing an atomic write-and-rename.

use crate::receipts::sha256_hex;
use crate::{records, sandbox_path_resolving};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const MAX_REVISIONS_PER_FILE: usize = 64;
pub const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
pub const CURRENT_SENTINEL: &str = "current";

static REV_SEQ: AtomicU64 = AtomicU64::new(0);
static HISTORY_LOCK: Mutex<()> = Mutex::new(());

fn with_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = HISTORY_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRevision {
    pub rev_id: String,
    pub path: String,
    pub timestamp_ms: u64,
    pub size_bytes: u64,
    pub sha256: String,
    pub reason: String,
    pub is_text: bool,
    pub line_count: usize,
}

impl FileRevision {
    pub fn to_json(&self) -> Value {
        json!({
            "rev_id": self.rev_id,
            "path": self.path,
            "timestamp_ms": self.timestamp_ms,
            "size_bytes": self.size_bytes,
            "sha256": self.sha256,
            "reason": self.reason,
            "is_text": self.is_text,
            "line_count": self.line_count,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(FileRevision {
            rev_id: v["rev_id"].as_str()?.to_string(),
            path: v["path"].as_str()?.to_string(),
            timestamp_ms: v["timestamp_ms"].as_u64().unwrap_or(0),
            size_bytes: v["size_bytes"].as_u64().unwrap_or(0),
            sha256: v["sha256"].as_str().unwrap_or("").to_string(),
            reason: v["reason"].as_str().unwrap_or("overwrite").to_string(),
            is_text: v["is_text"].as_bool().unwrap_or(true),
            line_count: v["line_count"].as_u64().unwrap_or(0) as usize,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: String, // "context" | "add" | "remove"
    pub left_line: Option<usize>,
    pub right_line: Option<usize>,
    pub text: String,
}

impl DiffLine {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "left_line": self.left_line,
            "right_line": self.right_line,
            "text": self.text,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffHunk {
    pub left_start: usize,
    pub left_count: usize,
    pub right_start: usize,
    pub right_count: usize,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn to_json(&self) -> Value {
        json!({
            "left_start": self.left_start,
            "left_count": self.left_count,
            "right_start": self.right_start,
            "right_count": self.right_count,
            "lines": self.lines.iter().map(|l| l.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct RevisionDiff {
    pub path: String,
    pub rev_a: String,
    pub rev_b: String,
    pub sha256_a: String,
    pub sha256_b: String,
    pub is_binary: bool,
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
    pub similarity_pct: u8,
    pub hunks: Vec<DiffHunk>,
    pub unified: String,
}

impl RevisionDiff {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "rev_a": self.rev_a,
            "rev_b": self.rev_b,
            "sha256_a": self.sha256_a,
            "sha256_b": self.sha256_b,
            "is_binary": self.is_binary,
            "added": self.added,
            "removed": self.removed,
            "unchanged": self.unchanged,
            "similarity_pct": self.similarity_pct,
            "hunks": self.hunks.iter().map(|h| h.to_json()).collect::<Vec<_>>(),
            "unified": self.unified,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RestoreResult {
    pub path: String,
    pub restored_rev_id: String,
    pub pre_restore_rev_id: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
}

impl RestoreResult {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "restored_rev_id": self.restored_rev_id,
            "pre_restore_rev_id": self.pre_restore_rev_id,
            "size_bytes": self.size_bytes,
            "sha256": self.sha256,
        })
    }
}

pub fn history_dir(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join(".history")
}

/// Strictly validate a workspace-relative path for history operations.
/// Rejects NULs, backslashes, absolute paths, `..`, `.history` segments,
/// control characters, and any symlink escape out of `workspace_dir`.
pub fn validate_history_path(workspace_dir: &Path, raw: &str) -> Result<(String, PathBuf), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("path is required".to_string());
    }
    if trimmed.len() > 512 {
        return Err("path exceeds maximum length (512 bytes)".to_string());
    }
    let low = trimmed.to_ascii_lowercase();
    if trimmed.contains('\0')
        || trimmed.contains('\\')
        || trimmed.contains(':')
        || trimmed.starts_with('/')
        || trimmed.chars().any(|c| c.is_control())
        || low.contains("%2e%2e")
        || low.contains("%252e")
        || low.contains("%2f")
        || low.contains("%5c")
        || low.contains("%00")
    {
        return Err(format!("invalid workspace path: {trimmed}"));
    }

    let mut clean_parts: Vec<String> = Vec::new();
    for comp in Path::new(trimmed).components() {
        match comp {
            Component::Normal(os) => {
                let part = os.to_string_lossy();
                if part.is_empty() || part == "." || part == ".." || part.starts_with('.') {
                    return Err(format!("path escapes the workspace: {trimmed}"));
                }
                clean_parts.push(part.into_owned());
            }
            _ => return Err(format!("path escapes the workspace: {trimmed}")),
        }
    }
    if clean_parts.is_empty() {
        return Err("path is required".to_string());
    }
    let rel = clean_parts.join("/");

    // Enforce canonical containment (follows any intermediate or final symlinks).
    let resolved = sandbox_path_resolving(workspace_dir, &rel, true)?;
    let unresolved = sandbox_path_resolving(workspace_dir, &rel, false)?;
    if let Ok(md) = unresolved.symlink_metadata() {
        if md.file_type().is_symlink() {
            return Err(format!("symlinks are not supported in file history: {rel}"));
        }
    }
    Ok((rel, resolved))
}

/// Strictly validate a revision identifier: ASCII alphanumeric, `-`, and `_`
/// only, 1..=96 chars. Blocks all path separators, `..`, and NUL injection.
pub fn validate_rev_id(rev_id: &str) -> Result<String, String> {
    let trimmed = rev_id.trim();
    if trimmed.is_empty() || trimmed.len() > 96 {
        return Err("invalid revision id".to_string());
    }
    if !trimmed
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(format!("invalid revision id: {trimmed}"));
    }
    Ok(trimmed.to_string())
}

fn bucket_dir(person_data_dir: &Path, normalized_rel: &str) -> PathBuf {
    history_dir(person_data_dir).join(sha256_hex(normalized_rel.as_bytes()))
}

fn manifest_path(bucket: &Path) -> PathBuf {
    bucket.join("manifest.json")
}

fn blob_path(bucket: &Path, rev_id: &str) -> PathBuf {
    bucket.join("blobs").join(format!("{rev_id}.blob"))
}

fn is_probably_text(bytes: &[u8]) -> bool {
    if bytes.contains(&0) {
        return false;
    }
    std::str::from_utf8(bytes).is_ok()
}

fn count_lines(bytes: &[u8]) -> usize {
    match std::str::from_utf8(bytes) {
        Ok(s) if !s.is_empty() => s.lines().count(),
        _ => 0,
    }
}

fn load_manifest(bucket: &Path) -> Vec<FileRevision> {
    let path = manifest_path(bucket);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v["revisions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(FileRevision::from_json)
                .filter(|r| validate_rev_id(&r.rev_id).is_ok())
                .collect()
        })
        .unwrap_or_default()
}

fn save_manifest(bucket: &Path, rel: &str, revisions: &[FileRevision]) -> Result<(), String> {
    std::fs::create_dir_all(bucket).map_err(|e| format!("create history dir: {e}"))?;
    let path = manifest_path(bucket);
    let tmp = bucket.join("manifest.json.tmp");
    let body = json!({
        "path": rel,
        "revisions": revisions.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    })
    .to_string();
    std::fs::write(&tmp, body).map_err(|e| format!("write history manifest: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("commit history manifest: {e}"))?;
    Ok(())
}

fn sanitize_reason(reason: &str) -> String {
    let cleaned: String = reason
        .replace(['\r', '\n'], " ")
        .trim()
        .chars()
        .take(48)
        .collect();
    if cleaned.is_empty() {
        "overwrite".to_string()
    } else {
        cleaned
    }
}

fn write_revision_unlocked(
    person_data_dir: &Path,
    rel: &str,
    bytes: &[u8],
    reason: &str,
) -> Result<FileRevision, String> {
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(format!(
            "file exceeds history snapshot limit of {MAX_SNAPSHOT_BYTES} bytes"
        ));
    }
    let bucket = bucket_dir(person_data_dir, rel);
    let blobs_dir = bucket.join("blobs");
    std::fs::create_dir_all(&blobs_dir).map_err(|e| format!("create blobs dir: {e}"))?;

    let ts = records::now_millis();
    let seq = REV_SEQ.fetch_add(1, Ordering::SeqCst);
    let rev_id = format!("rev-{ts}-{seq}");
    let digest = sha256_hex(bytes);
    let is_text = is_probably_text(bytes);
    let line_count = if is_text { count_lines(bytes) } else { 0 };

    let final_blob = blob_path(&bucket, &rev_id);
    let tmp_blob = blobs_dir.join(format!("{rev_id}.blob.tmp"));
    std::fs::write(&tmp_blob, bytes).map_err(|e| format!("write revision blob: {e}"))?;
    std::fs::rename(&tmp_blob, &final_blob).map_err(|e| format!("commit revision blob: {e}"))?;

    let rev = FileRevision {
        rev_id,
        path: rel.to_string(),
        timestamp_ms: ts,
        size_bytes: bytes.len() as u64,
        sha256: digest,
        reason: sanitize_reason(reason),
        is_text,
        line_count,
    };

    let mut revisions = load_manifest(&bucket);
    revisions.push(rev.clone());
    if revisions.len() > MAX_REVISIONS_PER_FILE {
        let excess = revisions.len() - MAX_REVISIONS_PER_FILE;
        for evicted in revisions.drain(0..excess) {
            let _ = std::fs::remove_file(blob_path(&bucket, &evicted.rev_id));
        }
    }
    save_manifest(&bucket, rel, &revisions)?;
    Ok(rev)
}

/// Record a snapshot of the CURRENT workspace file at `path` prior to an
/// overwrite or deletion. Returns `Ok(None)` when the file does not yet exist
/// on disk (first-time file creation).
pub fn record_revision(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
    reason: &str,
) -> Result<Option<FileRevision>, String> {
    let (rel, full) = validate_history_path(workspace_dir, path)?;
    if !full.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(&full).map_err(|e| format!("read {rel} for history: {e}"))?;
    with_lock(|| write_revision_unlocked(person_data_dir, &rel, &bytes, reason)).map(Some)
}

/// Record an explicit byte slice as a revision for `path` (still validating
/// `path` strictly against `workspace_dir`).
pub fn record_revision_bytes(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
    bytes: &[u8],
    reason: &str,
) -> Result<FileRevision, String> {
    let (rel, _full) = validate_history_path(workspace_dir, path)?;
    with_lock(|| write_revision_unlocked(person_data_dir, &rel, bytes, reason))
}

/// List recorded revisions for `path`, newest first.
pub fn list_revisions(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
) -> Result<Vec<FileRevision>, String> {
    let (rel, _full) = validate_history_path(workspace_dir, path)?;
    Ok(with_lock(|| {
        let mut revs = load_manifest(&bucket_dir(person_data_dir, &rel));
        revs.reverse();
        revs
    }))
}

/// Fast count of stored revisions for one validated workspace path.
pub fn revision_count(person_data_dir: &Path, workspace_dir: &Path, path: &str) -> usize {
    list_revisions(person_data_dir, workspace_dir, path)
        .map(|v| v.len())
        .unwrap_or(0)
}

/// List every workspace path that currently has recorded snapshots under
/// `<person_data_dir>/.history/` (including deleted files eligible for restore),
/// ordered by most recent snapshot timestamp descending.
pub fn list_tracked_files(person_data_dir: &Path) -> Vec<Value> {
    with_lock(|| {
        let root = history_dir(person_data_dir);
        let Ok(rd) = std::fs::read_dir(&root) else {
            return Vec::new();
        };
        let mut rows: Vec<(u64, Value)> = Vec::new();
        for entry in rd.filter_map(|e| e.ok()) {
            let bucket = entry.path();
            if !bucket.is_dir() {
                continue;
            }
            let revs = load_manifest(&bucket);
            let Some(latest) = revs.last() else {
                continue;
            };
            rows.push((
                latest.timestamp_ms,
                json!({
                    "path": latest.path,
                    "revision_count": revs.len(),
                    "latest_rev_id": latest.rev_id,
                    "latest_timestamp_ms": latest.timestamp_ms,
                    "latest_reason": latest.reason,
                    "latest_sha256": latest.sha256,
                }),
            ));
        }
        rows.sort_by(|a, b| b.0.cmp(&a.0));
        rows.into_iter().map(|(_, v)| v).collect()
    })
}

/// Ready-to-serve JSON payload for `GET /api/files/history?path=...`.
pub fn history_payload_json(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
) -> Result<Value, String> {
    let (rel, full) = validate_history_path(workspace_dir, path)?;
    let revisions = list_revisions(person_data_dir, workspace_dir, &rel)?;
    let (exists, current_bytes, current_sha256) = if full.is_file() {
        match std::fs::read(&full) {
            Ok(b) => (true, b.len() as u64, Some(sha256_hex(&b))),
            Err(_) => (false, 0, None),
        }
    } else {
        (false, 0, None)
    };
    Ok(json!({
        "path": rel,
        "exists": exists,
        "current_bytes": current_bytes,
        "current_sha256": current_sha256,
        "count": revisions.len(),
        "revisions": revisions.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    }))
}

/// Read the raw bytes of a specific revision (or `"current"` for the live workspace file).
pub fn read_revision(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
    rev_id: &str,
) -> Result<Vec<u8>, String> {
    let (rel, full) = validate_history_path(workspace_dir, path)?;
    let clean_rev = validate_rev_id(rev_id)?;
    if clean_rev == CURRENT_SENTINEL {
        return std::fs::read(&full).map_err(|e| format!("read current {rel}: {e}"));
    }
    with_lock(|| {
        let bucket = bucket_dir(person_data_dir, &rel);
        let revs = load_manifest(&bucket);
        let meta = revs
            .iter()
            .find(|r| r.rev_id == clean_rev)
            .ok_or_else(|| format!("revision '{clean_rev}' not found for {rel}"))?;
        let data = std::fs::read(blob_path(&bucket, &clean_rev))
            .map_err(|e| format!("read revision blob {clean_rev}: {e}"))?;
        let actual = sha256_hex(&data);
        if !meta.sha256.is_empty() && actual != meta.sha256 {
            return Err(format!(
                "revision '{clean_rev}' failed integrity verification (expected {}, got {actual})",
                meta.sha256
            ));
        }
        Ok(data)
    })
}

fn split_lines_normalized(text: &str) -> Vec<String> {
    let norm = text.replace("\r\n", "\n").replace('\r', "\n");
    if norm.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = norm.split('\n').map(|s| s.to_string()).collect();
    if lines.len() > 1 && lines.last().is_some_and(|s| s.is_empty()) {
        lines.pop();
    }
    lines
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Eq(usize, usize),
    Del(usize),
    Add(usize),
}

/// Optimal LCS diff for normal files (`m * n <= 250_000`), falling back to a
/// linear-time windowed hash-anchor diff for very large files.
fn compute_line_ops(left: &[String], right: &[String]) -> Vec<Op> {
    let m = left.len();
    let n = right.len();
    if m.saturating_mul(n) <= 250_000 {
        let width = n + 1;
        let mut dp = vec![0u32; (m + 1) * width];
        for i in (0..m).rev() {
            for j in (0..n).rev() {
                dp[i * width + j] = if left[i] == right[j] {
                    dp[(i + 1) * width + (j + 1)] + 1
                } else {
                    dp[(i + 1) * width + j].max(dp[i * width + (j + 1)])
                };
            }
        }
        let mut ops = Vec::with_capacity(m + n);
        let (mut i, mut j) = (0usize, 0usize);
        while i < m && j < n {
            if left[i] == right[j] {
                ops.push(Op::Eq(i, j));
                i += 1;
                j += 1;
            } else if dp[(i + 1) * width + j] >= dp[i * width + (j + 1)] {
                ops.push(Op::Del(i));
                i += 1;
            } else {
                ops.push(Op::Add(j));
                j += 1;
            }
        }
        while i < m {
            ops.push(Op::Del(i));
            i += 1;
        }
        while j < n {
            ops.push(Op::Add(j));
            j += 1;
        }
        return ops;
    }

    // Large-file linear-time hash-anchor resync diff.
    let mut right_pos: HashMap<&str, Vec<usize>> = HashMap::new();
    for (idx, line) in right.iter().enumerate() {
        let e = right_pos.entry(line.as_str()).or_default();
        if e.len() < 32 {
            e.push(idx);
        }
    }
    let mut left_pos: HashMap<&str, Vec<usize>> = HashMap::new();
    for (idx, line) in left.iter().enumerate() {
        let e = left_pos.entry(line.as_str()).or_default();
        if e.len() < 32 {
            e.push(idx);
        }
    }
    let find_in = |map: &HashMap<&str, Vec<usize>>, key: &str, lo: usize, hi: usize| -> Option<usize> {
        map.get(key)?
            .iter()
            .copied()
            .find(|&p| p >= lo && p <= hi)
    };

    let mut ops = Vec::with_capacity(m + n);
    let (mut i, mut j) = (0usize, 0usize);
    let window = 32usize;
    while i < m && j < n {
        if left[i] == right[j] {
            ops.push(Op::Eq(i, j));
            i += 1;
            j += 1;
            continue;
        }
        let mut best: Option<(usize, usize, usize)> = None;
        let max_di = (m - 1 - i).min(window);
        let max_j = (j + window).min(n - 1);
        for di in 0..=max_di {
            if best.is_some_and(|(cost, _, _)| di >= cost) {
                break;
            }
            if let Some(nj) = find_in(&right_pos, &left[i + di], j, max_j) {
                let cost = di + (nj - j);
                if best.map(|(c, _, _)| cost < c).unwrap_or(true) {
                    best = Some((cost, i + di, nj));
                }
            }
        }
        let max_dj = (n - 1 - j).min(window);
        let max_i = (i + window).min(m - 1);
        for dj in 0..=max_dj {
            if best.is_some_and(|(cost, _, _)| dj >= cost) {
                break;
            }
            if let Some(ni) = find_in(&left_pos, &right[j + dj], i, max_i) {
                let cost = (ni - i) + dj;
                if best.map(|(c, _, _)| cost < c).unwrap_or(true) {
                    best = Some((cost, ni, j + dj));
                }
            }
        }
        if let Some((_, ni, nj)) = best {
            while i < ni {
                ops.push(Op::Del(i));
                i += 1;
            }
            while j < nj {
                ops.push(Op::Add(j));
                j += 1;
            }
        } else {
            ops.push(Op::Del(i));
            ops.push(Op::Add(j));
            i += 1;
            j += 1;
        }
    }
    while i < m {
        ops.push(Op::Del(i));
        i += 1;
    }
    while j < n {
        ops.push(Op::Add(j));
        j += 1;
    }
    ops
}

fn build_hunks(ops: &[Op], left: &[String], right: &[String], context: usize) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let total = ops.len();
    let mut idx = 0usize;

    while idx < total {
        if matches!(ops[idx], Op::Eq(_, _)) {
            idx += 1;
            continue;
        }
        let start = idx.saturating_sub(context);
        let mut end = idx;
        while end < total {
            if !matches!(ops[end], Op::Eq(_, _)) {
                end += 1;
            } else {
                let mut look = end;
                let mut gap = 0usize;
                while look < total && matches!(ops[look], Op::Eq(_, _)) && gap <= context * 2 {
                    gap += 1;
                    look += 1;
                }
                if look < total && gap <= context * 2 {
                    end = look;
                } else {
                    break;
                }
            }
        }
        let finish = (end + context).min(total);

        let mut lines = Vec::with_capacity(finish - start);
        let mut left_start: Option<usize> = None;
        let mut right_start: Option<usize> = None;
        let mut left_count = 0usize;
        let mut right_count = 0usize;

        for op in &ops[start..finish] {
            match *op {
                Op::Eq(l, r) => {
                    left_start.get_or_insert(l + 1);
                    right_start.get_or_insert(r + 1);
                    left_count += 1;
                    right_count += 1;
                    lines.push(DiffLine {
                        kind: "context".to_string(),
                        left_line: Some(l + 1),
                        right_line: Some(r + 1),
                        text: left[l].clone(),
                    });
                }
                Op::Del(l) => {
                    left_start.get_or_insert(l + 1);
                    left_count += 1;
                    lines.push(DiffLine {
                        kind: "remove".to_string(),
                        left_line: Some(l + 1),
                        right_line: None,
                        text: left[l].clone(),
                    });
                }
                Op::Add(r) => {
                    right_start.get_or_insert(r + 1);
                    right_count += 1;
                    lines.push(DiffLine {
                        kind: "add".to_string(),
                        left_line: None,
                        right_line: Some(r + 1),
                        text: right[r].clone(),
                    });
                }
            }
        }

        hunks.push(DiffHunk {
            left_start: left_start.unwrap_or(1),
            left_count,
            right_start: right_start.unwrap_or(1),
            right_count,
            lines,
        });
        idx = finish;
    }

    hunks
}

/// Compute a structured line-by-line unified diff between two revisions of
/// `path`. Either `rev_a` or `rev_b` may be `"current"` to compare against the
/// live workspace file.
pub fn diff_revisions(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
    rev_a: &str,
    rev_b: &str,
) -> Result<RevisionDiff, String> {
    let (rel, _) = validate_history_path(workspace_dir, path)?;
    let clean_a = validate_rev_id(rev_a)?;
    let clean_b = validate_rev_id(rev_b)?;

    let bytes_a = read_revision(person_data_dir, workspace_dir, &rel, &clean_a)?;
    let bytes_b = read_revision(person_data_dir, workspace_dir, &rel, &clean_b)?;
    let sha256_a = sha256_hex(&bytes_a);
    let sha256_b = sha256_hex(&bytes_b);

    if !is_probably_text(&bytes_a) || !is_probably_text(&bytes_b) {
        let same = sha256_a == sha256_b;
        let unified = if same {
            format!("Binary files {rel}@{clean_a} and {rel}@{clean_b} are identical\n")
        } else {
            format!(
                "Binary files {rel}@{clean_a} ({} bytes) and {rel}@{clean_b} ({} bytes) differ\n",
                bytes_a.len(),
                bytes_b.len()
            )
        };
        return Ok(RevisionDiff {
            path: rel,
            rev_a: clean_a,
            rev_b: clean_b,
            sha256_a,
            sha256_b,
            is_binary: true,
            added: 0,
            removed: 0,
            unchanged: 0,
            similarity_pct: if same { 100 } else { 0 },
            hunks: Vec::new(),
            unified,
        });
    }

    let text_a = std::str::from_utf8(&bytes_a).unwrap_or("");
    let text_b = std::str::from_utf8(&bytes_b).unwrap_or("");
    let left = split_lines_normalized(text_a);
    let right = split_lines_normalized(text_b);
    let ops = compute_line_ops(&left, &right);

    let mut added = 0usize;
    let mut removed = 0usize;
    let mut unchanged = 0usize;
    for op in &ops {
        match op {
            Op::Eq(_, _) => unchanged += 1,
            Op::Del(_) => removed += 1,
            Op::Add(_) => added += 1,
        }
    }

    let max_lines = left.len().max(right.len());
    let similarity_pct = if max_lines == 0 {
        100u8
    } else {
        ((unchanged * 100) / max_lines).min(100) as u8
    };

    let hunks = build_hunks(&ops, &left, &right, 3);
    let mut unified = String::new();
    if !hunks.is_empty() {
        unified.push_str(&format!("--- {rel} ({clean_a})\n+++ {rel} ({clean_b})\n"));
        for hunk in &hunks {
            unified.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.left_start, hunk.left_count, hunk.right_start, hunk.right_count
            ));
            for line in &hunk.lines {
                let prefix = match line.kind.as_str() {
                    "add" => '+',
                    "remove" => '-',
                    _ => ' ',
                };
                unified.push(prefix);
                unified.push_str(&line.text);
                unified.push('\n');
            }
        }
    }

    Ok(RevisionDiff {
        path: rel,
        rev_a: clean_a,
        rev_b: clean_b,
        sha256_a,
        sha256_b,
        is_binary: false,
        added,
        removed,
        unchanged,
        similarity_pct,
        hunks,
        unified,
    })
}

/// Atomically restore `rev_id` back into `workspace_dir/<path>`.
/// Snapshots the existing live file first (as `"pre-restore"`) so every restore
/// is itself reversible, validates target integrity via SHA-256, and writes
/// atomically via `.tmp` + `rename`.
pub fn restore_revision(
    person_data_dir: &Path,
    workspace_dir: &Path,
    path: &str,
    rev_id: &str,
) -> Result<RestoreResult, String> {
    let (rel, full) = validate_history_path(workspace_dir, path)?;
    let clean_rev = validate_rev_id(rev_id)?;
    if clean_rev == CURRENT_SENTINEL {
        return Err("cannot restore 'current' — specify a historical revision id".to_string());
    }

    with_lock(|| {
        let bucket = bucket_dir(person_data_dir, &rel);
        let revs = load_manifest(&bucket);
        let meta = revs
            .iter()
            .find(|r| r.rev_id == clean_rev)
            .ok_or_else(|| format!("revision '{clean_rev}' not found for {rel}"))?
            .clone();

        let bytes = std::fs::read(blob_path(&bucket, &clean_rev))
            .map_err(|e| format!("read revision blob {clean_rev}: {e}"))?;
        let digest = sha256_hex(&bytes);
        if !meta.sha256.is_empty() && digest != meta.sha256 {
            return Err(format!(
                "revision '{clean_rev}' failed integrity check (expected {}, got {digest})",
                meta.sha256
            ));
        }

        // Snapshot current file before overwriting so restore is reversible.
        let pre_restore_rev_id = if full.is_file() {
            let cur_bytes = std::fs::read(&full)
                .map_err(|e| format!("read live {rel} before restore: {e}"))?;
            let pre = write_revision_unlocked(person_data_dir, &rel, &cur_bytes, "pre-restore")?;
            Some(pre.rev_id)
        } else {
            None
        };

        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir for restore: {e}"))?;
        }
        let seq = REV_SEQ.fetch_add(1, Ordering::SeqCst);
        let file_name = full
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("restore");
        let tmp = full.with_file_name(format!(".{file_name}.ergon-restore-{seq}.tmp"));
        std::fs::write(&tmp, &bytes).map_err(|e| format!("write restore temp file: {e}"))?;
        if let Err(e) = std::fs::rename(&tmp, &full) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("commit restored file {rel}: {e}"));
        }

        Ok(RestoreResult {
            path: rel,
            restored_rev_id: clean_rev,
            pre_restore_rev_id,
            size_bytes: bytes.len() as u64,
            sha256: digest,
        })
    })
}
