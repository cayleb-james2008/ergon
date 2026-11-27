# Architecture (`ARCHITECTURE.md`)

Ergon compiles into **one static binary** (Rust + embedded React UI) serving both the desktop app (`/`), the mobile decision surface (`/phone`), scheduled automation, and the authenticated company backend with zero external database or runtime dependencies.

---

## 1. High-Level Topology

```
                        +-----------------------------------+
                        |   Desk UI (/) & Mobile (/phone)   |
                        |   React 19 + TypeScript + Geist   |
                        |  Pre-gzipped /chunks/* + SSE bus  |
                        +-----------------+-----------------+
                                          |
                              Raw TCP / rustls HTTPS
                                          |
                        +-----------------v-----------------+
                        |       HTTP / Auth / RBAC Core     |
                        |   src/server/  +  src/auth.rs     |
                        | Sessions, OIDC, ergk_* keys, TLS  |
                        +-----------------+-----------------+
                                          |
       +--------------------+-------------+-------------+--------------------+
       |                    |                           |                    |
+------v-------+   +--------v--------+         +--------v--------+   +-------v--------+
| Agent Loop & |   |   DAG Pipeline  |         |  Workforce &    |   | Safety & Audit |
| Model Client |   |    Scheduler    |         |  Tool Ecosystem |   |     Engine     |
+--------------+   +-----------------+         +-----------------+   +----------------+
| src/lib.rs   |   | src/pipelines/  |         | src/library.rs  |   | src/tripwire.rs|
| src/llm.rs   |   | src/scheduler.rs|         | src/tool_builder|   | src/receipts.rs|
|  - Anthropic |   | src/routines.rs |         | src/proactive_  |   | src/audit_     |
|    Messages  |   |  - depends_on   |         |   offers.rs     |   |   bundle.rs    |
|  - OpenAI    |   |  - for_each     |         | src/self_       |   | src/secret_    |
|    /chat     |   |  - checkpoints  |         |   improve.rs    |   |   store.rs     |
|  - Local GW  |   |  - crash resume |         | src/quality_bar |   |  - Spotlighting|
+--------------+   +-----------------+         | src/company_db  |   |  - 16-hop FS   |
                                               | src/connectors  |   |  - SHA-256     |
                                               | src/mcp.rs      |   |    chain       |
                                               +-----------------+   +----------------+
```

---

## 2. Codebase Module Map

| Path | Responsibility |
|---|---|
| `src/main.rs`, `src/service.rs`, `src/tls.rs`, `src/update.rs` | CLI entrypoint, OS service wrapper (`systemd`, Windows Service), self-signed/PEM `rustls` TLS, checksum-verified self-updater |
| `src/llm.rs` | Dual protocol model client: native Anthropic `/v1/messages` (`x-api-key`, `cache_control: ephemeral`, SSE) + OpenAI `/v1/chat/completions`, retries (`429`/`529`), configurable `300s` per-read stream timeout |
| `src/lib.rs` | Agent loop, context compaction, honesty ledger (`WriteLedger`), standing memory, per-person `Config` |
| `src/server/` | Modular raw-TCP/TLS server: `mod.rs` (lifecycle, connection loop), `http.rs` (parser, CORS/Host/413/304, webhooks), `background.rs` (schedulers, folder/IMAP poll), `handoff.rs` (citation packs, HTML/ZIP diffs), and domain route modules (`routes_system`, `routes_files`, `routes_workflows`, `routes_company`, `routes_runs`) |
| `src/pipelines/` | Resumable hybrid DAG execution split into `spec.rs` (topological sort, template/expr evaluation, cycle validation), `store.rs` (atomic persistence, crash recovery), and `executor.rs` (bounded `MAX_DAG_CONCURRENCY = 4` workers, `for_each`, subpipelines) |
| `src/tools.rs`, `src/skills.rs`, `src/skills/*.rn`, `src/docs.rs` | Sandboxed workspace primitives, embedded deterministic workflows (`12` built-ins incl. `reconcile_ledger`, `audit_contract`, `make_spreadsheet` `from_csv`, `triage_inbox`), PDF/DOCX/XLSX/PPTX parsers & exporters |
| `src/search.rs`, `src/tables.rs` | Per-person `RwLock` BM25 + 256-d RRF chunked vault index; `i128` exact-decimal CSV/XLSX rollup engine |
| `src/company_db.rs`, `src/tool_builder.rs`, `src/proactive_offers.rs`, `src/self_improve.rs`, `src/quality_bar.rs` | Multi-tenant company DB & visibility-scoped memory; sandbox-tested tool synthesis + `"Would this help?"` offers; replay-gated version promotion & auto-rollback; per-task `Improve` / `Good enough` convergence |
| `src/auth.rs`, `src/connectors.rs`, `src/mcp.rs`, `src/secret_store.rs`, `src/backup.rs` | PBKDF2/OIDC/RBAC + scoped `ergk_*` keys; SMTP/IMAP/OAuth2/HMAC webhooks; stdio + HTTP/SSE MCP client & read/write MCP server; AES-256-GCM secret vault; encrypted backup/restore & SigV4 S3 sync |
| `frontend/` | React 19 + TypeScript + Tailwind 4 UI (`frontend/vendor/ui/` primitives), compiled to pre-gzipped embedded assets (`assets/`) and standalone `ergon-web.html` (`webEngine.ts`) |

---

## 3. Core Design Tradeoffs

1. **Raw `TcpListener` + `rustls` over async Tokio/Axum stacks:** Eliminates async runtime bloat, compiles quickly to a `11.62 MiB` single binary (`v0.42.0` Linux release), serves ~4,009 req/s under 16-worker concurrency with `22.19 MB` peak RSS (`v0.40.0` benchmark), and keeps SSE streaming (`RunEventBus`) simple and inspectable. Tradeoff: per-connection threads are sized for SMB/team deployments (tens to hundreds of concurrent seats) rather than C100k internet edge proxies.
2. **Embedded JSON/JSONL atomic stores + `.history/` + `.bak` quarantine over SQLite/Postgres:** Zero external services to install or migrate, 1-file full-instance backup/restore (`ergon backup`), and human-auditable workspace state. Tradeoff: all operational datasets fit comfortably in local NVMe + in-memory `PersonIndex` caches rather than distributed sharding.
3. **Deterministic sandboxed scripts for math & structure + LLM for routing & synthesis:** LLMs orchestrate tasks, draft prose, and synthesize new tools, while deterministic Rust + embedded workflow scripts perform penny-exact integer-cent ledger/BOM arithmetic (`i128`/`i64`), clause matching, and citation verification—preventing numeric hallucination and keeping replay tests reproducible.
4. **Honesty ledger before completion:** Every tool write is tracked in `WriteLedger` so an agent claiming `"done"` without actually writing cited files is stopped cleanly before deceiving the user or recording a fake receipt.
