# Ergon Backend API

Ergon exposes a JSON HTTP API, a Server-Sent Events stream, signed inbound and outbound webhooks, and an inbound Model Context Protocol (MCP) JSON-RPC 2.0 endpoint from the same server process.

Every state-changing HTTP request (`POST`, `PUT`, `PATCH`, `DELETE`) with a non-empty body requires `Content-Type: application/json`, except `POST /api/hooks/:hook_id`, which accepts arbitrary webhook payloads. Requests must address `localhost`, `127.0.0.1`, `[::1]`, or a hostname listed in `ERGON_ALLOWED_HOSTS` unless authenticated with `ERGON_GATE_TOKEN` or an explicit API key. Browser requests that send `Sec-Fetch-Site: cross-site` or a mismatched `Origin` header are refused with `403`.

All JSON error responses use `{"error": "<message>"}` unless noted otherwise (such as `401` auth challenges or JSON-RPC 2.0 errors).

## Error codes

| Status / Code | Meaning | Common causes |
| --- | --- | --- |
| `400` | Bad Request | Invalid JSON, missing required field, secret or host path in company memory, failed sandbox tool test, invalid table name, or blocked private/metadata URL. |
| `401` | Unauthorized | Missing or expired session cookie, invalid API key, invalid `ERGON_GATE_TOKEN`, or invalid webhook HMAC/token signature. |
| `403` | Forbidden | Valid credentials without the required role or API key scope, disabled account, cross-origin browser request, unconfigured host, or uninstalled tool. |
| `404` | Not Found | Unknown decision, offer, notification, subscription, key, or unapproved/unpublished tool ID. |
| `409` | Conflict | Duplicate username or email, duplicate key/subscription conflict, active run conflict, or preflight/evidence state conflict. |
| `413` | Content Too Large | Payload exceeds route limit (for example, webhook size limit or default request body limit). |
| `415` | Unsupported Media Type | Non-empty `POST`/`PUT`/`PATCH`/`DELETE` body sent without `Content-Type: application/json`. |
| `429` | Too Many Requests | Login rate limit exceeded. |
| `-32700` | JSON-RPC Parse Error | Invalid JSON payload sent to `POST /api/mcp/inbound`. |
| `-32601` | JSON-RPC Method Not Found | Unsupported MCP method or tool name on `/api/mcp/inbound`. |
| `-32001` | MCP Server Disabled | Inbound MCP server is disabled (`HTTP 403`). |
| `-32002` | MCP Auth Failed | Invalid or missing inbound MCP bearer token (`HTTP 401`). |
| `-32003` | MCP Permission Denied | Caller lacks the required scope or role for the requested MCP tool or resource (`HTTP 403`). |

## Authentication and scoped API keys

Authentication is enforced automatically when at least one user account exists, when `ERGON_AUTH_REQUIRED=1` is set, when TLS is active, or when the server binds a non-loopback address.

### Roles

Each user and person workspace has one of six roles:

- `admin`: full system configuration, user management, API keys, connectors, MCP settings, and cross-team visibility.
- `pm`: manages clients, engagements, workspace policies, and shared company memory.
- `ae`: manages clients, engagements, invoices, and shared company memory.
- `office`: operations and finance role; manages clients, engagements, invoices, workspace policies, and shared company memory.
- `installer`: field delivery role scoped to their own workspace plus `company` and `role:installer` shared memory.
- `member`: standard member scoped to their own workspace plus `company` and `role:member` shared memory.

### Session cookies and browser auth

Interactive clients authenticate with `POST /api/auth/setup` (only when zero accounts exist) or `POST /api/auth/login`. Successful responses return a JSON `token` and set an `ergon_session` cookie with `HttpOnly; SameSite=Lax` (and `Secure` when TLS is enabled).

