# Slice design-doc siblings and entity-scaffold engine

## Context

slice-001 shipped `heresy slice new/list` and the directory-entity pattern
(numeric dir, sister TOML, scaffolded prose, slug symlink, local `mkdir`
reservation). slice-002 proposed extracting that machinery into a reusable
engine — but a standalone refactor with a single caller freezes the abstraction
boundary before any second caller can correct it, and an adversarial review
showed the proposed boundary is already wrong (it assumes a fixed two-file
scaffold; a spec needs ~13).

The roadmap supplies the right second caller *now*, ahead of drift/spec: the
slice's own **design-doc sibling**. The schema bundle confirms the change-side
artefacts (design revision, implementation plan, phases) nest *inside* the change
directory — exactly slices-spec § Forward compatibility. The design doc
(`design_revision`) carries **no fenced data blocks — it is pure prose**, so it
is the lowest-risk sibling: template-copy into the slice dir, nothing relational.

This slice therefore does two things at once, which is the point: it **adds the
second caller and extracts the engine against both**, so the generalisation is
shaped by real use, not anticipation. It **supersedes slice-002**.

## Scope & Objectives

- **Design-doc sibling.** `heresy slice design <id>` (verb name TBD) scaffolds a
  `DES-<n>.md` (or single `design.md`) prose artefact inside `.doctrine/slice/<id>/`
  from an embedded template — pure prose, no TOML, no blocks. `slice list` /
  resolution unaffected.
- **Reservation `acquire` seam (the load-bearing extraction).** Lift the inlined
  `fs::create_dir` + `ErrorKind::AlreadyExists` in `reserve_create`
  (`src/slice.rs`) to a one-method seam — `acquire(key) -> Won | AlreadyHeld` —
  with the local `mkdir` as its sole impl (reservation-spec § Code seam). No
  `git-ref`, no full `LeaseBackend` trait; just the seam, so the later backend
  swap is real and not a caller rewrite.
- **Entity-scaffold engine.** Extract the kind-agnostic core — candidate id, scan,
  the `acquire` retry loop, slug, `today`, write, symlink — driven by a `Kind`
  descriptor. The descriptor supplies a **fileset as a function**
  (`scaffold(id, slug, title, date) -> Vec<(PathBuf, Bytes)>`), not a fixed
  toml+md pair, and an **optional reservation** (top-level kinds reserve an id;
  sub-artefacts do not). Slice and design-doc are its first two callers.

End state: a slice is a directory that can grow a design-doc sibling; the engine
that scaffolds both is shaped by two callers of *different* shape (top-level
reserved 2-file slice; sub-artefact non-reserved prose-only design doc), so drift
(third caller, slice-shaped) and spec (fourth, ~13-file) drop in as descriptors,
not forks.

## Non-Goals

- **The implementation-plan and phase siblings.** Next slice — they introduce the
  first relational block (`plan.overview`) and the first mutable runtime state
  (`phase.tracking`), a separate concern (spec-entity-spec § Design-data vs
  runtime-state).
- **Design *review* (`RVW-`) and any block-bearing sibling.** Design doc is prose
  only; anything with embedded tables waits for the table machinery.
- **The `git-ref` reservation backend / `LeaseBackend` trait beyond `acquire`.**
  reservation-spec, later. Only the seam lands here.
- **Drift and spec entities.** This slice only makes the engine fit to host them;
  both stay deferred (registry-gated).
- **A premature shared metadata trait.** The engine parameterises *file mechanics*
  (fileset, reservation), not a common `Meta`; metadata read/format stays
  kind-side until a second metadata-bearing caller proves a shared shape.

## Approach

Split `src/slice.rs` along the line already latent in it:

1. **`acquire` seam first** (cheap, load-bearing, valuable even alone). Rewrite
   the claim loop against `acquire`; local `mkdir` impl behind it. slice-001 tests
   stay green — behaviour-preserving.
2. **Engine module.** Move candidate-id, scan, the loop, slug, `today`, write,
   symlink behind a `Kind { dir, namespace, reserve: bool, scaffold: fn(...) ->
   Vec<(PathBuf, Bytes)> }`. The slice `Kind` reproduces today's two-file output.
3. **Design-doc `Kind`.** A sub-artefact kind: no reservation (it lives under an
   existing slice id), a single-prose-file scaffold. Proves the engine spans both
   shapes — the test that the generalisation is real, not nominal.
4. Keep the pure/imperative discipline and the shared seams (`crate::root`,
   `install::asset_text`, `&mut dyn Write`); reuse, don't duplicate.

## Risks

- **Over-abstraction (the slice-002 charge).** Mitigated structurally: extracted
  against **two callers of different shape**, not one — the boundary is corrected
  by use during this slice, and the fileset-as-function shape is forced by the
  design doc not fitting the two-file mould.
- **Scope creep into IP/phases.** Held off explicitly (Non-Goals); design doc is
  prose-only, the simplest sibling, on purpose.
- **`acquire` seam churn.** Small, behaviour-preserving, guarded by the slice-001
  suite. The risk is doing *too much* (pulling the full trait) — resist; only the
  one method.

## Verification

- slice-001's suite passes unchanged throughout (faithful extraction).
- Engine unit tests own the kind-blind cases (candidate-id incl. the
  `AlreadyHeld`→retry path through the `acquire` seam, scan, slug, fileset write,
  symlink), driven by a test `Kind`.
- The design-doc `Kind` produces a correctly-located prose file under an existing
  slice dir, with **no** id reservation — exercising the non-reserved,
  non-two-file path.
- `acquire` seam: a local-backend test asserts `Won` then `AlreadyHeld` on a
  re-claim; the retry loop lands the next free id.
- Lint clean (zero warnings), formatted.

## Follow-Ups

- **IP + phases** sibling slice (relational `plan.overview`, runtime `phase.tracking`).
- **Drift ledger / spec** entities become descriptor-sized once the engine lands
  (doc/drift-spec.md, doc/spec-entity-spec.md) — still registry-gated.
- **`git-ref` backend** composes over the `acquire` seam later without touching
  callers (reservation-spec § The unification, § Code seam).
