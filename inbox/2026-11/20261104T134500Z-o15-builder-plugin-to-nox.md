# o15 builder lane → Nox — PLUGIN FINISH (t35): what is proven live, what is staged, what is yours

**2026-11-04 13:45 ET (17:45Z).** The builder lane for fleet order o15 (systemd `o15-builder.service`, model
`ollama/minimax-m3.1-flash:cloud`). The order was unblocked as the ordering principal's own control op
(`op unblock` → `{"kind":"order","id":"o15","state":"in_progress"}`), and t35–t39 are claimed on the board as
`builder:o15-builder`. Every number below was read live in this session; the checker file
`~/fleet-scratch/overnight/o15-t35-check.md` re-ran them in a fresh context.

## 1. GitHub publish — DONE, and the pushed tree carries the goal

```
$ git ls-remote https://github.com/cayleb-james2008/dsh-fleet
27335f9346877e7fa12633a8ae40cef5436500dc	HEAD
27335f9346877e7fa12633a8ae40cef5436500dc	refs/heads/main
5f184baffbfde9dd69122057c8e01fe4ec4ab6e8	refs/tags/v0.3.6.2
9a4b053283dee15c95723ba83c8879ef65c8f717	refs/tags/v0.3.6.2^{}
$ gh repo view cayleb-james2008/dsh-fleet --json visibility,pushedAt
{"visibility":"PRIVATE","pushedAt":"2026-11-04T18:19:37Z"}
$ git show origin/main:README.md | head -8
# dsh-fleet

> The plugin is a **prompt-to-fruition machine**: a workflow that takes any idea and brings it to a ludicrous quality bar —
> far beyond anything we've seen professionally. The goal of the plugin is that a prompt goes in one end and a finished,
> verified result comes out the other, with the machine doing the work in between.
>
> — the owner's brief, in his words (Nox, 2026-11-03)
```

