//! Automated full-instance `ERGON_DATA_DIR` backup, offline verification,
//! retention pruning, offsite replication (local folder & S3 SigV4), and
//! atomic verified restore.
//!
//! Uses only crates already in `Cargo.toml` (`miniz_oxide`, `sha2`, `ureq`,
//! `serde_json`) plus `crate::tripwire::hmac_sha256` and `crate::docs::crc32`.

use crate::audit_bundle::EvidenceVerification;
use crate::docs::crc32;
use crate::receipts::sha256_hex;
use crate::tools::atomic_write_bytes;
use crate::tripwire::hmac_sha256;
use crate::{records, sandbox_path_resolving};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const BACKUP_MANIFEST_SCHEMA: &str = "ergon.backup_manifest.v1";
pub const BACKUP_MANIFEST_NAME: &str = "backup-manifest.json";
pub const BACKUP_CONFIG_FILE: &str = "backup_config.json";
pub const BACKUP_STATE_FILE: &str = "backup_state.json";
pub const DEFAULT_INTERVAL_HOURS: u64 = 24;
pub const DEFAULT_RETENTION_COUNT: usize = 14;
pub const DEFAULT_S3_REGION: &str = "us-east-1";
pub const DEFAULT_S3_PREFIX: &str = "ergon-backups";

const MAX_BACKUP_ENTRIES: usize = 100_000;
const MAX_BACKUP_ENTRY_BYTES: usize = 128 * 1024 * 1024;
const MAX_BACKUP_TOTAL_BYTES: usize = 512 * 1024 * 1024;

static BACKUP_SEQ: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Configuration & State
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupOffsiteConfig {
    /// `"none"`, `"folder"`, or `"s3"`.
    pub kind: String,
    pub folder_path: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: String,
    pub s3_prefix: String,
    pub s3_path_style: bool,
}

impl Default for BackupOffsiteConfig {
    fn default() -> Self {
        Self {
            kind: "none".to_string(),
            folder_path: None,
            s3_endpoint: None,
            s3_bucket: None,
            s3_region: DEFAULT_S3_REGION.to_string(),
            s3_prefix: DEFAULT_S3_PREFIX.to_string(),
            s3_path_style: true,
        }
    }
}

impl BackupOffsiteConfig {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "folder_path": self.folder_path.as_deref().unwrap_or(""),
            "s3_endpoint": self.s3_endpoint.as_deref().unwrap_or(""),
            "s3_bucket": self.s3_bucket.as_deref().unwrap_or(""),
            "s3_region": self.s3_region,
            "s3_prefix": self.s3_prefix,
            "s3_path_style": self.s3_path_style,
            "s3_credentials_configured": s3_credentials_from_env().is_some(),
        })
    }

    pub fn from_json(v: &Value) -> Self {
        let mut cfg = Self::default();
        if let Some(k) = v["kind"].as_str() {
            let low = k.trim().to_ascii_lowercase();
            if matches!(low.as_str(), "none" | "folder" | "s3") {
                cfg.kind = low;
            }
        }
        cfg.folder_path = non_empty_str(v["folder_path"].as_str());
        cfg.s3_endpoint = non_empty_str(v["s3_endpoint"].as_str());
        cfg.s3_bucket = non_empty_str(v["s3_bucket"].as_str());
        if let Some(r) = non_empty_str(v["s3_region"].as_str()) {
            cfg.s3_region = r;
        }
        if let Some(p) = v["s3_prefix"].as_str() {
            cfg.s3_prefix = p.trim().trim_matches('/').to_string();
        }
        if let Some(ps) = v["s3_path_style"].as_bool() {
            cfg.s3_path_style = ps;
        }
        cfg
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub retention_count: usize,
    /// Empty string means default `$ERGON_DATA_DIR/backups`.
    pub local_dir: String,
    pub offsite: BackupOffsiteConfig,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_hours: DEFAULT_INTERVAL_HOURS,
            retention_count: DEFAULT_RETENTION_COUNT,
            local_dir: String::new(),
            offsite: BackupOffsiteConfig::default(),
        }
    }
}

impl BackupConfig {
    pub fn effective_local_dir(&self, data_dir: &Path) -> PathBuf {
        let trimmed = self.local_dir.trim();
        if trimmed.is_empty() {
            data_dir.join("backups")
        } else {
            let p = PathBuf::from(trimmed);
            if p.is_absolute() {
                p
            } else {
                data_dir.join(p)
            }
        }
    }

