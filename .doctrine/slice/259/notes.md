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

## Review passes

*Written after RV-365 concluded, at design run revision 53.*

`RV-365` was the design pass: an external adversarial reviewer (codex-cli,
`gpt-5.6-sol`) against revision 40. Six findings, three of them blockers, all
verified against the tree before disposition, all terminal. Sections 2, 4, 5, 6,
7 and 9 were amended and the design rematerialised at revision 50; a seventh
defect introduced *by* those amendments was caught on the verification pass and
corrected at revision 49.

**No further design pass is needed before planning**, and the reason is specific
rather than a shrug: the pass that just ran was hostile, evidence-led and
adversarially verified against the source, and the three findings it did not
raise are the ones a second pass would go looking for. `sec-1` and `sec-3` were
probed and came back sound — the `payload_contract` single-source claim in
particular was checked against its pins, including the flatten envelope and the
internally-tagged enums, with an effective positive control.

**What a further pass should probe, if one is ever run:**

1. **The amended text itself.** Sections 2, 4, 5, 6, 7 and 9 are newer than the
   review that shaped them. One defect already entered that way (`sec-5`'s
   "whole surface", corrected at revision 49) and it was found only because the
   raiser was asked to re-read changed text. Changed text is where defects
   enter.
2. **Every file:line and count in the design** — `sec-9` `R0`. Three of this
   design's factual claims failed external checking and none was caught by the
   author. This is a re-verification task at implementation, not a review task.
3. **`DEC-250`'s hoist, once its implementation exists.** The ruling is now
   hoisting-on-principle with no witness behind it. The right moment to test
   whether the hoist is sufficient is against real hoisted code, not against
   prose.
4. **`IMP-446`'s trigger**, if delegation gains live traffic before this slice
   lands — that is the one deferral whose justification can expire underneath
   the slice.

A *code* review pass at implementation is a separate question and is expected;
this statement covers the design axis only.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-15 · PHASE-03 completed · d97bf389f

### Produced

`RV-365` (design review, concluded — 6 findings, 3 blockers, all terminal).
`IMP-446` (deferred `Declaration` wire/stored split, trigger-bound).
Design amendments across revisions 41-50; `DEC-243` and `DEC-250` amended.
PHASE-01 (leg 2): `95ca9676c` closes `ISS-450`, `a4d10d216` closes `ISS-367`.
PHASE-02 (leg 4): `419493b60` closes `ISS-315` — `StoredRow` / `RawRow` /
`Unreadable` and the disclosure at `render/change_row.rs`.
PHASE-03 (leg 3, key axis): `a05ff23f1`..`7ca1c082f` close `ISS-333` and
`ISS-328` — `design_run::contract_check`, a leaf walk of the payload against
`payload_contract`'s inventory before deserialisation, plus
`Refusal::UnknownPayloadKey` and all eight contract rows flipped to `Refused`.
`IMP-447` (collapse `UnknownKeys` if its arm stays uninhabited).
Dispose `ISS-450`, `ISS-367`, `ISS-315`, `ISS-333`, `ISS-328` at slice close.
Gate green at `d97bf389f`: `doctrine check gate` exit 0, `verify-vt` PHASE-01..03
all PASS. `.doctrine` changes committed with their code.
`mem.pattern.change-log.derived-difference-cannot-see-replacement`.
`mem.fact.design-run.change-log-degrades-state-refuses`,
`mem.pattern.testing.readable-arm-accessor-narrows-every-sweep`.

### Learned

A design defended by its own author fails factual checking in ways the author
does not catch: three of six findings were plain misstatements of the tree
(arithmetic, a citation, a call order). Recorded as `sec-9` `R0`. PHASE-02
found the same class again in its own plan — `EN-3`'s reader inventory was both
stale in line numbers and short by a whole file (PHASE-02 `F1`).
`DEC-249`'s history/state line is load-bearing in a second place nobody had
noticed — it is what forbids widening the `DEC-251` floor to cover stored
declarations (`RV-365` `F-3`).
`DEC-248`'s *empty prior* under-determines the parent (PHASE-01 `D4`).
The create/update split hid a second silent drop of the same shape (PHASE-01 `D5`).
A tolerance decision inverts the pins written before it: `DEC-249` turns
`SL-233` `EX-11(a)`'s wire half from *refuse the file* into *retain the row*,
with the guarded finding still closed (PHASE-02 `F5`).

The payload contract's own published document changes what it tells every
installed client: `unknown-keys: refused` where it said `silently-dropped`.

`mem.pattern.doctrine.vt-test-file-is-a-guess-retarget-it` (PHASE-03 `D1`) —
the second `VT` retarget in this slice; `PHASE-01` was the first.
`mem_019f89125fb275a2895bf58b5e29ed95` refined — `record-delta`, not the
`completed` flip, is what clears `UNATTRIBUTABLE`; conformance is the half that
does need the flip.
`mem_019fd03e13397240b4eb05af218f5cf5` updated — the flatten-drop defect is
closed, and on this surface a green suite is now positive evidence (PHASE-03
`F3`).

PHASE-03 `D2` — **no unification of the two contract descents** (`T9`, bar
stated in the sheet: unify only if the unified form is no more complex than
the two). Declined on four axes of variation, not one: subset vs equality
judgement, sparse payload vs fully-populated fixture, first-refusal vs
fault-accumulation (pin 2 must *count* how many untagged shapes a value
satisfies), and a coverage-site recorder that exists only for the test's
coverage-equality assertion. A visitor carrying all four would exceed both,
and would couple production to a test-only mechanism. The duplication that
mattered — `sec-2`'s tagging × payload table — was already single-sourced and
is now shared rather than copied: `place` / `Placement` / `Fields` left
`cfg(test)` for their first shipping consumer.

### Open

`ISS-361` stays open against `DEC-250`'s residual late-check window.
`IMP-446` open, trigger-bound on first live non-empty `delegation`.
`IMP-447` open — `UnknownKeys::SilentlyDropped` left with no production
inhabitant by `EX-3`, retained deliberately (PHASE-03 sheet `D5`).
`DEC-252` — the walk supersedes serde's message for the three attributed types,
so `SL-251`'s `VT-3` reads red if re-run. Whether that wants a `REV` against
`SL-251` or a line in this slice's reconciliation is audit's call.
PHASE-01 `D5` changes a refusal surface: a node declared `resolved` with no
disposition now refuses where it used to be seated `open` silently. Worth a
line at audit against leg 1's *error ⇒ nothing landed*.
PHASE-02 `F5` changes a second refusal surface, in the other direction — an
over-bound stored term degrades where it used to fail the file. `SL-233`'s
criterion needs reconciling, not just this slice's.
PHASE-02 `VA-1` held in intent but not in its literal wording (accessor-only
edits to two `ChangeEvent` roster tests were unavoidable); adjudicate at audit.
`submission.rs` reads `undelivered` in conformance, correctly: PHASE-04's target.
Research baseline for `SL-259` reports drift against its own downstream
artefacts only; judged not to invalidate a thread, not restamped (PHASE-02 `A5`).
