# Ergon Front End — v0.3.0 Design & Verification

## 1. The Job (`JOB.md`)

Ergon is a personal employee agent desk where each person states digital work in plain words, the agent finishes it end to end with sandboxed workspace tools and Rune skills, and every saved file and consequential hold is reported honestly.

First-screen questions answered within five seconds:
1. Is my agent ready, and is anything waiting on my approval right now?
2. What work did my agent finish recently, how are recent runs trending, and which workspace files were saved?
3. Where do I start a new task or return to an earlier conversation?
4. What skills, repeatable apps, and personal preferences has my agent learned for me?

## 2. Screen Architecture

### Persistent Desk Shell (`App.tsx`)
- **Left Rail (`256px` desktop, `Vaul` left drawer `<768px`)**:
  - Ergon wordmark and calm active-person desk card (`Working for` + `PersonSelect` over `Popover` + `Command`, keeping `/api/people` switching in the rail while keeping the top bar clean per single-person-session ergonomics).
  - Primary `New chat` action (`COSS Button`).
  - `/` conversation filter (`COSS InputGroup` + `Input` + `Kbd` `/` hint) with live client-side filtering and clear button.
  - Conversation list (`#thread-list`) with `Motion Primitives AnimatedBackground` gliding selection highlight, state glyph (finished check / waiting hourglass), relative time, and message count in `Geist Mono`.
  - Footer capabilities & preferences: `What my agent has learned` (`MorphingDialog` + count chip), `My suite and the library` (`MorphingDialog` + count chip + one-tap app runner), `Add a person` (`MorphingDialog`), `Light / Dark` `SegmentedControl`, and `VersionRow` (`Ergon 0.3.0` + `Check for updates`).
- **Top Navigation & Command Bar**:
  - `COSS UI Tabs` (`Chat · Today · Work Graph`) with gliding active indicator and live pending count `Pill` on `Today`.
  - Global keyboard shortcuts: `/` outside inputs focuses the conversation filter; `⌘K` / `Ctrl+K` jumps straight to the `Chat` composer from any view.
  - Honesty `fixture demo` `Pill` whenever running on standalone static demo data (`window.__ERGON_DEMO__`), plus `Agent ready` / `Not connected to a model yet` `Kibo UI Pill`.
- **Update Strip (`shell/Update.tsx`)**:
  - Follows `BANNER-FIX-BRIEF.md`: plain human error copy (`plainUpdateError` without raw GitHub URLs), single staged restart sentence (`Ergon <v> is installed. Restart to use it.`), loopback-only `Update now` / `Restart now`, and `Dismiss` -> `Show update` focus handoff.

### Today's Desk (`views/Paper.tsx`)
- **Morning Brief Header**: dateline, `Ask your agent for something` action, and headline with `Motion Primitives SlidingNumber` counters (`3 things finished, 1 waits on you.`).
- **Operational Outcome Strip (`Tremor Tracker`)**: 14-slot run outcome `Tracker` (sage finished, amber waiting on you, red stopped early, warm track open slots) with `HoverCard` run inspection and `Geist Mono` counters (`finished`, `waiting`, `files saved`).
- **`Waiting on you` Approval Queue**: plain-language consequence explanation (`Approving deletes the file for good. Keeping it leaves the file where it is...`), `Approve` and `Keep the file` / `Decline` keys, FLIP layout transitions, and `Recently settled` `Cult UI Expandable` history.
- **`Finished` & `Did not finish` Ledgers**: workspace file pills with per-extension `FileGlyph` icons (`spreadsheet`, `markdown`, `code`, `text`, `file`) and `Open` thread jump.
- **`Today so far` (`/api/today`)**: chronological operational stream of today's completed runs, raised/settled decisions, and capability offers with file glyphs.
- **Side Column**: `Offers from your agent` (`Make it one tap` / `No, thanks`), `What your agent has learned` (`standing_preferences`), and `Team` workspace switcher.

