//! Proactive App Recognition & Auto-Recommended Plugin Setups.
//!
//! Scans strictly non-sensitive signals from the employee's sandboxed workspace,
//! recent conversations, work log, personal memory, and standard installed desktop
//! apps/CLIs to recommend ready-to-connect plugins in plain English.
//!
//! Privacy & safety invariants:
//! - Never reads secret, key, token, certificate, or `.env` files, dotfiles/dotdirs,
//!   browser profiles, SSH keys, or wallets.
//! - Zero external network calls and zero LLM calls.
//! - Excludes apps already connected, installed, enabled locally, or dismissed.

use crate::records;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;

const MAX_WALK_DEPTH: usize = 4;
const MAX_WORKSPACE_FILES: usize = 120;
const MAX_FILE_READ_BYTES: usize = 32 * 1024;
#[cfg(any(target_os = "linux", target_os = "macos"))]
const MAX_DESKTOP_FILES: usize = 400;
#[cfg(target_os = "linux")]
const MAX_DESKTOP_READ_BYTES: usize = 4 * 1024;
const MAX_THREAD_SCAN: usize = 30;
const MAX_MESSAGES_PER_THREAD: usize = 24;
const MAX_MSG_CHARS: usize = 4096;
const MAX_WORK_SCAN: usize = 40;
const MAX_RECOMMENDATIONS: usize = 6;
const HOST_SCAN_CACHE_TTL_MS: u64 = 5 * 60 * 1000;

static HOST_APP_CACHE: Mutex<Option<(u64, HashSet<String>)>> = Mutex::new(None);

#[derive(Clone, Debug)]
pub struct PluginRecommendation {
    pub plugin_id: String,
    pub name: String,
    pub suite: String,
    pub suite_label: String,
    pub tagline: String,
    pub score: u32,
    pub confidence: &'static str,
    pub sources: Vec<String>,
    pub reasons: Vec<String>,
    pub reason_summary: String,
    pub setup_kind: String,
    pub setup_label: String,
    pub setup_note: Option<String>,
    pub oauth_provider: Option<String>,
    pub has_install_spec: bool,
    pub local_ready: bool,
}

impl PluginRecommendation {
    pub fn to_json(&self) -> Value {
        json!({
            "plugin_id": self.plugin_id,
            "name": self.name,
            "suite": self.suite,
            "suite_label": self.suite_label,
            "tagline": self.tagline,
            "score": self.score,
            "confidence": self.confidence,
            "sources": self.sources,
            "reasons": self.reasons,
            "reason_summary": self.reason_summary,
            "setup_kind": self.setup_kind,
            "setup_label": self.setup_label,
            "setup_note": self.setup_note,
            "oauth_provider": self.oauth_provider,
            "has_install_spec": self.has_install_spec,
            "local_ready": self.local_ready,
        })
    }
}

#[derive(Default)]
struct SignalAccum {
    score: u32,
    sources: BTreeSet<String>,
    reasons: Vec<String>,
    seen_reasons: HashSet<String>,
}

impl SignalAccum {
    fn add(&mut self, points: u32, source: &str, reason: String) {
        self.score = self.score.saturating_add(points);
        self.sources.insert(source.to_string());
        if self.seen_reasons.insert(reason.clone()) && self.reasons.len() < 4 {
            self.reasons.push(reason);
        }
    }
}

pub fn invalidate_host_app_cache() {
    let mut guard = HOST_APP_CACHE.lock().unwrap_or_else(|p| p.into_inner());
    *guard = None;
}

fn is_sensitive_filename(name_lower: &str) -> bool {
    if name_lower.starts_with('.') {
        return true;
    }
    let sensitive_tokens = [
        "secret",
        "credential",
        "token",
        "password",
        "passwd",
        ".env",
        "id_rsa",
        "id_ed25519",
        "wallet",
    ];
    if sensitive_tokens.iter().any(|t| name_lower.contains(t)) {
        return true;
    }
    let sensitive_exts = [".pem", ".key", ".p12", ".pfx", ".keystore", ".jks"];
    sensitive_exts.iter().any(|ext| name_lower.ends_with(ext))
}

