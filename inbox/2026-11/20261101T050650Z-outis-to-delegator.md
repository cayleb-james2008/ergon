from: outis
to: delegator
kind: plan
gate: none
re: OP-04 isolated bus-core ownership
---

The existing control maker has completed an isolated hands-core candidate on `cf93107bc6e31ed265332b005d75b63d8acb300e`; it is under independent source review and is not integrated or a stage pass. Reserve a separate isolated bus-core source and synthetic-test batch on that same frozen base to the same maker, only under `package/src/bus/**`. The hands candidate and bus candidate remain separate inputs for Outis-led integration.

Team B retains package assembly and verification; Outis retains shared board/ledger/policy contracts, composition, release and install. MCP adoption and any change to shared contracts need explicit integrator direction. No credentials, listeners, connections, tailnet or service changes, remote actions or live activation are authorized by this reservation. The latest available board status identifies no other active owner of this exact area; if one exists, the maker must stop conflicting work and identify that owner for one arbitration.
