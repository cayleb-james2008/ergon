from: delegator
to: nox
kind: B1 chunk-transport fix ACCEPTED after three checker rounds; single retry running on the clean Studio
re: Roblox B1 lane (Jobs 16-17)
---

1. **B1 ROOT CAUSE (recorded):** Roblox's HttpService caps one POST body at **1024 KB**; the traffic runtime report is ~1.4 MB, so the collector never received the runtime phase and the fixture timed out at 155 s. **Not a product bug** — a harness transport limit. The diagnostic Plays are evidence, not retries of the cell.
2. **FIX — CHUNKED TRANSPORT, accepted.** Runtime envelopes now go through the collector's existing `/<nonce>/chunk` transport (256 KiB parts, in order, whole-envelope sha256, wire ≤600 KiB) for `traffic:pass`/`traffic:abort-contact` only; a new `ExactReportTransport` helper is in the acceptance HELPERS and hashed manifest; small reports keep the direct path (≤512 KiB), so a >1 MiB single POST is impossible by construction. Branch `prep/b1-capture-format-20261101`, accepted head **`c0d4b22`** (base `271b601`); sealed artefacts untouched throughout (`84324d43…`, `c6651793…`); no game code, no build, no re-pin.
3. **THREE CHECKER ROUNDS — the process worked:** round 1 REJECT (acceptance bookkeeping passed `helpers=2` where the real count is 3 → `cleaned=False` on misleading evidence; the byte-identity test used a no-op validator and could not catch a broken production path); round 2 REJECT-narrow (both blockers fixed, but the new fail-closed `cleaned` clause was load-bearing and unguarded — deleting it left the suite green); round 3 **ACCEPT** after a test-only fix (+5/−2) that guards the clause and imports the computed helper count. All rounds verified by fresh independent checkers with mutation proofs; every fix was test-only or bookkeeping, never a weakening of the claim.
4. **SINGLE RETRY RUNNING NOW** on a clean Studio (pid 327254, fitted window 2200×1000, place sealed, 0/3 Plays): standard resume → `traffic:abort-contact` → if pass, `atlas:scout` + `atlas:highway-extension` in the same opening. The atlas cells also carry the fitted-window Play confirmation (`viewportMatchesPng: true`). If the retry fails it is filed and stopped — no blind retry. Studio closes/reopens are now automated by `studiocycle` (the owner told me to stop asking him), with a sealed-hash guard and crash-recovery autosaves parked rather than deleted.
5. Fences unchanged: no spend, no credential values, no public posts/publication, no host input injection, no image reads in the DSH app; sealed artifacts recorded before/after every run.

— delegator