**`origin/main` = 27335f9**, tag `v0.3.6.2` → 9a4b053, repo **PRIVATE** (so the ordered publish is inside the "no public
posts" fence). The goal leads the pushed README **verbatim** — read from `origin/main`, not from the worktree. Note for
the record: the published tree has advanced past the local clone (local HEAD 9a4b053), so the remote is the truth.

## 2. Wake proofs P2/P3 — PROVEN on the bus; one verdict file is one ping behind

```
h84 cayleb-local→delegator ping "wake proof P2"  2026-11-04T04:41:48.802Z  acked 04:42:05.506Z  nonce wp32f0412087
h86 cayleb-local→delegator ping "wake proof P3"  2026-11-04T10:30:28.586Z  acked 10:30:42.679Z  nonce wpfa6528f492
h89 cayleb-local→delegator ping "wake proof P3"  2026-11-04T10:52:50.100Z  acked 10:52:54.714Z  nonce wpf09109dcbf04
```

h89's delivery row: `state=acked`, `signals=1`, `first_delivered_at 10:52:51.601Z` → `acked_at 10:52:54.714Z`, the
ack_note quotes the nonce ⇒ T0 ≤ T1 ≤ T2 ≤ T3. `roster_wakes`: delegator handoff at 10:52:50.109Z.

**Correction to the earlier seat's claim (it matters):** `finish-state/W3/records/verdict-P3.json` is for **handoff_seq 86**
(`"at":"2026-11-04T10:30:42.679Z"`), *not* h89 as the closeout note said. Both pings are acked in the DB; the checker
file on disk covers the earlier one. No verdict file exists yet for h89.

## 3. Soak — RUNNING (the criterion), verdict honestly NOT YET

```
$ bash ~/fleet-scratch/overnight/soak-check.sh
days_aside      0  (retired 20261104T034517Z, now 2026-11-04T18:23:22Z)
restarts_seen   3 restart(s); 4 distinct Host pid(s) across 98 sample(s)
sampler         LIVE: newest sample 8 min old (2026-11-04T18:14:37.895Z); limit 30 min
soak_smoke      none: no ping whose subject mentions "soak" has been sent yet
verdict         NOT YET: 1 more day(s) aside; a soak wake smoke that reached acked with its nonce
$ systemctl --user status fleet-soak-sampler.service
Active: active (running) since Wed 2026-11-04 02:04:28 EST; 11h ago   Main PID: 4088220
```

**Soak running: YES** (the sampler is alive, 98 samples). The soak *READY* verdict is not, and I did not manufacture it:
a day aside cannot be faked, and the sheet-8.3 soak smoke is the owner's ritual ping. That stays on your list.

## 4. Crash-fix — STAGED, OPEN AND READY; it is an owner act, and here is the smallest form

```
$ bash ~/fleet-scratch/overnight/stage/install-reliability.sh --check
== 0. preconditions
   no overlap with his uncommitted work (29 dirty file(s) left alone)
== 1. watchdog dry check (read-only)
   (live checkout has no copy yet; dry-checking the branch copy)
checkout: /home/cayleb/workspace/harness-build/wt-app-reliability
dry check complete; nothing was launched, killed, or stopped.
log: /home/cayleb/.local/state/dsh-desktop/watchdog.log
2026-11-04T13:24:20-05:00 reaping dsh-subprocess-3926853-959d711fde53.scope: owning Host pid 3926853 is gone
== check complete: nothing merged, nothing built, nothing installed
EXIT=0
```

The **watchdog half is already live in the running app** (it reaped an orphan scope at 13:24:20 ET, minutes ago). The
fix's *code* (`fix/dsh-app-reliability` @ 8cee1813) is **not** in the live checkout: installing it means a
`git merge --ff-only` into your checkout (29 dirty files) plus a rebuild plus **an app relaunch**, which opens a window
on your screen — that is why I did not do it.

**Your act, smallest form:** in a quiet window, `bash ~/fleet-scratch/overnight/stage/install-reliability.sh`
(then relaunch the app once), then run `~/workspace/harness-build/finish-docs/APP-RELIABILITY-OWNER-RUN.md` §3
(A crash recovery, B watchdog relaunch, C duplicate launch refused, D goal re-arm). Rollback: `git revert --no-edit
8cee1813`.

## 5. Face panels — installed, live, and the live read is the fleet's own

- Installed package **0.3.6.2**; `src/client/client.js` sha256 **2474cbcb…9be283**; `src/face/` present
  (host.js panels.js snapshot.js attention.js redact.js).
- The Host is alive: `op status` → pid **1010625** (uptime 6.9 h), integrity ok, **11 pinned / 12 disabled**, manifest
  `e9364e57…488843`, `lanes_allowed: true`.
- The panel is fed by live read ports only (`src/face/snapshot.js`: roster-get lanes, the watchdog's episodes,
  `listOrders`/`orderTasks`, the read-only events feed, `comms.who`, `elevate.list`, policy) — read-only by
  construction, and it redacts titles through `redact.js`.
- The live read behind it: **o15 is `in_progress` with 6 open tasks** (`op order-get`), the fleet core answers on the
  control socket, and the earlier in-app capture shows the panel polling: spend moved **0.445 → 0.453 USD** between
  two shots, with **0** occurrences of "not connected" in the accessibility tree (70 named nodes).
- Captures on disk (prior seat, in-app, 10:50–10:55Z): `~/fleet-scratch/INSTALL36/shots/after-fleet.png`
  (sha256 c945c13b…), `after-fleet-2.png` (0f3851fb…), `after-agents.png` (f377af84…), `after-phonevm.png`
  (8a116251…), `cmp-after-sidebar.png` (d7282ad6…).

**One stale row found:** the connections probe still checks **8787** for the face producer, but the installed 0.3.6.2
face serves on the app's **own** connection under `/api/fleet/` — the probe's "face-producer NOT RUNNING" row is
history, not a fault. (The probe's `crouton-alpha` row is a real gap — see §7.)

## 6. Connections audit — 15 rows, live

```
$ bash ~/fleet-scratch/overnight/connections-probe.sh
control-socket  UP  ok pid=1010625 up=24930s pinned=11 integ=ok
single-writer   UP  WRITERS-SCAN: 1 process(es) hold the roots
facade-loopback UP  HTTP 401 (listening, token required)
facade-tailnet  UP  100.114.17.67 HTTP 401
bus-participants UP  cayleb-local:never[pending=38 head=h10] ext:delegator:absent(889m) ext:rsi:fresh(0m)[pending=1 head=h78] nox:fresh(7m)
nox-bus         UP  nox:fresh(7m)
owner-principal NEVER-PULLED pull-only, nothing has pulled as it
bus-ack-path    UP  newest ack: 85 acked signals=0 acked=1793816303497 (newest message overall: 90 signalled)
lane-wake       UP  roster_wakes=64, newest 1m ago
rsi-leg         PARTIAL manual board relay; newest to ext:rsi seq=78 state=delivered
provider-in-use UP  session.v4.jsonl.zstd last call "provider":"opencode-go","model":"deepseek-v4.1-flash"
crouton-alpha   UNPROVEN provider ref resolves but no answered call exists and the package carries no engine-chain config
browser-surface UP  127.0.0.1:8796 bound
containment     UP  sandbox=ok egress.sock
face-producer   NOT RUNNING nothing on 8787 (standalone producer stopped 01:23 ET; the face lead's 8796 is row 14)

15 rows: 11 up, 0 down, 4 between
```

**`nox` is fixed since the last pass** (it now has a bus token and pulled 7 min ago; h90 — its ruling on h85 — was
acked at 18:18Z). The one row that still cannot be proven is `crouton-alpha` (§7).

## 7. Crouton Alpha — the exact blocker, named (order criterion 5 allows this)

Crouton **cannot answer at all right now**:

```
$ opencode run -m opencode/crouton-alpha-free "Reply with exactly: OK"
Error: Free usage limit reached. Free models work in OpenCode (https://opencode.ai); subscribe to OpenCode Go to keep going: https://opencode.ai/go
EXIT=1
```

(Earlier today, 06:2x ET, the same route answered `OK` with exit 0 — that call is in `plugin-progress.md`. The free tier
is exhausted since.) The in-app route has four independent blockers, each with its file and line:

1. `…/profiles/fleet/cordis.patch.yml:162-170` — the live `llm-pi-ai` row declares `opencode-go` and `openrouter`
   with **no `models:` list**, so pi-ai's catalogue is used.
2. pi-ai 1.3.0's catalogue has **no crouton entry at all** (grep over `@earendil-works/pi-ai/dist/` → no hits).
3. `packages/subagent/tool-subagent/src/model-selection.ts:139-157` — `assertAllowedModelSelection` throws
   `child LLM route "P/M" is not allowed for this Session` for any route outside the session allowlist; the
   `subagent-model-selection-settings` row ships in `packages/bundle/web-app/cordis.patch.yml:58-59` **with no
   `config:`**, and an id-targeted entry in the *profile* overlay **cannot add a row** (the fleet bundle patch says so
   itself: `patch: entry "fleet-elevate" not found`). The allowlist therefore needs a **bundle-patch re-pack + install
   of the next plugin package** — the route you ruled on 4 Nov.
4. The free-tier limit above: even a wired route would fail its first call today.

**The app is running DeepSeek V4.1 Flash** (`provider-in-use` row: `opencode-go` / `deepseek-v4.1-flash`) — second in the
chain, not first. **Your acts:** (a) a decision on the engine chain once Crouton can answer again (a subscription, or
the chain's first slot left to DeepSeek V4.1 Flash), (b) the next plugin package carries the allowlist row. Nothing else.

## 8. Retired profile — intact, and the archive is blocked on exactly one thing

`…/profiles/desktop.retired-20261104T034517Z` is **still there, untouched**: 7,982 files / 322,348,281 bytes, digest
`05a8ef5c…b66cc1bd` (`evidence/retired-profile-baseline.json`, re-checkable with `retired-profile-baseline.sh --verify`).
The archive step (sheet row 7) is `release/relocation/archive-build.sh`, which **encrypts to an age recipient public
key** and refuses without one (`REFUSED: the recipient file is not an age public key`). **There is no age recipient key
on this machine**, and the sheet's own gate says so: `G-C. Off-machine key copy + Q8 — OPEN: Owner: copy
fleet/recovery.key off-machine … answer Q8 (Drive destination)`.

**Your acts:** (1) put `fleet/recovery.key` off-machine and give the archive its destination (Q8); (2) answer Q16.
Everything else about the archive is already built and verified. **Nothing was deleted, archived or moved by this lane.**

— o15 builder lane (`builder:o15-builder`, order o15, task t35)
