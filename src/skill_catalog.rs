//! Skill catalog metadata and sandboxed Rune VM validation / dry-run harness.
//!
//! Exposes rich, serializable metadata for all 12 built-in Rune skills
//! (`GET /api/skills`), compile-time validation across the full library
//! (`validate_all_skills`), and isolated ephemeral sandbox dry-runs
//! (`dry_run_skill` / `handle_skill_test_request` for `POST /api/skills/test`).

use crate::{sandbox_path, skills, truncate, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static DRY_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct SkillInputField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
    pub example: Value,
}

impl SkillInputField {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "type": self.field_type,
            "required": self.required,
            "description": self.description,
            "example": self.example,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillMetadata {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub deterministic: bool,
    pub outputs: Vec<String>,
    pub input_schema: Vec<SkillInputField>,
    pub sample_prompts: Vec<String>,
    pub source: String,
    pub source_lines: usize,
}

impl SkillMetadata {
    pub fn param_names(&self) -> Vec<String> {
        self.input_schema.iter().map(|f| f.name.clone()).collect()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "slug": self.slug,
            "title": self.title,
            "name": self.title,
            "category": self.category,
            "description": self.description,
            "deterministic": self.deterministic,
            "builtin": true,
            "outputs": self.outputs,
            "params": self.param_names(),
            "input_schema": self.input_schema.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "sample_prompts": self.sample_prompts,
            "source": self.source,
            "source_lines": self.source_lines,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillValidationEntry {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub compiled: bool,
    pub source_lines: usize,
    pub error: Option<String>,
}

impl SkillValidationEntry {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "slug": self.slug,
            "title": self.title,
            "compiled": self.compiled,
            "source_lines": self.source_lines,
            "error": self.error,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillValidationReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub all_ok: bool,
    pub duration_ms: u64,
    pub results: Vec<SkillValidationEntry>,
}

impl SkillValidationReport {
    pub fn to_json(&self) -> Value {
        json!({
            "total": self.total,
            "passed": self.passed,
            "failed": self.failed,
            "all_ok": self.all_ok,
            "duration_ms": self.duration_ms,
            "results": self.results.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DryRunFile {
    pub path: String,
    pub bytes: u64,
    pub preview: String,
}

impl DryRunFile {
    pub fn to_json(&self) -> Value {
        json!({
            "path": self.path,
            "bytes": self.bytes,
            "preview": self.preview,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillDryRunResult {
    pub skill_id: String,
    pub ok: bool,
    pub compiled: bool,
    pub output: String,
    pub error: Option<String>,
    pub files_written: Vec<DryRunFile>,
    pub memory_after: Option<String>,
    pub duration_ms: u64,
}

impl SkillDryRunResult {
    pub fn to_json(&self) -> Value {
        json!({
            "skill_id": self.skill_id,
            "ok": self.ok,
            "compiled": self.compiled,
            "output": self.output,
            "error": self.error,
            "files_written": self.files_written.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "memory_after": self.memory_after,
            "duration_ms": self.duration_ms,
        })
    }
}

fn parse_catalog_item(v: &Value) -> SkillMetadata {
    let id = v["id"].as_str().unwrap_or_default().to_string();
    let source = skills::builtin_script(&id);
    let str_vec = |arr: &Value| -> Vec<String> {
        arr.as_array()
            .map(|xs| {
                xs.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let input_schema = v["input_schema"]
        .as_array()
        .map(|xs| {
            xs.iter()
                .map(|f| SkillInputField {
                    name: f["name"].as_str().unwrap_or_default().to_string(),
                    field_type: f["type"].as_str().unwrap_or_default().to_string(),
                    required: f["required"].as_bool().unwrap_or(false),
                    description: f["description"].as_str().unwrap_or_default().to_string(),
                    example: f["example"].clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    SkillMetadata {
        id,
        slug: v["slug"].as_str().unwrap_or_default().to_string(),
        title: v["title"].as_str().unwrap_or_default().to_string(),
        category: v["category"].as_str().unwrap_or_default().to_string(),
        description: v["description"].as_str().unwrap_or_default().to_string(),
        deterministic: v["deterministic"].as_bool().unwrap_or(false),
        outputs: str_vec(&v["outputs"]),
        input_schema,
        sample_prompts: str_vec(&v["sample_prompts"]),
        source: source.to_string(),
        source_lines: source.lines().count(),
    }
}

fn catalog_root() -> &'static Value {
    static ROOT: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        static DEFLATED: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/skills_catalog.deflate"));
        let raw = miniz_oxide::inflate::decompress_to_vec(DEFLATED)
            .expect("skills catalog deflate valid");
        serde_json::from_slice(&raw).expect("skills catalog json valid")
    })
}

/// Complete catalog of all 12 built-in Rune skills shipped with Ergon.
pub fn all_skills() -> Vec<SkillMetadata> {
    static CATALOG: std::sync::OnceLock<Vec<SkillMetadata>> = std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let root = catalog_root();
            let items = root
                .get("skills")
                .and_then(|v| v.as_array())
                .or_else(|| root.as_array())
                .expect("skills catalog array valid");
            items.iter().map(parse_catalog_item).collect()
        })
        .clone()
}

/// Look up one built-in skill by either snake_case `id` or kebab-case `slug`.
pub fn get_skill(id_or_slug: &str) -> Option<SkillMetadata> {
    let q = id_or_slug.trim().to_ascii_lowercase();
    let q_snake = q.replace('-', "_");
    all_skills()
        .into_iter()
        .find(|s| s.id == q_snake || s.slug == q)
}

#[cold]
#[inline(never)]
pub fn starter_playbooks_json() -> Value {
    let items = catalog_root()
        .get("starter_playbooks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let out: Vec<Value> = items
        .into_iter()
        .map(|mut item| {
            let name = item["name"].as_str().unwrap_or("");
            let prompt = item["prompt"].as_str().unwrap_or("");
            let schedule = item["schedule"].as_str().unwrap_or("manual");
            let skill_id = item["skill_id"].as_str();
            let steps: Vec<String> = item["steps"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let expected_inputs: Vec<String> = item["expected_inputs"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let opts = crate::routines::PlaybookOptions {
                steps,
                expected_inputs,
                pack_deliverables: item["pack_deliverables"].as_bool().unwrap_or(true),
            };
            let digest = crate::routines::playbook_canonical_sha256(
                name,
                prompt,
                schedule,
                None,
                skill_id,
                &opts,
            );
            item["sha256"] = json!(digest);
            item["verified"] = json!(true);
            item
        })
        .collect();
    Value::Array(out)
}

fn list_workspace_input_candidates(workspace_dir: &Path) -> Vec<String> {
    const GENERATED_OUTPUTS: &[&str] = &[
        "Weekly Client Digest.md",
        "Weekly Client Digest.docx",
        "Release Readiness Audit.md",
        "Release Readiness Audit.csv",
        "Research Brief Pack.md",
        "Playbook Deliverable Pack.md",
    ];
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(workspace_dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || GENERATED_OUTPUTS.iter().any(|g| g.eq_ignore_ascii_case(name)) {
            continue;
        }
        out.push(name.to_string());
    }
    out.sort();
    out
}

fn ensure_starter_inputs(
    cfg: &Config,
    starter: &Value,
) -> Result<(std::collections::HashMap<String, String>, Vec<String>), String> {
    let mut inputs = std::collections::HashMap::new();
    let mut seeded = Vec::new();
    let candidates = list_workspace_input_candidates(&cfg.workspace_dir);
    let input_kind = starter["input_kind"].as_str().unwrap_or("");
    let seed_name = starter["seed_fixture"]["filename"].as_str().unwrap_or("");
    let seed_body = starter["seed_fixture"]["content"].as_str().unwrap_or("");

    let mut seed_if_needed = |default_name: &str| -> Result<String, String> {
        let fname = if seed_name.is_empty() { default_name } else { seed_name };
        let full = crate::sandbox_path(&cfg.workspace_dir, fname)?;
        if !full.exists() && !seed_body.is_empty() {
            crate::atomic_write_bytes(&full, seed_body.as_bytes())
                .map_err(|e| format!("seed input {fname}: {e}"))?;
            seeded.push(fname.to_string());
        }
        Ok(fname.to_string())
    };

    match input_kind {
        "notes_md" => {
            let md_txt: Vec<&String> = candidates
                .iter()
                .filter(|f| {
                    let low = f.to_ascii_lowercase();
                    low.ends_with(".md") || low.ends_with(".txt")
                })
                .collect();
            let preferred = md_txt
                .iter()
                .find(|f| {
                    let low = f.to_ascii_lowercase();
                    low.contains("meeting")
                        || low.contains("standup")
                        || low.contains("notes")
                        || low.contains("transcript")
                })
                .or_else(|| md_txt.first())
                .map(|s| (*s).clone());
            let chosen = match preferred {
                Some(existing) => existing,
                None => seed_if_needed("meeting-notes.md")?,
            };
            inputs.insert("notes_file".to_string(), chosen);
        }
        "ledger_csv" => {
            let chosen = candidates
                .iter()
                .find(|f| f.to_ascii_lowercase().ends_with(".csv"))
                .cloned();
            let file = match chosen {
                Some(existing) => existing,
                None => seed_if_needed("release-ledger.csv")?,
            };
            inputs.insert("ledger_file".to_string(), file);
        }
        "source_docs" => {
            let docs: Vec<String> = candidates
                .iter()
                .filter(|f| {
                    let low = f.to_ascii_lowercase();
                    low.ends_with(".md") || low.ends_with(".txt")
                })
                .take(5)
                .cloned()
                .collect();
            let chosen = if docs.is_empty() {
                vec![seed_if_needed("research-notes.md")?]
            } else {
                docs
            };
            inputs.insert("source_files".to_string(), chosen.join(","));
        }
        _ => {}
    }
    Ok((inputs, seeded))
}

#[cold]
#[inline(never)]
pub fn launch_starter_playbook(
    cfg: &Config,
    raw_id: &str,
    run_now: bool,
) -> Result<Value, (u16, String)> {
    let q = raw_id.trim().to_ascii_lowercase();
    let starters = starter_playbooks_json();
    let Some(starter) = starters.as_array().and_then(|arr| {
        arr.iter().find(|s| {
            s["id"].as_str().is_some_and(|v| v.eq_ignore_ascii_case(&q))
                || s["name"].as_str().is_some_and(|v| v.eq_ignore_ascii_case(&q))
        })
    }) else {
        return Err((404, "starter playbook not found".to_string()));
    };

    let name = starter["name"].as_str().unwrap_or("Starter Playbook");
    let prompt = starter["prompt"].as_str().unwrap_or("");
    let schedule = starter["schedule"].as_str().unwrap_or("manual");
    let skill_id = starter["skill_id"].as_str();
    let steps: Vec<String> = starter["steps"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let steps_v2 = crate::pipelines::parse_steps_v2(&starter["steps_v2"])
        .map_err(|e| (500, format!("invalid starter steps_v2: {e}")))?;
    let expected_inputs: Vec<String> = starter["expected_inputs"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let pack_deliverables = starter["pack_deliverables"].as_bool().unwrap_or(true);
    let pb_opts = crate::routines::PlaybookOptions {
        steps,
        expected_inputs,
        pack_deliverables,
    };

    let existing = crate::routines::list_routines(&cfg.data_dir)
        .into_iter()
        .find(|r| r["name"].as_str() == Some(name));
    let routine = if let Some(rt) = existing {
        let rid = rt["id"].as_str().unwrap_or("");
        if rt.get("steps_v2").is_none() && !rid.is_empty() {
            crate::routines::set_routine_steps_v2(&cfg.data_dir, rid, &steps_v2)
                .map_err(|e| (500, e))?
                .unwrap_or(rt)
        } else {
            rt
        }
    } else {
        crate::routines::create_pipeline_routine(
            &cfg.data_dir,
            name,
            prompt,
            schedule,
            None,
            skill_id,
            true,
            &pb_opts,
            &steps_v2,
        )
        .map_err(|e| (400, e))?
    };
    let routine_id = routine["id"].as_str().unwrap_or("").to_string();

    let mut created_files: Vec<String> = Vec::new();
    let mut seeded_inputs: Vec<String> = Vec::new();
    let mut receipt_summary = String::new();
    let mut pipeline_run_json = Value::Null;
    if run_now {
        let (inputs, seeded) = ensure_starter_inputs(cfg, starter).map_err(|e| (500, e))?;
        seeded_inputs = seeded;
        let run = crate::pipelines::execute_deterministic_steps_sync(
            cfg,
            Some(&routine_id),
            name,
            &steps_v2,
            inputs,
        )
        .map_err(|e| (500, e))?;
        let mut step_summaries = Vec::new();
        for st in &run.step_states {
            for f in &st.files {
                if !created_files.contains(f) {
                    created_files.push(f.clone());
                }
            }
            if !st.output.trim().is_empty() {
                step_summaries.push(st.output.clone());
            }
        }
        receipt_summary = step_summaries.join(" | ");
        pipeline_run_json = run.to_json();
    }

    let updated_routine = crate::routines::get_routine(&cfg.data_dir, &routine_id).unwrap_or(routine);
    let now_ms = crate::records::now_millis();
    let hist = crate::scheduler::list_run_history(&cfg.data_dir, Some(&routine_id), 20);
    let enriched = crate::scheduler::enrich_routine_json(&updated_routine, now_ms, &hist);
    let preview = crate::routines::preview_playbook_routine(
        &cfg.data_dir,
        &cfg.workspace_dir,
        &created_files,
        &routine_id,
    );

    Ok(json!({
        "ok": true,
        "starter_id": starter["id"],
        "sha256": starter["sha256"],
        "routine": enriched,
        "run": pipeline_run_json,
        "created_files": created_files,
        "seeded_inputs": seeded_inputs,
        "summary": receipt_summary,
        "preview": preview,
    }))
}

/// Ready-to-serve JSON payload for `GET /api/skills`.
pub fn catalog_json() -> Value {
    let list = all_skills();
    let mut categories: Vec<String> = Vec::new();
    for s in &list {
        if !categories.contains(&s.category) {
            categories.push(s.category.clone());
        }
    }
    json!({
        "count": list.len(),
        "categories": categories,
        "skills": list.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
        "starter_playbooks": starter_playbooks_json(),
    })
}

/// Compile-check all 12 built-in Rune skills against the real host Rune VM
/// context and verify metadata integrity (unique ids/slugs, non-empty prompts).
pub fn validate_all_skills(cfg: &Config) -> SkillValidationReport {
    let started = Instant::now();
    let list = all_skills();
    let mut results = Vec::with_capacity(list.len());
    let mut seen_ids: Vec<String> = Vec::new();
    let mut seen_slugs: Vec<String> = Vec::new();
    let mut passed = 0usize;

    for s in &list {
        let mut err: Option<String> = None;
        if seen_ids.contains(&s.id) {
            err = Some(format!("duplicate skill id: {}", s.id));
        } else if seen_slugs.contains(&s.slug) {
            err = Some(format!("duplicate skill slug: {}", s.slug));
        } else if s.sample_prompts.is_empty() || s.input_schema.is_empty() {
            err = Some("missing sample_prompts or input_schema".to_string());
        } else if let Err(compile_err) = skills::check_script(cfg, &s.source) {
            err = Some(compile_err);
        }

        seen_ids.push(s.id.clone());
        seen_slugs.push(s.slug.clone());

        let compiled = err.is_none();
        if compiled {
            passed += 1;
        }
        results.push(SkillValidationEntry {
            id: s.id.clone(),
            slug: s.slug.clone(),
            title: s.title.clone(),
            compiled,
            source_lines: s.source_lines,
            error: err,
        });
    }

    let total = results.len();
    let failed = total - passed;
    SkillValidationReport {
        total,
        passed,
        failed,
        all_ok: failed == 0,
        duration_ms: started.elapsed().as_millis() as u64,
        results,
    }
}

struct EphemeralSandbox {
    root: PathBuf,
    cfg: Config,
}

impl EphemeralSandbox {
    fn new(tag: &str) -> Result<Self, String> {
        let seq = DRY_RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let clean_tag: String = tag
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(24)
            .collect();
        let root = std::env::temp_dir().join(format!("ergon-skill-dryrun-{pid}-{seq}-{clean_tag}"));
        let data_dir = root.join("data");
        let workspace_dir = root.join("workspace");
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "dry-run".to_string(),
            api_key: None,
        };
        cfg.ensure_dirs()
            .map_err(|e| format!("create dry-run sandbox: {e}"))?;
        Ok(EphemeralSandbox { root, cfg })
    }

    fn seed_file(&self, rel: &str, content: &str) -> Result<(), String> {
        let full = sandbox_path(&self.cfg.workspace_dir, rel)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("seed mkdir {rel}: {e}"))?;
        }
        std::fs::write(&full, content).map_err(|e| format!("seed write {rel}: {e}"))
    }

    fn seed_from_value(&self, seed_files: &Value) -> Result<Vec<String>, String> {
        let mut seeded = Vec::new();
        match seed_files {
            Value::Object(map) => {
                for (path, val) in map {
                    let content = val
                        .as_str()
                        .ok_or_else(|| format!("seed file '{path}' content must be a string"))?;
                    self.seed_file(path, content)?;
                    seeded.push(path.clone());
                }
            }
            Value::Array(items) => {
                for item in items {
                    let path = item["path"]
                        .as_str()
                        .ok_or("seed file array items require a string 'path'")?;
                    let content = item["content"]
                        .as_str()
                        .ok_or("seed file array items require a string 'content'")?;
                    self.seed_file(path, content)?;
                    seeded.push(path.to_string());
                }
            }
            Value::Null => {}
            _ => return Err("seed files must be a JSON object or array of {path, content}".to_string()),
        }
        Ok(seeded)
    }

    fn collect_written_files(&self, pre_existing: &[String]) -> Vec<DryRunFile> {
        fn walk(root: &Path, dir: &Path, pre: &[String], out: &mut Vec<DryRunFile>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    walk(root, &p, pre, out);
                    continue;
                }
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                if pre.iter().any(|s| s == &rel) {
                    continue;
                }
                let bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                let preview = std::fs::read_to_string(&p)
                    .map(|s| truncate(&s, 2_000))
                    .unwrap_or_default();
                out.push(DryRunFile {
                    path: rel,
                    bytes,
                    preview,
                });
            }
        }
        let mut out = Vec::new();
        walk(&self.cfg.workspace_dir, &self.cfg.workspace_dir, pre_existing, &mut out);
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }
}

impl Drop for EphemeralSandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Spawn a tiny bounded one-shot local HTTP server used only when running
/// the built-in dry-run smoke test for `research` with default args so it
/// executes offline with zero external network dependencies.
fn spawn_local_sample_http(body: &'static str) -> Option<(String, std::thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    let _ = listener.set_nonblocking(true);
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
    Some((format!("http://127.0.0.1:{port}/sample"), handle))
}

/// Built-in realistic dry-run fixture (`args`, `seed_files`, `seed_memory`)
/// used when `dry_run_skill` is invoked with empty/null args so `POST /api/skills/test`
/// can smoke-test any of the 12 skills with one click.
#[cold]
#[inline(never)]
fn default_fixture_for(
    canonical_id: &str,
    local_research_url: Option<&str>,
) -> (Value, Value, Option<&'static str>) {
    let fx = &catalog_root()["fixtures"][canonical_id];
    if fx.is_null() {
        return (json!({}), json!({}), None);
    }
    let mut args = fx["args"].clone();
    if canonical_id == "research" {
        let url = local_research_url.unwrap_or("http://127.0.0.1:9/offline");
        args["urls"] = json!([url]);
    }
    let seeds = fx["seeds"].clone();
    let mem = fx["memory"].as_str();
    (args, seeds, mem)
}

/// Run a built-in Rune skill in a fresh ephemeral sandbox workspace.
/// When `args` is null or an empty object and `seed_files` is `None`,
/// uses realistic built-in sample inputs for that skill so the caller
/// can verify compile + execution end-to-end with zero manual setup.
pub fn dry_run_skill(
    skill_id: &str,
    args: &Value,
    seed_files: Option<&Value>,
    seed_memory: Option<&str>,
) -> SkillDryRunResult {
    let started = Instant::now();
    let Some(meta) = get_skill(skill_id) else {
        return SkillDryRunResult {
            skill_id: skill_id.to_string(),
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(format!("unknown built-in skill: {skill_id}")),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    };

    let sandbox = match EphemeralSandbox::new(&meta.id) {
        Ok(s) => s,
        Err(e) => {
            return SkillDryRunResult {
                skill_id: meta.id,
                ok: false,
                compiled: false,
                output: String::new(),
                error: Some(e),
                files_written: Vec::new(),
                memory_after: None,
                duration_ms: started.elapsed().as_millis() as u64,
            };
        }
    };

    if let Err(e) = skills::check_script(&sandbox.cfg, &meta.source) {
        return SkillDryRunResult {
            skill_id: meta.id,
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(e),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    }

    let args_is_empty = args.is_null() || args.as_object().map(|m| m.is_empty()).unwrap_or(false);
    let mut local_http_handle: Option<std::thread::JoinHandle<()>> = None;

    let (effective_args, effective_seeds, effective_mem) = if args_is_empty && seed_files.is_none()
    {
        let local_url = if meta.id == "research" {
            if let Some((url, handle)) = spawn_local_sample_http(
                "<html><body><h1>Sample Source</h1><p>Verified offline research snippet.</p></body></html>",
            ) {
                local_http_handle = Some(handle);
                Some(url)
            } else {
                None
            }
        } else {
            None
        };
        let (def_args, def_files, def_mem) = default_fixture_for(&meta.id, local_url.as_deref());
        (def_args, Some(def_files), seed_memory.or(def_mem))
    } else {
        (args.clone(), seed_files.cloned(), seed_memory)
    };

    if let Some(mem) = effective_mem {
        let _ = std::fs::write(&sandbox.cfg.memory_path, mem);
    }

    let pre_seeded = match effective_seeds.as_ref() {
        Some(v) => match sandbox.seed_from_value(v) {
            Ok(paths) => paths,
            Err(e) => {
                if let Some(h) = local_http_handle {
                    let _ = h.join();
                }
                return SkillDryRunResult {
                    skill_id: meta.id,
                    ok: false,
                    compiled: true,
                    output: String::new(),
                    error: Some(e),
                    files_written: Vec::new(),
                    memory_after: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        },
        None => Vec::new(),
    };

    let exec = skills::dispatch_builtin_skill(&sandbox.cfg, &meta.id, &effective_args)
        .unwrap_or_else(|| Err(format!("skill {} is not dispatchable", meta.id)));

    if let Some(h) = local_http_handle {
        let _ = h.join();
    }

    let files_written = sandbox.collect_written_files(&pre_seeded);
    let memory_after = if meta.id == "update_memory" {
        Some(sandbox.cfg.read_memory())
    } else {
        None
    };

    match exec {
        Ok(output) => SkillDryRunResult {
            skill_id: meta.id,
            ok: true,
            compiled: true,
            output,
            error: None,
            files_written,
            memory_after,
            duration_ms: started.elapsed().as_millis() as u64,
        },
        Err(err) => SkillDryRunResult {
            skill_id: meta.id,
            ok: false,
            compiled: true,
            output: String::new(),
            error: Some(err),
            files_written,
            memory_after,
            duration_ms: started.elapsed().as_millis() as u64,
        },
    }
}

/// Compile and execute an arbitrary Rune library script in a fresh ephemeral
/// sandbox workspace (strict 5M instruction budget & 128 MB allocation cap).
pub fn dry_run_custom_source(
    source: &str,
    params: &[String],
    seed_files: Option<&Value>,
    seed_memory: Option<&str>,
) -> SkillDryRunResult {
    let started = Instant::now();
    let sandbox = match EphemeralSandbox::new("custom") {
        Ok(s) => s,
        Err(e) => {
            return SkillDryRunResult {
                skill_id: "custom".to_string(),
                ok: false,
                compiled: false,
                output: String::new(),
                error: Some(e),
                files_written: Vec::new(),
                memory_after: None,
                duration_ms: started.elapsed().as_millis() as u64,
            };
        }
    };

    if let Err(e) = skills::check_script(&sandbox.cfg, source) {
        return SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: false,
            compiled: false,
            output: String::new(),
            error: Some(e),
            files_written: Vec::new(),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        };
    }

    if let Some(mem) = seed_memory {
        let _ = std::fs::write(&sandbox.cfg.memory_path, mem);
    }
    let pre_seeded = match seed_files {
        Some(v) => match sandbox.seed_from_value(v) {
            Ok(paths) => paths,
            Err(e) => {
                return SkillDryRunResult {
                    skill_id: "custom".to_string(),
                    ok: false,
                    compiled: true,
                    output: String::new(),
                    error: Some(e),
                    files_written: Vec::new(),
                    memory_after: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        },
        None => Vec::new(),
    };

    match skills::run_library_skill(&sandbox.cfg, source, params) {
        Ok(output) => SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: true,
            compiled: true,
            output,
            error: None,
            files_written: sandbox.collect_written_files(&pre_seeded),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        },
        Err(err) => SkillDryRunResult {
            skill_id: "custom".to_string(),
            ok: false,
            compiled: true,
            output: String::new(),
            error: Some(err),
            files_written: sandbox.collect_written_files(&pre_seeded),
            memory_after: None,
            duration_ms: started.elapsed().as_millis() as u64,
        },
    }
}

/// High-level JSON handler for `POST /api/skills/test`.
///
/// Supports either:
/// - `{ "skill_id": "reconcile_ledger", "args": {...}, "files": {...}, "memory": "..." }`
/// - `{ "source": "pub fn main(x) { ... }", "params": ["..."], "files": {...} }`
/// - `{ "validate_all": true }` (runs compile check on all 12 built-in skills)
pub fn handle_skill_test_request(body: &Value) -> Value {
    if body.get("validate_all").and_then(|v| v.as_bool()).unwrap_or(false) {
        let sandbox = match EphemeralSandbox::new("validate-all") {
            Ok(s) => s,
            Err(e) => return json!({ "ok": false, "error": e }),
        };
        return validate_all_skills(&sandbox.cfg).to_json();
    }

    let seed_files = body.get("files").or_else(|| body.get("seed_files"));
    let seed_memory = body.get("memory").and_then(|v| v.as_str());

    if let Some(source) = body.get("source").and_then(|v| v.as_str()) {
        if !source.trim().is_empty() {
            let params: Vec<String> = body
                .get("params")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|i| i.as_str().map(|s| s.to_string()).unwrap_or_else(|| i.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            return dry_run_custom_source(source, &params, seed_files, seed_memory).to_json();
        }
    }

    let skill_id = body
        .get("skill_id")
        .or_else(|| body.get("id"))
        .or_else(|| body.get("slug"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if skill_id.trim().is_empty() {
        return json!({
            "ok": false,
            "compiled": false,
            "error": "request requires 'skill_id', 'source', or 'validate_all': true"
        });
    }

    let args = body.get("args").unwrap_or(&Value::Null);
    dry_run_skill(skill_id, args, seed_files, seed_memory).to_json()
}
