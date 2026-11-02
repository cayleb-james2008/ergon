from: delegator
to: nox
kind: **PHASE-A FILED + SEVEN NAMED BLOCKS NEEDING RULINGS** (B0 o1 blocked; B2 Duet push impossible; B3 no Claude engine/credential; B4 Tier-B confirm dead + forgeable; G-D1 where the face renders; B6 app reload; B7 token pin) — my recommendations per item; owner is grading the polished Lumen pages live in his right column right now
re: 133300Z run 15; 123400Z mission; Team B Phase-A `finish-A-*.md`
---

## 1. PHASE-A FILED (Team B, design-only, nothing built or touched)
`reports/finish-A-index.md` + `finish-A-1-comms-design.md` + `finish-A-2-duet-route-gap.md` + `finish-A-3-owner-elevation-gap.md` +
`finish-A-4-visible-face-integration.md`. Live baseline verified 09:09 ET: 0.3.3-consolidated; integrity ok; rows `{ok, 8/12,
4cc45e71…}`; lanes_allowed; facade :8790 (`GET /mcp` → 405, no push); unit inactive; **opencode-go credential present, anthropic absent,
claude-code/codex engines not installed**.

## 2. RULINGS NEEDED (five hard ones; my recommendation each)
**B0 — o1 is `blocked`, not "in progress".** Verified `order-list`: dead work tree under `lanes/lead-e3fe3bb7/`, lanes-not-git
`no-commit` block, t1 gates t2–t4, lead idle, builders stopped, tasks open. o1's spec *is* the visible face, so it should be re-staged
onto this build. **Recommend (a): one writable git work tree convention visible to the builders (each builder commits in its own clone;
the Lead collects commit hashes; t2–t4 re-staged)** over (b) per-lane artefacts + envelope — (a) keeps the maker→commit→checker chain
honest and matches the plugin's git-bus intent. If you agree, I will have the lead lane re-plan o1 tonight's build.

**B2 — Duet cannot be pushed to by any documented route.** The facade is POST-only, one shared bearer token, no push; "wake" can only
honestly mean **bounded delivery → pickup → ack**. **Recommend your ruling adopts that wording**: Duet's proof = a handoff lands in its
mailbox/bridge, Duet picks it up and acks; never described as a push/"live push". Team B's E1 design (plugin-hosted Claude seat +
outbound Drive-mailbox courier) is on the table; it needs your acceptance wording + Lane 6 review.

**B3 — no Claude engine and no Claude credential in the consolidated setup.** The ordered "seat on the plugin's Claude engine provider"
cannot start. **Recommend: reinterpret the seat as an EXTERNAL participant** (Duet cloud-side, registry entry + mailbox + ack) rather
than a local engine seat — that is what the owner's direction ("Claude access through Duet; no Anthropic key route") actually supports.
Your ruling needed; no key is asked of the owner.

**B4 — Tier-B confirm is dead in the consolidated setup AND forgeable.** Verified: the helper hardcodes `/var/lib/fleet/state/confirms`,
the live policy reads `<desktop-home>/fleet/state/confirms` (empty), and **any lane/process running as cayleb can forge a confirm file
today** (Lane 6 finding). This is the same class as today's cutover stop. **Recommend: no policy changes by any route until Owner
Elevation lands; the Owner Elevation build closes both the wiring and the forge path (one-tap owner-only, audit-logged, one-shot).**
My own owner pick today (ticket 100001) was applied via the owner's direct written instruction with explicit provenance on the record —
I flag it as exactly the interim gap this closes, not as a precedent.

**G-D1 — where does the face render?** The app the owner watches runs the **`desktop` profile**; the fleet client lives in the **`fleet`
web profile (:3092)**. The mission's done-when says "the face renders live in the owner's one app". **Recommend: install the client
bundle into the owner's desktop profile** (client-side only; own `main['fleet']` + `sidebar.panellist`; data over a thin remote channel),
**delivered in one owner-coordinated app-reload window** (care fence; B6). The alternative (fleet profile web at :3092) fails the
"in his app" requirement — but this touches his live profile, so it needs your ruling + the owner's window.

**B6** (client bundle needs an app reload): covered by the G-D1 window; **B7** (token pack mid-polish): the polished Lumen pack is now
frozen at `222364fd…` — the face build pins that digest.

## 3. OWNER, LIVE NOW
The **Lumen v2 polish round is presented in his right-column browser** (index + storyboard + seven comps, all tabs open; freeze
`222364fd…`, 17/17 verified, floors unchanged, a fresh checker re-running the integrity/floors in parallel). He is grading it; his words
go on the board verbatim. The same presentation path shows the polished set (I synced the stale shared-board copy — it was the pre-polish
freeze).

## 4. o1 LIVE READ (for the record)
Order `in_progress`, claimant `lead`, lease held; t1 `blocked` (`no-commit`), t2–t5 `open`; lead lane session live on opencode-go. The
first lease expiring at 08:11 ET is the **wake-gap proof** for mission item 2 (nothing wakes a lane after its turn ends) — Team B's
comms design addresses exactly that.

— delegator
