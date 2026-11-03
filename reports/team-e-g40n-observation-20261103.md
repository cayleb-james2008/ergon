# G40N OBSERVATION RECORD — lane-shell NETWORK leg (Team E / Lane 6)
**Observer:** `upkeep-lead` (Team E / Lane 6) · **Observed:** 2026-11-03T00:19–00:22Z (19:19–19:22 ET, 2 Nov) · **Authority:** Lead's live instruction; Nox's G40 line (case W vs C for the four legs); order **`o9`** (G40-NET, throwaway) created by the finish lane.
**Read-only:** I created/ran/banked/closed/nudged nothing; no state-changing command, no install, no fleet-host change, no privileged act, no token/credential read, no payload sent by me; $0. **Verdict: PASS (observation executed + independently verified); the "network off" claim is CONDITIONAL — direct TCP reachability is absent, egress-proxy policy is not tested** (limits §6).

## 0. Context recorded (no action by me)
**`o8` (G40 confinement) closed administratively** under Nox's ruling — state `failed` with `fail_reason`: "closed administratively … t21 is done but unbanked, only its maker may bank … banking gap, instance two (o1 t5 was instance one); the stored failed state is a harness artefact". **Banking-gap instance two recorded.** I do not bank/close/nudge; **`o9` is expected to reach `done` at the checker and then sit unbanked** the same way — no action from me.

## 1. Order / task / probe timeline (fleet DB + posts, read-only)
| When (UTC) | What | Evidence |
|---|---|---|
| 00:19:22 | **`o9`** "G40-NET PROBE (throwaway): lane-shell network confinement, connection attempts only" created by `cayleb-local` (post 217), one task by design | posts 217 |
| 00:19:28 / 00:19:37 | lead preflight (218) + decomposition to **`t23`** (219) | posts 218, 219 |
| 00:19:53 | **`t23` evidence** by `builder:1` — complete `g40n-output.txt` + commit + hashes (post 220) | post 220 |
| 00:19:57 | **`t23` built**, commit **`47d0e91320d4d1188f3ffdc26e082187fae471be`** (parent `cbee8da`, clean tree), artefact hashes (post 221) | post 221 |
| 00:21:35 | **checker `checker:c510d8c61e59f` verdict: PASS** (round 1; verdict id 15, post 222); `t23` → **done** (post 223); `t24` still **open**, order still `in_progress`/lead | posts 222/223, verdicts 15, tasks 23/24 |
**Lane identity:** same builder lane/workspace as o8 — `fleet-builder-dfe0f0d5-070d-4cf8-afb9-878c2e3e9bed`, principal **builder:1**, lane repo `$DH/fleet/work/lanes/builder_1-2e3e9bed`, probe shell in transient `dsh-subprocess-3671044-…scope` (per the o8 pattern; the shell itself had exited before I could read its live `/proc`).

## 2. Probe script (verbatim `g40n.sh`, as specified — connection attempts only, no payload)
`readlink /proc/self/ns/net` · `cat /proc/net/dev` · `ss -ltnH 2>&1 | head -40` · `for t in 127.0.0.1:8790 127.0.0.1:3092 127.0.0.1:8791 127.0.0.1:9333 127.0.0.1:8799 127.0.0.1:53 1.1.1.1:443; do h=${t%:*}; p=${t#*:}; timeout 3 bash -c "exec 3<>/dev/tcp/$h/$p" 2>/dev/null; echo "connect $t exit=$?"; done`

## 3. Probe output (verbatim, 13 lines, ends `G40N-END`)
```
net:[4026533376]
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:       0       0    0    0    0     0          0         0        0       0    0    0     0       0          0
LISTEN 0      511    127.0.0.1:3128 0.0.0.0:*
connect 127.0.0.1:8790 exit=1
connect 127.0.0.1:3092 exit=1
connect 127.0.0.1:8791 exit=1
connect 127.0.0.1:9333 exit=1
connect 127.0.0.1:8799 exit=1
connect 127.0.0.1:53 exit=1
connect 1.1.1.1:443 exit=1
G40N-END
```

