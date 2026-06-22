# REV REV-009 — ADR-015 tag multiplier: delta-from-default form

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

**What & why.** ADR-015 §1 stated the tag term as a literal `Σ tag_coefficients`.
SL-142's design (and RV-143, the inquisition) exposed this as defective: with
`tag_coeff` defaulting to `1.0`, a literal sum (a) inflates `value_dim` by tag
*count* even when every coefficient is the inert default, and (b) collapses
`value_dim` to `0.0` for the untagged majority (the empty sum). The "absent tags ⇒
identity" clause papered over (b) for the empty case but left the default-tag
inflation and gave no account of demotion or sign.

This REV restates the tag term as a **delta-from-default** multiplier — each tag
contributes its excess over the unit default, floored at `0.0`:

`tag_multiplier = max(0.0, 1.0 + Σ (tag_coefficient − 1.0))`

This makes absent/all-default tags the identity (×1.0), lets a demoting
coefficient (< 1.0) actually reduce `value_dim`, and floors the multiplier so a
stack of demoting tags suppresses rather than inverts the score (closes RV-143
F-6 and folds in F-7's floor as canon).

**Scope.** One `modify` row against ADR-015 §1 — the formula line and the
identity bullet. No requirement status moves; no other ADR touched. Applied
manually (modify rows surface for manual handling).

### Before/after — ADR-015 §1

**Before (formula):**
```text
value_dim = (coefficients.value × value × kind_weight × Σ tag_coefficients) / estimate_midpoint
```
**Before (identity bullet):**
> - absent tags / tag storage not yet shipped ⇒ tag contribution is the identity.

**After (formula):**
```text
value_dim = (coefficients.value × value × kind_weight × tag_multiplier) / estimate_midpoint
tag_multiplier = max(0.0, 1.0 + Σ (tag_coefficient − 1.0))   # summed over the entity's tags
```
**After (identity bullet):**
> - absent tags / all-default-coefficient tags ⇒ `tag_multiplier = 1.0` (the
>   identity); a coefficient that differs from the `1.0` default shifts the
>   multiplier by that delta, and the multiplier floors at `0.0`.
