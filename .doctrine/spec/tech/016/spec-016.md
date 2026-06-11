# SPEC-016: Governance kinds (POL/STD)

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The governance kinds are the two standing-rule entity surfaces — **policy** (POL)
and **standard** (STD) — that record what governs a project rather than a single
decision (the ADR kind, SPEC-005). Each is a thin per-kind module
(`doctrine::policy`, `doctrine::standard`) that binds a `GovKind` descriptor onto
a shared command-tier **governance spine** (`doctrine::governance`), the same way
the ADR kind does. They are components of the entity engine (SPEC-004): identity,
the atomic claim, id allocation, edit-preserving status transitions, and the
scaffold/render pipeline all live in the parent container and are used here
unchanged. This spec carries only what is specific to the two governance kinds —
their status vocabularies, hide-sets, render contract, and their projection into
the governance boot snapshot.

The governance spine itself is shared command-tier machinery extracted so a
second (then third) governance kind rides the same list/status/show/scaffold
compute rather than copying it. Both POL and STD parameterise that spine through
the four-field `GovKind` descriptor; this component owns the descriptor *values*
and the per-kind templates, not the spine.

## Responsibilities

Mirrors the structured `responsibilities` list: the two `GovKind` descriptors,
the per-kind status vocabularies and known-sets, the hide-sets, the scaffold and
render contract with the inert relationships seam, the boot-snapshot projection,
and the supersession-as-relationship rule.

### The two kinds and their GovKind descriptor

Policy and standard are each a numeric directory tree (`.doctrine/policy/`,
`.doctrine/standard/`) of the ADR shape: a sister `<stem>-NNN.toml`, a scaffolded
`<stem>-NNN.md` prose body, and an `NNN-slug` symlink alias. Each kind binds a
`GovKind` — the entity-engine `Kind` (dir, canonical-id prefix, scaffold fn) plus
three governance-specific fields: `stem` (the file/JSON stem), `statuses` (the
known-set), and `hidden` (the hide-set predicate). Policy is the kind that proved
`stem` and `prefix` independent: its prefix is `POL` while its stem is `policy`,
the first kind where `stem != prefix.to_lowercase()`. Standard's `stem` happens
to equal its lowercased prefix, but the field stays explicit.

### Status vocabularies and known-sets

A policy's life is `draft → required → deprecated / retired`; a standard's is
`draft → default / required → deprecated / retired`. `required` is the in-force
mandated state for both; `default` (standard only) is the recommended-unless-
justified state. Each vocabulary is a flat clap `ValueEnum` with no per-state
stamping, and the matching `&[&str]` known-set is the authority that
`--status` filtering validates against. Enum and known-set are held in lockstep
by a per-kind drift-canary test, since the enum cannot store an out-of-vocab
status and so the known-set doubles as the complete vocabulary. The
edit-preserving transition itself is the spine's seam (clock injected by the
shell); this component only fixes the vocabularies.

### Hide-set

Both kinds hide **deprecated** and **retired** rows from the default list — those
no longer govern — bound as `GovKind.hidden`. The override (`--all` or any
explicit `--status`) reveals them, applied in the shared `listing` layer, not
here. This component fixes only the hide policy.

### Scaffold, render contract, and the relationships seam

Each kind's scaffold lays out the sister TOML, the prose body, and the slug
symlink by token substitution over embedded templates. The prose body is prose
only with **no YAML frontmatter** — a Statement / Rationale / Scope / Verification
/ References structure — with all queryable metadata in the sister TOML. The TOML
additionally carries a reserved, inert `[relationships]` table (`supersedes`,
`superseded_by`, `related`, `tags`), empty from the first record so the shape is
stable when supersession and tagging are wired; the reverse `superseded_by` link
is derived, never authored on both sides (outbound-only, ADR-004).

### Boot-snapshot projection

The governance boot snapshot projects each kind's in-force rows in-process via
`governance::list_rows`, taking the kind descriptor and an explicit in-force
status set: **Active Policies** filtered to `required`, **Active Standards**
filtered to `default` + `required`. The two sections sit immediately after the
Accepted ADRs section and before Memory, so an agent pays for governance once per
change rather than once per session.

## Concerns

- **Lockstep vocabulary.** Each known-set must mirror its status enum exactly; the
  per-kind drift canary is the proof, since an out-of-vocab status would otherwise
  be silently mis-filtered.
- **In-force set divergence.** The boot projection's in-force status set is
  authored separately from the status vocabulary; standard's two-element set
  (default + required) versus policy's single `required` must stay aligned with
  what each vocabulary calls in-force.
- **Inert seam stability.** The `[relationships]` table is reserved shape, not
  behaviour, and must round-trip untouched through a status transition until a
  supersession verb wires it.

## Hypotheses

- **A governance kind needs no mechanism of its own beyond its descriptor.**
  Everything past the status vocabulary, the hide-set, the templates, and the
  in-force projection set is satisfied by the shared spine and the parent engine;
  the kinds stay thin without becoming stubs because each still owns a real
  kind-specific surface (notably the differing vocabularies and in-force sets).
- **The third kind cost nothing structural.** Standard rode the policy shape
  verbatim, adding only the `default` status; this is taken as evidence the
  `GovKind`-over-spine seam generalises without a per-kind framework.

## Decisions

- **D1 — supersession is a relationship, not a status.** Both kinds record
  supersession via `relationships.supersedes`, so the status enum carries no
  Superseded variant; status describes only the rule's standing.
- **D2 — `stem` is explicit and independent of `prefix`.** Policy fixed the field
  by being the first kind where the file stem differs from the lowercased
  canonical prefix; the JSON envelope key and file stem share that one field so
  they can never diverge.
- **D3 — in-force status sets are authored at the projection site.** The boot
  snapshot names each kind's in-force set explicitly (required; default +
  required) rather than deriving it, keeping the projection a pure, byte-stable
  filter over the shared `list_rows`.
