//! Proactive offers ("Would this help?"), per-person notifications, and push subscriptions.
//!
//! Stores are per-person JSON files under `<person_data_dir>` written atomically with `0600`
//! permissions and guarded by dedicated mutexes around load -> mutate -> save sequences.

use crate::library::{Library, LibraryItem, Suite};
use crate::records::{load_offers, load_work, now_millis, offers_path};
use crate::sse::RunEventBus;
use crate::tools::atomic_write_bytes_with_mode;
use crate::{call_model_with_usage, slugify, truncate, Config};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

const MAX_NOTIFICATIONS: usize = 200;
const WORK_SCAN_LIMIT: usize = 30;
const MODEL_DIGEST_LIMIT: usize = 15;

static NOTIF_LOCK: Mutex<()> = Mutex::new(());
static PUSH_SUB_LOCK: Mutex<()> = Mutex::new(());
static OFFERS_LOCK: Mutex<()> = Mutex::new(());
static NEXT_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_seq() -> u64 {
    NEXT_SEQ.fetch_add(1, Ordering::SeqCst)
}

fn read_json_array(path: &Path, key: &str) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    if let Some(items) = v[key].as_array() {
        return items.clone();
    }
    v.as_array().cloned().unwrap_or_default()
}

fn write_json_object_0600(path: &Path, payload: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(payload)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    atomic_write_bytes_with_mode(path, &bytes, Some(0o600))
        .map_err(|e| format!("write {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// 1. Per-person Notifications store (<person_data_dir>/notifications.json)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: String,
    pub ts_ms: u64,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub card_type: String,
    pub card_id: String,
    pub read: bool,
}

impl NotificationItem {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "ts_ms": self.ts_ms,
            "kind": self.kind,
            "title": self.title,
            "body": self.body,
            "card_type": self.card_type,
            "card_id": self.card_id,
            "read": self.read,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            id: v["id"].as_str()?.to_string(),
            ts_ms: v["ts_ms"].as_u64().unwrap_or(0),
            kind: v["kind"].as_str().unwrap_or("info").to_string(),
            title: v["title"].as_str().unwrap_or("").to_string(),
            body: v["body"].as_str().unwrap_or("").to_string(),
            card_type: v["card_type"].as_str().unwrap_or("").to_string(),
            card_id: v["card_id"].as_str().unwrap_or("").to_string(),
            read: v["read"].as_bool().unwrap_or(false),
        })
    }
}

pub fn notifications_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("notifications.json")
}

fn load_notifications_unlocked(person_data_dir: &Path) -> Vec<NotificationItem> {
    read_json_array(&notifications_path(person_data_dir), "notifications")
        .iter()
        .filter_map(NotificationItem::from_json)
        .collect()
}

fn save_notifications_unlocked(
    person_data_dir: &Path,
    items: &[NotificationItem],
) -> Result<(), String> {
    let arr: Vec<Value> = items.iter().map(NotificationItem::to_json).collect();
    write_json_object_0600(
        &notifications_path(person_data_dir),
        &json!({ "notifications": arr }),
    )
}

pub fn load_notifications(person_data_dir: &Path) -> Vec<NotificationItem> {
    let _guard = NOTIF_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    load_notifications_unlocked(person_data_dir)
}

pub fn mark_notification_read(
    person_data_dir: &Path,
    id: &str,
) -> Result<NotificationItem, String> {
    let target = id.trim();
    if target.is_empty() {
        return Err("notification id is required".to_string());
    }
    let _guard = NOTIF_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut items = load_notifications_unlocked(person_data_dir);
    let idx = items
        .iter()
        .position(|n| n.id == target)
        .ok_or_else(|| "notification not found".to_string())?;
    items[idx].read = true;
    let updated = items[idx].clone();
    save_notifications_unlocked(person_data_dir, &items)?;
    Ok(updated)
}

