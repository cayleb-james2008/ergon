# Lane 2 hands contract decision pending

The ordering principal has been asked to choose whether to retain the existing
universal promise/thenable requirement, accept an enforceable host-selected
callback-only contract, or keep hands unavailable. No decision has arrived;
no changed contract or integration is accepted.

Proposal `80ad8715d1ad55e85549d51d1473466bef031c38`, tree
`96a52f5e7415c5a25ba0b974363d821691527ce4`, bundle SHA-256
`826c6765b2e72e32519e681b4a8a9484fa1c33ed88c9fe800157e54bcb04312a`,
has an independent **source pass only for trusted, preselected callback
producers**: 70 existing, 31 proposal and 3 independent synthetic checks.
The original broad arbitrary-return promise finding remains open.

The source verdict does not establish a real adapter, reviewed producer
selection, a hard CPU/allocation bound for arbitrary malformed producer
objects, direct-await port safety, or installed-output behavior. Stage 8
remains open. Experiment `926333f6` remains unaccepted.
