# PRD-015: Dispatch & worktree

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

A Doctrine repository is worked by many agents and humans at once. Without
isolation, concurrent work corrupts shared authored state (colliding edits to the
same TOML, a stray `git add -A` sweeping a sibling's untracked file), loses progress
to crashes mid-coordination, and lands unreviewed code on trunk. The need is to let
**multiple units of work proceed in parallel without interfering** — and to fold
their results back into durable state **safely, reviewably, and crash-recoverably**.

The desired outcome: a unit of work runs isolated; its result returns as a reviewable
delta; a single trusted coordinator funnels that delta into durable state and records
the derived knowledge only after the code is confirmed; integration to trunk happens
only after audit, never automatically. The coordinator's own progress is durable, so a
crash is a resume, not a loss. And the capability holds **regardless of which harness
drives it** — no orchestrating harness is privileged, none is required.

This PRD is the product intent realised by the dispatch & worktree mechanism container
(SPEC-012) and its orchestrator-process component (SPEC-021); the governing decisions
are ADR-006 (worktree posture), ADR-011 (harness-agnostic spawn), and ADR-012
(integration topology).

## 2. Scope

**In scope.** Isolation of a unit of work; the sole-writer funnel from isolated delta
to durable coordination state; non-leakage of the coordination/runtime tier across the
isolation boundary; reviewable per-phase and per-slice surfaces; audit-gated, opt-in,
non-destructive integration; crash-recovery from durable coordination state;
harness-agnostic spawn with an honestly-stated per-harness enforcement altitude.

**Out of scope.** The branching/topology *policy* a project adopts (named roles only,
ADR-006 — bound by the consumer). The mechanism internals of each coordination verb
(`provision`/`fork`/`import`/`land`/`gc`/sync — SPEC-012). The orchestrator's
step-ordering, routing, and operational procedure (SPEC-021). Raw-tree confinement of a
malicious or buggy worker beyond the funnel and the sandbox (deferred to the jail,
ADR-008). Slice/phase planning and execution semantics (PRD-001).

**Boundary.** This capability governs *how concurrent work is isolated and folded back*,
not *what the work is*. A unit of work is opaque to it; the capability only constrains
how that unit is isolated, returned, funneled, reviewed, and integrated.

## 3. Principles

- **The isolation boundary is enforced by construction, not trust.** The coordination/
  runtime tier is *absent* from an isolated unit, so there is nothing shared-mutable to
  corrupt — not present-but-trusted-not-to-be-touched.
- **Exactly one writer touches durable authored state.** An isolated worker returns a
  source delta and never writes doctrine-authored state, runtime state, or memory; the
  coordinator is the sole writer.
- **Knowledge trails confirmed code.** Derived knowledge (memory, evidence, notes) is
  recorded only after the code it describes is confirmed and committed — never ahead.
- **Integration is earned, never automatic.** Work lands on a reviewable surface by
  default; trunk integration is opt-in, audit-gated, non-destructive, and reported —
  never force-pushed, never auto-resolved.
- **Durable coordination state is the source of truth.** The coordinator's working
  context is disposable; a crash is recoverable from committed coordination state alone.
- **No harness is privileged or required.** The capability states honestly what each
  harness can enforce and degrades visibly, never silently excluding a harness or
  requiring a harness-specific command.
- **Policy-agnostic.** The capability names coordination roles and guarantees; the
  consuming project binds the branching topology (ADR-006).

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section that `spec
show` renders. This section carries only the constraints and invariants that bound
every valid implementation.

Constraints:

- The capability must require **no harness-specific command** as a mandatory element
  (e.g. an API-billed subprocess); at least one isolation+spawn path must exist for a
  harness with no worker environment channel (ADR-011).
- The capability must **not bind a branching policy**; it names roles and guarantees
  only, leaving topology to the consumer (ADR-006).
- Integration to trunk must be **opt-in, fast-forward-only, and expected-tip-guarded**;
  it must never force-push, auto-merge, or auto-resolve a moved/conflicting target.
- Worker confinement against a hand-edit or bare commit is **out of this capability's
  reach** and is deferred to the sandbox/jail (ADR-008); the funnel contains it, the
  capability does not claim to confine it.

Invariants:

- No isolated worker ever writes doctrine-authored state, runtime state, or memory.
- The coordination/runtime tier never crosses the isolation boundary into a worker.
- A confirmed integration is always a single reviewable commit per batch on the
  coordination branch, with derived knowledge recorded strictly after it.
- Durable coordination state is always sufficient to recover the run after a crash —
  no progress depends on the coordinator's in-memory context.
- Conflict, a moved target, or an authored-tree touch always halts with a report —
  never a silent auto-resolution.

## 5. Success Measures

- **Zero authored-tree corruption under parallel dispatch** — across a multi-phase,
  multi-agent run, no isolated worker's delta mutates doctrine-authored state, and no
  sibling's uncommitted/untracked work is swept into a coordination commit.
- **One reviewable commit per batch** — every funneled batch yields exactly one
  coordination commit, knowledge recorded after it; the history reads as a clean
  sequence of confirmed batches.
- **Crash ≡ resume** — after an induced coordinator crash mid-run, the run resumes from
  committed coordination state with no lost or double-applied delta.
- **Audit gates integration** — no slice integrates to trunk before its prepared review
  surfaces pass audit; an audit failure preserves all deliverable surfaces for remediation.
- **Harness parity of guarantee, honesty of altitude** — the same isolation+funnel
  guarantee holds on every supported harness, and any per-harness shortfall in
  enforcement altitude is stated, not hidden.
- **Non-destructive integration** — across all integrations, no force-push and no
  auto-resolution occurs; every moved/non-ff target is reported.

## 6. Behaviour

**Primary flow — isolate, funnel, integrate.** A unit of work is dispatched into
isolation; it returns a source delta. The coordinator preconditions the delta, applies
it, verifies, commits one coordination commit, and records derived knowledge. When the
slice's phases are complete, the coordinator materialises reviewable surfaces (per-phase
and per-slice), audit runs against them, and — only on pass — integration projects to
trunk, opt-in and non-destructively.

