# Audit — SL-005 Memory entity v1

Hand-authored (no `slice audit` scaffold yet — known CLI gap). Close-out review of
the full slice (PHASE-01..06) against `design.md` + `plan.toml`. Source: `/code-review`
(staff-engineer pass), 2026-06-04. **Close-out is on HOLD** — two blockers below.

## Verdict

**revision-required.** Engine widening is sound and the numeric suites are
preserved as designed. The blockers are both *escaping* gaps on the I/O boundary —
the write path renders TOML by string substitution, the read path frames a body it
does not sanitise — and they live exactly where the design flagged the risk
(hostile input, no-parallel-implementation).

## Blockers (must clear before close-out)

### A-1 🔴 TOML written by unescaped `str::replace` — silent corruption

- **Where**: `src/memory.rs:460` `render_memory_toml` + `install/templates/memory.toml`
  (`title`/`summary`/`tags` lines).
- **What**: interpolated values are spliced raw into `key = "{{v}}"`. A `"`, newline,
  or `]` in `title`/`summary`/`--tag` produces invalid TOML or injects keys.
- **Demonstrated**: `doctrine memory record 'broke"n title' --type fact` writes
  `title = "broke"n title"`, **reports success**, and the next `memory list` dies with
  `expected newline`. `record` never round-trips what it writes → corruption is silent
  until a later read.
- **Blast radius**: compounds with A-3 — one corrupt row blacks out `list` entirely.
- **Fix direction**: serialise through `toml`/`toml_edit` `Value`s (the read path's
  own serde stack) instead of `str::replace`, or escape every interpolated value.
- **Test debt**: `render_memory_toml_substitutes_and_parses` (`memory.rs:1015`) only
  uses benign input — it asserts nothing about hostile interpolation. Add quote /
  newline / bracket cases to title, summary, tag.

### A-2 🔴 `show` security frame is spoofable — the guarantee it advertises is unenforced

- **Where**: `src/memory.rs:583` `render_show`.
- **What**: `body` is interpolated verbatim between unescaped `=== END MEMORY ===`
  sentinels. A hand-edited / hostile `memory.md` (memory-spec § Security :360-367 — the
  stated threat model) emits the END sentinel then instruction-shaped text; a reading
  agent sees it *outside* the memory block. The header promises "data, not instruction";
  the renderer does not keep the promise.
- **Fix direction**: fence the body with a per-render nonce, or length-prefix it —
  something an adversarial body cannot reproduce.
- **Test debt**: `show_render_carries_the_full_header_and_frames_the_body_as_data`
  (`memory.rs:1300`) uses a benign body. Add a body carrying the END sentinel.

## Non-blocking findings

### A-3 🟠 Parallel named path / design drift

- `MaterialiseRequest::Named` (`entity.rs:131`) + `allocate_named` (`entity.rs:384`)
  have **no production caller** — `run_record` uses `materialise_named` (`entity.rs:411`,
  "seam A"). Two ways to materialise a named entity; the `materialise(Named)` arm is
  dead in prod, alive only on its own tests (`:1025`/`:1053`/`:1093`).
