#!/usr/bin/bash
# Harness cleanup phase 2: remove D1–D7 (ungated DELETE set) per INVENTORY.md.
# Archive, never hard-delete (ASSUMPTIONS 21:22; Nox ruling 21:36). Runs as cayleb, no sudo.
set -uo pipefail
export PATH=/home/cayleb/.local/share/mise/shims:/home/cayleb/.local/share/mise/installs/node/26.7.0/bin:/usr/bin
A=/home/cayleb/Work/_archive/harness-removal-20261030
H=/home/cayleb
TS=20261030
mkdir -p "$A" && chmod 700 "$A"
log() { printf '%s %s\n' "$(date +%H:%M:%S)" "$*"; }
arch() { local n=$1; shift; local p r
  for p in "$@"; do
    if [ ! -e "$p" ] && [ ! -L "$p" ]; then log "MISSING $p"; continue; fi
    r=${p#$H/}; mkdir -p "$A/$n/$(dirname "$r")"
    if [ -e "$A/$n/$r" ] || [ -L "$A/$n/$r" ]; then log "SKIP (already archived) $p"; continue; fi
    if mv -n -- "$p" "$A/$n/$r"; then log "ARCHIVED $p -> $A/$n/$r"; else log "ERROR mv $p (exit $?)"; fi
  done; }
bak() { [ -e "$1" ] && [ ! -e "$1.bak-$TS" ] && cp -a -- "$1" "$1.bak-$TS" && log "BACKUP $1.bak-$TS"; }

log "== backups of config files edited below"
bak $H/.bashrc; bak $H/.config/mise/config.toml
bak $H/.config/omarchy/plugins/cayleb.agents/manifest.json; bak $H/.config/omarchy/extensions/omarchy-menu.jsonc

log "== D1 Gemini CLI + Antigravity (agy)"
mise uninstall --all gemini 2>&1 | sed 's/^/   /' || log "ERROR mise uninstall gemini"
sed -i '/^gemini = { version = "latest"/d' $H/.config/mise/config.toml && log "mise config: gemini line removed"
mise exec node@26.7.0 -- npm uninstall -g @google/gemini-cli 2>&1 | sed 's/^/   /' || log "ERROR npm uninstall gemini-cli"
arch gemini-antigravity $H/.local/bin/agy $H/.local/bin/agy.1792436412526340068.old $H/.local/bin/antigravity \
  $H/.local/opt/Antigravity $H/.config/Antigravity $H/.gemini \
  $H/.local/share/applications/antigravity.desktop $H/.local/share/icons/hicolor/512x512/apps/antigravity.png \
  $H/.local/bin/omarchy-agent-usage-antigravity $H/.cache/mise/gemini
/usr/bin/python3 -I - <<'PY' && log "omarchy agents manifest: antigravity provider removed"
import json; p='/home/cayleb/.config/omarchy/plugins/cayleb.agents/manifest.json'
d=json.load(open(p))
def strip(o):
    if isinstance(o, dict):
        o.pop('antigravity', None)
        for v in o.values(): strip(v)
    elif isinstance(o, list):
        for v in o: strip(v)
strip(d)
if isinstance(d.get('description'), str): d['description']=d['description'].replace('Antigravity, ','')
json.dump(d, open(p,'w'), indent=2); open(p,'a').write('\n')
PY
sed -i 's/"aliases": \["usage", "agent-usage", "ai-usage", "antigravity", "ollama"\]/"aliases": ["usage", "agent-usage", "ai-usage", "ollama"]/; s/"Claude, Codex, Antigravity, Ollama telemetry"/"Claude, Codex, Ollama telemetry"/' \
  $H/.config/omarchy/extensions/omarchy-menu.jsonc && log "omarchy menu: antigravity alias/description removed"

log "== D2 OpenClaw"
mise exec node@26.7.0 -- npm uninstall -g openclaw 2>&1 | sed 's/^/   /' || log "ERROR npm uninstall openclaw"
arch openclaw $H/.openclaw $H/.cache/openclaw $H/.winvm-openclaw

log "== D3 Freebuff"
arch freebuff $H/Applications/freebuff $H/.config/Freebuff $H/.config/freebuff-desktop $H/.freebuff $H/.config/manicode \
  "$H/.cache/@codebufffreebuff-desktop-updater" $H/.local/bin/freebuff-desktop $H/.local/share/applications/freebuff.desktop \
  $H/.knowledge.md $H/.agents/mcp.json.before-freebuff-20261017

log "== D4 Grok CLI (not the Grok Bot app)"
arch grok-cli $H/.grok $H/.local/bin/grok $H/.local/bin/agent
sed -i '/^# >>> grok installer >>>$/,/^# <<< grok installer <<<$/d' $H/.bashrc && log ".bashrc: grok installer block removed"

log "== D5 OpenMausBot"
arch openmausbot $H/.local/opt/openmausbot-0.1.102

log "== D6 OpenWork remnant"
arch openwork $H/.config/com.differentai.openwork $H/.config/@openwork

log "== D7 oh-my-pi + mise agent remnants (omp is never executed)"
arch omp $H/.local/bin/omp $H/.local/bin/omp.bak-20260918-192028 $H/.omp $H/.cache/mise/pi \
  $H/.cache/mise/github-can1357-oh-my-pi $H/.cache/mise/cursor-agent

log "== finish"
mise reshim 2>&1 | sed 's/^/   /'
update-desktop-database $H/.local/share/applications 2>&1 | sed 's/^/   /' || true
log "archive size: $(du -sh "$A" 2>/dev/null | cut -f1)"
