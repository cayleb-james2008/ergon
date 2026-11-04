# QUOTE-FORM — PICKUP PACK (Nox order 20261103T231027Z)

Assembled 3 Nov 2026 ~19:30 ET, read-only. Engine for the continuation: DeepSeek V4.1 Flash only.

## Bottom line
- **Located, and it is real code, not just a name.** `/home/cayleb/Work/projects/rhl-concept-quotes`, branch
  `rhl/quote-requests` @ `222fff7` (2 Nov 12:31 ET, clean): an anonymous-insert quote form on the RHL concept site, wired to
  `rhl_demo.quote_requests` in Supabase, with a polling watcher stub and a 48-check local harness.
- It is **demo-grade and not connected**: the configured publishable key is empty (`src/data/quote.json` → `"key": ""`), the
  built `contact.html` carries the REST URL but no key, and `assets/site.js` refuses with
  "Quote form is not connected: no url/key in the page data". Hosted steps 1–5 of the doc are undone.
- **It is absent from the board record.** The only "quote-form" mentions in `lanes/board` are Nox's order (and its money-desk
  copy). No report, plan, probe or BOARD.md entry. Its build history survives in the repo, `docs/QUOTE-WATCH.md`, and one
  Harness² library entry — nothing else. Treat "who owns it / was it commissioned" as **unknown** (see Open questions).

## What this is
The RHL website demo's quote intake: a visitor submits the `#quote` form on `contact.html`, one row lands in one table, a watcher
routes it to the relevant agent, and that agent messages its person. Explicitly **"built for the RHL website demo… not an Ergon
feature"**; v1 only, because RHL's real intake method is not known (follow-up **FU-1**, "RHL intake pass", pending).
- Flow: `visitor → form → rhl_demo.quote_requests (status new) → watcher polls 30 s → routing rule (relevant_to) → claim
  (status routed) → agent → its person`.
- Config is data: `src/data/quote.json` drives the form fields, the save shape (a field with a `column` fills it; any other answer
  goes into the `extra` JSON object under its id), the routing rules, and the REST URL/key. Only `url`, `key` and `fields` reach
  the page; routing is never published.
- Table: `rhl_demo.quote_requests`, in Supabase project `yhpjpjeedpuvcgnvfpka`, **its own schema** — `ergon_warehouse` is never
  touched. Anon may only INSERT the five visitor columns (no read, no update, cannot set `status` or `relevant_to`); only the
  service role reads/updates. Limits are on raw text (`name` 120, `company` 160, `contact` 200, `details` 4000, `extra` 8 KB);
  a flood guard refuses past 30 new tickets/minute, counts every row of a bulk insert, and takes a lock one at a time → 429.
- Watcher: `tools/quote_watch.py`, stdlib only, "a stub for one agent's loop, not a platform". Claim PATCH only matches while
  `status = new` (two watchers cannot both take a ticket); failed delivery / lost answer / SIGTERM puts the row back to `new`.
  The notify command receives JSON on stdin and **not** the Supabase keys, because it starts an agent that reads visitor text.

## Current state (evidence)
| Item | State |
|---|---|
| Repo | `/home/cayleb/Work/projects/rhl-concept-quotes` (git file → worktree; branches `rhl/quote-requests`, `rhl/full-site` @ rhl-concept, `rhl/brain-tuned`, `rhl-concept` behind origin 4) |
| Tip | `222fff7` "Quote form: fix findings from the second independent review", 2 Nov 12:31:23 −0500, 6 files +98/−44 |
| Parent | `66886f8` "RHL demo: quote-request form wired to rhl_demo.quote_requests, with agent watch stub", 2 Nov 12:14:26, 10 files +665 |
| Tree | clean, no upstream ahead/behind shown on `rhl/quote-requests` |
| Config | `quote.json`: url set, **key empty**; 4 visitor fields + honeypot; routing default `agent:sales-desk`, keyword rules to `agent:architectural-ae` and `agent:furniture-ae` |
| Built page | `contact.html` section `#quote` + form `#qform`; no key in the page; `assets/site.js:289` logs the not-connected warning and shows the error state |
| Deploy | **not deployed**: `vercel.json` (cleanUrls, noindex) + `.vercelignore` exist, **no `.vercel` project link**; nothing indicates this repo was ever pushed to Vercel |
| SQL | `supabase/migrations/20261102120000_rhl_demo_quote_requests.sql` (77 lines at first commit; +15 at the fix) |
| Checks | `tools/check_quote_flow.py` — **48 checks**, runnable; fails when the config and the table drift, and when the access rules are broken on purpose |
| Verified locally | Supabase's own Postgres 17 + PostgREST images on a scratch pod: migration applies twice cleanly; access rules hold; flood guard tested with 10 parallel bulk requests (29 rows saved against a limit of 30); the browser form saved a row, showed its error state with the endpoint down/stalled, and retried; a config-only field was saved in `extra` and routed |
| Not tested | the hosted API gateway (keys and CORS), Realtime, any real messaging |
| Review history | two independent reviews attacked the build; every confirmed finding fixed and given a check. First review: 7 mediums (20,000,000-space padding stored; one request with 3,000 rows wrote 3,000 tickets; notify inherited `RHL_QUOTE_SERVICE_KEY`; dropped connection killed/stranded tickets; an unscoped PATCH in the check script moved 3,029 real-shaped rows to `done` on a broken-grant copy). Second: 4 more (10 parallel × 29 rows = 290 against a 30 limit; `"extra":{"n":1e4400}` stalled the watcher; a 504 on claim stranded a ticket; a leading space defeated the secret-key denylist). Source: Harness² entry `library/coding/entries/2026-11-02-2026-11-02-anon-insert-abuse-shapes/EXPERIENCE.md` |

