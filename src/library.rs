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

use crate::{backup_corrupt, slugify, Person};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct LibraryItem {
    pub id: String,
    pub kind: String, // "skill" | "subagent" | "app"
    pub name: String,
    pub description: String,
    pub author: String, // person id, or "ergon" for seeds
    pub version: u32,
    pub builtin: bool,
    pub params: Vec<String>, // skill: ordered parameter names
    pub source: String,      // skill: Rune source
    pub instructions: String, // subagent: system instructions
    pub template: String,    // app: task template with {field} slots
    pub fields: Vec<String>, // app: input field names
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
        })
    }

    pub fn from_json(v: &Value) -> Option<LibraryItem> {
        let kind = v["kind"].as_str()?.to_string();
        if !matches!(kind.as_str(), "skill" | "subagent" | "app") {
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
    }
}

/// The seed library: what a fresh install already carries, so a real
/// install is the adapted state, not a clean slate. The built-in
/// skills come first, then working library-native items.
pub fn seed_items() -> Vec<LibraryItem> {
    let mut out: Vec<LibraryItem> = Vec::new();

    for (id, name, desc, params) in [
        (
            "write-document",
            "Write document",
            "Built into every agent: formats sections into a finished markdown document in the workspace.",
            vec!["title", "sections"],
        ),
        (
            "make-spreadsheet",
            "Make spreadsheet",
            "Built into every agent: turns columns and rows into a finished CSV spreadsheet in the workspace.",
            vec!["title", "columns", "rows"],
        ),
        (
            "research",
            "Research",
            "Built into every agent: fetches several pages and compiles them into one research notes document.",
            vec!["topic", "urls"],
        ),
        (
            "update-memory",
            "Update memory",
            "Built into every agent: records a learned preference or fact into the person's long-term memory.",
            vec!["note"],
        ),
    ] {
        let mut it = item(id, "skill", name, desc, "ergon");
        it.builtin = true;
        it.params = params.into_iter().map(|s| s.to_string()).collect();
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
                let _ = backup_corrupt(&path);
            } else if let Ok(v) = parsed {
                let items: Vec<LibraryItem> = v["items"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(LibraryItem::from_json).collect())
                    .unwrap_or_default();
                if !items.is_empty() {
                    return Library { items };
                }
                // A valid shape holding no usable items is corruption
                // wearing a valid shape; keep the bytes.
                let _ = backup_corrupt(&path);
            }
        }
        let lib = Library {
            items: seed_items(),
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

    /// Publish a skill into the library — how agents maintain it.
    /// A new name creates version 1; re-publishing the same name by
    /// the same author updates it in place with a version bump.
    /// Built-in items can never be overwritten.
    pub fn publish_skill(
        &mut self,
        name: &str,
        description: &str,
        params: Vec<String>,
        source: String,
        author: &str,
    ) -> Result<LibraryItem, String> {
        let clean = |s: &str, cap: usize| -> String {
            let flat: String = s
                .replace(['\n', '\r'], " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            flat.chars().take(cap).collect()
        };
        let name = clean(name, 80);
        if name.is_empty() {
            return Err("publish_skill needs a name".to_string());
        }
        let id = slugify(&name);
        if let Some(existing) = self.items.iter_mut().find(|i| i.id == id) {
            if existing.builtin {
                return Err(format!(
                    "'{name}' is a built-in item and cannot be overwritten — publish under a different name"
                ));
            }
            if existing.kind != "skill" {
                return Err(format!(
                    "'{name}' already exists in the library as a {} — pick another name",
                    existing.kind
                ));
            }
            existing.description = clean(description, 300);
            existing.params = params;
            existing.source = source;
            existing.author = author.to_string();
            existing.version += 1;
            return Ok(existing.clone());
        }
        let mut it = item(&id, "skill", &name, &clean(description, 300), author);
        it.params = params;
        it.source = source;
        self.items.push(it.clone());
        Ok(it)
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
        if self.items.iter().any(|i| i.id == id) {
            // Already there (e.g. the offer fired twice): return the
            // existing item rather than erroring the acceptance.
            return Ok(self.get(&id).unwrap());
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
