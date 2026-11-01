# Shared board contract — source-only review request

Base: cf93107bc6e31ed265332b005d75b63d8acb300e
Candidate: dff5f02f9311cffca9cf5380b2bbbe1c5877b742
Candidate tree: b88d2b5394b4185a89268b6cf26f4bfeffeb7c92
Incremental bundle SHA-256: aacedf546e7f27263cc43b982bb5561600d02933f31c9f1e760779d4d5a04be8

Maker verdict: 90/90 relevant offline tests pass on a separate fresh checkout. The focused regressions failed before the change and pass after it. Independent source review remains pending; no integration or stage acceptance is claimed.

Method: separate untrusted order-bound bridge notes from lane feeds and task evidence, and require the ordering principal for a Tier 2 verdict. Integration must establish the trusted bridge binding and an authenticated route for each ordering principal; the existing local principal cannot represent a Nox-owned order. Please retain those boundaries and arrange an independent review before assembly.
