# Notes SL-259: Truthful apply

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- explore.triage, 2026-09-14. The exploring stage's map, distilled. -->

### Shaping decisions (all settled on the run's inquiry map)

| ruling | what it fixes |
|---|---|
| `DEC-243` | Settles `QUE-219`. Strictness refuses the **never-known**; a retired-member roster carries the **once-known**, over an `STD-003` disclosed-degradation floor. |
| `DEC-244` | Write-path strictness rides `payload_contract`'s pinned key inventory, not `deny_unknown_fields`. One change for `ISS-333`, `ISS-328`, `ISS-327`'s key axis. |
| `DEC-245` | Leg 3 refuses input the engine **will not act on**, never input that changed nothing. |
| `DEC-246` | All four of `ISS-327`'s state-inert keys refuse rather than silently persist. Discharges `DEC-183`'s deferral. |
| `DEC-247` | A term's `ValueKind` is checked at construction; `Outcome`'s **declaration** is the error, not its call site. |
| `DEC-248` | Material changes are emitted at the seam that performs them (`DEC-238`), never derived from a set difference. Covers `ISS-367` + `ISS-450`. |
| `DEC-249` | Degrade what is **history**; refuse what is **state**. |
| `DEC-250` | Leg 1 hoists every hoistable check ahead of the mints; the residual is `SPEC-029`'s specified late window. |
| `DEC-251` | An unreadable change row is preserved opaquely **at the row**, not by an opaque `ChangeEvent` variant. |

Evidence under them: `EVD-027` (wire/stored type overlap, latent),
`EVD-028` (malformed payload moves nothing — `ISS-361`'s mechanism ruled out),
`EVD-029` (records materialise before two checks that can still refuse).

### Constraining governance

- `SPEC-029` governs, and rules on leg 1 directly: reserve-then-journal and the
  late watermark re-check are **specified**, and *no failure path may delete
  authored knowledge to repair a runtime error*. Unwinding is prohibited, not
  merely unattractive. Caught at `explore.scope`, after `DEC-250` was already
  recorded — see its correction section.
- `ADR-001` bounds `DEC-244`: `payload_contract` is a **leaf**, so the
  contract-driven key walk sits at or below the command boundary that parses.
  It must not invert the layering.
- `STD-001` is why both `DEC-244` and `DEC-247` put their check where the
  single source already is, rather than beside it.
- `STD-003` is what leg 4 discharges.
- `DEC-239` still refuses the two cheap `ChangeEvent` repairs (bare serde alias;
  whole-row normalisation at deserialise). `DEC-251` takes neither.

### Risks and assumptions carried into drafting

- **R1 (measured, dates fast).** All 16 live snapshots probed: 15 parse,
  `SL-244` alone fails, none carries a stored proposal. The change log is a
  **32-revision window**, so `SL-244`'s offending row at revision 88 (floor 61)
  is four revisions from ageing out — `ISS-315` self-heals on an active run and
  bites a dormant one. Re-probe before implementing; the measurement is a
  snapshot of a moving tier.
- **R2 — resolved** by `DEC-244`: the fix moves the check off serde, so
  `flatten`'s incompatibility stops being load-bearing.
- **R3 — new.** `ChangeEvent`'s two `const _: ()` proofs are fragile in an
  unobvious way: the `is_subset(&EMITTABLE, &READABLE)` assert is (per its own
  comment) the only reader of `EMITTABLE` in `src/`, so deleting it passes
  `cargo check` and fails `cargo test --bin doctrine`. Do not "clean up" either
  proof. See `mem.pattern.rust.expect-dead-code-is-per-compilation-unit`.
- **A1 — `ISS-361` stays open**, against `DEC-250`'s residual window. `EVD-028`
  ruled out the reported mechanism; nothing ties the single witness to the
  surviving suspect.

### Verification anchors (from memory, for the plan)

Pin snapshot compat at `snapshot::parse` over a **literal legacy fragment**,
never a unit round-trip over the inner type. Precedents to copy:
`a_snapshot_written_before_the_policy_reads_as_human_only` and
`a_snapshot_written_before_the_intent_subject_key_still_parses`
(`src/design_run/snapshot.rs`). `design_run` tests may not name `crate::`
(`mem.pattern.design-run.leaf-rule-binds-tests`), which binds where `DEC-244`'s
walk is tested.

### Out

`ISS-362` struck — premise disproved by repro, recorded on the issue.
Splitting `Declaration` into separate wire and stored types: deferred debt under
`DEC-243`. Whether stage should gate acts it does not gate today: the residual
of the struck `ISS-362`.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
