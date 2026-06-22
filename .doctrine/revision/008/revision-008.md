# REV REV-008 — reconcile SL-133

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile of SL-133 (RV-138 Reconciliation Brief). One governance target: ADR-015
(the scoring-policy ADR). Two coherence amendments, both code-no-change — the code
is the coherent reading; the ADR prose lagged.

## Reconcile narrative (SL-133)

- **[RV-138 F-1 / RV-137 F-3]** — `dep_coeff` domain. ADR-015 ratified the domain as
  `(0, 1]`, but the implementation (config.rs `clamp_dep` + tests) and design §5.2/§7
  treat `≤ 0` as the explicit DISABLE sentinel (`→ 0.0`). Amend ADR-015 to ratify the
  domain as `[0, 1]` with `0` (and any clamped `≤ 0`) the explicit disable sentinel.

  - before: ``dep_coeff` is a recursive retention factor in `(0, 1]``
  - after:  ``dep_coeff` is a recursive retention factor in `[0, 1]`, where `0`
    (and any value clamped from `≤ 0`) explicitly DISABLES leverage`

- **[RV-138 F-2 / RV-137 F-4]** — `value_dim` formula. The code applies
  `coefficients.value`; the ADR-015 formula omitted it. Add the factor.

  - before: `value_dim = (value × kind_weight × Σ tag_coefficients) / estimate_midpoint`
  - after:  `value_dim = (coefficients.value × value × kind_weight × Σ tag_coefficients) / estimate_midpoint`

  (The F-4 formula edit was already physically applied to ADR-015 on `dispatch/133`
  @3ddeb307 and rides the candidate→main integrate; this REV records it as the
  governance change it is, and the manual landing reconciles the edge copy.)

No executable change. Per-slice `design.md` edits (§5.1 value_dim, §5.2/§7 dep_coeff
wording) are applied directly (RV-138 Reconciliation Brief, per-slice surface).
