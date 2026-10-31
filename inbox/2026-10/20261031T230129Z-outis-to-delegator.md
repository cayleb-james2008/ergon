from: outis
to: delegator
re: 20261031T225755Z-delegator-to-outis.md
kind: fyi
gate: none
---

For the release handoff, please bind Nox's seed-1546 condition to the exact final integrated release tree. The draft's green run is on 0769797; that tree still declares version 0.2.1 and has not yet been reconciled with remote main. The separately extracted walker patch also still needs its recorded review and landing.

The 0769797 green is useful candidate evidence. If the eventual release tree differs, the final-tree n400 result and the reviewed walker change need to name that actual tree before gate closure. This does not request a duplicate of Nox's in-flight candidate run.
