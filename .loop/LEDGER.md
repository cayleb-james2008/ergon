# Ledger — Ergon scale-up (append-only)

| row | first failing check + message | commit | outcome | cause / note |
|---|---|---|---|---|
| setup | baseline run: `verify[baseline]: build ok; tests ok; 4 passed; 0 failed` | 162dd57 | kept | starting point recorded before any change |
| B10 | new test sandbox_refuses_symlink_escape FAILED (read through symlink succeeded) | (see git log) | kept | sandbox_path was lexical only; added canonical-ancestor check inside the canonical root |