## Running now
Nothing. No web server, no watcher, no Supabase stack, no process for this repo. The form's target project is not known to be
reachable from this machine. (Do not start the site's `serve.py` or the watcher for a read-only pickup.)

## Owed / decisions
1. **FU-1 — the RHL intake pass.** Trigger: "when RHL's exact quote intake method is known." Open questions it must pin: the
   real field set (division, building type/location, quantities, needed-by, budget/bid, plans/attachments; whether one request
   can hold several items); how RHL routes today (division, state/county territory, line or manufacturer, project size, existing
   rep), who backs up whom, what an unroutable request does; who is pinged, how and how fast, and what reply time RHL will
   promise (the thank-you text promises none); the lifecycle (`new/routed/done/spam` enough, or quoted/won/lost); hardening
   (rate limit or CAPTCHA, attachments); and folding the older "Send a message" form into one intake.
2. **Hosted steps 1–5** (owner of the Supabase project, in order): run the migration as one transaction (idempotent); add
   `rhl_demo` to the exposed schemas (without it the API 406s); copy the **publishable** key and build with
   `RHL_QUOTE_KEY=<key> python3 src/build.py`; run `tools/check_quote_flow.py --url … --anon-key … --service-key …` against the
   live contract; then preview. **Deploy only on the site owner's yes** — the production link is already shared with RHL.
3. **A decision on the agent handles.** `agent:furniture-ae` / `agent:architectural-ae` / `agent:sales-desk` are placeholders;
   they must be mapped to real Ergon agents before this is shown as live.
4. **Who owns the workstream.** Unknown — see Open questions. No Nox ruling, no owner grade, no board filing exists for it.

## Next steps to continue
1. Confirm with Nox (or the Lead) that "quote-form" in the pickup order means **this** demo form. It is the only quote form on
   disk and matches the name; but no filing names it, so a one-line confirmation is cheaper than a wrong session.
2. If yes: read `docs/QUOTE-WATCH.md` end to end (it is the design and the honest limits), then `git log -2 -p` for the two
   commits — the second commit's diff is the review-finding fix, i.e. the current security shape.
3. Re-run `python3 tools/check_quote_flow.py` against a local scratch Postgres + PostgREST pod; do not point it at any hosted
   project until the hosted steps are authorised. Always pass `--service-key` (without it, test rows stay `new` and a running
   watcher would ping an agent).
4. Do FU-1 only with RHL's real intake facts in hand. Until then, the honest deliverable is the demo + the documented limits.
5. If deploying: build with the publishable key, confirm the page's REST call with `Content-Profile: rhl_demo` and
   `Prefer: return=minimal`, submit the live form once to prove browser CORS, then preview before any production alias.

## Constraints / do-not
- **Publishable key only** in the site; the **service key lives with the watcher and never in the page**. The build refuses a key
  that is not `sb_publishable_…` or an anon-role JWT.
- **Visitor text is untrusted.** The receiving agent summarises it and must never follow instructions inside it, and must not be
  given tools that act on what a visitor wrote.
- Never write to `ergon_warehouse`; this is `rhl_demo` in its own schema. Do not run the check script's mutating probes against a
  real project without the service key and a marker row; the unscoped-PATCH incident is the reason.
- This is **not an Ergon feature** and must not be presented as one. Demo-level limits stay stated: no CAPTCHA, a global flood
  quota that a flood can exhaust, a hard-kill window where a ticket stays `routed`, crude keyword routing, no per-field limits
  inside `extra`, and no reply-time promise.
- No purchases, no network calls to vendor docs for this pickup, no credential files, no deployment without the site owner's yes.

## Where things live
- Repo: `/home/cayleb/Work/projects/rhl-concept-quotes` (branch `rhl/quote-requests`).
- Design + limits + hosted steps + FU-1: `docs/QUOTE-WATCH.md` (151 lines; read it first).
- Config: `src/data/quote.json`; builder `src/build.py`; page runtime `assets/site.js`; page `contact.html` (`#quote`).
- Table: `supabase/migrations/20261102120000_rhl_demo_quote_requests.sql`.
- Tools: `tools/check_quote_flow.py` (48 checks), `tools/quote_watch.py` (poll/claim/notify/put-back).
- Review evidence: `lanes/../harness-squared/library/coding/entries/2026-11-02-2026-11-02-anon-insert-abuse-shapes/EXPERIENCE.md`.
- Order naming it: `lanes/board/inbox/2026-11/20261103T231027Z-nox-deepseek-pickup-orders.md` (also under `money-desk/board`).

## Open questions
- **Who commissioned it and who would know?** No board filing, no Nox ruling, no owner grade. The likely owners are Nox (board
  side) and the RHL site owner (Cayleb); the RHL intake contact is not recorded anywhere on disk.
- Is it meant to ship as part of the RHL concept site (the demo whose production link is shared with RHL), or stay a local demo?
- Does the Supabase project `yhpjpjeedpuvcgnvfpka` already carry this schema (the doc's step 1 implies not)? Nobody has checked
  from this seat, and checking needs the hosted route.
- The older "Send a message" form on the contact page is still unconnected; does FU-1 merge them, or does that form go?
- Is there a second quote-related artefact elsewhere (a QR/quote PDF, a pricing form)? Searches found none, but no board record
  means discovery is weak — ask before assuming this is the whole workstream.