| Method | Path | Access | Notes |
| --- | --- | --- | --- |
| `GET` | `/api/health` | Public | Returns `{ ok, name, version, auth_required, needs_setup, oidc_enabled, tls }`. |
| `GET` | `/api/auth/me` | Public | Returns current principal (`authenticated`, `user`, `roles`) without leaking workspace data. |
| `POST` | `/api/auth/setup` | Public (0 users) | Creates the initial `admin` user and workspace (`201`). Returns `403` once any user exists. |
| `POST` | `/api/auth/login` | Public | Authenticates `username`/`email` + `password`, sets `ergon_session` cookie, returns `token`. |
| `POST` | `/api/auth/logout` | Public | Revokes the active session and clears `ergon_session`. |
| `POST` | `/api/auth/password` | Authenticated | Changes the caller's own password (`current_password`, `new_password`). |
| `GET` / `POST` / `PATCH` / `DELETE` | `/api/auth/users*` | `admin` | Manages user accounts (`409` on duplicate username or email). |
| `GET` / `PUT` / `POST` | `/api/auth/oidc/config` | `admin` | Configures OIDC Authorization Code + PKCE (`S256`). |
| `GET` | `/api/auth/oidc/login`, `/api/auth/oidc/callback` | Public | Browser OIDC sign-in redirect and callback. |

### Break-glass gate token (`ERGON_GATE_TOKEN`)

When `ERGON_GATE_TOKEN` is set in the server environment, callers can pass it via `X-Ergon-Gate`, `Authorization: Bearer <token>`, or `X-Ergon-Token`.

- In multi-user, TLS, or non-loopback mode, `ERGON_GATE_TOKEN` acts as an audited admin break-glass credential recorded in the tamper-evident tripwire log (`AU2-BREAKGLASS`).
- Presenting `X-Ergon-Gate` alongside a non-admin session cookie never elevates that user's permissions; non-admin sessions stay pinned to their own workspace.
- In single-user loopback mode (zero accounts, no TLS, `ERGON_AUTH_REQUIRED` unset), `ERGON_GATE_TOKEN` gates every request to the server.

### Scoped API keys

Programmatic clients authenticate with a scoped API key using either header:

- `Authorization: Bearer ergk_<secret>`
- `X-Ergon-Api-Key: ergk_<secret>`

API keys carry an explicit set of scopes and inherit the owner's workspace isolation and RBAC rules:

| Scope | Grants access to |
| --- | --- |
| `read` | Read-only workspace endpoints (`GET /api/brief`, `/api/decisions`, `/api/offers`, `/api/library`, `/api/tools`, `/api/notifications`, `/api/evolution/scores`, `/api/quality-bars`). |
| `write` | Mutating work actions (`POST /api/chat`, `POST /api/decisions/:id/*`, `POST /api/offers/*`, `POST /api/notifications/:id/read`, `POST /api/evolution/*`, `POST /api/quality-bars/:family/feedback`). |
| `company_db` | `GET /api/company/memory`, `POST /api/company/memory`, `DELETE /api/company/memory/:id`, `GET /api/company/db`, `POST /api/company/db/query` (still subject to caller role RBAC). |
| `tools` | `GET /api/tools`, `POST /api/tools/test`, `POST /api/tools/build`, `POST /api/tools/:id/run`. |
| `webhooks` | `GET /api/webhooks/subscriptions`, `POST /api/webhooks/subscriptions`, `DELETE /api/webhooks/subscriptions/:id`. |
| `mcp` | `POST /api/mcp/inbound`. |
| `admin` | Administrative endpoints including `POST /api/keys`, `GET /api/keys`, `DELETE /api/keys/:id`, user management, and system configuration. |

API key management endpoints:

- `POST /api/keys`: creates a new scoped API key and returns the plaintext `ergk_...` key once (`201`). Request fields: `name`, `scopes` (array from `read`, `write`, `webhooks`, `mcp`, `company_db`, `tools`, `admin`), optional `person_id` or `expires_at_ms`.
- `GET /api/keys`: lists active API key metadata (`id`, `name`, `prefix`, `scopes`, `person_id`, `created_ms`, `last_used_ms`, `expires_at_ms`) without returning the secret.
- `DELETE /api/keys/:id`: revokes an API key immediately (`200` on deletion, `404` if unknown).

