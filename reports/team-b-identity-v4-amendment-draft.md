# IDENTITY V4 AMENDMENT — DRAFT (policy alternative to the installer fix), for Nox's one-read decision

Filed 2026-11-01T16:08Z by `fleet-lead` (Team B, Lane 1). **Draft only — not applied, not staged, not run.**
This is the slower alternative to the installer-fix proof (`board/probes/team-b-installer-mode-fix-proof-20261101T1607Z.md`).
Nox chooses; Team B executes neither without a new approval.

## 1. The contract text that would change

Current candidate contract (as implemented by the V3 caller):
- every installed file is a **single-link regular file**;
- each file's mode **equals its tar member mode exactly** (`archive_mode`), mode & 0o7777, exact equality;
- ownership `951:950`; every package directory `0750`; archive bytes exact.

**Proposed v4 text (mode leg only):**

> File modes are compared against the archive member mode **normalised by the documented installer umask**:
> `expected_mode = archive_mode & ~0o027`. A file is accepted if its mode equals `expected_mode`, or is **more
> restrictive** than the archive mode in a way that does not add group/other write, setuid/setgid, or execute
> bits absent from the archive. Any mode that **widens** access relative to the archive is refused.
> The single-link requirement, the `951:950` ownership requirement, the `0750` directory policy and the exact
> bytes comparison are **unchanged**.

Naming of the residual (required in the acceptance record): *"installed file mode bits are the archive's modes
masked by the installer's umask (0644→0640, 0755→0750); content, ownership, directory policy and single-link
layout are unaffected; the masking never widens access."*

## 2. What it does not do

- It **does not** accept hardlinked files: the single-link requirement stays. (If the installer leaves hardlinks
  — observed in scratch, pnpm store clones — v4 still fails and the installer must de-link; that part of the
  repair is not optional under either option.)
- It does not accept content differences, ownership differences, or writable directories.
- It does not re-define the stage, no-writer or runtime-identity legs.

## 3. Consequences and trade-offs (for Nox)

| | **FIX (preferred)** — normalisation step in the wrapper | **AMEND** — v4 mode leg as above |
|---|---|---|
| Identity claim strength | archive-identical modes survive — strongest | narrowed: modes are umask-masked (documented residual) |
| Work needed | wrapper change + review + owner approval + wrapper reinstall + release install act | new caller/procedure bytes + independent review + owner approval (no wrapper change) |
| Failure mode addressed | root cause removed at source; deterministic for every install | comparison relaxed; the same class of masking is accepted permanently |
| Hardlinks | fixed by the de-link step | **not** fixed — de-link still required separately |
| Reviewer position | consistent with the reviewer's "installer contract is the defect" finding | satisfies the 05:36Z correction (policy is a candidate the checker assesses) but must be an explicit Nox decision, never post-hoc on this run |
| Time to second run | longer (wrapper + release act) | shorter if hardlinks are already single-link in the real tree |

## 4. If Nox chooses AMEND — exact deliverables (not started)

1. v4 caller/procedure bytes with the comparison above; new pinned SHA-256s.
2. Fresh independent review (the v3 reviewer may review v4 but cannot be the formal installed-output checker).
3. New owner approval (one invocation of v4) — the v3 approval is consumed.
4. New operator relay of the narrow sudoers/polkit drop-ins (§A/§B of the command block, digest `88691743…`) —
   the current install was removed by the Lead as instructed.
5. Acceptance record carries the residual wording in §1 verbatim.

## 5. Recommendation

**FIX, not AMEND** — the normalisation proof shows the strong claim is recoverable at the installer layer, and it
also removes the hardlink class; AMEND would permanently relax a check whose failure mode (mode masking) can hide
a real widening if ever combined with a different store/install behaviour. Choose AMEND only if the wrapper change
cannot be reviewed/approved in the available window.
