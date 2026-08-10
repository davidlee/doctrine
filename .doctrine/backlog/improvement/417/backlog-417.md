# IMP-417: Third admission outcome for unavailable backend

Give `doctrine-control backend verify` a **third** top-level outcome, distinct
from `Admitted` and from `NotAdmitted`, for the case where the host cannot give
the backend what the rows need.

This implements the owner's ruling on `RV-352` `F-8`, taken 2026-08-10.

## The problem it settles

`PHASE-10`'s `EX-14` forbids an availability condition, an `#[ignore]`, a skip or
an early return reaching `Admitted` — tables A and B may not skip, because
`Admission` is computed from them and `DEC-156` proves admission by
one-property-removed controls. A row that did not run has proved nothing, and a
green skip is the vacuous pass moved to the top of the stack.

`DEC-180` settles the local host: the suite may red the tree, unconditionally,
here. **CI was left open.** A runner that cannot satisfy the rows makes the suite
red for a reason that is not a conformance defect, and every available reflex is
one `EX-14` forbids.

`RV-352` walked into the concrete instance: an audit jail built without this
slice's `flake.nix` addition could not resolve `setsid`, seven
`conformance::tests` convicted, and `backend verify` returned `not-admitted` on
`Property(ProcessTreeTeardown)`. Correct behaviour — and indistinguishable, from
the outside, from a genuinely broken confinement.

## The shape chosen

Of the three shapes `notes.md` § *Open* weighed, the owner accepted the second:
**make backend unavailability a distinct non-`Admitted` verdict.**

It is the only one that keeps the answer inside the type `admission()` already
returns. The row vocabulary already carries three values (`Proven`, `Unproven`,
`Indeterminate`) while the admission verdict carries two, so the information
exists and is being discarded at the fold. Restoring it adds no skip, no
`#[ignore]`, and no availability condition on any test — `EX-14` is satisfied by
construction rather than by promise, because the new outcome has no path to
`Admitted`.

The two shapes not taken: gating the suite to runners that declare the capability
and treating absence-of-run as a pipeline-level red (keeps the answer outside the
type, and depends on CI configuration staying correct); and accepting the red and
fixing the runner (does not scale past one runner, and leaves the two failure
kinds indistinguishable).

## The trap this must not fall into

**Do not derive the new outcome from row-level `Indeterminate`.**

`ISS-334` records four row-B5 sibling tests that return
`Indeterminate { arm: Probe, detail: NoObservation }` under CPU contention — a
genuine fixture race, correctly reported. If the third outcome is folded from
`Indeterminate`, that race becomes "the host could not establish this", reads as
benign, and the vacuous pass is rebuilt one level up with a new name.

The discrimination belongs at **precondition** level: a host-capability probe
that runs *before* the rows and convicts loudly, feeding the new outcome from its
own result. Row verdicts stay what they are.

The `S4` guard in `conformance.rs` is the existing precedent for the convict-loudly
posture and the model to follow.

## The first concrete probe, supplied by `ISS-339`'s off-jail run

`ISS-339` was done first as advised, and it produced one member of the probe set
rather than a hypothetical:

> **A readable root must cover the literal path a payload execs.**

The first off-jail run failed every row `Indeterminate { NoLiveness }` because
the derived bind set carried `/nix` but not `/bin`, so the capsule had no
`/bin/sh`. The derivation defect is fixed on `sl-248` (`8f837c312`), but the
*reporting* defect is this item's: a host structurally unable to start a payload
said nothing about the missing mount and let nineteen rows read as indeterminate.

This is the exact confusion the trap above warns of, arriving from the other
direction — a genuine host precondition wearing row-level `Indeterminate`'s
clothes. It is also why the check has to be **structural**, not existential:
`admission`'s current guard asks `host.path_exists(SHELL)`, which follows the
symlink and returns true on precisely the host that cannot exec it. The probe
must instead ask whether `SHELL` falls within a derived readable root —
`is_within` against `system_readable_roots`, both of which already exist.

Sizing note: the fix that made this pass was four lines in one function. What
remains here is the loud precondition, not the mount.

## Related

`ISS-339` — the off-jail run. Advised first; **done**, and it paid out (above).
`just capsule-verify` (added on `sl-248`) is the invocation. It stays open until
a real host is seen to *admit* the backend — a second precondition may sit behind
the first, and each one found is another probe member for this item.

## References

`SL-248` `notes.md` § *Open* (`EX-14` in CI) · `PHASE-10` `EX-14` · `DEC-156` ·
`DEC-180` · `sec-9` residual 3 · `RV-352` `F-8` · `ISS-334` · `ISS-339`