fn is_scannable_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        "md" | "txt"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "yml"
            | "html"
            | "htm"
            | "url"
            | "webloc"
            | "sql"
            | "log"
            | "eml"
            | "ics"
    )
}

fn read_bounded_prefix(path: &Path, max_bytes: usize) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut buf = Vec::with_capacity(max_bytes.min(8192));
    let mut handle = file.take(max_bytes as u64);
    if handle.read_to_end(&mut buf).is_err() {
        return None;
    }
    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn contains_ascii_word(haystack_lower: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() || haystack_lower.len() < needle_lower.len() {
        return false;
    }
    let bytes = haystack_lower.as_bytes();
    let nlen = needle_lower.len();
    let mut start = 0usize;
    while let Some(rel) = haystack_lower[start..].find(needle_lower) {
        let idx = start + rel;
        let before_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + nlen;
        let after_ok = after_idx >= bytes.len() || !bytes[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        start = idx + 1;
        if start >= haystack_lower.len() {
            break;
        }
    }
    false
}

struct DomainRule {
    plugin_id: &'static str,
    label: &'static str,
    patterns: &'static [&'static str],
}

const DOMAIN_RULES: &[DomainRule] = &[
    DomainRule {
        plugin_id: "google-gmail",
        label: "Gmail",
        patterns: &["mail.google.com", "gmail.com"],
    },
    DomainRule {
        plugin_id: "google-calendar",
        label: "Google Calendar",
        patterns: &["calendar.google.com", "meet.google.com"],
    },
    DomainRule {
        plugin_id: "google-drive",
        label: "Google Drive",
        patterns: &["drive.google.com"],
    },
    DomainRule {
        plugin_id: "google-docs",
        label: "Google Docs",
        patterns: &["docs.google.com/document"],
    },
    DomainRule {
        plugin_id: "google-sheets",
        label: "Google Sheets",
        patterns: &["docs.google.com/spreadsheets", "sheets.google.com"],
    },
    DomainRule {
        plugin_id: "google-slides",
        label: "Google Slides",
        patterns: &["docs.google.com/presentation", "slides.google.com"],
    },
    DomainRule {
        plugin_id: "outlook-mail",
        label: "Outlook Mail",
        patterns: &[
            "outlook.office.com",
            "outlook.office365.com",
            "outlook.live.com",
        ],
    },
    DomainRule {
        plugin_id: "onedrive-sharepoint",
        label: "OneDrive & SharePoint",
        patterns: &["onedrive.live.com", "sharepoint.com", "-my.sharepoint.com"],
    },
    DomainRule {
        plugin_id: "microsoft-teams",
        label: "Microsoft Teams",
        patterns: &["teams.microsoft.com", "teams.live.com"],
    },
    DomainRule {
        plugin_id: "slack",
        label: "Slack",
        patterns: &["slack.com", "app.slack.com"],
    },
    DomainRule {
        plugin_id: "notion",
        label: "Notion",
        patterns: &["notion.so", "notion.site"],
    },
    DomainRule {
        plugin_id: "github",
        label: "GitHub",
        patterns: &["github.com"],
    },
    DomainRule {
        plugin_id: "linear",
        label: "Linear",
        patterns: &["linear.app"],
    },
    DomainRule {
        plugin_id: "jira",
        label: "Jira",
        patterns: &["atlassian.net/jira", "atlassian.net/browse/"],
    },
    DomainRule {
        plugin_id: "confluence",
        label: "Confluence",
        patterns: &["atlassian.net/wiki"],
    },
    DomainRule {
        plugin_id: "asana",
        label: "Asana",
        patterns: &["app.asana.com", "asana.com/0/"],
    },
    DomainRule {
        plugin_id: "figma",
        label: "Figma",
        patterns: &["figma.com"],
    },
    DomainRule {
        plugin_id: "zoom",
        label: "Zoom",
        patterns: &["zoom.us"],
    },
    DomainRule {
        plugin_id: "hubspot",
        label: "HubSpot",
        patterns: &["hubspot.com"],
    },
    DomainRule {
        plugin_id: "salesforce",
        label: "Salesforce",
        patterns: &["salesforce.com", "lightning.force.com"],
    },
    DomainRule {
        plugin_id: "stripe",
        label: "Stripe",
        patterns: &["dashboard.stripe.com", "stripe.com/"],
    },
    DomainRule {
        plugin_id: "quickbooks",
        label: "QuickBooks Online",
        patterns: &["quickbooks.intuit.com", "qbo.intuit.com"],
    },
    DomainRule {
        plugin_id: "zendesk",
        label: "Zendesk",
        patterns: &["zendesk.com"],
    },
    DomainRule {
        plugin_id: "airtable",
        label: "Airtable",
        patterns: &["airtable.com"],
    },
    DomainRule {
        plugin_id: "shopify",
        label: "Shopify",
        patterns: &["myshopify.com", "admin.shopify.com"],
    },
    DomainRule {
        plugin_id: "postgres-sqlite",
        label: "Postgres & SQLite",
        patterns: &[
            "postgres://",
            "postgresql://",
            "supabase.co",
            "neon.tech",
        ],
    },
];

