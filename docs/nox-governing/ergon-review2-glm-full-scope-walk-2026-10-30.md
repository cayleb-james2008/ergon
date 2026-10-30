# Review 2 (GLM / Z.ai) — FULL-SCOPE instalment: Nox's walk + triage

Delivered by Cayleb 30 Oct 2026 ~05:02 ET as `Ergon-Full-Scope-Code-Review_3_rz5j.pdf`. Scope: the whole `cayleb-james2008/ergon` repo at commit **8218532** (605 files, ~125k lines, 182 test files) plus the fix branch below. **46 findings: 5 High, 16 Medium, 13 Low, 12 nits.** Reviewer's verdict: the engineering core (signed update chain, budget/admission, connectors + credential layer, control-plane server, documents engine) is elite; the periphery (test runner, store load paths, docs that outrun code, CLI polish) is not — and every High fix is mechanical, on boundaries that already exist.

Scope caveat (Nox): the review read the pre-pivot tree — its framing is still OpenClaw-era. Findings in the suite apps, library and onboarding carry straight into the DSH structure (the suite is the product core either way). Installer/legacy-phone/OpenClaw-surface items get re-checked against the DSH build before effort is spent — do not fix surfaces the pivot is retiring without confirming they survive.

## Delivered in the pass — Parts Handbook fixes (VERIFIED PRESENT by Nox)

GLM did not just report the prior bounded review's findings — it implemented all five plus the three nits on branch **`review/parts-handbook-fixes`**, head commit **b000ab541792586fca2c3864c45e6d72450530cf** (Nox verified via the GitHub connector 30 Oct ~05:05 ET: branch and commit exist on origin; the commit message matches the PDF's section 10 claim by claim).

- F-1: intake required at create(), stored with the draft, validateCoverage in create() + revise(), renderHandbook gates on coverage; intake schema now round-trippable; latest() carries the intake.
- F-2: scored token-based KB lookup — longest containment wins, a tie is a miss, partial word never matches; the KALLAX-with-doors acceptance case passes per the commit message.
- F-3: src/limits.js bounds all stored inputs, rejections name field + limit. F-4: KB + draft-store load paths validate and refuse loudly, naming the file. F-5: renderer tolerates a missing uses array and renders an explicit "unknown source" marker.
- Claimed test state: parts-handbook **75/75** (up from 39), sibling library suite **37/37**.

Status: **independent checker verification is running** (a fresh checker who did not build it re-runs both suites and exercises the F-1 create/revise/render paths and the F-2 doors/ambiguity cases). The branch merges only after that check passes — maker ≠ checker, same as every other gate. Note this also covers the F-1/F-2 fix Cayleb accepted as an Idea on 30 Oct ~04:56 — GLM's delivery is broader (F-1..F-5 + nits) and lands through the same gate.

## The High findings (plain names)

- **X-1 — the "full" test runner is not full.** scripts/test-all.mjs wires in only budget-proxy of the suite apps, and only 4 of its 8 test files; 180 passing tests across six apps are silently excluded; npm test is narrower still; there is no CI anywhere. Budget-proxy's own bare `node --test` also trips over its 3 lock-race helper scripts and reports 101 pass / 3 fail on a healthy tree. suite/README.md's "214 tests" total is stale (280 without parts-handbook, 355 with). GLM's sequencing note, endorsed: **fix X-1 first** — every other fix's acceptance test is only trustworthy once the runner runs the suites.
- **X-2 — every suite store trusts whatever is on disk.** Bare JSON.parse at load in the library registry (worst member: a nameless record is served keyed `null`; a file/content name mismatch resurrects a ghost record on every load), baseline inventory (three parse paths), onboarding protocol, shell session store. One truncated or hand-edited file bricks the service with a raw stack trace. Fix shape already exists in-house: parts-handbook's new load validation (validate on load, name the file, refuse loudly) ported via one shared helper + a corrupt-file test per store.
- **L-2 — the library's 413 is never delivered.** server.js rejects then immediately destroys the socket, so an oversized body gets ECONNRESET, not payload_too_large — a well-behaved client retries the oversized POST forever. Respond first, destroy next tick; refuse an oversized declared length before reading.
- **L-3 — a malformed %-path throws a 500 before auth.** decodeURIComponent on raw path segments throws URIError on a lone truncated percent-sequence (and double-decodes, since URL parsing already decoded once). Wrap per segment → 400, or drop the manual decode.
- **O-1 — onboarding filename collision silently overwrites another employee's session.** The id sanitiser maps emp:gio, emp..gio, emp/gio and "emp gio" all to emp_gio.json; the second write wins on disk. Onboarding is the system of record for what an employee confirmed — this is the worst single data-loss finding in the report. One-line fix class: reject an id whose sanitised form collides, or key files by a hash of the raw id.
- (SC-1 in the installer section is X-1 restated; B-1 and SH-1 are X-2 members.)

