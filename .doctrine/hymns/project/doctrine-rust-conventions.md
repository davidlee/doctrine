This repo IS doctrine, dogfooding itself: a Rust workspace whose worker habits
concretise the generic role/worker contract. The universal rules (negative
contract, hermetic goldens, component-anchored path scoping, function homes,
verify-as-you-go) already reached you from the Framework `role/worker` hymn —
this overlay only names the doctrine-specific *concretisations* the Framework
hymn deliberately leaves abstract.

## Build & test

- `cargo build` → the dev binary is `./target/debug/doctrine` (off-PATH; the
  installed `~/.cargo/bin/doctrine` is read-only in the jail).
- Focused tests: `cargo test --bin doctrine <filter>` — NOT `--lib`. Integration
  binaries under `tests/` are separate targets (`cargo test --test <name>`); a
  `--bin doctrine` run does NOT include them, so a change touching `tests/**`
  must run that binary explicitly.
- Each worktree builds into its OWN in-tree `target/` (cargo default — no shared
  `CARGO_TARGET_DIR`); no two worktrees thrash a shared cache.

## Format & lint (the owned cadence)

- Prefer the framework's own check verbs: `doctrine check quick` after each edit,
  `doctrine check commit` before handback, `doctrine check gate` for the full
  pre-commit gate. These resolve argv from owned config — no host build tool
  assumed.
- Under the hood: `cargo fmt`; `cargo clippy --bin doctrine` at ZERO warnings.
  Do NOT pass `--all-targets` — it turns on the `unwrap_used` / `expect_used`
  denials that are legitimate in test code, producing false failures.

## Module homes — ADR-001 layering

Layering is `leaf ← engine ← command`, no cycles. A leaf module depends on
nothing above it; each layer depends only downward. Concretely: `commands/*` is
the command layer; `install`/`boot` are engine; leaf modules (`hymns`, `fsutil`,
`root`) depend on nothing above. When you state a new function's home, state it
in these terms. The `tests/architecture_layering.rs` gate enforces this — run it
(`cargo test --test architecture_layering`, or `doctrine check gate`) after any
new module or cross-module move; a new top-level module must be indexed there.

## Pure / imperative split

No clock, rng, git, or disk in the pure layer — pass them in as inputs (the
date/uid pattern). Impurity lives in the thin shell. When changing shared
machinery (the entity engine), the existing suites are the behaviour-preservation
proof: they must stay green unchanged.
