//! Structured rotating logging, secret redaction, panic crash capture,
//! one-click IT diagnostic bundle generation, and workspace help/onboarding config.
//!
//! Zero external heavy dependencies: uses Rust std + `serde_json` + `docs::zip_write_refs`.

use crate::tools::atomic_write_bytes;
use crate::{connectors, docs, mcp, receipts, records, tripwire, Config, PeopleState};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_LOG_MAX_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB
pub const DEFAULT_LOG_MAX_BACKUPS: usize = 5;
const MAX_BUNDLE_LOG_FILE_BYTES: usize = 5 * 1024 * 1024;
const MAX_BUNDLE_CRASH_FILE_BYTES: usize = 256 * 1024;
const MAX_BUNDLE_CRASH_FILES: usize = 20;
const MAX_BACKTRACE_BYTES: usize = 64 * 1024;

static CRASH_SEQ: AtomicU64 = AtomicU64::new(0);
static GLOBAL_LOGGER: OnceLock<RwLock<Option<Arc<RotatingLogger>>>> = OnceLock::new();
static PANIC_DATA_DIR: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
static PANIC_HOOK_INSTALLED: OnceLock<()> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

// ---------------------------------------------------------------------------
// UTC ISO-8601 timestamp helpers (pure std, Howard Hinnant civil_from_days)
// ---------------------------------------------------------------------------

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

pub fn now_utc_iso() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_utc_iso_ms(dur.as_millis() as u64)
}

pub fn format_utc_iso_ms(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{millis:03}Z")
}

fn compact_utc_stamp() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let ms = dur.as_millis() as u64;
    let secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u32;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;
    format!("{y:04}{m:02}{d:02}T{hh:02}{mm:02}{ss:02}{millis:03}Z")
}

fn open_private_append(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
}

fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut opts = OpenOptions::new();
    opts.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Secret Redaction (zero-dep, multi-pass, JSON-safe)
// ---------------------------------------------------------------------------

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'+' | b'/' | b'=' | b'~' | b':')
}

