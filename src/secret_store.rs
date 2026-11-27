//! Authenticated encrypted-at-rest secret store and OS keychain envelope.
//!
//! All connector credentials (SMTP/IMAP usernames & passwords, OAuth2 client
//! secrets / refresh tokens / access tokens, webhook signing secrets & hook
//! tokens, REST/MCP bearer & env tokens) and saved model API keys are encrypted
//! individually with AES-256-GCM (`ring::aead`), using:
//! - a fresh cryptographically random 96-bit nonce per encryption, and
//! - authenticated associated data (AAD) bound to the exact secret key name
//!   (`ergon-secret-vault:v1:<key>`) to defeat key-substitution / splice attacks.
//!
//! Ciphertexts persist in `<data_dir>/secrets.enc.json` (`0600` POSIX).
//!
//! Master 256-bit key resolution order:
//! 1. Explicit per-`data_dir` override (for isolated tests) or `ERGON_SECRET_KEY`
//!    environment variable (keeps key material completely off disk).
//! 2. OS keychain envelope when `ERGON_SECRET_BACKEND=keychain` or auto-detected
//!    on a non-headless desktop (`secret-tool` on Linux libsecret, `security` on
//!    macOS Keychain, DPAPI `ProtectedData` on Windows), storing the 256-bit
//!    master key outside plaintext disk files.
//! 3. Headless / CI / fallback: `<data_dir>/.vault.key` (`0600` POSIX), created
//!    atomically on first use with `ring::rand::SystemRandom`.

use crate::tools::atomic_write_bytes_with_mode;
use crate::{records, tripwire};
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

pub const VAULT_FILE: &str = "secrets.enc.json";
pub const VAULT_KEY_FILE: &str = ".vault.key";
pub const VAULT_DPAPI_KEY_FILE: &str = ".vault.key.dpapi";
pub const VAULT_REF_PREFIX: &str = "vault:";
pub const VAULT_VERSION: u64 = 1;
pub const VAULT_CIPHER: &str = "aes-256-gcm";

const AAD_PREFIX: &str = "ergon-secret-vault:v1:";
const ENV_KEY_DOMAIN: &str = "ergon-secret-vault:env-key:v1:";
#[cfg(any(target_os = "linux", target_os = "macos"))]
const KEYCHAIN_SERVICE: &str = "ergon-vault-key";
const KEYCHAIN_TIMEOUT: Duration = Duration::from_millis(1500);

static VAULT_LOCK: Mutex<()> = Mutex::new(());
static VAULT_OVERRIDES: OnceLock<RwLock<HashMap<PathBuf, VaultOverride>>> = OnceLock::new();

fn with_vault_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = VAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn vault_path(data_dir: &Path) -> PathBuf {
    data_dir.join(VAULT_FILE)
}

pub fn vault_key_path(data_dir: &Path) -> PathBuf {
    data_dir.join(VAULT_KEY_FILE)
}

pub fn vault_ref(key: &str) -> String {
    format!("{VAULT_REF_PREFIX}{key}")
}

pub fn parse_vault_ref(value: &str) -> Option<&str> {
    value.trim().strip_prefix(VAULT_REF_PREFIX)
}

pub fn is_vault_ref(value: &str) -> bool {
    parse_vault_ref(value).is_some_and(|k| !k.is_empty())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendPreference {
    Auto,
    AeadFile,
    OsKeychain,
}

#[derive(Clone, Debug, Default)]
pub struct VaultOverride {
    pub key: Option<String>,
    pub backend: Option<BackendPreference>,
    pub disable_os_keychain_probe: bool,
}

fn canonical_or_lexical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn overrides_map() -> &'static RwLock<HashMap<PathBuf, VaultOverride>> {
    VAULT_OVERRIDES.get_or_init(|| RwLock::new(HashMap::new()))
}

fn lookup_override(data_dir: &Path) -> Option<VaultOverride> {
    let map = overrides_map().read().unwrap_or_else(|p| p.into_inner());
    if map.is_empty() {
        return None;
    }
    let target = canonical_or_lexical(data_dir);
    if let Some(ov) = map.get(&target) {
        return Some(ov.clone());
    }
    if let Some(ov) = map.get(data_dir) {
        return Some(ov.clone());
    }
    for (base, ov) in map.iter() {
        if target.starts_with(base) || data_dir.starts_with(base) {
            return Some(ov.clone());
        }
    }
    None
}