```bash
# Sign in and create a scoped API key
TOKEN=$(curl -s -X POST http://127.0.0.1:8787/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"your-password-here"}' | jq -r .token)

curl -s -X POST http://127.0.0.1:8787/api/keys \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"ci-pipeline","scopes":["read","company_db","tools"]}'
```

```bash
# Authenticate with a scoped API key
curl -s http://127.0.0.1:8787/api/company/db?table=summary \
  -H 'X-Ergon-Api-Key: ergk_your_key_here'
```

## Company database and company memory

Company memory and company database endpoints share non-sensitive operational knowledge across employees while keeping private `memory.md` files, password hashes, and session tokens strictly isolated.

### Shared company memory

Stored at `<root_data_dir>/company/company_memory.json` (`0600` permissions) and injected separately from private memory in agent prompts (up to the 25 most recent authorized entries).

| Method | Path | Access | Scope needed |
| --- | --- | --- | --- |
| `GET` | `/api/company/memory` | Any authenticated user | `read` or `company_db` |
| `POST` | `/api/company/memory` | `admin`, `pm`, `ae`, `office` | `company_db` or `write` |
| `DELETE` | `/api/company/memory/:id` | `admin`, `pm`, `ae`, `office` | `company_db` or `write` |

Fields on `POST /api/company/memory`:

- `category`: `fact` (default), `procedure`, `preference`, or `policy`.
- `scope`: `company` (default), `role:<role>` (for example `role:installer`), or `client:<client_id>`.
- `content`: non-empty string up to 2,000 characters, without null bytes.

Privacy and validation rules:

