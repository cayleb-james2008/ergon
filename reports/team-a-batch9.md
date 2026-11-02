# Team A — batch 9: Nox RUN-2 rulings recorded; parts-bin curation-gate work-up delivered

Lane: Team A / `ergon-lead`. Report to the Lead. Written 2026-11-02 ~00:46Z. **Spend $0**; no purchases, no vendoring, no
repo writes beyond Team-A reports, no image reads/screenshots, no credential values, no sends.

## 1. Nox RUN-2 rulings (`20261102T003600Z`) — recorded and applied

Read in full from the board mail file (quote-checked). State as recorded, and where each one is applied in this lane:

| # | Ruling | Recorded / applied where |
|---|---|---|
| 1 | **Tier A "Remember" — ALLOWED, NARROWLY SCOPED**: one named subject + one named action type, both named on the surface ("Remember for \<subject\> for \<action\>"); **no "always allow" wording anywhere**; never cross-subject, never cross-action; revocable; every remembered grant visible in the approvals record; Tier B never offers remember/always-allow. Comp 02's affordance approved **only** in this scoped form; any wider form returns to Nox; **comps not reworked** (owner grading deltas govern). | `board/reports/team-a-parts-bin-curation-gate.md` §0 (ruling table) and §4 requirement **C-REM**; applied to **S4** (approvals sheet) G2/G3 checklist and to any approvals-record surface. Not applied to any built surface this batch (nothing built). |
| 2 | **W1 = (a) ADD-only** (arbitrated): Ergon's own `main[key]` view + sidebar panel(s) + graph resource contributions; stock chat **not** replaced in W1; reshape = later W2 delta admissible only with the C4 contract page. | Curation gate §0 + §4 requirement **C-W1**; applied to S1–S8's wait notes and to S6 especially (must fit the ADD-only surface, no stock-chat replacement assumption). |
| 3 | **Job 03**: G1's **Lane-1 leg discharged by the proven 0.3.0 install only**; **G1 whole NOT satisfied** (Ergon-on-DSH package artefact, fresh host, recorded `dsh` version still owed); **RUN stays held/unscheduled**. **C1–C6 confirmed**, with **C2 re-pinned to installed 0.3.0** (0.3.1 re-pin only after install + checker, as a recorded bump); **C4** one-page client contract + example row is **Team B's** obligation. Do not build the RUN. | Recorded in the curation gate §0 (R3-J03) and here. **No RUN action taken; no Job 03 build started.** The batch-8 Q1 trigger ("Nox rules G1 + C1–C6/W1") is now **resolved as a ruling** but the RUN remains externally blocked on G1-whole + G4 + G5 — no self-startable work. |
| 4 | **Candidate branch: NO MERGE**; Stage 2 PASS stands; stage left to tmpfs clearing on reboot; no owner act. | Recorded in the curation gate §0 (R4-CAND); nothing for this lane. |

Also recorded from the same ruling: the **fold-in nit successor is closed** — nit PR **#15 merged as `598da57`**
(2026-11-02T00:08:11Z, verified by `gh api pulls/15` → `merged=true merge_commit=598da57…`; `origin/main` = `598da57`).
Simulation-bank proof runs (DeepSeek acting agent + one Haiku comparison pass) remain **GATED and unwaived**.

## 2. Deliverable — parts-bin curation gate work-up

`board/reports/team-a-parts-bin-curation-gate.md` (PROPOSAL/GATE PREP ONLY). Contains, per the Lead's bounded task:

- **Licence lines re-confirmed (2026-11-02 ~00:45Z):** Vengeance UI MIT, Animate UI MIT+Commons, Forge UI usage-only — all
  re-fetched **HTTP 200**, text unchanged. Uiverse (403 bot-block) and Uilora (client-rendered; titles only) were **not
  re-readable from this seat tonight** — their lines stand on the annex's recorded primary-source fetches, and the open item
  is Uiverse **provenance** (R2), which is created at admission. No licence line changed; no decision tonight depends on the
  two non-re-readable pages (Uilora is pattern-only).
- **Per-candidate work-up S1–S12:** TAKE, LEAVE fences against the pack's own rules, a tailored **all-states checklist**
  (default/hover/focus/pressed/disabled/loading/error/empty + reduced-motion + High Contrast, with keyboard/aria specifics
  where the row needs them), and **screenshot-comparison criteria** written against `design/tokens.json` + `DESIGN.md` +
  `design/motion-spec.md` (named tokens/spacing/type/motion, not "looks close").
- **Cross-cutting requirements** every admission must satisfy: C-REM (Remember scoping), C-W1 (ADD-only), C-TOK, C-MOTION,
  C-TARGET, C-A11Y, C-PROV.
- **What MUST wait:** **S1–S8 = owner comps grading** (comps not reworked; graded deltas only), then the component-layer
  admission proposal; **S9–S12 = fleet pack definition** (SWEEP.md Nox-side; provisional) with **S9 additionally waiting on
  the R1 boundary** (in-app only vs distributed bundle → clean-room/drop); **G3 screenshot compare cannot run in this DSH
  seat** (image-read crash class) — criteria authored, comparison must be done by an image-permitted seat.
- **Gate mechanics wall named:** G2 needs a component; G3 needs image permission; G4 is the Lead's fresh checker. Nothing
  executed, nothing admitted, nothing vendored, no purchases.

## 3. Queue state after RUN-2 (updated from batch 8)

| Pull | State now | What is left |
|---|---|---|
| Parts-bin curation gate | **Delivered (this batch, proposal only)** | Owner comps grading for S1–S8; fleet definition + S9 R1 boundary for S9–S12 |
| Component-layer admission proposal | Next after grading — **not started** (grading deltas only) | Owner verdict |
| Job 03 RUN | **Held**, as ruled | G1-whole (package/fresh host/recorded `dsh` version) + G4 + G5; no self-startable work |
| Hallmark intake | Queued behind roster/finish work | Lead's queue order |
| Job 04 laptop lane | Waiting | Monday 01:00 ET laptop session (no route from here) |
| Job 05 Kimi handback | Standing | handback arrival |
| Simulation-bank proof runs | Gated | Nox/Lead opening them |

Nothing in this lane is self-startable beyond what is above; the blocks are external and named.

## 4. Fences held

No spend ($0); no purchases; no vendoring; no pack admission; no image reads in DSH; no credential values; no sends; repo
untouched this batch (no branch, no commit, no PR). Deliverables are board reports only.

**Artefacts:** `board/reports/team-a-parts-bin-curation-gate.md` (deliverable), `board/reports/team-a-batch9.md` (this
record), `board/reports/team-a-parts-bin-shortlist.md` (S1–S12 source), `board/reports/team-a-batch8.md` (prior queue record).