    pub fn to_json(&self, data_dir: &Path) -> Value {
        json!({
            "enabled": self.enabled,
            "interval_hours": self.interval_hours,
            "retention_count": self.retention_count,
            "local_dir": self.local_dir,
            "effective_local_dir": self.effective_local_dir(data_dir).to_string_lossy(),
            "offsite": self.offsite.to_json(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.interval_hours == 0 || self.interval_hours > 8_760 {
            return Err("interval_hours must be between 1 and 8760".to_string());
        }
        if self.retention_count == 0 || self.retention_count > 1_000 {
            return Err("retention_count must be between 1 and 1000".to_string());
        }
        match self.offsite.kind.as_str() {
            "none" => Ok(()),
            "folder" => {
                if self
                    .offsite
                    .folder_path
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
                {
                    return Err("offsite folder target requires a non-empty folder_path".to_string());
                }
                Ok(())
            }
            "s3" => {
                let bucket = self
                    .offsite
                    .s3_bucket
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("");
                if bucket.is_empty() || bucket.contains('/') || bucket.contains("..") {
                    return Err("offsite S3 target requires a valid s3_bucket name".to_string());
                }
                if let Some(ep) = self.offsite.s3_endpoint.as_deref() {
                    let trimmed = ep.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with("http://")
                        && !trimmed.starts_with("https://")
                    {
                        return Err(
                            "offsite S3 endpoint must start with http:// or https://".to_string()
                        );
                    }
                }
                if self.offsite.s3_region.trim().is_empty() {
                    return Err("offsite S3 target requires a non-empty s3_region".to_string());
                }
                Ok(())
            }
            other => Err(format!(
                "offsite kind must be 'none', 'folder', or 's3' (got '{other}')"
            )),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BackupRuntimeState {
    pub last_run_ms: Option<u64>,
    pub last_backup_name: Option<String>,
    pub last_backup_sha256: Option<String>,
    pub last_file_count: Option<usize>,
    pub last_zip_bytes: Option<u64>,
    pub last_offsite_status: Option<String>,
    pub last_error: Option<String>,
}

impl BackupRuntimeState {
    pub fn to_json(&self) -> Value {
        json!({
            "last_run_ms": self.last_run_ms,
            "last_backup_name": self.last_backup_name,
            "last_backup_sha256": self.last_backup_sha256,
            "last_file_count": self.last_file_count,
            "last_zip_bytes": self.last_zip_bytes,
            "last_offsite_status": self.last_offsite_status,
            "last_error": self.last_error,
        })
    }
}

#[derive(Clone, Debug)]
pub struct S3Credentials {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}

pub fn s3_credentials_from_env() -> Option<S3Credentials> {
    let access_key = std::env::var("ERGON_BACKUP_S3_ACCESS_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    let secret_key = std::env::var("ERGON_BACKUP_S3_SECRET_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    let session_token = std::env::var("ERGON_BACKUP_S3_SESSION_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(S3Credentials {
        access_key,
        secret_key,
        session_token,
    })
}

fn non_empty_str(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn env_bool(name: &str) -> Option<bool> {
    let raw = std::env::var(name).ok()?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn load_backup_config(data_dir: &Path) -> BackupConfig {
    let mut cfg = BackupConfig::default();
    let path = data_dir.join(BACKUP_CONFIG_FILE);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(en) = v["enabled"].as_bool() {
                cfg.enabled = en;
            }
            if let Some(h) = v["interval_hours"].as_u64().filter(|&n| n >= 1) {
                cfg.interval_hours = h.min(8_760);
            }
            if let Some(r) = v["retention_count"].as_u64().filter(|&n| n >= 1) {
                cfg.retention_count = (r as usize).min(1_000);
            }
            if let Some(d) = v["local_dir"].as_str() {
                cfg.local_dir = d.trim().to_string();
            }
            if v.get("offsite").is_some_and(Value::is_object) {
                cfg.offsite = BackupOffsiteConfig::from_json(&v["offsite"]);
            }
        }
    }

    // Apply environment overrides (credentials always come exclusively from env).
    if let Some(en) = env_bool("ERGON_BACKUP_ENABLED") {
        cfg.enabled = en;
    }
    if let Ok(h) = std::env::var("ERGON_BACKUP_INTERVAL_HOURS") {
        if let Ok(n) = h.trim().parse::<u64>() {
            if n >= 1 {
                cfg.interval_hours = n.min(8_760);
            }
        }
    }
    if let Ok(r) = std::env::var("ERGON_BACKUP_RETENTION_COUNT") {
        if let Ok(n) = r.trim().parse::<usize>() {
            if n >= 1 {
                cfg.retention_count = n.min(1_000);
            }
        }
    }
    if let Ok(d) = std::env::var("ERGON_BACKUP_LOCAL_DIR") {
        let trimmed = d.trim();
        if !trimmed.is_empty() {
            cfg.local_dir = trimmed.to_string();
        }
    }
    if let Ok(kind) = std::env::var("ERGON_BACKUP_OFFSITE_KIND") {
        let low = kind.trim().to_ascii_lowercase();
        if matches!(low.as_str(), "none" | "folder" | "s3") {
            cfg.offsite.kind = low;
        }
    }
    if let Ok(folder) = std::env::var("ERGON_BACKUP_OFFSITE_FOLDER") {
        let trimmed = folder.trim();
        if !trimmed.is_empty() {
            cfg.offsite.folder_path = Some(trimmed.to_string());
            if cfg.offsite.kind == "none" {
                cfg.offsite.kind = "folder".to_string();
            }
        }
    }
    if let Ok(ep) = std::env::var("ERGON_BACKUP_S3_ENDPOINT") {
        let trimmed = ep.trim();
        if !trimmed.is_empty() {
            cfg.offsite.s3_endpoint = Some(trimmed.to_string());
        }
    }
    if let Ok(b) = std::env::var("ERGON_BACKUP_S3_BUCKET") {
        let trimmed = b.trim();
        if !trimmed.is_empty() {
            cfg.offsite.s3_bucket = Some(trimmed.to_string());
            if cfg.offsite.kind == "none" {
                cfg.offsite.kind = "s3".to_string();
            }
        }
    }
    if let Ok(reg) = std::env::var("ERGON_BACKUP_S3_REGION") {
        let trimmed = reg.trim();
        if !trimmed.is_empty() {
            cfg.offsite.s3_region = trimmed.to_string();
        }
    }
    if let Ok(pfx) = std::env::var("ERGON_BACKUP_S3_PREFIX") {
        cfg.offsite.s3_prefix = pfx.trim().trim_matches('/').to_string();
    }
    if let Some(ps) = env_bool("ERGON_BACKUP_S3_PATH_STYLE") {
        cfg.offsite.s3_path_style = ps;
    }

    cfg
}

pub fn save_backup_config(data_dir: &Path, cfg: &BackupConfig) -> Result<(), String> {
    cfg.validate()?;
    let body = serde_json::to_vec_pretty(&json!({
        "enabled": cfg.enabled,
        "interval_hours": cfg.interval_hours,
        "retention_count": cfg.retention_count,
        "local_dir": cfg.local_dir,
        "offsite": {
            "kind": cfg.offsite.kind,
            "folder_path": cfg.offsite.folder_path.as_deref().unwrap_or(""),
            "s3_endpoint": cfg.offsite.s3_endpoint.as_deref().unwrap_or(""),
            "s3_bucket": cfg.offsite.s3_bucket.as_deref().unwrap_or(""),
            "s3_region": cfg.offsite.s3_region,
            "s3_prefix": cfg.offsite.s3_prefix,
            "s3_path_style": cfg.offsite.s3_path_style,
        },
    }))
    .map_err(|e| format!("serialize backup config: {e}"))?;
    atomic_write_bytes(&data_dir.join(BACKUP_CONFIG_FILE), &body)
        .map_err(|e| format!("save backup config: {e}"))
}

pub fn update_backup_config_from_json(
    data_dir: &Path,
    patch: &Value,
) -> Result<BackupConfig, String> {
    let mut cfg = load_backup_config(data_dir);
    if let Some(en) = patch["enabled"].as_bool() {
        cfg.enabled = en;
    }
    if let Some(h) = patch["interval_hours"].as_u64() {
        cfg.interval_hours = h;
    }
    if let Some(r) = patch["retention_count"].as_u64() {
        cfg.retention_count = r as usize;
    }
    if let Some(d) = patch["local_dir"].as_str() {
        cfg.local_dir = d.trim().to_string();
    }
    if let Some(off) = patch.get("offsite").filter(|v| v.is_object()) {
        if let Some(k) = off["kind"].as_str() {
            cfg.offsite.kind = k.trim().to_ascii_lowercase();
        }
        if off.get("folder_path").is_some() {
            cfg.offsite.folder_path = non_empty_str(off["folder_path"].as_str());
        }
        if off.get("s3_endpoint").is_some() {
            cfg.offsite.s3_endpoint = non_empty_str(off["s3_endpoint"].as_str());
        }
        if off.get("s3_bucket").is_some() {
            cfg.offsite.s3_bucket = non_empty_str(off["s3_bucket"].as_str());
        }
        if let Some(reg) = non_empty_str(off["s3_region"].as_str()) {
            cfg.offsite.s3_region = reg;
        }
        if let Some(pfx) = off["s3_prefix"].as_str() {
            cfg.offsite.s3_prefix = pfx.trim().trim_matches('/').to_string();
        }
        if let Some(ps) = off["s3_path_style"].as_bool() {
            cfg.offsite.s3_path_style = ps;
        }
    }
    save_backup_config(data_dir, &cfg)?;
    Ok(cfg)
}

pub fn load_backup_state(data_dir: &Path) -> BackupRuntimeState {
    let mut st = BackupRuntimeState::default();
    let path = data_dir.join(BACKUP_STATE_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return st;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return st;
    };
    st.last_run_ms = v["last_run_ms"].as_u64();
    st.last_backup_name = non_empty_str(v["last_backup_name"].as_str());
    st.last_backup_sha256 = non_empty_str(v["last_backup_sha256"].as_str());
    st.last_file_count = v["last_file_count"].as_u64().map(|n| n as usize);
    st.last_zip_bytes = v["last_zip_bytes"].as_u64();
    st.last_offsite_status = non_empty_str(v["last_offsite_status"].as_str());
    st.last_error = non_empty_str(v["last_error"].as_str());
    st
}

pub fn save_backup_state(data_dir: &Path, st: &BackupRuntimeState) {
    if let Ok(body) = serde_json::to_vec_pretty(&st.to_json()) {
        let _ = atomic_write_bytes(&data_dir.join(BACKUP_STATE_FILE), &body);
    }
}

pub fn is_scheduled_backup_due(
    cfg: &BackupConfig,
    st: &BackupRuntimeState,
    now_ms: u64,
) -> bool {
    if !cfg.enabled {
        return false;
    }
    let interval_ms = std::env::var("ERGON_BACKUP_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&s| s >= 1)
        .map(|s| s.saturating_mul(1_000))
        .unwrap_or_else(|| cfg.interval_hours.max(1).saturating_mul(3_600_000));
    match st.last_run_ms {
        None => true,
        Some(last) => now_ms.saturating_sub(last) >= interval_ms,
    }
}

// ---------------------------------------------------------------------------
// Timestamps & Path Safety
// ---------------------------------------------------------------------------

fn format_utc_iso(ms: u64) -> String {
    let (y, m, d) = records::civil_from_millis(ms);
    let secs_in_day = (ms / 1_000) % 86_400;
    let hh = secs_in_day / 3_600;
    let mm = (secs_in_day % 3_600) / 60;
    let ss = secs_in_day % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn format_archive_stamp(ms: u64) -> String {
    let (y, m, d) = records::civil_from_millis(ms);
    let secs_in_day = (ms / 1_000) % 86_400;
    let hh = secs_in_day / 3_600;
    let mm = (secs_in_day % 3_600) / 60;
    let ss = secs_in_day % 60;
    let millis = ms % 1_000;
    format!("{y:04}{m:02}{d:02}-{hh:02}{mm:02}{ss:02}-{millis:03}Z")
}

fn dos_datetime_from_ms(ms: u64) -> (u16, u16) {
    let (y, m, d) = records::civil_from_millis(ms);
    let secs_in_day = (ms / 1_000) % 86_400;
    let hh = (secs_in_day / 3_600) as u16;
    let mm = ((secs_in_day % 3_600) / 60) as u16;
    let ss2 = ((secs_in_day % 60) / 2) as u16;
    let clamped_y = y.clamp(1980, 2107) as u16;
    let dos_time = (hh << 11) | (mm << 5) | ss2;
    let dos_date = ((clamped_y - 1980) << 9) | ((m as u16) << 5) | (d as u16);
    (dos_time, dos_date)
}

/// Strictly validates a relative path inside a backup archive or manifest.
/// Rejects empty paths, leading slashes, backslashes, Windows drive prefixes,
/// NUL/control characters, `.` and `..` segments, and trailing slashes.
pub fn validate_safe_rel_path(raw: &str) -> Result<(), String> {
    if raw.is_empty() {
        return Err("empty path in backup archive".to_string());
    }
    if raw.bytes().any(|b| b == 0 || b < 0x20) {
        return Err(format!("control or NUL byte in archive path '{raw}'"));
    }
    if raw.starts_with('/') || raw.starts_with('\\') || raw.ends_with('/') || raw.ends_with('\\') {
        return Err(format!("unsafe leading or trailing separator in archive path '{raw}'"));
    }
    if raw.contains('\\') {
        return Err(format!("backslash traversal sequence in archive path '{raw}'"));
    }
    let bytes = raw.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(format!("Windows drive prefix in archive path '{raw}'"));
    }
    for seg in raw.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(format!("path traversal segment in archive path '{raw}'"));
        }
    }
    for comp in Path::new(raw).components() {
        match comp {
            Component::Normal(_) => {}
            _ => return Err(format!("unsafe path component in archive path '{raw}'")),
        }
    }
    Ok(())
}

/// Strictly validates a single backup archive filename (used for API download & restore).
pub fn validate_backup_filename(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 160 {
        return Err("invalid backup filename".to_string());
    }
    if name.starts_with('.') || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("backup filename contains unsafe path characters".to_string());
    }
    if !name.ends_with(".zip") {
        return Err("backup filename must end with .zip".to_string());
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err("backup filename contains disallowed characters".to_string());
    }
    Ok(())
}

/// Resolves a backup filename strictly inside `backup_dir` (rejects traversal & symlinks).
pub fn resolve_confined_backup_path(backup_dir: &Path, name: &str) -> Result<PathBuf, String> {
    validate_backup_filename(name)?;
    let canonical_root = std::fs::canonicalize(backup_dir)
        .map_err(|_| "backup directory does not exist".to_string())?;
    let resolved = sandbox_path_resolving(&canonical_root, name, true)?;
    let meta = std::fs::symlink_metadata(&resolved)
        .map_err(|_| format!("backup archive '{name}' not found"))?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err(format!("backup archive '{name}' is not a regular file"));
    }
    if resolved.parent() != Some(canonical_root.as_path()) {
        return Err("backup archive escapes backup directory".to_string());
    }
    Ok(resolved)
}

// ---------------------------------------------------------------------------
// Hardened Backup ZIP Writer & Reader (DEFLATE + CRC-32 + Symlink Rejection)
// ---------------------------------------------------------------------------

struct BackupZipEntry {
    name: String,
    method: u16,
    crc32: u32,
    comp_size: usize,
    uncomp_size: usize,
    local_offset: usize,
    version_made_by: u16,
    external_attrs: u32,
}

impl BackupZipEntry {
    fn unix_mode(&self) -> u32 {
        (self.external_attrs >> 16) & 0o170000
    }

    fn is_symlink_or_special(&self) -> bool {
        let file_type = self.unix_mode();
        let unix_host = (self.version_made_by >> 8) == 3;
        if file_type == 0o120000 {
            return true;
        }
        if unix_host && file_type != 0 && file_type != 0o100000 && file_type != 0o040000 {
            return true;
        }
        false
    }
}

fn le16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *b.get(at)?,
        *b.get(at + 1)?,
        *b.get(at + 2)?,
        *b.get(at + 3)?,
    ]))
}