struct PhraseRule {
    plugin_id: &'static str,
    label: &'static str,
    phrases: &'static [&'static str],
}

/// Brand words and context-qualified phrases. Ambiguous English words like
/// "slack", "linear", "notion", "zoom", "stripe", "asana", "outlook" only
/// match in explicit workplace-app contexts so everyday prose never triggers them.
const PHRASE_RULES: &[PhraseRule] = &[
    PhraseRule {
        plugin_id: "google-gmail",
        label: "Gmail",
        phrases: &["gmail"],
    },
    PhraseRule {
        plugin_id: "google-calendar",
        label: "Google Calendar",
        phrases: &["google calendar", "gcal"],
    },
    PhraseRule {
        plugin_id: "google-drive",
        label: "Google Drive",
        phrases: &["google drive", "gdrive"],
    },
    PhraseRule {
        plugin_id: "google-docs",
        label: "Google Docs",
        phrases: &["google docs", "google doc"],
    },
    PhraseRule {
        plugin_id: "google-sheets",
        label: "Google Sheets",
        phrases: &["google sheets", "google sheet", "gsheet"],
    },
    PhraseRule {
        plugin_id: "google-slides",
        label: "Google Slides",
        phrases: &["google slides", "google slide deck"],
    },
    PhraseRule {
        plugin_id: "outlook-mail",
        label: "Outlook Mail",
        phrases: &[
            "in outlook",
            "outlook mail",
            "outlook email",
            "outlook inbox",
            "microsoft outlook",
        ],
    },
    PhraseRule {
        plugin_id: "outlook-calendar",
        label: "Outlook Calendar",
        phrases: &["outlook calendar", "teams invite"],
    },
    PhraseRule {
        plugin_id: "onedrive-sharepoint",
        label: "OneDrive & SharePoint",
        phrases: &["onedrive", "sharepoint"],
    },
    PhraseRule {
        plugin_id: "microsoft-teams",
        label: "Microsoft Teams",
        phrases: &["microsoft teams", "ms teams", "teams channel", "teams meeting"],
    },
    PhraseRule {
        plugin_id: "slack",
        label: "Slack",
        phrases: &[
            "in slack",
            "on slack",
            "via slack",
            "to slack",
            "from slack",
            "slack channel",
            "slack message",
            "slack dm",
            "slack thread",
            "slack workspace",
        ],
    },
    PhraseRule {
        plugin_id: "notion",
        label: "Notion",
        phrases: &[
            "in notion",
            "on notion",
            "notion page",
            "notion doc",
            "notion wiki",
            "notion database",
            "notion workspace",
        ],
    },
    PhraseRule {
        plugin_id: "github",
        label: "GitHub",
        phrases: &["github"],
    },
    PhraseRule {
        plugin_id: "linear",
        label: "Linear",
        phrases: &[
            "in linear",
            "on linear",
            "linear issue",
            "linear ticket",
            "linear board",
            "linear project",
            "linear sprint",
        ],
    },
    PhraseRule {
        plugin_id: "jira",
        label: "Jira",
        phrases: &["jira"],
    },
    PhraseRule {
        plugin_id: "confluence",
        label: "Confluence",
        phrases: &["confluence"],
    },
    PhraseRule {
        plugin_id: "asana",
        label: "Asana",
        phrases: &[
            "in asana",
            "on asana",
            "asana task",
            "asana project",
            "asana board",
        ],
    },
    PhraseRule {
        plugin_id: "figma",
        label: "Figma",
        phrases: &["figma", "figjam"],
    },
    PhraseRule {
        plugin_id: "zoom",
        label: "Zoom",
        phrases: &[
            "on zoom",
            "via zoom",
            "zoom call",
            "zoom meeting",
            "zoom link",
            "zoom recording",
        ],
    },
    PhraseRule {
        plugin_id: "hubspot",
        label: "HubSpot",
        phrases: &["hubspot"],
    },
    PhraseRule {
        plugin_id: "salesforce",
        label: "Salesforce",
        phrases: &["salesforce", "sfdc"],
    },
    PhraseRule {
        plugin_id: "stripe",
        label: "Stripe",
        phrases: &[
            "in stripe",
            "on stripe",
            "stripe invoice",
            "stripe payment",
            "stripe payout",
            "stripe customer",
            "stripe subscription",
        ],
    },
    PhraseRule {
        plugin_id: "quickbooks",
        label: "QuickBooks Online",
        phrases: &["quickbooks", "quickbooks online"],
    },
    PhraseRule {
        plugin_id: "zendesk",
        label: "Zendesk",
        phrases: &["zendesk"],
    },
    PhraseRule {
        plugin_id: "airtable",
        label: "Airtable",
        phrases: &["airtable"],
    },
    PhraseRule {
        plugin_id: "shopify",
        label: "Shopify",
        phrases: &["shopify"],
    },
    PhraseRule {
        plugin_id: "postgres-sqlite",
        label: "Postgres & SQLite",
        phrases: &["postgresql", "postgres", "sqlite", "duckdb"],
    },
];

