# ISS-440: selector doctor calls a design-target selector redundant against a scope-relevant glob

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine slice selector doctor` tests subsumption on path shape alone. It does
not consult `intent`, so a `design-target` selector nested under a broader
`scope-relevant` glob is reported as redundant when the two serve different
oracles.

On `SL-251` it fires six times:

```
SL-251: 6 selector finding(s)
  redundant     src/design_run/payload_contract.rs  (subsumed by src/design_run/**)
  redundant     src/design_run/mod.rs               (subsumed by src/design_run/**)
  redundant     src/design_run/render/envelope.rs   (subsumed by src/design_run/**)
  redundant     src/design_run/submission.rs        (subsumed by src/design_run/**)
  redundant     src/design_run/attestation.rs       (subsumed by src/design_run/**)
  redundant     src/design_run/tests.rs             (subsumed by src/design_run/**)
```

The subsuming `src/design_run/**` carries intent `scope-relevant`; all six
subsumed selectors carry `design-target`.

## Why the advice is actively harmful

`doctrine slice conformance` computes its undeclared / undelivered / conformant
algebra from the **`design-target` set alone**. Taking the doctor's advice on
this slice would empty the conformance oracle for all of `src/design_run/` — the
subsystem the slice exists to change — so every future touch there would report
conformant by vacuity. A selector-hygiene report that, followed literally,
disables the drift signal is worse than no report.

## Shape of the fix

Scope the subsumption test to selectors sharing an `intent`, or report
cross-intent nesting under a distinct, non-actionable label. A `design-target`
path inside a `scope-relevant` glob is the *normal* authored shape — `/slice`
seeds the broad glob and `/design` pins the targets — so this is the common
case, not an edge case.

## Provenance

Raised as `RV-361` `F-9` during `SL-251`'s reconciliation audit and dispositioned
`follow-up`: a defect in the selector tooling, not in `SL-251`'s selectors.
