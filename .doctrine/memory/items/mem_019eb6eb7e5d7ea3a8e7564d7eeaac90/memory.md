# Forward-edge validation only fits refs to numbered kinds in integrity::KINDS — free-text refs (DEC) must carry unvalidated

`integrity::ensure_ref_resolves(root, ref)` is the up-front forward-edge guard a
`new` verb uses to refuse a dangling edge BEFORE minting an id (review's
`--target`, rec's `--owning-slice`). It dispatches on the canonical prefix via the
`KINDS` table, so it ONLY accepts refs whose prefix is a numbered entity kind
(`SL`, `ADR`, `REQ`, `RV`, `REC`, …). A ref to anything NOT in `KINDS` —
notably a `DEC` decision reference (doc-local, e.g. `DEC-005-C`) — hard-errors
with `unknown kind prefix DEC`.

**Why:** SL-042 P1 `rec new --decision DEC-005-C` initially routed `decision_ref`
through `ensure_ref_resolves` and was refused at the smoke test. DEC is not a
doctrine entity kind; a `decision_ref` is a free-text pointer to a decision record
that lives in spec/design prose, not the corpus. Validating it was wrong.

**How to apply:** when adding a `new` verb with optional ref fields, forward-validate
ONLY the fields whose target is a numbered kind in `KINDS` (slices, specs, ADRs,
reviews, recs). Carry doc-local / external refs (`DEC-*`, and any future
non-entity pointer) as unvalidated free-text — splice them through `toml_string`
for safety, but never through `ensure_ref_resolves`. The design naming a pointer
(`decision_ref → DEC`) is a conceptual edge, not an entity-resolution obligation.

See [[mem.pattern.entity.numbered-kind-identity-table]] (the `KINDS` table
`ensure_ref_resolves` dispatches on) and
[[mem.pattern.install.authored-entity-wiring]] (wiring a new authored kind).
