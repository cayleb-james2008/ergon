//! Multi-user authentication, PBKDF2-HMAC-SHA256 password hashing, persisted
//! opaque sessions, optional OIDC (Discovery + Authorization Code + PKCE),
//! and centralized role-based access control (RBAC).

use crate::records::now_millis;
use crate::tools::atomic_write_bytes_with_mode;
use crate::tripwire;
use crate::{Config, PeopleState};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

pub const PBKDF2_ITERATIONS: u32 = 120_000;
const MIN_PBKDF2_ITERATIONS: u32 = 100_000;
const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 256;
const DEFAULT_SESSION_TTL_SECS: u64 = 12 * 3600;
const MIN_SESSION_TTL_SECS: u64 = 300;
const MAX_SESSION_TTL_SECS: u64 = 30 * 86_400;
const OIDC_PENDING_TTL_MS: u64 = 10 * 60 * 1000;
const MAX_PENDING_OIDC: usize = 256;
const LOGIN_FAIL_WINDOW_MS: u64 = 5 * 60 * 1000;
const LOGIN_FAIL_LIMIT: usize = 10;

// ---------------------------------------------------------------------------
// Cryptographic primitives (CSPRNG + RFC 2898 PBKDF2-HMAC-SHA256)
// ---------------------------------------------------------------------------

/// Fill `len` cryptographically secure random bytes using the OS CSPRNG via `ring`.
pub fn secure_random_bytes(len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut buf)
        .map_err(|_| "CSPRNG failure generating secure random bytes".to_string())?;
    Ok(buf)
}

pub fn base64url_encode_nopad(bytes: &[u8]) -> String {
    tripwire::base64_encode(bytes)
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

/// Optimized RFC 2898 PBKDF2-HMAC-SHA256 (32-byte single-block output).
/// Precomputes the inner and outer SHA-256 key states once per call.
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut k = [0u8; 64];
    if password.len() > 64 {
        let h = Sha256::digest(password);
        k[..32].copy_from_slice(&h);
    } else {
        k[..password.len()].copy_from_slice(password);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let inner_base = Sha256::new_with_prefix(ipad);
    let outer_base = Sha256::new_with_prefix(opad);

    // U_1 = PRF(Password, Salt || INT_32_BE(1))
    let mut inner = inner_base.clone();
    inner.update(salt);
    inner.update(1u32.to_be_bytes());
    let ih = inner.finalize();
    let mut outer = outer_base.clone();
    outer.update(ih);
    let mut u: [u8; 32] = outer.finalize().into();
    let mut out = u;

    for _ in 1..iterations {
        let mut i_h = inner_base.clone();
        i_h.update(u);
        let d = i_h.finalize();
        let mut o_h = outer_base.clone();
        o_h.update(d);
        u = o_h.finalize().into();
        for (dst, src) in out.iter_mut().zip(u.iter()) {
            *dst ^= *src;
        }
    }
    out
}

/// Hash a password with a unique 16-byte random salt and PBKDF2-HMAC-SHA256 (>= 120k iters).
pub fn hash_password(password: &str) -> Result<String, String> {
    validate_password_strength(password)?;
    let salt = secure_random_bytes(16)?;
    let dk = pbkdf2_hmac_sha256(password.as_bytes(), &salt, PBKDF2_ITERATIONS);
    Ok(format!(
        "pbkdf2_sha256${}${}${}",
        PBKDF2_ITERATIONS,
        base64url_encode_nopad(&salt),
        base64url_encode_nopad(&dk)
    ))
}

/// Verify a password against a stored PBKDF2-HMAC-SHA256 hash using constant-time equality.
pub fn verify_password(password: &str, stored: &str) -> bool {
    let mut parts = stored.split('$');
    let Some(alg) = parts.next() else {
        return false;
    };
    let Some(iter_s) = parts.next() else {
        return false;
    };
    let Some(salt_b64) = parts.next() else {
        return false;
    };
    let Some(dk_b64) = parts.next() else {
        return false;
    };
    if parts.next().is_some() || alg != "pbkdf2_sha256" {
        return false;
    }
    let Ok(iters) = iter_s.parse::<u32>() else {
        return false;
    };
    if !(MIN_PBKDF2_ITERATIONS..=2_000_000).contains(&iters) {
        return false;
    }
    let Some(salt) = tripwire::base64_decode(salt_b64) else {
        return false;
    };
    let Some(expected) = tripwire::base64_decode(dk_b64) else {
        return false;
    };
    if salt.len() < 12 || expected.len() != 32 {
        return false;
    }
    let derived = pbkdf2_hmac_sha256(password.as_bytes(), &salt, iters);
    tripwire::ct_eq(&derived, &expected)
}

pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters long"
        ));
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err(format!(
            "password must be at most {MAX_PASSWORD_LEN} characters long"
        ));
    }
    Ok(())
}

pub fn normalize_username(raw: &str) -> Result<String, String> {
    let u = raw.trim().to_ascii_lowercase();
    if u.len() < 2 || u.len() > 64 {
        return Err("username must be 2 to 64 characters".to_string());
    }
    if !u
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-' | '@'))
    {
        return Err(
            "username may only contain lowercase letters, digits, '.', '_', '-', or '@'".to_string(),
        );
    }
    if u.starts_with('.') || u.starts_with('-') || u.ends_with('.') || u.ends_with('-') {
        return Err("username must start and end with a letter, digit, or '_'".to_string());
    }
    Ok(u)
}

fn new_opaque_token() -> Result<String, String> {
    let raw = secure_random_bytes(32)?;
    Ok(format!("ergs_{}", base64url_encode_nopad(&raw)))
}

fn token_sha256(token: &str) -> String {
    tripwire::sha256_hex(token.as_bytes())
}

// ---------------------------------------------------------------------------
// Roles & Centralized RBAC
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    Pm,
    Ae,
    Office,
    Installer,
    Member,
}

