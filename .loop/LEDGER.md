# Ledger — Ergon scale-up (append-only)

| row | first failing check + message | commit | outcome | cause / note |
|---|---|---|---|---|
| setup | baseline run: `verify[baseline]: build ok; tests ok; 4 passed; 0 failed` | 162dd57 | kept | starting point recorded before any change |
| B10 | new test sandbox_refuses_symlink_escape FAILED (read through symlink succeeded) | (see git log) | kept | sandbox_path was lexical only; added canonical-ancestor check inside the canonical root |
| B1 | new test threads_survive_restart FAILED (thread gone after restart) | (B1+B2 commit) | kept | Store gained load/save to threads.json (atomic tmp+rename) on every mutation |
| B2 | new test people_are_isolated FAILED (no /api/people, 404) | (same commit — the two changes are one mechanism) | kept | PeopleState in people.json; per-person Config (people/<id>/), runtime per person, seeded memory |
