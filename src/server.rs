//! Embedded HTTP server, request parser, security gates, and API route handlers.

use crate::tools::{atomic_write_bytes, string_list};
use crate::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

// ---------------------------------------------------------------------------
// HTTP server (hand-rolled, std TcpListener)
// ---------------------------------------------------------------------------

/// One person's live state: their scoped config + their thread store.
pub struct PersonRuntime {
    pub cfg: Config,
    pub store: SharedStore,
}

fn load_runtime(root: &Config, person: &Person, library: &library::Library) -> PersonRuntime {
    let cfg = root.person_config(person);
    let fresh_memory = !cfg.memory_path.exists();
    let _ = cfg.ensure_dirs();
    // Their suite exists from the first moment too: drawn from the
    // company library, shaped by their role.
    library::ensure_suite(&cfg.data_dir, person, library);
    if fresh_memory && person.id != "default" {
        // Seed a new person's memory with who they are, so the agent
        // starts out knowing its person.
        let mut seed = format!("# Memory\n\n## About\n- Name: {}", person.name);
        if !person.role.is_empty() {
            seed.push_str(&format!("\n- Role: {}", person.role));
        }
        seed.push_str("\n\n## Learned\n");
        seed.push('\n');
        let _ = std::fs::write(&cfg.memory_path, seed);
    }
    let threads_path = cfg.data_dir.join("threads.json");
    PersonRuntime {
        cfg,
        store: Arc::new(Mutex::new(Store::load(&threads_path))),
    }
}

#[derive(Clone, Debug)]
pub struct LiveModelConfig {
    pub model_base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub auto_detected: bool,
}

pub struct AppState {
    /// Root config (data dir + startup model route).
    pub cfg: Config,
    /// Live mutable model configuration updated by POST /api/config without restarting.
    pub model_cfg: Arc<Mutex<LiveModelConfig>>,
    /// The company library — one shared store of skills, subagents
    /// and apps that the agents themselves maintain.
    pub library: Arc<Mutex<library::Library>>,
    /// The hosting-layer gate token (ERGON_GATE_TOKEN), when set:
    /// every request must carry it as the `X-Ergon-Gate` header. The
    /// hosting layer authenticates people; Ergon itself has no
    /// accounts and no passwords.
    pub gate_token: Option<String>,
    /// Extra Host names accepted without a gate token (ERGON_ALLOWED_HOSTS).
    pub allowed_hosts: Vec<String>,
    pub people: Arc<RwLock<PeopleState>>,
    pub runtimes: Arc<RwLock<HashMap<String, Arc<PersonRuntime>>>>,
    /// In-flight runs per person for live step progress streaming (`GET /api/runs/status` & `/api/runs/stream`).
    pub live_runs: LiveRuns,
    /// Broadcast SSE event bus for real-time run events (`GET /api/runs/stream`).
    pub events: sse::RunEventBus,
    /// Per-person RwLock-cached search index, SHA-256 receipt chain, and file listing.
    pub search_cache: search::SearchCache,
    /// Browser sessions (cookie id → person id). "Active person" used
    /// to be one global pointer: any client could switch it for every
    /// other client mid-task. A session pins the person per browser;
    /// cookie-less API clients keep the global-pointer behaviour.
    pub sessions: Arc<RwLock<HashMap<String, String>>>,
    /// Checks GitHub for newer releases and installs them on request.
    pub updater: Arc<update::Updater>,
    /// Whether the background routine scheduler thread is active (false when ERGON_DISABLE_SCHEDULER=1).
    pub scheduler_enabled: bool,
    /// Per-routine in-flight guard so overlapping ticks never double-fire the same routine.
    pub running_routines: Arc<Mutex<std::collections::HashSet<String>>>,
    /// Perimeter honeytraps: decoys, breadcrumbs, honeytoken use, bans.
    pub tripwire: Arc<tripwire::Perimeter>,
}

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Session ids are the only credential a browser presents, so they
/// must be unguessable: random bytes from the OS, not a timestamp and
/// a counter (an early version's ids were predictable to within a few
/// guesses of a victim's visit time).
fn new_session_id() -> String {
    let n = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut bytes = [0u8; 16];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut bytes))
        .is_ok();
    if filled {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        format!("s{hex}-{n}")
    } else {
        // No OS randomness available: fall back to a wide mix of
        // time, pid and counter — still far harder to bracket than a
        // bare millisecond timestamp.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("s{:x}-{:x}-{n}", nanos, std::process::id())
    }
}

/// Whether this session id is one the server actually issued. A
/// client-chosen id is never honoured (session fixation): unknown ids
/// are treated as "no session" and replaced with a fresh server id.
/// Insert a session, bounding the map: sessions are memory-only and
/// re-issued transparently, so past a generous cap the map is simply
/// cleared rather than growing for the life of the process.
fn insert_session(state: &AppState, sid: String, person_id: String) {
    let mut sessions = state.sessions.write().unwrap();
    if sessions.len() > 10_000 {
        sessions.clear();
    }
    sessions.insert(sid, person_id);
}

fn session_known(state: &AppState, session: &Option<String>) -> bool {
    match session {
        Some(sid) => state.sessions.read().unwrap().contains_key(sid),
        None => false,
    }
}

/// The person this request acts for: the session's person when the
/// caller presented a known session cookie, else the global active.
fn effective_person(state: &AppState, session: &Option<String>) -> Person {
    if let Some(sid) = session {
        let mapped = state.sessions.read().unwrap().get(sid).cloned();
        if let Some(pid) = mapped {
            let people = state.people.read().unwrap();
            if let Some(p) = people.people.iter().find(|p| p.id == pid) {
                return p.clone();
            }
        }
    }
    state.people.read().unwrap().active_person()
}

fn runtime_for(state: &AppState, person: &Person) -> Arc<PersonRuntime> {
    if let Some(rt) = state.runtimes.read().unwrap().get(&person.id) {
        return rt.clone();
    }
    let mut map = state.runtimes.write().unwrap();
    if let Some(rt) = map.get(&person.id) {
        return rt.clone();
    }
    let lib = state.library.lock().unwrap().clone();
    let rt = Arc::new(load_runtime(&state.cfg, person, &lib));
    map.insert(person.id.clone(), rt.clone());
    rt
}

pub fn serve(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    serve_env(listener, cfg, false)
}

/// What the `ergon` binary runs: `serve` plus the periodic check for a newer
/// GitHub release (see `update`). Tests use `serve`, which never calls out.
pub fn serve_with_updates(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    serve_env(listener, cfg, true)
}

fn serve_env(listener: TcpListener, cfg: Config, background_updates: bool) -> std::io::Result<()> {
    let gate = std::env::var("ERGON_GATE_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let hosts = allowed_hosts_from(&std::env::var("ERGON_ALLOWED_HOSTS").unwrap_or_default());
    serve_inner(listener, cfg, gate, hosts, background_updates)
}

/// Parses ERGON_ALLOWED_HOSTS: comma-separated host names, with or without
/// ports, case-insensitive. Each entry is kept as its bare host name.
pub fn allowed_hosts_from(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|h| host_name(&h.trim().to_ascii_lowercase()))
        .filter(|h| !h.is_empty())
        .collect()
}

pub fn serve_with_gate(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
) -> std::io::Result<()> {
    serve_with_hosts(listener, cfg, gate_token, Vec::new())
}

/// Serves with explicit gate token and extra allowed Host names.
pub fn serve_with_hosts(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    allowed_hosts: Vec<String>,
) -> std::io::Result<()> {
    serve_inner(listener, cfg, gate_token, allowed_hosts, false)
}

fn scheduler_tick_ms() -> u64 {
    std::env::var("ERGON_SCHEDULER_TICK_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ms| ms.clamp(250, 300_000))
        .unwrap_or(15_000)
}

fn is_history_security_error(err: &str) -> bool {
    let low = err.to_ascii_lowercase();
    low.contains("escapes")
        || low.contains("invalid")
        || low.contains("absolute")
        || low.contains("symlink")
        || low.contains("hidden")
        || low.contains("control")
        || low.contains("traversal")
        || low.contains("nul")
        || low.contains("workspace")
}

fn latest_receipt_id_for_thread(person_data_dir: &Path, thread_id: &str) -> Option<String> {
    let chain = receipts::load_chain(person_data_dir);
    chain
        .iter()
        .rev()
        .find(|r| r.thread_id == thread_id)
        .map(|r| r.id.clone())
}

struct RoutineExecResult {
    http_status: u16,
    routine: Value,
    thread_id: String,
    thread: Value,
    run_record: scheduler::RoutineRunRecord,
    error: Option<String>,
}

fn execute_routine_run(
    state: &AppState,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    id: &str,
    trigger: &str,
) -> Option<RoutineExecResult> {
    let routine = routines::get_routine(&runtime.cfg.data_dir, id)?;
    let lock_key = format!("{}:{}", person.id, id);
    {
        let mut active = state
            .running_routines
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if !active.insert(lock_key.clone()) {
            let now_ms = records::now_millis();
            let rec = scheduler::new_run_record(
                id,
                routine["name"].as_str().unwrap_or("Routine"),
                now_ms,
                now_ms,
                "error",
                None,
                None,
                "Routine is already running.",
                trigger,
                Some("routine is already running".to_string()),
            );
            return Some(RoutineExecResult {
                http_status: 409,
                routine,
                thread_id: String::new(),
                thread: Value::Null,
                run_record: rec,
                error: Some("routine is already running".to_string()),
            });
        }
    }
    struct RunSlotGuard<'a>(&'a Mutex<std::collections::HashSet<String>>, String);
    impl Drop for RunSlotGuard<'_> {
        fn drop(&mut self) {
            let mut active = self.0.lock().unwrap_or_else(|p| p.into_inner());
            active.remove(&self.1);
        }
    }
    let _slot_guard = RunSlotGuard(&state.running_routines, lock_key);

    let name = routine["name"].as_str().unwrap_or("Routine").to_string();
    let prompt = routine["prompt"].as_str().unwrap_or("").to_string();
    let pack_deliverables = routine["pack_deliverables"].as_bool().unwrap_or(false);
    let mut steps: Vec<String> = routine["steps"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if steps.is_empty() && !prompt.trim().is_empty() {
        steps.push(prompt.clone());
    }
    let started_ms = records::now_millis();
    let cfg = effective_cfg(state, runtime);
    let ctx = ToolCtx {
        library: state.library.clone(),
        root_data_dir: state.cfg.data_dir.clone(),
        person_id: person.id.clone(),
    };
    let thread_id = {
        let mut store = runtime.store.lock().unwrap();
        let tid = store.create_thread();
        store.push(&tid, "user", &format!("{name} — {prompt}"));
        tid
    };

    if !cfg.model_configured() {
        runtime
            .store
            .lock()
            .unwrap()
            .push(&thread_id, "error", NO_KEY_ERROR);
        let finished_ms = records::now_millis();
        let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "no_model")
            .unwrap_or(routine);
        let rec = scheduler::new_run_record(
            id,
            &name,
            started_ms,
            finished_ms,
            "error",
            None,
            Some(thread_id.clone()),
            NO_KEY_ERROR,
            trigger,
            Some(NO_KEY_ERROR.to_string()),
        );
        let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
        let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
        let enriched = scheduler::enrich_routine_json(&updated, finished_ms, &hist);
        return Some(RoutineExecResult {
            http_status: 503,
            routine: enriched,
            thread_id: thread_id.clone(),
            thread: json!({"id": thread_id}),
            run_record: rec,
            error: Some(NO_KEY_ERROR.to_string()),
        });
    }

    let total_steps = steps.len();
    let mut history: Vec<Value> = Vec::new();
    let mut outcome: Result<AgentOutcome, String> = Err("routine has no steps".to_string());
    let mut combined_summaries: Vec<String> = Vec::new();
    for (idx, step_prompt) in steps.iter().enumerate() {
        let task_msg = if total_steps > 1 {
            format!(
                "[Playbook step {}/{total_steps} for \"{name}\"] {step_prompt}",
                idx + 1
            )
        } else {
            step_prompt.clone()
        };
        if idx > 0 {
            runtime.store.lock().unwrap().push(&thread_id, "user", &task_msg);
        }
        let step_res = run_agent_live(
            &cfg,
            &ctx,
            &runtime.store,
            &thread_id,
            &task_msg,
            &history,
            Some((&state.live_runs, &state.events, &person.id)),
        );
        match step_res {
            Ok(out) => {
                history.push(json!({"role": "user", "content": task_msg}));
                history.push(json!({"role": "assistant", "content": out.summary}));
                combined_summaries.push(out.summary.clone());
                outcome = Ok(out);
            }
            Err(e) => {
                outcome = Err(e);
                break;
            }
        }
    }
    if total_steps > 1 {
        if let Ok(ref mut out) = outcome {
            out.summary = combined_summaries.join("\n\n");
        }
    }
    state.search_cache.invalidate(&person.id);
    if outcome.is_ok() && pack_deliverables {
        let idx = state.search_cache.get(
            &person.id,
            &runtime.cfg.data_dir,
            &runtime.cfg.workspace_dir,
            &runtime.cfg.memory_path,
        );
        let (_, items) = idx.list_files(0, Some(15), "");
        let pack_files: Vec<String> = items
            .iter()
            .filter_map(|item| item["path"].as_str().map(|s| s.to_string()))
            .filter(|p| p != "Playbook Deliverable Pack.md")
            .collect();
        if let Ok((md, pack_sha, _, _)) = build_workspace_pack(
            &runtime.cfg.workspace_dir,
            &person.name,
            &format!("{name} — Deliverable Pack"),
            &pack_files,
            false,
        ) {
            if let Ok(full) = sandbox_path(&runtime.cfg.workspace_dir, "Playbook Deliverable Pack.md") {
                if !full.symlink_metadata().is_ok_and(|m| m.file_type().is_symlink()) {
                    let _ = file_history::record_revision(
                        &runtime.cfg.data_dir,
                        &runtime.cfg.workspace_dir,
                        "Playbook Deliverable Pack.md",
                        "playbook",
                    );
                    if atomic_write_bytes(&full, md.as_bytes()).is_ok() {
                        state.search_cache.invalidate(&person.id);
                        runtime.store.lock().unwrap().push(
                            &thread_id,
                            "step",
                            &format!(
                                "wrote Playbook Deliverable Pack.md ({} bytes, sha256:{pack_sha})",
                                md.len()
                            ),
                        );
                    }
                }
            }
        }
    }
    let finished_ms = records::now_millis();
    let receipt_id = latest_receipt_id_for_thread(&runtime.cfg.data_dir, &thread_id);

    match outcome {
        Ok(agent_out) => {
            let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, "finished")
                .unwrap_or(routine);
            let rec = scheduler::new_run_record(
                id,
                &name,
                started_ms,
                finished_ms,
                "ok",
                receipt_id.clone(),
                Some(thread_id.clone()),
                &agent_out.summary,
                trigger,
                None,
            );
            let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
            let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let enriched = scheduler::enrich_routine_with_verification(
                &updated,
                finished_ms,
                &hist,
                &idx.chain,
                idx.chain_valid,
            );
            state.events.publish(
                "routine_fired",
                &person.id,
                &rec.run_id,
                &thread_id,
                json!({
                    "routine_id": id,
                    "routine_name": name,
                    "trigger": trigger,
                    "status": "ok",
                    "thread_id": thread_id,
                    "receipt_id": receipt_id,
                    "duration_ms": rec.duration_ms,
                    "summary": truncate(&agent_out.summary, 240),
                    "next_run_ms": enriched["next_run_ms"],
                    "ts_ms": finished_ms,
                }),
            );
            let store = runtime.store.lock().unwrap();
            let thread_v = store
                .thread_json(&thread_id)
                .unwrap_or_else(|| json!({"id": thread_id}));
            Some(RoutineExecResult {
                http_status: 200,
                routine: enriched,
                thread_id,
                thread: thread_v,
                run_record: rec,
                error: None,
            })
        }
        Err(e) => {
            let is_cancel = e == "run cancelled by user";
            let st = if is_cancel { "cancelled" } else { "error" };
            runtime.store.lock().unwrap().push(&thread_id, "error", &e);
            let updated = routines::record_run(&runtime.cfg.data_dir, id, &thread_id, st)
                .unwrap_or(routine);
            let rec = scheduler::new_run_record(
                id,
                &name,
                started_ms,
                finished_ms,
                st,
                receipt_id.clone(),
                Some(thread_id.clone()),
                &e,
                trigger,
                Some(e.clone()),
            );
            let _ = scheduler::record_run_history(&runtime.cfg.data_dir, rec.clone());
            let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(id), 20);
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let enriched = scheduler::enrich_routine_with_verification(
                &updated,
                finished_ms,
                &hist,
                &idx.chain,
                idx.chain_valid,
            );
            state.events.publish(
                "routine_fired",
                &person.id,
                &rec.run_id,
                &thread_id,
                json!({
                    "routine_id": id,
                    "routine_name": name,
                    "trigger": trigger,
                    "status": st,
                    "thread_id": thread_id,
                    "receipt_id": receipt_id,
                    "duration_ms": rec.duration_ms,
                    "error": truncate(&e, 240),
                    "next_run_ms": enriched["next_run_ms"],
                    "ts_ms": finished_ms,
                }),
            );
            Some(RoutineExecResult {
                http_status: 500,
                routine: enriched,
                thread_id: thread_id.clone(),
                thread: json!({"id": thread_id}),
                run_record: rec,
                error: Some(e),
            })
        }
    }
}

fn start_scheduler_thread(state: &Arc<AppState>) {
    let weak = Arc::downgrade(state);
    let tick_ms = scheduler_tick_ms();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(tick_ms));
            let Some(st) = weak.upgrade() else {
                break;
            };
            let people: Vec<Person> = st
                .people
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .people
                .clone();
            let now_ms = records::now_millis();
            for person in people {
                let p_cfg = st.cfg.person_config(&person);
                if !routines::routines_path(&p_cfg.data_dir).exists() {
                    continue;
                }
                let rt = runtime_for(&st, &person);
                let eff = effective_cfg(&st, &rt);
                if !eff.model_configured() {
                    continue;
                }
                let list = routines::list_routines(&rt.cfg.data_dir);
                let due = scheduler::find_due_routines(&list, now_ms);
                for r in due {
                    let Some(rid) = r["id"].as_str().map(|s| s.to_string()) else {
                        continue;
                    };
                    let lock_key = format!("{}:{}", person.id, rid);
                    let already = st
                        .running_routines
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .contains(&lock_key);
                    if already {
                        continue;
                    }
                    let st_run = st.clone();
                    let person_run = person.clone();
                    let rt_run = rt.clone();
                    std::thread::spawn(move || {
                        let _ = execute_routine_run(&st_run, &person_run, &rt_run, &rid, "scheduled");
                    });
                }
            }
        }
    });
}

