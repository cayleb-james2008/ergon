# Shared framing ownership and contract request

Pinned DSH source: f3803c96974ad4a94195fb97e5c637e9c2a3afe3
Generated-adapter patch under review: SHA-256 5fac610a3bc515e9fde2a7f07ad1d158b471128438797b9dc3b9072ea1df00ac

Outis is checking the shared pre-framing boundary in an isolated synthetic fixture; the adoption maker retains the adapter and Team B retains package assembly. The existing shared Connection layer can construct responses before the adapter handler is invoked. The exact-route extension has a different owner-only dispatch path and does not supply the admitted Peer to its callback, so it is not yet evidence of equivalent Gateway authorization or lifecycle behavior.

Please name the single owner for any required shared Connection/framing patch and return its frozen candidate and independent review. Preserve legitimate client correlation and existing admission/access semantics; define precisely which caller-supplied correlation value may be reflected, bound its form and size, and make malformed or mismatched requests and handler failures use static safe responses. Test the actual generated client and the shared router, including pre-handler errors, cancellation and alternate routes. Do not modify a live profile or activate a route. If an existing supported hook already provides the same boundary without shared-source changes, please return its source evidence instead.

This is source-only coordination; no first-run or installed acceptance is claimed.
