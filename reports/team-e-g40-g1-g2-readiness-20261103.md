# TEAM E / LANE 6 — G40 READINESS + G1/G2 GATE EVIDENCE MAP (post role change)
**Filed:** 2026-11-03T00:07Z (19:07 ET, 2 Nov) by `upkeep-lead` (Team E / Lane 6). **Context:** owner 17:55 ET order re-homed team queues; Nox 23:20Z state brief narrows Team E's live role to **G40 observation + G1/G2 hosting** (record-keeping only). **All prior estate work is stood down: no /tmp cleanup, no digest, no N5, no estate task will be started** — the Lead's earlier RESUMEs are superseded.

## 1. G40 readiness mail — FILED
`~/Work/shared-workflows/lanes/board/inbox/2026-11/20261103T000604Z-team-e-to-nox-g40-readiness.md` — states Team E (Lane 6) is **READY to observe G40**, read-only, creates no order/runs no probe; includes the W-vs-C definition, the observation method, the record format, and the known create-blocker (provider/key 403 stops any lane start — finish lane owns it).
**Condition for the finish lane** (their 232800Z §2): create-and-run fires when **both** h5 is acked (delegator 000700Z reports done) **and** this mail has landed. My mail is on disk; the board sensor's ops commit cycle posts inbox files to origin/team.

## 2. G40 observation plan (when the probe runs)
| Step | What | Evidence I take |
|---|---|---|
| 1 | Watch for the finish lane's probe order + its output; do not create or run anything | order id, command block, lane/session/preset/cwd/uid, per-command output + exit codes, timestamps |
| 2 | If the lane-shell PID is observable: independent `/proc` reads (read-only, may be permission-limited) | `/proc/<pid>/cgroup`, `status` (`Uid:1000`, `NoNewPrivs`, `CapEff`), `mountinfo` (`/home` ro? `/run` tmpfs?), `/proc/<pid>/root/run/user/1000/wayland-1`, `/proc/<pid>/root/run/fleet/control.sock`, `/proc/<pid>/ns/mnt` inode vs mine |
| 3 | Capability table from the probe + cross-checks | user-manager write / Wayland socket / control socket / `/run/user` contents → observed → quote |
| 4 | Record `board/reports/team-e-g40-observation-20261103.md` | W / C / BLOCKED / INCONCLUSIVE + **PASS** (real lane shell, W-vs-C determined with named evidence) or **CONDITIONAL** (channels unobservable, limits named) or **FAIL** (not a real lane shell / inconsistent or absent evidence / state-changing command found) |
**Blocker already known:** finish lane 234800Z/235700Z/000000Z — the delegator wake and any lane start are blocked by a provider/key path (403, Teams key/adapter absent), so G40 may be **create-blocked**. If so I record **BLOCKED — probe start refused: <quoted error>** and stand by; the plan applies unchanged the moment a real lane shell runs.
**Fences:** read-only; no state-changing commands; no order creation; no installs; no fleet-host changes; no privileged acts; no token/credential reads; $0.

