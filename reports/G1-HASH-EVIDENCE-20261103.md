# G1 HASH EVIDENCE — installed sandbox/egress files vs the release source (Lead, 2 Nov 19:30 ET)

Release worktree: `~/workspace/harness-build/finish-rel`, branch `release/0.3.5.1`, HEAD **`1f1f74024ab41eab4c78bb2736dc7424e04d8855`**, `package/` tracked and clean
(no modified paths under `package/src/egress` or `package/src/sandbox`). Installed tree: the live fleet profile's `@cayleb/dsh-fleet`
(`0.3.5-consolidated`, with 0.3.5.1's validate.js patch already verified separately).

| file | installed sha256 | release-source sha256 | identical |
|---|---|---|---|
| `src/sandbox/bwrap.mjs` | `7018f00252a16b16…` | `7018f00252a16b16…` | **YES** |
| `src/egress/address.js` | `3dcde26d08c7e81c…` | `3dcde26d08c7e81c…` | **YES** |
| `src/egress/dispatcher.js` | `218ee38601a7e8a8…` | `218ee38601a7e8a8…` | **YES** |
| `src/egress/guard.js` | `9d28ab8a31952ce3…` | `9d28ab8a31952ce3…` | **YES** |
| `src/egress/index.js` | `6bea2d9e6028eb27…` | `6bea2d9e6028eb27…` | **YES** |
| `src/egress/log.js` | `7a927e713c22ffdf…` | `7a927e713c22ffdf…` | **YES** |
| `src/egress/proxy.js` | `e8e808ba68a011f0…` | `e8e808ba68a011f0…` | **YES** |

**Result: 7/7 byte-identical.** The installed sandbox/egress components are the release source, verified by content, not by a claim.

## Limits (stated, not hidden)
1. This compares the installed tree with the release **source tree**; the release does **not** ship a per-file hash manifest for these
   components — `FILES-0.3.5.1.sha256` pins only `src/orders/validate.js`, and `rows.manifest.json` pins **rows** (9 row-field hashes + 12
   disabled), not files. G1's "installed-vs-manifest" item should therefore be recorded as **installed-vs-release-source (content-addressed)**, or
   the release should start shipping a per-file manifest (recommended for the next package).
2. Runtime not exercised here (Team E's config review covers policy; a runtime probe would be a separate throwaway order).
3. No tarball signature / provenance check was performed in this evidence item.
4. The 0.3.5.1 installer replaced only `validate.js`; these components are from the 0.3.5 install and match the 0.3.5.1 release source tree.

## Verdict
**G1's hash item: PASS in substance** for the sandbox/egress components, with the wording corrected (content-addressed comparison to the
committed release source). No drift found.

— Lead (record-keeping, read-only), 19:30 ET