fn is_val_stop(b: u8) -> bool {
    b.is_ascii_whitespace()
        || matches!(
            b,
            b'"' | b'\'' | b'`' | b'&' | b';' | b',' | b')' | b']' | b'}' | b'<' | b'>'
        )
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn redact_pem_blocks(input: &str) -> String {
    if !input.contains("PRIVATE KEY-----") {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("-----BEGIN ") {
        let after = &rest[start..];
        let Some(header_end) = after.find("PRIVATE KEY-----") else {
            out.push_str(&rest[..start + 11]);
            rest = &rest[start + 11..];
            continue;
        };
        if header_end > 48 {
            out.push_str(&rest[..start + 11]);
            rest = &rest[start + 11..];
            continue;
        }
        out.push_str(&rest[..start]);
        out.push_str("[REDACTED_PRIVATE_KEY]");
        if let Some(end_idx) = after.find("-----END ") {
            let tail = &after[end_idx..];
            if let Some(close_rel) = tail.find("PRIVATE KEY-----") {
                rest = &tail[close_rel + "PRIVATE KEY-----".len()..];
                continue;
            }
        }
        rest = "";
        break;
    }
    out.push_str(rest);
    out
}

fn redact_url_userinfo(input: &str) -> String {
    if !input.contains("://") || !input.contains('@') {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        if let Some(rel) = input[i..].find("://") {
            let scheme_end = i + rel + 3;
            out.push_str(&input[i..scheme_end]);
            let rest = &input[scheme_end..];
            // Scan authority up to first `/`, `?`, `#`, whitespace, or quote.
            let mut auth_end = 0;
            for &b in rest.as_bytes() {
                if b == b'/' || b == b'?' || b == b'#' || is_val_stop(b) {
                    break;
                }
                auth_end += 1;
            }
            let auth = &rest[..auth_end];
            if let Some(at_pos) = auth.rfind('@') {
                let userinfo = &auth[..at_pos];
                if userinfo.contains(':') {
                    out.push_str("[REDACTED]@");
                    out.push_str(&auth[at_pos + 1..]);
                    i = scheme_end + auth_end;
                    continue;
                }
            }
            i = scheme_end;
        } else {
            out.push_str(&input[i..]);
            break;
        }
    }
    out
}

fn redact_auth_schemes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if prev_ok {
            let rem = &input[i..];
            let scheme_len = if rem.len() >= 7 && rem[..7].eq_ignore_ascii_case("bearer ") {
                Some(("Bearer ", 7usize))
            } else if rem.len() >= 6 && rem[..6].eq_ignore_ascii_case("basic ") {
                Some(("Basic ", 6usize))
            } else {
                None
            };
            if let Some((label, slen)) = scheme_len {
                let mut j = i + slen;
                while j < bytes.len() && bytes[j] == b' ' {
                    j += 1;
                }
                let val_start = j;
                while j < bytes.len() && !is_val_stop(bytes[j]) {
                    j += 1;
                }
                if j > val_start {
                    let tok = &input[val_start..j];
                    if tok != "[REDACTED]" {
                        out.push_str(label);
                        out.push_str("[REDACTED]");
                        i = j;
                        continue;
                    }
                }
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn redact_env_secret_names(input: &str) -> String {
    const ENV_NAMES: &[&str] = &[
        "ERGON_MODEL_API_KEY",
        "ERGON_GATE_TOKEN",
        "ERGON_WEBHOOK_SECRET",
        "ERGON_SMTP_PASS",
        "ERGON_IMAP_PASS",
    ];
    let mut cur = input.to_string();
    for env_name in ENV_NAMES {
        if !cur.contains(env_name) {
            continue;
        }
        let mut out = String::with_capacity(cur.len());
        let bytes = cur.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if cur[i..].starts_with(env_name) {
                let mut j = i + env_name.len();
                // Check if followed by optional quote, separator `=` or `:`, and value.
                let mut k = j;
                if k < bytes.len() && matches!(bytes[k], b'"' | b'\'') {
                    k += 1;
                }
                while k < bytes.len() && bytes[k] == b' ' {
                    k += 1;
                }
                if k < bytes.len() && matches!(bytes[k], b'=' | b':') {
                    k += 1;
                    while k < bytes.len() && bytes[k] == b' ' {
                        k += 1;
                    }
                    if k < bytes.len() && matches!(bytes[k], b'"' | b'\'') {
                        let q = bytes[k];
                        k += 1;
                        while k < bytes.len() && bytes[k] != q {
                            if bytes[k] == b'\\' && k + 1 < bytes.len() {
                                k += 2;
                            } else {
                                k += 1;
                            }
                        }
                        if k < bytes.len() && bytes[k] == q {
                            k += 1;
                        }
                        j = k;
                    } else {
                        while k < bytes.len() && !is_val_stop(bytes[k]) {
                            k += 1;
                        }
                        j = k;
                    }
                    // If the env name itself was inside JSON quotes like `"ERGON_MODEL_API_KEY": "val"`,
                    // preserve valid JSON key-value syntax by writing `"[REDACTED_ENV_SECRET]": "[REDACTED]"`.
                    if out.ends_with('"') && cur[i + env_name.len()..].starts_with('"') {
                        out.push_str("[REDACTED_ENV_SECRET]\": \"[REDACTED]\"");
                        i = j;
                        continue;
                    }
                }
                out.push_str("[REDACTED_ENV_SECRET]");
                i = j;
                continue;
            }
            let ch = cur[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
        cur = out;
    }
    cur
}

fn is_sensitive_key_name(key: &str) -> bool {
    let low = key.to_ascii_lowercase();
    if matches!(
        low.as_str(),
        "has_secret"
            | "has_api_key"
            | "has_token"
            | "has_credentials"
            | "tokens"
            | "prompt_tokens"
            | "completion_tokens"
            | "cached_tokens"
            | "total_tokens"
            | "max_tokens"
            | "token_count"
            | "secret_count"
            | "secrets"
            | "username_secret_key"
            | "password_secret_key"
            | "secret_key"
            | "secret_ref"
            | "redaction_applied"
    ) {
        return false;
    }
    matches!(
        low.as_str(),
        "api_key"
            | "apikey"
            | "api-key"
            | "x-api-key"
            | "x-ergon-gate"
            | "password"
            | "passwd"
            | "pwd"
            | "secret"
            | "client_secret"
            | "webhook_secret"
            | "signing_secret"
            | "token"
            | "access_token"
            | "refresh_token"
            | "auth_token"
            | "id_token"
            | "gate_token"
            | "hook_token"
            | "private_key"
            | "authorization"
    ) || low.ends_with("_api_key")
        || low.ends_with("_password")
        || low.ends_with("_passwd")
        || low.ends_with("_secret")
        || low.ends_with("_token")
        || low.ends_with("_private_key")
}

fn redact_kv_secrets(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Case 1: JSON / quoted key `"key": "value"` or `'key': 'value'`
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let key_start = i + 1;
            let mut key_end = key_start;
            while key_end < bytes.len() && is_word_char(bytes[key_end]) && key_end - key_start <= 48 {
                key_end += 1;
            }
            if key_end > key_start && key_end < bytes.len() && bytes[key_end] == q {
                let key = &input[key_start..key_end];
                if is_sensitive_key_name(key) {
                    let mut p = key_end + 1;
                    while p < bytes.len() && bytes[p] == b' ' {
                        p += 1;
                    }
                    if p < bytes.len() && (bytes[p] == b':' || bytes[p] == b'=') {
                        let sep = bytes[p] as char;
                        p += 1;
                        while p < bytes.len() && bytes[p] == b' ' {
                            p += 1;
                        }
                        if p < bytes.len() && (bytes[p] == b'"' || bytes[p] == b'\'') {
                            let vq = bytes[p];
                            let vstart = p + 1;
                            let mut vend = vstart;
                            while vend < bytes.len() && bytes[vend] != vq {
                                if bytes[vend] == b'\\' && vend + 1 < bytes.len() {
                                    vend += 2;
                                } else {
                                    vend += 1;
                                }
                            }
                            if vend < bytes.len() && bytes[vend] == vq {
                                let val = &input[vstart..vend];
                                out.push(q as char);
                                out.push_str(key);
                                out.push(q as char);
                                out.push(sep);
                                out.push(' ');
                                out.push(vq as char);
                                if val.is_empty() {
                                    // Preserve empty string values as empty
                                } else {
                                    out.push_str("[REDACTED]");
                                }
                                out.push(vq as char);
                                i = vend + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // Case 2: Unquoted key=value or key: value (env, query string, log text)
        let prev_ok = i == 0 || !is_word_char(bytes[i - 1]);
        if prev_ok && bytes[i].is_ascii_alphabetic() {
            let key_start = i;
            let mut key_end = i;
            while key_end < bytes.len() && is_word_char(bytes[key_end]) && key_end - key_start <= 48 {
                key_end += 1;
            }
            let key = &input[key_start..key_end];
            if is_sensitive_key_name(key) && key_end < bytes.len() {
                let mut p = key_end;
                let mut had_space_before_sep = false;
                while p < bytes.len() && bytes[p] == b' ' {
                    had_space_before_sep = true;
                    p += 1;
                }
                if p < bytes.len()
                    && (bytes[p] == b'=' || (bytes[p] == b':' && !had_space_before_sep))
                {
                    let sep = bytes[p];
                    let sep_end = p + 1;
                    let mut vstart = sep_end;
                    while vstart < bytes.len() && bytes[vstart] == b' ' {
                        vstart += 1;
                    }
                    if vstart < bytes.len() {
                        if bytes[vstart] == b'"' || bytes[vstart] == b'\'' {
                            let vq = bytes[vstart];
                            let mut vend = vstart + 1;
                            while vend < bytes.len() && bytes[vend] != vq {
                                if bytes[vend] == b'\\' && vend + 1 < bytes.len() {
                                    vend += 2;
                                } else {
                                    vend += 1;
                                }
                            }
                            if vend < bytes.len() && bytes[vend] == vq {
                                out.push_str(&input[key_start..vstart]);
                                out.push(vq as char);
                                if vend > vstart + 1 {
                                    out.push_str("[REDACTED]");
                                }
                                out.push(vq as char);
                                i = vend + 1;
                                continue;
                            }
                        } else if !is_val_stop(bytes[vstart]) {
                            let mut vend = vstart;
                            while vend < bytes.len() && !is_val_stop(bytes[vend]) {
                                vend += 1;
                            }
                            let val = &input[vstart..vend];
                            // Preserve harmless JSON/log literals like `null`, `true`, `false`, `none`,
                            // and auth scheme labels already redacted by `redact_auth_schemes` (`Bearer [REDACTED]`).
                            let low_val = val.to_ascii_lowercase();
                            let skip_val = matches!(
                                low_val.as_str(),
                                "null" | "none" | "true" | "false" | "[redacted]" | "bearer" | "basic"
                            ) || (sep == b':' && val.chars().all(|c| c.is_ascii_digit()));
                            if !skip_val {
                                out.push_str(&input[key_start..vstart]);
                                out.push_str("[REDACTED]");
                                i = vend;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn redact_token_prefixes(input: &str) -> String {
    const PREFIXES: &[(&str, usize, &str)] = &[
        ("sk_live_", 4, "[REDACTED_KEY]"),
        ("sk_test_", 4, "[REDACTED_KEY]"),
        ("pk_live_", 4, "[REDACTED_KEY]"),
        ("pk_test_", 4, "[REDACTED_KEY]"),
        ("rk_live_", 4, "[REDACTED_KEY]"),
        ("rk_test_", 4, "[REDACTED_KEY]"),
        ("sk-", 6, "[REDACTED_KEY]"),
        ("xoxb-", 4, "[REDACTED_TOKEN]"),
        ("xoxp-", 4, "[REDACTED_TOKEN]"),
        ("xoxa-", 4, "[REDACTED_TOKEN]"),
        ("xoxr-", 4, "[REDACTED_TOKEN]"),
        ("xoxs-", 4, "[REDACTED_TOKEN]"),
        ("xox-", 4, "[REDACTED_TOKEN]"),
        ("github_pat_", 6, "[REDACTED_TOKEN]"),
        ("ghp_", 6, "[REDACTED_TOKEN]"),
        ("gho_", 6, "[REDACTED_TOKEN]"),
        ("ghu_", 6, "[REDACTED_TOKEN]"),
        ("ghs_", 6, "[REDACTED_TOKEN]"),
        ("ghr_", 6, "[REDACTED_TOKEN]"),
        ("whtok_", 6, "[REDACTED_TOKEN]"),
        ("AIza", 12, "[REDACTED_TOKEN]"),
    ];

    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if prev_ok {
            let rem = &input[i..];
            // AWS Access Key ID: AKIA + 16 uppercase alphanumeric chars
            if rem.len() >= 20
                && rem.starts_with("AKIA")
                && rem.as_bytes()[4..20]
                    .iter()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            {
                let mut end = 20;
                while i + end < bytes.len() && is_token_char(bytes[i + end]) {
                    end += 1;
                }
                out.push_str("[REDACTED_AWS_KEY]");
                i += end;
                continue;
            }
            let mut matched = false;
            for &(pfx, min_tail, repl) in PREFIXES {
                if rem.starts_with(pfx) {
                    let mut end = pfx.len();
                    while i + end < bytes.len() && is_token_char(bytes[i + end]) {
                        end += 1;
                    }
                    let tail_len = end - pfx.len();
                    if tail_len >= min_tail {
                        out.push_str(repl);
                        i += end;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Redact common secret patterns (`Bearer ...`, `sk-...`, `xox...`, `api_key`,
/// `password`, `token`, `secret`, `ERGON_GATE_TOKEN`, `ERGON_MODEL_API_KEY`,
/// PEM private keys, and URL credentials) before writing any log, crash, or
/// diagnostic artifact.
pub fn redact_secrets(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let s = redact_pem_blocks(input);
    let s = redact_url_userinfo(&s);
    let s = redact_env_secret_names(&s);
    let s = redact_auth_schemes(&s);
    let s = redact_kv_secrets(&s);
    redact_token_prefixes(&s)
}

/// Redact both generic secret patterns and any concrete secret values loaded
/// from env vars, `config.json`, or `connector_secrets.json`.
pub fn redact_bundle_text(input: &str, known_secrets: &[String]) -> String {
    let mut out = redact_secrets(input);
    for secret in known_secrets {
        if secret.len() >= 4 && out.contains(secret.as_str()) {
            out = out.replace(secret.as_str(), "[REDACTED]");
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Thread-safe size-rotating JSONL logger
// ---------------------------------------------------------------------------

struct LoggerInner {
    logs_dir: PathBuf,
    max_bytes: u64,
    max_backups: usize,
    mirror_stderr: bool,
    mirror_all_stderr: bool,
    file: Option<File>,
    current_size: u64,
}

impl LoggerInner {
    fn log_path(&self) -> PathBuf {
        self.logs_dir.join("ergon.log")
    }

    fn backup_path(&self, idx: usize) -> PathBuf {
        self.logs_dir.join(format!("ergon.log.{idx}"))
    }

    fn ensure_open(&mut self) -> std::io::Result<()> {
        if self.file.is_some() {
            return Ok(());
        }
        let path = self.log_path();
        let existing_len = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let file = open_private_append(&path)?;
        self.current_size = existing_len;
        self.file = Some(file);
        Ok(())
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        if let Some(mut f) = self.file.take() {
            let _ = f.flush();
        }
        let active = self.log_path();
        if self.max_backups == 0 {
            let _ = fs::remove_file(&active);
        } else {
            let oldest = self.backup_path(self.max_backups);
            let _ = fs::remove_file(&oldest);
            for idx in (1..self.max_backups).rev() {
                let src = self.backup_path(idx);
                let dst = self.backup_path(idx + 1);
                if src.exists() {
                    let _ = fs::remove_file(&dst);
                    let _ = fs::rename(&src, &dst);
                }
            }
            if active.exists() {
                let first = self.backup_path(1);
                let _ = fs::remove_file(&first);
                let _ = fs::rename(&active, &first);
            }
        }
        let file = open_private_append(&active)?;
        self.current_size = 0;
        self.file = Some(file);
        Ok(())
    }

    fn write_entry(&mut self, level: LogLevel, target: &str, msg: &str) {
        let clean_msg = redact_secrets(msg);
        let ts_utc = now_utc_iso();
        let mut line = json!({
            "ts_utc": ts_utc,
            "level": level.as_str(),
            "target": target,
            "msg": clean_msg,
        })
        .to_string();
        line.push('\n');

        let bytes = line.as_bytes();
        if self.ensure_open().is_ok() {
            if self.current_size > 0 && self.current_size.saturating_add(bytes.len() as u64) > self.max_bytes {
                let _ = self.rotate();
            }
            if let Some(f) = self.file.as_mut() {
                if f.write_all(bytes).is_ok() {
                    let _ = f.flush();
                    self.current_size = self.current_size.saturating_add(bytes.len() as u64);
                }
            }
        }

        if self.mirror_stderr
            && (self.mirror_all_stderr || matches!(level, LogLevel::Warn | LogLevel::Error))
        {
            eprintln!("[{ts_utc}] {} [{target}] {clean_msg}", level.as_str());
        }
    }
}

pub struct RotatingLogger {
    inner: Mutex<LoggerInner>,
}

impl RotatingLogger {
    pub fn new(
        logs_dir: impl Into<PathBuf>,
        max_bytes: u64,
        max_backups: usize,
        mirror_stderr: bool,
    ) -> Self {
        Self::with_stderr_mode(logs_dir, max_bytes, max_backups, mirror_stderr, false)
    }

    pub fn with_stderr_mode(
        logs_dir: impl Into<PathBuf>,
        max_bytes: u64,
        max_backups: usize,
        mirror_stderr: bool,
        mirror_all_stderr: bool,
    ) -> Self {
        let logs_dir = logs_dir.into();
        let _ = fs::create_dir_all(&logs_dir);
        let max_bytes = max_bytes.max(128);
        Self {
            inner: Mutex::new(LoggerInner {
                logs_dir,
                max_bytes,
                max_backups,
                mirror_stderr,
                mirror_all_stderr,
                file: None,
                current_size: 0,
            }),
        }
    }

    pub fn log(&self, level: LogLevel, target: &str, msg: &str) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.write_entry(level, target, msg);
    }

    pub fn info(&self, target: &str, msg: &str) {
        self.log(LogLevel::Info, target, msg);
    }

    pub fn warn(&self, target: &str, msg: &str) {
        self.log(LogLevel::Warn, target, msg);
    }

    pub fn error(&self, target: &str, msg: &str) {
        self.log(LogLevel::Error, target, msg);
    }

    pub fn debug(&self, target: &str, msg: &str) {
        self.log(LogLevel::Debug, target, msg);
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .logs_dir
            .clone()
    }
}

fn global_slot() -> &'static RwLock<Option<Arc<RotatingLogger>>> {
    GLOBAL_LOGGER.get_or_init(|| RwLock::new(None))
}

fn env_log_settings() -> (u64, usize, bool, bool) {
    let max_bytes = std::env::var("ERGON_LOG_MAX_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&n| n >= 256)
        .unwrap_or(DEFAULT_LOG_MAX_BYTES);
    let max_backups = std::env::var("ERGON_LOG_MAX_FILES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .map(|n| n.min(20))
        .unwrap_or(DEFAULT_LOG_MAX_BACKUPS);
    let stderr_env = std::env::var("ERGON_LOG_STDERR")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let mirror_stderr = !matches!(stderr_env.as_str(), "0" | "false" | "off" | "no");
    let mirror_all = matches!(stderr_env.as_str(), "all" | "verbose" | "debug");
    (max_bytes, max_backups, mirror_stderr, mirror_all)
}

/// Initialize (or re-target) the global rotating logger to `$data_dir/logs/ergon.log`.
pub fn init(data_dir: &Path) {
    let (max_bytes, max_backups, mirror_stderr, mirror_all) = env_log_settings();
    let logger = Arc::new(RotatingLogger::with_stderr_mode(
        data_dir.join("logs"),
        max_bytes,
        max_backups,
        mirror_stderr,
        mirror_all,
    ));
    let mut g = global_slot().write().unwrap_or_else(|p| p.into_inner());
    *g = Some(logger);
}

/// Initialize the global rotating logger only if not already initialized.
pub fn init_if_unset(data_dir: &Path) {
    let already = global_slot()
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .is_some();
    if !already {
        init(data_dir);
    }
}

/// Initialize the global rotating logger with explicit options (useful for tests).
pub fn init_with_options(
    data_dir: &Path,
    max_bytes: u64,
    max_backups: usize,
    mirror_stderr: bool,
) {
    let logger = Arc::new(RotatingLogger::new(
        data_dir.join("logs"),
        max_bytes,
        max_backups,
        mirror_stderr,
    ));
    let mut g = global_slot().write().unwrap_or_else(|p| p.into_inner());
    *g = Some(logger);
}

pub fn log(level: LogLevel, target: &str, msg: &str) {
    let opt = global_slot()
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    if let Some(logger) = opt {
        logger.log(level, target, msg);
    } else if matches!(level, LogLevel::Warn | LogLevel::Error) {
        let clean = redact_secrets(msg);
        eprintln!("[{}] {} [{target}] {clean}", now_utc_iso(), level.as_str());
    }
}

pub fn info(target: &str, msg: &str) {
    log(LogLevel::Info, target, msg);
}

pub fn warn(target: &str, msg: &str) {
    log(LogLevel::Warn, target, msg);
}

pub fn error(target: &str, msg: &str) {
    log(LogLevel::Error, target, msg);
}

pub fn debug(target: &str, msg: &str) {
    log(LogLevel::Debug, target, msg);
}

// ---------------------------------------------------------------------------
// Crash capture & global panic hook
// ---------------------------------------------------------------------------

pub fn crashes_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("crashes")
}

pub fn logs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("logs")
}

/// Write a redacted structured JSON crash report to
/// `$data_dir/crashes/crash-<timestamp>-<pid>.json`.
pub fn write_crash_report(
    data_dir: &Path,
    message: &str,
    location: Option<&str>,
    thread_name: &str,
    backtrace: &str,
) -> std::io::Result<PathBuf> {
    let dir = crashes_dir(data_dir);
    fs::create_dir_all(&dir)?;

    let known = collect_known_secrets(data_dir);
    let clean_msg = redact_bundle_text(message, &known);
    let clean_loc = location.map(|l| redact_bundle_text(l, &known));
    let mut clean_bt = redact_bundle_text(backtrace, &known);
    if clean_bt.len() > MAX_BACKTRACE_BYTES {
        let mut end = MAX_BACKTRACE_BYTES;
        while end > 0 && !clean_bt.is_char_boundary(end) {
            end -= 1;
        }
        clean_bt.truncate(end);
        clean_bt.push_str("\n...[truncated]");
    }

    let ts_ms = records::now_millis();
    let ts_utc = format_utc_iso_ms(ts_ms);
    let stamp = compact_utc_stamp();
    let pid = std::process::id();
    let seq = CRASH_SEQ.fetch_add(1, Ordering::Relaxed);
    let filename = if seq == 0 {
        format!("crash-{stamp}-{pid}.json")
    } else {
        format!("crash-{stamp}-{pid}-{seq}.json")
    };
    let path = dir.join(&filename);

    let report = json!({
        "schema": "ergon.crash.v1",
        "ts_utc": ts_utc,
        "ts_ms": ts_ms,
        "pid": pid,
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "thread": thread_name,
        "location": clean_loc,
        "message": clean_msg,
        "backtrace": clean_bt,
        "redacted": true,
    });
    let pretty = serde_json::to_vec_pretty(&report).unwrap_or_else(|_| report.to_string().into_bytes());
    write_private_file(&path, &pretty)?;
    Ok(path)
}

fn panic_dir_slot() -> &'static RwLock<Option<PathBuf>> {
    PANIC_DATA_DIR.get_or_init(|| RwLock::new(None))
}

/// Install the global panic hook that writes a redacted JSON crash report to
/// `$data_dir/crashes/crash-<timestamp>-<pid>.json` and emits an ERROR log entry.
/// Calling this multiple times safely updates the target `data_dir` without stacking hooks.
pub fn install_panic_hook(data_dir: &Path) {
    {
        let mut g = panic_dir_slot().write().unwrap_or_else(|p| p.into_inner());
        *g = Some(data_dir.to_path_buf());
    }
    let _ = fs::create_dir_all(crashes_dir(data_dir));

    PANIC_HOOK_INSTALLED.get_or_init(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let thread_name = std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_string();
            // Malformed PDFs are isolated behind `catch_unwind` in `docs.rs` on
            // `ergon-pdf-extract`; never emit a crash report for recovered PDF reads.
            if thread_name == "ergon-pdf-extract" {
                return;
            }
            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "non-string panic payload".to_string()
            };
            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
            let bt = std::backtrace::Backtrace::force_capture().to_string();

            let target_dir = panic_dir_slot()
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .clone();

            if let Some(dir) = target_dir {
                match write_crash_report(&dir, &payload, location.as_deref(), &thread_name, &bt) {
                    Ok(report_path) => {
                        let fname = report_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("crash.json");
                        error(
                            "ergon::panic",
                            &format!(
                                "panic on thread '{thread_name}' at {}: {payload} (crash report: {fname})",
                                location.as_deref().unwrap_or("unknown")
                            ),
                        );
                    }
                    Err(e) => {
                        error(
                            "ergon::panic",
                            &format!(
                                "panic on thread '{thread_name}': {payload} (failed to write crash report: {e})"
                            ),
                        );
                    }
                }
            }
            prev(info);
        }));
    });
}

// ---------------------------------------------------------------------------
// Secret collector (in-memory only, used to scrub diagnostic bundles)
// ---------------------------------------------------------------------------

fn collect_known_secrets(root_data_dir: &Path) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for env_name in [
        "ERGON_MODEL_API_KEY",
        "ERGON_GATE_TOKEN",
        "ERGON_WEBHOOK_SECRET",
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "ERGON_SMTP_PASS",
        "ERGON_IMAP_PASS",
    ] {
        if let Ok(val) = std::env::var(env_name) {
            let trimmed = val.trim();
            if trimmed.len() >= 4 {
                set.insert(trimmed.to_string());
            }
        }
    }

    // Root config.json -> api_key
    if let Ok(raw) = fs::read_to_string(root_data_dir.join("config.json")) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            if let Some(k) = v["api_key"].as_str().map(str::trim).filter(|s| s.len() >= 4) {
                set.insert(k.to_string());
            }
        }
    }

    // Scan root and per-person connector_secrets.json
    let mut secret_files = vec![connectors::connector_secrets_path(root_data_dir)];
    let people_dir = root_data_dir.join("people");
    if let Ok(rd) = fs::read_dir(&people_dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                secret_files.push(connectors::connector_secrets_path(&p));
            }
        }
    }
    for sf in secret_files {
        if let Ok(raw) = fs::read_to_string(&sf) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                collect_json_string_leaves(&v["secrets"], &mut set);
                collect_json_string_leaves(&v["webhook_secrets"], &mut set);
                collect_json_string_leaves(&v["oauth_tokens"], &mut set);
            }
        }
    }

    // Longest first so substring secrets don't leave fragments behind.
    let mut list: Vec<String> = set.into_iter().collect();
    list.sort_by(|a, b| b.len().cmp(&a.len()));
    list
}

fn collect_json_string_leaves(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::String(s) => {
            let t = s.trim();
            if t.len() >= 4 {
                out.insert(t.to_string());
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_json_string_leaves(item, out);
            }
        }
        Value::Object(map) => {
            for (k, val) in map {
                // Skip non-secret metadata fields like `token_type` ("Bearer") or `scope`
                if matches!(k.as_str(), "token_type" | "scope" | "updated_ms" | "expires_at_ms") {
                    continue;
                }
                collect_json_string_leaves(val, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// One-click IT Diagnostic Bundle (.zip)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct DiagnosticBundleResult {
    pub zip_bytes: Vec<u8>,
    pub sha256: String,
    pub filename: String,
    pub log_files: usize,
    pub crash_reports: usize,
    pub security_events: usize,
}

fn count_workspace_files_bounded(dir: &Path, limit: usize) -> usize {
    let mut count = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let Ok(rd) = fs::read_dir(&cur) else {
            continue;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                count += 1;
                if count >= limit {
                    return count;
                }
            }
        }
    }
    count
}

fn read_bounded_utf8(path: &Path, max_bytes: usize) -> Option<String> {
    let raw = fs::read(path).ok()?;
    let slice = if raw.len() > max_bytes {
        &raw[raw.len() - max_bytes..]
    } else {
        &raw[..]
    };
    Some(String::from_utf8_lossy(slice).into_owned())
}

fn collect_log_entries(data_dir: &Path, known_secrets: &[String]) -> Vec<(String, Vec<u8>)> {
    let dir = logs_dir(data_dir);
    let mut paths: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if !p.is_file() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name == "ergon.log" || name.starts_with("ergon.log.") {
                paths.push((name.to_string(), p));
            }
        }
    }
    paths.sort_by(|a, b| {
        let key = |n: &str| -> (u8, usize, String) {
            if n == "ergon.log" {
                (0, 0, n.to_string())
            } else if let Some(num) = n.strip_prefix("ergon.log.").and_then(|s| s.parse::<usize>().ok()) {
                (1, num, n.to_string())
            } else {
                (2, usize::MAX, n.to_string())
            }
        };
        key(&a.0).cmp(&key(&b.0))
    });

    let mut out = Vec::new();
    for (name, p) in paths {
        if let Some(raw) = read_bounded_utf8(&p, MAX_BUNDLE_LOG_FILE_BYTES) {
            let clean = redact_bundle_text(&raw, known_secrets);
            out.push((format!("logs/{name}"), clean.into_bytes()));
        }
    }
    out
}

fn collect_crash_entries(data_dir: &Path, known_secrets: &[String]) -> Vec<(String, Vec<u8>)> {
    let dir = crashes_dir(data_dir);
    let mut paths: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if !p.is_file() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.ends_with(".json") && !name.starts_with('.') {
                paths.push((name.to_string(), p));
            }
        }
    }
    // Newest timestamp filename first.
    paths.sort_by(|a, b| b.0.cmp(&a.0));
    paths.truncate(MAX_BUNDLE_CRASH_FILES);

    let mut out = Vec::new();
    for (name, p) in paths {
        if let Some(raw) = read_bounded_utf8(&p, MAX_BUNDLE_CRASH_FILE_BYTES) {
            let clean = redact_bundle_text(&raw, known_secrets);
            out.push((format!("crashes/{name}"), clean.into_bytes()));
        }
    }
    out
}

fn collect_connector_summaries(cfg: &Config, people: &PeopleState) -> Value {
    let mut triggers_summary = Vec::new();
    let mut outbound_summary = Vec::new();
    let mut mcp_summary = Vec::new();

    for person in &people.people {
        let p_cfg = cfg.person_config(person);
        let policy = crate::load_workspace_policy(&p_cfg.data_dir);

        if connectors::connectors_path(&p_cfg.data_dir).exists() {
            let listed = connectors::list_connectors_json(&p_cfg.data_dir, &policy);
            if let Some(arr) = listed["triggers"].as_array() {
                for t in arr {
                    triggers_summary.push(json!({
                        "workspace_id": person.id,
                        "id": t["id"].as_str().unwrap_or(""),
                        "name": t["name"].as_str().unwrap_or(""),
                        "kind": t["kind"].as_str().unwrap_or(""),
                        "enabled": t["enabled"].as_bool().unwrap_or(false),
                        "has_credentials": t["has_secret"].as_bool().unwrap_or(false),
                    }));
                }
            }
            if let Some(arr) = listed["outbound"].as_array() {
                for o in arr {
                    outbound_summary.push(json!({
                        "workspace_id": person.id,
                        "id": o["id"].as_str().unwrap_or(""),
                        "name": o["name"].as_str().unwrap_or(""),
                        "kind": o["kind"].as_str().unwrap_or(""),
                        "enabled": o["enabled"].as_bool().unwrap_or(false),
                        "has_credentials": o["has_secret"].as_bool().unwrap_or(false),
                    }));
                }
            }
        }

        if mcp::mcp_servers_path(&p_cfg.data_dir).exists() {
            let listed = mcp::list_mcp_servers_json(&p_cfg.data_dir, &policy);
            if let Some(arr) = listed["servers"].as_array() {
                for s in arr {
                    let tool_count = s["tools"].as_array().map(|a| a.len()).unwrap_or(0);
                    mcp_summary.push(json!({
                        "workspace_id": person.id,
                        "id": s["id"].as_str().unwrap_or(""),
                        "name": s["name"].as_str().unwrap_or(""),
                        "transport": s["transport"].as_str().unwrap_or(""),
                        "enabled": s["enabled"].as_bool().unwrap_or(false),
                        "tool_count": tool_count,
                    }));
                }
            }
        }
    }

    json!({
        "triggers": triggers_summary,
        "outbound": outbound_summary,
        "mcp_servers": mcp_summary,
        "secrets_excluded": true,
    })
}

fn collect_security_events_json(root_data_dir: &Path) -> (Value, usize) {
    let (verified, chain_len, head_hash, first_bad_index) = tripwire::verify_log(root_data_dir);
    let raw_events = fs::read_to_string(tripwire::events_path(root_data_dir))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["events"].as_array().cloned())
        .unwrap_or_default();
    let count = raw_events.len();
    let body = json!({
        "verified": verified,
        "chain_length": chain_len,
        "head_hash": head_hash,
        "first_bad_index": first_bad_index,
        "events": raw_events,
    });
    (body, count)
}

/// Lightweight non-secret summary for the Help & Support dialog.
pub fn diagnostic_summary_json(
    cfg: &Config,
    uptime_secs: u64,
    model_configured: bool,
) -> Value {
    let log_files = fs::read_dir(logs_dir(&cfg.data_dir))
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.path().is_file()
                        && e.file_name()
                            .to_str()
                            .is_some_and(|n| n == "ergon.log" || n.starts_with("ergon.log."))
                })
                .count()
        })
        .unwrap_or(0);
    let crash_reports = fs::read_dir(crashes_dir(&cfg.data_dir))
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.path().is_file()
                        && e.file_name()
                            .to_str()
                            .is_some_and(|n| n.ends_with(".json") && !n.starts_with('.'))
                })
                .count()
        })
        .unwrap_or(0);
    let people = PeopleState::load(cfg);
    json!({
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "uptime_secs": uptime_secs,
        "workspace_count": people.people.len(),
        "model_configured": model_configured,
        "log_files": log_files,
        "crash_reports": crash_reports,
    })
}

