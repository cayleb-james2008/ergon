# Ergon dress rehearsal — evidence pack

Two full passes from scratch on the installed `@cayleb/dsh-fleet` 0.2.2 platform (outis-desktop), driver
`docs/acceptance/demo-company/rehearsal/run-rehearsal.sh`.

| | pass 1 | pass 2 |
|---|---|---|
| Index | [pass-1/index.md](pass-1/index.md) | [pass-2/index.md](pass-2/index.md) |
| Index SHA-256 | `36b02f4a93a20268828bbcaca66245e7ba4b31d05e260b304476ae8faa8eb217` | `ecfbf1130ba2284b6e58bf703f2948588910026b738264a2b6e757306426410d` |
| Fresh company root | separate mktemp per pass | separate mktemp per pass |
| Verdict table | see index | see index |
| Verdict tables identical | — | **yes** |

- Pass diff: [pass-diff.md](pass-diff.md); literal index-table diff: [verdict-table.diff](verdict-table.diff).
- Second-pass secrets scans: `secrets-scan-final-pass-1.txt`, `secrets-scan-final-pass-2.txt`.
- The pass indexes carry the per-step verdict table, named pending pieces, check-by-check evidence, every command run, the artefact manifest and the checker sign-off block.
- A `pending` step is a PACKAGE-PENDING piece named in the index; it is counted, never recorded clean. The driver exits 0 only with zero `failed` steps, a clean secrets scan and (for pass 2) identical verdict tables.
