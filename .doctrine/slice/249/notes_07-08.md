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
