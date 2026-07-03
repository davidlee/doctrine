# Review RV-242 — reconciliation of SL-191

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit, self-review — orchestrator is
both author and reviewer, roles driven via `--as`).

**Reviewed surface (F-2).** Dispatched slice, but orchestrator-authored throughout
(no worker imports). Audited against the **impl-bundle ref `review/191`**
(tip `a9c568143141`) + the seven `phase/191-NN` evidence cuts, plus the live
in-repo corpus (the overlay reconciliation is data on `dispatch/191`). No candidate
interaction worktree spun — there is no independent worker handoff to review, and
every phase was verified inline as it landed.

**Invariants held:**
- POL-002 — client habits live in the `.doctrine/hymns/` overlay, never in the
  shipped Framework hymn; the universal contract stays host-agnostic.
- ADR-005 hymn cascade — the enriched Framework `role/worker` composes (single
  winner per slot, no ISS-206 doubling), the project habit composes additively.
- Conformance (design §6 selectors vs recorded source-deltas) tells the truth.
- Every phase's `EX-`/`VT-`/`VA-`/`VH-` criteria are satisfied and green.

**Lines of attack:**
1. Selector-registry conformance — is the declared design-target surface honest
   against what git actually touched? (11 undeclared / 2 undelivered — leads.)
2. Deferred criteria — PHASE-05 EX-4 subprocess-arm base-clean parity: decided
   and recorded, or silently dropped?
3. Live composition oracle — role/worker + model/adherence/low compose correctly
   in-repo after the PHASE-07 suppressor removal (EX-3/EX-4/VA-1/VH-1).
4. Gate health — `doctrine check gate` green, clippy zero-warnings.

**Evidence gathered:** `check gate` green (workspace tests + build). `slice
conformance 191` → 11 undeclared / 2 undelivered (dispositioned below). `slice
selector doctor 191` → 1 unmatched (`memory/…hymn-cascade/**`). Live: `prompt
explain --role worker` = FW preamble/core(r0) + FW role/worker(r1, single) + User
project/doctrine-rust-conventions(r2); `--model adherence/low` additionally
composes FW model/adherence/low. VH-1 signed off 2026-07-04.

## Synthesis

SL-191 lands the dispatch worker contract as a composing hymn cascade: the
enriched, host-agnostic Framework `role/worker` (P02) + the trait-space model band
`adherence/low` (P02) + the trait-bake plumbing (`worker_context`, agent-def
frontmatter parser, `traits_covered`; P01/P03), the author-time `check_corpus`
guards (P04), the non-mutating `prove` base-clean cadence and reject-and-halt
import gate (P05), the corpus knowledge refresh (P06), and finally (P07) the
in-repo overlay reconciliation that lets the enriched Framework contract actually
compose instead of being suppressed by the old self-`replaces` twin.

**Closure story.** Every phase's criteria are green: `check gate` passes (workspace
tests + build), clippy is zero-warning, all VTs pass (`verify-vt 191`), and the two
live oracles confirm the end-state — `prompt explain --role worker` shows a single
Framework `role/worker` winner with the User project habit composing additively (no
ISS-206 doubling, enrichment present), and `--model adherence/low` additionally
composes the model band. VH-1 is signed off: the POL-002 split is honoured (the
overlay carries only doctrine-repo concretisations; the universal contract stays in
the shipped Framework hymn).

**Findings — both minor, both terminal, no blockers.** F-1 (selector-registry
drift) and F-2 (EX-4 fork-arm parity deferral) are the only divergences, and
neither touches code correctness. F-1 is a truth-in-registry gap routed to
reconcile; F-2 is a conscious, recorded deferral routed to backlog.

**Standing risks / tradeoffs consciously accepted.**
- The overlay reconciliation is *this-repo-only* (hand-managed `.doctrine/hymns/`).
  The projection default that re-writes the self-`replaces` starter on `doctrine
  install` is unfixed by design — ISS-210, explicitly out of scope (design
  Sequencing F2 / user decision 1). A future `doctrine install` in this repo would
  re-project the suppressor; the manifest fix is deferred.
- Band-filter asymmetry (durable design insight): the agent-def bake
  (`resolve_worker_role_body` → `worker_context` = `BandFilter::Only([Role,Model])`)
  excludes the `project` band, while the session cascade (`prompt resolve/explain`
  = `BandFilter::All`) includes it. The `project`-band home for client habits is
  correct *because* EX-3/VA-1 verify the All-bands cascade — a habit reaches a
  worker via SessionStart, not the narrow baked contract. Harvested to memory.
- Fork/subprocess dispatch arm base-clean parity (F-2) is deferred; the shipped pi
  arm is correct and gated.

## Reconciliation Brief

### Per-slice (direct edit)

- **Selector registry (`slice-191.toml` via `slice selector` verbs) + design.md §6
  mirror** [F-1]. Make the design-target registry honest against the delivered
  surface:
  - `slice selector rm 191 src/dispatch.rs` — stale/aspirational; the slice
    delivered no change there (import gate landed in `src/worktree/import.rs`).
  - `slice selector rm 191 'memory/mem.concept.doctrine.hymn-cascade/**'` — glob
    mismatch: the P06 memory is a symlink file, not a directory tree; `/**` matches
    nothing (selector doctor: unmatched).
  - `slice selector add 191 …` (design-target) the delivered P05 surface —
    `src/worktree/import.rs`, `src/commands/check.rs`, `src/verify.rs`,
    `src/corpus.rs`, `justfile` — and the P06 memory —
    `memory/mem.concept.doctrine.hymn-cascade`,
    `memory/mem_88193c2859d72f043ef83a97a5952a96/**`.
  - Mirror the same set into `design.md §6` (the human-readable target list — the
    §6 prose is the mirror, the registry is load-bearing).
  - Coupled test files (`tests/e2e_{worktree_import,memory_sync,prompt_resolve_golden}.rs`)
    are acceptable coupling to the declared code — reconcile's judgment whether to
    declare them; not required for a truthful registry.
  - Verify: `slice conformance 191` clean (or only accepted test coupling remains)
    and `slice selector doctor 191` reports no unmatched.

### Governance/spec (REV)

- None. No ADR / spec / policy / standard change is required — the work conforms to
  ADR-005 (hymn cascade), POL-002 (host independence), and ADR-001 (layering).

### Off-surface (not reconcile)

- F-2 fork-arm base-clean parity → **backlog** (harvested), not a reconcile write
  surface (no design/spec/governance edit; owned future code work).
- ISS-210 (projection default) → already filed; out of scope.