**Alternate flow — serial dependent work.** Where a later unit depends on an earlier
one's result, the coordinator advances its baseline to the integrated tip before
dispatching the next, so the dependency is present in the next unit's isolation.

**Alternate flow — degraded harness.** On a harness with no worker environment channel
or no pre-dispatch verification moment, the capability still isolates and funnels; the
reduced enforcement altitude is stated, and shortfalls are caught later at the funnel
(at a bounded cost) rather than silently.

**Edge cases & guards.**
- A worker that returns more than a single non-merge delta, or touches authored state,
  is **refused at the funnel** — report and halt.
- A baseline that moved between dispatch and funnel **halts with a report**; the
  coordinator may re-anchor only on a proven-disjoint move, never silently.
- A reap of a spent isolation unit happens **only when its result provably landed** in
  durable git state — never on a disposable receipt that could survive a crash and lie.
- Concurrent dispatch of the same slice is **refused at creation**.

**Failure modes.** Conflict on apply, a moved integration target, an authored-tree
touch, or an unrecoverable verification failure all resolve to **report-and-halt**.
Integration never force-pushes or auto-resolves; a moved trunk tip is reported and
requires an explicit, re-based resolution.

## 7. Verification

The capability is proven primarily by the funnel's own guards and by induced-failure
exercises, not by unit tests alone. Isolation non-leakage is verified by construction —
the coordination/runtime tier is demonstrably absent from a provisioned unit even under
a maximally-broad allowlist — and corroborated by a static smell test whose green result
is explicitly *not* treated as completeness. The sole-writer invariant is verified by the
worker-mode guard refusing authored/coordinator-class writes under isolation, exercised
across harnesses. The single-commit-per-batch and knowledge-trails-code obligations are
verified by inspecting funneled history. Crash-recovery is verified by inducing a crash
between funnel steps and confirming a clean resume from committed coordination state.
Non-destructive, audit-gated integration is verified by compare-and-swap projection
exercises that confirm a moved target is reported, never force-resolved, and that trunk
integration cannot precede a passing audit.

Coverage of the functional and quality obligations is tracked against the requirement
entities (the `REQ-NNN` members synthesized below), not restated here. Where a check
cites a specific obligation it references the durable requirement entity, never its
mobile membership label. The per-harness enforcement altitude is itself a verification
obligation: each harness's reachable altitude is stated and the shortfalls named, so a
reviewer can confirm the guarantee holds and the honesty is kept.

## 8. Open Questions

- **OQ-1 — Positive coordination-tree identity.** Today the coordinator's
  write-permission rests on marker-*absence*, indistinguishable from an unstamped
  worker; the funnel + jail are defence-in-depth, not a coverage proof for the full
  coordinator verb class. A positive coordination marker (IMP-065) is the real close.
  Blocks: a provable, rather than fenced, sole-writer identity.
- **OQ-2 — In-coordinator re-anchor onto a moved baseline.** Re-anchoring a delta onto a
  moved baseline is today an out-of-band, proof-gated coordinator act; folding it into
  the funnel with a content-base assertion (IMP-043) would make parallel *landing*
  (not just execution) first-class. Blocks: throughput of concurrent landing.
- **OQ-3 — Worker raw-tree confinement.** Confining a worker that hand-edits or bare-
  commits outside its isolation unit is deferred to the sandbox/jail (ADR-008). Whether
  the framework should reach further than the funnel + jail is unresolved. Blocks: the
  worst-case bound on a buggy/hostile worker.
