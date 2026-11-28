//! Embedded HTTP server, request parser, security gates, and API route handlers.

pub(in crate::server) use crate::tools::{atomic_write_bytes, parse_tool_tests, string_list};
pub(in crate::server) use crate::*;
pub(in crate::server) use std::io::{BufRead, BufReader, Read, Write};
pub(in crate::server) use std::net::TcpListener;
pub(in crate::server) use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
pub(in crate::server) use std::sync::{Arc, Condvar, Mutex, Once, RwLock};

// ---------------------------------------------------------------------------
// HTTP server (hand-rolled, std TcpListener)
// ---------------------------------------------------------------------------

/// One person's live state: their scoped config + their thread store.
pub struct PersonRuntime {
    pub cfg: Config,
    pub store: SharedStore,
}

pub(super) fn load_runtime(root: &Config, person: &Person, library: &library::Library) -> PersonRuntime {
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
    let _ = pipelines::recover_stale_runs_on_boot(&cfg.data_dir);
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
    /// Admin-only break-glass gate token (`ERGON_GATE_TOKEN`).
    /// When multi-user auth is enabled, this token acts strictly as an
    /// audited admin break-glass credential and never bypasses cross-user
    /// isolation for non-admin sessions. In legacy single-user loopback mode
    /// (zero accounts, no TLS), it continues to gate all requests as before.
    /// Every request must carry it via `X-Ergon-Gate`, `Authorization: Bearer`,
    /// or `X-Ergon-Token`.
    pub gate_token: Option<String>,
    /// Optional dedicated admin token (ERGON_ADMIN_TOKEN) for update management
    /// and administrative operations on gated/shared hosts.
    pub admin_token: Option<String>,
    /// Extra Host names accepted without a gate token (ERGON_ALLOWED_HOSTS).
    pub allowed_hosts: Vec<String>,
    /// Multi-user accounts, persisted sessions, optional OIDC, and RBAC store.
    pub auth: Arc<auth::AuthStore>,
    /// Optional HTTPS/TLS server configuration (rustls).
    pub tls: Option<tls::TlsConfig>,
    /// Public externally-reachable origin behind a reverse proxy (`ERGON_PUBLIC_URL` / `SERVICE_URL_ERGON`).
    pub public_url: Option<String>,
    /// Whether `X-Forwarded-Proto` / `X-Forwarded-Host` from a reverse proxy are trusted (`ERGON_TRUST_PROXY=1`).
    pub trust_proxy: bool,
    /// True when the listener is bound to a non-loopback address.
    pub non_loopback_bind: bool,
    pub people: Arc<RwLock<PeopleState>>,
    pub runtimes: Arc<RwLock<HashMap<String, Arc<PersonRuntime>>>>,
    /// In-flight runs per person for live step progress streaming (`GET /api/runs/status` & `/api/runs/stream`).
    pub live_runs: LiveRuns,
    /// Backup filename if corrupt `live_runs.json` was quarantined on boot.
    pub live_runs_recovered: Option<String>,
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
    /// Serializes scheduled/manual backups and full-instance restores.
    pub backup_lock: Arc<Mutex<()>>,
    /// Perimeter honeytraps: decoys, breadcrumbs, honeytoken use, bans.
    pub tripwire: Arc<tripwire::Perimeter>,
    /// Server startup instant for uptime reporting in diagnostics.
    pub started_at: std::time::Instant,
}

pub(super) static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);
pub(super) static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
pub(super) static SIGNAL_HANDLERS_INSTALLED: Once = Once::new();

/// Default bounded drain window after `SIGTERM`/`SIGINT` before exiting.
pub(super) const DEFAULT_SHUTDOWN_GRACE_SECS: u64 = 10;
pub(super) const MAX_SHUTDOWN_GRACE_SECS: u64 = 300;

/// Returns `true` once a graceful shutdown signal (`SIGTERM`/`SIGINT` on Unix,
/// console control event on Windows) or explicit shutdown request has arrived.
pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Acquire)
}

/// Request a graceful shutdown drain programmatically (useful in supervisors or tests).
pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

pub(super) fn parse_shutdown_grace_secs(raw: Option<&str>) -> u64 {
    raw.and_then(|s| s.trim().parse::<u64>().ok())
        .map(|s| s.min(MAX_SHUTDOWN_GRACE_SECS))
        .unwrap_or(DEFAULT_SHUTDOWN_GRACE_SECS)
}

