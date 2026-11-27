# Multi-Employee Compressed-Week Simulation (4 Roles, 5 Workdays)

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/release/ergon` (`ergon 0.40.0`, release LTO build)
- Model: `qwen2.5:3b` via `http://127.0.0.1:11434/v1`
- Employees: **Alice** (`admin`), **Bob** (`pm`), **Cara** (`ae`), **Dan** (`installer`)
- Screenshots:
  - [sim-bob-phone.png](../screenshots/sim-bob-phone.png) (Bob `/phone` pending `Would this help?` tool approval card)
  - [sim-bob-phone-approved.png](../screenshots/sim-bob-phone-approved.png) (Bob `/phone` post-approval state)
  - [sim-cara-phone-delete.png](../screenshots/sim-cara-phone-delete.png) (Cara `/phone` consequential `delete_file` decision card)

## Day-by-Day Results
- **Day 1 (Company DB & Privacy Isolation)**: Alice wrote company/role/client memory and Acme Corp records; Bob & Cara saved private `memory.md` preferences via `qwen2.5:3b`; Dan received `403` on company-memory writes, client creation, and invoicing; zero cross-person or cross-client leaks in `/api/company/db`, `/api/company/memory`, or agent prompts.
- **Day 2 (Proactive Offers & Real Tool Builder)**: Bob completed checklist jobs, ran live Ollama `POST /api/offers/analyze`, built `checklist-verifier` (`2/2` sandbox tests passing), and received real-time `push_notification` SSE + `Would this help?` card.
- **Day 3 (/phone Approve -> Cross-Employee Sharing -> Consequential Gate)**: Bob approved `checklist-verifier` on `/phone` (Playwright), publishing it to Company Library and running it; Dan installed `checklist-verifier` from Company Library and ran it on Dan's workspace (`checklist_lines=4 status=verified`); Cara triggered `delete_file` and denied it on `/phone`, preserving `cara_obsolete_draft.txt`.
- **Day 4 (Self-Improvement Loop + Auto-Rollback + Meta-Improver)**: Recorded approvals, file edits via `PUT /api/files/content`, and reruns; live Ollama `qwen2.5:3b` promoted `greeter-badge` `v2` over `v1`; worse `v3` rejected (`regressions > 0`); live negative regression outcomes triggered auto-rollback to `v1`; `default-improver` `v2` promoted over `v1`.
- **Day 5 (Quality Bar + Scoped APIs / MCP / Webhooks)**: Cara's recurring brief improved from Run 1 -> Run 2 (`delta > 0`), locked `accept_good_enough`, and held steady on Run 3 (`stable_at_bar: true, changes_stopped: true`); external client authenticated with scoped `ergk_*` key, called MCP `company_db_query` + `run_tool`, and received HMAC-SHA256 verified `work.completed` & `quality_bar.accepted` webhook deliveries.

## Execution Log
```text
[10:46:17] Setup 4 employees: {'alice': 'alice-admin', 'bob': 'bob-pm', 'cara': 'cara-ae', 'dan': 'dan-installer'}
[10:46:17] === DAY 1: Company DB & Privacy Isolation ===
[10:46:40] Bob & Cara saved private memory.md preferences via live Ollama qwen2.5:3b
[10:46:40] Dan (installer) RBAC verified: 403 on company/memory write, invoicing, and client creation
[10:46:54] Day 1 PASS: Zero cross-person or cross-client leaks in /api/company/db, /api/company/memory, and prompts
[10:46:54] === DAY 2: Proactive Offers & Real Tool Builder ===
[10:47:14] Bob POST /api/offers/analyze (real Ollama) -> generated=1, all_offers=1
[10:47:14] Built & staged tool 'checklist-verifier' (pkg=tp-checklist-verifier-1795794434644-0, decision=d1795794434644-0, sandbox tests=2/2 PASS)
[10:47:16] Day 2 PASS: push_notification SSE arrived (2 event(s)) & 'Would this help?' card staged
[10:47:16] === DAY 3: /phone Approve -> Cross-Employee Sharing -> Consequential Gate ===
[10:47:18] Bob approved 'checklist-verifier' on /phone (screenshots: sim-bob-phone.png, sim-bob-phone-approved.png)
[10:47:18] Bob ran 'checklist-verifier' -> checklist_lines=1 status=verified
[10:47:18] Dan installed 'checklist-verifier' from Company Library and ran on Dan's workspace -> checklist_lines=4 status=verified
[10:47:26] Day 3 PASS: Cara denied delete_file (d1795794442821-1) on /phone; file preserved (screenshot: sim-cara-phone-delete.png)
[10:47:26] === DAY 4: Self-Improvement Loop + Auto-Rollback + Meta-Improver ===
[10:47:28] Ollama v2 PROMOTED: v1 -> v2 (score 0.67 -> 1.00)
[10:47:29] Worse v3 REJECTED: regressions=2
[10:47:29] Auto-rollback triggered on v2 -> v1: auto-rolled back v2 -> v1: live mean=0.000 (baseline=0.850), denial_rate=1.00 across 2 runs
[10:47:43] Day 4 PASS: Meta-improver default-improver promoted v1 -> v3
[10:47:43] === DAY 5: Quality Bar + Scoped APIs / MCP / Webhooks ===
[10:48:07] Cara Quality Bar Run 1 -> family=weekly-client-brief, score=0.65
[10:48:18] Cara Quality Bar Run 2 -> score=1.0 (delta=+0.35)
[10:48:30] Cara Quality Bar Run 3 -> status=accepted_good_enough, stable_at_bar=True, changes_stopped=True
[10:48:51] Day 5 PASS: MCP company_db_query + run_tool ok; HMAC-SHA256 webhooks verified: ['work.completed', 'quality_bar.accepted']
```
