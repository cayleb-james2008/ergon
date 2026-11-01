# TEAM D — ROBLOX — PHASE 1 REPORT (Jobs 16 and 17)

Lead: `roblox-lead` (delegated subagent). Machine: `outis-desktop`, Linux 7.2.8-9-omarchy, Wayland session.
Probe window: **2026-10-31 17:43–18:05 ET**. Tier: **A**. Envelope: **$0** (unstated = zero). **Publication NOT authorised.**
Checker: `roblox-checker` **NOT YET SPAWNED** — nothing in this file is a DONE; it is Phase-1 evidence + plan only.
Rule followed: **maker never checker**; every claim below is quoted with the command that produced it; anything not
verified is labelled **UNVERIFIED**.

---

## 0. Verdict in one paragraph

Both candidates were located from the projects' own records and their artifacts exist on disk (D1 done). The
**Studio-route story in the brief is partly wrong for this machine**, and that matters: this desktop already has a
working Vinegar/Roblox Studio install at **0.745** with `StudioMCP.exe` present and the launcher's pinned version
folder **not deleted**; what is actually down is (a) one disabled line in the Codex MCP config (a documented Nox
order, undo = delete the line) and (b) Studio not running / the sign-in state unverified. The "laptop (Windows)"
route is still **absent** — no SSH at all — so the laptop story remains BLOCKED-ON-MACHINE, but the games' native
plan is built for the **owner's desktop Vinegar Studio**, and that route is one line + one owner-opened Studio away.
Job 16's never-run set is fully documented by the project itself (51 native Plays in 19 Studio openings plus live
persistence/receipts scripts) and is turned into a 21-cell test matrix below with the evidence each cell must
produce. Job 17's art **exists** (24 local PNGs at 740x1024) but is **unbound** (all 24 `ArtIds` are empty strings,
no upload, no moderation); the cast is defined in `roster.md` (8 franchises x 3); a style brief derived from those
records is written below. Two new conflicts are load-bearing: the card game's on-disk build **does not match its own
recorded candidate hash**, and the creative-model rule on this machine **contradicts itself** between `~/.codex/AGENTS.md`
and `~/AGENTS.md` (2026-10-17 reversal).

---

## 1. Phase 1 — the jobs restated in my own words

**Job 16 (street game).** Take the sealed local candidate to *playtest-ready*: get Roblox Studio connected again,
then run the test classes that have never been run — inside Studio, on a physical device form factor, with two
clients at once, across a leave-and-rejoin (persistence), and through the purchase/receipt flows — fix what they
find, re-run, and hand Cayleb a packaged build plus one plain paragraph telling him what he will see, what to poke
at, and what is known-rough. Playtest-ready means **he** can play it. Never publish; never enable purchases for the
public; restore purchase prompts only in their final intended state for the receipt tests and then put them back to
the state he approved.

**Job 17 (card game).** Take the reviewed candidate to playtest-ready: 24 portraits at one consistent style,
**bound** (each card shows the portrait the data names, verified in Studio — a data-file assertion does not count),
8 audio cues as finals that are trigger-verified in Studio, a **by-test** proof that a run neither reads nor writes
legacy profile data, and the in-Studio test set green. The game stays **UNTITLED** with a fresh identity; it is never
named or branded after an earlier project. Never publish.

Shared: **one** Studio repair, done once for both games, never both at once.

---

## 2. Assumptions I am working on (each named so the Lead can strike it)

1. "Playtest-ready" for Job 16 = the project's own gate rows A1–A3, B1–B3, C1–C3 and D5 (`docs/launch/PUBLIC_FLIP_CHECKLIST.md`),
   not the whole E-row publication checklist (that is launch, and launch is not authorised).
2. The release candidate for Job 16 is `build/release.rbxlx` (1,990,548 B, `84324d43…`) built by `tools/usg build`
   from the pinned commit — **not** `build/test.rbxlx` (2,403,691 B, `c6651793…`), which the project's own runbook
   reserves for native test runs.
3. The card-game "candidate" is meant to be the artifact that `CURRENT-STATE.md` pins at `be972098…`. It does not
   match on disk (see C4); I treat candidate identity as **open** until re-established by a deterministic rebuild.
