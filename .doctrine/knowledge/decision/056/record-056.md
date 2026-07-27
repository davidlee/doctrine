# DEC-056: Design-specific contract with extraction-friendly seams

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 will define a new product and technical contract specifically for
Doctrine-managed `/design` runs. SPEC-023 remains a collaborating
prompt-composition contract rather than absorbing mutable workflow authority.

The design should notice adjacent workflows and take cheap opportunities to
keep names, pure transition seams, versioned wire boundaries, and projections
extractable. It must not introduce a generic behaviour-run public contract,
profile language, or extension framework without evidence from a contrasting
workflow.

This chooses bounded vertical-slice semantics over both premature
generalisation and folding orchestration into the prompt cascade.
