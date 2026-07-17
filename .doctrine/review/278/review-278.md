# Review RV-278 — design of SL-221

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

- Probe D1's merge law against the actual write surfaces: whether `ref-wins-existing`
  is sound once both `record_boundary` and `conclude_boundary_commit` are known
  to UPSERT by phase rather than append-only.
- Cross-examine the ordering claim in §5.3 against `plan_phases`: whether
  append-order can perturb the phase chain that prepare-review materialises.
- Test the claimed invariant restoration against ADR-012 / SL-064 §4.1: sync
  reads from the committed ref, working-tree reads stay on the funnel RMW side.
- Judge VT-1..VT-4 for sufficiency: clobber regression, idempotency, duplicate-
  phase divergence, and unchanged-suite behaviour-preservation.

## Synthesis

The design is tainted by two substantive false premises and one verification gap.
First, D1's `ref-wins-existing` rule rests on a confession the code will not
make: same-phase overlap is not pathological but an explicit UPSERT path on both
boundary writers, so silently discarding a newer working correction can preserve
the wrong boundary in committed truth. Second, §5.3 falsely blesses append-order
by claiming `plan_phases` is order-insensitive when it in fact chains phase refs
strictly in ledger row order. Third, VT-1..VT-4 do not try the very overlap and
ordering cases D1 depends on, so the proof is weaker than the design proclaims.

One administrative blemish remains on the ledger: `F-1` was recorded with shell-
mangled prose and is superseded in substance by `F-2`. Let no one mistake that
clerical scar for exculpation; the usable charges are `F-2` through `F-4`, and
the accused design still merits correction before implementation proceeds.
