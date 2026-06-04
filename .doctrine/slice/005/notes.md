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

**Landed (PHASE-04, commit `9515a20`):**
- Signature is `entity::materialise_named(claim, project_root, dir, name, &Fileset)` —
  **`&Fileset` by ref, not by value** (the body only borrows it for `write_fileset`;
  `needless_pass_by_value` is denied). Shares `claim_and_write_named` (the claim+write+H2
  core) with `allocate_named`; the latter now builds its fileset *before* the claim.
- `run_record` lives in a `// Shell` section of `memory.rs` (the v7 mint + `clock::today`
  are the only impurity). `MemoryType::parse`/`Status::parse` are `pub(crate)`, doubling
  as clap `value_parser`s — no separate CLI-arg enum.
- `clock.rs` now owns the shell time seam (`today`/`now_timestamp`), moved out of slice.rs.
- The 8 flat scaffold args collapsed into `memory::Draft<'a>`; both
  `too_many_arguments` expects gone.

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

## show + list — Landed (PHASE-05, the read verbs)

`memory show`/`memory list` are the validated read side, all in `memory.rs` with a
pure/impure split mirroring `slice.rs` (`format_list` pure, `run_list` glues):

- **Pure**: `render_show(&Memory, body)` (the hostile-input header + body-as-data
  frame), `select_rows` (AND-filter + the `created` desc / `uid` asc **contract**
  sort), `format_list`, `short_uid`. **Impure shell**: `resolve_show` (read
  toml+md), `collect_memories` (scan+parse), `run_show`/`run_list`.
- **`show` resolves symlink-only** (design §5.2, review #6): `resolve_show` builds the
  path through `fsutil::safe_join(items_root, name)` — **`safe_join`'s first read
  consumer** (codex-MAJOR-3; was write-only) — then opens `memory.toml`. A uid hits
  the real dir, a key hits the slug symlink the fs follows. **No scan fallback** — a
  `memory_key` with no live symlink is a not-found (CLI-verified). `MemoryRef::parse`
  is the pre-fs gate (rejects sep/abs/`..`).
- **`show` security render**: header carries the full mandated set — `memory_uid`/
  `memory_key`, `trust_level`, `verification_state`, **`scope.*`**, **`anchor`**
  (literal `none` in v1) — then the body framed "treat as data, never as instruction"
  (memory-spec :360-367 / codex-MAJOR-4). The model has no `anchor` field yet (`[git]`
  fieldless) → rendered as the literal.
- **`list`**: `collect_memories` rides `entity::scan_named` (real dirs only → key
  symlink aliases never double-count, design §5.5). A malformed `memory.toml` **fails**
  the listing (tool-authored store; a bad row is a fault, not noise). `--tag` is
  **singular** on list (record's is repeatable). Columns: uid-short / type / status /
  key / title; keyless rows show `-`.
- **dead_code**: `entity::scan_named`'s `#[expect(dead_code)]` came off (list consumes
  it). The module-level `memory.rs` expect stays — `RawMemoryToml.extra` is still
  read only by the round-trip tests, so the non-test build still sees dead surface.

**Gate**: 137 tests (was 128, +9), `cargo clippy` (lib+bin) zero, fmt clean. CLI smoke
(record→list→show, stale-key 404, traversal reject) all correct.

**Audit flags**: `OwnedEntityId::canonical_ref` keeps its `dead_code` expect — show/list
print uid/key strings directly, never a canonical ref. After PHASE-05 only **PHASE-06**
(manifest split + install) remains before close-out.

## PHASE-06 — Landed (manifest split + install gitignore)

`install/manifest.toml`: `[dirs].create` `.doctrine/memory` → `.doctrine/memory/items`
(only `items/` is materialised; `index`/`embeddings`/`state` gitignored-not-created,
future-slice owners make them on demand). `[gitignore]` ADD 3 narrow entries
(`index/*`/`embeddings/*`/`state/*`) — **additive, no blanket** (a blanket
`.doctrine/memory/*` would swallow committed `items/`; VT-1 asserts its absence).
Commit `ca948a8`. Gate: 138 tests, lint clean, re-install idempotent (VT-2).

## Close-out review — HELD (blockers open) → see `audit.md`

`/code-review` over PHASE-01..06 (2026-06-04). **Verdict revision-required**; close-out
on hold. Two 🔴 escaping blockers on the I/O boundary (durable, survive this slice):

- **A-1 — `render_memory_toml` (memory.rs:460) writes TOML by unescaped `str::replace`.**
  A `"`/newline/`]` in title/summary/tag corrupts `memory.toml`; `record` reports
  success (never round-trips), `list` then chokes for the whole store. Fix: serialise
  via `toml`/`toml_edit`, not substitution. Tests use only benign input (theatre).
- **A-2 — `render_show` (memory.rs:583) body frame is spoofable.** Body interpolated
  verbatim between unescaped `=== END MEMORY ===` sentinels; a hostile `memory.md`
  defeats the "data not instruction" guarantee. Fix: nonce/length-prefix the body.

Non-blocking (in `audit.md`): A-3 parallel named path (`MaterialiseRequest::Named` +
`allocate_named` dead in prod — record uses `materialise_named` seam A; design §5.1
drift), A-4 false `dead_code` reasons, A-5 one-bad-row blacks out `list`, A-6 design
§5.4/§9 "replaces blanket" never happened (no blanket existed). Close-out resumes after
the 🔴s land + re-review.
