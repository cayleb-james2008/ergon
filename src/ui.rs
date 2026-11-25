//! Embedded web UI. The unified responsive SPA is compiled into
//! `assets/index.html` and optional lazy-loaded chunks under `assets/chunks/`
//! pre-compressed and embedded at compile time. Both `/` and `/phone` serve this responsive app.

use std::borrow::Cow;
use std::sync::OnceLock;

static INDEX_HTML_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html.gz"));

/// Return the embedded initial HTML shell, inflating the pre-gzipped `.rodata` payload once on demand.
pub fn index_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| {
        let raw = if INDEX_HTML_GZ.len() >= 18 {
            miniz_oxide::inflate::decompress_to_vec(&INDEX_HTML_GZ[10..INDEX_HTML_GZ.len() - 8])
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        String::from_utf8(raw).unwrap_or_default()
    })
}

pub struct ChunkAsset {
    pub route: &'static str,
    pub mime: &'static str,
    pub bytes: &'static [u8],
    pub pre_gzipped: bool,
    pub etag_raw: &'static str,
    pub etag_gzip: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/embedded_chunks.rs"));

/// Look up an embedded `/chunks/...` asset by URL path.
pub fn chunk(path: &str) -> Option<&'static ChunkAsset> {
    CHUNKS
        .binary_search_by(|c| c.route.cmp(path))
        .ok()
        .map(|idx| &CHUNKS[idx])
}

/// Return uncompressed bytes for `asset`, inflating pre-gzipped `.rodata` only on non-gzip fallback.
pub fn chunk_raw(asset: &ChunkAsset) -> Cow<'static, [u8]> {
    if !asset.pre_gzipped || asset.bytes.len() < 18 {
        return Cow::Borrowed(asset.bytes);
    }
    match miniz_oxide::inflate::decompress_to_vec(&asset.bytes[10..asset.bytes.len() - 8]) {
        Ok(raw) => Cow::Owned(raw),
        Err(_) => Cow::Borrowed(asset.bytes),
    }
}