#[allow(clippy::too_many_arguments)]
pub fn emit_notification(
    person_data_dir: &Path,
    events: Option<&RunEventBus>,
    person_id: &str,
    kind: &str,
    title: &str,
    body: &str,
    card_type: &str,
    card_id: &str,
) -> NotificationItem {
    let ts_ms = now_millis();
    let item = NotificationItem {
        id: format!("notif-{ts_ms}-{}", next_seq()),
        ts_ms,
        kind: kind.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        card_type: card_type.to_string(),
        card_id: card_id.to_string(),
        read: false,
    };

    let unread_count = {
        let _guard = NOTIF_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let mut items = load_notifications_unlocked(person_data_dir);
        items.push(item.clone());
        if items.len() > MAX_NOTIFICATIONS {
            let excess = items.len() - MAX_NOTIFICATIONS;
            items.drain(..excess);
        }
        let unread = items.iter().filter(|n| !n.read).count();
        if let Err(e) = save_notifications_unlocked(person_data_dir, &items) {
            crate::logging::warn("ergon::proactive_offers", &e);
        }
        unread
    };

    if let Some(bus) = events {
        bus.publish(
            "push_notification",
            person_id,
            "",
            "",
            json!({
                "id": item.id,
                "ts_ms": item.ts_ms,
                "kind": item.kind,
                "title": item.title,
                "body": item.body,
                "card_type": item.card_type,
                "card_id": item.card_id,
                "unread_count": unread_count,
            }),
        );
    }

    item
}

// ---------------------------------------------------------------------------
// 2. Per-person Push Subscriptions store (<person_data_dir>/push_subscriptions.json)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushSubscription {
    pub id: String,
    pub channel: String,
    pub endpoint: String,
    pub label: String,
    pub created_ms: u64,
}

impl PushSubscription {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "channel": self.channel,
            "endpoint": self.endpoint,
            "label": self.label,
            "created_ms": self.created_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let channel = v["channel"].as_str()?;
        if !matches!(channel, "web_push" | "webhook") {
            return None;
        }
        Some(Self {
            id: v["id"].as_str()?.to_string(),
            channel: channel.to_string(),
            endpoint: v["endpoint"].as_str()?.to_string(),
            label: v["label"].as_str().unwrap_or("").to_string(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
        })
    }
}

pub fn push_subscriptions_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("push_subscriptions.json")
}

fn load_push_subscriptions_unlocked(person_data_dir: &Path) -> Vec<PushSubscription> {
    read_json_array(&push_subscriptions_path(person_data_dir), "subscriptions")
        .iter()
        .filter_map(PushSubscription::from_json)
        .collect()
}

fn save_push_subscriptions_unlocked(
    person_data_dir: &Path,
    subs: &[PushSubscription],
) -> Result<(), String> {
    let arr: Vec<Value> = subs.iter().map(PushSubscription::to_json).collect();
    write_json_object_0600(
        &push_subscriptions_path(person_data_dir),
        &json!({ "subscriptions": arr }),
    )
}

pub fn list_push_subscriptions(person_data_dir: &Path) -> Vec<PushSubscription> {
    let _guard = PUSH_SUB_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    load_push_subscriptions_unlocked(person_data_dir)
}

