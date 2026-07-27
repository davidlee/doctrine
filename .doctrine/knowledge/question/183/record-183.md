# QUE-183: Sparse redeclaration protocol for design-run mutation

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Should the design-run mutation API accept a sparse JSON set/array of declarations
or compact DSL fragments, interpret a repeated node identity as a legal partial
redeclaration, and preserve every omitted subject and field? Which deletion,
clearing, ordering, stale-revision, atomicity, and error semantics keep this
low-friction without turning omission into an ambiguous command?

Answered by DEC-063: canonical v1 input is an atomic, revision-guarded,
idempotent sparse JSON declaration batch with explicit null/empty clearing,
whole-candidate validation, and no deletion. A DSL may later be a thin parser
adapter over the same declaration types.
