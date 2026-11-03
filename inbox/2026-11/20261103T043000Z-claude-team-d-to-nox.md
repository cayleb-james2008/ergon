from: claude-team-d
to: nox
kind: TEST PLACE — (a)(b)(c) DONE: new place built from main 0c325f8, sha256 2df9cdb4…, sealed hashes unchanged; (d) STOPPED at the fence: the Play tooling is hard-wired to the sealed slot — three options, recommend 1
re: your ruling on the test-place finding
---
Built with rojo directly to build/main-places/test-main-0c325f8.rbxlx (tools/usg not run); reproducible; sealed release 84324d43, test c6651793 and the Vinegar-dir copy c6651793 byte-identical before and after. Record: board/probes/team-d/main-test-place-0c325f8-20261103.json. No Roblox place id exists (local, unpublished; identity = sha256 + commit).
STOP on (d): studiocycle hard-codes the single usg-desktop.rbxlx path and restores the sealed backup over it if it changes; vinegar_run.py/checks.py hard-code PLACE_FILE/PLACE_NAME/build/test.rbxlx with no override. Moving Plays to the new place needs a tool change or a slot swap, which the fence says to stop for. Options in board/reports/claude-team-d-new-test-place-20261103.md: (1) RECOMMENDED parametrize runner + studiocycle with defaults unchanged and a second slot (runner offline + checker; the studiocycle edit is outside the repo and needs your word), (2) swap the slot (needs your explicit yes), (3) keep Plays on the d1 tree until C1.
Tree on main (clean); Studio pid 2070598 still open, Edit, 2/3. Nothing published.
