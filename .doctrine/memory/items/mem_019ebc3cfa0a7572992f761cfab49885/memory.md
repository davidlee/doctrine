# comfy-table custom_styling is inseparable from tty/crossterm; force_no_tty mandatory

Adopting `comfy-table` for ANSI-aware column width forces a chain of dependencies
and one load-bearing call. Verified against crate **7.2.2**:

- `custom_styling = ["dep:ansi-str", "dep:console", "tty"]` and
  `tty = ["dep:crossterm"]` (Cargo.toml). You **cannot** get ANSI-aware width
  measurement without pulling crossterm — `default-features = false` does not save
  you. Accept crossterm as a transitive dep.
- The `tty` feature makes the content formatter read the terminal at *format time*:
  `Table::should_style()` → `is_tty()` → `stdout().is_terminal()`
  (`src/table.rs:396,360,371`). So `ContentArrangement::Disabled` alone buys
  **neither determinism nor purity** — piped output stays terminal-dependent.
- Fix: call `Table::force_no_tty()` before `to_string()`. This is the seam that
  keeps a pure render layer pure and piped goldens byte-stable. Treat it as
  load-bearing, not optional; pin it with a test that asserts identical bytes under
  forced-terminal vs forced-pipe stdout.

Context: SL-053 terminal output polish (`src/listing.rs::render_table`). An internal
adversarial pass assumed `custom_styling` was crossterm-free; an external codex pass
refuted it against the real manifest. Lesson: verify a crate's feature graph against
the *resolved* manifest, not the crate's prose docs.
