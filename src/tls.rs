//! HTTPS/TLS configuration and automatic self-signed certificate generation.
//!
//! Supports:
//! - Explicit certificate/key paths via `ERGON_TLS_CERT` + `ERGON_TLS_KEY`
//! - Automatic self-signed X.509v3 certificate generation (SANs for `localhost`,
//!   `127.0.0.1`, `::1`, plus any configured allowed hosts) stored in
//!   `$ERGON_DATA_DIR/tls/{cert.pem,key.pem}` with `0600` key permissions when
//!   `--tls` or `ERGON_TLS=1` is enabled.

use crate::tools::atomic_write_bytes_with_mode;
use crate::tripwire;
use rcgen::{CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, KeyPair, KeyUsagePurpose};
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct TlsConfig {
    pub server_config: Arc<ServerConfig>,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub self_signed: bool,
    pub fingerprint_sha256: String,
}

impl std::fmt::Debug for TlsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsConfig")
            .field("cert_path", &self.cert_path)
            .field("key_path", &self.key_path)
            .field("self_signed", &self.self_signed)
            .field("fingerprint_sha256", &self.fingerprint_sha256)
            .finish()
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Normalize a host/SAN list: always includes `localhost`, `127.0.0.1`, and `::1`,
/// plus any extra hostnames/IPs passed via `ERGON_ALLOWED_HOSTS` or `ERGON_TLS_SANS`.
fn collect_sans(extra_sans: &[String]) -> Vec<String> {
    let mut out = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    let mut push_clean = |raw: &str| {
        let trimmed = raw.trim().trim_start_matches('[').trim_end_matches(']');
        if trimmed.is_empty() {
            return;
        }
        // Strip :port on IPv4/hostnames (leave bare IPv6 intact).
        let host = if trimmed.parse::<std::net::IpAddr>().is_ok() {
            trimmed.to_string()
        } else {
            trimmed.split(':').next().unwrap_or("").trim().to_ascii_lowercase()
        };
        if !host.is_empty() && !out.iter().any(|existing| existing.eq_ignore_ascii_case(&host)) {
            out.push(host);
        }
    };
    for s in extra_sans {
        push_clean(s);
    }
    if let Ok(env_sans) = std::env::var("ERGON_TLS_SANS") {
        for s in env_sans.split(',') {
            push_clean(s);
        }
    }
    out
}

/// Ensure a self-signed certificate and private key exist in `tls_dir`
/// (`cert.pem` mode `0644`, `key.pem` mode `0600`). Idempotent: if both files
/// already exist and are non-empty, they are left untouched.
pub fn ensure_self_signed_cert(
    tls_dir: &Path,
    extra_sans: &[String],
) -> Result<(PathBuf, PathBuf), String> {
    let cert_path = tls_dir.join("cert.pem");
    let key_path = tls_dir.join("key.pem");
    if cert_path.is_file()
        && key_path.is_file()
        && std::fs::metadata(&cert_path).map(|m| m.len() > 0).unwrap_or(false)
        && std::fs::metadata(&key_path).map(|m| m.len() > 0).unwrap_or(false)
    {
        return Ok((cert_path, key_path));
    }

    std::fs::create_dir_all(tls_dir)
        .map_err(|e| format!("cannot create TLS directory {}: {e}", tls_dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(tls_dir, std::fs::Permissions::from_mode(0o700));
    }

    let sans = collect_sans(extra_sans);
    let mut params = CertificateParams::new(sans)
        .map_err(|e| format!("invalid TLS SAN list: {e}"))?;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "Ergon Local TLS");
    dn.push(DnType::OrganizationName, "Ergon");
    params.distinguished_name = dn;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

    let key_pair = KeyPair::generate()
        .map_err(|e| format!("failed to generate TLS private key: {e}"))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| format!("failed to generate self-signed TLS certificate: {e}"))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    atomic_write_bytes_with_mode(&key_path, key_pem.as_bytes(), Some(0o600))
        .map_err(|e| format!("failed to write TLS private key {}: {e}", key_path.display()))?;
    atomic_write_bytes_with_mode(&cert_path, cert_pem.as_bytes(), Some(0o644))
        .map_err(|e| format!("failed to write TLS certificate {}: {e}", cert_path.display()))?;

    Ok((cert_path, key_path))
}

