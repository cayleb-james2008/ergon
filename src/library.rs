//! The company library and per-person suites.
//!
//! ONE shared library of subagents, skills and apps lives at
//! `<root data dir>/library.json`. Every person's suite is drawn from
//! it; agents themselves maintain it (an agent can publish a skill it
//! wrote, and any other person's agent can install and run it).
//!
//! Item kinds:
//! - `skill` — a Rune script. Library skills follow one convention:
//!   `pub fn main(p1, p2, ...)` taking the item's declared `params` as
//!   strings, in order, and returning a string (plain result text, or
//!   `file:<name>` when the skill wrote a workspace file). Side effects
//!   go through the same sandboxed host functions as the built-ins.
//! - `subagent` — an instruction profile the agent can delegate to.
//! - `app` — a small purpose-built component: a task `template` with
//!   `{field}` placeholders plus the `fields` it asks for. Running an
//!   app hands the filled template to the person's agent as a task.
//!
//! The four built-in skills (write_document, make_spreadsheet,
//! research, update_memory) are listed as built-in items: they are
//! part of every agent already and are NOT run through the library
//! runner (their signatures predate the library convention).

use crate::{backed_up_name, slugify, Person};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct LibraryItem {
    pub id: String,
    pub kind: String, // "skill" | "subagent" | "app" | "tool"
    pub name: String,
    pub description: String,
    pub author: String, // person id, or "ergon" for seeds
    pub version: u32,
    pub builtin: bool,
    pub params: Vec<String>, // skill/tool: ordered parameter names
    pub source: String,      // skill/tool: Rune source
    pub instructions: String, // subagent: system instructions
    pub template: String,    // app: task template with {field} slots
    pub fields: Vec<String>, // app: input field names
    pub tests: Vec<Value>,   // tool: test case definitions
    pub test_report: Option<Value>, // tool/evolution: latest verification report
}

impl LibraryItem {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind,
            "name": self.name,
            "description": self.description,
            "author": self.author,
            "version": self.version,
            "builtin": self.builtin,
            "params": self.params,
            "instructions": self.instructions,
            "template": self.template,
            "fields": self.fields,
            // The Rune source is served too: the library is the
            // company's shared knowledge, and a person inspecting an
            // item should see exactly what would run.
            "source": self.source,
            "tests": self.tests,
            "test_report": self.test_report,
        })
    }

    pub fn from_json(v: &Value) -> Option<LibraryItem> {
        let kind = v["kind"].as_str()?.to_string();
        if !matches!(kind.as_str(), "skill" | "subagent" | "app" | "tool") {
            return None;
        }
        let strings = |key: &str| -> Vec<String> {
            v[key]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|i| i.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default()
        };
        let tests = v["tests"].as_array().cloned().unwrap_or_default();
        let test_report = v.get("test_report").filter(|r| !r.is_null()).cloned();
        Some(LibraryItem {
            id: v["id"].as_str()?.to_string(),
            kind,
            name: v["name"].as_str().unwrap_or("").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
            author: v["author"].as_str().unwrap_or("ergon").to_string(),
            version: v["version"].as_u64().unwrap_or(1) as u32,
            builtin: v["builtin"].as_bool().unwrap_or(false),
            params: strings("params"),
            source: v["source"].as_str().unwrap_or("").to_string(),
            instructions: v["instructions"].as_str().unwrap_or("").to_string(),
            template: v["template"].as_str().unwrap_or("").to_string(),
            fields: strings("fields"),
            tests,
            test_report,
        })
    }
}

fn item(
    id: &str,
    kind: &str,
    name: &str,
    description: &str,
    author: &str,
) -> LibraryItem {
    LibraryItem {
        id: id.to_string(),
        kind: kind.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        author: author.to_string(),
        version: 1,
        builtin: false,
        params: Vec::new(),
        source: String::new(),
        instructions: String::new(),
        template: String::new(),
        fields: Vec::new(),
        tests: Vec::new(),
        test_report: None,
    }
}

