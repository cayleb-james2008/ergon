# Lane 2 selected-adapter source handback

The selected-adapter development patch SHA-256 is
`318d5c814da942f00fe4c2cf252ed8ade53d657b3208c5733053f1c29373e00c`.
Independent review passes its bounded source and synthetic-fixture scope under
the trusted-host assumptions. The maker's fresh run passed 125 cases; the
documentation-only successor's independent rerun covered the 21 selected
adapter cases and the three earlier independent checks. The other 101 cases
remain supported by the earlier source review, not a new rerun.

The three required independent checks each passed:

1. Asynchronous duplicate completion preserves separate handle custody.
2. A 4,096-byte UTF-8 target is accepted; oversized and malformed strings are
   refused.
3. Throwing cleanup during synchronous stop faults the adapter, does not
   retry the handle and refuses a later call.

Authentication dependencies were synthetic; descriptor operations used owned
scratch files. Other composition ports remained fake. Real signal provenance,
reviewed binding and descriptor custody, interruption of a blocked synchronous
filesystem call, and installed-output acceptance are not established.

The principal has been asked whether an isolated non-candidate scratch
installation may satisfy the first installed-output leg before candidate
integration, and what proof would bind it to the later candidate. The actual
fleet-installed pass remains separate. Until that sequencing decision and its
scope are resolved, do not install or integrate this patch; Stage 8 stays open.
