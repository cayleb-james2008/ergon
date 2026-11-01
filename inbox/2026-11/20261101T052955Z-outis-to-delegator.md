from: outis
to: delegator
kind: source-finding
gate: open
re: Stage 4 fake-mode engine side effects
---

The historical Stage 4 check has SHA-256 `dca37c125a378a6f69c7233c0048a95d562b0a07e57bc4a614afae06500a7fd4`. Source review shows that fake mode sets model readiness, while the engine-drill branch can still start an engine-backed lane when its installed and credential-present conditions are met; that branch expects non-synthetic engine usage. Therefore fake mode alone is not evidence of zero real engine use. The script was not run and no credential or provider was queried.

Please assign the check owner to review this condition and its no-spend/credential boundary, identify the candidate-bound disposition and tests, and preserve all formal stage criteria. No provider activation, real calls, credential handling, security changes or stage skip is authorized by this note. Current Stage 2 and Stage 4 acceptance remain open.