/// RAII guard for overriding vault key/backend settings for a single `data_dir`
/// without mutating process-global environment variables.
pub struct ScopedVaultOverride {
    key_path: PathBuf,
    prev: Option<VaultOverride>,
}

impl ScopedVaultOverride {
    pub fn new(data_dir: &Path, ov: VaultOverride) -> Self {
        let key_path = canonical_or_lexical(data_dir);
        let mut map = overrides_map().write().unwrap_or_else(|p| p.into_inner());
        let prev = map.insert(key_path.clone(), ov);
        Self { key_path, prev }
    }
}

impl Drop for ScopedVaultOverride {
    fn drop(&mut self) {
        let mut map = overrides_map().write().unwrap_or_else(|p| p.into_inner());
        if let Some(prev) = self.prev.take() {
            map.insert(self.key_path.clone(), prev);
        } else {
            map.remove(&self.key_path);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySource {
    Env,
    OsKeychain,
    KeyFile,
}

impl KeySource {
    pub fn as_str(self) -> &'static str {
        match self {
            KeySource::Env => "env",
            KeySource::OsKeychain => "os_keychain",
            KeySource::KeyFile => "key_file",
        }
    }

    pub fn backend_str(self) -> &'static str {
        match self {
            KeySource::OsKeychain => "os_keychain_envelope",
            KeySource::Env | KeySource::KeyFile => "aead_file",
        }
    }
}

struct KeyBytes([u8; 32]);

impl Drop for KeyBytes {
    fn drop(&mut self) {
        for b in &mut self.0 {
            // SAFETY: volatile write to owned stack bytes prevents dead-store
            // elimination from leaving raw key material in stack memory.
            unsafe {
                std::ptr::write_volatile(b, 0);
            }
        }
    }
}

fn decode_hex_32(hex_str: &str) -> Option<[u8; 32]> {
    let trimmed = hex_str.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = (bytes[i * 2] as char).to_digit(16)? as u8;
        let lo = (bytes[i * 2 + 1] as char).to_digit(16)? as u8;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if clean.is_empty() {
        return Ok(Vec::new());
    }
    if clean.len() % 4 != 0 {
        return Err("invalid base64 length".to_string());
    }
    let val = |b: u8| -> Result<u8, String> {
        match b {
            b'A'..=b'Z' => Ok(b - b'A'),
            b'a'..=b'z' => Ok(b - b'a' + 26),
            b'0'..=b'9' => Ok(b - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err("invalid base64 character".to_string()),
        }
    };
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for (idx, chunk) in clean.chunks_exact(4).enumerate() {
        let is_last = (idx + 1) * 4 == clean.len();
        let a = val(chunk[0])?;
        let b = val(chunk[1])?;
        if chunk[2] == b'=' {
            if !is_last || chunk[3] != b'=' {
                return Err("invalid base64 padding".to_string());
            }
            out.push((a << 2) | (b >> 4));
        } else if chunk[3] == b'=' {
            if !is_last {
                return Err("invalid base64 padding".to_string());
            }
            let c = val(chunk[2])?;
            out.push((a << 2) | (b >> 4));
            out.push((b << 4) | (c >> 2));
        } else {
            let c = val(chunk[2])?;
            let d = val(chunk[3])?;
            out.push((a << 2) | (b >> 4));
            out.push((b << 4) | (c >> 2));
            out.push((c << 6) | d);
        }
    }
    Ok(out)
}

fn derive_key_from_str(raw: &str) -> KeyBytes {
    let trimmed = raw.trim();
    if let Some(exact) = decode_hex_32(trimmed) {
        return KeyBytes(exact);
    }
    if let Ok(decoded) = base64_decode(trimmed) {
        if decoded.len() == 32 {
            let mut exact = [0u8; 32];
            exact.copy_from_slice(&decoded);
            return KeyBytes(exact);
        }
    }
    let mut material = Vec::with_capacity(ENV_KEY_DOMAIN.len() + trimmed.len());
    material.extend_from_slice(ENV_KEY_DOMAIN.as_bytes());
    material.extend_from_slice(trimmed.as_bytes());
    KeyBytes(tripwire::sha256(&material))
}

fn generate_random_key() -> Result<KeyBytes, String> {
    let rng = SystemRandom::new();
    let mut buf = [0u8; 32];
    rng.fill(&mut buf)
        .map_err(|_| "OS CSPRNG failed while generating vault key".to_string())?;
    Ok(KeyBytes(buf))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn dir_fingerprint(data_dir: &Path) -> String {
    let canon = canonical_or_lexical(data_dir);
    let digest = tripwire::sha256(canon.to_string_lossy().as_bytes());
    tripwire::hex(&digest[..12])
}

fn is_temp_data_dir(data_dir: &Path) -> bool {
    let tmp = canonical_or_lexical(&std::env::temp_dir());
    let target = canonical_or_lexical(data_dir);
    target.starts_with(&tmp) || data_dir.starts_with(std::env::temp_dir())
}

fn env_backend_preference() -> BackendPreference {
    match std::env::var("ERGON_SECRET_BACKEND")
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("aead" | "file" | "vault" | "encrypted_file") => BackendPreference::AeadFile,
        Some("keychain" | "os" | "os_keychain") => BackendPreference::OsKeychain,
        _ => BackendPreference::Auto,
    }
}

fn run_cmd_with_timeout(
    mut cmd: Command,
    stdin_payload: Option<&[u8]>,
    timeout: Duration,
) -> Result<(bool, Vec<u8>), String> {
    cmd.stdin(if stdin_payload.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn keychain helper: {e}"))?;
    if let Some(payload) = stdin_payload {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(payload)
                .map_err(|e| format!("write keychain stdin: {e}"))?;
            let _ = stdin.flush();
        }
    }
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout_buf = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = std::io::Read::read_to_end(&mut out, &mut stdout_buf);
                }
                return Ok((status.success(), stdout_buf));
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("OS keychain command timed out".to_string());
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("wait keychain helper: {e}"));
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn os_keychain_lookup(data_dir: &Path) -> Result<Option<KeyBytes>, String> {
    if std::env::var("DBUS_SESSION_BUS_ADDRESS")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        return Err("DBus session unavailable".to_string());
    }
    let acct = dir_fingerprint(data_dir);
    let mut cmd = Command::new("secret-tool");
    cmd.args(["lookup", "service", KEYCHAIN_SERVICE, "account", &acct]);
    let (ok, stdout) = run_cmd_with_timeout(cmd, None, KEYCHAIN_TIMEOUT)?;
    if !ok {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let exact = decode_hex_32(trimmed)
        .ok_or_else(|| "OS keychain entry is not a valid 256-bit vault key".to_string())?;
    Ok(Some(KeyBytes(exact)))
}

#[cfg(target_os = "linux")]
fn os_keychain_store(data_dir: &Path, key: &KeyBytes) -> Result<(), String> {
    if std::env::var("DBUS_SESSION_BUS_ADDRESS")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        return Err("DBus session unavailable".to_string());
    }
    let acct = dir_fingerprint(data_dir);
    let hex_key = tripwire::hex(&key.0);
    let mut cmd = Command::new("secret-tool");
    cmd.args([
        "store",
        "--label=Ergon Secret Vault Key",
        "service",
        KEYCHAIN_SERVICE,
        "account",
        &acct,
    ]);
    let (ok, _) = run_cmd_with_timeout(cmd, Some(hex_key.as_bytes()), KEYCHAIN_TIMEOUT)?;
    if !ok {
        return Err("secret-tool store returned non-zero status".to_string());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn os_keychain_lookup(data_dir: &Path) -> Result<Option<KeyBytes>, String> {
    let acct = dir_fingerprint(data_dir);
    let mut cmd = Command::new("security");
    cmd.args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-a", &acct, "-w"]);
    let (ok, stdout) = run_cmd_with_timeout(cmd, None, KEYCHAIN_TIMEOUT)?;
    if !ok {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let exact = decode_hex_32(trimmed)
        .ok_or_else(|| "macOS Keychain entry is not a valid 256-bit vault key".to_string())?;
    Ok(Some(KeyBytes(exact)))
}

#[cfg(target_os = "macos")]
fn os_keychain_store(data_dir: &Path, key: &KeyBytes) -> Result<(), String> {
    let acct = dir_fingerprint(data_dir);
    let hex_key = tripwire::hex(&key.0);
    // Pass via stdin to `security -i` so the key never appears in process argv.
    let line = format!(
        "add-generic-password -U -s {KEYCHAIN_SERVICE} -a {acct} -w {hex_key}\n"
    );
    let mut cmd = Command::new("security");
    cmd.arg("-i");
    let (ok, _) = run_cmd_with_timeout(cmd, Some(line.as_bytes()), KEYCHAIN_TIMEOUT)?;
    if !ok {
        return Err("macOS security keychain store failed".to_string());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn os_keychain_lookup(data_dir: &Path) -> Result<Option<KeyBytes>, String> {
    let dpapi_path = data_dir.join(VAULT_DPAPI_KEY_FILE);
    let Ok(blob_b64) = std::fs::read_to_string(&dpapi_path) else {
        return Ok(None);
    };
    let trimmed = blob_b64.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let script = "Add-Type -AssemblyName System.Security; \
                  $in = [Console]::In.ReadToEnd().Trim(); \
                  $enc = [Convert]::FromBase64String($in); \
                  $raw = [System.Security.Cryptography.ProtectedData]::Unprotect($enc, $null, [System.Security.Cryptography.DataProtectionScope]::CurrentUser); \
                  [Console]::Out.Write([Text.Encoding]::UTF8.GetString($raw))";
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    let (ok, stdout) = run_cmd_with_timeout(cmd, Some(trimmed.as_bytes()), KEYCHAIN_TIMEOUT)?;
    if !ok {
        return Err("Windows DPAPI Unprotect failed".to_string());
    }
    let text = String::from_utf8_lossy(&stdout);
    let exact = decode_hex_32(text.trim())
        .ok_or_else(|| "Windows DPAPI entry is not a valid 256-bit vault key".to_string())?;
    Ok(Some(KeyBytes(exact)))
}

#[cfg(target_os = "windows")]
fn os_keychain_store(data_dir: &Path, key: &KeyBytes) -> Result<(), String> {
    let hex_key = tripwire::hex(&key.0);
    let script = "Add-Type -AssemblyName System.Security; \
                  $in = [Console]::In.ReadToEnd().Trim(); \
                  $raw = [Text.Encoding]::UTF8.GetBytes($in); \
                  $enc = [System.Security.Cryptography.ProtectedData]::Protect($raw, $null, [System.Security.Cryptography.DataProtectionScope]::CurrentUser); \
                  [Console]::Out.Write([Convert]::ToBase64String($enc))";
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    let (ok, stdout) = run_cmd_with_timeout(cmd, Some(hex_key.as_bytes()), KEYCHAIN_TIMEOUT)?;
    if !ok || stdout.is_empty() {
        return Err("Windows DPAPI Protect failed".to_string());
    }
    let dpapi_path = data_dir.join(VAULT_DPAPI_KEY_FILE);
    atomic_write_bytes_with_mode(&dpapi_path, &stdout, Some(0o600))
        .map_err(|e| format!("save Windows DPAPI vault key: {e}"))?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn os_keychain_lookup(_data_dir: &Path) -> Result<Option<KeyBytes>, String> {
    Err("OS keychain unsupported on this target".to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn os_keychain_store(_data_dir: &Path, _key: &KeyBytes) -> Result<(), String> {
    Err("OS keychain unsupported on this target".to_string())
}

fn load_or_create_key_file(data_dir: &Path, allow_create: bool) -> Result<Option<KeyBytes>, String> {
    let path = vault_key_path(data_dir);
    if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("read vault key file: {e}"))?;
        let exact = decode_hex_32(&raw)
            .ok_or_else(|| "corrupt vault key file (.vault.key): expected 64 hex chars".to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        return Ok(Some(KeyBytes(exact)));
    }
    if !allow_create {
        return Ok(None);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create vault dir: {e}"))?;
    }
    let key = generate_random_key()?;
    let content = format!("{}\n", tripwire::hex(&key.0));

    #[cfg(unix)]
    let create_res = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .and_then(|mut f| f.write_all(content.as_bytes()))
    };
    #[cfg(not(unix))]
    let create_res = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .and_then(|mut f| f.write_all(content.as_bytes()));

    match create_res {
        Ok(()) => Ok(Some(key)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let raw = std::fs::read_to_string(&path)
                .map_err(|re| format!("read existing vault key file: {re}"))?;
            let exact = decode_hex_32(&raw).ok_or_else(|| {
                "corrupt vault key file (.vault.key): expected 64 hex chars".to_string()
            })?;
            Ok(Some(KeyBytes(exact)))
        }
        Err(e) => Err(format!("create vault key file: {e}")),
    }
}

fn read_vault_key_source_hint(data_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(vault_path(data_dir)).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    v["key_source"].as_str().map(str::to_string)
}

fn resolve_master_key(
    data_dir: &Path,
    allow_create: bool,
) -> Result<Option<(KeyBytes, KeySource)>, String> {
    let ov = lookup_override(data_dir);
    if let Some(custom_key) = ov.as_ref().and_then(|o| o.key.as_ref()) {
        if !custom_key.trim().is_empty() {
            return Ok(Some((derive_key_from_str(custom_key), KeySource::Env)));
        }
    }
    if ov.as_ref().and_then(|o| o.key.as_ref()).is_none() {
        if let Ok(env_key) = std::env::var("ERGON_SECRET_KEY") {
            if !env_key.trim().is_empty() {
                return Ok(Some((derive_key_from_str(&env_key), KeySource::Env)));
            }
        }
    }

    let pref = ov
        .as_ref()
        .and_then(|o| o.backend)
        .unwrap_or_else(env_backend_preference);
    let disable_probe = ov
        .as_ref()
        .map(|o| o.disable_os_keychain_probe)
        .unwrap_or(false);
    let key_file_exists = vault_key_path(data_dir).exists();
    let hint = read_vault_key_source_hint(data_dir);

    let try_keychain = match pref {
        BackendPreference::AeadFile => false,
        BackendPreference::OsKeychain => !disable_probe,
        BackendPreference::Auto => {
            if disable_probe {
                false
            } else if hint.as_deref() == Some("os_keychain") {
                true
            } else {
                !(key_file_exists || is_temp_data_dir(data_dir))
            }
        }
    };

    if try_keychain {
        match os_keychain_lookup(data_dir) {
            Ok(Some(existing)) => return Ok(Some((existing, KeySource::OsKeychain))),
            Ok(None) if allow_create && !key_file_exists => {
                let fresh = generate_random_key()?;
                if os_keychain_store(data_dir, &fresh).is_ok() {
                    return Ok(Some((fresh, KeySource::OsKeychain)));
                }
            }
            Ok(None) | Err(_) => {
                // If an existing vault explicitly expects os_keychain and lookup failed
                // while no .vault.key exists, surface the failure on read rather than
                // creating a mismatched key.
                if !allow_create && hint.as_deref() == Some("os_keychain") && !key_file_exists {
                    return Err(
                        "OS keychain master key unavailable for existing keychain-backed vault"
                            .to_string(),
                    );
                }
            }
        }
    }

    match load_or_create_key_file(data_dir, allow_create)? {
        Some(k) => Ok(Some((k, KeySource::KeyFile))),
        None => Ok(None),
    }
}

#[derive(Clone, Debug)]
struct EncryptedEntry {
    nonce_b64: String,
    ciphertext_b64: String,
    updated_ms: u64,
}

#[derive(Clone, Debug)]
struct VaultDoc {
    key_source: String,
    entries: BTreeMap<String, EncryptedEntry>,
}

fn aad_for_key(secret_key: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(AAD_PREFIX.len() + secret_key.len());
    out.extend_from_slice(AAD_PREFIX.as_bytes());
    out.extend_from_slice(secret_key.as_bytes());
    out
}

fn seal_entry(
    key_bytes: &KeyBytes,
    secret_key: &str,
    plaintext: &str,
    now_ms: u64,
) -> Result<EncryptedEntry, String> {
    let rng = SystemRandom::new();
    let mut nonce_raw = [0u8; 12];
    rng.fill(&mut nonce_raw)
        .map_err(|_| "OS CSPRNG failed while generating 96-bit AEAD nonce".to_string())?;
    let unbound = UnboundKey::new(&aead::AES_256_GCM, &key_bytes.0)
        .map_err(|_| "failed to initialize AES-256-GCM key".to_string())?;
    let key = LessSafeKey::new(unbound);
    let aad = aad_for_key(secret_key);
    let mut in_out = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce_raw),
        Aad::from(&aad),
        &mut in_out,
    )
    .map_err(|_| format!("AES-256-GCM encryption failed for secret '{secret_key}'"))?;
    Ok(EncryptedEntry {
        nonce_b64: tripwire::base64_encode(&nonce_raw),
        ciphertext_b64: tripwire::base64_encode(&in_out),
        updated_ms: now_ms,
    })
}

fn open_entry(
    key_bytes: &KeyBytes,
    secret_key: &str,
    entry: &EncryptedEntry,
) -> Result<String, String> {
    let nonce_vec = base64_decode(&entry.nonce_b64).map_err(|e| {
        format!("secret vault authentication failed for '{secret_key}': invalid nonce ({e})")
    })?;
    if nonce_vec.len() != 12 {
        return Err(format!(
            "secret vault authentication failed for '{secret_key}': nonce must be 12 bytes, got {}",
            nonce_vec.len()
        ));
    }
    let mut nonce_raw = [0u8; 12];
    nonce_raw.copy_from_slice(&nonce_vec);

    let mut ciphertext = base64_decode(&entry.ciphertext_b64).map_err(|e| {
        format!("secret vault authentication failed for '{secret_key}': invalid ciphertext ({e})")
    })?;
    if ciphertext.len() < aead::AES_256_GCM.tag_len() {
        return Err(format!(
            "secret vault authentication failed for '{secret_key}': truncated ciphertext"
        ));
    }
    let unbound = UnboundKey::new(&aead::AES_256_GCM, &key_bytes.0)
        .map_err(|_| "failed to initialize AES-256-GCM key".to_string())?;
    let key = LessSafeKey::new(unbound);
    let aad = aad_for_key(secret_key);
    let plain = key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce_raw),
            Aad::from(&aad),
            &mut ciphertext,
        )
        .map_err(|_| {
            format!(
                "secret vault authentication failed for '{secret_key}': ciphertext, nonce, key, or AAD tampering detected"
            )
        })?;
    String::from_utf8(plain.to_vec()).map_err(|_| {
        format!("secret vault decryption failed for '{secret_key}': invalid UTF-8 payload")
    })
}

fn load_vault_doc_unlocked(data_dir: &Path) -> Result<Option<VaultDoc>, String> {
    let path = vault_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("read encrypted secret vault: {e}"))?;
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| format!("corrupt encrypted secret vault JSON: {e}"))?;
    let version = v["version"].as_u64().unwrap_or(0);
    if version != VAULT_VERSION {
        return Err(format!(
            "unsupported encrypted secret vault version {version}"
        ));
    }
    let cipher = v["cipher"].as_str().unwrap_or("");
    if cipher != VAULT_CIPHER {
        return Err(format!(
            "unsupported encrypted secret vault cipher '{cipher}'"
        ));
    }
    let key_source = v["key_source"]
        .as_str()
        .unwrap_or("key_file")
        .to_string();
    let entries_obj = v["entries"]
        .as_object()
        .ok_or_else(|| "corrupt encrypted secret vault: missing entries map".to_string())?;
    let mut entries = BTreeMap::new();
    for (k, ev) in entries_obj {
        let nonce_b64 = ev["nonce"]
            .as_str()
            .ok_or_else(|| format!("corrupt vault entry '{k}': missing nonce"))?
            .to_string();
        let ciphertext_b64 = ev["ciphertext"]
            .as_str()
            .ok_or_else(|| format!("corrupt vault entry '{k}': missing ciphertext"))?
            .to_string();
        let updated_ms = ev["updated_ms"].as_u64().unwrap_or(0);
        entries.insert(
            k.clone(),
            EncryptedEntry {
                nonce_b64,
                ciphertext_b64,
                updated_ms,
            },
        );
    }
    Ok(Some(VaultDoc {
        key_source,
        entries,
    }))
}

