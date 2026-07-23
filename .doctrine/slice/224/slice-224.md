# Honest dispatch refusals

## Context

Descends from RFC-016 (zero-rescue dispatch), **Cluster 1 / move C** — operator
knowledge and refusal quality mechanised into the verb. The RFC-011 case-notes
analysis (`.doctrine/rfc/011/case-notes-analysis.md`) ranks *selector
under-declaration* the single highest-frequency funnel burner (8+ repros), and
its sibling *empty `Refused.detail`* (4 repros) forces source-diving
`import.rs` / `conformance.rs` on every refusal. Both share one seam: the
`dispatch_import` scope belt refuses `undeclared-scope` but tells the
orchestrator neither *which* paths offended nor, at plan time, that the plan's
selector set was incomplete to begin with.

The zero-rescue move: a refusal must carry its own prescription (name the
offending paths), and the incompleteness that causes the refusal must be caught
**at plan time**, not at import time N phases later.

## Scope & Objectives

1. **Name the offending paths in the refusal.** `dispatch_import`'s
   `undeclared-scope` refusal populates `Refused.detail` with the concrete
   undeclared paths (and, where cheap, the nearest declared selector), so the
   orchestrator acts on the message instead of source-diving the belt.
2. **Plan-time selector-completeness lint.** Flag VT `test_file` paths (and
   other plan-declared touch targets) that are not covered by a `design-target`
   selector — the recurring "scope-relevant doesn't clear the belt; must be
   design-target" trap (SL-211, SL-219, SL-221). Surfaces at plan/phase time,
   before dispatch.

Closure intent: a fresh orchestrator hitting an undeclared-scope refusal
resolves it **from the refusal text alone**, and the plans that produced the 8+
repros would have been flagged before dispatch.

## Non-Goals

- The false-red / environmental-red funnel burners (worker_commit gate binary,
  marker-aware e2e skip, coord-worktree sync) — those are **SL-225** (move B).
- The `dispatch next` state machine and memory-blind benchmark — RFC-016
  Cluster 2 / move A, sliced later.
- Reworking the `design-target` vs `scope-relevant` taxonomy itself — this slice
  makes the *existing* taxonomy legible and checkable, it does not redesign it.
- Auto-widening scope on the orchestrator's behalf — refusals stay
  report-and-halt; we make the halt self-explanatory, not automatic.

## Affected surface (coarse — `/design` refines)

- `src/worktree/import.rs` — undeclared-scope refusal, `Refused.detail`.
- `src/conformance.rs` — design-target vs scope-relevant belt.
- `src/plan.rs`, `src/slice.rs` — selector model; plan-time completeness lint.
- `src/vtgate.rs`, `src/commands/check.rs` — VT `test_file` wiring / lint surface.

## Risks / Assumptions / Open questions

- **A:** the undeclared paths are already computed inside the belt (the check
  exists; only the detail is dropped) — cheap to surface. To confirm in `/design`.
- **OQ:** does the plan-time lint live in `slice plan` / `slice phases`, in
  `check`, or both? (Mirror RFC-016 OQ-1: prescribe vs gate.)
- **OQ:** lint severity — hard refusal at plan time, or advisory warning?

## Verification / closure intent

VT: a golden showing `undeclared-scope` `Refused.detail` naming the offending
path; a plan-lint test over a VT-`test_file`-not-in-design-target fixture.
Closes when both land green and the memory-blind refusal is demonstrable.

## Follow-Ups

- Feeds RFC-016 OQ-5 (memory-blind benchmark): honest refusals are a
  precondition for the benchmark's refusal scenarios.
