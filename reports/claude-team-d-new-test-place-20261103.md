# TEAM D (Claude) — new test place from main: BUILT and recorded (a)(b)(c); (d) STOPPED at the fence — decision needed (2026-11-02 late)

## Done: (a) (b) (c)
- **(a)** Built a NEW place, never over a sealed one: `.tools/bin/rojo build test.project.json -o build/main-places/test-main-0c325f8.rbxlx` run directly from `main` (clean tree). `tools/usg` was not run; `build/test.rbxlx` and `build/release.rbxlx` were never written.
- **(b)** Recorded in `board/probes/team-d/main-test-place-0c325f8-20261103.json`: built from **main `0c325f83222986d235f53ca916afd8fd60ce8598`** ("Merge the B1 chain…"), place sha256 **`2df9cdb49bbf4970c10c86edd3943a51d4d00b659e7f9ada51e0f9bbd4c31946`**, 2,408,111 bytes, reproducible (a second build to a scratch path gave the identical hash), same structure as the sealed test place (92 ModuleScripts, 1 Script). **There is no Roblox "place id"**: the place is a local unpublished `.rbxlx` (a Roblox id would mean publishing); its identity is the sha256 plus the commit.
- **(c)** Sealed hashes checked before and after the build, byte-identical lists: `build/release.rbxlx` 84324d43…, `build/test.rbxlx` c6651793…, and the Vinegar-dir copy `usg-desktop.rbxlx` c6651793…. **UNCHANGED.**

## STOPPED: (d) "from then on native Plays run from the new place"
Your order says to STOP and report if the tooling cannot target a new place without touching the sealed one. The *build* can; the *Play machinery* cannot:
1. **`~/.local/bin/studiocycle`** (the fleet's open/close tool) hard-codes ONE place path, `…/data/usg/usg-desktop.rbxlx`, hashes it before and after every close, and **restores the sealed backup over it if it changed**. Putting the new bytes at that path would be undone by the next `close`/`cycle`, and re-baselining it (`studiocycle snapshot`) would overwrite the sealed backup. Editing it is a change to a shared tool outside this repo.
2. **`tools/vinegar_run.py` / `tools/vinegar_lib/checks.py`**: `PLACE_FILE`, `PLACE_NAME = "usg-desktop.rbxlx"` and `build/test.rbxlx` are constants (no flag or environment override); the runner selects the Studio by that name and compares the checkout's sources with the open place's scripts (that comparison is what correctly refused the Play on main).
3. Studio is open on the SEALED place now (you ruled it stays open); moving to the new place means a reopen.
I did not swap the slot file, edit `studiocycle`, launch Studio by hand, or otherwise route around this.

## Options for you (recommend 1)
1. **Parametrize, with the sealed slot untouched.** Add an explicit override to the runner (`USG_PLACE_FILE` / `USG_PLACE_NAME` / `USG_BUILD_PLACE`, default = today's values, so nothing changes unless set) and the same to `studiocycle`, with a second slot `…/data/usg/usg-main-0c325f8.rbxlx` (copy of the new place) and its own backup/hash guard. Runner change: offline, ~20 lines + tests, fresh checker before use; the `studiocycle` edit is outside the repo and needs your word. Then reopen onto the new place (the Studio reopen the cadence 3/3 needs anyway) and run the queue from there.
2. **Swap the slot** (archive the sealed copy first; the sealed bytes still live in `build/test.rbxlx` and the studiocycle backup, which I would copy aside, then re-baseline). Faster, but it removes the sealed copy from the slot every tool treats as sealed, so it needs your explicit yes and I would not do it unprompted.
3. **Keep running native Plays from the d1 tree** (sealed place matches it exactly; Plays 22-23 were run that way) until RC time, and build/seal the main place as part of C1 with the owner present.

## State
Working tree: `main` @ 0c325f8, clean. Consequence: `vinegar_run.py verify-clean` now reports "Source differs from the checkout" while Studio holds the sealed place (read-only check, fail-closed): to run any native Play before (d) is decided, check out `prep/d1-closing-16-72-20261102` again. Studio pid 2070598 still open in Edit (2/3). Nothing published, uploaded or pushed.
