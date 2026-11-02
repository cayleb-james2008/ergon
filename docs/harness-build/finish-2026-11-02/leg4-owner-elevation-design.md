# Leg 4. Owner Elevation: design and threat model

**Status:** design only, for Nox to rule on before any build (Nox order 112300Z: "Report design + threat model to the board before building"). Revised after an independent critique (section 17 records what changed and what is disputed). Nothing was built, installed or exposed. No privileged command was run to write this. Every machine fact was a read-only observation on outis-desktop, 2 Nov 2026.

Tags: **[verified]** observed or run here; **[code]** read in the source; **[inferred]** reasoning, not run; **[unverified]** needs a test, named in section 15 or 16.

---

## 0. Reconciliation with Legs 1, 2 and 3 (2 Nov, binding; supersedes anything below that disagrees)

1. **One approval statement: section 9.** It follows Nox's ruling: a relayed phone approval may approve or deny an elevation request as a routing marker; the password is typed only on the machine; phone-side entry is an open owner pick, not built, not blocking. The other legs cite it. Authenticated fleet handoffs to Duet carry owner authority by default but never for approving an elevation request.
2. **The Tier B confirm-path fix and the forge-path closure are in this leg** (item L4-A, section 12), by Nox's ruling. They precede every live Leg 2 grant and every Leg 3 token (Leg 2 section 13.1).
3. **v1 is app notice plus the machine-side secure prompt, and the app panel is read-only** (8.1). The first draft assumed a browser-to-host data path and browser write actions that the shipped client does not have.
4. **G40 decides case W or C for all four legs** and the other docs now cite it. Until it shows case C they all assume W (section 3). The lane-reach risk (T20) and its acceptance are decision-file ruling R1.
5. **The facade is fifteen tools, closed (Nox ruling 2 Nov 10:48 ET):** the nine core tools plus `bus.hello/who/send/pull/ack/status`. This leg adds none and is written against that surface (section 7). A further tool needs a named gap and Nox's ruling first.
6. **Channel vocabulary and `authenticated` semantics are Leg 2 section 13.5.** The desktop lease (8.4) is independent of Leg 1's `present_until`; no wake path reads it. The status pill is o1's.
7. **The attacker population grows by network-facing uid-1000 services** such as the Duet courier (T29).
8. **Shared files** follow Leg 2 section 13.2; the touchpoint list in section 14 is updated and now names `src/core/bin/dsh-fleet.mjs` (the L4-A `confirm` change) and the shim `bin/dsh-fleet.mjs`. The earlier claim of no core edit was false and is withdrawn: L4-A is one core edit.
9. **No sudo and no privileged command during the build (Nox ruling 5, 2 Nov).** Everything here is authored and tested against a fake sudo. The real elevation proof (section 13) is an owner-run step at home after the build and gate, and it depends on D1, the open owner-run root install. Done-when item 4, "one real elevation request completes end-to-end", therefore **cannot close in this build**; the build closes it only as authored, fake-sudo-tested and ready for the owner's run.

---

## 1. Recommendation

Build **design A+**: a user-level secure-entry program that feeds `sudo -S -k` once, with its *code* installed as a handful of root-owned static files and launched by absolute path. There is no daemon, no systemd unit, no setuid bit and no sudoers change.

What the owner and Nox need to know:

1. **Authority is the password, nothing else.** Every lane runs as uid 1000, so any lane can forge a database row, drop a file in the confirms directory, call the control socket, or call the facade. None of those prove consent. The one act a lane cannot perform is typing the owner's password into a prompt that shows the argv the same process then runs. Rows, buttons and taps are routing and record only (section 3). Every control-socket channel is recorded `authenticated=0`.
2. **No uid-1000 launch path is trustworthy.** The app card, the CLI, the broker code and the control-socket ops are all lane-writable. A lane that wants the password can point them at its own look-alike prompt, and the owner can press the genuine button and still be shown it. Root-owned files therefore do **not** make the prompt trustworthy. What A+ buys is narrower: the genuine entry program cannot be silently edited, and a checker can tell whether the files on disk are the reviewed ones. The substituted-prompt attack stays open at uid 1000 (T3) and is the main residual risk. This is stated in the owner's install notes.
3. **The password is typed on the machine only** (Nox's ruling). No password travels through Nox, the bus, the board, a phone app or a log. A relayed phone approval may approve or deny a request as a routing marker; it is never authority (section 9, the one statement).
4. **The away path is a queue in this build.** Leg 2's bus now exists, but nothing reads bus mail for Nox and the owner reads it only through `comms-inbox`. A request files one `notice` to `cayleb-local` (id, requester, 12-hex hash, expiry; never a password) that sits in a queue; the app attention count (the status pill is o1's) and `dsh-fleet elevate list` are the real signals. An unanswered request expires after 15 minutes and the agent files again.
5. **"Take control briefly" is an audit marker**, not a quiet window. No lane has a GUI tool in this install, so there is nothing for a lease to gate (section 8.4).
6. **The one real proof run is `/usr/bin/id -u` under sudo**, expected output `0` (section 13). It is deferred: the owner runs it at home after the build and gate, once D1 (the root install) is done. It is not part of this build, and no sudo is run here (Nox ruling 5).
7. **The Tier B confirm-path break and the forge path are in this leg's scope** (L4-A, section 12): in the consolidated install any lane can forge a confirm file today, so confirms of both tiers move to this same password-gated entry program (A2). Hand-placed Tier A confirms exist only under Nox's explicit ruling, per-ticket or a bounded standing ruling (`TIER-A-GRANTS-LEDGER.md`) and only until the A3 flip; after the flip a file confirms nothing and Tier A spend confirms go through the entry program too.

Decisions: (D1) A+ with a one-time owner-run root install of static files: **open**. (D2) the phone/Nox rule: **settled** by Nox's ruling (section 9). (D3) the Tier B confirm fix: **settled**, part of this build (L4-A). (D4) facade `elevate.list` and `elevate.decide`: **NEEDS-RULING (decision file R4), not assumed**. The facade is closed at fifteen tools; neither tool exists, and one would need a named gap and Nox's ruling first (section 7).

---

## 2. What was checked

| Fact | Evidence |
|---|---|
| `kernel.yama.ptrace_scope` = 1; LSMs include yama | [verified] `/proc/sys/kernel/yama/ptrace_scope`, `/sys/kernel/security/lsm` |
| `fs.suid_dumpable` = 2; `core_pattern` pipes to systemd-coredump; `ulimit -c` unlimited in this shell | [verified]. A crashing dumpable process would be dumped today. |
| Swap exists: a 62 GB swapfile plus zram | [verified] `swapon --show` |
| uid 1000 is in `wheel`, not `input`. `/dev/input/event*` is `root:input 0660`, so no evdev capture. `/dev/uinput` has an ACL granting `cayleb` rw, so any uid-1000 process can **inject** keystrokes (not read them). | [verified] `id`, `ls -l`, `getfacl` |
| Wayland/Hyprland session. `/run/user/1000` and the compositor socket `/run/user/1000/wayland-1` are **owned by cayleb**, so any uid-1000 process can move the socket and bind a forwarding proxy. `polkitd` runs; **no graphical polkit agent** (`pgrep polkit` matches only polkitd). `/usr/bin/Hyprland` is root-owned 0755. | [verified] `ls -ld`, `pgrep` |
| `~/.config/systemd/user` is **cayleb-owned** and holds many units; systemd gives user-level units precedence over `/usr/lib/systemd/user`. `systemctl --user` and `systemd-run --user` are present. `systemctl --user set-environment` works for any uid-1000 process. | [verified] `ls -ld`; precedence [inferred] from systemd's unit search order |
| `sudo` 1.9.17p2, setuid root. `/usr/bin/id`, `/usr/bin/timeout` root-owned; `/`, `/usr`, `/usr/bin` not group/other-writable. | [verified] |
| `/etc/sudoers` is `0440 root`; `/etc/sudoers.d` is `0750 root` and **unreadable** to uid 1000, so `passprompt_override`, `use_pty`, `passwd_tries`, `timestamp_type`, `requiretty` and any `NOPASSWD` are unknown. | [verified] `ls`; settings [unverified], section 16 |
| sudo man page: `-k` with a command ignores cached credentials and does not update them; `-S` reads the password from stdin and writes the prompt to stderr. | [verified] `man sudo` |
| sudo's own journal lines carry `_EXE=/usr/bin/sudo`, `SYSLOG_IDENTIFIER=sudo`, `_TRANSPORT=syslog` (journald sets `_EXE` and `_TRANSPORT` from the sender; an ordinary process cannot choose them). uid 1000 can read them. | [verified] one record's field names via `journalctl -o json _COMM=sudo` |
| `pinentry`, `zenity`, PyGObject with GTK 4, `gcc`, `python3`, `foot`, `grim` present | [verified] |
| **A same-uid process can re-open another process's pipe through `/proc/<pid>/fd/N` while that process is dumpable. It cannot for a socket (ENXIO), nor for either once the holder is non-dumpable (EACCES).** | [verified] my test, 4 cases: pipe/dumpable=1 read the canary; pipe/dumpable=0 denied; socket/dumpable=1 ENXIO; socket/dumpable=0 denied |
| Control socket admits on SO_PEERCRED uid 1000 plus a cgroup deny list whose only entry is **`fleet-hands.service`**. Lanes are not excluded by it. | [code] `core/config.js:26`, `core/control.js:39-42` |
| No GUI tool class exists. `policy/holds.js` has deploy, post, credential, policy, spend and trade only; a grep of `src/` finds no computer-use or screenshot tool; v1.2 keeps GUI control off bots (lines 129-130, 164) in the separate `fleet-hands` child, which is not in this package. | [code] `policy/holds.js`; [verified] `ls src` |
| The facade had nine tools when this was first written (Leg 2 has since built it to fifteen, section 7) (`fleet.status`, `session.create/prompt/status`, `board.op`, `order.drive_roundtrip/stage_real`, `fleet.selftest`, `actions.status`), one shared bearer token, GET returns 405. No message, notify or elevate tool. | [code] `mcp/tools.js`, `duet-wake-evidence.md` |
| `src/` has no bus, phone or Telegram module. | [verified] `ls src` |
| Events table is append-only and hash-chained; the hash is `sha256(JSON[prev, seq, ts, kind, actor, data])` with **no secret**; heads go to journald by a plain write. | [code] `core/store.js`, `core/chainhead.js` |
| Tickets: forward-only states, immutable binding columns, no delete, grants single-use and 10 minutes. | [code] `policy/tickets.js` |
| `processConfirms` accepts a regular file owned by the host's uid whose JSON has `pkexec_uid` equal to `peerUid`. | [code] `policy/policy.js:253-296` |
| `policy/NOTES.md:60-63` says bots cannot write the confirms dir because fleet-sandbox masks `state/` and fsguard denies the in-process write. The consolidated unit sets no `ProtectHome`, `ProtectSystem` or `NoNewPrivileges` and runs as cayleb. | [code] `policy/NOTES.md`; [verified] `systemctl --user cat` |
| fleet-sandbox's bwrap profile: `--ro-bind / /`, `--tmpfs /run`, workspace-only writes, `--unshare-net`. The roster wraps Claude Code and Codex lane engines in it. **Whether DSH-native lane tools in the consolidated install run inside it is not established.** | [code] `sandbox/NOTES.md`, `roster/engines.js:102` |

