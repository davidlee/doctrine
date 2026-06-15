# Doctrine core loop

The one ordering every change in a doctrine repo follows. Don't reinvent it —
follow the skills that own each step.

```
/route → /slice → /design → /inquisition → /plan → /phase-plan → /execute → /audit → /close
 (gate)  (scope)  (author)  (adversarial)   (phases) (runtime)    (TDD)      (evidence)
```

- **Scope first.** `/slice` writes the scope doc + metadata under
  `.doctrine/slice/nnn/` (`doctrine slice new`). No code without a governing slice.
- **Design until locked.** `/design` authors `design.md`; `/inquisition` hunts it
  adversarially. Decisions lock before a plan exists.
- **Plan the phases.** `/plan` turns the locked design into `plan.toml` (objectives +
  EN/EX/VT criteria). `/phase-plan` expands one phase into its disposable runtime
  sheet just before you execute it.
- **Execute one phase, TDD.** `/execute` flips the phase `in_progress`, builds it
  red → green → REFACTOR, ends green, flips `completed`. See [[pattern.doctrine.tdd-loop]].
- **Audit, reconcile, close.** `/audit` reconciles evidence against the design via
  a review ledger (RV kind). `/close` requires a reconciliation gate (ADR-009
  closure seam): audit → reconcile → done. Resolves blockers and drift before
  landing the final commit. See [[signpost.doctrine.audit]].

**Authority order: design / `/canon` outrank the plan.** The plan is a sequencing
tool, not scripture — when it conflicts with the locked design or project
governance, the plan loses. When in doubt, `/canon`.

Mid-flight obstacle, tradeoff, or emergent complexity → `/consult`, don't improvise
past it. The full routing table lives in `.doctrine/state/boot.md`; honour the
conventions in [[pattern.doctrine.conventions]] throughout. See also
[[signpost.doctrine.lifecycle-start]] and [[signpost.doctrine.skill-map]].
