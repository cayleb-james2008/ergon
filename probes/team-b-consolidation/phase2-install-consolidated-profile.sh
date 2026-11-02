#!/usr/bin/bash
# PHASE 2 (LIVE, GATED) — create the sibling `fleet` profile under the owner's DSH home and install 0.3.1.
# NOT RUN YET: requires (1) Nox ruling on acceptance delta + "sibling fleet profile = one setup",
# (2) Lead approval. Writes ONLY: <desktop-home>/profiles/fleet/** (new dir) and nothing in profiles/desktop.
set -euo pipefail
DESKTOP_HOME=${DESKTOP_HOME:-$HOME/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home}
CLI=${CLI:-$HOME/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js}
TGZ=${TGZ:-$HOME/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz}
TGZ_SHA=5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4
FLEET_ROOT="$DESKTOP_HOME/fleet"
RUNDIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/fleet"
[ -f "$TGZ" ] || { echo "missing artifact $TGZ"; exit 2; }
[ "$(sha256sum "$TGZ" | cut -d' ' -f1)" = "$TGZ_SHA" ] || { echo "artifact sha mismatch"; exit 2; }
[ -e "$DESKTOP_HOME/profiles/fleet" ] && { echo "profile fleet already exists — refusing"; exit 2; }
# 0. record (hash-only) the live profile + credential store; never read values
sha256sum "$DESKTOP_HOME/profiles/desktop/cordis.patch.yml" "$DESKTOP_HOME/profiles/desktop/package.json" > /tmp/phase2-before.sha256
stat -c '%n mode=%a size=%s' "$DESKTOP_HOME/.credentials.yaml" >> /tmp/phase2-before.sha256
# 1. create the profile from the shipped web template (parity with the proven 0.3.1 composition)
DSH_HOME="$DESKTOP_HOME" node "$CLI" fleet --from-default-profile web --dump-config > /tmp/phase2-init-dump.yml
# 2. install the bundle (appends @cayleb/dsh-fleet to dsh.profile.bundles)
DSH_HOME="$DESKTOP_HOME" node "$CLI" plugin --profile fleet add --save-exact --ignore-scripts "$TGZ"
# 3. write the path overlay (NON-pinned rows only; fleet-mcp is integrity-pinned and needs the consolidated build)
mkdir -p "$RUNDIR"
# (recovery key for housekeeper backups: generated once, owner-level, never printed)
[ -f "$FLEET_ROOT/recovery.key" ] || { mkdir -p "$FLEET_ROOT"; age-keygen -o "$FLEET_ROOT/recovery.key" 2>/dev/null; age-keygen -y "$FLEET_ROOT/recovery.key" > "$FLEET_ROOT/recovery.pub"; }
sed -e "s#__FLEET_ROOT__#$FLEET_ROOT#g" -e "s#__RUNDIR__#$RUNDIR#g" \
    -e "s#__DESKTOP_HOME__#$DESKTOP_HOME#g" -e "s#__RECOVERY_PUB__#$FLEET_ROOT/recovery.pub#g" \
    "$HOME/Documents/deepseek-harness/default-workspace/board/probes/team-b-consolidation/profile-overlay.yml" \
    > "$DESKTOP_HOME/profiles/fleet/cordis.patch.yml"
# 4. composition check ONLY (no start)
DSH_HOME="$DESKTOP_HOME" node "$CLI" --profile fleet --dump-config > /tmp/phase2-dump.yml
grep -q 'id: fleet-roster' /tmp/phase2-dump.yml && echo "fleet-roster row composes"
grep -q 'maxActiveSubagents: 32' /tmp/phase2-dump.yml && echo "subagent 32 composes"
grep -q 'apiKeyEnv: OPENCODE_GO_API_KEY' /tmp/phase2-dump.yml && echo "opencode-go route composes (name only)"
# 5. package identity vs the tarball (extract to a temp dir; compare; no live write)
T=$(mktemp -d); tar -xzf "$TGZ" -C "$T"; INST="$DESKTOP_HOME/profiles/fleet/node_modules/@cayleb/dsh-fleet"
diff -rq "$T/package" "$INST" >/tmp/phase2-tree-diff.txt && echo "installed tree identical to tarball"
sha256sum "$INST/rows.manifest.json"   # expect 1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba
rm -rf "$T"
echo "NEXT: stop here (dump-config only). Phase 3 cutover is separate and needs Lead approval."