/// `background_updates` starts the periodic release check; only the real
/// binary does (tests must not call out to GitHub).
fn serve_inner(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    allowed_hosts: Vec<String>,
    background_updates: bool,
) -> std::io::Result<()> {
    cfg.ensure_dirs()?;
    // The agent may never fetch this server's own API (egress guard).
    if let Ok(addr) = listener.local_addr() {
        tripwire::register_self_port(addr.port());
    }
    let perimeter = tripwire::Perimeter::new(&cfg.data_dir);
    let gate_token_is_none = gate_token.is_none();
    let people = PeopleState::load(&cfg);
    let library = library::Library::load(&cfg.data_dir);
    let auto_detected = cfg.api_key.is_none() && cfg.model_configured();
    let model_cfg = Arc::new(Mutex::new(LiveModelConfig {
        model_base_url: cfg.model_base_url.clone(),
        model_id: cfg.model_id.clone(),
        api_key: cfg.api_key.clone(),
        auto_detected,
    }));
    let scheduler_disabled = scheduler::is_scheduler_disabled_by_env();
    let state = Arc::new(AppState {
        cfg,
        model_cfg,
        library: Arc::new(Mutex::new(library)),
        gate_token,
        allowed_hosts,
        people: Arc::new(RwLock::new(people)),
        runtimes: Arc::new(RwLock::new(HashMap::new())),
        live_runs: Arc::new(Mutex::new(HashMap::new())),
        events: sse::RunEventBus::new(),
        search_cache: search::SearchCache::new(),
        sessions: Arc::new(RwLock::new(HashMap::new())),
        updater: update::Updater::from_env("ERGON", "ergon", env!("CARGO_PKG_VERSION"), "cayleb-james2008/ergon", gate_token_is_none),
        scheduler_enabled: !scheduler_disabled,
        running_routines: Arc::new(Mutex::new(std::collections::HashSet::new())),
        tripwire: perimeter,
    });
    if background_updates {
        state.updater.start_background();
    }
    if state.scheduler_enabled {
        start_scheduler_thread(&state);
    }
    const MAX_ACTIVE_CONNS: usize = 128;
    const WARM_WORKER_FLOOR: usize = 4;
    let active_conns = Arc::new(AtomicUsize::new(0));
    struct ConnGuard(Arc<AtomicUsize>);
    impl Drop for ConnGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::Relaxed);
        }
    }
    struct PoolInner {
        queue: VecDeque<(std::net::TcpStream, ConnGuard)>,
        idle_workers: usize,
        total_workers: usize,
    }
    let pool = Arc::new((
        Mutex::new(PoolInner {
            queue: VecDeque::with_capacity(64),
            idle_workers: 0,
            total_workers: 0,
        }),
        Condvar::new(),
    ));

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let prev = active_conns.fetch_add(1, Ordering::Relaxed);
                if prev >= MAX_ACTIVE_CONNS {
                    active_conns.fetch_sub(1, Ordering::Relaxed);
                    let body = r#"{"error":"server busy — too many concurrent connections"}"#;
                    let resp = format!(
                        "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    continue;
                }
                let guard = ConnGuard(active_conns.clone());
                let (lock, cvar) = &*pool;
                let mut inner = lock.lock().unwrap_or_else(|p| p.into_inner());
                inner.queue.push_back((stream, guard));
                if inner.idle_workers == 0 && inner.total_workers < MAX_ACTIVE_CONNS {
                    inner.total_workers += 1;
                    let pool_w = pool.clone();
                    let state_w = state.clone();
                    let _ = std::thread::Builder::new()
                        .name("ergon-http".to_string())
                        .stack_size(2 * 1024 * 1024)
                        .spawn(move || {
                        struct ExitGuard<'a>(&'a (Mutex<PoolInner>, Condvar));
                        impl Drop for ExitGuard<'_> {
                            fn drop(&mut self) {
                                let (l, _) = self.0;
                                let mut g = l.lock().unwrap_or_else(|p| p.into_inner());
                                g.total_workers = g.total_workers.saturating_sub(1);
                            }
                        }
                        let _exit = ExitGuard(&pool_w);
                        let (l, cv) = &*pool_w;
                        loop {
                            let job = {
                                let mut g = l.lock().unwrap_or_else(|p| p.into_inner());
                                loop {
                                    if let Some(item) = g.queue.pop_front() {
                                        break Some(item);
                                    }
                                    g.idle_workers += 1;
                                    let (next_g, wait_res) = cv
                                        .wait_timeout(g, std::time::Duration::from_secs(10))
                                        .unwrap_or_else(|p| p.into_inner());
                                    g = next_g;
                                    g.idle_workers = g.idle_workers.saturating_sub(1);
                                    if g.queue.is_empty()
                                        && wait_res.timed_out()
                                        && g.total_workers > WARM_WORKER_FLOOR
                                    {
                                        break None;
                                    }
                                }
                            };
                            let Some((conn, conn_guard)) = job else {
                                break;
                            };
                            let _ = handle_connection(conn, &state_w);
                            drop(conn_guard);
                        }
                    });
                }
                cvar.notify_one();
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) query: String,
    pub(crate) body: Vec<u8>,
    /// The caller's `ergon_session` cookie, if they presented one.
    pub(crate) session: Option<String>,
    /// The hosting layer's gate header, if presented.
    pub(crate) gate: Option<String>,
    /// Browser-set headers the cross-site checks read (lowercased).
    pub(crate) content_type: String,
    pub(crate) origin: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) sec_fetch_site: Option<String>,
    /// Headers the tripwire reads (honeytoken use, decoy cookie, proxies).
    pub(crate) authorization: Option<String>,
    pub(crate) user_agent: Option<String>,
    pub(crate) cookie_raw: Option<String>,
    pub(crate) forwarded_for: Option<String>,
    /// Content negotiation & conditional GET headers.
    pub(crate) accept_encoding: Option<String>,
    pub(crate) if_none_match: Option<String>,
    pub(crate) keep_alive: bool,
}

thread_local! {
    static KEEP_ALIVE_CONN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static JSON_SCRATCH: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(4096));
}

/// Bodies are small JSON. A request claiming a larger one is dropped
/// before any allocation: an unbounded `vec![0u8; content_length]`
/// here once let a single header (Content-Length: 2^40, no body)
/// abort the whole server process.
const MAX_BODY_BYTES: usize = 1_048_576;
/// File uploads only: a 25 MB file, base64-encoded inside JSON. Every
/// other route keeps the 1 MB cap.
const MAX_UPLOAD_BODY_BYTES: usize = 36 * 1_048_576;
const MAX_HEAD_BYTES: usize = 32_768;

trait SockStream: Read + Write {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()>;
    fn shutdown_write(&self);
}

impl SockStream for std::net::TcpStream {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(dur))
    }
    fn shutdown_write(&self) {
        let _ = self.shutdown(std::net::Shutdown::Write);
    }
}

impl SockStream for &mut std::net::TcpStream {
    fn set_rt(&self, dur: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(dur))
    }
    fn shutdown_write(&self) {
        let _ = self.shutdown(std::net::Shutdown::Write);
    }
}

#[cfg(test)]
pub(crate) fn read_request(stream: &mut std::net::TcpStream) -> Option<Request> {
    let mut reader = BufReader::with_capacity(4096, stream);
    read_request_from(&mut reader, std::time::Duration::from_secs(30))
}

#[inline]
fn read_request_from<S: SockStream>(
    reader: &mut BufReader<S>,
    initial_timeout: std::time::Duration,
) -> Option<Request> {
    let deadline = std::cell::Cell::new(std::time::Instant::now() + initial_timeout);
    let arm = |s: &S| -> Option<()> {
        let remain = deadline.get().checked_duration_since(std::time::Instant::now())?;
        s.set_rt(remain).ok()?;
        Some(())
    };
    let mut line_buf = Vec::with_capacity(256);
    arm(reader.get_ref())?;
    let n_req = match head_line_into(reader, MAX_HEAD_BYTES, &mut line_buf) {
        Ok(Some(0)) => return None,
        Ok(Some(n)) => n,
        Ok(None) => return refuse_head(reader),
        Err(_) => return None,
    };
    // Once request-line bytes arrived, give the remaining head up to 30s total.
    deadline.set(std::time::Instant::now() + std::time::Duration::from_secs(30));
    let request_line = std::str::from_utf8(&line_buf)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::from_utf8_lossy(&line_buf).into_owned());
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let raw_path = parts.next()?.to_string();
    let proto = parts.next().unwrap_or("HTTP/1.1");
    let mut conn_close = false;
    let mut conn_keep_alive = false;
    let (path_part, query_part) = match raw_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_path.as_str(), ""),
    };
    let path = path_part.to_string();
    let query = query_part.to_string();
    let mut content_length: Option<usize> = None;
    let mut bad_length = false;
    let mut bad_host = false;
    let mut chunked = false;
    let mut session: Option<String> = None;
    let mut gate: Option<String> = None;
    let mut content_type = String::new();
    let mut origin: Option<String> = None;
    let mut host: Option<String> = None;
    let mut sec_fetch_site: Option<String> = None;
    let mut authorization: Option<String> = None;
    let mut user_agent: Option<String> = None;
    let mut cookie_raw: Option<String> = None;
    let mut forwarded_for: Option<String> = None;
    let mut accept_encoding: Option<String> = None;
    let mut if_none_match: Option<String> = None;
    let mut head_bytes = n_req;
    loop {
        arm(reader.get_ref())?;
        let room = MAX_HEAD_BYTES - head_bytes;
        let n = match head_line_into(reader, room, &mut line_buf) {
            Ok(Some(read)) => read,
            Ok(None) => return refuse_head(reader),
            Err(_) => return None,
        };
        if n == 0 || line_buf == b"\r\n" || line_buf == b"\n" {
            break;
        }
        head_bytes += n;
        let line_cow = String::from_utf8_lossy(&line_buf);
        let Some((raw_name, raw_val)) = line_cow.split_once(':') else {
            continue;
        };
        let name = raw_name.trim();
        let val = raw_val.trim();
        if name.eq_ignore_ascii_case("content-length") {
            match val.parse::<usize>() {
                Ok(len) => {
                    if let Some(prev) = content_length {
                        if prev != len {
                            bad_length = true;
                        }
                    } else {
                        content_length = Some(len);
                    }
                }
                Err(_) => bad_length = true,
            }
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            if val.to_ascii_lowercase().contains("chunked") {
                chunked = true;
            }
        } else if name.eq_ignore_ascii_case("content-type") {
            content_type = val.to_ascii_lowercase();
        } else if name.eq_ignore_ascii_case("origin") {
            origin = Some(val.to_ascii_lowercase());
        } else if name.eq_ignore_ascii_case("host") {
            if host.is_some() {
                bad_host = true;
            } else {
                host = Some(val.to_ascii_lowercase());
            }
        } else if name.eq_ignore_ascii_case("sec-fetch-site") {
            sec_fetch_site = Some(val.to_ascii_lowercase());
        } else if name.eq_ignore_ascii_case("accept-encoding") {
            accept_encoding = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("if-none-match") {
            if_none_match = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("authorization") {
            authorization = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("user-agent") {
            user_agent = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("x-forwarded-for") {
            forwarded_for = Some(val.to_string());
        } else if name.eq_ignore_ascii_case("cookie") {
            cookie_raw = Some(val.to_string());
            for pair in val.split(';') {
                let pair = pair.trim();
                if let Some((k, id)) = pair.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("ergon_session") {
                        let id = id.trim().to_string();
                        if !id.is_empty() {
                            session = Some(id);
                        }
                    }
                }
            }
        } else if name.eq_ignore_ascii_case("connection") {
            let low = val.to_ascii_lowercase();
            if low.contains("close") {
                conn_close = true;
            } else if low.contains("keep-alive") {
                conn_keep_alive = true;
            }
        } else if name.eq_ignore_ascii_case("x-ergon-gate") && !val.is_empty() {
            gate = Some(val.to_string());
        }
    }
    if bad_length || bad_host || chunked {
        let msg = if chunked {
            r#"{"error":"chunked transfer-encoding is not supported"}"#
        } else if bad_host {
            r#"{"error":"multiple Host headers are not permitted"}"#
        } else {
            r#"{"error":"invalid Content-Length header"}"#
        };
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{msg}",
            msg.len()
        );
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
        s.shutdown_write();
        return None;
    }
    let content_length = content_length.unwrap_or(0);
    let body_limit = if method == "POST" && path == "/api/files/upload" {
        MAX_UPLOAD_BODY_BYTES
    } else if method == "POST" && path == "/api/playbooks/import" {
        routines::MAX_PLAYBOOK_PACK_BYTES
    } else {
        MAX_BODY_BYTES
    };
    if content_length > body_limit {
        // Drain only already-arrived / fast in-flight body bytes using a short 50ms timeout
        // (and 250ms total cap) so real uploads don't get RST while forged huge Content-Length
        // headers (e.g. 99999999999999) with an open write-half return 413 promptly.
        let mut to_drain = content_length.min(4 * 1024 * 1024);
        let drain_end = std::time::Instant::now() + std::time::Duration::from_millis(250);
        let _ = reader
            .get_ref()
            .set_rt(std::time::Duration::from_millis(50));
        let mut sink = [0u8; 8192];
        while to_drain > 0 && std::time::Instant::now() < drain_end {
            let want = to_drain.min(sink.len());
            match reader.read(&mut sink[..want]) {
                Ok(0) => break,
                Ok(n) => to_drain -= n,
                Err(_) => break,
            }
        }
        let body = format!(
            "{{\"error\": \"request body too large: the limit is {body_limit} bytes\"}}"
        );
        let response = format!(
            "HTTP/1.1 413 Content Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
        s.shutdown_write();
        return None;
    }
    if body_limit == MAX_UPLOAD_BODY_BYTES && content_length > MAX_BODY_BYTES {
        deadline.set(std::time::Instant::now() + std::time::Duration::from_secs(180));
    }
    arm(reader.get_ref())?;
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).ok()?;
    }
    Some(Request {
        method,
        path,
        query,
        body,
        session,
        gate,
        content_type,
        origin,
        host,
        sec_fetch_site,
        authorization,
        user_agent,
        cookie_raw,
        forwarded_for,
        accept_encoding,
        if_none_match,
        keep_alive: if proto == "HTTP/1.0" {
            conn_keep_alive
        } else {
            !conn_close
        },
    })
}

/// Reads one request-head line of at most `room` bytes into `buf`, returning its
/// length. `Ok(None)` means the line is longer than `room`: it is never buffered
/// past the cap (E2).
fn head_line_into<R: Read>(
    reader: &mut BufReader<R>,
    room: usize,
    buf: &mut Vec<u8>,
) -> std::io::Result<Option<usize>> {
    buf.clear();
    let n = (&mut *reader).take(room as u64 + 1).read_until(b'\n', buf)?;
    if n > room {
        return Ok(None);
    }
    Ok(Some(n))
}

/// Answers an over-long request head with 431 at once, then reads and
/// discards what the client still sends until it closes (or the read
/// deadline passes) so the answer is not reset by the socket (E2).
fn refuse_head<S: SockStream>(reader: &mut BufReader<S>) -> Option<Request> {
    let body = format!(
        "{{\"error\": \"request head too large: the limit is {MAX_HEAD_BYTES} bytes\"}}"
    );
    let response = format!(
        "HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    {
        let s = reader.get_mut();
        let _ = s.write_all(response.as_bytes());
        let _ = s.flush();
    }
    let _ = reader.get_ref().set_rt(std::time::Duration::from_millis(200));
    let mut sink = [0u8; 8192];
    while matches!(reader.read(&mut sink), Ok(n) if n > 0) {}
    None
}

/// The note pushed into a thread when a delete is parked (EB1, ER1).
pub fn parked_note(path: &str) -> String {
    format!("Note: {path} has NOT been deleted yet; the deletion is waiting for your approval in Decisions.")
}

/// Whether a Host header names this machine (DNS-rebinding guard).
fn host_name(host: &str) -> String {
    match host.strip_prefix('[') {
        Some(rest) => format!("[{}]", rest.split(']').next().unwrap_or("")).to_ascii_lowercase(),
        None => host.split(':').next().unwrap_or("").to_ascii_lowercase(),
    }
}

/// Whether a Host names this machine or one the operator allowed (EH1).
#[inline]
pub(crate) fn host_allowed(host: &str, allowed: &[String]) -> bool {
    let name = host_name(host);
    matches!(name.as_str(), "localhost" | "127.0.0.1" | "[::1]")
        || allowed.iter().any(|a| a.eq_ignore_ascii_case(&name))
}

/// Refuses a request a page on another site could have sent (E1), with
/// the same observable rules as the Fleet app: a foreign Host (unless
/// the hosting gate is set), a state-changing request with a body that
/// is not JSON (a text/plain POST has no preflight), cross-site fetch
/// metadata, and an Origin whose authority is not the request's Host.
pub(crate) fn refusal(req: &Request, gated: bool, allowed: &[String]) -> Option<(u16, &'static str)> {
    if !gated && !host_allowed(req.host.as_deref().unwrap_or(""), allowed) {
        return Some((403, "requests must address this machine as localhost, 127.0.0.1 or [::1], or a host listed in ERGON_ALLOWED_HOSTS"));
    }
    if !matches!(req.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        return None;
    }
    if !req.body.is_empty() && !req.content_type.starts_with("application/json") {
        return Some((415, "POST requires Content-Type: application/json"));
    }
    if req.sec_fetch_site.as_deref() == Some("cross-site") {
        return Some((403, "cross-site requests are refused"));
    }
    if let Some(origin) = &req.origin {
        let authority = origin.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        let matches = req
            .host
            .as_deref()
            .map(|h| h.eq_ignore_ascii_case(authority))
            .unwrap_or(false);
        if !matches {
            return Some((403, "cross-origin requests are refused"));
        }
    }
    None
}

fn respond(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    respond_h(stream, status, content_type, body, &[])
}

#[inline]
fn respond_bytes(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let reason = match status {
        200 => "OK",
        201 => "Created",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let body_len = if status == 304 { 0 } else { body.len() };
    let keep_open = KEEP_ALIVE_CONN.get() && status < 400;
    if !keep_open {
        KEEP_ALIVE_CONN.set(false);
    }
    let conn_hdr = if keep_open { "keep-alive" } else { "close" };
    let mut head = String::with_capacity(384);
    if status == 304 {
        let _ = write!(&mut head, "HTTP/1.1 304 {reason}\r\nConnection: {conn_hdr}\r\n");
    } else {
        let _ = write!(
            &mut head,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {body_len}\r\nConnection: {conn_hdr}\r\n"
        );
    }
    if keep_open {
        head.push_str("Keep-Alive: timeout=2, max=10000\r\n");
    }
    for (name, value) in extra_headers {
        let _ = write!(&mut head, "{name}: {value}\r\n");
    }
    // Hardening on every response (unless the caller set its own).
    if extra_headers.is_empty() {
        head.push_str(tripwire::BASE_HEADERS_BLOCK);
        if content_type.starts_with("application/json") {
            head.push_str("Cache-Control: no-store\r\n");
        }
    } else {
        let has = |n: &str| extra_headers.iter().any(|(h, _)| h.eq_ignore_ascii_case(n));
        for (name, value) in tripwire::BASE_HEADERS {
            if !has(name) {
                let _ = write!(&mut head, "{name}: {value}\r\n");
            }
        }
        if content_type.starts_with("application/json") && !has("Cache-Control") {
            head.push_str("Cache-Control: no-store\r\n");
        }
    }
    head.push_str("\r\n");
    if status != 304 && !body.is_empty() {
        let mut bufs = [std::io::IoSlice::new(head.as_bytes()), std::io::IoSlice::new(body)];
        let mut slice_ref: &mut [std::io::IoSlice<'_>] = &mut bufs;
        while !slice_ref.is_empty() {
            let n = stream.write_vectored(slice_ref)?;
            if n == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "write_vectored 0"));
            }
            std::io::IoSlice::advance_slices(&mut slice_ref, n);
        }
        Ok(())
    } else {
        stream.write_all(head.as_bytes())
    }
}