/// Build a redacted one-click IT diagnostic ZIP bundle containing:
/// - `system.json` (version, OS, arch, uptime, workspace counts, model_configured bool)
/// - `logs/*` (rotated logs from `$ERGON_DATA_DIR/logs/`)
/// - `crashes/*` (recent crash reports from `$ERGON_DATA_DIR/crashes/`)
/// - `security_events.json` (redacted hash-chained security events)
/// - `connectors.json` (names/enabled flags only, zero secrets)
/// - `README.txt`
pub fn build_diagnostic_bundle_zip(
    cfg: &Config,
    uptime_secs: u64,
    scheduler_enabled: bool,
    live_model_override: Option<(bool, String, bool)>,
) -> Result<DiagnosticBundleResult, String> {
    let known_secrets = collect_known_secrets(&cfg.data_dir);
    let people = PeopleState::load(cfg);

    let (model_configured, model_id, model_is_loopback) = match live_model_override {
        Some(tup) => tup,
        None => (
            cfg.model_configured(),
            cfg.model_id.clone(),
            crate::is_loopback_url(&cfg.model_base_url),
        ),
    };

    let mut workspaces = Vec::new();
    for person in &people.people {
        let p_cfg = cfg.person_config(person);
        let file_count = count_workspace_files_bounded(&p_cfg.workspace_dir, 10_000);
        workspaces.push(json!({
            "id": person.id,
            "role": person.role,
            "file_count": file_count,
        }));
    }

    let log_entries = collect_log_entries(&cfg.data_dir, &known_secrets);
    let crash_entries = collect_crash_entries(&cfg.data_dir, &known_secrets);
    let (sec_json, sec_count) = collect_security_events_json(&cfg.data_dir);
    let conn_json = collect_connector_summaries(cfg, &people);

    let generated_at_utc = now_utc_iso();
    let system_json = json!({
        "schema": "ergon.diagnostics.v1",
        "generated_at_utc": generated_at_utc,
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "uptime_secs": uptime_secs,
        "workspace_count": people.people.len(),
        "workspaces": workspaces,
        "model_configured": model_configured,
        "model_id": redact_bundle_text(&model_id, &known_secrets),
        "model_is_loopback": model_is_loopback,
        "scheduler_enabled": scheduler_enabled,
        "log_files_included": log_entries.len(),
        "crash_reports_included": crash_entries.len(),
        "security_events_count": sec_count,
        "redaction_applied": true,
    });

    let system_str = redact_bundle_text(
        &serde_json::to_string_pretty(&system_json).map_err(|e| format!("serialize system.json: {e}"))?,
        &known_secrets,
    );
    let sec_str = redact_bundle_text(
        &serde_json::to_string_pretty(&sec_json).map_err(|e| format!("serialize security_events.json: {e}"))?,
        &known_secrets,
    );
    let conn_str = redact_bundle_text(
        &serde_json::to_string_pretty(&conn_json).map_err(|e| format!("serialize connectors.json: {e}"))?,
        &known_secrets,
    );
    let readme = format!(
        "Ergon IT Diagnostic Bundle\n\
         ==========================\n\
         Generated (UTC): {generated_at_utc}\n\
         App version:     {version}\n\
         Platform:        {os}/{arch}\n\n\
         Contents:\n\
         - system.json          Runtime metadata, uptime, workspace counts, model readiness\n\
         - logs/                Rotated application logs (ergon.log*)\n\
         - crashes/             Recent crash reports (JSON)\n\
         - security_events.json Tamper-evident security event log\n\
         - connectors.json      Connector names and enabled status only\n\n\
         Privacy & Secret Redaction:\n\
         All API keys, bearer tokens, passwords, private keys, connector secrets,\n\
         and credentialed URLs have been automatically redacted before packaging.\n",
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
    );

    let mut owned_files: Vec<(String, Vec<u8>)> = Vec::new();
    owned_files.push(("README.txt".to_string(), readme.into_bytes()));
    owned_files.push(("system.json".to_string(), system_str.into_bytes()));
    owned_files.push(("security_events.json".to_string(), sec_str.into_bytes()));
    owned_files.push(("connectors.json".to_string(), conn_str.into_bytes()));
    let log_files = log_entries.len();
    let crash_reports = crash_entries.len();
    owned_files.extend(log_entries);
    owned_files.extend(crash_entries);

    let refs: Vec<(&str, &[u8])> = owned_files
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_slice()))
        .collect();
    let zip_bytes = docs::zip_write_refs(&refs);
    let sha256 = receipts::sha256_hex(&zip_bytes);
    let stamp = compact_utc_stamp();
    let filename = format!("ergon-diagnostics-{stamp}.zip");

    Ok(DiagnosticBundleResult {
        zip_bytes,
        sha256,
        filename,
        log_files,
        crash_reports,
        security_events: sec_count,
    })
}

