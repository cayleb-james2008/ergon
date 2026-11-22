//! Ergon binary: bind and serve. Configuration is entirely via env:
//!   ERGON_PORT (default 8787), ERGON_HOST (default 127.0.0.1),
//!   ERGON_DATA_DIR, ERGON_WORKSPACE, ERGON_MODEL_BASE_URL,
//!   ERGON_MODEL_ID, ERGON_MODEL_API_KEY.

use std::net::TcpListener;

fn main() {
    if std::env::args().nth(1).as_deref().is_some_and(|a| a == "--version" || a == "-V") {
        println!("ergon {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let cfg = ergon_rs::Config::from_env();
    let host = std::env::var("ERGON_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("ERGON_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);
    let listener = match TcpListener::bind((host.as_str(), port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("ergon: cannot bind {host}:{port}: {e}");
            std::process::exit(1);
        }
    };
    println!("ergon listening on http://{host}:{port}");
    println!(
        "model: {} @ {} ({})",
        cfg.model_id,
        cfg.model_base_url,
        if cfg.api_key.is_some() {
            "key configured"
        } else if cfg.model_configured() {
            "local loopback ready"
        } else {
            "NOT configured"
        }
    );
    if let Err(e) = ergon_rs::serve_with_updates(listener, cfg) {
        eprintln!("ergon: server error: {e}");
        std::process::exit(1);
    }
}