impl Role {
    pub const ALL: &'static [&'static str] = &["admin", "pm", "ae", "office", "installer", "member"];

    pub fn parse_strict(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "pm" => Some(Self::Pm),
            "ae" => Some(Self::Ae),
            "office" => Some(Self::Office),
            "installer" => Some(Self::Installer),
            "member" => Some(Self::Member),
            _ => None,
        }
    }

    pub fn parse_lenient(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "admin" | "administrator" | "owner" | "it" | "it admin" => Self::Admin,
            "pm" | "project manager" | "project-manager" => Self::Pm,
            "ae" | "account executive" | "account-executive" | "sales" => Self::Ae,
            "office" | "finance" | "operations" | "ops" | "accounting" | "billing" => Self::Office,
            "installer" | "field" | "field installer" | "technician" | "tech" => Self::Installer,
            _ => Self::Member,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Pm => "pm",
            Self::Ae => "ae",
            Self::Office => "office",
            Self::Installer => "installer",
            Self::Member => "member",
        }
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_manage_clients(self) -> bool {
        matches!(self, Self::Admin | Self::Pm | Self::Ae | Self::Office)
    }

    pub fn can_view_invoices(self) -> bool {
        matches!(self, Self::Admin | Self::Pm | Self::Ae | Self::Office)
    }

    pub fn can_manage_invoices(self) -> bool {
        matches!(self, Self::Admin | Self::Ae | Self::Office)
    }

    pub fn can_manage_workspace_policy(self) -> bool {
        matches!(self, Self::Admin | Self::Pm | Self::Ae | Self::Office)
    }

    pub fn satisfies(self, access: RouteAccess) -> bool {
        match access {
            RouteAccess::Public | RouteAccess::PublicSetupOnly | RouteAccess::AuthenticatedSelf => {
                true
            }
            RouteAccess::RequireClientManage => self.can_manage_clients(),
            RouteAccess::RequireInvoiceView => self.can_view_invoices(),
            RouteAccess::RequireInvoiceManage => self.can_manage_invoices(),
            RouteAccess::RequireWorkspacePolicyManage => self.can_manage_workspace_policy(),
            RouteAccess::RequireAdmin => self.is_admin(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteAccess {
    /// Static UI shell, health, login, logout, OIDC redirect/callback, and `/api/auth/me`.
    Public,
    /// `POST /api/auth/setup`: allowed only when 0 accounts exist.
    PublicSetupOnly,
    /// Any authenticated user acting strictly on their own isolated workspace.
    AuthenticatedSelf,
    /// Create/update clients & engagements (`admin`, `pm`, `ae`, `office`).
    RequireClientManage,
    /// View invoices (`admin`, `pm`, `ae`, `office`).
    RequireInvoiceView,
    /// Generate/update invoices (`admin`, `ae`, `office`).
    RequireInvoiceManage,
    /// Change workspace audit policy (`admin`, `pm`, `ae`, `office`).
    RequireWorkspacePolicyManage,
    /// System administration: user CRUD, OIDC config, model config, updates,
    /// security tripwire policy, connectors, MCP servers, creating people (`admin` only).
    RequireAdmin,
}

/// Centralized route authorization table for EVERY HTTP route in `server.rs`.
/// Any unrecognized `/api/*` path fails closed (`AuthenticatedSelf`).
pub fn route_access(method: &str, path: &str) -> RouteAccess {
    match (method, path) {
        // Static UI shell (renders login/setup screen when unauthenticated; zero user data)
        ("GET" | "HEAD", "/" | "/phone") => RouteAccess::Public,
        ("GET" | "HEAD", p) if p.starts_with("/chunks/") => RouteAccess::Public,

        // Health & unauthenticated auth endpoints
        ("GET" | "HEAD", "/api/health") => RouteAccess::Public,
        ("GET", "/api/auth/me") => RouteAccess::Public,
        ("POST", "/api/auth/login" | "/api/auth/logout") => RouteAccess::Public,
        ("GET", "/api/auth/oidc/login" | "/api/auth/oidc/callback") => RouteAccess::Public,
        ("POST", "/api/auth/setup") => RouteAccess::PublicSetupOnly,

        // Authenticated self password change & per-person plugins catalog
        ("POST", "/api/auth/password") => RouteAccess::AuthenticatedSelf,
        ("GET", "/api/plugins") => RouteAccess::AuthenticatedSelf,
        ("GET", "/api/plugins/recommendations") => RouteAccess::AuthenticatedSelf,
        ("POST", "/api/plugins/recommendations/dismiss" | "/api/plugins/recommendations/scan") => {
            RouteAccess::AuthenticatedSelf
        }
        ("POST", "/api/plugins/toggle" | "/api/plugins/action") => RouteAccess::AuthenticatedSelf,
        ("GET", "/api/plugins/oauth/callback") => RouteAccess::Public,
        ("GET", "/api/plugins/oauth/apps") => RouteAccess::AuthenticatedSelf,
        ("PUT" | "POST", "/api/plugins/oauth/apps") => RouteAccess::RequireAdmin,
        ("GET" | "POST", "/api/plugins/oauth/start") => RouteAccess::AuthenticatedSelf,
        ("POST", "/api/plugins/oauth/disconnect") => RouteAccess::AuthenticatedSelf,
        ("GET", "/api/plugins/registry" | "/api/plugins/registry/search") => {
            RouteAccess::AuthenticatedSelf
        }
        ("POST", "/api/plugins/registry/sync" | "/api/plugins/registry/install" | "/api/plugins/registry/uninstall") => {
            RouteAccess::AuthenticatedSelf
        }
        ("PUT" | "POST", "/api/plugins/registry/config") => RouteAccess::RequireAdmin,

        // Admin-only auth & identity management
        (_, p) if p == "/api/auth/users" || p.starts_with("/api/auth/users/") => {
            RouteAccess::RequireAdmin
        }
        (_, "/api/auth/oidc/config") => RouteAccess::RequireAdmin,

        // Admin-only system settings, updates, security, connectors, MCP & people creation
        ("GET" | "PUT" | "POST" | "DELETE", "/api/config/personal") => RouteAccess::AuthenticatedSelf,
        ("GET" | "PUT" | "POST", "/api/config") => RouteAccess::RequireAdmin,
        ("POST", "/api/update/check" | "/api/update/apply" | "/api/update/restart") => {
            RouteAccess::RequireAdmin
        }
        ("GET", "/api/security") | ("POST", "/api/security/policy") => RouteAccess::RequireAdmin,
        ("POST", "/api/people") => RouteAccess::RequireAdmin,
        (_, p) if p == "/api/connectors" || p.starts_with("/api/connectors/") => {
            RouteAccess::RequireAdmin
        }
        (_, p)
            if p == "/api/mcp/servers"
                || p.starts_with("/api/mcp/servers/")
                || p == "/api/mcp/inbound/config" =>
        {
            RouteAccess::RequireAdmin
        }
        // Inbound MCP endpoint requires Admin session (or matching ERGON_MCP_INBOUND_TOKEN).
        ("POST", "/api/mcp/inbound") => RouteAccess::RequireAdmin,

        // CRM / Clients / Engagements / Invoicing / Workspace Policy RBAC
        ("POST", "/api/clients" | "/api/engagements") => RouteAccess::RequireClientManage,
        ("PATCH", p) if p.starts_with("/api/clients/") || p.starts_with("/api/engagements/") => {
            RouteAccess::RequireClientManage
        }
        ("GET", "/api/invoices") => RouteAccess::RequireInvoiceView,
        ("POST", "/api/invoices/generate") => RouteAccess::RequireInvoiceManage,
        ("PATCH", p) if p.starts_with("/api/invoices/") => RouteAccess::RequireInvoiceManage,
        ("PUT" | "POST", "/api/workspace/policy") => RouteAccess::RequireWorkspacePolicyManage,

        // Shared Company Memory, Company DB, Tool Builder, Proactive Offers & Push Notifications
        ("GET", "/api/company/memory") => RouteAccess::AuthenticatedSelf,
        ("POST" | "DELETE", p)
            if p == "/api/company/memory" || p.starts_with("/api/company/memory/") =>
        {
            RouteAccess::RequireClientManage
        }
        ("GET" | "POST", p) if p == "/api/company/db" || p == "/api/company/db/query" => {
            RouteAccess::AuthenticatedSelf
        }
        ("GET", "/api/tools") => RouteAccess::AuthenticatedSelf,
        ("POST", p)
            if p == "/api/tools/test"
                || p == "/api/tools/build"
                || (p.starts_with("/api/tools/") && p.ends_with("/run")) =>
        {
            RouteAccess::AuthenticatedSelf
        }
        ("POST", "/api/offers/analyze") => RouteAccess::AuthenticatedSelf,
        ("GET", "/api/notifications") => RouteAccess::AuthenticatedSelf,
        ("POST", p) if p.starts_with("/api/notifications/") && p.ends_with("/read") => {
            RouteAccess::AuthenticatedSelf
        }
        ("GET" | "POST" | "DELETE", p) if p.starts_with("/api/push/") => {
            RouteAccess::AuthenticatedSelf
        }

        // Scoped API keys, Outbound Webhooks, Self-Improvement & Quality Bars (Wave 3)
        ("GET" | "POST" | "DELETE", p) if p == "/api/keys" || p.starts_with("/api/keys/") => {
            RouteAccess::RequireAdmin
        }
        ("GET" | "POST" | "DELETE", p)
            if p == "/api/webhooks/subscriptions"
                || p.starts_with("/api/webhooks/subscriptions/") =>
        {
            RouteAccess::RequireClientManage
        }
        ("GET", "/api/evolution/scores") => RouteAccess::AuthenticatedSelf,
        ("GET", p) if p.starts_with("/api/evolution/history/") => RouteAccess::AuthenticatedSelf,
        ("POST", "/api/evolution/outcomes") => RouteAccess::AuthenticatedSelf,
        ("POST", "/api/evolution/cases" | "/api/evolution/propose" | "/api/evolution/rollback") => {
            RouteAccess::RequireClientManage
        }
        ("GET", "/api/quality-bars") => RouteAccess::AuthenticatedSelf,
        ("GET" | "POST", p) if p.starts_with("/api/quality-bars/") => {
            RouteAccess::AuthenticatedSelf
        }

        // Every other /api/* route requires authentication and is strictly scoped to caller's workspace
        _ => RouteAccess::AuthenticatedSelf,
    }
}

pub fn access_denied_reason(access: RouteAccess, role: Role) -> String {
    match access {
        RouteAccess::RequireAdmin => {
            format!(
                "forbidden: admin role is required for this endpoint (current role: {})",
                role.as_str()
            )
        }
        RouteAccess::RequireClientManage => {
            format!(
                "forbidden: client and engagement management requires admin, pm, ae, or office role (current role: {})",
                role.as_str()
            )
        }
        RouteAccess::RequireInvoiceView => {
            format!(
                "forbidden: viewing invoices requires admin, pm, ae, or office role (current role: {})",
                role.as_str()
            )
        }
        RouteAccess::RequireInvoiceManage => {
            format!(
                "forbidden: generating or modifying invoices requires admin, ae, or office role (current role: {})",
                role.as_str()
            )
        }
        RouteAccess::RequireWorkspacePolicyManage => {
            format!(
                "forbidden: changing workspace policy requires admin, pm, ae, or office role (current role: {})",
                role.as_str()
            )
        }
        _ => "forbidden".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Authenticated Caller Context
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AuthPrincipal {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub role: Role,
    /// Home person_id belonging to this user.
    pub home_person_id: String,
    /// Effective active person_id for this request (always == home_person_id for non-admins).
    pub active_person_id: String,
    pub auth_method: String,
    pub is_break_glass: bool,
    pub session_token_hash: Option<String>,
    pub api_key_scopes: Option<Vec<String>>,
}

impl AuthPrincipal {
    pub fn has_scope(&self, required: &str) -> bool {
        match &self.api_key_scopes {
            None => true,
            Some(scopes) => scopes.iter().any(|s| s == "admin" || s == required),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "id": self.user_id,
            "username": self.username,
            "display_name": self.display_name,
            "email": self.email,
            "role": self.role.as_str(),
            "person_id": self.home_person_id,
            "active_person_id": self.active_person_id,
            "auth_method": self.auth_method,
            "is_admin": self.role.is_admin(),
            "break_glass": self.is_break_glass,
        });
        if let Some(ref scopes) = self.api_key_scopes {
            obj["scopes"] = json!(scopes);
        }
        obj
    }
}

// ---------------------------------------------------------------------------
// Persisted Users & Sessions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub role: Role,
    pub person_id: String,
    pub password_hash: Option<String>,
    pub oidc_sub: Option<String>,
    pub disabled: bool,
    pub created_ms: u64,
    pub updated_ms: u64,
    pub last_login_ms: Option<u64>,
}

impl AuthUser {
    pub fn to_public_json(&self) -> Value {
        json!({
            "id": self.id,
            "username": self.username,
            "email": self.email,
            "display_name": self.display_name,
            "role": self.role.as_str(),
            "person_id": self.person_id,
            "has_password": self.password_hash.is_some(),
            "oidc_linked": self.oidc_sub.is_some(),
            "disabled": self.disabled,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
            "last_login_ms": self.last_login_ms,
        })
    }

    fn to_storage_json(&self) -> Value {
        json!({
            "id": self.id,
            "username": self.username,
            "email": self.email,
            "display_name": self.display_name,
            "role": self.role.as_str(),
            "person_id": self.person_id,
            "password_hash": self.password_hash,
            "oidc_sub": self.oidc_sub,
            "disabled": self.disabled,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
            "last_login_ms": self.last_login_ms,
        })
    }

    fn from_storage_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let username = normalize_username(v["username"].as_str()?).ok()?;
        let person_id = v["person_id"].as_str()?.trim().to_string();
        if id.is_empty() || !crate::valid_person_id(&person_id) {
            return None;
        }
        let role = Role::parse_lenient(v["role"].as_str().unwrap_or("member"));
        let display_name = v["display_name"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&username)
            .to_string();
        let email = v["email"]
            .as_str()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let password_hash = v["password_hash"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);
        let oidc_sub = v["oidc_sub"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);
        Some(Self {
            id,
            username,
            email,
            display_name,
            role,
            person_id,
            password_hash,
            oidc_sub,
            disabled: v["disabled"].as_bool().unwrap_or(false),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
            last_login_ms: v["last_login_ms"].as_u64(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct AuthSession {
    pub token_hash: String,
    pub user_id: String,
    pub active_person_id: String,
    pub auth_method: String,
    pub created_ms: u64,
    pub last_seen_ms: u64,
    pub expires_at_ms: u64,
}

impl AuthSession {
    fn to_storage_json(&self) -> Value {
        json!({
            "token_hash": self.token_hash,
            "user_id": self.user_id,
            "active_person_id": self.active_person_id,
            "auth_method": self.auth_method,
            "created_ms": self.created_ms,
            "last_seen_ms": self.last_seen_ms,
            "expires_at_ms": self.expires_at_ms,
        })
    }

    fn from_storage_json(v: &Value) -> Option<Self> {
        let token_hash = v["token_hash"].as_str()?.trim().to_string();
        let user_id = v["user_id"].as_str()?.trim().to_string();
        let active_person_id = v["active_person_id"].as_str()?.trim().to_string();
        if token_hash.len() != 64 || user_id.is_empty() || !crate::valid_person_id(&active_person_id)
        {
            return None;
        }
        Some(Self {
            token_hash,
            user_id,
            active_person_id,
            auth_method: v["auth_method"]
                .as_str()
                .unwrap_or("password")
                .to_string(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            last_seen_ms: v["last_seen_ms"].as_u64().unwrap_or(0),
            expires_at_ms: v["expires_at_ms"].as_u64().unwrap_or(0),
        })
    }
}

// ---------------------------------------------------------------------------
// OIDC Configuration & PKCE Flow
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub enabled: bool,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: String,
    pub default_role: Role,
    pub role_claim: String,
    pub auto_provision: bool,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer: String::new(),
            client_id: String::new(),
            client_secret: None,
            redirect_uri: String::new(),
            scopes: "openid profile email".to_string(),
            default_role: Role::Member,
            role_claim: "role".to_string(),
            auto_provision: true,
        }
    }
}

impl OidcConfig {
    pub fn is_configured(&self) -> bool {
        self.enabled && !self.issuer.trim().is_empty() && !self.client_id.trim().is_empty()
    }

    pub fn to_public_json(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "configured": self.is_configured(),
            "issuer": self.issuer,
            "client_id": self.client_id,
            "has_client_secret": self.client_secret.as_deref().is_some_and(|s| !s.is_empty()),
            "redirect_uri": self.redirect_uri,
            "scopes": self.scopes,
            "default_role": self.default_role.as_str(),
            "role_claim": self.role_claim,
            "auto_provision": self.auto_provision,
        })
    }

    fn to_storage_json(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "issuer": self.issuer,
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "redirect_uri": self.redirect_uri,
            "scopes": self.scopes,
            "default_role": self.default_role.as_str(),
            "role_claim": self.role_claim,
            "auto_provision": self.auto_provision,
        })
    }

    fn from_storage_json(v: &Value) -> Self {
        let mut cfg = Self::default();
        if let Some(b) = v["enabled"].as_bool() {
            cfg.enabled = b;
        }
        if let Some(s) = v["issuer"].as_str() {
            cfg.issuer = s.trim().trim_end_matches('/').to_string();
        }
        if let Some(s) = v["client_id"].as_str() {
            cfg.client_id = s.trim().to_string();
        }
        cfg.client_secret = v["client_secret"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);
        if let Some(s) = v["redirect_uri"].as_str() {
            cfg.redirect_uri = s.trim().to_string();
        }
        if let Some(s) = v["scopes"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            cfg.scopes = s.to_string();
        }
        if let Some(r) = v["default_role"].as_str().and_then(Role::parse_strict) {
            cfg.default_role = r;
        }
        if let Some(s) = v["role_claim"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            cfg.role_claim = s.to_string();
        }
        if let Some(b) = v["auto_provision"].as_bool() {
            cfg.auto_provision = b;
        }
        cfg
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(iss) = std::env::var("ERGON_OIDC_ISSUER") {
            let trimmed = iss.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                self.issuer = trimmed;
                self.enabled = true;
            }
        }
        if let Ok(cid) = std::env::var("ERGON_OIDC_CLIENT_ID") {
            let trimmed = cid.trim().to_string();
            if !trimmed.is_empty() {
                self.client_id = trimmed;
            }
        }
        if let Ok(sec) = std::env::var("ERGON_OIDC_CLIENT_SECRET") {
            let trimmed = sec.trim().to_string();
            if !trimmed.is_empty() {
                self.client_secret = Some(trimmed);
            }
        }
        if let Ok(redir) = std::env::var("ERGON_OIDC_REDIRECT_URI") {
            let trimmed = redir.trim().to_string();
            if !trimmed.is_empty() {
                self.redirect_uri = trimmed;
            }
        }
        if let Ok(scopes) = std::env::var("ERGON_OIDC_SCOPES") {
            let trimmed = scopes.trim().to_string();
            if !trimmed.is_empty() {
                self.scopes = trimmed;
            }
        }
        if let Ok(role) = std::env::var("ERGON_OIDC_DEFAULT_ROLE") {
            if let Some(r) = Role::parse_strict(&role) {
                self.default_role = r;
            }
        }
        if let Ok(claim) = std::env::var("ERGON_OIDC_ROLE_CLAIM") {
            let trimmed = claim.trim().to_string();
            if !trimmed.is_empty() {
                self.role_claim = trimmed;
            }
        }
        if let Ok(ap) = std::env::var("ERGON_OIDC_AUTO_PROVISION") {
            let low = ap.trim().to_ascii_lowercase();
            if matches!(low.as_str(), "0" | "false" | "no" | "off") {
                self.auto_provision = false;
            } else if matches!(low.as_str(), "1" | "true" | "yes" | "on") {
                self.auto_provision = true;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct OidcDiscovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
}

#[derive(Clone, Debug)]
struct OidcPendingFlow {
    nonce: String,
    code_verifier: String,
    redirect_uri: String,
    redirect_after: String,
    created_ms: u64,
}

fn validate_oidc_issuer_url(issuer: &str) -> Result<(), String> {
    let trimmed = issuer.trim();
    if trimmed.starts_with("https://") {
        return Ok(());
    }
    if trimmed.starts_with("http://") && crate::is_loopback_url(trimmed) {
        return Ok(());
    }
    Err("OIDC issuer must use https:// (or loopback http://127.0.0.1 / http://localhost for local testing)".to_string())
}

pub fn url_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn sanitize_redirect_after(raw: &str) -> String {
    let trimmed = raw.trim();
    // "/phone" and "/phone?..." are plain same-origin paths, so this one rule covers them.
    if trimmed.starts_with('/') && !trimmed.starts_with("//") && !trimmed.contains('\\') {
        trimmed.to_string()
    } else {
        "/".to_string()
    }
}

fn oidc_http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(15))
        .timeout_write(std::time::Duration::from_secs(10))
        .build()
}

fn fetch_oidc_discovery(issuer: &str) -> Result<OidcDiscovery, String> {
    validate_oidc_issuer_url(issuer)?;
    let base = issuer.trim().trim_end_matches('/');
    let url = format!("{base}/.well-known/openid-configuration");
    let resp = oidc_http_agent()
        .get(&url)
        .call()
        .map_err(|e| format!("OIDC discovery failed for {url}: {e}"))?;
    let body = resp
        .into_string()
        .map_err(|e| format!("OIDC discovery read error: {e}"))?;
    let v: Value =
        serde_json::from_str(&body).map_err(|e| format!("OIDC discovery invalid JSON: {e}"))?;
    let discovered_issuer = v["issuer"]
        .as_str()
        .unwrap_or("")
        .trim()
        .trim_end_matches('/')
        .to_string();
    if discovered_issuer != base {
        return Err(format!(
            "OIDC discovery issuer mismatch: expected '{base}', got '{discovered_issuer}'"
        ));
    }
    let authorization_endpoint = v["authorization_endpoint"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "OIDC discovery missing authorization_endpoint".to_string())?
        .to_string();
    let token_endpoint = v["token_endpoint"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "OIDC discovery missing token_endpoint".to_string())?
        .to_string();
    let userinfo_endpoint = v["userinfo_endpoint"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    Ok(OidcDiscovery {
        issuer: discovered_issuer,
        authorization_endpoint,
        token_endpoint,
        userinfo_endpoint,
    })
}

fn decode_jwt_payload(jwt: &str) -> Result<Value, String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err("malformed JWT id_token (expected 3 dot-separated segments)".to_string());
    }
    let payload_bytes = tripwire::base64_decode(parts[1])
        .ok_or_else(|| "invalid base64url in JWT id_token payload".to_string())?;
    serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("invalid JSON in JWT id_token payload: {e}"))
}

