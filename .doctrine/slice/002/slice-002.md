# Generalise slice machinery into a kind-parameterised entity engine

> **Superseded by slice-003.** An adversarial review (see `handover.md`) showed a
> standalone refactor with a single caller freezes the wrong abstraction boundary
> (it assumed a fixed two-file scaffold; a spec needs ~13). The engine extraction
> is folded into slice-003, where it lands against a genuine second caller (the
> design-doc sibling) of a *different* shape. The load-bearing `acquire` seam and
> the fileset-as-function boundary live there. This slice is retained for the
> audit trail; do not build it.

## Context

`heresy slice` (slice-001) established the directory-entity pattern: a numeric
directory under `.doctrine/<kind>/`, a sister `<kind>-<id>.toml`, a scaffolded
`<kind>-<id>.md`, an `<id>-<slug>` symlink alias, and a collision-free id
allocated by the local `mkdir` reservation (reservation-spec § local backend).

Two further entities are specified and waiting: the **drift ledger**
(doc/drift-spec.md) and the **spec family** (doc/spec-entity-spec.md). Both
explicitly inherit this exact shape — same numeric dir, same sister TOML + prose
split, same reservation. Both notes close with the *same* follow-up: generalise
the slice machinery before they become callers, or we grow three near-identical
copies of `src/slice.rs`.

The whole point of the reservation primitive (reservation-spec § The
unification) is that it is already kind-agnostic — namespace `<kind>/id/<n>`.
The directory-entity scaffolding around it is not yet, and that is the gap.

## Scope & Objectives

Extract the kind-agnostic core of `src/slice.rs` into a reusable **entity
engine** that the slice, drift, and spec kinds all drive through a small
descriptor — *one engine, many kinds*. Concretely:

- A `Kind` descriptor: the `.doctrine/` subdir, the reservation namespace, the
  file-stem prefix (`slice`/`drift`/`spec`), and the embedded template names.
- Generic, kind-blind operations: candidate-id, numeric-dir scan, the atomic
  `mkdir` reserve-and-retry loop, file + symlink writes, `today`, slug
  derivation.
- A seam for the kind-specific parts: the metadata type (its serde struct), its
  template render, and its list formatting.
- `heresy slice` re-expressed as the first caller of the engine, behaviour
  unchanged — the regression guard that the extraction is faithful.

End state: adding the drift or spec kind is *implementing the descriptor + its
metadata/render/format*, not copying the scan/claim/scaffold code.

## Non-Goals

- **Building the drift or spec entities.** This slice only makes them cheap to
  build; both stay deferred (their notes gate on a registry that does not exist).
- **The reservation backends beyond local.** `git-ref` / leases remain
  out of scope (reservation-spec, later). The engine consumes the existing local
  claim; it does not add the `LeaseBackend` trait.
- **The relation registry / FK validation.** doc/relation-index.md — separate,
  deferred. The engine reads and writes entity files; it does not index them.
- **CLI surface changes to `heresy slice`.** `new` / `list` behaviour is
  preserved exactly; no flags added or removed.

## Approach

Split `src/slice.rs` along the line that already exists in it — the kind-blind
plumbing versus the slice-specific data:

- **Engine (new module).** Owns `candidate_id`, `scan_ids`, `reserve_create`
  (the claim loop, still taking its scan as an injected closure for the
  EEXIST→retry test), `write_scaffold`, `derive_slug`, `today`, and the path
  layout (`<id>` dir, `<stem>-<id>.{toml,md}`, `<id>-<slug>` symlink) computed
  from a `Kind`. Templates are fetched via the existing `install::asset_text`
  seam.
- **`Kind` descriptor.** Plain data: `dir`, `namespace`, `stem`, template asset
  names. One value per kind; slice supplies the first.
- **Kind module (slice).** Keeps only `Meta`, its render (template + token
  substitution), and `format_list` / `sort_and_filter`. `run_new` / `run_list`
  become thin: resolve root → call the engine with the slice `Kind` → format.

Keep the same pure/imperative discipline (slices-spec § Architecture): the
engine's pure parts (candidate id, path layout, render, slug) stay clock- and
disk-free; the one impure point remains the `mkdir` claim behind the existing
seam. Reuse, do not reinvent — `crate::root`, `install::asset_text`, and the
output `&mut dyn Write` seam are already shared and stay shared.

## Risks

- **Over-abstraction.** With one real caller, a descriptor risks being a
  speculative framework. Mitigation: the design is *derived from two written
  specs* that name the exact shared surface; the engine generalises only what
  both already require, nothing more.
- **Premature metadata trait.** The drift/spec metadata types differ
  structurally (array-of-tables, FK fields). The engine must parameterise over
  *the file mechanics*, not force a common `Meta` trait. Keep metadata
  read/format on the kind side until a second caller proves a shared shape.
- **Regression in `heresy slice`.** Mitigated by keeping the slice-001 test
  suite green throughout — the extraction is behaviour-preserving by definition.

## Verification

- The full slice-001 test suite passes unchanged (faithful extraction).
- Engine unit tests own the kind-blind cases (candidate-id incl. EEXIST→retry,
  scan, path layout from a `Kind`, slug, scaffold write/symlink), driven with a
  test `Kind` so they are not slice-specific.
- A second, throwaway `Kind` in tests proves the engine produces a correctly
  named/located tree for a non-slice kind (the generalisation is real, not
  nominal) — without shipping that kind.
- Lint clean (zero warnings), formatted.

## Follow-Ups

- **Drift ledger / spec entities** become descriptor-sized once this lands
  (doc/drift-spec.md, doc/spec-entity-spec.md) — but stay gated on the registry.
- **Reservation `git-ref` backend** composes over the engine's local claim later
  without touching callers (reservation-spec § The unification).
