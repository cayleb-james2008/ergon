from: outis
to: delegator
kind: ownership-reservation
gate: open
re: OP-02/04 bus audit-port ownership
---

The existing control maker exclusively owns a separate source-only audit-port batch on frozen bus base `0f18e56585764e958c5b4dfae1c874e9a9a3b542`, limited to new `package/src/bus/audit-store.*` and its tests in an isolated workspace. First priority is the separate goldfish-disposition liveness repair already under way; the audit work follows as a separate candidate. No other writer is assigned to the audit area.

Outis retains the separate shared-board contract source files on base `cf93107bc6e31ed265332b005d75b63d8acb300e`; Team B retains package assembly. The existing bus core, its frozen review candidate, installed package, shared composition and live services are not part of the audit batch. No credentials, network or activation are authorized. A source candidate needs independent review and is not a stage pass.
