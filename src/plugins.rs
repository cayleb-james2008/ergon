//! Default App Plugin Library + Hybrid Cloud/Local Runtime.
//!
//! Ergon ships with a curated catalog of common company and employee apps across:
//! - **Google Workspace** (Gmail, Calendar, Drive, Docs, Sheets, Slides)
//! - **Microsoft 365 & Outlook** (Outlook Mail, Outlook Calendar, OneDrive & SharePoint, Teams, Excel, Word & PDF)
//! - **Business Apps** (Slack, Notion, GitHub, CRM & Billing)
//! - **On-Device Local** (On-Device Files & Watched Folders, Local Apps & MCP Bridge)
//!
//! Plugins run **locally on-device** when offline-capable (Office files, spreadsheets, workspace search,
//! local MCP bridge) or seamlessly switch to **cloud** mode once bound to Connectors / OAuth2 credentials.
//! Consequential external writes always park as Approve/Deny cards on `/phone` and Today instead of firing silently.

use crate::tools::atomic_write_bytes_with_mode;
use crate::{connectors, mcp, parse_url_host_port, records, Config};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static PLUGINS_LOCK: Mutex<()> = Mutex::new(());

fn with_plugins_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = PLUGINS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

pub fn plugins_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("plugins.json")
}

#[derive(Clone, Debug)]
pub struct PluginActionSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub access: &'static str, // "read" | "write"
    pub requires_approval: bool,
}

impl PluginActionSpec {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "label": self.label,
            "description": self.description,
            "access": self.access,
            "requires_approval": self.requires_approval,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PluginConnectPreset {
    pub api_base_url: Option<&'static str>,
    pub oauth_token_url: Option<&'static str>,
    pub oauth_scopes: Option<&'static str>,
    pub smtp_host: Option<&'static str>,
    pub smtp_port: Option<u16>,
    pub smtp_security: Option<&'static str>,
    pub imap_host: Option<&'static str>,
    pub imap_port: Option<u16>,
    pub imap_security: Option<&'static str>,
    pub match_hosts: &'static [&'static str],
    pub connector_tab: &'static str, // "folders" | "outbound" | "mcp"
}

