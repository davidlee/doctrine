# DEC-136: Project interpretation policy lives in doctrine.toml

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

`QUE-201` asks where a client project declares the language-bound surfaces that
trusted capsule machinery must not interpret. `DEC-099` already owns the
substantive safety rules: the declaration is required, Doctrine supplies no
ecosystem default, and the trusted control plane reads the declaration from the
contracted base `B`, never from capsule-authored output.

`EVD-011` observed the last rule mechanically. Rewriting an in-repository copy
of the declaration left trusted-side behaviour byte-identical to a baseline
whose declaration could not be rewritten. Location is therefore an ownership
and ergonomics choice, not a remaining substitution-security choice.

## Decision

The project-owned interpretation policy lives in an **`[interpretation]` block
in `.doctrine/doctrine.toml`**.

The block is mandatory when dispatch provisions an execution capsule. Its
absence refuses that operation; the optional-default convention used by other
`doctrine.toml` settings does not manufacture an interpretation policy. The
trusted control plane resolves the block from the contracted base commit and
carries that immutable resolved value through the transaction. It never
re-resolves policy from a capsule checkout or harvested result.

The work contract is a consumer, not an author, of project policy. It may make
a phase more restrictive: reduce permitted execution, add required checks, or
select a smaller declared path surface. It may not widen execution, remove or
weaken required verification, replace the project declaration, or make a
missing declaration acceptable. The target technical specification owns the
exact schema and the monotonic subset checks.

The term **default-deny manifest** in `DEC-099` describes these semantics. It
does not require a second physical manifest file.

## Why this home

`.doctrine/doctrine.toml` is already Doctrine's single canonical surface for
project-local configuration. It is project-owned, discoverable, projected for
project-level control, and parsed through one shared configuration seam. The
interpretation policy has the same owner and mutation authority as that
configuration, so splitting it into another file creates no trust boundary.

A dedicated manifest would make the declaration visually prominent, but would
also add another projected artifact, parser/read path, and synchronization
surface. That cost is not justified when `DEC-099`'s read-from-`B` invariant
already supplies the security boundary. This follows `ADR-019`'s minimal-
projection rule: physical separation earns its keep when ownership, mutation
authority, trust, consequence, or lifecycle differ; they do not differ here.

Making the work contract authoritative was rejected. Contracts are transient,
phase-scoped, and adjacent to agent-authored work. They are the right place to
carry a trusted derivation and a stricter phase restriction, but the wrong place
to define or relax standing project policy.

## Implementation handoff

The capsule implementation should:

1. Extend the existing shared `doctrine.toml` parser with a typed
   `[interpretation]` projection rather than add an independent reader.
2. Resolve the declaration once from contracted base `B`; bind the resolved
   value to the work contract and admission journal so every later stage uses
   the same policy input.
3. Refuse capsule provisioning when the block is missing or malformed.
4. Validate phase-level restrictions monotonically: permissions can only
   shrink and required verification can only strengthen.
5. Promote `SL-241`'s declaration-substitution probe into a production
   acceptance test, alongside missing-declaration and forbidden-widening tests.

The exact TOML keys and normalization rules belong in the target capsule
technical specification. This decision fixes the owner, physical home,
provenance rule, and direction of contract refinement without pre-empting that
schema design.

## Consequences

- `QUE-201` is answered and no longer blocks `REV-046`.
- Capsule dispatch gains a direct implementation seam in the existing
  `doctrine.toml` loader rather than a new configuration subsystem.
- The declaration remains conspicuously security-significant through its typed
  block, required-on-capsule-use semantics, and fail-closed validation, even
  though it shares a file with other project settings.
- Work contracts retain useful per-phase restriction without becoming an
  alternate policy-authoring surface.
- `QUE-202` remains the only unsettled question blocking a complete capsule
  admission and cutover design.

## Origin

Answers `QUE-201` for `RFC-025`. It applies `DEC-099`'s ownership and
read-from-base rules, consumes `EVD-011`'s substitution evidence, and supplies
the interpretation-policy input required by `REV-046`.
