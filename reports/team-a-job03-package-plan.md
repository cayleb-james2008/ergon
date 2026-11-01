# Job 03 — Ergon-on-DSH three-piece package: scope / effort / date plan (for the owner decision)

Prepared by Team A (`ergon-lead`), 2026-10-31 ~22:00 ET. PREP ONLY — no build is started: the Lead's ruling is that the
package stays Team A's Job 03 first build but **ownership must be confirmed** by the Lead/Nox before any code starts.
This plan is what the Lead can put to the owner in one page.

## What is missing today (the gap in one paragraph)

The ruled product is a **three-piece package**: (1) the management ticket/support software, (2) the DeepSeek Harness
baseline, (3) the Ergon plugin that turns the baseline into the Ergon app. Ergon Lite = pieces 2+3 without piece 1. Today
DSH 0.2.2-alpha.2 and the fleet bundle `@cayleb/dsh-fleet` 0.2.1 exist (Team B's product), but **no Ergon package
exists**: the `ergon` repo's `plugins/*` are OpenClaw-era (`"openclaw": {...}`), `grep 'dsh.bundle'` over the repo = 0,
and the Kimi redo handback has not arrived. There is no owner of the packaging work in the 19-job programme until the
Lead/Nox assign it; this plan assumes Team A builds it on merged `main` (Ergon is fully done first).

## Work breakdown (WBS) — refined from the Job 03 prep pack

| # | Work package | Tasks | Depends on | Effort (days) | Done-condition |
|---|---|---|---|---|---|
| W0 | **Contract freeze** | Read Team B's DSH plugin contract (bundle = npm package with `dsh.bundle.patch` + Cordis rows; reference: `dsh-fleet/package/package.json`, `dist/cayleb-dsh-fleet-0.2.1.tgz`); record the exact contract version; confirm with Team B/Nox. **STATUS: Team B answered C1–C6 (frozen set recorded: freeze alpha.2 + 0.2.2 delta review; pin alpha.2 until 0.2.2 install+proof; client injects declared slots + locale; Linux bundle + Windows companions; private tarball). Remaining: Nox freeze-decision records (Lead routing) and Team B's client-contract page (interim template approved). C3 follow-up answered: declared slots + resource model confirmed, stock-chat replacement explicitly not assumed.** | Team B Job 06/08 contract freeze | 0.5 | Written contract note + pinned DSH version (0.2.2-alpha.2) + recorded C1–C6 answers |
| W1 | **Ergon plugin bundle skeleton** | npm package declaring `dsh.bundle.patch`; Cordis rows for the Ergon front end. **Default W1 client row set (ADD-only, no arbitration):** `slots` + `locale` injection; Ergon's own keyed `main[key]` workspace view; `sidebar.panellist` entry/panels; app views as keyed `main` keys; live knowledge-graph view via the declared resource model (`ctx.resources.register` + `useResource`, plugin-owned host fetch/stream route). **Reshape path (only if Nox rules option b AND the C4 client-contract page covers it by W2):** add rows targeting the stock conversation view (`main[key=conversation]`/`chat`) — the C4 page is the blocking dependency; otherwise the reshape is a named phase-2 delta, never assumed. `dsh plugin add` by sha256 into a clean DSH profile; uninstall leaves the profile clean; `engines.dsh` pin. | W0 | 2–3 | Clean profile boots with the plugin, the Ergon `main[key]` view opens from its sidebar entry, panels + graph render, uninstall/re-add restores a byte-identical profile; automated check + fresh checker. (If reshape ruled: the stock conversation view additionally shows the Ergon reshape, with the C4 page cited.) |
| W2 | **Management piece packaging** | Package piece 1 as an app on the company host: control service (tickets, devices, budget proxy), ReviewDesk console, ticket queue; decide plugin↔service boundaries; the warehouse board is piece 1's first app but its source is on the work PC (fold-in item, blocked) | W0, W1 | 3–4 | Management service installs and runs on a fresh host; the plugin reaches it over the documented route; board app slot ready (actual board arrives with the work-PC window) |
| W3 | **Company setup, enrolment, data** | First-run company setup (name/timezone/roles); per-device enrolment (USB-kit join + owner approval); shareable non-localhost URL; Supabase per-company wiring (schema `ergon_warehouse` fallback; provider-swappable); agent home on the host | W2 | 2–3 | A fresh host runs setup to the first employee's first ask; a second device reaches the URL; Supabase project wired (or declared dry-run with a stub) |
| W4 | **Assemble + driven rehearsal** | Assemble the three pieces into one installable package; drive the R1–R7 rehearsal on a fresh host (non-technical operator's view, no pre-seeded state); fix loop; per-step evidence pack | W1–W3; rehearsal host | 2 | One complete clean rehearsal pass, evidenced step by step; every finding fixed or named with evidence |
| W5 | **Hardening, docs, gates** | Installer/upgrade/uninstall paths; plain-language enrolment runbook (draft exists); security-baseline revisit for the internet-facing host (doctrine requires a deliberate revisit); fresh-checker acceptance battery on the final package | W4 | 2 | Checker PASS on the package + acceptance battery; runbook reviewed; security decision recorded |
| | **Total** | | | **11–14 days maker + checker** | |

## Milestones and the owner's decision points

- **M0 (day 0–1): contract note + skeleton plan.** Owner decision needed first: **assign the package to Team A** (or
  elsewhere) and confirm Team B freezes the plugin contract. No build before this.
- **M1 (≈day 4–5): plugin skeleton in a clean DSH profile.** Demonstrable to the owner on a screen recording; no company
  data yet.
- **M2 (≈day 9–10): management piece + first-run company setup on a rehearsal host.** First point a non-technical user
  can see the product shape.
- **M3 (≈day 12–14): assembled package + driven rehearsal + checker gate.** Deployment candidate.
- **Owner acts inside the plan:** authorise the rehearsal host purchase (€8/mo CPX22-class — see
  `team-a-job03-host-card.md`); confirm Supabase org/project route; open a work-PC window (board fold-in); later, the
  laptop/Windows seat for the Windows-only surfaces.

## Date risk and the two honest options for w/c 2 Nov

The package does not exist on 31 Oct; the contract is not frozen; the rehearsal host is not purchased; the board source is
on an offline PC. **A full three-piece deployment in w/c 2 Nov is not achievable.** The Lead reports this as slippage.
Options for the owner:

1. **Limited pilot (recommended only if a date matters more than completeness):** deploy DSH + fleet bundle + the
   management piece on a host for the first employees, with the Ergon plugin as the next milestone; board stays LAN-demo
   on the work PC; rehearsal covers only what exists. Named risk: the "one long-running session with the personal agent"
   experience (the product's front door) is not in the pilot.
2. **Slip the full deployment** to the date the package is checker-proven (M3 ≈ 2–3 weeks from a start decision). This is
   the recommendation: the done-bar is "ready to be installed and set up in any company, fully end-to-end tested
   multiple times" — a pilot would not meet it, and the honesty clause says name it rather than smooth it.
3. **Hybrid:** start the build now (M0→M2) on the rehearsal host; keep the deployment date as "as soon as M3 passes";
   if the owner needs a w/c 2 Nov touchpoint, show M2 on the rehearsal host as a demonstration, not a deployment.

## What Team A needs to start (owner/Lead decisions, in order)

1. Ownership confirmation for the package build (this plan) — **the gate**.
2. Team B's contract freeze version and the DSH pin (0.2.2-alpha.2).
3. Rehearsal host purchase (Cayleb's €8/mo act) or the Lead's written OK to rehearse on a throwaway local VM/container
   first (asked in the consolidated ask).
4. Supabase org/project confirmation (one project per company; the existing "Ergon" project is the seed).
5. Work-PC window for the board source (never rebuild from the repo copy).

Refined prep artefacts beside this plan: `team-a-job03-host-card.md` (one-screen purchase card),
`team-a-job03-enrolment-runbook.md` (employee + manager runbook draft), `team-a-job03-slippage.md` (slippage list with
evidence), and the earlier `team-a-job03-prep.md` (hosting/Supabase/dependency detail).
