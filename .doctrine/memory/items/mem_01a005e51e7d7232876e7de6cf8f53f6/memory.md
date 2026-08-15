# A downward layering edge can still be a cohesion violation

`tests/architecture_layering.rs` enforces **tier direction** (leaf ← engine ←
command) and a per-tier cyclic-edge ratchet. It says nothing about whether the
module you are adding a function to should *own* that function.

So "the new edge is engine → leaf, therefore downward, therefore it cannot join
a cycle, therefore it is safe" is a complete answer to the gate and **not** an
answer to the design question. A gate-clean edge can still put per-kind
knowledge into a module whose whole value is being kind-blind.

## The instance

`SL-238` drafted a per-kind status reader — one that branches on `RV` and `REC`
— as `meta::authored_status(root, kref, id)`, and defended the resulting
`meta → kinds` edge on direction. But `src/meta.rs`'s own module doc says the
module carries **zero per-kind knowledge** (`meta.rs:3-13`), and it has ~17
consumers whose safety rests on exactly that. The edge was legal and the siting
was wrong.

## What to do instead

Before adding a function to an existing module, read that module's **doc
comment**, not just its tier row in `.doctrine/adr/001/layering.toml`. The
modules in this crate state their charters explicitly and several state what
they deliberately are *not* (`meta` is "deliberately not `entity.rs`";
`entity` is "a kind-blind scaffold engine").

A new module is often cheaper than it looks. Check whether some existing module
already carries the import set you need — `src/integrity.rs:19-20` already
imports `kinds`, `meta` and `entity` — because if one does, a new module with
that same set adds **no new module edge at all** and cannot move the tangle
baseline.

See [[mem.signpost.doctrine.adrs]].
