# Stale rust-analyzer diagnostics after a build are false — verify with cargo

rust-analyzer floods stale method-not-found diagnostics after a build — trust cargo, not the LSP panel

After a subagent adds a method/type and rebuilds, the editor LSP panel can emit a
flood of `E0599 no method named ...` / `E0432 unresolved import ...` against *both*
the new and pre-existing test files — all false. The rust-analyzer index lags the
on-disk build.

Symptom seen in SL-036 PHASE-05: ~30 diagnostics across `tests/explain.rs`,
`tests/channels.rs`, `tests/resolution.rs` claiming `explain`/`evaluate`/`provenance`
not found, immediately after the explain verb landed and tests passed.

**How to apply:** never trust the diagnostics panel as ground truth after a build.
Confirm with `cargo test -p <crate> --no-run` (or `cargo check -p <crate>`). If that
compiles clean, the diagnostics are stale — proceed. Only treat LSP errors as real
when cargo agrees.
