# plugin-finish (seat A) → Nox: the owner's exact command, run from the published tree, end to end on a faithful copy

**2026-11-04T10:12Z.** Addendum to my `100837Z` verification. Read-only on the live fleet: the target was a **full copy** of
`profiles/fleet` (2.6 MB, copied `node_modules/@cayleb/dsh-fleet` and `cordis.patch.yml`), and every command was
`--profile-dir`-pointed at that copy.

## Why this addition

My first check from `~/workspace/harness-build/dsh-fleet` used a **thin** fixture (three roster files) and the published
installer answered `the plan does not hold; nothing was written`. That refusal is the published installer's **import-closure
guard** doing its job: the payload's roster files import `kit.js`, `table.js`, `tree.js` and `../core/*`, which a three-file
fixture does not have. **On a faithful copy the same command says `PATCH PLAN: OK`.** I am writing this down because a checker who
reproduces my first fixture would conclude the published installer is broken when it is stricter than mine, by design.

## The sequence, from the published tree (`dsh-fleet` @ `9a6b6cc`)

| step | result |
|---|---|
| `--check --profile-dir <copy>` | `release payload: …/release/payload-0.3.6.2-lane-heal`, `PATCH PLAN: OK`, three `will patch` lines, `BEFORE` = the live 0.3.6.1 bytes |
| `--apply --profile-dir <copy>` | `patched` ×3 → `637cc359de4e / 78d8b4bea0a9 / 1634746525de`, `package.json version now 0.3.6.2`, `APPLIED.`, `.lane-heal-applied.json` written — i.e. P3's precondition |
| `--finish --profile-dir <copy>` | warned with the **real** Host (`pid 2313700, started 03:46:14Z` precedes `applied_at`), then `bridge row removed: 1`, module removed, `FINISHED.`, bridge row count **0**, and the config diff vs the backup has **0 added lines** |

So the command in `MORNING-READ-FIRST.md` (run from `~/workspace/harness-build/dsh-fleet`) is exercised on the artefact the owner
will actually get, and its failure modes are the ones already documented (relaunch first; bridge row is idempotent).

One consequence worth stating for the owner: the published installer and my branch's installer are **not** the same file — the
published one takes its payload from `release/payload-0.3.6.2-lane-heal/` instead of `package/`, so a later merge cannot change
what gets installed. Both produce byte-identical roster files (`BEFORE`/`AFTER` hashes above), and installing from either is safe.
