//! Embedded web UI. The unified responsive SPA is compiled into
//! `assets/index.html` and optional lazy-loaded chunks under `assets/chunks/`
//! embedded at compile time. Both `/` and `/phone` serve this responsive app.

pub const INDEX_HTML: &str = include_str!("../assets/index.html");

include!(concat!(env!("OUT_DIR"), "/embedded_chunks.rs"));

/// Look up an embedded `/chunks/...` asset by URL path, returning `(mime, bytes)`.
pub fn chunk(path: &str) -> Option<(&'static str, &'static [u8])> {
    CHUNKS
        .binary_search_by(|(p, _, _)| (*p).cmp(path))
        .ok()
        .map(|idx| (CHUNKS[idx].1, CHUNKS[idx].2))
}
