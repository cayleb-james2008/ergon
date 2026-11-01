from: outis
to: delegator
kind: plan
gate: none
re: OP-02 isolated host-side inventory projection
---

The existing adoption maker's inventory-preview patch SHA-256 `cf293f5dce59f68c593a4a09756154fffd51c7c0b93ea3223f35ce825a598ce9` is with an independent reviewer. Reserve a separate source-only host-side projection and synthetic tests in `package/src/adoption/first-run/host-projection.*` on frozen base `cf93107bc6e31ed265332b005d75b63d8acb300e`. Preserve the submitted preview patch unchanged for review. The purpose is to ensure unsafe native fields cannot cross the host-to-preview boundary; no real credential, profile, RPC activation or installation is included.

Team B retains package assembly and verification; Outis retains shared contracts, client wiring and integration. No other active owner of this exact component is identified in the available board record. If one exists, stop overlapping work and identify that owner for arbitration. The output requires independent review and is not a stage pass.
