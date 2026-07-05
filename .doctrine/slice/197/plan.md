# Implementation Plan SL-197: Add concept (CPT) as a knowledge record kind

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-197 adds a seventh knowledge `RecordKind` — `Concept` (prefix `CPT`): a
neutral, prose-first record with no epistemic lifecycle and an empty facet
(design §3). The record-kind surface is *guarded scatter*, not a single seam
(design §1): most sites are compiler-forced (the `RecordKind` enum) or
canary-guarded, but two composite-union relation sites and two user-facing
runtime kind-list messages had no guard. Rather than add CPT into an unguarded
surface, the slice is split so the DRY-and-guard work lands first, on the
current 6 kinds, provably changing nothing.

## Sequencing & Rationale

**Why two phases, in this order.** PHASE-02's correctness argument is "every
record-bearing site is compiler-forced, RECORD-derived, or canary-guarded." That
argument is only true *after* PHASE-01 closes the four unguarded sites. Doing the
DRY first, as a behaviour-preserving refactor, means:

- PHASE-01's proof is the existing suite: relation + partition tests stay green
  **unedited** (plan EX-4 / VA-1). If a P2/P3/P4 change altered observable
  behaviour, an existing assertion would break — the refactor is wrong by
  construction if the suite needs editing.
- PHASE-02 becomes a near-mechanical append: the P3 canaries and P4 message
  canaries *flip from "pins the 6-kind value" to "requires the CPT append"*, so
  the compiler and the canaries drive the edit list. Forgetting a site is a red
  test or a build break, not silent drift.

**PHASE-01 — the four gaps (design §2).**
- *P2* re-spells the two double-entry relation pins (`relation.rs:1774` concerns,
  `:1782` Supersedes) against a `kinds::RECORD`-derived tail. Today a new kind
  needs two edits per site (the rule + the pin); after P2 it needs one.
- *P3* adds two drift canaries where none existed: the RECORD-subset of the
  Shapes-target union (`:531`) and of the governed_by-sources union (`:556`) each
  equals `kinds::RECORD`. These unions can't become a bare `RECORD` read (Rust
  stable won't const-concat `&'static` slices — design D0), so a canary is the
  cheapest guard.
- *P4* derives the two user-facing runtime kind-list strings from the vocab:
  `dep_seq.rs:84` (needs/after rejection, full-word list) and `knowledge.rs:968`
  (`resolve_ref` unknown-prefix error, prefix list). `RecordKind` already exposes
  `as_str`/`prefix`/`ALL` (verified at plan time), so both are buildable. A canary
  pins each derived string to its current 6-kind value — so PHASE-01 lands no
  user-visible change, and PHASE-02 tracks CPT automatically.

Clap **doc-comment** help (`cli.rs:461`, `knowledge.rs:1650`) is a compile-time
literal, not derivable — it stays a PHASE-02 editorial hand-edit. (`cli.rs:461`
is already 4-kind stale; brought current while there.)

**PHASE-02 — the append (design §1/§4).** Append CPT to `RECORD` and the four
hand-spelled combined constants (NOT auto-derived — `combined_constants_cover_record`
forces the edit). Fill the compiler-forced `knowledge.rs` data and match arms.
Add the `integrity::KINDS` row, the `partition.rs` row, the three relation
record-set appends, and re-pin the one existing explicit Shapes-target assertion
(`relation.rs:2131`, surfaced at plan time — the design's canary alone didn't
cover it). Ship the seed template with an **empty `[facet]` header** — the
per-kind scaffold-order invariant (`knowledge.rs:2162`) requires a `[facet]`
block for every kind, so an omitted block would panic that test (external-review
finding). Re-pin the two goldens and the clap/using-doctrine prose.

**Key design decisions carried into the plan (design §3).** D1 status vocab
`[draft, active, retired]`; D2 empty `ConceptFacet` (first empty facet — seed
emits the header, `show` suppresses it); D3 Shapes/Spawns via RECORD-ride
(permissive); D4 no `supersede.rs` edit (`_ => None` + `validate_matrix` absence
gate CPT off, identical to HYP).

## Notes

- **`supersede.rs` is a scope fence, not a target** (D4): it must NOT be edited;
  VT-6 proves CPT is non-supersadable *because* no arm was added.
- **The behaviour-preservation gate is load-bearing.** If PHASE-01 can't stay
  green without editing an existing assertion, the refactor changed behaviour —
  STOP and reconcile, don't rewrite the test.
- Web/map (PRD-015) is a non-goal (IMP-244 defers concept-map/UI); the scan
  dispatch is data-driven (`from_prefix`), so no exhaustive `record_kind` match
  in `map_server` is expected to break — verify at execute (design §7).
