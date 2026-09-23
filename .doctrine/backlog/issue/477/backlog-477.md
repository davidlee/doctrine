# ISS-477: Locked design run accepts apply mutations

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found in `SL-261` design (inq-7, 2026-09-24). Neither `run::apply`
(`src/design_run/run.rs:302`) nor `run_apply` (`src/commands/design.rs`) guards
on `Stage::Locked`, and `Stage::Locked` has no forward gate that re-evaluates
anything (`gate.rs:1016`). So a declaration that changes a section body, or an
adoption, on a locked run invalidates section attestations and the run-level
`design-accepted` act while the stage still reads `locked` — acceptance dead,
nothing surfacing it.

`SL-261` closes this for the new `design adopt` verb only (`DEC-279`: refuse at
`locked`, name the regression). Ordinary `apply` mutations remain open. Decide
which mutations a locked run admits (stage regression certainly; traversal
probably) and refuse the rest with a remedy naming the regression.
