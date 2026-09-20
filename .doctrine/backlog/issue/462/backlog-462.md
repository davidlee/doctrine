# ISS-462: ReviewPolicy labels exceed 16 B bound

Two of the four `ReviewPolicy` variants the design-run contract advertises
cannot be declared: `design apply` refuses the payload before it lands.

```
$ doctrine design apply SL-260 --input -   # review_policy.policy = adversarial-then-human
Error: payload label term is 22 bytes, over the 16-byte admission bound: `adversarial-then-human`
```

## Mechanism

`src/design_run/run.rs:449` admits the policy label through
`PayloadTerm::label`, whose `ValueKind::Label` admission bound is
`bounds.rs` `DESIGN_STAGE_LABEL_BYTES = 16`. That constant's derivation is the
**stage** vocabulary — longest member `exploring`, 9 B — and it is const-asserted
for the two vocabularies that were considered:

- `src/design_run/mod.rs:176` — `widest_stage(&Stage::ALL) <= DESIGN_STAGE_LABEL_BYTES`
- `src/design_run/inquiry.rs:188` — `widest_form(&DispositionForm::ALL) <= …`

`ReviewPolicy` rides the same slot and has no such proof. Measured:

| variant | bytes | declarable |
|---|---|---|
| `human-only` | 10 | default — emits no row, so never admitted |
| `adversarial-only` | 16 | yes, exactly at the bound |
| `human-then-adversarial` | 22 | **no** |
| `adversarial-then-human` | 22 | **no** |

The failure is silent-by-omission in the sense STD-003 cares about: the contract
prompt lists four policies as a closed enum, so an agent reasonably selects one
and is refused at apply with a bound error that names bytes, not the policy
vocabulary.

## Shape of the fix

Whatever the resolution, it should end with a const-assert that binds
`ReviewPolicy::ALL` to the bound the same way the other two vocabularies are
bound — the defect is the missing proof, not the number. Options:

1. **Raise the bound** to the next power of two (32) and re-derive its provenance
   doc-comment from the widest member across *all three* vocabularies that use
   the label slot. Touches `render`'s row arithmetic (`render/mod.rs:203,205`).
2. **Give the policy label its own bound** (`DESIGN_POLICY_LABEL_BYTES`) derived
   from `ReviewPolicy::ALL`, leaving the stage bound at 16.

Option 2 keeps each bound honestly derived from one closed vocabulary, which is
what `bounds.rs`'s provenance rule (EX-16(a)) asks for.

## Provenance

Hit live on SL-260 — the user selected `adversarial-then-human` as the design
review policy and the run could not record it. Captured as a friction
observation at the same moment.
