# IMP-178: toml_edit root-insert positions keys above child table headers — read-order wrong, write is corruption-safe

Captured from [[mem.fact.doctrine.toml-edit-root-insert-above-headers]].

`toml_edit` root key renders above all child table headers. Root
insert-if-missing is corruption-safe but the read-order is semantically wrong.
Tagged `sl-136` in the memory; whether that slice already addresses this is
unverified — if so this item can be closed.
