# TEAM C — AUTONOMOUS FUTURES ORDER PATH: LARGE-BATCH BUILD PLAN (successor to ASK D14)

trading-lead, 2026-11-01 09:0x ET / 14:0xZ. **Plan only. No code written, no config edited, nothing armed, $0 spend.**
Authority: Cayleb's 31 Oct ruling (ORB15 beta is fully automated inside the rails, notify after, no per-trade taps) +
ASK **D14** (Lead proceeding with option 1 unless Cayleb says stop: build the autonomous order path as the priority Tier S
build) + Lead ruling X1 (Monday runs `cards`; attestation (e) says "NOT automated"). G0 is **answered**: the FundedNext Flex
$50K eval permits own bots/EAs **only through the platform login** and has **no API** — so the design's REST-API-preferred §4
is unavailable and the **CDP ticket route (design §4 fallback, G2.5) is the only autonomous path on this eval**
(`team-c-autonomous-path-amendment-g0.md`; proprietary survey `PROP-FIRMS-2026-10-27.md` marked `confidence: official`,
read 2026-10-27). **Monday 2 Nov remains guard/preflight only; no fills claimed; nothing here may run during Monday's session.**

## 0. Non-negotiables (unchanged; every batch below re-asserts them)
- Rails: -$400 daily / +$900 profit / $1,000 firm single-day cap / 40% consistency / flat **16:05 ET**; 1 MES
  (contract_id 4470322), **max 2 trades/day**, 15-min cooldown after a loss, no entries 16:05–18:00, min stop 2.0 pts,
  "no trade whose stop-out would cross -$400". No new funds, no second account, no size change, **exactly one live trader**.
- Tier S: **every gate signed by a fresh-context checker with veto** (maker never checker; the Lead spawns each seat) and
  **Cayleb's explicit written go** before `enable=1`. Any ambiguity → disable and alert, never retry blindly.
- No credential value is asked for, printed, copied, logged or committed anywhere. No inbound port on the desktop. No engine
  modification (it keeps emitting decisions/cards; only the new package may send).
- Evidence = venue record (orders/fills/balance frames) + decision log + executor hash chain; the engine's self-report is
  never evidence.

## 1. What changes from the 31 Oct design (the G0 correction)
| Design item | Before | Now (G0 answered) |
|---|---|---|
| §4 interface | Tradovate REST API preferred; CDP fallback | **CDP ticket automation only** on this eval. No API exists to design against. |
| G2.5 | extra gate "for the fallback" | **mandatory gate** before any demo send: pinned selectors vs a recorded page snapshot, tab-navigation test, guard that a send is possible only on the expected order ticket, per-step screenshots |
| NinjaTrader/NinjaScript EA | alternative | remains the *supported* alternative but requires **new spend + a platform move → Cayleb's decision**, not this build's; keep as the escalation option only if G2.5 cannot be passed |
| G0 evidence | on-disk 27 Oct read, network re-read failed | **re-read the official clause with its date before G1/B1** (network-dependent, not a Cayleb act). If it cannot be re-read, the build does not start — record, don't improvise |

## 2. Build batches (each tens of minutes to ~2 h; each names its done-condition and its gate)
### B0 — G0 re-read + firm-automation verdict artefact (network read, no build)
Do: re-read the FundedNext Flex automation/API clause from the official pages (plus the on-disk survey row), quote the clause
verbatim with the **date read**, and write `board/reports/team-c-firm-automation-verdict.md` (clause + date + what it permits
and forbids + the honesty limit if unreachable). Done-condition: file on disk with a dated quote, or a recorded
"unreachable at <time>, no build start" line. **Gate:** checker signs G0 (gate G1 cannot open without it). No spend, no sends.
*Scheduled after the 01:00 ET reset (network lane), not during Monday's session.*

### B1 — G1: design v2 for the CDP-only path (text only)
Do: rewrite design §4 to the CDP ticket route; promote G2.5 to a required gate; write the rails-to-code table (every §0 rail
mapped to a `rails.py` predicate with its refusal reason), the threat list (DOM change, tab navigation, wrong page, stale
frame, double send, orphan bracket, session expiry, 2FA prompt, engine/executor disagreement) and the kill-path list.
Done-condition: `board/reports/team-c-autonomous-path-design-v2.md` + rails table + threat list. **Gate: G1 checker veto**
(Veto conditions: any rail unrepresented, any credential in the repo plan, G0 absent, no kill path).

### B2 — containment package `live/exec/` (build, cannot send)
Do: build the design §5 package as a *disabled-by-construction* unit — `intent.py` (frozen intent schema v1, derived 1:1
from the engine ledger/cards, never invented), `rails.py` (pure `check(intent, state, config) -> (allow, reasons)`),
`guard.py` (fail-closed `enabled.json` + `go_id` + single-trader `flock` + flat-at-start + expiry), `executor.py` (one order in
flight; hash-chained `state/intents.jsonl`; disable-on-ambiguity; 16:05 auto-disable), `venue.py` (the **only** module allowed
to drive CDP — AST test asserts no other module opens a socket/page), `reconcile.py` (fills↔intents↔decisions 1:1).
Done-condition: unit tests green + AST proof that only `venue.py` touches CDP/network + `state/enabled.json` default-disabled
on disk + no change to `live/engine.py` or `live/config.json`. **No gate yet** (G2 needs the dry-run artefacts); a checker
pre-review of the test plan is welcome but is not a gate.

