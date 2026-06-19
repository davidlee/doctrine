# SL-026 — implementation notes

Harvested at audit (RV-099). Durable findings from the phases + the post-implementation
audit; the structured audit trail is RV-099 (`## Synthesis` / `## Reconciliation Brief`).

## The INV-7 spec-date finding (the one that mattered)

- **Specs are the only doctrine kind with no authored date on disk** — slices, ADRs,
  and backlog items carry `created`/`updated`; specs carry none (§5.4).
- The projection's `head.created.unwrap_or_default()` therefore emitted `date: ""` for
  every spec — a hard break against lazyspec's mandatory `%Y-%m-%d` `DocMeta.date`.
- **The clean fixtures hid it.** Every in-memory test corpus set a date; only a live
  smoke over the real (dateless) corpus surfaced the class. Lesson baked into a
  regression test that loads a **real dateless scaffold**, not a dated fixture.
- **Resolution (consult 2026-06-19):** inject the spec toml's filesystem **mtime** as
  the date, in the impure `load_spec` shell (`clock::date_of_system_time`); `project()`
  stays pure. Lossy-v1, read-only tradeoff — mtime is checkout-unstable across clones.
- **Durable fix deferred to IMP-108** — authored `created`/`updated` on the spec
  schema. Once it lands, `spec_date`'s mtime fallback can be removed.

## Integration

- Slice authored 29 commits behind main; the dispatch candidate 3-way-merged onto
  current main **conflict-free**, gate green on the merged surface (1913 tests, clippy
  clean), INV-7 0/433 over the live corpus.
- `clock.rs` now has a single `fmt_date` shared by `today()` + `date_of_system_time`
  (no parallel date formatter — the module contract).

## Lifecycle

- Phase-04 tracking + slice status had drifted in disposable per-worktree runtime
  state; reconciled to 4/4 and advanced `plan → audit`. The four `dispatch/026` phase
  commits + the fix `5de8e723` are the authoritative record.