/// Configured graceful shutdown drain window (`ERGON_SHUTDOWN_GRACE_SECS`, default 10s, max 300s).
pub fn shutdown_grace_secs() -> u64 {
    let raw = std::env::var("ERGON_SHUTDOWN_GRACE_SECS").ok();
    parse_shutdown_grace_secs(raw.as_deref())
}

/// Install OS signal handlers (`SIGTERM`/`SIGINT` on Unix, `SetConsoleCtrlHandler` on Windows)
/// so the server stops accepting new connections, drains in-flight requests/runs, and exits 0.
pub fn install_shutdown_handlers() {
    SIGNAL_HANDLERS_INSTALLED.call_once(|| {
        #[cfg(unix)]
        install_unix_signal_handlers();
        #[cfg(windows)]
        install_windows_signal_handlers();
    });
}

#[cfg(unix)]
pub(super) fn install_unix_signal_handlers() {
    extern "C" fn on_signal(_sig: libc::c_int) {
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
    }
    // SAFETY: `on_signal` only performs an async-signal-safe atomic store.
    // `sigaction` struct is zero-initialized and populated with a valid signal mask.
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = on_signal as *const () as usize;
        libc::sigemptyset(&mut sa.sa_mask);
        sa.sa_flags = 0;
        let _ = libc::sigaction(libc::SIGTERM, &sa, std::ptr::null_mut());
        let _ = libc::sigaction(libc::SIGINT, &sa, std::ptr::null_mut());
    }
}

#[cfg(windows)]
pub(super) fn install_windows_signal_handlers() {
    type HandlerRoutine = unsafe extern "system" fn(u32) -> i32;
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCtrlHandler(handler: Option<HandlerRoutine>, add: i32) -> i32;
    }
    unsafe extern "system" fn on_ctrl(ctrl_type: u32) -> i32 {
        // CTRL_C_EVENT=0, CTRL_BREAK_EVENT=1, CTRL_CLOSE_EVENT=2,
        // CTRL_LOGOFF_EVENT=5, CTRL_SHUTDOWN_EVENT=6
        match ctrl_type {
            0 | 1 | 2 | 5 | 6 => {
                SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
                1
            }
            _ => 0,
        }
    }
    // SAFETY: `on_ctrl` is a static `extern "system"` callback that only flips an AtomicBool.
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(on_ctrl), 1);
    }
}

/// Session ids are the only credential a browser presents, so they
/// must be unguessable: 128 bits from the OS CSPRNG (`ring::rand::SystemRandom`)
/// on both Unix and Windows, with zero predictable timestamp fallback.
pub(super) fn new_session_id() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    let n = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut bytes = [0u8; 16];
    SystemRandom::new()
        .fill(&mut bytes)
        .expect("OS CSPRNG unavailable for session id generation");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("s{hex}-{n}")
}

/// Whether this session id is one the server actually issued. A
/// client-chosen id is never honoured (session fixation): unknown ids
/// are treated as "no session" and replaced with a fresh server id.
/// Insert a session, bounding the map: sessions are memory-only and
/// re-issued transparently, so past a generous cap the map is simply
/// cleared rather than growing for the life of the process.
pub(super) fn insert_session(state: &AppState, sid: String, person_id: String) {
    let mut sessions = state.sessions.write().unwrap();
    if sessions.len() > 10_000 {
        sessions.clear();
    }
    sessions.insert(sid, person_id);
}

pub(super) fn session_known(state: &AppState, session: &Option<String>) -> bool {
    match session {
        Some(sid) => state.sessions.read().unwrap().contains_key(sid),
        None => false,
    }
}

