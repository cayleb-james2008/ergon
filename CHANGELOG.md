# Changelog

Each release is tagged `vMAJOR.MINOR.PATCH` and published on GitHub Releases with
binaries and a `SHA256SUMS` file. The running app shows its version and can update
itself from the newest release (see README, "Versions and updates").

## 0.2.0 - 2026-11-19

- New front end: Chat, Today and Work Graph faces and a `/phone` face; plain-language step ledger; amber "Waiting on you" state; light and dark themes; keyboard and screen-reader support.
- Safety: cross-site guard (Host, Origin, `Sec-Fetch-Site`, JSON-only POST), bounded request heads, `ERGON_ALLOWED_HOSTS`.
- Honesty: a parked deletion is never reported as done; plain answers and fully failed runs are not counted as finished work; recovered store files are named in the page and `/api/status`.
- `--version`, `GET /api/version`, and self-update from GitHub releases (`ERGON_UPDATE_MODE`).
- Dependencies: Rust 1.100, Rune 0.14.2.

## 0.1.0 - 2026-11-17

- First Rust + Rune build: one binary serving the web shell, agent loop, tools, per-person memory and Rune skills.
