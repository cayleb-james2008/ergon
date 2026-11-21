# Ergon UI checks

`npm run check:ui`, run from `frontend/`, builds the frontend and the binary. It then starts Ergon on free ports against the seeded data in `fixtures/data` and a stub model (`fixtures/stub-model.mjs` replays `fixtures/replies.json`), and runs three scripts, each on fresh data:

- `gate.mjs`: the Morning paper gate, 41 checks. It measures overflow, target size, accessible names, contrast and font size at 1440, 768, 390 and 320 px in light and dark. It then covers every interaction, the loading, empty and error states, console errors and reduced motion.
- `probe-graph.mjs`: Work graph labels at 1440, 1100, 900 and 768 px, 5 checks.
- `replay.mjs`: worst-case data fed in through `page.route` mocks, 16 checks.

The seed's timestamps are moved up to the moment of the run, so the brief's 48-hour window always holds them. The check fails unless every check passes and each script ran its full count. It stops everything it started, also when interrupted.

Setup, once:

- `npm ci` in `frontend/`. Playwright is the frontend's own `playwright-core` devDependency.
- A Rust toolchain with this crate's dependencies already fetched (`cargo build --offline` must work).
- Chrome for Testing 151 at `~/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome` (what `npx playwright@1.62.0 install chromium` puts there). To use another Chromium build, set `E2E_CHROME` to its path.

Flags, after `--` (`npm run check:ui -- --skip-build`):

- `--skip-build` reuses the last build.
- `--shots` adds screenshots at 1440 and 390, light and dark, on fresh data.
- `--lints` adds the taste, impeccable and Hallmark lints. They only report and never fail the check. The tools live outside this repo; `lints.sh` lists their default paths and the env vars that override them.

Screenshots, `result.json` files, logs and the run's data go to `e2e/.out/`, which git ignores. To bring the fixture up by hand, run `node e2e/fixture.mjs start` (it prints the URL); stop it with `node e2e/fixture.mjs stop`.