// ---------------------------------------------------------------------------
// First-run onboarding & IT Admin Contact preferences
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HelpPrefs {
    pub onboarding_completed: bool,
    pub onboarding_completed_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdminContact {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub channel_url: String,
    pub notes: String,
    pub updated_ms: u64,
    pub source: &'static str,
}

impl AdminContact {
    pub fn is_configured(&self) -> bool {
        !self.name.trim().is_empty()
            || !self.email.trim().is_empty()
            || !self.phone.trim().is_empty()
            || !self.channel_url.trim().is_empty()
            || !self.notes.trim().is_empty()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "email": self.email,
            "phone": self.phone,
            "channel_url": self.channel_url,
            "notes": self.notes,
            "configured": self.is_configured(),
            "source": self.source,
            "updated_ms": self.updated_ms,
        })
    }
}

pub fn help_prefs_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("help_prefs.json")
}

pub fn admin_contact_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("admin_contact.json")
}

pub fn load_help_prefs(person_data_dir: &Path) -> HelpPrefs {
    let path = help_prefs_path(person_data_dir);
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            return HelpPrefs {
                onboarding_completed: v["onboarding_completed"].as_bool().unwrap_or(false),
                onboarding_completed_at_ms: v["onboarding_completed_at_ms"].as_u64(),
            };
        }
    }
    HelpPrefs::default()
}

