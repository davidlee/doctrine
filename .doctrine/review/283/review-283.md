# Review RV-283 — reconciliation of SL-221

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject / surface.** SL-221 — unify dispatch boundary writes on the object-db
ref. Dispatched slice: reviewed the **candidate interaction branch**
`candidate/221/review-001` (tip `06442a5e`, base `refs/heads/main`), the
main-based projection of the `review/221` impl bundle (`c7e3fe73`). Evidence refs
`phase/221-01..05` are immutable; the candidate is the audit surface (per
`review-ledger` dispatched-slice caveat).

**The thesis under audit.** Collapse the split write/read seam: make the
`dispatch/<slice>` object-db ref the single source of truth for boundary rows and
retire the working-tree `boundaries.toml` entirely — killing the ISS-225 clobber
(prepare-review re-committing a stale working-tree ledger over concluded rows).

**Lines of attack / invariants held:**
1. **ISS-225 dead by construction** — prepare-review can no longer clobber
   concluded rows (VT-1 P05).
2. **ADR-001 clean** — the relocated object-db engine forces no reverse
   `dispatch → mcp_server` edge; `FUNNEL_MARKER` moves with `funnel_message`.
3. **Behaviour-preservation** — `commit_on_behalf` (now explicit `target_ref`) and
   `land_boundary_row` (the single writer) preserve empty-delta / lost-ref-race /
   UPSERT-by-phase invariants; conclude delegates unchanged.
4. **Consumer-normalised order** — `plan_phases` chains by phase ordinal, not row
   order (an out-of-order escape-hatch record cannot mischain ancestry).
5. **Working-tree path fully retired** — no production code reads/writes the
   working-tree ledger; grep-clean (VA-1 P01/P05).
6. **No silent coverage loss** — the worker's beyond-plan migration calls
   (vt6, round-trip, upsert deletion) preserve the invariants they inherited.
7. **Conformance** — the design-target selector set matches what git touched.

**Evidence gathered (candidate worktree):** `doctrine check gate` green
(clippy zero-warning); `dispatch::` unit tests 94 passed; `e2e_dispatch_sync` 42
passed (incl. `prepare_review_cannot_clobber_concluded_rows`,
`record_boundary_lands_on_the_dispatch_ref`,
`record_boundary_also_writes_the_arm_neutral_registry`, `vt6…`);
`e2e_dispatch_lifecycle` compiles + runs. Primary-tree `slice conformance`:
4 conformant, 0 undelivered, 1 undeclared (`tests/e2e_dispatch_lifecycle.rs`).

## Synthesis

**Closure story.** SL-221 delivers its thesis: the split write/read seam is
collapsed onto the `dispatch/<slice>` object-db ref, and the working-tree
`boundaries.toml` is retired as a production surface. The ISS-225 clobber is dead
**by construction** — with no `commit_boundaries` splice, prepare-review sources
boundary rows only from the committed ref, so a stale coord working tree cannot
overwrite concluded rows (`prepare_review_cannot_clobber_concluded_rows`, VT-1
P05, passes; the operator `git restore <ledger paths>` ritual is obsolete). The
five phases land the design's five moves cleanly:

- **P01** relocated the object-db engine down to the engine tier with `FUNNEL_MARKER`
  moved alongside `funnel_message` — no reverse `dispatch → mcp_server` edge
  survives (ADR-001 clean, grep-verified); `commit_on_behalf` now takes an explicit
  `target_ref`, its empty-delta / lost-ref-race / byte-unchanged invariants intact.
- **P02** introduced `land_boundary_row` as the single boundary writer (UPSERT-by-phase
  → splice → `commit_on_behalf`); conclude delegates, behaviour-identical.
- **P03** rewired the `record-boundary` escape hatch onto the ref working-tree-free,
  preserving the arm-neutral `record_source_delta` registry write
  (`record_boundary_lands_on_the_dispatch_ref` + `…_also_writes_the_arm_neutral_registry`).
- **P04** made `plan_phases` normalise by phase ordinal before chaining — the
  truthful replacement for the retracted "single writer ⇒ ordering cannot arise"
  claim; an out-of-order escape-hatch record can no longer mischain ancestry.
- **P05** deleted the working-tree path (`commit_boundaries`,
  `read_boundaries_file`, `record_boundary` + the four runtime-only splice e2es);
  grep-clean.

Gate green (clippy zero-warning), 94 unit + 42 sync e2e tests pass.

**Standing risks / tradeoffs consciously accepted.**
1. **Authored selector divergence (F-1).** Edge's `slice-221.toml` lacks the
   `tests/e2e_dispatch_lifecycle.rs` design-target selector (a real P03 scope
   discovery), so edge conformance is red (1 undeclared) until integrate's
   class-routed projection carries the dispatch/221 selector edit onto edge/main.
   A reconciliation item, not a code defect — routed to the brief below.
2. **IMP-272 tree split (F-4).** prepare-review's completeness gate reads the
   PRIMARY tree's phase sheets while conclude flips only the COORD tree's — forcing
   a manual `slice phase … -p <primary>` flip ×5 (a true-state reconciliation, 2nd
   live repro). Accepted operational drift; the machinery fix is owned by IMP-272.
   This is also why `slice conformance` run *from the candidate worktree* reports
   "incomplete" (unflipped sheets) — the primary-tree run is authoritative.
3. **Defensive coverage boundary (F-3).** `plan_phases`'s malformed/non-PHASE-NN
   sort-last branch is correct by inspection but unreachable by construction and
   untested. Low value to test; recorded, not remediated.
4. **Foreign trunk drift (integrate-time, not a payload defect).** The candidate
   (main-based) carries SL-222 / review-282 / case-notes content the earlier-cut
   `review/221` bundle did not. This is normal integrate-time trunk drift that the
   class-routed projection will carry at /close — surfaced here for awareness, not
   a divergence of SL-221's work from its design.

Beyond-plan worker judgment (vt6 migration, upsert-test deletion, round-trip
migration) was audited to closure and confirmed sound (F-2): no coverage silently
lost.

## Reconciliation Brief

### Per-slice (direct edit)
- **`slice-221.toml` selector registry (F-1)** — the load-bearing change is
  `doctrine slice selector add tests/e2e_dispatch_lifecycle.rs` (intent
  `design-target`) so edge/main's authored registry matches the 5-file touch set.
  This edit is **already authored on dispatch/221** and present on the candidate;
  the reconcile/close path must ensure integrate's class-routed projection lands it
  on edge/main. **Acceptance:** post-integrate, primary-tree
  `doctrine slice conformance SL-221` reports 0 undeclared / 0 undelivered. (The
  design §6 selector mirror, if any, follows the registry — the registry is
  authoritative for conformance.)

### Governance/spec (REV)
- None. No ADR / spec / requirement change is implied by this slice's audit. The
  design already carries the truthful `plan_phases` ordering narrative (§5.2e′,
  §5.3, §5.5) — no design-vs-code divergence found.

### Not a reconcile surface (tracked elsewhere)
- **IMP-272 (F-4)** — the coord/primary phase-sheet tree split is machinery, owned
  by the existing backlog item; no SL-221 artefact edit. Do not fold into this
  slice's reconciliation.
