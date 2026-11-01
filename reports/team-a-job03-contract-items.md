# Job 03 — six contract-freeze items for Team B (DSH contract/certified tree) and Nox (freeze decisions)

Prepared by Team A (`ergon-lead`), 2026-10-31 ~22:35 ET, on the Lead's instruction: send the six interface/planning items
now (they are not the package build; they cost nothing to answer and shorten the critical path when ownership lands).
Context and the full contract reading: `board/reports/team-a-job03-package-contract-note.md` (reference: the shipped
`@cayleb/dsh-fleet` 0.2.1 bundle, sha256 `55516f9ae5…`, DSH `0.2.2-alpha.2`, read-only).

**Boundaries stated up front:** no package build happens on these answers until the Lead/Nox rule package ownership. Items
that are design decisions inside the banked principles can be answered by Team B/Nox under the stop-asking ruling; anything
that **changes the shipped contract, licensing, or the DSH pin** must be recorded, and if it hardens into an acceptance
change it is surfaced to Cayleb via Nox (not smuggled through a build).

**One-line ask:** answer C1–C6 in one message (options below; my recommendation marked **REC**), record the answers as the
frozen contract set for the Ergon-on-DSH package, and name what changes if an answer is different.

---

## C1 — Who owns and freezes the DSH plugin contract, and at which version?

- **Question:** does the bundle/row contract (npm package with `dsh.bundle.patch`, Cordis rows, one provider per service,
  id-override cannot swap packages, client injection list) stay frozen at DSH `0.2.2-alpha.2`, and does Team B's release
  0.2.2 work (Job 06) plan any packaging-contract change?
- **Options:** (a) freeze now at alpha.2 and treat release 0.2.2 as version-string only; (b) freeze at the 0.2.2 release
  after Job 06; (c) **REC:** freeze at alpha.2 for the W1 skeleton, with an explicit contract-delta review when 0.2.2's
  release lands, before any package acceptance.
- **What freezes when:** the contract baseline freezes at Team B's Stage 2/3 contract close; (c) adds one recorded delta
  review gate at the 0.2.2 release.
- **Why it matters to the three-piece mapping:** piece 2 (DSH baseline) is only a baseline if its plugin/row contract is
  stable; a contract move after the Ergon plugin is built would re-open W1–W2 and invalidate rehearsal evidence.

## C2 — Which exact DSH build does the Ergon package pin?

- **Question:** does Ergon pin `0.2.2-alpha.2` (the build installed and fleet-proven on the real install) or wait for the
  0.2.2 release (Job 06)?
- **Options:** (a) **REC:** pin alpha.2 for W1 and re-pin to 0.2.2 only after Job 06's release is installed and
  checker-proven, recorded as a deliberate version bump with a re-run; (b) pin 0.2.2 now (blocked until Job 06); (c) no pin
  (unacceptable — the package must name its baseline).
- **What freezes when:** initial pin at W1 start; the re-pin decision at Job 06 close.
- **Why it matters:** the package's `engines.dsh` is the baseline piece; a floating pin makes every acceptance statement
  ambiguous and lets a stale harness pass as the product.

## C3 — Are the client-injection packages available and version-matched for an Ergon client module?

