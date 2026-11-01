set -euo pipefail
[ "$(id -u)" = 1000 ] || { echo "run as cayleb (uid 1000)"; exit 1; }

# ── 1. verify the reviewed wrapper bytes before anything root-owned changes
SRC=/home/cayleb/workspace/harness-build/scratch-tb-l1/proposed-dsh-fleet-admin
WANT=eb0ad02480282b9b89341beb991ce38e4b6ffcace5c755d945dbcf6f0f4fb888
[ "$(sha256sum "$SRC" | cut -d' ' -f1)" = "$WANT" ] || { echo "WRAPPER HASH MISMATCH — STOP"; exit 1; }
bash -n "$SRC"

# ── 2. install the fixed wrapper (root:root 0755) and verify
sudo /usr/bin/install -o root -g root -m 0755 "$SRC" /usr/local/bin/dsh-fleet-admin
[ "$(sha256sum /usr/local/bin/dsh-fleet-admin | cut -d' ' -f1)" = "$WANT" ] || { echo "INSTALLED HASH MISMATCH — STOP"; exit 1; }
echo "WRAPPER OK $WANT"

# ── 3. the approved one-time drop-ins (bytes unchanged, digest 88691743…)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cat > "$T/dsh-identity-v3" <<'SUDOERS'
# dsh-identity-v3 — ONE-TIME V3 identity run, owner-approved 2026-11-01 (Nox 151700Z).
Cmnd_Alias DSH_ID_V3_STAGE = \
  /usr/bin/mkdir -m 0700 -- /run/dsh-fleet-id-cf93107, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/caller.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/identity_snapshot_core.py conv=excl status=none, \
  /usr/bin/dd of=/run/dsh-fleet-id-cf93107/manifest.txt conv=excl status=none, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/chmod 0444 -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/stat -c %F|%u|%g|%a|%h -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/caller.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/identity_snapshot_core.py, \
  /usr/bin/sha256sum -- /run/dsh-fleet-id-cf93107/manifest.txt, \
  /usr/bin/chmod 0755 -- /run/dsh-fleet-id-cf93107
cayleb ALL=(root) NOPASSWD: DSH_ID_V3_STAGE
cayleb ALL=(fleet) NOPASSWD: /usr/bin/env -i PATH\=/usr/bin\:/bin LC_ALL\=C /usr/bin/python3 -I -S -B /run/dsh-fleet-id-cf93107/caller.py
SUDOERS
cat > "$T/49-dsh-identity-v3.rules" <<'POLKIT'
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        subject.user == "cayleb" && subject.local && subject.active &&
        (action.lookup("unit") == "fleet-dsh.service" || action.lookup("unit") == "fleet-dsh") &&
        (action.lookup("verb") == "stop" || action.lookup("verb") == "start")) {
        return polkit.Result.YES;
    }
});
POLKIT
/usr/bin/visudo -cf "$T/dsh-identity-v3"                       # validate BEFORE installing
sudo /usr/bin/install -o root -g root -m 0440 "$T/dsh-identity-v3" /etc/sudoers.d/dsh-identity-v3
sudo /usr/bin/install -o root -g root -m 0644 "$T/49-dsh-identity-v3.rules" /etc/polkit-1/rules.d/49-dsh-identity-v3.rules
sudo /usr/bin/visudo -cf /etc/sudoers.d/dsh-identity-v3        # re-validate installed file
sudo /usr/bin/visudo -c >/dev/null                              # whole sudo config still parses
echo "INSTALLED dsh-identity-v3 (sudoers 0440 + polkit 0644)"
echo "DONE — wrapper + drop-ins installed; nothing has run yet"
