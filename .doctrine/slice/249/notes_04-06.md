# SL-249 — notes shard, phases 04–06

Execution record for this shard's phases. `notes.md` stays the orchestrator's.

> Shard name: the orchestrator's worker-1 brief says `notes_03-06.md`; the
> PHASE-04 sheet's T10 says `notes_04-06.md`. Took the sheet's name — phase 03's
> record already lives in `notes_01-03.md`, so `notes_03-06.md` would have
> claimed a phase another shard already covers. Noted so nobody hunts for a
> missing file; the shard index in `notes.md` is the orchestrator's to write.

## PHASE-04 — the write seam and the edit surface

Split across two workers by movement: worker 1 took the **seam** (T0–T5), worker
2 takes the **surface** (T6–T10). The split is a context budget, not a design
boundary — but it does create one visible artefact, the `expect(dead_code)`
staging described under T5.

### T0 — the baseline

`cargo test --bin doctrine commands::facet::` → **33 passed, 0 failed**, with
`git diff --stat src/commands/facet.rs` empty. That pair is EN-2, and it is the
denominator VA-1 is measured against at T9.

### T1 — `KeyPosture` on the mixed writer

Red was a compile failure, as designed: `E0061: this function takes 3 arguments
but 4 arguments were supplied`, naming `KeyPosture::RequirePresent` as the
unexpected argument.

Implemented per D2. `KeyPosture<'a> { Create, RequirePresent { record: &'a str } }`
threads through `set_facet_mixed` and `apply_set_mixed`. The refusal sentence
lives in one private formatter, `missing_key_refusal(record, table, key)`
(STD-001) — the leaf cannot know a record id, so the posture carries it in, the
same shape `dep_seq::apply_status`'s `hint` uses.

Two placement details worth keeping:

- The presence check is **find-all-before-insert-any**, inside the `Some(item)`
  arm and after the shape check. Placing it before or after the existing no-op
  guard is observationally identical — an absent key always reads as `changed`,
  because `current` is `None` — so it sits where it reads as a precondition.
- The `None` arm bails **before** allocating under `RequirePresent`. An absent
  `[facet]` table is damage too, not a create-on-demand path.

One small addition the sheet did not name: `FacetField::key()`, so the presence
check can read a field's managed key without matching on its shape. One method,
no new type.

**Behaviour preservation (I8 / EX-5 / VA-1).** `commands::facet::` stayed at 33
green, and the whole of `src/commands/facet.rs`'s diff is the single production
call expression at `:711` — rustfmt split it across six lines, which is why
`--stat` reads `6 insertions(+), 1 deletion(-)` for what is one changed call.
`mod tests` (from `:1499`) is untouched. D1's boundary held exactly as it was
drawn: the five test call sites that necessarily moved are `src/facet_write.rs`'s
own, not risk set's.

### T2 — `plan_facet_edits`, the pure seam

`RawEdit` / `RawValue` in, `FacetEdit` out, `FacetEditRefusal` on refusal. The
kind-aware half of the write path, and the mirror of `validate_facet` on the
read path. No filesystem anywhere in it or in its ten tests.

`plan_facet_edits` is the **sole** constructor of `FacetEdit`, whose fields are
private to the module. That is what makes I4's facet half a property of the type
rather than a convention a future caller may forget.

`check_shape` is a separate private function: the row's `shape` is the
authority, and a `Closed` token is tested against **that row's own `KNOWN`
slice**, so neither a test nor a production line retypes a token (STD-001).
`EXPECTED_LIST` / `EXPECTED_SINGLE` are the two named `ShapeMismatch` reasons.

D6 held up under test: `""` is accepted on a `Closed` field and bypasses token
validation, because `optional_enum` maps `""` to `None` *before* parsing it. The
alternative would leave closed fields the one kind of field that cannot be
cleared.

One test beyond the sheet's list: `plan_refuses_a_text_value_on_a_list_field`.
The sheet named only the list-on-text direction; both directions are reachable
from PHASE-06's map-shaped caller, and the enum already had the variant.

### T3 — `apply_facet_edits`, the thin shell

Per D3, no second envelope: it maps `FacetEdit` → `facet_write::FacetField` and
hands it to `apply_set_mixed` with `RequirePresent`. `edit_in_place` already
gives read → parse → mutate → write-once-if-changed, which is I11's idempotence
and I7's inertness for free. `FACET_TABLE` is the one derivation of the table
name.

The I6/EX-4 test asserts on the file's **bytes** — two `std::fs::read` calls
compared — not on the error text. EX-7 is a property of the file, not of the
message.

**A correction the sheet and I both got wrong, and the build settled.** T3 was
predicted to retire `FacetEdit`'s and `KeyPosture::RequirePresent`'s
`expect(dead_code)`, because `apply_facet_edits` reads the one and constructs the
other. It retired neither. rustc's dead-code pass does not treat a
`dead_code`-suppressed item as a **live root** for what it reaches, so a staged
function reading a staged type leaves both staged; the whole chain goes live at
once when the first genuinely reachable production caller lands at T6. Both
`reason` strings were corrected to say T6.

This is precisely the property `expect` buys over `allow`: the wrong prediction
was a hard error on the next `cargo clippy`, not a stale comment nobody reads.
Recorded as `mem.fact.rust.dead-code-staging-does-not-cascade`.

### T4 — I11 / VT-3, edit preservation

`facet_bearing_decision()` gained one unknown `[facet]` sibling,
`notes = "keep me"` — no `facet_fields` row names it and `RawFacet` does not
carry it, so it is genuinely the forward-compatibility case I11 protects, not a
field in disguise. `RawRecordToml` has no `deny_unknown_fields`, so the read path
tolerates it; checked before relying on it.

Asserted by D8's three-region partition. Region (b), inside `[facet]`, is
compared **line by line and in place**: each line is either byte-identical or —
if its key was edited — holds the intended new value, read off the right of the
first `=` and trimmed. That is what keeps the assertion off `toml_edit`'s value
decor, which `insert` resets even as it preserves the key's. An `assert_ne!` on
the facet region guards against the test passing vacuously.