---

## 3. The governing principle and the attacker baseline

**Authority is the owner's password, typed at the machine into a prompt that shows the exact argv the same process will then run. Everything else is routing and record.**

Lanes can do all of these today [code + F1 accepted] unless confined (below): write the fleet database (a forged tail validates); write the confirms directory (finding 1); call every control-socket owner op, since the deny list names only `fleet-hands.service`; call the facade with the shared token.

Consequences:

- A forged request, row or owner-op call can only cause a prompt. The prompt shows the true argv, so the owner can refuse.
- The process that displays the argv must be the process that executes it. No other component may supply the text the owner judges.
- The real threat is **password phishing**: a substituted prompt, a captured keystroke, a read of the entry program's memory, a tampered program. With the password an attacker runs anything.

**Attacker baseline, two cases.**

- **W (worst, assumed by the design):** a lane's shell runs as uid 1000 unconfined, and so does any other uid-1000 process that parses external content (the Duet courier, a relay session: T29). It can write `~/.config`, `~/.config/systemd/user`, `/run/user/1000`, run `systemctl --user`, `systemd-run --user`, set the user manager's environment, bind a socket at the Wayland path, call every owner op.
- **C (confined):** the lane's tool shell runs inside fleet-sandbox: `/home` is read-only, `/run` is an empty tmpfs (no `/run/user/1000`, no control socket), network is off, writes only in its workspace. Then T3, T5, T8, T19 and T20 mostly close.

Which case holds in the consolidated install is **[unverified]**: the roster confines engine lanes, but nothing here shows the DSH-native tool shell is confined, and the unit has no sandbox properties. Gate item G40 settles it with a read-only probe run from a real lane, and settles it for all four legs, which now cite it. Until it shows case C they all assume W. Every control below is designed for W. If G40 shows lane shells inside the fleet unit's cgroup there is a config-only option: add that unit to `denyCgroupUnits` (`core/config.js`; `core/control.js admit`), which keeps lane shells off the control socket and leaves the owner's terminal on it. The app-launched entry program is a child of the host and would be refused too, so L0 must decide how it reaches the broker; the CLI path `dsh-fleet elevate enter` runs outside that cgroup.

---

## 4. The three options

(A) user-level helper feeding `sudo -S -k -p ... -- argv`. (B) polkit action plus pkexec, the desktop polkit agent prompting. (C) a root-owned helper daemon. "A+" is A with root-owned static code launched by absolute path.

| | A (as briefed) | **A+ (recommended)** | B (polkit/pkexec) | C (root daemon) |
|---|---|---|---|---|
| Password typed into | helper's own GTK prompt (uid 1000) | same | stock polkit agent (uid 1000; none runs today) | our prompt (uid 1000), sent to the daemon |
| Where it then lives | helper memory, then a socketpair to sudo | same | agent memory, then a pipe to `polkit-agent-helper-1` | helper memory, socket to a root process |
| For how long | typing time plus the helper's life | same | agent's life (not ours to bound) | typing time, then root memory until PAM returns |
| Who can read it (same uid) | blocked if non-dumpable and socket-fed; **but a lane can edit the helper's files** | same blocks; the genuine code is root-owned, so it cannot be silently edited | the agent is third-party and dumpable; its pipe is re-openable [verified pattern] | root memory is out of reach; the UI step is the same as A |
| What a same-uid lane can do | edit the helper; spoof the prompt; proxy the Wayland socket | spoof the prompt, substitute the launch path, proxy the socket (caught by H11) | spawn its own `pkexec` and raise a genuine dialog; spoof | spoof and key-capture on the UI step; attack the daemon's socket |
| Binds approval to the argv | one process shows and runs it | same | **no**: the action is generic | yes, in root memory |
| Root-owned install needed | no | **static files only** | policy, rules, runner, plus a polkit agent | daemon, unit, PAM service, socket |
| Privileged code added | none | none | small runner | a root daemon executing argv for a uid-1000 caller |
| Testable here with a fake sudo | yes | yes | no | partly |
| Fit for this build (no sudo here, Nox ruling 5) | yes | yes (code and fake-sudo tests need no sudo; the root install is one owner step at home) | no | no |

**Why not B.** It moves password checking to PAM but not the typing step. A generic action cannot carry a per-request grant, so our request layer would be advisory around a prompt any lane can raise by calling `pkexec`. It needs a root install and an agent that is not running.

**Why not C yet.** Strongest on audit, and it buys nothing at the typing step. It adds a root daemon that executes argv for a uid-1000 caller; any bug is a root exploit. Keep it as the upgrade: the entry program's execution step sits behind a small `Executor` interface (A+: spawn sudo; C: send to the daemon).

**Why A+ and not plain A, stated honestly.** With A, a lane edits the genuine helper to write the password to a file and the owner, doing everything right, leaks it. With A+ the lane must instead change what the owner launches (the broker, app or CLI), which is also lane-writable. So A+ is a smaller step than it first looks: it removes the *silent* edit of the genuine program and makes the files checkable against a manifest, and it costs one owner install of static files. It does not remove the substituted launch (T3, T19).

---

## 5. The recommended design (A+)

### 5.1 Parts

| Part | Where it runs | Sees the password | Job |
|---|---|---|---|
| **Broker** | in the fleet host (new module `src/elevate/`) | **never** | request and prompt tables, state machine, expiry timer, caps (anti-nuisance), agent tools, owner ops, audit events, app notice |
| **Entry program** | one Python 3 process per prompt, started by absolute path | yes | rebuild a clean environment, verify the display, fetch the request, show and judge the argv, run the prompt, run sudo, report the outcome |
| **Prompt** | inside the entry program (GTK 4 via PyGObject) | yes | masked entry, nothing else |
| **Install kit** | root-owned static files in `/usr/local/libexec/fleet-elevate/` | no | `enter.py`, prompt, catalog, rules file, an empty `home/` directory, a config file (compositor path, socket name), `MANIFEST` |

There is no systemd unit. A user unit cannot be started "by full path", user-level units shadow system ones, and a template unit cannot receive a descriptor from its caller. The broker (or the CLI) runs `/usr/bin/python3 -I /usr/local/libexec/fleet-elevate/enter.py <id>` directly with an explicit argument vector. The entry program ignores everything it inherits.

Why not the DSH web UI or the host process for entry: the app renders lane-written text and the host is the parent of the lanes' shells. The app card never has a password field.

The entry program uses stdlib, `ctypes` (for `prctl`, `setrlimit`, `mlock`) and PyGObject. The broker is JS. The argv rules, the rules file and the binding hash exist in both and are checked against one shared vector file (build step 1).

### 5.2 Flow

