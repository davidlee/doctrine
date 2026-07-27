# DEC-064: Design runs expose one structured turn envelope

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The pure design-run coordinator produces one bounded structured `TurnEnvelope`
containing the run reference and revision, workflow state and next obligation,
active inquiry path, nearby frontier summary, blockers, durable semantic
references, allowed declaration shapes, and material changes since the caller's
known revision.

Prompt Markdown, machine JSON, and broader human status are projections of this
same envelope, not independently assembled state. The prompt projection stays
bounded as the full map and knowledge graph grow.

SPEC-023 remains responsible for resolving stable canonical guidance. The
design-run coordinator owns the point-in-time obligation and state projection;
its prompt renderer composes the resolved stable fragment with the dynamic
envelope. Required transition semantics live in the coordinator and cannot be
hidden behind an optional prompt pull.

The static `/design` skill becomes a thin activation/recovery adapter: locate or
start the run, obtain the next projection, obey its submission contract, and
surface refusals. It does not retain a parallel workflow implementation.
