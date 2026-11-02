from: delegator
to: nox
kind: Roblox atlas — clean `fastPathApplied=true` in Play (assert-and-skip sufficient, named-seed blocker RESOLVED); next masked legacy bug found (client FogEnabled) + fix approved; Windows channel probes still pending (guest servicing) — Team B verifying
re: 032500Z/033200Z
---

## 1. ROBLOX — ATLAS NAMED-SEED BLOCKER RESOLVED IN PLAY
Fresh 3-Play opening, repeat Play A on the fixed code (`run atlas:scout --seed 2134783819`, **16 s**, one Play):
- **Clean fast-path verdict:** `fastPathApplied=true`, `actualSeed=2134783819`, `actualLayout=2` == expected 2134783819/2,
  `rebuild=false`, `starter.applied=2134783819` at t0 **before** `rootFoundAt`, world ready after a **4.249 s build** — trace
  identical in `result.json.worldTrace` and `world-trace.json`, published before the check and retained. **⇒ assert-and-skip
  suffices; no fallback (a); the named-seed blocker is resolved.**
- Sealed BEFORE = AFTER (`84324d43…`/`c6651793…`); plays **2/3** (third reserved); Studio alive; no extra Plays/build/publication;
  no image reads; $0.
- **Cell still FAIL (exit 1), fail-closed, on a new client-side legacy API unmasked by the server path working:**
  `FogEnabled is not a valid member of Lighting` at `AtlasClient:16` — the same property is written at `tests/atlas/Server.luau:95`.
  **Fix approved (offline):** guarded capability checks in Client/Server (record availability/absence instead of raising; never claim a
  fog-off write that did not happen), tests, plus the R4/R2 residuals; freeze → checker delta → Studio cycle to a fresh 3-Play opening →
  next atlas Play. Pattern noted: each fix unmasks the next legacy-API layer (starter require → root check → client fog) — each stays a
  named, separately committed fix with a test.

## 2. WINDOWS — FIRSTLOGON COMPLETED; CHANNEL PROBES STILL PENDING
Marker evidence: `?step=start` at 03:08:35Z and **`?step=done` at 03:26:46Z** — the FirstLogonCommands (RDP enable, OpenSSH Server
capability + sshd, rig key, done marker) **ran to completion on the guest**. But live probes in my last reads were not answering
(`socat :2222` no banner; RDP X.224 no bytes — though it answered `Connection Confirm` at 03:20Z). Listeners unchanged (QEMU PID
757632, hostfwd 3389/2222, VNC 5902, monitor, marker listener). Most likely the guest is servicing a post-FirstLogon reboot/capability
install; Team B is verifying text-level and will file the settled state. **No overclaim:** the driveable claim currently rests on the
completed marker sequence + the earlier protocol-level RDP handshake; the SSH leg stays pending until a banner/key round-trip is
captured. FL2 r2 is re-filed and a fresh checker is spot-verifying the filing; no "met" wording before it.

No spend, no credential values, no image reads, no publication, desks untouched.

— delegator
