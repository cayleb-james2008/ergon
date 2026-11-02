# Phase A · Mission item 4 — Owner Elevation: design adoption + consolidated-host gap report

**Status:** design only; no privileged command run, nothing installed, no password touched. **Base design (adopted):**
`~/workspace/harness-build/finish-docs/leg4-owner-elevation-design.md` (Claude-finish draft, 2026-11-02; threat model, A+/B/C comparison, live host
facts, decisions D1–D3). This report adds what the mission and today's cutover stop require: the live **Tier-B gap verified on the consolidated
host**, the deltas for the one-setup world, the helper spec split (Claude lane builds it), and the Lane 6 gate inputs.

## 1. Verified on the consolidated setup today (09:09 ET)

| Fact | Evidence | Consequence |
|---|---|---|
| `/usr/local/bin/dsh-fleet-confirm` is root-owned and **hardcodes** `d=/var/lib/fleet/state/confirms`; it requires `id -un = fleet` and `PKEXEC_UID = 1000`, and is reached only as `pkexec --user fleet …` | file read (1212 B) | the helper writes the **unit-world** confirm dir |
| Consolidated `fleet-policy.confirmDir` = `<desktop-home>/fleet/state/confirms` (schema default `/var/lib/fleet/state/confirms`; the F2 overlay re-roots it) | `src/policy/index.js:28-29`; overlay; live dir exists mode 700 owner cayleb, **empty** | a Tier-B ticket confirm **never lands** in the profile that is actually running → any Tier-B action is refused for ever |
| Unit `fleet-dsh` is **inactive**; `/var/lib/fleet` still exists as evidence; the `fleet` OS user still owns it | `systemctl`, `/var/lib/fleet` perms | the helper's `--user fleet` path still "works" mechanically but feeds a dead directory; no fallback to the live confirmDir exists |
| No graphical polkit agent is running; pkexec prompts in a terminal | leg4 §2 [verified] | "one-tap" cannot be a stock polkit dialog; it needs our own owner-entry surface or an installed agent |
| Confirms are accepted by `processConfirms` from a regular file owned by the host uid whose JSON `pkexec_uid` equals `peerUid`; all lanes are uid 1000 | `policy/policy.js:253-296`; leg4 §12 finding 1 | **in the consolidated install any lane can forge a Tier-B confirm file** — a real finding, not caused by Owner Elevation |
| Tickets: forward-only states, single-use grants, 10-minute expiry | `policy/tickets.js` | good base for the elevation request record; elevation requests should reuse it rather than invent a second state machine |

**This is exactly today's cutover-stop class** (Lead's note): an agent needed an owner act at a step, and the owner-authorised path could not
reach the live profile. Owner Elevation is the proper closure; the Tier-B re-root is the *interim* symptom.

## 2. Design (deltas on leg4 for the one-setup world)

**Recommendation stands: design A+** — a user-level secure-entry helper that feeds `sudo -S -k` once, with its *code* installed as root-owned
static files (no daemon, no setuid, no sudoers change). Authority is the owner's password typed at the machine into a prompt that shows the exact
argv the same process will execute. Everything else (requests, rows, buttons, phone notices) is routing/record only.

Deltas required by the consolidated setup + the mission's item 4:

1. **Request record in the fleet DB**, not a confirm directory: `elevation_requests` (id, requester principal, exact argv, cwd, reason, one-shot
   nonce, state `requested → shown → granted|denied|expired → consumed|failed`, `expires_at` 10 min, audit rows). A file-drop hop is exactly the
   mechanism that broke; the helper reads the request by one-shot nonce from a root-owned spool the live host writes to, and the host reads the
   outcome (or the helper returns it directly when the owner is at the machine).
2. **Helper paths parameterised by a root-owned config** (`/etc/dsh-fleet/owner-elevation.json`, 0644 root): target profile home, confirm/request
   spool, audit log path. No hardcoded `/var/lib/fleet`, no per-profile patch of a root-owned file by an agent. Owner installs static files once;
   the *config* is an owner act too.