pub fn write_backup_zip(files: &[(String, Vec<u8>)], created_at_ms: u64) -> Result<Vec<u8>, String> {
    if files.len() > u16::MAX as usize {
        return Err("too many files for standard ZIP backup archive".to_string());
    }
    let (dos_time, dos_date) = dos_datetime_from_ms(created_at_ms);
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    let unix_regular_attrs: u32 = 0o100644u32 << 16;
    let version_made_by: u16 = (3u16 << 8) | 20u16; // Unix + ZIP 2.0

    for (name, data) in files {
        if out.len() > u32::MAX as usize - 1024 {
            return Err("backup archive exceeds 4 GiB standard ZIP limit".to_string());
        }
        if data.len() > u32::MAX as usize {
            return Err(format!("file '{name}' exceeds 4 GiB ZIP entry limit"));
        }
        let crc = crc32(data);
        let deflated = miniz_oxide::deflate::compress_to_vec(data, 6);
        let (method, body): (u16, &[u8]) = if deflated.len() < data.len() {
            (8, &deflated)
        } else {
            (0, data.as_slice())
        };
        let offset = out.len() as u32;
        let nb = name.as_bytes();
        if nb.len() > u16::MAX as usize {
            return Err(format!("entry name too long: '{name}'"));
        }

        // Local file header
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // UTF-8
        out.extend_from_slice(&method.to_le_bytes());
        out.extend_from_slice(&dos_time.to_le_bytes());
        out.extend_from_slice(&dos_date.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(nb);
        out.extend_from_slice(body);

        // Central directory header
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&version_made_by.to_le_bytes());
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
        central.extend_from_slice(&unix_regular_attrs.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(nb);
    }

    let cd_off = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]); // disk numbers
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_len.to_le_bytes());
    out.extend_from_slice(&cd_off.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(out)
}

struct BackupZipReader<'a> {
    data: &'a [u8],
    entries: Vec<BackupZipEntry>,
    inflated: std::cell::Cell<usize>,
}

impl<'a> BackupZipReader<'a> {
    fn open(data: &'a [u8]) -> Result<Self, String> {
        if data.len() < 22 {
            return Err("not a valid ZIP archive (file too short)".to_string());
        }
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
        let eocd = eocd.ok_or_else(|| "not a valid ZIP archive (missing central directory)".to_string())?;
        let count = le16(data, eocd + 10).ok_or("corrupt ZIP end-of-central-directory")? as usize;
        let cd_size = le32(data, eocd + 12).ok_or("corrupt ZIP central directory size")? as usize;
        let cd_off = le32(data, eocd + 16).ok_or("corrupt ZIP central directory offset")? as usize;
        if cd_off == 0xFFFF_FFFF {
            return Err("ZIP64 backup archives are not supported".to_string());
        }
        if count > MAX_BACKUP_ENTRIES {
            return Err(format!("ZIP archive exceeds maximum entry count ({MAX_BACKUP_ENTRIES})"));
        }
        let cd_end = cd_off
            .checked_add(cd_size)
            .ok_or("corrupt ZIP central directory bounds")?;
        if cd_end > eocd {
            return Err("corrupt ZIP central directory extends past EOCD".to_string());
        }

        let mut entries = Vec::with_capacity(count);
        let mut seen = HashSet::with_capacity(count);
        let mut p = cd_off;
        for _ in 0..count {
            if le32(data, p) != Some(0x0201_4b50) {
                return Err("corrupt ZIP central directory header".to_string());
            }
            let version_made_by = le16(data, p + 4).ok_or("corrupt ZIP entry")?;
            let method = le16(data, p + 10).ok_or("corrupt ZIP entry")?;
            let crc = le32(data, p + 16).ok_or("corrupt ZIP entry")?;
            let comp_size = le32(data, p + 20).ok_or("corrupt ZIP entry")? as usize;
            let uncomp_size = le32(data, p + 24).ok_or("corrupt ZIP entry")? as usize;
            let n = le16(data, p + 28).ok_or("corrupt ZIP entry")? as usize;
            let e = le16(data, p + 30).ok_or("corrupt ZIP entry")? as usize;
            let c = le16(data, p + 32).ok_or("corrupt ZIP entry")? as usize;
            let external_attrs = le32(data, p + 38).ok_or("corrupt ZIP entry")?;
            let local_offset = le32(data, p + 42).ok_or("corrupt ZIP entry")? as usize;
            let name_bytes = data.get(p + 46..p + 46 + n).ok_or("truncated ZIP entry name")?;
            let name = std::str::from_utf8(name_bytes)
                .map_err(|_| "ZIP archive contains non-UTF-8 entry name".to_string())?
                .to_string();
            if !seen.insert(name.clone()) {
                return Err(format!("ZIP archive contains duplicate entry '{name}'"));
            }
            entries.push(BackupZipEntry {
                name,
                method,
                crc32: crc,
                comp_size,
                uncomp_size,
                local_offset,
                version_made_by,
                external_attrs,
            });
            p = p
                .checked_add(46 + n + e + c)
                .ok_or("corrupt ZIP directory pointer")?;
        }
        Ok(Self {
            data,
            entries,
            inflated: std::cell::Cell::new(0),
        })
    }

    fn read_entry(&self, entry: &BackupZipEntry) -> Result<Vec<u8>, String> {
        if entry.uncomp_size > MAX_BACKUP_ENTRY_BYTES {
            return Err(format!("archive member '{}' exceeds size limit", entry.name));
        }
        let lh = entry.local_offset;
        if le32(self.data, lh) != Some(0x0403_4b50) {
            return Err(format!("corrupt local header for '{}'", entry.name));
        }
        let lh_name_len = le16(self.data, lh + 26).ok_or("corrupt local header")? as usize;
        let lh_extra_len = le16(self.data, lh + 28).ok_or("corrupt local header")? as usize;
        let lh_name_bytes = self
            .data
            .get(lh + 30..lh + 30 + lh_name_len)
            .ok_or_else(|| format!("truncated local name for '{}'", entry.name))?;
        if lh_name_bytes != entry.name.as_bytes() {
            return Err(format!(
                "local header filename mismatch for '{}'",
                entry.name
            ));
        }
        let start = lh + 30 + lh_name_len + lh_extra_len;
        let end = start
            .checked_add(entry.comp_size)
            .ok_or_else(|| format!("truncated payload for '{}'", entry.name))?;
        let raw = self
            .data
            .get(start..end)
            .ok_or_else(|| format!("truncated payload for '{}'", entry.name))?;

        let budget = MAX_BACKUP_TOTAL_BYTES.saturating_sub(self.inflated.get());
        if budget == 0 {
            return Err("backup archive exceeds total expansion cap".to_string());
        }
        let limit = MAX_BACKUP_ENTRY_BYTES.min(budget);
        let out = match entry.method {
            0 => {
                if raw.len() > limit {
                    return Err(format!("archive member '{}' exceeds expansion cap", entry.name));
                }
                raw.to_vec()
            }
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(raw, limit)
                .map_err(|_| format!("archive member '{}' is corrupted or exceeds size limit", entry.name))?,
            m => {
                return Err(format!(
                    "archive member '{}' uses unsupported compression method {m}",
                    entry.name
                ));
            }
        };
        if out.len() != entry.uncomp_size {
            return Err(format!(
                "archive member '{}' size mismatch (declared {}, actual {})",
                entry.name,
                entry.uncomp_size,
                out.len()
            ));
        }
        let actual_crc = crc32(&out);
        if actual_crc != entry.crc32 {
            return Err(format!(
                "archive member '{}' CRC-32 mismatch (corrupted data)",
                entry.name
            ));
        }
        self.inflated.set(self.inflated.get() + out.len());
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Manifest Format & Canonical Digest
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct BackupManifestFile {
    pub path: String,
    pub archive_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct BackupManifest {
    pub schema: String,
    pub ergon_version: String,
    pub created_at_utc: String,
    pub created_at_ms: u64,
    pub file_count: usize,
    pub total_bytes: u64,
    pub files: Vec<BackupManifestFile>,
    pub manifest_sha256: String,
}

impl BackupManifest {
    pub fn compute_digest(&self) -> String {
        use std::fmt::Write as _;
        let mut hasher = Sha256::new();
        hasher.update(self.schema.as_bytes());
        hasher.update(b"\nversion:");
        hasher.update(self.ergon_version.as_bytes());
        hasher.update(b"\ncreated_at_utc:");
        hasher.update(self.created_at_utc.as_bytes());
        hasher.update(b"\ncreated_at_ms:");
        hasher.update(self.created_at_ms.to_string().as_bytes());
        hasher.update(b"\nfile_count:");
        hasher.update(self.file_count.to_string().as_bytes());
        hasher.update(b"\ntotal_bytes:");
        hasher.update(self.total_bytes.to_string().as_bytes());
        hasher.update(b"\n");
        for f in &self.files {
            hasher.update(f.path.as_bytes());
            hasher.update(b"|");
            hasher.update(f.archive_path.as_bytes());
            hasher.update(b"|");
            hasher.update(f.size_bytes.to_string().as_bytes());
            hasher.update(b"|");
            hasher.update(f.sha256.as_bytes());
            hasher.update(b"\n");
        }
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for b in digest {
            let _ = write!(&mut hex, "{b:02x}");
        }
        hex
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schema": self.schema,
            "ergon_version": self.ergon_version,
            "created_at_utc": self.created_at_utc,
            "created_at_ms": self.created_at_ms,
            "file_count": self.file_count,
            "total_bytes": self.total_bytes,
            "files": self.files.iter().map(|f| json!({
                "path": f.path,
                "archive_path": f.archive_path,
                "size_bytes": f.size_bytes,
                "sha256": f.sha256,
            })).collect::<Vec<_>>(),
            "manifest_sha256": self.manifest_sha256,
        })
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let schema = v["schema"]
            .as_str()
            .ok_or("backup-manifest.json missing schema")?
            .to_string();
        let ergon_version = v["ergon_version"]
            .as_str()
            .ok_or("backup-manifest.json missing ergon_version")?
            .to_string();
        let created_at_utc = v["created_at_utc"]
            .as_str()
            .ok_or("backup-manifest.json missing created_at_utc")?
            .to_string();
        let created_at_ms = v["created_at_ms"]
            .as_u64()
            .ok_or("backup-manifest.json missing created_at_ms")?;
        let file_count = v["file_count"]
            .as_u64()
            .ok_or("backup-manifest.json missing file_count")? as usize;
        let total_bytes = v["total_bytes"]
            .as_u64()
            .ok_or("backup-manifest.json missing total_bytes")?;
        let manifest_sha256 = v["manifest_sha256"]
            .as_str()
            .ok_or("backup-manifest.json missing manifest_sha256")?
            .trim()
            .to_ascii_lowercase();
        let arr = v["files"]
            .as_array()
            .ok_or("backup-manifest.json missing files array")?;
        let mut files = Vec::with_capacity(arr.len());
        for item in arr {
            let path = item["path"]
                .as_str()
                .ok_or("manifest entry missing path")?
                .to_string();
            let archive_path = item["archive_path"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("data/{path}"));
            let size_bytes = item["size_bytes"]
                .as_u64()
                .ok_or("manifest entry missing size_bytes")?;
            let sha256 = item["sha256"]
                .as_str()
                .ok_or("manifest entry missing sha256")?
                .trim()
                .to_ascii_lowercase();
            files.push(BackupManifestFile {
                path,
                archive_path,
                size_bytes,
                sha256,
            });
        }
        Ok(Self {
            schema,
            ergon_version,
            created_at_utc,
            created_at_ms,
            file_count,
            total_bytes,
            files,
            manifest_sha256,
        })
    }
}

// ---------------------------------------------------------------------------
// Snapshot Collection & Archive Creation
// ---------------------------------------------------------------------------

fn is_transient_temp_or_lock(file_name: &str) -> bool {
    let low = file_name.to_ascii_lowercase();
    low.ends_with(".tmp")
        || low.ends_with(".lock")
        || low.ends_with(".part")
        || low.contains(".ergon-") && low.ends_with(".tmp")
        || low.starts_with(".ergon-restore-stage-")
        || low.starts_with(".ergon-pre-restore-")
        || low == BACKUP_STATE_FILE
}

fn canonical_or_normalized(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

fn should_exclude_path(path: &Path, excluded_roots: &[PathBuf]) -> bool {
    let norm = canonical_or_normalized(path);
    for ex in excluded_roots {
        if norm == *ex || norm.starts_with(ex) {
            return true;
        }
    }
    false
}

fn collect_snapshot_files(
    data_dir: &Path,
    excluded_roots: &[PathBuf],
) -> Result<Vec<(String, Vec<u8>)>, String> {
    fn walk(
        root: &Path,
        dir: &Path,
        excluded_roots: &[PathBuf],
        depth: usize,
        out: &mut Vec<(String, PathBuf)>,
    ) -> Result<(), String> {
        if depth > 24 {
            return Ok(());
        }
        let rd = std::fs::read_dir(dir)
            .map_err(|e| format!("read directory '{}': {e}", dir.display()))?;
        for entry in rd {
            let Ok(entry) = entry else { continue };
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_transient_temp_or_lock(&name) {
                continue;
            }
            let Ok(ft) = entry.file_type() else { continue };
            // Never follow or archive symlinks (prevents data_dir escapes).
            if ft.is_symlink() {
                continue;
            }
            let p = entry.path();
            if should_exclude_path(&p, excluded_roots) {
                continue;
            }
            if ft.is_dir() {
                if depth == 0 && name == "backups" {
                    continue;
                }
                walk(root, &p, excluded_roots, depth + 1, out)?;
            } else if ft.is_file() {
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| format!("strip prefix: {e}"))?
                    .to_string_lossy()
                    .replace('\\', "/");
                validate_safe_rel_path(&rel)?;
                out.push((rel, p));
            }
        }
        Ok(())
    }

    let mut discovered = Vec::new();
    if data_dir.exists() {
        walk(data_dir, data_dir, excluded_roots, 0, &mut discovered)?;
    }
    discovered.sort_by(|a, b| a.0.cmp(&b.0));

    let mut files = Vec::with_capacity(discovered.len());
    let mut total_bytes = 0usize;
    for (rel, full_path) in discovered {
        let bytes = match std::fs::read(&full_path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("read '{}': {e}", full_path.display())),
        };
        if bytes.len() > MAX_BACKUP_ENTRY_BYTES {
            return Err(format!("file '{rel}' exceeds backup single-file limit"));
        }
        total_bytes = total_bytes.saturating_add(bytes.len());
        if total_bytes > MAX_BACKUP_TOTAL_BYTES {
            return Err("instance data exceeds backup size limit".to_string());
        }
        files.push((rel, bytes));
    }
    Ok(files)
}

