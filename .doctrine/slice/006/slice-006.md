# ADR support

## Context

doctrine records decisions nowhere structured. Architecture Decision Records are
the standard append-only ledger of *why* — and spec-driver
(`~/dev/spec-driver`) already runs a mature ADR system worth learning from: a
`create adr` command, a template, lifecycle-by-symlink-dir, and an `admin preboot`
that injects accepted ADRs into the agent boot context. This slice ports the
**concept**, not the implementation — landing ADRs on doctrine's existing entity
engine so there is no parallel machinery.

Where spec-driver and doctrine diverge, doctrine's idioms win. spec-driver stores
one flat `ADR-NNN-slug.md` with YAML frontmatter and mirrors `status` into
`accepted/` symlink dirs. doctrine's storage rule forbids queried data in prose,
and the entity engine (`src/entity.rs`) already materialises numeric dir-per-entity
artefacts as `NNN.toml` + `NNN.md` + `NNN-slug` alias symlink — exactly the slice
shape. So the ADR is the **next numeric caller of the unchanged engine**: a
top-level `Fresh` kind, structurally a slice minus the design/plan/phase
sub-artefacts.

Unlike SL-005 (memory's string identity broke the engine's numeric assumption),
ADR breaks nothing. It is the deliberately-thin proof that a *new top-level
governance entity* drops onto the engine with zero engine change — the seam works
as designed. The value (lifecycle transitions, supersession, a governance boot
listing) attaches to this proven entity in follow-ups, not here.

## Scope & Objectives

- **ADR entity (toml+md split).** A new `ADR_KIND` (`dir = ".doctrine/adr"`,
  `prefix = "ADR"`, `scaffold = adr_scaffold`) materialising
  `.doctrine/adr/NNN/adr-NNN.toml` (structured metadata: id, slug, title, status,
  created, updated, reserved `[relationships]`) + `adr-NNN.md` (Context / Decision /
  Consequences / Verification / References prose) + `NNN-slug` alias symlink. Two
  embedded templates (`install/templates/adr.{toml,md}`) over the existing token
  set (`{{id}} {{slug}} {{title}} {{date}} {{ref}}`). Authored, committed,
  reviewable — the storage rule, mirroring the slice entity exactly.

- **`doctrine adr new [TITLE]`.** Allocate the next `ADR-NNN` and scaffold the
  entity via `entity::materialise(&ADR_KIND, …, MaterialiseRequest::Fresh, …)` —
  the `slice new` path minus its slug-policy nuance (an ADR always slugs its
  title). The thin shell owns the clock (`today()`); the pure layer takes the date.

- **`doctrine adr list [--status S]`.** Read each `adr-NNN.toml`, AND-filter on
  status, format `id status slug title` rows. Reuses the pure-format / IO-seam
  split of `slice list` (`src/slice.rs` `read_metas` + `sort_and_filter` +
  `format_list`).

- **`doctrine adr status <ID> --status S`.** Flip the authored `status` (+ bump
  `updated`) in `adr-NNN.toml` via `toml_edit` — edit-preserving, comments/unknown
  keys intact. Modelled on `state::set_phase_status` (`src/state.rs`) but against
  the **authored committed** toml and without the runtime progress-log /
  started/completed bookkeeping. `AdrStatus` is a clap `ValueEnum`
  (`proposed|accepted|rejected|superseded|deprecated`, the spec-driver enum
  trimmed). This gives ADR a lifecycle-transition verb that slice still lacks
  (`CLAUDE.md` known gap) — and is the first consumer of an authored-status mutation
  that slice can later reuse.

End state: an ADR can be created, listed, and transitioned; it lives as a
committed, greppable, reviewable authored entity under the storage rule. The
engine has now hosted a **new top-level governance kind with zero engine change**,
so the deferred governance features (supersession, the symlink-by-status index,
the boot listing) attach to a proven entity.

## Non-Goals

- **`adr supersede` (relational lifecycle).** The paired
  `supersedes`/`superseded_by` link that makes ADRs an append-only ledger is the
  natural F1 follow-up. v1 authors the `[relationships]` fields present-but-empty
  so F1 is purely additive (no schema migration). Deferring keeps v1 to the entity
  and its three thin verbs — the slices-spec / SL-005 staging (scaffold + read +
  one mutate, everything relational later).

- **Symlink-by-status index.** spec-driver's `accepted/`/`proposed/` symlink dirs
  are a *derived* index (gitignored, regenerable) in doctrine terms. The F2
  follow-up (`adr reindex` + a `--format=tsv` on `list`). v1 represents status in
  toml and filters in `list` — the slice idiom; no new symlink pattern, no
  gitignore change.

- **Governance boot listing.** spec-driver's `admin preboot` injects accepted ADRs
  into the agent boot file. doctrine has **no boot-context generator** today;
  porting the listing waits until one exists. Out of scope.

- **Relations / backlinks.** `[relationships]` rows may be authored but are inert
  (no resolution, no FK validation) — the same posture as slice `[relationships]`.
  Forward-refs only when F1 lands (spec-driver ADR-002: never store backlinks,
  derive them).

- **Status vocabulary lock.** The `AdrStatus` members are provisional; the model
  does not depend on exact membership.

- **Engine change.** The engine is untouched. The existing entity/slice/state
  suites are the behaviour-preservation proof and must stay green unchanged.

## Summary

The first governance entity in doctrine: a native ADR (`adr-NNN.toml` +
`adr-NNN.md` under the storage rule) with `new` / `list` / `status`, riding the
unchanged entity engine as a top-level `Fresh` numeric kind — structurally a slice
minus its sub-artefacts. Two embedded templates, a thin `src/adr.rs` handler
module mirroring `src/slice.rs`, three `main.rs` arms. Lifecycle status lives in
toml and is transitioned via an edit-preserving `toml_edit` verb (the first
authored-status mutation, reusable by slice later). Supersession, the
symlink-by-status index, and the governance boot listing are explicitly deferred —
they attach to this proven entity in follow-ups.

The scaffold layout, the `run_status` mutation design, the template field set, and
the coupling note (shared authored-status primitive) live in the design doc
([design.md](design.md)) — authored with this slice, pending adversarial review
per the slice-002/003/004 rhythm.

## Follow-Ups

- **F1 — `adr supersede <new> <old>`.** Set `superseded_by` + `status=superseded`
  on the old, `supersedes` on the new — a two-file authored `toml_edit`. Forward-
  refs stored, reverse derived (ADR-002). The second consumer that justifies
  extracting a shared authored-status primitive.
- **F2 — symlink-by-status index.** `adr reindex` rebuilding `.doctrine/adr/<status>/`
  symlink dirs from toml (port of spec-driver `rebuild_status_symlinks`); gitignore
  the derived subtree; add `adr list --format=tsv`.
- **Governance boot listing.** When doctrine grows a boot-context generator, inject
  accepted ADRs via the `--format=tsv` seam (port of `admin preboot`).
- **Slice lifecycle transition.** Extract the authored-status `toml_edit` primitive
  and give `slice` the `status` verb it currently lacks (`CLAUDE.md` known gap).
- **CLAUDE.md.** Add `doctrine adr new|list|status` to the CLI surface, `.doctrine/adr/`
  to the layout, and ADRs to the storage-tier note when this lands.
