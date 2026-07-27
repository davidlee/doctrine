# IDE-045: Configurable design review postures and reviewer commands

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Direction

After SL-233 establishes managed design runs and section-level review
attestations, allow project configuration to choose the default design-review
posture instead of fixing Doctrine's built-in default.

The same configuration surface should let the user define the reviewer used by
an adversarial lane: model, harness, command, or equivalent invocation details.
Ship sensible defaults. Prefer a simple user-defined command or compact string
over structured configuration unless demonstrated requirements need Doctrine
to understand individual fields.

## Boundary

This is deliberately outside SL-233 v1. V1 should preserve an extension seam
but need not parse reviewer definitions from `.doctrine/doctrine.toml`, launch
configured reviewers, or define a general agent/harness registry.

## Relationship

DEC-073 defines the runtime reviewer lanes and attestations this idea would
configure. It is adjacent to, but not duplicated by, IMP-155's completed prompt
instruction cascade: selecting reviewer execution is distinct from composing
the selected agent's instructions.