#[derive(Clone, Debug)]
pub struct BuiltBackupArchive {
    pub zip_bytes: Vec<u8>,
    pub sha256: String,
    pub manifest: BackupManifest,
}

/// Build a verified in-memory ZIP backup snapshot of `data_dir`.
pub fn build_backup_archive(
    data_dir: &Path,
    extra_excludes: &[PathBuf],
) -> Result<BuiltBackupArchive, String> {
    let now_ms = records::now_millis();
    let created_at_utc = format_utc_iso(now_ms);

    let mut excluded: Vec<PathBuf> = vec![canonical_or_normalized(&data_dir.join("backups"))];
    for p in extra_excludes {
        excluded.push(canonical_or_normalized(p));
    }

    let collected = collect_snapshot_files(data_dir, &excluded)?;
    let mut manifest_files = Vec::with_capacity(collected.len());
    let mut zip_entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(collected.len() + 1);
    let mut total_bytes: u64 = 0;

    for (rel, bytes) in collected {
        let archive_path = format!("data/{rel}");
        let sz = bytes.len() as u64;
        let digest = sha256_hex(&bytes);
        total_bytes = total_bytes.saturating_add(sz);
        manifest_files.push(BackupManifestFile {
            path: rel,
            archive_path: archive_path.clone(),
            size_bytes: sz,
            sha256: digest,
        });
        zip_entries.push((archive_path, bytes));
    }

    let mut manifest = BackupManifest {
        schema: BACKUP_MANIFEST_SCHEMA.to_string(),
        ergon_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc,
        created_at_ms: now_ms,
        file_count: manifest_files.len(),
        total_bytes,
        files: manifest_files,
        manifest_sha256: String::new(),
    };
    manifest.manifest_sha256 = manifest.compute_digest();

    let manifest_bytes = serde_json::to_vec_pretty(&manifest.to_json())
        .map_err(|e| format!("serialize backup manifest: {e}"))?;
    zip_entries.insert(0, (BACKUP_MANIFEST_NAME.to_string(), manifest_bytes));

    let zip_bytes = write_backup_zip(&zip_entries, now_ms)?;
    let sha256 = sha256_hex(&zip_bytes);

    Ok(BuiltBackupArchive {
        zip_bytes,
        sha256,
        manifest,
    })
}

// ---------------------------------------------------------------------------
// Backup Archive Verification (ZIP-slip, Symlink, CRC-32, Manifest & SHA-256)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct BackupVerification {
    pub valid: bool,
    pub archive_sha256: String,
    pub manifest_sha256: String,
    pub expected_manifest_sha256: String,
    pub ergon_version: String,
    pub created_at_utc: String,
    pub file_count: usize,
    pub verified_files: usize,
    pub total_bytes: u64,
    pub errors: Vec<String>,
    pub manifest: Option<BackupManifest>,
}

