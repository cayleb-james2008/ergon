//! Office & PDF documents: read what employees actually have, write what
//! colleagues expect.
//!
//! Reading: PDF (text layer, page by page), Word `.docx` (as Markdown:
//! headings, lists, tables), Excel `.xlsx` (each sheet as CSV, with real
//! dates, booleans and cached formula values) and PowerPoint `.pptx`
//! (slides and speaker notes). One extractor serves the agent's
//! `read_file`/`search_files`, every built-in workflow (through
//! `host_workspace_read`), the file viewer and the search index, so a
//! contract PDF or an expenses workbook works everywhere a `.md` or `.csv`
//! does.
//!
//! Writing: Markdown → `.docx` (styled headings, bullet and numbered lists,
//! real tables, bold/italic/code, live hyperlinks) and CSV → `.xlsx`
//! (typed numbers, ISO dates as dates, bold frozen header row, autofilter,
//! fitted column widths).
//!
//! Zip containers are handled here directly (central directory + raw
//! DEFLATE via `miniz_oxide`) with per-entry and total decompression caps,
//! so a zip bomb cannot exhaust memory. PDF parsing (`pdf-extract`) runs on
//! its own thread behind `catch_unwind` and a timeout: a malformed PDF can
//! fail its own read, never the server.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Largest document Ergon will open for text (bytes on disk).
pub const MAX_DOC_BYTES: u64 = 25 * 1024 * 1024;
/// Zip-bomb guards: one entry, and everything read from one archive.
const MAX_ENTRY_BYTES: usize = 64 * 1024 * 1024;
const MAX_ARCHIVE_INFLATE: usize = 160 * 1024 * 1024;
const PDF_TIMEOUT_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// CRC-32 (zip)
// ---------------------------------------------------------------------------

fn crc32_table() -> &'static [u32; 256] {
    static T: OnceLock<[u32; 256]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, slot) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *slot = c;
        }
        t
    })
}

pub fn crc32(data: &[u8]) -> u32 {
    let t = crc32_table();
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = t[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

// ---------------------------------------------------------------------------
// Zip reading
// ---------------------------------------------------------------------------

fn le16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([*b.get(at)?, *b.get(at + 1)?, *b.get(at + 2)?, *b.get(at + 3)?]))
}

struct ZipEntry {
    name: String,
    method: u16,
    comp_size: usize,
    size: usize,
    local_offset: usize,
}

pub struct ZipReader<'a> {
    data: &'a [u8],
    entries: Vec<ZipEntry>,
    inflated: std::cell::Cell<usize>,
}

impl<'a> ZipReader<'a> {
    pub fn open(data: &'a [u8]) -> Result<ZipReader<'a>, String> {
        if data.len() < 22 {
            return Err("not a zip container (too short)".into());
        }
        // End of central directory: scan back over a possible comment.
        let lo = data.len().saturating_sub(22 + 65_535);
        let mut eocd = None;
        let mut i = data.len() - 22;
        loop {
            if le32(data, i) == Some(0x0605_4b50) {
                eocd = Some(i);
                break;
            }
            if i == lo {
                break;
            }
            i -= 1;
        }
        let eocd = eocd.ok_or("not a zip container (no directory)")?;
        let count = le16(data, eocd + 10).ok_or("bad zip directory")? as usize;
        let cd_off = le32(data, eocd + 16).ok_or("bad zip directory")? as usize;
        if cd_off == 0xFFFF_FFFF {
            return Err("ZIP64 documents are not supported".into());
        }
        let mut entries = Vec::with_capacity(count.min(10_000));
        let mut p = cd_off;
        for _ in 0..count.min(10_000) {
            if le32(data, p) != Some(0x0201_4b50) {
                return Err("damaged zip directory".into());
            }
            let method = le16(data, p + 10).ok_or("bad entry")?;
            let comp_size = le32(data, p + 20).ok_or("bad entry")? as usize;
            let size = le32(data, p + 24).ok_or("bad entry")? as usize;
            let n = le16(data, p + 28).ok_or("bad entry")? as usize;
            let e = le16(data, p + 30).ok_or("bad entry")? as usize;
            let c = le16(data, p + 32).ok_or("bad entry")? as usize;
            let local_offset = le32(data, p + 42).ok_or("bad entry")? as usize;
            let name_bytes = data.get(p + 46..p + 46 + n).ok_or("bad entry name")?;
            entries.push(ZipEntry {
                name: String::from_utf8_lossy(name_bytes).replace('\\', "/"),
                method,
                comp_size,
                size,
                local_offset,
            });
            p += 46 + n + e + c;
        }
        Ok(ZipReader { data, entries, inflated: std::cell::Cell::new(0) })
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }

    pub fn has(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        let e = self
            .entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| format!("missing part {name}"))?;
        if e.size > MAX_ENTRY_BYTES {
            return Err(format!("part {name} is too large to open safely"));
        }
        let lh = e.local_offset;
        if le32(self.data, lh) != Some(0x0403_4b50) {
            return Err(format!("damaged part {name}"));
        }
        let n = le16(self.data, lh + 26).ok_or("bad part")? as usize;
        let x = le16(self.data, lh + 28).ok_or("bad part")? as usize;
        let start = lh + 30 + n + x;
        let raw = self.data.get(start..start + e.comp_size).ok_or_else(|| format!("truncated part {name}"))?;
        let budget = MAX_ARCHIVE_INFLATE.saturating_sub(self.inflated.get());
        // The declared size may lie (zip bomb); the cap never does.
        let limit = MAX_ENTRY_BYTES.min(budget).max(1);
        let out = match e.method {
            0 => raw.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, limit)
                .map_err(|_| format!("part {name} is damaged or too large to open safely"))?,
            m => return Err(format!("part {name} uses unsupported compression ({m})")),
        };
        if out.len() > MAX_ENTRY_BYTES {
            return Err(format!("part {name} is too large to open safely"));
        }
        self.inflated.set(self.inflated.get() + out.len());
        if self.inflated.get() > MAX_ARCHIVE_INFLATE {
            return Err("document expands too large to open safely".into());
        }
        Ok(out)
    }

    pub fn read_string(&self, name: &str) -> Result<String, String> {
        Ok(String::from_utf8_lossy(&self.read(name)?).into_owned())
    }
}

// ---------------------------------------------------------------------------
// Zip writing (DEFLATE, UTF-8 names, fixed timestamp for reproducible files)
// ---------------------------------------------------------------------------

pub fn zip_write(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    // DOS date 2026-01-01 00:00 (stable bytes across runs).
    let dos_time: u16 = 0;
    let dos_date: u16 = ((2026 - 1980) << 9) | (1 << 5) | 1;
    for (name, data) in files {
        let crc = crc32(data);
        let deflated = miniz_oxide::deflate::compress_to_vec(data, 6);
        let (method, body): (u16, &[u8]) = if deflated.len() < data.len() { (8, &deflated) } else { (0, data) };
        let offset = out.len() as u32;
        let nb = name.as_bytes();
        let mut lh = Vec::with_capacity(30 + nb.len());
        lh.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        lh.extend_from_slice(&20u16.to_le_bytes());
        lh.extend_from_slice(&0x0800u16.to_le_bytes());
        lh.extend_from_slice(&method.to_le_bytes());
        lh.extend_from_slice(&dos_time.to_le_bytes());
        lh.extend_from_slice(&dos_date.to_le_bytes());
        lh.extend_from_slice(&crc.to_le_bytes());
        lh.extend_from_slice(&(body.len() as u32).to_le_bytes());
        lh.extend_from_slice(&(data.len() as u32).to_le_bytes());
        lh.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        lh.extend_from_slice(&0u16.to_le_bytes());
        lh.extend_from_slice(nb);
        out.extend_from_slice(&lh);
        out.extend_from_slice(body);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0x0800u16.to_le_bytes());
        central.extend_from_slice(&method.to_le_bytes());
        central.extend_from_slice(&dos_time.to_le_bytes());
        central.extend_from_slice(&dos_date.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(body.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0u8; 8]); // extra, comment, disk, internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(nb);
    }
    let cd_off = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_len.to_le_bytes());
    out.extend_from_slice(&cd_off.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

// ---------------------------------------------------------------------------
// Minimal XML scanning (OOXML is regular enough; no DTDs, no entities
// beyond the predefined five plus numeric references)
// ---------------------------------------------------------------------------

enum Xml<'a> {
    Open { name: &'a str, attrs: &'a str, self_close: bool },
    Close(&'a str),
    Text(&'a str),
}

fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

fn xml_events(s: &str) -> Vec<Xml<'_>> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            if s[i..].starts_with("<!--") {
                i = s[i..].find("-->").map(|e| i + e + 3).unwrap_or(b.len());
                continue;
            }
            if s[i..].starts_with("<![CDATA[") {
                let start = i + 9;
                let end = s[start..].find("]]>").map(|e| start + e).unwrap_or(b.len());
                out.push(Xml::Text(&s[start..end]));
                i = (end + 3).min(b.len());
                continue;
            }
            if s[i..].starts_with("<?") || s[i..].starts_with("<!") {
                i = s[i..].find('>').map(|e| i + e + 1).unwrap_or(b.len());
                continue;
            }
            // Find the tag end, honouring quoted attribute values.
            let mut j = i + 1;
            let mut quote: u8 = 0;
            while j < b.len() {
                let c = b[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    break;
                }
                j += 1;
            }
            let inner = &s[i + 1..j.min(b.len())];
            if let Some(name) = inner.strip_prefix('/') {
                out.push(Xml::Close(name.trim()));
            } else {
                let self_close = inner.ends_with('/');
                let inner = inner.trim_end_matches('/');
                let (name, attrs) = match inner.find(|c: char| c.is_whitespace()) {
                    Some(k) => (&inner[..k], &inner[k..]),
                    None => (inner, ""),
                };
                out.push(Xml::Open { name, attrs, self_close });
            }
            i = j + 1;
        } else {
            let end = s[i..].find('<').map(|e| i + e).unwrap_or(b.len());
            out.push(Xml::Text(&s[i..end]));
            i = end;
        }
    }
    out
}

fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        let tail = &rest[p..];
        let Some(semi) = tail.find(';').filter(|&k| k <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let ent = &tail[1..semi];
        let ch = match ent {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if ent.starts_with("#x") || ent.starts_with("#X") => u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32),
            _ if ent.starts_with('#') => ent[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            // Characters XML 1.0 forbids (control chars) are dropped.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

/// A relationship id (`r:id`), never an unprefixed `id` (a slide's
/// `<p:sldId id="256" r:id="rId2">` carries both).
fn rid(attrs: &str) -> Option<String> {
    let mut from = 0;
    while let Some(p) = attrs[from..].find(":id") {
        let at = from + p;
        let after = attrs[at + 3..].trim_start();
        let prefix_ok = attrs[..at].chars().next_back().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false);
        if prefix_ok && after.starts_with('=') {
            let v = after[1..].trim_start();
            let q = v.chars().next()?;
            if q == '"' || q == '\'' {
                let body = &v[1..];
                return body.find(q).map(|end| unescape(&body[..end]));
            }
        }
        from = at + 3;
    }
    None
}

fn attr(attrs: &str, key: &str) -> Option<String> {
    // Match `key="..."` or a prefixed `x:key="..."`, as a whole name.
    let mut from = 0;
    while let Some(p) = attrs[from..].find(key) {
        let at = from + p;
        let before_ok = at == 0 || {
            let c = attrs.as_bytes()[at - 1];
            c.is_ascii_whitespace() || c == b':'
        };
        let after = &attrs[at + key.len()..];
        let after_trim = after.trim_start();
        if before_ok && after_trim.starts_with('=') {
            let v = after_trim[1..].trim_start();
            let q = v.chars().next()?;
            if q == '"' || q == '\'' {
                let body = &v[1..];
                let end = body.find(q)?;
                return Some(unescape(&body[..end]));
            }
        }
        from = at + key.len();
    }
    None
}

/// Resolve an OOXML relationship target against the folder of its part.
fn resolve_target(base_dir: &str, target: &str) -> String {
    if let Some(abs) = target.strip_prefix('/') {
        return abs.to_string();
    }
    let mut parts: Vec<&str> = base_dir.split('/').filter(|p| !p.is_empty()).collect();
    for seg in target.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn rels_map(zip: &ZipReader<'_>, rels_path: &str, base_dir: &str) -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();
    if let Ok(x) = zip.read_string(rels_path) {
        for ev in xml_events(&x) {
            if let Xml::Open { name, attrs, .. } = ev {
                if local(name) == "Relationship" {
                    if let (Some(id), Some(t)) = (attr(attrs, "Id"), attr(attrs, "Target")) {
                        let ty = attr(attrs, "Type").unwrap_or_default();
                        let ext = attr(attrs, "TargetMode").map(|m| m == "External").unwrap_or(false);
                        let target = if ext { t } else { resolve_target(base_dir, &t) };
                        map.insert(id, (target, ty));
                    }
                }
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Word (.docx) → Markdown
// ---------------------------------------------------------------------------

fn md_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").trim().to_string()
}

pub fn docx_to_markdown(bytes: &[u8]) -> Result<String, String> {
    let zip = ZipReader::open(bytes)?;
    let doc = zip.read_string("word/document.xml").map_err(|_| "not a Word document (no word/document.xml)".to_string())?;
    // Numbered vs bulleted lists: numbering.xml says which numId is which.
    let mut numbered: HashMap<String, bool> = HashMap::new();
    if let Ok(nx) = zip.read_string("word/numbering.xml") {
        let mut abstract_fmt: HashMap<String, bool> = HashMap::new();
        let mut cur_abs: Option<String> = None;
        let mut first_lvl_seen = false;
        let mut cur_num: Option<String> = None;
        for ev in xml_events(&nx) {
            match ev {
                Xml::Open { name, attrs, .. } => match local(name) {
                    "abstractNum" => {
                        cur_abs = attr(attrs, "abstractNumId");
                        first_lvl_seen = false;
                    }
                    "lvl" => {}
                    "numFmt" if !first_lvl_seen => {
                        if let Some(a) = &cur_abs {
                            let v = attr(attrs, "val").unwrap_or_default();
                            abstract_fmt.insert(a.clone(), v != "bullet" && v != "none");
                            first_lvl_seen = true;
                        }
                    }
                    "num" => cur_num = attr(attrs, "numId"),
                    "abstractNumId" => {
                        if let (Some(n), Some(a)) = (&cur_num, attr(attrs, "val")) {
                            numbered.insert(n.clone(), *abstract_fmt.get(&a).unwrap_or(&false));
                        }
                    }
                    _ => {}
                },
                Xml::Close(name) if local(name) == "abstractNum" => cur_abs = None,
                _ => {}
            }
        }
    }
    let rels = rels_map(&zip, "word/_rels/document.xml.rels", "word");

    let mut out: Vec<String> = Vec::new();
    let mut para = String::new();
    let mut heading: usize = 0;
    let mut list: Option<(usize, bool)> = None; // (level, numbered)
    let mut list_num_id: Option<String> = None;
    let mut list_lvl: usize = 0;
    let mut in_t = false;
    let mut in_del = 0usize;
    let mut table_depth = 0usize;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut link_target: Option<String> = None;
    let mut link_text = String::new();
    let mut counters: HashMap<(String, usize), usize> = HashMap::new();
    let mut prev_list = false;

    for ev in xml_events(&doc) {
        match ev {
            Xml::Open { name, attrs, self_close } => match local(name) {
                "p" if !self_close => {
                    para.clear();
                    heading = 0;
                    list = None;
                    list_num_id = None;
                    list_lvl = 0;
                }
                "pStyle" => {
                    let v = attr(attrs, "val").unwrap_or_default();
                    let lower = v.to_ascii_lowercase();
                    if lower == "title" {
                        heading = 1;
                    } else if let Some(n) = lower.strip_prefix("heading") {
                        heading = n.trim().parse::<usize>().unwrap_or(1).clamp(1, 6);
                    } else if lower.contains("listbullet") {
                        list = Some((0, false));
                    } else if lower.contains("listnumber") {
                        list = Some((0, true));
                    }
                }
                "ilvl" => list_lvl = attr(attrs, "val").and_then(|v| v.parse().ok()).unwrap_or(0),
                "numId" => {
                    let id = attr(attrs, "val").unwrap_or_default();
                    if id != "0" {
                        let is_num = *numbered.get(&id).unwrap_or(&false);
                        list = Some((0, is_num));
                        list_num_id = Some(id);
                    }
                }
                "t" if !self_close && in_del == 0 => in_t = true,
                "del" if !self_close => in_del += 1,
                "tab" => para.push('\t'),
                "br" | "cr" => para.push('\n'),
                "hyperlink" if !self_close => {
                    link_target = rid(attrs).and_then(|id| rels.get(&id).map(|(t, _)| t.clone()));
                    link_text.clear();
                }
                "tbl" if !self_close => {
                    table_depth += 1;
                    if table_depth == 1 {
                        rows.clear();
                    }
                }
                "tr" if !self_close && table_depth == 1 => row.clear(),
                "tc" if !self_close && table_depth == 1 => cell.clear(),
                _ => {}
            },
            Xml::Text(t) if in_t => {
                let text = unescape(t);
                if link_target.is_some() {
                    link_text.push_str(&text);
                }
                para.push_str(&text);
            }
            Xml::Close(name) => match local(name) {
                "t" => in_t = false,
                "del" => in_del = in_del.saturating_sub(1),
                "hyperlink" => {
                    if let Some(url) = link_target.take() {
                        if url.starts_with("http") && !link_text.trim().is_empty() && !link_text.contains(&url) {
                            para.push_str(&format!(" ({url})"));
                        }
                    }
                }
                "p" => {
                    let text = para.trim_end().to_string();
                    if table_depth > 0 {
                        if !cell.is_empty() && !text.is_empty() {
                            cell.push(' ');
                        }
                        cell.push_str(text.trim());
                        continue;
                    }
                    if text.trim().is_empty() {
                        out.push(String::new());
                        continue;
                    }
                    let line = if heading > 0 {
                        format!("{} {}", "#".repeat(heading), text.trim())
                    } else if let Some((_, is_num)) = list {
                        let indent = "  ".repeat(list_lvl.min(4));
                        if is_num {
                            let key = (list_num_id.clone().unwrap_or_default(), list_lvl);
                            let n = counters.entry(key).or_insert(0);
                            *n += 1;
                            format!("{indent}{}. {}", n, text.trim())
                        } else {
                            format!("{indent}- {}", text.trim())
                        }
                    } else {
                        text
                    };
                    // Headings, new paragraphs and the start of a list get
                    // breathing room; consecutive list items stay together.
                    let is_list = list.is_some() && heading == 0;
                    if !is_list || !prev_list {
                        out.push(String::new());
                    }
                    prev_list = is_list;
                    out.push(line);
                }
                "tc" if table_depth == 1 => row.push(md_cell(&cell)),
                "tr" if table_depth == 1 => rows.push(std::mem::take(&mut row)),
                "tbl" => {
                    table_depth = table_depth.saturating_sub(1);
                    if table_depth == 0 && !rows.is_empty() {
                        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0).max(1);
                        out.push(String::new());
                        for (i, r) in rows.iter().enumerate() {
                            let mut cells = r.clone();
                            cells.resize(width, String::new());
                            out.push(format!("| {} |", cells.join(" | ")));
                            if i == 0 {
                                out.push(format!("|{}|", vec![" --- "; width].join("|")));
                            }
                        }
                        out.push(String::new());
                        rows.clear();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(tidy_lines(&out.join("\n")))
}

fn tidy_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank = 0;
    for line in s.lines() {
        let l = line.trim_end();
        if l.is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(l);
        out.push('\n');
    }
    out.trim().to_string() + "\n"
}

// ---------------------------------------------------------------------------
// PowerPoint (.pptx) → text, slide by slide, with speaker notes
// ---------------------------------------------------------------------------

fn drawing_paragraphs(xml: &str) -> Vec<String> {
    let mut paras = Vec::new();
    let mut cur = String::new();
    let mut in_t = false;
    for ev in xml_events(xml) {
        match ev {
            Xml::Open { name, self_close, .. } => match local(name) {
                "p" if !self_close => cur.clear(),
                "t" if !self_close => in_t = true,
                "br" => cur.push('\n'),
                _ => {}
            },
            Xml::Text(t) if in_t => cur.push_str(&unescape(t)),
            Xml::Close(name) => match local(name) {
                "t" => in_t = false,
                "p" => {
                    let t = cur.trim().to_string();
                    if !t.is_empty() {
                        paras.push(t);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    paras
}

pub fn pptx_to_text(bytes: &[u8]) -> Result<String, String> {
    let zip = ZipReader::open(bytes)?;
    let pres = zip.read_string("ppt/presentation.xml").map_err(|_| "not a PowerPoint deck (no ppt/presentation.xml)".to_string())?;
    let rels = rels_map(&zip, "ppt/_rels/presentation.xml.rels", "ppt");
    let mut slides: Vec<String> = Vec::new();
    for ev in xml_events(&pres) {
        if let Xml::Open { name, attrs, .. } = ev {
            if local(name) == "sldId" {
                if let Some((t, _)) = rid(attrs).and_then(|id| rels.get(&id).cloned()) {
                    slides.push(t);
                }
            }
        }
    }
    if slides.is_empty() {
        let mut names: Vec<&str> = zip.names().into_iter().filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml")).collect();
        names.sort_by_key(|n| n.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse::<u32>().unwrap_or(0));
        slides = names.into_iter().map(|s| s.to_string()).collect();
    }
    let mut out = String::new();
    for (i, path) in slides.iter().enumerate() {
        let Ok(x) = zip.read_string(path) else { continue };
        out.push_str(&format!("## Slide {}\n\n", i + 1));
        for p in drawing_paragraphs(&x) {
            out.push_str(&p);
            out.push('\n');
        }
        let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
        let srels = rels_map(&zip, &format!("{dir}/_rels/{file}.rels"), dir);
        if let Some((notes, _)) = srels.values().find(|(_, ty)| ty.ends_with("/notesSlide")) {
            if let Ok(nx) = zip.read_string(notes) {
                let notes: Vec<String> = drawing_paragraphs(&nx)
                    .into_iter()
                    .filter(|p| p.parse::<u32>().is_err())
                    .collect();
                if !notes.is_empty() {
                    out.push_str(&format!("\nSpeaker notes: {}\n", notes.join(" ")));
                }
            }
        }
        out.push('\n');
    }
    if out.trim().is_empty() {
        return Err("this deck has no text on its slides".into());
    }
    Ok(tidy_lines(&out))
}

// ---------------------------------------------------------------------------
// Excel (.xlsx) → sheets of cells → CSV
// ---------------------------------------------------------------------------

pub struct Sheet {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

fn col_index(cell_ref: &str) -> Option<usize> {
    let mut n: usize = 0;
    let mut seen = false;
    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            n = n * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
            seen = true;
        } else {
            break;
        }
    }
    if seen { Some(n - 1) } else { None }
}

fn is_date_format(id: u32, code: Option<&str>) -> (bool, bool) {
    // (is date, has time)
    if (14..=22).contains(&id) || (45..=47).contains(&id) || (27..=36).contains(&id) || (50..=58).contains(&id) {
        return (true, matches!(id, 18..=22 | 45..=47));
    }
    let Some(code) = code else { return (false, false) };
    // Strip quoted literals and bracketed colours/locales before looking.
    let mut plain = String::new();
    let mut in_q = false;
    let mut in_b = false;
    for c in code.chars() {
        match c {
            '"' => in_q = !in_q,
            '[' if !in_q => in_b = true,
            ']' if !in_q => in_b = false,
            _ if !in_q && !in_b => plain.push(c.to_ascii_lowercase()),
            _ => {}
        }
    }
    let date = plain.contains('y') || plain.contains('d') || (plain.contains('m') && !plain.contains('0') && !plain.contains('#'));
    let time = plain.contains('h') || plain.contains('s');
    (date || (time && !plain.contains('#') && !plain.contains('0')), time)
}

/// Excel serial (1900 date system) → "YYYY-MM-DD[ HH:MM]".
pub fn excel_serial_to_iso(serial: f64, with_time: bool) -> String {
    let days = serial.floor() as i64;
    // Excel counts a fictitious 1900-02-29 (serial 60): serials from 61 on
    // are days since 1899-12-30 (1970-01-01 = 25569); earlier ones are one
    // day closer to 1899-12-31.
    let unix_days = if days < 61 { days - 25568 } else { days - 25569 };
    let (y, m, d) = civil_from_days(unix_days);
    if with_time {
        let frac = serial - serial.floor();
        let mins = (frac * 1440.0).round() as i64;
        format!("{y:04}-{m:02}-{d:02} {:02}:{:02}", (mins / 60) % 24, mins % 60)
    } else {
        format!("{y:04}-{m:02}-{d:02}")
    }
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn clean_number(v: &str) -> String {
    match v.parse::<f64>() {
        Ok(f) if v.len() > 12 && f.is_finite() && (v.contains('.') || v.contains('E') || v.contains('e')) => {
            let s = format!("{:.10}", f);
            let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
            if s == "-0" { "0".into() } else { s }
        }
        _ => v.to_string(),
    }
}

pub fn xlsx_sheets(bytes: &[u8]) -> Result<Vec<Sheet>, String> {
    let zip = ZipReader::open(bytes)?;
    let wb = zip.read_string("xl/workbook.xml").map_err(|_| "not an Excel workbook (no xl/workbook.xml)".to_string())?;
    let rels = rels_map(&zip, "xl/_rels/workbook.xml.rels", "xl");
    let mut order: Vec<(String, String)> = Vec::new();
    let mut date1904 = false;
    for ev in xml_events(&wb) {
        if let Xml::Open { name, attrs, .. } = ev {
            match local(name) {
                "sheet" => {
                    let nm = attr(attrs, "name").unwrap_or_else(|| format!("Sheet{}", order.len() + 1));
                    if let Some((t, _)) = rid(attrs).and_then(|id| rels.get(&id).cloned()) {
                        order.push((nm, t));
                    }
                }
                "workbookPr" => date1904 = attr(attrs, "date1904").map(|v| v == "1" || v == "true").unwrap_or(false),
                _ => {}
            }
        }
    }
    // Shared strings (rich-text runs concatenated, phonetic runs skipped).
    let mut shared: Vec<String> = Vec::new();
    let ss_path = rels
        .values()
        .find(|(_, ty)| ty.ends_with("/sharedStrings"))
        .map(|(t, _)| t.clone())
        .unwrap_or_else(|| "xl/sharedStrings.xml".to_string());
    if let Ok(ss) = zip.read_string(&ss_path) {
        let mut cur = String::new();
        let mut in_t = false;
        let mut in_rph = false;
        for ev in xml_events(&ss) {
            match ev {
                Xml::Open { name, self_close, .. } => match local(name) {
                    "si" if !self_close => cur.clear(),
                    "si" => shared.push(String::new()),
                    "t" if !self_close && !in_rph => in_t = true,
                    "rPh" if !self_close => in_rph = true,
                    _ => {}
                },
                Xml::Text(t) if in_t => cur.push_str(&unescape(t)),
                Xml::Close(name) => match local(name) {
                    "t" => in_t = false,
                    "rPh" => in_rph = false,
                    "si" => shared.push(std::mem::take(&mut cur)),
                    _ => {}
                },
                _ => {}
            }
        }
    }
    // Styles: which cell formats are dates.
    let mut xf_date: Vec<(bool, bool)> = Vec::new();
    if let Ok(st) = zip.read_string("xl/styles.xml") {
        let mut custom: HashMap<u32, String> = HashMap::new();
        let mut in_xfs = false;
        for ev in xml_events(&st) {
            match ev {
                Xml::Open { name, attrs, self_close } => match local(name) {
                    "numFmt" => {
                        if let (Some(id), Some(code)) = (attr(attrs, "numFmtId").and_then(|v| v.parse().ok()), attr(attrs, "formatCode")) {
                            custom.insert(id, code);
                        }
                    }
                    "cellXfs" if !self_close => in_xfs = true,
                    "xf" if in_xfs => {
                        let id: u32 = attr(attrs, "numFmtId").and_then(|v| v.parse().ok()).unwrap_or(0);
                        xf_date.push(is_date_format(id, custom.get(&id).map(|s| s.as_str())));
                    }
                    _ => {}
                },
                Xml::Close(name) if local(name) == "cellXfs" => in_xfs = false,
                _ => {}
            }
        }
    }
    let mut sheets = Vec::new();
    for (nm, path) in order {
        let Ok(x) = zip.read_string(&path) else { continue };
        let mut grid: Vec<Vec<String>> = Vec::new();
        let mut row_idx: usize = 0;
        let mut next_row: usize = 0;
        let (mut c_col, mut c_type, mut c_style) = (0usize, String::new(), 0usize);
        let mut next_col: usize = 0;
        let mut val = String::new();
        let mut inline = String::new();
        let (mut in_v, mut in_is_t) = (false, false);
        for ev in xml_events(&x) {
            match ev {
                Xml::Open { name, attrs, self_close } => match local(name) {
                    "row" => {
                        row_idx = attr(attrs, "r").and_then(|v| v.parse::<usize>().ok()).map(|r| r.saturating_sub(1)).unwrap_or(next_row);
                        next_row = row_idx + 1;
                        next_col = 0;
                    }
                    "c" => {
                        c_col = attr(attrs, "r").and_then(|r| col_index(&r)).unwrap_or(next_col);
                        next_col = c_col + 1;
                        c_type = attr(attrs, "t").unwrap_or_default();
                        c_style = attr(attrs, "s").and_then(|v| v.parse().ok()).unwrap_or(0);
                        val.clear();
                        inline.clear();
                        if self_close {
                            continue;
                        }
                    }
                    "v" if !self_close => in_v = true,
                    "t" if !self_close => in_is_t = true,
                    _ => {}
                },
                Xml::Text(t) => {
                    if in_v {
                        val.push_str(&unescape(t));
                    } else if in_is_t {
                        inline.push_str(&unescape(t));
                    }
                }
                Xml::Close(name) => match local(name) {
                    "v" => in_v = false,
                    "t" => in_is_t = false,
                    "c" => {
                        let text = match c_type.as_str() {
                            "s" => val.trim().parse::<usize>().ok().and_then(|i| shared.get(i).cloned()).unwrap_or_default(),
                            "inlineStr" => inline.clone(),
                            "str" | "e" => val.clone(),
                            "b" => if val.trim() == "1" { "TRUE".into() } else { "FALSE".into() },
                            _ => {
                                let (is_date, has_time) = xf_date.get(c_style).copied().unwrap_or((false, false));
                                match val.trim().parse::<f64>() {
                                    Ok(f) if is_date && (0.0..2_958_466.0).contains(&f) => {
                                        excel_serial_to_iso(if date1904 { f + 1462.0 } else { f }, has_time && f.fract() != 0.0)
                                    }
                                    _ => clean_number(val.trim()),
                                }
                            }
                        };
                        if !text.is_empty() && row_idx < 1_048_576 && c_col < 16_384 {
                            if grid.len() <= row_idx {
                                grid.resize_with(row_idx + 1, Vec::new);
                            }
                            let r = &mut grid[row_idx];
                            if r.len() <= c_col {
                                r.resize(c_col + 1, String::new());
                            }
                            r[c_col] = text;
                        }
                    }
                    _ => {}
                },
            }
            if grid.len() > 200_000 {
                break;
            }
        }
        // Trim leading empty columns? No — column positions carry meaning.
        // Trim trailing empty rows and pad rows to a common width.
        while grid.last().map(|r| r.iter().all(|c| c.is_empty())).unwrap_or(false) {
            grid.pop();
        }
        let width = grid.iter().map(|r| r.len()).max().unwrap_or(0);
        for r in grid.iter_mut() {
            r.resize(width, String::new());
        }
        sheets.push(Sheet { name: nm, rows: grid });
    }
    if sheets.is_empty() {
        return Err("this workbook has no readable sheets".into());
    }
    Ok(sheets)
}

pub fn csv_line(cells: &[String]) -> String {
    cells
        .iter()
        .map(|c| {
            if c.contains([',', '"', '\n', '\r']) {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub fn rows_to_csv(rows: &[Vec<String>]) -> String {
    let mut s = String::new();
    for r in rows {
        s.push_str(&csv_line(r));
        s.push('\n');
    }
    s
}

// ---------------------------------------------------------------------------
// PDF → text (page by page), isolated
// ---------------------------------------------------------------------------

fn install_quiet_panic_hook() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if std::thread::current().name() == Some("ergon-pdf-extract") {
                return; // a malformed PDF: reported as a read error instead
            }
            prev(info);
        }));
    });
}

pub fn pdf_to_text(bytes: Vec<u8>) -> Result<(String, usize), String> {
    install_quiet_panic_hook();
    let (tx, rx) = std::sync::mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("ergon-pdf-extract".into())
        .spawn(move || {
            let res = std::panic::catch_unwind(move || {
                pdf_extract::extract_text_from_mem_by_pages(&bytes)
                    .or_else(|_| pdf_extract::extract_text_from_mem_by_pages_encrypted(&bytes, ""))
                    .map_err(|e| e.to_string())
            });
            let _ = tx.send(res);
        });
    if spawned.is_err() {
        return Err("could not start the PDF reader".into());
    }
    let pages = match rx.recv_timeout(std::time::Duration::from_secs(PDF_TIMEOUT_SECS)) {
        Ok(Ok(Ok(p))) => p,
        Ok(Ok(Err(e))) => {
            let low = e.to_ascii_lowercase();
            return Err(if low.contains("encrypt") || low.contains("password") {
                "this PDF is password-protected; open it with its password and save an unlocked copy".into()
            } else {
                format!("this PDF could not be read ({})", crate::truncate(&e, 160))
            });
        }
        Ok(Err(_)) => return Err("this PDF is damaged or uses features Ergon cannot read".into()),
        Err(_) => return Err("this PDF took too long to read".into()),
    };
    let n = pages.len();
    let mut out = String::new();
    for (i, p) in pages.iter().enumerate() {
        let text = tidy_lines(p);
        if text.trim().is_empty() {
            continue;
        }
        if n > 1 {
            out.push_str(&format!("## Page {}\n\n", i + 1));
        }
        out.push_str(&text);
        out.push('\n');
    }
    if out.trim().is_empty() {
        return Err("this PDF has no text layer (it is probably a scan or photo); Ergon cannot read text from images yet".into());
    }
    Ok((tidy_lines(&out), n))
}

// ---------------------------------------------------------------------------
// Dispatch + cache
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    Pdf,
    Word,
    Excel,
    PowerPoint,
}

impl DocKind {
    pub fn label(self) -> &'static str {
        match self {
            DocKind::Pdf => "PDF",
            DocKind::Word => "Word document",
            DocKind::Excel => "Excel workbook",
            DocKind::PowerPoint => "PowerPoint deck",
        }
    }
}

pub fn doc_kind(path: &str) -> Option<DocKind> {
    let lower = path.to_ascii_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "pdf" => Some(DocKind::Pdf),
        "docx" | "docm" => Some(DocKind::Word),
        "xlsx" | "xlsm" => Some(DocKind::Excel),
        "pptx" | "pptm" => Some(DocKind::PowerPoint),
        _ => None,
    }
}

/// `book.xlsx#Q2` → ("book.xlsx", Some("Q2")). Only for workbooks.
pub fn split_sheet(rel: &str) -> (&str, Option<&str>) {
    if let Some((base, sheet)) = rel.rsplit_once('#') {
        if doc_kind(base) == Some(DocKind::Excel) && !sheet.trim().is_empty() {
            return (base, Some(sheet.trim()));
        }
    }
    (rel, None)
}

enum Parsed {
    Text { text: String, pages: usize },
    Workbook(Vec<Sheet>),
}

/// (file, length, mtime ns) → its parsed text, shared by every reader.
type CacheKey = (PathBuf, u64, u128);
type ParseCache = HashMap<CacheKey, Arc<Result<Parsed, String>>>;

fn cache() -> &'static Mutex<ParseCache> {
    static C: OnceLock<Mutex<ParseCache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn parse_cached(full: &Path, kind: DocKind) -> Arc<Result<Parsed, String>> {
    let md = match std::fs::metadata(full) {
        Ok(m) => m,
        Err(_) => return Arc::new(Err("file not found".into())),
    };
    if md.len() > MAX_DOC_BYTES {
        return Arc::new(Err(format!(
            "this {} is larger than {} MB, too large to read",
            kind.label(),
            MAX_DOC_BYTES / (1024 * 1024)
        )));
    }
    let mtime = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let key = (full.to_path_buf(), md.len(), mtime);
    if let Some(hit) = cache().lock().unwrap_or_else(|p| p.into_inner()).get(&key) {
        return hit.clone();
    }
    let bytes = match std::fs::read(full) {
        Ok(b) => b,
        Err(e) => return Arc::new(Err(format!("could not read the file: {e}"))),
    };
    let parsed = match kind {
        DocKind::Pdf => pdf_to_text(bytes).map(|(text, pages)| Parsed::Text { text, pages }),
        DocKind::Word => docx_to_markdown(&bytes).map(|text| Parsed::Text { text, pages: 0 }),
        DocKind::PowerPoint => pptx_to_text(&bytes).map(|text| Parsed::Text { text, pages: 0 }),
        DocKind::Excel => xlsx_sheets(&bytes).map(Parsed::Workbook),
    };
    let arc = Arc::new(parsed);
    let mut c = cache().lock().unwrap_or_else(|p| p.into_inner());
    if c.len() >= 96 {
        c.clear();
    }
    // Older versions of the same file are dropped.
    c.retain(|k, _| k.0 != key.0);
    c.insert(key, arc.clone());
    arc
}

/// One document's text, ready for a reader.
pub struct Extracted {
    pub kind: DocKind,
    pub text: String,
    /// "markdown" (PDF, Word, PowerPoint) or "csv" (a workbook sheet).
    pub format: &'static str,
    pub sheets: Vec<String>,
    pub sheet: Option<String>,
    pub pages: usize,
}

pub fn extract(full: &Path, kind: DocKind, sheet: Option<&str>) -> Result<Extracted, String> {
    let parsed = parse_cached(full, kind);
    match parsed.as_ref() {
        Err(e) => Err(e.clone()),
        Ok(Parsed::Text { text, pages }) => Ok(Extracted {
            kind,
            text: text.clone(),
            format: "markdown",
            sheets: Vec::new(),
            sheet: None,
            pages: *pages,
        }),
        Ok(Parsed::Workbook(sheets)) => {
            let names: Vec<String> = sheets.iter().map(|s| s.name.clone()).collect();
            let chosen = match sheet {
                Some(want) => sheets
                    .iter()
                    .find(|s| s.name == want)
                    .or_else(|| sheets.iter().find(|s| s.name.eq_ignore_ascii_case(want)))
                    .ok_or_else(|| format!("no sheet named \"{want}\" (sheets: {})", names.join(", ")))?,
                None => sheets.iter().find(|s| !s.rows.is_empty()).unwrap_or(&sheets[0]),
            };
            Ok(Extracted {
                kind,
                text: rows_to_csv(&chosen.rows),
                format: "csv",
                sheets: names,
                sheet: Some(chosen.name.clone()),
                pages: 0,
            })
        }
    }
}

/// What the agent's `read_file` returns for a document: a one-line note
/// on what it is, then the text (CSV for a workbook sheet).
pub fn agent_view(full: &Path, rel: &str, kind: DocKind, sheet: Option<&str>) -> Result<String, String> {
    let ex = extract(full, kind, sheet).map_err(|e| format!("{rel}: {e}"))?;
    let note = match kind {
        DocKind::Excel => {
            let others: Vec<&String> = ex.sheets.iter().filter(|s| Some(*s) != ex.sheet.as_ref()).collect();
            if others.is_empty() {
                format!("(Excel workbook — sheet \"{}\" as CSV)", ex.sheet.clone().unwrap_or_default())
            } else {
                format!(
                    "(Excel workbook — sheet \"{}\" as CSV; other sheets: {} — read \"{}#<sheet>\" for one of them)",
                    ex.sheet.clone().unwrap_or_default(),
                    others.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", "),
                    rel.split('#').next().unwrap_or(rel)
                )
            }
        }
        DocKind::Pdf => format!("(text extracted from a {}-page PDF)", ex.pages.max(1)),
        k => format!("(text extracted from a {})", k.label()),
    };
    Ok(format!("{note}\n{}", ex.text))
}

// ---------------------------------------------------------------------------
// Markdown → Word (.docx)
// ---------------------------------------------------------------------------

struct DocxBuilder {
    body: String,
    links: Vec<String>,
    numbered_lists: usize,
}

const DOCX_STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="240"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="40"/><w:szCs w:val="40"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="280" w:after="100"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="60"/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:i/><w:sz w:val="22"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:sz w:val="22"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:i/><w:sz w:val="22"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="60"/><w:ind w:left="720"/><w:contextualSpacing/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:ind w:left="567" w:right="567"/><w:pBdr><w:left w:val="single" w:sz="12" w:space="8" w:color="BFBFBF"/></w:pBdr></w:pPr><w:rPr><w:i/><w:color w:val="404040"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/></w:pPr><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/><w:sz w:val="19"/></w:rPr></w:style><w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style><w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/><w:left w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/><w:right w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="BFBFBF"/></w:tblBorders><w:tblCellMar><w:top w:w="40" w:type="dxa"/><w:bottom w:w="40" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style></w:styles>"#;

fn numbering_xml(numbered_lists: usize) -> String {
    let lvl = |i: usize, fmt: &str, text: &str| {
        format!(
            r#"<w:lvl w:ilvl="{i}"><w:start w:val="1"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{text}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{}" w:hanging="360"/></w:pPr></w:lvl>"#,
            720 * (i + 1)
        )
    };
    let bullets = [("•", 0), ("◦", 1), ("▪", 2)];
    let mut s = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="hybridMultilevel"/>"#);
    for (sym, i) in bullets {
        s.push_str(&lvl(i, "bullet", sym));
    }
    s.push_str(r#"</w:abstractNum><w:abstractNum w:abstractNumId="1"><w:multiLevelType w:val="hybridMultilevel"/>"#);
    for (i, fmt) in ["decimal", "lowerLetter", "lowerRoman"].iter().enumerate() {
        s.push_str(&lvl(i, fmt, &format!("%{}.", i + 1)));
    }
    s.push_str(r#"</w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#);
    // One instance per numbered list so each restarts at 1.
    for k in 0..numbered_lists.max(1) {
        s.push_str(&format!(
            r#"<w:num w:numId="{}"><w:abstractNumId w:val="1"/><w:lvlOverride w:ilvl="0"><w:startOverride w:val="1"/></w:lvlOverride></w:num>"#,
            k + 2
        ));
    }
    s.push_str("</w:numbering>");
    s
}

#[derive(Clone, Copy, Default)]
struct Fmt {
    b: bool,
    i: bool,
    code: bool,
}

fn run_xml(text: &str, f: Fmt, style: Option<&str>) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut rpr = String::new();
    if let Some(s) = style {
        rpr.push_str(&format!(r#"<w:rStyle w:val="{s}"/>"#));
    }
    if f.code {
        rpr.push_str(r#"<w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/>"#);
    }
    if f.b {
        rpr.push_str("<w:b/><w:bCs/>");
    }
    if f.i {
        rpr.push_str("<w:i/><w:iCs/>");
    }
    let rpr = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{rpr}</w:rPr>") };
    format!(r#"<w:r>{rpr}<w:t xml:space="preserve">{}</w:t></w:r>"#, xml_escape(text))
}

impl DocxBuilder {
    /// Inline Markdown → runs: **bold**, *italic*, `code`, [text](url).
    fn inline(&mut self, text: &str, base: Fmt) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut buf = String::new();
        let mut f = base;
        let mut i = 0;
        let word = |c: Option<&char>| c.map(|c| c.is_alphanumeric()).unwrap_or(false);
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' && i + 1 < chars.len() && "\\`*_[]()#+-.!|".contains(chars[i + 1]) {
                buf.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '`' && !f.code {
                if let Some(end) = chars[i + 1..].iter().position(|&x| x == '`') {
                    out.push_str(&run_xml(&buf, f, None));
                    buf.clear();
                    let code: String = chars[i + 1..i + 1 + end].iter().collect();
                    out.push_str(&run_xml(&code, Fmt { code: true, ..f }, None));
                    i += end + 2;
                    continue;
                }
            }
            if (c == '*' || c == '_') && i + 1 < chars.len() && chars[i + 1] == c {
                // ** or __ toggles bold (__ only at word boundaries).
                if c == '*' || !(word(chars.get(i.wrapping_sub(1))) && word(chars.get(i + 2))) {
                    out.push_str(&run_xml(&buf, f, None));
                    buf.clear();
                    f.b = !f.b;
                    i += 2;
                    continue;
                }
            }
            if (c == '*' || c == '_') && !(c == '_' && word(chars.get(i.wrapping_sub(1))) && word(chars.get(i + 1))) {
                let closes = f.i;
                let opens = !f.i && chars[i + 1..].contains(&c) && chars.get(i + 1).map(|n| !n.is_whitespace()).unwrap_or(false);
                if closes || opens {
                    out.push_str(&run_xml(&buf, f, None));
                    buf.clear();
                    f.i = !f.i;
                    i += 1;
                    continue;
                }
            }
            if c == '[' {
                if let Some(close) = chars[i + 1..].iter().position(|&x| x == ']') {
                    let after = i + 1 + close + 1;
                    if chars.get(after) == Some(&'(') {
                        if let Some(end) = chars[after + 1..].iter().position(|&x| x == ')') {
                            let label: String = chars[i + 1..i + 1 + close].iter().collect();
                            let url: String = chars[after + 1..after + 1 + end].iter().collect();
                            out.push_str(&run_xml(&buf, f, None));
                            buf.clear();
                            if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
                                self.links.push(url);
                                let rid = format!("rIdL{}", self.links.len());
                                out.push_str(&format!(
                                    r#"<w:hyperlink r:id="{rid}" w:history="1">{}</w:hyperlink>"#,
                                    run_xml(&label, f, Some("Hyperlink"))
                                ));
                            } else {
                                out.push_str(&run_xml(&label, f, None));
                            }
                            i = after + 1 + end + 1;
                            continue;
                        }
                    }
                }
            }
            buf.push(c);
            i += 1;
        }
        out.push_str(&run_xml(&buf, f, None));
        out
    }

    fn para(&mut self, ppr: &str, text: &str, base: Fmt) {
        let runs = self.inline(text, base);
        let ppr = if ppr.is_empty() { String::new() } else { format!("<w:pPr>{ppr}</w:pPr>") };
        self.body.push_str(&format!("<w:p>{ppr}{runs}</w:p>"));
    }

    fn table(&mut self, rows: &[Vec<String>], aligns: &[&str]) {
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
        let col_w = 9360 / width; // 6.5" text width in twips
        let mut t = String::from(r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="5000" w:type="pct"/><w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="0" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/></w:tblPr><w:tblGrid>"#);
        for _ in 0..width {
            t.push_str(&format!(r#"<w:gridCol w:w="{col_w}"/>"#));
        }
        t.push_str("</w:tblGrid>");
        for (ri, r) in rows.iter().enumerate() {
            t.push_str("<w:tr>");
            if ri == 0 {
                t.push_str("<w:trPr><w:tblHeader/></w:trPr>");
            }
            for ci in 0..width {
                let cell = r.get(ci).map(|s| s.as_str()).unwrap_or("");
                let jc = match aligns.get(ci).copied().unwrap_or("") {
                    "center" => r#"<w:jc w:val="center"/>"#,
                    "right" => r#"<w:jc w:val="right"/>"#,
                    _ => "",
                };
                let shade = if ri == 0 { r#"<w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/>"# } else { "" };
                let runs = self.inline(cell, Fmt { b: ri == 0, ..Fmt::default() });
                t.push_str(&format!(
                    r#"<w:tc><w:tcPr><w:tcW w:w="{col_w}" w:type="dxa"/>{shade}</w:tcPr><w:p><w:pPr><w:spacing w:after="0"/>{jc}</w:pPr>{runs}</w:p></w:tc>"#
                ));
            }
            t.push_str("</w:tr>");
        }
        t.push_str("</w:tbl>");
        self.body.push_str(&t);
        // Word needs a paragraph between a table and what follows.
        self.body.push_str(r#"<w:p><w:pPr><w:spacing w:after="120"/></w:pPr></w:p>"#);
    }
}

fn split_table_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cur.push('|');
            chars.next();
        } else if c == '|' {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

fn is_table_sep(line: &str) -> bool {
    let t = line.trim();
    t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn list_item(line: &str) -> Option<(usize, bool, String)> {
    let indent = line.len() - line.trim_start().len();
    let t = line.trim_start();
    let level = (indent / 2).min(2);
    for p in ["- ", "* ", "+ "] {
        if let Some(rest) = t.strip_prefix(p) {
            let rest = rest.strip_prefix("[ ] ").map(|r| format!("☐ {r}")).or_else(|| rest.strip_prefix("[x] ").map(|r| format!("☑ {r}"))).unwrap_or_else(|| rest.to_string());
            return Some((level, false, rest));
        }
    }
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && digits.len() <= 4 {
        let rest = &t[digits.len()..];
        if let Some(r) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
            return Some((level, true, r.to_string()));
        }
    }
    None
}

/// Render Markdown as a Word document. `title` fills the document
/// properties (and a Title heading when the Markdown has no H1).
pub fn markdown_to_docx(md: &str, title: &str) -> Vec<u8> {
    let mut b = DocxBuilder { body: String::new(), links: Vec::new(), numbered_lists: 0 };
    let lines: Vec<&str> = md.lines().collect();
    let has_h1 = lines.iter().any(|l| l.starts_with("# "));
    if !has_h1 && !title.trim().is_empty() {
        b.para(r#"<w:pStyle w:val="Title"/>"#, title.trim(), Fmt::default());
    }
    let mut i = 0;
    let mut para_buf: Vec<String> = Vec::new();
    let mut in_numbered = false;
    let flush = |b: &mut DocxBuilder, buf: &mut Vec<String>| {
        if !buf.is_empty() {
            let text = buf.join(" ");
            b.para("", &text, Fmt::default());
            buf.clear();
        }
    };
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        // Fenced code.
        if t.starts_with("```") || t.starts_with("~~~") {
            flush(&mut b, &mut para_buf);
            in_numbered = false;
            let fence = &t[..3];
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with(fence) {
                let code = lines[i];
                b.body.push_str(&format!(
                    r#"<w:p><w:pPr><w:pStyle w:val="Code"/></w:pPr>{}</w:p>"#,
                    run_xml(if code.is_empty() { " " } else { code }, Fmt::default(), None)
                ));
                i += 1;
            }
            b.body.push_str(r#"<w:p><w:pPr><w:spacing w:after="60"/></w:pPr></w:p>"#);
            i += 1;
            continue;
        }
        if t.is_empty() {
            flush(&mut b, &mut para_buf);
            i += 1;
            continue;
        }
        // Headings.
        let hashes = t.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && t[hashes..].starts_with(' ') {
            flush(&mut b, &mut para_buf);
            in_numbered = false;
            let text = t[hashes..].trim().trim_end_matches('#').trim();
            b.para(&format!(r#"<w:pStyle w:val="Heading{hashes}"/>"#), text, Fmt::default());
            i += 1;
            continue;
        }
        // Horizontal rule.
        if t.len() >= 3 && (t.chars().all(|c| c == '-') || t.chars().all(|c| c == '*') || t.chars().all(|c| c == '_')) {
            flush(&mut b, &mut para_buf);
            b.body.push_str(r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="BFBFBF"/></w:pBdr></w:pPr></w:p>"#);
            i += 1;
            continue;
        }
        // Tables.
        if t.starts_with('|') && i + 1 < lines.len() && is_table_sep(lines[i + 1]) {
            flush(&mut b, &mut para_buf);
            in_numbered = false;
            let header = split_table_row(t);
            let aligns: Vec<&str> = split_table_row(lines[i + 1])
                .iter()
                .map(|c| {
                    let c = c.trim();
                    match (c.starts_with(':'), c.ends_with(':')) {
                        (true, true) => "center",
                        (false, true) => "right",
                        _ => "left",
                    }
                })
                .collect();
            let mut rows = vec![header];
            i += 2;
            while i < lines.len() && lines[i].trim().starts_with('|') {
                rows.push(split_table_row(lines[i]));
                i += 1;
            }
            b.table(&rows, &aligns);
            continue;
        }
        // Lists.
        if let Some((level, numbered, text)) = list_item(line) {
            flush(&mut b, &mut para_buf);
            let num_id = if numbered {
                if !in_numbered {
                    b.numbered_lists += 1;
                    in_numbered = true;
                }
                b.numbered_lists + 1
            } else {
                1
            };
            if !numbered && level == 0 {
                in_numbered = false;
            }
            b.para(
                &format!(r#"<w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{num_id}"/></w:numPr>"#),
                &text,
                Fmt::default(),
            );
            i += 1;
            continue;
        }
        // Quotes.
        if let Some(q) = t.strip_prefix('>') {
            flush(&mut b, &mut para_buf);
            b.para(r#"<w:pStyle w:val="Quote"/>"#, q.trim(), Fmt::default());
            i += 1;
            continue;
        }
        in_numbered = false;
        // Hard line break: two trailing spaces.
        if line.ends_with("  ") {
            para_buf.push(t.to_string());
            flush(&mut b, &mut para_buf);
        } else {
            para_buf.push(t.to_string());
        }
        i += 1;
    }
    flush(&mut b, &mut para_buf);

    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>{}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>"#,
        b.body
    );
    let mut doc_rels = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#);
    for (k, url) in b.links.iter().enumerate() {
        doc_rels.push_str(&format!(
            r#"<Relationship Id="rIdL{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>"#,
            k + 1,
            xml_escape(url)
        ));
    }
    doc_rels.push_str("</Relationships>");
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("[Content_Types].xml", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/></Types>"#.to_vec()),
        ("_rels/.rels", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#.to_vec()),
        ("word/document.xml", document.into_bytes()),
        ("word/_rels/document.xml.rels", doc_rels.into_bytes()),
        ("word/styles.xml", DOCX_STYLES.as_bytes().to_vec()),
        ("word/numbering.xml", numbering_xml(b.numbered_lists).into_bytes()),
        ("docProps/core.xml", core_props(title).into_bytes()),
        ("docProps/app.xml", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Ergon</Application></Properties>"#.to_vec()),
    ];
    zip_write(&files)
}

fn core_props(title: &str) -> String {
    let now = crate::records::now_millis();
    let (y, m, d) = crate::records::civil_from_millis(now);
    let secs = (now / 1000) % 86_400;
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>Ergon</dc:creator><dcterms:created xsi:type="dcterms:W3CDTF">{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z</dcterms:created></cp:coreProperties>"#,
        xml_escape(title),
        secs / 3600,
        (secs / 60) % 60,
        secs % 60
    )
}

// ---------------------------------------------------------------------------
// CSV rows → Excel (.xlsx)
// ---------------------------------------------------------------------------

enum CellValue {
    Text(String),
    /// value, style index (0 general, 2 #,##0.00, 3 0.00%, 4 date, 5 #,##0)
    Num(f64, u32),
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn typed_cell(raw: &str) -> CellValue {
    let s = raw.trim();
    if s.is_empty() || s.len() > 40 {
        return CellValue::Text(raw.to_string());
    }
    // ISO dates become real dates (sortable, filterable).
    let b = s.as_bytes();
    if s.len() == 10 && b[4] == b'-' && b[7] == b'-' {
        if let (Ok(y), Ok(m), Ok(d)) = (s[..4].parse::<i64>(), s[5..7].parse::<u32>(), s[8..].parse::<u32>()) {
            if (1900..=9999).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
                let serial = days_from_civil(y, m, d) + 25569;
                return CellValue::Num(serial as f64, 4);
            }
        }
    }
    let (neg_paren, body) = match s.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
        Some(inner) => (true, inner.trim()),
        None => (false, s),
    };
    let percent = body.ends_with('%');
    let body = body.trim_end_matches('%');
    let currency = body.starts_with(['$', '€', '£']) || body.starts_with("-$");
    let cleaned: String = body.chars().filter(|c| !matches!(c, '$' | '€' | '£' | ',' | ' ')).collect();
    // Keep identifiers as text: leading zeros, very long digit runs.
    let digits = cleaned.trim_start_matches('-');
    if digits.len() > 1 && digits.starts_with('0') && !digits.starts_with("0.") {
        return CellValue::Text(raw.to_string());
    }
    if digits.chars().filter(|c| c.is_ascii_digit()).count() > 15 {
        return CellValue::Text(raw.to_string());
    }
    if body.contains(',') && !body.split('.').next().unwrap_or("").split(',').skip(1).all(|g| g.len() == 3) {
        return CellValue::Text(raw.to_string());
    }
    let valid = !cleaned.is_empty()
        && cleaned.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && cleaned.chars().filter(|&c| c == '.').count() <= 1
        && cleaned.rfind('-').map(|p| p == 0).unwrap_or(true);
    if !valid {
        return CellValue::Text(raw.to_string());
    }
    let Ok(mut v) = cleaned.parse::<f64>() else { return CellValue::Text(raw.to_string()) };
    if neg_paren {
        v = -v.abs();
    }
    if percent {
        return CellValue::Num(v / 100.0, 3);
    }
    let two_dp = cleaned.split('.').nth(1).map(|d| d.len() == 2).unwrap_or(false);
    let has_dp = cleaned.contains('.');
    CellValue::Num(v, if currency || two_dp { 2 } else if body.contains(',') && !has_dp { 5 } else { 0 })
}

fn col_name(mut i: usize) -> String {
    let mut s = Vec::new();
    loop {
        s.push((b'A' + (i % 26) as u8) as char);
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    s.iter().rev().collect()
}

pub fn safe_sheet_name(name: &str) -> String {
    let n: String = name
        .chars()
        .map(|c| if matches!(c, '[' | ']' | ':' | '*' | '?' | '/' | '\\') { ' ' } else { c })
        .collect::<String>()
        .trim()
        .trim_matches('\'')
        .chars()
        .take(31)
        .collect();
    if n.trim().is_empty() { "Sheet1".to_string() } else { n }
}

/// Render rows (first row = header) as an Excel workbook.
pub fn rows_to_xlsx(rows: &[Vec<String>], sheet_name: &str, title: &str) -> Vec<u8> {
    let sheet = safe_sheet_name(sheet_name);
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0).max(1);
    let mut widths = vec![8usize; width];
    let mut data = String::new();
    for (ri, r) in rows.iter().enumerate() {
        data.push_str(&format!(r#"<row r="{}">"#, ri + 1));
        for (ci, cell) in r.iter().enumerate() {
            let len = cell.chars().count();
            widths[ci] = widths[ci].max(len.min(60));
            if cell.is_empty() {
                continue;
            }
            let rf = format!("{}{}", col_name(ci), ri + 1);
            if ri == 0 {
                data.push_str(&format!(r#"<c r="{rf}" t="inlineStr" s="1"><is><t xml:space="preserve">{}</t></is></c>"#, xml_escape(cell)));
                continue;
            }
            match typed_cell(cell) {
                CellValue::Num(v, s) => {
                    if s == 4 {
                        widths[ci] = widths[ci].max(10);
                    }
                    data.push_str(&format!(r#"<c r="{rf}" s="{s}"><v>{v}</v></c>"#));
                }
                CellValue::Text(t) => data.push_str(&format!(
                    r#"<c r="{rf}" t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                    xml_escape(&t)
                )),
            }
        }
        data.push_str("</row>");
    }
    let last = format!("{}{}", col_name(width - 1), rows.len().max(1));
    let mut cols = String::from("<cols>");
    for (i, w) in widths.iter().enumerate() {
        cols.push_str(&format!(r#"<col min="{0}" max="{0}" width="{1:.1}" customWidth="1"/>"#, i + 1, (*w as f64) * 1.1 + 2.0));
    }
    cols.push_str("</cols>");
    let freeze = if rows.len() > 1 {
        r#"<sheetViews><sheetView workbookViewId="0"><pane ySplit="1" topLeftCell="A2" activePane="bottomLeft" state="frozen"/><selection pane="bottomLeft" activeCell="A2" sqref="A2"/></sheetView></sheetViews>"#
    } else {
        r#"<sheetViews><sheetView workbookViewId="0"/></sheetViews>"#
    };
    let filter = if rows.len() > 1 { format!(r#"<autoFilter ref="A1:{last}"/>"#) } else { String::new() };
    let sheet_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><dimension ref="A1:{last}"/>{freeze}<sheetFormatPr defaultRowHeight="15"/>{cols}<sheetData>{data}</sheetData>{filter}<pageMargins left="0.7" right="0.7" top="0.75" bottom="0.75" header="0.3" footer="0.3"/></worksheet>"#
    );
    let defined = if rows.len() > 1 {
        format!(
            r#"<definedNames><definedName name="_xlnm._FilterDatabase" localSheetId="0" hidden="1">'{}'!$A$1:${}${}</definedName></definedNames>"#,
            xml_escape(&sheet.replace('\'', "''")),
            col_name(width - 1),
            rows.len()
        )
    } else {
        String::new()
    };
    let workbook = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><bookViews><workbookView/></bookViews><sheets><sheet name="{}" sheetId="1" r:id="rId1"/></sheets>{defined}</workbook>"#,
        xml_escape(&sheet)
    );
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><numFmts count="1"><numFmt numFmtId="164" formatCode="yyyy\-mm\-dd"/></numFmts><fonts count="2"><font><sz val="11"/><name val="Calibri"/><family val="2"/></font><font><b/><sz val="11"/><name val="Calibri"/><family val="2"/></font></fonts><fills count="3"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill><fill><patternFill patternType="solid"><fgColor rgb="FFF2F2F2"/><bgColor indexed="64"/></patternFill></fill></fills><borders count="2"><border><left/><right/><top/><bottom/><diagonal/></border><border><left/><right/><top/><bottom style="thin"><color rgb="FFBFBFBF"/></bottom><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="6"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"/><xf numFmtId="4" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="10" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="164" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="3" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/></cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>"#;
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("[Content_Types].xml", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/></Types>"#.to_vec()),
        ("_rels/.rels", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#.to_vec()),
        ("xl/workbook.xml", workbook.into_bytes()),
        ("xl/_rels/workbook.xml.rels", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#.to_vec()),
        ("xl/worksheets/sheet1.xml", sheet_xml.into_bytes()),
        ("xl/styles.xml", styles.as_bytes().to_vec()),
        ("docProps/core.xml", core_props(title).into_bytes()),
        ("docProps/app.xml", br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Ergon</Application></Properties>"#.to_vec()),
    ];
    zip_write(&files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn zip_round_trip_and_bomb_guard() {
        let big = vec![b'a'; 200_000];
        let z = zip_write(&[("a.txt", b"hello".to_vec()), ("dir/b.txt", big.clone())]);
        let r = ZipReader::open(&z).unwrap();
        assert_eq!(r.read("a.txt").unwrap(), b"hello");
        assert_eq!(r.read("dir/b.txt").unwrap(), big);
        assert!(r.read("missing").is_err());
        assert!(ZipReader::open(b"not a zip at all, just text....").is_err());
    }

    #[test]
    fn xml_scanner_and_entities() {
        assert_eq!(unescape("a &lt;b&gt; &amp; &#233; &#x41; &bogus; &"), "a <b> & é A &bogus; &");
        assert_eq!(attr(r#" w:val="Heading2" x="1""#, "val").as_deref(), Some("Heading2"));
        assert_eq!(attr(r#" r:id="rId3" Id="x""#, "id").as_deref(), Some("rId3"));
        assert_eq!(resolve_target("xl", "worksheets/sheet1.xml"), "xl/worksheets/sheet1.xml");
        assert_eq!(resolve_target("ppt/slides", "../notesSlides/n1.xml"), "ppt/notesSlides/n1.xml");
        assert_eq!(resolve_target("xl", "/xl/sheet.xml"), "xl/sheet.xml");
    }

    #[test]
    fn markdown_docx_round_trip_keeps_structure() {
        let md = "# Weekly report\n\nRevenue rose **12%** in *Q3*; see [the dashboard](https://example.com/d).\n\n## Actions\n\n- Ship the brief\n- Call Priya\n  - before Friday\n\n1. First\n2. Second\n\n| Item | Cost |\n| --- | ---: |\n| Coffee | 4.50 |\n| Tea \\| herbal | 3.00 |\n\n```\nlet x = 1;\n```\n";
        let docx = markdown_to_docx(md, "Weekly report");
        let back = docx_to_markdown(&docx).unwrap();
        assert!(back.contains("# Weekly report"), "{back}");
        assert!(back.contains("## Actions"), "{back}");
        assert!(back.contains("- Ship the brief"), "{back}");
        assert!(back.contains("  - before Friday"), "{back}");
        assert!(back.contains("1. First") && back.contains("2. Second"), "{back}");
        assert!(back.contains("| Item | Cost |"), "{back}");
        assert!(back.contains("| Tea \\| herbal | 3.00 |"), "{back}");
        assert!(back.contains("Revenue rose 12% in Q3; see the dashboard (https://example.com/d)."), "{back}");
        assert!(back.contains("let x = 1;"), "{back}");
    }

    #[test]
    fn xlsx_round_trip_types_cells() {
        let rows: Vec<Vec<String>> = vec![
            vec!["Date".into(), "Vendor".into(), "Amount".into(), "Zip".into(), "Share".into()],
            vec!["2026-11-24".into(), "Acme, Inc.".into(), "$1,250.50".into(), "02139".into(), "12.5%".into()],
            vec!["2026-11-25".into(), "Beta".into(), "(42.00)".into(), "10001".into(), "".into()],
        ];
        let x = rows_to_xlsx(&rows, "Q4 / spend", "Spend");
        let sheets = xlsx_sheets(&x).unwrap();
        assert_eq!(sheets[0].name, "Q4   spend");
        let r = &sheets[0].rows;
        assert_eq!(r[0], rows[0]);
        assert_eq!(r[1][0], "2026-11-24", "dates come back as dates");
        assert_eq!(r[1][1], "Acme, Inc.");
        assert_eq!(r[1][2], "1250.5");
        assert_eq!(r[1][3], "02139", "leading zeros stay text");
        assert_eq!(r[1][4], "0.125");
        assert_eq!(r[2][2], "-42");
        assert_eq!(r[2][3], "10001");
        let csv = rows_to_csv(r);
        assert!(csv.starts_with("Date,Vendor,Amount,Zip,Share\n2026-11-24,\"Acme, Inc.\",1250.5,02139,0.125\n"), "{csv}");
    }

    #[test]
    fn typed_cells_are_conservative() {
        assert!(matches!(typed_cell("1,234"), CellValue::Num(v, 5) if v == 1234.0));
        assert!(matches!(typed_cell("$1,234"), CellValue::Num(v, 2) if v == 1234.0));
        assert!(matches!(typed_cell("12,34"), CellValue::Text(_)));
        assert!(matches!(typed_cell("555-1234"), CellValue::Text(_)));
        assert!(matches!(typed_cell("4111111111111111111"), CellValue::Text(_)));
        assert!(matches!(typed_cell("2026-13-01"), CellValue::Text(_)));
        assert!(matches!(typed_cell("INV-001"), CellValue::Text(_)));
        assert!(matches!(typed_cell("-7"), CellValue::Num(v, 0) if v == -7.0));
    }

    #[test]
    fn serial_dates_convert() {
        assert_eq!(excel_serial_to_iso(45000.0, false), "2023-03-15");
        assert_eq!(excel_serial_to_iso(45000.5, true), "2023-03-15 12:00");
        assert_eq!(excel_serial_to_iso(1.0, false), "1900-01-01");
    }

    #[test]
    fn sheet_suffix_only_for_workbooks() {
        assert_eq!(split_sheet("book.xlsx#Q2"), ("book.xlsx", Some("Q2")));
        assert_eq!(split_sheet("notes#1.md"), ("notes#1.md", None));
        assert_eq!(split_sheet("book.xlsx"), ("book.xlsx", None));
    }
}