- **Question:** are `@deepseek-ai/dsh-client-locale` and `@deepseek-ai/dsh-client-ui-plugin-manager` (the reference
  bundle's declared injections) available/version-matched at the pinned DSH build, and what else may an Ergon client module
  inject?
- **Options:** (a) **REC:** confirm the two reference injections are the supported set (Ergon uses them and adds none), with
  a named contact for a missing one; (b) Ergon supplies its own client packages (more coupling, needs Nox sign-off);
  (c) server-only MVP now, front end deferred (named dependency, not invented).
- **What freezes when:** at W1 client-module wiring; the answer decides whether the Ergon front end is in phase 1 or phase 2.
- **Why it matters:** piece 3's identity is the front end — one long-running session with the personal agent and the sidebar
  with apps plus the live knowledge graph. Without the client contract the plugin is only half the product.

## C4 — What is the shipped web-client contract a bundle may render/replace?

- **Question:** what exactly may a bundle change in the web shell (session layout, sidebar sections, per-app routes)? The
  only shipped examples are locale + plugin-manager; nothing documents the surface Ergon needs.
- **Options:** (a) **REC:** Team B writes a one-page client contract plus one minimal example row (a sidebar section), so
  the Ergon plugin builds against a documented surface; (b) Ergon treats plugin-manager as the de-facto template (fast, but
  brittle and undocumented); (c) front-end reshape deferred to phase 2 with a stock-UI pilot (fallback if (a) cannot land
  before W2).
- **What freezes when:** before W1 client rows; the contract page is recorded in the Ergon package README.
- **Why it matters:** the plugin's job is to turn the DSH baseline into the Ergon app; an undocumented client seam means the
  core deliverable is built on inference, which the checker cannot certify.

## C5 — Platform scope: Linux-only bundle, Windows companions, or Windows rows?

- **Question:** the reference bundle declares `platforms: ["linux"]`; the product's Windows surfaces (Get Help window,
  native ReviewDesk, WPF setup window) sit outside it. Does the Ergon package ship (a) a Linux bundle + separately installed
  Windows companions — **REC**, mirroring the current product reality; (b) a Windows bundle row set in phase 2; (c) an
  unspecified platform claim (not acceptable)?
- **What freezes when:** at W1's package manifest/README (declared platforms + companion install path).
- **Why it matters to the three-piece mapping:** piece 1 (management software) and the Windows surfaces are outside the DSH
  bundle; the plan's rehearsal already marks them `stand-in`/`pending` on Linux. The package must state this rather than
  imply a single-artefact Windows product.

## C6 — Naming, licensing and distribution of the package

- **Question:** package name (`@cayleb/ergon`?), license policy, OpenClaw attribution files (the repo keeps upstream MIT
  notices), and distribution (pinned private tarball vs npm registry)?
- **Options:** (a) **REC:** private pinned tarball + sha256, `private: true` with UNLICENSED or an internal license, keep
  the upstream OpenClaw MIT attributions exactly where upstream code remains, no public registry publish; (b) private npm
  org publish (needs Nox/owner account decision); (c) public publish (not authorised by any source — would be a
  licensing/publication change for Cayleb via Nox).
- **What freezes when:** before W1 packaging; any licensing/publication change is recorded and surfaced to Cayleb via Nox,
  never decided inside the build.
- **Why it matters:** the package is the product's front door; the name/licence/distribution decide how a company installs
  it and what the repo's upstream obligations are.

---

## Answer format (cheap for the responders)

For each item: `C1: a|b|c (+ any condition)` — one line each is enough. The answers are recorded as the frozen contract set;
if an answer is (an option not recommended), the delta is written down with its acceptance impact. Target: before W1
starts; nothing else waits on these except the package build itself (which waits on ownership).

---

## ANSWERS — frozen contract set (Team B, 2026-10-31 22:30 ET)

Source: `board/reports/team-b-contract-answers-C1-C6.md` (basis: certified tree `cf93107` / tree `e15c17a3…` /
`@cayleb/dsh-fleet` 0.2.2 on `origin/integration/0.2.2`; shipped 0.2.1 bundle `55516f9ae5…`). One line each: `C1: c`,
`C2: a (+condition not met)`, `C3: a (corrected)`, `C4: a (+b interim)`, `C5: a`, `C6: a`. No build starts from these; no
Team B tree change.

