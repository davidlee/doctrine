# Implementation Plan SL-006: ADR support

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Five phases. The first is a behaviour-preserving refactor that pays the design's
one structural debt up front (the D4 shared-substrate extraction); the next three
build the ADR entity bottom-up (pure render → read verbs → the one mutating verb);
the last closes out with an end-to-end test and the doc updates. No engine change
in any phase — `src/entity.rs` is consumed, never touched, and its suite plus
slice's are the gate at P01 and P05.

The slice is deliberately small: ADR is the slice shape minus sub-artefacts, so
most phases are parameterisations of paths slice already walks. The only genuinely
new code is the templates, `adr_scaffold`, and `set_adr_status`.

## Sequencing & Rationale

- **P01 first, alone, green.** The extraction touches `slice.rs` (the reference
  numeric caller), so it goes first and in isolation — the same rhythm SL-005 used
  for its seam rename (isolated, behaviour-preserving, green before anything new
  rides it). Doing it first means ADR's `list` is *written against the shared
  module from birth*, never against a copy — so the no-parallel-implementation rule
  is satisfied by construction, not by later cleanup. Risk if deferred: ADR copies
  slice's helpers "just for now" and the extraction never happens.

- **P02 pure before P03 imperative.** Templates + `adr_scaffold` + `ADR_KIND` are
  pure (render over embedded text, no disk), testable without the CLI. This honours
  the pure/imperative split and keeps the disk-touching surface (P03/P04) thin.

- **P03 before P04.** `new`/`list` are the slice-proven paths (Fresh materialise +
  shared list reader); they land first so there are real ADRs on disk for P04's
  `status` to transition. `status` is split out because it is the slice's one novel
  mechanism — the first `toml_edit` mutation of a *committed authored* file — and
  deserves its own green boundary and its own focused tests (round-trip,
  preservation, the I5 no-op guard).

- **P05 last.** The end-to-end test exercises the real verb sequence; the CLAUDE.md
  updates document the new storage surface; close-out harvests anything durable.

Each phase ends green and is a conventional commit (`feat(SL-006): …`,
`refactor(SL-006): …` for P01, `doc(SL-006): …` for the P05 docs). Phase status is
flipped via `doctrine slice phase 006 PHASE-NN --status …` — runtime state, not
this file.

## Notes

- **Module siting (P01).** The shared substrate lands in a new `src/meta.rs`, not
  `src/entity.rs`: `format_list` is CLI presentation and `Meta` is an authored-toml
  reader, neither of which belongs in the kind-blind scaffold engine. Final name is
  a P01 call; `meta.rs` is the working choice.

- **Stem vs prefix.** `read_metas(tree_root, stem)` takes the *file* stem
  (`"slice"`/`"adr"` → `{stem}-NNN.toml`), which is distinct from `Kind.prefix`
  (`"SL"`/`"ADR"`, the canonical-id token). They happen to coincide in spelling for
  neither kind — keep them as separate arguments.

- **Audit trail (P04).** No in-file transition log. The ADR toml is committed, so
  git history is the ledger; an in-file log would duplicate git and re-store derived
  data in an authored file. This is the deliberate asymmetry with
  `state::set_phase_status` (which logs because runtime state is gitignored). See
  design § 5.3 / Q1-Q2.

- **Deferred seams pre-shaped.** v1 authors `[relationships]` present-but-empty so
  F1 `adr supersede` is purely additive; the flat `adr/NNN/` layout leaves room for
  F2's gitignored `adr/<status>/` symlink dirs beside it (engine's numeric-only
  `scan_ids` already ignores them). Neither is built here.

- **Backend context (not this slice).** forgettable's ADR-005 fixes forgettable as
  a generic event substrate and doctrine *memory* as a client riding opaque
  payloads — relevant to the memory roadmap (SL-008 anchoring / SL-009 links), not
  to ADR. Doctrine's ADR feature is a local authored entity with no backend.
