# Doctrine policies and standards

Policies and standards are governance standing rules — they are in force
continuously (unlike ADRs, which capture decisions at a point in time).

- **Policies (POL-NNN)** — statements of intent or constraint. "We will...",
  "We will not...". Example: commit policy, review policy.
- **Standards (STD-NNN)** — conventions of practice. "We do X this way...".
  Example: code style, naming conventions, directory layout.

## CLI

The CLI is the source of truth: `doctrine policy --help` and
`doctrine standard --help`.

Policies and standards appear in the boot snapshot's governance section when
active. If your project has no policies or standards yet, the boot snapshot
carries a nudge comment (`<!-- No active policies yet. See
mem.signpost.doctrine.policies-standards -->`) and `doctrine boot --check`
emits a warning — a prompt to bed in governance before too much work
accumulates.

See [[concept.doctrine.boot-snapshot]] for how governance reaches the boot
snapshot, [[signpost.doctrine.adrs]] for architectural decisions,
and [[signpost.doctrine.revisions]] for the governance change-axis.
