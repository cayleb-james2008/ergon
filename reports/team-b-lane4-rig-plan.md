# TEAM B / LANE 4 — surfaces + device rig: plan, acceptance, blockers

Filed 2026-11-01T15:00Z by `fleet-lead` (Team B, Lane 4). Scope per the Lead's lane order and Nox's MCP
amendment `20261101T132410Z` / crew-depth note `20261101T132614Z` / mandate-effects-router ruling (adopt
design, not code). All writes stay under `~/workspace/harness-build/scratch-tb-l4/rig/` as declared; **no
apt/systemd/profile changes**; any account, purchase or licence activation stops and reports.

## 0. Acceptance properties this lane must satisfy (from the rulings)

1. **MCP facade carries (i) Nox session-start and (ii) complete remote access (bus + Stage 8 hands) over the
   same connection.** If the current Stage 5 scope could pass its own gate without those two capabilities,
   the scope is wrong and is widened here. Stage 8 owner owns hands-over-MCP exposure with the same rails and
   kill switch.
2. **The completed harness is ONE plugin** whose sidebar shows the Windows VM panel and the Android emulator
   panel; the dress rehearsal drives phone flows **through that panel**, not a side channel.
3. **Crew depth 3–5 per lead, recursive**; Router/Worker session topology is the plugin's session shape (the
   DSH-native expression of lead+crew).