1. A lane calls `elevate_request`. The broker validates, computes the binding hash, writes a `pending` row and an `elevate.request` event, returns an id. No prompt appears.
2. The surfaces in this build: a read-only card on Leg 1's activity page (`elevations[]`) and a count in its `attention` (which o1's status pill reads), `dsh-fleet elevate list`, and one `notice` to `cayleb-local` on the bus (a queue, Leg 2 2.5). Further requests join the same list. Nothing is sent to anyone else.
3. **At the machine:** the owner reads the card or `dsh-fleet elevate show e7` and runs `dsh-fleet elevate enter e7` in his own terminal (there is no button in v1, 8.1). The broker inserts a `prompts` row (at most one open per request) and spawns the entry program. The owner always opens the prompt himself; a password window he did not open is not the real one (this blocks the *unsolicited* fake only; see T3).
4. The entry program starts: first `PR_SET_DUMPABLE 0` and `RLIMIT_CORE 0` (H1); a clean environment (H4); the Wayland identity check (H11); then it fetches the request by id from the broker with the owner op `elevate-fetch`. **Everything fetched is untrusted data**: the database and the broker's answer are lane-reachable. The program recomputes the binding hash from the fields it holds, runs the preflight (H9, rules in 5.5), classifies the argv with its own catalog, and shows the prompt (5.4). Its own in-memory copy of the argv is what it displays and what it will execute; it never re-reads the request.
5. The owner types the password and presses Enter. The program calls `elevate-start {id, prompt}`, an atomic compare-and-set from `pending` to `running` that exactly one caller can win (`already-started` otherwise, `not-pending` if withdrawn, denied or expired meanwhile). It then starts `/usr/bin/sudo -S -k -p <nonce> -- /usr/bin/timeout -k 5 <timeout_s> <argv...>` with stdin on a socketpair and a scrubbed environment, waits for the nonce on stderr, writes the password and a newline, calls `shutdown(SHUT_WR)` on the socket, and zeroes the buffer. If the nonce never appears (cached credentials, NOPASSWD, a broken policy) it writes nothing and ends with `no-prompt`.
6. **Wrong password.** Because the write end is shut down, sudo's retry reads EOF and sudo exits with the command never started. The program sees the nonce a second time (or a failed exit with the nonce seen twice), tells the owner, and allows one more entry: a new sudo process, a new nonce. A request allows at most `elevate.entries` (default 2) password entries in total, so sudo's own three tries are never reached. A second failure ends the request `bad-password`. A cancelled, timed-out or focus-lost prompt *before* the first submission leaves the request `pending`; the owner may open another prompt (limit in 5.6).
7. The command runs under the root-owned `/usr/bin/timeout`, whose argv is part of what the owner sees and what is hashed. The program caps the output. If sudo is still alive 15 seconds after the limit it sends SIGTERM to sudo as a best effort (whether it may signal sudo's root child is **[unverified]**, section 16).
8. The program reports the outcome with the owner op `elevate-outcome {id, result, ...}`, retrying for 60 seconds. There is no spool file; a lost outcome is reconciled at the next broker start (a `running` row with no outcome and no live entry process becomes `feeder-lost`). The outcome op is **unauthenticated**: any lane can call it, so an outcome is a claim, and the proof is sudo's own journal line (section 10).
9. The broker writes the `outcome` row and events; the lane's next `elevate_status` returns the result.

**What the binding hash does and does not guarantee.** It lets the owner compare the card and the prompt, and ties every audit row to one command. It is **not** integrity against a hostile database or broker: both are lane-reachable. What makes display equal execution is that one process holds one argv in memory, shows it and passes it to sudo. A forged row therefore changes what the prompt shows and what runs together, never one without the other (G38).

### 5.3 Hardening rules (each is a gate item)

- **H1. Non-dumpable and no core.** First act of the program: `prctl(PR_SET_DUMPABLE, 0)` and `RLIMIT_CORE = 0`, before any window or password exists. Dumpable 0 makes `/proc/<pid>/*` root-owned and blocks `mem`, `fd`, `maps` and ptrace by a same-uid process at any `ptrace_scope`. [verified in the pipe test.] `suid_dumpable=2` on this host makes `RLIMIT_CORE=0` necessary too.
- **H2. Socketpairs, never pipes, for anything that carries the password** (program to sudo). [verified: a dumpable holder's pipe is re-openable by any same-uid process; a socket is not.] Command stdout and stderr may be pipes.
- **H3. Memory.** The password lives in a `bytearray`, zeroed right after the write; nothing keeps it for later (there is no output scan, 5.5). `mlockall` where RLIMIT_MEMLOCK allows, otherwise the outcome records `mlock=false` and the footer says so. A Python `str` copy from the toolkit cannot be wiped; the process is non-dumpable and exits within seconds. A C entry program with `explicit_bzero` is the stronger alternative.
- **H4. Clean environment and no user configuration.** `enter.py` starts with a bootstrap that, if it finds any `LD_*`, `PYTHON*`, `GTK_*`, `GIO_*`, `GI_*` or `GDK_*` variable, re-executes itself with a built environment (a library named by the spawner is processed before line one, but it never sees the password). The environment it builds, ignoring everything inherited: a fixed `PATH`, `XDG_RUNTIME_DIR=/run/user/<uid>`, `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` all set to the root-owned empty `/usr/local/libexec/fleet-elevate/home`, `GTK_THEME` and `GSETTINGS_BACKEND=memory` set explicitly, `GTK_MODULES` empty, `GTK_A11Y=none`, `GDK_BACKEND=wayland`, `FONTCONFIG_FILE` pointing at a root-owned file. Without this, `~/.config/gtk-4.0/gtk.css` or `settings.ini` could restyle or hide the argv label (H7), and `systemctl --user set-environment LD_PRELOAD=...` could load a lane's library. No `NoNewPrivileges` is set anywhere (it would break setuid sudo); the program refuses to start if `/proc/self/status` shows `NoNewPrivs: 1`.
- **H5. No terminal.** The program never reads a password from a tty or stdin. With no secure display it ends with `no-secure-display`.
- **H6. Prompt behaviour.** Masked entry, no reveal toggle, **paste, drag-drop and the context menu disabled** (Hyprland supports data-control clients, so any same-uid process can read the clipboard). The field is **cleared and submission ignored whenever the window loses keyboard focus**. Total prompt life 120 seconds.
- **H7. Display fidelity.** Argv in monospace, one element per line; control characters, bidirectional overrides, zero-width characters and non-ASCII shown as visible escapes; never truncated (over-length requests are refused). The theme and style come from the pinned environment in H4, not from the owner's home.
- **H8. Exact sudo line.** `/usr/bin/sudo -S -k -p <nonce> -- /usr/bin/timeout -k 5 <T> <argv...>`. The nonce is 128 random bits. No `-E`, `-s`, `-i`, `-u` other than root. The `sudo` path is a constant; tests inject a fake through the library function only.
- **H9. Binary check.** At filing and again just before spawn: `argv[0]` and `/usr/bin/timeout` are absolute, their realpath and every parent directory are owned by uid 0 and not group/other-writable, and sha256, device and inode match what was filed. Otherwise `binary-not-root-protected` or `exe-changed`.
- **H10. Stdin and the nonce gate.** The command never inherits a live stdin: the password is written only after sudo's nonce prompt, and the write end is shut down at once, so a command reads EOF. If sudo never asked (NOPASSWD, a cached ticket), nothing is written.
- **H11. Display identity.** Before any window: the program opens the Wayland socket at the fixed path `/run/user/<uid>/<name>` (the name recorded in the root-owned config at install), reads `SO_PEERCRED`, and requires the peer's `/proc/<pid>/exe` to be the root-owned compositor binary named in the config and the peer's cgroup to be the owner's active logind session scope (this rejects a lane-run nested compositor). It then hands GTK that **same connected descriptor** through `WAYLAND_SOCKET`, so a socket moved after the check is not used. Otherwise it ends `display-identity-failed`. A forwarding proxy or a lane-moved socket therefore fails closed. **[inferred; G37]**

### 5.4 What the prompt shows

Top to bottom, all built by the entry program from the data it will execute:

1. `RUN AS ROOT`, then the **full sudo command line after `--`**, one element per line, each in quotes; the first three lines are the `timeout` wrapper, so the limit is visible.
2. cwd, and whether output is returned to the requester. When it is: **"Output goes to the agent. Root output can contain secrets."**
3. **Classification from the entry program's own catalog, never from the broker.** `KNOWN: id (prints your user id, changes nothing)` or `UNRECOGNIZED PROGRAM` plus risk flags (5.5). Unrecognized programs disable Enter for 5 seconds.
4. Requester: agent, session, order, request age.
5. The agent's reason, in a box titled **"Agent's claim, not checked"**, ASCII-sanitised, cut at 280 characters.
6. Binding hash, first 12 hex characters (the card shows the same), the footer, the expiry countdown. The footer says: *Only type here if you pressed the button yourself.*

The catalog is a convenience. The deny lists are a tripwire, not a fence: **the fence is the owner reading item 1.**

### 5.5 What a request may be (checked at filing by the broker, again by the entry program)

The exact lists below are the minimum; the build puts them in `vectors/elevate-rules.json`, the single source for both languages and for gate item G20.

- `argv`: 1 to 32 strings, each at most 4096 bytes, total at most 16 KiB, no NUL, no control characters. No shell, no `-c`, no wildcard expansion, no pipeline.
- `argv[0]` absolute and root-protected (H9).
- **`interpreter-refused`** (by basename of the realpath): `sh bash dash zsh fish ksh csh tcsh busybox env sudo su doas pkexec run0 nsenter unshare chroot setpriv script python python3 python3.* perl ruby node nodejs lua luajit php tclsh wish awk gawk mawk find xargs vi vim nvim nano ed ex less more most man gdb strace ltrace tar git rsync ssh scp sftp make docker podman flatpak systemd-run machinectl`.
- **`persistence-refused`**: any argument (or the part after the first `=`), normalised lexically, that equals or lies under `/etc/sudoers`, `/etc/sudoers.d`, `/etc/sudo.conf`, `/etc/pam.d`, `/etc/security`, `/etc/polkit-1`, `/usr/share/polkit-1`, `/etc/ld.so.conf`, `/etc/ld.so.conf.d`, `/etc/ld.so.preload`, `/etc/shadow`, `/etc/gshadow`, `/etc/passwd`, `/etc/group`, `/etc/ssh`, `/etc/cron*`, `/var/spool/cron`, `/etc/profile.d`, `/etc/environment`, `/root`, `/boot`, `/usr/local/libexec/fleet-elevate`; and `argv[0]` in `visudo passwd chpasswd usermod useradd userdel groupmod gpasswd chsh chfn vipw setcap chattr`.
- **`secret-in-argv`**: a leading `VAR=value` word, or an argument starting `--password`, `--token`, `--secret`, `--key=` (sudo logs the argv).
- **Classification catalog** (flags): `systemctl` : `changes-services`; `pacman` : `installs-packages writes-files network`; `cp mv rm tee chmod chown ln install mkdir touch truncate` : `writes-files`; `curl wget` : `network`; `KNOWN` (no flags, unchanged state): `id whoami true uname`. Anything else is `UNRECOGNIZED PROGRAM`; an argument naming a file writable by uid 1000 adds `argument-is-user-writable-file`.
- `cwd`: absolute, a real directory, not a symlink, default `/`; realpath recorded and re-checked.
- `timeout_s`: 1 to 120, default 30. Enforced by root-owned `/usr/bin/timeout -k 5`, which signals the command's process group: **best effort**. A command that daemonizes or changes its process group escapes; the owner approves with that in view.
- `return_output`: default false. When true the requester gets the last 2 KiB of each stream. **There is no output scan.** The password is never in the command's reach (H10), so a scan has nothing to find, and keeping a copy of the password for it would contradict H3. The risk that matters is that root output can hold secrets; the owner accepts it at the prompt (item 2).
- `reason`: required, 280 characters, control and bidirectional characters stripped.
- Run-as is always root in v1. There is no `-u`.

### 5.6 Times and caps (policy-table keys, defaults)

| Key | Default | Meaning |
|---|---|---|
| `elevate.pending-ms` | 900000 (15 min) | unanswered request expires; deny by default |
| `elevate.prompt-ms` | 120000 | one prompt's life |
| `elevate.entries` | 2 | password entries per request |
| `elevate.prompts-per-request` | 3 | prompt opens per request; at most 1 per 60 s |
| `elevate.max-pending-agent` / `-total` | 1 / 3 | pending cap |
| `elevate.agent-per-hour` / `-per-day-total` | 3 / 6 | rate caps (v1.2 target: at most 6 approvals a day) |
| `elevate.denied-cooldown-ms` | 3600000 | same binding hash refused after a denial |
| `elevate.suspend-after-denials` | 3 in 24 h | agent loses the tool until the owner clears it |

**These caps are anti-nuisance, not a boundary.** They stop an honest agent that loops. A lane with a shell can call `unlock`, clear its own suspension, forge rows and call owner ops, so against a lane the real controls are only that no prompt opens by itself and the owner reads the argv. The keys are declared by the module the way `orders/schema.js` declares its keys. Changing one is nominally a Tier B policy change, but that confirm path is broken (finding 2) and forgeable (finding 1): until finding 2 is fixed there is no sound way to change them, so they are constants in code, and a lane can change them only through the forgeable confirm.

---

## 6. Data model

Same conventions as `policy_tickets`: forward-only `state`, immutable binding columns, monotone counters, no delete. The one read-time overlay is `pending` past `pending_expires_at`, read as `expired`; the module's own 30-second timer writes the stored `expired` and the `elevate.expired` event once. (The housekeeper has no such pass and is not used.) Ids are `e<N>`.

```sql
CREATE TABLE elevation_requests (
  id INTEGER PRIMARY KEY,
  state TEXT NOT NULL DEFAULT 'pending' CHECK (state IN
    ('pending','running','done','failed','denied','expired','withdrawn','void')),
  -- bound, immutable
  argv TEXT NOT NULL,                 -- JSON array, the user's argv (the wrapper is derived)
  cwd TEXT NOT NULL, env_policy TEXT NOT NULL DEFAULT 'scrub-v1',
  run_as TEXT NOT NULL DEFAULT 'root' CHECK (run_as = 'root'),
  timeout_s INTEGER NOT NULL, return_output INTEGER NOT NULL CHECK (return_output IN (0,1)),
  exe_path TEXT NOT NULL, exe_sha256 TEXT NOT NULL, exe_dev INTEGER NOT NULL, exe_ino INTEGER NOT NULL,
  agent TEXT NOT NULL, session TEXT, order_id INTEGER, task_id INTEGER,
  reason TEXT NOT NULL,               -- untrusted label
  binding_sha256 TEXT NOT NULL,
  classification TEXT NOT NULL, risk_flags TEXT NOT NULL,   -- advisory; the entry program recomputes
  created_at INTEGER NOT NULL, pending_expires_at INTEGER NOT NULL,
  -- movable, monotone
  entries INTEGER NOT NULL DEFAULT 0, prompts INTEGER NOT NULL DEFAULT 0,
  closed_by TEXT, closed_channel TEXT,      -- who denied, withdrew or voided; channel is the Leg 2 section 13.5 vocabulary or timer
  remote_decision TEXT CHECK (remote_decision IN ('approved','denied')), remote_channel TEXT, remote_authenticated INTEGER, remote_at INTEGER,  -- a routing marker, never authority (section 9); set once
  finished_at INTEGER, void_reason TEXT
);
CREATE TABLE elevation_prompts (
  id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL REFERENCES elevation_requests(id),
  opened_at INTEGER NOT NULL, opened_by TEXT NOT NULL CHECK (opened_by IN ('app','cli')),
  entry_pid INTEGER, closed_at INTEGER, entries INTEGER NOT NULL DEFAULT 0,
  close_reason TEXT CHECK (close_reason IN ('submitted','cancelled','timeout','focus-lost','refused','crashed'))
);
CREATE UNIQUE INDEX one_open_prompt ON elevation_prompts(request_id) WHERE closed_at IS NULL;
CREATE TABLE elevation_outcomes (
  id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL UNIQUE REFERENCES elevation_requests(id),
  source TEXT NOT NULL CHECK (source IN ('entry-report','reconcile')),   -- unauthenticated either way
  result TEXT NOT NULL CHECK (result IN ('ok','command-failed','timeout','killed','bad-password','no-prompt',
    'exe-changed','binding-mismatch','spawn-failed','no-secure-display','display-identity-failed','feeder-lost')),
  started_at INTEGER, ended_at INTEGER NOT NULL, sudo_asked INTEGER, entries INTEGER NOT NULL,
  exit_code INTEGER, signal INTEGER,
  stdout_sha256 TEXT, stderr_sha256 TEXT, stdout_bytes INTEGER, stderr_bytes INTEGER,
  stdout_tail TEXT, stderr_tail TEXT,           -- only when return_output; at most 2 KiB each
  hardening TEXT NOT NULL                       -- JSON: dumpable, rlimit_core, mlock, nonewprivs, ptrace_scope, display_identity
);
```

Triggers: bound columns immutable; `state` moves only along the graph below; counters only increase; no delete; `elevation_outcomes` insert-only. No table holds a secret or anything derived from one. There is no grants table: the only authenticated act is sudo accepting the password, and the record of that is sudo's own journal line (section 10). Every uid-only owner channel (`control-socket`, `browser-peer`) is lane-reachable in case W and is recorded as such in the event, `authenticated:0` (Leg 2 section 13.5).

**Binding hash.** `binding_sha256 = sha256(canonical({v:1, argv, cwd, env:{policy:'scrub-v1', vars:{PATH:'/usr/sbin:/usr/bin', LC_ALL:'C'}}, run_as:'root', timeout_s, return_output, exe:{path, sha256, dev, ino}, wrapper:{path:'/usr/bin/timeout', sha256}, requester:{agent, session, order}}))`, canonical form (keys sorted, no whitespace) as in `policy/log.js`. See the limits in 5.2.

**State graph.**

```
pending -> running | denied | expired | withdrawn | void
running -> done | failed
```

`running` can only end, which is what makes a request single-use. The move to `running` is one compare-and-set in `elevate-start`; two callers race and one wins. A request is withdrawable only while `pending`. A prompt is not a request state: it is a row in `elevation_prompts`, at most one open per request (the partial unique index), and a cancelled or timed-out prompt closes its row and changes nothing else.

---

## 7. The agent-facing tools

Registered the way `holdTools` registers its tools: global tools, visible by role through `when`, the caller taken from `exec.agent`, never from arguments, refusals as `<code>: <detail>`. Default roles allowed: `lead`, `builder`. Checkers, researchers and consult children are default-deny.

**`elevate_request`**

| Arg | Type | Rule |
|---|---|---|
| `argv` | string[] | required; section 5.5 |
| `reason` | string | required, at most 280 characters |
| `cwd` | string | default `/` |
| `timeout_s` | integer 1..120 | default 30 |
| `return_output` | boolean | default false |

Returns `{id, state:'pending', expires_at, binding_sha256, classification}`. The call never blocks and never prompts. The description tells the agent: do not repeat, poll `elevate_status`, no password can be supplied, a denial means stop.

Refusals: `bad-argv`, `argv-not-absolute`, `binary-not-root-protected`, `interpreter-refused`, `persistence-refused`, `secret-in-argv`, `bad-cwd`, `reason-required`, `reason-too-long`, `role-not-permitted`, `too-many-pending`, `rate-limited`, `duplicate-pending` (returns the existing id), `recently-denied`, `agent-suspended`, `elevation-locked`, `no-secure-display` (no graphical session or kit not installed).

**`elevate_status {id}`** returns `{id, state, expires_at, outcome?}`; `outcome` is `{result, exit_code, signal, stdout_tail?, stderr_tail?}`, tails only if `return_output` was set. A caller sees only its own requests (`not-found`; `bad-id`).

**`elevate_withdraw {id}`** only from `pending`; `not-pending` once running.

**No facade tool. The facade is fifteen tools and closed (Nox ruling 2 Nov 10:48 ET):** the nine core tools plus `bus.hello`, `bus.who`, `bus.send`, `bus.pull`, `bus.ack`, `bus.status`. Any further tool needs a named gap and Nox's ruling first. This leg names no such gap in v1 and adds nothing: `elevate.list` and `elevate.decide` (approve or deny, section 9) are **not tools and not assumed**. They are NEEDS-RULING item R4 in the decision file, to be raised only if a reader for Nox and the phone makes the gap real (Leg 2 gap 22). Nothing in this design, the build list or the gates depends on them; the agent tools of this section are lane tools (`elevate_request`, `elevate_status`, `elevate_withdraw`), not facade tools. No elevate op is allowlisted on `board.op`, and `duet` and every `external` kind have no route to elevation at all.

---

## 8. Owner surfaces

### 8.1 In the DSH app: a read-only notice (v1)

**Correction of the first draft.** It assumed a browser-to-host data path and browser write actions ("Review and enter password", "Deny") on "the plugin client route the board panels use". The shipped client has neither: `client/index.js` injects only slots and locale, and the board panel is an inert placeholder (Leg 1 section 2). Leg 1's G0 spike is the first proof that a browser can read host data at all, and Leg 1 forbids browser writes until its flag 2 is ruled.

**v1 is app notice plus the machine-side secure prompt.** The app shows a read-only card on Leg 1's activity page, rendered from `elevations[]` (id, state, requester, age, expiry, the 12-hex binding hash, classification, `remote_decision`) and a count in `attention`, which o1's status pill reads. The pill is o1's; this leg does not touch it. The card has no argv, no password field and no button, and Leg 1's remote-authority gate applies to it. The owner reads the command at the machine (`dsh-fleet elevate show e7`), runs `dsh-fleet elevate enter e7`, and types the password into the entry program. The card is lane-supplied text drawn by lane-writable code and is trusted for nothing: the prompt, not the card, is what the owner judges, and even the prompt is only as trustworthy as T3 allows.

**A writable panel (not built) would need:** (1) Leg 1's G0 proving the browser-to-host RPC; (2) Leg 1's flag 2 ruling adding `elevate-enter` and `elevate-decide` to the write allowlist, recorded `channel: browser-peer, authenticated: 0`; (3) the same remote-authority gate on those writes; (4) a decision that a browser click may launch the entry program (the browser peer is owner-equivalent, and so is any uid-1000 holder of the launch token, so a click is no better than the CLI as a launch path: T3, T19); (5) Lane 6 review. None of these blocks v1.

### 8.2 CLI (also at the machine)

`dsh-fleet elevate list | show e7 | deny e7 | enter e7 | lock | unlock | verify e7 | proof [--probe-ticket]`. `enter` runs the same entry program; it reads no password in the terminal. These call owner ops over the control socket. **Those ops carry no authority:** the socket admits any uid-1000 peer that is not in `fleet-hands.service`, which includes every lane unless confined (case W). So `deny`, `lock`, `unlock`, clearing a suspension and `hand-back` are advisory against a lane; they are a convenience for the owner and an audit trail, not controls. v1.2 section 4.1 check 3 wants a polkit `auth_self` assertion for sensitive control ops; this install has no polkit agent and no polkit action, and adding either is a root install outside this leg. Those ops are therefore listed as residual risk (T13, T20).

### 8.3 Away from the machine

Leg 2's bus now exists, but nothing reads it for Nox, and the owner reads it only through `comms-inbox` (Leg 2 2.5). A request files one `notice` to `cayleb-local` (id, requester, 12-hex hash, expiry; never a password or a secret), idempotent on the request id. Until the Nox adapter N1 or a phone module exists that notice sits in a queue: it is not an away notification, and the app attention count and `dsh-fleet elevate list` remain the real signals. A remote approve or deny (Nox's ruling, section 9) needs a route that does not exist in this build: the facade is closed at fifteen tools (no elevate tool, section 7) and the phone module is deferred, so it arrives only as the owner op `elevate-decide` on the control socket (`authenticated 0`). Any other route is NEEDS-RULING item R4. The first draft's `arm` is replaced by the approve marker of section 9. A remote desktop session the owner already uses is a way to reach the same local prompt (8.4).

### 8.4 "Take control briefly"

A lease record, not a remote-access service and not a gate.

- **Take:** `dsh-fleet desktop take [--ttl 600]` (default 10 minutes, maximum 60). The broker writes a lease row and `control.taken`; `dsh-fleet desktop status` shows it (a banner on Leg 1's page needs a contract field and is deferred with the writable panel).
- **Hand back:** `dsh-fleet desktop hand-back`, or automatic at the TTL (a warning 60 seconds before). It writes `control.handed-back` with `by: owner|ttl` and the duration.
- **What it does not do:** nothing is gated. In this install no lane has a GUI tool, so there is no tool call to refuse. GUI work on the owner's desktop is done by `fleet-hands` for Nox and Cayleb (v1.2, not in this package) and by a lane's own shell (`hyprctl`, `grim`, `ydotool` as uid 1000), which a lease cannot stop. So the lease is an audit marker: it records when the owner took the desktop. It is independent of Leg 1's `present_until` (the owner-present signal): no wake path reads it, and the watchdog does not respect it (Leg 2 section 13.3). A guard code `owner-has-control` and a hands-side check of the lease are deferred until `fleet-hands` exists and can read it through a control op (build list item 13).
- **Remote:** v1 builds no transport and touches no network or tailscale setting. If the owner reaches his desktop with a remote-desktop tool he already uses, the password is typed into the same local prompt through that session. A purpose-built remote surface needs his exposure decision (he is already carrying open items on MeshCentral and Funnel) and is not part of this leg.

---

## 9. Authority and relayed approval (the one statement)

**This section is the single statement of the relayed-approval rule for all four legs. Legs 1 to 3 and the decision file cite it.** It replaces the first draft's recommendation, which dropped phone approval altogether, with Nox's ruling.

1. **The only authority** for root, and for a Tier B confirm, is the owner's password typed at this machine into the entry program; sudo and PAM verify it. Rows, buttons, taps, handoffs and relayed messages are routing and record.
2. **Nox's ruling:** a relayed phone approval may APPROVE or DENY an elevation request. **What that does:** a remote *approve* records `remote_decision = approved` with its channel and `authenticated` flag, extends the request's expiry to `elevate.approved-ms` (default 3 600 000, bounded; for a ticket-confirm request (12 A2) the extension is capped at the ticket's own expiry and never outlives it), and makes the machine-side prompt show "approved remotely at <time> via <channel>, unverified". It runs nothing, opens no prompt and changes no ticket, policy or order. A remote *deny* ends the request `denied` (fail-safe). The password is still typed on the machine and nothing runs without it. (This reading of "approve" is the reconciler's; decision file E1 asks Nox to confirm it.)
3. **Phone-side password entry** is an open owner pick: not built, not blocking. If the owner wants it, the minimum I would accept is a direct end-to-end channel from the phone's browser to the entry program (not via Nox, the bus or any relay), with a passkey and TLS the program terminates: a new network listener that needs his exposure decision.
4. **No bus message of any kind** confirms a Tier B ticket, a grant, a policy change, an order or an elevation. Leg 2 refuses kind `approval`, and no handoff text changes any of those rows. This keeps design v1.2 section 8 decision 4 in substance: no uid-1000 channel counts as authority, which is why every uid-only channel is recorded `authenticated:0`. It differs from v1.2 section 4.1 check 3 in one way: v1.2 gets a second factor from a polkit assertion on control ops, this design takes the control ops out of the authority path and leaves `lock`, `unlock` and clearing a suspension advisory (8.2). Flagged to Nox in the decision file.
5. **Duet.** Authenticated fleet handoffs to Duet carry the owner's authority by default, from the authenticated channel only (Leg 3 6.1). **That default authority never extends to approving an elevation request**, and a Duet message that says "approved" changes nothing here (Leg 3 checklist 16).
6. **Where a remote decision can come from in this build:** only the owner op `elevate-decide` on the control socket (`authenticated 0`). There is no facade tool (the facade is closed at fifteen, section 7); the phone module is deferred (section 8.3); a facade route would be NEEDS-RULING item R4.

**What the owner loses against Nox's first order:** he cannot clear a root step from his phone; the phone can only pre-approve or deny. **What he keeps:** no chat, bus, board or phone app ever holds the password; no secret travels through a model; the `id -u` proof and every later request work the same at the machine.

## 10. Audit events

Appended to the existing `events` chain (`core.appendEvent`) with kind `elevate.*`. **No event contains the password, its length, keystroke timing or any environment value.**

| Kind | When | Data |
|---|---|---|
| `elevate.request` | filed | id, argv, cwd, reason, agent, session, order, binding hash, exe sha256, classification, flags, expiry |
| `elevate.refused` | a filing was refused | agent, code, argv sha256 (not stored raw for `secret-in-argv`) |
| `elevate.flood` | a cap or cooldown fired | agent, rule |
| `elevate.remote-decision` | a relayed approve or deny | id, decision, channel, `authenticated` (section 9) |
| `elevate.denied` / `.withdrawn` / `.expired` / `.void` | those states | id, from-state, channel, `authenticated:0`, reason |
| `elevate.prompt-opened` | prompt shown | id, prompt id, opened-by (app/cli), entry pid |
| `elevate.prompt-closed` | submitted, cancelled, timed out, focus lost, crashed | id, why |
| `elevate.hardening` | entry preflight | dumpable, rlimit_core, mlock, nonewprivs, ptrace_scope, display identity (peer exe, cgroup match) |
| `elevate.entry` | password submitted | id, entry number (nothing else) |
| `elevate.spawned` | sudo started | id, argv sha256, exe sha256, sudo flags |
| `elevate.sudo-asked` | nonce prompt seen (or not) | id, asked true/false |
| `elevate.auth-failed` | nonce seen twice | id, entry number |
| `elevate.exited` | command ended | id, exit code, signal, duration, stdout/stderr sha256 and byte counts, output returned yes/no |
| `elevate.outcome` | final | id, result, source |
| `elevate.binding-mismatch`, `elevate.replay-refused` | refusals | id, detail |
| `control.taken` / `control.handed-back` | lease | actor, channel, ttl, duration, by |
| `elevate.unanchored` | verify found an outcome with no sudo journal line | id |
| `elevate.unrecorded-sudo` | verify found a successful sudo line with no outcome row | cursor, command (informational: includes the owner's own sudo use) |

**The anchor is sudo's own journal line, not ours.** A uid-1000 process can write any ordinary journal field and message (a `logger` or `systemd-cat` call, as `chainhead.js` itself does) but cannot choose `_EXE`, `_TRANSPORT` or `_UID`, which journald takes from the sender. `dsh-fleet elevate verify` therefore matches each `ok`, `command-failed` or `timeout` outcome against a journal entry with `_EXE=/usr/bin/sudo`, `SYSLOG_IDENTIFIER=sudo`, `_TRANSPORT=syslog`, within the outcome's time window, whose message is the anchored success form for exactly the approved argv (a failed-authentication line begins differently and cannot carry the success prefix). No match: `elevate.unanchored`. A lane could still run sudo itself to log a *failure* line, but not a success line without the password. Lines the entry program writes itself, and the chain head the broker writes, are **advisory**: a lane can write the same text. The chain hash has no secret, so a lane can append forged rows after the last head; the head anchor detects only accidental damage, not an adversary. That sudo logs success to journald in this exact shape is **[unverified]** (sudoers `log_*` settings are unreadable); G39 and the proof run P4/P5 check it on a real line.

---

## 11. Threat model

Attacker baseline: **case W of section 3**, a lane (or any process) running as uid 1000 in the owner's session, not root. F1 (lanes at uid 1000) is owner-accepted. Where case C holds, more rows close; G40 decides.

| # | Threat | What the attacker can do | Control | Residual |
|---|---|---|---|---|
| T1 | Same-uid ptrace and `/proc` memory | `ptrace` the entry program, read `/proc/<pid>/mem`, `process_vm_readv`, `environ` | H1: dumpable 0 makes `/proc/<pid>/*` root-owned and blocks attach at any `ptrace_scope`; scope 1 here also blocks non-descendants [verified]. Password never in argv or environment. | `cmdline` is world-readable and holds only non-secrets. |
| T2 | Pipe re-open | Open `/proc/<pid>/fd/N` of a pipe and read the password in flight | H2: socketpair. [verified: pipe+dumpable leaks; socket and non-dumpable do not.] | pinentry as a fallback must also be spawned over a socketpair. |
| T3 | **Fake or substituted prompt** | Draw a look-alike; or edit the app card, CLI, broker spawn or control-socket op so the owner's genuine button opens the lane's own program; or run the genuine program itself | Owner-opens-the-prompt rule and the footer stop an *unsolicited* fake. H11 stops a prompt drawn through a proxied or moved Wayland socket. | **Not closable at uid 1000.** The launch chain is lane-writable (T19), so pressing the genuine button does not prove the prompt is genuine. The owner can only notice a prompt he did not expect or a look he does not recognise. Upgrades: entry on a separate virtual console, or a hardware key. This is the main residual risk and the owner is told so before the first run. |
| T4 | Screenshot / computer-use capture | Read the prompt text and screen | The prompt holds only non-secrets; the password shows masked, no reveal toggle; `GTK_A11Y=none`. | A screenshot shows the argv, not the password. |
| T5 | Keystroke capture under Wayland/Hyprland | Read evdev; run a key hook; add a Hyprland bind per key through its IPC; **proxy the Wayland socket** and see every key sent to the surface | evdev is `root:input 0660` and uid 1000 is not in `input` [verified]. Wayland sends keys only to the focused surface. H6 clears the field on focus loss. **H11** rejects a socket whose peer is not the real compositor and uses the checked descriptor. | **[unverified]** Hyprland lets any uid-1000 process change binds at runtime through its IPC; a non-consuming bind per key may fire whatever window is focused unless the client holds a shortcut inhibitor. G17 measures this in a nested Hyprland. `/dev/uinput` lets a lane inject keys but not read them [verified]. |
| T6 | TOCTOU between approved text and executed argv | Change the argument, binary, cwd or environment between "approve" and "run" | One process holds one argv, shows it and runs it (5.2). Preflight at filing and again before spawn; root-protected binary only (H9). | A root-run package update in the gap is refused as `exe-changed`. An argument naming a user-writable file is flagged. |
| T7 | sudo timestamp caching | Ride a cached credential or leave one for a later lane | `-k` ignores and does not update the cache [verified, man page]. Nonce gate: if sudo does not ask, nothing is written. | sudo timestamps are per tty, or per parent process with no tty; the owner's terminal ticket is outside this design. P0 and P5 check what they can, in matching contexts. |
| T8 | Environment, config and library injection | Poison the user manager's environment, `PATH`, `GTK_*`, `PYTHON*`, `HOME`, `XDG_*`; restyle GTK through `~/.config/gtk-4.0` | H4: inherited environment ignored and rebuilt; HOME and XDG dirs pinned to a root-owned empty directory; theme and settings backend set; `python3 -I`; bootstrap re-exec if `LD_*` was inherited. Sudo gets exactly the scrub-v1 environment. | Files the toolkit reads outside those paths (system-wide files are root-owned). G21 and G37 test the known families and the hostile CSS case. |
| T9 | Argv visible in `/proc` and logs | Read every command line; read sudo's log lines | Only non-secret argv appears; `secret-in-argv` refuses credential flags. The password is never an argument. | The approved command appears in the system journal via sudo. Intended: it is the anchor. |
| T10 | Core dumps and swap | Read a core or swap page holding the password | H1: no core. H3: `mlock` where allowed, zeroing, short life. | `mlock` may fail (recorded). Python `str` copies live until exit. Swap is root-only. |
| T11 | Log, journal and scrollback leakage | Password in a log, terminal or trace | No event or log line carries the password, its length or timing. No terminal (H5). | A wrong-password retry is visible only as `auth-failed`. |
| T12 | Replay of an approval | Reuse an old approval | Single use by the compare-and-set to `running` and the state graph; one open prompt per request. No stored credential exists. | None beyond T3. |
| T13 | Request flooding and approval fatigue | Spam requests until the owner taps through | Caps in 5.6; no prompt ever self-opens; one coalesced card list; 5-second delay on unrecognized programs. | **The caps, cooldown, 3-denial suspension and `lock` are advisory against a shell-capable lane** (T20): it can clear its suspension and call `unlock`. A patient attacker can file a harmful command among harmless ones. The owner reading argv is the control. |
| T14 | Harmful command dressed as a harmless reason | Plausible reason, innocent-looking argv | The reason is labelled "not checked" and sits below the argv. Classification is the entry program's. Interpreters, shells and persistence paths are refused. | The tripwire list is incomplete by nature (`tee`, `cp`, `mv`, `chmod`, `systemctl enable` can all do harm). |
| T15 | Nox relay | A lane prompt-injects Nox into relaying "approve", or Nox misdescribes a request | Nothing relayed can open a prompt, run anything or replace the password; a relayed approve is a marker (section 9) and a relayed deny is fail-safe. | Nox could mislead the owner; the owner is told to trust the prompt only. |
| T16 | Audit tamper | Rewrite the database to hide a request or forge an outcome | Append-only triggers stop accidents. The outcome is a claim; `verify` checks it against sudo's own journal line (section 10). | A lane can append forged rows after the last head; the head is advisory. A lane can forge a journal line with any text but not `_EXE=/usr/bin/sudo`. If it holds the password it can run sudo itself, logged the same way; `unrecorded-sudo` shows it. |
| T17 | Confirms-directory mismatch and forgeable confirm file | See section 12 | Elevation does not use the confirms path; L4-A moves Tier B confirms onto the entry program and turns file confirms off once proved. | Open until L4-A is installed and proved; until then a file confirm is forgeable and recorded `authenticated 0`. |
| T18 | Phone approval vs decision 4 | See section 9 | Password typed only on the machine; no uid-1000 channel is authority; a remote approve is a marker. | Owner cannot clear a root step from his phone. |
| T19 | Entry code, launch path or install tampering | Edit the entry program; change what the app, CLI or broker launches; shadow a user unit | A+: the genuine `enter.py` and its siblings are root-owned and checked against `MANIFEST` by the program itself and by `elevate verify`; the manifest hash is in the journal at install. No systemd unit is involved, so there is nothing to shadow. | **Every launcher is lane-writable** (broker module, app, CLI, ops), so a lane can launch its own program instead (T3). A+ makes the genuine program tamper-evident, not the launch trustworthy. Before the owner's install the build copy is lane-writable and is not used for the real run. |
| T20 | Control socket and ops abuse | A lane calls `elevate-enter`, `deny`, `lock`, `unlock`, clear-suspension, `take`, `hand-back`, `elevate-fetch`, `elevate-outcome` | Rate-limited and logged `authenticated:0`. No op approves anything or supplies text the owner judges. | **Certain in case W:** the deny list is only `fleet-hands.service` [code], and lanes are in-process with the host, so every lane shell can call every op. Worst effects: an unsolicited prompt (T3), a griefing deny or lock, a forged outcome (a claim, section 10), an unlocked kill switch, a cleared suspension. Fail-safe, but not controls. |
| T21 | Shared facade token | A token holder reads or denies requests | There is no elevate facade tool, and the facade is closed at fifteen. | Only if Nox rules a named gap (R4/D4); until then none. |
| T22 | Root output reaching an agent | `cat` a root-only file and read it back | `return_output` is off by default, is stated in the prompt, is capped. There is no scan (5.5). | If the owner approves output for a root-read command, the requester sees it, including secrets and any encoded form. That is the request. |
| T23 | Password leaks into the command | A command that never prompted for a password receives one on stdin | Nonce-gated write; write end shut down; the command reads EOF (H10). | None known. |
| T24 | Clock tampering | Move time to extend a window | The broker's clock and the entry program's monotonic timer; uid 1000 cannot set the system clock. | Root-level change is out of scope. |
| T25 | Takeover abuse | A lane writes a false lease or hands back early | The lease gates nothing, so abuse gains nothing; it is logged. | A false lease record can mislead a reader of the audit. |
| T26 | Sudoers already NOPASSWD | The password is never needed | Nonce gate shows `no-prompt`; P0 tests it. | If sudo needs no password, every lane already has root; outside this leg and the owner should know. |
| T27 | Owner reuses his login password elsewhere | A captured password is worth more than this system | Not controllable here. | Stated so he can weigh it. |
| T28 | Timeout not enforced on a root child | A command outlives its limit | Root-owned `/usr/bin/timeout -k 5` is in the displayed, hashed argv; the entry program adds a best-effort SIGTERM to sudo. | Best effort: a daemonizing command escapes; whether uid 1000 may signal sudo's root child is unverified (P7). |
| T29 | A network-facing uid-1000 service as attacker | The Duet courier (and a relay session, or any later service that parses external content) runs as uid 1000: it can read the legacy bearer, open the control socket (every owner op, and the `elevate-*` ops) and substitute a prompt | Leg 3 5.6 runs the courier in its own mount and network namespace without the control socket, the DSH home, the legacy token directory or the display sockets, and tests it (Leg 3 checklist 1a, 1b); the `duet` token is `scope-denied` on every facade tool except its four bus calls, and no elevate tool exists (gate 43) | Not closed for an unsandboxed process or a relay session; case W stays the baseline. Joins the attacker population of section 3 |

---

## 12. The Tier B confirm path: findings, and item L4-A (in scope)

**Settled by Nox's ruling:** this leg owns the fix to the Tier B confirm wiring and the closure of the forge path. Every live Leg 2 grant (register, scope changes, token issue, add, rotate, and the `roster.*`/`comms.*` policy keys) and every Leg 3 token waits on it.

1. **Any lane can forge a Tier B confirm in the consolidated install.** `processConfirms` accepts a regular file owned by the host's uid whose JSON names the ticket and has `pkexec_uid` equal to the peer uid [code]. In the consolidated setup the host is uid 1000, peerUid is 1000, the confirms directory is mode 0700 owned by the same uid, and (case W) every lane shell is uid 1000. A lane that holds or can open a Tier B ticket can confirm it. [inferred from code plus F1; not run; in case C the sandbox mask may stop it, as `policy/NOTES.md:60-63` says.] The v1.2 protection (a separate `fleet` user behind a masked directory) does not exist in this install. A hand-placed confirm file with provenance `owner-direct-instruction` or `nox-manager-ruling-...` is a file, and a file proves nothing here.
2. **The CLI confirm path is broken, not just mismatched.** `dsh-fleet confirm` uses pkexec to write `/var/lib/fleet/state/confirms` as user `fleet`, while the consolidated poller reads `<DSH home>/fleet/state/confirms` and requires files owned by its own uid. Do not "fix" it by widening permissions; that widens finding 1.
3. **The facade token was shared.** Leg 2 gives every participant its own scoped token; the legacy bearer stays `allow` until the owner lowers it.

**L4-A: the build item (precedes every live Leg 2 grant and Leg 3 token).**

- **A1, provenance.** Record the true provenance and channel of every confirm in the grant row. Today the plugin hard-codes the approver `cayleb-local:polkit` and ignores extra confirm-file fields (`TIER-A-GRANTS-LEDGER.md`), so its own chain does not show how a hand-placed confirm was authorized.
- **A2, every ticket confirm through the entry program (Tier A and Tier B).** In the consolidated install `dsh-fleet confirm <ticket>` (the `confirm` case of `src/core/bin/dsh-fleet.mjs`) stops calling the pkexec helper. It files an elevation request for `["/usr/bin/true", "<t>:<first 12 hex of the ticket's args hash>"]`, where `<t>` is `ta` for a Tier A ticket and `tb` for a Tier B ticket, bound to the ticket, and the owner types the password into the entry program. The broker treats the ticket as confirmed only when `verify` finds sudo's own journal success line (`_EXE=/usr/bin/sudo`) for exactly that argv, then confirms it in process. The journal line is the proof; a file or an outcome claim is not.
  - **These are owner-filed confirm requests, not agent requests, and the agent caps of 5.6 do not apply to them.** They are exempt from `elevate.agent-per-hour`, `elevate.per-day-total`, `elevate.denied-cooldown-ms` and `elevate.suspend-after-denials`, because those throttle an agent that loops and the requester here is the owner's own CLI (`cayleb-local`, class `ticket-confirm`). What still applies: `elevate.pending-ms` (15 minutes, unanswered means expired), `elevate.max-pending-total`, the entry limits (`elevate.entries`, `prompts-per-request`) and the ticket's own expiry (a confirm request never outlives its ticket). A remote approve (section 9 item 2) may extend a ticket-confirm request's expiry from `elevate.pending-ms` to `elevate.approved-ms`, but the extension is capped at the ticket's expiry. A denied or expired confirm leaves the ticket `pending`; the owner re-runs `dsh-fleet confirm`, or voids the ticket with `ticket-void`. Leg 2's register, token issue and policy-key grants and Leg 3's A3-flip grant are filed this way, so they do not consume agent quota and cannot lock the owner out of his own grants.
- **A3, forge closure.** A policy key `policy.file-confirms` (default `on` until A2 is proved on the real machine). Flipping it `off` is itself a Tier B ticket confirmed through A2. After that `processConfirms` ignores confirm files in the consolidated install, for both tiers. **Tier A spend confirms after the flip** go through A2 like Tier B (a `ta:` marker), so the flip does not strand them; the flip is not offered to the owner until A2 is proved for both tiers on the real machine. Until then every file confirm is recorded `channel: confirm-file, authenticated: 0` and listed on the holds list, and hand-placed Tier A confirms exist only under Nox's explicit ruling (per-ticket or a bounded standing ruling, as ledger ticket 100006's o1-only ruling); no such ruling can be exercised after the flip.
- **A4.** Do not widen directory permissions; retire the pkexec helper path in the consolidated install.
- **Unverified:** that the policy confirm seam and the grant-row writer are where `policy/policy.js:253-296` suggests (read, not run); that the journal records the success line in the shape `verify` needs and that uid 1000 can read it (G39, P4, P5). L4-A touches `src/policy/policy.js` and `src/core/bin/dsh-fleet.mjs` (the `confirm` case, where the pkexec call to `/usr/local/bin/dsh-fleet-confirm` is, lines 94 to 130 and 120 on `release/0.3.4`), which the first draft's touchpoint list excluded. The `package/bin/dsh-fleet.mjs` shim is not where `confirm` lives. Gates 41 and 42.

---

## 13. The proof run (deferred to the owner at home)

**Deferred, by Nox's ruling 5 (no sudo and no privileged command during the build).** This is not run in this build and is not "for tonight". It is an owner-run step after the build and the Lane 6 gate, and it needs D1 (the one-time owner-run root install of the static files) first. Done-when item 4, "one real elevation request completes end-to-end", cannot close until the owner has run it; this build delivers it as authored, fake-sudo-tested and ready, and says so. Harmless on purpose: it runs `id -u` as root and expects `0`.

**Owner steps, at home, in order.**

- **P0, sudo prompt behaviour.** In his own terminal: `sudo -k` then `sudo -n true`; expect "a password is required" (else stop and tell Nox, T26). Then `sudo -k -p 'NONCE-TEST: ' true` and read the prompt text, then Ctrl-C. Expect the custom prompt text, which shows `-p` is honoured (a `passprompt_override` setting would break the nonce gate; the program would end `no-prompt`, which is safe but unusable). This is the only check of those sudoers settings available without reading `/etc/sudoers.d`.
- **P0b, baseline probe (read-only, run from a real lane's shell, G40).** `test -w ~/.config/systemd/user; test -S /run/user/1000/wayland-1; test -S /run/fleet/control.sock; ls /run/user`. All three `test`s true means case W; failures mean case C. Record the output.
- **P1, install the static files.** One command from the install kit, run by the owner under sudo, with the Wayland socket name and compositor path as arguments it prints first. It copies the files to the root-owned locations (0755/0644, owner root), creates the empty `home/` directory, prints each file's sha256, writes `MANIFEST`, and logs the manifest hash to the journal. No daemon, unit, setuid, sudoers or polkit change. Lane 6 reviews the kit before he runs it; the build never runs it.
- **P2, file the request.** A real lane calls `elevate_request` (or `dsh-fleet elevate proof` files it as agent `service:proof`):

  ```
  argv:          ["/usr/bin/id", "-u"]
  cwd:           "/"
  timeout_s:     10
  return_output: true
  reason:        "Owner Elevation end-to-end proof: print the effective uid under sudo"
  ```

  Expected: state `pending`, classification `KNOWN: id`, no risk flags, a read-only card on the activity page and a row in `dsh-fleet elevate list`.
- **P3, enter.** He runs `dsh-fleet elevate enter e<N>` himself. Compare the 12-character hash with the card. Check the prompt shows `RUN AS ROOT` and the lines `"/usr/bin/timeout"`, `"-k"`, `"5"`, `"10"`, `"/usr/bin/id"`, `"-u"`, cwd `/`, and the output warning. He types the password and presses Enter.
- **P4, expected result.** `stdout_tail` is `0`, exit code 0, `sudo_asked` true, `entries` 1, request `done`, outcome `ok`, `hardening` shows dumpable 0, core 0, no new-privs 0, display identity matched. The entry process is gone within 5 seconds.
- **P5, check.** `dsh-fleet elevate verify e<N>`: the outcome matches a journal entry with `_EXE=/usr/bin/sudo` and the success form for the approved argv (`unanchored` absent); no password-length or timing field exists; `coredumpctl list` shows no new entry. **Ticket probe:** `dsh-fleet elevate proof --probe-ticket` runs `sudo -n true` from a fresh no-tty process with the same parent as the entry program, after it has exited; expect "a password is required" (a `sudo -n true` from the owner's own terminal cannot see a ticket made without a tty, so it proves nothing and is not used). **Forgery probe (G39):** `logger -t sudo "x : COMMAND=/usr/bin/id -u"` then `verify`; the forged line must not anchor anything.
- **P6, negative probes (no password needed).** File `["/bin/sh","-c","id"]`: `interpreter-refused` at once. File the proof again and run `dsh-fleet elevate deny`: state `denied`, nothing runs. Optional: `dsh-fleet desktop take` with a 120-second lease, confirm `control.taken`, then `hand-back` and confirm `control.handed-back`; nothing is gated (8.4).
- **P7, optional timeout probe.** `["/usr/bin/sleep","30"]` with `timeout_s: 3`; second password entry. Expect outcome `timeout`, exit 124, the process gone. This is the only check that a root-owned child is stopped, because a fake-sudo test runs the child as uid 1000.

A fresh checker who did not build it reads the events, the outcome row, sudo's journal line and the install manifest, and records them as the evidence. If P3's prompt does not look right, the owner closes it and tells Nox; nothing runs without his typed password.

---

## 14. Build list, in order

Nothing in the build executes a privileged command. Tests use a fake sudo through the library's injected path; the entry point's sudo path is a constant. The install kit is authored and reviewed, not run. A Claude lane builds and runs the entry program, per the standing harness-executor rule; Lane 6 reviews. All of it happens in the worktree, never in the live consolidated install.

0. **L0, baseline probe.** G40 (P0b) run read-only from a real lane in the consolidated install by Lane 6 or the owner at the gate; it decides case W or C for all four legs. If lane shells sit inside the fleet unit's cgroup, decide whether to add that unit to `denyCgroupUnits` and how the app-launched entry program reaches the broker (section 3). The build never touches the live install.
A. **L4-A, the Tier B confirm path and forge closure** (section 12), built and tested early with a fake sudo and a fake journal; it precedes every live Leg 2 grant and Leg 3 token.
1. **Shared vectors** (`vectors/`): canonical-binding vectors (at least 30), `elevate-rules.json` (the exact interpreter, persistence, secret and catalog lists of 5.5), display-escape goldens; used by JS and Python tests.
2. **Migration and request store** (`src/elevate/requests.js`): the three tables, triggers, state graph, expiry overlay and timer, events. Mirrors `policy/tickets.js`.
3. **Validation and catalog** (JS): argv rules, classification, refusal codes, caps and cooldowns. Table-driven tests from the rules file.
4. **Agent tools:** `elevate_request`, `elevate_status`, `elevate_withdraw`, role gating.
5. **Entry library** (Python): H1 to H4, H11, binding recompute, exe check, fetch, start compare-and-set, socketpair sudo spawn with the nonce gate, the two-entry retry, the `timeout` wrapper, output capture, outcome report. Headless with an injected password source and display. Fake-sudo suite: normal, never prompts, echoes stdin, prompts twice, hangs, prints the canary, crashes.
6. **Prompt:** GTK 4 prompt (H6, H7), pinned environment, nested-Hyprland manual test script.
7. **Broker protocol:** owner ops `elevate-fetch`, `elevate-start`, `elevate-outcome`, `elevate-enter`, spawn of the entry program with explicit argv and empty environment, reconcile at boot. Registered with `core.registerControlOp` from the module, as `ledger` does.
8. **Owner ops and CLI:** list, show, deny, decide (the remote approve or deny marker, section 9), enter, lock, unlock, verify, proof, take, hand-back, status.
9. **Elevation data contract and read-only cards:** `fleetElevate.list()` (read-only: id, state, requester, age, expiry, binding12, classification, remote_decision) with fixtures; the cards are rendered by Leg 1's activity page, after Leg 1's G0 and host half (Leg 2 section 13.1 step 3). No password field, no button, no argv, no pill wiring (o1's).
10. **Takeover lease:** table, `take`, `hand-back`, `status`, TTL, events (audit marker only; no banner in v1).
11. **Audit:** the event kinds in section 10, `verify` with the sudo journal matcher and `unrecorded-sudo`.
12. **Install kit and proof script:** static file list, modes, `MANIFEST`, the P0 to P7 script. Authored, not run.
13. **Deferred, listed so they are not forgotten:** a facade route for remote elevation decisions, which is NEEDS-RULING (R4): `elevate.list`/`elevate.decide` are not tools, the facade is closed at fifteen, and a new tool needs a named gap and Nox's ruling first (the reader for Nox and the phone is Leg 2 gap 22); the Nox adapter reading the bus notice; a phone notice and decide; a writable panel (8.1); a `owner-has-control` guard code and a hands-side lease check (when `fleet-hands` exists); polkit `auth_self` for `unlock`/clear-suspension; console-switch entry; the C executor behind the same interface.

**Touchpoints outside `src/elevate/`** (the complete list; everything else is untouched):

1. `package.json` (export `./elevate`, `files`), edited in the integration commit (Leg 2 section 13.2).
2. `cordis.patch.yml` and `rows.manifest.json` (the plugin row), edited in the integration commit; the manifest is regenerated once.
3. `src/client/`: none of its own. The cards are Leg 1's page rendering this leg's data; the bundle is rebuilt once at integration.
4. `bin/dsh-fleet.mjs`, the 11-line shim: dispatch of the `elevate` and `desktop` subcommands to `src/elevate/cli.mjs`, the way it already dispatches `first-run`.
4a. `src/policy/policy.js` (L4-A: confirm path, grant provenance, the `policy.file-confirms` key).
4b. `src/core/bin/dsh-fleet.mjs` (L4-A: the `confirm` subcommand, which is where the pkexec call lives; the only core file this leg edits).
5. `presets/lead.patch.yml` and `presets/builder.patch.yml`, only if they enumerate tools **[unverified]**.
6. `package/NOTES.md` and docs; the install kit in a new `kit/elevate/` directory.

No edit to orders, roster, mcp or housekeeper. The two L4-A edits (4a, 4b) are the only edits to policy and core: control ops register through `core.registerControlOp`, policy keys are declared in the module, the expiry timer is the module's own.

Then the Lane 6 gate, then the owner's install and proof run.

---

## 15. Lane 6 gate checklist

Lane 6 is a hard gate before this ships. No template existed, so each item is something to run and record. "Canary" means a random string used as a stand-in password in tests; the real password is never used in any test.

**Password handling**

1. **No leak anywhere.** Run the full flow with fake sudo and a canary. Afterwards search byte-wise: the fleet database and WAL, every log file, `journalctl` (system and user) since test start, entry stdout/stderr, the test directory, `$XDG_RUNTIME_DIR`, `/tmp`, `/var/lib/systemd/coredump`. Zero hits. During the run sample `/proc/*/cmdline` and `/proc/*/environ` every 10 ms. Zero hits.
2. **Hardening report.** During entry the program's `/proc/<pid>` is owned by root:root (dumpable 0), `RLIMIT_CORE` is 0, `NoNewPrivs` is 0, and `elevate.hardening` says so. It refuses to start when `NoNewPrivs` is 1.
3. **Same-uid attack battery.** From a separate uid-1000 process that is not an ancestor, while the program holds the canary: `ptrace` attach and seize, read `/proc/<pid>/mem`, `process_vm_readv`, open every `/proc/<pid>/fd/N`, read `environ`, `maps`, `smaps`. All denied. Repeat with the attacker as a sibling child of the broker.
4. **Socketpair, not pipe.** `readlink` of the program's fds during entry shows `socket:` for the password path and no FIFO or pipe holding the secret. Re-run the pipe-re-open test from section 2 as a regression test.
5. **Nonce gating.** Fake sudo that never prompts: zero bytes written, result `no-prompt`. Fake sudo that echoes stdin to stdout: the canary is absent from the outcome. After the write, the command's stdin reads EOF.
6. **Wrong password.** Fake sudo that prompts twice per run: the first entry is written and the write end shut down; the program sees the nonce twice, accepts exactly one more entry in a new sudo process, never reaches a third prompt, the command never runs, and the second failure ends `bad-password`. A third entry is refused.
7. **No echo path.** A fake command that reads its stdin and prints it gets EOF and prints nothing; the canary is absent from the outcome and the database. (There is no output scan; the claim is that the command cannot reach the password.)
8. **Crash.** SIGSEGV and SIGABRT to the program mid-entry holding the canary: no core file, no `coredumpctl` entry, no canary in the coredump directory.
9. **Memory.** During entry `VmLck` covers the buffer or the event records `mlock=false` and the footer shows the note. The buffer is zero after the write, and no other copy is kept.
10. **Lifetime.** After every terminal state no entry process exists within 10 seconds.

**Prompt**

11. **No tty, no fallback.** With no display, result `no-secure-display`; nothing is read from stdin or a tty.
12. **Entry controls.** Paste, drag-drop and the context menu are disabled; no reveal toggle.
13. **Focus.** Simulated focus loss mid-typing clears the field; submit while inactive is ignored.
14. **Accessibility.** With the accessibility bus on, the prompt's tree holds no entry text; `GTK_A11Y=none` is set.
15. **Prompt limits (broker level).** The broker refuses a prompt for a request that is not `pending`, a fourth open for one request, a second open prompt (the unique index), or more than one open per 60 seconds. The entry program enforces only its own 120-second life and two entries; it has no trusted state and does not claim to enforce the broker's rules. A cancelled prompt leaves the request `pending` and does not count as an entry.
16. **Display fidelity.** Argv with control characters, bidirectional overrides, zero-width characters, newlines, non-ASCII and over-length content is refused or rendered as visible escapes; golden renders match in the card and the prompt.
17. **Keystroke capture probe.** In a **nested Hyprland instance** (not the owner's session), register non-consuming binds for every printable key through its IPC and type into the prompt. **Pass condition:** the result (captured or not, and whether a shortcut inhibitor stops it) is recorded in the evidence and the owner is told before the first real use. Capture working is not an automatic block.
37. **Display identity and pinned configuration.** (a) A fake compositor socket whose peer exe is not the configured compositor: result `display-identity-failed`, no window. (b) The socket moved or replaced between the check and GTK init: the connection used is the checked descriptor (`WAYLAND_SOCKET`), shown by the peer pid of the live connection. (c) A lane-run nested copy of the compositor binary fails the cgroup check. (d) A hostile `~/.config/gtk-4.0/gtk.css` and `settings.ini` that hide or recolour the argv label are present: the golden render is unchanged, because HOME and XDG dirs are pinned. (e) `GTK_THEME` and `GSETTINGS_BACKEND` set in the caller's environment do not change the render.

**Binding and execution**

18. **Hash vectors.** Changing any bound field changes the hash; the JS broker and the Python program agree on every shared vector.
38. **Forged row.** Change `elevation_requests.argv`, the classification and the stored hash in the database between `elevate-fetch` and `elevate-start`: the program runs and shows the argv it fetched, never a re-read one; no code path reads the request again after the display. A forged row *before* the fetch changes what is shown and what runs together, and the prompt shows the forged argv, not a stale one.
19. **Binary swap.** Replacing the target between filing and spawn gives `exe-changed` and no spawn. A binary or parent directory writable by uid 1000 is refused at filing.
20. **Argv rules.** Table-driven from `elevate-rules.json`: every program in the interpreter list, every persistence prefix (including `--opt=/etc/sudoers.d/x`, `..` traversal and the kit path), every refusal code in section 7, relative `argv[0]`, `VAR=value`, secret-flag patterns, NUL, limits. Each catalog entry yields its stated flags.
21. **Environment.** The fake sudo records its environment: exactly the scrub-v1 allowlist. With `LD_PRELOAD`, `PYTHONPATH`, `GI_TYPELIB_PATH`, `GTK_MODULES`, `HOME` and `XDG_CONFIG_HOME` hostile in the caller's environment, the entry program's rebuilt environment is unaffected and a canary library is not loaded into it after the re-exec.
22. **Launch path.** No systemd unit is used: with a user-level unit named like any elevation unit present in `~/.config/systemd/user`, nothing changes. The program refuses to run when a sibling file's sha256 differs from `MANIFEST` (`manifest-mismatch`, tested in the build tree). The test also records, as a stated limit and not a pass, that a lane-launched copy of a different program is not detected (T3, T19).
23. **Exact sudo line.** The fake records `/usr/bin/sudo -S -k -p <nonce> -- /usr/bin/timeout -k 5 <T> <argv>` and nothing else. The entry point has no environment or argument that changes the sudo path.
24. **Single use.** After `running` or `done`, a second `enter` is refused. Two concurrent `elevate-start` calls produce exactly one compare-and-set winner and one sudo spawn; two concurrent prompt opens produce one open prompt row (the loser is refused). A replayed start is refused with `elevate.replay-refused`.
25. **Expiry and timeout.** With an injected clock: pending expires at 15 minutes, a prompt at 120 seconds. If nobody acts every request ends `expired` and nothing runs; `lock` voids pending requests. **Timeout:** a fake-sudo run cannot show that a root-owned child is killed, because the fake child is uid 1000. Tested: the sudo line carries the `/usr/bin/timeout -k 5 <T>` prefix; the real `timeout` binary stops a uid-1000 child process group at T and exits 124; the entry program's fallback SIGTERM fires at T+15. The root-owned case is **real-run only** (P7).
26. **Caps.** Per-agent and total pending, hourly and daily rates, the denied-cooldown, three-denial suspension, each with its code. The test also records that a lane-origin `unlock` or clear-suspension call is accepted (advisory, T13).
27. **Classification.** A malicious broker that sends a false classification: the prompt shows the entry program's own.
28. **Output policy.** `return_output=false`: the agent sees exit code and hashes only. `true`: capped tails, and the prompt shows the output warning.

**Audit and surfaces**

29. **Events.** Every state change and prompt event in section 10 exists; none holds the password, its length or timing; `verifyChain` passes on an untouched chain.
30. **Anchor cross-check.** Each `ok`/`command-failed`/`timeout` outcome matches a sudo journal entry; an outcome with none is reported `unanchored`; a sudo success line with no outcome is reported `unrecorded-sudo`. Matcher unit tests use recorded journald JSON fixtures, including a failed-authentication line that must not match.
39. **Journal forgery.** A lane writes a journal line with `logger -t sudo` and with `systemd-cat -t sudo` carrying the exact success text for an approved argv, and a forged outcome row: `verify` still reports `unanchored`, because `_EXE` is not `/usr/bin/sudo`. The test is run against live journald in the proof run (P5).
31. **Who may do what.** A lane-origin call to `deny`, `lock`, `unlock`, `take`, `hand-back`, `elevate-outcome` is admitted and logged `authenticated:0` (case W); none of them can start a run, approve anything or change what a prompt shows. `elevate-enter` from a lane yields only a prompt. Relayed text saying "approve" does nothing.
32. **Takeover.** Take, hand back, TTL expiry all work; events and `desktop status` show it; no lane, window or app is closed, moved or paused. The lease gates no tool (stated in the evidence).
33. **No live-install effect.** The diff's file list equals this leg's touchpoint list in section 14 exactly (the shared files are excluded and checked once at integration, Leg 2 section 13.2; the `src/policy/policy.js` and `src/core/bin/dsh-fleet.mjs` changes of L4-A are on the list, and nothing else under core; nothing under orders, roster, mcp or housekeeper). The consolidated unit file, the confirms directory, the desktop profile and the owner's sessions are untouched, shown by hashing the unit file and listing the confirms directory before and after.
34. **No privileged execution in tests.** Run the whole suite under an exec trace (for example `strace -f -e trace=execve`) and show zero `execve` of `/usr/bin/sudo`.
35. **Install kit.** The file list is exactly the static files; modes are right; no setuid bit, no sudoers, polkit or PAM change, no unit, no `NoNewPrivileges`; the manifest hashes match the build; the kit prints what it will do before it does it.
36. **Real run.** The section 13 proof is run by the owner and checked by a fresh checker who did not build it; the evidence is attached to the board.
40. **Baseline probe.** The P0b probe is run from a real lane in the consolidated install and the output attached, deciding case W or C for sections 3 and 11 and for Legs 1 to 3.
41. **L4-A.** With a fake journal: a Tier B ticket confirms only on the anchored success line for exactly its `tb:` argv, and a Tier A spend ticket only for exactly its `ta:` argv; a forged `logger -t sudo` line does not confirm either; an owner-filed confirm request is not counted against the agent caps, the cooldown or the suspension (A2); a confirm file before `policy.file-confirms` is flipped is recorded `channel: confirm-file, authenticated: 0`; after the flip a lane-forged confirm file confirms nothing; the grant row records the true provenance; directory permissions are unchanged.
42. **L4-A flip.** Flipping `policy.file-confirms` off is refused without a ticket confirmed through A2.
43. **Remote decision and the courier.** A relayed approve runs nothing, opens no prompt, extends the expiry and shows the unverified mark; a deny ends the request; both record channel and `authenticated`. A `duet` message saying "approved" changes no request (with Leg 3 checklist 16). No `elevate-*` op is reachable from the sandboxed courier and the `duet` token is `scope-denied` on every facade tool except its four bus calls and on every `board.op` op; there is no elevate facade tool (with Leg 3 checklist 1a, 7).

---

## 16. Limits and what is unverified

- Nothing here has been built or run beyond the read-only checks and the pipe/socket test in section 2. The gate items are the proof, not this document.
- **Whether lanes are confined (case C)** is unverified (G40). The design assumes case W.
- **sudoers settings** are unreadable: `passprompt_override`, `use_pty`, `passwd_tries`, `timestamp_type`, `requiretty`, `NOPASSWD` and `log_*` are unknown. They affect the nonce gate, the single-write behaviour, the ticket probe and the journal anchor. P0, P4 and P5 test what they can.
- That sudo logs a success line to journald in the exact shape the matcher expects is unverified (one sudo record was seen; only its field names were read).
- Whether the entry program (uid 1000) may signal sudo's root child after the command starts is unverified (P7).
- Hyprland's runtime key-bind IPC as a keystroke logger, and whether the prompt can hold a shortcut inhibitor, are unverified (T5, G17). Whether a compositor peer's cgroup reliably identifies the owner's session scope, and whether `WAYLAND_SOCKET` is honoured by GTK 4's Wayland backend here, are unverified (G37).
- The substituted prompt (T3) is not solvable at uid 1000. The honest upgrades are a console-switch entry or a hardware key; neither is proposed for v1.
- GTK 4 is present for PyGObject. I did not try to open a window.
- No notification path exists to an away owner in 0.3.3 (section 8.3).
- Python cannot wipe immutable string copies. The mitigations are non-dumpable, short life and `mlock`; a C entry program is stronger if Lane 6 asks.
- The exclusions in 5.5 are a tripwire, not a fence. The owner reading the argv is the control.
- L4-A rests on seams read at `policy/policy.js:253-296` and not run, and on the journal shape of G39.

---

## 17. Review record

Every point of the critique was checked against the code or the machine. Changes made: no systemd unit and no descriptor hand-off (entry program fetches by id and treats the database as untrusted); the hash guarantee restated; the request lifecycle rebuilt (prompts are rows, `entering` and `armed` and the grants table are gone, one compare-and-set to `running`, two entries via two sudo processes); `arm` removed; the Wayland proxy and GTK configuration added (H4, H11); lane reach of owner ops stated as certain in case W and every control-socket channel recorded `authenticated:0`; takeover reduced to an audit marker; the away path and facade tools marked unbuilt; G33 given an exact touchpoint list; the output scan dropped; the timeout moved to a root-owned `timeout` wrapper; the audit anchor moved to sudo's own journal line; P5 and P0 rewritten and the sudoers unknowns listed; the baseline made explicit with a probe (G40); rule lists made exact.

**Reconciliation, 2 Nov.** Changes made after the cross-design critique: the approval statement is now the single section 9 following Nox's ruling; the confirm-path fix and forge closure are in scope (L4-A, gates 41 to 43); the panel is read-only and the false browser-path assumption is corrected (8.1); the facade is fifteen tools and this leg still adds none; the away path is a labelled queue; the courier joins the attacker population (T29); G33 and the touchpoint list follow Leg 2 section 13.2. Every Leg 4 assumption about the other legs is now stated against what Leg 2 built (b10e35f).

**Disputed, in part.**

- "The attacker baseline is assumed, not checked" and "lane shells can certainly call every owner op": the code facts are right (deny list is only `fleet-hands.service`) and are now stated as certain *for case W*. But fleet-sandbox puts `/run` on an empty tmpfs and makes `/home` read-only (`sandbox/NOTES.md`), and the roster wraps engine lanes in it (`roster/engines.js:102`), so the baseline may be case C for some lanes. It cannot be settled from code alone; the design stays on case W and G40 decides.
- "`systemd-run --user` with a pinned ExecStart" as the launch fix: not adopted. A transient unit is started over the same lane-reachable user manager, so it fixes shadowing but not substitution, and the root-owned file launched by absolute path with a rebuilt environment gives the same result with no D-Bus client. The critic's central point, that no uid-1000 launch path is trustworthy, is accepted and stated in section 1.
