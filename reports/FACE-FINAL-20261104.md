# FACE final — the designed front end is the shipped front end, and an independent checker graded it

**Owner order (overnight):** the DSH fleet plugin is not finished for two named reasons — (1) the Claude-style front end was
designed but never merged into the shipped face, (2) verification owed: renders of the real app screens plus a separate fresh
checker that runs the built app and grades it against the standard (Claude desktop chat, Linear, Vercel; shadcn/MUI/Radix
family; flat colour, no gradients). Both are now closed on frozen bytes.

## 1. Merged, not branched
`main` @ **`4b20883`** (trunk `~/workspace/harness-build/dsh-fleet`); `face/merge-w6c` is the same commit and the same tree.
Five rows ship: **Fleet / Agents / Phone VM / Windows VM / Browser**. Agents is the designed activity page (fact strip,
needs-attention, notes, agents, queue, bus, recent events). Every panel reads the host's own connection
(`/api/fleet/state|events|activity|avd/*|monitor/*`), so no panel says "not connected" while the core answers. The rig panels
show the host's own words for an absent rig in a neutral notice; a real failure still gets the destructive tone.

Old face recoverable by measurement: `archive/main-pre-face` (`bd38123`) has `client.js` `9e7480f5…` byte-identical to the
installed 0.3.6.1 bundle. Nothing has been installed by an agent; the swap needs a Host restart, which is the owner's window.

## 2. The artefact the owner would install
`~/fleet-scratch/face/dist/cayleb-dsh-fleet-0.3.6.2.tgz`
sha256 **`b9a4dfce0a109e6d72b44ecffc317ccaa4fc59067d92ed7a5c94640e9328dd7a`**, 223 files.

- packs `client.js` `2474cbcb…9be283` = the committed blob at `4b20883`;
- `release/FILES-0.3.6.sha256` rides in the same commit as the bundle: a clean `git archive 4b20883` satisfies 17/17, and the
  pack satisfies it 17/17;
- `dist/SHA256SUMS.txt` describes the pack on disk (`sha256sum -c` OK);
- `install-0.3.6.sh --dry-run --target-live` from the trunk: must-keep PASS, package.json gate PASS, "0 already installed,
  17 to install", nothing written.

## 3. The independent verdict (fresh checker, its own runs)
`~/fleet-scratch/checker-r6-final-report.md`, on `b9a4dfce`:

| Claim | Verdict |
|---|---|
| 1. The shipped face is the designed face | **PASS** |
| 2. The design standard | **PASS** |
| The five defects from its previous round | **D1–D5 all closed, re-measured** |
| Open HIGH | **none** (the installer-list HIGH it raised was closed at `458731c` and attested in its §10) |
| Its own LOW (Browser panel had no green gate) | closed at `4b20883`, attested in §11 |

Gates on the frozen bytes (maker runs, corroborated by the checker's own): client **148/148**, watch **23/23**, face
**173/173**, tsc clean, panel matrix **70/70** (light and dark), real shell **110/110**, artefact install **116/116**, Agents
page states **60/60** (15 states × 2 viewports × 2 schemes). Fleet core unaffected: roster 217, core 109, orders 217,
browser 119, comms 134, policy 73, ledger 13, all pass.

## 4. What this pass added beyond the merge
- **A recurrence guard** (`client/test/panel-registration.test.mjs`): a designed panel id must be in `PANEL_REGISTRATIONS`
  (the list `apply()` loops over) or a named exception with its unblock list; the Browser half must reach the bundle; the
  shipped rows are exactly the five; every module the host imports must be inside `files[]`. A checker broke the first
  version (substring match) and the second version fails all four of its attacks — verified in a scratch copy.
- **Both-theme state evidence**: the activity harness never set `body[data-ds-dark-theme]`, so its "dark" shots were light and
  dark had no state coverage. Fixed; 60 renders, 0 problems, min contrast 4.63.
- **Copy that matches the product**: the Plugins section names all five panels and says they read this app's own host by
  default; the Fleet panel localises lane states ("Parked", not "parked").
- **A Browser defect no gate rendered**: the populated page squeezed itself into three columns inside the 720 px reading
  column; fixed at `9dae401`.
- **Release integrity**: three times in this workstream the tarball and the tree disagreed. The line is now frozen, the list
  rides the bundle's commit, and the pack, list and checksums name each other.

## 5. Honest limits
1. **Install is the owner's window** (Host restart) — `dist/INSTALL-OWNER-RUN.md`.
2. **No real rig on this machine**: Phone/Windows are graded on the truthful 503 path only; a real SHA-256 value and real AVD
   content are unproven by render.
3. **Browser's live path**: the populated page was measured with a synthetic `/api/fleet-browser` view plus a real engine PNG.
   Layout is measured; a real agent's page arriving through that route is still unproven.
4. **References 03 (Linear) and 04 (Vercel) are login-walled**, so no screen is called gate-PASS against the owner's full
   reference set; the family judgement rests on 01/02 plus measurements.
5. **Studios stays deliberately unshipped**, with its reason and unblock list in the guard. Wiring it is an owner decision.
6. **Two wording calls** for the owner: the rig notice repeats the absence, and `npm install` cannot install a four-part
   version (the reviewed file installer is the path).