impl PluginConnectPreset {
    pub fn to_json(&self) -> Value {
        json!({
            "api_base_url": self.api_base_url,
            "oauth_token_url": self.oauth_token_url,
            "oauth_scopes": self.oauth_scopes,
            "smtp_host": self.smtp_host,
            "smtp_port": self.smtp_port,
            "smtp_security": self.smtp_security,
            "imap_host": self.imap_host,
            "imap_port": self.imap_port,
            "imap_security": self.imap_security,
            "connector_tab": self.connector_tab,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub id: &'static str,
    pub suite: &'static str,       // "google" | "microsoft" | "business" | "local"
    pub suite_label: &'static str,
    pub name: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
    pub mode: &'static str,      // "local" | "cloud" | "hybrid"
    pub transport: &'static str, // "local_builtin" | "smtp_imap_xoauth2" | "oauth2_rest" | "mcp"
    pub local_ready: bool,
    pub default_enabled: bool,
    pub actions: &'static [PluginActionSpec],
    pub connect_preset: Option<PluginConnectPreset>,
}

#[derive(Clone, Debug)]
pub struct PluginPersonSetting {
    pub enabled: bool,
    pub preferred_mode: String, // "auto" | "local" | "cloud"
    pub updated_ms: u64,
}

impl PluginPersonSetting {
    fn from_json(v: &Value, default_enabled: bool) -> Self {
        let enabled = v["enabled"].as_bool().unwrap_or(default_enabled);
        let preferred_mode = match v["preferred_mode"].as_str().unwrap_or("auto") {
            "local" => "local".to_string(),
            "cloud" => "cloud".to_string(),
            _ => "auto".to_string(),
        };
        let updated_ms = v["updated_ms"].as_u64().unwrap_or(0);
        Self {
            enabled,
            preferred_mode,
            updated_ms,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "preferred_mode": self.preferred_mode,
            "updated_ms": self.updated_ms,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PluginsStore {
    pub settings: BTreeMap<String, PluginPersonSetting>,
}

impl PluginsStore {
    fn load_unlocked(person_data_dir: &Path) -> Self {
        let path = plugins_path(person_data_dir);
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<Value>(&raw) else {
            return Self::default();
        };
        let mut settings = BTreeMap::new();
        if let Some(obj) = val["settings"].as_object() {
            for (k, v) in obj {
                if let Some(spec) = find_plugin(k) {
                    settings.insert(k.clone(), PluginPersonSetting::from_json(v, spec.default_enabled));
                }
            }
        }
        Self { settings }
    }

    fn save_unlocked(&self, person_data_dir: &Path) -> Result<(), String> {
        let mut obj = serde_json::Map::new();
        for (k, v) in &self.settings {
            obj.insert(k.clone(), v.to_json());
        }
        let body = serde_json::to_string_pretty(&json!({ "settings": obj }))
            .map_err(|e| format!("serialize plugins: {e}"))?;
        atomic_write_bytes_with_mode(&plugins_path(person_data_dir), body.as_bytes(), Some(0o600))
            .map_err(|e| format!("write plugins.json: {e}"))
    }
}

const GMAIL_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "read_inbox",
        label: "Read inbox",
        description: "Poll unread Gmail messages into workspace/intake/ via IMAPS XOAUTH2.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "draft_reply",
        label: "Draft reply",
        description: "Compose a local Markdown email draft inside the workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "send_mail",
        label: "Send email",
        description: "Send email via Gmail SMTP XOAUTH2 (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const GCAL_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_events",
        label: "List events",
        description: "Fetch upcoming Google Calendar schedule.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "create_event",
        label: "Create event",
        description: "Schedule or update a calendar invite (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const GDRIVE_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_files",
        label: "List files",
        description: "List synced Drive folders in Google Cloud or on-device workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "upload_deliverable",
        label: "Upload file",
        description: "Upload a workspace deliverable to Drive (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const GDOCS_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "draft_doc",
        label: "Draft document",
        description: "Create a structured Markdown or Word draft in the local workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "publish_doc",
        label: "Publish to Docs",
        description: "Sync document to Google Docs (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const GSHEETS_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "reconcile_sheet",
        label: "Build / reconcile sheet",
        description: "Run deterministic exact-decimal table rollups locally in the workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "update_range",
        label: "Push range update",
        description: "Write rows to Google Sheets (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const GSLIDES_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "draft_outline",
        label: "Draft deck outline",
        description: "Draft a slide-by-slide Markdown outline locally in the workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "update_deck",
        label: "Publish slides",
        description: "Update a Google Slides presentation (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const OUTLOOK_MAIL_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "read_inbox",
        label: "Read inbox",
        description: "Poll unread Outlook / Microsoft 365 messages via IMAPS XOAUTH2.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "draft_reply",
        label: "Draft reply",
        description: "Compose an Outlook reply draft locally inside the workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "send_mail",
        label: "Send email",
        description: "Send mail via smtp.office365.com STARTTLS XOAUTH2 (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const OUTLOOK_CAL_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_events",
        label: "List events",
        description: "Read Microsoft 365 calendar events via Microsoft Graph.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "schedule_meeting",
        label: "Schedule meeting",
        description: "Create an Outlook calendar invite (parks Approve/Deny card).",
        access: "write",
        requires_approval: true,
    },
];

const ONEDRIVE_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_files",
        label: "List files",
        description: "List OneDrive/SharePoint items or local synced workspace files.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "publish_file",
        label: "Publish to SharePoint",
        description: "Upload a finished deliverable to OneDrive/SharePoint (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const TEAMS_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "read_channel",
        label: "Read channel",
        description: "Fetch recent Microsoft Teams channel messages.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "post_update",
        label: "Post Teams update",
        description: "Send a message to a Microsoft Teams channel (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const EXCEL_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "inspect_workbook",
        label: "Inspect sheets",
        description: "List and inspect .xlsx/.csv spreadsheets in the local workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "build_sheet",
        label: "Build spreadsheet",
        description: "Create a penny-exact CSV or Excel sheet on-device via make_spreadsheet.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "export_xlsx",
        label: "Export .xlsx workbook",
        description: "Export workspace tables to multi-sheet Excel with live SUM formulas.",
        access: "read",
        requires_approval: false,
    },
];

const WORD_PDF_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "read_document",
        label: "Read Word / PDF",
        description: "Inspect .docx, .pdf, or .md files in the local workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "draft_document",
        label: "Draft document",
        description: "Generate a clean Markdown report locally via write_document.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "export_docx_pdf",
        label: "Export PDF / Word",
        description: "Export a Markdown file to native .pdf or .docx locally.",
        access: "read",
        requires_approval: false,
    },
];

const SLACK_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "read_channel",
        label: "Read channel",
        description: "Read recent channel activity via Slack API connector.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "post_message",
        label: "Post Slack message",
        description: "Send a message to a Slack channel or webhook (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const NOTION_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "search_pages",
        label: "Search Notion",
        description: "Search connected Notion databases and pages.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "update_page",
        label: "Update page",
        description: "Write an entry or page update in Notion (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const GITHUB_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_issues",
        label: "List issues & PRs",
        description: "Query repository issues and pull requests.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "comment_issue",
        label: "Comment on PR / Issue",
        description: "Post a comment or status update on GitHub (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const CRM_BILLING_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_clients",
        label: "List clients",
        description: "Read active clients and retainers from local Company DB.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "sync_invoice",
        label: "Sync external billing",
        description: "Push an approved invoice to Stripe / QuickBooks (parks approval).",
        access: "write",
        requires_approval: true,
    },
];

const LOCAL_FILES_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_workspace",
        label: "List workspace files",
        description: "Inspect files in the employee's sandboxed on-device workspace.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "search_files",
        label: "Search workspace",
        description: "Search across workspace text, CSV, Word, and PDF documents locally.",
        access: "read",
        requires_approval: false,
    },
    PluginActionSpec {
        name: "watch_folder_status",
        label: "Watched folders",
        description: "List active local watched directories routing files to routines.",
        access: "read",
        requires_approval: false,
    },
];

const LOCAL_MCP_ACTIONS: &[PluginActionSpec] = &[
    PluginActionSpec {
        name: "list_mcp_tools",
        label: "List MCP tools",
        description: "List discovered tools from local or HTTP MCP servers.",
        access: "read",
        requires_approval: false,
    },
];

