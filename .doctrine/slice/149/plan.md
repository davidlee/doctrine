# Implementation Plan SL-149: References role grammar

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-149 implements RFC-003 Axis B: collapse the work→canon relation family into one
`references` label refined by a closed role dimension `{implements, scoped_from,
concerns}`, re-keying target validation from `(source, label)` to `(source, label,
role)`. The design (`design.md`) is the canon for *what*; this plan is the *order*.

The dominant force on sequencing is **SPEC-018's no-dual-read hard cut**: there is no
valid intermediate corpus state where old and new vocabulary coexist. So the entire
role-aware machinery (vocabulary → storage → surfaces) must be built and green *before*
the corpus is touched, and the corpus rewrite must land in the **same commit** as the
parser that understands it. That dictates a strict build-then-migrate spine, migration
second-to-last.

## Sequencing & Rationale

**PHASE-01 — Ratifying ADR (the gate).** Routing folded the ADR into this slice's
design; the decisions are locked, but the ADR must be authored and *accepted* as canon
before any code. This is a governance gate, not a coding phase — RFC-003 asserts no
canon, and the slice may not implement an unratified direction. Kept as its own phase so
the gate is explicit and auditable.

**PHASE-02 — Vocabulary layer (leaf/engine, pure).** The `Role` enum, the re-keyed
`RELATION_RULES`, `lookup`/`legal_roles`/`inbound_name`/`validate_link` as pure
functions. This is the foundation every later phase depends on, and it is the cleanest to
test in isolation (no IO). The table invariants (F4 — one rule per `(source,label,role)`;
each `(source,label)` wholly roleful or roleless) are enforced here because `lookup` is
"first matching row wins" and `canonical_position` assumes one canonical row per
`(source,label)` — ambiguity introduced here would corrupt everything downstream. Note
the lockstep split (F4): VT-2 `sources_match_shipped_accessors` (`relation.rs`) is a
source-set audit and can land now; **VT-4** exact-coverage
(`reader_emitted_labels_equal_table_labels_per_source`, `relation_graph.rs`) needs the
reader to emit roles and therefore lands in PHASE-03.

**PHASE-03 — Storage + IO.** Edge/row role threading, `read_block` parse,
`append`/`remove` on the `(label,role,target)` triple, role-keyed `check_target_kind`,
role-aware `validate_relations`. VT-4 lights up here once the reader emits `(label,role)`.
The behaviour-preservation gate is explicit: the generic machinery suites must stay green
*unchanged* for label-only edges — only the vocabulary content changes, not the seam.

**PHASE-04 — Surfaces + CLI.** The widest blast radius (F1/F5 seam inventory). The
load-bearing subtlety is F1: role must ride as **edge payload** through
`CatalogEdge`→`InspectView` so inbound can group by `(label,role)`, while cordage
**overlay allocation stays label-keyed** (graph-effect is consumer policy; one
`references` overlay). The `show --json` schema change (named `specs`/`requirements`
fields → a `references`-by-role object) is consumer-facing and gated on EN-3 deciding the
shape first. Inbound wording (EN-2, R4) is settled before surface goldens harden, so
goldens are written once.

**PHASE-05 — Corpus migration (hard cut).** Second-to-last by necessity (needs the full
parser). The mechanism is the F3 fix: build the full rewrite in memory, apply as a single
atomic swap, validate only *after*; never row-by-row through the live `link` verb (which
would transit invalid intermediate states). The oracle is the F2 fix: assert role
*assignment* (exact `(source,target)→(label,role)` per deterministic row + a reviewed
disposition artifact per ambiguous row), with edge-set preservation only a secondary
sanity check. The ambiguous residue (SL→SPEC implements-vs-concerns; non-peer `related`)
gets human confirmation (VH-1) before the commit, because kind alone cannot classify it
(AR-1). Parser + rewritten corpus share one commit.

**PHASE-06 — Docs + reconcile.** SPEC-018 and `relation-vocabulary.md` are canon prose
describing the contract; they are rewritten last, against the shipped state, so they
describe what exists rather than what was planned. Reconcile the slice for `/audit`
handoff.

## Notes

- **Phase boundaries are dependency cuts, not size targets.** PHASE-04 is the largest;
  `/phase-plan` may surface a sensible internal split (e.g. inbound-payload vs
  show-schema vs CLI) when its runtime sheet is expanded — that is a runtime breakdown,
  not a re-plan, since the PHASE ids are immutable.
- **Golden churn is concentrated in PHASE-04 and PHASE-05.** The R2 machinery/content
  audit (PHASE-04 VA-1) is the guard that keeps a deliberate vocabulary change
  distinguishable from a regression.
- **Migration vehicle** (gated `#[ignore]` test vs throwaway bin) is a PHASE-05
  runtime-sheet decision; either is acceptable provided it honours the all-or-nothing
  in-memory-transform shape (F3). It is not a shipped CLI verb (SPEC-018 dogfood-only).
- **Re-census at execution.** Live counts drift (the RFC snapshot caveat); PHASE-05
  re-censuses live rather than trusting the gitignored P1 artifact.
