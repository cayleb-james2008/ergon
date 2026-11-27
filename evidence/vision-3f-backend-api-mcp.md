# Vision Step 3f E2E Proof — Scoped API Keys, Authenticated MCP, & HMAC Webhooks

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3f`

## Verified Checks
1. Created scoped API keys (`POST /api/keys`), enforced `403` when read-only key attempted writes.
2. Registered outbound webhook subscription (`POST /api/webhooks/subscriptions`) with encrypted signing secret (`0o600`, zero plaintext).
3. Authenticated MCP JSON-RPC 2.0 `write_workspace_file`, `write_company_memory`, and `resources/read` via `X-Ergon-Api-Key`.
4. Executed live Ollama `qwen2.5:3b` task using scoped API key and accepted quality bar.
5. Captured and cryptographically verified HMAC-SHA256 `X-Ergon-Signature` headers for `work.completed` and `quality_bar.accepted`.

## Log Excerpt
```text
Admin setup -> 201: person_id=alice-admin
Created scoped API keys: ro=ergk_FiReloJ..., mcp_rw=ergk_8sscL9o...
Read-only key blocked on POST /api/company/memory -> 403
Registered outbound webhook subscription id=whsub-1795792241919-0
MCP write_workspace_file('status.md') -> 200 ok
MCP write_company_memory -> 200 ok
MCP resources/read('ergon://company/memory') -> 200 ok
Live Ollama chat via scoped API key -> 200 (quality_bar family=read-status-md)
Verified HMAC-SHA256 outbound webhooks delivered: ['work.completed', 'quality_bar.accepted']
File modes verified: api_keys.json=0o600, webhook_subscriptions.json=0o600, zero plaintext secrets
```
