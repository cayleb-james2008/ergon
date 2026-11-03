from: delegator
to: nox
kind: completion — 0.3.6 BOUGHT UP LIVE (owner-run, INSTALL PASS); independently verified read-only; two hygiene notes; next steps named
re: Nox standing go; finish-state/PICKUP.md; finish-docs/INSTALL-0.3.6-OWNER-RUN.md
---

## Bottom line
**0.3.6 is installed on the live fleet and healthy.** The owner ran the guarded install at 15:07:56 ET; the installer ended `INSTALL PASS: 0.3.6 is in place, the bus is mounted and a rubric verdict with criteria as text registers on the running host`. I independently re-verified it read-only (no writes, no restart, no token values printed). Nothing blocked.

## What I verified after the install (independent, read-only)
| check | result |
|---|---|
| unit | `dsh-fleet-consolidated` active; **new MainPID 383250** (was 3671044), start 15:07:56, NRestarts 0; old pid gone |
| version/files | `@cayleb/dsh-fleet 0.3.6`; **220 files = 147 pre + 73 added**; 0 pre files missing; 37 in-package files changed — all in the release list, **0 changed outside it**; 73 added — all in the release list; **all 111 release-listed files match their release hashes** (incl. `profile/cordis.patch.yml`); `validate.js` `b68ce88c6de1` |
| host-rooted artefacts | `rows.manifest.json` `e9364e57`; `presets/delegator.patch.yml` `8478e16a`; profile `cordis.patch.yml` `a4713a8d` — all equal the reviewed artefacts; 11 pinned / 12 disabled (installer self-check) |
| DB | `pragma integrity_check` ok; events **2256**; **hash chain verified with the installed package's own `verifyChain` against the journald anchor seq 2198 / `d9c9ba87…` → ok, contiguous to head 2256 `63de3fb3…`**; policy 2 rows (`roster.models`, `comms.grant`); roster 43 rows; orders 12 / tasks 30; **0 non-fixture non-terminal orders** |
| runner surface | 8790 (loopback + tailnet + `[::1]`), 3092, **8796 loopback-only (new, hosted-browser MCP; a plain GET returns 405, auth required)**; `control.sock`, `egress.sock`, `mcp-admin.sock` are **back** in `/run/user/1000/fleet` (the 13:21:05 anomaly is fixed by the restart) |
| timer/desk | `fleet-tmp-sweep.timer` enabled + active, `~/fleet-scratch/bin/tmp-sweep.sh` installed 15:07; OpenBell desk untouched (heartbeat running, flat, feed ws) |
| bus | `bus.who` works (`ext:delegator` fresh, `lead` idle); `bus.pull` empty; facade 15 tools / 6 `bus.*` and `bus.status` routed (installer self-check lines) |

## Two hygiene notes (no action taken; for you/Cayleb)
1. **The new 8796 listener is live and loopback-only, but it is still absent from the release notes and the protected-ports lists** (this was the CHECK-PKG MEDIUM M1; it is now a fact on the live host). Doc-only fix owed: add 8796 to the notes, the protected-ports lists and the next release note; consider a per-host `mcpPort` for scratch hosts.
2. **The DSH host prints its live UI token into its start log** (`$HOME/fleet/log/consolidated-<UTC>.out`, mode 0644) — pre-existing DSH behaviour, not caused by the install; the same line is in journald. I did not file the value anywhere. Recommend the finish lane/owner decide: redact that line pattern, `chmod 600` the log, or rotate by restart. Flagged, not touched.

## Next steps (from PICKUP; name the hand for the registration confirm)
1. Post-install health = the table above (done by me; the finish lane's own check may re-run).
2. **W3 live steps that need neither owner nor Nox go** — the finish lane owns the list; if it is finished (its workflow is COMPLETE and the completion report was filed at 15:03), name me the steps and I will run the ones that are read-only/non-privileged, or say they wait for G2.
3. **W9: `ext:rsi` registration + courier start** — needs a registration confirm; **whose hand?** (owner terminal vs Nox vs a lane). I will not improvise a confirm.
4. Completion-report update: the finish lane's report is filed; it should gain the install result (0.3.6 live) and the two hygiene notes above.

— delegator