pub fn add_push_subscription(
    person_data_dir: &Path,
    channel: &str,
    endpoint: &str,
    label: &str,
) -> Result<PushSubscription, String> {
    let channel = channel.trim();
    let endpoint = endpoint.trim();
    let label = label.trim();

    if !matches!(channel, "web_push" | "webhook") {
        return Err("push channel must be \"web_push\" or \"webhook\"".to_string());
    }
    if endpoint.is_empty()
        || endpoint
            .chars()
            .any(|c| c.is_control() || c.is_whitespace())
    {
        return Err("push endpoint must be a valid non-empty URL".to_string());
    }
    match channel {
        "webhook" => {
            crate::tripwire::check_egress_for(person_data_dir, endpoint, "push")?;
        }
        "web_push" => {
            if endpoint != "browser://local" {
                if !endpoint.to_ascii_lowercase().starts_with("https://") {
                    return Err(
                        "web_push endpoint must be \"browser://local\" or an https:// URL"
                            .to_string(),
                    );
                }
                crate::tripwire::check_egress_for(person_data_dir, endpoint, "push")?;
            }
        }
        _ => unreachable!(),
    }

    let _guard = PUSH_SUB_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut subs = load_push_subscriptions_unlocked(person_data_dir);
    if let Some(existing) = subs
        .iter_mut()
        .find(|s| s.channel == channel && s.endpoint == endpoint)
    {
        if !label.is_empty() {
            existing.label = label.to_string();
        }
        let updated = existing.clone();
        save_push_subscriptions_unlocked(person_data_dir, &subs)?;
        return Ok(updated);
    }

    let created_ms = now_millis();
    let sub = PushSubscription {
        id: format!("psub-{created_ms}-{}", next_seq()),
        channel: channel.to_string(),
        endpoint: endpoint.to_string(),
        label: if label.is_empty() {
            channel.to_string()
        } else {
            label.to_string()
        },
        created_ms,
    };
    subs.push(sub.clone());
    save_push_subscriptions_unlocked(person_data_dir, &subs)?;
    Ok(sub)
}

pub fn remove_push_subscription(person_data_dir: &Path, id: &str) -> Result<bool, String> {
    let target = id.trim();
    if target.is_empty() {
        return Err("subscription id is required".to_string());
    }
    let _guard = PUSH_SUB_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut subs = load_push_subscriptions_unlocked(person_data_dir);
    let before = subs.len();
    subs.retain(|s| s.id != target);
    if subs.len() == before {
        return Ok(false);
    }
    save_push_subscriptions_unlocked(person_data_dir, &subs)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// 3. Proactive Offer Analyzer ("Would this help?")
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ProactiveOfferAnalysis {
    pub generated: Vec<Value>,
    pub all_offers: Vec<Value>,
    pub proposed_tool: Option<Value>,
}

impl ProactiveOfferAnalysis {
    pub fn to_json(&self) -> Value {
        json!({
            "generated": self.generated,
            "all_offers": self.all_offers,
            "proposed_tool": self.proposed_tool,
        })
    }
}

const KEYWORD_STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "from", "this", "that", "into", "your", "our", "are", "was",
    "were", "will", "can", "all", "any", "not", "but", "use", "using", "used", "tool", "skill",
    "helper", "task", "tasks", "work", "file", "files", "data", "make", "create", "build", "run",
    "runs", "step", "steps", "output", "input",
];

fn tokenize_keywords(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if raw.len() < 3 || KEYWORD_STOPWORDS.contains(&raw) {
            continue;
        }
        if seen.insert(raw.to_string()) {
            out.push(raw.to_string());
        }
    }
    out
}

fn item_keywords(item: &LibraryItem) -> Vec<String> {
    let combined = format!("{} {} {}", item.id, item.name, item.description);
    tokenize_keywords(&combined)
}

fn recent_finished_work(work_all: &[Value]) -> Vec<&Value> {
    work_all
        .iter()
        .rev()
        .take(WORK_SCAN_LIMIT)
        .filter(|e| e["finished"].as_bool() != Some(false))
        .filter(|e| e["answered"].as_bool() != Some(true))
        .collect()
}

fn count_matching_runs(item_kw: &[String], work_tokens: &[HashSet<String>]) -> usize {
    if item_kw.is_empty() {
        return 0;
    }
    let min_hits = item_kw.len().min(2);
    work_tokens
        .iter()
        .filter(|tokens| {
            let hits = item_kw.iter().filter(|kw| tokens.contains(*kw)).count();
            hits >= min_hits
        })
        .count()
}