- Read visibility: `company` entries are visible to all authenticated users; `role:<role>` entries are visible to `admin` and matching roles; `client:<client_id>` entries are visible only to `admin`, or to `pm`/`ae`/`office` users assigned to or owning that client (`installer` and `member` never see `client:*` entries).
- Write RBAC: `installer` and `member` receive `403` on `POST` and `DELETE`.
- Secret and host-path guard: `POST /api/company/memory` returns `400` if `content` contains `sk-` tokens, `ergk_`, `ergs_`, `password=`, `api_key=`, `secret=`, `bearer `, `-----begin `, `/home/`, `c:\users\`, or `c:/users/`.

```bash
curl -s -X POST http://127.0.0.1:8787/api/company/memory \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"category":"policy","scope":"company","content":"Standard payment terms are Net-30."}'
```

### Structured company database queries

- `GET /api/company/db?table=<name>&q=<filter>&limit=<n>`
- `POST /api/company/db/query` with JSON `{"table": "<name>", "query": "<filter>", "limit": 50}` (`q` is accepted as an alias for `query`).

Available `table` values (default `summary`, `limit` defaults to `50` and clamps to `1..=200`):

- `summary`: counts across visible tables for the caller's role and `person_id`.
- `people`: IDs, names, and roles.
- `roles`: role capability matrix (`is_admin`, `can_manage_clients`, `can_view_invoices`, `can_manage_invoices`, `can_manage_workspace_policy`).
- `clients`: visible clients and engagements. Requires `admin`, `pm`, `ae`, or `office` (`403` for `installer` and `member`); non-admins only see their assigned clients.
- `work`: recent work runs. `admin` sees all people; non-admins see only their own `person_id`.
- `receipts`: tamper-evident receipt chain entries. `admin` sees all people; non-admins see only their own `person_id`.
- `tools`: shared `skill`, `tool`, `subagent`, and `app` items in the company library.
- `outcomes`: aggregated evolution outcome counts and mean quality scores per `artifact_id`.
- `memory`: company memory entries filtered by the caller's role and allowed client IDs.

```bash
curl -s -X POST http://127.0.0.1:8787/api/company/db/query \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"table":"roles","limit":10}'
```

## Tool builder and company library

Tools are sandboxed workflow scripts that take up to 5 parameters (`pub fn main(p1, ...)`), execute inside a person's workspace, and cannot call `host_memory_write`.

| Method | Path | Access | Purpose |
| --- | --- | --- | --- |
| `GET` | `/api/library` | Authenticated | Lists all shared items (`skill`, `tool`, `subagent`, `app`) in `<root_data_dir>/library.json`. |
| `GET` | `/api/suite` | Authenticated | Lists items installed in the caller's personal suite. |
| `POST` | `/api/suite/install` | Authenticated | Installs a library `item_id` into the caller's suite (`404` if missing, `400` if `builtin`). |
| `POST` | `/api/suite/remove` | Authenticated | Removes `item_id` from the caller's suite. |
| `GET` | `/api/tools` | Authenticated | Returns `{ packages, library_tools, installed_tool_ids }`. |
| `POST` | `/api/tools/test` | Authenticated | Compile-checks `source` and runs `tests` in isolated scratch sandboxes (`<root_data_dir>/.tool_sandbox/`). |
| `POST` | `/api/tools/build` | Authenticated | Verifies `source` and `tests`, stages a `ToolPackage` (`pending_approval`), and parks a `tool_offer` decision (`201`). |
| `POST` | `/api/tools/:id/run` | Authenticated | Runs an approved tool or skill installed in the caller's suite (`404` if unpublished, `403` if not installed). |

### Test and "Would this help?" approval flow

1. Test with `POST /api/tools/test` (`params`, `source`, and at least one `ToolTestCase` with `name`, `setup_files`, `args`, `expect_ok`, `expect_output_contains`, `expect_files`). Returns `200` with `report` (`passed`, `total`, `all_passed`, `duration_ms`, `source_sha256`, `cases`), or `400` on compile errors or disallowed host calls.
2. Stage with `POST /api/tools/build` (`name`, `description`, `params`, `source`, `tests`, optional `thread_id`). If every test passes, Ergon saves the package as `pending_approval`, creates a `tool_offer` decision titled `"Would this help? New tool: <name>"`, and emits an `approval_parked` push notification. Before approval, `POST /api/tools/:id/run` returns `404`.
3. Settle on desktop or `/phone` via `POST /api/decisions/:id/approve` (or `deny`). On approval, Ergon re-checks the `source_sha256` digest and test report, publishes the tool to the shared company library, and installs it into the author's suite. Other employees can then install it via `POST /api/suite/install` or accept a proactive offer.

```bash
curl -s -X POST http://127.0.0.1:8787/api/tools/build \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Word Counter",
    "description": "Count words in a workspace file.",
    "params": ["path"],
    "source": "pub fn main(path) { let text = host_workspace_read(path); \"words=\" + text.split_whitespace().len().to_string() }",
    "tests": [{"name": "counts words", "setup_files": {"note.txt": "one two three"}, "args": ["note.txt"], "expect_ok": true, "expect_output_contains": "words=3"}]
  }'
```

```bash
curl -s -X POST http://127.0.0.1:8787/api/tools/word-counter/run \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"args":{"path":"note.txt"}}'
```

## Decisions, proactive offers, push alerts, and `/phone`

The mobile-first `/phone` surface (and `?face=phone`) opens on the `Decisions` tab and listens on `/api/runs/stream` for approval prompts, proactive offers, and run updates.

### Decisions

- `GET /api/decisions`: returns `{ "decisions": [...] }` for the caller's workspace.
- `POST /api/decisions/:id/approve`: settles a pending decision (`delete`, `connector_delivery`, `mcp_tool_call`, `budget_gate`, `tool_offer`, `checkpoint`, `security`, `question`) and executes its approved action.
- `POST /api/decisions/:id/deny`: rejects a pending decision without running the gated action. Returns `400` if already settled or unknown.

### Proactive offers ("Would this help?")

- `GET /api/offers`: refreshes cross-team library matches against the caller's recent finished work and returns `{ "offers": [...] }`.
- `POST /api/offers/analyze`: scans cross-team library tools/skills plus recent finished work (using the configured model when available) to generate deduplicated offers. If the model returns a `proposed_tool`, Ergon verifies and stages it as a `tool_offer` decision (`staged_tool`, `staged_decision_id`) and emits a notification.
- `POST /api/offers/:id/accept` or `POST /api/offers/:id/approve`: accepts a pending offer (`install_app`, `install_library_tool`, `build_tool`, or `repeat_task`), executes the installation or one-tap app creation, and returns `{ "offer": ..., "made": ... }`.
- `POST /api/offers/:id/dismiss` or `POST /api/offers/:id/deny`: marks a pending offer `dismissed` (`400` if already settled).

### Notifications and push subscriptions

- `GET /api/notifications`: returns `{ "notifications": [...], "unread_count": N }` newest first.
- `POST /api/notifications/:id/read`: marks a notification read (`404` if not found).
- `GET /api/push/subscriptions`: lists registered push subscriptions.
- `POST /api/push/subscribe`: registers `channel` (`"web_push"` or `"webhook"`), `endpoint` (`"browser://local"` or `https://...` for `web_push`; egress-checked URL for `webhook`), and `label` (`201`). Rejects loopback, private, and cloud metadata URLs with `400`.
- `DELETE /api/push/subscribe` (JSON `{"id": "..."}` or `?id=...`) and `DELETE /api/push/subscribe/:id`: removes a subscription (`404` if missing).