fn source_label(source: &str) -> &'static str {
    match source {
        "workspace" => "your workspace files",
        "conversations" => "your recent conversations",
        "work_log" => "your recent tasks",
        "memory" => "your saved notes",
        "desktop_app" => "this computer",
        _ => "your activity",
    }
}

fn match_text_signals(
    text: &str,
    source: &str,
    accums: &mut BTreeMap<String, SignalAccum>,
) {
    if text.trim().is_empty() {
        return;
    }
    let lower = text.to_ascii_lowercase();
    let origin = source_label(source);

    for rule in DOMAIN_RULES {
        if let Some(pat) = rule.patterns.iter().find(|p| lower.contains(**p)) {
            accums.entry(rule.plugin_id.to_string()).or_default().add(
                60,
                source,
                format!("Found {} ({pat}) in {origin}", rule.label),
            );
        }
    }

    for rule in PHRASE_RULES {
        let already_domain = accums
            .get(rule.plugin_id)
            .map(|a| a.sources.contains(source))
            .unwrap_or(false);
        if already_domain {
            continue;
        }
        if rule
            .phrases
            .iter()
            .any(|phrase| contains_ascii_word(&lower, phrase))
        {
            accums.entry(rule.plugin_id.to_string()).or_default().add(
                40,
                source,
                format!("{} mentioned in {origin}", rule.label),
            );
        }
    }
}

