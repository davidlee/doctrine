# ISS-453: knowledge edit list facets split prose at every comma

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`doctrine knowledge edit <kind>`'s list-shaped facet flags are declared

```rust
#[arg(long, num_args = 0.., value_delimiter = ',')]
```

(`src/knowledge.rs:2952`, `:2958`, `:2998`). So a **single** prose element
containing an internal comma is silently split into fragments at every comma,
and the fragments after the first retain their leading space. The record is
written, the verb reports success, and nothing discloses that the input the
caller sent is not the input that landed.

These facets are prose fields — `alternatives` and `consequences` on a
decision, and the same shape on other kinds. A sentence with a subordinate
clause is the *normal* case, not an edge case, so the delimiter and the field's
content type are simply mismatched.

## Witnesses

Both found while reconciling `SL-259` (`RV-366` `F-9`, and its escalation
trigger — *"if reconcile finds the same shape in a second record while editing,
that is the point to escalate it"*):

| record | field | elements written | elements intended |
|---|---|---|---|
| `DEC-250` | `consequences` | 4 | 3 |
| `DEC-243` | `consequences` | 8 | 4 |

Both were mangled by the same commit, `a3b565571` (*"amend six sections and two
rulings against RV-365"*), which amended the rulings through the CLI. Neither
record carried the shape at `a3b565571^`. Repaired by hand at `SL-259`'s
reconcile — the CLI cannot repair them, because re-sending the joined sentence
through `--consequences` re-splits it.

**The path matters.** Records whose facets arrive through the design run's JSON
payload are unaffected: `DEC-251`'s `alternatives` carry several internal commas
and are intact. This is CLI-path-only.

## Why it is worth a fix rather than a caution

It is the **silent-input-absorption class `SL-259` exists to close**, sitting in
the path that records `SL-259`'s own rulings. `RV-366` `F-9` called the effect
cosmetic and was right about `DEC-250`; the class is not, because the damage is
to durable governance prose, it is undisclosed, and the caller has no way to see
it short of reading the TOML back.

## Shape of the fix (needs a design call)

1. **Drop `value_delimiter` and take a repeatable flag** — `--consequences X
   --consequences Y`. One element per occurrence, no in-band delimiter, prose
   safe by construction. Breaks any caller relying on the comma form.
2. **Refuse an element containing a comma**, naming the repeatable form. Typed
   refusal over silent absorption — `DEC-245`'s own rule — but noisier for a
   caller with a legitimately comma-free list.

(1) is the better shape on this codebase's own terms; (2) is the cheaper
migration. Either way the current behaviour — split and report success — is the
one thing that should not survive.

## Related

`RV-366` `F-9` (the finding this escalates from), `DEC-245` (refuse input the
engine will not act on), `STD-003` (a degraded read is disclosed).
