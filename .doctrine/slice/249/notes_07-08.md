# SL-249 — notes shard, phases 07–08

Execution record for this shard's phases. `notes.md` stays the orchestrator's.

> Shard name: the PHASE-08 sheet's T9 says `notes_08.md`; the orchestrator's
> brief says `notes_07-08.md`. Took the orchestrator's name (it owns the
> sharding scheme) — noted so nobody hunts for a missing file.

## PHASE-08 — the kind-blind `knowledge edit` verb

### T1 — share the body-flag helpers (D1)

Moved `resolve_body`, `parse_body_mode` and `BODY_MODE_REQUIRES_BODY` from
`src/memory.rs` to `src/input.rs` as `pub(crate)`, doc comments carried across.
`parse_body_mode` had **no** self-contained unit tests to move — its only
coverage is `memory edit`'s behaviour suite (`unknown body mode` asserted at
`src/memory.rs:8789` through the verb), so nothing moved with it.

Call sites updated: three production (`memory.rs` `run_record`'s body resolve,
`run_edit`'s mode + body resolve) and two test (`record_body_from_stdin`), plus
two `BODY_MODE_REQUIRES_BODY` test assertions. Call form is qualified
(`crate::input::…`) rather than a fresh `use`, so the module of origin reads at
the call site.

`tests/e2e_mcp_server.rs:1067`'s doc comment named `memory::BODY_MODE_REQUIRES_BODY`
as the const it deliberately mirrors; repointed at `input::`. The literal
duplication there is intentional (an integration crate cannot see `pub(crate)`)
and was left alone.

STOP-4 did not fire: `cargo test --bin doctrine memory` — 313 passed, no
assertion changed. Clippy workspace clean, `cargo fmt --check` clean.

### T2/T3/T5 — VT-1 and the verb

Red first: the two VT-1 tests failed to compile on `cannot find function
run_edit` / `cannot find struct EditFields`. Not committed red — this is the
primary worktree and another agent shares the tree, so a non-compiling commit
on `edge` is a cost borne by someone else. Red/green/refactor is preserved in
the working order, not in the commit boundary.

**The byte-equality oracle.** VT-1's "everything else unchanged" is asserted by
comparing the file text from `\n[facet]` to EOF, before and after. That tail
holds populated `[facet]` values, `[evidence]`, `[relationships]`, a
`[[relation]]` row and a hand-written comment. Comparing the *tail* rather than
diffing whole files is what makes the assertion legible: everything the verb is
allowed to write is a top-level scalar above it. Two fixtures — a facet-bearing
`DEC-007` and an empty-facet `CPT-003` (DEC-172) — cover both ends of the facet
spectrum with one code path and no kind dispatch, which is EX-5's claim.

**Signature.** `run_edit(path, reference, &EditFields, writer)` — a writer
param, not `run_status`'s direct `io::stdout()`, so the post-state print is
assertable. `EditFields` mirrors `memory::EditFields`. `root::find` returns an
explicit `path` verbatim, so tests drive the real verb against a temp root with
no marker file.

**`--body -` stdin is NOT reachable through the verb.** `run_edit` calls
`io::stdin()` internally, exactly as `memory::run_edit` does; the injectable
seam is `input::resolve_body(raw, &mut impl Read)`. So VT-2's stdin arm is
asserted on that helper with a `Cursor`, which is also how `memory.rs`'s own
`record_body_from_stdin` does it. Parity with the precedent was preferred over
threading a reader through a second verb's signature for one test.

**`write_record_body` gained `mode`** (D3), and the *"always `BodyMode::Replace`"*
sentence moved to `src/commands/design.rs`'s call site, where the reason
(a resumed step 5 re-applies the same payload and must produce the same bytes)
actually lives. Still exactly one path to a record's `.md` — I4's prose half.

**A fourth site the sheet did not name:** `src/commands/guard.rs:212` matches
`KnowledgeCommand` exhaustively for the read/write classification, so a new
variant is a compile error there until classified. `Edit` → `Write("knowledge
edit")`. Cheap, and the exhaustive match is the design working as intended —
worth knowing before adding any verb to a guarded command enum.

### T4/T6/T7 — body-mode parity, the guards, and the no-op

**Divergence, stated plainly: T4 and T6 had no red step.** The sheet ordered
T4 (red, VT-2) before T5 (green, the mode parameter), and T6 as red→green. In
practice T3's `run_edit` could not compile without `write_record_body`'s `mode`
parameter, so T5 landed with T3; and `run_edit`'s guards are part of the same
function, so T6's refusals existed before their tests. Both test sets therefore
passed on first run. That is a weaker signal than red/green, so the guard
assertion was checked with a **positive control**: swapping the totality guard
and the at-least-one-flag gate makes
`knowledge_edit_refusals_leave_both_tiers_byte_identical` fail, and restoring
the order makes it pass. The ordering assertion is not vacuous.

**VT-2's oracle is `entity::write_body`'s doc comment**, read line by line, not
recollection. Four arms plus vocabulary: `replace` overwrites; `append` onto a
body ending in one `\n` inserts exactly one blank line; `append` onto a body
already ending in `\n\n` does NOT double it; `  APPEND ` normalises; `prepend`
is a worded refusal. Nothing about prose semantics was decided here (STOP-3
never came close to firing).

**The no-op is asserted across all three concerns, not just the body** — same
title, a tag already present, byte-identical prose → no `write_atomic` on
either tier, both mtimes hold, `updated` still `2026-01-01`. The concept
fixture's `tags` are deliberately **unsorted** (`["zeta", "alpha"]`) so R2 is
exercised: `apply_tags_set` compares as sets, so an idempotent re-add against a
hand-authored store does not spuriously write. A separate test pins R1's other
end — a body-only edit DOES stamp `updated`, exactly once.

**A hazard found in `apply_tags_set` and designed around.** Its self-heal
(insert `tags = []` when absent) runs BEFORE its set-compare no-op guard, so a
call with an empty `adds` on a record missing `tags` mutates the held document
while returning `false`. On the "one open, one write" shape that means a
`tags = []` could ride out on the back of an unrelated concern's change.
`run_edit` sidesteps it by not calling the core at all when `adds` is empty.
Captured as `mem.fact.doctrine.apply-tags-set-self-heals-before-noop`.

### T8 — the diff read (EX-3, EX-5)

Base `a1c603bee` (this phase's first parent) to working tree. Six files:
`src/knowledge.rs`, `src/input.rs`, `src/memory.rs`, `src/commands/design.rs`,
`src/commands/guard.rs`, `tests/e2e_mcp_server.rs`.

Added lines were split at the `mod tests` boundary (`src/knowledge.rs:2101`)
so production and test adds are checked separately — 195 production lines.

- **EX-5 (no kind dispatch, no facet knowledge, no facet table).** Zero
  occurrences of any `RecordKind::<Variant>` in the production adds; zero
  `match kind`, `RecordFacet`, `validate_facet`, or any facet field name
  (`choice`, `rationale`, `consequences`, `decided_by`, `decided_on`,
  `alternatives`, `claim`, `verdict`); zero new `FACET`-ish const or table.
  **Positive control** (`mem_019fa18161f47651af7687d8dccbbc67` — a negative
  grep needs one): the same pattern matches **4** times across the full diff,
  all in the test fixtures (`RecordKind::Decision` / `::Concept` driving
  `seed_record`). The grep is live; the production count of 0 is a fact, not a
  broken pattern.
- **EX-3 (relations untouched).** Zero calls to `append_edge`, `remove_edge`,
  `apply_string_append`, and no write to `"supersedes"` / `"superseded_by"`
  anywhere in the diff. The only `relationships` / `facet` strings in the
  production adds are inside doc comments explaining why the verb stays off
  them. Backed by the stronger evidence: VT-1 asserts the file bytes from
  `[facet]` to EOF — populated facet values, `[evidence]`,
  `[relationships]`, the `[[relation]]` row and a hand-written comment — are
  identical before and after, on both fixtures. Asserted on bytes, never by
  reading an error.

### T9 — harvest

- `mem.pattern.doctrine.new-cli-variant-needs-guard-classification`
  (`mem_019fe0eacfd27693baf5960c7d639857`) — the `guard.rs` exhaustive match.
- `mem.fact.doctrine.apply-tags-set-self-heals-before-noop`
  (`mem_019fe0eafebb7ab1b74f02c2b43d6bbb`) — the self-heal-before-no-op hazard.
- friction `019fe0eb-21fc-7140-b39c-91627b691125` — the sheet's reading list
  omitted `guard.rs`.
- friction `019fe0eb-2203-7cf3-a2cc-d97ff3af170a` — the notes-shard name
  disagreed between the sheet and the spawning brief.

Nothing owed to the reconciliation brief from this phase beyond the T4/T6
red-step divergence above, which the orchestrator may want in `notes.md`'s
*Owed* ledger.