fn scan_workspace_signals(person_data_dir: &Path, accums: &mut BTreeMap<String, SignalAccum>) {
    let ws = person_data_dir.join("workspace");
    if !ws.is_dir() {
        return;
    }
    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(ws, 0)];
    let mut files_visited = 0usize;

    while let Some((dir, depth)) = stack.pop() {
        if files_visited >= MAX_WORKSPACE_FILES {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if files_visited >= MAX_WORKSPACE_FILES {
                break;
            }
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let name_lower = name.to_ascii_lowercase();
            if is_sensitive_filename(&name_lower) {
                continue;
            }
            let path = entry.path();
            if ft.is_dir() {
                if depth < MAX_WALK_DEPTH {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            if !ft.is_file() {
                continue;
            }
            files_visited += 1;

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            match ext.as_str() {
                "fig" => accums.entry("figma".to_string()).or_default().add(
                    45,
                    "workspace",
                    format!("Found Figma file \"{name}\" in workspace"),
                ),
                "sql" | "sqlite" | "sqlite3" | "db" | "duckdb" => {
                    accums.entry("postgres-sqlite".to_string()).or_default().add(
                        45,
                        "workspace",
                        format!("Found database file \"{name}\" in workspace"),
                    )
                }
                "gsheet" => accums.entry("google-sheets".to_string()).or_default().add(
                    50,
                    "workspace",
                    format!("Found Google Sheet link \"{name}\" in workspace"),
                ),
                "gdoc" => accums.entry("google-docs".to_string()).or_default().add(
                    50,
                    "workspace",
                    format!("Found Google Doc link \"{name}\" in workspace"),
                ),
                "gslides" => accums.entry("google-slides".to_string()).or_default().add(
                    50,
                    "workspace",
                    format!("Found Google Slides link \"{name}\" in workspace"),
                ),
                "ics" => {
                    let cal_id = if accums.contains_key("outlook-mail")
                        || accums.contains_key("outlook-calendar")
                    {
                        "outlook-calendar"
                    } else {
                        "google-calendar"
                    };
                    accums.entry(cal_id.to_string()).or_default().add(
                        35,
                        "workspace",
                        format!("Found calendar invite \"{name}\" in workspace"),
                    );
                }
                "eml" | "mbox" => accums.entry("outlook-mail".to_string()).or_default().add(
                    35,
                    "workspace",
                    format!("Found email archive \"{name}\" in workspace"),
                ),
                _ => {}
            }

            if is_scannable_text_ext(&ext) {
                if let Some(content) = read_bounded_prefix(&path, MAX_FILE_READ_BYTES) {
                    match_text_signals(&content, "workspace", accums);
                }
            }
        }
    }
}

fn scan_person_history(person_data_dir: &Path, accums: &mut BTreeMap<String, SignalAccum>) {
    let threads_file = person_data_dir.join("threads.json");
    if threads_file.is_file() {
        let store = crate::Store::load(&threads_file);
        for thread in store.threads.iter().rev().take(MAX_THREAD_SCAN) {
            if !thread.title.trim().is_empty() {
                match_text_signals(&thread.title, "conversations", accums);
            }
            let msg_start = thread.messages.len().saturating_sub(MAX_MESSAGES_PER_THREAD);
            for msg in &thread.messages[msg_start..] {
                let sample = crate::truncate(&msg.content, MAX_MSG_CHARS);
                match_text_signals(&sample, "conversations", accums);
            }
        }
    }

    let work_entries = records::load_work(person_data_dir);
    for entry in work_entries.iter().rev().take(MAX_WORK_SCAN) {
        let task = entry["task"].as_str().unwrap_or("");
        let summary = entry["summary"].as_str().unwrap_or("");
        let files = entry["files"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let combined = format!("{task} {summary} {files}");
        match_text_signals(&combined, "work_log", accums);
    }

    let memory_file = person_data_dir.join("memory.md");
    if memory_file.is_file() {
        if let Some(mem) = read_bounded_prefix(&memory_file, MAX_FILE_READ_BYTES) {
            match_text_signals(&mem, "memory", accums);
        }
    }
}

fn env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn host_token_to_plugin(token_lower: &str) -> Option<&'static str> {
    if token_lower == "gh" || token_lower.contains("github") {
        return Some("github");
    }
    if token_lower.contains("slack") {
        return Some("slack");
    }
    if token_lower == "zoom" || token_lower.contains("zoom") && !token_lower.contains("zoomer") {
        return Some("zoom");
    }
    if token_lower.contains("figma") {
        return Some("figma");
    }
    if token_lower.contains("notion") {
        return Some("notion");
    }
    if token_lower == "linear" || token_lower.starts_with("linear-") || token_lower.ends_with("-linear") {
        return Some("linear");
    }
    if token_lower.contains("teams-for-linux") || token_lower.contains("microsoft teams") || token_lower == "teams" {
        return Some("microsoft-teams");
    }
    if token_lower.contains("outlook") {
        return Some("outlook-mail");
    }
    if token_lower == "psql"
        || token_lower == "sqlite3"
        || token_lower.contains("postgres")
        || token_lower.contains("sqlite")
        || token_lower.contains("dbeaver")
        || token_lower.contains("pgadmin")
    {
        return Some("postgres-sqlite");
    }
    None
}

fn find_in_path(bin_name: &str) -> bool {
    let Some(path_env) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(bin_name);
        if candidate.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{bin_name}.exe"));
            if exe.is_file() {
                return true;
            }
        }
    }
    false
}