pub fn save_help_prefs(person_data_dir: &Path, completed: bool) -> Result<HelpPrefs, String> {
    let prefs = HelpPrefs {
        onboarding_completed: completed,
        onboarding_completed_at_ms: if completed { Some(records::now_millis()) } else { None },
    };
    let body = serde_json::to_vec_pretty(&json!({
        "onboarding_completed": prefs.onboarding_completed,
        "onboarding_completed_at_ms": prefs.onboarding_completed_at_ms,
    }))
    .map_err(|e| format!("serialize help_prefs.json: {e}"))?;
    atomic_write_bytes(&help_prefs_path(person_data_dir), &body)
        .map_err(|e| format!("save help_prefs.json: {e}"))?;
    Ok(prefs)
}

fn clean_field(v: Option<&str>, max_len: usize) -> String {
    let s = v.unwrap_or("").trim();
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect()
    }
}

fn env_trimmed(name: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn validate_channel_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let low = trimmed.to_ascii_lowercase();
    if low.starts_with("https://")
        || low.starts_with("http://")
        || low.starts_with("mailto:")
        || low.starts_with("tel:")
        || low.starts_with("slack://")
    {
        Ok(clean_field(Some(trimmed), 320))
    } else {
        Err("Support / Slack / ticket URL must start with https://, http://, mailto:, tel:, or slack://".to_string())
    }
}

