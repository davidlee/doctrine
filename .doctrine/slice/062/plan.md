# Implementation Plan SL-062: Uniform lifecycle-transition + destructive verbs across kinds

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-062 closes two coupled absences (design §1): the edit-preserving authored-TOML
status setter is hand-copied 4× (gov/slice/backlog/requirement), and the lifecycle
FSM is trapped in `slice.rs` while ADR-009's *other* half (the conduct axis) already
lives in `conduct.rs`. On top of those, it adds the missing transactional owner of
supersession — the verb SL-048 OD-3 deferred "until IMP-006 builds it".

The work is three phases, sequenced by the **behaviour-preservation gate**: extract
under green suites *first*, add the only new behaviour (the verb) *last*. PHASE-01
re-homes the pure FSM (a pure-leaf move, no IO). PHASE-02 lifts the duplicated
write-core into one mutation seam and retires the four setters onto it. PHASE-03 adds
the `supersede` verb, composing PHASE-02's pure cores into a parse-once/hold-both/
write-once transaction. The decomposition mirrors SL-060's "lift once, dispatch per
kind" idiom (its structural template) — no parallel implementation.

## Sequencing & Rationale

- **PHASE-01 first — the pure-leaf move, in isolation.** The FSM
  (`classify`/`Transition`/`is_transition_terminal`/`crosses_closure_seam`/edge
  table) is genuinely pure `&str`-in state-machine code; re-homing it beside
  `conduct.rs` completes the ADR-009 pairing as two pure leaves (D1). Done as its
  own phase with the behaviour-preservation gate, it proves the move is mechanical
  *before* any consumer rewires. `is_terminal_status` and `transition_label`
  deliberately do **not** move (P3/P4) — the first is divergence semantics beside
  `is_divergent`, the second slice-CLI presentation; moving them would taint the
  kind-agnostic leaf. `SLICE_STATUSES`/`SliceStatus`/the drift canary stay too
  (command-tier vocab, not FSM logic). One consumer today is acceptable and stated —
  this is cohesion (ADR-009's correct address), not speculative reuse.

- **PHASE-02 — one mutation seam, then retire the four setters.** The duplicated
  write-core (read → parse → no-op guard → F-1 refuse → insert → single write) is the
  largest honest DRY move. It splits by responsibility (D1/P1): **pure cores** on a
  held `&mut DocumentMut` (`apply_status`, `apply_string_append`) + **thin IO
  wrappers** (`set_authored_status`, `append_string_array`). This shape is what lets
  PHASE-03 compose the *same* mutation logic over docs it parses once and writes once,
  with no third read/parse/write body and no `with_authored_doc` second seam (P1).
  `apply_string_append` reuses `dep_seq::append`'s string-membership (`needs`) path
  parametrized by field — **not** a widening to "any array" (which would entangle the
  `after` `{to,rank}` struct-eq path, codex C2); it is a real refactor carrying its
  own value-domain/idempotence/F-1 contract, not a free lift (R3). The gate **leaves**
  the setter (D3): each kind keeps its distinct policy in the shell — slice classifies,
  backlog couples resolution, gov/requirement are flat — and delegates only the write.
  Requirement's single-key managed set (status only, no `updated`) is exactly what
  exercises the helper's variable-length generality (codex C1). Retiring gov/requirement
  onto the shared core is the moment to reword their F-1 messages non-destructive — the
  SL-060 lesson ("never regenerate via `<kind> new`"), pinned as a test (§7 footnote).
  This phase depends on PHASE-01: slice's retired `run_status` gates via
  `lifecycle::classify`.

- **PHASE-03 last — the verb, the only new behaviour.** With the pure cores in place,
  `doctrine supersede <NEW> <OLD>` is a kind-agnostic shell over per-kind data:
  `supersede_policy(kind)` returns `Some` for ADR only (the sole kind whose vocab has
  `superseded` — F-C); POL/STD/slice are refused with the ADR-first message and folded
  into follow-up F2. The transaction is **parse-once / hold-both / write-once** (§5.4):
  a real parse-and-verify pre-flight over both held docs (every touched key/array
  scaffold-present) reduces the residual failure class to a mid-write I/O error (codex
  C3/P2 — there is no later re-parse for F-1 to re-fire at). The not-already-superseded
  guard checks **both** files (C4), and the drift case (`status==superseded` with an
  empty/malformed carve-out) refuses pointing at `doctrine validate` rather than
  self-healing (P5). NEW-then-OLD write ordering is what makes the one-sided no-op check
  sound, and the residual torn state is detectable by the SHIPPED SL-048 PHASE-05
  cross-check — *not* auto-run by the verb (C5). The verb writes the current canonical
  **typed** `supersedes`/`superseded_by` fields; the `[[relation]]` storage migration is
  downstream (F3), and the transaction shape is storage-agnostic so that migration just
  repoints `supersedes_field` (D5).

## Notes

- **File coherence / disjointness.** PHASE-01 owns `src/lifecycle.rs` (new) +
  `src/slice.rs` (donor). PHASE-02 owns the mutation seam (OQ-3: grow `dep_seq.rs` or a
  sibling leaf) + the gov/slice/backlog/requirement setter rewires. PHASE-03 owns the new
  `supersede` handler + `main.rs` wiring. PHASE-01↔PHASE-03 are largely file-disjoint, but
  PHASE-02 depends on PHASE-01's leaf and PHASE-03 depends on PHASE-02's cores — so the
  three run **serial**, not dispatched (handover). Authoring doctrine entities is not
  dispatchable regardless.
- **Three execution-detail OQs ride into the phases** (design §9, decide by smallest
  green diff): OQ-1 test home (PHASE-01), OQ-3 seam module name (PHASE-02), OQ-2 hint
  wording (PHASE-02/03). None are blocking.
- **Follow-ups F1/F2/F3 are minted at CLOSE**, not now (design §9): F1 destructive
  verbs (the carved-out IMP-006 axis), F2 supersession for POL/STD/slice (vocab growth),
  F3 the SL-048 OD-3 `supersedes` typed→`[[relation]]` migration this verb unblocks.
