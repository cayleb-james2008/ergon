# ERGON — quote form, corrected resubmission package #2 (filed 4 Nov 2026, ~06:10 UTC / 01:10 ET)

From: SLICE B of the ERGON day lead's fan-out (opencode-go/deepseek-v4.1-flash). One slice only: the RHL quote-request form's
corrected resubmission package. Board repo untouched apart from this report and the Nox note; nothing committed here.

**Package:** `~/Work/projects/rhl-concept-quotes`, commit **`29ac7ff`** (one commit on top of `cc42461`), tree clean.
**Read me first:** `docs/RESUBMISSION-20261104.md`. **Audit:** `docs/RESUBMISSION-20261103.md` §6.

## 1. Result

Agent-side complete and consistent: the package's headline claims all hold on disk, one real code-side gap was found and
closed (the build accepted any quote URL silently), three doc-side gaps in the authority document were fixed, and the
consistency is now machine-checkable (`tools/check_package.py`, proven able to fail). The six hosted steps remain Nox's; the
deploy remains the site owner's yes. **No hosted step was run: no hosted SQL, no migration, no `PATCH`, no deploy, no key
entry, no Supabase project write, no `ergon_warehouse` contact of any kind.**

## 2. What was wrong

The three governing invariants, checked one by one:

1. **Watcher model governs — code was right, the authority doc was not.** `tools/quote_watch.py` matches every lifecycle
   clause in `docs/QUOTE-WATCH.md` (poll at :108/:138, routing :57-67, start-up validation :70-76, claim :115-118, payload
   :81-82, put-back :97-132, headers :30). **No watcher code changed.** Four doc-side corrections:
   - the notify payload sample omitted the `warning` key the code sends (`quote_watch.py:81-82`) and the doc's own
     untrusted-text rule requires — an agent's parser written from the sample would have dropped the only guard there is;
   - the table's `contact` minimum (3 characters, `migration:34`) was undocumented;
   - "30 new tickets in a minute" read as `status = new`; the trigger counts tickets *created* in the minute whatever their
     status (`migration:63`);
   - "the migration applies twice" was stale — it has been applied three times.
   One unsourced number was removed: a "10 parallel bulk requests, 29 rows" result with no transcript in this repo or in
   `~/fleet-scratch/overnight/evidence/`. The evidenced result (31 rows in one request → 429, nothing saved) replaced it.
2. **Endpoint ref — already correct, now enforced.** `quote.json:2` and the built `contact.html:98` both carry
   `https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1`; `build.py:11` has no hardcoded host; a rebuild is byte-identical; no
   stale ref and no `localhost`/`127.0.0.1` in any page, script or config. **But nothing enforced it:** any `RHL_QUOTE_URL`,
   including the scratch host the recorded browser proof used (`http://127.0.0.1:3001`), built silently into the page. The
   build now stops unless the URL is this project's REST endpoint or a loopback host (`src/build.py:22-26`) — tested both
   ways, so the browser proof still runs.
3. **`rhl_demo` only — verified in code.** `site.js:295` sends `Content-Profile: rhl_demo` and no other profile; the watcher
   sends it on every call (`quote_watch.py:30`); the migration creates and grants only `rhl_demo` (`:18`, `:84-86`) and its
   only `ergon_warehouse` mention is the comment saying it is untouched (`:3`). No shipped file contains `ergon_warehouse`,
   `service_role` or `sb_secret`. The hosted instruction "append, keep the ones already there" was right, but it named the
   current list as `public,graphql_public,ergon_warehouse` with no hosted read behind it — a guess a literal operator could
   act on. The new package keeps the rule and drops the guess, and requires the current list to be recorded before editing.

Also checked, no change needed: field maxima match the CHECKs exactly (120/160/200/4000, `quote.json:5-8` vs `migration:32-35`)
and are now machine-compared; routing rules and default agree between `quote.json:10-16`, `quote_watch.py:57-76` and
`QUOTE-WATCH.md:53-57`; the flood-guard trade-off is stated as the first hardening item, not a feature
(`QUOTE-WATCH.md:145-152`, `migration:57-58`); the "not proved" list (hosted gateway/CORS, Realtime, real messaging) is still
accurate.

