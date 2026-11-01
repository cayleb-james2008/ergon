# Ergon dress rehearsal — evidence pack

Two full passes from scratch on the installed `@cayleb/dsh-fleet` platform (outis-desktop), driver
`docs/acceptance/demo-company/rehearsal/run-rehearsal.sh`. Tarball identity per pass is in each `pass-N/R0-pre/package-identity.json`.

| | pass 1 | pass 2 |
|---|---|---|
| Index | [pass-1/index.md](pass-1/index.md) | [pass-2/index.md](pass-2/index.md) |
| Index SHA-256 (sealed) | `716c9a1cadfe9e88d0455c685c2775ac82ea72ef16f9a64c57e49151f26a240e` | `f58f55beee5b0323ab91c4cee54746130cecc9c1cfa142e81689f6a7be59dca4` |
| Manifest SHA-256 | `bf939c9eee53bb837f0599dfd09247167902766234ff90ebbdd54b3f92e96166` | `135ccea8ef86bf130d2000528f6c4f5b3a3f6efa68201a691df131d7bdf9cb8f` |
| Seal-time verification | 72/72 | 74/74 |
| `sha256sum -c` (driver gate) | `manifest-check-pass-1.txt` | `manifest-check-pass-2.txt` |
| Fresh company root | separate mktemp per pass | separate mktemp per pass |
| Verdict tables identical | — | **yes** |

- Pass diff: [pass-diff.md](pass-diff.md); literal index-table diff: [verdict-table.diff](verdict-table.diff).
- Integrity: each pass was sealed last, its manifest verified in-process at seal time and then with an independent `sha256sum -c` (`manifest-check-pass-N.txt`); nothing is written inside a pass directory after its seal (`pack-commands.log` records the post-seal commands).
- Secrets scans: `secrets-scan-preseal-pass-N.txt` (pre-seal) and `secrets-scan-final-pass-N.txt` (post-seal, authoritative).
- A `pending` step is a PACKAGE-PENDING piece named in the index; it is counted, never recorded clean. The driver exits 0 only with zero `failed` steps, clean scans, sealed manifests that verify, and (for pass 2) identical verdict tables.
