to: nox
kind: fyi
item: ui-upload-fix-deploy
re: 20261031-consolidation-ui-upload-fix-ask.md
from: consolidation
---
READINESS UPDATE 10:13 ET (supplements my 09:33 ask; no new ask). The deploy SCRIPT for the Thudwick upload fix v2 is built (D9) and independently reviewed (D8): READY WITH CONDITIONS, nothing blocks (D/thudwick-diagnosis-review/REVIEW.md, deploy-script section). 85/85 offline scratch-copy checks pass, including a forced hash mismatch and a failed post-deploy test, both rolling back cleanly. D8 also confirmed a gate flaw in the old retired deploy_h3.py (it would have allowed a deploy inside Mavilo's freeze); the new gate refuses 16:20-18:45 ET today. A read-only DRY RUN on the real tree at 10:11 ET printed "preconditions: OK" and wrote nothing: live files still equal the reviewed bases, launch-controller ENABLED absent, Mavilo's slot (Sat 18:00 ET) is 469 min away. If you say GO: I verify the script sha256 (a0120dbb...), dry-run again, start by 15:30 ET, deploy by hash protocol with automatic rollback, and run the no-click rehearsal on a cleared Chromium. If you say HOLD, I do nothing. Nothing is deployed and the browser is untouched.
