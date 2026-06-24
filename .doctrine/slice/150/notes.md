# SL-150 — implementation notes (harvest)

Family-grouped help + boot-map projection. Both phases complete, audited
(RV-154, clean), reconciled (no-op), closed.

## What shipped

- **`FAMILIES` taxonomy + `SPINE`** (`cli.rs`): 8 families partition all top-level
  commands; the CRUD spine (`new/list/show/paths`) is declared once. A drift-guard
  test asserts every clap command classifies into exactly one family.
- **`render_top_level_help`** (human) and **`render_boot_map`** (dense, plain-text)
  — one taxonomy, two renderings, both walking the same clap tree so neither can
  drift from the real commands.
- **`--boot-map`** flag intercept (`main.rs`) — wins over `--commands`.
- **`SourceKind::CommandMap`** boot section, injected after "Routing & Process"
  (before Governance), rendered by an injected `command_map: fn() -> String`.
- **DRY lift (D8/OQ5):** full-width family-heading bands + row-start probe lifted
  from `search.rs` into shared `listing.rs` helpers; `search` refactored onto them
  behaviour-stable.

## Durable findings

- **D9 back-edge footgun** → recorded as `mem.pattern.lint.back-edge-tangle-inject-fnptr`
  (`mem_019ef885e7d97803a6617f48f9644fd6`). A direct `boot → cli` call closes a
  same-tier import cycle and ratchets the ADR-001 Command `tangle_baseline`
  (123 → 144). Resolved by dependency inversion — inject the renderer as a
  fn-pointer from `cli` (which already depends downward). The tangle gate IS the
  guard; baseline stays 123.

## Environmental gotchas (audit)

- Auditing in a **fresh worktree** fails to compile `src/map_server/assets.rs`
  (RustEmbed `#[folder = "web/map/dist/"]`) because `web/map/dist/` is a gitignored
  build artifact absent from a clean checkout. Symlink it from the main worktree to
  verify. Orthogonal to this slice. (Cf. `crane-strips-non-rust-embeds`.)
- `cargo test` under `DOCTRINE_RESERVATION_FALLBACK=1` fails
  `reserve::tests::vt3_auto_degradation_is_fail_closed_with_explicit_optin` — the
  env var IS the opt-in the test asserts absent. Run reserve tests with the env
  unset; module is 18/18 green.
- `just check`/`just gate`/`cargo fmt` are **mutating** (run `fmt` first) and would
  rewrite pre-existing CHR-025 rustfmt debt in `boot.rs`/`main.rs`/`memory.rs`
  (7 `fmt --check` diffs, all outside this slice's hunks). Verify with
  `cargo clippy --workspace` + `cargo test` instead; CHR-025 tracks the debt.

## Follow-up

- **IMP-135** (CLI help consistency pass) — left for separate revisit per scope
  ("do not fold"); may be partially subsumed by this slice's family regrouping.
