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
