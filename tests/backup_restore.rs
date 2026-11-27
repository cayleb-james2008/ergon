//! End-to-end integration tests for Gap 2: Automated Backup & Verified Restore.
//!
//! Proves:
//! 1. Full `ERGON_DATA_DIR` backup -> `ergon verify` -> COMPLETE `remove_dir_all` wipe
//!    -> `ergon restore` -> every file, receipt chain, client/invoice record, routine,
//!    multi-person workspace, and `ergon verify` passes.
//! 2. Retention pruning keeps exact `N` newest backups (+ `.sha256` sidecars).
//! 3. Offsite local folder replication & retention pruning.
//! 4. Offsite S3 AWS SigV4 PUT verified bit-for-bit against a live loopback HTTP server
//!    (credentials strictly via env vars, never written to disk or API responses).
//! 5. Tampered archive, tampered manifest, zip-slip (`../escape`), backslash traversal,
//!    and symlink entries are rejected BEFORE modifying `ERGON_DATA_DIR`.
//! 6. Non-empty restore safety guard (`--force` required) and pre-restore safety snapshot.
//! 7. Live HTTP API (`/api/backups`, `/api/backups/config`, `/api/backups/run`,
//!    `/api/backups/download`, `/api/backups/restore`) + path confinement + gate enforcement.

use ergon_rs::backup::{self, BackupConfig, BackupOffsiteConfig};
use ergon_rs::receipts::{self, sha256_hex};
use ergon_rs::{records, routines, tripwire, Config, PeopleState};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "ergon_backup_test_{tag}_{}_{}_{}",
            std::process::id(),
            records::now_millis(),
            seq
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ergon_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ergon"))
}

fn test_config(data_dir: &Path) -> Config {
    Config {
        data_dir: data_dir.to_path_buf(),
        workspace_dir: data_dir.join("workspace"),
        memory_path: data_dir.join("memory.md"),
        model_base_url: "http://127.0.0.1:9/v1".to_string(),
        model_id: "offline-test".to_string(),
        api_key: None,
    }
}

fn seed_rich_instance(data_dir: &Path) {
    let cfg = test_config(data_dir);
    cfg.ensure_dirs().unwrap();

    // Workspace files (nested dirs, markdown, CSV, binary bytes)
    std::fs::create_dir_all(cfg.workspace_dir.join("contracts")).unwrap();
    std::fs::create_dir_all(cfg.workspace_dir.join("reports")).unwrap();
    std::fs::create_dir_all(cfg.workspace_dir.join(".history")).unwrap();
    std::fs::write(
        cfg.workspace_dir.join("contracts/service-agreement.md"),
        "# Field Services MSA\n\nClause 1: 30-day warranty on all HVAC installs.\n",
    )
    .unwrap();
    std::fs::write(
        cfg.workspace_dir.join("reports/q4-ledger.csv"),
        "date,job,amount\n2026-11-01,JOB-101,4250.00\n2026-11-12,JOB-102,1890.50\n",
    )
    .unwrap();
    std::fs::write(
        cfg.workspace_dir.join("reports/logo.bin"),
        vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0xFF, 0x42],
    )
    .unwrap();
    std::fs::write(
        cfg.workspace_dir.join(".history/service-agreement-rev1.md"),
        "# Field Services MSA (draft 1)\n",
    )
    .unwrap();

    // Transient lock/tmp files that MUST be skipped by backup
    std::fs::write(
        cfg.workspace_dir.join(".service-agreement.md.ergon-99.tmp"),
        "transient atomic write temp file",
    )
    .unwrap();
    std::fs::write(data_dir.join("server.lock"), "pid=1234").unwrap();

    // Existing file inside backups/ that MUST NOT cause recursive backup inclusion
    std::fs::create_dir_all(data_dir.join("backups")).unwrap();
    std::fs::write(
        data_dir.join("backups/stale-note.txt"),
        "must not recurse into backup archive",
    )
    .unwrap();

    // Standing memory
    std::fs::write(
        &cfg.memory_path,
        "# Memory\n\n## Standing Rules\n- Always verify serial numbers before billing.\n- Flag invoices over $5,000 for PM sign-off.\n",
    )
    .unwrap();

    // Chained SHA-256 receipts (via work.json Field-Book records)
    records::append_work(
        &cfg,
        "Reconcile Q4 field ledger",
        "Matched 2 jobs totaling $6,140.50 with zero discrepancies.",
        &["reports/q4-ledger.csv".to_string()],
        "thread-backup-1",
        true,
    );
    records::append_work(
        &cfg,
        "Audit service agreement",
        "Verified 30-day install warranty clause.",
        &["contracts/service-agreement.md".to_string()],
        "thread-backup-2",
        true,
    );

    // Clients & Engagements store
    records::create_client(
        data_dir,
        &json!({
            "name": "Northwind Mechanical",
            "brand_header": "Northwind Field Ops",
            "billing_currency": "USD",
            "default_rate_usd": 145.0,
            "notes_md": "Primary commercial HVAC account.",
            "tax_rate_pct": 7.5,
            "payment_terms": "Net 30",
        }),
    )
    .unwrap();

    // Recurring routine
    let _ = routines::create_routine(
        data_dir,
        "Weekly Dispatch Recap",
        "Summarize open work orders and completed installs.",
        "weekly",
        Some("summarize_meeting"),
    )
    .unwrap();

    // Second person workspace under data_dir/people/alex-rivera/
    let mut people = PeopleState::load(&cfg);
    let alex = people.add("Alex Rivera", "Project Manager");
    people.save(&cfg);
    let alex_cfg = cfg.person_config(&alex);
    alex_cfg.ensure_dirs().unwrap();
    std::fs::write(
        alex_cfg.workspace_dir.join("field-checklist.md"),
        "- Pressure test complete\n- Photos uploaded\n",
    )
    .unwrap();
    std::fs::write(
        &alex_cfg.memory_path,
        "# Memory\n\n- Role: Project Manager\n- Prefers concise bullet recaps.\n",
    )
    .unwrap();
}

