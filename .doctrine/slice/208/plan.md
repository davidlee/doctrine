# Plan SL-208: Consistent cozy-table help for subcommand --help screens

## Sequencing Rationale

Three phases, linear dependency chain:

**PHASE-01: Renderer first.** The pure rendering functions (`render_subcommand_help`,
`render_options_section`) are the foundation. They are pure functions over the clap
command tree — testable in isolation without touching `main.rs`. Written TDD-style:
unit tests first (red), implementation (green), refactor. The `textwrap` Cargo.toml
dependency is added here since the options section needs it.

**PHASE-02: Wire the intercept.** Extend `main.rs`'s `DisplayHelp` error arm to call
the renderer. This is the risky change — it touches the binary entrypoint, error
handling, and exit semantics. The black-box integration tests exercise the actual
binary to catch regressions in the intercept path (exit codes, plain mode, nested
help tokens, MissingSubcommand contract). Existing snapshot tests updated to match
the new output format.

**PHASE-03: Gate.** Final lint/format/regression pass. Clippy zero warnings on
`--bin doctrine`. All pre-existing tests verified green. Visual smoke test of the
worst offenders (`worktree`, `dispatch`, `revision`).

## Boundaries

- Phase boundaries are file-based: PHASE-01 touches `cli.rs` + `Cargo.toml`,
  PHASE-02 touches `main.rs` + `tests/`, PHASE-03 is read-only verification.
- No cross-phase coupling — the renderer can be tested in PHASE-01 before the
  intercept exists, and the intercept in PHASE-02 only calls already-tested functions.
- The `listing.rs` file is NOT modified — we use `render_table` directly (as-is)
  for the subcommands table, and the options section is a standalone layout.

## Risks

- **clap API stability**: `render_usage().to_string()` ANSI behavior may vary by
  clap version. Mitigated by VT-7 (ANSI-stripping test) and the black-box
  `--color never` test.
- **Snapshot churn**: the updated help format changes the output of 4 existing
  `help_snapshot_*` tests. Mitigated by updating them in PHASE-02 and verifying
  no other tests break in PHASE-03.
- **Edge case coverage**: nested `help` tokens (`doctrine worktree help provision
  --help`), `--color` before subcommand, depth-2 commands. Covered by specific
  unit tests (VT-6) and integration tests (VT-4, VT-6).
