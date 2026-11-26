//! High-scale in-memory & persistent chunked vault search index.
//!
//! Designed for 5,000+ receipts and 500+ workspace files under 30+ concurrent
//! HTTP/SSE clients: uses per-person `RwLock` caching invalidated on writes
//! and verified via lightweight `stat()` fingerprints, persistent incremental
//! `<person_data_dir>/vault_index.json` keyed by `(path, mtime_ns, sha256)`,
//! full-document chunking (zero 512-char cutoff, all `.xlsx` sheets, `.pdf`
//! pages, `.pptx` slides, `.docx` sections, and text/CSV/Markdown files),
//! BM25 + semantic cosine similarity fused via Reciprocal Rank Fusion (RRF),
//! and zero-DOM streaming JSON serialization.

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant, UNIX_EPOCH};

use crate::docs::{self, DocKind};
use crate::receipts::{self, ChainedReceipt};
use crate::records;

const EMBED_DIM: usize = 256;
const SEMANTIC_MIN_THRESHOLD: f64 = 0.25;
const RRF_K: f64 = 60.0;
const MAX_CHUNK_CHARS: usize = 1000;
const MIN_SPLIT_CHARS: usize = 350;
const OVERLAP_CHARS: usize = 120;
const MAX_PLAIN_TEXT_BYTES: u64 = 5 * 1024 * 1024;
const MAX_VAULT_INDEX_BYTES: u64 = 128 * 1024 * 1024;
const MAX_EXTRACTED_TEXT_BYTES: usize = 2 * 1024 * 1024;
const MAX_CHUNKS_PER_FILE: usize = 1000;
const VAULT_INDEX_VERSION: u64 = 1;
const VAULT_INDEX_FILE: &str = "vault_index.json";

#[inline]
fn trim_allocator_memory() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        extern "C" {
            fn malloc_trim(pad: usize) -> std::os::raw::c_int;
        }
        unsafe {
            let _ = malloc_trim(0);
        }
    }
}

fn cap_utf8_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fingerprint {
    work_mtime: u128,
    work_len: u64,
    receipts_mtime: u128,
    receipts_len: u64,
    mem_mtime: u128,
    mem_len: u64,
    ws_mtime: u128,
    ws_meta_hash: u64,
    cfg_mtime: u128,
    clients_mtime: u128,
    clients_len: u64,
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

fn fnv1a_64(bytes: &[u8], seed: u64) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ seed;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn ws_metadata_hash_walk(root: &Path, dir: &Path, hash: &mut u64) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.filter_map(|e| e.ok()) {
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy();
        if (name.starts_with('.') && name != ".ergon-scope.json")
            || name.eq_ignore_ascii_case("connector_secrets.json")
        {
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
            ws_metadata_hash_walk(root, &path, hash);
            continue;
        }
        if !md.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| name.into_owned());
        let ns = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // Order-independent XOR mix over per-file FNV-1a digests so directory
        // traversal order never affects fingerprint stability.
        let mut fh = fnv1a_64(rel.as_bytes(), 0);
        fh = fnv1a_64(&md.len().to_le_bytes(), fh);
        fh = fnv1a_64(&ns.to_le_bytes(), fh);
        *hash ^= fh;
    }
}

fn compute_fingerprint(data_dir: &Path, workspace_dir: &Path, memory_path: &Path) -> Fingerprint {
    let (work_mtime, work_len) = stat_pair(&records::work_path(data_dir));
    let (receipts_mtime, receipts_len) = stat_pair(&data_dir.join("receipts.json"));
    let (mem_mtime, mem_len) = stat_pair(memory_path);
    let (ws_mtime, _) = stat_pair(workspace_dir);
    let (cfg_mtime, _) = stat_pair(&crate::config_file_path(data_dir));
    let root_data_dir = records::infer_root_data_dir(data_dir);
    let (clients_mtime, clients_len) = stat_pair(&records::clients_path(&root_data_dir));
    let mut ws_meta_hash = 0u64;
    ws_metadata_hash_walk(workspace_dir, workspace_dir, &mut ws_meta_hash);
    Fingerprint {
        work_mtime,
        work_len,
        receipts_mtime,
        receipts_len,
        mem_mtime,
        mem_len,
        ws_mtime,
        ws_meta_hash,
        cfg_mtime,
        clients_mtime,
        clients_len,
    }
}

#[inline]
pub(crate) fn write_json_str(out: &mut Vec<u8>, s: &str) {
    out.push(b'"');
    let bytes = s.as_bytes();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let esc = match b {
            b'"' => b"\\\"".as_slice(),
            b'\\' => b"\\\\".as_slice(),
            0x08 => b"\\b".as_slice(),
            0x0c => b"\\f".as_slice(),
            b'\n' => b"\\n".as_slice(),
            b'\r' => b"\\r".as_slice(),
            b'\t' => b"\\t".as_slice(),
            0..=0x1f => {
                if start < i {
                    out.extend_from_slice(&bytes[start..i]);
                }
                const HEX: &[u8; 16] = b"0123456789abcdef";
                out.extend_from_slice(&[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX[(b >> 4) as usize],
                    HEX[(b & 0x0f) as usize],
                ]);
                start = i + 1;
                continue;
            }
            _ => continue,
        };
        if start < i {
            out.extend_from_slice(&bytes[start..i]);
        }
        out.extend_from_slice(esc);
        start = i + 1;
    }
    if start < bytes.len() {
        out.extend_from_slice(&bytes[start..]);
    }
    out.push(b'"');
}

