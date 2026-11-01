#!/usr/bin/bash
# CHECKER harness (independent of the wrapper's own test): simulated-pnpm normalise test.
# Runs: (1) main: 104-file archive, masked modes + nlink=2, dirs as pnpm leaves them 0750
#       (2) variant: same but dirs start 0755 -> normalise must set 0750
#       (3) negative A: archive package.json name != @cayleb/dsh-fleet -> fail closed, tree untouched
#       (4) negative B: one member missing from installed root -> fail closed
# Every case runs the EXTRACTED heredoc block as its own process, exactly as the wrapper does.
set -uo pipefail
EV=/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/wrapper-review
TGZ=/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz
BLOCK=$EV/normalise-heredoc.py
PY=/usr/bin/python3
W=/tmp/checker-wrapper-review.$$
mkdir -p "$W"
mkdir -p "$EV/logs"
echo "workdir=$W" | tee "$EV/logs/00-workdir.txt"
echo "archive sha256: $(/usr/bin/sha256sum "$TGZ")"
echo "block sha256:   $(/usr/bin/sha256sum "$BLOCK")"

run_block() { # $1 label, $2 archive, $3 root/package
  local label=$1 archive=$2 pkg=$3 rc
  "$PY" -I -S -B "$BLOCK" "$archive" "$pkg" >"$EV/logs/$label.stdout" 2>"$EV/logs/$label.stderr"
  rc=$?
  echo "$rc" > "$EV/logs/$label.rc"
  return 0
}

echo
echo "############ CASE 1: MAIN — 0.3.0, 104 members, masked+hardlinked, dirs 0750 ############"
"$PY" "$EV/sim-pnpm.py" "$TGZ" "$W/case1/root" "$W/case1/store" 0750 | tee "$EV/logs/10-sim-main.txt"
echo "-- pre-state spot check (nlink/mode of 3 members) --" | tee -a "$EV/logs/10-sim-main.txt"
for f in package.json bin/dsh-fleet.mjs src/core/core.js; do
  stat -c '%A nlink=%h %n' "$W/case1/root/package/$f" | tee -a "$EV/logs/10-sim-main.txt"
done
echo "-- run extracted normalise block --" | tee -a "$EV/logs/10-sim-main.txt"
start=$(date +%s%N)
run_block main "$TGZ" "$W/case1/root/package"
end=$(date +%s%N)
echo "block rc=$(cat "$EV/logs/main.rc") (dur $(( (end-start)/1000000 )) ms)" | tee -a "$EV/logs/10-sim-main.txt"
echo "stdout: $(cat "$EV/logs/main.stdout")" | tee -a "$EV/logs/10-sim-main.txt"
echo "stderr: [$(cat "$EV/logs/main.stderr")]" | tee -a "$EV/logs/10-sim-main.txt"
"$PY" "$EV/assert-normalised.py" "$TGZ" "$W/case1/root/package" > "$EV/logs/11-assert-main.txt" 2>&1
echo "assert rc=$?" >> "$EV/logs/11-assert-main.txt"
cat "$EV/logs/11-assert-main.txt"

echo
echo "############ CASE 2: VARIANT — dirs start 0755 -> normalise must set 0750 ############"
"$PY" "$EV/sim-pnpm.py" "$TGZ" "$W/case2/root" "$W/case2/store" 0755 | tee "$EV/logs/12-sim-variant.txt"
run_block variant "$TGZ" "$W/case2/root/package"
echo "block rc=$(cat "$EV/logs/variant.rc")" | tee -a "$EV/logs/12-sim-variant.txt"
echo "stdout: $(cat "$EV/logs/variant.stdout")" | tee -a "$EV/logs/12-sim-variant.txt"
"$PY" "$EV/assert-normalised.py" "$TGZ" "$W/case2/root/package" >> "$EV/logs/12-sim-variant.txt" 2>&1
echo "assert rc=$?" >> "$EV/logs/12-sim-variant.txt"
cat "$EV/logs/12-sim-variant.txt"

echo
echo "############ CASE 3: NEGATIVE A — archive package name changed -> fail closed ############"
"$PY" "$EV/mk-bad-archive.py" "$TGZ" "$W/badname.tgz" "evil-package" | tee "$EV/logs/13-neg-bad-name.txt"
"$PY" "$EV/sim-pnpm.py" "$TGZ" "$W/case3/root" "$W/case3/store" 0750 >> "$EV/logs/13-neg-bad-name.txt"
# manifest of the installed tree before, to prove the failure left it untouched
"$PY" - "$W/case3/root/package" > "$W/case3.before" <<'PY'
import hashlib, os, stat, sys
pkg = sys.argv[1]
for dp, _d, fns in sorted(os.walk(pkg)):
    for fn in sorted(fns):
        p = os.path.join(dp, fn); st = os.lstat(p)
        print(os.path.relpath(p, pkg), f"{stat.S_IMODE(st.st_mode):04o}", st.st_nlink,
              hashlib.sha256(open(p,'rb').read()).hexdigest())