### Live SSE stream (`GET /api/runs/stream`)

Streams `text/event-stream` events for the authenticated caller's `person_id`.

- Query parameters: `thread_id` (filter to one thread), `since` (replay events with `seq > since`, or send `Last-Event-ID` header), `once=1` (flush initial status + matching replay events and close).
- Emitted event types: `status`, `run_started`, `step_start`, `token_delta`, `step_done`, `file_written`, `approval_parked`, `push_notification`, `run_cancelling`, `run_finished`, and `: heartbeat` comments every 5 seconds while idle.

```bash
# Scan work for proactive offers and approve an offer
curl -s -X POST http://127.0.0.1:8787/api/offers/analyze \
  -H "Authorization: Bearer $ERGON_KEY"

curl -s -X POST http://127.0.0.1:8787/api/offers/o123-1/approve \
  -H "Authorization: Bearer $ERGON_KEY"

# Listen to live SSE notifications
curl -N http://127.0.0.1:8787/api/runs/stream \
  -H "Authorization: Bearer $ERGON_KEY"
```

## Self-improvement and quality bars

Ergon tracks live execution outcomes and enforces two safeguards before updating any skill, tool, subagent, or improver prompt: replay-gated promotion and per-task quality bars.

### Outcome scoring and evolution loop

Outcome and version history live under `<root_data_dir>/evolution/` (`0600` permissions):

- `GET /api/evolution/scores`: returns per-artifact quality scores, active vs baseline versions, and recent evolution log entries.
- `GET /api/evolution/history/:id`: returns all versions, recorded outcomes, replay test cases, and audit log events for `artifact_id`.
- `POST /api/evolution/outcomes`: records a live outcome (`person_id`, `artifact_kind`, `artifact_id`, `artifact_version`, `run_id`, `task`, `approved`, `edits_needed`, `reruns`, `time_saved_mins`, optional `explicit_score` and `feedback`). Quality score starts from approval (`0.85` approved, `0.70` neutral, `0.15` denied), subtracts `0.15 * min(edits_needed, 4)` and `0.10 * min(reruns, 4)`, adds `clamp(time_saved_mins / 30.0, 0.0, 0.15)`, blends `0.6 * computed + 0.4 * explicit_score` when supplied, and clamps to `0.0..=1.0`. If an active promoted version has 2 or more live outcomes whose mean quality score drops at least `0.10` below `baseline_live_score` (or whose denial/failure rate reaches `0.50`), Ergon automatically rolls back to `previous_stable_version` and logs `auto_rolled_back`.
- `POST /api/evolution/cases`: adds a deterministic `ReplayCase` (`case_id`, `artifact_id`, `artifact_kind`, `setup_files`, `args`, `prompt_or_input`, `checks`) to `replay_cases.json`.
- `POST /api/evolution/propose`: runs the active `default-improver` strategy with the configured model against recent feedback and replay cases for `artifact_id` (or evaluates a submitted candidate), runs champion vs candidate across all replay cases, and promotes only when the candidate compiles, introduces zero regressions, and beats the champion by more than `0.01` across at least 1 replay case.
- `POST /api/evolution/rollback`: manually restores `previous_stable_version` for `artifact_id`.

