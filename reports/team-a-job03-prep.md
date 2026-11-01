# Team A — Job 03 preparation pack (dress rehearsal + RHL deployment w/c 2 Nov)

Prepared by `ergon-lead` 2026-10-31 ~19:35 ET, per the Lead's ruling (Q1: the Ergon-on-DSH package is Job 03's first
build; write the scope/effort/date-risk plan now, build nothing until reviewed). **Nothing in this pack is executed and
nothing was bought.** Sources are cited; the facts marked UNVERIFIED are exactly that.

## 0. Restatement (what Job 03 must deliver)

One complete clean **driven dress rehearsal** of Cayleb's actual RHL setup from the non-technical operator's view (fresh
host → company setup → first personalised agents → shareable non-localhost URL from a second device → phone enrolment →
desk task → voice/message flow), every finding fixed and re-run, the warehouse board folded in from the work PC and
wired to the company database, and a plain-language first-employee enrolment runbook plus a dated slippage list for the
week commencing 2 Nov 2026. Done-bar: Cayleb's words "ready to be installed and set up in any company, fully end-to-end
tested multiple times, code quality reviewed to the [expletive] moon" — with the honesty clause that anything short of
that is named with evidence. "Code proven" is not "product done"; the rehearsal is the done-bar.

## 1. The gap that sets the schedule: there is no Ergon package to rehearse yet (Q1 plan)

What exists **today** (verified 31 Oct, evidence in `board/reports/team-a-phase1.md` §2.5):

- The **DSH baseline**: DeepSeek Harness 0.2.2-alpha.2 is installed on the real fleet install, and the fleet plugin
  bundle `@cayleb/dsh-fleet` 0.2.1 exists as a packaged DSH bundle (`dsh-fleet/dist/cayleb-dsh-fleet-0.2.1.tgz`,
  sha256 `55516f9ae9535dc585da96587af45f55d83edac67c88e48d3bdc32ad652d10d4`, built 31 Oct 10:46 ET). That is Team B's
  product, not Ergon's.
- The **Ergon repo** contains the product's source pieces (`support/server` control service with the ticket/budget-proxy
  path, `support/review` ReviewDesk, `suite/*` modules, the `rhl-*`/OpenClaw-era plugin packages, the Windows setup
  window) — but **no Ergon-on-DSH bundle**: `grep -rn 'dsh.bundle'` over the repo = 0 hits; the `plugins/*` packages
  declare `"openclaw": {...}`. The three-piece package (management software + DSH baseline + Ergon plugin) is **not
  assembled anywhere reachable**.
- The doctrine's product shape (`docs/nox-governing/ergon-current-state-brief-v2-2026-10-30.md:14,35,41`): three pieces;
  Ergon Lite = pieces 2+3 without piece 1; the Ergon plugin reshapes the DSH front end into one long-running session with
  the user's one personal agent, carries its own sidebar section (click-through apps + live knowledge-graph view), and
  the agent's home is the company cloud host. "Ergon IS the installer."

So Job 03's first build is **W1–W5 below**. It consumes Team B's DSH plugin contract (a third-party DSH plugin is an npm
package whose `package.json` declares `dsh.bundle.patch` — a Cordis entry-patch list, e.g. `cordis.patch.yml` — and
whose patch rows name the package's own modules; the fleet bundle is the reference implementation of that contract,
`dsh-fleet/package/package.json`). The contract itself is Team B's to freeze (Job 06/08); until it is frozen, W1's
interface work is provisional.

### W1–W5: scope, effort, date risk (plan only — no build until the Lead reviews)

