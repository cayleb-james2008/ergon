# TEAM C — MONDAY BOUNDED PREP (2026-11-01 19:10–19:25 ET) — staged, dry-checked, nothing executed

Owner: `trading-lead`, Team C. Tier S. **Read-only prep only: no live action, no order, no wallet op, no page action, no
reload, no desk process touched, no engine/config change, $0 spend.** All artefacts under
`board/probes/team-c/monday-2026-11-02/`. This is the receipt for the Lead's bounded-prep order.

## 0. P0.1 receipt verified (the gate that lets Monday exist)
- Signoff: `board/reports/monday-runbook-v7-checker-signoff.md`, signer `checker-trading`, **2026-11-01 19:08 ET
  (2026-11-02 00:08Z)**, fresh-context Tier S. It pins `board/reports/monday-futures-runbook.md` **v7**, **407 lines**,
  **sha256 `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`**.
- Same signoff filed at the runbook's P0.1 path `board/probes/team-c/monday-2026-11-02-precheck-signoff.md`; the void v6
  signature preserved at `board/probes/team-c/monday-2026-11-02-precheck-signoff-v6-VOID.md`. Both present, mtime 19:10 ET.
- **I re-verified the pinned text myself at 19:11 ET:** `sha256sum board/reports/monday-futures-runbook.md` =
  `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`; `wc -l` = 407; v6 preserved copy still
  `027ca4e4…` / 320 lines. **P0.1 is valid on the current text and I will not edit the runbook.**
- The signoff's §5 operational precondition (page load after 09:00), §6 path note (done by the Lead) and §7 packet note
  (done by the Lead) are recorded; nothing else in it changes a PASS bar.

## 1. Staged files (all dry-checked; nothing run against the desk)
| File | What it is | Dry-check |
|---|---|---|
| `board/probes/team-c/monday-2026-11-02/START-HERE.md` | day index: order of operations 09:00 → 16:20, authoritative doc, helpers, non-negotiables | read-through |
| `board/probes/team-c/monday-2026-11-02/guard.md` | evidence-log skeleton with empty slots keyed to every runbook section: §0 pre-guard (hash + page load + P0.2/3/4), §1 09:05 guard (5 items), §2 09:15 five attestations individually + ARMED, §3 binding (c2), §4 mode re-verify, §5 window + ORB15 branch + cards, §6 16:05 flat, §7 16:20 outcome (6 items), §8 post-day | read-through; every slot names its command + PASS bar |
| `board/probes/team-c/monday-2026-11-02/runner.sh` | ONE copy-ready **read-only** operator runner: `hash, pages, guard, a, a2, b, c1, c2, d, e, armed, cards, flat, outcome, all`; reproduces the runbook's own commands verbatim, prints `EXPECT:` per leg, never changes a PASS bar | `bash -n` → **BASH_SYNTAX_OK**; `--help` clean; **static compile of all 18 embedded Python snippets → ok=18 bad=0** (bash -n does not see heredoc Python, so this check matters) |
| `board/probes/team-c/monday-2026-11-02/page-reload.py` | the ONE allowed page load (checker §5): one CDP `Page.reload` on the signed-in `trader.tradovate.com` tab, no sign-in/typing/clicks/navigation/cookie read; exit 3 = no tab ⇒ STOP, exit 4 = CDP failure ⇒ STOP | `ast.parse` OK; `--help` OK; **not executed** |
| `board/probes/team-c/monday-2026-11-02/orb15-branches.md` | the two ORB15 operator notes — **Layer B** (ruling = deliberate exception: who taps, tap procedure, card-expiry rule, rails that still apply, Layer-B stop rules) and **HOLD** (observation-only, 0 fills claimed, PASS-empty rules) | read-through; both sit inside the v7 gate rule |
| `board/probes/team-c/monday-2026-11-02/logs/` | created by the runner at run time (one timestamped file per step) | n/a |
Nothing outside `board/probes/team-c/monday-2026-11-02/` and this report was written; the runbook was not edited;
the signed v6/v7 artefacts were not touched.

## 2. Item-by-item against the Lead's order
1. **Evidence slots** — `guard.md` created, keyed to the runbook sections named (Guard / (a) / (a2) / (b) / (c1)(c2) /
   (d) / (e) / 09:45 window / 16:05 / 16:20), plus the 09:00–09:05 pre-guard block.
2. **Five attestation blocks as one runner** — `runner.sh`; read-only commands only; expected outputs printed from the
   runbook's own PASS bars; parses (`bash -n`) and all embedded Python compiles. No step executed.
