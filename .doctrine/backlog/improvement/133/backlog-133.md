# IMP-133: CLI usability shortfall UX review (agent run)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Systematic UX review of the `doctrine` CLI surface, run by an agent, to identify
usability shortfalls — inconsistent arg ordering, unclear help text, missing
confirmations, surprising defaults, hard-to-discover commands, etc.

## Motivation

- **ISS-040** documents one concrete instance: `needs`/`after` positional args
  are trivially reversed because the verb is implied between two bare refs, unlike
  `link subject-verb-object` form. Likely not the only such pattern.
- RFC-001's consumption-surfaces thesis argues the CLI is the primary outward
  surface — its usability directly gates team adoption.
- Several commands have accumulated without a holistic UX pass (`needs`, `after`,
  `link`, `unlink`, `backlog tag`, `estimate set`, `value set`, `spec`, etc.).

## Scope

- **Agent-run**: a bounded automated review — enumerate every CLI verb, check each
  for: argument order consistency, help text clarity, confirmation/echo on writes,
  discoverability of subcommands, error message quality, symmetry with sibling verbs.
- **Output**: a findings document with severity (nit/major/blocker), linking back
  to concrete examples.
- **Not in scope**: redesign, implementation, or user research. Pure audit.

## Links

- ISS-040 — Sequencing verb arg order too easy to reverse (concrete instance of
  the class of problem this review targets)
- RFC-001 — Thesis: graph value is gated on consumption surfaces