fn respond_h(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(&str, String)],
) -> std::io::Result<()> {
    respond_bytes(stream, status, content_type, body.as_bytes(), extra_headers)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(k) == key {
            return Some(percent_decode(v));
        }
    }
    None
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u8;
    for b in input.bytes() {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'=' {
            break;
        }
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("invalid base64 data".to_string()),
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "csv" => "text/csv; charset=utf-8",
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" | "log" | "rs" | "ts" | "tsx" | "js" | "py" | "toml" | "yaml" | "yml" => {
            "text/plain; charset=utf-8"
        }
        _ => "application/octet-stream",
    }
}

fn effective_cfg(state: &AppState, runtime: &PersonRuntime) -> Config {
    let mut cfg = runtime.cfg.clone();
    let live = state.model_cfg.lock().unwrap();
    cfg.model_base_url = live.model_base_url.clone();
    cfg.model_id = live.model_id.clone();
    cfg.api_key = live.api_key.clone();
    cfg
}

const VERSION_JSON_BYTES: &[u8] = concat!(
    r#"{"name":"ergon","version":""#,
    env!("CARGO_PKG_VERSION"),
    r#""}"#,
)
.as_bytes();

#[inline]
fn respond_json_with<F: FnOnce(&mut Vec<u8>)>(
    stream: &mut std::net::TcpStream,
    status: u16,
    fill: F,
) -> std::io::Result<()> {
    JSON_SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        buf.clear();
        fill(&mut buf);
        let res = respond_bytes(stream, status, "application/json", &buf, &[]);
        if buf.capacity() > 32 * 1024 {
            *buf = Vec::with_capacity(4096);
        }
        res
    })
}

fn respond_json(stream: &mut std::net::TcpStream, status: u16, v: &Value) -> std::io::Result<()> {
    JSON_SCRATCH.with(|cell| {
        let mut buf = cell.borrow_mut();
        buf.clear();
        if serde_json::to_writer(&mut *buf, v).is_err() {
            buf.clear();
            buf.extend_from_slice(v.to_string().as_bytes());
        }
        let res = respond_bytes(stream, status, "application/json", &buf, &[]);
        if buf.capacity() > 32 * 1024 {
            *buf = Vec::with_capacity(4096);
        }
        res
    })
}

/// Markdown for the audit bundle's tripwire section.
fn render_security_markdown(sec: &Value) -> String {
    let mut md = String::from("\n\n## Security tripwires\n\n");
    md.push_str(&format!(
        "- Log chain: **{}** ({} entries, head `{}`)\n- Events shown for this person: {} (SHA-256 `{}`)\n\n",
        if sec["log_verified"].as_bool().unwrap_or(false) { "verified" } else { "BROKEN — edited outside Ergon" },
        sec["log_length"],
        sec["log_head_hash"].as_str().unwrap_or(""),
        sec["events_for_person"],
        sec["events_sha256"].as_str().unwrap_or(""),
    ));
    if let Some(events) = sec["events"].as_array() {
        if events.is_empty() {
            md.push_str("No tripwire has fired.\n");
        }
        for e in events.iter().take(100) {
            md.push_str(&format!(
                "- {} · `{}` · {} · {} — {}\n",
                records::date_string(e["ts"].as_u64().unwrap_or(0)),
                e["code"].as_str().unwrap_or(""),
                e["severity"].as_str().unwrap_or(""),
                e["title"].as_str().unwrap_or(""),
                e["action"].as_str().unwrap_or(""),
            ));
        }
    }
    md
}

/// Carry out an approved (or declined) tripwire decision and return the
/// updated decision + outcome. `None` for decisions that are not the
/// tripwire's (deletes and questions keep their own handling).
fn settle_tripwire_decision(
    state: &AppState,
    runtime: &PersonRuntime,
    person: &Person,
    decision: &Value,
    approve: bool,
) -> Option<(Value, String)> {
    let kind = decision["kind"].as_str()?;
    let id = decision["id"].as_str()?;
    let root = &state.cfg.data_dir;
    let outcome = match (kind, approve) {
        ("security_notice", _) => "Noted — Ergon keeps the traps armed.".to_string(),
        ("security", false) => "Dismissed — nothing else was changed.".to_string(),
        ("security", true) => {
            let value = decision["source"]["value"].as_str().unwrap_or("");
            match decision["source"]["kind"].as_str() {
                Some("url") | Some("host") => {
                    let host = tripwire::normalize_host(value);
                    match tripwire::update_policy(root, "block", &host, true) {
                        Ok(_) => format!("Blocked {host}: your agent will not open it again (Settings → Security can undo this)."),
                        Err(e) => format!("Could not block that site: {e}"),
                    }
                }
                Some("file") => quarantine_file(&runtime.cfg, value),
                _ => "Noted — nothing else needed doing.".to_string(),
            }
        }
        ("allow_host", true) => {
            let host = decision["path"].as_str().unwrap_or("");
            match tripwire::update_policy(root, "allow", host, true) {
                Ok(_) => format!("Allowed {host}: future tasks can open it (Settings → Security can undo this)."),
                Err(e) => format!("Could not allow it: {e}"),
            }
        }
        ("allow_host", false) => "Kept blocked.".to_string(),
        ("memory_note", true) => {
            let note = decision["note"].as_str().unwrap_or("");
            let cfg = effective_cfg(state, runtime);
            match skills::update_memory(&cfg, note) {
                Ok(_) => "Saved to your agent's memory.".to_string(),
                Err(e) => format!("Approved, but saving failed: {e}"),
            }
        }
        ("memory_note", false) => "Discarded — your agent's memory is unchanged.".to_string(),
        ("publish_skill", true) => {
            let p = &decision["payload"];
            let source = p["source"].as_str().unwrap_or("");
            let params = string_list(&p["params"]);
            match skills::check_script(&runtime.cfg, source) {
                Err(e) => format!("Approved, but the skill no longer builds: {e}"),
                Ok(()) => {
                    let mut lib = state.library.lock().unwrap();
                    match lib.publish_skill(
                        p["name"].as_str().unwrap_or("skill"),
                        p["description"].as_str().unwrap_or(""),
                        params,
                        source.to_string(),
                        &person.id,
                    ) {
                        Ok(item) => {
                            lib.save(root);
                            format!("Published \"{}\" to the company library.", item.name)
                        }
                        Err(e) => format!("Approved, but publishing failed: {e}"),
                    }
                }
            }
        }
        ("publish_skill", false) => "Not published — it stays out of the library.".to_string(),
        _ => return None,
    };
    let updated = records::set_decision_outcome(&runtime.cfg.data_dir, id, &outcome)
        .unwrap_or_else(|| decision.clone());
    Some((updated, outcome))
}

/// Move a workspace file the tripwire flagged out of the agent's reach
/// (into the person's data dir, never deleted), with a history snapshot.
fn quarantine_file(cfg: &Config, rel: &str) -> String {
    let full = match sandbox_path_resolving(&cfg.workspace_dir, rel, false) {
        Ok(p) => p,
        Err(e) => return format!("Could not quarantine {rel}: {e}"),
    };
    match full.symlink_metadata() {
        Ok(md) if md.is_file() || md.file_type().is_symlink() => {}
        _ => return format!("{rel} is no longer in the workspace — nothing to quarantine."),
    }
    let _ = file_history::record_revision(&cfg.data_dir, &cfg.workspace_dir, rel, "quarantine");
    let dir = cfg.data_dir.join("quarantine");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return format!("Could not quarantine {rel}: {e}");
    }
    let base = full
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    let dest = dir.join(format!("{}-{base}", records::now_millis()));
    match std::fs::rename(&full, &dest) {
        Ok(()) => format!("Quarantined {rel}: moved out of your agent's reach into your Ergon data folder (quarantine/), kept intact."),
        Err(e) => format!("Could not quarantine {rel}: {e}"),
    }
}

/// A run the tripwire stopped is not a server error: the thread says
/// plainly what happened and where the details are, and the response is
/// the thread (200) marked `stopped_for_safety`.
fn respond_safety_stop(
    stream: &mut std::net::TcpStream,
    runtime: &PersonRuntime,
    thread_id: &str,
    err: &str,
) -> std::io::Result<()> {
    let reason = err.trim_start_matches(tripwire::HALT_PREFIX);
    let mut store = runtime.store.lock().unwrap();
    store.push(
        thread_id,
        "step",
        &format!("Ergon stopped this task to keep your work safe: {reason}. Nothing further was run or sent. The details and one-tap follow-ups are waiting in Today."),
    );
    let mut v = store
        .thread_json(thread_id)
        .unwrap_or_else(|| json!({"thread_id": thread_id}));
    v["stopped_for_safety"] = json!(true);
    respond_json(stream, 200, &v)
}

/// The CSP for the embedded desk page, computed once from its own inline
/// scripts (see `tripwire::csp_for_html`).
fn page_csp(page: &'static str) -> String {
    static INDEX: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| tripwire::csp_for_html(page)).clone()
}

fn accepts_gzip(header: Option<&str>) -> bool {
    let Some(raw) = header else { return false };
    for part in raw.split(',') {
        let mut pieces = part.trim().split(';');
        let enc = pieces.next().unwrap_or("").trim();
        if !enc.eq_ignore_ascii_case("gzip") {
            continue;
        }
        let mut rejected = false;
        for param in pieces {
            let param = param.trim();
            if let Some(q) = param.strip_prefix("q=").or_else(|| param.strip_prefix("Q=")) {
                if q.trim().parse::<f32>().ok().is_some_and(|v| v <= 0.0) {
                    rejected = true;
                }
            }
        }
        if !rejected {
            return true;
        }
    }
    false
}

fn etag_matches(if_none_match: Option<&str>, etag: &str) -> bool {
    let Some(raw) = if_none_match else { return false };
    for item in raw.split(',') {
        let tag = item.trim();
        if tag == "*" || tag == etag || tag.strip_prefix("W/") == Some(etag) {
            return true;
        }
    }
    false
}

fn serve_chunk(stream: &mut std::net::TcpStream, req: &Request, path: &str) -> std::io::Result<()> {
    let Some(chunk) = ui::chunk(path) else {
        return respond_json(stream, 404, &json!({"error": "chunk not found"}));
    };
    let wants_gzip = chunk.pre_gzipped && accepts_gzip(req.accept_encoding.as_deref());
    let etag = if wants_gzip { chunk.etag_gzip } else { chunk.etag_raw };
    let mut headers: Vec<(&str, String)> = vec![
        ("ETag", etag.to_string()),
        ("Vary", "Accept-Encoding".to_string()),
        ("Cache-Control", "public, max-age=31536000, immutable".to_string()),
    ];
    if etag_matches(req.if_none_match.as_deref(), etag) {
        return respond_bytes(stream, 304, chunk.mime, &[], &headers);
    }
    if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        respond_bytes(stream, 200, chunk.mime, chunk.bytes, &headers)
    } else {
        let raw = ui::chunk_raw(chunk);
        respond_bytes(stream, 200, chunk.mime, raw.as_ref(), &headers)
    }
}

/// Serve a desk page (`/` or `/phone`). First visit (or an unknown,
/// forged or stale cookie) gets a fresh server-generated session pinned
/// to the current active person — so this browser keeps its own person
/// even if another client switches the global one — plus the decoy
/// `ergon_role` cookie (HT1). Supports gzip (`Accept-Encoding: gzip`),
/// strong `ETag` + `304 Not Modified`, `Vary: Accept-Encoding`, and strict CSP.
fn serve_page(
    stream: &mut std::net::TcpStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    page: &'static str,
) -> std::io::Result<()> {
    let bundle = state.tripwire.decorate_html_bundle(page);
    let wants_gzip = accepts_gzip(req.accept_encoding.as_deref());
    let etag = if wants_gzip { &bundle.etag_gzip } else { &bundle.etag_raw };
    let mut headers: Vec<(&str, String)> = vec![
        ("Content-Security-Policy", page_csp(page)),
        ("ETag", etag.clone()),
        ("Vary", "Accept-Encoding".to_string()),
        ("Cache-Control", "no-cache".to_string()),
    ];
    if !session_known(state, &req.session) {
        let sid = new_session_id();
        insert_session(state, sid.clone(), person.id.clone());
        headers.push((
            "Set-Cookie",
            format!("ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"),
        ));
        headers.push((
            "Set-Cookie",
            format!("ergon_role={}; Path=/; SameSite=Strict", tripwire::ROLE_COOKIE_VALUE),
        ));
    }
    if etag_matches(req.if_none_match.as_deref(), etag) {
        return respond_bytes(stream, 304, "text/html; charset=utf-8", &[], &headers);
    }
    if wants_gzip {
        headers.push(("Content-Encoding", "gzip".to_string()));
        respond_bytes(stream, 200, "text/html; charset=utf-8", &bundle.gzip, &headers)
    } else {
        respond_bytes(stream, 200, "text/html; charset=utf-8", bundle.html.as_bytes(), &headers)
    }
}

fn handle_connection(stream: std::net::TcpStream, state: &AppState) -> std::io::Result<()> {
    let _ = stream.set_nodelay(true);
    // A banned scanner is dropped before one byte of its request is read
    // (never this computer: loopback is never banned).
    let peer = stream.peer_addr().ok().map(|a| a.ip());
    if peer.map(|ip| state.tripwire.is_banned(ip)).unwrap_or(false) {
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return Ok(());
    }
    let mut reader = BufReader::with_capacity(4096, stream);
    const MAX_KEEP_ALIVE_REQUESTS: usize = 10_000;
    for req_idx in 0..MAX_KEEP_ALIVE_REQUESTS {
        let initial_wait = if req_idx == 0 {
            std::time::Duration::from_secs(30)
        } else {
            std::time::Duration::from_secs(2)
        };
        let req = match read_request_from(&mut reader, initial_wait) {
            Some(r) => r,
            None => break,
        };
        let client = state.tripwire.client_ip(peer, req.forwarded_for.as_deref());
        if client != peer && client.map(|ip| state.tripwire.is_banned(ip)).unwrap_or(false) {
            let _ = reader.get_ref().shutdown(std::net::Shutdown::Both);
            break;
        }
        let view = tripwire::RequestView {
            method: &req.method,
            path: &req.path,
            query: &req.query,
            host: req.host.as_deref(),
            origin: req.origin.as_deref(),
            user_agent: req.user_agent.as_deref(),
            gate: req.gate.as_deref(),
            authorization: req.authorization.as_deref(),
            cookie: req.cookie_raw.as_deref(),
            body: &req.body,
        };
        // HT1/HT3: an altered decoy cookie, or a decoy key coming back.
        if let Some(d) = state.tripwire.inspect(&view, client) {
            KEEP_ALIVE_CONN.set(false);
            return respond(reader.get_mut(), d.status, d.content_type, &d.body);
        }
        // HT2: decoy resources and breadcrumbs (before the gate, so probes
        // of a gated or LAN-exposed desk are seen too).
        if let Some(d) = state.tripwire.decoy(&req.method, &req.path, client, req.user_agent.as_deref()) {
            KEEP_ALIVE_CONN.set(false);
            return respond(reader.get_mut(), d.status, d.content_type, &d.body);
        }
        let root_cfg = &state.cfg;
        // Hosting-layer gate: when a gate token is configured, the
        // hosting layer (which authenticates people its own way) must
        // present it on every request. Ergon itself keeps no accounts
        // and no passwords — this is a deployment gate, not a login.
        if let Some(expected) = &state.gate_token {
            if req.gate.as_deref() != Some(expected.as_str()) {
                state.tripwire.note_refusal("gate", &view, client);
                KEEP_ALIVE_CONN.set(false);
                return respond_json(
                    reader.get_mut(),
                    401,
                    &json!({"error": "this Ergon is gated by its hosting layer — the gate header is missing or wrong"}),
                );
            }
        }
        if let Some((status, message)) = refusal(&req, state.gate_token.is_some(), &state.allowed_hosts) {
            // RE1: a page on another site (CSRF) or a rebound DNS name tried
            // to drive this desk. Refused as before — and now the person hears of it.
            let kind = if message.starts_with("requests must address") { "host" } else { "cross-site" };
            state.tripwire.note_refusal(kind, &view, client);
            KEEP_ALIVE_CONN.set(false);
            return respond_json(reader.get_mut(), status, &json!({"error": message}));
        }
        let is_sse = req.path == "/api/runs/stream";
        KEEP_ALIVE_CONN.set(req.keep_alive && !is_sse);
        // The person this request acts for (session-pinned when the caller
        // has a session cookie, else the global active person).
        let person = effective_person(state, &req.session);
        let runtime = runtime_for(state, &person);
        let stream = reader.get_mut();
        let res = if let Some(res) = try_route_system(stream, state, &req, &person, &runtime) {
            res
        } else if let Some(res) = try_route_files_and_audit(stream, state, &req, &person, &runtime) {
            res
        } else if let Some(res) = try_route_knowledge_and_routines(stream, state, &req, &person, &runtime) {
            res
        } else {
            let ctx = ToolCtx {
                library: state.library.clone(),
                root_data_dir: root_cfg.data_dir.clone(),
                person_id: person.id.clone(),
            };
            if let Some(res) = try_route_runs_and_chat(stream, state, &req, &person, &runtime, &ctx) {
                res
            } else {
                respond_json(stream, 404, &json!({"error": "not found"}))
            }
        };
        let continue_loop = KEEP_ALIVE_CONN.get() && !is_sse && res.is_ok();
        KEEP_ALIVE_CONN.set(false);
        res?;
        if !continue_loop {
            break;
        }
    }
    Ok(())
}

