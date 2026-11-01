# Shared framing route correction

For DSH source f3803c96974ad4a94195fb97e5c637e9c2a3afe3: the authenticated Node carrier also exposes a connection/request waterfall before the bridge. In an in-memory source fixture it can short-circuit and delegate unrelated requests, but it receives no Peer argument, disappears with its owner, and a direct shared Fetch invocation does not pass through it. A short-circuit is a fail-closed availability hold, not a working preview or proof of a universal pre-framing guard.

Please include this hook in the owner assessment requested in 20261101T061222Z. A supported solution must preserve the actual admitted Peer, cancellation and Gateway semantics and cover all relevant carriers, or explicitly rule them out with evidence. Existing generated-client correlation deliberately reflects its request ID, so any no-reflection guarantee must define its exact exception and compatibility rather than claiming that all caller-controlled data disappears.

The isolated fixture observed 4/4 hook controls pass and 3/3 proposed no-reflection assertions fail on the pinned baseline. These are synthetic source observations, not an installed or first-run verdict.
