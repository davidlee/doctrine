## The shape

Doctrine has two document-level write cores that both take a held
`&mut toml_edit::DocumentMut` and return `anyhow::Result<bool>` ("did I change
the document"):

- `facet_write::set_facet_mixed(doc, table, fields, posture)` — the `[facet]` keys;
- `dep_seq::apply_status(doc, managed, hint)` — the top-level `status`/`updated`.

`facet_write::edit_in_place(path, |doc| …)` is the shared read → parse → mutate →
write-once-if-changed envelope. Composing both cores inside one `edit_in_place`
closure is how a verb writes two concerns in ONE write (SL-249 `apply_settlement`).

## The footgun

The natural spelling of "did either change it" is wrong:

```rust
Ok(set_facet_mixed(doc, …)? || apply_status(doc, …)?)   // WRONG
```

`||` short-circuits. When the facet leg returns `true` the status leg is
**never called**, so the status never moves — and the command exits 0. You get
a green verb that half-writes, which for a settle verb is the exact defect it
existed to abolish.

Bind both first:

```rust
let facet_changed = set_facet_mixed(doc, …)?;
let status_changed = apply_status(doc, …)?;
Ok(facet_changed || status_changed)
```

Same trap for any `changed_a || changed_b` over effectful calls. It survives
review because the obvious positive test (settle a healthy record, read it
back) passes: the facet leg reports `true` and the status assertion is the one
you did not write.

## The test that actually catches it

Not the positive round-trip. Two negatives, one per leg, each asserting the
file is BYTE-IDENTICAL afterwards:

- delete the top-level `status` key → the status leg takes `apply_status`'s F-1
  bail. A facet-first two-write implementation has already landed the facet.
- delete the target `[facet]` key → the facet leg takes `RequirePresent`'s F-1
  refusal. A status-first two-write implementation has already moved the status.

Together they admit no sequential implementation in either order, which is what
"one write" actually means. Both were run against naive sequential versions
during SL-249 PHASE-05 and each failed in its predicted direction.
