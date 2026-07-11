# IMP-280: RFC-019 Phase C: elicitation queue and capture loop

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

RFC-019 Phase C — the elicitation side of comparison-based value capture.
Phases A (SL-210, capture) and B (SL-213, inference) are shipped; nothing yet
*asks* for judgements. **Needs-gated on CHR-042** (empirical evaluation of
Phase B — the RFC's explicit entry criterion; see its report at
`.doctrine/rfc/019/phase-b-evaluation.md` once written).

## Scope (from RFC-019 §Implementation path, Phase C)

- `doctrine value elicit [--lens L]`: ranked candidate queue —
  guaranteed-yield × decision-impact ordering, reasons attached,
  JSON-emittable for an agent curator; un-compared items enter by binary
  insertion against the projected median.
- Capture loop: render both items with enough context to choose (title, body
  summary, deps, risk), accept the answer, append to the ledger.
- Queue is a pure function over `(bounds, costs, ledger, statuses, context)`.
- Estimate/value edits re-surface reprobe candidates through the same
  determinacy check.
- Offers audience/session-scoped, not kind-gated; selector prioritises human
  confirmation of agent-seeded orderings.
- Queue head = naive selector, ships first; agent curation layers on as a
  skill, not engine code. Sequencing decision context ships here.

Non-trivial: this is slice-scale work — route through `/slice` when picked
up, not a quick-fix. Carries the algorithmic risk of the RFC (question
selection quality).

## References

- RFC-019 §Pair selection, §Implementation path Phase C
- CHR-042 — entry-gate evaluation (prerequisite)
- SL-213 — the inference layer this consumes