`inert_tail` is deliberately **not** the oracle here: it slices from `\n[facet]`
onward, which is exactly the region this phase writes. It remains PHASE-08's.

**The compensating control, watched failing twice.** T4 structurally cannot
stage a red — a correct I11 test passes the moment T3 lands, and "it compiles"
proves the assertion runs, not that it catches. So `apply_facet_edits`'s body was
temporarily replaced with a naive `toml::from_str` → mutate → `toml::to_string`
round-trip:

1. First run went red on region (a) — the naive writer alphabetises every
   top-level key, so the meta tier moved.
2. (a) short-circuits before the tail, so it was temporarily disabled to reach
   the region the sheet actually names. Second run went red on *"everything from
   `[evidence]` on must be byte-identical"*, and the failure text carries both
   predicted losses verbatim: the hand-written
   `# a hand-written comment no verb may eat` is **gone**, and `[[relation]]` has
   been **reordered** ahead of `[relationships]`.

Both temporary edits were reverted from a pre-injection snapshot rather than by
hand, and `grep -c "REVERT ME"` on the restored file returns 0.

### T5 — movement-1 close, and the hand-off to worker 2

The named exit state: three items staged behind
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, all retiring together
at T6 when the CLI dispatch arm gives the chain a production caller. They are
listed by name in the sheet's Findings. `expect` rather than `allow` throughout,
so the retirement is enforced by the build rather than remembered.

## Movement 2 — the surface (T6–T10)

### T6 — the seven subverbs, and the staging retired

`KnowledgeFacetEdit` is a seven-variant `Subcommand` in `src/knowledge.rs`, one
variant per `RecordKind`, plus `FacetEditTarget` — a `#[derive(Args)]` bundle
carrying `id` and `-p/--path`, flattened into every variant so the pair is
declared once (STD-001) and `id` stays the first positional. `raw_edits()` is
the single argv→`RawEdit` mapping; `run_facet_edit` (beside `run_edit`) is the
run shell, ordering its four refusals ahead of every write.

**Red observed**, and it was a runtime panic rather than a compile failure —
worth noting, because the sheet predicted the latter. `VT-4` names no Rust type
of this phase's (it reaches the surface through clap's introspection API), so it
compiled against the tree as it stood and failed on the assertion:

```
panicked at src/knowledge.rs:4478: `knowledge edit assumption` is a subverb
```

That is a *better* red than the predicted one — a compile failure proves only
that a symbol is missing, whereas this proves the oracle actually interrogates
the shipped command tree.

**All five `expect(dead_code)` attributes retired in one edit**, exactly as
`mem.fact.rust.dead-code-staging-does-not-cascade` predicted: the chain went
live together the moment the dispatch arm reached it. `cargo build` clean, so
none of the five was still warranted.

**Two clap findings, one of them somebody else's bug.**

1. `ISS-330` — **pre-existing, live, not this phase's.** The first spelling of
   `VT-4` did `<Cli as CommandFactory>::command()` then `Command::build()`.
   `build()` recurses *every* sibling subtree and runs each one's
   `debug_asserts`, so `doctrine config set`'s
   `required` + `required_unless_present` positional
   (`src/commands/config.rs:34`, and the same shape at `:52`/`:75`) panicked the
   test. It panics on a real invocation too — `./target/debug/doctrine config
   set --help` dies in any `debug_assertions` build. Nothing in the suite had
   ever built the whole tree, and no test invokes `config set`, so it had gone
   unnoticed. **Worked around, not fixed:** `built_edit_command()` clones and
   builds only the `knowledge` subtree, with a comment naming ISS-330 and the
   condition for removing the narrowing.
2. `clippy::large_enum_variant`. Nesting `KnowledgeFacetEdit` directly in
   `KnowledgeCommand::Edit` inlines the widest facet variant into the parent:
   384 bytes against a 152-byte second-largest, and `warnings = "deny"` makes
   that a build failure. `Option<Box<KnowledgeFacetEdit>>` fixes it at no
   derive cost — clap implements `Subcommand for Box<T>`
   (`clap_builder/src/derive.rs:375`). Call sites are unchanged by auto-deref.

Both are in `mem.fact.clap.introspect-subtree-and-box-wide-subcommand`.

**D4 confirmed against the running binary, not only the parser source.**
`doctrine knowledge edit --help` renders `[ID] [COMMAND]` side by side and all
seven subverbs; `S3` did not fire.

`src/commands/guard.rs:214` needed no change, as the sheet predicted — the arm
is `KnowledgeCommand::Edit { .. }`, so the new field does not break
exhaustiveness and the subverbs inherit the `knowledge edit` write label.

**One addition beyond the sheet:** a one-line doc comment on each of the thirty
field flags and on the two positional/`--path` args, so the shipped `--help` is
not a column of bare flag names. The `concept` variant's doc comment was also
rewritten — its first draft leaked `D10 / DEC-173` into user-facing help; the
rationale moved to a `//` comment beside it.

### T7 — VT-1, the generated round-trip, and its sensitivity control

Generated over `facet_fields`, one argv drive per field per kind, no field list
written anywhere in the test. Three helpers carry it:

- `seed_from_template(root, kind, id)` — seeds through the **shipped**
  `render_record_toml_seed`, not a hand-built fixture, so the `RequirePresent`
  posture meets the seeded keys it was designed against (A2).
- `round_trip_case(row)` — derives *both* the argv value and the `[facet]` line
  the read model must render back, from the row's own shape. A `Closed` row's
  token is its own `KNOWN`'s first (STD-001; no token is retyped).
- `drive_subverb(argv)` — `Cli::try_parse_from` → `Command::Knowledge` →
  `KnowledgeCommand::Edit { facet: Some(sub), .. }` → `sub.raw_edits()` →
  `run_facet_edit`, i.e. exactly what `dispatch` does.

The read-back oracle is the **typed** one: `read_record` runs `validate_facet`,
then `render_facet` re-emits the `RecordFacet`, so the assertion is on the read
model's view of the value rather than on the bytes just written.

