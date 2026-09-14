# ISS-451: Undeclared payload key sorts to the end instead of refusing

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`ChangeEvent::ordered` (`change_log.rs:422`) sorts a row's payload terms by
their position in `payload_terms()`, and a key the declaration does not name
falls to `unwrap_or(usize::MAX)` — sorted to the end and stored, rather than
refused.

This is the key-axis sibling of `ISS-290` (a term's declared `ValueKind` is
never checked against the constructed one), at the same seam: `ordered` is the
one place that holds both the event and its terms, which is why `SL-259`
`PHASE-05` puts the `ValueKind` check there. The key axis was not in that
slice's originating set and was left out deliberately rather than widening the
phase.

Found while re-verifying `SL-259`'s design premises against the tree before
planning.
