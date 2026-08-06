# DEC-177: Read stays tolerant; inert facet keys are caught by doctor

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The same defect, one surface over

`ISS-318` is the inert-key defect class: a key that carries meaning nowhere at
its subject's kind is accepted and discarded in silence. Objective 3 closes the
instance on the design-run wire. The knowledge read path is a second instance of
the same shape, and `SL-249`'s `OQ-6` asked whether the fix reaches it.

It reads that way in the code deliberately. `RawFacet` is the kind-blind
superset — every field of every kind, each `#[serde(default)]` — and
`validate_facet` dispatches on `record_kind` and takes only the fields that kind
owns. The doc comment names the intent: *"the kind-aware half of 'kind-blind
read, kind-aware validate'"*. So a `DEC` whose `[facet]` carries a non-empty
`answer` parses without complaint, and the text is gone.

## Why the refusal does not follow

The write side already refuses, and `DEC-170` gives the reason: facet fields are
scaffold-seeded, so an absent key is damage and gets reported rather than
absorbed. The symmetry argument says an *inert* key is damage on the same terms.

What breaks the symmetry is not the diagnosis but the blast radius. `read_kind`
and `read_all` propagate with `?`. One hand-mangled record therefore takes down
`knowledge list` across the whole corpus, not just its own `show` — and
`adoptable` sits on the design-run adoption path, so the refusal could stop a
run adopting any record at all. A write-time refusal fires at the record it
names, with an operator in the loop who is holding the thing that caused it. A
read-time refusal fires at every later reader of an unrelated record, and takes
out the tool you would have used to find the damage.

## What detection costs

Nothing new. `catalog::scan::check_facet_residue` is already a parse-free
key-presence tripwire built for `SL-222`'s deleted `[estimate]`/`[value]`
facets, emitting a `CatalogDiagnostic` with a remedy string;
`doctor_checks::is_facet_diagnostic` already gates `field: Some("facet")`
through to `doctrine doctor`. The inert-key check is the same shape with a
per-kind key table in place of two literals.

The population it has to catch is small and human: templates are per-kind, and
objective 1's per-kind subverbs cannot spell an inert key. A hand-edit or an
out-of-band writer is the only way one reaches disk, which is exactly the
population a corpus health scan exists for.

## The condition this ruling carries

The check needs the per-kind facet field sets as **data**. `DEC-169` established
that Rust gives no reflection over struct fields, so that table is authored
either way, and objective 1's subverbs need the same one — the cost is shared,
not additive. If objective 1's design ends up not producing such a table, the
cheapness claim here fails, and the honest fallback is to rule read-tolerance
deliberate and put the missing canary on the backlog rather than pay for a table
twice.

## Reach

Under [[DEC-172]] every facet key is inert at `CPT`, so a populated `[facet]` on
a concept record is the tripwire's widest case. `validate_facet` itself is
untouched, so `SPEC-019`'s description of the kind-blind read stays true as
written and objective 4's REV owes it no amendment row.
