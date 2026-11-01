# OP-02 — Finish the plugin: Stages 3–9 into ONE installable package

State: Stages 0–1 closed; Stage 2 closing under OP-01 on the installed 0.2.2. Remaining build (library-first per the verified gap-fill map — adopt/wrap before custom; minimalism is an acceptance property: thin harness, dense instructions, delete what stronger models make redundant):
- Stage 3: security/permissions core + the guided first run (fleet user, hardened unit, tailnet bindings). Cayleb's confirmed terminal pass is his act (99) — everything before and after it is yours: package the run, rehearse it, make his ten minutes the only ten minutes.
- Stage 4/4b: providers — Anthropic (Haiku 5.5) with caps; engine providers for Claude-engine and Codex-engine lanes. Codex credential = Cayleb's ChatGPT login, entered by him in person at the 4b gate (decided); the build proceeds credential-less until then. Subscription engines + OpenRouter + NVIDIA NIM + OpenCode Go/Zen in scope per Cayleb's list; DeepSeek V4.1 Flash via OpenCode Go is the volume default.
- Stage 5: fleet bus live (leased claims, tailnet-only) + remote web UI (tailnet-only) + MCP facade. This is the layer that lets the agents talk to each other without Cayleb handoffs.
- Stage 6/7: Duet digest + messaging bridge + phone surface (Telegram wrapper; BotFather token is Cayleb's later act) + fleet activity view in-harness (session ownership badges + sidebar dashboard) + phone VM and Windows VM as SIDEBAR PANELS in DSH (adopted: ZSeven-W/dsh-android; pureexe/dsh-windows-remote-ssh wrapped) — the rehearsal drives phone flows through that panel.
- Stage 8: hands module (the Nox desktop endpoint absorbed): token auth, full GUI control, local action log, local kill switch.
- Stage 9: close-out — injection drill, Harness² shared-library export, witnessed end-to-end proof.

Acceptance property (Cayleb, binding): the package BUNDLES every adopted plugin it relies on; first run inventories his existing local plugins and presents each as absorb-or-remove (nothing silently deleted); credentials entered once, inside the first run; the plugin ships no secrets. A needed capability living outside the single package at the end is a defect.

Done when: each stage passes its own fresh-checker gate on the installed package, and the single-package acceptance property is demonstrated by a clean install run. Work stage by stage; OP-01's closure is the base, not a prerequisite for Stage 3+ prep work — prep in parallel, install in order.
