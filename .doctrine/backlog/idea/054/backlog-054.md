# IDE-054: Audit CLI verbs for format-content axis coupling; rule if a pattern emerges

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Where this came from

`SL-246`'s design review (`RV-370` `F-9`) found `doctrine design show` carrying
two flags on two different axes with no stated precedence: `--format`, which
selects *which rendering of the turn envelope* to emit, and `--json`, which the
design gave the *design-document read* in JSON. One names a content selection,
the other an encoding, and the shared word "format" hides the difference.

`inspect` does not have the tension, because there `--json` and `--format json`
are the same axis and collapse at one line (`src/commands/inspect.rs:59`). That
is why `inspect`'s precedent could not be borrowed.

`SL-246` resolved its own instance the cheap way — `--format` gains `document`
and takes the default, `--json` keeps the document read and is refused
alongside an explicit `--format`. That is a local fix, chosen because the
principled split moves an existing surface.

## What to do

Sweep the CLI for verbs where a `--format`-style flag is carrying content
selection, encoding, or both:

- Which verbs offer `--format` *and* `--json`, and do they collapse (one axis)
  or compose (two)?
- Which `--format` value sets mix a *what* with a *how* — e.g. an enum holding
  both `status` (a content projection) and `json` (a serialisation)?
- Where a verb renders more than one thing, is the choice of thing expressed in
  `--format`, in a separate flag, or in a subcommand?

## The decision that may be in there

If the mixing is widespread, the rule worth minting is probably: **content
selection and encoding are separate axes and must not share a flag** — one flag
names *what* is rendered, another names *how* it is serialised. That would be a
`STD` (an authoring convention with a conformance check available) rather than
an `ADR`, since it constrains surface shape rather than architecture.

If it turns out to be one or two verbs, the honest outcome is to fix those and
mint nothing — a rule with two instances costs more to carry than it saves.

Either way the sweep is what makes the call, so it comes first.

## Not in scope

`SL-246`'s own resolution. It is settled and shipped with that slice; this item
asks whether it should have been the general rule, not whether it was right
locally.