4. Studio sessions are one-at-a-time, owner-opened (the project's own runbook says so — C6), so every native cell in
   the matrices is scheduled into **Studio openings**, not into "when the agent feels like it".
5. The 24 portraits are the **existing** user-attested set; the job is to bind and verify them, not to generate a new
   cast. If Cayleb wants new art, that is a different (and creative-model-sensitive) job.
6. Zero spend: uploads to his own Roblox account are platform writes, not purchases, but they are still held for his
   word because they touch his account and Roblox moderation (Q6).

---

## 3. Conflicts, gaps, and things that cannot be verified from what I was given

**C1 — Studio is installed on THIS desktop (contradicts GROUND-STATE M4 and the brief's "Studio lives on the laptop").**
`flatpak list` → `Vinegar  org.vinegarhq.Vinegar  1.9.4  stable  user`. A complete Studio version folder exists:
`…/vinegar/versions/version-7062aec9dbe74b14/` (Oct 27 18:30) containing `RobloxStudioBeta.exe` **222,384,592 B** and
`StudioMCP.exe` **8,224,720 B**. Studio ran here as recently as **2026-10-31T00:09:17Z** — the newest log is
`0.745.0.7451137_20261029T083958Z_Studio_4AAE8_last.log` (43,104,082 B). GROUND-STATE M4's "no .desktop entries, no
binary" is right about `.desktop` and PATH binaries and **wrong about the app**: Vinegar is a flatpak app, so it
appears neither in `/usr/share/applications` nor on `$PATH`. **Consequence: the Studio route is not necessarily a
laptop-only problem; it can be restored on this machine.** This is the single most plan-changing fact in this report.

**C2 — The brief's "one-line repair" (launcher points at the deleted old version folder) does NOT reproduce here.**
The launcher resolves its version folder dynamically and the folder it points at exists:
- `~/.local/bin/roblox-studio-mcp` (1,620 B, Oct 11) — a Python wrapper: reads
  `…/prefixes/studio/drive_c/users/cayleb/AppData/Local/Roblox/mcp.bat`, regex-extracts `"Z:\…\StudioMCP.exe"`,
  requires the Vinegar flatpak instance to be running, then `flatpak enter … wine StudioMCP.exe`.
- `mcp.bat` (Oct 27 18:30) pins `…/versions/version-7062aec9dbe74b14/StudioMCP.exe` **and** has a registry fallback.
- That pinned file **is present**; so nothing points at a deleted folder on this machine.

What *is* down is a different single line — see D2.

**C3 — Job 17's "the art does not exist yet" is wrong; the art exists and is unbound.**
The brief says "ALL 24 portraits are unbound (the art does not exist yet)". On disk there are exactly **24** portrait
PNGs (`assets/images/*.png`, 740x1024 RGB, ~1.0–1.5 MB each, e.g. `DB-GOKU-01.png`, `BC-ASTA-01.png`), and
`src/shared/content/ArtIds.luau` is an auto-generated table of **24 empty strings**. So the job is **bind + verify**,
not **generate**. This changes the plan's first step and removes most of the creative-model exposure (see C7).

**C4 — The card game's on-disk build does not match its own recorded candidate (candidate identity OPEN).**
`CURRENT-STATE.md` pins the candidate as `build/UntitledAnimeCardGame.rbxlx`, SHA-256
`be972098432f5bac4ccbc67712b104f3fb26073ab303d173f10b567baa70e7aa`, **520,854 bytes**. On disk it is **520,846 bytes**
with SHA-256 `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` — **8 bytes and a different hash**.
There is also an unexplained read-only `build/helikon-candidate/HELIKON-CANDIDATE-9a98936d.rbxlx` (599,937 B, Oct 7).
No artifact on disk currently equals the reviewed candidate. Everything in Job 17 depends on which bytes are the
candidate, so this is question Q4.

**C5 — The street game's recorded offline baseline is behind its own HEAD.**
`docs/STATE.md` records "`main` = `1bef02f…`", suite "**926 passed / 0 failed across 128 specs**", Python "273 tests
OK", `store_check` PASS. Live git says HEAD is `8dfb204` ("Docs: correct the stale numbers and the broken reference
the audit found"), i.e. **two commits later**. Those offline numbers are therefore *not* measured on the current tree
and must be re-run on the exact candidate before any native result is attributed to it.

**C6 — §12 item 13 is answered by the record: the "laptop window" IS a Cayleb act.**
PROGRAMME §12.13 says treat the Studio window as "a scheduling dependency, not a Cayleb act, unless Phase 1 shows
otherwise". It shows otherwise. The project's own runbook (`docs/verify/2026-10-19/vinegar/SPRINTS.md`) states:
"**The owner opens Studio; the agent never does.** Runbook steps 4-7: park the window, open the place, sign in as
Chrono_H3X if asked, leave it in Edit, say go. One Studio per Roblox login, ever: two Studios revoked that login on
2026-10-16." So every Studio opening in the matrices requires Cayleb to open Studio (≈1 minute each, 19 openings for
the street game's native set as planned) — that is scheduling, but it is **his hands**, and it is the reason the job
cannot be done in one unattended overnight run.

**C7 — The creative-model rule on this machine contradicts itself; this DSH team is not on the model it names.**
- `~/.codex/AGENTS.md:29`: "Keep the lead and every subagent in every role on **GPT-6.1 Sol/high**, as selected by
  Cayleb… do not silently substitute Luna, another model or a cheaper QC/research worker."
- `~/AGENTS.md:1-3`: "## Creative model — Cayleb, 2026-10-17 (**replaces the Sol lock**)… creative work uses the
  strongest model available, or the model he names. **Do not force GPT-6.1 Sol. Do not refuse a session because it is
  not Sol. The Sol-only paragraph below is history.**"
This session runs `deepseek-v4.1-flash`, so it is **not** GPT-6.1 Sol and cannot become it. Under `~/AGENTS.md`
(dated later, and explicitly a reversal) that is permitted; under the stale paragraph in `~/.codex/AGENTS.md` it
would not be. PROGRAMME §2.9 names **no** Roblox/creative model policy, and neither job brief names one. **Flagged to
the Lead for Cayleb — Q1.** Note the exposure is now small: the portraits exist; the job is binding, verification and
two finals, not new creative generation. Any *new* art or a re-shoot of the 8 cues would land squarely in it.

**C8 — No route to the laptop, but the tailnet is not empty (both facts, plainly).**
`~/.ssh` does not exist (`ls: cannot access '/home/cayleb/.ssh/': No such file or directory`), and `getent hosts muse`
returns nothing (`rc=2`). Tailnet (`tailscale status`): `muse 100.110.154.92 … active` (ICMP from here: 1 packet,
0% loss, 235 ms, relayed "nyc"); `outis 100.120.51.19 windows … offline, last seen 151d ago`;
`cayleb-desk 100.95.153.50 windows … offline, last seen 253d ago`. So: **IP reachability to `muse` exists**, but no
SSH client config/keys means no authenticated route, and the two Windows machines have been offline for 5 and 8
months — if "the laptop" is one of those, it has not been on the tailnet since spring. I could not verify anything
about a laptop's Studio (version, sign-in page, launcher) from here: **UNVERIFIED**.

**C9 — The street game has no named "purchase prompts disabled" switch in source that I could find in this pass.**
The brief says "purchase prompts are currently disabled". `src/server/Services/Monetization.luau` exposes the prompt
paths (`PromptGamePassPurchase` at `:471`, `PromptProductPurchase` at `:480`, `PromptRobloxSubscriptionPurchase` at
`:446`) and installs `MarketplaceService.ProcessReceipt` at `:435`; no `_ENABLED`/`DISABLE` constant exists in
`GameplayConfig.luau` or `AudioConfig.luau`. **Where the disable lives is UNVERIFIED** and is the first action of the
receipt cell (R1): locate the switch, record it, and prove both states.

**C10 — Legacy-profile data is never defined by name anywhere I can read.**
I searched both repos for "legacy": the only hits in the card game are the **Legacy rarity flag** (a retention
identity inside its own roster) and prose. The card game's own store scope is explicit: "Production path:
`DataStoreAdapter` on the explicit `UACG_DEV_v1`" (`docs/persistence.md:122`), plus a dev probe store
`uacg-dev-probe` (`src/server/Bootstrap.luau:302`). So the safety proof can be built (**instrument the DataStore
service, assert the only names touched are the allowed two, over a real play session**), but *which* data counts as
"legacy profile data" is Cayleb's definition, not mine — Q5.

---

## 4. D1 — The two candidates, located from the projects' own records

### 4.1 Street game — `~/Work/projects/untitled-street-game`

The project names itself and its live place in its own README:

> `README.md`: "A Roblox night-driving game: run taxi fares, shake the cars tailing you, win street sprints, and build
> a garage of original 80s/90s tuner cars, classics by now, in Kuroyo City, 2077."
> "Play / store page: https://www.roblox.com/games/107924030342600 (private until launch)"

The platform state is pinned in its own records:

> `docs/STATE.md`: "Universe `10770936993`, root place `107924030342600`, owner Chrono_H3X (1170918805). PRIVATE,
> published version 7 (source `342b355`, release SHA-256 `5014db1f...` retained as rollback). Computer, phone and
> tablet enabled; console and VR off; server size 20. 7 passes and 5 products exist at fixed prices… Managed Pricing off".
> `docs/STATE.md`: "**Current:** `main` = `1bef02facdb90f340637c1d2595a4496ae5b4ef1` … Lune **926 passed / 0 failed
> across 128 specs** … Python `tests/tools` **273 tests OK** … `python3 tools/store_check.py` **PASS** … **Nothing has
> been re-run natively on this source yet.**"

Live git and the build artifacts on disk (my runs):

```
$ git -C ~/Work/projects/untitled-street-game log --oneline -1
8dfb204 Docs: correct the stale numbers and the broken reference the audit found
$ git status --short | grep -v '^??'      # (nothing — tracked tree clean)
$ sha256sum build/release.rbxlx build/test.rbxlx
84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce  build/release.rbxlx   (1,990,548 B, Oct 21 03:42)
c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98  build/test.rbxlx      (2,403,691 B, Oct 21 03:42)
```

The release history is recorded in the repo, so the sealed candidate has a lineage:

> `docs/RELEASE_LOG.md`: "| 2026-10-16 10:45 | Saved | 7 | 342b355 | 5014db1f4d1235a7 |" and the same row Published
> (version 7, the rollback artifact at `~/Work/session-artifacts/street-launch-20261018/rollback-v7.rbxlx`,
> 1,821,671 B, hash `5014db1f…` per `PUBLIC_FLIP_CHECKLIST.md` row F1).

**What has never run** is stated by the project itself, in one line — this is the Job 16 scope, verbatim:

> `docs/STATE.md:27`: "Never done: listening to the 27 audio files, physical phone and tablet, **two clients**, live
> persistence and receipt retries, performance budgets, truthful rating, eligible non-owner opening."

and, for every lane, the same honest limit repeats: "**nothing has run in a real Studio since 2026-10-18**, so every
native, GPU, device, audio-listening and live-persistence check is still open."

### 4.2 Card game — `~/Work/projects/untitled-anime-card-game`

> `CURRENT-STATE.md`: "**LOCAL READINESS ONLY — NOT A PUBLIC RELEASE.** The current candidate is
> `build/UntitledAnimeCardGame.rbxlx`, SHA-256 `be972098…a70e7aa`, 520,854 bytes."
> `CURRENT-STATE.md`: "1. **Studio sign-in and Play:** no supported signed-in Studio session or Play run is evidenced
> for this hash… 2. **Creator, universe, and place authorization:** no authorized target is recorded;
> `ops/release/target.json` is absent. 3. **Art upload and moderation:** the 24-image set is user-attested for project
> use, but no upload or moderation result is recorded and every Roblox art ID remains empty."
> `docs/VISUAL-DIRECTION-20260926.md`: "Roblox `ArtIds` stay empty until the selected sources are uploaded to the
> verified creator/game target and Roblox moderation results are recorded."

Live disk state (my runs): 24 PNGs present; `src/shared/content/ArtIds.luau` is a 24-row table of `""`; build hash
mismatch as in C4; `ops/release/target.json` absent; `git status --short` shows ten modified tracked files
(`CURRENT-STATE.md`, `README.md`, `TASKS.md`, `VALIDATION.md`, `assets/images/manifest.json`, `docs/ART-PROVENANCE-CURRENT.md`,
`docs/LAUNCH.md`, `docs/RIGHTS-DECISION-20260926.md`, `docs/STUDIO-LAUNCH.md`, `docs/brief.md`) — i.e. the working tree
is **dirty**, which must be reconciled before a candidate is declared.

### 4.3 What the two Studio attempts in the card game's record actually proved

> `docs/STUDIO-ISOLATION-20260926.md`: "**Verdict: isolated Edit script PASS; isolated Play BLOCKED by
> authentication.**… The private Studio log reported `Authenticated : NO`; the isolated virtual display showed Studio
> sign-in."
> `CURRENT-STATE.md`: "The native Roblox UI… the game's own in-engine check suite reported 13/13 PASS" — obtained at
> commit `9b33523` ("Studio-proven: agent-controlled Play run passes studio-device gate 13/13").

So the card game has had **one** authenticated Play session on an **older hash**, and every claim since is staging.
The blocker is authentication, not code.

---

## 5. D2 — The Studio route: what exists, what is blocked, and the exact HELD repair

### 5.1 The route on this desktop (all quoted from live runs, 17:44–18:04 ET)

```
$ flatpak list | grep -i vinegar
Vinegar  org.vinegarhq.Vinegar  1.9.4  stable  user
$ ls …/vinegar/versions/
version-7062aec9dbe74b14                      # Oct 27 18:30 — the ONLY version folder
$ ls -la …/version-7062aec9dbe74b14/*.exe
RobloxStudioBeta.exe   222,384,592   Oct 27 18:30
StudioMCP.exe            8,224,720   Oct 27 18:30
NativeDialog.exe / RobloxCrashHandler.exe           # present
$ flatpak ps --columns=instance,application      # (empty — Studio is NOT running)
$ ~/.local/bin/roblox-studio-mcp
Error: Roblox Studio (org.vinegarhq.Vinegar) is not running in Flatpak.
$ ls -t …/AppData/Local/Roblox/logs/ | head -1
0.745.0.7451137_20261029T083958Z_Studio_4AAE8_last.log     # 43,104,082 B; last lines 2026-10-31T00:09:17Z
```

The newest Studio log contains one `Authenticated : NO` and one `Authenticated : YES`, and ends with
`[FLog::LoginController] Exiting user scope` at `2026-10-31T00:09:17.390Z` — i.e. the last Studio session started
2026-10-29 08:39Z, had a signed-in scope at some point, and closed cleanly on **Oct 30 at 20:09 ET**. Whether the
sign-in **still** works today is **UNVERIFIED** (proving it requires launching Studio — held; see 5.3).

### 5.2 The three actual blockers, in order

| # | Blocker | Evidence | Owner | Cost |
|---|---|---|---|---|
| B-1 | The Codex MCP Studio server is **switched off by one line** | `~/.codex/config.toml:190-192`: `[mcp_servers.roblox_studio]` / `command = "/home/cayleb/.local/bin/roblox-studio-mcp"` / `enabled = false   # DISABLED 2026-10-28 by the consolidation lead (Nox order street-unblock): native Roblox fixtures need exclusive port 13469; undo = delete this line (backup: lanes/consolidation-20261027/A/codex-mcp-disable-20261028/config.toml.pre-disable)` | Lead/Cayleb ruling (Nox order still standing) | 1 line |
| B-2 | Studio is not running | `flatpak ps` empty; launcher probe above | **Cayleb's hands** (runbook: owner opens Studio, agent never does; one Studio per login) | ~1 min |
| B-3 | Sign-in state unknown / possibly revoked | brief: "token was revoked at Roblox's end"; log shows a login cycle and a clean exit; **no fresh credential demand may be attempted** | Cayleb (existing session first; a fresh credential demand **stops for him**) | ~1 min if session valid |

### 5.3 The exact repair to perform ONCE, for both games — **HELD**

**R-1 (the real one-line repair).** File: `/home/cayleb/.codex/config.toml`
- Old line (line 192): `enabled = false   # DISABLED 2026-10-28 by the consolidation lead (Nox order street-unblock): native Roblox fixtures need exclusive port 13469; undo = delete this line (backup: lanes/consolidation-20261027/A/codex-mcp-disable-20261028/config.toml.pre-disable)`
- New line: *(deleted — the recorded undo is "delete this line")*
- Pre-condition: the two games are not both going to want port **13469** at once ("Do not run both Studio repairs at
  once — there is one shared repair for both games"), and the Nox order that disabled it was about **native Roblox
  fixtures** contending for that port. So the repair is not just a line — it is a **scheduling ruling** (Q3).
- Verification of the repair (when un-held): `grep -n "enabled" ~/.codex/config.toml` shows no `enabled = false` under
  `[mcp_servers.roblox_studio]`, then, with Studio already running and signed in, the launcher's `tools/list`
  handshake answers — record the exact bytes of that handshake as evidence.

**R-2 (the brief's repair, kept as the fallback only).** File:
`~/.var/app/org.vinegarhq.Vinegar/data/vinegar/prefixes/studio/drive_c/users/cayleb/AppData/Local/Roblox/mcp.bat`
- Old line (line 2): `if exist "Z:\home\cayleb\.var\app\org.vinegarhq.Vinegar\data\vinegar\versions\version-7062aec9dbe74b14\StudioMCP.exe" ( "Z:\…\StudioMCP.exe" %* ) else (for /f "tokens=2*" %%A in ('reg query HKEY_CURRENT_USER\Software\Roblox\RobloxStudio /v ContentFolder') do ( "%%B/..\StudioMCP.exe" %* ))`
- New line: replace the pinned `version-7062aec9dbe74b14` with whichever `version-<hash>` folder Studio creates on its
  next auto-update, **or** delete the `if exist` clause entirely so the registry `ContentFolder` fallback always runs.
- Why it is not tonight's repair: the pinned folder **exists**, so this line is currently correct. It becomes the
  repair only after an auto-update deletes the old folder — which is exactly what the brief describes happening on the
  **laptop**, and that machine is unreachable from here (C8). **UNVERIFIED as a live defect.**

**Both R-1 and R-2 are HELD. Nothing was edited.** Neither was attempted twice; no Studio was launched; no second
Studio was created; no credential was requested or handled.

**Sign-in hold (part of the same repair, never separate).** After Studio is opened by Cayleb and left in Edit mode:
verify the account is `Chrono_H3X` (1170918805) from inside Studio's own UI, do **not** type credentials, and if a
fresh credential or 2FA prompt appears, **stop — that is Cayleb's**. Record the account name shown + the Studio
version string (`0.745.0.7451137`) as the repair's evidence.

---

## 6. D3 — Job 16: the never-run test set as a runnable matrix

Every cell below comes from the street project's own records (its gate rows, its runner, its live scripts), not from
my invention. "Evidence the cell must produce" is the acceptance object: if a cell cannot produce exactly that, it is
**FAILED**, not "probably fine".

**Cell classes.** A = in-Studio behaviour · B = native fixture Plays (GPU host) · C = device · D = two clients ·
E = persistence across rejoins · F = receipt flows · G = audio/performance.

| ID | Test | Exact invocation | Evidence the cell must produce | Now |
|---|---|---|---|---|
| A0 | Offline baseline on the exact candidate (not the stale one) | `TMPDIR=~/.cache/usg-tmp tools/usg test` then `python3 -m unittest discover -s tests/tools` then `python3 tools/store_check.py` | log lines with **total passed/failed/spec count**, exit 0, run on the recorded commit + `sha256sum build/release.rbxlx`; the numbers must match the doc or the doc must be corrected in the same commit | OPEN (C5) |
| A1 | In-Studio behaviour: 14 studio specs | `tools/usg test --studio` (needs a connected, signed-in Studio) | per-spec pass/fail lines for `tests/studio/{baseline-mechanics, customization, driving-modes, driving, gameplay-settings, game, highway, showroom, traffic-shared, ui, visual}.spec.luau` + the 3 runtime `.luau` (`CareersRuntimeServer`, `PhysicsIntegration`, `TestCourse`); exit 0; Studio log path; place hash = `c6651793…` (test place) | OPEN — needs Studio |
| A2 | First-five-minutes inside a real client | `python3 tools/vinegar_run.py run first-fare` / `run moving` (runner, not Lune) | the runner's JSON result + screenshots of HUD/hint/result/phone layout; note the offline rig is *not* a human (STATE.md: "the first five minutes are really judged in B3 and C3") | OPEN |
| B1 | Handling matrix + city/traffic/race fixtures on a quiet host | `python3 tools/hostcheck.py preflight` then the 10 B1 Plays | `quiet: true` preflight output; per-Play exit code **0** (1 = real finding, 2 = preflight refused, 4 = host-contaminated, 5 = crash); JSON result per Play; `verify-clean` at the sprint end | OPEN, never run (SPRINTS.md: "This is a PLAN. Nothing in it has ever been run.") |
| B2 | Traffic + rival gates (4 Plays) | as above | same evidence shape; mandatory fixtures `traffic:pass`, `traffic:abort-contact` require a quiet host | OPEN |
| B3 | Look gate (12 Plays) + independent critic | the 12 flagged `run` commands in SPRINTS.md | look-metric thresholds met **and** an independent critic ≥4 on every rubric category against the locked anchor board `docs/ui-research/anchors/anchors_v1.png`; critic's name/value recorded, never self-graded | OPEN |
| B4 | Feel & performance sprint (6 Plays) | SPRINTS.md B4 | frame time, memory, part counts, phone/tablet budget estimates, on a real GPU | OPEN |
| C1 | Physical phone | owner's score sheet `docs/launch/OWNER_PLAYTEST.md` | his filled sheet (1–5 per axis) + device model + screenshots; **not Studio emulation** (the row explicitly separates them) | OPEN — needs an uploaded/saved version + his device |
| C2 | Physical tablet | as C1 | as C1 | OPEN |
| D1 | Two clients at once (local multi-client under Vinegar/wine if it works) | local server + 2 clients in Studio/wine | two client logs with **two distinct userIds**, one server log, and a synced moment (both see the other's car/fare state); if wine cannot, the run is **labelled simulated** with the attempt's exit evidence — the plan already allows that (`COMPLETION_PLAN.md:85,125`) | OPEN, never attempted |
| E1 | Live persistence across rejoins | `tools/usg luau tests/live/persistence.luau --version N` (needs a **saved** version + key) | script output + the DataStore round-trip it prints + `PROFILE_VERSION` unchanged (6) + a second join showing the same profile | OPEN |
| E2 | Rejoin simulation (offline twin) | `tests/playtest/rejoin.luau` in the runner | run output; labelled **simulated**, not live | OPEN |
| F0 | Locate and record the purchase-prompt state switch | grep the prompt path (`Monetization.luau:471,480,446`) + the place's own state | the exact constant/property that disables prompts, quoted; both states proven by a toggle run | OPEN — UNVERIFIED where it lives (C9) |
| F1 | Receipt flow with prompts in their intended final state | `tools/usg luau tests/live/receipts.luau --version N` | receipt log per purchase: `PurchaseId` **granted once**, retry does **not** double-grant, no grant before profile load, rollback path recorded; then prompts returned to the state Cayleb approved, and that restoration proven | OPEN; a real 39 R$ purchase needs **his separate Robux yes** (`PUBLIC_FLIP_CHECKLIST.md` D5) |
| F2 | Store/receipt offline twin | `tests/playtest/store.luau` | run output; simulated label | OPEN |
| G1 | Audio reel: 27 runtime files listened | `tools/audio_review.py` reel + the owner's listening | reel file list with hashes, plus **listening actually performed** — by Cayleb (his score sheet) and/or an independent critic; "listening NOT PERFORMED" stays in the label if not done | OPEN (27 never heard) |
| G2 | Performance budgets inside Studio | part/frame/memory measurements during A2/B4 | measured numbers against the documented caps, on the host named | OPEN |
| H | Packaging + his note | — | `build/release.rbxlx` hash + version bump + one paragraph (what he sees / what to poke / what is known-rough) + the prompts-state note | NOT STARTED |

**Sequencing rule for the matrix.** A0 must pass before any B/C/D/E cell is attributed to this candidate (C5). A1
must pass before B1 (the runner needs a Studio that is healthy). B1–B4 need **19 Studio openings** between them; each
opening is 1–3 Plays, then Cayleb reopens Studio. C/E/F need a **saved** version; D needs a local server. Nothing in
this matrix is a "two-minute errand": A1 ≈ 30–45 min, B1–B4 ≈ one afternoon of openings, C ≈ 30 min of his time,
D ≈ 30–45 min, E/F ≈ 45–60 min.

---

## 7. D4 — Job 17: art/audio inventory, style brief, binding plan, safety proof

### 7.1 The 24 portraits — inventory

| Fact | Value | Source |
|---|---|---|
| Files | exactly **24** PNGs in `assets/images/` | `ls assets/images/*.png \| wc -l` → `24` |
| Format | PNG, **740 x 1024**, 8-bit RGB, ~1.0–1.5 MB each | `file assets/images/DB-GOKU-01.png` → `PNG image data, 740 x 1024, 8-bit/color RGB` |
| Ids | `BC-ASTA-01 BC-SEA-01 BC-WIND-01 BK-FALCON-01 BK-GUTS-01 BK-WIND-01 BL-BYAKUYA-01 BL-FROST-01 BL-ICHIGO-01 DB-GOKU-01 DB-NAMEK-01 DB-PRINCE-01 MHA-BLAST-01 MHA-DEKU-01 MHA-FROSTBURN-01 NR-KAKASHI-01 NR-NARUTO-01 NR-SHADOW-01 VG-HEAD-01 VG-MUSASHI-01 VG-RIVAL-01 VS-BEAR-01 VS-SNAKE-01 VS-THORFINN-01 | `src/shared/content/ArtIds.luau` |
| Bound? | **0 / 24** — every value is the empty string: `-- AUTO-GENERATED by ops/release/publish.py upload-art. Empty string = no uploaded asset (placeholder art shown).` | `cat src/shared/content/ArtIds.luau` |
| Rights | **USER-ATTESTED**, not cleared: "status remains USER-ATTESTED, not independently cleared; keep that wording and do not claim public rights clearance" | `CURRENT-STATE.md` gate 6 |
| Upload/moderation | none recorded; "every Roblox art ID remains empty" | `CURRENT-STATE.md` gate 3 |
| Target | `ops/release/target.json` **absent** — no authorised creator/universe/place | `CURRENT-STATE.md` gate 2 |

### 7.2 The cast (defined by the records — this is the cast, not one I chose)

> `docs/specs/spec-untitled-anime-card-game/roster.md`: "## Fixed franchises (user-confirmed, eight)
> Bleach, Naruto, Dragon Ball, Berserk, Vinland Saga, Vagabond, My Hero Academia, Black Clover."
> "## Proposed 24-character ceiling (three per franchise…)
> - Bleach: Ichigo Kurosaki, Rukia Kuchiki, Byakuya Kuchiki — one explicit Soul Reaper-era costume each.
> - Naruto: Naruto Uzumaki, Sasuke Uchiha, Kakashi Hatake — one Shippuden-era reference set each.
> - Dragon Ball: Son Goku, Vegeta, Piccolo — fixed Z-era base forms; transformations are separate editions.
> - Berserk: Guts (Black Swordsman), Casca (Golden Age), Serpico (one verified later-era outfit).
> - Vinland Saga: Thorfinn (young vs adult resolved to one first edition), Askeladd, Thorkell.
> - Vagabond: Miyamoto Musashi, Sasaki Kojiro, Seijuro Yoshioka — manga pipeline; unverified colors labeled proposed.
> - My Hero Academia: Izuku Midoriya, Katsuki Bakugo, Shoto Todoroki — one hero-costume revision each.
> - Black Clover: Asta, Yuno, Noelle Silva — one early-series base outfit each."

Mapping the 24 file ids onto the roster's 24 names is **not** asserted here — it must be read from
`assets/images/manifest.json` (54 KB, with `art_provenance` and a hash-bound 24-row inventory in
`docs/ART-PROVENANCE-CURRENT.md`) as **phase step P2**, and the mapping is exactly what the binding test will prove,
card by card. Guessing an alias-to-character mapping would be inventing.

### 7.3 The 8 audio cues — inventory (all unbound, all "listening NOT PERFORMED, Studio NOT RUN")

| # | purpose | dur | local file | roblox_id | status |
|---|---|---|---|---|---|
| 1 | `contact_normal` | 2.0 s | `contact_tap_v1.wav` | **null** | finished local; listening NOT PERFORMED, Studio NOT RUN |
| 2 | `contact_light` | 2.0 s | `contact_tap_light_v1.wav` | **null** | same |
| 3 | `contact_heavy` | 2.0 s | `contact_thud_heavy_v1.wav` | **null** | same |
| 4 | `contact_critical` | 2.0 s | `contact_crack_critical_v1.wav` | **null** | same |
| 5 | `card_move` | 1.0 s | `card_slide_whoosh_v1.wav` | **null** | same |
| 6 | `ui_click` | 1.0 s | `ui_click_v1.wav` | **null** | same |
| 7 | `menu_music` | 45.0 s | `menu_theme_v1.flac` | **null** | finished local (job ctk-8feb3a631a35, 2026-09-21) |
| 8 | `battle_music` | 40.0 s | `battle_theme_v1.flac` | **null** | finished local (job ctk-ea6be2649dd9, 2026-09-21) |

Source: `assets/audio/manifest.json` (8 entries, `roblox_id: null` on every one). The current build's sound is
**engine-bundled stand-ins** bound in `9b33523`: "all 8 audio cues had null soundIds … Fixed by binding verified
engine-bundled `rbxasset://sounds/…` stand-ins" (`CURRENT-STATE.md:102-104`). So the stand-ins are what make the
13/13 Studio gate possible today; finals must replace them **and** beat them in a listening check.

### 7.4 The style brief (derived from the design records — no invented cast, no legacy branding)

**Identity.** The game is **UNTITLED**. Working title only (`UntitledAnimeCardGame`); it is never named or branded
after an earlier project, and it never reads, writes or migrates legacy profile data. Anything in the repo that looks
like a discarded identity (e.g. `build/helikon-candidate/`) is **evidence to be explained, not a name to adopt**.

**World and card treatment** — from `docs/VISUAL-DIRECTION-20260926.md`:
- The reference is a bright, open-air **Skybridge** battle scene with two upright, **flat** cards meeting at centre;
  small player/rival health panels above; an ordered four-card reserve strip with Auto, speed, Skip and settings
  below. Both Dark and Light variants exist.
- **Cards stay flat cards.** The card face is **monochrome**; family colour lives in the world/UI, never on the card
  face. The previous family-coloured geometric treatment and the centred "Duel Archive" shell are **superseded**.
- Unbound art shows an explicit **ART PENDING** state — never a guessed placeholder portrait.

**Portrait brief (the 24):**
- Format: **740 x 1024** portrait crop (the existing set's exact size — keep it so the card view's aspect does not
  change; a different aspect is a UI change, not an art change).
- One figure per portrait, single canonical form per the roster's per-character instruction (e.g. one Soul
  Reaper-era costume for Bleach; fixed Z-era base for Dragon Ball; one first edition for Thorfinn; unverified colours
  labelled *proposed* for Vagabond).
- Read as a flat card illustration, not a poster: centred figure, quiet background that survives being shrunk to the
  card's monochrome treatment, no baked text, no series logo, no watermark, no UI furniture.
- Consistency is enforced across all 24 by **one** treatment pass: same crop, same lighting key, same contrast
  envelope, same line weight, same background value range. A portrait that needs a different treatment to read is
  rejected, not special-cased.
- Provenance per image is mandatory (the roster's own image-job contract): canonical character id, series, form id,
  reference-set version, display alias, local reference path + hash + conditioning role, model/graph revision hashes,
  prompt/seed/sampler/steps/CFG, output purpose and crop, rights status, licence snapshot, provenance, output hash,
  validation results, rejection reasons.
- **Display alias is separate from the canonical name**: the card shows the alias; the data names the canonical id.
  The binding test compares *the id the card shows* to *the id the data names* — not a pretty label.

**Audio brief (the 8):** all original; SFX dry and isolated (no speech, no music baked in); UI/contact cues short
(1–2 s), music 40–45 s and loop-verified by test; contact cues fire at the actual visual contact; separate saved
Music/SFX/UI volumes; manifest per asset with the Roblox ID only when real (null while pending).

### 7.5 Binding and verification steps (the only acceptable proof shape)

1. **Re-establish the candidate** (C4) — deterministic rebuild from the pinned commit, twice, identical hash, receipts
   written. Done-condition: two builds byte-identical + a receipt naming commit, toolchain and hash.
2. **Read the 24-row mapping** from `assets/images/manifest.json` + `docs/ART-PROVENANCE-CURRENT.md`; write the
   alias→canonical-id→file table; every row hash-checked against the file on disk. Done: 24/24 rows hash-match.
3. **Upload** the 24 (and the 8 finals) to the authorised target via the project's own path (`ops/release/publish.py
   upload-art`; `tools/apply_art_ids.py` "Bind one Roblox asset ID only from a complete offline evidence receipt").
   Each upload needs: complete offline evidence receipt, moderation result, real asset id. **Credential-gated**
   (Open Cloud) and **account-touching** → held for Cayleb's word (Q6). Done: 24 ids + 8 ids recorded, each with its
   receipt and moderation result.
4. **Bind**: `apply_art_ids.py` writes `ArtIds.luau` (auto-generated table) from the receipts; rebuild. Done: 24/24
   non-empty ids in the built place, + 8 sound ids.
5. **Verify in Studio, per card, per cue** — the part that counts:
   - for each of 24 cards: open the card view in Studio, screenshot it, and record the card id + the asset id the
     running game reports for that card; the screenshot's portrait must be the portrait the data names;
   - for each of 8 cues: trigger it in the real flow (contact light/normal/heavy/critical, card move, UI click, menu
     music, battle music) and record the play log + a listening verdict;
   - Done: **24/24 screenshots + 8/8 trigger records**, each naming card id, asset id, hash of screenshot, Studio log
     line, and the version string. A data-file assertion alone is **explicitly not admissible**.
6. **Legacy-profile safety proof — by test.** Run a full session (draw → battle → reward → claim → save → rejoin)
   under an instrumented DataStore adapter that records every store name/key opened, then assert the recorded set is
   a subset of the allowed names (today: `UACG_DEV_v1` and the dev probe `uacg-dev-probe`) and that **zero** reads or
   writes hit any legacy name. Done: the recorded call log + the assertion output + a negative control (a deliberately
   planted legacy-name read makes the proof fail). **Blocked on the definition of "legacy" — Q5.**
7. **Test set with art and audio in place**, fix, re-run, package with the one-paragraph note. Done: the matrix rows
   A1-style for the card game + the note.

---

## 8. Phase-1 plan — large steps, each with a checkable done-condition

**Job 16 (street).**

| # | Step | Size | Done-condition (checkable) |
|---|---|---|---|
| S1 | **Route restoration, once for both games** (R-1 after the Q3 ruling; Cayleb opens Studio; sign-in verified in-UI; record version + account) | 30–45 min, needs Cayleb ~1 min | `roblox_studio` MCP handshake bytes recorded; Studio log line showing the session; account `Chrono_H3X` visible in Studio's own UI; **one** Studio only |
| S2 | **Candidate re-measure (A0 + C5)** on the recorded commit | 45–60 min | full Lune + Python + store_check totals on `8dfb204`, exit 0, with `sha256sum build/release.rbxlx`; totals in `docs/STATE.md` corrected if they moved |
| S3 | **In-Studio behaviour (A1 + A2)** | 45–60 min per opening; 2 openings | 14 studio specs + first-fare/moving runs, exit 0, logs + place hash `c6651793…`; any failure kept as a failure |
| S4 | **Native fixtures B1–B4** (51 Plays / 19 openings) | one afternoon of openings | per-Play exit 0 + JSON + `verify-clean` + `hostcheck` quiet; real findings (exit 1) kept, not re-run to pass |
| S5 | **Device + two clients + persistence + receipts (C1, D1, E1, F1)** | 2.5–3.5 h incl. his device time | his filled score sheet; two-userId client logs (or honest "simulated" label); persistence script output on a saved version; receipt log with once-only grants and prompts restored |
| S6 | **Package + note** | 30 min | release build hash + one paragraph + the prompts-state note + rollback line |

**Job 17 (card).**

| # | Step | Size | Done-condition |
|---|---|---|---|
| T1 | **Candidate identity (C4)** — reconcile dirty tree, deterministic rebuild twice, receipts | 45–60 min | two byte-identical builds + receipts pinning commit/toolchain/hash; the recorded `CURRENT-STATE.md` hash corrected or the artifact restored |
| T2 | **Mapping table** (24 rows: alias → canonical id → file → hash) | 45 min | 24/24 rows hash-match on disk; no guessed mapping |
| T3 | **Upload + bind** (held on Q6) | 1–1.5 h | 24 ids + 8 ids, each with receipt + moderation result; `ArtIds.luau` 24/24 non-empty; rebuilt place carries them |
| T4 | **Per-card and per-cue Studio verification** | 1.5–2 h per Studio opening | 24/24 screenshots matched to data-named ids, 8/8 trigger records, logs + version string |
| T5 | **Legacy-profile safety proof** (after Q5) | 1 h | instrumented run log + subset assertion + negative control fails as designed |
| T6 | **Test set with art/audio in place + package + note** | 45–60 min | card-game matrix green (or named failures), package + paragraph |

Both plans are gated, in order, by: **Q3 → S1/T3 → the rest**. Neither job can reach its done-condition without a
Studio window that Cayleb opens.

---

## 9. Questions the Lead must batch to Cayleb (each with my recommendation)

**Q1 — Creative model.** `~/.codex/AGENTS.md` still says "GPT-6.1 Sol/high, as selected by Cayleb" for every role;
`~/AGENTS.md` (2026-10-17) says that Sol lock is replaced: "creative work uses the strongest model available, or the
model he names. Do not force GPT-6.1 Sol." This Roblox team runs `deepseek-v4.1-flash`, so it is not Sol. **Is that
acceptable for the Roblox art/creative work now (which is binding and verifying the existing 24, not generating new
art), or must any creative leg wait for a Sol lane?** *Recommendation: acceptable — the dated reversal governs and no
new art is being generated; if he wants a new set of portraits or a re-shoot of the 8 finals, that leg waits.*

**Q2 — Which Studio, whose hands.** The brief says Studio is on the laptop (Windows); the record shows this desktop
has Vinegar Studio 0.745 with `StudioMCP.exe` present and the games' whole native plan built for it; the laptop has
no authenticated route from here (no SSH; the two Windows tailnet machines are 151 and 253 days offline). **May the
route be restored on this desktop's Vinegar Studio instead of waiting for a laptop window?** *Recommendation: yes —
this desktop, because the street game's 51-Play plan is written for it and the card game's last Play ran on a local
Vinegar Studio; the "laptop" leg is unverifiable from here and would add days.*

**Q3 — The disabled MCP line (Nox order vs Job 16/17).** `~/.codex/config.toml` disables `roblox_studio` since
2026-10-28 by a Nox order because "native Roblox fixtures need exclusive port 13469". **May the line be deleted for
the shared Studio repair, with the two games sequenced so they never both want the port?** *Recommendation: yes, as
the shared repair, run once; tell Nox so the ordering is on his board.*

**Q4 — Card-game candidate.** The reviewed candidate's recorded hash (`be972098…`, 520,854 B) is **not** the file on
disk (`55cef5d3…`, 520,846 B); a read-only `HELIKON-CANDIDATE-9a98936d.rbxlx` (599,937 B, Oct 7) also exists. **Which
artifact is the reviewed candidate — rebuild it deterministically, or is the Helikon file the one?** *Recommendation:
rebuild deterministically from the commit that `CURRENT-STATE.md` was written against, pin the rebuild by receipt,
and treat Helikon as an unexplained artifact to archive, not to build on.*

**Q5 — What exactly is "legacy profile data"?** The card game's own store scope is `UACG_DEV_v1` (+ dev probe
`uacg-dev-probe`); no document in either repo names the legacy data the game must never touch. **Name it (store
scope / keys / where it lives) so the safety proof can be a test rather than a promise?** *Recommendation: define it
as "everything except `UACG_DEV_v1` and `uacg-dev-probe` on any DataStore", and make the proof assert the whole
observed store-name set is a subset — that is provable tonight without naming another service.*

**Q6 — 24 art uploads to his account.** Binding needs real Roblox image asset ids; every one is an upload to **his**
account and goes through Roblox moderation; the set is user-attested, not rights-cleared, and the game is unlisted.
**Does he authorise the 24 portraits + 8 audio finals to be uploaded to his account for binding (not published)?**
*Recommendation: yes, upload to the private/unlisted target only, keep publication not authorised, and record each
moderation result — this is not publication and is the only way "bound and seen correct in Studio" can be true.*

**Q7 — Audio listening and device passes are his body, not the agent's.** 27 street files never heard; phone and
tablet never touched. **When can he give 10 minutes to the audio reel and ~15 minutes to the phone/tablet score
sheet, in the same window as the Studio opening?** *Recommendation: attach both to the first Studio window — the
reel plays while the place loads, the device pass follows the first saved version.*

---

## 10. Status, spend, holds

- **Status:** Jobs 16 and 17 = **BLOCKED-ON-MACHINE** (Studio window) + **PREP COMPLETE** for D1–D4. No job is DONE;
  no checker has run; no item in this file may be reported as done.
- **Spend:** $0 of $0. Nothing purchased, no account created, nothing published, no wallet/trading action touched.
- **Holds respected:** publication not authorised; purchase prompts not enabled for the public; no credential read,
  printed, copied or logged (the Open Cloud key's *existence* is noted only as a file path; its content was never
  opened); maker ≠ checker; `~/Work/shared-workflows/lanes/board/**` untouched (read-only); no Studio launched, no
  launcher edited, both repairs HELD.
- **Next step + successor:** Q1–Q7 batched to Cayleb → on his word, **S1 (the single shared Studio repair)** →
  S2/S3 for the street game and T1/T2 for the card game in parallel (both are offline) → then the Studio-bound cells.

---

## Appendix — evidence index (commands run, 2026-10-31 17:43–18:05 ET, on `outis-desktop`)

| # | Command | Result (quoted) |
|---|---|---|
| 1 | `flatpak list \| grep -i vinegar` | `Vinegar  org.vinegarhq.Vinegar  1.9.4  stable  user` |
| 2 | `ls …/vinegar/versions/` | `version-7062aec9dbe74b14` (Oct 27 18:30) — only version folder |
| 3 | `ls -la …/version-7062aec9dbe74b14/*.exe` | `RobloxStudioBeta.exe 222,384,592`; `StudioMCP.exe 8,224,720` |
| 4 | `flatpak ps --columns=instance,application` | empty → Studio not running |
| 5 | `~/.local/bin/roblox-studio-mcp` | `Error: Roblox Studio (org.vinegarhq.Vinegar) is not running in Flatpak.` |
| 6 | `cat …/AppData/Local/Roblox/mcp.bat` | pins `…\version-7062aec9dbe74b14\StudioMCP.exe` + registry fallback (folder exists) |
| 7 | `grep -n -i roblox ~/.codex/config.toml` | `[mcp_servers.roblox_studio]` … `enabled = false   # DISABLED 2026-10-28 … undo = delete this line` |
| 8 | `ls -t …/Roblox/logs/ \| head -1` | `0.745.0.7451137_20261029T083958Z_Studio_4AAE8_last.log` 43,104,082 B, last line `2026-10-31T00:09:17Z`, `Authenticated : NO` ×1 / `YES` ×1 |
| 9 | `ls -la ~/.ssh/` | `No such file or directory` |
| 10 | `getent hosts muse` | (empty, rc=2) |
| 11 | `tailscale status` | `muse 100.110.154.92 active`; `outis … windows offline, last seen 151d ago`; `cayleb-desk … offline, 253d ago` |
| 12 | `git -C …/untitled-street-game log --oneline -1` | `8dfb204 Docs: correct the stale numbers and the broken reference the audit found` |
| 13 | `sha256sum …/street-game/build/*.rbxlx` | `84324d43…` release.rbxlx 1,990,548 B · `c6651793…` test.rbxlx 2,403,691 B |
| 14 | `sha256sum …/card-game/build/UntitledAnimeCardGame.rbxlx` | `55cef5d3…`, 520,846 B **≠** recorded `be972098…`, 520,854 B |
| 15 | `cat …/card-game/src/shared/content/ArtIds.luau` | 24 rows, every value `""` |
| 16 | `python3 -c "json.load(assets/audio/manifest.json)"` | 8 assets, every `roblox_id: null` |
| 17 | `file assets/images/DB-GOKU-01.png` | `PNG image data, 740 x 1024, 8-bit/color RGB` |
| 18 | `ls assets/images/*.png \| wc -l` | `24` |
| 19 | `grep -n -i "GPT-6.1" ~/.codex/AGENTS.md ~/AGENTS.md` | `.codex/AGENTS.md:29` Sol lock **vs** `~/AGENTS.md:1-3` "replaces the Sol lock" |
| 20 | `cat …/street-game/docs/verify/2026-10-19/vinegar/SPRINTS.md` | 51 Plays in 19 Studio openings; "The owner opens Studio; the agent never does." |

*Not performed, deliberately: no Studio launch, no config edit, no upload, no credential access, no spend, no
publication. Nothing here was run twice to make a failure disappear.*

---
---

# BATCH 2 — D5–D8 (offline only)

Window: **2026-10-31 18:09–18:25 ET**. Constraints given by the Lead: offline only, **no Studio launch, no uploads,
no config edits, no spend**. All four items were executed inside those constraints. Rulings applied: creative-model Q1
CLOSED (existing 24 + 8 may be bound/verified by this team; no new art here); Q4 rebuild-from-pinned-commit;
Q5 stricter legacy reading; Q3 (config) **not touched — held for Nox**.

**Files produced in this batch**
- `board/probes/team-d/uacg-candidate-receipt.json` — the source pin for the card-game candidate (D5)
- `board/probes/team-d/portrait-integrity.md` + `.json` — the 24-row integrity table (D7)
- `board/probes/team-d/shared-repair-procedure.md` — the once-only repair procedure + Nox mail (D6)
- this Batch 2 section (D8 templates inside)

---

## D5 — Card-game candidate identity: rebuild from the pinned commit

**Question asked:** rebuild deterministically from the commit `CURRENT-STATE.md` pins, sha256 it, record the receipt,
and report whether it matches `be972098…` or not.

**Short answer: no — and the interesting part is that the candidate is reproducible from the *working tree*, not from
any commit.** Two experiments, both run twice, in scratch copies (the live `build/` was never touched; the live
artifact was copied to `board/probes/team-d/artifacts/` first).

### The pinned commit is not the candidate's source

```
$ git -C ~/Work/projects/untitled-anime-card-game log -1 --format="%H %ad %s" --date=iso
04031c0f9da5d0b41d9e5395d30023098e9086be 2026-09-26 18:11:25 -0400 Refine native card identity and result presentation
$ git status --short | awk '{print $1}' | sort | uniq -c
    114 ??          # untracked
      4 D           # deleted, tracked
     57 M           # modified, tracked
$ git status --short | grep -E "^ M src/" | wc -l      ->  21
$ git status --short | grep -E "^\?\? src/"            ->  src/server/AwayProgressService.luau, src/shared/offers/, src/shared/progression/AwayProgress.luau
```
The repository's **last commit is 2026-09-26**, but the candidate was installed **2026-09-29** and its code lives in
**uncommitted working-tree state**: 21 modified files under `src/`, 3 untracked source entries (the away-progression
: taxes and the offers layer), 4 deletions. The P1–P5 "completion cycle" work described in `CURRENT-STATE.md` was
never committed. **So "the pinned commit" cannot reproduce the candidate, and no commit can — the source of record is
a dirty tree.**

### Experiment A — clean worktree at `04031c0` (deterministic, but an older artifact)

```
$ git worktree add --detach ~/.cache/team-d/uacg-rebuild 04031c0      # clean, no working-tree drift
$ cp .tools/bin/{rojo,lune} ~/.cache/team-d/uacg-rebuild/.tools/bin/   # bin is gitignored, so copied
$ bash tools/build.sh   ->  stamped … with manifest b13d3519f9b2
  sha256  1259b13621e9ab21d12c006317571c263e47496223ec2a26887e6e72f547fc09   497,070 B
$ bash tools/build.sh   ->  same stamp, same hash   (twice identical)
A1_SHA = A2_SHA = 1259b13621e9ab21d12c006317571c263e47496223ec2a26887e6e72f547fc09
```
The build is **deterministic**, and the hash it produces is a *historically recorded* one:
`docs/VISUAL-DIRECTION-20260926.md` cites "The preceding build in `CURRENT-STATE.md` recorded place SHA-256
`1259b13621e9ab21d12c006317571c263e47496223ec2a26887e6e72f547fc09`". So commit `04031c0` = the old presentation
tree, and A is a valid reproduction of that old place — **not** the candidate.

### Experiment B — copy of the live working tree (reproduces the candidate exactly)

```
$ rsync -a --exclude=.git --exclude=session-artifacts … ~/.cache/team-d/uacg-live/
$ bash tools/build.sh   ->  stamped … with manifest c976b6242c0d
  sha256  55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9   520,846 B
  build receipt place_sha256 = 55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9
$ bash tools/build.sh   ->  identical again
B1_SHA = B2_SHA = 55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9
```

**Verdict (all four possibilities tested):**

| Artifact | sha256 | bytes | reproducible? |
|---|---|---|---|
| `be972098…` — 2026-09-27 candidate the doc's top line still calls current | `be972098432f5bac4ccbc67712b104f3fb26073ab303d173f10b567baa70e7aa` | 520,854 | **no** — superseded by the repo's own dated notes; its P5 receipt stays valid for its own bytes only |
| `9d9a80b0…` — 2026-09-28 live build (verification FAIL) | `9d9a80b09ef16b2621e8fd5fada5e1ad333bacd17efa029ca09f29f81d1e5049` | 612,585 | no — backed up and replaced on 2026-09-29 |
| **`55cef5d3…` — installed 2026-09-29, operator-approved, twice-verified** | `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` | 520,846 | **yes — from the working tree, twice byte-identical, and it matches `ops/release/build-receipt.json`** |
| `1259b136…` — what commit `04031c0` builds | `1259b13621e9ab21d12c006317571c263e47496223ec2a26887e6e72f547fc09` | 497,070 | yes, from the commit — an older place (cited in `VISUAL-DIRECTION`) |

The record also resolves the candidate question: `CURRENT-STATE.md` (2026-09-29, "Portfolio run … verified build
installed (operator-approved)") says "**Local verification PASS for build sha256 `55cef5d3…` (520,846 bytes)** …
the operator approved replacing the live `build/UntitledAnimeCardGame.rbxlx` with the twice-verified rebuild", with
`docs/verification/portfolio-20260928/verify-3/` and `verify-4/` as "two full canonical `tools/verify.sh` runs
against the installed file: all 23 steps exit 0 in each". So the on-disk file **is** the reviewed candidate; it is
the **doc's top line** that is stale (it still says `be972098…`).

### What I did to make it reproducible without touching git

Two facts force a receipt rather than a commit: (a) the source is a dirty tree, and (b) both `build/` and `.tools/`
are **gitignored** (`.gitignore`: `build/`, `.tools/`, `*.rbxlx`, `*.rbxl`), so neither the candidate bytes nor the
toolchain exist in git at all. I therefore wrote a **source-snapshot receipt** — no git write, no repo change:

> `board/probes/team-d/uacg-candidate-receipt.json`
> head `04031c0` (dirty: 61 tracked changes / 114 untracked) · artifact `55cef5d3…` 520,846 B, **matches
> `build-receipt.json`** · rebuild twice → `55cef5d3…`, stamped manifest `c976b6242c0d` · commit rebuild →
> `1259b136…` 497,070 B (older place) · **157 source files hashed, aggregate
> `727993e39bce2fd10ef7f5020cf023df091713f6feaad0a2feff09c0e198dc2f`** · toolchain sha256 for `rojo` and `lune` ·
> per-file hashes for `src/ tools/ ops/ tests/ default.project.json`.

**No Studio, no network, no spend was needed:** the build uses the vendored `rojo 7.7.0` / `lune 0.10.5` binaries at
`.tools/bin/`, and `stamp_manifest_hash.py` + `publish.refresh_build_receipt` are local Python. Production was done in
`~/.cache/team-d/` copies; the live `build/` was untouched (the live artifact sits, byte-identical, at
`board/probes/team-d/artifacts/UntitledAnimeCardGame.live-20261031T181409.rbxlx`).

**Follow-on for the Lead (not done, needs a ruling):** to make the candidate recoverable from git itself, someone with
authority must **commit the working tree** (or bless the receipt as the pin). I did not commit — that is a repo write
Cayleb/Lead should name. Until then the receipt is the pin, and it should travel with the build.

---

## D6 — Shared-repair procedure: written and HELD

`board/probes/team-d/shared-repair-procedure.md`. It contains, with the quoted probes: the exact edit (file
`/home/cayleb/.codex/config.toml`, line 192 `enabled = false …` **deleted**, context lines 190–191 left intact); the
**backup trap** (the named pre-disable backup differs from today's config by **two** lines — the `enabled = false`
line and `model_reasoning_effort = "ultra"` → `"xhigh"` — so restoring it wholesale would revert the reasoning-effort
setting; delete only line 192); the restore point to take at execution time; the **port 13469 sequence** (9 steps with
owner/gate per step, including the two-step stale-`StudioMCP.exe` reap that requires Cayleb's yes to named PIDs); the
**street-first** ruling with four reasons (its 51-Play plan is written, it has a live private place so persistence and
receipts can run against a saved version, the card game has no authorised target at all, and its first Studio work
needs the Q6 upload authority); the Studio-open handoff (Cayleb's four mechanical steps, verbatim from the runbook);
the repair verification (six evidence items incl. a negative control in a separately scheduled window); the fallback
`mcp.bat` edit; the "must not happen" list; and a **board-mail-ready note to Nox** asking for the port-sequencing
ruling (§9 of that file — the Lead sends it).

State at writing (so the repair is cheaper than feared): **no Studio running, port 13469 not listening, no stale
`StudioMCP.exe`, `flatpak ps` empty** — the machine is clean, so the repair is one line plus an owner-opened Studio,
not a process cleanup. The 3-Plays counter (`~/.cache/usg/vinegar-plays.json`, Oct 26 22:12) is stale and harmless.

Nothing was edited. `~/.codex/config.toml` sha256 is unchanged from the value the Lead verified.

---

## D7 — Portrait integrity: 24/24 PASS at file level

Full table: `board/probes/team-d/portrait-integrity.md` (markdown, 24 rows) + `.json` (machine-readable).
The four 24-row sets reconcile exactly: **ArtIds.luau rows (24) = `assets/images/manifest.json` rows (24) =
`Cards.luau` card rows (24) = `docs/ART-PROVENANCE-CURRENT.md` rows (24) = PNGs on disk (24)**.

| Check | Result |
|---|---|
| dimensions / format | **24/24** `740 x 1024`, PNG colour type **2** (RGB, no alpha), 8-bit — matches manifest `width`/`height` |
| sha256 vs manifest | **24/24 match** |
| bytes vs manifest | **24/24 match** |
| sha256 vs the provenance doc's own final-PNG column | **24/24 match** |
| duplicates | **0** — 24 unique sha256 values, no two files share bytes |
| extra / missing vs the 24 `ArtIds` rows | **0 extra, 0 missing** (no stray top-level PNGs; the extra dirs `incoming/ replacements/ _superseded/ sheets/` are separate and are not in the ArtId set) |
| embedded text metadata (file-level text/watermark signal) | **0** — no `tEXt`, `iTXt` or `zTXt` chunk in any of the 24; no `eXIf` chunk either |
| binding state in the data layer | **24/24** `artwork = "PLACEHOLDER-no-asset-id"` in `Cards.luau`; **24/24** `ArtIds` values `""` — so the candidate genuinely cannot show real art yet |
| roster mapping | franchise-level mapping is present in the data (`Cards.luau` carries `franchise`, `canonical`, `alias` per id, e.g. `BC-ASTA-01 / Black Clover / "Magicless Knight — Anti-Magic Blades" / "Anti-Magic Knight"`); 3 per franchise x 8 franchises = the roster's 24 |

**What this does and does not prove.** It proves the 24 files on disk are the exact bytes the project's own records
name, that nothing is duplicated, renamed, missing or substituted, and that no text metadata rides along. It does
**not** prove the picture shows the right character, or that there is no *painted* watermark — that is a visual
judgement and it stays a Studio/human cell (`board` matrix D4 step 5: 24 screenshots matched to data-named ids).
Note the roster's character names (Ichigo, Asta, Guts, …) are **not** repeated in the data layer: `Cards.luau` gives
franchise + canonical + alias labels only, so the character-level mapping is by id abbreviation and must be confirmed
by eye, one card at a time. That is a residual to carry, not a defect.

---

## D8 — The two playtest-note templates (unknowns marked, no claims)

Templates only: every bracket is an unknown to be filled **from measured results at packaging time**. No test is
claimed to have passed; both builds remain unverified natively.

### Job 16 — street game (for Cayleb)

> **What you will see.** You join as a night-shift driver in Kuroyo City, 2077: a neon street hub with two prompts on
> the ground — **OPEN GARAGE** and **OPEN DISPATCH** — and your first fare offered within the first few minutes, so
> you are driving quickly. Behind that: a garage of original 80s/90s tuner cars you can buy and kit out with credits,
> street sprints, live traffic, and rivals that lean, brake and chase you. **Purchases are switched off in this build**
> — nothing is for sale and nothing can be bought. ⟨BUILD ID: version N / sha256 …⟩ ⟨PLATFORM: desktop / phone /
> tablet⟩ ⟨KNOWN-NAP: the date this build was cut⟩
>
> **What to poke at.** (1) The first five minutes without help: can you take a fare and finish it without guessing
> what to do? (2) The garage: open it, buy or fit something cheap, back out, and see the change hold. (3) One sprint
> against the rival field. (4) A hard left and a handbrake turn at speed — does the car do what you expect? (5) Leave
> the game mid-fare and come back: is your progress still there? ⟨HOW TO LEAVE/REJOIN: the exact steps in this
> build⟩
>
> **What is known-rough.** ⟨AUDIO: the 27 runtime files have not been listened to by a human — judge for yourself;
> ⟨MEASURED? / not measured⟩⟩ ⟨LOOK: the open items are the road flecks and cyan sheen at night, dark mid-distance,
> blank overheads, repeated frontages, and the void at the map edges — recorded, not hidden⟩ ⟨PERFORMANCE: phone and
> tablet figures are estimates until someone runs it on the real device; ⟨MEASURED? / not measured⟩⟩ ⟨TWO CLIENTS:
> ⟨VERIFIED / SIMULATED ONLY — say which⟩⟩ ⟨KNOWN BUGS: ⟨list from the run, or "none found">⟩ ⟨PROMPTS: purchase
> prompts are off in this build; that is deliberate, not a bug⟩

### Job 17 — card game (for Cayleb)

> **What you will see.** An untitled collectible card battler — deliberately untitled, a fresh identity with no
> earlier project's name on it. You open on a bright open-air Skybridge scene, draw from a deck of flat monochrome
> cards, and fight through a run: hover or select a card to play it, watch the strike land, then claim the result on
> the result screen before you can continue. There is a store to recover spent things and a rebirth flow that resets
> a run while keeping what the game says it keeps. ⟨BUILD ID: sha256 55cef5d3… / ⟨or the newly bound build⟩⟩
> ⟨CAST: 8 series x 3 cards = 24 — ⟨ART BOUND? placeholder "ART PENDING" until Q6⟩⟩
>
> **What to poke at.** (1) Draw and play a card, and check the **sound** fires exactly when the card lands. (2) A full
> battle to the result screen, then **Claim** — the result must not let you continue until the claim lands. (3) The
> store: does it recover what it says it recovers? (4) The rebirth preview: read the list of what you keep and what
> you lose, then cancel — nothing should have changed. (5) Each of the 24 cards you can reach: does the picture match
> the card's name? ⟨WHERE TO PLAY: local Studio / ⟨device⟩⟩
>
> **What is known-rough.** ⟨ART: 24 portraits are produced but **not yet uploaded/bound** — cards show an explicit
> "ART PENDING" state, not a wrong picture; when bound, ⟨24/24 verified in Studio?⟩⟩ ⟨AUDIO: the 8 cues are
> **stand-ins** (engine-bundled) until the finals are uploaded; the finals themselves have never been listened to by a
> human — "listening NOT PERFORMED" stays on the record⟩ ⟨LEGACY PROFILES: this game never reads or writes legacy
> profile data; the proof is ⟨test record / pending⟩⟩ ⟨STUDIO PLAY: the tested Play run was on an older build; this
> build's Studio Play is ⟨RUN / NOT RUN⟩⟩ ⟨KNOWN BUGS: ⟨list from the run, or "none found">⟩

Both templates deliberately say **"not measured"** rather than giving an impression, and they cannot be filled
honestly until the Studio cells and the window exist.

---

## Batch 2 — states, holds, next

- **Every constraint held:** no Studio launch, no upload, no config edit, no git commit, no spend, no credential
  access. The only writes were inside `board/probes/team-d/`, `board/reports/team-d-phase1.md`, and scratch copies
  under `~/.cache/team-d/`.
- **D5 finding (headline):** the card-game candidate is reproducible **exactly** (`55cef5d3…`, twice) from the
  working tree and **not** from the pinned commit; the tree carries 21 modified + 3 untracked source files; the
  doc's `be972098…` top line is stale. Pinned by receipt: 157 files, aggregate `727993e3…`.
- **D7 finding (headline):** 24/24 portraits reconcile across all five records (ArtIds, manifest, Cards.luau,
  provenance doc, disk) with no duplicates, no text metadata, and 24/24 still unbound placeholders.
- **D6:** procedure + Nox mail ready; waiting on the Lead to send it and on the port ruling.
- **Next + successor:** (1) Lead sends the Nox note; (2) ruling on committing the card-game working tree (or
  blessing the receipt as the pin); (3) then S1 (the single shared Studio repair) when Cayleb's window opens —
  street first. Nothing in Batch 2 changes any job's status: **16 and 17 remain BLOCKED-ON-MACHINE / PREP COMPLETE.**

---
---

# BATCH 3 — rulings applied + the three Studio-window prep documents

Window: **2026-10-31 18:22–18:38 ET.** The Lead's ruling 1 (preservation branch) executed; ruling 2 noted and queued;
rulings 3 and 4 acknowledged; items (a)(b)(c) prepared. Still **no Studio launch, no config edit, no upload, no spend**.

## R1 — Preservation branch: DONE and proven

**Branch:** `preserve/candidate-55cef5d3` in `~/Work/projects/untitled-anime-card-game` (base `04031c0`, branch `master`
left untouched at `04031c0`). **No remote exists** (`git remote -v` → empty), so nothing was pushed; no rebase, no
history rewrite, no tag.

| Commit | What | Files |
|---|---|---|
| **`add37b63211f9e81b83c5164c7f99e360aaf957b`** — *the pin* | "preserve: candidate 55cef5d3 source state (working-tree commit)" — the 61 tracked changes (57 M / 4 D) + every untracked item, message naming the receipt, the artifact hash, the source aggregate, the base-commit finding and the superseded hashes | 830 |
| `43574a56c9508cc17f2003b7fca29bd87ba1ca31` | "preserve: stop tracking Python bytecode caches (tracking hygiene only)" — removes 13 `.pyc` from the index (they stay on disk) and adds `__pycache__/` + `*.pyc` to `.gitignore`; **no source change** | 14 |

**Why a second commit instead of amending:** the first commit carried 13 untracked `.pyc` files because no ignore rule
covered `__pycache__/`; left alone, every tool run would regenerate them and the pin would look dirty after any build.
The honest fixes were amend (a history rewrite — expressly forbidden) or a labelled follow-up. I chose the follow-up;
the **pin remains `add37b6`**, and the receipt names it.

**Evidence:**

```
$ git rev-parse --abbrev-ref HEAD        ->  preserve/candidate-55cef5d3
$ git rev-parse HEAD                     ->  43574a56c9508cc17f2003b7fca29bd87ba1ca31
$ git status --short | wc -l             ->  0                       # clean
$ git ls-files | wc -l                   ->  7814                    # tracked files now
$ git remote -v                          ->  (empty)                 # no remote; nothing pushed
$ git worktree add --detach … 43574a5 && bash tools/build.sh  (x2)
    =>  55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9  (520,846 B)  both times
    =>  worktree clean after both builds (0 lines)
$ sha256sum ~/Work/projects/untitled-anime-card-game/build/UntitledAnimeCardGame.rbxlx
    =>  55cef5d3… (live artifact untouched)
```

`.gitignore` now covers `build/`, `.tools/`, `*.rbxlx`, `*.rbxl`, `__pycache__/`, `*.pyc` — so the ignored set is
exactly the Lead's exclusion, plus regenerable bytecode. Secret scan **before** staging: 954 candidate files scanned
(filename + content: private-key blocks, AWS/GitHub/Slack/OpenAI-style tokens, `ROBLOX_SECURITY`, `Bearer`, key-ish
assignments), **0 hits**, 5 binaries skipped; nothing was excluded and nothing needed to stop.

**The source pin now works end-to-end:** `git checkout preserve/candidate-55cef5d3` + the vendored `rojo`/`lune`
(copied, since `.tools/` is ignored) rebuilds the operator-approved candidate byte-for-byte, twice.

## R2 — `CURRENT-STATE.md` stale top line: noted, deliberately NOT changed yet

The correction is queued as the **first docs edit on the next repo touch** (the upload/binding round is the natural
moment). It was not made now on purpose: the preservation branch must stay a byte-faithful image of the state that
produced `55cef5d3…`, and a docs edit inside it would be a second, unrecorded change to the pin. The ruling's
"only when the Team is next touching docs, and note the change" is being taken literally: the note will land with that
edit, as a dated line over the top, mirroring the style already used in that file ("This dated note supersedes …").
`be972098…` is now recorded in the branch message and in the Batch-2 section above as the superseded 2026-09-27
candidate; `55cef5d3…` is the candidate of record.

## R3/R4 — acknowledged; no action needed

The D6 procedure already carries the backup trap (two-line diff, delete only line 192, never restore wholesale) and
`config.toml` is untouched (mtime `Oct 31 16:09`, the single `enabled = false` line present — re-checked at 18:36).
D7 and D8 are accepted as recorded.

## (a) S1 Studio-window card — `board/probes/team-d/studio-window-card.md`

One screen, four blocks: **before he opens** (backup + delete line 192 only + clean port + `hostcheck` quiet + `status`
exit 0), **handoff to Cayleb** (the three sentences he actually needs: open Studio and sign in as `Chrono_H3X` *if
asked*, open the named place, leave it in Edit and say go), **the window itself** (handshake → read version/account
from Studio's own UI → **street first**, max 3 Plays per process → `verify-clean` → card game second, only if real
asset ids exist → `verify-clean` → "Don't Save" + `hyprctl reload`), and **after he closes** (the line stays deleted —
the repair is for both games — plus the repair record and the restore path if it failed). It includes the stop-rule
table (exit 1 = real finding, keep it; exit 2 = refused, nothing changed; 4 = host-contaminated; 5 = crash → re-copy
the place) and the one-screen truth table.

## (b) Upload plan — `board/probes/team-d/upload-plan.md` (plan only)

The important discovery while writing it: **the project's binding gate is stricter than "upload and copy an id".**
`ops/release/publish.py` → `validate_mapping_receipt` refuses a card unless its receipt carries **all** of: exact
`card_id`; `source_sha256` equal to the current approved bytes; a 6–19 digit `asset_id`; a `target` **exactly** equal
to `ops/release/target.json`'s creator/universe/place; the card's `franchise` from `Cards.luau`; an
`upload_receipt` (`uacg-upload-receipt/v1`, `COMPLETE`, `returned_on`); and a `moderation` receipt
(`uacg-moderation-receipt/v1`, verdict **PASS**, `moderated_on`, fresh). It also refuses duplicate uploads and refuses
an automatic re-upload when a prior attempt is unresolved. `tools/apply_art_ids.py --receipt` is the only binding path
("bare copied IDs are refused").

Two hard blockers found, both already in the Lead's ask but now precise:
1. **`ops/release/target.json` does not exist** — only `target.example.json`. Every upload and every binding is
   refused without it, so filling it is step **U0**, and it needs the experience's own creator/universe/place values.
2. **Audio has no upload route in the project's tooling at all** — `publish.py` has no audio/sound upload path
   (`grep -i "upload.*audio\|sound"` → nothing). The 8 finals therefore need one of: Studio Asset Manager manual
   import in a window (recommended first pass), a small new script (code work, same key/target), or keeping the
   engine-bundled stand-ins and deferring finals. That is a scope fact the Lead should know before promising audio
   finals in a window.

The plan itself: **U0** target → **U1** 8 audio finals → **U2–U4** portraits in franchise blocks of 6–9 (small blast
radius if moderation balks) → **U5** rebuild + bind + verify. Neutral asset names (`uacg-art-<card_id>`,
`uacg-cue-<purpose>`) with **no franchise/character words in the asset list**; most restrictive privacy allowed; no
publication; rights stay USER-ATTESTED; a per-asset record table (card id, source sha, asset id, upload receipt,
moderation PASS, exact target, franchise, asset name, operator); failure table (duplicate → verify not force;
unresolved attempt → resolve by hand; moderation not PASS → keep the row, block the card, never re-upload the same
bytes hoping); and the credential rule (if the tool asks for a key it does not already hold, that is Cayleb's secure
paste — stop).

## (c) Studio evidence capture sheet — `board/probes/team-d/studio-evidence-sheet.md`

Session header (date, Studio version **from Studio's UI**, account, place, build sha at open **and** close, ArtIds
state, capture root, handshake file) + a **24-row per-card table** (data-named asset id read from the *running game*,
"portrait visibly correct? Y/N" with a one-sentence description of what is actually seen, screenshot file, screenshot
sha256, note) + an **8-row per-cue table** (exact in-game trigger, fired-on-trigger, audible/correct with one sentence,
log line, capture file) + supporting captures (the game's own build self-report, ArtIds readback, place hash before/after,
the Studio log, the creator dashboard rows for the 32 assets, failure frames) + a **negative control** (a deliberately
wrong binding must fail the visual check, proving the check can fail) + the close-out checklist and an explicit "what
this does not prove" line (rights stay USER-ATTESTED; fun is not proven; moderation may re-review).

## Batch 3 — state

- **R1 DONE** (`preserve/candidate-55cef5d3`, pin `add37b6`, tip `43574a5`, clean, proven to rebuild `55cef5d3…`).
- **R2 queued** as the next docs edit, with its note.
- **(a)(b)(c) written**, plan-only, nothing uploaded or launched.
- **Unchanged:** Jobs 16/17 remain **BLOCKED-ON-MACHINE / PREP COMPLETE**; `$0`; publication not authorised; config
  untouched; publication/upload/purchase all still held for Cayleb's word and Nox's ruling.
- **Next + successor:** on Nox's port word + Cayleb's Studio-route answer → run the S1 card in the first window
  (street first). On Cayleb's upload authorisation → fill `target.json` (U0), upload 8 audio then 24 portraits in
  franchise blocks, then the evidence sheet round.

---
---

# BATCH 4 — S1a EXECUTED: the shared Studio-route repair is done on the config side

Window: **2026-10-31 18:38–18:42 ET.** Authorised by the Lead's go, **Nox's written green light** (received ~18:23 ET:
the `enabled = false` line was a one-session measure from 28 Oct, never a standing prohibition; no fleet gate/fixture
depends on the disabled state — if the repair surfaces one, record it and do **not** re-disable) and **Cayleb's answer
"Use the desktop Studio now."** The gate is fully closed; the Studio side is now the only thing left.

## The edit — one line, nothing else

```
$ cp -p ~/.codex/config.toml ~/.codex/config.toml.pre-studio-repair-20261031T183910   # FRESH backup
$ sha256sum <backup>
e10f08e43f6ed5601d8cee48d9ab8c549bbd158ab9d2dcf6f5e9782995cff56c        # = BEFORE state of the live file
# (the stale config.toml.pre-disable was deliberately NOT used: it differs by
#  model_reasoning_effort "ultra" vs today's "xhigh" — restoring it would revert that setting)
$ diff -u <backup> ~/.codex/config.toml
 [mcp_servers.roblox_studio]
 command = "/home/cayleb/.local/bin/roblox-studio-mcp"
-enabled = false   # DISABLED 2026-10-28 … undo = delete this line (backup: …pre-disable)
$ diff changed-line count -> 1        # exactly one line; bytes 10,843 -> 10,583 (260 = line + newline)
```

| | sha256 | bytes |
|---|---|---|
| **before** | `e10f08e43f6ed5601d8cee48d9ab8c549bbd158ab9d2dcf6f5e9782995cff56c` | 10,843 |
| **after** | `752a7a2f0e80183f6679efa332930af7fca2d492bd9a6529f35ebcd40eff3ac8` | 10,583 |
| backup | `~/.codex/config.toml.pre-studio-repair-20261031T183910` | sha = before |

## Proof the file is intact and the server is enabled

```
$ python3 -c "import tomllib; tomllib.load(open('~/.codex/config.toml','rb'))"
PARSE_OK: top-level keys = 17
parsed [mcp_servers.roblox_studio] = {'command': '/home/cayleb/.local/bin/roblox-studio-mcp'}   # 'enabled' key GONE
all 7 servers intact: blender, chrome-devtools, free_3d_assets, linux_background, linux_desktop, node_repl, roblox_studio
$ grep -c "enabled = false   # DISABLED 2026-10-28"   -> 0
$ flatpak ps | wc -l -> 0     $ ss -ltnp | grep -c 13469 -> 0     real StudioMCP processes -> 0
$ ls -la ~/.codex/config.toml -> -rw------- 1 cayleb cayleb 10583 Oct 31 18:39   (mode preserved)
```

No Studio was launched, no credential read, no second repair improvised, no other file touched.

## What remains — the Studio side (Cayleb's acts at run time)

| Step | Who | State |
|---|---|---|
| Open **one** Studio (Vinegar), leave in Edit; sign in as `Chrono_H3X` **if asked** | Cayleb | waiting |
| MCP handshake (`initialize` + `tools/list`), save the raw bytes | agent | waiting — needs Studio |
| Read version + account from Studio's own UI (`0.745.0.7451137`) | agent | waiting |
| **Street game first** — ≤3 Plays per process, then `verify-clean` | agent | waiting |
| Card game second — only once real asset ids exist | agent | blocked on the uploads |
| Close: **Don't Save** + `hyprctl reload` | Cayleb | — |

If the handshake fails, restore the timestamped backup above and report the failure; do not improvise a second fix.
Nox's condition stands: if a fleet gate/fixture that depended on the disabled state surfaces, record it and do not
re-disable. Nothing has surfaced yet — that check needs the live handshake, i.e. the window.

## Batch 4 — state

- **S1a DONE** (config side). `board/probes/team-d/shared-repair-procedure.md` §10 and
  `board/probes/team-d/studio-window-card.md` were updated with the executed state and what remains (S1b).
- **Not done, deliberately:** Studio opened/signed in (Cayleb's act), uploads (authorisation + Open Cloud key still
  pending), `CURRENT-STATE.md` top-line correction (queued for the next normal docs touch, per R2).
- **Unchanged:** `$0`; publication not authorised; no upload; jobs 16/17 remain **PREP COMPLETE / BLOCKED-ON-MACHINE
  until the window**, with the route now repaired on the config side.
- **Next:** D9–D11 (target.json dry-run note, audio-import script design, one-page run-sheet) — in progress; D9's core
  finding so far is that **the card game's own records contain no universe/place ids** (both `ReleaseConfig.luau`
  files hold `universeId = 0, placeId = 0` with the comment "Zero IDs keep the published-server path closed until the
  owner selects the experience"), while `publish.py` **does** have a credential-free dry-run mode (`upload-art` /
  `publish-place` without `--live` print the exact POST and return 0), and `.gitignore` does **not** cover
  `ops/release/target.json` even though the project's own example says "never commit the filled file" — a latent leak
  risk to fix on the next repo touch.

---
---

# BATCH 5 — D9 dry-run evidence, D10 audio route, D11 one-page run-sheet

Window: **2026-10-31 18:52–19:02 ET.** Still offline: no Studio, no upload, no credential read, no repo edit (the
`.gitignore`/CURRENT-STATE edits stay queued for the next normal repo touch, per the Lead's rulings 1–2). $0.

## D9 — `target.json` dry-run evidence, quote-ready

Two credential-free runs, captured verbatim in `board/probes/team-d/u9-upload-dryrun.txt`:

```
$ python3 ops/release/publish.py upload-art            # no --live
DRY RUN: 24 candidate image upload(s); operator authorization is recorded, public rights/license are not
independently verified, and no network requests are made.
POST https://apis.roblox.com/assets/v1/assets (x-api-key: <redacted>) card=BC-SEA-01
… (24 lines, one per current image candidate) …
exit=0   captured in 25 lines

$ python3 ops/release/publish.py publish-place          # no --live, no target.json
BLOCKED: valid ops/release/target.json is required to construct the publish URL
exit=2
```

So the project **has** a real dry-run mode (no `--live` → zero network, key printed as `<redacted>`), and the U0 gate is
proven rather than asserted: `publish-place` refuses at the target, and `upload-art --live` would go through
`live_context()` → `target.json`. The fabrication risk is closed on our side: **we will not write a `target.json`
without the ids**, because the game's own records say the owner must select the experience
(`src/server/ReleaseConfig.luau`: `universeId = 0, placeId = 0`, "Zero IDs keep the published-server path closed
until the owner selects the experience and its pass is created there"). That selection is D12 in Cayleb's ask.

Queued for the next repo touch (not done now): add `ops/release/target.json` to `.gitignore` (the project's own
`target.example.json` says "never commit the filled file", but the ignore rule is missing) **and** the dated
CURRENT-STATE top-line correction — same edit, both recorded.

## D10 — audio import: **a real Open Cloud route exists** (design only)

`board/probes/team-d/audio-import-design.md`. Verdict first: **`assetType: "Audio"` is a supported Assets API type**, on
the same endpoint the project already polls for Decals — not an Asset Manager-only fallback. Evidence (read-only
official docs, fetched 2026-10-31, external content treated as data, source
<https://create.roblox.com/docs/cloud/open-cloud/usage-assets>):

> Asset type **Audio** · Format **.mp3 .ogg .wav .flac** · Content type **audio/mpeg audio/ogg audio/wav audio/flac** ·
> Restrictions: **"Up to 7 minutes of duration." / "Up to 100 uploads per month if you're ID-verified." / "Up to 10
> total uploads per month if you aren't ID-verified." / "Not available for updating."**

and the request/response shape: `POST /assets/v1/assets` (multipart `request` + `fileContent`, `x-api-key`) →
`operationId` → `GET /assets/v1/operations/{id}` → `assetId` + `moderationResult.moderationState`
(`MODERATION_STATE_APPROVED`). The project's `upload_image` already does exactly this for `Decal`
(`publish.py:700,721,732,740`), so the script is a sibling.

The design (`tools/upload_audio.py`, **not written into the repo**, spec only): local pre-validation first
(extension/content-type map, real duration ≤420 s and matching the manifest within ±0.1 s, ≤20 MB, sha256, not already
bound, `target.json` present) → single multipart POST with neutral name `uacg-cue-<purpose>` → attempt registry with
`SUBMISSION_STARTED`/`AMBIGUOUS` and **no automatic retry** → operation polling → moderation handling
(APPROVED binds; PENDING stays open; REJECTED keeps the row, blocks the cue, never re-uploads the same bytes) → write
`roblox_id = "rbxassetid://<id>"` into `src/client/AudioCatalog.luau` for that purpose (the stand-ins are the
`rbxasset://sounds/…` lines there) → rebuild → Studio trigger check. Receipt schema `uacg-audio-upload/v1` mirrors the
image receipt so a future binder can refuse bare ids. Dry-run prints the 8 POSTs with the key `<redacted>`.

**Two constraints the Lead/Cayleb should hold in mind:** audio is **non-updatable** (a bad upload can only be replaced
by a second upload) and the monthly cap may be **10** if the account is not ID-verified — 8 cues is one shot. Both are
why the design validates locally and refuses silent retries. Asset Manager import remains the manual fallback.

## D11 — one-page run-sheet: `board/probes/team-d/studio-run-sheet.md`

Seven short blocks, no duplication (the other three docs are named as appendices): **preflight** (config line count 0,
toml parses, 0 Studio/0 listeners/0 StudioMCP, `hostcheck` quiet, `status` exit 0, build hashes on disk), **handoff**
(the three sentences, street place first), **window order** (handshake → version/account from Studio's UI → street
cells ≤3 Plays → reopen → `verify-clean` → card game only when ids exist → `verify-clean` → Don't Save + `hyprctl
reload`), **stop rules** (exit 0/1/2/4/5 table + the fail-twice stop), **evidence to leave with** (handshake file,
per-Play exits, verify-clean, capture sheet, session header), **close-down and rollback** (restore the timestamped
backup on failure; Nox's do-not-re-disable condition), and **what the window does not do** (no uploads, no publication,
no card binding yet).

## Batch 5 — state

- **D9 evidence ready to quote**; no `target.json` fabricated; `.gitignore` + CURRENT-STATE corrections queued.
- **D10 answered: real API route, with limits named**; design only, nothing written into the game repo.
- **D11 done**: one page the Lead can hand over.
- **Unchanged:** $0, no upload, no Studio launched, no credential touched, jobs 16/17 PREP COMPLETE with the route
  repaired config-side; **holding for the window** (sequence and rollback in the run-sheet).