**It went green on first run**, which is what the sheet predicted and why the
control is mandatory rather than optional.

**The sensitivity control, watched failing.** The first injection — deleting
`text_flag("rationale", rationale.as_ref())` and leaving the binding in place —
did not even compile: `error: unused variable: rationale`, `-D unused-variables
implied by -D unused`. That is a *finding*, not a nuisance: `raw_edits`
destructures every variant exhaustively (no `..`), so **addition** drift is a
compile error and an unused binding is too.

To reach the shape R2 actually warns about, the injection was made realistic —
`ref rationale` dropped from the pattern and `..` added, which is precisely how
the drift would appear in a real edit. That compiles, and the asymmetry is
exactly as D5 predicted:

```
test knowledge::tests::every_subverbs_flags_are_exactly_its_kinds_facet_row ... ok
test knowledge::tests::every_facet_field_round_trips_from_argv_through_its_subverb ... FAILED
  `decision --rationale` should write:
  `knowledge edit decision` requires at least one field flag
```

`VT-4` — the name oracle — cannot see it, because the flag is still *declared*.
`VT-1` sees it because it drives argv end to end. Reverted from a pre-injection
snapshot; `git diff --stat src/knowledge.rs` after revert shows insertions only
(the new test), zero deletions.

Harvested as `mem.pattern.rust.exhaustive-destructure-pins-hand-written-mappings`.

### T8 — the refusal catalogue (VT-5), and why it needed a *positive* control

Four cases, one helper. `refused_leaving_bytes_intact` seeds a record from the
shipped template, snapshots **both** authored tiers, drives the argv, asserts
the refusal, and asserts both tiers verbatim. The helper owns the scratch root
and builds the argv around it, so no case retypes a path — the earlier draft
had each case hard-coding `std::env::temp_dir().join(…)` twice over, which is
the sort of duplication that rots the first time the root convention moves.

| case | argv | criterion |
|---|---|---|
| a | `assumption DEC-007 --claim x` | EX-3 — names `DEC-007` **and** `knowledge edit decision` |
| b | `concept CPT-003` | EX-2 / D10 — "carry no facet fields", names `knowledge edit CPT-003` |
| c | `decision DEC-007` (no flag) | mirrors `run_edit`'s at-least-one-flag guard |
| d | `assumption ASM-003 --confidence banana` | refused by `plan_facet_edits`, before the open |

(d) asserts every member of `Confidence::KNOWN` appears in the refusal, read
from the row rather than retyped — STD-001, and it means a token added to the
vocabulary joins the assertion without a test edit.

**All four were green on first run, and no meaningful red was stageable.** T6
landed these guards as part of `run_facet_edit`; VT-5 tests the surface T6
built, and the sheet ordered T8 after T6 deliberately. Staging a red here would
have meant deleting a guard I had just written and watching my own deletion —
theatre, not evidence.

So the control was aimed at the *oracle* rather than the guard. The proposition
worth doubting is not "does it refuse" (four distinct messages came back) but
"would these assertions have noticed a write at all". The same helper was run
over a **valid** edit — `decision DEC-007 --rationale written` — and it failed
exactly where it should: the toml tier moved `rationale = ""` to
`rationale = "written"` while the `.md` tier stayed identical. The tier oracle
sees writes; the four greens are assertions, not vacuities. Reverted.

**Refactor out of the control.** That failure printed both tiers as `Vec<u8>` —
some six hundred decimal integers, unreadable. The oracle now reads `String`.
`String` equality *is* byte equality, so EX-7 is unweakened and a failure is
legible. Worth stating plainly: "assert the bytes" is a claim about the
comparison, not about the Rust type you hold them in.

### T9 — the two behaviour-preservation gates

Diffed against the **phase base** `c72bd7f61` (`1683a5703^`). Bare `git diff`
is working-tree-vs-index, and movement 1 is committed, so it returns empty —
which reads exactly like "nothing was edited" while being no evidence at all.

`src/commands/facet.rs`: 6 insertions, 1 deletion, all of it the single
`apply_set_mixed` call site gaining `KeyPosture::Create`, rustfmt-split across
five lines. `mod tests` starts at `:784`; the only hunk is at `:708`. D1's
boundary held exactly as movement 1 reported, and `commands::facet::` is 33
passed — the T0 baseline.