/// Routes UI pages, server status/config, updates, people, library/suite, and security.
fn try_route_system(
    stream: &mut std::net::TcpStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => serve_page(stream, state, req, person, ui::index_html()),
        ("GET", path) if path.starts_with("/chunks/") => serve_chunk(stream, req, path),
        ("GET", "/api/version") => {
            respond_bytes(stream, 200, "application/json", VERSION_JSON_BYTES, &[])
        }
        ("GET", "/api/update") => respond_json(stream, 200, &state.updater.snapshot()),
        ("POST", "/api/update/check") => respond_json(stream, 200, &state.updater.check_now()),
        ("POST", "/api/update/apply") | ("POST", "/api/update/restart") => {
            // Replacing the program is for whoever sits at this machine: not
            // through a LAN host name, not through the shared gate.
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "updating is only allowed from this machine, not through a shared address"}),
                ));
            }
            if req.path == "/api/update/apply" {
                match state.updater.apply() {
                    Ok(v) => {
                        let mut snap = state.updater.snapshot();
                        snap["applied"] = json!(v);
                        respond_json(stream, 200, &snap)
                    }
                    Err(e) => respond_json(stream, 409, &json!({"error": e})),
                }
            } else if let Err(e) = state.updater.snapshot()["staged"].as_str().map(|_| ()).ok_or("no update is waiting for a restart") {
                respond_json(stream, 409, &json!({"error": e}))
            } else if RUNS_IN_FLIGHT.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                respond_json(stream, 409, &json!({"error": "a task is still running; restart when it has finished"}))
            } else {
                let _ = respond_json(stream, 200, &json!({"restarting": true}));
                let up = state.updater.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    if let Err(e) = up.restart() {
                        eprintln!("ergon: restart failed: {e}");
                    }
                });
                Ok(())
            }
        }
        ("GET", "/api/status") => {
            let mut recovered: Vec<String> = Vec::new();
            {
                let people = state.people.read().unwrap();
                if !people.recovered.is_empty() {
                    recovered.extend(people.recovered.iter().cloned());
                }
            }
            {
                let lib = state.library.lock().unwrap();
                if let Some(s) = &lib.recovered {
                    recovered.push(s.clone());
                }
            }
            {
                let runtimes = state.runtimes.read().unwrap();
                for rt in runtimes.values() {
                    let st = rt.store.lock().unwrap();
                    if let Some(s) = &st.recovered {
                        recovered.push(s.clone());
                    }
                }
            }
            if recovered.len() > 1 {
                recovered.sort();
                recovered.dedup();
            }
            let (model_configured, model_id, model_base_url, auto_detected) = {
                let live = state.model_cfg.lock().unwrap();
                (
                    crate::model_configured_for(&live.model_base_url, live.api_key.is_some()),
                    live.model_id.clone(),
                    live.model_base_url.clone(),
                    live.auto_detected,
                )
            };
            let is_loopback = is_loopback_url(&model_base_url);
            respond_json_with(stream, 200, |buf| {
                buf.extend_from_slice(br#"{"version":"#);
                search::write_json_str(buf, env!("CARGO_PKG_VERSION"));
                buf.extend_from_slice(br#","model_configured":"#);
                search::write_json_bool(buf, model_configured);
                buf.extend_from_slice(br#","model":"#);
                search::write_json_str(buf, &model_id);
                buf.extend_from_slice(br#","model_base_url":"#);
                search::write_json_str(buf, &model_base_url);
                buf.extend_from_slice(br#","is_loopback_model":"#);
                search::write_json_bool(buf, is_loopback);
                buf.extend_from_slice(br#","auto_detected":"#);
                search::write_json_bool(buf, auto_detected);
                buf.extend_from_slice(
                    br#","capabilities":{"sse":true,"skills":true,"routines":true,"playbooks":true,"search":true,"receipts_export":true,"cancel":true,"scheduler":"#,
                );
                search::write_json_bool(buf, state.scheduler_enabled);
                buf.extend_from_slice(
                    br#","file_history":true,"audit_bundle":true,"tripwire":true},"scheduler_enabled":"#,
                );
                search::write_json_bool(buf, state.scheduler_enabled);
                buf.extend_from_slice(br#","person":{"id":"#);
                search::write_json_str(buf, &person.id);
                buf.extend_from_slice(br#","name":"#);
                search::write_json_str(buf, &person.name);
                buf.extend_from_slice(br#","role":"#);
                search::write_json_str(buf, &person.role);
                buf.extend_from_slice(br#"},"store_recovered":["#);
                for (i, item) in recovered.iter().enumerate() {
                    if i > 0 {
                        buf.push(b',');
                    }
                    search::write_json_str(buf, item);
                }
                buf.extend_from_slice(b"]}");
            })
        }
        ("GET", "/api/config") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "model settings are only available from this machine, not through a shared address"}),
                ));
            }
            let eff = effective_cfg(state, runtime);
            let live_meta = state.model_cfg.lock().unwrap().clone();
            let gateways: Vec<Value> = detect_local_gateways()
                .into_iter()
                .map(|g| g.to_json())
                .collect();
            respond_json(
                stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": live_meta.auto_detected,
                    "detected_gateways": gateways,
                }),
            )
        }
        ("POST", "/api/config") => {
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "changing model settings is only allowed from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let base = parsed["model_base_url"]
                .as_str()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/')
                .to_string();
            let model_id = parsed["model_id"].as_str().unwrap_or("").trim().to_string();
            if !(base.starts_with("http://") || base.starts_with("https://")) || parse_url_host_port(&base).is_none() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "model_base_url must be a valid http:// or https:// URL"}),
                ));
            }
            if model_id.is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "model_id is required"}),
                ));
            }
            let clear_key = parsed["clear_api_key"].as_bool().unwrap_or(false);
            let next_key = if clear_key {
                None
            } else if let Some(raw_k) = parsed["api_key"].as_str() {
                let k = raw_k.trim().to_string();
                if k.is_empty() {
                    state.model_cfg.lock().unwrap().api_key.clone()
                } else {
                    Some(k)
                }
            } else {
                state.model_cfg.lock().unwrap().api_key.clone()
            };
            let saved = SavedModelConfig {
                model_base_url: base.clone(),
                model_id: model_id.clone(),
                api_key: next_key.clone(),
            };
            if let Err(e) = save_model_config(&root_cfg.data_dir, &saved) {
                return Some(respond_json(stream, 500, &json!({"error": e})));
            }
            {
                let mut live = state.model_cfg.lock().unwrap();
                live.model_base_url = base;
                live.model_id = model_id;
                live.api_key = next_key;
                live.auto_detected = false;
            }
            let eff = effective_cfg(state, runtime);
            respond_json(
                stream,
                200,
                &json!({
                    "model_base_url": eff.model_base_url,
                    "model_id": eff.model_id,
                    "has_api_key": eff.api_key.is_some(),
                    "model_configured": eff.model_configured(),
                    "is_loopback": is_loopback_url(&eff.model_base_url),
                    "auto_detected": false,
                }),
            )
        }
        ("GET", "/api/people") => {
            let people = state.people.read().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(person.id);
            respond_json(stream, 200, &v)
        }
        ("POST", "/api/people") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("").trim().to_string();
            if name.is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "name is required"}),
                ));
            }
            let role = parsed["role"].as_str().unwrap_or("").trim().to_string();
            let person = {
                let mut people = state.people.write().unwrap();
                let p = people.add(&name, &role);
                people.save(root_cfg);
                p
            };
            // Create their dirs + seeded memory now, so the person is
            // real on disk from this moment.
            let _ = runtime_for(state, &person);
            respond_json(stream, 201, &person.to_json())
        }
        ("POST", "/api/active-person") | ("POST", "/api/people/switch") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["id"]
                .as_str()
                .or_else(|| parsed["person_id"].as_str())
                .unwrap_or("")
                .to_string();
            {
                let people = state.people.read().unwrap();
                if !people.people.iter().any(|p| p.id == id) {
                    return Some(respond_json(
                        stream,
                        404,
                        &json!({"error": "person not found"}),
                    ));
                }
            }
            // Who does this switch move? A browser with a known session
            // moves ONLY its own session — never the global pointer
            // every other client inherits. A cookie-less API client
            // moves the global default (and is issued a session pinned
            // to it). A forged/unknown cookie is treated as no cookie:
            // the server issues its own id, never the client's.
            let mut set_cookie: Option<String> = None;
            if session_known(state, &req.session) {
                let sid = req.session.clone().unwrap();
                insert_session(state, sid, id.clone());
            } else {
                {
                    let mut people = state.people.write().unwrap();
                    people.active = id.clone();
                    people.save(root_cfg);
                }
                let sid = new_session_id();
                insert_session(state, sid.clone(), id.clone());
                set_cookie = Some(format!(
                    "ergon_session={sid}; Path=/; HttpOnly; SameSite=Lax"
                ));
            }
            let people = state.people.read().unwrap();
            let mut v = people.to_json();
            v["active"] = json!(id);
            match set_cookie {
                Some(cookie) => respond_h(
                    stream,
                    200,
                    "application/json",
                    &v.to_string(),
                    &[("Set-Cookie", cookie)],
                ),
                None => respond_json(stream, 200, &v),
            }
        }
        ("GET", "/api/memory") => {
            let content = runtime.cfg.read_memory();
            let health = skills::audit_memory_health(&runtime.cfg);
            respond_json(
                stream,
                200,
                &json!({"person_id": person.id, "content": content, "health": health}),
            )
        }
        ("GET", "/api/memory/health") => {
            let mut health = skills::audit_memory_health(&runtime.cfg);
            if let Some(obj) = health.as_object_mut() {
                obj.insert("person_id".to_string(), json!(person.id));
            }
            respond_json(stream, 200, &health)
        }
        ("POST", "/api/memory/heal") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let mode = parsed["mode"].as_str().unwrap_or("all");
            match skills::heal_memory(&runtime.cfg, mode) {
                Ok(mut out) => {
                    state.search_cache.invalidate(&person.id);
                    if let Some(obj) = out.as_object_mut() {
                        obj.insert("person_id".to_string(), json!(person.id));
                    }
                    respond_json(stream, 200, &out)
                }
                Err(e) => respond_json(stream, 400, &json!({"error": e})),
            }
        }
        ("GET", "/phone") => serve_page(stream, state, req, person, ui::index_html()),
        ("GET", "/api/library") => {
            let lib = state.library.lock().unwrap();
            respond_json(
                stream,
                200,
                &json!({
                    "items": lib.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
                }),
            )
        }
        ("GET", "/api/suite") => {
            let lib = state.library.lock().unwrap();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let installed: Vec<Value> = suite
                .resolved(&lib)
                .iter()
                .map(|i| i.to_json())
                .collect();
            respond_json(
                stream,
                200,
                &json!({"person_id": person.id, "installed": installed}),
            )
        }
        ("POST", "/api/suite/install") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            let lib = state.library.lock().unwrap();
            match lib.get(id) {
                None => respond_json(
                    stream,
                    404,
                    &json!({"error": "library item not found"}),
                ),
                Some(item) if item.builtin => respond_json(
                    stream,
                    400,
                    &json!({"error": "that item is built into every agent already"}),
                ),
                Some(item) => {
                    library::with_suite_lock(|| {
                        let mut suite = library::load_suite(&runtime.cfg.data_dir);
                        suite.install(&item.id);
                        library::save_suite(&runtime.cfg.data_dir, &suite);
                    });
                    respond_json(stream, 200, &json!({"installed": item.id}))
                }
            }
        }
        ("POST", "/api/suite/remove") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let id = parsed["item_id"].as_str().unwrap_or("");
            library::with_suite_lock(|| {
                let mut suite = library::load_suite(&runtime.cfg.data_dir);
                suite.installed.retain(|i| i != id);
                library::save_suite(&runtime.cfg.data_dir, &suite);
            });
            respond_json(stream, 200, &json!({"removed": id}))
        }
        ("GET", "/api/security") => {
            let (ok, len, head) = tripwire::check_log_integrity(&root_cfg.data_dir);
            let events = tripwire::events_for(&root_cfg.data_dir, &person.id, 60);
            let now = records::now_millis();
            let day: Vec<&Value> = events
                .iter()
                .filter(|e| now.saturating_sub(e["ts"].as_u64().unwrap_or(0)) < 86_400_000)
                .collect();
            let policy = tripwire::load_policy(&root_cfg.data_dir);
            let local = state.gate_token.is_none() && host_allowed(req.host.as_deref().unwrap_or(""), &[]);
            respond_json(
                stream,
                200,
                &json!({
                    "armed": tripwire::armed_traps(),
                    "events": events,
                    "last_24h": {
                        "events": day.len(),
                        "critical": day.iter().filter(|e| e["severity"] == "critical").count(),
                        "blocked": day.iter().filter(|e| matches!(e["action"].as_str(), Some("blocked" | "stopped run" | "banned" | "refused" | "held"))).count(),
                    },
                    "log": {"verified": ok, "length": len, "head_hash": head},
                    "bans": state.tripwire.bans_json(),
                    "policy": {
                        "allow_hosts": policy.allow_hosts,
                        "block_hosts": policy.block_hosts,
                        "allow_private_env": std::env::var("ERGON_FETCH_ALLOW_PRIVATE").map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true")).unwrap_or(false),
                    },
                    "can_edit": local,
                }),
            )
        }
        ("POST", "/api/security/policy") => {
            // Same rule as model settings: only from this machine.
            if state.gate_token.is_some() || !host_allowed(req.host.as_deref().unwrap_or(""), &[]) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "security settings can only be changed from this machine, not through a shared address"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            if let Some(ip) = parsed["unban"].as_str() {
                return Some(match ip.trim().parse::<std::net::IpAddr>() {
                    Ok(ip) => respond_json(stream, 200, &json!({"unbanned": state.tripwire.unban(ip), "bans": state.tripwire.bans_json()})),
                    Err(_) => respond_json(stream, 400, &json!({"error": "unban needs an IP address"})),
                });
            }
            let list = parsed["list"].as_str().unwrap_or("");
            let host = parsed["host"].as_str().unwrap_or("");
            let add = parsed["add"].as_bool().unwrap_or(true);
            match tripwire::update_policy(&root_cfg.data_dir, list, host, add) {
                Ok(p) => respond_json(stream, 200, &json!({"policy": {"allow_hosts": p.allow_hosts, "block_hosts": p.block_hosts}})),
                Err(e) => respond_json(stream, 400, &json!({"error": e})),
            }
        }
        _ => return None,
    })
}

struct PackCheckedFile {
    path: String,
    size_bytes: usize,
    non_empty: bool,
    count: usize,
    unit: &'static str,
    digest: String,
    body: String,
}

struct PackPreflight {
    ready_count: usize,
    empty_count: usize,
    missing_paths: Vec<String>,
    total_bytes: usize,
    items: Vec<PackCheckedFile>,
    citations: Vec<CitationCheck>,
    citations_verified: usize,
    citations_stale: usize,
    citations_broken: usize,
}

impl PackPreflight {
    fn freshness_pct(&self) -> u8 {
        let total = self.citations.len();
        if total == 0 {
            100
        } else {
            ((self.citations_verified * 100) / total) as u8
        }
    }

    fn citation_gate(&self) -> &'static str {
        if self.citations_broken == 0 && self.citations_stale == 0 {
            "100% Verified"
        } else if self.citations_stale > 0 {
            "Stale Line Drift"
        } else {
            "Broken Citations"
        }
    }

    fn ok(&self) -> bool {
        self.ready_count > 0
            && self.empty_count == 0
            && self.missing_paths.is_empty()
            && self.citations_stale == 0
            && self.citations_broken == 0
    }

    fn to_json(&self) -> Value {
        let mut files_json: Vec<Value> = self
            .items
            .iter()
            .map(|f| {
                json!({
                    "path": f.path,
                    "size_bytes": f.size_bytes,
                    "non_empty": f.non_empty,
                    "count": f.count,
                    "unit": f.unit,
                    "sha256": f.digest,
                    "status": if f.non_empty { "ready" } else { "empty" },
                })
            })
            .collect();
        for miss in &self.missing_paths {
            files_json.push(json!({
                "path": miss,
                "size_bytes": 0,
                "non_empty": false,
                "count": 0,
                "unit": "bytes",
                "sha256": "",
                "status": "missing",
            }));
        }
        json!({
            "ok": self.ok(),
            "ready_count": self.ready_count,
            "empty_count": self.empty_count,
            "missing_count": self.missing_paths.len(),
            "total_bytes": self.total_bytes,
            "citations_checked": self.citations.len(),
            "citations_verified": self.citations_verified,
            "citations_stale": self.citations_stale,
            "citations_broken": self.citations_broken,
            "citations_freshness_pct": self.freshness_pct(),
            "citation_gate": self.citation_gate(),
            "citations": self.citations.iter().map(CitationCheck::to_json).collect::<Vec<_>>(),
            "files": files_json,
        })
    }
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Find the pre-overwrite SHA-256 recorded just after `receipt_ts` (oldest snapshot with `ts > receipt_ts`).
fn history_baseline_sha_for_receipt(
    data_dir: &std::path::Path,
    workspace_dir: &std::path::Path,
    rel_path: &str,
    receipt_ts: u64,
) -> Option<String> {
    if receipt_ts == 0 {
        return None;
    }
    let revs = file_history::list_revisions(data_dir, workspace_dir, rel_path).ok()?;
    revs.into_iter()
        .filter(|r| r.timestamp_ms > receipt_ts)
        .min_by_key(|r| r.timestamp_ms)
        .map(|r| r.sha256)
}

fn status_severity(status: &str) -> u8 {
    match status {
        "verified" => 0,
        "stale" => 1,
        _ => 2,
    }
}

fn adjust_citation_counts(status: &str, delta: isize, verified: &mut usize, stale: &mut usize, broken: &mut usize) {
    let target = match status {
        "verified" => verified,
        "stale" => stale,
        _ => broken,
    };
    if delta >= 0 {
        *target = target.saturating_add(delta as usize);
    } else {
        *target = target.saturating_sub((-delta) as usize);
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_citations_into(
    data_dir: Option<&std::path::Path>,
    workspace_dir: &std::path::Path,
    text: &str,
    receipt_ts: u64,
    expected_shas: Option<&std::collections::HashMap<String, String>>,
    citations: &mut Vec<CitationCheck>,
    seen: &mut std::collections::HashMap<(String, usize, Option<usize>), usize>,
    verified: &mut usize,
    stale: &mut usize,
    broken: &mut usize,
) -> (usize, usize, usize, usize) {
    if text.is_empty() {
        return (0, 0, 0, 0);
    }
    let mut rep = verify_text_citations_against(workspace_dir, text, expected_shas);
    let mut run_total = 0usize;
    let mut run_verified = 0usize;
    let mut run_stale = 0usize;
    let mut run_broken = 0usize;
    for mut c in rep.citations.drain(..) {
        if c.status == "verified" && receipt_ts > 0 {
            if let Some(ddir) = data_dir {
                if let Some(base_sha) = history_baseline_sha_for_receipt(ddir, workspace_dir, &c.path, receipt_ts) {
                    if !base_sha.is_empty() && !base_sha.eq_ignore_ascii_case(&c.file_sha256) {
                        c.status = "stale";
                        c.baseline_sha256 = Some(base_sha);
                    }
                }
            }
        }
        run_total += 1;
        match c.status {
            "verified" => run_verified += 1,
            "stale" => run_stale += 1,
            _ => run_broken += 1,
        }
        let key = (c.path.clone(), c.line, c.end_line);
        if let Some(&existing_idx) = seen.get(&key) {
            let old_status = citations[existing_idx].status;
            if status_severity(c.status) > status_severity(old_status) {
                adjust_citation_counts(old_status, -1, verified, stale, broken);
                adjust_citation_counts(c.status, 1, verified, stale, broken);
                citations[existing_idx] = c;
            }
            continue;
        }
        if citations.len() >= 128 {
            continue;
        }
        let idx = citations.len();
        seen.insert(key, idx);
        adjust_citation_counts(c.status, 1, verified, stale, broken);
        citations.push(c);
    }
    (run_total, run_verified, run_stale, run_broken)
}

#[derive(Clone)]
struct SummaryRunGate<'a> {
    receipt: &'a receipts::ChainedReceipt,
    checked: usize,
    verified: usize,
    stale: usize,
    broken: usize,
}

impl SummaryRunGate<'_> {
    fn gate_label(&self) -> &'static str {
        if self.checked == 0 {
            "Deliverables ready"
        } else if self.broken == 0 && self.stale == 0 {
            "Verified"
        } else if self.stale > 0 {
            "Stale Line Drift"
        } else {
            "Broken Citations"
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "id": self.receipt.id,
            "task": self.receipt.task,
            "ts": self.receipt.ts,
            "sha256": self.receipt.sha256,
            "files": self.receipt.files,
            "citations_checked": self.checked,
            "citations_verified": self.verified,
            "citations_stale": self.stale,
            "citations_broken": self.broken,
            "citation_gate": self.gate_label(),
        })
    }
}