| Item | Answer | What it means for the Ergon package | Status / next |
|---|---|---|---|
| **C1** contract freeze | **(c)** freeze at DSH `0.2.2-alpha.2` for W1 + one recorded contract-delta review when 0.2.2 lands. **Correction: 0.2.2 is NOT version-string only** — additive delta: exports `./policy`, `./ledger`; `src/policy/*`, `src/ledger/*` in files; rows.manifest +15; cordis.patch +29/−3 (fleet-policy/ledger rows); `engines.dsh` unchanged; no removals. | The delta to record at 0.2.2 acceptance is small and enumerable; the W1 skeleton targets the alpha.2 contract. | **Frozen for W1**; delta review due at 0.2.2 acceptance (Job 06). |
| **C2** DSH pin | **(a)** pin `0.2.2-alpha.2` now; re-pin only after Job 06's 0.2.2 release is installed + checker-proven, as a recorded version bump with a re-run. **Condition not met:** 0.2.2 not installed (row 18 blocked on owner Wait; `fleet-dsh` still 0.2.1); the 0.2.2 package still declares `engines.dsh: 0.2.2-alpha.2`. | Ergon stays on alpha.2 until the install + checker proof lands; the package plan's W0 baseline is alpha.2. | **Frozen now (alpha.2)**; re-pin is a named future gate, not a blocker to prep. |
| **C3** client injections | **(a, corrected)** supported set = declared shell slots + `locale`; **C3 follow-up answered:** declared slots are `root`, `main`, `sidebar`, `sidebar.panellist`, `shell.overlay`, `shell.leading`, `plugins.bundle.config`, `new_session`, `more`, `dsh.plugin-manager.install-registry`; Ergon uses its own keyed `main` view + contributions into the conversation-domain slots for the one-session shape (replacing stock chat is a replacement risk, not assumed), `sidebar.panellist` for the section, keyed `main` keys for app views, and the declared resource model (`ctx.resources.register` + `useResource`) for the live knowledge graph; no plugin-manager injection. | The Ergon client module declares only the slots/resources it uses; no new global service needed. | **Answered / frozen-pending-Nox**; W1 plans the ADD-only surface (own `main[key]` + sidebar + graph resource); reshaping the stock conversation view is a separate named item escalated to Nox (options a/b/c in `team-a-job03-package-contract-note.md` §C3 follow-up) — plan shows both paths so the ruling changes only W1 client-row scope. |
| **C4** web-client contract | **(a)+(b)** Team B can write a one-page client contract + one minimal example row (sidebar section); the certified tree already carries a real packaged example: fleet client registers `main[fleet]`, a `shell.overlay` pill and a `plugins.bundle.config` contribution, Stage 5 recipe adds the sidebar entry. It lands with the operator-path/dashboard work, **not as a build now**. | The Ergon front end (single-session reshape + sidebar with apps + live knowledge graph) is buildable against the interim plugin-manager template, with the documented contract page as the named dependency; no contract change implied. | **Interim template approved**; contract page due with Team B's operator-path/dashboard work. |
| **C5** platform scope | **(a)** Linux-only bundle (`platforms: ["linux"]`) + separately installed Windows companions; fleet's device-control panels can drive an Android emulator / Windows VM remotely from the Linux bundle, but that does not make the bundle a Windows artefact. | The package declares Linux for pieces 2+3; the Windows surfaces (Get Help, native ReviewDesk, WPF setup) stay companions — matches the rehearsal spec's stand-in/pending marking. | **Frozen.** |
| **C6** naming/licensing/distribution | **(a)** private pinned tarball + sha256, `private: true` + UNLICENSED/internal licence, keep upstream MIT notices where upstream code remains, no public registry publish. Fleet pattern verified (`license: UNLICENSED`, `dsh plugin add <sha256> <file>`; `private` not set in fleet package — Ergon will set it). No OpenClaw code vendored by the fleet; Ergon keeps its own upstream notices. | Package name/licence/distribution decided: private tarball, no publish; OpenClaw attribution stays where Ergon's upstream code remains. | **Frozen**; any licensing/publication change remains Cayleb-via-Nox, never decided inside a build. |

**Effect on the package plan:** W0 (contract freeze) is now **grounded**: no ownership ruling dependency on the contract
questions themselves; the two named future gates are (i) the 0.2.2 install + checker proof before any re-pin (C2), and
(ii) Team B's one-page client contract + example row before the front-end contract is formally documented (interim
plugin-manager template approved for W1). No package build has started; ownership ruling still the gate.