3. **ORB15 branches** — `orb15-branches.md`; Layer B names **Cayleb** as the tap (or the Lead as his operator only with
   Cayleb's written authorisation; never the Team C seat), keeps 1 MES / 2 trades/day / 15-min cooldown / −$400 / +$900 /
   flat 16:05 / 09:46–12:00 / card 2-min expiry / no stale-feed entry / firm-side $1,000 + 40 %, forbids blind retries and
   requires broker-record fills; **no ruling ⇒ HOLD is the default** (observation-only, 0 fills claimed).
4. **Hash re-verify at 09:00–09:05** — confirmed: my first command of the day is `bash runner.sh hash` (and a manual
   `sha256sum`), expected `512752e4…` / 407 lines. **If it differs I report to the Lead immediately and the day does not
   start until a re-sign** — no editing the runbook, no improvisation. Recorded in `guard.md` §0.1 and `START-HERE.md`.
5. **Page-reload precondition** — staged as `page-reload.py` + `guard.md` §0.2, with the three post-reload verifications
   (tabkeeper `ok`; feed re-attached, `paused=false` — a FEED STALE card is recorded, not "fixed"; `account_state.frame_t`
   age < 60 s) and the stop rules (no tab / logged out / CDP failure ⇒ STOP for the Lead/Cayleb, **never a sign-in**). The
   reload is **not** performed tonight; it is performed once after 09:00 Monday, before the guard.

## 3. What I need from the Lead (only two things)
1. **Confirm the reload mechanism.** I staged the desk's own CDP `Page.reload` (`page-reload.py`) as the F5-equivalent for
   the checker's §5 precondition — it is the desk's documented allowed action (`reports/DASH-LIVE-AUDIT-2026-10-27.md`
   line 104: "reloaded the Tradovate tab (F5 only, which is the one action allowed on it)"). If you prefer a human F5 on the
   desk window instead, say so and I will use that; either way it is **one** reload, no sign-in.
2. **ORB15 status at 09:00 ET.** Unless the Nox/Cayleb ruling is on record in writing, Monday runs **HOLD /
   observation-only with 0 fills claimed** — that is the default I will take without further instruction. Nothing else is
   needed from you for the observation day.

## 4. Fences held
Read-only; no execution of any desk command or page action; no order, wallet op, credential, config/engine change, image
read, host input injection, or spend ($0). The runbook and both signoff artefacts are untouched; the only writes are the
five prep files above plus this report.
## 5. ADDENDUM — the Lead's answers recorded (2026-11-01 19:20 ET)
1. **Reload mechanism CONFIRMED:** the desk's own CDP `Page.reload` is approved for that single use (F5-equivalent,
   documented allowed action, precedent `reports/DASH-LIVE-AUDIT-2026-10-27.md:104`). Exit 3 / logged-out / CDP failure =
   STOP for the Lead/Cayleb, never a sign-in. Recorded in `guard.md` §0.2 and `START-HERE.md`.
2. **ORB15 — RULED (supersedes the 19:20 ET "no ruling" state): Nox 20261102T003600Z §1 — ORB15 HOLD (option ii)**
   (overnight drive run 2, 2026-11-01 19:31 ET = 2026-11-02T00:36Z filing; relayed by the Lead 2026-11-01 19:43 ET).
   **Monday is an observation day: branch H is the RULED state, 0 fills claimed; branch B is NOT available — the
   tap-to-approve model is revoked** ("cards are observation-only until the record is re-cleared and the G3/G4 executor is
   built and signed"). Recorded in `guard.md` §5 and at the top of `orb15-branches.md` (with the verbatim quote), and
   `START-HERE.md`'s line corrected from "default" to "ruled". The future re-clear is a **Team C record question brought
   back with evidence**, not a Monday improvisation. The P0.1-signed runbook v7 already defaults to exactly this — **the
   runbook was not edited** (`sha256sum` re-checked after the edits: `512752e4…`, 407 lines, MATCH).
3. **Hash addition recorded verbatim in substance** in `guard.md` §0.1: if the runbook hash differs at 09:00, report
   immediately — the day does not start until a re-sign; **do not edit the runbook**. Re-verified after these edits:
   `sha256sum board/reports/monday-futures-runbook.md` = `512752e4…`, 407 lines (**MATCH**; the runbook was not touched).
   The three patched files are markdown only; `runner.sh` and `page-reload.py` were not changed by this addendum.

— `trading-lead`, 2026-11-01T19:25 ET. Standing by for Monday 09:05 ET.