/// Validates workspace deliverable paths, computes non-empty/row/line counts, citations, and SHA-256 digests.
fn inspect_workspace_pack(
    data_dir: &std::path::Path,
    workspace_dir: &std::path::Path,
    requested: &[String],
    explicit: bool,
    extra_receipt_texts: &[(&str, u64)],
    expected_shas: Option<&std::collections::HashMap<String, String>>,
) -> Result<PackPreflight, (u16, String)> {
    const MAX_PACK_BYTES: usize = 2 * 1024 * 1024;
    const MAX_FILE_BYTES: usize = 512 * 1024;
    let mut items: Vec<PackCheckedFile> = Vec::new();
    let mut missing_paths: Vec<String> = Vec::new();
    let mut ready_count = 0usize;
    let mut empty_count = 0usize;
    let mut total_bytes = 0usize;
    let mut rendered_bytes = 0usize;
    let mut citations: Vec<CitationCheck> = Vec::new();
    let mut seen_citations: std::collections::HashMap<(String, usize, Option<usize>), usize> =
        std::collections::HashMap::new();
    let mut citations_verified = 0usize;
    let mut citations_stale = 0usize;
    let mut citations_broken = 0usize;
    for &(rtext, rts) in extra_receipt_texts {
        collect_citations_into(
            Some(data_dir),
            workspace_dir,
            rtext,
            rts,
            expected_shas,
            &mut citations,
            &mut seen_citations,
            &mut citations_verified,
            &mut citations_stale,
            &mut citations_broken,
        );
    }
    for rel in requested {
        let full = match sandbox_path(workspace_dir, rel) {
            Ok(p) => p,
            Err(e) => {
                if explicit {
                    return Err((403, e));
                }
                continue;
            }
        };
        if full.symlink_metadata().is_ok_and(|md| md.file_type().is_symlink()) {
            if explicit {
                return Err((403, "cannot pack symlinked path".to_string()));
            }
            continue;
        }
        let Ok(bytes) = std::fs::read(&full) else {
            if explicit {
                missing_paths.push(rel.clone());
            }
            continue;
        };
        let digest = receipts::sha256_hex(&bytes);
        let ext = full
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let non_empty = !bytes.iter().all(|b| b.is_ascii_whitespace());
        if non_empty {
            ready_count += 1;
        } else {
            empty_count += 1;
        }
        total_bytes += bytes.len();
        let (count, unit, body) = if bytes.len() > MAX_FILE_BYTES {
            (
                bytes.len(),
                "bytes",
                format!(
                    "_Skipped inline preview ({} bytes exceeds 512 KB cap; SHA-256 `{}`)._",
                    bytes.len(),
                    digest
                ),
            )
        } else if let Ok(text) = std::str::from_utf8(&bytes) {
            let line_count = text.lines().filter(|l| !l.trim().is_empty()).count();
            let (metric, unit_name) = if ext == "csv" || ext == "tsv" {
                (line_count.saturating_sub(1), "rows")
            } else {
                (line_count, "lines")
            };
            collect_citations_into(
                Some(data_dir),
                workspace_dir,
                text,
                0,
                expected_shas,
                &mut citations,
                &mut seen_citations,
                &mut citations_verified,
                &mut citations_stale,
                &mut citations_broken,
            );
            let rendered = if ext == "md" || ext == "markdown" || ext == "txt" {
                text.trim_end().to_string()
            } else {
                let fence = match ext.as_str() {
                    "csv" | "tsv" => "csv",
                    "json" => "json",
                    _ => "text",
                };
                format!("```{fence}\n{}\n```", text.trim_end())
            };
            (metric, unit_name, rendered)
        } else {
            (
                bytes.len(),
                "bytes",
                format!(
                    "_Binary deliverable ({} bytes, `.{ext}`; download original file from workspace; SHA-256 `{}`)._",
                    bytes.len(),
                    digest
                ),
            )
        };
        if rendered_bytes + body.len() > MAX_PACK_BYTES {
            break;
        }
        rendered_bytes += body.len();
        items.push(PackCheckedFile {
            path: rel.clone(),
            size_bytes: bytes.len(),
            non_empty,
            count,
            unit,
            digest,
            body,
        });
    }
    if explicit && items.is_empty() {
        return Err((404, "no matching workspace files found".to_string()));
    }
    Ok(PackPreflight {
        ready_count,
        empty_count,
        missing_paths,
        total_bytes,
        items,
        citations,
        citations_verified,
        citations_stale,
        citations_broken,
    })
}

fn format_workspace_pack_markdown(
    preflight: &PackPreflight,
    person_name: &str,
    title: &str,
    summary_runs: &[SummaryRunGate<'_>],
) -> String {
    let ok = preflight.ok();
    let ready = preflight.ready_count;
    let status_word = if ok { "PASS" } else { "WARN" };
    let total_cits = preflight.citations.len();
    let ratio_label = if total_cits == 0 {
        "no inline citations".to_string()
    } else {
        format!(
            "{}/{} verified ({}%)",
            preflight.citations_verified,
            total_cits,
            preflight.freshness_pct()
        )
    };
    let mut md = format!(
        "# {title}\n\n- Prepared for: **{person_name}**\n- Deliverables included: {}\n- Completed runs summarized: {}\n- Pre-flight check: **{status_word}** ({ready} non-empty, {} empty, {} missing, {} total bytes; citations: {} — {ratio_label})\n\n",
        preflight.items.len(),
        summary_runs.len(),
        preflight.empty_count,
        preflight.missing_paths.len(),
        preflight.total_bytes,
        preflight.citation_gate(),
    );
    if !summary_runs.is_empty() {
        md.push_str("## Verified Work Summary\n\n");
        if preflight.citations_stale > 0 || preflight.citations_broken > 0 {
            md.push_str(&format!(
                "- **Batch Citation Gate:** WARN — {}/{} verified, **{} stale**, {} broken across {} runs\n\n",
                preflight.citations_verified,
                total_cits,
                preflight.citations_stale,
                preflight.citations_broken,
                summary_runs.len(),
            ));
        } else if total_cits > 0 {
            md.push_str(&format!(
                "- **Batch Citation Gate:** 100% Verified ({}/{} citations across {} runs)\n\n",
                preflight.citations_verified,
                total_cits,
                summary_runs.len(),
            ));
        }
        for sg in summary_runs {
            let r = sg.receipt;
            let short_sha = &r.sha256[..12.min(r.sha256.len())];
            let snippet = truncate(r.summary.trim(), 180);
            let files_part = if r.files.is_empty() {
                String::new()
            } else {
                format!(" · Files: {}", r.files.join(", "))
            };
            let cit_part = if sg.checked == 0 {
                String::new()
            } else if sg.stale > 0 || sg.broken > 0 {
                format!(
                    " · **[CITATION ALERT: {}/{} verified, {} stale, {} broken]**",
                    sg.verified, sg.checked, sg.stale, sg.broken
                )
            } else {
                format!(" · [citations: {}/{} verified]", sg.verified, sg.checked)
            };
            md.push_str(&format!(
                "- **{}** (`{}` · `sha256:{}`{files_part}{cit_part}): {}\n",
                r.task, r.id, short_sha, snippet
            ));
        }
        md.push('\n');
    }
    md.push_str("## Included Deliverables\n\n");
    for f in &preflight.items {
        md.push_str(&format!(
            "- `{}` ({} {}, `sha256:{}`)\n",
            f.path, f.count, f.unit, f.digest
        ));
    }
    if !preflight.citations.is_empty() {
        md.push_str("\n## Citation Verification\n\n");
        for c in &preflight.citations {
            md.push_str(&format!(
                "- `{}` -> **{}** ({} lines in file, live `sha256:{}`)\n",
                c.raw,
                c.status,
                c.total_lines,
                &c.file_sha256[..12.min(c.file_sha256.len())]
            ));
        }
    }
    for f in &preflight.items {
        md.push_str(&format!(
            "\n---\n\n## `{}`\n\n_SHA-256: `{}` · {} {} · {} bytes_\n\n{}\n",
            f.path, f.digest, f.count, f.unit, f.size_bytes, f.body
        ));
    }
    md
}

/// Builds a Markdown Client Deliverable Pack across workspace files with pre-flight verification & SHA-256 digests.
fn build_workspace_pack(
    workspace_dir: &std::path::Path,
    person_name: &str,
    title: &str,
    requested: &[String],
    explicit: bool,
) -> Result<(String, String, bool, usize), (u16, String)> {
    let preflight = inspect_workspace_pack(workspace_dir, workspace_dir, requested, explicit, &[], None)?;
    let ok = preflight.ok();
    let ready = preflight.ready_count;
    let md = format_workspace_pack_markdown(&preflight, person_name, title, &[]);
    let pack_sha = receipts::sha256_hex(md.as_bytes());
    Ok((md, pack_sha, ok, ready))
}

/// Builds a standalone, zero-external-request, printable Client Handoff (`.html`) with SHA-256 Manifest.
fn build_workspace_handoff_html(
    preflight: &PackPreflight,
    person_name: &str,
    title: &str,
    delta_md: Option<&str>,
    summary_runs: &[SummaryRunGate<'_>],
) -> (String, String) {
    let gate = preflight.citation_gate();
    let gate_class = if preflight.ok() { "ok" } else { "warn" };
    let total_cits = preflight.citations.len();
    let cit_ratio = if total_cits == 0 {
        "0 inline citations".to_string()
    } else {
        format!(
            "{}/{} verified ({}%)",
            preflight.citations_verified,
            total_cits,
            preflight.freshness_pct()
        )
    };
    let mut summary_section = String::new();
    if !summary_runs.is_empty() {
        summary_section.push_str("<section><h2>Verified Work Summary</h2><table><thead><tr><th>Task</th><th>Summary</th><th>Deliverables</th><th>Citations</th><th>Receipt SHA-256</th></tr></thead><tbody>");
        for sg in summary_runs {
            let r = sg.receipt;
            let short_sha = &r.sha256[..12.min(r.sha256.len())];
            let cit_pill_cls = if sg.stale > 0 || sg.broken > 0 { "warn" } else { "ok" };
            let cit_pill_txt = if sg.checked == 0 {
                "ready".to_string()
            } else if sg.stale > 0 || sg.broken > 0 {
                format!("{}/{} ({} stale)", sg.verified, sg.checked, sg.stale + sg.broken)
            } else {
                format!("{}/{} verified", sg.verified, sg.checked)
            };
            summary_section.push_str(&format!(
                "<tr><td><strong>{}</strong></td><td>{}</td><td><code>{}</code></td><td><span class=\"pill {}\">{}</span></td><td><code>{}</code></td></tr>",
                html_escape(&r.task),
                html_escape(&truncate(r.summary.trim(), 180)),
                html_escape(&r.files.join(", ")),
                cit_pill_cls,
                html_escape(&cit_pill_txt),
                html_escape(short_sha)
            ));
        }
        summary_section.push_str("</tbody></table></section>");
    }
    let mut files_rows = String::new();
    for f in &preflight.items {
        files_rows.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{} {}</td><td>{} B</td><td><code>{}</code></td></tr>\n",
            html_escape(&f.path),
            f.count,
            f.unit,
            f.size_bytes,
            html_escape(&f.digest)
        ));
    }
    let mut cit_section = String::new();
    if !preflight.citations.is_empty() {
        cit_section.push_str("<section><h2>Live Citation Verification</h2><table><thead><tr><th>Citation</th><th>Gate Status</th><th>Live File SHA-256</th><th>Excerpt</th></tr></thead><tbody>");
        for c in &preflight.citations {
            let short_sha = if c.file_sha256.is_empty() {
                "—"
            } else {
                &c.file_sha256[..12.min(c.file_sha256.len())]
            };
            cit_section.push_str(&format!(
                "<tr><td><code>{}</code></td><td><span class=\"pill {}\">{}</span></td><td><code>{}</code></td><td>{}</td></tr>",
                html_escape(&c.raw),
                if c.status == "verified" { "ok" } else { "warn" },
                html_escape(c.status),
                html_escape(short_sha),
                html_escape(&c.line_excerpt)
            ));
        }
        cit_section.push_str("</tbody></table></section>");
    }
    let delta_section = delta_md
        .filter(|d| !d.trim().is_empty())
        .map(|d| format!("<section><h2>Verified Delta</h2><pre>{}</pre></section>", html_escape(d)))
        .unwrap_or_default();
    let mut previews = String::new();
    for f in &preflight.items {
        previews.push_str(&format!(
            "<section class=\"doc\"><h3><code>{}</code></h3><p class=\"meta\">SHA-256: <code>{}</code> &middot; {} {} &middot; {} bytes</p><pre>{}</pre></section>\n",
            html_escape(&f.path),
            html_escape(&f.digest),
            f.count,
            f.unit,
            f.size_bytes,
            html_escape(&f.body)
        ));
    }
    let manifest_seed = preflight
        .items
        .iter()
        .map(|f| format!("{}:{}", f.path, f.digest))
        .collect::<Vec<_>>()
        .join("|");
    let manifest_sha = receipts::sha256_hex(manifest_seed.as_bytes());
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>:root{{color-scheme:light}}body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:920px;margin:32px auto;padding:0 20px;color:#111827;background:#fff;line-height:1.55}}header{{border-bottom:1px solid #e5e7eb;padding-bottom:16px;margin-bottom:24px}}h1{{margin:0 0 8px;font-size:24px}}h2{{font-size:17px;margin:28px 0 10px}}h3{{font-size:14px;margin:0 0 6px}}.meta{{color:#4b5563;font-size:13px;margin:4px 0 12px}}.pill{{display:inline-block;padding:2px 9px;border-radius:999px;font-size:12px;font-weight:600}}.pill.ok{{background:#dcfce7;color:#166534}}.pill.warn{{background:#fef3c7;color:#92400e}}table{{width:100%;border-collapse:collapse;font-size:13px}}th,td{{text-align:left;padding:8px 10px;border-bottom:1px solid #e5e7eb;vertical-align:top}}th{{background:#f9fafb;font-weight:600}}code{{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;word-break:break-all}}pre{{background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;padding:12px;overflow:auto;font-size:12px;white-space:pre-wrap;word-break:break-word}}.doc{{margin-top:20px;break-inside:avoid}}@media print{{body{{margin:0}}header,section{{break-inside:avoid}}}}</style></head><body><header><h1>{}</h1><p class=\"meta\">Prepared for <strong>{}</strong> &middot; <span class=\"pill {}\">{}</span> &middot; Citations: {} &middot; Manifest SHA-256: <code>{}</code></p></header>{}<section><h2>SHA-256 Deliverable Manifest</h2><table><thead><tr><th>File</th><th>Contents</th><th>Size</th><th>SHA-256</th></tr></thead><tbody>{}</tbody></table></section>{}{}<section><h2>Deliverable Previews</h2>{}</section></body></html>",
        html_escape(title),
        html_escape(title),
        html_escape(person_name),
        gate_class,
        html_escape(gate),
        html_escape(&cit_ratio),
        html_escape(&manifest_sha),
        summary_section,
        files_rows,
        cit_section,
        delta_section,
        previews
    );
    let pack_sha = receipts::sha256_hex(html.as_bytes());
    (html, pack_sha)
}

fn build_receipt_compare(a: &receipts::ChainedReceipt, b: &receipts::ChainedReceipt) -> (Value, String) {
    let diff = file_history::diff_texts(
        &format!("{} ({})", a.id, a.task),
        &format!("{} ({})", b.id, b.task),
        &a.summary,
        &b.summary,
    );
    let files_added: Vec<String> = b
        .files
        .iter()
        .filter(|f| !a.files.contains(f))
        .cloned()
        .collect();
    let files_removed: Vec<String> = a
        .files
        .iter()
        .filter(|f| !b.files.contains(f))
        .cloned()
        .collect();
    let files_shared: Vec<String> = b
        .files
        .iter()
        .filter(|f| a.files.contains(f))
        .cloned()
        .collect();
    let line_delta = diff.added as isize - diff.removed as isize;
    let file_delta = b.files.len() as isize - a.files.len() as isize;
    let mut delta_md = format!(
        "## Verified Run Delta (`{}` -> `{}`)\n\n- **Baseline:** `{}` — {} (`sha256:{}`)\n- **Target:** `{}` — {} (`sha256:{}`)\n- **Summary Diff:** +{} / -{} lines ({}% similarity, net {:+} lines)\n- **File Outputs:** {} added, {} removed, {} shared (net {:+} files)\n",
        a.id,
        b.id,
        a.id,
        a.task,
        &a.sha256[..12.min(a.sha256.len())],
        b.id,
        b.task,
        &b.sha256[..12.min(b.sha256.len())],
        diff.added,
        diff.removed,
        diff.similarity_pct,
        line_delta,
        files_added.len(),
        files_removed.len(),
        files_shared.len(),
        file_delta
    );
    if !files_added.is_empty() {
        delta_md.push_str(&format!("- **New Deliverables:** {}\n", files_added.join(", ")));
    }
    if !files_removed.is_empty() {
        delta_md.push_str(&format!("- **Removed Deliverables:** {}\n", files_removed.join(", ")));
    }
    if !diff.unified.trim().is_empty() {
        delta_md.push_str(&format!("\n```diff\n{}\n```\n", diff.unified.trim_end()));
    }
    let payload = json!({
        "receipt_a": a.to_json(),
        "receipt_b": b.to_json(),
        "added": diff.added,
        "removed": diff.removed,
        "unchanged": diff.unchanged,
        "similarity_pct": diff.similarity_pct,
        "line_delta": line_delta,
        "file_delta": file_delta,
        "files_added": files_added,
        "files_removed": files_removed,
        "files_shared": files_shared,
        "hunks": diff.hunks.iter().map(file_history::DiffHunk::to_json).collect::<Vec<_>>(),
        "unified": diff.unified,
        "delta_markdown": delta_md,
    });
    (payload, delta_md)
}

