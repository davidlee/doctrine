# Implementation Plan SL-227: Library read surface and minimal projection

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-227 delivers RFC-021 Claim C1 as one paired change: the `doctrine library`
read surface (Contract B, SPEC-026) and the minimal-projection install flip
(Contract A mechanism, SPEC-009). The design (`design.md`, RV-299 integrated,
Option A) fixed the reachability contract — the library publishes the **full
projection complement**, so the pairing invariant becomes a *derived* gate
(`delta ⊆ published`) rather than a curated list.

Three phases split the design's two along the seam the external pass exposed:
engine additions (small, additive) → read surface + manifest population (the
bulk) → the flip (the risk). The order is load-bearing: the read path must
provably exist before any file stops landing.

## Sequencing & Rationale

**PHASE-01 — Publication engine additions.** Purely additive to `publication.rs`
and the leaf `asset_source`: the metadata accessors + availability probe the
veneer needs, and the additive `ContentKind` widening. It ships no user-visible
behaviour, so its whole risk profile is *not disturbing SL-223*. That is why it
is first and separate — a clean behaviour-preservation checkpoint (EX-3 / VA-1)
before the veneer builds on it. ADR-001 layering holds: leaf ← engine, no
command-tier code yet.

**PHASE-02 — Library read surface + full-complement manifest.** The veneer
(`library.rs` + `main.rs` wiring) and the manifest population land together
because they are tested together — `library show` over the populated manifest is
the round-trip. This phase *completes the read path*: every asset the flip will
drop is published and resolvable here, before PHASE-03 removes it. The manifest
grows to the full complement (~70 entries, all MIT — everything under `install/`
is MIT per `install/LICENSE`); it is authored, not derived (NF-001). NF-002
read-only is proved structural (VT-6) and FR-001 sole-authority behavioural
(VT-5).

**PHASE-03 — Minimal-projection flip.** The cut, structurally last. `build_plan`
leg 2 stops copying the whole embed and reads the `[base]` set; the eager
`[dirs].create` is trimmed (FR-008 is already lazy in `entity.rs`); the eager
`seed_authoring_memories` is gated (D8) so a fresh install is genuinely three
files. The projection-*result* tests flip polarity; the `*_is_shipped` embed
tests stay green (they assert embedding, which survives — ADR-019). The crux is
VT-2: the derived `delta ⊆ published` gate that mechanically enforces the pairing
invariant a future edit cannot silently break.

**Why phase order + the derived gate, not either alone.** Phase order sequences
read-before-cut for *this* implementation; the VT-2 gate is what keeps the
invariant true for every *future* asset. Both are needed — the external pass
(RV-299 X-F4) rejected phase order as sufficient on its own.

## Notes

- **Deferred, explicitly pending (not silently dropped):** SPEC-009 FR-009
  (D6 — no customization verb) and FR-010 (D4 — no define verb); SPEC-026
  REQ-375's *unsupported-source-type* and *metadata-without-bytes* error classes
  (D3 — indistinct with a single adapter). Each carries a follow-up pointer in
  `slice-227.md`.
- **Standing risk carried from RV-299 into execution:** D8 gates a live
  behaviour (the eager orientation-memory seed). Confirm at PHASE-03 that
  `project-orientation.md` carries what the seed used to, so boot orientation
  does not regress.
- **Gate-drift guard:** VT-2's `base backings` must derive from the same
  `install/manifest.toml [base]` that `build_plan` leg 2 reads — keep one source,
  or the gate and the projection can diverge.
- **PHASE-01 may fold** into PHASE-02 if it proves trivially small at
  `/phase-plan`; kept separate here for a clean behaviour-preservation checkpoint.
- The design's `OutputFormat` is indicative (design F5); the real house type is
  `Format` (`main.rs`) — used in the VT keywords.
