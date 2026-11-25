use std::env;
use std::fs;
use std::path::Path;

fn mime_for_chunk(name: &str) -> &'static str {
    if name.ends_with(".js") || name.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if name.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if name.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let chunks_dir = Path::new(&manifest_dir).join("assets").join("chunks");
    println!("cargo:rerun-if-changed={}", chunks_dir.display());

    let mut entries: Vec<(String, String, &'static str)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&chunks_dir) {
        for ent in rd.flatten() {
            let path = ent.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            let abs = path
                .canonicalize()
                .unwrap_or_else(|_| path.clone())
                .to_string_lossy()
                .into_owned();
            entries.push((format!("/chunks/{name}"), abs, mime_for_chunk(name)));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut code = String::from("pub static CHUNKS: &[(&str, &str, &[u8])] = &[\n");
    for (route, abs, mime) in entries {
        code.push_str(&format!(
            "    ({route:?}, {mime:?}, include_bytes!({abs:?})),\n"
        ));
    }
    code.push_str("];\n");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("embedded_chunks.rs");
    fs::write(dest, code).unwrap();
}