/// Routes workspace file listing/content/upload/export, file history/restore, and audit bundles.
fn try_route_files_and_audit(
    stream: &mut std::net::TcpStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/files/history") => {
            let rel = query_param(&req.query, "path").unwrap_or_default();
            let rel = rel.trim().to_string();
            if rel.is_empty() {
                let tracked = file_history::list_tracked_files(&runtime.cfg.data_dir);
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "person_id": person.id,
                        "count": tracked.len(),
                        "tracked_files": tracked,
                    }),
                ));
            }
            let mut payload = match file_history::history_payload_json(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &rel,
            ) {
                Ok(v) => v,
                Err(e) => {
                    let code = if is_history_security_error(&e) { 403 } else { 400 };
                    return Some(respond_json(stream, code, &json!({"error": e})));
                }
            };
            let diff_pair = query_param(&req.query, "diff")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let (rev_a, rev_b) = if let Some(pair) = diff_pair {
                let (a, b) = pair
                    .split_once(',')
                    .map(|(x, y)| (x.trim().to_string(), y.trim().to_string()))
                    .unwrap_or((pair, "current".to_string()));
                (Some(a), if b.is_empty() { "current".to_string() } else { b })
            } else {
                let a = query_param(&req.query, "rev_a")
                    .or_else(|| query_param(&req.query, "from"))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let b = query_param(&req.query, "rev_b")
                    .or_else(|| query_param(&req.query, "to"))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "current".to_string());
                (a, b)
            };
            if let Some(a) = rev_a {
                match file_history::diff_revisions(
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &rel,
                    &a,
                    &rev_b,
                ) {
                    Ok(diff) => {
                        payload["diff"] = diff.to_json();
                    }
                    Err(e) => {
                        let code = if is_history_security_error(&e) {
                            403
                        } else if e.contains("not found") {
                            404
                        } else {
                            400
                        };
                        return Some(respond_json(stream, code, &json!({"error": e})));
                    }
                }
            }
            respond_json(stream, 200, &payload)
        }
        ("POST", "/api/files/restore") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let rel = parsed["path"].as_str().unwrap_or("").trim().to_string();
            let rev_id = parsed["rev_id"]
                .as_str()
                .or_else(|| parsed["revision_id"].as_str())
                .or_else(|| parsed["id"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if rel.is_empty() || rev_id.is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "path and rev_id are required"}),
                ));
            }
            match file_history::restore_revision(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &rel,
                &rev_id,
            ) {
                Ok(res) => {
                    state.search_cache.invalidate(&person.id);
                    let mut out = res.to_json();
                    out["restored"] = json!(true);
                    respond_json(stream, 200, &out)
                }
                Err(e) => {
                    let code = if is_history_security_error(&e) {
                        403
                    } else if e.contains("not found") {
                        404
                    } else {
                        400
                    };
                    respond_json(stream, code, &json!({"error": e}))
                }
            }
        }
        ("GET", "/api/audit/bundle") => {
            let format = query_param(&req.query, "format")
                .unwrap_or_else(|| "json".to_string())
                .to_ascii_lowercase();
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let security = tripwire::audit_section(&root_cfg.data_dir, &person.id);
            if format == "md" || format == "markdown" {
                let mut bundle = audit_bundle::build_audit_bundle(&runtime.cfg, person);
                bundle["integrity"]["security_events_sha256"] = security["events_sha256"].clone();
                bundle["security"] = security;
                let mut md = audit_bundle::render_audit_bundle_markdown(&bundle);
                md.push_str(&render_security_markdown(&bundle["security"]));
                let mut headers = vec![("X-Content-Type-Options", "nosniff".to_string())];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-audit-bundle.md\"".to_string(),
                    ));
                }
                respond_bytes(
                    stream,
                    200,
                    "text/markdown; charset=utf-8",
                    md.as_bytes(),
                    &headers,
                )
            } else {
                let cached = state.search_cache.get_if_fresh(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                );
                let bytes = audit_bundle::build_audit_bundle_json_bytes_with_chain(
                    &runtime.cfg,
                    person,
                    &security,
                    cached
                        .as_ref()
                        .map(|idx| (idx.chain.as_slice(), idx.chain_valid, idx.head_hash.as_str())),
                );
                let mut headers = vec![("X-Content-Type-Options", "nosniff".to_string())];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-audit-bundle.json\"".to_string(),
                    ));
                }
                respond_bytes(
                    stream,
                    200,
                    "application/json; charset=utf-8",
                    &bytes,
                    &headers,
                )
            }
        }
        ("GET", "/api/files/pack") => {
            let raw_files = query_param(&req.query, "files").unwrap_or_default();
            let title = query_param(&req.query, "title")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Client Deliverable Pack".to_string());
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let format = query_param(&req.query, "format")
                .unwrap_or_else(|| "md".to_string())
                .to_ascii_lowercase();
            let preflight_only = query_param(&req.query, "preflight")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let include_summary = query_param(&req.query, "summary")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let receipt_id = query_param(&req.query, "receipt_id")
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let mut requested: Vec<String> = raw_files
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .take(25)
                .collect();
            let explicit = !requested.is_empty();
            let idx_opt = if requested.is_empty() || !receipt_id.is_empty() || include_summary {
                Some(state.search_cache.get(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                ))
            } else {
                state.search_cache.get_if_fresh(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                )
            };
            if requested.is_empty() {
                if let Some(idx) = &idx_opt {
                    let (_, items) = idx.list_files(0, Some(15), "");
                    for item in items {
                        if let Some(p) = item["path"].as_str() {
                            requested.push(p.to_string());
                        }
                    }
                }
            }
            let mut extra_receipts: Vec<(&str, u64)> = Vec::new();
            let mut summary_runs: Vec<&receipts::ChainedReceipt> = Vec::new();
            if let Some(idx) = &idx_opt {
                if !receipt_id.is_empty() {
                    if let Some(rcpt) = receipts::find_receipt(&idx.chain, &receipt_id) {
                        extra_receipts.push((rcpt.summary.as_str(), rcpt.ts));
                    }
                }
                let scan_limit = if include_summary { 15 } else { 8 };
                for rcpt in idx.chain.iter().rev() {
                    if include_summary && rcpt.finished && !rcpt.answered && summary_runs.len() < 15 {
                        summary_runs.push(rcpt);
                        if !explicit {
                            for f in &rcpt.files {
                                if requested.len() < 25 && !requested.iter().any(|x| x == f) {
                                    requested.push(f.clone());
                                }
                            }
                        }
                    }
                    if extra_receipts.len() < scan_limit
                        && !rcpt.summary.is_empty()
                        && rcpt.id != receipt_id
                        && rcpt.thread_id != receipt_id
                    {
                        extra_receipts.push((rcpt.summary.as_str(), rcpt.ts));
                    }
                    if summary_runs.len() >= 15 && extra_receipts.len() >= scan_limit {
                        break;
                    }
                }
            }
            let preflight = match inspect_workspace_pack(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &requested,
                explicit,
                &extra_receipts,
                None,
            ) {
                Ok(res) => res,
                Err((status, err)) => return Some(respond_json(stream, status, &json!({"error": err}))),
            };
            let mut summary_gates: Vec<SummaryRunGate<'_>> = Vec::with_capacity(summary_runs.len());
            if !summary_runs.is_empty() {
                let mut dummy_cits = Vec::new();
                let mut dummy_seen = std::collections::HashMap::new();
                for &rcpt in &summary_runs {
                    dummy_cits.clear();
                    dummy_seen.clear();
                    let mut v = 0usize;
                    let mut s = 0usize;
                    let mut b = 0usize;
                    let (tot, rv, rs, rb) = collect_citations_into(
                        Some(&runtime.cfg.data_dir),
                        &runtime.cfg.workspace_dir,
                        &rcpt.summary,
                        rcpt.ts,
                        None,
                        &mut dummy_cits,
                        &mut dummy_seen,
                        &mut v,
                        &mut s,
                        &mut b,
                    );
                    summary_gates.push(SummaryRunGate {
                        receipt: rcpt,
                        checked: tot,
                        verified: rv,
                        stale: rs,
                        broken: rb,
                    });
                }
            }
            if preflight_only {
                let mut out = preflight.to_json();
                if include_summary {
                    out["summary_runs_count"] = json!(summary_gates.len());
                    out["summary_runs"] = json!(summary_gates.iter().map(SummaryRunGate::to_json).collect::<Vec<_>>());
                }
                return Some(respond_json(stream, 200, &out));
            }
            let preflight_ok = preflight.ok();
            let ready_count = preflight.ready_count;
            let gate_label = preflight.citation_gate();
            let delta_a = query_param(&req.query, "delta_a").unwrap_or_default();
            let delta_b = query_param(&req.query, "delta_b").unwrap_or_default();
            let delta_file = query_param(&req.query, "delta_file").unwrap_or_default();
            let mut delta_md_opt: Option<String> = None;
            if !delta_a.trim().is_empty() && !delta_b.trim().is_empty() {
                let owned_idx;
                let idx = match &idx_opt {
                    Some(i) => i,
                    None => {
                        owned_idx = state.search_cache.get(
                            &person.id,
                            &runtime.cfg.data_dir,
                            &runtime.cfg.workspace_dir,
                            &runtime.cfg.memory_path,
                        );
                        &owned_idx
                    }
                };
                let Some(ra) = receipts::find_receipt(&idx.chain, delta_a.trim()) else {
                    return Some(respond_json(stream, 404, &json!({"error": "delta receipt a not found"})));
                };
                let Some(rb) = receipts::find_receipt(&idx.chain, delta_b.trim()) else {
                    return Some(respond_json(stream, 404, &json!({"error": "delta receipt b not found"})));
                };
                let (_, delta_md) = build_receipt_compare(ra, rb);
                delta_md_opt = Some(delta_md);
            } else if !delta_file.trim().is_empty() {
                let rev_a = query_param(&req.query, "rev_a").unwrap_or_default();
                let rev_b = query_param(&req.query, "rev_b").unwrap_or_else(|| "current".to_string());
                match file_history::diff_revisions(
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    delta_file.trim(),
                    rev_a.trim(),
                    rev_b.trim(),
                ) {
                    Ok(diff) => {
                        let mut dmd = format!(
                            "## Verified Snapshot Delta (`{}`: `{}` -> `{}`)\n\n- **Changes:** +{} / -{} lines ({}% similarity)\n- **SHA-256:** `{}` -> `{}`\n",
                            diff.path,
                            diff.rev_a,
                            diff.rev_b,
                            diff.added,
                            diff.removed,
                            diff.similarity_pct,
                            &diff.sha256_a[..12.min(diff.sha256_a.len())],
                            &diff.sha256_b[..12.min(diff.sha256_b.len())]
                        );
                        if !diff.unified.trim().is_empty() {
                            dmd.push_str(&format!("\n```diff\n{}\n```\n", diff.unified.trim_end()));
                        }
                        delta_md_opt = Some(dmd);
                    }
                    Err(e) => {
                        let code = if is_history_security_error(&e) { 403 } else { 404 };
                        return Some(respond_json(stream, code, &json!({"error": e})));
                    }
                }
            }
            let has_delta = delta_md_opt.is_some();
            let has_summary = !summary_gates.is_empty();
            let total_cits = preflight.citations.len();
            let summary_cits_hdr = format!("{}/{}", preflight.citations_verified, total_cits);
            let summary_stale_hdr = (preflight.citations_stale + preflight.citations_broken).to_string();
            if format == "html" {
                let (html, pack_sha) = build_workspace_handoff_html(
                    &preflight,
                    &person.name,
                    &title,
                    delta_md_opt.as_deref(),
                    &summary_gates,
                );
                let mut headers = vec![
                    ("X-Content-Type-Options", "nosniff".to_string()),
                    (
                        "Content-Security-Policy",
                        "default-src 'none'; style-src 'unsafe-inline'".to_string(),
                    ),
                    ("X-Ergon-Pack-SHA256", pack_sha),
                    (
                        "X-Ergon-Pack-Preflight",
                        if preflight_ok { "pass" } else { "warn" }.to_string(),
                    ),
                    ("X-Ergon-Pack-Citation-Gate", gate_label.to_string()),
                    ("X-Ergon-Pack-Ready-Count", ready_count.to_string()),
                    (
                        "X-Ergon-Pack-Delta",
                        if has_delta { "yes" } else { "none" }.to_string(),
                    ),
                    (
                        "X-Ergon-Pack-Summary",
                        if has_summary { "yes" } else { "none" }.to_string(),
                    ),
                    ("X-Ergon-Pack-Summary-Runs", summary_gates.len().to_string()),
                    ("X-Ergon-Pack-Summary-Citations", summary_cits_hdr),
                    ("X-Ergon-Pack-Summary-Stale", summary_stale_hdr),
                ];
                if download {
                    headers.push((
                        "Content-Disposition",
                        "attachment; filename=\"ergon-client-handoff.html\"".to_string(),
                    ));
                }
                return Some(respond_bytes(
                    stream,
                    200,
                    "text/html; charset=utf-8",
                    html.as_bytes(),
                    &headers,
                ));
            }
            let mut md = format_workspace_pack_markdown(
                &preflight,
                &person.name,
                &title,
                &summary_gates,
            );
            if let Some(delta_md) = delta_md_opt {
                md.push_str("\n---\n\n");
                md.push_str(&delta_md);
            }
            let pack_sha = receipts::sha256_hex(md.as_bytes());
            let mut headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                ("X-Ergon-Pack-SHA256", pack_sha),
                (
                    "X-Ergon-Pack-Preflight",
                    if preflight_ok { "pass" } else { "warn" }.to_string(),
                ),
                ("X-Ergon-Pack-Citation-Gate", gate_label.to_string()),
                ("X-Ergon-Pack-Ready-Count", ready_count.to_string()),
                (
                    "X-Ergon-Pack-Delta",
                    if has_delta { "yes" } else { "none" }.to_string(),
                ),
                (
                    "X-Ergon-Pack-Summary",
                    if has_summary { "yes" } else { "none" }.to_string(),
                ),
                ("X-Ergon-Pack-Summary-Runs", summary_gates.len().to_string()),
                ("X-Ergon-Pack-Summary-Citations", summary_cits_hdr),
                ("X-Ergon-Pack-Summary-Stale", summary_stale_hdr),
            ];
            if download {
                headers.push((
                    "Content-Disposition",
                    "attachment; filename=\"ergon-client-pack.md\"".to_string(),
                ));
            }
            respond_bytes(
                stream,
                200,
                "text/markdown; charset=utf-8",
                md.as_bytes(),
                &headers,
            )
        }
        ("POST", "/api/citations/verify") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let mut text = parsed["text"].as_str().unwrap_or("").to_string();
            let receipt_id = parsed["receipt_id"].as_str().unwrap_or("").trim();
            let mut receipt_ts = 0u64;
            if !receipt_id.is_empty() {
                let idx = state.search_cache.get(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                );
                if let Some(rcpt) = receipts::find_receipt(&idx.chain, receipt_id) {
                    receipt_ts = rcpt.ts;
                    if text.is_empty() {
                        text = format!("{}\n{}", rcpt.task, rcpt.summary);
                    }
                }
            }
            let expected_map: std::collections::HashMap<String, String> = parsed["expected_shas"]
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            let exp_opt = (!expected_map.is_empty()).then_some(&expected_map);
            let mut citations = Vec::new();
            let mut seen = std::collections::HashMap::new();
            let mut verified = 0usize;
            let mut stale = 0usize;
            let mut broken = 0usize;
            collect_citations_into(
                Some(&runtime.cfg.data_dir),
                &runtime.cfg.workspace_dir,
                &text,
                receipt_ts,
                exp_opt,
                &mut citations,
                &mut seen,
                &mut verified,
                &mut stale,
                &mut broken,
            );
            let report = CitationReport {
                total: citations.len(),
                verified,
                stale,
                broken,
                citations,
            };
            respond_json(stream, 200, &report.to_json())
        }
        ("GET", "/api/files") => {
            let (offset, limit, q) = if req.query.is_empty() {
                (0usize, None, String::new())
            } else {
                (
                    query_param(&req.query, "offset")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0),
                    query_param(&req.query, "limit").and_then(|s| s.parse::<usize>().ok()),
                    query_param(&req.query, "q").unwrap_or_default(),
                )
            };
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            respond_json_with(stream, 200, |buf| {
                idx.write_files_json(buf, &person.id, offset, limit, &q);
            })
        }
        ("GET", "/api/files/content") => {
            let rel = query_param(&req.query, "path").unwrap_or_default();
            let rel = rel.trim().to_string();
            if rel.is_empty() {
                return Some(respond_json(stream, 400, &json!({"error": "path is required"})));
            }
            if rel.len() > 240 || rel.split('/').any(|s| s.len() > 200) {
                return Some(respond_json(stream, 400, &json!({"error": "path is too long"})));
            }
            if rel.contains('\0')
                || rel.contains('\\')
                || rel.contains(':')
                || rel.chars().any(|c| c.is_control())
                || rel.starts_with('/')
                || rel.to_ascii_lowercase().contains("%2e%2e")
                || rel.split('/').any(|seg| seg == ".." || seg == "." || seg.is_empty() || seg.starts_with('.'))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "path escapes the workspace"}),
                ));
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &rel, true) {
                Ok(p) => p,
                Err(e) => return Some(respond_json(stream, 403, &json!({"error": e})),)};
            if full == runtime.cfg.workspace_dir || full.is_dir() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "path must name a file inside the workspace"}),
                ));
            }
            let md = match std::fs::metadata(&full) {
                Ok(m) => m,
                Err(_) => return Some(respond_json(stream, 404, &json!({"error": "file not found"})),)};
            const MAX_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;
            let download_requested = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            // Documents preview as their extracted text (a workbook one
            // sheet at a time); downloading still sends the original bytes.
            if let (false, Some(kind)) = (download_requested, docs::doc_kind(&rel)) {
                let sheet = query_param(&req.query, "sheet");
                let name = full.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string();
                let ext = full.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
                let mtime_ms = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let mut v = json!({
                    "path": rel, "name": name, "size": md.len(), "mtime_ms": mtime_ms, "ext": ext,
                });
                match docs::extract(&full, kind, sheet.as_deref()) {
                    Ok(ex) => {
                        v["is_binary"] = json!(false);
                        v["content"] = json!(ex.text);
                        v["extracted"] = json!({
                            "kind": kind.label(),
                            "format": ex.format,
                            "sheets": ex.sheets,
                            "sheet": ex.sheet,
                            "pages": ex.pages,
                        });
                    }
                    Err(e) => {
                        v["is_binary"] = json!(true);
                        v["content"] = json!("");
                        v["extract_error"] = json!(e);
                    }
                }
                return Some(respond_json(stream, 200, &v));
            }
            if md.len() > MAX_PREVIEW_BYTES && !download_requested {
                return Some(respond_json(
                    stream,
                    413,
                    &json!({"error": "file exceeds the 2 MiB preview limit"}),
                ));
            }
            let bytes = match std::fs::read(&full) {
                Ok(b) => b,
                Err(_) => return Some(respond_json(stream, 403, &json!({"error": "file could not be read"})),)};
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let download = query_param(&req.query, "download")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if download {
                let safe_name: String = name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') { c } else { '_' })
                    .collect();
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"{safe_name}\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                return Some(respond_bytes(stream, 200, mime_for_ext(&ext), &bytes, &headers));
            }
            let mtime_ms = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            match String::from_utf8(bytes) {
                Ok(content) => respond_json(
                    stream,
                    200,
                    &json!({
                        "path": rel,
                        "name": name,
                        "size": md.len(),
                        "mtime_ms": mtime_ms,
                        "ext": ext,
                        "is_binary": false,
                        "content": content,
                    }),
                ),
                Err(_) => respond_json(
                    stream,
                    200,
                    &json!({
                        "path": rel,
                        "name": name,
                        "size": md.len(),
                        "mtime_ms": mtime_ms,
                        "ext": ext,
                        "is_binary": true,
                        "content": "",
                    }),
                ),
            }
        }
        ("POST", "/api/files/export") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let rel = parsed["path"].as_str().unwrap_or("").trim().to_string();
            if rel.is_empty() {
                return Some(respond_json(stream, 400, &json!({"error": "path is required"})));
            }
            match export_workspace_file(&runtime.cfg, &rel, parsed["format"].as_str(), None) {
                Ok((out, bytes, label)) => {
                    state.search_cache.invalidate(&person.id);
                    respond_json(stream, 201, &json!({"path": out, "size": bytes, "kind": label, "source": rel}))
                }
                Err(e) => {
                    let status = if e.contains("escapes")
                        || e.contains("absolute")
                        || e.contains("symlink")
                        || e.contains("hidden")
                    {
                        403
                    } else {
                        400
                    };
                    respond_json(stream, status, &json!({"error": e}))
                }
            }
        }
        ("POST", "/api/files/upload") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let raw_rel = parsed["path"]
                .as_str()
                .or_else(|| parsed["name"].as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw_rel.is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "filename is required"}),
                ));
            }
            if raw_rel.len() > 240 || raw_rel.split('/').any(|s| s.len() > 200) {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "filename is too long"}),
                ));
            }
            if raw_rel.contains('\0')
                || raw_rel.contains('\\')
                || raw_rel.contains(':')
                || raw_rel.chars().any(|c| c.is_control())
                || raw_rel.starts_with('/')
                || raw_rel.to_ascii_lowercase().contains("%2e%2e")
                || raw_rel.split('/').any(|seg| seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('.'))
            {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "invalid workspace filename"}),
                ));
            }
            let full = match sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, false) {
                Ok(p) => p,
                Err(e) => return Some(respond_json(stream, 403, &json!({"error": e})),)};
            if sandbox_path_resolving(&runtime.cfg.workspace_dir, &raw_rel, true).is_err() {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "path escapes the workspace"}),
                ));
            }
            if full == runtime.cfg.workspace_dir {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "filename must name a file"}),
                ));
            }
            let encoding = parsed["encoding"].as_str().unwrap_or("utf8");
            let content_str = match parsed["content"].as_str() {
                Some(s) => s,
                None => {
                    return Some(respond_json(
                        stream,
                        400,
                        &json!({"error": "content is required"}),
                    ))}
            };
            let bytes = if encoding.eq_ignore_ascii_case("base64") {
                match decode_base64(content_str) {
                    Ok(b) => b,
                    Err(e) => return Some(respond_json(stream, 400, &json!({"error": e})),)}
            } else {
                content_str.as_bytes().to_vec()
            };
            if bytes.len() as u64 > docs::MAX_DOC_BYTES {
                return Some(respond_json(
                    stream,
                    413,
                    &json!({"error": "files over 25 MB cannot be uploaded"}),
                ));
            }
            if full.symlink_metadata().is_ok_and(|md| md.file_type().is_symlink()) {
                return Some(respond_json(
                    stream,
                    403,
                    &json!({"error": "cannot overwrite symlink"}),
                ));
            }
            let _ = file_history::record_revision(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &raw_rel,
                "upload",
            );
            if let Err(e) = atomic_write_bytes(&full, &bytes) {
                return Some(respond_json(
                    stream,
                    500,
                    &json!({"error": format!("save file failed: {e}")}),
                ));
            }
            state.search_cache.invalidate(&person.id);
            let name = full
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&raw_rel)
                .to_string();
            let ext = full
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let mtime_ms = std::fs::metadata(&full)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            respond_json(
                stream,
                201,
                &json!({
                    "path": raw_rel,
                    "name": name,
                    "size": bytes.len(),
                    "mtime_ms": mtime_ms,
                    "ext": ext,
                }),
            )
        }
        _ => return None,
    })
}