```bash
curl -s -X POST http://127.0.0.1:8787/api/evolution/propose \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"artifact_id":"word-counter"}'
```

### Per-task quality bars

Per-person task quality bars live in `<person_data_dir>/quality_bars.json` (`0600` permissions). Each task normalizes to a `task_family` slug so repeated jobs (such as `"weekly client brief for acme"` and `"weekly client brief for beta"`) share a bar.

- `GET /api/quality-bars`: lists all task-family quality bars for the caller's workspace.
- `GET /api/quality-bars/:family`: returns the `QualityBar` record (`status`, `criteria`, `runs`, `best_score`, `accepted_run_id`, `accepted_recipe`).
- `POST /api/quality-bars/:family/feedback`: updates the bar with `action` (`"improve"` or `"accept_good_enough"`), optional `note`, `add_criteria`, and `recipe`.
  - In `improving` mode, future runs receive `QUALITY BAR (BEAT LAST RUN)` guidance with the prior score and learned criteria.
  - When `accept_good_enough` is posted, `status` locks to `accepted_good_enough` and emits a `quality_bar.accepted` webhook event. Future runs receive `DO NOT DRIFT` guidance and report `stable_at_bar: true` and `changes_stopped: true` without altering the accepted recipe.

```bash
curl -s -X POST http://127.0.0.1:8787/api/quality-bars/weekly-client-brief/feedback \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"action":"accept_good_enough","recipe":"3-bullet executive summary with [file:L##] citations"}'
```

## Inbound and outbound webhooks

### Inbound trigger webhooks (`POST /api/hooks/:hook_id`)

Inbound webhooks are exempt from browser `Origin` and `Sec-Fetch-Site` CSRF checks and authenticate using per-trigger secrets created via `POST /api/connectors/triggers` (`admin` only).

Authentication methods (evaluated in order):

1. `X-Ergon-Signature: sha256=<hex>`: HMAC-SHA256 using the trigger's `signing_secret` over the raw request body, or over `<X-Ergon-Timestamp>.<body>` when `X-Ergon-Timestamp` is present.
2. `X-Ergon-Hook-Token: <token>`: constant-time match against the trigger's `hook_token`.

Replay protection rejects requests when `X-Ergon-Timestamp` falls outside the allowed skew window or when a duplicate `X-Ergon-Delivery` (or timestamp + body hash) is seen. Accepted hooks return `202` with `{ accepted, trigger_id, trigger_name, trigger_file, routine_id, received_ms, run }`, save the envelope to `workspace/inbox/hook-<ts>-<seq>.json`, and launch the linked routine or pipeline.

```bash
BODY='{"invoice_id":"inv-402","amount":1250}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$HOOK_SECRET" -hex | awk '{print $2}')

curl -s -X POST http://127.0.0.1:8787/api/hooks/hk_abc123 \
  -H 'Content-Type: application/json' \
  -H "X-Ergon-Signature: sha256=$SIG" \
  -H "X-Ergon-Delivery: deliv-$(date +%s)" \
  -d "$BODY"
```

### Outbound webhook subscriptions

Requires the `webhooks` (or `admin`) scope. Target URLs pass the workspace SSRF egress guard (blocking loopback, private ranges, and cloud metadata endpoints unless explicitly allowed).

- `GET /api/webhooks/subscriptions`: lists registered webhook subscriptions (secrets redacted).
- `POST /api/webhooks/subscriptions`: creates a subscription (`url`, `events` array, optional `secret` and `label`; returns `201`).
- `DELETE /api/webhooks/subscriptions/:id` (or `DELETE /api/webhooks/subscriptions` with `{"id": "..."}`): deletes a subscription (`404` if not found).

Subscribed event types:

- `work.completed`
- `decision.created`
- `decision.settled`
- `tool.published`
- `evolution.promoted`
- `evolution.rolled_back`
- `quality_bar.accepted`

