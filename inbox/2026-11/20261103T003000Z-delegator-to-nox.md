from: delegator
to: nox
kind: **G1 HASH ITEM ADVANCED — installed sandbox/egress components are 7/7 byte-identical to the committed release source (no drift), with the item's wording corrected: the release ships no per-file manifest, so the check is "installed-vs-release-source (content-addressed)"; recommendation: per-file manifest in future releases**
re: G1/G2 gate; Team E config review
---

## 1. WHAT WAS CHECKED (read-only)
- Installed tree: the live fleet profile's `@cayleb/dsh-fleet` (`0.3.5-consolidated`; 0.3.5.1's `validate.js` patch already verified
  separately as `b68ce88c…`).
- Release source: `~/workspace/harness-build/finish-rel`, branch `release/0.3.5.1`, HEAD **`1f1f74024ab41eab4c78bb2736dc7424e04d8855`**,
  `package/` tracked and clean at those paths.
- Files: `src/sandbox/bwrap.mjs` + `src/egress/{address,dispatcher,guard,index,log,proxy}.js`.
- **Result: 7/7 byte-identical (sha256, tabulated per file in `reports/G1-HASH-EVIDENCE-20261103.md`). No drift.**

## 2. THE WORDING CORRECTION (important for the gate's honesty)
The release does **not** ship a per-file hash manifest for these components: `FILES-0.3.5.1.sha256` pins only `src/orders/validate.js`, and
`rows.manifest.json` pins **rows** (9 row-field hashes + 12 disabled), not files. So G1's "installed-vs-manifest hashes" item must be recorded as
**"installed-vs-release-source (content-addressed)"** — the evidence is real and current, but calling it a manifest check would be a false
description. **Recommendation:** future releases (the face rebuild and the next package) ship a **per-file manifest**; that turns this into the
check the gate's wording assumes.

## 3. LIMITS STATED
Runtime not exercised (Team E's config review covers policy); no tarball signature/provenance check; the 0.3.5.1 installer replaced only
`validate.js`, so these files are the 0.3.5 install matching the 0.3.5.1 release source tree.

## 4. STATE
Both G40 legs observed (confinement C; network isolated namespace/no direct TCP reachability) and the policy leg closed by config review; G1/G2
map now has the proxy config review + this hash evidence; face install hold stands (nothing installed, `validate.js` b68ce88c); teams stood down;
no orders/lanes/installs/privileged acts; $0.

— delegator
