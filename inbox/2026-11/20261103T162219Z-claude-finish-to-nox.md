---
from: claude-finish
to: nox
kind: W6BS verdict
re: the 3 Nov order
---
# claude-finish -> Nox: W6BS (contained Studio spike, real bindings + containment proof) verdict PASS

VERDICT: PASS for the spike's stated job (containment proof). Native embedding is NOT complete: it fails at Studio's login screen, reported honestly below.

BRANCH / HEAD: finish/w6bs, commit 628f49e "face/studios: real launcher (systemd unit + pasta + bwrap + Wine), read-only protected-session observer, W6BS containment proof harness and live-real panel harness".

WHAT WAS BUILT AND RUN FOR REAL
- Instance 1 = street game, instance 2 = card game. Each has its own game copy, systemd unit (fleet-studio-street / fleet-studio-card), Xvfb display (:91 / :92), pasta network namespace, bwrap mount namespace, Wine prefix, Studio file copy, team domain, account label and MCP endpoint (127.0.0.1:18101 / :18102). Both show in the sidebar as "Needs login".
- Each instance runs a real StudioMCP.exe host inside its own namespace. The supervisor's MCP client completed the real handshake on both; list_roblox_studios answered [] (10 reads per instance per round). Tool lists are empty because no Studio is attached.
- 40-check proof passed 40/40 in each of 2 consecutive rounds on the committed code. Cayleb's real Studio was observed read-only before and after and sampled about every 4 s during each round: no violations. Positive control (observer reports a violation when the fake protected unit is stopped) passed.
- Negative tests ran against a fake protected session the spike started itself: all eight bind attempts refused; a registry file edited to point at the protected port refused on load; every guardSignal attempt refused; the fake session ended unchanged.
- Study of limits: one negative test is detection (tripwire), not prevention.

TEST COUNTS
- Proof harness: 40/40 x 2 rounds (builder). Checker drive: 38/38, plus 3 rounds x 2 instances of own raw JSON-RPC handshake, 0 failures.
- Whole package: base 1143 total / 1114 pass / 29 fail; head 1158 / 1129 / 29. Failing names identical, 0 head-only failures.

MUTATION RESULT
- Builder mutation field: null (none recorded).
- Checker mutant: removing --no-map-gw makes the drive FAIL (decoy reached, 2 hits); base fails at launcher import. So the namespace isolation check bites.

FRESH CHECKER VERDICT: PASS (6 checks)
1. real-instances PASS: units active, distinct InvocationIDs/cgroups/Xvfb :91/:92, passt+bwrap+socat+wine+RobloxStudioBeta each; hosts on 18101/18102; own client completed the real handshake and list_roblox_studios ({studios:[]}) on both.
2. sidebar 390/1440 PASS (limits): live-real harness in headless Chromium, 22 ok lines on rerun; per-instance game, studio-street/studio-card team label, Needs login, MCP off; layout intact at 390x844 and 1440x900. MCP shows off because the supervisor never connected; team members are harness fixture data. First cold-prefix run flaked at the crash step.
3. containment rerun PASS: own observer (flatpak/cgroup, pid@start@cmdline hash, hyprctl window, ss 13469, AT-SPI, log needles) identical before, 3 samples during, after. Own decoy: 8/8 binds refused, clean record accepted (control), 4/4 guardSignal refusals, RealLauncher.stop refused the decoy unit, unlisted decoy caught by tripwire with only handshake + one list_roblox_studios. Namespace probes: host-loopback decoy 0 hits, 13469 unreachable from inside, ~/.var hidden.
4. doc claims and publish PASS with 3 doc corrections: numbers match; no "publish" in any studios source; tool allowlist excludes publish/upload_image; call logs show only initialize/tools/list/list_roblox_studios. Misstatements D1-D3 in CHECK-W6BS.md.
5. embedding failure and owner item PASS (honest failure confirmed): Studio log in instance: "Authenticated: NO", show login dialog, "LoginDialog Error: Embedded Web Browser fail to load". x11vnc confirmed not installed. Owner items (2 accounts, x11vnc viewer route, studio-confirm-login) are concrete but mis-ordered: the agent-side network/login-page failure blocks first and is not listed first (D1).
6. suites by name PASS: identical failing names, 0 head-only.

LIMITS / NOT DONE / NOT PROVEN
- Studio never gets past its login screen in either instance. No place was opened; no real execute_luau read or edit ran. Native embedding of a working, logged-in Studio is NOT proven.
- No MCP tool use beyond handshake and list_roblox_studios (empty). Sidebar MCP status shows off; team members are fixture data.
- Containment proven for the tested paths only; one negative test is detection, not prevention.
- Owner-side items still open: two accounts, an x11vnc viewer route, studio-confirm-login. Nothing was published.
- Three checker doc corrections (D1-D3 in CHECK-W6BS.md) are not yet applied to the write-up.

NEXT STEP: fix the Embedded Web Browser/login-page failure in the instances (agent-side network first), then owner sign-in on the two accounts and rerun the proof with real execute_luau reads while Cayleb's personal Studio stays the protected session.
