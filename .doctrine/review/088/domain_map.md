# SL-107 reconciliation — domain_map

Port surface: hand-port of `candidate/101/review-001` onto main (base `5e8b2a64` → HEAD `1e382acd`), 6 files / +436.

## Areas
- src/value.rs — NEW pure leaf (ADR-001): ValueFacet/ValueConfig, single finite f64, present-facet validation, unit magic_beans; tests V1–V7 + deserialize
- src/main.rs — `mod value;`
- src/dtoml.rs — estimation/value `#[serde(default)]` config fields + module doc + tests
- src/slice.rs — SliceDoc +2 Option facet fields, Eq dropped, fixtures updated, round-trip + malformed tests
- src/estimate.rs — blanket `#![allow(dead_code)]` → item-level `expect(dead_code)` (5) + 6th on `pub(crate) mod display;`
- install/doctrine.toml.example — commented [estimation]/[value] sections
- .doctrine/slice/107/design.md — D1 (dead-code), D2 (narrow boundary), D3 (port-not-merge), §6 verification
- .doctrine/slice/107/plan.toml — EX-4 (5 expects), EN/EX/VT criteria

## Invariants
- Behaviour-preservation (AGENTS.md): existing suites green unchanged; only additive +20
- D1 dead-code tripwire: expect not allow; plain clippy clean AND no expect fires; unfulfilled = scope breach
- D2 narrow boundary: display→SL-102, graph→SL-103; SL-107 ships data only (parsed not rendered, VA-1)
- D3 port-not-merge: candidate is reference, no branch merge
- Contract authority: PRD-014/SPEC-020 (REV-002) authoritative, unchanged by this slice
- ADR-001 leaf purity: no clock/disk/rng/git in value.rs/estimate.rs

## Risks
- EX-4 says "exactly 5" expects; SL-102's display.rs landed post-plan → 6th surface (F-1)
- SliceDoc shape change feeds corpus-walk oracles → full gate is the catch, not per-suite
- expect(dead_code) over-fires if a helper becomes unexpectedly live → scope breach