fn probe_desktop_apps_uncached() -> HashSet<String> {
    let mut detected = HashSet::new();

    for bin in [
        "gh",
        "psql",
        "sqlite3",
        "slack",
        "zoom",
        "figma-linux",
        "notion-app",
        "teams-for-linux",
    ] {
        if find_in_path(bin) {
            if let Some(pid) = host_token_to_plugin(bin) {
                detected.insert(pid.to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mut desktop_dirs = vec![std::path::PathBuf::from("/usr/share/applications")];
        if let Some(home) = std::env::var_os("HOME") {
            desktop_dirs.push(Path::new(&home).join(".local/share/applications"));
        }
        let mut inspected = 0usize;
        for dir in desktop_dirs {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if inspected >= MAX_DESKTOP_FILES {
                    break;
                }
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                inspected += 1;
                let fname = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if let Some(pid) = host_token_to_plugin(&fname) {
                    detected.insert(pid.to_string());
                    continue;
                }
                if let Some(text) = read_bounded_prefix(&path, MAX_DESKTOP_READ_BYTES) {
                    for line in text.lines() {
                        if let Some(rest) = line.strip_prefix("Name=").or_else(|| line.strip_prefix("Exec=")) {
                            let val_lower = rest.trim().to_ascii_lowercase();
                            if let Some(pid) = host_token_to_plugin(&val_lower) {
                                detected.insert(pid.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = std::fs::read_dir("/Applications") {
            for entry in entries.flatten().take(MAX_DESKTOP_FILES) {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if let Some(pid) = host_token_to_plugin(name.trim_end_matches(".app")) {
                    detected.insert(pid.to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let roots = ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"];
        let checks = [
            ("Slack", "slack"),
            ("Zoom", "zoom"),
            ("Figma", "figma"),
            ("Notion", "notion"),
            ("Linear", "linear"),
            ("GitHub CLI", "github"),
            ("Microsoft\\Teams", "microsoft-teams"),
        ];
        for env_key in roots {
            if let Some(base) = std::env::var_os(env_key) {
                let base_path = Path::new(&base);
                for (sub, pid) in checks {
                    if base_path.join(sub).exists() {
                        detected.insert(pid.to_string());
                    }
                }
            }
        }
    }

    detected
}

fn scan_host_apps(rescan_host: bool, accums: &mut BTreeMap<String, SignalAccum>) {
    if env_truthy("ERGON_DISABLE_HOST_APP_SCAN") {
        return;
    }

    let found: HashSet<String> = if let Ok(raw) = std::env::var("ERGON_TEST_HOST_APPS") {
        raw.split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .filter_map(|tok| host_token_to_plugin(&tok).map(str::to_string))
            .collect()
    } else {
        let now = records::now_millis();
        let mut cache = HOST_APP_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        let cached = if !rescan_host {
            cache.as_ref().and_then(|(ts, apps)| {
                if now.saturating_sub(*ts) < HOST_SCAN_CACHE_TTL_MS {
                    Some(apps.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        if let Some(apps) = cached {
            apps
        } else {
            let fresh = probe_desktop_apps_uncached();
            *cache = Some((now, fresh.clone()));
            fresh
        }
    };

    for pid in found {
        let label = match pid.as_str() {
            "postgres-sqlite" => "PostgreSQL / SQLite CLI",
            "github" => "GitHub CLI / Desktop",
            "microsoft-teams" => "Microsoft Teams",
            "outlook-mail" => "Outlook",
            "slack" => "Slack",
            "zoom" => "Zoom",
            "figma" => "Figma",
            "notion" => "Notion",
            "linear" => "Linear",
            other => other,
        };
        let reason = format!("{label} detected on this computer");
        accums.entry(pid).or_default().add(
            35,
            "desktop_app",
            reason,
        );
    }
}

fn resolve_setup_action(
    plugin: &Value,
    google_oauth_configured: bool,
    ms_oauth_configured: bool,
    air_gapped: bool,
) -> (String, String, Option<String>) {
    let name = plugin["name"].as_str().unwrap_or("App");
    let oauth_provider = plugin["oauth_provider"].as_str();
    let local_ready = plugin["local_ready"].as_bool() == Some(true);
    let has_install_spec = plugin["install_spec"].is_object();

    let (kind, label, mut note) = match oauth_provider {
        Some("google") => {
            if google_oauth_configured {
                (
                    "oauth_google".to_string(),
                    "Sign in with Google".to_string(),
                    None,
                )
            } else if local_ready {
                (
                    "enable_local".to_string(),
                    "Turn on offline".to_string(),
                    Some("Works offline right away; an admin can enable Google sign-in anytime.".to_string()),
                )
            } else {
                (
                    "oauth_google_setup".to_string(),
                    "Set up Google sign-in".to_string(),
                    Some("Needs a one-time Google sign-in setup by your workspace admin.".to_string()),
                )
            }
        }
        Some("microsoft") => {
            if ms_oauth_configured {
                (
                    "oauth_microsoft".to_string(),
                    "Sign in with Microsoft".to_string(),
                    None,
                )
            } else if local_ready {
                (
                    "enable_local".to_string(),
                    "Turn on offline".to_string(),
                    Some("Works offline right away; an admin can enable Microsoft sign-in anytime.".to_string()),
                )
            } else {
                (
                    "oauth_microsoft_setup".to_string(),
                    "Set up Microsoft sign-in".to_string(),
                    Some("Needs a one-time Microsoft sign-in setup by your workspace admin.".to_string()),
                )
            }
        }
        _ if has_install_spec => (
            "one_click_connect".to_string(),
            format!("Connect {name}"),
            None,
        ),
        _ if local_ready => ("enable_local".to_string(), "Turn on".to_string(), None),
        _ => (
            "connector".to_string(),
            format!("Set up {name}"),
            Some("Connect via Settings or Connectors when your workspace allows it.".to_string()),
        ),
    };

    if air_gapped && !local_ready && note.is_none() {
        note = Some(
            "Air-Gapped mode is active — cloud sync pauses until your admin allows outbound apps."
                .to_string(),
        );
    }

    (kind, label, note)
}

/// Scans workspace, history, and host desktop signals for a person and returns
/// up to `MAX_RECOMMENDATIONS` ranked, plain-English plugin setup recommendations.
pub fn detect_recommendations_for_plugins(
    person_data_dir: &Path,
    plugins: &[Value],
    dismissed: &HashSet<String>,
    google_oauth_configured: bool,
    ms_oauth_configured: bool,
    air_gapped: bool,
    rescan_host: bool,
) -> Vec<PluginRecommendation> {
    let mut accums: BTreeMap<String, SignalAccum> = BTreeMap::new();
    scan_workspace_signals(person_data_dir, &mut accums);
    scan_person_history(person_data_dir, &mut accums);
    scan_host_apps(rescan_host, &mut accums);

    let mut out = Vec::new();
    for plugin in plugins {
        let id = plugin["id"].as_str().unwrap_or("");
        if id.is_empty() || dismissed.contains(id) {
            continue;
        }
        let Some(acc) = accums.get(id) else {
            continue;
        };
        let cloud_connected = plugin["cloud_connected"].as_bool() == Some(true);
        let installed = plugin["installed"].as_bool() == Some(true);
        let local_ready = plugin["local_ready"].as_bool() == Some(true);
        let enabled = plugin["enabled"].as_bool() == Some(true);
        if cloud_connected || installed || (local_ready && enabled) {
            continue;
        }

        let sources: Vec<String> = acc.sources.iter().cloned().collect();
        let confidence = if acc.score >= 55 || sources.len() >= 2 {
            "high"
        } else {
            "medium"
        };
        let reason_summary = acc
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "Matched your recent work".to_string());
        let (setup_kind, setup_label, setup_note) =
            resolve_setup_action(plugin, google_oauth_configured, ms_oauth_configured, air_gapped);

        out.push(PluginRecommendation {
            plugin_id: id.to_string(),
            name: plugin["name"].as_str().unwrap_or(id).to_string(),
            suite: plugin["suite"].as_str().unwrap_or("business").to_string(),
            suite_label: plugin["suite_label"]
                .as_str()
                .unwrap_or("Business Apps")
                .to_string(),
            tagline: plugin["tagline"].as_str().unwrap_or("").to_string(),
            score: acc.score,
            confidence,
            sources,
            reasons: acc.reasons.clone(),
            reason_summary,
            setup_kind,
            setup_label,
            setup_note,
            oauth_provider: plugin["oauth_provider"].as_str().map(str::to_string),
            has_install_spec: plugin["install_spec"].is_object(),
            local_ready,
        });
    }

    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| (b.confidence == "high").cmp(&(a.confidence == "high")))
            .then_with(|| a.plugin_id.cmp(&b.plugin_id))
    });
    out.truncate(MAX_RECOMMENDATIONS);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundary_avoids_false_positives_in_ordinary_prose() {
        let mut accums = BTreeMap::new();
        match_text_signals(
            "We need to pick up the slack with a linear trend and zoom in on the stripe pattern.",
            "conversations",
            &mut accums,
        );
        assert!(!accums.contains_key("slack"));
        assert!(!accums.contains_key("linear"));
        assert!(!accums.contains_key("zoom"));
        assert!(!accums.contains_key("stripe"));
    }

    #[test]
    fn recognizes_domain_and_contextual_app_references() {
        let mut accums = BTreeMap::new();
        match_text_signals(
            "Track https://linear.app/acme/issue/ENG-42 and post summary in our Slack channel #ops.",
            "workspace",
            &mut accums,
        );
        assert!(accums.contains_key("linear"));
        assert!(accums.contains_key("slack"));
        assert!(accums["linear"].score >= 60);
        assert!(accums["slack"].score >= 40);
    }
}
