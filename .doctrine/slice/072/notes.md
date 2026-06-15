# SL-072 Notes — Dispatch Progress

## PHASE-01–05 (completed via serial dispatch)

5 phases landed on coordination branch `dispatch/072` at `d7f534d`:
- `e071ac5` PHASE-01: error model, core types, Cargo.toml
- `ec08f3a` PHASE-02: asset serving (rust-embed)
- `333935d` PHASE-03: entity markdown lookup
- `f743942` PHASE-04: Graphviz bridge
- `d7f534d` PHASE-05: HTTP routes + integration tests

## Surprises / Adaptations
- `tokio::fs` required adding `"fs"` to tokio features in Cargo.toml (PHASE-03)
- `RealDotRenderer` needs `"process"` + `"io-util"` tokio features (PHASE-04)
- `async-trait` crate added in PHASE-01 (used by DotRenderer trait)
- `which = "7"` added to `[dev-dependencies]` for conditional dot tests (PHASE-04)
- `#[expect(dead_code)]` unfulfilled in tests — use `#[allow(dead_code)]` for items consumed later
- `catalog::test_helpers` visibility widened to `pub(crate)` in PHASE-05 (needed by route integration tests)

## Remaining
- PHASE-06: CLI entry + server startup (commands/map.rs, open.rs, main.rs edit)
- PHASE-07: Browser placeholder (HTML/JS/CSS)
- PHASE-08: Gate (clippy, fmt, just gate, design conformance)

## Worker Worktrees
`.worktrees/w-072-01` through `w-072-05` — spent, GC after close.