/// Routes search, receipts, skills, routines, decisions, offers, brief, today, and graph.
fn try_route_knowledge_and_routines(
    stream: &mut std::net::TcpStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/search") => {
            let (q, scope, limit) = if req.query.is_empty() {
                (String::new(), String::from("all"), 30usize)
            } else {
                (
                    query_param(&req.query, "q").unwrap_or_default(),
                    query_param(&req.query, "scope")
                        .map(|s| s.to_ascii_lowercase())
                        .filter(|s| matches!(s.as_str(), "all" | "receipts" | "files" | "memory"))
                        .unwrap_or_else(|| "all".to_string()),
                    query_param(&req.query, "limit")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(30),
                )
            };
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            respond_json_with(stream, 200, |buf| {
                idx.write_search_json(buf, &q, &scope, limit);
            })
        }
        ("GET", "/api/receipts") => {
            let (offset, limit, q, status) = if req.query.is_empty() {
                (0usize, 50usize, String::new(), String::from("all"))
            } else {
                (
                    query_param(&req.query, "offset")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0),
                    query_param(&req.query, "limit")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(50),
                    query_param(&req.query, "q").unwrap_or_default(),
                    query_param(&req.query, "status")
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_else(|| "all".to_string()),
                )
            };
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            respond_json_with(stream, 200, |buf| {
                idx.write_receipts_json(buf, offset, limit, &q, &status);
            })
        }
        ("GET", "/api/receipts/compare") => {
            let a_id = query_param(&req.query, "a").unwrap_or_default();
            let b_id = query_param(&req.query, "b").unwrap_or_default();
            if a_id.trim().is_empty() || b_id.trim().is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "query parameters a and b are required"}),
                ));
            }
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let Some(ra) = receipts::find_receipt(&idx.chain, a_id.trim()) else {
                return Some(respond_json(stream, 404, &json!({"error": "receipt a not found"})));
            };
            let Some(rb) = receipts::find_receipt(&idx.chain, b_id.trim()) else {
                return Some(respond_json(stream, 404, &json!({"error": "receipt b not found"})));
            };
            let (payload, _) = build_receipt_compare(ra, rb);
            respond_json(stream, 200, &payload)
        }
        ("GET", "/api/receipts/verify") => {
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            respond_json(
                stream,
                200,
                &json!({
                    "total": idx.chain.len(),
                    "valid": idx.chain_valid,
                    "head_hash": idx.head_hash,
                }),
            )
        }
        ("GET", path) if path.starts_with("/api/receipts/") && path.ends_with("/export") => {
            let raw_id = path
                .strip_prefix("/api/receipts/")
                .and_then(|r| r.strip_suffix("/export"))
                .unwrap_or("");
            let id = percent_decode(raw_id);
            let fmt = query_param(&req.query, "format")
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "json".to_string());
            if fmt != "json" && fmt != "md" {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "format must be md or json"}),
                ));
            }
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let Some(rcpt) = receipts::find_receipt(&idx.chain, &id) else {
                return Some(respond_json(stream, 404, &json!({"error": "receipt not found"})));
            };
            let safe_id: String = rcpt
                .id
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') { c } else { '_' })
                .collect();
            if fmt == "md" {
                let md = receipts::export_receipt_markdown(
                    rcpt,
                    &runtime.cfg.workspace_dir,
                    idx.chain.len(),
                    idx.chain_valid,
                );
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"receipt-{safe_id}.md\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                respond_bytes(stream, 200, "text/markdown; charset=utf-8", md.as_bytes(), &headers)
            } else {
                let body = receipts::export_receipt_json(
                    rcpt,
                    &runtime.cfg.workspace_dir,
                    idx.chain.len(),
                    idx.chain_valid,
                );
                let bytes = serde_json::to_vec_pretty(&body).unwrap_or_else(|_| body.to_string().into_bytes());
                let headers = [
                    ("Content-Disposition", format!("attachment; filename=\"receipt-{safe_id}.json\"")),
                    ("X-Content-Type-Options", "nosniff".to_string()),
                ];
                respond_bytes(stream, 200, "application/json; charset=utf-8", &bytes, &headers)
            }
        }
        ("GET", "/api/skills") => {
            respond_json(stream, 200, &skill_catalog::catalog_json())
        }
        ("POST", "/api/skills/test") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let res = skill_catalog::handle_skill_test_request(&parsed);
            respond_json(stream, 200, &res)
        }
        ("GET", path) if path.starts_with("/api/skills/") => {
            let id = path.trim_start_matches("/api/skills/").trim();
            match skill_catalog::get_skill(id) {
                Some(skill) => respond_json(stream, 200, &skill.to_json()),
                None => respond_json(stream, 404, &json!({"error": "skill not found"})),
            }
        }
        ("GET", "/api/routines") => {
            let now_ms = records::now_millis();
            let raw = routines::list_routines(&runtime.cfg.data_dir);
            let history = scheduler::list_run_history(&runtime.cfg.data_dir, None, 500);
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let enriched: Vec<Value> = raw
                .iter()
                .map(|r| {
                    scheduler::enrich_routine_with_verification(
                        r,
                        now_ms,
                        &history,
                        &idx.chain,
                        idx.chain_valid,
                    )
                })
                .collect();
            let tick_secs = (scheduler_tick_ms() / 1000).max(1);
            let sched_status = scheduler::scheduler_status_json(
                state.scheduler_enabled,
                tick_secs,
                &raw,
                &history,
                now_ms,
            );
            respond_json(
                stream,
                200,
                &json!({
                    "routines": enriched,
                    "scheduler": sched_status,
                }),
            )
        }
        ("POST", "/api/routines") => {
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let name = parsed["name"].as_str().unwrap_or("");
            let prompt = parsed["prompt"].as_str().unwrap_or("");
            let schedule = parsed["schedule"].as_str().unwrap_or("manual");
            let skill_id = parsed["skill_id"].as_str();
            let enabled = parsed["enabled"].as_bool().unwrap_or(true);
            let interval_secs = if parsed.get("interval_secs").is_some() && !parsed["interval_secs"].is_null() {
                match parsed["interval_secs"].as_u64() {
                    Some(v) => Some(v),
                    None => {
                        return Some(respond_json(
                            stream,
                            400,
                            &json!({"error": "interval_secs must be a positive integer"}),
                        ))}
                }
            } else {
                None
            };
            let playbook = routines::PlaybookOptions {
                steps: parsed["steps"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                expected_inputs: parsed["expected_inputs"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                pack_deliverables: parsed["pack_deliverables"].as_bool().unwrap_or(false),
            };
            match routines::create_playbook_routine(
                &runtime.cfg.data_dir,
                name,
                prompt,
                schedule,
                interval_secs,
                skill_id,
                enabled,
                &playbook,
            ) {
                Ok(routine) => {
                    let now_ms = records::now_millis();
                    let enriched = scheduler::enrich_routine_json(&routine, now_ms, &[]);
                    respond_json(stream, 201, &json!({"routine": enriched}))
                }
                Err(e) => respond_json(stream, 400, &json!({"error": e})),
            }
        }
        ("GET", path) if path.starts_with("/api/routines/") && path.ends_with("/history") => {
            let id = path
                .strip_prefix("/api/routines/")
                .and_then(|r| r.strip_suffix("/history"))
                .unwrap_or("")
                .trim()
                .to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_json(stream, 400, &json!({"error": "invalid routine id"})));
            }
            let limit = query_param(&req.query, "limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50)
                .clamp(1, 500);
            let runs = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), limit);
            if runs.is_empty() && routines::get_routine(&runtime.cfg.data_dir, &id).is_none() {
                return Some(respond_json(stream, 404, &json!({"error": "routine not found"})));
            }
            let items: Vec<Value> = runs.iter().map(|r| r.to_json()).collect();
            respond_json(
                stream,
                200,
                &json!({
                    "routine_id": id,
                    "count": items.len(),
                    "runs": items,
                    "history": items,
                }),
            )
        }
        ("PATCH", path) if path.starts_with("/api/routines/") => {
            let id = path.trim_start_matches("/api/routines/").trim().to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_json(stream, 400, &json!({"error": "invalid routine id"})));
            }
            let parsed: Value = match serde_json::from_slice::<Value>(&req.body) {
                Ok(v) if v.is_object() => v,
                _ => {
                    return Some(respond_json(
                        stream,
                        400,
                        &json!({"error": "PATCH body must be a JSON object"}),
                    ))}
            };
            match routines::patch_routine(&runtime.cfg.data_dir, &id, &parsed) {
                Ok(Some(routine)) => {
                    let now_ms = records::now_millis();
                    let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), 20);
                    let enriched = scheduler::enrich_routine_json(&routine, now_ms, &hist);
                    respond_json(stream, 200, &json!({"routine": enriched}))
                }
                Ok(None) => respond_json(stream, 404, &json!({"error": "routine not found"})),
                Err(e) => respond_json(stream, 400, &json!({"error": e})),
            }
        }
        ("DELETE", path) if path.starts_with("/api/routines/") => {
            let id = path.trim_start_matches("/api/routines/").trim().to_string();
            if id.is_empty() || id.contains('/') {
                return Some(respond_json(stream, 400, &json!({"error": "invalid routine id"})));
            }
            match routines::delete_routine(&runtime.cfg.data_dir, &id) {
                Ok(true) => respond_json(stream, 200, &json!({"deleted": id})),
                Ok(false) => respond_json(stream, 404, &json!({"error": "routine not found"})),
                Err(e) => respond_json(stream, 500, &json!({"error": e})),
            }
        }
        ("POST", "/api/scheduler/catchup") => {
            let now_ms = records::now_millis();
            let raw = routines::list_routines(&runtime.cfg.data_dir);
            let due = scheduler::find_due_routines(&raw, now_ms);
            let mut results: Vec<Value> = Vec::new();
            let mut caught_up_count = 0usize;
            let mut failed_count = 0usize;
            for rt in due.iter().take(10) {
                let rid = rt["id"].as_str().unwrap_or("");
                if rid.is_empty() {
                    continue;
                }
                let missed = scheduler::count_missed_windows(rt, now_ms);
                if let Some(exec) = execute_routine_run(state, person, runtime, rid, "catchup") {
                    let ok = exec.error.is_none();
                    if ok {
                        caught_up_count += 1;
                    } else {
                        failed_count += 1;
                    }
                    results.push(json!({
                        "routine_id": rid,
                        "ok": ok,
                        "missed_windows_cleared": if ok { missed } else { 0 },
                        "thread_id": exec.thread_id,
                        "error": exec.error,
                    }));
                }
            }
            let updated_now = records::now_millis();
            let updated_raw = routines::list_routines(&runtime.cfg.data_dir);
            let history = scheduler::list_run_history(&runtime.cfg.data_dir, None, 500);
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let enriched: Vec<Value> = updated_raw
                .iter()
                .map(|r| {
                    scheduler::enrich_routine_with_verification(
                        r,
                        updated_now,
                        &history,
                        &idx.chain,
                        idx.chain_valid,
                    )
                })
                .collect();
            let tick_secs = (scheduler_tick_ms() / 1000).max(1);
            let sched_status = scheduler::scheduler_status_json(
                state.scheduler_enabled,
                tick_secs,
                &updated_raw,
                &history,
                updated_now,
            );
            respond_json(
                stream,
                200,
                &json!({
                    "total_due": due.len(),
                    "caught_up_count": caught_up_count,
                    "failed_count": failed_count,
                    "results": results,
                    "routines": enriched,
                    "scheduler": sched_status,
                }),
            )
        }
        ("POST", path)
            if (path.starts_with("/api/routines/") || path.starts_with("/api/playbooks/"))
                && path.ends_with("/catchup") =>
        {
            let prefix = if path.starts_with("/api/playbooks/") {
                "/api/playbooks/"
            } else {
                "/api/routines/"
            };
            let raw_id = path
                .strip_prefix(prefix)
                .and_then(|r| r.strip_suffix("/catchup"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_json(stream, 400, &json!({"error": "invalid routine id"})));
            }
            let Some(rt) = routines::get_routine(&runtime.cfg.data_dir, &id) else {
                return Some(respond_json(stream, 404, &json!({"error": "routine not found"})));
            };
            let now_ms = records::now_millis();
            let missed = scheduler::count_missed_windows(&rt, now_ms);
            let force = query_param(&req.query, "force")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            if missed == 0 && !force {
                let hist = scheduler::list_run_history(&runtime.cfg.data_dir, Some(&id), 20);
                let idx = state.search_cache.get(
                    &person.id,
                    &runtime.cfg.data_dir,
                    &runtime.cfg.workspace_dir,
                    &runtime.cfg.memory_path,
                );
                let enriched = scheduler::enrich_routine_with_verification(
                    &rt,
                    now_ms,
                    &hist,
                    &idx.chain,
                    idx.chain_valid,
                );
                return Some(respond_json(
                    stream,
                    200,
                    &json!({
                        "routine": enriched,
                        "caught_up": false,
                        "missed_windows": 0,
                    }),
                ));
            }
            let Some(exec) = execute_routine_run(state, person, runtime, &id, "catchup") else {
                return Some(respond_json(stream, 404, &json!({"error": "routine not found"})));
            };
            if let Some(err) = exec.error {
                respond_json(
                    stream,
                    exec.http_status,
                    &json!({
                        "error": err,
                        "caught_up": false,
                        "missed_windows": missed,
                        "thread_id": exec.thread_id,
                        "routine": exec.routine,
                        "run": exec.run_record.to_json(),
                    }),
                )
            } else {
                respond_json(
                    stream,
                    200,
                    &json!({
                        "caught_up": true,
                        "missed_windows_cleared": missed,
                        "routine": exec.routine,
                        "thread": exec.thread,
                        "thread_id": exec.thread_id,
                        "run": exec.run_record.to_json(),
                    }),
                )
            }
        }
        ("POST", path) if path.starts_with("/api/routines/") && path.ends_with("/run") => {
            let id = path
                .strip_prefix("/api/routines/")
                .and_then(|r| r.strip_suffix("/run"))
                .unwrap_or("")
                .trim()
                .to_string();
            let Some(exec) = execute_routine_run(state, person, runtime, &id, "manual") else {
                return Some(respond_json(stream, 404, &json!({"error": "routine not found"})));
            };
            if let Some(err) = exec.error {
                respond_json(
                    stream,
                    exec.http_status,
                    &json!({
                        "error": err,
                        "thread_id": exec.thread_id,
                        "routine": exec.routine,
                        "run": exec.run_record.to_json(),
                    }),
                )
            } else {
                respond_json(
                    stream,
                    200,
                    &json!({
                        "routine": exec.routine,
                        "thread": exec.thread,
                        "thread_id": exec.thread_id,
                        "run": exec.run_record.to_json(),
                    }),
                )
            }
        }
        ("GET", path) if path.starts_with("/api/playbooks/") && path.ends_with("/export") => {
            let raw_id = path
                .strip_prefix("/api/playbooks/")
                .and_then(|r| r.strip_suffix("/export"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "invalid playbook id"}),
                ));
            }
            let Some((pack, digest, filename)) =
                routines::export_playbook_pack(&runtime.cfg.data_dir, &id)
            else {
                return Some(respond_json(
                    stream,
                    404,
                    &json!({"error": "playbook not found"}),
                ));
            };
            let body = serde_json::to_vec_pretty(&pack).unwrap_or_default();
            let headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                ("X-Ergon-Playbook-SHA256", digest),
                (
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                ),
            ];
            respond_bytes(
                stream,
                200,
                "application/json; charset=utf-8",
                &body,
                &headers,
            )
        }
        ("POST", "/api/playbooks/import") => {
            match routines::import_playbook_pack(&runtime.cfg.data_dir, &req.body) {
                Ok((routine, digest)) => {
                    let now_ms = records::now_millis();
                    let enriched = scheduler::enrich_routine_json(&routine, now_ms, &[]);
                    respond_json(
                        stream,
                        201,
                        &json!({
                            "routine": enriched,
                            "sha256": digest,
                            "verified": true,
                        }),
                    )
                }
                Err((status, msg)) => respond_json(stream, status, &json!({"error": msg})),
            }
        }
        ("POST", path)
            if (path.starts_with("/api/playbooks/") || path.starts_with("/api/routines/"))
                && path.ends_with("/preview") =>
        {
            let prefix = if path.starts_with("/api/playbooks/") {
                "/api/playbooks/"
            } else {
                "/api/routines/"
            };
            let raw_id = path
                .strip_prefix(prefix)
                .and_then(|r| r.strip_suffix("/preview"))
                .unwrap_or("")
                .trim();
            let id = percent_decode(raw_id).trim().to_string();
            if id.is_empty() || id.contains('/') || id.contains("..") {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "invalid routine id"}),
                ));
            }
            let idx = state.search_cache.get(
                &person.id,
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &runtime.cfg.memory_path,
            );
            let (_, ws_files) = idx.list_files(0, Some(500), "");
            let ws_paths: Vec<String> = ws_files
                .into_iter()
                .filter_map(|v| v["path"].as_str().map(str::to_string))
                .collect();
            match routines::preview_playbook_routine(
                &runtime.cfg.data_dir,
                &runtime.cfg.workspace_dir,
                &ws_paths,
                &id,
            ) {
                Some(preview) => respond_json(stream, 200, &preview),
                None => respond_json(stream, 404, &json!({"error": "routine not found"})),
            }
        }
        ("GET", "/api/decisions") => {
            let decisions = records::load_decisions(&runtime.cfg.data_dir);
            respond_json(stream, 200, &json!({"decisions": decisions}))
        }
        ("GET", "/api/offers") => {
            // Refresh first, so the inbox shows what the work log has
            // newly earned since the last look. The library is cloned
            // and the lock dropped before the scan does file I/O —
            // the mutex guards the in-memory items, nothing slower.
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            let offers = records::refresh_offers(&runtime.cfg, &suite, &lib);
            respond_json(stream, 200, &json!({"offers": offers}))
        }
        ("GET", "/api/brief") => {
            let lib = state.library.lock().unwrap().clone();
            let suite = library::load_suite(&runtime.cfg.data_dir);
            records::refresh_offers(&runtime.cfg, &suite, &lib);
            let brief = records::build_brief(&runtime.cfg, &person.name, &suite, &lib);
            respond_json(stream, 200, &brief)
        }
        ("GET", "/api/today") => {
            let mut today = records::build_today(&runtime.cfg);
            // Tripwire events from today join the stream — warnings and
            // up only (individual decoy probes stay in Settings → Security),
            // and never twice: an event already shown as a decision title
            // is not repeated.
            let date = today["date"].as_str().unwrap_or("").to_string();
            let mut decision_titles: Vec<String> = records::load_decisions(&runtime.cfg.data_dir)
                .iter()
                .filter_map(|d| d["summary"].as_str().map(|s| s.to_string()))
                .collect();
            if person.id != "default" {
                // Machine-level notices live in the default desk's inbox.
                decision_titles.extend(
                    records::load_decisions(&root_cfg.data_dir)
                        .iter()
                        .filter_map(|d| d["summary"].as_str().map(|s| s.to_string())),
                );
            }
            if let Some(events) = today["events"].as_array_mut() {
                for e in tripwire::events_for(&root_cfg.data_dir, &person.id, 200) {
                    let ts = e["ts"].as_u64().unwrap_or(0);
                    let sev = tripwire::Severity::from_label(e["severity"].as_str().unwrap_or(""));
                    let title = e["title"].as_str().unwrap_or("");
                    if sev >= tripwire::Severity::Warning
                        && records::date_string_local(ts) == date
                        && !decision_titles.iter().any(|t| t == title)
                    {
                        events.push(json!({
                            "ts": ts,
                            "kind": "security",
                            "title": e["title"],
                            "detail": e["detail"],
                            "severity": e["severity"],
                            "action": e["action"],
                        }));
                    }
                }
                events.sort_by_key(|e| std::cmp::Reverse(e["ts"].as_u64().unwrap_or(0)));
            }
            respond_json(stream, 200, &today)
        }
        ("GET", "/api/graph") => {
            // Cloned snapshots, locks dropped: build_graph reads
            // every person's files from disk and must not hold the
            // people/library locks across that I/O.
            let people_snapshot = state.people.read().unwrap().people.clone();
            let lib = state.library.lock().unwrap().clone();
            respond_json(
                stream,
                200,
                &records::build_graph(&people_snapshot, root_cfg, &lib),
            )
        }
        ("POST", path) if path.starts_with("/api/decisions/") => {
            let rest = path.trim_start_matches("/api/decisions/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return Some(respond_json(
                        stream,
                        404,
                        &json!({"error": "not found"}),
                    ))}
            };
            if action != "approve" && action != "deny" {
                return Some(respond_json(stream, 404, &json!({"error": "not found"})));
            }
            match records::resolve_decision(&runtime.cfg, &id, action == "approve") {
                Ok((decision, outcome)) => {
                    // Tripwire decisions act after the settle (blocking a
                    // site, quarantining a file, publishing a held skill
                    // need app state the records lock must not span).
                    let (decision, outcome) = settle_tripwire_decision(state, runtime, person, &decision, action == "approve")
                        .unwrap_or((decision, outcome));
                    // The outcome reports back into the thread the
                    // decision came from, so the record is complete
                    // where the work happened.
                    let tid = decision["thread_id"].as_str().unwrap_or("");
                    if !tid.is_empty() {
                        let mut store = runtime.store.lock().unwrap();
                        store.push(
                            tid,
                            "step",
                            &format!(
                                "Decision settled — {}: {}",
                                decision["summary"].as_str().unwrap_or(""),
                                outcome
                            ),
                        );
                        // The parked-delete note says the file is still
                        // there; once settled it must say what happened (ER1).
                        if let Some(path) = decision["path"].as_str().filter(|_| decision["kind"] == "delete") {
                            let settled = if action != "approve" {
                                format!("Note: {path} was kept because you declined the deletion.")
                            } else if outcome.starts_with("Approved and done") {
                                format!("Note: {path} was deleted after you approved it.")
                            } else if outcome.starts_with("Approved: ") {
                                format!("Note: {path} was already gone when you approved it.")
                            } else {
                                format!("Note: {path} was not deleted; see the decision settled above.")
                            };
                            store.settle_note(tid, &parked_note(path), &settled);
                        }
                    }
                    state.search_cache.invalidate(&person.id);
                    respond_json(
                        stream,
                        200,
                        &json!({"decision": decision, "outcome": outcome}),
                    )
                }
                Err(e) => respond_json(stream, 400, &json!({"error": e})),
            }
        }
        ("POST", path) if path.starts_with("/api/offers/") => {
            let rest = path.trim_start_matches("/api/offers/");
            let (id, action) = match rest.rsplit_once('/') {
                Some((i, a)) => (i.to_string(), a.to_string()),
                None => {
                    return Some(respond_json(
                        stream,
                        404,
                        &json!({"error": "not found"}),
                    ))}
            };
            match action.as_str() {
                "dismiss" => match records::set_offer_status(&runtime.cfg.data_dir, &id, "dismissed") {
                    Ok(offer) => respond_json(stream, 200, &json!({"offer": offer})),
                    Err(e) => respond_json(stream, 400, &json!({"error": e})),
                },
                "accept" => {
                    // Look the offer up first; acceptance DOES the
                    // thing offered (install the app / create the
                    // one-tap app from the repeated task), then marks
                    // the offer settled.
                    let offers = records::load_offers(&runtime.cfg.data_dir);
                    let Some(offer) = offers.iter().find(|o| o["id"].as_str() == Some(id.as_str())).cloned() else {
                        return Some(respond_json(stream, 404, &json!({"error": "offer not found"})));
                    };
                    if offer["status"].as_str() != Some("pending") {
                        return Some(respond_json(
                            stream,
                            400,
                            &json!({"error": "that offer was already settled"}),
                        ));
                    }
                    let kind = offer["kind"].as_str().unwrap_or("");
                    let mut made: Option<Value> = None;
                    if kind == "install_app" {
                        let item_id = offer["item_id"].as_str().unwrap_or("");
                        library::with_suite_lock(|| {
                            let mut suite = library::load_suite(&runtime.cfg.data_dir);
                            if suite.install(item_id) {
                                library::save_suite(&runtime.cfg.data_dir, &suite);
                            }
                        });
                        made = Some(json!({"installed": item_id}));
                    } else if kind == "repeat_task" {
                        let template = offer["template"].as_str().unwrap_or("");
                        let short: String = template.chars().take(48).collect();
                        let name = format!("One-tap: {short}");
                        let mut lib = state.library.lock().unwrap();
                        match lib.publish_app(
                            &name,
                            "Created from an offer: work this person kept asking for, as a one-tap app.",
                            template.to_string(),
                            Vec::new(),
                            &person.id,
                        ) {
                            Ok(item) => {
                                lib.save(&root_cfg.data_dir);
                                library::with_suite_lock(|| {
                                    let mut suite =
                                        library::load_suite(&runtime.cfg.data_dir);
                                    suite.install(&item.id);
                                    library::save_suite(&runtime.cfg.data_dir, &suite);
                                });
                                made = Some(item.to_json());
                            }
                            Err(e) => {
                                return Some(respond_json(stream, 400, &json!({"error": e})))}
                        }
                    }
                    match records::set_offer_status(&runtime.cfg.data_dir, &id, "accepted") {
                        Ok(offer) => respond_json(
                            stream,
                            200,
                            &json!({"offer": offer, "made": made}),
                        ),
                        Err(e) => respond_json(stream, 400, &json!({"error": e})),
                    }
                }
                _ => respond_json(stream, 404, &json!({"error": "not found"})),
            }
        }
        _ => return None,
    })
}