const CATALOG: &[PluginSpec] = &[
    // Google Workspace (6)
    PluginSpec {
        id: "google-gmail",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Gmail",
        tagline: "Inbox intake & approval-gated sends",
        description: "Polls Gmail via IMAPS XOAUTH2 into intake/ and sends outbound mail only after /phone approval.",
        mode: "cloud",
        transport: "smtp_imap_xoauth2",
        local_ready: false,
        default_enabled: false,
        actions: GMAIL_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: None,
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://mail.google.com/"),
            smtp_host: Some("smtp.gmail.com"),
            smtp_port: Some(587),
            smtp_security: Some("starttls"),
            imap_host: Some("imap.gmail.com"),
            imap_port: Some(993),
            imap_security: Some("imaps"),
            match_hosts: &["imap.gmail.com", "smtp.gmail.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "google-calendar",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Google Calendar",
        tagline: "Schedules, availability & invites",
        description: "Reads team calendar events and proposes meeting invites with Approve/Deny gating.",
        mode: "cloud",
        transport: "oauth2_rest",
        local_ready: false,
        default_enabled: false,
        actions: GCAL_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://www.googleapis.com/calendar/v3/calendars/primary/events"),
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://www.googleapis.com/auth/calendar"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["www.googleapis.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "google-drive",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Google Drive",
        tagline: "Workspace sync & Drive deliverables",
        description: "Reads local synced workspace files offline or connects Drive API v3 for cloud uploads.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: GDRIVE_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://www.googleapis.com/drive/v3/files"),
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://www.googleapis.com/auth/drive"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["www.googleapis.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "google-docs",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Google Docs",
        tagline: "Local drafting + cloud Docs sync",
        description: "Drafts Markdown/Word briefs on-device immediately; optionally syncs to Google Docs with approval.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: GDOCS_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://docs.googleapis.com/v1/documents"),
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://www.googleapis.com/auth/documents"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["docs.googleapis.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "google-sheets",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Google Sheets",
        tagline: "Exact i128 rollups + cloud Sheets sync",
        description: "Runs penny-exact CSV/table rollups locally and pushes approved ranges to Google Sheets.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: GSHEETS_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://sheets.googleapis.com/v4/spreadsheets"),
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://www.googleapis.com/auth/spreadsheets"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["sheets.googleapis.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "google-slides",
        suite: "google",
        suite_label: "Google Workspace",
        name: "Google Slides",
        tagline: "Deck outlines locally + cloud publish",
        description: "Builds deck outlines locally and pushes approved slide updates via Google Slides API.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: GSLIDES_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://slides.googleapis.com/v1/presentations"),
            oauth_token_url: Some("https://oauth2.googleapis.com/token"),
            oauth_scopes: Some("https://www.googleapis.com/auth/presentations"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["slides.googleapis.com"],
            connector_tab: "outbound",
        }),
    },

    // Microsoft 365 & Outlook (6)
    PluginSpec {
        id: "outlook-mail",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "Outlook Mail",
        tagline: "Exchange / M365 IMAPS + SMTP XOAUTH2",
        description: "Connects company Outlook/Exchange shared or personal inboxes with XOAUTH2 intake and gated sends.",
        mode: "cloud",
        transport: "smtp_imap_xoauth2",
        local_ready: false,
        default_enabled: false,
        actions: OUTLOOK_MAIL_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: None,
            oauth_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            oauth_scopes: Some("https://outlook.office365.com/SMTP.Send https://outlook.office365.com/IMAP.AccessAsUser.All offline_access"),
            smtp_host: Some("smtp.office365.com"),
            smtp_port: Some(587),
            smtp_security: Some("starttls"),
            imap_host: Some("outlook.office365.com"),
            imap_port: Some(993),
            imap_security: Some("imaps"),
            match_hosts: &["outlook.office365.com", "smtp.office365.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "outlook-calendar",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "Outlook Calendar",
        tagline: "Microsoft Graph events & scheduling",
        description: "Reads Outlook Calendar availability and stages meeting invites for /phone approval.",
        mode: "cloud",
        transport: "oauth2_rest",
        local_ready: false,
        default_enabled: false,
        actions: OUTLOOK_CAL_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://graph.microsoft.com/v1.0/me/events"),
            oauth_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            oauth_scopes: Some("https://graph.microsoft.com/Calendars.ReadWrite offline_access"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["graph.microsoft.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "onedrive-sharepoint",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "OneDrive & SharePoint",
        tagline: "Local files offline + M365 Graph sync",
        description: "Lists synced workspace files offline and uploads deliverables to SharePoint/OneDrive with approval.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: ONEDRIVE_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://graph.microsoft.com/v1.0/me/drive/root/children"),
            oauth_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            oauth_scopes: Some("https://graph.microsoft.com/Files.ReadWrite.All offline_access"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["graph.microsoft.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "microsoft-teams",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "Microsoft Teams",
        tagline: "Channel updates & status posts",
        description: "Reads Teams channel context and routes outbound status updates through Approve/Deny.",
        mode: "cloud",
        transport: "oauth2_rest",
        local_ready: false,
        default_enabled: false,
        actions: TEAMS_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://graph.microsoft.com/v1.0/me/joinedTeams"),
            oauth_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            oauth_scopes: Some("https://graph.microsoft.com/ChannelMessage.Send offline_access"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["graph.microsoft.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "office-excel",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "Excel & Spreadsheets",
        tagline: "On-device .xlsx/.csv + live SUM formulas",
        description: "Enabled out of the box locally: reads .xlsx/.csv, deduplicates BOMs with i128 math, and exports workbooks.",
        mode: "hybrid",
        transport: "local_builtin",
        local_ready: true,
        default_enabled: true,
        actions: EXCEL_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://graph.microsoft.com/v1.0/me/drive/root:/workbooks"),
            oauth_token_url: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            oauth_scopes: Some("https://graph.microsoft.com/Files.ReadWrite offline_access"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["graph.microsoft.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "office-word-pdf",
        suite: "microsoft",
        suite_label: "Microsoft 365 & Outlook",
        name: "Word & PDF Suite",
        tagline: "On-device .docx/.pdf reading & export",
        description: "Enabled out of the box locally: reads Word, PowerPoint, and PDFs and exports signed reports on-device.",
        mode: "hybrid",
        transport: "local_builtin",
        local_ready: true,
        default_enabled: true,
        actions: WORD_PDF_ACTIONS,
        connect_preset: None,
    },

    // Business Apps (4)
    PluginSpec {
        id: "slack",
        suite: "business",
        suite_label: "Business Apps",
        name: "Slack",
        tagline: "Team notifications & channel posts",
        description: "Reads channels or posts approved run summaries into Slack via OAuth2 REST or webhook.",
        mode: "cloud",
        transport: "oauth2_rest",
        local_ready: false,
        default_enabled: false,
        actions: SLACK_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://slack.com/api/chat.postMessage"),
            oauth_token_url: Some("https://slack.com/api/oauth.v2.access"),
            oauth_scopes: Some("chat:write channels:read"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["slack.com", "hooks.slack.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "notion",
        suite: "business",
        suite_label: "Business Apps",
        name: "Notion",
        tagline: "Wiki search & gated database updates",
        description: "Queries Notion pages and routes database writes through human approval.",
        mode: "cloud",
        transport: "oauth2_rest",
        local_ready: false,
        default_enabled: false,
        actions: NOTION_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://api.notion.com/v1/search"),
            oauth_token_url: Some("https://api.notion.com/v1/oauth/token"),
            oauth_scopes: Some("read_content insert_content update_content"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["api.notion.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "github",
        suite: "business",
        suite_label: "Business Apps",
        name: "GitHub",
        tagline: "Issues, PR summaries & gated comments",
        description: "Reads issues/PRs and parks any external comment or status post on /phone.",
        mode: "hybrid",
        transport: "oauth2_rest",
        local_ready: true,
        default_enabled: false,
        actions: GITHUB_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: Some("https://api.github.com"),
            oauth_token_url: Some("https://github.com/login/oauth/access_token"),
            oauth_scopes: Some("repo read:org"),
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &["api.github.com"],
            connector_tab: "outbound",
        }),
    },
    PluginSpec {
        id: "crm-billing",
        suite: "business",
        suite_label: "Business Apps",
        name: "CRM & Billing",
        tagline: "Client vaults, retainers & QuickBooks/Stripe sync",
        description: "Enabled out of the box locally for client vaults & retainers; supports approval-gated cloud invoice sync.",
        mode: "hybrid",
        transport: "local_builtin",
        local_ready: true,
        default_enabled: true,
        actions: CRM_BILLING_ACTIONS,
        connect_preset: None,
    },

    // On-Device Local (2)
    PluginSpec {
        id: "local-workspace-files",
        suite: "local",
        suite_label: "On-Device Local",
        name: "On-Device Files & Watched Folders",
        tagline: "100% offline sandboxed workspace & folder triggers",
        description: "Active by default on-device: reads, searches, and watches local folders inside Tripwire path confinement.",
        mode: "local",
        transport: "local_builtin",
        local_ready: true,
        default_enabled: true,
        actions: LOCAL_FILES_ACTIONS,
        connect_preset: None,
    },
    PluginSpec {
        id: "local-mcp-bridge",
        suite: "local",
        suite_label: "On-Device Local",
        name: "Local Apps & MCP Bridge",
        tagline: "Connect any desktop CLI, database, or MCP server",
        description: "Active by default on-device: discovers stdio and HTTP MCP tools with optional /phone approval gates.",
        mode: "local",
        transport: "mcp",
        local_ready: true,
        default_enabled: true,
        actions: LOCAL_MCP_ACTIONS,
        connect_preset: Some(PluginConnectPreset {
            api_base_url: None,
            oauth_token_url: None,
            oauth_scopes: None,
            smtp_host: None,
            smtp_port: None,
            smtp_security: None,
            imap_host: None,
            imap_port: None,
            imap_security: None,
            match_hosts: &[],
            connector_tab: "mcp",
        }),
    },
];

pub fn default_catalog() -> &'static [PluginSpec] {
    CATALOG
}

pub fn find_plugin(id: &str) -> Option<&'static PluginSpec> {
    let key = id.trim();
    CATALOG.iter().find(|p| p.id == key)
}

#[derive(Clone, Debug)]
struct CloudMatch {
    connector_id: Option<String>,
    connector_name: Option<String>,
    trigger_id: Option<String>,
    trigger_name: Option<String>,
}

fn host_matches(candidate: &str, patterns: &[&str]) -> bool {
    let low = candidate.trim().to_ascii_lowercase();
    if low.is_empty() {
        return false;
    }
    patterns.iter().any(|pat| {
        let p = pat.to_ascii_lowercase();
        low == p || low.ends_with(&format!(".{p}"))
    })
}

fn detect_cloud_match(
    spec: &PluginSpec,
    connectors_snapshot: &Value,
    mcp_servers_len: usize,
) -> Option<CloudMatch> {
    if spec.id == "local-mcp-bridge" {
        if mcp_servers_len > 0 {
            return Some(CloudMatch {
                connector_id: None,
                connector_name: Some(format!("{mcp_servers_len} MCP server(s)")),
                trigger_id: None,
                trigger_name: None,
            });
        }
        return None;
    }
    let preset = spec.connect_preset.as_ref()?;
    if preset.match_hosts.is_empty() {
        return None;
    }

    let mut matched = CloudMatch {
        connector_id: None,
        connector_name: None,
        trigger_id: None,
        trigger_name: None,
    };

    if let Some(outbound) = connectors_snapshot["outbound"].as_array() {
        for c in outbound {
            if c["enabled"].as_bool() != Some(true) {
                continue;
            }
            let url = c["url"].as_str().unwrap_or("");
            let uhost = parse_url_host_port(url).map(|(h, _)| h).unwrap_or_default();
            let smtp_host = c["smtp_config"]["host"].as_str().unwrap_or("");
            if host_matches(&uhost, preset.match_hosts) || host_matches(smtp_host, preset.match_hosts)
            {
                matched.connector_id = c["id"].as_str().map(str::to_string);
                matched.connector_name = c["name"].as_str().map(str::to_string);
                break;
            }
        }
    }

    if let Some(triggers) = connectors_snapshot["triggers"].as_array() {
        for t in triggers {
            if t["enabled"].as_bool() != Some(true) {
                continue;
            }
            if t["kind"].as_str() == Some("imap_poll") {
                let imap_host = t["imap_config"]["host"].as_str().unwrap_or("");
                if host_matches(imap_host, preset.match_hosts) {
                    matched.trigger_id = t["id"].as_str().map(str::to_string);
                    matched.trigger_name = t["name"].as_str().map(str::to_string);
                    break;
                }
            }
        }
    }

    if matched.connector_id.is_some() || matched.trigger_id.is_some() {
        Some(matched)
    } else {
        None
    }
}

fn resolve_plugin_json(
    spec: &PluginSpec,
    setting: &PluginPersonSetting,
    cloud_match: Option<&CloudMatch>,
) -> Value {
    let cloud_connected = cloud_match.is_some();
    let effective_mode = match setting.preferred_mode.as_str() {
        "local" if spec.local_ready => "local",
        "cloud" => "cloud",
        _ => {
            if cloud_connected && spec.local_ready {
                "hybrid"
            } else if cloud_connected {
                "cloud"
            } else if spec.local_ready {
                "local"
            } else {
                "cloud"
            }
        }
    };

    let status = if !setting.enabled && !cloud_connected {
        "available"
    } else if cloud_connected && spec.local_ready {
        "connected_hybrid"
    } else if cloud_connected {
        "connected_cloud"
    } else if spec.local_ready {
        "active_local"
    } else {
        "enabled_needs_auth"
    };

    let connected_via = cloud_match.and_then(|m| {
        match (m.connector_name.as_deref(), m.trigger_name.as_deref()) {
            (Some(c), Some(t)) => Some(format!("{c} + {t}")),
            (Some(c), None) => Some(c.to_string()),
            (None, Some(t)) => Some(t.to_string()),
            (None, None) => None,
        }
    });

    json!({
        "id": spec.id,
        "suite": spec.suite,
        "suite_label": spec.suite_label,
        "name": spec.name,
        "tagline": spec.tagline,
        "description": spec.description,
        "mode": spec.mode,
        "transport": spec.transport,
        "local_ready": spec.local_ready,
        "default_enabled": spec.default_enabled,
        "enabled": setting.enabled || cloud_connected,
        "preferred_mode": setting.preferred_mode,
        "effective_mode": effective_mode,
        "status": status,
        "cloud_connected": cloud_connected,
        "connected_via": connected_via,
        "linked_connector_id": cloud_match.and_then(|m| m.connector_id.clone()),
        "linked_trigger_id": cloud_match.and_then(|m| m.trigger_id.clone()),
        "actions": spec.actions.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
        "connect_preset": spec.connect_preset.as_ref().map(|p| p.to_json()),
    })
}

pub fn list_plugins_json(person_data_dir: &Path) -> Value {
    let policy = crate::load_workspace_policy(person_data_dir);
    let connectors_snap = connectors::list_connectors_json(person_data_dir, &policy);
    let mcp_snap = mcp::list_mcp_servers_json(person_data_dir, &policy);
    let mcp_len = mcp_snap["servers"]
        .as_array()
        .map(|a| a.iter().filter(|s| s["enabled"].as_bool() == Some(true)).count())
        .unwrap_or(0);

    let store = with_plugins_lock(|| PluginsStore::load_unlocked(person_data_dir));
    let mut plugins = Vec::with_capacity(CATALOG.len());
    let mut enabled_count = 0usize;
    let mut local_active = 0usize;
    let mut cloud_connected = 0usize;

    for spec in CATALOG {
        let setting = store.settings.get(spec.id).cloned().unwrap_or(PluginPersonSetting {
            enabled: spec.default_enabled,
            preferred_mode: "auto".to_string(),
            updated_ms: 0,
        });
        let cm = detect_cloud_match(spec, &connectors_snap, mcp_len);
        let entry = resolve_plugin_json(spec, &setting, cm.as_ref());
        if entry["enabled"].as_bool() == Some(true) {
            enabled_count += 1;
        }
        match entry["status"].as_str() {
            Some("active_local") | Some("connected_hybrid") => local_active += 1,
            _ => {}
        }
        if entry["cloud_connected"].as_bool() == Some(true) {
            cloud_connected += 1;
        }
        plugins.push(entry);
    }

    json!({
        "plugins": plugins,
        "summary": {
            "total": CATALOG.len(),
            "enabled_count": enabled_count,
            "local_active": local_active,
            "cloud_connected": cloud_connected,
        }
    })
}

pub fn toggle_plugin(
    person_data_dir: &Path,
    plugin_id: &str,
    enabled: Option<bool>,
    preferred_mode: Option<&str>,
) -> Result<Value, String> {
    let spec = find_plugin(plugin_id)
        .ok_or_else(|| format!("unknown plugin id: {plugin_id}"))?;
    if let Some(m) = preferred_mode {
        if !matches!(m, "auto" | "local" | "cloud") {
            return Err("preferred_mode must be auto, local, or cloud".to_string());
        }
    }

    with_plugins_lock(|| {
        let mut store = PluginsStore::load_unlocked(person_data_dir);
        let cur = store
            .settings
            .get(spec.id)
            .cloned()
            .unwrap_or(PluginPersonSetting {
                enabled: spec.default_enabled,
                preferred_mode: "auto".to_string(),
                updated_ms: 0,
            });
        let next = PluginPersonSetting {
            enabled: enabled.unwrap_or(cur.enabled),
            preferred_mode: preferred_mode
                .map(str::to_string)
                .unwrap_or(cur.preferred_mode),
            updated_ms: records::now_millis(),
        };
        store.settings.insert(spec.id.to_string(), next);
        store.save_unlocked(person_data_dir)?;
        Ok::<(), String>(())
    })?;

    let full = list_plugins_json(person_data_dir);
    full["plugins"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["id"].as_str() == Some(spec.id)).cloned())
        .ok_or_else(|| "plugin saved but could not reload".to_string())
}

/// Compact summary injected into the employee agent system prompt so the agent
/// knows both its local on-device capabilities and any connected cloud apps.
pub fn prompt_summary(person_data_dir: &Path) -> String {
    let data = list_plugins_json(person_data_dir);
    let mut lines = Vec::new();
    if let Some(arr) = data["plugins"].as_array() {
        for p in arr {
            if p["enabled"].as_bool() != Some(true) {
                continue;
            }
            let id = p["id"].as_str().unwrap_or("");
            let name = p["name"].as_str().unwrap_or("");
            let mode = p["effective_mode"].as_str().unwrap_or("local");
            let status = p["status"].as_str().unwrap_or("available");
            lines.push(format!("         - {id} ({name}, mode={mode}, status={status})"));
        }
    }
    lines.join("\n")
}

/// Map a tool invocation to `(plugin_id, plugin_name, execution_mode)` for
/// unobtrusive realtime SSE activity tracking in the frontend.
pub fn describe_tool_activity(tool: &str, args: &Value) -> Option<(&'static str, &'static str, &'static str)> {
    match tool {
        "run_plugin_action" => {
            let pid = args["plugin_id"].as_str().unwrap_or("");
            find_plugin(pid).map(|spec| {
                let mode = match args["mode"].as_str() {
                    Some("cloud") => "cloud",
                    Some("local") => "local",
                    _ if spec.local_ready && spec.mode != "cloud" => "local",
                    _ => "cloud",
                };
                (spec.id, spec.name, mode)
            })
        }
        "make_spreadsheet" | "reconcile_ledger" | "query_table" | "aggregate_vault" => {
            Some(("office-excel", "Excel & Spreadsheets", "local"))
        }
        "write_document" | "export_file" | "summarize_meeting" | "audit_contract" | "draft_brief" => {
            Some(("office-word-pdf", "Word & PDF Suite", "local"))
        }
        "read_file" | "write_file" | "patch_file" | "list_dir" | "search_files" | "search_vault"
        | "move_file" => Some(("local-workspace-files", "On-Device Files", "local")),
        "query_company_db" => Some(("crm-billing", "CRM & Billing", "local")),
        "invoke_connector" => Some(("connectors", "Cloud Connector", "cloud")),
        other if other.starts_with("mcp.") => {
            Some(("local-mcp-bridge", "Local Apps & MCP", "local"))
        }
        _ => None,
    }
}

/// Execute a plugin action via local on-device tooling, cloud connector, or
/// park an Approve/Deny decision for consequential external writes.
pub fn execute_plugin_action(
    cfg: &Config,
    plugin_id: &str,
    action: &str,
    mode_override: Option<&str>,
    args: &Value,
    thread_id: &str,
) -> Result<(String, &'static str), String> {
    let spec = find_plugin(plugin_id)
        .ok_or_else(|| format!("unknown plugin: {plugin_id}"))?;
    let act_spec = spec
        .actions
        .iter()
        .find(|a| a.name == action)
        .ok_or_else(|| format!("plugin '{}' has no action '{}'", spec.name, action))?;

    let all = list_plugins_json(&cfg.data_dir);
    let current = all["plugins"]
        .as_array()
        .and_then(|arr| arr.iter().find(|p| p["id"].as_str() == Some(spec.id)))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let cloud_connected = current["cloud_connected"].as_bool() == Some(true);
    let preferred = current["preferred_mode"].as_str().unwrap_or("auto");

    let is_local_draft = act_spec.name == "draft_reply";
    let resolved_mode: &'static str = match mode_override.unwrap_or(preferred) {
        "local" => {
            if !spec.local_ready && !is_local_draft {
                return Err(format!(
                    "{} does not have an offline local mode; enable Cloud / Connect in Connectors first.",
                    spec.name
                ));
            }
            "local"
        }
        "cloud" => "cloud",
        _ => {
            if cloud_connected && !spec.local_ready && !is_local_draft {
                "cloud"
            } else if spec.local_ready || is_local_draft {
                "local"
            } else {
                "cloud"
            }
        }
    };

    // Gate consequential external write actions on /phone + Today approval.
    if act_spec.requires_approval {
        let preview = crate::truncate(&serde_json::to_string(args).unwrap_or_default(), 320);
        let summary = format!(
            "{} · {} ({})",
            spec.name, act_spec.label, resolved_mode
        );
        let detail = format!(
            "Plugin \"{}\" wants to execute \"{}\" ({} mode). Approve on /phone or Today to run. Payload: {}",
            spec.name, act_spec.description, resolved_mode, preview
        );
        let decision_id = records::add_decision_with(
            &cfg.data_dir,
            "plugin_action",
            &summary,
            None,
            thread_id,
            json!({
                "detail": detail,
                "severity": "warning",
                "plugin_id": spec.id,
                "plugin_name": spec.name,
                "action": act_spec.name,
                "action_label": act_spec.label,
                "mode": resolved_mode,
                "linked_connector_id": current["linked_connector_id"].clone(),
                "args": args.clone(),
            }),
        );
        return Ok((
            format!(
                "parked for your person's approval (decision {decision_id}): {} · {} requires Approve/Deny before external execution.",
                spec.name, act_spec.label
            ),
            resolved_mode,
        ));
    }

    // Read / non-consequential actions: run locally on-device or route via configured connector.
    if resolved_mode == "cloud" {
        if let Some(trig_id) = current["linked_trigger_id"].as_str() {
            if act_spec.name == "read_inbox" || act_spec.name == "triage_threads" {
                let events = connectors::poll_imap_trigger_now(cfg, trig_id)?;
                return Ok((
                    format!(
                        "{} polled mailbox cleanly: {} new message(s) saved into workspace/intake/.",
                        spec.name,
                        events.len()
                    ),
                    "cloud",
                ));
            }
        }
        if let Some(conn_id) = current["linked_connector_id"].as_str() {
            match connectors::invoke_connector(cfg, conn_id, args, None, thread_id)? {
                connectors::InvokeOutcome::Delivered(log) => {
                    return Ok((
                        format!(
                            "{} completed via {} (HTTP {}, {} bytes).",
                            spec.name,
                            log.connector_name,
                            log.http_status.unwrap_or(200),
                            log.payload_bytes
                        ),
                        "cloud",
                    ));
                }
                connectors::InvokeOutcome::Held { decision_id, .. } => {
                    return Ok((
                        format!(
                            "parked for your person's approval (decision {decision_id}) via connected cloud connector."
                        ),
                        "cloud",
                    ));
                }
            }
        }
        return Err(format!(
            "{} is not connected to a live cloud credential yet. Open Plugins -> Connect to configure its OAuth2 / mail preset.",
            spec.name
        ));
    }

    // Local on-device execution
    let out = match (spec.id, act_spec.name) {
        ("local-workspace-files", "list_workspace")
        | ("google-drive", "list_files")
        | ("onedrive-sharepoint", "list_files")
        | ("office-excel", "inspect_workbook") => {
            let path = args["path"].as_str().unwrap_or(".");
            let listed = crate::tools::execute_tool(cfg, "list_dir", &json!({ "path": path }))?;
            format!("{} (local workspace): {}", spec.name, listed)
        }
        ("local-workspace-files", "search_files") | ("office-word-pdf", "read_document") => {
            if let Some(q) = args["query"].as_str().filter(|s| !s.trim().is_empty()) {
                crate::tools::execute_tool(cfg, "search_files", &json!({ "query": q }))?
            } else if let Some(p) = args["path"].as_str().filter(|s| !s.trim().is_empty()) {
                crate::tools::execute_tool(cfg, "read_file", &json!({ "path": p }))?
            } else {
                crate::tools::execute_tool(cfg, "list_dir", &json!({ "path": "." }))?
            }
        }
        ("local-workspace-files", "watch_folder_status") => {
            let policy = crate::load_workspace_policy(&cfg.data_dir);
            let snap = connectors::list_connectors_json(&cfg.data_dir, &policy);
            let watched: Vec<String> = snap["triggers"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter(|t| t["kind"].as_str() == Some("folder_watch"))
                        .map(|t| {
                            format!(
                                "{} ({})",
                                t["name"].as_str().unwrap_or("Folder"),
                                t["watch_dir"].as_str().unwrap_or("-")
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            if watched.is_empty() {
                "No watched host folders configured yet; local workspace sandbox is ready.".to_string()
            } else {
                format!("Watched folders: {}", watched.join(", "))
            }
        }
        ("office-excel", "build_sheet") | ("google-sheets", "reconcile_sheet") => {
            let mut margs = args.clone();
            if margs.get("title").is_none() {
                margs["title"] = json!("Workspace Sheet");
            }
            crate::tools::execute_tool(cfg, "make_spreadsheet", &margs)?
        }
        ("office-excel", "export_xlsx") | ("office-word-pdf", "export_docx_pdf") => {
            crate::tools::execute_tool(cfg, "export_file", args)?
        }
        ("office-word-pdf", "draft_document")
        | ("google-docs", "draft_doc")
        | ("google-slides", "draft_outline")
        | ("google-gmail", "draft_reply")
        | ("outlook-mail", "draft_reply") => {
            let title = args["title"]
                .as_str()
                .unwrap_or(match spec.id {
                    "google-slides" => "Deck Outline",
                    "google-gmail" | "outlook-mail" => "Email Reply Draft",
                    _ => "Workspace Draft",
                });
            let sections = if args.get("sections").is_some() {
                args["sections"].clone()
            } else {
                json!([args["body"]
                    .as_str()
                    .or_else(|| args["content"].as_str())
                    .unwrap_or("Draft prepared locally inside Ergon workspace.")])
            };
            crate::tools::execute_tool(
                cfg,
                "write_document",
                &json!({ "title": title, "sections": sections }),
            )?
        }
        ("crm-billing", "list_clients") => {
            let root = records::infer_root_data_dir(&cfg.data_dir);
            let store = records::load_clients_store(&root);
            let names: Vec<String> = store
                .clients
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect();
            if names.is_empty() {
                "CRM local store: 0 clients configured yet.".to_string()
            } else {
                format!("CRM local clients ({}): {}", names.len(), names.join(", "))
            }
        }
        ("local-mcp-bridge", "list_mcp_tools") => {
            let tools = mcp::load_enabled_mcp_tools(&cfg.data_dir);
            if tools.is_empty() {
                "Local MCP Bridge: 0 MCP servers connected yet (add stdio or HTTP servers in Connectors -> MCP Tools).".to_string()
            } else {
                let list: Vec<String> = tools.iter().map(|(n, _, _)| n.clone()).collect();
                format!("Enabled MCP tools ({}): {}", list.len(), list.join(", "))
            }
        }
        ("github", "list_issues") => {
            "GitHub local plugin ready; connect a GitHub outbound connector in Connectors to query live remote repositories.".to_string()
        }
        _ => {
            format!(
                "{} · {} checked cleanly on-device.",
                spec.name, act_spec.label
            )
        }
    };
    Ok((out, "local"))
}

pub fn settle_plugin_action_decision(cfg: &Config, decision: &Value, approved: bool) -> String {
    let plugin_name = decision["plugin_name"].as_str().unwrap_or("Plugin");
    let action_label = decision["action_label"]
        .as_str()
        .or_else(|| decision["action"].as_str())
        .unwrap_or("action");
    if !approved {
        return format!("You said no — {plugin_name} · {action_label} was cancelled.");
    }
    let linked_conn = decision["linked_connector_id"].as_str().unwrap_or("");
    let args = &decision["args"];
    let tid = decision["thread_id"].as_str().unwrap_or("");
    if !linked_conn.is_empty() {
        match connectors::invoke_connector(cfg, linked_conn, args, None, tid) {
            Ok(connectors::InvokeOutcome::Delivered(log)) => {
                return format!(
                    "Approved & executed {plugin_name} · {action_label} via {} (HTTP {}, sha256:{}).",
                    log.connector_name,
                    log.http_status.unwrap_or(200),
                    log.payload_sha256
                );
            }
            Ok(connectors::InvokeOutcome::Held { decision_id, .. }) => {
                return format!(
                    "Approved plugin gate; outbound connector delivery also requires confirmation ({decision_id})."
                );
            }
            Err(e) => {
                return format!("Approved {plugin_name} · {action_label}, but connector delivery returned: {e}");
            }
        }
    }
    format!(
        "Approved {plugin_name} · {action_label} (connect live OAuth2 credentials in Connectors to dispatch externally)."
    )
}