impl BackupVerification {
    pub fn to_evidence_verification(&self, filename: Option<&str>) -> EvidenceVerification {
        let title = filename
            .and_then(|p| Path::new(p).file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("ergon-backup.zip")
            .to_string();
        let summary = if self.valid {
            format!(
                "Backup archive passed offline SHA-256 verification ({} files, {} bytes, Ergon {}).",
                self.verified_files, self.total_bytes, self.ergon_version
            )
        } else {
            format!(
                "Backup archive verification failed with {} issue(s).",
                self.errors.len()
            )
        };
        let manifest_ok = !self.expected_manifest_sha256.is_empty()
            && self.manifest_sha256 == self.expected_manifest_sha256;
        let items = vec![
            json!({
                "label": "Backup manifest digest",
                "status": if manifest_ok { "verified" } else { "tampered" },
                "detail": format!("{} files ({} bytes)", self.file_count, self.total_bytes),
                "sha256": self.manifest_sha256,
            }),
            json!({
                "label": "Archive member SHA-256 & path safety",
                "status": if self.valid { "verified" } else { "tampered" },
                "detail": format!("{}/{} files verified", self.verified_files, self.file_count),
                "sha256": self.archive_sha256,
            }),
        ];
        EvidenceVerification {
            valid: self.valid,
            kind: "backup_archive",
            title,
            receipt_chain_valid: None,
            bundle_hash_valid: Some(self.valid),
            manifest_valid: Some(manifest_ok),
            citations_valid: None,
            workspace_valid: None,
            receipt_count: 0,
            deliverable_count: self.verified_files,
            citation_count: 0,
            stale_citations: 0,
            broken_citations: 0,
            expected_sha256: self.expected_manifest_sha256.clone(),
            actual_sha256: self.archive_sha256.clone(),
            summary,
            errors: self.errors.clone(),
            items,
        }
    }
}

/// Independently verify a backup ZIP archive in memory:
/// - ZIP structure, local/central directory consistency, no duplicate entries
/// - Rejects any path traversal (zip-slip), absolute paths, backslashes, or symlinks
/// - Validates `backup-manifest.json` schema, counts, and canonical `manifest_sha256`
/// - Verifies CRC-32, byte size, and SHA-256 of EVERY member file
/// - Rejects any unmanifested extra entries in the archive
pub fn verify_backup_archive_bytes(bytes: &[u8]) -> BackupVerification {
    let archive_sha256 = sha256_hex(bytes);
    let mut errors = Vec::new();

    let reader = match BackupZipReader::open(bytes) {
        Ok(r) => r,
        Err(e) => {
            errors.push(e);
            return BackupVerification {
                valid: false,
                archive_sha256,
                manifest_sha256: String::new(),
                expected_manifest_sha256: String::new(),
                ergon_version: String::new(),
                created_at_utc: String::new(),
                file_count: 0,
                verified_files: 0,
                total_bytes: 0,
                errors,
                manifest: None,
            };
        }
    };

    // Pass 1: Security scan EVERY ZIP entry BEFORE looking at the manifest.
    let mut entry_map: BTreeMap<String, usize> = BTreeMap::new();
    for (idx, entry) in reader.entries.iter().enumerate() {
        if let Err(e) = validate_safe_rel_path(&entry.name) {
            errors.push(format!("unsafe ZIP entry '{}': {e}", entry.name));
        }
        if entry.is_symlink_or_special() {
            errors.push(format!(
                "ZIP entry '{}' is a symlink or special file (forbidden)",
                entry.name
            ));
        }
        entry_map.insert(entry.name.clone(), idx);
    }

    let Some(&manifest_idx) = entry_map.get(BACKUP_MANIFEST_NAME) else {
        errors.push(format!("ZIP archive is missing {BACKUP_MANIFEST_NAME}"));
        return BackupVerification {
            valid: false,
            archive_sha256,
            manifest_sha256: String::new(),
            expected_manifest_sha256: String::new(),
            ergon_version: String::new(),
            created_at_utc: String::new(),
            file_count: 0,
            verified_files: 0,
            total_bytes: 0,
            errors,
            manifest: None,
        };
    };

    let manifest_bytes = match reader.read_entry(&reader.entries[manifest_idx]) {
        Ok(b) => b,
        Err(e) => {
            errors.push(format!("cannot read {BACKUP_MANIFEST_NAME}: {e}"));
            return BackupVerification {
                valid: false,
                archive_sha256,
                manifest_sha256: String::new(),
                expected_manifest_sha256: String::new(),
                ergon_version: String::new(),
                created_at_utc: String::new(),
                file_count: 0,
                verified_files: 0,
                total_bytes: 0,
                errors,
                manifest: None,
            };
        }
    };

    let manifest_val: Value = match serde_json::from_slice(&manifest_bytes) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("invalid JSON in {BACKUP_MANIFEST_NAME}: {e}"));
            return BackupVerification {
                valid: false,
                archive_sha256,
                manifest_sha256: String::new(),
                expected_manifest_sha256: String::new(),
                ergon_version: String::new(),
                created_at_utc: String::new(),
                file_count: 0,
                verified_files: 0,
                total_bytes: 0,
                errors,
                manifest: None,
            };
        }
    };

    let manifest = match BackupManifest::from_json(&manifest_val) {
        Ok(m) => m,
        Err(e) => {
            errors.push(e);
            return BackupVerification {
                valid: false,
                archive_sha256,
                manifest_sha256: String::new(),
                expected_manifest_sha256: String::new(),
                ergon_version: String::new(),
                created_at_utc: String::new(),
                file_count: 0,
                verified_files: 0,
                total_bytes: 0,
                errors,
                manifest: None,
            };
        }
    };

    if manifest.schema != BACKUP_MANIFEST_SCHEMA {
        errors.push(format!(
            "unexpected backup manifest schema '{}'",
            manifest.schema
        ));
    }
    if manifest.file_count != manifest.files.len() {
        errors.push(format!(
            "manifest file_count ({}) does not match files length ({})",
            manifest.file_count,
            manifest.files.len()
        ));
    }
    let expected_manifest_sha256 = manifest.compute_digest();
    if manifest.manifest_sha256 != expected_manifest_sha256 {
        errors.push("manifest_sha256 mismatch — backup manifest was tampered with".to_string());
    }

    let mut expected_archive_paths: HashSet<String> = HashSet::with_capacity(manifest.files.len() + 1);
    expected_archive_paths.insert(BACKUP_MANIFEST_NAME.to_string());
    let mut seen_rel_paths: HashSet<String> = HashSet::with_capacity(manifest.files.len());
    let mut verified_files = 0usize;
    let mut summed_bytes = 0u64;

    for mf in &manifest.files {
        if let Err(e) = validate_safe_rel_path(&mf.path) {
            errors.push(format!("unsafe manifest path '{}': {e}", mf.path));
            continue;
        }
        if let Err(e) = validate_safe_rel_path(&mf.archive_path) {
            errors.push(format!(
                "unsafe manifest archive_path '{}': {e}",
                mf.archive_path
            ));
            continue;
        }
        if !seen_rel_paths.insert(mf.path.clone()) {
            errors.push(format!("duplicate manifest path '{}'", mf.path));
            continue;
        }
        expected_archive_paths.insert(mf.archive_path.clone());
        summed_bytes = summed_bytes.saturating_add(mf.size_bytes);

        let Some(&idx) = entry_map.get(&mf.archive_path) else {
            errors.push(format!(
                "manifest file '{}' (archive '{}') is missing from ZIP",
                mf.path, mf.archive_path
            ));
            continue;
        };
        match reader.read_entry(&reader.entries[idx]) {
            Ok(data) => {
                if data.len() as u64 != mf.size_bytes {
                    errors.push(format!(
                        "file '{}' byte size mismatch (expected {}, got {})",
                        mf.path,
                        mf.size_bytes,
                        data.len()
                    ));
                    continue;
                }
                let actual_sha = sha256_hex(&data);
                if actual_sha != mf.sha256 {
                    errors.push(format!(
                        "file '{}' SHA-256 mismatch (tampered or corrupted)",
                        mf.path
                    ));
                    continue;
                }
                verified_files += 1;
            }
            Err(e) => {
                errors.push(format!("failed to read '{}': {e}", mf.path));
            }
        }
    }

    if summed_bytes != manifest.total_bytes {
        errors.push(format!(
            "manifest total_bytes ({}) does not match summed file sizes ({summed_bytes})",
            manifest.total_bytes
        ));
    }

    for name in entry_map.keys() {
        if !expected_archive_paths.contains(name) {
            errors.push(format!(
                "ZIP archive contains unmanifested entry '{name}'"
            ));
        }
    }

    let valid = errors.is_empty() && verified_files == manifest.files.len();
    BackupVerification {
        valid,
        archive_sha256,
        manifest_sha256: manifest.manifest_sha256.clone(),
        expected_manifest_sha256,
        ergon_version: manifest.ergon_version.clone(),
        created_at_utc: manifest.created_at_utc.clone(),
        file_count: manifest.files.len(),
        verified_files,
        total_bytes: manifest.total_bytes,
        errors,
        manifest: Some(manifest),
    }
}

/// Cross-check a restored (or live) `data_dir` on disk against a verified backup manifest.
pub fn verify_data_dir_against_manifest(
    data_dir: &Path,
    manifest: &BackupManifest,
) -> Result<usize, Vec<String>> {
    let mut errs = Vec::new();
    let canonical_root = match std::fs::canonicalize(data_dir) {
        Ok(c) => c,
        Err(e) => return Err(vec![format!("cannot access data_dir '{}': {e}", data_dir.display())]),
    };
    let mut matched = 0usize;
    for mf in &manifest.files {
        if let Err(e) = validate_safe_rel_path(&mf.path) {
            errs.push(e);
            continue;
        }
        let resolved = match sandbox_path_resolving(&canonical_root, &mf.path, true) {
            Ok(p) => p,
            Err(e) => {
                errs.push(format!("file '{}' path check failed: {e}", mf.path));
                continue;
            }
        };
        let meta = match std::fs::symlink_metadata(&resolved) {
            Ok(m) => m,
            Err(_) => {
                errs.push(format!("restored file '{}' is missing on disk", mf.path));
                continue;
            }
        };
        if meta.file_type().is_symlink() || !meta.is_file() {
            errs.push(format!("restored path '{}' is not a regular file", mf.path));
            continue;
        }
        match std::fs::read(&resolved) {
            Ok(bytes) => {
                if bytes.len() as u64 != mf.size_bytes {
                    errs.push(format!(
                        "restored file '{}' size mismatch (expected {}, got {})",
                        mf.path,
                        mf.size_bytes,
                        bytes.len()
                    ));
                    continue;
                }
                let actual = sha256_hex(&bytes);
                if actual != mf.sha256 {
                    errs.push(format!(
                        "restored file '{}' SHA-256 mismatch on disk",
                        mf.path
                    ));
                    continue;
                }
                matched += 1;
            }
            Err(e) => errs.push(format!("cannot read restored file '{}': {e}", mf.path)),
        }
    }
    if errs.is_empty() {
        Ok(matched)
    } else {
        Err(errs)
    }
}

pub fn verify_backup_file(
    archive_path: &Path,
    check_data_dir: Option<&Path>,
) -> EvidenceVerification {
    let label = archive_path.to_string_lossy().into_owned();
    let bytes = match std::fs::read(archive_path) {
        Ok(b) => b,
        Err(e) => {
            return EvidenceVerification {
                valid: false,
                kind: "backup_archive",
                title: archive_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("backup.zip")
                    .to_string(),
                receipt_chain_valid: None,
                bundle_hash_valid: Some(false),
                manifest_valid: None,
                citations_valid: None,
                workspace_valid: None,
                receipt_count: 0,
                deliverable_count: 0,
                citation_count: 0,
                stale_citations: 0,
                broken_citations: 0,
                expected_sha256: String::new(),
                actual_sha256: String::new(),
                summary: format!("Cannot read backup archive: {e}"),
                errors: vec![format!("cannot read {label}: {e}")],
                items: Vec::new(),
            };
        }
    };
    let bv = verify_backup_archive_bytes(&bytes);
    let mut ev = bv.to_evidence_verification(Some(&label));
    if let (Some(dir), Some(manifest)) = (check_data_dir, bv.manifest.as_ref()) {
        match verify_data_dir_against_manifest(dir, manifest) {
            Ok(n) => {
                ev.workspace_valid = Some(true);
                ev.items.push(json!({
                    "label": "Data directory files on disk",
                    "status": "verified",
                    "detail": format!("{n} files matched manifest SHA-256"),
                }));
            }
            Err(disk_errs) => {
                ev.valid = false;
                ev.workspace_valid = Some(false);
                ev.items.push(json!({
                    "label": "Data directory files on disk",
                    "status": "tampered",
                    "detail": format!("{} mismatch(es)", disk_errs.len()),
                }));
                ev.errors.extend(disk_errs);
                ev.summary = format!(
                    "Backup verification against data directory failed with {} issue(s).",
                    ev.errors.len()
                );
            }
        }
    }
    ev
}

// ---------------------------------------------------------------------------
// Retention Pruning & Listing
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct BackupListEntry {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_ms: u64,
    pub sha256: Option<String>,
    pub created_at_utc: Option<String>,
    pub file_count: Option<usize>,
    pub kind: &'static str,
}

impl BackupListEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "size_bytes": self.size_bytes,
            "modified_ms": self.modified_ms,
            "sha256": self.sha256,
            "created_at_utc": self.created_at_utc,
            "file_count": self.file_count,
            "kind": self.kind,
        })
    }
}

fn read_sidecar_sha256(zip_path: &Path) -> Option<String> {
    let sidecar = PathBuf::from(format!("{}.sha256", zip_path.display()));
    let text = std::fs::read_to_string(sidecar).ok()?;
    let token = text.split_whitespace().next()?.trim().to_ascii_lowercase();
    if token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(token)
    } else {
        None
    }
}

fn peek_manifest_meta(zip_path: &Path) -> (Option<String>, Option<usize>) {
    let Ok(bytes) = std::fs::read(zip_path) else {
        return (None, None);
    };
    let Ok(reader) = BackupZipReader::open(&bytes) else {
        return (None, None);
    };
    let Some(entry) = reader.entries.iter().find(|e| e.name == BACKUP_MANIFEST_NAME) else {
        return (None, None);
    };
    let Ok(raw) = reader.read_entry(entry) else {
        return (None, None);
    };
    let Ok(v) = serde_json::from_slice::<Value>(&raw) else {
        return (None, None);
    };
    let created = v["created_at_utc"].as_str().map(str::to_string);
    let count = v["file_count"].as_u64().map(|n| n as usize);
    (created, count)
}

