# Implementation Plan SL-025: Uniform DRY CLI surface: shared list/show/filter/render contract

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases lift the read/inspect surface (list, show, filter, render) off five
bespoke per-kind implementations onto a single shared contract — the design's
C-hardened spine (design D1): a pure clap-free `listing.rs` leaf for the
invariant axis (filter / id-form / format / hide-set), with the variant axis
(columns, kind flags, ordering) kept local. The cut follows dependency order and
the behaviour-preservation gate: the leaf first, then one kind at a time, with a
final cross-cutting conformance pass.

The spine is built once and proven in isolation (PHASE-01), then exercised
end-to-end on the simplest kind (PHASE-02, adr) where `CommonListArgs` and
`listing::build` are also introduced — so the full command→leaf path and the
`show` seam are validated before the harder kinds ride them. The remaining kinds
follow in increasing order of local complexity: slice carries the governance
weight (PHASE-03), spec/backlog are mid-weight (PHASE-04), memory carries the
uid exception and the boot consumer (PHASE-05). PHASE-06 proves uniformity holds
and readies the slice for audit.

## Sequencing & Rationale

**Why the leaf first (PHASE-01).** Everything depends on `listing.rs`, and it is
the one part that is fully testable with no CLI wiring. Building and unit-proving
it in isolation — including the A-1 domain-distinction guard — means every later
phase composes a known-good seam rather than co-evolving the contract and its
callers. Relocating `render_table` here is the only change that touches existing
code, and it is behaviour-preserving by construction (callers repointed, output
unchanged), so the engine and `is_divergent` suites stay green from the outset.

**Why adr is the reference migration (PHASE-02).** adr is the simplest kind —
numeric, `Meta`-shaped, no existing `show`, the last `meta::format_list` caller.
Migrating it first lets us stand up `CommonListArgs` + `listing::build` and the
shared `show` resolver against the least incidental complexity, and it forces the
boot ADR consumer through the new path immediately (F-N2) — so the
boot-as-consumer claim is tested on the easiest kind, not discovered late on the
hardest. Every subsequent per-kind phase then depends only on PHASE-02, so
PHASE-03/04/05 are mutually independent and could be reordered or parallelised.

**Why slice carries its own phase (PHASE-03).** Slice is where the read refactor
meets governance: the spec-vocabulary amendment (D10), read-side enforcement, the
drift marker (C-1), the one-slice data migration (C-3), and the divergence
predicate split (F-N1/C-2). These are coupled to each other and to an evergreen
spec edit, and they carry the highest correctness risk in the slice, so they are
isolated from the mechanical per-kind migrations. The load-bearing invariant —
`is_divergent` and `is_terminal_status` stay untouched — is asserted here.

**Why spec + backlog share a phase (PHASE-04).** Both already have `show` and
already diverge less from the target; the work is bringing them up, not
inventing. spec's only real complexity is the per-subtype block layout and the
single-envelope-with-subtype JSON (A-8); backlog's is preserving its deprecated
positional as a `--filter` alias (A-7) and reusing its existing
`Status::is_terminal`. Neither warrants a phase alone.

**Why memory is last among the kinds (PHASE-05).** Memory is the outlier: a named
(uid) entity exempt from `canonical_id`, a six-status hide-set, a distinct sort,
and the second boot consumer — which here diverges deliberately from the CLI
default to render active-only (C-4), the one place a kind's boot view and its CLI
view differ. Doing it last means the spine is fully settled before the exception
case stresses it.

**Why a final conformance phase (PHASE-06).** Three things can only be proven
once every kind is migrated: the behavioural parse-conformance test across all
five (R5/A-4), the ordering-preservation sweep (C-5), and the short-flag
collision audit (OQ-3). This phase also runs the full snapshot reconciliation and
the lint/format gate, and checks the closure intent — leaving the slice in a
clean, audit-ready state.

## Notes

- **Behaviour-preservation gate runs through every phase.** The engine suites
  (entity, registry, meta readers) and the slice `is_divergent` suite must stay
  green *unchanged* at every phase boundary. Only output-snapshot tests change,
  and only for the kind a phase migrates — each with its reason in the commit.
- **Boot is a declared consumer, not collateral.** Its ADR section changes in
  PHASE-02 and its memory section in PHASE-05; both have explicit snapshot tests.
  The heavier F-8 memory-section *trim* remains a separate follow-up.
- **Open questions deferred to build:** OQ-1 (per-kind JSON field shapes) is
  resolved per kind as that kind is migrated; OQ-2 (`show --json` relationships
  inclusion) at PHASE-03; OQ-3 (collision audit) at PHASE-06.
- **Phase independence:** PHASE-03, -04, -05 each depend only on PHASE-02. The
  linear authored order reflects risk (slice first) and the outlier (memory
  last), not a hard dependency chain.
