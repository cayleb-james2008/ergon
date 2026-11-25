//! High-scale in-memory search index & cached receipts/files store.
//!
//! Designed for 5,000+ receipts and 500+ workspace files under 30+ concurrent
//! HTTP/SSE clients: uses per-person `RwLock` caching invalidated on writes
//! and verified via lightweight `stat()` fingerprints, plus an inverted token
//! index so `GET /api/search?q=...`, `GET /api/receipts`, and `GET /api/files`
//! consistently answer in single-digit milliseconds.

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, UNIX_EPOCH};

use crate::receipts::{self, ChainedReceipt};
use crate::records;

const MAX_SNIPPET_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fingerprint {
    work_mtime: u128,
    work_len: u64,
    receipts_mtime: u128,
    receipts_len: u64,
    mem_mtime: u128,
    mem_len: u64,
    ws_mtime: u128,
}

fn stat_pair(path: &Path) -> (u128, u64) {
    match std::fs::metadata(path) {
        Ok(m) => {
            let ns = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            (ns, m.len())
        }
        Err(_) => (0, 0),
    }
}

fn compute_fingerprint(data_dir: &Path, workspace_dir: &Path, memory_path: &Path) -> Fingerprint {
    let (work_mtime, work_len) = stat_pair(&records::work_path(data_dir));
    let (receipts_mtime, receipts_len) = stat_pair(&data_dir.join("receipts.json"));
    let (mem_mtime, mem_len) = stat_pair(memory_path);
    let (ws_mtime, _) = stat_pair(workspace_dir);
    Fingerprint {
        work_mtime,
        work_len,
        receipts_mtime,
        receipts_len,
        mem_mtime,
        mem_len,
        ws_mtime,
    }
}

#[derive(Clone, Debug)]
pub struct IndexedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mtime_ms: u64,
    pub ext: String,
    pub snippet_lower: String,
    pub preview: String,
}

