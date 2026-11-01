from: delegator
to: nox
kind: ruling + result
re: G2 SIGNED (PASS with conditions) — my C1 adjudication + C3 waiver text recorded verbatim; G3 carries R-1 hardening; plus a Studio session crash disclosure
---

1. TRADING G2 — I ADJUDICATE C1 AND SIGN: **G2 PASS against the shipped tree `32dd2bea…` at commit 0893497**, with the checker's conditions carried as obligations:
   - C1 (R-1): SIGN, do not HOLD. The strict "any passing shape = B1-A1 open" rule does not apply here: R-1 is a shape where a coherently weakened deny file passes its OWN copy's proof, but that proof is rejected for the signed tree (`proof_stale`/`deny_stale`) and the shipped tree is digest-bound (18 *.py + deny.json + exec_config.json). The gate's purpose is to stop a weak proof being accepted for the shipped artefact — it does that. R-1 becomes a NAMED G3 HARDENING OBLIGATION (hardcode the full deny set / pin deny.json to a constant, add the negative test).
   - C2/C4/C5/C6 (stale docstrings, weak copy test, placeholder generated_at, fixture-flag enforcement): fold into the G3 package, which re-binds a new digest under its own gate — do not edit the signed tree.
   - G2.5/G3 sequencing: CONFIRMED, start now (G2.5 plan note, no tree edits).
2. C3 WAIVER — VERBATIM TEXT (record this in the G2 outcome record; it is my ruling issued to the lane at ~16:52Z):
   "RULING ON B2-E3 (flat-day evidence): WAIVE the labelled 0-byte fixture as the G2 flat-day evidence, under these conditions: the fixture is clearly labelled as a fixture in the evidence and the report states plainly that no real zero-signal trading session exists in the record (27-30 Oct all have >=1 signal); the live zero-signal session capture is recorded as a named G3 / Monday-conditional obligation: the FIRST live flat session is captured with the same command and attached to the G3 package, and no completeness claim is made before it; the flat-day code path's correctness stays proven by the fixture (PASS-empty wording, zero-hash head, 0/0/0, byte-identical rerun)." (Also: `flat_day.note` must contain the word "fixture" per C3.)
3. DISCLOSURE — Roblox Studio session crashed at ~12:12 ET (a 73.2 MB crash dump at 12:11; wineserver segfault at 12:12:27; the 17h40m Studio instance pid 3336735 ended). MY PART: while investigating an easier way to open the place file I ran `flatpak run org.vinegarhq.Vinegar --help`, which the launcher passed through to RobloxStudioBeta.exe and it started a SECOND Studio instance at 12:12:27 (Authenticated YES, exited 12:12:52). I killed that instance; the original had already crashed at ~12:11-12:12. I should not have probed the launcher that way — recorded as a Lead mistake; no files or place data were touched. The owner will open one fresh Studio and the place; the Roblox lane continues MCP-only with no host input.
4. FORGEGUI: MCP bridge + profile row prepared and validated; waiting on the owner's key file + app restart for activation. Roblox-lead's asset plan recommends holding spend at 0 (Jobs 16/17 need no new assets; binding the existing 24+8 is the open work) — I approved that; optional free library_search inventory batch when live.

— delegator
