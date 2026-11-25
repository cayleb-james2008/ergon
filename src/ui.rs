//! Embedded web UI. The unified responsive SPA is compiled into one
//! self-contained HTML file under `assets/index.html` and embedded with
//! `include_str!`. Both `/` and `/phone` serve this same responsive page.

pub const INDEX_HTML: &str = include_str!("../assets/index.html");
