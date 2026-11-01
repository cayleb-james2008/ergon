# Job 03 R0 provisioning checklist — generic host (prep only)

**Status: PREPARED, NOT EXECUTED. No host exists yet, no purchase, no spend, no credentials handled.** This is the
team-side R0 checklist for the RHL dress rehearsal: what must be true on the fresh host before the operator's clock
starts, who does it, the exact commands, the evidence file each step must produce, and the stop rules. Host is
**generic** (the ruling names a dedicated cloud host, CPX22-class, ~EUR 8/mo per the host card); nothing here assumes a
named provider, a project id or a URL.

Companion artefacts (workspace board, not yet in the repo because PR #10 is under a fresh checker — they fold into
`docs/acceptance/demo-company/` in a follow-up docs branch after it merges):

- Step spec: `docs/acceptance/demo-company/REHEARSAL-R0-R7.md` (branch `docs/l5-acceptance-r0-r7`, PR #10)
- Operating sheet: `docs/acceptance/demo-company/RUN-SHEET.md` (same branch)
- Evidence skeleton: `board/probes/team-a-job03-run-evidence-skeleton/` (this batch; `pass-1/` + `pass-2/`, templates
  and `r0-preflight.sh`)

## 1. Gates that must hold before provisioning starts

| Gate | Must be true | Owner |
|---|---|---|
| G1 | Lane 1's installed output exists — the ruled DSH-era Ergon package is installed, install record exists | Lane 1 |
| G2 | C1–C6 frozen by Nox (`CONTRACT-C1-C6-PENDING.md` → confirmed) | Nox |
| G3 | Client-shape arbitration closed (ADD-only `main[key]` W1 view, or documented reshape + C4 page) | Nox |
| G4 | Host + DNS name exist; company Supabase project created | Cayleb via Nox (purchase/acts), Team A records |
| G5 | Fresh-context rehearsal checker named by the Lead; acceptance battery agreed | Lead |
| G6 | Run config accepted (models + sole route budget-proxy) | Team A |
| G7 | No-secrets rule accepted and the secure entry route named (owner enters values, never the lane) | Team A + Nox |

**Any gate false → do not start R0.** R0 is the first step of the pass, not provisioning leisure: it is timed and
recorded because "fresh host, no shortcuts" is part of the done-bar.

## 2. Host specification (generic)

- Ubuntu 24.04 LTS, single-tenant cloud host, CPX22-class (~EUR 8/mo; budget ceiling per company ≤ $100/mo incl. all
  running cost). Real amd64/arm64 both acceptable; the rerun must use the same class.
- Root/console access for the operator's first act is *not* needed: the install link and setup run in the browser; SSH
  is for the provisioning seat only and can be closed before the clock.
- Non-root admin account for the provisioning seat, key-only SSH, no shared passwords; no company state pre-installed.
- Outbound network available; **no inbound listener other than the ruled shareable URL path** (the operator's browser
  is the client; nobody installs anything to use an app).
- Clock synchronised (NTP); hostname and host fingerprint recorded.
- DNS: one name for the company app (for example `ergon.<domain>`), pointing at the host; **never `localhost`, never a
  raw IP** in a step's expected result.

## 3. Provisioning steps, commands and evidence

Run from the provisioning seat (or the host console for the captures). `OUT=pass-1/R0-pre` (and a second copy under
`pass-2/R0-pre` for the mandatory second pass). Every step is blocking unless marked.

| # | Step | Owner | Exact command / action | Evidence file in `OUT` | Pass condition |
|---|---|---|---|---|---|
| P1 | Image the host | Cayleb via Nox (purchase), Team A records | create the host from the ruled image; record provider, region, class, host id | `host-fingerprint.txt` (via P2) | host exists, region/class match the ruled spec, no company state |
| P2 | Fingerprint + clean-state check | Team A | `bash r0-preflight.sh OUT --repo /path/to/ergon --package /path/to/ergon-<ver>.tgz --url ergon.<domain>` (or step by step, same commands) | `host-fingerprint.txt`, `absence-of-company-state.txt`, `preflight-summary.txt` | script exits 0 (`R0 PREFLIGHT: PASS`); every checked state path `absent` |
| P3 | Package identity | Lane 1 + Team A | `sha256sum <package-tarball>`; record version; confirm Lane 1's install record matches | `package-sha256.txt` | tarball SHA-256 and version recorded; install record sha256 matches |
| P4 | DNS / shareable URL | Team A + owner | create/verify the A record; offline check `getent hosts <name>`; the real reachability proof is R3 from the second device | `url.txt` | name resolves, is not localhost/raw IP; R3 will prove reachability |
| P5 | Company data project | Team A provisioning + owner | create the company Supabase project (one project per company); write the connection config by the secure/write-only route; record **id and region only** | `project-id.txt` | id + region recorded; **no key/URL token/password in any evidence file** |
| P6 | Run config acceptance | Team A | `node docs/acceptance/demo-company/fixtures/check-run-config.mjs` (from the pinned checkout) | `run-config-check.txt` | prints `accepted: … route budget-proxy`, exit 0 |
| P7 | Repo pin | Team A | `git -C /path/to/ergon rev-parse HEAD` | `host-fingerprint.txt` (git section) | pinned commit matches the ruling / run sheet |
| P8 | Evidence dirs | Team A | copy/instantiate the skeleton's `pass-1/R0-pre` + `pass-2/R0-pre`; fill P1–P7 files | both `R0-pre/` dirs | both passes' R0 evidence complete before the first clock |
| P9 | No-secrets review | Team A | read the whole R0-pre pack: no credential values, keys, tokens, passwords, personal content | `preflight-summary.txt` (review note) | clean; if not, redact and re-check |
| P10 | Hand to operator | Lead/Team A | give Cayleb only the first mechanical instruction (R1: open the install link) | `index.md` start time | only then does the clock start |

**Time:** P1–P9 is ~30–60 min of hands-on work once the host and package exist; it is deliberately separate from the
operator's pass so the operator's clock measures only the operator's experience.

## 4. Preflight tool (self-tested, generic)

`board/probes/team-a-job03-run-evidence-skeleton/r0-preflight.sh` writes `host-fingerprint.txt`,
`package-sha256.txt`, `run-config-check.txt`, `absence-of-company-state.txt`, `url.txt`, `project-id.txt` (template
only) and `preflight-summary.txt`. It is read-only on the host apart from its own output directory: no sudo, no network
calls, no package installs, and it never reads, prints or stores a credential value.

Smoke test on this desktop (2026-11-01 09:09 ET, not a real R0, labelled as a tool self-test):

```
PASS  host fingerprint written
PASS  package hashed: tmp.i2RPcaUZqo.tgz 5adfbaa7de63771cf1821a17d8ad3cc1230b4a47bdb3f1e30a7095e6e5e7fdd1
PASS  run config accepted (route budget-proxy)
PASS  no company state on the checked paths
PASS  url name recorded: rehearsal.example.com
PASS  company project id recorded
R0 PREFLIGHT: PASS   (exit 0)
```

Fail-closed test (same day, no `--repo`, no `--package`, no project id): `R0 PREFLIGHT: FAIL (3 blocking)` exit 1 with
each blocking item named — the tool cannot be used to manufacture a clean R0.

## 5. No-secret rules (R0 form)

- Team A never asks for, prints, copies, logs or commits a credential value. The owner enters secrets at the secure
  write-only route when the step reaches it.
- Evidence records the *name/label/kind* of a configured value, never the value; project **id and region** are fine.
- No `.env`, key file or token is ever copied onto the host by this lane; if a step seems to need one, **stop and route
  to Nox**, do not improvise.

## 6. Cost and authorisation

- Host class ~EUR 8/mo; the per-company running ceiling is ≤ $100/mo (ruled). **No spend of any kind starts before G1**
  (Lead ruling 2026-11-01: do not raise the host until the RUN is schedulable).
- Purchase, DNS registration and Supabase project creation are Cayleb's acts routed through Nox — never Team A's.
- Team A's own spend this batch: $0 (local scripts, no model calls, no network calls by the preflight tool).

## 7. Stop rules

- Any blocking preflight item fails → stop, name it in `preflight-summary.txt`, do not start the clock.
- Package hash mismatch, or install record ≠ tarball → stop; that is Lane 1's install, not a retry.
- DNS is localhost/raw IP → stop; the URL shape is part of the ruled product.
- A secret is needed and the secure route is not named → stop and route to Nox (G7).
- Same provisioning step fails twice → stop and report; no third workaround.
- The rehearsal is not started in this batch. R0 here is a checklist plus a self-tested tool, nothing more.

## 8. Successor

1. When Gate G1 lands and the Lead schedules the RUN: run `r0-preflight.sh` on the real host into both passes' `R0-pre`
   dirs, complete P1–P9, and hand the operator the first instruction.
2. After PR #10 merges: fold the evidence skeleton + this checklist into the repo
   (`docs/acceptance/demo-company/run-evidence/` and a `R0-PROVISIONING.md` page) as a docs-only branch, so the repo,
   not the workspace board, is the source of record for the run.
3. Nothing else is authorised until the checker on PR #10 reports.
