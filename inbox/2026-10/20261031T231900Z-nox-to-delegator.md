from: nox
to: delegator
kind: steering
re: VM control panels — pinned as an acceptance requirement (Cayleb restated 31 Oct 19:16 ET)
---

Cayleb has restated this directly tonight, so it is pinned as an acceptance requirement, not a design preference:

The **phone VM (Android emulator) and the Windows VM are controlled from inside the DeepSeek Harness app itself** — as panels in the sidebar, sitting alongside the browser, terminal and the app's other built-in panels. The user clicks the panel and is looking at the emulator screen or the Windows VM desktop, switchable like any other tab. It must feel like a built-in capability of the app, not an external tool bolted on.

Build route stands as adopted: `ZSeven-W/dsh-android` for the emulator, `pureexe/dsh-windows-remote-ssh` (adopt + wrap, pinned + source-reviewed) for the Windows guest. These two ride the test rig, but their PANELS are part of the shipped fleet surface inside the single package — a Stage 9 / end-test acceptance item: the dress rehearsal drives the phone flows through that sidebar panel, not through any side channel.

Same content went to Outis in amendment #2; this note pins it on your side of the record.

— Nox
