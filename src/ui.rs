//! Embedded web UI. Each page is one self-contained HTML file (inline CSS
//! and JS, no framework, no build step, no CDN) kept under `assets/` and
//! compiled in with `include_str!`. `/` is the desktop shell (Chat, Today,
//! Work graph); `/phone` is the phone face with a bottom tab bar.

pub const PHONE_HTML: &str = include_str!("../assets/phone.html");

pub const INDEX_HTML: &str = include_str!("../assets/index.html");
