# Implementation Plan SL-210: Comparison ledger capture

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Three phases, strictly ordered, tracking the ADR-001 layering inward-out:
pure model first (PHASE-01), then the impure capture shell over it
(PHASE-02), then the read/correct surfaces plus the one adjacent facet change
(PHASE-03). Each phase ends green and shippable; nothing consumes the ledger,
so the behaviour-preservation gate holds throughout, asserted explicitly at
the end.

## Sequencing & Rationale

- **PHASE-01 before any CLI**: the wire model is the contract RV-262 F-1
  bled on — locking the byte shape in a pure, disk-free module with golden
  tests means the shell phases never negotiate schema. Admissibility lives
  here too because it is pure over kind strings; the shell only resolves
  refs to kinds.
- **PHASE-02 owns the only genuinely risky mechanics**: the clap shape
  (bare capture args beside subcommands — no in-repo precedent for
  `args_conflicts_with_subcommands`). The fallback (`compare record`) is
  pre-authorised by the design and EX-4 makes taking it a recorded decision,
  not silent drift. Everything else is assembly of verified seams:
  `resolve_entity_path_and_canonical` (facet.rs precedent),
  `fsutil::create_new_file` (atomic clobber refusal), `clock::today` +
  uuid v7 at the impure edge.
- **PHASE-03 groups the read-side verbs** (list, withdraw) because withdraw
  needs list's dir-scan/flatten machinery to locate target uids; splitting
  them buys nothing. The REV-022 Q1 warn rides here rather than earlier so
  the facet.rs touch happens once, next to its test, after the noisy new
  files have settled.

Phase A of RFC-019 ends at PHASE-03; supersession *resolution*, bounds, and
projection are the next slice (Phase B), which enters against this ledger's
frozen wire contract.

## Notes

- Verb naming, session mechanics, tombstone timing, and the full flag
  surface were adjudicated at design (D1–D9) and re-verified by RV-262 —
  the plan does not reopen them.
- IMP-227 (id-form split): the new verbs use `parse_canonical_ref`
  full-form refs by design decision D4; if IMP-227 lands a unification
  later, compare follows it then.
- `.doctrine/comparisons/` is authored tier: session files are committed,
  diffable evidence. Tests must use temp roots (house pattern), never the
  repo's own corpus.