#[inline]
pub(crate) fn write_json_u64(out: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        out.push(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut idx = buf.len();
    while n > 0 {
        idx -= 1;
        buf[idx] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out.extend_from_slice(&buf[idx..]);
}

#[inline]
pub(crate) fn write_json_bool(out: &mut Vec<u8>, v: bool) {
    out.extend_from_slice(if v { b"true" } else { b"false" });
}

#[inline]
fn write_json_f64(out: &mut Vec<u8>, v: f64) {
    use std::io::Write as _;
    let clean = if v.is_finite() { v } else { 0.0 };
    let rounded = (clean * 10_000.0).round() / 10_000.0;
    let _ = write!(out, "{rounded}");
}

fn write_receipt_json(out: &mut Vec<u8>, r: &ChainedReceipt) {
    out.extend_from_slice(br#"{"id":"#);
    write_json_str(out, &r.id);
    out.extend_from_slice(br#","index":"#);
    write_json_u64(out, r.index as u64);
    out.extend_from_slice(br#","ts":"#);
    write_json_u64(out, r.ts);
    out.extend_from_slice(br#","task":"#);
    write_json_str(out, &r.task);
    out.extend_from_slice(br#","summary":"#);
    write_json_str(out, &r.summary);
    out.extend_from_slice(br#","files":["#);
    for (i, f) in r.files.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        write_json_str(out, f);
    }
    out.extend_from_slice(br#"],"thread_id":"#);
    write_json_str(out, &r.thread_id);
    out.extend_from_slice(br#","finished":"#);
    write_json_bool(out, r.finished);
    out.extend_from_slice(br#","answered":"#);
    write_json_bool(out, r.answered);
    out.extend_from_slice(br#","prev_hash":"#);
    write_json_str(out, &r.prev_hash);
    out.extend_from_slice(br#","sha256":"#);
    write_json_str(out, &r.sha256);
    if let Some(eid) = r
        .engagement_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        out.extend_from_slice(br#","engagement_id":"#);
        write_json_str(out, eid);
    }
    if let Some(u) = r.usage.as_ref().filter(|u| !u.is_null()) {
        out.extend_from_slice(br#","usage":"#);
        out.extend_from_slice(u.to_string().as_bytes());
    }
    if let Some(digest) = r.usage_digest() {
        out.extend_from_slice(br#","usage_digest":"#);
        write_json_str(out, &digest);
    }
    out.push(b'}');
}

fn write_file_summary_json(out: &mut Vec<u8>, f: &IndexedFile) {
    out.extend_from_slice(br#"{"path":"#);
    write_json_str(out, &f.path);
    out.extend_from_slice(br#","name":"#);
    write_json_str(out, &f.name);
    out.extend_from_slice(br#","size":"#);
    write_json_u64(out, f.size);
    out.extend_from_slice(br#","mtime_ms":"#);
    write_json_u64(out, f.mtime_ms);
    out.extend_from_slice(br#","ext":"#);
    write_json_str(out, &f.ext);
    if let Some(cid) = f.client_id.as_deref() {
        out.extend_from_slice(br#","client_id":"#);
        write_json_str(out, cid);
    }
    if let Some(eid) = f.engagement_id.as_deref() {
        out.extend_from_slice(br#","engagement_id":"#);
        write_json_str(out, eid);
    }
    out.push(b'}');
}

fn write_chunk_hit_json(out: &mut Vec<u8>, hit: &ChunkHit<'_>) {
    out.extend_from_slice(br#"{"chunk_id":"#);
    write_json_str(out, &hit.chunk.chunk_id);
    out.extend_from_slice(br#","score":"#);
    write_json_f64(out, hit.score);
    out.extend_from_slice(br#","bm25_score":"#);
    write_json_f64(out, hit.bm25_score);
    out.extend_from_slice(br#","semantic_score":"#);
    write_json_f64(out, hit.semantic_score);
    if let Some(page) = hit.chunk.page {
        out.extend_from_slice(br#","page":"#);
        write_json_u64(out, page as u64);
    }
    if let Some(sheet) = hit.chunk.sheet.as_deref() {
        out.extend_from_slice(br#","sheet":"#);
        write_json_str(out, sheet);
    }
    out.extend_from_slice(br#","line_start":"#);
    write_json_u64(out, hit.chunk.line_start as u64);
    out.extend_from_slice(br#","line_end":"#);
    write_json_u64(out, hit.chunk.line_end as u64);
    if let Some(heading) = hit.chunk.heading.as_deref() {
        out.extend_from_slice(br#","heading":"#);
        write_json_str(out, heading);
    }
    if let Some(cid) = hit.chunk.client_id.as_deref() {
        out.extend_from_slice(br#","client_id":"#);
        write_json_str(out, cid);
    }
    if let Some(eid) = hit.chunk.engagement_id.as_deref() {
        out.extend_from_slice(br#","engagement_id":"#);
        write_json_str(out, eid);
    }
    out.extend_from_slice(br#","excerpt":"#);
    write_json_str(out, &hit.excerpt);
    out.push(b'}');
}

fn write_file_hit_json(
    out: &mut Vec<u8>,
    f: &IndexedFile,
    score: f64,
    chunks: &[ChunkHit<'_>],
) {
    out.extend_from_slice(br#"{"path":"#);
    write_json_str(out, &f.path);
    out.extend_from_slice(br#","name":"#);
    write_json_str(out, &f.name);
    out.extend_from_slice(br#","size":"#);
    write_json_u64(out, f.size);
    out.extend_from_slice(br#","mtime_ms":"#);
    write_json_u64(out, f.mtime_ms);
    out.extend_from_slice(br#","ext":"#);
    write_json_str(out, &f.ext);
    if let Some(cid) = f.client_id.as_deref() {
        out.extend_from_slice(br#","client_id":"#);
        write_json_str(out, cid);
    }
    if let Some(eid) = f.engagement_id.as_deref() {
        out.extend_from_slice(br#","engagement_id":"#);
        write_json_str(out, eid);
    }
    out.extend_from_slice(br#","preview":"#);
    write_json_str(out, &f.preview);
    out.extend_from_slice(br#","score":"#);
    write_json_f64(out, score);
    out.extend_from_slice(br#","chunks":["#);
    for (i, c) in chunks.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        write_chunk_hit_json(out, c);
    }
    out.extend_from_slice(b"]}");
}

// ---------------------------------------------------------------------------
// Embeddings (optional OpenAI-compatible + pure-Rust deterministic fallback)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
struct EmbeddingConfig {
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl EmbeddingConfig {
    fn signature(&self) -> String {
        format!("remote:{}:{}", self.base_url, self.model)
    }
}

fn load_embedding_config(data_dir: &Path) -> Option<EmbeddingConfig> {
    let candidates = [
        Some(crate::config_file_path(data_dir)),
        data_dir
            .parent()
            .and_then(|p| p.parent())
            .map(crate::config_file_path),
    ];
    for cand in candidates.into_iter().flatten() {
        let Ok(raw) = std::fs::read_to_string(&cand) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let base_url = v["embedding_base_url"]
            .as_str()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty());
        let Some(base_url) = base_url else {
            continue;
        };
        let policy = crate::load_workspace_policy(data_dir);
        if (policy.block_outbound_fetch || policy.profile == "air_gapped")
            && !crate::is_loopback_url(&base_url)
        {
            return None;
        }
        let model = v["embedding_model"]
            .as_str()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "text-embedding-3-small".to_string());
        let api_key = v["embedding_api_key"]
            .as_str()
            .or_else(|| v["api_key"].as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(EmbeddingConfig {
            base_url,
            model,
            api_key,
        });
    }
    None
}

fn l2_normalize(vec: &mut [f32]) {
    let norm_sq: f64 = vec.iter().map(|&x| (x as f64) * (x as f64)).sum();
    if norm_sq > 0.0 {
        let inv = (1.0 / norm_sq.sqrt()) as f32;
        for x in vec.iter_mut() {
            *x *= inv;
        }
    }
}

/// Pure-Rust deterministic hashed word + char n-gram dense vector embedder.
/// Works 100% offline/air-gapped with zero external runtime dependencies.
pub fn deterministic_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; EMBED_DIM];
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
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
                words.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        words.push(cur);
    }

    let mut add_feat = |tag: u8, bytes: &[u8], weight: f32| {
        let h = fnv1a_64(bytes, tag as u64);
        let idx = (h as usize) % EMBED_DIM;
        let sign = if ((h >> 32) & 1) == 0 { 1.0 } else { -1.0 };
        vec[idx] += weight * sign;
    };

    for (i, w) in words.iter().enumerate() {
        add_feat(1, w.as_bytes(), 1.0);
        if i + 1 < words.len() {
            let mut bigram = Vec::with_capacity(w.len() + 1 + words[i + 1].len());
            bigram.extend_from_slice(w.as_bytes());
            bigram.push(b' ');
            bigram.extend_from_slice(words[i + 1].as_bytes());
            add_feat(2, &bigram, 0.8);
        }
        let wb = w.as_bytes();
        if wb.len() >= 3 {
            for win in wb.windows(3) {
                add_feat(3, win, 0.45);
            }
        }
        if wb.len() >= 4 {
            for win in wb.windows(4) {
                add_feat(4, win, 0.55);
            }
        }
    }

    l2_normalize(&mut vec);
    vec
}

fn fetch_remote_embeddings(cfg: &EmbeddingConfig, inputs: &[&str]) -> Option<Vec<Vec<f32>>> {
    if inputs.is_empty() {
        return Some(Vec::new());
    }
    let url = format!("{}/embeddings", cfg.base_url);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();
    let mut out: Vec<Vec<f32>> = Vec::with_capacity(inputs.len());
    for batch in inputs.chunks(32) {
        let mut req = agent.post(&url);
        if let Some(key) = &cfg.api_key {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
        let body = json!({
            "model": cfg.model,
            "input": batch,
        });
        let resp = req.send_json(body).ok()?;
        let val: Value = resp.into_json().ok()?;
        let data = val["data"].as_array()?;
        if data.len() != batch.len() {
            return None;
        }
        let mut items: Vec<(usize, Vec<f32>)> = Vec::with_capacity(data.len());
        for (fallback_idx, item) in data.iter().enumerate() {
            let idx = item["index"].as_u64().unwrap_or(fallback_idx as u64) as usize;
            let emb_arr = item["embedding"].as_array()?;
            let mut vec: Vec<f32> = Vec::with_capacity(emb_arr.len());
            for v in emb_arr {
                let f = v.as_f64()? as f32;
                vec.push(if f.is_finite() { f } else { 0.0 });
            }
            if vec.is_empty() {
                return None;
            }
            l2_normalize(&mut vec);
            items.push((idx, vec));
        }
        items.sort_by_key(|(idx, _)| *idx);
        for (_, v) in items {
            out.push(v);
        }
    }
    Some(out)
}

#[inline]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f64;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += (x as f64) * (y as f64);
    }
    dot.clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------------------
// Data Model, Client/Engagement Scope Resolution & Chunking
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopeTag {
    pub client_id: Option<String>,
    pub engagement_id: Option<String>,
}

impl ScopeTag {
    fn from_json(v: &Value) -> Self {
        let client_id = v["client_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let engagement_id = v["engagement_id"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        Self {
            client_id,
            engagement_id,
        }
    }

    pub fn matches_filter(
        &self,
        client_filter: Option<&str>,
        engagement_filter: Option<&str>,
    ) -> bool {
        let eng_f = engagement_filter.map(str::trim).filter(|s| !s.is_empty());
        let cli_f = client_filter.map(str::trim).filter(|s| !s.is_empty());
        if let Some(eid) = eng_f {
            if self.engagement_id.as_deref() != Some(eid) {
                return false;
            }
        }
        if let Some(cid) = cli_f {
            if self.client_id.as_deref() != Some(cid) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct ScopeResolver {
    pub store: records::ClientsStore,
    dir_scopes: HashMap<String, ScopeTag>,
    file_scopes: HashMap<String, ScopeTag>,
    prefix_scopes: Vec<(String, ScopeTag)>,
}

impl ScopeResolver {
    pub fn load(root_data_dir: &Path, workspace_dir: &Path) -> Self {
        let store = records::load_clients_store(root_data_dir);
        let mut dir_scopes = HashMap::new();
        let mut file_scopes = HashMap::new();
        let mut prefix_scopes = Vec::new();

        fn walk_scopes(
            root: &Path,
            dir: &Path,
            depth: usize,
            dir_scopes: &mut HashMap<String, ScopeTag>,
            file_scopes: &mut HashMap<String, ScopeTag>,
            prefix_scopes: &mut Vec<(String, ScopeTag)>,
        ) {
            if depth > 8 {
                return;
            }
            let scope_file = dir.join(".ergon-scope.json");
            if let Ok(md) = std::fs::symlink_metadata(&scope_file) {
                if !md.file_type().is_symlink() && md.is_file() && md.len() <= 256 * 1024 {
                    if let Ok(text) = std::fs::read_to_string(&scope_file) {
                        if let Ok(v) = serde_json::from_str::<Value>(&text) {
                            let rel_dir = dir
                                .strip_prefix(root)
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_default()
                                .trim_matches('/')
                                .to_string();
                            let tag = ScopeTag::from_json(&v);
                            if tag.client_id.is_some() || tag.engagement_id.is_some() {
                                dir_scopes.insert(rel_dir.clone(), tag);
                            }
                            if rel_dir.is_empty() {
                                if let Some(obj) = v["files"].as_object() {
                                    for (k, fv) in obj {
                                        let rel = k.trim().replace('\\', "/").trim_start_matches("./").to_string();
                                        let ftag = ScopeTag::from_json(fv);
                                        if !rel.is_empty() && (ftag.client_id.is_some() || ftag.engagement_id.is_some()) {
                                            file_scopes.insert(rel, ftag);
                                        }
                                    }
                                }
                                let pfx_obj = v["prefixes"]
                                    .as_object()
                                    .or_else(|| v["paths"].as_object());
                                if let Some(obj) = pfx_obj {
                                    for (k, pv) in obj {
                                        let pfx = k.trim().replace('\\', "/").trim_matches('/').to_string();
                                        let ptag = ScopeTag::from_json(pv);
                                        if !pfx.is_empty() && (ptag.client_id.is_some() || ptag.engagement_id.is_some()) {
                                            prefix_scopes.push((pfx, ptag));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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
                if ft.is_symlink() || !ft.is_dir() {
                    continue;
                }
                walk_scopes(root, &entry.path(), depth + 1, dir_scopes, file_scopes, prefix_scopes);
            }
        }

        walk_scopes(
            workspace_dir,
            workspace_dir,
            0,
            &mut dir_scopes,
            &mut file_scopes,
            &mut prefix_scopes,
        );
        Self {
            store,
            dir_scopes,
            file_scopes,
            prefix_scopes,
        }
    }

    pub fn resolve(&self, rel_path: &str) -> ScopeTag {
        let rel = rel_path.trim().replace('\\', "/");
        let rel = rel.trim_start_matches("./");
        let mut tag = ScopeTag::default();

        if let Some(ft) = self.file_scopes.get(rel) {
            tag = ft.clone();
        }
        if tag.client_id.is_none() || tag.engagement_id.is_none() {
            let mut cur = match rel.rsplit_once('/') {
                Some((parent, _)) => parent.to_string(),
                None => String::new(),
            };
            loop {
                if let Some(dt) = self.dir_scopes.get(&cur) {
                    if tag.engagement_id.is_none() {
                        tag.engagement_id = dt.engagement_id.clone();
                    }
                    if tag.client_id.is_none() {
                        tag.client_id = dt.client_id.clone();
                    }
                    break;
                }
                if cur.is_empty() {
                    break;
                }
                cur = match cur.rsplit_once('/') {
                    Some((parent, _)) => parent.to_string(),
                    None => String::new(),
                };
            }
        }
        if tag.engagement_id.is_none() || tag.client_id.is_none() {
            if let Some((_, ptag)) = self
                .prefix_scopes
                .iter()
                .filter(|(pfx, _)| records::path_matches_prefix(rel, pfx))
                .max_by_key(|(pfx, _)| pfx.len())
            {
                if tag.engagement_id.is_none() {
                    tag.engagement_id = ptag.engagement_id.clone();
                }
                if tag.client_id.is_none() {
                    tag.client_id = ptag.client_id.clone();
                }
            }
        }
        if tag.engagement_id.is_none() || tag.client_id.is_none() {
            if let Some(eng) = records::resolve_engagement_for_path(&self.store, rel) {
                if tag.engagement_id.is_none() {
                    tag.engagement_id = Some(eng.id.clone());
                }
                if tag.client_id.is_none() {
                    tag.client_id = Some(eng.client_id.clone());
                }
            }
        }
        if tag.client_id.is_none() {
            if let Some(ref eid) = tag.engagement_id {
                if let Some(eng) = self.store.engagements.iter().find(|e| &e.id == eid) {
                    tag.client_id = Some(eng.client_id.clone());
                }
            }
        }
        tag
    }

    pub fn matches_path(
        &self,
        rel_path: &str,
        client_filter: Option<&str>,
        engagement_filter: Option<&str>,
    ) -> bool {
        self.resolve(rel_path)
            .matches_filter(client_filter, engagement_filter)
    }

    pub fn is_cross_client_path(
        &self,
        rel_path: &str,
        target_client_id: Option<&str>,
        target_engagement_id: Option<&str>,
    ) -> Option<String> {
        let tag = self.resolve(rel_path);
        if let Some(cid) = target_client_id {
            if let Some(ref path_cid) = tag.client_id {
                if path_cid != cid {
                    return Some(path_cid.clone());
                }
            }
        }
        if let Some(eid) = target_engagement_id {
            let resolved_target_cid = target_client_id.map(str::to_string).or_else(|| {
                self.store
                    .engagements
                    .iter()
                    .find(|e| e.id == eid)
                    .map(|e| e.client_id.clone())
            });
            if let Some(ref tcid) = resolved_target_cid {
                if let Some(ref path_cid) = tag.client_id {
                    if path_cid != tcid {
                        return Some(path_cid.clone());
                    }
                }
            }
        }
        None
    }

    pub fn other_client_prefixes(&self, target_client_id: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for eng in &self.store.engagements {
            if !eng.workspace_prefix.is_empty() && eng.client_id != target_client_id {
                out.push((eng.workspace_prefix.clone(), eng.client_id.clone()));
            }
        }
        for (pfx, tag) in &self.prefix_scopes {
            if let Some(ref cid) = tag.client_id {
                if cid != target_client_id {
                    out.push((pfx.clone(), cid.clone()));
                }
            }
        }
        for (dir, tag) in &self.dir_scopes {
            if !dir.is_empty() {
                if let Some(ref cid) = tag.client_id {
                    if cid != target_client_id {
                        out.push((dir.clone(), cid.clone()));
                    }
                }
            }
        }
        out
    }
}

#[derive(Clone, Debug)]
pub struct VaultChunk {
    pub chunk_id: String,
    pub path: String,
    pub doc_kind: String,
    pub sheet: Option<String>,
    pub page: Option<u32>,
    pub line_start: usize,
    pub line_end: usize,
    pub heading: Option<String>,
    pub text: String,
    pub token_len: usize,
    pub embedding: Vec<f32>,
    pub file_idx: u32,
    pub client_id: Option<String>,
    pub engagement_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct IndexedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mtime_ms: u64,
    pub mtime_ns: u128,
    pub sha256: String,
    pub ext: String,
    pub doc_kind: String,
    pub preview: String,
    pub indexed_bytes: u64,
    pub chunk_range: Range<usize>,
    pub client_id: Option<String>,
    pub engagement_id: Option<String>,
}

impl IndexedFile {
    pub fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("path".into(), json!(self.path));
        obj.insert("name".into(), json!(self.name));
        obj.insert("size".into(), json!(self.size));
        obj.insert("mtime_ms".into(), json!(self.mtime_ms));
        obj.insert("ext".into(), json!(self.ext));
        if let Some(ref cid) = self.client_id {
            obj.insert("client_id".into(), json!(cid));
        }
        if let Some(ref eid) = self.engagement_id {
            obj.insert("engagement_id".into(), json!(eid));
        }
        Value::Object(obj)
    }
}

#[derive(Clone, Debug)]
pub struct MemoryLine {
    pub line_no: usize,
    pub text: String,
    pub lower: String,
}

#[derive(Clone, Debug)]
pub struct ChunkHit<'a> {
    pub chunk: &'a VaultChunk,
    pub score: f64,
    pub bm25_score: f64,
    pub semantic_score: f64,
    pub excerpt: String,
}

#[derive(Clone, Debug)]
pub struct OwnedChunkHit {
    pub chunk_id: String,
    pub path: String,
    pub doc_kind: String,
    pub sheet: Option<String>,
    pub page: Option<u32>,
    pub line_start: usize,
    pub line_end: usize,
    pub heading: Option<String>,
    pub score: f64,
    pub bm25_score: f64,
    pub semantic_score: f64,
    pub excerpt: String,
    pub client_id: Option<String>,
    pub engagement_id: Option<String>,
}

pub struct PersonIndex {
    fp: Fingerprint,
    pub chain: Vec<ChainedReceipt>,
    pub chain_valid: bool,
    pub head_hash: String,
    receipt_tokens: HashMap<String, Box<[u32]>>,
    pub files: Vec<IndexedFile>,
    file_tokens: HashMap<String, Box<[u32]>>,
    pub chunks: Vec<VaultChunk>,
    /// Inverted BM25 posting lists: token -> sorted `[(chunk_idx, term_freq)]`.
    chunk_tokens: HashMap<String, Box<[(u32, u16)]>>,
    avg_chunk_len: f64,
    pub indexed_bytes: u64,
    pub semantic_enabled: bool,
    embedding_cfg: Option<EmbeddingConfig>,
    uses_remote_embeddings: bool,
    pub memory: Vec<MemoryLine>,
    engagement_to_client: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Tokenization & Helpers
// ---------------------------------------------------------------------------

#[inline]
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
        return hb.windows(nb.len()).any(|w| w.eq_ignore_ascii_case(nb));
    }
    haystack.to_lowercase().contains(needle_lower)
}

#[inline]
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
        set.insert(cur.clone());
    }
    cur.clear();
}

fn tokenize_tf(s: &str, counts: &mut HashMap<String, u16>, cur: &mut String) -> usize {
    counts.clear();
    cur.clear();
    let mut total = 0usize;
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
                total += 1;
                let entry = counts.entry(std::mem::take(cur)).or_insert(0);
                *entry = entry.saturating_add(1);
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        total += 1;
        let entry = counts.entry(std::mem::take(cur)).or_insert(0);
        *entry = entry.saturating_add(1);
    } else {
        cur.clear();
    }
    total
}

fn tokenize_query(s: &str) -> Vec<String> {
    let mut set = HashSet::new();
    let mut cur = String::new();
    tokenize_into(s, &mut set, &mut cur);
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

fn count_tokens(s: &str) -> usize {
    let mut total = 0usize;
    let mut cur_len = 0usize;
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            cur_len += ch.len_utf8();
        } else {
            if cur_len >= 2 {
                total += 1;
            }
            cur_len = 0;
        }
    }
    if cur_len >= 2 {
        total += 1;
    }
    total
}

fn narrow_candidates(
    index: &HashMap<String, Box<[u32]>>,
    query: &str,
    tokens: &[String],
    disallow_slash: bool,
) -> Option<Vec<u32>> {
    if tokens.is_empty()
        || query.contains('.')
        || query.contains('-')
        || (disallow_slash && query.contains('/'))
    {
        return None;
    }
    let mut lists: Vec<&[u32]> = Vec::with_capacity(tokens.len());
    for tok in tokens {
        lists.push(index.get(tok)?.as_ref());
    }
    if lists.is_empty() {
        return None;
    }
    lists.sort_by_key(|l| l.len());
    let mut acc: Vec<u32> = lists[0].to_vec();
    for other in &lists[1..] {
        acc.retain(|id| other.binary_search(id).is_ok());
    }
    Some(acc)
}

fn doc_kind_label(ext: &str, doc: Option<DocKind>) -> String {
    match doc {
        Some(DocKind::Pdf) => "pdf".to_string(),
        Some(DocKind::Word) => "docx".to_string(),
        Some(DocKind::Excel) => "xlsx".to_string(),
        Some(DocKind::PowerPoint) => "pptx".to_string(),
        None => match ext {
            "md" | "markdown" => "markdown".to_string(),
            "csv" => "csv".to_string(),
            "tsv" => "tsv".to_string(),
            "json" => "json".to_string(),
            "" => "text".to_string(),
            other => other.to_string(),
        },
    }
}

fn parse_page_marker(trimmed: &str) -> Option<u32> {
    trimmed
        .strip_prefix("## Page ")
        .and_then(|rest| rest.trim().parse::<u32>().ok())
}

fn parse_slide_marker(trimmed: &str) -> Option<u32> {
    trimmed
        .strip_prefix("## Slide ")
        .and_then(|rest| rest.trim().parse::<u32>().ok())
}

fn split_long_line_windows(line: &str) -> Vec<String> {
    let total_chars = line.chars().count();
    if total_chars <= MAX_CHUNK_CHARS {
        return vec![line.to_string()];
    }
    let char_offsets: Vec<usize> = line.char_indices().map(|(i, _)| i).collect();
    let step = MAX_CHUNK_CHARS.saturating_sub(OVERLAP_CHARS).max(1);
    let mut out = Vec::new();
    let mut start_ci = 0usize;
    while start_ci < char_offsets.len() {
        let end_ci = (start_ci + MAX_CHUNK_CHARS).min(char_offsets.len());
        let start_b = char_offsets[start_ci];
        let end_b = if end_ci >= char_offsets.len() {
            line.len()
        } else {
            char_offsets[end_ci]
        };
        let slice = line[start_b..end_b].trim();
        if !slice.is_empty() {
            out.push(slice.to_string());
        }
        if end_ci >= char_offsets.len() {
            break;
        }
        start_ci += step;
    }
    out
}

struct ChunkSource<'a> {
    path: &'a str,
    doc_kind: &'a str,
    sheet: Option<&'a str>,
    is_pdf: bool,
    total_pdf_pages: usize,
    is_pptx: bool,
}

fn chunk_text_lines(
    src: &ChunkSource<'_>,
    text: &str,
    chunk_seq: &mut usize,
    out: &mut Vec<VaultChunk>,
) {
    let path = src.path;
    let doc_kind = src.doc_kind;
    let sheet = src.sheet;
    let is_pdf = src.is_pdf;
    let total_pdf_pages = src.total_pdf_pages;
    let is_pptx = src.is_pptx;
    let mut cur_page: Option<u32> = if is_pdf { Some(1) } else { None };
    let mut cur_heading: Option<String> = None;
    let mut buf = String::new();
    let mut buf_start: usize = 0;
    let mut buf_end: usize = 0;
    let mut buf_page: Option<u32> = cur_page;
    let mut buf_heading: Option<String> = None;

    let flush = |buf: &mut String,
                 buf_start: &mut usize,
                 buf_end: &mut usize,
                 buf_page: Option<u32>,
                 buf_heading: &Option<String>,
                 chunk_seq: &mut usize,
                 out: &mut Vec<VaultChunk>| {
        let trimmed = buf.trim();
        if !trimmed.is_empty() {
            let seq = *chunk_seq;
            *chunk_seq += 1;
            let chunk_id = match sheet {
                Some(sh) => format!("{path}#{sh}:{seq}"),
                None => format!("{path}:{seq}"),
            };
            let chunk_text = trimmed.to_string();
            let token_len = count_tokens(&chunk_text);
            out.push(VaultChunk {
                chunk_id,
                path: path.to_string(),
                doc_kind: doc_kind.to_string(),
                sheet: sheet.map(str::to_string),
                page: buf_page,
                line_start: (*buf_start).max(1),
                line_end: (*buf_end).max((*buf_start).max(1)),
                heading: buf_heading.clone(),
                text: chunk_text,
                token_len,
                embedding: Vec::new(),
                file_idx: 0,
                client_id: None,
                engagement_id: None,
            });
        }
        buf.clear();
        *buf_start = 0;
        *buf_end = 0;
    };

    for (idx, raw_line) in cap_utf8_bytes(text, MAX_EXTRACTED_TEXT_BYTES).lines().enumerate() {
        if out.len() >= MAX_CHUNKS_PER_FILE {
            break;
        }
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        let mut boundary_changed = false;

        if is_pdf && total_pdf_pages > 1 {
            if let Some(p) = parse_page_marker(trimmed) {
                if cur_page != Some(p) {
                    cur_page = Some(p);
                    boundary_changed = true;
                }
            }
        }
        if is_pptx {
            if let Some(slide) = parse_slide_marker(trimmed) {
                let h = format!("Slide {slide}");
                if cur_heading.as_deref() != Some(h.as_str()) {
                    cur_heading = Some(h);
                    boundary_changed = true;
                }
            }
        } else if trimmed.starts_with('#') && !is_pdf {
            let heading_txt: String = trimmed
                .trim_start_matches('#')
                .trim()
                .chars()
                .take(120)
                .collect();
            if !heading_txt.is_empty() && cur_heading.as_deref() != Some(heading_txt.as_str()) {
                cur_heading = Some(heading_txt);
                boundary_changed = true;
            }
        }

        if boundary_changed && !buf.trim().is_empty() {
            flush(
                &mut buf,
                &mut buf_start,
                &mut buf_end,
                buf_page,
                &buf_heading,
                chunk_seq,
                out,
            );
        }

        if raw_line.chars().count() > MAX_CHUNK_CHARS {
            if !buf.trim().is_empty() {
                flush(
                    &mut buf,
                    &mut buf_start,
                    &mut buf_end,
                    buf_page,
                    &buf_heading,
                    chunk_seq,
                    out,
                );
            }
            for win in split_long_line_windows(raw_line) {
                if out.len() >= MAX_CHUNKS_PER_FILE {
                    break;
                }
                let seq = *chunk_seq;
                *chunk_seq += 1;
                let chunk_id = match sheet {
                    Some(sh) => format!("{path}#{sh}:{seq}"),
                    None => format!("{path}:{seq}"),
                };
                let token_len = count_tokens(&win);
                out.push(VaultChunk {
                    chunk_id,
                    path: path.to_string(),
                    doc_kind: doc_kind.to_string(),
                    sheet: sheet.map(str::to_string),
                    page: cur_page,
                    line_start: line_no,
                    line_end: line_no,
                    heading: cur_heading.clone(),
                    text: win,
                    token_len,
                    embedding: Vec::new(),
                    file_idx: 0,
                    client_id: None,
                    engagement_id: None,
                });
            }
            continue;
        }

        if trimmed.is_empty() {
            if buf.len() >= MIN_SPLIT_CHARS {
                flush(
                    &mut buf,
                    &mut buf_start,
                    &mut buf_end,
                    buf_page,
                    &buf_heading,
                    chunk_seq,
                    out,
                );
            }
            continue;
        }

        if !buf.is_empty() && buf.len() + raw_line.len() + 1 > MAX_CHUNK_CHARS {
            flush(
                &mut buf,
                &mut buf_start,
                &mut buf_end,
                buf_page,
                &buf_heading,
                chunk_seq,
                out,
            );
        }

        if buf.is_empty() {
            buf_start = line_no;
            buf_page = cur_page;
            buf_heading = cur_heading.clone();
        } else {
            buf.push('\n');
        }
        buf.push_str(raw_line);
        buf_end = line_no;
    }

    if !buf.trim().is_empty() {
        flush(
            &mut buf,
            &mut buf_start,
            &mut buf_end,
            buf_page,
            &buf_heading,
            chunk_seq,
            out,
        );
    }
}

// ---------------------------------------------------------------------------
// Incremental Persistence (<person_data_dir>/vault_index.json)
// ---------------------------------------------------------------------------

struct CachedFileEntry {
    path: String,
    name: String,
    size: u64,
    mtime_ms: u64,
    mtime_ns: u128,
    sha256: String,
    ext: String,
    doc_kind: String,
    preview: String,
    indexed_bytes: u64,
    chunks: Vec<VaultChunk>,
}

fn load_persisted_vault(
    data_dir: &Path,
    expected_emb_sig: &str,
) -> (HashMap<String, CachedFileEntry>, bool) {
    let mut map = HashMap::new();
    let path = data_dir.join(VAULT_INDEX_FILE);
    let Ok(md) = std::fs::metadata(&path) else {
        return (map, false);
    };
    if md.len() > MAX_VAULT_INDEX_BYTES {
        let _ = std::fs::remove_file(&path);
        return (map, true);
    }
    let Ok(file) = std::fs::File::open(&path) else {
        return (map, true);
    };
    let Ok(v) = serde_json::from_reader::<_, Value>(std::io::BufReader::new(file)) else {
        let _ = std::fs::remove_file(&path);
        return (map, true);
    };
    if !v.is_object() || v["version"].as_u64() != Some(VAULT_INDEX_VERSION) {
        let _ = std::fs::remove_file(&path);
        return (map, true);
    }
    if v["embedding_sig"].as_str() != Some(expected_emb_sig) {
        return (map, true);
    }
    let Some(files) = v["files"].as_array() else {
        let _ = std::fs::remove_file(&path);
        return (map, true);
    };
    let mut had_corrupt_entry = false;
    for f in files {
        let Some(fpath) = f["path"].as_str().map(str::to_string) else {
            had_corrupt_entry = true;
            continue;
        };
        let mtime_ns = f["mtime_ns"]
            .as_str()
            .and_then(|s| s.parse::<u128>().ok())
            .or_else(|| f["mtime_ns"].as_u64().map(|n| n as u128))
            .unwrap_or(0);
        let Some(sha256) = f["sha256"].as_str().map(str::to_string) else {
            had_corrupt_entry = true;
            continue;
        };
        let name = f["name"].as_str().unwrap_or("").to_string();
        let size = f["size"].as_u64().unwrap_or(0);
        let mtime_ms = f["mtime_ms"].as_u64().unwrap_or(0);
        let ext = f["ext"].as_str().unwrap_or("").to_string();
        let doc_kind = f["doc_kind"].as_str().unwrap_or("text").to_string();
        let preview = f["preview"].as_str().unwrap_or("").to_string();
        let indexed_bytes = f["indexed_bytes"].as_u64().unwrap_or(0);
        let mut chunks = Vec::new();
        let mut bad_chunk = false;
        if let Some(arr) = f["chunks"].as_array() {
            for c in arr.iter().take(MAX_CHUNKS_PER_FILE) {
                let Some(chunk_id) = c["chunk_id"].as_str().map(str::to_string) else {
                    bad_chunk = true;
                    break;
                };
                let Some(text) = c["text"].as_str().map(str::to_string) else {
                    bad_chunk = true;
                    break;
                };
                let mut embedding: Vec<f32> = c["embedding"]
                    .as_array()
                    .map(|ea| {
                        ea.iter()
                            .filter_map(|x| x.as_f64().map(|n| n as f32))
                            .collect()
                    })
                    .unwrap_or_default();
                if embedding.is_empty() || !embedding.iter().all(|x| x.is_finite()) {
                    bad_chunk = true;
                    break;
                }
                embedding.shrink_to_fit();
                chunks.push(VaultChunk {
                    chunk_id,
                    path: fpath.clone(),
                    doc_kind: c["doc_kind"].as_str().unwrap_or(&doc_kind).to_string(),
                    sheet: c["sheet"].as_str().map(str::to_string),
                    page: c["page"].as_u64().map(|n| n as u32),
                    line_start: c["line_start"].as_u64().unwrap_or(1) as usize,
                    line_end: c["line_end"].as_u64().unwrap_or(1) as usize,
                    heading: c["heading"].as_str().map(str::to_string),
                    text,
                    token_len: c["token_len"].as_u64().unwrap_or(0) as usize,
                    embedding,
                    file_idx: 0,
                    client_id: None,
                    engagement_id: None,
                });
            }
        } else {
            bad_chunk = true;
        }
        if bad_chunk {
            had_corrupt_entry = true;
            continue;
        }
        chunks.shrink_to_fit();
        map.insert(
            fpath.clone(),
            CachedFileEntry {
                path: fpath,
                name,
                size,
                mtime_ms,
                mtime_ns,
                sha256,
                ext,
                doc_kind,
                preview,
                indexed_bytes,
                chunks,
            },
        );
    }
    (map, had_corrupt_entry)
}

fn save_persisted_vault(
    data_dir: &Path,
    emb_sig: &str,
    files: &[IndexedFile],
    chunks: &[VaultChunk],
) {
    use std::io::Write as _;
    use std::sync::atomic::{AtomicU64, Ordering};
    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

    let target = data_dir.join(VAULT_INDEX_FILE);
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = target.with_file_name(format!(".{VAULT_INDEX_FILE}.ergon-{seq}.tmp"));
    let Ok(file) = std::fs::File::create(&tmp) else {
        return;
    };
    let mut out = std::io::BufWriter::with_capacity(64 * 1024, file);
    let mut line_buf: Vec<u8> = Vec::with_capacity(4096);

    let write_res: std::io::Result<()> = (|| {
        line_buf.clear();
        line_buf.extend_from_slice(br#"{"version":"#);
        write_json_u64(&mut line_buf, VAULT_INDEX_VERSION);
        line_buf.extend_from_slice(br#","embedding_sig":"#);
        write_json_str(&mut line_buf, emb_sig);
        line_buf.extend_from_slice(br#","files":["#);
        out.write_all(&line_buf)?;

        for (i, f) in files.iter().enumerate() {
            line_buf.clear();
            if i > 0 {
                line_buf.push(b',');
            }
            line_buf.extend_from_slice(br#"{"path":"#);
            write_json_str(&mut line_buf, &f.path);
            line_buf.extend_from_slice(br#","name":"#);
            write_json_str(&mut line_buf, &f.name);
            line_buf.extend_from_slice(br#","size":"#);
            write_json_u64(&mut line_buf, f.size);
            line_buf.extend_from_slice(br#","mtime_ms":"#);
            write_json_u64(&mut line_buf, f.mtime_ms);
            line_buf.extend_from_slice(br#","mtime_ns":"#);
            write_json_str(&mut line_buf, &f.mtime_ns.to_string());
            line_buf.extend_from_slice(br#","sha256":"#);
            write_json_str(&mut line_buf, &f.sha256);
            line_buf.extend_from_slice(br#","ext":"#);
            write_json_str(&mut line_buf, &f.ext);
            line_buf.extend_from_slice(br#","doc_kind":"#);
            write_json_str(&mut line_buf, &f.doc_kind);
            line_buf.extend_from_slice(br#","preview":"#);
            write_json_str(&mut line_buf, &f.preview);
            line_buf.extend_from_slice(br#","indexed_bytes":"#);
            write_json_u64(&mut line_buf, f.indexed_bytes);
            line_buf.extend_from_slice(br#","chunks":["#);
            out.write_all(&line_buf)?;

            for (ci, c) in chunks[f.chunk_range.clone()].iter().enumerate() {
                line_buf.clear();
                if ci > 0 {
                    line_buf.push(b',');
                }
                line_buf.extend_from_slice(br#"{"chunk_id":"#);
                write_json_str(&mut line_buf, &c.chunk_id);
                line_buf.extend_from_slice(br#","doc_kind":"#);
                write_json_str(&mut line_buf, &c.doc_kind);
                if let Some(sh) = &c.sheet {
                    line_buf.extend_from_slice(br#","sheet":"#);
                    write_json_str(&mut line_buf, sh);
                }
                if let Some(pg) = c.page {
                    line_buf.extend_from_slice(br#","page":"#);
                    write_json_u64(&mut line_buf, pg as u64);
                }
                line_buf.extend_from_slice(br#","line_start":"#);
                write_json_u64(&mut line_buf, c.line_start as u64);
                line_buf.extend_from_slice(br#","line_end":"#);
                write_json_u64(&mut line_buf, c.line_end as u64);
                if let Some(h) = &c.heading {
                    line_buf.extend_from_slice(br#","heading":"#);
                    write_json_str(&mut line_buf, h);
                }
                line_buf.extend_from_slice(br#","text":"#);
                write_json_str(&mut line_buf, &c.text);
                line_buf.extend_from_slice(br#","token_len":"#);
                write_json_u64(&mut line_buf, c.token_len as u64);
                line_buf.extend_from_slice(br#","embedding":["#);
                for (ei, &val) in c.embedding.iter().enumerate() {
                    if ei > 0 {
                        line_buf.push(b',');
                    }
                    let clean = if val.is_finite() { val } else { 0.0 };
                    let _ = write!(&mut line_buf, "{clean:.4}");
                }
                line_buf.extend_from_slice(b"]}");
                out.write_all(&line_buf)?;
            }
            out.write_all(b"]}")?;
        }
        out.write_all(b"]}")?;
        out.flush()
    })();
    drop(out);
    if write_res.is_ok() {
        if std::fs::rename(&tmp, &target).is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
    } else {
        let _ = std::fs::remove_file(&tmp);
    }
}

// ---------------------------------------------------------------------------
// Workspace Walking & Extraction
// ---------------------------------------------------------------------------

struct DiscoveredFile {
    entry: CachedFileEntry,
    recomputed: bool,
}

fn first_preview_line(text: &str) -> String {
    text.lines()
        .map(|l| l.trim_start_matches('#').trim())
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .chars()
        .take(160)
        .collect()
}

fn collect_files(
    root: &Path,
    dir: &Path,
    cached: &mut HashMap<String, CachedFileEntry>,
    force_reindex: bool,
    out: &mut Vec<DiscoveredFile>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.filter_map(|e| e.ok()) {
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy().into_owned();
        if name.starts_with('.') || name.eq_ignore_ascii_case("connector_secrets.json") {
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
            collect_files(root, &path, cached, force_reindex, out);
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
        let modified = md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok());
        let mtime_ms = modified.map(|d| d.as_millis() as u64).unwrap_or(0);
        let mtime_ns = modified.map(|d| d.as_nanos()).unwrap_or(0);
        let size = md.len();

        let doc = docs::doc_kind(&rel);
        let kind_label = doc_kind_label(&ext, doc);

        let max_read = if doc.is_some() {
            docs::MAX_DOC_BYTES
        } else {
            MAX_PLAIN_TEXT_BYTES
        };
        let mut prefetched_bytes: Option<Vec<u8>> = None;
        let sha256 = if size <= max_read {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let h = receipts::sha256_hex(&bytes);
                    if doc.is_none() {
                        prefetched_bytes = Some(bytes);
                    }
                    h
                }
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };

        if !force_reindex && !sha256.is_empty() {
            if let Some(hit) = cached.remove(&rel) {
                if hit.mtime_ns == mtime_ns && hit.sha256 == sha256 && hit.size == size {
                    out.push(DiscoveredFile {
                        entry: hit,
                        recomputed: false,
                    });
                    continue;
                }
            }
        }

        let mut chunks: Vec<VaultChunk> = Vec::new();
        let mut chunk_seq = 0usize;
        let mut preview = String::new();
        let mut indexed_bytes = 0u64;

        if let Some(dk) = doc.filter(|_| size <= docs::MAX_DOC_BYTES) {
            drop(prefetched_bytes);
            match dk {
                DocKind::Excel => {
                    if let Ok(first) = docs::extract(&path, DocKind::Excel, None) {
                        let sheets = if first.sheets.is_empty() {
                            first.sheet.clone().into_iter().collect::<Vec<_>>()
                        } else {
                            first.sheets
                        };
                        for sheet_name in sheets {
                            if chunks.len() >= MAX_CHUNKS_PER_FILE {
                                break;
                            }
                            if let Ok(ex) =
                                docs::extract(&path, DocKind::Excel, Some(&sheet_name))
                            {
                                if preview.is_empty() {
                                    preview = first_preview_line(&ex.text);
                                }
                                let capped = cap_utf8_bytes(&ex.text, MAX_EXTRACTED_TEXT_BYTES);
                                indexed_bytes += capped.len() as u64;
                                let src = ChunkSource {
                                    path: &rel,
                                    doc_kind: &kind_label,
                                    sheet: Some(&sheet_name),
                                    is_pdf: false,
                                    total_pdf_pages: 0,
                                    is_pptx: false,
                                };
                                chunk_text_lines(&src, capped, &mut chunk_seq, &mut chunks);
                            }
                        }
                    }
                }
                DocKind::Pdf => {
                    if let Ok(ex) = docs::extract(&path, DocKind::Pdf, None) {
                        preview = first_preview_line(&ex.text);
                        let capped = cap_utf8_bytes(&ex.text, MAX_EXTRACTED_TEXT_BYTES);
                        indexed_bytes = capped.len() as u64;
                        let src = ChunkSource {
                            path: &rel,
                            doc_kind: &kind_label,
                            sheet: None,
                            is_pdf: true,
                            total_pdf_pages: ex.pages.max(1),
                            is_pptx: false,
                        };
                        chunk_text_lines(&src, capped, &mut chunk_seq, &mut chunks);
                    }
                }
                DocKind::PowerPoint => {
                    if let Ok(ex) = docs::extract(&path, DocKind::PowerPoint, None) {
                        preview = first_preview_line(&ex.text);
                        let capped = cap_utf8_bytes(&ex.text, MAX_EXTRACTED_TEXT_BYTES);
                        indexed_bytes = capped.len() as u64;
                        let src = ChunkSource {
                            path: &rel,
                            doc_kind: &kind_label,
                            sheet: None,
                            is_pdf: false,
                            total_pdf_pages: 0,
                            is_pptx: true,
                        };
                        chunk_text_lines(&src, capped, &mut chunk_seq, &mut chunks);
                    }
                }
                DocKind::Word => {
                    if let Ok(ex) = docs::extract(&path, DocKind::Word, None) {
                        preview = first_preview_line(&ex.text);
                        let capped = cap_utf8_bytes(&ex.text, MAX_EXTRACTED_TEXT_BYTES);
                        indexed_bytes = capped.len() as u64;
                        let src = ChunkSource {
                            path: &rel,
                            doc_kind: &kind_label,
                            sheet: None,
                            is_pdf: false,
                            total_pdf_pages: 0,
                            is_pptx: false,
                        };
                        chunk_text_lines(&src, capped, &mut chunk_seq, &mut chunks);
                    }
                }
            }
        } else if doc.is_none() && size <= MAX_PLAIN_TEXT_BYTES {
            if let Some(bytes) = prefetched_bytes.or_else(|| std::fs::read(&path).ok()) {
                if !bytes.contains(&0) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        preview = first_preview_line(&text);
                        let capped = cap_utf8_bytes(&text, MAX_EXTRACTED_TEXT_BYTES);
                        indexed_bytes = capped.len() as u64;
                        let src = ChunkSource {
                            path: &rel,
                            doc_kind: &kind_label,
                            sheet: None,
                            is_pdf: false,
                            total_pdf_pages: 0,
                            is_pptx: false,
                        };
                        chunk_text_lines(&src, capped, &mut chunk_seq, &mut chunks);
                    }
                }
            }
        }
        chunks.shrink_to_fit();

        out.push(DiscoveredFile {
            entry: CachedFileEntry {
                path: rel,
                name,
                size,
                mtime_ms,
                mtime_ns,
                sha256,
                ext,
                doc_kind: kind_label,
                preview,
                indexed_bytes,
                chunks,
            },
            recomputed: true,
        });
    }
}

fn make_excerpt(text: &str, q_lower: &str, tokens: &[String]) -> String {
    const MAX_EXCERPT: usize = 300;
    let total_chars = text.chars().count();
    if total_chars <= MAX_EXCERPT {
        return text.trim().to_string();
    }
    let lower = text.to_lowercase();
    let match_byte = if !q_lower.is_empty() {
        lower.find(q_lower)
    } else {
        None
    }
    .or_else(|| tokens.iter().filter_map(|t| lower.find(t.as_str())).min());

    let char_start = match match_byte {
        Some(mb) => {
            let prefix_chars = lower[..mb.min(lower.len())].chars().count();
            prefix_chars.saturating_sub(80)
        }
        None => 0,
    };
    let mut slice: String = text.chars().skip(char_start).take(MAX_EXCERPT).collect();
    if char_start > 0 {
        slice = format!("…{}", slice.trim_start());
    }
    if char_start + MAX_EXCERPT < total_chars {
        slice = format!("{}…", slice.trim_end());
    }
    slice
}

impl PersonIndex {
    fn build(
        data_dir: &Path,
        workspace_dir: &Path,
        memory_path: &Path,
        fp: Fingerprint,
        force_reindex: bool,
    ) -> Self {
        let mut chain = receipts::load_chain(data_dir);
        chain.shrink_to_fit();
        let (chain_valid, head_hash) = receipts::verify_chain(&chain);

        let mut receipt_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        let mut tok_buf = HashSet::new();
        let mut cur_buf = String::new();
        for (i, r) in chain.iter().enumerate() {
            for field in [
                r.id.as_str(),
                r.task.as_str(),
                r.summary.as_str(),
                r.thread_id.as_str(),
            ] {
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
        let mut receipt_tokens: HashMap<String, Box<[u32]>> = receipt_tokens
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_slice()))
            .collect();
        receipt_tokens.shrink_to_fit();

        let embedding_cfg = load_embedding_config(data_dir);
        let wanted_emb_sig = embedding_cfg
            .as_ref()
            .map(EmbeddingConfig::signature)
            .unwrap_or_else(|| "det-256".to_string());

        let (mut cached, cache_needs_rebuild) = if force_reindex {
            (HashMap::new(), true)
        } else {
            load_persisted_vault(data_dir, &wanted_emb_sig)
        };
        let initial_cached_len = cached.len();
        let mut discovered = Vec::new();
        collect_files(
            workspace_dir,
            workspace_dir,
            &mut cached,
            force_reindex,
            &mut discovered,
        );
        let removed_any = !cached.is_empty();
        drop(cached);
        let any_recomputed = discovered.iter().any(|d| d.recomputed);

        // Compute embeddings for newly parsed/changed chunks.
        let mut uses_remote_embeddings = embedding_cfg.is_some();
        if any_recomputed {
            if let Some(ecfg) = &embedding_cfg {
                let mut missing_texts: Vec<&str> = Vec::new();
                for d in &discovered {
                    if d.recomputed {
                        for c in &d.entry.chunks {
                            missing_texts.push(c.text.as_str());
                        }
                    }
                }
                if !missing_texts.is_empty() {
                    if let Some(remote_vecs) = fetch_remote_embeddings(ecfg, &missing_texts) {
                        let mut vi = 0usize;
                        for d in &mut discovered {
                            if d.recomputed {
                                for c in &mut d.entry.chunks {
                                    c.embedding = remote_vecs[vi].clone();
                                    vi += 1;
                                }
                            }
                        }
                    } else {
                        uses_remote_embeddings = false;
                    }
                }
            }
            for d in &mut discovered {
                if d.recomputed {
                    for c in &mut d.entry.chunks {
                        if c.embedding.is_empty() {
                            c.embedding = deterministic_embedding(&c.text);
                        }
                    }
                }
            }
        } else if let Some(first_c) = discovered.iter().flat_map(|d| d.entry.chunks.iter()).next() {
            if first_c.embedding.len() == EMBED_DIM && embedding_cfg.is_none() {
                uses_remote_embeddings = false;
            }
        } else {
            uses_remote_embeddings = false;
        }

        // If remote embeddings fell back to deterministic for new chunks while
        // older cached chunks had remote dims, normalize all to deterministic
        // so cosine similarity stays dimension-consistent.
        if !uses_remote_embeddings && embedding_cfg.is_some() {
            for d in &mut discovered {
                for c in &mut d.entry.chunks {
                    if c.embedding.len() != EMBED_DIM {
                        c.embedding = deterministic_embedding(&c.text);
                    }
                }
            }
        }

        discovered.sort_by(|a, b| {
            b.entry
                .mtime_ms
                .cmp(&a.entry.mtime_ms)
                .then_with(|| a.entry.path.cmp(&b.entry.path))
        });

        let root_data_dir = records::infer_root_data_dir(data_dir);
        let scope_resolver = ScopeResolver::load(&root_data_dir, workspace_dir);
        let engagement_to_client: HashMap<String, String> = scope_resolver
            .store
            .engagements
            .iter()
            .map(|e| (e.id.clone(), e.client_id.clone()))
            .collect();

        let mut files: Vec<IndexedFile> = Vec::with_capacity(discovered.len());
        let mut chunks: Vec<VaultChunk> = Vec::new();
        let mut indexed_bytes = 0u64;

        for (file_idx, d) in discovered.into_iter().enumerate() {
            let e = d.entry;
            let scope = scope_resolver.resolve(&e.path);
            let start = chunks.len();
            for mut c in e.chunks {
                c.file_idx = file_idx as u32;
                c.client_id = scope.client_id.clone();
                c.engagement_id = scope.engagement_id.clone();
                c.embedding.shrink_to_fit();
                chunks.push(c);
            }
            let end = chunks.len();
            indexed_bytes += e.indexed_bytes;
            files.push(IndexedFile {
                path: e.path,
                name: e.name,
                size: e.size,
                mtime_ms: e.mtime_ms,
                mtime_ns: e.mtime_ns,
                sha256: e.sha256,
                ext: e.ext,
                doc_kind: e.doc_kind,
                preview: e.preview,
                indexed_bytes: e.indexed_bytes,
                chunk_range: start..end,
                client_id: scope.client_id,
                engagement_id: scope.engagement_id,
            });
        }

        let actual_emb_sig = if uses_remote_embeddings {
            wanted_emb_sig
        } else {
            "det-256".to_string()
        };
        let vault_path = data_dir.join(VAULT_INDEX_FILE);
        if force_reindex
            || cache_needs_rebuild
            || any_recomputed
            || removed_any
            || initial_cached_len == 0 && !files.is_empty()
            || !vault_path.exists()
        {
            save_persisted_vault(data_dir, &actual_emb_sig, &files, &chunks);
        }

        // Build inverted token index for files & BM25 inverted index for chunks.
        let mut file_tokens: HashMap<String, Vec<u32>> = HashMap::new();
        let mut chunk_tokens_raw: HashMap<String, Vec<(u32, u16)>> = HashMap::new();
        let mut tf_buf: HashMap<String, u16> = HashMap::new();
        let mut total_chunk_tokens = 0usize;

        for (fi, f) in files.iter().enumerate() {
            tokenize_into(&f.path, &mut tok_buf, &mut cur_buf);
            for tok in tok_buf.drain() {
                let list = file_tokens.entry(tok).or_default();
                if list.last().copied() != Some(fi as u32) {
                    list.push(fi as u32);
                }
            }
            for ci in f.chunk_range.clone() {
                let c = &chunks[ci];
                let tok_count = tokenize_tf(&c.text, &mut tf_buf, &mut cur_buf);
                total_chunk_tokens += tok_count;
                for (tok, tf) in tf_buf.drain() {
                    let flist = file_tokens.entry(tok.clone()).or_default();
                    if flist.last().copied() != Some(fi as u32) {
                        flist.push(fi as u32);
                    }
                    chunk_tokens_raw
                        .entry(tok)
                        .or_default()
                        .push((ci as u32, tf));
                }
                if let Some(sh) = &c.sheet {
                    tokenize_into(sh, &mut tok_buf, &mut cur_buf);
                    for tok in tok_buf.drain() {
                        let flist = file_tokens.entry(tok.clone()).or_default();
                        if flist.last().copied() != Some(fi as u32) {
                            flist.push(fi as u32);
                        }
                        let clist = chunk_tokens_raw.entry(tok).or_default();
                        if clist.last().map(|x| x.0) != Some(ci as u32) {
                            clist.push((ci as u32, 1));
                        }
                    }
                }
            }
        }

        let mut file_tokens: HashMap<String, Box<[u32]>> = file_tokens
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_slice()))
            .collect();
        file_tokens.shrink_to_fit();
        let mut chunk_tokens: HashMap<String, Box<[(u32, u16)]>> = chunk_tokens_raw
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_slice()))
            .collect();
        chunk_tokens.shrink_to_fit();
        let avg_chunk_len = if chunks.is_empty() {
            1.0
        } else {
            (total_chunk_tokens as f64 / chunks.len() as f64).max(1.0)
        };
        files.shrink_to_fit();
        chunks.shrink_to_fit();

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
            chunks,
            chunk_tokens,
            avg_chunk_len,
            indexed_bytes,
            semantic_enabled: true,
            embedding_cfg,
            uses_remote_embeddings,
            memory,
            engagement_to_client,
        }
    }

    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn index_stats(&self) -> Value {
        json!({
            "total_files": self.files.len(),
            "total_chunks": self.chunks.len(),
            "indexed_bytes": self.indexed_bytes,
            "semantic_enabled": self.semantic_enabled,
        })
    }

    fn query_embedding(&self, query: &str) -> Option<Vec<f32>> {
        if self.uses_remote_embeddings {
            if let Some(cfg) = &self.embedding_cfg {
                if let Some(mut vecs) = fetch_remote_embeddings(cfg, &[query]) {
                    return vecs.pop();
                }
            }
            return None;
        }
        Some(deterministic_embedding(query))
    }

    fn normalize_prefix(path_prefix: &str) -> String {
        let mut p = path_prefix.trim().replace('\\', "/");
        while let Some(rest) = p.strip_prefix("./") {
            p = rest.to_string();
        }
        while let Some(rest) = p.strip_prefix('/') {
            p = rest.to_string();
        }
        if p == "." {
            p.clear();
        }
        p
    }

    fn file_matches_prefix(path: &str, norm_prefix: &str) -> bool {
        if norm_prefix.is_empty() {
            return true;
        }
        path.starts_with(norm_prefix)
    }

    fn tag_matches(
        item_client_id: Option<&str>,
        item_engagement_id: Option<&str>,
        client_filter: Option<&str>,
        engagement_filter: Option<&str>,
    ) -> bool {
        let eng_f = engagement_filter.map(str::trim).filter(|s| !s.is_empty());
        let cli_f = client_filter.map(str::trim).filter(|s| !s.is_empty());
        if let Some(eid) = eng_f {
            if item_engagement_id != Some(eid) {
                return false;
            }
        }
        if let Some(cid) = cli_f {
            if item_client_id != Some(cid) {
                return false;
            }
        }
        true
    }

    fn receipt_matches_scope(
        &self,
        r: &ChainedReceipt,
        client_filter: Option<&str>,
        engagement_filter: Option<&str>,
    ) -> bool {
        let eng_f = engagement_filter.map(str::trim).filter(|s| !s.is_empty());
        let cli_f = client_filter.map(str::trim).filter(|s| !s.is_empty());
        if eng_f.is_none() && cli_f.is_none() {
            return true;
        }
        let r_eid = r
            .engagement_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        if let Some(eid) = eng_f {
            if r_eid != Some(eid) {
                return false;
            }
        }
        if let Some(cid) = cli_f {
            let r_cid = r_eid.and_then(|eid| self.engagement_to_client.get(eid).map(String::as_str));
            if r_cid != Some(cid) {
                return false;
            }
        }
        true
    }

    fn score_chunks<'a>(
        &'a self,
        query: &str,
        mode: &str,
        path_prefix: &str,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) -> (Vec<(usize, f64, Vec<ChunkHit<'a>>)>, usize) {
        let q_trim = query.trim();
        let q_low = q_trim.to_lowercase();
        let norm_prefix = Self::normalize_prefix(path_prefix);
        let norm_mode = match mode.trim().to_ascii_lowercase().as_str() {
            "bm25" => "bm25",
            "exact" => "exact",
            _ => "hybrid",
        };
        let chunk_ok = |c: &VaultChunk| -> bool {
            Self::file_matches_prefix(&c.path, &norm_prefix)
                && Self::tag_matches(
                    c.client_id.as_deref(),
                    c.engagement_id.as_deref(),
                    client_id,
                    engagement_id,
                )
        };
        let file_ok = |f: &IndexedFile| -> bool {
            Self::file_matches_prefix(&f.path, &norm_prefix)
                && Self::tag_matches(
                    f.client_id.as_deref(),
                    f.engagement_id.as_deref(),
                    client_id,
                    engagement_id,
                )
        };

        if q_low.is_empty() {
            let mut out = Vec::new();
            let mut total = 0usize;
            for (fi, f) in self.files.iter().enumerate() {
                if file_ok(f) {
                    total += 1;
                    out.push((fi, 0.0, Vec::new()));
                }
            }
            return (out, total);
        }

        let tokens = tokenize_query(&q_low);
        let n_docs = self.chunks.len() as f64;

        let mut bm25_scores: Vec<f64> = vec![0.0; self.chunks.len()];
        let mut exact_flags: Vec<bool> = vec![false; self.chunks.len()];

        if norm_mode == "exact" {
            for (ci, c) in self.chunks.iter().enumerate() {
                if !chunk_ok(c) {
                    continue;
                }
                let in_text = contains_icase(&c.text, &q_low);
                let in_meta = c
                    .sheet
                    .as_deref()
                    .is_some_and(|s| contains_icase(s, &q_low))
                    || c.heading
                        .as_deref()
                        .is_some_and(|h| contains_icase(h, &q_low));
                if in_text || in_meta {
                    exact_flags[ci] = true;
                    bm25_scores[ci] = if in_text { 1.0 } else { 0.5 };
                }
            }
        } else {
            // BM25 over inverted chunk token postings.
            const K1: f64 = 1.2;
            const B: f64 = 0.75;
            for tok in &tokens {
                if let Some(postings) = self.chunk_tokens.get(tok) {
                    let df = postings.len() as f64;
                    let idf = (((n_docs - df + 0.5) / (df + 0.5)) + 1.0).ln().max(0.01);
                    for &(ci_u32, tf_u16) in postings.iter() {
                        let ci = ci_u32 as usize;
                        if !chunk_ok(&self.chunks[ci]) {
                            continue;
                        }
                        let tf = tf_u16 as f64;
                        let dl = (self.chunks[ci].token_len as f64).max(1.0);
                        let denom = tf + K1 * (1.0 - B + B * (dl / self.avg_chunk_len));
                        bm25_scores[ci] += idf * (tf * (K1 + 1.0)) / denom;
                    }
                }
            }

            // Phrase match boost (and substring fallback when tokens is empty or query has punctuation).
            let need_substring_scan =
                tokens.is_empty() || q_low.contains('.') || q_low.contains('-');
            for (ci, c) in self.chunks.iter().enumerate() {
                if !chunk_ok(c) {
                    continue;
                }
                if (bm25_scores[ci] > 0.0 || need_substring_scan) && contains_icase(&c.text, &q_low) {
                    exact_flags[ci] = true;
                    bm25_scores[ci] += 1.5;
                }
            }
        }

        // Semantic cosine similarity (only in hybrid mode).
        let mut sem_scores: Vec<f64> = vec![0.0; self.chunks.len()];
        if norm_mode == "hybrid" {
            if let Some(q_emb) = self.query_embedding(q_trim) {
                for (ci, c) in self.chunks.iter().enumerate() {
                    if !chunk_ok(c) {
                        continue;
                    }
                    sem_scores[ci] = cosine_similarity(&q_emb, &c.embedding);
                }
            }
        }

        // Select matching chunks and compute RRF (hybrid) or single-signal score.
        let mut candidates: Vec<usize> = Vec::new();
        for ci in 0..self.chunks.len() {
            if !chunk_ok(&self.chunks[ci]) {
                continue;
            }
            let matched = match norm_mode {
                "exact" => exact_flags[ci],
                "bm25" => bm25_scores[ci] > 0.0 || exact_flags[ci],
                _ => {
                    bm25_scores[ci] > 0.0
                        || exact_flags[ci]
                        || sem_scores[ci] >= SEMANTIC_MIN_THRESHOLD
                }
            };
            if matched {
                candidates.push(ci);
            }
        }

        let mut final_scores: Vec<(usize, f64)> = Vec::with_capacity(candidates.len());
        if norm_mode == "hybrid" {
            let mut bm25_ranked: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|&ci| bm25_scores[ci] > 0.0 || exact_flags[ci])
                .collect();
            bm25_ranked.sort_by(|&a, &b| {
                bm25_scores[b]
                    .partial_cmp(&bm25_scores[a])
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cmp(&b))
            });
            let mut bm25_rank_map = HashMap::with_capacity(bm25_ranked.len());
            for (rank0, ci) in bm25_ranked.into_iter().enumerate() {
                bm25_rank_map.insert(ci, rank0 + 1);
            }

            let mut sem_ranked: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|&ci| sem_scores[ci] >= SEMANTIC_MIN_THRESHOLD)
                .collect();
            sem_ranked.sort_by(|&a, &b| {
                sem_scores[b]
                    .partial_cmp(&sem_scores[a])
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cmp(&b))
            });
            let mut sem_rank_map = HashMap::with_capacity(sem_ranked.len());
            for (rank0, ci) in sem_ranked.into_iter().enumerate() {
                sem_rank_map.insert(ci, rank0 + 1);
            }

            for ci in candidates {
                let mut rrf = 0.0;
                if let Some(&r) = bm25_rank_map.get(&ci) {
                    rrf += 1.0 / (RRF_K + r as f64);
                }
                if let Some(&r) = sem_rank_map.get(&ci) {
                    rrf += 1.0 / (RRF_K + r as f64);
                }
                rrf += bm25_scores[ci] * 1e-4 + sem_scores[ci].max(0.0) * 1e-4;
                final_scores.push((ci, rrf));
            }
        } else {
            for ci in candidates {
                final_scores.push((ci, bm25_scores[ci]));
            }
        }

        // Group chunk hits by file_idx.
        let mut by_file: HashMap<usize, Vec<ChunkHit<'a>>> = HashMap::new();
        for (ci, score) in final_scores {
            let chunk = &self.chunks[ci];
            let fi = chunk.file_idx as usize;
            let excerpt = make_excerpt(&chunk.text, &q_low, &tokens);
            by_file.entry(fi).or_default().push(ChunkHit {
                chunk,
                score,
                bm25_score: bm25_scores[ci],
                semantic_score: sem_scores[ci],
                excerpt,
            });
        }

        // Also match filenames/paths that might have 0 chunk hits (e.g. binary files or filename searches).
        for (fi, f) in self.files.iter().enumerate() {
            if !file_ok(f) {
                continue;
            }
            if by_file.contains_key(&fi) {
                continue;
            }
            let path_match = contains_icase(&f.path, &q_low)
                || (norm_mode != "exact"
                    && !tokens.is_empty()
                    && tokens.iter().all(|t| contains_icase(&f.path, t.as_str())));
            if path_match {
                by_file.insert(fi, Vec::new());
            }
        }

        let mut file_results: Vec<(usize, f64, Vec<ChunkHit<'a>>)> = Vec::with_capacity(by_file.len());
        for (fi, mut hits) in by_file {
            hits.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.chunk.line_start.cmp(&b.chunk.line_start))
            });
            let mut file_score = hits.first().map(|h| h.score).unwrap_or(0.01);
            if contains_icase(&self.files[fi].path, &q_low) {
                file_score += 0.005;
            }
            hits.truncate(5);
            file_results.push((fi, file_score, hits));
        }

        file_results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.files[b.0].mtime_ms.cmp(&self.files[a.0].mtime_ms))
                .then_with(|| self.files[a.0].path.cmp(&self.files[b.0].path))
        });

        let total_files = file_results.len();
        (file_results, total_files)
    }

    pub fn search_vault_chunks(
        &self,
        query: &str,
        path_prefix: &str,
        mode: &str,
        limit: usize,
    ) -> Vec<OwnedChunkHit> {
        self.search_vault_chunks_scoped(query, path_prefix, mode, limit, None, None)
    }

    pub fn search_vault_chunks_scoped(
        &self,
        query: &str,
        path_prefix: &str,
        mode: &str,
        limit: usize,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) -> Vec<OwnedChunkHit> {
        let limit = limit.clamp(1, 50);
        let (file_results, _) = self.score_chunks(query, mode, path_prefix, client_id, engagement_id);
        let mut flat: Vec<OwnedChunkHit> = Vec::new();
        for (fi, _, hits) in file_results {
            if hits.is_empty() {
                let f = &self.files[fi];
                flat.push(OwnedChunkHit {
                    chunk_id: format!("{}:0", f.path),
                    path: f.path.clone(),
                    doc_kind: f.doc_kind.clone(),
                    sheet: None,
                    page: None,
                    line_start: 1,
                    line_end: 1,
                    heading: None,
                    score: 0.01,
                    bm25_score: 0.0,
                    semantic_score: 0.0,
                    excerpt: f.preview.clone(),
                    client_id: f.client_id.clone(),
                    engagement_id: f.engagement_id.clone(),
                });
            } else {
                for h in hits {
                    flat.push(OwnedChunkHit {
                        chunk_id: h.chunk.chunk_id.clone(),
                        path: h.chunk.path.clone(),
                        doc_kind: h.chunk.doc_kind.clone(),
                        sheet: h.chunk.sheet.clone(),
                        page: h.chunk.page,
                        line_start: h.chunk.line_start,
                        line_end: h.chunk.line_end,
                        heading: h.chunk.heading.clone(),
                        score: h.score,
                        bm25_score: h.bm25_score,
                        semantic_score: h.semantic_score,
                        excerpt: h.excerpt,
                        client_id: h.chunk.client_id.clone(),
                        engagement_id: h.chunk.engagement_id.clone(),
                    });
                }
            }
        }
        flat.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
                .then_with(|| a.line_start.cmp(&b.line_start))
        });
        flat.truncate(limit);
        flat
    }

    pub fn search(&self, query: &str, scope: &str, limit: usize) -> Value {
        self.search_with(query, scope, "hybrid", limit, "")
    }

    pub fn search_with(
        &self,
        query: &str,
        scope: &str,
        mode: &str,
        limit: usize,
        path_prefix: &str,
    ) -> Value {
        self.search_scoped(query, scope, mode, limit, path_prefix, None, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn search_scoped(
        &self,
        query: &str,
        scope: &str,
        mode: &str,
        limit: usize,
        path_prefix: &str,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) -> Value {
        let mut buf = Vec::with_capacity(4096);
        self.write_search_json_scoped(
            &mut buf,
            query,
            scope,
            mode,
            limit,
            path_prefix,
            client_id,
            engagement_id,
        );
        serde_json::from_slice(&buf).unwrap_or_else(|_| json!({}))
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

    fn file_matches_query(&self, fi: usize, q_low: &str) -> bool {
        let f = &self.files[fi];
        if contains_icase(&f.path, q_low) {
            return true;
        }
        self.chunks[f.chunk_range.clone()]
            .iter()
            .any(|c| contains_icase(&c.text, q_low) || c.sheet.as_deref().is_some_and(|s| contains_icase(s, q_low)))
    }

    pub fn list_files(&self, offset: usize, limit: Option<usize>, q: &str) -> (usize, Vec<Value>) {
        self.list_files_scoped(offset, limit, q, None, None)
    }

    pub fn list_files_scoped(
        &self,
        offset: usize,
        limit: Option<usize>,
        q: &str,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) -> (usize, Vec<Value>) {
        let q_low = q.trim().to_lowercase();
        let matches_scope = |f: &IndexedFile| -> bool {
            Self::tag_matches(
                f.client_id.as_deref(),
                f.engagement_id.as_deref(),
                client_id,
                engagement_id,
            )
        };
        if q_low.is_empty() {
            let scoped: Vec<&IndexedFile> =
                self.files.iter().filter(|f| matches_scope(f)).collect();
            let total = scoped.len();
            let iter = scoped.into_iter().skip(offset);
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
                .filter(|&id| {
                    matches_scope(&self.files[id as usize])
                        && self.file_matches_query(id as usize, &q_low)
                })
                .map(|id| &self.files[id as usize])
                .collect(),
            None => (0..self.files.len())
                .filter(|&fi| matches_scope(&self.files[fi]) && self.file_matches_query(fi, &q_low))
                .map(|fi| &self.files[fi])
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

    /// Zero-DOM streaming writer equivalent to `list_files` + `/api/files` wrapper.
    pub fn write_files_json(
        &self,
        out: &mut Vec<u8>,
        person_id: &str,
        offset: usize,
        limit: Option<usize>,
        q: &str,
    ) {
        self.write_files_json_scoped(out, person_id, offset, limit, q, None, None);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_files_json_scoped(
        &self,
        out: &mut Vec<u8>,
        person_id: &str,
        offset: usize,
        limit: Option<usize>,
        q: &str,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) {
        let q_trim = q.trim();
        let matches_scope = |f: &IndexedFile| -> bool {
            Self::tag_matches(
                f.client_id.as_deref(),
                f.engagement_id.as_deref(),
                client_id,
                engagement_id,
            )
        };
        out.extend_from_slice(br#"{"person_id":"#);
        write_json_str(out, person_id);
        if q_trim.is_empty() {
            let scoped: Vec<&IndexedFile> =
                self.files.iter().filter(|f| matches_scope(f)).collect();
            let total = scoped.len();
            out.extend_from_slice(br#","total":"#);
            write_json_u64(out, total as u64);
            out.extend_from_slice(br#","files":["#);
            let iter = scoped.into_iter().skip(offset);
            match limit {
                Some(l) => {
                    for (i, f) in iter.take(l.clamp(1, 1000)).enumerate() {
                        if i > 0 {
                            out.push(b',');
                        }
                        write_file_summary_json(out, f);
                    }
                }
                None => {
                    for (i, f) in iter.enumerate() {
                        if i > 0 {
                            out.push(b',');
                        }
                        write_file_summary_json(out, f);
                    }
                }
            }
            out.extend_from_slice(b"]}");
            return;
        }
        let q_low = q_trim.to_lowercase();
        let tokens = tokenize_query(&q_low);
        let candidates = narrow_candidates(&self.file_tokens, &q_low, &tokens, true);
        let filtered: Vec<&IndexedFile> = match candidates {
            Some(ids) => ids
                .into_iter()
                .filter(|&id| {
                    matches_scope(&self.files[id as usize])
                        && self.file_matches_query(id as usize, &q_low)
                })
                .map(|id| &self.files[id as usize])
                .collect(),
            None => (0..self.files.len())
                .filter(|&fi| matches_scope(&self.files[fi]) && self.file_matches_query(fi, &q_low))
                .map(|fi| &self.files[fi])
                .collect(),
        };
        let total = filtered.len();
        out.extend_from_slice(br#","total":"#);
        write_json_u64(out, total as u64);
        out.extend_from_slice(br#","files":["#);
        let iter = filtered.into_iter().skip(offset);
        match limit {
            Some(l) => {
                for (i, f) in iter.take(l.clamp(1, 1000)).enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    write_file_summary_json(out, f);
                }
            }
            None => {
                for (i, f) in iter.enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    write_file_summary_json(out, f);
                }
            }
        }
        out.extend_from_slice(b"]}");
    }

    /// Zero-DOM streaming writer equivalent to `list_receipts`.
    pub fn write_receipts_json(
        &self,
        out: &mut Vec<u8>,
        offset: usize,
        limit: usize,
        q: &str,
        status: &str,
    ) {
        let limit = limit.clamp(1, 5000);
        let q_trim = q.trim();
        if q_trim.is_empty() && status != "finished" && status != "failed" {
            let total = self.chain.len();
            out.extend_from_slice(br#"{"receipts":["#);
            for (i, r) in self.chain.iter().rev().skip(offset).take(limit).enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_receipt_json(out, r);
            }
            out.extend_from_slice(br#"],"total":"#);
            write_json_u64(out, total as u64);
            out.extend_from_slice(br#","offset":"#);
            write_json_u64(out, offset as u64);
            out.extend_from_slice(br#","limit":"#);
            write_json_u64(out, limit as u64);
            out.extend_from_slice(br#","chain_valid":"#);
            write_json_bool(out, self.chain_valid);
            out.extend_from_slice(br#","head_hash":"#);
            write_json_str(out, &self.head_hash);
            out.push(b'}');
            return;
        }
        let q_low = q_trim.to_lowercase();
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
        out.extend_from_slice(br#"{"receipts":["#);
        for (i, r) in matched.into_iter().skip(offset).take(limit).enumerate() {
            if i > 0 {
                out.push(b',');
            }
            write_receipt_json(out, r);
        }
        out.extend_from_slice(br#"],"total":"#);
        write_json_u64(out, total as u64);
        out.extend_from_slice(br#","offset":"#);
        write_json_u64(out, offset as u64);
        out.extend_from_slice(br#","limit":"#);
        write_json_u64(out, limit as u64);
        out.extend_from_slice(br#","chain_valid":"#);
        write_json_bool(out, self.chain_valid);
        out.extend_from_slice(br#","head_hash":"#);
        write_json_str(out, &self.head_hash);
        out.push(b'}');
    }

    pub fn write_search_json(&self, out: &mut Vec<u8>, query: &str, scope: &str, limit: usize) {
        self.write_search_json_with(out, query, scope, "hybrid", limit, "");
    }

    /// Zero-DOM streaming writer equivalent to `search_with`.
    pub fn write_search_json_with(
        &self,
        out: &mut Vec<u8>,
        query: &str,
        scope: &str,
        mode: &str,
        limit: usize,
        path_prefix: &str,
    ) {
        self.write_search_json_scoped(out, query, scope, mode, limit, path_prefix, None, None);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_search_json_scoped(
        &self,
        out: &mut Vec<u8>,
        query: &str,
        scope: &str,
        mode: &str,
        limit: usize,
        path_prefix: &str,
        client_id: Option<&str>,
        engagement_id: Option<&str>,
    ) {
        let t0 = Instant::now();
        let q_trim = query.trim();
        let q = q_trim.to_lowercase();
        let limit = limit.clamp(1, 200);
        let tokens = tokenize_query(&q);
        let norm_mode = match mode.trim().to_ascii_lowercase().as_str() {
            "bm25" => "bm25",
            "exact" => "exact",
            _ => "hybrid",
        };
        let has_scope_filter = client_id.is_some_and(|s| !s.trim().is_empty())
            || engagement_id.is_some_and(|s| !s.trim().is_empty());

        let want_receipts = scope == "all" || scope == "receipts";
        let want_files = scope == "all" || scope == "files";
        let want_memory = scope == "all" || scope == "memory";

        let single_exact_token = tokens.len() == 1
            && tokens[0] == q
            && q.bytes().all(|b| b.is_ascii_alphanumeric());

        let mut receipt_hits: Vec<&ChainedReceipt> = Vec::new();
        let mut total_receipt_hits = 0usize;
        if want_receipts {
            if q.is_empty() {
                for r in self.chain.iter().rev() {
                    if !self.receipt_matches_scope(r, client_id, engagement_id) {
                        continue;
                    }
                    total_receipt_hits += 1;
                    if receipt_hits.len() < limit {
                        receipt_hits.push(r);
                    }
                }
            } else {
                let candidates = narrow_candidates(&self.receipt_tokens, &q, &tokens, false);
                match candidates {
                    Some(ids) if single_exact_token && !has_scope_filter => {
                        total_receipt_hits = ids.len();
                        for &id in ids.iter().rev().take(limit) {
                            receipt_hits.push(&self.chain[id as usize]);
                        }
                    }
                    Some(ids) => {
                        for &id in ids.iter().rev() {
                            let idx = id as usize;
                            let r = &self.chain[idx];
                            if self.receipt_matches_scope(r, client_id, engagement_id)
                                && receipt_matches(r, &q, &tokens)
                            {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(r);
                                }
                            }
                        }
                    }
                    None => {
                        for r in self.chain.iter().rev() {
                            if self.receipt_matches_scope(r, client_id, engagement_id)
                                && receipt_matches(r, &q, &tokens)
                            {
                                total_receipt_hits += 1;
                                if receipt_hits.len() < limit {
                                    receipt_hits.push(r);
                                }
                            }
                        }
                    }
                }
            }
        }

        let (file_hits, total_file_hits) = if want_files {
            self.score_chunks(q_trim, norm_mode, path_prefix, client_id, engagement_id)
        } else {
            (Vec::new(), 0)
        };

        let mut memory_hits: Vec<&MemoryLine> = Vec::new();
        let mut total_memory_hits = 0usize;
        if want_memory {
            for m in &self.memory {
                let matched = q.is_empty()
                    || m.lower.contains(&q)
                    || (!tokens.is_empty() && tokens.iter().all(|t| m.lower.contains(t.as_str())));
                if matched {
                    total_memory_hits += 1;
                    if memory_hits.len() < limit {
                        memory_hits.push(m);
                    }
                }
            }
        }

        let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;
        out.extend_from_slice(br#"{"query":"#);
        write_json_str(out, q_trim);
        out.extend_from_slice(br#","scope":"#);
        write_json_str(out, scope);
        out.extend_from_slice(br#","mode":"#);
        write_json_str(out, norm_mode);
        out.extend_from_slice(br#","elapsed_ms":"#);
        let _ = serde_json::to_writer(&mut *out, &elapsed_ms);
        out.extend_from_slice(br#","total_hits":"#);
        write_json_u64(
            out,
            (total_receipt_hits + total_file_hits + total_memory_hits) as u64,
        );
        out.extend_from_slice(br#","counts":{"receipts":"#);
        write_json_u64(out, total_receipt_hits as u64);
        out.extend_from_slice(br#","files":"#);
        write_json_u64(out, total_file_hits as u64);
        out.extend_from_slice(br#","memory":"#);
        write_json_u64(out, total_memory_hits as u64);
        out.extend_from_slice(br#"},"indexed":{"receipts":"#);
        write_json_u64(out, self.chain.len() as u64);
        out.extend_from_slice(br#","files":"#);
        write_json_u64(out, self.files.len() as u64);
        out.extend_from_slice(br#","memory_lines":"#);
        write_json_u64(out, self.memory.len() as u64);
        out.extend_from_slice(br#"},"index_stats":{"total_files":"#);
        write_json_u64(out, self.files.len() as u64);
        out.extend_from_slice(br#","total_chunks":"#);
        write_json_u64(out, self.chunks.len() as u64);
        out.extend_from_slice(br#","indexed_bytes":"#);
        write_json_u64(out, self.indexed_bytes);
        out.extend_from_slice(br#","semantic_enabled":"#);
        write_json_bool(out, self.semantic_enabled);
        out.extend_from_slice(br#"},"receipts":["#);
        for (i, r) in receipt_hits.into_iter().enumerate() {
            if i > 0 {
                out.push(b',');
            }
            write_receipt_json(out, r);
        }
        out.extend_from_slice(br#"],"files":["#);
        for (i, (fi, score, chunks)) in file_hits.into_iter().take(limit).enumerate() {
            if i > 0 {
                out.push(b',');
            }
            write_file_hit_json(out, &self.files[fi], score, &chunks);
        }
        out.extend_from_slice(br#"],"memory":["#);
        for (i, m) in memory_hits.into_iter().enumerate() {
            if i > 0 {
                out.push(b',');
            }
            out.extend_from_slice(br#"{"line":"#);
            write_json_u64(out, m.line_no as u64);
            out.extend_from_slice(br#","text":"#);
            write_json_str(out, &m.text);
            out.push(b'}');
        }
        out.extend_from_slice(b"]}");
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

    pub fn global() -> &'static Self {
        static CACHE: OnceLock<SearchCache> = OnceLock::new();
        CACHE.get_or_init(SearchCache::new)
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

    /// Return an already-cached `Arc<PersonIndex>` if its fingerprint is still fresh,
    /// without building a full token index when cold.
    pub fn get_if_fresh(
        &self,
        person_id: &str,
        data_dir: &Path,
        workspace_dir: &Path,
        memory_path: &Path,
    ) -> Option<Arc<PersonIndex>> {
        let fp = compute_fingerprint(data_dir, workspace_dir, memory_path);
        let slot = self.slot(person_id);
        let r = slot.read().unwrap();
        r.as_ref().filter(|idx| idx.fp == fp).cloned()
    }

    /// Return a fresh shared `Arc<PersonIndex>` for `person_id`.
    #[inline]
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
        *w = None;
        let built = Arc::new(PersonIndex::build(
            data_dir,
            workspace_dir,
            memory_path,
            fp,
            false,
        ));
        *w = Some(built.clone());
        drop(w);
        trim_allocator_memory();
        built
    }

    /// Force a full vault rebuild (bypassing persisted incremental cache) and
    /// refresh `<person_data_dir>/vault_index.json`.
    pub fn reindex(
        &self,
        person_id: &str,
        data_dir: &Path,
        workspace_dir: &Path,
        memory_path: &Path,
    ) -> Arc<PersonIndex> {
        let fp = compute_fingerprint(data_dir, workspace_dir, memory_path);
        let slot = self.slot(person_id);
        let mut w = slot.write().unwrap();
        *w = None;
        let built = Arc::new(PersonIndex::build(
            data_dir,
            workspace_dir,
            memory_path,
            fp,
            true,
        ));
        *w = Some(built.clone());
        drop(w);
        trim_allocator_memory();
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
            let tag = if i == 9876 {
                "needle_quarter_audit"
            } else {
                "routine_sync"
            };
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
