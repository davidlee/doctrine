# DEC-130: The OS boundary is the confinement boundary

A capsule runs an agent harness. That harness has its own permission system.
`worker-agent.sh` turns it off.

Stated that baldly it sounds like a weakening, which is exactly why this needs
to be a record and not a line in a script.

## The claim

Confinement is asserted **once, at the operating system**: mount namespaces,
`--unshare-all`, `ulimit`. Everything inside the capsule — including the
harness's own notion of what it is allowed to do — is *inside*, and is not a
second boundary.

A harness permission prompt inside an already-confined capsule does not add
security. It adds a **second, weaker boundary whose failure modes are invisible
to the bwrap profile.** Someone auditing the profile would see a correct
confinement story and have no way to know that some of the real enforcement was
happening a layer up, in a tool's configuration, subject to change by anyone
who edits a harness flag.

Confinement you have to re-audit per tool is confinement nobody audits.

## Why this was disclosed, not discovered

The implementing agent made this call and said so, with reasoning: a
harness-level flag inside an already-confined capsule is not a change to the
bwrap boundary, and STOP-5 as written arguably covered it. The operator ruled
the reasoning sound.

The disclosure is load-bearing. A quiet `--dangerously-skip-permissions` in a
worker script is a finding; a disclosed one with an argument attached is a
decision someone can disagree with. Only the second kind can be ruled on.

## The inversion this record exists to prevent

The relationship has a direction, and it is easy to reverse under pressure:

> **The sandbox is the invariant. The harness setting is the variable.**

A future design that tightens the bwrap profile *because* some harness needs a
permission prompt — or, worse, that loosens the profile to accommodate a harness
that wants more room — has inverted it. If a harness cannot operate inside the
boundary, the answer is a different harness or a different capsule kind, never a
softer boundary.

The claim is only as strong as the profile, which is why P-C2 **asserts** on the
profile every run rather than trusting it — see `evidence/results-c2.tsv`, seven
rows, all pass.

## Where this does not reach

It locates confinement at a **Linux** mechanism. macOS has no bwrap; an
equivalent would be `sandbox-exec` or a VM, a different mechanism with a
different failure surface. Nothing in this claim is established off Linux.

See [[DEC-131]] for what the boundary lets through at `$HOME`, and [[DEC-132]]
for the credential property that survives it.