## The "owner decision" — SC-4 — is already answered by Cayleb's own ruling

SC-4: two same-day 23 Oct documents state opposite model policies — SUITE-TEMPLATE.md declares MiniMax M3.1 Flash the only model, while setup.mjs + README/PRODUCT/capabilities.json enforce Haiku-default + Opus-expert. GLM is right that only the owner can say which is current — and the owner already did: **Cayleb's 29 Oct ruling — the Ergon build runs Haiku through Claude (Anthropic provider); the MiniMax-only lock is retired for this platform unless he reinstates it.** So the Haiku side is the current truth and the fix is doc alignment (one transitional sentence in SUITE-TEMPLATE.md; README already agrees with the code). No new decision is needed; if Cayleb wants MiniMax reinstated anywhere, he says so and the docs flip instead. The same pattern resolves O-4's retention contradiction: the 30-day post-confirmation retention was DECIDED 23 Oct (PROTOCOL.md) — the work is to implement to the decided story (or mark Pause/Withdraw honestly as not-yet-implemented in all three documents), not to re-decide it.

## Triage — where this lands in the existing queues

This is Review 2's full instalment; it joins Workstream 2 item 2 (code-review implementation) alongside Review 1's confirmed answers (S1 + S2 in full). Dedupes to honour — one fix, not two:

- Phone legacy module: Review 1 S1#2/#3 (promote the webhook, wire the plugin to the broker) ↔ GLM SC-2 (live code hiding in legacy-twilio-service.mjs) — the rename in Cayleb's confirmed Q3 answer (rhl-phone → product-neutral, broker owns the service) subsumes SC-2.
- Load-validation class: Review 1 S1#5 (private-path enforcement consolidation) is adjacent but distinct; GLM X-2 is its own programme (shared load helper, four stores).
- Parts Handbook F-1..F-5: DONE by GLM on the branch (see above) — remove from the implementation queue; what remains is the checker verdict + merge.

Recommended order (GLM's roadmap, endorsed): **P0** — X-1 runner honesty first, then X-2 load helper + corrupt-file tests, O-1, L-2, L-3 (GLM estimates 1–2 afternoons). **P1** — invariant truth: O-3 validate-before-mutate, B-2 stop rewriting committed snapshots (or amend the promise — Nox recommends stop rewriting; snapshots are the audit trail), L-4 one lifecycle rulebook, L-5 connections in the manifest validator, O-4/SC-4 doc alignment per the rulings above, SC-2/SC-3, X-3 machine codes, SC-5 error-code allowlist, L-6 self-contained 409 test (2–3 afternoons). **P2** — craft pass in passing: exit-code table, minimal lint/CI, X-6 evidence freshness, SC-6..SC-11, L-7a–d, O-5, BL-2, PY-1/2, SV-1.

## What holds up (so the remediation doesn't disturb it)

Per §9, each read in full: the filesystem defence discipline (~4,000 lines share it without drift); the signed update chain end to end; money + admission engineered to banking standard (Decimal-only, freeze-on-breach, "never discard a real bill to preserve an artificial budget invariant"); network egress defended as a class (own DNS resolution, full private-range denial, address pinning); and the documentation culture where it holds. The operator plane (section 5) produced NO High or Medium findings at all; the Python services produced two nits.
