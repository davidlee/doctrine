# Review RV-132 — design of SL-133

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-133 design inquisition, focused on the post-RV-130 consequence-model revision rather than reheating the stale one-hop review wholesale. Lines of attack:

- recursive `needs` leverage: DP traversal order, SCC condensation, coefficient bounds, reconvergent inflation;
- `ref` optionality and CyclePolicy/overlay membership boundary;
- `seq` structural precedence, strict-`<`/ULP scoring claims, and `next` display/mint ordering;
- ADR-001 layering for risk/facet extraction and ADR-015 policy boundary.

Held against the curated `domain_map` invariants in `.doctrine/review/132/domain_map.toml`.
