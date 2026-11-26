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

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn gzip_compress(data: &[u8]) -> Vec<u8> {
    let deflated = miniz_oxide::deflate::compress_to_vec(data, 9);
    let crc = crc32(data);
    let isize_bytes = (data.len() as u32).to_le_bytes();
    let mut out = Vec::with_capacity(10 + deflated.len() + 8);
    out.extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0x00, 0xff]);
    out.extend_from_slice(&deflated);
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&isize_bytes);
    out
}

fn etag_tag(data: &[u8], crc: u32) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}{crc:08x}")
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let index_path = Path::new(&manifest_dir).join("assets").join("index.html");
    let chunks_dir = Path::new(&manifest_dir).join("assets").join("chunks");
    let out_dir = env::var("OUT_DIR").unwrap();
    let gz_dir = Path::new(&out_dir).join("chunks_gz");
    fs::create_dir_all(&gz_dir).unwrap();
    println!("cargo:rerun-if-changed={}", index_path.display());
    println!("cargo:rerun-if-changed={}", chunks_dir.display());

    let index_raw = fs::read(&index_path).unwrap();
    let index_gz = gzip_compress(&index_raw);
    let index_gz_path = Path::new(&out_dir).join("index.html.gz");
    fs::write(&index_gz_path, &index_gz).unwrap();

    struct Entry {
        route: String,
        abs: String,
        mime: &'static str,
        pre_gzipped: bool,
        etag_raw: String,
        etag_gzip: String,
    }

    let mut entries: Vec<Entry> = Vec::new();
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
            let raw = fs::read(&path).unwrap();
            let short = etag_tag(&raw, crc32(&raw));
            let etag_raw = format!("\"{short}\"");
            let mime = mime_for_chunk(name);
            let compressible = mime.starts_with("application/javascript") || mime.starts_with("text/");
            if compressible {
                let gz = gzip_compress(&raw);
                let gz_path = gz_dir.join(format!("{name}.gz"));
                fs::write(&gz_path, &gz).unwrap();
                entries.push(Entry {
                    route: format!("/chunks/{name}"),
                    abs: gz_path.to_string_lossy().into_owned(),
                    mime,
                    pre_gzipped: true,
                    etag_gzip: format!("\"{short}-gzip\""),
                    etag_raw,
                });
            } else {
                let abs = path
                    .canonicalize()
                    .unwrap_or_else(|_| path.clone())
                    .to_string_lossy()
                    .into_owned();
                entries.push(Entry {
                    route: format!("/chunks/{name}"),
                    abs,
                    mime,
                    pre_gzipped: false,
                    etag_gzip: etag_raw.clone(),
                    etag_raw,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.route.cmp(&b.route));

    let mut code = String::from("pub static CHUNKS: &[ChunkAsset] = &[\n");
    for e in entries {
        code.push_str(&format!(
            "    ChunkAsset {{ route: {:?}, mime: {:?}, bytes: include_bytes!({:?}), pre_gzipped: {}, etag_raw: {:?}, etag_gzip: {:?} }},\n",
            e.route, e.mime, e.abs, e.pre_gzipped, e.etag_raw, e.etag_gzip
        ));
    }
    code.push_str("];\n");

    let dest = Path::new(&out_dir).join("embedded_chunks.rs");
    fs::write(dest, code).unwrap();

    // Pre-compress built-in `.rn` skill sources (`~107 KB` raw -> `~25 KB`
    // deflated) so `.rodata` stays lean while runtime inflates lazily on use.
    println!("cargo:rerun-if-changed=src/skills");
    let skills_dir = Path::new("src/skills");
    let skills_out_dir = Path::new(&out_dir).join("skills_deflate");
    fs::create_dir_all(&skills_out_dir).unwrap();
    let mut skill_entries: Vec<(String, String)> = Vec::new();
    if skills_dir.is_dir() {
        for entry in fs::read_dir(skills_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rn") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("valid utf-8 skill stem")
                .to_string();
            println!("cargo:rerun-if-changed={}", path.display());
            let raw = fs::read(&path).unwrap();
            let deflated = miniz_oxide::deflate::compress_to_vec(&raw, 9);
            let out_file = skills_out_dir.join(format!("{id}.deflate"));
            fs::write(&out_file, &deflated).unwrap();
            let abs = fs::canonicalize(&out_file).unwrap();
            skill_entries.push((id, abs.to_string_lossy().into_owned()));
        }
    }
    skill_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut skills_code = String::from(
        "pub struct EmbeddedSkillDeflate { pub id: &'static str, pub deflated: &'static [u8] }\n\
         pub static EMBEDDED_SKILLS: &[EmbeddedSkillDeflate] = &[\n",
    );
    for (id, abs) in skill_entries {
        skills_code.push_str(&format!(
            "    EmbeddedSkillDeflate {{ id: {id:?}, deflated: include_bytes!({abs:?}) }},\n",
        ));
    }
    skills_code.push_str("];\n");
    fs::write(Path::new(&out_dir).join("embedded_skills.rs"), skills_code).unwrap();

    let catalog_path = Path::new("src/skills/catalog.json");
    println!("cargo:rerun-if-changed={}", catalog_path.display());
    let catalog_raw = fs::read(catalog_path).unwrap();
    let catalog_deflated = miniz_oxide::deflate::compress_to_vec(&catalog_raw, 9);
    fs::write(Path::new(&out_dir).join("skills_catalog.deflate"), catalog_deflated).unwrap();
}