PY
run_block neg-bad-name "$W/badname.tgz" "$W/case3/root/package"
"$PY" - "$W/case3/root/package" > "$W/case3.after" <<'PY'
import hashlib, os, stat, sys
pkg = sys.argv[1]
for dp, _d, fns in sorted(os.walk(pkg)):
    for fn in sorted(fns):
        p = os.path.join(dp, fn); st = os.lstat(p)
        print(os.path.relpath(p, pkg), f"{stat.S_IMODE(st.st_mode):04o}", st.st_nlink,
              hashlib.sha256(open(p,'rb').read()).hexdigest())
PY
{
  echo "block rc=$(cat "$EV/logs/neg-bad-name.rc")"
  echo "stdout: [$(cat "$EV/logs/neg-bad-name.stdout")]"
  echo "stderr: $(cat "$EV/logs/neg-bad-name.stderr")"
  if cmp -s "$W/case3.before" "$W/case3.after"; then echo "installed tree UNCHANGED by the refusal: YES"; else echo "installed tree UNCHANGED by the refusal: NO"; fi
} | tee -a "$EV/logs/13-neg-bad-name.txt"

echo
echo "############ CASE 4: NEGATIVE B — one member missing from installed root -> fail closed ############"
"$PY" "$EV/sim-pnpm.py" "$TGZ" "$W/case4/root" "$W/case4/store" 0750 | tee "$EV/logs/14-neg-missing-member.txt"
rm -f "$W/case4/root/package/src/core/core.js"
echo "removed from installed root: package/src/core/core.js" | tee -a "$EV/logs/14-neg-missing-member.txt"
run_block neg-missing "$TGZ" "$W/case4/root/package"
{
  echo "block rc=$(cat "$EV/logs/neg-missing.rc")"
  echo "stdout: [$(cat "$EV/logs/neg-missing.stdout")]"
  echo "stderr: $(cat "$EV/logs/neg-missing.stderr")"
} | tee -a "$EV/logs/14-neg-missing-member.txt"

echo
echo "############ EXPECTATION GATE ############"
FAILED=0
chk() { # $1 label, $2 rc, $3 condition-result (0 ok)
  if [ "$3" = 0 ]; then echo "EXPECT PASS  $1 (rc=$2)"; else echo "EXPECT FAIL  $1 (rc=$2)"; FAILED=1; fi
}
[ "$(cat "$EV/logs/main.rc")" = 0 ]; chk "case1 main rc==0" "$(cat "$EV/logs/main.rc")" $?
[ "$(tail -1 "$EV/logs/11-assert-main.txt" | grep -c 'assert rc=0')" = 1 ]; chk "case1 assert rc==0" 0 $?
[ "$(cat "$EV/logs/variant.rc")" = 0 ]; chk "case2 variant rc==0" "$(cat "$EV/logs/variant.rc")" $?
grep -q 'dirs_wrong_mode=0' "$EV/logs/12-sim-variant.txt"; chk "case2 dirs 0750 + assert clean" "$(cat "$EV/logs/variant.rc")" $?
[ "$(cat "$EV/logs/neg-bad-name.rc")" != 0 ]; chk "case3 bad-name rc!=0" "$(cat "$EV/logs/neg-bad-name.rc")" $?
grep -q "archive package name is 'evil-package', not '@cayleb/dsh-fleet'" "$EV/logs/neg-bad-name.stderr"; chk "case3 message exact" 0 $?
grep -q 'installed tree UNCHANGED by the refusal: YES' "$EV/logs/13-neg-bad-name.txt"; chk "case3 tree untouched" 0 $?
[ "$(cat "$EV/logs/neg-missing.rc")" != 0 ]; chk "case4 missing-member rc!=0" "$(cat "$EV/logs/neg-missing.rc")" $?
grep -q 'normalise: missing member src/core/core.js' "$EV/logs/neg-missing.stderr"; chk "case4 message exact" 0 $?
echo "EXPECTATION GATE: $([ "$FAILED" = 0 ] && echo ALL-PASS || echo SOME-FAILED)"

echo
echo "############ SUMMARY ############"
{
  echo "case1 main        rc=$(cat "$EV/logs/main.rc")            stdout=$(cat "$EV/logs/main.stdout")"
  echo "case2 variant     rc=$(cat "$EV/logs/variant.rc")         stdout=$(cat "$EV/logs/variant.stdout")"
  echo "case3 bad-name    rc=$(cat "$EV/logs/neg-bad-name.rc")    stderr=$(cat "$EV/logs/neg-bad-name.stderr")"
  echo "case4 missing     rc=$(cat "$EV/logs/neg-missing.rc")     stderr=$(cat "$EV/logs/neg-missing.stderr")"
} | tee "$EV/logs/15-summary.txt"