## 3. G1 gate — evidence needed (Lane 6 hosts; offline, before any external write)
**Gate definition:** `leg3-duet-bridge-design.md` §11 step 6 / B7 steps 1–5 — the gate on the built code (rendezvous + `dir` adapter, fake-Duet harness E2E, fence module + hostile corpus, courier service, scope/auth mutation tests; adapter conformance; design/step list). **Nothing external is written before G1 passes.**
| Evidence package I need from the builder | Source |
|---|---|
| Frozen commit + file list + diff scope (one transport = Leg 2's; no unshipped listener/mailbox leftovers) | repo, B7 steps 1–5 |
| Fake-Duet harness **deliver-pickup-ack** pass (states `queued→delivered→picked→acked`, reply bound to handoff, pickup latency) — unit evidence only, labelled | checklist 12 / §12 |
| No-listener + sandbox C0: courier opens no socket; from its namespace the control socket, DSH home/state, legacy-token dir, Wayland/X and `/run/user/1000` are absent/denied; only its own dir mounted; C0 result recorded (holds or not) | items 1, 1a, 1b |
| Egress allowlist + facade source allowlist unchanged; E2E under `unshare -rn` with connect recorder (no non-loopback connect); no bare `fetch` | items 3, 4, 5 |
| Token/canary: canary never in logs/board/events/argv/env; token files 0600 outside DSH home, absent from lane sandbox, stated uid-1000 readability; revocation takes effect next request; mutation tests fail the suite | items 6, 9, 21 |
| Scope matrix: `duet` token = reply only to nox/cayleb-local/lead with valid envelope; 11 other facade tools + every `board.op` `scope-denied`; pulls only own mailbox; no claim/lease | items 7, 8, 10; amendment **C1** built |
| Envelope **C2** built at send (closed keys, `in_reply_to`→handoff-to-duet, `to`=original sender, nonce, enum/text rules) + bounds/UTF-8/path rejects + idempotency (incl. nonce-absent ping definition) | items 12, 13; C2 |
| Fence/tripwires: ≥50 hostile payload corpus → closed fields only, boundary intact; tripwires hold; exfil tests (lane handoff asking mailbox/calendar/Drive content refused; leaky fake dropped; nox request text labelled+scanned) | items 14–17, 28 |
| Authority/spoof: a Duet message claiming approval changes no ticket/grant/policy/elevation; foreign-owner file / lane-claiming-Nox / control-socket request held / copied-token-without-scope → none delivered as an authenticated handoff; no direct path to builders/checkers/elevation/confirm (indirect path listed) | items 16, 16a, 15 |
| Reliability: kill -9 between rendezvous and bus post → exactly one reply; overdue exactly one notice; courier outage leaves facade/leases/liveness unchanged; chain verifies (accidental-damage wording) | items 23–25 |
| Rate limits + bell/Drive adapter conformance (both modes) + `comms-inbox` withholding (C5) | items 19, 29, 30 |
| Open-limits list + **G40 result** (C5/A4 finality: single lift `comms.control-channel-trusted`, default false, awaits G40) | claude-finish 235200Z §27; PA-9 |
**My output:** per-item verdict + gate verdict PASS / CONDITIONAL (named condition) / FAIL (named failing leg + fix). Maker never checker: the builder cannot run this gate; I do not build.

## 4. G2 gate — evidence needed (live proof; owner present; external services touched)
| Item | Evidence required | Runner/constraint |
|---|---|---|
| **2** | read-only `tailscale serve status` and `tailscale funnel status` **byte-identical** before install, after install, and during a run | owner or gate runs it; never in the build |
| **22** | Google granted scope read-back **equals** the decision tree's scope and nothing else | at setup, owner present |
| **26** | live proof bundle: ping text **byte-identical** to Nox ruling 3b; **exactly one** ping; pickup latency; `via` recorded; Drive revision metadata (pinned modifier in variant D); report states which of "delivered into a pull" / "a turn was started" the evidence supports and claims nothing beyond; `revoke-all` + courier stop leaves **no live credential** | owner-present; no credential values in any record |
**Also rides this window:** PA-9 C5/A4 are **built but not final** until G40 lifts `comms.control-channel-trusted`; G1 follows that lift. **Lane 6 G2 hosting = observe + verify the bundle against the items; I run no privileged command, open no token, touch no live service.**
**Current state:** no G1/G2 material has been submitted yet (courier + fake-Duet harness are offline build work; G40 is the precondition). I stand by; when the builder freezes a candidate I need the §3 evidence package, and for G2 the owner-present run + §4 bundle.

## 5. Standing (nothing else)
No estate/digest/N5/tmp work will start. Read-only; no installs; no privileged acts; no fleet-host changes; no credential content; $0. The G40 observation record and the G1/G2 verdicts are the only outputs I will file.

---

## 6. G1 CONFIG-REVIEW ITEM — egress proxy destination policy (added 2026-11-03T00:27Z from the G40-NET ruling)
**Why:** G40-NET showed the lane's net ns has exactly one listener, **`127.0.0.1:3128`** (designed egress forwarder). Its destination policy is a **config property, not a probe result**, so it is carried here as a Lane 6 G1 config-review item: *does the proxy refuse host-private destinations (host loopback services and the tailnet address)?* **Method: configuration + code read only — no probe, nothing run, nothing installed.**

### 6.1 Where it lives (exact paths, all readable as this user)
| Piece | Path |
|---|---|
| **Runtime policy config** | `$DH/profiles/fleet/cordis.patch.yml` — `fleet-egress` row (lines 27–35) and `fleet-sandbox` row (lines 20–26), where `DH=/home/cayleb/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home` |
| Proxy implementation | `$DH/profiles/fleet/node_modules/@cayleb/dsh-fleet/src/egress/{proxy.js,guard.js,address.js,dispatcher.js}` |
| Sandbox wiring | `$DH/profiles/fleet/node_modules/@cayleb/dsh-fleet/src/sandbox/bwrap.mjs` |
| Audit log | `$DH/fleet/log/egress.jsonl` (mode 0640 cayleb:cayleb; the pre-connect line is the guarantee) |
| Socket | `/run/user/1000/fleet/egress.sock` (srw------- cayleb) — exposed inside the sandbox as `/run/fleet-egress.sock` |

### 6.2 The rules, quoted verbatim
**Runtime config (`cordis.patch.yml`, `fleet-egress`):** `socketPath: /run/user/1000/fleet/egress.sock` · `logPath: …/fleet/log/egress.jsonl` · **`allow: [registry.npmjs.org]`** · **`allowPorts: [80, 443]`** · `connectTimeoutMs: 10000` · `idleTimeoutMs: 300000` · `guardInProcess: true`.
**Sandbox row (`fleet-sandbox`):** `egressSocket: /run/user/1000/fleet/egress.sock` · `tmpSizeMiB: 512` · `maskRoots: [fleet/dsh, fleet/state, fleet/log, fleet/work, fleet/backups, fleet/checks]`; `--unshare-net` with only loopback, and the forwarder is a **byte pipe** — `FORWARDER_SOURCE` in `bwrap.mjs:65-77` listens on `127.0.0.1:3128` and `net.connect({ path: sock })` to the unix socket, so "the egress proxy at the other end speaks the HTTP/CONNECT protocol"; `PROXY_ENV` sets `HTTP(S)_PROXY=http://127.0.0.1:3128` **and `NO_PROXY=''`** ("nothing bypasses the egress allowlist").
**Proxy (`proxy.js`):** allowlist checked on the **requested host name** — `hostAllowed(patterns, host)` (exact, `*.sub`, or `*`; here only `registry.npmjs.org`), port must be in `allowPorts`; refusal is `HTTP 403` with `X-Fleet-Egress: allowlist|port`; the request is **logged before any outbound connection** and a log-write failure denies (503).
**Guard (`guard.js`):** `checkLiteral()` **refuses IP-literal hosts before any socket exists** ("net.connect never calls lookup for those"); `makeLookup()` resolves **exactly once** and fails if **ANY** resolved address is forbidden ("a mixed answer is the signature of rebinding"); `connectGuarded()` checks the **connected remote address** again before any byte is written.
**Address classes (`address.js`, `forbiddenReason`):** refuses **`127.0.0.0/8` (loopback)**, `0.0.0.0/8`, **`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` (private)**, **`100.64.0.0/10` (reason string `cgnat-tailnet`)**, `169.254.0.0/16` (link-local), `224.0.0.0/4`, `240.0.0.0/4`; IPv6 `::/96` (unspecified/loopback), IPv4-mapped/NAT64/6to4 decided by the embedded IPv4, `fc00::/7` (unique-local), `fe80::/10` (link-local), `ff00::/8`; anything unparseable ⇒ `'unparseable'` (fail closed).
**Host-side in-process path (`dispatcher.js`, `guardInProcess: true`):** the same literal/lookup/peer checks are installed on undici's global dispatcher, so in-process HTTP clients get the same guard.
**Config integrity (relevant because this is a policy property):** the config and code sit under the DSH home; the G40 observation's own mount evidence shows the lane's namespace has **`/home` read-only** (and `maskRoots` cover fleet state/log/work), so a confined lane cannot rewrite this policy. Whether every other lane class has the same property is a separate question (not asserted here).

### 6.3 Verdict per destination class (config/code read, no probe)
| Destination class | Expected policy | Where the rule is | Verdict |
|---|---|---|---|
| **Host loopback services** (`127.0.0.1:8790/3092/8791/9333/8799`, `::1`, `localhost`) | unreachable | (a) host allowlist is only `registry.npmjs.org` ⇒ 403 `allowlist`; (b) even if named, `127.0.0.0/8` is `loopback` in `address.js`, and an IP literal is refused pre-socket by `checkLiteral` | **PASS — refused (two independent controls)** |
| **Tailnet / CGNAT address** (`100.64.0.0/10`, e.g. the host's `100.114.17.67`) | unreachable | `address.js` `100.64/10 → 'cgnat-tailnet'` (fail closed at resolve **and** at connected-peer re-check); not in allowlist either | **PASS — refused** |
| **Other private / link-local / ULA** (`10/8`, `172.16/12`, `192.168/16`, `169.254/16`, `fc00::/7`, `fe80::/10`) | unreachable | same `address.js` classes | **PASS — refused** |
| **Public internet** | deny-by-default; one named exception | only `allow: [registry.npmjs.org]` + `allowPorts: [80,443]` pass the name/port gate; the resolved address must also be public (loopback/private answers to that name are refused) | **PASS for "refuses non-allowlisted public"; CONDITIONAL as a class** — egress is allowlist-only, not "off": widening `allow`/`allowPorts` in this config is the one knob that changes public reachability (today: one host, two ports) |
| **IP-literal destinations (any class)** | refused before connect | `checkLiteral` | **PASS — refused** |
| **Arbitrary TCP via the 3128 forwarder** | not a relay | forwarder is a byte pipe to the HTTP/CONNECT proxy's unix socket; the proxy parses/answers HTTP and destroys client errors — no arbitrary TCP path | **PASS (by construction)** |
| **Audit of attempts** | pre-connect log line; write failure denies | `proxy.js` `admit()` (503 on log failure); `egress.jsonl` | **PASS (fail closed)** |

### 6.4 Limits of this review (stated, not hidden)
1. **Config/code read only — no probe**, per the ruling. Runtime behaviour is evidenced by the installed source + config, not observed here; the G40-NET observation (separate file) already showed the lane's *direct* reachability is closed.
2. **Installed-vs-read:** this reads the installed fleet package under the live profile; it does not re-derive hashes against the release manifest (G1's manifest item covers that).
3. **DNS path** is host-side resolution through the proxy's guarded lookup; the sandbox's resolver configuration is not reviewed here.
4. **Config integrity depends on the sandbox/fsguard** (lane's `/home` is ro per the G40 observation); "no lane class can rewrite the policy" is not asserted for classes not probed.
5. **The allowlist is the open knob** — correct for "no host-private destinations", but "no public egress" holds only while `allow: [registry.npmjs.org]` stands; changes to that config need their own review.

**Disposition:** carried on **G1** as **PASS (host loopback / tailnet / other private classes refused; public deny-by-default with the single named allowlist exception, CONDITIONAL on the config knob)**; **no third probe required for the policy question** — it is a config property, and the config is quoted above. Nothing was run, installed or changed.
