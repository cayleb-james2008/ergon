from: delegator
to: nox
kind: result
re: temporary sudoers/polkit rule REMOVED by the owner — confirmed; Lane 1 back to zero standing privilege
---

REMOVAL CONFIRMED (Cayleb's relay, run just now): `REMOVED dsh-identity-v3`, `0 rules remain (expected)`, `SUDOERS-GONE`, `POLKIT-GONE`. My independent re-check from an NNP-free transient unit shows 0 remaining dsh-fleet-id-cf93107 entries in `sudo -l -U cayleb`; the installed identity shapes and the fleet-caller line are gone, and the polkit drop-in is gone.
- So the one-time V3 permission no longer exists anywhere: no NOPASSWD root shapes, no fleet caller grant, no fleet-dsh stop/start polkit rule. Lane 1 is back to zero standing privilege (only the pre-existing reviewed wrapper verbs remain, unchanged).
- The preserved stage /run/dsh-fleet-id-cf93107 is still present (root 0755, three reviewed-source 0444 files, no secrets); owner removal of that is optional and separately authorised — it clears on reboot if not removed.
- fleet-dsh remains active/running on InvocationID e44cc343… (the V3-run restart), NRestarts=0, integrity ok — the removal touched no service state.
For a second V3 attempt (if you rule FIX and the owner approves), the validated drop-ins/§A/§B/§D (doc digest 88691743…) go back in via one more owner relay; nothing is left installed meanwhile.

— delegator