fn jwt_aud_matches(aud: &Value, client_id: &str) -> bool {
    if let Some(s) = aud.as_str() {
        return s == client_id;
    }
    if let Some(arr) = aud.as_array() {
        return arr.iter().any(|item| item.as_str() == Some(client_id));
    }
    false
}

// ---------------------------------------------------------------------------
// AuthStore (thread-safe persisted manager)
// ---------------------------------------------------------------------------

struct AuthInner {
    users: Vec<AuthUser>,
    sessions: HashMap<String, AuthSession>,
    oidc: OidcConfig,
}

pub struct AuthStore {
    root_data_dir: PathBuf,
    inner: RwLock<AuthInner>,
    pending_oidc: Mutex<HashMap<String, OidcPendingFlow>>,
    failed_logins: Mutex<HashMap<String, Vec<u64>>>,
    auth_required_env: bool,
    session_ttl_ms: u64,
}

fn role_from_users(users: &[AuthUser], auth_required_env: bool, person_id: &str) -> Role {
    if users.is_empty() {
        if auth_required_env {
            return Role::Member;
        }
        return Role::Admin;
    }
    users
        .iter()
        .find(|u| !u.disabled && u.person_id == person_id)
        .map(|u| u.role)
        .unwrap_or(Role::Member)
}