Other corrections: the evidence path in the old doc did not resolve (the transcripts lived in `~/fleet-scratch/overnight/`);
both cited logs are now in `evidence/` and `evidence` is excluded from the static deploy; "nine lines" → seven queries; a
tracked `.pyc` untracked; `.gitignore` added; the 3 Nov doc now points at the 4 Nov package for execution.

## 3. The machine check

`tools/check_package.py` (stdlib only, no network) fails when (a) the endpoint is not this project's REST URL in the config or
the built page, (b) a shipped file names `ergon_warehouse`, `service_role` or `sb_secret`, (c) the POST's `Content-Profile` is
not `rhl_demo` or another profile is present, (d) a form maximum differs from the migration's CHECK, (e) a doc or built page
names another Supabase ref, (f) the built page does not match `quote.json` (how a stale endpoint really ships), (g) the
published key is neither empty nor publishable.

Clean tree, run on the committed tree:

```
PASS  a1 quote.json url is the project REST endpoint
PASS  a2 built contact.html carries the same endpoint
PASS  b  no shipped file names ergon_warehouse, service_role or sb_secret
PASS  c1 site.js sends Content-Profile: rhl_demo
PASS  c2 site.js sends no other Content-Profile
PASS  c3 built pages load that same site.js
PASS  d  name: form max 120 == table max 120
PASS  d  company: form max 160 == table max 160
PASS  d  contact: form max 200 == table max 200
PASS  d  details: form max 4000 == table max 4000
PASS  e  docs/ and the built pages name only this project ref
PASS  f  built contact.html quote block matches quote.json
PASS  g  quote.json key is empty or publishable
PASS  g  contact.html key is empty or publishable

PACKAGE CONSISTENT
```

It can fail: on a `/tmp` copy mutated four ways (stale ref and a secret-shaped key in `quote.json`, `details` CHECK 4000→3999,
`Content-Profile`→`public` plus a `service_role` comment in `site.js`, another ref in `QUOTE-WATCH.md`) it reports **8 FAILED**
and exits 1; with a stale ref hand-edited into `contact.html` alone it reports **3 FAILED** (a2, e, f). The build refuses to
produce that page. The check also caught one of my own audit sentences quoting a ref-shaped test host — fixed rather than
whitelisted.

## 4. What Nox must run (details and outputs in `docs/RESUBMISSION-20261104.md` §2)

1. **Apply the migration** — `supabase/migrations/20261102120000_rhl_demo_quote_requests.sql`, one transaction, `rhl_demo`
   only. Idempotent (three applications proven). Expect zero `ERROR` lines.
2. **Exposed schemas** — append `rhl_demo`, keep every schema already listed, save. Without it the API answers 406.
3. **Build with the publishable key** — `RHL_QUOTE_KEY=<publishable> python3 src/build.py` → `built 44 pages`, only
   `contact.html` changes; then `python3 tools/check_package.py` → `PACKAGE CONSISTENT`. Hand back the publishable key only.
4. **Contract check** — `tools/check_quote_flow.py --url https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1 --anon-key <pub>
   --service-key <secret>` → **48/48**, `ALL CHECKS PASSED`, exit 0. ~7 test rows per run, closed at the end; >4 runs a minute
   trips the flood guard (wait a minute).
5. **Prove the exposed schema is live** — one POST with `Content-Profile: rhl_demo` + publishable key → `201`; the same with
   `Prefer: return=representation` → refused (anon has no SELECT). Close the `TEST-CHECK-…` rows.
6. **Preview; deploy only on the site owner's yes.** Nox previews, the owner says yes; the link is already shared with RHL.

## 5. What remains unproven

The hosted gateway accepting the key; hosted browser CORS from `https://rhl-concept.vercel.app`; the exposed-schema change
taking effect; the project's role set being the standard one; Realtime (unused) and real messaging (not used); the rollback's
raw transcript (the cascade output is quoted in the 3 Nov doc §3b; the transcript is not on disk). Only Nox's run closes the
first four.

**Evidence reuse, stated plainly:** the 48/48 harness and the three-times idempotency run were **not re-run** — they are the
recorded, independently checked results, now copied into the repo (`evidence/`) so the claim is traceable. The only new
executions in this slice are the static checks, the rebuild (byte-identical), and the mutated-copy demonstrations.

— SLICE B
