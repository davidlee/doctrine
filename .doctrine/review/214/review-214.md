# Review RV-214 — design of SL-190

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial pass on SL-190's design (`design.md`) via codex mcp
(GPT-5.5), Inquisitor posture. Lines of attack: composite phase-truth soundness
(is the resolution table total/unambiguous?), cross-tree write safety vs ADR-006
sole-writer, the gc landed-oracle "behaviour-preserving lift" claim, POL-002
boundary (which inspections are host-neutral), ADR-001 pure/imperative split, and
parallel-implementation risk vs IMP-174 / IMP-191 / SL-180. Held to: ADR-006,
ADR-012 (preserved `phase/*` refs), ADR-001, POL-002; the storage-tier discipline
(authored / runtime / derived).

## Synthesis

**Verdict: the design was heretical at its foundation — and is now reconciled.**
Seven charges raised, all confirmed against source, all disposed `design-wrong`
and verified terminal. Three were blockers:

- **F-1 (blocker).** The load-bearing claim — that `boundaries.toml` is a
  "committed, durable" registry underwriting cross-machine landed recovery — was
  false. `.doctrine/state/` is gitignored (`.gitignore:56`); `write_registry` is
  annotated disposable (`state.rs:642`); `git ls-files` tracks zero. The durable,
  git-native, cross-machine artifact is the immutable `phase/<slice>-NN` ref cut
  (`PHASE_REF_PREFIX`, read via `count_phase_refs`/`for-each-ref`). Penance: the
  composite's "landed" is regrounded refs-first, cache-fallback; the registry is
  correctly named a primary-local derived cache. This *strengthened* the
  cross-machine story (refs travel via fetch; the prior draft had no travelling
  landed source at all).
- **F-2 (blocker).** `resolve_phase_truth` was fed `PhaseRollup` bucket counts
  (`state.rs:74`), which cannot resolve *which* phase holds *which* status. Penance:
  the core now takes per-phase status maps from a per-phase reader, not `fold_rollup`.
- **F-3 (blocker).** Reconcile reused `set_phase_status`, which appends a
  `[[progress]]` row and mutates the boundary registry (`state.rs:424,452,461`) —
  reconcile would clobber the very source it read as truth, non-idempotently.
  Penance: a dedicated status-only writer, with a no-side-effect proof in
  verification.

The majors: F-4 (truth table made total, with an explicit catch-all and phase-set
drift → UNKNOWN); F-5 (cross-tree write no longer leans on sole-writer-as-lock —
reconcile writes only the primary tree and refuses when a live coord tree exists);
F-6 (the gc-oracle lift split into a behaviour-preserving *extraction* + a new,
separately-tested *generalized-target* contract). The minor F-7 fixed the shared
selector predicate's home in design (`conformance.rs`), leaving only land-order to
plan. POL-002 was interrogated and **cleared** — `dispatch/<slice>` is ADR-012
doctrine-owned topology, not a host convention; no charge.

**Standing risks (consciously carried, not defects):**
- Cross-machine landed recovery is contingent on the `phase/*` refs being fetched;
  never-committed in-flight state is inherently unrecoverable. Documented limit.
- Concurrency: doctrine has no cross-machine lock; two operators reconciling one
  repo concurrently is out of contract (documented, not guarded — consistent with
  the runtime tier). The live-coord refusal (F-5) covers the common in-repo race.

**Follow-up harvested:** IDE-028 (auto primary-sheet-push in the Record beat) —
the deferred automation of the manual `reconcile-phases` this slice ships; linked
`after` + `originates_from` SL-190.

**HERESIS URITOR; DOCTRINA MANET**