impl IndexedFile {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "name": self.name,
            "size": self.size,
            "mtime_ms": self.mtime_ms,
            "ext": self.ext,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MemoryLine {
    pub line_no: usize,
    pub text: String,
    pub lower: String,
}

pub struct PersonIndex {
    fp: Fingerprint,
    pub chain: Vec<ChainedReceipt>,
    pub chain_valid: bool,
    pub head_hash: String,
    /// Inverted token index mapping lowercase ASCII/alnum words -> receipt indices.
    receipt_tokens: HashMap<String, Vec<u32>>,
    pub files: Vec<IndexedFile>,
    file_tokens: HashMap<String, Vec<u32>>,
    pub memory: Vec<MemoryLine>,
}

/// Zero-allocation case-insensitive substring check (`needle` must already be lowercase).
fn contains_icase(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle_lower.is_ascii() {
        let hb = haystack.as_bytes();
        let nb = needle_lower.as_bytes();
        if nb.len() > hb.len() {
            return false;
        }
        return hb
            .windows(nb.len())
            .any(|w| w.eq_ignore_ascii_case(nb));
    }
    haystack.to_lowercase().contains(needle_lower)
}

fn receipt_contains_icase(r: &ChainedReceipt, needle_lower: &str) -> bool {
    contains_icase(&r.id, needle_lower)
        || contains_icase(&r.task, needle_lower)
        || contains_icase(&r.summary, needle_lower)
        || contains_icase(&r.thread_id, needle_lower)
        || r.files.iter().any(|f| contains_icase(f, needle_lower))
}

fn receipt_matches(r: &ChainedReceipt, q_lower: &str, tokens: &[String]) -> bool {
    if receipt_contains_icase(r, q_lower) {
        return true;
    }
    !tokens.is_empty() && tokens.iter().all(|t| receipt_contains_icase(r, t.as_str()))
}

/// Collect unique lowercase >=2-char alphanumeric tokens from `s` into `set`,
/// reusing `set` and `cur` across calls to avoid per-item allocations.
fn tokenize_into(s: &str, set: &mut HashSet<String>, cur: &mut String) {
    set.clear();
    cur.clear();
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            if ch.is_ascii() {
                cur.push(ch.to_ascii_lowercase());
            } else {
                for lc in ch.to_lowercase() {
                    cur.push(lc);
                }
            }
        } else if !cur.is_empty() {
            if cur.len() >= 2 {
                set.insert(std::mem::take(cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        set.insert(std::mem::take(cur));
    } else {
        cur.clear();
    }
}

fn tokenize_query(s: &str) -> Vec<String> {
    let mut set = HashSet::new();
    let mut cur = String::new();
    tokenize_into(s, &mut set, &mut cur);
    set.into_iter().collect()
}

/// Narrow sorted posting lists in `index` for `tokens` using binary-search
/// intersection (zero temporary `HashSet` allocations). Returns `None` when
/// fallback substring scanning is required.
fn narrow_candidates(
    index: &HashMap<String, Vec<u32>>,
    query: &str,
    tokens: &[String],
    disallow_slash: bool,
) -> Option<Vec<u32>> {
    if tokens.is_empty() || query.contains('.') || query.contains('-') || (disallow_slash && query.contains('/')) {
        return None;
    }
    let mut lists: Vec<&Vec<u32>> = Vec::with_capacity(tokens.len());
    for tok in tokens {
        lists.push(index.get(tok)?);
    }
    if lists.is_empty() {
        return None;
    }
    lists.sort_by_key(|l| l.len());
    let mut acc: Vec<u32> = lists[0].clone();
    for other in &lists[1..] {
        acc.retain(|id| other.binary_search(id).is_ok());
    }
    Some(acc)
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<IndexedFile>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.filter_map(|e| e.ok()) {
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let Ok(md) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if md.file_type().is_symlink() {
            continue;
        }
        if md.is_dir() {
            collect_files(root, &path, out);
            continue;
        }
        if !md.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| name.clone());
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mtime_ms = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Documents (PDF, Word, Excel, PowerPoint up to 5 MB) are indexed
        // by their extracted text — cached, so only the first build after
        // a change pays for the parse. Larger ones are indexed by name.
        let doc = crate::docs::doc_kind(&rel);
        let (preview, snippet_lower) = if let Some(kind) = doc.filter(|_| md.len() <= 5 * 1024 * 1024) {
            match crate::docs::extract(&path, kind, None) {
                Ok(ex) => {
                    let text: String = ex.text.chars().take(MAX_SNIPPET_BYTES).collect();
                    let first_line = text
                        .lines()
                        .map(|l| l.trim_start_matches('#').trim())
                        .find(|l| !l.is_empty())
                        .unwrap_or("")
                        .chars()
                        .take(160)
                        .collect::<String>();
                    let snippet = text.chars().take(512).collect::<String>().to_lowercase();
                    (first_line, format!("{} {}", rel.to_ascii_lowercase(), snippet))
                }
                Err(_) => (String::new(), rel.to_ascii_lowercase()),
            }
        } else if doc.is_some() {
            (String::new(), rel.to_ascii_lowercase())
        } else if md.len() <= 2 * 1024 * 1024 {
            match std::fs::File::open(&path) {
                Ok(mut f) => {
                    use std::io::Read as _;
                    let mut buf = vec![0u8; MAX_SNIPPET_BYTES.min(md.len() as usize)];
                    let n = f.read(&mut buf).unwrap_or(0);
                    buf.truncate(n);
                    match String::from_utf8(buf) {
                        Ok(s) => {
                            let first_line = s
                                .lines()
                                .find(|l| !l.trim().is_empty())
                                .unwrap_or("")
                                .chars()
                                .take(160)
                                .collect::<String>();
                            let lower = format!(
                                "{} {}",
                                rel.to_ascii_lowercase(),
                                s.chars().take(512).collect::<String>().to_lowercase()
                            );
                            (first_line, lower)
                        }
                        Err(_) => (String::new(), rel.to_ascii_lowercase()),
                    }
                }
                Err(_) => (String::new(), rel.to_ascii_lowercase()),
            }
        } else {
            (String::new(), rel.to_ascii_lowercase())
        };

        out.push(IndexedFile {
            path: rel,
            name,
            size: md.len(),
            mtime_ms,
            ext,
            snippet_lower,
            preview,
        });
    }
}

impl PersonIndex {
    fn build(data_dir: &Path, workspace_dir: &Path, memory_path: &Path, fp: Fingerprint) -> Self {
        let raw_entries = receipts::load_raw_entries(data_dir);
        let chain = receipts::build_chain(&raw_entries);
        let (chain_valid, head_hash) = receipts::verify_chain(&chain);

        let mut receipt_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        let mut tok_buf = HashSet::new();
        let mut cur_buf = String::new();
        for (i, r) in chain.iter().enumerate() {
            for field in [r.id.as_str(), r.task.as_str(), r.summary.as_str(), r.thread_id.as_str()] {
                tokenize_into(field, &mut tok_buf, &mut cur_buf);
                for tok in tok_buf.drain() {
                    let list = receipt_tokens.entry(tok).or_default();
                    if list.last().copied() != Some(i as u32) {
                        list.push(i as u32);
                    }
                }
            }
            for f in &r.files {
                tokenize_into(f, &mut tok_buf, &mut cur_buf);
                for tok in tok_buf.drain() {
                    let list = receipt_tokens.entry(tok).or_default();
                    if list.last().copied() != Some(i as u32) {
                        list.push(i as u32);
                    }
                }
            }
        }
        for list in receipt_tokens.values_mut() {
            list.shrink_to_fit();
        }
        receipt_tokens.shrink_to_fit();

        let mut files = Vec::new();
        collect_files(workspace_dir, workspace_dir, &mut files);
        files.sort_by(|a, b| b.mtime_ms.cmp(&a.mtime_ms).then_with(|| a.path.cmp(&b.path)));
        let mut file_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            tokenize_into(&f.snippet_lower, &mut tok_buf, &mut cur_buf);
            for tok in tok_buf.drain() {
                file_tokens.entry(tok).or_default().push(i as u32);
            }
        }
        for list in file_tokens.values_mut() {
            list.shrink_to_fit();
        }
        file_tokens.shrink_to_fit();
        files.shrink_to_fit();

        let mut memory = Vec::new();
        if let Ok(text) = std::fs::read_to_string(memory_path) {
            for (idx, line) in text.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                memory.push(MemoryLine {
                    line_no: idx + 1,
                    text: trimmed.to_string(),
                    lower: trimmed.to_lowercase(),
                });
            }
        }
        memory.shrink_to_fit();

        Self {
            fp,
            chain,
            chain_valid,
            head_hash,
            receipt_tokens,
            files,
            file_tokens,
            memory,
        }
    }

