# Ergon dress rehearsal — evidence pack

Two full passes from scratch on the installed `@cayleb/dsh-fleet` platform (outis-desktop), driver
`docs/acceptance/demo-company/rehearsal/run-rehearsal.sh`. Tarball identity per pass is in each `pass-N/R0-pre/package-identity.json`.

| | pass 1 | pass 2 |
|---|---|---|
| Index | [pass-1/index.md](pass-1/index.md) | [pass-2/index.md](pass-2/index.md) |
| Index SHA-256 (sealed) | `12b60cd35b031e949c7984170ed13fdb6bd195b7730519942accd0669ad6082f` | `876ba3b860c744ab258effa13d757c58b729ae534c3c59011e5c5ec04384f003` |
| Manifest SHA-256 | `a17a337315cc6dfd1f1eae80dc8c51d0a5aa2a14418f9d4a11ab7d0fa485fc65` | `04ffeaf1255e504b6104c74f6999ed038e618183ceae09bb47a159de089be1f6` |
| Seal-time verification | 72/72 | 74/74 |
| `sha256sum -c` (driver gate) | `manifest-check-pass-1.txt` | `manifest-check-pass-2.txt` |
| Fresh company root | separate mktemp per pass | separate mktemp per pass |
| Verdict tables identical | — | **yes** |

- Pass diff: [pass-diff.md](pass-diff.md); literal index-table diff: [verdict-table.diff](verdict-table.diff).
- Integrity: each pass was sealed last, its manifest verified in-process at seal time and then with an independent `sha256sum -c` (`manifest-check-pass-N.txt`); nothing is written inside a pass directory after its seal (`pack-commands.log` records the post-seal commands).
- Secrets scans: `secrets-scan-preseal-pass-N.txt` (pre-seal) and `secrets-scan-final-pass-N.txt` (post-seal, authoritative).
- A `pending` step is a PACKAGE-PENDING piece named in the index; it is counted, never recorded clean. The driver exits 0 only with zero `failed` steps, clean scans, sealed manifests that verify, and (for pass 2) identical verdict tables.
