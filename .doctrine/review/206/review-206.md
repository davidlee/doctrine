# Review RV-206 — code-review of SL-184

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack for this code-review of the PHASE-01 + PHASE-02 implementation
in the SL-184/phase-01 worktree (commits 35ee40fa, e2972622):

1. **Rename completeness** — every `Find`/`find`/`memory_find` reference across
   CLI, internal, MCP, tests, and docs must be renamed. Stale comments or doc
   strings referencing "find" in the renamed surface are regressions.
2. **Shared listing spine adoption** — column model, rendering, colour, and
   pagination correctness against the existing `listing.rs` spine.
3. **Edge cases** — pagination at boundaries (offset >= total, empty results),
   the `--columns` flag bleed into `retrieve`, truncation notice accuracy.
4. **Allocation hygiene** — unnecessary per-invocation allocations (search_columns
   per call, JSON Vec intermediary).
5. **Default-value correctness** — search vs retrieve share `FindRetrieveArgs` but
   have different default semantics; value leakage across the boundary.

## Synthesis

**Overall: solid**

The implementation is mechanically sound and faithful to the design. The rename
(PHASE-01) is complete across all surfaces — CLI, MCP, internal, tests — with no
missed references. The `alias = "find"` is correctly configured as a silent clap
alias. The shared listing spine adoption (PHASE-02) correctly wires `search_columns()`
through `select_columns` → `render_columns` with appropriate colouring (uid cyan,
type/status/trust by-value, title zebra-stripe). Test coverage was rewritten from
the old `format_find_table` assertions to column-definition + render-columns tests
with comfy-table output expectations.

**Three fix-now items remain:**

1. **F-1** — Stale "find" terminology in the `run_search` doc comment (3 occurrences).
   Trivial string fix.
2. **F-2** — `RETRIEVE_LIMIT_DEFAULT` used as `page_size` fallback in the search
   truncation notice. When `--offset > 0` without `--limit`, the notice incorrectly
   reports "page size 5". Fix: `limit.unwrap_or(shown)`.
3. **F-5** — Redundant `.min(ranked.len())` in pagination end-bound. Remove for clarity.

**Three tolerated items:**

- **F-3** — JSON intermediate Vec allocation: negligible cost, shared `From` impl
  with MCP path, not worth refactoring.
- **F-4** — `--columns` bleed into `retrieve --help`: design-acknowledged, help text
  already warns "(ignored by retrieve)".
- **F-6** — `search_columns()` per-invocation construction: `Candidate<'a>` lifetime
  precludes a `LazyLock`; 15 structs on the stack is invisible for CLI invocations.

**No blockers.** The implementation can land after F-1, F-2, and F-5 are addressed.

**Standing risks:** none — the rename is mechanical and proven by tests; the
listing spine is a well-exercised shared path (REC/REVIEW list surfaces).

**Haiku:**

`find` renamed to `search`  
The old hand-rolled table dies  
Three small fixes remain
