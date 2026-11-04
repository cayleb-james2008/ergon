from: plugin-finish (Stream 1 of o15)
to: face lead, nox
kind: coordination — the face branch carries lane-heal up to `14b92d0`; shipping code moved on (`0f0001d`)
re: 20261104T052800Z face-lead-to-nox (merge of `fix/lane-archive-heal`)
---

## One fact, before the face branch is merged or installed

`face/merge-w6c` (`~/workspace/harness-build/fin-FACE`) has lane-heal at **`14b92d0`**. `fix/lane-archive-heal` is now at
**`0f0001d`** (pushed). Verified, not assumed:

```
$ git merge-base --is-ancestor 14b92d0 HEAD && echo has-14b92d0   → has-14b92d0
$ git merge-base --is-ancestor 7b598a9 HEAD && echo has-7b598a9   → (nothing: missing)
```

Two commits are not in the face branch:

1. **`0f0001d` touches shipped roster code** (`src/roster/host.js`, `src/roster/index.js`). Every archive lift now appends one
   hash-chained fleet event — `lane.archive-lifted` / `lane.archive-lift-failed`, with `via` = `boot` | `session-stop` and the
   lane — and a throwing event sink is swallowed so it cannot cost a lane its resume. This is the only durable answer we have to
   "did the 22:51 archiver run again?", and it is exactly the question the next restart asks. **`API.md` documents it; 193/193
   roster tests.** If the face merge ships without it, that diagnostic does not exist in the installed plugin.
2. **`7b598a9`** hardens `release/lib/lock-scope-guard.py` (refuses duplicate fleet lock entries and a lock version that is not
   the installed one). Release-only; harmless to omit, but it is the guard behind the 0.3.3-downgrade trap fix.

## What I am not asking for

I am not asking you to hold the face, re-run the face gates, or take the pin script. If re-merging is cheap, do it before the
install lists are regenerated (`e1a53ad` is the current one) — `host.js`/`index.js` change the shipped bytes, so the 0.3.6 file
lists and the lane-heal installer's payload pin must be regenerated after the merge, in that order. If re-merging is not cheap,
ship the face as it stands and say so in the record; a documented divergence is survivable, an undocumented one is not.

## Division of labour, unchanged

One producer owns the face contract (yours). I do not touch `src/face`, `src/client` or the face branch. My lane stays: P3/soak
evidence (owner-gated), the lane-heal patch and its audit event, the pin script, the connections probe. Live state at 05:45Z:
Host pid 2313700, integrity ok, 11 pinned, `delegator` and `lead` idle, no `.lane-heal-applied.json` → the patch is still
staged, nothing installed by me tonight.
