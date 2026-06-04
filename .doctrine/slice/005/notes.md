# Notes SL-005: Memory entity v1

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Seam A — how memory threads its non-identity render fields (PHASE-03 decision)

The design (§5.1) widened the engine for a *named* identity but never specified how
memory's **non-identity** record fields (`type`/`status`/`summary`/`key`/`tags`)
reach the scaffold. They do not fit `ScaffoldCtx` (`eid`/`slug`/`title`/`date`), and
numeric `Kind`s are `const` so `Kind.scaffold` must stay a bare `fn` (no captured
draft). PHASE-03 surfaced this gap; resolved with the user as **seam A**:

- Memory renders its whole `Fileset` **eagerly** in a rich free fn
  `memory::memory_scaffold(uid, key, type, status, title, summary, date, tags)` — the
  shell mints the uid up front (PHASE-04), so nothing needs the engine to hand back a
  name. Pure: values in, `Fileset` out.
- PHASE-04 adds `entity::materialise_named(claim, project_root, dir, name, fileset)`
  that claims `<dir>/<name>` and hands the **pre-built** fileset to the existing
  `write_fileset` (+ the H2 "Won ⟹ ours ⟹ clean a partial scaffold" guarantee).
  Factor the claim+write+H2 core it shares with `allocate_named` — **no parallel
  writer** (CLAUDE.md). Restore the `MEMORY_ITEMS_DIR` const beside it.

Why not the alternatives: an `Option<&MemoryRender>` on `ScaffoldCtx` reintroduces the
exact Option-bag D8 removed; a `Box<dyn Fn>` scaffold breaks the `const` numeric
kinds. A is the only one with no invalid-state surface and zero numeric-caller churn.

**Consequences (also flagged for the audit):**
- **EX-3 reinterpreted** — no const `MEMORY_KIND { dir, scaffold }`; the `dir` const +
  the `memory_scaffold` builder play that role. Phase criteria are immutable, so this
  is an interpretation note, not a renumber.
- `MaterialiseRequest::Named`'s `#[expect(dead_code)]` does **not** come off in
  PHASE-04 (run_record uses `materialise_named`, not the request variant). It remains
  the engine's named-via-`const`-scaffold capability, test-covered. (Supersedes the
  PHASE-02 handover's "Named → PHASE-04 [dead_code off]" note.)
- The 8 flat args on `render_memory_toml`/`memory_scaffold` carry
  `#[expect(clippy::too_many_arguments)]`; a `Draft` struct in `run_record` (PHASE-04)
  is the natural collapse — built when its consumer shapes it, not before.
