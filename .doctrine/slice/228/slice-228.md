# Zero-rescue dispatch funnel

## Context

RFC-016 Cluster 2 — the RFC's undelivered *core* (Cluster 1, moves B/C, landed via
SL-224 / SL-225 as the cleaned baseline). Tracked by **IMP-311**. The governing
forward-intent was settled by **REV-032** (approved + applied, introduce-only after
adversarial review **RV-300**), which minted 6 `pending` requirements this slice
delivers and reconciles to `active`:

- **SPEC-021** (orchestrator process): FR-008 REQ-384, FR-009 REQ-385,
  FR-010 REQ-386, FR-011 REQ-387.
- **SPEC-022** (git interaction model): FR-010 REQ-388, FR-011 REQ-389.

**The problem.** Dispatch cadence lives as prose the orchestrator must hold
(SPEC-021's eight-step ordered contract) plus a memory corpus of recovery idioms
recalled mid-run — ~40–49k tokens of ceremony + rescue-archaeology per phase
(RFC-011 evidence). Zero-rescue moves the invariants into verbs: the tool tells you
what's next, you do it, you halt on refusal. LLM judgment is reserved for genuine
judgment (conflict content, red-verify triage, scope).

## Scope & Objectives

One shippable change, phased, **move E first** (self-contained, shuts the live
ISS-234 hazard early, retires checkout-import idioms — first OQ-6 win), then the
design-heavier move A on a cleaned base, then the benchmark as terminal acceptance.

- **Move A — funnel state machine (SPEC-021 FR-008/009/010/011).**
  - FR-008: funnel position persisted per-phase as authoritative run-state,
    advancing through explicit transitions **including an authoritative `verified`
    state** (spawned → worker-committed → imported → verified → concluded → reaped);
    single-writer authority, crash-safe idempotent recovery.
  - FR-009: every funnel verb legality-gated on position — refuses out-of-order and
    names the expected next verb; conclude refuses after skipped/failed verification.
  - FR-010: `dispatch next` emits the single prescribed action for the **per-phase
    import→verify→conclude→reap sub-funnel only**.
  - FR-011: one state machine owns the transition semantics; each transport projects
    into that single authority (see Non-Goals for the deferrable part).
- **Move E — read verbs + write guard (SPEC-022 FR-010/011).**
  - FR-010: every funnel git read is a first-class read verb over object-db/ref
    primitives; **no funnel read shells raw git**. Reuse/relocate existing seams
    (e.g. `is_linked_worktree` for isolation detection); build only genuinely absent
    reads. OQ-7's read gaps are largely pre-enumerated.
  - FR-011: funnel is working-tree-free; every coord-tree write is bounded by a
    no-pathless-commit / safe-commit guard — **absorbs ISS-234** (the reverse-diff
    can no longer commit mass reversions).
- **OQ-5 memory-blind benchmark (terminal acceptance).** A fresh orchestrator with
  zero dispatch memories completes a standard run + the top-5 quirk scenarios by
  following verb output alone, measured against the Cluster-1-cleaned baseline
  (SL-224/225). Prioritisation already done (RFC-011 case-notes); the harness +
  measurement remain.

### Pre-design extraction (happens BEFORE `/design`)

**Design is gated on a bounded extraction/recon pass, not open-ended speculation.**
The funnel transition graph is largely *latent in the code already* — a bounded
recon reads `NextGuidance` / `select_guidance` (the deterministic ~7-state advisory
sequencer), `ReceiptStatus`, and **SL-199's confined-arm machine**, and emits the
**as-built state diagram** as an artifact. `/design` then works the *visible delta*
on top of that graph:

1. add the authoritative **`verified`** state (confirm/falsify that it is absent
   today — the claim that most changes move A's size);
2. carve the **per-phase sub-funnel** out of `select_guidance`'s full guidance
   domain (which also routes PrepareReview / Audit / Integrate);
3. pick the **OQ-2 run-state home** and its CAS/concurrency contract.

## Non-Goals

- **Subprocess-arm cross-transport projection (deferrable).** Build the
  single-authority machine so projection is *additive*, and deliver the
  main-thread/claude arm in-slice; forcing the codex/pi subprocess arm (currently
  `worktree fork --worker` + CLI `record-boundary`/`sync`) fully through the gated
  funnel may spin out to a fast-follow. FR-011/SPEC-021 may stay `pending` across a
  second slice without harm.
- **Move-D tail** — IMP-174 (split-brain close), IMP-201 (split-lineage bundle),
  IMP-304 (superseding-candidate rescue→verb): separate follow-ups.
- **OQ-4 candidate auto-sourcing** (default close_target ← repaired review_surface):
  its own sibling revision; explicitly outside the `dispatch next` oracle scope.
- **The 4 active-requirement modifies** (REQ-287/293/294/318): their §-prose
  reconciliation → a **ship-time sibling REV** at this slice's close, when evidence
  exists (retrospective-charter constraint, RV-300 F-1/F-3).
- **IDE-028** interim auto-sync — explicitly the *wrong* path; the durable ISS-234
  fix is read-verbs + write-guard, not auto-sync.

## Summary

Deliver the zero-rescue funnel: read verbs + write guard (move E), the persisted
state machine + gates + `dispatch next` oracle (move A), proven by the memory-blind
benchmark. Extraction-before-design de-risks move A; the benchmark reconciles the
cluster.

## Follow-Ups

Open questions carried into `/design`:

- **OQ-2** — run-state record home (extend `boundaries`/`journal` vs. a new record)
  and its CAS/concurrency + idempotent-recovery contract. The FR-008 durable
  half is in scope; the concrete home is design.
- **NEW-OQ-A** — governing home for the state machine: a **new tech SPEC** for the
  funnel state machine, vs. inline in SPEC-021. Weigh altitude (is the machine
  evergreen mechanism deserving its own spec?) against fragmenting the orchestrator
  spec.
- **NEW-OQ-B** — **derive-from-code vs. drift-resistance**: can the state machine be
  derived from the code (single source of truth), or — if authored as a governing
  spec — what keeps it from drifting? Doctrine's conventions (reconciled coverage,
  requirement entities, `spec validate`) should make a governing spec fairly
  drift-resilient; decide whether a derivation/check closes the residual gap.

Post-close: ship-time sibling REV (the 4 active-requirement modifies); subprocess-arm
projection if deferred; move-D tail; OQ-4 sibling revision.
