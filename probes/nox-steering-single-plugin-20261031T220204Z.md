from: nox
to: delegator
kind: steering
re: single-plugin acceptance property (Cayleb, 31 Oct 17:59 ET)
---

Cayleb restated the deliverable bar to me directly, and it sharpens the finish definition for Jobs 06–11. Record it as an acceptance property of the finished plugin:

**ONE finished, fully merged, single plugin.** Cayleb deletes every other plugin he has installed, installs `@cayleb/dsh-fleet` alone, and it works — no other setup. Concretely, this means the package must:

1. **Bundle the adopted plugins** the design already relies on (remote web UI layer, messaging bridge, session archive manager, subscriptions/engine providers, sage-mem store, Telegram surface base, API dashboard meters, and the two device-control adoptions from my 31 Oct sweep: `ZSeven-W/dsh-android` for the emulator rig, `pureexe/dsh-windows-remote-ssh` wrapped for the Windows VM guest). One install unit — nothing Cayleb must fetch separately.
2. **Ship a guided first run** that performs the machine-level setup a plugin zip cannot contain — fleet user, hardened unit, tailnet bindings — as one confirmed pass (the Stage 3 terminal run is that pass; it is Cayleb's only setup act besides entering credentials).
3. **Absorb or retire his current local plugins deliberately.** First run inventories what exists in his DSH profile — including the desktop-app session's goal-runner plugin and English translation forks under `~/.local/opt/deepseek-harness-official/local-plugins/`, and the stray community DSH Tauri 0.25.2 app — and presents each as absorb-into-package or remove. Nothing is silently deleted; nothing the fleet needs is left as a separate install.
4. **Credentials stay Cayleb-entered, once, inside that first run** (API keys, subscription logins, bot tokens). The plugin never ships with secrets and never asks twice.

5. **DSH's own defaults are NOT packaged.** What ships with DeepSeek Harness itself stays shipped with it — never duplicated into `@cayleb/dsh-fleet`: sub-agents, web search, agent loop, shell, agent teams, voice input, custom plugin indexes, change request, worktrees, personalised recommendations, conversation following, titles, Codex sub-agents, Claude Code sub-agents, DSH badge skill. The package contains only the fleet layer the defaults don't provide. Three defaults Cayleb does not use and the fleet must not depend on: machine translation of reasoning, auto authorisation review, and the DSH badge skill.

What this does NOT change: the stage gates, the maker–checker rule, or the witnessed 9.3 proof. It changes the packaging target: if a capability the fleet needs lives outside the single package at the end, that is a defect against this property, reported as such.

— Nox
