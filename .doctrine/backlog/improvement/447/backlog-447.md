# IMP-447: Collapse UnknownKeys if it stays uninhabited

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SL-259` `PHASE-03` flipped all eight `UnknownKeys::SilentlyDropped` contract
rows to `Refused` (`EX-3`), so `payload_contract`'s `SilentlyDropped` arm has
**no production inhabitant**. Its only remaining use is the exemplar contract in
the `sec-2` suite and the model-arm census that reads it.

That leaves a two-member enum where one member cannot occur, and a
`TypeForm::Struct { unknown_keys, .. }` field that can only hold one value —
which is the shape `TypeForm`'s own doc argues against: *"a field whose only job
is to be meaningless on half its inhabitants is a slot for a wrong answer."*

**Not done in `SL-259`, deliberately** (`PHASE-03` sheet `D5`): `EX-3` says the
rows become `Refused`, not that the arm dies, and a model that cannot express
the defect cannot describe a type that reacquires it — `ISS-333` was live for
four slices before anyone refused a key.

## The work, if taken

Decide between two, rather than assuming the collapse:

1. **Collapse.** Delete the arm, the `unknown_keys` field, `unknown_keys_token`,
   `unknown_keys_note`, and the per-struct header disclosure; state the rule once
   in the rendering's preamble. Cheapest to read, and the contract stops carrying
   a column with one value.
2. **Keep, and make the arm earn its place.** Leave it as the model's way of
   describing a type that has *not* been brought under the walk — useful the
   moment a wire surface outside `design_run`'s closure wants the same treatment.

Trigger: whenever the contract model is next revisited. If (1), check the
census pin (`the_exemplar_reaches_every_arm_of_the_model`) and
`every_payload_contract_row_refuses_unknown_keys`'s positive control, both of
which read the arm today.