pub fn load_admin_contact(root_data_dir: &Path) -> AdminContact {
    let path = admin_contact_path(root_data_dir);
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            let url_raw = clean_field(v["channel_url"].as_str(), 320);
            let channel_url = validate_channel_url(&url_raw).unwrap_or_default();
            let c = AdminContact {
                name: clean_field(v["name"].as_str(), 120),
                email: clean_field(v["email"].as_str(), 180),
                phone: clean_field(v["phone"].as_str(), 64),
                channel_url,
                notes: clean_field(v["notes"].as_str(), 400),
                updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
                source: "saved",
            };
            if c.is_configured() {
                return c;
            }
        }
    }

    let env_url_raw = {
        let u = env_trimmed("ERGON_ADMIN_CONTACT_URL");
        if !u.is_empty() {
            u
        } else {
            let s = env_trimmed("ERGON_ADMIN_CONTACT_SLACK");
            if !s.is_empty() {
                s
            } else {
                env_trimmed("ERGON_ADMIN_CONTACT_TICKET_URL")
            }
        }
    };
    let env_contact = AdminContact {
        name: clean_field(Some(&env_trimmed("ERGON_ADMIN_CONTACT_NAME")), 120),
        email: clean_field(Some(&env_trimmed("ERGON_ADMIN_CONTACT_EMAIL")), 180),
        phone: clean_field(Some(&env_trimmed("ERGON_ADMIN_CONTACT_PHONE")), 64),
        channel_url: validate_channel_url(&env_url_raw).unwrap_or_default(),
        notes: clean_field(Some(&env_trimmed("ERGON_ADMIN_CONTACT_NOTES")), 400),
        updated_ms: 0,
        source: "env",
    };
    if env_contact.is_configured() {
        return env_contact;
    }

    AdminContact {
        source: "default",
        ..AdminContact::default()
    }
}

