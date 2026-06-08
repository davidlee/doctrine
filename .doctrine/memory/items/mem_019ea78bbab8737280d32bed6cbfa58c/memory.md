# Conformance tests must assert every surface, not just the JSON envelope

When a slice exists to establish a *uniform contract* across N kinds (e.g.
SL-025's shared list/show spine), the conformance test must assert the contract
on **every output surface** each kind emits — the human table (header row,
prefixed ids, empty-suppression) AND the JSON envelope — not just one of them.

**Why:** SL-025's `tests/e2e_list_conformance.rs` asserted only that
`<kind> list --filter x --json` parses and emits a `{kind, rows}` envelope. It
passed for all five kinds while `backlog list` shipped with **no table header**
— a §5.5 contract violation that survived six phases and the conformance phase
itself, caught only at code review (audit finding F-1). Envelope-parity is not
surface-parity; a green conformance suite that checks one surface gives false
confidence about the others.

**How to apply:** for a uniform-surface slice, make the conformance test
table-driven over the kinds AND over the surfaces — for each kind assert the
table header/shape and the json envelope. If a contract clause names a surface
("header row", "prefixed ids", "empty → \"\""), there must be a per-kind
assertion that exercises *that* surface, not a proxy.

Related: [[mem.pattern.doctrine.tdd-loop]], [[mem.concept.doctrine.entity-engine]].
