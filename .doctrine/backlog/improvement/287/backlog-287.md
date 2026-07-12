# IMP-287: RFC-019 Phase E: scoping context and cross-domain yield

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

RFC-019 Phase E: scoping decision context + cross-domain yield.

- `DecisionContext::Scoping { budget }` slots into SL-217 D3's seam:
  dependency-closed greedy cut at per-invocation budget (RV-260 F-7
  skip-and-continue algorithm), membership-stability band rendered
  explicitly; cut report surface.
- Cross-domain nomination: estimate-refinement questions ranked alongside
  comparisons in the queue.
- **Entry criterion (RV-260 F-5):** formal estimate feasible-region model
  over `(lower, upper)` — what an answer constrains, what the hypothetical
  answer space is — designed and reviewed before this opens. Until then
  estimate questions stay curator-surfaced (D17).
- Sequencing: after IMP-286 (estimate domain); IMP-281 (REQ kinds join
  VALUE_BEARING) before/with.
- Vocabulary caution: ratio/band rows void the D8 marginal-exactness lemma —
  the phase admitting them revisits `determined` (LP or richer propagation).