pub fn save_admin_contact(
    root_data_dir: &Path,
    name: &str,
    email: &str,
    phone: &str,
    channel_url: &str,
    notes: &str,
) -> Result<AdminContact, String> {
    let valid_url = validate_channel_url(channel_url)?;
    let updated_ms = records::now_millis();
    let contact = AdminContact {
        name: clean_field(Some(name), 120),
        email: clean_field(Some(email), 180),
        phone: clean_field(Some(phone), 64),
        channel_url: valid_url,
        notes: clean_field(Some(notes), 400),
        updated_ms,
        source: "saved",
    };
    let body = serde_json::to_vec_pretty(&json!({
        "name": contact.name,
        "email": contact.email,
        "phone": contact.phone,
        "channel_url": contact.channel_url,
        "notes": contact.notes,
        "updated_ms": contact.updated_ms,
    }))
    .map_err(|e| format!("serialize admin_contact.json: {e}"))?;
    atomic_write_bytes(&admin_contact_path(root_data_dir), &body)
        .map_err(|e| format!("save admin_contact.json: {e}"))?;
    Ok(contact)
}

pub fn help_config_json(
    root_data_dir: &Path,
    person_data_dir: &Path,
    can_edit_admin_contact: bool,
) -> Value {
    let prefs = load_help_prefs(person_data_dir);
    let contact = load_admin_contact(root_data_dir);
    json!({
        "onboarding_completed": prefs.onboarding_completed,
        "onboarding_completed_at_ms": prefs.onboarding_completed_at_ms,
        "admin_contact": contact.to_json(),
        "can_edit_admin_contact": can_edit_admin_contact,
        "diagnostics_url": "/api/diagnostics/bundle",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secret_patterns_and_preserves_json() {
        let sample = concat!(
            "Authorization: Bearer tok-secret-8888 ",
            "key=sk-live-secret-9999 ",
            "slack=xoxb-123456-abcdef ",
            "password=supersecret ",
            "ERGON_MODEL_API_KEY=sk-env-9999 ",
            "ERGON_GATE_TOKEN=gate-secret-777 ",
            "https://alice:hunter2@mail.example.com/inbox ",
            "{\"api_key\": \"my-api-secret\", \"has_secret\": true, \"total_tokens\": 420}"
        );
        let out = redact_secrets(sample);
        for forbidden in [
            "tok-secret-8888",
            "sk-live-secret-9999",
            "xoxb-123456-abcdef",
            "supersecret",
            "sk-env-9999",
            "gate-secret-777",
            "ERGON_MODEL_API_KEY",
            "ERGON_GATE_TOKEN",
            "hunter2",
            "my-api-secret",
        ] {
            assert!(!out.contains(forbidden), "secret leaked ({forbidden}) in: {out}");
        }
        assert!(out.contains("Bearer [REDACTED]"));
        assert!(out.contains("\"has_secret\": true"));
        assert!(out.contains("\"total_tokens\": 420"));
        assert!(out.contains("https://[REDACTED]@mail.example.com/inbox"));
    }

    #[test]
    fn channel_url_rejects_javascript_scheme() {
        assert!(validate_channel_url("javascript:alert(1)").is_err());
        assert!(validate_channel_url("https://help.example.com/tickets").is_ok());
        assert!(validate_channel_url("mailto:it@example.com").is_ok());
        assert!(validate_channel_url("slack://open").is_ok());
    }
}
