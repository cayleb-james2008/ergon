//! Resumable Hybrid Workflow & Pipeline Engine (`pipeline_runs.json`).
//!
//! Executes typed multi-step pipelines (`skill`, `agent`, `checkpoint`, `export`)
//! asynchronously on worker threads, persists step-by-step state atomically,
//! pauses cleanly at human checkpoints (`paused_checkpoint`), survives server
//! restarts, and resumes downstream steps with `{{steps.<id>.human_answer}}`
//! and `{{steps.<id>.files[0]}}` template resolution.

pub(super) use crate::connectors;
pub(super) use crate::receipts;
pub(super) use crate::records;
pub(super) use crate::routines;
pub(super) use crate::scheduler;
pub(super) use crate::search::SearchCache;
pub(super) use crate::skills;
pub(super) use crate::sse::RunEventBus;
pub(super) use crate::tools::{
    self, atomic_write_bytes_with_mode, sandbox_path, sandbox_path_resolving, ToolCtx,
};
pub(super) use crate::{
    run_agent_for_pipeline_step, AgentLoopOptions, AgentSnapshot, CheckpointPause, Config,
    LiveRuns, SharedStore,
};
pub(super) use serde_json::{json, Map, Value};
pub(super) use std::collections::{HashMap, HashSet, VecDeque};
pub(super) use std::path::{Path, PathBuf};
pub(super) use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
pub(super) use std::sync::{mpsc, Arc, Mutex};
pub(super) use std::time::{Duration, Instant};

pub const MAX_PIPELINE_STEPS: usize = 24;
pub const MAX_FANOUT_ITEMS: usize = 50;
pub const MAX_DAG_CONCURRENCY: usize = 4;
pub(super) const MAX_STORED_RUNS: usize = 200;
pub const MAX_TEMPLATE_DEPTH: usize = 16;
pub const MAX_TEMPLATE_EXPANSIONS: usize = 128;
pub const MAX_TEMPLATE_OUTPUT_BYTES: usize = 256 * 1024;
pub(super) const MAX_SUBPIPELINE_DEPTH: usize = 8;

pub(super) const DAG_META_KEY: &str = "__ergon_dag";
pub(super) const CHILD_ITEMS_KEY: &str = "__child_items";
pub(super) const SNAPSHOT_INPUT_KEY: &str = "__ergon_agent_snapshot";
pub(super) const SNAPSHOTS_INPUT_KEY: &str = "__ergon_step_snapshots";
pub(super) const CHECKPOINTS_INPUT_KEY: &str = "__ergon_pending_checkpoints";
pub(super) const SUBPIPELINE_DEPTH_KEY: &str = "__ergon_subpipeline_depth";
pub(super) const SUBPIPELINE_STACK_KEY: &str = "__ergon_subpipeline_stack";
pub(super) const ROUTINE_TRIGGER_KEY: &str = "__ergon_routine_trigger";
pub(super) const ITEM_INPUT_KEY: &str = "__ergon_item";
pub(super) const ITEM_INDEX_INPUT_KEY: &str = "__ergon_item_index";

pub(super) static PIPELINE_SEQ: AtomicU64 = AtomicU64::new(0);
pub(super) static CANCEL_GEN_SEQ: AtomicU64 = AtomicU64::new(1);
pub(super) static PIPELINES_LOCK: Mutex<()> = Mutex::new(());
pub(super) type ActiveCancelMap = HashMap<String, (u64, Arc<AtomicBool>)>;

pub(super) static ACTIVE_CANCELS: std::sync::OnceLock<Mutex<ActiveCancelMap>> =
    std::sync::OnceLock::new();
pub(super) static ACTIVE_SUBPIPELINE_CHILDREN: std::sync::OnceLock<Mutex<HashMap<String, Vec<String>>>> =
    std::sync::OnceLock::new();
pub(super) static RECOVERED_DIRS: std::sync::OnceLock<Mutex<HashSet<PathBuf>>> =
    std::sync::OnceLock::new();

pub(super) fn with_pipelines_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = PIPELINES_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

pub(super) fn cancel_registry() -> &'static Mutex<HashMap<String, (u64, Arc<AtomicBool>)>> {
    ACTIVE_CANCELS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn subpipeline_children_registry() -> &'static Mutex<HashMap<String, Vec<String>>> {
    ACTIVE_SUBPIPELINE_CHILDREN.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn recovered_dirs() -> &'static Mutex<HashSet<PathBuf>> {
    RECOVERED_DIRS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn pipeline_runs_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("pipeline_runs.json")
}


mod executor;
mod spec;
mod store;

pub use executor::*;
pub use spec::*;
pub use store::*;
