# N5 — STOP THE CODEX EXTENSION-HOST (durable, Chromium untouched) — PLAN FOR APPROVAL
Lane 6 · 2026-11-01 10:35 ET · Authority: Lead ruling N5 (trading-lead verified read-only: desk depends on Chromium PID 2960519 / 127.0.0.1:9222; the extension-host is a child with no socket). **Nothing executed yet** — the ruling says do not touch before Monday's 09:05 guard, and projected profile surgery must be approved first.

## 1. What was found (read-only, 10:30–10:34 ET)
- **Chromium PID 2960519** (`/usr/lib/chromium/chromium`, default profile `~/.config/chromium`, launched Sat 2026-10-31 16:19:24, `--remote-debugging-port=9222`). Listener: `ss -ltnp` → `127.0.0.1:9222` owned by **chromium 2960519**. `openbell-tabkeeper.service` active (keeps Chromium + Tradovate tab; restarts it weekdays 08:30–16:00 ET).
- **Target: PID 2960775**, PPID **2960519**, started with the browser. It is a **Chrome native-messaging host**, not a port owner: `/proc/2960775/cmdline` = `…/extension-host/linux/x64/extension-host chrome-extension://hehggadaopoacecdllhhajmbjkdcmajg/`; fds = stdio pipes to the browser + one socket. It is spawned from the manifest `~/.config/chromium/NativeMessagingHosts/com.openai.codexextension.json` (`"path": …/extension-host/linux/x64/extension-host`, allowed origins = ChatGPT extension `hehgg…` + `odlomj…`). A byte-identical mirror exists at `~/.config/google-chrome/NativeMessagingHosts/com.openai.codexextension.json`.
- The extension itself (`hehggadaopoacecdllhhajmbjkdcmajg`, "ChatGPT" v1.26.1020.11845, from webstore, 3.5 MB in `~/.config/chromium/Default/Extensions/`) is a normal profile install — it is **not** in Chromium's `--load-extension` list, and there is **no extension policy** in `/etc/chromium/policies/managed/` (only color.json + omarchy.json).
- `chrome-devtools-mcp` (PIDs 2961438/2961451) is a separate client of :9222; unrelated to the extension-host.
- **Desk acceptance baseline:** `openbell-live` (PID 1484) currently logs `feed attach/read error: RuntimeError('no trader.tradovate.com tab')` every 30 s — the exact "no Tradovate tab" class, not a CDP-port failure. (Weekend: no tab is expected.)

## 2. Durable-stop options (why the literal "disable the extension in the profile" is not safely doable without a restart)
| option | method | durable? | touches Chromium/profile? | verdict |
|---|---|---|---|---|
| **A — remove the native-host registration + stop the host (RECOMMENDED)** | move both `com.openai.codexextension.json` manifests to the retirement archive, then `kill -TERM 2960775`; the browser can no longer spawn/re-spawn the host because its manifest is gone | **yes** — no manifest, no host launch (Chrome reads the manifest on each `connectNative`) | no process/profile-content edit; only the registration file moves (reversible) | **recommended** |
| B — disable extension in profile (`extensions.settings.<id>.state=0` / edit Preferences) | edit `~/.config/chromium/Default/Preferences` | **no while Chromium runs** — Preferences is held in memory and rewritten on exit; protected-prefs integrity can reject the edit; would need a browser restart (= touching the desk route) | yes (profile content + restart) | **not recommended** — the Lead's "profile surgery you are unsure about" case |
| C — root policy block (`/etc/chromium/policies/managed/codex-extension.json`, `ExtensionSettings` blocked) | root step, applies on next policy load/restart (Monday 08:30 tabkeeper restart is **before** the 09:05 guard) | yes | needs root + browser restart; timing risk | needs ruling (root), not now |
| D — uninstall/disable via browser UI or CDP | GUI/CDP on the desk browser | yes | touches Chromium (:9222) — forbidden | rejected |

**Method A is the only clean, reversible, no-restart, no-profile-surgery route that makes the stop durable** (the host cannot respawn without its manifest). It does not disable the extension's UI/state, but it durably disables the extension's *control path* — the thing that must stop. If the Lead requires the extension itself to be shown "disabled", that needs option C (root + restart); recommend A now, C only if the extension must disappear from the profile.

## 3. Execution (only after approval + after Monday 09:05 guard)
`stop-codex-extension-host.sh` (dry-run verified; `--apply` gated):
1. Verify PID 2960519 alive, `ss -ltnp | grep :9222` owned by 2960519, PID 2960775 alive with PPID 2960519.
2. Move both native-host manifests (chromium + google-chrome) to `/home/cayleb/Work/_archive/lane6-retirement-20261101/codex-extension-host/` with sha256 + timestamp (this is the durable disable).
3. `kill -TERM 2960775`; wait ≤10 s; confirm gone and **no respawn** for 120 s.
4. Acceptance (read-only): `ss -ltnp | grep :9222` still shows chromium **2960519**; no new extension-host PID; `journalctl -u openbell-live` last lines remain the **"no trader.tradovate.com tab"** class — never "no Chromium on CDP port 9222"; tabkeeper/Chromium untouched.
5. Record `RESULT.txt` + return-code evidence; reversal line = move the two manifests back (browser may need a reload/start to reconnect the extension).
**Never:** kill/restart Chromium 2960519, touch tabkeeper, :9222, the MCP client, or run before the Monday guard.

## 4. Recommended window
- **Preferred: Monday 2 Nov after the 16:20 ET outcome check** (desk flat/closed) — zero trading risk, guard long passed.
- Alternative if the Lead wants it earlier: **Monday 09:05 guard passed → 09:10–09:15 ET**, before 09:45 open, with trading-lead watching. The stop does not touch Chromium or :9222, so the desk route is unaffected either way.
- Ask: approve **method A** and name the window (post-close preferred).