/// Routes live run status/SSE/cancel, apps execution, threads, and chat.
fn try_route_runs_and_chat(
    stream: &mut std::net::TcpStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
    ctx: &ToolCtx,
) -> Option<std::io::Result<()>> {
    let root_cfg = &state.cfg;
    let _ = root_cfg;
    Some(match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/runs/status") => {
            let active = state.live_runs.lock().unwrap().get(&person.id).cloned();
            match active {
                Some(run) => {
                    let elapsed_ms = records::now_millis().saturating_sub(run.started_ms);
                    respond_json(
                        stream,
                        200,
                        &json!({
                            "active": true,
                            "run_id": run.run_id,
                            "thread_id": run.thread_id,
                            "task": run.task,
                            "started_ms": run.started_ms,
                            "elapsed_ms": elapsed_ms,
                            "steps": run.steps,
                        }),
                    )
                }
                None => respond_json(
                    stream,
                    200,
                    &json!({
                        "active": false,
                        "run_id": "",
                        "thread_id": "",
                        "task": "",
                        "started_ms": 0,
                        "elapsed_ms": 0,
                        "steps": [],
                    }),
                ),
            }
        }
        ("GET", "/api/runs/stream") => {
            let thread_filter = query_param(&req.query, "thread_id");
            let once = query_param(&req.query, "once")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let active = state.live_runs.lock().unwrap().get(&person.id).cloned();
            let initial = match active {
                Some(run) => json!({
                    "active": true,
                    "run_id": run.run_id,
                    "thread_id": run.thread_id,
                    "task": run.task,
                    "started_ms": run.started_ms,
                    "elapsed_ms": records::now_millis().saturating_sub(run.started_ms),
                    "steps": run.steps,
                }),
                None => json!({
                    "active": false,
                    "run_id": "",
                    "thread_id": "",
                    "task": "",
                    "started_ms": 0,
                    "elapsed_ms": 0,
                    "steps": [],
                }),
            };
            sse::stream_sse(
                stream,
                &state.events,
                &person.id,
                thread_filter.as_deref(),
                &initial,
                once,
            )
        }
        ("POST", path) if path.starts_with("/api/runs/") && path.ends_with("/cancel") => {
            let id = path
                .strip_prefix("/api/runs/")
                .and_then(|r| r.strip_suffix("/cancel"))
                .unwrap_or("")
                .trim()
                .to_string();
            let found = {
                let map = state.live_runs.lock().unwrap();
                map.get(&person.id).and_then(|run| {
                    if id == "active" || id.is_empty() || run.run_id == id || run.thread_id == id {
                        run.cancel.store(true, Ordering::SeqCst);
                        Some((run.run_id.clone(), run.thread_id.clone()))
                    } else {
                        None
                    }
                })
            };
            match found {
                Some((run_id, thread_id)) => {
                    state.events.publish(
                        "run_cancelling",
                        &person.id,
                        &run_id,
                        &thread_id,
                        json!({"ts_ms": records::now_millis()}),
                    );
                    respond_json(
                        stream,
                        200,
                        &json!({"cancelled": true, "run_id": run_id, "thread_id": thread_id}),
                    )
                }
                None => respond_json(
                    stream,
                    404,
                    &json!({"error": "no matching active run"}),
                ),
            }
        }
        ("POST", path) if path.starts_with("/api/apps/") && path.ends_with("/run") => {
            // Single strip, not trim_*_matches (which strips
            // repeatedly and would resolve "/api/apps/foo/run/run"
            // to the app "foo").
            let id = path
                .strip_prefix("/api/apps/")
                .and_then(|rest| rest.strip_suffix("/run"))
                .unwrap_or("")
                .to_string();
            let item = {
                let lib = state.library.lock().unwrap();
                lib.get(&id)
            };
            let Some(item) = item else {
                return Some(respond_json(stream, 404, &json!({"error": "app not found"})));
            };
            if item.kind != "app" {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "that library item is not an app"}),
                ));
            }
            let suite = library::load_suite(&runtime.cfg.data_dir);
            if !suite.installed.contains(&item.id) {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "that app is not in this person's suite"}),
                ));
            }
            let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            // Every field the template actually uses must be filled
            // BEFORE substitution — checking afterwards cannot work:
            // substitution erases the evidence of an empty field and
            // the run would start with a hole in its instructions.
            for field in &item.fields {
                if item.template.contains(&format!("{{{field}}}")) {
                    let value = parsed["fields"][field.as_str()]
                        .as_str()
                        .unwrap_or("")
                        .trim();
                    if value.is_empty() {
                        return Some(respond_json(
                            stream,
                            400,
                            &json!({"error": "fill in every field before running the app"}),
                        ));
                    }
                }
            }
            // Fill the template's {field} slots from the run's inputs.
            let mut task = item.template.clone();
            for field in &item.fields {
                let value = parsed["fields"][field.as_str()]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                task = task.replace(&format!("{{{field}}}"), &value);
            }
            let cfg = effective_cfg(state, runtime);
            let thread_id = {
                let mut store = runtime.store.lock().unwrap();
                let tid = store.create_thread();
                store.push(&tid, "user", &format!("{} — {}", item.name, task));
                tid
            };
            if !cfg.model_configured() {
                runtime
                    .store
                    .lock()
                    .unwrap()
                    .push(&thread_id, "error", NO_KEY_ERROR);
                return Some(respond_json(
                    stream,
                    503,
                    &json!({"error": NO_KEY_ERROR, "thread_id": thread_id}),
                ));
            }
            let outcome = run_agent_live(
                &cfg,
                ctx,
                &runtime.store,
                &thread_id,
                &task,
                &[],
                Some((&state.live_runs, &state.events, &person.id)),
            );
            state.search_cache.invalidate(&person.id);
            match outcome {
                Ok(_outcome) => {
                    let store = runtime.store.lock().unwrap();
                    let v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    respond_json(stream, 200, &v)
                }
                Err(e) if e.starts_with(tripwire::HALT_PREFIX) => {
                    respond_safety_stop(stream, runtime, &thread_id, &e)
                }
                Err(e) => {
                    runtime.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", "/api/threads") => {
            let store = runtime.store.lock().unwrap();
            respond_json(stream, 200, &store.list_json())
        }
        ("POST", "/api/threads") => {
            let mut store = runtime.store.lock().unwrap();
            let id = store.create_thread();
            let v = store.thread_json(&id).unwrap_or_else(|| json!({"id": id}));
            respond_json(stream, 201, &v)
        }
        ("POST", "/api/chat") => {
            let parsed: Value = if req.body.iter().all(u8::is_ascii_whitespace) {
                Value::Null
            } else {
                match serde_json::from_slice(&req.body) {
                    Ok(v) => v,
                    Err(_) => {
                        return Some(respond_json(
                            stream,
                            400,
                            &json!({"error": "request body is not valid JSON"}),
                        ))}
                }
            };
            let message = parsed["message"].as_str().unwrap_or("").trim().to_string();
            if message.is_empty() {
                return Some(respond_json(
                    stream,
                    400,
                    &json!({"error": "message is required"}),
                ));
            }
            let cfg = effective_cfg(state, runtime);
            // The user's words are recorded FIRST, before any failure
            // path: a refused or failed run must never silently lose
            // the task the person typed (the no-key path used to drop
            // it, so the message vanished on reload).
            let (thread_id, history) = {
                let mut store = runtime.store.lock().unwrap();
                let requested = parsed["thread_id"].as_str().unwrap_or("");
                // A named thread must exist: opening a new one silently
                // would put the message somewhere the person never asked (EB3).
                if !requested.is_empty() && store.thread_json(requested).is_none() {
                    return Some(respond_json(stream, 404, &json!({"error": "no such thread"})));
                }
                let tid = store.ensure_thread(requested);
                let history = store.history_json(&tid);
                store.push(&tid, "user", &message);
                (tid, history)
            };
            if !cfg.model_configured() {
                runtime
                    .store
                    .lock()
                    .unwrap()
                    .push(&thread_id, "error", NO_KEY_ERROR);
                return Some(respond_json(
                    stream,
                    503,
                    &json!({"error": NO_KEY_ERROR, "thread_id": thread_id}),
                ));
            }
            let outcome = run_agent_live(
                &cfg,
                ctx,
                &runtime.store,
                &thread_id,
                &message,
                &history,
                Some((&state.live_runs, &state.events, &person.id)),
            );
            state.search_cache.invalidate(&person.id);
            match outcome {
                Ok(_outcome) => {
                    let store = runtime.store.lock().unwrap();
                    let v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    respond_json(stream, 200, &v)
                }
                Err(e) if e == "run cancelled by user" => {
                    let mut store = runtime.store.lock().unwrap();
                    store.push(&thread_id, "step", "Run cancelled by user.");
                    let mut v = store
                        .thread_json(&thread_id)
                        .unwrap_or_else(|| json!({"thread_id": thread_id}));
                    v["cancelled"] = json!(true);
                    respond_json(stream, 200, &v)
                }
                Err(e) if e.starts_with(tripwire::HALT_PREFIX) => {
                    respond_safety_stop(stream, runtime, &thread_id, &e)
                }
                Err(e) => {
                    runtime.store.lock().unwrap().push(&thread_id, "error", &e);
                    respond_json(stream, 500, &json!({"error": e, "thread_id": thread_id}))
                }
            }
        }
        ("GET", path) if path.starts_with("/api/threads/") => {
            let id = path.trim_start_matches("/api/threads/");
            let store = runtime.store.lock().unwrap();
            match store.thread_json(id) {
                Some(v) => respond_json(stream, 200, &v),
                None => respond_json(stream, 404, &json!({"error": "thread not found"})),
            }
        }
        _ => return None,
    })
}
