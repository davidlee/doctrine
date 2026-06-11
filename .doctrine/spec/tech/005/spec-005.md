# SPEC-005: ADR entity surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The ADR entity surface is the architecture-decision-record kind: the
project-global, citable record of one decision realising **PRD-008**. It is a
component of the entity engine (SPEC-004) — the first kind built on that
substrate. All shared mechanism (identity, the atomic claim, id allocation,
edit-preserving status transition, and the scaffold/render pipeline) lives in the
parent container and is used here unchanged; this spec carries only what is
specific to the ADR kind.

## Responsibilities

Mirrors the structured `responsibilities` list: the ADR `Kind` descriptor and
scaffold, the ADR status vocabulary and list hide-set, the identity TOML fields
plus the reserved `[relationships]` seam, and the prose render contract.

### Kind descriptor and scaffold

The ADR kind binds a data-valued `Kind` into the shared materialiser. Its scaffold
produces three artefacts under the ADR tree root: the `adr-<id>.toml` sister TOML,
the `adr-<id>.md` prose body, and an `<id>-<slug>` symlink. The fileset shape and
the materialisation path itself are the parent container's; only the templates and
the descriptor are ADR-specific.

### Status vocabulary and list hide-set

ADR status is one of **proposed · accepted · rejected · superseded · deprecated**.
This set is both the transition vocabulary and the known-set that `--status`
filtering is validated against. The default `adr list` hides the three statuses
that no longer govern — **rejected, superseded, deprecated** — surfaced again by
`--all` or an explicit `--status`. The status transition itself is the parent's
edit-preserving seam (status + `updated`, clock injected by the shell); this
component only fixes the vocabulary and the hide policy.

### Identity TOML and the relationships seam

The ADR identity TOML carries `id`, `slug`, `title`, `status`, and `created` /
`updated` dates. It additionally holds a reserved `[relationships]` table —
`supersedes`, `superseded_by`, `related`, `tags` — present from the first record
and empty by default. The seam is authored-but-inert: its shape is stable so that
supersession and tagging attach later without reshaping the record, and the
reverse `superseded_by` link is derived, never authored on both sides
(outbound-only, ADR-004).

### Prose render contract

The ADR body is prose only, with **no YAML frontmatter** — all queryable metadata
lives in the sister TOML. The scaffolded body carries a Context / Decision /
Consequences (Positive · Negative · Neutral) / Verification / References
structure. As with every kind, the render reads the TOML facets and treats the
prose headings as a write-once scaffold, never parsing their structure.

## Concerns

- **Lockstep vocabulary.** The status known-set must mirror the status enum
  exactly; an out-of-vocab status would otherwise be silently accepted or wrongly
  rejected.
- **Inert seam stability.** The `[relationships]` table is reserved shape, not
  behaviour; it must round-trip untouched through a status transition until a
  supersession verb wires it.

## Hypotheses

- **The ADR kind needs no mechanism of its own.** Everything beyond the status
  vocabulary, the relationships seam, and the render templates is satisfied by the
  parent container's shared substrate; the component stays thin without becoming a
  stub because it still owns its real kind-specific surface.

## Decisions

- **D1 — metadata in TOML, prose has no frontmatter.** The ADR body is pure prose;
  identity and lifecycle live in the sister TOML, consistent with the storage rule
  the parent container realises.
- **D2 — the relationships seam ships inert.** Supersession, related links, and
  tags occupy a reserved table from the first record, present so the shape is
  stable, with no verb yet wiring them.
