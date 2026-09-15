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
fresh-as-of: 2026-09-15 · audit (RV-366) resolved · 503c5a2a3

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
PHASE-04 (leg 3, state axis): `6b7bc165f` closes `ISS-327` — `KeyWhen` column on
`Declaration::WIRE_KEYS`, `Declaration::inert_at_state`, `Refusal::InertAtState`,
`run::subject_state`, and `Batch::validate` taking the state as an injected
`state_of`. Plan amended at `f3558f3dd` (four cells → three).
PHASE-05 (leg 3, value axis): `0a793ee63`..`e4a3e9b1d` close `ISS-290` —
`ChangeEvent::ordered` absorbed into `ChangeEvent::shaped` (the declaration
governs a row's term order AND kinds, through one call), `Refusal::UndeclaredTerm`,
fallible `Pending::{about,run_wide}`, and `StepDischarged`'s `Outcome` declared
`Label`. `ValueKind` gained `as_str` + `ALL`; `gate::join` widened to `pub(super)`
rather than respelt.
Dispose `ISS-450`, `ISS-367`, `ISS-315`, `ISS-333`, `ISS-328`, `ISS-327`, `ISS-290`
at slice close.
Gate green at `e4a3e9b1d`: `doctrine check gate` exit 0, `verify-vt` PHASE-01..05
all PASS (PHASE-06's `VT-1` is the only FAIL, and is unwritten). `.doctrine`
changes committed with their code.
`mem.pattern.change-log.derived-difference-cannot-see-replacement`.
`mem.fact.design-run.change-log-degrades-state-refuses`,
`mem.pattern.testing.readable-arm-accessor-narrows-every-sweep`.
`mem.pattern.doctrine.earlier-phase-may-close-a-later-phases-cell` (new,
`f0873ee1c`); `mem_019fd0ceae9d7913840328c2ded75ee7` gained a third instance.
`mem.pattern.design-run.guard-declaration-construction-at-the-seam` and
`mem.pattern.lint.closed-vocabulary-tokens-in-diagnostics` (both new, PHASE-05).

`RV-366` (implementation audit, resolved — 9 findings, 0 blockers, all verified
terminal; synthesis + reconciliation brief in `review-366.md`).
`IMP-449` (slice conformance reports a slice's own lifecycle artefacts as
undeclared — systemic, cross-checked on `SL-256`) and `ISS-452` (opening a review
pass emits no change row), both from `RV-366` and linked `originates_from SL-259`.
Audit evidence: `doctrine check gate` exit 0 at `503c5a2a3`; `slice verify-vt 259`
PASS on all 18 `VT`s across six phases; `PHASE-01` `VA-1`, `PHASE-02` `VA-1`/`VA-2`
and `PHASE-06` `VA-1` all discharged in `RV-366`'s synthesis.

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

PHASE-04 `F-1` — `PHASE-01` closed one of `DEC-246`'s four state-inert cells
three phases early: collapsing `declare_node` onto one row-producing path made
`lifecycle` honoured at creation, which was the whole of that cell. Leg 2
repaired a leg 3 defect. `EN-2`'s named anchors were all live; the **count** was
the only false claim, and it was the one thing the criterion did not say to
check. Amended, not restored — re-refusing `lifecycle` would regress `PHASE-01`.

PHASE-04 `D-6` — the state column joined `WIRE_KEYS` rather than sitting in a
separate three-row table. The separate table is closer to `EX-3`'s wording and
far more compact; it was rejected because a wire key added without a state answer
would silently default to state-insensitive, which is this slice's own defect.

PHASE-04 — the kind-axis matrix (`I10`) and the state-axis matrix guard each
other, which is `EX-2`'s claim showing up as a property: widening the refusal
into a ban fails the state control **and** `I10`, because a key both refused and
effectful is `I10`'s own documented hazard.

PHASE-04 `F-4` — a probe harness reverting with `git checkout -- <path>` took a
phase's uncommitted work on the invocation whose grep matched nothing. The
corpus already held the rule (`mem_019fd0ceae…`); retrieval had been scoped to
the subject matter, not to the technique.

PHASE-05 `F-2` — `EN-4` asked whether narrowing `Outcome` could degrade a stored
row and answered from the value lengths. The real answer is structural: a stored
term re-enters through `PayloadTermWire` carrying its **own** kind, so a
declaration change cannot reach history at all — and the corpus had already
settled it, **65 of 65** live stored `outcome` terms reading `kind = "label"`.
The declaration moved onto what the disk said. `mem_019fcd1727fa7061b771179b113f5726`
(a `DesignSnapshot`-reachable wire form outlives its binary) is discharged by the
census, not argued past.

PHASE-05 `F-4` — the probe that mattered was the one that found a hole in the
**test**, not in the code. Emptying one event's declared shape left the `VT-1`
matrix green: a sweep-wide `cells > 0` total cannot see one input dropping out of
the sweep. That is `mem_019fe0c6db677dd1aa6a8ef8e91f3828`'s point 2 landing on the
matrix that cited it, and the reason every `VT` is probed rather than read.

PHASE-05 `F-3` — `clippy::use_debug` is denied repo-wide, which overturned a
phase-plan decision written to avoid a second spelling of a serde token. The
house rule (`gate.rs:1214`) is the opposite: every closed vocabulary carries
`as_str` beside `rename_all`, and the two spellings get a round-trip pin rather
than hand-written serde impls. `ChangeEvent` is the exception, not the pattern.
Recorded as `mem.pattern.lint.closed-vocabulary-tokens-in-diagnostics`.

PHASE-05 `D-1` — the seam is `shaped` (absorbing `ordered`), not the drain at
`run.rs:495` and not `PayloadTerm::admit`. Fusing the kind check into the
ordering call, rather than adding an `admits()` beside it, is what stops a future
third `Pending` constructor from reaching the log through the sorter alone. Three
instances of this declaration/construction class now exist across `SL-233`,
`SL-249` and this slice — `mem.pattern.design-run.guard-declaration-construction-at-the-seam`.

PHASE-06 `F-1` — leg 1's hoist had a witness after all, on an axis the design
review did not examine. `RV-365` `F-2` cleared the pass-1/pass-2 differential and
`sec-6` generalised that into "*hoisting on principle, with no witness behind
it*". But `execute_mint` runs **once per plan to completion** and its step 1
carried a refusal (SL-249 `D8`'s retry guard), so a two-checkpoint payload whose
second plan trips the guard materialises the **first** plan's record and then
refuses. Reproduced before it was claimed. Not a `SPEC-029` breach — the spec
says in terms that the guarantee is *the run does not advance*, not that nothing
was written — so `DEC-250` is the only thing it repairs. Now
`refuse_unresumable_mints`, over the whole batch, ahead of the loop. Recorded as
`mem.pattern.atomicity.hoist-a-loops-refusal-out-of-the-loop`.

PHASE-06 `F-2` — a multi-checkpoint payload was an unknown, not a given. No
fixture in the tree declared two `cp-` subjects in one `declare` array; the
phase-plan carried it as `A1` with a STOP condition. It is admissible and both
plans mint in declare order, which is what makes `F-1` reachable rather than
theoretical. Single-item fixtures are how a per-item guard hides a cross-item
hole.

PHASE-06 `F-3` — pass 2's resolution-dependence was exactly two things, and both
were true by observation rather than by construction. The key set
`CheckpointRecordUnresolved` fires on was built by two separate walks that merely
agreed (now one expression, `resolution_of`). The `DESIGN_ID_BYTES` bound on a
resolved record was argued away in `sec-6` by counting bytes in prose (now
`widest_canonical_id`, a const proof over the whole `KINDS` table, in
`gate.rs`'s `widest_condition` idiom). Probe `P6` — tighten the bound to 14 —
fails the build, so the proof is live and the widest mintable id is exactly 15 B
against 32.

PHASE-06 `F-5` — the sheet's own risk `R2` was refuted by its probe. It assumed
`sec-9` `R3`'s `EMITTABLE` fragility generalised to any const proof whose helper
has no other reader. It does not: `design_run/mod.rs:68-74` carries a module-wide
`cfg_attr(not(test), expect(dead_code))` and `commands/` does not, so deleting
the new proof fails `cargo check` outright. The claim "nothing will catch this"
is about the module's lint posture, never about const proofs. Corrected in the
code's own comment and recorded as
`mem.fact.lint.dead-code-exemption-decides-whether-a-const-proof-defends-itself`.


Harvested from the phase sheets at audit — durable, and not previously lifted:

- **A duplicated guard is invisible to the suite by construction** (`PHASE-06`
  probe `P2`). Hoisting `refuse_unresumable_mints` out of `execute_mint` step 1
  was pinned by probe `P1` (removing the hoisted guard reds both `T3` and
  `SL-249`'s own retry test, so no second copy was left behind), but the inverse
   — *restoring* the bail inside the loop as well — breaks nothing and no test
  can see it. Recorded as a standing blind spot: a silent parallel implementation
  is caught by review or not at all.
- **`review_pass_plan` can never trip the retry guard** (`PHASE-06` `P3`). It
  carries `payload_digest: None` (`design.rs:924-942`) and `resumable_under` is
  `is_none_or`. Including it in the hoisted pre-pass is consistency, not
  coverage — worth knowing before someone "simplifies" it out.
- **`read_rows()` is `#[cfg(test)]` on purpose** (`PHASE-02` `F7`). The
  readable-arm accessor drops opaque rows by construction, which would make a
  sweep silently vacuous rather than red; closed in place with
  `assert_eq!(log.read_rows().count(), log.rows.len())`. Production deliberately
  has no such accessor — `envelope.rs` must see both arms, and does. Verified at
  audit: both callers are in test modules.

### Open
`IMP-448` open — `entity.rs:557` re-spells `kinds::canonical_id`'s
`{prefix}-{id:03}` by hand, and that hand-built string, not the documented format
authority, is what every minted id actually is (`on_reserved` has one call site).
`STD-001`, pre-existing, found by PHASE-06's const proof — which can therefore
prove the *form* but not that there is one formatter. The limit is stated in the
proof's doc comment and should be deleted with `IMP-448`.
`design.md` `sec-6` states the hoist has no witness (PHASE-06 `F-1`). True of the
pass-1/pass-2 axis it was written about, false of the mint loop. Needs qualifying
by axis at reconcile, not striking.
`src/commands/design.rs` gains `refuse_unresumable_mints`, `resolution_of`,
`widest_canonical_id` and a rewritten module doc — `sec-7`'s code-impact row for
this file says only "`apply`'s check ordering", which understates it. Same class
as the `ids.rs` and `gate.rs` deltas above.


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
`submission.rs` conformant as of `6b7bc165f` — supersedes the `undelivered`
line carried here through PHASE-03.
`src/design_run/ids.rs` reads `undeclared` in conformance: `sec-7`'s code-impact
table did not anticipate `SubjectState` landing beside `IdKind`. Authored-truth
delta for reconcile.
`src/design_run/gate.rs` joins it (PHASE-05): `join` was widened to `pub(super)`
so the new refusal renders its list through the one comma-separated join rather
than a twelfth spelling. Same class — a one-line change to a file `sec-7` does
not list.
`design.md` `sec-3` still states `DEC-246`'s four cells where three exist
(PHASE-04 `F-1`). Reconcile against the locked design; not editable mid-phase.
Research baseline for `SL-259` reports drift against its own downstream
artefacts only; judged not to invalidate a thread, not restamped (PHASE-02 `A5`).


*Audit (`RV-366`) — what it added to this list, and what it closed.*

`RV-366` `F-1`/`F-2`/`F-3`/`F-8` carry the four `design.md` divergences already
listed above (`sec-3`'s cell count, `sec-6`'s "no witness", `sec-7`'s code-impact
table, and — newly found at audit — `sec-5`'s "`ChangeEvent` is not touched") into
the reconciliation brief as per-slice direct edits. `F-4` names the load-bearing
repair for the `ids.rs` / `gate.rs` conformance rows and five more the notes had
not caught: the **selector registry**, not `sec-7`'s prose.

**`DEC-252` is ruled** (`F-6`): a line in this slice's reconciliation, **not** a
`REV` against `SL-251`. `PHASE-NN`/`VT-n` ids are immutable-append, so a `REV`
would resolve to no legal write; no governance artefact changed; the real gap is
discoverability, closed by settling `DEC-252` and relating it to both slices.
The same reasoning disposes `SL-233` `EX-11(a)` (`PHASE-02` `F5`): a closed
slice's plan criterion is off-surface for `/reconcile`, and the inversion is
already recorded here and in `mem.fact.design-run.change-log-degrades-state-refuses`.

**`PHASE-02` `VA-1` is adjudicated: held.** The two forced `snapshot.rs` pin
edits (`[row]` → `[StoredRow::Read(row)]`) *tighten* — they now additionally
assert the row reads — and both `const _: ()` proofs are intact.

**`PHASE-01` `D5` needs no line against leg 1.** A node declared `resolved` with
no disposition now refuses *before* anything lands, which is what "error ⇒
nothing landed" promises; it is `DEC-245` working, not drift.

`ISS-361`, `IMP-446`, `IMP-447`, `IMP-448` remain open with stated reasons and,
where applicable, triggers. No governance/spec `REV` is owed by this slice.
