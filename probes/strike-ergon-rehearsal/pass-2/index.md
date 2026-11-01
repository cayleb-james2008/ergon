# Ergon dress rehearsal — pass 2 (installed @cayleb/dsh-fleet platform)

Generated: 2026-11-01T22:31:51Z · Driver: `bash docs/acceptance/demo-company/rehearsal/run-rehearsal.sh --pass 2 --out <pack>`

## Run metadata

| | |
|---|---|
| Pass | 2 |
| Host | outis-desktop (the host the platform is installed on) |
| Fresh company root | `/tmp/ergon-rehearsal-20261101T223149Z-GWu5wW` (mktemp, fresh for this pass) |
| Repo pin | `003d9ef2265af313d4fbde796b1827dbe5b72b5f` on `strike/ergon-rehearsal` |
| Seed | 20261102 (fixture default) |
| Installed package identity | `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` (`cayleb-dsh-fleet-0.2.2.tgz`) |
| Started / finished (UTC) | 2026-11-01T22:31:49Z / 2026-11-01T22:31:51Z |
| Verdicts | clean 4 · pending 4 · failed 0 |

## Verdict table

<!-- VERDICT-TABLE-START -->
| Step | What it covers | Word | Named piece / notes |
|---|---|---|---|
| R0 | Install proof (read-only): control socket, fleet-dsh unit, installed package identity, repo pin, r0-preflight | clean | read-only install proof: control socket status/integrity/policy, fleet-dsh unit, installed package identity digest + authorised wrapper, repo pin, repo r0-preflight. Dedicated fresh host/DNS/Supabase are owner acts, named out of scope. |
| R1 | Company setup into a fresh mktemp root; canonical state dump hashed | clean | fixture company/cast/seed stood up in a fresh mktemp root through the product's own modules; canonical state dump hash matches the pinned digest in both passes. The installer first-run UI is not in the installed plugin (stand-in named). |
| R2 | First personalised assistants: fixture record + real fleet agent sessions | pending | fixture profile + onboarding session per cast member; onboarding driven to active through the product protocol with fixture answers (stand-in for the employee's own words); one real fleet agent session per cast member on the installed plugin. The model-quality assistant leg is PENDING: no provider credential (owner act). |
| R3 | Shareable non-localhost URL from a second device | pending | company host origin recorded from the fixture config and shape-checked; the fixture origin is a made-up name that does not resolve and the host binds 127.0.0.1 only. Hosted non-localhost URL + DNS + second-device reachability PENDING (not built; owner acts out of scope). |
| R4 | Phone enrolment | pending | host-side device registry row for the warehouse lead's phone recorded as a stand-in (operator-registered, approved). The phone surface is PACKAGE-PENDING (no phone client; a phone cannot use the fleet join — F5), so the step is pending with the missing piece named. |
| R5 | Desk task: ticket round trip + a real warehouse-board task as a fleet order | clean | scenario (a)+(b) ticket round trip through the product's control service (ids, states, receipts, device readback, recorded product limits F3/F4/F6) AND a real warehouse-board-shaped task as a fleet order on the installed plugin (create -> decompose -> claim -> built -> check -> verdict -> bank -> done). Stand-ins named: Get Help/phone UI, scripted board. |
| R6 | Voice / message flow | pending | voice/phone surface PACKAGE-PENDING (P20-F5): no voice button, dictation client or phone surface on the installed plugin. Supporting fixture check only: the cast's written voices separate blind; that is not the voice surface and the step stays pending. |
| R7 | Close-out: state record, teardown, and the second from-scratch pass (pass-2 diff) | clean | state record (company/cast/tickets/board/order/digests) and teardown of the fresh company root; the second from-scratch pass runs R0-R7 into a new root and its verdict table is diffed (R7 is clean only when the tables match). |
<!-- VERDICT-TABLE-END -->

**Words:** `clean` 4 · `pending` (named, counted) 4 · `failed` 0. A pending step is a PACKAGE-PENDING piece named here — it is never recorded clean and never skipped.

## Pending named (counted)

| Step | Named missing piece |
|---|---|
| R2 | provider credential for the fleet profile (owner act): the assistant's model-quality role answer and the model-conducted onboarding conversation cannot run. The deterministic stub route produced no provider call on this install (stub-turn probes recorded). What ran instead: fixture assistant records, onboarding to `active` through the product protocol with fixture answers (stand-in for the employee's own words), and one real fleet agent session per cast member. |
| R3 | hosted shareable non-localhost URL + DNS name + second-device reachability: not built in this strike. The fixture origin (https://lf-host-01.example-tailnet.ts.net:8443) is a made-up name that never resolves, the current host binds 127.0.0.1 only, and a dedicated host/DNS name are owner acts out of scope (no purchase, no DNS work). Device-approval via a second device therefore cannot be shown; the host-side device registry is covered as a stand-in under R4. |
| R4 | phone surface (PACKAGE-PENDING: no phone client; a phone cannot use the fleet join — SCENARIOS.md F5 / package boundary item 6): "Add this phone" and the manager approval on the phone cannot be shown, and the desktop-off answer test needs the host agent runtime/model credential. Host-side device registry row for the phone recorded as a stand-in (operator-registered, approved). |
| R6 | voice/message flow — voice surface PACKAGE-PENDING (P20-F5): no voice button, dictation client, transcript round trip or voice-control badge exists on the installed plugin; the Windows-only surfaces are stand-in/pending, never clean. Only the fixture's written-voices check ran (labelled: not the voice surface). |

## Out of scope for this strike (named, not silently skipped)

- Dedicated fresh cloud host, DNS name and company Supabase project — owner acts (R0-PROVISIONING P1/P4/P5); this strike runs on the installed platform host and does no purchase, DNS or Supabase work. Recorded in R0 evidence: the generic-host preflight's only blocking item is `project-id.txt`.
- A real model provider credential for the fleet profile — owner act. Every model-quality leg is PENDING, named at R2; the deterministic stub route is the only model-adjacent route and it produced no provider call on this install (recorded in R2 findings).
- The ruled DSH-era Ergon product package (installer first-run UI) — absent from the pinned tree; company setup is the fixture's `setup.mjs` (named stand-in, R1).
- Windows-only surfaces (Get Help window, native ReviewDesk), the phone surface and the voice surface — PACKAGE-PENDING (R4, R6); stand-ins are named, never counted clean.
- The warehouse-board app source (only on the work PC, RHLCO-WKS-09) — the fleet order at R5 carries a board-shaped task; the board state itself is the fixture's scripted state (named stand-in).

## Secrets check

Result: **clean**. The driver scanned its own evidence for key-like material (provider keys, JWTs, bearer tokens, private keys, named secret assignments, credential-URL connection strings) and failed the run on any hit. The scan report is `secrets-scan.txt`; a second, independent scan over the finished pass (including this index) is written outside the pass directory and its exit code gates the driver.

```
# Secrets scan — rehearsal evidence pack

- Directory: `/home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2`
- Files scanned: 71
- Patterns checked: 11
- Result: clean (0 hits)
```

## Checks per step

### R0 — Pre-pass install proof (read-only) → **clean**
- PASS control socket answers status — name=fleet-core protocol=1 pid=427587
- PASS fleet-core identity — name=fleet-core protocol=1 node=v26.12.0
- PASS lanes allowed — lanes_allowed=true
- PASS integrity ok (chain, sandbox, rows, guard) — integrity.ok=true chain.ok=true verified_at_boot=42154 sandbox.ok=true rows.ok=true guard.ok=true
- PASS control peer is uid 1000 (normal user, no root) — peer_uid=1000 accepted=906 rejected=0
- PASS selftest flag on (fixture:true orders and selftest sessions are gated by it) — policy.selftest=true source=default
- PASS policy-get answers — rows=19
- PASS policy pins spend ceiling + selftest — spend.hard_ceiling_usd=5 selftest=true
- PASS fleet-dsh unit active — is-active=active ActiveState=active SubState=running MainPID=427587
- PASS unit runs as user fleet — User=fleet Group=fleet
- PASS unit is contained (ProtectSystem strict, NoNewPrivileges, egress IP deny) — ProtectSystem=strict NoNewPrivileges=yes IPAddressDeny=169.254.0.0/16 100.64.0.0/10 ::/0 192.168.0.0/16 172.16.0.0/12 10.0.0.0/8 0.0.0.0/8
- PASS installed release identity recorded from the install's own audit line — cayleb-dsh-fleet-0.2.2.tgz sha256=37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177
- PASS the installed release tarball is present and hashes to the installed digest — cayleb-dsh-fleet-0.2.2.tgz sha256=37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177 expected=37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177
- PASS installed release version recorded from the installed release name (not assumed) — version=0.2.2
- PASS drop directory holds no ambiguity about what is installed (staged releases enumerated) — cayleb-dsh-fleet-0.3.0.tgz=b60019a0f361…, cayleb-dsh-fleet-0.2.2.tgz=37e3a542a32f…, cayleb-dsh-fleet-0.2.1.tgz=55516f9ae953…, cayleb-dsh-fleet-0.2.0.tgz=842e76f258fc…
- PASS wrapper version runs read-only and reports a DSH version — exit=0 version=0.2.2-alpha.2
- PASS wrapper status confirms the fleet install — exit=0 (uid/version lines present)
- PASS dump-config exits 0 and mounts the fleet bundle rows — exit=0 bundle rows=75
- PASS repo pin recorded — HEAD=003d9ef2265af313d4fbde796b1827dbe5b72b5f branch=strike/ergon-rehearsal dirty=0
- PASS repo r0-preflight ran and every install-related check passed — exit=1 PASS=5 FAIL=1 → FAIL  company project id not recorded (R0 blocking: write id + region only, no secret)
- PASS r0-preflight names its only blocking item as the company Supabase project (P5 owner act) if it blocks — exit=1 because project-id.txt is absent: the company Supabase project is a P5 owner act, out of scope for this strike (named, not silently skipped)
- PASS preflight fails closed with no evidence supplied — exit=1 stdin=R0 PREFLIGHT: FAIL (3 blocking)
- note staged but not installed — The drop directory also carries cayleb-dsh-fleet-0.3.0.tgz, cayleb-dsh-fleet-0.2.1.tgz, cayleb-dsh-fleet-0.2.0.tgz — listed in package-identity.json but NOT claimed as installed.
- note host — This pass runs on the platform host itself (outis-desktop), where the fleet plugin 0.2.2 is installed. The real R0 also wants a dedicated fresh cloud host + DNS + company Supabase project: those are owner acts (R0-PROVISIONING P1/P4/P5) and out of scope for this strike by its anti-goals. They are named in the pass index, not skipped silently.
- note no company state on the deploy paths — The generic preflight checked /srv/ergon /opt/ergon /var/lib/ergon /etc/ergon: absent (see preflight/absence-of-company-state.txt).
- evidence: `R0-pre/control-status.json`, `R0-pre/policy-get.json`, `R0-pre/fleet-unit.txt`, `R0-pre/package-identity.json`, `R0-pre/package-identity.txt`, `R0-pre/wrapper-version.txt`, `R0-pre/wrapper-status.txt`, `R0-pre/wrapper-dump-config-excerpt.txt`, `R0-pre/wrapper-dump-config.json`, `R0-pre/repo-pin.txt`, `R0-pre/preflight/run.log`, `R0-pre/preflight/summary.txt`, `R0-pre/preflight-failclosed.txt`

### R1 — Company setup and canonical state dump → **clean**
- PASS fresh rehearsal root is empty before setup — root=/tmp/ergon-rehearsal-20261101T223149Z-GWu5wW entries=0
- PASS setup.mjs exits 0 — exit=0
- PASS setup summary parses as JSON — keys=tickets,reviewDesk,library,profiles,stackArtefacts,devices,root,seed
- PASS setup reports one profile per cast member (6) — profiles=6 ids=dana-whitcomb,priya-lindqvist,marcus-oyelaran,teo-brandvold,rosa-quintero-hale,sam-akintola
- PASS setup reports the enrolled device set (6 cast devices + 4 background, incl. the pending sales laptop) — devices=10
- PASS setup reports the open-source stack artefacts — stackArtefacts=9
- PASS setup reports the seeded backlog + ReviewDesk + library summaries — tickets={"raised":104,"outcomes":{"resolved":79,"waiting":13,"escalated":3,"abandoned":1,"queued":8},"replies":133,"firstAt":"20 reviewDesk={"dana-whitcomb":{"requests":5,"later":2,"approve":1,"pending":1,"reject":1},"pr library={"tools":[{"name":"crew-sheet-builder","latestPublished":"1.0.0","versions":["1.
- PASS setup root is this pass's fresh root — summary.root=/tmp/ergon-rehearsal-20261101T223149Z-GWu5wW
- PASS company record matches the demo company — Lanternfield Moving & Storage / America/New_York
- PASS cast record holds six distinct named employees, each with a personalised agent profile — dana-whitcomb,priya-lindqvist,marcus-oyelaran,teo-brandvold,rosa-quintero-hale,sam-akintola
- PASS dump-state.mjs exits 0 — exit=0
- PASS canonical dump digest matches the pinned fresh-stand-up digest — sha256=558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1 pinned=558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1
- PASS canonical dump size matches the pinned size — bytes=315258 pinned=315258
- PASS canonical dump normalisation counters present — {"randomIdsAliased":1050,"wallClockTimestamps":73,"secretValuesHashed":3,"rootPathsReplaced":4}
- PASS board is recorded as scripted fixture state (board app PACKAGE-PENDING) — scriptedState=true day=pre-week status=PACKAGE-PENDING
- PASS onboarding sessions exist for the whole cast (fresh, stage installed) — session files=6
- note installer UI — R1 on a real host is the operator opening the install link and running the product installer. That package is not in the pinned tree (PACKAGE-PENDING); this rehearsal drives the fixture `setup.mjs`, which stands the company up through the product's own modules. Named stand-in: the first-run UI, not the company state.
- evidence: `R1/setup.log`, `R1/setup-summary.json`, `R1/company-cast.json`, `R1/state-dump.log`, `R1/state-dump-summary.json`

### R2 — First personalised assistants → **pending**
- PASS every cast member has a personalised agent profile in the fixture state — 6/6 profiles
- PASS every cast member has an onboarding session (fresh stage `installed`) — 6/6 sessions
- PASS each profile names its own employee and role — dana-whitcomb:operations-manager priya-lindqvist:dispatch-coordinator marcus-oyelaran:billing-estimates-clerk teo-brandvold:warehouse-lead rosa-quintero-hale:crew-chief sam-akintola:office-it-support
- PASS assigned agent ports are distinct per cast member — 19489,19491,19493,19495,19497,19499
- PASS product onboarding protocol reaches `active` for every cast member with a confirmed operating model — 6/6 active+confirmed
- PASS every onboarding drive answered all six interview stages and recorded observations — 6/2 6/2 6/2 6/2 6/2 6/2
- PASS a real fleet agent session runs for every cast member on the installed plugin — 6/6 sessions with session id + call id
- PASS the sessions are distinct real sessions — 6 distinct session ids
- note stand-in — The onboarding drive uses the fixture answer set (data/onboarding.json) as the employee's own answers and confirmation; that is a named stand-in for the employee's device taps. The state machine, validation and gate are the product's own code, and the confirmation gate still refuses a non-employee confirmer by construction (suite/onboarding/src/protocol.js confirm()).
- note stub-turn route — On this install the deterministic stub route (`stub-turn`, provider fleet-selftest-stub) returned status=completed with provider_calls=0 and empty assistant text for 2/2 probes: the turn machinery starts a session but no model turn is produced. The stub route therefore does NOT stand in for a model-quality answer here; the model leg is named PENDING below.
- evidence: `R2/fixture-assistant-records.json`, `R2/onboarding-drive.json`, `R2/fleet-sessions.json`, `R2/stub-turn-probes.json`, `R2/model-leg.txt`

### R3 — Shareable non-localhost URL from a second device → **pending**
- PASS company host origin recorded from the fixture config — origin=https://lf-host-01.example-tailnet.ts.net:8443 port=0 (0 = ephemeral loopback at run time)
- PASS origin is a DNS name, not localhost and not a raw IP — hostname=lf-host-01.example-tailnet.ts.net scheme=https: port=8443
- PASS origin is recorded as a made-up name that is never resolved (fixture note) — The Anthropic budget proxy's activation gate accepts only an https://*.ts.net:8443 origin (support/server/anthropic-budget-proxy.mjs validOrigin). The name is made up and never resolved; the rehearsal
- PASS the shareable name does not resolve (hosted URL/DNS absent) — getent exit=2 stdout=(none)
- PASS current host code listens on 127.0.0.1 only (no shareable listener) — address=127.0.0.1 port=42045 base=http://127.0.0.1:42045
- note device approval flow — The product's device approval flow is exercised host-side by the fixture setup (USB-kit join + operator approval asserted at R1/R4). What R3 needs and cannot show is the second device reaching a real URL.
- evidence: `R3/url-check.txt`, `R3/listener.json`

### R4 — Phone enrolment → **pending**
- PASS a phone device row exists in the host-side device registry — aliases=crew-tablet-1,crew-tablet-2,dana-desktop,front-desk,marcus-desktop,north-kiosk,priya-desktop,sales-laptop,sam-desktop,teo-phone
- PASS phone device is operator-registered and approved — enrolment=operator-registry status=approved employee=teo-brandvold
- PASS phone device belongs to the phone-first cast member — employeeId=teo-brandvold
- note device registry — Host-side fleet registry: 9 devices, 8 approved, 1 pending (the sales laptop is left waiting by design — a Monday act for the support desk).
- note desktop-off answer test — Scenario (e) — assistant answers while the desktop is off — needs the host agent runtime and a model; it is covered by the R2 model-leg PENDING, not claimed here.
- evidence: `R4/device-registry-standin.json`

### R5 — Desk task: ticket round trip and a real fleet order → **clean**
- PASS a1 ticket stored queued from the phone device, duplicate false — ticketId=c32bdd43-c3d4-4420-85a0-aeb16e637fc5 state=queued duplicate=false
- PASS a1 host record: state queued, device teo-phone, the exact reviewed preview — state=queued device=lf-teo-phone subject=Which line for the Harlow Legal job, C or D?
- PASS a2 operator queue lists the new ticket first (newest first) — first=c32bdd43-c3d4-4420-85a0-aeb16e637fc5 new=c32bdd43-c3d4-4420-85a0-aeb16e637fc5 total=100
- PASS a3 worker credential claims the ticket and gets a real lease — lease=issued (64 hex) leaseUntil=1793625000000
- PASS a3 ticket reads back working with the claiming worker — state=working worker=agent-sam-akintola
- PASS a4a worker reply refused (agent cannot reply on current code — F4 recorded) — HTTP 403 worker_route_denied (SCENARIOS.md F4: only the operator token can reply)
- PASS a4b operator (Sam, the person) posts the reply — replyId=lf-2-lf52-mvyen7yo-688d98-a4 duplicate=false
- PASS a5a agent resolved-finish outcome recorded against SCENARIOS.md F3 (409 remote_binding_required unless the device is remote-bound) — HTTP 409 remote_binding_required
- PASS a5b operator resolves the ticket with the repair receipt — state=resolved
- PASS a6 device read equals the host record: resolved, Sam's reply, the receipt summary — state=resolved replies=1 receipt=Told the warehouse lead: line C.
- PASS b1 ticket stored queued from marcus-desktop — ticketId=8963c19e-2557-4d9a-a186-5d3f25a684a3 state=queued
- PASS b2 ticket resolved with the reply and receipt, and Marcus's device reads both — host=resolved device=resolved replies=1
- PASS b3 the management record names the worker who handled it — worker=sam-akintola
- PASS both tickets have a product audit trail (stored -> claimed -> reply_added -> resolved) — audit rows=8: stored,claimed,reply_added,resolved,stored,claimed,reply_added,resolved
- PASS B1 fixture order created open on the installed plugin — order=o3283 state=open duplicate=false
- PASS B2 lead claims the order and closes its pre-flight — preflight=closed
- PASS B3 order decomposed to one board-shaped task — task=t4156 order_state=in_progress
- PASS B4 builder claims, posts evidence and submits the task as built — state=built evidence_post=7409
- PASS B5 a checker runs and records a pass verdict — checker=checker:selftest-chk-33648f1f8671 verdict_post=7411 task_state=done
- PASS B6 the order rolls up to done through its own done-when — state=done waited=506ms
- PASS B6b the board task is done and banked, and the order's final task is done+banked — task=done/banked=true final=done/banked=true
- PASS B7 every board post the pipeline wrote is readable (evidence + verdict + order-created + report) — posts=13: order-created,preflight,preflight-answers,decomposition,evidence,built,verdict,task-done,report,built,verdict,task-done,order-done
- PASS every fleet op answered ok and is recorded — ops=12, refused=none
- note product limit F4 — agent ticket-worker credential cannot post a reply — observed HTTP 403 worker_route_denied
- note product limit F3 — agent ticket-worker credential cannot finish resolved without a remote binding — observed HTTP 409 remote_binding_required
- note product limit F6 — the ticket record carries a free-text worker and replies have no author; Dana and Sam share one operator token — observed worker=sam-akintola reply-authors=0
- note warehouse board app — The warehouse-board app source lives only on the offline work PC (PACKAGE-PENDING). This order is a real fleet order carrying the board-shaped task; the board state it names is the fixture's scripted board (see R1: board.scriptedState=true). Named stand-in: the board app, never counted clean.
- evidence: `R5/tickets-round-trip.json`, `R5/fleet-order-ops.json`, `R5/fleet-order-readback.json`

### R6 — Voice / message flow → **pending**
- PASS supporting fixture check: the cast's written voices separate blind (not the voice surface) — ALL VOICES PASS
- PASS installed plugin bundle mounts no voice or phone surface row — bundle rows=75, voice/phone rows=0
- note fixture week voice acts — The simulated week's voice acts (fixtures/week/*.json) are fixture-side stand-ins for the voice surface; they are not the surface and are not counted here.
- evidence: `R6/check-voices.txt`, `R6/installed-surface-rows.json`

### R7 — Close-out: state record, teardown, second from-scratch pass → **clean**
- PASS prior step R0 produced its data — R0
- PASS prior step R1 produced its data — R1
- PASS prior step R2 produced its data — R2
- PASS prior step R3 produced its data — R3
- PASS prior step R4 produced its data — R4
- PASS prior step R5 produced its data — R5
- PASS prior step R6 produced its data — R6
- PASS final canonical state dump succeeds before teardown — exit=0 sha256=6f35be79869ed76df062f68691b8508be47a7d5d0a87ba63bc820c56f71984c9
- PASS state record written (company / cast / tickets / order / digests) — R7-close/state-record.md
- PASS no fixture secret value appears in the evidence pack — 25 values checked, 0 hit(s)
- PASS teardown empties the fresh company root — exit=0 entries after=0
- PASS the root was a fresh company root before teardown (state existed to destroy) — entries before=2
- PASS second from-scratch pass reproduces the same verdict table — pass 2 reproduced pass 1's words, named pending pieces and hard-check results from a fresh company root; seed digests identical (558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1)
- note state advanced by the pass — seed digest 558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1 -> final digest 6f35be79869ed76df062f68691b8508be47a7d5d0a87ba63bc820c56f71984c9 (the pass created real state: tickets, onboarded assistants, a acted-on board task via the fleet order)
- evidence: `R7-close/state-record.md`, `R7-close/secret-leak-check.txt`, `R7-close/host-teardown.txt`

## Commands run (this pass)

```
2026-11-01T22:31:49Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r0-pre.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:50Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r1-setup.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:50Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r2-assistants.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:50Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r3-url.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:50Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r4-phones.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:51Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r5-desk.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:51Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r6-voice.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:51Z  node /home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/steps/r7-close.mjs --pass 2 --repo /home/cayleb/workspace/harness-build/ergon-strike --run-root /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW --pass-dir /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
2026-11-01T22:31:51Z  rmdir /tmp/ergon-rehearsal-20261101T223149Z-GWu5wW  -> exit 0
2026-11-01T22:31:51Z  node lib/pass-diff.mjs --pack /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal --pass 2 --other 1  -> exit 0
2026-11-01T22:31:51Z  node lib/secrets.mjs /home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal/pass-2  -> exit 0
```

## Evidence manifest

The manifest is **sealed last** by `lib/seal.mjs`, after every other file in this pass directory is final
(steps, data, logs, `commands.log`, `secrets-scan.txt`, this index and its log). It covers every artefact in the
pass directory except the three hashed outside it: this index, `index-summary.json` and `manifest.sha256` itself.
The seal record — file count, bytes, the manifest's own SHA-256 and the seal-time verification result — is
appended below; the driver then runs `sha256sum -c manifest.sha256` from the pass directory and fails the run if
any entry is stale. Nothing is appended to the pass directory after the seal.

Each step directory also carries `notes.md` with the action → expected → actual → word record and the raw logs.

## Evidence manifest (sealed 2026-11-01T22:31:51Z)

- **Coverage:** 74 artefacts, 838058 bytes — every file in this pass directory except the three hashed
  outside it: `index.md` (this index), `index-summary.json` and `manifest.sha256` itself.
- **`manifest.sha256` SHA-256:** `04ffeaf1255e504b6104c74f6999ed038e618183ceae09bb47a159de089be1f6`
- **Verification at seal time:** 74/74 OK (all entries re-read and re-hashed in the same process).
- **Independent check:** after the seal, the driver runs `sha256sum -c manifest.sha256` from this directory; its output
  is `<pack>/manifest-check-pass-2.txt` and a non-clean result fails the run.
- **Nothing is appended to this pass directory after the seal.** `commands.log`'s last row is the pre-seal scan; the
  seal's own record is `<pack>/seal-pass-2.log`, and the post-seal secrets scan is
  `<pack>/secrets-scan-final-pass-2.txt`.


## Second pass (R7)

Verdict tables identical: **yes**. Second pass from scratch reproduced the same per-step verdict table; ids, timestamps and the mktemp root differ as expected.

Comparator: `node docs/acceptance/demo-company/rehearsal/lib/pass-diff.mjs` — pass 1 data under `pass-1/data` vs this pass's data; the diff file is [pass-diff.md](../pass-diff.md).

## Checker sign-off

Maker never checker: this pass was built and driven by the rehearsal lane (Team A / ergon-lead). A fresh-context checker who built no part of this candidate re-runs the driver against the same installed platform and signs here.

| | |
|---|---|
| Re-run command | `bash docs/acceptance/demo-company/rehearsal/run-rehearsal.sh --pass 1 --out <fresh-pack>` (then `--pass 2` into the same pack) |
| Index SHA-256 at signing | (fill after re-run) |
| Verdict table matches this index | yes / no |
| Secrets scan clean on the re-run pack | yes / no |
| Signed by (fresh-context checker) | |
| Signed at (UTC) | |

---

Vocabulary: `clean` — the step's checks held with no help; `pending` — the product piece the step needs is absent and named (PACKAGE-PENDING), never clean, never skipped; `failed` — the product was there and the step did not hold. Stand-ins (fixture code in place of a named surface) are named in the notes and never upgrade a pending word.
