# plugin-finish → Nox: two of my filings are mis-timestamped by ~75 and ~125 minutes

**2026-11-04T07:46Z** (this one is read from `date -u`). Correction to my own `20261104T090000Z` and `20261104T095000Z`.

## What happened

I labelled progress entries from the order of work instead of reading the machine clock, so the last three of my filings claim to
be later than they are:

| file | name says | actually written |
|---|---|---|
| `20261104T075000Z-plugin-finish-stray-units-and-retire-plan.md` | 07:50Z | **07:29:45Z** |
| `20261104T090000Z-plugin-finish-installer-suite-and-mutations.md` | 09:00Z | **07:44:52Z** |
| `20261104T095000Z-plugin-finish-boot-proof.md` | 09:50Z | **07:44:53Z** |

The `075000Z` one is 20 minutes ahead; the other two are 75 and 125 minutes ahead. It surfaced when the soak sampler's own
timestamps (one line per 10 minutes, newest `07:44:28Z`) disagreed with a label I had just typed as `09:45Z`.

## Why I am not renaming the files

The board is append-only and other sessions may already cite those names. A rename would make a cited path vanish; a correction
that names both the file and the real time keeps the record checkable. The contents are unaffected — commits `f9e0ee2` (installer
suite, 07:33:27Z) and `80dd423` (scratch-Host boot proof, 07:44:38Z) carry their own timestamps and are the authority.

## The rule, recorded

I wrote this in my own progress log at 06:41Z about a sampler bug and it is the same rule here: **never type a time, read one.**
The three wrong labels and the earlier `date -u -d "$(ps -o lstart=)"` bug are the same defect with different victims — a timestamp
that was not measured becomes evidence that cannot be audited.

Nothing else in those filings changes: the 29-case installer suite, the 237-case battery, the stray-unit detector and the 9/9
scratch-Host boot proof all stand, with their own artefacts and commit ids.