/// List verified Ergon backup archives inside `backup_dir` (newest first).
pub fn list_backups(backup_dir: &Path) -> Vec<BackupListEntry> {
    let Ok(rd) = std::fs::read_dir(backup_dir) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".zip") {
            continue;
        }
        let kind = if name.starts_with("ergon-backup-") {
            "backup"
        } else if name.starts_with("pre-restore-") {
            "pre_restore"
        } else {
            continue;
        };
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let path = entry.path();
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let sha256 = read_sidecar_sha256(&path).or_else(|| {
            std::fs::read(&path).ok().map(|b| sha256_hex(&b))
        });
        let (created_at_utc, file_count) = peek_manifest_meta(&path);
        items.push(BackupListEntry {
            name,
            path,
            size_bytes: meta.len(),
            modified_ms,
            sha256,
            created_at_utc,
            file_count,
            kind,
        });
    }
    items.sort_by(|a, b| b.name.cmp(&a.name).then_with(|| b.modified_ms.cmp(&a.modified_ms)));
    items
}

/// Prune oldest `ergon-backup-*.zip` files (and their `.sha256` sidecars) so at most
/// `keep` regular backups remain in `dir`. Returns names of pruned archives.
pub fn prune_backups_in_dir(dir: &Path, keep: usize) -> Vec<String> {
    if keep == 0 {
        return Vec::new();
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut zips: Vec<(String, PathBuf)> = Vec::new();
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("ergon-backup-")
            && name.ends_with(".zip")
            && entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
        {
            zips.push((name, entry.path()));
        }
    }
    // Filenames include UTC timestamp YYYYMMDD-HHMMSS-mmmZ, so descending sort puts newest first.
    zips.sort_by(|a, b| b.0.cmp(&a.0));
    let mut pruned = Vec::new();
    if zips.len() > keep {
        for (name, path) in zips.into_iter().skip(keep) {
            if std::fs::remove_file(&path).is_ok() {
                let sidecar = PathBuf::from(format!("{}.sha256", path.display()));
                let _ = std::fs::remove_file(sidecar);
                pruned.push(name);
            }
        }
    }
    pruned
}

// ---------------------------------------------------------------------------
// AWS SigV4 S3 Upload (Zero Extra Dependencies)
// ---------------------------------------------------------------------------

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn aws_uri_encode(input: &str, encode_slash: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        let unreserved = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~');
        if unreserved || (!encode_slash && b == b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct SignedS3PutRequest {
    pub url: String,
    pub host: String,
    pub canonical_uri: String,
    pub amz_date: String,
    pub payload_sha256: String,
    pub authorization: String,
    pub session_token: Option<String>,
}

fn parse_s3_endpoint(
    endpoint: Option<&str>,
    bucket: &str,
    region: &str,
    object_key: &str,
    path_style: bool,
) -> Result<(String, String, String), String> {
    let default_ep = format!("https://s3.{region}.amazonaws.com");
    let raw_ep = endpoint
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&default_ep);
    let (scheme, rest) = if let Some(r) = raw_ep.strip_prefix("https://") {
        ("https", r)
    } else if let Some(r) = raw_ep.strip_prefix("http://") {
        ("http", r)
    } else {
        return Err("S3 endpoint must start with http:// or https://".to_string());
    };
    let (authority, base_path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], rest[idx..].trim_end_matches('/')),
        None => (rest, ""),
    };
    if authority.is_empty() {
        return Err("S3 endpoint is missing host".to_string());
    }
    let clean_key = object_key.trim_start_matches('/');
    let host_only = authority.split(':').next().unwrap_or(authority);
    let is_ip_or_local = host_only == "localhost"
        || host_only.parse::<std::net::IpAddr>().is_ok()
        || host_only.starts_with('[');
    let use_path_style = path_style || is_ip_or_local || !base_path.is_empty();
    let (host_header, canonical_uri) = if use_path_style {
        let mut uri = String::new();
        if !base_path.is_empty() {
            uri.push_str(&aws_uri_encode(base_path, false));
        }
        uri.push('/');
        uri.push_str(&aws_uri_encode(bucket, true));
        uri.push('/');
        uri.push_str(&aws_uri_encode(clean_key, false));
        (authority.to_ascii_lowercase(), uri)
    } else {
        let host = format!("{}.{}", bucket.to_ascii_lowercase(), authority.to_ascii_lowercase());
        let uri = format!("/{}", aws_uri_encode(clean_key, false));
        (host, uri)
    };
    let url = format!("{scheme}://{host_header}{canonical_uri}");
    Ok((url, host_header, canonical_uri))
}

/// Build an AWS Signature Version 4 signed PUT request for an S3-compatible endpoint.
#[allow(clippy::too_many_arguments)] // each arg is a distinct SigV4 input
pub fn build_sigv4_s3_put(
    endpoint: Option<&str>,
    bucket: &str,
    region: &str,
    object_key: &str,
    path_style: bool,
    body: &[u8],
    creds: &S3Credentials,
    now_ms: u64,
) -> Result<SignedS3PutRequest, String> {
    let (url, host, canonical_uri) =
        parse_s3_endpoint(endpoint, bucket, region, object_key, path_style)?;
    let (y, m, d) = records::civil_from_millis(now_ms);
    let secs_in_day = (now_ms / 1_000) % 86_400;
    let hh = secs_in_day / 3_600;
    let mm = (secs_in_day % 3_600) / 60;
    let ss = secs_in_day % 60;
    let date_stamp = format!("{y:04}{m:02}{d:02}");
    let amz_date = format!("{date_stamp}T{hh:02}{mm:02}{ss:02}Z");
    let payload_sha256 = sha256_hex(body);

    let mut headers: BTreeMap<&str, String> = BTreeMap::new();
    headers.insert("host", host.clone());
    headers.insert("x-amz-content-sha256", payload_sha256.clone());
    headers.insert("x-amz-date", amz_date.clone());
    if let Some(ref tok) = creds.session_token {
        headers.insert("x-amz-security-token", tok.trim().to_string());
    }

    let mut canonical_headers = String::new();
    let mut signed_header_names = Vec::with_capacity(headers.len());
    for (k, v) in &headers {
        canonical_headers.push_str(k);
        canonical_headers.push(':');
        canonical_headers.push_str(v.trim());
        canonical_headers.push('\n');
        signed_header_names.push(*k);
    }
    let signed_headers = signed_header_names.join(";");

    let canonical_request = format!(
        "PUT\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_sha256}"
    );
    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());
    let credential_scope = format!("{date_stamp}/{region}/s3/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{canonical_request_hash}"
    );

    let k_date = hmac_sha256(
        format!("AWS4{}", creds.secret_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hex_lower(&hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        creds.access_key
    );

    Ok(SignedS3PutRequest {
        url,
        host,
        canonical_uri,
        amz_date,
        payload_sha256,
        authorization,
        session_token: creds.session_token.clone(),
    })
}

fn s3_put_object(
    offsite: &BackupOffsiteConfig,
    object_key: &str,
    content_type: &str,
    body: &[u8],
    creds: &S3Credentials,
) -> Result<String, String> {
    let bucket = offsite
        .s3_bucket
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or("S3 offsite bucket is not configured")?;
    let signed = build_sigv4_s3_put(
        offsite.s3_endpoint.as_deref(),
        bucket,
        &offsite.s3_region,
        object_key,
        offsite.s3_path_style,
        body,
        creds,
        records::now_millis(),
    )?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(120))
        .build();
    let mut req = agent
        .put(&signed.url)
        .set("Host", &signed.host)
        .set("Content-Type", content_type)
        .set("x-amz-date", &signed.amz_date)
        .set("x-amz-content-sha256", &signed.payload_sha256)
        .set("Authorization", &signed.authorization);
    if let Some(ref tok) = signed.session_token {
        req = req.set("x-amz-security-token", tok);
    }

    match req.send_bytes(body) {
        Ok(resp) => {
            let status = resp.status();
            if (200..300).contains(&status) {
                Ok(signed.url)
            } else {
                Err(format!("S3 PUT returned HTTP {status}"))
            }
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("S3 PUT failed with HTTP {code}")),
        Err(ureq::Error::Transport(t)) => Err(format!("S3 PUT transport error: {}", t.kind())),
    }
}

/// Parse a CLI `--offsite` string into a `BackupOffsiteConfig`.
/// Accepts:
/// - `s3://<bucket>[/<prefix>]`
/// - `http(s)://<endpoint>/<bucket>[/<prefix>]`
/// - Local folder path
pub fn parse_offsite_target(raw: &str) -> Result<BackupOffsiteConfig, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(BackupOffsiteConfig::default());
    }
    if let Some(rest) = trimmed.strip_prefix("s3://") {
        let mut parts = rest.splitn(2, '/');
        let bucket = parts.next().unwrap_or("").trim();
        let prefix = parts.next().unwrap_or("").trim().trim_matches('/');
        if bucket.is_empty() {
            return Err("s3:// URL must include a bucket name".to_string());
        }
        let region = std::env::var("ERGON_BACKUP_S3_REGION")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_S3_REGION.to_string());
        let endpoint = std::env::var("ERGON_BACKUP_S3_ENDPOINT")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        return Ok(BackupOffsiteConfig {
            kind: "s3".to_string(),
            folder_path: None,
            s3_endpoint: endpoint,
            s3_bucket: Some(bucket.to_string()),
            s3_region: region,
            s3_prefix: prefix.to_string(),
            s3_path_style: true,
        });
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let scheme_end = trimmed.find("://").unwrap() + 3;
        let after_scheme = &trimmed[scheme_end..];
        let (authority, path_part) = after_scheme
            .split_once('/')
            .ok_or("HTTP(S) S3 URL must include /<bucket>")?;
        let mut segs = path_part.trim_matches('/').splitn(2, '/');
        let bucket = segs.next().unwrap_or("").trim();
        let prefix = segs.next().unwrap_or("").trim().trim_matches('/');
        if bucket.is_empty() {
            return Err("HTTP(S) S3 URL must include a bucket name".to_string());
        }
        let endpoint = format!("{}{authority}", &trimmed[..scheme_end]);
        let region = std::env::var("ERGON_BACKUP_S3_REGION")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_S3_REGION.to_string());
        return Ok(BackupOffsiteConfig {
            kind: "s3".to_string(),
            folder_path: None,
            s3_endpoint: Some(endpoint),
            s3_bucket: Some(bucket.to_string()),
            s3_region: region,
            s3_prefix: prefix.to_string(),
            s3_path_style: true,
        });
    }
    Ok(BackupOffsiteConfig {
        kind: "folder".to_string(),
        folder_path: Some(trimmed.to_string()),
        ..BackupOffsiteConfig::default()
    })
}

