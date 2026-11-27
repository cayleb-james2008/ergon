//! Real Tool Builder: sandboxed verification, executable test runner,
//! phone approval packaging, and shared company library publishing.

use crate::library::{load_suite, save_suite, with_suite_lock, Library};
use crate::records::{add_decision_with, now_millis};
use crate::skills::{check_script, run_library_skill};
use crate::tools::{atomic_write_bytes_with_mode, sandbox_path};
use crate::tripwire::sha256_hex;
use crate::{slugify, Config};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

static SANDBOX_SEQ: AtomicU64 = AtomicU64::new(0);
static PKG_SEQ: AtomicU64 = AtomicU64::new(0);
static TOOL_PKG_LOCK: Mutex<()> = Mutex::new(());

fn with_tool_pkg_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = TOOL_PKG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

fn parse_string_map(v: &Value) -> BTreeMap<String, String> {
    v.as_object()
        .map(|map| {
            map.iter()
                .filter_map(|(k, val)| val.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_string_vec(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn clean_text(s: &str, cap: usize) -> String {
    let flat: String = s
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    flat.chars().take(cap).collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTestCase {
    pub name: String,
    pub setup_files: BTreeMap<String, String>,
    pub args: Vec<String>,
    pub expect_ok: bool,
    pub expect_output_contains: Option<String>,
    pub expect_files: BTreeMap<String, String>,
}

impl ToolTestCase {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "setup_files": self.setup_files,
            "args": self.args,
            "expect_ok": self.expect_ok,
            "expect_output_contains": self.expect_output_contains,
            "expect_files": self.expect_files,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let name = v["name"].as_str()?.to_string();
        Some(Self {
            name,
            setup_files: parse_string_map(&v["setup_files"]),
            args: parse_string_vec(&v["args"]),
            expect_ok: v["expect_ok"].as_bool().unwrap_or(true),
            expect_output_contains: v["expect_output_contains"].as_str().map(|s| s.to_string()),
            expect_files: parse_string_map(&v["expect_files"]),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTestCaseResult {
    pub name: String,
    pub passed: bool,
    pub ok: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolTestCaseResult {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "passed": self.passed,
            "ok": self.ok,
            "output": self.output,
            "error": self.error,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        Some(Self {
            name: v["name"].as_str()?.to_string(),
            passed: v["passed"].as_bool()?,
            ok: v["ok"].as_bool().unwrap_or(false),
            output: v["output"].as_str().unwrap_or("").to_string(),
            error: v["error"].as_str().map(|s| s.to_string()),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTestReport {
    pub passed: usize,
    pub total: usize,
    pub all_passed: bool,
    pub duration_ms: u64,
    pub source_sha256: String,
    pub ran_at_ms: u64,
    pub cases: Vec<ToolTestCaseResult>,
}

impl ToolTestReport {
    pub fn to_json(&self) -> Value {
        json!({
            "passed": self.passed,
            "total": self.total,
            "all_passed": self.all_passed,
            "duration_ms": self.duration_ms,
            "source_sha256": self.source_sha256,
            "ran_at_ms": self.ran_at_ms,
            "cases": self.cases.iter().map(ToolTestCaseResult::to_json).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let cases = v["cases"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(ToolTestCaseResult::from_json)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Some(Self {
            passed: v["passed"].as_u64()? as usize,
            total: v["total"].as_u64()? as usize,
            all_passed: v["all_passed"].as_bool()?,
            duration_ms: v["duration_ms"].as_u64().unwrap_or(0),
            source_sha256: v["source_sha256"].as_str()?.to_string(),
            ran_at_ms: v["ran_at_ms"].as_u64().unwrap_or(0),
            cases,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ToolPackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub params: Vec<String>,
    pub source: String,
    pub tests: Vec<ToolTestCase>,
    pub test_report: ToolTestReport,
    pub status: String,
    pub decision_id: String,
    pub created_ms: u64,
    pub updated_ms: u64,
}

impl ToolPackage {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "author": self.author,
            "params": self.params,
            "source": self.source,
            "tests": self.tests.iter().map(ToolTestCase::to_json).collect::<Vec<_>>(),
            "test_report": self.test_report.to_json(),
            "status": self.status,
            "decision_id": self.decision_id,
            "created_ms": self.created_ms,
            "updated_ms": self.updated_ms,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let tests = v["tests"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(ToolTestCase::from_json)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let test_report = ToolTestReport::from_json(&v["test_report"])?;
        Some(Self {
            id: v["id"].as_str()?.to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
            author: v["author"].as_str().unwrap_or("default").to_string(),
            params: parse_string_vec(&v["params"]),
            source: v["source"].as_str().unwrap_or("").to_string(),
            tests,
            test_report,
            status: v["status"]
                .as_str()
                .unwrap_or("pending_approval")
                .to_string(),
            decision_id: v["decision_id"].as_str().unwrap_or("").to_string(),
            created_ms: v["created_ms"].as_u64().unwrap_or(0),
            updated_ms: v["updated_ms"].as_u64().unwrap_or(0),
        })
    }
}

struct SandboxDir {
    root: PathBuf,
    cfg: Config,
}

impl SandboxDir {
    fn new(root_data_dir: &Path) -> Result<Self, String> {
        let seq = SANDBOX_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = root_data_dir
            .join(".tool_sandbox")
            .join(format!("run-{seq}"));
        let _ = std::fs::remove_dir_all(&root);
        let data_dir = root.join("data_dir");
        let workspace_dir = root.join("workspace");
        std::fs::create_dir_all(&data_dir).map_err(|e| format!("create sandbox data_dir: {e}"))?;
        std::fs::create_dir_all(&workspace_dir)
            .map_err(|e| format!("create sandbox workspace: {e}"))?;
        let memory_path = data_dir.join("memory.md");
        let cfg = Config {
            data_dir,
            workspace_dir,
            memory_path,
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "tool-sandbox".to_string(),
            api_key: None,
        };
        Ok(Self { root, cfg })
    }
}

impl Drop for SandboxDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Compile-check `source` and execute every `ToolTestCase` inside an
/// isolated RAII-cleaned sandbox under `<root_data_dir>/.tool_sandbox/`.
pub fn verify_and_test_tool(
    root_data_dir: &Path,
    params: &[String],
    source: &str,
    tests: &[ToolTestCase],
) -> Result<ToolTestReport, String> {
    if params.len() > 5 {
        return Err("tools take at most 5 parameters".to_string());
    }
    if source.trim().is_empty() {
        return Err("tool source cannot be empty".to_string());
    }
    if tests.is_empty() {
        return Err("at least 1 test case is required".to_string());
    }
    if source.contains("host_memory_write") {
        return Err("tool source cannot call host_memory_write".to_string());
    }

    let started = Instant::now();
    {
        let compile_sandbox = SandboxDir::new(root_data_dir)?;
        check_script(&compile_sandbox.cfg, source)?;
    }

    let mut cases = Vec::with_capacity(tests.len());
    for tc in tests {
        let sandbox = SandboxDir::new(root_data_dir)?;
        for (rel_path, content) in &tc.setup_files {
            if rel_path.trim().is_empty() {
                return Err("setup_files path cannot be empty".to_string());
            }
            let full = sandbox_path(&sandbox.cfg.workspace_dir, rel_path)
                .map_err(|e| format!("invalid setup_files path '{rel_path}': {e}"))?;
            if full == sandbox.cfg.workspace_dir {
                return Err(format!("invalid setup_files path '{rel_path}'"));
            }
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create setup_files directory for '{rel_path}': {e}"))?;
            }
            std::fs::write(&full, content.as_bytes())
                .map_err(|e| format!("write setup_files '{rel_path}': {e}"))?;
        }

        let (ok, output, run_err) = match run_library_skill(&sandbox.cfg, source, &tc.args) {
            Ok(out) => (true, out, None),
            Err(err) => (false, String::new(), Some(err)),
        };

        let mut failures: Vec<String> = Vec::new();
        if ok != tc.expect_ok {
            let detail = run_err
                .as_ref()
                .map(|e| format!(" ({e})"))
                .unwrap_or_default();
            failures.push(format!(
                "expected ok={}, got ok={}{detail}",
                tc.expect_ok, ok
            ));
        }

        if let Some(ref needle) = tc.expect_output_contains {
            let matched =
                output.contains(needle) || run_err.as_deref().is_some_and(|e| e.contains(needle));
            if !matched {
                failures.push(format!("output did not contain {needle:?}"));
            }
        }

        for (rel_path, expected_substr) in &tc.expect_files {
            if rel_path.trim().is_empty() {
                return Err("expect_files path cannot be empty".to_string());
            }
            let full = sandbox_path(&sandbox.cfg.workspace_dir, rel_path)
                .map_err(|e| format!("invalid expect_files path '{rel_path}': {e}"))?;
            match std::fs::read_to_string(&full) {
                Ok(actual) => {
                    if !actual.contains(expected_substr) {
                        failures.push(format!(
                            "file '{rel_path}' did not contain {expected_substr:?}"
                        ));
                    }
                }
                Err(e) => {
                    failures.push(format!(
                        "expected file '{rel_path}' missing or unreadable: {e}"
                    ));
                }
            }
        }

        let passed = failures.is_empty();
        let error = if passed {
            run_err
        } else {
            Some(failures.join("; "))
        };
        cases.push(ToolTestCaseResult {
            name: tc.name.clone(),
            passed,
            ok,
            output,
            error,
        });
    }

    let passed = cases.iter().filter(|c| c.passed).count();
    let total = cases.len();
    let all_passed = total > 0 && passed == total;
    Ok(ToolTestReport {
        passed,
        total,
        all_passed,
        duration_ms: started.elapsed().as_millis() as u64,
        source_sha256: sha256_hex(source.as_bytes()),
        ran_at_ms: now_millis(),
        cases,
    })
}

pub fn tool_packages_path(root_data_dir: &Path) -> PathBuf {
    root_data_dir.join("tool_packages.json")
}

fn load_unlocked(root_data_dir: &Path) -> Vec<ToolPackage> {
    let path = tool_packages_path(root_data_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let arr = v["packages"].as_array().or_else(|| v.as_array());
    arr.map(|items| items.iter().filter_map(ToolPackage::from_json).collect())
        .unwrap_or_default()
}

fn save_unlocked(root_data_dir: &Path, packages: &[ToolPackage]) -> Result<(), String> {
    let payload = json!({
        "version": 1,
        "packages": packages.iter().map(ToolPackage::to_json).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| format!("serialize tool_packages.json: {e}"))?;
    atomic_write_bytes_with_mode(&tool_packages_path(root_data_dir), &bytes, Some(0o600))
        .map_err(|e| format!("write tool_packages.json: {e}"))
}

pub fn load_tool_packages(root_data_dir: &Path) -> Vec<ToolPackage> {
    with_tool_pkg_lock(|| load_unlocked(root_data_dir))
}

pub fn get_tool_package(root_data_dir: &Path, id: &str) -> Option<ToolPackage> {
    with_tool_pkg_lock(|| {
        load_unlocked(root_data_dir)
            .into_iter()
            .find(|p| p.id == id)
    })
}

pub fn list_tool_packages(root_data_dir: &Path) -> Vec<ToolPackage> {
    with_tool_pkg_lock(|| {
        let mut pkgs = load_unlocked(root_data_dir);
        pkgs.sort_by_key(|p| std::cmp::Reverse(p.updated_ms));
        pkgs
    })
}

/// Verify `source` against `tests` in the sandbox, stage the `ToolPackage`
/// as `pending_approval`, and park a `tool_offer` decision in `person_cfg.data_dir`.
/// Does NOT publish into `library.json` or install into `suite.json`.
#[allow(clippy::too_many_arguments)]
pub fn stage_tool_for_approval(
    root_data_dir: &Path,
    person_cfg: &Config,
    person_id: &str,
    thread_id: &str,
    name: &str,
    description: &str,
    params: Vec<String>,
    source: String,
    tests: Vec<ToolTestCase>,
) -> Result<ToolPackage, String> {
    let clean_name = clean_text(name, 80);
    if clean_name.is_empty() {
        return Err("tool name cannot be empty".to_string());
    }
    let clean_desc = clean_text(description, 300);
    let author = if person_id.trim().is_empty() {
        "default".to_string()
    } else {
        person_id.trim().to_string()
    };

    let report = verify_and_test_tool(root_data_dir, &params, &source, &tests)?;
    if !report.all_passed {
        let failing: Vec<String> = report
            .cases
            .iter()
            .filter(|c| !c.passed)
            .map(|c| {
                let reason = c.error.as_deref().unwrap_or("assertion failed");
                format!("{}: {reason}", c.name)
            })
            .collect();
        return Err(format!(
            "tool verification failed ({}/{} passed): {}",
            report.passed,
            report.total,
            failing.join("; ")
        ));
    }

    let now = now_millis();
    let seq = PKG_SEQ.fetch_add(1, Ordering::Relaxed);
    let slug: String = slugify(&clean_name).chars().take(32).collect();
    let slug = slug.trim_end_matches('-');
    let pkg_id = format!("tp-{slug}-{now}-{seq}");

    let mut pkg = ToolPackage {
        id: pkg_id.clone(),
        name: clean_name.clone(),
        description: clean_desc.clone(),
        author,
        params: params.clone(),
        source,
        tests,
        test_report: report.clone(),
        status: "pending_approval".to_string(),
        decision_id: String::new(),
        created_ms: now,
        updated_ms: now,
    };

    with_tool_pkg_lock(|| {
        let mut pkgs = load_unlocked(root_data_dir);
        pkgs.push(pkg.clone());
        save_unlocked(root_data_dir, &pkgs)
    })?;

    let detail = if clean_desc.is_empty() {
        format!(
            "Verified with {}/{} passing sandbox tests ({} ms). Approve to install into your suite and share with the company library.",
            report.passed, report.total, report.duration_ms
        )
    } else {
        format!(
            "{clean_desc} — Verified with {}/{} passing sandbox tests ({} ms). Approve to install into your suite and share with the company library.",
            report.passed, report.total, report.duration_ms
        )
    };
    let summary = format!("Would this help? New tool: {clean_name}");
    let extra = json!({
        "detail": detail,
        "severity": "info",
        "tool_package_id": pkg_id,
        "tool_name": clean_name,
        "description": clean_desc,
        "params": params,
        "test_report": report.to_json(),
    });
    let decision_id = add_decision_with(
        &person_cfg.data_dir,
        "tool_offer",
        &summary,
        None,
        thread_id,
        extra,
    );

    pkg.decision_id = decision_id.clone();
    pkg.updated_ms = now_millis();
    with_tool_pkg_lock(|| {
        let mut pkgs = load_unlocked(root_data_dir);
        if let Some(existing) = pkgs.iter_mut().find(|p| p.id == pkg.id) {
            existing.decision_id = decision_id;
            existing.updated_ms = pkg.updated_ms;
        }
        save_unlocked(root_data_dir, &pkgs)
    })?;

    Ok(pkg)
}

/// Settle an approved or denied `tool_offer` decision.
/// Denied offers mark the package `"denied"` without touching the library or suite.
/// Approved offers re-verify the source digest and test report, publish into the
/// shared company library, install into the person's suite, and mark `"approved"`.
pub fn settle_tool_offer_decision(
    root_data_dir: &Path,
    person_data_dir: &Path,
    library: &mut Library,
    decision: &Value,
    approve: bool,
) -> String {
    let Some(pkg_id) = decision["tool_package_id"]
        .as_str()
        .filter(|s| !s.is_empty())
    else {
        return "Could not settle tool offer: missing tool_package_id.".to_string();
    };

    let Some(pkg) = get_tool_package(root_data_dir, pkg_id) else {
        return "Could not settle tool offer: tool package not found.".to_string();
    };

    if !approve {
        with_tool_pkg_lock(|| {
            let mut pkgs = load_unlocked(root_data_dir);
            if let Some(p) = pkgs.iter_mut().find(|p| p.id == pkg_id) {
                p.status = "denied".to_string();
                p.updated_ms = now_millis();
            }
            let _ = save_unlocked(root_data_dir, &pkgs);
        });
        return "You said no — tool was not installed or shared.".to_string();
    }

    if pkg.status != "pending_approval" {
        return format!("Tool package is already '{}'.", pkg.status);
    }

    let digest_ok = sha256_hex(pkg.source.as_bytes()) == pkg.test_report.source_sha256;
    if !digest_ok || !pkg.test_report.all_passed || pkg.source.contains("host_memory_write") {
        return "Approved, but tool verification failed: source digest mismatch or tests did not pass."
            .to_string();
    }

    let tests_json: Vec<Value> = pkg.tests.iter().map(ToolTestCase::to_json).collect();
    let report_json = pkg.test_report.to_json();
    match library.publish_tool(
        &pkg.name,
        &pkg.description,
        pkg.params.clone(),
        pkg.source.clone(),
        tests_json,
        report_json,
        &pkg.author,
    ) {
        Ok(item) => {
            library.save(root_data_dir);
            with_suite_lock(|| {
                let mut suite = load_suite(person_data_dir);
                suite.install(&item.id);
                save_suite(person_data_dir, &suite);
            });
            with_tool_pkg_lock(|| {
                let mut pkgs = load_unlocked(root_data_dir);
                if let Some(p) = pkgs.iter_mut().find(|p| p.id == pkg_id) {
                    p.status = "approved".to_string();
                    p.updated_ms = now_millis();
                }
                let _ = save_unlocked(root_data_dir, &pkgs);
            });
            crate::connectors::dispatch_event_webhooks(
                root_data_dir,
                "tool.published",
                json!({
                    "id": item.id,
                    "name": item.name,
                    "kind": item.kind,
                    "version": item.version,
                    "author": item.author,
                    "tool_package_id": pkg_id,
                }),
            );
            format!(
                "Approved and installed \"{}\" into your suite and the company library.",
                item.name
            )
        }
        Err(e) => format!("Approved, but publishing failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::load_decisions;
    use std::sync::atomic::AtomicU64;

    static TEST_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_env(label: &str) -> (PathBuf, PathBuf, Config) {
        let seq = TEST_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ergon-tool-builder-{}-{}-{label}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::remove_dir_all(&root);
        let root_data_dir = root.join("root_data");
        let person_data_dir = root_data_dir.join("people").join("alice");
        let cfg = Config {
            data_dir: person_data_dir.clone(),
            workspace_dir: person_data_dir.join("workspace"),
            memory_path: person_data_dir.join("memory.md"),
            model_base_url: "http://127.0.0.1:9/v1".to_string(),
            model_id: "test".to_string(),
            api_key: None,
        };
        std::fs::create_dir_all(&root_data_dir).unwrap();
        cfg.ensure_dirs().unwrap();
        (root, root_data_dir, cfg)
    }

    fn word_counter_source() -> String {
        r#"pub fn main(path) {
    let text = host_workspace_read(path);
    if text.starts_with("error:") {
        return text;
    }
    let count = 0;
    for line in text.replace("\r\n", "\n").split("\n") {
        for part in line.split(" ") {
            if !part.trim().is_empty() {
                count += 1;
            }
        }
    }
    let res = host_workspace_write("word_count.txt", `words=${count}`);
    if res != "ok" {
        return res;
    }
    `words=${count}`
}"#
        .to_string()
    }

    #[test]
    fn verify_and_test_tool_runs_sandboxed_cases_and_cleans_up() {
        let (root, root_data, _) = temp_env("verify");
        let mut setup = BTreeMap::new();
        setup.insert("notes.txt".to_string(), "one two three four".to_string());
        let mut expect_files = BTreeMap::new();
        expect_files.insert("word_count.txt".to_string(), "words=4".to_string());

        let tests = vec![
            ToolTestCase {
                name: "counts words and writes output".to_string(),
                setup_files: setup,
                args: vec!["notes.txt".to_string()],
                expect_ok: true,
                expect_output_contains: Some("words=4".to_string()),
                expect_files,
            },
            ToolTestCase {
                name: "missing file returns error".to_string(),
                setup_files: BTreeMap::new(),
                args: vec!["absent.txt".to_string()],
                expect_ok: false,
                expect_output_contains: Some("read absent.txt".to_string()),
                expect_files: BTreeMap::new(),
            },
        ];

        let report = verify_and_test_tool(
            &root_data,
            &["path".to_string()],
            &word_counter_source(),
            &tests,
        )
        .unwrap();

        assert!(report.all_passed, "{report:?}");
        assert_eq!(report.passed, 2);
        assert_eq!(report.total, 2);
        assert_eq!(
            report.source_sha256,
            sha256_hex(word_counter_source().as_bytes())
        );

        let sandbox_entries = std::fs::read_dir(root_data.join(".tool_sandbox"))
            .map(|rd| rd.count())
            .unwrap_or(0);
        assert_eq!(sandbox_entries, 0, "RAII sandbox dirs must be cleaned up");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn verify_rejects_invalid_inputs_and_traversal() {
        let (root, root_data, _) = temp_env("rejects");
        let tc = ToolTestCase {
            name: "basic".to_string(),
            setup_files: BTreeMap::new(),
            args: vec![],
            expect_ok: true,
            expect_output_contains: None,
            expect_files: BTreeMap::new(),
        };

        // > 5 params
        let six = vec![
            "a".into(),
            "b".into(),
            "c".into(),
            "d".into(),
            "e".into(),
            "f".into(),
        ];
        assert!(
            verify_and_test_tool(&root_data, &six, "pub fn main() { \"ok\" }", &[tc.clone()])
                .is_err()
        );

        // empty source
        assert!(verify_and_test_tool(&root_data, &[], "   ", &[tc.clone()]).is_err());

        // empty tests
        assert!(verify_and_test_tool(&root_data, &[], "pub fn main() { \"ok\" }", &[]).is_err());

        // host_memory_write banned
        assert!(verify_and_test_tool(
            &root_data,
            &[],
            "pub fn main() { host_memory_write(\"x\") }",
            &[tc]
        )
        .is_err());

        // traversal in setup_files rejected
        let mut evil_setup = BTreeMap::new();
        evil_setup.insert("../escape.txt".to_string(), "pwn".to_string());
        let evil_tc = ToolTestCase {
            name: "traversal".to_string(),
            setup_files: evil_setup,
            args: vec![],
            expect_ok: true,
            expect_output_contains: None,
            expect_files: BTreeMap::new(),
        };
        assert!(
            verify_and_test_tool(&root_data, &[], "pub fn main() { \"ok\" }", &[evil_tc]).is_err()
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn stage_and_settle_approve_and_deny_flows() {
        let (root, root_data, person_cfg) = temp_env("stage-settle");
        let mut setup = BTreeMap::new();
        setup.insert("doc.txt".to_string(), "alpha beta".to_string());
        let tc_pass = ToolTestCase {
            name: "passes".to_string(),
            setup_files: setup,
            args: vec!["doc.txt".to_string()],
            expect_ok: true,
            expect_output_contains: Some("words=2".to_string()),
            expect_files: BTreeMap::new(),
        };
        let tc_fail = ToolTestCase {
            name: "fails".to_string(),
            setup_files: BTreeMap::new(),
            args: vec!["missing.txt".to_string()],
            expect_ok: true,
            expect_output_contains: None,
            expect_files: BTreeMap::new(),
        };

        // Failing test refuses to stage or park decision.
        let err = stage_tool_for_approval(
            &root_data,
            &person_cfg,
            "alice",
            "t1",
            "Word Counter",
            "Counts words in a file",
            vec!["path".into()],
            word_counter_source(),
            vec![tc_fail],
        )
        .unwrap_err();
        assert!(err.contains("fails"), "{err}");
        assert!(load_tool_packages(&root_data).is_empty());
        assert!(load_decisions(&person_cfg.data_dir).is_empty());

        // Passing test stages package and parks tool_offer decision without touching library/suite.
        let pkg = stage_tool_for_approval(
            &root_data,
            &person_cfg,
            "alice",
            "t1",
            "Word Counter",
            "Counts words in a file",
            vec!["path".into()],
            word_counter_source(),
            vec![tc_pass.clone()],
        )
        .unwrap();
        assert_eq!(pkg.status, "pending_approval");
        assert!(!pkg.decision_id.is_empty());

        let mut lib = Library::load(&root_data);
        assert!(lib.get("word-counter").is_none());
        assert!(!load_suite(&person_cfg.data_dir)
            .installed
            .contains(&"word-counter".to_string()));

        let decisions = load_decisions(&person_cfg.data_dir);
        assert_eq!(decisions.len(), 1);
        let d = &decisions[0];
        assert_eq!(d["kind"], "tool_offer");
        assert_eq!(d["summary"], "Would this help? New tool: Word Counter");
        assert!(!d["detail"].as_str().unwrap().contains("Rune"));

        // Approve settles into library + suite + package status "approved".
        let outcome =
            settle_tool_offer_decision(&root_data, &person_cfg.data_dir, &mut lib, d, true);
        assert!(outcome.contains("Approved and installed"), "{outcome}");
        let published = lib.get("word-counter").unwrap();
        assert_eq!(published.kind, "tool");
        assert_eq!(published.author, "alice");
        assert_eq!(published.tests.len(), 1);
        assert!(published.test_report.is_some());
        assert!(load_suite(&person_cfg.data_dir)
            .installed
            .contains(&"word-counter".to_string()));
        assert_eq!(
            get_tool_package(&root_data, &pkg.id).unwrap().status,
            "approved"
        );

        // Second tool -> Deny marks "denied" and does not publish or install.
        let pkg2 = stage_tool_for_approval(
            &root_data,
            &person_cfg,
            "alice",
            "t2",
            "Second Tool",
            "Another tool",
            vec!["path".into()],
            word_counter_source(),
            vec![tc_pass],
        )
        .unwrap();
        let decisions2 = load_decisions(&person_cfg.data_dir);
        let d2 = decisions2
            .iter()
            .find(|x| x["id"].as_str() == Some(&pkg2.decision_id))
            .unwrap();
        let deny_msg =
            settle_tool_offer_decision(&root_data, &person_cfg.data_dir, &mut lib, d2, false);
        assert_eq!(deny_msg, "You said no — tool was not installed or shared.");
        assert!(lib.get("second-tool").is_none());
        assert_eq!(
            get_tool_package(&root_data, &pkg2.id).unwrap().status,
            "denied"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
