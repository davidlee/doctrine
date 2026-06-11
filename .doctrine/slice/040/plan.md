# Implementation Plan SL-040: RV review-ledger kind and review verb family

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases build the RV review kind bottom-up — leaf, then authored kind, then
the live coordination seam, then teeth, then cache, then the pilot — so each
phase ends on a green, demonstrable increment and the highest-risk code
(`with_turn`/CAS) lands on a foundation already proven by tests. The slice is
large but not sprawling (one coherent capability, governed wholly by ADR-007);
phasing absorbs the size. Design is locked (`design.md`, round-1 inquisition
survived); this plan only sequences it — it re-decides nothing.

## Sequencing & Rationale

**PHASE-01 (pure core) first because everything downstream depends on it and it
has zero I/O risk.** `contentset` (D3) and the `review.rs` pure core — the enums,
`derived_status` (§8), `can()` (§5), render fns — are pure functions the whole
slice leans on. Unit-testing them in isolation pins the status algebra and the
transition graph before any file, lock, or CLI surface can confound a failure.
ADR-001 layering falls out naturally: the leaf is built before its consumer.

**PHASE-02 (authored kind) next — make an RV exist before making it move.** Engine
registration (`KINDS` row), install wiring (manifest dir + gitignore negation +
embedded templates), the D2 scan-path id-only reader, and `review new`/`show`/
`list`. The increment is a creatable, showable RV with an empty ledger — the
empty-ledger `Active`/`await=Raiser` state (D-C8) is the first end-to-end proof
that derived status works off real authored bytes. D2 is sequenced here because
the id-only reader is what lets a status-less review toml survive `scan_kind`;
the matching half of its test (a corrupt non-review toml still hard-fails) guards
the shared `Meta` contract at the same time.

**PHASE-03 (verbs + coordination) is the heart and the highest risk.** The
`with_turn` seam (D6) — lock, the twice-fired CAS (entry + pre-write), authored-
first/baton-last ordering — is greenfield concurrency (R-c) with no in-repo
precedent for the file lock or the lost-update guard. It is sequenced as its own
phase, after a working authored kind exists to exercise, so the no-clobber
scripted-interleaving tests (the proof obligation) run against a real ledger. At
this phase's exit the RV is fully usable **without** the warm-cache — the
mid-slice checkpoint the scope calls for.

**PHASE-04 and PHASE-05 both depend only on PHASE-03 and are independent of each
other**, so they can proceed in either order (or concurrently). PHASE-04 adds the
lifecycle teeth — `unresolved_blockers_for` and the close-shell gate (D8/D-C9) —
the one-way `slice-shell → review-query` coupling, with the sole-caller VT
(Charge VIII) guarding the shell-injection choice. PHASE-05 adds the warm-cache
and `prime` (D9/D-C10); it is deliberately **late** so the RV-without-cache
increment is demonstrable first and the cache reads as the optimization signal it
is, not a gate. PHASE-05 reuses the per-review lock and `contentset` that
PHASE-03 and PHASE-01 already shipped.

**PHASE-06 (the `/audit` pilot) is last because it consumes the whole stack** —
verbs, teeth, and cache — as a single proof-of-integration. Rewiring exactly one
skill (D11; the rest are IMP-023) closes the `audit.md` scaffold gap structurally
and gives the slice its end-to-end VA acceptance. The final `just check`/clippy
gate rides here.

## Notes

- **Criteria immutability.** `PHASE-NN` and `EN-/EX-/VT-/VA-` ids are fixed once
  authored — corrections append, they never renumber.
- **Verification provenance.** Every VT/VA traces to design §14's matrix; this
  plan only assigns each obligation to a phase (§14 left phase assignment to
  `/plan`). The matrix's `just check`/clippy row is split: each phase carries its
  own zero-warning gate, and PHASE-06 carries the whole-workspace final gate.
- **Tensions go up, not sideways.** Any conflict surfaced against ADR-007 during
  execution returns through `/consult` (the slice re-decides nothing — scope §
  Context).
- **Deferrals already have backlog homes** (no closure surprise): IMP-022 (Drift
  Ledger), IMP-023 (remaining skill rewires), IMP-024 (large-review funnel /
  subject-root / per-worktree reconciliation), IMP-025 (promote `contentset`),
  IDE-002 (durable region anchor).