### B3 — G2: dry-run proof on closed days (no send anywhere)
Do: run `--dry-run --day <closed day>` through the real intent/rails/executor code with `venue.py` replaced by a recording
stub. Prove: (i) rerun → identical sha256 per artefact; (ii) compare vs that day's cards → 0 missing / 0 extra / 0 mismatched;
(iii) refusal table — every rail, stale feed, non-flat start, expired go, second process, unknown ack — each asserted to
produce **no send**; (iv) static AST proof of the send boundary + no credential read from the repo; (v) two dry-run days
(30 Oct cards day + one flat day) to cover both shapes. Done-condition: artefacts + hashes + refusal table in
`board/probes/team-c/exec-dryrun-<date>/`. **Gate: G2 checker veto** (any non-determinism, any refusal that sends, any second
socket opener = veto).

### B4 — G2.5: CDP selector/tab gate (read-only page work; checker pre-sign first)
Do: with the market closed (never during Monday's session), open the Tradovate order-ticket page in the existing signed-in
Chromium tab and record: a page snapshot (DOM/HTML hash), a pinned selector map, a tab-navigation test (ticket page ↔
positions page), a guard test that a send is possible only on the expected ticket page, and screenshot evidence per step.
Nothing is typed into the ticket, nothing is submitted. Done-condition: selector map + snapshot hashes + screenshots +
relation test in `board/probes/team-c/exec-dom-<date>/`. **Gate: G2.5 checker veto**, signed **before** the first demo send.
*Note: this step touches the live browser page read-only; it is not a live order and not a Monday action.*

### B5 — G3: demo venue (blocked-on-owner-assets)
Do: place/cancel/replace/duplicate-clientOrderId/partial/reject on a demo account; join the demo's own order-history export to
intents 1:1. **Blocked** until the demo environment and demo access are named (design Q3: FundedNext demo vs Tradovate demo;
who holds the demo keys — Cayleb/Nox; no credential handled by this lane). Done-condition: demo order-history export + 1:1
join file + the ack-semantics table. **Gate: G3 checker veto.**

### B6 — G4: pre-arming (never before every gate above is signed)
Do: kill-path drill — disable flag, cancel-all, manual flatten card, engine stop — each proven with evidence; then the go
record: **Cayleb's explicit written go**, relayed in writing, attached to a `go_id`. Done-condition: `go_id` + kill-path
evidence + G1–G3 signatures. **Gate: G4 checker veto.** Arming is then one manual step (`state/enabled.json` + service env
`EXEC_ENABLE=1`, restart the executor only); disarming is always allowed without approval.

### B7 — first live day (outside Monday; only after B0–B6)
Do: engine stays in `cards` as information-only; only `executor.py` sends; if the two disagree the executor refuses and
records it. 16:20 outcome check uses the design §10 six-item "autonomous fill" definition; auto-disable at 16:05. Any
ambiguity disables the day.

## 3. Ordering, dependencies, and what can start when
```
B0 (G0 re-read) ─► B1 (G1 design v2, checker veto) ─► B2 (package+tests) ─► B3 (G2 dry-run, checker veto)
                                                        └─► B4 (G2.5 DOM gate, checker veto) ─► B5 (G3 demo, owner assets) ─► B6 (G4 go, Cayleb) ─► B7 first live day
```
- **Can start after the 01:00 ET reset (not during Monday's session):** B0 (network read), B1 (text), B2 (local build).
- **Needs the Lead's checker seat:** G1, G2, G2.5, G3, G4 — one fresh checker per gate, veto each.
- **Needs owner acts:** B5 demo access (Cayleb/Nox names the demo + who holds keys), B6 Cayleb's written go, and only if
  CDP is rejected: the NinjaTrader route needs new spend (Cayleb).
- **Never:** any step inside Monday's 09:05–16:20 trading sequence; any edit to `live/config.json`/`live/engine.py`; any
  order on the live account before B6 is signed.
- **Envelope:** $0. The CDP route reuses the existing signed-in session, so no new spend. Demo access, if owner-provided, is
  free; the NinjaTrader alternative is the only spend path and it is a separate Cayleb decision.

## 4. Successor + adjacent prep (this plan's next write)
1. **01:00 ET reset lane (Monday):** Kalshi resume per `team-c-monday-packet-v7.md` §01:00 (units → readiness → fit-split
   pull → Stage A a–f; L\*=30 applied; freeze Tue–Wed). This lane then picks up **B0 → B1 → B2** with no metered spend.
2. **07:45 ET Monday loose-ends sweep:** trading legs in packet v7 (runner liveness, units/feed/tab, runbook hash, engine
   start time) — owned by upkeep-lead with lead backup; trading-lead supplies the checks.
3. **Monday trading sequence:** guard → attestations → binding → window → flat → outcome, per packet v7. No autonomy claim.
4. **What this plan explicitly does not do:** no multi-instrument, no sizing change, no strategy change, no Kalshi/meme
   integration, no unattended hours outside 09:30–16:05 ET, no credential in any repo/log.

## 5. Risks (top) and mitigations
| Risk | Mitigation |
|---|---|
| A bug that trades | G1–G4, default-disabled artefact, refusal table, one order in flight, disable-on-ambiguity, 16:05 auto-disable |
| DOM/page change breaks or mis-sends | G2.5 pinned selectors + page snapshot + expected-page guard + screenshots; fail closed |
| Stale feed drives an entry | rails require feed freshness at intent time (same 60 s rule as the engine) |
| Double control (engine cards + executor) | cards information-only; only `executor.py` sends; disagreement → refuse + record |
| Orphan bracket after a failed cancel | reconcile joins fills/orders; disable on unmatched fill; flatten path proven at G4 |
| Credential creep | no key needed on the CDP route (existing session); AST/no-secret scan in the dry-run; fresh 2FA = stop for Cayleb |
| Build creep past the rail | design §11 out-of-scope list is the fence; any addition needs a new gate |
