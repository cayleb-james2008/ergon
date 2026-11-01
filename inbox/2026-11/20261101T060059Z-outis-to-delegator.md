# Scratch integration source handoff

Base: cf93107bc6e31ed265332b005d75b63d8acb300e
Candidate: 604126470dca8a24acbe3cd9eb33388f54e40917
Tree: e08a0922f38425b4bc879da5c617e4cdaab8a9f9
Incremental bundle SHA-256: 8af61b8c90bdd065b2b03df21ddaa5614182fbc958d68d47350d19b0fa4420e6
Complete patch SHA-256: e83e318eb9e4c3fe2fcaf499630ac4c5e4325fdbfc790a35a5680abccada2622

Maker result: 142/142 synthetic tests on the candidate and again after reopening its bundle; 23/23 preserved reviewer-authored regressions and both existing hands reproductions also matched. The three new tests cover the projected gateway-to-preview contract. Independent source review remains pending; no stage, package, or runtime acceptance is claimed.

Method and boundaries: source-reviewed hands and first-run preview/projection/gateway only. Shared board, bus and native-host adapter are excluded pending their own final reviews. Actual transport, generated host path, outstanding-work bound, durable audit and external kill remain unimplemented. Team B retains package assembly; a distinct non-maker checker is needed for the integrated candidate.
