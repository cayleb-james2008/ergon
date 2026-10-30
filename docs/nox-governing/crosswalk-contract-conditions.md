# Crosswalk — verdict and contract conditions

Source: Nox's Crosswalk MCP intake and evaluation, 30 Oct 2026 (`research_notes/crosswalk-mcp-intake-2026-10-30/report.md` on Nox's side).

## Verdict

**GO-WITH-HARD-CONDITIONS for low-sensitivity chore traffic only. NO-GO for anything sensitive.**

Why the ceiling: Crosswalk is a hosted MCP message board run by what appears to be a solo developer, weeks old, closed-source server. There is **no end-to-end encryption** — posts are stored as readable plaintext in the operator's cloud; public-crosswalk content is mirrored to a public GitHub repo; and there are no published terms, retention policy, or DPA. Anything sensitive (credentials, client data, trading/financial detail, personal information) must never cross it.

## Hard conditions (all of them, or no use)

1. **Low-sensitivity chore traffic only** — coordination, status, task handoffs. Nothing sensitive, ever.
2. **Private crosswalks only** — never post to public crosswalks (public content is world-readable and mirrored publicly).
3. **One key per agent** — a separate named API key for each agent, so access can be revoked per agent. **Read-only keys wherever a read-only key is enough** for that agent's role.
4. **The owner claims the account** — sign-in, account ownership, and key creation are the owner's acts (magic-link sign-in to his mailbox; keys handed to agents through the secure path, never pasted into chat).
5. **Publish approval ON** — posts require approval before publishing wherever that control is offered.

## Open question — the bus is NOT settled

The harness spec draft (`harness-spec-draft-2026-10-30.md` in this folder) recommends a **self-hosted board on our own machines as the single agent bus, with Crosswalk demoted** to an optional external chore channel. That recommendation conflicts with a Crosswalk-first reading of the verdict above, and the conflict is deliberate and unresolved: **the bus question is OPEN pending the goldfish spec pass, and neither answer is final.** Do not build on either assumption until the spec pass rules.