fn save_vault_doc_unlocked(
    data_dir: &Path,
    doc: &VaultDoc,
    source: KeySource,
) -> Result<(), String> {
    let mut entries_json = serde_json::Map::new();
    for (k, e) in &doc.entries {
        entries_json.insert(
            k.clone(),
            json!({
                "nonce": e.nonce_b64,
                "ciphertext": e.ciphertext_b64,
                "updated_ms": e.updated_ms,
            }),
        );
    }
    let payload = json!({
        "version": VAULT_VERSION,
        "cipher": VAULT_CIPHER,
        "backend": source.backend_str(),
        "key_source": source.as_str(),
        "entries": entries_json,
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize encrypted secret vault: {e}"))?;
    atomic_write_bytes_with_mode(&vault_path(data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("save secrets.enc.json: {e}"))
}

/// Decrypt and return a single secret by exact vault key, or `Ok(None)` if absent.
/// Returns `Err(...)` if the vault file is corrupted or AEAD authentication fails.
pub fn get_secret(data_dir: &Path, key: &str) -> Result<Option<String>, String> {
    with_vault_lock(|| {
        let Some(doc) = load_vault_doc_unlocked(data_dir)? else {
            return Ok(None);
        };
        let Some(entry) = doc.entries.get(key) else {
            return Ok(None);
        };
        let (master, _) = resolve_master_key(data_dir, false)?
            .ok_or_else(|| "missing vault master key for encrypted secrets.enc.json".to_string())?;
        open_entry(&master, key, entry).map(Some)
    })
}

/// Encrypt and store a single secret under `key` with a fresh random 96-bit nonce
/// and key-bound AAD. Refuses to overwrite an existing corrupted/tampered vault.
pub fn set_secret(data_dir: &Path, key: &str, value: &str) -> Result<(), String> {
    set_secrets_batch(data_dir, &[(key.to_string(), value.to_string())])
}

/// Atomically encrypt and store multiple secrets in one write.
pub fn set_secrets_batch(data_dir: &Path, pairs: &[(String, String)]) -> Result<(), String> {
    if pairs.is_empty() {
        return Ok(());
    }
    with_vault_lock(|| {
        let existing = load_vault_doc_unlocked(data_dir)?;
        let (master, source) = resolve_master_key(data_dir, true)?
            .ok_or_else(|| "unable to initialize vault master key".to_string())?;
        let mut doc = match existing {
            Some(d) => {
                // Verify existing vault entries before mutating so tampering is never masked.
                for (ek, ev) in &d.entries {
                    open_entry(&master, ek, ev)?;
                }
                d
            }
            None => VaultDoc {
                key_source: source.as_str().to_string(),
                entries: BTreeMap::new(),
            },
        };
        let now_ms = records::now_millis();
        for (k, v) in pairs {
            let enc = seal_entry(&master, k, v, now_ms)?;
            doc.entries.insert(k.clone(), enc);
        }
        doc.key_source = source.as_str().to_string();
        save_vault_doc_unlocked(data_dir, &doc, source)
    })
}

/// Delete a single secret from the encrypted vault. Returns `Ok(true)` if removed.
pub fn delete_secret(data_dir: &Path, key: &str) -> Result<bool, String> {
    with_vault_lock(|| {
        let Some(mut doc) = load_vault_doc_unlocked(data_dir)? else {
            return Ok(false);
        };
        if doc.entries.remove(key).is_none() {
            return Ok(false);
        }
        let (master, source) = resolve_master_key(data_dir, false)?
            .ok_or_else(|| "missing vault master key for encrypted secrets.enc.json".to_string())?;
        for (ek, ev) in &doc.entries {
            open_entry(&master, ek, ev)?;
        }
        doc.key_source = source.as_str().to_string();
        save_vault_doc_unlocked(data_dir, &doc, source)?;
        Ok(true)
    })
}

/// Decrypt all entries whose vault key starts with `prefix`.
/// Fails cleanly if ANY entry in that prefix (or the vault JSON itself) fails
/// AEAD authentication.
pub fn load_prefix_secrets(
    data_dir: &Path,
    prefix: &str,
) -> Result<BTreeMap<String, String>, String> {
    with_vault_lock(|| {
        let Some(doc) = load_vault_doc_unlocked(data_dir)? else {
            return Ok(BTreeMap::new());
        };
        let matching: Vec<(&String, &EncryptedEntry)> = doc
            .entries
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .collect();
        if matching.is_empty() {
            return Ok(BTreeMap::new());
        }
        let (master, _) = resolve_master_key(data_dir, false)?
            .ok_or_else(|| "missing vault master key for encrypted secrets.enc.json".to_string())?;
        let mut out = BTreeMap::new();
        for (k, entry) in matching {
            let plain = open_entry(&master, k, entry)?;
            out.insert(k.clone(), plain);
        }
        Ok(out)
    })
}

/// Atomically synchronize all entries matching `prefix` to `desired`, leaving
/// entries outside `prefix` (such as `model:api_key`) untouched.
/// Validates existing entries first so a tampered vault is never silently overwritten.
pub fn sync_prefix_secrets(
    data_dir: &Path,
    prefix: &str,
    desired: &BTreeMap<String, String>,
) -> Result<(), String> {
    with_vault_lock(|| {
        let existing = load_vault_doc_unlocked(data_dir)?;
        if existing.is_none() && desired.is_empty() {
            return Ok(());
        }
        let (master, source) = resolve_master_key(data_dir, true)?
            .ok_or_else(|| "unable to initialize vault master key".to_string())?;
        let mut doc = match existing {
            Some(d) => {
                for (ek, ev) in &d.entries {
                    open_entry(&master, ek, ev)?;
                }
                d
            }
            None => VaultDoc {
                key_source: source.as_str().to_string(),
                entries: BTreeMap::new(),
            },
        };
        doc.entries.retain(|k, _| !k.starts_with(prefix));
        let now_ms = records::now_millis();
        for (k, v) in desired {
            let enc = seal_entry(&master, k, v, now_ms)?;
            doc.entries.insert(k.clone(), enc);
        }
        doc.key_source = source.as_str().to_string();
        save_vault_doc_unlocked(data_dir, &doc, source)
    })
}

/// Verify that every entry in `secrets.enc.json` decrypts and passes AEAD
/// authentication. Returns the number of verified secrets.
pub fn verify_vault_integrity(data_dir: &Path) -> Result<usize, String> {
    with_vault_lock(|| {
        let Some(doc) = load_vault_doc_unlocked(data_dir)? else {
            return Ok(0);
        };
        if doc.entries.is_empty() {
            return Ok(0);
        }
        let (master, _) = resolve_master_key(data_dir, false)?
            .ok_or_else(|| "missing vault master key for encrypted secrets.enc.json".to_string())?;
        for (k, entry) in &doc.entries {
            let _ = open_entry(&master, k, entry)?;
        }
        Ok(doc.entries.len())
    })
}

/// Non-secret storage metadata safe for API responses (`GET /api/connectors`).
pub fn active_backend_info(data_dir: &Path) -> Value {
    let path = vault_path(data_dir);
    let exists = path.exists();
    let key_source = read_vault_key_source_hint(data_dir).unwrap_or_else(|| {
        if lookup_override(data_dir)
            .and_then(|o| o.key)
            .is_some_and(|k| !k.trim().is_empty())
            || std::env::var("ERGON_SECRET_KEY")
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        {
            "env".to_string()
        } else {
            "key_file".to_string()
        }
    });
    let backend = if key_source == "os_keychain" {
        "os_keychain_envelope"
    } else {
        "aead_file"
    };
    json!({
        "backend": backend,
        "cipher": VAULT_CIPHER,
        "key_source": key_source,
        "vault_exists": exists,
    })
}
