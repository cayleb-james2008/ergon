# Vision Step 3a E2E Proof — Company DB & Role-Scoped Company Memory

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3a`

## Verified Checks
1. Multi-user RBAC (`admin`, `pm`, `installer`): Installer blocked (`403`) on `POST /api/company/memory` and `GET /api/company/db?table=clients`.
2. Role & client scoping: Installer sees `company` + `role:installer` only, never `client:acme` or Alice's private `memory.md`.
3. Secret rejection (`400`) on `sk-...` content.
4. Live Ollama `qwen2.5:3b` agent runs answered from authorized company memory and queried `query_company_db`.
5. Restart persistence verified (`company_memory.json` mode `0o600`, 3 entries retained).

## Log Excerpt
```text
Admin setup -> 201: person_id=alice-admin
Created user carol_pm (pm) -> 201: person_id=carol-pm
Created user bob_inst (installer) -> 201: person_id=bob-installer
PM created client acme -> 201
Installer POST /api/company/memory -> 403 (RBAC blocked)
Installer GET /api/company/db?table=clients -> 403 (RBAC blocked)
PM wrote company memory [policy|company] -> 201 id=cmem-1795788874520-0
PM wrote company memory [procedure|role:installer] -> 201 id=cmem-1795788874529-1
PM wrote company memory [preference|client:acme] -> 201 id=cmem-1795788874539-2
Secret rejection -> 400: {'error': 'company memory cannot contain secrets or host paths'}
Installer GET /api/company/memory -> 200, count=2 (client:acme & private memory excluded)
Live Ollama qwen2.5:3b Installer chat -> 200: [{"content": "Call read_company_memory, then use write_file to save the exact payment terms and breaker panel photo rule to answer.txt, and include those exact facts in your summary.", "role": "user"}, {"content": "Tool: read_company_memory\n{\"count\":2,\"entries\":[{\"author_person_id\":\"carol-pm\",\"category\":\"policy\",\"content\":\"Standard payment terms are Net-30.\",\"created_ms\":1795788874520,\"id\":\"cmem-1795788874520-0\",\"scope\":\"company\",\"updated_ms\":1795788874520},{\"author_person_id\":\"carol-pm\",\"category\":\"procedure\",\"content\":\"Always take two breaker panel photos before site sign-off.\",\"created_ms\":1795788874529,\"id\":\"cmem-1795788874529-1\",\"scope\":\"role:installer\",\"updated_ms\":1795788874529}],\"role\":\"installer\"}", "role": "step"}, {"content": "Tool: write_file\nerror: write_file needs args.path", "role": "step"}, {"content": "Payment ter
Live Ollama qwen2.5:3b PM chat (query_company_db) -> 200: [{"content": "Call query_company_db with {\"table\": \"roles\"}, then write the role names to roles.txt using write_file.", "role": "user"}, {"content": "Tool: query_company_db\n{\"limit\":50,\"rows\":[{\"can_manage_clients\":true,\"can_manage_invoices\":true,\"can_manage_workspace_policy\":true,\"can_view_invoices\":true,\"description\":\"Full workspace administration, user management, and cross-team visibility.\",\"is_admin\":true,\"role\":\"admin\"},{\"can_manage_clients\":true,\"can_manage_invoices\":false,\"can_manage_workspace_policy\":true,\"can_view_invoices\":true,\"description\":\"Manage clients, engagements, workspace policies, and shared company memory.\",\"is_admin\":false,\"role\":\"pm\"},{\"can_\n\u2026[truncated]", "role": "step"}, {"content": "Tool: query_company_db\n{\"limit\":50,\"rows\":[{\"can_manage_clients\":true,\"can_manage_invoices\":true,\"can_manage_workspace_
Restart persistence verified: admin sees 3 entries, pm sees 2 entries, file mode=0o600
```
