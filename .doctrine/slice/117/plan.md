# SL-117 Implementation Plan

## Rationale

A single phase. The three edits (dispatch_config.rs field + tests, dtoml.rs test,
dispatch/SKILL.md prose) are tightly coupled — the config field, its tests, and
the prose that consumes it form one coherent change. Splitting them would create
intermediate states where a field exists with no routing prose, or routing prose
points at a field that doesn't yet exist.

## Sequencing

PHASE-01 does everything in one TDD pass:

1. **Red** — write tests first (dispatch_config.rs + dtoml.rs)
2. **Green** — add the field to `DispatchConfig`, update `dispatch/SKILL.md`
3. **Refactor** — `cargo clippy`, review for dead code / lint

No dependencies on other slices. IMP-101's `preferred-subprocess-harness` wiring
in dispatch-subprocess is a separate concern — SL-117's routing prose names a
concrete `pi` fallback so the system degrades gracefully until that work lands.

## Phase summary

| Phase | Name | Files |
|---|---|---|
| PHASE-01 | Add claude-force-subprocess-dispatch key and update routing prose | `src/dispatch_config.rs`, `src/dtoml.rs`, `.agents/skills/dispatch/SKILL.md` |