fn save_offers_0600(person_data_dir: &Path, offers: &[Value]) -> Result<(), String> {
    write_json_object_0600(&offers_path(person_data_dir), &json!({ "offers": offers }))
}

fn known_offer_keys(offers: &[Value]) -> HashSet<String> {
    offers
        .iter()
        .filter_map(|o| o["key"].as_str().map(|s| s.trim().to_string()))
        .filter(|k| !k.is_empty())
        .collect()
}

fn ensure_would_this_help(title: &str) -> String {
    let t = title.trim();
    if t.is_empty() {
        "Would this help?".to_string()
    } else if t.to_ascii_lowercase().starts_with("would this help?") {
        t.to_string()
    } else {
        format!("Would this help? {t}")
    }
}

fn sanitize_user_text(s: &str) -> String {
    // Never surface internal engine names to end users.
    let mut out = s.to_string();
    for bad in ["Rune", "rune"] {
        if out.contains(bad) {
            out = out.replace(bad, "workflow");
        }
    }
    out
}

fn redact_for_model(person_data_dir: &Path, raw: &str) -> String {
    let person_redacted = crate::connectors::redact_for_person(person_data_dir, raw);
    crate::tripwire::redact(&person_redacted)
}

fn build_model_messages(
    person_data_dir: &Path,
    recent_work: &[&Value],
    known_keys: &HashSet<String>,
) -> Vec<Value> {
    let digest: Vec<Value> = recent_work
        .iter()
        .take(MODEL_DIGEST_LIMIT)
        .map(|entry| {
            let task = redact_for_model(person_data_dir, entry["task"].as_str().unwrap_or(""));
            let summary =
                redact_for_model(person_data_dir, entry["summary"].as_str().unwrap_or(""));
            let files: Vec<String> = entry["files"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| f.as_str())
                        .take(8)
                        .map(|f| redact_for_model(person_data_dir, f))
                        .collect()
                })
                .unwrap_or_default();
            json!({
                "task": truncate(&task, 200),
                "summary": truncate(&summary, 280),
                "files": files,
            })
        })
        .collect();

    let mut existing: Vec<&str> = known_keys.iter().map(String::as_str).collect();
    existing.sort_unstable();

    let system = concat!(
        "You analyze a person's recent finished work log and decide whether ONE proactive helper ",
        "(\"Would this help?\") would save them real time.\n",
        "Rules:\n",
        "- Never mention internal scripting engine names.\n",
        "- Only propose an offer if there is a clear repeated workflow (>= 2 similar tasks or outputs).\n",
        "- Never reuse any key listed in existing_keys.\n",
        "- Reply with strictly valid JSON matching ONE of these two shapes:\n",
        "  {\"offer\": null}\n",
        "  OR\n",
        "  {\"offer\": {\"key\": \"short-slug\", \"title\": \"Would this help? ...\", \"detail\": \"...\", ",
        "\"kind\": \"repeat_task\"|\"build_tool\", \"template\": \"...\", \"tool_name\": \"...\", ",
        "\"tool_description\": \"...\", \"params\": [\"...\"], \"source\": \"...\", \"tests\": [...]}}\n",
        "- For kind=\"repeat_task\", provide a non-empty \"template\" prompt.\n",
        "- For kind=\"build_tool\", provide a self-contained sandboxed workflow script in \"source\" ",
        "with `pub fn main(...)` using only `host_workspace_read`, `host_workspace_write`, and ",
        "`host_workspace_list`, plus at least 1 test case in \"tests\" (`{\"name\":..., \"setup_files\":{}, ",
        "\"args\":[...], \"expect_ok\":true, \"expect_output_contains\":..., \"expect_files\":{}}`)."
    );

    let user = json!({
        "existing_keys": existing,
        "recent_work": digest,
    })
    .to_string();

    vec![
        json!({ "role": "system", "content": system }),
        json!({ "role": "user", "content": user }),
    ]
}

