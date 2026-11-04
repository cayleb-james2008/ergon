# ERGON → Nox: the RHL quote-form package is ready — six hosted steps, all yours

From: ERGON SLICE B, 4 Nov 2026, 11:05Z. Repo `~/Work/projects/rhl-concept-quotes`, commit **`29ac7ff`**, tree clean.
Execute from **`docs/RESUBMISSION-20261104.md`** (standalone; the 3 Nov doc is history, audited in its §6). Authority for
behaviour stays `docs/QUOTE-WATCH.md`.

**Nothing hosted was touched from this seat:** no hosted SQL, no migration, no `PATCH`, no deploy, no key entry, no Supabase
project write, and no `ergon_warehouse` contact of any kind. The 48/48 and the three-times idempotency results are **reused
evidence, not re-run**; their logs are now in the repo's `evidence/`.

Project: ref `yhpjpjeedpuvcgnvfpka`, schema **`rhl_demo` only**. The migration is not applied anywhere yet.

## The six steps

- [ ] **1. Apply the migration** — `supabase/migrations/20261102120000_rhl_demo_quote_requests.sql`, the whole file (one
      transaction). Idempotent: applied three times, no error, no duplicate object. Compare: zero `ERROR` lines; then 1 table,
      7 CHECK constraints, 1 trigger, 1 policy, 3 indexes.
- [ ] **2. Exposed schemas** — Project Settings → Data API → **append** `rhl_demo`, keep every schema already listed, save.
      Do not remove any entry (record the current list first). Without this the API answers 406 for the schema.
- [ ] **3. Build the page with the publishable key** — `RHL_QUOTE_KEY=<publishable> python3 src/build.py` → `built 44 pages`;
      only `contact.html` changes; then `python3 tools/check_package.py` → `PACKAGE CONSISTENT`. Hand back the **publishable**
      key only (the build refuses a secret key and refuses any endpoint but this project's).
- [ ] **4. Contract check** — `python3 tools/check_quote_flow.py --url https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1
      --anon-key <publishable> --service-key <secret>` → **48/48**, last line `ALL CHECKS PASSED`, exit 0. It inserts ~7
      `TEST-CHECK-…` rows and closes them; always pass `--service-key`; >4 runs in a minute trips the flood guard (wait a minute).
- [ ] **5. Prove the exposed schema is live** — one `POST /rest/v1/quote_requests` with `Content-Profile: rhl_demo` and the
      publishable key → **201**; the same with `Prefer: return=representation` → **refused** (anon has no SELECT). Close the
      test rows with the service key (`PATCH ?details=like.TEST-CHECK-*` → `status=done`).
- [ ] **6. Preview, then the deploy gate** — the production RHL link is already shared with RHL: deploy only on the **site
      owner's yes**. Not yours and not mine.

## Two traps (in the doc, §5)

1. The migration's `revoke … from public, anon, authenticated` line needs those roles. On a bare `postgres:17` it stops with
   `role "anon" does not exist` — that is the target, not a defect. Run it on the Supabase project; do not delete the revokes.
2. `Prefer: return=representation` is refused for anon **by design** (reading the row back needs SELECT). A 401/403 there is the
   access model answering; the 201 from the minimal-return request is the proof.

## What only your run can prove

That the hosted gateway accepts the key, hosted browser CORS from the real origin, that the exposed-schema change took effect,
and that the project's roles are the standard set. Realtime and real messaging are not used, so nothing about them is claimed.

## What this pass changed in the package

One real code gap: the build accepted any quote URL, including a scratch host, silently — it now refuses anything but this
project's endpoint or a loopback host (`src/build.py:22-26`). Three doc corrections in `QUOTE-WATCH.md`: the notify payload
sample now includes the `warning` key the code sends, the `contact` 3-character minimum is stated, and the flood guard is
described as counting tickets created in the minute (not `status = new`). The hosted step no longer names a guessed
Exposed-schemas list. `tools/check_package.py` now fails on a wrong ref, a warehouse/secret string in a shipped file, a
`Content-Profile` other than `rhl_demo`, form maxima that differ from the CHECKs, or a stale build — proven to fail on mutated
copies, clean on the real tree.

— ERGON SLICE B
