# FACE — the designed front end is now merged into the trunk (main)

Workstream: DSH FACE (overnight). Model `opencode-go/deepseek-v4.1-flash`. Repo `~/workspace/harness-build` (harness plugin
`@cayleb/dsh-fleet`). At 2026-11-04T07:0xZ.

## Why this filing exists
The owner's ruling was that the Claude-style front end "was designed but never merged into the shipped plugin face". At 06:5x
that was still literally true one level up: the finished face lived only on `face/merge-w6c`, `main` was nine docs-only
commits away, and the installed package was still 0.3.6.1. This pass closed that: the face is on the trunk.

## What changed (repository state, nothing pushed to the plugin remote)
| Ref | Commit | Meaning |
|---|---|---|
| `main` | **`2ecc08f`** | the trunk, now carrying the designed face (`Merge branch 'main' into face/merge-w6c`) |
| `face/merge-w6c` | `2ecc08f` | same commit (source worktree `~/workspace/harness-build/fin-FACE`) |
| `archive/main-pre-face` | `bd38123` | the pre-face trunk, kept so the old face is recoverable |
| installed package | 0.3.6.1 | untouched; the swap needs a Host restart = owner window |

The merge was clean: main's delta was 5 files, all docs (README + `docs/GOAL.md` + three design docs, 179 insertions, zero
code). No conflict, no code from main to reconcile.

## Proof the merge moved no shipped byte
- clean `node build.mjs` at `2ecc08f` reproduces `src/client/client.js` **byte for byte**
  (sha256 `5fbc190da2733bb39d00dfb5657200cd4d3054034bb08849a93acbfbc260a58d`);
- `npm pack` from the trunk produces the **same** tarball sha256 `41c1192fda8cd8f016b0d4eb6eda622602e8b040edfab997fbcd8d7cf5baec74`
  (223 files), so every render/install measurement taken before the merge still describes these bytes;
- `release/install-0.3.6.sh --dry-run --target-live` run **from the trunk worktree** (`~/workspace/harness-build/dsh-fleet`):
  must-keep gate PASS, `package.json` gate PASS, "0 already installed, 17 to install", nothing written, no service touched.

## State of the claims
1. **Designed face is the shipped face — implemented, independently graded twice, pending one fresh verdict on the trunk.**
   Five rows (Fleet / Agents / Phone VM / Windows VM / Browser), Agents is the designed activity page, every panel reads the
   host's own routes, no panel says "not connected" while `/api/fleet/state` answers 200. `LiveFleetSource` defaults to
   same-origin; a rig that is genuinely absent renders a neutral "Phone rig not available" notice in the host's own words.
2. **Design standard — PASS / PASS-WITH-NOTES on the previous two independent checks** (r1: PASS-WITH-LIMITS; r3: Claim 1 PASS,
   Claim 2 PASS-WITH-NOTES, its one MEDIUM closed). Flat colour only, no gradient/shadow/backdrop-blur on any plugin surface,
   one type scale / radius / gutter, 44 px targets, face follows the shell theme in light and dark (measured painted
   backgrounds, not asserted).

## Gates on these bytes (maker runs, this tree)
client **142/142**, watch **23/23**, `src/face/**` **173/173** (dir form 64), `tsc` clean, panel state matrix **70/70** renders
(light and dark), real shell **110/110**, artefact install **116/116** with 17/17 installer files byte-identical inside the
tarball, Agents page **30/30** states. Fleet-core suites unchanged: no new failure against the recorded baseline.

## Honest limits
- The r4 checker completed every gate green (`fleet-scratch/checker-r4/logs/`: suites `fail 0`, tsc `EXIT=0`, matrix
  `70 renders, 0 problems`, shell and installed `EXIT=0`) and then its session ended before writing a report. Those are another
  session's logs, so they are corroboration, not a verdict. **r5** (fresh, pinned deepseek-v4.1-flash) is grading trunk
  `2ecc08f` + the same tarball; report → `~/fleet-scratch/checker-r5-report.md`.
- Reference shots 03 (Linear) and 04 (Vercel) remain login-walled, so no screen can be called gate-PASS against the owner's
  full reference set — pre-existing, recorded by W6A/W6C.
- Install is still the owner's window; the reviewed file installer is the path (`npm install` refuses the four-part version).
- Open owner calls, not changed silently: Browser column 64 rem vs 45 rem elsewhere; `Run round-trip` stays enabled while the
  rig is absent; raw lane-state enums on the Fleet panel vs localised words on Agents.

## Where the evidence is
`~/fleet-scratch/face/EVIDENCE-INDEX.md` (review order), `~/fleet-scratch/face/dist/INSTALL-OWNER-RUN.md` (owner commands,
rollback), `~/fleet-scratch/overnight/face-progress.md` (full log), shots in `~/fleet-scratch/face/shots-{r4,installed-r4,
panels-r4,activity-states-r4}/`, progress checkpoint `~/fleet-scratch/overnight/face-NEXT.json`.
