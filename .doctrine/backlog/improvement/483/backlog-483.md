# IMP-483: Stored proposals preserve Sparse null

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A delegated proposal's declarations are stored in the design-run snapshot at
`Propose` (`src/design_run/run.rs`, `DelegationAct::Propose`) and replayed at
`Accept`. `Sparse::Null` and `Sparse::Omitted` both serialise as none
(`src/design_run/submission.rs`, `impl Serialize for Sparse`), TOML drops the
key, and the stored declaration reads back as `Omitted` — so a `null` (clear)
in a proposal became a persist. Found as `RV-389` `F-16` (`SL-264`); inherited
from `SL-233`'s delegation work, affecting every `Sparse` field (`question`,
`parent`, `needs`, `blocking`).

`SL-264` closes the silent half: `Propose` refuses any `null` outright, so
nothing is dropped unannounced. The cost is capability — a proposal cannot
clear a field. This item restores it: give the stored proposal an encoding that
distinguishes `Null` from `Omitted` (bounded by
`mem.fact.design-run.snapshot-outlives-the-binary` — stored snapshots must stay
readable), then lift the refusal.
