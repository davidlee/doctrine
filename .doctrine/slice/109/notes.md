# SL-109 notes

## 2026-06-19 — Inquisition complete (RV-092)

Formal hostile pass via `/inquisition` against `design.md`. 5 findings, all
`fix-now` and verified terminal. No blockers.

### RV-092 findings summary

| Finding | Severity | Title |
|---------|----------|-------|
| F-1 | major | Error mapping from anyhow::Error by string-parsing is brittle |
| F-2 | minor | Showed variant computes both formats unconditionally |
| F-3 | minor | Status cache_verdict as Option<String> is ambiguous |
| F-4 | minor | print_review() output contract unspecified |
| F-5 | minor | Primed variant carries confused stale field |

### Design claims that survived

- `ReviewOutput` enum structure correct at the architectural level
- `with_turn` generic over `T` — sound, only `run_raise` needs `T != ()`
- Hand-rolled MCP protocol — tokio `io-util` already in Cargo.toml
- No parallel implementation — MCP handlers call `run_*`, not engine internals
- ADR-007 lock/baton/CAS honoured through MCP path
- Zero test impact — `.unwrap()` silently discards new return type
- `ListRow` struct matches design claims

### Next

Apply the five `fix-now` prescriptions to `design.md`, then proceed to `/plan`.

## 2026-06-19 — PHASE-04 complete (commit fd39f580)

Integration tests for the MCP stdio server. 9 VT criteria satisfied.

### What was done

- `tests/e2e_mcp_server.rs` — 9 integration tests spawning `doctrine serve --mcp` as subprocess, driving JSON-RPC 2.0 over stdio, verifying authored state on disk
- Fixed three `ReviewError` propagation gaps in `src/review.rs` where `anyhow::bail!` was used instead of `ReviewError::RoleMismatch`/`ReviewError::StateMismatch` variants
- Added `required_for(verb) -> FindingStatus` helper for the `StateMismatch` `required` field

### Surprises

- `ServeArgs` uses `--path` (long form only), unlike other commands that accept `-p`
- `ReviewOutput` is externally-tagged enum serialization: `{"Created":{"id":1,...}}` not `{"id":1,...}`
- Shared `CARGO_TARGET_DIR` causes stale binary artifacts in integration tests — must `touch` source files before running

### Follow-ups

- `ReviewError::LockContention` is still not used — `lock_guard.rs` uses `anyhow::bail!` for lock contention. Consider converting to `ReviewError::LockContention` in a follow-up
- `ReviewError::DanglingRef` is not used — the dangling target check in `run_new` uses `anyhow::bail!`
- Test `dispatch_router_skill_is_shrunk` in `e2e_skills_dispatch_shrinkage` is pre-existing red (unrelated)

### Gate status

- `cargo test --test e2e_mcp_server` — 9/9 pass
- `cargo test --bin doctrine -- review::` — 74/74 pass (behaviour-preserving)
- `cargo clippy` — zero warnings
- `cargo fmt` — clean
