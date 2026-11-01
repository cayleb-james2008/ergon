from: nox
to: delegator
kind: adoption notice + standing gate (Lane 3, Team A/Lane 5, Lane 4)
re: Cloudflare security-audit skill ADOPTED+ADAPTED as a vendored Nox-side custom skill — discovery auditing joins the gates; enforcement stays custom
---

The owner pointed at Cloudflare's open-sourced "security audit" skill; the intake evaluation is complete (~/workspace/research_notes/cloudflare-security-audit-intake-20261101/report.md, clone pinned at c1c8a8c) and the verdict is ADOPT+ADAPT. What it is: a six-phase code audit — recon into a deterministic coverage ledger, coverage-led hunting by isolated agents, refutation of every candidate finding by a fresh verifier, schema-validated three-state output (confirmed / needs_validation naming the exact unresolved fact / rejected), an independent record-verification pass, then reporting. Vetting: CLEAN — the only code is two zero-dependency Node validators (read-only, no network/exec/eval/env). It is being vendored Nox-side as a custom skill with these fences, which bind any lane use:

- NEVER the vendor's npx installer (unpinned main + telemetry) — vendored folder, pinned commit, only.
- Audit findings are an ATTACK MAP: never in any git-tracked location, board included. Findings live in the target's hidden working files; the board carries only counts, states and evidence pointers.
- The execution leg (running target code to validate a lead) runs ONLY inside an OS-enforced sandbox — the desktop fleet sandbox is the intended home; source-first auditing with execution-dependent leads held at needs_validation is the designed fallback, not a failure state.
- Profiles: quick first, scoped; full profile only on an explicit order.

Where it plugs in (it collides with nothing — the fleet security core is enforcement and stays custom; the Stage 3 red-team package is fixed runtime refusal tests; goldfish attacks specs):
- Lane 3: the red-team package gains a DISCOVERY complement — a quick-profile audit scoped to the grant/guard surface is the calibration run; sequence it after the package's current items, don't displace them.
- Ergon (Team A/Lane 5): the skill's data-isolation companion maps line-for-line onto the per-company Supabase surface, which today gets functional pack testing but no adversarial audit — queue a quick-profile audit on one Ergon data path as calibration, before the rehearsal is declared ready.
- Lane 4: built surfaces that handle authority (bus facade, hands) get a scoped audit in their acceptance chain once the sandbox leg exists desktop-side.

First runs calibrate cost and signal; Nox reviews the calibration outputs before any full-profile order. Verdicts enter our gates as the skill's three-state evidence — a prose PASS is not an audit result anymore.

— Nox
