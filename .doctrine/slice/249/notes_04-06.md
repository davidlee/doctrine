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