pub fn resolve_person_role(root_data_dir: &Path, person_id: &str) -> Role {
    let users = AuthStore::load_users_file(&AuthStore::users_path(root_data_dir));
    role_from_users(&users, env_truthy("ERGON_AUTH_REQUIRED"), person_id)
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn session_ttl_ms_from_env() -> u64 {
    let secs = std::env::var("ERGON_SESSION_TTL_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SESSION_TTL_SECS)
        .clamp(MIN_SESSION_TTL_SECS, MAX_SESSION_TTL_SECS);
    secs * 1000
}

impl AuthStore {
    pub fn users_path(root_data_dir: &Path) -> PathBuf {
        root_data_dir.join("auth_users.json")
    }

    pub fn sessions_path(root_data_dir: &Path) -> PathBuf {
        root_data_dir.join("auth_sessions.json")
    }

    pub fn oidc_path(root_data_dir: &Path) -> PathBuf {
        root_data_dir.join("oidc_config.json")
    }

    pub fn load(root_data_dir: &Path) -> Self {
        let now = now_millis();
        let users = Self::load_users_file(&Self::users_path(root_data_dir));
        let mut sessions = Self::load_sessions_file(&Self::sessions_path(root_data_dir));
        sessions.retain(|_, s| s.expires_at_ms > now);

        let mut oidc = Self::load_oidc_file(&Self::oidc_path(root_data_dir));
        oidc.apply_env_overrides();

        Self {
            root_data_dir: root_data_dir.to_path_buf(),
            inner: RwLock::new(AuthInner {
                users,
                sessions,
                oidc,
            }),
            pending_oidc: Mutex::new(HashMap::new()),
            failed_logins: Mutex::new(HashMap::new()),
            auth_required_env: env_truthy("ERGON_AUTH_REQUIRED"),
            session_ttl_ms: session_ttl_ms_from_env(),
        }
    }

    fn load_users_file(path: &Path) -> Vec<AuthUser> {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            let _ = crate::backup_corrupt(path);
            return Vec::new();
        };
        v["users"]
            .as_array()
            .map(|arr| arr.iter().filter_map(AuthUser::from_storage_json).collect())
            .unwrap_or_default()
    }

    fn load_sessions_file(path: &Path) -> HashMap<String, AuthSession> {
        let Ok(text) = std::fs::read_to_string(path) else {
            return HashMap::new();
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            return HashMap::new();
        };
        let mut out = HashMap::new();
        if let Some(arr) = v["sessions"].as_array() {
            for item in arr {
                if let Some(s) = AuthSession::from_storage_json(item) {
                    out.insert(s.token_hash.clone(), s);
                }
            }
        }
        out
    }

    fn load_oidc_file(path: &Path) -> OidcConfig {
        let Ok(text) = std::fs::read_to_string(path) else {
            return OidcConfig::default();
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            return OidcConfig::default();
        };
        OidcConfig::from_storage_json(&v)
    }

    fn persist_users_locked(&self, inner: &AuthInner) -> Result<(), String> {
        let payload = json!({
            "version": 1,
            "users": inner.users.iter().map(AuthUser::to_storage_json).collect::<Vec<_>>(),
        });
        let bytes = serde_json::to_vec_pretty(&payload)
            .map_err(|e| format!("serialize auth users: {e}"))?;
        atomic_write_bytes_with_mode(&Self::users_path(&self.root_data_dir), &bytes, Some(0o600))
            .map_err(|e| format!("save auth_users.json: {e}"))
    }

    fn persist_sessions_locked(&self, inner: &AuthInner) -> Result<(), String> {
        let payload = json!({
            "version": 1,
            "sessions": inner.sessions.values().map(AuthSession::to_storage_json).collect::<Vec<_>>(),
        });
        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| format!("serialize auth sessions: {e}"))?;
        atomic_write_bytes_with_mode(
            &Self::sessions_path(&self.root_data_dir),
            &bytes,
            Some(0o600),
        )
        .map_err(|e| format!("save auth_sessions.json: {e}"))
    }

    fn persist_oidc_locked(&self, inner: &AuthInner) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&inner.oidc.to_storage_json())
            .map_err(|e| format!("serialize oidc config: {e}"))?;
        atomic_write_bytes_with_mode(&Self::oidc_path(&self.root_data_dir), &bytes, Some(0o600))
            .map_err(|e| format!("save oidc_config.json: {e}"))
    }

    /// Bootstrap initial admin account from `ERGON_ADMIN_USER` + `ERGON_ADMIN_PASSWORD`
    /// when no accounts exist yet.
    pub fn bootstrap_admin_from_env(
        &self,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<Option<AuthUser>, String> {
        if self.has_users() {
            return Ok(None);
        }
        let user = std::env::var("ERGON_ADMIN_USER")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let pass = std::env::var("ERGON_ADMIN_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty());
        let (Some(username), Some(password)) = (user, pass) else {
            return Ok(None);
        };
        let display_name = std::env::var("ERGON_ADMIN_NAME")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| username.clone());
        let email = std::env::var("ERGON_ADMIN_EMAIL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let created = self.setup_initial_admin(
            &username,
            &password,
            Some(&display_name),
            email.as_deref(),
            people,
            root_cfg,
        )?;
        Ok(Some(created))
    }

    pub fn has_users(&self) -> bool {
        !self.inner.read().unwrap().users.is_empty()
    }

    pub fn user_count(&self) -> usize {
        self.inner.read().unwrap().users.len()
    }

    pub fn auth_required_by_env(&self) -> bool {
        self.auth_required_env
    }

    /// Auth is enforced whenever ANY account exists, OR `ERGON_AUTH_REQUIRED=1`,
    /// OR TLS is active, OR the server binds a non-loopback address.
    pub fn is_enforced(&self, tls_enabled: bool, non_loopback_bind: bool) -> bool {
        self.has_users() || self.auth_required_env || tls_enabled || non_loopback_bind
    }

    pub fn oidc_config_public(&self) -> Value {
        self.inner.read().unwrap().oidc.to_public_json()
    }

    pub fn oidc_enabled(&self) -> bool {
        self.inner.read().unwrap().oidc.is_configured()
    }

    pub fn update_oidc_config(&self, body: &Value) -> Result<Value, (u16, String)> {
        if !body.is_object() {
            return Err((400, "request body must be a JSON object".to_string()));
        }
        let mut inner = self.inner.write().unwrap();
        let mut next = inner.oidc.clone();
        if let Some(b) = body["enabled"].as_bool() {
            next.enabled = b;
        }
        if let Some(s) = body["issuer"].as_str() {
            let trimmed = s.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                validate_oidc_issuer_url(&trimmed).map_err(|e| (400, e))?;
            }
            next.issuer = trimmed;
        }
        if let Some(s) = body["client_id"].as_str() {
            next.client_id = s.trim().to_string();
        }
        if body["clear_client_secret"].as_bool().unwrap_or(false) {
            next.client_secret = None;
        } else if let Some(s) = body["client_secret"].as_str() {
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                next.client_secret = Some(trimmed);
            }
        }
        if let Some(s) = body["redirect_uri"].as_str() {
            next.redirect_uri = s.trim().to_string();
        }
        if let Some(s) = body["scopes"].as_str() {
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                next.scopes = trimmed;
            }
        }
        if let Some(s) = body["default_role"].as_str() {
            next.default_role = Role::parse_strict(s).ok_or_else(|| {
                (
                    400,
                    format!("default_role must be one of: {}", Role::ALL.join(", ")),
                )
            })?;
        }
        if let Some(s) = body["role_claim"].as_str() {
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                next.role_claim = trimmed;
            }
        }
        if let Some(b) = body["auto_provision"].as_bool() {
            next.auto_provision = b;
        }
        if next.enabled && (next.issuer.is_empty() || next.client_id.is_empty()) {
            return Err((
                400,
                "OIDC requires both issuer and client_id when enabled".to_string(),
            ));
        }
        if !next.issuer.is_empty() {
            validate_oidc_issuer_url(&next.issuer).map_err(|e| (400, e))?;
        }
        inner.oidc = next;
        self.persist_oidc_locked(&inner).map_err(|e| (500, e))?;
        Ok(inner.oidc.to_public_json())
    }

    fn ensure_person_for_user(
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
        display_name: &str,
        role: Role,
        preferred_person_id: Option<&str>,
    ) -> String {
        let mut ps = people.write().unwrap();
        if let Some(pid) = preferred_person_id {
            if let Some(existing) = ps.people.iter_mut().find(|p| p.id == pid) {
                existing.name = display_name.to_string();
                existing.role = role.as_str().to_string();
                let id = existing.id.clone();
                ps.save(root_cfg);
                return id;
            }
        }
        let created = ps.add(display_name, role.as_str());
        ps.save(root_cfg);
        created.id
    }

    /// Create the very first admin account. Fails with 403 if any user already exists.
    pub fn setup_initial_admin(
        &self,
        username: &str,
        password: &str,
        display_name: Option<&str>,
        email: Option<&str>,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<AuthUser, String> {
        let norm_user = normalize_username(username)?;
        let pw_hash = hash_password(password)?;
        let mut inner = self.inner.write().unwrap();
        if !inner.users.is_empty() {
            return Err("initial admin setup already completed".to_string());
        }
        let name = display_name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&norm_user)
            .to_string();
        let clean_email = email
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let person_id = Self::ensure_person_for_user(people, root_cfg, &name, Role::Admin, None);
        let now = now_millis();
        let rand_id = base64url_encode_nopad(&secure_random_bytes(9)?);
        let user = AuthUser {
            id: format!("usr_{rand_id}"),
            username: norm_user,
            email: clean_email,
            display_name: name,
            role: Role::Admin,
            person_id,
            password_hash: Some(pw_hash),
            oidc_sub: None,
            disabled: false,
            created_ms: now,
            updated_ms: now,
            last_login_ms: None,
        };
        inner.users.push(user.clone());
        self.persist_users_locked(&inner)?;
        Ok(user)
    }

    /// Admin/CLI creates a new local user account with its own isolated Person workspace.
    #[allow(clippy::too_many_arguments)]
    pub fn create_user(
        &self,
        username: &str,
        password: &str,
        role_str: &str,
        display_name: Option<&str>,
        email: Option<&str>,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<AuthUser, (u16, String)> {
        let norm_user = normalize_username(username).map_err(|e| (400, e))?;
        let role = Role::parse_strict(role_str).ok_or_else(|| {
            (
                400,
                format!("role must be one of: {}", Role::ALL.join(", ")),
            )
        })?;
        let pw_hash = hash_password(password).map_err(|e| (400, e))?;
        let clean_email = email
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let name = display_name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&norm_user)
            .to_string();

        let mut inner = self.inner.write().unwrap();
        if inner.users.iter().any(|u| u.username == norm_user) {
            return Err((409, format!("username '{norm_user}' already exists")));
        }
        if let Some(ref em) = clean_email {
            if inner
                .users
                .iter()
                .any(|u| u.email.as_deref() == Some(em.as_str()))
            {
                return Err((409, format!("email '{em}' is already in use")));
            }
        }
        let person_id = Self::ensure_person_for_user(people, root_cfg, &name, role, None);
        let now = now_millis();
        let rand_id = base64url_encode_nopad(&secure_random_bytes(9).map_err(|e| (500, e))?);
        let user = AuthUser {
            id: format!("usr_{rand_id}"),
            username: norm_user,
            email: clean_email,
            display_name: name,
            role,
            person_id,
            password_hash: Some(pw_hash),
            oidc_sub: None,
            disabled: false,
            created_ms: now,
            updated_ms: now,
            last_login_ms: None,
        };
        inner.users.push(user.clone());
        self.persist_users_locked(&inner).map_err(|e| (500, e))?;
        Ok(user)
    }

    pub fn list_users(&self) -> Vec<AuthUser> {
        self.inner.read().unwrap().users.clone()
    }

    pub fn list_users_public(&self) -> Vec<Value> {
        self.inner
            .read()
            .unwrap()
            .users
            .iter()
            .map(AuthUser::to_public_json)
            .collect()
    }

    pub fn find_by_id(&self, id: &str) -> Option<AuthUser> {
        self.inner
            .read()
            .unwrap()
            .users
            .iter()
            .find(|u| u.id == id)
            .cloned()
    }

    pub fn find_by_username(&self, username: &str) -> Option<AuthUser> {
        let low = username.trim().to_ascii_lowercase();
        self.inner
            .read()
            .unwrap()
            .users
            .iter()
            .find(|u| u.username == low)
            .cloned()
    }

    pub fn role_for_person(&self, person_id: &str) -> Role {
        let inner = self.inner.read().unwrap();
        role_from_users(&inner.users, self.auth_required_env, person_id)
    }

    /// Update an existing user (role, display_name, email, disabled, or password reset).
    pub fn update_user(
        &self,
        id_or_username: &str,
        patch: &Value,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<AuthUser, (u16, String)> {
        if !patch.is_object() {
            return Err((400, "request body must be a JSON object".to_string()));
        }
        let target_key = id_or_username.trim();
        let mut inner = self.inner.write().unwrap();
        let Some(idx) = inner
            .users
            .iter()
            .position(|u| u.id == target_key || u.username.eq_ignore_ascii_case(target_key))
        else {
            return Err((404, "user not found".to_string()));
        };

        let next_role = if let Some(r_str) = patch["role"].as_str() {
            Some(Role::parse_strict(r_str).ok_or_else(|| {
                (
                    400,
                    format!("role must be one of: {}", Role::ALL.join(", ")),
                )
            })?)
        } else {
            None
        };
        let next_disabled = patch["disabled"].as_bool();
        let active_admins = inner
            .users
            .iter()
            .filter(|u| u.role.is_admin() && !u.disabled)
            .count();
        let cur = &inner.users[idx];
        let would_lose_admin = cur.role.is_admin()
            && !cur.disabled
            && (next_role.is_some_and(|r| !r.is_admin()) || next_disabled == Some(true));
        if would_lose_admin && active_admins <= 1 {
            return Err((
                400,
                "cannot demote or disable the last active admin account".to_string(),
            ));
        }

        let next_pw_hash = if let Some(pw) = patch["password"].as_str() {
            Some(hash_password(pw).map_err(|e| (400, e))?)
        } else {
            None
        };

        if let Some(em_raw) = patch["email"].as_str() {
            let em = em_raw.trim().to_ascii_lowercase();
            if !em.is_empty()
                && inner
                    .users
                    .iter()
                    .enumerate()
                    .any(|(i, u)| i != idx && u.email.as_deref() == Some(em.as_str()))
            {
                return Err((409, format!("email '{em}' is already in use")));
            }
        }

        let mut revoke_sessions = false;
        let user_id = inner.users[idx].id.clone();
        let person_id = inner.users[idx].person_id.clone();
        {
            let u = &mut inner.users[idx];
            if let Some(dn) = patch["display_name"].as_str() {
                let t = dn.trim();
                if !t.is_empty() {
                    u.display_name = t.to_string();
                }
            }
            if let Some(em_raw) = patch["email"].as_str() {
                let em = em_raw.trim().to_ascii_lowercase();
                u.email = if em.is_empty() { None } else { Some(em) };
            }
            if let Some(r) = next_role {
                if u.role != r {
                    u.role = r;
                    revoke_sessions = true;
                }
            }
            if let Some(d) = next_disabled {
                if u.disabled != d {
                    u.disabled = d;
                    if d {
                        revoke_sessions = true;
                    }
                }
            }
            if let Some(h) = next_pw_hash {
                u.password_hash = Some(h);
                revoke_sessions = true;
            }
            u.updated_ms = now_millis();
        }
        let updated = inner.users[idx].clone();
        if revoke_sessions {
            inner.sessions.retain(|_, s| s.user_id != user_id);
            let _ = self.persist_sessions_locked(&inner);
        }
        self.persist_users_locked(&inner).map_err(|e| (500, e))?;
        drop(inner);

        // Keep linked Person name/role synchronized.
        let _ = Self::ensure_person_for_user(
            people,
            root_cfg,
            &updated.display_name,
            updated.role,
            Some(&person_id),
        );
        Ok(updated)
    }

    pub fn set_password_cli(&self, username: &str, new_password: &str) -> Result<AuthUser, String> {
        let pw_hash = hash_password(new_password)?;
        let low = username.trim().to_ascii_lowercase();
        let mut inner = self.inner.write().unwrap();
        let Some(u) = inner.users.iter_mut().find(|u| u.username == low) else {
            return Err(format!("user '{low}' not found"));
        };
        let uid = u.id.clone();
        u.password_hash = Some(pw_hash);
        u.updated_ms = now_millis();
        let updated = u.clone();
        inner.sessions.retain(|_, s| s.user_id != uid);
        self.persist_users_locked(&inner)?;
        let _ = self.persist_sessions_locked(&inner);
        Ok(updated)
    }

    pub fn change_own_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
        keep_session_hash: Option<&str>,
    ) -> Result<(), (u16, String)> {
        let new_hash = hash_password(new_password).map_err(|e| (400, e))?;
        let mut inner = self.inner.write().unwrap();
        let Some(u) = inner.users.iter_mut().find(|u| u.id == user_id) else {
            return Err((404, "user not found".to_string()));
        };
        if let Some(ref existing) = u.password_hash {
            if !verify_password(current_password, existing) {
                return Err((401, "current password is incorrect".to_string()));
            }
        }
        u.password_hash = Some(new_hash);
        u.updated_ms = now_millis();
        inner.sessions.retain(|h, s| {
            s.user_id != user_id || keep_session_hash.is_some_and(|k| k == h)
        });
        self.persist_users_locked(&inner).map_err(|e| (500, e))?;
        let _ = self.persist_sessions_locked(&inner);
        Ok(())
    }

    pub fn delete_user(&self, id: &str) -> Result<AuthUser, (u16, String)> {
        let mut inner = self.inner.write().unwrap();
        let Some(idx) = inner.users.iter().position(|u| u.id == id) else {
            return Err((404, "user not found".to_string()));
        };
        let active_admins = inner
            .users
            .iter()
            .filter(|u| u.role.is_admin() && !u.disabled)
            .count();
        if inner.users[idx].role.is_admin() && !inner.users[idx].disabled && active_admins <= 1 {
            return Err((400, "cannot delete the last active admin account".to_string()));
        }
        let removed = inner.users.remove(idx);
        inner.sessions.retain(|_, s| s.user_id != removed.id);
        self.persist_users_locked(&inner).map_err(|e| (500, e))?;
        let _ = self.persist_sessions_locked(&inner);
        Ok(removed)
    }

    // -----------------------------------------------------------------------
    // Local login & rate limiting
    // -----------------------------------------------------------------------

    fn check_rate_limit(&self, key: &str) -> Result<(), (u16, String)> {
        let now = now_millis();
        let mut guard = self.failed_logins.lock().unwrap();
        let list = guard.entry(key.to_string()).or_default();
        list.retain(|&ts| now.saturating_sub(ts) < LOGIN_FAIL_WINDOW_MS);
        if list.len() >= LOGIN_FAIL_LIMIT {
            return Err((
                429,
                "too many failed sign-in attempts; wait a few minutes and try again".to_string(),
            ));
        }
        Ok(())
    }

    fn note_login_failure(&self, key: &str) {
        let now = now_millis();
        let mut guard = self.failed_logins.lock().unwrap();
        let list = guard.entry(key.to_string()).or_default();
        list.retain(|&ts| now.saturating_sub(ts) < LOGIN_FAIL_WINDOW_MS);
        list.push(now);
    }

    fn clear_login_failures(&self, key: &str) {
        self.failed_logins.lock().unwrap().remove(key);
    }

    /// Authenticate username/email + password and issue a persisted session token.
    pub fn login_local(
        &self,
        username_or_email: &str,
        password: &str,
        client_key: &str,
    ) -> Result<(AuthUser, String, u64), (u16, String)> {
        let ident = username_or_email.trim().to_ascii_lowercase();
        if ident.is_empty() || password.is_empty() {
            return Err((400, "username and password are required".to_string()));
        }
        let rl_key = format!("{client_key}|{ident}");
        self.check_rate_limit(&rl_key)?;

        let candidate = {
            let inner = self.inner.read().unwrap();
            inner
                .users
                .iter()
                .find(|u| u.username == ident || u.email.as_deref() == Some(ident.as_str()))
                .cloned()
        };

        let Some(user) = candidate else {
            // Perform a dummy PBKDF2 hash to equalize timing against username enumeration.
            let _ = pbkdf2_hmac_sha256(password.as_bytes(), b"ergon-dummy-salt", PBKDF2_ITERATIONS);
            self.note_login_failure(&rl_key);
            return Err((401, "invalid username or password".to_string()));
        };
        if user.disabled {
            self.note_login_failure(&rl_key);
            return Err((403, "this account has been disabled by an administrator".to_string()));
        }
        let Some(ref stored_hash) = user.password_hash else {
            self.note_login_failure(&rl_key);
            return Err((
                401,
                "this account uses SSO sign-in (no local password set)".to_string(),
            ));
        };
        if !verify_password(password, stored_hash) {
            self.note_login_failure(&rl_key);
            return Err((401, "invalid username or password".to_string()));
        }

        self.clear_login_failures(&rl_key);
        let (raw_token, expires_at_ms) = self
            .create_session(&user, "password")
            .map_err(|e| (500, e))?;
        Ok((user, raw_token, expires_at_ms))
    }

    pub fn create_session(&self, user: &AuthUser, auth_method: &str) -> Result<(String, u64), String> {
        let raw_token = new_opaque_token()?;
        let hash = token_sha256(&raw_token);
        let now = now_millis();
        let expires_at_ms = now.saturating_add(self.session_ttl_ms);

        let mut inner = self.inner.write().unwrap();
        // Prune expired sessions and cap total sessions safely.
        inner.sessions.retain(|_, s| s.expires_at_ms > now);
        if inner.sessions.len() >= 10_000 {
            let mut oldest: Vec<(String, u64)> = inner
                .sessions
                .iter()
                .map(|(k, v)| (k.clone(), v.last_seen_ms))
                .collect();
            oldest.sort_by_key(|(_, ts)| *ts);
            for (k, _) in oldest.into_iter().take(1_000) {
                inner.sessions.remove(&k);
            }
        }
        if let Some(u) = inner.users.iter_mut().find(|u| u.id == user.id) {
            u.last_login_ms = Some(now);
            let _ = self.persist_users_locked(&inner);
        }
        inner.sessions.insert(
            hash.clone(),
            AuthSession {
                token_hash: hash,
                user_id: user.id.clone(),
                active_person_id: user.person_id.clone(),
                auth_method: auth_method.to_string(),
                created_ms: now,
                last_seen_ms: now,
                expires_at_ms,
            },
        );
        self.persist_sessions_locked(&inner)?;
        Ok((raw_token, expires_at_ms))
    }

    /// Resolve an opaque session token (from cookie or `Authorization: Bearer`) to an `AuthPrincipal`.
    pub fn validate_session_token(&self, raw_token: &str) -> Option<AuthPrincipal> {
        let trimmed = raw_token.trim();
        if trimmed.is_empty() {
            return None;
        }
        let hash = token_sha256(trimmed);
        let now = now_millis();
        let inner = self.inner.read().unwrap();
        let sess = inner.sessions.get(&hash)?;
        if sess.expires_at_ms <= now {
            return None;
        }
        let user = inner.users.iter().find(|u| u.id == sess.user_id)?;
        if user.disabled {
            return None;
        }
        // Non-admins are ALWAYS strictly pinned to their own home person_id.
        let active_person_id = if user.role.is_admin() {
            sess.active_person_id.clone()
        } else {
            user.person_id.clone()
        };
        Some(AuthPrincipal {
            user_id: user.id.clone(),
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            email: user.email.clone(),
            role: user.role,
            home_person_id: user.person_id.clone(),
            active_person_id,
            auth_method: sess.auth_method.clone(),
            is_break_glass: false,
            session_token_hash: Some(hash),
            api_key_scopes: None,
        })
    }

    /// Admin-only workspace switch for a specific authenticated session (never mutates global active).
    pub fn switch_admin_session_person(&self, token_hash: &str, target_person_id: &str) -> Result<(), String> {
        let mut inner = self.inner.write().unwrap();
        let Some(sess) = inner.sessions.get_mut(token_hash) else {
            return Err("session not found".to_string());
        };
        sess.active_person_id = target_person_id.to_string();
        sess.last_seen_ms = now_millis();
        self.persist_sessions_locked(&inner)
    }

    pub fn revoke_session(&self, raw_token: &str) -> bool {
        let hash = token_sha256(raw_token.trim());
        let mut inner = self.inner.write().unwrap();
        let removed = inner.sessions.remove(&hash).is_some();
        if removed {
            let _ = self.persist_sessions_locked(&inner);
        }
        removed
    }

    // -----------------------------------------------------------------------
    // OIDC Authorization Code + PKCE Flow
    // -----------------------------------------------------------------------

    /// Initiate OIDC Authorization Code + PKCE (`S256`) login. Returns the redirect URL.
    pub fn start_oidc_login(
        &self,
        redirect_after: &str,
        request_origin: &str,
    ) -> Result<String, (u16, String)> {
        let cfg = self.inner.read().unwrap().oidc.clone();
        if !cfg.is_configured() {
            return Err((400, "OIDC sign-in is not configured on this server".to_string()));
        }
        let discovery = fetch_oidc_discovery(&cfg.issuer).map_err(|e| (502, e))?;
        let redirect_uri = if cfg.redirect_uri.is_empty() {
            format!("{}/api/auth/oidc/callback", request_origin.trim_end_matches('/'))
        } else {
            cfg.redirect_uri.clone()
        };

        let state = base64url_encode_nopad(&secure_random_bytes(24).map_err(|e| (500, e))?);
        let nonce = base64url_encode_nopad(&secure_random_bytes(24).map_err(|e| (500, e))?);
        let code_verifier = base64url_encode_nopad(&secure_random_bytes(32).map_err(|e| (500, e))?);
        let challenge_digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = base64url_encode_nopad(&challenge_digest);

        let now = now_millis();
        {
            let mut pending = self.pending_oidc.lock().unwrap();
            pending.retain(|_, f| now.saturating_sub(f.created_ms) < OIDC_PENDING_TTL_MS);
            if pending.len() >= MAX_PENDING_OIDC {
                pending.clear();
            }
            pending.insert(
                state.clone(),
                OidcPendingFlow {
                    nonce: nonce.clone(),
                    code_verifier,
                    redirect_uri: redirect_uri.clone(),
                    redirect_after: sanitize_redirect_after(redirect_after),
                    created_ms: now,
                },
            );
        }

        let sep = if discovery.authorization_endpoint.contains('?') {
            '&'
        } else {
            '?'
        };
        let auth_url = format!(
            "{}{sep}response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&nonce={}&code_challenge={}&code_challenge_method=S256",
            discovery.authorization_endpoint,
            url_encode_component(&cfg.client_id),
            url_encode_component(&redirect_uri),
            url_encode_component(&cfg.scopes),
            url_encode_component(&state),
            url_encode_component(&nonce),
            url_encode_component(&code_challenge),
        );
        Ok(auth_url)
    }

    /// Complete OIDC Authorization Code + PKCE callback, map claims to a local
    /// account/role, and mint an opaque Ergon session token.
    pub fn complete_oidc_callback(
        &self,
        code: &str,
        state_param: &str,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<(AuthUser, String, u64, String), (u16, String)> {
        let code = code.trim();
        let state_param = state_param.trim();
        if code.is_empty() || state_param.is_empty() {
            return Err((400, "missing OIDC code or state parameter".to_string()));
        }

        let now = now_millis();
        let pending = {
            let mut map = self.pending_oidc.lock().unwrap();
            map.retain(|_, f| now.saturating_sub(f.created_ms) < OIDC_PENDING_TTL_MS);
            map.remove(state_param)
        }
        .ok_or_else(|| (400, "invalid or expired OIDC state parameter".to_string()))?;

        let cfg = self.inner.read().unwrap().oidc.clone();
        if !cfg.is_configured() {
            return Err((400, "OIDC sign-in is not configured on this server".to_string()));
        }
        let discovery = fetch_oidc_discovery(&cfg.issuer).map_err(|e| (502, e))?;

        let mut form_pairs: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &pending.redirect_uri),
            ("client_id", &cfg.client_id),
            ("code_verifier", &pending.code_verifier),
        ];
        if let Some(ref secret) = cfg.client_secret {
            form_pairs.push(("client_secret", secret.as_str()));
        }

        let token_resp = oidc_http_agent()
            .post(&discovery.token_endpoint)
            .set("Accept", "application/json")
            .send_form(&form_pairs)
            .map_err(|e| (502, format!("OIDC token exchange failed: {e}")))?;
        let token_raw = token_resp
            .into_string()
            .map_err(|e| (502, format!("OIDC token response read error: {e}")))?;
        let token_json: Value = serde_json::from_str(&token_raw)
            .map_err(|e| (502, format!("OIDC token response invalid JSON: {e}")))?;

        let access_token = token_json["access_token"].as_str().map(str::trim);
        let id_token = token_json["id_token"].as_str().map(str::trim);

        let mut id_claims: Option<Value> = None;
        if let Some(jwt) = id_token.filter(|s| !s.is_empty()) {
            let claims = decode_jwt_payload(jwt).map_err(|e| (401, e))?;
            let iss = claims["iss"]
                .as_str()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/');
            if iss != discovery.issuer {
                return Err((401, format!("OIDC id_token issuer mismatch: '{iss}'")));
            }
            if !jwt_aud_matches(&claims["aud"], &cfg.client_id) {
                return Err((401, "OIDC id_token audience does not match client_id".to_string()));
            }
            let now_secs = now / 1000;
            if let Some(exp) = claims["exp"].as_u64() {
                if exp.saturating_add(60) < now_secs {
                    return Err((401, "OIDC id_token has expired".to_string()));
                }
            }
            let token_nonce = claims["nonce"].as_str().unwrap_or("");
            if !tripwire::ct_eq(token_nonce.as_bytes(), pending.nonce.as_bytes()) {
                return Err((401, "OIDC id_token nonce mismatch".to_string()));
            }
            id_claims = Some(claims);
        }

        let mut userinfo_claims: Option<Value> = None;
        if let (Some(ep), Some(at)) = (
            discovery.userinfo_endpoint.as_deref(),
            access_token.filter(|s| !s.is_empty()),
        ) {
            let ui_resp = oidc_http_agent()
                .get(ep)
                .set("Authorization", &format!("Bearer {at}"))
                .set("Accept", "application/json")
                .call()
                .map_err(|e| (502, format!("OIDC userinfo request failed: {e}")))?;
            let ui_raw = ui_resp
                .into_string()
                .map_err(|e| (502, format!("OIDC userinfo read error: {e}")))?;
            let ui_json: Value = serde_json::from_str(&ui_raw)
                .map_err(|e| (502, format!("OIDC userinfo invalid JSON: {e}")))?;
            userinfo_claims = Some(ui_json);
        }

        if id_claims.is_none() && userinfo_claims.is_none() {
            return Err((
                401,
                "OIDC provider returned neither id_token nor userinfo claims".to_string(),
            ));
        }

        let id_sub = id_claims
            .as_ref()
            .and_then(|c| c["sub"].as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let ui_sub = userinfo_claims
            .as_ref()
            .and_then(|c| c["sub"].as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        if let (Some(a), Some(b)) = (id_sub, ui_sub) {
            if a != b {
                return Err((
                    401,
                    "OIDC userinfo 'sub' does not match id_token 'sub'".to_string(),
                ));
            }
        }
        let sub = id_sub
            .or(ui_sub)
            .ok_or_else(|| (401, "OIDC response missing 'sub' claim".to_string()))?
            .to_string();

        let claim_str = |key: &str| -> Option<String> {
            userinfo_claims
                .as_ref()
                .and_then(|c| c[key].as_str())
                .or_else(|| id_claims.as_ref().and_then(|c| c[key].as_str()))
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        };

        let email = claim_str("email").map(|s| s.to_ascii_lowercase());
        let display_name = claim_str("name")
            .or_else(|| claim_str("preferred_username"))
            .or_else(|| email.clone())
            .unwrap_or_else(|| sub.clone());
        let raw_role = claim_str(&cfg.role_claim);
        let mapped_role = raw_role
            .as_deref()
            .and_then(Role::parse_strict)
            .unwrap_or(cfg.default_role);

        let user = self.upsert_oidc_user(
            &sub,
            email.as_deref(),
            &display_name,
            mapped_role,
            cfg.auto_provision,
            people,
            root_cfg,
        )?;
        let (raw_token, expires_at_ms) = self
            .create_session(&user, "oidc")
            .map_err(|e| (500, e))?;
        Ok((user, raw_token, expires_at_ms, pending.redirect_after))
    }

    #[allow(clippy::too_many_arguments)]
    fn upsert_oidc_user(
        &self,
        sub: &str,
        email: Option<&str>,
        display_name: &str,
        mapped_role: Role,
        auto_provision: bool,
        people: &RwLock<PeopleState>,
        root_cfg: &Config,
    ) -> Result<AuthUser, (u16, String)> {
        let now = now_millis();
        let mut inner = self.inner.write().unwrap();

        // Match existing user by oidc_sub first, then by verified email.
        let existing_idx = inner
            .users
            .iter()
            .position(|u| u.oidc_sub.as_deref() == Some(sub))
            .or_else(|| {
                email.and_then(|em| {
                    inner
                        .users
                        .iter()
                        .position(|u| u.email.as_deref() == Some(em))
                })
            });

        if let Some(idx) = existing_idx {
            if inner.users[idx].disabled {
                return Err((403, "this account has been disabled by an administrator".to_string()));
            }
            let u = &mut inner.users[idx];
            u.oidc_sub = Some(sub.to_string());
            if u.email.is_none() {
                u.email = email.map(String::from);
            }
            u.updated_ms = now;
            let cloned = u.clone();
            self.persist_users_locked(&inner).map_err(|e| (500, e))?;
            return Ok(cloned);
        }

        if !auto_provision {
            return Err((
                403,
                "no matching local account exists and OIDC auto-provisioning is disabled"
                    .to_string(),
            ));
        }

        // Mint a clean unique username from email/display_name/sub.
        let base_raw = email
            .and_then(|e| e.split('@').next())
            .unwrap_or(display_name);
        let mut base = crate::slugify(base_raw);
        if base.len() < 2 {
            base = format!("user-{base}");
        }
        let mut candidate = base.clone();
        let mut suffix = 2usize;
        while inner.users.iter().any(|u| u.username == candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        let person_id = Self::ensure_person_for_user(people, root_cfg, display_name, mapped_role, None);
        let rand_id = base64url_encode_nopad(&secure_random_bytes(9).map_err(|e| (500, e))?);
        let user = AuthUser {
            id: format!("usr_{rand_id}"),
            username: candidate,
            email: email.map(String::from),
            display_name: display_name.to_string(),
            role: mapped_role,
            person_id,
            password_hash: None,
            oidc_sub: Some(sub.to_string()),
            disabled: false,
            created_ms: now,
            updated_ms: now,
            last_login_ms: Some(now),
        };
        inner.users.push(user.clone());
        self.persist_users_locked(&inner).map_err(|e| (500, e))?;
        Ok(user)
    }
}

// ---------------------------------------------------------------------------
// Cookie / Header Helpers
// ---------------------------------------------------------------------------

pub fn session_cookie_header(token: &str, tls_enabled: bool, max_age_secs: u64) -> String {
    if tls_enabled {
        format!(
            "ergon_session={token}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={max_age_secs}"
        )
    } else {
        format!("ergon_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}")
    }
}

pub fn clear_session_cookie_header(tls_enabled: bool) -> String {
    if tls_enabled {
        "ergon_session=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0".to_string()
    } else {
        "ergon_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string()
    }
}

pub fn extract_bearer_token(authorization: Option<&str>) -> Option<&str> {
    let raw = authorization?.trim();
    let (scheme, token) = raw.split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") {
        let t = token.trim();
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

/// Audit break-glass `ERGON_GATE_TOKEN` usage into the tamper-evident tripwire log.
pub fn record_break_glass_audit(
    root_data_dir: &Path,
    method: &str,
    path: &str,
    ip: Option<std::net::IpAddr>,
) {
    let ip_s = ip
        .map(|i| i.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    tripwire::record(
        root_data_dir,
        &tripwire::Trip::new(
            "AU2-BREAKGLASS",
            tripwire::Severity::Info,
            "Admin break-glass gate token used",
            format!(
                "{ip_s} used ERGON_GATE_TOKEN break-glass admin access for {method} {}",
                crate::truncate(path, 80)
            ),
        )
        .action("allowed_break_glass")
        .source(json!({"kind": "ip", "value": ip_s}))
        .coalesce(format!("au2|{ip_s}|{method}|{path}")),
    );
}

// ---------------------------------------------------------------------------
// Scoped API Keys (<root_data_dir>/api_keys.json, 0600 POSIX)
// ---------------------------------------------------------------------------

pub const API_KEY_PREFIX: &str = "ergk_";
pub const VALID_API_KEY_SCOPES: [&str; 7] = [
    "read",
    "write",
    "webhooks",
    "mcp",
    "company_db",
    "tools",
    "admin",
];

static API_KEYS_LOCK: Mutex<()> = Mutex::new(());

fn with_api_keys_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = API_KEYS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

pub fn api_keys_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("api_keys.json")
}

#[derive(Clone, Debug)]
pub struct ApiKeyRecord {
    pub id: String,
    pub name: String,
    pub key_sha256: String,
    pub prefix: String,
    pub person_id: String,
    pub role: Role,
    pub scopes: Vec<String>,
    pub created_ms: u64,
    pub last_used_ms: Option<u64>,
    pub expires_at_ms: Option<u64>,
}

impl ApiKeyRecord {
    pub fn to_public_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "prefix": self.prefix,
            "person_id": self.person_id,
            "role": self.role.as_str(),
            "scopes": self.scopes,
            "created_ms": self.created_ms,
            "last_used_ms": self.last_used_ms,
            "expires_at_ms": self.expires_at_ms,
        })
    }

    fn to_storage_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "key_sha256": self.key_sha256,
            "prefix": self.prefix,
            "person_id": self.person_id,
            "role": self.role.as_str(),
            "scopes": self.scopes,
            "created_ms": self.created_ms,
            "last_used_ms": self.last_used_ms,
            "expires_at_ms": self.expires_at_ms,
        })
    }

    fn from_storage_json(v: &Value) -> Option<Self> {
        let id = v["id"].as_str()?.trim().to_string();
        let name = v["name"].as_str()?.trim().to_string();
        let key_sha256 = v["key_sha256"].as_str()?.trim().to_string();
        let prefix = v["prefix"].as_str().unwrap_or("").to_string();
        let person_id = v["person_id"].as_str().unwrap_or("default").trim().to_string();
        if id.is_empty() || name.is_empty() || key_sha256.len() != 64 {
            return None;
        }
        let role = Role::parse_lenient(v["role"].as_str().unwrap_or("member"));
        let scopes = v["scopes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            id,
            name,
            key_sha256,
            prefix,
            person_id,
            role,
            scopes,
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            last_used_ms: v["last_used_ms"].as_u64(),
            expires_at_ms: v["expires_at_ms"].as_u64(),
        })
    }
}

