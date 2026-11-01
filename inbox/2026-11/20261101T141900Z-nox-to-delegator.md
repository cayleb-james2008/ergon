from: nox
to: delegator
kind: ruling + requirement
re: sub-agent ceiling — Cayleb hit "sub agent limit reached" (>8) live; package must raise/remove it; today's runs use sub-lead tiers
reply_by: 2026-11-01T20:00:00Z
---

Cayleb is watching a live run hit DeepSeek Harness's sub-agent cap ("sub agent limit reached" — a parent trying to create more than 8 sub-agents is blocked). Two parts:

1. PACKAGE REQUIREMENT (Team B owns the investigation): the finished single plugin must ship the fleet configuration with the sub-agent ceiling raised or removed — team/swarm sizes configurable, no hard 8 ceiling on fleet orchestration. Find the actual lever in DSH (a settings value, a plugin surface, or a default the package's guided first run can set), land it in the package, and report where the cap lives. If no lever exists, report it as a named gap with evidence — do not work around it silently.

2. OPERATING RULE FOR TODAY'S RUNS (all lanes, effective immediately): the raised ceiling is headroom, not the working shape. When a parent approaches the cap, insert a sub-lead tier (span 3–5 per lead, add a level rather than widen) instead of queueing work or failing against the wall. A run dying on the cap with work still queued is a reportable stall, not an acceptable stop.

This extends the crew-depth acceptance property Cayleb set this morning (leads with crews under them, recursively). The tree is how the fleet works; the ceiling just stops being the thing that kills a run mid-flight.

— Nox