## 4. My independent verification (not just the post)
1. **Artifact byte-check:** `g40n-output.txt` read from the lane workspace on disk is **byte-identical** to post 220 (13 lines, `G40N-END`); `sha256sum` → `g40n.sh` **`6df3ef08…8fe34e0`**, `g40n-output.txt` **`9f46524d…763745d98`** — exactly the built record's values.
2. **`g40n.sh` content:** exactly the four-section script from `t23`'s spec (no extra command).
3. **Git:** commit **`47d0e913…71be`** exists (parent `cbee8da`, the o8 commit; tree `f81b9ddc…`), `git status --short` **clean** — the lane's own repo.
4. **Namespace identity:** the lane's `net:[4026533376]` is **distinct from the host/observer net ns `net:[4026531833]`**; `/proc/net/dev` in the lane lists **only `lo`** (all counters zero). No live process remained in `net:[4026533376]` when I scanned `/proc/*/ns/net` (transient shell — named as a limit).
5. **Host-side contrast (what the lane could NOT reach):** on the host, listeners **exist** on `127.0.0.1:8790` (facade), `:3092` (fleet UI), `:8791` (OpenBell dash), `:9333` (chrome-devtools MCP) — all four target ports **failed from the lane**. For completeness: `127.0.0.1:8799` has **no host listener** (only `127.0.0.53/.54` and `172.17.0.1` own `:53`), so those two failures are **not** isolation evidence by themselves.

## 5. Capability table + determination (network leg)
| Probed capability | Lane result | Host-side/Method note | Implication |
|---|---|---|---|
| Network namespace | `net:[4026533376]` — **distinct** from host `net:[4026531833]` | readlink `/proc/self/ns/net`, both read | **isolated net ns** |
| Interfaces | only `lo`, counters all 0 | `/proc/net/dev` | no physical/virtual uplink in the lane ns |
| In-ns listeners | one: `127.0.0.1:3128` | `ss -ltnH` | an in-namespace **loopback egress proxy** exists (not probed by this order) |
| Host-local service reachability | **0/4 succeeded** — `8790`, `3092`, `8791`, `9333` all `exit=1` (connect-and-close; no payload) | host has listeners on all four; no timeouts (124) | **no direct TCP reachability to host-local services** |
| Targets with no host listener | `8799`, `127.0.0.1:53` → `exit=1` | host: 8799 none; 127.0.0.1:53 none | consistent, but **not** isolation evidence |
| Public internet | `1.1.1.1:443` → **`exit=1`** | connect-and-close, no payload | **no direct outbound internet reachability from the shell** |
**Determination:** network leg = **C (confined)** in the direct-reachability sense: isolated net ns, loopback-only interface table, and every direct connect failed, including all host-local ports that do exist and the public internet. **NOT proven:** egress policy — the in-ns `:3128` proxy is the likely sanctioned path and was not probed; the probe shows TCP reachability only, not firewall/policy semantics. So the honest formulation is "**isolated net ns, no direct reachability**", not "no network stack whatsoever".

## 6. Limits (stated, not smoothed)
1. **No live `/proc` of the lane shell:** the probe shell was transient; no process remained in `net:[4026533376]` at read time. My independent evidence is the committed artifact on disk (byte-identical), git object/clean tree, the lane's own output, and the host-side contrast — post-hoc file evidence, not a live namespace read.
2. **Egress-proxy / policy untested:** the in-ns `127.0.0.1:3128` listener was not among the order's targets; whether it is reachable and what policy it enforces is **unknown** from this probe. "Network off" should therefore be read as "no direct TCP reachability", not "no egress capability".
3. **TCP reachability only:** the probe opens and closes sockets (no payload, no HTTP, no token), so application-level reachability and DNS behaviour are not measured.
4. **Scope:** one builder lane's DSH-native tool shell; other lane classes need their own probe before their case is extended.
5. **Banking:** `t23`'s checker verdict landed **PASS at 00:21:35Z** (verdict 15) and `t23` is `done`; the order still sits `in_progress` with `t24` open — the expected **banking-gap instance two** shape (o8 was instance one, administratively closed under Nox's ruling). **Not mine to bank, close or nudge**; recorded only.

## 7. Verdict (gate)
**PASS — G40N observation executed on a real lane shell, artefacts independently verified, and the network leg determined: isolated net ns, no direct TCP reachability to host-local services or the public internet.** The stronger "network off" claim is **CONDITIONAL**: the in-namespace `:3128` egress proxy and its policy are unprobed (§6.2), so the design should record the weaker, evidenced wording. No commands beyond the probe's own read-only attempts were run by me; no state changed; $0.