4. **Effects-first durability:** orders record intended effects before execution, retry with backoff, and
   recover on restart (this weekend's mid-run lane deaths are the evidence).
5. Hard fences unchanged: separate fleet user, sandbox, Tier B; the prototype's danger-full-access worker
   model is rejected.

## 1. Host facts (from Team E's host card, 2026-11-01; treated as input, re-verified where used)

32-thread Ryzen AI MAX+ 395 · AMD-V · `/dev/kvm` usable (mode 666; root:kvm 0660 recommended but that is a
host change — **HELD, needs ruling**) · QEMU 11.1.2 · OVMF + Secure Boot + swtpm present (Win11 TPM legit)
· libvirt NOT installed (not needed — QEMU direct) · Boxes 50.0 / virt-manager 5.1.0 flatpaks · NO Android
SDK · NO Windows media · 594 G free / 42 GiB RAM · budget Win 8–12 GiB RAM + 80 G disk, Android 6–8 GiB RAM +
30 G disk · licence route: Win11 Enterprise Eval (free 90 d) or Cayleb's key/purchase (purchase = hard hold).

## 2. Work packages (each with a checkable done-condition; large batches)

| # | Package | What is built | Done-condition | Blocker/ruling |
|---|---|---|---|---|
| L4-1 | **Rig skeleton** | `scratch-tb-l4/rig/{win,android,iso,disks,logs,panel}`; QEMU/OVMF/swtpm paths pinned and hash-recorded; disk/RAM budget file; no host package changes | layout + tool path digests recorded; free-space check ≥ 200 G | none (free downloads only) |
| L4-2 | **Windows Pro/Enterprise VM** | Download Win11 **Enterprise Eval** ISO into rig; `autounattend.xml` generated locally (local account, no cloud); qcow2 80 G; QEMU line with OVMF+SecureBoot+swtpm, virtio-net/virtio-blk, VNC on loopback; guest carries the pinned `dsh-windows-remote-ssh` route | VM boots to desktop under QEMU; guest reachable over a loopback-only port; clean shutdown/re-boot reproducible; all artefacts under rig with digests | **Licence/EULA acceptance on download+install — reported to Lead as a possible "licence activation" under the hold; Windows media acquisition route to be confirmed.** `/dev/kvm` mode change HELD |
| L4-3 | **Android emulator node** | Android cmdline-tools + platform-tools + emulator + a pinned system image in rig; AVD headless (`-no-window`, gRPC/ADB on loopback); the pinned `dsh-android` device-control map wired to it | `adb devices` shows the AVD; a scripted tap/screenshot round-trip succeeds; AVD state under rig | **Google SDK licence acceptance** (`sdkmanager --licenses`) — same licence question; image download is free |
| L4-4 | **Panels in the harness** | A client-plugin surface showing the Windows VM screen (VNC/WebRTC) and the Android screen (emulator gRPC/scrcpy) as sidebar panels; panel can drive input | both panels render live frames in the DSH sidebar; input reaches the guest; panel source reviewed by a fresh checker | client-plugin build needs the web bundle rebuild path; no host change |
| L4-5 | **MCP facade widening (Stage 5 scope)** | Session-start over MCP (`dsh` session create/run through the fleet MCP facade, token-authenticated, tailnet-only); bus ops + Stage 8 hands reachable over the same facade with the same rails/kill switch | Nox, from his VM, (a) starts a session and (b) drives a real work order to completion through the facade; every hands-op carries the action log and obeys the kill switch | hands implementation is Stage 8 (next stage); the facade contract can be specified now |
| L4-6 | **Remote UI + fleet dashboard + Telegram wrapper** | Thin surfaces over the same digest/status contract as Stages 6–7; Telegram Tier A approvals only, Tier B shown-not-grantable | each surface reads state-only digests; phone can approve Tier A, cannot grant Tier B (proven by test) | Stage 6/7 build dependencies |
| L4-7 | **Adopted patterns (design, no code import)** | (i) mandate kernel in **observe mode** as the authorisation layer for self-initiated work (records would-accept, does nothing until rules promote); (ii) **write-first Effects/Outbox** for orders (effects recorded before execution, retry+backoff, restart recovery); (iii) Router/Worker session topology as the plugin's session shape; (iv) **our divergence: mandate strength revalidated at resume/fan-out** (a later message resuming old mandate is refused) | per stage: pattern named + gap it fills + where it lands; observe mode default proven by a dry run that records and does nothing | none — but these are **build items for the plugin (Stages 5–9)**, not a rig prerequisite; nothing from the prototype is imported |

## 3. Live evidence this lane will carry (already on disk)

- The mid-run lane deaths this weekend (DSH delegator context death; Codex/Outis lane retirement) are the
  evidence for Effects/Outbox and for the mandate-revalidation divergence — recorded, not manufactured.
- Studio status (for the surfaces picture only, not this lane's work): Studio is running and waiting at
  sign-in (HTTP 401 on its login endpoint); Cayleb's remaining act is to sign in and leave it open. Lane 4
  does not touch Studio.
- The current DSH host already renders sidebar panels through its client-plugin surface; the rig panels are
  built as a client plugin against that same surface, no host change.

## 4. Immediate next actions (this batch, no ruling needed)

1. Create the rig skeleton and record tool/space facts (L4-1).
2. Download the free Android command-line tools + platform-tools zips and the Windows eval ISO **only after**
   the licence question is ruled; meanwhile verify the exact download URLs and hashes without fetching
   (no account, no purchase).
3. Specify the MCP facade contract (session-start + bus + hands ops, auth, rails, kill switch) into the Stage 5
   scope file so the Stage 5 gate cannot pass without it (L4-5 spec).
4. Write the panel-plugin skeleton against the DSH client surface in a throwaway worktree (L4-4).

## 5. Blockers needing a ruling (to the Lead, recommendation included)

1. **Licence/EULA acceptance** — accepting Google SDK licences and the Microsoft eval EULA inside a
   scratch-dir install: is that a "licence activation" under the hold? *Recommendation: yes for any paid
   activation; free-eval/EULA acceptance is not a purchase and is confined to the rig, so allow it and record
   the exact texts accepted. If ruled out, the rig cannot boot Windows/Android at all and we stop at L4-1.*
2. **`/dev/kvm` mode** — root:kvm 0660 would be the proper mode; a host change, HELD. Current 666 works; *
   Recommendation: leave it as-is for the scratch rig; do not change host permissions.*
3. **Windows media route** — eval ISO acquisition may need a Microsoft eval-centre form; *recommendation: use
   the eval ISO only, no account, nothing purchased; if the form requires sign-in, stop and report.*

**Status: plan filed, skeleton starting, nothing installed, nothing enabled.**
