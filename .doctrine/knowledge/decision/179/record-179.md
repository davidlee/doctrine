# DEC-179: The edit verbs already share their machinery

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What the question assumed

`inq-12` was posed as a fork — extract the shared read-mutate-write transaction
from three bespoke `edit` verbs, or add a fourth bespoke one. Both branches
assume the transaction is currently triplicated. It is not.

## What is actually there

| verb | shared leaves it already calls |
|---|---|
| `memory edit` | `entity::write_body`, `dep_seq::apply_status` |
| `backlog edit` | `dep_seq::set_authored_status`, `dep_seq::append` |
| `spec edit` | `dep_seq::apply_scalar` (×2, the descent scalars) |

One set of leaf writers, three callers. The extraction the question proposed has
already happened; what is left in each verb is its clap arg set and the mapping
from those args to field names.

## Why that remainder cannot be shared

Memory's thirteen flags, backlog's status/resolution pair and spec's descent
scalars have **no field in common**. An abstraction over them could only be a
dispatch table with one row per verb — a worse spelling of what the compiler
already does through three separate `Args` structs, and one that would have to be
read by everyone who later touches any of the three.

The duplication people see is in the CLI *surface*, where duplication is the
point: each verb states its own domain's fields, which is exactly what makes the
wrong field unspellable rather than refused.

## The confirmation from this slice's own seam

SL-249 designs the same shape from the other end and lands in the same place.
`plan_facet_edits` is pure and knowledge-specific. `apply_facet_edits` is a thin
shell over `facet_write`, which already existed. The only genuinely new
leaf-level thing in the whole slice is a `KeyPosture` parameter on a writer that
was already shared — and that parameter is what keeps `doctrine risk set`
behaviour-preserving by construction rather than by test.

A slice adding a fourth caller and contributing one parameter to a shared leaf is
not a slice compounding duplication.

## Consequence for the plan

No refactor phase. The phase set is exactly [[DEC-165]]'s split, so nothing
competes with the data-loss fix for phase 1 — which is the outcome the user's
standing tie-breaker asked for, reached here on measurement rather than on the
tie-breaker.