/// The person this request acts for: the session's person when the
/// caller presented a known session cookie, else the global active.
pub(super) fn effective_person(state: &AppState, session: &Option<String>) -> Person {
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

pub(super) fn runtime_for(state: &AppState, person: &Person) -> Arc<PersonRuntime> {
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
    serve_env(listener, cfg, None, false)
}

/// What the `ergon` binary runs: `serve` plus the periodic check for a newer
/// GitHub release (see `update`). Tests use `serve`, which never calls out.
pub fn serve_with_updates(listener: TcpListener, cfg: Config) -> std::io::Result<()> {
    install_shutdown_handlers();
    serve_env(listener, cfg, None, true)
}

/// Serve with an explicit TLS configuration (HTTPS via rustls).
pub fn serve_with_tls(
    listener: TcpListener,
    cfg: Config,
    tls_config: Option<tls::TlsConfig>,
    background_updates: bool,
) -> std::io::Result<()> {
    serve_env(listener, cfg, tls_config, background_updates)
}

pub(super) fn serve_env(
    listener: TcpListener,
    cfg: Config,
    tls_override: Option<tls::TlsConfig>,
    background_updates: bool,
) -> std::io::Result<()> {
    let gate = std::env::var("ERGON_GATE_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let admin = std::env::var("ERGON_ADMIN_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let hosts = allowed_hosts_from(&std::env::var("ERGON_ALLOWED_HOSTS").unwrap_or_default());
    let tls_cfg = match tls_override {
        Some(t) => Some(t),
        None => tls::resolve_tls(&cfg.data_dir, false, None, None, &hosts)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?,
    };
    serve_inner(listener, cfg, gate, admin, hosts, tls_cfg, None, background_updates)
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
    serve_inner(listener, cfg, gate_token, None, allowed_hosts, None, None, false)
}

/// Serves with explicit gate token, optional admin token, allowed hosts, and custom updater
/// (used by tests and embedded hosting layers).
pub fn serve_with_updater(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    admin_token: Option<String>,
    allowed_hosts: Vec<String>,
    updater: Arc<update::Updater>,
) -> std::io::Result<()> {
    serve_inner(listener, cfg, gate_token, admin_token, allowed_hosts, None, Some(updater), false)
}


mod background;
mod handoff;
mod http;
mod routes_company;
mod routes_files;
mod routes_runs;
mod routes_system;
mod routes_workflows;

pub(in crate::server) use background::*;
pub(in crate::server) use handoff::*;
pub(in crate::server) use http::*;
pub(in crate::server) use routes_runs::{execute_resumed_run, pipeline_exec_env};
use routes_company::try_route_company_work;
use routes_files::try_route_files_and_audit;
use routes_runs::try_route_runs_and_chat;
use routes_system::try_route_system;
use routes_workflows::try_route_knowledge_and_routines;

pub use handoff::build_handoff_summary_markdown;
pub use http::parked_note;
#[cfg(test)]
pub(crate) use http::read_request;
pub(crate) use http::{host_allowed, refusal, Request};

/// `background_updates` starts the periodic release check; only the real
/// binary does (tests must not call out to GitHub).
pub(super) fn resolve_public_url_from_env() -> Option<(String, String)> {
    let raw = std::env::var("ERGON_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("SERVICE_URL_ERGON")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })?;
    crate::parse_public_url(&raw).map(|(origin, host, _scheme)| (origin, host))
}

fn env_truthy_server(key: &str) -> bool {
    std::env::var(key)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

#[allow(clippy::too_many_arguments)] // internal entry point shared by the serve* wrappers
pub(super) fn serve_inner(
    listener: TcpListener,
    cfg: Config,
    gate_token: Option<String>,
    admin_token: Option<String>,
    mut allowed_hosts: Vec<String>,
    tls: Option<tls::TlsConfig>,
    custom_updater: Option<Arc<update::Updater>>,
    background_updates: bool,
) -> std::io::Result<()> {
    let public_parsed = resolve_public_url_from_env();
    let public_url = public_parsed.as_ref().map(|(origin, _)| origin.clone());
    if let Some((_, ref pub_host)) = public_parsed {
        if !allowed_hosts.iter().any(|h| h.eq_ignore_ascii_case(pub_host)) {
            allowed_hosts.push(pub_host.clone());
        }
    }
    let trust_proxy = env_truthy_server("ERGON_TRUST_PROXY");
    cfg.ensure_dirs()?;
    logging::init_if_unset(&cfg.data_dir);
    // The agent may never fetch this server's own API (egress guard).
    let non_loopback_bind = listener
        .local_addr()
        .map(|addr| {
            tripwire::register_self_port(addr.port());
            !addr.ip().is_loopback()
        })
        .unwrap_or(false);
    let perimeter = tripwire::Perimeter::new(&cfg.data_dir);
    let gate_token_is_none = gate_token.is_none();
    let updater = custom_updater.unwrap_or_else(|| {
        update::Updater::from_env_in_dir(
            "ERGON",
            "ergon",
            env!("CARGO_PKG_VERSION"),
            "cayleb-james2008/ergon",
            gate_token_is_none,
            Some(&cfg.data_dir),
        )
    });
    let people = Arc::new(RwLock::new(PeopleState::load(&cfg)));
    let auth_store = Arc::new(auth::AuthStore::load(&cfg.data_dir));
    if let Err(e) = auth_store.bootstrap_admin_from_env(&people, &cfg) {
        crate::logging::error("ergon::server", &format!("ergon: ERGON_ADMIN_USER bootstrap error: {e}"));
    }
    let library = library::Library::load(&cfg.data_dir);
    let auto_detected = cfg.api_key.is_none() && cfg.model_configured();
    let boot_model_configured =
        crate::model_configured_for(&cfg.model_base_url, cfg.api_key.is_some());
    let (boot_recovered_runs, live_runs_recovered) =
        live_runs::recover_stale_runs_on_boot(&cfg.data_dir, boot_model_configured);
    let model_cfg = Arc::new(Mutex::new(LiveModelConfig {
        model_base_url: cfg.model_base_url.clone(),
        model_id: cfg.model_id.clone(),
        api_key: cfg.api_key.clone(),
        auto_detected,
    }));
    let scheduler_disabled = tripwire::scheduler_disabled_override(&cfg.data_dir)
        .unwrap_or_else(scheduler::is_scheduler_disabled_by_env);
    let state = Arc::new(AppState {
        cfg,
        model_cfg,
        library: Arc::new(Mutex::new(library)),
        gate_token,
        admin_token,
        allowed_hosts,
        auth: auth_store,
        tls,
        public_url,
        trust_proxy,
        non_loopback_bind,
        people,
        runtimes: Arc::new(RwLock::new(HashMap::new())),
        live_runs: Arc::new(Mutex::new(HashMap::new())),
        live_runs_recovered,
        events: sse::RunEventBus::new(),
        search_cache: search::SearchCache::new(),
        sessions: Arc::new(RwLock::new(HashMap::new())),
        updater,
        scheduler_enabled: !scheduler_disabled,
        running_routines: Arc::new(Mutex::new(std::collections::HashSet::new())),
        backup_lock: Arc::new(Mutex::new(())),
        tripwire: perimeter,
        started_at: std::time::Instant::now(),
    });
    // Recover any stale "running" pipeline runs left behind by a prior server exit.
    {
        let ppl = state.people.read().unwrap().people.clone();
        for p in &ppl {
            let p_cfg = state.cfg.person_config(p);
            let _ = pipelines::recover_stale_runs_on_boot(&p_cfg.data_dir);
        }
    }
    // Publish boot-recovery events for interrupted ad-hoc chat runs and optionally
    // auto-resume them when a model is configured and auto-resume is enabled.
    if !boot_recovered_runs.is_empty() {
        let auto_resume = boot_model_configured && live_runs::auto_resume_enabled();
        let ppl = state.people.read().unwrap().people.clone();
        let mut by_person: HashMap<String, Vec<(Person, live_runs::PersistedLiveRun)>> =
            HashMap::new();
        for run in boot_recovered_runs {
            let eligible_auto =
                auto_resume && run.resume_count < live_runs::MAX_AUTO_RESUMES;
            state.events.publish(
                "run_recovered",
                &run.person_id,
                &run.run_id,
                &run.thread_id,
                json!({
                    "task": truncate(&run.task, 240),
                    "steps_count": run.steps.len(),
                    "status": "interrupted",
                    "auto_resume": eligible_auto,
                    "can_resume": boot_model_configured,
                    "ts_ms": records::now_millis(),
                }),
            );
            if eligible_auto {
                if let Some(person) = ppl.iter().find(|p| p.id == run.person_id).cloned() {
                    if let Ok(claimed) = live_runs::claim_for_resume(
                        &state.cfg.data_dir,
                        &person.id,
                        Some(&run.run_id),
                        None,
                        false,
                    ) {
                        by_person
                            .entry(person.id.clone())
                            .or_default()
                            .push((person, claimed));
                    }
                }
            }
        }
        for (_, queue) in by_person {
            let st_bg = state.clone();
            let _ = std::thread::Builder::new()
                .name("ergon-boot-resume".to_string())
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    for (person, claimed) in queue {
                        let _ = execute_resumed_run(&st_bg, &person, &claimed);
                    }
                });
        }
    }
    if background_updates {
        state.updater.start_background();
        crate::plugin_registry::start_background_sync(state.cfg.data_dir.clone());
    }
    if state.scheduler_enabled {
        start_scheduler_thread(&state);
        start_folder_watch_thread(&state);
        start_imap_poll_thread(&state);
        start_backup_scheduler_thread(&state);
    }
    const MAX_ACTIVE_CONNS: usize = 128;
    const WARM_WORKER_FLOOR: usize = 2;
    let active_conns = Arc::new(AtomicUsize::new(0));
    struct ConnGuard(Arc<AtomicUsize>);
    impl Drop for ConnGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }
    struct PoolInner {
        queue: VecDeque<(std::net::TcpStream, ConnGuard)>,
        idle_workers: usize,
        starting_workers: usize,
        total_workers: usize,
        shutting_down: bool,
    }
    let pool = Arc::new((
        Mutex::new(PoolInner {
            queue: VecDeque::with_capacity(64),
            idle_workers: 0,
            starting_workers: 0,
            total_workers: 0,
            shutting_down: false,
        }),
        Condvar::new(),
    ));

    listener.set_nonblocking(true)?;

    while !shutdown_requested() {
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Ensure accepted client sockets use blocking I/O with explicit
                // per-phase timeouts even on OSes where accept inherits O_NONBLOCK.
                let _ = stream.set_nonblocking(false);
                let prev = active_conns.fetch_add(1, Ordering::SeqCst);
                if prev >= MAX_ACTIVE_CONNS {
                    active_conns.fetch_sub(1, Ordering::SeqCst);
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
                if inner.queue.len() > inner.idle_workers + inner.starting_workers
                    && inner.total_workers < MAX_ACTIVE_CONNS
                {
                    inner.total_workers += 1;
                    inner.starting_workers += 1;
                    let pool_w = pool.clone();
                    let state_w = state.clone();
                    if std::thread::Builder::new()
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
                            let mut first_iter = true;
                            loop {
                                let job = {
                                    let mut g = l.lock().unwrap_or_else(|p| p.into_inner());
                                    if first_iter {
                                        g.starting_workers = g.starting_workers.saturating_sub(1);
                                        first_iter = false;
                                    }
                                    loop {
                                        if let Some(item) = g.queue.pop_front() {
                                            break Some(item);
                                        }
                                        if g.shutting_down {
                                            break None;
                                        }
                                        g.idle_workers += 1;
                                        let (next_g, wait_res) = cv
                                            .wait_timeout(g, std::time::Duration::from_secs(10))
                                            .unwrap_or_else(|p| p.into_inner());
                                        g = next_g;
                                        g.idle_workers = g.idle_workers.saturating_sub(1);
                                        if g.queue.is_empty()
                                            && (g.shutting_down
                                                || (wait_res.timed_out()
                                                    && g.total_workers > WARM_WORKER_FLOOR))
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
                        })
                        .is_err()
                    {
                        inner.total_workers = inner.total_workers.saturating_sub(1);
                        inner.starting_workers = inner.starting_workers.saturating_sub(1);
                    }
                }
                cvar.notify_one();
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                if shutdown_requested() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    // Stop accepting new connections immediately and wake idle SSE / pool threads.
    drop(listener);
    state.events.wake_all();
    {
        let (lock, cvar) = &*pool;
        let mut inner = lock.lock().unwrap_or_else(|p| p.into_inner());
        inner.shutting_down = true;
        cvar.notify_all();
    }

    // Drain in-flight HTTP requests, agent runs, and running routines up to the
    // configured grace window before exiting cleanly with status 0.
    let grace = std::time::Duration::from_secs(shutdown_grace_secs());
    let deadline = std::time::Instant::now() + grace;
    let mut logged_drain = false;
    loop {
        let conns = active_conns.load(Ordering::SeqCst);
        let runs = RUNS_IN_FLIGHT.load(Ordering::SeqCst);
        let routines = state
            .running_routines
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len();
        if conns == 0 && runs == 0 && routines == 0 {
            break;
        }
        if !logged_drain {
            eprintln!(
                "ergon: shutdown requested, draining {conns} active connection(s), {runs} in-flight run(s), {routines} routine(s) (grace {}s)...",
                grace.as_secs()
            );
            logged_drain = true;
        }
        if std::time::Instant::now() >= deadline {
            eprintln!(
                "ergon: shutdown grace window ({}s) elapsed with {conns} active connection(s), {runs} in-flight run(s), {routines} routine(s); exiting",
                grace.as_secs()
            );
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    Ok(())
}


pub(super) fn handle_connection(mut stream: std::net::TcpStream, state: &Arc<AppState>) -> std::io::Result<()> {
    let _ = stream.set_nodelay(true);
    // A banned scanner is dropped before one byte of its request is read
    // (never this computer: loopback is never banned).
    let peer = stream.peer_addr().ok().map(|a| a.ip());
    if peer.map(|ip| state.tripwire.is_banned(ip)).unwrap_or(false) {
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return Ok(());
    }
    struct TlsResetGuard;
    impl Drop for TlsResetGuard {
        fn drop(&mut self) {
            TLS_CONN.set(false);
        }
    }
    let _tls_reset = TlsResetGuard;
    let conn_stream = if let Some(ref tls_cfg) = state.tls {
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));
        let mut first = [0u8; 1];
        match stream.peek(&mut first) {
            Ok(0) | Err(_) => return Ok(()),
            Ok(_) if first[0] != 0x16 => {
                // Plaintext HTTP hit a TLS port: reject cleanly without serving any app data.
                let body = r#"{"error":"HTTPS is required on this port — reconnect using https://"}"#;
                let resp = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return Ok(());
            }
            Ok(_) => {}
        }
        let Ok(tls_conn) = rustls::ServerConnection::new(tls_cfg.server_config.clone()) else {
            return Ok(());
        };
        TLS_CONN.set(true);
        ConnStream::Tls(Box::new(rustls::StreamOwned::new(tls_conn, stream)))
    } else {
        TLS_CONN.set(false);
        ConnStream::Plain(stream)
    };

    let mut reader = BufReader::with_capacity(2048, conn_stream);
    const MAX_KEEP_ALIVE_REQUESTS: usize = 10_000;
    for req_idx in 0..MAX_KEEP_ALIVE_REQUESTS {
        let initial_wait = if req_idx == 0 {
            std::time::Duration::from_secs(30)
        } else {
            std::time::Duration::from_secs(2)
        };
        let req = match read_request_from(&mut reader, initial_wait, req_idx > 0) {
            Some(r) => r,
            None => break,
        };
        let client = state.tripwire.client_ip(peer, req.forwarded_for.as_deref());
        if client != peer && client.map(|ip| state.tripwire.is_banned(ip)).unwrap_or(false) {
            reader.get_ref().shutdown_both();
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
        // Inbound webhook endpoint: exempt from browser Origin/CSRF checks,
        // authenticated strictly via X-Ergon-Signature or X-Ergon-Hook-Token.
        if req.method == "POST" && req.path.starts_with("/api/hooks/") {
            KEEP_ALIVE_CONN.set(false);
            return handle_webhook_hook_request(reader.get_mut(), state, &req);
        }
        let root_cfg = &state.cfg;
        let auth_enforced = state
            .auth
            .is_enforced(state.tls.is_some(), state.non_loopback_bind);
        let raw_auth_token = auth::extract_bearer_token(req.authorization.as_deref())
            .or(req.session.as_deref());
        let presented_api_key = req
            .api_key_header
            .as_deref()
            .or_else(|| {
                auth::extract_bearer_token(req.authorization.as_deref())
                    .filter(|t| t.starts_with(auth::API_KEY_PREFIX))
            });
        let mut api_key_verified = false;
        let api_key_principal = if let Some(raw_key) = presented_api_key {
            match auth::authenticate_api_key(&state.cfg.data_dir, raw_key, &state.people) {
                Some(p) => {
                    api_key_verified = true;
                    Some(p)
                }
                None => {
                    KEEP_ALIVE_CONN.set(false);
                    return respond_json(
                        reader.get_mut(),
                        401,
                        &json!({"error": "invalid or expired API key"}),
                    );
                }
            }
        } else {
            None
        };
        let session_principal = if api_key_principal.is_some() {
            None
        } else {
            raw_auth_token.and_then(|t| state.auth.validate_session_token(t))
        };

        let mut break_glass_verified = false;
        let principal: Option<auth::AuthPrincipal> = if !auth_enforced {
            // Classic single-user loopback dev/test mode (0 accounts, no TLS,
            // no ERGON_AUTH_REQUIRED): preserve legacy gate_token check.
            // Hosting-layer gate: when a gate token is configured, every request
            // must present a valid gate or admin credential (`X-Ergon-Gate`,
            // `Authorization: Bearer`, or `X-Ergon-Token`).
            if state.gate_token.is_some() {
                if !gate_authenticated(state, &req) {
                    state.tripwire.note_refusal("gate", &view, client);
                    KEEP_ALIVE_CONN.set(false);
                    return respond_json(
                        reader.get_mut(),
                        401,
                        &json!({"error": "this Ergon is gated by its hosting layer — the gate header is missing or wrong"}),
                    );
                }
                break_glass_verified = true;
            }
            api_key_principal.or(session_principal)
        } else {
            // Multi-user / TLS / non-loopback auth-enforced mode:
            // ERGON_GATE_TOKEN is an audited admin-only break-glass token and
            // NEVER bypasses cross-user isolation for a non-admin session.
            if let Some(presented) = req.gate.as_deref() {
                let matched = state
                    .gate_token
                    .as_deref()
                    .is_some_and(|exp| tripwire::ct_eq(presented.as_bytes(), exp.as_bytes()));
                if !matched {
                    state.tripwire.note_refusal("gate", &view, client);
                    KEEP_ALIVE_CONN.set(false);
                    return respond_json(
                        reader.get_mut(),
                        401,
                        &json!({"error": "invalid break-glass gate token"}),
                    );
                }
                break_glass_verified = true;
            }
            if let Some(key_p) = api_key_principal {
                Some(key_p)
            } else if let Some(user_p) = session_principal {
                // Never allow X-Ergon-Gate to elevate a non-admin user session.
                Some(user_p)
            } else if break_glass_verified {
                auth::record_break_glass_audit(&state.cfg.data_dir, &req.method, &req.path, client);
                let active = state.people.read().unwrap().active_person();
                Some(auth::AuthPrincipal {
                    user_id: "break-glass".to_string(),
                    username: "break-glass".to_string(),
                    display_name: "Admin Break-Glass".to_string(),
                    email: None,
                    role: auth::Role::Admin,
                    home_person_id: active.id.clone(),
                    active_person_id: active.id,
                    auth_method: "gate_token".to_string(),
                    is_break_glass: true,
                    session_token_hash: None,
                    api_key_scopes: None,
                })
            } else {
                None
            }
        };

        let host_gate_exempt = if !auth_enforced {
            state.gate_token.is_some() || api_key_verified
        } else {
            break_glass_verified || api_key_verified
        };
        if let Some((status, message)) = refusal(&req, host_gate_exempt, &state.allowed_hosts) {
            // RE1: a page on another site (CSRF) or a rebound DNS name tried
            // to drive this desk. Refused as before — and now the person hears of it.
            let kind = if message.starts_with("requests must address") { "host" } else { "cross-site" };
            state.tripwire.note_refusal(kind, &view, client);
            KEEP_ALIVE_CONN.set(false);
            return respond_err(reader.get_mut(), status, message);
        }

        // Centralized authentication & RBAC guard for ALL routes.
        if auth_enforced {
            let access = auth::route_access(&req.method, &req.path);
            let is_inbound_mcp = req.method == "POST" && req.path == "/api/mcp/inbound";
            let mcp_token_bypass =
                is_inbound_mcp && mcp_inbound_bearer_valid(req.authorization.as_deref());
            match access {
                auth::RouteAccess::Public => {}
                auth::RouteAccess::PublicSetupOnly => {
                    if state.auth.has_users() {
                        KEEP_ALIVE_CONN.set(false);
                        return respond_err(
                            reader.get_mut(),
                            403,
                            "initial admin setup already completed",
                        );
                    }
                }
                _ if mcp_token_bypass => {}
                other => {
                    let Some(ref p) = principal else {
                        KEEP_ALIVE_CONN.set(false);
                        return respond_json(
                            reader.get_mut(),
                            401,
                            &json!({
                                "error": "authentication required",
                                "auth_required": true,
                                "needs_setup": !state.auth.has_users(),
                            }),
                        );
                    };
                    if is_inbound_mcp && p.api_key_scopes.is_some() {
                        if !p.has_scope("mcp") {
                            KEEP_ALIVE_CONN.set(false);
                            return respond_err(
                                reader.get_mut(),
                                403,
                                "forbidden: API key lacks required scope 'mcp'",
                            );
                        }
                    } else if !p.role.satisfies(other) {
                        KEEP_ALIVE_CONN.set(false);
                        return respond_err(
                            reader.get_mut(),
                            403,
                            &auth::access_denied_reason(other, p.role),
                        );
                    }
                }
            }
        }
        if let Some(ref p) = principal {
            if let Err(msg) = enforce_api_key_scope(p, &req.method, &req.path) {
                KEEP_ALIVE_CONN.set(false);
                return respond_err(reader.get_mut(), 403, &msg);
            }
        }

        let is_sse = req.path == "/api/runs/stream";
        KEEP_ALIVE_CONN.set(req.keep_alive && !is_sse && !shutdown_requested());
        // Resolve the person strictly from the authenticated principal when auth
        // is enforced (non-admins are pinned to their own home workspace).
        let person = if let Some(ref p) = principal {
            person_by_id(state, &p.active_person_id)
        } else if !auth_enforced {
            effective_person(state, &req.session)
        } else {
            state.people.read().unwrap().active_person()
        };
        let runtime = runtime_for(state, &person);
        let stream = reader.get_mut();
        let res = if let Some(res) = try_route_system(
            stream,
            state,
            &req,
            &person,
            &runtime,
            principal.as_ref(),
            auth_enforced,
            client,
        ) {
            res
        } else if let Some(res) = try_route_files_and_audit(stream, state, &req, &person, &runtime) {
            res
        } else if let Some(res) = try_route_knowledge_and_routines(
            stream,
            state,
            &req,
            &person,
            &runtime,
            principal.as_ref(),
            auth_enforced,
        ) {
            res
        } else if let Some(res) = try_route_company_work(
            stream,
            state,
            &req,
            &person,
            &runtime,
            principal.as_ref(),
            auth_enforced,
        ) {
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
                respond_err(stream, 404, "not found")
            }
        };
        let continue_loop = KEEP_ALIVE_CONN.get() && !is_sse && res.is_ok() && !shutdown_requested();
        KEEP_ALIVE_CONN.set(false);
        res?;
        if !continue_loop {
            break;
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    pub(in crate::server) use std::collections::HashSet;

    #[test]
    fn new_session_id_uses_csprng_with_32_hex_high_entropy_and_uniqueness() {
        let mut full_ids = HashSet::new();
        let mut random_hexes = HashSet::new();
        let mut nibble_counts = [0usize; 16];
        let mut prev_counter: Option<u64> = None;

        for _ in 0..256 {
            let sid = new_session_id();
            assert!(
                sid.starts_with('s'),
                "session id must start with 's': {sid}"
            );
            let rest = &sid[1..];
            let (hex, suffix) = rest
                .split_once('-')
                .unwrap_or_else(|| panic!("session id missing counter separator: {sid}"));
            assert_eq!(
                hex.len(),
                32,
                "session id random payload must be 32 hex chars (128-bit CSPRNG): {sid}"
            );
            assert!(
                hex.bytes()
                    .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
                "session id payload must be lowercase hex: {sid}"
            );
            let counter: u64 = suffix
                .parse()
                .unwrap_or_else(|e| panic!("invalid counter suffix in {sid}: {e}"));
            if let Some(prev) = prev_counter {
                assert!(counter > prev, "counter must strictly increase");
            }
            prev_counter = Some(counter);

            assert!(
                random_hexes.insert(hex.to_string()),
                "duplicate 128-bit random payload generated"
            );
            for b in hex.bytes() {
                let idx = match b {
                    b'0'..=b'9' => (b - b'0') as usize,
                    b'a'..=b'f' => (b - b'a' + 10) as usize,
                    _ => unreachable!(),
                };
                nibble_counts[idx] += 1;
            }
            assert!(full_ids.insert(sid), "duplicate session id generated");
        }

        // Across 256 * 32 = 8192 nibbles, all 16 hex digits should appear generously.
        for (digit, count) in nibble_counts.iter().enumerate() {
            assert!(
                *count >= 250,
                "suspiciously low entropy for hex digit {digit:x}: count={count}"
            );
        }
    }

    #[test]
    fn shutdown_grace_secs_parses_bounds_and_defaults() {
        assert_eq!(parse_shutdown_grace_secs(None), DEFAULT_SHUTDOWN_GRACE_SECS);
        assert_eq!(parse_shutdown_grace_secs(Some("")), DEFAULT_SHUTDOWN_GRACE_SECS);
        assert_eq!(
            parse_shutdown_grace_secs(Some("not-a-num")),
            DEFAULT_SHUTDOWN_GRACE_SECS
        );
        assert_eq!(parse_shutdown_grace_secs(Some("0")), 0);
        assert_eq!(parse_shutdown_grace_secs(Some("15")), 15);
        assert_eq!(parse_shutdown_grace_secs(Some(" 25 ")), 25);
        assert_eq!(parse_shutdown_grace_secs(Some("9999")), MAX_SHUTDOWN_GRACE_SECS);
    }
}
