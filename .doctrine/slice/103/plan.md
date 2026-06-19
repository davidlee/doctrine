# Implementation Plan SL-103: SPEC-020: Estimate graph exposure

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases that follow the catalog data flow — **scan → hydrate → graph** —
one per layer, honouring ADR-001 (leaf ← engine ← command, no cycles). Each phase
ends green with its own tests (TDD red/green/refactor); the contract-level
assertions land in PHASE-03, where the JSON contract first becomes observable on a
graph node. The design (`design.md`) is canon; this plan only sequences it.

## Sequencing & Rationale

**Why layered, not vertical-by-facet.** Estimate and value are symmetric from the
first line (D3's `read_facets` is generic over both, D5 reuses both leaf
serialisers). Splitting "estimate first, value second" would build the generic
helper in one phase and add a near-empty wiring delta in the next — artificial
seams. Splitting by *layer* instead gives each phase a real, independently testable
unit of behaviour.

**PHASE-01 — Scan-side facet read.** The new behaviour starts at the read: a
kind-agnostic `read_facets` + generic `parse_facet` off every entity TOML, with
per-facet malformed isolation (D4 — a bad facet yields a loud diagnostic and drops
*that* facet to `None`, never a coerced bound, never a dropped node). This phase is
verifiable in isolation at the read tier (faceted / non-faceted / malformed /
kind-agnostic) before any projection exists, so the riskiest mechanism — the
fail-loud-not-repair isolation — is pinned first.

**PHASE-02 — Hydrate + units resolution.** Carry the facets one layer up
(`ScannedEntity → CatalogEntity`) and introduce the top-level `Units` block. D2's
key constraint: units are project-wide, resolved **once in the shell**, injected
into a pure `from_scanned` — so this phase holds the pure/imperative line
(`from_scanned` reads no disk). The `doctrine.toml` read defaults **only on
NotFound** and propagates every other error (RV-094 F-4) — a one-line discipline
that the prior draft got wrong, so it earns an explicit exit criterion and test.

**PHASE-03 — Graph projection + contract sealing.** Project onto
`CatalogNode`/`CatalogGraph`, then verify the whole contract end-to-end: the seven
design VTs collapse into observable graph-node assertions here, plus the two agent
attestations (NF-001 structural non-blocking; vocabulary denylist). This phase also
retires the now-fulfilled `expect(dead_code)` markers — and must leave the
SL-102-owned symbols dead — so clippy stays zero-warning (no *unfulfilled-expect*).
The contract is sealed on **both** surfaces it reaches: the `catalog graph` dump
and `/api/graph` (which serves `CatalogGraph` raw — RV-094 F-3).

## Notes

- **Behaviour preservation** rides every phase: the existing scan / hydrate / graph
  / map_server suites stay green unchanged — the contract evolution is additive
  (new struct fields, new top-level key), never a regression. Updating the direct
  construction sites (`map_server/routes.rs` literal, test literals, the single
  `from_scanned` call site) is mechanical, not behavioural.
- **Open governance carried, not blocking** (`design.md` §7, RV-094 F-1): value
  graph exposure is deliberate traced-pending scope; its requirement is authored and
  spec-homed at **reconcile** (CHR-011), and SL-103 traces to it and to REQ-280
  there. The plan implements the value path now under that recorded obligation — it
  is not re-litigated per phase.
- **Reconcile-tier options** noted in the design, not planned here: the read-once
  seam (one parse → status/title/facets, closing D3's second parse and F-6's
  divergent-read window) and the D2 unit-literalism ratification (F-5).
