# IMP-405: SPEC-030 keys backends to platforms; mechanisms are many per platform

`SPEC-030`'s confinement contract is sound and this is a wording defect on top
of it, not a contract defect.

## What is right already

§ Platform backend contract states every requirement as an observable property —
fresh mutable state, explicit base and input set, no writable canonical
repository or credentials, bounded host filesystem visibility, explicit network
posture, deterministic working directory, process-tree teardown, trusted
observation of limits and termination. No mechanism appears in that list. `D8`
makes property-equivalence the admission rule, and `REQ-459` criterion 3 admits
*"no macOS **or other backend** … until its mechanism passes the same property
suite independently"* — which is already many backends rather than two.

## What reads narrowly

The section is titled *platform* backend contract and names the measured v0
backend as *"Linux/bubblewrap"*, which keys mechanism to platform. Several
mechanisms sit on one platform and are not decided against:

- bubblewrap supplemented with Landlock, which is likely wanted on non-NixOS
  Linux where the bubblewrap-only floor is weaker than on the measured host;
- Docker and LXC;
- a virtual machine.

Each is a distinct backend on a platform doctrine already supports, and each is
admissible today under criterion 3 — the naming just does not invite the
reading. macOS/Seatbelt is then one more entry rather than the sole alternative.

## Why it matters

A reader who takes "platform backend" literally concludes there is one backend
per platform, and builds the second Linux mechanism as a variant of the first
rather than as a peer under the contract. [[DEC-155]] and [[DEC-157]] were
written against the correct reading — bubblewrap is *a* backend, and the
per-base export is neutral with respect to how a backend exposes it — so the
prose and the code would drift apart.

## Suggested resolution

Small: rename the section and restate the v0 selection so mechanism and platform
are separate axes. Whether that needs a Revision against `SPEC-030` or fits as
an editorial amendment is the first question to settle. Related: [[ASM-009]]
records the one dependency that is genuinely *not* substitutable.

Raised from `SL-248`'s design inquiry, on the operator's observation.