// ---------------------------------------------------------------------------
// Backup Execution (Local + Retention + Offsite)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct OffsiteSyncReport {
    pub kind: String,
    pub ok: bool,
    pub target: String,
    pub pruned: Vec<String>,
    pub detail: String,
}

impl OffsiteSyncReport {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "ok": self.ok,
            "target": self.target,
            "pruned": self.pruned,
            "detail": self.detail,
        })
    }
}

#[derive(Clone, Debug)]
pub struct BackupRunReport {
    pub filename: String,
    pub path: PathBuf,
    pub sha256_path: PathBuf,
    pub sha256: String,
    pub manifest_sha256: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub zip_bytes: u64,
    pub created_at_utc: String,
    pub created_at_ms: u64,
    pub pruned_local: Vec<String>,
    pub offsite: Option<OffsiteSyncReport>,
}

impl BackupRunReport {
    pub fn to_json(&self) -> Value {
        json!({
            "ok": true,
            "filename": self.filename,
            "path": self.path.to_string_lossy(),
            "sha256_path": self.sha256_path.to_string_lossy(),
            "sha256": self.sha256,
            "manifest_sha256": self.manifest_sha256,
            "file_count": self.file_count,
            "total_bytes": self.total_bytes,
            "zip_bytes": self.zip_bytes,
            "created_at_utc": self.created_at_utc,
            "created_at_ms": self.created_at_ms,
            "pruned_local": self.pruned_local,
            "offsite": self.offsite.as_ref().map(OffsiteSyncReport::to_json),
        })
    }
}

fn pick_unique_backup_path(dir: &Path, created_at_ms: u64, prefix: &str) -> (String, PathBuf) {
    let stamp = format_archive_stamp(created_at_ms);
    let seq = BACKUP_SEQ.fetch_add(1, Ordering::Relaxed);
    let base_name = format!("{prefix}-{stamp}.zip");
    let candidate = dir.join(&base_name);
    if !candidate.exists() {
        return (base_name, candidate);
    }
    for n in 1..=9999u64 {
        let name = format!("{prefix}-{stamp}-{:02}.zip", (seq + n) % 10_000);
        let p = dir.join(&name);
        if !p.exists() {
            return (name, p);
        }
    }
    (base_name, candidate)
}

/// Run a full verified backup of `data_dir`:
/// - Writes verified ZIP + `.sha256` sidecar atomically
/// - Self-verifies the written archive immediately
/// - Applies local retention pruning (`keep` count)
/// - Replicates to configured offsite target (`folder` or `s3`)
/// - Updates `backup_state.json`
pub fn run_backup_with_config(
    data_dir: &Path,
    cfg: &BackupConfig,
    explicit_out: Option<&Path>,
) -> Result<BackupRunReport, String> {
    cfg.validate()?;

    let local_dir = cfg.effective_local_dir(data_dir);
    let mut excludes = vec![local_dir.clone()];
    if cfg.offsite.kind == "folder" {
        if let Some(ref f) = cfg.offsite.folder_path {
            excludes.push(PathBuf::from(f));
        }
    }
    if let Some(out) = explicit_out {
        excludes.push(out.to_path_buf());
    }

    let built = build_backup_archive(data_dir, &excludes)?;
    // Immediately self-verify the generated archive in memory before writing.
    let self_check = verify_backup_archive_bytes(&built.zip_bytes);
    if !self_check.valid {
        return Err(format!(
            "backup self-verification failed: {}",
            self_check.errors.join("; ")
        ));
    }

    let (target_dir, filename, archive_path, apply_retention) = match explicit_out {
        Some(p) if p.to_string_lossy().to_ascii_lowercase().ends_with(".zip") => {
            let parent = p
                .parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            std::fs::create_dir_all(&parent)
                .map_err(|e| format!("create output directory '{}': {e}", parent.display()))?;
            let fname = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("ergon-backup.zip")
                .to_string();
            (parent, fname, p.to_path_buf(), false)
        }
        Some(dir) => {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("create output directory '{}': {e}", dir.display()))?;
            let (fname, full) =
                pick_unique_backup_path(dir, built.manifest.created_at_ms, "ergon-backup");
            (dir.to_path_buf(), fname, full, true)
        }
        None => {
            std::fs::create_dir_all(&local_dir)
                .map_err(|e| format!("create backup directory '{}': {e}", local_dir.display()))?;
            let (fname, full) =
                pick_unique_backup_path(&local_dir, built.manifest.created_at_ms, "ergon-backup");
            (local_dir.clone(), fname, full, true)
        }
    };

    atomic_write_bytes(&archive_path, &built.zip_bytes)
        .map_err(|e| format!("write backup archive '{}': {e}", archive_path.display()))?;
    let sha256_path = PathBuf::from(format!("{}.sha256", archive_path.display()));
    let sha256_body = format!("{}  {filename}\n", built.sha256);
    atomic_write_bytes(&sha256_path, sha256_body.as_bytes())
        .map_err(|e| format!("write sha256 sidecar: {e}"))?;

    let pruned_local = if apply_retention || filename.starts_with("ergon-backup-") {
        prune_backups_in_dir(&target_dir, cfg.retention_count)
    } else {
        Vec::new()
    };

    // Optional offsite sync
    let offsite_report = match cfg.offsite.kind.as_str() {
        "folder" => {
            let dest_root = PathBuf::from(
                cfg.offsite
                    .folder_path
                    .as_deref()
                    .unwrap_or("")
                    .trim(),
            );
            let sync_res: Result<OffsiteSyncReport, String> = (|| {
                std::fs::create_dir_all(&dest_root).map_err(|e| {
                    format!("create offsite directory '{}': {e}", dest_root.display())
                })?;
                let dest_zip = dest_root.join(&filename);
                atomic_write_bytes(&dest_zip, &built.zip_bytes)
                    .map_err(|e| format!("write offsite backup: {e}"))?;
                let dest_sha = PathBuf::from(format!("{}.sha256", dest_zip.display()));
                atomic_write_bytes(&dest_sha, sha256_body.as_bytes())
                    .map_err(|e| format!("write offsite sha256: {e}"))?;
                // Verify copied bytes on disk match SHA-256.
                let copied = std::fs::read(&dest_zip)
                    .map_err(|e| format!("re-read offsite backup: {e}"))?;
                if sha256_hex(&copied) != built.sha256 {
                    return Err("offsite backup copy SHA-256 verification failed".to_string());
                }
                let pruned = prune_backups_in_dir(&dest_root, cfg.retention_count);
                Ok(OffsiteSyncReport {
                    kind: "folder".to_string(),
                    ok: true,
                    target: dest_zip.to_string_lossy().into_owned(),
                    pruned,
                    detail: "verified offsite folder copy".to_string(),
                })
            })();
            match sync_res {
                Ok(r) => Some(r),
                Err(e) => {
                    let mut st = load_backup_state(data_dir);
                    st.last_run_ms = Some(built.manifest.created_at_ms);
                    st.last_backup_name = Some(filename.clone());
                    st.last_backup_sha256 = Some(built.sha256.clone());
                    st.last_offsite_status = Some(format!("error: {e}"));
                    st.last_error = Some(e.clone());
                    save_backup_state(data_dir, &st);
                    return Err(format!("offsite folder sync failed: {e}"));
                }
            }
        }
        "s3" => {
            let creds = match s3_credentials_from_env() {
                Some(c) => c,
                None => {
                    let msg = "S3 offsite configured but ERGON_BACKUP_S3_ACCESS_KEY / ERGON_BACKUP_S3_SECRET_KEY are not set".to_string();
                    let mut st = load_backup_state(data_dir);
                    st.last_run_ms = Some(built.manifest.created_at_ms);
                    st.last_backup_name = Some(filename.clone());
                    st.last_backup_sha256 = Some(built.sha256.clone());
                    st.last_offsite_status = Some(format!("error: {msg}"));
                    st.last_error = Some(msg.clone());
                    save_backup_state(data_dir, &st);
                    return Err(msg);
                }
            };
            let prefix = cfg.offsite.s3_prefix.trim().trim_matches('/');
            let zip_key = if prefix.is_empty() {
                filename.clone()
            } else {
                format!("{prefix}/{filename}")
            };
            let sha_key = format!("{zip_key}.sha256");
            let upload_res = s3_put_object(
                &cfg.offsite,
                &zip_key,
                "application/zip",
                &built.zip_bytes,
                &creds,
            )
            .and_then(|zip_url| {
                let _ = s3_put_object(
                    &cfg.offsite,
                    &sha_key,
                    "text/plain; charset=utf-8",
                    sha256_body.as_bytes(),
                    &creds,
                )?;
                Ok(zip_url)
            });
            match upload_res {
                Ok(url) => Some(OffsiteSyncReport {
                    kind: "s3".to_string(),
                    ok: true,
                    target: url,
                    pruned: Vec::new(),
                    detail: "uploaded via AWS SigV4 PUT".to_string(),
                }),
                Err(e) => {
                    let mut st = load_backup_state(data_dir);
                    st.last_run_ms = Some(built.manifest.created_at_ms);
                    st.last_backup_name = Some(filename.clone());
                    st.last_backup_sha256 = Some(built.sha256.clone());
                    st.last_offsite_status = Some(format!("error: {e}"));
                    st.last_error = Some(e.clone());
                    save_backup_state(data_dir, &st);
                    return Err(format!("offsite S3 upload failed: {e}"));
                }
            }
        }
        _ => None,
    };

    let state = BackupRuntimeState {
        last_run_ms: Some(built.manifest.created_at_ms),
        last_backup_name: Some(filename.clone()),
        last_backup_sha256: Some(built.sha256.clone()),
        last_file_count: Some(built.manifest.file_count),
        last_zip_bytes: Some(built.zip_bytes.len() as u64),
        last_offsite_status: offsite_report.as_ref().map(|r| r.detail.clone()),
        last_error: None,
    };
    save_backup_state(data_dir, &state);

    Ok(BackupRunReport {
        filename,
        path: archive_path,
        sha256_path,
        sha256: built.sha256,
        manifest_sha256: built.manifest.manifest_sha256,
        file_count: built.manifest.file_count,
        total_bytes: built.manifest.total_bytes,
        zip_bytes: built.zip_bytes.len() as u64,
        created_at_utc: built.manifest.created_at_utc,
        created_at_ms: built.manifest.created_at_ms,
        pruned_local,
        offsite: offsite_report,
    })
}

