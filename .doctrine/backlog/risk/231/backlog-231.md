# RSK-231: Capsule programme has exceeded its complexity budget

Capsule-based provisioning was conceived as a way to **shed** complexity —
simplify the delivery infrastructure while raising the level of containment
guaranteed. The evidence accumulated through `SL-248` and `SL-252` is that it has
instead **traded one kind of complexity for another**, and the kind it traded
into is worse on three axes at once.

## The evidence

**Live design errors, not implementation slips.** `SL-252`'s design round alone
surfaced `ISS-344` — readable inputs are canonicalized and then bound at the
resolved path, destroying the declared name — and with it an authored
contradiction between `SL-248` `PHASE-05` `EX-3` and `EX-9`. The single-file
readable root, the feature that exists specifically so shebang interpreters work,
does not work on any host where `/bin/sh` is a symlink.

**Defects that are invisible where the work happens.** `ISS-339`, `ISS-340`,
`ISS-341` and now `ISS-344` are four instances of one pattern: a derivation that
is accidentally correct in the single environment it has ever run in. Development
happens in a NixOS bubblewrap jail whose `/bin/sh` is a bind mount rather than a
symlink and whose every `$PATH` entry is already an individual package
directory. Each defect is subtle, host-environment-sensitive, and expensive to
diagnose precisely because the environment that would reveal it is not the
environment anyone works in.

**The engagement cost is itself the risk.** Forming a defensible opinion about
any one of these decisions requires loading the production backend, the fixture,
the placement validator, the provisioning steps and several phases of `SL-248`'s
authored plan. It is a large, highly interactive surface with long-range
coupling — the class of problem where an LLM agent is *least* able to converge on
correctness, and where a human reviewer's attention is most expensive. `SL-252`'s
design conversation consumed most of a context window to settle two questions and
opened three more.

## Why this is a risk and not an issue

Nothing here is a bug to fix. Each individual defect is fixable and several are
already captured. The risk is that the **rate** at which this surface produces
host-sensitive defects, multiplied by the **cost** of engaging with it, exceeds
what the programme can sustain — and that patching forward keeps the cost curve
where it is rather than bending it.

## The call this needs

An architectural revision, not another remediation slice. Candidate questions,
none of them settled here:

- Is bubblewrap-with-declared-inputs the right containment primitive, or does the
  guarantee wanted here belong to a VM/microVM boundary where the input-set
  problem does not arise?
- Is the readable-input model — operator-declared paths plus a closure resolver —
  carrying complexity disproportionate to the property it delivers?
- Can the conformance surface be reduced so that correctness is checkable without
  loading the whole design?

Route through `/rfc` or an `ADR` revisit rather than a slice. `ADR-020` (execution
capsules as the dispatch authority boundary) is the decision most directly in
question.

## Provenance

Raised by the owner during `SL-252`'s design run, 2026-08-11, as the reason for
taking `DEC-185`'s S3 fallback: *"take S3, not because it is correct — because
this program of work is off the rails, has exceeded its complexity budget without
delivering a usable implementation, and needs architectural revision."*

Recorded as `DEC-188`'s stated justification, so the slice's own record does not
read as a routine scope trim.
