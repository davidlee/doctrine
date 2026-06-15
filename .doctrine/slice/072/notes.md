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

## PHASE-06–08 (completed via dispatch continuation)

3 phases landed on `dispatch/072` at `b754770`:
- `eba1d00` PHASE-06: CLI entry + server startup + URL construction + browser open
- `a9cb25e` PHASE-07: Browser placeholder SPA with entity list, markdown rendering, DOT editor
- `526564a` PHASE-08 + audit: health version fix + RV-035 synthesis
- `b754770` close: reconcile → done

## Audit (RV-035)
- 2 findings: F-1 (minor, follow-up — missing --path flag), F-2 (minor, fix-now — health dot version)
- F-2 fixed inline then verified; F-1 deferred to follow-up
- Design conformance confirmed: all §1-12 requirements met

## Standout Implementation Decisions
- `run_serve` is sync `fn` (spawns its own tokio runtime) — correct since main() dispatch is sync
- `RealDotRenderer` struct lives in `state.rs`, not `shell.rs` (established in PHASE-01)
- Browser SVG rendered via `<img>` data-uri — no inline injection (XSS prevention)

## Worker Worktrees
`.worktrees/w-072-01` through `w-072-07` — spent, GC after close.