| # | Work package | Contents | Effort (working days) |
|---|---|---|---|
| W1 | **Ergon plugin bundle skeleton** | npm package declaring `dsh.bundle.patch`; plugin rows for the Ergon front end (one long-running session with the personal agent, sidebar with the app list + live knowledge-graph view); install into a clean DSH profile with `dsh plugin add` by pinned sha256; uninstall cleanly | 2–3 |
| W2 | **Management piece packaging** | Package the management ticket/support software (control service + ticket queue + ReviewDesk console + budget proxy) as the piece-1 app; decide what runs on the company host vs what the plugin calls; the RHL warehouse board is piece-1's first app but its source is on the work PC (see slippage) | 3–4 |
| W3 | **Company setup + enrolment + data** | First-run company setup (company name/timezone/roles), per-device enrolment (USB-kit join + owner approval), per-company Supabase wiring (one project per company, schema `ergon_warehouse` fallback), shareable non-localhost URL, agent home on the host | 2–3 |
| W4 | **Integration + rehearsal evidence** | DSH baseline + plugin + management piece assembled into one installable package; the driven R1–R7 rehearsal pass on a fresh host; fix loop; per-step evidence pack | 2 |
| W5 | **Hardening, docs, checks** | Installer/upgrade/uninstall paths, the plain-language enrolment runbook, security-baseline revisit for an internet-facing host (the doctrine requires this deliberately, never silently), fresh-checker acceptance battery | 2 |

**Total 11–14 working days** for one maker plus a fresh checker; call it **2.5–3 weeks** at programme pace. **Date risk:
high against w/c 2 Nov** — the package does not exist on 31 Oct, the fleet contract is not frozen, and the rehearsal host
is not purchased. A w/c 2 Nov deployment of the full three-piece product is **not achievable**; the honest alternatives
are (a) a w/c 2 Nov **limited pilot** on the pieces that can exist by then (the DSH baseline + fleet bundle + the
management piece running on a host, with the Ergon plugin as the next milestone), or (b) slip the full deployment to the
date the package is checker-proven. This is the slippage the Lead is reporting; the pick is Cayleb's.

**Dependencies of the plan:** Team B's DSH plugin contract freeze (Job 06/08); Team B's release 0.2.2 (Job 06) as the
baseline version to pin; Cayleb's host purchase (§4) and Supabase access decision (§3); the work-PC window for the board
source (§5). Kimi's handback (Job 05), if it arrives and is adopted, may change the plugin's shape — no plan assumes it.

## 2. Hosting plan for the ruled single-tenant model

Ruled model (doctrine v2 `:41`): a **dedicated cloud host per company**, never a shared multi-company service; standard
**Hetzner CPX22 class, ~€8/mo, US Ashburn**; Nox holds native control so issues can be fixed remotely. Hosted access is
the pre-approval phase of the same product: apps hosted on one machine, everyone uses the same apps through a shareable
URL (never `localhost`), nobody installs anything to use an app; agent profiles and keys live on the host only. Budget:
**≤ $100/month per company running cost** (Lead's Q3 ruling: the product running cost governs; API credits are a
planning anchor, not authorised spend).

