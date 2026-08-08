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
