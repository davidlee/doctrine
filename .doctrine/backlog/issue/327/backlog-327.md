# ISS-327: Declaration keys inert at the subject's state are silently ignored

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`ISS-318`'s defect class, rotated one axis. That item's declaration axis is *a
key honoured at one subject kind and inert at the rest*; this is *a key honoured
at one subject kind and inert at that kind in one of its two states*. Same
symptom — accepted, revision bumped, receipt written, nothing changed, exit 0 —
and the same detection path, which is reading state back and noticing.

Found 2026-08-08 while discharging `SL-249` `PHASE-02/EN-2`. `DEC-183` is the
ruling that kept it out of `PHASE-02`: `I10`'s generated matrix is quantified
over the subject-kind axis only, so it will pass with these four cells intact.

## The four cells

Each read is the only one in the tree; each arm is entered by subject state.

| key | honoured when | dropped when |
|---|---|---|
| `provenance` | `declare_node` creates (`src/design_run/run.rs:1158`) | the node already exists — `rebuild` carries `existing.provenance()` forward |
| `lifecycle` | `declare_node` updates (`run.rs:1249`) | the node is new — the create branch returns before reading it |
| `concerns` | `declare_finding` raises (`run.rs:1419`) | the finding already exists |
| `blocking` | `declare_finding` raises (`run.rs:1429`) | the finding already exists |

The last two are the sharper pair. `declare_finding`'s dispose branch writes
`resolution` and `summary` into the held finding and nothing else, so a caller
correcting a finding's `blocking` flag — the flag the lock gate reads — is told
it succeeded and changes nothing.

## Why it is not simply `SL-249` `PHASE-02` widened

`PHASE-02`'s table is key → honouring kind, and its refusal fires at admission on
the subject's prefix alone. Covering this axis makes the table key →
(kind, applicable state) and requires admission to know whether the subject is
already held — available, since `declare` runs against the next snapshot, but a
different check with a different shape.

It is also not obviously a refusal in every cell. `provenance` on an existing node
may be better read as *persist the prior value*, which is what `Sparse::Omitted`
already means for the three sparse-typed keys — in which case the fix is to say so
rather than to refuse. That judgement is the work here; `DEC-183` records that it
was not made, not that it was made one way.

## Related

- `ISS-318` — the class, and the kind axis `SL-249` `PHASE-02` closes.
- `ISS-328` — the same class one level down, inside `CreateRecord`.
- `DEC-183` — the ruling that scoped this out of `PHASE-02`.