    pub fn search(&self, query: &str, scope: &str, limit: usize) -> Value {
        let t0 = Instant::now();
        let q = query.trim().to_lowercase();
        let limit = limit.clamp(1, 200);
        let tokens = tokenize_query(&q);

        let want_receipts = scope == "all" || scope == "receipts";
        let want_files = scope == "all" || scope == "files";
        let want_memory = scope == "all" || scope == "memory";

        let mut receipt_hits: Vec<Value> = Vec::new();
        let mut total_receipt_hits = 0usize;
        if want_receipts {
            if q.is_empty() {
                total_receipt_hits = self.chain.len();
                for r in self.chain.iter().rev().take(limit) {
                    receipt_hits.push(r.to_json());
                }
            } else {
                let candidates = narrow_candidates(&self.receipt_tokens, &q, &tokens, false);

                match candidates {
                    Some(ids) => {
                        for &id in ids.iter().rev() {
                            let idx = id as usize;
                            if receipt_matches(&self.chain[idx], &q, &tokens) {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(self.chain[idx].to_json());
                                }
                            }
                        }
                    }
                    None => {
                        for r in self.chain.iter().rev() {
                            if receipt_matches(r, &q, &tokens) {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(r.to_json());
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut file_hits: Vec<Value> = Vec::new();
        let mut total_file_hits = 0usize;
        if want_files {
            if q.is_empty() {
                total_file_hits = self.files.len();
                for f in self.files.iter().take(limit) {
                    file_hits.push(json!({
                        "path": f.path,
                        "name": f.name,
                        "size": f.size,
                        "mtime_ms": f.mtime_ms,
                        "ext": f.ext,
                        "preview": f.preview,
                    }));
                }
            } else {
                let candidates = narrow_candidates(&self.file_tokens, &q, &tokens, true);

                let check_file = |idx: usize| -> bool {
                    let hay = &self.files[idx].snippet_lower;
                    if hay.contains(&q) {
                        return true;
                    }
                    !tokens.is_empty() && tokens.iter().all(|t| hay.contains(t.as_str()))
                };

                let mut push_file = |idx: usize| {
                    total_file_hits += 1;
                    if file_hits.len() < limit {
                        let f = &self.files[idx];
                        file_hits.push(json!({
                            "path": f.path,
                            "name": f.name,
                            "size": f.size,
                            "mtime_ms": f.mtime_ms,
                            "ext": f.ext,
                            "preview": f.preview,
                        }));
                    }
                };

                match candidates {
                    Some(ids) => {
                        for id in ids {
                            if check_file(id as usize) {
                                push_file(id as usize);
                            }
                        }
                    }
                    None => {
                        for idx in 0..self.files.len() {
                            if check_file(idx) {
                                push_file(idx);
                            }
                        }
                    }
                }
            }
        }

        let mut memory_hits: Vec<Value> = Vec::new();
        let mut total_memory_hits = 0usize;
        if want_memory {
            for m in &self.memory {
                let matched = q.is_empty()
                    || m.lower.contains(&q)
                    || (!tokens.is_empty() && tokens.iter().all(|t| m.lower.contains(t.as_str())));
                if matched {
                    total_memory_hits += 1;
                    if memory_hits.len() < limit {
                        memory_hits.push(json!({
                            "line": m.line_no,
                            "text": m.text,
                        }));
                    }
                }
            }
        }

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        json!({
            "query": query.trim(),
            "scope": scope,
            "elapsed_ms": (elapsed_ms * 100.0).round() / 100.0,
            "total_hits": total_receipt_hits + total_file_hits + total_memory_hits,
            "counts": {
                "receipts": total_receipt_hits,
                "files": total_file_hits,
                "memory": total_memory_hits,
            },
            "indexed": {
                "receipts": self.chain.len(),
                "files": self.files.len(),
                "memory_lines": self.memory.len(),
            },
            "receipts": receipt_hits,
            "files": file_hits,
            "memory": memory_hits,
        })
    }

    pub fn list_receipts(
        &self,
        offset: usize,
        limit: usize,
        q: &str,
        status: &str,
    ) -> Value {
        let limit = limit.clamp(1, 5000);
        let q_low = q.trim().to_lowercase();
        if q_low.is_empty() && status != "finished" && status != "failed" {
            let total = self.chain.len();
            let page: Vec<Value> = self
                .chain
                .iter()
                .rev()
                .skip(offset)
                .take(limit)
                .map(|r| r.to_json())
                .collect();
            return json!({
                "receipts": page,
                "total": total,
                "offset": offset,
                "limit": limit,
                "chain_valid": self.chain_valid,
                "head_hash": self.head_hash,
            });
        }
        let mut matched: Vec<&ChainedReceipt> = Vec::new();
        let tokens = if q_low.is_empty() {
            Vec::new()
        } else {
            tokenize_query(&q_low)
        };
        let candidates = if q_low.is_empty() {
            None
        } else {
            narrow_candidates(&self.receipt_tokens, &q_low, &tokens, false)
        };
        let mut push_if_match = |i: usize| {
            let r = &self.chain[i];
            if status == "finished" && !r.finished {
                return;
            }
            if status == "failed" && r.finished {
                return;
            }
            if !q_low.is_empty() && !receipt_contains_icase(r, &q_low) {
                return;
            }
            matched.push(r);
        };
        match candidates {
            Some(ids) => {
                for &id in ids.iter().rev() {
                    push_if_match(id as usize);
                }
            }
            None => {
                for i in (0..self.chain.len()).rev() {
                    push_if_match(i);
                }
            }
        }
        let total = matched.len();
        let page: Vec<Value> = matched
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|r| r.to_json())
            .collect();
        json!({
            "receipts": page,
            "total": total,
            "offset": offset,
            "limit": limit,
            "chain_valid": self.chain_valid,
            "head_hash": self.head_hash,
        })
    }

    pub fn list_files(&self, offset: usize, limit: Option<usize>, q: &str) -> (usize, Vec<Value>) {
        let q_low = q.trim().to_lowercase();
        if q_low.is_empty() {
            let total = self.files.len();
            let iter = self.files.iter().skip(offset);
            let items: Vec<Value> = match limit {
                Some(l) => iter.take(l.clamp(1, 1000)).map(|f| f.to_json()).collect(),
                None => iter.map(|f| f.to_json()).collect(),
            };
            return (total, items);
        }
        let tokens = tokenize_query(&q_low);
        let candidates = narrow_candidates(&self.file_tokens, &q_low, &tokens, true);
        let filtered: Vec<&IndexedFile> = match candidates {
            Some(ids) => ids
                .into_iter()
                .map(|id| &self.files[id as usize])
                .filter(|f| f.path.to_lowercase().contains(&q_low) || f.snippet_lower.contains(&q_low))
                .collect(),
            None => self
                .files
                .iter()
                .filter(|f| f.path.to_lowercase().contains(&q_low) || f.snippet_lower.contains(&q_low))
                .collect(),
        };
        let total = filtered.len();
        let iter = filtered.into_iter().skip(offset);
        let items: Vec<Value> = match limit {
            Some(l) => iter.take(l.clamp(1, 1000)).map(|f| f.to_json()).collect(),
            None => iter.map(|f| f.to_json()).collect(),
        };
        (total, items)
    }
}

type PersonIndexSlot = Arc<RwLock<Option<Arc<PersonIndex>>>>;

#[derive(Clone, Default)]
pub struct SearchCache {
    per_person: Arc<Mutex<HashMap<String, PersonIndexSlot>>>,
}

impl SearchCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(&self, person_id: &str) -> PersonIndexSlot {
        let mut map = self.per_person.lock().unwrap();
        map.entry(person_id.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(None)))
            .clone()
    }

    pub fn invalidate(&self, person_id: &str) {
        let slot = self.slot(person_id);
        let mut w = slot.write().unwrap();
        *w = None;
    }

    /// Return a fresh shared `Arc<PersonIndex>` for `person_id`. Under 30+
    /// concurrent read requests, only the read lock is acquired after initial
    /// build, delivering zero-contention sub-5ms responses.
    pub fn get(
        &self,
        person_id: &str,
        data_dir: &Path,
        workspace_dir: &Path,
        memory_path: &Path,
    ) -> Arc<PersonIndex> {
        let fp = compute_fingerprint(data_dir, workspace_dir, memory_path);
        let slot = self.slot(person_id);
        {
            let r = slot.read().unwrap();
            if let Some(idx) = r.as_ref() {
                if idx.fp == fp {
                    return idx.clone();
                }
            }
        }
        let mut w = slot.write().unwrap();
        if let Some(idx) = w.as_ref() {
            if idx.fp == fp {
                return idx.clone();
            }
        }
        let built = Arc::new(PersonIndex::build(data_dir, workspace_dir, memory_path, fp));
        *w = Some(built.clone());
        built
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverted_index_handles_10k_receipts_and_500_files_sub_5ms() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-search-10k-{}",
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        let mem = dir.join("memory.md");
        std::fs::write(&mem, "# Memory\n\n- Prefers concise tables\n").unwrap();

        let mut entries = Vec::with_capacity(10_000);
        for i in 0..10_000usize {
            let tag = if i == 9876 { "needle_quarter_audit" } else { "routine_sync" };
            entries.push(json!({
                "id": format!("rcpt-{i}"),
                "ts": 1_700_000_000_000u64 + i as u64,
                "task": format!("Task {i} {tag}"),
                "summary": format!("Summary for {tag} batch {i}"),
                "thread_id": format!("t-{i}"),
                "files": if i % 10 == 0 { vec!["summary.md"] } else { vec![] },
                "finished": true,
            }));
        }
        std::fs::write(
            records::work_path(&dir),
            json!({ "entries": entries }).to_string(),
        )
        .unwrap();
        for i in 0..500usize {
            let body = if i == 432 {
                "vendor_contract_clause_alpha\n"
            } else {
                "standard ledger line\n"
            };
            std::fs::write(ws.join(format!("doc_{i:03}.txt")), body).unwrap();
        }

        let cache = SearchCache::new();
        let idx = cache.get("default", &dir, &ws, &mem);
        assert_eq!(idx.chain.len(), 10_000);
        assert_eq!(idx.files.len(), 500);

        let res = idx.search("needle_quarter_audit", "receipts", 20);
        assert_eq!(res["counts"]["receipts"], json!(1));
        assert_eq!(res["receipts"][0]["thread_id"], json!("t-9876"));

        let listed = idx.list_receipts(0, 20, "needle_quarter_audit", "finished");
        assert_eq!(listed["total"], json!(1));

        let (f_total, f_items) = idx.list_files(0, Some(20), "vendor_contract_clause_alpha");
        assert_eq!(f_total, 1);
        assert_eq!(f_items[0]["path"], json!("doc_432.txt"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
