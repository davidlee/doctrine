# ISS-333: `ApplyRequest` cannot deny unknown fields, so a top-level key is discarded

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Split out of `ISS-318` observation 3 at `SL-249`'s close, so that the envelope
axis keeps an owner once `ISS-318` resolves on its declaration axis.

## What

`ApplyRequest` carries `#[serde(flatten)] envelope: SubmissionEnvelope`, and
serde cannot reconcile `flatten` with `deny_unknown_fields` — the flattened
field's own keys would themselves be refused as unknown. The type says so in
its own doc comment. So the **outermost** submission type, the only one a
caller hand-authors from scratch, structurally cannot refuse a key it does not
know: the key is discarded before any of the run's own checks see it, the
revision bumps, a receipt is written, no change row prints, and the command
exits 0.

Witnessed on `SL-248`'s design run, 2026-08-06, while probing for the section
payload shape. Three successive probes were all accepted and all established
nothing:

```json
{"…envelope…", "sections": []}                  → revision 47, no change rows
{"…envelope…", "sections": [{"foo": "bar"}]}    → revision 48, no change rows
{"…envelope…", "zzz_nonsense": [{"a": 1}]}      → revision 49, no change rows
```

The answer — sections are declared through `declare` with a `body` field — was
findable only by reading `src/design_run/submission.rs`.

## Why it is a separate item from `ISS-318`

`SL-249` closed `ISS-318`'s declaration axis: `Declaration::WIRE_KEYS` crossed
with `IdKind::declarable` is now total, and a key inert at its subject's kind
is refused by name at `Batch::validate`. The slice's Non-Goals declared this
axis out of scope by name — *"same class, different mechanism, separate
change"*. It is different in mechanism: `ISS-318`'s fix was a hand-written
correspondence table over a known key set, and this one is a serde structural
limit with no table to write.

## Shape

`deny_unknown_fields` is unavailable, so the fix is not a derive. Candidates,
none costed:

1. Hand-write `Deserialize` for `ApplyRequest`, collecting residual keys and
   refusing them after the flatten is resolved.
2. Deserialize to `serde_json::Value` first, subtract the union of the known
   key sets, refuse the remainder.
3. Accept the limit and close the discoverability half instead — `design apply`
   documents only `--input`, so a caller cannot learn the payload shape without
   reading the source. A schema dump would defeat the probing loop that made
   this cost three revisions, without touching serde.

Option 3 is the cheap one and addresses the observed cost rather than the
observed mechanism; it is not obviously the right answer.

## Constraint

`ISS-328` records why the sibling was not a drive-by fix, and the same
constraint binds here: stored proposal declarations ride the run snapshot and
outlive the binary that wrote them
(`mem.fact.design-run.snapshot-outlives-the-binary`). Tightening a read path
converts previously-readable stored state into a parse failure at exactly the
moment someone is resuming.

## Related

- `ISS-318` — the parent class; its declaration axis, closed by `SL-249`.
- `ISS-328` — the same envelope defect at the nested types, where the serde
  excuse does **not** apply.
- `ISS-327` — the same class on the subject-state axis.
- `DEC-183` — the ruling that scoped the residue out of `SL-249` `PHASE-02`.
