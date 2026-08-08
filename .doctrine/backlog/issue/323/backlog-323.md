# ISS-323: SL-248 design: sec-6 EXPORTED constant omits clock::today

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`SL-248` `design.md` `sec-6` § *What the root package must export* sketches the
root library's export-set constant as five entries:

```
const EXPORTED: &[&str] = &[
    "interpretation", "DOCTRINE_TOML", "read_doctrine_toml_text",
    "read_path_at", "CaptureError",
];
```

`today` is absent from it — while the `src/lib.rs` sketch immediately above
declares `pub use clock::today;`.

Since the export-set test asserts that the public items reachable from
`doctrine`'s crate root are **exactly** this set, the constant as written fails
against the `lib.rs` it sits beside.

## Why the constant is the outlier, not the sketch

Three independent statements in the design agree that `today` is exported:

1. `sec-6`'s own prose — *"`clock::today` is the third export, and taking it is
   what avoids a fourth dependency edge"*, with the argument that `verify` takes
   `today: String` so the `time` edge is never opened (`RV-346` `F-29`).
2. The `src/lib.rs` sketch's `pub use clock::today;`.
3. `sec-8`'s touch-set row for `src/lib.rs` — *"five items behind three private
   modules"*, which is only true counting `today` (`config_file` ×2, `git` ×2,
   `clock` ×1), with `interpretation` exported as a whole module beside them.

So the resolution is unambiguous: `EXPORTED` carries **six** entries.

## Disposition

Transcription-grade, and deliberately **not** repaired in `design.md`.
`design.md` is watermarked and a hand-edit costs a recovery cycle (`ISS-320`),
which is not worth spending on a code-sketch slip whose prose already resolves
it.

- **Implementation** follows the prose: `PHASE-01` authors `EXPORTED` with six
  entries and writes the export-set assertion against that. Recorded in
  `SL-248` `plan.md` § *Two corrections owed at execution*.
- **Reconcile** owes the correction to `design.md` alongside the four already
  collected in `sec-9` § *Corrections owed to the reconciliation brief* — this
  item exists so it reaches that list, since `sec-9`'s enumeration cannot be
  edited to include it without the recovery cycle above.

Closes when `PHASE-01` lands the six-entry constant and reconcile records the
design correction.
