# IMP-474: KeyContract home duplicates WIRE_KEYS

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SL-264` PHASE-02 (`a67060fe5`) added `home: Option<KeyHome>` to every
`KeyContract` (`src/design_run/payload_contract.rs`) so `blocking` renders once
per wire home. Only the `DECLARATION` block sets a value; those values copy the
home column of `Declaration::WIRE_KEYS` (`src/design_run/submission.rs`), the
table the engine actually enforces through `inert_key`. The two are held
together only by `the_payload_contract_names_every_declaration_keys_home`
(`src/design_run/tests.rs`), and the general contract type carries ~72
`home: None` rows for one struct's concern.

Design-sanctioned in `SL-264` (`design.md` sec-5), so not a defect of that
slice; raised as `RV-389` `F-5` and routed here. It extends a pre-existing
duplication: `DECLARATION`'s key list already mirrors `WIRE_KEYS`, likewise
held by a sync test.

Candidate shape: the `DECLARATION` renderer reads each key's home from
`WIRE_KEYS` (or generates `DECLARATION`'s key rows from `WIRE_KEYS` plus a type
column), leaving `KeyContract` unchanged and retiring both sync tests.