/// The seed library: what a fresh install already carries, so a real
/// install is the adapted state, not a clean slate. The built-in
/// skills come first, then working library-native items.
pub fn seed_items() -> Vec<LibraryItem> {
    let mut out: Vec<LibraryItem> = Vec::new();

    for meta in crate::skill_catalog::all_skills() {
        let mut it = item(
            &meta.slug,
            "skill",
            &meta.title,
            &meta.description,
            "ergon",
        );
        it.builtin = true;
        it.params = meta.param_names();
        it.source = meta.source;
        out.push(it);
    }

    // Library-native skills (runnable through the library runner).
    let mut daily_log = item(
        "daily-log",
        "skill",
        "Daily log",
        "Appends one entry line to the person's Daily Log file in the workspace.",
        "ergon",
    );
    daily_log.params = vec!["entry".to_string()];
    daily_log.source = r##"//! Ergon library skill: daily_log (Rune)
//! Appends one entry to "Daily Log.md" in the workspace.

pub fn main(entry) {
    let existing = host_workspace_read("Daily Log.md");
    let body = if existing.starts_with("error:") {
        "# Daily Log\n\n"
    } else {
        existing
    };
    let updated = `${body}- ${entry}\n`;
    let result = host_workspace_write("Daily Log.md", updated);
    if result != "ok" {
        return result;
    }
    "file:Daily Log.md"
}
"##
    .to_string();
    out.push(daily_log);

    let mut reading_list = item(
        "reading-list",
        "skill",
        "Reading list",
        "Adds one titled link to the person's Reading List file in the workspace.",
        "ergon",
    );
    reading_list.params = vec!["title".to_string(), "url".to_string()];
    reading_list.source = r##"//! Ergon library skill: reading_list (Rune)
//! Appends one titled link to "Reading List.md" in the workspace.

pub fn main(title, url) {
    let existing = host_workspace_read("Reading List.md");
    let body = if existing.starts_with("error:") {
        "# Reading List\n\n"
    } else {
        existing
    };
    let updated = `${body}- ${title} - ${url}\n`;
    let result = host_workspace_write("Reading List.md", updated);
    if result != "ok" {
        return result;
    }
    "file:Reading List.md"
}
"##
    .to_string();
    out.push(reading_list);

    // Subagent profiles.
    let mut researcher = item(
        "researcher",
        "subagent",
        "Researcher",
        "Digs through files and fetched pages and comes back with sourced, organised notes.",
        "ergon",
    );
    researcher.instructions = "You are a research specialist working for the person's main agent. \
        Be thorough: search the workspace, fetch the pages you need, and read carefully before \
        concluding. Return organised notes with the sources you used named explicitly. Never \
        claim a source you did not actually read through a tool."
        .to_string();
    out.push(researcher);

    let mut editor = item(
        "editor",
        "subagent",
        "Editor",
        "Tightens writing: plain, professional, no bravado, facts stated simply.",
        "ergon",
    );
    editor.instructions = "You are an editor working for the person's main agent. Rewrite for \
        clarity and plain professional tone: short sentences, no slogans, no figurative \
        flourishes, no sales language. Keep every fact from the original; change wording, \
        never meaning. Produce the finished text as a file when asked for a document."
        .to_string();
    out.push(editor);

    let mut bookkeeper = item(
        "bookkeeper",
        "subagent",
        "Bookkeeper",
        "Turns lists of expenses and figures into clean, checked spreadsheets.",
        "ergon",
    );
    bookkeeper.instructions = "You are a bookkeeping specialist working for the person's main \
        agent. Turn expense lists and figures into clean spreadsheets with clear columns. \
        Check every total twice against the source figures and say plainly if a figure is \
        missing or does not add up — never invent a number to make a column balance."
        .to_string();
    out.push(bookkeeper);

    // Apps (purpose-built components).
    let mut meeting = item(
        "meeting-actions",
        "app",
        "Meeting actions",
        "Reads a meeting notes file and writes the decisions and action items as a document.",
        "ergon",
    );
    meeting.fields = vec!["notes_file".to_string()];
    meeting.template = "Read {notes_file} and write a document of the decisions and action \
        items from it, with owners where the notes name them."
        .to_string();
    out.push(meeting);

    let mut expenses = item(
        "expense-sheet",
        "app",
        "Expense sheet",
        "Turns a file of expenses into a clean spreadsheet with Item and Cost columns.",
        "ergon",
    );
    expenses.fields = vec!["source_file".to_string()];
    expenses.template = "Read {source_file} and turn the expenses in it into a spreadsheet \
        with columns Item and Cost, using the make_spreadsheet tool."
        .to_string();
    out.push(expenses);

    let mut weekly = item(
        "weekly-summary",
        "app",
        "Weekly summary",
        "Looks through the workspace and writes a short summary of what is there and what changed.",
        "ergon",
    );
    weekly.template = "Look through the workspace, read the main documents you find, and \
        write a short summary document of what the work is and its current state."
        .to_string();
    out.push(weekly);

    out
}

