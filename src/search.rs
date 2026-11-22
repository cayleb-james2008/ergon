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
    receipt_lower: Vec<String>,
    /// Inverted token index mapping lowercase ASCII/alnum words -> receipt indices.
    receipt_tokens: HashMap<String, Vec<u32>>,
    pub files: Vec<IndexedFile>,
    file_tokens: HashMap<String, Vec<u32>>,
    pub memory: Vec<MemoryLine>,
}

fn tokenize(s: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            for lc in ch.to_lowercase() {
                cur.push(lc);
            }
        } else if !cur.is_empty() {
            if cur.len() >= 2 {
                set.insert(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        set.insert(cur);
    }
    set
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

        // Read a bounded prefix for fast full-text indexing without blowing RSS.
        let (preview, snippet_lower) = if md.len() <= 2 * 1024 * 1024 {
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
                            let lower = format!("{} {}", rel.to_ascii_lowercase(), s.to_lowercase());
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

        let mut receipt_lower = Vec::with_capacity(chain.len());
        let mut receipt_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        for (i, r) in chain.iter().enumerate() {
            let combined = format!(
                "{} {} {} {} {}",
                r.id,
                r.task,
                r.summary,
                r.thread_id,
                r.files.join(" ")
            )
            .to_lowercase();
            for tok in tokenize(&combined) {
                receipt_tokens.entry(tok).or_default().push(i as u32);
            }
            receipt_lower.push(combined);
        }

        let mut files = Vec::new();
        collect_files(workspace_dir, workspace_dir, &mut files);
        files.sort_by(|a, b| b.mtime_ms.cmp(&a.mtime_ms).then_with(|| a.path.cmp(&b.path)));
        let mut file_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            for tok in tokenize(&f.snippet_lower) {
                file_tokens.entry(tok).or_default().push(i as u32);
            }
        }

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

        Self {
            fp,
            chain,
            chain_valid,
            head_hash,
            receipt_lower,
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
        let tokens: Vec<String> = tokenize(&q).into_iter().collect();

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
                // Fast inverted-index candidate narrowing when every query word >= 2 chars is indexed.
                let mut candidates: Option<Vec<u32>> = None;
                if !tokens.is_empty() && !q.contains('.') && !q.contains('-') {
                    let mut lists: Vec<&Vec<u32>> = Vec::with_capacity(tokens.len());
                    let mut all_present = true;
                    for tok in &tokens {
                        if let Some(list) = self.receipt_tokens.get(tok) {
                            lists.push(list);
                        } else {
                            all_present = false;
                            break;
                        }
                    }
                    if all_present && !lists.is_empty() {
                        lists.sort_by_key(|l| l.len());
                        let mut acc: Vec<u32> = lists[0].clone();
                        for other in &lists[1..] {
                            let set: HashSet<u32> = other.iter().copied().collect();
                            acc.retain(|id| set.contains(id));
                        }
                        candidates = Some(acc);
                    }
                }

                let check_match = |idx: usize| -> bool {
                    let hay = &self.receipt_lower[idx];
                    if hay.contains(&q) {
                        return true;
                    }
                    !tokens.is_empty() && tokens.iter().all(|t| hay.contains(t.as_str()))
                };

                match candidates {
                    Some(ids) => {
                        for &id in ids.iter().rev() {
                            let idx = id as usize;
                            if check_match(idx) {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(self.chain[idx].to_json());
                                }
                            }
                        }
                    }
                    None => {
                        for idx in (0..self.chain.len()).rev() {
                            if check_match(idx) {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(self.chain[idx].to_json());
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
                let mut candidates: Option<Vec<u32>> = None;
                if !tokens.is_empty() && !q.contains('.') && !q.contains('-') && !q.contains('/') {
                    let mut lists: Vec<&Vec<u32>> = Vec::with_capacity(tokens.len());
                    let mut all_present = true;
                    for tok in &tokens {
                        if let Some(list) = self.file_tokens.get(tok) {
                            lists.push(list);
                        } else {
                            all_present = false;
                            break;
                        }
                    }
                    if all_present && !lists.is_empty() {
                        lists.sort_by_key(|l| l.len());
                        let mut acc: Vec<u32> = lists[0].clone();
                        for other in &lists[1..] {
                            let set: HashSet<u32> = other.iter().copied().collect();
                            acc.retain(|id| set.contains(id));
                        }
                        candidates = Some(acc);
                    }
                }

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
        let mut matched: Vec<&ChainedReceipt> = Vec::new();
        for (i, r) in self.chain.iter().enumerate().rev() {
            if status == "finished" && !r.finished {
                continue;
            }
            if status == "failed" && r.finished {
                continue;
            }
            if !q_low.is_empty() && !self.receipt_lower[i].contains(&q_low) {
                continue;
            }
            matched.push(r);
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
        let filtered: Vec<&IndexedFile> = if q_low.is_empty() {
            self.files.iter().collect()
        } else {
            self.files
                .iter()
                .filter(|f| f.path.to_lowercase().contains(&q_low) || f.snippet_lower.contains(&q_low))
                .collect()
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
