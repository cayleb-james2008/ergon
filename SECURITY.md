# Security Model & Threat Controls (`SECURITY.md`)

Ergon runs autonomous file, connector, email, and MCP actions on behalf of non-technical employees. Its core design principle is **fail-closed sandboxing at every trust boundary** combined with **human approval reserved strictly for consequential actions**.

---

## 1. Threat Model & Boundaries

| Attack Surface | Threat | Mitigation in Ergon | Module |
|---|---|---|---|
| **Untrusted Documents & Web Content** | Indirect prompt injection inside uploaded PDFs, CSVs, emails, or fetched URLs trying to hijack tool calls | Spotlighting delimiters wrap external/workspace observations; injection-pattern alerts hold suspicious memory writes or skills | `src/tripwire.rs`, `src/lib.rs` |
| **Workspace Filesystem Access** | Path traversal (`../`), sibling-prefix bypass, null bytes, or symlink escapes reading `/etc/passwd` or `.env` | Component-wise path confinement with a **16-hop symlink cap** (`sandbox_path`, `sandbox_path_resolving`) refusing symlinks and outside targets | `src/tools.rs`, `src/tripwire.rs` |
| **Credential Exfiltration** | Model lured into leaking API keys or sending secrets to an external host | Traceable bait keys (`sk-ergon-...`), honeytrap paths (`/.env`, `/.git/config`), encrypted secret vault (`secrets.enc.json`, `0600` AES-256-GCM + OS keychain), host-side `{{secret:key}}` substitution only *after* egress checks pass | `src/tripwire.rs`, `src/secret_store.rs`, `src/connectors.rs` |
| **SSRF & Network Egress** | Fetching loopback admin endpoints, cloud metadata (`169.254.169.254`), or private IPv4/IPv6 ranges | IP/CIDR/redirect checks blocking loopback, private networks, metadata endpoints, NAT64, 6to4, Teredo, and DNS rebinding `Host` headers; `Air-Gapped` profile blocks all network fetches/connectors | `src/tripwire.rs`, `src/server/http.rs` |
| **Consequential Actions** | Silent deletions, unreviewed outbound emails/webhooks, runaway retainer spend, untrusted generated tools | `delete_file`, `flag_decision`, `connector_delivery`, `mcp_tool_call`, `budget_gate`, and `tool_offer` park as concise Approve/Deny cards on `/` and `/phone` | `src/records.rs`, `src/tool_builder.rs` |
| **Multi-User & Client Isolation** | Cross-person workspace access or cross-client data leaks in shared memory/DB | PBKDF2-HMAC-SHA256 (`120k`), `SystemRandom` `HttpOnly` sessions, optional OIDC + PKCE `S256`, centralized RBAC, engagement-prefix vaults (`HTTP 409` under `Strict Client`), scoped `ergk_*` API keys | `src/auth.rs`, `src/company_db.rs`, `src/server/` |
| **Deliverable & Citation Integrity** | Hallucinated line citations or accidentally shipping `.env`/keys/host paths in client packs | `[file:L##]` verifier checks claims against actual indexed files and `.history/`; `Strict Client` leak pre-check blocks exports with exact `file:L##` hints; SHA-256 chained receipts verifiable offline via `ergon verify` | `src/audit_bundle.rs`, `src/receipts.rs` |
| **Supply Chain & Parser Safety** | Zip bombs, PDF parser hangs, CSV/XLSX formula injection, corrupt JSON stores | `25 MB` upload cap, zip decompression caps, `=`/`+`/`-`/`@` spreadsheet cell escaping, `.bak` quarantine on corrupt JSON recovery | `src/docs.rs`, `src/skills.rs` |

---

## 2. Workspace Policy Profiles

Administrators and operators switch profiles per workspace (`GET`/`PUT`/`POST /api/workspace/policy`):
- **`Fast Internal`:** Autonomous execution; consequential deletes/decisions still park for approval.
- **`Strict Client`:** Blocks handoff/pack exports with `HTTP 409` on stale/missing `[file:L##]` citations, redacts `~/` host paths, blocks exposed secrets/SSN patterns, and requires approval on external deliveries and MCP tool calls.
- **`Air-Gapped`:** Hard-disables outbound web fetches (`fetch_url`, `research`) and outbound connectors.

---

## 3. Verifying Evidence & Reporting Vulnerabilities

```sh
# Offline verification of any exported Client Handoff ZIP, audit bundle, or backup archive
ergon verify path/to/bundle.json --workspace ./data/workspace
```

Security regressions are continuously tested across `tests/tripwire.rs`, `tests/hardening_v032.rs`, `tests/hardening_v036.rs`, `tests/auth_tls_rbac.rs`, and `evals/run_evals.py`.

To report a security issue privately, open a confidential GitHub Security Advisory on `cayleb-james2008/ergon`.