// ---------------------------------------------------------------------------
// Atomic Verified Restore
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RestoreSummary {
    pub archive_path: PathBuf,
    pub target_data_dir: PathBuf,
    pub archive_sha256: String,
    pub manifest_sha256: String,
    pub ergon_version: String,
    pub created_at_utc: String,
    pub restored_files: usize,
    pub total_bytes: u64,
    pub verified_on_disk: bool,
    pub pre_restore_snapshot: Option<String>,
}

impl RestoreSummary {
    pub fn to_json(&self) -> Value {
        json!({
            "ok": true,
            "archive": self.archive_path.to_string_lossy(),
            "data_dir": self.target_data_dir.to_string_lossy(),
            "archive_sha256": self.archive_sha256,
            "manifest_sha256": self.manifest_sha256,
            "ergon_version": self.ergon_version,
            "created_at_utc": self.created_at_utc,
            "restored_files": self.restored_files,
            "total_bytes": self.total_bytes,
            "verified_on_disk": self.verified_on_disk,
            "pre_restore_snapshot": self.pre_restore_snapshot,
        })
    }
}

fn dir_has_non_backup_content(dir: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "backups" || is_transient_temp_or_lock(&name) {
            continue;
        }
        return true;
    }
    false
}

fn copy_dir_recursive_no_symlinks(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("create dir '{}': {e}", dst.display()))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("read dir '{}': {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let ft = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        if ft.is_symlink() {
            continue;
        }
        let s = entry.path();
        let d = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_recursive_no_symlinks(&s, &d)?;
        } else if ft.is_file() {
            std::fs::copy(&s, &d).map_err(|e| format!("copy '{}': {e}", s.display()))?;
        }
    }
    Ok(())
}

/// Validate `archive_path` (ZIP integrity, manifest SHA-256, member SHA-256,
/// zero path-traversal/symlinks), extract into a sibling staging directory,
/// preserve a pre-restore safety snapshot if `target_data_dir` is non-empty,
/// atomically swap into `target_data_dir`, and verify every restored file on disk.
pub fn restore_backup(
    archive_path: &Path,
    target_data_dir: &Path,
    force: bool,
) -> Result<RestoreSummary, String> {
    // 1. Read & verify archive completely BEFORE touching target_data_dir.
    let bytes = std::fs::read(archive_path)
        .map_err(|e| format!("cannot read backup archive '{}': {e}", archive_path.display()))?;
    let verification = verify_backup_archive_bytes(&bytes);
    if !verification.valid {
        return Err(format!(
            "backup archive failed verification (target untouched): {}",
            verification.errors.join("; ")
        ));
    }
    let manifest = verification
        .manifest
        .ok_or("verified archive missing parsed manifest")?;
    let reader = BackupZipReader::open(&bytes)?;
    let entry_map: BTreeMap<&str, usize> = reader
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();

    // 2. Check non-empty target directory policy.
    let target_exists = target_data_dir.exists();
    let has_existing_data = target_exists && dir_has_non_backup_content(target_data_dir);
    if has_existing_data && !force {
        return Err(format!(
            "target data directory '{}' is not empty; pass --force to restore (a pre-restore safety snapshot will be preserved)",
            target_data_dir.display()
        ));
    }

    // 3. Prepare sibling temp staging directory (same parent => atomic rename).
    let target_abs = if target_data_dir.is_absolute() {
        target_data_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("current_dir: {e}"))?
            .join(target_data_dir)
    };
    let parent_dir = target_abs
        .parent()
        .ok_or("target data directory has no parent directory")?;
    std::fs::create_dir_all(parent_dir)
        .map_err(|e| format!("create parent directory '{}': {e}", parent_dir.display()))?;

    let now_ms = records::now_millis();
    let seq = BACKUP_SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let stage_dir = parent_dir.join(format!(".ergon-restore-stage-{pid}-{now_ms}-{seq}"));
    std::fs::create_dir_all(&stage_dir)
        .map_err(|e| format!("create staging directory: {e}"))?;

    // RAII cleanup guard for staging directory if anything fails before swap.
    struct DirCleanup(Option<PathBuf>);
    impl Drop for DirCleanup {
        fn drop(&mut self) {
            if let Some(ref p) = self.0 {
                let _ = std::fs::remove_dir_all(p);
            }
        }
    }
    let mut stage_guard = DirCleanup(Some(stage_dir.clone()));

    // 4. Extract every verified manifest file into stage_dir with strict confinement.
    let canonical_stage = std::fs::canonicalize(&stage_dir)
        .map_err(|e| format!("canonicalize staging dir: {e}"))?;
    for mf in &manifest.files {
        validate_safe_rel_path(&mf.path)?;
        let &idx = entry_map
            .get(mf.archive_path.as_str())
            .ok_or_else(|| format!("missing entry '{}'", mf.archive_path))?;
        let data = reader.read_entry(&reader.entries[idx])?;
        if sha256_hex(&data) != mf.sha256 {
            return Err(format!("SHA-256 verification failed during extraction for '{}'", mf.path));
        }
        let dest = sandbox_path_resolving(&canonical_stage, &mf.path, false)?;
        if let Some(p) = dest.parent() {
            std::fs::create_dir_all(p)
                .map_err(|e| format!("create staged dir '{}': {e}", p.display()))?;
        }
        std::fs::write(&dest, &data)
            .map_err(|e| format!("write staged file '{}': {e}", mf.path))?;
    }

    // Verify every file inside stage_dir BEFORE swapping!
    if let Err(errs) = verify_data_dir_against_manifest(&stage_dir, &manifest) {
        return Err(format!(
            "staged restore verification failed (target untouched): {}",
            errs.join("; ")
        ));
    }

    // 5. Preserve a pre-restore safety snapshot if target_data_dir has existing data.
    let mut pre_restore_snapshot: Option<String> = None;
    if has_existing_data {
        let backups_dir = target_abs.join("backups");
        std::fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("create pre-restore backups dir: {e}"))?;
        let pre = build_backup_archive(&target_abs, std::slice::from_ref(&backups_dir))?;
        let (pre_name, pre_path) =
            pick_unique_backup_path(&backups_dir, pre.manifest.created_at_ms, "pre-restore");
        atomic_write_bytes(&pre_path, &pre.zip_bytes)
            .map_err(|e| format!("write pre-restore safety snapshot: {e}"))?;
        let pre_sha_path = PathBuf::from(format!("{}.sha256", pre_path.display()));
        let _ = atomic_write_bytes(
            &pre_sha_path,
            format!("{}  {pre_name}\n", pre.sha256).as_bytes(),
        );
        pre_restore_snapshot = Some(pre_name);
    }

    // 6. Atomic swap into target_data_dir (preserving existing backups/ directory).
    if !target_exists {
        std::fs::rename(&stage_dir, &target_abs)
            .map_err(|e| format!("atomic swap into '{}': {e}", target_abs.display()))?;
        stage_guard.0 = None;
    } else {
        let old_dir = parent_dir.join(format!(".ergon-pre-restore-old-{pid}-{now_ms}-{seq}"));
        match std::fs::rename(&target_abs, &old_dir) {
            Ok(()) => {
                if let Err(e) = std::fs::rename(&stage_dir, &target_abs) {
                    // Roll back immediately on swap failure.
                    let _ = std::fs::rename(&old_dir, &target_abs);
                    return Err(format!("atomic swap into '{}' failed: {e}", target_abs.display()));
                }
                stage_guard.0 = None;
                // Carry over existing backups/ directory (including pre-restore snapshot).
                let old_backups = old_dir.join("backups");
                let new_backups = target_abs.join("backups");
                if old_backups.exists()
                    && !new_backups.exists()
                    && std::fs::rename(&old_backups, &new_backups).is_err()
                {
                    let _ = copy_dir_recursive_no_symlinks(&old_backups, &new_backups);
                }
                let _ = std::fs::remove_dir_all(&old_dir);
            }
            Err(_) => {
                // Fallback for environments (e.g. open directory handle on Windows)
                // where renaming target_abs directly is rejected by the OS:
                // clear non-backup entries in target_abs, then copy staged files cleanly.
                if let Ok(rd) = std::fs::read_dir(&target_abs) {
                    for entry in rd.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if name == "backups" {
                            continue;
                        }
                        let p = entry.path();
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            let _ = std::fs::remove_dir_all(&p);
                        } else {
                            let _ = std::fs::remove_file(&p);
                        }
                    }
                }
                copy_dir_recursive_no_symlinks(&stage_dir, &target_abs)?;
            }
        }
    }

    // 7. Post-restore verification of every file on disk inside target_data_dir!
    let verified_count = verify_data_dir_against_manifest(&target_abs, &manifest).map_err(|e| {
        format!(
            "post-restore disk verification failed: {}",
            e.join("; ")
        )
    })?;

    Ok(RestoreSummary {
        archive_path: archive_path.to_path_buf(),
        target_data_dir: target_abs,
        archive_sha256: verification.archive_sha256,
        manifest_sha256: manifest.manifest_sha256,
        ergon_version: manifest.ergon_version,
        created_at_utc: manifest.created_at_utc,
        restored_files: verified_count,
        total_bytes: manifest.total_bytes,
        verified_on_disk: true,
        pre_restore_snapshot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigv4_produces_valid_aws4_authorization_header() {
        let creds = S3Credentials {
            access_key: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: None,
        };
        let body = b"ergon backup payload";
        let req = build_sigv4_s3_put(
            Some("http://127.0.0.1:18999"),
            "backups-bucket",
            "us-east-1",
            "ergon-backups/test.zip",
            true,
            body,
            &creds,
            1_700_000_000_000,
        )
        .unwrap();
        assert_eq!(
            req.url,
            "http://127.0.0.1:18999/backups-bucket/ergon-backups/test.zip"
        );
        assert_eq!(req.host, "127.0.0.1:18999");
        assert_eq!(req.amz_date, "20231114T221320Z");
        assert!(req.authorization.starts_with(
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20231114/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature="
        ));
        let sig = req.authorization.rsplit("Signature=").next().unwrap();
        assert_eq!(sig.len(), 64);
        assert!(sig.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn validate_safe_rel_path_blocks_traversal_and_drive_prefixes() {
        assert!(validate_safe_rel_path("workspace/report.md").is_ok());
        assert!(validate_safe_rel_path("data/people/alice/memory.md").is_ok());
        assert!(validate_safe_rel_path("../escape.txt").is_err());
        assert!(validate_safe_rel_path("data/../../escape.txt").is_err());
        assert!(validate_safe_rel_path("/etc/passwd").is_err());
        assert!(validate_safe_rel_path("..\\escape.txt").is_err());
        assert!(validate_safe_rel_path("C:escape.txt").is_err());
        assert!(validate_safe_rel_path("a\0b.txt").is_err());
    }
}