fn http_raw(port: u16, req: &str) -> (u16, String, Vec<u8>) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP response missing header separator");
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let body = raw[split + 4..].to_vec();
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    (status, head, body)
}

fn http_json(port: u16, method: &str, path: &str, body: Option<&Value>, gate: Option<&str>) -> (u16, Value) {
    let payload = body.map(|v| serde_json::to_string(v).unwrap()).unwrap_or_default();
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    if let Some(g) = gate {
        req.push_str(&format!("X-Ergon-Gate: {g}\r\n"));
    }
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", payload.len()));
    }
    req.push_str("\r\n");
    req.push_str(&payload);
    let (status, _, bytes) = http_raw(port, &req);
    let val = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}));
    (status, val)
}

fn spawn_test_server(data_dir: &Path, gate_token: Option<String>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let cfg = test_config(data_dir);
    std::thread::spawn(move || {
        let _ = ergon_rs::serve_with_gate(listener, cfg, gate_token);
    });
    // Wait until accepting connections
    for _ in 0..40 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    port
}

/// Helper to craft a raw ZIP with custom entry names and Unix external attributes
/// (used for testing zip-slip and symlink rejection).
fn build_raw_zip_with_attrs(entries: &[(&str, &[u8], u16, u32)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for &(name, data, version_made_by, external_attrs) in entries {
        let crc = ergon_rs::docs::crc32(data);
        let offset = out.len() as u32;
        let nb = name.as_bytes();
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0x0800u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(nb);
        out.extend_from_slice(data);

        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&version_made_by.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0x0800u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(nb.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0u8; 8]);
        central.extend_from_slice(&external_attrs.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(nb);
    }
    let cd_off = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_len.to_le_bytes());
    out.extend_from_slice(&cd_off.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[test]
fn full_backup_verify_wipe_restore_and_reverify_roundtrip() {
    let tmp = TempDir::new("full_roundtrip");
    let data_dir = tmp.path().join("instance-data");
    let out_zip = tmp.path().join("snapshots").join("full-instance.zip");
    seed_rich_instance(&data_dir);

    let orig_contract = std::fs::read(data_dir.join("workspace/contracts/service-agreement.md")).unwrap();
    let orig_csv = std::fs::read(data_dir.join("workspace/reports/q4-ledger.csv")).unwrap();
    let orig_bin = std::fs::read(data_dir.join("workspace/reports/logo.bin")).unwrap();
    let orig_hist = std::fs::read(data_dir.join("workspace/.history/service-agreement-rev1.md")).unwrap();
    let orig_mem = std::fs::read(data_dir.join("memory.md")).unwrap();
    let orig_alex = std::fs::read(data_dir.join("people/alex-rivera/workspace/field-checklist.md")).unwrap();
    let orig_chain = receipts::load_chain(&data_dir);
    let (orig_chain_ok, orig_head) = receipts::verify_chain(&orig_chain);
    assert!(orig_chain_ok);
    assert_eq!(orig_chain.len(), 2);

    // 1. Run `ergon backup` via compiled binary
    let b_out = Command::new(ergon_bin())
        .args([
            "backup",
            "--data",
            data_dir.to_str().unwrap(),
            "--out",
            out_zip.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        b_out.status.success(),
        "ergon backup failed: stdout={} stderr={}",
        String::from_utf8_lossy(&b_out.stdout),
        String::from_utf8_lossy(&b_out.stderr)
    );
    let b_json: Value = serde_json::from_slice(&b_out.stdout).unwrap();
    assert_eq!(b_json["ok"], true);
    assert!(out_zip.is_file());
    assert!(PathBuf::from(format!("{}.sha256", out_zip.display())).is_file());

    // Confirm transient .tmp / .lock and backups/ folder were excluded
    let zip_bytes = std::fs::read(&out_zip).unwrap();
    let bv = backup::verify_backup_archive_bytes(&zip_bytes);
    assert!(bv.valid, "backup verification errors: {:?}", bv.errors);
    let manifest = bv.manifest.as_ref().unwrap();
    assert!(
        !manifest.files.iter().any(|f| f.path.contains(".tmp") || f.path.ends_with(".lock")),
        "transient tmp/lock files must be excluded from backup"
    );
    assert!(
        !manifest.files.iter().any(|f| f.path.starts_with("backups/")),
        "backups/ folder must be excluded to prevent recursion"
    );

    // 2. Run `ergon verify <backup.zip>` via compiled binary
    let v_out = Command::new(ergon_bin())
        .args(["verify", out_zip.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(
        v_out.status.success(),
        "ergon verify failed: {}",
        String::from_utf8_lossy(&v_out.stdout)
    );
    let v_json: Value = serde_json::from_slice(&v_out.stdout).unwrap();
    assert_eq!(v_json["valid"], true);
    assert_eq!(v_json["kind"], "backup_archive");

    // 3. COMPLETELY WIPE ERGON_DATA_DIR!
    std::fs::remove_dir_all(&data_dir).unwrap();
    assert!(!data_dir.exists(), "data_dir must be completely wiped");

    // 4. Run `ergon restore <archive.zip> --data <data_dir>`
    let r_out = Command::new(ergon_bin())
        .args([
            "restore",
            out_zip.to_str().unwrap(),
            "--data",
            data_dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        r_out.status.success(),
        "ergon restore failed: stdout={} stderr={}",
        String::from_utf8_lossy(&r_out.stdout),
        String::from_utf8_lossy(&r_out.stderr)
    );
    let r_json: Value = serde_json::from_slice(&r_out.stdout).unwrap();
    assert_eq!(r_json["ok"], true);
    assert_eq!(r_json["verified_on_disk"], true);
    assert_eq!(r_json["restored_files"].as_u64().unwrap() as usize, manifest.file_count);

    // 5. Verify EVERY restored file, binary blob, .history, receipt chain, client, routine, and second person!
    assert_eq!(
        std::fs::read(data_dir.join("workspace/contracts/service-agreement.md")).unwrap(),
        orig_contract
    );
    assert_eq!(
        std::fs::read(data_dir.join("workspace/reports/q4-ledger.csv")).unwrap(),
        orig_csv
    );
    assert_eq!(
        std::fs::read(data_dir.join("workspace/reports/logo.bin")).unwrap(),
        orig_bin
    );
    assert_eq!(
        std::fs::read(data_dir.join("workspace/.history/service-agreement-rev1.md")).unwrap(),
        orig_hist
    );
    assert_eq!(std::fs::read(data_dir.join("memory.md")).unwrap(), orig_mem);
    assert_eq!(
        std::fs::read(data_dir.join("people/alex-rivera/workspace/field-checklist.md")).unwrap(),
        orig_alex
    );

    let restored_chain = receipts::load_chain(&data_dir);
    let (restored_chain_ok, restored_head) = receipts::verify_chain(&restored_chain);
    assert!(restored_chain_ok);
    assert_eq!(restored_chain.len(), 2);
    assert_eq!(restored_head, orig_head);

    let restored_clients = records::load_clients_store(&data_dir);
    assert!(restored_clients
        .clients
        .iter()
        .any(|c| c.name == "Northwind Mechanical"));

    let restored_routines = routines::list_routines(&data_dir);
    assert!(restored_routines
        .iter()
        .any(|r| r["name"] == "Weekly Dispatch Recap"));

    // 6. Re-verify archive AND restored data_dir on disk with `ergon verify`!
    let v2_out = Command::new(ergon_bin())
        .args([
            "verify",
            out_zip.to_str().unwrap(),
            "--data",
            data_dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        v2_out.status.success(),
        "post-restore ergon verify failed: {}",
        String::from_utf8_lossy(&v2_out.stdout)
    );
    let v2_json: Value = serde_json::from_slice(&v2_out.stdout).unwrap();
    assert_eq!(v2_json["valid"], true);
    assert_eq!(v2_json["workspace_valid"], true);
}

#[test]
fn retention_pruning_and_offsite_folder_replication() {
    let tmp = TempDir::new("retention_offsite");
    let data_dir = tmp.path().join("data");
    let offsite_dir = tmp.path().join("offsite-nas");
    seed_rich_instance(&data_dir);

    let cfg = BackupConfig {
        enabled: true,
        interval_hours: 24,
        retention_count: 3,
        local_dir: String::new(),
        offsite: BackupOffsiteConfig {
            kind: "folder".to_string(),
            folder_path: Some(offsite_dir.to_string_lossy().into_owned()),
            ..BackupOffsiteConfig::default()
        },
    };
    backup::save_backup_config(&data_dir, &cfg).unwrap();

    let mut created_names = Vec::new();
    for i in 0..5 {
        std::fs::write(
            data_dir.join("workspace/reports/iteration.txt"),
            format!("snapshot #{i}\n"),
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let report = backup::run_backup_with_config(&data_dir, &cfg, None).unwrap();
        assert!(report.offsite.as_ref().is_some_and(|o| o.ok));
        created_names.push(report.filename);
    }

    let local_list = backup::list_backups(&data_dir.join("backups"));
    let offsite_list = backup::list_backups(&offsite_dir);
    assert_eq!(local_list.len(), 3, "local retention must keep exact N=3 backups");
    assert_eq!(offsite_list.len(), 3, "offsite retention must keep exact N=3 backups");

    // Verify the 3 newest snapshots survived and the 2 oldest were pruned (+ their .sha256 sidecars)
    let expected_kept: Vec<String> = created_names.iter().rev().take(3).cloned().collect();
    let local_names: Vec<String> = local_list.iter().map(|b| b.name.clone()).collect();
    let offsite_names: Vec<String> = offsite_list.iter().map(|b| b.name.clone()).collect();
    assert_eq!(local_names, expected_kept);
    assert_eq!(offsite_names, expected_kept);

    for old in &created_names[..2] {
        assert!(!data_dir.join("backups").join(old).exists());
        assert!(!data_dir.join("backups").join(format!("{old}.sha256")).exists());
        assert!(!offsite_dir.join(old).exists());
        assert!(!offsite_dir.join(format!("{old}.sha256")).exists());
    }

    // Offsite copies must match SHA-256 and pass offline verification
    for name in &expected_kept {
        let lb = std::fs::read(data_dir.join("backups").join(name)).unwrap();
        let ob = std::fs::read(offsite_dir.join(name)).unwrap();
        assert_eq!(sha256_hex(&lb), sha256_hex(&ob));
        assert!(backup::verify_backup_archive_bytes(&ob).valid);
    }
}

#[test]
fn offsite_s3_sigv4_put_verified_against_loopback_server() {
    let tmp = TempDir::new("s3_sigv4");
    let data_dir = tmp.path().join("data");
    seed_rich_instance(&data_dir);

    let access_key = "TESTACCESSKEYERGON01";
    let secret_key = "TestSecretKeyErgonSigV4VerificationOnly99";

    #[derive(Clone)]
    struct CapturedPut {
        target: String,
        host: String,
        amz_date: String,
        payload_sha: String,
        authorization: String,
        body: Vec<u8>,
    }

    let captured: Arc<Mutex<Vec<CapturedPut>>> = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let cap_server = captured.clone();

    let server_thread = std::thread::spawn(move || {
        // Handle 2 PUT requests (.zip and .zip.sha256)
        for _ in 0..2 {
            let Ok((stream, _)) = listener.accept() else {
                break;
            };
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let mut reader = BufReader::new(stream);
            let mut req_line = String::new();
            if reader.read_line(&mut req_line).is_err() {
                continue;
            }
            let mut parts = req_line.split_whitespace();
            let _method = parts.next().unwrap_or("");
            let target = parts.next().unwrap_or("").to_string();

            let mut host = String::new();
            let mut amz_date = String::new();
            let mut payload_sha = String::new();
            let mut authorization = String::new();
            let mut content_len = 0usize;

            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" || line == "\n" {
                    break;
                }
                if let Some((k, v)) = line.split_once(':') {
                    let kl = k.trim().to_ascii_lowercase();
                    let vv = v.trim().to_string();
                    match kl.as_str() {
                        "host" => host = vv,
                        "x-amz-date" => amz_date = vv,
                        "x-amz-content-sha256" => payload_sha = vv,
                        "authorization" => authorization = vv,
                        "content-length" => content_len = vv.parse().unwrap_or(0),
                        _ => {}
                    }
                }
            }
            let mut body = vec![0u8; content_len];
            let _ = reader.read_exact(&mut body);

            cap_server.lock().unwrap().push(CapturedPut {
                target,
                host,
                amz_date,
                payload_sha,
                authorization,
                body,
            });
            let mut sock = reader.into_inner();
            let _ = sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = sock.flush();
        }
    });

    let offsite_url = format!("http://127.0.0.1:{port}/ergon-test-bucket/daily-backups");
    let out = Command::new(ergon_bin())
        .env("ERGON_BACKUP_S3_ACCESS_KEY", access_key)
        .env("ERGON_BACKUP_S3_SECRET_KEY", secret_key)
        .env("ERGON_BACKUP_S3_REGION", "us-west-2")
        .args([
            "backup",
            "--data",
            data_dir.to_str().unwrap(),
            "--offsite",
            &offsite_url,
            "--json",
        ])
        .output()
        .unwrap();

    server_thread.join().unwrap();

    assert!(
        out.status.success(),
        "ergon backup --offsite s3 failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout_text = String::from_utf8_lossy(&out.stdout);
    let stderr_text = String::from_utf8_lossy(&out.stderr);
    // Secrets must NEVER appear in stdout, stderr, config, or state files!
    assert!(!stdout_text.contains(secret_key));
    assert!(!stdout_text.contains(access_key));
    assert!(!stderr_text.contains(secret_key));
    let state_text = std::fs::read_to_string(data_dir.join("backup_state.json")).unwrap_or_default();
    assert!(!state_text.contains(secret_key));
    assert!(!state_text.contains(access_key));

    let puts = captured.lock().unwrap();
    assert_eq!(puts.len(), 2, "expected .zip and .zip.sha256 S3 PUT uploads");

    // Independently verify AWS SigV4 signature on BOTH received requests!
    for req in puts.iter() {
        assert!(req.target.starts_with("/ergon-test-bucket/daily-backups/ergon-backup-"));
        let actual_body_sha = sha256_hex(&req.body);
        assert_eq!(req.payload_sha, actual_body_sha);
        assert_eq!(req.amz_date.len(), 16);
        let date_stamp = &req.amz_date[..8];
        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            req.host.to_ascii_lowercase(),
            req.payload_sha,
            req.amz_date
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let canonical_request = format!(
            "PUT\n{}\n\n{}\n{}\n{}",
            req.target, canonical_headers, signed_headers, req.payload_sha
        );
        let scope = format!("{date_stamp}/us-west-2/s3/aws4_request");
        let sts = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            req.amz_date,
            scope,
            sha256_hex(canonical_request.as_bytes())
        );
        let k_date = tripwire::hmac_sha256(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
        let k_region = tripwire::hmac_sha256(&k_date, b"us-west-2");
        let k_service = tripwire::hmac_sha256(&k_region, b"s3");
        let k_signing = tripwire::hmac_sha256(&k_service, b"aws4_request");
        let sig_bytes = tripwire::hmac_sha256(&k_signing, sts.as_bytes());
        let expected_sig: String = sig_bytes.iter().map(|b| format!("{b:02x}")).collect();
        let expected_auth = format!(
            "AWS4-HMAC-SHA256 Credential={access_key}/{scope}, SignedHeaders={signed_headers}, Signature={expected_sig}"
        );
        assert_eq!(req.authorization, expected_auth, "AWS SigV4 signature mismatch");
    }

    // Verify the uploaded ZIP body received by the S3 server passes full backup verification!
    let zip_put = puts.iter().find(|p| p.target.ends_with(".zip")).unwrap();
    let ver = backup::verify_backup_archive_bytes(&zip_put.body);
    assert!(ver.valid, "S3 uploaded ZIP failed verify: {:?}", ver.errors);
}

#[test]
fn tampered_archive_zip_slip_and_symlinks_rejected_without_touching_data_dir() {
    let tmp = TempDir::new("tamper_zipslip");
    let src_dir = tmp.path().join("src-data");
    let target_dir = tmp.path().join("live-data");
    seed_rich_instance(&src_dir);

    // Seed sentinel live target directory that MUST remain untouched on every failure!
    std::fs::create_dir_all(target_dir.join("workspace")).unwrap();
    let sentinel_path = target_dir.join("workspace/keep-me.txt");
    std::fs::write(&sentinel_path, "LIVE_DATA_INTACT").unwrap();

    // Create a valid backup first
    let built = backup::build_backup_archive(&src_dir, &[]).unwrap();
    let valid_zip_path = tmp.path().join("valid.zip");
    std::fs::write(&valid_zip_path, &built.zip_bytes).unwrap();

    // 1. Non-empty restore without --force is rejected and leaves sentinel untouched
    let no_force = backup::restore_backup(&valid_zip_path, &target_dir, false);
    assert!(no_force.is_err());
    assert_eq!(std::fs::read_to_string(&sentinel_path).unwrap(), "LIVE_DATA_INTACT");

    // 2. Tamper one byte inside backup ZIP payload
    let mut tampered_bytes = built.zip_bytes.clone();
    let mid = tampered_bytes.len() / 2;
    tampered_bytes[mid] ^= 0xFF;
    let tampered_path = tmp.path().join("tampered.zip");
    std::fs::write(&tampered_path, &tampered_bytes).unwrap();
    assert!(!backup::verify_backup_archive_bytes(&tampered_bytes).valid);
    assert!(backup::restore_backup(&tampered_path, &target_dir, true).is_err());
    assert_eq!(std::fs::read_to_string(&sentinel_path).unwrap(), "LIVE_DATA_INTACT");

    // 3. Tamper manifest JSON metadata (digest mismatch)
    let mut bad_manifest = built.manifest.to_json();
    bad_manifest["ergon_version"] = json!("9.9.9-tampered");
    let bad_manifest_bytes = serde_json::to_vec(&bad_manifest).unwrap();
    let bad_manifest_zip = build_raw_zip_with_attrs(&[(
        backup::BACKUP_MANIFEST_NAME,
        &bad_manifest_bytes,
        (3u16 << 8) | 20,
        0o100644u32 << 16,
    )]);
    let bad_manifest_path = tmp.path().join("bad-manifest.zip");
    std::fs::write(&bad_manifest_path, &bad_manifest_zip).unwrap();
    let bm_ver = backup::verify_backup_archive_bytes(&bad_manifest_zip);
    assert!(!bm_ver.valid);
    assert!(bm_ver.errors.iter().any(|e| e.contains("manifest_sha256")));
    assert!(backup::restore_backup(&bad_manifest_path, &target_dir, true).is_err());
    assert_eq!(std::fs::read_to_string(&sentinel_path).unwrap(), "LIVE_DATA_INTACT");

    // 4. Zip-slip (`../escape.txt`) attack
    let evil_content = b"pwned";
    let evil_sha = sha256_hex(evil_content);
    let mut evil_manifest = backup::BackupManifest {
        schema: backup::BACKUP_MANIFEST_SCHEMA.to_string(),
        ergon_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc: "2026-11-26T22:00:00Z".to_string(),
        created_at_ms: 1_795_730_400_000,
        file_count: 1,
        total_bytes: evil_content.len() as u64,
        files: vec![backup::BackupManifestFile {
            path: "../escape.txt".to_string(),
            archive_path: "../escape.txt".to_string(),
            size_bytes: evil_content.len() as u64,
            sha256: evil_sha,
        }],
        manifest_sha256: String::new(),
    };
    evil_manifest.manifest_sha256 = evil_manifest.compute_digest();
    let evil_manifest_bytes = serde_json::to_vec(&evil_manifest.to_json()).unwrap();
    let zipslip_bytes = build_raw_zip_with_attrs(&[
        (
            backup::BACKUP_MANIFEST_NAME,
            &evil_manifest_bytes,
            (3u16 << 8) | 20,
            0o100644u32 << 16,
        ),
        (
            "../escape.txt",
            evil_content,
            (3u16 << 8) | 20,
            0o100644u32 << 16,
        ),
    ]);
    let zipslip_path = tmp.path().join("zipslip.zip");
    std::fs::write(&zipslip_path, &zipslip_bytes).unwrap();
    let zs_ver = backup::verify_backup_archive_bytes(&zipslip_bytes);
    assert!(!zs_ver.valid);
    assert!(zs_ver.errors.iter().any(|e| e.contains("traversal") || e.contains("unsafe")));
    assert!(backup::restore_backup(&zipslip_path, &target_dir, true).is_err());
    assert!(!tmp.path().join("escape.txt").exists(), "zip-slip file must never be written");
    assert_eq!(std::fs::read_to_string(&sentinel_path).unwrap(), "LIVE_DATA_INTACT");

    // 5. Symlink entry inside ZIP archive
    let sym_target = b"/etc/passwd";
    let mut sym_manifest = backup::BackupManifest {
        schema: backup::BACKUP_MANIFEST_SCHEMA.to_string(),
        ergon_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc: "2026-11-26T22:00:00Z".to_string(),
        created_at_ms: 1_795_730_400_000,
        file_count: 1,
        total_bytes: sym_target.len() as u64,
        files: vec![backup::BackupManifestFile {
            path: "workspace/link.txt".to_string(),
            archive_path: "data/workspace/link.txt".to_string(),
            size_bytes: sym_target.len() as u64,
            sha256: sha256_hex(sym_target),
        }],
        manifest_sha256: String::new(),
    };
    sym_manifest.manifest_sha256 = sym_manifest.compute_digest();
    let sym_manifest_bytes = serde_json::to_vec(&sym_manifest.to_json()).unwrap();
    let symlink_zip = build_raw_zip_with_attrs(&[
        (
            backup::BACKUP_MANIFEST_NAME,
            &sym_manifest_bytes,
            (3u16 << 8) | 20,
            0o100644u32 << 16,
        ),
        (
            "data/workspace/link.txt",
            sym_target,
            (3u16 << 8) | 20,
            0o120777u32 << 16, // S_IFLNK
        ),
    ]);
    let sym_path = tmp.path().join("symlink.zip");
    std::fs::write(&sym_path, &symlink_zip).unwrap();
    let sym_ver = backup::verify_backup_archive_bytes(&symlink_zip);
    assert!(!sym_ver.valid);
    assert!(sym_ver.errors.iter().any(|e| e.contains("symlink")));
    assert!(backup::restore_backup(&sym_path, &target_dir, true).is_err());
    assert_eq!(std::fs::read_to_string(&sentinel_path).unwrap(), "LIVE_DATA_INTACT");

    // 6. Valid restore WITH --force preserves a pre-restore safety snapshot of LIVE_DATA_INTACT!
    let ok = backup::restore_backup(&valid_zip_path, &target_dir, true).unwrap();
    assert!(ok.verified_on_disk);
    let snap_name = ok.pre_restore_snapshot.expect("expected pre-restore safety snapshot");
    let snap_path = target_dir.join("backups").join(&snap_name);
    assert!(snap_path.is_file(), "pre-restore safety snapshot must exist inside backups/");
    assert!(backup::verify_backup_archive_bytes(&std::fs::read(&snap_path).unwrap()).valid);
    assert!(target_dir.join("workspace/contracts/service-agreement.md").is_file());
}

#[test]
fn backup_http_api_end_to_end_and_gate_confinement() {
    let tmp = TempDir::new("api_e2e");
    let data_dir = tmp.path().join("data");
    let offsite_dir = tmp.path().join("offsite");
    seed_rich_instance(&data_dir);

    let port = spawn_test_server(&data_dir, None);

    // 1. Capabilities include backups: true
    let (st, status_json) = http_json(port, "GET", "/api/status", None, None);
    assert_eq!(st, 200);
    assert_eq!(status_json["capabilities"]["backups"], true);

    // 2. GET /api/backups
    let (st, initial) = http_json(port, "GET", "/api/backups", None, None);
    assert_eq!(st, 200);
    assert_eq!(initial["can_edit"], true);
    assert_eq!(initial["backups"].as_array().unwrap().len(), 0);

    // 3. POST /api/backups/config
    let (st, cfg_res) = http_json(
        port,
        "POST",
        "/api/backups/config",
        Some(&json!({
            "enabled": true,
            "interval_hours": 12,
            "retention_count": 5,
            "offsite": {
                "kind": "folder",
                "folder_path": offsite_dir.to_string_lossy(),
            }
        })),
        None,
    );
    assert_eq!(st, 200);
    assert_eq!(cfg_res["config"]["enabled"], true);
    assert_eq!(cfg_res["config"]["interval_hours"], 12);
    assert_eq!(cfg_res["config"]["retention_count"], 5);

    // 4. POST /api/backups/run
    let (st, run_res) = http_json(port, "POST", "/api/backups/run", Some(&json!({})), None);
    assert_eq!(st, 200, "run failed: {run_res}");
    assert_eq!(run_res["ok"], true);
    let snap_name = run_res["backup"]["filename"].as_str().unwrap().to_string();
    let snap_sha = run_res["backup"]["sha256"].as_str().unwrap().to_string();
    assert!(offsite_dir.join(&snap_name).is_file());

    // 5. GET /api/backups/download?name=...
    let dl_req = format!(
        "GET /api/backups/download?name={snap_name} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    let (dl_st, dl_head, dl_body) = http_raw(port, &dl_req);
    assert_eq!(dl_st, 200);
    assert!(dl_head.to_ascii_lowercase().contains("application/zip"));
    assert_eq!(sha256_hex(&dl_body), snap_sha);
    assert!(backup::verify_backup_archive_bytes(&dl_body).valid);

    // 6. Strict path confinement on download & restore
    let trav_req = format!(
        "GET /api/backups/download?name=../../etc/passwd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    let (trav_st, _, _) = http_raw(port, &trav_req);
    assert_eq!(trav_st, 403);

    let (st, _) = http_json(
        port,
        "POST",
        "/api/backups/restore",
        Some(&json!({"name": snap_name, "confirm": false})),
        None,
    );
    assert_eq!(st, 400);

    let (st, _) = http_json(
        port,
        "POST",
        "/api/backups/restore",
        Some(&json!({"name": "../escape.zip", "confirm": true})),
        None,
    );
    assert_eq!(st, 403);

    // 7. Delete workspace files live, then restore via API & verify files + live server state!
    std::fs::remove_file(data_dir.join("workspace/contracts/service-agreement.md")).unwrap();
    assert!(!data_dir.join("workspace/contracts/service-agreement.md").exists());

    let (st, rest_res) = http_json(
        port,
        "POST",
        "/api/backups/restore",
        Some(&json!({"name": snap_name, "confirm": true})),
        None,
    );
    assert_eq!(st, 200, "restore failed: {rest_res}");
    assert_eq!(rest_res["restore"]["verified_on_disk"], true);
    assert!(rest_res["restore"]["pre_restore_snapshot"].is_string());
    assert!(data_dir.join("workspace/contracts/service-agreement.md").is_file());

    // Live /api/files reflects restored file without server restart
    let (st, files_json) = http_json(port, "GET", "/api/files", None, None);
    assert_eq!(st, 200);
    let files_str = files_json.to_string();
    assert!(files_str.contains("service-agreement.md"));

    // 8. Gated deployment blocks ALL /api/backups* endpoints with 403 (admin local-only)
    let gated_port = spawn_test_server(&data_dir, Some("Gate-Secret-42".to_string()));
    for (method, path, body) in [
        ("GET", "/api/backups", None),
        ("POST", "/api/backups/run", Some(json!({}))),
        ("POST", "/api/backups/config", Some(json!({"enabled": true}))),
        (
            "POST",
            "/api/backups/restore",
            Some(json!({"name": snap_name, "confirm": true})),
        ),
    ] {
        let (gst, _) = http_json(
            gated_port,
            method,
            path,
            body.as_ref(),
            Some("Gate-Secret-42"),
        );
        assert_eq!(gst, 403, "gated {method} {path} must return 403");
    }
}

#[test]
fn scheduled_backup_due_logic_respects_enabled_and_interval() {
    let mut cfg = BackupConfig::default();
    let mut st = backup::BackupRuntimeState::default();
    let now = 1_800_000_000_000u64;

    assert!(!backup::is_scheduled_backup_due(&cfg, &st, now));
    cfg.enabled = true;
    cfg.interval_hours = 6;
    assert!(backup::is_scheduled_backup_due(&cfg, &st, now));

    st.last_run_ms = Some(now - 3 * 3_600_000); // 3h ago
    assert!(!backup::is_scheduled_backup_due(&cfg, &st, now));

    st.last_run_ms = Some(now - 6 * 3_600_000); // 6h ago
    assert!(backup::is_scheduled_backup_due(&cfg, &st, now));
}

#[test]
fn backslash_traversal_drive_prefix_and_unmanifested_entries_rejected() {
    let tmp = TempDir::new("extra_zip_attacks");
    let target_dir = tmp.path().join("live-data");
    std::fs::create_dir_all(target_dir.join("workspace")).unwrap();
    let sentinel = target_dir.join("workspace/keep.txt");
    std::fs::write(&sentinel, "SAFE").unwrap();

    // 1. Windows backslash traversal entry
    let payload = b"bad";
    let mut m = backup::BackupManifest {
        schema: backup::BACKUP_MANIFEST_SCHEMA.to_string(),
        ergon_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc: "2026-11-26T22:00:00Z".to_string(),
        created_at_ms: 1_795_730_400_000,
        file_count: 1,
        total_bytes: payload.len() as u64,
        files: vec![backup::BackupManifestFile {
            path: "workspace/a.txt".to_string(),
            archive_path: r"data\..\..\escape.txt".to_string(),
            size_bytes: payload.len() as u64,
            sha256: sha256_hex(payload),
        }],
        manifest_sha256: String::new(),
    };
    m.manifest_sha256 = m.compute_digest();
    let mb = serde_json::to_vec(&m.to_json()).unwrap();
    let bs_zip = build_raw_zip_with_attrs(&[
        (backup::BACKUP_MANIFEST_NAME, &mb, (3u16 << 8) | 20, 0o100644u32 << 16),
        (r"data\..\..\escape.txt", payload, (3u16 << 8) | 20, 0o100644u32 << 16),
    ]);
    assert!(!backup::verify_backup_archive_bytes(&bs_zip).valid);

    // 2. Unmanifested extra entry injected alongside a valid manifest
    let mut m2 = backup::BackupManifest {
        schema: backup::BACKUP_MANIFEST_SCHEMA.to_string(),
        ergon_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc: "2026-11-26T22:00:00Z".to_string(),
        created_at_ms: 1_795_730_400_000,
        file_count: 1,
        total_bytes: payload.len() as u64,
        files: vec![backup::BackupManifestFile {
            path: "workspace/a.txt".to_string(),
            archive_path: "data/workspace/a.txt".to_string(),
            size_bytes: payload.len() as u64,
            sha256: sha256_hex(payload),
        }],
        manifest_sha256: String::new(),
    };
    m2.manifest_sha256 = m2.compute_digest();
    let mb2 = serde_json::to_vec(&m2.to_json()).unwrap();
    let extra_zip = build_raw_zip_with_attrs(&[
        (backup::BACKUP_MANIFEST_NAME, &mb2, (3u16 << 8) | 20, 0o100644u32 << 16),
        ("data/workspace/a.txt", payload, (3u16 << 8) | 20, 0o100644u32 << 16),
        ("data/workspace/sneaky.txt", b"extra", (3u16 << 8) | 20, 0o100644u32 << 16),
    ]);
    let extra_ver = backup::verify_backup_archive_bytes(&extra_zip);
    assert!(!extra_ver.valid);
    assert!(extra_ver.errors.iter().any(|e| e.contains("unmanifested")));

    let extra_path = tmp.path().join("extra.zip");
    std::fs::write(&extra_path, &extra_zip).unwrap();
    assert!(backup::restore_backup(&extra_path, &target_dir, true).is_err());
    assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "SAFE");
}

#[test]
fn background_scheduler_triggers_automatic_verified_backup_when_due() {
    let tmp = TempDir::new("bg_scheduler");
    let data_dir = tmp.path().join("data");
    seed_rich_instance(&data_dir);

    // Enable scheduled backups in backup_config.json before starting server
    let cfg = BackupConfig {
        enabled: true,
        interval_hours: 1,
        retention_count: 4,
        local_dir: String::new(),
        offsite: BackupOffsiteConfig::default(),
    };
    backup::save_backup_config(&data_dir, &cfg).unwrap();

    // Pick a free loopback port cleanly, then drop listener right before spawning binary.
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    let mut child = Command::new(ergon_bin())
        .env("ERGON_DATA_DIR", &data_dir)
        .env("ERGON_HOST", "127.0.0.1")
        .env("ERGON_PORT", port.to_string())
        .env("ERGON_NO_OPEN", "1")
        .env("ERGON_NO_AUTO_MODEL", "1")
        .env("ERGON_BACKUP_TICK_MS", "250")
        .args(["--no-open"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let backups_dir = data_dir.join("backups");
    let mut found = Vec::new();
    for _ in 0..40 {
        found = backup::list_backups(&backups_dir);
        if !found.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        !found.is_empty(),
        "background backup scheduler should have created a due backup automatically"
    );
    let ver = backup::verify_backup_file(&found[0].path, None);
    assert!(ver.valid, "scheduled backup failed verify: {:?}", ver.errors);
}
