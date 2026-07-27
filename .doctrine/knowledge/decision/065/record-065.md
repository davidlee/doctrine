# DEC-065: Design coordination uses small orthogonal state models

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 uses a small coarse design-run stage vocabulary—initially `exploring`,
`inquiring`, `drafting`, `reviewing`, and `locked`—without treating every
observable condition as another state on that FSM.

Inquiry-node lifecycle, the active cursor, dependency blocking, traversal
direction, section alignment, review findings, and the next turn obligation are
separate stateful dimensions or derived views where needed. They must not be
folded into compound run states or disguised as nested substates.

The coordinator mechanically enforces only load-bearing boundaries. Loops,
question selection, approach generation, decomposition, and most conversational
sequencing remain intelligent-model judgement constrained by explicit
obligations and invariants.

V1 does not introduce a hierarchical-state-machine framework, external workflow
graph language, or generic interpreter. The stage vocabulary and pure transition
classifier may evolve through ordinary versioned product changes; cheap
adjustability is preferred over speculative configurability.