#[derive(Clone, Default)]
pub struct Library {
    pub items: Vec<LibraryItem>,
    /// The backed-up file name when the library started fresh (E5).
    pub recovered: Option<String>,
}

impl Library {
    pub fn path(root_data_dir: &Path) -> std::path::PathBuf {
        root_data_dir.join("library.json")
    }

    /// Load the library, seeding it on first run. Corrupt files are
    /// backed up (never silently overwritten) and the library is
    /// re-seeded — the same discipline as the other stores.
    pub fn load(root_data_dir: &Path) -> Library {
        let path = Self::path(root_data_dir);
        let mut recovered: Option<String> = None;
        if let Ok(text) = std::fs::read_to_string(&path) {
            let parsed = serde_json::from_str::<Value>(&text);
            let shape_ok = parsed
                .as_ref()
                .map(|v| {
                    v.is_object()
                        && v.get("items").map(|p| p.is_array()).unwrap_or(false)
                })
                .unwrap_or(false);
            if !shape_ok {
                recovered = backed_up_name(&path);
            } else if let Ok(v) = parsed {
                // Ids from disk are validated, not trusted — the same
                // discipline as person ids: an item id is a slug, and
                // a crafted library.json must not smuggle in an id
                // shaped like a path.
                let items: Vec<LibraryItem> = v["items"]
                    .as_array()
                    .map(|arr| {
                        // First occurrence wins: a duplicated id in
                        // the file must not shadow-list a second
                        // item that `get` can never reach.
                        let mut seen: Vec<String> = Vec::new();
                        arr.iter()
                            .filter_map(LibraryItem::from_json)
                            .filter(|i| crate::valid_person_id(&i.id))
                            .filter(|i| {
                                if seen.contains(&i.id) {
                                    false
                                } else {
                                    seen.push(i.id.clone());
                                    true
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if !items.is_empty() {
                    return Library { items, recovered: None };
                }
                // A valid shape holding no usable items is corruption
                // wearing a valid shape; keep the bytes.
                recovered = backed_up_name(&path);
            }
        }
        let lib = Library {
            items: seed_items(),
            recovered,
        };
        lib.save(root_data_dir);
        lib
    }

    pub fn save(&self, root_data_dir: &Path) {
        let _ = std::fs::create_dir_all(root_data_dir);
        let v = json!({
            "items": self.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
        });
        let path = Self::path(root_data_dir);
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, v.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    pub fn get(&self, id: &str) -> Option<LibraryItem> {
        self.items.iter().find(|i| i.id == id).cloned()
    }

    pub fn search(&self, query: &str) -> Vec<LibraryItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.items.clone();
        }
        self.items
            .iter()
            .filter(|i| {
                i.name.to_lowercase().contains(&q)
                    || i.description.to_lowercase().contains(&q)
                    || i.kind.contains(&q)
                    || i.id.contains(&q)
            })
            .cloned()
            .collect()
    }

    fn clean_text(s: &str, cap: usize) -> String {
        let flat: String = s
            .replace(['\n', '\r'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        flat.chars().take(cap).collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn publish_code_item(
        &mut self,
        kind: &str,
        method: &str,
        name: &str,
        description: &str,
        params: Vec<String>,
        source: String,
        tests: Vec<Value>,
        test_report: Option<Value>,
        author: &str,
    ) -> Result<LibraryItem, String> {
        let name = Self::clean_text(name, 80);
        if name.is_empty() {
            return Err(format!("{method} needs a name"));
        }
        // Cap the id at the loader's validity bound (64 chars), so a
        // long-named skill or tool is never published in a shape the
        // loader would drop on the next boot.
        let id: String = slugify(&name).chars().take(64).collect();
        let id = id.trim_end_matches('-').to_string();
        if let Some(existing) = self.items.iter_mut().find(|i| i.id == id) {
            if existing.builtin {
                return Err(format!(
                    "'{name}' is a built-in item and cannot be overwritten — publish under a different name"
                ));
            }
            if existing.kind != kind {
                return Err(format!(
                    "'{name}' already exists in the library as a {} — pick another name",
                    existing.kind
                ));
            }
            if existing.author != author {
                return Err(format!(
                    "'{name}' is {}'s {} (v{}) — you cannot overwrite another person's library item. Publish your version under a different name",
                    existing.author, kind, existing.version
                ));
            }
            existing.description = Self::clean_text(description, 300);
            existing.params = params;
            existing.source = source;
            existing.tests = tests;
            existing.test_report = test_report;
            existing.author = author.to_string();
            existing.version += 1;
            return Ok(existing.clone());
        }
        let mut it = item(&id, kind, &name, &Self::clean_text(description, 300), author);
        it.params = params;
        it.source = source;
        it.tests = tests;
        it.test_report = test_report;
        self.items.push(it.clone());
        Ok(it)
    }

    /// Publish a skill into the library — how agents maintain it.
    /// A new name creates version 1; re-publishing the same name by
    /// the SAME author updates it in place with a version bump.
    /// Built-in items can never be overwritten, and neither can
    /// another author's item (the seeded items belong to "ergon"):
    /// the library is company-wide, so an in-place overwrite by a
    /// different author would silently redefine a working skill for
    /// every person who carries it. Improvements to someone else's
    /// skill are published under a new name, in the open.
    pub fn publish_skill(
        &mut self,
        name: &str,
        description: &str,
        params: Vec<String>,
        source: String,
        author: &str,
    ) -> Result<LibraryItem, String> {
        self.publish_code_item(
            "skill",
            "publish_skill",
            name,
            description,
            params,
            source,
            Vec::new(),
            None,
            author,
        )
    }

    /// Publish a verified tool into the shared library with its test suite
    /// and latest verification report. Shares the same slugify and
    /// author/builtin collision guarantees as `publish_skill`.
    #[allow(clippy::too_many_arguments)]
    pub fn publish_tool(
        &mut self,
        name: &str,
        description: &str,
        params: Vec<String>,
        source: String,
        tests: Vec<Value>,
        test_report: Value,
        author: &str,
    ) -> Result<LibraryItem, String> {
        let report = (!test_report.is_null()).then_some(test_report);
        self.publish_code_item(
            "tool",
            "publish_tool",
            name,
            description,
            params,
            source,
            tests,
            report,
            author,
        )
    }

    /// Update an existing non-builtin library item's implementation,
    /// version, and optional test report (for self-improvement promotion
    /// or rollback).
    pub fn update_item_for_evolution(
        &mut self,
        id: &str,
        new_source_or_instructions: &str,
        new_version: u32,
        test_report: Option<Value>,
    ) -> Result<LibraryItem, String> {
        let existing = self
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| format!("library item '{id}' not found"))?;
        if existing.builtin {
            return Err(format!(
                "'{id}' is a built-in item and cannot be overwritten"
            ));
        }
        match existing.kind.as_str() {
            "skill" | "tool" => existing.source = new_source_or_instructions.to_string(),
            "subagent" => existing.instructions = new_source_or_instructions.to_string(),
            "app" => existing.template = new_source_or_instructions.to_string(),
            other => return Err(format!("unsupported library item kind '{other}'")),
        }
        existing.version = new_version;
        existing.test_report = test_report.filter(|v| !v.is_null());
        Ok(existing.clone())
    }

    /// Publish an app component into the library (used when a person
    /// accepts an offer to turn repeated work into a one-tap app).
    pub fn publish_app(
        &mut self,
        name: &str,
        description: &str,
        template: String,
        fields: Vec<String>,
        author: &str,
    ) -> Result<LibraryItem, String> {
        let id = slugify(name);
        if let Some(existing) = self.items.iter_mut().find(|i| i.id == id) {
            // A collision is only benign when it is the SAME app by
            // the SAME author (an offer accepted twice): then it is
            // an update. Returning a skill that happens to share the
            // slug handed the suite a "one-tap app" with no Run
            // button, and overwriting someone else's item is the
            // cross-author hole publish_skill already closed.
            if existing.kind != "app" {
                return Err(format!(
                    "an item named \"{name}\" already exists in the library as a {} — the app needs a different name",
                    existing.kind
                ));
            }
            if existing.author != author {
                return Err(format!(
                    "an app named \"{name}\" already exists in the library (published by {}) — the app needs a different name",
                    existing.author
                ));
            }
            existing.template = template;
            existing.fields = fields;
            existing.description = description.to_string();
            existing.version += 1;
            return Ok(existing.clone());
        }
        let mut it = item(&id, "app", name, description, author);
        it.template = template;
        it.fields = fields;
        self.items.push(it.clone());
        Ok(it)
    }
}

// ---------------------------------------------------------------------------
// Suites — the items one person's agent actually carries.
// ---------------------------------------------------------------------------

#[derive(Clone, Default, Debug)]
pub struct Suite {
    pub installed: Vec<String>, // library item ids
}

pub fn suite_path(person_data_dir: &Path) -> std::path::PathBuf {
    person_data_dir.join("suite.json")
}

/// Suites are load → mutate → save files too, and the same person
/// can install from desktop, phone and an agent run at once. This
/// lock serialises those sequences so concurrent installs don't
/// silently drop each other. Callers wrap ONLY the outermost
/// sequence; load_suite/save_suite themselves never lock (the lock
/// is not reentrant).
static SUITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn with_suite_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = SUITE_LOCK.lock().unwrap();
    f()
}

pub fn load_suite(person_data_dir: &Path) -> Suite {
    let path = suite_path(person_data_dir);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(items) = v["installed"].as_array() {
                return Suite {
                    installed: items
                        .iter()
                        .filter_map(|i| i.as_str().map(|s| s.to_string()))
                        .collect(),
                };
            }
        }
    }
    Suite::default()
}

pub fn save_suite(person_data_dir: &Path, suite: &Suite) {
    let _ = std::fs::create_dir_all(person_data_dir);
    let v = json!({ "installed": suite.installed });
    let path = suite_path(person_data_dir);
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, v.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

/// The starter suite for a new person, drawn from the library and
/// shaped by their role. Everyone starts with the Researcher and the
/// meeting-actions app; finance-shaped roles get the Bookkeeper and
/// the expense sheet; writing-shaped roles get the Editor.
pub fn starter_suite(person: &Person, library: &Library) -> Suite {
    let mut ids: Vec<String> = vec!["researcher".into(), "meeting-actions".into()];
    let role = person.role.to_lowercase();
    if ["financ", "bookkeep", "account", "ops"].iter().any(|k| role.contains(k)) {
        ids.push("bookkeeper".into());
        ids.push("expense-sheet".into());
    }
    if ["writ", "edit", "content", "market"].iter().any(|k| role.contains(k)) {
        ids.push("editor".into());
    }
    if role.contains("research") {
        ids.push("reading-list".into());
    }
    ids.retain(|id| library.get(id).is_some());
    Suite { installed: ids }
}

/// Ensure a person's suite file exists (starter suite on first run).
pub fn ensure_suite(person_data_dir: &Path, person: &Person, library: &Library) -> Suite {
    if suite_path(person_data_dir).exists() {
        return load_suite(person_data_dir);
    }
    let suite = starter_suite(person, library);
    save_suite(person_data_dir, &suite);
    suite
}

impl Suite {
    pub fn install(&mut self, id: &str) -> bool {
        if self.installed.iter().any(|i| i == id) {
            return false;
        }
        self.installed.push(id.to_string());
        true
    }

    /// The installed items, resolved against the library. Items whose
    /// id no longer exists in the library are dropped from view (the
    /// suite file itself is left alone).
    pub fn resolved(&self, library: &Library) -> Vec<LibraryItem> {
        self.installed
            .iter()
            .filter_map(|id| library.get(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_item_backward_compatible_and_tool_round_trip() {
        let legacy = json!({
            "id": "legacy-skill",
            "kind": "skill",
            "name": "Legacy",
            "description": "Old schema without tests",
            "author": "alice",
            "version": 1,
            "builtin": false,
            "params": ["x"],
            "source": "pub fn main(x) { x }"
        });
        let parsed = LibraryItem::from_json(&legacy).unwrap();
        assert!(parsed.tests.is_empty());
        assert!(parsed.test_report.is_none());

        let mut lib = Library::default();
        let tool = lib
            .publish_tool(
                "CSV Helper",
                "Parses CSVs",
                vec!["file".into()],
                "pub fn main(file) { file }".into(),
                vec![json!({"name": "t1"})],
                json!({"passed": 1, "total": 1, "all_passed": true}),
                "alice",
            )
            .unwrap();
        assert_eq!(tool.kind, "tool");
        assert_eq!(tool.version, 1);

        let rt = LibraryItem::from_json(&tool.to_json()).unwrap();
        assert_eq!(rt.kind, "tool");
        assert_eq!(rt.tests.len(), 1);
        assert_eq!(rt.test_report.unwrap()["all_passed"], json!(true));

        let cross = lib.publish_tool(
            "CSV Helper",
            "Hijack",
            vec![],
            "pub fn main() { \"x\" }".into(),
            vec![],
            Value::Null,
            "bob",
        );
        assert!(cross.is_err());
    }

    #[test]
    fn update_item_for_evolution_promotes_and_rolls_back() {
        let mut lib = Library {
            items: seed_items(),
            recovered: None,
        };
        assert!(lib
            .update_item_for_evolution("write-document", "x", 2, None)
            .is_err());

        lib.publish_tool(
            "Evo Tool",
            "Evolves",
            vec![],
            "pub fn main() { \"v1\" }".into(),
            vec![],
            json!({"passed": 1, "total": 1}),
            "alice",
        )
        .unwrap();

        let promoted = lib
            .update_item_for_evolution(
                "evo-tool",
                "pub fn main() { \"v2\" }",
                2,
                Some(json!({"passed": 2, "total": 2})),
            )
            .unwrap();
        assert_eq!(promoted.version, 2);
        assert!(promoted.source.contains("v2"));
        assert_eq!(promoted.test_report.unwrap()["total"], json!(2));

        let rolled_back = lib
            .update_item_for_evolution("evo-tool", "pub fn main() { \"v1\" }", 1, None)
            .unwrap();
        assert_eq!(rolled_back.version, 1);
        assert!(rolled_back.source.contains("v1"));
        assert!(rolled_back.test_report.is_none());
    }
}