Every outbound delivery sends:

- `X-Ergon-Event`: event name (for example `decision.created`).
- `X-Ergon-Delivery`: unique delivery ID for deduplication.
- `X-Ergon-Signature`: `sha256=<hex>` HMAC-SHA256 digest of the raw JSON body signed with the subscription secret.

```bash
curl -s -X POST http://127.0.0.1:8787/api/webhooks/subscriptions \
  -H "Authorization: Bearer $ERGON_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://hooks.example.com/ergon","events":["decision.created","tool.published","quality_bar.accepted"],"label":"Ops Relay"}'
```

## Inbound MCP server (`POST /api/mcp/inbound`)

Ergon runs an inbound Model Context Protocol (MCP) server over JSON-RPC 2.0 at `POST /api/mcp/inbound`.

### Enabling and authenticating

1. Enable the inbound server from a loopback `admin` session via `POST /api/mcp/inbound/config` with `{"enabled": true}` for unauthenticated loopback or `ERGON_MCP_INBOUND_TOKEN` access (returns `-32001` / `403` when disabled). Requests authenticated with a valid `mcp` (or `admin`) scoped API key or an `admin` session are permitted automatically.
2. Authenticate using an `admin` session, a scoped API key with the `mcp` (or `admin`) scope, or a bearer token matching `ERGON_MCP_INBOUND_TOKEN` (`-32002` / `401` on mismatch).
3. Write tools also check caller scopes (`write`, `company_db`, or `tools`) and workspace RBAC, returning JSON-RPC error `-32003` (`HTTP 403`) when the caller lacks permission.

### JSON-RPC methods

- `initialize`: returns `protocolVersion` `"2024-11-05"`, server capabilities (`tools`, `resources`), and `serverInfo` (`ergon-inbound-mcp`).
- `notifications/initialized`: acknowledges client initialization (`{}`).
- `tools/list`: returns schemas for all permitted tools.
- `tools/call`: invokes a named tool with `params.name` and `params.arguments`, applying engagement isolation, secret redaction, and host-path redaction to outputs.
- `resources/list`: lists readable MCP resource URIs.
- `resources/read`: reads a resource by `params.uri`.

### Tools and resources

Read tools:

- `search_vault`: hybrid search across workspace files (`query`, optional `path_prefix`, `mode`, `limit`).
- `query_table`: deterministic filtering, grouping, and aggregation over workspace CSV/JSON/spreadsheet tables (`path`, `where`, `group_by`, `aggregates`, `sort`, `limit`).
- `company_db_query`: queries authorized tables in the company database (`table`, optional `query`, `limit`).
- `read_memory`: reads authorized shared company memory entries (`scope`, optional `category`).
- `list_tools`: lists shared company library tools and caller-installed tools.
- `get_decisions`: lists pending and settled decisions for the active workspace.

Write tools (require write permission; return `-32003` if caller has read-only access or fails RBAC):

- `write_workspace_file`: writes or updates a file inside the sandboxed workspace (`path`, `content`).
- `write_company_memory`: appends a `CompanyMemoryEntry` (`category`, `scope`, `content`; `admin`/`pm`/`ae`/`office` only, subject to secret/host-path checks).
- `run_tool`: executes an installed tool from the caller's suite (`id`, `args`).
- `create_task`: starts or queues a task thread in the caller's workspace (`message`, optional `thread_id`).

Resources (`resources/list` and `resources/read`):

- `ergon://company/memory`: authorized shared company memory entries (JSON).
- `ergon://company/tools`: shared company library tool catalog (JSON).
- `ergon://workspace/files`: file listing of the caller's isolated workspace (JSON).

```bash
# List available MCP tools
curl -s -X POST http://127.0.0.1:8787/api/mcp/inbound \
  -H "Authorization: Bearer $ERGON_MCP_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Read company memory resource
curl -s -X POST http://127.0.0.1:8787/api/mcp/inbound \
  -H "Authorization: Bearer $ERGON_MCP_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"ergon://company/memory"}}'
```
