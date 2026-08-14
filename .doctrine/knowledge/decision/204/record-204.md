# DEC-204: Retire worker_commit's transport, re-home its belts

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Correction at reconcile (2026-08-14, `RV-356` `F-3`)

The dissolution note above says `classify_import` "survives untouched and remains
the belt's enforcing caller". That is true of the **two hard-coded floors**
(`.doctrine/**`, `.claude/**`) and false of the rest of the hard tier as this
record defines it. `classify_import` never read `[dispatch] worker-forbidden-writes`
at all — the matcher's only production reader was `worker_commit`, deleted at
`SL-254` `PHASE-06`, and nothing replaced it.

So this record's own second consequence — *"Confirm `worker-forbidden-writes` still
has an enforcing reader at the admit step, or retire the key with the tool
(`STD-001`)"* — was not discharged in either direction. Neither branch was taken:
the key survives as a **declaration of intent with no production reader**
(`src/dispatch_config.rs:93-95`, `install/doctrine.toml.example:113-115` both now
say so honestly), so its configurable tail — `.agents/**`, `install/agents/**`,
`flake.nix`, "the highest security leverage in the repo" per the shipped example —
enforces nothing for any project that sets it.

No live exposure: no config in this repo sets the key. Escalated during execution
and accepted by the owner for `SL-254`'s scope on 2026-08-13, routed to `SL-255`
and refined so the fix is kernel-level extra `--ro-bind` paths at spawn rather than
a post-import Rust belt — which would be strictly weaker than this slice's own
`PHASE-02/VT-3` standard, *refused by the KERNEL, not a hook*. Carried as `IDE-051`.

`INV-2` as worded survives: it is a claim about the floors, and the floors are
intact. `DEC-213` carries the same error and the same correction.
