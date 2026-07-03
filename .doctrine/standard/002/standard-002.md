# STD-002: Naming conventions — short entity titles, ids not slugs

<!-- Body sections reuse the tuned prior art from spec-driver/supekku
     templates/standard-template.md; its YAML frontmatter is dropped — metadata
     lives in the sister standard.toml (storage rule / design D1). -->

## Statement

An entity **title** is a short, queryable **handle** — one crisp line, not an
essay. Rationale, cross-references, and design sketch belong in the sister
`.md` body, never crammed into the title. As a heuristic ceiling, a title past
**~115 characters** is carrying body content — move it.

The **id** is identity. Cite entities by their durable prefixed id (`SL-023`,
`ADR-005`, `REQ-059`, `STD-002`) in prose, commits, and comments. The **slug is
never authoritative** — it is a mobile, human-readable alias that may be
regenerated. Never key logic, references, or membership on a slug; never cite a
mobile membership label (`FR-`/`NF-`) where the durable `REQ-NNN` is meant.

## Rationale

A title is read in `list` views and slugified into a directory name. A long
title makes both unreadable and produces unwieldy paths. The corpus norm bears
this out: committed backlog titles run ~85–115 chars and 190/190 sampled fill
their `.md` body; the outliers that hit 208–242 char titles carried empty
bodies — a **tier inversion** where prose leaked up into the handle.

Treating the id (not the slug) as identity is what lets titles and slugs stay
mobile without breaking references — the single stable anchor is the prefixed
id. Reference drift is silent and corrosive precisely when a slug is mistaken
for identity.

## Scope

Applies to: **all authored doctrine entities** — slices, backlog items,
requirements, memories, ADRs, standards, specs, RFCs, revisions. Covers the
`title` field (brevity, handle-not-essay) and every reference to an entity
(prose, commit scopes, comments, code). Excluded: doc-local enumerations
(`OQ-1`, `D1`, `R1`) which are intentionally bare and local.

## Verification

`VH` — authoring/review time. On authoring an entity, keep the title to a
one-line handle and place elaboration in the `.md`. A title past the ~115-char
heuristic, or an empty `.md` body beside a long title, is a review finding
(tier inversion). References citing a slug or a mobile `FR-`/`NF-` label where a
durable id is meant are findings. `/inquisition` and reviewers grep for both.

## References

- Boot snapshot / `glossary.md` — reference forms, immutable ids, mobile
  membership labels.
- STD-001 — precedent for a project authoring standard verified `VH` at review.
- POL-001 — precedent for a flavoured, enforced project convention.