Plan (per company): a single CPX22 instance (2 vCPU/4 GB/40 GB class, Ubuntu 24.04 LTS — a stock image, no exotic
packages), one hostname on the company's own subdomain in an https URL, DSH + Ergon plugin + management piece installed
by the product installer, per-company Supabase project for data, backups and OS patching enabled, and the
private-network-only security clause revisited deliberately for an internet-facing host (doctrine requires recording
what replaces it — the zero-friction baseline: per-device revocable tokens, passkey + one-time recovery code, full
logging, automatic patching, private backups). Rehearsal plan: passes 1–3 on a **throwaway local VM/container** on this
desktop (fresh host, no spend — this is Q2's "may we" question), final pass on the provisioned cloud host once Cayleb
authorises the €8/mo purchase. **No purchase has been made and none will be without his recorded go.**

## 3. Supabase plan

- One Pro org of ours, **one project per company** (each project its own isolated database; doctrine v2 `:43`). Data
  provider stays swappable (RHL may have its own database hosting; if so the wiring adapts).
- Seed state available today: the **"Ergon" Supabase project**, schema `ergon_warehouse`, tables `lines`/`entries` —
  verified **19 lines / 12 entries** on 30 Oct (doctrine §3; weekend mission orders name project ref
  `yhpjpjeedpuvcgnvfpka`). Board data in that project is RHL's first app's data path.
- Per-company provisioning steps to design (not executed): create project under the org → create the company schema
  (per-app subsections under an **apps** category, legible to a non-technical browser) → service credentials into the
  host's write-only config → backup/retention → provider-swap seam.
- UNVERIFIED: current Supabase org plan/limits and who holds the credentials. **This is not ours to handle** — if a
  credential or a plan upgrade is needed it goes to Cayleb through the secure route; the pack needs his one-line
  confirmation that the "Ergon" org is the one to provision companies in.

## 4. First-employee enrolment runbook (draft; plain language, no secrets)

Written for the employee at RHL; each step says what they will see. Final wording waits on the package, but the shape is
fixed by the doctrine (hosted URL, no installs, per-device enrolment, agent home on the host):

1. **Open the link the company sends you** in your normal browser (Chrome/Safari/Edge). You will see the company name
   and a "Join this device" button. **Nothing is installed on your computer to use the apps.**
2. **Join this device:** click the button; the page shows a short code. Your manager approves the device (they see a
   request in their management screen). You will see "This device is approved".
3. **Set your sign-in:** choose a passkey (fingerprint/face or device PIN) and save the one-time recovery code the page
   shows. You will see "Signed in on this device".
4. **Meet your assistant:** the chat opens with your one personal assistant, already knowing your role and the company's
   tools. Ask it the first suggested question ("What is on my plate today?"). You will see an answer in plain language.
5. **Add your phone:** on your phone, open the same link → "Add this phone" → approve; you will see the phone listed as
   an approved device.
6. **Voice:** in any app, press the microphone once; the first time the page offers an optional "Enable voice" button;
   after it, the badge reads "Voice control activated". (Optional; the app works fully with typing.)
7. **Everyday use:** open the link, ask the assistant, use the apps; nothing to install, and the assistant keeps working
   when your desktop is off (it lives on the company host).

Manager-side steps (approving devices, re-issuing a recovery code, revoking a lost device) go in a second one-pager.

## 5. Slippage list (evidence on every line)

| # | Item that will miss / is at risk | Evidence | Consequence / owner |
|---|---|---|---|
| S1 | **Full three-piece package for w/c 2 Nov** | No Ergon-on-DSH bundle exists (A5; `dsh.bundle` grep = 0); effort 11–14 days (§1) | Deployment slips unless Cayleb accepts a limited pilot; Lead is reporting it. Owner: Team A plan, Lead decision. |
| S2 | **Board fold-in** | Accepted V11 board source lives only on work PC `RHLCO-WKS-09`, OFFLINE; repo copy is OLDER (doctrine v2 `:75`); tailnet `tailscale status` 31 Oct lists 6 nodes, no work PC; no SSH route | Fold-in waits on a work-PC window; never rebuild from the repo copy. Owner: machine window (Cayleb/office). |
| S3 | **Rehearsal host** | Host is a purchase (Hetzner CPX22 ~€8/mo); no spend authorised (Q2 ruling) | Rehearsal passes 1–3 can use a local throwaway VM/container **if the Lead/Cayleb allow**; final pass needs the host. Owner: Cayleb's go. |
| S4 | **Supabase provisioning** | Org/project plan and credentials UNVERIFIED (§3) | Data wiring cannot be rehearsal-tested against a fresh company project until one exists; RHL's seed project exists. Owner: Cayleb confirmation + secure credential route. |
| S5 | **Internet-facing security baseline** | Doctrine requires a deliberate revisit of the private-network-only clause for a cloud host (`:45`), never a silent drop | A security decision, not a code detail; must be written before deployment. Owner: plan → checker. |
| S6 | **Windows-only surfaces** | Get Help window, native ReviewDesk, the WPF setup window are Windows surfaces; this desktop has no .NET SDK (host 10.0.13 only) | The rehearsal's Windows steps need a Windows seat/laptop window; the rehearsal must mark them stand-in or pending, never clean. Owner: laptop window. |
| S7 | **API credits** | The ~$2,000/mo API anchor is a planning figure, not authorised spend (Q3) | Live-model stretches are capped by the Ergon testing envelope; a per-company API cap is a Cayleb decision before the first employee. Owner: Cayleb. |
| S8 | **Kimi handback** | 93 remote branches, 0 matching `kimi`/`redo`; no PR (job 05 check, 31 Oct) | If it arrives and is adopted it may change the plugin/management pieces; the plan does not wait on it. Owner: outside. |

## 6. What Team A does next without answers

No build, no spend, no credential. On the Lead's go after review, start W1 (plugin bundle skeleton against the frozen
fleet contract) and prepare the local rehearsal host; S2/S3/S4 remain named blockers. The rehearsal spec, per-step
evidence layout and the plain-language runbook are ready to be filled as the package lands.