fn load_api_keys_unlocked(root_data_dir: &Path) -> Vec<ApiKeyRecord> {
    let path = api_keys_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v["keys"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(ApiKeyRecord::from_storage_json)
                .collect()
        })
        .unwrap_or_default()
}

fn save_api_keys_unlocked(root_data_dir: &Path, keys: &[ApiKeyRecord]) -> Result<(), String> {
    let payload = json!({
        "version": 1,
        "keys": keys.iter().map(ApiKeyRecord::to_storage_json).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize api_keys.json: {e}"))?;
    atomic_write_bytes_with_mode(&api_keys_path(root_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write api_keys.json: {e}"))
}

pub fn create_api_key(
    root_data_dir: &Path,
    name: &str,
    person_id: &str,
    role: Role,
    scopes: &[String],
    expires_at_ms: Option<u64>,
) -> Result<(ApiKeyRecord, String), (u16, String)> {
    let clean_name = name.trim();
    if clean_name.is_empty() || clean_name.len() > 80 {
        return Err((400, "name must be 1 to 80 characters".to_string()));
    }
    let clean_person = person_id.trim();
    if clean_person.is_empty() || !crate::valid_person_id(clean_person) {
        return Err((400, "invalid person_id".to_string()));
    }
    if scopes.is_empty() {
        return Err((400, "scopes array must not be empty".to_string()));
    }
    let mut norm_scopes = Vec::new();
    for raw in scopes {
        let s = raw.trim().to_ascii_lowercase();
        if !VALID_API_KEY_SCOPES.contains(&s.as_str()) {
            return Err((
                400,
                format!(
                    "unsupported API key scope '{raw}'; valid scopes: {}",
                    VALID_API_KEY_SCOPES.join(", ")
                ),
            ));
        }
        if !norm_scopes.contains(&s) {
            norm_scopes.push(s);
        }
    }

    let raw_secret = secure_random_bytes(32).map_err(|e| (500, e))?;
    let raw_key = format!("{API_KEY_PREFIX}{}", base64url_encode_nopad(&raw_secret));
    let key_sha256 = token_sha256(&raw_key);
    let prefix = raw_key.chars().take(12).collect::<String>();
    let id_rand = base64url_encode_nopad(&secure_random_bytes(9).map_err(|e| (500, e))?);
    let created_ms = now_millis();

    let rec = ApiKeyRecord {
        id: format!("key_{id_rand}"),
        name: clean_name.to_string(),
        key_sha256,
        prefix,
        person_id: clean_person.to_string(),
        role,
        scopes: norm_scopes,
        created_ms,
        last_used_ms: None,
        expires_at_ms,
    };

    with_api_keys_lock(|| {
        let mut keys = load_api_keys_unlocked(root_data_dir);
        keys.push(rec.clone());
        save_api_keys_unlocked(root_data_dir, &keys).map_err(|e| (500, e))?;
        Ok((rec, raw_key))
    })
}

pub fn list_api_keys(root_data_dir: &Path) -> Vec<ApiKeyRecord> {
    with_api_keys_lock(|| load_api_keys_unlocked(root_data_dir))
}

pub fn revoke_api_key(root_data_dir: &Path, id: &str) -> Result<bool, String> {
    let target = id.trim();
    if target.is_empty() {
        return Err("key id is required".to_string());
    }
    with_api_keys_lock(|| {
        let mut keys = load_api_keys_unlocked(root_data_dir);
        let before = keys.len();
        keys.retain(|k| k.id != target);
        if keys.len() == before {
            return Ok(false);
        }
        save_api_keys_unlocked(root_data_dir, &keys)?;
        Ok(true)
    })
}

pub fn authenticate_api_key(
    root_data_dir: &Path,
    raw_key: &str,
    people: &RwLock<PeopleState>,
) -> Option<AuthPrincipal> {
    let trimmed = raw_key.trim();
    if !trimmed.starts_with(API_KEY_PREFIX) {
        return None;
    }
    let presented_hash = token_sha256(trimmed);
    let now = now_millis();

    let matched = with_api_keys_lock(|| {
        let mut keys = load_api_keys_unlocked(root_data_dir);
        let idx = keys.iter().position(|k| {
            tripwire::ct_eq(k.key_sha256.as_bytes(), presented_hash.as_bytes())
        })?;
        if keys[idx].expires_at_ms.is_some_and(|exp| exp <= now) {
            return None;
        }
        keys[idx].last_used_ms = Some(now);
        let cloned = keys[idx].clone();
        let _ = save_api_keys_unlocked(root_data_dir, &keys);
        Some(cloned)
    })?;

    let users = AuthStore::load_users_file(&AuthStore::users_path(root_data_dir));
    if let Some(u) = users.iter().find(|u| u.person_id == matched.person_id) {
        if u.disabled {
            return None;
        }
        return Some(AuthPrincipal {
            user_id: u.id.clone(),
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            email: u.email.clone(),
            role: u.role,
            home_person_id: u.person_id.clone(),
            active_person_id: u.person_id.clone(),
            auth_method: format!("api_key:{}", matched.id),
            is_break_glass: false,
            session_token_hash: None,
            api_key_scopes: Some(matched.scopes),
        });
    }

    let person_name = people
        .read()
        .unwrap()
        .people
        .iter()
        .find(|p| p.id == matched.person_id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| matched.name.clone());

    Some(AuthPrincipal {
        user_id: matched.id.clone(),
        username: matched.name,
        display_name: person_name,
        email: None,
        role: matched.role,
        home_person_id: matched.person_id.clone(),
        active_person_id: matched.person_id,
        auth_method: format!("api_key:{}", matched.id),
        is_break_glass: false,
        session_token_hash: None,
        api_key_scopes: Some(matched.scopes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pbkdf2_hmac_sha256_matches_known_rfc_vector() {
        // Test vector from draft-josefsson-scrypt / RFC 7914 & IEEE 802.11 PBKDF2-HMAC-SHA256:
        // password="password", salt="salt", c=1 -> 120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b
        let dk1 = pbkdf2_hmac_sha256(b"password", b"salt", 1);
        let hex1: String = dk1.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex1,
            "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
        );
        // c=2 -> ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43
        let dk2 = pbkdf2_hmac_sha256(b"password", b"salt", 2);
        let hex2: String = dk2.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex2,
            "ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43"
        );
    }

    #[test]
    fn password_hash_uses_unique_salt_and_verifies_constant_time() {
        let h1 = hash_password("correct-horse-battery").unwrap();
        let h2 = hash_password("correct-horse-battery").unwrap();
        assert_ne!(h1, h2, "each user hash must have a unique random salt");
        assert!(h1.starts_with("pbkdf2_sha256$120000$"));
        assert!(verify_password("correct-horse-battery", &h1));
        assert!(verify_password("correct-horse-battery", &h2));
        assert!(!verify_password("wrong-password", &h1));
        assert!(!verify_password("correct-horse-battery", "pbkdf2_sha256$100$AAAA$BBBB"));
    }

    #[test]
    fn rbac_matrix_enforces_role_boundaries() {
        assert!(Role::Admin.satisfies(RouteAccess::RequireAdmin));
        assert!(!Role::Pm.satisfies(RouteAccess::RequireAdmin));
        assert!(!Role::Ae.satisfies(RouteAccess::RequireAdmin));
        assert!(!Role::Office.satisfies(RouteAccess::RequireAdmin));
        assert!(!Role::Installer.satisfies(RouteAccess::RequireAdmin));
        assert!(!Role::Member.satisfies(RouteAccess::RequireAdmin));

        assert!(Role::Pm.satisfies(RouteAccess::RequireClientManage));
        assert!(Role::Ae.satisfies(RouteAccess::RequireInvoiceManage));
        assert!(Role::Office.satisfies(RouteAccess::RequireInvoiceManage));
        assert!(!Role::Pm.satisfies(RouteAccess::RequireInvoiceManage));
        assert!(Role::Pm.satisfies(RouteAccess::RequireInvoiceView));
        assert!(!Role::Installer.satisfies(RouteAccess::RequireClientManage));
        assert!(!Role::Installer.satisfies(RouteAccess::RequireInvoiceView));
        assert!(!Role::Member.satisfies(RouteAccess::RequireInvoiceView));

        assert_eq!(
            route_access("GET", "/api/company/memory"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/company/memory"),
            RouteAccess::RequireClientManage
        );
        assert_eq!(
            route_access("DELETE", "/api/company/memory/cmem-1"),
            RouteAccess::RequireClientManage
        );
        assert_eq!(
            route_access("GET", "/api/company/db"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/company/db/query"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("GET", "/api/tools"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/tools/test"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/tools/build"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/tools/my-tool/run"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/offers/analyze"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("GET", "/api/notifications"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/notifications/notif-1/read"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/push/subscribe"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("DELETE", "/api/push/subscribe/psub-1"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(route_access("GET", "/api/keys"), RouteAccess::RequireAdmin);
        assert_eq!(route_access("POST", "/api/keys"), RouteAccess::RequireAdmin);
        assert_eq!(
            route_access("DELETE", "/api/keys/key_1"),
            RouteAccess::RequireAdmin
        );
        assert_eq!(
            route_access("POST", "/api/webhooks/subscriptions"),
            RouteAccess::RequireClientManage
        );
        assert_eq!(
            route_access("GET", "/api/evolution/scores"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/evolution/propose"),
            RouteAccess::RequireClientManage
        );
        assert_eq!(
            route_access("POST", "/api/evolution/outcomes"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("GET", "/api/quality-bars"),
            RouteAccess::AuthenticatedSelf
        );
        assert_eq!(
            route_access("POST", "/api/quality-bars/weekly-brief/feedback"),
            RouteAccess::AuthenticatedSelf
        );
    }

    #[test]
    fn scoped_api_key_lifecycle_and_scope_checks() {
        let dir = std::env::temp_dir().join(format!(
            "ergon-apikey-test-{}-{}",
            std::process::id(),
            now_millis()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = Config {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            memory_path: dir.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs().unwrap();
        let people = RwLock::new(PeopleState::load(&cfg));

        let (rec, raw) = create_api_key(
            &dir,
            "ci-reader",
            "default",
            Role::Pm,
            &["read".to_string(), "mcp".to_string()],
            None,
        )
        .unwrap();
        assert!(raw.starts_with(API_KEY_PREFIX));
        assert_eq!(rec.scopes, vec!["read".to_string(), "mcp".to_string()]);
        assert_eq!(list_api_keys(&dir).len(), 1);

        let principal = authenticate_api_key(&dir, &raw, &people).unwrap();
        assert_eq!(principal.role, Role::Pm);
        assert!(principal.has_scope("read"));
        assert!(principal.has_scope("mcp"));
        assert!(!principal.has_scope("write"));
        assert!(!principal.has_scope("admin"));
        assert!(authenticate_api_key(&dir, "ergk_wrong_secret", &people).is_none());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(api_keys_path(&dir))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        assert!(revoke_api_key(&dir, &rec.id).unwrap());
        assert!(authenticate_api_key(&dir, &raw, &people).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