- Design §5.1 promised memory rides `materialise(MaterialiseRequest::Named)`; seam A
  diverged (correctly — `ScaffoldCtx` can't carry type/status/summary/tags) but the
  abandoned arm was never retired. CLAUDE.md: *no parallel implementation*.
- **Disposition (fix-up, 2026-06-04 — DONE, delete)**: deleted `MaterialiseRequest::Named`
  + `allocate_named` + their three tests. `materialise_named` (seam A) is now the **single**
  named path; its tests already prove the shared `claim_and_write_named` core (duplicate,
  H2 won-dir cleanup, pre-existing-alias rollback) the dead arm re-proved. Cascade: with no
  named entity ever riding `ScaffoldCtx`, `EntityId` was orphaned — collapsed it entirely;
  `ScaffoldCtx` now carries `id: u32` + `canonical: &str` directly and `numbered()` is gone
  (numeric scaffolds read the fields). Numeric suites (slice/state) stayed green unchanged —
  the behaviour-preservation gate held.

### A-4 🟠 `#[expect(dead_code, reason=…)]` strings that lie

- `entity.rs:140` Named-variant reason claims "constructed by `memory record`" — it is
  not. `entity.rs:168` `canonical_ref` reason claims "read by the memory verbs" — only
  an entity test reads it (`:1047`; notes.md already flagged this). Suppressions hold,
  reasons rot. Wire them or correct the reason. Folds into A-3.
- **Disposition (fix-up, 2026-06-04 — DONE)**: both lying suppressions are gone with the
  code they guarded — the `Named` variant and `allocate_named` were deleted (A-3), and
  `canonical_ref` (its false reason) was deleted as genuinely dead. No suppression was
  retained; no honest one was needed.

### A-5 🟡 One bad row blacks out `list`

- `src/memory.rs:716` `collect_memories` fails the whole listing on a single malformed
  `memory.toml`. Design accepted this on "tool-authored ⇒ a bad row is a real fault" —
  but A-1 shows the tool itself writes malformed rows. Re-evaluate once A-1 lands; may
  be acceptable when the writer can no longer emit corruption.
- **Disposition (fix-up, 2026-06-04 — accept, no code)**: A-1 closed the only path by
  which the tool emitted corruption — the writer now serialises through `toml`, so a
  malformed `memory.toml` can only arrive by hand-edit, which *is* a real fault. The
  design's "fail the listing" stance is now sound; left as-is.

### A-6 🟡 Design drift — manifest "replaces blanket" never happened

- `design.md` §5.4/§9 assert the install change "replaces `.doctrine/memory/*`" and the
  test asserts the blanket is "replaced". No blanket ever existed; PHASE-06 shipped
  *additive* (correct). Plan/phase notes caught it; `design.md` was never back-patched.
  Left as a recorded premise-correction; not re-editing approved design post-hoc.

## Credit (no action)

- 👍 `entity.rs:81-186` — `EntityId`/`MaterialiseRequest`/`OwnedEntityId` is a clean
  invalid-state-removing widening; each request variant *is* its payload.
- 👍 `entity.rs:531-574` — component-wise `ensure_parent_dirs` + `remove_dir` (never
  `remove_dir_all`) rollback, concurrent-writer reasoning explicit.
- 👍 `memory.rs:203,694` — `MemoryRef::parse` boundary reject + `safe_join` on the read
  path: defence in depth (codex-MAJOR-3 honoured).

## Close-out gate (blocked on A-1, A-2)

- [x] A-1 fixed + hostile-interpolation tests green.
- [x] A-2 fixed + sentinel-spoof test green.
- [x] A-3/A-4 dispositioned (delete or unify the named path).
- [ ] Re-review (warm reviewer) → verdict ≥ acceptable.
- [ ] Then: `slice-005.toml` status `proposed` → done; harvest residuals.

## Fix-up record (2026-06-04 — for the warm re-reviewer)

Scope held to A-1/A-2 + the A-3/A-4 disposition. TDD, red first on both blockers.

- **A-1** — `render_memory_toml` no longer splices values with `str::replace`. The four
  user-supplied lines (`title`/`summary`/`tags`/`key`) emit through `toml_string` (a thin
  `toml::Value::String(_).to_string()`) — the serializer owns escaping. Template lost the
  hand-quotes on the `title`/`summary` lines (the literal is now quoted). RED:
  `render_memory_toml_escapes_hostile_interpolation_and_round_trips` (`"`, newline, `]` in
  title/summary/tag) — re-parses + round-trips. Verified in the real binary: a title with
  `"`/`]`/newline records, lists, and shows clean (serializer chose a `"""…"""` literal).
- **A-2 — DONE (shell nonce, 2026-06-04)** — `render_show` now fences the body with a
  per-render random **nonce**, not the uid. A body author owns the dir named by the uid,
  so a uid-derived guard was forgeable; the nonce is minted in the impure shell
  (`run_show`: `uuid::Uuid::new_v4().simple()`, dep already enabled) and threaded into the
  still-pure `render_show(&m, &body, &nonce)`. Header carries `body-guard: <nonce>`,
  terminator `=== END MEMORY <nonce> ===`. RED strengthened to the case the uid-guard
  could NOT defend: a body embedding the memory's OWN uid sentinel — now bounded, the real
  close carries the unpredictable nonce. Real-binary smoke: the guard differs every render.
  Residual (inherent, not deferrable): any sentinel frame is advisory — binds a cooperating
  reader, not the bytes; the nonce defeats forging the close, it cannot compel a reader. No
  "future secret" deferral — the nonce IS the secret.
  RED: `show_render_fences_a_body_that_spoofs_the_end_sentinel`.
- **A-3/A-4/A-5** — see each finding's *Disposition* line above. Net engine change: the
  named placement path is singular (`materialise_named`); `EntityId`/`numbered()`/
  `canonical_ref`/`MaterialiseRequest::Named` deleted.

**Gate**: `cargo clippy` (lib+bin) zero, `cargo test` 137 green (was 138; net −1 from the
3 deleted dead-named tests + 2 new hostile-input tests), `cargo fmt` clean. Slice status
left `proposed` (not flipped) pending re-review.
