# Ergon dress rehearsal — evidence pack

Two full passes from scratch on the installed `@cayleb/dsh-fleet` platform (outis-desktop), driver
`docs/acceptance/demo-company/rehearsal/run-rehearsal.sh`. Tarball identity per pass is in each `pass-N/R0-pre/package-identity.json`.

| | pass 1 | pass 2 |
|---|---|---|
| Index | [pass-1/index.md](pass-1/index.md) | [pass-2/index.md](pass-2/index.md) |
| Index SHA-256 (sealed) | `b10d11bc343a9151aac2e302e252b0355bd5424065c91a2998d1be4950c338d4` | `4e2a1852a083a59ec9cfceed49e20fdd3b52caf3a15777fe49a6e4b4b82c7a58` |
| Manifest SHA-256 | `75262e528d3c639dc604d3438511a7bca7a237f05db90f1202005e81431db3d3` | `583293af72ddad37af5c878e6fe6cb12216f25747216cef1615b8e2623b088ef` |
| Seal-time verification | 72/72 | 74/74 |
| `sha256sum -c` (driver gate) | `manifest-check-pass-1.txt` | `manifest-check-pass-2.txt` |
| Fresh company root | separate mktemp per pass | separate mktemp per pass |
| Verdict tables identical | — | **yes** |

- Pass diff: [pass-diff.md](pass-diff.md); literal index-table diff: [verdict-table.diff](verdict-table.diff).
- Integrity: each pass was sealed last, its manifest verified in-process at seal time and then with an independent `sha256sum -c` (`manifest-check-pass-N.txt`); nothing is written inside a pass directory after its seal (`pack-commands.log` records the post-seal commands).
- Secrets scans: `secrets-scan-preseal-pass-N.txt` (pre-seal) and `secrets-scan-final-pass-N.txt` (post-seal, authoritative).
- A `pending` step is a PACKAGE-PENDING piece named in the index; it is counted, never recorded clean. The driver exits 0 only with zero `failed` steps, clean scans, sealed manifests that verify, and (for pass 2) identical verdict tables.
