# OWNER RELAY — N2 (remove herdr) + N6 (/dev/kvm 0660 + kvm group) — ONE BLOCK
**Prepared:** 2026-11-01 by `upkeep-lead` (Team E / Lane 6), build 2026-11-01T23:45Z (18:45 ET).
**Authority:** Nox line rulings 201800Z N2 = YES / N6 = YES (batched as ONE relay block), restated in Nox 20261101T233413Z §3.
**STATUS: HELD — NOT SENT, NOT RUN.** Nothing in this file has been executed; no root command has been run from this seat.
Owner window: Lane 6's next owner window (Lead surfaces it at that point, not overnight).

**Why this is one block:** both items are root acts and were ruled to travel together. Order inside the block matters only for N6 (add the group *before* tightening the mode, so the seat is never locked out of KVM).

**What it does:** (1) archives the unused `herdr` config, then removes the `herdr` package (20.73 MiB); (2) adds `cayleb` to the `kvm` group, installs one udev rule that makes `/dev/kvm` `root:kvm 0660` (from today's `0666`), and applies it now.
**What it does NOT do:** it does not touch `~/.talos`, `~/.talos-wallet`, any VM, any service, any credential, any trading/desk process or Chromium; it does not delete any wallet material; it removes no other package (simulation: only `herdr-0.9.1-2`).

---

## 1. Measured state this block was written against (all read-only, 2026-11-01T23:37Z / 18:37 ET)
| Fact | Value |
|---|---|
| `pacman -Q herdr` | `herdr 0.9.1-2` (installed 2026-10-19 12:05:59 EDT, 20.73 MiB) |
| `/usr/bin/herdr` | 21,721,208 B, sha256 `bc20ad9255c358ea0cdde1eca854ddacb16b385f41ab3b012f40b88960bde83e` |
| `~/.config/herdr/config.toml` | 2,771 B, sha256 `d618349e3a17b84a530e5c4b41429aae2e98d6fd06601cd59f0386965bc95ca4` |
| herdr processes / units | 0 processes; no system or user systemd unit; no keybind/config reference found |
| `/dev/kvm` today | `crw-rw-rw- root kvm 10,232` → **mode 666** |
| group `kvm` today | `kvm:x:990:qemu` — `cayleb` **not** a member (`id cayleb`: `cayleb,wheel`) |
| udev default rule | `/usr/lib/udev/rules.d/50-udev-default.rules:116` sets `MODE="0666"` — the new rule must be numbered **higher** (the block uses `99-kvm-isolation.rules`, applied after it) |
| Removal simulation | `pacman -Rs --print herdr` → `herdr-0.9.1-2` only (no dependency cleanup) |

## 2. THE ONE PASTE-ABLE BLOCK
Paste the whole block below into one terminal, as user `cayleb`. `sudo` may ask for the password once — that is the only interaction. Expected output per step is in §3.

```bash
set -euo pipefail

# ============ N2 — REMOVE UNUSED herdr (config archived first) ============
A="$HOME/Work/_archive/lane6-retirement-20261101/herdr-0.9.1-2"
mkdir -p "$A"
if [ -f "$HOME/.config/herdr/config.toml" ]; then
  echo "N2 before: $(pacman -Q herdr 2>/dev/null || echo 'herdr already absent')"
  sha256sum "$HOME/.config/herdr/config.toml" | tee "$A/SHA256-BEFORE.txt"
  mv "$HOME/.config/herdr/config.toml" "$A/config.toml"
  rmdir "$HOME/.config/herdr" 2>/dev/null || true
  echo "N2 archived config sha256:"
  sha256sum "$A/config.toml"          # must be d618349e… (see §3)
else
  echo "N2: config already archived or absent — continuing"
fi
if pacman -Q herdr >/dev/null 2>&1; then
  sudo pacman -Rns --noconfirm herdr
else
  echo "N2: herdr package already absent"
fi
command -v herdr || echo "N2 OK: herdr binary absent"

# ============ N6 — /dev/kvm 0660 + cayleb in kvm group (GROUP FIRST) ============
echo "N6 before: $(ls -l /dev/kvm)"
sudo groupadd -f kvm
sudo usermod -aG kvm cayleb
printf 'KERNEL=="kvm", GROUP="kvm", MODE="0660"\n' | sudo tee /etc/udev/rules.d/99-kvm-isolation.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --name-match=kvm
sleep 1
sudo chgrp kvm /dev/kvm
sudo chmod 0660 /dev/kvm
echo "N6 after: $(ls -l /dev/kvm)"
getent group kvm
echo "DONE — say: N2+N6 installed"
```

## 3. Expected output, step by step
| # | Command | Expected output (or silence) |
|---|---|---|
| 1 | `pacman -Q herdr` | `herdr 0.9.1-2` |
| 2 | `sha256sum …/config.toml` | `d618349e3a17b84a530e5c4b41429aae2e98d6fd06601cd59f0386965bc95ca4  /home/cayleb/.config/herdr/config.toml` |
| 3 | `mv` config → archive | no output (move, nothing deleted) |
| 4 | `sha256sum $A/config.toml` | **same hash** `d618349e…` with the archive path |
| 5 | `sudo pacman -Rns --noconfirm herdr` | `checking dependencies…` then `removing herdr…` (only `herdr-0.9.1-2`); post-transaction hooks may print. No other package is removed. |
| 6 | `command -v herdr` | prints nothing; then `N2 OK: herdr binary absent` |
| 7 | `ls -l /dev/kvm` (before) | `crw-rw-rw- 1 root kvm 10, 232 … /dev/kvm` |
| 8 | `sudo groupadd -f kvm` / `sudo usermod -aG kvm cayleb` | no output |
| 9 | `sudo tee /etc/udev/rules.d/99-kvm-isolation.rules` | echoes `KERNEL=="kvm", GROUP="kvm", MODE="0660"` |
| 10 | `udevadm control --reload-rules` + `trigger --name-match=kvm` | no output |
| 11 | `ls -l /dev/kvm` (after) | `crw-rw---- 1 root kvm 10, 232 … /dev/kvm` |
| 12 | `getent group kvm` | `kvm:x:990:qemu,cayleb` |

**One more step after the block (no root):** log out and back in once (or reboot) so the new `kvm` group membership is live in the desktop session. Until then the block's own shell does not have it — that is expected and safe.

## 4. Verification (paste after the block; all read-only)
```bash
pacman -Q herdr 2>&1 || true                                # expect: error: package 'herdr' was not found
test -e /usr/bin/herdr && echo "FAIL: binary present" || echo "OK: /usr/bin/herdr absent"
sha256sum "$HOME/Work/_archive/lane6-retirement-20261101/herdr-0.9.1-2/config.toml"   # expect d618349e…
ls -l /dev/kvm                                              # expect crw-rw---- 1 root kvm
getent group kvm                                            # expect kvm:x:990:qemu,cayleb
id -nG cayleb                                               # kvm appears only after the logout/login
```
**Done-condition:** steps 11+12 match the expected lines, `pacman -Q herdr` says not found, and the archived config hash matches. Then reply to the Lead with exactly: **“N2+N6 installed”** — the Lead records it; Lane 6 then files the post-act state read-back.

## 5. Rollback (both acts are reversible; nothing is deleted)
```bash
# N2 rollback — reinstall the package and restore the config
sudo pacman -S --needed herdr
mkdir -p "$HOME/.config/herdr"
mv "$HOME/Work/_archive/lane6-retirement-20261101/herdr-0.9.1-2/config.toml" "$HOME/.config/herdr/config.toml"

# N6 rollback — back to today's state (0666, cayleb not in kvm)
sudo rm -f /etc/udev/rules.d/99-kvm-isolation.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --name-match=kvm
sudo chmod 0666 /dev/kvm
sudo gpasswd -d cayleb kvm
```
After an N6 rollback, `/dev/kvm` is `0666` again immediately; the group removal takes effect at the next login. Reinstall version: the repo's current `herdr` (today `0.9.1-2`); the archived config is byte-identical to the one measured above.

## 6. Risk notes (plain)
- `herdr` is a terminal workspace manager for AI CLIs, unused since install: 0 processes, no unit, no keybind reference. Removing it cannot stop a running lane.
- `/dev/kvm` is currently usable by every local account (666). Tightening to `root:kvm 0660` is the VM-rig isolation step Nox approved before any VM-rig exposure. Adding `cayleb` to `kvm` first keeps this desktop's own VM tooling (Boxes/virt-manager) working after re-login.
- No credential is read, printed or stored by this block. No wallet path is touched. No service is restarted.
- If any expected line differs, stop after the failing step and report the actual output — do not improvise a fix.