/// Load a PEM certificate chain and private key into a `rustls::ServerConfig`
/// using the `ring` crypto provider and safe default TLS 1.2/1.3 versions.
pub fn load_tls_config(
    cert_path: &Path,
    key_path: &Path,
    self_signed: bool,
) -> Result<TlsConfig, String> {
    let cert_iter = CertificateDer::pem_file_iter(cert_path)
        .map_err(|e| format!("cannot read TLS certificate {}: {e}", cert_path.display()))?;
    let mut certs: Vec<CertificateDer<'static>> = Vec::new();
    for item in cert_iter {
        let cert = item
            .map_err(|e| format!("invalid PEM certificate in {}: {e}", cert_path.display()))?;
        certs.push(cert);
    }
    if certs.is_empty() {
        return Err(format!(
            "no PEM certificates found in {}",
            cert_path.display()
        ));
    }
    let fingerprint_sha256 = tripwire::sha256_hex(certs[0].as_ref());

    let key = PrivateKeyDer::from_pem_file(key_path)
        .map_err(|e| format!("cannot load TLS private key {}: {e}", key_path.display()))?;

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut server_config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("TLS protocol version setup failed: {e}"))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("TLS certificate/key validation failed: {e}"))?;
    server_config.alpn_protocols = vec![b"http/1.1".to_vec()];

    Ok(TlsConfig {
        server_config: Arc::new(server_config),
        cert_path: cert_path.to_path_buf(),
        key_path: key_path.to_path_buf(),
        self_signed,
        fingerprint_sha256,
    })
}

/// Resolve TLS configuration from environment variables and CLI flags.
///
/// Precedence:
/// 1. `ERGON_TLS_CERT` + `ERGON_TLS_KEY` (explicit certificate/key)
/// 2. `force_tls` (`--tls`) or `ERGON_TLS=1` -> auto-generate self-signed cert in
///    `$ERGON_DATA_DIR/tls/{cert.pem,key.pem}`
/// 3. `None` (plain HTTP on loopback dev/test mode)
pub fn resolve_tls(
    data_dir: &Path,
    force_tls: bool,
    cert_override: Option<&Path>,
    key_override: Option<&Path>,
    extra_sans: &[String],
) -> Result<Option<TlsConfig>, String> {
    let env_cert = std::env::var("ERGON_TLS_CERT")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let env_key = std::env::var("ERGON_TLS_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let cert = cert_override.map(Path::to_path_buf).or(env_cert);
    let key = key_override.map(Path::to_path_buf).or(env_key);

    match (cert, key) {
        (Some(c), Some(k)) => Ok(Some(load_tls_config(&c, &k, false)?)),
        (Some(_), None) => Err("ERGON_TLS_CERT is set without ERGON_TLS_KEY".to_string()),
        (None, Some(_)) => Err("ERGON_TLS_KEY is set without ERGON_TLS_CERT".to_string()),
        (None, None) => {
            if force_tls || env_truthy("ERGON_TLS") {
                let tls_dir = data_dir.join("tls");
                let (c, k) = ensure_self_signed_cert(&tls_dir, extra_sans)?;
                Ok(Some(load_tls_config(&c, &k, true)?))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_loads_self_signed_cert_with_0600_key() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-tls-unit-{}-{}",
            std::process::id(),
            crate::records::now_millis()
        ));
        let (cert, key) =
            ensure_self_signed_cert(&dir, &["desk.internal:8787".to_string()]).unwrap();
        assert!(cert.is_file());
        assert!(key.is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&key).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "TLS private key must be 0600");
        }
        let cfg = load_tls_config(&cert, &key, true).unwrap();
        assert!(cfg.self_signed);
        assert_eq!(cfg.fingerprint_sha256.len(), 64);
        // Second call is idempotent (same cert fingerprint)
        let (cert2, key2) = ensure_self_signed_cert(&dir, &[]).unwrap();
        let cfg2 = load_tls_config(&cert2, &key2, true).unwrap();
        assert_eq!(cfg.fingerprint_sha256, cfg2.fingerprint_sha256);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