### Chat & Field-Book Entries (`chat/Chat.tsx`, `chat/Entry.tsx`)
- **Empty State Starter Task Cards**: role-tailored rich cards (never plain text boxes) pairing category `Badge`, Rune skill/subagent `Pill` (`Bookkeeper`, `Meeting actions`, `Write document`, `Researcher`), prompt title, outcome description, and `Fill task box` footer, above `Finished recently`.
- **Live Active Run Ledger**: `Motion Primitives BorderTrail` + `TextShimmer` (`Agent is working on it`) while tools execute.
- **Field-Book Result Cards**: dark/deep-step ink header with `Geist Mono` timestamp, ruled metadata cells (`Task`, `Saved to` with `FileGlyph` chips, `Steps` count in `Geist Mono`, `Finished` `Pill`), plain result prose, and `Cult UI Expandable` (`Inspect tool steps (N)`) revealing verbatim tool calls and workspace file paths.
- **Parked Deletion Card (`WaitingEntry`)**: amber `Waiting on you` `Pill` and `Review in Today` action when `delete_file` parks a decision.

### Work Graph & Unified Responsive `/phone` (`views/Graph.tsx`)
- **Survey Map + List Toggle**: four-band deterministic SVG (`People -> Suite items -> Finished work -> Files`) drawn at 1:1 CSS pixels via `ResizeObserver` so every SVG `<text>` label stays `>= 13px` across `768px..1440px` with zero overlap, paired with a `Map & list` / `List only` `SegmentedControl` and accessible grouped list (`#graph-list`).
- **Unified `/phone`**: `vite.config.ts` writes the single-file bundle to both `assets/index.html` and `assets/phone.html`, giving `390x844`, `320x640`, and `/phone` identical responsive ergonomics and `>= 44px` touch targets.

## 3. Typography, Palette & Role Coverage

- **Self-hosted Typography (`src/fonts/`)**: `Geist` (`400`, `500`, `600`) for UI & headings and `Geist Mono` (`400`, tabular numerals) for counts, timestamps, step counts, and filenames. Zero external font requests.
- **Light-First Furrow Palette + Full Dark Theme (`src/theme.css`)**:
  - Light: `--canvas: #f5f4ee`, `--surface: #fdfcfa`, `--board: #edebe2`, `--hairline: #e3e1d6`, `--ink: #141413`, `--ink-2: #4a4843`, `--ink-3: #66635b`, `--blue: #d97757` (clay accent).
  - Dark: `--canvas: #141413`, `--surface: #1f1e1c`, `--board: #282724`, `--hairline: #33322f`, `--ink: #f5f4ee`, `--ink-2: #c2c0b6`, `--ink-3: #96938a`, `--blue: #d97757`.
- **Library Role Coverage (`ROLE-COVERAGE.md`)**: all 8 House Frontend libraries (`Motion Primitives`, `COSS UI`, `Cult UI`, `Kibo UI`, `Tremor`, `Vaul`, `Sonner`, `Motion`) serve genuine operational needs (`0` "no need here" rows).
- **Motion Contract (`src/motion.ts` & `ALIVE.md`)**: every duration and easing maps to `src/tokens.css`, verified by `motion-source.mjs` and `prefers-reduced-motion: reduce`.

## 4. Verification Summary (v0.3.0)

- `cargo test --offline`: `101/101` passed (`17` unit + `84` E2E integration).
- `cargo clippy --all-targets -- -D warnings`: `0` warnings.
- `npm run check:ui`: `gate` (`52/52`), `probe-graph` (`6/6`), `replay` (`20/20`).
- `hallmark/scripts/sloplint.mjs --scope app --render`: `0 FAIL`.
- `ultra-design/kit/probe.mjs`: `PASS` across `1440, 1100, 900, 768` in Light & Dark (`34` SVG labels, min `13px`, `0` overlap, `0` horizontal overflow).
- `ultra-design/kit/replay.mjs` (`frontend/e2e/cases.mjs`): `80/80` passed (all 15 first-load `/api/*` endpoints swept for malformed `200` JSON and unreachable server + worst-case null/empty/long-token payloads at `1440px` and `390px`).