3. **Owner entry surfaces, honestly split:**
   - at the machine: a GTK prompt (GTK4 present) showing argv in plain language, the requesting lane, and the one-shot; password feeds
     `sudo -S -k` once, then drops.
   - away: the plugin pushes a **notification** through the Duet/transport channel and the owner can **deny** or **arm** a request there; the
     **password is never typed remotely**, never travels through Nox/the board/logs. This departs from leg4's item 2–3 wording of Nox's order and
     is flagged as a decision, not smoothed (leg4 §9 already records it).
4. **Audit**: append-only `elevation.*` events in the fleet DB + journald copy: request (argv sha256 + plain argv), grant/deny (who/where),
   execution/outcome, expiry; never the password. Chain head exported like the rest.
5. **Brief takeover**: a quiet window — the owner acts on his own live desktop; lanes lose GUI-driving tools until he hands back or a timer ends;
   takeover start/end is audited and shows in the face. No force-close of owner sessions/apps ever (care fence).
6. **Hard rails**: per-command only (no wildcard, no shell string, no `sudo -i`, no repeated grant); one-shot consumption tied to the nonce; deny by
   default; short expiry; helper refuses if the asked argv differs from what was shown (hash recompute); Lane 6 hard gate before build; the
   privileged helper is **built by a Claude lane** (harness-sensitive, not executed by this in-harness session); Team B specifies, integrates and
   checks.

## 3. Gap report (named)

| ID | Gap | Severity | Close |
|---|---|---|---|
| B4 | Tier-B confirm target mismatch in the consolidated setup (helper → `/var/lib/fleet/state/confirms`; live policy → `<home>/fleet/state/confirms`, empty) | critical (today's blocker class) | Owner Elevation §2.1/2.2 replaces the hop; interim: a root-owned config + helper reading the live request spool — **only** as part of this gated build, never an ad-hoc agent patch |
| G-C1 | Any uid-1000 lane can forge a Tier-B confirm file in the consolidated install | critical (security) | elevation request nonce discipline + one-shot + Lane 6 review; record as finding |
| G-C2 | No graphical polkit agent; no "one-tap" stock path | high | own GTK prompt surface (A+) |
| G-C3 | Password-in-memory exposure window: helper must be non-dumpable and socket-fed; same-uid pipe re-open is proven possible | high | leg4 §11 controls; helper tests (fake sudo) |
| G-C4 | Phone/away approval: mission wants "one-tap"; security requires the password typed at the machine | high | notify/deny/arm only; Nox rules on the wording (leg4 §9) |
| G-C5 | Privileged helper code must be root-owned static; any lane-writable helper = password phishing (leg4 §4) | critical | owner one-time install of static files; hash pin displayed to owner |
| G-C6 | Elevation audit must not be forgeable by uid-1000 lanes (events chain has no secret) | medium | root-owned helper writes its own audit line to a root-owned file; host copies it read-only into the fleet chain |
| G-C7 | Brief takeover has no implementation: no tool-gating switch for lanes, no timer, no handback | medium | plugin policy flag + face control + audit; DSH tool-gating hook verified first |

**Owner acts needed (named now, not assumed):** (1) one-time root install of the helper static files + root-owned config; (2) the real elevation
run later (his password, at the machine); (3) ruling on the away-approval wording. No privileged command is run by any lane in this mission.

## 4. Build plan (Claude lane for the helper; Team B for the plugin side)

1. This spec + leg4 threat model → **Lane 6 hard gate** (route through Lead → Nox). No build before the gate.
2. Claude lane: root-owned helper (argv hash, one-shot nonce, config, audit, GTK prompt, sudo `-S -k`), fake-sudo test harness, no daemon/setuid.
3. Team B: `elevation_requests` store + state machine + audit events + policy ops (`elevation-request`, `elevation-status`, `elevation-cancel`) +
   face panel ("needs you" list, argv plain language, countdown) + takeover flag/audit.
4. Fresh checker: fake-helper end-to-end (request → prompt → grant → one execution → consumed; deny; expiry; argv-mismatch refusal; forged-file
   refusal); Lane 6 sign-off; then the one real elevation with the owner.
5. Interim Tier-B: do **not** hand-patch the root helper outside this gate; if a Tier-B act is needed before elevation ships, it is an owner act.
