# Vision Step 3c E2E Proof — Proactive Offers, Phone Surface & Push Notifications

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3c`

## Verified Checks
1. `GET /phone` serves the `/phone` HTML shell (`200`).
2. Push subscription (`browser://local`) persisted (`201`) and metadata SSRF webhook blocked (`400`).
3. Live `POST /api/offers/analyze` with Ollama `qwen2.5:3b` generated cross-team offer(s) and fired real-time `event: push_notification` over `GET /api/runs/stream`.
4. `POST /api/offers/:id/approve` alias installed the offered shared tool into Bob's suite.
5. `GET /api/notifications` + `POST /api/notifications/:id/read` tracked and cleared unread state.

## Log Excerpt
```text
GET /phone -> 200 (199476 bytes)
POST /api/push/subscribe (browser://local) -> 201 id=psub-1795788948373-0
Push SSRF metadata URL rejected -> 400
Alice published shared tool invoice-parser
POST /api/offers/analyze (live Ollama qwen2.5:3b) -> 200: generated=2, proposed_tool=False, staged_tool=False, staged_error=None
Analysis payload: {"all_offers": [{"author": "alice", "detail": "Shared by alice \u2014 Parses vendor invoice receipts and totals. Matched 3 recent finished jobs; add it to your suite?", "id": "o1795788948576-2", "item_id": "invoice-parser", "key": "libtool:invoice-parser", "kind": "install_library_tool", "status": "pending", "title": "Would this help? Use Invoice Parser (tool)", "ts": 1795788948576}, {"detail": "Automate the parsing of multiple vendor invoice batches with a single command.", "id": "o1795788955957-4", "key": "invoice-parser-batch", "kind": "repeat_task", "status": "pending", "template": "Parse vendor invoice parser batch {batch_number} using the new helper.", "title": "Would this help? Parse multiple vendor invoice batches", "ts": 1795788955957}], "generated": [{"author": "alice", "detail":
Live SSE push_notification delivered (1 event captured)
Approved offer o1795788948576-2 via /approve alias -> made={'installed': 'invoice-parser'}
GET /api/notifications -> unread=2; marked notif-1795788955958-5 read
```