/// Scans cross-team library tools/skills and (when a model is configured) recent work logs
/// to generate deduplicated "Would this help?" offers and notifications.
pub fn analyze_proactive_offers(
    cfg: &Config,
    person_id: &str,
    suite: &Suite,
    library: &Library,
    events: Option<&RunEventBus>,
) -> ProactiveOfferAnalysis {
    let work_all = load_work(&cfg.data_dir);
    let recent = recent_finished_work(&work_all);
    let work_tokens: Vec<HashSet<String>> = recent
        .iter()
        .map(|e| {
            let text = format!(
                "{} {}",
                e["task"].as_str().unwrap_or(""),
                e["summary"].as_str().unwrap_or("")
            );
            tokenize_keywords(&text).into_iter().collect()
        })
        .collect();

    let mut generated: Vec<Value> = Vec::new();
    let mut pending_notifs: Vec<(String, String, String)> = Vec::new();
    let rec_snapshot = crate::plugins::recommendations_json(&cfg.data_dir, false);
    let high_plugin_recs: Vec<Value> = rec_snapshot["recommendations"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|r| r["confidence"].as_str() == Some("high"))
                .take(2)
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    // Step 1: Cross-team library tool/skill + high-confidence app setup matching under OFFERS_LOCK.
    let known_after_lib = {
        let _guard = OFFERS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let mut offers = load_offers(&cfg.data_dir);
        let mut known = known_offer_keys(&offers);
        let mut changed = false;
        let ts = now_millis();

        for item in &library.items {
            if item.author == person_id
                || item.builtin
                || suite.installed.iter().any(|id| id == &item.id)
                || !matches!(item.kind.as_str(), "tool" | "skill")
            {
                continue;
            }
            let key = format!("libtool:{}", item.id);
            if known.contains(&key) {
                continue;
            }
            let kw = item_keywords(item);
            let matches = count_matching_runs(&kw, &work_tokens);
            if matches < 2 {
                continue;
            }

            let id = format!("o{ts}-{}", next_seq());
            let title = sanitize_user_text(&format!(
                "Would this help? Use {} ({})",
                item.name, item.kind
            ));
            let detail = sanitize_user_text(&format!(
                "Shared by {} — {}. Matched {} recent finished jobs; add it to your suite?",
                item.author,
                truncate(&item.description, 160),
                matches
            ));
            let offer = json!({
                "id": id,
                "ts": ts,
                "kind": "install_library_tool",
                "key": key,
                "item_id": item.id,
                "author": item.author,
                "title": title,
                "detail": detail,
                "status": "pending",
            });
            offers.push(offer.clone());
            generated.push(offer);
            pending_notifs.push((id, title, detail));
            known.insert(key);
            changed = true;
        }

        for rec in &high_plugin_recs {
            let pid = rec["plugin_id"].as_str().unwrap_or("");
            let name = rec["name"].as_str().unwrap_or(pid);
            if pid.is_empty() {
                continue;
            }
            let key = format!("plugin_setup:{pid}");
            if known.contains(&key) {
                continue;
            }
            let reason = rec["reason_summary"]
                .as_str()
                .unwrap_or("Recognized from your recent work");
            let setup_kind = rec["setup_kind"].as_str().unwrap_or("one_click_connect");
            let setup_label = rec["setup_label"].as_str().unwrap_or("Set up app");
            let id = format!("o{ts}-{}", next_seq());
            let title = sanitize_user_text(&format!("Would this help? Connect {name}"));
            let detail = sanitize_user_text(&format!(
                "{reason}. Connect {name} so your assistant can work with it directly."
            ));
            let offer = json!({
                "id": id,
                "ts": ts,
                "kind": "plugin_setup",
                "key": key,
                "plugin_id": pid,
                "setup_kind": setup_kind,
                "setup_label": setup_label,
                "title": title,
                "detail": detail,
                "status": "pending",
            });
            offers.push(offer.clone());
            generated.push(offer);
            pending_notifs.push((id, title, detail));
            known.insert(key);
            changed = true;
        }

        if changed {
            if let Err(e) = save_offers_0600(&cfg.data_dir, &offers) {
                crate::logging::warn("ergon::proactive_offers", &e);
            }
        }
        known
    };

    for (offer_id, title, detail) in pending_notifs {
        emit_notification(
            &cfg.data_dir,
            events,
            person_id,
            "proactive_offer",
            &title,
            &detail,
            "offer",
            &offer_id,
        );
    }

    // Step 2: Model-driven proactive offer (never hold any file lock across the model call).
    let mut proposed_tool: Option<Value> = None;
    if cfg.model_configured() && recent.len() >= 2 {
        let messages = build_model_messages(&cfg.data_dir, &recent, &known_after_lib);
        match call_model_with_usage(cfg, "ergon-proactive-offers", &messages) {
            Ok((reply, _usage)) => {
                if let Some(offer_spec) = reply.get("offer").and_then(|v| v.as_object()) {
                    let raw_key = offer_spec
                        .get("key")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.trim().is_empty())
                        .or_else(|| offer_spec.get("tool_name").and_then(|v| v.as_str()))
                        .or_else(|| offer_spec.get("title").and_then(|v| v.as_str()))
                        .unwrap_or("");
                    let key = slugify(raw_key);
                    let kind = offer_spec
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("repeat_task");
                    let title = sanitize_user_text(&ensure_would_this_help(
                        offer_spec
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Automate a repeated task"),
                    ));
                    let detail = sanitize_user_text(
                        offer_spec
                            .get("detail")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Based on your recent work, this could save you a step."),
                    );

                    if !key.is_empty() && !known_after_lib.contains(&key) {
                        if kind == "build_tool" {
                            let source = offer_spec
                                .get("source")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .trim();
                            let tests = offer_spec
                                .get("tests")
                                .and_then(|v| v.as_array())
                                .filter(|a| !a.is_empty());
                            if !source.is_empty() && tests.is_some() {
                                let tool_name = sanitize_user_text(
                                    offer_spec
                                        .get("tool_name")
                                        .and_then(|v| v.as_str())
                                        .filter(|s| !s.trim().is_empty())
                                        .unwrap_or(&key),
                                );
                                let tool_description = sanitize_user_text(
                                    offer_spec
                                        .get("tool_description")
                                        .and_then(|v| v.as_str())
                                        .filter(|s| !s.trim().is_empty())
                                        .unwrap_or(&detail),
                                );
                                proposed_tool = Some(json!({
                                    "key": key,
                                    "kind": "build_tool",
                                    "title": title,
                                    "detail": detail,
                                    "tool_name": tool_name,
                                    "tool_description": tool_description,
                                    "params": offer_spec.get("params").cloned().unwrap_or_else(|| json!([])),
                                    "source": source,
                                    "tests": Value::Array(tests.unwrap().clone()),
                                }));
                            }
                        } else if kind == "repeat_task" {
                            let template = sanitize_user_text(
                                offer_spec
                                    .get("template")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .trim(),
                            );
                            if !template.is_empty() {
                                let mut created: Option<(String, Value)> = None;
                                {
                                    let _guard =
                                        OFFERS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
                                    let mut offers = load_offers(&cfg.data_dir);
                                    let known = known_offer_keys(&offers);
                                    if !known.contains(&key) {
                                        let ts = now_millis();
                                        let id = format!("o{ts}-{}", next_seq());
                                        let offer = json!({
                                            "id": id,
                                            "ts": ts,
                                            "kind": "repeat_task",
                                            "key": key,
                                            "title": title,
                                            "detail": detail,
                                            "template": template,
                                            "status": "pending",
                                        });
                                        offers.push(offer.clone());
                                        if let Err(e) = save_offers_0600(&cfg.data_dir, &offers) {
                                            crate::logging::warn("ergon::proactive_offers", &e);
                                        }
                                        created = Some((id, offer));
                                    }
                                }
                                if let Some((offer_id, offer)) = created {
                                    generated.push(offer);
                                    emit_notification(
                                        &cfg.data_dir,
                                        events,
                                        person_id,
                                        "proactive_offer",
                                        &title,
                                        &detail,
                                        "offer",
                                        &offer_id,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                crate::logging::warn(
                    "ergon::proactive_offers",
                    &format!("proactive offer model call skipped: {e}"),
                );
            }
        }
    }

    let all_offers = {
        let _guard = OFFERS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        load_offers(&cfg.data_dir)
    };

    ProactiveOfferAnalysis {
        generated,
        all_offers,
        proposed_tool,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ergon-proactive-test-{label}-{}-{}",
            std::process::id(),
            next_seq()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_cfg(data_dir: &Path) -> Config {
        Config {
            data_dir: data_dir.to_path_buf(),
            workspace_dir: data_dir.join("workspace"),
            memory_path: data_dir.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test-model".to_string(),
            api_key: None,
        }
    }

    fn make_skill_item(id: &str, name: &str, description: &str, author: &str) -> LibraryItem {
        LibraryItem::from_json(&json!({
            "id": id,
            "kind": "skill",
            "name": name,
            "description": description,
            "author": author,
            "version": 1,
            "builtin": false,
            "params": ["path"],
            "source": "pub fn main(p) { \"ok\" }"
        }))
        .unwrap()
    }

    fn write_work_entries(data_dir: &Path, entries: &[Value]) {
        write_json_object_0600(&data_dir.join("work.json"), &json!({ "entries": entries }))
            .unwrap();
    }

    #[test]
    fn notifications_emit_read_and_cap_200() {
        let dir = temp_dir("notif");
        let bus = RunEventBus::new();

        let n1 = emit_notification(
            &dir,
            Some(&bus),
            "alice",
            "proactive_offer",
            "Would this help?",
            "Try the invoice helper",
            "offer",
            "o-1",
        );
        assert!(!n1.read);
        let events = bus.wait_since(0, Duration::from_millis(10));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, "push_notification");
        assert_eq!(events[0].person_id, "alice");
        assert_eq!(events[0].data["unread_count"].as_u64(), Some(1));
        assert_eq!(events[0].data["card_id"].as_str(), Some("o-1"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(notifications_path(&dir))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let marked = mark_notification_read(&dir, &n1.id).unwrap();
        assert!(marked.read);
        assert!(mark_notification_read(&dir, "missing-id").is_err());

        for i in 0..205 {
            emit_notification(
                &dir,
                None,
                "alice",
                "info",
                &format!("Title {i}"),
                "body",
                "offer",
                &format!("o-{i}"),
            );
        }
        let all = load_notifications(&dir);
        assert_eq!(all.len(), MAX_NOTIFICATIONS);
        assert_eq!(all.last().unwrap().title, "Title 204");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn push_subscriptions_crud_and_egress_guard() {
        let dir = temp_dir("push");

        let s1 = add_push_subscription(&dir, "web_push", "browser://local", "Phone").unwrap();
        assert_eq!(s1.channel, "web_push");
        assert_eq!(s1.endpoint, "browser://local");

        // Re-registering same endpoint updates label instead of duplicating.
        let s1_dup =
            add_push_subscription(&dir, "web_push", "browser://local", "Work Phone").unwrap();
        assert_eq!(s1_dup.id, s1.id);
        assert_eq!(s1_dup.label, "Work Phone");

        let s2 = add_push_subscription(
            &dir,
            "webhook",
            "https://hooks.example.com/ergon",
            "Team Relay",
        )
        .unwrap();
        assert_eq!(s2.channel, "webhook");

        // Loopback / metadata / bad schemes must be rejected.
        assert!(
            add_push_subscription(&dir, "webhook", "http://127.0.0.1:9000/hook", "bad").is_err()
        );
        assert!(
            add_push_subscription(&dir, "webhook", "http://169.254.169.254/latest", "bad").is_err()
        );
        assert!(add_push_subscription(&dir, "sms", "https://example.com", "bad").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(push_subscriptions_path(&dir))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let listed = list_push_subscriptions(&dir);
        assert_eq!(listed.len(), 2);

        assert!(remove_push_subscription(&dir, &s1.id).unwrap());
        assert!(!remove_push_subscription(&dir, &s1.id).unwrap());
        assert_eq!(list_push_subscriptions(&dir).len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cross_team_library_offer_matching_and_dismissed_dedup() {
        let dir = temp_dir("libmatch");
        let cfg = test_cfg(&dir);
        write_work_entries(
            &dir,
            &[
                json!({
                    "task": "Reconcile vendor invoice CSV ledger",
                    "summary": "Matched invoice rows against CSV export",
                    "files": ["invoices.csv"],
                    "finished": true,
                    "answered": false
                }),
                json!({
                    "task": "Audit October invoice CSV totals",
                    "summary": "Checked invoice amounts in CSV sheet",
                    "files": ["oct.csv"],
                    "finished": true,
                    "answered": false
                }),
            ],
        );

        let lib = Library {
            items: vec![
                // Another colleague's skill that matches >= 2 runs -> should trigger offer.
                make_skill_item(
                    "invoice-csv-helper",
                    "Invoice CSV Reconciler",
                    "Reconciles vendor invoice CSV totals automatically",
                    "bob",
                ),
                // Own item -> must NOT be offered.
                make_skill_item(
                    "alice-invoice-csv",
                    "My Invoice CSV",
                    "Invoice CSV helper",
                    "alice",
                ),
                // Already installed -> must NOT be offered.
                make_skill_item(
                    "installed-invoice-csv",
                    "Installed Invoice CSV",
                    "Invoice CSV helper",
                    "carol",
                ),
                // Unrelated item -> must NOT be offered.
                make_skill_item(
                    "travel-itinerary",
                    "Travel Itinerary Planner",
                    "Builds flight and hotel schedules",
                    "carol",
                ),
            ],
            recovered: None,
        };
        let suite = Suite {
            installed: vec!["installed-invoice-csv".to_string()],
        };
        let bus = RunEventBus::new();

        let res1 = analyze_proactive_offers(&cfg, "alice", &suite, &lib, Some(&bus));
        assert_eq!(res1.generated.len(), 1);
        assert_eq!(
            res1.generated[0]["kind"].as_str(),
            Some("install_library_tool")
        );
        assert_eq!(
            res1.generated[0]["item_id"].as_str(),
            Some("invoice-csv-helper")
        );
        assert_eq!(res1.generated[0]["author"].as_str(), Some("bob"));
        assert_eq!(
            res1.generated[0]["key"].as_str(),
            Some("libtool:invoice-csv-helper")
        );
        assert!(res1.generated[0]["title"]
            .as_str()
            .unwrap()
            .starts_with("Would this help?"));
        assert_eq!(load_notifications(&dir).len(), 1);

        // Dismiss the offer and verify re-running analyzer does NOT re-offer ("a no is a no").
        let mut offers = load_offers(&dir);
        offers[0]["status"] = json!("dismissed");
        save_offers_0600(&dir, &offers).unwrap();

        let res2 = analyze_proactive_offers(&cfg, "alice", &suite, &lib, Some(&bus));
        assert!(res2.generated.is_empty());
        assert_eq!(res2.all_offers.len(), 1);
        assert_eq!(res2.all_offers[0]["status"].as_str(), Some("dismissed"));
        assert_eq!(load_notifications(&dir).len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
