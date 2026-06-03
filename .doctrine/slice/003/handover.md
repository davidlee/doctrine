# slice-003 handover — engine + design-doc sibling

Brief for a fresh agent picking up **review & refinement** of slice-003's design.
The design doc ([design.md](design.md)) is written and committed; no code yet.
Your job is to adversarially review the *design*, refine it, and (if it survives)
take it toward build-ready. This file is your priors and the audit-trail home —
append your review + dispositions here, the same way slice-002 recorded its
rounds.

## What slice-003 is

Extract the kind-agnostic directory-entity machinery out of `src/slice.rs` into an
engine, shaped by **two callers of different shape**: the slice (top-level,
reserved, toml+md+symlink) and a new **design-doc sibling** (sub-artefact,
non-reserved, single prose file under an existing slice dir). First step is
lifting the inlined `mkdir` claim to a one-method `acquire` seam. It **supersedes
slice-002** (`supersedes = [2]` in slice-003.toml `[relationships]`).

Status is `proposed` — *not* flipped to `ready`; that gate is the user's. Build
sequence is design.md § 8 (seam → engine → design-doc Kind → verb), each step
green against the slice-001 suite.

## Read first, in this order

1. [design.md](design.md) — the artefact under review. The engine: `acquire` seam,
   `Kind` descriptor (fileset-as-function + optional reservation), materialise loop.
   Decisions D1–D7, open Q1–Q3.
2. [slice-003.md](slice-003.md) — the slice contract (scope, non-goals, approach).
3. `src/slice.rs` — the only built code; the engine's starting point. Note
   `reserve_create` (the mkdir inline to lift), `build_scaffold`/`Scaffold` (the
   fixed-pair struct that becomes `Fileset`), and the slice-001 test suite (the
   behaviour-preservation gate).
4. `../002/handover.md` — **your priors.** Two dispositioned rounds on the broader
   entity model + a drift-schema realignment + the consolidation direction. The
   settled calls below come from there.
5. Notes the design leans on: [reservation-spec](../../../doc/reservation-spec.md)
   § Code seam (the `acquire` invariant), [slices-spec](../../../doc/slices-spec.md)
   (on-disk shape, pure/imperative split), [entity-model](../../../doc/entity-model.md)
   (the storage rule + where this engine sits for drift/spec).

## Already settled — do not re-open without new argument

These were reasoned through over two review rounds (002/handover.md). Hold the
line unless you bring genuinely new evidence; if you re-litigate, say so and
argue it.

- **Extract against two callers, not one** (slice-002 superseded). A standalone
  one-caller refactor froze the boundary wrong; the design-doc is the corrective
  second caller.
- **Fileset is a function, not a fixed toml+md pair** (slice-002 M3). The frozen
  2-file boundary was *the* bug; `Vec<Artifact>` is the fix.
- **Only the `acquire` seam now — not the full `LeaseBackend` trait, not `git-ref`.**
  Lifting more is the over-build risk the slice names.
- **No premature shared `Meta` trait.** Parameterise file mechanics, not metadata;
  `Meta`/list stays slice-side until a second metadata-bearing reader exists.
- **Spec family = one model, three subtypes, own folders, per-subtype filesets**
  (entity-model.md). Relevant because spec is the engine's eventual fourth caller.
- **Keep deferred deferred.** drift and spec stay registry-gated; this slice only
  makes the engine *fit to host* them. A finding is not a licence to start building
  a deferred entity. Roadmap changes go via supersede (002→003), not scope creep.

## High-value review targets (fresh surface — expect the real findings here)

The engine boundary is the live build decision; it is cheaper to fix on paper now
than after extraction. Push hardest on:

- **D1 — the `acquire` seam is path-based (`&Path`), not the abstract `key: &str`.**
  The design argues the git-ref evolution is additive (second impl + split the
  dir-creation out of `acquire`). Is that true, or does path-baking leak in a way
  that *will* force a caller rewrite — exactly the F1 failure slice-002's seam was
  meant to prevent? This is the load-bearing claim; verify it, don't take it.
- **D2 — `Kind` as a data struct with a `fn` pointer vs a trait.** Does the `fn`
  signature carry everything (template reads return `Result`; sub-artefact ctx)?
  Where does a struct-of-fns crack first?
- **The `Artifact` model (File | Symlink).** Does it cover everything the slice
  does today (symlink `AlreadyExists` tolerance) and everything a spec subtype
  needs, or is it the next boundary frozen too low?
- **The non-reserved branch.** Is "resolve existing dir + refuse clobber" the
  right shape for *all* sub-artefacts, or design-doc-specific?
- **slice.rs / entity.rs split.** Is the kind-blind / slice-specific line drawn in
  the right place — especially `Meta`/`read_metas`/`format_list` staying behind?

## The bar (same discipline as 002/handover.md)

- **Verify before accepting.** A claim about the schema, a code path, or a file is
  a hypothesis — check it. Verification source: `spec-driver-schemas.local.md` at
  repo root (**gitignored — do not commit**), and the live corpus
  `~/dev/spec-driver/.spec-driver/`.
- **Re-judge severity; push back on over-claims; cheapest fix wins.** A sentence, a
  field, a renamed boundary — not a rewrite.
- **If a finding reshapes the roadmap**, do it the established way (supersede +
  banner + `supersedes`), not by silently widening this slice.

## If you touch code

Project gate: `cargo test` + `cargo clippy` (deny-level; `--all-targets` is *not*
the gate) + `cargo fmt`. Red/green/refactor. Behaviour-preserving extraction —
the slice-001 suite must stay green at every step. Update slice-003.toml
`status`/`Done` per convention.

## Audit trail

Append your review verbatim + a disposition table (`finding → call → landed-in`)
below, as 002/handover.md does. Keep the design's reasoning reconstructable.

---

(review rounds append here)