`src/knowledge.rs`: 1226 insertions, **2** deletions, both production (`Edit`'s
`id: String` → `Option<String>`, and the dispatch arm's `&id`). That is where
it would be tempting to stop, and it would be a bad stop: an insertion *inside*
an existing test body is an edit too, and it lands in the same count as a new
test appended below. So the three named risk-set items were extracted from both
revisions by brace balance and compared:

```
IDENTICAL    728 bytes  fn render_record_toml(              (the I1 oracle)
IDENTICAL    778 bytes  fn populated_record_round_trips_byte_stable_per_kind(
IDENTICAL    410 bytes  fn scaffold_escapes_hostile_title_and_slug(
```

Harvested as `mem.pattern.testing.pin-named-items-not-diff-lines`.

**S1 never fired across T6–T9.** No existing test was touched at any point.

### Movement 2 close

Full suite 4447 passed, 0 failed, 2 ignored. Five staged `expect(dead_code)`
attributes: all gone, retired together in T6.

Owed to reconcile, not fixed here: the design's §5.1 still says `FacetField`
where the code says `FacetFieldRow` (ISS-329's rename), and **ISS-330** —
`doctrine config set` panics in any debug build on a clap
`required`/`required_unless*` conflict, which is why VT-4 introspects the
`knowledge` subtree rather than the built root command. The workaround carries
the id and its own removal condition in a comment.

## PHASE-05 — settling, in one write

**How this record was written, because it bears on how much to trust it.** One
worker took T0–T5 and was killed by a session limit at T7, with the whole phase
green but uncommitted. The orchestrator recovered the working tree as
`582ac7ba8` and re-ran the gate (exit 0). A second worker wrote T6 and this
record — **reconstructed from that commit's diff, not from the first worker's
context.** Everything below is a fact about the code as it stands in
`582ac7ba8`; where the first worker's *process* cannot be recovered from the
artefact, it says so rather than guessing. One control (T1's C2) is in that
category, and it is stated as a gap, not narrated as a success.

Recorded during the phase and surviving it: `mem.pattern.doctrine.compose-two-write-cores-bind-both-legs`
(`28a5da9d8`). The habit-shaped friction — the whole execution record parked in
a single terminal task, which is what made this reconstruction necessary — is
captured as a `friction` observation against `/phase-plan`.

### T0 — the baseline (reconstructed; the pair collapsed to a point)

The first worker's T0 numbers were never written down. Re-measured at T7:
`cargo test --bin doctrine commands::facet::` → **33 passed, 0 failed**, and
`git diff --stat src/commands/facet.rs` empty.

`R-red` named the T0/T7 *pair* as the compensating control for a task with
nothing to fail. With one half missing, the surviving measurement proves the
number is 33 but not that it never moved. That gap is closed by the diff's
shape rather than by the measurement: `git show --stat 582ac7ba8` lists three
files — `src/commands/guard.rs` (+1), `src/facet_write.rs` (+7/−1),
`src/knowledge.rs` (+1193/−26) — and `src/commands/facet.rs` is not among them.
A file the phase never opened cannot have had its suite bent to pass.

### T1 — EN-2: the vocabulary check, extracted

`ensure_status_token(kind, state) -> anyhow::Result<()>` at `knowledge.rs:249`,
sited beside `statuses` (`:230`) as the sheet asked. A guard, not a predicate
(`D-E`): the refusal *sentence* is the part that must not be retyped, and it is
byte-identical to the inline original — the deleted `anyhow::bail!` and the new
`anyhow::ensure!` carry the same format string, `` `{state}` is not a {kind}
status (known: {joined}) ``.

Two callers, and only two: `set_record_status:2699` and `run_settle:2520`. A
brace-balance extraction of `set_record_status` across the commit confirms its
whole delta is the eight inline lines becoming one call — no other line moved.

`C1` discharged and labelled in place: `ensure_status_token_refuses_a_foreign_kind_state`
(`:5500`) pins the sentence as a literal with the known-set read from
`statuses`, never retyped (STD-001), plus a totality companion
`ensure_status_token_admits_every_token_of_its_own_kind` (`:5519`) that
generates over all seven vocabularies, so a new token joins the coverage without
a test edit.

**`C2` is an evidence gap, and is recorded as one.** C2 wanted either a named
pre-existing characterization test of `set_record_status`'s foreign-state
refusal (left unedited, as the control), or a new one watched green *before* the
refactor. Neither is in the tree. No test names `set_record_status`; `run_status`
— its only production caller — has no refusal test either. The sentence is
pinned at the extracted guard (`:5500`) and, through the settle path, at
`vt3_3` (`:6126`, whose doc comment names the shared check). So the extraction's
behavioural equivalence rests on **inspection** — one call site, identical format
string, unchanged callers — rather than on a control that would have gone red.
Whether the first worker ran a green-first characterization and dropped it
cannot be established from the commit, and is not asserted here.

### T2 — the `Settlement` annotation, the derived set, the pins

`Settlement { state, captures }` and `settlements(kind)` at `:1048`/`:1090`;
four rows and no more — QUE `answered`/`answer`, ASM `validated`/none, ASM
`invalidated`/none, CON `waived`/`waiver_reason`. `DEC`, `EVD`, `HYP`, `CPT`
share one `NO_SETTLEMENTS` empty slice, and its doc comment carries the point
worth keeping: the empty row is what makes `I5` hold **by shape** — `accepted`
is not guarded out of `settle`, it was never in the derived set.

`derived_settleable` (`:1113`) is the `D-A` reading: quantified over
`statuses(kind)`, kept iff `facet_fields(kind)` carries both
`<state>_by` and `<state>_on`. The two suffixes are named constants
(`SETTLEMENT_ACTOR_SUFFIX` / `SETTLEMENT_DATE_SUFFIX`), so `run_settle` spells
the keys it writes by the same derivation that admitted the state.

All four pins landed, `P-c` with both exclusions asserted by name:
`every_settlement_captures_a_field_its_kind_owns` (P-a, `:5541`),
`the_annotations_states_equal_the_derived_settleable_set` (P-b, `:5561`),
`the_derived_settleable_union_is_exactly_the_ruling_table` (P-c, `:5587`),
`kinds_without_an_actor_date_pair_derive_nothing` (P-d, `:5625`).

`R-dead` never bit: T2–T4 landed as one commit, and no `#[expect(dead_code)]`
appears anywhere in the diff.

### T3 — `apply_settlement`: the one write

`edit_in_place` promoted to `pub(crate)` (`facet_write.rs:335`, `D-B`) with the
reason in its doc comment rather than in a commit message. `apply_settlement`
(`knowledge.rs:1352`) is the composition: one `edit_in_place`, `set_facet_mixed`
under `KeyPosture::RequirePresent`, then `dep_seq::apply_status`, both bound to
locals before the `||`. The `R-short-circuit` footgun is written into the body
as a comment, not just avoided — which is the only form in which it survives the
next reader.

The `FacetEdit` → `facet_write::FacetField` conversion came out of
`apply_facet_edits` into `writer_fields` (`:1322`), used by both callers. No
copy (carried constraint 3). `malformed_status_hint` (`:2683`) is the single
source of the F-1 bail sentence for `set_record_status` and `apply_settlement`
alike (constraint 4, STD-001).

The three VT-1 tests are the sheet's, named for it —
`vt1a_a_settlement_moves_the_capture_the_actor_the_date_and_the_status` (`:5755`),
`vt1b_a_failing_status_leg_leaves_the_facet_leg_unwritten` (`:5804`),
`vt1c_a_failing_facet_leg_leaves_the_status_unmoved` (`:5838`). VT-1b and VT-1c
are the ones that carry the claim: each kills one leg and asserts the file's
bytes are unchanged, so a two-write implementation passes VT-1a and fails both.
Whether the first worker wrote the sequential version to watch them fail is not
recoverable from the commit.

### T4 — `run_settle`, the CLI variant, the guard

`run_settle` (`:2504`) implements the eight steps in the sheet's order, and the
order is documented at the function rather than left as a property of the
listing. Notes worth keeping:

- Steps 1–4 complete before `read_record`, which is what makes the "refused
  before any open" observable possible at all (see T5).
- Step 3 refuses via the **derived** set, so the message is
  `` `accepted` is not a settle transition for a decision; use `knowledge status` ``
  and no line anywhere names `accepted`. The `settlements` lookup that follows
  supplies only `captures`, and its `None` safely means "captures no text"
  rather than "unknown state" precisely because `P-b` pins the two sets equal.
- Step 4 grew a case the sheet did not list: a **blank `--by`**. clap makes
  `--by` required, but required is not non-empty, and for an assumption the
  actor is the *whole* disposition — so it is checked on the same footing as a
  capture, through the same `blank_settle_flag_refusal` sentence.
- Step 6 calls `clock::today()` once and spends the same `String` on
  `<state>_on` and on `updated` (constraint 6), so the two cannot disagree by a
  midnight.

`R-inventory` held at three touch sites, all present in the diff: the
`KnowledgeCommand::Settle(SettleArgs)` variant (`:2812`), `dispatch`'s arm
(`:3265`), and `guard.rs:216` classifying it `Write("knowledge settle")`.

`R-clippy` did not fire as predicted: `SettleArgs` is a `#[derive(clap::Args)]`
struct held by a **tuple** variant, `Settle(SettleArgs)`, not a `Box`. The
struct-per-variant shape sidesteps `large_enum_variant` without the indirection
the sheet budgeted for, and it also gives the argv→capture mapping one home —
`SettleArgs::captures()` returns the `(field, value)` pairs argv actually
carried, mirroring `KnowledgeFacetEdit::raw_edits` a tier down.

The optional flag oracle was kept: `every_captured_field_has_a_declared_settle_flag`
(`:6249`) asserts every `Settlement.captures` name has a declared `--<kebab>`
flag, so a fifth settlement cannot ship unreachable from argv. It reuses
PHASE-04's ISS-330 workaround by generalising that helper —
`built_edit_command()` became `built_knowledge_verb(verb)`. **That is a
one-line edit inside an existing test body**
(`every_subverbs_flags_are_exactly_its_kinds_facet_row`, `built_edit_command()`
→ `built_knowledge_verb("edit")`). Declared rather than buried: it is a
rename-through, no assertion changed, and it is not one of the three suites `S1`
protects.

### T5 — the refusal catalogue (VT-3, I7)

**The inventory ran larger for the third time in this slice.** The plan said
five cases, the sheet corrected it to six, and the shipped catalogue is **nine**:

| # | case | test |
|---|---|---|
| 1 | capture flag omitted | `vt3_1_an_omitted_capture_flag_is_refused` `:6072` |
| 2 | capture flag blank | `vt3_2_a_blank_capture_flag_is_refused` `:6091` |
| 2b | **`--by` blank** | `vt3_2b_a_blank_actor_is_refused` `:6109` |
| 3 | foreign-kind state | `vt3_3_a_foreign_kind_state_is_refused` `:6126` |
| 4 | non-settleable state | `vt3_4_…_names_the_escape_hatch` `:6143` |
| 5 | state-to-itself | `vt3_5_a_state_to_itself_transition_is_refused` `:6164` |
| 5b | **the D-D precedence case** | `vt3_5b_the_overlapping_case_gets_the_more_precise_remedy` `:6184` |
| 6 | withdrawn record | `vt3_6_a_withdrawn_record_is_refused` `:6210` |
| 7 | **wrong capture flag** | `vt3_a_foreign_capture_flag_is_refused_naming_the_right_one` `:6230` |

The three additions are all things the sheet's *prose* asked for (T4 step 4's
one `if`, the blank-actor case, `D-D`'s ordering) that its *table* omitted.
`5b` is the one that earns its keep as more than a variant: an already-`waived`
constraint is simultaneously the state-to-itself case and the withdrawn case,
and the test pins that it gets `D7`'s more actionable remedy, which is the only
observable difference `D-D` makes.

Two helpers carry the evidence, and the second is the interesting one:
`settle_refused_leaving_bytes_intact` (`:6026`) compares the file's bytes across
the refusal, and `settle_refused_before_any_open` (`:6053`) runs the *same*
argv against an id naming no record on disk and asserts the same refusal
message. Cases 1, 2, 2b, 3 and 4 assert both. That pair is what distinguishes
"refused early" from "refused late without writing" — a bytes assertion alone
cannot, and `I7` is a claim about *order*, not about damage. Cases 5, 5b and 6
necessarily read the record first, so only the bytes half applies to them.

Fixtures were generalised rather than pasted (`R-fixtures`): `fixture_literal`
+ `facet_bearing_record(kind, id, status)` (`:5656`, `:5675`) build any kind's
scaffold from its own `facet_fields` row, with `settle_fixture` (`:5727`)
seeding it on disk.

### T6 — VA-1: does `settle` still earn a separate verb? **Confirm.**

Design §10 press item 2 reasons: `DEC-178`'s case was *partly* that the
transition is a coupled multi-write; `F-2` made it one write of one document;
`knowledge edit question` is also one write of one document; so the remaining
case is `DEC-062`'s alone — the whole case rather than the larger half of one.

Two of those steps do not survive contact with the record and the code.

**1. The multi-write argument was never `DEC-178`'s.** Read the record
(`doctrine knowledge show DEC-178`): its `context` is that `set_record_status`
"documents itself as having no resolution coupling — `status` and `updated`,
nothing else", and its `rationale` is entirely about **reach** — the status
vocabularies laid against the facet field names. It nowhere argues atomicity or
write ordering. The ordering argument was the design's own drafted `D6`, which
`F-2` superseded. So `F-2` struck a *drafting* artefact and removed nothing from
`DEC-178`'s recorded case. The press item is accounting for a loss the ledger
never booked.

**2. "One write of one document" is true of the mechanism and false of the
reach.** `knowledge edit question` writes `[facet]` keys and nothing else —
`apply_facet_edits` (`:1305`) hands `FacetField`s to the facet writer and never
touches `status` or `updated`. `run_settle` step 7 passes
`&[("status", state), ("updated", &today)]` into the **same**
`apply_settlement` call as the facet edits. Both commands write one document;
they write **disjoint key sets**. Reaching `answered` *with* its answer through
the existing verbs takes `knowledge edit question QUE-005 --answer … --answered-by …
--answered-on …` **and** `knowledge status QUE-005 answered` — two commands,
two writes, and the window between them is exactly the 0-of-38 half-settlement.
`F-2` collapsed an ordering *inside* `settle`; it gave no other verb the ability
to move both key sets.

So the mechanical likeness is real and beside the point. What earns the verb is
what it **refuses**, and every one of these is in T4's code and none can be added
to `edit`:

- **The token moves with the disposition or neither moves** (step 7). No other
  single command spans both key sets.
- **The omitted capture is refused** (step 4): *"the disposition is part of
  resolving, not a field to fill in later"* — `DEC-062`'s rule, made mechanical.
  It cannot live on `edit`, whose job is to write one field at a time.
- **The blank capture is refused** (`blank_settle_flag_refusal`, `:2483`).
  `plan_facet_edits` reads `Text("")` as a legitimate *clear* (`:1126`), so
  without this check a blank settlement writes green. `edit` **must** keep the
  clear semantics; `settle` **must not**. The two verbs need opposite readings
  of the same empty string — on its own a reason they cannot be one verb wearing
  a flag.
- **The date is not the caller's** (step 6): one `clock::today()`, spent twice.
  `edit question --answered-on` takes any date typed and cannot stamp `updated`
  in the same act, so `D7`'s requirement that `answered_on` mean *when answered*
  is enforceable only on the settle path.

Two further supports that are structural rather than about a single write. The
reach is **derived**, so QUE/ASM/CON are covered by one rule and `accepted` is
excluded by absence (`I5`, `DEC-088`); expressing settling as an `edit` flag
combination would mean hand-listing per kind which combination constitutes a
resolution — the "coverage hand-listed rather than derived" alternative
`DEC-178` explicitly rejected. And `settle` refuses a state-to-itself
transition while `edit` exists precisely to amend one; each refusal names the
other verb as the remedy, which makes the pair a **division** rather than a
duplication.

**Verdict.** `DEC-062`'s case carries the verb on its own, and carries it
comfortably. The verb exists to make forgetting the disposition impossible, and
every mechanism that achieves that is a refusal — none of which can be added to
`edit` without breaking `edit`. `F-2` narrowed the design's *rationale* for the
verb; it did not narrow the *case*. **No id minted**: there is no open question
left to carry into reconcile, only a stale press item.

**Owed to reconcile:** design §10 press item 2 should be **struck**, citing this
adjudication — and, if the wording is preserved anywhere, corrected on the point
that `DEC-178` never made the multi-write argument. Not done here: `design.md`
is outside this phase's write scope (carried constraint 5, the storage rule).

### Divergences

- **`D-A` — design §5.2 is under-stated, not wrong.** §5.2 says a state is
  settleable when the kind's facet carries `<state>_by` and `<state>_on`,
  "mechanical over `facet_fields`", yielding four transitions. Read literally
  over `facet_fields` alone it yields **five**: `DECISION_FACET_FIELDS` carries
  `decided_by` and `decided_on`, so `decided` derives — and `decided` is not a
  decision status. The shipped derivation is the intersection `DEC-178`'s own
  rationale describes ("laying the status vocabularies against the facet field
  names"): quantified over `statuses(kind)`, filtered by the facet row. `I5`
  then holds from **both** sides — `accepted` excluded by the facet leg,
  `decided` by the status leg — and `P-c` pins both by name. `S2` did not fire.
  Reconcile should tighten §5.2's sentence to say intersection.
- **`D-C` — `apply_settlement`'s signature.** Takes `canonical` and `hint`
  beyond the design's illustrative signature (§5.2), both mechanically forced by
  the cores composed: `canonical` by `KeyPosture::RequirePresent`'s refusal,
  `hint` by `apply_status`'s F-1 bail. A divergence of signature, not of design;
  the reason is in the function's doc comment.
- **`R-withdrawn-overlap` — intended, and surprising enough to write down.**
  `waived` and `invalidated` sit in both a settleable set and
  `WITHDRAWN_STATUSES`. Consequence: an already-`invalidated` assumption cannot
  be settled to `validated`, and an already-`waived` constraint cannot be
  re-settled. `knowledge status` remains the correction path, and both refusals
  say so. This falls out of reusing one `is_withdrawn` predicate rather than
  writing a second list, which is the right trade — but it means `settle` is a
  one-way door per record, and that is a product fact, not an implementation
  detail.
- **Inventory, third occurrence.** VT-3's refusal cases: plan 5 → sheet 6 →
  shipped 9. Every addition was named in the sheet's prose and missing from its
  table. Worth carrying to audit as a pattern about this slice rather than as
  three separate deltas.

### Residual, small, for audit not for now

`kebab_flag` (`:2475`, production) and the test-only `kebab` (`:5194`,
PHASE-04's) are two spellings of one transform, `field.replace('_', "-")` — and
clap's derive is a third, since it kebab-cases `waiver_reason` into
`--waiver-reason` itself. The oracle at `:6249` binds clap to `kebab`, so the
flag *names* cannot drift; what is unbound is `kebab_flag`, which spells the
flag names inside the refusal *messages*. A mild STD-001 residual, no observable
defect.

### PHASE-05 close

- `cargo test --bin doctrine` → **4468 passed, 0 failed, 2 ignored**. PHASE-04's
  movement-2 close measured 4447, and `582ac7ba8` is the only source commit
  between: **21 tests added**, which is exactly the count of `#[test]` items in
  the diff. The two numbers were derived independently and agree.
- `cargo test --bin doctrine commands::facet::` → **33 passed**, `git diff --stat
  src/commands/facet.rs` empty. `S1` did not fire.
- `I1`/`I8` behaviour preservation, checked the way T9 checked it — by brace-balance
  extraction of the named items from `582ac7ba8^` and `582ac7ba8`, not by trusting a
  diff line count:

  ```
  IDENTICAL  728 bytes  fn render_record_toml(                             (the I1 oracle)
  IDENTICAL  778 bytes  fn populated_record_round_trips_byte_stable_per_kind(
  IDENTICAL  410 bytes  fn scaffold_escapes_hostile_title_and_slug(
  ```

  The only two production items that changed are the two intended extractions,
  `set_record_status` and `apply_facet_edits`.
- `./target/debug/doctrine check gate` → **exit 0** (run by the orchestrator on
  recovery; not re-run for this record, which changes no source).
- `doctrine slice verify-vt 249` → PHASE-05 `VT-1`, `VT-2`, `VT-3` all **PASS**.
  `VA-1` is discharged by the adjudication above.
- No `STOP` condition fired: `S1`–`S5` all clear. `S5` in particular was the live
  one, and T6 came down on confirm.

## PHASE-06 — the filled mint

The last of the three phases in this shard. It gives a `create` disposition a
`facet` slot on the wire, validates it at **admission** (before any id is
reserved), and writes it at **step 5** of the six-step mint — so a checkpoint
that creates a decision can now land `choice`, `alternatives` and the rest in
one act instead of minting a hollow record and editing it after.

Harvested per task rather than at the end (PHASE-05's terminal-task habit is
what forced that record to be reconstructed). Each task is one source commit
plus its harvest commit, and every commit was path-limited — two foreign paths
(`.claude/settings.json`, `scripts/find-empty-reqs.sh`) sat dirty in the shared
index throughout and are absent from all seven commits.

| task | source | harvest | criteria |
|---|---|---|---|
| T1 the wire slot, admission, refusals | `db93e4bd5` | `f921224df`, `b7a97b558` | `EX-2`, `EX-3`, `VT-2` |
| T2 step 5 writes it, idempotently | `803e56e98` | `0088ef7c5` | `EX-1`, `EX-4`, `VT-1` |
| T3 the digest covers it | `0096c7041` | `dd9e5a147` | `EX-5`, `VT-3` |

### What shipped

`WireFacetValue` (untagged `List`/`Text`) and `CreateRecord.facet:
BTreeMap<String, WireFacetValue>` in `src/design_run/submission.rs`; in
`src/commands/design.rs` a `raw_facet` mapper, a `plan_facet_edits` call inside
`plan_checkpoints`' `Dispose::Create` arm, `MintKind::Knowledge.facet`,
`MintPlan::record_facet()`, and an `apply_facet_edits` call in
`apply_record_effects` that is **skipped entirely on an empty slice** — a
`concept` owns no facet fields (`CONCEPT_FACET_FIELDS = &[]`), and
`KeyPosture::RequirePresent` would refuse an empty write.

`plan_facet_edits` runs **once**, at admission (`D-B`). There is deliberately no
second validation at the writer: it is the sole constructor of `FacetEdit`
(private fields), so possession of one is the proof, and a second check would be
a second place to drift.

Two files touched, `+585/−7`. The seven deleted lines are an import reflow and
one doc comment this phase rewrote — **no existing test body was edited**.
`src/knowledge.rs` and `src/commands/facet.rs` both have an empty diff for the
whole phase.

### The inventory, re-derived — and correct for once

`R7` asked for the refusal catalogue to be re-derived against the tree rather
than copied, because every counted inventory on this slice had shipped short
three times (plan 5 → sheet 6 → shipped 9, at PHASE-05's VT-3). Re-derived here:
`FacetEditRefusal` has **three** variants (`knowledge.rs:1170-1187`) and
`check_shape` (`:1260-1285`) yields **two distinct** `ShapeMismatch` messages
(`EXPECTED_LIST`, `EXPECTED_SINGLE`) — **five** reachable wire cases, which is
exactly what the sheet's table said. Fourth occurrence of the check, first time
it found nothing. All five are asserted: case **a** in `VT-2`, cases **b–e** in
`every_other_facet_refusal_the_wire_can_reach_leaves_the_run_inert`.

`EX-3` is discharged structurally: the test helper `refusal_of()` calls
`knowledge::plan_facet_edits` directly for its expectation, so no refusal message
is retyped in a test and the two cannot drift.

### Measured

- Suite **4468 → 4470 → 4472 → 4473**. The diff `5c11ea307..HEAD` adds **5**
  `#[test]` items and removes none. The two numbers were derived independently
  and agree.
- `cargo test --bin doctrine commands::facet::` → **33 passed**, `git diff
  src/commands/facet.rs` **empty** (`S1`).
- `tests/architecture_layering.rs` → **25 passed**, with **no**
  `ACCEPTED_VIOLATIONS` entry added (`S3`). `D-A`'s whole point.
- `./target/debug/doctrine check gate` → **exit 0**.
- `doctrine slice verify-vt 249` → PHASE-06 `VT-1`, `VT-2`, `VT-3` all **PASS**.
- No STOP fired: `S1`–`S8` all clear.

`VT-2` asserts `D5` whole, not merely that a refusal happened: `Err` with the
typed message in the chain; snapshot bytes identical **and** `run.revision`
still 2; no `decision/` directory on disk **and** an empty intent journal; and
the next successful create reserving `DEC-001` — the assertion that separates
"refused" from "refused *before reservation*".

### The two compensating controls

Both tasks that structurally could not stage a red ran their control, and both
behaved as the sheet specified.

**`C1` (T2, VT-1's idempotence half).** Perturbing the second submission's
`choice` to `"C1: a perturbed value"` failed the byte-equality assertion with the
difference **confined to the `[facet]` table's `choice` row** — `choice = "C1: a
perturbed value"` against `choice = "the leaf-local wire type"`, `alternatives`
and every other span byte-identical. Restored. One refactor was kept from it: the
comparison now reads the document as a `String` (TOML is UTF-8, so still
byte-for-byte) so a failure prints two documents rather than two ~700-element
byte vectors.

**`C2` (T3, VT-3).** Replacing `skip_serializing_if` with `skip_serializing` made
`VT-3` fail:

```
assertion `left != right` failed: two payloads differing only in the facet digest differently
  left:  Some(Fingerprint("16ffbeca58e6fbd0ecb00ab76937de434b45641909ec0d034f09ea5446437101"))
  right: Some(Fingerprint("16ffbeca58e6fbd0ecb00ab76937de434b45641909ec0d034f09ea5446437101"))
```

Blast radius, which the sheet asked for as information either way: **exactly one
test failed** (4472 passed, 1 failed). Deserialisation is unaffected, so T1's and
T2's tests stayed green — and no other test in the tree depends on
`CreateRecord.facet` reaching the *serialised* form. **`VT-3` is the sole guard
on `EX-5`.** Restored.

### Divergences

- **`D-A` — the design's wire snippet names a type a leaf cannot see.** Design
  §5.2 spells the facet value `RawValue`, which lives in `knowledge`, a
  command-tier module. `design_run` is a leaf with **crate out-degree 0**
  (ADR-001, `.doctrine/adr/001/layering.toml:31`) — it may not name `crate::` at
  all, and `tests/architecture_layering.rs` enforces it. So the wire carries a
  leaf-local `WireFacetValue`, converted in one place shell-side (`raw_facet`),
  following the same-file `WireKey` precedent. The name is `WireFacetValue` and
  not `FacetValue` deliberately: see the `CHR-060` entry below. Implemented as
  ruled, both halves; no layering violation baselined.
- **The sheet's `EX-4` resume fixture cannot work, and this was established from
  the tree rather than argued.** The sheet held that the abandoned-write hook at
  `design.rs:2418` leaves the intent journalled so re-submitting the same
  `submission_id` "resumes through step 5". It does not re-enter step 5 at all:
  the mint runs in full **before** the pre-write abandon, so the intent is
  journalled at `IntentState::Applied` and `execute_mint`'s `if intent.state() <
  IntentState::Applied` guard (`:1115`) skips the step whole. A resume-shaped
  fixture would have compared a file to itself with **no writer between the two
  reads** — vacuous on any tree, including one carrying no feature at all, which
  is a strictly worse failure than the one it was meant to guard. The fault hook
  cannot substitute: `injected_fault` is a hard `std::process::exit(70)`
  (`:672-690`), deliberately non-unwinding, so it is an e2e instrument only.
  **Not treated as a STOP** — `EX-4` holds exactly as written, only the recipe
  was wrong, and the sheet itself allowed "the same test **or a sibling**". The
  test now pins the guard state explicitly (`assert_eq!(journal.intents[0]
  .state(), IntentState::Applied, …)`, green — the in-tree proof that replay
  skips the step) and drives idempotence by calling `apply_record_effects` a
  second time directly, which is the property `EX-4` claims and is what a crash
  between the effect and the journal write actually produces.
- **`clippy::pedantic` refused the sheet's literal green step.** The sheet
  suggested `let _validated = plan_facet_edits(…)?;`; `[workspace.lints.clippy]
  pedantic = "deny"` makes `no_effect_underscore_binding` a hard error, so the
  call is a bare expression statement with `?`.
- **`VT-1`'s `test_file` points at `src/design_run/submission.rs`, but the test
  lives in `src/commands/design.rs`** — and this is correct, not a miss. The
  behaviour under test is an end-to-end mint, which only the command tier can
  drive; `design_run` is a leaf and cannot reach a fixture. Same residency
  split PHASE-01 took. Recorded so an audit reading the criterion literally does
  not score it as a gap.

### Carried to reconcile

Neither is a task; neither was acted on here.

- **Design §5.2's wire snippet.** It spells the facet value `RawValue`, a
  command-tier type a leaf module cannot name. `D-A` sites a leaf-local
  `WireFacetValue` instead. Reconcile settles the design's wording.
- **`CHR-060`'s title names a now-contested target.** The chore is *"Rename
  `facet_write::FacetField` to `FacetValue`"*. `D-A` declines to take
  `FacetValue` here precisely so `CHR-060` keeps its target free — but in
  reasoning about that, an argument against the chore's own premise surfaced and
  should not have to be rediscovered when someone picks it up:
  **`facet_write::FacetField` is `Str { key, value }` / `Arr { key, values }`, a
  key-plus-value pair, so it really *is* a field; the type with the better claim
  to `FacetValue` is an unkeyed value type — of which the crate now has two
  (`knowledge::RawValue` and `design_run::WireFacetValue`).** So `CHR-060` may
  want **re-aiming rather than executing**: either at a different name for the
  writer enum, or at consolidating the two value types. That is a ruling for
  `CHR-060`, with the user, not for this phase — recorded here only so the
  argument survives. `CHR-060` was **not** edited.

### Harvested

- `mem.pattern.testing.anyhow-alternate-display-for-the-cause-chain`
  (`mem_019fe1f55fc771f197bf58546e59aeb6`) — `anyhow::Error`'s `Display` renders
  only the outermost context, so `err.to_string().contains(<cause>)` fails; or,
  where the context line shares a substring, passes **vacuously**, which is the
  worse case. `format!("{err:#}")` renders the chain. Cost one cycle here.
- `mem.pattern.testing.replay-cannot-prove-idempotence-behind-a-state-guard`
  (`mem_019fe1fce9567f33b87263b85c049cd9`) — wherever recorded progress gates a
  step, a replay test exercises the guard, not the writer.
- `mem.pattern.testing.serde-skip-serializing-is-the-control-for-a-by-construction-digest`
  (`mem_019fe2010c2f7c01afc77905a79e9518`) — the reusable shape of `C2`, plus
  counting the blast radius, and excluding doc comments when grepping call sites.
- A `friction` observation (`019fe1fd-1633-7291-b969-421e8fd933e1`) on the
  `/phase-plan` beat that would have caught the `EX-4` recipe: when a sheet
  prescribes a resume/retry fixture, it must name the guard that decides whether
  the step re-runs, and require the worker to pin it.
